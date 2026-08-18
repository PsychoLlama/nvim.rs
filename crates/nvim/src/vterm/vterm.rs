//! The terminal object: its size, the buffer replies are collected in, and
//! the storage its state machine and screen are built out of.
//!
//! This is also where the vocabulary the rest of the subtree shares lives —
//! the property, attribute and selection numbers, and the parser's states.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint, c_void};

use crate::memory::{xfree, xmalloc};
use crate::types::{
    VTerm, VTermAttr, VTermDamageSize, VTermKey, VTermModifier, VTermOutputCallback,
    VTermParserState, VTermProp, VTermRect, VTermSelectionMask, VTermState_mouse_protocol,
    VTermState_tmp_selection_state, VTermTerminator, VTermValueType, size_t,
};
use crate::vterm::screen::vterm_screen_free;
use crate::vterm::state::entry::vterm_state_free;
use ::libc::memset;

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
const VTERM_BUFFER_SIZE: size_t = 4096;

/// Zero-filled storage for one of the terminal's internal arrays.
///
/// # Safety
///
/// The result is only valid until it is handed to [`vterm_dealloc`].
pub unsafe fn vterm_alloc(size: size_t) -> *mut c_void {
    // SAFETY: the allocator answers a run of `size` writable bytes -- it
    // dies rather than return null, but the guard upstream wrote is kept.
    let ptr = unsafe { xmalloc(size) };
    if !ptr.is_null() {
        // SAFETY: exactly the run just allocated.
        unsafe { memset(ptr, 0, size) };
    }
    ptr
}

/// Releases storage from [`vterm_alloc`].
pub unsafe fn vterm_dealloc(ptr: *mut c_void) {
    // SAFETY: forwarded to this function's own caller.
    unsafe { xfree(ptr) };
}

/// A terminal `rows` by `cols`, with no state machine or screen yet — those
/// are built on first ask by `vterm_obtain_state` and `vterm_obtain_screen`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_new(rows: c_int, cols: c_int) -> *mut VTerm {
    // SAFETY: the allocator answers a zeroed `VTerm`-sized run, and all-zero
    // is a valid `VTerm` -- every field is a scalar, a raw pointer, or a
    // nullable function pointer -- so a reference to it is sound and the
    // fields below are then ordinary assignments.
    let vt = unsafe { vterm_alloc(::core::mem::size_of::<VTerm>()) }.cast::<VTerm>();
    // SAFETY: as above.
    let term = unsafe { &mut *vt };
    term.rows = rows;
    term.cols = cols;
    term.parser.state = NORMAL;
    term.parser.callbacks = ::core::ptr::null();
    term.parser.cbdata = ::core::ptr::null_mut();
    term.parser.emit_nul = false;
    term.outfunc = None;
    term.outdata = ::core::ptr::null_mut();
    term.outbuffer_len = VTERM_BUFFER_SIZE;
    term.outbuffer_cur = 0;
    term.tmpbuffer_len = VTERM_BUFFER_SIZE;
    // SAFETY: as above -- two more runs from the same allocator.
    term.outbuffer = unsafe { vterm_alloc(VTERM_BUFFER_SIZE) }.cast::<c_char>();
    // SAFETY: as above.
    term.tmpbuffer = unsafe { vterm_alloc(VTERM_BUFFER_SIZE) }.cast::<c_char>();
    vt
}

pub unsafe fn vterm_free(vt: *mut VTerm) {
    // Everything the terminal owns is read out before anything is freed, so
    // that the last release -- the terminal itself -- has nothing left to
    // invalidate.
    //
    // SAFETY: the caller hands over a terminal from `vterm_new` and does not
    // use it again.
    let term = unsafe { *vt };
    if !term.screen.is_null() {
        // SAFETY: a screen this terminal obtained, released exactly once.
        unsafe { vterm_screen_free(term.screen) };
    }
    if !term.state.is_null() {
        // SAFETY: a state this terminal obtained, released exactly once.
        unsafe { vterm_state_free(term.state) };
    }
    // SAFETY: the three runs `vterm_new` allocated, each released exactly
    // once, the terminal itself last.
    unsafe { vterm_dealloc(term.outbuffer.cast::<c_void>()) };
    // SAFETY: as above.
    unsafe { vterm_dealloc(term.tmpbuffer.cast::<c_void>()) };
    // SAFETY: as above.
    unsafe { vterm_dealloc(vt.cast::<c_void>()) };
}

pub unsafe fn vterm_get_size(vt: *const VTerm, rowsp: *mut c_int, colsp: *mut c_int) {
    // SAFETY: the caller hands over a live terminal.
    let (rows, cols) = unsafe { ((*vt).rows, (*vt).cols) };
    // A null out-parameter means "don't report this one".
    //
    // SAFETY: a non-null one points at a writable `c_int`.
    if let Some(slot) = unsafe { rowsp.as_mut() } {
        *slot = rows;
    }
    // SAFETY: as above.
    if let Some(slot) = unsafe { colsp.as_mut() } {
        *slot = cols;
    }
}

