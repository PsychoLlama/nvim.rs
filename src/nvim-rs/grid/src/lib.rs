//! Grid utilities for Neovim
//!
//! This crate provides Rust implementations of grid/screen character functions
//! from `src/nvim/grid.c`.

#![allow(unsafe_code)] // FFI requires unsafe

use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_uint, CStr};
use std::sync::{LazyLock, Mutex};

/// Type alias for screen character (matches C's `schar_T` which is `uint32_t`).
type ScharT = u32;

/// Unicode replacement character
const REPLACEMENT_CHAR: i32 = 0xFFFD;

/// Maximum size of a screen character in bytes (including NUL)
const MAX_SCHAR_SIZE: usize = 32;

/// Capacity threshold for clearing the glyph cache (2^21)
const GLYPH_CACHE_CLEAR_THRESHOLD: usize = 1 << 21;

// =============================================================================
// Glyph Cache Implementation (Phase 26)
// =============================================================================

/// A cache for glyphs that don't fit in a 4-byte `schar_T`.
///
/// Glyphs > 4 bytes are stored in this cache, and the `schar_T` value
/// stores an index into the cache with a 0xFF marker byte.
struct GlyphCache {
    /// Map from glyph bytes to their index in the cache
    map: HashMap<Box<[u8]>, u32>,
    /// Storage for glyph strings (stored contiguously with NUL separators)
    /// Each entry's index points to the start of its string in this buffer
    strings: Vec<u8>,
    /// Number of entries (for index allocation)
    count: u32,
}

impl GlyphCache {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            strings: Vec::new(),
            count: 0,
        }
    }

    /// Insert a glyph into the cache, returning its index.
    ///
    /// If the glyph already exists, returns its existing index.
    fn insert(&mut self, bytes: &[u8]) -> u32 {
        // Check if already cached
        if let Some(&idx) = self.map.get(bytes) {
            return idx;
        }

        // Store the string and get its index
        #[allow(clippy::cast_possible_truncation)] // Cache can't exceed 2^24 entries
        let idx = self.strings.len() as u32;

        // Store the bytes followed by NUL
        self.strings.extend_from_slice(bytes);
        self.strings.push(0); // NUL terminator

        // Add to map
        self.map.insert(bytes.into(), idx);
        self.count += 1;

        idx
    }

    /// Get a glyph string by index.
    ///
    /// Returns the bytes (without NUL terminator) or None if out of bounds.
    fn get(&self, idx: u32) -> Option<&[u8]> {
        let start = idx as usize;
        if start >= self.strings.len() {
            return None;
        }

        // Find the NUL terminator
        let end = self.strings[start..]
            .iter()
            .position(|&b| b == 0)
            .map(|pos| start + pos)?;

        Some(&self.strings[start..end])
    }

    /// Check if the cache has exceeded the clear threshold.
    const fn is_full(&self) -> bool {
        self.count as usize > GLYPH_CACHE_CLEAR_THRESHOLD
    }

    /// Clear the cache completely.
    fn clear(&mut self) {
        self.map.clear();
        self.strings.clear();
        self.count = 0;
    }

    /// Get the number of entries in the cache.
    #[allow(dead_code)]
    const fn len(&self) -> u32 {
        self.count
    }
}

/// Global glyph cache instance.
///
/// Uses a Mutex for thread-safety, though Neovim is primarily single-threaded.
/// `LazyLock` ensures the cache is initialized on first access.
static GLYPH_CACHE: LazyLock<Mutex<GlyphCache>> =
    LazyLock::new(|| Mutex::new(GlyphCache::new()));

// FFI declarations for C callback functions
extern "C" {
    /// Called when glyph cache is cleared to invalidate decoration glyphs.
    fn nvim_decor_check_invalid_glyphs();

    /// Called when glyph cache is cleared to regenerate char options.
    /// Returns non-zero on error (which should never happen).
    fn nvim_check_chars_options() -> c_int;
}

/// Check if a screen character is stored in the high (cache) format.
///
/// Returns true if the schar uses the glyph cache format (high byte is 0xFF).
/// The format depends on endianness:
/// - Big endian: high bit is in first byte position
/// - Little endian: high bit is in last byte position (position 0 in memory)
#[inline]
const fn schar_high_impl(sc: ScharT) -> bool {
    // On little-endian systems (most common), check if lowest byte is 0xFF
    // On big-endian systems, check if highest byte is 0xFF
    #[cfg(target_endian = "big")]
    {
        (sc & 0xFF00_0000) == 0xFF00_0000
    }
    #[cfg(target_endian = "little")]
    {
        (sc & 0xFF) == 0xFF
    }
}

/// FFI wrapper for `schar_high`.
///
/// Check if a screen character is stored in the high (cache) format.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_schar_high(sc: ScharT) -> bool {
    schar_high_impl(sc)
}

/// Get ASCII character from an schar, or NUL if not ASCII.
///
/// Returns the ASCII character if the schar represents a single ASCII byte,
/// otherwise returns NUL (0). The check and extraction depend on endianness.
#[inline]
#[allow(clippy::cast_possible_truncation)] // intentionally truncating to char
const fn schar_get_ascii_impl(sc: ScharT) -> i8 {
    #[cfg(target_endian = "big")]
    {
        // On big-endian: check if lower 3 bytes are 0 and high byte < 0x80
        if (sc & 0x80FF_FFFF) == 0 {
            // Extract the high byte as ASCII char
            (sc >> 24) as i8
        } else {
            0
        }
    }
    #[cfg(target_endian = "little")]
    {
        // On little-endian: check if value < 0x80 (fits in one ASCII byte)
        if sc < 0x80 {
            sc as i8
        } else {
            0
        }
    }
}

/// FFI wrapper for `schar_get_ascii`.
///
/// Get ASCII character from an schar, or NUL if not ASCII.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_schar_get_ascii(sc: ScharT) -> i8 {
    schar_get_ascii_impl(sc)
}

/// Put a unicode character in a screen cell.
///
/// Converts a Unicode codepoint to an `schar_T` by encoding it as UTF-8
/// and storing the bytes in the u32 value.
/// Characters >= 0x200000 are replaced with the Unicode replacement character (0xFFFD).
#[inline]
fn schar_from_char_impl(c: c_int) -> ScharT {
    let c = if c >= 0x20_0000 { REPLACEMENT_CHAR } else { c };

    // Write UTF-8 bytes into a buffer
    let mut buf = [0u8; 4];
    nvim_mbyte::utf_char2bytes(c, &mut buf);

    // Convert the buffer to schar_T (native endianness)
    // On little-endian: first UTF-8 byte goes to lowest byte of u32
    // On big-endian: first UTF-8 byte goes to highest byte of u32
    ScharT::from_ne_bytes(buf)
}

