//! The garbage collector: marking every value reachable from a root, then
//! freeing the lists and dicts nothing marked.
//!
//! `copy_id` is the mark. Anything that can hold a reference has a
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
#![allow(unsafe_code)]

use crate::guard::Depth;
use core::ffi::c_int;
use core::mem::{ManuallyDrop, offset_of};
use core::ptr::NonNull;

use crate::autocmd::aucmd_wins;
use crate::channel::channels;
use crate::eval::gc::{dict_at, dict_slots, list_at, list_slots};
use crate::eval::gc::{garbage_collect_at_exit, may_garbage_collect, want_garbage_collect};
use crate::eval::typval::{
    DictRef, ListRef, blob_copy, dict_copy, list_copy, list_free_contents, list_free_list, tv_copy,
    tv_dict_free_contents, tv_dict_free_dict, tv_in_free_unref_items,
};
use crate::eval::userfunc::{
    free_unref_funccal, set_ref_in_call_stack, set_ref_in_func, set_ref_in_func_args,
    set_ref_in_functions, set_ref_in_previous_funccal,
};
use crate::eval::vars::emsg_static;
use crate::eval::vars::{
    garbage_collect_globvars, garbage_collect_scriptvars, garbage_collect_vimvars,
};
use crate::eval::{
    COPYID_INC, COPYID_MASK, DICT_MAXNEST, e_variable_nested_too_deep_for_making_copy,
    set_ref_in_callback, timers,
};
use crate::ex_docmd::set_ref_in_findfunc;
use crate::global_cell::GlobalCell;
use crate::insexpand::{mark_cpt_callbacks, set_ref_in_insexpand_funcs};
use crate::mbyte::string_convert_cstr;
use crate::message::{internal_error, verb_msg};
use crate::ops::set_ref_in_opfunc;
use crate::option::vars::p_verbose;
use crate::os::cshim::gettext;
use crate::quickfix::set_ref_in_quickfix;
use crate::registry::SlotTable;
use crate::runtime::exestack;
use crate::tag::set_ref_in_tagfunc;
use crate::types::{
    CONV_NONE, Callback, CallbackReader, Channel, Failed, List, OptInt, PartialRef, Timer, TypVal,
    VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_PARTIAL,
    VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN, VimConv,
};
use crate::winlayer::{Live, buffers, tab_windows, tabs};

/// How much slack the execution stack may keep before a collection trims
/// it back.
const EXESTACK_SLACK: usize = 500;

/// The garray `growsize` the execution stack was declared with, which is the
/// floor [`trim_exestack`] will not shrink below.
const EXESTACK_GROWSIZE: usize = 50;

/// The next mark. Two apart, so `set_ref_in_previous_funccal` can use the
/// odd value in between.
pub(crate) fn get_copy_id() -> c_int {
    static CURRENT_COPY_ID: GlobalCell<c_int> = GlobalCell::new(0);
    CURRENT_COPY_ID.set(CURRENT_COPY_ID.get() + COPYID_INC);
    CURRENT_COPY_ID.get()
}

/// Mark one root's variable, with neither stack: the collector recurses
/// into whatever it holds rather than deferring it to a caller's loop.
pub(crate) fn mark_root(tv: &TypVal, copy_id: c_int) -> bool {
    set_ref_in_item(tv, copy_id, None, None)
}

/// Mark one callback, with neither stack.
fn mark_cb(cb: &Callback, copy_id: c_int) -> bool {
    set_ref_in_callback(cb, copy_id, None, None)
}

/// Mark what a callback *reader* keeps alive, with neither stack: its
/// callback, and the `self` dictionary it would be called with.
///
/// # Safety
/// `reader` must be a live reader, whose `self` dictionary is null or live.
unsafe fn mark_reader(reader: *mut CallbackReader, copy_id: c_int) -> bool {
    // SAFETY: the caller's promise -- the reader outlives the call, and its
    // `cb` is the callback it owns.
    if mark_cb(unsafe { &(*reader).cb }, copy_id) {
        return true;
    }
    // A view of the reader's dictionary, which takes no reference: the
    // reader keeps the one it has.
    // SAFETY: as above.
    let Some(self_dict) = (unsafe { DictRef::owning((*reader).self_0) }).map(ManuallyDrop::new)
    else {
        return false;
    };
    set_ref_in_item_dict(&self_dict, copy_id, None, None)
}

