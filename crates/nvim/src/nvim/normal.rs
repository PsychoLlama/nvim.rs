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

use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::mouse::{nv_mouse, nv_mousescroll};
use crate::src::nvim::types::{
    Array, Direction, MarkGet, MarkMove, MarkMoveRes, MotionType, Object, SpellAddType, VimState,
    VimVarIndex, cmdarg_T, getf_values, hlf_T, int16_t, key_extra, oparg_T, pos_T, size_t, smt_T,
    uint16_t,
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
pub use self::ident::*;
mod motion;
pub use self::motion::*;
mod search;
pub(crate) use self::search::*;
mod brackets;
pub(crate) use self::brackets::*;
mod scroll;
pub use self::scroll::*;
mod edit;
pub use self::edit::*;
mod operator;
pub(crate) use self::operator::*;
mod gcmd;
pub use self::gcmd::*;
mod misc;
pub(crate) use self::misc::*;
pub const _ISlower: c_uint = 512;
pub const _ISupper: c_uint = 256;
pub const MAXCOL: c_uint = 2147483647;
pub const HLF_MSG: hlf_T = 63;
pub const HLF_LC: hlf_T = 9;
pub const HLF_L: hlf_T = 8;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const kMarkChangedCursor: MarkMoveRes = 32;
pub const kMarkChangedLine: MarkMoveRes = 16;
pub const kMarkSwitchedBuf: MarkMoveRes = 4;
pub const kMarkMoveFailed: MarkMoveRes = 2;
pub const kMarkMoveSuccess: MarkMoveRes = 1;
pub const kMarkJumpList: MarkMove = 16;
pub const kMarkSetView: MarkMove = 8;
pub const KMarkNoContext: MarkMove = 4;
pub const kMarkContext: MarkMove = 2;
pub const kMarkBeginLine: MarkMove = 1;
pub const kMarkAll: MarkGet = 1;
pub const GETF_ALT: getf_values = 2;
pub const GETF_SETMARK: getf_values = 1;
pub const OPENLINE_DO_COM: c_uint = 2;
pub const SHM_SEARCHCOUNT: c_uint = 83;
pub const SHM_FILEINFO: c_uint = 70;
pub const HIST_SEARCH: c_int = 1;
pub const BL_FIX: c_uint = 4;
pub const BL_SOL: c_uint = 2;
pub const BL_WHITE: c_uint = 1;
pub const VV_OP: VimVarIndex = 55;
pub const ECMD_HIDE: c_uint = 1;
pub const ECMD_LAST: c_int = -1;
pub const DOCMD_KEEPLINE: c_uint = 32;
pub const VSE_NONE: c_uint = 0;
pub const KE_COMMAND: key_extra = 104;
pub const KE_LUA: key_extra = 103;
pub const KE_EVENT: key_extra = 102;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_NOP: key_extra = 97;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_C_END: key_extra = 88;
pub const KE_C_HOME: key_extra = 87;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_CMDWIN: key_extra = 84;
pub const KE_KDEL: key_extra = 80;
pub const KE_KINS: key_extra = 79;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_LEFTRELEASE_NM: key_extra = 70;
pub const KE_LEFTMOUSE_NM: key_extra = 69;
pub const KE_XF1: key_extra = 57;
pub const KE_IGNORE: key_extra = 53;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_RIGHTDRAG: key_extra = 51;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_MIDDLEDRAG: key_extra = 48;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_LEFTMOUSE: key_extra = 44;
pub const KE_S_DOWN: key_extra = 5;
pub const KE_S_UP: key_extra = 4;
pub const ML_DEL_MESSAGE: c_uint = 1;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub const CA_NO_ADJ_OP_END: c_uint = 2;
pub const CA_COMMAND_BUSY: c_uint = 1;
pub const REPLACE_NL_NCHAR: c_int = -2;
pub const REPLACE_CR_NCHAR: c_int = -1;
pub const SHOWCMD_COLS: c_uint = 10;
pub(crate) const SHOWCMD_BUFLEN: c_uint = 41;
pub const MSCR_RIGHT: c_int = -2;
pub const MSCR_LEFT: c_int = -1;
pub const MSCR_UP: c_int = 1;
pub const MSCR_DOWN: c_int = 0;
pub const FIND_EVAL: c_uint = 4;
pub const FIND_STRING: c_uint = 2;
pub const FIND_IDENT: c_uint = 1;
#[derive(Copy, Clone)]
#[repr(C)]
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
pub const OP_NOP: c_uint = 0;
pub const OP_YANK: c_uint = 2;
pub const OP_RSHIFT: c_uint = 5;
pub const OP_LSHIFT: c_uint = 4;
pub const OP_DELETE: c_uint = 1;
pub const PUT_LINE_FORWARD: c_uint = 32;
pub const PUT_LINE_SPLIT: c_uint = 16;
pub const PUT_LINE: c_uint = 8;
pub const PUT_BLOCK_INNER: c_uint = 64;
pub const PUT_CURSEND: c_uint = 2;
pub const PUT_FIXINDENT: c_uint = 1;
pub const SEARCH_START: c_uint = 256;
pub const FM_FORWARD: c_uint = 2;
pub const RE_LAST: c_uint = 2;
pub const SEARCH_MSG: c_uint = 12;
pub const SEARCH_ECHO: c_uint = 2;
pub const SEARCH_OPT: c_uint = 16;
pub const OP_CHANGE: c_uint = 3;
pub const OP_NR_SUB: c_uint = 29;
pub const OP_NR_ADD: c_uint = 28;
pub const OP_TILDE: c_uint = 7;
pub const SPELL_ADD_BAD: SpellAddType = 1;
pub const SPELL_ADD_GOOD: SpellAddType = 0;
pub const SMT_RARE: smt_T = 2;
pub const SMT_BAD: smt_T = 1;
pub const SMT_ALL: smt_T = 0;
pub const OP_FOLD: c_uint = 19;
pub const OP_LOWER: c_uint = 12;
pub const OP_FORMAT: c_uint = 9;
pub const SEARCH_MARK: c_uint = 512;
pub const FM_BACKWARD: c_uint = 1;
pub const ACTION_GOTO: c_uint = 2;
pub const ACTION_SHOW: c_uint = 1;
pub const ACTION_SHOW_ALL: c_uint = 4;
pub const FIND_ANY: c_uint = 1;
pub const FIND_DEFINE: c_uint = 2;
pub const OP_UPPER: c_uint = 11;
pub const SEARCH_REV: c_uint = 1;
pub const OP_ROT13: c_uint = 15;
pub const DT_POP: c_uint = 2;
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
pub const OP_COLON: c_uint = 10;
pub const NULL: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const ML_EMPTY: c_int = 0x1 as c_int;
pub const OK: c_int = 1 as c_int;
pub const FAIL: c_int = 0 as c_int;
pub const NUL: c_int = '\0' as c_int;
pub const TAB: c_int = 9;
pub const NL: c_int = '\n' as c_int;
pub const CAR: c_int = '\r' as c_int;
pub const ESC: c_int = '\u{1b}' as c_int;
pub const DEL: c_int = 0x7f as c_int;
pub(crate) const POUND: c_int = 0xa3 as c_int;
pub const Ctrl_A: c_int = 1 as c_int;
pub const Ctrl_B: c_int = 2 as c_int;
pub const Ctrl_C: c_int = 3 as c_int;
pub const Ctrl_D: c_int = 4 as c_int;
pub const Ctrl_E: c_int = 5 as c_int;
pub const Ctrl_F: c_int = 6 as c_int;
pub const Ctrl_G: c_int = 7;
pub const Ctrl_H: c_int = 8;
pub const Ctrl_I: c_int = 9 as c_int;
pub const Ctrl_K: c_int = 11 as c_int;
pub const Ctrl_L: c_int = 12 as c_int;
pub const Ctrl_N: c_int = 14 as c_int;
pub const Ctrl_O: c_int = 15 as c_int;
pub const Ctrl_P: c_int = 16 as c_int;
pub const Ctrl_Q: c_int = 17 as c_int;
pub const Ctrl_R: c_int = 18 as c_int;
pub const Ctrl_S: c_int = 19 as c_int;
pub const Ctrl_T: c_int = 20 as c_int;
pub const Ctrl_U: c_int = 21 as c_int;
pub const Ctrl_V: c_int = 22 as c_int;
pub const Ctrl_W: c_int = 23 as c_int;
pub const Ctrl_X: c_int = 24;
pub const Ctrl_Y: c_int = 25 as c_int;
pub const Ctrl_Z: c_int = 26 as c_int;
pub const Ctrl_BSL: c_int = 28 as c_int;
pub const Ctrl_RSB: c_int = 29 as c_int;
pub const Ctrl_HAT: c_int = 30 as c_int;
pub const Ctrl__: c_int = 31 as c_int;
pub(crate) const FO_OPEN_COMS: c_int = 'o' as c_int;
pub(crate) const CPO_DIGRAPH: c_int = 'D' as c_int;
pub(crate) const CPO_CHANGEW: c_int = '_' as c_int;
pub const VALID_WCOL: c_int = 0x2 as c_int;
pub const VALID_CROW: c_int = 0x10 as c_int;
pub const B_IMODE_LMAP: c_int = 1 as c_int;
pub const K_ZERO: c_int = -(255 as c_int + (('X' as c_int) << 8 as c_int));
pub const K_UP: c_int = -30059;
pub const K_DOWN: c_int = -25707;
pub const K_LEFT: c_int = -('k' as c_int + (('l' as c_int) << 8 as c_int));
pub const K_RIGHT: c_int = -('k' as c_int + (('r' as c_int) << 8 as c_int));
pub const K_S_LEFT: c_int = -('#' as c_int + (('4' as c_int) << 8 as c_int));
pub const K_S_RIGHT: c_int = -('%' as c_int + (('i' as c_int) << 8 as c_int));
pub const K_S_HOME: c_int = -('#' as c_int + (('2' as c_int) << 8 as c_int));
pub const K_S_END: c_int = -('*' as c_int + (('7' as c_int) << 8 as c_int));
pub const K_F1: c_int = -('k' as c_int + (('1' as c_int) << 8 as c_int));
pub const K_HELP: c_int = -('%' as c_int + (('1' as c_int) << 8 as c_int));
pub const K_UNDO: c_int = -('&' as c_int + (('8' as c_int) << 8 as c_int));
pub const K_BS: c_int = -25195;
pub const K_INS: c_int = -('k' as c_int + (('I' as c_int) << 8 as c_int));
pub const K_DEL: c_int = -('k' as c_int + (('D' as c_int) << 8 as c_int));
pub const K_HOME: c_int = -26731;
pub const K_KHOME: c_int = -12619;
pub const K_END: c_int = -('@' as c_int + (('7' as c_int) << 8 as c_int));
pub const K_KEND: c_int = -('K' as c_int + (('4' as c_int) << 8 as c_int));
pub const K_PAGEUP: c_int = -('k' as c_int + (('P' as c_int) << 8 as c_int));
pub const K_PAGEDOWN: c_int = -('k' as c_int + (('N' as c_int) << 8 as c_int));
pub const K_KPAGEUP: c_int = -('K' as c_int + (('3' as c_int) << 8 as c_int));
pub const K_KPAGEDOWN: c_int = -('K' as c_int + (('5' as c_int) << 8 as c_int));
pub const K_KENTER: c_int = -16715;
pub const K_PASTE_START: c_int = -('P' as c_int + (('S' as c_int) << 8 as c_int));
pub const K_SELECT: c_int = -(245 as c_int + (('X' as c_int) << 8 as c_int));
pub const MOD_MASK_SHIFT: c_int = 0x2 as c_int;
pub const MOD_MASK_CTRL: c_int = 0x4 as c_int;
pub const GRAPHEME_STATE_INIT: c_int = 0 as c_int;
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
    cmd(Ctrl_C, Some(nv_esc), 0, true_0),
    cmd(Ctrl_D, Some(nv_halfpage), 0, 0),
    cmd(Ctrl_E, Some(nv_scroll_line), 0, true_0),
    cmd(Ctrl_F, Some(nv_page), NV_STS, FORWARD as c_int),
    cmd(Ctrl_G, Some(nv_ctrlg), 0, 0),
    cmd(Ctrl_H, Some(nv_ctrlh), 0, 0),
    cmd(Ctrl_I, Some(nv_pcmark), 0, 0),
    cmd(NL, Some(nv_down), 0, false_0),
    cmd(Ctrl_K, Some(nv_error), 0, 0),
    cmd(Ctrl_L, Some(nv_clear), 0, 0),
    cmd(CAR, Some(nv_down), 0, true_0),
    cmd(Ctrl_N, Some(nv_down), NV_STS, false_0),
    cmd(Ctrl_O, Some(nv_ctrlo), 0, 0),
    cmd(Ctrl_P, Some(nv_up), NV_STS, false_0),
    cmd(Ctrl_Q, Some(nv_visual), 0, false_0),
    cmd(Ctrl_R, Some(nv_redo_or_register), 0, 0),
    cmd(Ctrl_S, Some(nv_ignore), 0, 0),
    cmd(Ctrl_T, Some(nv_tagpop), NV_NCW, 0),
    cmd(Ctrl_U, Some(nv_halfpage), 0, 0),
    cmd(Ctrl_V, Some(nv_visual), 0, false_0),
    cmd('V' as c_int, Some(nv_visual), 0, false_0),
    cmd('v' as c_int, Some(nv_visual), 0, false_0),
    cmd(Ctrl_W, Some(nv_window), 0, 0),
    cmd(Ctrl_X, Some(nv_addsub), 0, 0),
    cmd(Ctrl_Y, Some(nv_scroll_line), 0, false_0),
    cmd(Ctrl_Z, Some(nv_suspend), 0, 0),
    cmd(ESC, Some(nv_esc), 0, false_0),
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
    cmd('\'' as c_int, Some(nv_gomark), NV_NCH_ALW, true_0),
    cmd('(' as c_int, Some(nv_brace), 0, BACKWARD as c_int),
    cmd(')' as c_int, Some(nv_brace), 0, FORWARD as c_int),
    cmd('*' as c_int, Some(nv_ident), 0, 0),
    cmd('+' as c_int, Some(nv_down), 0, true_0),
    cmd(',' as c_int, Some(nv_csearch), 0, true_0),
    cmd('-' as c_int, Some(nv_up), 0, true_0),
    cmd('.' as c_int, Some(nv_dot), NV_KEEPREG, 0),
    cmd('/' as c_int, Some(nv_search), 0, false_0),
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
    cmd(';' as c_int, Some(nv_csearch), 0, false_0),
    cmd('<' as c_int, Some(nv_operator), NV_RL, 0),
    cmd('=' as c_int, Some(nv_operator), 0, 0),
    cmd('>' as c_int, Some(nv_operator), NV_RL, 0),
    cmd('?' as c_int, Some(nv_search), 0, false_0),
    cmd('@' as c_int, Some(nv_at), NV_NCH_NOP, false_0),
    cmd('A' as c_int, Some(nv_edit), 0, 0),
    cmd('B' as c_int, Some(nv_bck_word), 0, 1),
    cmd('C' as c_int, Some(nv_abbrev), NV_KEEPREG, 0),
    cmd('D' as c_int, Some(nv_abbrev), NV_KEEPREG, 0),
    cmd('E' as c_int, Some(nv_wordcmd), 0, true_0),
    cmd(
        'F' as c_int,
        Some(nv_csearch),
        NV_NCH_ALW | NV_LANG,
        BACKWARD as c_int,
    ),
    cmd('G' as c_int, Some(nv_goto), 0, true_0),
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
    cmd('R' as c_int, Some(nv_Replace), 0, false_0),
    cmd('S' as c_int, Some(nv_subst), NV_KEEPREG, 0),
    cmd(
        'T' as c_int,
        Some(nv_csearch),
        NV_NCH_ALW | NV_LANG,
        BACKWARD as c_int,
    ),
    cmd('U' as c_int, Some(nv_Undo), 0, 0),
    cmd('W' as c_int, Some(nv_wordcmd), 0, true_0),
    cmd('X' as c_int, Some(nv_abbrev), NV_KEEPREG, 0),
    cmd('Y' as c_int, Some(nv_abbrev), NV_KEEPREG, 0),
    cmd('Z' as c_int, Some(nv_Zet), NV_NCH_NOP | NV_NCW, 0),
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
        BL_WHITE as c_int | BL_FIX as c_int,
    ),
    cmd('_' as c_int, Some(nv_lineop), 0, 0),
    cmd('`' as c_int, Some(nv_gomark), NV_NCH_ALW, false_0),
    cmd('a' as c_int, Some(nv_edit), NV_NCH, 0),
    cmd('b' as c_int, Some(nv_bck_word), 0, 0),
    cmd('c' as c_int, Some(nv_operator), 0, 0),
    cmd('d' as c_int, Some(nv_operator), 0, 0),
    cmd('e' as c_int, Some(nv_wordcmd), 0, false_0),
    cmd(
        'f' as c_int,
        Some(nv_csearch),
        NV_NCH_ALW | NV_LANG,
        FORWARD as c_int,
    ),
    cmd('g' as c_int, Some(nv_g_cmd), NV_NCH_ALW, false_0),
    cmd('h' as c_int, Some(nv_left), NV_RL, 0),
    cmd('i' as c_int, Some(nv_edit), NV_NCH, 0),
    cmd('j' as c_int, Some(nv_down), 0, false_0),
    cmd('k' as c_int, Some(nv_up), 0, false_0),
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
    cmd('w' as c_int, Some(nv_wordcmd), 0, false_0),
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
    cmd(K_UP, Some(nv_up), NV_SSS | NV_STS, false_0),
    cmd(
        -(253 as c_int + ((KE_S_UP as c_int) << 8 as c_int)),
        Some(nv_page),
        NV_SS,
        BACKWARD as c_int,
    ),
    cmd(K_DOWN, Some(nv_down), NV_SSS | NV_STS, false_0),
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
    cmd(K_S_RIGHT, Some(nv_wordcmd), NV_SS | NV_RL, false_0),
    cmd(
        -(253 as c_int + ((KE_C_RIGHT as c_int) << 8 as c_int)),
        Some(nv_wordcmd),
        NV_SSS | NV_RL | NV_STS,
        true_0,
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
    cmd(K_END, Some(nv_end), NV_SSS | NV_STS, false_0),
    cmd(K_KEND, Some(nv_end), NV_SSS | NV_STS, false_0),
    cmd(K_S_END, Some(nv_end), NV_SS, false_0),
    cmd(
        -(253 as c_int + ((KE_C_END as c_int) << 8 as c_int)),
        Some(nv_end),
        NV_SSS | NV_STS,
        true_0,
    ),
    cmd(K_HOME, Some(nv_home), NV_SSS | NV_STS, 0),
    cmd(K_KHOME, Some(nv_home), NV_SSS | NV_STS, 0),
    cmd(K_S_HOME, Some(nv_home), NV_SS, 0),
    cmd(
        -(253 as c_int + ((KE_C_HOME as c_int) << 8 as c_int)),
        Some(nv_goto),
        NV_SSS | NV_STS,
        false_0,
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
static showcmd_is_clear: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
static showcmd_visual: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub const INT_MAX: c_int = __INT_MAX__;
pub const true_0: c_int = 1 as c_int;
pub const false_0: c_int = 0 as c_int;
pub const __INT_MAX__: c_int = 2147483647 as c_int;
