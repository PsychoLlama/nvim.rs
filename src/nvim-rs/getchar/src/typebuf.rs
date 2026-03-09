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

use std::ffi::c_int;

// Import MAXMAPLEN from C
extern "C" {
    /// Get the MAXMAPLEN constant from C
    fn nvim_get_maxmaplen() -> c_int;
}

/// Maximum length for a key mapping sequence.
/// This is lazily initialized from C's MAXMAPLEN.
fn maxmaplen() -> i32 {
    // SAFETY: This is a simple accessor for a C constant
    unsafe { nvim_get_maxmaplen() }
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
    pub fn clear(&mut self) {
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
// =============================================================================

#[allow(dead_code)]
extern "C" {
    fn nvim_get_typebuf_change_cnt() -> c_int;
    fn nvim_get_typebuf_was_filled() -> c_int;
    fn nvim_set_typebuf_was_filled(val: c_int);
    fn nvim_get_typebuf_buf() -> *mut u8;
    fn nvim_get_typebuf_noremap() -> *mut u8;
    fn nvim_get_typebuf_buflen() -> c_int;
    fn nvim_get_typebuf_off() -> c_int;
    fn nvim_get_typebuf_len() -> c_int;
    fn nvim_get_typebuf_maplen() -> c_int;
    fn nvim_get_typebuf_silent() -> c_int;
    fn nvim_get_typebuf_no_abbr_cnt() -> c_int;
    fn nvim_set_typebuf_off(val: c_int);
    fn nvim_set_typebuf_len(val: c_int);
    fn nvim_set_typebuf_maplen(val: c_int);
    fn nvim_set_typebuf_silent(val: c_int);
    fn nvim_set_typebuf_no_abbr_cnt(val: c_int);
    fn nvim_set_typebuf_change_cnt(val: c_int);
    // nvim_get_maxmaplen already declared at module level
    fn nvim_get_cmd_silent() -> c_int;
    fn nvim_set_cmd_silent(val: c_int);
}

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

    let current = nvim_get_typebuf_change_cnt();
    let was_filled = nvim_get_typebuf_was_filled();

    c_int::from(current != old_change_cnt || was_filled != 0)
}

/// Increment the typeahead buffer change counter.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn increment_typebuf_change_cnt() {
    let mut cnt = nvim_get_typebuf_change_cnt().wrapping_add(1);
    if cnt == 0 {
        cnt = 1;
    }
    nvim_set_typebuf_change_cnt(cnt);
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

    let maxmaplen = nvim_get_maxmaplen();
    let mut tb_off = nvim_get_typebuf_off();
    let tb_len = nvim_get_typebuf_len() - len;
    let tb_buflen = nvim_get_typebuf_buflen();

    nvim_set_typebuf_len(tb_len);

    // Easy case: just increase tb_off
    if offset == 0 && tb_buflen - (tb_off + len) >= 3 * maxmaplen + 3 {
        nvim_set_typebuf_off(tb_off + len);
    } else {
        // Need to move characters
        let tb_buf = nvim_get_typebuf_buf();
        let tb_noremap = nvim_get_typebuf_noremap();
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
            nvim_set_typebuf_off(tb_off);
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
    let mut tb_maplen = nvim_get_typebuf_maplen();
    if tb_maplen > offset {
        if tb_maplen < offset + len {
            tb_maplen = offset;
        } else {
            tb_maplen -= len;
        }
        nvim_set_typebuf_maplen(tb_maplen);
    }

    // Adjust silent
    let mut tb_silent = nvim_get_typebuf_silent();
    if tb_silent > offset {
        if tb_silent < offset + len {
            tb_silent = offset;
        } else {
            tb_silent -= len;
        }
        nvim_set_typebuf_silent(tb_silent);
    }

    // Adjust no_abbr_cnt
    let mut tb_no_abbr_cnt = nvim_get_typebuf_no_abbr_cnt();
    if tb_no_abbr_cnt > offset {
        if tb_no_abbr_cnt < offset + len {
            tb_no_abbr_cnt = offset;
        } else {
            tb_no_abbr_cnt -= len;
        }
        nvim_set_typebuf_no_abbr_cnt(tb_no_abbr_cnt);
    }

    // Reset typebuf_was_filled flag
    nvim_set_typebuf_was_filled(0);
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
    let tb_maplen = nvim_get_typebuf_maplen();
    let tb_off = nvim_get_typebuf_off();
    let tb_len = nvim_get_typebuf_len();

    nvim_set_typebuf_off(tb_off + tb_maplen);
    nvim_set_typebuf_len(tb_len - tb_maplen);
    nvim_set_typebuf_maplen(0);
    nvim_set_typebuf_silent(0);
    nvim_set_cmd_silent(0);
    nvim_set_typebuf_no_abbr_cnt(0);
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
    let maxmaplen = nvim_get_maxmaplen();

    nvim_set_typebuf_off(maxmaplen);
    nvim_set_typebuf_len(0);
    nvim_set_typebuf_maplen(0);
    nvim_set_typebuf_silent(0);
    nvim_set_typebuf_no_abbr_cnt(0);
    nvim_set_typebuf_was_filled(0);
    increment_typebuf_change_cnt();
}

/// Get a byte from the typeahead buffer at the given offset.
///
/// # Safety
/// Calls C accessor functions and performs pointer operations.
#[no_mangle]
pub unsafe extern "C" fn rs_typebuf_get_byte(offset: c_int) -> c_int {
    let tb_off = nvim_get_typebuf_off();
    let tb_len = nvim_get_typebuf_len();

    if offset < 0 || offset >= tb_len {
        return -1;
    }

    let tb_buf = nvim_get_typebuf_buf();
    c_int::from(*tb_buf.offset((tb_off + offset) as isize))
}

/// Get the remap flag at the given offset in the typeahead buffer.
///
/// # Safety
/// Calls C accessor functions and performs pointer operations.
#[no_mangle]
pub unsafe extern "C" fn rs_typebuf_get_noremap(offset: c_int) -> c_int {
    let tb_off = nvim_get_typebuf_off();
    let tb_len = nvim_get_typebuf_len();

    if offset < 0 || offset >= tb_len {
        return RemapFlag::Yes as c_int;
    }

    let tb_noremap = nvim_get_typebuf_noremap();
    c_int::from(*tb_noremap.offset((tb_off + offset) as isize))
}

/// Get the current typeahead buffer length.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_typebuf_len() -> c_int {
    nvim_get_typebuf_len()
}

