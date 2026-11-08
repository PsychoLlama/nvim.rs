//! The terminal state machine: the object the escape-sequence parser drives.
//!
//! It remembers everything a terminal remembers apart from the screen itself —
//! where the cursor is, which modes are set, what the scroll region and the
//! tab stops are, which character sets are mapped — and hands every visible
//! effect to its consumer, which in practice is [`crate::vterm::screen`].
//!
//! The state machine is driven through a [`StateCx`]: the state itself plus
//! what it borrows of the terminal around it for the length of one call —
//! the mode bits it shares with the parser, the reply channel, the text
//! decoder's scratch and the screen it paints on. The sequence handling lives
//! in sibling modules — [`super::text`] for printable text and the C0/C1
//! controls, [`super::csi`] for control sequences, [`super::mode`] for modes
//! and properties, [`super::selection`] for OSC, [`super::dcs`] for the other
//! control strings, and `geometry` for the shape of the screen — all of it
//! written against the methods here. The state machine's construction and its
//! parser events live in [`entry`](self::entry).
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

pub mod entry;

use crate::narrow::as_index;
use crate::narrow::count_as_int;
use core::ffi::{c_int, c_long, c_uint};
use core::ops::{Deref, DerefMut};

use crate::grid::schar_from_buf;
use crate::mbyte::GraphemeState;
use crate::mbyte::{c_partial_cells_in, encode_char, iscomposing};
use crate::types::ScreenChar;
use crate::vterm::abi::ForeignState;
use crate::vterm::encoding::{self, Charset};
use crate::vterm::host::{Fallbacks, Fragment, SelectionHost, Value};
use crate::vterm::output::EscapeSeq;
use crate::vterm::pen::{apply_sgr, reset_pen, restore_pen};
use crate::vterm::screen::{BUFIDX_ALTSCREEN, BUFIDX_PRIMARY};
use crate::vterm::types::{
    VTerm_mode, VTermAttr, VTermGlyphInfo, VTermLineInfo, VTermPos, VTermProp, VTermRect,
    VTermScreen, VTermSelectionMask, VTermState, VTermState_tmp, VTermState_tmp_selection,
    VTermStateFields,
};
use crate::vterm::vterm::{Output, ScrollTarget, scroll_rect};

/// What the host asked to be told about the pointer, as
/// `VTERM_PROP_MOUSE` reduces to.
pub const MOUSE_WANT_CLICK: c_int = 0x1;
pub const MOUSE_WANT_DRAG: c_int = 0x2;
pub const MOUSE_WANT_MOVE: c_int = 0x4;

/// A change to the pen that the consumer is told about.
pub(super) enum PenChange<'a> {
    /// Apply an SGR control sequence.
    Sgr(&'a [c_long]),
    /// Put back the pen saved under DEC mode 1048.
    Restore,
    /// Return the pen to its defaults.
    Reset,
}

/// A line with no double-width, double-height or continuation marking.
pub(super) const BLANK_LINE: VTermLineInfo = VTermLineInfo {
    doublewidth_doubleheight_continuation: [0; 1],
    _pad: [0; 3],
};

/// Whether `next` extends the grapheme that `prev` ended.
pub(super) fn is_composing(prev: u32, next: u32, grapheme: &mut GraphemeState) -> bool {
    iscomposing(prev.cast_signed(), next.cast_signed(), Some(grapheme))
}

/// How many bytes the tab-stop bit vector needs for `cols` columns.
fn tabstop_bytes(cols: c_int) -> usize {
    as_index(cols.max(0)).div_ceil(8)
}

/// Who the state machine paints through.
#[derive(Copy, Clone)]
pub(crate) enum Consumer {
    None,
    /// The terminal's own screen.
    Screen,
    /// A C consumer's callback table.
    Foreign(ForeignState),
}

