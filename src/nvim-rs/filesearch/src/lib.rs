//! File search infrastructure for Neovim
//!
//! Provides file searching for 'path', 'tags' and 'cdpath' options.
//!
//! The search algorithm is depth-first with support for:
//! - Wildcard patterns (`*`, `**`)
//! - Upward search (using stopdirs)
//! - Visited file tracking to avoid cycles
//!
//! Key functions:
//! - `vim_findfile_init` - Create/initialize search context
//! - `vim_findfile` - Find next matching file
//! - `vim_findfile_cleanup` - Free search context
//! - `vim_findfile_free_visited` - Clear visited list

#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]

use std::ffi::{c_char, c_int};
use std::ptr;

use nvim_os::fs::FileID;

// ============================================================================
// Constants
// ============================================================================

/// Maximum depth for `**` wildcard expansion.
const FF_MAX_STAR_STAR_EXPAND: u8 = 30;

/// Find only files.
pub const FINDFILE_FILE: c_int = 0;
/// Find only directories.
pub const FINDFILE_DIR: c_int = 1;
/// Find both files and directories.
pub const FINDFILE_BOTH: c_int = 2;

/// Return values matching nvim's OK/FAIL.
const OK: c_int = 1;
const FAIL: c_int = 0;

/// Maximum path length.
const MAXPATHL: usize = 4096;

// ============================================================================
// External C functions
// ============================================================================