/// Mark, then free. Answers whether anything was freed.
pub fn garbage_collect(testing: bool) -> bool {
    let mut abort = false;
    if !testing {
        // Only once per request.
        want_garbage_collect.set(false);
        may_garbage_collect.set(false);
        garbage_collect_at_exit.set(false);
    }

    trim_exestack();

    let copy_id = get_copy_id();

    // 1. Mark everything reachable from a root.

    // Variables in the previous_funccal list must not be freed unless
    // they are reachable *only* through it, so this goes first.
    abort = abort || set_ref_in_previous_funccal(copy_id);
    abort = abort || garbage_collect_scriptvars(copy_id);

    for buf in buffers() {
        // buffer-local variables
        abort = abort || mark_root(&buf.b_bufvar.di_tv, copy_id);
        // buffer callback functions
        for cb in [
            &buf.b_prompt_callback,
            &buf.b_prompt_interrupt,
            &buf.b_cfu_cb,
            &buf.b_ofu_cb,
            &buf.b_tsrfu_cb,
            &buf.b_tfu_cb,
            &buf.b_ffu_cb,
        ] {
            abort = abort || mark_cb(cb, copy_id);
        }
        // The buffer's own 'complete' callback list.
        abort = abort || mark_cpt_callbacks(buf, copy_id);
    }

    // 'completefunc', 'omnifunc', 'thesaurusfunc', 'operatorfunc',
    // 'tagfunc' and 'findfunc' callbacks.
    abort = abort || set_ref_in_insexpand_funcs(copy_id);
    abort = abort || set_ref_in_opfunc(copy_id);
    abort = abort || set_ref_in_tagfunc(copy_id);
    abort = abort || set_ref_in_findfunc(copy_id);

    // window-local variables, in every tab page
    for wp in tab_windows() {
        abort = abort || mark_root(&wp.w_winvar.di_tv, copy_id);
    }

    // window-local variables in the autocommand windows
    let wins = aucmd_wins();
    for i in 0..wins.len() {
        if let Some(win) = wins.window(i) {
            abort = abort || mark_root(&win.w_winvar.di_tv, copy_id);
        }
    }

    // tabpage-local variables
    for tp in tabs() {
        abort = abort || mark_root(&tp.tp_winvar.di_tv, copy_id);
    }

    abort = abort || garbage_collect_globvars(copy_id) != 0;
    // function-local variables, then named functions (closures)
    abort = abort || set_ref_in_call_stack(copy_id);
    abort = abort || set_ref_in_functions(copy_id);

    // Channels. Deliberately not `abort`ed on: upstream discards these
    // answers, and doing otherwise would change when a collection is
    // abandoned.
    for data in channels.with(SlotTable::snapshot_values) {
        // SAFETY: the snapshot holds the registered live channels.
        let ch = unsafe { Live::<Channel>::new(data) };
        let on_data = ch.field_ptr(offset_of!(Channel, on_data));
        let on_stderr = ch.field_ptr(offset_of!(Channel, on_stderr));
        let on_exit: *mut Callback = ch.field_ptr(offset_of!(Channel, on_exit));
        // SAFETY: all three are the channel's own callbacks.
        unsafe { mark_reader(on_data, copy_id) };
        // SAFETY: as above.
        unsafe { mark_reader(on_stderr, copy_id) };
        // SAFETY: as above.
        mark_cb(unsafe { &*on_exit }, copy_id);
    }

    // Timers, likewise.
    for timer in timers.with(SlotTable::snapshot_values) {
        // SAFETY: the snapshot holds the registered live timers.
        let cb: *mut Callback =
            unsafe { Live::<Timer>::new(timer) }.field_ptr(offset_of!(Timer, callback));
        // SAFETY: `cb` is the timer's own callback.
        mark_cb(unsafe { &*cb }, copy_id);
    }

    // function call arguments, if v:testing is set
    abort = abort || set_ref_in_func_args(copy_id);
    abort = abort || garbage_collect_vimvars(copy_id);
    abort = abort || set_ref_in_quickfix(copy_id);

    // 2. Free what nothing marked — but only if every root was seen.
    if abort {
        if p_verbose() > 0 as OptInt {
            let msg = c"Not enough memory to set references, garbage collection aborted!";
            // SAFETY: the message is a NUL-terminated literal.
            verb_msg(gettext(msg));
        }
        return false;
    }
    let did_free = free_unref_items(copy_id) != 0;
    // 3. Any funccal that can go now. May call back into here.
    let freed_funccal = free_unref_funccal(copy_id, testing as c_int);
    freed_funccal || did_free
}

