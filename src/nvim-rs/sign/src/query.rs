//! Sign querying and listing
//!
//! This module handles querying placed signs and listing sign definitions.

use std::ffi::{c_char, c_int};

use nvim_decoration::types::{DecorInline, DecorSignHighlight, MTKey, MTPair};

use crate::{
    bref, text::describe_sign_text_impl, DecorSignHighlightHandle, LinenrT, MTKeyHandle,
    SignBufHandle, NS_ALL, NS_GLOBAL, NS_INVALID,
};

// Flag constants (from marktree)
const MT_FLAG_DECOR_EXT: u16 = 1 << 7;
const MT_FLAG_DECOR_SIGNTEXT: u16 = 1 << 9;
const MT_FLAG_DECOR_SIGNHL: u16 = 1 << 10;

/// Convert an MTKey to a DecorInline for use with decor_find_sign.
#[allow(dead_code)]
#[inline]
fn mtkey_to_decor_inline(key: &MTKey) -> DecorInline {
    DecorInline {
        ext: (key.flags & MT_FLAG_DECOR_EXT) != 0,
        _pad: [0; 7],
        data: key.decor_data,
    }
}

/// Check if an MTKey is a sign decoration.
#[allow(dead_code)]
#[inline]
fn mtkey_is_decor_sign(key: &MTKey) -> bool {
    (key.flags & (MT_FLAG_DECOR_SIGNTEXT | MT_FLAG_DECOR_SIGNHL)) != 0
}

