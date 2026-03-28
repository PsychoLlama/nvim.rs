//! Typeahead buffer implementation
//!
//! This module provides the Rust implementation of the typeahead buffer
//! used for storing and managing typed characters, mapping results, and
//! special key sequences.

// Allow integer casts that are safe given the constraints of the typeahead buffer
// (buffer sizes are bounded by MAXMAPLEN and fit comfortably in i32)
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use std::ffi::{c_int, c_void};

/// Maximum length for a key mapping sequence (matches C MAXMAPLEN = 50).
const fn maxmaplen() -> i32 {
    MAXMAPLEN_VAL as i32
}

/// Remapping flags for typeahead buffer entries.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemapFlag {
    /// Allow remapping
    Yes = 0,
    /// Don't remap
    None = 1,
    /// Remap script-local mappings only
    Script = 2,
    /// Don't remap, do abbrev
    Abbr = 4,
}

impl From<u8> for RemapFlag {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::None,
            2 => Self::Script,
            4 => Self::Abbr,
            _ => Self::Yes,
        }
    }
}

/// Values for the "noremap" argument of ins_typebuf()
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemapValues {
    /// Allow remapping
    Yes = 0,
    /// No remapping
    None = -1,
    /// Remap script-local mappings only
    Script = -2,
    /// No remapping for first char
    Skip = -3,
}

impl From<c_int> for RemapValues {
    fn from(value: c_int) -> Self {
        match value {
            0 => Self::Yes,
            -1 => Self::None,
            -2 => Self::Script,
            -3 => Self::Skip,
            _ => {
                // Positive values indicate number of chars not to remap
                if value > 0 {
                    // This case is handled specially - return None as default
                    Self::None
                } else {
                    Self::Yes
                }
            }
        }
    }
}

// =============================================================================
// Phase 2: TypebufT repr(C) struct and direct global access
// =============================================================================

/// MAXMAPLEN constant value (matches C: 50)
const MAXMAPLEN_VAL: usize = 50;

/// TYPELEN_INIT = 5 * (MAXMAPLEN + 3) = 5 * 53 = 265
const TYPELEN_INIT: usize = 5 * (MAXMAPLEN_VAL + 3);

/// NSCRIPT: maximum number of nested script files
const NSCRIPT: usize = 15;

/// Mirror of C `typebuf_T` from getchar_defs.h.
///
/// Layout: 2 pointers (16 bytes) + 7 ints (28 bytes) = 44 bytes on 64-bit.
/// With alignment, the struct is 48 bytes (matches C sizeof(typebuf_T)).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct TypebufT {
    /// Buffer for typed characters
    pub tb_buf: *mut u8,
    /// Mapping flags for characters in tb_buf[]
    pub tb_noremap: *mut u8,
    /// Size of tb_buf[]
    pub tb_buflen: c_int,
    /// Current position in tb_buf[]
    pub tb_off: c_int,
    /// Number of valid bytes in tb_buf[]
    pub tb_len: c_int,
    /// Number of mapped bytes in tb_buf[]
    pub tb_maplen: c_int,
    /// Number of silently mapped bytes in tb_buf[]
    pub tb_silent: c_int,
    /// Number of bytes without abbrev. in tb_buf[]
    pub tb_no_abbr_cnt: c_int,
    /// Counter for buffer changes; never zero
    pub tb_change_cnt: c_int,
}

// SAFETY: Neovim is single-threaded; these statics are only accessed from the main thread.
unsafe impl Send for TypebufT {}

extern "C" {
    /// The C `typebuf` global (direct access)
    static mut typebuf: TypebufT;
    /// The C `typebuf_was_filled` global
    static mut typebuf_was_filled: bool;
    /// xmalloc: allocate memory
    fn xmalloc(size: usize) -> *mut u8;
    /// xrealloc: reallocate memory
    fn xrealloc(ptr: *mut u8, size: usize) -> *mut u8;
    /// xfree: free memory
    fn xfree(ptr: *mut c_void);
    /// internal_error: report internal error
    fn internal_error(msg: *const std::ffi::c_char);
    /// curscript: index in scriptin (non-static in C after Phase 3)
    static curscript: c_int;
    /// cmd_silent: don't echo the command line
    static mut cmd_silent: bool;
}

/// Initial buffer for typebuf when not allocated (avoids early malloc).
/// This is the Rust equivalent of C's `static uint8_t typebuf_init[TYPELEN_INIT]`.
static mut TYPEBUF_INIT: [u8; TYPELEN_INIT] = [0u8; TYPELEN_INIT];

/// Initial noremap buffer for typebuf (mirrors C's `noremapbuf_init`).
static mut NOREMAPBUF_INIT: [u8; TYPELEN_INIT] = [0u8; TYPELEN_INIT];

/// Saved typebuf array for nested script files (mirrors C's `saved_typebuf[NSCRIPT]`).
static mut SAVED_TYPEBUF: [TypebufT; NSCRIPT] = [TypebufT {
    tb_buf: std::ptr::null_mut(),
    tb_noremap: std::ptr::null_mut(),
    tb_buflen: 0,
    tb_off: 0,
    tb_len: 0,
    tb_maplen: 0,
    tb_silent: 0,
    tb_no_abbr_cnt: 0,
    tb_change_cnt: 0,
}; NSCRIPT];

/// Initialize `typebuf.tb_buf` to point to `TYPEBUF_INIT`.
/// Replaces C `init_typebuf()`.
///
/// # Safety
/// Accesses `typebuf` C global and `TYPEBUF_INIT` static.
pub unsafe fn rs_init_typebuf_impl() {
    if !typebuf.tb_buf.is_null() {
        return;
    }
    typebuf.tb_buf = std::ptr::addr_of_mut!(TYPEBUF_INIT[0]);
    typebuf.tb_noremap = std::ptr::addr_of_mut!(NOREMAPBUF_INIT[0]);
    typebuf.tb_buflen = TYPELEN_INIT as c_int;
    typebuf.tb_len = 0;
    typebuf.tb_off = (MAXMAPLEN_VAL + 4) as c_int;
    typebuf.tb_change_cnt = 1;
}

