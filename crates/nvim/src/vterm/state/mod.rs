//! The terminal state machine: the object the escape-sequence parser drives.
//!
//! It remembers everything a terminal remembers apart from the screen itself —
//! where the cursor is, which modes are set, what the scroll region and the
//! tab stops are, which character sets are mapped — and hands every visible
//! effect to the callback table its consumer installed with
//! [`vterm_state_set_callbacks`], which in practice is
//! [`crate::vterm::screen`].
//!
//! This file owns the boundary. Callback tables, the out-of-line line-mark and
//! tab-stop arrays and the allocator all arrive as raw pointers, so they are
//! wrapped here exactly once, as safe methods on [`VTermState`]. The sequence
//! handling itself lives in sibling modules — [`text`] for printable text and
//! the C0/C1 controls, [`csi`] for control sequences, [`mode`] for modes and
//! properties, [`selection`] for OSC, [`dcs`] for the other control strings,
//! and `geometry` for the shape of the screen — all of which are written
//! against those methods and do no pointer work of their own. The state
//! machine's own lifetime and the `extern "C"` entry points, which cannot be
//! written that way, live in [`entry`](self::entry).
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod entry;

use core::ffi::{c_char, c_int, c_long, c_uint, c_void};
use core::slice;

use self::entry::vterm_state_set_termprop;
use crate::global_cell::GlobalCell;
use crate::grid::schar_from_buf;
use crate::mbyte::{utf_char2bytes, utf_iscomposing, utf_ptr2cells_len};
use crate::types::{
    GraphemeState, VTerm, VTermGlyphInfo, VTermLineInfo, VTermPos, VTermProp, VTermRect,
    VTermSelectionCallbacks, VTermSelectionMask, VTermState, VTermState_tmp_selection,
    VTermStateCallbacks, VTermStateFallbacks, VTermStateFields, VTermStringFragment, VTermValue,
    schar_T, uint8_t,
};
use crate::vterm::encoding::{ENC_SINGLE_94, ENC_UTF8, vterm_lookup_encoding};
use crate::vterm::output::EscapeSeq;
use crate::vterm::pen::{apply_sgr, reset_pen, restore_pen};
use crate::vterm::screen::{BUFIDX_ALTSCREEN, BUFIDX_PRIMARY};
use crate::vterm::selection;
use crate::vterm::vterm::{vterm_alloc, vterm_dealloc, vterm_push_output_bytes, vterm_scroll_rect};

/// What the host asked to be told about the pointer, as
/// `VTERM_PROP_MOUSE` reduces to.
pub const MOUSE_WANT_CLICK: c_int = 0x1;
pub const MOUSE_WANT_DRAG: c_int = 0x2;
pub const MOUSE_WANT_MOVE: c_int = 0x4;

/// Primary Device Attributes (DA1) response, `61;22;52` and its terminator.
/// Exported so that the TUI tests can substitute another terminal's answer
/// through FFI, which is why it is spelled as the `c_char` array the wire
/// wants rather than as a byte string.
#[unsafe(no_mangle)]
pub static vterm_primary_device_attr: GlobalCell<[c_char; 9]> = GlobalCell::new([
    b'6' as c_char,
    b'1' as c_char,
    b';' as c_char,
    b'2' as c_char,
    b'2' as c_char,
    b';' as c_char,
    b'5' as c_char,
    b'2' as c_char,
    0,
]);

/// A change to the pen that the consumer is told about.
pub(super) enum PenChange<'a> {
    /// Apply an SGR control sequence.
    Sgr(&'a [c_long]),
    /// Put back the pen saved under DEC mode 1048.
    Restore,
    /// Return the pen to its defaults.
    Reset,
}

/// The bytes a control string fragment carries.
pub(super) fn fragment_bytes(frag: &VTermStringFragment) -> &[u8] {
    if frag.str.is_null() {
        return &[];
    }
    // SAFETY: the parser promises a fragment's `str` points at `len` bytes.
    unsafe { slice::from_raw_parts(frag.str.cast::<u8>(), frag.len()) }
}

/// A line with no double-width, double-height or continuation marking.
pub(super) const BLANK_LINE: VTermLineInfo = VTermLineInfo {
    doublewidth_doubleheight_continuation: [0; 1],
    _pad: [0; 3],
};

