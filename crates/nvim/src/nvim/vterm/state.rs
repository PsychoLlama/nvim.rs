//! The terminal state machine: the object the escape-sequence parser drives.
//!
//! It remembers everything a terminal remembers apart from the screen itself —
//! where the cursor is, which modes are set, what the scroll region and the
//! tab stops are, which character sets are mapped — and hands every visible
//! effect to the callback table its consumer installed with
//! [`vterm_state_set_callbacks`], which in practice is
//! [`crate::src::nvim::vterm::screen`].
//!
//! This file owns the boundary. Callback tables, the out-of-line line-mark and
//! tab-stop arrays and the allocator all arrive as raw pointers, so they are
//! wrapped here exactly once, as safe methods on [`VTermState`]. The sequence
//! handling itself lives in sibling modules — [`text`] for printable text and
//! the C0/C1 controls, [`csi`] for control sequences, [`mode`] for modes and
//! properties, [`selection`] for OSC, [`dcs`] for the other control strings,
//! and `geometry` for the shape of the screen — all of which are written
//! against those methods and do no pointer work of their own.

use core::ffi::{c_char, c_int, c_long, c_uint, c_void};
use core::slice;

use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::schar_from_buf;
use crate::src::nvim::mbyte::{utf_char2bytes, utf_iscomposing, utf_ptr2cells_len};
use crate::src::nvim::types::{
    GraphemeState, VTerm, VTermGlyphInfo, VTermKeyEncodingFlags, VTermLineInfo,
    VTermParserCallbacks, VTermPos, VTermProp, VTermRect, VTermSelectionCallbacks,
    VTermSelectionMask, VTermState, VTermState_tmp_selection, VTermStateCallbacks,
    VTermStateFallbacks, VTermStateFields, VTermStringFragment, VTermValue, schar_T, size_t,
    uint8_t,
};
use crate::src::nvim::vterm::encoding::{ENC_SINGLE_94, ENC_UTF8, vterm_lookup_encoding};
use crate::src::nvim::vterm::output::EscapeSeq;
use crate::src::nvim::vterm::parser::vterm_parser_set_callbacks;
use crate::src::nvim::vterm::pen::{apply_sgr, init_pen, reset_pen, restore_pen};
use crate::src::nvim::vterm::screen::{BUFIDX_ALTSCREEN, BUFIDX_PRIMARY};
use crate::src::nvim::vterm::vterm::{
    MOUSE_X10, VTERM_PROP_ALTSCREEN, VTERM_PROP_CURSORBLINK, VTERM_PROP_CURSORSHAPE,
    VTERM_PROP_CURSORVISIBLE, VTERM_PROP_FOCUSREPORT, VTERM_PROP_ICONNAME, VTERM_PROP_MOUSE,
    VTERM_PROP_REVERSE, VTERM_PROP_SYNCOUTPUT, VTERM_PROP_THEMEUPDATES, VTERM_PROP_TITLE,
    vterm_alloc, vterm_dealloc, vterm_push_output_bytes, vterm_scroll_rect,
};
use crate::src::nvim::vterm::{csi, dcs, mode, selection, text};

/// What the host asked to be told about the pointer, as
/// `VTERM_PROP_MOUSE` reduces to.
pub const MOUSE_WANT_CLICK: c_int = 0x1;
pub const MOUSE_WANT_DRAG: c_int = 0x2;
pub const MOUSE_WANT_MOVE: c_int = 0x4;

/// Primary Device Attributes (DA1) response. Exported so that the TUI tests
/// can substitute another terminal's answer through FFI.
#[unsafe(no_mangle)]
pub static vterm_primary_device_attr: GlobalCell<[c_char; 9]> =
    GlobalCell::new(unsafe { ::core::mem::transmute::<[u8; 9], [c_char; 9]>(*b"61;22;52\0") });

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
    unsafe { slice::from_raw_parts(frag.str.cast::<u8>(), frag.len()) }
}

