//! The state machine's construction, its parser events and its terminal
//! properties.
//!
//! The parser reports to a [`StateCx`] as its [`ParserHandler`]: printable
//! text through the character-set decoders, controls, escapes, control
//! sequences and control strings to the sibling modules that act on them. An
//! unrecognised sequence goes out to the consumer's fallbacks, which may want
//! the state machine back -- see [`StateCx::with_fallbacks`].
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

use crate::narrow::as_index;
use crate::narrow::len_as_int;
use core::ffi::c_int;

use crate::mbyte::GraphemeState;
use crate::vterm::encoding::{self, Charset};
use crate::vterm::host::{Csi, Fragment, ParserHandler, Value};
use crate::vterm::internal::{
    VTermEncodingInstance, VTermKeyEncodingFlags, VTermKeyEncodingStack, VTermPen,
};
use crate::vterm::pen::init_pen;
use crate::vterm::types::{
    VTerm, VTermPos, VTermProp, VTermRect, VTermState, VTermState_mode, VTermState_saved,
    VTermState_saved_mode, VTermState_selection, VTermState_tmp, VTermState_tmp_selection,
};
use crate::vterm::vterm::{
    MOUSE_X10, VTERM_PROP_ALTSCREEN, VTERM_PROP_CURSORBLINK, VTERM_PROP_CURSORSHAPE,
    VTERM_PROP_CURSORVISIBLE, VTERM_PROP_FOCUSREPORT, VTERM_PROP_ICONNAME, VTERM_PROP_MOUSE,
    VTERM_PROP_MOUSE_DRAG, VTERM_PROP_MOUSE_MOVE, VTERM_PROP_REVERSE, VTERM_PROP_SYNCOUTPUT,
    VTERM_PROP_THEMEUPDATES, VTERM_PROP_TITLE,
};
use crate::vterm::{csi, dcs, selection, text};

use super::{MOUSE_WANT_CLICK, MOUSE_WANT_DRAG, MOUSE_WANT_MOVE, StateCx, StateOwned};

/// A leader or intermediate run, of which only a single byte means anything
/// here, so two bytes decide it.
fn two_bytes(bytes: Option<&core::ffi::CStr>) -> [u8; 2] {
    match bytes.map(core::ffi::CStr::to_bytes) {
        None | Some([]) => [0, 0],
        Some([only]) => [*only, 0],
        Some([first, second, ..]) => [*first, *second],
    }
}

/// A pen with every attribute off and both colours unset.
const fn blank_pen() -> VTermPen {
    VTermPen {
        fg: crate::vterm::color::UNSET,
        bg: crate::vterm::color::UNSET,
        uri: 0,
        bold_underline_italic_blink_reverse_conceal_strike_font_small_baseline_dim_overline: [0; 3],
        _pad: [0; 1],
    }
}

// ------------------------------------------------------------------ lifetime

