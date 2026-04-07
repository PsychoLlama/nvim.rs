//! Grid utilities for Neovim
//!
//! This crate provides Rust implementations of grid/screen character functions
//! from `src/nvim/grid.c`.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::similar_names)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::branches_sharing_code)]
#![allow(clippy::if_not_else)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::range_plus_one)]

pub mod cell;
pub mod drawing;
pub mod helpers;
pub mod lifecycle;

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
static GLYPH_CACHE: LazyLock<Mutex<GlyphCache>> = LazyLock::new(|| Mutex::new(GlyphCache::new()));

// FFI declarations for C callback functions
extern "C" {
    /// Rust implementation of decor_check_invalid_glyphs (in decoration crate).
    fn rs_decor_check_invalid_glyphs();

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

#[no_mangle]
pub extern "C" fn schar_high(sc: ScharT) -> bool {
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

#[no_mangle]
pub extern "C" fn schar_get_ascii(sc: ScharT) -> i8 {
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

#[no_mangle]
pub extern "C" fn schar_from_char(c: c_int) -> ScharT {
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

#[no_mangle]
pub extern "C" fn schar_from_str(str_ptr: *const c_char) -> ScharT {
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

#[no_mangle]
pub unsafe extern "C" fn schar_from_buf(buf: *const c_char, len: usize) -> ScharT {
    rs_schar_from_buf(buf, len)
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
#[export_name = "schar_cache_clear_if_full"]
pub extern "C" fn rs_schar_cache_clear_if_full() -> bool {
    schar_cache_clear_if_full_impl()
}

/// Clear the glyph cache completely.
fn schar_cache_clear_impl() {
    // Call C callbacks before clearing
    // SAFETY: these C functions have no safety requirements
    unsafe {
        rs_decor_check_invalid_glyphs();
    }

    // Clear the cache
    {
        let mut cache = GLYPH_CACHE.lock().unwrap();
        cache.clear();
    }

    // Regenerate char options (must not fail)
    // SAFETY: this C function has no safety requirements
    let result = unsafe { nvim_check_chars_options() };
    assert!(
        result == 0,
        "check_chars_options() failed after cache clear"
    );
}

/// FFI wrapper for `schar_cache_clear`.
///
/// Clear the glyph cache completely.
#[export_name = "schar_cache_clear"]
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
    // Set final NUL: cap at MAX_SCHAR_SIZE - 1 to avoid OOB write
    let nul_pos = len.min(MAX_SCHAR_SIZE - 1);
    buf[nul_pos] = 0;
    nul_pos
}

#[no_mangle]
pub unsafe extern "C" fn schar_get(buf_out: *mut c_char, sc: ScharT) -> usize {
    rs_schar_get(buf_out, sc)
}

/// FFI wrapper for `schar_get_adv`.
///
/// Convert an `schar_T` to UTF-8 bytes, advancing the buffer pointer.
/// Does NOT set final NUL.
///
/// # Safety
/// - `buf_out` must point to a valid `*mut c_char` pointer
/// - The pointed-to buffer must be valid for writing at least `MAX_SCHAR_SIZE` bytes
#[export_name = "schar_get_adv"]
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
#[export_name = "schar_len"]
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
#[export_name = "schar_cells"]
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

#[no_mangle]
pub extern "C" fn schar_get_first_codepoint(sc: ScharT) -> c_int {
    schar_get_first_codepoint_impl(sc)
}

// =============================================================================
// Phase 30: Grid Line Content Functions
// =============================================================================

/// Type alias for screen attribute (matches C's `sattr_T` which is `int32_t`).
type SattrT = i32;

/// Type alias for column number (matches C's `colnr_T` which is `int32_t`).
type ColnrT = i32;

/// Type alias for handle (matches C's `handle_T` which is `int`).
type HandleT = c_int;

// =============================================================================
// Grid line state: Rust-owned statics (Phase 2)
// These replace the C static variables in grid.c that were accessed via
// nvim_get/set_grid_line_* C accessor functions.
// =============================================================================

/// Current grid being rendered
static mut GRID_LINE_GRID: *mut std::ffi::c_void = std::ptr::null_mut();
/// Current row being rendered (-1 = none)
static mut GRID_LINE_ROW: c_int = -1;
/// Column offset of the current grid view
static mut GRID_LINE_COLOFF: c_int = 0;
/// Maximum column for the current line
static mut GRID_LINE_MAXCOL: c_int = 0;
/// First dirty column (initialized to linebuf_size, set to i32::MAX equivalent)
static mut GRID_LINE_FIRST: c_int = i32::MAX;
/// Last dirty column
static mut GRID_LINE_LAST: c_int = 0;
/// Column to clear to
static mut GRID_LINE_CLEAR_TO: c_int = 0;
/// Background attribute
static mut GRID_LINE_BG_ATTR: c_int = 0;
/// Clear attribute
static mut GRID_LINE_CLEAR_ATTR: c_int = 0;
/// Line flags (SLF_*)
static mut GRID_LINE_FLAGS: c_int = 0;

// Inline accessors replacing nvim_get/set_grid_line_* C functions
#[inline]
unsafe fn nvim_get_grid_line_grid() -> *mut std::ffi::c_void {
    GRID_LINE_GRID
}
#[inline]
unsafe fn nvim_set_grid_line_grid(grid: *mut std::ffi::c_void) {
    GRID_LINE_GRID = grid;
}
#[inline]
unsafe fn nvim_get_grid_line_row() -> c_int {
    GRID_LINE_ROW
}
#[inline]
unsafe fn nvim_set_grid_line_row(row: c_int) {
    GRID_LINE_ROW = row;
}
#[inline]
unsafe fn nvim_get_grid_line_coloff() -> c_int {
    GRID_LINE_COLOFF
}
#[inline]
unsafe fn nvim_set_grid_line_coloff(coloff: c_int) {
    GRID_LINE_COLOFF = coloff;
}
#[inline]
unsafe fn nvim_get_grid_line_maxcol() -> c_int {
    GRID_LINE_MAXCOL
}
#[inline]
unsafe fn nvim_set_grid_line_maxcol(maxcol: c_int) {
    GRID_LINE_MAXCOL = maxcol;
}
#[inline]
unsafe fn nvim_get_grid_line_first() -> c_int {
    GRID_LINE_FIRST
}
#[inline]
unsafe fn nvim_set_grid_line_first(first: c_int) {
    GRID_LINE_FIRST = first;
}
#[inline]
unsafe fn nvim_get_grid_line_last() -> c_int {
    GRID_LINE_LAST
}
#[inline]
unsafe fn nvim_set_grid_line_last(last: c_int) {
    GRID_LINE_LAST = last;
}
#[inline]
unsafe fn nvim_get_grid_line_clear_to() -> c_int {
    GRID_LINE_CLEAR_TO
}
#[inline]
unsafe fn nvim_set_grid_line_clear_to(clear_to: c_int) {
    GRID_LINE_CLEAR_TO = clear_to;
}
#[inline]
unsafe fn nvim_get_grid_line_bg_attr() -> c_int {
    GRID_LINE_BG_ATTR
}
#[inline]
unsafe fn nvim_set_grid_line_bg_attr(bg_attr: c_int) {
    GRID_LINE_BG_ATTR = bg_attr;
}
#[inline]
unsafe fn nvim_get_grid_line_clear_attr() -> c_int {
    GRID_LINE_CLEAR_ATTR
}
#[inline]
unsafe fn nvim_set_grid_line_clear_attr(clear_attr: c_int) {
    GRID_LINE_CLEAR_ATTR = clear_attr;
}
#[inline]
unsafe fn nvim_get_grid_line_flags() -> c_int {
    GRID_LINE_FLAGS
}
#[inline]
unsafe fn nvim_set_grid_line_flags(flags: c_int) {
    GRID_LINE_FLAGS = flags;
}

// C accessor functions for line buffer arrays
extern "C" {
    // Line buffer accessors
    fn nvim_get_linebuf_char() -> *mut ScharT;
    fn nvim_get_linebuf_attr() -> *mut SattrT;
    fn nvim_get_linebuf_vcol() -> *mut ColnrT;
    fn nvim_get_linebuf_size() -> usize;

    // UI function
    fn nvim_ui_grid_cursor_goto(grid_handle: HandleT, row: c_int, col: c_int);

    // ScreenGrid field accessor (we need the handle field)
    fn nvim_screengrid_get_handle(grid: *mut std::ffi::c_void) -> HandleT;
    fn nvim_screengrid_get_rows(grid: *mut std::ffi::c_void) -> c_int;

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
    static mut exmode_active: bool;
    fn nvim_get_p_arshape() -> c_int;
    fn nvim_get_p_tbidi() -> c_int;

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

    // hl_combine_attr from highlight module (Rust-exported)
    fn hl_combine_attr(char_attr: c_int, prim_attr: c_int) -> c_int;
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

#[no_mangle]
pub unsafe extern "C" fn grid_line_put_schar(col: c_int, schar: ScharT, attr: c_int) {
    rs_grid_line_put_schar(col, schar, attr);
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

#[no_mangle]
pub unsafe extern "C" fn grid_line_fill(
    start_col: c_int,
    end_col: c_int,
    sc: ScharT,
    attr: c_int,
) -> c_int {
    rs_grid_line_fill(start_col, end_col, sc, attr)
}

/// Put a string of text at a column position.
///
/// Handles multibyte characters, double-width characters, and truncation.
/// Returns the number of cells used.
///
/// # Safety
/// Must be called after `grid_line_start()` and before `grid_line_flush()`.
/// `text` must be a valid pointer to UTF-8 bytes.
/// If `textlen >= 0`, at most `textlen` bytes are read.
/// If `textlen < 0`, the text must be NUL-terminated.
#[export_name = "grid_line_puts"]
pub unsafe extern "C" fn rs_grid_line_puts(
    mut col: c_int,
    text: *const c_char,
    textlen: c_int,
    attr: c_int,
) -> c_int {
    if text.is_null() {
        return 0;
    }

    let grid = nvim_get_grid_line_grid();
    debug_assert!(!grid.is_null());

    let start_col = col;
    let max_col = nvim_get_grid_line_maxcol();
    let linebuf_char = nvim_get_linebuf_char();
    let linebuf_attr = nvim_get_linebuf_attr();
    let linebuf_vcol = nvim_get_linebuf_vcol();

    let text_u8 = text.cast::<u8>();
    let mut ptr_offset: isize = 0;

    // Continue while: in bounds, within length limit (if set), and not at NUL
    while col < max_col
        && (textlen < 0 || (ptr_offset as c_int) < textlen)
        && *text_u8.offset(ptr_offset) != 0
    {
        // Get the multibyte character length
        let mbyte_blen = if textlen >= 0 {
            let maxlen = textlen as isize - ptr_offset;
            let len = nvim_mbyte::utfc_ptr2len_len(
                std::slice::from_raw_parts(text_u8.offset(ptr_offset), maxlen as usize),
                maxlen as usize,
            );
            if len > maxlen as usize {
                1
            } else {
                len
            }
        } else {
            // NUL-terminated string
            let remaining = std::slice::from_raw_parts(
                text_u8.offset(ptr_offset),
                MAX_SCHAR_SIZE, // sufficient for one character
            );
            nvim_mbyte::utfc_ptr2len(remaining)
        };

        // Get schar and first codepoint
        let ptr_slice = std::slice::from_raw_parts(text_u8.offset(ptr_offset), mbyte_blen);
        let (schar, mbyte_cells) = utfc_ptrlen2schar_impl(ptr_slice);

        // Handle invalid or too-wide characters
        let (schar, mbyte_cells) = if mbyte_cells > 2 || schar == 0 {
            (schar_from_char_impl(REPLACEMENT_CHAR), 1)
        } else {
            (schar, mbyte_cells)
        };

        // Handle truncation at edge
        let (schar, mbyte_cells) = if col + mbyte_cells > max_col {
            // Only 1 cell left, but character requires 2: use '>'
            (schar_from_char_impl(b'>' as c_int), 1)
        } else {
            (schar, mbyte_cells)
        };

        // Handle overwriting right half of double-width character
        if ptr_offset == 0 {
            let first = nvim_get_grid_line_first();
            let last = nvim_get_grid_line_last();
            if col > first && col < last && *linebuf_char.offset(col as isize) == 0 {
                *linebuf_char.offset((col - 1) as isize) = schar_from_char_impl(b'>' as c_int);
            }
        }

        // Write to line buffer
        *linebuf_char.offset(col as isize) = schar;
        #[allow(clippy::cast_possible_truncation)]
        {
            *linebuf_attr.offset(col as isize) = attr as SattrT;
        }
        *linebuf_vcol.offset(col as isize) = -1;

        // Handle double-width character
        if mbyte_cells == 2 {
            *linebuf_char.offset((col + 1) as isize) = 0;
            #[allow(clippy::cast_possible_truncation)]
            {
                *linebuf_attr.offset((col + 1) as isize) = attr as SattrT;
            }
            *linebuf_vcol.offset((col + 1) as isize) = -1;
        }

        col += mbyte_cells;
        ptr_offset += mbyte_blen as isize;
    }

    // Update dirty region
    if col > start_col {
        let first = nvim_get_grid_line_first();
        let last = nvim_get_grid_line_last();
        nvim_set_grid_line_first(first.min(start_col));
        nvim_set_grid_line_last(last.max(col));
    }

    col - start_col
}

/// Convert a UTF-8 byte sequence to an schar_T and get its display width.
///
/// This implements the logic of C's `utfc_ptrlen2schar`, handling:
/// - Invalid sequences (returns 0 schar)
/// - Composing characters as first char (prepends space)
/// - Display width calculation
fn utfc_ptrlen2schar_impl(bytes: &[u8]) -> (ScharT, c_int) {
    if bytes.is_empty() {
        return (0, 1);
    }

    let len = bytes.len();

    // Invalid or truncated sequence
    if (len == 1 && bytes[0] >= 0x80) || len == 0 {
        return (0, 1);
    }

    // Get first codepoint
    let c = nvim_mbyte::utf_ptr2char(bytes);

    // Check if first character is a composing character
    let first_compose = nvim_mbyte::utf_iscomposing_first(c);

    // Limit length for schar storage
    let maxlen = MAX_SCHAR_SIZE - 1 - if first_compose { 1 } else { 0 };
    let actual_len = if len > maxlen {
        nvim_mbyte::utfc_ptr2len_len(bytes, maxlen)
    } else {
        len
    };

    // Create schar, prepending space if first char is a composing character
    let schar = if first_compose {
        let mut buf = [0u8; MAX_SCHAR_SIZE];
        buf[0] = b' ';
        buf[1..1 + actual_len].copy_from_slice(&bytes[..actual_len]);
        schar_from_buf_impl(&buf[..actual_len + 1])
    } else {
        schar_from_buf_impl(&bytes[..actual_len])
    };

    // Get display width
    let cells = nvim_mbyte::utf_ptr2cells_len(bytes, actual_len);

    (schar, cells)
}

/// Convert a UTF-8 string to an schar_T.
///
/// This implements the C `utfc_ptr2schar` function. Returns the schar_T value
/// and also sets `*firstc` to the first codepoint of the string.
///
/// # Safety
/// - `p` must be a valid pointer to a NUL-terminated UTF-8 string
/// - `firstc` must be a valid pointer to a c_int
#[no_mangle]
pub unsafe extern "C" fn rs_utfc_ptr2schar(p: *const c_char, firstc: *mut c_int) -> ScharT {
    if p.is_null() || firstc.is_null() {
        if !firstc.is_null() {
            *firstc = 0;
        }
        return 0;
    }

    // Find string length (up to reasonable limit)
    let mut len = 0usize;
    while len < MAX_SCHAR_SIZE + 10 && *p.add(len) != 0 {
        len += 1;
    }

    if len == 0 {
        *firstc = 0;
        return 0;
    }

    let bytes = std::slice::from_raw_parts(p as *const u8, len);

    // Get first codepoint
    let c = nvim_mbyte::utf_ptr2char(bytes);
    *firstc = c;

    // Invalid sequence
    if len == 1 && bytes[0] >= 0x80 {
        return 0;
    }

    let first_compose = nvim_mbyte::utf_iscomposing_first(c);
    let maxlen = MAX_SCHAR_SIZE - 1 - if first_compose { 1 } else { 0 };
    let actual_len = nvim_mbyte::utfc_ptr2len_len(bytes, maxlen);

    // Invalid sequence (length 1 but not ASCII)
    if actual_len == 1 && bytes[0] >= 0x80 {
        return 0;
    }

    // Create schar, prepending space if first char is a composing character
    if first_compose {
        let mut buf = [0u8; MAX_SCHAR_SIZE];
        buf[0] = b' ';
        buf[1..1 + actual_len].copy_from_slice(&bytes[..actual_len]);
        schar_from_buf_impl(&buf[..actual_len + 1])
    } else {
        schar_from_buf_impl(&bytes[..actual_len])
    }
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

#[no_mangle]
pub unsafe extern "C" fn grid_line_clear_end(
    start_col: c_int,
    end_col: c_int,
    bg_attr: c_int,
    clear_attr: c_int,
) {
    rs_grid_line_clear_end(start_col, end_col, bg_attr, clear_attr);
}

/// Move the cursor to a position in the currently rendered line.
///
/// # Safety
/// Must be called after `grid_line_start()` and before `grid_line_flush()`.
#[export_name = "grid_line_cursor_goto"]
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

#[no_mangle]
pub unsafe extern "C" fn grid_line_flush() {
    rs_grid_line_flush();
}

/// Flush grid line but only if on a valid row.
///
/// This is a stopgap until message.c has been refactored to behave.
/// If the row is invalid and kOptRdbFlagInvalid is set, aborts.
///
/// # Safety
/// Must be called after `grid_line_start()`.
#[export_name = "grid_line_flush_if_valid_row"]
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
    if exmode_active || (nvim_get_rdb_flags() & K_OPT_RDB_FLAG_NODELTA) != 0 {
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
    let invalid_row = grid != default_grid && grid_invalid_row(grid_attrs, line_off) && col == 0;

    let off_to = line_off + coloff as usize;
    let max_off_to = line_off + grid_cols as usize;

    // Handle overwriting right half of double-width character
    if col > 0 && *grid_chars.add(off_to + col as usize) == 0 {
        *linebuf_char.offset((col - 1) as isize) = schar_from_char_impl(b'>' as c_int);
        *linebuf_attr.offset((col - 1) as isize) = *grid_attrs.add(off_to + (col - 1) as usize);
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
        rs_line_do_arabic_shape(linebuf_char.offset(col as isize), endcol - col);
    }

    // Combine background attribute with line attributes
    if bg_attr != 0 {
        for c in col..endcol {
            let attr = *linebuf_attr.offset(c as isize) as c_int;
            #[allow(clippy::cast_possible_truncation)]
            {
                *linebuf_attr.offset(c as isize) = hl_combine_attr(bg_attr, attr) as SattrT;
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
    clear_attr = hl_combine_attr(bg_attr, clear_attr);

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

#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn grid_put_linebuf(
    grid: *mut std::ffi::c_void,
    row: c_int,
    coloff: c_int,
    col: c_int,
    endcol: c_int,
    clear_width: c_int,
    bg_attr: c_int,
    clear_attr: c_int,
    last_vcol: ColnrT,
    flags: c_int,
) {
    rs_grid_put_linebuf(
        grid,
        row,
        coloff,
        col,
        endcol,
        clear_width,
        bg_attr,
        clear_attr,
        last_vcol,
        flags,
    );
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
        cache
            .get(idx)
            .map_or(0, |bytes| bytes.first().copied().unwrap_or(0))
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
#[export_name = "line_do_arabic_shape"]
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
        let c0new = nvim_arabic::rs_arabic_shape(c0, &mut c1new, c0prev, 0, c0next);

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
        let c1_len = if c1 != 0 {
            nvim_mbyte::utf_char2len(c1)
        } else {
            0
        };
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
            scbuf_new[len..len + rest_len]
                .copy_from_slice(&scbuf[rest_start..rest_start + rest_len]);
        }

        // Create new schar from buffer
        *buf.add(i as usize) = schar_from_buf_impl(&scbuf_new[..len + rest_len]);

        c0prev = c0;
        c0 = c0next;
        c1 = c1next;
        i += 1;
    }
}

// =============================================================================
// Phase 34: Grid Operations
// =============================================================================

/// Opaque pointer type for GridView
type GridViewPtr = *mut std::ffi::c_void;

// Additional C accessor functions for Phase 34
extern "C" {
    // GridView field accessors
    fn nvim_gridview_get_target(view: GridViewPtr) -> *mut std::ffi::c_void;
    fn nvim_gridview_get_row_offset(view: GridViewPtr) -> c_int;
    fn nvim_gridview_get_col_offset(view: GridViewPtr) -> c_int;
}

/// Adjust grid viewport coordinates and return target grid.
///
/// This is the Rust equivalent of C's `grid_adjust()`.
/// Adds the viewport offsets to row/col and returns the target ScreenGrid.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_adjust(
    view: GridViewPtr,
    row_off: *mut c_int,
    col_off: *mut c_int,
) -> *mut std::ffi::c_void {
    *row_off += nvim_gridview_get_row_offset(view);
    *col_off += nvim_gridview_get_col_offset(view);
    nvim_gridview_get_target(view)
}

#[no_mangle]
pub unsafe extern "C" fn grid_adjust(
    view: GridViewPtr,
    row_off: *mut c_int,
    col_off: *mut c_int,
) -> *mut std::ffi::c_void {
    rs_grid_adjust(view, row_off, col_off)
}

/// Clear a line in the grid starting at "off" until "width" characters are cleared.
///
/// This is the Rust equivalent of C's `grid_clear_line()`.
#[export_name = "grid_clear_line"]
pub unsafe extern "C" fn rs_grid_clear_line(
    grid: *mut std::ffi::c_void,
    off: usize,
    width: c_int,
    valid: bool,
) {
    let chars = nvim_screengrid_get_chars(grid);
    let attrs = nvim_screengrid_get_attrs(grid);
    let vcols = nvim_screengrid_get_vcols(grid);

    // Fill chars with space
    let space = schar_from_char_impl(b' ' as c_int);
    for col in 0..width as usize {
        *chars.add(off + col) = space;
    }

    // Fill attrs with 0 (valid) or -1 (invalid)
    let fill: SattrT = if valid { 0 } else { -1 };
    for col in 0..width as usize {
        *attrs.add(off + col) = fill;
    }

    // Fill vcols with -1
    for col in 0..width as usize {
        *vcols.add(off + col) = -1;
    }
}

/// Invalidate all rows in a grid by setting all attrs to -1.
///
/// This is the Rust equivalent of C's `grid_invalidate()`.
#[export_name = "grid_invalidate"]
pub unsafe extern "C" fn rs_grid_invalidate(grid: *mut std::ffi::c_void) {
    let attrs = nvim_screengrid_get_attrs(grid);
    let rows = nvim_screengrid_get_rows(grid);
    let cols = nvim_screengrid_get_cols(grid);
    let total = (rows as usize) * (cols as usize);

    // Set all attrs to -1
    for i in 0..total {
        *attrs.add(i) = -1;
    }
}

/// Get a single character directly from grid.chars.
///
/// This is the Rust equivalent of C's `grid_getchar()`.
/// Returns NUL if out of bounds or grid.chars is NULL.
///
/// @param grid  The ScreenGrid to read from
/// @param row   Row index
/// @param col   Column index
/// @param attrp Optional pointer to receive the character's attribute
#[export_name = "grid_getchar"]
pub unsafe extern "C" fn rs_grid_getchar(
    grid: *mut std::ffi::c_void,
    row: c_int,
    col: c_int,
    attrp: *mut c_int,
) -> ScharT {
    let chars = nvim_screengrid_get_chars(grid);

    // Safety check
    if chars.is_null() {
        return 0; // NUL
    }

    let rows = nvim_screengrid_get_rows(grid);
    let cols = nvim_screengrid_get_cols(grid);

    if row >= rows || col >= cols {
        return 0; // NUL
    }

    let line_offset = nvim_screengrid_get_line_offset(grid);
    let off = *line_offset.add(row as usize) + col as usize;

    if !attrp.is_null() {
        let attrs = nvim_screengrid_get_attrs(grid);
        *attrp = (*attrs.add(off)) as c_int;
    }

    *chars.add(off)
}

/// Clear a rectangular region of a grid.
///
/// This is the Rust equivalent of C's `grid_clear()`.
/// Clears from start_row to end_row, start_col to end_col with given attribute.
#[export_name = "grid_clear"]
pub unsafe extern "C" fn rs_grid_clear(
    view: GridViewPtr,
    start_row: c_int,
    end_row: c_int,
    start_col: c_int,
    mut end_col: c_int,
    attr: c_int,
) {
    for row in start_row..end_row {
        // Call grid_line_start to properly initialize all grid line state
        rs_grid_line_start(view, row);

        // Clamp end_col to grid_line_maxcol (set by grid_line_start)
        end_col = std::cmp::min(end_col, nvim_get_grid_line_maxcol());

        // Get the grid and row from the line state
        let grid = nvim_get_grid_line_grid();
        let grid_line_row = nvim_get_grid_line_row();
        let grid_rows = nvim_screengrid_get_rows(grid);

        if grid_line_row >= grid_rows || start_col >= end_col {
            // TODO(bfredl): make callers behave instead
            nvim_set_grid_line_grid(std::ptr::null_mut());
            return;
        }

        // grid_line_clear_end equivalent
        rs_grid_line_clear_end(start_col, end_col, attr, 0);

        // grid_line_flush
        rs_grid_line_flush();
    }
}

// =============================================================================
// Phase 36: Grid Scrolling
// =============================================================================

// Additional C accessors for Phase 37
extern "C" {
    fn nvim_get_full_screen() -> bool;
    fn nvim_get_linebuf_scratch() -> *mut c_char;
}

// C accessor for UI scroll call
extern "C" {
    fn nvim_ui_call_grid_scroll(
        handle: HandleT,
        top: c_int,
        bot: c_int,
        left: c_int,
        right: c_int,
        rows: c_int,
        cols: c_int,
    );
}

/// Copy a portion of one line to another within a grid.
///
/// This is the Rust equivalent of C's `linecopy()`.
#[inline]
unsafe fn linecopy_impl(
    grid: *mut std::ffi::c_void,
    to: c_int,
    from: c_int,
    col: c_int,
    width: c_int,
) {
    let chars = nvim_screengrid_get_chars(grid);
    let attrs = nvim_screengrid_get_attrs(grid);
    let vcols = nvim_screengrid_get_vcols(grid);
    let line_offset = nvim_screengrid_get_line_offset(grid);

    let off_to = *line_offset.add(to as usize) + col as usize;
    let off_from = *line_offset.add(from as usize) + col as usize;
    let width_usize = width as usize;

    // Use memmove since regions may overlap
    std::ptr::copy(chars.add(off_from), chars.add(off_to), width_usize);
    std::ptr::copy(attrs.add(off_from), attrs.add(off_to), width_usize);
    std::ptr::copy(vcols.add(off_from), vcols.add(off_to), width_usize);
}

/// Insert lines in a grid by scrolling down.
///
/// This is the Rust equivalent of C's `grid_ins_lines()`.
/// Shifts lines down and clears the inserted lines at the top.
#[export_name = "grid_ins_lines"]
pub unsafe extern "C" fn rs_grid_ins_lines(
    grid: *mut std::ffi::c_void,
    row: c_int,
    line_count: c_int,
    end: c_int,
    col: c_int,
    width: c_int,
) {
    if line_count <= 0 {
        return;
    }

    let grid_cols = nvim_screengrid_get_cols(grid);
    let line_offset = nvim_screengrid_get_line_offset(grid);

    // Shift line_offset[] line_count down to reflect the inserted lines.
    // Clear the inserted lines.
    for i in 0..line_count {
        if width != grid_cols {
            // need to copy part of a line
            let mut j = end - 1 - i;
            // C does: while ((j -= line_count) >= row)
            // We need to decrement j BEFORE checking condition
            loop {
                j -= line_count;
                if j < row {
                    break;
                }
                linecopy_impl(grid, j + line_count, j, col, width);
            }
            j += line_count;
            let off = *line_offset.add(j as usize) + col as usize;
            rs_grid_clear_line(grid, off, width, false);
        } else {
            // whole width, moving the line pointers is faster
            let mut j = end - 1 - i;
            let temp = *line_offset.add(j as usize);
            // C does: while ((j -= line_count) >= row)
            // We need to decrement j BEFORE checking condition
            loop {
                j -= line_count;
                if j < row {
                    break;
                }
                *line_offset.add((j + line_count) as usize) = *line_offset.add(j as usize);
            }
            *line_offset.add((j + line_count) as usize) = temp;
            rs_grid_clear_line(grid, temp, grid_cols, false);
        }
    }

    if !nvim_screengrid_get_throttled(grid) {
        let handle = nvim_screengrid_get_handle(grid);
        nvim_ui_call_grid_scroll(handle, row, end, col, col + width, -line_count, 0);
    }
}

/// Delete lines in a grid by scrolling up.
///
/// This is the Rust equivalent of C's `grid_del_lines()`.
/// Shifts lines up and clears the deleted lines at the bottom.
#[export_name = "grid_del_lines"]
pub unsafe extern "C" fn rs_grid_del_lines(
    grid: *mut std::ffi::c_void,
    row: c_int,
    line_count: c_int,
    end: c_int,
    col: c_int,
    width: c_int,
) {
    if line_count <= 0 {
        return;
    }

    let grid_cols = nvim_screengrid_get_cols(grid);
    let line_offset = nvim_screengrid_get_line_offset(grid);

    // Now shift line_offset[] line_count up to reflect the deleted lines.
    // Clear the inserted lines.
    for i in 0..line_count {
        if width != grid_cols {
            // need to copy part of a line
            let mut j = row + i;
            // C does: while ((j += line_count) <= end - 1)
            // We need to increment j BEFORE checking condition
            loop {
                j += line_count;
                if j > end - 1 {
                    break;
                }
                linecopy_impl(grid, j - line_count, j, col, width);
            }
            j -= line_count;
            let off = *line_offset.add(j as usize) + col as usize;
            rs_grid_clear_line(grid, off, width, false);
        } else {
            // whole width, moving the line pointers is faster
            let mut j = row + i;
            let temp = *line_offset.add(j as usize);
            // C does: while ((j += line_count) <= end - 1)
            // We need to increment j BEFORE checking condition
            loop {
                j += line_count;
                if j > end - 1 {
                    break;
                }
                *line_offset.add((j - line_count) as usize) = *line_offset.add(j as usize);
            }
            *line_offset.add((j - line_count) as usize) = temp;
            rs_grid_clear_line(grid, temp, grid_cols, false);
        }
    }

    if !nvim_screengrid_get_throttled(grid) {
        let handle = nvim_screengrid_get_handle(grid);
        nvim_ui_call_grid_scroll(handle, row, end, col, col + width, line_count, 0);
    }
}

// =============================================================================
// Phase 37: Grid Line Start/Getchar/Mirror
// =============================================================================

/// kOptRdbFlagInvalid (0x04)
const K_OPT_RDB_FLAG_INVALID_P37: c_uint = 0x04;

/// Start rendering a grid line at the low level.
///
/// This is the Rust equivalent of C's `screengrid_line_start()`.
/// Sets up all grid line state variables for rendering.
///
/// # Safety
/// - `grid` must be a valid ScreenGrid pointer
#[export_name = "screengrid_line_start"]
pub unsafe extern "C" fn rs_screengrid_line_start(
    grid: *mut std::ffi::c_void,
    row: c_int,
    col: c_int,
) {
    let grid_cols = nvim_screengrid_get_cols(grid);
    let linebuf_sz = nvim_get_linebuf_size();

    // grid_line_maxcol = grid->cols
    // assert(grid_line_grid == NULL)
    debug_assert!(nvim_get_grid_line_grid().is_null());

    nvim_set_grid_line_row(row);
    nvim_set_grid_line_grid(grid);
    nvim_set_grid_line_coloff(col);

    // grid_line_first = (int)linebuf_size
    #[allow(clippy::cast_possible_truncation)]
    nvim_set_grid_line_first(linebuf_sz as c_int);

    // grid_line_maxcol = MIN(grid_line_maxcol, grid->cols - grid_line_coloff)
    let effective_maxcol = (grid_cols - col).min(grid_cols);
    nvim_set_grid_line_maxcol(effective_maxcol);

    nvim_set_grid_line_last(0);
    nvim_set_grid_line_clear_to(0);
    nvim_set_grid_line_bg_attr(0);
    nvim_set_grid_line_clear_attr(0);
    nvim_set_grid_line_flags(0);

    #[allow(clippy::cast_possible_truncation)]
    let maxcol = effective_maxcol as usize;
    debug_assert!(maxcol <= linebuf_sz);

    // In debug/invalid mode, fill linebuf with invalid values
    if nvim_get_full_screen() && (nvim_get_rdb_flags() & K_OPT_RDB_FLAG_INVALID_P37) != 0 {
        let linebuf_char = nvim_get_linebuf_char();
        let linebuf_attr = nvim_get_linebuf_attr();
        debug_assert!(!linebuf_char.is_null());

        // memset with 0xFF
        std::ptr::write_bytes(linebuf_char, 0xFF, linebuf_sz);
        std::ptr::write_bytes(linebuf_attr, 0xFF, linebuf_sz);
    }
}

/// Start a group of grid_line_puts calls that builds a single grid line.
///
/// This is the Rust equivalent of C's `grid_line_start()`.
/// Must be matched with a grid_line_flush call before moving to another line.
///
/// # Safety
/// - `view` must be a valid GridView pointer
#[no_mangle]
pub unsafe extern "C" fn rs_grid_line_start(view: *mut std::ffi::c_void, row: c_int) {
    let mut adjusted_row = row;
    let mut col: c_int = 0;
    let grid = rs_grid_adjust(view, &mut adjusted_row, &mut col);
    rs_screengrid_line_start(grid, adjusted_row, col);
}

#[no_mangle]
pub unsafe extern "C" fn grid_line_start(view: *mut std::ffi::c_void, row: c_int) {
    rs_grid_line_start(view, row);
}

/// Get present char from current rendered screen line.
///
/// This is the Rust equivalent of C's `grid_line_getchar()`.
/// This indicates what already is on screen, not the pending render buffer.
///
/// # Safety
/// Must be called after `grid_line_start()`.
///
/// @return char or space if out of bounds
#[export_name = "grid_line_getchar"]
pub unsafe extern "C" fn rs_grid_line_getchar(col: c_int, attr: *mut c_int) -> ScharT {
    let maxcol = nvim_get_grid_line_maxcol();

    if col < maxcol {
        let grid = nvim_get_grid_line_grid();
        let coloff = nvim_get_grid_line_coloff();
        let row = nvim_get_grid_line_row();
        let line_offset = nvim_screengrid_get_line_offset(grid);
        let adjusted_col = col + coloff;
        let off = *line_offset.add(row as usize) + adjusted_col as usize;

        if !attr.is_null() {
            let attrs = nvim_screengrid_get_attrs(grid);
            *attr = (*attrs.add(off)) as c_int;
        }

        let chars = nvim_screengrid_get_chars(grid);
        *chars.add(off)
    } else {
        // NUL is a very special value (right-half of double width), space is True Neutral™
        schar_from_char_impl(b' ' as c_int)
    }
}

/// Mirror the line buffer for right-to-left text.
///
/// This is the Rust equivalent of C's `linebuf_mirror()`.
/// Reverses the order of characters, attributes, and vcols in the buffer.
///
/// # Safety
/// - `firstp`, `lastp`, `clearp` must be valid pointers
#[no_mangle]
pub unsafe extern "C" fn rs_linebuf_mirror(
    firstp: *mut c_int,
    lastp: *mut c_int,
    clearp: *mut c_int,
    width: c_int,
) {
    let first = *firstp;
    let last = *lastp;

    let n = (last - first) as usize;
    let mirror = width - 1; // Mirrors are more fun than television.

    let linebuf_char = nvim_get_linebuf_char();
    let linebuf_attr = nvim_get_linebuf_attr();
    let linebuf_vcol = nvim_get_linebuf_vcol();
    let scratch = nvim_get_linebuf_scratch();

    // Mirror characters
    {
        let scratch_char = scratch.cast::<ScharT>();
        // Copy linebuf_char[first..last] to scratch_char[first..last]
        std::ptr::copy_nonoverlapping(
            linebuf_char.offset(first as isize),
            scratch_char.offset(first as isize),
            n,
        );

        let mut col = first;
        while col < last {
            let rev = mirror - col;
            // Check for double-width character (second cell is 0)
            if col + 1 < last && *scratch_char.offset((col + 1) as isize) == 0 {
                *linebuf_char.offset((rev - 1) as isize) = *scratch_char.offset(col as isize);
                *linebuf_char.offset(rev as isize) = 0;
                col += 1;
            } else {
                *linebuf_char.offset(rev as isize) = *scratch_char.offset(col as isize);
            }
            col += 1;
        }
    }

    // Mirror attributes (assumes doublewidth chars are self-consistent)
    {
        let scratch_attr = scratch.cast::<SattrT>();
        std::ptr::copy_nonoverlapping(
            linebuf_attr.offset(first as isize),
            scratch_attr.offset(first as isize),
            n,
        );

        for col in first..last {
            *linebuf_attr.offset((mirror - col) as isize) = *scratch_attr.offset(col as isize);
        }
    }

    // Mirror vcols
    {
        let scratch_vcol = scratch.cast::<ColnrT>();
        std::ptr::copy_nonoverlapping(
            linebuf_vcol.offset(first as isize),
            scratch_vcol.offset(first as isize),
            n,
        );

        for col in first..last {
            *linebuf_vcol.offset((mirror - col) as isize) = *scratch_vcol.offset(col as isize);
        }
    }

    // Update the pointers
    *firstp = width - *clearp;
    *clearp = width - first;
    *lastp = width - last;
}

#[no_mangle]
pub unsafe extern "C" fn linebuf_mirror(
    firstp: *mut c_int,
    lastp: *mut c_int,
    clearp: *mut c_int,
    width: c_int,
) {
    rs_linebuf_mirror(firstp, lastp, clearp, width);
}

/// Mirror the current grid line for right-to-left text display.
///
/// This is the Rust equivalent of C's `grid_line_mirror()`.
/// Updates the grid line state and sets the SLF_RIGHTLEFT flag.
///
/// # Safety
/// Must be called after `grid_line_start()`.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_line_mirror(width: c_int) {
    let last = nvim_get_grid_line_last();
    let clear_to = nvim_get_grid_line_clear_to();

    // grid_line_clear_to = MAX(grid_line_last, grid_line_clear_to)
    let new_clear_to = last.max(clear_to);
    nvim_set_grid_line_clear_to(new_clear_to);

    let first = nvim_get_grid_line_first();
    if first >= new_clear_to {
        return;
    }

    // Call linebuf_mirror with pointers to update state
    let mut first_val = first;
    let mut last_val = last;
    let mut clear_val = new_clear_to;

    rs_linebuf_mirror(&mut first_val, &mut last_val, &mut clear_val, width);

    // Update the grid line state
    nvim_set_grid_line_first(first_val);
    nvim_set_grid_line_last(last_val);
    nvim_set_grid_line_clear_to(clear_val);

    // Set the RTL flag
    let flags = nvim_get_grid_line_flags();
    nvim_set_grid_line_flags(flags | SLF_RIGHTLEFT);
}

#[no_mangle]
pub unsafe extern "C" fn grid_line_mirror(width: c_int) {
    rs_grid_line_mirror(width);
}

// =============================================================================
// Phase 39: Grid Handle Assignment and Border Text Alignment
// =============================================================================

use std::sync::atomic::{AtomicI32, Ordering};

/// Default grid handle value (matches C's DEFAULT_GRID_HANDLE)
const DEFAULT_GRID_HANDLE: i32 = 1;

/// Static counter for grid handle assignment, kept in Rust.
/// Starts at DEFAULT_GRID_HANDLE (1).
static LAST_GRID_HANDLE: AtomicI32 = AtomicI32::new(DEFAULT_GRID_HANDLE);

/// Alignment positions for border text (matches C's AlignTextPos enum).
#[repr(i32)]
#[allow(dead_code)]
enum AlignTextPos {
    Left = 0,
    Center = 1,
    Right = 2,
}

// External C functions for ScreenGrid handle access
extern "C" {
    fn nvim_screengrid_get_handle_ptr(grid: *mut std::ffi::c_void) -> *mut HandleT;
}

/// Assign a unique handle to a grid.
///
/// If the grid already has a handle (non-zero), it is not modified.
/// Otherwise, a new unique handle is assigned using an atomic counter.
///
/// # Safety
/// `grid` must be a valid ScreenGrid pointer.
#[export_name = "grid_assign_handle"]
pub unsafe extern "C" fn rs_grid_assign_handle(grid: *mut std::ffi::c_void) {
    let handle_ptr = nvim_screengrid_get_handle_ptr(grid);
    if handle_ptr.is_null() {
        return;
    }

    // Only assign if not already assigned
    if *handle_ptr == 0 {
        // Atomically increment and get the new value
        *handle_ptr = LAST_GRID_HANDLE.fetch_add(1, Ordering::Relaxed) + 1;
    }
}

/// Calculate the starting column for border text based on alignment.
///
/// Returns 1-indexed column position for the text within the border.
///
/// # Arguments
/// * `total_col` - Total number of columns available
/// * `text_width` - Width of the text to place
/// * `align` - Alignment position (0=left, 1=center, 2=right)
#[no_mangle]
pub extern "C" fn rs_get_bordertext_col(
    total_col: c_int,
    text_width: c_int,
    align: c_int,
) -> c_int {
    match align {
        0 => 1, // kAlignLeft
        1 => {
            // kAlignCenter
            let col = (total_col - text_width) / 2 + 1;
            col.max(1)
        }
        2 => {
            // kAlignRight
            let col = total_col - text_width + 1;
            col.max(1)
        }
        _ => 1, // Fallback to left alignment
    }
}

// Message grid accessors
extern "C" {
    /// Check if default_grid.chars is non-NULL
    fn nvim_get_default_grid_has_chars() -> c_int;
    /// Check if any UI has kUIMessages extension
    fn ui_has(ext: c_int) -> bool;
}

/// UIExtension value for kUIMessages (ui_defs.h)
const K_UI_MESSAGES: c_int = 4;

/// Check if message grid should be used.
///
/// Returns true if default_grid.chars is allocated and no UI has kUIMessages extension.
///
/// # Safety
/// Calls C accessor functions for grid and UI state.
#[export_name = "msg_use_grid"]
pub unsafe extern "C" fn rs_msg_use_grid() -> bool {
    nvim_get_default_grid_has_chars() != 0 && !ui_has(K_UI_MESSAGES)
}

// Message scroll accessors
extern "C" {
    static mut msg_scrolled: c_int;
    static mut p_ch: i64;
}

const K_OPT_RDB_FLAG_NOTHROTTLE: c_uint = 0x02;

/// Calculate the message scroll size including horizontal separator.
///
/// Returns `msg_scrolled + p_ch + separator` where separator is 1 if
/// `p_ch > 0` or `msg_scrolled > 1`, otherwise 0.
///
/// # Safety
/// Calls C accessor functions for global state.
#[export_name = "msg_scrollsize"]
pub unsafe extern "C" fn rs_msg_scrollsize() -> c_int {
    let scrolled = msg_scrolled;
    let separator = c_int::from(p_ch > 0 || scrolled > 1);
    scrolled + p_ch as c_int + separator
}

/// Check if message throttling should be used.
///
/// Returns true if message grid is used and nothrottle debug flag is not set.
///
/// # Safety
/// Calls C accessor functions for grid and debug state.
#[export_name = "msg_do_throttle"]
pub unsafe extern "C" fn rs_msg_do_throttle() -> bool {
    let use_grid = rs_msg_use_grid();
    let rdb_flags = nvim_get_rdb_flags();
    use_grid && (rdb_flags & K_OPT_RDB_FLAG_NOTHROTTLE) == 0
}

// Message redirection state (accessed as extern statics)
#[allow(clashing_extern_declarations)]
extern "C" {
    static mut redir_fd: *mut std::ffi::c_void;
    static mut p_vfile: *mut std::ffi::c_char;
    static mut redir_reg: c_int;
    static mut redir_vname: bool;
    static mut capture_ga: *mut std::ffi::c_void;
}

/// Check if output is being redirected.
///
/// # Safety
/// Reads redirection state globals directly.
#[export_name = "redirecting"]
pub unsafe extern "C" fn rs_redirecting() -> c_int {
    c_int::from(
        !redir_fd.is_null()
            || (!p_vfile.is_null() && *p_vfile != 0)
            || redir_reg != 0
            || redir_vname
            || !capture_ga.is_null(),
    )
}

// Message printf mode accessors
extern "C" {
    static mut embedded_mode: bool;
    fn ui_active() -> usize;
}

/// Check if messages should be printed to stdout/stderr.
///
/// Returns true in "batch mode" (silent mode, -es/-Es/-l) when
/// no UI is active and not in embedded mode.
///
/// # Safety
/// Calls C accessor functions for embedded mode and UI state.
#[export_name = "msg_use_printf"]
pub unsafe extern "C" fn rs_msg_use_printf() -> c_int {
    c_int::from(!embedded_mode && ui_active() == 0)
}

// UI cursor position accessors
extern "C" {
    fn nvim_get_ui_cursor_row() -> c_int;
    fn nvim_get_ui_cursor_col() -> c_int;
}

/// Get the current UI cursor row.
///
/// # Safety
/// Calls C accessor function for cursor state.
#[export_name = "ui_current_row"]
pub unsafe extern "C" fn rs_ui_current_row() -> c_int {
    nvim_get_ui_cursor_row()
}

/// Get the current UI cursor column.
///
/// # Safety
/// Calls C accessor function for cursor state.
#[export_name = "ui_current_col"]
pub unsafe extern "C" fn rs_ui_current_col() -> c_int {
    nvim_get_ui_cursor_col()
}

// UI extension accessors
extern "C" {
    fn nvim_get_ui_ext(ext: c_int) -> c_int;
}

/// Check if a UI extension is enabled.
///
/// # Safety
/// Calls C accessor function for UI extension state.
#[export_name = "ui_has"]
pub unsafe extern "C" fn rs_ui_has(ext: c_int) -> c_int {
    nvim_get_ui_ext(ext)
}

// ============================================================================
// Phase 429: Additional Message Grid Functions
// ============================================================================

/// C `ScreenGrid` struct layout (verified with offsetof).
///
/// Offsets: handle=0, chars=8, attrs=16, vcols=24, line_offset=32,
/// dirty_col=40, rows=48, cols=52, valid=56, throttled=57, blending=58,
/// mouse_enabled=59, zindex=60, comp_row=64..comp_index=80, total=96 bytes.
#[repr(C)]
pub struct ScreenGrid {
    pub handle: c_int,
    _pad0: c_int,
    pub chars: *mut u32,
    pub attrs: *mut i32,
    pub vcols: *mut c_int,
    pub line_offset: *mut usize,
    pub dirty_col: *mut c_int,
    pub rows: c_int,
    pub cols: c_int,
    pub valid: bool,
    pub throttled: bool,
    pub blending: bool,
    pub mouse_enabled: bool,
    pub zindex: c_int,
    pub comp_row: c_int,
    pub comp_col: c_int,
    pub comp_width: c_int,
    pub comp_height: c_int,
    _pad1: c_int,
    pub comp_index: usize,
    pub comp_disabled: bool,
    pub pending_comp_index_update: bool,
    _pad2: [u8; 6],
}

extern "C" {
    /// Get msg_grid_pos (current message grid row position)
    static mut msg_grid_pos: c_int;
    /// Get msg_scrolled_at_flush
    static mut msg_scrolled_at_flush: c_int;
    /// The message grid
    static mut msg_grid: ScreenGrid;
}

/// Get the current message grid row position.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_grid_pos() -> c_int {
    msg_grid_pos
}

/// Set the message grid row position.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_grid_pos(pos: c_int) {
    msg_grid_pos = pos;
}

/// Get the msg_scrolled value at last flush.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_scrolled_at_flush() -> c_int {
    msg_scrolled_at_flush
}

/// Set the msg_scrolled value at flush.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_scrolled_at_flush(val: c_int) {
    msg_scrolled_at_flush = val;
}

/// Check if the message grid has chars allocated.
///
/// # Safety
/// Accesses msg_grid global.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_grid_allocated() -> c_int {
    c_int::from(!msg_grid.chars.is_null())
}

/// Check if the message grid is being throttled.
///
/// # Safety
/// Accesses msg_grid global.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_grid_throttled() -> c_int {
    c_int::from(msg_grid.throttled)
}

/// Set the message grid throttled state.
///
/// # Safety
/// Writes to msg_grid global.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_grid_throttled(val: c_int) {
    msg_grid.throttled = val != 0;
}

