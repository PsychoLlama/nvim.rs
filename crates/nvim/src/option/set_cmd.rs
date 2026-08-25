//! The `:set` family: parsing one argument into an option, a prefix, an
//! operator and a new value.
//!
//! An argument is `[no|inv]name[?|!|&|<|[+^-]=value]`, and the parse is
//! left to right: the prefix, the name, an optional operator, then the
//! character after it decides everything else. [`do_one_set_option`] is
//! that decision tree; [`get_option_newval`] turns what is left into a
//! value of the option's own type.
//!
//! Error reporting is deferred: a rejected argument is reported by
//! [`do_set`], with the text of the argument appended, which is why the
//! parse hands messages back through an out-parameter rather than
//! reporting them where they are found.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;
use core::slice;

use super::{
    NIL_OPTVAL, OptSlot, boolean_optval, option_last_set, optval_boolean, set_op_T,
    ui_refresh_options,
};
use crate::api::private::helpers::cstr_as_string;
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::{skiptowhite_esc, skipwhite, trans_characters, vim_str2nr};
use crate::drawscreen::{UPD_CLEAR, redraw_all_later};
use crate::eval::last_set_msg;
use crate::ex_getln::gotocmdline;
use crate::guard::Suppress;
use crate::keycodes::{K_ZERO, find_special_key};
use crate::main::{
    curbuf, curwin, e_invarg, e_sandbox, e_trailing, info_message, p_mle, p_verbose, sandbox,
    silent_mode,
};
use crate::memory::{strequal, xstrlcpy};
use crate::message::{emsg, msg_ext_set_kind, msg_putchar};
use crate::options::{
    kOptAleph, kOptFoldmethod, kOptInvalid, kOptWildchar, kOptWildcharm, kOptWrap,
};
use crate::os::cshim::{gettext, memmove, strncmp};
use crate::strings::{vim_snprintf, vim_strchr};
use crate::types::{
    CMD_index, CMD_setglobal, CMD_setlocal, FAIL, IOSIZE, NUL, OK, OptIndex, OptInt, OptVal,
    OptValData, OptionSetFlags, exarg_T, scid_T, size_t, uint8_t, uint32_t, uvarnumber_T, win_T,
};
use ::libc::strlen;

use super::{
    FSK_KEEP_X_KEY, FSK_KEYCODE, FSK_SIMPLIFY, OP_ADDING, OP_NONE, OP_PREPENDING, OP_REMOVING,
    STR2NR_ALL, didset_options, didset_options2, get_option, get_option_default, get_option_value,
    get_varp, get_varp_scope, is_tty_option, kOptFlagMLE, kOptFlagSecure, kOptScopeBuf,
    kOptScopeWin, kOptValTypeBoolean, kOptValTypeNil, kOptValTypeNumber, kOptValTypeString,
    option_has_scope, option_has_type, option_is_global_local, option_is_window_local,
    option_scope_idx, option_var, optval_copy, optval_from_varp, set_option, set_options_default,
    showoneopt, showoptions, stropt_get_newval, unset_option_local_value,
};

/// The messages the parse reports. They go back to [`do_set`] rather than
/// being emitted here, so that the offending argument can be appended.
const E_UNKNOWN_OPTION: &CStr = c"E518: Unknown option";
const E_NOT_ALLOWED_IN_MODELINE: &CStr = c"E520: Not allowed in a modeline";
const E_MODELINE_NEEDS_MODELINEEXPR: &CStr =
    c"E992: Not allowed in a modeline when 'modelineexpr' is off";
const E_NUMBER_REQUIRED_AFTER_EQUAL: &CStr = c"E521: Number required after =";

/// What `no` or `inv` in front of a boolean option's name asks for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Prefix {
    None,
    No,
    Inv,
}

