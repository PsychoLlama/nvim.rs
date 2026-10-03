//! The stacks of lists, who owns them, and which one a command works on.
//!
//! There is exactly one quickfix stack, and one location list stack per
//! window that has asked for one. Every stack is owned by one table here,
//! [`QF_STACKS`], and everything else names a stack by its [`QfId`]: a
//! window holds its location list stack's id in `w_llist`, and the location
//! list window showing that stack holds the same id in `w_llist_ref`. A
//! location list stack is reference counted, because either window may be
//! closed first, and is removed from the table at its last reference.
//!
//! The quickfix code works on a stack through a [`Qi`]: the id and a view
//! of the stack at its fixed address, whose fields are ordinary reads and
//! writes. A list is a [`Qfl`]: the stack's view and the list's slot. Both
//! are views, not borrows, because most quickfix commands run autocommands
//! or user functions between finding a list and finishing with it, and an
//! autocommand can reach the same stack — read it with `getqflist()`, add a
//! list with `setqflist()`, or close the window whose stack it is. So:
//!
//! - nothing holds a `&mut QfStack` or `&mut QfList` across a call that can
//!   run user code; the leaf functions that take one run none;
//! - a command that holds a stack across such a call holds a
//!   [`QuickfixBusy`], which defers freeing a stack whose last reference
//!   goes meanwhile, so the view it holds stays good;
//! - what it does with the stack afterwards it checks first, by list id and
//!   change tick, the way upstream does.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::id_table::IdTable;
use crate::types::{CmdIdx, Failed, QfId, Refcount};
use crate::winlayer::{Buf, Live, Win, windows};
use core::ffi::{CStr, c_int, c_uint};
use core::ops::{Deref, DerefMut};

/// Every stack, and what the table's owner keeps beside them.
struct QfStacks {
    table: IdTable<QfStack>,
    /// The quickfix stack, made the first time anything asks for it.
    global: Option<QfId>,
    /// How many commands are holding a stack across code that can fire
    /// autocommands. While this is above zero, a stack's last reference
    /// going does not free it: the id goes to `pending_free` instead.
    busy: c_int,
    /// Location list stacks whose last reference went while busy, newest
    /// last.
    pending_free: Vec<QfId>,
}

static QF_STACKS: GlobalCell<QfStacks> = GlobalCell::new(QfStacks {
    table: IdTable::new(),
    global: None,
    busy: 0,
    pending_free: Vec::new(),
});

/// The id the next list created is given. Ids are never reused, so a caller
/// that saved one can tell whether the list it saw is still there.
static last_qf_id: GlobalCell<c_uint> = GlobalCell::new(0);

/// A fresh list id.
pub(crate) fn next_list_id() -> c_uint {
    let id = last_qf_id.get().wrapping_add(1);
    last_qf_id.set(id);
    id
}

/// One stack: its id, and a view of it at the fixed address the table keeps
/// it at.
///
/// A view, so that a `Qi` may be held across a call that runs user code —
/// under a [`QuickfixBusy`], which keeps the stack from being freed — and
/// every field access is a fresh, momentary borrow. Two are equal when they
/// name the same stack.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Qi {
    id: QfId,
    live: Live<QfStack>,
}

impl Deref for Qi {
    type Target = QfStack;

    fn deref(&self) -> &QfStack {
        &self.live
    }
}

impl DerefMut for Qi {
    fn deref_mut(&mut self) -> &mut QfStack {
        &mut self.live
    }
}

impl QfId {
    /// The stack this id names.
    ///
    /// # Panics
    /// When the stack has been freed.
    pub(crate) fn stack(self) -> Qi {
        Qi {
            id: self,
            live: QF_STACKS.with(|stacks| stacks.table.view(self)),
        }
    }
}

impl Qi {
    /// The quickfix stack.
    pub(crate) fn global() -> Qi {
        global_id().stack()
    }

    pub(crate) fn id(self) -> QfId {
        self.id
    }

