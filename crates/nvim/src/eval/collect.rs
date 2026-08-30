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

use crate::guard::Depth;
use core::ffi::{c_char, c_int, c_void};
use core::mem::{offset_of, size_of};
use core::ptr::{null, null_mut};

use crate::autocmd::aucmd_wins;
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
use crate::eval::vars::emsg_static;
use crate::eval::vars::{
    garbage_collect_globvars, garbage_collect_scriptvars, garbage_collect_vimvars,
};
use crate::eval::{
    COPYID_INC, COPYID_MASK, DICT_MAXNEST, Tv, e_variable_nested_too_deep_for_making_copy,
    kMTCharWise, set_ref_in_callback, set_ref_in_callback_reader, timers,
};
use crate::ex_docmd::set_ref_in_findfunc;
use crate::global_cell::GlobalCell;
use crate::hashtab::hash_removed;
use crate::insexpand::{set_ref_in_cpt_callbacks, set_ref_in_insexpand_funcs};
use crate::main::{
    channels, garbage_collect_at_exit, may_garbage_collect, p_verbose, want_garbage_collect,
};
use crate::mark::mark_global_iter;
use crate::mbyte::string_convert;
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::message::{internal_error, verb_msg};
use crate::ops::set_ref_in_opfunc;
use crate::os::cshim::gettext;
use crate::quickfix::set_ref_in_quickfix;
use crate::register::op_global_reg_iter;
use crate::registry::SlotTable;
use crate::runtime::exestack;
use crate::tag::set_ref_in_tagfunc;
use crate::types::{
    AdditionalData, CONV_NONE, Callback, CallbackReader, Channel, DictWatcher, Failed, NUL, OptInt,
    QUEUE, String_0, VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER,
    VAR_PARTIAL, VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN, VarLock, buf_T, dict_T, dictitem_T, fmark_T,
    fmarkv_T, hashitem_T, hashtab_T, ht_stack_T, list_T, list_stack_T, listitem_T, partial_T,
    pos_T, size_t, tabpage_T, timer_T, typval_T, typval_vval_union, ufunc_T, vimconv_T, win_T,
    xfmark_T, yankreg_T,
};
use crate::winlayer::{Live, buffers, tab_windows, tabs};

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VarLock::Unlocked,
    vval: typval_vval_union { v_number: 0 },
};

/// How much slack the execution stack may keep before a collection trims
/// it back.
const EXESTACK_SLACK: usize = 500;

/// The garray `growsize` the execution stack was declared with, which is the
/// floor [`trim_exestack`] will not shrink below.
const EXESTACK_GROWSIZE: usize = 50;

