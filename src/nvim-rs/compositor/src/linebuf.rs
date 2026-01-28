//! Line buffer and attribute blending for compositor
//!
//! This module handles the line buffer operations used during grid composition:
//! - Line buffer allocation and access
//! - Attribute blending for transparency (pumblend, winblend)
//! - Character conversion utilities (schar_T operations)
//!
//! # Architecture
//!
//! The compositor uses a temporary line buffer (`linebuf`, `attrbuf`) to build
//! each composed line before sending to the UI. This allows proper layering
//! of overlapping grids with transparency support.

use std::ffi::c_int;

use crate::{SattrT, ScharT, ScreenGridHandle};

// =============================================================================
// C accessor declarations for linebuf operations
// =============================================================================

extern "C" {
    // Line buffer accessors
    fn nvim_comp_get_linebuf_char() -> *mut ScharT;
    fn nvim_comp_get_linebuf_attr() -> *mut SattrT;
    fn nvim_comp_get_linebuf_size() -> usize;

    // Message separator character
    fn nvim_get_msg_sep_char() -> ScharT;
    fn nvim_set_msg_sep_char(c: ScharT);

    // Grid blending state (from grid.c)
    fn nvim_screengrid_get_blending(grid: ScreenGridHandle) -> bool;

    // Grid position-based character/attribute access (from ui_compositor.c)
    fn nvim_comp_grid_get_char_at(grid: ScreenGridHandle, row: c_int, col: c_int) -> ScharT;
    fn nvim_comp_grid_get_attr_at(grid: ScreenGridHandle, row: c_int, col: c_int) -> SattrT;
    fn nvim_comp_grid_set_char_at(grid: ScreenGridHandle, row: c_int, col: c_int, c: ScharT);
    fn nvim_comp_grid_set_attr_at(grid: ScreenGridHandle, row: c_int, col: c_int, a: SattrT);

    // Highlight blending
    fn nvim_hl_blend_attrs(back_attr: c_int, front_attr: c_int, through: *mut bool) -> c_int;
}

// =============================================================================
// schar_T utilities
// =============================================================================

/// Constant for NUL character
pub const SCHAR_NUL: ScharT = 0;

/// Create an schar_T from an ASCII byte
#[inline]
pub const fn schar_from_ascii(c: u8) -> ScharT {
    c as ScharT
}

/// Check if character is ASCII space
#[inline]
pub const fn is_space(c: ScharT) -> bool {
    c == schar_from_ascii(b' ')
}

/// Check if character is NUL
#[inline]
pub const fn is_nul(c: ScharT) -> bool {
    c == SCHAR_NUL
}

/// Braille empty pattern (U+2800) for transparency detection
pub const BRAILLE_EMPTY: ScharT = 0x2800;

/// Check if character is "transparent" (space or braille empty)
#[inline]
pub const fn is_transparent_char(c: ScharT) -> bool {
    is_space(c) || c == BRAILLE_EMPTY
}

// =============================================================================
// FFI exports for schar utilities
// =============================================================================
// Note: Basic schar functions (rs_schar_from_ascii, rs_schar_is_space,
// rs_schar_is_nul) are already defined in nvim-grid/src/helpers.rs.
// These compositor-specific exports add transparency checking.

/// Check if schar_T is transparent (space or braille empty)
#[no_mangle]
pub extern "C" fn rs_comp_schar_is_transparent(c: ScharT) -> bool {
    is_transparent_char(c)
}

/// Get the braille empty pattern constant
#[no_mangle]
pub extern "C" fn rs_comp_braille_empty() -> ScharT {
    BRAILLE_EMPTY
}

// =============================================================================
// Line buffer access
// =============================================================================

/// Handle to the compositor's line buffer
#[repr(C)]
pub struct LineBufHandle {
    chars: *mut ScharT,
    attrs: *mut SattrT,
    size: usize,
}

impl LineBufHandle {
    /// Get the current linebuf from compositor
    pub fn get() -> Self {
        unsafe {
            Self {
                chars: nvim_comp_get_linebuf_char(),
                attrs: nvim_comp_get_linebuf_attr(),
                size: nvim_comp_get_linebuf_size(),
            }
        }
    }

    /// Check if linebuf is valid (allocated)
    pub fn is_valid(&self) -> bool {
        !self.chars.is_null() && !self.attrs.is_null() && self.size > 0
    }

    /// Get character at offset
    ///
    /// # Safety
    /// Caller must ensure offset is within bounds
    pub unsafe fn get_char(&self, offset: usize) -> ScharT {
        debug_assert!(offset < self.size);
        *self.chars.add(offset)
    }