impl VTermState {
    /// A state machine for a terminal `rows` by `cols`, with its power-on
    /// settings. `vt` is the terminal it belongs to, kept for the C entry
    /// points that start from a state pointer.
    pub(crate) fn new(rows: c_int, cols: c_int, vt: *mut VTerm) -> Self {
        let no_flags = VTermKeyEncodingFlags {
            disambiguate_report_events_report_alternate_report_all_keys_report_associated: [0; 1],
        };
        let no_charset = VTermEncodingInstance {
            enc: None,
            data: [0; 16],
        };
        let mut state = VTermState {
            vt,
            callbacks: core::ptr::null(),
            cbdata: core::ptr::null_mut(),
            fallbacks: core::ptr::null(),
            fbdata: core::ptr::null_mut(),
            rows,
            cols,
            pos: VTermPos { row: 0, col: 0 },
            at_phantom: 0,
            scrollregion_top: 0,
            scrollregion_bottom: 0,
            scrollregion_left: 0,
            scrollregion_right: 0,
            owned: Box::new(StateOwned::new(rows, cols)),
            _lineinfo_slots: [0; 3],
            mouse_col: 0,
            mouse_row: 0,
            mouse_buttons: 0,
            mouse_flags: 0,
            mouse_protocol: MOUSE_X10,
            grapheme_buf: [0; 32],
            grapheme_len: 0,
            grapheme_last: 0,
            grapheme_state: GraphemeState::default(),
            combine_width: 0,
            combine_pos: VTermPos { row: -1, col: 0 },
            mode: VTermState_mode {
                keypad_cursor_autowrap_insert_newline_cursor_visible_cursor_blink_cursor_shape_alt_screen_origin_screen_leftrightmargin_bracketpaste_report_focus_theme_updates_synchronized_output: [0; 3],
                _pad: [0; 1],
            },
            encoding: [no_charset; 4],
            encoding_utf8: VTermEncodingInstance {
                enc: encoding::lookup(encoding::ENC_UTF8, b'u'),
                data: [0; 16],
            },
            gl_set: 0,
            gr_set: 0,
            gsingle_set: 0,
            pen: blank_pen(),
            default_fg: crate::vterm::color::UNSET,
            default_bg: crate::vterm::color::UNSET,
            colors: [crate::vterm::color::UNSET; 16],
            bold_is_highbright: 0,
            protected_cell: [0; 1],
            _pad: [0; 3],
            saved: VTermState_saved {
                pos: VTermPos { row: 0, col: 0 },
                pen: blank_pen(),
                mode: VTermState_saved_mode {
                    cursor_visible_cursor_blink_cursor_shape: [0; 1],
                    _pad: [0; 3],
                },
            },
            tmp: VTermState_tmp {
                selection: VTermState_tmp_selection {
                    mask: 0,
                    state: [0; 1],
                    _pad: [0; 1],
                    recvpartial: 0,
                    sendpartial: 0,
                },
            },
            selection: VTermState_selection {
                callbacks: core::ptr::null(),
                user: core::ptr::null_mut(),
                _buffer_slot: 0,
                buflen: 0,
            },
            key_encoding_stacks: [(); 2].map(|()| VTermKeyEncodingStack {
                items: [no_flags; 16],
                size: 1,
            }),
        };
        init_pen(&mut state);
        let utf8 = &mut state.encoding_utf8;
        if let Some(charset) = utf8.enc.map(Charset::of) {
            charset.init(&mut utf8.data);
        }
        state
    }
}

// ------------------------------------------------------- the parser's events

impl ParserHandler for StateCx<'_> {
    /// Decodes a run of input bytes through the live character set and prints
    /// the graphemes it yields. Answers how many input bytes were consumed,
    /// which is short of the run when it ends part-way through a sequence.
    fn text(&mut self, bytes: &[u8]) -> Option<c_int> {
        // A high bit selects the right-hand set, a single shift overrides
        // both, and any slot designated UTF-8 collapses onto the one shared
        // decoder so that a multi-byte sequence can span calls.
        let lead = bytes.first().copied().unwrap_or(0);
        let slot = if self.gsingle_set != 0 {
            as_index(self.gsingle_set)
        } else if lead & 0x80 == 0 {
            as_index(self.gl_set)
        } else if self.utf8() {
            usize::MAX
        } else {
            as_index(self.gr_set)
        };
        let utf8 = self.encoding_utf8.enc;
        let shared = match self.encoding.get(slot) {
            Some(instance) => same_charset(instance.enc, utf8),
            None => true,
        };
        let room = if self.gsingle_set != 0 {
            1
        } else {
            self.scratch.len()
        };

        // The decoder writes into the terminal's scratch, which is out of
        // the state machine while the glyphs it decoded are printed.
        let scratch = core::mem::take(&mut self.scratch);
        let instance = if shared {
            &mut self.encoding_utf8
        } else {
            &mut self.encoding[slot]
        };
        let enc = instance.enc.expect("character set without a decoder");
        let (got, eaten) = Charset::of(enc).decode(&mut instance.data, bytes, &mut scratch[..room]);

        // A stateful set may not have seen enough bytes for even one
        // codepoint.
        if got > 0 {
            self.gsingle_set = 0;
            text::print(self, &scratch[..got]);
        }
        self.scratch = scratch;
        Some(len_as_int(eaten))
    }

    fn control(&mut self, control: u8) -> Option<c_int> {
        if text::control(self, control) {
            return Some(1);
        }
        let taken = self.with_fallbacks(|fallbacks, _| fallbacks.control(control));
        Some(c_int::from(taken.is_some_and(|taken| taken != 0)))
    }

    fn escape(&mut self, sequence: &[u8]) -> Option<c_int> {
        Some(text::escape(self, sequence))
    }

    fn csi(&mut self, csi: &Csi<'_>) -> Option<c_int> {
        let outcome = csi::dispatch(
            self,
            two_bytes(csi.leader),
            csi.args,
            two_bytes(csi.intermed),
            csi.command,
        );
        match outcome {
            csi::Outcome::Handled => Some(1),
            csi::Outcome::Ignored => Some(0),
            csi::Outcome::Unrecognised => {
                let taken = self.with_fallbacks(|fallbacks, _| fallbacks.csi(csi));
                Some(c_int::from(taken.is_some_and(|taken| taken != 0)))
            }
        }
    }

    /// Upstream offers every OSC to the fallback, even the ones it handled
    /// itself, and reports only what the fallback made of it.
    fn osc(&mut self, command: c_int, frag: Fragment<'_>) -> Option<c_int> {
        selection::osc(self, command, frag);
        let taken = self.with_fallbacks(|fallbacks, state| fallbacks.osc(command, frag, state));
        Some(c_int::from(taken.is_some_and(|taken| taken != 0)))
    }

    fn dcs(&mut self, command: &[u8], frag: Fragment<'_>) -> Option<c_int> {
        if dcs::device_control(self, command, frag) {
            return Some(1);
        }
        let taken = self.with_fallbacks(|fallbacks, _| fallbacks.dcs(command, frag));
        Some(c_int::from(taken.is_some_and(|taken| taken != 0)))
    }

    // APC, PM and SOS carry nothing this terminal understands, so each only
    // reaches for its own fallback.
    fn apc(&mut self, frag: Fragment<'_>) -> Option<c_int> {
        let taken = self.with_fallbacks(|fallbacks, _| fallbacks.apc(frag));
        Some(c_int::from(taken.is_some_and(|taken| taken != 0)))
    }

    fn pm(&mut self, frag: Fragment<'_>) -> Option<c_int> {
        let taken = self.with_fallbacks(|fallbacks, _| fallbacks.pm(frag));
        Some(c_int::from(taken.is_some_and(|taken| taken != 0)))
    }

    fn sos(&mut self, frag: Fragment<'_>) -> Option<c_int> {
        let taken = self.with_fallbacks(|fallbacks, _| fallbacks.sos(frag));
        Some(c_int::from(taken.is_some_and(|taken| taken != 0)))
    }

    fn resize(&mut self, rows: c_int, cols: c_int) -> Option<c_int> {
        StateCx::resize(self, rows, cols)
    }
}

