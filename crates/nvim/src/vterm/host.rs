//! What the emulator tells whoever drives it, in Rust's terms.
//!
//! libvterm reports through tables of C function pointers, each handed a
//! `void *` of the consumer's. Those tables are still the interface a C
//! consumer installs (see [`super::abi`]); inside the emulator every report
//! goes through the traits here instead, and a C table is just one more
//! implementation of them. `:terminal` implements them directly.
//!
//! Each trait mirrors one of libvterm's tables. A table slot can be empty,
//! and the emulator behaves differently when it is (a scroll the consumer
//! does not take whole is driven out of moves and erases; a missing
//! scrollback callback means rows are not pushed at all), so every method
//! answers `None` for "no such slot" and `Some` with what the callback
//! returned otherwise.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int, c_long};

use crate::vterm::types::{
    VTermColor, VTermPos, VTermProp, VTermRect, VTermScreenCell, VTermSelectionMask,
    VTermTerminator,
};

use super::state::StateCx;

/// A piece of a control string's payload, as the parser hands it on.
///
/// `initial` marks the first piece of a string and `last` the one its
/// terminator ended. A fragment can carry no bytes at all; one the selection
/// decoder sends to clear a selection carries no *buffer* either, which a C
/// consumer sees as a null pointer rather than an empty run.
#[derive(Copy, Clone)]
pub(crate) struct Fragment<'a> {
    bytes: Option<&'a [u8]>,
    pub initial: bool,
    pub last: bool,
    pub terminator: VTermTerminator,
}

impl<'a> Fragment<'a> {
    pub(crate) fn new(
        bytes: &'a [u8],
        initial: bool,
        last: bool,
        terminator: VTermTerminator,
    ) -> Self {
        Fragment {
            bytes: Some(bytes),
            initial,
            last,
            terminator,
        }
    }

    /// A fragment with no buffer behind it.
    pub(crate) fn absent(initial: bool, last: bool) -> Self {
        Fragment {
            bytes: None,
            initial,
            last,
            terminator: 0,
        }
    }

    /// The payload, empty for a fragment with no buffer.
    pub(crate) fn bytes(&self) -> &'a [u8] {
        self.bytes.unwrap_or_default()
    }

    /// The payload, `None` for a fragment with no buffer.
    pub(crate) fn buffer(&self) -> Option<&'a [u8]> {
        self.bytes
    }
}

/// The value a terminal property or a pen attribute is set to.
///
/// libvterm passes a union and lets the property or attribute say which arm
/// is live; here the arm is the variant.
#[derive(Copy, Clone)]
pub(crate) enum Value<'a> {
    Bool(bool),
    Int(c_int),
    Str(Fragment<'a>),
    Color(VTermColor),
}

impl Value<'_> {
    /// The `int` libvterm's union would hold: a flag is 0 or 1. A string or
    /// colour has no `int` arm, and reads as 0.
    pub(crate) fn int(&self) -> c_int {
        match *self {
            Value::Bool(flag) => c_int::from(flag),
            Value::Int(number) => number,
            Value::Str(_) | Value::Color(_) => 0,
        }
    }

    /// The colour, or the default colour for a value that is not one.
    pub(crate) fn color(&self) -> VTermColor {
        match *self {
            Value::Color(color) => color,
            _ => crate::vterm::color::UNSET,
        }
    }
}

/// A complete control sequence, as the parser gathered it.
pub(crate) struct Csi<'a> {
    /// The private-marker bytes ahead of the parameters, if any.
    pub leader: Option<&'a CStr>,
    pub args: &'a [c_long],
    /// The intermediate bytes before the final one, if any.
    pub intermed: Option<&'a CStr>,
    pub command: u8,
}

