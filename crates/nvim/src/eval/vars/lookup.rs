//! Resolving a name to the `DictItem` that holds it.
//!
//! [`find_var_ht_dict`] picks the scope from the name's prefix,
//! [`find_var_in_ht`] finds the entry in it (and is where a bare
//! `g:`/`b:`/`l:` becomes the scope's own dictionary item), and
//! [`eval_variable`] is the whole path an expression takes.
//! [`get_user_var_name`] walks the same scopes for completion.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use crate::message_fmt::c_str_len;
use crate::semsg;
use crate::winlayer::TabPage;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};
use core::mem::offset_of;
use core::ptr;

use super::*;
use crate::eval::typval::NumBuf;
use crate::types::{Failed, NUL};

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
    // SAFETY: the caller's obligation -- a NUL-terminated name -- and the
    // buffer below is grown to hold the prefix, the name and its NUL.
    let mut len = unsafe { cstr::bytes_at(name) }.len() + 3;
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
/// `expand` is a live expansion context.
pub unsafe fn get_user_var_name(expand: *mut Expand, idx: c_int) -> *mut c_char {
    static gdone: GlobalCell<size_t> = GlobalCell::new(0);
    static bdone: GlobalCell<size_t> = GlobalCell::new(0);
    static wdone: GlobalCell<size_t> = GlobalCell::new(0);
    static tdone: GlobalCell<size_t> = GlobalCell::new(0);
    static vidx: GlobalCell<size_t> = GlobalCell::new(0);
    /// The hashtab cursor, shared by the four hashtab scopes: only one of
    /// them is being walked at a time.
    ///
    /// A slot *index*, parked across calls into the editor: the table's
    /// small run lives inside the table, so a pointer cursor would not
    /// survive the next mutation of it.
    static slot: GlobalCell<usize> = GlobalCell::new(0);

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
    let step = |done: &GlobalCell<size_t>, ht: *const HashTab| -> Option<*mut c_char> {
        let n = done.get();
        if n >= unsafe { (*ht).ht_used } {
            return None;
        }
        done.set(n + 1);
        slot.set(if n == 0 { 0 } else { slot.get() + 1 });
        // SAFETY: the caller's table, which holds `ht_used > n` live items,
        // so a kept slot is still ahead of the cursor.
        while !unsafe { (*ht).slot(slot.get()) }.is_kept() {
            slot.set(slot.get() + 1);
        }
        Some(unsafe { (*ht).slot(slot.get()) }.hi_key)
    };

    if let Some(key) = step(&gdone, get_globvar_ht()) {
        if unsafe { cstr::starts_with((*expand).xp_pattern, b"g:") } {
            return unsafe { cat_prefix_varname(b'g' as c_int, key) };
        }
        return key;
    }
    // The window this completes for is the one the command line was
    // opened over, which is `prevwin` while the command-line window is
    // current.
    let win = prevwin_curwin();
    // SAFETY: a live window's buffer and variable dictionaries are live.
    let bvars = unsafe { &raw const (*win.buffer().b_vars).dv_hashtab };
    if let Some(key) = step(&bdone, bvars) {
        return unsafe { cat_prefix_varname(b'b' as c_int, key) };
    }
    if let Some(key) = step(&wdone, unsafe { &raw const (*win.w_vars).dv_hashtab }) {
        return unsafe { cat_prefix_varname(b'w' as c_int, key) };
    }
    let tvars = unsafe { &raw const (*TabPage::current().tp_vars).dv_hashtab };
    if let Some(key) = step(&tdone, tvars) {
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

/// Read the variable `name[0..len]` into `result`, reporting E121 if it does
/// not exist.
///
/// `result` may be NULL to ask only whether the variable exists, and `dip`
/// takes the item it was found in.  `verbose` allows the error; the message
/// is suppressed for a lookup that is allowed to fail.
///
/// # Safety
/// `name` points at `len` readable bytes; `result`/`dip` are writable or
/// NULL.
pub unsafe fn eval_variable(
    name: *const c_char,
    len: c_int,
    result: Option<&mut TypVal>,
    dip: *mut *mut DictItem,
    verbose: bool,
    no_autoload: bool,
) -> Result<(), Failed> {
    // SAFETY: the caller's obligation -- `len` readable bytes and `dip`
    // writable or NULL.
    let v = unsafe { find_var(name, len as size_t, ptr::null_mut(), no_autoload) };
    if v.is_null() {
        if result.is_some() && verbose {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let name = unsafe { c_str_len(name, len as usize) };
            semsg!("E121: Undefined variable: {name}");
        }
        return Err(Failed);
    }
    if !dip.is_null() {
        unsafe { *dip = v };
    }
    if let Some(result) = result {
        let item = unsafe { Di::new(v) };
        let value = item.field_ptr::<TypVal>(offset_of!(DictItem, di_tv));
        unsafe { tv_copy(&*value, result) };
    }
    Ok(())
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
    if (ht == get_funccal_local_ht() || ht == get_funccal_args_ht())
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
    htp: *mut *mut HashTab,
    no_autoload: bool,
) -> *mut DictItem {
    let mut varname: *const c_char = ptr::null();
    let ht = unsafe { find_var_ht(name, name_len, &raw mut varname) };
    if !htp.is_null() {
        unsafe { *htp = ht };
    }
    if ht.is_null() {
        return ptr::null_mut();
    }
    let no_autoload = no_autoload || !htp.is_null();
    // SAFETY: `varname` points inside `name`, so the subtraction cannot go
    // negative; the scope's first character is what names it.
    let htname = c_int::from(unsafe { *name });
    let vlen = name_len - unsafe { varname.offset_from(name) } as size_t;
    let ret = unsafe { find_var_in_ht(ht, htname, varname, vlen, no_autoload) };
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
    ht: *mut HashTab,
    htname: c_int,
    varname: *const c_char,
    varname_len: size_t,
    no_autoload: bool,
) -> *mut DictItem {
    if varname_len == 0 {
        // Something like "s:", or `ht` would have been NULL.
        return match htname as u8 {
            b's' => (unsafe { &raw mut (*script_sv(current_sctx.get().sc_sid)).sv_var }).cast(),
            b'g' => globvar_scope_item().cast(),
            b'v' => vimvar_scope_item().cast(),
            b'b' => (unsafe { &raw mut (*Buf::current_raw()).b_bufvar }).cast(),
            b'w' => (unsafe { &raw mut (*Win::current_raw()).w_winvar }).cast(),
            b't' => (unsafe { &raw mut (*TabPage::current_raw()).tp_winvar }).cast(),
            b'l' => get_funccal_local_var(),
            b'a' => get_funccal_args_var(),
            _ => ptr::null_mut(),
        };
    }

    let mut hi = unsafe { hash_find_len(ht, varname, varname_len) };
    if !hi.is_kept() {
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
        if !hi.is_kept() {
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
    d: *mut *mut Dict,
) -> *mut HashTab {
    // SAFETY: the caller's obligation -- `name_len` readable bytes, and two
    // writable out-parameters that are the caller's own locals.
    let (mut dict, mut vname) = unsafe { (Live::new(d), Live::new(varname)) };
    *dict = ptr::null_mut();
    if name_len == 0 {
        return ptr::null_mut();
    }

    let lead = unsafe { *name };
    if name_len == 1 || unsafe { *name.add(1) } != b':' as c_char {
        // An implicit scope. The name must not start with a colon or a
        // '#'.
        if lead == b':' as c_char || lead == AUTOLOAD_CHAR {
            return ptr::null_mut();
        }
        *vname = name;

        // "version" is "v:version" in every scope.
        if unsafe { hash_find_len(get_compat_ht(), name, name_len) }.is_kept() {
            return get_compat_ht();
        }

        *dict = get_funccal_local_dict();
        if dict.is_null() {
            *dict = get_globvar_dict();
        }
    } else {
        *vname = unsafe { name.add(2) };
        if lead == b'g' as c_char {
            *dict = get_globvar_dict();
        } else if name_len > 2
            && (!unsafe { memchr(name.add(2).cast(), b':' as c_int, name_len - 2) }.is_null()
                || !unsafe { memchr(name.add(2).cast(), AUTOLOAD_CHAR as c_int, name_len - 2) }
                    .is_null())
        {
            // Without `g:` there must be no ':' or '#' in the rest.
            return ptr::null_mut();
        }

        match lead as u8 {
            b'b' => *dict = Buf::current().b_vars,
            b'w' => *dict = Win::current().w_vars,
            b't' => *dict = TabPage::current().tp_vars,
            b'v' => *dict = get_vimvar_dict(),
            b'a' => *dict = get_funccal_args_dict(),
            b'l' => *dict = get_funccal_local_dict(),
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
                    *dict = unsafe { &raw mut (*script_sv(sctx.sc_sid)).sv_dict };
                }
            }
            _ => {}
        }
    }

    // SAFETY: the dictionary just chosen is live or NULL, and its hashtab
    // is a field of it.
    unsafe { (*dict).as_mut() }.map_or(ptr::null_mut(), |d| &raw mut d.dv_hashtab)
}

/// [`find_var_ht_dict`] without the dictionary.
///
/// # Safety
/// As [`find_var_ht_dict`].
pub unsafe fn find_var_ht(
    name: *const c_char,
    name_len: size_t,
    varname: *mut *const c_char,
) -> *mut HashTab {
    let mut d: *mut Dict = ptr::null_mut();
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
    // SAFETY: the caller's obligation -- a NUL-terminated name; the answer
    // borrows the item that was found or the caller's scratch.
    let v = unsafe { find_var(name, cstr::bytes_at(name).len(), ptr::null_mut(), false) };
    if v.is_null() {
        return ptr::null_mut();
    }
    let tv = unsafe { Di::new(v) }.field_ptr::<TypVal>(offset_of!(DictItem, di_tv));
    unsafe { numbuf.string(&*tv) as *mut c_char }
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
        n = unsafe { eval_variable(name, len, Some(&mut tv), ptr::null_mut(), false, true) }
            .is_ok();
        if n {
            // Handle `d.key`, `l[idx]` and `Func()`.
            n = unsafe { handle_subscript(&raw mut var, &mut tv, &raw mut evalarg, false) }.is_ok();
            if n {
                clear_local(&mut tv);
            }
        }
    }
    if unsafe { *var } != NUL as c_char {
        n = false;
    }
    unsafe { xfree(tofree.cast()) };
    n
}
