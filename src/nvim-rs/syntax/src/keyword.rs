//! Keyword handling for syntax highlighting.
//!
//! This module handles:
//! - Keyword hash table management
//! - Keyword matching functions
//! - Case-sensitive/insensitive handling
//! - Keyword entry accessors

use std::ffi::{c_char, c_int, c_void};

use crate::synblock_struct::{synblock_mut, synblock_ref};
use crate::types::{
    IdListHandle, KeyEntryHandle, StateItemHandle, SynBlockHandle, WinHandle, HL_CONTAINED,
    MAXKEYWLEN,
};

// =============================================================================
// FFI declarations for keyword operations
// =============================================================================

extern "C" {
    // Keyword matching functions (nvim_syn_keyword_find replaced by rs_syn_keyword_find Rust impl)
    fn nvim_syn_keyword_foldcase(src: *mut c_char, srclen: c_int, dst: *mut c_char, dstlen: c_int);
    fn utfc_ptr2len(p: *mut c_char) -> c_int;
    fn nvim_syn_vim_iswordp_buf(p: *mut c_char) -> c_int;

    // Keyword table accessors
    fn nvim_syn_has_keywords() -> c_int;
    fn nvim_syn_has_keywords_ic() -> c_int;

    // Window-level keyword accessors
    fn nvim_win_get_keywtab_used(win: WinHandle) -> c_int;
    fn nvim_win_get_keywtab_ic_used(win: WinHandle) -> c_int;

    // current_next_list access

    // Phase 11: Rust keyword_find and hash_insert_keyword implementation helpers
    fn nvim_syn_get_syn_block() -> SynBlockHandle;
    fn nvim_ht_get_used(ht: *const c_void) -> usize;
    fn nvim_ht_find_ke(ht: *mut c_void, keyword: *mut c_char) -> KeyEntryHandle;
    fn nvim_curwin_get_keywtab(use_ic: c_int) -> *mut c_void;
    fn nvim_hash_hash(key: *const c_char) -> u64;
    fn nvim_hash_lookup(ht: *mut c_void, key: *const c_char, len: usize, hash: u64) -> *mut c_void;
    fn nvim_hashitem_is_empty(hi: *const c_void) -> c_int;
    fn nvim_hash_add_item(ht: *mut c_void, hi: *mut c_void, key: *mut c_char, hash: u64);
    fn nvim_hikey2ke(hi: *const c_void) -> KeyEntryHandle;
    fn nvim_ke2hikey(kp: KeyEntryHandle) -> *mut c_char;
    fn nvim_curwin_set_containedin();
    fn nvim_hashitem_set_key(hi: *mut c_void, key: *mut c_char);
}

// =============================================================================
// Keyword entry accessors
// =============================================================================

/// Get the next keyword entry in the hash collision chain.
#[must_use]
pub fn keyentry_next(ke: KeyEntryHandle) -> KeyEntryHandle {
    if ke.is_null() {
        return KeyEntryHandle::null();
    }
    KeyEntryHandle(unsafe { (*ke.as_ptr()).ke_next })
}

/// Get the syntax ID (highlight group ID) for a keyword entry.
#[must_use]
pub fn keyentry_syn_id(ke: KeyEntryHandle) -> i16 {
    if ke.is_null() {
        return 0;
    }
    unsafe { (*ke.as_ptr()).k_syn.id }
}

/// Get the include tag for a keyword entry.
#[must_use]
pub fn keyentry_inc_tag(ke: KeyEntryHandle) -> i32 {
    if ke.is_null() {
        return 0;
    }
    unsafe { (*ke.as_ptr()).k_syn.inc_tag }
}

/// Get the flags for a keyword entry.
#[must_use]
pub fn keyentry_flags(ke: KeyEntryHandle) -> i32 {
    if ke.is_null() {
        return 0;
    }
    unsafe { (*ke.as_ptr()).flags }
}

/// Get the conceal character for a keyword entry.
#[must_use]
pub fn keyentry_cchar(ke: KeyEntryHandle) -> i32 {
    if ke.is_null() {
        return 0;
    }
    unsafe { (*ke.as_ptr()).k_char }
}