extern "C" {
    // Memory allocation
    fn xmalloc(size: usize) -> *mut libc::c_void;
    fn xcalloc(count: usize, size: usize) -> *mut libc::c_void;
    fn xrealloc(ptr: *mut libc::c_void, size: usize) -> *mut libc::c_void;
    fn xfree(ptr: *mut libc::c_void);
    fn xmemdupz(data: *const u8, len: usize) -> *mut c_char;

    // Path functions
    fn path_tail(fname: *const c_char) -> *const c_char;
    fn vim_ispathsep(c: c_int) -> c_int;
    fn after_pathsep(b: *const c_char, p: *const c_char) -> c_int;
    fn vim_isAbsName(name: *const c_char) -> c_int;
    fn path_with_url(p: *const c_char) -> c_int;
    fn path_fnamecmp(fname1: *const c_char, fname2: *const c_char) -> c_int;
    fn path_fnamencmp(fname1: *const c_char, fname2: *const c_char, len: usize) -> c_int;
    fn simplify_filename(filename: *mut c_char) -> usize;
    fn path_shorten_fname(full_path: *mut c_char, dir_name: *const c_char) -> *mut c_char;
    fn FullName_save(fname: *const c_char, force: bool) -> *mut c_char;

    // OS functions
    fn os_dirname(buf: *mut c_char, len: usize) -> c_int;
    fn os_isdir(name: *const c_char) -> c_int;
    fn os_path_exists(name: *const c_char) -> c_int;
    fn os_fileid(path: *const c_char, file_id: *mut FileID) -> bool;
    fn os_fileid_equal(id1: *const FileID, id2: *const FileID) -> bool;
    fn os_breakcheck();

    // Wildcard expansion
    fn expand_wildcards(
        num_pat: c_int,
        pat: *mut *mut c_char,
        num_files: *mut c_int,
        files: *mut *mut *mut c_char,
        flags: c_int,
    ) -> c_int;
    fn FreeWild(count: c_int, files: *mut *mut c_char);

    // String functions
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn vim_snprintf(str: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;

    // Error handling
    fn emsg(s: *const c_char);
    fn semsg(fmt: *const c_char, ...);

    // Global state
    static mut got_int: c_int;

    // Option values
    fn nvim_get_p_fic() -> c_int;

    // Current buffer suffix option
    fn nvim_get_curbuf_sua() -> *const c_char;
    fn copy_option_part(
        option: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep_chars: *const c_char,
    ) -> usize;

    // Multibyte
    fn utf_ptr2char(p: *const u8) -> c_int;
    fn utfc_ptr2len(p: *const u8) -> usize;
    #[allow(dead_code)]
    fn utf_head_off(base: *const u8, p: *const u8) -> usize;
    fn mb_tolower(c: c_int) -> c_int;
}

// ============================================================================
// Types - Search Stack
// ============================================================================

/// Stack element for directory search.
///
/// The search uses a stack-based depth-first traversal.
#[repr(C)]
struct FfStack {
    /// Previous stack element (linked list).
    prev: *mut FfStack,

    /// Fixed part of path (no wildcards).
    fix_path: NvimString,

    /// Part containing wildcards.
    wc_path: NvimString,

    /// Files/dirs found by wildcard expansion.
    filearray: *mut *mut c_char,
    filearray_size: c_int,
    filearray_cur: c_int,

    /// 0 = first time working on this dir, 1 = partly searched.
    stage: c_int,

    /// How deep in directory tree (counts down from initial level).
    level: c_int,

    /// Did we already expand `**` to empty string?
    star_star_empty: c_int,
}

// ============================================================================
// Types - Visited Tracking
// ============================================================================

/// Visited file/directory entry.
///
/// Uses FileID for comparison when possible (handles hardlinks),
/// otherwise uses filename.
#[repr(C)]
struct FfVisited {
    /// Next visited entry (linked list).
    next: *mut FfVisited,

    /// Wildcard path at time of visit.
    wc_path: *mut c_char,

    /// Use FileID for comparison?
    file_id_valid: bool,
    file_id: FileID,

    /// Filename (flexible array member, allocated inline).
    /// This is actually a flexible array - the struct is allocated
    /// with extra space for the filename.
    fname: [c_char; 0],
}

/// Header for a visited list (grouped by filename being searched).
#[repr(C)]
struct FfVisitedListHdr {
    /// Next list header.
    next: *mut FfVisitedListHdr,

    /// The filename this visited list is for.
    filename: *mut c_char,

    /// The actual visited entries.
    visited_list: *mut FfVisited,
}

// ============================================================================
// Types - Search Context
// ============================================================================

/// Simple string type matching nvim's String.
#[repr(C)]
#[derive(Clone, Copy)]
struct NvimString {
    data: *mut c_char,
    size: usize,
}

impl NvimString {
    const NULL: Self = Self {
        data: ptr::null_mut(),
        size: 0,
    };

    fn is_null(&self) -> bool {
        self.data.is_null()
    }

    /// Free the string data.
    unsafe fn free(&mut self) {
        if !self.data.is_null() {
            xfree(self.data.cast());
            self.data = ptr::null_mut();
            self.size = 0;
        }
    }
}

/// The main search context.
///
/// Contains all state needed for a file search operation.
#[repr(C)]
struct FfSearchCtx {
    /// Stack of directories to search.
    stack_ptr: *mut FfStack,

    /// Currently active visited list.
    visited_list: *mut FfVisitedListHdr,

    /// Currently active visited list for search dirs.
    dir_visited_list: *mut FfVisitedListHdr,

    /// All visited lists (for file searches).
    visited_lists_list: *mut FfVisitedListHdr,

    /// All visited lists (for directory searches).
    dir_visited_lists_list: *mut FfVisitedListHdr,

    /// The file we're searching for.
    file_to_search: NvimString,

    /// Starting directory (for relative paths).
    start_dir: NvimString,

    /// Fixed part of search path.
    fix_path: NvimString,

    /// Wildcard part of search path.
    wc_path: NvimString,

    /// Max depth for downward search.
    level: c_int,

    /// Stop directories for upward search.
    stopdirs_v: *mut NvimString,

    /// What to find: FINDFILE_FILE, _DIR, or _BOTH.
    find_what: c_int,

    /// Searching for tags file (don't use 'suffixesadd').
    tagfile: c_int,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a string from a C buffer.
unsafe fn cbuf_to_string(data: *const c_char, len: usize) -> NvimString {
    if data.is_null() || len == 0 {
        return NvimString::NULL;
    }
    let ptr = xmemdupz(data.cast(), len);
    NvimString {
        data: ptr,
        size: len,
    }
}

/// Copy a string, optionally appending a path separator.
unsafe fn copy_string(src: &NvimString, add_sep: bool) -> NvimString {
    if src.is_null() {
        return NvimString::NULL;
    }
    let extra = usize::from(add_sep);
    let new_size = src.size + extra;
    let ptr = xmalloc(new_size + 1).cast::<c_char>();
    ptr::copy_nonoverlapping(src.data.cast::<u8>(), ptr.cast::<u8>(), src.size);
    if add_sep {
        *ptr.add(src.size) = b'/' as c_char;
    }
    *ptr.add(new_size) = 0;
    NvimString {
        data: ptr,
        size: new_size,
    }
}

/// Create stack element from path pieces.
unsafe fn ff_create_stack_element(
    fix_part: *const c_char,
    fix_partlen: usize,
    wc_part: *const c_char,
    wc_partlen: usize,
    level: c_int,
    star_star_empty: c_int,
) -> *mut FfStack {
    let stack = xmalloc(std::mem::size_of::<FfStack>()).cast::<FfStack>();

    (*stack).prev = ptr::null_mut();
    (*stack).filearray = ptr::null_mut();
    (*stack).filearray_size = 0;
    (*stack).filearray_cur = 0;
    (*stack).stage = 0;
    (*stack).level = level;
    (*stack).star_star_empty = star_star_empty;

    // The following saves NULL pointer checks in vim_findfile
    let (fp, fplen) = if fix_part.is_null() {
        (c"".as_ptr(), 0)
    } else {
        (fix_part, fix_partlen)
    };
    (*stack).fix_path = cbuf_to_string(fp, fplen);

    let (wp, wplen) = if wc_part.is_null() {
        (c"".as_ptr(), 0)
    } else {
        (wc_part, wc_partlen)
    };
    (*stack).wc_path = cbuf_to_string(wp, wplen);

    stack
}

/// Free a stack element and its contents.
unsafe fn ff_free_stack_element(stack_ptr: *mut FfStack) {
    if stack_ptr.is_null() {
        return;
    }

    (*stack_ptr).fix_path.free();
    (*stack_ptr).wc_path.free();

    if !(*stack_ptr).filearray.is_null() {
        FreeWild((*stack_ptr).filearray_size, (*stack_ptr).filearray);
    }

    xfree(stack_ptr.cast());
}

/// Push a directory onto the search stack.
unsafe fn ff_push(search_ctx: *mut FfSearchCtx, stack_ptr: *mut FfStack) {
    if stack_ptr.is_null() {
        return;
    }

    (*stack_ptr).prev = (*search_ctx).stack_ptr;
    (*search_ctx).stack_ptr = stack_ptr;
}

/// Pop a directory from the search stack.
unsafe fn ff_pop(search_ctx: *mut FfSearchCtx) -> *mut FfStack {
    let sptr = (*search_ctx).stack_ptr;
    if !(*search_ctx).stack_ptr.is_null() {
        (*search_ctx).stack_ptr = (*(*search_ctx).stack_ptr).prev;
    }
    sptr
}

/// Clear the search context, but NOT the visited lists.
unsafe fn ff_clear(search_ctx: *mut FfSearchCtx) {
    // Clear up stack
    loop {
        let sptr = ff_pop(search_ctx);
        if sptr.is_null() {
            break;
        }
        ff_free_stack_element(sptr);
    }

    // Free stopdirs
    if !(*search_ctx).stopdirs_v.is_null() {
        let mut i = 0;
        while !(*(*search_ctx).stopdirs_v.add(i)).data.is_null() {
            xfree((*(*search_ctx).stopdirs_v.add(i)).data.cast());
            i += 1;
        }
        xfree((*search_ctx).stopdirs_v.cast());
        (*search_ctx).stopdirs_v = ptr::null_mut();
    }

    // Reset string fields
    (*search_ctx).file_to_search.free();
    (*search_ctx).start_dir.free();
    (*search_ctx).fix_path.free();
    (*search_ctx).wc_path.free();
    (*search_ctx).level = 0;
}

/// Free a visited list.
unsafe fn ff_free_visited_list(mut vl: *mut FfVisited) {
    while !vl.is_null() {
        let vp = (*vl).next;
        xfree((*vl).wc_path.cast());
        xfree(vl.cast());
        vl = vp;
    }
}

/// Free a visited list header and all its entries.
unsafe fn vim_findfile_free_visited_list(list_headp: *mut *mut FfVisitedListHdr) {
    while !(*list_headp).is_null() {
        let vp = (**list_headp).next;
        ff_free_visited_list((**list_headp).visited_list);
        xfree((**list_headp).filename.cast());
        xfree((*list_headp).cast());
        *list_headp = vp;
    }
    *list_headp = ptr::null_mut();
}

/// Check if two wildcard paths are equal.
///
/// They are equal if:
/// - Both are NULL
/// - Same length and char-by-char match
/// - Only differences are in counters behind `**`
unsafe fn ff_wc_equal(s1: *const c_char, s2: *const c_char) -> bool {
    if s1 == s2 {
        return true;
    }
    if s1.is_null() || s2.is_null() {
        return false;
    }

    let mut i = 0usize;
    let mut j = 0usize;
    let mut prev1 = 0i32;
    let mut prev2 = 0i32;

    loop {
        let c1 = utf_ptr2char(s1.add(i).cast());
        let c2 = utf_ptr2char(s2.add(j).cast());

        if c1 == 0 && c2 == 0 {
            return true;
        }
        if c1 == 0 || c2 == 0 {
            return false;
        }

        let fic = nvim_get_p_fic() != 0;
        let c1_cmp = if fic { mb_tolower(c1) } else { c1 };
        let c2_cmp = if fic { mb_tolower(c2) } else { c2 };

        if c1_cmp != c2_cmp && !(prev1 == b'*' as i32 && prev2 == b'*' as i32) {
            return false;
        }

        prev2 = prev1;
        prev1 = c1;

        i += utfc_ptr2len(s1.add(i).cast());
        j += utfc_ptr2len(s2.add(j).cast());
    }
}

/// Get or create a visited list for the given filename.
unsafe fn ff_get_visited_list(
    filename: *const c_char,
    filenamelen: usize,
    list_headp: *mut *mut FfVisitedListHdr,
) -> *mut FfVisitedListHdr {
    // Check if a visited list for the given filename exists
    if !(*list_headp).is_null() {
        let mut retptr = *list_headp;
        while !retptr.is_null() {
            if path_fnamecmp(filename, (*retptr).filename) == 0 {
                return retptr;
            }
            retptr = (*retptr).next;
        }
    }

    // Allocate new list
    let retptr = xmalloc(std::mem::size_of::<FfVisitedListHdr>()).cast::<FfVisitedListHdr>();

    (*retptr).visited_list = ptr::null_mut();
    (*retptr).filename = xmemdupz(filename.cast(), filenamelen);
    (*retptr).next = *list_headp;
    *list_headp = retptr;

    retptr
}

/// Maintains the list of already visited files and dirs.
///
/// Returns FAIL if the given file/dir is already in the list,
/// OK if it is newly added.
unsafe fn ff_check_visited(
    visited_list: *mut *mut FfVisited,
    fname: *const c_char,
    fnamelen: usize,
    wc_path: *const c_char,
    wc_pathlen: usize,
) -> c_int {
    let mut url = false;
    let mut file_id: FileID = std::mem::zeroed();

    // Expand buffer for URL comparison
    let expand_buffer = xmalloc(MAXPATHL).cast::<c_char>();

    // For a URL we only compare the name, otherwise we compare the device/inode
    if path_with_url(fname) != 0 {
        ptr::copy_nonoverlapping(fname.cast::<u8>(), expand_buffer.cast::<u8>(), fnamelen);
        *expand_buffer.add(fnamelen) = 0;
        url = true;
    } else {
        *expand_buffer = 0;
        if !os_fileid(fname, std::ptr::addr_of_mut!(file_id)) {
            xfree(expand_buffer.cast());
            return FAIL;
        }
    }

    // Check against list of already visited files
    let mut vp = *visited_list;
    while !vp.is_null() {
        let fname_ptr = (*vp).fname.as_ptr();
        if (url && path_fnamecmp(fname_ptr, expand_buffer) == 0)
            || (!url
                && (*vp).file_id_valid
                && os_fileid_equal(
                    std::ptr::addr_of!((*vp).file_id),
                    std::ptr::addr_of!(file_id),
                ))
        {
            // Are the wildcard parts equal?
            if ff_wc_equal((*vp).wc_path, wc_path) {
                xfree(expand_buffer.cast());
                return FAIL; // Already visited
            }
        }
        vp = (*vp).next;
    }

    // New file/dir. Add it to the list.
    let expand_len = if url { fnamelen } else { 0 };
    let vp_size = std::mem::size_of::<FfVisited>() + expand_len + 1;
    let new_vp = xmalloc(vp_size).cast::<FfVisited>();

    if url {
        (*new_vp).file_id_valid = false;
        ptr::copy_nonoverlapping(
            expand_buffer.cast::<u8>(),
            (*new_vp).fname.as_mut_ptr().cast::<u8>(),
            expand_len + 1,
        );
    } else {
        (*new_vp).file_id_valid = true;
        (*new_vp).file_id = file_id;
        // fname will be empty for non-URL
        *(*new_vp).fname.as_mut_ptr() = 0;
    }

    if !wc_path.is_null() && wc_pathlen > 0 {
        (*new_vp).wc_path = xmemdupz(wc_path.cast(), wc_pathlen);
    } else {
        (*new_vp).wc_path = ptr::null_mut();
    }

    (*new_vp).next = *visited_list;
    *visited_list = new_vp;

    xfree(expand_buffer.cast());
    OK
}

/// Check if the given path is in the stopdirs.
unsafe fn ff_path_in_stoplist(
    path: *const c_char,
    mut path_len: usize,
    stopdirs_v: *const NvimString,
) -> bool {
    // Eat up trailing path separators, except the first
    while path_len > 1 && vim_ispathsep(*path.add(path_len - 1) as c_int) != 0 {
        path_len -= 1;
    }

    // If no path, consider it as match
    if path_len == 0 {
        return true;
    }

    let mut i = 0;
    while !(*stopdirs_v.add(i)).data.is_null() {
        // Match for parent directory
        if path_fnamencmp((*stopdirs_v.add(i)).data, path, path_len) == 0
            && ((*stopdirs_v.add(i)).size <= path_len
                || vim_ispathsep(*(*stopdirs_v.add(i)).data.add(path_len) as c_int) != 0)
        {
            return true;
        }
        i += 1;
    }

    false
}

// ============================================================================
// Public API - Initialization
// ============================================================================

/// Initialize the file search.
///
/// Returns the newly allocated search context or NULL if an error occurred.
///
/// # Parameters
/// - `path`: The path to search in (may contain wildcards)
/// - `filename`: The file name to search for (no wildcards)
/// - `filenamelen`: Length of filename
/// - `stopdirs`: Stop directories for upward search (semicolon separated)
/// - `level`: Maximum depth for downward search
/// - `free_visited`: If true, clear the visited list
/// - `find_what`: FINDFILE_FILE, FINDFILE_DIR, or FINDFILE_BOTH
/// - `search_ctx_arg`: Existing search context to reuse (or NULL)
/// - `tagfile`: Searching for tags file (don't use 'suffixesadd')
/// - `rel_fname`: File name for relative path resolution
///
/// # Safety
/// All pointer parameters must be valid or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_findfile_init(
    path: *mut c_char,
    filename: *const c_char,
    filenamelen: usize,
    stopdirs: *const c_char,
    level: c_int,
    free_visited: c_int,
    find_what: c_int,
    search_ctx_arg: *mut libc::c_void,
    tagfile: c_int,
    rel_fname: *const c_char,
) -> *mut libc::c_void {
    // Allocate or reuse search context
    let search_ctx = if search_ctx_arg.is_null() {
        xcalloc(1, std::mem::size_of::<FfSearchCtx>()).cast::<FfSearchCtx>()
    } else {
        search_ctx_arg.cast::<FfSearchCtx>()
    };

    (*search_ctx).find_what = find_what;
    (*search_ctx).tagfile = tagfile;

    // Clear the search context, but NOT the visited lists
    ff_clear(search_ctx);

    // Clear visited list if wanted
    if free_visited != 0 {
        rs_vim_findfile_free_visited(search_ctx.cast());
    } else {
        // Reuse old visited lists
        (*search_ctx).visited_list = ff_get_visited_list(
            filename,
            filenamelen,
            std::ptr::addr_of_mut!((*search_ctx).visited_lists_list),
        );
        if (*search_ctx).visited_list.is_null() {
            rs_vim_findfile_cleanup(search_ctx.cast());
            return ptr::null_mut();
        }
        (*search_ctx).dir_visited_list = ff_get_visited_list(
            filename,
            filenamelen,
            std::ptr::addr_of_mut!((*search_ctx).dir_visited_lists_list),
        );
        if (*search_ctx).dir_visited_list.is_null() {
            rs_vim_findfile_cleanup(search_ctx.cast());
            return ptr::null_mut();
        }
    }

    // Allocate expand buffer
    let ff_expand_buffer = xmalloc(MAXPATHL).cast::<c_char>();
    #[allow(unused_assignments)]
    let mut ff_expand_size: usize = 0;

    // Get CPO_DOTTAG value (we need to check if '.' is in cpo)
    // For now, assume tagfile handling is simpler
    let path_byte0 = if path.is_null() { 0 } else { *path as u8 };
    let path_byte1 = if path.is_null() || path_byte0 == 0 {
        0
    } else {
        *path.add(1) as u8
    };

    let mut path_ptr = path;

    // Store information on starting dir now if path is relative
    if path_byte0 == b'.'
        && (vim_ispathsep(path_byte1 as c_int) != 0 || path_byte1 == 0)
        && !rel_fname.is_null()
    {
        let tail = path_tail(rel_fname);
        let len = (tail as usize) - (rel_fname as usize);

        if vim_isAbsName(rel_fname) == 0 && len + 1 < MAXPATHL {
            // Make the start dir an absolute path name
            ptr::copy_nonoverlapping(rel_fname.cast::<u8>(), ff_expand_buffer.cast::<u8>(), len);
            *ff_expand_buffer.add(len) = 0;
            let _ = len; // Buffer is now set up
            let full = FullName_save(ff_expand_buffer, false);
            (*search_ctx).start_dir = NvimString {
                data: full,
                size: if full.is_null() {
                    0
                } else {
                    libc::strlen(full)
                },
            };
        } else {
            (*search_ctx).start_dir = cbuf_to_string(rel_fname, len);
        }

        // Skip past "./" in path
        if *path_ptr.add(1) != 0 {
            path_ptr = path_ptr.add(2);
        } else {
            path_ptr = path_ptr.add(1);
        }
    } else if path_byte0 == 0 || vim_isAbsName(path) == 0 {
        // Get current directory
        if os_dirname(ff_expand_buffer, MAXPATHL) == FAIL {
            xfree(ff_expand_buffer.cast());
            rs_vim_findfile_cleanup(search_ctx.cast());
            return ptr::null_mut();
        }
        ff_expand_size = libc::strlen(ff_expand_buffer);
        (*search_ctx).start_dir = cbuf_to_string(ff_expand_buffer, ff_expand_size);
    }

    // Parse stopdirs
    if !stopdirs.is_null() {
        let mut walker = stopdirs;

        // Skip leading semicolons
        while *walker == b';' as c_char {
            walker = walker.add(1);
        }

        let mut dircount: usize = 1;
        (*search_ctx).stopdirs_v = xmalloc(std::mem::size_of::<NvimString>()).cast::<NvimString>();

        loop {
            let helper = walker;
            let new_ptr = xrealloc(
                (*search_ctx).stopdirs_v.cast(),
                (dircount + 1) * std::mem::size_of::<NvimString>(),
            )
            .cast::<NvimString>();
            (*search_ctx).stopdirs_v = new_ptr;

            let semi = vim_strchr(walker, b';' as c_int);
            let len = if semi.is_null() {
                libc::strlen(helper)
            } else {
                (semi as usize) - (helper as usize)
            };

            // "" means ascent till top of directory tree
            if *helper != 0 && vim_isAbsName(helper) == 0 && len + 1 < MAXPATHL {
                // Make the stop dir an absolute path name
                ptr::copy_nonoverlapping(helper.cast::<u8>(), ff_expand_buffer.cast::<u8>(), len);
                *ff_expand_buffer.add(len) = 0;
                let full = FullName_save(helper, false);
                *(*search_ctx).stopdirs_v.add(dircount - 1) = NvimString {
                    data: full,
                    size: if full.is_null() {
                        0
                    } else {
                        libc::strlen(full)
                    },
                };
            } else {
                *(*search_ctx).stopdirs_v.add(dircount - 1) = cbuf_to_string(helper, len);
            }

            if semi.is_null() {
                break;
            }
            walker = semi.add(1);
            dircount += 1;
        }

        // Terminate the array
        *(*search_ctx).stopdirs_v.add(dircount) = NvimString::NULL;
    }

    (*search_ctx).level = level;

    // Split into fix path and wildcard stuff
    let wc_part = vim_strchr(path_ptr, b'*' as c_int);

    if wc_part.is_null() {
        (*search_ctx).fix_path = NvimString {
            data: xmemdupz(path_ptr.cast(), libc::strlen(path_ptr)),
            size: libc::strlen(path_ptr),
        };
    } else {
        // Save the fix part of the path
        let fix_len = (wc_part as usize) - (path_ptr as usize);
        (*search_ctx).fix_path = cbuf_to_string(path_ptr, fix_len);

        // Process wildcard path with ** restriction encoding
        ff_expand_size = 0;
        let mut wp = wc_part;

        while *wp != 0 {
            if ff_expand_size + 5 >= MAXPATHL {
                // Path too long
                static E854: &[u8] = b"E854: Path too long for completion\0";
                emsg(E854.as_ptr().cast());
                break;
            }

            *ff_expand_buffer.add(ff_expand_size) = *wp;
            ff_expand_size += 1;
            if libc::strncmp(wp, c"**".as_ptr(), 2) == 0 {
                wp = wp.add(1);
                *ff_expand_buffer.add(ff_expand_size) = *wp;
                ff_expand_size += 1;
                wp = wp.add(1);

                // Parse the restriction number after **
                let mut errpt: *mut c_char = ptr::null_mut();
                let llevel = libc::strtol(wp, std::ptr::addr_of_mut!(errpt), 10);

                if errpt != wp && llevel > 0 && llevel < 255 {
                    *ff_expand_buffer.add(ff_expand_size) = llevel as c_char;
                    ff_expand_size += 1;
                } else if errpt != wp && llevel == 0 {
                    // Restrict is 0 -> remove already added **
                    ff_expand_size -= 2;
                } else {
                    *ff_expand_buffer.add(ff_expand_size) = FF_MAX_STAR_STAR_EXPAND as c_char;
                    ff_expand_size += 1;
                }
                wp = errpt;

                if *wp != 0 && vim_ispathsep(*wp as c_int) == 0 {
                    static E343: &[u8] =
                        b"E343: Invalid path: '**[number]' must be at the end of the path or be followed by '%s'.\0";
                    static SEP: &[u8] = b"/\0";
                    semsg(E343.as_ptr().cast(), SEP.as_ptr());
                    xfree(ff_expand_buffer.cast());
                    rs_vim_findfile_cleanup(search_ctx.cast());
                    return ptr::null_mut();
                }
            } else {
                wp = wp.add(1);
            }
        }
        *ff_expand_buffer.add(ff_expand_size) = 0;
        (*search_ctx).wc_path = cbuf_to_string(ff_expand_buffer, ff_expand_size);
    }

    // If start_dir is still NULL, use fix_path as start_dir
    if (*search_ctx).start_dir.data.is_null() {
        (*search_ctx).start_dir = copy_string(&(*search_ctx).fix_path, false);
        *(*search_ctx).fix_path.data = 0;
        (*search_ctx).fix_path.size = 0;
    }

    // Create an absolute path
    let total_len = (*search_ctx).start_dir.size + (*search_ctx).fix_path.size + 3;
    if total_len >= MAXPATHL {
        static E854: &[u8] = b"E854: Path too long for completion\0";
        emsg(E854.as_ptr().cast());
        xfree(ff_expand_buffer.cast());
        rs_vim_findfile_cleanup(search_ctx.cast());
        return ptr::null_mut();
    }

    let start_end = (*search_ctx)
        .start_dir
        .data
        .add((*search_ctx).start_dir.size);
    let add_sep = after_pathsep((*search_ctx).start_dir.data, start_end) == 0;

    ff_expand_size = vim_snprintf(
        ff_expand_buffer,
        MAXPATHL,
        c"%s%s".as_ptr(),
        (*search_ctx).start_dir.data,
        if add_sep { c"/".as_ptr() } else { c"".as_ptr() },
    ) as usize;

    // Build full path and check if it's a directory
    let bufsize = ff_expand_size + (*search_ctx).fix_path.size + 1;
    let buf = xmalloc(bufsize).cast::<c_char>();
    vim_snprintf(
        buf,
        bufsize,
        c"%s%s".as_ptr(),
        ff_expand_buffer,
        (*search_ctx).fix_path.data,
    );

    if os_isdir(buf) != 0 {
        if (*search_ctx).fix_path.size > 0 {
            let fix_end = (*search_ctx).fix_path.data.add((*search_ctx).fix_path.size);
            let add_sep2 = after_pathsep((*search_ctx).fix_path.data, fix_end) == 0;
            ff_expand_size += vim_snprintf(
                ff_expand_buffer.add(ff_expand_size),
                MAXPATHL - ff_expand_size,
                c"%s%s".as_ptr(),
                (*search_ctx).fix_path.data,
                if add_sep2 {
                    c"/".as_ptr()
                } else {
                    c"".as_ptr()
                },
            ) as usize;
        }
    } else {
        let p = path_tail((*search_ctx).fix_path.data);
        let mut len = (*search_ctx).fix_path.size as c_int;

        if p > (*search_ctx).fix_path.data {
            len = (p as usize - (*search_ctx).fix_path.data as usize) as c_int - 1;

            // Do not add '..' to the path
            if (len >= 2 && libc::strncmp((*search_ctx).fix_path.data, c"..".as_ptr(), 2) == 0)
                && (len == 2 || *(*search_ctx).fix_path.data.add(2) == b'/' as c_char)
            {
                xfree(buf.cast());
                xfree(ff_expand_buffer.cast());
                rs_vim_findfile_cleanup(search_ctx.cast());
                return ptr::null_mut();
            }

            let fix_end = (*search_ctx).fix_path.data.add((*search_ctx).fix_path.size);
            let add_sep2 = after_pathsep((*search_ctx).fix_path.data, fix_end) == 0;
            ff_expand_size += vim_snprintf(
                ff_expand_buffer.add(ff_expand_size),
                MAXPATHL - ff_expand_size,
                c"%.*s%s".as_ptr(),
                len,
                (*search_ctx).fix_path.data,
                if add_sep2 {
                    c"/".as_ptr()
                } else {
                    c"".as_ptr()
                },
            ) as usize;
        }

        // Prepend remaining fix_path to wc_path
        if !(*search_ctx).wc_path.data.is_null() {
            let remaining = (*search_ctx).fix_path.size - len as usize;
            let tempsize = remaining + (*search_ctx).wc_path.size + 1;
            let temp = xmalloc(tempsize).cast::<c_char>();
            let new_wc_size = vim_snprintf(
                temp,
                tempsize,
                c"%s%s".as_ptr(),
                (*search_ctx).fix_path.data.add(len as usize),
                (*search_ctx).wc_path.data,
            ) as usize;
            xfree((*search_ctx).wc_path.data.cast());
            (*search_ctx).wc_path.data = temp;
            (*search_ctx).wc_path.size = new_wc_size;
        }
    }
    xfree(buf.cast());

    // Create initial stack element
    let sptr = ff_create_stack_element(
        ff_expand_buffer,
        ff_expand_size,
        (*search_ctx).wc_path.data,
        (*search_ctx).wc_path.size,
        level,
        0,
    );

    ff_push(search_ctx, sptr);
    (*search_ctx).file_to_search = cbuf_to_string(filename, filenamelen);

    xfree(ff_expand_buffer.cast());
    search_ctx.cast()
}

/// Free the list of lists of visited files and directories.
///
/// Can handle NULL search_ctx.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_findfile_free_visited(search_ctx_arg: *mut libc::c_void) {
    if search_ctx_arg.is_null() {
        return;
    }

    let search_ctx = search_ctx_arg.cast::<FfSearchCtx>();
    vim_findfile_free_visited_list(std::ptr::addr_of_mut!((*search_ctx).visited_lists_list));
    vim_findfile_free_visited_list(std::ptr::addr_of_mut!((*search_ctx).dir_visited_lists_list));
}