    /// The list in slot `idx`, which must be one the stack has room for.
    pub(crate) fn slot(self, idx: c_int) -> Qfl {
        debug_assert!(idx >= 0 && idx < self.max_count(), "a list slot");
        Qfl { qi: self, idx }
    }

    /// The list `:cc` and friends work on.
    pub(crate) fn current_slot(self) -> Qfl {
        self.slot(self.current)
    }
}

/// One list: its stack, and its slot there.
///
/// A view for the same reason as [`Qi`]. A command that ran user code
/// re-checks the list by its id before trusting that the slot still holds
/// the list it started on.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Qfl {
    qi: Qi,
    idx: c_int,
}

impl Deref for Qfl {
    type Target = QfList;

    fn deref(&self) -> &QfList {
        self.qi.live.list(self.idx)
    }
}

impl DerefMut for Qfl {
    fn deref_mut(&mut self) -> &mut QfList {
        self.qi.live.list_mut(self.idx)
    }
}

impl Qfl {
    /// The stack the list is on.
    pub(crate) fn stack(self) -> Qi {
        self.qi
    }
}

/// The quickfix stack's id, making the stack if it does not exist yet.
fn global_id() -> QfId {
    if let Some(id) = QF_STACKS.with(|stacks| stacks.global) {
        return id;
    }
    let mut stack = QfStack::new(QFLT_QUICKFIX);
    stack.bufnr = INVALID_QFBUFNR;
    QF_STACKS.with_mut(|stacks| {
        let (id, _) = stacks.table.insert((), stack);
        stacks.global = Some(id);
        id
    })
}

/// A command holding a stack across code that can fire autocommands.
///
/// While one is held, a location list stack whose last reference goes is
/// not freed but queued, and the queue is emptied when the last hold ends —
/// which can itself fire autocommands, as freeing a stack wipes its buffer.
pub(crate) struct QuickfixBusy(());

impl QuickfixBusy {
    pub(crate) fn hold() -> QuickfixBusy {
        QF_STACKS.with_mut(|stacks| stacks.busy += 1);
        QuickfixBusy(())
    }
}

impl Drop for QuickfixBusy {
    fn drop(&mut self) {
        let left = QF_STACKS.with_mut(|stacks| {
            stacks.busy -= 1;
            stacks.busy
        });
        if left != 0 {
            return;
        }
        // Freeing one wipes a buffer, which fires autocommands that may queue
        // another; taking the newest each time round is upstream's
        // pop-from-the-head loop.
        while let Some(id) = QF_STACKS.with_mut(|stacks| stacks.pending_free.pop()) {
            release(id);
        }
    }
}

/// `emsg(_(msg))`: report an error whose text is a static C string.
pub(crate) fn qf_emsg(msg: &'static CStr) {
    emsg(gettext(msg));
}

/// Fire `QuickFixCmdPre`/`QuickFixCmdPost` for a quickfix command, and say
/// whether an autocommand claimed the event.
///
/// `on_fname` is upstream's split: the commands that read a file or run a
/// program match the pattern against the current buffer's name and force the
/// event, the ones taking their input from Vimscript match on neither.
pub(crate) fn fire_qf_autocmd(event: AutoEvent, name: &CStr, on_fname: bool) -> bool {
    let fname = if on_fname {
        Buf::current().name.shown().map(CStr::to_owned)
    } else {
        None
    };
    fire_autocmds_for(
        event,
        Some(name),
        fname.as_deref(),
        on_fname,
        Buf::current_or_none(),
    )
}

// ---------------------------------------------------------------------------
// Which stack a window or a command works on.

impl Win {
    /// Whether the window shows a quickfix or location list buffer, without
    /// saying which: C's bare `bt_quickfix(wp->w_buffer)`.
    pub(crate) fn shows_quickfix_buffer(self) -> bool {
        buf_is_quickfix(self.buffer_or_none())
    }

    /// Whether the window *is* a location list window: one showing another
    /// window's location list rather than owning one.
    pub(crate) fn is_location_list_window(self) -> bool {
        self.shows_quickfix_buffer() && self.w_llist_ref.is_some()
    }