/// Check if an MTKey is an end mark.
#[allow(dead_code)]
#[inline]
fn mtkey_is_end(key: &MTKey) -> bool {
    const MT_FLAG_END: u16 = 1 << 1;
    (key.flags & MT_FLAG_END) != 0
}

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

    // Error reporting
    fn semsg(fmt: *const c_char, ...);

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

    // Phase 1: decor_find_sign — exported from nvim-decoration crate
    fn decor_find_sign(decor: DecorInline) -> *mut DecorSignHighlight;

    // Phase 1: dict/list construction
    fn tv_dict_alloc() -> *mut std::ffi::c_void;
    fn tv_dict_add_str(
        d: *mut std::ffi::c_void,
        key: *const c_char,
        key_len: usize,
        val: *const c_char,
    );
    fn tv_dict_add_nr(d: *mut std::ffi::c_void, key: *const c_char, key_len: usize, nr: i64);
    fn tv_list_alloc(len: i64) -> *mut std::ffi::c_void;
    fn tv_list_append_dict(l: *mut std::ffi::c_void, d: *mut std::ffi::c_void);
    fn tv_dict_add_list(
        d: *mut std::ffi::c_void,
        key: *const c_char,
        key_len: usize,
        list: *mut std::ffi::c_void,
    ) -> c_int;

    // Phase 1: message functions for list_placed
    fn msg_puts_title(s: *const c_char);
    fn msg_puts_hl(s: *const c_char, hl_id: c_int, hist: bool);
    fn msg_puts(s: *const c_char);
    fn msg_putchar(c: c_int);
    fn vim_snprintf(dst: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;

    // Phase 1: global state
    static mut got_int: bool;
    fn nvim_get_firstbuf() -> SignBufHandle;

    // Phase 1: marktree iteration (heap-allocated iterator)
    fn nvim_marktree_itr_alloc() -> *mut std::ffi::c_void;
    #[link_name = "xfree"]
    fn nvim_marktree_itr_free(itr: *mut std::ffi::c_void);
    fn rs_marktree_itr_get(
        b: *mut std::ffi::c_void,
        row: c_int,
        col: c_int,
        itr: *mut std::ffi::c_void,
    );
    fn rs_marktree_itr_current(itr: *mut std::ffi::c_void) -> MTKey;
    fn rs_marktree_itr_next(b: *mut std::ffi::c_void, itr: *mut std::ffi::c_void) -> bool;
    fn rs_marktree_itr_get_overlap(
        b: *mut std::ffi::c_void,
        row: c_int,
        col: c_int,
        itr: *mut std::ffi::c_void,
    ) -> bool;
    fn rs_marktree_itr_step_overlap(
        b: *mut std::ffi::c_void,
        itr: *mut std::ffi::c_void,
        pair: *mut MTPair,
    ) -> bool;
    fn extmark_del_id(buf: SignBufHandle, ns: u32, id: u32) -> bool;

    // Phase 1: buffer marktree handle
    fn nvim_buf_get_marktree(buf: SignBufHandle) -> *mut std::ffi::c_void;

    // Phase 1: iterator validity check
    fn nvim_mtitr_has_x(itr: *const std::ffi::c_void) -> bool;
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

    let name = (*sh.0.cast::<nvim_decoration::types::DecorSignHighlight>()).sign_name;
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

// =============================================================================
// Phase 1: Sort comparator for signs (replaces C sign_row_cmp / qsort callback)
// =============================================================================

static EMPTY_CSTR: &[u8] = b"\0";

/// Compare two MTKey signs: ascending row, then descending priority/id/add_id.
#[allow(clippy::cast_possible_wrap)]
unsafe fn cmp_signs(a: &MTKey, b: &MTKey) -> std::cmp::Ordering {
    if a.pos.row != b.pos.row {
        return a.pos.row.cmp(&b.pos.row);
    }
    let sha = decor_find_sign(mtkey_to_decor_inline(a));
    let shb = decor_find_sign(mtkey_to_decor_inline(b));
    let (prio_a, add_a) = if sha.is_null() {
        (0u16, 0i32)
    } else {
        ((*sha).priority, (*sha).sign_add_id)
    };
    let (prio_b, add_b) = if shb.is_null() {
        (0u16, 0i32)
    } else {
        ((*shb).priority, (*shb).sign_add_id)
    };
    if prio_a != prio_b {
        return prio_b.cmp(&prio_a);
    }
    if a.id != b.id {
        return b.id.cmp(&a.id);
    }
    add_b.cmp(&add_a)
}

// =============================================================================
// Phase 1: Rust implementations replacing C _impl functions
// =============================================================================

/// Build a dict_T containing placement info for a single sign mark.
#[allow(clippy::cast_possible_wrap)]
unsafe fn build_placed_info_dict(mark: *const MTKey) -> *mut std::ffi::c_void {
    let d = tv_dict_alloc();
    let sh = decor_find_sign(mtkey_to_decor_inline(&*mark));
    let name = rs_sign_get_display_name(crate::DecorSignHighlightHandle(sh.cast()));
    tv_dict_add_str(d, c"name".as_ptr(), 4, name);
    tv_dict_add_nr(d, c"id".as_ptr(), 2, i64::from((*mark).id));
    let group_name = describe_ns((*mark).ns as c_int, EMPTY_CSTR.as_ptr().cast());
    tv_dict_add_str(d, c"group".as_ptr(), 5, group_name);
    tv_dict_add_nr(d, c"lnum".as_ptr(), 4, i64::from((*mark).pos.row + 1));
    let priority = if sh.is_null() {
        0i64
    } else {
        i64::from((*sh).priority)
    };
    tv_dict_add_nr(d, c"priority".as_ptr(), 8, priority);
    d
}

/// Replace C `nvim_sign_get_placed_info_dict_impl`.
///
/// # Safety
/// `mark_ptr` must be a valid pointer to an MTKey.
#[unsafe(export_name = "nvim_sign_get_placed_info_dict_impl")]
pub unsafe extern "C" fn rs_nvim_sign_get_placed_info_dict_impl(
    mark_ptr: *mut std::ffi::c_void,
) -> *mut std::ffi::c_void {
    if mark_ptr.is_null() {
        return std::ptr::null_mut();
    }
    build_placed_info_dict(mark_ptr.cast::<MTKey>())
}

/// Replace C `nvim_get_buffer_signs_impl`.
///
/// Returns a list_T containing all placed signs in the buffer.
///
/// # Safety
/// `buf` must be a valid buffer handle.
#[unsafe(export_name = "nvim_get_buffer_signs_impl")]
pub unsafe extern "C" fn rs_nvim_get_buffer_signs_impl(
    buf: SignBufHandle,
) -> *mut std::ffi::c_void {
    const K_LIST_LEN_MAY_KNOW: i64 = -3;
    let l = tv_list_alloc(K_LIST_LEN_MAY_KNOW);
    let b = nvim_buf_get_marktree(buf);
    let itr = nvim_marktree_itr_alloc();
    rs_marktree_itr_get(b, 0, 0, itr);
    while nvim_mtitr_has_x(itr) {
        let mark = rs_marktree_itr_current(itr);
        if !mtkey_is_end(&mark) && mtkey_is_decor_sign(&mark) {
            let d = build_placed_info_dict(&raw const mark);
            tv_list_append_dict(l, d);
        }
        rs_marktree_itr_next(b, itr);
    }
    nvim_marktree_itr_free(itr);
    l
}

/// Replace C `nvim_sign_get_placed_in_buf_impl`.
///
/// Appends a dict to `retlist` with keys "bufnr" and "signs".
/// The "signs" list contains one dict per matching placed sign.
///
/// # Safety
/// `buf` must be a valid buffer handle.
/// `group` must be null or a valid null-terminated C string.
/// `retlist` must be a valid list_T pointer.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
#[unsafe(export_name = "nvim_sign_get_placed_in_buf_impl")]
pub unsafe extern "C" fn rs_nvim_sign_get_placed_in_buf_impl(
    buf: SignBufHandle,
    lnum: LinenrT,
    sign_id: c_int,
    group: *const c_char,
    retlist: *mut std::ffi::c_void,
) {
    const K_LIST_LEN_MAY_KNOW: i64 = -3;

    // Build the outer dict: { bufnr: N, signs: [...] }
    let d = tv_dict_alloc();
    tv_list_append_dict(retlist, d);
    let fnum = bref(buf).handle;
    tv_dict_add_nr(d, c"bufnr".as_ptr(), 5, i64::from(fnum));
    let l = tv_list_alloc(K_LIST_LEN_MAY_KNOW);
    tv_dict_add_list(d, c"signs".as_ptr(), 5, l);

    let ns = rs_sign_get_ns_filter(group);
    if !rs_sign_buffer_has_signs(buf) || ns < 0 {
        return;
    }

    // Iterate marktree from (lnum-1) or 0
    let b = nvim_buf_get_marktree(buf);
    let itr = nvim_marktree_itr_alloc();
    let start_row = if lnum > 0 { lnum - 1 } else { 0 };
    rs_marktree_itr_get(b, start_row, 0, itr);

    let mut signs: Vec<MTKey> = Vec::new();
    while nvim_mtitr_has_x(itr) {
        let mark = rs_marktree_itr_current(itr);
        // If filtering by lnum, stop once we pass that row
        if lnum > 0 && mark.pos.row >= lnum {
            break;
        }
        let mark_lnum = mark.pos.row + 1;
        let mark_id = mark.id as c_int;
        let ns_matches = ns == NS_ALL || ns as u32 == mark.ns;
        let id_lnum_matches =
            (sign_id == mark_id || sign_id == 0) && (lnum == mark_lnum || lnum == 0);
        if !mtkey_is_end(&mark) && ns_matches && id_lnum_matches && mtkey_is_decor_sign(&mark) {
            signs.push(mark);
        }
        rs_marktree_itr_next(b, itr);
    }
    nvim_marktree_itr_free(itr);

    if !signs.is_empty() {
        signs.sort_by(|a, bk| cmp_signs(a, bk));
        for mark in &signs {
            let dict = build_placed_info_dict(&raw const *mark);
            tv_list_append_dict(l, dict);
        }
    }
}

/// Replace C `nvim_sign_list_placed_impl`.
///
/// Lists all placed signs, optionally filtered by buffer and group.
///
/// # Safety
/// `rbuf` must be a valid buffer handle or null.
/// `group` must be null or a valid null-terminated C string.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
#[unsafe(export_name = "nvim_sign_list_placed_impl")]
pub unsafe extern "C" fn rs_nvim_sign_list_placed_impl(rbuf: SignBufHandle, group: *const c_char) {
    // HLF_D = 5 (directory highlight for sign listing headers)
    const HLF_D: c_int = 5;
    const MSG_BUF_LEN: usize = 480;

    let ns = rs_sign_get_ns_filter(group);
    msg_puts_title(c"\n--- Signs ---".as_ptr());
    msg_putchar(c_int::from(b'\n'));

    let mut buf = if rbuf.is_null() {
        nvim_get_firstbuf()
    } else {
        rbuf
    };

    while !buf.is_null() && !got_int {
        if rs_sign_buffer_has_signs(buf) {
            let fname = bref(buf).b_fname;
            let mut lbuf = [0u8; MSG_BUF_LEN];
            vim_snprintf(
                lbuf.as_mut_ptr().cast(),
                MSG_BUF_LEN,
                c"Signs for %s:".as_ptr(),
                fname,
            );
            msg_puts_hl(lbuf.as_ptr().cast(), HLF_D, false);
            msg_putchar(c_int::from(b'\n'));
        }
        if ns >= 0 {
            let b = nvim_buf_get_marktree(buf);
            let itr = nvim_marktree_itr_alloc();
            rs_marktree_itr_get(b, 0, 0, itr);
            let mut signs: Vec<MTKey> = Vec::new();
            while nvim_mtitr_has_x(itr) {
                let mark = rs_marktree_itr_current(itr);
                if !mtkey_is_end(&mark)
                    && mtkey_is_decor_sign(&mark)
                    && (ns == NS_ALL || mark.ns == ns as u32)
                {
                    signs.push(mark);
                }
                rs_marktree_itr_next(b, itr);
            }
            nvim_marktree_itr_free(itr);
            if !signs.is_empty() {
                signs.sort_by(|a, bk| cmp_signs(a, bk));
                for mark in &signs {
                    let mut namebuf = [0u8; MSG_BUF_LEN];
                    let mut groupbuf = [0u8; MSG_BUF_LEN];
                    let sh = decor_find_sign(mtkey_to_decor_inline(mark));
                    if !sh.is_null() && !(*sh).sign_name.is_null() {
                        let display_name =
                            rs_sign_get_display_name(crate::DecorSignHighlightHandle(sh.cast()));
                        vim_snprintf(
                            namebuf.as_mut_ptr().cast(),
                            MSG_BUF_LEN,
                            c"  name=%s".as_ptr(),
                            display_name,
                        );
                    }
                    if mark.ns != 0 {
                        let ns_name = describe_ns(mark.ns as c_int, EMPTY_CSTR.as_ptr().cast());
                        vim_snprintf(
                            groupbuf.as_mut_ptr().cast(),
                            MSG_BUF_LEN,
                            c"  group=%s".as_ptr(),
                            ns_name,
                        );
                    }
                    let priority = if sh.is_null() {
                        0i32
                    } else {
                        i32::from((*sh).priority)
                    };
                    let mut lbuf = [0u8; MSG_BUF_LEN];
                    vim_snprintf(
                        lbuf.as_mut_ptr().cast(),
                        MSG_BUF_LEN,
                        c"    line=%d  id=%u%s%s  priority=%d".as_ptr(),
                        mark.pos.row + 1,
                        mark.id,
                        groupbuf.as_ptr(),
                        namebuf.as_ptr(),
                        priority,
                    );
                    msg_puts(lbuf.as_ptr().cast());
                    msg_putchar(c_int::from(b'\n'));
                }
            }
        }
        if !rbuf.is_null() {
            return;
        }
        buf = SignBufHandle(bref(buf).b_next);
    }
}

/// List placed signs for a buffer or all buffers.
///
/// If `rbuf` is null, lists signs for all buffers.
///
/// # Safety
///
/// `rbuf` must be a valid buffer handle or null.
/// `group` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_list_placed(rbuf: SignBufHandle, group: *const c_char) {
    rs_nvim_sign_list_placed_impl(rbuf, group);
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
