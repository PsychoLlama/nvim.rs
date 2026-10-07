//! The funccall's life, the function's life, and the GC roots.
//!
//! `create_funccal` / `cleanup_function_call` / `funccal_unref` own the
//! funccall's lifetime -- including the case where a closure, an escaped
//! `l:` or a returned `a:000` outlives the call that made it and the funccall
//! has to be kept for the collector. `func_ptr_ref`/`func_ptr_unref` and the
//! `func_clear*` group own a function's editor-visible life: the counts that
//! decide when it is cleared and taken out of the table. The `set_ref_in_*`
//! group is what the garbage collector calls to mark everything reachable
//! from a call in progress.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use core::cell::{Cell, RefCell};
use core::ffi::{CStr, c_int};
use core::mem::ManuallyDrop;
use core::ptr;
use std::rc::{Rc, Weak};

use super::*;
use crate::eval::gc::{RootId, unroot_dict, unroot_list};
use crate::eval::typval::{CallFrame, DictRef, ListRef, RemovedItem, tv_dict_alloc, tv_list_alloc};
use crate::eval::vars::list_dict_vars;
use crate::hashtab::{HT_INIT_SIZE, hash_reset};
use crate::types::{DictKey, HashItem, Refcount, ScopeDictItem, ScopeType};

/// A funccall's three scopes: `l:`, `a:` and `a:000`, with the entries that
/// name `l:` and `a:` themselves (what `l:` alone evaluates to).
///
/// Upstream embedded all three in the funccall, seeded with
/// `DO_NOT_FREE_CNT` so that nothing could free them, and compared the count
/// against that seed to tell whether one had escaped. Here they are ordinary
/// dictionaries and a list the funccall holds handles to, carrying the same
/// seed while a call is using them ([`FrameScopes::activate`]) so the same
/// comparison works. They are not in the collector's registry, as upstream's
/// embedded ones were not: a running or parked funccall marks them itself.
///
/// A funccall whose scopes nothing else holds gives them back emptied, and
/// the next call starts from them: a call then allocates nothing for its
/// scopes, where upstream allocated nothing because it embedded them.
pub(crate) struct FrameScopes {
    pub(crate) l_vars: DictRef,
    /// The entry `l:` evaluates to. It holds a counted reference of its own,
    /// which `Drop` gives back; vars/ reaches it through
    /// [`with_funccal_scope_entry`].
    l_vars_var: RefCell<ScopeDictItem>,
    pub(crate) a_vars: DictRef,
    a_vars_var: RefCell<ScopeDictItem>,
    pub(crate) a_list: ListRef,
    /// Whether the counts carry the `DO_NOT_FREE_CNT` seed.
    active: Cell<bool>,
    /// Emptied entries of earlier calls, for this call's. Boxed on
    /// purpose: a dictionary takes its items as boxes, and keeping the
    /// allocations is the point.
    #[allow(clippy::vec_box)]
    spare: RefCell<Vec<Box<DictItem>>>,
}

/// The holders a scope dictionary has besides any escape: the funccall's
/// handle and the scope's own entry.
const DICT_HOLDERS: c_int = 2;
/// The holders `a:000` has besides any escape: the funccall's handle. The
/// `a:000` entry names it uncounted, as upstream's did.
const LIST_HOLDERS: c_int = 1;

/// A scope dictionary and the entry that names it.
fn scope_dict(scope: ScopeType) -> (DictRef, RefCell<ScopeDictItem>) {
    let dict = tv_dict_alloc();
    {
        let d = dict.edit();
        // Out of the collector's registry: the funccall marks it.
        unroot_dict(d.dv_root);
        d.dv_root = RootId::NONE;
        d.dv_scope = scope;
    }
    let entry = ScopeDictItem(ManuallyDrop::new(DictItem {
        di_tv: TypVal::dict(Some(dict.clone())),
        di_lock: VarLock::Fixed,
        di_flags: DI_FLAGS_RO | DI_FLAGS_FIX,
        di_key: DictKey::EMPTY,
    }));
    (dict, RefCell::new(entry))
}

