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
// Compose Line Implementation
// =============================================================================

/// LineFlags constants (from ui_defs.h)
pub mod line_flags {
    /// Line wraps to next line
    pub const WRAP: i32 = 1;
    /// Line contains invalid content
    pub const INVALID: i32 = 2;
}

extern "C" {
    // Additional grid accessors for compose_line (those not already declared above)
    fn nvim_screengrid_get_comp_width(grid: crate::ScreenGridHandle) -> c_int;
    fn nvim_screengrid_get_comp_height(grid: crate::ScreenGridHandle) -> c_int;
    fn nvim_screengrid_get_comp_disabled(grid: crate::ScreenGridHandle) -> bool;
    fn nvim_screengrid_get_comp_row(grid: crate::ScreenGridHandle) -> c_int;
    fn nvim_screengrid_get_comp_col(grid: crate::ScreenGridHandle) -> c_int;
    fn nvim_screengrid_get_comp_index(grid: crate::ScreenGridHandle) -> usize;
    fn nvim_screengrid_get_chars(grid: crate::ScreenGridHandle) -> *mut ScharT;
    fn nvim_screengrid_get_attrs(grid: crate::ScreenGridHandle) -> *mut SattrT;
    fn nvim_screengrid_get_line_offset(grid: crate::ScreenGridHandle) -> *mut usize;

    // Layer accessors
    fn nvim_layers_size() -> usize;
    fn nvim_layers_get(i: usize) -> crate::ScreenGridHandle;

    // Message grid and separator
    fn nvim_get_msg_grid() -> crate::ScreenGridHandle;
    fn nvim_get_msg_sep_row() -> c_int;

    // Global state
    fn nvim_get_columns() -> c_int;
    fn nvim_get_hl_attr_active() -> *const c_int;

    // schar_from_char for the braille character
    fn rs_schar_from_char(c: c_int) -> ScharT;
}

/// HLF_MSGSEP index (from highlight_defs.h)
const HLF_MSGSEP: usize = 122; // MsgSeparator highlight group

/// Get highlight attribute for a highlight group.
#[inline]
fn hl_attr(n: usize) -> c_int {
    unsafe {
        let hl_attr_active = nvim_get_hl_attr_active();
        if hl_attr_active.is_null() {
            0
        } else {
            *hl_attr_active.add(n)
        }
    }
}