/// Give back the execution stack's slack, keeping 150% of what is in use.
///
/// Upstream reaches into the garray and reallocs by hand; a `Vec` says the
/// same thing with `shrink_to`, which is also free to keep more than it is
/// asked for. The `growsize` floor and the `EXESTACK_SLACK` threshold are
/// upstream's and are kept: this runs after every garbage-collection pass, and
/// shrinking a stack that is about to grow again is what they avoid.
fn trim_exestack() {
    exestack.with_mut(|stack| {
        let len = stack.len();
        if stack.capacity() - len <= EXESTACK_SLACK {
            return;
        }
        let keep = len + (len / 2).max(EXESTACK_GROWSIZE);
        // Never grow it here.
        if keep >= stack.capacity() {
            return;
        }
        stack.shrink_to(keep);
    });
}

/// Free every list and dict whose mark is not `copy_id`.
///
/// Contents go first and the structures second, in two passes each: a
/// dictionary's contents may hold the last reference to another one, so
/// nothing may be *unlinked* until every unreachable value has been
/// emptied.
pub(crate) fn free_unref_items(copy_id: c_int) -> c_int {
    /// Is this mark stale? The low bit is the previous-funccal flag and
    /// is not part of the comparison.
    fn stale(mark: c_int, copy_id: c_int) -> bool {
        mark & COPYID_MASK != copy_id & COPYID_MASK
    }

    let mut did_free = false;
    tv_in_free_unref_items.set(true);

    // Pass 1: empty the unreachable dictionaries…
    //
    // Both passes walk the registry by slot index rather than borrowing it:
    // `tv_in_free_unref_items` holds off every free for the duration, so no
    // slot is vacated under the walk, but emptying a container runs
    // arbitrary teardown and the registry must not be borrowed across that.
    for idx in 0..dict_slots() {
        let Some(dd) = dict_at(idx).map(NonNull::as_ptr) else {
            continue;
        };
        if stale(unsafe { (*dd).dv_copy_id }, copy_id) {
            // SAFETY: a registered dictionary; the view takes no reference.
            tv_dict_free_contents(&::core::mem::ManuallyDrop::new(
                unsafe { DictRef::owning(dd) }.expect("a live dictionary"),
            ));
            did_free = true;
        }
    }
    // …and the unreachable lists. A list with a watcher is left alone:
    // the watcher is a borrow the collector cannot see.
    for idx in 0..list_slots() {
        let Some(ll) = list_at(idx).map(NonNull::as_ptr) else {
            continue;
        };
        if stale(unsafe { (*ll).copy_id() }, copy_id) && !list_has_watchers(unsafe { ll.as_ref() })
        {
            // SAFETY: a registered list; the view takes no reference.
            list_free_contents(&::core::mem::ManuallyDrop::new(
                unsafe { ListRef::owning(ll) }.expect("a live list"),
            ));
            did_free = true;
        }
    }

    // Pass 2: take the structures themselves out of the registry and free
    // them. The walk reads each slot afresh, so a slot this pass vacates is
    // simply skipped when the index reaches it.
    for idx in 0..dict_slots() {
        let Some(dd) = dict_at(idx).map(NonNull::as_ptr) else {
            continue;
        };
        if stale(unsafe { (*dd).dv_copy_id }, copy_id) {
            unsafe { tv_dict_free_dict(dd) };
        }
    }
    for idx in 0..list_slots() {
        let Some(ll) = list_at(idx).map(NonNull::as_ptr) else {
            continue;
        };
        if stale(unsafe { (*ll).lv_copy_id }, copy_id) && !list_has_watchers(unsafe { ll.as_ref() })
        {
            unsafe { list_free_list(ll) };
        }
    }

    tv_in_free_unref_items.set(false);
    did_free as c_int
}

/// Dictionaries a marking walk has met and not yet looked inside: upstream's
/// `ht_stack`, a linked list of `xmalloc`ed nodes there.
pub(crate) type DictStack = Vec<DictRef>;
/// Lists likewise: upstream's `list_stack`.
pub(crate) type ListStack = Vec<ListRef>;

/// Mark every item of `dict`, draining the nested dictionaries it finds into
/// its own stack rather than recursing into them; nested lists go onto
/// `list_stack` when there is one.
///
/// The dictionary itself is not marked: its holder does that.
pub fn set_ref_in_dict_items(
    dict: &DictRef,
    copy_id: c_int,
    mut list_stack: Option<&mut ListStack>,
) -> bool {
    let mut ht_stack = DictStack::new();
    let mut abort = mark_dict_items(dict, copy_id, &mut ht_stack, list_stack.as_deref_mut());
    while !abort && let Some(next) = ht_stack.pop() {
        abort = mark_dict_items(&next, copy_id, &mut ht_stack, list_stack.as_deref_mut());
    }
    abort
}

