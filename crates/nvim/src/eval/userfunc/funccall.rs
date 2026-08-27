//! The funccall_T stack, the function table, and the GC roots.
//!
//! Three families that all read the same two globals.  `create_funccal` /
//! `cleanup_function_call` / `funccal_unref` own the funccall's lifetime --
//! including the case where a closure outlives the call that made it and
//! the funccall has to be kept alive with it.  `func_ref`/`func_unref` and
//! the `func_clear*` group own the `ufunc_T`'s.  The `set_ref_in_*` group
//! is what the garbage collector calls to mark everything reachable from a
//! call in progress, and `find_var_in_scoped_ht` is how a closure body
//! reaches the `l:` of the call it captured.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::mem::offset_of;
use core::ptr;

use super::*;
use crate::types::{NUL, Refcount};

/// A handle on the global user-function table.
///
/// It *names* the table rather than borrowing it, so it stays valid across
/// the callbacks — autocommands, `:function` listings, closures — that add to
/// and remove from the table while a walk over it is in progress. That is
/// what `ht_changed` is for, and a `&mut` could not survive it.
#[derive(Clone, Copy)]
pub(crate) struct FuncTable(*mut hashtab_T);

/// The one place the function table's address is taken.
pub(crate) fn func_table() -> FuncTable {
    FuncTable(func_hashtab.ptr())
}

impl FuncTable {
    /// The address, for the callers outside this family that still take one.
    pub(crate) fn raw(self) -> *mut hashtab_T {
        self.0
    }

    /// Build the table, once, at startup.
    pub(crate) fn init(self) {
        // SAFETY: the only constructor names a `static`, and this runs before
        // anything reads the table.
        unsafe { hash_init(self.0) };
    }

    /// How many live entries the table holds.
    pub(crate) fn used(self) -> size_t {
        // SAFETY: the only constructor names a `static`.
        unsafe { (*self.0).ht_used }
    }

    /// The bucket array, which every walk starts at.
    pub(crate) fn array(self) -> *mut hashitem_T {
        // SAFETY: as `used`.
        unsafe { (*self.0).ht_array }
    }

    /// The generation counter: every add and remove bumps it, so a walk can
    /// tell that a callback rearranged the table under it.
    pub(crate) fn changed(self) -> c_int {
        // SAFETY: as `used`.
        unsafe { (*self.0).ht_changed }
    }

    /// The item for `name`, or the empty slot it would go in.
    ///
    /// # Safety
    /// `name` must be a NUL-terminated string.
    pub(crate) unsafe fn find(self, name: *const c_char) -> *mut hashitem_T {
        // SAFETY: the caller's key; the table is this crate's `static`.
        unsafe { hash_find(self.0, name) }
    }

    /// Add `key` — a `ufunc_T`'s own `uf_name` — to the table.
    ///
    /// # Safety
    /// `key` must be a NUL-terminated string that outlives the entry.
    pub(crate) unsafe fn add(self, key: *mut c_char) -> c_int {
        // SAFETY: the caller's key; the table is this crate's `static`.
        unsafe { hash_add(self.0, key) }
    }

    /// Drop the entry `hi`, which must be one this table answered.
    ///
    /// # Safety
    /// `hi` must be a live item of this table.
    pub(crate) unsafe fn remove(self, hi: *mut hashitem_T) {
        // SAFETY: the caller's item; the table is this crate's `static`.
        unsafe { hash_remove(self.0, hi) };
    }
}

/// Build the function table, once, at startup.
pub fn func_init() {
    func_table().init();
}

/// The function table itself, for the callers outside this family.
pub fn func_tbl_get() -> *mut hashtab_T {
    func_table().raw()
}

/// The functions a funccall registered as closures over it.
///
/// # Safety
/// `fc` is a live funccall.
unsafe fn fc_ufuncs(fc: *mut funccall_T) -> *mut [*mut ufunc_T] {
    // SAFETY: the caller's promise -- `fc` is a live funccall, so its
    // `fc_ufuncs` garray holds `ga_len` initialised function pointers.
    let ga = unsafe { &raw mut (*fc).fc_ufuncs };
    let (data, len) = unsafe { ((*ga).ga_data as *mut *mut ufunc_T, (*ga).ga_len as usize) };
    ptr::slice_from_raw_parts_mut(data, len)
}

