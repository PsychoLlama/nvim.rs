//! `:unlet`, `:lockvar` and `:unlockvar`.
//!
//! All three share [`ex_unletlock`]'s argument walk and differ only in the
//! callback it is given, so deleting and locking are written here together --
//! as they are upstream.  That one walk is what makes `:unlet` and
//! `:lockvar` agree on what an argument means.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::eval::typval::{
    index_of, tv_list_items, tv_list_iter_mut, tv_list_remove_at, tv_list_remove_range,
};
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::CmdIdx;
use core::ffi::{c_char, c_int};
use core::mem::offset_of;
use core::ptr;

use super::*;
use crate::types::{Failed, NUL};

/// `:unlet`.
///
/// # Safety
/// `args` is a live `:unlet` command.
pub unsafe fn ex_unlet(args: *mut ExArg) {
    // `:unlet!` means "do not complain", which reaches `get_lval` as
    // GLV_QUIET and `do_unlet` as `forceit`.
    // SAFETY: the caller's obligation -- a live command, which the
    // `do_cmdline` frame that owns the `ExArg` outlives.
    let ea = unsafe { Ea::new(args) };
    let glv_flags = if ea.forceit != 0 { GLV_QUIET } else { 0 };
    let arg = ea.arg;
    unsafe { ex_unletlock(args, arg, 0, glv_flags, do_unlet_var) };
}

/// `:lockvar` and `:unlockvar`.
///
/// # Safety
/// `args` is a live `:lockvar`/`:unlockvar` command.
pub unsafe fn ex_lockvar(args: *mut ExArg) {
    // SAFETY: the caller's obligation -- a live command whose argument text
    // is NUL-terminated.
    let ea = unsafe { Ea::new(args) };
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
    unsafe { ex_unletlock(args, arg, deep, 0, do_lock_var) };
}