/// Compose a single line from multiple grid layers.
///
/// This is the core compositing algorithm that merges overlapping grids
/// into the linebuf/attrbuf for display. It handles:
/// - Layer stacking and overlap detection
/// - Double-width character edge cases
/// - Message separator rendering
/// - Transparency blending (pumblend/winblend)
///
/// # Arguments
/// * `row` - Screen row to compose
/// * `startcol` - Starting column (inclusive)
/// * `endcol` - Ending column (exclusive)
/// * `flags` - LineFlags (WRAP, INVALID)
#[allow(clippy::too_many_lines)]
fn compose_line_impl(row: i64, mut startcol: i64, mut endcol: i64, mut flags: c_int) {
    unsafe {
        let default_grid = nvim_get_default_grid();
        let default_cols = i64::from(nvim_screengrid_get_cols(default_grid));

        // If rightleft is set, startcol may be -1
        startcol = startcol.max(0);

        // Check for double-width char edge cases
        let mut skipstart = 0i64;
        let mut skipend = 0i64;
        if startcol > 0 && (flags & line_flags::INVALID) != 0 {
            startcol -= 1;
            skipstart = 1;
        }
        if endcol < default_cols && (flags & line_flags::INVALID) != 0 {
            endcol += 1;
            skipend = 1;
        }

        let linebuf = nvim_comp_get_linebuf_char();
        let attrbuf = nvim_comp_get_linebuf_attr();

        // Get background line data from default_grid
        let default_chars = nvim_screengrid_get_chars(default_grid);
        let default_attrs = nvim_screengrid_get_attrs(default_grid);
        let default_line_offset = nvim_screengrid_get_line_offset(default_grid);

        let row_offset = *default_line_offset.add(row as usize) + startcol as usize;
        let bg_line = default_chars.add(row_offset);
        let bg_attrs = default_attrs.add(row_offset);

        let msg_grid = nvim_get_msg_grid();
        let msg_sep_row = nvim_get_msg_sep_row();
        let msg_grid_comp_index = nvim_screengrid_get_comp_index(msg_grid);
        let msg_sep_char = nvim_get_msg_sep_char();

        let layers_size = nvim_layers_size();
        let mut col = startcol as c_int;
        let mut last_grid = crate::ScreenGridHandle(std::ptr::null_mut());

        while i64::from(col) < endcol {
            let mut until = 0i32;
            let mut grid = crate::ScreenGridHandle(std::ptr::null_mut());

            // Find the topmost grid covering this column
            for i in 0..layers_size {
                let g = nvim_layers_get(i);

                // Check for pending resize - use min of actual and comp dimensions
                let grid_width = nvim_screengrid_get_cols(g).min(nvim_screengrid_get_comp_width(g));
                let grid_height =
                    nvim_screengrid_get_rows(g).min(nvim_screengrid_get_comp_height(g));
                let comp_row = nvim_screengrid_get_comp_row(g);
                let comp_col = nvim_screengrid_get_comp_col(g);

                // Skip if row is outside this grid
                if comp_row > row as c_int
                    || row as c_int >= comp_row + grid_height
                    || nvim_screengrid_get_comp_disabled(g)
                {
                    continue;
                }

                if comp_col <= col && col < comp_col + grid_width {
                    grid = g;
                    until = comp_col + grid_width;
                } else if comp_col > col && (until == 0 || comp_col < until) {
                    until = comp_col;
                }
            }
            until = until.min(endcol as c_int);

            // These should hold if layers is properly set up
            debug_assert!(!grid.is_null());
            debug_assert!(until > col);
            debug_assert!(i64::from(until) <= default_cols);

            let n = (until - col) as usize;
            let buf_offset = (col - startcol as c_int) as usize;

            // Check for message separator row
            if row as c_int == msg_sep_row
                && nvim_screengrid_get_comp_index(grid) <= msg_grid_comp_index
            {
                // Fill with message separator
                let msg_sep_attr = hl_attr(HLF_MSGSEP) as SattrT;
                for i in col..until {
                    let idx = (i - startcol as c_int) as usize;
                    *linebuf.add(idx) = msg_sep_char;
                    *attrbuf.add(idx) = msg_sep_attr;
                }
                last_grid = msg_grid;
            } else {
                // Copy from the grid
                let grid_chars = nvim_screengrid_get_chars(grid);
                let grid_attrs = nvim_screengrid_get_attrs(grid);
                let grid_line_offset = nvim_screengrid_get_line_offset(grid);
                let comp_row = nvim_screengrid_get_comp_row(grid);
                let comp_col = nvim_screengrid_get_comp_col(grid);

                let grid_row = (row as c_int - comp_row) as usize;
                let grid_col = (col - comp_col) as usize;
                let off = *grid_line_offset.add(grid_row) + grid_col;

                std::ptr::copy_nonoverlapping(grid_chars.add(off), linebuf.add(buf_offset), n);
                std::ptr::copy_nonoverlapping(grid_attrs.add(off), attrbuf.add(buf_offset), n);

                // Handle doublewidth char cut-off at end
                let grid_cols = nvim_screengrid_get_cols(grid);
                if comp_col + grid_cols > until && *grid_chars.add(off + n) == SCHAR_NUL {
                    *linebuf.add(buf_offset + n - 1) = schar_from_ascii(b' ');
                    if col == startcol as c_int && n == 1 {
                        skipstart = 0;
                    }
                }
                last_grid = grid;
            }

            // Handle blending (pumblend/winblend)
            if nvim_screengrid_get_blending(grid) {
                let mut i = buf_offset;
                let until_offset = (until - startcol as c_int) as usize;

                while i < until_offset {
                    let mut width = 1usize;
                    let fg_char = *linebuf.add(i);
                    let bg_char = *bg_line.add(i);

                    // Check for transparent foreground
                    let braille_empty = rs_schar_from_char(0x2800);
                    let mut thru = (fg_char == schar_from_ascii(b' ') || fg_char == braille_empty)
                        && bg_char != SCHAR_NUL;

                    // Check for doublewidth background
                    if i + 1 < (endcol - startcol) as usize && *bg_line.add(i + 1) == SCHAR_NUL {
                        width = 2;
                        let fg_char2 = *linebuf.add(i + 1);
                        thru &= fg_char2 == schar_from_ascii(b' ') || fg_char2 == braille_empty;
                    }

                    // Blend attributes
                    *attrbuf.add(i) = nvim_hl_blend_attrs(
                        c_int::from(*bg_attrs.add(i)),
                        c_int::from(*attrbuf.add(i)),
                        &raw mut thru,
                    ) as SattrT;

                    if width == 2 {
                        *attrbuf.add(i + 1) = nvim_hl_blend_attrs(
                            c_int::from(*bg_attrs.add(i + 1)),
                            c_int::from(*attrbuf.add(i + 1)),
                            &raw mut thru,
                        ) as SattrT;
                    }

                    // Copy background through if transparent
                    if thru {
                        std::ptr::copy_nonoverlapping(bg_line.add(i), linebuf.add(i), width);
                    }

                    i += width;
                }
            }

            // Handle doublewidth char cut-off at start
            if *linebuf.add(buf_offset) == SCHAR_NUL {
                *linebuf.add(buf_offset) = schar_from_ascii(b' ');
                if col == endcol as c_int - 1 {
                    skipend = 0;
                }
            } else if col == startcol as c_int && n > 1 && *linebuf.add(1) == SCHAR_NUL {
                skipstart = 0;
            }

            col = until;
        }

        // Final doublewidth check
        let last_idx = (endcol - startcol - 1) as usize;
        if *linebuf.add(last_idx) == SCHAR_NUL {
            skipend = 0;
        }

        // Clear wrap flag if not at full width
        let columns = nvim_get_columns();
        if last_grid.is_null()
            || (last_grid.0 != default_grid.0
                && !(nvim_screengrid_get_comp_col(last_grid) == 0
                    && nvim_screengrid_get_cols(last_grid) == columns))
        {
            flags &= !line_flags::WRAP;
        }

        // Validate attributes
        let rdb_flags = nvim_get_rdb_flags();
        for i in skipstart as usize..(endcol - skipend - startcol) as usize {
            if *attrbuf.add(i) < 0 {
                if (rdb_flags & rdb_flags::INVALID) != 0 {
                    std::process::abort();
                } else {
                    *attrbuf.add(i) = 0;
                }
            }
        }

        // Output the composed line
        ui_composed_call_raw_line(
            1,
            row,
            startcol + skipstart,
            endcol - skipend,
            endcol - skipend,
            0,
            flags,
            linebuf.add(skipstart as usize),
            attrbuf.add(skipstart as usize),
        );
    }
}

