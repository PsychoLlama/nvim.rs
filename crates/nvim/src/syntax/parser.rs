//! The parser's state, [`SynState`], and the one way to it, [`with_parser`].
//!
//! Upstream keeps the parser in some thirty file-scope statics -- where it
//! is (`current_lnum`/`current_col`), what it is inside (`current_state`),
//! what it found ahead (`next_match_*`), what it decided (`current_attr`
//! and friends) and what it is parsing (`syn_win`/`syn_buf`/`syn_block`).
//! Here they are the fields of one value, and the steps of the parse --
//! [`state`](super::state)'s per-line driver, [`attr`](super::attr)'s
//! per-column walk, [`items`](super::items)' stack operations,
//! [`endpos`](super::endpos)' end search and [`sync`](super::sync)'s
//! backward scan -- are its methods, so they take it by `&mut` and the
//! buffer being parsed comes with it.
//!
//! # Re-entry
//!
//! The parser runs no user code of its own: `'syntax'` autocommands come
//! from `:syntax` commands, and `synID()` and friends call *into* it and run
//! it to completion. But it emits messages -- a pattern error from the
//! matcher, `'redrawtime'` running out -- and a message can flush pending
//! output to a Lua `ui_attach` handler, which can call `synID()`. So no
//! borrow of the cell is held across a parse: [`with_parser`] moves the
//! state out for the call and back after, and a nested call finds the cell
//! empty and parses with a fresh state of its own. See there.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use super::{ExtMatchRef, StateItem, SynBlockRef, SynFlags};
use crate::global_cell::GlobalCell;
use crate::types::{ColNr, LPos, LineNr, ProfTime, VarNumber, int16_t};
use crate::winlayer::{Buf, BufId, WinId};

/// The match [`SynState::current_attr`] found ahead of the current column and
/// has not pushed yet.
#[derive(Clone)]
pub(crate) struct NextMatch {
    /// Its pattern; -1 for "not looked yet".
    pub(crate) idx: c_int,
    /// The column it starts at; `MAXCOL` for "nothing found in this line".
    pub(crate) col: c_int,
    /// Where the match ends, and where its highlighting starts and ends.
    pub(crate) m_endpos: LPos,
    pub(crate) h_startpos: LPos,
    pub(crate) h_endpos: LPos,
    /// For a region: where its start match ends, and where the end match's
    /// own highlighting ends.
    pub(crate) eos_pos: LPos,
    pub(crate) eoe_pos: LPos,
    /// The end pattern's index when it has a `matchgroup=`, else 0.
    pub(crate) end_idx: c_int,
    pub(crate) flags: SynFlags,
    /// The start match's `\z(` captures.
    pub(crate) extmatch: Option<ExtMatchRef>,
}

/// What the last attribute walk decided about the current position. The
/// query API reads these back, so they outlive the call that set them.
pub(crate) struct CurrentAttr {
    /// Attribute number of the current character.
    pub(crate) attr: c_int,
    /// Syntax id of the current character, before and after transparency.
    pub(crate) id: c_int,
    pub(crate) trans_id: c_int,
    /// `HL_*` flags of the current character.
    pub(crate) flags: SynFlags,
    /// Sequence number of the item the current character belongs to, which
    /// is what tells two runs of the same group apart.
    pub(crate) seqnr: c_int,
    /// The `cchar=` of the current character, for `conceal`.
    pub(crate) sub_char: c_int,
}

impl CurrentAttr {
    /// Nothing: outside every item.
    pub(crate) const NONE: CurrentAttr = CurrentAttr {
        attr: 0,
        id: 0,
        trans_id: 0,
        flags: SynFlags::NONE,
        seqnr: 0,
        sub_char: 0,
    };
}

/// The syntax parser.
pub(crate) struct SynState {
    // --------------------------------------------------- what is parsed
    /// The window being parsed for. An identity, not an address: it outlives
    /// the call that set it, and the next [`Win::syntax_start`] compares it to
    /// decide whether the state still applies.
    ///
    /// [`Win::syntax_start`]: crate::winlayer::Win::syntax_start
    pub(crate) win: Option<WinId>,
    /// The buffer being parsed, on [`Self::win`]'s terms.
    pub(crate) buf: Option<BufId>,
    /// That buffer as a handle, resolved by the entry point of the current
    /// call -- the per-column path must not pay a registry lookup per
    /// character. Not to be read outside a call.
    pub(crate) buffer: Buf,
    /// The syntax block being parsed -- the window's, which for `:ownsyntax`
    /// is not the buffer's. `None` until the first start.
    pub(crate) parsed: Option<SynBlockRef>,
    /// The buffer's change tick at the last start: a change may have
    /// invalidated the state, so it counts as part of the buffer's identity.
    pub(crate) changedtick: VarNumber,
    /// When parsing must give up ('redrawtime'), or `None` for no limit.
    pub(crate) deadline: Option<ProfTime>,

