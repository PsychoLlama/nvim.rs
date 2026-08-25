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
use crate::types::NUL;

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
    unsafe {
        let ga = &raw mut (*fc).fc_ufuncs;
        ptr::slice_from_raw_parts_mut((*ga).ga_data as *mut *mut ufunc_T, (*ga).ga_len as usize)
    }
}

/// Free the funccall itself, having already dealt with what it holds.
///
/// # Safety
/// `fc` has been through [`cleanup_function_call`] and is off every list.
unsafe fn free_funccal(fc: *mut funccall_T) {
    unsafe {
        for i in 0..(*fc).fc_ufuncs.ga_len as usize {
            let fp = (*fc_ufuncs(fc))[i];
            // When garbage collecting, a funccall_T may be freed before the
            // function that references it, so clear its `uf_scoped`.  The
            // function may have been redefined and now point at another
            // funccall_T; don't clear it then.
            if !fp.is_null() && (*fp).uf_scoped == fc {
                (*fp).uf_scoped = ptr::null_mut();
            }
        }
        ga_clear(&raw mut (*fc).fc_ufuncs);

        // The reference `create_funccal` took.  This is the *only* place it
        // is given back, which is why a funccall parked for the garbage
        // collector keeps its function undeletable until then.
        func_ptr_unref((*fc).fc_func);
        xfree(fc as *mut c_void);
    }
}

/// Free `fc` and everything in it.  Only for a funccall that was kept beyond
/// its call, i.e. after [`cleanup_function_call`] has run on it.
///
/// # Safety
/// `fc` is a parked funccall, already unlinked from `previous_funccal`.
unsafe fn free_funccal_contents(fc: *mut funccall_T) {
    unsafe {
        // All l: variables, then all a: variables, then the a:000 items.
        vars_clear(&raw mut (*fc).fc_l_vars.dv_hashtab);
        vars_clear(&raw mut (*fc).fc_l_avars.dv_hashtab);
        for li in tv_list_iter(Some(&(*fc).fc_l_varlist)) {
            tv_clear(&raw mut (*li).li_tv);
        }
        free_funccal(fc);
    }
}