/// Get the keyword string for a keyword entry.
/// Returns a raw pointer that should not be freed.
#[must_use]
pub fn keyentry_keyword_ptr(ke: KeyEntryHandle) -> *const c_char {
    if ke.is_null() {
        return std::ptr::null();
    }
    unsafe { crate::ffi_types::KeyEntry::keyword_ptr(ke.as_ptr()) }
}

/// Get the next_list for a keyword entry.
#[must_use]
pub fn keyentry_next_list(ke: KeyEntryHandle) -> IdListHandle {
    if ke.is_null() {
        return IdListHandle::null();
    }
    IdListHandle(unsafe { (*ke.as_ptr()).next_list })
}

/// Get the cont_in_list for a keyword entry.
#[must_use]
pub fn keyentry_cont_in_list(ke: KeyEntryHandle) -> IdListHandle {
    if ke.is_null() {
        return IdListHandle::null();
    }
    IdListHandle(unsafe { (*ke.as_ptr()).k_syn.cont_in_list })
}

/// Check if a keyword entry has a next_list.
#[must_use]
pub fn keyentry_has_next_list(ke: KeyEntryHandle) -> bool {
    if ke.is_null() {
        return false;
    }
    !unsafe { (*ke.as_ptr()).next_list }.is_null()
}

/// Check if a keyword entry has a cont_in_list.
#[must_use]
pub fn keyentry_has_cont_in_list(ke: KeyEntryHandle) -> bool {
    if ke.is_null() {
        return false;
    }
    !unsafe { (*ke.as_ptr()).k_syn.cont_in_list }.is_null()
}

/// Check if a keyword entry is contained (not top-level).
#[must_use]
pub fn keyentry_is_contained(ke: KeyEntryHandle) -> bool {
    if ke.is_null() {
        return false;
    }
    (unsafe { (*ke.as_ptr()).flags } & HL_CONTAINED) != 0
}

// =============================================================================
// Keyword iteration
// =============================================================================

/// Iterator over keyword entries in a hash collision chain.
pub struct KeywordIter {
    current: KeyEntryHandle,
}

impl KeywordIter {
    /// Create a new iterator starting at the given keyword entry.
    #[must_use]
    pub fn new(start: KeyEntryHandle) -> Self {
        Self { current: start }
    }
}

impl Iterator for KeywordIter {
    type Item = KeyEntryHandle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }
        let result = self.current;
        self.current = keyentry_next(self.current);
        Some(result)
    }
}

// =============================================================================
// Keyword matching
// =============================================================================

/// Find a keyword in the hash table.
///
/// Replaces C `nvim_syn_keyword_find`. Uses syn_block's appropriate hashtab
/// (case-sensitive or insensitive) to look up the keyword.
///
/// # Arguments
/// * `keyword` - The keyword to search for (C string pointer)
/// * `use_ic` - If true, use case-insensitive matching
///
/// # Safety
/// The keyword pointer must be valid and null-terminated.
#[must_use]
pub unsafe fn keyword_find(keyword: *mut c_char, use_ic: bool) -> KeyEntryHandle {
    rs_syn_keyword_find(keyword, if use_ic { 1 } else { 0 })
}

/// Rust implementation of keyword lookup in syn_block's hashtab.
///
/// Replaces C `nvim_syn_keyword_find`.
///
/// # Safety
/// The keyword pointer must be valid and null-terminated; calls C globals.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_keyword_find(
    keyword: *mut c_char,
    use_ic: c_int,
) -> KeyEntryHandle {
    let block = nvim_syn_get_syn_block();
    if block.is_null() {
        return KeyEntryHandle::null();
    }
    let b = synblock_mut(block);
    let ht: *mut c_void = if use_ic != 0 {
        &mut b.b_keywtab_ic as *mut _ as *mut c_void
    } else {
        &mut b.b_keywtab as *mut _ as *mut c_void
    };
    if nvim_ht_get_used(ht as *const c_void) == 0 {
        return KeyEntryHandle::null();
    }
    nvim_ht_find_ke(ht, keyword)
}