/// Whether `next` extends the grapheme that `prev` ended.
pub(super) fn is_composing(prev: u32, next: u32, grapheme: &mut GraphemeState) -> bool {
    // SAFETY: two codepoints and the caller's own grapheme scratch.
    unsafe { utf_iscomposing(prev as c_int, next as c_int, grapheme) }
}

/// How many bytes the tab-stop bit vector needs for `cols` columns.
fn tabstop_bytes(cols: c_int) -> usize {
    (cols.max(0) as usize).div_ceil(8)
}

// --------------------------------------------------------- the raw boundary

impl VTermState {
    /// The terminal this state belongs to.
    ///
    /// A `VTerm` and its `VTermState` are separate allocations, made and
    /// freed together, so a borrow of one never aliases the other.
    fn terminal(&self) -> &VTerm {
        // SAFETY: `vt` is the terminal this state was made for.
        unsafe { &*self.vt }
    }

    /// Whether the host asked for single-byte C1 controls on the wire.
    pub(super) fn ctrl8bit(&self) -> bool {
        self.terminal().mode.ctrl8bit() != 0
    }

    /// Selects between the 7-bit and 8-bit forms of the C1 controls.
    pub(super) fn set_ctrl8bit(&mut self, eight_bit: bool) {
        // SAFETY: as `terminal` above.
        unsafe { (*self.vt).mode.set_ctrl8bit(eight_bit as c_uint) };
    }

    /// Whether the terminal is decoding its input as UTF-8.
    pub(super) fn utf8(&self) -> bool {
        self.terminal().mode.utf8() != 0
    }

    /// Which of the two line-mark arrays the live screen is using.
    pub(super) fn active_buffer(&self) -> usize {
        if self.mode.alt_screen() != 0 {
            BUFIDX_ALTSCREEN
        } else {
            BUFIDX_PRIMARY
        }
    }

    /// Per-row double-width and double-height marks, one entry per screen row.
    pub(super) fn lineinfo(&self) -> &[VTermLineInfo] {
        // SAFETY: `lineinfo` points at one of the two mark arrays, each of
        // which is allocated and resized to hold exactly `rows` entries.
        unsafe { slice::from_raw_parts(self.lineinfo, self.rows.max(0) as usize) }
    }

    pub(super) fn lineinfo_mut(&mut self) -> &mut [VTermLineInfo] {
        // SAFETY: as `lineinfo` above.
        unsafe { slice::from_raw_parts_mut(self.lineinfo, self.rows.max(0) as usize) }
    }

    /// The tab-stop bit vector, one bit per column.
    pub(super) fn tabstops_mut(&mut self) -> &mut [u8] {
        // SAFETY: `tabstops` is allocated and resized to `tabstop_bytes(cols)`
        // bytes, which is what is asked for here.
        unsafe { slice::from_raw_parts_mut(self.tabstops, tabstop_bytes(self.cols)) }
    }

    /// One slot of the table of screen effects the consumer installed.
    ///
    /// The slot is read *out* of the table rather than handed back behind a
    /// borrow: every one of these is a callback into the consumer, which may
    /// re-enter this state machine, so nothing may still be borrowed when it
    /// is called.
    pub(super) fn consumer<T>(
        &self,
        pick: impl FnOnce(&VTermStateCallbacks) -> Option<T>,
    ) -> Option<T> {
        // SAFETY: the consumer promised its table outlives its installation.
        unsafe { self.callbacks.as_ref() }.and_then(pick)
    }

    /// One slot of the table the consumer installed for sequences this module
    /// does not recognise. Read out for the reason `consumer` gives.
    pub(super) fn fallback<T>(
        &self,
        pick: impl FnOnce(&VTermStateFallbacks) -> Option<T>,
    ) -> Option<T> {
        // SAFETY: as `consumer` above.
        unsafe { self.fallbacks.as_ref() }.and_then(pick)
    }

    /// Writes a reply back to the host. A sequence that outgrew its builder is
    /// dropped whole rather than truncated onto the wire.
    pub(super) fn reply(&mut self, seq: &EscapeSeq) {
        if let Some(bytes) = seq.finish() {
            let (buf, len) = (bytes.as_ptr().cast::<c_char>(), bytes.len());
            // SAFETY: `vt` is this state's terminal and `buf`/`len` is the
            // finished sequence, which outlives the call.
            unsafe { vterm_push_output_bytes(self.vt, buf, len) };
        }
    }

