//! `:unlet`, `:lockvar` and `:unlockvar`.
//!
//! All three share [`ex_unletlock`]'s argument walk and differ only in the
//! callback it is given, so deleting and locking are written here together --
//! as they are upstream.  That one walk is what makes `:unlet` and
//! `:lockvar` agree on what an argument means.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int};
use core::mem::offset_of;
use core::ptr;

use super::*;
use crate::types::{FAIL, NUL, OK};

/// `:unlet`.
///
/// # Safety
/// `eap` is a live `:unlet` command.
pub unsafe fn ex_unlet(eap: *mut exarg_T) {
    // `:unlet!` means "do not complain", which reaches `get_lval` as
    // GLV_QUIET and `do_unlet` as `forceit`.
    // SAFETY: the caller's obligation -- a live command, which the
    // `do_cmdline` frame that owns the `exarg_T` outlives.
    let ea = unsafe { Ea::new(eap) };
    let glv_flags = if ea.forceit != 0 { GLV_QUIET } else { 0 };
    let arg = ea.arg;
    unsafe { ex_unletlock(eap, arg, 0, glv_flags, do_unlet_var) };
}

/// `:lockvar` and `:unlockvar`.
///
/// # Safety
/// `eap` is a live `:lockvar`/`:unlockvar` command.
pub unsafe fn ex_lockvar(eap: *mut exarg_T) {
    // SAFETY: the caller's obligation -- a live command whose argument text
    // is NUL-terminated.
    let ea = unsafe { Ea::new(eap) };
    let mut arg = ea.arg;
    // Two levels by default: the variable and what it directly holds.
    // `!` is everything, and an explicit count says how deep.
    let mut deep = 2;
    if ea.forceit != 0 {
        deep = -1;
    } else if ascii_isdigit(c_int::from(unsafe { *arg })) {
        deep = unsafe { getdigits_int(&raw mut arg, false, -1) };
        arg = unsafe { skipwhite(arg) };
    }
    unsafe { ex_unletlock(eap, arg, deep, 0, do_lock_var) };
}

/// The argument walk `:unlet`, `:lockvar` and `:unlockvar` share, calling
/// `callback` on each name it resolves.
///
/// A failure does not stop the walk: parsing carries on so that the trailing
/// arguments are still checked, but `error` suppresses every later callback.
///
/// # Safety
/// `eap` is a live command and `argstart` a NUL-terminated string.
unsafe fn ex_unletlock(
    eap: *mut exarg_T,
    argstart: *mut c_char,
    deep: c_int,
    glv_flags: c_int,
    callback: ex_unletlock_callback,
) {
    // SAFETY: the caller's obligation -- a live command and a NUL-terminated
    // argument text, which `arg` and `name_end` both stay inside.
    let mut ea = unsafe { Ea::new(eap) };
    let mut arg = argstart;
    let mut name_end;
    let mut error = false;
    let mut lv = LVAL_INITIAL_VALUE;
    let lvp = &raw mut lv;

    loop {
        if unsafe { *arg } == b'$' as c_char {
            // An environment variable: `get_lval` does not parse one, so
            // the lvalue is filled in by hand.
            lv.ll_name = arg;
            lv.ll_tv = ptr::null_mut();
            arg = unsafe { arg.add(1) };
            if unsafe { get_env_len(&raw mut arg as *mut *const c_char) } == 0 {
                semsg_c!(
                    unsafe { gettext(&raw const e_invarg2 as *const c_char) },
                    unsafe { arg.sub(1) }
                );
                return;
            }
            if !error && ea.skip == 0 && unsafe { callback(lvp, arg, eap, deep) } == FAIL {
                error = true;
            }
            name_end = arg;
        } else {
            let quiet = ea.skip != 0 || error;
            let nil = ptr::null_mut();
            name_end = unsafe { get_lval(arg, nil, lvp, true, quiet, glv_flags, FNE_CHECK_START) };
            if lv.ll_name.is_null() {
                // An error, but carry on parsing.
                error = true;
            }
            // The byte is only read once `name_end` has proved not to be
            // NULL, which is upstream's order.
            let trailing = (!name_end.is_null()).then(|| c_int::from(unsafe { *name_end }));
            if trailing.is_none_or(|c| !ascii_iswhite(c) && ends_excmd(c) == 0) {
                if !name_end.is_null() {
                    emsg_severe.set(true);
                    semsg_c!(
                        unsafe { gettext(&raw const e_trailing_arg as *const c_char) },
                        name_end,
                    );
                }
                if !(ea.skip != 0 || error) {
                    unsafe { clear_lval(lvp) };
                }
                break;
            }

            if !error && ea.skip == 0 && unsafe { callback(lvp, name_end, eap, deep) } == FAIL {
                error = true;
            }
            if ea.skip == 0 {
                unsafe { clear_lval(lvp) };
            }
        }
        arg = unsafe { skipwhite(name_end) };
        if ends_excmd(c_int::from(unsafe { *arg })) != 0 {
            break;
        }
    }

    ea.nextcmd = unsafe { check_nextcmd(arg) };
}