/// Rust implementation of keyword insertion into curwin's hashtab.
///
/// Replaces C `nvim_syn_hash_insert_keyword`.
/// Uses `nvim_ke_alloc_and_insert` for the offsetof-based allocation.
///
/// # Safety
/// All pointer arguments follow the same ownership semantics as C add_keyword.
/// `cont_in_list_copy` and `next_list_copy` ownership is transferred.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_hash_insert_keyword(
    name_ic: *const c_char,
    name_iclen: c_int,
    id: c_int,
    inc_tag: c_int,
    flags: c_int,
    conceal_char: c_int,
    cont_in_list_copy: *mut i16,
    next_list_copy: *mut i16,
    use_ic: c_int,
) {
    let ht = nvim_curwin_get_keywtab(use_ic);
    if ht.is_null() {
        return;
    }
    rs_ke_alloc_and_insert(
        ht,
        name_ic,
        name_iclen,
        id,
        inc_tag,
        flags,
        conceal_char,
        cont_in_list_copy,
        next_list_copy,
    );
}

/// Match a keyword considering the current syntax state.
/// This respects contained/containedin lists.
///
/// Ported from C `nvim_syn_match_keyword`.
///
/// # Arguments
/// * `keyword` - The keyword to match (C string pointer)
/// * `use_ic` - If true, use case-insensitive matching
/// * `cur_si` - Current state item (may be null)
///
/// # Safety
/// The keyword pointer must be valid and null-terminated.
#[must_use]
pub unsafe fn match_keyword(
    keyword: *mut c_char,
    use_ic: bool,
    cur_si: StateItemHandle,
) -> KeyEntryHandle {
    let kp = keyword_find(keyword, use_ic);
    if kp.is_null() {
        return KeyEntryHandle::null();
    }
    let has_next_list = !crate::statics::CURRENT_NEXT_LIST.is_null();
    let current_next_list = if has_next_list {
        IdListHandle(crate::statics::CURRENT_NEXT_LIST)
    } else {
        IdListHandle::null()
    };
    let mut entry = kp;
    while !entry.is_null() {
        let ke_ptr = entry.as_ptr();
        let kp_id = i32::from(unsafe { (*ke_ptr).k_syn.id });
        let kp_inc_tag = unsafe { (*ke_ptr).k_syn.inc_tag };
        let kp_cont_in = IdListHandle(unsafe { (*ke_ptr).k_syn.cont_in_list });
        let kp_flags = unsafe { (*ke_ptr).flags };
        let matched = if has_next_list {
            crate::containment::rs_syn_in_id_list(
                StateItemHandle(std::ptr::null_mut()),
                current_next_list,
                kp_id,
                kp_inc_tag,
                kp_cont_in,
                0,
            ) != 0
        } else if cur_si.is_null() {
            (kp_flags & HL_CONTAINED) == 0
        } else {
            let cont_list = crate::containment::stateitem_cont_list(cur_si);
            crate::containment::rs_syn_in_id_list(
                cur_si, cont_list, kp_id, kp_inc_tag, kp_cont_in, kp_flags,
            ) != 0
        };
        if matched {
            return entry;
        }
        entry = KeyEntryHandle(unsafe { (*entry.as_ptr()).ke_next });
    }
    KeyEntryHandle::null()
}

/// Check keyword at the given position in line and return its syntax ID.
///
/// Ported from C `nvim_syn_check_keyword_id`. Finds the first word starting at
/// `startcol`, checks both case-sensitive and case-insensitive hash tables, and
/// returns the matching syntax ID (or 0 if no match).
///
/// Output parameters are only written on a successful match.
///
/// # Safety
/// - `line` must be a valid pointer to the current syntax line.
/// - `endcolp`, `flagsp`, `next_listp`, `ccharp` must be valid writable pointers.
/// - `cur_si` may be null (checked internally).
#[must_use]
pub unsafe fn check_keyword_id(
    line: *mut c_char,
    startcol: c_int,
    endcolp: *mut c_int,
    flagsp: *mut c_int,
    next_listp: *mut IdListHandle,
    cur_si: StateItemHandle,
    ccharp: *mut c_int,
) -> c_int {
    let kwp = line.add(startcol as usize);

    // Scan forward to find end of keyword.  First character was already checked.
    let mut kwlen: c_int = 0;
    loop {
        kwlen += utfc_ptr2len(kwp.add(kwlen as usize));
        if nvim_syn_vim_iswordp_buf(kwp.add(kwlen as usize)) == 0 {
            break;
        }
    }

    if kwlen > MAXKEYWLEN {
        return 0;
    }

    // Copy keyword into a NUL-terminated stack buffer.
    let mut keyword = [0u8; (MAXKEYWLEN + 1) as usize];
    std::ptr::copy_nonoverlapping(kwp.cast::<u8>(), keyword.as_mut_ptr(), kwlen as usize);
    // NUL is already 0 from initialization.

    let kw_ptr = keyword.as_mut_ptr().cast::<c_char>();

    // Case-sensitive match.
    let mut kp = KeyEntryHandle::null();
    if nvim_syn_has_keywords() != 0 {
        kp = match_keyword(kw_ptr, false, cur_si);
    }

    // Case-insensitive match.
    if kp.is_null() && nvim_syn_has_keywords_ic() != 0 {
        nvim_syn_keyword_foldcase(kwp, kwlen, kw_ptr, MAXKEYWLEN + 1);
        kp = match_keyword(kw_ptr, true, cur_si);
    }

    if !kp.is_null() {
        *endcolp = startcol + kwlen;
        let kp_ptr = kp.as_ptr();
        *flagsp = unsafe { (*kp_ptr).flags };
        *next_listp = IdListHandle(unsafe { (*kp_ptr).next_list });
        *ccharp = unsafe { (*kp_ptr).k_char };
        return i32::from(unsafe { (*kp_ptr).k_syn.id });
    }

    0
}

