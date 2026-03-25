//! Normal mode command dispatch table.
//!
//! This module provides the Rust representation of the `nv_cmds[]` command table
//! from `src/nvim/normal.c`. The table contains one entry for every Normal or
//! Visual mode command.
//!
//! The actual command handlers remain as C functions - this table only contains
//! metadata (flags, arguments) and command character information.

#![allow(clippy::cast_possible_truncation)] // Intentional casts for small constants
#![allow(clippy::cast_sign_loss)] // Intentional casts for flag values
#![allow(clippy::cast_possible_wrap)] // Intentional casts in index initialization
#![allow(clippy::cast_lossless)] // Allow b'x' as c_int for const contexts

use std::ffi::c_int;

#[cfg(not(test))]
use crate::types::CmdargT;

use super::constants::{
    BACKWARD, FORWARD, NV_KEEPREG, NV_LANG, NV_NCH, NV_NCH_ALW, NV_NCH_NOP, NV_NCW, NV_RL, NV_SS,
    NV_SSS, NV_STS,
};

// =============================================================================
// Key Constants (from keycodes.h and ascii_defs.h)
// =============================================================================

/// Convert termcap codes to internal key representation.
/// TERMCAP2KEY(a, b) = -((a) + ((int)(b) << 8))
const fn termcap2key(a: c_int, b: c_int) -> c_int {
    -((a) + (b << 8))
}

// Special key source bytes
const KS_EXTRA: c_int = 253;
#[allow(dead_code)]
const KS_MOUSE: c_int = 251;
const KS_SELECT: c_int = 245;

// KE_* values for special keys
const KE_S_UP: c_int = 4;
const KE_S_DOWN: c_int = 5;
const KE_LEFTMOUSE: c_int = 44;
const KE_LEFTDRAG: c_int = 46;
const KE_LEFTRELEASE: c_int = 45;
const KE_MIDDLEMOUSE: c_int = 47;
const KE_MIDDLEDRAG: c_int = 49;
const KE_MIDDLERELEASE: c_int = 48;
const KE_RIGHTMOUSE: c_int = 50;
const KE_RIGHTDRAG: c_int = 52;
const KE_RIGHTRELEASE: c_int = 51;
const KE_IGNORE: c_int = 53;
const KE_TAB: c_int = 54;
const KE_XF1: c_int = 57;
const KE_LEFTMOUSE_NM: c_int = 69;
const KE_LEFTRELEASE_NM: c_int = 70;
const KE_MOUSEDOWN: c_int = 75;
const KE_MOUSEUP: c_int = 76;
const KE_MOUSELEFT: c_int = 77;
const KE_MOUSERIGHT: c_int = 78;
const KE_KINS: c_int = 79;
const KE_KDEL: c_int = 80;
const KE_C_LEFT: c_int = 85;
const KE_C_RIGHT: c_int = 86;
const KE_C_HOME: c_int = 87;
const KE_C_END: c_int = 88;
const KE_X1MOUSE: c_int = 89;
const KE_X1DRAG: c_int = 90;
const KE_X1RELEASE: c_int = 91;
const KE_X2MOUSE: c_int = 92;
const KE_X2DRAG: c_int = 93;
const KE_X2RELEASE: c_int = 94;
const KE_NOP: c_int = 97;
const KE_MOUSEMOVE: c_int = 100;
const KE_EVENT: c_int = 102;
const KE_LUA: c_int = 103;
const KE_COMMAND: c_int = 104;

// Control characters
const NUL: c_int = 0;
const CTRL_A: c_int = 1;
const CTRL_B: c_int = 2;
const CTRL_C: c_int = 3;
const CTRL_D: c_int = 4;
const CTRL_E: c_int = 5;
const CTRL_F: c_int = 6;
const CTRL_G: c_int = 7;
const CTRL_H: c_int = 8;
const CTRL_I: c_int = 9;
const NL: c_int = 10; // Ctrl-J
const CTRL_K: c_int = 11;
const CTRL_L: c_int = 12;
const CAR: c_int = 13; // Ctrl-M, Carriage Return
const CTRL_N: c_int = 14;
const CTRL_O: c_int = 15;
const CTRL_P: c_int = 16;
const CTRL_Q: c_int = 17;
const CTRL_R: c_int = 18;
const CTRL_S: c_int = 19;
const CTRL_T: c_int = 20;
const CTRL_U: c_int = 21;
const CTRL_V: c_int = 22;
const CTRL_W: c_int = 23;
const CTRL_X: c_int = 24;
const CTRL_Y: c_int = 25;
const CTRL_Z: c_int = 26;
const ESC: c_int = 27;
const CTRL_BSL: c_int = 28; // Ctrl-\
const CTRL_RSB: c_int = 29; // Ctrl-]
const CTRL_HAT: c_int = 30; // Ctrl-^
const CTRL__: c_int = 31; // Ctrl-_

// Pound sign
const POUND: c_int = 0xA3;

