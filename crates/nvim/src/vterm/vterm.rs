//! The terminal object: its size, the buffer replies are collected in, and
//! the storage its state machine and screen are built out of.
//!
//! This is also where the vocabulary the rest of the subtree shares lives —
//! the property, attribute and selection numbers, and the parser's states.
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

use core::ffi::{c_int, c_uint};

use crate::types::{
    ParserScratch, VTerm, VTerm_mode, VTerm_parser, VTermAttr, VTermColor, VTermDamageSize,
    VTermKey, VTermModifier, VTermParserState, VTermPos, VTermProp, VTermRect, VTermScreen,
    VTermScreenCell, VTermSelectionMask, VTermState, VTermState_mouse_protocol,
    VTermState_tmp_selection_state, VTermTerminator, VTermValueType,
};
use crate::vterm::abi::{ForeignOutput, ForeignParser};
use crate::vterm::host::{
    Fallbacks, NoParserHandler, ParserHandler, ScreenHost, SelectionHost, Value,
};
use crate::vterm::mode;
use crate::vterm::output::EscapeSeq;
use crate::vterm::parser;
use crate::vterm::pen::{convert_color_to_rgb, set_palette_color};
use crate::vterm::state::StateCx;

/// How much of the screen a single damaged cell is reported as.
pub const VTERM_DAMAGE_CELL: VTermDamageSize = 0;
pub const VTERM_DAMAGE_ROW: VTermDamageSize = 1;
pub const VTERM_DAMAGE_SCREEN: VTermDamageSize = 2;
pub const VTERM_DAMAGE_SCROLL: VTermDamageSize = 3;
/// How a control string was ended.
pub const VTERM_TERMINATOR_BEL: VTermTerminator = 0;
pub const VTERM_TERMINATOR_ST: VTermTerminator = 1;

/// The properties a consumer is told about.
pub const VTERM_PROP_CURSORVISIBLE: VTermProp = 1;
pub const VTERM_PROP_CURSORBLINK: VTermProp = 2;
pub const VTERM_PROP_ALTSCREEN: VTermProp = 3;
pub const VTERM_PROP_TITLE: VTermProp = 4;
pub const VTERM_PROP_ICONNAME: VTermProp = 5;
pub const VTERM_PROP_REVERSE: VTermProp = 6;
pub const VTERM_PROP_CURSORSHAPE: VTermProp = 7;
pub const VTERM_PROP_MOUSE: VTermProp = 8;
pub const VTERM_PROP_FOCUSREPORT: VTermProp = 9;
pub const VTERM_PROP_THEMEUPDATES: VTermProp = 10;
pub const VTERM_PROP_SYNCOUTPUT: VTermProp = 11;
pub const VTERM_N_PROPS: VTermProp = 12;

/// Which selection buffer an OSC 52 names. The eight cut buffers run upwards
/// from `CUT0`.
pub const VTERM_SELECTION_CLIPBOARD: VTermSelectionMask = 1;
pub const VTERM_SELECTION_PRIMARY: VTermSelectionMask = 2;
pub const VTERM_SELECTION_SECONDARY: VTermSelectionMask = 4;
pub const VTERM_SELECTION_SELECT: VTermSelectionMask = 8;
pub const VTERM_SELECTION_CUT0: VTermSelectionMask = 16;

/// How far an OSC 52 has been read.
pub const SELECTION_INITIAL: VTermState_tmp_selection_state = 0;
pub const SELECTION_SELECTED: VTermState_tmp_selection_state = 1;
pub const SELECTION_QUERY: VTermState_tmp_selection_state = 2;
pub const SELECTION_SET_INITIAL: VTermState_tmp_selection_state = 3;
pub const SELECTION_SET: VTermState_tmp_selection_state = 4;
pub const SELECTION_INVALID: VTermState_tmp_selection_state = 5;

/// How pointer events are encoded on the wire.
pub const MOUSE_X10: VTermState_mouse_protocol = 0;
pub const MOUSE_UTF8: VTermState_mouse_protocol = 1;
pub const MOUSE_SGR: VTermState_mouse_protocol = 2;
pub const MOUSE_RXVT: VTermState_mouse_protocol = 3;

