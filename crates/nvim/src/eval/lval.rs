//! Resolving the left-hand side of an assignment, and performing it.
//!
//! `get_lval` walks a name and its subscripts down to the container and key
//! that `set_var_lval` will write through. The two halves communicate only
//! through `lval_T`, and which of its fields are set is what says *what
//! kind* of assignment this is:
//!
//! | `ll_tv` | `ll_blob` | `ll_newkey` | `ll_range` | the target |
//! | --- | --- | --- | --- | --- |
//! | null | null | — | — | a plain variable, by name |
//! | null | set | — | — | a Blob byte or byte range |
//! | set | — | null | false | an existing List or Dict item |
//! | set | — | set | false | a Dict key that does not exist yet |
//! | set | — | null | true | a List slice |
//!
//! The ownership rule that matters, and the one a tidier rewrite gets
//! wrong: `oldtv` in `set_var_lval` is a *separate* typval from the value
//! being written. It is the value a dictionary's watchers are told the key
//! used to have, it is only filled for a key that already existed, and its
//! being left unset is exactly how the notification tells a new key from an
//! overwritten one. Merging it with anything would notify with the wrong
//! value and then clear it twice.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr::null_mut;

use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::skipwhite;
use crate::eval::executor::eexe_mod_op;
use crate::eval::typval::{
    tv_blob_alloc_ret, tv_blob_check_index, tv_blob_check_range, tv_blob_len, tv_blob_set_append,
    tv_blob_set_range, tv_check_lock, tv_check_str, tv_clear, tv_copy, tv_dict_add, tv_dict_alloc,
    tv_dict_find, tv_dict_is_watched, tv_dict_item_alloc, tv_dict_watcher_notify,
    tv_dict_wrong_func_name, tv_get_number, tv_get_number_chk, tv_get_string, tv_is_func,
    tv_list_alloc_ret, tv_list_assign_range, tv_list_check_range_index_one,
    tv_list_check_range_index_two, value_check_lock,
};
use crate::eval::userfunc::get_funccal_args_ht;
use crate::eval::vars::{
    eval_variable, find_var, get_vimvar_dict, set_var, set_var_const, set_vvar_item, valid_varname,
    var_check_lock, var_check_ro, var_wrong_func_name,
};
use crate::eval::{
    FNE_INCL_BR, GLV_FAIL, GLV_NO_AUTOLOAD, GLV_OK, GLV_QUIET, GLV_READ_ONLY, GLV_STOP, TV_CSTRING,
    e_cannot_slice_dictionary, e_dot_can_only_be_used_on_dictionary_str, e_missbrac, eval_isnamec,
    eval_isnamec1, eval1, find_name_end, glv_status_T, make_expanded_name, tv_init, tv_is_luafunc,
};
use crate::ex_docmd::ends_excmd;
use crate::ex_eval::aborting;
use crate::main::{
    EVALARG_EVALUATE, e_cannot_mod, e_dictkey, e_illvar, e_invalid_value_for_blob_nr, e_invarg2,
    e_letwrong, e_listreq, e_trailing_arg, emsg_severe,
};
use crate::mbyte::utfc_ptr2len;
use crate::memory::{xfree, xmemdupz, xstrdup};
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::strings::vim_strchr;
use crate::types::{
    FAIL, NUL, OK, VAR_BLOB, VAR_DEF_SCOPE, VAR_DICT, VAR_LIST, VAR_UNKNOWN, VAR_UNLOCKED,
    VarLockStatus, dict_T, dictitem_T, hashtab_T, kListLenUnknown, list_T, lval_T, ptrdiff_t,
    size_t, typval_T, typval_vval_union, uint8_t, varnumber_T,
};
use ::libc::{memset, strlen};

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
    vval: typval_vval_union { v_number: 0 },
};

/// The namespace letters a `x:` prefix may use. A `:` anywhere else ends
/// the name.
const NAMESPACES: &core::ffi::CStr = c"bgstvw";

