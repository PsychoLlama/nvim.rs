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
use core::mem::{offset_of, size_of};
use core::ptr::null_mut;

use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::skipwhite;
use crate::eval::EVALARG_EVALUATE;
use crate::eval::executor::eexe_mod_op;
use crate::eval::typval::{
    NumBuf, tv_blob_alloc_ret, tv_blob_check_index, tv_blob_check_range, tv_blob_len,
    tv_blob_set_append, tv_blob_set_range, tv_check_lock, tv_check_str, tv_clear, tv_copy,
    tv_dict_add, tv_dict_alloc, tv_dict_find, tv_dict_is_watched, tv_dict_item_alloc,
    tv_dict_watcher_notify, tv_dict_wrong_func_name, tv_get_number, tv_get_number_chk, tv_is_func,
    tv_list_alloc_ret, tv_list_assign_range, tv_list_check_range_index_one,
    tv_list_check_range_index_two, value_check_lock,
};
use crate::eval::userfunc::get_funccal_args_ht;
use crate::eval::vars::{clear_local, emsg_static};
use crate::eval::vars::{
    eval_variable, find_var, get_vimvar_dict, set_var, set_var_const, set_vvar_item, valid_varname,
    var_check_lock, var_check_ro, var_wrong_func_name,
};
use crate::eval::{
    FNE_INCL_BR, GLV_FAIL, GLV_NO_AUTOLOAD, GLV_OK, GLV_QUIET, GLV_READ_ONLY, GLV_STOP, TV_CSTRING,
    e_cannot_slice_dictionary, e_dot_can_only_be_used_on_dictionary_str, e_missbrac, eval_isnamec,
    eval_isnamec1, eval1, find_name_end, glv_status_T, make_expanded_name, tv_init, tv_is_luafunc,
};
use crate::eval::{Lv, Tv};
use crate::ex_docmd::ends_excmd;
use crate::ex_eval::aborting;
use crate::main::{
    e_cannot_mod, e_dictkey, e_illvar, e_invalid_value_for_blob_nr, e_invarg2, e_letwrong,
    e_listreq, e_trailing_arg, emsg_severe,
};
use crate::mbyte::utfc_ptr2len;
use crate::memory::{xfree, xmemdupz, xstrdup};
use crate::os::cshim::gettext;
use crate::strings::vim_strchr;
use crate::types::{
    FAIL, NUL, OK, VAR_BLOB, VAR_DEF_SCOPE, VAR_DICT, VAR_LIST, VAR_UNKNOWN, VarLock, dict_T,
    dictitem_T, hashtab_T, kListLenUnknown, list_T, lval_T, ptrdiff_t, size_t, typval_T,
    typval_vval_union, uint8_t, varnumber_T,
};
use ::libc::{memset, strlen};

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VarLock::Unlocked,
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
    // SAFETY: the caller's promise -- `arg` is NUL-terminated, so its first byte is readable.
    let first = unsafe { *arg };
    if !eval_isnamec1(first as c_int) {
        return arg;
    }
    // SAFETY: a name character is not the terminator, so the byte after it is inside the string.
    let start = unsafe { arg.add(1) };
    let mut p = start;
    loop {
        // SAFETY: `p` walks the string and every step stops at the terminator.
        let c = unsafe { *p };
        if c as c_int == NUL || !eval_isnamec(c as c_int) {
            break;
        }
        if c == b':' as c_char {
            // A `:` continues the name only as the one namespace letter.
            // SAFETY: `NAMESPACES` is a NUL-terminated literal.
            let namespaced = use_namespace
                && p == start
                && !unsafe { vim_strchr(NAMESPACES.as_ptr(), first as c_int) }.is_null();
            if !namespaced {
                break;
            }
        }
        // SAFETY: `c` is not the terminator, so `p` is on a character.
        p = unsafe { p.offset(utfc_ptr2len(p as *mut c_char) as isize) };
    }
    p
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
    let mut numbuf = NumBuf::new();
    let quiet = flags & GLV_QUIET as c_int != 0;
    // SAFETY: the caller's promise; `key_end` holds a cursor into `name`.
    let (mut lp, p) = unsafe { (Lv::new(lp), *key_end) };
    // SAFETY: the caller's promise: `ll_tv` holds a Dict, so `v_dict` is live.
    let mut container = unsafe { Tv::new(lp.ll_tv) };
    // "[key]": the key is `var1`'s string, which is a Number or String.
    let key = if len == -1 {
        // SAFETY: `var1` is the caller's, and `numbuf` outlives the string rendered into it.
        unsafe { numbuf.string(var1) as *mut c_char }
    } else {
        key
    };
    lp.ll_list = null_mut::<list_T>();

    // A null Dict is an empty Dict; allocate one now.
    // SAFETY: as above.
    if unsafe { container.vval.v_dict }.is_null() {
        // SAFETY: `tv_dict_alloc` never answers NULL.
        container.vval.v_dict = unsafe { tv_dict_alloc() };
        // SAFETY: the typval now holds the reference this takes.
        unsafe { (*container.vval.v_dict).dv_refcount.retain() };
    }
    // SAFETY: as above.
    lp.ll_dict = unsafe { container.vval.v_dict };
    // SAFETY: `ll_dict` is a live Dict, and `key` is NUL-terminated or `len` bytes long.
    lp.ll_di = unsafe { tv_dict_find(lp.ll_dict, key, len as ptrdiff_t) };
    // `dict_T` holds a `hashtab_T`, which points at its own inline array,
    // so it is not a pointee a `Live` may wrap — see `winlayer::live`'s
    // note. The one field this needs is read through the pointer.
    // SAFETY: `ll_dict` is the live Dict just resolved.
    let dv_scope = unsafe { (*lp.ll_dict).dv_scope };

    // Assigning into a scope dictionary: check that the name is a valid
    // variable name, and a valid *function* name too unless the scope is
    // `l:` or `g:`. Overwriting a builtin function is not allowed.
    if !rettv.is_null() && dv_scope != 0 {
        // The two checks want a NUL-terminated key, so a `.key` is
        // terminated in place and put back.
        // SAFETY: a `.key`'s `len` bytes are inside the writable `name`.
        let prevval = if len != -1 {
            unsafe { *key.offset(len as isize) }
        } else {
            0
        };
        if len != -1 {
            // SAFETY: as above.
            unsafe { *key.offset(len as isize) = NUL as c_char };
        }
        // SAFETY: `rettv` is the caller's, and `key` is NUL-terminated either way now.
        let existing = lp.ll_di.is_null();
        let wrong = (dv_scope == VAR_DEF_SCOPE
            && unsafe { tv_is_func(*rettv) }
            && unsafe { var_wrong_func_name(key, existing) })
            || !unsafe { valid_varname(key) };
        if len != -1 {
            // SAFETY: as above -- the byte cut out is put back.
            unsafe { *key.offset(len as isize) = prevval };
        }
        if wrong {
            return GLV_FAIL;
        }
    }

    // SAFETY: a non-null `ll_di` is a live dictionary item.
    let lua_key = !lp.ll_di.is_null()
        && unsafe { tv_is_luafunc(&raw mut (*lp.ll_di).di_tv) }
        && len == -1
        && rettv.is_null();
    if lua_key {
        let what = c"v:['lua']".as_ptr();
        // SAFETY: the format takes one NUL-terminated string.
        unsafe { semsg_c!(e_illvar.as_ptr(), what) };
        return GLV_FAIL;
    }

    if lp.ll_di.is_null() {
        // A "v:" or "a:" variable cannot be added.
        // SAFETY: naming a live Dict's hashtab reads nothing.
        let ht = unsafe { &raw mut (*lp.ll_dict).dv_hashtab };
        // SAFETY: the function-call stack is the editor's own.
        let args_ht = unsafe { get_funccal_args_ht() };
        if lp.ll_dict == get_vimvar_dict() || ht == args_ht {
            // SAFETY: the format takes one NUL-terminated string.
            unsafe { semsg_c!(gettext(e_illvar.as_ptr()), name) };
            return GLV_FAIL;
        }
        // The key does not exist. It may be added — unless something
        // follows it to subscript, or this is an `:unlet`.
        // SAFETY: `p` is a cursor into the NUL-terminated `name`.
        let after = unsafe { *p };
        if after == b'[' as c_char || after == b'.' as c_char || unlet {
            if !quiet {
                // SAFETY: the format takes one NUL-terminated string.
                unsafe { semsg_c!(gettext(e_dictkey.as_ptr()), key) };
            }
            return GLV_FAIL;
        }
        // SAFETY: `key` is NUL-terminated when `len` is -1, and `len` bytes long otherwise.
        lp.ll_newkey = if len == -1 {
            unsafe { xstrdup(key) }
        } else {
            unsafe { xmemdupz(key as *const c_void, len as size_t) as *mut c_char }
        };
        // SAFETY: the caller's promise about `key_end`.
        unsafe { *key_end = p };
        return GLV_STOP;
    }

    // An existing item: check it may be changed.
    // SAFETY: `ll_di` is a live item, and `p` and `name` are cursors into the one string.
    let di_flags = unsafe { (*lp.ll_di).di_flags } as c_int;
    // SAFETY: as above.
    let name_len = unsafe { p.offset_from(name) } as size_t;
    let refused = flags & GLV_READ_ONLY as c_int == 0
        && (unsafe { var_check_ro(di_flags, name, name_len) }
            || unsafe { var_check_lock(di_flags, name, name_len) });
    if refused {
        return GLV_FAIL;
    }

    // SAFETY: `ll_di` is a live item, whose typval is the target.
    lp.ll_tv = unsafe { &raw mut (*lp.ll_di).di_tv };
    GLV_OK
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
    // SAFETY: the caller's promise: `ll_tv` holds a Blob, so `v_blob` is live.
    let mut lp = unsafe { Lv::new(lp) };
    // SAFETY: as above.
    let bloblen = unsafe { tv_blob_len(Tv::new(lp.ll_tv).vval.v_blob) };
    lp.ll_n1 = if empty1 {
        0
    } else {
        // SAFETY: `var1` is the caller's index expression.
        unsafe { tv_get_number(var1) as c_int }
    };
    let n1 = lp.ll_n1 as varnumber_T;
    // SAFETY: the index is checked against the length measured above.
    if unsafe { tv_blob_check_index(bloblen, n1, quiet) } == FAIL {
        return FAIL;
    }
    if lp.ll_range && !lp.ll_empty2 {
        // SAFETY: `var2` is the caller's second index expression.
        lp.ll_n2 = unsafe { tv_get_number(var2) as c_int };
        let n2 = lp.ll_n2 as varnumber_T;
        // SAFETY: as above.
        if unsafe { tv_blob_check_range(bloblen, n1, n2, quiet) } == FAIL {
            return FAIL;
        }
    }
    // SAFETY: as above -- the typval still holds the Blob.
    lp.ll_blob = unsafe { Tv::new(lp.ll_tv).vval.v_blob };
    lp.ll_tv = null_mut();
    OK
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
    // The two range callees write back through the addresses of `ll_n1` and
    // `ll_n2`, so those addresses are live across the rest of this body.
    // Every field written below therefore goes through the pointer rather
    // than through `DerefMut`, which would borrow the whole record and pop
    // them — `winlayer::live`'s note, and the bug it names in
    // `set_buflocal_cpt_callbacks`.
    // SAFETY: the caller's promise: `lp` outlives the call with `ll_tv`
    // holding a List.
    let lp = unsafe { Lv::new(lp) };
    let (rec, n1, n2) = (
        lp.raw(),
        lp.field_ptr::<c_int>(offset_of!(lval_T, ll_n1)),
        lp.field_ptr::<c_int>(offset_of!(lval_T, ll_n2)),
    );
    let first = if empty1 {
        0
    } else {
        // SAFETY: `var1` is the caller's index expression.
        unsafe { tv_get_number(var1) as c_int }
    };
    // SAFETY: `VAR_LIST` says `v_list` is the union's live member, and
    // `rec` is the caller's record.
    unsafe {
        *n1 = first;
        (*rec).ll_dict = null_mut::<dict_T>();
        (*rec).ll_list = Tv::new((*rec).ll_tv).vval.v_list;
    };
    // SAFETY: `ll_list` is the typval's List and `n1` is `lp`'s own field.
    let (list, li) = unsafe {
        let list = (*rec).ll_list;
        let li = tv_list_check_range_index_one(list, n1, quiet);
        (*rec).ll_li = li;
        (list, li)
    };
    if li.is_null() {
        return FAIL;
    }
    // SAFETY: `rec` is the caller's record.
    let ranged = unsafe { (*rec).ll_range && !(*rec).ll_empty2 };
    if ranged {
        // SAFETY: `var2` is the caller's second index expression, and both
        // indexes are `lp`'s own fields.
        unsafe { *n2 = tv_get_number(var2) as c_int };
        // SAFETY: `li` is the item index one selected.
        if unsafe { tv_list_check_range_index_two(list, n1, li, n2, quiet) } == FAIL {
            return FAIL;
        }
    }
    // SAFETY: `ll_li` is a live item, whose typval is the target.
    unsafe { (*rec).ll_tv = &raw mut (*li).li_tv };
    OK
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
    let mut evalarg = EVALARG_EVALUATE;
    let quiet = flags & GLV_QUIET as c_int != 0;
    // SAFETY, for every region in this body: the caller's promise is that
    // `lp` outlives the call with `ll_tv` on a live typval, and that `p` is
    // a cursor into the NUL-terminated `name` — which the walk below keeps
    // true, because it never steps past a byte it has not first seen is not
    // the terminator. Each union member read is the one the `v_type` just
    // tested names; `var1`, `var2`, `evalarg` and `p` are this frame's; and
    // every message named is a NUL-terminated literal or one of the shared
    // `e_*` texts. The notes below add only what is local to a site.
    let mut lp = unsafe { Lv::new(lp) };
    // The two index expressions. They are cleared and reset at the end
    // of every pass, so an early `return` below never leaks one.
    let mut var1 = UNSET_TV;
    let mut var2 = UNSET_TV;
    let mut empty1 = false;
    let mut rc = FAIL;

    'done: {
        loop {
            let c = unsafe { *p };
            let subscript = c == b'[' as c_char
                || (c == b'.' as c_char
                    && unsafe { *p.add(1) } != b'=' as c_char
                    && unsafe { *p.add(1) } != b'.' as c_char);
            if !subscript {
                break;
            }
            let mut container = unsafe { Tv::new(lp.ll_tv) };
            if c == b'.' as c_char && container.v_type != VAR_DICT {
                if !quiet {
                    let fmt = e_dot_can_only_be_used_on_dictionary_str.as_ptr();
                    // SAFETY: a shared message, whose format takes one
                    // NUL-terminated string.
                    unsafe { semsg_c!(gettext(fmt), name) };
                }
                return null_mut();
            }
            if container.v_type != VAR_LIST
                && container.v_type != VAR_DICT
                && container.v_type != VAR_BLOB
            {
                if !quiet {
                    emsg_static(c"E689: Can only index a List, Dictionary or Blob");
                }
                return null_mut();
            }

            // A null List or Blob works like an empty one; allocate now.
            if container.v_type == VAR_LIST && unsafe { container.vval.v_list }.is_null() {
                unsafe { tv_list_alloc_ret(lp.ll_tv, kListLenUnknown as ptrdiff_t) };
            } else if container.v_type == VAR_BLOB && unsafe { container.vval.v_blob }.is_null() {
                unsafe { tv_blob_alloc_ret(lp.ll_tv) };
            }

            if lp.ll_range {
                if !quiet {
                    emsg_static(c"E708: [:] must come last");
                }
                break 'done;
            }

            let mut len: c_int = -1;
            let mut key: *mut c_char = null_mut();
            if c == b'.' as c_char {
                key = unsafe { p.add(1) };
                len = 0;
                loop {
                    let b = unsafe { *key.offset(len as isize) } as u8;
                    if !(b.is_ascii_alphabetic() || ascii_isdigit(b.into()) || b == b'_') {
                        break;
                    }
                    len += 1;
                }
                if len == 0 {
                    if !quiet {
                        emsg_static(c"E713: Cannot use empty key after .");
                    }
                    return null_mut();
                }
                p = unsafe { key.offset(len as isize) };
            } else {
                // The index `[expr]`, or the first of `[expr : expr]`.
                p = unsafe { skipwhite(p.add(1)) };
                if unsafe { *p } == b':' as c_char {
                    empty1 = true;
                } else {
                    empty1 = false;
                    if unsafe { eval1(&raw mut p, &raw mut var1, &raw mut evalarg) } == FAIL {
                        break 'done;
                    }
                    if !unsafe { tv_check_str(&raw mut var1) } {
                        break 'done;
                    }
                    p = unsafe { skipwhite(p) };
                }

                if unsafe { *p } == b':' as c_char {
                    if container.v_type == VAR_DICT {
                        if !quiet {
                            emsg_static(e_cannot_slice_dictionary);
                        }
                        break 'done;
                    }
                    // The value being assigned has to be sliceable too.
                    // A null `rettv` is `:unlet`, which assigns nothing.
                    // SAFETY: `rettv` is non-null here; `v_type` names the member read.
                    let sliceable = rettv.is_null()
                        || (unsafe { (*rettv).v_type } == VAR_LIST
                            && !unsafe { (*rettv).vval.v_list }.is_null())
                        || (unsafe { (*rettv).v_type } == VAR_BLOB
                            && !unsafe { (*rettv).vval.v_blob }.is_null());
                    if !sliceable {
                        if !quiet {
                            emsg_static(c"E709: [:] requires a List or Blob value");
                        }
                        break 'done;
                    }
                    p = unsafe { skipwhite(p.add(1)) };
                    if unsafe { *p } == b']' as c_char {
                        lp.ll_empty2 = true;
                    } else {
                        lp.ll_empty2 = false;
                        let ev = unsafe { eval1(&raw mut p, &raw mut var2, &raw mut evalarg) };
                        if ev == FAIL {
                            break 'done;
                        }
                        if !unsafe { tv_check_str(&raw mut var2) } {
                            break 'done;
                        }
                    }
                    lp.ll_range = true;
                } else {
                    lp.ll_range = false;
                }

                if unsafe { *p } != b']' as c_char {
                    if !quiet {
                        emsg_static(e_missbrac);
                    }
                    break 'done;
                }
                p = unsafe { p.add(1) };
            }

            container = unsafe { Tv::new(lp.ll_tv) };
            if container.v_type == VAR_DICT {
                let (rec, end, idx) = (lp.raw(), &raw mut p, &raw mut var1);
                let status = unsafe {
                    get_lval_dict_item(rec, name, key, len, end, idx, flags, unlet, rettv)
                };
                match status {
                    GLV_FAIL => break 'done,
                    // The key is new: `ll_newkey` holds it and there is
                    // nothing left to descend into.
                    GLV_STOP => break,
                    _ => {}
                }
            } else if container.v_type == VAR_BLOB {
                let (a, b) = (&raw mut var1, &raw mut var2);
                if unsafe { get_lval_blob(lp.raw(), a, b, empty1, quiet) } == FAIL {
                    break 'done;
                }
                // A Blob byte is never a container, so this is the end.
                break;
            } else {
                let (a, b) = (&raw mut var1, &raw mut var2);
                if unsafe { get_lval_list(lp.raw(), a, b, empty1, flags, quiet) } == FAIL {
                    break 'done;
                }
            }

            clear_local(&mut var1);
            clear_local(&mut var2);
            var1.v_type = VAR_UNKNOWN;
            var2.v_type = VAR_UNKNOWN;
        }
        rc = OK;
    }

    clear_local(&mut var1);
    clear_local(&mut var2);
    if rc == OK { p } else { null_mut() }
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
    let quiet = flags & GLV_QUIET as c_int != 0;
    // SAFETY: the caller's promise; every field is written before it is read.
    let mut lp = unsafe { Lv::new(lp) };
    // SAFETY: as above -- the whole record is the caller's.
    unsafe { memset(lp.raw() as *mut c_void, 0, size_of::<lval_T>()) };

    if skip {
        // Only the name matters; nothing is resolved.
        lp.ll_name = name;
        let fne = FNE_INCL_BR | fne_flags;
        // SAFETY: `name` is NUL-terminated and the walk wants no braces.
        return unsafe { find_name_end(name, null_mut(), null_mut(), fne) } as *mut c_char;
    }

    // `find_name_end` writes `*const` and `make_expanded_name` wants
    // `*mut`; the two spell the same bytes of `name`, which is writable.
    let mut expr_start: *mut c_char = null_mut();
    let mut expr_end: *mut c_char = null_mut();
    let (starts, ends) = (
        (&raw mut expr_start).cast::<*const c_char>(),
        (&raw mut expr_end).cast::<*const c_char>(),
    );
    // SAFETY: `name` is NUL-terminated and the two out-parameters are this frame's.
    let mut p = unsafe { find_name_end(name, starts, ends, fne_flags) } as *mut c_char;

    if !expr_start.is_null() {
        // A curly-braces name: expand it.
        // SAFETY: `p` is a cursor into the NUL-terminated `name`.
        let after = unsafe { *p };
        if unlet
            && !ascii_iswhite(after as c_int)
            && ends_excmd(after as c_int) == 0
            && after != b'[' as c_char
            && after != b'.' as c_char
        {
            // SAFETY: the format takes one NUL-terminated string.
            unsafe { semsg_c!(gettext(e_trailing_arg.as_ptr()), p) };
            return null_mut();
        }
        // SAFETY: all four cursors are into the one writable string.
        lp.ll_exp_name = unsafe { make_expanded_name(name, expr_start, expr_end, p) };
        lp.ll_name = lp.ll_exp_name;
        if lp.ll_exp_name.is_null() {
            if !aborting() && !quiet {
                emsg_severe.set(true);
                // SAFETY: the format takes one NUL-terminated string.
                unsafe { semsg_c!(gettext(e_invarg2.as_ptr()), name) };
                return null_mut();
            }
            lp.ll_name_len = 0 as size_t;
        } else {
            // SAFETY: the expansion is NUL-terminated.
            lp.ll_name_len = unsafe { strlen(lp.ll_name) };
        }
    } else {
        lp.ll_name = name;
        // SAFETY: `p` and the name are cursors into the one string.
        lp.ll_name_len = unsafe { p.offset_from(lp.ll_name) } as size_t;
    }

    // Nothing is subscripted: the name is the whole left-hand side.
    // SAFETY: `p` is a cursor into the NUL-terminated `name`.
    let after = unsafe { *p };
    if (after != b'[' as c_char && after != b'.' as c_char) || lp.ll_name.is_null() {
        return p;
    }

    let mut ht: *mut hashtab_T = null_mut();
    let htp = if flags & GLV_READ_ONLY as c_int != 0 {
        null_mut()
    } else {
        &raw mut ht
    };
    let no_autoload = flags & GLV_NO_AUTOLOAD as c_int != 0;
    // SAFETY: the name is NUL-terminated and `ht` is this frame's.
    let v = unsafe { find_var(lp.ll_name, lp.ll_name_len, htp, no_autoload) };
    if v.is_null() {
        if !quiet {
            let (n, s) = (lp.ll_name_len as c_int, lp.ll_name);
            // SAFETY: the format takes a length and the string it bounds.
            let fmt = unsafe { gettext(c"E121: Undefined variable: %.*s".as_ptr()) };
            // SAFETY: as above.
            unsafe { semsg_c!(fmt, n, s) };
        }
        return null_mut();
    }

    // SAFETY: `v` is the live dictionary item the name resolved to.
    lp.ll_tv = unsafe { &raw mut (*v).di_tv };
    // SAFETY: `ll_tv` is that item's typval.
    if unsafe { tv_is_luafunc(lp.ll_tv) } {
        return p;
    }

    // SAFETY: `lp` has `ll_tv` set, `p` points into `name`, and `ht` and `v` are this frame's.
    p = unsafe { get_lval_subscript(lp.raw(), p, name, rettv, ht, v, unlet, flags) };
    if p.is_null() {
        return null_mut();
    }
    // SAFETY: `p` and the name are cursors into the one string.
    lp.ll_name_len = unsafe { p.offset_from(lp.ll_name) } as size_t;
    p
}

