//! Turning what the user wrote into the name a `ufunc_T` is stored under.
//!
//! `trans_function_name` is the whole of it: it resolves `s:`/`<SID>` to
//! the `<SNR>N_` mangling, evaluates a curly-brace name, follows a
//! dictionary subscript to a numbered function, and rejects the spellings
//! that are not names at all.  `fname_trans_sid` and `cat_func_name` are
//! the two smaller manglings around it, and `builtin_function` is what
//! decides a name belongs to the builtin table instead.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int, c_void};
use core::mem::{offset_of, size_of_val};
use core::{ptr, slice};

use super::*;
use crate::keycodes::KE_SNR;
use crate::types::NUL;

/// The name of the function `name` refers to.
///
/// When `name` is a variable holding a funcref or a partial, that is the
/// function's own name and `*lenp` is updated to match; otherwise `name` is
/// handed straight back.  `*partialp` is the partial it came out of, when the
/// caller asked for it.
///
/// # Safety
/// `name` has `*lenp` readable bytes and the out-parameters are null or
/// writable.
pub unsafe fn deref_func_name(
    name: *const c_char,
    lenp: *mut c_int,
    partialp: *mut *mut partial_T,
    no_autoload: bool,
    found_var: *mut bool,
) -> *mut c_char {
    unsafe {
        if !partialp.is_null() {
            *partialp = ptr::null_mut();
        }

        // Looking the *variable* up is also what autoloads `pkg#name`'s
        // package: `find_var` -> `check_vars` sources it on the way.
        let v = find_var(name, *lenp as size_t, ptr::null_mut(), no_autoload);
        if v.is_null() {
            return name as *mut c_char;
        }
        let tv = &raw mut (*v).di_tv;
        if !found_var.is_null() {
            *found_var = true;
        }

        if (*tv).v_type == VAR_FUNC {
            if (*tv).vval.v_string.is_null() {
                // Just in case.
                *lenp = 0;
                return c"".as_ptr() as *mut c_char;
            }
            *lenp = strlen((*tv).vval.v_string) as c_int;
            return (*tv).vval.v_string;
        }

        if (*tv).v_type == VAR_PARTIAL {
            let pt = (*tv).vval.v_partial;
            if pt.is_null() {
                // Just in case.
                *lenp = 0;
                return c"".as_ptr() as *mut c_char;
            }
            if !partialp.is_null() {
                *partialp = pt;
            }
            let s = partial_name(pt);
            *lenp = strlen(s) as c_int;
            return s;
        }

        name as *mut c_char
    }
}

/// Report `errmsg` about `name`, rendering the `<SNR>` mangling back into
/// something a user can read.
///
/// # Safety
/// `errmsg` is an untranslated format with one `%s`, and `name` is
/// NUL-terminated.
pub unsafe fn emsg_funcname(errmsg: *const c_char, name: *const c_char) {
    unsafe {
        let mut p = name as *mut c_char;
        if *name as u8 as c_int == K_SPECIAL && *name.add(1) != 0 && *name.add(2) != 0 {
            p = concat_str(c"<SNR>".as_ptr(), name.add(3));
        }
        semsg_c!(gettext(errmsg), p);
        if !core::ptr::eq(p, name) {
            xfree(p as *mut c_void);
        }
    }
}

/// How long a mangled name may be before `fname_trans_sid` has to allocate.
pub const FLEN_FIXED: c_int = 40;

/// Whether a script-local prefix was written `s:` rather than `<SNR>` --
/// which decides whether the *current* script id has to be substituted in.
///
/// # Safety
/// `name` is a prefix `eval_fname_script` already accepted, so it has at
/// least three readable bytes.
unsafe fn eval_fname_sid(name: *const c_char) -> bool {
    unsafe { *name == b's' as c_char || (*name.add(2) as u8).eq_ignore_ascii_case(&b'I') }
}