/// Make `typebuf` empty and allocate new buffers.
/// Replaces C `alloc_typebuf()`.
///
/// # Safety
/// Accesses `typebuf` C global.
pub unsafe fn rs_alloc_typebuf_impl() {
    typebuf.tb_buf = xmalloc(TYPELEN_INIT);
    typebuf.tb_noremap = xmalloc(TYPELEN_INIT);
    typebuf.tb_buflen = TYPELEN_INIT as c_int;
    typebuf.tb_off = (MAXMAPLEN_VAL + 4) as c_int;
    typebuf.tb_len = 0;
    typebuf.tb_maplen = 0;
    typebuf.tb_silent = 0;
    typebuf.tb_no_abbr_cnt = 0;
    typebuf.tb_change_cnt = typebuf.tb_change_cnt.wrapping_add(1);
    if typebuf.tb_change_cnt == 0 {
        typebuf.tb_change_cnt = 1;
    }
}

/// Free the buffers of `typebuf`.
/// Replaces C `free_typebuf()`.
///
/// # Safety
/// Accesses `typebuf` C global and `TYPEBUF_INIT`/`NOREMAPBUF_INIT` statics.
pub unsafe fn rs_free_typebuf_impl() {
    if typebuf.tb_buf == std::ptr::addr_of_mut!(TYPEBUF_INIT[0]) {
        internal_error(c"Free typebuf 1".as_ptr());
    } else {
        xfree(typebuf.tb_buf.cast::<c_void>());
        typebuf.tb_buf = std::ptr::null_mut();
    }
    if typebuf.tb_noremap == std::ptr::addr_of_mut!(NOREMAPBUF_INIT[0]) {
        internal_error(c"Free typebuf 2".as_ptr());
    } else {
        xfree(typebuf.tb_noremap.cast::<c_void>());
        typebuf.tb_noremap = std::ptr::null_mut();
    }
}

/// Save current typebuf to `SAVED_TYPEBUF[curscript]`.
/// Replaces C `save_typebuf()`.
///
/// # Safety
/// Accesses `typebuf`, `SAVED_TYPEBUF`, and calls `rs_init_typebuf_impl`.
///
/// # Panics
/// Panics if `curscript < 0` (should not happen during normal script execution).
pub unsafe fn rs_save_typebuf_impl() {
    let cs = curscript;
    assert!(cs >= 0, "save_typebuf called with curscript < 0");
    rs_init_typebuf_impl();
    SAVED_TYPEBUF[cs as usize] = typebuf;
    rs_alloc_typebuf_impl();
}

/// Free `typebuf` buffers and restore from `SAVED_TYPEBUF[curscript]`.
/// Used by `closescript()` in C.
///
/// # Safety
/// Accesses `typebuf`, `SAVED_TYPEBUF`, and `curscript`.
///
/// # Panics
/// Panics if `curscript < 0`.
pub unsafe fn rs_close_typebuf_impl() {
    let cs = curscript;
    assert!(cs >= 0, "rs_close_typebuf called with curscript < 0");
    rs_free_typebuf_impl();
    typebuf = SAVED_TYPEBUF[cs as usize];
}

/// Exported C-callable wrapper for `closescript` typebuf restore.
///
/// # Safety
/// Accesses `typebuf`, `SAVED_TYPEBUF`, and `curscript`.
#[no_mangle]
pub unsafe extern "C" fn rs_close_typebuf() {
    rs_close_typebuf_impl();
}

/// Exported C-callable `init_typebuf()`.
///
/// # Safety
/// Accesses C global `typebuf`.
#[no_mangle]
pub unsafe extern "C" fn rs_init_typebuf() {
    rs_init_typebuf_impl();
}

/// Exported C-callable `alloc_typebuf()`.
///
/// # Safety
/// Accesses C global `typebuf`.
#[no_mangle]
pub unsafe extern "C" fn rs_alloc_typebuf() {
    rs_alloc_typebuf_impl();
}

/// Exported C-callable `free_typebuf()`.
///
/// # Safety
/// Accesses C global `typebuf`.
#[no_mangle]
pub unsafe extern "C" fn rs_free_typebuf() {
    rs_free_typebuf_impl();
}

/// Exported C-callable `save_typebuf()`.
///
/// # Safety
/// Accesses C global `typebuf`.
#[no_mangle]
pub unsafe extern "C" fn rs_save_typebuf() {
    rs_save_typebuf_impl();
}

// =============================================================================
// Phase 2 end
// =============================================================================

/// Typeahead buffer structure.
///
/// This mirrors the C `typebuf_T` structure and manages the buffer of
/// characters that have been typed but not yet consumed.
///
/// # Buffer Layout
///
/// The buffer has three logical parts:
/// 1. Room in front (for inserting mapping results)
/// 2. The current typeahead content
/// 3. Room at the end (for new characters)
///
/// The layout is:
/// ```text
/// [unused front space][mapped chars][typed chars][unused end space]
///                      ^-- tb_off
///                      |<-- tb_maplen -->|
///                      |<-------- tb_len -------->|
/// ```
#[derive(Debug)]
pub struct TypeaheadBuffer {
    /// Buffer for typed characters
    buf: Vec<u8>,
    /// Mapping flags for characters in buf
    noremap: Vec<u8>,
    /// Current position in buf (offset to first valid char)
    off: i32,
    /// Number of valid bytes in buf
    len: i32,
    /// Number of mapped bytes at start of valid region
    maplen: i32,
    /// Number of silently mapped bytes at start
    silent: i32,
    /// Number of bytes without abbreviation at start
    no_abbr_cnt: i32,
    /// Counter for detecting buffer changes; never zero
    change_cnt: i32,
}