/// Get the current typeahead buffer offset.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_typebuf_off() -> c_int {
    nvim_get_typebuf_off()
}

/// Check if the typeahead buffer is empty.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_typebuf_is_empty() -> c_int {
    c_int::from(nvim_get_typebuf_len() == 0)
}

/// Check if the first character in the typeahead should use silent mode.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_typebuf_is_silent() -> c_int {
    c_int::from(nvim_get_typebuf_silent() > 0)
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
    nvim_init_typebuf();

    // Increment change counter
    increment_typebuf_change_cnt();

    // Notify state is no longer safe
    nvim_state_no_longer_safe();

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

    let maxmaplen = nvim_get_maxmaplen();
    let tb_off = nvim_get_typebuf_off();
    let tb_len = nvim_get_typebuf_len();
    let tb_buflen = nvim_get_typebuf_buflen();
    let tb_buf = nvim_get_typebuf_buf();

    // Easy case: there is room in front of typebuf.tb_buf[typebuf.tb_off]
    if offset == 0 && addlen <= tb_off {
        let new_off = tb_off - addlen;
        nvim_set_typebuf_off(new_off);
        std::ptr::copy_nonoverlapping(str, tb_buf.offset(new_off as isize), addlen as usize);
    } else if tb_len == 0 && tb_buflen >= addlen + 3 * (maxmaplen + 4) {
        // Buffer is empty and string fits in the existing buffer.
        // Leave some space before and after, if possible.
        let new_off = ((tb_buflen - addlen - 3 * (maxmaplen + 4)) / 2).max(0);
        nvim_set_typebuf_off(new_off);
        std::ptr::copy_nonoverlapping(str, tb_buf.offset(new_off as isize), addlen as usize);
    } else {
        // Need to allocate more room for the buffer
        let newoff = maxmaplen + 4;
        let extra = addlen + newoff + 4 * (maxmaplen + 4);

        if tb_len > i32::MAX - extra {
            // String is too long
            return FAIL;
        }

        // Allocate new buffers (done in C via xrealloc)
        let new_buflen = tb_len + extra;
        if nvim_grow_typebuf(new_buflen) == 0 {
            return FAIL;
        }

        // Get updated buffer pointers after reallocation
        let tb_buf = nvim_get_typebuf_buf();
        let tb_noremap = nvim_get_typebuf_noremap();
        let old_off = nvim_get_typebuf_off();

        // Copy existing content to new position, making room for insertion
        // First: bytes before offset
        if offset > 0 {
            std::ptr::copy(
                tb_buf.offset(old_off as isize),
                tb_buf.offset(newoff as isize),
                offset as usize,
            );
            std::ptr::copy(
                tb_noremap.offset(old_off as isize),
                tb_noremap.offset(newoff as isize),
                offset as usize,
            );
        }

        // Copy new string at offset position
        std::ptr::copy_nonoverlapping(
            str,
            tb_buf.offset((newoff + offset) as isize),
            addlen as usize,
        );

        // Copy bytes after offset (including NUL)
        let bytes_after = (tb_len - offset + 1) as usize;
        std::ptr::copy(
            tb_buf.offset((old_off + offset) as isize),
            tb_buf.offset((newoff + offset + addlen) as isize),
            bytes_after,
        );
        let noremap_after = (tb_len - offset) as usize;
        std::ptr::copy(
            tb_noremap.offset((old_off + offset) as isize),
            tb_noremap.offset((newoff + offset + addlen) as isize),
            noremap_after,
        );

        nvim_set_typebuf_off(newoff);
    }

    // Update length
    let new_len = nvim_get_typebuf_len() + addlen;
    nvim_set_typebuf_len(new_len);

    // Set the NUL terminator
    let tb_buf = nvim_get_typebuf_buf();
    let tb_off = nvim_get_typebuf_off();
    *tb_buf.offset((tb_off + new_len) as isize) = 0;

    // Determine remap value and count for noremap flags
    let (val, nrm) = match noremap {
        _ if noremap == REMAP_SCRIPT => (RM_SCRIPT, addlen),
        _ if noremap == REMAP_SKIP => (RM_ABBR, 1),
        _ if noremap < 0 => (RM_NONE, addlen),
        _ if noremap > 0 => (RM_NONE, noremap),
        _ => (RM_YES, 0),
    };

    // Set noremap flags for the inserted characters
    let tb_noremap = nvim_get_typebuf_noremap();
    for i in 0..addlen {
        let idx = tb_off + i + offset;
        *tb_noremap.offset(idx as isize) = if i < nrm { val } else { RM_YES };
    }

    // Adjust maplen if characters were not typed (or offset is within mapped region)
    let mut tb_maplen = nvim_get_typebuf_maplen();
    if nottyped != 0 || tb_maplen > offset {
        tb_maplen += addlen;
        nvim_set_typebuf_maplen(tb_maplen);
    }

    // Adjust silent if needed
    let mut tb_silent = nvim_get_typebuf_silent();
    if silent != 0 || tb_silent > offset {
        tb_silent += addlen;
        nvim_set_typebuf_silent(tb_silent);
    }

    // Adjust no_abbr_cnt if needed (when inserting at the beginning)
    if offset == 0 {
        let mut tb_no_abbr_cnt = nvim_get_typebuf_no_abbr_cnt();
        if tb_no_abbr_cnt != 0 {
            tb_no_abbr_cnt += addlen;
            nvim_set_typebuf_no_abbr_cnt(tb_no_abbr_cnt);
        }
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
    fn nvim_init_typebuf();
    fn nvim_state_no_longer_safe();
    fn nvim_grow_typebuf(new_buflen: c_int) -> c_int;
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
