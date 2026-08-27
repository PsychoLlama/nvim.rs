//! Resolving a name to the `dictitem_T` that holds it.
//!
//! [`find_var_ht_dict`] picks the scope from the name's prefix,
//! [`find_var_in_ht`] finds the entry in it (and is where a bare
//! `g:`/`b:`/`l:` becomes the scope's own dictionary item), and
//! [`eval_variable`] is the whole path an expression takes.
//! [`get_user_var_name`] walks the same scopes for completion.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::eval::typval::NumBuf;
use crate::types::{FAIL, NUL, OK};

/// The buffer [`cat_prefix_varname`] hands its answer back in, and its size.
///
/// One buffer for the whole completion walk: every name it produces is read
/// and copied before the next call, which is what lets it be reused.
static varnamebuf: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());
static varnamebuflen: GlobalCell<size_t> = GlobalCell::new(0);

/// `"<prefix>:<name>"`, in a buffer that lives until the next call.
///
/// # Safety
/// `name` is a NUL-terminated string.
pub unsafe fn cat_prefix_varname(prefix: c_int, name: *const c_char) -> *mut c_char {
    let mut len = unsafe { strlen(name) } + 3;
    if len > varnamebuflen.get() {
        unsafe { xfree(varnamebuf.get().cast()) };
        len += 10;
        varnamebuf.set(unsafe { xmalloc(len) } as *mut c_char);
        varnamebuflen.set(len);
    }
    let buf = varnamebuf.get();
    unsafe { *buf = prefix as c_char };
    unsafe { *buf.add(1) = b':' as c_char };
    unsafe { strcpy(buf.add(2), name) };
    buf
}

/// The `idx`-th variable name for command-line completion, or NULL when
/// there are no more.
///
/// This is a generator, not a function: `idx == 0` restarts the walk and
/// every later call resumes it, so the cursor into each scope lives in a
/// `static`.  The five scopes are visited in turn -- `g:`, `b:`, `w:`, `t:`,
/// then the whole `v:` table -- and only `g:` answers a bare name, because
/// that is the scope an unprefixed one completes in.
///
/// # Safety
/// `xp` is a live expansion context.
pub unsafe fn get_user_var_name(xp: *mut expand_T, idx: c_int) -> *mut c_char {
    static gdone: GlobalCell<size_t> = GlobalCell::new(0);
    static bdone: GlobalCell<size_t> = GlobalCell::new(0);
    static wdone: GlobalCell<size_t> = GlobalCell::new(0);
    static tdone: GlobalCell<size_t> = GlobalCell::new(0);
    static vidx: GlobalCell<size_t> = GlobalCell::new(0);
    /// The hashtab cursor, shared by the four hashtab scopes: only one
    /// of them is being walked at a time.
    static hi: GlobalCell<*mut hashitem_T> = GlobalCell::new(ptr::null_mut());

    if idx == 0 {
        gdone.set(0);
        bdone.set(0);
        wdone.set(0);
        tdone.set(0);
        vidx.set(0);
    }

    // One step through `ht`: the first call starts at the array, every
    // later one advances past the slot the previous call answered and
    // then skips the empty and removed ones.
    let step = |done: &GlobalCell<size_t>, ht: *const hashtab_T| -> Option<*mut c_char> {
        let n = done.get();
        if n >= unsafe { (*ht).ht_used } {
            return None;
        }
        done.set(n + 1);
        hi.set(if n == 0 {
            unsafe { (*ht).ht_array }
        } else {
            unsafe { hi.get().add(1) }
        });
        while !unsafe { (*hi.get()).is_kept() } {
            hi.set(unsafe { hi.get().add(1) });
        }
        Some(unsafe { (*hi.get()).hi_key })
    };

    if let Some(key) = step(&gdone, get_globvar_ht()) {
        if unsafe { strncmp(c"g:".as_ptr(), (*xp).xp_pattern, 2) } == 0 {
            return unsafe { cat_prefix_varname(b'g' as c_int, key) };
        }
        return key;
    }
    // The window this completes for is the one the command line was
    // opened over, which is `prevwin` while the command-line window is
    // current.
    let win = unsafe { prevwin_curwin() };
    if let Some(key) = step(&bdone, unsafe {
        &raw const (*(*(*win).w_buffer).b_vars).dv_hashtab
    }) {
        return unsafe { cat_prefix_varname(b'b' as c_int, key) };
    }
    if let Some(key) = step(&wdone, unsafe { &raw const (*(*win).w_vars).dv_hashtab }) {
        return unsafe { cat_prefix_varname(b'w' as c_int, key) };
    }
    if let Some(key) = step(&tdone, unsafe {
        &raw const (*(*curtab.get()).tp_vars).dv_hashtab
    }) {
        return unsafe { cat_prefix_varname(b't' as c_int, key) };
    }
    let v = vidx.get();
    if let Ok(vv) = Vv::try_from(v) {
        vidx.set(v + 1);
        return unsafe { cat_prefix_varname(b'v' as c_int, get_vim_var_name(vv)) };
    }

    unsafe { xfree(varnamebuf.get().cast()) };
    varnamebuf.set(ptr::null_mut());
    varnamebuflen.set(0);
    ptr::null_mut()
}