/// Free the funccall itself, having already dealt with what it holds.
///
/// # Safety
/// `fc` has been through [`cleanup_function_call`] and is off every list.
unsafe fn free_funccal(fc: *mut funccall_T) {
    // SAFETY: the caller's promise -- `fc` is a live funccall off every list.
    let frame = unsafe { Fc::new(fc) };
    for i in 0..frame.fc_ufuncs.ga_len as usize {
        // SAFETY: `i` is inside the garray `fc_ufuncs` just measured.
        let fp = unsafe { (*fc_ufuncs(fc))[i] };
        // When garbage collecting, a funccall_T may be freed before the
        // function that references it, so clear its `uf_scoped`.  The
        // function may have been redefined and now point at another
        // funccall_T; don't clear it then.
        if !fp.is_null() && unsafe { (*fp).uf_scoped } == fc {
            unsafe { (*fp).uf_scoped = ptr::null_mut() };
        }
    }
    // SAFETY: as above -- the garray and the function are this call's own.
    unsafe { ga_clear(&raw mut (*fc).fc_ufuncs) };

    // The reference `create_funccal` took.  This is the *only* place it
    // is given back, which is why a funccall parked for the garbage
    // collector keeps its function undeletable until then.
    unsafe { func_ptr_unref(frame.fc_func) };
    unsafe { xfree(fc as *mut c_void) };
}

/// Free `fc` and everything in it.  Only for a funccall that was kept beyond
/// its call, i.e. after [`cleanup_function_call`] has run on it.
///
/// # Safety
/// `fc` is a parked funccall, already unlinked from `previous_funccal`.
unsafe fn free_funccal_contents(fc: *mut funccall_T) {
    // All l: variables, then all a: variables, then the a:000 items.
    // SAFETY: the caller's promise -- `fc` is a parked funccall, so the two
    // scope hashtables and the `a:000` list are its own and unreferenced.
    let (vars, avars, items) = unsafe { scopes_of(fc) };
    unsafe { vars_clear(vars) };
    unsafe { vars_clear(avars) };
    for li in unsafe { tv_list_iter(items.as_ref()) } {
        unsafe { tv_clear(&raw mut (*li).li_tv) };
    }
    unsafe { free_funccal(fc) };
}

/// The last part of returning from a function: free the local hashtable,
/// unless a closure, a returned `a:000` or an escaped `l:` is still using it.
///
/// # Safety
/// `fc` is the funccall that has just finished, and is `current_funccal`.
pub(crate) unsafe fn cleanup_function_call(fc: *mut funccall_T) {
    let mut free_fc = true;
    // SAFETY: the caller's promise -- `fc` is the funccall that has just
    // finished, and every scope below is its own.
    let mut frame = unsafe { Fc::new(fc) };
    let may_free_fc = frame.fc_refcount <= Refcount::ZERO;
    current_funccal.set(frame.fc_caller);

    // Free all l: variables if not referred to.
    if may_free_fc && frame.fc_l_vars.dv_refcount == Refcount::new(DO_NOT_FREE_CNT) {
        unsafe { vars_clear(&raw mut (*fc).fc_l_vars.dv_hashtab) };
    } else {
        free_fc = false;
    }

    // If the a:000 list and the l: and a: dicts are not referenced and no
    // closure is using them, the funccall_T and what is in it can go.
    if may_free_fc && frame.fc_l_avars.dv_refcount == Refcount::new(DO_NOT_FREE_CNT) {
        unsafe { vars_clear_ext(&raw mut (*fc).fc_l_avars.dv_hashtab, false) };
    } else {
        free_fc = false;
        // Make a copy of the a: variables, since that was not done above.
        // SAFETY: as above -- the `a:` dictionary is this funccall's own.
        for hi in unsafe { tv_dict_iter(&(*fc).fc_l_avars) } {
            let di = unsafe { tv_dict_hi2di(hi) };
            unsafe { tv_copy(&raw mut (*di).di_tv, &raw mut (*di).di_tv) };
        }
    }

    if may_free_fc && frame.fc_l_varlist.lv_refcount == Refcount::new(DO_NOT_FREE_CNT) {
        frame.fc_l_varlist.lv_first = ptr::null_mut();
    } else {
        free_fc = false;
        // Make a copy of the a:000 items, since that was not done above.
        // SAFETY: as above -- the `a:000` list is this funccall's own.
        for li in unsafe { tv_list_iter(Some(&(*fc).fc_l_varlist)) } {
            unsafe { tv_copy(&raw mut (*li).li_tv, &raw mut (*li).li_tv) };
        }
    }

    if free_fc {
        unsafe { free_funccal(fc) };
        return;
    }

    // "fc" is still in use.  This happens when returning "a:000",
    // assigning "l:" to a global variable, or defining a closure.  Link
    // it into the list for garbage collection later.
    static made_copy: GlobalCell<c_int> = GlobalCell::new(0);
    frame.fc_caller = previous_funccal.get();
    previous_funccal.set(fc);

    if want_garbage_collect.get() {
        // The collector is ready anyway; clear the count.
        made_copy.set(0);
    } else {
        made_copy.set(made_copy.get() + 1);
        if made_copy.get() >= (4096 * 1024 / size_of::<funccall_T>()) as c_int {
            // Four megabytes' worth of copies, which happens when a
            // function that references itself is called repeatedly.  Ask
            // for a collection soon rather than grow without bound.
            made_copy.set(0);
            want_garbage_collect.set(true);
        }
    }
}