/// A line with no double-width, double-height or continuation marking.
pub(super) const BLANK_LINE: VTermLineInfo = VTermLineInfo {
    doublewidth_doubleheight_continuation: [0; 1],
    c2rust_padding: [0; 3],
};

/// Whether `next` extends the grapheme that `prev` ended.
pub(super) fn is_composing(prev: u32, next: u32, grapheme: &mut GraphemeState) -> bool {
    unsafe { utf_iscomposing(prev as c_int, next as c_int, grapheme) }
}

/// How many bytes the tab-stop bit vector needs for `cols` columns.
fn tabstop_bytes(cols: c_int) -> usize {
    (cols.max(0) as usize).div_ceil(8)
}

// --------------------------------------------------------- the raw boundary

impl VTermState {
    /// The terminal this state belongs to.
    fn terminal(&self) -> &VTerm {
        unsafe { &*self.vt }
    }

    /// Whether the host asked for single-byte C1 controls on the wire.
    pub(super) fn ctrl8bit(&self) -> bool {
        self.terminal().mode.ctrl8bit() != 0
    }

    /// Selects between the 7-bit and 8-bit forms of the C1 controls.
    pub(super) fn set_ctrl8bit(&mut self, eight_bit: bool) {
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
        unsafe { slice::from_raw_parts(self.lineinfo, self.rows.max(0) as usize) }
    }

    pub(super) fn lineinfo_mut(&mut self) -> &mut [VTermLineInfo] {
        unsafe { slice::from_raw_parts_mut(self.lineinfo, self.rows.max(0) as usize) }
    }

