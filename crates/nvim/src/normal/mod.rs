//! Normal mode: the state loop, the command table, and the vocabulary the
//! thirteen command families share.
//!
//! The families are the modules below, grouped by what a command *does* to the
//! editor rather than by which key runs it -- which is the seam
//! [`NV_CMD_TABLE`] already draws, because every row of it names a handler.
//!
//! What is left in this file is the table, the two structures it is made of,
//! and the constants at least one family imports by name. Nothing here is
//! code.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::ascii::TAB;
use crate::edit::BeginlineOpts;
use crate::global_cell::GlobalCell;
use crate::keycodes::SpecialKey;
use crate::mark::defs::MarkMove;
use crate::mouse::{MSCR_DOWN, MSCR_LEFT, MSCR_RIGHT, MSCR_UP, nv_mouse, nv_mousescroll};
use crate::normal::defs::CmdArg;
use crate::os::libc::uint16_t;

use crate::types::{Key, Pos};
use core::ffi::{CStr, c_int, c_uint};

mod state;
pub(crate) use self::state::*;
mod dispatch;
pub(crate) use self::dispatch::*;
mod showcmd;
pub(crate) use self::showcmd::*;
mod visual;
pub(crate) use self::visual::*;
mod ident;
pub(crate) use self::ident::*;
mod identfind;
pub(crate) use self::identfind::*;
mod motion;
pub(crate) use self::motion::*;
mod search;
pub(crate) use self::search::*;
mod brackets;
pub(crate) use self::brackets::*;
mod scroll;
pub(crate) use self::scroll::*;
mod edit;
pub(crate) use self::edit::*;
mod operator;
pub(crate) use self::operator::*;
mod gcmd;
pub(crate) use self::gcmd::*;
use crate::search::{BACKWARD, FORWARD, SEARCH_REV};
pub(crate) mod defs;
mod misc;
pub(crate) use self::misc::*;
pub(crate) const K_MARK_NO_CONTEXT: MarkMove = 4;
/// Columns `'showcmd'` reserves at the right of the last line.
pub(crate) const SHOWCMD_COLS: c_int = 10;
pub(crate) const SHOWCMD_BUFLEN: c_uint = 41;
pub(crate) const FIND_EVAL: c_int = 4;
pub(crate) const FIND_STRING: c_int = 2;
pub(crate) const FIND_IDENT: c_int = 1;
#[derive(Debug)]
pub(crate) struct NvCmd {
    pub cmd_char: Key,
    pub cmd_func: NvFunc,
    pub cmd_flags: NvFlags,
    pub cmd_arg: c_int,
}
/// What a row of [`NV_CMD_TABLE`] runs.
///
/// Nothing outside this crate reaches the table or its handlers -- neither the
/// ABI ledger nor the unit-test cdefs name any of them -- so the handlers are
/// ordinary Rust functions rather than `extern "C"` ones.
///
/// A *safe* `fn`, which is what makes the table its own oracle: a safe `fn`
/// coerces to an `unsafe fn` pointer but not the other way round, so a row
/// still naming an `unsafe fn` fails to build and rustc says which one. And
/// not an `Option`: every row has a handler, and the one that means "nothing
/// to do" says so by name ([`nv_nop`]).
pub(crate) type NvFunc = fn(&mut CmdArg);
/// The normal-mode state machine's own state: one per running
/// `normal_enter` (lent to `state_enter` for the length of its loop) or
/// `normal_cmd` (on that call's stack).
///
/// The operator is not in here. `op_pending` reads the innermost frame's
/// operator from inside whatever a handler re-entered, while the handler
/// holds this state mutably, so the operator lives in the frame table
/// ([`Op`](crate::ops::Op), a handle to it) and this state only names it.
#[derive(Debug)]
pub(crate) struct NormalState {
    pub command_finished: bool,
    pub ctrl_w: bool,
    pub need_flushbuf: bool,
    pub set_prevcount: bool,
    pub previous_got_int: bool,
    pub cmdwin: bool,
    pub noexmode: bool,
    pub toplevel: bool,
    pub oa: crate::ops::Op,
    pub ca: CmdArg,
    pub mapped_len: c_int,
    pub old_mapped_len: c_int,
    /// The row of [`NV_CMD_TABLE`] the command resolved to.
    pub idx: Option<usize>,
    pub c: Key,
    pub old_col: c_int,
    pub old_pos: Pos,
}
static V_ISUAL_MODE_ORIG: GlobalCell<VisualMode> = GlobalCell::new(VisualMode::NONE);
const E_CHANGELIST_IS_EMPTY: &CStr = c"E664: Changelist is empty";
const E_CMDLINE_WINDOW_ALREADY_OPEN: &CStr = c"E1292: Command-line window is already open";
crate::flag_set! {
    /// What a row of [`NV_CMD_TABLE`] asks the dispatcher to do around the
    /// handler: whether to read more characters, how the key behaves under
    /// 'keymodel' and 'rightleft', and what happens to the register.
    ///
    /// The word is the row's own `uint16_t` field, so the type is what that
    /// field is.
    pub(crate) struct NvFlags: uint16_t;

    /// The command takes a second character. Never set on its own: one of
    /// the two below says *when* it is read.
    const NCH = 0x1;
    /// ... but only when no operator is pending.
    const NCH_NOP = 0x2 | Self::NCH.bits();
    /// ... always.
    const NCH_ALW = 0x4 | Self::NCH.bits();
    /// The second character is text rather than a command key, so it goes
    /// through 'langmap' and collects its combining characters.
    const LANG = 0x8;
    /// 'keymodel' startsel: a shifted special key starts a selection and
    /// then acts as its unshifted self.
    const SS = 0x10;
    /// As [`Self::SS`], for a key that is shifted by a modifier rather than
    /// by having a shifted twin.
    const SSS = 0x20;
    /// 'keymodel' stopsel: an unshifted movement ends the selection.
    const STS = 0x40;
    /// Left and right swap in a 'rightleft' window.
    const RL = 0x80;
    /// The register named in front of this command is *not* released after
    /// it: the command is the one that uses it.
    const KEEPREG = 0x100;
    /// Not allowed while the text or the current buffer is locked.
    const NCW = 0x200;
}