    // -------------------------------------------------------- screen effects

    /// Stamps one grapheme onto the screen at `pos`.
    pub(super) fn put_glyph(&mut self, schar: schar_T, width: c_int, pos: VTermPos) {
        let line = self.lineinfo()[pos.row as usize];
        let mut info = VTermGlyphInfo {
            schar,
            width,
            protected_cell_dwl_dhl: [0; 1],
            _pad: [0; 3],
        };
        info.set_protected_cell(self.protected_cell());
        info.set_dwl(line.doublewidth());
        info.set_dhl(line.doubleheight());
        let cbdata = self.cbdata;
        if let Some(f) = self.consumer(|c| c.putglyph) {
            unsafe { f(&raw mut info, pos, cbdata) };
        }
    }

    /// Reports a cursor move away from `oldpos`. A move that did not happen is
    /// not reported, and does not cancel a pending wrap.
    pub(super) fn update_cursor(&mut self, oldpos: VTermPos, cancel_phantom: bool) {
        if self.pos.col == oldpos.col && self.pos.row == oldpos.row {
            return;
        }
        if cancel_phantom {
            self.at_phantom = 0;
        }
        self.force_cursor_report(oldpos);
    }

    /// Reports the cursor's position whether or not it moved, which is what a
    /// full reset needs so that the consumer redraws it.
    pub(super) fn force_cursor_report(&mut self, oldpos: VTermPos) {
        let (pos, visible, cbdata) = (self.pos, self.mode.cursor_visible() as c_int, self.cbdata);
        if let Some(f) = self.consumer(|c| c.movecursor) {
            unsafe { f(pos, oldpos, visible, cbdata) };
        }
    }

    /// Blanks a rectangle. `selective` spares cells marked protected.
    pub(super) fn erase(&mut self, rect: VTermRect, selective: bool) {
        if rect.end_col == self.cols {
            // Erasing the final cells of a line cancels the continuation mark
            // on the line that followed it.
            let last = (rect.end_row + 1).min(self.rows);
            for row in (rect.start_row + 1).max(0)..last {
                self.lineinfo_mut()[row as usize].set_continuation(0);
            }
        }
        let cbdata = self.cbdata;
        if let Some(f) = self.consumer(|c| c.erase) {
            unsafe { f(rect, selective as c_int, cbdata) };
        }
    }

    /// Scrolls a rectangle by `downward` rows and `rightward` columns, either
    /// of which may be negative. A shift at least as large as the rectangle
    /// erases it instead.
    pub(super) fn scroll(&mut self, rect: VTermRect, downward: c_int, rightward: c_int) {
        if downward == 0 && rightward == 0 {
            return;
        }
        let rows = rect.end_row - rect.start_row;
        let downward = downward.max(-rows).min(rows);
        let cols = rect.end_col - rect.start_col;
        let rightward = rightward.max(-cols).min(cols);

        // A full-width vertical scroll moves the line marks with the lines.
        if rect.start_col == 0 && rect.end_col == self.cols && rightward == 0 {
            let start = rect.start_row as usize;
            let height = (rows - downward.abs()) as usize;
            let lines = self.lineinfo_mut();
            if downward > 0 {
                let shift = downward as usize;
                lines.copy_within(start + shift..start + shift + height, start);
                lines[(rect.end_row - downward) as usize..rect.end_row as usize].fill(BLANK_LINE);
            } else {
                let shift = (-downward) as usize;
                lines.copy_within(start..start + height, start + shift);
                lines[start..start + shift].fill(BLANK_LINE);
            }
        }

        let cbdata = self.cbdata;
        if let Some(f) = self.consumer(|c| c.scrollrect)
            && unsafe { f(rect, downward, rightward, cbdata) } != 0
        {
            return;
        }
        // The consumer did not take the scroll whole, so drive it out of the
        // move and erase primitives instead.
        if let Some((moverect, erase)) = self.consumer(|c| Some((c.moverect, c.erase))) {
            // SAFETY: both slots are the consumer's own, and `cbdata` is what
            // it installed beside them; nothing here is borrowed.
            unsafe { vterm_scroll_rect(rect, downward, rightward, moverect, erase, cbdata) };
        }
    }

