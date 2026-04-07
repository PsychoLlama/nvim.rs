//! Sign querying and listing
//!
//! This module handles querying placed signs and listing sign definitions.

use std::ffi::{c_char, c_int};

use crate::{
    text::describe_sign_text_impl, DecorSignHighlightHandle, LinenrT, MTKeyHandle, SignBufHandle,
    NS_ALL, NS_GLOBAL, NS_INVALID,
};

// =============================================================================
// C Accessor Extern Declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Sign map operations
    fn nvim_sign_map_has(name: *const c_char) -> c_int;
    fn nvim_sign_map_get(name: *const c_char) -> crate::SignHandle;

    // Namespace operations
    fn nvim_namespace_lookup(name: *const c_char) -> c_int;
    fn describe_ns(ns: c_int, unknown: *const c_char) -> *const c_char;

    // Buffer operations
    fn nvim_buf_meta_total_sign_hl(buf: SignBufHandle) -> u64;
    fn nvim_buf_meta_total_sign_text(buf: SignBufHandle) -> u64;

    // MTKey accessors
    fn nvim_mtkey_get_row(key: MTKeyHandle) -> c_int;
    fn nvim_mtkey_get_ns(key: MTKeyHandle) -> u32;
    fn nvim_mtkey_get_id(key: MTKeyHandle) -> u32;

    // DecorSignHighlight accessors
    fn nvim_decor_sh_get_sign_name(sh: DecorSignHighlightHandle) -> *const c_char;

    // Error reporting
    fn semsg(fmt: *const c_char, ...);

    // Display/listing composite accessors
    fn nvim_sign_list_placed_impl(rbuf: SignBufHandle, group: *const c_char);

    // Message functions for sign_list_defined
    fn nvim_smsg0(msg: *const c_char);
    fn nvim_msg_puts(s: *const c_char);
    fn nvim_msg_outtrans(s: *const c_char);
    fn nvim_msg_puts_priority(prio: c_int);
    fn get_highlight_name_ext(
        xp: *mut std::ffi::c_void,
        idx: c_int,
        skip_cleared: bool,
    ) -> *const c_char;
}

const E155_FMT: &[u8] = b"E155: Unknown sign: %s\0";

// =============================================================================
// Sign Name Lookup
// =============================================================================

/// Get the display name for a placed sign.
///
/// Returns:
/// - The sign name if valid and defined
/// - "[Deleted]" if the sign was deleted
/// - Empty string if no name
///
/// # Safety
///
/// `sh` must be a valid DecorSignHighlight handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_get_display_name(sh: DecorSignHighlightHandle) -> *const c_char {
    static EMPTY: &[u8] = b"\0";
    static DELETED: &[u8] = b"[Deleted]\0";

    if sh.is_null() {
        return EMPTY.as_ptr().cast::<c_char>();
    }

    let name = nvim_decor_sh_get_sign_name(sh);
    if name.is_null() {
        return EMPTY.as_ptr().cast::<c_char>();
    }

    // Check if the sign is still defined
    if nvim_sign_map_has(name) != 0 {
        name
    } else {
        DELETED.as_ptr().cast::<c_char>()
    }
}

// =============================================================================
// Namespace Queries
// =============================================================================

/// Get namespace description for display.
///
/// Returns empty string for global namespace (ns = 0).
///
/// # Safety
///
/// The returned pointer is valid only as long as the namespace exists.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_describe_ns(ns: c_int) -> *const c_char {
    static EMPTY: &[u8] = b"\0";
    describe_ns(ns, EMPTY.as_ptr().cast::<c_char>())
}

/// Check if a namespace matches a filter.
///
/// Returns true if:
/// - filter is NS_ALL (matches everything)
/// - filter equals the mark's namespace
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub extern "C" fn rs_sign_ns_matches(mark_ns: u32, filter_ns: i64) -> bool {
    filter_ns == NS_ALL || (filter_ns >= 0 && mark_ns == filter_ns as u32)
}

// =============================================================================
// Query Filters
// =============================================================================

/// Filter parameters for sign queries
#[repr(C)]
pub struct SignQueryFilter {
    /// Namespace filter (-1 = invalid, 0 = global, UINT32_MAX = all)
    pub ns: i64,
    /// Sign ID filter (0 = all)
    pub sign_id: c_int,
    /// Line number filter (0 = all)
    pub lnum: LinenrT,
}

impl SignQueryFilter {
    /// Check if this filter accepts a sign at the given location
    #[allow(clippy::cast_possible_wrap)]
    pub fn matches(&self, mark_ns: u32, mark_id: u32, mark_row: c_int) -> bool {
        // Namespace check
        if !rs_sign_ns_matches(mark_ns, self.ns) {
            return false;
        }

        // ID check
        if self.sign_id != 0 && self.sign_id != mark_id as c_int {
            return false;
        }

        // Line check (mark_row is 0-based, lnum is 1-based)
        if self.lnum != 0 && self.lnum != mark_row + 1 {
            return false;
        }

        true
    }
}