// Special keys
const K_UP: c_int = termcap2key(b'k' as c_int, b'u' as c_int);
#[allow(dead_code)]
const K_KUP: c_int = termcap2key(b'K' as c_int, b'u' as c_int);
const K_DOWN: c_int = termcap2key(b'k' as c_int, b'd' as c_int);
#[allow(dead_code)]
const K_KDOWN: c_int = termcap2key(b'K' as c_int, b'd' as c_int);
const K_LEFT: c_int = termcap2key(b'k' as c_int, b'l' as c_int);
#[allow(dead_code)]
const K_KLEFT: c_int = termcap2key(b'K' as c_int, b'l' as c_int);
const K_RIGHT: c_int = termcap2key(b'k' as c_int, b'r' as c_int);
#[allow(dead_code)]
const K_KRIGHT: c_int = termcap2key(b'K' as c_int, b'r' as c_int);
const K_S_UP: c_int = termcap2key(KS_EXTRA, KE_S_UP);
const K_S_DOWN: c_int = termcap2key(KS_EXTRA, KE_S_DOWN);
const K_S_LEFT: c_int = termcap2key(b'#' as c_int, b'4' as c_int);
const K_C_LEFT: c_int = termcap2key(KS_EXTRA, KE_C_LEFT);
const K_S_RIGHT: c_int = termcap2key(b'%' as c_int, b'i' as c_int);
const K_C_RIGHT: c_int = termcap2key(KS_EXTRA, KE_C_RIGHT);
const K_S_HOME: c_int = termcap2key(b'#' as c_int, b'2' as c_int);
const K_C_HOME: c_int = termcap2key(KS_EXTRA, KE_C_HOME);
const K_S_END: c_int = termcap2key(b'*' as c_int, b'7' as c_int);
const K_C_END: c_int = termcap2key(KS_EXTRA, KE_C_END);
#[allow(dead_code)]
const K_TAB: c_int = termcap2key(KS_EXTRA, KE_TAB);
#[allow(dead_code)]
const K_S_TAB: c_int = termcap2key(b'k' as c_int, b'B' as c_int);
const K_F1: c_int = termcap2key(b'k' as c_int, b'1' as c_int);
const K_XF1: c_int = termcap2key(KS_EXTRA, KE_XF1);
const K_HELP: c_int = termcap2key(b'%' as c_int, b'1' as c_int);
const K_UNDO: c_int = termcap2key(b'&' as c_int, b'8' as c_int);
const K_BS: c_int = termcap2key(b'k' as c_int, b'b' as c_int);
const K_INS: c_int = termcap2key(b'k' as c_int, b'I' as c_int);
const K_KINS: c_int = termcap2key(KS_EXTRA, KE_KINS);
const K_DEL: c_int = termcap2key(b'k' as c_int, b'D' as c_int);
const K_KDEL: c_int = termcap2key(KS_EXTRA, KE_KDEL);
const K_HOME: c_int = termcap2key(b'k' as c_int, b'h' as c_int);
const K_KHOME: c_int = termcap2key(b'K' as c_int, b'1' as c_int);
const K_END: c_int = termcap2key(b'@' as c_int, b'7' as c_int);
const K_KEND: c_int = termcap2key(b'K' as c_int, b'4' as c_int);
const K_PAGEUP: c_int = termcap2key(b'k' as c_int, b'P' as c_int);
const K_KPAGEUP: c_int = termcap2key(b'K' as c_int, b'3' as c_int);
const K_PAGEDOWN: c_int = termcap2key(b'k' as c_int, b'N' as c_int);
const K_KPAGEDOWN: c_int = termcap2key(b'K' as c_int, b'5' as c_int);
const K_PASTE_START: c_int = termcap2key(b'P' as c_int, b'S' as c_int);
#[allow(dead_code)]
const K_MOUSE: c_int = termcap2key(KS_MOUSE, b'X' as c_int);
const K_SELECT: c_int = termcap2key(KS_SELECT, b'X' as c_int);
const K_MOUSEMOVE: c_int = termcap2key(KS_EXTRA, KE_MOUSEMOVE);
const K_IGNORE: c_int = termcap2key(KS_EXTRA, KE_IGNORE);
const K_NOP: c_int = termcap2key(KS_EXTRA, KE_NOP);
const K_MOUSEDOWN: c_int = termcap2key(KS_EXTRA, KE_MOUSEDOWN);
const K_MOUSEUP: c_int = termcap2key(KS_EXTRA, KE_MOUSEUP);
const K_MOUSELEFT: c_int = termcap2key(KS_EXTRA, KE_MOUSELEFT);
const K_MOUSERIGHT: c_int = termcap2key(KS_EXTRA, KE_MOUSERIGHT);
const K_LEFTMOUSE: c_int = termcap2key(KS_EXTRA, KE_LEFTMOUSE);
const K_LEFTMOUSE_NM: c_int = termcap2key(KS_EXTRA, KE_LEFTMOUSE_NM);
const K_LEFTDRAG: c_int = termcap2key(KS_EXTRA, KE_LEFTDRAG);
const K_LEFTRELEASE: c_int = termcap2key(KS_EXTRA, KE_LEFTRELEASE);
const K_LEFTRELEASE_NM: c_int = termcap2key(KS_EXTRA, KE_LEFTRELEASE_NM);
const K_MIDDLEMOUSE: c_int = termcap2key(KS_EXTRA, KE_MIDDLEMOUSE);
const K_MIDDLEDRAG: c_int = termcap2key(KS_EXTRA, KE_MIDDLEDRAG);
const K_MIDDLERELEASE: c_int = termcap2key(KS_EXTRA, KE_MIDDLERELEASE);
const K_RIGHTMOUSE: c_int = termcap2key(KS_EXTRA, KE_RIGHTMOUSE);
const K_RIGHTDRAG: c_int = termcap2key(KS_EXTRA, KE_RIGHTDRAG);
const K_RIGHTRELEASE: c_int = termcap2key(KS_EXTRA, KE_RIGHTRELEASE);
const K_X1MOUSE: c_int = termcap2key(KS_EXTRA, KE_X1MOUSE);
const K_X1DRAG: c_int = termcap2key(KS_EXTRA, KE_X1DRAG);
const K_X1RELEASE: c_int = termcap2key(KS_EXTRA, KE_X1RELEASE);
const K_X2MOUSE: c_int = termcap2key(KS_EXTRA, KE_X2MOUSE);
const K_X2DRAG: c_int = termcap2key(KS_EXTRA, KE_X2DRAG);
const K_X2RELEASE: c_int = termcap2key(KS_EXTRA, KE_X2RELEASE);
const K_EVENT: c_int = termcap2key(KS_EXTRA, KE_EVENT);
const K_COMMAND: c_int = termcap2key(KS_EXTRA, KE_COMMAND);
const K_LUA: c_int = termcap2key(KS_EXTRA, KE_LUA);

// Scroll directions
const MSCR_UP: c_int = 0;
const MSCR_DOWN: c_int = 1;
const MSCR_LEFT: c_int = 2;
const MSCR_RIGHT: c_int = 3;

// Beginline flags
const BL_WHITE: c_int = 1;
const BL_FIX: c_int = 2;

// Search directions
const SEARCH_REV: c_int = 0x02;

// =============================================================================
// Command Handler Identifiers
// =============================================================================

/// Identifies which command handler function to call.
///
/// Each variant corresponds to an `nv_*` function in normal.c.
/// The actual handlers remain as C functions - we just need to know
/// which one to dispatch to.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdHandler {
    Error = 0,
    Ignore,
    Nop,
    AddsUb,
    Page,
    Esc,
    HalfPage,
    ScrollLine,
    Ctrlg,
    Ctrlh,
    Pcmark,
    Down,
    Clear,
    Up,
    Visual,
    Window,
    Suspend,
    Normal,
    Ident,
    Hat,
    Right,
    Left,
    Operator,
    Regname,
    Dollar,
    Percent,
    Optrans,
    Gomark,
    Brace,
    Csearch,
    Dot,
    Search,
    Beginline,
    Colon,
    Next,
    At,
    Edit,
    BckWord,
    Abbrev,
    Wordcmd,
    Goto,
    Scroll,
    Join,
    Open,
    Put,
    Regreplay,
    Replace,
    Subst,
    Undo,
    Zet,
    SmallZet,
    Brackets,
    Lineop,
    Tilde,
    Findpar,
    Pipe,
    Mark,
    Record,
    RedoOrRegister,
    ReplaceMode,
    UndoLine,
    GCmd,
    Mouse,
    Mousescroll,
    End,
    Home,
    Kundo,
    Help,
    Select,
    Paste,
    Event,
    Ctrlo,
    Tagpop,
    Object,
}

// =============================================================================
// Command Entry Structure
// =============================================================================

/// A single entry in the command table.
///
/// This mirrors the C `struct nv_cmd` from normal.c.
#[derive(Debug, Clone, Copy)]
pub struct NvCmd {
    /// Command character (first character).
    pub cmd_char: c_int,
    /// Command handler identifier.
    pub handler: CmdHandler,
    /// NV_* flags.
    pub flags: u16,
    /// Value for ca.arg.
    pub arg: i16,
}

impl NvCmd {
    /// Create a new command entry.
    const fn new(cmd_char: c_int, handler: CmdHandler, flags: u16, arg: i16) -> Self {
        Self {
            cmd_char,
            handler,
            flags,
            arg,
        }
    }

    /// Check if this command needs a second character.
    #[inline]
    #[must_use]
    pub const fn needs_second_char(&self) -> bool {
        (self.flags as c_int & NV_NCH) != 0
    }

    /// Check if this command needs a second char only when no operator pending.
    #[inline]
    #[must_use]
    pub const fn needs_second_char_no_op(&self) -> bool {
        (self.flags as c_int & NV_NCH_NOP) == NV_NCH_NOP
    }

    /// Check if this command always needs a second char.
    #[inline]
    #[must_use]
    pub const fn needs_second_char_always(&self) -> bool {
        (self.flags as c_int & NV_NCH_ALW) == NV_NCH_ALW
    }