/// What a state machine owns that C does not read.
pub(crate) struct StateOwned {
    /// One bit per column.
    tabstops: Vec<u8>,
    /// Per-row double-width, double-height and continuation marks, for the
    /// primary screen and the alternate one.
    pub(crate) lineinfos: [Vec<VTermLineInfo>; 2],
    /// Which of `lineinfos` the state is reading. Moved by the alternate
    /// screen switch and by a resize, and by nothing else -- a reset leaves
    /// it where it was, as libvterm's pointer was left.
    pub(crate) active_lineinfo: usize,
    /// Where an OSC 52 write is decoded into, a chunk at a time.
    selection_buffer: Vec<u8>,
    pub(crate) consumer: Consumer,
    pub(crate) fallbacks: Option<Box<dyn Fallbacks>>,
    pub(crate) selection: Option<Box<dyn SelectionHost>>,
}

impl StateOwned {
    pub(super) fn new(rows: c_int, cols: c_int) -> Self {
        let rows = as_index(rows.max(0));
        StateOwned {
            tabstops: vec![0; tabstop_bytes(cols)],
            // TODO(vterm): the altscreen's marks could wait until it is
            // switched on.
            lineinfos: [vec![BLANK_LINE; rows], vec![BLANK_LINE; rows]],
            active_lineinfo: BUFIDX_PRIMARY,
            selection_buffer: Vec::new(),
            consumer: Consumer::None,
            fallbacks: None,
            selection: None,
        }
    }
}

impl VTermState_tmp {
    /// The bytes of a DECRQSS request gathered so far, NUL-terminated. They
    /// share their four bytes with the selection decoder's progress, as
    /// libvterm's union had it: only one of the two is ever part-way through.
    pub(super) fn decrqss(&self) -> [u8; 4] {
        let [low, high] = self.selection.mask.to_ne_bytes();
        [low, high, self.selection.state[0], self.selection._pad[0]]
    }

    pub(super) fn set_decrqss(&mut self, request: [u8; 4]) {
        self.selection.mask = u16::from_ne_bytes([request[0], request[1]]);
        self.selection.state = [request[2]];
        self.selection._pad = [request[3]];
    }
}

// ------------------------------------------------- what the state owns alone

impl VTermState {
    /// Which of the two buffers the live screen is: the key-encoding stacks
    /// are kept per buffer.
    pub(super) fn active_buffer(&self) -> usize {
        if self.mode.alt_screen() != 0 {
            BUFIDX_ALTSCREEN
        } else {
            BUFIDX_PRIMARY
        }
    }

    /// Per-row double-width and double-height marks, one entry per screen row.
    pub(crate) fn lineinfo(&self) -> &[VTermLineInfo] {
        &self.owned.lineinfos[self.owned.active_lineinfo]
    }

    pub(super) fn lineinfo_mut(&mut self) -> &mut [VTermLineInfo] {
        let active = self.owned.active_lineinfo;
        &mut self.owned.lineinfos[active]
    }

    /// Points the state at the line marks of the buffer the mode bits say is
    /// live.
    fn follow_active_buffer(&mut self) {
        self.owned.active_lineinfo = self.active_buffer();
    }