impl FrameScopes {
    /// Fresh scopes.
    fn new() -> FrameScopes {
        let (l_vars, l_vars_var) = scope_dict(VAR_DEF_SCOPE);
        let (a_vars, a_vars_var) = scope_dict(VAR_SCOPE);
        let a_list = tv_list_alloc(-1);
        {
            let l = a_list.edit();
            unroot_list(l.lv_root);
            l.lv_root = RootId::NONE;
        }
        FrameScopes {
            l_vars,
            l_vars_var,
            a_vars,
            a_vars_var,
            a_list,
            active: Cell::new(false),
            spare: RefCell::new(Vec::new()),
        }
    }

    /// Seed the counts for a call: from here until the call is done a count
    /// other than `DO_NOT_FREE_CNT` means something else holds the scope.
    fn activate(&self) {
        for dict in [&self.l_vars, &self.a_vars] {
            let d = dict.edit();
            d.dv_refcount = Refcount::new(DO_NOT_FREE_CNT);
            d.dv_lock = VarLock::Unlocked;
            d.dv_copy_id = 0;
        }
        let l = self.a_list.edit();
        l.lv_refcount = Refcount::new(DO_NOT_FREE_CNT);
        l.lv_lock = VarLock::Fixed;
        l.lv_copy_id = 0;
        self.active.set(true);
    }

    /// Give the seed back: what is left is the holders and any escape.
    fn deactivate(&self) {
        if !self.active.replace(false) {
            return;
        }
        for dict in [&self.l_vars, &self.a_vars] {
            dict.edit()
                .dv_refcount
                .release_many(DO_NOT_FREE_CNT - DICT_HOLDERS);
        }
        self.a_list
            .edit()
            .lv_refcount
            .release_many(DO_NOT_FREE_CNT - LIST_HOLDERS);
    }

    /// Whether anything besides the funccall holds one of the three.
    fn referenced(&self) -> bool {
        self.a_list.lv_refcount != Refcount::new(DO_NOT_FREE_CNT)
            || self.l_vars.dv_refcount != Refcount::new(DO_NOT_FREE_CNT)
            || self.a_vars.dv_refcount != Refcount::new(DO_NOT_FREE_CNT)
    }

    /// Whether these can start another call: nothing else holds them, and
    /// no Lua table mirrors them.
    fn reusable(&self) -> bool {
        self.l_vars.dv_refcount == Refcount::new(DICT_HOLDERS)
            && self.a_vars.dv_refcount == Refcount::new(DICT_HOLDERS)
            && self.a_list.lv_refcount == Refcount::new(LIST_HOLDERS)
            && self.l_vars.lua_table_ref == LUA_NOREF
            && self.a_vars.lua_table_ref == LUA_NOREF
            && self.a_list.lua_table_ref == LUA_NOREF
            && self.l_vars.watchers.is_empty()
            && self.a_vars.watchers.is_empty()
    }

    /// An entry for one of the scopes: read-only, holding `value` under
    /// `key`, built in an emptied one when there is one.
    pub(crate) fn entry(&self, key: &[u8], value: TypVal, lock: VarLock) -> Box<DictItem> {
        let alloc = u8::try_from(crate::eval::typval::DI_FLAGS_ALLOC).expect("a flag byte");
        let flags = alloc | DI_FLAGS_RO | DI_FLAGS_FIX;
        match self.spare.borrow_mut().pop() {
            Some(mut item) => {
                item.di_key = DictKey::new(key);
                item.di_flags = flags;
                item.di_lock = lock;
                item.di_tv = value;
                item
            }
            None => {
                let mut item = DictItem::boxed(key);
                item.di_flags = flags;
                item.di_lock = lock;
                item.di_tv = value;
                item
            }
        }
    }
}

impl Drop for FrameScopes {
    fn drop(&mut self) {
        self.deactivate();
        // The two scope entries' own references.
        for entry in [&mut self.l_vars_var, &mut self.a_vars_var] {
            drop(entry.get_mut().0.di_tv.take());
        }
    }
}