/// The argument walk `:unlet`, `:lockvar` and `:unlockvar` share, calling
/// `callback` on each name it resolves.
///
/// A failure does not stop the walk: parsing carries on so that the trailing
/// arguments are still checked, but `error` suppresses every later callback.
///
/// # Safety
/// `args` is a live command and `argstart` a NUL-terminated string.
unsafe fn ex_unletlock(
    args: *mut ExArg,
    argstart: *mut c_char,
    deep: c_int,
    glv_flags: c_int,
    callback: UnletLockCallback,
) {
    // SAFETY: the caller's obligation -- a live command and a NUL-terminated
    // argument text, which `arg` and `name_end` both stay inside.
    let mut ea = unsafe { Ea::new(args) };
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
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let arg0 = unsafe { c_str(arg.sub(1)) };
                semsg!("E475: Invalid argument: {arg0}");
                return;
            }
            if !error && ea.skip == 0 && unsafe { callback(lvp, arg, args, deep) }.is_err() {
                error = true;
            }
            name_end = arg;
        } else {
            let quiet = ea.skip != 0 || error;
            name_end = unsafe { get_lval(arg, None, lvp, true, quiet, glv_flags, FNE_CHECK_START) };
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
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let name_end = unsafe { c_str(name_end) };
                    semsg!("E488: Trailing characters: {name_end}");
                }
                if !(ea.skip != 0 || error) {
                    unsafe { clear_lval(lvp) };
                }
                break;
            }

            if !error && ea.skip == 0 && unsafe { callback(lvp, name_end, args, deep) }.is_err() {
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

/// `:unlet`'s callback: delete what `lval` names.
///
/// # Safety
/// `lval` is a resolved lvalue, `name_end` points into the command line and
/// `args` is live.
unsafe fn do_unlet_var(
    lval: *mut LVal,
    name_end: *mut c_char,
    args: *mut ExArg,
    _deep: c_int,
) -> Result<(), Failed> {
    // SAFETY: the caller's obligation -- a resolved lvalue and a live
    // command, both of which outlive this call.
    let lval = unsafe { Lv::new(lval) };
    let ea = unsafe { Ea::new(args) };
    if lval.ll_tv.is_null() {
        // A whole variable: an environment variable, a plain name or an
        // expanded one.  Terminate the name in place, so that the error
        // does not quote the rest of the command.
        // SAFETY: `name_end` points into the command line, and a resolved
        // lvalue's name is NUL-terminated there.
        let cc = unsafe { *name_end };
        unsafe { *name_end = NUL as c_char };
        let ret = if unsafe { *lval.ll_name } == b'$' as c_char {
            unsafe { vim_unsetenv_ext(lval.ll_name.add(1)) };
            Ok(())
        } else {
            unsafe { do_unlet(lval.ll_name, lval.ll_name_len, ea.forceit != 0) }
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
    if !lval.ll_list.is_null() {
        let lock = unsafe { tv_list_locked(lval.ll_list) };
        locked = unsafe { value_check_lock(lock, lval.ll_name, lval.ll_name_len) };
    }
    if !locked && !lval.ll_dict.is_null() {
        let lock = unsafe { (*lval.ll_dict).dv_lock };
        locked = unsafe { value_check_lock(lock, lval.ll_name, lval.ll_name_len) };
    }
    if locked {
        return Err(Failed);
    }

    if lval.ll_range {
        let (n1, n2, to_end) = (lval.ll_n1, lval.ll_n2, !lval.ll_empty2);
        // SAFETY: a resolved lvalue's list and the item it starts at.
        unsafe { tv_list_unlet_range(lval.ll_list, lval.ll_li, n1, to_end, n2) };
    } else if !lval.ll_list.is_null() {
        // One List item.
        unsafe { tv_list_remove_at(lval.ll_list, lval.ll_li) };
    } else {
        // One Dict item.
        let d = lval.ll_dict;
        debug_assert!(!d.is_null());
        // SAFETY: a resolved lvalue's item of that dictionary.
        let di = unsafe { Di::new(lval.ll_di) };
        let watched = unsafe { tv_dict_is_watched(d) };

        let mut oldtv = TV_INITIAL_VALUE;
        let mut key: *mut c_char = ptr::null_mut();
        if watched {
            let tv = di.field_ptr::<TypVal>(offset_of!(DictItem, di_tv));
            unsafe { tv_copy(&*tv, &mut oldtv) };
            // The key has to be saved: removing the item frees it.
            key = unsafe { xstrdup(tv_dict_item_key(di.raw())) };
        }

        unsafe { tv_dict_item_remove(d, di.raw()) };

        if watched {
            unsafe { tv_dict_watcher_notify(d, key, None, Some(&oldtv)) };
            clear_local(&mut oldtv);
            unsafe { xfree(key.cast()) };
        }
    }
    Ok(())
}

/// Delete the items of `l` from `first` through the `n2`-th, or to the
/// end when `has_n2` is false.
///
/// # Safety
/// `l` is a live list and `first` an index into it.
unsafe fn tv_list_unlet_range(l: *mut List, first: usize, n1: c_int, has_n2: bool, n2: c_int) {
    debug_assert!(!l.is_null());
    // SAFETY: the caller's promise: a live list.
    let len = unsafe { tv_list_items(l) }.len();
    // The run ends at `n2` when there is one, and at the last item either
    // way.  An empty list has no run at all; `get_lval` refuses the index
    // that would name one, so this only guards the arithmetic.
    let Some(end) = len.checked_sub(1) else {
        return;
    };
    let last = if has_n2 {
        first + usize::try_from(n2 - n1).unwrap_or(0)
    } else {
        end
    };
    // SAFETY: as above; `first..=last` is a run of the list's items.
    unsafe { tv_list_remove_range(l, first, last.min(end)) };
}

/// Delete the variable `name[0..name_len]`, reporting E108 if it does not
/// exist and `forceit` is not set.
///
/// # Safety
/// `name` points at `name_len` readable bytes and is NUL-terminated there.
pub unsafe fn do_unlet(name: *const c_char, name_len: size_t, forceit: bool) -> Result<(), Failed> {
    let mut varname: *const c_char = ptr::null();
    let mut dict: *mut Dict = ptr::null_mut();
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
                d = unsafe { (*di).di_tv.dict_or_null() };
            }
            if d.is_null() {
                unsafe { internal_error(c"do_unlet()".as_ptr()) };
                return Err(Failed);
            }
        }

        let found = unsafe { hash_find(ht, varname) };
        let hi = if found.is_kept() {
            Some(found)
        } else {
            unsafe { find_hi_in_scoped_ht(name, &raw mut ht) }
        };
        if let Some(hi) = hi.filter(|hi| hi.is_kept()) {
            // SAFETY: a kept item of a live variable hashtab.
            let di = unsafe { Di::new(tv_dict_hi2di(hi)) };
            let flags = di.di_flags as c_int;
            let (len, lock) = (TV_CSTRING as size_t, unsafe { (*d).dv_lock });
            if unsafe { var_check_fixed(flags, name, len) }
                || unsafe { var_check_ro(flags, name, len) }
                || unsafe { value_check_lock(lock, name, len) }
            {
                return Err(Failed);
            }
            // Upstream asks the same question a second time here. It can
            // only answer the same way -- nothing above it changes
            // `dv_lock` -- so the repetition is dead; kept because
            // deleting it is a change no gate could confirm.
            if unsafe { value_check_lock((*d).dv_lock, name, len) } {
                return Err(Failed);
            }

            let mut oldtv = TV_INITIAL_VALUE;
            let watched = unsafe { tv_dict_is_watched(dict) };
            if watched {
                let tv = di.field_ptr::<TypVal>(offset_of!(DictItem, di_tv));
                unsafe { tv_copy(&*tv, &mut oldtv) };
            }

            unsafe { delete_var(ht, hi) };

            if watched {
                unsafe { tv_dict_watcher_notify(dict, varname, None, Some(&oldtv)) };
                clear_local(&mut oldtv);
            }
            return Ok(());
        }
    }

    if forceit {
        return Ok(());
    }
    // SAFETY: a message argument the caller holds as a NUL-terminated string.
    let name = unsafe { c_str(name) };
    semsg!("E108: No such variable: \"{name}\"");
    Err(Failed)
}

