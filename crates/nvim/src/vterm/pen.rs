//! The terminal's pen: the graphic rendition every glyph is stamped with.
//!
//! Three jobs live here. Applying an SGR control sequence to the pen
//! ([`apply_sgr`]), reporting the pen back as SGR parameters
//! ([`pen_sgr_params`]) for a DECRQSS query, and resolving a pen colour
//! against the terminal's palette ([`convert_color_to_rgb`]).
//!
//! Every change is echoed to the state's `setpenattr` callback, which is a
//! raw function pointer, so the mutating entry points pass that obligation
//! on to their callers. Everything the callback is not involved in — colour
//! arithmetic, parameter encoding — is a plain safe function.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_int, c_long, c_uint, c_void};

use crate::types::{
    VTermAttr, VTermColor, VTermColor_rgb, VTermPen, VTermState, VTermStateCallbacks, VTermValue,
    VTermValueType,
};
use crate::vterm::color::{ColorValue, ansi_default, fixed_palette};
use crate::vterm::vterm::{
    VTERM_ATTR_BACKGROUND, VTERM_ATTR_BASELINE, VTERM_ATTR_BLINK, VTERM_ATTR_BOLD,
    VTERM_ATTR_CONCEAL, VTERM_ATTR_DIM, VTERM_ATTR_FONT, VTERM_ATTR_FOREGROUND, VTERM_ATTR_ITALIC,
    VTERM_ATTR_OVERLINE, VTERM_ATTR_REVERSE, VTERM_ATTR_SMALL, VTERM_ATTR_STRIKE,
    VTERM_ATTR_UNDERLINE, VTERM_ATTR_URI, VTERM_BASELINE_LOWER, VTERM_BASELINE_NORMAL,
    VTERM_BASELINE_RAISE, VTERM_COLOR_DEFAULT_BG, VTERM_COLOR_DEFAULT_FG, VTERM_COLOR_DEFAULT_MASK,
    VTERM_COLOR_INDEXED, VTERM_COLOR_RGB, VTERM_COLOR_TYPE_MASK, VTERM_UNDERLINE_CURLY,
    VTERM_UNDERLINE_DOUBLE, VTERM_UNDERLINE_OFF, VTERM_UNDERLINE_SINGLE, vterm_get_attr_type,
};

/// Set on a control-sequence parameter that is followed by a sub-parameter,
/// i.e. one written `38:5:1` rather than `38;5;1`.
pub const CSI_ARG_FLAG_MORE: c_uint = (1 as c_uint) << 31;
/// The value bits of a control-sequence parameter, below the flag.
pub const CSI_ARG_MASK: c_uint = !((1 as c_uint) << 31);
/// The value the parser leaves for an omitted parameter.
pub const CSI_ARG_MISSING: c_long = 2147483647;

// ---------------------------------------------------------------- colours

/// Splits a colour into its type byte and the payload that byte selects.
fn color_parts(col: &VTermColor) -> (c_uint, ColorValue) {
    // SAFETY: the type byte overlaps the first byte of every arm of the
    // union, so it is initialised whichever arm was written last.
    let flags = c_uint::from(unsafe { col.type_0 });
    // The type bit names the only arm that may be read: the indexed arm is
    // two bytes where the RGB arm is four, so reading the wrong one would
    // read past what was written.
    let value = if flags & VTERM_COLOR_TYPE_MASK == VTERM_COLOR_INDEXED {
        // SAFETY: the type byte says the indexed arm is the live one.
        ColorValue::Indexed(unsafe { col.indexed.idx })
    } else {
        // SAFETY: the type byte says the RGB arm is the live one.
        let rgb = unsafe { col.rgb };
        ColorValue::Rgb {
            red: rgb.red,
            green: rgb.green,
            blue: rgb.blue,
        }
    };
    (flags, value)
}

/// Rebuilds a colour from a type byte and a payload. The payload owns the
/// type bit; the marks above it ride through unchanged.
///
/// Both payloads are written through the RGB arm so the whole union ends up
/// initialised — an indexed colour's index shares its byte with the red
/// channel, and upstream left the two bytes past it as whatever was there.
fn make_color(flags: c_uint, value: ColorValue) -> VTermColor {
    let marks = (flags & !VTERM_COLOR_TYPE_MASK) as u8;
    let (kind, red, green, blue) = match value {
        ColorValue::Rgb { red, green, blue } => (VTERM_COLOR_RGB as u8, red, green, blue),
        ColorValue::Indexed(idx) => (VTERM_COLOR_INDEXED as u8, idx, 0, 0),
    };
    VTermColor {
        rgb: VTermColor_rgb {
            type_0: marks | kind,
            red,
            green,
            blue,
        },
    }
}