    /// Whether the window is the quickfix window.
    pub(crate) fn is_quickfix_window(self) -> bool {
        self.shows_quickfix_buffer() && self.w_llist_ref.is_none()
    }

    /// The location list stack the window works on: the one it shows when
    /// it is a location list window, otherwise its own.
    pub(crate) fn location_list(self) -> Option<QfId> {
        if self.is_location_list_window() {
            self.w_llist_ref
        } else {
            self.w_llist
        }
    }

    /// [`location_list`](Win::location_list), making the window one if it
    /// has none.
    pub(crate) fn location_list_or_new(mut self) -> Qi {
        if self.is_location_list_window() {
            return self
                .w_llist_ref
                .expect("a location list window shows a stack")
                .stack();
        }
        // A window that is not a location list window has no business
        // referencing someone else's list.
        let shown = self.w_llist_ref.take();
        drop_stack_ref(shown);
        let own = match self.w_llist {
            Some(id) => id,
            None => {
                let id = new_location_stack(QFLT_LOCATION, self.w_onebuf_opt.wo_lhi);
                self.w_llist = Some(id);
                id
            }
        };
        own.stack()
    }

    /// Give the window `qi` as its location list, taking a reference.
    pub(crate) fn set_location_list(mut self, mut qi: Qi) {
        debug_assert!(self.w_llist.is_none(), "the window already holds a list");
        self.w_llist = Some(qi.id());
        qi.refcount.retain();
    }

    /// Drop the window's references to its own location list stack and the
    /// one it shows.
    pub(crate) fn free_location_lists(mut self) {
        let own = self.w_llist.take();
        drop_stack_ref(own);
        let shown = self.w_llist_ref.take();
        drop_stack_ref(shown);
    }

    /// Give `to` a copy of this window's location list stack, list by list.
    /// `to` must have no location list yet.
    pub(crate) fn copy_location_lists_to(self, mut to: Win) {
        let Some(from) = self.location_list() else {
            return;
        };
        let qi = from.stack();
        let id = new_location_stack(QFLT_LOCATION, self.w_onebuf_opt.wo_lhi);
        to.w_llist = Some(id);
        let mut copy = id.stack();
        to.w_onebuf_opt.wo_lhi = OptInt::from(copy.max_count());
        copy.list_count = qi.list_count;
        for idx in 0..qi.list_count {
            // Two stacks, so the borrows are of two different objects; and
            // copying runs nothing that could reach either.
            copy_loclist(&qi.slot(idx), &mut copy.slot(idx));
        }
        copy.current = qi.current;
    }

    /// Give the window's location list stack room for `n` lists
    /// (`'lhistory'`).
    pub(crate) fn resize_location_list_stack(self, n: c_int) {
        // A location list window and the window it belongs to share the
        // stack, so whichever of them was set must tell the other.
        if self.is_location_list_window() {
            sync_lhistory_to_owner(self);
        } else {
            sync_lhistory_to_window(self);
        }
        resize_stack(self.location_list_or_new(), n);
    }
}

/// Whether `window` shows a help file.
pub(crate) fn is_help_buffer(window: Win) -> bool {
    buf_is_help(window.buffer_or_none())
}

/// Whether `window` shows an ordinary file.
pub(crate) fn is_normal_buffer(window: Win) -> bool {
    buf_is_normal(window.buffer_or_none())
}

/// The stack an Ex command works on. For a location list command that is
/// the current window's, and there may be none — reported as E776 when
/// `print_emsg`.
///
/// The command is named by its `cmdidx` alone, which is all that decides
/// between the two stacks -- so an address that asks a quickfix question
/// can ask it while `get_address` still holds the command line.
pub(crate) fn stack_for_cmd(cmdidx: CmdIdx, print_emsg: bool) -> Option<Qi> {
    if !is_loclist_cmd(cmdidx) {
        return Some(Qi::global());
    }
    let qi = Win::current().location_list();
    if qi.is_none() && print_emsg {
        qf_emsg(e_loclist);
    }
    qi.map(QfId::stack)
}