/// The last part of returning from a function: free the local hashtable,
/// unless a closure, a returned `a:000` or an escaped `l:` is still using it.
///
/// # Safety
/// `fc` is the funccall that has just finished, and is `current_funccal`.
pub(crate) unsafe fn cleanup_function_call(fc: *mut funccall_T) {
    let mut free_fc = true;
    unsafe {
        let may_free_fc = (*fc).fc_refcount <= 0;
        current_funccal.set((*fc).fc_caller);

        // Free all l: variables if not referred to.
        if may_free_fc && (*fc).fc_l_vars.dv_refcount == DO_NOT_FREE_CNT {
            vars_clear(&raw mut (*fc).fc_l_vars.dv_hashtab);
        } else {
            free_fc = false;
        }

        // If the a:000 list and the l: and a: dicts are not referenced and no
        // closure is using them, the funccall_T and what is in it can go.
        if may_free_fc && (*fc).fc_l_avars.dv_refcount == DO_NOT_FREE_CNT {
            vars_clear_ext(&raw mut (*fc).fc_l_avars.dv_hashtab, false);
        } else {
            free_fc = false;
            // Make a copy of the a: variables, since that was not done above.
            for hi in tv_dict_iter(&(*fc).fc_l_avars) {
                let di = tv_dict_hi2di(hi);
                tv_copy(&raw mut (*di).di_tv, &raw mut (*di).di_tv);
            }
        }

        if may_free_fc && (*fc).fc_l_varlist.lv_refcount == DO_NOT_FREE_CNT {
            (*fc).fc_l_varlist.lv_first = ptr::null_mut();
        } else {
            free_fc = false;
            // Make a copy of the a:000 items, since that was not done above.
            for li in tv_list_iter(Some(&(*fc).fc_l_varlist)) {
                tv_copy(&raw mut (*li).li_tv, &raw mut (*li).li_tv);
            }
        }

        if free_fc {
            free_funccal(fc);
            return;
        }

        // "fc" is still in use.  This happens when returning "a:000",
        // assigning "l:" to a global variable, or defining a closure.  Link
        // it into the list for garbage collection later.
        static made_copy: GlobalCell<c_int> = GlobalCell::new(0);
        (*fc).fc_caller = previous_funccal.get();
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
}

/// Drop a reference to `fc` and free it when the last one goes.  `fp` is
/// detached from it either way.
///
/// # Safety
/// `fc` is null or a live funccall; `fp` is a live function.
pub(crate) unsafe fn funccal_unref(fc: *mut funccall_T, fp: *mut ufunc_T, force: bool) {
    unsafe {
        if fc.is_null() {
            return;
        }

        (*fc).fc_refcount -= 1;
        let unused = if force {
            (*fc).fc_refcount <= 0
        } else {
            !fc_referenced(fc)
        };
        if unused && unlink_parked_funccals(|parked| parked == fc) {
            return;
        }
        for slot in (*fc_ufuncs(fc)).iter_mut() {
            if *slot == fp {
                *slot = ptr::null_mut();
            }
        }
    }
}

/// Take the function out of the function hashtable.  Answers whether it was
/// there: a function deleted while it still had references is already gone.
///
/// # Safety
/// `fp` is a live function.
pub(crate) unsafe fn func_remove(fp: *mut ufunc_T) -> bool {
    unsafe {
        let hi = func_table().find(uf_name_ptr(fp));
        if !(*hi).is_kept() {
            return false;
        }
        func_table().remove(hi);
        true
    }
}

/// Free everything hanging off `fp` -- its argument names, its defaults, its
/// body, its Lua reference and its profiling counters.
///
/// # Safety
/// `fp` is a live function.
pub(crate) unsafe fn func_clear_items(fp: *mut ufunc_T) {
    unsafe {
        ga_clear_strings(&raw mut (*fp).uf_args);
        ga_clear_strings(&raw mut (*fp).uf_def_args);
        ga_clear_strings(&raw mut (*fp).uf_lines);

        if (*fp).uf_flags & FC_LUAREF != 0 {
            api_free_luaref((*fp).uf_luaref);
            (*fp).uf_luaref = LUA_NOREF as LuaRef;
        }
        for counter in [
            &raw mut (*fp).uf_tml_count as *mut *mut c_void,
            &raw mut (*fp).uf_tml_total as *mut *mut c_void,
            &raw mut (*fp).uf_tml_self as *mut *mut c_void,
        ] {
            xfree(*counter);
            *counter = ptr::null_mut();
        }
    }
}

/// Free everything `fp` holds, once.
///
/// # Safety
/// `fp` is a live function.
unsafe fn func_clear(fp: *mut ufunc_T, force: bool) {
    unsafe {
        if (*fp).uf_cleared {
            return;
        }
        (*fp).uf_cleared = true;
        func_clear_items(fp);
        // Drop the reference on the scope this function closed over.
        funccal_unref((*fp).uf_scoped, fp, force);
    }
}

/// Free `fp` itself, having already cleared what it holds.
///
/// # Safety
/// `fp` has been through [`func_clear`].
unsafe fn func_free(fp: *mut ufunc_T) {
    unsafe {
        // Only remove it when not done already, otherwise we would remove a
        // newer version of the function.
        if (*fp).uf_flags & (FC_DELETED | FC_REMOVED) == 0 {
            func_remove(fp);
        }
        xfree((*fp).uf_name_exp as *mut c_void);
        (*fp).uf_name_exp = ptr::null_mut();
        xfree(fp as *mut c_void);
    }
}

/// Free a function and everything it holds.
///
/// # Safety
/// `fp` is a live function that nothing is running.
pub(crate) unsafe fn func_clear_free(fp: *mut ufunc_T, force: bool) {
    unsafe {
        func_clear(fp, force);
        func_free(fp);
    }
}

/// Start a call of `fp`: allocate its funccall, make it the current one, and
/// take a reference to the function for as long as it lives.
///
/// # Safety
/// `fp` is a live function and `rettv` outlives the call.
pub unsafe fn create_funccal(fp: *mut ufunc_T, rettv: *mut typval_T) -> *mut funccall_T {
    unsafe {
        let fc = xcalloc(1, size_of::<funccall_T>()) as *mut funccall_T;
        (*fc).fc_caller = current_funccal.get();
        current_funccal.set(fc);
        (*fc).fc_func = fp;
        func_ptr_ref(fp);
        (*fc).fc_rettv = rettv;
        fc
    }
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
    unsafe {
        (*entry).top_funccal = current_funccal.get() as *mut c_void;
        (*entry).next = funccal_stack.get();
        funccal_stack.set(entry);
        current_funccal.set(ptr::null_mut());
    }
}

/// Put back what [`save_funccal`] set aside.
pub unsafe fn restore_funccal() {
    unsafe {
        if funccal_stack.get().is_null() {
            iemsg(c"INTERNAL: restore_funccal()".as_ptr());
            return;
        }
        current_funccal.set((*funccal_stack.get()).top_funccal as *mut funccall_T);
        funccal_stack.set((*funccal_stack.get()).next);
    }
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
    unsafe {
        if name.is_null() || !func_name_refcount(name) {
            return;
        }
        let fp = find_func(name);
        if fp.is_null() && (*name as u8).is_ascii_digit() {
            // Only give an error for a numbered function.
            internal_error(c"func_unref()".as_ptr());
            abort();
        }
        func_ptr_unref(fp);
    }
}

/// Drop a reference and free the function when the last one goes.
///
/// # Safety
/// `fp` is null or a live function.
pub unsafe fn func_ptr_unref(fp: *mut ufunc_T) {
    unsafe {
        if fp.is_null() {
            return;
        }
        (*fp).uf_refcount -= 1;
        // Only delete it when it is not running; otherwise that is done when
        // `uf_calls` reaches zero.
        if (*fp).uf_refcount <= 0 && (*fp).uf_calls == 0 {
            func_clear_free(fp, false);
        }
    }
}

/// Count a reference held by *name*.
///
/// # Safety
/// `name` is null or NUL-terminated.
pub unsafe fn func_ref(name: *mut c_char) {
    unsafe {
        if name.is_null() || !func_name_refcount(name) {
            return;
        }
        let fp = find_func(name);
        if !fp.is_null() {
            (*fp).uf_refcount += 1;
        } else if (*name as u8).is_ascii_digit() {
            // Only give an error for a numbered function; fail silently when
            // a named or lambda function isn't found.
            internal_error(c"func_ref()".as_ptr());
        }
    }
}

/// Count a reference held by pointer.
///
/// # Safety
/// `fp` is null or a live function.
pub unsafe fn func_ptr_ref(fp: *mut ufunc_T) {
    unsafe {
        if !fp.is_null() {
            (*fp).uf_refcount += 1;
        }
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
    unsafe {
        (*fc).fc_l_varlist.lv_refcount != DO_NOT_FREE_CNT
            || (*fc).fc_l_vars.dv_refcount != DO_NOT_FREE_CNT
            || (*fc).fc_l_avars.dv_refcount != DO_NOT_FREE_CNT
            || (*fc).fc_refcount > 0
    }
}

/// Whether nothing in `fc` carries `copyID`, i.e. nothing in use reaches it.
///
/// # Safety
/// `fc` is a live funccall.
unsafe fn can_free_funccal(fc: *mut funccall_T, copyID: c_int) -> bool {
    unsafe {
        (*fc).fc_l_varlist.lv_copyID != copyID
            && (*fc).fc_l_vars.dv_copyID != copyID
            && (*fc).fc_l_avars.dv_copyID != copyID
            && (*fc).fc_copyID != copyID
    }
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
    unsafe {
        let mut funccal = current_funccal.get();
        // The bound is re-read every step on purpose: the overflow arm below
        // lowers it, and that is what ends the walk.
        let mut i = 0;
        while i < debug_backtrace_level.get() {
            let caller = (*funccal).fc_caller;
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
}

/// Whether there is a `l:` scope to read at all.
unsafe fn have_funccal_scope() -> bool {
    unsafe {
        !current_funccal.get().is_null() && (*current_funccal.get()).fc_l_vars.dv_refcount != 0
    }
}

/// The `l:` scope dictionary, or null when there is no call.
pub unsafe fn get_funccal_local_dict() -> *mut dict_T {
    unsafe {
        if !have_funccal_scope() {
            return ptr::null_mut();
        }
        &raw mut (*get_funccal()).fc_l_vars
    }
}

/// The `l:` scope hashtab, or null when there is no call.
pub unsafe fn get_funccal_local_ht() -> *mut hashtab_T {
    unsafe {
        let d = get_funccal_local_dict();
        if d.is_null() {
            ptr::null_mut()
        } else {
            &raw mut (*d).dv_hashtab
        }
    }
}

/// The `l:` scope variable, or null when there is no call.
pub unsafe fn get_funccal_local_var() -> *mut dictitem_T {
    unsafe {
        if !have_funccal_scope() {
            return ptr::null_mut();
        }
        (&raw mut (*get_funccal()).fc_l_vars_var) as *mut dictitem_T
    }
}

/// The `a:` scope dictionary, or null when there is no call.
pub unsafe fn get_funccal_args_dict() -> *mut dict_T {
    unsafe {
        if !have_funccal_scope() {
            return ptr::null_mut();
        }
        &raw mut (*get_funccal()).fc_l_avars
    }
}

/// The `a:` scope hashtab, or null when there is no call.
pub unsafe fn get_funccal_args_ht() -> *mut hashtab_T {
    unsafe {
        let d = get_funccal_args_dict();
        if d.is_null() {
            ptr::null_mut()
        } else {
            &raw mut (*d).dv_hashtab
        }
    }
}

/// The `a:` scope variable, or null when there is no call.
pub unsafe fn get_funccal_args_var() -> *mut dictitem_T {
    unsafe {
        if !have_funccal_scope() {
            return ptr::null_mut();
        }
        (&raw mut (*get_funccal()).fc_l_avars_var) as *mut dictitem_T
    }
}

/// List the `l:` variables, when there is a function running.
///
/// # Safety
/// `first` is writable.
pub unsafe fn list_func_vars(first: *mut c_int) {
    unsafe {
        if !current_funccal.get().is_null() && (*current_funccal.get()).fc_l_vars.dv_refcount > 0 {
            list_hashtable_vars(
                &raw mut (*current_funccal.get()).fc_l_vars.dv_hashtab,
                c"l:".as_ptr(),
                false,
                first,
            );
        }
    }
}

/// The dictionary `ht` belongs to, when `ht` is the current `l:`.
///
/// # Safety
/// `ht` is a live hashtab.
pub unsafe fn get_current_funccal_dict(ht: *mut hashtab_T) -> *mut dict_T {
    unsafe {
        if !current_funccal.get().is_null()
            && ht == &raw mut (*current_funccal.get()).fc_l_vars.dv_hashtab
        {
            return &raw mut (*current_funccal.get()).fc_l_vars;
        }
        ptr::null_mut()
    }
}

/// Walk the chain of captured scopes a closure body can see, running `probe`
/// on each in turn with `current_funccal` set to it, and stop at the first
/// that answers.
///
/// # Safety
/// A call is in progress and its function has a captured scope.
unsafe fn walk_scoped_funccals<T>(mut probe: impl FnMut() -> Option<T>) -> Option<T> {
    unsafe {
        let old_current_funccal = current_funccal.get();
        let mut found = None;
        current_funccal.set((*(*current_funccal.get()).fc_func).uf_scoped);
        while !current_funccal.get().is_null() {
            found = probe();
            if found.is_some() {
                break;
            }
            let scoped = (*(*current_funccal.get()).fc_func).uf_scoped;
            if current_funccal.get() == scoped {
                break;
            }
            current_funccal.set(scoped);
        }
        current_funccal.set(old_current_funccal);
        found
    }
}

/// Find a hashitem in a parent scope, i.e. one a lambda captured.
///
/// # Safety
/// `name` is NUL-terminated and `pht` is writable.
pub unsafe fn find_hi_in_scoped_ht(
    name: *const c_char,
    pht: *mut *mut hashtab_T,
) -> *mut hashitem_T {
    unsafe {
        if current_funccal.get().is_null()
            || (*(*current_funccal.get()).fc_func).uf_scoped.is_null()
        {
            return ptr::null_mut();
        }
        let namelen = strlen(name);
        // Upstream answers the *last* hashitem it looked at, not only a
        // found one, so a miss still hands back a non-null empty slot.
        let mut last: *mut hashitem_T = ptr::null_mut();
        walk_scoped_funccals(|| {
            let mut varname: *const c_char = ptr::null();
            let ht = find_var_ht(name, namelen, &raw mut varname);
            if !ht.is_null() && *varname != NUL as c_char {
                let hi = hash_find_len(
                    ht,
                    varname,
                    namelen.wrapping_sub(varname.offset_from(name) as size_t),
                );
                last = hi;
                if (*hi).is_kept() {
                    *pht = ht;
                    return Some(hi);
                }
            }
            None
        });
        last
    }
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
    unsafe {
        if current_funccal.get().is_null()
            || (*(*current_funccal.get()).fc_func).uf_scoped.is_null()
        {
            return ptr::null_mut();
        }
        walk_scoped_funccals(|| {
            let mut varname: *const c_char = ptr::null();
            let ht = find_var_ht(name, namelen, &raw mut varname);
            if !ht.is_null() && *varname != NUL as c_char {
                let v = find_var_in_ht(
                    ht,
                    *name as c_int,
                    varname,
                    namelen.wrapping_sub(varname.offset_from(name) as size_t),
                    no_autoload != 0,
                );
                if !v.is_null() {
                    return Some(v);
                }
            }
            None
        })
        .unwrap_or(ptr::null_mut())
    }
}

/// Mark the parked funccalls with `copyID + 1`, so that the collector can
/// tell "reachable from a live value" from "merely parked".
pub unsafe fn set_ref_in_previous_funccal(copyID: c_int) -> bool {
    unsafe {
        let mut fc = previous_funccal.get();
        while !fc.is_null() {
            (*fc).fc_copyID = copyID + 1;
            if set_ref_in_ht(
                &raw mut (*fc).fc_l_vars.dv_hashtab,
                copyID + 1,
                ptr::null_mut(),
            ) || set_ref_in_ht(
                &raw mut (*fc).fc_l_avars.dv_hashtab,
                copyID + 1,
                ptr::null_mut(),
            ) || set_ref_in_list_items(&raw mut (*fc).fc_l_varlist, copyID + 1, ptr::null_mut())
            {
                return true;
            }
            fc = (*fc).fc_caller;
        }
        false
    }
}

/// Mark everything `fc` holds, once per collection.
///
/// # Safety
/// `fc` is a live funccall.
unsafe fn set_ref_in_funccal(fc: *mut funccall_T, copyID: c_int) -> bool {
    unsafe {
        if (*fc).fc_copyID == copyID {
            return false;
        }
        (*fc).fc_copyID = copyID;
        set_ref_in_ht(&raw mut (*fc).fc_l_vars.dv_hashtab, copyID, ptr::null_mut())
            || set_ref_in_ht(
                &raw mut (*fc).fc_l_avars.dv_hashtab,
                copyID,
                ptr::null_mut(),
            )
            || set_ref_in_list_items(&raw mut (*fc).fc_l_varlist, copyID, ptr::null_mut())
            || set_ref_in_func(ptr::null_mut(), (*fc).fc_func, copyID)
    }
}

/// Mark every local and argument on the call stack, including the stacks
/// `save_funccal` set aside.
pub unsafe fn set_ref_in_call_stack(copyID: c_int) -> bool {
    unsafe {
        let mut fc = current_funccal.get();
        while !fc.is_null() {
            if set_ref_in_funccal(fc, copyID) {
                return true;
            }
            fc = (*fc).fc_caller;
        }

        let mut entry = funccal_stack.get();
        while !entry.is_null() {
            let mut fc = (*entry).top_funccal as *mut funccall_T;
            while !fc.is_null() {
                if set_ref_in_funccal(fc, copyID) {
                    return true;
                }
                fc = (*fc).fc_caller;
            }
            entry = (*entry).next;
        }
        false
    }
}

/// Mark everything reachable from a function that is still available by name.
pub unsafe fn set_ref_in_functions(copyID: c_int) -> bool {
    unsafe {
        let mut todo = func_table().used() as c_int;
        let mut hi = func_table().array();
        while todo > 0 && !got_int.get() {
            if (*hi).is_kept() {
                todo -= 1;
                let fp = (*hi).hi_key.sub(offset_of!(ufunc_T, uf_name)) as *mut ufunc_T;
                if !func_name_refcount(uf_name_ptr(fp))
                    && set_ref_in_func(ptr::null_mut(), fp, copyID)
                {
                    return true;
                }
            }
            hi = hi.add(1);
        }
        false
    }
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
    unsafe {
        if name.is_null() && fp_in.is_null() {
            return false;
        }

        let mut error = FCERR_NONE;
        let mut fname_buf: [c_char; FLEN_FIXED as usize + 1] = [0; FLEN_FIXED as usize + 1];
        let mut tofree: *mut c_char = ptr::null_mut();
        let fp = if fp_in.is_null() {
            let fname = fname_trans_sid(
                name,
                fname_buf.as_mut_ptr(),
                &raw mut tofree,
                &raw mut error,
            );
            find_func(fname)
        } else {
            fp_in
        };

        let mut aborted = false;
        if !fp.is_null() {
            let mut fc = (*fp).uf_scoped;
            while !fc.is_null() {
                aborted = aborted || set_ref_in_funccal(fc, copyID);
                fc = (*(*fc).fc_func).uf_scoped;
            }
        }
        xfree(tofree as *mut c_void);
        aborted
    }
}