    /// Marks row `row` double-width and/or double-height. The consumer gets a
    /// veto unless `force` is set, which is how a reset clears the marks even
    /// on a screen that would rather keep them.
    pub(super) fn set_lineinfo(
        &mut self,
        row: c_int,
        force: bool,
        doublewidth: bool,
        doubleheight: c_uint,
    ) {
        let mut info = self.lineinfo()[row as usize];
        info.set_doublewidth(doublewidth as c_uint);
        info.set_doubleheight(doubleheight);
        let cbdata = self.cbdata;
        let mut accepted = false;
        if let Some(f) = self.consumer(|c| c.setlineinfo) {
            let current = self.lineinfo.cast_const().wrapping_add(row as usize);
            accepted = unsafe { f(row, &raw const info, current, cbdata) } != 0;
        }
        if accepted || force {
            self.lineinfo_mut()[row as usize] = info;
        }
    }

    /// Rings the terminal bell.
    pub(super) fn bell(&mut self) {
        let cbdata = self.cbdata;
        if let Some(f) = self.consumer(|c| c.bell) {
            unsafe { f(cbdata) };
        }
    }

    /// Drops the scrollback, reporting whether the consumer took the request.
    pub(super) fn clear_scrollback(&mut self) -> bool {
        let cbdata = self.cbdata;
        match self.consumer(|c| c.sb_clear) {
            Some(f) => unsafe { f(cbdata) != 0 },
            None => false,
        }
    }

    /// Asks the consumer whether it is showing a dark colour scheme; `None`
    /// when it does not know.
    pub(super) fn theme_is_dark(&mut self) -> Option<bool> {
        let cbdata = self.cbdata;
        let f = self.consumer(|c| c.theme)?;
        let mut dark = false;
        (unsafe { f(&raw mut dark, cbdata) } != 0).then_some(dark)
    }

    /// Tells the consumer to return its own pen to the defaults.
    pub(super) fn init_consumer_pen(&mut self) {
        let cbdata = self.cbdata;
        if let Some(f) = self.consumer(|c| c.initpen) {
            unsafe { f(cbdata) };
        }
    }

    /// Resizes the consumer's screen, letting it move the cursor and swap in
    /// its own line-mark arrays.
    fn resize_consumer(&mut self, rows: c_int, cols: c_int) {
        let mut fields = VTermStateFields {
            pos: self.pos,
            lineinfos: self.lineinfos,
        };
        let cbdata = self.cbdata;
        let Some(f) = self.consumer(|c| c.resize) else {
            return;
        };
        unsafe { f(rows, cols, &raw mut fields, cbdata) };
        self.pos = fields.pos;
        self.lineinfos = fields.lineinfos;
    }

    /// Reallocates the tab-stop vector for a new column count, keeping the
    /// stops that survive and giving fresh columns the default stop every
    /// eighth column.
    fn resize_tabstops(&mut self, cols: c_int) {
        let bytes = tabstop_bytes(cols);
        let mut stops = Vec::with_capacity(bytes);
        for byte in 0..bytes {
            let mut bits = 0u8;
            for bit in 0..8 {
                let col = (byte as c_int) << 3 | bit;
                // Columns the old screen had keep their stop; fresh ones get
                // the default one every eighth column.
                let stop = if col < self.cols {
                    self.tabstops_mut()[byte] & 1 << bit != 0
                } else {
                    col % 8 == 0
                };
                bits |= u8::from(stop && col < cols) << bit;
            }
            stops.push(bits);
        }
        // SAFETY: a fresh allocation of `bytes` bytes, filled from a vector
        // of exactly that length, replacing the array it succeeds.
        let fresh = unsafe { vterm_alloc(bytes) }.cast::<uint8_t>();
        unsafe { fresh.copy_from_nonoverlapping(stops.as_ptr(), bytes) };
        unsafe { vterm_dealloc(self.tabstops.cast::<c_void>()) };
        self.tabstops = fresh;
    }

    // -------------------------------------------------------------- the pen