/// Empty `dict`, slot by slot, releasing each value -- or, when the values
/// are only named (an `a:` item names the caller's argument), giving each up
/// unreleased. Upstream's `vars_clear_ext`. The emptied items are kept in
/// `spare` for the next call's entries.
#[allow(clippy::vec_box)] // the boxes a dictionary hands back, kept
fn clear_scope(dict: &DictRef, release_values: bool, spare: &RefCell<Vec<Box<DictItem>>>) {
    if dict.dv_hashtab.ht_used == 0 {
        return;
    }
    dict.edit().lock_table();
    let mut slot = 0;
    while dict.dv_hashtab.ht_used > 0 && slot < dict.dv_hashtab.size() {
        if dict.dv_hashtab.slots()[slot].is_kept() {
            let removed = dict.edit().remove_at(slot);
            // Released with the borrow of the dictionary over: a value can
            // name the dictionary it was in.
            let value = match removed {
                RemovedItem::Allocated(mut item) => {
                    let value = item.di_tv.take();
                    let mut spare = spare.borrow_mut();
                    if spare.len() < SPARE_ITEMS {
                        spare.push(item);
                    }
                    value
                }
                RemovedItem::Embedded(value) => value,
            };
            if release_values {
                drop(value);
            } else {
                // Named, not owned: given up unreleased.
                core::mem::forget(value);
            }
        }
        slot += 1;
    }
    // Not unlocked first: the table is replaced or emptied whole, which
    // undoes the lock, and unlocking would only resize what is about to go.
    let table = &mut dict.edit().dv_hashtab;
    if table.size() == HT_INIT_SIZE {
        // Upstream's `hash_clear` + `hash_init`, without building a new
        // table to move over this one: the initial slots, emptied.
        table.slots_mut().fill(HashItem::EMPTY);
        table.ht_used = 0;
        table.ht_filled = 0;
        table.ht_changed = 0;
        table.ht_locked = 0;
    } else {
        hash_reset(table);
    }
}

/// How many emptied entries a set of scopes keeps for its next call.
const SPARE_ITEMS: usize = 24;

/// Empty `a:000`, giving the named items up unreleased.
fn disown_list(list: &ListRef) {
    list.edit().disown_items();
}

/// Free the funccall itself, having already dealt with what its scopes hold.
fn free_funccal(frame: &FuncCall) {
    // When garbage collecting, a funccall may be freed before the function
    // that references it, so clear the functions' scope. A function that
    // was redefined since may point at another funccall; leave it then.
    for func in frame.ufuncs.take() {
        if let Some(func) = func.upgrade()
            && func.scoped.get() == Some(frame.id)
        {
            func.scoped.set(None);
        }
    }
    // The reference `create_funccal` took. This is the *only* place it is
    // given back, which is why a funccall parked for the collector keeps its
    // function undeletable until then.
    func_ptr_unref(&frame.func);
    drop(release_funccal(frame.id));
}

/// Free a parked funccall and everything in it, once it is off the parked
/// list.
fn free_funccal_contents(frame: &FuncCall) {
    // All l: variables, then all a: variables, then the a:000 items.
    clear_scope(&frame.scopes.l_vars, true, &frame.scopes.spare);
    clear_scope(&frame.scopes.a_vars, true, &frame.scopes.spare);
    frame.scopes.a_list.edit().lv_items.clear();
    free_funccal(frame);
}

