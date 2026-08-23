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

use crate::edit::BeginlineOpts;
use crate::global_cell::GlobalCell;
use crate::mouse::{nv_mouse, nv_mousescroll};
use crate::types::{
    Array, Direction, MarkGet, MarkMove, MarkMoveRes, MotionType, NUL, Object, SpellAddType,
    VimState, cmdarg_T, getf_values, int16_t, oparg_T, pos_T, size_t, smt_T, uint16_t,
};
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};

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
use crate::keycodes::{
    Ctrl__, Ctrl_A, Ctrl_B, Ctrl_BSL, Ctrl_C, Ctrl_D, Ctrl_E, Ctrl_F, Ctrl_G, Ctrl_H, Ctrl_HAT,
    Ctrl_I, Ctrl_K, Ctrl_L, Ctrl_N, Ctrl_O, Ctrl_P, Ctrl_Q, Ctrl_R, Ctrl_RSB, Ctrl_S, Ctrl_T,
    Ctrl_U, Ctrl_V, Ctrl_W, Ctrl_X, Ctrl_Y, Ctrl_Z, K_BS, K_DEL, K_DOWN, K_END, K_F1, K_HELP,
    K_HOME, K_INS, K_KEND, K_KHOME, K_KPAGEDOWN, K_KPAGEUP, K_LEFT, K_PAGEDOWN, K_PAGEUP,
    K_PASTE_START, K_RIGHT, K_S_END, K_S_HOME, K_S_LEFT, K_S_RIGHT, K_SELECT, K_UNDO, K_UP,
    KE_C_END, KE_C_HOME, KE_C_LEFT, KE_C_RIGHT, KE_COMMAND, KE_EVENT, KE_IGNORE, KE_KDEL, KE_KINS,
    KE_LEFTDRAG, KE_LEFTMOUSE, KE_LEFTMOUSE_NM, KE_LEFTRELEASE, KE_LEFTRELEASE_NM, KE_LUA,
    KE_MIDDLEDRAG, KE_MIDDLEMOUSE, KE_MIDDLERELEASE, KE_MOUSEDOWN, KE_MOUSELEFT, KE_MOUSEMOVE,
    KE_MOUSERIGHT, KE_MOUSEUP, KE_NOP, KE_RIGHTDRAG, KE_RIGHTMOUSE, KE_RIGHTRELEASE, KE_S_DOWN,
    KE_S_UP, KE_X1DRAG, KE_X1MOUSE, KE_X1RELEASE, KE_X2DRAG, KE_X2MOUSE, KE_X2RELEASE, KE_XF1,
};
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
pub(crate) const GETF_ALT: getf_values = 2;
pub(crate) const GETF_SETMARK: getf_values = 1;
pub(crate) const OPENLINE_DO_COM: c_uint = 2;
pub(crate) const HIST_SEARCH: c_int = 1;
pub(crate) const ECMD_HIDE: c_uint = 1;
pub(crate) const ECMD_LAST: c_int = -1;
pub(crate) const VSE_NONE: c_uint = 0;
pub(crate) const ML_DEL_MESSAGE: c_uint = 1;
pub(crate) const kMTLineWise: MotionType = 1;
pub(crate) const kMTCharWise: MotionType = 0;
pub(crate) const CA_NO_ADJ_OP_END: c_uint = 2;
pub(crate) const CA_COMMAND_BUSY: c_uint = 1;
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
#[derive(Copy, Clone)]
pub(crate) struct nv_cmd {
    pub cmd_char: c_int,
    pub cmd_func: nv_func_T,
    pub cmd_flags: uint16_t,
    pub cmd_arg: int16_t,
}
/// What a row of [`nv_cmds`] runs.
///
/// Nothing outside this crate reaches the table or its handlers -- neither the
/// ABI ledger nor the unit-test cdefs name any of them -- so the handlers are
/// ordinary Rust functions rather than `extern "C"` ones.
pub(crate) type nv_func_T = Option<unsafe fn(*mut cmdarg_T)>;
pub(crate) const FM_FORWARD: c_uint = 2;
pub(crate) const SPELL_ADD_BAD: SpellAddType = 1;
pub(crate) const SPELL_ADD_GOOD: SpellAddType = 0;
pub(crate) const SMT_RARE: smt_T = 2;
pub(crate) const SMT_BAD: smt_T = 1;
pub(crate) const FM_BACKWARD: c_uint = 1;
pub(crate) const ACTION_GOTO: c_uint = 2;
pub(crate) const ACTION_SHOW: c_uint = 1;
pub(crate) const ACTION_SHOW_ALL: c_uint = 4;
pub(crate) const FIND_ANY: c_uint = 1;
pub(crate) const FIND_DEFINE: c_uint = 2;
pub(crate) const DT_POP: c_uint = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct NormalState {
    pub state: VimState,
    pub command_finished: bool,
    pub ctrl_w: bool,
    pub need_flushbuf: bool,
    pub set_prevcount: bool,
    pub previous_got_int: bool,
    pub cmdwin: bool,
    pub noexmode: bool,
    pub toplevel: bool,
    pub oa: oparg_T,
    pub ca: cmdarg_T,
    pub mapped_len: c_int,
    pub old_mapped_len: c_int,
    pub idx: c_int,
    pub c: c_int,
    pub old_col: c_int,
    pub old_pos: pos_T,
}
pub(crate) const NULL: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub(crate) const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub(crate) const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub(crate) const TAB: c_int = 9;
pub(crate) const NL: c_int = '\n' as c_int;
pub(crate) const CAR: c_int = '\r' as c_int;
pub(crate) const ESC: c_int = '\u{1b}' as c_int;
pub(crate) const DEL: c_int = 0x7f as c_int;
pub(crate) const POUND: c_int = 0xa3 as c_int;
pub(crate) const B_IMODE_LMAP: c_int = 1 as c_int;
pub(crate) const MOD_MASK_SHIFT: c_int = 0x2 as c_int;
pub(crate) const MOD_MASK_CTRL: c_int = 0x4 as c_int;
pub(crate) const GRAPHEME_STATE_INIT: c_int = 0 as c_int;
static VIsual_mode_orig: GlobalCell<c_int> = GlobalCell::new(NUL);
const e_changelist_is_empty: &CStr = c"E664: Changelist is empty";
const e_cmdline_window_already_open: &CStr = c"E1292: Command-line window is already open";
pub(crate) const NV_NCH: c_int = 0x1 as c_int;
pub(crate) const NV_NCH_NOP: c_int = 0x2 as c_int | NV_NCH;
pub(crate) const NV_NCH_ALW: c_int = 0x4 as c_int | NV_NCH;
pub(crate) const NV_LANG: c_int = 0x8 as c_int;
pub(crate) const NV_SS: c_int = 0x10 as c_int;
pub(crate) const NV_SSS: c_int = 0x20 as c_int;
pub(crate) const NV_STS: c_int = 0x40 as c_int;
pub(crate) const NV_RL: c_int = 0x80 as c_int;
pub(crate) const NV_KEEPREG: c_int = 0x100 as c_int;
pub(crate) const NV_NCW: c_int = 0x200 as c_int;
/// One row of [`nv_cmds`].
///
/// The flags and the argument are written as the `c_int` constants that name
/// them and narrowed here, so a row reads as the four things it is rather than
/// as four casts.
const fn cmd(cmd_char: c_int, cmd_func: nv_func_T, cmd_flags: c_int, cmd_arg: c_int) -> nv_cmd {
    nv_cmd {
        cmd_char,
        cmd_func,
        cmd_flags: cmd_flags as uint16_t,
        cmd_arg: cmd_arg as int16_t,
    }
}