/// The pen attributes a consumer is told about.
pub const VTERM_ATTR_BOLD: VTermAttr = 1;
pub const VTERM_ATTR_UNDERLINE: VTermAttr = 2;
pub const VTERM_ATTR_ITALIC: VTermAttr = 3;
pub const VTERM_ATTR_BLINK: VTermAttr = 4;
pub const VTERM_ATTR_REVERSE: VTermAttr = 5;
pub const VTERM_ATTR_CONCEAL: VTermAttr = 6;
pub const VTERM_ATTR_STRIKE: VTermAttr = 7;
pub const VTERM_ATTR_FONT: VTermAttr = 8;
pub const VTERM_ATTR_FOREGROUND: VTermAttr = 9;
pub const VTERM_ATTR_BACKGROUND: VTermAttr = 10;
pub const VTERM_ATTR_SMALL: VTermAttr = 11;
pub const VTERM_ATTR_BASELINE: VTermAttr = 12;
pub const VTERM_ATTR_URI: VTermAttr = 13;
pub const VTERM_ATTR_DIM: VTermAttr = 14;
pub const VTERM_ATTR_OVERLINE: VTermAttr = 15;
pub const VTERM_N_ATTRS: VTermAttr = 16;

/// Where the escape-sequence parser is in a sequence. Everything from
/// `OSC_COMMAND` down collects a control string.
pub const NORMAL: VTermParserState = 0;
pub const OSC_COMMAND: VTermParserState = 5;
pub const OSC: VTermParserState = 6;
pub const APC: VTermParserState = 8;
pub const PM: VTermParserState = 9;
pub const SOS: VTermParserState = 10;