/// One row of [`NV_CMD_TABLE`].
///
/// The argument is written as the `c_int` constant that names it, so a row
/// reads as the four things it is rather than as a cast.
const fn cmd(cmd_char: Key, cmd_func: NvFunc, cmd_flags: NvFlags, cmd_arg: c_int) -> NvCmd {
    NvCmd {
        cmd_char,
        cmd_func,
        cmd_flags,
        cmd_arg,
    }
}

/// The Normal-mode command table, in source order.
///
/// A `const` so that [`NV_CMD_IDX`](crate::normal::NV_CMD_IDX) can be sorted
/// at compile time; `NV_CMD_TABLE` is the `static` everything indexes, so the rows
/// exist once.
pub(crate) const NV_CMDS: [NvCmd; 188] = [
    cmd(Key::NUL, nv_error, NvFlags::NONE, 0),
    cmd(Key::CTRL_A, nv_addsub, NvFlags::NONE, 0),
    cmd(Key::CTRL_B, nv_page, NvFlags::STS, BACKWARD as c_int),
    cmd(Key::CTRL_C, nv_esc, NvFlags::NONE, 1),
    cmd(Key::CTRL_D, nv_halfpage, NvFlags::NONE, 0),
    cmd(Key::CTRL_E, nv_scroll_line, NvFlags::NONE, 1),
    cmd(Key::CTRL_F, nv_page, NvFlags::STS, FORWARD as c_int),
    cmd(Key::CTRL_G, nv_ctrlg, NvFlags::NONE, 0),
    cmd(Key::CTRL_H, nv_ctrlh, NvFlags::NONE, 0),
    cmd(Key::CTRL_I, nv_pcmark, NvFlags::NONE, 0),
    cmd(Key::NL, nv_down, NvFlags::NONE, 0),
    cmd(Key::CTRL_K, nv_error, NvFlags::NONE, 0),
    cmd(Key::CTRL_L, nv_clear, NvFlags::NONE, 0),
    cmd(Key::CAR, nv_down, NvFlags::NONE, 1),
    cmd(Key::CTRL_N, nv_down, NvFlags::STS, 0),
    cmd(Key::CTRL_O, nv_ctrlo, NvFlags::NONE, 0),
    cmd(Key::CTRL_P, nv_up, NvFlags::STS, 0),
    cmd(Key::CTRL_Q, nv_visual, NvFlags::NONE, 0),
    cmd(Key::CTRL_R, nv_redo_or_register, NvFlags::NONE, 0),
    cmd(Key::CTRL_S, nv_ignore, NvFlags::NONE, 0),
    cmd(Key::CTRL_T, nv_tagpop, NvFlags::NCW, 0),
    cmd(Key::CTRL_U, nv_halfpage, NvFlags::NONE, 0),
    cmd(Key::CTRL_V, nv_visual, NvFlags::NONE, 0),
    cmd(Key::ascii(b'V'), nv_visual, NvFlags::NONE, 0),
    cmd(Key::ascii(b'v'), nv_visual, NvFlags::NONE, 0),
    cmd(Key::CTRL_W, nv_window, NvFlags::NONE, 0),
    cmd(Key::CTRL_X, nv_addsub, NvFlags::NONE, 0),
    cmd(Key::CTRL_Y, nv_scroll_line, NvFlags::NONE, 0),
    cmd(Key::CTRL_Z, nv_suspend, NvFlags::NONE, 0),
    cmd(Key::ESC, nv_esc, NvFlags::NONE, 0),
    cmd(Key::CTRL_BSL, nv_normal, NvFlags::NCH_ALW, 0),
    cmd(Key::CTRL_RSB, nv_ident, NvFlags::NCW, 0),
    cmd(Key::CTRL_HAT, nv_hat, NvFlags::NCW, 0),
    cmd(Key::CTRL_UNDERSCORE, nv_error, NvFlags::NONE, 0),
    cmd(Key::ascii(b' '), nv_right, NvFlags::NONE, 0),
    cmd(Key::ascii(b'!'), nv_operator, NvFlags::NONE, 0),
    cmd(
        Key::ascii(b'"'),
        nv_regname,
        NvFlags::NCH_NOP.or(NvFlags::KEEPREG),
        0,
    ),
    cmd(Key::ascii(b'#'), nv_ident, NvFlags::NONE, 0),
    cmd(Key::ascii(b'$'), nv_dollar, NvFlags::NONE, 0),
    cmd(Key::ascii(b'%'), nv_percent, NvFlags::NONE, 0),
    cmd(Key::ascii(b'&'), nv_optrans, NvFlags::NONE, 0),
    cmd(Key::ascii(b'\''), nv_gomark, NvFlags::NCH_ALW, 1),
    cmd(Key::ascii(b'('), nv_brace, NvFlags::NONE, BACKWARD as c_int),
    cmd(Key::ascii(b')'), nv_brace, NvFlags::NONE, FORWARD as c_int),
    cmd(Key::ascii(b'*'), nv_ident, NvFlags::NONE, 0),
    cmd(Key::ascii(b'+'), nv_down, NvFlags::NONE, 1),
    cmd(Key::ascii(b','), nv_csearch, NvFlags::NONE, 1),
    cmd(Key::ascii(b'-'), nv_up, NvFlags::NONE, 1),
    cmd(Key::ascii(b'.'), nv_dot, NvFlags::KEEPREG, 0),
    cmd(Key::ascii(b'/'), nv_search, NvFlags::NONE, 0),
    cmd(Key::ascii(b'0'), nv_beginline, NvFlags::NONE, 0),
    cmd(Key::ascii(b'1'), nv_ignore, NvFlags::NONE, 0),
    cmd(Key::ascii(b'2'), nv_ignore, NvFlags::NONE, 0),
    cmd(Key::ascii(b'3'), nv_ignore, NvFlags::NONE, 0),
    cmd(Key::ascii(b'4'), nv_ignore, NvFlags::NONE, 0),
    cmd(Key::ascii(b'5'), nv_ignore, NvFlags::NONE, 0),
    cmd(Key::ascii(b'6'), nv_ignore, NvFlags::NONE, 0),
    cmd(Key::ascii(b'7'), nv_ignore, NvFlags::NONE, 0),
    cmd(Key::ascii(b'8'), nv_ignore, NvFlags::NONE, 0),
    cmd(Key::ascii(b'9'), nv_ignore, NvFlags::NONE, 0),
    cmd(Key::ascii(b':'), nv_colon, NvFlags::NONE, 0),
    cmd(Key::ascii(b';'), nv_csearch, NvFlags::NONE, 0),
    cmd(Key::ascii(b'<'), nv_operator, NvFlags::RL, 0),
    cmd(Key::ascii(b'='), nv_operator, NvFlags::NONE, 0),
    cmd(Key::ascii(b'>'), nv_operator, NvFlags::RL, 0),
    cmd(Key::ascii(b'?'), nv_search, NvFlags::NONE, 0),
    cmd(Key::ascii(b'@'), nv_at, NvFlags::NCH_NOP, 0),
    cmd(Key::ascii(b'A'), nv_edit, NvFlags::NONE, 0),
    cmd(Key::ascii(b'B'), nv_bck_word, NvFlags::NONE, 1),
    cmd(Key::ascii(b'C'), nv_abbrev, NvFlags::KEEPREG, 0),
    cmd(Key::ascii(b'D'), nv_abbrev, NvFlags::KEEPREG, 0),
    cmd(Key::ascii(b'E'), nv_wordcmd, NvFlags::NONE, 1),
    cmd(
        Key::ascii(b'F'),
        nv_csearch,
        NvFlags::NCH_ALW.or(NvFlags::LANG),
        BACKWARD as c_int,
    ),
    cmd(Key::ascii(b'G'), nv_goto, NvFlags::NONE, 1),
    cmd(Key::ascii(b'H'), nv_scroll, NvFlags::NONE, 0),
    cmd(Key::ascii(b'I'), nv_edit, NvFlags::NONE, 0),
    cmd(Key::ascii(b'J'), nv_join, NvFlags::NONE, 0),
    cmd(Key::ascii(b'K'), nv_ident, NvFlags::NONE, 0),
    cmd(Key::ascii(b'L'), nv_scroll, NvFlags::NONE, 0),
    cmd(Key::ascii(b'M'), nv_scroll, NvFlags::NONE, 0),
    cmd(
        Key::ascii(b'N'),
        nv_next,
        NvFlags::NONE,
        SEARCH_REV as c_int,
    ),
    cmd(Key::ascii(b'O'), nv_open, NvFlags::NONE, 0),
    cmd(Key::ascii(b'P'), nv_put, NvFlags::NONE, 0),
    cmd(Key::ascii(b'Q'), nv_regreplay, NvFlags::NONE, 0),
    cmd(Key::ascii(b'R'), nv_replace_mode, NvFlags::NONE, 0),
    cmd(Key::ascii(b'S'), nv_subst, NvFlags::KEEPREG, 0),
    cmd(
        Key::ascii(b'T'),
        nv_csearch,
        NvFlags::NCH_ALW.or(NvFlags::LANG),
        BACKWARD as c_int,
    ),
    cmd(Key::ascii(b'U'), nv_undo_line, NvFlags::NONE, 0),
    cmd(Key::ascii(b'W'), nv_wordcmd, NvFlags::NONE, 1),
    cmd(Key::ascii(b'X'), nv_abbrev, NvFlags::KEEPREG, 0),
    cmd(Key::ascii(b'Y'), nv_abbrev, NvFlags::KEEPREG, 0),
    cmd(
        Key::ascii(b'Z'),
        nv_exit_command,
        NvFlags::NCH_NOP.or(NvFlags::NCW),
        0,
    ),
    cmd(
        Key::ascii(b'['),
        nv_brackets,
        NvFlags::NCH_ALW,
        BACKWARD as c_int,
    ),
    cmd(Key::ascii(b'\\'), nv_error, NvFlags::NONE, 0),
    cmd(
        Key::ascii(b']'),
        nv_brackets,
        NvFlags::NCH_ALW,
        FORWARD as c_int,
    ),
    cmd(
        Key::ascii(b'^'),
        nv_beginline,
        NvFlags::NONE,
        BeginlineOpts::WHITE.or(BeginlineOpts::FIX).bits(),
    ),
    cmd(Key::ascii(b'_'), nv_lineop, NvFlags::NONE, 0),
    cmd(Key::ascii(b'`'), nv_gomark, NvFlags::NCH_ALW, 0),
    cmd(Key::ascii(b'a'), nv_edit, NvFlags::NCH, 0),
    cmd(Key::ascii(b'b'), nv_bck_word, NvFlags::NONE, 0),
    cmd(Key::ascii(b'c'), nv_operator, NvFlags::NONE, 0),
    cmd(Key::ascii(b'd'), nv_operator, NvFlags::NONE, 0),
    cmd(Key::ascii(b'e'), nv_wordcmd, NvFlags::NONE, 0),
    cmd(
        Key::ascii(b'f'),
        nv_csearch,
        NvFlags::NCH_ALW.or(NvFlags::LANG),
        FORWARD as c_int,
    ),
    cmd(Key::ascii(b'g'), nv_g_cmd, NvFlags::NCH_ALW, 0),
    cmd(Key::ascii(b'h'), nv_left, NvFlags::RL, 0),
    cmd(Key::ascii(b'i'), nv_edit, NvFlags::NCH, 0),
    cmd(Key::ascii(b'j'), nv_down, NvFlags::NONE, 0),
    cmd(Key::ascii(b'k'), nv_up, NvFlags::NONE, 0),
    cmd(Key::ascii(b'l'), nv_right, NvFlags::RL, 0),
    cmd(Key::ascii(b'm'), nv_mark, NvFlags::NCH_NOP, 0),
    cmd(Key::ascii(b'n'), nv_next, NvFlags::NONE, 0),
    cmd(Key::ascii(b'o'), nv_open, NvFlags::NONE, 0),
    cmd(Key::ascii(b'p'), nv_put, NvFlags::NONE, 0),
    cmd(Key::ascii(b'q'), nv_record, NvFlags::NCH, 0),
    cmd(
        Key::ascii(b'r'),
        nv_replace,
        NvFlags::NCH_NOP.or(NvFlags::LANG),
        0,
    ),
    cmd(Key::ascii(b's'), nv_subst, NvFlags::KEEPREG, 0),
    cmd(
        Key::ascii(b't'),
        nv_csearch,
        NvFlags::NCH_ALW.or(NvFlags::LANG),
        FORWARD as c_int,
    ),
    cmd(Key::ascii(b'u'), nv_undo, NvFlags::NONE, 0),
    cmd(Key::ascii(b'w'), nv_wordcmd, NvFlags::NONE, 0),
    cmd(Key::ascii(b'x'), nv_abbrev, NvFlags::KEEPREG, 0),
    cmd(Key::ascii(b'y'), nv_operator, NvFlags::NONE, 0),
    cmd(Key::ascii(b'z'), nv_zet, NvFlags::NCH_ALW, 0),
    cmd(
        Key::ascii(b'{'),
        nv_findpar,
        NvFlags::NONE,
        BACKWARD as c_int,
    ),
    cmd(Key::ascii(b'|'), nv_pipe, NvFlags::NONE, 0),
    cmd(
        Key::ascii(b'}'),
        nv_findpar,
        NvFlags::NONE,
        FORWARD as c_int,
    ),
    cmd(Key::ascii(b'~'), nv_tilde, NvFlags::NONE, 0),
    cmd(Key::from_char('\u{a3}'), nv_ident, NvFlags::NONE, 0),
    cmd(
        Key::from_special(SpecialKey::Mouseup),
        nv_mousescroll,
        NvFlags::NONE,
        MSCR_UP,
    ),
    cmd(
        Key::from_special(SpecialKey::Mousedown),
        nv_mousescroll,
        NvFlags::NONE,
        MSCR_DOWN,
    ),
    cmd(
        Key::from_special(SpecialKey::Mouseleft),
        nv_mousescroll,
        NvFlags::NONE,
        MSCR_LEFT,
    ),
    cmd(
        Key::from_special(SpecialKey::Mouseright),
        nv_mousescroll,
        NvFlags::NONE,
        MSCR_RIGHT,
    ),
    cmd(
        Key::from_special(SpecialKey::Leftmouse),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::LeftmouseNm),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Leftdrag),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Leftrelease),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::LeftreleaseNm),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Mousemove),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Middlemouse),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Middledrag),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Middlerelease),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Rightmouse),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Rightdrag),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Rightrelease),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::X1mouse),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::X1drag),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::X1release),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::X2mouse),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::X2drag),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::X2release),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Ignore),
        nv_ignore,
        NvFlags::KEEPREG,
        0,
    ),
    cmd(Key::from_special(SpecialKey::Nop), nv_nop, NvFlags::NONE, 0),
    cmd(
        Key::from_special(SpecialKey::Ins),
        nv_edit,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Kins),
        nv_edit,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Bs),
        nv_ctrlh,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Up),
        nv_up,
        NvFlags::SSS.or(NvFlags::STS),
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::SUp),
        nv_page,
        NvFlags::SS,
        BACKWARD as c_int,
    ),
    cmd(
        Key::from_special(SpecialKey::Down),
        nv_down,
        NvFlags::SSS.or(NvFlags::STS),
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::SDown),
        nv_page,
        NvFlags::SS,
        FORWARD as c_int,
    ),
    cmd(
        Key::from_special(SpecialKey::Left),
        nv_left,
        NvFlags::SSS.or(NvFlags::STS).or(NvFlags::RL),
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::SLeft),
        nv_bck_word,
        NvFlags::SS.or(NvFlags::RL),
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::CLeft),
        nv_bck_word,
        NvFlags::SSS.or(NvFlags::RL).or(NvFlags::STS),
        1,
    ),
    cmd(
        Key::from_special(SpecialKey::Right),
        nv_right,
        NvFlags::SSS.or(NvFlags::STS).or(NvFlags::RL),
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::SRight),
        nv_wordcmd,
        NvFlags::SS.or(NvFlags::RL),
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::CRight),
        nv_wordcmd,
        NvFlags::SSS.or(NvFlags::RL).or(NvFlags::STS),
        1,
    ),
    cmd(
        Key::from_special(SpecialKey::Pageup),
        nv_page,
        NvFlags::SSS.or(NvFlags::STS),
        BACKWARD as c_int,
    ),
    cmd(
        Key::from_special(SpecialKey::Kpageup),
        nv_page,
        NvFlags::SSS.or(NvFlags::STS),
        BACKWARD as c_int,
    ),
    cmd(
        Key::from_special(SpecialKey::Pagedown),
        nv_page,
        NvFlags::SSS.or(NvFlags::STS),
        FORWARD as c_int,
    ),
    cmd(
        Key::from_special(SpecialKey::Kpagedown),
        nv_page,
        NvFlags::SSS.or(NvFlags::STS),
        FORWARD as c_int,
    ),
    cmd(
        Key::from_special(SpecialKey::End),
        nv_end,
        NvFlags::SSS.or(NvFlags::STS),
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Kend),
        nv_end,
        NvFlags::SSS.or(NvFlags::STS),
        0,
    ),
    cmd(Key::from_special(SpecialKey::SEnd), nv_end, NvFlags::SS, 0),
    cmd(
        Key::from_special(SpecialKey::CEnd),
        nv_end,
        NvFlags::SSS.or(NvFlags::STS),
        1,
    ),
    cmd(
        Key::from_special(SpecialKey::Home),
        nv_home,
        NvFlags::SSS.or(NvFlags::STS),
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Khome),
        nv_home,
        NvFlags::SSS.or(NvFlags::STS),
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::SHome),
        nv_home,
        NvFlags::SS,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::CHome),
        nv_goto,
        NvFlags::SSS.or(NvFlags::STS),
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Del),
        nv_abbrev,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Kdel),
        nv_abbrev,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Undo),
        nv_kundo,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Help),
        nv_help,
        NvFlags::NCW,
        0,
    ),
    cmd(Key::from_special(SpecialKey::F1), nv_help, NvFlags::NCW, 0),
    cmd(Key::from_special(SpecialKey::Xf1), nv_help, NvFlags::NCW, 0),
    cmd(
        Key::from_special(SpecialKey::Select),
        nv_select,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::PasteStart),
        nv_paste,
        NvFlags::KEEPREG,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Event),
        nv_event,
        NvFlags::KEEPREG,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Command),
        nv_colon,
        NvFlags::NONE,
        0,
    ),
    cmd(
        Key::from_special(SpecialKey::Lua),
        nv_colon,
        NvFlags::NONE,
        0,
    ),
];
static NV_CMD_TABLE: [NvCmd; 188] = NV_CMDS;
pub(crate) const NV_CMDS_SIZE: usize = NV_CMDS.len();
static SHOWCMD_IS_CLEAR: GlobalCell<bool> = GlobalCell::new(true);
static SHOWCMD_VISUAL: GlobalCell<bool> = GlobalCell::new(false);