    /// Set character at offset
    ///
    /// # Safety
    /// Caller must ensure offset is within bounds
    pub unsafe fn set_char(&mut self, offset: usize, c: ScharT) {
        debug_assert!(offset < self.size);
        *self.chars.add(offset) = c;
    }

    /// Get attribute at offset
    ///
    /// # Safety
    /// Caller must ensure offset is within bounds
    pub unsafe fn get_attr(&self, offset: usize) -> SattrT {
        debug_assert!(offset < self.size);
        *self.attrs.add(offset)
    }

    /// Set attribute at offset
    ///
    /// # Safety
    /// Caller must ensure offset is within bounds
    pub unsafe fn set_attr(&mut self, offset: usize, a: SattrT) {
        debug_assert!(offset < self.size);
        *self.attrs.add(offset) = a;
    }
}

// =============================================================================
// FFI exports for line buffer
// =============================================================================
// Note: The nvim-grid crate has rs_linebuf_* functions that use c_int column
// indices. These compositor functions use usize offsets for direct buffer access.

/// Get linebuf character pointer
#[no_mangle]
pub extern "C" fn rs_comp_linebuf_chars() -> *mut ScharT {
    unsafe { nvim_comp_get_linebuf_char() }
}

/// Get linebuf attribute pointer
#[no_mangle]
pub extern "C" fn rs_comp_linebuf_attrs() -> *mut SattrT {
    unsafe { nvim_comp_get_linebuf_attr() }
}

/// Get linebuf size
#[no_mangle]
pub extern "C" fn rs_comp_linebuf_size() -> usize {
    unsafe { nvim_comp_get_linebuf_size() }
}

/// Check if linebuf is allocated
#[no_mangle]
pub extern "C" fn rs_comp_linebuf_is_valid() -> bool {
    LineBufHandle::get().is_valid()
}

/// Get character from linebuf at offset
#[no_mangle]
pub unsafe extern "C" fn rs_comp_linebuf_get_char(offset: usize) -> ScharT {
    let buf = LineBufHandle::get();
    if offset < buf.size {
        buf.get_char(offset)
    } else {
        SCHAR_NUL
    }
}

/// Set character in linebuf at offset
#[no_mangle]
pub unsafe extern "C" fn rs_comp_linebuf_set_char(offset: usize, c: ScharT) {
    let mut buf = LineBufHandle::get();
    if offset < buf.size {
        buf.set_char(offset, c);
    }
}

/// Get attribute from linebuf at offset
#[no_mangle]
pub unsafe extern "C" fn rs_comp_linebuf_get_attr(offset: usize) -> SattrT {
    let buf = LineBufHandle::get();
    if offset < buf.size {
        buf.get_attr(offset)
    } else {
        0
    }
}

/// Set attribute in linebuf at offset
#[no_mangle]
pub unsafe extern "C" fn rs_comp_linebuf_set_attr(offset: usize, a: SattrT) {
    let mut buf = LineBufHandle::get();
    if offset < buf.size {
        buf.set_attr(offset, a);
    }
}

// =============================================================================
// Message separator
// =============================================================================

/// Get the message separator character
#[no_mangle]
pub extern "C" fn rs_msg_sep_char() -> ScharT {
    unsafe { nvim_get_msg_sep_char() }
}

/// Set the message separator character
#[no_mangle]
pub extern "C" fn rs_set_msg_sep_char(c: ScharT) {
    unsafe { nvim_set_msg_sep_char(c) }
}

// =============================================================================
// Blending support
// =============================================================================

/// Check if a grid has blending enabled
#[no_mangle]
pub extern "C" fn rs_grid_get_blending(grid: ScreenGridHandle) -> bool {
    if grid.is_null() {
        return false;
    }
    unsafe { nvim_screengrid_get_blending(grid) }
}

/// Get character from grid at position
#[no_mangle]
pub extern "C" fn rs_comp_grid_get_char(grid: ScreenGridHandle, row: c_int, col: c_int) -> ScharT {
    if grid.is_null() {
        return SCHAR_NUL;
    }
    unsafe { nvim_comp_grid_get_char_at(grid, row, col) }
}

/// Get attribute from grid at position
#[no_mangle]
pub extern "C" fn rs_comp_grid_get_attr(grid: ScreenGridHandle, row: c_int, col: c_int) -> SattrT {
    if grid.is_null() {
        return 0;
    }
    unsafe { nvim_comp_grid_get_attr_at(grid, row, col) }
}