/// `:unlet`'s callback: delete what `lp` names.
///
/// # Safety
/// `lp` is a resolved lvalue, `name_end` points into the command line and
/// `eap` is live.
unsafe fn do_unlet_var(
    lp: *mut lval_T,
    name_end: *mut c_char,
    eap: *mut exarg_T,
    _deep: c_int,
) -> c_int {
    // SAFETY: the caller's obligation -- a resolved lvalue and a live
    // command, both of which outlive this call.
    let lp = unsafe { Lv::new(lp) };
    let ea = unsafe { Ea::new(eap) };
    if lp.ll_tv.is_null() {
        // A whole variable: an environment variable, a plain name or an
        // expanded one.  Terminate the name in place, so that the error
        // does not quote the rest of the command.
        // SAFETY: `name_end` points into the command line, and a resolved
        // lvalue's name is NUL-terminated there.
        let cc = unsafe { *name_end };
        unsafe { *name_end = NUL as c_char };
        let ret = if unsafe { *lp.ll_name } == b'$' as c_char {
            unsafe { vim_unsetenv_ext(lp.ll_name.add(1)) };
            OK
        } else {
            unsafe { do_unlet(lp.ll_name, lp.ll_name_len, ea.forceit != 0) }
        };
        unsafe { *name_end = cc };
        return ret;
    }

    // `ll_list` is non-NULL whenever the lvalue *is* in a list; a NULL
    // list yields E689 before reaching here. Both tests are written out
    // because `value_check_lock` reports, so the second must not run when
    // the first already answered true.
    // SAFETY: a resolved lvalue's list and dictionary are live or NULL.
    let mut locked = false;
    if !lp.ll_list.is_null() {
        let lock = unsafe { tv_list_locked(lp.ll_list) };
        locked = unsafe { value_check_lock(lock, lp.ll_name, lp.ll_name_len) };
    }
    if !locked && !lp.ll_dict.is_null() {
        let lock = unsafe { (*lp.ll_dict).dv_lock };
        locked = unsafe { value_check_lock(lock, lp.ll_name, lp.ll_name_len) };
    }
    if locked {
        return FAIL;
    }

    if lp.ll_range {
        let (n1, n2, to_end) = (lp.ll_n1, lp.ll_n2, !lp.ll_empty2);
        // SAFETY: a resolved lvalue's list and the item it starts at.
        unsafe { tv_list_unlet_range(lp.ll_list, lp.ll_li, n1, to_end, n2) };
    } else if !lp.ll_list.is_null() {
        // One List item.
        unsafe { tv_list_item_remove(lp.ll_list, lp.ll_li) };
    } else {
        // One Dict item.
        let d = lp.ll_dict;
        debug_assert!(!d.is_null());
        // SAFETY: a resolved lvalue's item of that dictionary.
        let di = unsafe { Di::new(lp.ll_di) };
        let watched = unsafe { tv_dict_is_watched(d) };

        let mut oldtv = TV_INITIAL_VALUE;
        let mut key: *mut c_char = ptr::null_mut();
        if watched {
            let tv = di.field_ptr(offset_of!(dictitem_T, di_tv));
            unsafe { tv_copy(tv, &raw mut oldtv) };
            // The key has to be saved: removing the item frees it.
            key = unsafe { xstrdup(tv_dict_item_key(di.raw())) };
        }

        unsafe { tv_dict_item_remove(d, di.raw()) };

        if watched {
            unsafe { tv_dict_watcher_notify(d, key, ptr::null_mut(), &raw mut oldtv) };
            unsafe { tv_clear(&raw mut oldtv) };
            unsafe { xfree(key.cast()) };
        }
    }
    OK
}