/// Create a query filter for all signs in a namespace.
#[no_mangle]
pub extern "C" fn rs_sign_query_filter_ns(ns: i64) -> SignQueryFilter {
    SignQueryFilter {
        ns,
        sign_id: 0,
        lnum: 0,
    }
}

/// Create a query filter for a specific sign.
#[no_mangle]
pub extern "C" fn rs_sign_query_filter_id(ns: i64, sign_id: c_int) -> SignQueryFilter {
    SignQueryFilter {
        ns,
        sign_id,
        lnum: 0,
    }
}

/// Create a query filter for signs at a specific line.
#[no_mangle]
pub extern "C" fn rs_sign_query_filter_line(ns: i64, lnum: LinenrT) -> SignQueryFilter {
    SignQueryFilter {
        ns,
        sign_id: 0,
        lnum,
    }
}

/// Create a query filter with all criteria.
#[no_mangle]
pub extern "C" fn rs_sign_query_filter_full(
    ns: i64,
    sign_id: c_int,
    lnum: LinenrT,
) -> SignQueryFilter {
    SignQueryFilter { ns, sign_id, lnum }
}

// =============================================================================
// Buffer Sign Queries
// =============================================================================

/// Check if a buffer has any signs.
///
/// # Safety
///
/// `buf` must be a valid buffer handle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_buffer_has_signs(buf: SignBufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    let hl = nvim_buf_meta_total_sign_hl(buf);
    let text = nvim_buf_meta_total_sign_text(buf);
    (hl + text) > 0
}

/// buf_has_signs — public C API replacement.
///
/// This replaces the `buf_has_signs(const buf_T *buf)` function in sign.c.
///
/// # Safety
/// `buf` must be a valid buffer handle or null.
#[unsafe(export_name = "buf_has_signs")]
pub unsafe extern "C" fn rs_buf_has_signs_export(buf: SignBufHandle) -> bool {
    rs_sign_buffer_has_signs(buf)
}

/// Get the namespace filter for a group name.
///
/// # Safety
///
/// `group` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_get_ns_filter(group: *const c_char) -> i64 {
    if group.is_null() {
        return NS_GLOBAL;
    }

    let first_byte = *group.cast::<u8>();
    if first_byte == b'*' {
        return NS_ALL;
    }

    let ns = nvim_namespace_lookup(group);
    if ns != 0 {
        i64::from(ns)
    } else {
        NS_INVALID
    }
}

// =============================================================================
// Sign Placement Info Extraction
// =============================================================================

/// Extract sign placement info from an MTKey.
///
/// Returns a struct containing row, namespace, and id.
#[repr(C)]
pub struct SignPlacementInfo {
    /// Row (0-based)
    pub row: c_int,
    /// Namespace
    pub ns: u32,
    /// Sign ID
    pub id: u32,
}

/// Extract placement info from an MTKey handle.
///
/// # Safety
///
/// `key` must be a valid MTKey handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_extract_placement_info(key: MTKeyHandle) -> SignPlacementInfo {
    if key.is_null() {
        return SignPlacementInfo {
            row: -1,
            ns: 0,
            id: 0,
        };
    }

    SignPlacementInfo {
        row: nvim_mtkey_get_row(key),
        ns: nvim_mtkey_get_ns(key),
        id: nvim_mtkey_get_id(key),
    }
}

/// Get the line number (1-based) from an MTKey.
///
/// # Safety
///
/// `key` must be a valid MTKey handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_get_lnum_from_key(key: MTKeyHandle) -> LinenrT {
    if key.is_null() {
        return 0;
    }
    nvim_mtkey_get_row(key) + 1
}

// =============================================================================
// Sign Display/Listing
// =============================================================================

/// List placed signs for a buffer or all buffers.
///
/// If `rbuf` is null, lists signs for all buffers.
/// Delegates to C composite accessor for marktree iteration and message output.
///
/// # Safety
///
/// `rbuf` must be a valid buffer handle or null.
/// `group` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_list_placed(rbuf: SignBufHandle, group: *const c_char) {
    nvim_sign_list_placed_impl(rbuf, group);
}