/// The end of the plain name starting at `arg`, or `arg` itself when it
/// does not start one. With `use_namespace`, a single leading `x:` from
/// `NAMESPACES` is part of the name rather than its end.
///
/// # Safety
/// `arg` must be NUL-terminated.
pub(crate) unsafe fn to_name_end(arg: *const c_char, use_namespace: bool) -> *const c_char {
    unsafe {
        if !eval_isnamec1(*arg as c_int) {
            return arg;
        }
        let mut p = arg.add(1);
        while *p as c_int != NUL && eval_isnamec(*p as c_int) {
            if *p == b':' as c_char
                && (p != arg.add(1)
                    || !use_namespace
                    || vim_strchr(NAMESPACES.as_ptr(), *arg as c_int).is_null())
            {
                break;
            }
            p = p.offset(utfc_ptr2len(p as *mut c_char) as isize);
        }
        p
    }
}

/// Resolve one `.key` or `[key]` subscript against the Dictionary in
/// `lp->ll_tv`. `key` is the text for a `.key`; for a `[key]` it is taken
/// from `var1` and `len` is -1.
///
/// Answers `GLV_STOP` when the key does not exist yet and may be added —
/// `ll_newkey` then holds it — and `GLV_FAIL` when it may not.
///
/// # Safety
/// `lp` must be valid with `ll_tv` a Dict; the rest as `get_lval`'s.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn get_lval_dict_item(
    lp: *mut lval_T,
    name: *mut c_char,
    key: *mut c_char,
    len: c_int,
    key_end: *mut *mut c_char,
    var1: *mut typval_T,
    flags: c_int,
    unlet: bool,
    rettv: *mut typval_T,
) -> glv_status_T {
    unsafe {
        let quiet = flags & GLV_QUIET as c_int != 0;
        let p = *key_end;
        // "[key]": the key is `var1`'s string, which is a Number or String.
        let key = if len == -1 {
            tv_get_string(var1) as *mut c_char
        } else {
            key
        };
        (*lp).ll_list = null_mut::<list_T>();

        // A null Dict is an empty Dict; allocate one now.
        if (*(*lp).ll_tv).vval.v_dict.is_null() {
            (*(*lp).ll_tv).vval.v_dict = tv_dict_alloc();
            (*(*(*lp).ll_tv).vval.v_dict).dv_refcount += 1;
        }
        (*lp).ll_dict = (*(*lp).ll_tv).vval.v_dict;
        (*lp).ll_di = tv_dict_find((*lp).ll_dict, key, len as ptrdiff_t);

        // Assigning into a scope dictionary: check that the name is a valid
        // variable name, and a valid *function* name too unless the scope is
        // `l:` or `g:`. Overwriting a builtin function is not allowed.
        if !rettv.is_null() && (*(*lp).ll_dict).dv_scope != 0 {
            // The two checks want a NUL-terminated key, so a `.key` is
            // terminated in place and put back.
            let prevval = if len != -1 {
                let c = *key.offset(len as isize);
                *key.offset(len as isize) = NUL as c_char;
                c
            } else {
                0
            };
            let wrong = ((*(*lp).ll_dict).dv_scope == VAR_DEF_SCOPE
                && tv_is_func(*rettv)
                && var_wrong_func_name(key, (*lp).ll_di.is_null()))
                || !valid_varname(key);
            if len != -1 {
                *key.offset(len as isize) = prevval;
            }
            if wrong {
                return GLV_FAIL;
            }
        }

        if !(*lp).ll_di.is_null()
            && tv_is_luafunc(&raw mut (*(*lp).ll_di).di_tv)
            && len == -1
            && rettv.is_null()
        {
            semsg_c!(e_illvar.as_ptr(), c"v:['lua']".as_ptr());
            return GLV_FAIL;
        }

        if (*lp).ll_di.is_null() {
            // A "v:" or "a:" variable cannot be added.
            if (*lp).ll_dict == get_vimvar_dict()
                || &raw mut (*(*lp).ll_dict).dv_hashtab == get_funccal_args_ht()
            {
                semsg_c!(gettext(e_illvar.as_ptr()), name);
                return GLV_FAIL;
            }
            // The key does not exist. It may be added — unless something
            // follows it to subscript, or this is an `:unlet`.
            if *p == b'[' as c_char || *p == b'.' as c_char || unlet {
                if !quiet {
                    semsg_c!(gettext(e_dictkey.as_ptr()), key);
                }
                return GLV_FAIL;
            }
            (*lp).ll_newkey = if len == -1 {
                xstrdup(key)
            } else {
                xmemdupz(key as *const c_void, len as size_t) as *mut c_char
            };
            *key_end = p;
            return GLV_STOP;
        }

        // An existing item: check it may be changed.
        if flags & GLV_READ_ONLY as c_int == 0
            && (var_check_ro(
                (*(*lp).ll_di).di_flags as c_int,
                name,
                p.offset_from(name) as size_t,
            ) || var_check_lock(
                (*(*lp).ll_di).di_flags as c_int,
                name,
                p.offset_from(name) as size_t,
            ))
        {
            return GLV_FAIL;
        }

        (*lp).ll_tv = &raw mut (*(*lp).ll_di).di_tv;
        GLV_OK
    }
}

