//! The shape, blink and colour of the terminal cursor.
//!
//! The editor describes the cursor it wants per mode -- one
//! [`cursorentry_T`] for each entry in `'guicursor'` -- and the TUI turns
//! whichever is current into escape sequences. Two of those are terminfo
//! capabilities (`set_cursor_style`, `set_cursor_color`); the shape
//! numbering they take is DECSCUSR's, which is not the numbering the editor
//! uses, and [`decscusr_code`] is where the two meet.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cursor_shape::{SHAPE_BLOCK, SHAPE_HOR, SHAPE_IDX_N, SHAPE_VER, shape_entry};
use crate::global_cell::GlobalCell;
use crate::highlight::HlAttrFlags;
use crate::log::{LOGLVL_WRN, logmsg_c};
use crate::tui::output::{terminfo_out, terminfo_print_nums, terminfo_print_str};
use crate::tui::terminfo::caps::{
    kTerm_reset_cursor_color, kTerm_reset_cursor_style, kTerm_set_cursor_color,
    kTerm_set_cursor_style,
};
use crate::types::{CursorShape, Dict, HlAttrs, RgbValue, TUIData, cursorentry_T, int32_t};
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
fn decscusr_code(entry: &cursorentry_T) -> c_int {
    let base = match entry.shape {
        SHAPE_BLOCK => 1,
        SHAPE_HOR => 3,
        SHAPE_VER => 5,
        _ => 0,
    };
    let steady = entry.blinkon == 0 || entry.blinkoff == 0;
    base + steady as c_int
}

/// The shape `'guicursor'` names, falling back to a block.
///
/// # Safety
/// `shape_str` must be null or a NUL-terminated string.
unsafe fn decode_shape(shape_str: *const core::ffi::c_char) -> CursorShape {
    let name = if shape_str.is_null() {
        b"".as_slice()
    } else {
        // SAFETY: the caller guarantees NUL termination.
        unsafe { CStr::from_ptr(shape_str) }.to_bytes()
    };
    match name {
        b"block" => SHAPE_BLOCK,
        b"vertical" => SHAPE_VER,
        b"horizontal" => SHAPE_HOR,
        _ => {
            // SAFETY: a NUL-terminated format and argument.
            unsafe {
                logmsg_c!(
                    LOGLVL_WRN,
                    core::ptr::null(),
                    c"tui_cursor_decode_shape".as_ptr(),
                    0,
                    true,
                    c"Unknown shape value '%s'".as_ptr(),
                    shape_str,
                );
            }
            SHAPE_BLOCK
        }
    }
}

/// Read one mode's cursor description out of the API dict the editor sent.
///
/// Unrecognised keys are ignored and missing ones keep the default the
/// shape table starts every entry with.
///
/// # Safety
/// `args` must be a valid `Dict` whose items outlive the call.
pub unsafe fn decode_cursor_entry(args: Dict) -> cursorentry_T {
    // SAFETY: the caller guarantees the dict and its items are valid.
    unsafe {
        let mut entry = shape_entry(SHAPE_IDX_N);
        for i in 0..args.size {
            let item = &*args.items.add(i);
            let key = if item.key.data().is_null() {
                b"".as_slice()
            } else {
                CStr::from_ptr(item.key.data()).to_bytes()
            };
            match key {
                b"cursor_shape" => entry.shape = decode_shape(item.value.data.string.data()),
                b"blinkon" => entry.blinkon = item.value.data.integer as c_int,
                b"blinkoff" => entry.blinkoff = item.value.data.integer as c_int,
                b"attr_id" => entry.id = item.value.data.integer as c_int,
                _ => {}
            }
        }
        entry
    }
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
fn apply_color(tui: &mut TUIData, entry: &cursorentry_T) {
    let in_range = entry.id != 0 && (entry.id as usize) < tui.attrs.len() && tui.rgb;
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

    let aep: HlAttrs = tui.attrs[entry.id as usize];
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
            let color = aep.rgb_bg_color as u32 & 0xff_ffff;
            hex[0] = b'#';
            for i in 0..6 {
                let nibble = (color >> (20 - 4 * i)) & 0xf;
                hex[1 + i as usize] = char::from_digit(nibble, 16).unwrap_or('0') as u8;
            }
            let s = CStr::from_bytes_with_nul(&hex).expect("hex buffer is NUL-terminated");
            terminfo_print_str(tui, kTerm_set_cursor_color, s);
        } else {
            terminfo_print_nums(tui, kTerm_set_cursor_color, &[aep.rgb_bg_color as c_int]);
        }
        tui.cursor_has_color = true;
    }
}