/// Rewrite `s:`/`<SID>` at the front of `name` into the `<SNR>N_` byte
/// sequence, using `fname_buf` when the result fits and an allocation
/// (handed back through `tofree`) when it does not.
///
/// # Safety
/// `name` is NUL-terminated, `fname_buf` has `FLEN_FIXED + 1` bytes, and
/// `tofree`/`error` are writable.
pub(crate) unsafe fn fname_trans_sid(
    name: *const c_char,
    fname_buf: *mut c_char,
    tofree: *mut *mut c_char,
    error: *mut c_int,
) -> *mut c_char {
    unsafe {
        let script_name = name.offset(eval_fname_script(name) as isize);
        if script_name == name {
            // "name" doesn't start with "s:" or "<SID>".
            return name as *mut c_char;
        }

        *fname_buf = K_SPECIAL as c_char;
        *fname_buf.add(1) = KS_EXTRA as c_char;
        *fname_buf.add(2) = KE_SNR as c_char;
        let mut fname_buflen: size_t = 3;
        if !eval_fname_sid(name) {
            // "<SID>" or "<SNR>"
            *fname_buf.add(fname_buflen) = NUL as c_char;
        } else if current_sctx.get().sc_sid <= 0 {
            *error = FCERR_SCRIPT;
        } else {
            fname_buflen += snprintf(
                fname_buf.add(fname_buflen),
                (FLEN_FIXED as size_t + 1).wrapping_sub(fname_buflen),
                c"%d_".as_ptr(),
                current_sctx.get().sc_sid,
            ) as size_t;
        }
        let fnamelen = fname_buflen + strlen(script_name);
        if fnamelen < FLEN_FIXED as size_t {
            strcpy(fname_buf.add(fname_buflen), script_name);
            fname_buf
        } else {
            let fname = xmalloc(fnamelen + 1) as *mut c_char;
            *tofree = fname;
            snprintf(
                fname,
                fnamelen + 1,
                c"%s%s".as_ptr(),
                fname_buf,
                script_name,
            );
            fname
        }
    }
}

/// The function stored under `name`, or null.
///
/// # Safety
/// `name` is NUL-terminated.
pub unsafe fn find_func(name: *const c_char) -> *mut ufunc_T {
    unsafe {
        let hi = hash_find(func_hashtab.ptr(), name);
        if (*hi).is_kept() {
            // The key *is* the function's trailing name member, so the
            // function is that many bytes before it.
            (*hi).hi_key.sub(offset_of!(ufunc_T, uf_name)) as *mut ufunc_T
        } else {
            ptr::null_mut()
        }
    }
}

/// Whether `ufunc` is a global function rather than a script-local one --
/// which is exactly whether its stored name carries the `<SNR>` mangling.
///
/// # Safety
/// `ufunc` is a live function.
unsafe fn func_is_global(ufunc: *const ufunc_T) -> bool {
    unsafe { *((&raw const (*ufunc).uf_name) as *const c_char) as u8 as c_int != K_SPECIAL }
}

/// Write `fp`'s printable name into `buf`, answering how much was written
/// (capped at `bufsize - 1`).
///
/// # Safety
/// `fp` is a live function and `buf` has `bufsize` writable bytes.
pub(crate) unsafe fn cat_func_name(buf: *mut c_char, bufsize: size_t, fp: *const ufunc_T) -> c_int {
    unsafe {
        let uflen = (*fp).uf_namelen;
        debug_assert!(uflen > 0);
        let name = (&raw const (*fp).uf_name) as *const c_char;
        let len = if !func_is_global(fp) && uflen > 3 {
            snprintf(buf, bufsize, c"<SNR>%s".as_ptr(), name.add(3))
        } else {
            snprintf(buf, bufsize, c"%s".as_ptr(), name)
        };
        debug_assert!(len > 0);
        len.min(bufsize as c_int - 1)
    }
}

/// Whether a function of this name is reference-counted: the numbered
/// dictionary functions and the lambdas, and nothing else.
///
/// # Safety
/// `name` is NUL-terminated.
pub(crate) unsafe fn func_name_refcount(name: *const c_char) -> bool {
    unsafe {
        (*name as u8).is_ascii_digit()
            || (*name == b'<' as c_char && *name.add(1) == b'l' as c_char)
    }
}