/// Read the variable `name[0..len]` into `rettv`, reporting E121 if it does
/// not exist.
///
/// `rettv` may be NULL to ask only whether the variable exists, and `dip`
/// takes the item it was found in.  `verbose` allows the error; the message
/// is suppressed for a lookup that is allowed to fail.
///
/// # Safety
/// `name` points at `len` readable bytes; `rettv`/`dip` are writable or
/// NULL.
pub unsafe fn eval_variable(
    name: *const c_char,
    len: c_int,
    rettv: *mut typval_T,
    dip: *mut *mut dictitem_T,
    verbose: bool,
    no_autoload: bool,
) -> c_int {
    let v = unsafe { find_var(name, len as size_t, ptr::null_mut(), no_autoload) };
    if v.is_null() {
        if !rettv.is_null() && verbose {
            semsg_c!(
                unsafe { gettext(c"E121: Undefined variable: %.*s".as_ptr()) },
                len,
                name,
            );
        }
        return FAIL;
    }
    if !dip.is_null() {
        unsafe { *dip = v };
    }
    if !rettv.is_null() {
        unsafe { tv_copy(&raw mut (*v).di_tv, rettv) };
    }
    OK
}

/// Note in `eval_lavars_used` that `name[0..len]` is a function-local
/// variable or an argument, which is what makes a lambda capture it.
///
/// # Safety
/// `name` points at `len` readable bytes.
pub unsafe fn check_vars(name: *const c_char, len: size_t) {
    if eval_lavars_used.get().is_null() {
        return;
    }
    let mut varname: *const c_char = ptr::null();
    let ht = unsafe { find_var_ht(name, len, &raw mut varname) };
    if (ht == unsafe { get_funccal_local_ht() } || ht == unsafe { get_funccal_args_ht() })
        && !unsafe { find_var(name, len, ptr::null_mut(), true) }.is_null()
    {
        unsafe { *eval_lavars_used.get() = true };
    }
}

/// The item holding the variable `name[0..name_len]`, or NULL.
///
/// A non-NULL `htp` means the caller is about to *write*, and takes the
/// scope's hashtab; it also suppresses autoloading, since a write does not
/// want the script sourced.
///
/// # Safety
/// `name` points at `name_len` readable bytes; `htp` is writable or NULL.
pub unsafe fn find_var(
    name: *const c_char,
    name_len: size_t,
    htp: *mut *mut hashtab_T,
    no_autoload: bool,
) -> *mut dictitem_T {
    let mut varname: *const c_char = ptr::null();
    let ht = unsafe { find_var_ht(name, name_len, &raw mut varname) };
    if !htp.is_null() {
        unsafe { *htp = ht };
    }
    if ht.is_null() {
        return ptr::null_mut();
    }
    let no_autoload = no_autoload || !htp.is_null();
    let ret = unsafe {
        find_var_in_ht(
            ht,
            *name as c_int,
            varname,
            name_len - varname.offset_from(name) as size_t,
            no_autoload,
        )
    };
    if !ret.is_null() {
        return ret;
    }
    // Search the parent scope, which a lambda can reference.
    unsafe { find_var_in_scoped_ht(name, name_len, no_autoload as c_int) }
}

