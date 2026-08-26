//! The stack of lists a window works on.
//!
//! There is exactly one quickfix stack ([`QfStack::Global`], a static), and
//! one location list stack per window that has asked for one. A location
//! list stack is reference counted, because `:lopen` gives the location list
//! window a second reference to the same stack, and either window may be
//! closed first.
//!
//! The two are the same struct and the same code works on both, so most of
//! this module still takes a `*mut qf_info_T`. [`QfStack`] is for the places
//! where the difference matters: [`qf_alloc_stack`] makes a location list
//! stack and only a location list stack, and [`qf_free_lists`] frees one and
//! only one. [`qf_resize_stack_base`] changes how many lists a stack holds
//! (`'chistory'`/`'lhistory'`) and [`ll_free_all`] drops a reference.
//! Freeing is deferred while [`incr_quickfix_busy`] is in effect: an
//! autocommand fired from the middle of a quickfix command can close the
//! window whose location list that command is still walking.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::event_T;
use crate::types::{FAIL, OK};
use crate::winlayer::{Buf, Live, Win};
use core::ffi::{CStr, c_char, c_int, c_uint};
use core::ptr;

/// A stack the caller has promised outlives the value.
///
/// The quickfix code hands `*mut qf_info_T` around because an autocommand
/// can reach the same stack while a command is walking it, so no borrow may
/// outlive one field access — which is exactly what [`Live`]'s `Deref`
/// gives. Wrapping is the unsafe step, once per entry point; every
/// `(*qi).field` after it is ordinary checked code.
pub(crate) type Qi = Live<qf_info_T>;

/// One list on a stack — the same promise as [`Qi`].
pub(crate) type Qfl = Live<qf_list_T>;

/// One entry on a list — the same promise as [`Qi`].
pub(crate) type Qfe = Live<qfline_T>;

/// The Ex command being run — the same promise as [`Qi`], discharged by the
/// `do_cmdline` frame that owns the `exarg_T` outliving the call.
pub(crate) type Ea = Live<exarg_T>;

/// The one quickfix stack. It is a static rather than an allocation
/// because it outlives every window and is never freed. Reached through
/// [`QfStack::Global`], which is the only thing in the tree that knows the
/// storage is a static at all.
static ql_info_actual: GlobalCell<qf_info_T> = GlobalCell::new(qf_info_T::new(QFLT_QUICKFIX));

/// Which of the two kinds of stack a `*mut qf_info_T` names.
///
/// They are the same struct, but they are not owned the same way. The
/// quickfix stack is a static: one per editor, live before `main` reads its
/// first command, never freed, and its [`qf_refcount`](qf_info_T) means
/// nothing. A location list stack is an allocation shared by the window that
/// owns the list and the location list window showing it, freed at the last
/// reference. So the global variant names a *slot* and carries no address,
/// and "is this the quickfix stack?" — the question the free path has to get
/// right — is a `match` rather than a pointer comparison.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum QfStack {
    /// The quickfix stack, which every `:c…` command works on.
    Global,
    /// A location list stack, or the throwaway `QFLT_INTERNAL` one
    /// `getqflist({'lines': …})` parses into. Never null.
    Local(*mut qf_info_T),
}

impl QfStack {
    /// Which stack `qi` is. `qi` must name a live stack.
    pub(crate) fn of(qi: *mut qf_info_T) -> QfStack {
        debug_assert!(!qi.is_null());
        if ptr::eq(qi, QfStack::Global.raw()) {
            QfStack::Global
        } else {
            QfStack::Local(qi)
        }
    }

    /// The address, for the rest of the quickfix code — which passes a
    /// `*mut qf_info_T` around because an autocommand can reach the same
    /// stack while a command is walking it, so no borrow may outlive one
    /// field access.
    ///
    /// This is the module's single `ql_info_actual.ptr()`: the quickfix
    /// stack's address is what every `qf_*` function takes, and this is
    /// where it comes from.
    pub(crate) fn raw(self) -> *mut qf_info_T {
        match self {
            QfStack::Global => ql_info_actual.ptr(),
            QfStack::Local(qi) => qi,
        }
    }
}