/// Delete the items of `l` from `li_first` through the `n2`-th, or to the
/// end when `has_n2` is false.
///
/// # Safety
/// `l` is a live list and `li_first` one of its items.
unsafe fn tv_list_unlet_range(
    l: *mut list_T,
    li_first: *mut listitem_T,
    n1_arg: c_int,
    has_n2: bool,
    n2: c_int,
) {
    debug_assert!(!l.is_null());
    let mut li_last = li_first;
    let mut n1 = n1_arg;
    loop {
        let li = unsafe { (*li_last).li_next };
        n1 += 1;
        if li.is_null() || (has_n2 && n2 < n1) {
            break;
        }
        li_last = li;
    }
    unsafe { tv_list_remove_items(l, li_first, li_last) };
}

/// Delete the variable `name[0..name_len]`, reporting E108 if it does not
/// exist and `forceit` is not set.
///
/// # Safety
/// `name` points at `name_len` readable bytes and is NUL-terminated there.
pub unsafe fn do_unlet(name: *const c_char, name_len: size_t, forceit: bool) -> c_int {
    let mut varname: *const c_char = ptr::null();
    let mut dict: *mut dict_T = ptr::null_mut();
    let mut ht = unsafe { find_var_ht_dict(name, name_len, &raw mut varname, &raw mut dict) };

    if !ht.is_null() && unsafe { *varname } != NUL as c_char {
        // The dictionary whose lock decides whether the item may go.
        let mut d = unsafe { get_current_funccal_dict(ht) };
        if d.is_null() {
            if ht == get_globvar_ht() {
                d = get_globvar_dict();
            } else if ht == get_compat_ht() {
                d = get_vimvar_dict();
            } else {
                // The scope's own dictionary item holds it.
                let di = unsafe { find_var_in_ht(ht, *name as c_int, c"".as_ptr(), 0, false) };
                d = unsafe { (*di).di_tv.vval.v_dict };
            }
            if d.is_null() {
                unsafe { internal_error(c"do_unlet()".as_ptr()) };
                return FAIL;
            }
        }

        let mut hi = unsafe { hash_find(ht, varname) };
        if !unsafe { (*hi).is_kept() } {
            hi = unsafe { find_hi_in_scoped_ht(name, &raw mut ht) };
        }
        if !hi.is_null() && unsafe { (*hi).is_kept() } {
            // SAFETY: a kept item of a live variable hashtab.
            let di = unsafe { Di::new(tv_dict_hi2di(hi)) };
            let flags = di.di_flags as c_int;
            let (len, lock) = (TV_CSTRING as size_t, unsafe { (*d).dv_lock });
            if unsafe { var_check_fixed(flags, name, len) }
                || unsafe { var_check_ro(flags, name, len) }
                || unsafe { value_check_lock(lock, name, len) }
            {
                return FAIL;
            }
            // Upstream asks the same question a second time here. It can
            // only answer the same way -- nothing above it changes
            // `dv_lock` -- so the repetition is dead; kept because
            // deleting it is a change no gate could confirm.
            if unsafe { value_check_lock((*d).dv_lock, name, len) } {
                return FAIL;
            }

            let mut oldtv = TV_INITIAL_VALUE;
            let watched = unsafe { tv_dict_is_watched(dict) };
            if watched {
                let tv = di.field_ptr(offset_of!(dictitem_T, di_tv));
                unsafe { tv_copy(tv, &raw mut oldtv) };
            }

            unsafe { delete_var(ht, hi) };

            if watched {
                unsafe { tv_dict_watcher_notify(dict, varname, ptr::null_mut(), &raw mut oldtv) };
                unsafe { tv_clear(&raw mut oldtv) };
            }
            return OK;
        }
    }

    if forceit {
        return OK;
    }
    semsg_c!(
        unsafe { gettext(c"E108: No such variable: \"%s\"".as_ptr()) },
        name
    );
    FAIL
}