/// The item holding `varname[0..varname_len]` in `ht`, or NULL.
///
/// An empty name is the scope itself (`let g:` and friends), and answers the
/// scope's own dictionary item; `htname` -- the name's first character -- is
/// what says which scope that is.  Otherwise the name is looked up, and for
/// `g:` a miss may source an autoload script and look again.
///
/// # Safety
/// `ht` is a live hashtab and `varname` points at `varname_len` readable
/// bytes.
pub unsafe fn find_var_in_ht(
    ht: *mut hashtab_T,
    htname: c_int,
    varname: *const c_char,
    varname_len: size_t,
    no_autoload: bool,
) -> *mut dictitem_T {
    if varname_len == 0 {
        // Something like "s:", or `ht` would have been NULL.
        return match htname as u8 {
            b's' => (unsafe { &raw mut (*script_sv(current_sctx.get().sc_sid)).sv_var }).cast(),
            b'g' => globvar_scope_item().cast(),
            b'v' => vimvar_scope_item().cast(),
            b'b' => (unsafe { &raw mut (*curbuf.get()).b_bufvar }).cast(),
            b'w' => (unsafe { &raw mut (*curwin.get()).w_winvar }).cast(),
            b't' => (unsafe { &raw mut (*curtab.get()).tp_winvar }).cast(),
            b'l' => unsafe { get_funccal_local_var() },
            b'a' => unsafe { get_funccal_args_var() },
            _ => ptr::null_mut(),
        };
    }

    let mut hi = unsafe { hash_find_len(ht, varname, varname_len) };
    if !unsafe { (*hi).is_kept() } {
        // A global may be an autoload variable; sourcing its script may
        // define it.  Don't source one that ran already, or every check
        // of "is this name a Funcref variable" would re-run it.
        if ht == get_globvar_ht() && !no_autoload {
            // script_autoload() may invalidate `hi`, so it has to be
            // asked for again rather than reused.
            if !unsafe { script_autoload(varname, varname_len, false) } || aborting() {
                return ptr::null_mut();
            }
            hi = unsafe { hash_find_len(ht, varname, varname_len) };
        }
        if !unsafe { (*hi).is_kept() } {
            return ptr::null_mut();
        }
    }
    unsafe { tv_dict_hi2di(hi) }
}

/// The scope dictionary and hashtab `name[0..name_len]` belongs to, or NULL
/// when the name names no scope.
///
/// A name with no prefix is `v:version` if the compatibility table has it,
/// otherwise the function-local scope if there is one and `g:` if not.  A
/// prefixed one names its scope directly -- and `s:` is where an anonymous
/// Lua or `:execute` chunk is given a script id, so that it can have script
/// variables at all (#15994).
///
/// # Safety
/// `name` points at `name_len` readable bytes; `varname` and `d` are
/// writable.
pub(crate) unsafe fn find_var_ht_dict(
    name: *const c_char,
    name_len: size_t,
    varname: *mut *const c_char,
    d: *mut *mut dict_T,
) -> *mut hashtab_T {
    unsafe { *d = ptr::null_mut() };
    if name_len == 0 {
        return ptr::null_mut();
    }

    if name_len == 1 || unsafe { *name.add(1) } != b':' as c_char {
        // An implicit scope. The name must not start with a colon or a
        // '#'.
        if unsafe { *name } == b':' as c_char || unsafe { *name } == AUTOLOAD_CHAR {
            return ptr::null_mut();
        }
        unsafe { *varname = name };

        // "version" is "v:version" in every scope.
        if unsafe { (*hash_find_len(get_compat_ht(), name, name_len)).is_kept() } {
            return get_compat_ht();
        }

        unsafe { *d = unsafe { get_funccal_local_dict() } };
        if unsafe { (*d).is_null() } {
            unsafe { *d = get_globvar_dict() };
        }
    } else {
        unsafe { *varname = unsafe { name.add(2) } };
        if unsafe { *name } == b'g' as c_char {
            unsafe { *d = get_globvar_dict() };
        } else if name_len > 2
            && (!unsafe { memchr(name.add(2).cast(), b':' as c_int, name_len - 2) }.is_null()
                || !unsafe { memchr(name.add(2).cast(), AUTOLOAD_CHAR as c_int, name_len - 2) }
                    .is_null())
        {
            // Without `g:` there must be no ':' or '#' in the rest.
            return ptr::null_mut();
        }

        match unsafe { *name } as u8 {
            b'b' => unsafe { *d = cur_buf().b_vars },
            b'w' => unsafe { *d = cur_win().w_vars },
            b't' => unsafe { *d = unsafe { (*curtab.get()).tp_vars } },
            b'v' => unsafe { *d = get_vimvar_dict() },
            b'a' => unsafe { *d = unsafe { get_funccal_args_dict() } },
            b'l' => unsafe { *d = unsafe { get_funccal_local_dict() } },
            b's' => {
                // Both calls below fill `sctx` in, and neither reads the
                // cell, so the round trip through a local is what the C's
                // write-through-the-pointer amounts to.
                let mut sctx = current_sctx.get();
                if (sctx.sc_sid > 0 || sctx.sc_sid == SID_STR || sctx.sc_sid == SID_LUA)
                    && sctx.sc_sid <= script_count()
                {
                    // Resolve the Lua filename and line number, so that
                    // a later "Last set from" can name them.
                    unsafe { nlua_set_sctx(&raw mut sctx) };
                    if sctx.sc_sid == SID_STR || sctx.sc_sid == SID_LUA {
                        // An anonymous chunk has no script item yet.
                        unsafe { new_script_item(ptr::null_mut(), &raw mut sctx.sc_sid) };
                    }
                    current_sctx.set(sctx);
                    unsafe { *d = unsafe { &raw mut (*script_sv(sctx.sc_sid)).sv_dict } };
                }
            }
            _ => {}
        }
    }

    unsafe { (*d).as_mut() }.map_or(ptr::null_mut(), |d| &raw mut d.dv_hashtab)
}