/// FFI wrapper for compose_line.
///
/// Composes a single line from multiple grid layers.
///
/// # Safety
/// This function accesses global compositor state and grid data.
#[no_mangle]
pub extern "C" fn rs_compose_line(row: i64, startcol: i64, endcol: i64, flags: c_int) {
    compose_line_impl(row, startcol, endcol, flags);
}

// =============================================================================
// Compose Area Implementation
// =============================================================================

/// Compose a rectangular area from multiple grid layers.
///
/// This function first calls compose_debug for visualization, then
/// iterates over all rows in the area calling compose_line for each.
///
/// # Arguments
/// * `startrow` - Starting row (inclusive)
/// * `endrow` - Ending row (exclusive)
/// * `startcol` - Starting column (inclusive)
/// * `endcol` - Ending column (exclusive)
fn compose_area_impl(startrow: i64, mut endrow: i64, startcol: i64, mut endcol: i64) {
    unsafe {
        // Get debug highlight ID for recompose
        let dbghl_recompose = nvim_comp_get_dbghl_recompose();

        // Call debug visualization
        compose_debug_impl(startrow, endrow, startcol, endcol, dbghl_recompose, true);

        // Clamp to default grid bounds
        let default_grid = nvim_get_default_grid();
        let grid_rows = i64::from(nvim_screengrid_get_rows(default_grid));
        let grid_cols = i64::from(nvim_screengrid_get_cols(default_grid));

        endrow = endrow.min(grid_rows);
        endcol = endcol.min(grid_cols);

        if endcol <= startcol {
            return;
        }

        // Compose each row
        for r in startrow..endrow {
            compose_line_impl(r, startcol, endcol, line_flags::INVALID);
        }
    }
}

/// FFI wrapper for compose_area.
///
/// Composes a rectangular area from multiple grid layers.
///
/// # Safety
/// This function accesses global compositor state and grid data.
#[no_mangle]
pub extern "C" fn rs_compose_area(startrow: i64, endrow: i64, startcol: i64, endcol: i64) {
    compose_area_impl(startrow, endrow, startcol, endcol);
}

// =============================================================================
// UI Comp Raw Line Implementation
// =============================================================================