/// `:set`, `:setlocal` and `:setglobal`.
///
/// # Safety
///
/// `eap` must be the command's own argument block.
pub(crate) unsafe fn ex_set(eap: *mut exarg_T) {
    // SAFETY: the caller's argument block.
    unsafe {
        let mut flags = match (*eap).cmdidx as CMD_index {
            CMD_setlocal => OptionSetFlags::LOCAL,
            CMD_setglobal => OptionSetFlags::GLOBAL,
            _ => OptionSetFlags::NONE,
        };
        // `:set!` lists one option per line.
        if (*eap).forceit != 0 {
            flags |= OptionSetFlags::ONECOLUMN;
        }
        do_set((*eap).arg, flags);
    }
}

/// The operator at `arg`, if the two characters there are one.
///
/// # Safety
///
/// `arg` must be NUL-terminated.
unsafe fn get_op(arg: *const c_char) -> set_op_T {
    // SAFETY: the caller's string; the second read only happens once the
    // first byte is known not to be the terminator.
    unsafe {
        if *arg == NUL as c_char || *arg.add(1) as c_int != '=' as c_int {
            return OP_NONE;
        }
        match *arg as u8 {
            b'+' => OP_ADDING,
            b'^' => OP_PREPENDING,
            b'-' => OP_REMOVING,
            _ => OP_NONE,
        }
    }
}

/// The prefix at `*argp`, advancing past it.
///
/// # Safety
///
/// `*argp` must be NUL-terminated.
unsafe fn get_option_prefix(argp: &mut *mut c_char) -> Prefix {
    // SAFETY: the caller's string.
    unsafe {
        for (spelling, prefix) in [(c"no", Prefix::No), (c"inv", Prefix::Inv)] {
            let len = spelling.count_bytes();
            if strncmp(*argp, spelling.as_ptr(), len) == 0 {
                *argp = argp.add(len);
                return prefix;
            }
        }
    }
    Prefix::None
}

/// Whether this option may be set at all, here and now: some are refused in
/// a modeline or in the sandbox, and `:setlocal`/`:setglobal` on a modeline
/// only reach the scopes the caller asked for.
///
/// # Safety
///
/// `win` must be a live window.
unsafe fn validate_opt_idx(
    win: *mut win_T,
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
    flags: uint32_t,
    prefix: Prefix,
    errmsg: &mut *const c_char,
) -> c_int {
    if prefix != Prefix::None && !option_has_type(opt_idx, kOptValTypeBoolean) {
        *errmsg = e_invarg.as_ptr();
        return FAIL;
    }
    // A `:set` sweeping over windows or buffers only wants its own kind.
    if opt_flags.has(OptionSetFlags::WINONLY) && !option_is_window_local(opt_idx) {
        return FAIL;
    }
    if opt_flags.has(OptionSetFlags::NOWIN) && option_is_window_local(opt_idx) {
        return FAIL;
    }
    if opt_flags.has(OptionSetFlags::MODELINE) {
        if flags & kOptFlagSecure as uint32_t != 0 {
            *errmsg = E_NOT_ALLOWED_IN_MODELINE.as_ptr();
            return FAIL;
        }
        if flags & kOptFlagMLE as uint32_t != 0 && p_mle.get() == 0 {
            *errmsg = E_MODELINE_NEEDS_MODELINEEXPR.as_ptr();
            return FAIL;
        }
        // A modeline must not undo what `:diffthis` set up.
        // SAFETY: the caller's window is live.
        if unsafe { (*win).w_onebuf_opt.wo_diff != 0 }
            && (opt_idx == kOptFoldmethod || opt_idx == kOptWrap)
        {
            return FAIL;
        }
    }
    if sandbox.get() != 0 && flags & kOptFlagSecure as uint32_t != 0 {
        *errmsg = e_sandbox.as_ptr();
        return FAIL;
    }
    OK
}