/// Drop a reference to `fc` and free it when the last one goes.  `fp` is
/// detached from it either way.
///
/// # Safety
/// `fc` is null or a live funccall; `fp` is a live function.
pub(crate) unsafe fn funccal_unref(fc: *mut funccall_T, fp: *mut ufunc_T, force: bool) {
    if fc.is_null() {
        return;
    }

    // SAFETY: the caller's promise, and `fc` is not null.
    let left = unsafe { (*fc).fc_refcount.release() };
    let unused = if force {
        left <= 0
    } else {
        unsafe { !fc_referenced(fc) }
    };
    if unused && unsafe { unlink_parked_funccals(|parked| parked == fc) } {
        return;
    }
    // SAFETY: as above -- the closure array is this funccall's own.
    for slot in unsafe { (*fc_ufuncs(fc)).iter_mut() } {
        if *slot == fp {
            *slot = ptr::null_mut();
        }
    }
}

/// Take the function out of the function hashtable.  Answers whether it was
/// there: a function deleted while it still had references is already gone.
///
/// # Safety
/// `fp` is a live function.
pub(crate) unsafe fn func_remove(fp: *mut ufunc_T) -> bool {
    // SAFETY: the caller's promise -- `fp` is a live function, so its
    // inline name is the key it was added under.
    let hi = unsafe { func_table().find(uf_name_ptr(fp)) };
    if !unsafe { (*hi).is_kept() } {
        return false;
    }
    unsafe { func_table().remove(hi) };
    true
}

/// Free everything hanging off `fp` -- its argument names, its defaults, its
/// body, its Lua reference and its profiling counters.
///
/// # Safety
/// `fp` is a live function.
pub(crate) unsafe fn func_clear_items(fp: *mut ufunc_T) {
    // SAFETY: the caller's promise -- `fp` is a live function, so the three
    // garrays, the Lua reference and the three counters are all its own.
    let mut f = unsafe { Uf::new(fp) };
    unsafe { ga_clear_strings(&raw mut (*fp).uf_args) };
    unsafe { ga_clear_strings(&raw mut (*fp).uf_def_args) };
    unsafe { ga_clear_strings(&raw mut (*fp).uf_lines) };

    if f.uf_flags & FC_LUAREF != 0 {
        unsafe { api_free_luaref(f.uf_luaref) };
        f.uf_luaref = LUA_NOREF as LuaRef;
    }
    // Addresses, not reads: `field_ptr` is the object's address plus a
    // constant, so naming the three counters needs no dereference.
    let counters: [*mut *mut c_void; 3] = [
        f.field_ptr(offset_of!(ufunc_T, uf_tml_count)),
        f.field_ptr(offset_of!(ufunc_T, uf_tml_total)),
        f.field_ptr(offset_of!(ufunc_T, uf_tml_self)),
    ];
    for counter in counters {
        unsafe { xfree(*counter) };
        unsafe { *counter = ptr::null_mut() };
    }
}

/// Free everything `fp` holds, once.
///
/// # Safety
/// `fp` is a live function.
unsafe fn func_clear(fp: *mut ufunc_T, force: bool) {
    // SAFETY: the caller's promise -- `fp` is a live function.
    let mut f = unsafe { Uf::new(fp) };
    if f.uf_cleared {
        return;
    }
    f.uf_cleared = true;
    unsafe { func_clear_items(fp) };
    // Drop the reference on the scope this function closed over.
    unsafe { funccal_unref(f.uf_scoped, fp, force) };
}

/// Free `fp` itself, having already cleared what it holds.
///
/// # Safety
/// `fp` has been through [`func_clear`].
unsafe fn func_free(fp: *mut ufunc_T) {
    // SAFETY: the caller's promise -- `fp` has been through `func_clear`.
    let mut f = unsafe { Uf::new(fp) };
    // Only remove it when not done already, otherwise we would remove a
    // newer version of the function.
    if f.uf_flags & (FC_DELETED | FC_REMOVED) == 0 {
        unsafe { func_remove(fp) };
    }
    unsafe { xfree(f.uf_name_exp as *mut c_void) };
    f.uf_name_exp = ptr::null_mut();
    unsafe { xfree(fp as *mut c_void) };
}

/// Free a function and everything it holds.
///
/// # Safety
/// `fp` is a live function that nothing is running.
pub(crate) unsafe fn func_clear_free(fp: *mut ufunc_T, force: bool) {
    // SAFETY: the caller's promise, handed straight on to both.
    unsafe { func_clear(fp, force) };
    unsafe { func_free(fp) };
}