/// A colour carrying no default-foreground/background mark.
fn plain_color(value: ColorValue) -> VTermColor {
    make_color(0, value)
}

/// Stamps a colour as *being* the terminal's default foreground or
/// background, replacing any mark it already carried.
fn mark_default(col: VTermColor, mark: c_uint) -> VTermColor {
    let (flags, value) = color_parts(&col);
    make_color((flags & !VTERM_COLOR_DEFAULT_MASK) | mark, value)
}

/// The colour palette slot `index` resolves to: the state's repaintable ANSI
/// slots below 16, the fixed cube and grey ramp above, nothing outside.
fn palette_color(state: &VTermState, index: c_long) -> Option<VTermColor> {
    if (0..16).contains(&index) {
        return Some(state.colors[index as usize]);
    }
    fixed_palette(index).map(plain_color)
}

/// Resolves `col` against the palette and clears every mark but the type
/// bit, so that afterwards it is a literal colour.
///
/// A repainted ANSI slot that itself holds an indexed colour stays indexed:
/// upstream resolves one step, not the whole chain, and so does this.
pub fn convert_color_to_rgb(state: &VTermState, col: &mut VTermColor) {
    let (flags, value) = color_parts(col);
    if flags & VTERM_COLOR_TYPE_MASK == VTERM_COLOR_INDEXED
        && let ColorValue::Indexed(idx) = value
        && let Some(resolved) = palette_color(state, c_long::from(idx))
    {
        *col = resolved;
    }
    let (_, value) = color_parts(col);
    *col = plain_color(value);
}

/// Repaints one of the sixteen ANSI palette slots. Indices outside them are
/// ignored.
pub fn set_palette_color(state: &mut VTermState, index: c_int, col: &VTermColor) {
    if (0..16).contains(&index) {
        state.colors[index as usize] = *col;
    }
}

/// Gives a fresh terminal its default colours: a 90% grey foreground, so
/// that a pure white still reads as brighter, on black, plus the built-in
/// ANSI palette.
pub fn init_pen(state: &mut VTermState) {
    state.default_fg = mark_default(
        plain_color(ColorValue::rgb(240, 240, 240)),
        VTERM_COLOR_DEFAULT_FG,
    );
    state.default_bg = mark_default(
        plain_color(ColorValue::rgb(0, 0, 0)),
        VTERM_COLOR_DEFAULT_BG,
    );
    for slot in 0..state.colors.len() {
        if let Some(value) = ansi_default(slot as c_long) {
            state.colors[slot] = plain_color(value);
        }
    }
}

// ------------------------------------------------------- reporting changes

/// Where a pen change is reported to.
///
/// Captured before the change is made so that no borrow of the state is held
/// across the callback, which is free to re-enter the terminal. Holding one
/// *is* the promise that the callback table is live, which is why every
/// report through it is then ordinary code and only [`PenSink::of`] is not.
#[derive(Copy, Clone)]
struct PenSink {
    callbacks: *const VTermStateCallbacks,
    cbdata: *mut c_void,
}

impl PenSink {
    /// The sink `state` reports its pen changes to.
    ///
    /// # Safety
    ///
    /// The state's callback table and its data pointer must stay live for as
    /// long as the returned sink is used.
    unsafe fn of(state: &VTermState) -> Self {
        PenSink {
            callbacks: state.callbacks,
            cbdata: state.cbdata,
        }
    }

    /// Reports one attribute's new value.
    fn set(self, attr: VTermAttr, val: VTermValue) {
        // SAFETY: constructing the sink promised the table is still live, so
        // it is either null or a readable `VTermStateCallbacks`.
        let Some(callbacks) = (unsafe { self.callbacks.as_ref() }) else {
            return;
        };
        let Some(setpenattr) = callbacks.setpenattr else {
            return;
        };
        let mut val = val;
        // SAFETY: the consumer's own callback, taking the attribute, a
        // pointer to a value that outlives the call, and the data it was
        // registered with.
        unsafe { setpenattr(attr, &raw mut val, self.cbdata) };
    }
}