/// [`find_var_ht_dict`] without the dictionary.
///
/// # Safety
/// As [`find_var_ht_dict`].
pub unsafe fn find_var_ht(
    name: *const c_char,
    name_len: size_t,
    varname: *mut *const c_char,
) -> *mut hashtab_T {
    let mut d: *mut dict_T = ptr::null_mut();
    unsafe { find_var_ht_dict(name, name_len, varname, &raw mut d) }
}

/// The string value of the variable `name`, or NULL when it does not exist.
///
/// A variable holding a Number has no string of its own, so the caller lends
/// `numbuf` for it to be rendered into; the answer borrows either that or the
/// variable, and lives no longer than the shorter of the two.
///
/// # Safety
/// `name` is a NUL-terminated string.
pub unsafe fn get_var_value(name: *const c_char, numbuf: &mut NumBuf) -> *mut c_char {
    let v = unsafe { find_var(name, strlen(name), ptr::null_mut(), false) };
    if v.is_null() {
        return ptr::null_mut();
    }
    unsafe { numbuf.string(&raw mut (*v).di_tv) as *mut c_char }
}

/// `exists()` over a variable name: whether `var` names something, including
/// everything a subscript on it reaches.
///
/// # Safety
/// `var` is a NUL-terminated string.
pub unsafe fn var_exists(mut var: *const c_char) -> bool {
    let mut evalarg = EVALARG_EVALUATE;
    let mut tofree: *mut c_char = ptr::null_mut();
    let mut n = false;
    let mut name = var;
    // Get the variable name, expanding a `{curly}` name into `tofree`.
    let len = unsafe { get_name_len(&raw mut var, &raw mut tofree, true, false) };
    if len > 0 {
        let mut tv = TV_INITIAL_VALUE;
        if !tofree.is_null() {
            name = tofree;
        }
        n = unsafe { eval_variable(name, len, &raw mut tv, ptr::null_mut(), false, true) } == OK;
        if n {
            // Handle `d.key`, `l[idx]` and `Func()`.
            n = unsafe { handle_subscript(&raw mut var, &raw mut tv, &raw mut evalarg, false) }
                == OK;
            if n {
                unsafe { tv_clear(&raw mut tv) };
            }
        }
    }
    if unsafe { *var } != NUL as c_char {
        n = false;
    }
    unsafe { xfree(tofree.cast()) };
    n
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