/// `:lockvar`'s and `:unlockvar`'s callback: lock or unlock what `lval` names,
/// to `deep` levels.
///
/// # Safety
/// As [`do_unlet_var`].
unsafe fn do_lock_var(
    lval: *mut LVal,
    _name_end: *mut c_char,
    args: *mut ExArg,
    deep: c_int,
) -> Result<(), Failed> {
    // SAFETY: the caller's obligation -- a resolved lvalue and a live
    // command, both of which outlive this call.
    let mut lval = unsafe { Lv::new(lval) };
    let ea = unsafe { Ea::new(args) };
    let lock = ea.cmdidx == CmdIdx::lockvar;
    let name = lval.ll_name;

    if lval.ll_tv.is_null() {
        // A whole variable.
        // SAFETY: a resolved lvalue's name is NUL-terminated.
        if unsafe { *name } == b'$' as c_char {
            // An environment variable has no lock to set.
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let name = unsafe { c_str(name) };
            semsg!("E940: Cannot lock or unlock variable {name}");
            return Err(Failed);
        }
        let nil = ptr::null_mut();
        // SAFETY: a resolved lvalue's name and its measured length.
        let di = unsafe { find_var(name, lval.ll_name_len, nil, true) };
        if di.is_null() {
            return Err(Failed);
        }
        // SAFETY: `find_var` answers a live item or NULL.
        let mut di = unsafe { Di::new(di) };
        // A fixed variable -- one of `v:` or a scope dictionary -- can
        // only be locked through the container it holds.
        if di.di_flags & DI_FLAGS_FIX != 0
            && di.di_tv.v_type() != VAR_DICT
            && di.di_tv.v_type() != VAR_LIST
        {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let name = unsafe { c_str(name) };
            semsg!("E940: Cannot lock or unlock variable {name}");
            return Err(Failed);
        }
        if lock {
            di.di_flags |= DI_FLAGS_LOCK;
        } else {
            di.di_flags &= !DI_FLAGS_LOCK;
        }
        // The value's address is taken after the flag write: it points into
        // the item, and the write goes through a borrow of the whole item.
        let tv = di.field_ptr::<TypVal>(offset_of!(DictItem, di_tv));
        let lock_of = di_lock(di.raw());
        if deep != 0 {
            unsafe { tv_item_lock(lock_of, &mut *tv, deep, lock, false) };
        }
    } else if deep != 0 {
        if !lval.ll_list.is_null() {
            // The one List item the lvalue named, or the run of them a
            // range named -- which ends at `ll_n2` unless the range was
            // open, and at the last item either way.
            let count = if !lval.ll_range {
                1
            } else if lval.ll_empty2 {
                usize::MAX
            } else {
                usize::try_from(lval.ll_n2 - lval.ll_n1 + 1).unwrap_or(0)
            };
            // SAFETY: a resolved lvalue's own list, and `ll_li` an index of it.
            let items = tv_list_iter_mut(unsafe { lval.ll_list.as_mut() });
            let mut done = 0;
            for li in items.skip(lval.ll_li).take(count) {
                // SAFETY: an item of that list.
                unsafe { tv_item_lock(&raw mut li.li_lock, &mut li.li_tv, deep, lock, false) };
                done += 1;
            }
            lval.ll_n1 += index_of(done);
        } else {
            // One Dict item.
            let di = lval.ll_di;
            // SAFETY: a resolved lvalue's own item.
            unsafe { tv_item_lock(di_lock(di), &mut *di_tv(di), deep, lock, false) };
        }
    }
    Ok(())
}