// ---------------------------------------------------------------------------
// The safe front the rest of the family calls.
//
// Each of these wraps one transpiled `unsafe fn` whose only precondition is
// "the editor exists", or "the stack is live" — which is the promise a
// [`Qi`]/[`Qfl`] already records. Paying it once here is what lets the
// commands themselves be ordinary checked code, and it is why the family's
// unchecked line count is a list of these bodies rather than of its calls.

/// `emsg(_(msg))`: report an error whose text is already a C string.
///
/// `msg` must be NUL-terminated, which every caller's static message is.
pub(crate) fn qf_emsg(msg: *const c_char) {
    // SAFETY: a NUL-terminated static message, per the contract above.
    unsafe { emsg(gettext(msg)) };
}

/// Fire `QuickFixCmdPre`/`QuickFixCmdPost` for a quickfix command, and say
/// whether an autocommand claimed the event.
///
/// `on_fname` is upstream's split: the commands that read a file or run a
/// program match the pattern against the current buffer's name and force the
/// event, the ones taking their input from Vimscript match on neither.
pub(crate) fn fire_qf_autocmd(event: event_T, name: &CStr, on_fname: bool) -> bool {
    let pat = name.as_ptr().cast_mut();
    let fname = if on_fname {
        cur_buf().b_fname
    } else {
        ptr::null_mut()
    };
    // SAFETY: a static event name, the current buffer's own file name, and
    // the current buffer — all live across the call.
    unsafe { apply_autocmds(event, pat, fname, on_fname, curbuf.get()) }
}

/// The quickfix stack, as a [`Qi`]. It is a static, so it is always live.
pub(crate) fn qf_global() -> Qi {
    // SAFETY: the quickfix stack is a static, live before the first command.
    unsafe { Qi::new(QfStack::Global.raw()) }
}

/// A stack that may be absent — what a location list command finds in a
/// window that has no location list. `qi` must be null or a live stack.
pub(crate) fn qf_opt(qi: *mut qf_info_T) -> Option<Qi> {
    // SAFETY: the caller's stack, tested for null first.
    (!qi.is_null()).then(|| unsafe { Qi::new(qi) })
}

/// [`qf_cmd_get_stack`], as a stack that may be absent.
pub(crate) fn qf_cmd_stack(eap: Ea, print_emsg: bool) -> Option<Qi> {
    // SAFETY: `eap`'s promise — a live command.
    qf_opt(unsafe { qf_cmd_get_stack(eap.raw(), print_emsg) })
}

/// [`qf_cmd_get_or_alloc_stack`], which never answers null.
pub(crate) fn qf_cmd_stack_or_alloc(eap: Ea, pwinp: *mut *mut win_T) -> Qi {
    // SAFETY: `eap`'s promise, and `pwinp` is the caller's own local.
    unsafe { Qi::new(qf_cmd_get_or_alloc_stack(eap.raw().cast_const(), pwinp)) }
}

/// [`win_loclist`], as a stack that may be absent. `wp` must be a live
/// window.
pub(crate) fn qf_win_loclist(wp: *mut win_T) -> Option<Qi> {
    // SAFETY: the caller's window.
    qf_opt(unsafe { win_loclist(wp) })
}

/// [`qf_jump`]: go to the `errornr`th entry, counting from the current one
/// in `dir`.
pub(crate) fn qf_goto(qi: Qi, dir: c_int, errornr: c_int, forceit: c_int) {
    // SAFETY: `qi`'s promise -- a live stack.
    unsafe { qf_jump(qi.raw(), dir, errornr, forceit) };
}

/// `qf_get_curlist()`: the list a stack is currently on.
pub(crate) fn qf_current_list(qi: Qi) -> Qfl {
    // SAFETY: `qi`'s promise, and a stack always has a current list.
    unsafe { Qfl::new(qf_get_curlist(qi.raw())) }
}

/// `qf_get_list()`: the `idx`th list, which must be a slot the stack has.
pub(crate) fn qf_nth_list(qi: Qi, idx: c_int) -> Qfl {
    // SAFETY: `qi`'s promise, and the caller's slot.
    unsafe { Qfl::new(qf_get_list(qi.raw(), idx)) }
}

/// `qf_stack_empty()`: whether the stack holds no lists at all.
pub(crate) fn qf_is_empty(qi: Qi) -> bool {
    // SAFETY: `qi`'s promise.
    unsafe { qf_stack_empty(qi.raw()) }
}

