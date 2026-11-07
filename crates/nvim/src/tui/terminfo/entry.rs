#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! A terminal's description as the TUI reads it out of terminfo.

use std::borrow::Cow;

/// One string capability: absent, or the bytes of its sequence.
///
/// The built-in descriptions and the sequences the TUI patches in are this
/// binary's static data and are borrowed; what the system's terminfo
/// database answered is copied in and owned, so the entry lives exactly as
/// long as the strings it holds.
pub type Capability = Option<Cow<'static, [u8]>>;

/// A terminal's description, as the TUI reads it: three flags, three
/// numbers, and the string capabilities by slot.
pub struct TerminfoEntry {
    pub bce: bool,
    pub has_tc_or_rgb: bool,
    pub su: bool,
    pub max_colors: ::core::ffi::c_int,
    pub lines: ::core::ffi::c_int,
    pub columns: ::core::ffi::c_int,
    /// By `tui::terminfo::caps` slot: the standard string capabilities, then
    /// the extensions.
    pub defs: [Capability; 49],
    /// The special keys, by `caps::KEYS` slot: the sequence and its shifted
    /// variant.
    pub keys: [[Capability; 2]; 16],
    /// `key_f1` upward.
    pub f_keys: [Capability; 63],
}

impl Default for TerminfoEntry {
    /// The empty description: nothing known, every number zero.
    fn default() -> Self {
        Self {
            bce: false,
            has_tc_or_rgb: false,
            su: false,
            max_colors: 0,
            lines: 0,
            columns: 0,
            defs: [const { None }; 49],
            keys: [const { [None, None] }; 16],
            f_keys: [const { None }; 63],
        }
    }
}