/// Resolve a `[n]` or `[n:m]` subscript against the Blob in `lp->ll_tv`.
/// Leaves `ll_tv` null, which is what tells `set_var_lval` this is a Blob.
///
/// # Safety
/// `lp` must be valid with `ll_tv` a Blob; `var1`/`var2` valid.
pub(crate) unsafe fn get_lval_blob(
    lp: *mut lval_T,
    var1: *mut typval_T,
    var2: *mut typval_T,
    empty1: bool,
    quiet: bool,
) -> c_int {
    unsafe {
        let bloblen = tv_blob_len((*(*lp).ll_tv).vval.v_blob);
        (*lp).ll_n1 = if empty1 {
            0
        } else {
            tv_get_number(var1) as c_int
        };
        if tv_blob_check_index(bloblen, (*lp).ll_n1 as varnumber_T, quiet) == FAIL {
            return FAIL;
        }
        if (*lp).ll_range && !(*lp).ll_empty2 {
            (*lp).ll_n2 = tv_get_number(var2) as c_int;
            if tv_blob_check_range(
                bloblen,
                (*lp).ll_n1 as varnumber_T,
                (*lp).ll_n2 as varnumber_T,
                quiet,
            ) == FAIL
            {
                return FAIL;
            }
        }
        (*lp).ll_blob = (*(*lp).ll_tv).vval.v_blob;
        (*lp).ll_tv = null_mut();
        OK
    }
}

/// Resolve a `[n]` or `[n:m]` subscript against the List in `lp->ll_tv`,
/// leaving `ll_tv` on the item it selected.
///
/// # Safety
/// `lp` must be valid with `ll_tv` a List; `var1`/`var2` valid.
pub(crate) unsafe fn get_lval_list(
    lp: *mut lval_T,
    var1: *mut typval_T,
    var2: *mut typval_T,
    empty1: bool,
    _flags: c_int,
    quiet: bool,
) -> c_int {
    unsafe {
        (*lp).ll_n1 = if empty1 {
            0
        } else {
            tv_get_number(var1) as c_int
        };
        (*lp).ll_dict = null_mut::<dict_T>();
        (*lp).ll_list = (*(*lp).ll_tv).vval.v_list;
        (*lp).ll_li = tv_list_check_range_index_one((*lp).ll_list, &raw mut (*lp).ll_n1, quiet);
        if (*lp).ll_li.is_null() {
            return FAIL;
        }
        if (*lp).ll_range && !(*lp).ll_empty2 {
            (*lp).ll_n2 = tv_get_number(var2) as c_int;
            if tv_list_check_range_index_two(
                (*lp).ll_list,
                &raw mut (*lp).ll_n1,
                (*lp).ll_li,
                &raw mut (*lp).ll_n2,
                quiet,
            ) == FAIL
            {
                return FAIL;
            }
        }
        (*lp).ll_tv = &raw mut (*(*lp).ll_li).li_tv;
        OK
    }
}

