//! Key mapping and abbreviation functions for Neovim
//!
//! This module provides Rust implementations of key mapping functions
//! from `src/nvim/mapping.c`. It uses the opaque handle pattern for
//! `mapblock_T*` with field access through C accessor functions.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(dead_code)] // Some FFI declarations are pre-declared for future use
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_truncation)] // Intentional for FFI
#![allow(clippy::cast_sign_loss)] // Intentional for FFI
#![allow(clippy::cast_possible_wrap)] // Intentional for FFI
#![allow(clippy::missing_const_for_fn)] // extern "C" functions can't be const
#![allow(clippy::must_use_candidate)] // Internal helper functions

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Constants
// =============================================================================

/// Maximum length of key sequence to be mapped.
pub const MAXMAPLEN: usize = 50;

/// Maximum number of hash buckets for mappings.
pub const MAX_MAPHASH: c_int = 256;

/// Mode flag constants (matching C enum in `state_defs.h`).
pub const MODE_NORMAL: c_int = 0x01;
pub const MODE_VISUAL: c_int = 0x02;
pub const MODE_OP_PENDING: c_int = 0x04;
pub const MODE_CMDLINE: c_int = 0x08;
pub const MODE_INSERT: c_int = 0x10;
pub const MODE_LANGMAP: c_int = 0x20;
pub const MODE_SELECT: c_int = 0x40;
pub const MODE_TERMINAL: c_int = 0x80;

/// All mode bits used for mapping.
pub const MAP_ALL_MODES: c_int = 0xff;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to a mapping block (`mapblock_T*`).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapblockHandle(*mut c_void);