/// Whether two slots name the same character set: the same table, as
/// libvterm compared them.
fn same_charset(
    a: Option<core::ptr::NonNull<crate::vterm::internal::VTermEncoding>>,
    b: Option<core::ptr::NonNull<crate::vterm::internal::VTermEncoding>>,
) -> bool {
    a == b
}

// ------------------------------------------------------ terminal properties

impl StateCx<'_> {
    /// Applies a terminal property, offering it to the consumer first, which
    /// may refuse it. Refusal matters most for the alternate screen: the
    /// state must not believe it switched if the screen did not.
    pub(crate) fn set_termprop(&mut self, prop: VTermProp, val: &Value<'_>) -> bool {
        if !self.offer_termprop(prop, val) {
            return false;
        }
        let boolean = val.int().cast_unsigned();
        let number = val.int();
        match prop {
            // Titles are passed straight through, never stored.
            VTERM_PROP_TITLE | VTERM_PROP_ICONNAME => {}
            VTERM_PROP_CURSORVISIBLE => self.mode.set_cursor_visible(boolean),
            VTERM_PROP_CURSORBLINK => self.mode.set_cursor_blink(boolean),
            VTERM_PROP_CURSORSHAPE => self.mode.set_cursor_shape(number.cast_unsigned()),
            VTERM_PROP_REVERSE => self.mode.set_screen(boolean),
            VTERM_PROP_ALTSCREEN => {
                self.mode.set_alt_screen(boolean);
                self.follow_active_buffer();
                if self.mode.alt_screen() != 0 {
                    let rect = VTermRect {
                        start_row: 0,
                        end_row: self.rows,
                        start_col: 0,
                        end_col: self.cols,
                    };
                    self.erase(rect, false);
                }
            }
            VTERM_PROP_MOUSE => {
                self.mouse_flags = 0;
                if number != 0 {
                    self.mouse_flags |= MOUSE_WANT_CLICK;
                }
                if number == VTERM_PROP_MOUSE_DRAG {
                    self.mouse_flags |= MOUSE_WANT_DRAG;
                }
                if number == VTERM_PROP_MOUSE_MOVE {
                    self.mouse_flags |= MOUSE_WANT_MOVE;
                }
            }
            VTERM_PROP_FOCUSREPORT => self.mode.set_report_focus(boolean),
            VTERM_PROP_THEMEUPDATES => self.mode.set_theme_updates(boolean),
            VTERM_PROP_SYNCOUTPUT => self.mode.set_synchronized_output(boolean),
            _ => return false,
        }
        true
    }
}