impl Default for TypeaheadBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeaheadBuffer {
    /// Initial buffer size for typeahead
    const TYPELEN_INIT: usize = 5 * (256 + 3); // 5 * (MAXMAPLEN + 3)

    /// Create a new, empty typeahead buffer.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn new() -> Self {
        let initial_off: usize = 256 + 4; // MAXMAPLEN + 4
        let mut buf = vec![0u8; Self::TYPELEN_INIT];
        let noremap = vec![0u8; Self::TYPELEN_INIT];

        // Ensure NUL termination at initial position
        buf[initial_off] = 0;

        Self {
            buf,
            noremap,
            off: initial_off as i32,
            len: 0,
            maplen: 0,
            silent: 0,
            no_abbr_cnt: 0,
            change_cnt: 1,
        }
    }

    /// Initialize the buffer if needed.
    ///
    /// This is called before any operation that requires the buffer to be valid.
    pub const fn init(&mut self) {
        // Already initialized via new()
    }

    /// Returns true if the buffer is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of characters in the buffer.
    #[must_use]
    pub const fn len(&self) -> i32 {
        self.len
    }

    /// Returns the number of mapped characters at the start.
    #[must_use]
    pub const fn maplen(&self) -> i32 {
        self.maplen
    }

    /// Returns the number of silent characters at the start.
    #[must_use]
    pub const fn silent(&self) -> i32 {
        self.silent
    }

    /// Returns the current change counter.
    #[must_use]
    pub const fn change_cnt(&self) -> i32 {
        self.change_cnt
    }

    /// Returns true if there are no characters that have not been typed
    /// (i.e., no mapping results in the buffer).
    #[must_use]
    pub const fn typed(&self) -> bool {
        self.maplen == 0
    }

    /// Get a byte at the given offset from the start of valid content.
    #[must_use]
    pub fn get_byte(&self, offset: i32) -> Option<u8> {
        if offset < 0 || offset >= self.len {
            return None;
        }
        let idx = (self.off + offset) as usize;
        self.buf.get(idx).copied()
    }

    /// Get a slice of the buffer content.
    #[must_use]
    pub fn content(&self) -> &[u8] {
        let start = self.off as usize;
        let end = start + self.len as usize;
        &self.buf[start..end]
    }

    /// Get the remap flag at the given offset.
    #[must_use]
    pub fn get_noremap(&self, offset: i32) -> RemapFlag {
        if offset < 0 || offset >= self.len {
            return RemapFlag::Yes;
        }
        let idx = (self.off + offset) as usize;
        self.noremap
            .get(idx)
            .copied()
            .map_or(RemapFlag::Yes, RemapFlag::from)
    }

    /// Increment the change counter, wrapping to 1 if it would become 0.
    const fn increment_change_cnt(&mut self) {
        self.change_cnt = self.change_cnt.wrapping_add(1);
        if self.change_cnt == 0 {
            self.change_cnt = 1;
        }
    }

    /// Insert a string into the typeahead buffer at the given offset.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to insert
    /// * `noremap` - Remapping control:
    ///   - `REMAP_YES` (0): new string can be mapped again
    ///   - `REMAP_NONE` (-1): new string cannot be mapped
    ///   - `REMAP_SCRIPT` (-2): only script-local mappings
    ///   - `REMAP_SKIP` (-3): first char cannot be mapped
    ///   - `> 0`: that many characters cannot be mapped
    /// * `offset` - Position in the buffer to insert at
    /// * `nottyped` - If true, characters are marked as not typed
    /// * `silent_flag` - If true, cmd_silent will be set when chars are read
    ///
    /// # Returns
    ///
    /// `true` on success, `false` on failure (string too long)
    pub fn insert(
        &mut self,
        s: &[u8],
        noremap: c_int,
        offset: i32,
        nottyped: bool,
        silent_flag: bool,
    ) -> bool {
        self.increment_change_cnt();

        let addlen = s.len() as i32;
        if addlen == 0 {
            return true;
        }

        let maxmaplen = maxmaplen();

        // Check if there's room in front of the current content
        if offset == 0 && addlen <= self.off {
            // Easy case: room in front
            self.off -= addlen;
            self.buf[self.off as usize..(self.off + addlen) as usize].copy_from_slice(s);
        } else if self.len == 0 && self.buf.len() >= (addlen + 3 * (maxmaplen + 4)) as usize {
            // Buffer is empty and string fits
            self.off = ((self.buf.len() as i32 - addlen - 3 * (maxmaplen + 4)) / 2).max(0);
            self.buf[self.off as usize..(self.off + addlen) as usize].copy_from_slice(s);
        } else {
            // Need to reallocate or shift content
            let newoff = maxmaplen + 4;
            let extra = addlen + newoff + 4 * (maxmaplen + 4);

            if self.len > i32::MAX - extra {
                // String would be too long
                return false;
            }

            let newlen = (self.len + extra) as usize;
            let mut new_buf = vec![0u8; newlen];
            let mut new_noremap = vec![0u8; newlen];

            // Copy old chars before insertion point
            let offset_usize = offset as usize;
            new_buf[newoff as usize..(newoff as usize + offset_usize)]
                .copy_from_slice(&self.buf[self.off as usize..(self.off as usize + offset_usize)]);

            // Copy new chars
            new_buf[(newoff as usize + offset_usize)..(newoff as usize + offset_usize + s.len())]
                .copy_from_slice(s);

            // Copy old chars after insertion point (including NUL)
            let old_after_offset = self.off as usize + offset_usize;
            let bytes_after = (self.len - offset + 1) as usize;
            new_buf[(newoff as usize + offset_usize + s.len())
                ..(newoff as usize + offset_usize + s.len() + bytes_after)]
                .copy_from_slice(&self.buf[old_after_offset..(old_after_offset + bytes_after)]);

            // Copy noremap flags
            new_noremap[newoff as usize..(newoff as usize + offset_usize)].copy_from_slice(
                &self.noremap[self.off as usize..(self.off as usize + offset_usize)],
            );
            let after_insert = newoff as usize + offset_usize + s.len();
            let old_noremap_after = (self.len - offset) as usize;
            new_noremap[after_insert..(after_insert + old_noremap_after)].copy_from_slice(
                &self.noremap[(self.off as usize + offset_usize)
                    ..(self.off as usize + offset_usize + old_noremap_after)],
            );

            self.buf = new_buf;
            self.noremap = new_noremap;
            self.off = newoff;
        }

        self.len += addlen;

        // Determine noremap value and count
        let val = if noremap == RemapValues::Script as c_int {
            RemapFlag::Script as u8
        } else if noremap == RemapValues::Skip as c_int {
            RemapFlag::Abbr as u8
        } else {
            RemapFlag::None as u8
        };

        let nrm = if noremap == RemapValues::Skip as c_int {
            1
        } else if noremap < 0 {
            addlen
        } else {
            noremap
        };

        // Set noremap flags for inserted characters
        for i in 0..addlen {
            let idx = (self.off + i + offset) as usize;
            self.noremap[idx] = if i < nrm { val } else { RemapFlag::Yes as u8 };
        }

        // Adjust maplen and silent counts
        if nottyped || self.maplen > offset {
            self.maplen += addlen;
        }
        if silent_flag || self.silent > offset {
            self.silent += addlen;
        }
        if self.no_abbr_cnt > 0 && offset == 0 {
            self.no_abbr_cnt += addlen;
        }

        true
    }

    /// Remove `len` characters from the buffer at the given offset.
    pub fn delete(&mut self, len: i32, offset: i32) {
        if len == 0 {
            return;
        }

        self.len -= len;

        let maxmaplen = maxmaplen();

        // Easy case: just increase offset
        if offset == 0 && (self.buf.len() as i32 - (self.off + len)) >= 3 * maxmaplen + 3 {
            self.off += len;
        } else {
            // Need to move characters
            let i = self.off + offset;

            // Leave some extra room at the end
            if self.off > maxmaplen {
                // Move content before deletion point to new position
                self.buf.copy_within(
                    self.off as usize..(self.off + offset) as usize,
                    maxmaplen as usize,
                );
                self.noremap.copy_within(
                    self.off as usize..(self.off + offset) as usize,
                    maxmaplen as usize,
                );
                self.off = maxmaplen;
            }

            // Move content after deletion point
            let bytes = (self.len - offset + 1) as usize;
            let src_start = (i + len) as usize;
            let dst_start = (self.off + offset) as usize;
            self.buf
                .copy_within(src_start..(src_start + bytes), dst_start);

            let noremap_bytes = (self.len - offset) as usize;
            self.noremap
                .copy_within(src_start..(src_start + noremap_bytes), dst_start);
        }

        // Adjust maplen
        if self.maplen > offset {
            if self.maplen < offset + len {
                self.maplen = offset;
            } else {
                self.maplen -= len;
            }
        }

        // Adjust silent
        if self.silent > offset {
            if self.silent < offset + len {
                self.silent = offset;
            } else {
                self.silent -= len;
            }
        }

        // Adjust no_abbr_cnt
        if self.no_abbr_cnt > offset {
            if self.no_abbr_cnt < offset + len {
                self.no_abbr_cnt = offset;
            } else {
                self.no_abbr_cnt -= len;
            }
        }

        self.increment_change_cnt();
    }

    /// Clear the buffer, keeping allocations.
    pub const fn clear(&mut self) {
        let maxmaplen = maxmaplen();
        self.off = maxmaplen + 4;
        self.len = 0;
        self.maplen = 0;
        self.silent = 0;
        self.no_abbr_cnt = 0;
        self.increment_change_cnt();
    }

    /// Flush mapped characters from the start of the buffer.
    pub const fn flush_mapped(&mut self) {
        self.off += self.maplen;
        self.len -= self.maplen;
        self.maplen = 0;
        self.silent = 0;
        self.no_abbr_cnt = 0;
        self.increment_change_cnt();
    }
}