/// The end of a terminal option's name at `arg`, or null when `arg` does not
/// start with one. `term` and `ttytype` are spelled out; the rest are
/// `t_xx`, optionally wrapped in `<>`.
///
/// # Safety
///
/// `arg` must be NUL-terminated.
pub(crate) unsafe fn find_tty_option_end(arg: *const c_char) -> *const c_char {
    // SAFETY: the caller's string. Every read below is guarded by the one
    // before it, so the walk stops at the terminator.
    unsafe {
        for name in [c"term", c"ttytype"] {
            if strequal(arg, name.as_ptr()) {
                return arg.add(name.count_bytes());
            }
        }

        let mut p = arg;
        let delimit = *arg as c_int == '<' as c_int;
        if delimit {
            p = p.add(1);
        }
        if *p as c_int == 't' as c_int
            && *p.add(1) as c_int == '_' as c_int
            && *p.add(2) != 0
            && *p.add(3) != 0
        {
            p = p.add(4);
        } else if delimit {
            while *p != NUL as c_char && *p as c_int != '>' as c_int {
                p = p.add(1);
            }
        }
        if delimit {
            if *p as c_int != '>' as c_int {
                return ptr::null();
            }
            p = p.add(1);
        }
        if arg == p { ptr::null() } else { p }
    }
}

/// The end of the option name at `arg`, and the option it names. A terminal
/// option ends where it ends but resolves to `kOptInvalid`; anything that
/// does not start with a letter is not a name at all.
///
/// # Safety
///
/// `arg` must be NUL-terminated, and `opt_idxp` writable.
pub(crate) unsafe fn find_option_end(arg: *const c_char, opt_idxp: *mut OptIndex) -> *const c_char {
    // SAFETY: the caller's string and out-parameter.
    unsafe {
        let tty_end = find_tty_option_end(arg);
        if !tty_end.is_null() {
            *opt_idxp = kOptInvalid;
            return tty_end;
        }
        let mut p = arg;
        while (*p as u8).is_ascii_alphabetic() {
            p = p.add(1);
        }
        if p == arg {
            *opt_idxp = kOptInvalid;
            return ptr::null();
        }
        *opt_idxp = super::find_option_len(slice::from_raw_parts(
            arg.cast::<u8>(),
            p.offset_from(arg) as usize,
        ));
        p
    }
}

/// The value `nextchar` and what follows it ask for, in the option's own
/// type. Owned by the caller.
///
/// `:set opt&` and `:set opt<` short-circuit; everything else is a value of
/// the option's declared type, with the operator applied to the old one.
///
/// # Safety
///
/// `varp` must be the option's variable in the scope `opt_flags` names, and
/// `*argp` NUL-terminated.
#[allow(clippy::too_many_arguments)]
unsafe fn get_option_newval(
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
    prefix: Prefix,
    argp: &mut *mut c_char,
    nextchar: c_int,
    op: set_op_T,
    flags: uint32_t,
    varp: OptSlot,
    errmsg: &mut *const c_char,
) -> OptVal {
    debug_assert!(!varp.is_none());

    let nil = NIL_OPTVAL;

    // SAFETY: the caller's `varp` is this option's variable, and `*argp` is
    // NUL-terminated.
    unsafe {
        // Setting the local value of a global-local option amends whatever
        // it is currently reading through, which may be the global value.
        let oldval_is_global =
            option_is_global_local(opt_idx) && opt_flags.has(OptionSetFlags::LOCAL);
        let oldval = optval_from_varp(
            opt_idx,
            if oldval_is_global {
                get_varp(opt_idx)
            } else {
                varp
            },
        );

        // `:set opt&`. Deliberately `OptionSetFlags::GLOBAL` rather than `opt_flags`, so
        // that a `:setlocal opt&` on a global-local option gets the real
        // default rather than the unset marker.
        if nextchar == '&' as c_int {
            return optval_copy(get_option_default(opt_idx, OptionSetFlags::GLOBAL));
        }
        // `:set opt<` resets to the global value; `:setlocal opt<` copies it
        // into the local one.
        if nextchar == '<' as c_int {
            if option_is_global_local(opt_idx) && !opt_flags.has(OptionSetFlags::LOCAL) {
                unset_option_local_value(opt_idx);
            }
            return get_option_value(opt_idx, OptionSetFlags::GLOBAL);
        }

        match oldval.type_0 {
            kOptValTypeBoolean => {
                let boolean = if nextchar == '!' as c_int {
                    // `:set opt!` inverts, leaving an unset global-local
                    // value unset.
                    optval_boolean(oldval.data).map(|b| !b)
                } else if prefix == Prefix::Inv {
                    Some(*varp.boolean_var() == 0)
                } else {
                    Some(prefix != Prefix::No)
                };
                boolean_optval(boolean)
            }
            kOptValTypeNumber => {
                let oldval_num = oldval.data.number;
                let arg = argp.add(1);
                let Some(newval_num) = take_number(opt_idx, arg, errmsg) else {
                    return nil;
                };
                let number = match op {
                    OP_ADDING => oldval_num + newval_num,
                    // `^=` on a number multiplies; there is nothing to
                    // prepend to.
                    OP_PREPENDING => oldval_num * newval_num,
                    OP_REMOVING => oldval_num - newval_num,
                    _ => newval_num,
                };
                OptVal {
                    type_0: kOptValTypeNumber,
                    data: OptValData { number },
                }
            }
            kOptValTypeString => {
                let mut op = op;
                let newval_str = stropt_get_newval(
                    opt_idx,
                    argp,
                    varp,
                    oldval.data.string.data(),
                    &raw mut op,
                    flags,
                );
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string(newval_str),
                    },
                }
            }
            _ => unreachable!("an option with no type has no value to set"),
        }
    }
}