/// Whether `name` names a builtin function: it starts lowercase, is not a
/// scoped name, and carries no `#` (which would make it an autoload name).
///
/// `len` is the name's length, or -1 for "NUL-terminated".
///
/// # Safety
/// `name` has `len` readable bytes, or is NUL-terminated when `len` is -1.
pub(crate) unsafe fn builtin_function(name: *const c_char, len: c_int) -> bool {
    unsafe {
        if !(*name as u8).is_ascii_lowercase() || *name.add(1) == b':' as c_char {
            return false;
        }
        // The two spellings upstream uses -- `strchr` when the length is
        // unknown, `memchr` when it is -- are one search over the same bytes.
        let n = if len == -1 {
            strlen(name)
        } else {
            len as size_t
        };
        !slice::from_raw_parts(name as *const u8, n).contains(&(AUTOLOAD_CHAR as u8))
    }
}

/// The name to show a user: the unmangled `<SNR>123_name` when there is one.
///
/// # Safety
/// `fp` is a live function.
pub unsafe fn printable_func_name(fp: *mut ufunc_T) -> *mut c_char {
    unsafe {
        if !(*fp).uf_name_exp.is_null() {
            (*fp).uf_name_exp
        } else {
            uf_name_ptr(fp)
        }
    }
}

/// Build the stored name out of a resolved lvalue: strip the scope prefix,
/// prepend the `<SNR>` mangling when the name is script-local, and reject
/// the two spellings that cannot be function names.
///
/// `lead` comes in as `eval_fname_script`'s answer (0, 2 or 5) and is
/// reworked here into the *number of bytes* to prepend: 0 for a global name,
/// 3 for `<SNR>` alone, or 3 plus the script id for `s:`/`<SID>`.
///
/// # Safety
/// `lv` is a resolved lvalue with a non-null `ll_name`, and `start`/`end`
/// bracket the name in the command line.
unsafe fn mangle_function_name(
    pp: *mut *mut c_char,
    lv: &mut lval_T,
    start: *const c_char,
    end: *const c_char,
    mut lead: c_int,
    skip: bool,
    flags: c_int,
) -> *mut c_char {
    unsafe {
        let mut len;
        if !lv.ll_exp_name.is_null() {
            len = strlen(lv.ll_exp_name) as c_int;
            if lead <= 2
                && core::ptr::eq(lv.ll_name, lv.ll_exp_name)
                && lv.ll_name_len >= 2
                && memcmp(
                    lv.ll_name as *const c_void,
                    c"s:".as_ptr() as *const c_void,
                    2,
                ) == 0
            {
                // When there was "s:" already, or the name expanded to get a
                // leading "s:", remove it.
                lv.ll_name = lv.ll_name.add(2);
                lv.ll_name_len = lv.ll_name_len.wrapping_sub(2);
                len -= 2;
                lead = 2;
            }
        } else {
            // Skip over "s:" and "g:".  The length subtraction wraps, and
            // upstream's does too: `get_lval` in *skip* mode leaves
            // `ll_name_len` 0, which `:function s:Name()` inside a false
            // `:if` reaches.  Nothing reads the wrapped length on that path
            // (`skip` forces `lead` to 0 and gates the E884 check), but a
            // plain `-=` aborts a debug build there.
            if lead == 2 || (*lv.ll_name == b'g' as c_char && *lv.ll_name.add(1) == b':' as c_char)
            {
                lv.ll_name = lv.ll_name.add(2);
                lv.ll_name_len = lv.ll_name_len.wrapping_sub(2);
            }
            len = end.offset_from(lv.ll_name) as c_int;
        }
        let mut sid_buf: [c_char; 20] = [0; 20];
        let mut sid_buflen: size_t = 0;

        // Accept <SID>name() inside a script, translated into <SNR>123_name();
        // accept <SNR>123_name() outside one.
        if skip {
            lead = 0; // do nothing
        } else if lead > 0 {
            lead = 3;
            if (!lv.ll_exp_name.is_null() && eval_fname_sid(lv.ll_exp_name)) || eval_fname_sid(*pp)
            {
                // It's "s:" or "<SID>".
                if current_sctx.get().sc_sid <= 0 {
                    emsg(gettext(&raw const e_usingsid as *const c_char));
                    return ptr::null_mut();
                }
                sid_buflen = snprintf(
                    sid_buf.as_mut_ptr(),
                    size_of_val(&sid_buf),
                    c"%d_".as_ptr(),
                    current_sctx.get().sc_sid,
                ) as size_t;
                lead += sid_buflen as c_int;
            }
        } else if flags & TFN_INT == 0 && builtin_function(lv.ll_name, lv.ll_name_len as c_int) {
            semsg_c!(
                gettext(c"E128: Function name must start with a capital or \"s:\": %s".as_ptr()),
                start,
            );
            return ptr::null_mut();
        }

        if !skip && flags & TFN_QUIET == 0 && flags & TFN_NO_DEREF == 0 {
            // Upstream also asks `cp < end`.  `cp` points into `lv.ll_name`,
            // which for a curly-brace name is a fresh allocation while `end`
            // points into the command line: that compares two unrelated
            // objects and answers whatever the allocator happened to do.
            // `xmemrchr` is already bounded by `ll_name_len`, so every colon
            // it finds is inside the name and the extra test adds nothing but
            // the coin flip (O-B14-12).
            let cp = xmemrchr(lv.ll_name as *const c_void, b':', lv.ll_name_len);
            if !cp.is_null() {
                semsg_c!(
                    gettext(c"E884: Function name cannot contain a colon: %s".as_ptr()),
                    start,
                );
                return ptr::null_mut();
            }
        }

        let name = xmalloc(len as size_t + lead as size_t + 1) as *mut c_char;
        if !skip && lead > 0 {
            *name = K_SPECIAL as c_char;
            *name.add(1) = KS_EXTRA as c_char;
            *name.add(2) = KE_SNR as c_char;
            if sid_buflen > 0 {
                // It's "<SID>", so the script id goes in as well.
                memcpy(
                    name.add(3) as *mut c_void,
                    sid_buf.as_ptr() as *const c_void,
                    sid_buflen,
                );
            }
        }
        memmove(
            name.offset(lead as isize) as *mut c_void,
            lv.ll_name as *const c_void,
            len as size_t,
        );
        *name.offset((lead + len) as isize) = NUL as c_char;
        *pp = end as *mut c_char;
        name
    }
}