/// `qf_list_empty()`: whether the list holds no entries.
pub(crate) fn qfl_is_empty(qfl: Qfl) -> bool {
    // SAFETY: `qfl`'s promise.
    unsafe { qf_list_empty(qfl.raw()) }
}

/// `qf_list_changed()`: bump the list's change tick.
pub(crate) fn qfl_changed(qfl: Qfl) {
    // SAFETY: `qfl`'s promise.
    unsafe { qf_list_changed(qfl.raw()) };
}

/// `qf_update_buffer()`: redraw the quickfix window, if there is one.
///
/// `old_last` is the entry the buffer was filled to last time, or null for a
/// full rewrite.
pub(crate) fn qf_redraw(qi: Qi, old_last: *mut qfline_T) {
    // SAFETY: `qi`'s promise, and the caller's entry.
    unsafe { qf_update_buffer(qi.raw(), old_last) };
}

/// `qflist_valid()`: whether the list `qf_id` names is still the one a
/// command started on. `wp` is null for the quickfix stack.
pub(crate) fn qf_list_still_valid(wp: *mut win_T, qf_id: c_uint) -> bool {
    // SAFETY: a live window or null, which is what every caller holds.
    unsafe { qflist_valid(wp, qf_id) }
}

/// [`decr_quickfix_busy`], which only ever frees stacks nothing can reach.
pub(crate) fn qf_busy_end() {
    // SAFETY: the deferred frees are stacks `ll_free_all` had already
    // removed the last reachable reference to.
    unsafe { decr_quickfix_busy() };
}

/// How deep the quickfix code is inside a command that holds a stack
/// pointer. While this is above zero, freeing a location list stack is
/// deferred to [`PENDING_FREE`].
static quickfix_busy: GlobalCell<c_int> = GlobalCell::new(0);

/// Location list stacks whose free was deferred, newest last.
static PENDING_FREE: GlobalCell<Vec<*mut qf_info_T>> = GlobalCell::new(Vec::new());

/// Whether the stack holds no lists at all. A null stack counts as empty,
/// which is how the location list commands report "no location list".
#[inline]
pub(crate) unsafe fn qf_stack_empty(qi: *const qf_info_T) -> bool {
    // SAFETY: the caller's stack, which may be null.
    unsafe { qi.is_null() || (*qi).qf_listcount <= 0 }
}

/// Whether `wp` *is* a location list window, i.e. shows another window's
/// location list rather than owning one.
#[inline]
pub(crate) unsafe fn is_ll_window(wp: *const win_T) -> bool {
    // SAFETY: the caller's promise -- a live `win_T`.
    let wp = unsafe { Win::new(wp.cast_mut()) };
    // SAFETY: the caller's window.
    unsafe { bt_quickfix(wp.w_buffer) && !wp.w_llist_ref.is_null() }
}

/// Whether `wp` is a *quickfix* window, as opposed to a location list one.
pub(crate) unsafe fn is_qf_window(wp: *const win_T) -> bool {
    // SAFETY: the caller's promise -- a live `win_T`.
    let wp = unsafe { Win::new(wp.cast_mut()) };
    // SAFETY: the caller's window.
    unsafe { bt_quickfix(wp.w_buffer) && wp.w_llist_ref.is_null() }
}

/// The location list stack `wp` works on: the one it references when it is
/// a location list window, otherwise its own. May be null.
#[inline]
pub(crate) unsafe fn win_loclist(wp: *mut win_T) -> *mut qf_info_T {
    // SAFETY: the caller's promise -- a live `win_T`.
    let wp = unsafe { Win::new(wp) };
    // SAFETY: the caller's window.
    if unsafe { is_ll_window(wp.raw().cast_const()) } {
        wp.w_llist_ref
    } else {
        wp.w_llist
    }
}

/// The `idx`th list in the stack.
///
/// # Safety
///
/// `qi` must be a live stack and `idx` below its
/// [`max_count`](qf_info_T::max_count).
#[inline]
pub(crate) unsafe fn qf_get_list(qi: *mut qf_info_T, idx: c_int) -> *mut qf_list_T {
    // SAFETY: the caller's promise -- a live `qf_info_T`.
    let mut qi = unsafe { Qi::new(qi) };
    // SAFETY: the caller's stack and a slot it has room for. The pointer
    // is into the `Vec`'s heap buffer, which outlives every borrow of the
    // stack itself and is only invalidated by `qf_resize_stack_base`.
    unsafe { qi.qf_lists.as_mut_ptr().add(idx as usize) }
}