/// The number at `arg`, or `None` with `errmsg` set.
///
/// 'wildchar' and 'wildcharm' hold a key rather than a count, so for those
/// two a `<xx>` name, a `^x` control spelling or any single non-digit is a
/// key; everything else is `[-]0-9` in any of the bases `vim_str2nr` reads.
///
/// # Safety
///
/// `arg` must be NUL-terminated.
unsafe fn take_number(
    opt_idx: OptIndex,
    arg: *mut c_char,
    errmsg: &mut *const c_char,
) -> Option<OptInt> {
    // SAFETY: the caller's variable and string.
    unsafe {
        let is_key_option = matches!(opt_idx, kOptWildchar | kOptWildcharm);
        let looks_like_key = *arg as c_int == '<' as c_int
            || *arg as c_int == '^' as c_int
            || (*arg != NUL as c_char
                && (*arg.add(1) == 0 || ascii_iswhite(*arg.add(1) as c_int))
                && !ascii_isdigit(*arg as c_int));
        if is_key_option && looks_like_key {
            let key = string_to_key(arg) as OptInt;
            if key == 0 {
                *errmsg = e_invarg.as_ptr();
                return None;
            }
            return Some(key);
        }

        if *arg as c_int != '-' as c_int && !ascii_isdigit(*arg as c_int) {
            *errmsg = E_NUMBER_REQUIRED_AFTER_EQUAL.as_ptr();
            return None;
        }
        let mut len: c_int = 0;
        let mut number: OptInt = 0;
        vim_str2nr(
            arg,
            ptr::null_mut(),
            &raw mut len,
            STR2NR_ALL as c_int,
            &raw mut number,
            ptr::null_mut::<uvarnumber_T>(),
            0,
            true,
            ptr::null_mut::<bool>(),
        );
        if len == 0
            || (*arg.offset(len as isize) != NUL as c_char
                && !ascii_iswhite(*arg.offset(len as isize) as c_int))
        {
            *errmsg = E_NUMBER_REQUIRED_AFTER_EQUAL.as_ptr();
            return None;
        }
        Some(number)
    }
}

