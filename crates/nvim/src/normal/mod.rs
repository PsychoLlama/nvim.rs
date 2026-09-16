//! Normal mode: the state loop, the command table, and the vocabulary the
//! thirteen command families share.
//!
//! The families are the modules below, grouped by what a command *does* to the
//! editor rather than by which key runs it -- which is the seam
//! [`nv_cmds`] already draws, because every row of it names a handler.
//!
//! What is left in this file is the table, the two structures it is made of,
//! and the constants at least one family imports by name. Nothing here is
//! code.

#![deny(unsafe_op_in_unsafe_fn)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::edit::BeginlineOpts;
use crate::global_cell::GlobalCell;
use crate::keycodes::{
    Ctrl__, Ctrl_A, Ctrl_B, Ctrl_BSL, Ctrl_C, Ctrl_D, Ctrl_E, Ctrl_F, Ctrl_G, Ctrl_H, Ctrl_HAT,
    Ctrl_I, Ctrl_K, Ctrl_L, Ctrl_N, Ctrl_O, Ctrl_P, Ctrl_Q, Ctrl_R, Ctrl_RSB, Ctrl_S, Ctrl_T,
    Ctrl_U, Ctrl_V, Ctrl_W, Ctrl_X, Ctrl_Y, Ctrl_Z, KE_C_END, KE_C_HOME, KE_C_LEFT, KE_C_RIGHT,
    KE_COMMAND, KE_EVENT, KE_IGNORE, KE_KDEL, KE_KINS, KE_LEFTDRAG, KE_LEFTMOUSE, KE_LEFTMOUSE_NM,
    KE_LEFTRELEASE, KE_LEFTRELEASE_NM, KE_LUA, KE_MIDDLEDRAG, KE_MIDDLEMOUSE, KE_MIDDLERELEASE,
    KE_MOUSEDOWN, KE_MOUSELEFT, KE_MOUSEMOVE, KE_MOUSERIGHT, KE_MOUSEUP, KE_NOP, KE_RIGHTDRAG,
    KE_RIGHTMOUSE, KE_RIGHTRELEASE, KE_S_DOWN, KE_S_UP, KE_X1DRAG, KE_X1MOUSE, KE_X1RELEASE,
    KE_X2DRAG, KE_X2MOUSE, KE_X2RELEASE, KE_XF1, Key,
};
use crate::mouse::{nv_mouse, nv_mousescroll};
use crate::types::{
    CmdArg, Direction, GetFileFlags, MarkGet, MarkMove, MarkMoveRes, MotionType, NUL, OpArg, Pos,
    SpellAddType, SpellMoveType, int16_t, uint16_t,
};
use core::ffi::{CStr, c_int, c_uint, c_void};

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
mod misc;
pub(crate) use self::misc::*;
pub(crate) const _ISlower: c_uint = 512;
pub(crate) const _ISupper: c_uint = 256;
pub(crate) const kDirectionNotSet: Direction = 0;
pub(crate) const kMarkChangedCursor: MarkMoveRes = 32;
pub(crate) const kMarkChangedLine: MarkMoveRes = 16;
pub(crate) const kMarkSwitchedBuf: MarkMoveRes = 4;
pub(crate) const kMarkMoveFailed: MarkMoveRes = 2;
pub(crate) const kMarkMoveSuccess: MarkMoveRes = 1;
pub(crate) const kMarkJumpList: MarkMove = 16;
pub(crate) const kMarkSetView: MarkMove = 8;
pub(crate) const KMarkNoContext: MarkMove = 4;
pub(crate) const kMarkContext: MarkMove = 2;
pub(crate) const kMarkBeginLine: MarkMove = 1;
pub(crate) const kMarkAll: MarkGet = 1;
pub(crate) const GETF_ALT: GetFileFlags = 2;
pub(crate) const GETF_SETMARK: GetFileFlags = 1;
pub(crate) const OPENLINE_DO_COM: c_uint = 2;
pub(crate) const HIST_SEARCH: c_int = 1;
pub(crate) const VSE_NONE: c_uint = 0;
pub(crate) const ML_DEL_MESSAGE: c_uint = 1;
pub(crate) const kMTLineWise: MotionType = 1;
pub(crate) const kMTCharWise: MotionType = 0;
pub(crate) const REPLACE_NL_NCHAR: c_int = -2;
pub(crate) const REPLACE_CR_NCHAR: c_int = -1;
pub(crate) const SHOWCMD_COLS: c_uint = 10;
pub(crate) const SHOWCMD_BUFLEN: c_uint = 41;
pub(crate) const MSCR_RIGHT: c_int = -2;
pub(crate) const MSCR_LEFT: c_int = -1;
pub(crate) const MSCR_UP: c_int = 1;
pub(crate) const MSCR_DOWN: c_int = 0;
pub(crate) const FIND_EVAL: c_uint = 4;
pub(crate) const FIND_STRING: c_uint = 2;
pub(crate) const FIND_IDENT: c_uint = 1;
pub(crate) struct NvCmd {
    pub cmd_char: c_int,
    pub cmd_func: NvFunc,
    pub cmd_flags: NvFlags,
    pub cmd_arg: int16_t,
}
/// What a row of [`nv_cmds`] runs.
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
pub(crate) const FM_FORWARD: c_uint = 2;
pub(crate) const SPELL_ADD_BAD: SpellAddType = 1;
pub(crate) const SPELL_ADD_GOOD: SpellAddType = 0;
pub(crate) const SMT_RARE: SpellMoveType = 2;
pub(crate) const SMT_BAD: SpellMoveType = 1;
pub(crate) const FM_BACKWARD: c_uint = 1;
pub(crate) const ACTION_GOTO: c_uint = 2;
pub(crate) const ACTION_SHOW: c_uint = 1;
pub(crate) const ACTION_SHOW_ALL: c_uint = 4;
pub(crate) const FIND_ANY: c_uint = 1;
pub(crate) const FIND_DEFINE: c_uint = 2;
pub(crate) const DT_POP: c_uint = 2;
pub(crate) struct NormalState {
    pub command_finished: bool,
    pub ctrl_w: bool,
    pub need_flushbuf: bool,
    pub set_prevcount: bool,
    pub previous_got_int: bool,
    pub cmdwin: bool,
    pub noexmode: bool,
    pub toplevel: bool,
    pub oa: OpArg,
    pub ca: CmdArg,
    pub mapped_len: c_int,
    pub old_mapped_len: c_int,
    pub idx: c_int,
    pub c: c_int,
    pub old_col: c_int,
    pub old_pos: Pos,
}
pub(crate) const NULL: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub(crate) const TAB: c_int = 9;
pub(crate) const NL: c_int = '\n' as c_int;
pub(crate) const CAR: c_int = '\r' as c_int;
pub(crate) const ESC: c_int = '\u{1b}' as c_int;
pub(crate) const DEL: c_int = 0x7f as c_int;
pub(crate) const POUND: c_int = 0xa3 as c_int;
pub(crate) const B_IMODE_LMAP: c_int = 1 as c_int;
pub(crate) const GRAPHEME_STATE_INIT: c_int = 0 as c_int;
static VIsual_mode_orig: GlobalCell<VisualMode> = GlobalCell::new(VisualMode::NONE);
const e_changelist_is_empty: &CStr = c"E664: Changelist is empty";
const e_cmdline_window_already_open: &CStr = c"E1292: Command-line window is already open";
crate::flag_set! {
    /// What a row of [`nv_cmds`] asks the dispatcher to do around the
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

/// One row of [`nv_cmds`].
///
/// The argument is written as the `c_int` constant that names it and narrowed
/// here, so a row reads as the four things it is rather than as a cast.
const fn cmd(cmd_char: c_int, cmd_func: NvFunc, cmd_flags: NvFlags, cmd_arg: c_int) -> NvCmd {
    NvCmd {
        cmd_char,
        cmd_func,
        cmd_flags,
        cmd_arg: cmd_arg as int16_t,
    }
}

/// The Normal-mode command table, in source order.
///
/// A `const` so that [`NV_CMD_IDX`](crate::normal::NV_CMD_IDX) can be sorted
/// at compile time; `nv_cmds` is the `static` everything indexes, so the rows
/// exist once.
pub(crate) const NV_CMDS: [NvCmd; 188] = [
    cmd(NUL, nv_error, NvFlags::NONE, 0),
    cmd(Ctrl_A, nv_addsub, NvFlags::NONE, 0),
    cmd(Ctrl_B, nv_page, NvFlags::STS, BACKWARD as c_int),
    cmd(Ctrl_C, nv_esc, NvFlags::NONE, 1),
    cmd(Ctrl_D, nv_halfpage, NvFlags::NONE, 0),
    cmd(Ctrl_E, nv_scroll_line, NvFlags::NONE, 1),
    cmd(Ctrl_F, nv_page, NvFlags::STS, FORWARD as c_int),
    cmd(Ctrl_G, nv_ctrlg, NvFlags::NONE, 0),
    cmd(Ctrl_H, nv_ctrlh, NvFlags::NONE, 0),
    cmd(Ctrl_I, nv_pcmark, NvFlags::NONE, 0),
    cmd(NL, nv_down, NvFlags::NONE, 0),
    cmd(Ctrl_K, nv_error, NvFlags::NONE, 0),
    cmd(Ctrl_L, nv_clear, NvFlags::NONE, 0),
    cmd(CAR, nv_down, NvFlags::NONE, 1),
    cmd(Ctrl_N, nv_down, NvFlags::STS, 0),
    cmd(Ctrl_O, nv_ctrlo, NvFlags::NONE, 0),
    cmd(Ctrl_P, nv_up, NvFlags::STS, 0),
    cmd(Ctrl_Q, nv_visual, NvFlags::NONE, 0),
    cmd(Ctrl_R, nv_redo_or_register, NvFlags::NONE, 0),
    cmd(Ctrl_S, nv_ignore, NvFlags::NONE, 0),
    cmd(Ctrl_T, nv_tagpop, NvFlags::NCW, 0),
    cmd(Ctrl_U, nv_halfpage, NvFlags::NONE, 0),
    cmd(Ctrl_V, nv_visual, NvFlags::NONE, 0),
    cmd('V' as c_int, nv_visual, NvFlags::NONE, 0),
    cmd('v' as c_int, nv_visual, NvFlags::NONE, 0),
    cmd(Ctrl_W, nv_window, NvFlags::NONE, 0),
    cmd(Ctrl_X, nv_addsub, NvFlags::NONE, 0),
    cmd(Ctrl_Y, nv_scroll_line, NvFlags::NONE, 0),
    cmd(Ctrl_Z, nv_suspend, NvFlags::NONE, 0),
    cmd(ESC, nv_esc, NvFlags::NONE, 0),
    cmd(Ctrl_BSL, nv_normal, NvFlags::NCH_ALW, 0),
    cmd(Ctrl_RSB, nv_ident, NvFlags::NCW, 0),
    cmd(Ctrl_HAT, nv_hat, NvFlags::NCW, 0),
    cmd(Ctrl__, nv_error, NvFlags::NONE, 0),
    cmd(' ' as c_int, nv_right, NvFlags::NONE, 0),
    cmd('!' as c_int, nv_operator, NvFlags::NONE, 0),
    cmd(
        '"' as c_int,
        nv_regname,
        NvFlags::NCH_NOP.or(NvFlags::KEEPREG),
        0,
    ),
    cmd('#' as c_int, nv_ident, NvFlags::NONE, 0),
    cmd('$' as c_int, nv_dollar, NvFlags::NONE, 0),
    cmd('%' as c_int, nv_percent, NvFlags::NONE, 0),
    cmd('&' as c_int, nv_optrans, NvFlags::NONE, 0),
    cmd('\'' as c_int, nv_gomark, NvFlags::NCH_ALW, 1),
    cmd('(' as c_int, nv_brace, NvFlags::NONE, BACKWARD as c_int),
    cmd(')' as c_int, nv_brace, NvFlags::NONE, FORWARD as c_int),
    cmd('*' as c_int, nv_ident, NvFlags::NONE, 0),
    cmd('+' as c_int, nv_down, NvFlags::NONE, 1),
    cmd(',' as c_int, nv_csearch, NvFlags::NONE, 1),
    cmd('-' as c_int, nv_up, NvFlags::NONE, 1),
    cmd('.' as c_int, nv_dot, NvFlags::KEEPREG, 0),
    cmd('/' as c_int, nv_search, NvFlags::NONE, 0),
    cmd('0' as c_int, nv_beginline, NvFlags::NONE, 0),
    cmd('1' as c_int, nv_ignore, NvFlags::NONE, 0),
    cmd('2' as c_int, nv_ignore, NvFlags::NONE, 0),
    cmd('3' as c_int, nv_ignore, NvFlags::NONE, 0),
    cmd('4' as c_int, nv_ignore, NvFlags::NONE, 0),
    cmd('5' as c_int, nv_ignore, NvFlags::NONE, 0),
    cmd('6' as c_int, nv_ignore, NvFlags::NONE, 0),
    cmd('7' as c_int, nv_ignore, NvFlags::NONE, 0),
    cmd('8' as c_int, nv_ignore, NvFlags::NONE, 0),
    cmd('9' as c_int, nv_ignore, NvFlags::NONE, 0),
    cmd(':' as c_int, nv_colon, NvFlags::NONE, 0),
    cmd(';' as c_int, nv_csearch, NvFlags::NONE, 0),
    cmd('<' as c_int, nv_operator, NvFlags::RL, 0),
    cmd('=' as c_int, nv_operator, NvFlags::NONE, 0),
    cmd('>' as c_int, nv_operator, NvFlags::RL, 0),
    cmd('?' as c_int, nv_search, NvFlags::NONE, 0),
    cmd('@' as c_int, nv_at, NvFlags::NCH_NOP, 0),
    cmd('A' as c_int, nv_edit, NvFlags::NONE, 0),
    cmd('B' as c_int, nv_bck_word, NvFlags::NONE, 1),
    cmd('C' as c_int, nv_abbrev, NvFlags::KEEPREG, 0),
    cmd('D' as c_int, nv_abbrev, NvFlags::KEEPREG, 0),
    cmd('E' as c_int, nv_wordcmd, NvFlags::NONE, 1),
    cmd(
        'F' as c_int,
        nv_csearch,
        NvFlags::NCH_ALW.or(NvFlags::LANG),
        BACKWARD as c_int,
    ),
    cmd('G' as c_int, nv_goto, NvFlags::NONE, 1),
    cmd('H' as c_int, nv_scroll, NvFlags::NONE, 0),
    cmd('I' as c_int, nv_edit, NvFlags::NONE, 0),
    cmd('J' as c_int, nv_join, NvFlags::NONE, 0),
    cmd('K' as c_int, nv_ident, NvFlags::NONE, 0),
    cmd('L' as c_int, nv_scroll, NvFlags::NONE, 0),
    cmd('M' as c_int, nv_scroll, NvFlags::NONE, 0),
    cmd('N' as c_int, nv_next, NvFlags::NONE, SEARCH_REV as c_int),
    cmd('O' as c_int, nv_open, NvFlags::NONE, 0),
    cmd('P' as c_int, nv_put, NvFlags::NONE, 0),
    cmd('Q' as c_int, nv_regreplay, NvFlags::NONE, 0),
    cmd('R' as c_int, nv_replace_mode, NvFlags::NONE, 0),
    cmd('S' as c_int, nv_subst, NvFlags::KEEPREG, 0),
    cmd(
        'T' as c_int,
        nv_csearch,
        NvFlags::NCH_ALW.or(NvFlags::LANG),
        BACKWARD as c_int,
    ),
    cmd('U' as c_int, nv_undo_line, NvFlags::NONE, 0),
    cmd('W' as c_int, nv_wordcmd, NvFlags::NONE, 1),
    cmd('X' as c_int, nv_abbrev, NvFlags::KEEPREG, 0),
    cmd('Y' as c_int, nv_abbrev, NvFlags::KEEPREG, 0),
    cmd(
        'Z' as c_int,
        nv_exit_command,
        NvFlags::NCH_NOP.or(NvFlags::NCW),
        0,
    ),
    cmd(
        '[' as c_int,
        nv_brackets,
        NvFlags::NCH_ALW,
        BACKWARD as c_int,
    ),
    cmd('\\' as c_int, nv_error, NvFlags::NONE, 0),
    cmd(
        ']' as c_int,
        nv_brackets,
        NvFlags::NCH_ALW,
        FORWARD as c_int,
    ),
    cmd(
        '^' as c_int,
        nv_beginline,
        NvFlags::NONE,
        BeginlineOpts::WHITE.or(BeginlineOpts::FIX).bits(),
    ),
    cmd('_' as c_int, nv_lineop, NvFlags::NONE, 0),
    cmd('`' as c_int, nv_gomark, NvFlags::NCH_ALW, 0),
    cmd('a' as c_int, nv_edit, NvFlags::NCH, 0),
    cmd('b' as c_int, nv_bck_word, NvFlags::NONE, 0),
    cmd('c' as c_int, nv_operator, NvFlags::NONE, 0),
    cmd('d' as c_int, nv_operator, NvFlags::NONE, 0),
    cmd('e' as c_int, nv_wordcmd, NvFlags::NONE, 0),
    cmd(
        'f' as c_int,
        nv_csearch,
        NvFlags::NCH_ALW.or(NvFlags::LANG),
        FORWARD as c_int,
    ),
    cmd('g' as c_int, nv_g_cmd, NvFlags::NCH_ALW, 0),
    cmd('h' as c_int, nv_left, NvFlags::RL, 0),
    cmd('i' as c_int, nv_edit, NvFlags::NCH, 0),
    cmd('j' as c_int, nv_down, NvFlags::NONE, 0),
    cmd('k' as c_int, nv_up, NvFlags::NONE, 0),
    cmd('l' as c_int, nv_right, NvFlags::RL, 0),
    cmd('m' as c_int, nv_mark, NvFlags::NCH_NOP, 0),
    cmd('n' as c_int, nv_next, NvFlags::NONE, 0),
    cmd('o' as c_int, nv_open, NvFlags::NONE, 0),
    cmd('p' as c_int, nv_put, NvFlags::NONE, 0),
    cmd('q' as c_int, nv_record, NvFlags::NCH, 0),
    cmd(
        'r' as c_int,
        nv_replace,
        NvFlags::NCH_NOP.or(NvFlags::LANG),
        0,
    ),
    cmd('s' as c_int, nv_subst, NvFlags::KEEPREG, 0),
    cmd(
        't' as c_int,
        nv_csearch,
        NvFlags::NCH_ALW.or(NvFlags::LANG),
        FORWARD as c_int,
    ),
    cmd('u' as c_int, nv_undo, NvFlags::NONE, 0),
    cmd('w' as c_int, nv_wordcmd, NvFlags::NONE, 0),
    cmd('x' as c_int, nv_abbrev, NvFlags::KEEPREG, 0),
    cmd('y' as c_int, nv_operator, NvFlags::NONE, 0),
    cmd('z' as c_int, nv_zet, NvFlags::NCH_ALW, 0),
    cmd('{' as c_int, nv_findpar, NvFlags::NONE, BACKWARD as c_int),
    cmd('|' as c_int, nv_pipe, NvFlags::NONE, 0),
    cmd('}' as c_int, nv_findpar, NvFlags::NONE, FORWARD as c_int),
    cmd('~' as c_int, nv_tilde, NvFlags::NONE, 0),
    cmd(POUND, nv_ident, NvFlags::NONE, 0),
    cmd(
        -(253 as c_int + ((KE_MOUSEUP as c_int) << 8 as c_int)),
        nv_mousescroll,
        NvFlags::NONE,
        MSCR_UP as c_int,
    ),
    cmd(
        -(253 as c_int + ((KE_MOUSEDOWN as c_int) << 8 as c_int)),
        nv_mousescroll,
        NvFlags::NONE,
        MSCR_DOWN as c_int,
    ),
    cmd(
        -(253 as c_int + ((KE_MOUSELEFT as c_int) << 8 as c_int)),
        nv_mousescroll,
        NvFlags::NONE,
        MSCR_LEFT as c_int,
    ),
    cmd(
        -(253 as c_int + ((KE_MOUSERIGHT as c_int) << 8 as c_int)),
        nv_mousescroll,
        NvFlags::NONE,
        MSCR_RIGHT as c_int,
    ),
    cmd(
        -(253 as c_int + ((KE_LEFTMOUSE as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_LEFTMOUSE_NM as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_LEFTDRAG as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_LEFTRELEASE as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_LEFTRELEASE_NM as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_MOUSEMOVE as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_MIDDLEMOUSE as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_MIDDLEDRAG as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_MIDDLERELEASE as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_RIGHTMOUSE as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_RIGHTDRAG as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_RIGHTRELEASE as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X1MOUSE as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X1DRAG as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X1RELEASE as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X2MOUSE as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X2DRAG as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X2RELEASE as c_int) << 8 as c_int)),
        nv_mouse,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int)),
        nv_ignore,
        NvFlags::KEEPREG,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_NOP as c_int) << 8 as c_int)),
        nv_nop,
        NvFlags::NONE,
        0,
    ),
    cmd(Key::Ins.code(), nv_edit, NvFlags::NONE, 0),
    cmd(
        -(253 as c_int + ((KE_KINS as c_int) << 8 as c_int)),
        nv_edit,
        NvFlags::NONE,
        0,
    ),
    cmd(Key::Bs.code(), nv_ctrlh, NvFlags::NONE, 0),
    cmd(Key::Up.code(), nv_up, NvFlags::SSS.or(NvFlags::STS), 0),
    cmd(
        -(253 as c_int + ((KE_S_UP as c_int) << 8 as c_int)),
        nv_page,
        NvFlags::SS,
        BACKWARD as c_int,
    ),
    cmd(Key::Down.code(), nv_down, NvFlags::SSS.or(NvFlags::STS), 0),
    cmd(
        -(253 as c_int + ((KE_S_DOWN as c_int) << 8 as c_int)),
        nv_page,
        NvFlags::SS,
        FORWARD as c_int,
    ),
    cmd(
        Key::Left.code(),
        nv_left,
        NvFlags::SSS.or(NvFlags::STS).or(NvFlags::RL),
        0,
    ),
    cmd(
        Key::SLeft.code(),
        nv_bck_word,
        NvFlags::SS.or(NvFlags::RL),
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_C_LEFT as c_int) << 8 as c_int)),
        nv_bck_word,
        NvFlags::SSS.or(NvFlags::RL).or(NvFlags::STS),
        1,
    ),
    cmd(
        Key::Right.code(),
        nv_right,
        NvFlags::SSS.or(NvFlags::STS).or(NvFlags::RL),
        0,
    ),
    cmd(
        Key::SRight.code(),
        nv_wordcmd,
        NvFlags::SS.or(NvFlags::RL),
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_C_RIGHT as c_int) << 8 as c_int)),
        nv_wordcmd,
        NvFlags::SSS.or(NvFlags::RL).or(NvFlags::STS),
        1,
    ),
    cmd(
        Key::Pageup.code(),
        nv_page,
        NvFlags::SSS.or(NvFlags::STS),
        BACKWARD as c_int,
    ),
    cmd(
        Key::Kpageup.code(),
        nv_page,
        NvFlags::SSS.or(NvFlags::STS),
        BACKWARD as c_int,
    ),
    cmd(
        Key::Pagedown.code(),
        nv_page,
        NvFlags::SSS.or(NvFlags::STS),
        FORWARD as c_int,
    ),
    cmd(
        Key::Kpagedown.code(),
        nv_page,
        NvFlags::SSS.or(NvFlags::STS),
        FORWARD as c_int,
    ),
    cmd(Key::End.code(), nv_end, NvFlags::SSS.or(NvFlags::STS), 0),
    cmd(Key::Kend.code(), nv_end, NvFlags::SSS.or(NvFlags::STS), 0),
    cmd(Key::SEnd.code(), nv_end, NvFlags::SS, 0),
    cmd(
        -(253 as c_int + ((KE_C_END as c_int) << 8 as c_int)),
        nv_end,
        NvFlags::SSS.or(NvFlags::STS),
        1,
    ),
    cmd(Key::Home.code(), nv_home, NvFlags::SSS.or(NvFlags::STS), 0),
    cmd(Key::Khome.code(), nv_home, NvFlags::SSS.or(NvFlags::STS), 0),
    cmd(Key::SHome.code(), nv_home, NvFlags::SS, 0),
    cmd(
        -(253 as c_int + ((KE_C_HOME as c_int) << 8 as c_int)),
        nv_goto,
        NvFlags::SSS.or(NvFlags::STS),
        0,
    ),
    cmd(Key::Del.code(), nv_abbrev, NvFlags::NONE, 0),
    cmd(
        -(253 as c_int + ((KE_KDEL as c_int) << 8 as c_int)),
        nv_abbrev,
        NvFlags::NONE,
        0,
    ),
    cmd(Key::Undo.code(), nv_kundo, NvFlags::NONE, 0),
    cmd(Key::Help.code(), nv_help, NvFlags::NCW, 0),
    cmd(Key::F1.code(), nv_help, NvFlags::NCW, 0),
    cmd(
        -(253 as c_int + ((KE_XF1 as c_int) << 8 as c_int)),
        nv_help,
        NvFlags::NCW,
        0,
    ),
    cmd(Key::Select.code(), nv_select, NvFlags::NONE, 0),
    cmd(Key::PasteStart.code(), nv_paste, NvFlags::KEEPREG, 0),
    cmd(
        -(253 as c_int + ((KE_EVENT as c_int) << 8 as c_int)),
        nv_event,
        NvFlags::KEEPREG,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_COMMAND as c_int) << 8 as c_int)),
        nv_colon,
        NvFlags::NONE,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_LUA as c_int) << 8 as c_int)),
        nv_colon,
        NvFlags::NONE,
        0,
    ),
];
static nv_cmds: [NvCmd; 188] = NV_CMDS;
pub(crate) const NV_CMDS_SIZE: usize = ::core::mem::size_of::<[NvCmd; 188]>()
    .wrapping_div(::core::mem::size_of::<NvCmd>())
    .wrapping_div(
        (::core::mem::size_of::<[NvCmd; 188]>().wrapping_rem(::core::mem::size_of::<NvCmd>()) == 0)
            as c_int as usize,
    );
static current_oap: GlobalCell<*mut OpArg> = GlobalCell::new(::core::ptr::null_mut::<OpArg>());
static showcmd_is_clear: GlobalCell<bool> = GlobalCell::new(true);
static showcmd_visual: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) const INT_MAX: c_int = __INT_MAX__;
pub(crate) const __INT_MAX__: c_int = 2147483647 as c_int;