/// The stack an Ex command works on, making a location list stack for the
/// current window if it has none.
///
/// The window comes back with it: a location list command works on the
/// current window's stack and the caller has to know whose it was, while a
/// quickfix command works on the global one and answers `None`.
pub(crate) fn stack_or_new_for_cmd(excmd: &ExArg) -> (Qi, Option<Win>) {
    if !is_loclist_cmd(excmd.cmdidx) {
        return (Qi::global(), None);
    }
    let wp = Win::current();
    (wp.location_list_or_new(), Some(wp))
}

/// The stack `window` works on, or the quickfix stack for `None`. `None`
/// when the window has no location list — or is gone: `window` may have
/// been saved before an autocommand that closed it.
pub(crate) fn stack_of(window: Option<Win>) -> Option<Qi> {
    match window {
        None => Some(Qi::global()),
        Some(wp) if win_valid(wp.id()) => wp.location_list().map(QfId::stack),
        Some(_) => None,
    }
}

/// A window that is not a quickfix window and owns this location list.
pub(crate) fn qf_find_win_with_loclist(ll: QfId) -> Option<Win> {
    windows().find(|wp| wp.w_llist == Some(ll) && !wp.shows_quickfix_buffer())
}

/// Copy a location list window's `'lhistory'` to the window it belongs to.
fn sync_lhistory_to_owner(llw: Win) {
    let owner = llw.w_llist_ref.and_then(qf_find_win_with_loclist);
    if let Some(mut wp) = owner {
        wp.w_onebuf_opt.wo_lhi = llw.w_onebuf_opt.wo_lhi;
    }
}

/// Copy a window's `'lhistory'` to its location list window, if it has one.
fn sync_lhistory_to_window(owner: Win) {
    let Some(ll) = owner.w_llist else {
        return;
    };
    if let Some(mut wp) =
        windows().find(|wp| wp.w_llist_ref == Some(ll) && wp.shows_quickfix_buffer())
    {
        wp.w_onebuf_opt.wo_lhi = owner.w_onebuf_opt.wo_lhi;
    }
}

// ---------------------------------------------------------------------------
// Making, resizing and freeing stacks.

/// Room for `n` lists, all unused. `n` is an option's value
/// (`'chistory'`/`'lhistory'`), which the option keeps within 1..=100.
fn new_slots(n: OptInt) -> Vec<QfList> {
    debug_assert!(n >= 0);
    (0..n.max(0)).map(|_| QfList::new()).collect()
}

/// A new location list stack with room for `n` lists, holding the one
/// reference its caller is about to store — or, with `QFLT_INTERNAL`, the
/// throwaway stack `getqflist({'lines': …})` parses into.
pub(crate) fn new_location_stack(kind: QfListType, n: OptInt) -> QfId {
    debug_assert_ne!(kind, QFLT_QUICKFIX);
    let mut stack = QfStack::new(kind);
    stack.refcount = Refcount::ONE;
    stack.bufnr = INVALID_QFBUFNR;
    stack.lists = new_slots(n);
    QF_STACKS.with_mut(|stacks| stacks.table.insert((), stack).0)
}

/// Give the quickfix stack its `'chistory'` slots. Called once, during
/// startup.
pub fn qf_init_stack() {
    let mut qi = Qi::global();
    qi.bufnr = INVALID_QFBUFNR;
    qi.lists = new_slots(p_chi());
}

/// Give the quickfix stack room for `n` lists (`'chistory'`).
pub fn qf_resize_stack(n: c_int) {
    resize_stack(Qi::global(), n);
}

