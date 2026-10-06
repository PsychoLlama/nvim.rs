//! Printing functions back, and `:delfunction`.
//!
//! `list_functions` walks the whole table, `list_functions_matching_pat`
//! the subset a `/pattern/` matches, and `list_one_function` prints one
//! with its numbered body lines.  `ex_delfunction` is here because it is
//! the same argument parse in reverse; `function_exists` and
//! `get_user_func_name` answer `exists('*x')` and completion.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::snprintf;
use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::types::{Candidate, ExpandContext, IOSIZE};
use core::ffi::CStr;
use std::ffi::CString;

/// Print the head of every function, or of the ones `pattern` matches.
pub(crate) fn list_functions(mut pattern: Option<&mut RegMatch>) {
    let prev_ht_changed = func_table().changed();
    let mut todo = func_table().used();
    let mut idx = 0;

    msg_ext_set_kind(c"list_cmd");
    while todo > 0 && !got_int.get() {
        let hi = func_table().slot(idx);
        if hi.is_kept() {
            // The key *is* the function's trailing name member, so the
            // function is that many bytes before it.
            let fp = uf_from_name_ptr(hi.hi_key);
            todo -= 1;
            // Without a pattern, skip what the user filtered out and the
            // numbered/lambda functions; with one, skip the numbered
            // functions and ask the pattern.
            // SAFETY: a function's name is NUL-terminated.
            let name = unsafe { cstr::at(uf_name_ptr(fp)) };
            let show = match pattern.as_deref_mut() {
                None => !message_filtered(name) && !func_name_refcount(name.to_bytes()),
                Some(pattern) => {
                    !cstr::first(name).is_ascii_digit() && vim_regexec(pattern, name, 0)
                }
            };
            if show {
                if unsafe { list_func_head(fp, false, false) }.is_err() {
                    return;
                }
                if function_list_modified(prev_ht_changed) != 0 {
                    return;
                }
            }
        }
        idx += 1;
    }
}

/// `:function /pattern/`: compile the pattern, list what it matches, and
/// answer where it ends.
pub(crate) fn list_functions_matching_pat(excmd: &mut ExArg) -> usize {
    let start = excmd.line.arg + 1;
    let mut at = start + skip_regexp_at(excmd.line.tail(start), b'/' as c_int, 1);
    if !excmd.skip {
        let mut regmatch = REGMATCH_INIT;
        // The compiler reads a C string and the pattern is a span of the
        // command line, so it is copied rather than terminated in place.
        let pat = cstr::owned(excmd.line.slice_at(start, at - start));
        regmatch.regprog = vim_regcomp(&pat, RE_MAGIC);
        if !regmatch.regprog.is_null() {
            regmatch.rm_ic = p_ic();
            list_functions(Some(&mut regmatch));
            unsafe { vim_regfree(regmatch.regprog) };
        }
    }
    if excmd.line.byte_at(at) == b'/' {
        at += 1;
    }
    at
}

/// `:function Name`: print one function with its numbered body lines.
/// Answers the function, so that the caller can go on to redefine it.
///
/// `name` is the translated name and `at` where it ends in the command line.
pub(crate) fn list_one_function(excmd: &mut ExArg, name: &[u8], at: usize) -> *mut UserFunc {
    if ends_excmd(c_int::from(excmd.line.byte_at(excmd.line.skip_white(at)))) == 0 {
        let rest = msg_bytes(excmd.line.rest_of(at));
        semsg!("E488: Trailing characters: {rest}");
        return ptr::null_mut();
    }
    excmd.line.next = excmd.line.check_next(at);
    if excmd.line.next.is_some() {
        excmd.line.terminate_at(at);
    }
    if excmd.skip || got_int.get() {
        return ptr::null_mut();
    }

    let fp = find_func(name);
    if fp.is_null() {
        emsg_funcname(c"E123: Undefined function: %s", name);
        return ptr::null_mut();
    }

    // Check no function was added or removed from a callback, and
    // therefore that `fp` is still the function this started on.
    let prev_ht_changed = func_table().changed();
    msg_ext_set_kind(c"list_cmd");
    if unsafe { list_func_head(fp, !excmd.forceit, excmd.forceit) }.is_err() {
        return fp;
    }
    // SAFETY: `fp` is the live function just listed.
    let f = unsafe { Uf::new(fp) };
    let lines = ga_strings(&f.uf_lines);
    for (j, &line) in lines.iter().enumerate() {
        if got_int.get() {
            break;
        }
        if line.is_null() {
            continue;
        }
        msg_putchar(b'\n' as c_int);
        if !excmd.forceit {
            // The line number, right-aligned in three columns.
            msg_outnum(j as c_int + 1);
            if j < 9 {
                msg_putchar(b' ' as c_int);
            }
            if j < 99 {
                msg_putchar(b' ' as c_int);
            }
            if function_list_modified(prev_ht_changed) != 0 {
                break;
            }
        }
        msg_prt_line(unsafe { cstr::at(line) }, false);
        line_breakcheck();
    }
    if !got_int.get() {
        msg_putchar(b'\n' as c_int);
        if function_list_modified(prev_ht_changed) == 0 {
            let end = if excmd.forceit {
                c"endfunction".as_ptr()
            } else {
                c"   endfunction".as_ptr()
            };
            msg_str(unsafe { cstr::at(end) });
        }
    }
    fp
}