/// What a `VTermValue` holds, which the attribute or property it accompanies
/// decides.
pub const VTERM_VALUETYPE_BOOL: VTermValueType = 1;
pub const VTERM_VALUETYPE_INT: VTermValueType = 2;
pub const VTERM_VALUETYPE_STRING: VTermValueType = 3;
pub const VTERM_VALUETYPE_COLOR: VTermValueType = 4;
pub const VTERM_N_VALUETYPES: VTermValueType = 5;
/// The libvterm version this emulator was ported from.
pub const VTERM_VERSION_MAJOR: c_int = 0;
pub const VTERM_VERSION_MINOR: c_int = 3;
/// The emulator's key names, from libvterm's `VTermKey`.
pub const VTERM_KEY_NONE: VTermKey = 0;
pub const VTERM_KEY_ENTER: VTermKey = 1;
pub const VTERM_KEY_TAB: VTermKey = 2;
pub const VTERM_KEY_BACKSPACE: VTermKey = 3;
pub const VTERM_KEY_ESCAPE: VTermKey = 4;
pub const VTERM_KEY_UP: VTermKey = 5;
pub const VTERM_KEY_DOWN: VTermKey = 6;
pub const VTERM_KEY_LEFT: VTermKey = 7;
pub const VTERM_KEY_RIGHT: VTermKey = 8;
pub const VTERM_KEY_INS: VTermKey = 9;
pub const VTERM_KEY_DEL: VTermKey = 10;
pub const VTERM_KEY_HOME: VTermKey = 11;
pub const VTERM_KEY_END: VTermKey = 12;
pub const VTERM_KEY_PAGEUP: VTermKey = 13;
pub const VTERM_KEY_PAGEDOWN: VTermKey = 14;
pub const VTERM_KEY_FUNCTION_0: VTermKey = 256;
pub const VTERM_KEY_FUNCTION_MAX: VTermKey = 511;
pub const VTERM_KEY_KP_0: VTermKey = 512;
pub const VTERM_KEY_KP_1: VTermKey = 513;
pub const VTERM_KEY_KP_2: VTermKey = 514;
pub const VTERM_KEY_KP_3: VTermKey = 515;
pub const VTERM_KEY_KP_4: VTermKey = 516;
pub const VTERM_KEY_KP_5: VTermKey = 517;
pub const VTERM_KEY_KP_6: VTermKey = 518;
pub const VTERM_KEY_KP_7: VTermKey = 519;
pub const VTERM_KEY_KP_8: VTermKey = 520;
pub const VTERM_KEY_KP_9: VTermKey = 521;
pub const VTERM_KEY_KP_MULT: VTermKey = 522;
pub const VTERM_KEY_KP_PLUS: VTermKey = 523;
pub const VTERM_KEY_KP_MINUS: VTermKey = 525;
pub const VTERM_KEY_KP_PERIOD: VTermKey = 526;
pub const VTERM_KEY_KP_DIVIDE: VTermKey = 527;
pub const VTERM_KEY_KP_ENTER: VTermKey = 528;
/// The modifier bits a key or mouse event carries.
pub const VTERM_MOD_NONE: VTermModifier = 0;
pub const VTERM_MOD_SHIFT: VTermModifier = 1;
pub const VTERM_MOD_ALT: VTermModifier = 2;
pub const VTERM_MOD_CTRL: VTermModifier = 4;
/// How a `VTermColor` is to be read, and which default it stands for.
///
/// Bit 0 of the type byte says which representation is live. Bits 1-2 mark
/// a colour as *being* the terminal's default foreground or background,
/// which keeps it out of an SGR report entirely.
pub const VTERM_COLOR_RGB: ::core::ffi::c_uint = 0;
pub const VTERM_COLOR_INDEXED: ::core::ffi::c_uint = 1;
pub const VTERM_COLOR_TYPE_MASK: ::core::ffi::c_uint = 1;
pub const VTERM_COLOR_DEFAULT_FG: ::core::ffi::c_uint = 2;
pub const VTERM_COLOR_DEFAULT_BG: ::core::ffi::c_uint = 4;
pub const VTERM_COLOR_DEFAULT_MASK: c_uint = 6;
/// The underline styles a cell's pen can ask for.
pub const VTERM_UNDERLINE_OFF: c_uint = 0;
pub const VTERM_UNDERLINE_SINGLE: c_uint = 1;
pub const VTERM_UNDERLINE_DOUBLE: c_uint = 2;
pub const VTERM_UNDERLINE_CURLY: c_uint = 3;
/// Superscript/subscript positioning of a cell's glyph.
pub const VTERM_BASELINE_NORMAL: c_uint = 0;
pub const VTERM_BASELINE_RAISE: c_uint = 1;
pub const VTERM_BASELINE_LOWER: c_uint = 2;
/// The values the cursor-shape and mouse termprops take.
pub const VTERM_PROP_MOUSE_NONE: c_int = 0;
pub const VTERM_PROP_CURSORSHAPE_BLOCK: c_int = 1;
pub const VTERM_PROP_MOUSE_CLICK: c_int = 1;
pub const VTERM_PROP_CURSORSHAPE_UNDERLINE: c_int = 2;
pub const VTERM_PROP_MOUSE_DRAG: c_int = 2;
pub const VTERM_PROP_CURSORSHAPE_BAR_LEFT: c_int = 3;
pub const VTERM_PROP_MOUSE_MOVE: c_int = 3;

/// How much room the reply buffer and the text decoder's scratch get.
const VTERM_BUFFER_SIZE: usize = 4096;

/// A Rust consumer's reply sink.
pub(crate) type ReplySink = Box<dyn FnMut(&[u8])>;

/// Where replies to the host go.
pub(crate) enum OutputSink {
    /// Nowhere yet: they collect in the terminal's own buffer until it is
    /// full, and a reply that does not fit is dropped whole.
    Buffer,
    /// A C consumer's callback.
    Foreign(ForeignOutput),
    /// A Rust consumer's.
    Rust(ReplySink),
}

/// The terminal's reply channel: what keys, mouse reports and the state
/// machine's answers are written to.
pub(crate) struct Output {
    sink: OutputSink,
    buffer: Vec<u8>,
}

