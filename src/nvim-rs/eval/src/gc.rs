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

// =============================================================================
// Stack node types for GC iteration (inlined from eval_shim.c Phase 5)
// =============================================================================

/// Linked-list node for explicit hashtab GC stack (mirrors ht_stack_T).
#[repr(C)]
struct HtStackNode {
    ht: HtHandle,
    prev: *mut HtStackNode,
}

/// Linked-list node for explicit list GC stack (mirrors list_stack_T).
#[repr(C)]
struct ListStackNode {
    list: ListHandle,
    prev: *mut ListStackNode,
}

extern "C" {
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn xrealloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    static mut exestack: nvim_collections::garray::GArray;
}

/// Push an ht onto the stack (inlined from nvim_eval_ht_stack_push).
#[inline]
unsafe fn ht_stack_push(stack: *mut *mut c_void, ht: HtHandle) {
    let node = xmalloc(std::mem::size_of::<HtStackNode>()).cast::<HtStackNode>();
    (*node).ht = ht;
    (*node).prev = (*stack).cast::<HtStackNode>();
    *stack = node.cast::<c_void>();
}

/// Pop an ht from the stack (inlined from nvim_eval_ht_stack_pop).
#[inline]
unsafe fn ht_stack_pop(stack: *mut *mut c_void) -> HtHandle {
    let node = (*stack).cast::<HtStackNode>();
    let ht = (*node).ht;
    *stack = (*node).prev.cast::<c_void>();
    xfree(node.cast::<c_void>());
    ht
}

/// Push a list onto the stack (inlined from nvim_eval_list_stack_push).
#[inline]
unsafe fn list_stack_push(stack: *mut *mut c_void, list: ListHandle) {
    let node = xmalloc(std::mem::size_of::<ListStackNode>()).cast::<ListStackNode>();
    (*node).list = list;
    (*node).prev = (*stack).cast::<ListStackNode>();
    *stack = node.cast::<c_void>();
}

/// Shrink the exestack garray if it grew too large (inlined from nvim_gc_shrink_exestack).
#[inline]
unsafe fn gc_shrink_exestack() {
    let maxlen = exestack.ga_maxlen;
    let len = exestack.ga_len;
    if maxlen - len > 500 {
        let mut n = len / 2;
        if n < exestack.ga_growsize {
            n = exestack.ga_growsize;
        }
        if len + n < maxlen {
            let new_len = (exestack.ga_itemsize as usize) * ((len + n) as usize);
            let pp = xrealloc(exestack.ga_data, new_len);
            exestack.ga_maxlen = len + n;
            exestack.ga_data = pp;
        }
    }
}

/// Pop a list from the stack (inlined from nvim_eval_list_stack_pop).
#[inline]
unsafe fn list_stack_pop(stack: *mut *mut c_void) -> ListHandle {
    let node = (*stack).cast::<ListStackNode>();
    let list = (*node).list;
    *stack = (*node).prev.cast::<c_void>();
    xfree(node.cast::<c_void>());
    list
}

use super::typval::{
    dict_get_ht, list_get_copyid, list_set_copyid, CallbackReaderT, DictTHead, PartialT, TypvalT,
};

/// Opaque handles for C types
type TvHandle = *const c_void;
type TvHandleMut = *mut c_void;
type DictHandle = *mut c_void;
type ListHandle = *mut c_void;
type HtHandle = *mut c_void;
type CallbackReaderHandle = *mut c_void;

// C VarType enum values (verified by _Static_assert in eval.c)
const VAR_DICT: c_int = 5;
const VAR_LIST: c_int = 4;
const VAR_FUNC: c_int = 3;
const VAR_PARTIAL: c_int = 9;

// C CallbackType enum values
const K_CALLBACK_PARTIAL: c_int = 2;