/// The last part of returning from a function: free the local scopes,
/// unless a closure, a returned `a:000` or an escaped `l:` is still using
/// them. Answers the scopes for the next call when they can be reused.
pub(crate) fn cleanup_function_call(frame: Rc<FuncCall>) {
    let scopes = &frame.scopes;
    let may_free_fc = frame.refcount.get() <= 0;
    set_current_fc(frame.id.caller());

    let mut free_fc = true;
    // Free all l: variables if not referred to.
    if may_free_fc && scopes.l_vars.dv_refcount == Refcount::new(DO_NOT_FREE_CNT) {
        clear_scope(&scopes.l_vars, true, &scopes.spare);
    } else {
        free_fc = false;
    }

    // If the a:000 list and the l: and a: dicts are not referenced and no
    // closure is using them, the funccall and what is in it can go.
    if may_free_fc && scopes.a_vars.dv_refcount == Refcount::new(DO_NOT_FREE_CNT) {
        clear_scope(&scopes.a_vars, false, &scopes.spare);
    } else {
        free_fc = false;
        // Make a copy of the a: variables, since that was not done above.
        let d = scopes.a_vars.edit();
        for item in d.items_mut() {
            let owned = item.di_tv.clone();
            item.di_tv.overwrite(owned);
        }
    }

    if may_free_fc && scopes.a_list.lv_refcount == Refcount::new(DO_NOT_FREE_CNT) {
        disown_list(&scopes.a_list);
    } else {
        free_fc = false;
        // Make a copy of the a:000 items, since that was not done above.
        scopes.a_list.edit().own_items();
    }

    if free_fc {
        free_funccal(&frame);
        frame.scopes.deactivate();
        if Rc::strong_count(&frame) == 1
            && Rc::weak_count(&frame) == 0
            && frame.scopes.reusable()
            && frame.defer.borrow().is_empty()
            && frame.ufuncs.borrow().is_empty()
        {
            give_spare_funccal(frame);
        }
        return;
    }

    // The funccall is still in use. This happens when returning "a:000",
    // assigning "l:" to a global variable, or defining a closure. Keep it
    // for the collector to free later.
    static made_copy: GlobalCell<c_int> = GlobalCell::new(0);
    park_funccal(frame.id);

    if want_garbage_collect.get() {
        // The collector is ready anyway; clear the count.
        made_copy.set(0);
    } else {
        made_copy.set(made_copy.get() + 1);
        if made_copy.get()
            >= c_int::try_from(4096 * 1024 / size_of::<FuncCall>()).unwrap_or(c_int::MAX)
        {
            // Four megabytes' worth of copies, which happens when a function
            // that references itself is called repeatedly. Ask for a
            // collection soon rather than grow without bound.
            made_copy.set(0);
            want_garbage_collect.set(true);
        }
    }
}

/// Drop a reference to the funccall `scope` names and free it when the last
/// one goes. `func` lets go of it either way.
pub(crate) fn funccal_unref(scope: Option<FcId>, func: &UserFunc, force: bool) {
    let Some(frame) = scope.and_then(FcId::try_funccall) else {
        return;
    };
    let left = frame.refcount.get() - 1;
    frame.refcount.set(left);
    let unused = if force {
        left <= 0
    } else {
        !fc_referenced(&frame)
    };
    if unused && unlink_parked_funccals(Sweep::First, |parked| parked.id == frame.id) {
        return;
    }
    for slot in frame.ufuncs.borrow_mut().iter_mut() {
        if ptr::eq(slot.as_ptr(), func) {
            *slot = Weak::new();
        }
    }
}

/// Free everything hanging off `func` -- its arguments, its defaults, its
/// body, its Lua reference and its profiling counters.
pub(crate) fn func_clear_items(func: &UserFunc) {
    *func.body.borrow_mut() = Rc::new(FuncBody::default());
    if func.has_flag(FuncFlags::LUAREF) {
        release_luaref(func.luaref.replace(LUA_NOREF));
    }
    let mut prof = func.prof.borrow_mut();
    prof.tml_count = Vec::new();
    prof.tml_total = Vec::new();
    prof.tml_self = Vec::new();
}

/// Free everything `func` holds, once.
fn func_clear(func: &UserFunc, force: bool) {
    if func.cleared.replace(true) {
        return;
    }
    func_clear_items(func);
    // Drop the reference on the scope this function closed over.
    funccal_unref(func.scoped.get(), func, force);
}

/// Clear `func` and take it out of the table. What is left of it goes with
/// its last holder.
pub(crate) fn func_clear_free(func: &UserFunc, force: bool) {
    func_clear(func, force);
    // Only remove it when not done already, otherwise a newer version of the
    // function would go.
    if !func.has_flag(FuncFlags::DELETED | FuncFlags::REMOVED) {
        remove_func(func.name().as_bytes());
    }
}

