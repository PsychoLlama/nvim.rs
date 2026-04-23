//! Execution stack management
//!
//! This module handles the execution stack (exestack) which tracks the source
//! of currently executing code for error messages and debugging.

use std::ffi::{c_char, c_int, c_void};

use nvim_memory::xstrdup;

use crate::globals::{self, EstackT};
use crate::{EstackArgT, EstackHandle, EtypeT, LinenrT, ScidT};

// =============================================================================
// Opaque Handles for typval types
// =============================================================================

/// Opaque handle to ufunc_T
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct UfuncHandle(*mut c_void);

impl UfuncHandle {
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

/// Opaque handle to AutoPatCmd
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct AucmdHandle(*mut c_void);

/// Opaque handle to dict_T
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct DictHandle(*mut c_void);

/// Opaque handle to list_T
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ListHandle(*mut c_void);

// =============================================================================
// EstackHandle helpers
// =============================================================================

/// Convert a raw EstackT pointer to an EstackHandle.
#[inline]
fn handle_from_ptr(p: *mut EstackT) -> EstackHandle {
    EstackHandle(p.cast::<c_void>())
}

impl EstackHandle {
    /// Cast to a *mut EstackT for direct field access.
    #[inline]
    fn as_estackt(self) -> *mut EstackT {
        self.0.cast::<EstackT>()
    }
}

// =============================================================================
// C Accessor Extern Declarations (still needed: opaque struct internals)
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Execution stack length (from ex_docmd.c)
    fn nvim_get_exestack_len() -> c_int;

    // ufunc_T field accessors (opaque)
    fn nvim_ufunc_get_name(fp: UfuncHandle) -> *const c_char;
    fn nvim_ufunc_get_name_exp(fp: UfuncHandle) -> *const c_char;
    fn nvim_ufunc_get_script_ctx_sid(fp: UfuncHandle) -> c_int;
    fn nvim_ufunc_get_script_ctx_lnum(fp: UfuncHandle) -> LinenrT;

    // AutoPatCmd field accessors (opaque)
    fn nvim_aucmd_get_script_ctx_sid(apc: AucmdHandle) -> c_int;
    fn nvim_aucmd_get_script_ctx_lnum(apc: AucmdHandle) -> LinenrT;

    // Source context via sctx_T* in union (dereferences sctx pointer)
    fn nvim_estack_get_sctx_sid(entry: EstackHandle) -> ScidT;

    // Format a stack entry (calls vim_snprintf with PRIdLINENR)
    fn nvim_estack_format_entry(
        buf: *mut c_char,
        buflen: usize,
        type_name: *const c_char,
        name: *const c_char,
        lnum: LinenrT,
        dots: *const c_char,
    ) -> c_int;

    // Script item accessors (accesses script_items garray)
    fn nvim_script_items_get_len() -> c_int;
    fn nvim_script_item_get(id: ScidT) -> *mut c_void;
    fn nvim_scriptitem_get_name(si: *mut c_void) -> *const c_char;

    // estack_sfile: def context helpers (accesses ufunc/aucmd deeply)
    fn nvim_estack_get_def_ctx_sid(entry: EstackHandle) -> c_int;
    fn nvim_estack_get_def_script_name(entry: EstackHandle) -> *mut c_char;

    // Typval operations for stacktrace
    #[link_name = "tv_dict_alloc_lock"]
    fn tv_dict_alloc_lock_c(lock: c_int) -> DictHandle;
    #[link_name = "tv_list_alloc"]
    fn tv_list_alloc_c(count: isize) -> ListHandle;
    #[link_name = "tv_dict_add_func"]
    fn tv_dict_add_func_c(d: DictHandle, key: *const c_char, keylen: usize, fp: UfuncHandle);
    #[link_name = "tv_dict_add_str"]
    fn tv_dict_add_str_c(d: *mut c_void, key: *const c_char, keylen: usize, val: *const c_char);
    #[link_name = "tv_dict_add_nr"]
    fn tv_dict_add_nr_c(d: *mut c_void, key: *const c_char, keylen: usize, nr: i64);
    #[link_name = "tv_list_append_dict"]
    fn tv_list_append_dict_c(l: *mut c_void, d: *mut c_void);
    fn nvim_rt_list_set_ret(rettv: *mut c_void, l: ListHandle);