// =============================================================================
// C FFI Accessor Functions
// (typebuf fields now accessed directly via `typebuf` global declared above)
// =============================================================================

/// Check if a typeahead change has occurred.
///
/// Returns true if `tb_change_cnt` changed or `typebuf_was_filled` is true.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_typebuf_was_changed(old_change_cnt: c_int) -> c_int {
    if old_change_cnt == 0 {
        return 0;
    }

    let current = typebuf.tb_change_cnt;
    let was_filled = typebuf_was_filled;

    c_int::from(current != old_change_cnt || was_filled)
}

/// Increment the typeahead buffer change counter.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn increment_typebuf_change_cnt() {
    typebuf.tb_change_cnt = typebuf.tb_change_cnt.wrapping_add(1);
    if typebuf.tb_change_cnt == 0 {
        typebuf.tb_change_cnt = 1;
    }
}

/// Delete characters from the typeahead buffer.
///
/// Removes `len` characters starting at `offset` from `typebuf.tb_off`.
///
/// # Safety
/// Calls C accessor functions and performs pointer operations.
#[export_name = "del_typebuf"]
pub unsafe extern "C" fn rs_del_typebuf(len: c_int, offset: c_int) {
    if len == 0 {
        return;
    }

    let maxmaplen = maxmaplen();
    let mut tb_off = typebuf.tb_off;
    let tb_len = typebuf.tb_len - len;
    let tb_buflen = typebuf.tb_buflen;

    typebuf.tb_len = tb_len;

    // Easy case: just increase tb_off
    if offset == 0 && tb_buflen - (tb_off + len) >= 3 * maxmaplen + 3 {
        typebuf.tb_off = tb_off + len;
    } else {
        // Need to move characters
        let tb_buf = typebuf.tb_buf;
        let tb_noremap = typebuf.tb_noremap;
        let i = tb_off + offset;

        // Leave some extra room at the end
        if tb_off > maxmaplen {
            // Move content before deletion point to new position
            std::ptr::copy(
                tb_buf.offset(tb_off as isize),
                tb_buf.offset(maxmaplen as isize),
                offset as usize,
            );
            std::ptr::copy(
                tb_noremap.offset(tb_off as isize),
                tb_noremap.offset(maxmaplen as isize),
                offset as usize,
            );
            tb_off = maxmaplen;
            typebuf.tb_off = tb_off;
        }

        // Move content after deletion point (including NUL)
        let bytes = (tb_len - offset + 1) as usize;
        std::ptr::copy(
            tb_buf.offset((i + len) as isize),
            tb_buf.offset((tb_off + offset) as isize),
            bytes,
        );
        // Move noremap flags
        let noremap_bytes = (tb_len - offset) as usize;
        std::ptr::copy(
            tb_noremap.offset((i + len) as isize),
            tb_noremap.offset((tb_off + offset) as isize),
            noremap_bytes,
        );
    }

    // Adjust maplen
    if typebuf.tb_maplen > offset {
        if typebuf.tb_maplen < offset + len {
            typebuf.tb_maplen = offset;
        } else {
            typebuf.tb_maplen -= len;
        }
    }

    // Adjust silent
    if typebuf.tb_silent > offset {
        if typebuf.tb_silent < offset + len {
            typebuf.tb_silent = offset;
        } else {
            typebuf.tb_silent -= len;
        }
    }

    // Adjust no_abbr_cnt
    if typebuf.tb_no_abbr_cnt > offset {
        if typebuf.tb_no_abbr_cnt < offset + len {
            typebuf.tb_no_abbr_cnt = offset;
        } else {
            typebuf.tb_no_abbr_cnt -= len;
        }
    }

    // Reset typebuf_was_filled flag
    typebuf_was_filled = false;
    increment_typebuf_change_cnt();
}