    /// The tab-stop bit vector, one bit per column.
    pub(super) fn tabstops_mut(&mut self) -> &mut [u8] {
        &mut self.owned.tabstops
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
                let col = count_as_int(byte) << 3 | bit;
                // Columns the old screen had keep their stop; fresh ones get
                // the default one every eighth column.
                let stop = if col < self.cols {
                    self.owned.tabstops[byte] & 1 << bit != 0
                } else {
                    col % 8 == 0
                };
                bits |= u8::from(stop && col < cols) << bit;
            }
            stops.push(bits);
        }
        self.owned.tabstops = stops;
    }

    /// Makes the terminal's own screen this state's consumer, as libvterm's
    /// `vterm_obtain_screen` installed the screen's callback table.
    pub(crate) fn attach_screen(&mut self) {
        self.callbacks = core::ptr::null();
        self.cbdata = core::ptr::null_mut();
        self.owned.consumer = Consumer::Screen;
    }

    /// Installs a consumer, telling it to reset its pen when it is a C one
    /// that asked to be told.
    pub(crate) fn set_consumer(&mut self, consumer: Consumer) {
        self.owned.consumer = consumer;
    }

    /// Installs the sequences this state does not recognise.
    pub(crate) fn set_fallbacks(&mut self, fallbacks: Option<Box<dyn Fallbacks>>) {
        self.owned.fallbacks = fallbacks;
    }

    /// Installs OSC 52 handling, with a staging buffer of `buflen` bytes.
    pub(crate) fn set_selection(&mut self, host: Option<Box<dyn SelectionHost>>, buflen: usize) {
        self.owned.selection = host;
        self.owned.selection_buffer = vec![0; buflen];
        self.selection.buflen = buflen;
    }

    /// The line-mark array the state is reading, for a C caller to index.
    pub(crate) fn lineinfo_base(&self) -> *const VTermLineInfo {
        self.lineinfo().as_ptr()
    }

    // ------------------------------------------------- the grapheme buffer

    /// Appends `codepoint` to the pending grapheme at offset `at`, returning
    /// how many bytes it took.
    ///
    /// Upstream guarded the buffer against a four-byte encoding but let a
    /// six-byte one write past its end; anything that does not fit is dropped
    /// here instead.
    pub(super) fn append_grapheme(&mut self, at: usize, codepoint: u32) -> usize {
        let mut encoded = [0u8; 8];
        // Eight bytes, more than any encoding needs.
        let len = encode_char(codepoint.cast_signed(), &mut encoded);
        let len = len.min(self.grapheme_buf.len() - at);
        for (to, &from) in self.grapheme_buf[at..at + len].iter_mut().zip(&encoded) {
            *to = from.cast_signed();
        }
        len
    }

    /// How many columns the first `len` bytes of the pending grapheme occupy,
    /// and the handle the screen stores that grapheme under.
    pub(super) fn grapheme_metrics(&self, len: usize) -> (c_int, ScreenChar) {
        let bytes = self.grapheme_buf.map(core::ffi::c_char::cast_unsigned);
        let cells = c_partial_cells_in(&bytes, 0, len);
        (cells, schar_from_buf(&bytes[..len]))
    }

    // --------------------------------------------------------- character sets

    /// Designates slot `slot` (G0 to G3) to the 94-character set named by
    /// `designator`. An unknown name leaves the slot alone.
    pub(super) fn designate_charset(&mut self, slot: usize, designator: u8) {
        if let Some(enc) = encoding::lookup(encoding::ENC_SINGLE_94, designator) {
            self.encoding[slot].enc = Some(enc);
            self.init_encoding(slot);
        }
    }

    /// Points every slot at the terminal's default set: UTF-8 when the
    /// terminal decodes UTF-8, ASCII otherwise.
    pub(super) fn reset_charsets(&mut self, utf8: bool) {
        let default = if utf8 {
            encoding::lookup(encoding::ENC_UTF8, b'u')
        } else {
            encoding::lookup(encoding::ENC_SINGLE_94, b'B')
        };
        for slot in 0..self.encoding.len() {
            self.encoding[slot].enc = default;
            self.init_encoding(slot);
        }
    }

    /// Resets slot `slot`'s decoder scratch.
    fn init_encoding(&mut self, slot: usize) {
        let instance = &mut self.encoding[slot];
        if let Some(charset) = instance.enc.map(Charset::of) {
            charset.init(&mut instance.data);
        }
    }

    // ------------------------------------------------------------- selection

    /// How far the OSC 52 decoder has got. It shares its storage with the
    /// DECRQSS request, so only one of the two can be part-way through.
    pub(super) fn selection_progress(&self) -> VTermState_tmp_selection {
        self.tmp.selection
    }

    pub(super) fn set_selection_progress(&mut self, progress: VTermState_tmp_selection) {
        self.tmp.selection = progress;
    }

    /// Whether the consumer wants to hear about OSC 52 selection traffic.
    pub(super) fn selection_enabled(&self) -> bool {
        self.owned.selection.is_some()
    }

    /// Whether the consumer will take a selection at all.
    pub(super) fn selection_accepts_set(&self) -> bool {
        self.owned
            .selection
            .as_ref()
            .is_some_and(|host| host.takes_set())
    }

    /// The staging buffer selection data is decoded into, one chunk at a time.
    pub(super) fn selection_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.owned.selection_buffer
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
        let StateOwned {
            selection,
            selection_buffer,
            ..
        } = &mut *self.owned;
        let Some(host) = selection else {
            return;
        };
        let frag = match len {
            Some(len) => Fragment::new(&selection_buffer[..len], initial, last, 0),
            None => Fragment::absent(initial, true),
        };
        let _ = host.set(mask, frag);
    }

    /// Asks the consumer to report the current selection back to the host.
    pub(super) fn selection_query(&mut self, mask: VTermSelectionMask) {
        if let Some(host) = &mut self.owned.selection {
            let _ = host.query(mask);
        }
    }
}