/// FFI wrapper for `schar_from_char`.
///
/// Put a unicode character in a screen cell.
#[no_mangle]
pub extern "C" fn rs_schar_from_char(c: c_int) -> ScharT {
    schar_from_char_impl(c)
}

/// Pack RGB values into a single 24-bit integer.
///
/// The format is 0x00RRGGBB:
/// - Red in bits 16-23
/// - Green in bits 8-15
/// - Blue in bits 0-7
///
/// # Arguments
/// * `r` - Red component (0-255)
/// * `g` - Green component (0-255)
/// * `b` - Blue component (0-255)
///
/// # Returns
/// Packed RGB value as a 32-bit integer
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_rgb(r: c_int, g: c_int, b: c_int) -> c_int {
    // Original C: (((r) << 16) | ((g) << 8) | (b))
    ((r & 0xFF) << 16) | ((g & 0xFF) << 8) | (b & 0xFF)
}

// =============================================================================
// Phase 25: schar_T Core Functions
// =============================================================================

/// Extract the cache index from a "high" schar.
///
/// For schars stored in the glyph cache, the index is stored in the upper 24 bits
/// (on little-endian) or lower 24 bits (on big-endian).
#[inline]
const fn schar_idx(sc: ScharT) -> u32 {
    #[cfg(target_endian = "big")]
    {
        sc & 0x00FF_FFFF
    }
    #[cfg(target_endian = "little")]
    {
        sc >> 8
    }
}

/// Convert an schar to its UTF-8 bytes, writing to a buffer.
///
/// For inline schars (<=4 bytes), extracts directly from the u32.
/// For high schars, reads from the Rust glyph cache.
///
/// Returns the number of bytes written (not including any NUL).
/// Does NOT write a terminating NUL.
fn schar_get_bytes(sc: ScharT, buf: &mut [u8]) -> usize {
    if schar_high_impl(sc) {
        // Read from Rust glyph cache
        let idx = schar_idx(sc);
        let cache = GLYPH_CACHE.lock().unwrap();
        cache.get(idx).map_or(0, |bytes| {
            let len = bytes.len().min(buf.len());
            buf[..len].copy_from_slice(&bytes[..len]);
            len
        })
    } else {
        // Inline schar: extract bytes from u32
        let bytes = sc.to_ne_bytes();
        // Find length by looking for first NUL or end of 4 bytes
        let len = bytes.iter().position(|&b| b == 0).unwrap_or(4);
        let copy_len = len.min(buf.len());
        buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
        copy_len
    }
}

/// Convert a NUL-terminated string to an schar.
///
/// This is a wrapper around `schar_from_buf_impl` that handles NULL input.
#[inline]
fn schar_from_str_impl(str_ptr: *const c_char) -> ScharT {
    if str_ptr.is_null() {
        return 0;
    }
    // SAFETY: str_ptr is not null and is expected to be NUL-terminated
    let cstr = unsafe { CStr::from_ptr(str_ptr) };
    schar_from_buf_impl(cstr.to_bytes())
}

/// Convert a byte slice to an schar.
///
/// For <=4 bytes, stores inline in the u32.
/// For >4 bytes, stores in the Rust glyph cache and returns index with 0xFF marker.
#[inline]
fn schar_from_buf_impl(bytes: &[u8]) -> ScharT {
    debug_assert!(bytes.len() < MAX_SCHAR_SIZE);

    if bytes.len() <= 4 {
        // Store inline
        let mut sc_bytes = [0u8; 4];
        sc_bytes[..bytes.len()].copy_from_slice(bytes);
        ScharT::from_ne_bytes(sc_bytes)
    } else {
        // Store in glyph cache
        let idx = GLYPH_CACHE.lock().unwrap().insert(bytes);
        debug_assert!(idx < 0x00FF_FFFF);

        // Encode index with 0xFF marker
        #[cfg(target_endian = "big")]
        {
            idx + (0xFF << 24)
        }
        #[cfg(target_endian = "little")]
        {
            0xFF + (idx << 8)
        }
    }
}

/// FFI wrapper for `schar_from_str`.
///
/// Convert a NUL-terminated string to an `schar_T`.
#[no_mangle]
pub extern "C" fn rs_schar_from_str(str_ptr: *const c_char) -> ScharT {
    schar_from_str_impl(str_ptr)
}

/// FFI wrapper for `schar_from_buf`.
///
/// Convert a byte buffer to an `schar_T`.
/// The buffer need not be NUL-terminated but must not contain embedded NULs.
/// Caller must ensure `len < MAX_SCHAR_SIZE`.
///
/// # Safety
/// - `buf` must be valid for reading `len` bytes
/// - `len` must be less than `MAX_SCHAR_SIZE`
#[no_mangle]
pub unsafe extern "C" fn rs_schar_from_buf(buf: *const c_char, len: usize) -> ScharT {
    debug_assert!(len < MAX_SCHAR_SIZE);
    if buf.is_null() || len == 0 {
        return 0;
    }
    // SAFETY: caller guarantees buf is valid for len bytes
    let bytes = std::slice::from_raw_parts(buf.cast::<u8>(), len);
    schar_from_buf_impl(bytes)
}

/// Check if cache is full, and if so, clear it.
///
/// Returns true if cache was cleared.
fn schar_cache_clear_if_full_impl() -> bool {
    let is_full = {
        let cache = GLYPH_CACHE.lock().unwrap();
        cache.is_full()
    };

    if is_full {
        schar_cache_clear_impl();
        true
    } else {
        false
    }
}

/// FFI wrapper for `schar_cache_clear_if_full`.
///
/// Check if cache is full, and if so, clear it.
/// Returns true if cache was cleared.
#[no_mangle]
pub extern "C" fn rs_schar_cache_clear_if_full() -> bool {
    schar_cache_clear_if_full_impl()
}

/// Clear the glyph cache completely.
fn schar_cache_clear_impl() {
    // Call C callbacks before clearing
    // SAFETY: these C functions have no safety requirements
    unsafe {
        nvim_decor_check_invalid_glyphs();
    }

    // Clear the cache
    {
        let mut cache = GLYPH_CACHE.lock().unwrap();
        cache.clear();
    }

    // Regenerate char options (must not fail)
    // SAFETY: this C function has no safety requirements
    let result = unsafe { nvim_check_chars_options() };
    assert!(result == 0, "check_chars_options() failed after cache clear");
}

/// FFI wrapper for `schar_cache_clear`.
///
/// Clear the glyph cache completely.
#[no_mangle]
pub extern "C" fn rs_schar_cache_clear() {
    schar_cache_clear_impl();
}

// =============================================================================
// Phase 27: schar_get Functions
// =============================================================================