/// The boolean form of an attribute value.
fn flag(on: bool) -> VTermValue {
    VTermValue {
        boolean: c_int::from(on),
    }
}

/// The integer form of an attribute value.
fn number(n: c_uint) -> VTermValue {
    VTermValue { number: n as c_int }
}

/// The colour form of an attribute value.
fn color(col: VTermColor) -> VTermValue {
    VTermValue { color: col }
}

// ------------------------------------------------------------ pen changes

/// Returns the pen to its defaults, as SGR 0 does.
///
/// # Safety
///
/// The state's callback table must still be live.
pub unsafe fn reset_pen(state: &mut VTermState) {
    // SAFETY: forwarded to this function's own caller.
    let sink = unsafe { PenSink::of(state) };
    reset_pen_to(state, sink);
}

/// [`reset_pen`] against a sink already in hand.
fn reset_pen_to(state: &mut VTermState, sink: PenSink) {
    state.pen.set_bold(0);
    sink.set(VTERM_ATTR_BOLD, flag(false));
    state.pen.set_underline(VTERM_UNDERLINE_OFF);
    sink.set(VTERM_ATTR_UNDERLINE, number(0));
    state.pen.set_italic(0);
    sink.set(VTERM_ATTR_ITALIC, flag(false));
    state.pen.set_blink(0);
    sink.set(VTERM_ATTR_BLINK, flag(false));
    state.pen.set_reverse(0);
    sink.set(VTERM_ATTR_REVERSE, flag(false));
    state.pen.set_conceal(0);
    sink.set(VTERM_ATTR_CONCEAL, flag(false));
    state.pen.set_strike(0);
    sink.set(VTERM_ATTR_STRIKE, flag(false));
    state.pen.set_font(0);
    sink.set(VTERM_ATTR_FONT, number(0));
    state.pen.set_small(0);
    sink.set(VTERM_ATTR_SMALL, flag(false));
    state.pen.set_baseline(VTERM_BASELINE_NORMAL);
    sink.set(VTERM_ATTR_BASELINE, number(0));
    state.pen.set_dim(0);
    sink.set(VTERM_ATTR_DIM, flag(false));
    state.pen.set_overline(0);
    sink.set(VTERM_ATTR_OVERLINE, flag(false));
    state.pen.fg = state.default_fg;
    sink.set(VTERM_ATTR_FOREGROUND, color(state.default_fg));
    state.pen.bg = state.default_bg;
    sink.set(VTERM_ATTR_BACKGROUND, color(state.default_bg));
    state.pen.uri = 0;
    sink.set(VTERM_ATTR_URI, number(0));
}

/// Stashes the pen for a later [`restore_pen`], as DECSC does.
pub fn save_pen(state: &mut VTermState) {
    state.saved.pen = state.pen;
}

/// Brings back the pen [`save_pen`] stashed, reporting every attribute of it,
/// as DECRC does.
///
/// # Safety
///
/// The state's callback table must still be live.
pub unsafe fn restore_pen(state: &mut VTermState) {
    // SAFETY: forwarded to this function's own caller.
    let sink = unsafe { PenSink::of(state) };
    state.pen = state.saved.pen;
    let pen = state.pen;
    sink.set(VTERM_ATTR_BOLD, flag(pen.bold() != 0));
    sink.set(VTERM_ATTR_UNDERLINE, number(pen.underline()));
    sink.set(VTERM_ATTR_ITALIC, flag(pen.italic() != 0));
    sink.set(VTERM_ATTR_BLINK, flag(pen.blink() != 0));
    sink.set(VTERM_ATTR_REVERSE, flag(pen.reverse() != 0));
    sink.set(VTERM_ATTR_CONCEAL, flag(pen.conceal() != 0));
    sink.set(VTERM_ATTR_STRIKE, flag(pen.strike() != 0));
    sink.set(VTERM_ATTR_FONT, number(pen.font()));
    sink.set(VTERM_ATTR_SMALL, flag(pen.small() != 0));
    sink.set(VTERM_ATTR_BASELINE, number(pen.baseline()));
    sink.set(VTERM_ATTR_DIM, flag(pen.dim() != 0));
    sink.set(VTERM_ATTR_OVERLINE, flag(pen.overline() != 0));
    sink.set(VTERM_ATTR_FOREGROUND, color(pen.fg));
    sink.set(VTERM_ATTR_BACKGROUND, color(pen.bg));
    sink.set(VTERM_ATTR_URI, number(pen.uri as c_uint));
}

