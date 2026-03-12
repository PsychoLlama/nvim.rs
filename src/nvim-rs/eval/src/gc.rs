//! GC reference marking functions migrated from eval.c.
//!
//! All 8 functions are migrated atomically since they mutually call each other:
//! - `set_ref_in_item`: Type dispatcher for GC marking
//! - `set_ref_in_item_dict`: Mark dict refs
//! - `set_ref_in_item_list`: Mark list refs
//! - `set_ref_in_item_partial`: Mark partial refs
//! - `set_ref_in_ht`: Mark hash table refs
//! - `set_ref_in_list_items`: Mark list item refs
//! - `set_ref_in_callback`: Mark callback refs
//! - `set_ref_in_callback_reader`: Mark callback reader refs

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr,
    clippy::borrow_as_ptr
)]

use std::ffi::{c_int, c_void};

/// Opaque handles for C types
type TvHandle = *const c_void;
type TvHandleMut = *mut c_void;
type DictHandle = *mut c_void;
type ListHandle = *mut c_void;
type PartialHandle = *mut c_void;
type HtHandle = *mut c_void;
type CallbackReaderHandle = *mut c_void;

// C VarType enum values (verified by _Static_assert in eval.c)
const VAR_DICT: c_int = 5;
const VAR_LIST: c_int = 4;
const VAR_FUNC: c_int = 3;
const VAR_PARTIAL: c_int = 9;

// C CallbackType enum values
const K_CALLBACK_PARTIAL: c_int = 2;

// CallbackT: Rust mirror of C Callback struct (16 bytes, layout validated by _Static_assert).
#[repr(C)]
pub union CallbackData {
    pub funcref: *mut std::ffi::c_char,
    pub partial: *mut c_void,
    pub luaref: c_int,
}

#[repr(C)]
pub struct CallbackT {
    data: CallbackData,
    cb_type: c_int,
    // 4 bytes trailing padding
}

type CallbackHandle = *mut CallbackT;