/// FFI wrapper for `schar_get`.
///
/// Convert an `schar_T` to a NUL-terminated UTF-8 string.
/// Writes to `buf_out` and sets final NUL.
///
/// # Safety
/// - `buf_out` must be valid for writing at least `MAX_SCHAR_SIZE` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_schar_get(buf_out: *mut c_char, sc: ScharT) -> usize {
    debug_assert!(!buf_out.is_null());
    // SAFETY: caller guarantees buf_out is valid for MAX_SCHAR_SIZE bytes
    let buf = std::slice::from_raw_parts_mut(buf_out.cast::<u8>(), MAX_SCHAR_SIZE);
    let len = schar_get_bytes(sc, buf);
    // Set final NUL
    buf[len] = 0;
    len
}

/// FFI wrapper for `schar_get_adv`.
///
/// Convert an `schar_T` to UTF-8 bytes, advancing the buffer pointer.
/// Does NOT set final NUL.
///
/// # Safety
/// - `buf_out` must point to a valid `*mut c_char` pointer
/// - The pointed-to buffer must be valid for writing at least `MAX_SCHAR_SIZE` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_schar_get_adv(buf_out: *mut *mut c_char, sc: ScharT) -> usize {
    debug_assert!(!buf_out.is_null());
    debug_assert!(!(*buf_out).is_null());
    // SAFETY: caller guarantees buf_out points to valid pointer with enough space
    let ptr = *buf_out;
    let buf = std::slice::from_raw_parts_mut(ptr.cast::<u8>(), MAX_SCHAR_SIZE);
    let len = schar_get_bytes(sc, buf);
    // Advance the pointer
    *buf_out = ptr.add(len);
    len
}

/// Get the byte length of an schar's UTF-8 content.
///
/// For inline schars, counts bytes until NUL or end of 4 bytes.
/// For high schars, reads from Rust glyph cache.
#[inline]
fn schar_len_impl(sc: ScharT) -> usize {
    if schar_high_impl(sc) {
        let idx = schar_idx(sc);
        let cache = GLYPH_CACHE.lock().unwrap();
        cache.get(idx).map_or(0, <[u8]>::len)
    } else {
        // Inline: find first NUL byte
        let bytes = sc.to_ne_bytes();
        bytes.iter().position(|&b| b == 0).unwrap_or(4)
    }
}

/// FFI wrapper for `schar_len`.
///
/// Get the byte length of an schar's UTF-8 content.
#[no_mangle]
pub extern "C" fn rs_schar_len(sc: ScharT) -> usize {
    schar_len_impl(sc)
}

/// Get the display width (in cells) of an schar.
///
/// Returns 1 for ASCII and most characters, 2 for wide CJK characters.
#[inline]
fn schar_cells_impl(sc: ScharT) -> c_int {
    // Hot path: ASCII characters have width 1
    #[cfg(target_endian = "big")]
    {
        if (sc & 0x80FF_FFFF) == 0 {
            return 1;
        }
    }
    #[cfg(target_endian = "little")]
    {
        if sc < 0x80 {
            return 1;
        }
    }

    // For non-ASCII, get the bytes and use utf_ptr2cells
    let mut buf = [0u8; MAX_SCHAR_SIZE];
    let len = schar_get_bytes(sc, &mut buf);
    if len == 0 {
        return 1;
    }

    // Use nvim_mbyte's utf_ptr2cells
    nvim_mbyte::utf_ptr2cells(&buf[..len])
}

/// FFI wrapper for `schar_cells`.
///
/// Get the display width (in cells) of an schar.
#[no_mangle]
pub extern "C" fn rs_schar_cells(sc: ScharT) -> c_int {
    schar_cells_impl(sc)
}

/// Get the first Unicode codepoint from an schar.
#[inline]
fn schar_get_first_codepoint_impl(sc: ScharT) -> c_int {
    let mut buf = [0u8; MAX_SCHAR_SIZE];
    let len = schar_get_bytes(sc, &mut buf);
    if len == 0 {
        return 0;
    }
    nvim_mbyte::utf_ptr2char(&buf[..len])
}

/// FFI wrapper for `schar_get_first_codepoint`.
///
/// Get the first Unicode codepoint from an schar.
#[no_mangle]
pub extern "C" fn rs_schar_get_first_codepoint(sc: ScharT) -> c_int {
    schar_get_first_codepoint_impl(sc)
}

// =============================================================================
// Phase 30: Grid Line Content Functions
// =============================================================================

/// Type alias for screen attribute (matches C's `sattr_T` which is `int16_t`).
type SattrT = i16;

/// Type alias for column number (matches C's `colnr_T` which is `int32_t`).
type ColnrT = i32;

/// Type alias for handle (matches C's `handle_T` which is `int`).
type HandleT = c_int;

// C accessor functions for line buffer arrays and grid line state
extern "C" {
    // Line buffer accessors
    fn nvim_get_linebuf_char() -> *mut ScharT;
    fn nvim_get_linebuf_attr() -> *mut SattrT;
    fn nvim_get_linebuf_vcol() -> *mut ColnrT;
    fn nvim_get_linebuf_size() -> usize;

    // Grid line state accessors
    fn nvim_get_grid_line_grid() -> *mut std::ffi::c_void;
    fn nvim_get_grid_line_row() -> c_int;
    fn nvim_get_grid_line_maxcol() -> c_int;
    fn nvim_get_grid_line_first() -> c_int;
    fn nvim_set_grid_line_first(first: c_int);
    fn nvim_get_grid_line_last() -> c_int;
    fn nvim_set_grid_line_last(last: c_int);
    fn nvim_get_grid_line_clear_to() -> c_int;
    fn nvim_set_grid_line_clear_to(clear_to: c_int);
    fn nvim_set_grid_line_bg_attr(bg_attr: c_int);
    fn nvim_set_grid_line_clear_attr(clear_attr: c_int);

    // UI function
    fn nvim_ui_grid_cursor_goto(grid_handle: HandleT, row: c_int, col: c_int);

    // ScreenGrid field accessor (we need the handle field)
    fn nvim_screengrid_get_handle(grid: *mut std::ffi::c_void) -> HandleT;
    fn nvim_screengrid_get_rows(grid: *mut std::ffi::c_void) -> c_int;

    // For grid_line_flush
    fn nvim_get_grid_line_coloff() -> c_int;
    fn nvim_get_grid_line_bg_attr() -> c_int;
    fn nvim_get_grid_line_clear_attr() -> c_int;
    fn nvim_get_grid_line_flags() -> c_int;
    fn nvim_set_grid_line_grid(grid: *mut std::ffi::c_void);

    // rdb_flags global
    fn nvim_get_rdb_flags() -> c_uint;

    // ScreenGrid array accessors
    fn nvim_screengrid_get_chars(grid: *mut std::ffi::c_void) -> *mut ScharT;
    fn nvim_screengrid_get_attrs(grid: *mut std::ffi::c_void) -> *mut SattrT;
    fn nvim_screengrid_get_vcols(grid: *mut std::ffi::c_void) -> *mut ColnrT;
    fn nvim_screengrid_get_line_offset(grid: *mut std::ffi::c_void) -> *mut usize;
    fn nvim_screengrid_get_dirty_col(grid: *mut std::ffi::c_void) -> *mut c_int;

    // ScreenGrid field accessors
    fn nvim_screengrid_get_cols(grid: *mut std::ffi::c_void) -> c_int;
    fn nvim_screengrid_get_throttled(grid: *mut std::ffi::c_void) -> bool;

    // Global accessors
    fn nvim_get_default_grid() -> *mut std::ffi::c_void;
    fn nvim_get_exmode_active() -> bool;
    fn nvim_get_p_arshape() -> c_int;
    fn nvim_get_p_tbidi() -> c_int;

    // Function wrappers
    fn nvim_line_do_arabic_shape(buf: *mut ScharT, cols: c_int);
    fn nvim_ui_line(
        grid: *mut std::ffi::c_void,
        row: c_int,
        invalid_row: bool,
        startcol: c_int,
        endcol: c_int,
        clearcol: c_int,
        clearattr: c_int,
        wrap: bool,
    );

    // hl_combine_attr from highlight module
    fn rs_hl_combine_attr(char_attr: c_int, prim_attr: c_int) -> c_int;

    // Arabic shaping function
    fn nvim_arabic_shape(
        c: c_int,
        c1p: *mut c_int,
        prev_c: c_int,
        prev_c1: c_int,
        next_c: c_int,
    ) -> c_int;
}