extern "C" {
    // curgrid accessors (not already declared above)
    fn nvim_get_curgrid() -> crate::ScreenGridHandle;

    // curgrid_covered_above from Rust
    fn rs_curgrid_covered_above(row: c_int) -> bool;

    // ui_comp_should_draw and ui_comp_set_grid from Rust
    fn rs_ui_comp_should_draw() -> c_int;
    fn rs_ui_comp_set_grid(handle: c_int) -> c_int;
}

/// Process a raw line update from a grid.
///
/// This function handles incoming raw line updates from grids and either:
/// - Recomposes the line through the full compositor (if there's overlap or blending)
/// - Passes through directly to the UI (if uncovered and no blending)
///
/// # Arguments
/// * `grid` - Grid handle
/// * `row` - Row in grid coordinates
/// * `startcol` - Start column in grid coordinates
/// * `endcol` - End of content in grid coordinates
/// * `clearcol` - End of clear area in grid coordinates
/// * `clearattr` - Attribute for cleared area
/// * `flags` - LineFlags
/// * `chunk` - Character data
/// * `attrs` - Attribute data
#[allow(clippy::too_many_arguments)]
fn ui_comp_raw_line_impl(
    grid_handle: i64,
    mut row: i64,
    mut startcol: i64,
    mut endcol: i64,
    mut clearcol: i64,
    clearattr: i64,
    mut flags: c_int,
    chunk: *const ScharT,
    attrs: *const SattrT,
) {
    unsafe {
        // Early return if can't draw or can't set grid
        if rs_ui_comp_should_draw() == 0 || rs_ui_comp_set_grid(grid_handle as c_int) == 0 {
            return;
        }

        let curgrid = nvim_get_curgrid();
        let comp_row = nvim_screengrid_get_comp_row(curgrid);
        let comp_col = nvim_screengrid_get_comp_col(curgrid);

        // Transform from grid coordinates to screen coordinates
        row += i64::from(comp_row);
        startcol += i64::from(comp_col);
        endcol += i64::from(comp_col);
        clearcol += i64::from(comp_col);

        // Clear wrap flag if not on default grid
        let default_grid = nvim_get_default_grid();
        if curgrid.0 != default_grid.0 {
            flags &= !line_flags::WRAP;
        }

        debug_assert!(endcol <= clearcol);

        // Bounds checking
        let default_rows = i64::from(nvim_screengrid_get_rows(default_grid));
        let default_cols = i64::from(nvim_screengrid_get_cols(default_grid));

        if row >= default_rows {
            // Invalid row - skip silently (logged in C)
            return;
        }

        if clearcol > default_cols {
            // Column out of bounds - clamp
            if startcol >= default_cols {
                return;
            }
            clearcol = default_cols;
            endcol = endcol.min(clearcol);
        }

        // Check if line needs full recomposition
        let covered = rs_curgrid_covered_above(row as c_int);
        let blending = nvim_screengrid_get_blending(curgrid);

        if (flags & line_flags::INVALID) != 0 || covered || blending {
            // Full recomposition needed
            let dbghl_composed = nvim_comp_get_dbghl_composed();
            compose_debug_impl(row, row + 1, startcol, clearcol, dbghl_composed, true);
            compose_line_impl(row, startcol, clearcol, flags);
        } else {
            // Direct passthrough
            let dbghl_normal = nvim_comp_get_dbghl_normal();
            let dbghl_clear = nvim_comp_get_dbghl_clear();

            compose_debug_impl(
                row,
                row + 1,
                startcol,
                endcol,
                dbghl_normal,
                endcol >= clearcol,
            );
            compose_debug_impl(row, row + 1, endcol, clearcol, dbghl_clear, true);

            // Debug: verify all attributes are non-negative
            #[cfg(debug_assertions)]
            {
                for i in 0..(endcol - startcol) as usize {
                    debug_assert!(*attrs.add(i) >= 0);
                }
            }

            ui_composed_call_raw_line(
                1, row, startcol, endcol, clearcol, clearattr, flags, chunk, attrs,
            );
        }
    }
}

/// FFI wrapper for ui_comp_raw_line.
///
/// Processes a raw line update from a grid.
///
/// # Safety
/// This function accesses global compositor state and grid data.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn rs_ui_comp_raw_line(
    grid: i64,
    row: i64,
    startcol: i64,
    endcol: i64,
    clearcol: i64,
    clearattr: i64,
    flags: c_int,
    chunk: *const ScharT,
    attrs: *const SattrT,
) {
    ui_comp_raw_line_impl(
        grid, row, startcol, endcol, clearcol, clearattr, flags, chunk, attrs,
    );
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
