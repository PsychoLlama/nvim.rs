//! The garbage collector: marking every value reachable from a root, then
//! freeing the lists and dicts nothing marked.
//!
//! `copyID` is the mark. Anything that can hold a reference has a
//! `set_ref_in_*` that stamps it and recurses. The counter advances by two
//! (`COPYID_INC`) per collection, because `set_ref_in_previous_funccal`
//! adds one to distinguish "reachable only through a previous funccal"
//! from "reachable at all".
//!
//! Three invariants hold this together and every one of them is load
//! bearing:
//!
//! 1. **Every root is visited before anything is freed.** A value that no
//!    `set_ref_in_*` walks is invisible to the mark, and freeing it is a
//!    use-after-free rather than a leak. The root list in
//!    `garbage_collect` is therefore in the C's order and nothing has been
//!    merged or hoisted out of it.
//! 2. **`abort` short-circuits.** Each root is visited as
//!    `abort = abort || …`, so once a marker has failed (out of memory in
//!    its stack), no later root is visited *and* nothing is freed. Turning
//!    the chain into "visit everything, then look at the flag" would be
//!    the same use-after-free with extra steps.
//! 3. **The stack parameters decide recursion or deferral.** A null
//!    `ht_stack`/`list_stack` means "recurse now"; a non-null one means
//!    "push and let the caller's loop get to it". `set_ref_in_ht` and
//!    `set_ref_in_list_items` are the two loops that drain them.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::mem::{offset_of, size_of};
use core::ptr::{null, null_mut};

use crate::eval::gc::{gc_first_dict, gc_first_list};
use crate::eval::typval::{
    tv_blob_copy, tv_copy, tv_dict_copy, tv_dict_free_contents, tv_dict_free_dict,
    tv_dict_watcher_node_data, tv_in_free_unref_items, tv_list_copy, tv_list_copyid,
    tv_list_free_contents, tv_list_free_list, tv_list_ref,
};
use crate::eval::userfunc::{
    free_unref_funccal, set_ref_in_call_stack, set_ref_in_func, set_ref_in_func_args,
    set_ref_in_functions, set_ref_in_previous_funccal,
};
use crate::eval::vars::{
    garbage_collect_globvars, garbage_collect_scriptvars, garbage_collect_vimvars,
};
use crate::eval::{
    COPYID_INC, COPYID_MASK, DICT_MAXNEST, NUL, e_variable_nested_too_deep_for_making_copy,
    kMTCharWise, set_ref_in_callback, set_ref_in_callback_reader, timers,
};
use crate::ex_docmd::set_ref_in_findfunc;
use crate::global_cell::GlobalCell;
use crate::hashtab::hash_removed;
use crate::insexpand::{set_ref_in_cpt_callbacks, set_ref_in_insexpand_funcs};
use crate::main::{
    aucmd_win_vec, channels, curtab, first_tabpage, firstbuf, firstwin, garbage_collect_at_exit,
    may_garbage_collect, p_verbose, want_garbage_collect,
};
use crate::mark::mark_global_iter;
use crate::mbyte::string_convert;
use crate::memory::{xfree, xmalloc, xrealloc, xstrdup};
use crate::message::{emsg, internal_error, verb_msg};
use crate::ops::set_ref_in_opfunc;
use crate::os::cshim::gettext;
use crate::quickfix::set_ref_in_quickfix;
use crate::register::op_global_reg_iter;
use crate::runtime::exestack;
use crate::tag::set_ref_in_tagfunc;
use crate::types::{
    AdditionalData, CONV_NONE, Channel, DictWatcher, FAIL, OK, OptInt, QUEUE, String_0, VAR_BLOB,
    VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_PARTIAL, VAR_SPECIAL,
    VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, buf_T, dict_T, dictitem_T, fmark_T, fmarkv_T,
    hashitem_T, hashtab_T, ht_stack_T, list_T, list_stack_T, listitem_T, partial_T, pos_T, size_t,
    tabpage_T, timer_T, typval_T, typval_vval_union, ufunc_T, vimconv_T, win_T, xfmark_T,
    yankreg_T,
};

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
    vval: typval_vval_union { v_number: 0 },
};

/// How much slack the execution stack may keep before a collection trims
/// it back.
const EXESTACK_SLACK: c_int = 500;