/// The list `:cc` and friends work on.
///
/// # Safety
///
/// `qi` must be a live stack with at least one list.
#[inline]
pub(crate) unsafe fn qf_get_curlist(qi: *mut qf_info_T) -> *mut qf_list_T {
    // SAFETY: the caller's promise -- a live `qf_info_T`.
    let mut qi = unsafe { Qi::new(qi) };
    // SAFETY: forwarded from the caller.
    unsafe { qf_get_list(qi.raw(), qi.qf_curlist) }
}

/// Drop the oldest list and shuffle the rest down, leaving a zeroed slot at
/// the top.
///
/// With `adjust`, the stack also shrinks and `qf_curlist` follows the list
/// it pointed at — or, if that was the one dropped, the newest.
///
/// # Safety
///
/// `qi` must be a live stack holding at least one list.
pub(crate) unsafe fn qf_pop_stack(raw: *mut qf_info_T, adjust: bool) {
    // SAFETY: forwarded from the caller -- a live stack.
    let mut qi = unsafe { Qi::new(raw) };
    // SAFETY: as above; slot 0 exists because the stack holds a list.
    unsafe { qf_free(qf_get_list(raw, 0)) };
    let count = qi.qf_listcount as usize;
    drop_oldest_list(&mut qi.qf_lists, count);
    if adjust {
        qi.qf_listcount -= 1;
        qi.qf_curlist = if qi.qf_curlist == 0 {
            qi.qf_listcount - 1
        } else {
            qi.qf_curlist - 1
        };
    }
}

/// Shift the newest `count - 1` lists down one slot and leave the top empty.
///
/// This is `slice::copy_within` without the `Copy` bound. A list owns its
/// entries, its title and a `qf_qftf_cb`, so the lists are *moved*: cloning
/// upwards leaves each source intact until it has been read, and the slot
/// the shift vacates is overwritten before anything can see it -- which is
/// exactly what the `memmove` it replaces did.
fn drop_oldest_list(lists: &mut [qf_list_T], count: usize) {
    for at in 1..count {
        lists[at - 1] = lists[at].clone();
    }
    lists[count - 1] = empty_list();
}

/// The buffer the quickfix window shows, or `INVALID_QFBUFNR`.
pub fn qf_stack_get_bufnr() -> c_int {
    // One field of the static, read and copied out: the borrow cannot
    // outlive the expression, so nothing an autocommand does can reach it.
    ql_info_actual.with(|qi| qi.qf_bufnr)
}

/// Wipe the quickfix window's buffer, if it is not displayed anywhere.
///
/// # Safety
///
/// `qi` must be a live stack.
unsafe fn wipe_qf_buffer(qi: *mut qf_info_T) {
    // SAFETY: the caller's promise -- a live `qf_info_T`.
    let mut qi = unsafe { Qi::new(qi) };
    // SAFETY: forwarded from the caller.
    if qi.qf_bufnr == INVALID_QFBUFNR {
        return;
    }
    let qfbuf = buflist_findnr(qi.qf_bufnr);
    if qfbuf.is_null() || unsafe { (*qfbuf).b_nwindows } != 0 {
        return;
    }
    // `close_buffer` insists that `curwin->w_buffer == curbuf`, and it
    // may not: this is reachable from `win_free_mem` after `win_close`
    // already released the current window's buffer.
    let buf_was_null = cur_win().w_buffer.is_null();
    if buf_was_null {
        cur_win().w_buffer = curbuf.get();
    }
    unsafe { close_buffer(ptr::null_mut(), qfbuf, DOBUF_WIPE as c_int, false, false) };
    qi.qf_bufnr = INVALID_QFBUFNR;
    if buf_was_null {
        cur_win().w_buffer = ptr::null_mut();
    }
}

/// Free every list in the stack, leaving the stack itself.
///
/// # Safety
///
/// `qi` must be a live stack.
unsafe fn qf_free_list_stack_items(qi: *mut qf_info_T) {
    // SAFETY: the caller's promise -- a live `qf_info_T`.
    let mut qi = unsafe { Qi::new(qi) };
    // SAFETY: forwarded from the caller.
    for i in 0..qi.qf_listcount {
        unsafe { qf_free(qf_get_list(qi.raw(), i)) };
    }
}