/// Resizes the terminal, telling the parser's consumer so that it can move
/// the cursor and the screen contents. A degenerate size is refused.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_set_size(vt: *mut VTerm, rows: c_int, cols: c_int) {
    if rows < 1 || cols < 1 {
        return;
    }
    // The consumer's resize callback is free to re-enter the terminal, so
    // the borrow of it ends before the callback is reached.
    //
    // SAFETY: the caller hands over a live terminal, whose parser callback
    // table is the consumer's own and live for as long as it is installed.
    let (resize, cbdata) = unsafe {
        let term = &mut *vt;
        term.rows = rows;
        term.cols = cols;
        let resize = term.parser.callbacks.as_ref().and_then(|c| c.resize);
        (resize, term.parser.cbdata)
    };
    if let Some(resize) = resize {
        // SAFETY: the consumer's own callback, taking the data it registered.
        unsafe { resize(rows, cols, cbdata) };
    }
}

/// Selects whether input bytes are decoded as UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_set_utf8(vt: *mut VTerm, is_utf8: c_int) {
    // SAFETY: the caller hands over a live terminal.
    unsafe { &mut *vt }.mode.set_utf8(is_utf8 as c_uint);
}

/// Installs the sink replies are written to. Without one they collect in the
/// terminal's own buffer until it is full.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_output_set_callback(
    vt: *mut VTerm,
    func: Option<VTermOutputCallback>,
    user: *mut c_void,
) {
    // SAFETY: the caller hands over a live terminal.
    let term = unsafe { &mut *vt };
    term.outfunc = func;
    term.outdata = user;
}

/// Writes a reply back to the host. With no sink installed and no room left
/// in the buffer, the reply is dropped whole rather than truncated.
pub unsafe fn vterm_push_output_bytes(vt: *mut VTerm, bytes: *const c_char, len: size_t) {
    // The consumer's sink is free to re-enter the terminal, so it is reached
    // with nothing borrowed.
    //
    // SAFETY: the caller hands over a live terminal.
    let (outfunc, outdata) = unsafe { ((*vt).outfunc, (*vt).outdata) };
    if let Some(outfunc) = outfunc {
        // SAFETY: the consumer's own sink, taking the reply and the data it
        // registered; `bytes` and `len` are the caller's, unchanged.
        unsafe { outfunc(bytes, len, outdata) };
        return;
    }
    // SAFETY: the same live terminal; nothing below re-enters.
    let term = unsafe { &mut *vt };
    let room = term.outbuffer_len - term.outbuffer_cur;
    if len > room {
        return;
    }
    // SAFETY: `outbuffer_cur` bytes of the terminal's own buffer are in use,
    // so that offset is in bounds of it.
    let free = unsafe { term.outbuffer.add(term.outbuffer_cur) };
    // SAFETY: `len` bytes fit in what is left, and the terminal's buffer is a
    // separate allocation from the caller's `bytes`.
    unsafe { free.copy_from_nonoverlapping(bytes, len) };
    term.outbuffer_cur += len;
}

/// What a `VTermValue` accompanying `attr` holds.
pub fn vterm_get_attr_type(attr: VTermAttr) -> VTermValueType {
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

/// Drives a scroll out of a consumer's move and erase primitives, for a
/// consumer that did not want to take the whole scroll itself.
pub unsafe fn vterm_scroll_rect(
    rect: VTermRect,
    downward: c_int,
    rightward: c_int,
    moverect: Option<unsafe extern "C" fn(VTermRect, VTermRect, *mut c_void) -> c_int>,
    eraserect: Option<unsafe extern "C" fn(VTermRect, c_int, *mut c_void) -> c_int>,
    user: *mut c_void,
) {
    let erase = eraserect.expect("scrolling without a way to erase");
    let Some((dest, src)) = scroll_split(rect, downward, rightward) else {
        // SAFETY: the consumer's own primitives, taking the data it passed in
        // alongside them.
        unsafe { erase(rect, 0, user) };
        return;
    };
    if let Some(moverect) = moverect {
        // SAFETY: as above.
        unsafe { moverect(dest, src, user) };
    }
    // SAFETY: as above.
    unsafe { erase(scroll_vacated(rect, downward, rightward), 0, user) };
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
        assert_eq!(vterm_get_attr_type(VTERM_ATTR_BOLD), VTERM_VALUETYPE_BOOL);
        assert_eq!(vterm_get_attr_type(VTERM_ATTR_FONT), VTERM_VALUETYPE_INT);
        assert_eq!(
            vterm_get_attr_type(VTERM_ATTR_FOREGROUND),
            VTERM_VALUETYPE_COLOR
        );
        // Past the last attribute there is no value at all.
        assert_eq!(vterm_get_attr_type(VTERM_N_ATTRS), 0);
        assert_eq!(vterm_get_attr_type(0), 0);
    }
}