extern "C" {
    // GC composite wrappers (Phase 13)
    fn nvim_gc_mark_buffers(copy_id: c_int, abort: bool) -> bool;
    fn nvim_gc_mark_tab_windows(copy_id: c_int, abort: bool) -> bool;
    fn nvim_gc_mark_tabs(copy_id: c_int, abort: bool) -> bool;
    fn nvim_gc_mark_channels(copy_id: c_int, abort: bool) -> bool;
    fn nvim_gc_mark_timers(copy_id: c_int, abort: bool) -> bool;
    fn nvim_gc_iterate_registers();
    fn nvim_gc_iterate_marks();
    fn nvim_gc_shrink_exestack();
    fn nvim_gc_clear_flags();
    fn nvim_gc_verb_msg_abort();

    // C mark/collect functions called by garbage_collect
    fn set_ref_in_previous_funccal(copy_id: c_int) -> bool;
    fn garbage_collect_scriptvars(copy_id: c_int) -> bool;
    fn set_ref_in_insexpand_funcs(copy_id: c_int) -> bool;
    fn set_ref_in_opfunc(copy_id: c_int) -> bool;
    fn rs_set_ref_in_tagfunc(copy_id: c_int) -> bool;
    fn set_ref_in_findfunc(copy_id: c_int) -> bool;
    fn garbage_collect_globvars(copy_id: c_int) -> c_int;
    fn set_ref_in_call_stack(copy_id: c_int) -> bool;
    fn set_ref_in_functions(copy_id: c_int) -> bool;
    fn set_ref_in_func_args(copy_id: c_int) -> bool;
    fn garbage_collect_vimvars(copy_id: c_int) -> bool;
    #[link_name = "rs_set_ref_in_quickfix"]
    fn set_ref_in_quickfix(copy_id: c_int) -> bool;
    fn free_unref_funccal(copy_id: c_int, testing: bool) -> bool;

    // typval field accessors (Phase 4 already defined some)
    fn nvim_eval_tv_get_type(tv: TvHandle) -> c_int;
    fn nvim_tv_get_vstring(tv: TvHandleMut) -> *mut std::ffi::c_char;
    fn nvim_eval_tv_get_partial(tv: TvHandle) -> PartialHandle;

    // typval_T field accessors for dict and list
    fn nvim_eval_tv_get_dict(tv: TvHandle) -> DictHandle;
    fn nvim_eval_tv_get_list(tv: TvHandle) -> ListHandle;

    // Dict accessors
    fn nvim_eval_dict_get_copyid(dd: DictHandle) -> c_int;
    fn nvim_eval_dict_set_copyid(dd: DictHandle, copyid: c_int);
    fn nvim_eval_dict_get_ht(dd: DictHandle) -> HtHandle;

    // List accessors
    fn nvim_eval_list_get_copyid(ll: ListHandle) -> c_int;
    fn nvim_eval_list_set_copyid(ll: ListHandle, copyid: c_int);

    // Partial accessors
    fn nvim_eval_partial_get_copyid(pt: PartialHandle) -> c_int;
    fn nvim_eval_partial_set_copyid(pt: PartialHandle, copyid: c_int);
    fn nvim_eval_partial_get_name(pt: PartialHandle) -> *mut std::ffi::c_char;
    fn nvim_eval_partial_get_func(pt: PartialHandle) -> *mut c_void;
    fn nvim_eval_partial_get_dict(pt: PartialHandle) -> DictHandle;
    fn nvim_eval_partial_get_argc(pt: PartialHandle) -> c_int;
    fn nvim_eval_partial_get_argv(pt: PartialHandle, idx: c_int) -> TvHandleMut;

    // Hashtab iteration: calls set_ref_in_item for each entry
    fn nvim_eval_ht_foreach_di_tv(
        ht: HtHandle,
        copyid: c_int,
        ht_stack: *mut *mut c_void,
        list_stack: *mut *mut c_void,
    ) -> bool;

    // List iteration: calls set_ref_in_item for each entry
    fn nvim_eval_list_foreach_tv(
        l: ListHandle,
        copyid: c_int,
        ht_stack: *mut *mut c_void,
        list_stack: *mut *mut c_void,
    ) -> bool;

    // Dict watcher iteration: calls set_ref_in_callback for each watcher
    fn nvim_eval_dict_foreach_watcher_callback(
        dd: DictHandle,
        copyid: c_int,
        ht_stack: *mut *mut c_void,
        list_stack: *mut *mut c_void,
    );

    // Callback reader accessors
    fn nvim_eval_cbr_get_cb(reader: CallbackReaderHandle) -> CallbackHandle;
    fn nvim_eval_cbr_get_self(reader: CallbackReaderHandle) -> DictHandle;

    // Stack operations (using C malloc/free for ht_stack/list_stack)
    fn nvim_eval_ht_stack_push(stack: *mut *mut c_void, ht: HtHandle);
    fn nvim_eval_ht_stack_pop(stack: *mut *mut c_void) -> HtHandle;
    fn nvim_eval_list_stack_push(stack: *mut *mut c_void, list: ListHandle);
    fn nvim_eval_list_stack_pop(stack: *mut *mut c_void) -> ListHandle;

    // External: set_ref_in_func (remains in C)
    fn set_ref_in_func(name: *mut std::ffi::c_char, fp: *mut c_void, copyid: c_int) -> bool;

    // Free unreferenced items (in eval_exec crate)
    fn rs_free_unref_items(copy_id: c_int) -> c_int;
}