/// Flush mapped characters from the typeahead buffer (minimal flush).
///
/// Removes only the mapped characters at the start of the buffer,
/// leaving typed characters intact.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_flush_typebuf_mapped() {
    typebuf.tb_off += typebuf.tb_maplen;
    typebuf.tb_len -= typebuf.tb_maplen;
    typebuf.tb_maplen = 0;
    typebuf.tb_silent = 0;
    cmd_silent = false;
    typebuf.tb_no_abbr_cnt = 0;
    increment_typebuf_change_cnt();
}

/// Clear all typeahead.
///
/// Resets the typeahead buffer to empty state.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_typebuf() {
    typebuf.tb_off = maxmaplen();
    typebuf.tb_len = 0;
    typebuf.tb_maplen = 0;
    typebuf.tb_silent = 0;
    typebuf.tb_no_abbr_cnt = 0;
    typebuf_was_filled = false;
    increment_typebuf_change_cnt();
}

/// Get a byte from the typeahead buffer at the given offset.
///
/// # Safety
/// Calls C accessor functions and performs pointer operations.
#[no_mangle]
pub unsafe extern "C" fn rs_typebuf_get_byte(offset: c_int) -> c_int {
    if offset < 0 || offset >= typebuf.tb_len {
        return -1;
    }

    c_int::from(*typebuf.tb_buf.offset((typebuf.tb_off + offset) as isize))
}

/// Get the remap flag at the given offset in the typeahead buffer.
///
/// # Safety
/// Calls C accessor functions and performs pointer operations.
#[no_mangle]
pub unsafe extern "C" fn rs_typebuf_get_noremap(offset: c_int) -> c_int {
    if offset < 0 || offset >= typebuf.tb_len {
        return RemapFlag::Yes as c_int;
    }

    c_int::from(
        *typebuf
            .tb_noremap
            .offset((typebuf.tb_off + offset) as isize),
    )
}

/// Get the current typeahead buffer length.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_typebuf_len() -> c_int {
    typebuf.tb_len
}

/// Get the current typeahead buffer offset.
///
/// # Safety
/// Reads C global `typebuf` directly.
#[no_mangle]
pub unsafe extern "C" fn rs_typebuf_off() -> c_int {
    typebuf.tb_off
}

/// Check if the typeahead buffer is empty.
///
/// # Safety
/// Reads C global `typebuf` directly.
#[no_mangle]
pub unsafe extern "C" fn rs_typebuf_is_empty() -> c_int {
    c_int::from(typebuf.tb_len == 0)
}

/// Check if the first character in the typeahead should use silent mode.
///
/// # Safety
/// Reads C global `typebuf` directly.
#[no_mangle]
pub unsafe extern "C" fn rs_typebuf_is_silent() -> c_int {
    c_int::from(typebuf.tb_silent > 0)
}