impl Output {
    /// Writes a reply back to the host.
    pub(crate) fn push(&mut self, bytes: &[u8]) {
        match &mut self.sink {
            OutputSink::Foreign(sink) => sink.write(bytes),
            OutputSink::Rust(sink) => sink(bytes),
            OutputSink::Buffer => {
                if bytes.len() <= VTERM_BUFFER_SIZE - self.buffer.len() {
                    self.buffer.extend_from_slice(bytes);
                }
            }
        }
    }
}

/// Who the parser reports to.
pub(crate) enum ParserTarget {
    None,
    /// The terminal's own state machine, once it has one.
    State,
    /// A C consumer's callback table.
    Foreign(ForeignParser),
}

/// What a terminal owns that C does not read.
pub(crate) struct VTermInner {
    pub(crate) output: Output,
    /// Where the text decoder writes codepoints, sized as libvterm's scratch.
    scratch: Box<[u32]>,
    pub(crate) state: Option<Box<VTermState>>,
    pub(crate) screen: Option<Box<VTermScreen>>,
    pub(crate) parser_target: ParserTarget,
}

/// The state machine's view of `inner`, borrowing the mode bits beside it:
/// what every entry point that drives the state machine builds first.
pub(crate) fn state_cx<'a>(
    mode: &'a mut VTerm_mode,
    inner: &'a mut VTermInner,
) -> Option<StateCx<'a>> {
    let VTermInner {
        output,
        scratch,
        state,
        screen,
        ..
    } = inner;
    let state = state.as_deref_mut()?;
    Some(StateCx::new(
        state,
        mode,
        output,
        scratch,
        screen.as_deref_mut(),
    ))
}

impl VTerm {
    /// A terminal `rows` by `cols`, with no state machine or screen yet --
    /// those are built on first ask by [`VTerm::obtain_state`] and
    /// [`VTerm::obtain_screen`].
    pub(crate) fn new(rows: c_int, cols: c_int) -> Box<VTerm> {
        Box::new(VTerm {
            rows,
            cols,
            mode: VTerm_mode {
                utf8_ctrl8bit: [0; 1],
                _pad: [0; 3],
            },
            parser: VTerm_parser {
                state: NORMAL,
                in_esc: [0; 1],
                _pad: [0; 3],
                intermedlen: 0,
                intermed: [0; 16],
                v: ParserScratch {
                    leaderlen: 0,
                    leader: [0; 16],
                    argi: 0,
                    args: [0; 32],
                },
                callbacks: core::ptr::null(),
                cbdata: core::ptr::null_mut(),
                string_initial: false,
                emit_nul: false,
            },
            inner: Box::new(VTermInner {
                output: Output {
                    sink: OutputSink::Buffer,
                    buffer: Vec::with_capacity(VTERM_BUFFER_SIZE),
                },
                scratch: vec![0; VTERM_BUFFER_SIZE / size_of::<u32>()].into_boxed_slice(),
                state: None,
                screen: None,
                parser_target: ParserTarget::None,
            }),
        })
    }

    /// The terminal's size, rows first.
    pub(crate) fn size(&self) -> (c_int, c_int) {
        (self.rows, self.cols)
    }

    /// Resizes the terminal, telling the parser's consumer so that it can
    /// move the cursor and the screen contents. A degenerate size is refused.
    pub(crate) fn set_size(&mut self, rows: c_int, cols: c_int) {
        if rows < 1 || cols < 1 {
            return;
        }
        self.rows = rows;
        self.cols = cols;
        match &self.inner.parser_target {
            ParserTarget::None => {}
            ParserTarget::Foreign(foreign) => {
                let _ = { *foreign }.resize(rows, cols);
            }
            ParserTarget::State => {
                if let Some(mut state) = state_cx(&mut self.mode, &mut self.inner) {
                    let _ = state.resize(rows, cols);
                }
            }
        }
    }

    /// Selects whether input bytes are decoded as UTF-8.
    pub(crate) fn set_utf8(&mut self, utf8: bool) {
        self.mode.set_utf8(c_uint::from(utf8));
    }

    /// Installs the sink replies are written to.
    pub(crate) fn set_output(&mut self, sink: OutputSink) {
        self.inner.output.sink = sink;
    }