    // get_scriptname wrappers
    fn nvim_ufunc_get_scriptname(fp: UfuncHandle) -> *const c_char;
    fn nvim_aucmd_get_scriptname(apc: AucmdHandle) -> *const c_char;

    // Memory
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn strlen(s: *const c_char) -> usize;
}

// =============================================================================
// Direct estack_T field helpers
// =============================================================================

/// Get `es_lnum` from an entry.
#[inline]
unsafe fn estack_get_lnum(entry: *mut EstackT) -> LinenrT {
    (*entry).es_lnum
}

/// Set `es_lnum` on an entry.
#[inline]
unsafe fn estack_set_lnum(entry: *mut EstackT, lnum: LinenrT) {
    (*entry).es_lnum = lnum;
}

/// Get `es_name` from an entry.
#[inline]
unsafe fn estack_get_name(entry: *mut EstackT) -> *const c_char {
    (*entry).es_name
}

/// Set `es_name` on an entry.
#[inline]
unsafe fn estack_set_name(entry: *mut EstackT, name: *mut c_char) {
    (*entry).es_name = name;
}

/// Get `es_type` from an entry.
#[inline]
unsafe fn estack_get_type(entry: *mut EstackT) -> c_int {
    (*entry).es_type
}

/// Set `es_type` on an entry.
#[inline]
unsafe fn estack_set_type(entry: *mut EstackT, etype: c_int) {
    (*entry).es_type = etype;
}

/// Initialize an entry: set type, name, lnum, es_info = NULL.
#[inline]
unsafe fn estack_set_entry(entry: *mut EstackT, etype: c_int, name: *mut c_char, lnum: LinenrT) {
    estack_set_type(entry, etype);
    estack_set_name(entry, name);
    estack_set_lnum(entry, lnum);
    (*entry).es_info = EstackT::null_info();
}

/// Get `es_info` interpreted as a `UfuncHandle`.
#[inline]
unsafe fn estack_get_info_ufunc(entry: *mut EstackT) -> UfuncHandle {
    UfuncHandle((*entry).es_info)
}

/// Set `es_info` to a `UfuncHandle`.
#[inline]
unsafe fn estack_set_info_ufunc(entry: *mut EstackT, ufunc: UfuncHandle) {
    (*entry).es_info = ufunc.0;
}

/// Get `es_info` interpreted as an `AucmdHandle`.
#[inline]
unsafe fn estack_get_info_aucmd(entry: *mut EstackT) -> AucmdHandle {
    AucmdHandle((*entry).es_info)
}

// =============================================================================
// Phase 1: Execution Stack Functions
// =============================================================================

/// Initialize the execution stack.
///
/// Grows the exestack garray and pushes an initial ETYPE_TOP entry.
#[export_name = "estack_init"]
pub unsafe extern "C" fn rs_estack_init() {
    globals::exestack_ga_grow(10);
    let entry = globals::exestack_get_next_slot();
    estack_set_entry(entry, EtypeT::Top as c_int, std::ptr::null_mut(), 0);
    globals::exestack_inc_len();
}

/// Add an item to the execution stack.
///
/// Returns a handle to the new entry.
#[export_name = "estack_push"]
pub unsafe extern "C" fn rs_estack_push(
    etype: c_int,
    name: *mut c_char,
    lnum: LinenrT,
) -> EstackHandle {
    globals::exestack_ga_grow(1);
    let entry = globals::exestack_get_next_slot();
    estack_set_entry(entry, etype, name, lnum);
    globals::exestack_inc_len();
    handle_from_ptr(entry)
}

/// Add a user function to the execution stack.
#[export_name = "estack_push_ufunc"]
pub unsafe extern "C" fn rs_estack_push_ufunc(ufunc: UfuncHandle, lnum: LinenrT) {
    // Pick uf_name_exp if available, otherwise uf_name
    let name_exp = nvim_ufunc_get_name_exp(ufunc);
    let name = if name_exp.is_null() {
        nvim_ufunc_get_name(ufunc)
    } else {
        name_exp
    };
    let entry_handle = rs_estack_push(EtypeT::Ufunc as c_int, name.cast_mut(), lnum);
    if !entry_handle.is_null() {
        estack_set_info_ufunc(entry_handle.as_estackt(), ufunc);
    }
}