    /// Check if second character needs language adjustment.
    #[inline]
    #[must_use]
    pub const fn needs_lang_adjust(&self) -> bool {
        (self.flags as c_int & NV_LANG) != 0
    }

    /// Check if command may start selection.
    #[inline]
    #[must_use]
    pub const fn may_start_selection(&self) -> bool {
        (self.flags as c_int & NV_SS) != 0
    }

    /// Check if command may start selection with shift.
    #[inline]
    #[must_use]
    pub const fn may_start_selection_shift(&self) -> bool {
        (self.flags as c_int & NV_SSS) != 0
    }

    /// Check if command may stop selection without shift.
    #[inline]
    #[must_use]
    pub const fn may_stop_selection(&self) -> bool {
        (self.flags as c_int & NV_STS) != 0
    }

    /// Check if rightleft modifies command.
    #[inline]
    #[must_use]
    pub const fn is_rightleft_modified(&self) -> bool {
        (self.flags as c_int & NV_RL) != 0
    }

    /// Check if command keeps regname.
    #[inline]
    #[must_use]
    pub const fn keeps_regname(&self) -> bool {
        (self.flags as c_int & NV_KEEPREG) != 0
    }

    /// Check if command not allowed in cmdline window.
    #[inline]
    #[must_use]
    pub const fn not_in_cmdwin(&self) -> bool {
        (self.flags as c_int & NV_NCW) != 0
    }
}

// =============================================================================
// Command Table
// =============================================================================

/// Flag helper - casts combined constants to u16.
const fn f(val: c_int) -> u16 {
    val as u16
}