/// One dictionary's items, for [`set_ref_in_dict_items`].
///
/// By slot, with the dictionary borrowed only to read each slot: marking
/// writes the `copyID` of every container it reaches, and one of them may be
/// this dictionary -- `g:` holding `g:`.
fn mark_dict_items(
    dict: &DictRef,
    copy_id: c_int,
    ht_stack: &mut DictStack,
    mut list_stack: Option<&mut ListStack>,
) -> bool {
    let mut slot = 0;
    while slot < dict.dv_hashtab.slots().len() {
        let item = dict.item_at(slot);
        slot += 1;
        if let Some(item) = item
            && set_ref_in_item(
                &item.di_tv,
                copy_id,
                Some(ht_stack),
                list_stack.as_deref_mut(),
            )
        {
            return true;
        }
    }
    false
}

/// Mark every item of `list`, draining the nested lists it finds into its
/// own stack rather than recursing into them; nested dictionaries go onto
/// `ht_stack` when there is one.
pub fn set_ref_in_list_items(
    list: &ListRef,
    copy_id: c_int,
    mut ht_stack: Option<&mut DictStack>,
) -> bool {
    let mut list_stack = ListStack::new();
    let mut abort = mark_list_items(list, copy_id, ht_stack.as_deref_mut(), &mut list_stack);
    while !abort && let Some(next) = list_stack.pop() {
        abort = mark_list_items(&next, copy_id, ht_stack.as_deref_mut(), &mut list_stack);
    }
    abort
}

/// One list's items, for [`set_ref_in_list_items`]; by index, for the
/// reason [`mark_dict_items`] gives.
fn mark_list_items(
    list: &ListRef,
    copy_id: c_int,
    mut ht_stack: Option<&mut DictStack>,
    list_stack: &mut ListStack,
) -> bool {
    let mut at = 0;
    while let Some(item) = list.items().get(at) {
        at += 1;
        if set_ref_in_item(
            &item.li_tv,
            copy_id,
            ht_stack.as_deref_mut(),
            Some(list_stack),
        ) {
            return true;
        }
    }
    false
}

/// Mark a dictionary. With no `ht_stack` it recurses; with one it defers,
/// pushing the dictionary for the caller's loop to drain.
pub(crate) fn set_ref_in_item_dict(
    dd: &DictRef,
    copy_id: c_int,
    ht_stack: Option<&mut DictStack>,
    mut list_stack: Option<&mut ListStack>,
) -> bool {
    if dd.dv_copy_id == copy_id {
        return false;
    }
    // Not seen yet.
    dd.edit().dv_copy_id = copy_id;
    let Some(ht_stack) = ht_stack else {
        return set_ref_in_dict_items(dd, copy_id, list_stack);
    };
    ht_stack.push(dd.clone());

    // The watchers' callbacks are marked only on this branch, which is
    // upstream's. A dictionary reached with no `ht_stack` — that is,
    // one recursed into directly — does not have them marked. A copy of the
    // list, so that no borrow of the dictionary spans the marking.
    let watchers = dd.watchers.clone();
    for watcher in &watchers {
        set_ref_in_callback(
            &watcher.callback,
            copy_id,
            Some(ht_stack),
            list_stack.as_deref_mut(),
        );
    }
    false
}

/// Mark a list. With no `list_stack` it recurses; with one it defers.
pub(crate) fn set_ref_in_item_list(
    ll: &ListRef,
    copy_id: c_int,
    ht_stack: Option<&mut DictStack>,
    list_stack: Option<&mut ListStack>,
) -> bool {
    if ll.lv_copy_id == copy_id {
        return false;
    }
    ll.edit().lv_copy_id = copy_id;
    let Some(list_stack) = list_stack else {
        return set_ref_in_list_items(ll, copy_id, ht_stack);
    };
    list_stack.push(ll.clone());
    false
}

/// Mark a partial: its function, its bound dictionary and its bound
/// arguments.
pub(crate) fn set_ref_in_item_partial(
    pt: &PartialRef,
    copy_id: c_int,
    mut ht_stack: Option<&mut DictStack>,
    mut list_stack: Option<&mut ListStack>,
) -> bool {
    if pt.pt_copy_id == copy_id {
        return false;
    }
    pt.edit().pt_copy_id = copy_id;

    let name = pt.pt_name.as_ref().map(|name| name.as_bytes());
    let mut abort = set_ref_in_func(name, pt.pt_func.as_ref(), copy_id);
    if let Some(dict) = &pt.pt_dict {
        abort = abort
            || set_ref_in_item_dict(
                dict,
                copy_id,
                ht_stack.as_deref_mut(),
                list_stack.as_deref_mut(),
            );
    }
    let mut at = 0;
    while !abort && let Some(arg) = pt.pt_argv.get(at) {
        at += 1;
        abort = set_ref_in_item(
            arg,
            copy_id,
            ht_stack.as_deref_mut(),
            list_stack.as_deref_mut(),
        );
    }
    abort
}

