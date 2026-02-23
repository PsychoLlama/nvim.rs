//! Garbage collection for VimL dicts and lists.
//!
//! Migrated from `free_unref_items` in `src/nvim/eval_shim.c`.
//! This implements the two-pass GC algorithm that frees unreferenced
//! VimL lists and dicts.

use std::ffi::c_int;

use nvim_typval::{DictHandle, ListHandle};

/// Bitmask used for GC copy-ID comparisons (matches C's `COPYID_MASK`).
const COPYID_MASK: c_int = !0x1;

extern "C" {
    fn nvim_gc_get_first_dict() -> DictHandle;
    fn nvim_gc_get_first_list() -> ListHandle;
    fn nvim_dict_get_copyid(d: DictHandle) -> c_int;
    fn nvim_dict_get_used_next(d: DictHandle) -> DictHandle;
    fn nvim_list_get_copyid(l: ListHandle) -> c_int;
    fn nvim_list_get_used_next(l: ListHandle) -> ListHandle;
    fn nvim_list_has_watchers(l: ListHandle) -> c_int;
    fn nvim_set_tv_in_free_unref_items(val: c_int);
    fn tv_dict_free_contents(d: DictHandle);
    fn tv_list_free_contents(l: ListHandle);
    fn tv_dict_free_dict(d: DictHandle);
    fn tv_list_free_list(l: ListHandle);
}

/// Free lists and dictionaries that are no longer referenced.
///
/// Direct replacement for the C `free_unref_items` function.
///
/// Two-pass algorithm:
/// - Pass 1: free the contents of unreferenced dicts and lists without freeing
///   the container structs, so that refcount decrements can still work.
/// - Pass 2: free the container structs themselves.
///
/// # Safety
///
/// Must only be called from `garbage_collect()`. All pointer arguments come
/// from the C GC linked lists and are guaranteed valid by the caller.
///
/// # Returns
///
/// Non-zero if anything was freed, zero otherwise (matches C bool-to-int).
#[no_mangle]
pub unsafe extern "C" fn rs_free_unref_items(copy_id: c_int) -> c_int {
    let mut did_free = false;

    // Let all "free" functions know that we are here. This means no
    // dictionaries, lists, or jobs are to be freed, because we will
    // do that here.
    // SAFETY: Global flag write is safe in single-threaded Neovim.
    unsafe { nvim_set_tv_in_free_unref_items(1) };

    // PASS 1: free the contents of the items. We don't free the items
    // themselves yet, so that it is possible to decrement refcount counters.

    // Go through the list of dicts and free items without the copyID.
    // Don't free dicts that are referenced internally.
    // SAFETY: gc_first_dict is the head of a valid C linked list.
    let mut dd = unsafe { nvim_gc_get_first_dict() };
    while !dd.is_null() {
        // SAFETY: dd is a valid non-null dict pointer.
        let copyid = unsafe { nvim_dict_get_copyid(dd) };
        if (copyid & COPYID_MASK) != (copy_id & COPYID_MASK) {
            // Free the Dictionary and ordinary items it contains, but don't
            // recurse into Lists and Dictionaries, they will be in the list
            // of dicts or list of lists.
            // SAFETY: dd is a valid non-null dict pointer.
            unsafe { tv_dict_free_contents(dd) };
            did_free = true;
        }
        // SAFETY: dd is a valid non-null dict pointer.
        dd = unsafe { nvim_dict_get_used_next(dd) };
    }

    // Go through the list of lists and free items without the copyID.
    // But don't free a list that has a watcher (used in a for loop), these
    // are not referenced anywhere.
    // SAFETY: gc_first_list is the head of a valid C linked list.
    let mut ll = unsafe { nvim_gc_get_first_list() };
    while !ll.is_null() {
        // SAFETY: ll is a valid non-null list pointer.
        let copyid = unsafe { nvim_list_get_copyid(ll) };
        let has_watchers = unsafe { nvim_list_has_watchers(ll) } != 0;
        if (copyid & COPYID_MASK) != (copy_id & COPYID_MASK) && !has_watchers {
            // Free the List and ordinary items it contains, but don't recurse
            // into Lists and Dictionaries, they will be in the list of dicts
            // or list of lists.
            // SAFETY: ll is a valid non-null list pointer.
            unsafe { tv_list_free_contents(ll) };
            did_free = true;
        }
        // SAFETY: ll is a valid non-null list pointer.
        ll = unsafe { nvim_list_get_used_next(ll) };
    }

    // PASS 2: free the items themselves.
    // Must save the next pointer BEFORE freeing, because the free function
    // removes the node from the linked list.
    // SAFETY: gc_first_dict is the head of a valid C linked list.
    let mut dd = unsafe { nvim_gc_get_first_dict() };
    while !dd.is_null() {
        // SAFETY: dd is valid; save next before potential free.
        let dd_next = unsafe { nvim_dict_get_used_next(dd) };
        // SAFETY: dd is a valid non-null dict pointer.
        let copyid = unsafe { nvim_dict_get_copyid(dd) };
        if (copyid & COPYID_MASK) != (copy_id & COPYID_MASK) {
            // SAFETY: dd is a valid non-null dict pointer.
            unsafe { tv_dict_free_dict(dd) };
        }
        dd = dd_next;
    }

    // SAFETY: gc_first_list is the head of a valid C linked list.
    let mut ll = unsafe { nvim_gc_get_first_list() };
    while !ll.is_null() {
        // SAFETY: ll is valid; save next before potential free.
        let ll_next = unsafe { nvim_list_get_used_next(ll) };
        // SAFETY: ll is a valid non-null list pointer.
        let copyid = unsafe { nvim_list_get_copyid(ll) };
        let has_watchers = unsafe { nvim_list_has_watchers(ll) } != 0;
        if (copyid & COPYID_MASK) != (copy_id & COPYID_MASK) && !has_watchers {
            // Free the List and ordinary items it contains, but don't recurse
            // into Lists and Dictionaries, they will be in the list of dicts
            // or list of lists.
            // SAFETY: ll is a valid non-null list pointer.
            unsafe { tv_list_free_list(ll) };
        }
        ll = ll_next;
    }

    // SAFETY: Global flag write.
    unsafe { nvim_set_tv_in_free_unref_items(0) };

    c_int::from(did_free)
}