/// Whether a function of this *already translated* name exists, builtin or
/// user-defined.
pub(crate) fn translated_function_exists(name: &[u8]) -> bool {
    if builtin_function(name) {
        return find_builtin(name).is_some();
    }
    !find_func(name).is_null()
}

/// `exists('*name')`: whether `name` names a function, without autoloading
/// one to find out.
pub(crate) fn function_exists(name: &[u8], no_deref: bool) -> bool {
    let mut flag = TFN_INT | TFN_QUIET | TFN_NO_AUTOLOAD;
    if no_deref {
        flag |= TFN_NO_DEREF;
    }
    let found = trans_function_name(name, false, flag, false);
    let after = found.end + skip::white(&name[found.end..]);

    // Only accept "funcname", "funcname ", "funcname (..." and
    // "funcname(...", not "funcname!...".
    found.name.is_some_and(|translated| {
        matches!(name.get(after), None | Some(b'(')) && translated_function_exists(&translated)
    })
}

/// Whether `ufunc` is a global function rather than a script-local one --
/// which is exactly whether its stored name carries the `<SNR>` mangling.
///
/// # Safety
/// `ufunc` is a live function.
unsafe fn func_is_global(ufunc: *const UserFunc) -> bool {
    unsafe { *((&raw const (*ufunc).uf_name) as *const c_char) as u8 as c_int != K_SPECIAL }
}

/// Write `func`'s printable name into `buf`, answering how much was written
/// (capped at `bufsize - 1`).
///
/// # Safety
/// `func` is a live function and `buf` has `bufsize` writable bytes.
unsafe fn cat_func_name(buf: *mut c_char, bufsize: size_t, func: *const UserFunc) -> c_int {
    let uflen = unsafe { (*func).uf_namelen };
    debug_assert!(uflen > 0);
    let name = unsafe { &raw const (*func).uf_name } as *const c_char;
    let len = if !unsafe { func_is_global(func) } && uflen > 3 {
        unsafe { snprintf!(buf, bufsize, c"<SNR>%s".as_ptr(), name.add(3)) }
    } else {
        unsafe { snprintf!(buf, bufsize, c"%s".as_ptr(), name) }
    };
    debug_assert!(len > 0);
    len.min(bufsize as c_int - 1)
}