/// `:lockvar`'s and `:unlockvar`'s callback: lock or unlock what `lp` names,
/// to `deep` levels.
///
/// # Safety
/// As [`do_unlet_var`].
unsafe fn do_lock_var(
    lp: *mut lval_T,
    _name_end: *mut c_char,
    eap: *mut exarg_T,
    deep: c_int,
) -> c_int {
    // SAFETY: the caller's obligation -- a resolved lvalue and a live
    // command, both of which outlive this call.
    let mut lp = unsafe { Lv::new(lp) };
    let ea = unsafe { Ea::new(eap) };
    let lock = ea.cmdidx as c_int == CMD_lockvar as c_int;
    let name = lp.ll_name;

    if lp.ll_tv.is_null() {
        // A whole variable.
        // SAFETY: a resolved lvalue's name is NUL-terminated.
        if unsafe { *name } == b'$' as c_char {
            // An environment variable has no lock to set.
            semsg_c!(unsafe { gettext(e_lock_unlock.as_ptr()) }, name);
            return FAIL;
        }
        let nil = ptr::null_mut();
        // SAFETY: a resolved lvalue's name and its measured length.
        let di = unsafe { find_var(name, lp.ll_name_len, nil, true) };
        if di.is_null() {
            return FAIL;
        }
        // SAFETY: `find_var` answers a live item or NULL.
        let mut di = unsafe { Di::new(di) };
        let tv = di.field_ptr(offset_of!(dictitem_T, di_tv));
        // A fixed variable -- one of `v:` or a scope dictionary -- can
        // only be locked through the container it holds.
        if di.di_flags & DI_FLAGS_FIX != 0
            && di.di_tv.v_type != VAR_DICT
            && di.di_tv.v_type != VAR_LIST
        {
            semsg_c!(unsafe { gettext(e_lock_unlock.as_ptr()) }, name);
            return FAIL;
        }
        if lock {
            di.di_flags |= DI_FLAGS_LOCK;
        } else {
            di.di_flags &= !DI_FLAGS_LOCK;
        }
        if deep != 0 {
            unsafe { tv_item_lock(tv, deep, lock, false) };
        }
    } else if deep != 0 {
        if lp.ll_range {
            // A range of List items.
            let mut li = lp.ll_li;
            while !li.is_null() && (lp.ll_empty2 || lp.ll_n2 >= lp.ll_n1) {
                // SAFETY: a resolved lvalue's items, walked to the end.
                unsafe { tv_item_lock(&raw mut (*li).li_tv, deep, lock, false) };
                li = unsafe { (*li).li_next };
                lp.ll_n1 += 1;
            }
        } else if !lp.ll_list.is_null() {
            // One List item.
            // SAFETY: a resolved lvalue's own item.
            unsafe { tv_item_lock(&raw mut (*lp.ll_li).li_tv, deep, lock, false) };
        } else {
            // One Dict item.
            unsafe { tv_item_lock(&raw mut (*lp.ll_di).di_tv, deep, lock, false) };
        }
    }
    OK
}