impl MapblockHandle {
    /// Create a new mapblock handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `mapblock_T*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a Neovim buffer (`buf_T*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufHandle(*mut c_void);

impl BufHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// C accessor functions for mapblock_T fields
// =============================================================================

extern "C" {
    // Field accessors for MapblockHandle
    fn nvim_mapblock_get_next(mp: MapblockHandle) -> MapblockHandle;
    fn nvim_mapblock_get_alt(mp: MapblockHandle) -> MapblockHandle;
    fn nvim_mapblock_get_keys(mp: MapblockHandle) -> *const c_char;
    fn nvim_mapblock_get_str(mp: MapblockHandle) -> *const c_char;
    fn nvim_mapblock_get_orig_str(mp: MapblockHandle) -> *const c_char;
    fn nvim_mapblock_get_keylen(mp: MapblockHandle) -> c_int;
    fn nvim_mapblock_get_mode(mp: MapblockHandle) -> c_int;
    fn nvim_mapblock_get_simplified(mp: MapblockHandle) -> c_int;
    fn nvim_mapblock_get_noremap(mp: MapblockHandle) -> c_int;
    fn nvim_mapblock_is_silent(mp: MapblockHandle) -> c_int;
    fn nvim_mapblock_is_nowait(mp: MapblockHandle) -> c_int;
    fn nvim_mapblock_is_expr(mp: MapblockHandle) -> c_int;
    fn nvim_mapblock_get_luaref(mp: MapblockHandle) -> c_int;
    fn nvim_mapblock_get_desc(mp: MapblockHandle) -> *const c_char;
    fn nvim_mapblock_get_replace_keycodes(mp: MapblockHandle) -> c_int;

    // Hash table accessors
    fn nvim_get_maphash_entry(index: c_int) -> MapblockHandle;
    fn nvim_get_first_abbr() -> MapblockHandle;
    fn nvim_buf_get_maphash_entry(buf: BufHandle, index: c_int) -> MapblockHandle;
    fn nvim_buf_get_first_abbr(buf: BufHandle) -> MapblockHandle;

    // Global state
    fn nvim_get_curbuf() -> BufHandle;
}

// =============================================================================
// Hash Function
// =============================================================================

/// Compute the hash value for a mapping.
///
/// The hash is computed from the mode and first character of the LHS.
/// Normal/Visual mode mappings are mostly separated from Insert/Cmdline mode.
///
/// # Arguments
/// * `mode` - The mode bits for the mapping
/// * `c1` - The first character of the mapping's LHS (as unsigned byte)
///
/// # Returns
/// A value between 0 and 255, used as index in the maphash table.
#[inline]
#[must_use]
pub const fn map_hash(mode: c_int, c1: u8) -> c_int {
    // Put Normal/Visual mode mappings mostly separately from Insert/Cmdline mode.
    // MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING | MODE_TERMINAL
    let nv_modes = MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING | MODE_TERMINAL;
    if (mode & nv_modes) != 0 {
        c1 as c_int
    } else {
        (c1 ^ 0x80) as c_int
    }
}

/// FFI wrapper for `MAP_HASH` macro.
///
/// # Safety
/// This function is safe to call with any values.
#[no_mangle]
pub extern "C" fn rs_map_hash(mode: c_int, c1: c_int) -> c_int {
    map_hash(mode, c1 as u8)
}

// =============================================================================
// Mode to/from string conversion
// =============================================================================

/// Put characters to represent the map mode in a buffer.
///
/// The output buffer must be at least 7 bytes (including NUL).
///
/// # Arguments
/// * `mode` - The mode bits for the mapping
/// * `buf` - Output buffer for the mode string
///
/// # Safety
/// The buffer must be at least 7 bytes long.
#[no_mangle]
pub unsafe extern "C" fn rs_map_mode_to_chars(mode: c_int, buf: *mut c_char) {
    if buf.is_null() {
        return;
    }

    let mut p = buf;

    if (mode & (MODE_INSERT | MODE_CMDLINE)) == (MODE_INSERT | MODE_CMDLINE) {
        *p = b'!' as c_char;
        p = p.add(1);
    } else if (mode & MODE_INSERT) != 0 {
        *p = b'i' as c_char;
        p = p.add(1);
    } else if (mode & MODE_LANGMAP) != 0 {
        *p = b'l' as c_char;
        p = p.add(1);
    } else if (mode & MODE_CMDLINE) != 0 {
        *p = b'c' as c_char;
        p = p.add(1);
    } else if (mode & (MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING))
        == (MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING)
    {
        *p = b' ' as c_char;
        p = p.add(1);
    } else {
        if (mode & MODE_NORMAL) != 0 {
            *p = b'n' as c_char;
            p = p.add(1);
        }
        if (mode & MODE_OP_PENDING) != 0 {
            *p = b'o' as c_char;
            p = p.add(1);
        }
        if (mode & MODE_TERMINAL) != 0 {
            *p = b't' as c_char;
            p = p.add(1);
        }
        if (mode & (MODE_VISUAL | MODE_SELECT)) == (MODE_VISUAL | MODE_SELECT) {
            *p = b'v' as c_char;
            p = p.add(1);
        } else {
            if (mode & MODE_VISUAL) != 0 {
                *p = b'x' as c_char;
                p = p.add(1);
            }
            if (mode & MODE_SELECT) != 0 {
                *p = b's' as c_char;
                p = p.add(1);
            }
        }
    }

    // NUL-terminate
    *p = 0;
}

/// Get mode bits from a mode string.
///
/// # Arguments
/// * `modechars` - NUL-terminated string of mode characters
///
/// # Returns
/// The mode bits, or -1 if the string contains an invalid mode character.
///
/// # Safety
/// `modechars` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_map_mode_from_chars(modechars: *const c_char) -> c_int {
    if modechars.is_null() {
        return -1;
    }

    let mut mode: c_int = 0;
    let mut p = modechars;

    while *p != 0 {
        let c = *p as u8;
        match c {
            b'n' => mode |= MODE_NORMAL,
            b'v' => mode |= MODE_VISUAL | MODE_SELECT,
            b'x' => mode |= MODE_VISUAL,
            b's' => mode |= MODE_SELECT,
            b'o' => mode |= MODE_OP_PENDING,
            b'i' => mode |= MODE_INSERT,
            b'l' => mode |= MODE_LANGMAP,
            b'c' => mode |= MODE_CMDLINE,
            b't' => mode |= MODE_TERMINAL,
            b'!' => mode |= MODE_INSERT | MODE_CMDLINE,
            b' ' => mode |= MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING,
            _ => return -1, // Invalid mode character
        }
        p = p.add(1);
    }