/// Parse and apply one `:set` argument, advancing `*argp` past the value.
///
/// # Safety
///
/// `*argp` must be NUL-terminated and `errbuf` writable for `errbuflen`
/// bytes.
unsafe fn do_one_set_option(
    opt_flags: OptionSetFlags,
    argp: &mut *mut c_char,
    did_show: &mut bool,
    errbuf: *mut c_char,
    errbuflen: size_t,
    errmsg: &mut *const c_char,
) {
    let prefix = unsafe { get_option_prefix(argp) };
    let arg = *argp;

    // SAFETY: the caller's string and error buffer.
    unsafe {
        let mut opt_idx: OptIndex = kOptAleph;
        let option_end = find_option_end(arg, &raw mut opt_idx);
        if opt_idx == kOptInvalid {
            // A terminal option is accepted and discarded.
            if !is_tty_option(CStr::from_ptr(arg)) {
                *errmsg = E_UNKNOWN_OPTION.as_ptr();
            }
            return;
        }
        debug_assert!(option_end >= arg);

        // What ends the name decides whether a trailing character is an
        // error; `:set ai  ?` is allowed, `:set ai?x` is not.
        let afterchar = *option_end as uint8_t;
        let mut p = option_end as *mut c_char;
        while ascii_iswhite(*p as c_int) {
            p = p.add(1);
        }
        let op = get_op(p);
        if op != OP_NONE {
            p = p.add(1);
        }
        let nextchar = *p as uint8_t as c_int;

        let flags = get_option(opt_idx).flags;
        let varp = get_varp_scope(opt_idx, opt_flags);

        if validate_opt_idx(curwin.get(), opt_idx, opt_flags, flags, prefix, errmsg) == FAIL {
            return;
        }

        if !vim_strchr(c"?=:!&<".as_ptr(), nextchar).is_null() {
            *argp = p;
            // `:set opt&vi` and `:set opt&vim` both mean `:set opt&` here;
            // nvim has no separate Vi default.
            if nextchar == '&' as c_int
                && *argp.add(1) as c_int == 'v' as c_int
                && *argp.add(2) as c_int == 'i' as c_int
            {
                *argp = argp.add(if *argp.add(3) as c_int == 'm' as c_int {
                    3
                } else {
                    2
                });
            }
            // Nothing may follow the ones that take no value.
            if !vim_strchr(c"?!&<".as_ptr(), nextchar).is_null()
                && *argp.add(1) != NUL as c_char
                && !ascii_iswhite(*argp.add(1) as c_int)
            {
                *errmsg = e_trailing.as_ptr();
                return;
            }
        }

        // `:set opt?`, and a bare `:set opt` for anything but a boolean,
        // shows the value rather than setting it.
        let showing = nextchar == '?' as c_int
            || (prefix == Prefix::None
                && vim_strchr(c"=:&<".as_ptr(), nextchar).is_null()
                && !option_has_type(opt_idx, kOptValTypeBoolean));
        if showing {
            show_one(opt_idx, opt_flags, varp, did_show);
            if nextchar != '?' as c_int
                && nextchar != NUL as c_int
                && !ascii_iswhite(afterchar as c_int)
            {
                *errmsg = e_trailing.as_ptr();
            }
            return;
        }

        if option_has_type(opt_idx, kOptValTypeBoolean) {
            // A boolean takes no value, and nothing may follow it.
            if !vim_strchr(c"=:".as_ptr(), nextchar).is_null() {
                *errmsg = e_invarg.as_ptr();
                return;
            }
            if vim_strchr(c"!&<".as_ptr(), nextchar).is_null()
                && nextchar != NUL as c_int
                && !ascii_iswhite(afterchar as c_int)
            {
                *errmsg = e_trailing.as_ptr();
                return;
            }
        } else if vim_strchr(c"=:&<".as_ptr(), nextchar).is_null() {
            *errmsg = e_invarg.as_ptr();
            return;
        }

        let newval = get_option_newval(
            opt_idx, opt_flags, prefix, argp, nextchar, op, flags, varp, errmsg,
        );
        if newval.type_0 == kOptValTypeNil || !errmsg.is_null() {
            return;
        }
        *errmsg = set_option(
            opt_idx,
            newval,
            opt_flags,
            0 as scid_T,
            false,
            // `+=`/`^=`/`-=` amend the value; only a plain assignment
            // replaces it, which is what clears the insecure mark.
            op == OP_NONE,
            errbuf,
            errbuflen,
        );
    }
}