/// Mark whatever a typval holds. The scalar types hold nothing
/// collectable and fall through.
///
/// Marking writes the containers' `copyID`s, never the typval, so a shared
/// borrow is all it needs -- which is what lets a caller mark through a
/// value it can only see.
pub fn set_ref_in_item(
    tv: &TypVal,
    copy_id: c_int,
    ht_stack: Option<&mut DictStack>,
    list_stack: Option<&mut ListStack>,
) -> bool {
    let (ht, ls) = (ht_stack, list_stack);
    match tv.v_type() {
        VAR_DICT => tv
            .dict_shared()
            .is_some_and(|dict| set_ref_in_item_dict(dict, copy_id, ht, ls)),
        VAR_LIST => tv
            .list_shared()
            .is_some_and(|list| set_ref_in_item_list(list, copy_id, ht, ls)),
        // A Funcref names a function, which may be a closure holding a
        // scope of its own.
        VAR_FUNC => {
            let name = tv.func_name().map(|name| name.as_bytes());
            set_ref_in_func(name, None, copy_id)
        }
        VAR_PARTIAL => tv
            .partial_shared()
            .is_some_and(|pt| set_ref_in_item_partial(pt, copy_id, ht, ls)),
        _ => false,
    }
}

/// Copy a value, optionally deeply and optionally converting its strings.
///
/// `copy_id` is what makes a *deep* copy of a self-referential structure
/// terminate: a container already copied under this id answers with the
/// copy it made rather than making another. `to` is overwritten without
/// being released.
pub(crate) fn var_item_copy(
    conv: Option<&VimConv>,
    from: &TypVal,
    to: &mut TypVal,
    deep: bool,
    copy_id: c_int,
) -> Result<(), Failed> {
    static RECURSE: GlobalCell<c_int> = GlobalCell::new(0);

    if RECURSE.get() >= DICT_MAXNEST {
        emsg_static(e_variable_nested_too_deep_for_making_copy);
        return Err(Failed);
    }
    // The un-bump is the guard's, so that an early exit cannot skip it.
    let _depth = Depth::of(&RECURSE);

    let src = from;
    let mut ret = Ok(());
    match src.v_type() {
        VAR_STRING => {
            let converting = conv.filter(|conv| conv.vc_type != CONV_NONE);
            match (src.string_ref(), converting) {
                (Some(text), Some(conv)) => {
                    // A conversion that failed keeps the original bytes.
                    let converted = string_convert_cstr(conv, text.as_cstr());
                    to.write_string(Some(converted.unwrap_or_else(|| text.clone())));
                }
                _ => tv_copy(from, to),
            }
        }
        VAR_LIST => {
            let orig = src.list_shared();
            let copied = orig.and_then(|orig| {
                // The copy it was given under this id, which gains this
                // reference, or a fresh one.
                orig.copy_under(copy_id)
                    .or_else(|| list_copy(conv, orig, deep, copy_id))
            });
            let failed = orig.is_some() && copied.is_none();
            to.write_list(copied);
            if failed {
                ret = Err(Failed);
            }
        }
        VAR_DICT => {
            let orig = src.dict_shared();
            let copied = orig.and_then(|orig| {
                // The copy it was given under this id, which gains this
                // reference, or a fresh one.
                orig.copy_under(copy_id)
                    .or_else(|| dict_copy(conv, orig, deep, copy_id))
            });
            let failed = orig.is_some() && copied.is_none();
            to.write_dict(copied);
            if failed {
                ret = Err(Failed);
            }
        }
        VAR_BLOB => blob_copy(src.blob_ref(), to),
        VAR_UNKNOWN => {
            internal_error(c"var_item_copy(UNKNOWN)");
            ret = Err(Failed);
        }
        // Number, Float, Funcref, partial, Boolean and Special copy by
        // value or by reference count.
        VAR_NUMBER | VAR_FLOAT | VAR_FUNC | VAR_PARTIAL | VAR_BOOL | VAR_SPECIAL => {
            tv_copy(from, to);
        }
        _ => {}
    }

    ret
}

/// Is anything watching this list? A watched list is never freed, because
/// the watcher is a borrow the mark cannot see.
///
#[inline]
pub(crate) fn list_has_watchers(l: Option<&List>) -> bool {
    l.is_some_and(List::is_watched)
}
