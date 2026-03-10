//! Directory stack for quickfix error parsing
//!
//! This module implements the directory stack used by quickfix to track directory
//! changes during `:make` output parsing. The directory stack allows errorformat
//! patterns like `%D` (entering directory) and `%X` (leaving directory) to track
//! the current working directory, enabling correct file path resolution for errors
//! in subdirectories.
//!
//! The stack is a linked list where:
//! - Each node holds a directory name (C-allocated string)
//! - The top of the stack is the current directory context
//! - Push adds a new directory (resolving relative paths against the stack)
//! - Pop removes the top and returns the new current directory

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

// =============================================================================
// FFI: External C functions we depend on
// =============================================================================

extern "C" {
    // Memory allocation (from nvim's memory.c)
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn xstrdup(str: *const c_char) -> *mut c_char;

    // Path operations (from nvim's path.c)
    fn concat_fnames(fname1: *const c_char, fname2: *const c_char, sep: bool) -> *mut c_char;
}

// Import from sibling crates
extern "C" {
    // From os crate
    fn rs_os_isdir(path: *const c_char) -> c_int;
    fn rs_os_path_exists(path: *const c_char) -> c_int;

    // From path crate
    #[link_name = "vim_isAbsName"]
    fn vim_isAbsName(name: *const c_char) -> bool;
}

// =============================================================================
// Directory Stack Node
// =============================================================================

/// A node in the directory stack.
///
/// This structure mirrors C's `dir_stack_T` and uses C memory allocation
/// to ensure compatibility with nvim's memory management.
#[repr(C)]
pub struct DirStackNode {
    /// Pointer to the next node (toward bottom of stack)
    pub next: *mut DirStackNode,
    /// Directory name (C-allocated string via xmalloc/xstrdup)
    pub dirname: *mut c_char,
}

// =============================================================================
// Core Stack Operations (work with raw stack pointer)
// =============================================================================

/// Push a directory onto the stack and return a pointer to the actual directory.
///
/// This function handles relative path resolution by searching the stack for a
/// parent directory that contains the given subdirectory. It also cleans up
/// stale directories that no longer exist.
///
/// # Arguments
///
/// * `dirbuf` - The directory name to push (may be relative or absolute)
/// * `stackptr` - Pointer to the stack head pointer (will be modified)
/// * `is_file_stack` - If true, always store dirbuf as-is (don't resolve relative paths)
///
/// # Returns
///
/// Pointer to the directory name (owned by the stack), or NULL on error.
///
/// # Safety
///
/// * `dirbuf` must be a valid, non-null C string
/// * `stackptr` must be a valid pointer to a stack head pointer
pub unsafe fn push_dir_raw(
    dirbuf: *mut c_char,
    stackptr: *mut *mut DirStackNode,
    is_file_stack: bool,
) -> *mut c_char {
    if dirbuf.is_null() || stackptr.is_null() {
        return ptr::null_mut();
    }

    // Allocate new stack node
    let ds_new = xmalloc(std::mem::size_of::<DirStackNode>()).cast::<DirStackNode>();
    if ds_new.is_null() {
        return ptr::null_mut();
    }

    // Hook the new node at the top of the stack
    (*ds_new).next = *stackptr;
    *stackptr = ds_new;

    // Determine how to store the directory name
    if vim_isAbsName(dirbuf)
        || (*stackptr).is_null()
        || (*(*stackptr)).next.is_null()
        || is_file_stack
    {
        // Store as-is: absolute path, empty stack, or file stack
        (*(*stackptr)).dirname = xstrdup(dirbuf);
    } else {
        // Relative path - search stack for parent directory
        let mut ds_search = (*(*stackptr)).next;
        (*(*stackptr)).dirname = ptr::null_mut();

        while !ds_search.is_null() {
            // Free any previous attempt
            xfree((*(*stackptr)).dirname.cast::<c_void>());

            // Try concatenating with this stack entry
            (*(*stackptr)).dirname = concat_fnames((*ds_search).dirname, dirbuf, true);

            // Check if this concatenated path is a directory
            if rs_os_isdir((*(*stackptr)).dirname) != 0 {
                break;
            }

            ds_search = (*ds_search).next;
        }

        // Clean up intermediate directories we've passed
        // (they're no longer part of the current directory path)
        while (*(*stackptr)).next != ds_search {
            let ds_ptr = (*(*stackptr)).next;
            (*(*stackptr)).next = (*ds_ptr).next;
            xfree((*ds_ptr).dirname.cast::<c_void>());
            xfree(ds_ptr.cast::<c_void>());
        }

        // If nothing found, store the original relative path
        // (it must be at the top level)
        if ds_search.is_null() {
            xfree((*(*stackptr)).dirname.cast::<c_void>());
            (*(*stackptr)).dirname = xstrdup(dirbuf);
        }
    }

    // Return the dirname, or clean up and return NULL if allocation failed
    if !(*(*stackptr)).dirname.is_null() {
        return (*(*stackptr)).dirname;
    }

    // Allocation failed - remove the node we just added
    let ds_ptr = *stackptr;
    *stackptr = (*ds_ptr).next;
    xfree(ds_ptr.cast::<c_void>());
    ptr::null_mut()
}