/// Do garbage collection for lists and dicts.
///
/// Direct Rust replacement for the C `garbage_collect` function.
///
/// # Safety
///
/// Must only be called from Neovim's main thread.
#[must_use]
#[export_name = "garbage_collect"]
pub unsafe extern "C" fn rs_garbage_collect(testing: bool) -> bool {
    use super::rs_get_copyID;

    if !testing {
        nvim_gc_clear_flags();
    }

    // Shrink the execution stack if it grew too large.
    nvim_gc_shrink_exestack();

    // Advance by two (COPYID_INC) because we add one for items referenced
    // through previous_funccal.
    let copy_id = rs_get_copyID();

    // 1. Go through all accessible variables and mark all lists and dicts
    // with copyID.

    // Don't free variables in the previous_funccal list unless they are only
    // referenced through previous_funccal. This must be first, because if
    // the item is referenced elsewhere the funccal must not be freed.
    let mut abort = set_ref_in_previous_funccal(copy_id);

    // script-local variables
    abort = abort || garbage_collect_scriptvars(copy_id);

    // buffer-local variables and callbacks
    abort = nvim_gc_mark_buffers(copy_id, abort);

    // 'completefunc', 'omnifunc' and 'thesaurusfunc' callbacks
    abort = abort || set_ref_in_insexpand_funcs(copy_id);

    // 'operatorfunc' callback
    abort = abort || set_ref_in_opfunc(copy_id);

    // 'tagfunc' callback
    abort = abort || rs_set_ref_in_tagfunc(copy_id);

    // 'findfunc' callback
    abort = abort || set_ref_in_findfunc(copy_id);

    // window-local variables (all tab windows + autocmd windows)
    abort = nvim_gc_mark_tab_windows(copy_id, abort);

    // registers (ShaDa additional data) -- no marking, preserves side effects
    nvim_gc_iterate_registers();

    // global marks (ShaDa additional data) -- no marking, preserves side effects
    nvim_gc_iterate_marks();

    // tabpage-local variables
    abort = nvim_gc_mark_tabs(copy_id, abort);

    // global variables
    abort = abort || (garbage_collect_globvars(copy_id) != 0);

    // function-local variables
    abort = abort || set_ref_in_call_stack(copy_id);

    // named functions (matters for closures)
    abort = abort || set_ref_in_functions(copy_id);

    // channels
    abort = nvim_gc_mark_channels(copy_id, abort);

    // timers
    abort = nvim_gc_mark_timers(copy_id, abort);

    // function call arguments, if v:testing is set.
    abort = abort || set_ref_in_func_args(copy_id);

    // v: vars
    abort = abort || garbage_collect_vimvars(copy_id);

    abort = abort || set_ref_in_quickfix(copy_id);

    let mut did_free = false;
    if abort {
        // Use p_verbose check via the verb_msg wrapper
        nvim_gc_verb_msg_abort();
    } else {
        // 2. Free lists and dictionaries that are not referenced.
        did_free = rs_free_unref_items(copy_id) != 0;

        // 3. Check if any funccal can be freed now.
        //    This may call us back recursively.
        did_free = free_unref_funccal(copy_id, testing) || did_free;
    }

    did_free
}

/// Mark all lists and dicts referenced through hashtab `ht` with `copyID`.
///
/// # Safety
///
/// `ht` must be a valid hashtab pointer.
/// `list_stack` may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_set_ref_in_ht(
    ht: HtHandle,
    copy_id: c_int,
    list_stack: *mut *mut c_void,
) -> bool {
    let mut abort = false;
    let mut ht_stack: *mut c_void = std::ptr::null_mut();

    let mut cur_ht = ht;
    loop {
        if !abort {
            abort = nvim_eval_ht_foreach_di_tv(cur_ht, copy_id, &raw mut ht_stack, list_stack);
        }

        if ht_stack.is_null() {
            break;
        }

        // take an item from the stack
        cur_ht = nvim_eval_ht_stack_pop(&raw mut ht_stack);
    }

    abort
}

/// Mark all lists and dicts referenced through list `l` with `copyID`.
///
/// # Safety
///
/// `l` must be a valid list pointer.
/// `ht_stack` may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_set_ref_in_list_items(
    l: ListHandle,
    copy_id: c_int,
    ht_stack: *mut *mut c_void,
) -> bool {
    let mut list_stack: *mut c_void = std::ptr::null_mut();

    let mut cur_l = l;
    let mut abort;
    loop {
        abort = nvim_eval_list_foreach_tv(cur_l, copy_id, ht_stack, &raw mut list_stack);

        if list_stack.is_null() {
            break;
        }

        // take an item from the stack
        cur_l = nvim_eval_list_stack_pop(&raw mut list_stack);
    }

    abort
}

/// Mark the dict `dd` with `copyID`.
unsafe fn set_ref_in_item_dict(
    dd: DictHandle,
    copy_id: c_int,
    ht_stack: *mut *mut c_void,
    list_stack: *mut *mut c_void,
) -> bool {
    if dd.is_null() || nvim_eval_dict_get_copyid(dd) == copy_id {
        return false;
    }

    // Didn't see this dict yet.
    nvim_eval_dict_set_copyid(dd, copy_id);
    if ht_stack.is_null() {
        return rs_set_ref_in_ht(nvim_eval_dict_get_ht(dd), copy_id, list_stack);
    }

    nvim_eval_ht_stack_push(ht_stack, nvim_eval_dict_get_ht(dd));

    // Iterate over dict watchers
    nvim_eval_dict_foreach_watcher_callback(dd, copy_id, ht_stack, list_stack);

    false
}