/// The next mark. Two apart, so `set_ref_in_previous_funccal` can use the
/// odd value in between.
pub unsafe fn get_copyID() -> c_int {
    static CURRENT_COPY_ID: GlobalCell<c_int> = GlobalCell::new(0);
    // SAFETY: a plain counter in a cell nothing else touches.
    unsafe { *CURRENT_COPY_ID.ptr() += COPYID_INC };
    CURRENT_COPY_ID.get()
}

/// The `dictitem_T` a hashtab entry's inline key belongs to; the C spells
/// it `TV_DICT_HI2DI`.
///
/// # Safety
/// `hi` must be a live entry of a dictionary's hashtab.
unsafe fn hi2di(hi: *mut hashitem_T) -> *mut dictitem_T {
    unsafe { (*hi).hi_key.sub(offset_of!(dictitem_T, di_key)) as *mut dictitem_T }
}

/// Mark, then free. Answers whether anything was freed.
///
/// # Safety
/// Called from a point where no typval is held in a Rust temporary — see
/// the module docs; anything the marking pass cannot see is freed.
pub unsafe fn garbage_collect(testing: bool) -> bool {
    unsafe {
        let mut abort = false;
        if !testing {
            // Only once per request.
            want_garbage_collect.set(false);
            may_garbage_collect.set(false);
            garbage_collect_at_exit.set(false);
        }

        trim_exestack();

        let copy_id = get_copyID();

        // 1. Mark everything reachable from a root.

        // Variables in the previous_funccal list must not be freed unless
        // they are reachable *only* through it, so this goes first.
        abort = abort || set_ref_in_previous_funccal(copy_id);
        abort = abort || garbage_collect_scriptvars(copy_id);

        let mut buf: *mut buf_T = firstbuf.get();
        while !buf.is_null() {
            // buffer-local variables
            abort = abort
                || set_ref_in_item(
                    &raw mut (*buf).b_bufvar.di_tv,
                    copy_id,
                    null_mut(),
                    null_mut(),
                );
            // buffer callback functions
            for cb in [
                &raw mut (*buf).b_prompt_callback,
                &raw mut (*buf).b_prompt_interrupt,
                &raw mut (*buf).b_cfu_cb,
                &raw mut (*buf).b_ofu_cb,
                &raw mut (*buf).b_tsrfu_cb,
                &raw mut (*buf).b_tfu_cb,
                &raw mut (*buf).b_ffu_cb,
            ] {
                abort = abort || set_ref_in_callback(cb, copy_id, null_mut(), null_mut());
            }
            if !abort && !(*buf).b_p_cpt_cb.is_null() {
                abort = abort
                    || set_ref_in_cpt_callbacks((*buf).b_p_cpt_cb, (*buf).b_p_cpt_count, copy_id);
            }
            buf = (*buf).b_next;
        }

        // 'completefunc', 'omnifunc', 'thesaurusfunc', 'operatorfunc',
        // 'tagfunc' and 'findfunc' callbacks.
        abort = abort || set_ref_in_insexpand_funcs(copy_id);
        abort = abort || set_ref_in_opfunc(copy_id);
        abort = abort || set_ref_in_tagfunc(copy_id);
        abort = abort || set_ref_in_findfunc(copy_id);

        // window-local variables, in every tab page
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                abort = abort
                    || set_ref_in_item(
                        &raw mut (*wp).w_winvar.di_tv,
                        copy_id,
                        null_mut(),
                        null_mut(),
                    );
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }

        // window-local variables in the autocommand windows
        let wins = aucmd_win_vec.ptr();
        for i in 0..(*wins).size as isize {
            let win = (*(*wins).items.offset(i)).auc_win;
            if !win.is_null() {
                abort = abort
                    || set_ref_in_item(
                        &raw mut (*win).w_winvar.di_tv,
                        copy_id,
                        null_mut(),
                        null_mut(),
                    );
            }
        }

        walk_shada_iterators();

        // tabpage-local variables
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            abort = abort
                || set_ref_in_item(
                    &raw mut (*tp).tp_winvar.di_tv,
                    copy_id,
                    null_mut(),
                    null_mut(),
                );
            tp = (*tp).tp_next as *mut tabpage_T;
        }

        abort = abort || garbage_collect_globvars(copy_id) != 0;
        // function-local variables, then named functions (closures)
        abort = abort || set_ref_in_call_stack(copy_id);
        abort = abort || set_ref_in_functions(copy_id);

        // Channels. Deliberately not `abort`ed on: upstream discards these
        // answers, and doing otherwise would change when a collection is
        // abandoned.
        let chans = channels.ptr();
        for i in 0..(*chans).set.h.n_keys {
            let data = *(*chans).values.offset(i as isize) as *mut Channel;
            set_ref_in_callback_reader(&raw mut (*data).on_data, copy_id, null_mut(), null_mut());
            set_ref_in_callback_reader(&raw mut (*data).on_stderr, copy_id, null_mut(), null_mut());
            set_ref_in_callback(&raw mut (*data).on_exit, copy_id, null_mut(), null_mut());
        }

        // Timers, likewise.
        let tmrs = timers.ptr();
        for i in 0..(*tmrs).set.h.n_keys {
            let timer = *(*tmrs).values.offset(i as isize) as *mut timer_T;
            set_ref_in_callback(&raw mut (*timer).callback, copy_id, null_mut(), null_mut());
        }

        // function call arguments, if v:testing is set
        abort = abort || set_ref_in_func_args(copy_id);
        abort = abort || garbage_collect_vimvars(copy_id);
        abort = abort || set_ref_in_quickfix(copy_id);

        // 2. Free what nothing marked — but only if every root was seen.
        if abort {
            if p_verbose.get() > 0 as OptInt {
                verb_msg(gettext(
                    c"Not enough memory to set references, garbage collection aborted!".as_ptr(),
                ));
            }
            return false;
        }
        let did_free = free_unref_items(copy_id) != 0;
        // 3. Any funccal that can go now. May call back into here.
        free_unref_funccal(copy_id, testing as c_int) || did_free
    }
}