/// The command table - contains one entry for every Normal or Visual mode command.
///
/// The order matches the C `nv_cmds[]` table in normal.c.
/// This array will be sorted at initialization time.
pub static NV_CMDS: [NvCmd; 188] = [
    // NUL
    NvCmd::new(NUL, CmdHandler::Error, 0, 0),
    // Ctrl-A
    NvCmd::new(CTRL_A, CmdHandler::AddsUb, 0, 0),
    // Ctrl-B
    NvCmd::new(CTRL_B, CmdHandler::Page, f(NV_STS), BACKWARD as i16),
    // Ctrl-C
    NvCmd::new(CTRL_C, CmdHandler::Esc, 0, 1),
    // Ctrl-D
    NvCmd::new(CTRL_D, CmdHandler::HalfPage, 0, 0),
    // Ctrl-E
    NvCmd::new(CTRL_E, CmdHandler::ScrollLine, 0, 1),
    // Ctrl-F
    NvCmd::new(CTRL_F, CmdHandler::Page, f(NV_STS), FORWARD as i16),
    // Ctrl-G
    NvCmd::new(CTRL_G, CmdHandler::Ctrlg, 0, 0),
    // Ctrl-H (Backspace)
    NvCmd::new(CTRL_H, CmdHandler::Ctrlh, 0, 0),
    // Ctrl-I (Tab)
    NvCmd::new(CTRL_I, CmdHandler::Pcmark, 0, 0),
    // NL (Ctrl-J)
    NvCmd::new(NL, CmdHandler::Down, 0, 0),
    // Ctrl-K
    NvCmd::new(CTRL_K, CmdHandler::Error, 0, 0),
    // Ctrl-L
    NvCmd::new(CTRL_L, CmdHandler::Clear, 0, 0),
    // CAR (Ctrl-M, Enter)
    NvCmd::new(CAR, CmdHandler::Down, 0, 1),
    // Ctrl-N
    NvCmd::new(CTRL_N, CmdHandler::Down, f(NV_STS), 0),
    // Ctrl-O
    NvCmd::new(CTRL_O, CmdHandler::Ctrlo, 0, 0),
    // Ctrl-P
    NvCmd::new(CTRL_P, CmdHandler::Up, f(NV_STS), 0),
    // Ctrl-Q
    NvCmd::new(CTRL_Q, CmdHandler::Visual, 0, 0),
    // Ctrl-R
    NvCmd::new(CTRL_R, CmdHandler::RedoOrRegister, 0, 0),
    // Ctrl-S
    NvCmd::new(CTRL_S, CmdHandler::Ignore, 0, 0),
    // Ctrl-T
    NvCmd::new(CTRL_T, CmdHandler::Tagpop, f(NV_NCW), 0),
    // Ctrl-U
    NvCmd::new(CTRL_U, CmdHandler::HalfPage, 0, 0),
    // Ctrl-V
    NvCmd::new(CTRL_V, CmdHandler::Visual, 0, 0),
    // 'V'
    NvCmd::new(b'V' as c_int, CmdHandler::Visual, 0, 0),
    // 'v'
    NvCmd::new(b'v' as c_int, CmdHandler::Visual, 0, 0),
    // Ctrl-W
    NvCmd::new(CTRL_W, CmdHandler::Window, 0, 0),
    // Ctrl-X
    NvCmd::new(CTRL_X, CmdHandler::AddsUb, 0, 0),
    // Ctrl-Y
    NvCmd::new(CTRL_Y, CmdHandler::ScrollLine, 0, 0),
    // Ctrl-Z
    NvCmd::new(CTRL_Z, CmdHandler::Suspend, 0, 0),
    // ESC
    NvCmd::new(ESC, CmdHandler::Esc, 0, 0),
    // Ctrl-\ (Ctrl-BSL)
    NvCmd::new(CTRL_BSL, CmdHandler::Normal, f(NV_NCH_ALW), 0),
    // Ctrl-] (Ctrl-RSB)
    NvCmd::new(CTRL_RSB, CmdHandler::Ident, f(NV_NCW), 0),
    // Ctrl-^ (Ctrl-HAT)
    NvCmd::new(CTRL_HAT, CmdHandler::Hat, f(NV_NCW), 0),
    // Ctrl-_
    NvCmd::new(CTRL__, CmdHandler::Error, 0, 0),
    // Space
    NvCmd::new(b' ' as c_int, CmdHandler::Right, 0, 0),
    // '!'
    NvCmd::new(b'!' as c_int, CmdHandler::Operator, 0, 0),
    // '"'
    NvCmd::new(
        b'"' as c_int,
        CmdHandler::Regname,
        f(NV_NCH_NOP | NV_KEEPREG),
        0,
    ),
    // '#'
    NvCmd::new(b'#' as c_int, CmdHandler::Ident, 0, 0),
    // '$'
    NvCmd::new(b'$' as c_int, CmdHandler::Dollar, 0, 0),
    // '%'
    NvCmd::new(b'%' as c_int, CmdHandler::Percent, 0, 0),
    // '&'
    NvCmd::new(b'&' as c_int, CmdHandler::Optrans, 0, 0),
    // '\''
    NvCmd::new(b'\'' as c_int, CmdHandler::Gomark, f(NV_NCH_ALW), 1),
    // '('
    NvCmd::new(b'(' as c_int, CmdHandler::Brace, 0, BACKWARD as i16),
    // ')'
    NvCmd::new(b')' as c_int, CmdHandler::Brace, 0, FORWARD as i16),
    // '*'
    NvCmd::new(b'*' as c_int, CmdHandler::Ident, 0, 0),
    // '+'
    NvCmd::new(b'+' as c_int, CmdHandler::Down, 0, 1),
    // ','
    NvCmd::new(b',' as c_int, CmdHandler::Csearch, 0, 1),
    // '-'
    NvCmd::new(b'-' as c_int, CmdHandler::Up, 0, 1),
    // '.'
    NvCmd::new(b'.' as c_int, CmdHandler::Dot, f(NV_KEEPREG), 0),
    // '/'
    NvCmd::new(b'/' as c_int, CmdHandler::Search, 0, 0),
    // '0'
    NvCmd::new(b'0' as c_int, CmdHandler::Beginline, 0, 0),
    // '1' - '9': ignored (count digits)
    NvCmd::new(b'1' as c_int, CmdHandler::Ignore, 0, 0),
    NvCmd::new(b'2' as c_int, CmdHandler::Ignore, 0, 0),
    NvCmd::new(b'3' as c_int, CmdHandler::Ignore, 0, 0),
    NvCmd::new(b'4' as c_int, CmdHandler::Ignore, 0, 0),
    NvCmd::new(b'5' as c_int, CmdHandler::Ignore, 0, 0),
    NvCmd::new(b'6' as c_int, CmdHandler::Ignore, 0, 0),
    NvCmd::new(b'7' as c_int, CmdHandler::Ignore, 0, 0),
    NvCmd::new(b'8' as c_int, CmdHandler::Ignore, 0, 0),
    NvCmd::new(b'9' as c_int, CmdHandler::Ignore, 0, 0),
    // ':'
    NvCmd::new(b':' as c_int, CmdHandler::Colon, 0, 0),
    // ';'
    NvCmd::new(b';' as c_int, CmdHandler::Csearch, 0, 0),
    // '<'
    NvCmd::new(b'<' as c_int, CmdHandler::Operator, f(NV_RL), 0),
    // '='
    NvCmd::new(b'=' as c_int, CmdHandler::Operator, 0, 0),
    // '>'
    NvCmd::new(b'>' as c_int, CmdHandler::Operator, f(NV_RL), 0),
    // '?'
    NvCmd::new(b'?' as c_int, CmdHandler::Search, 0, 0),
    // '@'
    NvCmd::new(b'@' as c_int, CmdHandler::At, f(NV_NCH_NOP), 0),
    // 'A'
    NvCmd::new(b'A' as c_int, CmdHandler::Edit, 0, 0),
    // 'B'
    NvCmd::new(b'B' as c_int, CmdHandler::BckWord, 0, 1),
    // 'C'
    NvCmd::new(b'C' as c_int, CmdHandler::Abbrev, f(NV_KEEPREG), 0),
    // 'D'
    NvCmd::new(b'D' as c_int, CmdHandler::Abbrev, f(NV_KEEPREG), 0),
    // 'E'
    NvCmd::new(b'E' as c_int, CmdHandler::Wordcmd, 0, 1),
    // 'F'
    NvCmd::new(
        b'F' as c_int,
        CmdHandler::Csearch,
        f(NV_NCH_ALW | NV_LANG),
        BACKWARD as i16,
    ),
    // 'G'
    NvCmd::new(b'G' as c_int, CmdHandler::Goto, 0, 1),
    // 'H'
    NvCmd::new(b'H' as c_int, CmdHandler::Scroll, 0, 0),
    // 'I'
    NvCmd::new(b'I' as c_int, CmdHandler::Edit, 0, 0),
    // 'J'
    NvCmd::new(b'J' as c_int, CmdHandler::Join, 0, 0),
    // 'K'
    NvCmd::new(b'K' as c_int, CmdHandler::Ident, 0, 0),
    // 'L'
    NvCmd::new(b'L' as c_int, CmdHandler::Scroll, 0, 0),
    // 'M'
    NvCmd::new(b'M' as c_int, CmdHandler::Scroll, 0, 0),
    // 'N'
    NvCmd::new(b'N' as c_int, CmdHandler::Next, 0, SEARCH_REV as i16),
    // 'O'
    NvCmd::new(b'O' as c_int, CmdHandler::Open, 0, 0),
    // 'P'
    NvCmd::new(b'P' as c_int, CmdHandler::Put, 0, 0),
    // 'Q'
    NvCmd::new(b'Q' as c_int, CmdHandler::Regreplay, 0, 0),
    // 'R'
    NvCmd::new(b'R' as c_int, CmdHandler::ReplaceMode, 0, 0),
    // 'S'
    NvCmd::new(b'S' as c_int, CmdHandler::Subst, f(NV_KEEPREG), 0),
    // 'T'
    NvCmd::new(
        b'T' as c_int,
        CmdHandler::Csearch,
        f(NV_NCH_ALW | NV_LANG),
        BACKWARD as i16,
    ),
    // 'U'
    NvCmd::new(b'U' as c_int, CmdHandler::UndoLine, 0, 0),
    // 'W'
    NvCmd::new(b'W' as c_int, CmdHandler::Wordcmd, 0, 1),
    // 'X'
    NvCmd::new(b'X' as c_int, CmdHandler::Abbrev, f(NV_KEEPREG), 0),
    // 'Y'
    NvCmd::new(b'Y' as c_int, CmdHandler::Abbrev, f(NV_KEEPREG), 0),
    // 'Z'
    NvCmd::new(b'Z' as c_int, CmdHandler::Zet, f(NV_NCH_NOP | NV_NCW), 0),
    // '['
    NvCmd::new(
        b'[' as c_int,
        CmdHandler::Brackets,
        f(NV_NCH_ALW),
        BACKWARD as i16,
    ),
    // '\\'
    NvCmd::new(b'\\' as c_int, CmdHandler::Error, 0, 0),
    // ']'
    NvCmd::new(
        b']' as c_int,
        CmdHandler::Brackets,
        f(NV_NCH_ALW),
        FORWARD as i16,
    ),
    // '^'
    NvCmd::new(
        b'^' as c_int,
        CmdHandler::Beginline,
        0,
        (BL_WHITE | BL_FIX) as i16,
    ),
    // '_'
    NvCmd::new(b'_' as c_int, CmdHandler::Lineop, 0, 0),
    // '`'
    NvCmd::new(b'`' as c_int, CmdHandler::Gomark, f(NV_NCH_ALW), 0),
    // 'a'
    NvCmd::new(b'a' as c_int, CmdHandler::Edit, f(NV_NCH), 0),
    // 'b'
    NvCmd::new(b'b' as c_int, CmdHandler::BckWord, 0, 0),
    // 'c'
    NvCmd::new(b'c' as c_int, CmdHandler::Operator, 0, 0),
    // 'd'
    NvCmd::new(b'd' as c_int, CmdHandler::Operator, 0, 0),
    // 'e'
    NvCmd::new(b'e' as c_int, CmdHandler::Wordcmd, 0, 0),
    // 'f'
    NvCmd::new(
        b'f' as c_int,
        CmdHandler::Csearch,
        f(NV_NCH_ALW | NV_LANG),
        FORWARD as i16,
    ),
    // 'g'
    NvCmd::new(b'g' as c_int, CmdHandler::GCmd, f(NV_NCH_ALW), 0),
    // 'h'
    NvCmd::new(b'h' as c_int, CmdHandler::Left, f(NV_RL), 0),
    // 'i'
    NvCmd::new(b'i' as c_int, CmdHandler::Edit, f(NV_NCH), 0),
    // 'j'
    NvCmd::new(b'j' as c_int, CmdHandler::Down, 0, 0),
    // 'k'
    NvCmd::new(b'k' as c_int, CmdHandler::Up, 0, 0),
    // 'l'
    NvCmd::new(b'l' as c_int, CmdHandler::Right, f(NV_RL), 0),
    // 'm'
    NvCmd::new(b'm' as c_int, CmdHandler::Mark, f(NV_NCH_NOP), 0),
    // 'n'
    NvCmd::new(b'n' as c_int, CmdHandler::Next, 0, 0),
    // 'o'
    NvCmd::new(b'o' as c_int, CmdHandler::Open, 0, 0),
    // 'p'
    NvCmd::new(b'p' as c_int, CmdHandler::Put, 0, 0),
    // 'q'
    NvCmd::new(b'q' as c_int, CmdHandler::Record, f(NV_NCH), 0),
    // 'r'
    NvCmd::new(
        b'r' as c_int,
        CmdHandler::Replace,
        f(NV_NCH_NOP | NV_LANG),
        0,
    ),
    // 's'
    NvCmd::new(b's' as c_int, CmdHandler::Subst, f(NV_KEEPREG), 0),
    // 't'
    NvCmd::new(
        b't' as c_int,
        CmdHandler::Csearch,
        f(NV_NCH_ALW | NV_LANG),
        FORWARD as i16,
    ),
    // 'u'
    NvCmd::new(b'u' as c_int, CmdHandler::Undo, 0, 0),
    // 'w'
    NvCmd::new(b'w' as c_int, CmdHandler::Wordcmd, 0, 0),
    // 'x'
    NvCmd::new(b'x' as c_int, CmdHandler::Abbrev, f(NV_KEEPREG), 0),
    // 'y'
    NvCmd::new(b'y' as c_int, CmdHandler::Operator, 0, 0),
    // 'z'
    NvCmd::new(b'z' as c_int, CmdHandler::SmallZet, f(NV_NCH_ALW), 0),
    // '{'
    NvCmd::new(b'{' as c_int, CmdHandler::Findpar, 0, BACKWARD as i16),
    // '|'
    NvCmd::new(b'|' as c_int, CmdHandler::Pipe, 0, 0),
    // '}'
    NvCmd::new(b'}' as c_int, CmdHandler::Findpar, 0, FORWARD as i16),
    // '~'
    NvCmd::new(b'~' as c_int, CmdHandler::Tilde, 0, 0),
    // POUND sign
    NvCmd::new(POUND, CmdHandler::Ident, 0, 0),
    // Mouse scroll keys
    NvCmd::new(K_MOUSEUP, CmdHandler::Mousescroll, 0, MSCR_UP as i16),
    NvCmd::new(K_MOUSEDOWN, CmdHandler::Mousescroll, 0, MSCR_DOWN as i16),
    NvCmd::new(K_MOUSELEFT, CmdHandler::Mousescroll, 0, MSCR_LEFT as i16),
    NvCmd::new(K_MOUSERIGHT, CmdHandler::Mousescroll, 0, MSCR_RIGHT as i16),
    // Mouse keys
    NvCmd::new(K_LEFTMOUSE, CmdHandler::Mouse, 0, 0),
    NvCmd::new(K_LEFTMOUSE_NM, CmdHandler::Mouse, 0, 0),
    NvCmd::new(K_LEFTDRAG, CmdHandler::Mouse, 0, 0),
    NvCmd::new(K_LEFTRELEASE, CmdHandler::Mouse, 0, 0),
    NvCmd::new(K_LEFTRELEASE_NM, CmdHandler::Mouse, 0, 0),
    NvCmd::new(K_MOUSEMOVE, CmdHandler::Mouse, 0, 0),
    NvCmd::new(K_MIDDLEMOUSE, CmdHandler::Mouse, 0, 0),
    NvCmd::new(K_MIDDLEDRAG, CmdHandler::Mouse, 0, 0),
    NvCmd::new(K_MIDDLERELEASE, CmdHandler::Mouse, 0, 0),
    NvCmd::new(K_RIGHTMOUSE, CmdHandler::Mouse, 0, 0),
    NvCmd::new(K_RIGHTDRAG, CmdHandler::Mouse, 0, 0),
    NvCmd::new(K_RIGHTRELEASE, CmdHandler::Mouse, 0, 0),
    NvCmd::new(K_X1MOUSE, CmdHandler::Mouse, 0, 0),
    NvCmd::new(K_X1DRAG, CmdHandler::Mouse, 0, 0),
    NvCmd::new(K_X1RELEASE, CmdHandler::Mouse, 0, 0),
    NvCmd::new(K_X2MOUSE, CmdHandler::Mouse, 0, 0),
    NvCmd::new(K_X2DRAG, CmdHandler::Mouse, 0, 0),
    NvCmd::new(K_X2RELEASE, CmdHandler::Mouse, 0, 0),
    // Other special keys
    NvCmd::new(K_IGNORE, CmdHandler::Ignore, f(NV_KEEPREG), 0),
    NvCmd::new(K_NOP, CmdHandler::Nop, 0, 0),
    NvCmd::new(K_INS, CmdHandler::Edit, 0, 0),
    NvCmd::new(K_KINS, CmdHandler::Edit, 0, 0),
    NvCmd::new(K_BS, CmdHandler::Ctrlh, 0, 0),
    NvCmd::new(K_UP, CmdHandler::Up, f(NV_SSS | NV_STS), 0),
    NvCmd::new(K_S_UP, CmdHandler::Page, f(NV_SS), BACKWARD as i16),
    NvCmd::new(K_DOWN, CmdHandler::Down, f(NV_SSS | NV_STS), 0),
    NvCmd::new(K_S_DOWN, CmdHandler::Page, f(NV_SS), FORWARD as i16),
    NvCmd::new(K_LEFT, CmdHandler::Left, f(NV_SSS | NV_STS | NV_RL), 0),
    NvCmd::new(K_S_LEFT, CmdHandler::BckWord, f(NV_SS | NV_RL), 0),
    NvCmd::new(K_C_LEFT, CmdHandler::BckWord, f(NV_SSS | NV_RL | NV_STS), 1),
    NvCmd::new(K_RIGHT, CmdHandler::Right, f(NV_SSS | NV_STS | NV_RL), 0),
    NvCmd::new(K_S_RIGHT, CmdHandler::Wordcmd, f(NV_SS | NV_RL), 0),
    NvCmd::new(
        K_C_RIGHT,
        CmdHandler::Wordcmd,
        f(NV_SSS | NV_RL | NV_STS),
        1,
    ),
    NvCmd::new(
        K_PAGEUP,
        CmdHandler::Page,
        f(NV_SSS | NV_STS),
        BACKWARD as i16,
    ),
    NvCmd::new(
        K_KPAGEUP,
        CmdHandler::Page,
        f(NV_SSS | NV_STS),
        BACKWARD as i16,
    ),
    NvCmd::new(
        K_PAGEDOWN,
        CmdHandler::Page,
        f(NV_SSS | NV_STS),
        FORWARD as i16,
    ),
    NvCmd::new(
        K_KPAGEDOWN,
        CmdHandler::Page,
        f(NV_SSS | NV_STS),
        FORWARD as i16,
    ),
    NvCmd::new(K_END, CmdHandler::End, f(NV_SSS | NV_STS), 0),
    NvCmd::new(K_KEND, CmdHandler::End, f(NV_SSS | NV_STS), 0),
    NvCmd::new(K_S_END, CmdHandler::End, f(NV_SS), 0),
    NvCmd::new(K_C_END, CmdHandler::End, f(NV_SSS | NV_STS), 1),
    NvCmd::new(K_HOME, CmdHandler::Home, f(NV_SSS | NV_STS), 0),
    NvCmd::new(K_KHOME, CmdHandler::Home, f(NV_SSS | NV_STS), 0),
    NvCmd::new(K_S_HOME, CmdHandler::Home, f(NV_SS), 0),
    NvCmd::new(K_C_HOME, CmdHandler::Goto, f(NV_SSS | NV_STS), 0),
    NvCmd::new(K_DEL, CmdHandler::Abbrev, 0, 0),
    NvCmd::new(K_KDEL, CmdHandler::Abbrev, 0, 0),
    NvCmd::new(K_UNDO, CmdHandler::Kundo, 0, 0),
    NvCmd::new(K_HELP, CmdHandler::Help, f(NV_NCW), 0),
    NvCmd::new(K_F1, CmdHandler::Help, f(NV_NCW), 0),
    NvCmd::new(K_XF1, CmdHandler::Help, f(NV_NCW), 0),
    NvCmd::new(K_SELECT, CmdHandler::Select, 0, 0),
    NvCmd::new(K_PASTE_START, CmdHandler::Paste, f(NV_KEEPREG), 0),
    NvCmd::new(K_EVENT, CmdHandler::Event, f(NV_KEEPREG), 0),
    NvCmd::new(K_COMMAND, CmdHandler::Colon, 0, 0),
    NvCmd::new(K_LUA, CmdHandler::Colon, 0, 0),
];