    /// Installs a C consumer's parser callbacks in place of the state
    /// machine's.
    pub(crate) fn set_parser_target(&mut self, target: ParserTarget) {
        self.inner.parser_target = target;
    }

    /// Feeds bytes of the child's output to the parser, which reports what it
    /// recognises to its consumer.
    pub(crate) fn input_write(&mut self, bytes: &[u8]) {
        let VTerm {
            mode,
            parser,
            inner,
            ..
        } = self;
        let utf8 = mode.utf8() != 0;
        match &inner.parser_target {
            ParserTarget::None => parser::write(parser, utf8, bytes, &mut NoParserHandler),
            ParserTarget::Foreign(foreign) => {
                let mut foreign = *foreign;
                parser::write(parser, utf8, bytes, &mut foreign);
            }
            ParserTarget::State => match state_cx(mode, inner) {
                Some(mut state) => parser::write(parser, utf8, bytes, &mut state),
                None => parser::write(parser, utf8, bytes, &mut NoParserHandler),
            },
        }
    }

    /// The terminal's state machine, created and wired to the parser on
    /// first ask.
    pub(crate) fn obtain_state(&mut self) -> &mut VTermState {
        if self.inner.state.is_none() {
            let back = core::ptr::from_mut(self);
            let state = VTermState::new(self.rows, self.cols, back);
            self.inner.state = Some(Box::new(state));
            self.inner.parser_target = ParserTarget::State;
            self.parser.callbacks = core::ptr::null();
            self.parser.cbdata = core::ptr::null_mut();
        }
        self.inner
            .state
            .as_deref_mut()
            .expect("the state machine was just made")
    }

    /// The terminal's screen, created on first ask and installed as its state
    /// machine's consumer.
    pub(crate) fn obtain_screen(&mut self) -> &mut VTermScreen {
        if self.inner.screen.is_none() {
            let back = core::ptr::from_mut(self);
            let state = core::ptr::from_mut(self.obtain_state());
            let screen = VTermScreen::new(self.rows, self.cols, back, state);
            self.inner.screen = Some(Box::new(screen));
            self.obtain_state().attach_screen();
        }
        self.inner
            .screen
            .as_deref_mut()
            .expect("the screen was just made")
    }

    /// The state machine's view of this terminal, once it has a state
    /// machine.
    pub(crate) fn state_cx(&mut self) -> Option<StateCx<'_>> {
        state_cx(&mut self.mode, &mut self.inner)
    }

    /// The terminal's screen, if it has one.
    pub(crate) fn screen_mut(&mut self) -> Option<&mut VTermScreen> {
        self.inner.screen.as_deref_mut()
    }

    /// Writes a reply back to the host.
    pub(crate) fn push_output(&mut self, bytes: &[u8]) {
        self.inner.output.push(bytes);
    }

    /// Writes a reply built by an encoder, if it built one.
    pub(crate) fn send(&mut self, report: Option<EscapeSeq>) {
        if let Some(bytes) = report.as_ref().and_then(EscapeSeq::finish) {
            self.push_output(bytes);
        }
    }

    /// Returns the screen and the state machine to their power-on settings,
    /// repainting through the screen.
    pub(crate) fn screen_reset(&mut self, hard: bool) {
        if let Some(screen) = self.screen_mut() {
            screen.discard_damage();
        }
        if let Some(mut state) = self.state_cx() {
            mode::reset(&mut state, hard);
        }
        if let Some(screen) = self.screen_mut() {
            screen.flush_damage();
        }
    }
}

// ------------------------------------------------- the front for the editor

impl VTerm {
    /// Allocates the screen's alternate buffer, at the terminal's size.
    pub(crate) fn screen_enable_altscreen(&mut self) {
        let (rows, cols) = self.size();
        if let Some(screen) = self.screen_mut() {
            screen.enable_altscreen(rows, cols);
        }
    }

    /// Installs what the screen reports to.
    pub(crate) fn set_screen_host(&mut self, host: Box<dyn ScreenHost>) {
        self.obtain_screen().set_host(Some(host));
    }