/// The parser's consumer: libvterm's `VTermParserCallbacks`.
pub(crate) trait ParserHandler {
    /// Ordinary text; answers how many bytes it consumed.
    fn text(&mut self, bytes: &[u8]) -> Option<c_int>;
    fn control(&mut self, control: u8) -> Option<c_int>;
    /// An escape sequence: its intermediates then its final byte.
    fn escape(&mut self, sequence: &[u8]) -> Option<c_int>;
    fn csi(&mut self, csi: &Csi<'_>) -> Option<c_int>;
    fn osc(&mut self, command: c_int, frag: Fragment<'_>) -> Option<c_int>;
    fn dcs(&mut self, command: &[u8], frag: Fragment<'_>) -> Option<c_int>;
    fn apc(&mut self, frag: Fragment<'_>) -> Option<c_int>;
    fn pm(&mut self, frag: Fragment<'_>) -> Option<c_int>;
    fn sos(&mut self, frag: Fragment<'_>) -> Option<c_int>;
    fn resize(&mut self, rows: c_int, cols: c_int) -> Option<c_int>;
}

/// A parser with no consumer: every event is dropped.
pub(crate) struct NoParserHandler;

impl ParserHandler for NoParserHandler {
    fn text(&mut self, _: &[u8]) -> Option<c_int> {
        None
    }
    fn control(&mut self, _: u8) -> Option<c_int> {
        None
    }
    fn escape(&mut self, _: &[u8]) -> Option<c_int> {
        None
    }
    fn csi(&mut self, _: &Csi<'_>) -> Option<c_int> {
        None
    }
    fn osc(&mut self, _: c_int, _: Fragment<'_>) -> Option<c_int> {
        None
    }
    fn dcs(&mut self, _: &[u8], _: Fragment<'_>) -> Option<c_int> {
        None
    }
    fn apc(&mut self, _: Fragment<'_>) -> Option<c_int> {
        None
    }
    fn pm(&mut self, _: Fragment<'_>) -> Option<c_int> {
        None
    }
    fn sos(&mut self, _: Fragment<'_>) -> Option<c_int> {
        None
    }
    fn resize(&mut self, _: c_int, _: c_int) -> Option<c_int> {
        None
    }
}

/// The sequences the state machine does not recognise, offered to whoever
/// drives it: libvterm's `VTermStateFallbacks`.
///
/// An OSC is offered with the state machine itself, so that a consumer can
/// act on it at once -- `:terminal` applies a hyperlink to the pen before the
/// next glyph is printed.
pub(crate) trait Fallbacks {
    fn control(&mut self, _control: u8) -> Option<c_int> {
        None
    }
    fn csi(&mut self, _csi: &Csi<'_>) -> Option<c_int> {
        None
    }
    fn osc(
        &mut self,
        _command: c_int,
        _frag: Fragment<'_>,
        _state: &mut StateCx<'_>,
    ) -> Option<c_int> {
        None
    }
    fn dcs(&mut self, _command: &[u8], _frag: Fragment<'_>) -> Option<c_int> {
        None
    }
    fn apc(&mut self, _frag: Fragment<'_>) -> Option<c_int> {
        None
    }
    fn pm(&mut self, _frag: Fragment<'_>) -> Option<c_int> {
        None
    }
    fn sos(&mut self, _frag: Fragment<'_>) -> Option<c_int> {
        None
    }
}

/// OSC 52 traffic: libvterm's `VTermSelectionCallbacks`.
pub(crate) trait SelectionHost {
    /// Whether the consumer takes writes at all; the decoder does not run
    /// otherwise.
    fn takes_set(&self) -> bool;
    fn set(&mut self, mask: VTermSelectionMask, frag: Fragment<'_>) -> Option<c_int>;
    fn query(&mut self, mask: VTermSelectionMask) -> Option<c_int>;
}

/// What the screen reports to whoever drives it: libvterm's
/// `VTermScreenCallbacks`.
///
/// The screen asks [`ScreenHost::takes_moves`] and
/// [`ScreenHost::takes_scrollback`] before it does the work a move or a
/// pushed line needs, so those two answer for their slots ahead of time.
pub(crate) trait ScreenHost {
    fn takes_moves(&self) -> bool {
        false
    }
    fn takes_scrollback(&self) -> bool {
        false
    }
    fn damage(&mut self, _rect: VTermRect) -> Option<c_int> {
        None
    }
    fn moverect(&mut self, _dest: VTermRect, _src: VTermRect) -> Option<c_int> {
        None
    }
    fn movecursor(&mut self, _pos: VTermPos, _oldpos: VTermPos, _visible: c_int) -> Option<c_int> {
        None
    }
    fn settermprop(&mut self, _prop: VTermProp, _val: &Value<'_>) -> Option<c_int> {
        None
    }
    fn bell(&mut self) -> Option<c_int> {
        None
    }
    fn resize(&mut self, _rows: c_int, _cols: c_int) -> Option<c_int> {
        None
    }
    fn theme(&mut self, _dark: &mut bool) -> Option<c_int> {
        None
    }
    fn sb_pushline(&mut self, _cells: &[VTermScreenCell]) -> Option<c_int> {
        None
    }
    fn sb_popline(&mut self, _cells: &mut [VTermScreenCell]) -> Option<c_int> {
        None
    }
    fn sb_clear(&mut self) -> Option<c_int> {
        None
    }
}