/// Free a whole location list stack.
///
/// # Safety
///
/// `qi` must be a boxed stack with no references left — never the quickfix
/// stack, which is a static.
pub(crate) unsafe fn qf_free_lists(qi: *mut qf_info_T) {
    // SAFETY: forwarded from the caller.
    debug_assert!(matches!(QfStack::of(qi), QfStack::Local(_)));
    unsafe { qf_free_list_stack_items(qi) };
    drop(unsafe { Box::from_raw(qi) });
}

/// Drop the reference `pqi` holds to a location list stack, clearing it.
///
/// # Safety
///
/// `*pqi` must be null or a live location list stack.
pub(crate) unsafe fn ll_free_all(pqi: *mut *mut qf_info_T) {
    // SAFETY: forwarded from the caller.
    let qi = unsafe { *pqi };
    if qi.is_null() {
        return;
    }
    unsafe { *pqi = ptr::null_mut() };
    if quickfix_busy.get() > 0 {
        PENDING_FREE.with_mut(|pending| pending.push(qi));
        return;
    }
    unsafe { ll_release(qi) };
}

/// Drop one reference, freeing the stack at the last.
///
/// # Safety
///
/// `qi` must be a live location list stack.
unsafe fn ll_release(qi: *mut qf_info_T) {
    // SAFETY: the caller's promise -- a live `qf_info_T`.
    let mut qi = unsafe { Qi::new(qi) };
    // SAFETY: forwarded from the caller.
    qi.qf_refcount -= 1;
    if qi.qf_refcount < 1 {
        unsafe { wipe_qf_buffer(qi.raw()) };
        unsafe { qf_free_lists(qi.raw()) };
    }
}

/// Free the lists a window's location list stacks hold, or — for a null
/// window — those of the quickfix stack.
///
/// # Safety
///
/// `wp` must be null or a live window.
pub unsafe fn qf_free_all(wp: *mut win_T) {
    // SAFETY: forwarded from the caller.
    if !wp.is_null() {
        unsafe { ll_free_all(&raw mut (*wp).w_llist) };
        unsafe { ll_free_all(&raw mut (*wp).w_llist_ref) };
    } else {
        unsafe { qf_free_list_stack_items(QfStack::Global.raw()) };
    }
}

/// Note that a stack pointer is being held across code that can fire
/// autocommands. Must be paired with exactly one [`decr_quickfix_busy`].
pub(crate) fn incr_quickfix_busy() {
    quickfix_busy.set(quickfix_busy.get() + 1);
}

/// Release the hold, and free whatever asked to be freed meanwhile.
pub(crate) unsafe fn decr_quickfix_busy() {
    quickfix_busy.set(quickfix_busy.get() - 1);
    if quickfix_busy.get() != 0 {
        return;
    }
    // Freeing one wipes a buffer, which fires autocommands that may queue
    // another; taking the newest each time round is upstream's
    // pop-from-the-head loop.
    while let Some(qi) = PENDING_FREE.with_mut(Vec::pop) {
        // SAFETY: nothing but `ll_free_all` queues, and it queues a stack
        // it has just removed the last reachable reference to.
        unsafe { ll_release(qi) };
    }
}

/// Room for `n` lists, all unused.
fn qf_alloc_list_stack(n: c_int) -> Vec<qf_list_T> {
    debug_assert!(n >= 0);
    vec![empty_list(); n.max(0) as usize]
}

/// A new location list stack with room for `n` lists, holding the one
/// reference its caller is about to store.
///
/// Never the quickfix stack: that one is [`QfStack::Global`], a static that
/// exists before this module is first entered and that [`qf_init_stack`]
/// only gives its slots to.
pub(crate) fn qf_alloc_stack(qfltype: qfltype_T, n: c_int) -> *mut qf_info_T {
    debug_assert_ne!(qfltype, QFLT_QUICKFIX);
    let mut stack = Box::new(qf_info_T::new(qfltype));
    stack.qf_refcount = 1;
    stack.qf_bufnr = INVALID_QFBUFNR;
    stack.qf_lists = qf_alloc_list_stack(n);
    Box::into_raw(stack)
}