/// Fold case for keyword comparison.
///
/// # Safety
/// Both src and dst must be valid pointers with sufficient space.
pub unsafe fn keyword_foldcase(src: *mut c_char, srclen: i32, dst: *mut c_char, dstlen: i32) {
    nvim_syn_keyword_foldcase(src, srclen, dst, dstlen);
}

// utfc_ptr2len wrapper deleted: callers use extern directly.

/// Get the maximum keyword length constant.
#[must_use]
pub fn max_keyword_len() -> i32 {
    MAXKEYWLEN
}

/// The maximum keyword length (constant).
pub const MAX_KEYWORD_LEN: i32 = MAXKEYWLEN;

// =============================================================================
// Keyword table queries
// =============================================================================

/// Check if the current synblock has any case-sensitive keywords.
#[must_use]
pub fn has_keywords() -> bool {
    unsafe { nvim_syn_has_keywords() != 0 }
}

/// Check if the current synblock has any case-insensitive keywords.
#[must_use]
pub fn has_keywords_ic() -> bool {
    unsafe { nvim_syn_has_keywords_ic() != 0 }
}

/// Check if a synblock has any case-sensitive keywords.
#[must_use]
pub fn synblock_has_keywords(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { synblock_ref(block).b_keywtab.ht_used > 0 }
}

/// Check if a synblock has any case-insensitive keywords.
#[must_use]
pub fn synblock_has_keywords_ic(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { synblock_ref(block).b_keywtab_ic.ht_used > 0 }
}

/// Get the count of case-sensitive keywords in a synblock.
#[must_use]
pub fn synblock_keyword_count(block: SynBlockHandle) -> usize {
    if block.is_null() {
        return 0;
    }
    unsafe { synblock_ref(block).b_keywtab.ht_used }
}

/// Get the count of case-insensitive keywords in a synblock.
#[must_use]
pub fn synblock_keyword_ic_count(block: SynBlockHandle) -> usize {
    if block.is_null() {
        return 0;
    }
    unsafe { synblock_ref(block).b_keywtab_ic.ht_used }
}

/// Get the case-sensitive keyword hash table for a synblock.
/// Returns a raw pointer to the hashtab_T.
#[must_use]
pub fn synblock_keywtab(block: SynBlockHandle) -> *mut c_void {
    if block.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { &mut synblock_mut(block).b_keywtab as *mut _ as *mut c_void }
}

/// Get the case-insensitive keyword hash table for a synblock.
/// Returns a raw pointer to the hashtab_T.
#[must_use]
pub fn synblock_keywtab_ic(block: SynBlockHandle) -> *mut c_void {
    if block.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { &mut synblock_mut(block).b_keywtab_ic as *mut _ as *mut c_void }
}

// =============================================================================
// Window-level keyword queries
// =============================================================================

/// Get the count of case-sensitive keywords used in a window's synblock.
#[must_use]
pub fn win_keyword_count(win: WinHandle) -> i32 {
    if win.is_null() {
        return 0;
    }
    unsafe { nvim_win_get_keywtab_used(win) }
}

