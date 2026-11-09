//! Setting a string option: the checks, the completion, and the callback
//! the option table names for each one.
//!
//! The module is one file per kind of work; the parent holds only what
//! more than one of them needs — the message texts, the two character
//! structs the screen draws from, and the constants the transpiled
//! headers left behind.
//!
//! | child | what |
//! | --- | --- |
//! | [`frame`] | what a callback is handed |
//! | [`check`] | vetting a value, and the sweeps that re-vet every buffer |
//! | [`flags`] | flag-letter sets and fixed word lists |
//! | [`expand`] | command-line completion of a value |
//! | [`chars`] | 'fillchars'/'listchars' field lists |
//! | [`display`] | the callbacks for what the screen looks like |
//! | [`buffer`] | the callbacks for how a buffer's text is handled |
//! | [`complete`] | the callbacks for completion, spelling and tags |
//! | [`statusline`] | the callbacks for format strings and session specs |

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use core::ffi::{CStr, c_int, c_uint};

mod check;
mod read;
pub(crate) use self::read::*;
pub(crate) mod frame;
pub use self::check::*;
mod flags;
pub use self::flags::*;
mod expand;
pub use self::expand::*;
mod chars;
pub use self::chars::*;
mod display;
pub use self::display::*;
mod buffer;
pub use self::buffer::*;
mod complete;
pub use self::complete::*;
mod statusline;
pub use self::statusline::*;
use crate::regexp::vim_regexec;
use crate::terminal::terminal_notify_theme;
pub const CPT_MENU: c_uint = 2;
pub const CPT_KIND: c_uint = 1;
pub const CPT_ABBR: c_uint = 0;
pub const LSIZE: c_uint = 512;
/// 'imsearch' at -1 follows 'iminsert' rather than answering for itself.
pub const B_IMODE_USE_INSERT: c_int = -1;
pub const B_IMODE_NONE: c_int = 0;
pub const B_IMODE_LMAP: c_int = 1;
/// The only value nvim accepts for 'highlight': Vim's highlight-group
/// mapping is not implemented, so the option can only be left alone.
pub const EOL_MAC: c_int = 2;
/// The letters 'formatoptions' accepts.
pub const FO_ALL: &CStr = c"tcro/q2vlb1mMBn,aw]jp";
/// The letters 'cpoptions' accepts.
pub const CPO_VI: &CStr = c"aAbBcCdDeEfFiIJKlLmMnoOpPqrRsStuvWxXyZ$!%+>;~_";
/// The letters 'whichwrap' accepts.
pub const WW_ALL: &CStr = c"bshl<>[]~";
/// The letters 'mouse' accepts.
pub const MOUSE_ALL: &CStr = c"anvichr";
pub const MOUSESCROLL_VERT_DFLT: c_int = 3;
pub const MOUSESCROLL_HOR_DFLT: c_int = 6;
/// The letters 'concealcursor' accepts.
pub const COCU_ALL: &CStr = c"nvic";
/// The flag letters a 'comments' part may carry before its own separator.
pub const COM_ALL: &CStr = c"nbsmexflrO";
pub const SCL_NO: c_int = -1;
pub const SID_NONE: c_int = -6;
/// "E535: Illegal character after <%c>", for the options that spell a
/// field as a character followed by a value.
pub(crate) const E_ILLEGAL_CHARACTER_AFTER_CHR: &CStr = c"E535: Illegal character after <%c>";
/// "E536: Comma required", for 'foldmarker'.
pub(crate) const E_COMMA_REQUIRED: &CStr = c"E536: Comma required";
/// "E540: Unclosed expression sequence", for a `%{` with no `}`.
pub(crate) const E_UNCLOSED_EXPRESSION_SEQUENCE: &CStr = c"E540: Unclosed expression sequence";
/// "E542: Unbalanced groups", for a `%(` with no `%)`.
pub(crate) const E_UNBALANCED_GROUPS: &CStr = c"E542: Unbalanced groups";
/// "E589: 'backupext' and 'patchmode' are equal".
pub(crate) const E_BACKUPEXT_AND_PATCHMODE_ARE_EQUAL: &CStr =
    c"E589: 'backupext' and 'patchmode' are equal";
/// "E595: 'showbreak' contains unprintable or wide character".
pub(crate) const E_SHOWBREAK_CONTAINS_UNPRINTABLE_OR_WIDE_CHARACTER: &CStr =
    c"E595: 'showbreak' contains unprintable or wide character";
/// "E1511: Wrong number of characters for field", for a 'fillchars' or
/// 'listchars' field given too few or too many characters.
pub(crate) const E_WRONG_NUMBER_OF_CHARACTERS_FOR_FIELD_STR: &CStr =
    c"E1511: Wrong number of characters for field \"%s\"";
/// "E1512: Wrong character width for field", for one that does not fit
/// in a single screen cell.
pub(crate) const E_WRONG_CHARACTER_WIDTH_FOR_FIELD_STR: &CStr =
    c"E1512: Wrong character width for field \"%s\"";
/// The letters 'shortmess' accepts. The trailing "nfxi" were removed
/// as flags and are now silently ignored.
pub const SHM_ALL: &CStr = c"rmlwaWtToOsAIcCqFSnfxi";
/// "E834: Conflicts with value of 'listchars'", reported when something
/// other than `:set` made the current value unrenderable.
pub(crate) const E_CONFLICTS_WITH_VALUE_OF_LISTCHARS: &CStr =
    c"E834: Conflicts with value of 'listchars'";
/// "E835: Conflicts with value of 'fillchars'", as above.
pub(crate) const E_CONFLICTS_WITH_VALUE_OF_FILLCHARS: &CStr =
    c"E835: Conflicts with value of 'fillchars'";
pub const INT_MAX: c_int = 2147483647;

pub type CharsOption = ::core::ffi::c_uint;

/// The set of characters 'breakat' allows a line to be broken before, one
/// bit per byte value.
///
/// A bitset rather than the 256-entry `char[]` upstream keeps, because
/// `vim_isbreak` asks this once per character of every drawn line and wants
/// [`GlobalCell::get`](crate::global_cell::GlobalCell::get) — the cheap
/// accessor — rather than `with`, whose debug-build borrow tracking costs
/// far more than the lookup, or a 256-byte copy per character.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct BreakAt([u64; 4]);

impl BreakAt {
    /// No character may be broken before.
    pub(crate) const NONE: Self = Self([0; 4]);

    /// Whether a line may be broken before `byte`.
    pub(crate) fn has(self, byte: u8) -> bool {
        self.0[usize::from(byte) / 64] >> (byte % 64) & 1 != 0
    }

    /// Allow a break before `byte`.
    pub(crate) fn insert(&mut self, byte: u8) {
        self.0[usize::from(byte) / 64] |= 1 << (byte % 64);
    }
}
