//! A C consumer's callback tables, as the emulator's report traits.
//!
//! Each `Foreign*` value is a table and the data pointer installed beside it
//! by one of [`super`]'s exports; every method calls one slot of the table,
//! or answers `None` for an empty one.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `vterm/abi/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr::NonNull;

use crate::types::{
    VTermAttr, VTermGlyphInfo, VTermLineInfo, VTermOutputCallback, VTermParserCallbacks, VTermPos,
    VTermProp, VTermRect, VTermScreenCallbacks, VTermScreenCell, VTermSelectionCallbacks,
    VTermSelectionMask, VTermStateCallbacks, VTermStateFallbacks, VTermStateFields,
    VTermStringFragment, VTermValue,
};
use crate::vterm::host::{
    Csi, Fallbacks, Fragment, ParserHandler, ScreenHost, SelectionHost, Value,
};
use crate::vterm::state::StateCx;

// --------------------------------------------------------- the conversions

impl Fragment<'_> {
    /// The fragment as a C consumer is handed it.
    fn to_c(self) -> VTermStringFragment {
        let mut frag = VTermStringFragment {
            str: self
                .buffer()
                .map_or(core::ptr::null(), |bytes| bytes.as_ptr().cast()),
            len_initial_final_0: [0; 4],
            terminator: self.terminator,
        };
        frag.set_len(self.bytes().len());
        frag.set_initial(self.initial);
        frag.set_final_0(self.last);
        frag
    }
}

/// A value as a C consumer is handed it.
fn value_to_c(val: &Value<'_>) -> VTermValue {
    match *val {
        Value::Bool(flag) => VTermValue {
            boolean: c_int::from(flag),
        },
        Value::Int(number) => VTermValue { number },
        Value::Str(frag) => VTermValue {
            string: frag.to_c(),
        },
        Value::Color(color) => VTermValue { color },
    }
}

/// A count as C's `int`. Every count handed over here is a row of cells or a
/// control sequence's parameters, far below `int`'s range.
fn c_len(len: usize) -> c_int {
    c_int::try_from(len).expect("a count C is handed fits an int")
}

/// A leader or intermediate run as C sees it: NUL-terminated, or null.
fn cstr_or_null(bytes: Option<&CStr>) -> *const c_char {
    bytes.map_or(core::ptr::null(), CStr::as_ptr)
}

// ------------------------------------------------- a C consumer's tables

/// A C consumer's parser callbacks.
#[derive(Copy, Clone)]
pub(crate) struct ForeignParser {
    pub(super) table: NonNull<VTermParserCallbacks>,
    pub(super) data: *mut c_void,
}

impl ForeignParser {
    fn table(&self) -> &VTermParserCallbacks {
        // SAFETY: installed by `vterm_parser_set_callbacks`, whose caller
        // keeps the table live while it is installed.
        unsafe { self.table.as_ref() }
    }
}

impl ParserHandler for ForeignParser {
    fn text(&mut self, bytes: &[u8]) -> Option<c_int> {
        let text = self.table().text?;
        // SAFETY: the consumer's own callback (see the module docs), handed a
        // run of bytes that outlives the call.
        Some(unsafe { text(bytes.as_ptr().cast(), bytes.len(), self.data) })
    }

    fn control(&mut self, control: u8) -> Option<c_int> {
        let f = self.table().control?;
        // SAFETY: as for `text`.
        Some(unsafe { f(control, self.data) })
    }

    fn escape(&mut self, sequence: &[u8]) -> Option<c_int> {
        let f = self.table().escape?;
        // SAFETY: as for `text`.
        Some(unsafe { f(sequence.as_ptr().cast(), sequence.len(), self.data) })
    }

    fn csi(&mut self, csi: &Csi<'_>) -> Option<c_int> {
        let f = self.table().csi?;
        let (leader, intermed) = (cstr_or_null(csi.leader), cstr_or_null(csi.intermed));
        let argcount = c_len(csi.args.len());
        let command = csi.command.cast_signed();
        // SAFETY: as for `text`; the strings are NUL-terminated and, with
        // the parameters, outlive the call.
        Some(unsafe {
            f(
                leader,
                csi.args.as_ptr(),
                argcount,
                intermed,
                command,
                self.data,
            )
        })
    }