/// Walk every `[idx]` and `.key` following the name, descending `lp->ll_tv`
/// one container at a time. Answers the cursor after the last subscript, or
/// null on an error.
///
/// # Safety
/// `lp` must be valid with `ll_tv` set; `p` must point into the
/// NUL-terminated `name`.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn get_lval_subscript(
    lp: *mut lval_T,
    mut p: *mut c_char,
    name: *mut c_char,
    rettv: *mut typval_T,
    _ht: *mut hashtab_T,
    _v: *mut dictitem_T,
    unlet: bool,
    flags: c_int,
) -> *mut c_char {
    unsafe {
        let quiet = flags & GLV_QUIET as c_int != 0;
        // The two index expressions. They are cleared and reset at the end
        // of every pass, so an early `return` below never leaks one.
        let mut var1 = UNSET_TV;
        let mut var2 = UNSET_TV;
        let mut empty1 = false;
        let mut rc = FAIL;

        'done: {
            while *p == b'[' as c_char
                || (*p == b'.' as c_char
                    && *p.add(1) != b'=' as c_char
                    && *p.add(1) != b'.' as c_char)
            {
                if *p == b'.' as c_char && (*(*lp).ll_tv).v_type != VAR_DICT {
                    if !quiet {
                        semsg_c!(
                            gettext(e_dot_can_only_be_used_on_dictionary_str.as_ptr()),
                            name,
                        );
                    }
                    return null_mut();
                }
                if (*(*lp).ll_tv).v_type != VAR_LIST
                    && (*(*lp).ll_tv).v_type != VAR_DICT
                    && (*(*lp).ll_tv).v_type != VAR_BLOB
                {
                    if !quiet {
                        emsg(gettext(
                            c"E689: Can only index a List, Dictionary or Blob".as_ptr(),
                        ));
                    }
                    return null_mut();
                }

                // A null List or Blob works like an empty one; allocate now.
                if (*(*lp).ll_tv).v_type == VAR_LIST && (*(*lp).ll_tv).vval.v_list.is_null() {
                    tv_list_alloc_ret((*lp).ll_tv, kListLenUnknown as ptrdiff_t);
                } else if (*(*lp).ll_tv).v_type == VAR_BLOB && (*(*lp).ll_tv).vval.v_blob.is_null()
                {
                    tv_blob_alloc_ret((*lp).ll_tv);
                }

                if (*lp).ll_range {
                    if !quiet {
                        emsg(gettext(c"E708: [:] must come last".as_ptr()));
                    }
                    break 'done;
                }

                let mut len: c_int = -1;
                let mut key: *mut c_char = null_mut();
                if *p == b'.' as c_char {
                    key = p.add(1);
                    len = 0;
                    while {
                        let b = *key.offset(len as isize) as u8;
                        b.is_ascii_alphabetic()
                            || ascii_isdigit(*key.offset(len as isize) as c_int)
                            || b == b'_'
                    } {
                        len += 1;
                    }
                    if len == 0 {
                        if !quiet {
                            emsg(gettext(c"E713: Cannot use empty key after .".as_ptr()));
                        }
                        return null_mut();
                    }
                    p = key.offset(len as isize);
                } else {
                    // The index `[expr]`, or the first of `[expr : expr]`.
                    p = skipwhite(p.add(1));
                    if *p == b':' as c_char {
                        empty1 = true;
                    } else {
                        empty1 = false;
                        if eval1(&raw mut p, &raw mut var1, EVALARG_EVALUATE.ptr()) == FAIL {
                            break 'done;
                        }
                        if !tv_check_str(&raw mut var1) {
                            break 'done;
                        }
                        p = skipwhite(p);
                    }

                    if *p == b':' as c_char {
                        if (*(*lp).ll_tv).v_type == VAR_DICT {
                            if !quiet {
                                emsg(gettext(e_cannot_slice_dictionary.as_ptr()));
                            }
                            break 'done;
                        }
                        // The value being assigned has to be sliceable too.
                        if !rettv.is_null()
                            && !((*rettv).v_type == VAR_LIST && !(*rettv).vval.v_list.is_null())
                            && !((*rettv).v_type == VAR_BLOB && !(*rettv).vval.v_blob.is_null())
                        {
                            if !quiet {
                                emsg(gettext(c"E709: [:] requires a List or Blob value".as_ptr()));
                            }
                            break 'done;
                        }
                        p = skipwhite(p.add(1));
                        if *p == b']' as c_char {
                            (*lp).ll_empty2 = true;
                        } else {
                            (*lp).ll_empty2 = false;
                            if eval1(&raw mut p, &raw mut var2, EVALARG_EVALUATE.ptr()) == FAIL {
                                break 'done;
                            }
                            if !tv_check_str(&raw mut var2) {
                                break 'done;
                            }
                        }
                        (*lp).ll_range = true;
                    } else {
                        (*lp).ll_range = false;
                    }

                    if *p != b']' as c_char {
                        if !quiet {
                            emsg(gettext(e_missbrac.as_ptr()));
                        }
                        break 'done;
                    }
                    p = p.add(1);
                }

                if (*(*lp).ll_tv).v_type == VAR_DICT {
                    match get_lval_dict_item(
                        lp,
                        name,
                        key,
                        len,
                        &raw mut p,
                        &raw mut var1,
                        flags,
                        unlet,
                        rettv,
                    ) {
                        GLV_FAIL => break 'done,
                        // The key is new: `ll_newkey` holds it and there is
                        // nothing left to descend into.
                        GLV_STOP => break,
                        _ => {}
                    }
                } else if (*(*lp).ll_tv).v_type == VAR_BLOB {
                    if get_lval_blob(lp, &raw mut var1, &raw mut var2, empty1, quiet) == FAIL {
                        break 'done;
                    }
                    // A Blob byte is never a container, so this is the end.
                    break;
                } else if get_lval_list(lp, &raw mut var1, &raw mut var2, empty1, flags, quiet)
                    == FAIL
                {
                    break 'done;
                }

                tv_clear(&raw mut var1);
                tv_clear(&raw mut var2);
                var1.v_type = VAR_UNKNOWN;
                var2.v_type = VAR_UNKNOWN;
            }
            rc = OK;
        }

        tv_clear(&raw mut var1);
        tv_clear(&raw mut var2);
        if rc == OK { p } else { null_mut() }
    }
}

