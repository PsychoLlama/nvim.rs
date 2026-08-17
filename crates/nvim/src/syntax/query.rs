//! The public query API, and command-line completion.
//!
//! `synID()`, `synstack()`, `synIDattr()`, `foldlevel()` for
//! `'foldmethod'=syntax` and the `:syntax`/`:echohl` completions all answer from
//! here. Everything in this module reads state the rest of the family produced;
//! nothing here parses.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};

#[allow(unused_imports)]
use super::*;
use crate::pos::MAXCOL;

/// Does this window's block define any syntax at all?
pub unsafe fn syntax_present(win: *mut win_T) -> bool {
    unsafe {
        (*(*win).w_s).b_syn_patterns.ga_len != 0
            || (*(*win).w_s).b_syn_clusters.ga_len != 0
            || (*(*win).w_s).b_keywtab.ht_used > 0
            || (*(*win).w_s).b_keywtab_ic.ht_used > 0
    }
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
pub unsafe fn reset_expand_highlight() {
    include_none.set(0);
    include_default.set(0);
    include_link.set(0);
}

/// Command-line completion for `:match` and `:echohl`: highlight group names,
/// plus `None`.
pub unsafe fn set_context_in_echohl_cmd(xp: *mut expand_T, arg: *const c_char) {
    unsafe {
        (*xp).xp_context = EXPAND_HIGHLIGHT;
        (*xp).xp_pattern = arg as *mut c_char;
        include_none.set(1);
    }
}

/// Command-line completion for `:syntax`.
pub unsafe fn set_context_in_syntax_cmd(xp: *mut expand_T, arg: *const c_char) {
    unsafe {
        // Default: expand subcommands.
        (*xp).xp_context = EXPAND_SYNTAX;
        EXPAND_WHAT.set(ExpandWhat::SubCmd);
        (*xp).xp_pattern = arg as *mut c_char;
        include_link.set(0);
        include_default.set(0);
        if *arg as c_int == NUL {
            return;
        }

        // (Part of) the subcommand has been typed.
        let mut p = skiptowhite(arg);
        if *p as c_int == NUL {
            return;
        }

        // Past the first word.
        (*xp).xp_pattern = skipwhite(p);
        let word_len = p.offset_from(arg) as size_t;
        let first_word_is = |name: &CStr| strncasecmp(arg, name.as_ptr(), word_len) == 0;

        if *skiptowhite((*xp).xp_pattern) as c_int != NUL {
            (*xp).xp_context = EXPAND_NOTHING;
        } else if first_word_is(c"case") {
            EXPAND_WHAT.set(ExpandWhat::Case);
        } else if first_word_is(c"spell") {
            EXPAND_WHAT.set(ExpandWhat::Spell);
        } else if first_word_is(c"sync") {
            EXPAND_WHAT.set(ExpandWhat::Sync);
        } else if first_word_is(c"list") {
            p = skipwhite(p);
            if *p as c_int == '@' as c_int {
                EXPAND_WHAT.set(ExpandWhat::Cluster);
            } else {
                (*xp).xp_context = EXPAND_HIGHLIGHT;
            }
        } else if first_word_is(c"keyword") || first_word_is(c"region") || first_word_is(c"match") {
            (*xp).xp_context = EXPAND_HIGHLIGHT;
        } else {
            (*xp).xp_context = EXPAND_NOTHING;
        }
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

/// `ExpandGeneric`'s callback: the `idx`th completion candidate, or NULL past
/// the end.
pub unsafe extern "C" fn get_syntax_name(xp: *mut expand_T, idx: c_int) -> *mut c_char {
    unsafe {
        let nth = |names: &[&CStr]| {
            usize::try_from(idx)
                .ok()
                .and_then(|i| names.get(i))
                .map_or(::core::ptr::null_mut(), |s| s.as_ptr().cast_mut())
        };
        match EXPAND_WHAT.get() {
            ExpandWhat::SubCmd => usize::try_from(idx)
                .ok()
                .and_then(|i| SUBCOMMANDS.get(i))
                .map_or(::core::ptr::null_mut(), |s| s.name.as_ptr().cast_mut()),
            ExpandWhat::Case => nth(&CASE_ARGS),
            ExpandWhat::Spell => nth(&SPELL_ARGS),
            ExpandWhat::Sync => nth(&SYNC_ARGS),
            ExpandWhat::Cluster => {
                if idx >= cur_cluster_count() {
                    return ::core::ptr::null_mut();
                }
                vim_snprintf(
                    &raw mut (*xp).xp_buf as *mut c_char,
                    EXPAND_BUF_LEN as size_t,
                    c"@%s".as_ptr(),
                    (*cur_cluster(idx)).scl_name,
                );
                &raw mut (*xp).xp_buf as *mut c_char
            }
        }
    }
}

/// The syntax id at a buffer position, for expression evaluation.
///
/// `trans` removes transparency; `spellp` answers whether spell checking
/// applies there; `keep_state` keeps the state of the character at `col` so
/// that [`syn_get_stack_item`] can be asked about it afterwards.
pub unsafe fn syn_get_id(
    wp: *mut win_T,
    lnum: linenr_T,
    col: colnr_T,
    trans: c_int,
    spellp: *mut bool,
    keep_state: c_int,
) -> c_int {
    unsafe {
        // Parsing has to restart unless this position is at or after the
        // current one, in the same line of the same window and buffer.
        if wp != syn_win.get()
            || (*wp).w_buffer != syn_buf.get()
            || lnum != current_lnum.get()
            || col < current_col.get()
        {
            syntax_start(wp, lnum);
        } else if col > current_col.get() {
            // `next_match` may be wrong when moving around, e.g. with the
            // "skip" expression of `searchpair()`.
            next_match_idx.set(-1);
        }

        get_syntax_attr(col, spellp, keep_state != 0);
        if trans != 0 {
            current_trans_id.get()
        } else {
            current_id.get()
        }
    }
}

/// Extra information about the current syntax item: answers its flags and
/// stores its sequence number. Must be called right after [`get_syntax_attr`].
pub unsafe fn get_syntax_info(seqnrp: *mut c_int) -> c_int {
    unsafe {
        *seqnrp = current_seqnr.get();
        current_flags.get()
    }
}

/// The conceal substitution character of the current item.
pub unsafe fn syn_get_sub_char() -> c_int {
    current_sub_char.get()
}

/// The syntax id at position `i` of the current state stack, or -1 when `i` is
/// out of range.
///
/// The caller must have called [`syn_get_id`] first, to fill the stack.
pub unsafe fn syn_get_stack_item(i: c_int) -> c_int {
    unsafe {
        if i >= state_len() {
            // The state was not properly finished for the last character
            // (`keep_state` was true), so it has to be invalidated.
            invalidate_current_state();
            current_col.set(MAXCOL as colnr_T);
            return -1;
        }
        (*state_at(i)).si_id
    }
}

/// How many `fold` items are open at the current position.
unsafe fn syn_cur_foldlevel() -> c_int {
    unsafe {
        let mut level = 0;
        for i in 0..state_len() {
            if (*state_at(i)).si_flags & HL_FOLD != 0 {
                level += 1;
            }
        }
        level
    }
}

/// The fold level of line `lnum`, for `'foldmethod'=syntax`.
pub unsafe fn syn_get_foldlevel(wp: *mut win_T, lnum: linenr_T) -> c_int {
    unsafe {
        let mut level = 0;

        // Answer quickly when there are no fold items at all.
        if (*(*wp).w_s).b_syn_folditems != 0
            && !(*(*wp).w_s).b_syn_error
            && !(*(*wp).w_s).b_syn_slow
        {
            syntax_start(wp, lnum);

            // Start with the fold level at the start of the line.
            level = syn_cur_foldlevel();

            if (*(*wp).w_s).b_syn_foldlevel == SYNFLD_MINIMUM {
                // Find the lowest fold level that is followed by a higher one.
                let mut low_level = level;
                while !current_finished.get() {
                    syn_current_attr(false, false, ::core::ptr::null_mut(), false);
                    let cur_level = syn_cur_foldlevel();
                    if cur_level < low_level {
                        low_level = cur_level;
                    } else if cur_level > low_level {
                        level = low_level;
                    }
                    current_col.set(current_col.get() + 1);
                }
            }
        }

        if level as OptInt > (*wp).w_onebuf_opt.wo_fdn {
            level = (*wp).w_onebuf_opt.wo_fdn as c_int;
            if level < 0 {
                level = 0;
            }
        }
        level
    }
}