/// Take an item off of the execution stack.
#[export_name = "estack_pop"]
pub unsafe extern "C" fn rs_estack_pop() {
    globals::exestack_dec_len();
}

/// Get the current value for <sfile> in allocated memory.
///
/// `which`: ESTACK_SFILE for <sfile>, ESTACK_STACK for <stack>,
///          ESTACK_SCRIPT for <script>.
#[export_name = "estack_sfile"]
pub unsafe extern "C" fn rs_estack_sfile(which: c_int) -> *mut c_char {
    let len = nvim_get_exestack_len();
    if len <= 0 {
        return std::ptr::null_mut();
    }

    let Some(which_enum) = EstackArgT::from_int(which) else {
        return std::ptr::null_mut();
    };

    let top_entry = globals::exestack_get_entry(len - 1);

    // ESTACK_SFILE: if not in a ufunc, return the top entry's name
    if which_enum == EstackArgT::Sfile {
        let entry_type = estack_get_type(top_entry);
        if entry_type != EtypeT::Ufunc as c_int {
            let name = estack_get_name(top_entry);
            if name.is_null() {
                return std::ptr::null_mut();
            }
            return xstrdup(name);
        }
    }

    // ESTACK_SCRIPT: walk backwards to find defining script
    if which_enum == EstackArgT::Script {
        for idx in (0..len).rev() {
            let entry = globals::exestack_get_entry(idx);
            let entry_type = estack_get_type(entry);

            if entry_type == EtypeT::Ufunc as c_int || entry_type == EtypeT::Aucmd as c_int {
                let def_sid = nvim_estack_get_def_ctx_sid(handle_from_ptr(entry));
                if def_sid > 0 {
                    return nvim_estack_get_def_script_name(handle_from_ptr(entry));
                }
                return std::ptr::null_mut();
            } else if entry_type == EtypeT::Script as c_int {
                let name = estack_get_name(entry);
                if name.is_null() {
                    return std::ptr::null_mut();
                }
                return xstrdup(name);
            }
        }
        return std::ptr::null_mut();
    }

    // ESTACK_SFILE (ufunc case) or ESTACK_STACK: build full stack string
    // Allocate a growing buffer
    let mut buf_size: usize = 256;
    let mut buf = xmalloc(buf_size).cast::<c_char>();
    let mut buf_len: usize = 0;

    let mut last_type = EtypeT::Script as c_int;

    for idx in 0..len {
        let entry = globals::exestack_get_entry(idx);
        let name = estack_get_name(entry);
        if name.is_null() {
            continue;
        }

        let entry_type = estack_get_type(entry);
        let name_len = strlen(name);

        // Determine type prefix
        let type_name: &[u8] = if entry_type == last_type {
            b"\0"
        } else {
            last_type = entry_type;
            if entry_type == EtypeT::Script as c_int {
                b"script \0"
            } else if entry_type == EtypeT::Ufunc as c_int {
                b"function \0"
            } else {
                b"\0"
            }
        };

        // Calculate needed space: type_name + name + "[" + lnum_digits + "]" + ".." + nul
        let needed = strlen(type_name.as_ptr().cast::<c_char>()) + name_len + 25;

        // Grow buffer if needed
        if buf_len + needed > buf_size {
            buf_size = (buf_len + needed) * 2;
            let new_buf = xmalloc(buf_size).cast::<c_char>();
            std::ptr::copy_nonoverlapping(buf, new_buf, buf_len);
            xfree(buf.cast::<c_void>());
            buf = new_buf;
        }

        // Determine line number
        let lnum = if idx == len - 1 {
            if which_enum == EstackArgT::Stack {
                globals::get_sourcing_lnum_direct()
            } else {
                0
            }
        } else {
            estack_get_lnum(entry)
        };

        let dots: &[u8] = if idx == len - 1 { b"\0" } else { b"..\0" };

        let written = nvim_estack_format_entry(
            buf.add(buf_len),
            buf_size - buf_len,
            type_name.as_ptr().cast::<c_char>(),
            name,
            lnum,
            dots.as_ptr().cast::<c_char>(),
        );
        buf_len += written as usize;
    }

    // Null-terminate
    if buf_len < buf_size {
        *buf.add(buf_len) = 0;
    }

    buf
}