/// Read a function name at `*pp` and answer it in allocated memory, or null
/// when there is not one there.
///
/// # Safety
/// `*pp` is a NUL-terminated command line; `fdp` and `partial` are null or
/// writable.
pub unsafe fn trans_function_name(
    pp: *mut *mut c_char,
    skip: bool,
    flags: c_int,
    fdp: *mut funcdict_T,
    partial: *mut *mut partial_T,
) -> *mut c_char {
    unsafe {
        let mut name: *mut c_char = ptr::null_mut();
        let mut len;
        let mut lv = LVAL_INITIAL_VALUE;

        if !fdp.is_null() {
            memset(fdp as *mut c_void, 0, size_of::<funcdict_T>());
        }
        let mut start: *const c_char = *pp;

        // A hard-coded <SNR> is an already translated function id, from a
        // user command.
        if *(*pp) as u8 as c_int == K_SPECIAL
            && *(*pp).add(1) as u8 as c_int == KS_EXTRA
            && *(*pp).add(2) as c_int == KE_SNR as c_int
        {
            *pp = (*pp).add(3);
            len = get_id_len(pp as *mut *const c_char) + 3;
            return xmemdupz(start as *const c_void, len as size_t) as *mut c_char;
        }

        // A name starting with "<SID>" or "<SNR>" is local to a script.  But
        // don't skip over "s:", `get_lval` needs it for "s:dict.func".
        let lead = eval_fname_script(start);
        if lead > 2 {
            start = start.add(lead as usize);
        }

        // The TFN_ flags use the same values as the GLV_ ones.
        let end: *const c_char = get_lval(
            start as *mut c_char,
            ptr::null_mut(),
            &raw mut lv,
            false,
            skip,
            flags | GLV_READ_ONLY,
            if lead > 2 { 0 } else { FNE_CHECK_START },
        );

        'theend: {
            if end == start {
                if !skip {
                    emsg(gettext(c"E129: Function name required".as_ptr()));
                }
                break 'theend;
            }
            if end.is_null() || (!lv.ll_tv.is_null() && (lead > 2 || lv.ll_range)) {
                // Report an invalid expression in braces, unless the
                // evaluation was cancelled by an aborting error, an interrupt
                // or an exception.
                if !aborting() {
                    if !end.is_null() {
                        semsg_c!(gettext(&raw const e_invarg2 as *const c_char), start);
                    }
                } else {
                    *pp = find_name_end(start, ptr::null_mut(), ptr::null_mut(), FNE_INCL_BR)
                        as *mut c_char;
                }
                break 'theend;
            }

            if !lv.ll_tv.is_null() {
                if !fdp.is_null() {
                    (*fdp).fd_dict = lv.ll_dict;
                    (*fdp).fd_newkey = lv.ll_newkey;
                    lv.ll_newkey = ptr::null_mut();
                    (*fdp).fd_di = lv.ll_di;
                }
                if (*lv.ll_tv).v_type == VAR_FUNC && !(*lv.ll_tv).vval.v_string.is_null() {
                    name = xstrdup((*lv.ll_tv).vval.v_string);
                    *pp = end as *mut c_char;
                } else if (*lv.ll_tv).v_type == VAR_PARTIAL && !(*lv.ll_tv).vval.v_partial.is_null()
                {
                    if is_luafunc((*lv.ll_tv).vval.v_partial) && *end == b'.' as c_char {
                        len = check_luafunc_name(end.add(1), true);
                        if len == 0 {
                            semsg_c!(&raw const e_invexpr2 as *const c_char, c"v:lua".as_ptr());
                            break 'theend;
                        }
                        name = xmallocz(len as size_t) as *mut c_char;
                        memcpy(
                            name as *mut c_void,
                            end.add(1) as *const c_void,
                            len as size_t,
                        );
                        *pp = (end as *mut c_char).add(1).offset(len as isize);
                    } else {
                        name = xstrdup(partial_name((*lv.ll_tv).vval.v_partial));
                        *pp = end as *mut c_char;
                    }
                    if !partial.is_null() {
                        *partial = (*lv.ll_tv).vval.v_partial;
                    }
                } else {
                    if !skip
                        && flags & TFN_QUIET == 0
                        && (fdp.is_null() || lv.ll_dict.is_null() || (*fdp).fd_newkey.is_null())
                    {
                        emsg(gettext(E_FUNCREF.as_ptr()));
                    } else {
                        *pp = end as *mut c_char;
                    }
                    name = ptr::null_mut();
                }
                break 'theend;
            }

            if lv.ll_name.is_null() {
                // Error found, but carry on after the function name.
                *pp = end as *mut c_char;
                break 'theend;
            }

            // Check whether the name is a funcref; if so, use its value.
            if !lv.ll_exp_name.is_null() {
                len = strlen(lv.ll_exp_name) as c_int;
                name = deref_func_name(
                    lv.ll_exp_name,
                    &raw mut len,
                    partial,
                    flags & TFN_NO_AUTOLOAD != 0,
                    ptr::null_mut(),
                );
                if name == lv.ll_exp_name {
                    name = ptr::null_mut();
                }
            } else if flags & TFN_NO_DEREF == 0 {
                len = end.offset_from(*pp) as c_int;
                name = deref_func_name(
                    *pp,
                    &raw mut len,
                    partial,
                    flags & TFN_NO_AUTOLOAD != 0,
                    ptr::null_mut(),
                );
                if name == *pp {
                    name = ptr::null_mut();
                }
            }
            if !name.is_null() {
                name = xstrdup(name);
                *pp = end as *mut c_char;
                if strncmp(name, c"<SNR>".as_ptr(), 5) == 0 {
                    // Change "<SNR>" to the byte sequence.
                    *name = K_SPECIAL as c_char;
                    *name.add(1) = KS_EXTRA as c_char;
                    *name.add(2) = KE_SNR as c_char;
                    memmove(
                        name.add(3) as *mut c_void,
                        name.add(5) as *const c_void,
                        strlen(name.add(5)) + 1,
                    );
                }
                break 'theend;
            }

            name = mangle_function_name(pp, &mut lv, start, end, lead, skip, flags);
        }

        clear_lval(&raw mut lv);
        name
    }
}