    /// Makes one of the changes to the pen that are echoed to the consumer's
    /// raw `setpenattr` callback.
    pub(super) fn change_pen(&mut self, change: PenChange<'_>) {
        // SAFETY: each takes this state machine, whose consumer table is
        // what the pen is echoed through.
        match change {
            PenChange::Sgr(args) => unsafe { apply_sgr(self, args) },
            PenChange::Restore => unsafe { restore_pen(self) },
            PenChange::Reset => unsafe { reset_pen(self) },
        }
    }

    // -------------------------------------------------- the grapheme buffer

    /// Appends `codepoint` to the pending grapheme at offset `at`, returning
    /// how many bytes it took.
    ///
    /// Upstream guarded the buffer against a four-byte encoding but let a
    /// six-byte one write past its end; anything that does not fit is dropped
    /// here instead.
    pub(super) fn append_grapheme(&mut self, at: usize, codepoint: u32) -> usize {
        let mut encoded = [0 as c_char; 8];
        // SAFETY: `encoded` is eight bytes, more than any encoding needs.
        let len = unsafe { utf_char2bytes(codepoint as c_int, encoded.as_mut_ptr()) } as usize;
        let len = len.min(self.grapheme_buf.len() - at);
        self.grapheme_buf[at..at + len].copy_from_slice(&encoded[..len]);
        len
    }

    /// How many columns the first `len` bytes of the pending grapheme occupy,
    /// and the handle the screen stores that grapheme under.
    pub(super) fn grapheme_metrics(&self, len: usize) -> (c_int, schar_T) {
        let buf = self.grapheme_buf.as_ptr();
        // SAFETY: `len` bytes of the pending grapheme, which is this array.
        let cells = unsafe { utf_ptr2cells_len(buf, len as c_int) };
        (cells, unsafe { schar_from_buf(buf, len) })
    }

    // --------------------------------------------------------- character sets

    /// Designates slot `slot` (G0 to G3) to the 94-character set named by
    /// `designator`. An unknown name leaves the slot alone.
    pub(super) fn designate_charset(&mut self, slot: usize, designator: c_char) {
        let enc = vterm_lookup_encoding(ENC_SINGLE_94, designator);
        if !enc.is_null() {
            self.encoding[slot].enc = enc;
            self.init_encoding(slot);
        }
    }

    /// Points every slot at the terminal's default set: UTF-8 when the
    /// terminal decodes UTF-8, ASCII otherwise.
    pub(super) fn reset_charsets(&mut self) {
        let default = if self.utf8() {
            vterm_lookup_encoding(ENC_UTF8, b'u' as c_char)
        } else {
            vterm_lookup_encoding(ENC_SINGLE_94, b'B' as c_char)
        };
        for slot in 0..self.encoding.len() {
            self.encoding[slot].enc = default;
            self.init_encoding(slot);
        }
    }

    /// Hands slot `slot` its per-instance decoder scratch to initialise.
    fn init_encoding(&mut self, slot: usize) {
        let enc = self.encoding[slot].enc;
        let data = (&raw mut self.encoding[slot].data).cast::<c_void>();
        // SAFETY: `enc` is one of the encoding module's static tables, and
        // `data` is this slot's own inline scratch.
        if let Some(init) = unsafe { (*enc).init } {
            unsafe { init(enc, data) };
        }
    }

    // ------------------------------------------------------- terminal properties

    fn set_termprop_value(&mut self, prop: VTermProp, mut val: VTermValue) -> bool {
        // SAFETY: this state machine and a value of the arm `prop` calls for.
        unsafe { vterm_state_set_termprop(self, prop, &raw mut val) != 0 }
    }

    pub(super) fn set_termprop_bool(&mut self, prop: VTermProp, value: bool) -> bool {
        let boolean = c_int::from(value);
        self.set_termprop_value(prop, VTermValue { boolean })
    }

    pub(super) fn set_termprop_int(&mut self, prop: VTermProp, value: c_int) -> bool {
        self.set_termprop_value(prop, VTermValue { number: value })
    }

    pub(super) fn set_termprop_string(&mut self, prop: VTermProp, s: VTermStringFragment) -> bool {
        self.set_termprop_value(prop, VTermValue { string: s })
    }

    /// The bytes of a DECRQSS request gathered so far, NUL-terminated.
    pub(super) fn decrqss(&self) -> [c_char; 4] {
        // SAFETY: the union is four bytes of request or a selection's
        // progress, both plain data, so either arm reads what was written to
        // whichever of them the state is part-way through.
        unsafe { self.tmp.decrqss }
    }