/// Get the number of rows in the message grid.
///
/// # Safety
/// Accesses msg_grid global.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_grid_rows() -> c_int {
    msg_grid.rows
}

/// Get the number of columns in the message grid.
///
/// # Safety
/// Accesses msg_grid global.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_grid_cols() -> c_int {
    msg_grid.cols
}

/// Check if message grid is ready for use.
///
/// Returns true if grid is allocated and has valid dimensions.
///
/// # Safety
/// Accesses msg_grid global.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_grid_ready() -> c_int {
    let has_chars = !msg_grid.chars.is_null();
    let rows = msg_grid.rows;
    let cols = msg_grid.cols;
    c_int::from(has_chars && rows > 0 && cols > 0)
}

/// Check if message grid should be flushed.
///
/// Returns true if throttled and grid has pending changes.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_grid_needs_flush() -> c_int {
    let throttled = msg_grid.throttled;
    let scrolled = msg_scrolled;
    let at_flush = msg_scrolled_at_flush;
    c_int::from(throttled && scrolled > at_flush)
}

/// Calculate pending scroll delta.
///
/// Returns how many lines need to be scrolled since last flush.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_grid_scroll_delta() -> c_int {
    let scrolled = msg_scrolled;
    let at_flush = msg_scrolled_at_flush;
    if scrolled > at_flush {
        scrolled - at_flush
    } else {
        0
    }
}