    fn osc(&mut self, command: c_int, frag: Fragment<'_>) -> Option<c_int> {
        let f = self.table().osc?;
        // SAFETY: as for `text`.
        Some(unsafe { f(command, frag.to_c(), self.data) })
    }

    fn dcs(&mut self, command: &[u8], frag: Fragment<'_>) -> Option<c_int> {
        let f = self.table().dcs?;
        let name = command.as_ptr().cast();
        // SAFETY: as for `text`.
        Some(unsafe { f(name, command.len(), frag.to_c(), self.data) })
    }

    fn apc(&mut self, frag: Fragment<'_>) -> Option<c_int> {
        let f = self.table().apc?;
        // SAFETY: as for `text`.
        Some(unsafe { f(frag.to_c(), self.data) })
    }

    fn pm(&mut self, frag: Fragment<'_>) -> Option<c_int> {
        let f = self.table().pm?;
        // SAFETY: as for `text`.
        Some(unsafe { f(frag.to_c(), self.data) })
    }

    fn sos(&mut self, frag: Fragment<'_>) -> Option<c_int> {
        let f = self.table().sos?;
        // SAFETY: as for `text`.
        Some(unsafe { f(frag.to_c(), self.data) })
    }

    fn resize(&mut self, rows: c_int, cols: c_int) -> Option<c_int> {
        let f = self.table().resize?;
        // SAFETY: as for `text`.
        Some(unsafe { f(rows, cols, self.data) })
    }
}

/// A C consumer's state callbacks: what the state machine paints through
/// when the consumer is not the terminal's own screen.
#[derive(Copy, Clone)]
pub(crate) struct ForeignState {
    pub(super) table: NonNull<VTermStateCallbacks>,
    pub(super) data: *mut c_void,
}

impl ForeignState {
    fn table(&self) -> &VTermStateCallbacks {
        // SAFETY: installed by `vterm_state_set_callbacks`, whose caller
        // keeps the table live while it is installed.
        unsafe { self.table.as_ref() }
    }

    pub(crate) fn has_erase(&self) -> bool {
        self.table().erase.is_some()
    }

    pub(crate) fn putglyph(self, info: &mut VTermGlyphInfo, pos: VTermPos) -> Option<c_int> {
        let f = self.table().putglyph?;
        // SAFETY: the consumer's own callback (see the module docs), handed a
        // glyph that outlives the call.
        Some(unsafe { f(info, pos, self.data) })
    }

    pub(crate) fn movecursor(self, pos: VTermPos, old: VTermPos, visible: c_int) -> Option<c_int> {
        let f = self.table().movecursor?;
        // SAFETY: as for `putglyph`.
        Some(unsafe { f(pos, old, visible, self.data) })
    }

    pub(crate) fn scrollrect(self, rect: VTermRect, down: c_int, right: c_int) -> Option<c_int> {
        let f = self.table().scrollrect?;
        // SAFETY: as for `putglyph`.
        Some(unsafe { f(rect, down, right, self.data) })
    }

    pub(crate) fn moverect(self, dest: VTermRect, src: VTermRect) -> Option<c_int> {
        let f = self.table().moverect?;
        // SAFETY: as for `putglyph`.
        Some(unsafe { f(dest, src, self.data) })
    }

    pub(crate) fn erase(self, rect: VTermRect, selective: c_int) -> Option<c_int> {
        let f = self.table().erase?;
        // SAFETY: as for `putglyph`.
        Some(unsafe { f(rect, selective, self.data) })
    }

    pub(crate) fn initpen(self) -> Option<c_int> {
        let f = self.table().initpen?;
        // SAFETY: as for `putglyph`.
        Some(unsafe { f(self.data) })
    }

    pub(crate) fn setpenattr(self, attr: VTermAttr, val: &Value<'_>) -> Option<c_int> {
        let f = self.table().setpenattr?;
        let mut val = value_to_c(val);
        // SAFETY: as for `putglyph`.
        Some(unsafe { f(attr, &raw mut val, self.data) })
    }

    pub(crate) fn settermprop(self, prop: VTermProp, val: &Value<'_>) -> Option<c_int> {
        let f = self.table().settermprop?;
        let mut val = value_to_c(val);
        // SAFETY: as for `putglyph`.
        Some(unsafe { f(prop, &raw mut val, self.data) })
    }

    pub(crate) fn bell(self) -> Option<c_int> {
        let f = self.table().bell?;
        // SAFETY: as for `putglyph`.
        Some(unsafe { f(self.data) })
    }