// =============================================================================
// Phase 1: Stacktrace Functions
// =============================================================================

/// Push a single item onto the stacktrace list.
unsafe fn stacktrace_push_item(
    l: ListHandle,
    fp: UfuncHandle,
    event: *const c_char,
    lnum: LinenrT,
    filepath: *const c_char,
) {
    // VAR_FIXED = 2
    let d = tv_dict_alloc_lock_c(2);

    if !fp.is_null() {
        tv_dict_add_func_c(d, b"funcref\0".as_ptr().cast(), 7, fp);
    }
    if !event.is_null() {
        tv_dict_add_str_c(d.0, b"event\0".as_ptr().cast(), 5, event);
    }
    tv_dict_add_nr_c(d.0, b"lnum\0".as_ptr().cast(), 4, i64::from(lnum));
    tv_dict_add_str_c(d.0, b"filepath\0".as_ptr().cast(), 8, filepath);

    tv_list_append_dict_c(l.0, d.0);
}

/// Create the stacktrace from the execution stack.
///
/// Returns an opaque list_T handle.
#[export_name = "stacktrace_create"]
pub unsafe extern "C" fn rs_stacktrace_create() -> ListHandle {
    let len = nvim_get_exestack_len();
    let l = tv_list_alloc_c(len as isize);

    for i in 0..len {
        let entry = globals::exestack_get_entry(i);
        let entry_type = estack_get_type(entry);
        let mut lnum = estack_get_lnum(entry);

        if entry_type == EtypeT::Script as c_int {
            let name = estack_get_name(entry);
            stacktrace_push_item(l, UfuncHandle::null(), std::ptr::null(), lnum, name);
        } else if entry_type == EtypeT::Ufunc as c_int {
            let fp = estack_get_info_ufunc(entry);
            let filepath = nvim_ufunc_get_scriptname(fp);
            lnum += nvim_ufunc_get_script_ctx_lnum(fp);
            stacktrace_push_item(l, fp, std::ptr::null(), lnum, filepath);
        } else if entry_type == EtypeT::Aucmd as c_int {
            let apc = estack_get_info_aucmd(entry);
            let filepath = nvim_aucmd_get_scriptname(apc);
            lnum += nvim_aucmd_get_script_ctx_lnum(apc);
            let name = estack_get_name(entry);
            stacktrace_push_item(l, UfuncHandle::null(), name, lnum, filepath);
        }
    }

    l
}

/// VimL getstacktrace() builtin.
#[no_mangle]
pub unsafe extern "C" fn rs_f_getstacktrace(
    _argvars: *mut c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let l = rs_stacktrace_create();
    nvim_rt_list_set_ret(rettv, l);
}

// =============================================================================
// Stack Entry Access (existing helper functions)
// =============================================================================

/// Get the top entry of the execution stack (most recent).
///
/// Returns null handle if the stack is empty.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_top() -> EstackHandle {
    let len = nvim_get_exestack_len();
    if len <= 0 {
        return EstackHandle::null();
    }
    handle_from_ptr(globals::exestack_get_entry(len - 1))
}

/// Get an entry from the execution stack by index.
///
/// Index 0 is the bottom (oldest), len-1 is the top (newest).
/// Returns null handle if index is out of bounds.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_get(idx: c_int) -> EstackHandle {
    let len = nvim_get_exestack_len();
    if idx < 0 || idx >= len {
        return EstackHandle::null();
    }
    handle_from_ptr(globals::exestack_get_entry(idx))
}

/// Get the line number from an execution stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_entry_lnum(entry: EstackHandle) -> LinenrT {
    if entry.is_null() {
        return 0;
    }
    estack_get_lnum(entry.as_estackt())
}

/// Get the name from an execution stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_entry_name(entry: EstackHandle) -> *const c_char {
    if entry.is_null() {
        return std::ptr::null();
    }
    estack_get_name(entry.as_estackt())
}