/// Give back the execution stack's slack, keeping 150% of what is in use.
///
/// # Safety
/// Called with the stack initialised.
unsafe fn trim_exestack() {
    unsafe {
        let st = exestack.ptr();
        if (*st).ga_maxlen - (*st).ga_len <= EXESTACK_SLACK {
            return;
        }
        let n = ((*st).ga_len / 2).max((*st).ga_growsize);
        // Never grow it here.
        if (*st).ga_len + n >= (*st).ga_maxlen {
            return;
        }
        let new_len = (*st).ga_itemsize as size_t * ((*st).ga_len + n) as size_t;
        (*st).ga_data = xrealloc((*st).ga_data, new_len);
        (*st).ga_maxlen = (*st).ga_len + n;
    }
}

/// Walk the register and global-mark iterators.
///
/// Upstream does this and throws every answer away: the ShaDa "additional
/// data" these carry used to hold typvals and no longer does. The walks
/// are kept because they are what would have to change if it ever holds
/// them again.
///
/// # Safety
/// Called with the register and mark tables initialised.
unsafe fn walk_shada_iterators() {
    unsafe {
        let mut reg_iter: *const c_void = null();
        loop {
            let mut reg = yankreg_T {
                y_array: null_mut::<String_0>(),
                y_size: 0,
                y_type: kMTCharWise,
                y_width: 0,
                timestamp: 0,
                additional_data: null_mut::<AdditionalData>(),
            };
            let mut name: c_char = NUL as c_char;
            let mut is_unnamed = false;
            reg_iter =
                op_global_reg_iter(reg_iter, &raw mut name, &raw mut reg, &raw mut is_unnamed);
            if reg_iter.is_null() {
                break;
            }
        }

        let mut mark_iter: *const c_void = null();
        loop {
            let mut fm = xfmark_T {
                fmark: fmark_T {
                    mark: pos_T {
                        lnum: 0,
                        col: 0,
                        coladd: 0,
                    },
                    fnum: 0,
                    timestamp: 0,
                    view: fmarkv_T {
                        topline_offset: 0,
                        skipcol: 0,
                    },
                    additional_data: null_mut::<AdditionalData>(),
                },
                fname: null_mut::<c_char>(),
            };
            let mut name: c_char = NUL as c_char;
            mark_iter = mark_global_iter(mark_iter, &raw mut name, &raw mut fm);
            if mark_iter.is_null() {
                break;
            }
        }
    }
}

