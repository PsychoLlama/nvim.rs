//! The shape, blink and colour of the terminal cursor.
//!
//! The editor describes the cursor it wants per mode -- one
//! [`CursorEntry`] for each entry in `'guicursor'` -- and the TUI turns
//! whichever is current into escape sequences. Two of those are terminfo
//! capabilities (`set_cursor_style`, `set_cursor_color`); the shape
//! numbering they take is DECSCUSR's, which is not the numbering the editor
//! uses, and [`decscusr_code`] is where the two meet.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cursor_shape::{SHAPE_BLOCK, SHAPE_HOR, SHAPE_IDX_N, SHAPE_VER, shape_entry};
use crate::global_cell::GlobalCell;
use crate::highlight::HlAttrFlags;
use crate::log::{LOGLVL_WRN, logmsg};
use crate::message_fmt::msg_cstr_opt;
use crate::narrow::as_index;
use crate::narrow::number_as_int;
use crate::os::libc::int32_t;
use crate::tui::output::{terminfo_out, terminfo_print_nums, terminfo_print_str};
use crate::tui::terminfo::caps::{
    kTerm_reset_cursor_color, kTerm_reset_cursor_style, kTerm_set_cursor_color,
    kTerm_set_cursor_style,
};
use crate::types::{
    ApiDict, ApiString, CursorEntry, CursorShape, HlAttrs, Object, RgbValue, TUIData,
};
use core::ffi::{CStr, c_int};

/// Does the editor want the TUI driving cursor style at all? `'guicursor'`
/// being empty switches the whole mechanism off, and then the TUI must put
/// the cursor back the way it found it.
pub static cursor_style_enabled: GlobalCell<bool> = GlobalCell::new(false);

/// The blend value `'guicursor'` uses to mean "no cursor at all".
const BLEND_INVISIBLE: int32_t = 100;

/// The DECSCUSR parameter for a shape, in its steady form.
///
/// DECSCUSR numbers the shapes 1/3/5 for block/underline/bar and uses the
/// odd number for blinking, the even one above it for steady -- the inverse
/// of the editor's `blinkon`/`blinkoff` pair, where zero means "do not
/// blink". Hence the `+ 1` rather than a second table.
fn decscusr_code(entry: &CursorEntry) -> c_int {
    let base = match entry.shape {
        SHAPE_BLOCK => 1,
        SHAPE_HOR => 3,
        SHAPE_VER => 5,
        _ => 0,
    };
    let steady = entry.blinkon == 0 || entry.blinkoff == 0;
    base + c_int::from(steady)
}

/// The shape `'guicursor'` names, falling back to a block.
fn decode_shape(shape: &ApiString) -> CursorShape {
    match shape.as_bytes() {
        b"block" => SHAPE_BLOCK,
        b"vertical" => SHAPE_VER,
        b"horizontal" => SHAPE_HOR,
        _ => {
            let shown = msg_cstr_opt((!shape.is_null()).then(|| shape.as_cstr()));
            logmsg!(
                LOGLVL_WRN,
                c"tui_cursor_decode_shape",
                0,
                "Unknown shape value '{}'",
                shown
            );
            SHAPE_BLOCK
        }
    }
}

/// Read one mode's cursor description out of the API dict the editor sent.
///
/// Unrecognised keys are ignored and missing ones keep the default the
/// shape table starts every entry with.
pub fn decode_cursor_entry(args: &ApiDict) -> CursorEntry {
    let mut entry = shape_entry(SHAPE_IDX_N);
    for item in args {
        // A value of the wrong kind keeps the default, the same as a
        // key nobody recognised; the transpile read the union arm the
        // *key* named and would have dereferenced whatever was there.
        match (item.key.bytes(), &item.value) {
            (b"cursor_shape", Object::String(name)) => entry.shape = decode_shape(name),
            (b"blinkon", Object::Integer(ms)) => entry.blinkon = number_as_int(*ms),
            (b"blinkoff", Object::Integer(ms)) => entry.blinkoff = number_as_int(*ms),
            (b"attr_id", Object::Integer(id)) => entry.id = number_as_int(*id),
            _ => {}
        }
    }
    entry
}

/// Put the cursor style back to the terminal's default.
pub fn reset_style(tui: &mut TUIData) {
    terminfo_out(tui, kTerm_reset_cursor_style);
}

/// Emit the cursor the editor wants for `mode`.
pub fn set_mode(tui: &mut TUIData, mode: usize) {
    if !cursor_style_enabled.get() {
        reset_style(tui);
        return;
    }
    // `mode` indexes the fixed-size shape array, whose bounds Rust checks.
    let entry = tui.cursor_shapes[mode];
    apply_color(tui, &entry);
    terminfo_print_nums(tui, kTerm_set_cursor_style, &[decscusr_code(&entry)]);
}

/// Colour the cursor to match the highlight group its mode names, or undo a
/// colour a previous mode set.
///
/// Only meaningful with an attribute id in range and a truecolour terminal:
/// the cursor colour capability takes an RGB value, and there is nothing
/// sensible to send it from a palette index.
///
fn apply_color(tui: &mut TUIData, entry: &CursorEntry) {
    let in_range = entry.id != 0 && as_index(entry.id) < tui.attrs.len() && tui.rgb;
    if !in_range {
        // Attribute 0 means "no special cursor". Only bother resetting if a
        // previous mode actually changed something.
        if entry.id == 0 && (tui.want_invisible || tui.cursor_has_color) {
            tui.want_invisible = false;
            tui.cursor_has_color = false;
            terminfo_out(tui, kTerm_reset_cursor_color);
        }
        return;
    }

    let aep: HlAttrs = tui.attrs[as_index(entry.id)];
    tui.want_invisible = aep.hl_blend == BLEND_INVISIBLE;
    if tui.want_invisible {
        return;
    }
    if aep.rgb_ae_attr.has(HlAttrFlags::INVERSE) {
        // The terminal's own inverse video is the cursor; any colour we set
        // would fight it.
        terminfo_out(tui, kTerm_reset_cursor_color);
    } else if aep.rgb_bg_color >= 0 as RgbValue {
        // Some terminals want `#rrggbb`, the rest a bare number; which one
        // was settled when the description was augmented.
        if tui.set_cursor_color_as_str {
            let mut hex = [0u8; 8];
            // 24 bits is all the capability can carry.
            let color = aep.rgb_bg_color.cast_unsigned() & 0xff_ffff;
            hex[0] = b'#';
            for i in 0..6 {
                let nibble = (color >> (20 - 4 * i)) & 0xf;
                hex[1 + as_index(i)] = char::from_digit(nibble, 16).unwrap_or('0') as u8;
            }
            let s = CStr::from_bytes_with_nul(&hex).expect("hex buffer is NUL-terminated");
            terminfo_print_str(tui, kTerm_set_cursor_color, s);
        } else {
            terminfo_print_nums(tui, kTerm_set_cursor_color, &[aep.rgb_bg_color as c_int]);
        }
        tui.cursor_has_color = true;
    }
}
