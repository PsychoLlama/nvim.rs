//! Menu translation functions.
//!
//! This module provides menu name translation support via the `:menutranslate`
//! command. Translations are stored in a C-side growable array (`menutrans_ga`).

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

extern "C" {
    fn nvim_menu_eap_get_arg(eap: *mut c_void) -> *mut c_char;

    // menutrans_ga accessors
    fn nvim_menu_menutrans_ga_itemsize() -> c_int;
    fn nvim_menu_menutrans_ga_init() -> c_int;
    fn nvim_menu_menutrans_ga_len() -> c_int;
    fn nvim_menu_menutrans_get_from(idx: c_int) -> *const c_char;
    fn nvim_menu_menutrans_get_from_noamp(idx: c_int) -> *const c_char;
    fn nvim_menu_menutrans_get_to(idx: c_int) -> *const c_char;
    fn nvim_menu_menutrans_clear();
    fn nvim_menu_menutrans_append(from: *mut c_char, from_noamp: *mut c_char, to: *mut c_char);

    // String/memory functions
    fn xfree(ptr: *mut c_void);
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xmemdupz(s: *const c_char, len: usize) -> *mut c_char;

    // Utility functions
    fn skipwhite(s: *const c_char) -> *mut c_char;
    fn ends_excmd(c: c_int) -> bool;
    fn del_menutrans_vars();

    // Error reporting
    fn nvim_menu_emsg_invarg();
}

use crate::path::{
    rs_menu_skip_part, rs_menu_text, rs_menu_translate_tab_and_shift, rs_menu_unescape_name,
};

/// Handle the `:menutranslate` command.
///
/// This is the Rust implementation of C `ex_menutranslate()`.
///
/// # Safety
/// `eap` must be a valid pointer to an `exarg_T` structure.
#[export_name = "ex_menutranslate"]
pub unsafe extern "C" fn rs_ex_menutranslate(eap: *mut c_void) {
    let arg = unsafe { nvim_menu_eap_get_arg(eap) };

    // Initialize if needed
    if unsafe { nvim_menu_menutrans_ga_itemsize() } == 0 {
        unsafe { nvim_menu_menutrans_ga_init() };
    }

    // ":menutrans clear": clear all translations.
    if unsafe { libc::strncmp(arg, c"clear".as_ptr(), 5) } == 0 {
        let after = unsafe { skipwhite(arg.add(5)) };
        if unsafe { ends_excmd(*after as c_int) } {
            unsafe { nvim_menu_menutrans_clear() };
            unsafe { del_menutrans_vars() };
            return;
        }
    }

    // ":menutrans from to": add translation
    let from_start = arg;
    let arg_end = unsafe { rs_menu_skip_part(arg) } as *mut c_char;
    let to_start = unsafe { skipwhite(arg_end) };
    // NUL-terminate the "from" part
    unsafe { *arg_end = 0 };
    let to_end = unsafe { rs_menu_skip_part(to_start) } as *mut c_char;

    if to_end == to_start {
        unsafe { nvim_menu_emsg_invarg() };
        return;
    }

    let from = unsafe { xstrdup(from_start) };
    let result = unsafe { rs_menu_text(from) };
    let from_noamp = result.text;
    // Free the actext if present (we only need the text)
    if !result.actext.is_null() {
        unsafe { xfree(result.actext as *mut c_void) };
    }

    assert!(to_end >= to_start);
    let to_len = unsafe { to_end.offset_from(to_start) } as usize;
    let to = unsafe { xmemdupz(to_start, to_len) };

    unsafe { rs_menu_translate_tab_and_shift(from) };
    unsafe { rs_menu_translate_tab_and_shift(to) };
    unsafe { rs_menu_unescape_name(from) };
    unsafe { rs_menu_unescape_name(to) };

    unsafe { nvim_menu_menutrans_append(from, from_noamp, to) };
}

/// Lookup a menu name in the translation table.
///
/// Returns a pointer to the translated name, or NULL if not found.
/// First tries exact match, then tries ignoring '&' characters.
///
/// This is the Rust implementation of C `menutrans_lookup()`.
///
/// # Safety
/// `name` must be a valid pointer to a mutable NUL-terminated C string.
/// The function temporarily modifies `name[len]` and restores it.
#[export_name = "menutrans_lookup"]
pub unsafe extern "C" fn rs_menutrans_lookup(name: *mut c_char, len: c_int) -> *mut c_char {
    let count = unsafe { nvim_menu_menutrans_ga_len() };

    // Try exact match with case-insensitive comparison
    for i in 0..count {
        let from = unsafe { nvim_menu_menutrans_get_from(i) };
        if unsafe { libc::strncasecmp(name, from, len as usize) } == 0
            && unsafe { *from.add(len as usize) } == 0
        {
            return unsafe { nvim_menu_menutrans_get_to(i) } as *mut c_char;
        }
    }

    // Now try again while ignoring '&' characters.
    let c = unsafe { *name.add(len as usize) };
    unsafe { *name.add(len as usize) = 0 };
    let result = unsafe { rs_menu_text(name) };
    let dname = result.text;
    // Free the actext if present
    if !result.actext.is_null() {
        unsafe { xfree(result.actext as *mut c_void) };
    }
    unsafe { *name.add(len as usize) = c };

    for i in 0..count {
        let from_noamp = unsafe { nvim_menu_menutrans_get_from_noamp(i) };
        if unsafe { libc::strcasecmp(dname, from_noamp) } == 0 {
            unsafe { xfree(dname as *mut c_void) };
            return unsafe { nvim_menu_menutrans_get_to(i) } as *mut c_char;
        }
    }

    unsafe { xfree(dname as *mut c_void) };
    ptr::null_mut()
}