/// Free every list and dict whose mark is not `copy_id`.
///
/// Contents go first and the structures second, in two passes each: a
/// dictionary's contents may hold the last reference to another one, so
/// nothing may be *unlinked* until every unreachable value has been
/// emptied.
///
/// # Safety
/// Called only from `garbage_collect`, after a complete marking pass.
pub(crate) unsafe fn free_unref_items(copy_id: c_int) -> c_int {
    unsafe {
        /// Is this mark stale? The low bit is the previous-funccal flag and
        /// is not part of the comparison.
        fn stale(mark: c_int, copy_id: c_int) -> bool {
            mark & COPYID_MASK != copy_id & COPYID_MASK
        }

        let mut did_free = false;
        tv_in_free_unref_items.set(true);

        // Pass 1: empty the unreachable dictionaries…
        let mut dd = gc_first_dict.get();
        while !dd.is_null() {
            if stale((*dd).dv_copyID, copy_id) {
                tv_dict_free_contents(dd);
                did_free = true;
            }
            dd = (*dd).dv_used_next;
        }
        // …and the unreachable lists. A list with a watcher is left alone:
        // the watcher is a borrow the collector cannot see.
        let mut ll = gc_first_list.get();
        while !ll.is_null() {
            if stale(tv_list_copyid(ll), copy_id) && !tv_list_has_watchers(ll) {
                tv_list_free_contents(ll);
                did_free = true;
            }
            ll = (*ll).lv_used_next;
        }

        // Pass 2: unlink and free the structures themselves. The `next`
        // pointer is read before the free.
        let mut dd = gc_first_dict.get();
        while !dd.is_null() {
            let next = (*dd).dv_used_next;
            if stale((*dd).dv_copyID, copy_id) {
                tv_dict_free_dict(dd);
            }
            dd = next;
        }
        let mut ll = gc_first_list.get();
        while !ll.is_null() {
            let next = (*ll).lv_used_next;
            if stale((*ll).lv_copyID, copy_id) && !tv_list_has_watchers(ll) {
                tv_list_free_list(ll);
            }
            ll = next;
        }

        tv_in_free_unref_items.set(false);
        did_free as c_int
    }
}

/// Mark every item of a hashtab, draining the nested hashtabs it finds
/// into its own stack rather than recursing into them.
///
/// # Safety
/// `ht` must be valid; `list_stack` null or valid.
pub unsafe fn set_ref_in_ht(
    ht: *mut hashtab_T,
    copy_id: c_int,
    list_stack: *mut *mut list_stack_T,
) -> bool {
    unsafe {
        let mut abort = false;
        let mut ht_stack: *mut ht_stack_T = null_mut();
        let mut cur_ht = ht;
        loop {
            if !abort {
                // A nested hashtab is pushed onto `ht_stack`, a nested list
                // onto the caller's `list_stack`.
                let mut todo = (*cur_ht).ht_used;
                let mut hi: *mut hashitem_T = (*cur_ht).ht_array;
                while todo != 0 {
                    if !(*hi).hi_key.is_null()
                        && (*hi).hi_key != &raw const hash_removed as *mut c_char
                    {
                        todo -= 1;
                        abort = abort
                            || set_ref_in_item(
                                &raw mut (*hi2di(hi)).di_tv,
                                copy_id,
                                &raw mut ht_stack,
                                list_stack,
                            );
                    }
                    hi = hi.add(1);
                }
            }
            // The stack is drained even while aborting, so nothing leaks.
            if ht_stack.is_null() {
                break;
            }
            cur_ht = (*ht_stack).ht;
            let done = ht_stack;
            ht_stack = (*ht_stack).prev as *mut ht_stack_T;
            xfree(done as *mut c_void);
        }
        abort
    }
}

/// Mark every item of a list, draining the nested lists it finds into its
/// own stack rather than recursing into them.
///
/// # Safety
/// `l` must be null or valid; `ht_stack` null or valid.
pub unsafe fn set_ref_in_list_items(
    l: *mut list_T,
    copy_id: c_int,
    ht_stack: *mut *mut ht_stack_T,
) -> bool {
    unsafe {
        let mut abort = false;
        let mut list_stack: *mut list_stack_T = null_mut();
        let mut cur_l = l;
        loop {
            if !cur_l.is_null() {
                let mut li: *mut listitem_T = (*cur_l).lv_first;
                while !li.is_null() {
                    if abort {
                        break;
                    }
                    abort = set_ref_in_item(
                        &raw mut (*li).li_tv,
                        copy_id,
                        ht_stack,
                        &raw mut list_stack,
                    );
                    li = (*li).li_next;
                }
            }
            if list_stack.is_null() {
                break;
            }
            cur_l = (*list_stack).list;
            let done = list_stack;
            list_stack = (*list_stack).prev as *mut list_stack_T;
            xfree(done as *mut c_void);
        }
        abort
    }
}