/// List a single sign definition.
///
/// Outputs sign attributes via message functions.
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[no_mangle]
#[allow(clippy::manual_c_str_literals)]
pub unsafe extern "C" fn rs_sign_list_defined(sp: crate::SignHandle) {
    if sp.is_null() {
        return;
    }
    let s = &*sp;
    // "sign <name>"
    // Build "sign %s" message
    let mut msg_buf = [0u8; 512];
    let name_bytes = if s.sn_name.is_null() {
        b"(unnamed)\0".as_slice()
    } else {
        std::slice::from_raw_parts(s.sn_name.cast::<u8>(), libc_strlen(s.sn_name) + 1)
    };
    let prefix = b"sign \0";
    let plen = prefix.len() - 1;
    let nlen = name_bytes.len() - 1; // exclude null terminator
    let total = plen + nlen;
    if total < msg_buf.len() {
        msg_buf[..plen].copy_from_slice(&prefix[..plen]);
        msg_buf[plen..total].copy_from_slice(&name_bytes[..nlen]);
        msg_buf[total] = 0;
    }
    nvim_smsg0(msg_buf.as_ptr().cast());

    // icon
    if !s.sn_icon.is_null() {
        nvim_msg_puts(b" icon=\0".as_ptr().cast());
        nvim_msg_outtrans(s.sn_icon);
        nvim_msg_puts(b" (not supported)\0".as_ptr().cast());
    }
    // text
    if s.sn_text[0] != 0 {
        nvim_msg_puts(b" text=\0".as_ptr().cast());
        let mut buf = [0u8; crate::SIGN_WIDTH * crate::text::MAX_SCHAR_SIZE];
        describe_sign_text_impl(&mut buf, &s.sn_text);
        nvim_msg_outtrans(buf.as_ptr().cast());
    }
    // priority
    if s.sn_priority > 0 {
        nvim_msg_puts_priority(s.sn_priority);
    }
    // highlights
    let hl_prefixes: [&[u8]; 4] = [b" linehl=\0", b" texthl=\0", b" culhl=\0", b" numhl=\0"];
    let hl_ids = [s.sn_line_hl, s.sn_text_hl, s.sn_cul_hl, s.sn_num_hl];
    for (prefix, &hl_id) in hl_prefixes.iter().zip(hl_ids.iter()) {
        if hl_id > 0 {
            nvim_msg_puts(prefix.as_ptr().cast());
            let p = get_highlight_name_ext(std::ptr::null_mut(), hl_id - 1, false);
            let name_ptr = if p.is_null() {
                b"NONE\0".as_ptr().cast()
            } else {
                p
            };
            nvim_msg_puts(name_ptr);
        }
    }
}

unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

/// List the sign matching a given name.
///
/// Looks up the sign in the sign map and displays its definition.
/// Emits E155 error for unknown sign names.
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_list_by_name(name: *const c_char) {
    if name.is_null() {
        return;
    }
    let sp = nvim_sign_map_get(name);
    if sp.is_null() {
        semsg(E155_FMT.as_ptr().cast(), name);
    } else {
        rs_sign_list_defined(sp);
    }
}

/// Get the display name for a placed sign — static C wrapper replacement.
///
/// This replaces `sign_get_name(DecorSignHighlight *sh)` in sign.c.
///
/// # Safety
/// `sh` must be a valid DecorSignHighlight handle.
#[unsafe(export_name = "sign_get_name")]
pub unsafe extern "C" fn rs_sign_get_name_wrapper(sh: DecorSignHighlightHandle) -> *const c_char {
    rs_sign_get_display_name(sh)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_ns_matches() {
        // NS_ALL matches everything
        assert!(rs_sign_ns_matches(0, NS_ALL));
        assert!(rs_sign_ns_matches(100, NS_ALL));

        // Exact match
        assert!(rs_sign_ns_matches(0, 0));
        assert!(rs_sign_ns_matches(100, 100));

        // No match
        assert!(!rs_sign_ns_matches(0, 100));
        assert!(!rs_sign_ns_matches(100, 0));

        // Invalid namespace
        assert!(!rs_sign_ns_matches(100, NS_INVALID));
    }

    #[test]
    fn test_sign_query_filter() {
        let filter = rs_sign_query_filter_ns(NS_ALL);
        assert!(filter.matches(0, 1, 0));
        assert!(filter.matches(100, 5, 10));

        let filter = rs_sign_query_filter_id(0, 5);
        assert!(filter.matches(0, 5, 0));
        assert!(!filter.matches(0, 6, 0));
        assert!(!filter.matches(1, 5, 0)); // Wrong namespace

        let filter = rs_sign_query_filter_line(NS_ALL, 10);
        assert!(filter.matches(0, 1, 9)); // row 9 = lnum 10
        assert!(!filter.matches(0, 1, 10)); // row 10 = lnum 11
    }

    // Note: test_sign_placement_info_null is skipped because it would require
    // linking against C symbols that aren't available in pure Rust test mode.
    // The function is tested via integration tests when linked with C code.
}