static nv_cmds: GlobalCell<[nv_cmd; 188]> = GlobalCell::new([
    cmd(NUL, Some(nv_error), 0, 0),
    cmd(Ctrl_A, Some(nv_addsub), 0, 0),
    cmd(Ctrl_B, Some(nv_page), NV_STS, BACKWARD as c_int),
    cmd(Ctrl_C, Some(nv_esc), 0, 1),
    cmd(Ctrl_D, Some(nv_halfpage), 0, 0),
    cmd(Ctrl_E, Some(nv_scroll_line), 0, 1),
    cmd(Ctrl_F, Some(nv_page), NV_STS, FORWARD as c_int),
    cmd(Ctrl_G, Some(nv_ctrlg), 0, 0),
    cmd(Ctrl_H, Some(nv_ctrlh), 0, 0),
    cmd(Ctrl_I, Some(nv_pcmark), 0, 0),
    cmd(NL, Some(nv_down), 0, 0),
    cmd(Ctrl_K, Some(nv_error), 0, 0),
    cmd(Ctrl_L, Some(nv_clear), 0, 0),
    cmd(CAR, Some(nv_down), 0, 1),
    cmd(Ctrl_N, Some(nv_down), NV_STS, 0),
    cmd(Ctrl_O, Some(nv_ctrlo), 0, 0),
    cmd(Ctrl_P, Some(nv_up), NV_STS, 0),
    cmd(Ctrl_Q, Some(nv_visual), 0, 0),
    cmd(Ctrl_R, Some(nv_redo_or_register), 0, 0),
    cmd(Ctrl_S, Some(nv_ignore), 0, 0),
    cmd(Ctrl_T, Some(nv_tagpop), NV_NCW, 0),
    cmd(Ctrl_U, Some(nv_halfpage), 0, 0),
    cmd(Ctrl_V, Some(nv_visual), 0, 0),
    cmd('V' as c_int, Some(nv_visual), 0, 0),
    cmd('v' as c_int, Some(nv_visual), 0, 0),
    cmd(Ctrl_W, Some(nv_window), 0, 0),
    cmd(Ctrl_X, Some(nv_addsub), 0, 0),
    cmd(Ctrl_Y, Some(nv_scroll_line), 0, 0),
    cmd(Ctrl_Z, Some(nv_suspend), 0, 0),
    cmd(ESC, Some(nv_esc), 0, 0),
    cmd(Ctrl_BSL, Some(nv_normal), NV_NCH_ALW, 0),
    cmd(Ctrl_RSB, Some(nv_ident), NV_NCW, 0),
    cmd(Ctrl_HAT, Some(nv_hat), NV_NCW, 0),
    cmd(Ctrl__, Some(nv_error), 0, 0),
    cmd(' ' as c_int, Some(nv_right), 0, 0),
    cmd('!' as c_int, Some(nv_operator), 0, 0),
    cmd('"' as c_int, Some(nv_regname), NV_NCH_NOP | NV_KEEPREG, 0),
    cmd('#' as c_int, Some(nv_ident), 0, 0),
    cmd('$' as c_int, Some(nv_dollar), 0, 0),
    cmd('%' as c_int, Some(nv_percent), 0, 0),
    cmd('&' as c_int, Some(nv_optrans), 0, 0),
    cmd('\'' as c_int, Some(nv_gomark), NV_NCH_ALW, 1),
    cmd('(' as c_int, Some(nv_brace), 0, BACKWARD as c_int),
    cmd(')' as c_int, Some(nv_brace), 0, FORWARD as c_int),
    cmd('*' as c_int, Some(nv_ident), 0, 0),
    cmd('+' as c_int, Some(nv_down), 0, 1),
    cmd(',' as c_int, Some(nv_csearch), 0, 1),
    cmd('-' as c_int, Some(nv_up), 0, 1),
    cmd('.' as c_int, Some(nv_dot), NV_KEEPREG, 0),
    cmd('/' as c_int, Some(nv_search), 0, 0),
    cmd('0' as c_int, Some(nv_beginline), 0, 0),
    cmd('1' as c_int, Some(nv_ignore), 0, 0),
    cmd('2' as c_int, Some(nv_ignore), 0, 0),
    cmd('3' as c_int, Some(nv_ignore), 0, 0),
    cmd('4' as c_int, Some(nv_ignore), 0, 0),
    cmd('5' as c_int, Some(nv_ignore), 0, 0),
    cmd('6' as c_int, Some(nv_ignore), 0, 0),
    cmd('7' as c_int, Some(nv_ignore), 0, 0),
    cmd('8' as c_int, Some(nv_ignore), 0, 0),
    cmd('9' as c_int, Some(nv_ignore), 0, 0),
    cmd(':' as c_int, Some(nv_colon), 0, 0),
    cmd(';' as c_int, Some(nv_csearch), 0, 0),
    cmd('<' as c_int, Some(nv_operator), NV_RL, 0),
    cmd('=' as c_int, Some(nv_operator), 0, 0),
    cmd('>' as c_int, Some(nv_operator), NV_RL, 0),
    cmd('?' as c_int, Some(nv_search), 0, 0),
    cmd('@' as c_int, Some(nv_at), NV_NCH_NOP, 0),
    cmd('A' as c_int, Some(nv_edit), 0, 0),
    cmd('B' as c_int, Some(nv_bck_word), 0, 1),
    cmd('C' as c_int, Some(nv_abbrev), NV_KEEPREG, 0),
    cmd('D' as c_int, Some(nv_abbrev), NV_KEEPREG, 0),
    cmd('E' as c_int, Some(nv_wordcmd), 0, 1),
    cmd(
        'F' as c_int,
        Some(nv_csearch),
        NV_NCH_ALW | NV_LANG,
        BACKWARD as c_int,
    ),
    cmd('G' as c_int, Some(nv_goto), 0, 1),
    cmd('H' as c_int, Some(nv_scroll), 0, 0),
    cmd('I' as c_int, Some(nv_edit), 0, 0),
    cmd('J' as c_int, Some(nv_join), 0, 0),
    cmd('K' as c_int, Some(nv_ident), 0, 0),
    cmd('L' as c_int, Some(nv_scroll), 0, 0),
    cmd('M' as c_int, Some(nv_scroll), 0, 0),
    cmd('N' as c_int, Some(nv_next), 0, SEARCH_REV as c_int),
    cmd('O' as c_int, Some(nv_open), 0, 0),
    cmd('P' as c_int, Some(nv_put), 0, 0),
    cmd('Q' as c_int, Some(nv_regreplay), 0, 0),
    cmd('R' as c_int, Some(nv_replace_mode), 0, 0),
    cmd('S' as c_int, Some(nv_subst), NV_KEEPREG, 0),
    cmd(
        'T' as c_int,
        Some(nv_csearch),
        NV_NCH_ALW | NV_LANG,
        BACKWARD as c_int,
    ),
    cmd('U' as c_int, Some(nv_undo_line), 0, 0),
    cmd('W' as c_int, Some(nv_wordcmd), 0, 1),
    cmd('X' as c_int, Some(nv_abbrev), NV_KEEPREG, 0),
    cmd('Y' as c_int, Some(nv_abbrev), NV_KEEPREG, 0),
    cmd('Z' as c_int, Some(nv_exit_command), NV_NCH_NOP | NV_NCW, 0),
    cmd(
        '[' as c_int,
        Some(nv_brackets),
        NV_NCH_ALW,
        BACKWARD as c_int,
    ),
    cmd('\\' as c_int, Some(nv_error), 0, 0),
    cmd(
        ']' as c_int,
        Some(nv_brackets),
        NV_NCH_ALW,
        FORWARD as c_int,
    ),
    cmd(
        '^' as c_int,
        Some(nv_beginline),
        0,
        BeginlineOpts::WHITE.or(BeginlineOpts::FIX).bits(),
    ),
    cmd('_' as c_int, Some(nv_lineop), 0, 0),
    cmd('`' as c_int, Some(nv_gomark), NV_NCH_ALW, 0),
    cmd('a' as c_int, Some(nv_edit), NV_NCH, 0),
    cmd('b' as c_int, Some(nv_bck_word), 0, 0),
    cmd('c' as c_int, Some(nv_operator), 0, 0),
    cmd('d' as c_int, Some(nv_operator), 0, 0),
    cmd('e' as c_int, Some(nv_wordcmd), 0, 0),
    cmd(
        'f' as c_int,
        Some(nv_csearch),
        NV_NCH_ALW | NV_LANG,
        FORWARD as c_int,
    ),
    cmd('g' as c_int, Some(nv_g_cmd), NV_NCH_ALW, 0),
    cmd('h' as c_int, Some(nv_left), NV_RL, 0),
    cmd('i' as c_int, Some(nv_edit), NV_NCH, 0),
    cmd('j' as c_int, Some(nv_down), 0, 0),
    cmd('k' as c_int, Some(nv_up), 0, 0),
    cmd('l' as c_int, Some(nv_right), NV_RL, 0),
    cmd('m' as c_int, Some(nv_mark), NV_NCH_NOP, 0),
    cmd('n' as c_int, Some(nv_next), 0, 0),
    cmd('o' as c_int, Some(nv_open), 0, 0),
    cmd('p' as c_int, Some(nv_put), 0, 0),
    cmd('q' as c_int, Some(nv_record), NV_NCH, 0),
    cmd('r' as c_int, Some(nv_replace), NV_NCH_NOP | NV_LANG, 0),
    cmd('s' as c_int, Some(nv_subst), NV_KEEPREG, 0),
    cmd(
        't' as c_int,
        Some(nv_csearch),
        NV_NCH_ALW | NV_LANG,
        FORWARD as c_int,
    ),
    cmd('u' as c_int, Some(nv_undo), 0, 0),
    cmd('w' as c_int, Some(nv_wordcmd), 0, 0),
    cmd('x' as c_int, Some(nv_abbrev), NV_KEEPREG, 0),
    cmd('y' as c_int, Some(nv_operator), 0, 0),
    cmd('z' as c_int, Some(nv_zet), NV_NCH_ALW, 0),
    cmd('{' as c_int, Some(nv_findpar), 0, BACKWARD as c_int),
    cmd('|' as c_int, Some(nv_pipe), 0, 0),
    cmd('}' as c_int, Some(nv_findpar), 0, FORWARD as c_int),
    cmd('~' as c_int, Some(nv_tilde), 0, 0),
    cmd(POUND, Some(nv_ident), 0, 0),
    cmd(
        -(253 as c_int + ((KE_MOUSEUP as c_int) << 8 as c_int)),
        Some(nv_mousescroll),
        0,
        MSCR_UP as c_int,
    ),
    cmd(
        -(253 as c_int + ((KE_MOUSEDOWN as c_int) << 8 as c_int)),
        Some(nv_mousescroll),
        0,
        MSCR_DOWN as c_int,
    ),
    cmd(
        -(253 as c_int + ((KE_MOUSELEFT as c_int) << 8 as c_int)),
        Some(nv_mousescroll),
        0,
        MSCR_LEFT as c_int,
    ),
    cmd(
        -(253 as c_int + ((KE_MOUSERIGHT as c_int) << 8 as c_int)),
        Some(nv_mousescroll),
        0,
        MSCR_RIGHT as c_int,
    ),
    cmd(
        -(253 as c_int + ((KE_LEFTMOUSE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_LEFTMOUSE_NM as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_LEFTDRAG as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_LEFTRELEASE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_LEFTRELEASE_NM as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_MOUSEMOVE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_MIDDLEMOUSE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_MIDDLEDRAG as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_MIDDLERELEASE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_RIGHTMOUSE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_RIGHTDRAG as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_RIGHTRELEASE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X1MOUSE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X1DRAG as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X1RELEASE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X2MOUSE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X2DRAG as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X2RELEASE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int)),
        Some(nv_ignore),
        NV_KEEPREG,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_NOP as c_int) << 8 as c_int)),
        Some(nv_nop),
        0,
        0,
    ),
    cmd(K_INS, Some(nv_edit), 0, 0),
    cmd(
        -(253 as c_int + ((KE_KINS as c_int) << 8 as c_int)),
        Some(nv_edit),
        0,
        0,
    ),
    cmd(K_BS, Some(nv_ctrlh), 0, 0),
    cmd(K_UP, Some(nv_up), NV_SSS | NV_STS, 0),
    cmd(
        -(253 as c_int + ((KE_S_UP as c_int) << 8 as c_int)),
        Some(nv_page),
        NV_SS,
        BACKWARD as c_int,
    ),
    cmd(K_DOWN, Some(nv_down), NV_SSS | NV_STS, 0),
    cmd(
        -(253 as c_int + ((KE_S_DOWN as c_int) << 8 as c_int)),
        Some(nv_page),
        NV_SS,
        FORWARD as c_int,
    ),
    cmd(K_LEFT, Some(nv_left), NV_SSS | NV_STS | NV_RL, 0),
    cmd(K_S_LEFT, Some(nv_bck_word), NV_SS | NV_RL, 0),
    cmd(
        -(253 as c_int + ((KE_C_LEFT as c_int) << 8 as c_int)),
        Some(nv_bck_word),
        NV_SSS | NV_RL | NV_STS,
        1,
    ),
    cmd(K_RIGHT, Some(nv_right), NV_SSS | NV_STS | NV_RL, 0),
    cmd(K_S_RIGHT, Some(nv_wordcmd), NV_SS | NV_RL, 0),
    cmd(
        -(253 as c_int + ((KE_C_RIGHT as c_int) << 8 as c_int)),
        Some(nv_wordcmd),
        NV_SSS | NV_RL | NV_STS,
        1,
    ),
    cmd(K_PAGEUP, Some(nv_page), NV_SSS | NV_STS, BACKWARD as c_int),
    cmd(K_KPAGEUP, Some(nv_page), NV_SSS | NV_STS, BACKWARD as c_int),
    cmd(K_PAGEDOWN, Some(nv_page), NV_SSS | NV_STS, FORWARD as c_int),
    cmd(
        K_KPAGEDOWN,
        Some(nv_page),
        NV_SSS | NV_STS,
        FORWARD as c_int,
    ),
    cmd(K_END, Some(nv_end), NV_SSS | NV_STS, 0),
    cmd(K_KEND, Some(nv_end), NV_SSS | NV_STS, 0),
    cmd(K_S_END, Some(nv_end), NV_SS, 0),
    cmd(
        -(253 as c_int + ((KE_C_END as c_int) << 8 as c_int)),
        Some(nv_end),
        NV_SSS | NV_STS,
        1,
    ),
    cmd(K_HOME, Some(nv_home), NV_SSS | NV_STS, 0),
    cmd(K_KHOME, Some(nv_home), NV_SSS | NV_STS, 0),
    cmd(K_S_HOME, Some(nv_home), NV_SS, 0),
    cmd(
        -(253 as c_int + ((KE_C_HOME as c_int) << 8 as c_int)),
        Some(nv_goto),
        NV_SSS | NV_STS,
        0,
    ),
    cmd(K_DEL, Some(nv_abbrev), 0, 0),
    cmd(
        -(253 as c_int + ((KE_KDEL as c_int) << 8 as c_int)),
        Some(nv_abbrev),
        0,
        0,
    ),
    cmd(K_UNDO, Some(nv_kundo), 0, 0),
    cmd(K_HELP, Some(nv_help), NV_NCW, 0),
    cmd(K_F1, Some(nv_help), NV_NCW, 0),
    cmd(
        -(253 as c_int + ((KE_XF1 as c_int) << 8 as c_int)),
        Some(nv_help),
        NV_NCW,
        0,
    ),
    cmd(K_SELECT, Some(nv_select), 0, 0),
    cmd(K_PASTE_START, Some(nv_paste), NV_KEEPREG, 0),
    cmd(
        -(253 as c_int + ((KE_EVENT as c_int) << 8 as c_int)),
        Some(nv_event),
        NV_KEEPREG,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_COMMAND as c_int) << 8 as c_int)),
        Some(nv_colon),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_LUA as c_int) << 8 as c_int)),
        Some(nv_colon),
        0,
        0,
    ),
]);
pub(crate) const NV_CMDS_SIZE: usize = ::core::mem::size_of::<[nv_cmd; 188]>()
    .wrapping_div(::core::mem::size_of::<nv_cmd>())
    .wrapping_div(
        (::core::mem::size_of::<[nv_cmd; 188]>().wrapping_rem(::core::mem::size_of::<nv_cmd>())
            == 0) as c_int as usize,
    );
static nv_cmd_idx: GlobalCell<[int16_t; 188]> = GlobalCell::new([0; 188]);
static nv_max_linear: GlobalCell<c_int> = GlobalCell::new(0);
static current_oap: GlobalCell<*mut oparg_T> = GlobalCell::new(::core::ptr::null_mut::<oparg_T>());
static old_showcmd_buf: GlobalCell<[c_char; 41]> = GlobalCell::new([0; 41]);
static showcmd_is_clear: GlobalCell<bool> = GlobalCell::new(true);
static showcmd_visual: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) const INT_MAX: c_int = __INT_MAX__;
pub(crate) const __INT_MAX__: c_int = 2147483647 as c_int;