/// Mark a dictionary. With no `ht_stack` it recurses; with one it defers,
/// pushing its hashtab for the caller's loop to drain.
///
/// # Safety
/// `dd` must be null or valid; the stacks null or valid.
pub(crate) unsafe fn set_ref_in_item_dict(
    dd: *mut dict_T,
    copy_id: c_int,
    ht_stack: *mut *mut ht_stack_T,
    list_stack: *mut *mut list_stack_T,
) -> bool {
    unsafe {
        if dd.is_null() || (*dd).dv_copyID == copy_id {
            return false;
        }
        // Not seen yet.
        (*dd).dv_copyID = copy_id;
        if ht_stack.is_null() {
            return set_ref_in_ht(&raw mut (*dd).dv_hashtab, copy_id, list_stack);
        }

        let newitem = xmalloc(size_of::<ht_stack_T>()) as *mut ht_stack_T;
        (*newitem).ht = &raw mut (*dd).dv_hashtab;
        (*newitem).prev = *ht_stack;
        *ht_stack = newitem;

        // The watchers' callbacks are marked only on this branch, which is
        // upstream's. A dictionary reached with no `ht_stack` — that is,
        // one recursed into directly — does not have them marked.
        let mut w: *mut QUEUE = (*dd).watchers.next as *mut QUEUE;
        while w != &raw mut (*dd).watchers {
            let next: *mut QUEUE = (*w).next as *mut QUEUE;
            let watcher: *mut DictWatcher = tv_dict_watcher_node_data(w);
            set_ref_in_callback(&raw mut (*watcher).callback, copy_id, ht_stack, list_stack);
            w = next;
        }
        false
    }
}

/// Mark a list. With no `list_stack` it recurses; with one it defers.
///
/// # Safety
/// `ll` must be null or valid; the stacks null or valid.
pub(crate) unsafe fn set_ref_in_item_list(
    ll: *mut list_T,
    copy_id: c_int,
    ht_stack: *mut *mut ht_stack_T,
    list_stack: *mut *mut list_stack_T,
) -> bool {
    unsafe {
        if ll.is_null() || (*ll).lv_copyID == copy_id {
            return false;
        }
        (*ll).lv_copyID = copy_id;
        if list_stack.is_null() {
            return set_ref_in_list_items(ll, copy_id, ht_stack);
        }
        let newitem = xmalloc(size_of::<list_stack_T>()) as *mut list_stack_T;
        (*newitem).list = ll;
        (*newitem).prev = *list_stack;
        *list_stack = newitem;
        false
    }
}

/// Mark a partial: its function, its bound dictionary and its bound
/// arguments.
///
/// # Safety
/// `pt` must be null or valid; the stacks null or valid.
pub(crate) unsafe fn set_ref_in_item_partial(
    pt: *mut partial_T,
    copy_id: c_int,
    ht_stack: *mut *mut ht_stack_T,
    list_stack: *mut *mut list_stack_T,
) -> bool {
    unsafe {
        if pt.is_null() || (*pt).pt_copyID == copy_id {
            return false;
        }
        (*pt).pt_copyID = copy_id;

        let mut abort = set_ref_in_func((*pt).pt_name, (*pt).pt_func, copy_id);
        if !(*pt).pt_dict.is_null() {
            // A borrowed view, not an owner: `dtv` is never cleared.
            let mut dtv = UNSET_TV;
            dtv.v_type = VAR_DICT;
            dtv.vval.v_dict = (*pt).pt_dict;
            abort = abort || set_ref_in_item(&raw mut dtv, copy_id, ht_stack, list_stack);
        }
        for i in 0..(*pt).pt_argc {
            abort = abort
                || set_ref_in_item(
                    (*pt).pt_argv.offset(i as isize),
                    copy_id,
                    ht_stack,
                    list_stack,
                );
        }
        abort
    }
}

/// Mark whatever a typval holds. The scalar types hold nothing
/// collectable and fall through.
///
/// # Safety
/// `tv` must be valid; the stacks null or valid.
pub unsafe fn set_ref_in_item(
    tv: *mut typval_T,
    copy_id: c_int,
    ht_stack: *mut *mut ht_stack_T,
    list_stack: *mut *mut list_stack_T,
) -> bool {
    unsafe {
        match (*tv).v_type {
            VAR_DICT => set_ref_in_item_dict((*tv).vval.v_dict, copy_id, ht_stack, list_stack),
            VAR_LIST => set_ref_in_item_list((*tv).vval.v_list, copy_id, ht_stack, list_stack),
            // A Funcref names a function, which may be a closure holding a
            // scope of its own.
            VAR_FUNC => set_ref_in_func((*tv).vval.v_string, null_mut::<ufunc_T>(), copy_id),
            VAR_PARTIAL => {
                set_ref_in_item_partial((*tv).vval.v_partial, copy_id, ht_stack, list_stack)
            }
            _ => false,
        }
    }
}