/// Insert a string into the typeahead buffer.
///
/// This is the Rust implementation of ins_typebuf().
///
/// # Arguments
/// * `str` - NUL-terminated string to insert
/// * `noremap` - Remapping control:
///   - 0 (REMAP_YES): new string can be mapped again
///   - -1 (REMAP_NONE): new string cannot be mapped
///   - -2 (REMAP_SCRIPT): only script-local mappings
///   - -3 (REMAP_SKIP): first char cannot be mapped
///   - > 0: that many characters cannot be mapped
/// * `offset` - Position in the buffer to insert at
/// * `nottyped` - If true, characters are marked as not typed
/// * `silent` - If true, cmd_silent will be set when chars are read
///
/// # Returns
/// OK (1) on success, FAIL (0) if the string is too long.
///
/// # Safety
/// * `str` must be a valid NUL-terminated string pointer.
/// * Calls C accessor functions and performs pointer operations.
#[no_mangle]
#[allow(clippy::too_many_lines)] // Port of C ins_typebuf() which is also large
pub unsafe extern "C" fn rs_ins_typebuf(
    str: *const u8,
    noremap: c_int,
    offset: c_int,
    nottyped: c_int,
    silent: c_int,
) -> c_int {
    // Initialize typebuf if needed
    rs_init_typebuf_impl();

    // Increment change counter
    increment_typebuf_change_cnt();

    // Notify state is no longer safe
    state_no_longer_safe(c"rs_ins_typebuf()".as_ptr());

    if str.is_null() {
        return OK;
    }

    // Get string length (count bytes until NUL)
    let addlen = {
        let mut len: c_int = 0;
        let mut p = str;
        while *p != 0 {
            len += 1;
            p = p.add(1);
        }
        len
    };
    if addlen == 0 {
        return OK;
    }

    let maxmaplen = maxmaplen();

    // Easy case: there is room in front of typebuf.tb_buf[typebuf.tb_off]
    if offset == 0 && addlen <= typebuf.tb_off {
        let new_off = typebuf.tb_off - addlen;
        typebuf.tb_off = new_off;
        std::ptr::copy_nonoverlapping(
            str,
            typebuf.tb_buf.offset(new_off as isize),
            addlen as usize,
        );
    } else if typebuf.tb_len == 0 && typebuf.tb_buflen >= addlen + 3 * (maxmaplen + 4) {
        // Buffer is empty and string fits in the existing buffer.
        // Leave some space before and after, if possible.
        let new_off = ((typebuf.tb_buflen - addlen - 3 * (maxmaplen + 4)) / 2).max(0);
        typebuf.tb_off = new_off;
        std::ptr::copy_nonoverlapping(
            str,
            typebuf.tb_buf.offset(new_off as isize),
            addlen as usize,
        );
    } else {
        // Need to allocate more room for the buffer
        let newoff = maxmaplen + 4;
        let extra = addlen + newoff + 4 * (maxmaplen + 4);

        if typebuf.tb_len > i32::MAX - extra {
            // String is too long
            return FAIL;
        }

        // Grow the typeahead buffer via xrealloc
        let new_buflen = typebuf.tb_len + extra;
        let new_buf = xrealloc(typebuf.tb_buf, new_buflen as usize);
        let new_noremap = xrealloc(typebuf.tb_noremap, new_buflen as usize);
        if new_buf.is_null() || new_noremap.is_null() {
            return FAIL;
        }
        typebuf.tb_buf = new_buf;
        typebuf.tb_noremap = new_noremap;
        typebuf.tb_buflen = new_buflen;

        let old_off = typebuf.tb_off;

        // Copy existing content to new position, making room for insertion
        // First: bytes before offset
        if offset > 0 {
            std::ptr::copy(
                typebuf.tb_buf.offset(old_off as isize),
                typebuf.tb_buf.offset(newoff as isize),
                offset as usize,
            );
            std::ptr::copy(
                typebuf.tb_noremap.offset(old_off as isize),
                typebuf.tb_noremap.offset(newoff as isize),
                offset as usize,
            );
        }

        // Copy new string at offset position
        std::ptr::copy_nonoverlapping(
            str,
            typebuf.tb_buf.offset((newoff + offset) as isize),
            addlen as usize,
        );

        // Copy bytes after offset (including NUL)
        let bytes_after = (typebuf.tb_len - offset + 1) as usize;
        std::ptr::copy(
            typebuf.tb_buf.offset((old_off + offset) as isize),
            typebuf.tb_buf.offset((newoff + offset + addlen) as isize),
            bytes_after,
        );
        let noremap_after = (typebuf.tb_len - offset) as usize;
        std::ptr::copy(
            typebuf.tb_noremap.offset((old_off + offset) as isize),
            typebuf
                .tb_noremap
                .offset((newoff + offset + addlen) as isize),
            noremap_after,
        );

        typebuf.tb_off = newoff;
    }

    // Update length
    let new_len = typebuf.tb_len + addlen;
    typebuf.tb_len = new_len;

    // Set the NUL terminator
    *typebuf.tb_buf.offset((typebuf.tb_off + new_len) as isize) = 0;

    // Determine remap value and count for noremap flags
    let (val, nrm) = match noremap {
        _ if noremap == REMAP_SCRIPT => (RM_SCRIPT, addlen),
        _ if noremap == REMAP_SKIP => (RM_ABBR, 1),
        _ if noremap < 0 => (RM_NONE, addlen),
        _ if noremap > 0 => (RM_NONE, noremap),
        _ => (RM_YES, 0),
    };

    // Set noremap flags for the inserted characters
    for i in 0..addlen {
        let idx = typebuf.tb_off + i + offset;
        *typebuf.tb_noremap.offset(idx as isize) = if i < nrm { val } else { RM_YES };
    }

    // Adjust maplen if characters were not typed (or offset is within mapped region)
    if nottyped != 0 || typebuf.tb_maplen > offset {
        typebuf.tb_maplen += addlen;
    }

    // Adjust silent if needed
    if silent != 0 || typebuf.tb_silent > offset {
        typebuf.tb_silent += addlen;
    }

    // Adjust no_abbr_cnt if needed (when inserting at the beginning)
    if offset == 0 && typebuf.tb_no_abbr_cnt != 0 {
        typebuf.tb_no_abbr_cnt += addlen;
    }

    OK
}

/// `ins_typebuf(char*, int, int, bool, bool)` -- Phase 2 export replacing C implementation
///
/// Accepts C-ABI-compatible `bool` for `nottyped` and `silent`, delegating to
/// `rs_ins_typebuf` which uses `c_int`.
///
/// # Safety
/// `str` must be a valid NUL-terminated string pointer. Calls C accessor functions.
#[must_use]
#[export_name = "ins_typebuf"]
pub unsafe extern "C" fn ins_typebuf_export(
    str: *const u8,
    noremap: c_int,
    offset: c_int,
    nottyped: bool,
    silent: bool,
) -> c_int {
    rs_ins_typebuf(
        str,
        noremap,
        offset,
        c_int::from(nottyped),
        c_int::from(silent),
    )
}