    /// The tab-stop bit vector, one bit per column.
    pub(super) fn tabstops_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.tabstops, tabstop_bytes(self.cols)) }
    }

    /// The table of screen effects the consumer installed, if any.
    fn callback_table(&self) -> Option<&VTermStateCallbacks> {
        unsafe { self.callbacks.as_ref() }
    }

    /// The table the consumer installed for sequences this module does not
    /// recognise, if any.
    fn fallback_table(&self) -> Option<&VTermStateFallbacks> {
        unsafe { self.fallbacks.as_ref() }
    }

    /// Writes a reply back to the host. A sequence that outgrew its builder is
    /// dropped whole rather than truncated onto the wire.
    pub(super) fn reply(&mut self, seq: &EscapeSeq) {
        if let Some(bytes) = seq.finish() {
            unsafe {
                vterm_push_output_bytes(self.vt, bytes.as_ptr().cast::<c_char>(), bytes.len())
            };
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
            c2rust_padding: [0; 3],
        };
        info.set_protected_cell(self.protected_cell());
        info.set_dwl(line.doublewidth());
        info.set_dhl(line.doubleheight());
        let cbdata = self.cbdata;
        if let Some(f) = self.callback_table().and_then(|c| c.putglyph) {
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
        if let Some(f) = self.callback_table().and_then(|c| c.movecursor) {
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
        if let Some(f) = self.callback_table().and_then(|c| c.erase) {
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
                for row in (rect.end_row - downward) as usize..rect.end_row as usize {
                    lines[row] = BLANK_LINE;
                }
            } else {
                let shift = (-downward) as usize;
                lines.copy_within(start..start + height, start + shift);
                for row in start..start + shift {
                    lines[row] = BLANK_LINE;
                }
            }
        }

        let cbdata = self.cbdata;
        if let Some(f) = self.callback_table().and_then(|c| c.scrollrect)
            && unsafe { f(rect, downward, rightward, cbdata) } != 0
        {
            return;
        }
        // The consumer did not take the scroll whole, so drive it out of the
        // move and erase primitives instead.
        if let Some(table) = self.callback_table() {
            let (moverect, erase) = (table.moverect, table.erase);
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
        if let Some(f) = self.callback_table().and_then(|c| c.setlineinfo) {
            let current = &raw const self.lineinfo()[row as usize];
            accepted = unsafe { f(row, &raw const info, current, cbdata) } != 0;
        }
        if accepted || force {
            self.lineinfo_mut()[row as usize] = info;
        }
    }

    /// Rings the terminal bell.
    pub(super) fn bell(&mut self) {
        let cbdata = self.cbdata;
        if let Some(f) = self.callback_table().and_then(|c| c.bell) {
            unsafe { f(cbdata) };
        }
    }

    /// Drops the scrollback, reporting whether the consumer took the request.
    pub(super) fn clear_scrollback(&mut self) -> bool {
        let cbdata = self.cbdata;
        match self.callback_table().and_then(|c| c.sb_clear) {
            Some(f) => unsafe { f(cbdata) != 0 },
            None => false,
        }
    }

    /// Asks the consumer whether it is showing a dark colour scheme; `None`
    /// when it does not know.
    pub(super) fn theme_is_dark(&mut self) -> Option<bool> {
        let cbdata = self.cbdata;
        let f = self.callback_table().and_then(|c| c.theme)?;
        let mut dark = false;
        (unsafe { f(&raw mut dark, cbdata) } != 0).then_some(dark)
    }

    /// Tells the consumer to return its own pen to the defaults.
    pub(super) fn init_consumer_pen(&mut self) {
        let cbdata = self.cbdata;
        if let Some(f) = self.callback_table().and_then(|c| c.initpen) {
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
        let Some(f) = self.callback_table().and_then(|c| c.resize) else {
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
        unsafe {
            let fresh = vterm_alloc(bytes).cast::<uint8_t>();
            fresh.copy_from_nonoverlapping(stops.as_ptr(), bytes);
            vterm_dealloc(self.tabstops.cast::<c_void>());
            self.tabstops = fresh;
        }
    }

    // -------------------------------------------------------------- the pen

    /// Makes one of the changes to the pen that are echoed to the consumer's
    /// raw `setpenattr` callback.
    pub(super) fn change_pen(&mut self, change: PenChange<'_>) {
        unsafe {
            match change {
                PenChange::Sgr(args) => apply_sgr(self, args),
                PenChange::Restore => restore_pen(self),
                PenChange::Reset => reset_pen(self),
            }
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
        let len = unsafe { utf_char2bytes(codepoint as c_int, encoded.as_mut_ptr()) } as usize;
        let len = len.min(self.grapheme_buf.len() - at);
        self.grapheme_buf[at..at + len].copy_from_slice(&encoded[..len]);
        len
    }

    /// How many columns the first `len` bytes of the pending grapheme occupy,
    /// and the handle the screen stores that grapheme under.
    pub(super) fn grapheme_metrics(&self, len: usize) -> (c_int, schar_T) {
        let buf = self.grapheme_buf.as_ptr();
        unsafe {
            (
                utf_ptr2cells_len(buf, len as c_int),
                schar_from_buf(buf, len),
            )
        }
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
        unsafe {
            if let Some(init) = (*enc).init {
                init(enc, data);
            }
        }
    }

    // ------------------------------------------------------- terminal properties

    fn set_termprop_value(&mut self, prop: VTermProp, mut val: VTermValue) -> bool {
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
        self.selection_table().is_some_and(|c| c.set.is_some())
    }

    /// How far the OSC 52 decoder has got. It shares its storage with the
    /// DECRQSS request, so only one of the two can be part-way through.
    pub(super) fn selection_progress(&self) -> VTermState_tmp_selection {
        unsafe { self.tmp.selection }
    }

    pub(super) fn set_selection_progress(&mut self, progress: VTermState_tmp_selection) {
        self.tmp.selection = progress;
    }

    fn selection_table(&self) -> Option<&VTermSelectionCallbacks> {
        unsafe { self.selection.callbacks.as_ref() }
    }

    /// The staging buffer selection data is decoded into, one chunk at a time.
    pub(super) fn selection_buffer_mut(&mut self) -> &mut [u8] {
        if self.selection.buffer.is_null() {
            return &mut [];
        }
        unsafe {
            slice::from_raw_parts_mut(self.selection.buffer.cast::<u8>(), self.selection.buflen)
        }
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
        if let Some(f) = self.selection_table().and_then(|c| c.set) {
            unsafe { f(mask, frag, user) };
        }
    }

    /// Asks the consumer to report the current selection back to the host.
    pub(super) fn selection_query(&mut self, mask: VTermSelectionMask) {
        let user = self.selection.user;
        if let Some(f) = self.selection_table().and_then(|c| c.query) {
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

// ------------------------------------------------------------------ lifetime

/// Allocates a state machine for `vt` and gives it its power-on settings.
unsafe fn vterm_state_new(vt: *mut VTerm) -> *mut VTermState {
    let state = vterm_alloc(::core::mem::size_of::<VTermState>()) as *mut VTermState;

    (*state).vt = vt;
    (*state).rows = (*vt).rows;
    (*state).cols = (*vt).cols;
    (*state).mouse_col = 0;
    (*state).mouse_row = 0;
    (*state).mouse_buttons = 0;
    (*state).mouse_protocol = MOUSE_X10;
    (*state).callbacks = core::ptr::null();
    (*state).cbdata = core::ptr::null_mut();
    (*state).selection.callbacks = core::ptr::null();
    (*state).selection.user = core::ptr::null_mut();
    (*state).selection.buffer = core::ptr::null_mut();

    init_pen(&mut *state);
    (*state).bold_is_highbright = 0;
    (*state).combine_pos.row = -1;

    (*state).tabstops = vterm_alloc(tabstop_bytes((*state).cols)).cast::<uint8_t>();
    let marks = ((*state).rows.max(0) as size_t) * ::core::mem::size_of::<VTermLineInfo>();
    // TODO(vterm): the altscreen's marks could wait until it is switched on.
    (*state).lineinfos[BUFIDX_PRIMARY] = vterm_alloc(marks).cast::<VTermLineInfo>();
    (*state).lineinfos[BUFIDX_ALTSCREEN] = vterm_alloc(marks).cast::<VTermLineInfo>();
    (*state).lineinfo = (*state).lineinfos[BUFIDX_PRIMARY];

    let utf8 = vterm_lookup_encoding(ENC_UTF8, b'u' as c_char);
    (*state).encoding_utf8.enc = utf8;
    if let Some(init) = (*utf8).init {
        init(
            utf8,
            (&raw mut (*state).encoding_utf8.data).cast::<c_void>(),
        );
    }

    for stack in &mut (*state).key_encoding_stacks {
        stack.items = [VTermKeyEncodingFlags {
            disambiguate_report_events_report_alternate_report_all_keys_report_associated: [0; 1],
        }; 16];
        stack.size = 1;
    }

    state
}

pub unsafe extern "C" fn vterm_state_free(state: *mut VTermState) {
    vterm_dealloc((*state).tabstops.cast::<c_void>());
    vterm_dealloc((*state).lineinfos[BUFIDX_PRIMARY].cast::<c_void>());
    if !(*state).lineinfos[BUFIDX_ALTSCREEN].is_null() {
        vterm_dealloc((*state).lineinfos[BUFIDX_ALTSCREEN].cast::<c_void>());
    }
    vterm_dealloc(state.cast::<c_void>());
}

/// The terminal's state machine, created and wired to the parser on first ask.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_obtain_state(vt: *mut VTerm) -> *mut VTermState {
    if !(*vt).state.is_null() {
        return (*vt).state;
    }
    let state = vterm_state_new(vt);
    (*vt).state = state;
    vterm_parser_set_callbacks(vt, PARSER_CALLBACKS.ptr(), state.cast::<c_void>());
    state
}

// ------------------------------------------------------- the parser's events

static PARSER_CALLBACKS: GlobalCell<VTermParserCallbacks> = GlobalCell::new(VTermParserCallbacks {
    text: Some(on_text),
    control: Some(on_control),
    escape: Some(on_escape),
    csi: Some(on_csi),
    osc: Some(on_osc),
    dcs: Some(on_dcs),
    apc: Some(on_apc),
    pm: Some(on_pm),
    sos: Some(on_sos),
    resize: Some(on_resize),
});

/// Decodes a run of input bytes through the live character set and prints the
/// graphemes it yields. Returns how many input bytes were consumed, which is
/// short of `len` when the run ends part-way through a sequence.
unsafe extern "C" fn on_text(bytes: *const c_char, len: size_t, user: *mut c_void) -> c_int {
    let state = &mut *(user as *mut VTermState);

    // A high bit selects the right-hand set, a single shift overrides both,
    // and any slot designated UTF-8 collapses onto the one shared decoder so
    // that a multi-byte sequence can span calls.
    let lead = if len == 0 { 0 } else { *bytes as u8 };
    let slot = if state.gsingle_set != 0 {
        state.gsingle_set as usize
    } else if lead & 0x80 == 0 {
        state.gl_set as usize
    } else if state.utf8() {
        usize::MAX
    } else {
        state.gr_set as usize
    };
    let shared = match state.encoding.get(slot) {
        Some(instance) => instance.enc == state.encoding_utf8.enc,
        None => true,
    };
    let encoding = if shared {
        &raw mut state.encoding_utf8
    } else {
        &raw mut state.encoding[slot]
    };

    let vt = state.vt;
    let codepoints = (*vt).tmpbuffer.cast::<u32>();
    let maxpoints = (*vt).tmpbuffer_len / ::core::mem::size_of::<u32>();
    let mut npoints: c_int = 0;
    let mut eaten: size_t = 0;
    let enc = (*encoding).enc;
    let decode = (*enc).decode.expect("character set without a decoder");
    decode(
        enc,
        (&raw mut (*encoding).data).cast::<c_void>(),
        codepoints,
        &raw mut npoints,
        if state.gsingle_set != 0 {
            1
        } else {
            maxpoints as c_int
        },
        bytes,
        &raw mut eaten,
        len,
    );

    // A stateful set may not have seen enough bytes for even one codepoint.
    if npoints == 0 {
        return eaten as c_int;
    }
    state.gsingle_set = 0;
    text::print(state, slice::from_raw_parts(codepoints, npoints as usize));
    eaten as c_int
}

unsafe extern "C" fn on_control(control: uint8_t, user: *mut c_void) -> c_int {
    let state = &mut *(user as *mut VTermState);
    if text::control(state, control) {
        return 1;
    }
    match state.fallback_table().and_then(|f| f.control) {
        Some(f) => (f(control, state.fbdata) != 0) as c_int,
        None => 0,
    }
}

unsafe extern "C" fn on_escape(bytes: *const c_char, len: size_t, user: *mut c_void) -> c_int {
    let state = &mut *(user as *mut VTermState);
    text::escape(state, slice::from_raw_parts(bytes.cast::<u8>(), len))
}

unsafe extern "C" fn on_csi(
    leader: *const c_char,
    args: *const c_long,
    argcount: c_int,
    intermed: *const c_char,
    command: c_char,
    user: *mut c_void,
) -> c_int {
    // A leader or intermediate arrives as a NUL-terminated string, but only a
    // single-byte one means anything here, so two bytes decide it.
    let leader_bytes = if leader.is_null() || *leader == 0 {
        [0, 0]
    } else {
        [*leader as u8, *leader.offset(1) as u8]
    };
    let intermed_bytes = if intermed.is_null() || *intermed == 0 {
        [0, 0]
    } else {
        [*intermed as u8, *intermed.offset(1) as u8]
    };
    let state = &mut *(user as *mut VTermState);
    let outcome = csi::dispatch(
        state,
        leader_bytes,
        slice::from_raw_parts(args, argcount.max(0) as usize),
        intermed_bytes,
        command as u8,
    );
    match outcome {
        csi::Outcome::Handled => 1,
        csi::Outcome::Ignored => 0,
        csi::Outcome::Unrecognised => {
            let fbdata = state.fbdata;
            match state.fallback_table().and_then(|f| f.csi) {
                Some(f) => (f(leader, args, argcount, intermed, command, fbdata) != 0) as c_int,
                None => 0,
            }
        }
    }
}

/// Upstream offers every OSC to the fallback, even the ones it handled
/// itself, and reports only what the fallback made of it.
unsafe extern "C" fn on_osc(command: c_int, frag: VTermStringFragment, user: *mut c_void) -> c_int {
    let state = &mut *(user as *mut VTermState);
    selection::osc(state, command, frag);
    match state.fallback_table().and_then(|f| f.osc) {
        Some(f) => (f(command, frag, state.fbdata) != 0) as c_int,
        None => 0,
    }
}

unsafe extern "C" fn on_dcs(
    command: *const c_char,
    commandlen: size_t,
    frag: VTermStringFragment,
    user: *mut c_void,
) -> c_int {
    let state = &mut *(user as *mut VTermState);
    let name = slice::from_raw_parts(command.cast::<u8>(), commandlen);
    if dcs::device_control(state, name, frag) {
        return 1;
    }
    let fbdata = state.fbdata;
    match state.fallback_table().and_then(|f| f.dcs) {
        Some(f) => (f(command, commandlen, frag, fbdata) != 0) as c_int,
        None => 0,
    }
}

// APC, PM and SOS carry nothing this terminal understands, so each only
// reaches for its own fallback.
unsafe extern "C" fn on_apc(frag: VTermStringFragment, user: *mut c_void) -> c_int {
    let state = &mut *(user as *mut VTermState);
    match state.fallback_table().and_then(|f| f.apc) {
        Some(f) => (f(frag, state.fbdata) != 0) as c_int,
        None => 0,
    }
}

unsafe extern "C" fn on_pm(frag: VTermStringFragment, user: *mut c_void) -> c_int {
    let state = &mut *(user as *mut VTermState);
    match state.fallback_table().and_then(|f| f.pm) {
        Some(f) => (f(frag, state.fbdata) != 0) as c_int,
        None => 0,
    }
}

unsafe extern "C" fn on_sos(frag: VTermStringFragment, user: *mut c_void) -> c_int {
    let state = &mut *(user as *mut VTermState);
    match state.fallback_table().and_then(|f| f.sos) {
        Some(f) => (f(frag, state.fbdata) != 0) as c_int,
        None => 0,
    }
}

unsafe extern "C" fn on_resize(rows: c_int, cols: c_int, user: *mut c_void) -> c_int {
    let state = &mut *(user as *mut VTermState);
    let oldpos = state.pos;

    if cols != state.cols {
        state.resize_tabstops(cols);
    }
    state.rows = rows;
    state.cols = cols;
    if state.scrollregion_bottom > -1 {
        state.scrollregion_bottom = state.scrollregion_bottom.min(rows);
    }
    if state.scrollregion_right > -1 {
        state.scrollregion_right = state.scrollregion_right.min(cols);
    }

    // Upstream reallocated the line marks itself when no consumer took the
    // resize, but guarded that on a row count it had already overwritten, so
    // the branch never ran; the screen module is the only consumer there is,
    // and it always takes the resize.
    state.resize_consumer(rows, cols);
    state.lineinfo = state.lineinfos[state.active_buffer()];

    if state.at_phantom != 0 && state.pos.col < cols - 1 {
        state.at_phantom = 0;
        state.pos.col += 1;
    }
    state.pos.row = state.pos.row.clamp(0, (rows - 1).max(0));
    state.pos.col = state.pos.col.clamp(0, (cols - 1).max(0));
    state.update_cursor(oldpos, true);
    1
}

// ---------------------------------------------------------------- public API

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_reset(state: *mut VTermState, hard: c_int) {
    mode::reset(&mut *state, hard != 0);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_set_callbacks(
    state: *mut VTermState,
    callbacks: *const VTermStateCallbacks,
    user: *mut c_void,
) {
    let installed = !callbacks.is_null();
    (*state).callbacks = callbacks;
    (*state).cbdata = if installed {
        user
    } else {
        core::ptr::null_mut()
    };
    if installed {
        (*state).init_consumer_pen();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_set_unrecognised_fallbacks(
    state: *mut VTermState,
    fallbacks: *const VTermStateFallbacks,
    user: *mut c_void,
) {
    (*state).fallbacks = fallbacks;
    (*state).fbdata = if fallbacks.is_null() {
        core::ptr::null_mut()
    } else {
        user
    };
}

/// Applies a terminal property, offering it to the consumer first, which may
/// refuse it. Refusal matters most for the alternate screen: the state must
/// not believe it switched if the screen did not.
pub unsafe extern "C" fn vterm_state_set_termprop(
    state: *mut VTermState,
    prop: VTermProp,
    val: *mut VTermValue,
) -> c_int {
    let state = &mut *state;
    let cbdata = state.cbdata;
    if let Some(f) = state.callback_table().and_then(|c| c.settermprop)
        && f(prop, val, cbdata) == 0
    {
        return 0;
    }

    match prop {
        // Titles are passed straight through, never stored.
        VTERM_PROP_TITLE | VTERM_PROP_ICONNAME => {}
        VTERM_PROP_CURSORVISIBLE => state.mode.set_cursor_visible((*val).boolean as c_uint),
        VTERM_PROP_CURSORBLINK => state.mode.set_cursor_blink((*val).boolean as c_uint),
        VTERM_PROP_CURSORSHAPE => state.mode.set_cursor_shape((*val).number as c_uint),
        VTERM_PROP_REVERSE => state.mode.set_screen((*val).boolean as c_uint),
        VTERM_PROP_ALTSCREEN => {
            state.mode.set_alt_screen((*val).boolean as c_uint);
            state.lineinfo = state.lineinfos[state.active_buffer()];
            if state.mode.alt_screen() != 0 {
                let rect = VTermRect {
                    start_row: 0,
                    end_row: state.rows,
                    start_col: 0,
                    end_col: state.cols,
                };
                state.erase(rect, false);
            }
        }
        VTERM_PROP_MOUSE => {
            let level = (*val).number;
            state.mouse_flags = 0;
            if level != 0 {
                state.mouse_flags |= MOUSE_WANT_CLICK;
            }
            if level == mode::VTERM_PROP_MOUSE_DRAG {
                state.mouse_flags |= MOUSE_WANT_DRAG;
            }
            if level == mode::VTERM_PROP_MOUSE_MOVE {
                state.mouse_flags |= MOUSE_WANT_MOVE;
            }
        }
        VTERM_PROP_FOCUSREPORT => state.mode.set_report_focus((*val).boolean as c_uint),
        VTERM_PROP_THEMEUPDATES => state.mode.set_theme_updates((*val).boolean as c_uint),
        VTERM_PROP_SYNCOUTPUT => state.mode.set_synchronized_output((*val).boolean as c_uint),
        _ => return 0,
    }
    1
}

/// Reports that the terminal window gained focus, if the host asked to hear.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_focus_in(state: *mut VTermState) {
    (*state).report_focus(b'I');
}

/// Reports that the terminal window lost focus, if the host asked to hear.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_focus_out(state: *mut VTermState) {
    (*state).report_focus(b'O');
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_get_lineinfo(
    state: *const VTermState,
    row: c_int,
) -> *const VTermLineInfo {
    (*state).lineinfo.offset(row as isize)
}

/// Installs the consumer's selection handling, allocating the staging buffer
/// the decoder chunks through when the consumer did not supply one.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_state_set_selection_callbacks(
    state: *mut VTermState,
    callbacks: *const VTermSelectionCallbacks,
    user: *mut c_void,
    buffer: *mut c_char,
    buflen: size_t,
) {
    let buffer = if buflen != 0 && buffer.is_null() {
        vterm_alloc(buflen).cast::<c_char>()
    } else {
        buffer
    };
    (*state).selection.callbacks = callbacks;
    (*state).selection.user = user;
    (*state).selection.buffer = buffer;
    (*state).selection.buflen = buflen;
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