// --------------------------------------------- the state, driven for a call

/// A state machine and what it borrows of its terminal for the length of one
/// call: everything that call can reach.
///
/// Dereferences to the [`VTermState`], so the sequence handlers read and write
/// its fields directly; what goes past the state -- a reply on the wire, a
/// glyph on the screen, a report to the consumer -- is a method here.
pub(crate) struct StateCx<'a> {
    state: &'a mut VTermState,
    /// The terminal's mode bits: whether input is UTF-8, and whether C1
    /// controls are sent 8-bit.
    vt_mode: &'a mut VTerm_mode,
    output: &'a mut Output,
    /// Where the text decoder writes codepoints.
    pub(super) scratch: &'a mut [u32],
    screen: Option<&'a mut VTermScreen>,
}

impl Deref for StateCx<'_> {
    type Target = VTermState;

    fn deref(&self) -> &VTermState {
        self.state
    }
}

impl DerefMut for StateCx<'_> {
    fn deref_mut(&mut self) -> &mut VTermState {
        self.state
    }
}

impl<'a> StateCx<'a> {
    pub(crate) fn new(
        state: &'a mut VTermState,
        vt_mode: &'a mut VTerm_mode,
        output: &'a mut Output,
        scratch: &'a mut [u32],
        screen: Option<&'a mut VTermScreen>,
    ) -> Self {
        StateCx {
            state,
            vt_mode,
            output,
            scratch,
            screen,
        }
    }
}