/// Resolve the left-hand side of an assignment or an `:unlet` into `lp`,
/// and answer the cursor after it.
///
/// # Safety
/// `name` must be a writable, NUL-terminated string; `lp` must be valid;
/// `rettv` null or the value about to be assigned.
pub unsafe fn get_lval(
    name: *mut c_char,
    rettv: *mut typval_T,
    lp: *mut lval_T,
    unlet: bool,
    skip: bool,
    flags: c_int,
    fne_flags: c_int,
) -> *mut c_char {
    unsafe {
        let quiet = flags & GLV_QUIET as c_int != 0;
        memset(lp as *mut c_void, 0, size_of::<lval_T>());

        if skip {
            // Only the name matters; nothing is resolved.
            (*lp).ll_name = name;
            return find_name_end(name, null_mut(), null_mut(), FNE_INCL_BR | fne_flags)
                as *mut c_char;
        }

        // `find_name_end` writes `*const` and `make_expanded_name` wants
        // `*mut`; the two spell the same bytes of `name`, which is writable.
        let mut expr_start: *mut c_char = null_mut();
        let mut expr_end: *mut c_char = null_mut();
        let mut p = find_name_end(
            name,
            (&raw mut expr_start).cast::<*const c_char>(),
            (&raw mut expr_end).cast::<*const c_char>(),
            fne_flags,
        ) as *mut c_char;

        if !expr_start.is_null() {
            // A curly-braces name: expand it.
            if unlet
                && !ascii_iswhite(*p as c_int)
                && ends_excmd(*p as c_int) == 0
                && *p != b'[' as c_char
                && *p != b'.' as c_char
            {
                semsg_c!(gettext(e_trailing_arg.as_ptr()), p);
                return null_mut();
            }
            (*lp).ll_exp_name = make_expanded_name(name, expr_start, expr_end, p);
            (*lp).ll_name = (*lp).ll_exp_name;
            if (*lp).ll_exp_name.is_null() {
                if !aborting() && !quiet {
                    emsg_severe.set(true);
                    semsg_c!(gettext(e_invarg2.as_ptr()), name);
                    return null_mut();
                }
                (*lp).ll_name_len = 0 as size_t;
            } else {
                (*lp).ll_name_len = strlen((*lp).ll_name);
            }
        } else {
            (*lp).ll_name = name;
            (*lp).ll_name_len = p.offset_from((*lp).ll_name) as size_t;
        }

        // Nothing is subscripted: the name is the whole left-hand side.
        if (*p != b'[' as c_char && *p != b'.' as c_char) || (*lp).ll_name.is_null() {
            return p;
        }

        let mut ht: *mut hashtab_T = null_mut();
        let v = find_var(
            (*lp).ll_name,
            (*lp).ll_name_len,
            if flags & GLV_READ_ONLY as c_int != 0 {
                null_mut()
            } else {
                &raw mut ht
            },
            flags & GLV_NO_AUTOLOAD as c_int != 0,
        );
        if v.is_null() {
            if !quiet {
                semsg_c!(
                    gettext(c"E121: Undefined variable: %.*s".as_ptr()),
                    (*lp).ll_name_len as c_int,
                    (*lp).ll_name,
                );
            }
            return null_mut();
        }

        (*lp).ll_tv = &raw mut (*v).di_tv;
        if tv_is_luafunc((*lp).ll_tv) {
            return p;
        }

        p = get_lval_subscript(lp, p, name, rettv, ht, v, unlet, flags);
        if p.is_null() {
            return null_mut();
        }
        (*lp).ll_name_len = p.offset_from((*lp).ll_name) as size_t;
        p
    }
}