/// Get the count of case-insensitive keywords used in a window's synblock.
#[must_use]
pub fn win_keyword_ic_count(win: WinHandle) -> i32 {
    if win.is_null() {
        return 0;
    }
    unsafe { nvim_win_get_keywtab_ic_used(win) }
}

// =============================================================================
// High-level keyword matching result
// =============================================================================

/// Result of a keyword match operation.
#[derive(Debug, Clone, Copy)]
pub struct KeywordMatch {
    /// The syntax ID (highlight group) of the matched keyword.
    pub syn_id: i16,
    /// Flags for the keyword.
    pub flags: i32,
    /// The conceal character (0 if none).
    pub cchar: i32,
    /// The next_list for this keyword (may be null).
    pub next_list: IdListHandle,
}

impl KeywordMatch {
    /// Create a keyword match result from a keyword entry.
    #[must_use]
    pub fn from_entry(ke: KeyEntryHandle) -> Option<Self> {
        if ke.is_null() {
            return None;
        }
        Some(Self {
            syn_id: keyentry_syn_id(ke),
            flags: keyentry_flags(ke),
            cchar: keyentry_cchar(ke),
            next_list: keyentry_next_list(ke),
        })
    }
}

// =============================================================================
// Phase 6: add_keyword + copy_id_list migration
// =============================================================================

extern "C" {
    fn nvim_syn_get_curwin_syn_ic() -> c_int;
}

extern "C" {
    fn xmalloc(size: usize) -> *mut std::ffi::c_void;
    fn xfree(ptr: *mut std::ffi::c_void);
}

/// Make a copy of a null-terminated int16_t ID list.
/// Returns null if the input is null.
/// Equivalent to C copy_id_list().
///
/// # Safety
/// `list` must be null or a valid null-terminated i16 array.
pub unsafe fn copy_id_list_impl(list: *const i16) -> *mut i16 {
    if list.is_null() {
        return std::ptr::null_mut();
    }
    let mut count: usize = 0;
    while *list.add(count) != 0 {
        count += 1;
    }
    let len = (count + 1) * std::mem::size_of::<i16>();
    let retval = xmalloc(len) as *mut i16;
    std::ptr::copy_nonoverlapping(list, retval, count + 1);
    retval
}

/// FFI export: copy a null-terminated int16_t ID list.
///
/// # Safety
/// `list` must be null or a valid null-terminated i16 array.
#[no_mangle]
pub unsafe extern "C" fn rs_copy_id_list(list: *const i16) -> *mut i16 {
    copy_id_list_impl(list)
}

/// Allocate a keyentry_T, fill all fields, and insert into the given hashtab.
/// Rust replacement for C nvim_ke_alloc_and_insert.
/// Ownership of cont_in_list_copy and next_list_copy is transferred.
/// Sets curwin->w_s->b_syn_containedin if cont_in_list_copy is non-NULL.
///
/// # Safety
/// All pointer arguments follow the same ownership semantics as C add_keyword.
#[allow(clippy::too_many_arguments)]
unsafe fn rs_ke_alloc_and_insert(
    ht: *mut c_void,
    name_ic: *const c_char,
    name_iclen: c_int,
    id: c_int,
    inc_tag: c_int,
    flags: c_int,
    conceal_char: c_int,
    cont_in_list_copy: *mut i16,
    next_list_copy: *mut i16,
) {
    use crate::ffi_types::KeyEntry;
    // Allocation: sizeof(KeyEntry) + name_iclen + 1  (matching C offsetof arithmetic)
    let alloc_size = std::mem::size_of::<KeyEntry>() + name_iclen as usize + 1;
    let kp_raw = xmalloc(alloc_size) as *mut KeyEntry;
    // Copy keyword string after the fixed fields
    let kw_dst = (kp_raw as *mut u8).add(std::mem::size_of::<KeyEntry>()) as *mut c_char;
    std::ptr::copy_nonoverlapping(name_ic, kw_dst, name_iclen as usize);
    *kw_dst.add(name_iclen as usize) = 0;
    // Fill fields
    (*kp_raw).k_syn.id = id as i16;
    (*kp_raw).k_syn.inc_tag = inc_tag;
    (*kp_raw).flags = flags;
    (*kp_raw).k_char = conceal_char;
    (*kp_raw).k_syn.cont_in_list = cont_in_list_copy;
    if !cont_in_list_copy.is_null() {
        nvim_curwin_set_containedin();
    }
    (*kp_raw).next_list = next_list_copy;
    // Hash and insert
    let kp = KeyEntryHandle(kp_raw);
    let kw_ptr = kw_dst; // points to keyword in the allocation
    let hash = nvim_hash_hash(kw_ptr);
    let hi = nvim_hash_lookup(ht, kw_ptr, name_iclen as usize, hash);
    if nvim_hashitem_is_empty(hi) != 0 {
        (*kp_raw).ke_next = std::ptr::null_mut();
        nvim_hash_add_item(ht, hi, kw_ptr, hash);
    } else {
        let existing = nvim_hikey2ke(hi);
        (*kp_raw).ke_next = if existing.is_null() {
            std::ptr::null_mut()
        } else {
            existing.as_ptr()
        };
        let new_hikey = nvim_ke2hikey(kp);
        // Update hi->hi_key to point to new entry's keyword (chain replacement)
        nvim_hashitem_set_key(hi, new_hikey);
    }
}