/// Get the type from an execution stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_entry_type(entry: EstackHandle) -> c_int {
    if entry.is_null() {
        return EtypeT::Top as c_int;
    }
    estack_get_type(entry.as_estackt())
}

/// Get the script ID from an execution stack entry (for Script/Modeline types).
#[no_mangle]
pub unsafe extern "C" fn rs_estack_entry_sid(entry: EstackHandle) -> ScidT {
    if entry.is_null() {
        return 0;
    }
    nvim_estack_get_sctx_sid(entry)
}

// =============================================================================
// Stack Search Functions
// =============================================================================

/// Find the most recent script entry in the execution stack.
///
/// Returns the index of the entry, or -1 if not found.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_find_script() -> c_int {
    let len = nvim_get_exestack_len();

    // Search from top to bottom
    for i in (0..len).rev() {
        let entry = globals::exestack_get_entry(i);
        if estack_get_type(entry) == EtypeT::Script as c_int {
            return i;
        }
    }

    -1
}

/// Find the most recent entry with a given type in the execution stack.
///
/// Returns the index of the entry, or -1 if not found.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_find_type(etype: c_int) -> c_int {
    let len = nvim_get_exestack_len();

    // Search from top to bottom
    for i in (0..len).rev() {
        let entry = globals::exestack_get_entry(i);
        if estack_get_type(entry) == etype {
            return i;
        }
    }

    -1
}

/// Check if a given entry type is on the execution stack.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_has_type(etype: c_int) -> bool {
    rs_estack_find_type(etype) >= 0
}

// =============================================================================
// Stack Information
// =============================================================================

/// Get info about the execution stack suitable for display.
///
/// Returns the entry type at the given stack depth (0 = top).
/// Returns ETYPE_TOP if depth is out of range.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_type_at_depth(depth: c_int) -> c_int {
    let len = nvim_get_exestack_len();
    let idx = len - 1 - depth;

    if idx < 0 || idx >= len {
        return EtypeT::Top as c_int;
    }

    estack_get_type(globals::exestack_get_entry(idx))
}

/// Count how many entries of a given type are on the stack.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_count_type(etype: c_int) -> c_int {
    let len = nvim_get_exestack_len();
    let mut count = 0;

    for i in 0..len {
        let entry = globals::exestack_get_entry(i);
        if estack_get_type(entry) == etype {
            count += 1;
        }
    }

    count
}

// =============================================================================
// estack_sfile() Helper
// =============================================================================

/// Determine what to return for estack_sfile() based on the argument.
///
/// Returns the appropriate stack index, or -1 if nothing should be returned.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_sfile_index(which: c_int) -> c_int {
    let len = nvim_get_exestack_len();
    if len <= 0 {
        return -1;
    }

    match EstackArgT::from_int(which) {
        Some(EstackArgT::Sfile | EstackArgT::Script) => {
            // Return the top script entry for <sfile> or <script>
            rs_estack_find_script()
        }
        Some(EstackArgT::Stack) => {
            // Return top for <stack>
            len - 1
        }
        Some(EstackArgT::None) | None => -1,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etype_values() {
        // Ensure enum values match C definitions
        assert_eq!(EtypeT::Top as c_int, 0);
        assert_eq!(EtypeT::Script as c_int, 1);
        assert_eq!(EtypeT::Ufunc as c_int, 2);
        assert_eq!(EtypeT::Aucmd as c_int, 3);
        assert_eq!(EtypeT::Modeline as c_int, 4);
        assert_eq!(EtypeT::Except as c_int, 5);
        assert_eq!(EtypeT::Args as c_int, 6);
        assert_eq!(EtypeT::Env as c_int, 7);
        assert_eq!(EtypeT::Internal as c_int, 8);
        assert_eq!(EtypeT::Spell as c_int, 9);
    }

    #[test]
    fn test_estack_arg_values() {
        assert_eq!(EstackArgT::None as c_int, 0);
        assert_eq!(EstackArgT::Sfile as c_int, 1);
        assert_eq!(EstackArgT::Stack as c_int, 2);
        assert_eq!(EstackArgT::Script as c_int, 3);
    }
}