/// Release what `get_lval` allocated into `lp`.
///
/// # Safety
/// `lp` must be valid.
pub unsafe fn clear_lval(lp: *mut lval_T) {
    unsafe {
        xfree((*lp).ll_exp_name as *mut c_void);
        xfree((*lp).ll_newkey as *mut c_void);
    }
}

/// Perform the assignment `get_lval` resolved. `endp` is the cursor after
/// the left-hand side, which is terminated in place while a message might
/// name the variable.
///
/// # Safety
/// `lp` must come from `get_lval`; `endp` must point into the same writable
/// string; `rettv` must be valid.
pub unsafe fn set_var_lval(
    lp: *mut lval_T,
    endp: *mut c_char,
    rettv: *mut typval_T,
    copy: bool,
    is_const: bool,
    op: *const c_char,
) {
    unsafe {
        if (*lp).ll_tv.is_null() {
            set_whole_var(lp, endp, rettv, copy, is_const, op);
            return;
        }

        // A locked container refuses the write; the lock to test is the
        // Dict's own when a key is being added to it.
        let lock = if (*lp).ll_newkey.is_null() {
            (*(*lp).ll_tv).v_lock as VarLockStatus
        } else {
            (*(*(*lp).ll_tv).vval.v_dict).dv_lock as VarLockStatus
        };
        if value_check_lock(lock, (*lp).ll_name, TV_CSTRING as size_t) {
            return;
        }

        if (*lp).ll_range {
            if is_const {
                emsg(gettext(c"E996: Cannot lock a range".as_ptr()));
                return;
            }
            // Crash fix, upstream reads the union the wrong way here: the
            // lval resolver accepts a Blob value for a `[:]` because the
            // *target* may be a Blob, but a Blob target leaves `ll_tv`
            // null and never reaches this branch. So a Blob reaching it
            // means a List target, and upstream hands its `v_blob` to
            // `tv_list_assign_range` through `vval.v_list` — walking a
            // `blob_T` as a `list_T`. `let l = [1,2] | let l[0:] = 0z11`
            // is enough. Report what the assignment actually needs.
            if (*rettv).v_type != VAR_LIST {
                emsg(gettext(e_listreq.as_ptr()));
                return;
            }
            tv_list_assign_range(
                (*lp).ll_list,
                (*rettv).vval.v_list,
                (*lp).ll_n1,
                (*lp).ll_n2,
                (*lp).ll_empty2,
                op,
                (*lp).ll_name,
            );
            return;
        }

        // The value the watchers are told the key used to have. It stays
        // unset for a key that did not exist, and that is how the
        // notification below tells the two cases apart — see the module
        // docs. It must never be the same typval as the new value.
        let mut oldtv = UNSET_TV;
        let dict = (*lp).ll_dict;
        let watched = tv_dict_is_watched(dict);

        if is_const {
            emsg(gettext(c"E996: Cannot lock a list or dict".as_ptr()));
            return;
        }

        // Writing an *existing* key of the `v:` scope dictionary is a write
        // to a `v:` variable, and has to pass the same type enforcement the
        // unsubscripted spelling does. Upstream stores straight into the
        // item, which permanently re-types the variable and, for
        // `v:oldfiles`, crashes the next reader (docket O-B14-10). A new key
        // cannot happen here: `get_lval` refuses to add one to `v:`.
        if dict == get_vimvar_dict() && (*lp).ll_newkey.is_null() {
            set_vvar_item((*lp).ll_di, rettv, copy, op);
            return;
        }

        'notify: {
            if !(*lp).ll_newkey.is_null() {
                // The key has to be added to the Dictionary first.
                if !op.is_null() && *op != b'=' as c_char {
                    semsg_c!(gettext(e_dictkey.as_ptr()), (*lp).ll_newkey);
                    return;
                }
                if tv_dict_wrong_func_name((*(*lp).ll_tv).vval.v_dict, rettv, (*lp).ll_newkey) != 0
                {
                    return;
                }
                let di = tv_dict_item_alloc((*lp).ll_newkey);
                if tv_dict_add((*(*lp).ll_tv).vval.v_dict, di) == FAIL {
                    xfree(di as *mut c_void);
                    return;
                }
                (*lp).ll_tv = &raw mut (*di).di_tv;
            } else {
                if watched {
                    tv_copy((*lp).ll_tv, &raw mut oldtv);
                }
                if !op.is_null() && *op != b'=' as c_char {
                    // `+=` and friends modify in place; there is nothing to
                    // assign afterwards.
                    eexe_mod_op((*lp).ll_tv, rettv, op);
                    break 'notify;
                }
                tv_clear((*lp).ll_tv);
            }

            if copy {
                tv_copy(rettv, (*lp).ll_tv);
            } else {
                *(*lp).ll_tv = *rettv;
                (*(*lp).ll_tv).v_lock = VAR_UNLOCKED;
                tv_init(rettv);
            }
        }

        if !watched {
            return;
        }
        if oldtv.v_type == VAR_UNKNOWN {
            // Nothing was saved, so this is the new-key case.
            debug_assert!(!(*lp).ll_newkey.is_null());
            tv_dict_watcher_notify(dict, (*lp).ll_newkey, (*lp).ll_tv, null_mut());
        } else {
            let di = (*lp).ll_di;
            debug_assert!(!(&raw mut (*di).di_key as *mut c_char).is_null());
            tv_dict_watcher_notify(
                dict,
                &raw mut (*di).di_key as *mut c_char,
                (*lp).ll_tv,
                &raw mut oldtv,
            );
            tv_clear(&raw mut oldtv);
        }
    }
}