/// Give the quickfix stack its `'chistory'` slots. Called once, during
/// startup; the stack itself is a static and needs no allocating.
pub fn qf_init_stack() {
    let n = p_chi.get() as c_int;
    // A leaf closure over one static: nothing it calls can re-enter the
    // cell, which is what lets this be an exclusive borrow at all.
    ql_info_actual.with_mut(|qi| {
        qi.qf_bufnr = INVALID_QFBUFNR;
        qi.qf_lists = qf_alloc_list_stack(n);
    });
}

/// Give the quickfix stack room for `n` lists (`'chistory'`).
pub fn qf_resize_stack(n: c_int) {
    // SAFETY: the quickfix stack is a static, so it is always live -- which
    // is the whole of `qf_resize_stack_base`'s precondition.
    unsafe { qf_resize_stack_base(QfStack::Global.raw(), n) };
}

/// Give a window's location list stack room for `n` lists (`'lhistory'`).
///
/// # Safety
///
/// `wp` must be a live window.
pub unsafe fn ll_resize_stack(wp: *mut win_T, n: c_int) {
    // SAFETY: forwarded from the caller.
    // A location list window and the window it belongs to share the
    // stack, so whichever of them was set must tell the other.
    if unsafe { is_ll_window(wp) } {
        unsafe { qf_sync_llw_to_win(wp) };
    } else {
        unsafe { qf_sync_win_to_llw(wp) };
    }
    unsafe { qf_resize_stack_base(ll_get_or_alloc_list(wp), n) };
}

/// Resize a stack, dropping the oldest lists if they no longer fit.
///
/// # Safety
///
/// `qi` must be a live stack.
unsafe fn qf_resize_stack_base(qi: *mut qf_info_T, n: c_int) {
    // SAFETY: the caller's promise -- a live `qf_info_T`.
    let mut qi = unsafe { Qi::new(qi) };
    // SAFETY: forwarded from the caller.
    let max = (*qi).max_count();
    if n == max {
        return;
    }
    if n < max && n < qi.qf_listcount {
        for _ in 0..qi.qf_listcount - n {
            unsafe { qf_pop_stack(qi.raw(), true) };
        }
    }
    qi.qf_lists.resize(n.max(0) as usize, empty_list());
    qf_redraw(qi, ptr::null_mut());
}

/// Copy a location list window's `'lhistory'` to the window it belongs to.
///
/// # Safety
///
/// `llw` must be a live location list window.
unsafe fn qf_sync_llw_to_win(llw: *mut win_T) {
    // SAFETY: forwarded from the caller.
    let wp = unsafe { qf_find_win_with_loclist((*llw).w_llist_ref) };
    if !wp.is_null() {
        unsafe { (*wp).w_onebuf_opt.wo_lhi = (*llw).w_onebuf_opt.wo_lhi };
    }
}

/// Copy a window's `'lhistory'` to its location list window, if it has one.
///
/// # Safety
///
/// `pwp` must be a live window.
unsafe fn qf_sync_win_to_llw(pwp: *mut win_T) {
    // SAFETY: forwarded from the caller.
    let llw = unsafe { (*pwp).w_llist };
    if llw.is_null() {
        return;
    }
    let mut wp = firstwin.get();
    while !wp.is_null() {
        if unsafe { (*wp).w_llist_ref } == llw && unsafe { bt_quickfix((*wp).w_buffer) } {
            unsafe { (*wp).w_onebuf_opt.wo_lhi = (*pwp).w_onebuf_opt.wo_lhi };
            return;
        }
        wp = unsafe { (*wp).w_next };
    }
}

/// The location list stack for a window, allocating one if it has none.
///
/// # Safety
///
/// `wp` must be a live window.
pub(crate) unsafe fn ll_get_or_alloc_list(wp: *mut win_T) -> *mut qf_info_T {
    // SAFETY: the caller's promise -- a live `win_T`.
    let mut wp = unsafe { Win::new(wp) };
    // SAFETY: forwarded from the caller.
    if unsafe { is_ll_window(wp.raw().cast_const()) } {
        return wp.w_llist_ref;
    }
    // A window that is not a location list window has no business
    // referencing someone else's list.
    unsafe { ll_free_all(&raw mut wp.w_llist_ref) };
    if wp.w_llist.is_null() {
        wp.w_llist = qf_alloc_stack(QFLT_LOCATION, wp.w_onebuf_opt.wo_lhi as c_int);
    }
    wp.w_llist
}