/// Number of commands in the table.
pub const NV_CMDS_SIZE: usize = NV_CMDS.len();

// =============================================================================
// Sorted Index
// =============================================================================

use std::sync::OnceLock;

/// Sorted index of commands in NV_CMDS.
/// Lazily initialized on first use.
static NV_CMD_IDX: OnceLock<[i16; NV_CMDS_SIZE]> = OnceLock::new();

/// The highest index for which nv_cmds[idx].cmd_char == nv_cmd_idx[nv_cmds[idx].cmd_char].
static NV_MAX_LINEAR: OnceLock<c_int> = OnceLock::new();

/// Initialize the sorted command index.
fn init_cmd_idx() -> [i16; NV_CMDS_SIZE] {
    let mut idx: [i16; NV_CMDS_SIZE] = [0; NV_CMDS_SIZE];

    // Fill with one-to-one relation
    for (i, item) in idx.iter_mut().enumerate() {
        *item = i as i16;
    }

    // Sort by absolute value of command character
    idx.sort_by(|&a, &b| {
        let ca = NV_CMDS[a as usize].cmd_char.abs();
        let cb = NV_CMDS[b as usize].cmd_char.abs();
        ca.cmp(&cb)
    });

    idx
}

/// Find the max linear index.
fn find_max_linear(idx: &[i16; NV_CMDS_SIZE]) -> c_int {
    for i in 0..NV_CMDS_SIZE as c_int {
        if i != NV_CMDS[idx[i as usize] as usize].cmd_char {
            return i - 1;
        }
    }
    NV_CMDS_SIZE as c_int - 1
}