/// Points the pen's foreground or background at an ANSI palette slot.
fn set_pen_col_ansi(state: &mut VTermState, sink: PenSink, attr: VTermAttr, index: c_long) {
    let col = plain_color(ColorValue::Indexed(index as u8));
    if attr == VTERM_ATTR_BACKGROUND {
        state.pen.bg = col;
    } else {
        state.pen.fg = col;
    }
    sink.set(attr, color(col));
}

/// Sets one attribute of the pen directly, bypassing SGR. Rejects a value
/// whose type does not match the attribute's.
///
/// # Safety
///
/// The state's callback table must still be live.
pub unsafe fn set_pen_attr(
    state: &mut VTermState,
    attr: VTermAttr,
    value_type: VTermValueType,
    val: &VTermValue,
) -> bool {
    if value_type != vterm_get_attr_type(attr) {
        return false;
    }
    // One reader per arm of the union, so that only the arm the attribute's
    // own value type named is ever read.
    //
    // SAFETY: `value_type` was just checked against that type, and the
    // caller promised `val` holds the arm it names.
    let boolean = || unsafe { val.boolean } as c_uint;
    // SAFETY: as above.
    let number = || unsafe { val.number };
    // SAFETY: as above.
    let color_of = || unsafe { val.color };
    match attr {
        VTERM_ATTR_BOLD => state.pen.set_bold(boolean()),
        VTERM_ATTR_UNDERLINE => state.pen.set_underline(number() as c_uint),
        VTERM_ATTR_ITALIC => state.pen.set_italic(boolean()),
        VTERM_ATTR_BLINK => state.pen.set_blink(boolean()),
        VTERM_ATTR_REVERSE => state.pen.set_reverse(boolean()),
        VTERM_ATTR_CONCEAL => state.pen.set_conceal(boolean()),
        VTERM_ATTR_STRIKE => state.pen.set_strike(boolean()),
        VTERM_ATTR_FONT => state.pen.set_font(number() as c_uint),
        VTERM_ATTR_FOREGROUND => state.pen.fg = color_of(),
        VTERM_ATTR_BACKGROUND => state.pen.bg = color_of(),
        VTERM_ATTR_SMALL => state.pen.set_small(boolean()),
        VTERM_ATTR_BASELINE => state.pen.set_baseline(number() as c_uint),
        VTERM_ATTR_URI => state.pen.uri = number(),
        VTERM_ATTR_DIM => state.pen.set_dim(boolean()),
        VTERM_ATTR_OVERLINE => state.pen.set_overline(boolean()),
        _ => return false,
    }
    // SAFETY: forwarded to this function's own caller.
    unsafe { PenSink::of(state) }.set(attr, *val);
    true
}

// -------------------------------------------------------- SGR parameters

/// Parameter `i`, or the "omitted" value past the end of the list.
///
/// Upstream indexed one past the parameter list in two places and got
/// whatever its fixed-size array still held from an earlier sequence; an
/// omitted parameter is the well-defined stand-in.
fn arg_at(args: &[c_long], i: usize) -> c_long {
    args.get(i).copied().unwrap_or(CSI_ARG_MISSING)
}

/// [`arg_at`] without the sub-parameter flag.
fn arg_value(args: &[c_long], i: usize) -> c_long {
    arg_at(args, i) & c_long::from(CSI_ARG_MASK)
}