/// The next mark. Two apart, so `set_ref_in_previous_funccal` can use the
/// odd value in between.
pub unsafe fn get_copy_id() -> c_int {
    static CURRENT_COPY_ID: GlobalCell<c_int> = GlobalCell::new(0);
    CURRENT_COPY_ID.set(CURRENT_COPY_ID.get() + COPYID_INC);
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

/// Mark one root's variable, with neither stack: the collector recurses
/// into whatever it holds rather than deferring it to a caller's loop.
///
/// # Safety
/// `tv` must be a live typval.
unsafe fn mark_root(tv: *mut typval_T, copy_id: c_int) -> bool {
    // SAFETY: the caller's promise; the two nulls are what say "recurse".
    unsafe { set_ref_in_item(tv, copy_id, null_mut(), null_mut()) }
}

/// Mark one callback, with neither stack.
///
/// # Safety
/// `cb` must be a live callback.
unsafe fn mark_cb(cb: *mut Callback, copy_id: c_int) -> bool {
    // SAFETY: as [`mark_root`].
    unsafe { set_ref_in_callback(cb, copy_id, null_mut(), null_mut()) }
}

/// Mark one callback reader, with neither stack.
///
/// # Safety
/// `reader` must be a live reader.
unsafe fn mark_reader(reader: *mut CallbackReader, copy_id: c_int) -> bool {
    // SAFETY: as [`mark_root`].
    unsafe { set_ref_in_callback_reader(reader, copy_id, null_mut(), null_mut()) }
}

/// Mark, then free. Answers whether anything was freed.
///
/// # Safety
/// Called from a point where no typval is held in a Rust temporary — see
/// the module docs; anything the marking pass cannot see is freed.
pub unsafe fn garbage_collect(testing: bool) -> bool {
    let mut abort = false;
    if !testing {
        // Only once per request.
        want_garbage_collect.set(false);
        may_garbage_collect.set(false);
        garbage_collect_at_exit.set(false);
    }

    trim_exestack();

    let copy_id = unsafe { get_copy_id() };

    // 1. Mark everything reachable from a root.

    // Variables in the previous_funccal list must not be freed unless
    // they are reachable *only* through it, so this goes first.
    abort = abort || unsafe { set_ref_in_previous_funccal(copy_id) };
    abort = abort || unsafe { garbage_collect_scriptvars(copy_id) };

    for buf in buffers() {
        // The addresses come off `Buf::raw`, never through `DerefMut`, so
        // no `&mut buf_T` is formed while they are live. `Live::field_ptr`
        // is what says where a field is without reading the object, so
        // naming these seven is ordinary code.
        // SAFETY: `buffers()` walks the editor's own list of live buffers.
        let buf = unsafe { Live::<buf_T>::new(buf.raw()) };
        // buffer-local variables
        let bufvar = buf.field_ptr(offset_of!(buf_T, b_bufvar.di_tv));
        // SAFETY: `bufvar` is the buffer's own variable dictionary.
        abort = abort || unsafe { mark_root(bufvar, copy_id) };
        // buffer callback functions
        for offset in [
            offset_of!(buf_T, b_prompt_callback),
            offset_of!(buf_T, b_prompt_interrupt),
            offset_of!(buf_T, b_cfu_cb),
            offset_of!(buf_T, b_ofu_cb),
            offset_of!(buf_T, b_tsrfu_cb),
            offset_of!(buf_T, b_tfu_cb),
            offset_of!(buf_T, b_ffu_cb),
        ] {
            let cb: *mut Callback = buf.field_ptr(offset);
            // SAFETY: `cb` is one of the buffer's own callbacks.
            abort = abort || unsafe { mark_cb(cb, copy_id) };
        }
        // The buffer's own 'complete' callback list.
        let (cpt_cb, cpt_count) = (buf.b_p_cpt_cb, buf.b_p_cpt_count);
        if !abort && !cpt_cb.is_null() {
            // SAFETY: as above -- `cpt_count` entries of the buffer's list.
            abort = abort || unsafe { set_ref_in_cpt_callbacks(cpt_cb, cpt_count, copy_id) };
        }
    }

    // 'completefunc', 'omnifunc', 'thesaurusfunc', 'operatorfunc',
    // 'tagfunc' and 'findfunc' callbacks.
    abort = abort || unsafe { set_ref_in_insexpand_funcs(copy_id) };
    abort = abort || unsafe { set_ref_in_opfunc(copy_id) };
    abort = abort || unsafe { set_ref_in_tagfunc(copy_id) };
    abort = abort || unsafe { set_ref_in_findfunc(copy_id) };

    // window-local variables, in every tab page
    for wp in tab_windows() {
        // SAFETY: the walk answers the editor's own live windows.
        let wp = unsafe { Live::<win_T>::new(wp.raw()) };
        let winvar = wp.field_ptr(offset_of!(win_T, w_winvar.di_tv));
        // SAFETY: `winvar` is the window's own variable dictionary.
        abort = abort || unsafe { mark_root(winvar, copy_id) };
    }

    // window-local variables in the autocommand windows
    let wins = aucmd_wins();
    for i in 0..wins.len() {
        // SAFETY: `i` is inside the table, whose windows are live.
        let win = unsafe { (*wins.slot(i)).auc_win };
        if !win.is_null() {
            // SAFETY: as above.
            let win = unsafe { Live::<win_T>::new(win) };
            let winvar = win.field_ptr(offset_of!(win_T, w_winvar.di_tv));
            // SAFETY: `winvar` is that window's own variable dictionary.
            abort = abort || unsafe { mark_root(winvar, copy_id) };
        }
    }

    unsafe { walk_shada_iterators() };

    // tabpage-local variables
    for tp in tabs() {
        // SAFETY: the walk answers the editor's own live tab pages.
        let tp = unsafe { Live::<tabpage_T>::new(tp.raw()) };
        let tpvar = tp.field_ptr(offset_of!(tabpage_T, tp_winvar.di_tv));
        // SAFETY: `tpvar` is the tab page's own variable dictionary.
        abort = abort || unsafe { mark_root(tpvar, copy_id) };
    }

    abort = abort || unsafe { garbage_collect_globvars(copy_id) } != 0;
    // function-local variables, then named functions (closures)
    abort = abort || unsafe { set_ref_in_call_stack(copy_id) };
    abort = abort || unsafe { set_ref_in_functions(copy_id) };

    // Channels. Deliberately not `abort`ed on: upstream discards these
    // answers, and doing otherwise would change when a collection is
    // abandoned.
    for data in channels.with(SlotTable::snapshot_values) {
        // SAFETY: the snapshot holds the registered live channels.
        let ch = unsafe { Live::<Channel>::new(data) };
        let on_data = ch.field_ptr(offset_of!(Channel, on_data));
        let on_stderr = ch.field_ptr(offset_of!(Channel, on_stderr));
        let on_exit = ch.field_ptr(offset_of!(Channel, on_exit));
        // SAFETY: all three are the channel's own callbacks.
        unsafe { mark_reader(on_data, copy_id) };
        // SAFETY: as above.
        unsafe { mark_reader(on_stderr, copy_id) };
        // SAFETY: as above.
        unsafe { mark_cb(on_exit, copy_id) };
    }

    // Timers, likewise.
    for timer in timers.with(SlotTable::snapshot_values) {
        // SAFETY: the snapshot holds the registered live timers.
        let cb = unsafe { Live::<timer_T>::new(timer) }.field_ptr(offset_of!(timer_T, callback));
        // SAFETY: `cb` is the timer's own callback.
        unsafe { mark_cb(cb, copy_id) };
    }

    // function call arguments, if v:testing is set
    abort = abort || unsafe { set_ref_in_func_args(copy_id) };
    abort = abort || unsafe { garbage_collect_vimvars(copy_id) };
    abort = abort || unsafe { set_ref_in_quickfix(copy_id) };

    // 2. Free what nothing marked — but only if every root was seen.
    if abort {
        if p_verbose.get() > 0 as OptInt {
            let msg = c"Not enough memory to set references, garbage collection aborted!";
            // SAFETY: the message is a NUL-terminated literal.
            unsafe { verb_msg(gettext(msg).as_ptr()) };
        }
        return false;
    }
    // SAFETY: the marking pass above is complete, which is what
    // `free_unref_items` and `free_unref_funccal` both rest on.
    let did_free = unsafe { free_unref_items(copy_id) } != 0;
    // 3. Any funccal that can go now. May call back into here.
    // SAFETY: as above.
    let freed_funccal = unsafe { free_unref_funccal(copy_id, testing as c_int) };
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
        let (n, r, u) = (&raw mut name, &raw mut reg, &raw mut is_unnamed);
        // SAFETY: the caller's promise; the three are this frame's.
        reg_iter = unsafe { op_global_reg_iter(reg_iter, n, r, u) };
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
        mark_iter = unsafe { mark_global_iter(mark_iter, &raw mut name, &raw mut fm) };
        if mark_iter.is_null() {
            break;
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
        if stale(unsafe { (*dd).dv_copyID }, copy_id) {
            unsafe { tv_dict_free_contents(dd) };
            did_free = true;
        }
        dd = unsafe { (*dd).dv_used_next };
    }
    // …and the unreachable lists. A list with a watcher is left alone:
    // the watcher is a borrow the collector cannot see.
    let mut ll = gc_first_list.get();
    while !ll.is_null() {
        if stale(unsafe { tv_list_copyid(ll) }, copy_id) && !unsafe { tv_list_has_watchers(ll) } {
            unsafe { tv_list_free_contents(ll) };
            did_free = true;
        }
        ll = unsafe { (*ll).lv_used_next };
    }

    // Pass 2: unlink and free the structures themselves. The `next`
    // pointer is read before the free.
    let mut dd = gc_first_dict.get();
    while !dd.is_null() {
        let next = unsafe { (*dd).dv_used_next };
        if stale(unsafe { (*dd).dv_copyID }, copy_id) {
            unsafe { tv_dict_free_dict(dd) };
        }
        dd = next;
    }
    let mut ll = gc_first_list.get();
    while !ll.is_null() {
        let next = unsafe { (*ll).lv_used_next };
        if stale(unsafe { (*ll).lv_copyID }, copy_id) && !unsafe { tv_list_has_watchers(ll) } {
            unsafe { tv_list_free_list(ll) };
        }
        ll = next;
    }

    tv_in_free_unref_items.set(false);
    did_free as c_int
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
    let mut abort = false;
    let mut ht_stack: *mut ht_stack_T = null_mut();
    let mut cur_ht = ht;
    loop {
        if !abort {
            // A nested hashtab is pushed onto `ht_stack`, a nested list
            // onto the caller's `list_stack`.
            let mut todo = unsafe { (*cur_ht).ht_used };
            let mut hi: *mut hashitem_T = unsafe { (*cur_ht).ht_array };
            while todo != 0 {
                if !unsafe { (*hi).hi_key }.is_null()
                    && !core::ptr::eq(unsafe { (*hi).hi_key }, &raw const hash_removed)
                {
                    todo -= 1;
                    // SAFETY: `hi` is a live entry, so the item its inline
                    // key belongs to is live too; `ht_stack` is this
                    // frame's and `list_stack` the caller's.
                    let tv = unsafe { &raw mut (*hi2di(hi)).di_tv };
                    let stack = &raw mut ht_stack;
                    // SAFETY: as above.
                    abort = abort || unsafe { set_ref_in_item(tv, copy_id, stack, list_stack) };
                }
                hi = unsafe { hi.add(1) };
            }
        }
        // The stack is drained even while aborting, so nothing leaks.
        if ht_stack.is_null() {
            break;
        }
        cur_ht = unsafe { (*ht_stack).ht };
        let done = ht_stack;
        ht_stack = unsafe { (*ht_stack).prev } as *mut ht_stack_T;
        unsafe { xfree(done as *mut c_void) };
    }
    abort
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
    let mut abort = false;
    let mut list_stack: *mut list_stack_T = null_mut();
    let mut cur_l = l;
    loop {
        if !cur_l.is_null() {
            let mut li: *mut listitem_T = unsafe { (*cur_l).lv_first };
            while !li.is_null() {
                if abort {
                    break;
                }
                abort = unsafe {
                    set_ref_in_item(&raw mut (*li).li_tv, copy_id, ht_stack, &raw mut list_stack)
                };
                li = unsafe { (*li).li_next };
            }
        }
        if list_stack.is_null() {
            break;
        }
        cur_l = unsafe { (*list_stack).list };
        let done = list_stack;
        list_stack = unsafe { (*list_stack).prev } as *mut list_stack_T;
        unsafe { xfree(done as *mut c_void) };
    }
    abort
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
    if dd.is_null() || unsafe { (*dd).dv_copyID } == copy_id {
        return false;
    }
    // Not seen yet.
    unsafe { (*dd).dv_copyID = copy_id };
    if ht_stack.is_null() {
        return unsafe { set_ref_in_ht(&raw mut (*dd).dv_hashtab, copy_id, list_stack) };
    }

    let newitem = unsafe { xmalloc(size_of::<ht_stack_T>()) } as *mut ht_stack_T;
    // SAFETY: `newitem` is the block just allocated, and `dd` is a live
    // Dict whose hashtab lives inside it.
    unsafe { (*newitem).ht = &raw mut (*dd).dv_hashtab };
    // SAFETY: the caller's promise about `ht_stack`, which this pushes on.
    unsafe { (*newitem).prev = *ht_stack };
    // SAFETY: as above.
    unsafe { *ht_stack = newitem };

    // The watchers' callbacks are marked only on this branch, which is
    // upstream's. A dictionary reached with no `ht_stack` — that is,
    // one recursed into directly — does not have them marked.
    let mut w: *mut QUEUE = unsafe { (*dd).watchers.next } as *mut QUEUE;
    // SAFETY: `dd` is a live Dict, and the queue head lives inside it.
    let head: *mut QUEUE = unsafe { &raw mut (*dd).watchers };
    while w != head {
        let next: *mut QUEUE = unsafe { (*w).next } as *mut QUEUE;
        let watcher: *mut DictWatcher = unsafe { tv_dict_watcher_node_data(w) };
        unsafe { set_ref_in_callback(&raw mut (*watcher).callback, copy_id, ht_stack, list_stack) };
        w = next;
    }
    false
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
    if ll.is_null() || unsafe { (*ll).lv_copyID } == copy_id {
        return false;
    }
    unsafe { (*ll).lv_copyID = copy_id };
    if list_stack.is_null() {
        return unsafe { set_ref_in_list_items(ll, copy_id, ht_stack) };
    }
    // SAFETY: `xmalloc` never answers NULL, and the caller's promise about
    // `list_stack`, which this pushes the new entry onto.
    let newitem = unsafe { xmalloc(size_of::<list_stack_T>()) } as *mut list_stack_T;
    unsafe { (*newitem).list = ll };
    // SAFETY: as above.
    unsafe { (*newitem).prev = *list_stack };
    // SAFETY: as above.
    unsafe { *list_stack = newitem };
    false
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
    if pt.is_null() || unsafe { (*pt).pt_copyID } == copy_id {
        return false;
    }
    unsafe { (*pt).pt_copyID = copy_id };

    let mut abort = unsafe { set_ref_in_func((*pt).pt_name, (*pt).pt_func, copy_id) };
    if !unsafe { (*pt).pt_dict }.is_null() {
        // A borrowed view, not an owner: `dtv` is never cleared.
        let mut dtv = UNSET_TV;
        dtv.v_type = VAR_DICT;
        dtv.vval.v_dict = unsafe { (*pt).pt_dict };
        abort = abort || unsafe { set_ref_in_item(&raw mut dtv, copy_id, ht_stack, list_stack) };
    }
    // SAFETY: `pt` is a live partial, so it holds `pt_argc` bound
    // arguments and `pt_argv` names them.
    for i in 0..unsafe { (*pt).pt_argc } {
        // SAFETY: as above -- `i` is one of them.
        let arg = unsafe { (*pt).pt_argv.offset(i as isize) };
        // SAFETY: as above; the stacks are the caller's.
        abort = abort || unsafe { set_ref_in_item(arg, copy_id, ht_stack, list_stack) };
    }
    abort
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
    // SAFETY: the caller's promise -- the typval outlives the call, and the
    // union member each arm reads is the one its `v_type` names; the stacks
    // are the caller's.
    let tv = unsafe { Tv::new(tv) };
    let (ht, ls) = (ht_stack, list_stack);
    match tv.v_type {
        // SAFETY: as above.
        VAR_DICT => unsafe { set_ref_in_item_dict(tv.vval.v_dict, copy_id, ht, ls) },
        VAR_LIST => unsafe { set_ref_in_item_list(tv.vval.v_list, copy_id, ht, ls) },
        // A Funcref names a function, which may be a closure holding a
        // scope of its own.
        VAR_FUNC => unsafe { set_ref_in_func(tv.vval.v_string, null_mut::<ufunc_T>(), copy_id) },
        VAR_PARTIAL => unsafe { set_ref_in_item_partial(tv.vval.v_partial, copy_id, ht, ls) },
        _ => false,
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
) -> Result<(), Failed> {
    static RECURSE: GlobalCell<c_int> = GlobalCell::new(0);

    if RECURSE.get() >= DICT_MAXNEST {
        emsg_static(e_variable_nested_too_deep_for_making_copy);
        return Err(Failed);
    }
    // The un-bump is the guard's, so that an early exit cannot skip it.
    let _depth = Depth::of(&RECURSE);

    // SAFETY: the caller's promise -- both typvals outlive the call. Every
    // union member read below is the one `src.v_type` names, and the
    // matching member of `dst` is written before it is read.
    let (src, mut dst) = unsafe { (Tv::new(from), Tv::new(to)) };
    let mut ret = Ok(());
    match src.v_type {
        VAR_STRING => {
            // SAFETY: as above; a null `conv` is not read.
            let plain = conv.is_null()
                || unsafe { (*conv).vc_type } == CONV_NONE
                || unsafe { src.vval.v_string }.is_null();
            if plain {
                // SAFETY: both typvals are the caller's.
                unsafe { tv_copy(from, to) };
            } else {
                dst.v_type = VAR_STRING;
                dst.v_lock = VarLock::Unlocked;
                let (cv, s) = (conv as *mut vimconv_T, unsafe { src.vval.v_string });
                // SAFETY: `s` is the source string and `cv` the conversion.
                dst.vval.v_string = unsafe { string_convert(cv, s, null_mut::<size_t>()) };
                // A conversion that failed keeps the original bytes.
                // SAFETY: `v_string` is the member just written.
                if unsafe { dst.vval.v_string }.is_null() {
                    // SAFETY: `s` is the source's NUL-terminated string.
                    dst.vval.v_string = unsafe { xstrdup(s) };
                }
            }
        }
        VAR_LIST => {
            dst.v_type = VAR_LIST;
            dst.v_lock = VarLock::Unlocked;
            // SAFETY: `VAR_LIST` says `v_list` is the live member.
            let l = unsafe { src.vval.v_list };
            if l.is_null() {
                dst.vval.v_list = null_mut::<list_T>();
            // SAFETY: `l` is the source's live List.
            } else if copy_id != 0 && unsafe { tv_list_copyid(l) } == copy_id {
                // Already copied under this id: share that copy.
                // SAFETY: as above -- the copy it was given under this id.
                dst.vval.v_list = unsafe { tv_list_latest_copy(l) };
                // SAFETY: the shared copy gains this reference.
                unsafe { tv_list_ref(dst.vval.v_list) };
            } else {
                // SAFETY: as above; `conv` is null or the caller's.
                dst.vval.v_list = unsafe { tv_list_copy(conv, l, deep, copy_id) };
            }
            // SAFETY: `v_list` is the member just written.
            if unsafe { dst.vval.v_list }.is_null() && !l.is_null() {
                ret = Err(Failed);
            }
        }
        VAR_DICT => {
            dst.v_type = VAR_DICT;
            dst.v_lock = VarLock::Unlocked;
            // SAFETY: `VAR_DICT` says `v_dict` is the live member.
            let d = unsafe { src.vval.v_dict };
            if d.is_null() {
                dst.vval.v_dict = null_mut::<dict_T>();
            // SAFETY: `d` is the source's live Dict.
            } else if copy_id != 0 && unsafe { (*d).dv_copyID } == copy_id {
                // SAFETY: as above -- the copy it was given under this id,
                // which gains this reference.
                dst.vval.v_dict = unsafe { (*d).dv_copydict };
                // SAFETY: as above -- the shared copy gains this reference.
                unsafe { (*dst.vval.v_dict).dv_refcount.retain() };
            } else {
                // SAFETY: as above; `conv` is null or the caller's.
                dst.vval.v_dict = unsafe { tv_dict_copy(conv, d, deep, copy_id) };
            }
            // SAFETY: `v_dict` is the member just written.
            if unsafe { dst.vval.v_dict }.is_null() && !d.is_null() {
                ret = Err(Failed);
            }
        }
        VAR_BLOB => {
            // SAFETY: `VAR_BLOB` says `v_blob` is the live member, and `to`
            // is the caller's typval.
            unsafe { tv_blob_copy(src.vval.v_blob, to) };
        }
        VAR_UNKNOWN => {
            // SAFETY: the text is a NUL-terminated literal.
            unsafe { internal_error(c"var_item_copy(UNKNOWN)".as_ptr()) };
            ret = Err(Failed);
        }
        // Number, Float, Funcref, partial, Boolean and Special copy by
        // value or by reference count.
        VAR_NUMBER | VAR_FLOAT | VAR_FUNC | VAR_PARTIAL | VAR_BOOL | VAR_SPECIAL => {
            // SAFETY: both typvals are the caller's.
            unsafe { tv_copy(from, to) };
        }
        _ => {}
    }

    ret
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