/// Start a call of `func`: file its funccall, make it the current one, and
/// take a counted reference to the function for as long as the funccall
/// lives.
pub(crate) fn create_funccal(func: &Rc<UserFunc>) -> Rc<FuncCall> {
    let spare = take_spare_funccal();
    let level = ex_nesting_level.get();
    let frame = adopt_funccal(current_fc_id(), |id| match spare {
        Some(mut frame) => {
            // Built in place: the funccall's fields are reset, its emptied
            // scopes kept.
            let reused = Rc::get_mut(&mut frame).expect("a spare funccall is unshared");
            reused.func = func.clone();
            reused.id = id;
            reused.level = level;
            reused.scope_ready.set(false);
            reused.linenr.set(0);
            reused.returned.set(false);
            reused.breakpoint.set(0);
            reused.dbg_tick.set(0);
            reused.prof_child.set(0);
            reused.refcount.set(0);
            reused.copy_id.set(0);
            reused.scopes.activate();
            frame
        }
        None => {
            let scopes = FrameScopes::new();
            scopes.activate();
            Rc::new(FuncCall {
                func: func.clone(),
                scopes,
                scope_ready: Cell::new(false),
                linenr: Cell::new(0),
                returned: Cell::new(false),
                rettv: RefCell::new(TypVal::Unknown),
                breakpoint: Cell::new(0),
                dbg_tick: Cell::new(0),
                level,
                defer: RefCell::new(Vec::new()),
                prof_child: Cell::new(0),
                id,
                refcount: Cell::new(0),
                copy_id: Cell::new(0),
                ufuncs: RefCell::new(Vec::new()),
            })
        }
    });
    set_current_fc(Some(frame.id));
    func_ptr_ref(func);
    frame
}

/// Drop a reference held by *name*, which only the numbered functions and
/// the lambdas have.
pub(crate) fn func_unref_name(name: &CStr) {
    let name = name.to_bytes();
    if !func_name_refcount(name) {
        return;
    }
    match find_func(name) {
        Some(func) => func_ptr_unref(&func),
        // Only give an error for a numbered function.
        None if name.first().is_some_and(u8::is_ascii_digit) => {
            internal_error(c"func_unref()");
            std::process::abort();
        }
        None => {}
    }
}

/// Drop a counted reference and clear the function when the last one goes.
pub(crate) fn func_ptr_unref(func: &UserFunc) {
    // Only clear it when it is not running; otherwise that is done when
    // `calls` reaches zero.
    if func.release() <= 0 && func.calls.get() == 0 {
        func_clear_free(func, false);
    }
}

/// Count a reference held by *name*.
pub(crate) fn func_ref_name(name: &CStr) {
    let name = name.to_bytes();
    if !func_name_refcount(name) {
        return;
    }
    match find_func(name) {
        Some(func) => func.retain(),
        // Only give an error for a numbered function; fail silently when a
        // named or lambda function isn't found.
        None if name.first().is_some_and(u8::is_ascii_digit) => {
            internal_error(c"func_ref()");
        }
        None => {}
    }
}

/// Count a reference held by pointer.
pub(crate) fn func_ptr_ref(func: &UserFunc) {
    func.retain();
}

/// Whether anything outside `frame` still holds it.
fn fc_referenced(frame: &FuncCall) -> bool {
    frame.scopes.referenced() || frame.refcount.get() > 0
}

/// Whether nothing in `frame` carries `copy_id`, i.e. nothing in use reaches
/// it.
fn can_free_funccal(frame: &FuncCall, copy_id: c_int) -> bool {
    frame.scopes.a_list.lv_copy_id != copy_id
        && frame.scopes.l_vars.dv_copy_id != copy_id
        && frame.scopes.a_vars.dv_copy_id != copy_id
        && frame.copy_id.get() != copy_id
}

/// Free every parked funccall the garbage collector did not reach. This is
/// what finally gives back the reference `create_funccal` took.
pub fn free_unref_funccal(copy_id: c_int, testing: c_int) -> bool {
    let did_free = unlink_parked_funccals(Sweep::All, |frame| can_free_funccal(frame, copy_id));
    if did_free {
        // Freeing a funccall may have made more items collectable.
        garbage_collect(testing != 0);
    }
    did_free
}

/// How far a walk of the parked list goes.
enum Sweep {
    /// Stop as soon as one has been freed: `funccal_unref` is looking for
    /// one specific funccall.
    First,
    /// Walk the whole list: the collector frees every one it can.
    All,
}