/// Get the sorted command index (lazily initialized).
#[inline]
pub fn get_cmd_idx() -> &'static [i16; NV_CMDS_SIZE] {
    NV_CMD_IDX.get_or_init(init_cmd_idx)
}

/// Get the max linear index (lazily initialized).
#[inline]
pub fn get_max_linear() -> c_int {
    *NV_MAX_LINEAR.get_or_init(|| {
        let idx = get_cmd_idx();
        find_max_linear(idx)
    })
}

// =============================================================================
// Command Lookup Functions
// =============================================================================

/// Search for a command in the commands table.
///
/// Returns the index in NV_CMDS, or -1 for invalid command.
#[inline]
#[must_use]
pub fn find_command(cmdchar: c_int) -> c_int {
    // A multi-byte character is never a command.
    if cmdchar >= 0x100 {
        return -1;
    }

    let abs_char = cmdchar.abs();
    let nv_max_linear = get_max_linear();
    let idx = get_cmd_idx();

    // If the character is in the first part: The character is the index into nv_cmd_idx[].
    if abs_char <= nv_max_linear {
        return c_int::from(idx[abs_char as usize]);
    }

    // Perform a binary search.
    let mut bot = nv_max_linear + 1;
    let mut top = NV_CMDS_SIZE as c_int - 1;

    while bot <= top {
        let i = c_int::midpoint(bot, top);
        let c = NV_CMDS[idx[i as usize] as usize].cmd_char.abs();
        if abs_char == c {
            return c_int::from(idx[i as usize]);
        }
        if abs_char > c {
            bot = i + 1;
        } else {
            top = i - 1;
        }
    }

    -1
}

/// Get the command entry at the given index.
///
/// Returns None if index is out of bounds.
#[inline]
#[must_use]
pub fn get_cmd_entry(idx: c_int) -> Option<&'static NvCmd> {
    if idx >= 0 && (idx as usize) < NV_CMDS_SIZE {
        Some(&NV_CMDS[idx as usize])
    } else {
        None
    }
}

/// Get the command flags at the given index.
#[inline]
#[must_use]
pub fn get_cmd_flags(idx: c_int) -> c_int {
    get_cmd_entry(idx).map_or(0, |e| c_int::from(e.flags))
}

/// Get the command argument at the given index.
#[inline]
#[must_use]
pub fn get_cmd_arg(idx: c_int) -> c_int {
    get_cmd_entry(idx).map_or(0, |e| c_int::from(e.arg))
}

/// Get the command character at the given index.
#[inline]
#[must_use]
pub fn get_cmd_char(idx: c_int) -> c_int {
    get_cmd_entry(idx).map_or(0, |e| e.cmd_char)
}

/// Get the command handler at the given index.
#[inline]
#[must_use]
pub fn get_cmd_handler(idx: c_int) -> Option<CmdHandler> {
    get_cmd_entry(idx).map(|e| e.handler)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Initialize normal mode command table (triggers lazy OnceLock initialization).
///
/// This replaces the C `init_normal_cmds()` function. The Rust table uses
/// `OnceLock` for lazy initialization so explicit initialization is a no-op,
/// but calling this ensures the table is ready before first use.
#[export_name = "init_normal_cmds"]
pub extern "C" fn rs_init_normal_cmds() {
    // Force initialization of sorted index and max_linear
    let _ = get_cmd_idx();
    let _ = get_max_linear();
}

/// FFI: Find command by character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_table_find_command(cmdchar: c_int) -> c_int {
    find_command(cmdchar)
}

/// FFI: Get command flags.
#[unsafe(no_mangle)]
pub extern "C" fn rs_table_get_cmd_flags(idx: c_int) -> c_int {
    get_cmd_flags(idx)
}

/// FFI: Get command argument.
#[unsafe(no_mangle)]
pub extern "C" fn rs_table_get_cmd_arg(idx: c_int) -> c_int {
    get_cmd_arg(idx)
}

/// FFI: Get command character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_table_get_cmd_char(idx: c_int) -> c_int {
    get_cmd_char(idx)
}

/// FFI: Get the table size.
#[unsafe(no_mangle)]
pub extern "C" fn rs_table_get_size() -> c_int {
    NV_CMDS_SIZE as c_int
}

/// FFI: Get the max linear index.
#[unsafe(no_mangle)]
pub extern "C" fn rs_table_get_max_linear() -> c_int {
    get_max_linear()
}

/// FFI: Get the sorted index entry.
#[unsafe(no_mangle)]
pub extern "C" fn rs_table_get_cmd_idx(idx: c_int) -> i16 {
    let table_idx = get_cmd_idx();
    if idx >= 0 && (idx as usize) < NV_CMDS_SIZE {
        table_idx[idx as usize]
    } else {
        0
    }
}