    /// Installs what the state machine offers the sequences it does not
    /// recognise to.
    pub(crate) fn set_fallbacks(&mut self, fallbacks: Box<dyn Fallbacks>) {
        self.obtain_state().set_fallbacks(Some(fallbacks));
    }

    /// Installs OSC 52 handling, decoding through a staging buffer of
    /// `buflen` bytes.
    pub(crate) fn set_selection_host(&mut self, host: Box<dyn SelectionHost>, buflen: usize) {
        self.obtain_state().set_selection(Some(host), buflen);
    }

    /// Hands the screen's host everything held back by the merge level.
    pub(crate) fn flush_damage(&mut self) {
        if let Some(screen) = self.screen_mut() {
            screen.flush_damage();
        }
    }

    /// Changes how the screen batches damage.
    pub(crate) fn set_damage_merge(&mut self, size: VTermDamageSize) {
        if let Some(screen) = self.screen_mut() {
            screen.set_damage_merge(size);
        }
    }

    /// The reported form of the screen's cell at `pos`, into `out`; false,
    /// leaving `out` alone, outside the screen.
    pub(crate) fn get_cell(&self, pos: VTermPos, out: &mut VTermScreenCell) -> bool {
        self.inner
            .screen
            .as_deref()
            .is_some_and(|screen| screen.export_cell(pos, out) != 0)
    }

    /// Repaints one of the sixteen ANSI palette slots.
    pub(crate) fn set_palette_color(&mut self, index: c_int, color: &VTermColor) {
        set_palette_color(self.obtain_state(), index, color);
    }

    /// `color` resolved against the palette to a literal colour.
    pub(crate) fn rgb_of(&mut self, mut color: VTermColor) -> VTermColor {
        convert_color_to_rgb(self.obtain_state(), &mut color);
        color
    }

    /// Applies a terminal property; false if the screen refused it.
    pub(crate) fn set_termprop(&mut self, prop: VTermProp, val: Value<'_>) -> bool {
        self.obtain_state();
        self.state_cx()
            .is_some_and(|mut state| state.set_termprop(prop, &val))
    }

    /// Tells the child whether its window has focus, if it asked to hear.
    pub(crate) fn focus(&mut self, focused: bool) {
        if let Some(mut state) = self.state_cx() {
            state.report_focus(if focused { b'I' } else { b'O' });
        }
    }
}

/// What a `VTermValue` accompanying `attr` holds.
pub(crate) fn attr_value_type(attr: VTermAttr) -> VTermValueType {
    match attr {
        VTERM_ATTR_UNDERLINE | VTERM_ATTR_FONT | VTERM_ATTR_BASELINE | VTERM_ATTR_URI => {
            VTERM_VALUETYPE_INT
        }
        VTERM_ATTR_FOREGROUND | VTERM_ATTR_BACKGROUND => VTERM_VALUETYPE_COLOR,
        VTERM_ATTR_BOLD | VTERM_ATTR_ITALIC | VTERM_ATTR_BLINK | VTERM_ATTR_REVERSE
        | VTERM_ATTR_CONCEAL | VTERM_ATTR_STRIKE | VTERM_ATTR_SMALL | VTERM_ATTR_DIM
        | VTERM_ATTR_OVERLINE => VTERM_VALUETYPE_BOOL,
        _ => 0,
    }
}

/// The two primitives a scroll can be driven out of, for a consumer that
/// does not take the whole scroll itself.
pub(crate) trait ScrollTarget {
    /// Moves the cells of `src` to `dest`.
    fn moverect(&mut self, dest: VTermRect, src: VTermRect);
    /// Blanks `rect`.
    fn erase(&mut self, rect: VTermRect);
}

