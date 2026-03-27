//! Tag file iteration for Neovim C-to-Rust migration
//!
//! This module provides Rust implementations for tag file iteration,
//! including handling of `'tags'` option parsing and help file tag discovery.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::borrow_as_ptr)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

/// Maximum path length (matches MAXPATHL in Neovim)
pub const MAXPATHL: usize = 4096;

/// Return value indicating success
const OK: c_int = 1;
/// Return value indicating failure
const FAIL: c_int = 0;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // Memory functions
    fn xfree(ptr: *mut c_void);
    fn xstrdup(s: *const c_char) -> *mut c_char;

    // Runtime path searching (calls do_in_runtimepath with hardcoded args for tags)
    fn nvim_do_in_runtimepath_for_tags();

    // Current buffer check
    fn nvim_curbuf_is_help() -> bool;

    // Options
    fn nvim_get_p_hf() -> *const c_char;
    fn nvim_get_curbuf_tags() -> *const c_char;
    fn nvim_get_p_tags() -> *const c_char;

    // Path functions (direct C functions)
    fn path_tail(fname: *const c_char) -> *mut c_char;
    fn simplify_filename(filename: *mut c_char) -> usize;

    // File search functions (direct C functions)
    fn vim_findfile_init(
        path: *mut c_char,
        filename: *mut c_char,
        filename_len: usize,
        stopdirs: *mut c_char,
        level: c_int,
        free_visited: bool,
        find_what: c_int,
        search_ctx: *mut c_void,
        tagfile: bool,
        buf_ffname: *mut c_char,
    ) -> *mut c_void;
    fn vim_findfile(search_ctx: *mut c_void) -> *mut c_char;
    fn vim_findfile_cleanup(search_ctx: *mut c_void);
    fn vim_findfile_stopdir(buf: *mut c_char) -> *mut c_char;
    fn nvim_get_curbuf_ffname() -> *const c_char;

    // String functions
    fn copy_option_part(
        option: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep_chars: *mut c_char,
    );
    fn strlen(s: *const c_char) -> usize;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;

    // Path manipulation (for STRMOVE-like operation)
    fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;

    // Path comparison (returns FileComparison enum as c_int, nonzero means equal)
    fn path_full_compare(
        s1: *mut c_char,
        s2: *mut c_char,
        checkname: bool,
        expandenv: bool,
    ) -> c_int;

    // Path utilities (direct C functions)
    fn path_has_wildcard(p: *const c_char) -> bool;
    fn nvim_expand_one_file(fname: *mut c_char) -> *mut c_char;
    fn vim_isAbsName(name: *const c_char) -> bool;
    fn nvim_get_p_tr() -> bool;
    fn xmalloc(size: usize) -> *mut c_void;
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, dstsize: usize) -> usize;
}

/// FINDFILE_FILE constant from file_search.h
const FINDFILE_FILE: c_int = 0;

/// kEqualFiles bit from FileComparison enum (path.h)
const K_EQUAL_FILES: c_int = 0x02;

// =============================================================================
// Tag file iterator state
// =============================================================================

/// State structure for iterating through tag file names.
///
/// This mirrors the C `tagname_T` structure.
#[repr(C)]
pub struct TagFileIterator {
    /// Copy of 'tags' option value
    pub tn_tags: *mut c_char,
    /// Current position in tn_tags
    pub tn_np: *mut c_char,
    /// Whether vim_findfile_init was called
    pub tn_did_filefind_init: c_int,
    /// Current index in help file tag names
    pub tn_hf_idx: c_int,
    /// File search context
    pub tn_search_ctx: *mut c_void,
}

impl Default for TagFileIterator {
    fn default() -> Self {
        Self {
            tn_tags: ptr::null_mut(),
            tn_np: ptr::null_mut(),
            tn_did_filefind_init: 0,
            tn_search_ctx: ptr::null_mut(),
            tn_hf_idx: 0,
        }
    }
}

// =============================================================================
// Iterator lifecycle
// =============================================================================