    pub(crate) fn resize(
        self,
        rows: c_int,
        cols: c_int,
        fields: &mut VTermStateFields,
    ) -> Option<c_int> {
        let f = self.table().resize?;
        // SAFETY: as for `putglyph`; the line-mark arrays in `fields` hold
        // `rows` marks each.
        Some(unsafe { f(rows, cols, fields, self.data) })
    }

    pub(crate) fn theme(self, dark: &mut bool) -> Option<c_int> {
        let f = self.table().theme?;
        // SAFETY: as for `putglyph`.
        Some(unsafe { f(dark, self.data) })
    }

    pub(crate) fn setlineinfo(
        self,
        row: c_int,
        new: &VTermLineInfo,
        old: &VTermLineInfo,
    ) -> Option<c_int> {
        let f = self.table().setlineinfo?;
        // SAFETY: as for `putglyph`.
        Some(unsafe { f(row, new, old, self.data) })
    }

    pub(crate) fn sb_clear(self) -> Option<c_int> {
        let f = self.table().sb_clear?;
        // SAFETY: as for `putglyph`.
        Some(unsafe { f(self.data) })
    }
}

/// A C consumer's fallbacks for the sequences the state does not recognise.
pub(super) struct ForeignFallbacks {
    pub(super) table: NonNull<VTermStateFallbacks>,
    pub(super) data: *mut c_void,
}

impl ForeignFallbacks {
    fn table(&self) -> &VTermStateFallbacks {
        // SAFETY: installed by `vterm_state_set_unrecognised_fallbacks`,
        // whose caller keeps the table live while it is installed.
        unsafe { self.table.as_ref() }
    }
}

impl Fallbacks for ForeignFallbacks {
    fn control(&mut self, control: u8) -> Option<c_int> {
        let f = self.table().control?;
        // SAFETY: the consumer's own callback (see the module docs).
        Some(unsafe { f(control, self.data) })
    }

    fn csi(&mut self, csi: &Csi<'_>) -> Option<c_int> {
        let f = self.table().csi?;
        let (leader, intermed) = (cstr_or_null(csi.leader), cstr_or_null(csi.intermed));
        let argcount = c_len(csi.args.len());
        let command = csi.command.cast_signed();
        // SAFETY: as for `control`; the strings are NUL-terminated and, with
        // the parameters, outlive the call.
        Some(unsafe {
            f(
                leader,
                csi.args.as_ptr(),
                argcount,
                intermed,
                command,
                self.data,
            )
        })
    }

    fn osc(&mut self, command: c_int, frag: Fragment<'_>, _: &mut StateCx<'_>) -> Option<c_int> {
        let f = self.table().osc?;
        // SAFETY: as for `control`.
        Some(unsafe { f(command, frag.to_c(), self.data) })
    }

    fn dcs(&mut self, command: &[u8], frag: Fragment<'_>) -> Option<c_int> {
        let f = self.table().dcs?;
        let name = command.as_ptr().cast();
        // SAFETY: as for `control`.
        Some(unsafe { f(name, command.len(), frag.to_c(), self.data) })
    }

    fn apc(&mut self, frag: Fragment<'_>) -> Option<c_int> {
        let f = self.table().apc?;
        // SAFETY: as for `control`.
        Some(unsafe { f(frag.to_c(), self.data) })
    }

    fn pm(&mut self, frag: Fragment<'_>) -> Option<c_int> {
        let f = self.table().pm?;
        // SAFETY: as for `control`.
        Some(unsafe { f(frag.to_c(), self.data) })
    }

    fn sos(&mut self, frag: Fragment<'_>) -> Option<c_int> {
        let f = self.table().sos?;
        // SAFETY: as for `control`.
        Some(unsafe { f(frag.to_c(), self.data) })
    }
}

/// A C consumer's OSC 52 callbacks.
pub(super) struct ForeignSelection {
    pub(super) table: NonNull<VTermSelectionCallbacks>,
    pub(super) data: *mut c_void,
}

impl ForeignSelection {
    fn table(&self) -> &VTermSelectionCallbacks {
        // SAFETY: installed by `vterm_state_set_selection_callbacks`, whose
        // caller keeps the table live while it is installed.
        unsafe { self.table.as_ref() }
    }
}