/// Expand `s:`/`<SID>` at the front of `funcname` into `<SNR>N_`, in
/// allocated memory.  Answers null when there is no such prefix, or when
/// there is no script to take the id from.
///
/// # Safety
/// `funcname` is null or NUL-terminated.
pub unsafe fn get_scriptlocal_funcname(funcname: *mut c_char) -> *mut c_char {
    unsafe {
        if funcname.is_null() {
            return ptr::null_mut();
        }
        if strncmp(funcname, c"s:".as_ptr(), 2) != 0 && strncmp(funcname, c"<SID>".as_ptr(), 5) != 0
        {
            // The function name does not have a script-local prefix.
            return ptr::null_mut();
        }
        let sid = current_sctx.get().sc_sid;
        if !(sid > 0 && sid <= (*script_items.ptr()).ga_len) {
            emsg(gettext(&raw const e_usingsid as *const c_char));
            return ptr::null_mut();
        }

        let mut sid_buf: [c_char; 25] = [0; 25];
        let sid_buflen = snprintf(
            sid_buf.as_mut_ptr(),
            size_of_val(&sid_buf),
            c"<SNR>%d_".as_ptr(),
            sid,
        ) as size_t;
        let off = if *funcname == b's' as c_char { 2 } else { 5 };
        let newnamesize = sid_buflen + strlen(funcname.add(off)) + 1;
        let newname = xmalloc(newnamesize) as *mut c_char;
        snprintf(
            newname,
            newnamesize,
            c"%s%s".as_ptr(),
            sid_buf.as_ptr(),
            funcname.add(off),
        );
        newname
    }
}