/// Decodes the colour selector that follows SGR 38 or 48, returning the
/// colour it names and how many parameters it spans.
///
/// A malformed selector still reports the span upstream would have skipped,
/// so that the caller stays in step with the rest of the sequence.
fn parse_sgr_color(palette: c_long, args: &[c_long]) -> (Option<ColorValue>, usize) {
    match palette {
        // Three parameters holding the channels directly.
        2 => {
            let Some(channels) = args.get(..3) else {
                return (None, args.len());
            };
            let channel = |i: usize| (channels[i] & c_long::from(CSI_ARG_MASK)) as u8;
            let value = ColorValue::rgb(channel(0), channel(1), channel(2));
            (Some(value), 3)
        }
        // One parameter holding a palette index.
        5 => {
            let Some(&index) = args.first() else {
                return (None, 0);
            };
            if index & c_long::from(CSI_ARG_MASK) == CSI_ARG_MISSING {
                return (None, 1);
            }
            (Some(ColorValue::Indexed(index as u8)), 1)
        }
        _ => (None, 0),
    }
}

/// Applies an SGR (Select Graphic Rendition) control sequence to the pen,
/// reporting each attribute it touches.
///
/// # Safety
///
/// The state's callback table must still be live.
pub unsafe fn apply_sgr(state: &mut VTermState, args: &[c_long]) {
    // SAFETY: forwarded to this function's own caller.
    let sink = unsafe { PenSink::of(state) };
    let mut argi = 0;
    while argi < args.len() {
        let raw = arg_at(args, argi);
        let arg = raw & c_long::from(CSI_ARG_MASK);
        match arg {
            CSI_ARG_MISSING | 0 => reset_pen_to(state, sink),
            1 => {
                // Bold. On a terminal that conflates bold with brightness,
                // it also promotes one of the low eight palette colours.
                let (flags, value) = color_parts(&state.pen.fg);
                state.pen.set_bold(1);
                sink.set(VTERM_ATTR_BOLD, flag(true));
                if flags & VTERM_COLOR_DEFAULT_FG == 0
                    && flags & VTERM_COLOR_TYPE_MASK == VTERM_COLOR_INDEXED
                    && state.bold_is_highbright != 0
                    && let ColorValue::Indexed(idx) = value
                    && idx < 8
                {
                    set_pen_col_ansi(state, sink, VTERM_ATTR_FOREGROUND, c_long::from(idx) + 8);
                }
            }
            2 => {
                state.pen.set_dim(1);
                sink.set(VTERM_ATTR_DIM, flag(true));
            }
            3 => {
                state.pen.set_italic(1);
                sink.set(VTERM_ATTR_ITALIC, flag(true));
            }
            4 => {
                // A sub-parameter picks the underline style.
                state.pen.set_underline(VTERM_UNDERLINE_SINGLE);
                if raw & c_long::from(CSI_ARG_FLAG_MORE) != 0 {
                    argi += 1;
                    match arg_value(args, argi) {
                        0 => state.pen.set_underline(VTERM_UNDERLINE_OFF),
                        1 => state.pen.set_underline(VTERM_UNDERLINE_SINGLE),
                        2 => state.pen.set_underline(VTERM_UNDERLINE_DOUBLE),
                        3 => state.pen.set_underline(VTERM_UNDERLINE_CURLY),
                        _ => {}
                    }
                }
                sink.set(VTERM_ATTR_UNDERLINE, number(state.pen.underline()));
            }
            5 => {
                state.pen.set_blink(1);
                sink.set(VTERM_ATTR_BLINK, flag(true));
            }
            7 => {
                state.pen.set_reverse(1);
                sink.set(VTERM_ATTR_REVERSE, flag(true));
            }
            8 => {
                state.pen.set_conceal(1);
                sink.set(VTERM_ATTR_CONCEAL, flag(true));
            }
            9 => {
                state.pen.set_strike(1);
                sink.set(VTERM_ATTR_STRIKE, flag(true));
            }
            10..=19 => {
                state.pen.set_font((arg - 10) as c_uint);
                sink.set(VTERM_ATTR_FONT, number(state.pen.font()));
            }
            21 => {
                state.pen.set_underline(VTERM_UNDERLINE_DOUBLE);
                sink.set(VTERM_ATTR_UNDERLINE, number(state.pen.underline()));
            }
            22 => {
                state.pen.set_bold(0);
                sink.set(VTERM_ATTR_BOLD, flag(false));
                state.pen.set_dim(0);
                sink.set(VTERM_ATTR_DIM, flag(false));
            }
            23 => {
                state.pen.set_italic(0);
                sink.set(VTERM_ATTR_ITALIC, flag(false));
            }
            24 => {
                state.pen.set_underline(VTERM_UNDERLINE_OFF);
                sink.set(VTERM_ATTR_UNDERLINE, number(0));
            }
            25 => {
                state.pen.set_blink(0);
                sink.set(VTERM_ATTR_BLINK, flag(false));
            }
            27 => {
                state.pen.set_reverse(0);
                sink.set(VTERM_ATTR_REVERSE, flag(false));
            }
            28 => {
                state.pen.set_conceal(0);
                sink.set(VTERM_ATTR_CONCEAL, flag(false));
            }
            29 => {
                state.pen.set_strike(0);
                sink.set(VTERM_ATTR_STRIKE, flag(false));
            }
            30..=37 => {
                let mut slot = arg - 30;
                if state.pen.bold() != 0 && state.bold_is_highbright != 0 {
                    slot += 8;
                }
                set_pen_col_ansi(state, sink, VTERM_ATTR_FOREGROUND, slot);
            }
            38 | 48 => {
                let (attr, foreground) = if arg == 38 {
                    (VTERM_ATTR_FOREGROUND, true)
                } else {
                    (VTERM_ATTR_BACKGROUND, false)
                };
                let palette = arg_value(args, argi + 1);
                let rest = args.get(argi + 2..).unwrap_or(&[]);
                let (found, consumed) = parse_sgr_color(palette, rest);
                argi += 1 + consumed;
                if let Some(value) = found {
                    let col = plain_color(value);
                    if foreground {
                        state.pen.fg = col;
                    } else {
                        state.pen.bg = col;
                    }
                }
                let col = if foreground {
                    state.pen.fg
                } else {
                    state.pen.bg
                };
                sink.set(attr, color(col));
            }
            39 => {
                state.pen.fg = state.default_fg;
                sink.set(VTERM_ATTR_FOREGROUND, color(state.pen.fg));
            }
            40..=47 => set_pen_col_ansi(state, sink, VTERM_ATTR_BACKGROUND, arg - 40),
            49 => {
                state.pen.bg = state.default_bg;
                sink.set(VTERM_ATTR_BACKGROUND, color(state.pen.bg));
            }
            53 => {
                state.pen.set_overline(1);
                sink.set(VTERM_ATTR_OVERLINE, flag(true));
            }
            55 => {
                state.pen.set_overline(0);
                sink.set(VTERM_ATTR_OVERLINE, flag(false));
            }
            // Superscript, subscript, and off. All three share a body.
            73..=75 => {
                state.pen.set_small(c_uint::from(arg != 75));
                state.pen.set_baseline(match arg {
                    73 => VTERM_BASELINE_RAISE,
                    74 => VTERM_BASELINE_LOWER,
                    _ => VTERM_BASELINE_NORMAL,
                });
                sink.set(VTERM_ATTR_SMALL, number(state.pen.small()));
                sink.set(VTERM_ATTR_BASELINE, number(state.pen.baseline()));
            }
            90..=97 => set_pen_col_ansi(state, sink, VTERM_ATTR_FOREGROUND, arg - 90 + 8),
            100..=107 => set_pen_col_ansi(state, sink, VTERM_ATTR_BACKGROUND, arg - 100 + 8),
            _ => {}
        }
        // Step past this parameter and any sub-parameters it carried.
        while arg_at(args, argi) & c_long::from(CSI_ARG_FLAG_MORE) != 0 {
            argi += 1;
        }
        argi += 1;
    }
}