/// Start a call of `fp`: allocate its funccall, make it the current one, and
/// take a reference to the function for as long as it lives.
///
/// # Safety
/// `fp` is a live function and `rettv` outlives the call.
pub unsafe fn create_funccal(fp: *mut ufunc_T, rettv: *mut typval_T) -> *mut funccall_T {
    // SAFETY: a fresh, zeroed allocation of the right size, and the
    // caller's promise that `fp` is live and `rettv` outlives the call.
    let fc = unsafe { xcalloc(1, size_of::<funccall_T>()) } as *mut funccall_T;
    let mut frame = unsafe { Fc::new(fc) };
    frame.fc_caller = current_funccal.get();
    current_funccal.set(fc);
    frame.fc_func = fp;
    unsafe { func_ptr_ref(fp) };
    frame.fc_rettv = rettv;
    fc
}

/// The stack of saved call stacks: what `save_funccal` pushes when something
/// (an autocommand, a callback) has to run outside the call in progress.
pub(crate) static funccal_stack: GlobalCell<*mut funccal_entry_T> =
    GlobalCell::new(ptr::null_mut());

/// Put the call stack aside, so that what runs next starts from nothing.
///
/// # Safety
/// `entry` outlives the matching [`restore_funccal`].
pub unsafe fn save_funccal(entry: *mut funccal_entry_T) {
    // SAFETY: the caller's promise -- `entry` outlives the restore.
    let mut saved = unsafe { Live::new(entry) };
    saved.top_funccal = current_funccal.get() as *mut c_void;
    saved.next = funccal_stack.get();
    funccal_stack.set(entry);
    current_funccal.set(ptr::null_mut());
}

/// Put back what [`save_funccal`] set aside.
pub unsafe fn restore_funccal() {
    let top = funccal_stack.get();
    if top.is_null() {
        // SAFETY: a literal message.
        unsafe { iemsg(c"INTERNAL: restore_funccal()".as_ptr()) };
        return;
    }
    // SAFETY: `save_funccal`'s caller promised the entry outlives this, and
    // the stack is this module's own.
    let saved = unsafe { Live::new(top) };
    current_funccal.set(saved.top_funccal as *mut funccall_T);
    funccal_stack.set(saved.next);
}

/// The call in progress, or null.
pub unsafe fn get_current_funccal() -> *mut funccall_T {
    current_funccal.get()
}

/// Make `fc` the call in progress.
pub unsafe fn set_current_funccal(fc: *mut funccall_T) {
    current_funccal.set(fc);
}

/// Drop a reference held by *name*, which only the numbered functions and
/// the lambdas have.
///
/// # Safety
/// `name` is null or NUL-terminated.
pub unsafe fn func_unref(name: *mut c_char) {
    // SAFETY: the caller's promise -- `name` is null or NUL-terminated.
    if name.is_null() || unsafe { !func_name_refcount(name) } {
        return;
    }
    let fp = unsafe { find_func(name) };
    if fp.is_null() && unsafe { *name as u8 }.is_ascii_digit() {
        // Only give an error for a numbered function.
        unsafe { internal_error(c"func_unref()".as_ptr()) };
        unsafe { abort() };
    }
    unsafe { func_ptr_unref(fp) };
}

/// Drop a reference and free the function when the last one goes.
///
/// # Safety
/// `fp` is null or a live function.
pub unsafe fn func_ptr_unref(fp: *mut ufunc_T) {
    if fp.is_null() {
        return;
    }
    // SAFETY: the caller's promise, and `fp` is not null.
    let mut f = unsafe { Uf::new(fp) };
    f.uf_refcount.release();
    // Only delete it when it is not running; otherwise that is done when
    // `uf_calls` reaches zero.
    if f.uf_refcount <= Refcount::ZERO && f.uf_calls == 0 {
        unsafe { func_clear_free(fp, false) };
    }
}

/// Count a reference held by *name*.
///
/// # Safety
/// `name` is null or NUL-terminated.
pub unsafe fn func_ref(name: *mut c_char) {
    // SAFETY: the caller's promise -- `name` is null or NUL-terminated.
    if name.is_null() || unsafe { !func_name_refcount(name) } {
        return;
    }
    let fp = unsafe { find_func(name) };
    if !fp.is_null() {
        unsafe { (*fp).uf_refcount.retain() };
    } else if unsafe { *name as u8 }.is_ascii_digit() {
        // Only give an error for a numbered function; fail silently when
        // a named or lambda function isn't found.
        unsafe { internal_error(c"func_ref()".as_ptr()) };
    }
}