/// FFI: Check if command needs additional character.
///
/// This implements the Rust-table-based version of `rs_need_additional_char`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_table_needs_additional_char(
    idx: c_int,
    cmdchar: c_int,
    pending_op: bool,
) -> bool {
    let Some(entry) = get_cmd_entry(idx) else {
        return false;
    };

    // Without NV_NCH we never need to check for an additional char
    if !entry.needs_second_char() {
        return false;
    }

    // NV_NCH_NOP is set and no operator is pending, get a second char
    if entry.needs_second_char_no_op() && !pending_op {
        return true;
    }

    // NV_NCH_ALW is set, always get a second char
    if entry.needs_second_char_always() {
        return true;
    }

    // 'q' without a pending operator, recording or executing a register,
    // needs to be followed by a second char
    // Note: We still need to check reg_recording/reg_executing from C
    // This simplified version just checks the flag pattern
    if cmdchar == c_int::from(b'q') && !pending_op {
        // The caller needs to also check reg_recording == 0 && reg_executing == 0
        return true;
    }

    // 'a' or 'i' after an operator is a text object
    // Also in visual mode
    if cmdchar == c_int::from(b'a') || cmdchar == c_int::from(b'i') {
        // The caller needs to also check VIsual_active
        if pending_op {
            return true;
        }
    }

    false
}

// =============================================================================
// Dispatch Execution
// =============================================================================

// All command handler function pointers (called from rs_execute_dispatch)
#[cfg(not(test))]
extern "C" {
    fn rs_nv_ignore(cap: *mut std::ffi::c_void);
    fn rs_nv_nop(cap: *mut std::ffi::c_void);
    fn rs_nv_error(cap: *mut std::ffi::c_void);
    fn rs_nv_help(cap: *mut std::ffi::c_void);
    fn rs_nv_suspend(cap: *mut std::ffi::c_void);
    fn rs_nv_page(cap: *mut std::ffi::c_void);
    fn rs_nv_halfpage(cap: *mut std::ffi::c_void);
    fn rs_nv_ctrlg(cap: *mut std::ffi::c_void);
    fn rs_nv_scroll_line(cap: *mut std::ffi::c_void);
    fn rs_nv_kundo(cap: *mut std::ffi::c_void);
    fn rs_nv_goto(cap: *mut std::ffi::c_void);
    fn rs_nv_beginline(cap: *mut std::ffi::c_void);
    fn rs_nv_dollar(cap: *mut std::ffi::c_void);
    fn rs_nv_end(cap: *mut std::ffi::c_void);
    fn rs_nv_home(cap: *mut std::ffi::c_void);
    fn rs_nv_pipe(cap: *mut std::ffi::c_void);
    fn rs_nv_wordcmd(cap: *mut std::ffi::c_void);
    fn rs_nv_bck_word(cap: *mut std::ffi::c_void);
    fn rs_nv_findpar(cap: *mut std::ffi::c_void);
    fn rs_nv_brace(cap: *mut std::ffi::c_void);
    fn rs_nv_csearch(cap: *mut std::ffi::c_void);
    fn rs_nv_mark(cap: *mut std::ffi::c_void);
    fn rs_nv_gomark(cap: *mut std::ffi::c_void);
    fn rs_nv_pcmark(cap: *mut std::ffi::c_void);
    fn rs_nv_regname(cap: *mut std::ffi::c_void);
    fn rs_nv_put(cap: *mut std::ffi::c_void);
    fn rs_nv_visual(cap: *mut std::ffi::c_void);
    fn rs_nv_window(cap: *mut std::ffi::c_void);
    fn rs_nv_clear(cap: *mut std::ffi::c_void);
    fn rs_nv_ctrlo(cap: *mut std::ffi::c_void);
    fn rs_nv_hat(cap: *mut std::ffi::c_void);
    fn rs_nv_Zet(cap: *mut std::ffi::c_void);
    fn rs_nv_esc(cap: *mut std::ffi::c_void);
    fn rs_nv_edit(cap: *mut std::ffi::c_void);
    fn rs_nv_search(cap: *mut std::ffi::c_void);
    fn rs_nv_next(cap: *mut std::ffi::c_void);
    fn rs_nv_ident(cap: *mut std::ffi::c_void);
    fn rs_nv_operator(cap: *mut std::ffi::c_void);
    fn rs_nv_optrans(cap: *mut std::ffi::c_void);
    fn rs_nv_tilde(cap: *mut std::ffi::c_void);
    fn rs_nv_subst(cap: *mut std::ffi::c_void);
    fn rs_nv_select(cap: *mut std::ffi::c_void);
    fn rs_nv_brackets(cap: *mut std::ffi::c_void);
    fn rs_nv_undo(cap: *mut std::ffi::c_void);
    fn rs_nv_Undo(cap: *mut std::ffi::c_void);
    fn rs_nv_dot(cap: *mut std::ffi::c_void);
    fn rs_nv_redo_or_register(cap: *mut std::ffi::c_void);
    fn rs_nv_replace(cap: *mut std::ffi::c_void);
    fn rs_nv_Replace(cap: *mut std::ffi::c_void);
    fn rs_nv_zet(cap: *mut std::ffi::c_void);
    fn rs_nv_scroll(cap: *mut std::ffi::c_void);
    fn rs_nv_right(cap: *mut std::ffi::c_void);
    fn rs_nv_left(cap: *mut std::ffi::c_void);
    fn rs_nv_up(cap: *mut std::ffi::c_void);
    fn rs_nv_down(cap: *mut std::ffi::c_void);
    fn rs_nv_g_cmd(cap: *mut std::ffi::c_void);
    fn rs_nv_at(cap: *mut std::ffi::c_void);
    fn rs_nv_join(cap: *mut std::ffi::c_void);
    fn rs_nv_open(cap: *mut std::ffi::c_void);
    fn rs_nv_abbrev(cap: *mut std::ffi::c_void);
    fn rs_nv_lineop(cap: *mut std::ffi::c_void);
    fn rs_nv_normal(cap: *mut std::ffi::c_void);
    fn rs_nv_percent(cap: *mut std::ffi::c_void);
    fn rs_nv_tagpop(cap: *mut std::ffi::c_void);
    fn rs_nv_regreplay(cap: *mut std::ffi::c_void);
    fn rs_nv_ctrlh(cap: *mut std::ffi::c_void);
    fn rs_nv_addsub(cap: *mut std::ffi::c_void);
    fn rs_nv_colon(cap: *mut std::ffi::c_void);
    fn rs_nv_record(cap: *mut std::ffi::c_void);
    fn rs_nv_paste(cap: *mut std::ffi::c_void);
    fn rs_nv_event(cap: *mut std::ffi::c_void);
    fn nv_mouse(cap: *mut std::ffi::c_void);
    fn nv_mousescroll(cap: *mut std::ffi::c_void);
}

