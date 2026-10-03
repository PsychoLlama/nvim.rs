//! The public query API, and command-line completion.
//!
//! `synID()`, `synstack()`, `synIDattr()`, `foldlevel()` for
//! `'foldmethod'=syntax` and the `:syntax`/`:echohl` completions all answer from
//! here. Everything in this module reads state the rest of the family produced;
//! nothing here parses.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::charset::skip;
use core::ffi::{CStr, c_int};

use super::*;
use crate::pos::MAXCOL;
use crate::types::{Candidate, ExpandContext};
use std::ffi::CString;

/// Does this window's block define any syntax at all?
pub(crate) fn syntax_present(win: Win) -> bool {
    let block = win.syntax();
    !block.b_syn_patterns.is_empty()
        || !block.b_syn_clusters.is_empty()
        || block.b_keywtab.ht_used > 0
        || block.b_keywtab_ic.ht_used > 0
}

/// What the next `get_syntax_name` call should offer, which
/// `set_context_in_syntax_cmd` decides from the part of the command already
/// typed.
#[derive(Copy, Clone, PartialEq, Eq)]
enum ExpandWhat {
    /// `:syntax` subcommand names.
    SubCmd,
    /// `:syntax case` arguments.
    Case,
    /// `:syntax spell` arguments.
    Spell,
    /// `:syntax sync` arguments.
    Sync,
    /// `:syntax list @cluster` arguments.
    Cluster,
}

static EXPAND_WHAT: GlobalCell<ExpandWhat> = GlobalCell::new(ExpandWhat::SubCmd);

/// Done expanding: forget what `:highlight` completion was asked to include.
pub(crate) fn reset_expand_highlight() {
    include_none.set(0);
    include_default.set(0);
    include_link.set(0);
}

/// Command-line completion for `:match` and `:echohl`: highlight group names,
/// plus `None`. The argument starts at `arg` in the completion's line.
pub(crate) fn set_context_in_echohl_cmd(expand: &mut Expand, arg: usize) {
    expand.context = ExpandContext::Highlight;
    expand.pattern = arg;
    include_none.set(1);
}

/// Command-line completion for `:syntax`, whose argument starts at `arg` in
/// the completion's line.
pub(crate) fn set_context_in_syntax_cmd(expand: &mut Expand, arg: usize) {
    // Default: expand subcommands.
    expand.context = ExpandContext::Syntax;
    EXPAND_WHAT.set(ExpandWhat::SubCmd);
    expand.pattern = arg;
    include_link.set(0);
    include_default.set(0);

    let line = expand.line_cstr().to_bytes().to_vec();
    let at = |i: usize| line.get(i).copied().unwrap_or(0);
    let skipwhite = |i: usize| i + skip::white(line.get(i..).unwrap_or_default());
    let skiptowhite = |i: usize| i + skip::to_white(line.get(i..).unwrap_or_default());
    if at(arg) == 0 {
        return;
    }

    // (Part of) the subcommand has been typed.
    let mut p = skiptowhite(arg);
    if at(p) == 0 {
        return;
    }

    // Past the first word.
    expand.pattern = skipwhite(p);
    let word = &line[arg..p];
    let first_word_is = |name: &CStr| word.eq_ignore_ascii_case(name.to_bytes());

    if at(skiptowhite(expand.pattern)) != 0 {
        expand.context = ExpandContext::Nothing;
    } else if first_word_is(c"case") {
        EXPAND_WHAT.set(ExpandWhat::Case);
    } else if first_word_is(c"spell") {
        EXPAND_WHAT.set(ExpandWhat::Spell);
    } else if first_word_is(c"sync") {
        EXPAND_WHAT.set(ExpandWhat::Sync);
    } else if first_word_is(c"list") {
        p = skipwhite(p);
        if at(p) == b'@' {
            EXPAND_WHAT.set(ExpandWhat::Cluster);
        } else {
            expand.context = ExpandContext::Highlight;
        }
    } else if first_word_is(c"keyword") || first_word_is(c"region") || first_word_is(c"match") {
        expand.context = ExpandContext::Highlight;
    } else {
        expand.context = ExpandContext::Nothing;
    }
}

/// The arguments `:syntax case` takes.
const CASE_ARGS: [&CStr; 2] = [c"match", c"ignore"];
/// The arguments `:syntax spell` takes.
const SPELL_ARGS: [&CStr; 3] = [c"toplevel", c"notoplevel", c"default"];
/// The arguments `:syntax sync` takes.
const SYNC_ARGS: [&CStr; 10] = [
    c"ccomment",
    c"clear",
    c"fromstart",
    c"linebreaks=",
    c"linecont",
    c"lines=",
    c"match",
    c"maxlines=",
    c"minlines=",
    c"region",
];