/// Copy a value, optionally deeply and optionally converting its strings.
///
/// `copyID` is what makes a *deep* copy of a self-referential structure
/// terminate: a container already copied under this id answers with the
/// copy it made rather than making another.
///
/// # Safety
/// `from` and `to` must be valid; `conv` null or valid.
pub unsafe fn var_item_copy(
    conv: *const vimconv_T,
    from: *mut typval_T,
    to: *mut typval_T,
    deep: bool,
    copy_id: c_int,
) -> c_int {
    static RECURSE: GlobalCell<c_int> = GlobalCell::new(0);

    unsafe {
        if RECURSE.get() >= DICT_MAXNEST {
            emsg(gettext(e_variable_nested_too_deep_for_making_copy.as_ptr()));
            return FAIL;
        }
        *RECURSE.ptr() += 1;

        let mut ret = OK;
        match (*from).v_type {
            VAR_STRING => {
                if conv.is_null() || (*conv).vc_type == CONV_NONE || (*from).vval.v_string.is_null()
                {
                    tv_copy(from, to);
                } else {
                    (*to).v_type = VAR_STRING;
                    (*to).v_lock = VAR_UNLOCKED;
                    (*to).vval.v_string = string_convert(
                        conv as *mut vimconv_T,
                        (*from).vval.v_string,
                        null_mut::<size_t>(),
                    );
                    // A conversion that failed keeps the original bytes.
                    if (*to).vval.v_string.is_null() {
                        (*to).vval.v_string = xstrdup((*from).vval.v_string);
                    }
                }
            }
            VAR_LIST => {
                (*to).v_type = VAR_LIST;
                (*to).v_lock = VAR_UNLOCKED;
                if (*from).vval.v_list.is_null() {
                    (*to).vval.v_list = null_mut::<list_T>();
                } else if copy_id != 0 && tv_list_copyid((*from).vval.v_list) == copy_id {
                    // Already copied under this id: share that copy.
                    (*to).vval.v_list = tv_list_latest_copy((*from).vval.v_list);
                    tv_list_ref((*to).vval.v_list);
                } else {
                    (*to).vval.v_list = tv_list_copy(conv, (*from).vval.v_list, deep, copy_id);
                }
                if (*to).vval.v_list.is_null() && !(*from).vval.v_list.is_null() {
                    ret = FAIL;
                }
            }
            VAR_DICT => {
                (*to).v_type = VAR_DICT;
                (*to).v_lock = VAR_UNLOCKED;
                if (*from).vval.v_dict.is_null() {
                    (*to).vval.v_dict = null_mut::<dict_T>();
                } else if copy_id != 0 && (*(*from).vval.v_dict).dv_copyID == copy_id {
                    (*to).vval.v_dict = (*(*from).vval.v_dict).dv_copydict;
                    (*(*to).vval.v_dict).dv_refcount += 1;
                } else {
                    (*to).vval.v_dict = tv_dict_copy(conv, (*from).vval.v_dict, deep, copy_id);
                }
                if (*to).vval.v_dict.is_null() && !(*from).vval.v_dict.is_null() {
                    ret = FAIL;
                }
            }
            VAR_BLOB => {
                tv_blob_copy((*from).vval.v_blob, to);
            }
            VAR_UNKNOWN => {
                internal_error(c"var_item_copy(UNKNOWN)".as_ptr());
                ret = FAIL;
            }
            // Number, Float, Funcref, partial, Boolean and Special copy by
            // value or by reference count.
            VAR_NUMBER | VAR_FLOAT | VAR_FUNC | VAR_PARTIAL | VAR_BOOL | VAR_SPECIAL => {
                tv_copy(from, to);
            }
            _ => {}
        }

        *RECURSE.ptr() -= 1;
        ret
    }
}

/// The copy this list was last given under the current `copyID`.
///
/// # Safety
/// `l` must be valid.
#[inline]
pub(crate) unsafe fn tv_list_latest_copy(l: *const list_T) -> *mut list_T {
    unsafe { (*l).lv_copylist }
}

/// Is anything watching this list? A watched list is never freed, because
/// the watcher is a borrow the mark cannot see.
///
/// # Safety
/// `l` must be null or valid.
#[inline]
pub(crate) unsafe fn tv_list_has_watchers(l: *const list_T) -> bool {
    unsafe { !l.is_null() && !(*l).lv_watch.is_null() }
}