/// The stack an Ex command works on. For a location list command that is
/// the current window's, and there may be none — reported as E776 when
/// `print_emsg`.
///
/// # Safety
///
/// `eap` must be a live command.
pub(crate) unsafe fn qf_cmd_get_stack(eap: *mut exarg_T, print_emsg: bool) -> *mut qf_info_T {
    // SAFETY: the caller's promise -- a live `exarg_T`.
    let eap = unsafe { Ea::new(eap) };
    // SAFETY: forwarded from the caller.
    if !unsafe { is_loclist_cmd(eap.cmdidx as c_int) } {
        return QfStack::Global.raw();
    }
    let qi = unsafe { win_loclist(curwin.get()) };
    if qi.is_null() && print_emsg {
        qf_emsg(&raw const e_loclist as *const c_char);
    }
    qi
}

/// The stack an Ex command works on, allocating a location list stack for
/// the current window if it has none. Never null.
///
/// # Safety
///
/// `eap` must be a live command and `pwinp` writable.
pub(crate) unsafe fn qf_cmd_get_or_alloc_stack(
    eap: *const exarg_T,
    pwinp: *mut *mut win_T,
) -> *mut qf_info_T {
    // SAFETY: the caller's promise -- a live `exarg_T`.
    let eap = unsafe { Ea::new(eap.cast_mut()) };
    // SAFETY: forwarded from the caller.
    if !unsafe { is_loclist_cmd(eap.cmdidx as c_int) } {
        return QfStack::Global.raw();
    }
    unsafe { *pwinp = curwin.get() };
    unsafe { ll_get_or_alloc_list(curwin.get()) }
}

/// The index of the list with the given id, or `INVALID_QFIDX`.
///
/// # Safety
///
/// `qi` must be a live stack.
pub(crate) unsafe fn qf_id2nr(qi: *const qf_info_T, qfid: ::core::ffi::c_uint) -> c_int {
    // SAFETY: the caller's promise -- a live `qf_info_T`.
    let mut qi = unsafe { Qi::new(qi.cast_mut()) };
    // SAFETY: forwarded from the caller.
    let count = qi.qf_listcount as usize;
    // SAFETY: as above; the borrow is dropped before the caller can touch
    // the stack again.
    let lists = &qi.qf_lists;
    for (idx, list) in lists[..count].iter().enumerate() {
        if list.qf_id == qfid {
            return idx as c_int;
        }
    }
    INVALID_QFIDX
}

/// Make the list with the given id current again, after autocommands may
/// have pushed others. Answers `FAIL` when it is gone.
///
/// # Safety
///
/// `qi` must be a live stack.
pub(crate) unsafe fn qf_restore_list(qi: *mut qf_info_T, save_qfid: ::core::ffi::c_uint) -> c_int {
    // SAFETY: the caller's promise -- a live `qf_info_T`.
    let mut qi = unsafe { Qi::new(qi) };
    // SAFETY: forwarded from the caller.
    if unsafe { (*qf_get_curlist(qi.raw())).qf_id } == save_qfid {
        return OK;
    }
    let curlist = unsafe { qf_id2nr(qi.raw().cast_const(), save_qfid) };
    if curlist < 0 {
        return FAIL;
    }
    qi.qf_curlist = curlist;
    OK
}

/// Copy a window's location list stack to another window, list by list.
///
/// # Safety
///
/// Both windows must be live, and `to` must have no location list yet.
pub unsafe fn copy_loclist_stack(from: *mut win_T, to: *mut win_T) {
    // SAFETY: forwarded from the caller.
    let qi = unsafe { win_loclist(from) };
    if qi.is_null() {
        return;
    }
    let copy = qf_alloc_stack(QFLT_LOCATION, unsafe { (*from).w_onebuf_opt.wo_lhi }
        as c_int);
    unsafe { (*to).w_llist = copy };
    unsafe { (*to).w_onebuf_opt.wo_lhi = (*copy).max_count() as OptInt };
    unsafe { (*copy).qf_listcount = (*qi).qf_listcount };
    for idx in 0..unsafe { (*qi).qf_listcount } {
        unsafe { (*copy).qf_curlist = idx };
        unsafe { copy_loclist(qf_get_list(qi, idx), qf_get_list(copy, idx)) };
    }
    unsafe { (*copy).qf_curlist = (*qi).qf_curlist };
}