/// Completion over the user functions: answers the `idx`th name, resuming
/// from where the last call stopped.
///
/// Called with `idx` 0 first, then increasing; a change to the function
/// table in between ends the walk.
pub fn get_user_func_name(expand: &Expand, idx: usize) -> Option<Candidate> {
    static done: GlobalCell<size_t> = GlobalCell::new(0);
    static changed: GlobalCell<c_int> = GlobalCell::new(0);
    // The cursor is a slot *index*: it is parked in a `static` across calls
    // into the editor, and the table's small run lives inside the table, so
    // a pointer would not survive the next mutation of it.
    static slot: GlobalCell<usize> = GlobalCell::new(0);

    if idx == 0 {
        done.set(0);
        slot.set(0);
        changed.set(func_table().changed());
    }
    if changed.get() != func_table().changed() || done.get() >= func_table().used() {
        return None;
    }

    if done.get() > 0 {
        slot.set(slot.get() + 1);
    }
    done.set(done.get() + 1);
    while !func_table().slot(slot.get()).is_kept() {
        slot.set(slot.get() + 1);
    }
    // The key *is* the function's trailing name member, so the function is
    // that many bytes before it.
    let fp = uf_from_name_ptr(func_table().slot(slot.get()).hi_key);
    // SAFETY: as above.
    let f = unsafe { Uf::new(fp) };
    // SAFETY: a live function's name is NUL-terminated.
    let name = unsafe { CStr::from_ptr(uf_name_ptr(fp)) };

    if f.uf_flags.has(FuncFlags::DICT) || name.to_bytes().starts_with(b"<lambda>") {
        // Don't show dict and lambda functions.
        return Some(Candidate::Borrowed(c""));
    }
    if f.uf_namelen + 4 >= IOSIZE as size_t {
        // Prevent overflow.
        return Some(Candidate::Owned(name.to_owned()));
    }

    let mut buf = [0 as c_char; IOSIZE as usize];
    // SAFETY: `buf` is `IOSIZE` bytes and `fp` the live function.
    let len = unsafe { cat_func_name(buf.as_mut_ptr(), IOSIZE as size_t, fp) };
    let mut text = cstr::as_bytes(&buf[..len as usize]).to_vec();
    if expand.context != ExpandContext::UserFunc {
        text.push(b'(');
        if f.uf_varargs == 0 && f.uf_args.ga_len <= 0 {
            text.push(b')');
        }
    }
    Some(Candidate::Owned(
        CString::new(text).expect("a function name holds no NUL"),
    ))
}

/// `:delfunction`.
pub fn ex_delfunction(excmd: &mut ExArg) {
    let arg = excmd.line.arg;
    let FunctionName {
        name, end, dict, ..
    } = trans_function_name(excmd.line.rest_of(arg), excmd.skip, 0, true);
    let Some(name) = name else {
        if dict.is_some() && !excmd.skip {
            emsg(gettext(E_FUNCREF));
        }
        return;
    };
    let at = arg + end;
    if ends_excmd(c_int::from(excmd.line.byte_at(excmd.line.skip_white(at)))) == 0 {
        let rest = msg_bytes(excmd.line.rest_of(at));
        semsg!("E488: Trailing characters: {rest}");
        return;
    }
    excmd.line.next = excmd.line.check_next(at);
    if excmd.line.next.is_some() {
        excmd.line.terminate_at(at);
    }

    if name.first().is_some_and(u8::is_ascii_digit) && dict.is_none() {
        // Numbered function.
        if !excmd.skip {
            let arg = msg_bytes(excmd.line.arg());
            semsg!("E475: Invalid argument: {arg}");
        }
        return;
    }
    if excmd.skip {
        return;
    }
    let fp = find_func(&name);

    if fp.is_null() {
        if !excmd.forceit {
            let arg = msg_bytes(excmd.line.arg());
            semsg!("E130: Unknown function: {arg}");
        }
        return;
    }
    if unsafe { (*fp).uf_calls } > 0 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = msg_bytes(excmd.line.arg());
        semsg!("E131: Cannot delete function {arg}: It is in use");
        return;
    }
    // `> 2` because deleting a function should also drop a reference, and
    // 1 is the initial refcount.  A funccall that outlived its call --
    // one that returned `a:000`, or that a closure captured -- holds one
    // of its own until the garbage collector frees it, which is why this
    // arm is reachable at all (see the docket's O-B14-13).
    if unsafe { (*fp).uf_refcount }.get() > 2 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = msg_bytes(excmd.line.arg());
        semsg!("Cannot delete function {arg}: It is being used internally");
        return;
    }

    if let Some(FuncDict { mut dict, key, .. }) = dict {
        // Delete the dict item that refers to the function; that invokes
        // `func_unref` and possibly deletes the function.
        if !dict.remove_key(&key) {
            let arg0 = "tv_dict_item_remove()";
            semsg!("E685: Internal error: {arg0}");
        }
        return;
    }
    // A normal function has a refcount of 1 for its entry in the
    // hashtable; a numbered function or a lambda has none.  Above that,
    // something else still holds it, so unlink it but keep it.
    // SAFETY: `fp` is the live function just found, whose name is
    // NUL-terminated.
    let held = if func_name_refcount(unsafe { cstr::bytes_at(uf_name_ptr(fp)) }) {
        0
    } else {
        1
    };
    if unsafe { (*fp).uf_refcount }.get() > held {
        if unsafe { func_remove(fp) } {
            unsafe { (*fp).uf_refcount.release() };
        }
        unsafe { (*fp).uf_flags |= FuncFlags::DELETED };
    } else {
        unsafe { func_clear_free(fp, false) };
    }
}