    mode
}

// =============================================================================
// Hash List Accessors
// =============================================================================

/// Get the start of the hashed map list for "state" and first character "c".
///
/// # Arguments
/// * `state` - The mode bits
/// * `c` - The first character of the mapping's LHS
///
/// # Returns
/// The first mapblock in the hash bucket, or null if empty.
#[no_mangle]
pub extern "C" fn rs_get_maphash_list(state: c_int, c: c_int) -> MapblockHandle {
    let hash = map_hash(state, c as u8);
    // SAFETY: We trust the C accessor function to return a valid handle.
    unsafe { nvim_get_maphash_entry(hash) }
}

/// Get the buffer-local hashed map list for "state" and first character "c".
///
/// # Arguments
/// * `buf` - The buffer handle (uses curbuf if null)
/// * `state` - The mode bits
/// * `c` - The first character of the mapping's LHS
///
/// # Returns
/// The first mapblock in the buffer-local hash bucket, or null if empty.
#[no_mangle]
pub extern "C" fn rs_get_buf_maphash_list(
    buf: BufHandle,
    state: c_int,
    c: c_int,
) -> MapblockHandle {
    let hash = map_hash(state, c as u8);
    let actual_buf = if buf.is_null() {
        // SAFETY: We trust the C accessor function.
        unsafe { nvim_get_curbuf() }
    } else {
        buf
    };
    // SAFETY: We trust the C accessor function to return a valid handle.
    unsafe { nvim_buf_get_maphash_entry(actual_buf, hash) }
}

// =============================================================================
// Mapblock field accessors (safe wrappers)
// =============================================================================

/// Get the next mapblock in the list.
#[inline]
pub fn mapblock_next(mp: MapblockHandle) -> MapblockHandle {
    if mp.is_null() {
        return mp;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_mapblock_get_next(mp) }
}

/// Get the alternate mapblock (for simplified key mappings).
#[inline]
pub fn mapblock_alt(mp: MapblockHandle) -> MapblockHandle {
    if mp.is_null() {
        return mp;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_mapblock_get_alt(mp) }
}

/// Get the keys (LHS) of the mapping.
#[inline]
pub fn mapblock_keys(mp: MapblockHandle) -> *const c_char {
    if mp.is_null() {
        return std::ptr::null();
    }
    // SAFETY: We check for null above.
    unsafe { nvim_mapblock_get_keys(mp) }
}

/// Get the mapping string (RHS) of the mapping.
#[inline]
pub fn mapblock_str(mp: MapblockHandle) -> *const c_char {
    if mp.is_null() {
        return std::ptr::null();
    }
    // SAFETY: We check for null above.
    unsafe { nvim_mapblock_get_str(mp) }
}

/// Get the original RHS string of the mapping.
#[inline]
pub fn mapblock_orig_str(mp: MapblockHandle) -> *const c_char {
    if mp.is_null() {
        return std::ptr::null();
    }
    // SAFETY: We check for null above.
    unsafe { nvim_mapblock_get_orig_str(mp) }
}

/// Get the key length of the mapping.
#[inline]
pub fn mapblock_keylen(mp: MapblockHandle) -> c_int {
    if mp.is_null() {
        return 0;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_mapblock_get_keylen(mp) }
}

/// Get the mode bits of the mapping.
#[inline]
pub fn mapblock_mode(mp: MapblockHandle) -> c_int {
    if mp.is_null() {
        return 0;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_mapblock_get_mode(mp) }
}

/// Check if the mapping was simplified.
#[inline]
pub fn mapblock_simplified(mp: MapblockHandle) -> bool {
    if mp.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_mapblock_get_simplified(mp) != 0 }
}

/// Get the noremap value of the mapping.
#[inline]
pub fn mapblock_noremap(mp: MapblockHandle) -> c_int {
    if mp.is_null() {
        return 0;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_mapblock_get_noremap(mp) }
}

/// Check if the mapping is silent.
#[inline]
pub fn mapblock_silent(mp: MapblockHandle) -> bool {
    if mp.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_mapblock_is_silent(mp) != 0 }
}