    pub(super) fn set_decrqss(&mut self, request: [c_char; 4]) {
        self.tmp.decrqss = request;
    }

    // ------------------------------------------------------------- selection

    /// Whether the consumer wants to hear about OSC 52 selection traffic.
    pub(super) fn selection_enabled(&self) -> bool {
        !self.selection.callbacks.is_null()
    }

    /// Whether the consumer will take a selection at all.
    pub(super) fn selection_accepts_set(&self) -> bool {
        self.selection_slot(|c| c.set).is_some()
    }

    /// How far the OSC 52 decoder has got. It shares its storage with the
    /// DECRQSS request, so only one of the two can be part-way through.
    pub(super) fn selection_progress(&self) -> VTermState_tmp_selection {
        // SAFETY: as `decrqss` above.
        unsafe { self.tmp.selection }
    }

    pub(super) fn set_selection_progress(&mut self, progress: VTermState_tmp_selection) {
        self.tmp.selection = progress;
    }

    /// One slot of the consumer's selection table, read out for the reason
    /// [`VTermState::consumer`] gives.
    fn selection_slot<T>(
        &self,
        pick: impl FnOnce(&VTermSelectionCallbacks) -> Option<T>,
    ) -> Option<T> {
        // SAFETY: the consumer promised its table outlives its installation.
        unsafe { self.selection.callbacks.as_ref() }.and_then(pick)
    }

    /// The staging buffer selection data is decoded into, one chunk at a time.
    pub(super) fn selection_buffer_mut(&mut self) -> &mut [u8] {
        if self.selection.buffer.is_null() {
            return &mut [];
        }
        let (buf, len) = (self.selection.buffer.cast::<u8>(), self.selection.buflen);
        // SAFETY: the buffer and its length are what the consumer installed,
        // or what `vterm_state_set_selection_callbacks` allocated for it.
        unsafe { slice::from_raw_parts_mut(buf, len) }
    }

    /// Hands on the first `len` bytes of the staging buffer, or ends the
    /// selection with no content at all, which clears it.
    pub(super) fn selection_set(
        &mut self,
        mask: VTermSelectionMask,
        len: Option<usize>,
        initial: bool,
        last: bool,
    ) {
        let frag = match len {
            Some(len) => selection::fragment(self.selection.buffer, len, initial, last),
            None => selection::fragment(core::ptr::null(), 0, initial, true),
        };
        let user = self.selection.user;
        if let Some(f) = self.selection_slot(|c| c.set) {
            unsafe { f(mask, frag, user) };
        }
    }

    /// Asks the consumer to report the current selection back to the host.
    pub(super) fn selection_query(&mut self, mask: VTermSelectionMask) {
        let user = self.selection.user;
        if let Some(f) = self.selection_slot(|c| c.query) {
            unsafe { f(mask, user) };
        }
    }

    // ------------------------------------------------------------- fallbacks

    /// `CSI I` / `CSI O`, the focus reports.
    fn report_focus(&mut self, command: u8) {
        if self.mode.report_focus() == 0 {
            return;
        }
        let mut seq = EscapeSeq::csi(self.ctrl8bit());
        seq.push(command);
        self.reply(&seq);
    }
}

/// A bare state machine for the sequence handlers' own tests: no terminal
/// behind it and no consumer callbacks, with the caller supplying the two
/// arrays that live outside the struct. Only handlers that stay clear of the
/// terminal and the callbacks can be driven with one.
#[cfg(test)]
pub(super) fn test_state(
    lineinfo: &mut [VTermLineInfo],
    tabstops: &mut [u8],
    cols: c_int,
) -> VTermState {
    let mut state: VTermState = unsafe { ::core::mem::zeroed() };
    state.rows = lineinfo.len() as c_int;
    state.cols = cols;
    // Both arrays are borrowed exactly once: taking a second pointer out of
    // the same reference would invalidate the first.
    let marks = lineinfo.as_mut_ptr();
    state.lineinfo = marks;
    state.lineinfos = [marks; 2];
    state.tabstops = tabstops.as_mut_ptr();
    state.scrollregion_bottom = -1;
    state.scrollregion_right = -1;
    state
}
