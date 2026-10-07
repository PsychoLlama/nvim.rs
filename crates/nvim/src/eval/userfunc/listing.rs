//! Printing functions back, and `:delfunction`.
//!
//! `list_functions` walks the whole table, `list_functions_matching_pat`
//! the subset a `/pattern/` matches, and `list_one_function` prints one
//! with its numbered body lines.  `ex_delfunction` is here because it is
//! the same argument parse in reverse; `function_exists` and
//! `get_user_func_name` answer `exists('*x')` and completion.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use crate::message_fmt::msg_bytes;
use crate::regexp::OwnedProg;
use crate::semsg;
use core::ffi::c_int;
use std::rc::Rc;

use super::*;
use crate::types::{Candidate, ExpandContext, IOSIZE};
use std::ffi::CString;

/// Print the head of every function, or of the ones `pattern` matches.
pub(crate) fn list_functions(mut pattern: Option<&mut OwnedProg>) {
    let prev_ht_changed = func_table_changed();
    let mut todo = func_table_used();
    let mut idx = 0;

    msg_ext_set_kind(c"list_cmd");
    while todo > 0 && !got_int.get() {
        if let Some(func) = func_at_slot(idx) {
            todo -= 1;
            // Without a pattern, skip what the user filtered out and the
            // numbered/lambda functions; with one, skip the numbered
            // functions and ask the pattern.
            let name = func.name().as_cstr();
            let show = match pattern.as_deref_mut() {
                None => !message_filtered(name) && !func_name_refcount(name.to_bytes()),
                Some(pattern) => {
                    !cstr::first(name).is_ascii_digit() && pattern.exec(name, 0, p_ic()).is_some()
                }
            };
            if show {
                if list_func_head(&func, false, false).is_err() {
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
    let mut at = start + skip_regexp_at(excmd.line.tail(start), c_int::from(b'/'), 1);
    if !excmd.skip {
        // The compiler reads a C string and the pattern is a span of the
        // command line, so it is copied rather than terminated in place.
        let pat = cstr::owned(excmd.line.slice_at(start, at - start));
        if let Some(mut prog) = OwnedProg::compile(&pat, RE_MAGIC) {
            list_functions(Some(&mut prog));
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
pub(crate) fn list_one_function(excmd: &mut ExArg, name: &[u8], at: usize) -> Option<Rc<UserFunc>> {
    if ends_excmd(c_int::from(excmd.line.byte_at(excmd.line.skip_white(at)))) == 0 {
        let rest = msg_bytes(excmd.line.rest_of(at));
        semsg!("E488: Trailing characters: {rest}");
        return None;
    }
    excmd.line.next = excmd.line.check_next(at);
    if excmd.line.next.is_some() {
        excmd.line.terminate_at(at);
    }
    if excmd.skip || got_int.get() {
        return None;
    }

    let Some(func) = find_func(name) else {
        emsg_funcname(c"E123: Undefined function: %s", name);
        return None;
    };

    // Check no function was added or removed from a callback, and
    // therefore that `func` is still the function this started on.
    let prev_ht_changed = func_table_changed();
    msg_ext_set_kind(c"list_cmd");
    if list_func_head(&func, !excmd.forceit, excmd.forceit).is_err() {
        return Some(func);
    }
    let body = func.body();
    for (j, line) in body.lines.iter().enumerate() {
        if got_int.get() {
            break;
        }
        let Some(line) = line else {
            continue;
        };
        msg_putchar(c_int::from(b'\n'));
        if !excmd.forceit {
            // The line number, right-aligned in three columns.
            msg_outnum(c_int::try_from(j + 1).unwrap_or(c_int::MAX));
            if j < 9 {
                msg_putchar(c_int::from(b' '));
            }
            if j < 99 {
                msg_putchar(c_int::from(b' '));
            }
            if function_list_modified(prev_ht_changed) != 0 {
                break;
            }
        }
        cstr::with_terminated(line, |line| msg_prt_line(line, false));
        line_breakcheck();
    }
    if !got_int.get() {
        msg_putchar(c_int::from(b'\n'));
        if function_list_modified(prev_ht_changed) == 0 {
            msg_str(if excmd.forceit {
                c"endfunction"
            } else {
                c"   endfunction"
            });
        }
    }
    Some(func)
}

/// Whether a function of this *already translated* name exists, builtin or
/// user-defined.
pub(crate) fn translated_function_exists(name: &[u8]) -> bool {
    if builtin_function(name) {
        return find_builtin(name).is_some();
    }
    func_exists(name)
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

/// `func`'s printable name: a script-local one's mangling spelled `<SNR>`.
fn cat_func_name(func: &UserFunc) -> Vec<u8> {
    let name = func.name().as_bytes();
    debug_assert!(!name.is_empty());
    let global = name.first().map(|&b| c_int::from(b)) != Some(K_SPECIAL);
    if !global && name.len() > 3 {
        let mut text = b"<SNR>".to_vec();
        text.extend_from_slice(&name[3..]);
        text
    } else {
        name.to_vec()
    }
}

/// Completion over the user functions: answers the `idx`th name, resuming
/// from where the last call stopped.
///
/// Called with `idx` 0 first, then increasing; a change to the function
/// table in between ends the walk.
pub fn get_user_func_name(expand: &Expand, idx: usize) -> Option<Candidate> {
    static done: GlobalCell<usize> = GlobalCell::new(0);
    static changed: GlobalCell<c_int> = GlobalCell::new(0);
    // The cursor is a slot *index*, parked in a `static` across calls into
    // the editor, which may change the table in between.
    static slot: GlobalCell<usize> = GlobalCell::new(0);

    if idx == 0 {
        done.set(0);
        slot.set(0);
        changed.set(func_table_changed());
    }
    if changed.get() != func_table_changed() || done.get() >= func_table_used() {
        return None;
    }

    if done.get() > 0 {
        slot.set(slot.get() + 1);
    }
    done.set(done.get() + 1);
    let func = loop {
        if let Some(func) = func_at_slot(slot.get()) {
            break func;
        }
        slot.set(slot.get() + 1);
    };
    let name = func.name();

    if func.has_flag(FuncFlags::DICT) || name.as_bytes().starts_with(b"<lambda>") {
        // Don't show dict and lambda functions.
        return Some(Candidate::Borrowed(c""));
    }
    if name.as_bytes().len() + 4 >= IOSIZE as usize {
        // Prevent overflow.
        return Some(Candidate::Owned(name.as_cstr().to_owned()));
    }

    let mut text = cat_func_name(&func);
    if expand.context != ExpandContext::UserFunc {
        text.push(b'(');
        let body = func.body();
        if !body.varargs && body.args.is_empty() {
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
    let Some(func) = find_func(&name) else {
        if !excmd.forceit {
            let arg = msg_bytes(excmd.line.arg());
            semsg!("E130: Unknown function: {arg}");
        }
        return;
    };
    if func.calls.get() > 0 {
        let arg = msg_bytes(excmd.line.arg());
        semsg!("E131: Cannot delete function {arg}: It is in use");
        return;
    }
    // `> 2` because deleting a function should also drop a reference, and
    // 1 is the initial refcount.  A funccall that outlived its call --
    // one that returned `a:000`, or that a closure captured -- holds one
    // of its own until the garbage collector frees it, which is why this
    // arm is reachable at all (see the docket's O-B14-13).
    if func.refcount.get().get() > 2 {
        let arg = msg_bytes(excmd.line.arg());
        semsg!("Cannot delete function {arg}: It is being used internally");
        return;
    }

    if let Some(FuncDict { mut dict, key, .. }) = dict {
        // Delete the dict item that refers to the function; that invokes
        // `func_unref` and possibly deletes the function.
        if dict.remove_key(&key).is_none() {
            let arg0 = "tv_dict_item_remove()";
            semsg!("E685: Internal error: {arg0}");
        }
        return;
    }
    // A normal function has a refcount of 1 for its entry in the
    // hashtable; a numbered function or a lambda has none.  Above that,
    // something else still holds it, so unlink it but keep it.
    let held = if func_name_refcount(func.name().as_bytes()) {
        0
    } else {
        1
    };
    if func.refcount.get().get() > held {
        if remove_func(func.name().as_bytes()) {
            func.release();
        }
        func.flags.set(func.flags.get() | FuncFlags::DELETED);
    } else {
        func_clear_free(&func, false);
    }
}