/// Check if the mapping has nowait flag.
#[inline]
pub fn mapblock_nowait(mp: MapblockHandle) -> bool {
    if mp.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_mapblock_is_nowait(mp) != 0 }
}

/// Check if the mapping is an expression.
#[inline]
pub fn mapblock_expr(mp: MapblockHandle) -> bool {
    if mp.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_mapblock_is_expr(mp) != 0 }
}

/// Get the Lua reference of the mapping (or LUA_NOREF).
#[inline]
pub fn mapblock_luaref(mp: MapblockHandle) -> c_int {
    if mp.is_null() {
        return -1; // LUA_NOREF
    }
    // SAFETY: We check for null above.
    unsafe { nvim_mapblock_get_luaref(mp) }
}

/// Get the description of the mapping.
#[inline]
pub fn mapblock_desc(mp: MapblockHandle) -> *const c_char {
    if mp.is_null() {
        return std::ptr::null();
    }
    // SAFETY: We check for null above.
    unsafe { nvim_mapblock_get_desc(mp) }
}

/// Check if the mapping should replace keycodes in expression result.
#[inline]
pub fn mapblock_replace_keycodes(mp: MapblockHandle) -> bool {
    if mp.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_mapblock_get_replace_keycodes(mp) != 0 }
}

// =============================================================================
// Query Functions
// =============================================================================

/// Check if a mapping exists for the given keys and mode.
///
/// This is used by `mapcheck()` and `hasmapto()`.
///
/// # Arguments
/// * `keys` - The LHS keys to check
/// * `mode` - The mode bits to check
/// * `abbr` - True to check abbreviations, false for mappings
///
/// # Returns
/// True if a mapping exists, false otherwise.
///
/// # Safety
/// `keys` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_map_to_exists(keys: *const c_char, mode: c_int, abbr: c_int) -> c_int {
    if keys.is_null() {
        return 0;
    }

    let is_abbr = abbr != 0;

    // Check global mappings
    if map_to_exists_impl(keys, mode, is_abbr, BufHandle(std::ptr::null_mut())) {
        return 1;
    }

    // Check buffer-local mappings
    let curbuf = nvim_get_curbuf();
    if map_to_exists_impl(keys, mode, is_abbr, curbuf) {
        return 1;
    }

    0
}

/// Internal implementation of map_to_exists.
unsafe fn map_to_exists_impl(
    keys: *const c_char,
    mode: c_int,
    is_abbr: bool,
    buf: BufHandle,
) -> bool {
    let is_buffer_local = !buf.is_null();

    if is_abbr {
        // Only one abbreviation list
        let mp = if is_buffer_local {
            nvim_buf_get_first_abbr(buf)
        } else {
            nvim_get_first_abbr()
        };
        return check_mapping_list_for_rhs(mp, keys, mode);
    }

    // Check all hash buckets
    for hash in 0..MAX_MAPHASH {
        let mp = if is_buffer_local {
            nvim_buf_get_maphash_entry(buf, hash)
        } else {
            nvim_get_maphash_entry(hash)
        };
        if check_mapping_list_for_rhs(mp, keys, mode) {
            return true;
        }
    }

    false
}

/// Compare two C strings for equality.
///
/// # Safety
/// Both pointers must be valid, non-null, NUL-terminated C strings.
#[inline]
unsafe fn c_str_eq(a: *const c_char, b: *const c_char) -> bool {
    let mut pa = a;
    let mut pb = b;
    loop {
        let ca = *pa;
        let cb = *pb;
        if ca != cb {
            return false;
        }
        if ca == 0 {
            return true;
        }
        pa = pa.add(1);
        pb = pb.add(1);
    }
}

/// Check a mapping list for an RHS that matches the given keys.
unsafe fn check_mapping_list_for_rhs(
    start: MapblockHandle,
    keys: *const c_char,
    mode: c_int,
) -> bool {
    let mut mp = start;
    while !mp.is_null() {
        // Check if mode matches
        if (mapblock_mode(mp) & mode) != 0 {
            // Check if RHS matches
            let rhs = mapblock_str(mp);
            if !rhs.is_null() && c_str_eq(rhs, keys) {
                return true;
            }
        }
        mp = mapblock_next(mp);
    }
    false
}