/// The `ll_tv == NULL` half of `set_var_lval`: the target is a whole
/// variable by name, or a Blob byte or byte range.
///
/// # Safety
/// As `set_var_lval`.
unsafe fn set_whole_var(
    lp: *mut lval_T,
    endp: *mut c_char,
    rettv: *mut typval_T,
    copy: bool,
    is_const: bool,
    op: *const c_char,
) {
    unsafe {
        // Terminate the left-hand side in place: the messages below name the
        // variable and would otherwise print the rest of the command too.
        let cc = *endp;
        *endp = NUL as c_char;

        if !(*lp).ll_blob.is_null() {
            // Upstream's three early returns here leave the left-hand side
            // terminated in place rather than putting `cc` back. Preserved:
            // anything that reads the command line after a rejected Blob
            // assignment sees the truncated form.
            if !set_blob_var(lp, rettv, op) {
                return;
            }
        } else if !op.is_null() && *op != b'=' as c_char {
            // `+=`, `-=`, `*=`, `/=`, `%=` and `..=`.
            if is_const {
                emsg(gettext(e_cannot_mod.as_ptr()));
                *endp = cc;
                return;
            }
            let mut tv = UNSET_TV;
            let mut di: *mut dictitem_T = null_mut();
            if eval_variable(
                (*lp).ll_name,
                (*lp).ll_name_len as c_int,
                &raw mut tv,
                &raw mut di,
                true,
                false,
            ) == OK
            {
                if (di.is_null()
                    || (!var_check_ro(
                        (*di).di_flags as c_int,
                        (*lp).ll_name,
                        TV_CSTRING as size_t,
                    ) && !tv_check_lock(
                        &raw mut (*di).di_tv,
                        (*lp).ll_name,
                        TV_CSTRING as size_t,
                    )))
                    && eexe_mod_op(&raw mut tv, rettv, op) == OK
                {
                    set_var((*lp).ll_name, (*lp).ll_name_len, &raw mut tv, false);
                }
                tv_clear(&raw mut tv);
            }
        } else {
            set_var_const((*lp).ll_name, (*lp).ll_name_len, rettv, copy, is_const);
        }

        *endp = cc;
    }
}