/// Release what `get_lval` allocated into `lp`.
///
/// # Safety
/// `lp` must be valid.
pub unsafe fn clear_lval(lp: *mut lval_T) {
    // SAFETY: the caller's promise; both strings are `get_lval`'s own.
    let lp = unsafe { Lv::new(lp) };
    // SAFETY: as above -- both are owned, and null is fine for `xfree`.
    unsafe { xfree(lp.ll_exp_name as *mut c_void) };
    // SAFETY: as above.
    unsafe { xfree(lp.ll_newkey as *mut c_void) };
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
    // SAFETY, for every region in this body and in the two helpers below:
    // the caller's promise is that `lp` is the record `get_lval` filled in
    // and outlives the call, that `rettv` is the value being assigned, and
    // that `endp` points into the same writable NUL-terminated string. Each
    // union member read is the one the `v_type` just tested names; a
    // non-null `op` is NUL-terminated; `oldtv` and `tv` are frame locals;
    // and every message named is a literal or a shared `e_*` text. The
    // notes below add only what is local to a site.
    let (mut lp, value) = unsafe { (Lv::new(lp), Tv::new(rettv)) };
    if lp.ll_tv.is_null() {
        // SAFETY: as above; `endp` points into the same writable string.
        unsafe { set_whole_var(lp.raw(), endp, rettv, copy, is_const, op) };
        return;
    }

    // A locked container refuses the write; the lock to test is the
    // Dict's own when a key is being added to it.
    // SAFETY: a pending new key means `ll_tv` holds the Dict it goes into.
    let target = unsafe { Tv::new(lp.ll_tv) };
    let lock = if lp.ll_newkey.is_null() {
        target.v_lock
    } else {
        // SAFETY: as above -- the Dict the key is being added to.
        unsafe { (*target.vval.v_dict).dv_lock }
    };
    if unsafe { value_check_lock(lock, lp.ll_name, TV_CSTRING as size_t) } {
        return;
    }

    if lp.ll_range {
        if is_const {
            emsg_static(c"E996: Cannot lock a range");
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
        if value.v_type != VAR_LIST {
            emsg_static(e_listreq);
            return;
        }
        // SAFETY: `VAR_LIST` says `v_list` is live; `ll_list` is the target.
        let src = unsafe { value.vval.v_list };
        let (list, n1, n2) = (lp.ll_list, lp.ll_n1, lp.ll_n2);
        let (empty2, name) = (lp.ll_empty2, lp.ll_name);
        // SAFETY: as above.
        unsafe { tv_list_assign_range(list, src, n1, n2, empty2, op, name) };
        return;
    }

    // The value the watchers are told the key used to have. It stays
    // unset for a key that did not exist, and that is how the
    // notification below tells the two cases apart — see the module
    // docs. It must never be the same typval as the new value.
    let mut oldtv = UNSET_TV;
    let dict = lp.ll_dict;
    let watched = unsafe { tv_dict_is_watched(dict) };

    if is_const {
        emsg_static(c"E996: Cannot lock a list or dict");
        return;
    }

    // Writing an *existing* key of the `v:` scope dictionary is a write
    // to a `v:` variable, and has to pass the same type enforcement the
    // unsubscripted spelling does. Upstream stores straight into the
    // item, which permanently re-types the variable and, for
    // `v:oldfiles`, crashes the next reader (docket O-B14-10). A new key
    // cannot happen here: `get_lval` refuses to add one to `v:`.
    if dict == get_vimvar_dict() && lp.ll_newkey.is_null() {
        // SAFETY: `ll_di` is the existing item, and `rettv` the caller's.
        unsafe { set_vvar_item(lp.ll_di, rettv, copy, op) };
        return;
    }

    'notify: {
        if !lp.ll_newkey.is_null() {
            // The key has to be added to the Dictionary first.
            if !op.is_null() && unsafe { *op } != b'=' as c_char {
                unsafe { semsg_c!(gettext(e_dictkey.as_ptr()), lp.ll_newkey) };
                return;
            }
            // SAFETY: `ll_tv` holds the Dict; `ll_newkey` is the owned key text.
            let target = unsafe { Tv::new(lp.ll_tv).vval.v_dict };
            if unsafe { tv_dict_wrong_func_name(target, rettv, lp.ll_newkey) } != 0 {
                return;
            }
            let di = unsafe { tv_dict_item_alloc(lp.ll_newkey) };
            if unsafe { tv_dict_add(target, di) } == FAIL {
                unsafe { xfree(di as *mut c_void) };
                return;
            }
            // SAFETY: `di` belongs to the Dict; its typval is the target.
            lp.ll_tv = unsafe { &raw mut (*di).di_tv };
        } else {
            if watched {
                // SAFETY: `oldtv` is this frame's separate record of the old value.
                unsafe { tv_copy(lp.ll_tv, &raw mut oldtv) };
            }
            if !op.is_null() && unsafe { *op } != b'=' as c_char {
                // `+=` and friends modify in place; there is nothing to
                // assign afterwards.
                // SAFETY: `ll_tv` is the live target and `rettv` the caller's value.
                unsafe { eexe_mod_op(lp.ll_tv, rettv, op) };
                break 'notify;
            }
            unsafe { tv_clear(lp.ll_tv) };
        }

        if copy {
            unsafe { tv_copy(rettv, lp.ll_tv) };
        } else {
            // SAFETY: the value moves out of `rettv`, which is reset after it.
            let mut target = unsafe { Tv::new(lp.ll_tv) };
            // SAFETY: as above.
            *target = unsafe { *rettv };
            target.v_lock = VarLock::Unlocked;
            // SAFETY: `rettv` is reset so nothing frees the value twice.
            unsafe { tv_init(rettv) };
        }
    }

    if !watched {
        return;
    }
    if oldtv.v_type == VAR_UNKNOWN {
        // Nothing was saved, so this is the new-key case.
        debug_assert!(!lp.ll_newkey.is_null());
        // SAFETY: the watched Dict, its new key, and the value just written.
        unsafe { tv_dict_watcher_notify(dict, lp.ll_newkey, lp.ll_tv, null_mut()) };
    } else {
        let di = lp.ll_di;
        // SAFETY: the key is inline, so naming its address reads nothing.
        let key = unsafe { &raw mut (*di).di_key } as *mut c_char;
        debug_assert!(!key.is_null());
        // SAFETY: the watched Dict, its key, the new value and the old copy.
        unsafe { tv_dict_watcher_notify(dict, key, lp.ll_tv, &raw mut oldtv) };
        clear_local(&mut oldtv);
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
    let lp = unsafe { Lv::new(lp) };
    // Terminate the left-hand side in place: the messages below name the
    // variable and would otherwise print the rest of the command too.
    // SAFETY: the caller's promise -- `endp` points into the same writable NUL-terminated string.
    let cc = unsafe { *endp };
    // SAFETY: as above -- the byte is put back before returning.
    unsafe { *endp = NUL as c_char };

    if !lp.ll_blob.is_null() {
        // Upstream's three early returns here leave the left-hand side
        // terminated in place rather than putting `cc` back. Preserved:
        // anything that reads the command line after a rejected Blob
        // assignment sees the truncated form.
        // SAFETY: `lp` has `ll_blob` set, and `rettv` is the caller's.
        if !unsafe { set_blob_var(lp.raw(), rettv, op) } {
            return;
        }
    } else if !op.is_null() && unsafe { *op } != b'=' as c_char {
        // `+=`, `-=`, `*=`, `/=`, `%=` and `..=`.
        if is_const {
            emsg_static(e_cannot_mod);
            unsafe { *endp = cc };
            return;
        }
        let mut tv = UNSET_TV;
        let mut di: *mut dictitem_T = null_mut();
        let (name, name_len) = (lp.ll_name, lp.ll_name_len);
        // SAFETY: the name is the one `get_lval` resolved, and `tv` and `di` are this frame's.
        let (tvp, dip) = (&raw mut tv, &raw mut di);
        let found = unsafe { eval_variable(name, name_len as c_int, tvp, dip, true, false) };
        if found == OK {
            // SAFETY: a non-null `di` is live; `tv` is this frame's copy.
            let (n, dtv) = if di.is_null() {
                (0, null_mut())
            } else {
                // SAFETY: `di` is live, so naming its typval reads nothing.
                (unsafe { (*di).di_flags } as c_int, unsafe {
                    &raw mut (*di).di_tv
                })
            };
            let writable = di.is_null()
                || (!unsafe { var_check_ro(n, name, TV_CSTRING as size_t) }
                    && !unsafe { tv_check_lock(dtv, name, TV_CSTRING as size_t) });
            if writable && unsafe { eexe_mod_op(&raw mut tv, rettv, op) } == OK {
                // SAFETY: as above -- the folded value goes back by name.
                unsafe { set_var(name, name_len, &raw mut tv, false) };
            }
            clear_local(&mut tv);
        }
    } else {
        let (name, name_len) = (lp.ll_name, lp.ll_name_len);
        // SAFETY: the name is the one `get_lval` resolved, and `rettv` is the caller's value.
        unsafe { set_var_const(name, name_len, rettv, copy, is_const) };
    }

    unsafe { *endp = cc };
}

/// Write a byte or a byte range into the Blob `lp` resolved. Answers
/// whether the caller should put the terminated left-hand side back — the
/// three refusal paths say no, which is upstream's.
///
/// # Safety
/// As `set_var_lval`, with `lp->ll_blob` set.
unsafe fn set_blob_var(lp: *mut lval_T, rettv: *mut typval_T, op: *const c_char) -> bool {
    // SAFETY: the caller's promise -- both outlive the call.
    let (mut lp, value) = unsafe { (Lv::new(lp), Tv::new(rettv)) };
    if !op.is_null() && unsafe { *op } != b'=' as c_char {
        unsafe { semsg_c!(gettext(e_letwrong.as_ptr()), op) };
        return false;
    }
    // SAFETY: the caller's promise: `ll_blob` is live, the name resolved.
    let lock = unsafe { (*lp.ll_blob).bv_lock };
    // SAFETY: as above.
    let locked = unsafe { value_check_lock(lock, lp.ll_name, TV_CSTRING as size_t) };
    if locked {
        return false;
    }

    if lp.ll_range && value.v_type == VAR_BLOB {
        if lp.ll_empty2 {
            lp.ll_n2 = unsafe { tv_blob_len(lp.ll_blob) } - 1;
        }
        let (blob, n1, n2) = (lp.ll_blob, lp.ll_n1 as varnumber_T, lp.ll_n2 as varnumber_T);
        // SAFETY: as above; `rettv` holds the Blob being assigned.
        if unsafe { tv_blob_set_range(blob, n1, n2, rettv) } == FAIL {
            return false;
        }
        return true;
    }

    let mut error = false;
    let val = unsafe { tv_get_number_chk(rettv, &raw mut error) };
    if !error {
        if !(0..=255).contains(&val) {
            unsafe { semsg_c!(gettext(e_invalid_value_for_blob_nr.as_ptr()), val) };
        } else {
            // SAFETY: `ll_blob` is the live Blob and `ll_n1` a byte of it.
            unsafe { tv_blob_set_append(lp.ll_blob, lp.ll_n1, val as uint8_t) };
        }
    }
    true
}