/// Appends the SGR parameters that name `col` as a foreground or background.
/// A colour that *is* the terminal default contributes nothing: the report
/// leaves it out and the receiver falls back to its own default.
fn push_color_params(col: &VTermColor, foreground: bool, out: &mut Vec<c_long>) {
    let (flags, value) = color_parts(col);
    let default_mark = if foreground {
        VTERM_COLOR_DEFAULT_FG
    } else {
        VTERM_COLOR_DEFAULT_BG
    };
    if flags & default_mark != 0 {
        return;
    }
    let more = c_long::from(CSI_ARG_FLAG_MORE);
    let selector = if foreground { 38 } else { 48 };
    match value {
        ColorValue::Indexed(idx) if idx < 8 => {
            out.push(c_long::from(idx) + if foreground { 30 } else { 40 });
        }
        ColorValue::Indexed(idx) if idx < 16 => {
            out.push(c_long::from(idx) - 8 + if foreground { 90 } else { 100 });
        }
        ColorValue::Indexed(idx) => {
            out.push(more | selector);
            out.push(more | 5);
            out.push(c_long::from(idx));
        }
        ColorValue::Rgb { red, green, blue } => {
            out.push(more | selector);
            out.push(more | 2);
            out.push(more | c_long::from(red));
            out.push(more | c_long::from(green));
            out.push(c_long::from(blue));
        }
    }
}