/// Throw away every list in a stack, and give a location list window that
/// was showing it a fresh empty stack to show.
///
/// # Safety
///
/// `qi` must be a live stack and `wp` null or a live window.
pub(crate) unsafe fn qf_free_stack(mut wp: *mut win_T, qi: *mut qf_info_T) {
    // SAFETY: the caller's promise -- a live `qf_info_T`.
    let mut qi = unsafe { Qi::new(qi) };
    // SAFETY: forwarded from the caller.
    let qfwin = unsafe { qf_find_win(qi.raw().cast_const()) };
    if !qfwin.is_null() {
        if qi.qf_curlist < qi.qf_listcount {
            unsafe { qf_free(qf_get_curlist(qi.raw())) };
        }
        qf_redraw(qi, ptr::null_mut());
    }
    if !wp.is_null() && unsafe { is_ll_window(wp) } {
        // Prefer the window the location list belongs to over the
        // location list window showing it.
        let llwin = unsafe { qf_find_win_with_loclist(qi.raw().cast_const()) };
        if !llwin.is_null() {
            wp = llwin;
        }
    }
    unsafe { qf_free_all(wp) };
    if wp.is_null() {
        qi.qf_curlist = 0;
        qi.qf_listcount = 0;
    } else if !qfwin.is_null() {
        let new_ll = qf_alloc_stack(QFLT_LOCATION, unsafe { (*wp).w_onebuf_opt.wo_lhi } as c_int);
        unsafe { (*new_ll).qf_bufnr = (*(*qfwin).w_buffer).handle as c_int };
        unsafe { ll_free_all(&raw mut (*qfwin).w_llist_ref) };
        unsafe { (*qfwin).w_llist_ref = new_ll };
        if wp != qfwin {
            unsafe { win_set_loclist(wp, new_ll) };
        }
    }
}

/// The window the editor is working in.
///
/// The whole family shares this one rather than each file keeping its own:
/// the promise `Win::current` wants is the same everywhere, and paying it
/// once is the point of the exercise.
pub(crate) fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// The buffer the editor is working in — see [`cur_win`].
pub(crate) fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The global variant names the static; anything else is a `Local`, and
    /// the two are told apart without either one being dereferenced.
    #[test]
    fn the_static_is_the_only_global_stack() {
        assert_eq!(QfStack::of(QfStack::Global.raw()), QfStack::Global);

        let mut elsewhere = qf_info_T::new(QFLT_LOCATION);
        let other = &raw mut elsewhere;
        assert_eq!(QfStack::of(other), QfStack::Local(other));
    }

    /// `qf_alloc_stack` hands back a stack holding the one reference its
    /// caller is about to store, with its slots and no lists in them.
    #[test]
    fn a_new_location_list_stack_holds_one_reference() {
        let stack = qf_alloc_stack(QFLT_LOCATION, 3);
        assert_eq!(QfStack::of(stack), QfStack::Local(stack));
        // SAFETY: `qf_alloc_stack` leaked the box a statement ago and the
        // pointer has not left this test, so this is the last reference.
        let owned = unsafe { Box::from_raw(stack) };
        assert_eq!(owned.qf_refcount, 1);
        assert_eq!(owned.qfl_type, QFLT_LOCATION);
        assert_eq!(owned.qf_bufnr, INVALID_QFBUFNR);
        assert_eq!(owned.qf_listcount, 0);
        assert_eq!(owned.max_count(), 3);
    }

    /// Dropping the oldest list moves the rest down a slot and leaves the
    /// top one empty -- upstream's `memmove`, without the `Copy` bound.
    #[test]
    fn dropping_the_oldest_list_shifts_the_rest_down() {
        let mut lists = qf_alloc_list_stack(4);
        for (nr, list) in lists.iter_mut().enumerate() {
            list.qf_id = nr as c_uint + 1;
        }
        drop_oldest_list(&mut lists, 3);
        let ids: Vec<c_uint> = lists.iter().map(|list| list.qf_id).collect();
        assert_eq!(ids, vec![2, 3, 0, 4]);
    }
}
