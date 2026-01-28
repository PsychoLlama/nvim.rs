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

    // Garray accessors for help file tag names
    fn nvim_tag_fnames_len() -> c_int;
    fn nvim_tag_fnames_get(idx: c_int) -> *const c_char;
    fn nvim_tag_fnames_clear();
    fn nvim_tag_fnames_init();
    fn nvim_tag_fnames_add(fname: *mut c_char);

    // Runtime path searching
    fn nvim_do_in_runtimepath_for_tags();

    // Current buffer check
    fn nvim_curbuf_is_help() -> bool;

    // Options
    fn nvim_get_p_hf() -> *const c_char;
    fn nvim_get_curbuf_tags() -> *const c_char;
    fn nvim_get_p_tags() -> *const c_char;

    // Path functions
    fn nvim_path_tail(path: *mut c_char) -> *mut c_char;
    fn nvim_simplify_filename(fname: *mut c_char);

    // File search functions
    fn nvim_vim_findfile_init(
        path: *const c_char,
        filename: *const c_char,
        filename_len: usize,
        stopdirs: *const c_char,
        level: c_int,
        free_visited: bool,
        find_what: c_int,
        search_ctx: *mut c_void,
        tagfile: bool,
        buf_ffname: *const c_char,
    ) -> *mut c_void;
    fn nvim_vim_findfile(search_ctx: *mut c_void) -> *mut c_char;
    fn nvim_vim_findfile_cleanup(search_ctx: *mut c_void);
    fn nvim_vim_findfile_stopdir(buf: *mut c_char) -> *mut c_char;
    fn nvim_get_curbuf_ffname() -> *const c_char;

    // String functions
    fn nvim_copy_option_part(
        option: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep: *const c_char,
    );
    fn strlen(s: *const c_char) -> usize;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;

    // Path manipulation (for STRMOVE-like operation)
    fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

/// FINDFILE_FILE constant from file_search.h
const FINDFILE_FILE: c_int = 0;

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
        nvim_vim_findfile_cleanup(tnp.tn_search_ctx);
    }

    // Clear the global help tag file names
    nvim_tag_fnames_clear();

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
        nvim_vim_findfile_cleanup(tnp.tn_search_ctx);
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
        // Reset the iterator
        if !tnp.tn_tags.is_null() {
            xfree(tnp.tn_tags as *mut c_void);
            tnp.tn_tags = ptr::null_mut();
        }
        if !tnp.tn_search_ctx.is_null() {
            nvim_vim_findfile_cleanup(tnp.tn_search_ctx);
            tnp.tn_search_ctx = ptr::null_mut();
        }
        tnp.tn_did_filefind_init = 0;
        tnp.tn_hf_idx = 0;
        tnp.tn_np = ptr::null_mut();
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
        nvim_tag_fnames_clear();
        nvim_tag_fnames_init();
        nvim_do_in_runtimepath_for_tags();
    }

    let tag_fnames_len = nvim_tag_fnames_len();

    if tnp.tn_hf_idx >= tag_fnames_len {
        // Not found in 'runtimepath', use 'helpfile', if it exists and
        // wasn't used yet, replacing "help.txt" with "tags".
        let p_hf = nvim_get_p_hf();
        if tnp.tn_hf_idx > tag_fnames_len || p_hf.is_null() || *p_hf == 0 {
            return FAIL;
        }
        tnp.tn_hf_idx += 1;
        strcpy(buf, p_hf);
        let tail = nvim_path_tail(buf);
        strcpy(tail, c"tags".as_ptr());
        nvim_simplify_filename(buf);

        // Check if this file was already in the list
        for i in 0..tag_fnames_len {
            let existing = nvim_tag_fnames_get(i);
            if !existing.is_null() && strcmp(buf, existing) == 0 {
                return FAIL; // avoid duplicate file names
            }
        }
    } else {
        let fname = nvim_tag_fnames_get(tnp.tn_hf_idx);
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
            let fname = nvim_vim_findfile(tnp.tn_search_ctx);
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
                    nvim_vim_findfile_cleanup(tnp.tn_search_ctx);
                    tnp.tn_search_ctx = ptr::null_mut();
                }
                return FAIL;
            }

            // Copy next file name into buf.
            *buf = 0;
            nvim_copy_option_part(&mut tnp.tn_np, buf, MAXPATHL - 1, c" ,".as_ptr());

            let r_ptr = nvim_vim_findfile_stopdir(buf);

            // Move the filename one char forward and truncate the filepath with a NUL
            let filename = nvim_path_tail(buf);
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

            tnp.tn_search_ctx = nvim_vim_findfile_init(
                buf,
                filename,
                filename_len,
                if r_ptr.is_null() {
                    ptr::null()
                } else {
                    r_ptr.add(1)
                },
                100,
                false, // don't free visited list
                FINDFILE_FILE,
                tnp.tn_search_ctx,
                true,
                buf_ffname,
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
    unsafe { nvim_tag_fnames_len() }
}

// =============================================================================
// Helper function for C wrapper
// =============================================================================

/// Wrapper to call the tag file callback from C.
///
/// This is called by C code when finding tag files in runtimepath.
#[no_mangle]
pub unsafe extern "C" fn rs_found_tagfile_cb(
    num_fnames: c_int,
    fnames: *const *const c_char,
    _all: bool,
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

        nvim_simplify_filename(tag_fname);
        nvim_tag_fnames_add(tag_fname);
    }

    num_fnames > 0
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