/// Show one option's value, on its own line, opening the message area the
/// first time.
///
/// # Safety
///
/// `varp` must be the option's variable in the scope `opt_flags` names.
unsafe fn show_one(
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
    varp: OptSlot,
    did_show: &mut bool,
) {
    // SAFETY: `curwin`/`curbuf` are live and the option table is a plain
    // array.
    unsafe {
        if *did_show {
            msg_putchar('\n' as c_int);
        } else {
            msg_ext_set_kind(c"list_cmd".as_ptr());
            gotocmdline(true);
            *did_show = true;
        }
        showoneopt(opt_idx, opt_flags);

        // With 'verbose' set, say where the value came from — from the
        // script context of the scope the value is being read from.
        if p_verbose.get() <= 0 {
            return;
        }
        if varp == option_var(opt_idx) {
            last_set_msg(option_last_set(opt_idx));
        } else if option_has_scope(opt_idx, kOptScopeWin) {
            let at = option_scope_idx(opt_idx, kOptScopeWin) as usize;
            last_set_msg((*curwin.get()).w_onebuf_opt.wo_script_ctx[at]);
        } else if option_has_scope(opt_idx, kOptScopeBuf) {
            let at = option_scope_idx(opt_idx, kOptScopeBuf) as usize;
            last_set_msg((*curbuf.get()).b_p_script_ctx[at]);
        }
    }
}

/// The whole of a `:set` command line: a list of arguments, `all`, `all&`,
/// or nothing at all.
///
/// Returns `FAIL` on the first argument that is rejected, having reported
/// it; the rest of the line is not looked at.
///
/// # Safety
///
/// `arg` must be NUL-terminated.
pub(crate) unsafe fn do_set(arg: *mut c_char, opt_flags: OptionSetFlags) -> c_int {
    let mut did_show = false;
    let mut arg = arg;

    // SAFETY: the caller's string.
    unsafe {
        if *arg == NUL as c_char {
            showoptions(false, opt_flags);
            did_show = true;
        }
        while *arg != NUL as c_char {
            // "all" is only the keyword when it is a whole word, and never
            // in a modeline.
            let is_all = strncmp(arg, c"all".as_ptr(), 3) == 0
                && !(*arg.add(3) as u8).is_ascii_alphabetic()
                && !opt_flags.has(OptionSetFlags::MODELINE);
            if is_all {
                arg = arg.add(3);
                if *arg as c_int == '&' as c_int {
                    arg = arg.add(1);
                    set_options_default(opt_flags);
                    didset_options();
                    didset_options2();
                    ui_refresh_options();
                    redraw_all_later(UPD_CLEAR);
                } else {
                    showoptions(true, opt_flags);
                    did_show = true;
                }
            } else {
                let startarg = arg;
                let mut errmsg: *const c_char = ptr::null();
                let mut errbuf: [c_char; 80] = [0; 80];
                do_one_set_option(
                    opt_flags,
                    &mut arg,
                    &mut did_show,
                    errbuf.as_mut_ptr(),
                    errbuf.len(),
                    &mut errmsg,
                );
                // Step over the value, and over an `=` that starts another
                // one — twice at most, which is what `:set opt=a=b` needs.
                for _ in 0..2 {
                    arg = skiptowhite_esc(arg);
                    arg = skipwhite(arg);
                    if *arg as c_int != '=' as c_int {
                        break;
                    }
                }
                if !errmsg.is_null() {
                    report(errmsg, startarg, arg);
                    return FAIL;
                }
            }
            arg = skipwhite(arg);
        }

        // `-s` suppresses messages, but a `:set` that was asked to show
        // something still ends its listing with a newline.
        if silent_mode.get() && did_show {
            silent_mode.set(false);
            info_message.set(true);
            msg_putchar('\n' as c_int);
            silent_mode.set(true);
            info_message.set(false);
        }
    }
    OK
}