/// Set character on grid at position
#[no_mangle]
pub extern "C" fn rs_comp_grid_set_char(grid: ScreenGridHandle, row: c_int, col: c_int, c: ScharT) {
    if !grid.is_null() {
        unsafe { nvim_comp_grid_set_char_at(grid, row, col, c) }
    }
}

/// Set attribute on grid at position
#[no_mangle]
pub extern "C" fn rs_comp_grid_set_attr(grid: ScreenGridHandle, row: c_int, col: c_int, a: SattrT) {
    if !grid.is_null() {
        unsafe { nvim_comp_grid_set_attr_at(grid, row, col, a) }
    }
}

/// Blend foreground attribute onto background
///
/// Returns the blended attribute. The `through` parameter indicates if the
/// foreground character is transparent and background should show through.
///
/// # Safety
/// The `through` pointer must be valid and writable.
#[no_mangle]
pub unsafe extern "C" fn rs_blend_attrs(
    back_attr: c_int,
    front_attr: c_int,
    through: *mut bool,
) -> c_int {
    nvim_hl_blend_attrs(back_attr, front_attr, through)
}

/// Check if a character position should blend through
///
/// A position blends through if the foreground character is transparent
/// (space or braille empty) and there's a background character.
#[no_mangle]
pub extern "C" fn rs_should_blend_through(fg_char: ScharT, bg_char: ScharT) -> bool {
    is_transparent_char(fg_char) && !is_nul(bg_char)
}

// =============================================================================
// Debug highlight support
// =============================================================================

/// Opaque handle to debug highlight IDs
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct DebugHighlights {
    pub normal: c_int,
    pub clear: c_int,
    pub composed: c_int,
    pub recompose: c_int,
}

extern "C" {
    fn nvim_comp_get_dbghl_normal() -> c_int;
    fn nvim_comp_get_dbghl_clear() -> c_int;
    fn nvim_comp_get_dbghl_composed() -> c_int;
    fn nvim_comp_get_dbghl_recompose() -> c_int;
}

/// Get all debug highlight IDs
#[no_mangle]
pub extern "C" fn rs_get_debug_highlights() -> DebugHighlights {
    unsafe {
        DebugHighlights {
            normal: nvim_comp_get_dbghl_normal(),
            clear: nvim_comp_get_dbghl_clear(),
            composed: nvim_comp_get_dbghl_composed(),
            recompose: nvim_comp_get_dbghl_recompose(),
        }
    }
}

/// Get the normal debug highlight ID
#[no_mangle]
pub extern "C" fn rs_dbghl_normal() -> c_int {
    unsafe { nvim_comp_get_dbghl_normal() }
}

/// Get the clear debug highlight ID
#[no_mangle]
pub extern "C" fn rs_dbghl_clear() -> c_int {
    unsafe { nvim_comp_get_dbghl_clear() }
}

/// Get the composed debug highlight ID
#[no_mangle]
pub extern "C" fn rs_dbghl_composed() -> c_int {
    unsafe { nvim_comp_get_dbghl_composed() }
}

/// Get the recompose debug highlight ID
#[no_mangle]
pub extern "C" fn rs_dbghl_recompose() -> c_int {
    unsafe { nvim_comp_get_dbghl_recompose() }
}

// =============================================================================
// Debug Delay and Compose Debug
// =============================================================================

extern "C" {
    // UI functions
    fn ui_call_flush();
    fn ui_composed_call_raw_line(
        grid: i64,
        row: i64,
        startcol: i64,
        endcol: i64,
        clearcol: i64,
        clearattr: i64,
        flags: c_int,
        chunk: *const ScharT,
        attrs: *const SattrT,
    );

    // Option accessors
    fn nvim_get_p_wd() -> i64;
    fn nvim_get_rdb_flags() -> u32;

    // Highlight function
    fn nvim_syn_id2attr(hl_id: c_int) -> c_int;

    // Default grid accessors
    fn nvim_get_default_grid() -> crate::ScreenGridHandle;
    fn nvim_screengrid_get_rows(grid: crate::ScreenGridHandle) -> c_int;
    fn nvim_screengrid_get_cols(grid: crate::ScreenGridHandle) -> c_int;
}

/// RedrawDebug flag constants (from option_vars.generated.h)
pub mod rdb_flags {
    /// Compositor debug mode
    pub const COMPOSITOR: u32 = 0x01;
    /// No throttle
    pub const NOTHROTTLE: u32 = 0x02;
    /// Invalid flag
    pub const INVALID: u32 = 0x04;
    /// No delta
    pub const NODELTA: u32 = 0x08;
    /// Line flag
    pub const LINE: u32 = 0x10;
    /// Flush flag
    pub const FLUSH: u32 = 0x20;
}