/// The two rectangles a scroll moves between, or `None` when the shift is at
/// least as large as the rectangle and nothing survives it.
fn scroll_split(
    rect: VTermRect,
    downward: c_int,
    rightward: c_int,
) -> Option<(VTermRect, VTermRect)> {
    if downward.abs() >= rect.end_row - rect.start_row
        || rightward.abs() >= rect.end_col - rect.start_col
    {
        return None;
    }
    let (dest_cols, src_cols) = if rightward >= 0 {
        (
            rect.start_col..rect.end_col - rightward,
            rect.start_col + rightward..rect.end_col,
        )
    } else {
        (
            rect.start_col - rightward..rect.end_col,
            rect.start_col..rect.end_col + rightward,
        )
    };
    let (dest_rows, src_rows) = if downward >= 0 {
        (
            rect.start_row..rect.end_row - downward,
            rect.start_row + downward..rect.end_row,
        )
    } else {
        (
            rect.start_row - downward..rect.end_row,
            rect.start_row..rect.end_row + downward,
        )
    };
    let dest = VTermRect {
        start_row: dest_rows.start,
        end_row: dest_rows.end,
        start_col: dest_cols.start,
        end_col: dest_cols.end,
    };
    let src = VTermRect {
        start_row: src_rows.start,
        end_row: src_rows.end,
        start_col: src_cols.start,
        end_col: src_cols.end,
    };
    Some((dest, src))
}

/// The part of `rect` the scroll leaves behind, which has to be erased.
fn scroll_vacated(mut rect: VTermRect, downward: c_int, rightward: c_int) -> VTermRect {
    if downward > 0 {
        rect.start_row = rect.end_row - downward;
    } else if downward < 0 {
        rect.end_row = rect.start_row - downward;
    }
    if rightward > 0 {
        rect.start_col = rect.end_col - rightward;
    } else if rightward < 0 {
        rect.end_col = rect.start_col - rightward;
    }
    rect
}

/// Drives a scroll out of a consumer's move and erase primitives.
pub(crate) fn scroll_rect(
    rect: VTermRect,
    downward: c_int,
    rightward: c_int,
    target: &mut impl ScrollTarget,
) {
    let Some((dest, src)) = scroll_split(rect, downward, rightward) else {
        target.erase(rect);
        return;
    };
    target.moverect(dest, src);
    target.erase(scroll_vacated(rect, downward, rightward));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(start_row: c_int, end_row: c_int, start_col: c_int, end_col: c_int) -> VTermRect {
        VTermRect {
            start_row,
            end_row,
            start_col,
            end_col,
        }
    }

    #[test]
    fn a_shift_as_large_as_the_rectangle_leaves_nothing_to_move() {
        let whole = rect(0, 4, 0, 8);
        assert_eq!(scroll_split(whole, 4, 0), None);
        assert_eq!(scroll_split(whole, -9, 0), None);
        assert_eq!(scroll_split(whole, 0, 8), None);
        assert!(scroll_split(whole, 3, 0).is_some());
    }

    #[test]
    fn scrolling_up_moves_the_lower_rows_over_the_upper_ones() {
        let (dest, src) = scroll_split(rect(0, 4, 0, 8), 1, 0).unwrap();
        assert_eq!(dest, rect(0, 3, 0, 8));
        assert_eq!(src, rect(1, 4, 0, 8));
        assert_eq!(scroll_vacated(rect(0, 4, 0, 8), 1, 0), rect(3, 4, 0, 8));
    }

    #[test]
    fn scrolling_down_and_right_moves_the_other_way() {
        let (dest, src) = scroll_split(rect(0, 4, 0, 8), -1, -2).unwrap();
        assert_eq!(dest, rect(1, 4, 2, 8));
        assert_eq!(src, rect(0, 3, 0, 6));
        assert_eq!(scroll_vacated(rect(0, 4, 0, 8), -1, -2), rect(0, 1, 0, 2));
    }

    #[test]
    fn every_attribute_reports_the_kind_of_value_it_carries() {
        assert_eq!(attr_value_type(VTERM_ATTR_BOLD), VTERM_VALUETYPE_BOOL);
        assert_eq!(attr_value_type(VTERM_ATTR_FONT), VTERM_VALUETYPE_INT);
        assert_eq!(
            attr_value_type(VTERM_ATTR_FOREGROUND),
            VTERM_VALUETYPE_COLOR
        );
        // Past the last attribute there is no value at all.
        assert_eq!(attr_value_type(VTERM_N_ATTRS), 0);
        assert_eq!(attr_value_type(0), 0);
    }
}