/// Create a new tag file iterator.
///
/// Returns a pointer to a new TagFileIterator or null on failure.
#[no_mangle]
pub extern "C" fn rs_tagfname_new() -> *mut TagFileIterator {
    Box::into_raw(Box::new(TagFileIterator::default()))
}

/// Free a tag file iterator and its resources.
///
/// # Safety
///
/// - `tnp` must have been created by `rs_tagfname_new` or be null
#[no_mangle]
pub unsafe extern "C" fn rs_tagfname_free(tnp: *mut TagFileIterator) {
    if tnp.is_null() {
        return;
    }

    let tnp = &mut *tnp;

    // Free the tags string copy
    if !tnp.tn_tags.is_null() {
        xfree(tnp.tn_tags as *mut c_void);
    }

    // Cleanup file search context
    if !tnp.tn_search_ctx.is_null() {
        vim_findfile_cleanup(tnp.tn_search_ctx);
    }

    // Clear the global help tag file names
    crate::tag_fnames_clear();

    drop(Box::from_raw(tnp));
}

/// Initialize a tag file iterator for a new iteration.
///
/// # Safety
///
/// - `tnp` must be a valid pointer to a TagFileIterator
#[no_mangle]
pub unsafe extern "C" fn rs_tagfname_init(tnp: *mut TagFileIterator) {
    if tnp.is_null() {
        return;
    }

    let tnp = &mut *tnp;

    // Clear previous state
    if !tnp.tn_tags.is_null() {
        xfree(tnp.tn_tags as *mut c_void);
    }
    if !tnp.tn_search_ctx.is_null() {
        vim_findfile_cleanup(tnp.tn_search_ctx);
    }

    // Reset to initial state
    tnp.tn_tags = ptr::null_mut();
    tnp.tn_np = ptr::null_mut();
    tnp.tn_did_filefind_init = 0;
    tnp.tn_hf_idx = 0;
    tnp.tn_search_ctx = ptr::null_mut();
}

// =============================================================================
// Main iteration function
// =============================================================================

/// Get the next tag file name.
///
/// For help files, searches "doc/tags" and "doc/tags-??" in 'runtimepath'.
/// For regular files, parses the 'tags' option.
///
/// # Arguments
///
/// * `tnp` - Tag file iterator state
/// * `first` - True when getting the first file name
/// * `buf` - Buffer to store the file name (must be MAXPATHL bytes)
///
/// # Returns
///
/// OK if a tag file name was found, FAIL if no more files.
///
/// # Safety
///
/// - `tnp` must be a valid pointer to a TagFileIterator
/// - `buf` must point to a buffer of at least MAXPATHL bytes
#[no_mangle]
pub unsafe extern "C" fn rs_get_tagfname(
    tnp: *mut TagFileIterator,
    first: c_int,
    buf: *mut c_char,
) -> c_int {
    if tnp.is_null() || buf.is_null() {
        return FAIL;
    }

    let tnp = &mut *tnp;

    if first != 0 {
        // Zero the entire struct, matching C's CLEAR_POINTER(tnp).
        // The struct may be stack-allocated and uninitialized on the first call,
        // so we must NOT dereference any pointers before zeroing.
        ptr::write_bytes(tnp, 0, 1);
    }

    if nvim_curbuf_is_help() {
        // For help files, search in runtimepath
        return get_help_tagfname(tnp, first != 0, buf);
    }

    // Regular tag file iteration using 'tags' option
    get_regular_tagfname(tnp, first != 0, buf)
}