/// Resize a stack, dropping the oldest lists if they no longer fit.
fn resize_stack(mut qi: Qi, n: c_int) {
    let max = qi.max_count();
    if n == max {
        return;
    }
    if n < max && n < qi.list_count {
        for _ in 0..qi.list_count - n {
            pop_stack(qi, true);
        }
    }
    let n = usize::try_from(n.max(0)).expect("clamped above");
    // Lists past the new end go, and their values with them.
    let dropped = if n < qi.lists.len() {
        qi.lists.split_off(n)
    } else {
        qi.lists.resize_with(n, QfList::new);
        Vec::new()
    };
    drop(dropped);
    qf_update_buffer(qi, None);
}

/// Drop the oldest list and shuffle the rest down, leaving an unused slot
/// at the top.
///
/// With `adjust`, the stack also shrinks and the current list follows the
/// list it pointed at — or, if that was the one dropped, the newest.
pub(crate) fn pop_stack(mut qi: Qi, adjust: bool) {
    let count = usize::try_from(qi.list_count).expect("a list count is never negative");
    let oldest = qi.lists.remove(0);
    qi.lists.insert(count - 1, QfList::new());
    drop(oldest);
    if adjust {
        qi.list_count -= 1;
        qi.current = if qi.current == 0 {
            qi.list_count - 1
        } else {
            qi.current - 1
        };
    }
}

/// The buffer the quickfix window shows, or `INVALID_QFBUFNR`.
pub fn qf_stack_get_bufnr() -> c_int {
    Qi::global().bufnr
}

/// Wipe the quickfix window's buffer, if it is not displayed anywhere.
fn wipe_qf_buffer(mut qi: Qi) {
    if qi.bufnr == INVALID_QFBUFNR {
        return;
    }
    let Some(qfbuf) = find_buf(qi.bufnr).filter(|b| b.b_nwindows == 0) else {
        return;
    };
    // `close_buffer` insists that `curwin->w_buffer == curbuf`, and it
    // may not: this is reachable from `win_free_mem` after `win_close`
    // already released the current window's buffer.
    let buf_was_null = Win::current().w_buffer.is_null();
    if buf_was_null {
        Win::current().w_buffer = Buf::current_or_none().unwrap_or(Buf::NULL);
    }
    close_buffer(None, qfbuf, DOBUF_WIPE.cast_signed(), false, false);
    qi.bufnr = INVALID_QFBUFNR;
    if buf_was_null {
        Win::current().w_buffer = Buf::NULL;
    }
}

/// Drop one reference to a location list stack — the one a window slot
/// held, which the caller has just emptied. While a command is
/// [busy](QuickfixBusy) the release waits for it to finish.
pub(crate) fn drop_stack_ref(id: Option<QfId>) {
    let Some(id) = id else {
        return;
    };
    let queued = QF_STACKS.with_mut(|stacks| {
        if stacks.busy > 0 {
            stacks.pending_free.push(id);
        }
        stacks.busy > 0
    });
    if !queued {
        release(id);
    }
}

/// Drop one reference, freeing the stack at the last.
fn release(id: QfId) {
    let mut qi = id.stack();
    if qi.refcount.release() < 1 {
        wipe_qf_buffer(qi);
        free_stack(id);
    }
}

/// Take a location list stack out of the table and free it, lists and all.
fn free_stack(id: QfId) {
    debug_assert!(QF_STACKS.with(|stacks| stacks.global != Some(id)));
    let stack = QF_STACKS.with_mut(|stacks| stacks.table.remove(id));
    // Dropped out here rather than in the table's cell: what the lists hold
    // is released as they go.
    drop(stack);
}

/// Free a stack no window ever held — the throwaway `QFLT_INTERNAL` one.
pub(crate) fn free_unreferenced_stack(id: QfId) {
    free_stack(id);
}

/// Free the lists of the quickfix stack, leaving the stack itself.
pub fn free_quickfix_lists() {
    let qi = Qi::global();
    for idx in 0..qi.list_count {
        qf_free(qi.slot(idx));
    }
}