/// Put a single schar at a column position.
///
/// # Safety
/// Must be called after `grid_line_start()` and before `grid_line_flush()`.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_line_put_schar(col: c_int, schar: ScharT, attr: c_int) {
    let grid = nvim_get_grid_line_grid();
    debug_assert!(!grid.is_null());

    let maxcol = nvim_get_grid_line_maxcol();
    if col >= maxcol {
        return;
    }

    let linebuf_char = nvim_get_linebuf_char();
    let linebuf_attr = nvim_get_linebuf_attr();
    let linebuf_vcol = nvim_get_linebuf_vcol();

    *linebuf_char.offset(col as isize) = schar;
    #[allow(clippy::cast_possible_truncation)]
    {
        *linebuf_attr.offset(col as isize) = attr as SattrT;
    }
    *linebuf_vcol.offset(col as isize) = -1;

    let first = nvim_get_grid_line_first();
    let last = nvim_get_grid_line_last();
    nvim_set_grid_line_first(first.min(col));
    nvim_set_grid_line_last(last.max(col + 1));
}

/// Fill a range of columns with a single schar.
///
/// # Safety
/// Must be called after `grid_line_start()` and before `grid_line_flush()`.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_line_fill(
    start_col: c_int,
    end_col: c_int,
    sc: ScharT,
    attr: c_int,
) -> c_int {
    let maxcol = nvim_get_grid_line_maxcol();
    let end_col = end_col.min(maxcol);

    if start_col >= end_col {
        return end_col;
    }

    let linebuf_char = nvim_get_linebuf_char();
    let linebuf_attr = nvim_get_linebuf_attr();
    let linebuf_vcol = nvim_get_linebuf_vcol();

    #[allow(clippy::cast_possible_truncation)]
    let attr_val = attr as SattrT;

    for col in start_col..end_col {
        *linebuf_char.offset(col as isize) = sc;
        *linebuf_attr.offset(col as isize) = attr_val;
        *linebuf_vcol.offset(col as isize) = -1;
    }

    let first = nvim_get_grid_line_first();
    let last = nvim_get_grid_line_last();
    nvim_set_grid_line_first(first.min(start_col));
    nvim_set_grid_line_last(last.max(end_col));

    end_col
}

/// Set the clear range for the current line.
///
/// # Safety
/// Must be called after `grid_line_start()` and before `grid_line_flush()`.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_line_clear_end(
    start_col: c_int,
    end_col: c_int,
    bg_attr: c_int,
    clear_attr: c_int,
) {
    let first = nvim_get_grid_line_first();
    if first > start_col {
        nvim_set_grid_line_first(start_col);
        nvim_set_grid_line_last(start_col);
    }
    nvim_set_grid_line_clear_to(end_col);
    nvim_set_grid_line_bg_attr(bg_attr);
    nvim_set_grid_line_clear_attr(clear_attr);
}

/// Move the cursor to a position in the currently rendered line.
///
/// # Safety
/// Must be called after `grid_line_start()` and before `grid_line_flush()`.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_line_cursor_goto(col: c_int) {
    let grid = nvim_get_grid_line_grid();
    let handle = nvim_screengrid_get_handle(grid);
    let row = nvim_get_grid_line_row();
    nvim_ui_grid_cursor_goto(handle, row, col);
}

// =============================================================================
// Phase 31: Grid Line Flush Functions
// =============================================================================

/// rdb_flags value for kOptRdbFlagInvalid (0x04)
const K_OPT_RDB_FLAG_INVALID: c_uint = 0x04;

/// Flush the current grid line to the UI.
///
/// This commits the line buffer to the grid and sends UI updates.
/// Calls `grid_put_linebuf` (which remains in C for now).
///
/// # Safety
/// Must be called after `grid_line_start()` with an active grid line.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_line_flush() {
    let grid = nvim_get_grid_line_grid();
    // Clear grid_line_grid to indicate flush is done
    nvim_set_grid_line_grid(std::ptr::null_mut());

    let first = nvim_get_grid_line_first();
    let last = nvim_get_grid_line_last();
    let clear_to = nvim_get_grid_line_clear_to();
    let maxcol = nvim_get_grid_line_maxcol();

    // grid_line_clear_to = MAX(grid_line_last, grid_line_clear_to)
    let clear_to = clear_to.max(last);
    nvim_set_grid_line_clear_to(clear_to);

    debug_assert!(clear_to <= maxcol);

    // Early exit if nothing to flush
    if first >= clear_to {
        return;
    }

    let row = nvim_get_grid_line_row();
    let coloff = nvim_get_grid_line_coloff();
    let bg_attr = nvim_get_grid_line_bg_attr();
    let clear_attr = nvim_get_grid_line_clear_attr();
    let flags = nvim_get_grid_line_flags();

    // Call Rust implementation directly (not through C wrapper to avoid recursion)
    rs_grid_put_linebuf(
        grid, row, coloff, first, last, clear_to, bg_attr, clear_attr, -1, flags,
    );
}