impl SelectionHost for ForeignSelection {
    fn takes_set(&self) -> bool {
        self.table().set.is_some()
    }

    fn set(&mut self, mask: VTermSelectionMask, frag: Fragment<'_>) -> Option<c_int> {
        let f = self.table().set?;
        // SAFETY: the consumer's own callback (see the module docs).
        Some(unsafe { f(mask, frag.to_c(), self.data) })
    }

    fn query(&mut self, mask: VTermSelectionMask) -> Option<c_int> {
        let f = self.table().query?;
        // SAFETY: as for `set`.
        Some(unsafe { f(mask, self.data) })
    }
}

/// A C host's screen callbacks.
pub(super) struct ForeignScreen {
    pub(super) table: NonNull<VTermScreenCallbacks>,
    pub(super) data: *mut c_void,
}

impl ForeignScreen {
    fn table(&self) -> &VTermScreenCallbacks {
        // SAFETY: installed by `vterm_screen_set_callbacks`, whose caller
        // keeps the table live while it is installed.
        unsafe { self.table.as_ref() }
    }
}

impl ScreenHost for ForeignScreen {
    fn takes_moves(&self) -> bool {
        self.table().moverect.is_some()
    }

    fn takes_scrollback(&self) -> bool {
        self.table().sb_pushline.is_some()
    }

    fn damage(&mut self, rect: VTermRect) -> Option<c_int> {
        let f = self.table().damage?;
        // SAFETY: the host's own callback (see the module docs).
        Some(unsafe { f(rect, self.data) })
    }

    fn moverect(&mut self, dest: VTermRect, src: VTermRect) -> Option<c_int> {
        let f = self.table().moverect?;
        // SAFETY: as for `damage`.
        Some(unsafe { f(dest, src, self.data) })
    }

    fn movecursor(&mut self, pos: VTermPos, oldpos: VTermPos, visible: c_int) -> Option<c_int> {
        let f = self.table().movecursor?;
        // SAFETY: as for `damage`.
        Some(unsafe { f(pos, oldpos, visible, self.data) })
    }

    fn settermprop(&mut self, prop: VTermProp, val: &Value<'_>) -> Option<c_int> {
        let f = self.table().settermprop?;
        let mut val = value_to_c(val);
        // SAFETY: as for `damage`.
        Some(unsafe { f(prop, &raw mut val, self.data) })
    }

    fn bell(&mut self) -> Option<c_int> {
        let f = self.table().bell?;
        // SAFETY: as for `damage`.
        Some(unsafe { f(self.data) })
    }

    fn resize(&mut self, rows: c_int, cols: c_int) -> Option<c_int> {
        let f = self.table().resize?;
        // SAFETY: as for `damage`.
        Some(unsafe { f(rows, cols, self.data) })
    }

    fn theme(&mut self, dark: &mut bool) -> Option<c_int> {
        let f = self.table().theme?;
        // SAFETY: as for `damage`.
        Some(unsafe { f(dark, self.data) })
    }

    fn sb_pushline(&mut self, cells: &[VTermScreenCell]) -> Option<c_int> {
        let f = self.table().sb_pushline?;
        // SAFETY: as for `damage`; the row outlives the call.
        Some(unsafe { f(c_len(cells.len()), cells.as_ptr(), self.data) })
    }

    fn sb_popline(&mut self, cells: &mut [VTermScreenCell]) -> Option<c_int> {
        let f = self.table().sb_popline?;
        // SAFETY: as for `damage`; the host fills the row it is handed.
        Some(unsafe { f(c_len(cells.len()), cells.as_mut_ptr(), self.data) })
    }

    fn sb_clear(&mut self) -> Option<c_int> {
        let f = self.table().sb_clear?;
        // SAFETY: as for `damage`.
        Some(unsafe { f(self.data) })
    }
}

/// A C consumer's reply sink.
pub(crate) struct ForeignOutput {
    pub(super) func: VTermOutputCallback,
    pub(super) data: *mut c_void,
}

impl ForeignOutput {
    pub(crate) fn write(&mut self, bytes: &[u8]) {
        // SAFETY: installed by `vterm_output_set_callback`, whose caller keeps
        // the sink and its data live while it is installed; the reply
        // outlives the call.
        unsafe { (self.func)(bytes.as_ptr().cast(), bytes.len(), self.data) };
    }
}