use super::typval::CallbackT;
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
    static mut want_garbage_collect: bool;
    static mut may_garbage_collect: bool;
    static mut garbage_collect_at_exit: bool;
    fn nvim_gc_verb_msg_abort();

    // C mark/collect functions called by garbage_collect
    fn set_ref_in_previous_funccal(copy_id: c_int) -> bool;
    fn garbage_collect_scriptvars(copy_id: c_int) -> bool;
    fn set_ref_in_insexpand_funcs(copy_id: c_int) -> bool;
    fn set_ref_in_opfunc(copy_id: c_int) -> bool;
    fn rs_set_ref_in_tagfunc(copy_id: c_int) -> bool;
    fn nvim_docmd_set_ref_in_findfunc_impl(copy_id: c_int) -> bool;
    fn garbage_collect_globvars(copy_id: c_int) -> c_int;
    fn set_ref_in_call_stack(copy_id: c_int) -> bool;
    fn set_ref_in_functions(copy_id: c_int) -> bool;
    fn set_ref_in_func_args(copy_id: c_int) -> bool;
    fn garbage_collect_vimvars(copy_id: c_int) -> bool;
    #[link_name = "rs_set_ref_in_quickfix"]
    fn set_ref_in_quickfix(copy_id: c_int) -> bool;
    fn free_unref_funccal(copy_id: c_int, testing: bool) -> bool;

    fn nvim_tv_get_vstring(tv: TvHandleMut) -> *mut std::ffi::c_char;

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
        want_garbage_collect = false;
        may_garbage_collect = false;
        garbage_collect_at_exit = false;
    }

    // Shrink the execution stack if it grew too large.
    gc_shrink_exestack();

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
    abort = abort || nvim_docmd_set_ref_in_findfunc_impl(copy_id);

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
        cur_ht = ht_stack_pop(&raw mut ht_stack);
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
        cur_l = list_stack_pop(&raw mut list_stack);
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
    if dd.is_null() || (*dd.cast::<DictTHead>()).dv_copyID == copy_id {
        return false;
    }

    // Didn't see this dict yet.
    (*dd.cast::<DictTHead>()).dv_copyID = copy_id;
    let ht = dict_get_ht(dd);
    if ht_stack.is_null() {
        return rs_set_ref_in_ht(ht, copy_id, list_stack);
    }

    ht_stack_push(ht_stack, ht);

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
    if ll.is_null() || list_get_copyid(ll) == copy_id {
        return false;
    }

    // Didn't see this list yet.
    list_set_copyid(ll, copy_id);
    if list_stack.is_null() {
        return rs_set_ref_in_list_items(ll, copy_id, ht_stack);
    }

    list_stack_push(list_stack, ll);

    false
}

/// Mark the partial `pt` with `copyID`.
unsafe fn set_ref_in_item_partial(
    pt: *mut c_void,
    copy_id: c_int,
    ht_stack: *mut *mut c_void,
    list_stack: *mut *mut c_void,
) -> bool {
    let pt_ref = &mut *pt.cast::<PartialT>();
    if pt.is_null() || pt_ref.pt_copyID == copy_id {
        return false;
    }

    // Didn't see this partial yet.
    pt_ref.pt_copyID = copy_id;

    let mut abort = set_ref_in_func(pt_ref.pt_name, pt_ref.pt_func, copy_id);

    let dict = pt_ref.pt_dict;
    if !dict.is_null() {
        abort = abort || set_ref_in_item_dict(dict, copy_id, ht_stack, list_stack);
    }

    let argc = pt_ref.pt_argc;
    for i in 0..argc {
        abort = abort
            || rs_set_ref_in_item(
                pt_ref.pt_argv.add(i as usize).cast::<c_void>().cast_const(),
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
    let tv_ref = &*tv.cast::<TypvalT>();
    let v_type = tv_ref.v_type;

    match v_type {
        VAR_DICT => set_ref_in_item_dict(tv_ref.vval.v_dict, copy_id, ht_stack, list_stack),
        VAR_LIST => set_ref_in_item_list(tv_ref.vval.v_list, copy_id, ht_stack, list_stack),
        VAR_FUNC => set_ref_in_func(
            nvim_tv_get_vstring(tv.cast_mut()),
            std::ptr::null_mut(),
            copy_id,
        ),
        VAR_PARTIAL => {
            set_ref_in_item_partial(tv_ref.vval.v_partial, copy_id, ht_stack, list_stack)
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
    let cbr = &mut *reader.cast::<CallbackReaderT>();
    let cb = std::ptr::addr_of_mut!(cbr.cb).cast::<CallbackT>();
    if rs_set_ref_in_callback(cb, copy_id, ht_stack, list_stack) {
        return true;
    }

    let self_dict = cbr.self_;
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