/// Get the next tag file name for help files.
unsafe fn get_help_tagfname(tnp: &mut TagFileIterator, first: bool, buf: *mut c_char) -> c_int {
    if first {
        // Find all "doc/tags" and "doc/tags-??" files in runtimepath
        crate::tag_fnames_clear();
        crate::tag_fnames_init();
        nvim_do_in_runtimepath_for_tags();
    }

    let tag_fnames_len = crate::tag_fnames_len();

    if tnp.tn_hf_idx >= tag_fnames_len {
        // Not found in 'runtimepath', use 'helpfile', if it exists and
        // wasn't used yet, replacing "help.txt" with "tags".
        let p_hf = nvim_get_p_hf();
        if tnp.tn_hf_idx > tag_fnames_len || p_hf.is_null() || *p_hf == 0 {
            return FAIL;
        }
        tnp.tn_hf_idx += 1;
        strcpy(buf, p_hf);
        let tail = path_tail(buf);
        strcpy(tail, c"tags".as_ptr());
        simplify_filename(buf);

        // Check if this file was already in the list
        for i in 0..tag_fnames_len {
            let existing = crate::tag_fnames_get(i);
            if !existing.is_null() && strcmp(buf, existing) == 0 {
                return FAIL; // avoid duplicate file names
            }
        }
    } else {
        let fname = crate::tag_fnames_get(tnp.tn_hf_idx);
        tnp.tn_hf_idx += 1;
        if fname.is_null() {
            return FAIL;
        }
        // Copy the filename, ensuring we don't overflow
        let len = strlen(fname);
        if len >= MAXPATHL {
            return FAIL;
        }
        strcpy(buf, fname);
    }

    OK
}

/// Get the next tag file name for regular files.
unsafe fn get_regular_tagfname(tnp: &mut TagFileIterator, first: bool, buf: *mut c_char) -> c_int {
    if first {
        // Init. We make a copy of 'tags', because autocommands may change
        // the value without notifying us.
        let curbuf_tags = nvim_get_curbuf_tags();
        let p_tags = nvim_get_p_tags();

        let tags_source = if !curbuf_tags.is_null() && *curbuf_tags != 0 {
            curbuf_tags
        } else {
            p_tags
        };

        if tags_source.is_null() {
            return FAIL;
        }

        tnp.tn_tags = xstrdup(tags_source);
        tnp.tn_np = tnp.tn_tags;
    }

    // Loop until we have found a file name that can be used.
    // There are two states:
    // tn_did_filefind_init == 0: setup for next part in 'tags'.
    // tn_did_filefind_init == 1: find next file in this part.
    loop {
        if tnp.tn_did_filefind_init != 0 {
            let fname = vim_findfile(tnp.tn_search_ctx);
            if !fname.is_null() {
                strcpy(buf, fname);
                xfree(fname as *mut c_void);
                return OK;
            }

            tnp.tn_did_filefind_init = 0;
        } else {
            // Stop when used all parts of 'tags'.
            if tnp.tn_np.is_null() || *tnp.tn_np == 0 {
                if !tnp.tn_search_ctx.is_null() {
                    vim_findfile_cleanup(tnp.tn_search_ctx);
                    tnp.tn_search_ctx = ptr::null_mut();
                }
                return FAIL;
            }

            // Copy next file name into buf.
            *buf = 0;
            #[allow(clippy::ptr_cast_constness)]
            copy_option_part(&mut tnp.tn_np, buf, MAXPATHL - 1, c" ,".as_ptr().cast_mut());

            let r_ptr = vim_findfile_stopdir(buf);

            // Move the filename one char forward and truncate the filepath with a NUL
            let filename = path_tail(buf);
            let filename_len = strlen(filename);

            if !r_ptr.is_null() {
                // Move r_ptr content one position forward
                let r_len = strlen(r_ptr);
                memmove(
                    r_ptr.add(1) as *mut c_void,
                    r_ptr as *const c_void,
                    r_len + 1,
                );
            }

            // Move filename one position forward and add NUL before it
            let file_len = strlen(filename);
            memmove(
                filename.add(1) as *mut c_void,
                filename as *const c_void,
                file_len + 1,
            );
            *filename = 0;
            let filename = filename.add(1);

            let buf_ffname = nvim_get_curbuf_ffname();

            tnp.tn_search_ctx = vim_findfile_init(
                buf,
                filename,
                filename_len,
                if r_ptr.is_null() {
                    ptr::null_mut()
                } else {
                    r_ptr.add(1)
                },
                100,
                false, // don't free visited list
                FINDFILE_FILE,
                tnp.tn_search_ctx,
                true,
                buf_ffname.cast_mut(),
            );

            if !tnp.tn_search_ctx.is_null() {
                tnp.tn_did_filefind_init = 1;
            }
        }
    }
}