/// Clean up the given search context. Can handle NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_findfile_cleanup(ctx: *mut libc::c_void) {
    if ctx.is_null() {
        return;
    }

    rs_vim_findfile_free_visited(ctx);
    ff_clear(ctx.cast());
    xfree(ctx);
}

// ============================================================================
// Public API - File Finding
// ============================================================================

/// Wildcard expansion flags.
const EW_DIR: c_int = 0x0001;
const EW_ADDSLASH: c_int = 0x0004;
const EW_SILENT: c_int = 0x0020;
const EW_NOTWILD: c_int = 0x0200;

/// Find a file in a search context.
///
/// The search context was created with `vim_findfile_init()`.
/// To get all matching files, call this function until you get NULL.
///
/// If the passed search_context is NULL, NULL is returned.
/// The search algorithm is depth first.
///
/// Returns a pointer to an allocated file name or NULL if nothing found.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_findfile(search_ctx_arg: *mut libc::c_void) -> *mut c_char {
    if search_ctx_arg.is_null() {
        return ptr::null_mut();
    }

    let search_ctx = search_ctx_arg.cast::<FfSearchCtx>();

    // filepath is used as buffer and to return a found filename
    let file_path = xmalloc(MAXPATHL).cast::<c_char>();
    let mut file_path_size: usize;

    // Store the end of the start dir -- needed for upward search
    let mut path_end = if (*search_ctx).start_dir.data.is_null() {
        ptr::null_mut()
    } else {
        (*search_ctx)
            .start_dir
            .data
            .add((*search_ctx).start_dir.size)
    };

    // Upward search loop
    'upward: loop {
        // Downward search loop
        'downward: loop {
            // Check if user wants to stop
            os_breakcheck();
            if got_int != 0 {
                break 'downward;
            }

            // Get directory to work on from stack
            let stackp = ff_pop(search_ctx);
            if stackp.is_null() {
                break 'downward;
            }

            // Check if we already searched this directory
            if (*stackp).filearray.is_null()
                && ff_check_visited(
                    std::ptr::addr_of_mut!((*(*search_ctx).dir_visited_list).visited_list),
                    (*stackp).fix_path.data,
                    (*stackp).fix_path.size,
                    (*stackp).wc_path.data,
                    (*stackp).wc_path.size,
                ) == FAIL
            {
                ff_free_stack_element(stackp);
                continue 'downward;
            }

            // Check depth
            if (*stackp).level <= 0 {
                ff_free_stack_element(stackp);
                continue 'downward;
            }

            *file_path = 0;
            file_path_size = 0;

            // If no filearray yet, expand wildcards
            if (*stackp).filearray.is_null() {
                let mut dirptrs: [*mut c_char; 2] = [file_path, ptr::null_mut()];

                // If we have a start dir, copy it in
                if vim_isAbsName((*stackp).fix_path.data) == 0
                    && !(*search_ctx).start_dir.data.is_null()
                {
                    if (*search_ctx).start_dir.size + 1 >= MAXPATHL {
                        ff_free_stack_element(stackp);
                        xfree(file_path.cast());
                        return ptr::null_mut();
                    }
                    let start_end = (*search_ctx)
                        .start_dir
                        .data
                        .add((*search_ctx).start_dir.size);
                    let add_sep = after_pathsep((*search_ctx).start_dir.data, start_end) == 0;
                    file_path_size = vim_snprintf(
                        file_path,
                        MAXPATHL,
                        c"%s%s".as_ptr(),
                        (*search_ctx).start_dir.data,
                        if add_sep { c"/".as_ptr() } else { c"".as_ptr() },
                    ) as usize;
                    if file_path_size >= MAXPATHL {
                        ff_free_stack_element(stackp);
                        xfree(file_path.cast());
                        return ptr::null_mut();
                    }
                }

                // Append the fix part of the search path
                if file_path_size + (*stackp).fix_path.size + 1 >= MAXPATHL {
                    ff_free_stack_element(stackp);
                    xfree(file_path.cast());
                    return ptr::null_mut();
                }
                let fix_end = (*stackp).fix_path.data.add((*stackp).fix_path.size);
                let add_sep = after_pathsep((*stackp).fix_path.data, fix_end) == 0;
                file_path_size += vim_snprintf(
                    file_path.add(file_path_size),
                    MAXPATHL - file_path_size,
                    c"%s%s".as_ptr(),
                    (*stackp).fix_path.data,
                    if add_sep { c"/".as_ptr() } else { c"".as_ptr() },
                ) as usize;
                if file_path_size >= MAXPATHL {
                    ff_free_stack_element(stackp);
                    xfree(file_path.cast());
                    return ptr::null_mut();
                }

                let mut rest_of_wildcards = (*stackp).wc_path;

                if *rest_of_wildcards.data != 0 {
                    if libc::strncmp(rest_of_wildcards.data, c"**".as_ptr(), 2) == 0 {
                        // Pointer to the restrict byte
                        let p = rest_of_wildcards.data.add(2);

                        if *p > 0 {
                            *p -= 1;
                            if file_path_size + 1 >= MAXPATHL {
                                ff_free_stack_element(stackp);
                                xfree(file_path.cast());
                                return ptr::null_mut();
                            }
                            *file_path.add(file_path_size) = b'*' as c_char;
                            file_path_size += 1;
                        }

                        if *p == 0 {
                            // Remove **<numb> from wildcards
                            libc::memmove(
                                rest_of_wildcards.data.cast(),
                                rest_of_wildcards.data.add(3).cast(),
                                (rest_of_wildcards.size - 3) + 1,
                            );
                            rest_of_wildcards.size -= 3;
                            (*stackp).wc_path.size = rest_of_wildcards.size;
                        } else {
                            rest_of_wildcards.data = rest_of_wildcards.data.add(3);
                            rest_of_wildcards.size -= 3;
                        }

                        if (*stackp).star_star_empty == 0 {
                            // Expand ** to empty
                            (*stackp).star_star_empty = 1;
                            dirptrs[1] = (*stackp).fix_path.data;
                        }
                    }

                    // Copy until next path separator or end
                    while *rest_of_wildcards.data != 0
                        && vim_ispathsep(*rest_of_wildcards.data as c_int) == 0
                    {
                        if file_path_size + 1 >= MAXPATHL {
                            ff_free_stack_element(stackp);
                            xfree(file_path.cast());
                            return ptr::null_mut();
                        }
                        *file_path.add(file_path_size) = *rest_of_wildcards.data;
                        file_path_size += 1;
                        rest_of_wildcards.data = rest_of_wildcards.data.add(1);
                        rest_of_wildcards.size -= 1;
                    }

                    *file_path.add(file_path_size) = 0;
                    if vim_ispathsep(*rest_of_wildcards.data as c_int) != 0 {
                        rest_of_wildcards.data = rest_of_wildcards.data.add(1);
                        rest_of_wildcards.size -= 1;
                    }
                }

                // Expand wildcards
                if path_with_url(dirptrs[0]) != 0 {
                    (*stackp).filearray = xmalloc(std::mem::size_of::<*mut c_char>()).cast();
                    *(*stackp).filearray = xmemdupz(dirptrs[0].cast(), file_path_size);
                    (*stackp).filearray_size = 1;
                } else {
                    expand_wildcards(
                        if dirptrs[1].is_null() { 1 } else { 2 },
                        dirptrs.as_mut_ptr(),
                        std::ptr::addr_of_mut!((*stackp).filearray_size),
                        std::ptr::addr_of_mut!((*stackp).filearray),
                        EW_DIR | EW_ADDSLASH | EW_SILENT | EW_NOTWILD,
                    );
                }

                (*stackp).filearray_cur = 0;
                (*stackp).stage = 0;

                // Update rest_of_wildcards for stage 0 check
                // We need to recalculate it based on current wc_path state
            }

            // Get rest_of_wildcards for checking
            let rest_of_wildcards = if (*stackp).stage == 0 {
                (*stackp).wc_path
            } else {
                NvimString {
                    data: (*stackp).wc_path.data.add((*stackp).wc_path.size),
                    size: 0,
                }
            };

            if (*stackp).stage == 0 {
                // First time working on this directory
                #[allow(clippy::branches_sharing_code)]
                if *rest_of_wildcards.data == 0 || rest_of_wildcards.size == 0 {
                    // No more wildcards, check for final file
                    let mut i = (*stackp).filearray_cur;
                    while i < (*stackp).filearray_size {
                        let filearray_entry = *(*stackp).filearray.add(i as usize);
                        if path_with_url(filearray_entry) == 0 && os_isdir(filearray_entry) == 0 {
                            i += 1;
                            continue;
                        }

                        // Prepare filename to check
                        let entry_len = libc::strlen(filearray_entry);
                        if entry_len + 1 + (*search_ctx).file_to_search.size >= MAXPATHL {
                            ff_free_stack_element(stackp);
                            xfree(file_path.cast());
                            return ptr::null_mut();
                        }

                        let add_sep =
                            after_pathsep(filearray_entry, filearray_entry.add(entry_len)) == 0;
                        file_path_size = vim_snprintf(
                            file_path,
                            MAXPATHL,
                            c"%s%s%s".as_ptr(),
                            filearray_entry,
                            if add_sep { c"/".as_ptr() } else { c"".as_ptr() },
                            (*search_ctx).file_to_search.data,
                        ) as usize;
                        if file_path_size >= MAXPATHL {
                            ff_free_stack_element(stackp);
                            xfree(file_path.cast());
                            return ptr::null_mut();
                        }

                        // Try without extra suffix and then with suffixes from 'suffixesadd'
                        let base_len = file_path_size;
                        let mut suf = if (*search_ctx).tagfile != 0 {
                            c"".as_ptr().cast_mut()
                        } else {
                            nvim_get_curbuf_sua().cast_mut()
                        };

                        loop {
                            // Check if file exists and we didn't already find it
                            let exists = path_with_url(file_path) != 0
                                || (os_path_exists(file_path) != 0
                                    && ((*search_ctx).find_what == FINDFILE_BOTH
                                        || (((*search_ctx).find_what == FINDFILE_DIR)
                                            == (os_isdir(file_path) != 0))));

                            if exists
                                && ff_check_visited(
                                    std::ptr::addr_of_mut!(
                                        (*(*search_ctx).visited_list).visited_list
                                    ),
                                    file_path,
                                    file_path_size,
                                    c"".as_ptr(),
                                    0,
                                ) == OK
                            {
                                // Push dir to examine rest of subdirs later
                                (*stackp).filearray_cur = i + 1;
                                ff_push(search_ctx, stackp);

                                if path_with_url(file_path) == 0 {
                                    file_path_size = simplify_filename(file_path);
                                }

                                // Try to shorten the path
                                let dirname_buf = xmalloc(MAXPATHL).cast::<c_char>();
                                if os_dirname(dirname_buf, MAXPATHL) == OK {
                                    let shortened = path_shorten_fname(file_path, dirname_buf);
                                    if !shortened.is_null() {
                                        let short_len = file_path_size
                                            - (shortened as usize - file_path as usize);
                                        libc::memmove(
                                            file_path.cast(),
                                            shortened.cast(),
                                            short_len + 1,
                                        );
                                        let _ = short_len; // Used for memmove above
                                    }
                                }
                                xfree(dirname_buf.cast());

                                return file_path;
                            }

                            // Not found or found already, try next suffix
                            if *suf == 0 {
                                break;
                            }
                            file_path_size = base_len
                                + copy_option_part(
                                    std::ptr::addr_of_mut!(suf),
                                    file_path.add(base_len),
                                    MAXPATHL - base_len,
                                    c",".as_ptr(),
                                );
                        }

                        i += 1;
                    }
                } else {
                    // Still wildcards left, push directories for further search
                    let mut i = (*stackp).filearray_cur;
                    while i < (*stackp).filearray_size {
                        let filearray_entry = *(*stackp).filearray.add(i as usize);
                        if os_isdir(filearray_entry) == 0 {
                            i += 1;
                            continue;
                        }
                        let entry_len = libc::strlen(filearray_entry);
                        ff_push(
                            search_ctx,
                            ff_create_stack_element(
                                filearray_entry,
                                entry_len,
                                rest_of_wildcards.data,
                                rest_of_wildcards.size,
                                (*stackp).level - 1,
                                0,
                            ),
                        );
                        i += 1;
                    }
                }
                (*stackp).filearray_cur = 0;
                (*stackp).stage = 1;
            }

            // If wildcards contains ** we have to descend till leaves
            if libc::strncmp((*stackp).wc_path.data, c"**".as_ptr(), 2) == 0 {
                let mut i = (*stackp).filearray_cur;
                while i < (*stackp).filearray_size {
                    let filearray_entry = *(*stackp).filearray.add(i as usize);
                    if path_fnamecmp(filearray_entry, (*stackp).fix_path.data) == 0 {
                        i += 1;
                        continue; // Don't repush same directory
                    }
                    if os_isdir(filearray_entry) == 0 {
                        i += 1;
                        continue;
                    }
                    let entry_len = libc::strlen(filearray_entry);
                    ff_push(
                        search_ctx,
                        ff_create_stack_element(
                            filearray_entry,
                            entry_len,
                            (*stackp).wc_path.data,
                            (*stackp).wc_path.size,
                            (*stackp).level - 1,
                            1,
                        ),
                    );
                    i += 1;
                }
            }

            // Done with this directory
            ff_free_stack_element(stackp);
        }

        // If we reached here, we didn't find anything downwards.
        // Check if we should do an upward search.
        if !(*search_ctx).start_dir.data.is_null()
            && !(*search_ctx).stopdirs_v.is_null()
            && got_int == 0
        {
            // path_end may point to NUL or previous path separator
            let plen = (path_end as usize - (*search_ctx).start_dir.data as usize)
                + usize::from(*path_end != 0);

            // Is the last starting directory in the stop list?
            if ff_path_in_stoplist((*search_ctx).start_dir.data, plen, (*search_ctx).stopdirs_v) {
                break 'upward;
            }

            // Cut off last dir
            while path_end > (*search_ctx).start_dir.data && vim_ispathsep(*path_end as c_int) != 0
            {
                path_end = path_end.sub(1);
            }
            while path_end > (*search_ctx).start_dir.data
                && vim_ispathsep(*path_end.sub(1) as c_int) == 0
            {
                path_end = path_end.sub(1);
            }
            *path_end = 0;

            // Update start_dir length
            (*search_ctx).start_dir.size =
                (path_end as usize) - ((*search_ctx).start_dir.data as usize);
            path_end = path_end.sub(1);

            if *(*search_ctx).start_dir.data == 0 {
                break 'upward;
            }

            if (*search_ctx).start_dir.size + 1 + (*search_ctx).fix_path.size >= MAXPATHL {
                xfree(file_path.cast());
                return ptr::null_mut();
            }

            let start_end = (*search_ctx)
                .start_dir
                .data
                .add((*search_ctx).start_dir.size);
            let add_sep = after_pathsep((*search_ctx).start_dir.data, start_end) == 0;
            file_path_size = vim_snprintf(
                file_path,
                MAXPATHL,
                c"%s%s%s".as_ptr(),
                (*search_ctx).start_dir.data,
                if add_sep { c"/".as_ptr() } else { c"".as_ptr() },
                (*search_ctx).fix_path.data,
            ) as usize;
            if file_path_size >= MAXPATHL {
                xfree(file_path.cast());
                return ptr::null_mut();
            }

            // Create a new stack entry
            let sptr = ff_create_stack_element(
                file_path,
                file_path_size,
                (*search_ctx).wc_path.data,
                (*search_ctx).wc_path.size,
                (*search_ctx).level,
                0,
            );
            ff_push(search_ctx, sptr);
        } else {
            break 'upward;
        }
    }

    xfree(file_path.cast());
    ptr::null_mut()
}