/// Sleep for debugging visualization.
///
/// Flushes the UI and sleeps based on 'writedelay' option and number of lines.
///
/// This is the Rust implementation of the C `debug_delay()` function.
fn debug_delay_impl(lines: i64) {
    unsafe {
        ui_call_flush();
        let wd = nvim_get_p_wd().unsigned_abs();
        let factor = lines.clamp(1, 5) as u64;
        if wd > 0 {
            std::thread::sleep(std::time::Duration::from_millis(factor * wd));
        }
    }
}

/// Visualize composited area for debugging.
///
/// When 'redrawdebug' contains "compositor", highlights the affected area
/// and optionally delays to make the changes visible.
///
/// This is the Rust implementation of the C `compose_debug()` function.
fn compose_debug_impl(
    startrow: i64,
    endrow: i64,
    startcol: i64,
    endcol: i64,
    syn_id: c_int,
    delay: bool,
) {
    unsafe {
        let flags = nvim_get_rdb_flags();
        if (flags & rdb_flags::COMPOSITOR) == 0 || startcol >= endcol {
            return;
        }

        let default_grid = nvim_get_default_grid();
        let grid_rows = i64::from(nvim_screengrid_get_rows(default_grid));
        let grid_cols = i64::from(nvim_screengrid_get_cols(default_grid));

        let endrow = endrow.min(grid_rows);
        let endcol = endcol.min(grid_cols);
        let attr = i64::from(nvim_syn_id2attr(syn_id));

        if delay {
            debug_delay_impl(endrow - startrow);
        }

        // Get linebuf pointers for the call (we pass empty data, attr fills it)
        let linebuf = nvim_comp_get_linebuf_char();
        let attrbuf = nvim_comp_get_linebuf_attr();

        for row in startrow..endrow {
            ui_composed_call_raw_line(
                1, // grid handle for composed output
                row, startcol, startcol, // endcol same as startcol - no content
                endcol,   // clearcol - area to clear with attr
                attr, 0, // no flags
                linebuf, attrbuf,
            );
        }

        if delay {
            debug_delay_impl(endrow - startrow);
        }
    }
}

/// FFI wrapper for debug_delay.
///
/// Flushes UI and sleeps for debugging visualization.
///
/// # Safety
/// This function calls C UI flush and OS sleep.
#[no_mangle]
pub extern "C" fn rs_debug_delay(lines: i64) {
    debug_delay_impl(lines);
}

/// FFI wrapper for compose_debug.
///
/// Highlights the specified area for debugging when redrawdebug=compositor.
///
/// # Safety
/// This function accesses global state and calls UI functions.
#[no_mangle]
pub extern "C" fn rs_compose_debug(
    startrow: i64,
    endrow: i64,
    startcol: i64,
    endcol: i64,
    syn_id: c_int,
    delay: bool,
) {
    compose_debug_impl(startrow, endrow, startcol, endcol, syn_id, delay);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schar_constants() {
        assert_eq!(SCHAR_NUL, 0);
        assert_eq!(schar_from_ascii(b' '), 0x20);
        assert_eq!(BRAILLE_EMPTY, 0x2800);
    }

    #[test]
    fn test_schar_predicates() {
        assert!(is_space(schar_from_ascii(b' ')));
        assert!(!is_space(schar_from_ascii(b'a')));

        assert!(is_nul(SCHAR_NUL));
        assert!(!is_nul(schar_from_ascii(b' ')));

        assert!(is_transparent_char(schar_from_ascii(b' ')));
        assert!(is_transparent_char(BRAILLE_EMPTY));
        assert!(!is_transparent_char(schar_from_ascii(b'a')));
    }

    #[test]
    fn test_debug_highlights_size() {
        // DebugHighlights should be 4 c_int values
        assert_eq!(
            std::mem::size_of::<DebugHighlights>(),
            std::mem::size_of::<c_int>() * 4
        );
    }

    #[test]
    fn test_linebuf_handle_size() {
        // LineBufHandle should be 2 pointers + size_t
        let expected = std::mem::size_of::<*mut ScharT>()
            + std::mem::size_of::<*mut SattrT>()
            + std::mem::size_of::<usize>();
        assert_eq!(std::mem::size_of::<LineBufHandle>(), expected);
    }
}