// =============================================================================
// Tag file iterator accessors
// =============================================================================

/// Get the current help file index.
#[no_mangle]
pub unsafe extern "C" fn rs_tagfname_get_hf_idx(tnp: *const TagFileIterator) -> c_int {
    if tnp.is_null() {
        return 0;
    }
    (*tnp).tn_hf_idx
}

/// Set the current help file index.
#[no_mangle]
pub unsafe extern "C" fn rs_tagfname_set_hf_idx(tnp: *mut TagFileIterator, idx: c_int) {
    if tnp.is_null() {
        return;
    }
    (*tnp).tn_hf_idx = idx;
}

/// Check if the iterator has been initialized for file finding.
#[no_mangle]
pub unsafe extern "C" fn rs_tagfname_did_init(tnp: *const TagFileIterator) -> bool {
    if tnp.is_null() {
        return false;
    }
    (*tnp).tn_did_filefind_init != 0
}

/// Check if the iterator has more tags entries to process.
#[no_mangle]
pub unsafe extern "C" fn rs_tagfname_has_more(tnp: *const TagFileIterator) -> bool {
    if tnp.is_null() {
        return false;
    }
    let tnp = &*tnp;
    !tnp.tn_np.is_null() && *tnp.tn_np != 0
}

/// Get the number of help tag files found.
#[no_mangle]
pub extern "C" fn rs_tag_fnames_count() -> c_int {
    unsafe { crate::tag_fnames_len() }
}

// =============================================================================
// Helper function for C wrapper
// =============================================================================

/// Callback for finding all "tags" and "tags-??" files in 'runtimepath' doc
/// directories. Called by `do_in_runtimepath`.
///
/// # Safety
///
/// - `fnames` must point to an array of `num_fnames` valid C strings
#[no_mangle]
pub unsafe extern "C" fn rs_found_tagfile_cb(
    num_fnames: c_int,
    fnames: *mut *mut c_char,
    all: bool,
    _cookie: *mut c_void,
) -> bool {
    if fnames.is_null() || num_fnames <= 0 {
        return false;
    }

    for i in 0..num_fnames {
        let fname = *fnames.add(i as usize);
        if fname.is_null() {
            continue;
        }

        let tag_fname = xstrdup(fname);
        if tag_fname.is_null() {
            continue;
        }

        simplify_filename(tag_fname);
        crate::tag_fnames_add(tag_fname);

        if !all {
            break;
        }
    }

    num_fnames > 0
}

// =============================================================================
// Phase 1: Tag filename and current-file utilities
// =============================================================================

use crate::parse::TagPtrs;

/// Find the actual file name of a tag by expanding the tag file's relative path.
///
/// Temporarily null-terminates the fname field, calls expand_tag_fname, then
/// restores the original character.
///
/// Returns an allocated string (caller must free with xfree).
///
/// # Safety
///
/// - `tagp` must be a valid pointer to a `TagPtrs` struct
/// - `tagp.fname` and `tagp.fname_end` must be valid pointers
/// - `tagp.tag_fname` must be a valid C string
#[no_mangle]
pub unsafe extern "C" fn rs_tag_full_fname(tagp: *mut TagPtrs) -> *mut c_char {
    if tagp.is_null() {
        return ptr::null_mut();
    }

    let tagp = &mut *tagp;

    if tagp.fname.is_null() || tagp.fname_end.is_null() || tagp.tag_fname.is_null() {
        return ptr::null_mut();
    }

    // Save and null-terminate the filename
    let saved_char = *tagp.fname_end;
    *tagp.fname_end = 0;

    let fullname = rs_expand_tag_fname(tagp.fname, tagp.tag_fname, false);

    // Restore the original character
    *tagp.fname_end = saved_char;

    fullname
}