/// Report a rejected argument as "message: argument".
///
/// The argument is appended only when both fit in the report buffer; a
/// message that long stands on its own.
///
/// # Safety
///
/// `errmsg` must be NUL-terminated and `start..=end` one argument of the
/// command line.
unsafe fn report(errmsg: *const c_char, start: *mut c_char, end: *mut c_char) {
    let mut report = [0 as c_char; IOSIZE as usize];
    // SAFETY: the caller's strings, and `report` is `IOSIZE` writable bytes.
    unsafe {
        let buf = report.as_mut_ptr();
        // Two past the message, leaving room for the ": " written back over
        // its terminator.
        let at = vim_snprintf(buf, IOSIZE as size_t, c"%s".as_ptr(), gettext(errmsg)) + 2;
        debug_assert!(end >= start);
        let arglen = end.offset_from(start);
        if at as isize + arglen < IOSIZE as isize {
            xstrlcpy(
                buf.offset(at as isize - 2),
                c": ".as_ptr(),
                (IOSIZE - at + 2) as size_t,
            );
            memmove(
                buf.offset(at as isize).cast::<c_void>(),
                start.cast::<c_void>(),
                arglen as size_t,
            );
            *buf.offset(at as isize + arglen) = NUL as c_char;
        }
        trans_characters(buf, IOSIZE);
        // The message is the whole report; do not make the user acknowledge
        // the half of it that has already scrolled past.
        let _no_prompt = Suppress::wait_return();
        emsg(buf);
    }
}

/// The key `arg` names, given that it is `len` bytes long and, with
/// `has_lt`, arrived inside `<>`. Zero for anything that is not a key.
///
/// # Safety
///
/// `arg` must be readable for `len` bytes.
unsafe fn find_key_len(arg: *const c_char, len: size_t, has_lt: bool) -> c_int {
    // SAFETY: the caller's string.
    unsafe {
        if len >= 4 && *arg as c_int == 't' as c_int && *arg.add(1) as c_int == '_' as c_int {
            // A `t_xx` termcap name, which is two bytes packed into one
            // negative key code.
            if !has_lt || *arg.add(4) as c_int == '>' as c_int {
                return -(*arg.add(2) as uint8_t as c_int
                    + ((*arg.add(3) as uint8_t as c_int) << 8));
            }
            return 0;
        }
        if !has_lt {
            return 0;
        }
        // Back up over the `<` that `has_lt` says was there.
        let mut p = arg.sub(1);
        let mut modifiers = 0;
        let key = find_special_key(
            &raw mut p,
            len.wrapping_add(1),
            &raw mut modifiers,
            FSK_KEYCODE as c_int | FSK_KEEP_X_KEY as c_int | FSK_SIMPLIFY as c_int,
            ptr::null_mut::<bool>(),
        );
        // A key with a modifier left over does not fit in one option value.
        if modifiers != 0 { 0 } else { key }
    }
}

/// The key a 'wildchar'-like option's value names: `<xx>`, `^x`, or the
/// first byte taken literally.
///
/// # Safety
///
/// `arg` must be NUL-terminated.
pub(crate) unsafe fn string_to_key(arg: *mut c_char) -> c_int {
    // SAFETY: the caller's string; the second byte is only read once the
    // first is known not to be the terminator.
    unsafe {
        if *arg as c_int == '<' as c_int && *arg.add(1) != 0 {
            return find_key_len(arg.add(1), strlen(arg), true);
        }
        if *arg as c_int == '^' as c_int && *arg.add(1) != 0 {
            // CTRL-x, where NUL would be ambiguous with "no key".
            let key = ((*arg.add(1) as u8).to_ascii_uppercase() as c_int) ^ 0x40;
            return if key == 0 { K_ZERO } else { key };
        }
        *arg as uint8_t as c_int
    }
}