/// Reset message grid flush state.
///
/// Synchronizes flush state with current scrolled value.
///
/// # Safety
/// Calls C accessor/mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_grid_sync_flush() {
    let scrolled = msg_scrolled;
    msg_scrolled_at_flush = scrolled;
}

/// Enable message grid throttling.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_grid_start_throttle() {
    msg_grid.throttled = true;
}

/// Disable message grid throttling.
///
/// # Safety
/// Writes to msg_grid global.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_grid_stop_throttle() {
    msg_grid.throttled = false;
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

    #[test]
    fn test_get_bordertext_col_left() {
        // Left alignment always returns 1
        assert_eq!(rs_get_bordertext_col(10, 5, 0), 1);
        assert_eq!(rs_get_bordertext_col(100, 50, 0), 1);
        assert_eq!(rs_get_bordertext_col(5, 10, 0), 1); // text wider than space
    }

    #[test]
    fn test_get_bordertext_col_center() {
        // Center alignment: (total - width) / 2 + 1
        assert_eq!(rs_get_bordertext_col(10, 4, 1), 4); // (10-4)/2 + 1 = 4
        assert_eq!(rs_get_bordertext_col(20, 10, 1), 6); // (20-10)/2 + 1 = 6
        assert_eq!(rs_get_bordertext_col(10, 10, 1), 1); // (10-10)/2 + 1 = 1
                                                         // When text is wider, clamp to 1
        assert_eq!(rs_get_bordertext_col(5, 10, 1), 1); // (5-10)/2 + 1 = -1.5 -> 1
    }

    #[test]
    fn test_get_bordertext_col_right() {
        // Right alignment: total - width + 1
        assert_eq!(rs_get_bordertext_col(10, 4, 2), 7); // 10 - 4 + 1 = 7
        assert_eq!(rs_get_bordertext_col(20, 10, 2), 11); // 20 - 10 + 1 = 11
        assert_eq!(rs_get_bordertext_col(10, 10, 2), 1); // 10 - 10 + 1 = 1
                                                         // When text is wider, clamp to 1
        assert_eq!(rs_get_bordertext_col(5, 10, 2), 1); // 5 - 10 + 1 = -4 -> 1
    }

    #[test]
    fn test_get_bordertext_col_invalid() {
        // Invalid alignment values fallback to left (1)
        assert_eq!(rs_get_bordertext_col(10, 5, 3), 1);
        assert_eq!(rs_get_bordertext_col(10, 5, -1), 1);
        assert_eq!(rs_get_bordertext_col(10, 5, 100), 1);
    }
}