/// Add a keyword to the current window's syntax hash table.
/// Equivalent to C add_keyword().
///
/// # Safety
/// All pointer arguments follow the same ownership semantics as C add_keyword.
/// `cont_in_list` and `next_list` are NOT consumed (copies are made internally).
#[no_mangle]
pub unsafe extern "C" fn rs_add_keyword(
    name: *mut c_char,
    namelen: c_int,
    id: c_int,
    flags: c_int,
    cont_in_list: *mut i16,
    next_list: *mut i16,
    conceal_char: c_int,
) {
    if name.is_null() || namelen <= 0 {
        return;
    }

    let use_ic = nvim_syn_get_curwin_syn_ic();
    let inc_tag = crate::statics::CURRENT_SYN_INC_TAG;

    // Perform case folding if needed.
    let (name_ic_ptr, name_ic_len): (*const c_char, c_int) = if use_ic != 0 {
        use crate::types::MAXKEYWLEN;
        let buf_size = (MAXKEYWLEN + 1) as usize;
        let buf = xmalloc(buf_size) as *mut c_char;
        nvim_syn_keyword_foldcase(name, namelen, buf, MAXKEYWLEN + 1);
        let len = {
            let mut l = 0usize;
            while *buf.add(l) != 0 {
                l += 1;
            }
            l as c_int
        };
        (buf as *const c_char, len)
    } else {
        (name as *const c_char, namelen)
    };

    // Make copies of the ID lists.
    let cont_in_copy = copy_id_list_impl(cont_in_list);
    let next_copy = copy_id_list_impl(next_list);

    rs_syn_hash_insert_keyword(
        name_ic_ptr,
        name_ic_len,
        id,
        inc_tag,
        flags,
        conceal_char,
        cont_in_copy,
        next_copy,
        use_ic,
    );

    if use_ic != 0 {
        xfree(name_ic_ptr as *mut std::ffi::c_void);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_handle_checks() {
        // Test null handle creation and checking
        // Note: We can only test is_null(), not functions that call extern FFI
        let null_ke = KeyEntryHandle::null();
        assert!(null_ke.is_null());

        let null_block = SynBlockHandle(std::ptr::null_mut());
        assert!(null_block.is_null());

        let null_win = WinHandle(std::ptr::null_mut());
        assert!(null_win.is_null());

        // Non-null handle creation (for testing purposes)
        let fake_ptr = std::ptr::dangling_mut::<crate::ffi_types::KeyEntry>();
        let non_null_ke = KeyEntryHandle(fake_ptr);
        assert!(!non_null_ke.is_null());
    }

    #[test]
    fn test_max_keyword_len_constant() {
        // Test the constant value matches
        assert_eq!(MAX_KEYWORD_LEN, MAXKEYWLEN);
        // Also verify the value is 80
        assert_eq!(MAXKEYWLEN, 80);
    }

    #[test]
    fn test_keyword_iter_from_null() {
        // Test iterator creation with null handle
        // Note: We can't call next() because it triggers FFI
        let iter = KeywordIter::new(KeyEntryHandle::null());
        assert!(iter.current.is_null());
    }

    // Note: Tests that would call functions with extern FFI (even with null checks)
    // cannot be included here because the test binary doesn't link against the C library.
    // Such tests are covered by integration tests with the full build.
}