/// Count a reference held by pointer.
///
/// # Safety
/// `fp` is null or a live function.
pub unsafe fn func_ptr_ref(fp: *mut ufunc_T) {
    if !fp.is_null() {
        // SAFETY: the caller's promise, and `fp` is not null.
        unsafe { (*fp).uf_refcount.retain() };
    }
}

/// Whether anything outside `fc` still holds it.
///
/// `l:`, `a:` and `a:000` all live inside the funccall_T, so a reference to
/// any of them is a reference to the whole thing.
///
/// # Safety
/// `fc` is a live funccall.
unsafe fn fc_referenced(fc: *const funccall_T) -> bool {
    // SAFETY: the caller's promise -- `fc` is a live funccall.
    let frame = unsafe { Fc::new(fc.cast_mut()) };
    frame.fc_l_varlist.lv_refcount != Refcount::new(DO_NOT_FREE_CNT)
        || frame.fc_l_vars.dv_refcount != Refcount::new(DO_NOT_FREE_CNT)
        || frame.fc_l_avars.dv_refcount != Refcount::new(DO_NOT_FREE_CNT)
        || frame.fc_refcount > Refcount::ZERO
}

/// Whether nothing in `fc` carries `copyID`, i.e. nothing in use reaches it.
///
/// # Safety
/// `fc` is a live funccall.
unsafe fn can_free_funccal(fc: *mut funccall_T, copyID: c_int) -> bool {
    // SAFETY: the caller's promise -- `fc` is a live funccall.
    let frame = unsafe { Fc::new(fc) };
    frame.fc_l_varlist.lv_copyID != copyID
        && frame.fc_l_vars.dv_copyID != copyID
        && frame.fc_l_avars.dv_copyID != copyID
        && frame.fc_copyID != copyID
}

/// Free every parked funccall the garbage collector did not reach.  This is
/// what finally gives back the reference `create_funccal` took.
///
/// # Safety
/// Called from the collector, with `copyID` the mark just used.
pub unsafe fn free_unref_funccal(copyID: c_int, testing: c_int) -> bool {
    // SAFETY: the collector's own mark, and the parked list is this module's.
    let did_free = unsafe { unlink_parked_funccals(|fc| can_free_funccal(fc, copyID)) };
    if did_free {
        // Freeing a funccal may have made more items collectable.
        // SAFETY: called from the collector, which is between marks.
        unsafe { garbage_collect(testing != 0) };
    }
    did_free
}

/// Unlink and free every parked funccall `doomed` accepts; answers whether
/// any went.
///
/// The C threads a `funccall_T **` through the list so that the head and an
/// interior link are written the same way. The head here is a cell, so the
/// walk carries the *previous* node instead and writes through whichever of
/// the two is right.
///
/// # Safety
/// Every node of the parked list must be live, which is the list's own
/// invariant; `doomed` must not touch the list.
unsafe fn unlink_parked_funccals(mut doomed: impl FnMut(*mut funccall_T) -> bool) -> bool {
    let mut freed = false;
    let mut prev = ptr::null_mut::<funccall_T>();
    let mut fc = previous_funccal.get();
    while !fc.is_null() {
        // SAFETY: a live node of the list.
        let next = unsafe { (*fc).fc_caller };
        if doomed(fc) {
            if prev.is_null() {
                previous_funccal.set(next);
            } else {
                // SAFETY: `prev` is the live node before `fc`.
                unsafe { (*prev).fc_caller = next };
            }
            // SAFETY: unlinked above, so nothing reaches it any more.
            unsafe { free_funccal_contents(fc) };
            freed = true;
        } else {
            prev = fc;
        }
        fc = next;
    }
    freed
}

/// The funccall the debugger is looking at, which `:backtrace` moves.
pub unsafe fn get_funccal() -> *mut funccall_T {
    let mut funccal = current_funccal.get();
    // The bound is re-read every step on purpose: the overflow arm below
    // lowers it, and that is what ends the walk.
    let mut i = 0;
    while i < debug_backtrace_level.get() {
        // SAFETY: the walk starts at the call in progress and steps only to
        // callers, every one of which is live for as long as it is.
        let caller = unsafe { (*funccal).fc_caller };
        if !caller.is_null() {
            funccal = caller;
        } else {
            // Backtrace level overflow; reset it to the maximum.
            debug_backtrace_level.set(i);
        }
        i += 1;
    }
    funccal
}

/// Whether there is a `l:` scope to read at all.
unsafe fn have_funccal_scope() -> bool {
    let fc = current_funccal.get();
    // SAFETY: `current_funccal` is null or the live call in progress.
    !fc.is_null() && unsafe { (*fc).fc_l_vars.dv_refcount } != Refcount::ZERO
}