/// Pop a directory from the stack and return the new top directory.
///
/// # Arguments
///
/// * `stackptr` - Pointer to the stack head pointer (will be modified)
///
/// # Returns
///
/// Pointer to the new top directory name (owned by the stack), or NULL if empty.
///
/// # Safety
///
/// * `stackptr` must be a valid pointer to a stack head pointer
pub unsafe fn pop_dir_raw(stackptr: *mut *mut DirStackNode) -> *mut c_char {
    if stackptr.is_null() {
        return ptr::null_mut();
    }

    // Pop top element and free it
    if !(*stackptr).is_null() {
        let ds_ptr = *stackptr;
        *stackptr = (*ds_ptr).next;
        xfree((*ds_ptr).dirname.cast::<c_void>());
        xfree(ds_ptr.cast::<c_void>());
    }

    // Return new top element's dirname, or NULL if stack is now empty
    if (*stackptr).is_null() {
        ptr::null_mut()
    } else {
        (*(*stackptr)).dirname
    }
}

/// Clean up the entire directory stack, freeing all nodes.
///
/// # Arguments
///
/// * `stackptr` - Pointer to the stack head pointer (will be set to NULL)
///
/// # Safety
///
/// * `stackptr` must be a valid pointer to a stack head pointer
pub unsafe fn clean_dir_stack_raw(stackptr: *mut *mut DirStackNode) {
    if stackptr.is_null() {
        return;
    }

    while !(*stackptr).is_null() {
        let ds_ptr = *stackptr;
        *stackptr = (*ds_ptr).next;
        xfree((*ds_ptr).dirname.cast::<c_void>());
        xfree(ds_ptr.cast::<c_void>());
    }
}

// =============================================================================
// Path Resolution
// =============================================================================

/// Search the directory stack for a directory containing the given filename.
///
/// This function walks through the directory stack (starting from the second entry)
/// looking for a directory where the given file exists. It also cleans up
/// intermediate entries that are no longer valid.
///
/// # Arguments
///
/// * `dir_stack` - The directory stack head (from `qf_list_T.qf_dir_stack`)
/// * `filename` - The filename to search for
///
/// # Returns
///
/// Pointer to the directory name (owned by the stack) where the file was found,
/// or NULL if not found.
///
/// # Safety
///
/// * `dir_stack` may be NULL (returns NULL immediately)
/// * `filename` must be a valid, non-null C string
pub unsafe fn guess_filepath_raw(
    dir_stack: *mut DirStackNode,
    filename: *mut c_char,
) -> *const c_char {
    if dir_stack.is_null() || filename.is_null() {
        return ptr::null();
    }

    // Start searching from the second entry (skip current)
    let mut ds_ptr = (*dir_stack).next;
    let mut fullname: *mut c_char = ptr::null_mut();

    while !ds_ptr.is_null() {
        // Free previous fullname attempt
        xfree(fullname.cast::<c_void>());

        // Concatenate directory with filename
        fullname = concat_fnames((*ds_ptr).dirname, filename, true);

        // Check if this file exists
        if rs_os_path_exists(fullname) != 0 {
            break;
        }

        ds_ptr = (*ds_ptr).next;
    }

    // Free the fullname - we just return the directory, not the full path
    xfree(fullname.cast::<c_void>());

    // Clean up intermediate entries we've passed
    // (they're no longer relevant to the current path)
    while (*dir_stack).next != ds_ptr {
        let ds_tmp = (*dir_stack).next;
        (*dir_stack).next = (*ds_tmp).next;
        xfree((*ds_tmp).dirname.cast::<c_void>());
        xfree(ds_tmp.cast::<c_void>());
    }

    // Return the found directory, or NULL if not found
    if ds_ptr.is_null() {
        ptr::null()
    } else {
        (*ds_ptr).dirname
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirstack_node_size() {
        // Ensure our struct matches C's layout expectations
        // (pointer + pointer = 16 bytes on 64-bit)
        assert_eq!(
            std::mem::size_of::<DirStackNode>(),
            std::mem::size_of::<*mut c_void>() * 2
        );
    }
}