/// Check if a substring exists in a C string.
///
/// # Safety
/// Both pointers must be valid, non-null, NUL-terminated C strings.
#[inline]
unsafe fn c_strstr(haystack: *const c_char, needle: *const c_char) -> bool {
    if *needle == 0 {
        return true; // Empty needle always matches
    }

    let mut h = haystack;
    while *h != 0 {
        let mut n = needle;
        let mut temp = h;
        while *n != 0 && *temp != 0 && *n == *temp {
            n = n.add(1);
            temp = temp.add(1);
        }
        if *n == 0 {
            return true; // Found the needle
        }
        h = h.add(1);
    }
    false
}

/// Check if a mapping exists with the given RHS substring (hasmapto support).
///
/// This implements the core logic of map_to_exists_mode - checking if any
/// mapping has an RHS that contains the given string.
///
/// # Arguments
/// * `rhs` - The RHS substring to search for
/// * `mode` - The mode bits to check
/// * `abbr` - True to check abbreviations, false for mappings
///
/// # Returns
/// True if a mapping with matching RHS substring exists, false otherwise.
///
/// # Safety
/// `rhs` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_map_to_exists_mode(
    rhs: *const c_char,
    mode: c_int,
    abbr: c_int,
) -> c_int {
    if rhs.is_null() {
        return 0;
    }

    let is_abbr = abbr != 0;

    // Check global mappings
    if map_to_exists_mode_impl(rhs, mode, is_abbr, BufHandle(std::ptr::null_mut())) {
        return 1;
    }

    // Check buffer-local mappings
    let curbuf = nvim_get_curbuf();
    if map_to_exists_mode_impl(rhs, mode, is_abbr, curbuf) {
        return 1;
    }

    0
}

/// Internal implementation of map_to_exists_mode.
unsafe fn map_to_exists_mode_impl(
    rhs: *const c_char,
    mode: c_int,
    is_abbr: bool,
    buf: BufHandle,
) -> bool {
    let is_buffer_local = !buf.is_null();

    if is_abbr {
        // Only one abbreviation list
        let mp = if is_buffer_local {
            nvim_buf_get_first_abbr(buf)
        } else {
            nvim_get_first_abbr()
        };
        return check_mapping_list_for_rhs_substring(mp, rhs, mode);
    }

    // Check all hash buckets
    for hash in 0..MAX_MAPHASH {
        let mp = if is_buffer_local {
            nvim_buf_get_maphash_entry(buf, hash)
        } else {
            nvim_get_maphash_entry(hash)
        };
        if check_mapping_list_for_rhs_substring(mp, rhs, mode) {
            return true;
        }
    }

    false
}

/// Check a mapping list for an RHS that contains the given substring.
unsafe fn check_mapping_list_for_rhs_substring(
    start: MapblockHandle,
    rhs_substr: *const c_char,
    mode: c_int,
) -> bool {
    let mut mp = start;
    while !mp.is_null() {
        // Check if mode matches
        if (mapblock_mode(mp) & mode) != 0 {
            // Check if RHS contains the substring
            let mp_rhs = mapblock_str(mp);
            if !mp_rhs.is_null() && c_strstr(mp_rhs, rhs_substr) {
                return true;
            }
        }
        mp = mapblock_next(mp);
    }
    false
}

/// Count the number of mappings for a given mode.
///
/// # Arguments
/// * `mode` - The mode bits to count
/// * `abbr` - True to count abbreviations, false for mappings
/// * `local` - True to count buffer-local mappings only
///
/// # Returns
/// The number of mappings matching the criteria.
///
/// # Safety
/// This function accesses global state through FFI. It is safe to call
/// as long as the calling C code has properly initialized the mapping tables.
#[no_mangle]
pub unsafe extern "C" fn rs_map_count(mode: c_int, abbr: c_int, local: c_int) -> c_int {
    let is_abbr = abbr != 0;
    let is_local = local != 0;
    let mut count: c_int = 0;

    if is_abbr {
        // Only one abbreviation list
        let mp = if is_local {
            nvim_buf_get_first_abbr(nvim_get_curbuf())
        } else {
            nvim_get_first_abbr()
        };
        count += count_mappings_in_list(mp, mode);
    } else {
        // Check all hash buckets
        for hash in 0..MAX_MAPHASH {
            let mp = if is_local {
                nvim_buf_get_maphash_entry(nvim_get_curbuf(), hash)
            } else {
                nvim_get_maphash_entry(hash)
            };
            count += count_mappings_in_list(mp, mode);
        }
    }

    count
}