// Remap constants matching C
const RM_YES: u8 = 0;
const RM_NONE: u8 = 1;
const RM_SCRIPT: u8 = 2;
const RM_ABBR: u8 = 4;

// noremap parameter constants
const REMAP_SCRIPT: c_int = -2;
const REMAP_SKIP: c_int = -3;

// Return codes
const OK: c_int = 1;
const FAIL: c_int = 0;

// Additional C function declarations needed
extern "C" {
    fn state_no_longer_safe(reason: *const std::ffi::c_char);
    /// Read input characters into buf (up to maxlen), waiting wait_time ms.
    fn inchar(buf: *mut u8, maxlen: c_int, wait_time: std::ffi::c_long) -> c_int;
}

/// `flush_buffers_T` enum constants -- must match C enum order
const FLUSH_MINIMAL: c_int = 0;
// FLUSH_TYPEAHEAD = 1 is the implicit else branch (clear typebuf without draining inchar)
const FLUSH_INPUT: c_int = 2;

/// `flush_buffers(flush_buffers_T)` -- Phase 3 export replacing C implementation
///
/// Remove the contents of the stuff buffer and the mapped characters in the
/// typeahead buffer (used in case of an error). If `flush_typeahead` is
/// `FLUSH_INPUT`, also flush all typeahead characters (used when interrupted
/// by CTRL-C).
///
/// # Safety
/// Calls C accessor functions and performs pointer operations.
#[export_name = "flush_buffers"]
pub unsafe extern "C" fn flush_buffers_export(flush_typeahead: c_int) {
    rs_init_typebuf_impl();

    crate::buffheader::rs_start_stuff();
    while crate::buffheader::rs_read_readbuffers(1) != 0 {}

    if flush_typeahead == FLUSH_MINIMAL {
        // Remove only mapped characters at the start
        typebuf.tb_off += typebuf.tb_maplen;
        typebuf.tb_len -= typebuf.tb_maplen;
    } else {
        // Remove all typeahead
        if flush_typeahead == FLUSH_INPUT {
            // Drain all pending input chars (may be part of escape sequence)
            while inchar(typebuf.tb_buf, typebuf.tb_buflen - 1, 10) != 0 {}
        }
        typebuf.tb_off = maxmaplen();
        typebuf.tb_len = 0;
        // Reset the flag that text received from a client or feedkeys() was
        // inserted in the typeahead buffer.
        typebuf_was_filled = false;
    }

    typebuf.tb_maplen = 0;
    typebuf.tb_silent = 0;
    cmd_silent = false;
    typebuf.tb_no_abbr_cnt = 0;
    increment_typebuf_change_cnt();
}

// =============================================================================
// Phase 5: put_string_in_typebuf and at_ins_compl_key (migrated from C)
// =============================================================================

extern "C" {
    fn rs_ctrl_x_mode_not_default() -> c_int;
    fn rs_compl_status_local() -> c_int;
    fn rs_ins_compl_pum_key(c: c_int) -> c_int;
    fn rs_vim_is_ctrl_x_key(c: c_int) -> c_int;
}

/// K_SPECIAL byte (0x80)
const K_SPECIAL_BYTE: u8 = 0x80;
/// KS_MODIFIER
const KS_MODIFIER_BYTE: u8 = 252;
/// MOD_MASK_CTRL
const MOD_MASK_CTRL: u8 = 0x04;
/// Ctrl-N and Ctrl-P
const CTRL_N: c_int = 14;
const CTRL_P: c_int = 16;
/// REMAP_YES = 0
const REMAP_YES: c_int = 0;

/// `put_string_in_typebuf(int, int, uint8_t*, int)` -- Phase 5 export replacing C static
///
/// Put `string[new_slen]` in typebuf, replacing `slen` bytes at `offset`.
/// Returns FAIL for error, OK otherwise.
///
/// # Safety
/// `string` must point to at least `new_slen + 1` bytes.
/// Calls C accessor functions.
#[must_use]
#[export_name = "put_string_in_typebuf"]
pub unsafe extern "C" fn put_string_in_typebuf_export(
    offset: c_int,
    slen: c_int,
    string: *mut u8,
    new_slen: c_int,
) -> c_int {
    let extra = new_slen - slen;
    *string.add(new_slen as usize) = 0; // NUL-terminate

    if extra < 0 {
        rs_del_typebuf(-extra, offset);
    } else if extra > 0 {
        let str_part = string.add(slen as usize);
        if rs_ins_typebuf(str_part, REMAP_YES, offset, 0, 0) == FAIL {
            return FAIL;
        }
    }

    // Copy the new content into place (del/ins_typebuf may have reallocated)
    std::ptr::copy(
        string,
        typebuf.tb_buf.add((typebuf.tb_off + offset) as usize),
        new_slen as usize,
    );

    OK
}

/// `at_ins_compl_key(void)` -- Phase 5 export replacing C static
///
/// Check if the bytes at the start of the typeahead buffer are a character used
/// in Insert mode completion. Includes the form with a CTRL modifier.
///
/// # Safety
/// Calls C accessor functions.
#[must_use]
#[export_name = "at_ins_compl_key"]
pub unsafe extern "C" fn at_ins_compl_key_export() -> bool {
    let p = typebuf.tb_buf.add(typebuf.tb_off as usize);
    let c = if typebuf.tb_len > 3
        && *p == K_SPECIAL_BYTE
        && *p.add(1) == KS_MODIFIER_BYTE
        && (*p.add(2) & MOD_MASK_CTRL != 0)
    {
        c_int::from(*p.add(3) & 0x1f)
    } else {
        c_int::from(*p)
    };

    (rs_ctrl_x_mode_not_default() != 0
        && (rs_ins_compl_pum_key(c) != 0 || rs_vim_is_ctrl_x_key(c) != 0))
        || (rs_compl_status_local() != 0 && (c == CTRL_N || c == CTRL_P))
}