/// Dispatch a command to the appropriate handler function.
///
/// Sets ca->arg from the table entry and calls the handler.
///
/// # Safety
/// `cap` must be a valid cmdarg_T pointer. `idx` must be a valid table index.
#[cfg(not(test))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_execute_dispatch(idx: c_int, cap: *mut std::ffi::c_void) {
    let Some(entry) = get_cmd_entry(idx) else {
        return;
    };
    // Set ca->arg from the table (mimics nvim_execute_nv_cmd behavior)
    (*cap.cast::<CmdargT>()).arg = c_int::from(entry.arg);
    match entry.handler {
        CmdHandler::Ignore => rs_nv_ignore(cap),
        CmdHandler::Nop => rs_nv_nop(cap),
        CmdHandler::AddsUb => rs_nv_addsub(cap),
        CmdHandler::Page => rs_nv_page(cap),
        CmdHandler::Esc => rs_nv_esc(cap),
        CmdHandler::HalfPage => rs_nv_halfpage(cap),
        CmdHandler::ScrollLine => rs_nv_scroll_line(cap),
        CmdHandler::Ctrlg => rs_nv_ctrlg(cap),
        CmdHandler::Ctrlh => rs_nv_ctrlh(cap),
        CmdHandler::Pcmark => rs_nv_pcmark(cap),
        CmdHandler::Down => rs_nv_down(cap),
        CmdHandler::Clear => rs_nv_clear(cap),
        CmdHandler::Up => rs_nv_up(cap),
        CmdHandler::Visual => rs_nv_visual(cap),
        CmdHandler::Window => rs_nv_window(cap),
        CmdHandler::Suspend => rs_nv_suspend(cap),
        CmdHandler::Normal => rs_nv_normal(cap),
        CmdHandler::Ident => rs_nv_ident(cap),
        CmdHandler::Hat => rs_nv_hat(cap),
        CmdHandler::Right => rs_nv_right(cap),
        CmdHandler::Left => rs_nv_left(cap),
        CmdHandler::Operator => rs_nv_operator(cap),
        CmdHandler::Regname => rs_nv_regname(cap),
        CmdHandler::Dollar => rs_nv_dollar(cap),
        CmdHandler::Percent => rs_nv_percent(cap),
        CmdHandler::Optrans => rs_nv_optrans(cap),
        CmdHandler::Gomark => rs_nv_gomark(cap),
        CmdHandler::Brace => rs_nv_brace(cap),
        CmdHandler::Csearch => rs_nv_csearch(cap),
        CmdHandler::Dot => rs_nv_dot(cap),
        CmdHandler::Search => rs_nv_search(cap),
        CmdHandler::Beginline => rs_nv_beginline(cap),
        CmdHandler::Colon => rs_nv_colon(cap),
        CmdHandler::Next => rs_nv_next(cap),
        CmdHandler::At => rs_nv_at(cap),
        CmdHandler::Edit => rs_nv_edit(cap),
        CmdHandler::BckWord => rs_nv_bck_word(cap),
        CmdHandler::Abbrev => rs_nv_abbrev(cap),
        CmdHandler::Wordcmd => rs_nv_wordcmd(cap),
        CmdHandler::Goto => rs_nv_goto(cap),
        CmdHandler::Scroll => rs_nv_scroll(cap),
        CmdHandler::Join => rs_nv_join(cap),
        CmdHandler::Open => rs_nv_open(cap),
        CmdHandler::Put => rs_nv_put(cap),
        CmdHandler::Regreplay => rs_nv_regreplay(cap),
        CmdHandler::Replace => rs_nv_replace(cap),
        CmdHandler::Subst => rs_nv_subst(cap),
        CmdHandler::Undo => rs_nv_undo(cap),
        CmdHandler::Zet => rs_nv_Zet(cap),
        CmdHandler::SmallZet => rs_nv_zet(cap),
        CmdHandler::Brackets => rs_nv_brackets(cap),
        CmdHandler::Lineop => rs_nv_lineop(cap),
        CmdHandler::Tilde => rs_nv_tilde(cap),
        CmdHandler::Findpar => rs_nv_findpar(cap),
        CmdHandler::Pipe => rs_nv_pipe(cap),
        CmdHandler::Mark => rs_nv_mark(cap),
        CmdHandler::Record => rs_nv_record(cap),
        CmdHandler::RedoOrRegister => rs_nv_redo_or_register(cap),
        CmdHandler::ReplaceMode => rs_nv_Replace(cap),
        CmdHandler::UndoLine => rs_nv_Undo(cap),
        CmdHandler::GCmd => rs_nv_g_cmd(cap),
        CmdHandler::Mouse => nv_mouse(cap),
        CmdHandler::Mousescroll => nv_mousescroll(cap),
        CmdHandler::End => rs_nv_end(cap),
        CmdHandler::Home => rs_nv_home(cap),
        CmdHandler::Kundo => rs_nv_kundo(cap),
        CmdHandler::Help => rs_nv_help(cap),
        CmdHandler::Select => rs_nv_select(cap),
        CmdHandler::Paste => rs_nv_paste(cap),
        CmdHandler::Event => rs_nv_event(cap),
        CmdHandler::Ctrlo => rs_nv_ctrlo(cap),
        CmdHandler::Tagpop => rs_nv_tagpop(cap),
        CmdHandler::Error | CmdHandler::Object => rs_nv_error(cap),
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_size() {
        assert_eq!(NV_CMDS_SIZE, 188);
    }

    #[test]
    fn test_cmd_idx_initialized() {
        let idx = get_cmd_idx();
        assert_eq!(idx.len(), NV_CMDS_SIZE);
    }

    #[test]
    fn test_max_linear_positive() {
        let max = get_max_linear();
        assert!(max >= 0);
        assert!(max < NV_CMDS_SIZE as c_int);
    }

    #[test]
    fn test_find_command_basic() {
        // Test some basic commands
        let idx_a = find_command(c_int::from(b'a'));
        assert!(idx_a >= 0);
        assert_eq!(NV_CMDS[idx_a as usize].cmd_char, c_int::from(b'a'));

        let idx_h = find_command(c_int::from(b'h'));
        assert!(idx_h >= 0);
        assert_eq!(NV_CMDS[idx_h as usize].cmd_char, c_int::from(b'h'));
    }

    #[test]
    fn test_find_command_invalid() {
        // Multi-byte characters should return -1
        assert_eq!(find_command(0x100), -1);
        assert_eq!(find_command(0x200), -1);
    }

    #[test]
    fn test_cmd_entry_flags() {
        // Test 'g' command has NV_NCH_ALW flag
        let idx_g = find_command(c_int::from(b'g'));
        let entry = get_cmd_entry(idx_g).unwrap();
        assert!(entry.needs_second_char_always());
        assert!(entry.needs_second_char());
    }

    #[test]
    fn test_cmd_entry_handler() {
        let idx_a = find_command(c_int::from(b'a'));
        let handler = get_cmd_handler(idx_a);
        assert_eq!(handler, Some(CmdHandler::Edit));
    }

    #[test]
    fn test_special_keys() {
        // Test that K_UP is findable
        let idx_up = find_command(K_UP);
        assert!(idx_up >= 0);
        assert_eq!(NV_CMDS[idx_up as usize].cmd_char, K_UP);
    }

    #[test]
    fn test_direction_args() {
        // Test '(' has BACKWARD arg
        let idx_paren = find_command(c_int::from(b'('));
        let entry = get_cmd_entry(idx_paren).unwrap();
        assert_eq!(entry.arg, BACKWARD as i16);

        // Test ')' has FORWARD arg
        let idx_paren = find_command(c_int::from(b')'));
        let entry = get_cmd_entry(idx_paren).unwrap();
        assert_eq!(entry.arg, FORWARD as i16);
    }

    #[test]
    fn test_keepreg_flag() {
        // Test '.' has NV_KEEPREG flag
        let idx_dot = find_command(c_int::from(b'.'));
        let entry = get_cmd_entry(idx_dot).unwrap();
        assert!(entry.keeps_regname());
    }
}