/// Flush grid line but only if on a valid row.
///
/// This is a stopgap until message.c has been refactored to behave.
/// If the row is invalid and kOptRdbFlagInvalid is set, aborts.
///
/// # Safety
/// Must be called after `grid_line_start()`.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_line_flush_if_valid_row() {
    let grid = nvim_get_grid_line_grid();
    let row = nvim_get_grid_line_row();
    let grid_rows = nvim_screengrid_get_rows(grid);

    if row < 0 || row >= grid_rows {
        let rdb_flags = nvim_get_rdb_flags();
        if (rdb_flags & K_OPT_RDB_FLAG_INVALID) != 0 {
            // In debug/invalid mode, abort on invalid row
            std::process::abort();
        } else {
            // Clear grid_line_grid and return without flushing
            nvim_set_grid_line_grid(std::ptr::null_mut());
            return;
        }
    }

    rs_grid_line_flush();
}

// =============================================================================
// Phase 32: grid_put_linebuf Implementation
// =============================================================================

/// SLF_RIGHTLEFT flag (0x01)
const SLF_RIGHTLEFT: c_int = 1;
/// SLF_WRAP flag (0x02)
const SLF_WRAP: c_int = 2;
/// SLF_INC_VCOL flag (0x04)
const SLF_INC_VCOL: c_int = 4;

/// rdb_flags value for kOptRdbFlagNodelta (0x08)
const K_OPT_RDB_FLAG_NODELTA: c_uint = 0x08;

/// Check whether the given character needs redrawing.
///
/// Returns true if the character at `col` differs from what's in the grid
/// or if forced redraw flags are set.
#[inline]
unsafe fn grid_char_needs_redraw(
    linebuf_char: *const ScharT,
    linebuf_attr: *const SattrT,
    grid_chars: *const ScharT,
    grid_attrs: *const SattrT,
    col: c_int,
    off_to: usize,
    cols: c_int,
) -> bool {
    if cols <= 0 {
        return false;
    }

    let col_idx = col as isize;
    let off = off_to;

    // Check if char or attr differs
    if *linebuf_char.offset(col_idx) != *grid_chars.add(off)
        || *linebuf_attr.offset(col_idx) != *grid_attrs.add(off)
    {
        return true;
    }

    // Check second cell of double-width char
    if cols > 1
        && *linebuf_char.offset(col_idx + 1) == 0
        && *linebuf_char.offset(col_idx + 1) != *grid_chars.add(off + 1)
    {
        return true;
    }

    // Force redraw in exmode or with nodelta flag
    if nvim_get_exmode_active() || (nvim_get_rdb_flags() & K_OPT_RDB_FLAG_NODELTA) != 0 {
        return true;
    }

    false
}

/// Check if a grid row is invalid (marked for full redraw).
#[inline]
unsafe fn grid_invalid_row(grid_attrs: *const SattrT, line_offset: usize) -> bool {
    // A row is invalid if its first attribute is negative
    *grid_attrs.add(line_offset) < 0
}