/// The `l:` scope dictionary, or null when there is no call.
pub unsafe fn get_funccal_local_dict() -> *mut dict_T {
    // SAFETY: `get_funccal` answers a live call, and the address of a field
    // of it is taken without reading the object.
    if !unsafe { have_funccal_scope() } {
        return ptr::null_mut();
    }
    unsafe { &raw mut (*get_funccal()).fc_l_vars }
}

/// The `l:` scope hashtab, or null when there is no call.
pub unsafe fn get_funccal_local_ht() -> *mut hashtab_T {
    // SAFETY: `get_funccal_local_dict` answers null or a live dictionary.
    let d = unsafe { get_funccal_local_dict() };
    if d.is_null() {
        ptr::null_mut()
    } else {
        unsafe { &raw mut (*d).dv_hashtab }
    }
}

/// The `l:` scope variable, or null when there is no call.
pub unsafe fn get_funccal_local_var() -> *mut dictitem_T {
    // SAFETY: as [`get_funccal_local_dict`].
    if !unsafe { have_funccal_scope() } {
        return ptr::null_mut();
    }
    unsafe { &raw mut (*get_funccal()).fc_l_vars_var }.cast()
}

/// The `a:` scope dictionary, or null when there is no call.
pub unsafe fn get_funccal_args_dict() -> *mut dict_T {
    // SAFETY: as [`get_funccal_local_dict`].
    if !unsafe { have_funccal_scope() } {
        return ptr::null_mut();
    }
    unsafe { &raw mut (*get_funccal()).fc_l_avars }
}

/// The `a:` scope hashtab, or null when there is no call.
pub unsafe fn get_funccal_args_ht() -> *mut hashtab_T {
    // SAFETY: `get_funccal_args_dict` answers null or a live dictionary.
    let d = unsafe { get_funccal_args_dict() };
    if d.is_null() {
        ptr::null_mut()
    } else {
        unsafe { &raw mut (*d).dv_hashtab }
    }
}

/// The `a:` scope variable, or null when there is no call.
pub unsafe fn get_funccal_args_var() -> *mut dictitem_T {
    // SAFETY: as [`get_funccal_local_dict`].
    if !unsafe { have_funccal_scope() } {
        return ptr::null_mut();
    }
    unsafe { &raw mut (*get_funccal()).fc_l_avars_var }.cast()
}

/// List the `l:` variables, when there is a function running.
///
/// # Safety
/// `first` is writable.
pub unsafe fn list_func_vars(first: *mut c_int) {
    let fc = current_funccal.get();
    if fc.is_null() {
        return;
    }
    // SAFETY: `current_funccal` is the live call in progress, and `first` is
    // the caller's own.
    let frame = unsafe { Fc::new(fc) };
    if frame.fc_l_vars.dv_refcount > Refcount::ZERO {
        let ht = unsafe { &raw mut (*fc).fc_l_vars.dv_hashtab };
        unsafe { list_hashtable_vars(ht, c"l:".as_ptr(), false, first) };
    }
}

/// The dictionary `ht` belongs to, when `ht` is the current `l:`.
///
/// # Safety
/// `ht` is a live hashtab.
pub unsafe fn get_current_funccal_dict(ht: *mut hashtab_T) -> *mut dict_T {
    let fc = current_funccal.get();
    if fc.is_null() {
        return ptr::null_mut();
    }
    // SAFETY: `current_funccal` is the live call in progress; both are the
    // addresses of its fields, taken without reading the object.
    if ht == unsafe { &raw mut (*fc).fc_l_vars.dv_hashtab } {
        return unsafe { &raw mut (*fc).fc_l_vars };
    }
    ptr::null_mut()
}

/// Walk the chain of captured scopes a closure body can see, running `probe`
/// on each in turn with `current_funccal` set to it, and stop at the first
/// that answers.
///
/// # Safety
/// A call is in progress and its function has a captured scope.
unsafe fn walk_scoped_funccals<T>(mut probe: impl FnMut() -> Option<T>) -> Option<T> {
    // The scope a live call's function closed over, which is a live funccall
    // or null.
    //
    // SAFETY: the caller's promise -- a call is in progress, so
    // `current_funccal` and its `fc_func` are both live.
    let scope_of_current = || unsafe { (*(*current_funccal.get()).fc_func).uf_scoped };
    let old_current_funccal = current_funccal.get();
    let mut found = None;
    current_funccal.set(scope_of_current());
    while !current_funccal.get().is_null() {
        found = probe();
        if found.is_some() {
            break;
        }
        let scoped = scope_of_current();
        if current_funccal.get() == scoped {
            break;
        }
        current_funccal.set(scoped);
    }
    current_funccal.set(old_current_funccal);
    found
}