    // ------------------------------------------------------ where it is
    /// The line being parsed.
    pub(crate) lnum: LineNr,
    /// The column being parsed.
    pub(crate) col: ColNr,
    /// Whether the state at [`Self::lnum`] has been put in the cache.
    pub(crate) state_stored: bool,
    /// Whether the line has been parsed to its end.
    pub(crate) finished: bool,
    /// Counts the lines parsed, so a pattern can tell that what it
    /// remembered of a line (`sp_line_id`) is about this one.
    pub(crate) line_id: c_int,

    // ---------------------------------------------- what it is inside
    /// The state stack, outermost item first: what the parser is inside at
    /// [`Self::col`]. Empty while invalid.
    pub(crate) stack: Vec<StateItem>,
    /// Whether the stack describes a real position: upstream's
    /// `VALID_STATE`, which it keeps in the growarray's `ga_itemsize`.
    pub(crate) stack_valid: bool,
    /// Stack index of the outermost `keepend` item in effect, or -1.
    pub(crate) keepend_level: c_int,
    /// The sequence number the next pushed item gets.
    pub(crate) next_seqnr: c_int,
    /// The `nextgroup=` list in effect, or null: borrowed from the pattern,
    /// keyword or item that set it.
    pub(crate) next_list: *mut int16_t,
    /// The `skipwhite`/`skipnl`/`skipempty` flags that came with it.
    pub(crate) next_flags: SynFlags,

    // --------------------------------------------------- what it found
    /// The match found ahead of the current column.
    pub(crate) next_match: NextMatch,
    /// The previous column found a match it could not use (an empty one,
    /// or one already matched there), so the next must look again.
    pub(crate) try_next_column: bool,
    /// What the last attribute walk decided.
    pub(crate) current: CurrentAttr,
}

impl SynState {
    /// A parser that has never started.
    pub(crate) fn new() -> SynState {
        let zero = LPos { lnum: 0, col: 0 };
        SynState {
            win: None,
            buf: None,
            buffer: Buf::NULL,
            parsed: None,
            changedtick: 0,
            deadline: None,
            lnum: 0,
            col: 0,
            state_stored: false,
            finished: false,
            line_id: 0,
            stack: Vec::new(),
            stack_valid: false,
            keepend_level: -1,
            next_seqnr: 1,
            next_list: ::core::ptr::null_mut(),
            next_flags: SynFlags::NONE,
            next_match: NextMatch {
                idx: 0,
                col: 0,
                m_endpos: zero,
                h_startpos: zero,
                h_endpos: zero,
                eos_pos: zero,
                eoe_pos: zero,
                end_idx: 0,
                flags: SynFlags::NONE,
                extmatch: None,
            },
            try_next_column: false,
            current: CurrentAttr::NONE,
        }
    }

    /// The syntax block being parsed, which during a `:syntax` command is
    /// not necessarily [`cur_syn_block`](super::cur_syn_block), the one
    /// being configured.
    ///
    /// # Panics
    ///
    /// Before the first start -- a parse step with no parse under way.
    #[inline]
    pub(crate) fn block(&self) -> SynBlockRef {
        self.parsed.expect("syntax_start named a block")
    }
}

/// The parser, between calls. `None` only while a call holds it.
static PARSER: GlobalCell<Option<Box<SynState>>> = GlobalCell::new(None);

/// Run `f` on the parser.
///
/// The state is moved out of its cell for the call and back after, so `f`
/// holds it by `&mut` with no borrow of the cell open. A call that re-enters
/// -- a message the parse emits reaching a Lua handler that asks for
/// `synID()` -- finds the cell empty and parses with a fresh state of its
/// own, which the outer call's put-back then replaces: neither sees the
/// other's half-done state, where upstream's statics would have been
/// clobbered under the outer parse. Moving a `Box` is a pointer swap, which
/// is what lets the per-column entry points pay for it.
#[inline]
pub(crate) fn with_parser<R>(f: impl FnOnce(&mut SynState) -> R) -> R {
    let mut parser = PARSER.take().unwrap_or_else(fresh_parser);
    let answer = f(&mut parser);
    let nested = PARSER.with_mut(|slot| slot.replace(parser));
    if nested.is_some() {
        drop_nested(nested);
    }
    answer
}

/// A parser for the first call, or for one nested inside another.
#[cold]
#[inline(never)]
fn fresh_parser() -> Box<SynState> {
    Box::new(SynState::new())
}

/// Drop what a nested call left in the cell. Out of line, so the usual
/// empty slot costs the caller a test and no registers.
#[cold]
#[inline(never)]
fn drop_nested(nested: Option<Box<SynState>>) {
    drop(nested);
}

/// Set the time limit for parsing ('redrawtime'), or clear it with `None`.
pub(crate) fn syn_set_timeout(deadline: Option<ProfTime>) {
    with_parser(|parser| parser.deadline = deadline);
}