/// Count mappings in a list that match the given mode.
unsafe fn count_mappings_in_list(start: MapblockHandle, mode: c_int) -> c_int {
    let mut count: c_int = 0;
    let mut mp = start;
    while !mp.is_null() {
        // Count if mode matches and not simplified
        if (mapblock_mode(mp) & mode) != 0 && !mapblock_simplified(mp) {
            count += 1;
        }
        mp = mapblock_next(mp);
    }
    count
}

// =============================================================================
// Mode String Validation (for mapset)
// =============================================================================

/// Mode bitmask for visual + select modes.
const MASK_V: c_int = MODE_VISUAL | MODE_SELECT;

/// Mode bitmask for normal map modes (n/v/s/o).
const MASK_MAP: c_int = MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING;

/// Mode bitmask for insert + cmdline (bang) modes.
const MASK_BANG: c_int = MODE_INSERT | MODE_CMDLINE;

/// Get mode bits from a mode string with validation.
///
/// This function is used by `mapset()` and validates that the mode string
/// is valid for either a mapping or an abbreviation.
///
/// # Arguments
/// * `mode_string` - NUL-terminated string of mode characters
/// * `abbr` - true for abbreviations, false for mappings
///
/// # Returns
/// The mode bits, or 0 if the mode string is invalid.
///
/// # Safety
/// `mode_string` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_map_mode_string(mode_string: *const c_char, abbr: c_int) -> c_int {
    if mode_string.is_null() {
        return 0;
    }

    let mut p = mode_string;

    // Compatibility: empty string is treated as " " (all normal-like modes)
    if *p == 0 {
        p = c" ".as_ptr();
    }

    let mut mode: c_int = 0;

    while *p != 0 {
        let c = *p as u8;
        let tmode = match c {
            b'i' => MODE_INSERT,
            b'l' => MODE_LANGMAP,
            b'c' => MODE_CMDLINE,
            b'n' => MODE_NORMAL,
            b'x' => MODE_VISUAL,
            b's' => MODE_SELECT,
            b'o' => MODE_OP_PENDING,
            b't' => MODE_TERMINAL,
            b'v' => MASK_V,
            b'!' => MASK_BANG,
            b' ' => MASK_MAP,
            _ => return 0, // Unknown mode character
        };
        mode |= tmode;
        p = p.add(1);
    }

    // Validation
    let is_abbr = abbr != 0;
    if is_abbr {
        // Abbreviations can only use insert and/or cmdline modes
        if (mode & !MASK_BANG) != 0 {
            return 0;
        }
    } else {
        // For mappings, if multiple bits are set, they must form a valid combination
        let has_multiple_bits = (mode & (mode - 1)) != 0;
        if has_multiple_bits {
            // Check if mode is fully contained in one of the allowed masks
            let in_bang = (mode & MASK_BANG) != 0 && (mode & !MASK_BANG) == 0;
            let in_map = (mode & MASK_MAP) != 0 && (mode & !MASK_MAP) == 0;
            if !in_bang && !in_map {
                return 0;
            }
        }
    }

    mode
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_hash() {
        // Normal mode should use c1 directly
        assert_eq!(map_hash(MODE_NORMAL, b'a'), c_int::from(b'a'));
        assert_eq!(map_hash(MODE_VISUAL, b'b'), c_int::from(b'b'));
        assert_eq!(map_hash(MODE_TERMINAL, b'c'), c_int::from(b'c'));

        // Insert/Cmdline mode should XOR with 0x80
        assert_eq!(map_hash(MODE_INSERT, b'a'), c_int::from(b'a' ^ 0x80));
        assert_eq!(map_hash(MODE_CMDLINE, b'a'), c_int::from(b'a' ^ 0x80));

        // Combined modes with normal should use c1 directly
        assert_eq!(map_hash(MODE_NORMAL | MODE_INSERT, b'x'), c_int::from(b'x'));
    }

    #[test]
    fn test_mode_to_chars() {
        let mut buf = [0i8; 7];

        // Test single modes
        unsafe {
            rs_map_mode_to_chars(MODE_NORMAL, buf.as_mut_ptr());
            assert_eq!(buf[0], b'n' as i8);
            assert_eq!(buf[1], 0);

            rs_map_mode_to_chars(MODE_VISUAL, buf.as_mut_ptr());
            assert_eq!(buf[0], b'x' as i8);
            assert_eq!(buf[1], 0);

            rs_map_mode_to_chars(MODE_INSERT, buf.as_mut_ptr());
            assert_eq!(buf[0], b'i' as i8);
            assert_eq!(buf[1], 0);

            rs_map_mode_to_chars(MODE_CMDLINE, buf.as_mut_ptr());
            assert_eq!(buf[0], b'c' as i8);
            assert_eq!(buf[1], 0);

            // Test combined insert + cmdline = '!'
            rs_map_mode_to_chars(MODE_INSERT | MODE_CMDLINE, buf.as_mut_ptr());
            assert_eq!(buf[0], b'!' as i8);
            assert_eq!(buf[1], 0);

            // Test all normal-like modes = ' '
            rs_map_mode_to_chars(
                MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING,
                buf.as_mut_ptr(),
            );
            assert_eq!(buf[0], b' ' as i8);
            assert_eq!(buf[1], 0);
        }
    }

    #[test]
    fn test_mode_from_chars() {
        unsafe {
            assert_eq!(rs_map_mode_from_chars(c"n".as_ptr()), MODE_NORMAL);
            assert_eq!(rs_map_mode_from_chars(c"i".as_ptr()), MODE_INSERT);
            assert_eq!(rs_map_mode_from_chars(c"c".as_ptr()), MODE_CMDLINE);
            assert_eq!(
                rs_map_mode_from_chars(c"!".as_ptr()),
                MODE_INSERT | MODE_CMDLINE
            );
            assert_eq!(
                rs_map_mode_from_chars(c"nvo".as_ptr()),
                MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING
            );
            assert_eq!(rs_map_mode_from_chars(c"x".as_ptr()), MODE_VISUAL);
            assert_eq!(rs_map_mode_from_chars(c"s".as_ptr()), MODE_SELECT);
            assert_eq!(
                rs_map_mode_from_chars(c"v".as_ptr()),
                MODE_VISUAL | MODE_SELECT
            );

            // Invalid mode character
            assert_eq!(rs_map_mode_from_chars(c"z".as_ptr()), -1);

            // Null pointer
            assert_eq!(rs_map_mode_from_chars(std::ptr::null()), -1);
        }
    }

    #[test]
    fn test_mapblock_handle_null() {
        // Test handle construction and null check
        // (Note: Testing accessor functions requires C library linkage,
        //  which is verified in integration tests)
        let null_handle = MapblockHandle(std::ptr::null_mut());
        assert!(null_handle.is_null());

        let non_null = unsafe { MapblockHandle::from_ptr(std::ptr::dangling_mut::<c_void>()) };
        assert!(!non_null.is_null());
        assert_eq!(non_null.as_ptr(), std::ptr::dangling_mut::<c_void>());
    }

    #[test]
    fn test_buf_handle_null() {
        let null_handle = BufHandle(std::ptr::null_mut());
        assert!(null_handle.is_null());

        let non_null = BufHandle(std::ptr::dangling_mut::<c_void>());
        assert!(!non_null.is_null());
    }

    #[test]
    fn test_c_str_eq() {
        unsafe {
            // Equal strings
            assert!(c_str_eq(c"hello".as_ptr(), c"hello".as_ptr()));
            assert!(c_str_eq(c"".as_ptr(), c"".as_ptr()));

            // Unequal strings
            assert!(!c_str_eq(c"hello".as_ptr(), c"world".as_ptr()));
            assert!(!c_str_eq(c"hello".as_ptr(), c"hell".as_ptr()));
            assert!(!c_str_eq(c"hell".as_ptr(), c"hello".as_ptr()));
        }
    }

    #[test]
    fn test_c_strstr() {
        unsafe {
            // Empty needle always matches
            assert!(c_strstr(c"hello".as_ptr(), c"".as_ptr()));
            assert!(c_strstr(c"".as_ptr(), c"".as_ptr()));

            // Substring at start
            assert!(c_strstr(c"hello world".as_ptr(), c"hello".as_ptr()));

            // Substring in middle
            assert!(c_strstr(c"hello world".as_ptr(), c"lo wo".as_ptr()));

            // Substring at end
            assert!(c_strstr(c"hello world".as_ptr(), c"world".as_ptr()));

            // Exact match
            assert!(c_strstr(c"hello".as_ptr(), c"hello".as_ptr()));

            // No match
            assert!(!c_strstr(c"hello".as_ptr(), c"xyz".as_ptr()));

            // Needle longer than haystack
            assert!(!c_strstr(c"hi".as_ptr(), c"hello".as_ptr()));
        }
    }

    #[test]
    fn test_get_map_mode_string_mappings() {
        unsafe {
            // Single modes for mappings
            assert_eq!(rs_get_map_mode_string(c"n".as_ptr(), 0), MODE_NORMAL);
            assert_eq!(rs_get_map_mode_string(c"i".as_ptr(), 0), MODE_INSERT);
            assert_eq!(rs_get_map_mode_string(c"c".as_ptr(), 0), MODE_CMDLINE);
            assert_eq!(rs_get_map_mode_string(c"x".as_ptr(), 0), MODE_VISUAL);
            assert_eq!(rs_get_map_mode_string(c"s".as_ptr(), 0), MODE_SELECT);
            assert_eq!(rs_get_map_mode_string(c"o".as_ptr(), 0), MODE_OP_PENDING);
            assert_eq!(rs_get_map_mode_string(c"t".as_ptr(), 0), MODE_TERMINAL);
            assert_eq!(rs_get_map_mode_string(c"l".as_ptr(), 0), MODE_LANGMAP);

            // Combined modes
            assert_eq!(
                rs_get_map_mode_string(c"v".as_ptr(), 0),
                MODE_VISUAL | MODE_SELECT
            );
            assert_eq!(
                rs_get_map_mode_string(c"!".as_ptr(), 0),
                MODE_INSERT | MODE_CMDLINE
            );
            assert_eq!(
                rs_get_map_mode_string(c" ".as_ptr(), 0),
                MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING
            );

            // Empty string = " "
            assert_eq!(
                rs_get_map_mode_string(c"".as_ptr(), 0),
                MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING
            );

            // Invalid mode character
            assert_eq!(rs_get_map_mode_string(c"z".as_ptr(), 0), 0);

            // Invalid combination (normal + insert, not in same mask)
            assert_eq!(rs_get_map_mode_string(c"ni".as_ptr(), 0), 0);
        }
    }

    #[test]
    fn test_get_map_mode_string_abbreviations() {
        unsafe {
            // Abbreviations can only use insert and/or cmdline modes
            assert_eq!(rs_get_map_mode_string(c"i".as_ptr(), 1), MODE_INSERT);
            assert_eq!(rs_get_map_mode_string(c"c".as_ptr(), 1), MODE_CMDLINE);
            assert_eq!(
                rs_get_map_mode_string(c"!".as_ptr(), 1),
                MODE_INSERT | MODE_CMDLINE
            );
            assert_eq!(
                rs_get_map_mode_string(c"ic".as_ptr(), 1),
                MODE_INSERT | MODE_CMDLINE
            );

            // Normal mode is invalid for abbreviations
            assert_eq!(rs_get_map_mode_string(c"n".as_ptr(), 1), 0);
            assert_eq!(rs_get_map_mode_string(c"x".as_ptr(), 1), 0);
            assert_eq!(rs_get_map_mode_string(c" ".as_ptr(), 1), 0);
        }
    }
}