/// Return the stopdir string. Check that ';' is not escaped.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_findfile_stopdir(buf: *mut c_char) -> *mut c_char {
    if buf.is_null() {
        return ptr::null_mut();
    }

    let mut p = buf;

    // Find first unescaped ';' or NUL
    while *p != 0 && *p != b';' as c_char && !(*p == b'\\' as c_char && *p.add(1) == b';' as c_char)
    {
        p = p.add(1);
    }

    let mut dst = p;
    if *p == b';' as c_char {
        *p = 0;
        return p.add(1);
    }
    if *p == 0 {
        return ptr::null_mut();
    }

    // Handle escaped semicolons
    loop {
        if *p == 0 || *p == b';' as c_char {
            break;
        }
        if *p == b'\\' as c_char && *p.add(1) == b';' as c_char {
            // Overwrite the escape char
            *dst = b';' as c_char;
            dst = dst.add(1);
            p = p.add(2);
        } else {
            *dst = *p;
            dst = dst.add(1);
            p = p.add(1);
        }
    }

    *dst = 0;
    if *p == b';' as c_char {
        p.add(1)
    } else {
        ptr::null_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(FINDFILE_FILE, 0);
        assert_eq!(FINDFILE_DIR, 1);
        assert_eq!(FINDFILE_BOTH, 2);
        assert_eq!(FF_MAX_STAR_STAR_EXPAND, 30);
    }
}