// =============================================================================
// Phase 3: check_simplify_modifier and no_reduce_keys
// =============================================================================

/// no_reduce_keys: do not apply modifiers to the key (moved from C to Rust).
/// Exported `#[no_mangle]` so C can reference it via extern.
#[no_mangle]
pub static mut no_reduce_keys: c_int = 0;

/// Increment no_reduce_keys (called from C getchar_common when !simplify).
///
/// # Safety
/// Accesses `no_reduce_keys` static.
#[no_mangle]
pub unsafe extern "C" fn rs_inc_no_reduce_keys() {
    no_reduce_keys += 1;
}

/// Decrement no_reduce_keys (called from C getchar_common when !simplify).
///
/// # Safety
/// Accesses `no_reduce_keys` static.
#[no_mangle]
pub unsafe extern "C" fn rs_dec_no_reduce_keys() {
    no_reduce_keys -= 1;
}

// Constants for check_simplify_modifier

/// K_SPECIAL byte (0x80)
const K_SPECIAL_VAL: u8 = 0x80;
/// KS_MODIFIER (252)
const KS_MODIFIER_VAL: u8 = 252;
/// MODE_TERMINAL flag (matches C MODE_TERMINAL = 0x4000)
const MODE_TERMINAL: c_int = 0x4000;
/// MB_MAXBYTES = 21 (max bytes for a multibyte char)
const MB_MAXBYTES: usize = 21;

extern "C" {
    /// merge_modifiers: apply modifier to key, update modifier mask, return new key
    fn merge_modifiers(c: c_int, modifiers: *mut c_int) -> c_int;
    /// utf_char2bytes: encode Unicode char to UTF-8 bytes, returns byte count
    fn utf_char2bytes(c: c_int, buf: *mut std::ffi::c_char) -> c_int;
    /// vgetc_char: the character from vgetc (direct C global access)
    static mut vgetc_char: c_int;
    /// vgetc_mod_mask: the mod_mask from vgetc (direct C global access)
    static mut vgetc_mod_mask: c_int;
    /// Get State global (for MODE_TERMINAL check)
    fn nvim_get_state() -> c_int;
}

/// Convert a special key code to its 3-byte encoding.
/// Returns the second and third bytes for K_SPECIAL encoding.
/// Matches C `K_SECOND(c)` and `K_THIRD(c)`.
const fn key2termcap(c: c_int) -> (u8, u8) {
    let neg_c = (-c) as u32;
    let b0 = (neg_c & 0xff) as u8;
    let b1 = ((neg_c >> 8) & 0xff) as u8;
    (b0, b1)
}

/// Check if typebuf contains a modifier+key sequence that can be simplified.
///
/// Checks from `typebuf.tb_off` to `typebuf.tb_off + max_offset`.
///
/// # Returns
/// Length of replaced bytes, 0 if nothing changed, -1 for error.
///
/// # Safety
/// Accesses `typebuf`, `no_reduce_keys`, and calls C functions.
#[no_mangle]
pub unsafe extern "C" fn rs_check_simplify_modifier(max_offset: c_int) -> c_int {
    // We want full modifiers in Terminal mode or when no_reduce_keys > 0
    if (nvim_get_state() & MODE_TERMINAL) != 0 || no_reduce_keys > 0 {
        return 0;
    }

    for offset in 0..max_offset {
        if offset + 3 >= typebuf.tb_len {
            break;
        }
        let tp = typebuf.tb_buf.add((typebuf.tb_off + offset) as usize);
        if *tp == K_SPECIAL_VAL && *tp.add(1) == KS_MODIFIER_VAL {
            // A modifier was not used for a mapping, apply it to ASCII keys
            let mut modifier = c_int::from(*tp.add(2));
            let c = c_int::from(*tp.add(3));
            let new_c = merge_modifiers(c, &raw mut modifier);

            if new_c != c {
                if offset == 0 {
                    // At the start: remember character and mod_mask before merging
                    vgetc_char = c;
                    vgetc_mod_mask = c_int::from(*tp.add(2));
                }

                let mut new_string = [0u8; MB_MAXBYTES + 1];
                let len: c_int = if new_c < 0 {
                    // IS_SPECIAL(new_c): encode as 3-byte sequence
                    new_string[0] = K_SPECIAL_VAL;
                    let (b1, b2) = key2termcap(new_c);
                    new_string[1] = b1;
                    new_string[2] = b2;
                    3
                } else {
                    utf_char2bytes(new_c, new_string.as_mut_ptr().cast())
                };
                if modifier == 0 {
                    if put_string_in_typebuf_export(offset, 4, new_string.as_mut_ptr(), len) == FAIL
                    {
                        return -1;
                    }
                } else {
                    *tp.add(2) = modifier as u8;
                    if put_string_in_typebuf_export(offset + 3, 1, new_string.as_mut_ptr(), len)
                        == FAIL
                    {
                        return -1;
                    }
                }
                return len;
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buf = TypeaheadBuffer::new();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.maplen(), 0);
        assert!(buf.typed());
    }

    #[test]
    fn test_remap_flag_conversion() {
        assert_eq!(RemapFlag::from(0), RemapFlag::Yes);
        assert_eq!(RemapFlag::from(1), RemapFlag::None);
        assert_eq!(RemapFlag::from(2), RemapFlag::Script);
        assert_eq!(RemapFlag::from(4), RemapFlag::Abbr);
        assert_eq!(RemapFlag::from(255), RemapFlag::Yes); // Unknown defaults to Yes
    }

    #[test]
    fn test_remap_values_conversion() {
        assert_eq!(RemapValues::from(0), RemapValues::Yes);
        assert_eq!(RemapValues::from(-1), RemapValues::None);
        assert_eq!(RemapValues::from(-2), RemapValues::Script);
        assert_eq!(RemapValues::from(-3), RemapValues::Skip);
    }
}