/// Throw away every list in a stack, and give a location list window that
/// was showing it a fresh empty stack to show.
///
/// `window` is `None` for the quickfix stack, which belongs to no window.
pub(crate) fn qf_free_stack(window: Option<Win>, mut qi: Qi) {
    let qfwin = qf_find_win(qi);
    if qfwin.is_some() {
        if qi.current < qi.list_count {
            qf_free(qi.current_slot());
        }
        qf_update_buffer(qi, None);
    }
    let mut window = window;
    if window.is_some_and(Win::is_location_list_window) {
        // Prefer the window the location list belongs to over the
        // location list window showing it.
        window = qf_find_win_with_loclist(qi.id()).or(window);
    }
    let Some(wp) = window else {
        free_quickfix_lists();
        qi.current = 0;
        qi.list_count = 0;
        return;
    };
    wp.free_location_lists();
    if let Some(mut qfwin) = qfwin {
        let fresh = new_location_stack(QFLT_LOCATION, wp.w_onebuf_opt.wo_lhi);
        fresh.stack().bufnr = qfwin.buffer().handle;
        let shown = qfwin.w_llist_ref.take();
        drop_stack_ref(shown);
        qfwin.w_llist_ref = Some(fresh);
        if wp != qfwin {
            wp.set_location_list(fresh.stack());
        }
    }
}

// ---------------------------------------------------------------------------
// Finding a list again.

/// Make the list with the given id current again, after autocommands may
/// have pushed others. Answers `Err` when it is gone.
pub(crate) fn qf_restore_list(mut qi: Qi, save_qfid: c_uint) -> Result<(), Failed> {
    if qi.current_list().id == save_qfid {
        return Ok(());
    }
    let curlist = qi.find_list(save_qfid).ok_or(Failed)?;
    qi.current = curlist;
    Ok(())
}

/// Whether the list `qf_id` names is still on the window's stack — or, for
/// `None`, on the quickfix stack.
///
/// `window` may name a window that has since been closed; it is checked.
pub(crate) fn qflist_valid(window: Option<Win>, qf_id: c_uint) -> bool {
    let qi = match window {
        None => Some(Qi::global()),
        // By identity: `window` was saved before the autocommand that may
        // have closed it, and the handle it carries was read while it was
        // live.
        Some(wp) if win_valid(wp.id()) => wp.location_list().map(QfId::stack),
        Some(_) => None,
    };
    qi.is_some_and(|qi| qi.find_list(qf_id).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_cell::editor_state_lock;

    /// A new location list stack holds the one reference its caller is
    /// about to store, with its slots and no lists in them; dropping that
    /// reference frees it.
    #[test]
    fn a_new_location_list_stack_holds_one_reference() {
        let _held = editor_state_lock();
        let id = new_location_stack(QFLT_LOCATION, 3);
        let qi = id.stack();
        assert_eq!(qi.refcount, Refcount::ONE);
        assert_eq!(qi.kind, QFLT_LOCATION);
        assert_eq!(qi.bufnr, INVALID_QFBUFNR);
        assert_eq!(qi.list_count, 0);
        assert_eq!(qi.max_count(), 3);
        assert!(qi != Qi::global());
        drop_stack_ref(Some(id));
        assert!(QF_STACKS.with(|stacks| stacks.pending_free.is_empty()));
    }

    /// Dropping the oldest list moves the rest down a slot and leaves an
    /// unused one where the newest was.
    #[test]
    fn popping_the_stack_shifts_the_rest_down() {
        let _held = editor_state_lock();
        let id = new_location_stack(QFLT_LOCATION, 4);
        let mut qi = id.stack();
        for (nr, list) in qi.lists.iter_mut().enumerate() {
            list.id = c_uint::try_from(nr).unwrap() + 1;
        }
        qi.list_count = 3;
        qi.current = 2;
        pop_stack(qi, true);
        let ids: Vec<c_uint> = qi.lists.iter().map(|list| list.id).collect();
        assert_eq!(ids, vec![2, 3, 0, 4]);
        assert_eq!((qi.list_count, qi.current), (2, 1));
        drop_stack_ref(Some(id));
    }
}