impl StateCx<'_> {
    /// Whether the host asked for single-byte C1 controls on the wire.
    pub(super) fn ctrl8bit(&self) -> bool {
        self.vt_mode.ctrl8bit() != 0
    }

    /// Selects between the 7-bit and 8-bit forms of the C1 controls.
    pub(super) fn set_ctrl8bit(&mut self, eight_bit: bool) {
        self.vt_mode.set_ctrl8bit(c_uint::from(eight_bit));
    }

    /// Whether the terminal is decoding its input as UTF-8.
    pub(super) fn utf8(&self) -> bool {
        self.vt_mode.utf8() != 0
    }

    /// Writes a reply back to the host. A sequence that outgrew its builder is
    /// dropped whole rather than truncated onto the wire.
    pub(super) fn reply(&mut self, seq: &EscapeSeq) {
        if let Some(bytes) = seq.finish() {
            self.output.push(bytes);
        }
    }

    /// The consumer to report to, and the screen if that is the consumer.
    fn consumer(&mut self) -> (Consumer, Option<&mut VTermScreen>, &VTermState) {
        let consumer = self.state.owned.consumer;
        let screen = match consumer {
            Consumer::Screen => self.screen.as_deref_mut(),
            _ => None,
        };
        (consumer, screen, self.state)
    }

    // -------------------------------------------------------- screen effects

    /// Stamps one grapheme onto the screen at `pos`.
    pub(super) fn put_glyph(&mut self, schar: ScreenChar, width: c_int, pos: VTermPos) {
        let line = self.lineinfo()[as_index(pos.row)];
        let mut info = VTermGlyphInfo {
            schar,
            width,
            protected_cell_dwl_dhl: [0; 1],
            _pad: [0; 3],
        };
        info.set_protected_cell(self.protected_cell());
        info.set_dwl(line.doublewidth());
        info.set_dhl(line.doubleheight());
        match self.consumer() {
            (Consumer::Screen, Some(screen), _) => {
                screen.put_glyph(&info, pos);
            }
            (Consumer::Foreign(foreign), ..) => {
                let _ = foreign.putglyph(&mut info, pos);
            }
            _ => {}
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
        let (pos, visible) = (self.pos, self.mode.cursor_visible().cast_signed());
        match self.consumer() {
            (Consumer::Screen, Some(screen), _) => {
                screen.movecursor(pos, oldpos, visible);
            }
            (Consumer::Foreign(foreign), ..) => {
                let _ = foreign.movecursor(pos, oldpos, visible);
            }
            _ => {}
        }
    }

    /// Blanks a rectangle. `selective` spares cells marked protected.
    pub(super) fn erase(&mut self, rect: VTermRect, selective: bool) {
        if rect.end_col == self.cols {
            // Erasing the final cells of a line cancels the continuation mark
            // on the line that followed it.
            let last = (rect.end_row + 1).min(self.rows);
            for row in (rect.start_row + 1).max(0)..last {
                self.lineinfo_mut()[as_index(row)].set_continuation(0);
            }
        }
        match self.consumer() {
            (Consumer::Screen, Some(screen), state) => {
                screen.erase(rect, selective, state.rows, state.lineinfo());
            }
            (Consumer::Foreign(foreign), ..) => {
                let _ = foreign.erase(rect, c_int::from(selective));
            }
            _ => {}
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
            let start = as_index(rect.start_row);
            let height = as_index(rows - downward.abs());
            let lines = self.lineinfo_mut();
            if downward > 0 {
                let shift = as_index(downward);
                lines.copy_within(start + shift..start + shift + height, start);
                lines[as_index(rect.end_row - downward)..as_index(rect.end_row)].fill(BLANK_LINE);
            } else {
                let shift = as_index(-downward);
                lines.copy_within(start..start + height, start + shift);
                lines[start..start + shift].fill(BLANK_LINE);
            }
        }

        match self.consumer() {
            (Consumer::Screen, Some(screen), state) => {
                screen.scroll_rect(rect, downward, rightward, state.rows, state.lineinfo());
            }
            (Consumer::Foreign(mut foreign), ..) => {
                let taken = foreign.scrollrect(rect, downward, rightward);
                if taken.is_none_or(|taken| taken == 0) {
                    // The consumer did not take the scroll whole, so drive it
                    // out of the move and erase primitives instead.
                    assert!(foreign.has_erase(), "scrolling without a way to erase");
                    scroll_rect(rect, downward, rightward, &mut foreign);
                }
            }
            _ => {}
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
        let mut info = self.lineinfo()[as_index(row)];
        info.set_doublewidth(c_uint::from(doublewidth));
        info.set_doubleheight(doubleheight);
        let accepted = match self.consumer() {
            (Consumer::Screen, Some(screen), state) => {
                let lines = state.lineinfo();
                screen.set_lineinfo(row, &info, &lines[as_index(row)], state.rows, lines) != 0
            }
            (Consumer::Foreign(foreign), _, state) => {
                let current = &state.lineinfo()[as_index(row)];
                foreign
                    .setlineinfo(row, &info, current)
                    .is_some_and(|accepted| accepted != 0)
            }
            _ => false,
        };
        if accepted || force {
            self.lineinfo_mut()[as_index(row)] = info;
        }
    }

    /// Rings the terminal bell.
    pub(super) fn bell(&mut self) {
        match self.consumer() {
            (Consumer::Screen, Some(screen), _) => {
                screen.bell();
            }
            (Consumer::Foreign(foreign), ..) => {
                let _ = foreign.bell();
            }
            _ => {}
        }
    }

    /// Drops the scrollback, reporting whether the consumer took the request.
    pub(super) fn clear_scrollback(&mut self) -> bool {
        match self.consumer() {
            (Consumer::Screen, Some(screen), _) => screen.sb_clear() != 0,
            (Consumer::Foreign(foreign), ..) => foreign.sb_clear().is_some_and(|taken| taken != 0),
            _ => false,
        }
    }

    /// Asks the consumer whether it is showing a dark colour scheme; `None`
    /// when it does not know.
    pub(super) fn theme_is_dark(&mut self) -> Option<bool> {
        let mut dark = false;
        let known = match self.consumer() {
            (Consumer::Screen, Some(screen), _) => screen.theme(&mut dark),
            (Consumer::Foreign(foreign), ..) => foreign.theme(&mut dark)?,
            _ => return None,
        };
        (known != 0).then_some(dark)
    }

    /// Tells the consumer to return its own pen to the defaults.
    pub(super) fn init_consumer_pen(&mut self) {
        if let (Consumer::Foreign(foreign), ..) = self.consumer() {
            let _ = foreign.initpen();
        }
    }

    /// Tells the consumer one attribute of the pen changed.
    pub(super) fn report_pen(&mut self, attr: VTermAttr, val: Value<'_>) {
        match self.consumer() {
            (Consumer::Screen, Some(screen), _) => {
                screen.set_pen_attr(attr, &val);
            }
            (Consumer::Foreign(foreign), ..) => {
                let _ = foreign.setpenattr(attr, &val);
            }
            _ => {}
        }
    }

    /// Offers a terminal property to the consumer, which may refuse it.
    /// Answers whether it was accepted; no consumer accepts everything.
    fn offer_termprop(&mut self, prop: VTermProp, val: &Value<'_>) -> bool {
        match self.consumer() {
            (Consumer::Screen, Some(screen), _) => screen.set_termprop(prop, val) != 0,
            (Consumer::Foreign(foreign), ..) => foreign.settermprop(prop, val) != Some(0),
            _ => true,
        }
    }

    /// Resizes the consumer's screen, letting it move the cursor and replace
    /// the line-mark arrays.
    fn resize_consumer(&mut self, rows: c_int, cols: c_int) {
        let consumer = self.state.owned.consumer;
        if let (Consumer::Screen, Some(screen)) = (consumer, self.screen.as_deref_mut()) {
            let state = &mut *self.state;
            screen.resize(rows, cols, &mut state.pos, &mut state.owned.lineinfos);
            return;
        }
        // No screen to rebuild them, so the marks follow the new height here,
        // keeping the rows that survive.
        for lines in &mut self.state.owned.lineinfos {
            lines.resize(as_index(rows.max(0)), BLANK_LINE);
        }
        if let Consumer::Foreign(foreign) = consumer {
            let [primary, altscreen] = &mut self.state.owned.lineinfos;
            let mut fields = VTermStateFields {
                pos: self.state.pos,
                lineinfos: [primary.as_mut_ptr(), altscreen.as_mut_ptr()],
            };
            if foreign.resize(rows, cols, &mut fields).is_some() {
                self.state.pos = fields.pos;
            }
        }
    }

    /// The parser's resize: the screen changed size underneath the state.
    pub(crate) fn resize(&mut self, rows: c_int, cols: c_int) -> Option<c_int> {
        let oldpos = self.pos;
        if cols != self.cols {
            self.resize_tabstops(cols);
        }
        self.rows = rows;
        self.cols = cols;
        if self.scrollregion_bottom > -1 {
            self.scrollregion_bottom = self.scrollregion_bottom.min(rows);
        }
        if self.scrollregion_right > -1 {
            self.scrollregion_right = self.scrollregion_right.min(cols);
        }

        self.resize_consumer(rows, cols);
        self.follow_active_buffer();

        if self.at_phantom != 0 && self.pos.col < cols - 1 {
            self.at_phantom = 0;
            self.pos.col += 1;
        }
        self.pos.row = self.pos.row.clamp(0, (rows - 1).max(0));
        self.pos.col = self.pos.col.clamp(0, (cols - 1).max(0));
        self.update_cursor(oldpos, true);
        Some(1)
    }

    // -------------------------------------------------------------- the pen

    /// Makes one of the changes to the pen that are echoed to the consumer.
    pub(super) fn change_pen(&mut self, change: PenChange<'_>) {
        match change {
            PenChange::Sgr(args) => apply_sgr(self, args),
            PenChange::Restore => restore_pen(self),
            PenChange::Reset => reset_pen(self),
        }
    }

    // --------------------------------------------------------- character sets

    /// [`VTermState::reset_charsets`] for the terminal's own UTF-8 setting.
    pub(super) fn reset_charsets(&mut self) {
        let utf8 = self.utf8();
        self.state.reset_charsets(utf8);
    }

    // ------------------------------------------------------- terminal properties

    pub(super) fn set_termprop_bool(&mut self, prop: VTermProp, value: bool) -> bool {
        self.set_termprop(prop, &Value::Bool(value))
    }

    pub(super) fn set_termprop_int(&mut self, prop: VTermProp, value: c_int) -> bool {
        self.set_termprop(prop, &Value::Int(value))
    }

    pub(super) fn set_termprop_string(&mut self, prop: VTermProp, s: Fragment<'_>) -> bool {
        self.set_termprop(prop, &Value::Str(s))
    }

    // ------------------------------------------------------------- fallbacks

    /// `CSI I` / `CSI O`, the focus reports.
    pub(crate) fn report_focus(&mut self, command: u8) {
        if self.mode.report_focus() == 0 {
            return;
        }
        let mut seq = EscapeSeq::csi(self.ctrl8bit());
        seq.push(command);
        self.reply(&seq);
    }

    /// Runs `call` against the consumer's fallbacks with the state machine
    /// in hand, if there are fallbacks; the table is out of the state while
    /// it runs, so a fallback that reaches back into the state finds none.
    pub(super) fn with_fallbacks<T>(
        &mut self,
        call: impl FnOnce(&mut dyn Fallbacks, &mut StateCx<'_>) -> Option<T>,
    ) -> Option<T> {
        let mut fallbacks = self.state.owned.fallbacks.take()?;
        let answer = call(&mut *fallbacks, self);
        self.state.owned.fallbacks = Some(fallbacks);
        answer
    }
}

impl ScrollTarget for ForeignState {
    fn moverect(&mut self, dest: VTermRect, src: VTermRect) {
        let _ = ForeignState::moverect(*self, dest, src);
    }

    fn erase(&mut self, rect: VTermRect) {
        let _ = ForeignState::erase(*self, rect, 0);
    }
}

#[cfg(test)]
pub(super) mod test_cx {
    //! A bare state machine for the sequence handlers' own tests: no consumer
    //! and nobody listening for replies.

    use super::*;
    use crate::vterm::types::VTerm;

    /// Runs `body` against a state machine `rows` by `cols`.
    pub(in crate::vterm) fn with_state(
        rows: c_int,
        cols: c_int,
        body: impl FnOnce(&mut StateCx<'_>),
    ) {
        let mut vt = VTerm::new(rows, cols);
        vt.obtain_state();
        let mut cx = vt.state_cx().expect("a state machine was just made");
        cx.scrollregion_bottom = -1;
        cx.scrollregion_right = -1;
        body(&mut cx);
    }
}