/// `expand_generic`'s callback: the `idx`th completion candidate, or `None`
/// past the end.
pub(crate) fn get_syntax_name(_expand: &Expand, idx: usize) -> Option<Candidate> {
    let nth = |names: &[&'static CStr]| names.get(idx).map(|&name| Candidate::Borrowed(name));
    match EXPAND_WHAT.get() {
        ExpandWhat::SubCmd => SUBCOMMANDS
            .get(idx)
            .map(|sub| Candidate::Borrowed(sub.name)),
        ExpandWhat::Case => nth(&CASE_ARGS),
        ExpandWhat::Spell => nth(&SPELL_ARGS),
        ExpandWhat::Sync => nth(&SYNC_ARGS),
        ExpandWhat::Cluster => {
            if c_int::try_from(idx).ok()? >= cur_cluster_count() {
                return None;
            }
            let block = cur_syn_block();
            let name = &block.clusters()[idx].scl_name;
            let mut text = Vec::with_capacity(name.count_bytes() + 1);
            text.push(b'@');
            text.extend_from_slice(name.to_bytes());
            Some(Candidate::Owned(
                CString::new(text).expect("a cluster name holds no NUL"),
            ))
        }
    }
}

impl Win {
    /// The syntax id at a buffer position, for expression evaluation.
    ///
    /// `trans` removes transparency; `spellp` answers whether spell checking
    /// applies there; `keep_state` keeps the state of the character at `col`
    /// so that [`syn_get_stack_item`] can be asked about it afterwards.
    pub(crate) fn syntax_id(
        self,
        lnum: LineNr,
        col: ColNr,
        trans: bool,
        spellp: Option<&mut bool>,
        keep_state: bool,
    ) -> c_int {
        with_parser(|parser| {
            // Parsing has to restart unless this position is at or after the
            // current one, in the same line of the same window and buffer.
            if parser.win != Some(self.id())
                || parser.buf != self.buffer().try_id()
                || lnum != parser.lnum
                || col < parser.col
            {
                parser.start(self, lnum);
            } else if col > parser.col {
                // `next_match` may be wrong when moving around, e.g. with the
                // "skip" expression of `searchpair()`.
                parser.next_match.idx = -1;
            }

            parser.buffer = self.buffer();
            parser.attr_at(col, spellp, keep_state);
            if trans {
                parser.current.trans_id
            } else {
                parser.current.id
            }
        })
    }
}

/// Extra information about the current syntax item: answers its flags and
/// its sequence number. Must be called right after [`get_syntax_attr`].
pub(crate) fn get_syntax_info() -> (SynFlags, c_int) {
    with_parser(|parser| (parser.current.flags, parser.current.seqnr))
}

/// The conceal substitution character of the current item.
pub(crate) fn syn_get_sub_char() -> c_int {
    with_parser(|parser| parser.current.sub_char)
}

/// The syntax id at position `i` of the state stack, or -1 when `i` is out of
/// range.
///
/// The caller must have called [`Win::syntax_id`] first, to fill the stack.
pub(crate) fn syn_get_stack_item(i: c_int) -> c_int {
    with_parser(|parser| {
        if i >= parser.state_len() {
            // The state was not properly finished for the last character
            // (`keep_state` was true), so it has to be invalidated.
            parser.invalidate_current_state();
            parser.col = MAXCOL as ColNr;
            return -1;
        }
        parser.item(i).si_id
    })
}

impl SynState {
    /// How many `fold` items are open at the current position.
    fn cur_foldlevel(&self) -> c_int {
        self.stack
            .iter()
            .filter(|si| si.si_flags.has(SynFlags::FOLD))
            .count() as c_int
    }
}

/// The fold level of line `lnum`, for `'foldmethod'=syntax`.
pub(crate) fn syn_get_foldlevel(window: Win, lnum: LineNr) -> c_int {
    let mut level = 0;

    // Answer quickly when there are no fold items at all.
    let block = window.syntax();
    if block.b_syn_folditems != 0 && !block.b_syn_error && !block.b_syn_slow {
        level = with_parser(|parser| {
            parser.start(window, lnum);

            // Start with the fold level at the start of the line.
            let mut level = parser.cur_foldlevel();

            if block.b_syn_foldlevel == SYNFLD_MINIMUM {
                // Find the lowest fold level that is followed by a higher
                // one.
                let mut low_level = level;
                while !parser.finished {
                    parser.current_attr(false, false, None, false);
                    let cur_level = parser.cur_foldlevel();
                    if cur_level < low_level {
                        low_level = cur_level;
                    } else if cur_level > low_level {
                        level = low_level;
                    }
                    parser.col += 1;
                }
            }
            level
        });
    }

    if level as OptInt > window.w_onebuf_opt.wo_fdn {
        level = window.w_onebuf_opt.wo_fdn as c_int;
        if level < 0 {
            level = 0;
        }
    }
    level
}