/// Find a hashitem in a parent scope, i.e. one a lambda captured.
///
/// # Safety
/// `name` is NUL-terminated and `pht` is writable.
pub unsafe fn find_hi_in_scoped_ht(
    name: *const c_char,
    pht: *mut *mut hashtab_T,
) -> *mut hashitem_T {
    // SAFETY: `current_funccal` is null or the live call in progress, whose
    // `fc_func` is live too; `name` is the caller's NUL-terminated string.
    if current_funccal.get().is_null()
        || unsafe { (*(*current_funccal.get()).fc_func).uf_scoped }.is_null()
    {
        return ptr::null_mut();
    }
    let namelen = unsafe { strlen(name) };
    // Upstream answers the *last* hashitem it looked at, not only a
    // found one, so a miss still hands back a non-null empty slot.
    let mut last: *mut hashitem_T = ptr::null_mut();
    // SAFETY: as above; `varname` is a tail of `name`, so the subtraction
    // leaves the length of what is left of it. That holds for every
    // dereference in the probe.
    let probe = || {
        let mut varname: *const c_char = ptr::null();
        let ht = unsafe { find_var_ht(name, namelen, &raw mut varname) };
        if !ht.is_null() && unsafe { *varname } != NUL as c_char {
            let past = unsafe { varname.offset_from(name) } as size_t;
            let hi = unsafe { hash_find_len(ht, varname, namelen.wrapping_sub(past)) };
            last = hi;
            if unsafe { (*hi).is_kept() } {
                unsafe { *pht = ht };
                return Some(hi);
            }
        }
        None
    };
    unsafe { walk_scoped_funccals(probe) };
    last
}

/// Find a variable in a parent scope, i.e. one a lambda captured.
///
/// # Safety
/// `name` has `namelen` readable bytes.
pub unsafe fn find_var_in_scoped_ht(
    name: *const c_char,
    namelen: size_t,
    no_autoload: c_int,
) -> *mut dictitem_T {
    // SAFETY: `current_funccal` is null or the live call in progress, whose
    // `fc_func` is live too; `name` has `namelen` readable bytes and
    // `varname` is a tail of it.
    if current_funccal.get().is_null()
        || unsafe { (*(*current_funccal.get()).fc_func).uf_scoped }.is_null()
    {
        return ptr::null_mut();
    }
    let probe = || {
        let mut varname: *const c_char = ptr::null();
        let ht = unsafe { find_var_ht(name, namelen, &raw mut varname) };
        if !ht.is_null() && unsafe { *varname } != NUL as c_char {
            let past = unsafe { varname.offset_from(name) } as size_t;
            let left = namelen.wrapping_sub(past);
            let first = unsafe { *name } as c_int;
            let v = unsafe { find_var_in_ht(ht, first, varname, left, no_autoload != 0) };
            if !v.is_null() {
                return Some(v);
            }
        }
        None
    };
    unsafe { walk_scoped_funccals(probe) }.unwrap_or(ptr::null_mut())
}

/// Mark the parked funccalls with `copyID + 1`, so that the collector can
/// tell "reachable from a live value" from "merely parked".
pub unsafe fn set_ref_in_previous_funccal(copyID: c_int) -> bool {
    let mut fc = previous_funccal.get();
    let mark = copyID + 1;
    while !fc.is_null() {
        // SAFETY: every node of the parked list is live, which is the list's
        // own invariant, and the three scopes are that node's own.
        unsafe { (*fc).fc_copyID = mark };
        let (vars, avars, items) = unsafe { scopes_of(fc) };
        let reached = unsafe {
            set_ref_in_ht(vars, mark, ptr::null_mut())
                || set_ref_in_ht(avars, mark, ptr::null_mut())
                || set_ref_in_list_items(items, mark, ptr::null_mut())
        };
        if reached {
            return true;
        }
        fc = unsafe { (*fc).fc_caller };
    }
    false
}

/// The three scopes a funccall owns, by address: `l:`, `a:` and `a:000`.
///
/// # Safety
/// `fc` is a live funccall.
unsafe fn scopes_of(
    fc: *mut funccall_T,
) -> (*mut hashtab_T, *mut hashtab_T, *mut crate::types::list_T) {
    // SAFETY: the caller's promise; a field's address is the object's plus a
    // constant, so none of the three reads it.
    unsafe {
        (
            &raw mut (*fc).fc_l_vars.dv_hashtab,
            &raw mut (*fc).fc_l_avars.dv_hashtab,
            &raw mut (*fc).fc_l_varlist,
        )
    }
}