/// Mark the list `ll` with `copyID`.
unsafe fn set_ref_in_item_list(
    ll: ListHandle,
    copy_id: c_int,
    ht_stack: *mut *mut c_void,
    list_stack: *mut *mut c_void,
) -> bool {
    if ll.is_null() || nvim_eval_list_get_copyid(ll) == copy_id {
        return false;
    }

    // Didn't see this list yet.
    nvim_eval_list_set_copyid(ll, copy_id);
    if list_stack.is_null() {
        return rs_set_ref_in_list_items(ll, copy_id, ht_stack);
    }

    nvim_eval_list_stack_push(list_stack, ll);

    false
}

/// Mark the partial `pt` with `copyID`.
unsafe fn set_ref_in_item_partial(
    pt: PartialHandle,
    copy_id: c_int,
    ht_stack: *mut *mut c_void,
    list_stack: *mut *mut c_void,
) -> bool {
    if pt.is_null() || nvim_eval_partial_get_copyid(pt) == copy_id {
        return false;
    }

    // Didn't see this partial yet.
    nvim_eval_partial_set_copyid(pt, copy_id);

    let mut abort = set_ref_in_func(
        nvim_eval_partial_get_name(pt),
        nvim_eval_partial_get_func(pt),
        copy_id,
    );

    let dict = nvim_eval_partial_get_dict(pt);
    if !dict.is_null() {
        abort = abort || set_ref_in_item_dict(dict, copy_id, ht_stack, list_stack);
    }

    let argc = nvim_eval_partial_get_argc(pt);
    for i in 0..argc {
        abort = abort
            || rs_set_ref_in_item(
                nvim_eval_partial_get_argv(pt, i),
                copy_id,
                ht_stack,
                list_stack,
            );
    }

    abort
}

/// Mark all lists and dicts referenced through typval `tv` with `copyID`.
///
/// # Safety
///
/// `tv` must be a valid typval pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_set_ref_in_item(
    tv: TvHandle,
    copy_id: c_int,
    ht_stack: *mut *mut c_void,
    list_stack: *mut *mut c_void,
) -> bool {
    let v_type = nvim_eval_tv_get_type(tv);

    match v_type {
        VAR_DICT => set_ref_in_item_dict(nvim_eval_tv_get_dict(tv), copy_id, ht_stack, list_stack),
        VAR_LIST => set_ref_in_item_list(nvim_eval_tv_get_list(tv), copy_id, ht_stack, list_stack),
        VAR_FUNC => set_ref_in_func(
            nvim_tv_get_vstring(tv.cast_mut()),
            std::ptr::null_mut(),
            copy_id,
        ),
        VAR_PARTIAL => {
            set_ref_in_item_partial(nvim_eval_tv_get_partial(tv), copy_id, ht_stack, list_stack)
        }
        _ => false,
    }
}

/// Mark callback refs with `copyID`.
///
/// # Safety
///
/// `callback` must be a valid Callback pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_set_ref_in_callback(
    callback: CallbackHandle,
    copy_id: c_int,
    ht_stack: *mut *mut c_void,
    list_stack: *mut *mut c_void,
) -> bool {
    // Direct field access: replaces nvim_eval_cb_get_type / nvim_eval_cb_get_partial
    if (*callback).cb_type == K_CALLBACK_PARTIAL {
        let partial = (*callback).data.partial;
        return set_ref_in_item_partial(partial, copy_id, ht_stack, list_stack);
    }
    false
}

/// Mark callback reader refs with `copyID`.
///
/// # Safety
///
/// `reader` must be a valid CallbackReader pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_set_ref_in_callback_reader(
    reader: CallbackReaderHandle,
    copy_id: c_int,
    ht_stack: *mut *mut c_void,
    list_stack: *mut *mut c_void,
) -> bool {
    let cb = nvim_eval_cbr_get_cb(reader);
    if rs_set_ref_in_callback(cb, copy_id, ht_stack, list_stack) {
        return true;
    }

    let self_dict = nvim_eval_cbr_get_self(reader);
    if !self_dict.is_null() {
        return set_ref_in_item_dict(self_dict, copy_id, ht_stack, list_stack);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_type_constants() {
        assert_eq!(VAR_FUNC, 3);
        assert_eq!(VAR_LIST, 4);
        assert_eq!(VAR_DICT, 5);
        assert_eq!(VAR_PARTIAL, 9);
    }

    #[test]
    fn test_callback_type_constants() {
        assert_eq!(K_CALLBACK_PARTIAL, 2);
    }
}