/// Move one buffered line to the window grid, but only the characters that
/// have actually changed.
///
/// This is the core rendering function that handles:
/// - Delta detection (only redraw changed characters)
/// - Double-width character handling
/// - Right-to-left text support
/// - Arabic shaping
/// - Attribute combination
///
/// # Safety
/// - `grid` must be a valid ScreenGrid pointer
/// - All grid arrays must be valid and properly sized
#[no_mangle]
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub unsafe extern "C" fn rs_grid_put_linebuf(
    grid: *mut std::ffi::c_void,
    row: c_int,
    coloff: c_int,
    mut col: c_int,
    mut endcol: c_int,
    mut clear_width: c_int,
    bg_attr: c_int,
    mut clear_attr: c_int,
    mut last_vcol: ColnrT,
    flags: c_int,
) {
    debug_assert!(row >= 0 && row < nvim_screengrid_get_rows(grid));

    let grid_cols = nvim_screengrid_get_cols(grid);

    // Clamp endcol to grid width
    if endcol > grid_cols {
        endcol = grid_cols;
    }

    // Get grid arrays
    let grid_chars = nvim_screengrid_get_chars(grid);
    let grid_attrs = nvim_screengrid_get_attrs(grid);
    let grid_vcols = nvim_screengrid_get_vcols(grid);
    let grid_line_offset = nvim_screengrid_get_line_offset(grid);

    // Safety check
    if grid_chars.is_null() || row >= nvim_screengrid_get_rows(grid) || coloff >= grid_cols {
        return;
    }

    // Get line buffers
    let linebuf_char = nvim_get_linebuf_char();
    let linebuf_attr = nvim_get_linebuf_attr();
    let linebuf_vcol = nvim_get_linebuf_vcol();

    // Check if this is an invalid row (needs full redraw)
    let default_grid = nvim_get_default_grid();
    let line_off = *grid_line_offset.add(row as usize);
    let invalid_row =
        grid != default_grid && grid_invalid_row(grid_attrs, line_off) && col == 0;

    let off_to = line_off + coloff as usize;
    let max_off_to = line_off + grid_cols as usize;

    // Handle overwriting right half of double-width character
    if col > 0 && *grid_chars.add(off_to + col as usize) == 0 {
        *linebuf_char.offset((col - 1) as isize) = schar_from_char_impl(b'>' as c_int);
        *linebuf_attr.offset((col - 1) as isize) =
            *grid_attrs.add(off_to + (col - 1) as usize);
        col -= 1;
    }

    // Handle right-to-left mode
    let mut clear_start = endcol;
    if (flags & SLF_RIGHTLEFT) != 0 {
        clear_start = col;
        col = endcol;
        endcol = clear_width;
        clear_width = col;
    }

    // Apply Arabic shaping if enabled
    if nvim_get_p_arshape() != 0 && nvim_get_p_tbidi() == 0 && endcol > col {
        nvim_line_do_arabic_shape(linebuf_char.offset(col as isize), endcol - col);
    }

    // Combine background attribute with line attributes
    if bg_attr != 0 {
        for c in col..endcol {
            let attr = *linebuf_attr.offset(c as isize) as c_int;
            #[allow(clippy::cast_possible_truncation)]
            {
                *linebuf_attr.offset(c as isize) = rs_hl_combine_attr(bg_attr, attr) as SattrT;
            }
        }
    }

    // Initialize dirty range tracking
    let mut start_dirty: c_int = -1;
    let mut end_dirty: c_int = 0;
    let mut clear_next = false;

    // Check if first character needs redraw
    let mut redraw_next = grid_char_needs_redraw(
        linebuf_char,
        linebuf_attr,
        grid_chars,
        grid_attrs,
        col,
        off_to + col as usize,
        endcol - col,
    );

    // Process each character
    while col < endcol {
        // Determine if this is a double-width character
        let char_cells = if col + 1 < endcol && *linebuf_char.offset((col + 1) as isize) == 0 {
            2
        } else {
            1
        };

        let redraw_this = redraw_next;
        let off = off_to + col as usize;

        // Check if next character needs redraw
        redraw_next = grid_char_needs_redraw(
            linebuf_char,
            linebuf_attr,
            grid_chars,
            grid_attrs,
            col + char_cells,
            off + char_cells as usize,
            endcol - col - char_cells,
        );

        if redraw_this {
            if start_dirty == -1 {
                start_dirty = col;
            }
            end_dirty = col + char_cells;

            // Check if we need to clear the right half of an old double-width char
            if col + char_cells == endcol
                && (off + char_cells as usize) < max_off_to
                && *grid_chars.add(off + char_cells as usize) == 0
            {
                clear_next = true;
            }

            // Copy character to grid
            *grid_chars.add(off) = *linebuf_char.offset(col as isize);
            if char_cells == 2 {
                *grid_chars.add(off + 1) = *linebuf_char.offset((col + 1) as isize);
            }

            // Copy attributes to grid
            *grid_attrs.add(off) = *linebuf_attr.offset(col as isize);
            if char_cells == 2 {
                *grid_attrs.add(off + 1) = *linebuf_attr.offset(col as isize);
            }
        }

        // Always update vcols
        *grid_vcols.add(off) = *linebuf_vcol.offset(col as isize);
        if char_cells == 2 {
            *grid_vcols.add(off + 1) = *linebuf_vcol.offset((col + 1) as isize);
        }

        col += char_cells;
    }

    // Clear right half of overwritten double-width character
    if clear_next {
        *grid_chars.add(off_to + col as usize) = schar_from_char_impl(b' ' as c_int);
        end_dirty += 1;
    }

    // Handle double-width character at clear boundary
    if (off_to + clear_width as usize) < max_off_to
        && *grid_chars.add(off_to + clear_width as usize) == 0
    {
        clear_width += 1;
    }

    // Update vcols in RTL clear region
    let mut clear_dirty_start: c_int = -1;
    let mut clear_end: c_int = -1;

    if (flags & SLF_RIGHTLEFT) != 0 {
        for c in (clear_start..clear_width).rev() {
            let off = off_to + c as usize;
            if (flags & SLF_INC_VCOL) != 0 {
                last_vcol += 1;
                *grid_vcols.add(off) = last_vcol;
            } else {
                *grid_vcols.add(off) = last_vcol;
            }
        }
    }

    // Combine clear_attr with bg_attr
    clear_attr = rs_hl_combine_attr(bg_attr, clear_attr);

    // Clear the rest of the line
    let space_char = schar_from_char_impl(b' ' as c_int);
    #[allow(clippy::cast_possible_truncation)]
    let clear_attr_val = clear_attr as SattrT;

    for c in clear_start..clear_width {
        let off = off_to + c as usize;
        if *grid_chars.add(off) != space_char
            || *grid_attrs.add(off) != clear_attr_val
            || (nvim_get_rdb_flags() & K_OPT_RDB_FLAG_NODELTA) != 0
        {
            *grid_chars.add(off) = space_char;
            *grid_attrs.add(off) = clear_attr_val;
            if clear_dirty_start == -1 {
                clear_dirty_start = c;
            }
            clear_end = c + 1;
        }
        if (flags & SLF_RIGHTLEFT) == 0 {
            if (flags & SLF_INC_VCOL) != 0 {
                last_vcol += 1;
                *grid_vcols.add(off) = last_vcol;
            } else {
                *grid_vcols.add(off) = last_vcol;
            }
        }
    }

    // Determine final dirty range and send to UI
    if (flags & SLF_RIGHTLEFT) != 0 && start_dirty != -1 && clear_dirty_start != -1 {
        let throttled = nvim_screengrid_get_throttled(grid);
        if throttled || clear_dirty_start >= start_dirty - 5 {
            // Cannot draw now or too small to be worth a separate "clear" event
            start_dirty = clear_dirty_start;
        } else {
            nvim_ui_line(
                grid,
                row,
                invalid_row,
                coloff + clear_dirty_start,
                coloff + clear_dirty_start,
                coloff + clear_end,
                clear_attr,
                (flags & SLF_WRAP) != 0,
            );
        }
        clear_end = end_dirty;
    } else {
        if start_dirty == -1 {
            // Clear only
            start_dirty = clear_dirty_start;
            end_dirty = clear_dirty_start;
        } else if clear_end < end_dirty {
            // Put only
            clear_end = end_dirty;
        } else {
            end_dirty = endcol;
        }
    }

    // Send final UI update
    if clear_end > start_dirty {
        let throttled = nvim_screengrid_get_throttled(grid);
        if !throttled {
            nvim_ui_line(
                grid,
                row,
                invalid_row,
                coloff + start_dirty,
                coloff + end_dirty,
                coloff + clear_end,
                clear_attr,
                (flags & SLF_WRAP) != 0,
            );
        } else {
            let dirty_col = nvim_screengrid_get_dirty_col(grid);
            if !dirty_col.is_null() {
                if clear_end > *dirty_col.add(row as usize) {
                    *dirty_col.add(row as usize) = clear_end;
                }
            }
        }
    }
}

// =============================================================================
// Phase 33: Arabic Shaping
// =============================================================================

/// Arabic character range check: ((ch) & 0xFF00) == 0x0600
#[inline]
const fn arabic_char(ch: c_int) -> bool {
    (ch & 0xFF00) == 0x0600
}

/// Check if first byte indicates Arabic block (0xD8 or 0xD9)
#[inline]
fn schar_in_arabic_block(sc: ScharT) -> bool {
    let first_byte = if schar_high_impl(sc) {
        // High schar: read from cache
        let idx = schar_idx(sc);
        let cache = GLYPH_CACHE.lock().unwrap();
        cache.get(idx).map_or(0, |bytes| bytes.first().copied().unwrap_or(0))
    } else {
        // Inline schar: extract first byte
        let bytes = sc.to_ne_bytes();
        bytes[0]
    };
    (first_byte & 0xFE) == 0xD8
}

/// Get the first two Unicode codepoints from an schar.
#[inline]
fn schar_get_first_two_codepoints(sc: ScharT) -> (c_int, c_int) {
    let mut buf = [0u8; MAX_SCHAR_SIZE];
    let len = schar_get_bytes(sc, &mut buf);

    if len == 0 {
        return (0, 0);
    }

    let c0 = nvim_mbyte::utf_ptr2char(&buf[..len]);
    if c0 == 0 {
        return (0, 0);
    }

    let c0_len = nvim_mbyte::utf_char2len(c0);
    let c1 = if c0_len < len {
        nvim_mbyte::utf_ptr2char(&buf[c0_len..len])
    } else {
        0
    };

    (c0, c1)
}