/// Mark everything `fc` holds, once per collection.
///
/// # Safety
/// `fc` is a live funccall.
unsafe fn set_ref_in_funccal(fc: *mut funccall_T, copyID: c_int) -> bool {
    // SAFETY: the caller's promise -- `fc` is a live funccall, so the three
    // scopes and the function are its own.
    let mut frame = unsafe { Fc::new(fc) };
    if frame.fc_copyID == copyID {
        return false;
    }
    frame.fc_copyID = copyID;
    let (vars, avars, items) = unsafe { scopes_of(fc) };
    let func = frame.fc_func;
    unsafe {
        set_ref_in_ht(vars, copyID, ptr::null_mut())
            || set_ref_in_ht(avars, copyID, ptr::null_mut())
            || set_ref_in_list_items(items, copyID, ptr::null_mut())
            || set_ref_in_func(ptr::null_mut(), func, copyID)
    }
}

/// Mark every local and argument on the call stack, including the stacks
/// `save_funccal` set aside.
pub unsafe fn set_ref_in_call_stack(copyID: c_int) -> bool {
    // SAFETY: every funccall on the current stack and on each set-aside
    // stack is live, which is what `save_funccal`'s caller promised. That
    // holds for every dereference below.
    let mut fc = current_funccal.get();
    while !fc.is_null() {
        if unsafe { set_ref_in_funccal(fc, copyID) } {
            return true;
        }
        fc = unsafe { (*fc).fc_caller };
    }

    let mut entry = funccal_stack.get();
    while !entry.is_null() {
        let mut fc = unsafe { (*entry).top_funccal } as *mut funccall_T;
        while !fc.is_null() {
            if unsafe { set_ref_in_funccal(fc, copyID) } {
                return true;
            }
            fc = unsafe { (*fc).fc_caller };
        }
        entry = unsafe { (*entry).next };
    }
    false
}

/// Mark everything reachable from a function that is still available by name.
pub unsafe fn set_ref_in_functions(copyID: c_int) -> bool {
    let mut todo = func_table().used() as c_int;
    let mut hi = func_table().array();
    // SAFETY: the walk covers the `ht_used` kept items of the function
    // table's own bucket array, and every key in it is a live function's
    // inline name.
    while todo > 0 && !got_int.get() {
        if unsafe { (*hi).is_kept() } {
            todo -= 1;
            // The key *is* the function's trailing name member, so the
            // function is that many bytes before it.
            let fp = unsafe { (*hi).hi_key.sub(offset_of!(ufunc_T, uf_name)) } as *mut ufunc_T;
            let named = unsafe { func_name_refcount(uf_name_ptr(fp)) };
            if !named && unsafe { set_ref_in_func(ptr::null_mut(), fp, copyID) } {
                return true;
            }
        }
        hi = unsafe { hi.add(1) };
    }
    false
}

/// Mark everything reachable from an argument of a call in progress.
pub unsafe fn set_ref_in_func_args(copyID: c_int) -> bool {
    // Marking only reads; nothing it reaches calls a function, so holding the
    // borrow across the walk is sound.
    funcargs.with(|args| {
        args.iter().any(|&tv| {
            // SAFETY: each entry points at a live caller's argument.
            unsafe { set_ref_in_item(tv, copyID, ptr::null_mut(), ptr::null_mut()) }
        })
    })
}

/// Mark every list and dictionary reachable through the function `name`, or
/// through `fp_in` when the caller already has it.  Answers whether marking
/// failed somehow.
///
/// # Safety
/// `name` is null or NUL-terminated; `fp_in` is null or a live function.
pub unsafe fn set_ref_in_func(name: *mut c_char, fp_in: *mut ufunc_T, copyID: c_int) -> bool {
    if name.is_null() && fp_in.is_null() {
        return false;
    }

    let mut error = FCERR_NONE;
    let mut fname_buf: [c_char; FLEN_FIXED as usize + 1] = [0; FLEN_FIXED as usize + 1];
    let mut tofree: *mut c_char = ptr::null_mut();
    let buf = fname_buf.as_mut_ptr();
    let (freep, errp) = (&raw mut tofree, &raw mut error);
    // SAFETY: the caller's promise -- `name` is NUL-terminated when it is
    // used; `buf` has `FLEN_FIXED + 1` bytes and the out-parameters are this
    // frame's locals.
    let fp = if fp_in.is_null() {
        unsafe { find_func(fname_trans_sid(name, buf, freep, errp)) }
    } else {
        fp_in
    };

    let mut aborted = false;
    if !fp.is_null() {
        // SAFETY: `fp` is a live function, and every scope on the chain is a
        // live funccall whose own function is live too.
        let mut fc = unsafe { (*fp).uf_scoped };
        while !fc.is_null() {
            aborted = aborted || unsafe { set_ref_in_funccal(fc, copyID) };
            fc = unsafe { (*(*fc).fc_func).uf_scoped };
        }
    }
    unsafe { xfree(tofree as *mut c_void) };
    aborted
}