/// Unpark and free every parked funccall `doomed` accepts, newest first;
/// answers whether any went.
///
/// **The cursor is asked of the list afresh on every step, never carried
/// across the free.** Freeing a parked funccall runs the destructors of
/// everything it holds, and a closure among them re-enters this walk through
/// `funccal_unref`, which can unpark and free others. The walk remembers the
/// last funccall it *kept* and asks for the one after it; should that one
/// have gone meanwhile, it starts again from the newest, which only asks
/// `doomed` again about funccalls it already kept.
fn unlink_parked_funccals(stop: Sweep, mut doomed: impl FnMut(&FuncCall) -> bool) -> bool {
    let mut freed = false;
    let mut kept = None;
    while let Some(id) = parked_after(kept) {
        let frame = id.funccall();
        if !doomed(&frame) {
            kept = Some(id);
            continue;
        }
        unpark_funccal(id);
        free_funccal_contents(&frame);
        freed = true;
        if matches!(stop, Sweep::First) {
            break;
        }
    }
    freed
}

/// The funccall the debugger is looking at, which `:backtrace` moves.
pub fn get_funccal() -> Option<Rc<FuncCall>> {
    let mut funccal = current_fc_id()?;
    // The bound is re-read every step on purpose: the overflow arm below
    // lowers it, and that is what ends the walk.
    let mut i = 0;
    while i < debug_backtrace_level.get() {
        if let Some(caller) = funccal.caller() {
            funccal = caller;
        } else {
            // Backtrace level overflow; reset it to the maximum.
            debug_backtrace_level.set(i);
        }
        i += 1;
    }
    Some(funccal.funccall())
}

/// Run `f` on the funccall whose scopes a variable lookup sees: the call in
/// progress, or the one `:backtrace` moved to.
fn with_scope_funccal<R>(f: impl FnOnce(&FuncCall) -> R) -> Option<R> {
    // Upstream's `have_funccal_scope`: asked of the call in progress, even
    // when `:backtrace` looks at another.
    if !with_current_fc(|frame| frame.is_some_and(|frame| frame.scope_ready.get())) {
        return None;
    }
    // Every variable lookup lands here; without a backtrace level it is the
    // call in progress, read without a reference.
    if debug_backtrace_level.get() == 0 {
        with_current_fc(|frame| frame.map(f))
    } else {
        current_fc_id()?;
        get_funccal().map(|frame| f(&frame))
    }
}

/// The `l:` (`local`) or `a:` scope dictionary of the call a variable
/// lookup sees, or `None` when there is none.
pub(crate) fn funccal_scope(local: bool) -> Option<DictRef> {
    with_scope_funccal(|frame| {
        if local {
            frame.scopes.l_vars.clone()
        } else {
            frame.scopes.a_vars.clone()
        }
    })
}

/// Run `f` over the entry a bare `l:` (`local`) or `a:` evaluates to, or
/// answer `None` when there is no call. `f` must not run user code.
pub(crate) fn with_funccal_scope_entry<R>(
    local: bool,
    f: impl FnOnce(&mut DictItem) -> R,
) -> Option<R> {
    with_scope_funccal(|frame| {
        let entry = if local {
            &frame.scopes.l_vars_var
        } else {
            &frame.scopes.a_vars_var
        };
        f(&mut entry.borrow_mut())
    })
}

/// Whether `dict` is the `l:` scope of the call a variable lookup sees.
pub(crate) fn is_funccal_local_dict(dict: &Dict) -> bool {
    with_scope_funccal(|frame| ptr::eq(frame.scopes.l_vars.as_ptr().cast_const(), dict))
        .unwrap_or(false)
}

/// List the `l:` variables, when there is a function running.
pub fn list_func_vars(first: &mut c_int) {
    if let Some(frame) = current_fc().filter(|frame| frame.scope_ready.get()) {
        list_dict_vars(&frame.scopes.l_vars, c"l:", false, first);
    }
}

/// Walk the chain of captured scopes a closure body can see, running `probe`
/// on each in turn with it made the current call, and stop at the first
/// that answers.
pub(crate) fn walk_scoped_funccals<T>(mut probe: impl FnMut() -> Option<T>) -> Option<T> {
    // The scope the function of the call in progress closed over.
    let scope_of_current =
        || with_current_fc(|frame| frame.and_then(|frame| frame.func.scoped.get()));
    let old_current = current_fc_id();
    let mut found = None;
    set_current_fc(scope_of_current());
    while current_fc_id().is_some() {
        found = probe();
        if found.is_some() {
            break;
        }
        let scoped = scope_of_current();
        if current_fc_id() == scoped {
            break;
        }
        set_current_fc(scoped);
    }
    set_current_fc(old_current);
    found
}