/// Check if a tag is for the current buffer.
///
/// Expands the tag's filename and compares it to the buffer's full filename.
///
/// Returns nonzero (true) if the tag is for the current file.
///
/// # Safety
///
/// - All pointers must be valid
/// - `fname_end` must point into the same buffer as `fname`
#[no_mangle]
pub unsafe extern "C" fn rs_test_for_current(
    fname: *mut c_char,
    fname_end: *mut c_char,
    tag_fname: *mut c_char,
    buf_ffname: *mut c_char,
) -> c_int {
    if buf_ffname.is_null() {
        return 0;
    }

    // Save and null-terminate the filename
    let saved_char = *fname_end;
    *fname_end = 0;

    let fullname = rs_expand_tag_fname(fname, tag_fname, true);
    let retval = path_full_compare(fullname as *mut c_char, buf_ffname, true, true) & K_EQUAL_FILES;

    xfree(fullname as *mut c_void);

    // Restore the original character
    *fname_end = saved_char;

    retval
}

// =============================================================================
// Phase 3: expand_tag_fname
// =============================================================================

/// If `expand` is true, expand wildcards in `fname`.
/// If 'tagrelative' option is set, change fname (name of file containing tag)
/// according to `tag_fname` (name of tag file containing fname).
///
/// Returns a pointer to allocated memory (caller must free).
///
/// # Safety
///
/// - `fname` and `tag_fname` must be valid C strings
#[no_mangle]
pub unsafe extern "C" fn rs_expand_tag_fname(
    mut fname: *mut c_char,
    tag_fname: *mut c_char,
    expand: bool,
) -> *mut c_char {
    let mut expanded_fname: *mut c_char = ptr::null_mut();

    // Expand file name (for environment variables) when needed.
    if expand && path_has_wildcard(fname) {
        expanded_fname = nvim_expand_one_file(fname);
        if !expanded_fname.is_null() {
            fname = expanded_fname;
        }
    }

    let retval;
    let p_tr = nvim_get_p_tr();
    let is_help = nvim_curbuf_is_help();

    if (p_tr || is_help) && !vim_isAbsName(fname) {
        let p = path_tail(tag_fname);
        if p == tag_fname {
            retval = xstrdup(fname);
        } else {
            retval = xmalloc(MAXPATHL).cast::<c_char>();
            strcpy(retval, tag_fname);
            let offset = p.offset_from(tag_fname) as usize;
            xstrlcpy(retval.add(offset), fname, MAXPATHL - offset);
            // Translate names like "src/a/../b/file.c" into "src/b/file.c".
            simplify_filename(retval);
        }
    } else {
        retval = xstrdup(fname);
    }

    xfree(expanded_fname as *mut c_void);

    retval
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterator_default() {
        let iter = TagFileIterator::default();
        assert!(iter.tn_tags.is_null());
        assert!(iter.tn_np.is_null());
        assert_eq!(iter.tn_did_filefind_init, 0);
        assert_eq!(iter.tn_hf_idx, 0);
        assert!(iter.tn_search_ctx.is_null());
    }

    #[test]
    fn test_iterator_new() {
        let iter = rs_tagfname_new();
        assert!(!iter.is_null());
        unsafe {
            assert!((*iter).tn_tags.is_null());
            // Note: Can't call rs_tagfname_free in tests without C runtime
        }
    }

    #[test]
    fn test_maxpathl_constant() {
        assert_eq!(MAXPATHL, 4096);
    }

    #[test]
    fn test_null_safety() {
        unsafe {
            // These should not crash with null pointers
            rs_tagfname_init(ptr::null_mut());
            assert_eq!(rs_get_tagfname(ptr::null_mut(), 1, ptr::null_mut()), FAIL);
            assert_eq!(rs_tagfname_get_hf_idx(ptr::null()), 0);
            rs_tagfname_set_hf_idx(ptr::null_mut(), 5);
            assert!(!rs_tagfname_did_init(ptr::null()));
            assert!(!rs_tagfname_has_more(ptr::null()));
        }
    }

    #[test]
    fn test_get_tagfname_null_buf() {
        let iter = rs_tagfname_new();
        unsafe {
            assert_eq!(rs_get_tagfname(iter, 1, ptr::null_mut()), FAIL);
        }
    }
}