/// Write a byte or a byte range into the Blob `lp` resolved. Answers
/// whether the caller should put the terminated left-hand side back — the
/// three refusal paths say no, which is upstream's.
///
/// # Safety
/// As `set_var_lval`, with `lp->ll_blob` set.
unsafe fn set_blob_var(lp: *mut lval_T, rettv: *mut typval_T, op: *const c_char) -> bool {
    unsafe {
        if !op.is_null() && *op != b'=' as c_char {
            semsg_c!(gettext(e_letwrong.as_ptr()), op);
            return false;
        }
        if value_check_lock(
            (*(*lp).ll_blob).bv_lock,
            (*lp).ll_name,
            TV_CSTRING as size_t,
        ) {
            return false;
        }

        if (*lp).ll_range && (*rettv).v_type == VAR_BLOB {
            if (*lp).ll_empty2 {
                (*lp).ll_n2 = tv_blob_len((*lp).ll_blob) - 1;
            }
            if tv_blob_set_range(
                (*lp).ll_blob,
                (*lp).ll_n1 as varnumber_T,
                (*lp).ll_n2 as varnumber_T,
                rettv,
            ) == FAIL
            {
                return false;
            }
            return true;
        }

        let mut error = false;
        let val = tv_get_number_chk(rettv, &raw mut error);
        if !error {
            if !(0..=255).contains(&val) {
                semsg_c!(gettext(e_invalid_value_for_blob_nr.as_ptr()), val);
            } else {
                tv_blob_set_append((*lp).ll_blob, (*lp).ll_n1, val as uint8_t);
            }
        }
        true
    }
}