/// The SGR parameters that would reproduce `pen`, for a DECRQSS report.
pub fn pen_sgr_params(pen: &VTermPen) -> Vec<c_long> {
    let mut out = Vec::new();
    if pen.bold() != 0 {
        out.push(1);
    }
    if pen.dim() != 0 {
        out.push(2);
    }
    if pen.italic() != 0 {
        out.push(3);
    }
    if pen.underline() == VTERM_UNDERLINE_SINGLE {
        out.push(4);
    }
    if pen.underline() == VTERM_UNDERLINE_CURLY {
        out.push(4 | c_long::from(CSI_ARG_FLAG_MORE));
        out.push(3);
    }
    if pen.blink() != 0 {
        out.push(5);
    }
    if pen.reverse() != 0 {
        out.push(7);
    }
    if pen.conceal() != 0 {
        out.push(8);
    }
    if pen.strike() != 0 {
        out.push(9);
    }
    if pen.font() != 0 {
        out.push(10 + c_long::from(pen.font()));
    }
    if pen.underline() == VTERM_UNDERLINE_DOUBLE {
        out.push(21);
    }
    push_color_params(&pen.fg, true, &mut out);
    push_color_params(&pen.bg, false, &mut out);
    if pen.overline() != 0 {
        out.push(53);
    }
    if pen.small() != 0 {
        if pen.baseline() == VTERM_BASELINE_RAISE {
            out.push(73);
        } else if pen.baseline() == VTERM_BASELINE_LOWER {
            out.push(74);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const MORE: c_long = CSI_ARG_FLAG_MORE as c_long;

    fn blank_pen() -> VTermPen {
        VTermPen {
            fg: plain_color(ColorValue::rgb(0, 0, 0)),
            bg: plain_color(ColorValue::rgb(0, 0, 0)),
            uri: 0,
            bold_underline_italic_blink_reverse_conceal_strike_font_small_baseline_dim_overline: [0;
                3],
            c2rust_padding: [0; 1],
        }
    }

    #[test]
    fn a_colour_survives_a_round_trip_through_the_union() {
        let rgb = plain_color(ColorValue::rgb(1, 2, 3));
        assert_eq!(
            color_parts(&rgb),
            (VTERM_COLOR_RGB, ColorValue::rgb(1, 2, 3))
        );
        let indexed = plain_color(ColorValue::Indexed(200));
        assert_eq!(
            color_parts(&indexed),
            (VTERM_COLOR_INDEXED, ColorValue::Indexed(200))
        );
    }

    #[test]
    fn default_marks_replace_each_other_and_keep_the_payload() {
        let fg = mark_default(
            plain_color(ColorValue::rgb(9, 8, 7)),
            VTERM_COLOR_DEFAULT_FG,
        );
        let (flags, value) = color_parts(&fg);
        assert_eq!(flags, VTERM_COLOR_DEFAULT_FG | VTERM_COLOR_RGB);
        assert_eq!(value, ColorValue::rgb(9, 8, 7));

        let bg = mark_default(fg, VTERM_COLOR_DEFAULT_BG);
        assert_eq!(color_parts(&bg).0, VTERM_COLOR_DEFAULT_BG | VTERM_COLOR_RGB);
    }

    #[test]
    fn rgb_selector_wants_three_channels() {
        assert_eq!(
            parse_sgr_color(2, &[10, 20, 30, 40]),
            (Some(ColorValue::rgb(10, 20, 30)), 3)
        );
        // Too few: nothing is set, but the whole remainder is skipped.
        assert_eq!(parse_sgr_color(2, &[10, 20]), (None, 2));
        assert_eq!(parse_sgr_color(2, &[]), (None, 0));
    }

    #[test]
    fn indexed_selector_takes_one_parameter() {
        assert_eq!(
            parse_sgr_color(5, &[123, 99]),
            (Some(ColorValue::Indexed(123)), 1)
        );
        // An omitted index is skipped without setting anything.
        assert_eq!(parse_sgr_color(5, &[CSI_ARG_MISSING]), (None, 1));
        assert_eq!(parse_sgr_color(5, &[]), (None, 0));
    }

    #[test]
    fn an_unknown_selector_consumes_nothing() {
        assert_eq!(parse_sgr_color(3, &[1, 2, 3]), (None, 0));
        assert_eq!(parse_sgr_color(CSI_ARG_MISSING, &[]), (None, 0));
    }

    #[test]
    fn a_blank_pen_reports_no_parameters_but_its_colours() {
        // Neither colour is marked as the terminal default, so both report,
        // as literal black.
        assert_eq!(
            pen_sgr_params(&blank_pen()),
            vec![
                MORE | 38,
                MORE | 2,
                MORE,
                MORE,
                0,
                MORE | 48,
                MORE | 2,
                MORE,
                MORE,
                0
            ]
        );
    }

    #[test]
    fn default_colours_stay_out_of_the_report() {
        let mut pen = blank_pen();
        pen.fg = mark_default(pen.fg, VTERM_COLOR_DEFAULT_FG);
        pen.bg = mark_default(pen.bg, VTERM_COLOR_DEFAULT_BG);
        assert!(pen_sgr_params(&pen).is_empty());
    }

    #[test]
    fn palette_colours_pick_the_shortest_form() {
        let mut pen = blank_pen();
        pen.bg = mark_default(pen.bg, VTERM_COLOR_DEFAULT_BG);

        pen.fg = plain_color(ColorValue::Indexed(3));
        assert_eq!(pen_sgr_params(&pen), vec![33]);

        pen.fg = plain_color(ColorValue::Indexed(11));
        assert_eq!(pen_sgr_params(&pen), vec![93]);

        pen.fg = plain_color(ColorValue::Indexed(200));
        assert_eq!(pen_sgr_params(&pen), vec![MORE | 38, MORE | 5, 200]);

        pen.fg = plain_color(ColorValue::rgb(1, 2, 3));
        assert_eq!(
            pen_sgr_params(&pen),
            vec![MORE | 38, MORE | 2, MORE | 1, MORE | 2, 3]
        );
    }

    #[test]
    fn every_attribute_at_once_stays_within_the_reply_buffer() {
        let mut pen = blank_pen();
        pen.set_bold(1);
        pen.set_dim(1);
        pen.set_italic(1);
        pen.set_underline(VTERM_UNDERLINE_CURLY);
        pen.set_blink(1);
        pen.set_reverse(1);
        pen.set_conceal(1);
        pen.set_strike(1);
        pen.set_font(3);
        pen.set_overline(1);
        pen.set_small(1);
        pen.set_baseline(VTERM_BASELINE_RAISE);
        pen.fg = plain_color(ColorValue::rgb(1, 2, 3));
        pen.bg = plain_color(ColorValue::rgb(4, 5, 6));
        let params = pen_sgr_params(&pen);
        assert_eq!(params.len(), 22);
        assert_eq!(params[0], 1);
        assert_eq!(*params.last().unwrap(), 73);
    }

    #[test]
    fn underline_styles_are_mutually_exclusive() {
        let mut pen = blank_pen();
        pen.bg = mark_default(pen.bg, VTERM_COLOR_DEFAULT_BG);
        pen.fg = mark_default(pen.fg, VTERM_COLOR_DEFAULT_FG);

        pen.set_underline(VTERM_UNDERLINE_SINGLE);
        assert_eq!(pen_sgr_params(&pen), vec![4]);
        pen.set_underline(VTERM_UNDERLINE_CURLY);
        assert_eq!(pen_sgr_params(&pen), vec![MORE | 4, 3]);
        pen.set_underline(VTERM_UNDERLINE_DOUBLE);
        assert_eq!(pen_sgr_params(&pen), vec![21]);
    }

    #[test]
    fn reading_past_the_parameter_list_yields_an_omitted_parameter() {
        assert_eq!(arg_at(&[1, 2], 5), CSI_ARG_MISSING);
        assert_eq!(arg_value(&[MORE | 7], 0), 7);
    }
}