/// Whether the call in progress runs a function that closed over a scope.
pub(crate) fn current_func_has_scope() -> bool {
    with_current_fc(|frame| frame.is_some_and(|frame| frame.func.scoped.get().is_some()))
}

/// Mark the parked funccalls with `copyID + 1`, so that the collector can
/// tell "reachable from a live value" from "merely parked".
pub fn set_ref_in_previous_funccal(copy_id: c_int) -> bool {
    let mark = copy_id + 1;
    for frame in parked_funccals().into_iter().map(FcId::funccall) {
        frame.copy_id.set(mark);
        let scopes = &frame.scopes;
        let reached = set_ref_in_dict_items(&scopes.l_vars, mark, None)
            || set_ref_in_dict_items(&scopes.a_vars, mark, None)
            || set_ref_in_list_items(&scopes.a_list, mark, None);
        if reached {
            return true;
        }
    }
    false
}

/// Mark everything `frame` holds, once per collection.
fn set_ref_in_funccal(frame: &FuncCall, copy_id: c_int) -> bool {
    if frame.copy_id.get() == copy_id {
        return false;
    }
    frame.copy_id.set(copy_id);
    let scopes = &frame.scopes;
    set_ref_in_dict_items(&scopes.l_vars, copy_id, None)
        || set_ref_in_dict_items(&scopes.a_vars, copy_id, None)
        || set_ref_in_list_items(&scopes.a_list, copy_id, None)
        || set_ref_in_func(None, Some(&frame.func), copy_id)
}

/// Mark every local and argument on the call stack, including the stacks
/// set aside.
pub fn set_ref_in_call_stack(copy_id: c_int) -> bool {
    let stacks = ::core::iter::once(current_fc_id()).chain(set_aside_call_stacks());
    stacks
        .flat_map(call_chain)
        .any(|id| set_ref_in_funccal(&id.funccall(), copy_id))
}

/// Mark everything reachable from a function that is still available by name.
pub fn set_ref_in_functions(copy_id: c_int) -> bool {
    for func in all_funcs() {
        if got_int.get() {
            break;
        }
        if !func_name_refcount(func.name().as_bytes())
            && set_ref_in_func(None, Some(&func), copy_id)
        {
            return true;
        }
    }
    false
}

/// Under `v:testing`, keep a call's `args` markable; [`pop_func_args`] undoes.
pub(crate) fn push_func_args(args: &[TypVal]) -> usize {
    if !testing_enabled() {
        return 0;
    }
    let mut frame = CallFrame::new();
    frame.extend_borrowed(args);
    funcargs.with_mut(|kept| kept.push(frame));
    1
}

/// Forget what the last [`push_func_args`] kept, if it kept anything.
pub(crate) fn pop_func_args(pushed: usize) {
    if pushed > 0 {
        let frame = funcargs.with_mut(Vec::pop);
        drop(frame);
    }
}

/// Mark everything reachable from an argument of a call in progress.
pub fn set_ref_in_func_args(copy_id: c_int) -> bool {
    // Marking only reads and calls no function: the borrow may span it.
    funcargs.with(|frames| {
        frames
            .iter()
            .any(|frame| frame.args().iter().any(|tv| mark_root(tv, copy_id)))
    })
}

/// Mark every list and dictionary reachable through the function `name`, or
/// through `func` when the caller already has it. Answers whether marking
/// failed somehow.
pub fn set_ref_in_func(name: Option<&[u8]>, func: Option<&Rc<UserFunc>>, copy_id: c_int) -> bool {
    let found;
    let func = match (func, name) {
        (Some(func), _) => func,
        (None, Some(name)) => {
            found = find_func(&fname_trans_sid(name).0);
            match &found {
                Some(func) => func,
                None => return false,
            }
        }
        (None, None) => return false,
    };

    let mut aborted = false;
    let mut scope = func.scoped.get();
    while let Some(frame) = scope.and_then(FcId::try_funccall) {
        aborted = aborted || set_ref_in_funccal(&frame, copy_id);
        scope = frame.func.scoped.get();
    }
    aborted
}