/// [`trans_function_name`], except that a `<lambda>N` is taken as-is.
/// Answers the name in allocated memory.
///
/// # Safety
/// `*name` is a NUL-terminated command line; `fudi` is null or writable.
pub unsafe fn save_function_name(
    name: *mut *mut c_char,
    skip: bool,
    flags: c_int,
    fudi: *mut funcdict_T,
) -> *mut c_char {
    unsafe {
        let mut p = *name;
        let saved;
        if strncmp(p, c"<lambda>".as_ptr(), 8) == 0 {
            p = p.add(8);
            getdigits(&raw mut p, false, 0);
            saved = xmemdupz(*name as *const c_void, p.offset_from(*name) as size_t) as *mut c_char;
            if !fudi.is_null() {
                memset(fudi as *mut c_void, 0, size_of::<funcdict_T>());
            }
        } else {
            saved = trans_function_name(&raw mut p, skip, flags, fudi, ptr::null_mut());
        }
        *name = p;
        saved
    }
}

/// How long the script-local prefix at `p` is: 5 for `<SID>`/`<SNR>`, 2 for
/// `s:`, 0 for neither.
///
/// # Safety
/// `p` is NUL-terminated.
pub unsafe fn eval_fname_script(p: *const c_char) -> c_int {
    unsafe {
        // Writing `s:` instead of `<SID>` is allowed, and `<SNR>` is what a
        // name that has already been translated looks like.
        if *p == b'<' as c_char
            && (mb_strnicmp(p.add(1), c"SID>".as_ptr(), 4) == 0
                || mb_strnicmp(p.add(1), c"SNR>".as_ptr(), 4) == 0)
        {
            return 5;
        }
        if *p == b's' as c_char && *p.add(1) == b':' as c_char {
            return 2;
        }
        0
    }
}