/// Apply Arabic shaping to a line buffer.
///
/// This function modifies characters in the buffer to apply contextual
/// Arabic shaping rules.
///
/// # Safety
/// - `buf` must be a valid pointer to `cols` schar_T elements
#[no_mangle]
pub unsafe extern "C" fn rs_line_do_arabic_shape(buf: *mut ScharT, cols: c_int) {
    if buf.is_null() || cols <= 0 {
        return;
    }

    // Find first Arabic character
    let mut i = 0;
    while i < cols {
        if schar_in_arabic_block(*buf.add(i as usize)) {
            break;
        }
        i += 1;
    }

    // No Arabic characters found
    if i >= cols {
        return;
    }

    let mut c0prev: c_int = 0;
    let (mut c0, mut c1) = schar_get_first_two_codepoints(*buf.add(i as usize));

    while i < cols {
        let (c0next, c1next) = if i + 1 < cols {
            schar_get_first_two_codepoints(*buf.add((i + 1) as usize))
        } else {
            (0, 0)
        };

        if !arabic_char(c0) {
            // Not an Arabic character, skip to next
            c0prev = c0;
            c0 = c0next;
            c1 = c1next;
            i += 1;
            continue;
        }

        let mut c1new = c1;
        let c0new = nvim_arabic_shape(c0, &mut c1new, c0next, c1next, c0prev);

        if c0new == c0 && c1new == c1 {
            // Unchanged, skip to next
            c0prev = c0;
            c0 = c0next;
            c1 = c1next;
            i += 1;
            continue;
        }

        // Get original schar bytes
        let mut scbuf = [0u8; MAX_SCHAR_SIZE];
        let _ = schar_get_bytes(*buf.add(i as usize), &mut scbuf);

        // Build new schar with shaped characters
        let mut scbuf_new = [0u8; MAX_SCHAR_SIZE];
        let mut len = nvim_mbyte::utf_char2bytes(c0new, &mut scbuf_new);
        if c1new != 0 {
            len += nvim_mbyte::utf_char2bytes(c1new, &mut scbuf_new[len..]);
        }

        // Calculate offset past the original c0 and c1
        let c0_len = nvim_mbyte::utf_char2len(c0);
        let c1_len = if c1 != 0 { nvim_mbyte::utf_char2len(c1) } else { 0 };
        let off = c0_len + c1_len;

        // Copy remaining bytes from original schar
        let rest_start = off;
        let mut rest_len = 0;
        while rest_start + rest_len < MAX_SCHAR_SIZE && scbuf[rest_start + rest_len] != 0 {
            rest_len += 1;
        }

        // Check if result fits, discard a codepoint if too big
        if rest_len + len + 1 > MAX_SCHAR_SIZE {
            // Find last codepoint boundary and remove one
            if rest_len > 0 {
                let rest_slice = &scbuf[rest_start..rest_start + rest_len];
                let bounds = nvim_mbyte::utf_cp_bounds(rest_slice, rest_len - 1);
                rest_len = rest_len.saturating_sub(bounds.begin_off as usize + 1);
            }
        }

        // Copy rest to new buffer
        if rest_len > 0 {
            scbuf_new[len..len + rest_len].copy_from_slice(&scbuf[rest_start..rest_start + rest_len]);
        }

        // Create new schar from buffer
        *buf.add(i as usize) = schar_from_buf_impl(&scbuf_new[..len + rest_len]);

        c0prev = c0;
        c0 = c0next;
        c1 = c1next;
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schar_high_true() {
        // Test values that should return true (cache format)
        #[cfg(target_endian = "little")]
        {
            assert!(schar_high_impl(0x0000_00FF)); // Lowest byte is 0xFF
            assert!(schar_high_impl(0x1234_56FF)); // High bytes set, lowest is 0xFF
            assert!(schar_high_impl(0xFFFF_FFFF)); // All bytes are 0xFF
        }
        #[cfg(target_endian = "big")]
        {
            assert!(schar_high_impl(0xFF00_0000)); // Highest byte is 0xFF
            assert!(schar_high_impl(0xFF12_3456)); // Low bytes set, highest is 0xFF
            assert!(schar_high_impl(0xFFFF_FFFF)); // All bytes are 0xFF
        }
    }

    #[test]
    fn test_schar_high_false() {
        // Test values that should return false (inline format)
        #[cfg(target_endian = "little")]
        {
            assert!(!schar_high_impl(0x0000_0000)); // All zeros
            assert!(!schar_high_impl(0x0000_0041)); // ASCII 'A'
            assert!(!schar_high_impl(0xFFFF_FF00)); // High bytes 0xFF but lowest is 0
            assert!(!schar_high_impl(0x0000_00FE)); // Close to 0xFF but not quite
        }
        #[cfg(target_endian = "big")]
        {
            assert!(!schar_high_impl(0x0000_0000)); // All zeros
            assert!(!schar_high_impl(0x4100_0000)); // ASCII 'A'
            assert!(!schar_high_impl(0x00FF_FFFF)); // Low bytes 0xFF but highest is 0
            assert!(!schar_high_impl(0xFE00_0000)); // Close to 0xFF but not quite
        }
    }

    #[test]
    fn test_ffi_schar_high() {
        #[cfg(target_endian = "little")]
        {
            assert!(rs_schar_high(0x0000_00FF));
            assert!(!rs_schar_high(0x0000_0041));
        }
        #[cfg(target_endian = "big")]
        {
            assert!(rs_schar_high(0xFF00_0000));
            assert!(!rs_schar_high(0x4100_0000));
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)] // b'A' as i8 is safe for ASCII
    fn test_schar_get_ascii_valid() {
        // Test valid ASCII characters
        #[cfg(target_endian = "little")]
        {
            assert_eq!(schar_get_ascii_impl(0x0000_0041), b'A' as i8); // 'A'
            assert_eq!(schar_get_ascii_impl(0x0000_0061), b'a' as i8); // 'a'
            assert_eq!(schar_get_ascii_impl(0x0000_0020), b' ' as i8); // space
            assert_eq!(schar_get_ascii_impl(0x0000_007F), 0x7F_i8); // DEL (max ASCII)
            assert_eq!(schar_get_ascii_impl(0x0000_0000), 0); // NUL
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(schar_get_ascii_impl(0x4100_0000), b'A' as i8); // 'A'
            assert_eq!(schar_get_ascii_impl(0x6100_0000), b'a' as i8); // 'a'
            assert_eq!(schar_get_ascii_impl(0x2000_0000), b' ' as i8); // space
            assert_eq!(schar_get_ascii_impl(0x7F00_0000), 0x7F_i8); // DEL (max ASCII)
            assert_eq!(schar_get_ascii_impl(0x0000_0000), 0); // NUL
        }
    }

    #[test]
    fn test_schar_get_ascii_invalid() {
        // Test non-ASCII characters return NUL
        #[cfg(target_endian = "little")]
        {
            assert_eq!(schar_get_ascii_impl(0x0000_0080), 0); // >= 0x80 is not ASCII
            assert_eq!(schar_get_ascii_impl(0x0000_00FF), 0); // 0xFF is not ASCII
            assert_eq!(schar_get_ascii_impl(0x1234_0041), 0); // Multi-byte, not pure ASCII
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(schar_get_ascii_impl(0x8000_0000), 0); // >= 0x80 is not ASCII
            assert_eq!(schar_get_ascii_impl(0xFF00_0000), 0); // 0xFF is not ASCII
            assert_eq!(schar_get_ascii_impl(0x4100_0001), 0); // Multi-byte, not pure ASCII
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)] // b'A' as i8 is safe for ASCII
    fn test_ffi_schar_get_ascii() {
        #[cfg(target_endian = "little")]
        {
            assert_eq!(rs_schar_get_ascii(0x0000_0041), b'A' as i8);
            assert_eq!(rs_schar_get_ascii(0x0000_0080), 0);
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(rs_schar_get_ascii(0x4100_0000), b'A' as i8);
            assert_eq!(rs_schar_get_ascii(0x8000_0000), 0);
        }
    }

    #[test]
    fn test_schar_from_char_ascii() {
        // ASCII characters should be stored in the first byte
        #[cfg(target_endian = "little")]
        {
            assert_eq!(schar_from_char_impl(b'A' as c_int), 0x0000_0041);
            assert_eq!(schar_from_char_impl(b'a' as c_int), 0x0000_0061);
            assert_eq!(schar_from_char_impl(b' ' as c_int), 0x0000_0020);
            assert_eq!(schar_from_char_impl(0), 0x0000_0000);
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(schar_from_char_impl(b'A' as c_int), 0x4100_0000);
            assert_eq!(schar_from_char_impl(b'a' as c_int), 0x6100_0000);
            assert_eq!(schar_from_char_impl(b' ' as c_int), 0x2000_0000);
            assert_eq!(schar_from_char_impl(0), 0x0000_0000);
        }
    }

    #[test]
    fn test_schar_from_char_multibyte() {
        // 2-byte UTF-8: U+00E9 (é) = 0xC3 0xA9
        #[cfg(target_endian = "little")]
        {
            assert_eq!(schar_from_char_impl(0x00E9), 0x0000_A9C3);
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(schar_from_char_impl(0x00E9), 0xC3A9_0000);
        }

        // 3-byte UTF-8: U+4E2D (中) = 0xE4 0xB8 0xAD
        #[cfg(target_endian = "little")]
        {
            assert_eq!(schar_from_char_impl(0x4E2D), 0x00AD_B8E4);
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(schar_from_char_impl(0x4E2D), 0xE4B8_AD00);
        }

        // 4-byte UTF-8: U+1F600 (😀) = 0xF0 0x9F 0x98 0x80
        #[cfg(target_endian = "little")]
        {
            assert_eq!(schar_from_char_impl(0x1_F600), 0x8098_9FF0);
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(schar_from_char_impl(0x1_F600), 0xF09F_9880);
        }
    }

    #[test]
    fn test_schar_from_char_replacement() {
        // Characters >= 0x200000 should be replaced with U+FFFD
        // U+FFFD = 0xEF 0xBF 0xBD
        #[cfg(target_endian = "little")]
        {
            let replacement = 0x00BD_BFEF_u32;
            assert_eq!(schar_from_char_impl(0x20_0000), replacement);
            assert_eq!(schar_from_char_impl(0x7FFF_FFFF), replacement);
        }
        #[cfg(target_endian = "big")]
        {
            let replacement = 0xEFBF_BD00_u32;
            assert_eq!(schar_from_char_impl(0x20_0000), replacement);
            assert_eq!(schar_from_char_impl(0x7FFF_FFFF), replacement);
        }
    }

    #[test]
    fn test_ffi_schar_from_char() {
        // Just verify the FFI wrapper works
        #[cfg(target_endian = "little")]
        {
            assert_eq!(rs_schar_from_char(b'A' as c_int), 0x0000_0041);
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(rs_schar_from_char(b'A' as c_int), 0x4100_0000);
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_schar_roundtrip() {
        // Verify that schar_from_char and schar_get_ascii are consistent for ASCII
        for c in 0..0x80 {
            let sc = schar_from_char_impl(c);
            assert_eq!(schar_get_ascii_impl(sc), c as i8);
        }
    }

    #[test]
    fn test_rgb_basic() {
        // Test basic RGB packing: 0x00RRGGBB format
        assert_eq!(rs_rgb(0, 0, 0), 0x00_00_00); // Black
        assert_eq!(rs_rgb(255, 255, 255), 0xFF_FF_FF); // White
        assert_eq!(rs_rgb(255, 0, 0), 0xFF_00_00); // Red
        assert_eq!(rs_rgb(0, 255, 0), 0x00_FF_00); // Green
        assert_eq!(rs_rgb(0, 0, 255), 0x00_00_FF); // Blue
    }

    #[test]
    fn test_rgb_components() {
        // Test that components are placed in correct positions
        // Red in bits 16-23
        assert_eq!(rs_rgb(0x12, 0, 0), 0x12_00_00);
        // Green in bits 8-15
        assert_eq!(rs_rgb(0, 0x34, 0), 0x00_34_00);
        // Blue in bits 0-7
        assert_eq!(rs_rgb(0, 0, 0x56), 0x00_00_56);
        // All components combined
        assert_eq!(rs_rgb(0x12, 0x34, 0x56), 0x12_34_56);
    }

    #[test]
    fn test_rgb_masking() {
        // Test that only the lower 8 bits of each component are used
        // Values >= 256 should have upper bits masked off
        assert_eq!(rs_rgb(0x1FF, 0, 0), 0xFF_00_00); // 0x1FF & 0xFF = 0xFF
        assert_eq!(rs_rgb(0, 0x1FF, 0), 0x00_FF_00);
        assert_eq!(rs_rgb(0, 0, 0x1FF), 0x00_00_FF);
        assert_eq!(rs_rgb(0x100, 0x100, 0x100), 0x00_00_00); // 0x100 & 0xFF = 0
    }

    #[test]
    fn test_rgb_negative() {
        // Test with negative values (shouldn't happen normally, but test the masking)
        // -1 as signed int has all bits set, & 0xFF = 0xFF
        assert_eq!(rs_rgb(-1, 0, 0), 0xFF_00_00);
        assert_eq!(rs_rgb(0, -1, 0), 0x00_FF_00);
        assert_eq!(rs_rgb(0, 0, -1), 0x00_00_FF);
    }
}
