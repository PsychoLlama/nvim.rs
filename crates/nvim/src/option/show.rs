//! `:set` with no value, `:mkvimrc`, and the UI's option broadcast.
//!
//! Three ways of rendering an option's value: [`showoneopt`] writes it to
//! the message area, [`put_set`] writes a `:set` command that would restore
//! it to a session file, and [`ui_refresh_options`] hands the ones a UI
//! cares about over the RPC channel.
//!
//! [`option_value2string`] is the shared bottom of the first two: it leaves
//! the rendered value in `NameBuff`, which is why nothing here may hold a
//! previous rendering across a call to it.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::api::private::helpers::cstr_as_string;
use crate::charset::{transchar, vim_strsize};
use crate::ex_session::{put_eol, put_line};
use crate::keycodes::{get_special_key_name, has_key_name};
use crate::main::{
    Columns, NameBuff, curbuf, curwin, got_int, info_message, p_mouse, p_pp, p_rtp, p_wc, p_wcm,
    silent_mode,
};
use crate::mapping::{EscTarget, put_escstr};
use crate::memory::{xfree, xmalloc, xstrlcpy};
use crate::message::{
    message_filtered, msg_advance, msg_ext_set_kind, msg_outtrans, msg_putchar, msg_puts,
    msg_puts_title,
};
use crate::mouse::setmouse;
use crate::options::{
    kOptAleph, kOptCount, kOptFiletype, kOptFoldenable, kOptFoldexpr, kOptFoldignore,
    kOptFoldlevel, kOptFoldmarker, kOptFoldmethod, kOptFoldminlines, kOptFoldnestmax, kOptSyntax,
    options,
};
use crate::os::env::home_replace;
use crate::os::input::os_breakcheck;
use crate::os::libc::{fprintf, fputs, gettext, snprintf, strlen};
use crate::strings::vim_strchr;
use crate::types::{
    FILE, OptIndex, OptInt, OptVal, buf_T, kNone, kTrue, size_t, uint32_t, vimoption_T,
};
use crate::ui::ui_call_option_set;
use crate::undo::curbufIsChanged;

use super::{
    FAIL, MAXPATHL, NUL, OK, OPT_GLOBAL, OPT_LOCAL, OPT_ONECOLUMN, OPT_SKIPRTP, copy_option_part,
    get_opt_idx, get_option, get_option_unset_value, get_varp, get_varp_scope, kOptFlagComma,
    kOptFlagExpand, kOptFlagNoGlob, kOptFlagNoMkrc, kOptFlagPriMkrc, kOptFlagUIOption,
    kOptValTypeBoolean, kOptValTypeNumber, kOptValTypeString, option_has_type,
    option_is_global_local, option_is_global_only, option_is_window_local, option_var,
    optval_as_object, optval_equal, optval_from_varp, optval_is_default,
};

/// The column width one option gets in the multi-column listing, and the
/// gap kept between columns.
const INC: c_int = 20;
const GAP: c_int = 3;

/// Every option, in table order.
fn all_options() -> impl Iterator<Item = OptIndex> {
    kOptAleph..kOptCount as OptIndex
}

/// List the options: `all` includes the ones still at their default.
///
/// Two passes, because the short ones are laid out in columns and the long
/// ones get a line each; the first pass collects everything that fits in a
/// column, the second everything that does not.
///
/// # Safety
///
/// The current window and buffer must be live.
pub(crate) unsafe fn showoptions(all: bool, opt_flags: c_int) {
    // SAFETY: the option table, the message area, and the current window
    // and buffer.
    unsafe {
        let mut items: Vec<*mut vimoption_T> = Vec::with_capacity(kOptCount as usize);

        msg_ext_set_kind(c"list_cmd".as_ptr());
        msg_puts_title(gettext(if opt_flags & OPT_GLOBAL != 0 {
            c"\n--- Global option values ---".as_ptr()
        } else if opt_flags & OPT_LOCAL != 0 {
            c"\n--- Local option values ---".as_ptr()
        } else {
            c"\n--- Options ---".as_ptr()
        }));

        for run in 1..=2 {
            if got_int.get() {
                break;
            }
            items.clear();
            for opt_idx in all_options() {
                let opt: *mut vimoption_T = &raw mut (*options.ptr())[opt_idx as usize];
                if message_filtered((*opt).fullname) {
                    continue;
                }
                // An explicit `:setlocal`/`:setglobal` listing skips the
                // options that only exist globally.
                let varp = if opt_flags & (OPT_LOCAL | OPT_GLOBAL) != 0 {
                    if option_is_global_only(opt_idx) {
                        ptr::null_mut()
                    } else {
                        get_varp_scope(opt, opt_flags)
                    }
                } else {
                    get_varp(opt)
                };
                if varp.is_null() || (!all && optval_is_default(opt_idx, varp)) {
                    continue;
                }
                // `:set!` gives every option a line of its own.
                let len = if opt_flags & OPT_ONECOLUMN != 0 {
                    Columns.get()
                } else if option_has_type(opt_idx, kOptValTypeBoolean) {
                    1
                } else {
                    option_value2string(opt, opt_flags);
                    strlen((*opt).fullname) as c_int
                        + vim_strsize(NameBuff.ptr().cast::<c_char>())
                        + 1
                };
                let fits = len <= INC - GAP;
                if fits == (run == 1) {
                    items.push(opt);
                }
            }

            let rows = if run == 1 {
                let cols = ((Columns.get() + GAP - 3) / INC).max(1);
                (items.len() as c_int + cols - 1) / cols
            } else {
                items.len() as c_int
            };

            // Down the first column, then the second: item `i` of row `r`
            // is `r + i * rows`.
            let mut row = 0;
            while row < rows && !got_int.get() {
                msg_putchar('\n' as c_int);
                if got_int.get() {
                    break;
                }
                let mut col = 0;
                let mut i = row as usize;
                while i < items.len() {
                    msg_advance(col);
                    showoneopt(items[i], opt_flags);
                    col += INC;
                    i += rows as usize;
                }
                os_breakcheck();
                row += 1;
            }
        }
    }
}

/// Tell the UI every option it declared an interest in.
pub fn ui_refresh_options() {
    // SAFETY: the option table is a plain array, and each `var` is the
    // option's own global variable.
    unsafe {
        for opt_idx in all_options() {
            let opt = get_option(opt_idx);
            if (*opt).flags & kOptFlagUIOption as uint32_t == 0 {
                continue;
            }
            let name = cstr_as_string((*opt).fullname);
            let value = optval_as_object(optval_from_varp(opt_idx, option_var(opt)));
            ui_call_option_set(name, value);
        }
        // 'mouse' is not a UI option, but the UI has to be told about it
        // all the same.
        if !p_mouse.get().is_null() {
            setmouse();
        }
    }
}

/// Write one option to the message area, the way `:set opt?` shows it.
///
/// # Safety
///
/// `opt` must point into the option table.
pub(crate) unsafe fn showoneopt(opt: *mut vimoption_T, opt_flags: c_int) {
    // `:set` output is a message even under `-s`, which otherwise
    // suppresses everything.
    let save_silent = silent_mode.get();
    silent_mode.set(false);
    info_message.set(true);

    // SAFETY: the caller's table row, and the current buffer.
    unsafe {
        let opt_idx = get_opt_idx(opt);
        let varp = get_varp_scope(opt, opt_flags);
        let boolean = option_has_type(opt_idx, kOptValTypeBoolean);

        // The variable is only read for a boolean option, because for
        // anything else it is not an `int` at all. 'modified' has no
        // variable worth reading either; the undo state decides.
        let is_off = || {
            if varp.cast::<c_int>() == &raw mut (*curbuf.get()).b_changed {
                !curbufIsChanged()
            } else {
                *varp.cast::<c_int>() == 0
            }
        };
        msg_puts(if boolean && is_off() {
            c"no".as_ptr()
        } else if boolean && *varp.cast::<c_int>() < 0 {
            // A global-local boolean with no local value.
            c"--".as_ptr()
        } else {
            c"  ".as_ptr()
        });
        msg_puts((*opt).fullname);

        if !boolean {
            msg_putchar('=' as c_int);
            option_value2string(opt, opt_flags);
            if *NameBuff.ptr().cast::<c_char>() != NUL as c_char {
                msg_outtrans(NameBuff.ptr().cast::<c_char>(), 0, false);
            }
        }
    }

    silent_mode.set(save_silent);
    info_message.set(false);
}

/// Write the `:set` commands that would restore the current options to a
/// session or vimrc file.
///
/// Two priority passes: `kOptFlagPriMkrc` options are written first,
/// because the others may depend on them.
///
/// # Safety
///
/// `fd` must be an open file, and the current window and buffer live.
pub unsafe fn makeset(fd: *mut FILE, opt_flags: c_int, local_only: c_int) -> c_int {
    // SAFETY: the caller's file, and the option table.
    unsafe {
        for priority_pass in [true, false] {
            for opt_idx in all_options() {
                let opt: *mut vimoption_T = &raw mut (*options.ptr())[opt_idx as usize];
                let flags = (*opt).flags;

                if flags & kOptFlagNoMkrc as uint32_t != 0 {
                    continue;
                }
                if priority_pass != (flags & kOptFlagPriMkrc as uint32_t != 0) {
                    continue;
                }
                // A global-only option is written only by `:mkvimrc`, which
                // asks for the global scope.
                if option_is_global_only(opt_idx) && opt_flags & OPT_GLOBAL == 0 {
                    continue;
                }
                if opt_flags & OPT_GLOBAL != 0 && flags & kOptFlagNoGlob as uint32_t != 0 {
                    continue;
                }
                let mut varp = get_varp_scope(opt, opt_flags);
                if varp.is_null() {
                    continue;
                }
                // A global value still at its default needs no command.
                if opt_flags & OPT_GLOBAL != 0 && optval_is_default(opt_idx, varp) {
                    continue;
                }
                // `:mksession` skips the runtime paths, which belong to the
                // installation rather than the session.
                if opt_flags & OPT_SKIPRTP != 0
                    && (option_var(opt) == p_rtp.ptr().cast::<c_void>()
                        || option_var(opt) == p_pp.ptr().cast::<c_void>())
                {
                    continue;
                }

                // A window-local option whose global value is not the
                // default needs two commands: a `:set` for the global one,
                // then a `:setlocal` for this window's.
                let mut varp_local: *mut c_void = ptr::null_mut();
                let mut round = 2;
                if option_is_window_local(opt_idx) {
                    if opt_flags & OPT_LOCAL == 0 {
                        continue;
                    }
                    if opt_flags & OPT_GLOBAL == 0 && local_only == 0 {
                        let varp_global = get_varp_scope(opt, OPT_GLOBAL);
                        if !optval_is_default(opt_idx, varp_global) {
                            round = 1;
                            varp_local = varp;
                            varp = varp_global;
                        }
                    }
                }

                while round <= 2 {
                    let cmd = if round == 1 || opt_flags & OPT_GLOBAL != 0 {
                        c"set".as_ptr() as *mut c_char
                    } else {
                        c"setlocal".as_ptr() as *mut c_char
                    };
                    // 'syntax' and 'filetype' fire autocommands that would
                    // undo the rest of the session, so they are only set
                    // when they are not already right.
                    let guarded = opt_idx == kOptSyntax || opt_idx == kOptFiletype;
                    if guarded {
                        if fprintf(
                            fd,
                            c"if &%s != '%s'".as_ptr(),
                            (*opt).fullname,
                            *varp.cast::<*mut c_char>(),
                        ) < 0
                            || put_eol(fd) < 0
                        {
                            return FAIL;
                        }
                    }
                    if put_set(fd, cmd, opt_idx, varp) == FAIL {
                        return FAIL;
                    }
                    if guarded && put_line(fd, c"endif".as_ptr() as *mut c_char) == FAIL {
                        return FAIL;
                    }
                    varp = varp_local;
                    round += 1;
                }
            }
        }
    }
    OK
}

/// Write the current window's fold settings, for `:mksession`.
///
/// # Safety
///
/// `fd` must be an open file and the current window live.
pub unsafe fn makefoldset(fd: *mut FILE) -> c_int {
    // SAFETY: the caller's file, and `curwin` is live.
    unsafe {
        let wo = &raw mut (*curwin.get()).w_onebuf_opt;
        let fields: [(OptIndex, *mut c_void); 8] = [
            (kOptFoldmethod, (&raw mut (*wo).wo_fdm).cast()),
            (kOptFoldexpr, (&raw mut (*wo).wo_fde).cast()),
            (kOptFoldmarker, (&raw mut (*wo).wo_fmr).cast()),
            (kOptFoldignore, (&raw mut (*wo).wo_fdi).cast()),
            (kOptFoldlevel, (&raw mut (*wo).wo_fdl).cast()),
            (kOptFoldminlines, (&raw mut (*wo).wo_fml).cast()),
            (kOptFoldnestmax, (&raw mut (*wo).wo_fdn).cast()),
            (kOptFoldenable, (&raw mut (*wo).wo_fen).cast()),
        ];
        for (opt_idx, varp) in fields {
            if put_set(fd, c"setlocal".as_ptr() as *mut c_char, opt_idx, varp) == FAIL {
                return FAIL;
            }
        }
    }
    OK
}

/// Write one `<cmd> <option>=<value>` line.
///
/// # Safety
///
/// `fd` must be an open file, `cmd` NUL-terminated, and `varp` the option's
/// variable in some scope.
pub(crate) unsafe fn put_set(
    fd: *mut FILE,
    cmd: *mut c_char,
    opt_idx: OptIndex,
    varp: *mut c_void,
) -> c_int {
    // SAFETY: the caller's file and variable, and the option table.
    unsafe {
        let value: OptVal = optval_from_varp(opt_idx, varp);
        let opt: *mut vimoption_T = &raw mut (*options.ptr())[opt_idx as usize];
        let name = (*opt).fullname;
        let flags = (*opt).flags;

        // A global-local option with no local value has nothing to say.
        if option_is_global_local(opt_idx)
            && varp != option_var(opt)
            && optval_equal(value, get_option_unset_value(opt_idx))
        {
            return OK;
        }

        match value.type_0 {
            kOptValTypeBoolean => {
                debug_assert!(value.data.boolean != kNone);
                let prefix = if value.data.boolean == kTrue {
                    c"".as_ptr()
                } else {
                    c"no".as_ptr()
                };
                if fprintf(fd, c"%s %s%s".as_ptr(), cmd, prefix, name) < 0 {
                    return FAIL;
                }
            }
            kOptValTypeNumber => {
                if fprintf(fd, c"%s %s=".as_ptr(), cmd, name) < 0 {
                    return FAIL;
                }
                // 'wildchar' and 'wildcharm' hold a key, which reads back
                // as its name.
                let mut wc: OptInt = 0;
                if wc_use_keyname(varp, &mut wc) {
                    if fputs(get_special_key_name(wc as c_int, 0), fd) < 0 {
                        return FAIL;
                    }
                } else if fprintf(fd, c"%ld".as_ptr(), value.data.number) < 0 {
                    return FAIL;
                }
            }
            kOptValTypeString => {
                if fprintf(fd, c"%s %s=".as_ptr(), cmd, name) < 0 {
                    return FAIL;
                }
                let value_str = value.data.string.data;
                if !value_str.is_null() {
                    match put_string_value(fd, cmd, name, value_str, flags) {
                        Written::Failed => return FAIL,
                        // Each `+=` line already carries its terminator.
                        Written::Lines => return OK,
                        Written::Value => {}
                    }
                }
            }
            _ => unreachable!("an option with no type has no value to write"),
        }

        if put_eol(fd) < 0 {
            return FAIL;
        }
    }
    OK
}

/// What [`put_string_value`] managed to write.
enum Written {
    /// The value is on the current line, which still needs its terminator.
    Value,
    /// The value went out as a run of `+=` lines, each already terminated.
    Lines,
    Failed,
}

/// Whether a `kOptFlagExpand` value is too long to write in one line, so
/// that it has to go out as a run of `+=` commands.
///
/// # Safety
///
/// `value_str` must be NUL-terminated.
unsafe fn needs_splitting(value_str: *const c_char, flags: uint32_t) -> bool {
    // SAFETY: the caller's string.
    unsafe {
        strlen(value_str).wrapping_add(1) >= MAXPATHL as size_t
            && flags & kOptFlagComma as uint32_t != 0
            && !vim_strchr(value_str, ',' as c_int).is_null()
    }
}

/// Write a string option's value, with `$HOME` folded back to `~` for the
/// path-like ones, splitting a long comma list into `+=` lines.
///
/// # Safety
///
/// `fd` must be an open file and the strings NUL-terminated.
unsafe fn put_string_value(
    fd: *mut FILE,
    cmd: *mut c_char,
    name: *mut c_char,
    value_str: *const c_char,
    flags: uint32_t,
) -> Written {
    // SAFETY: the caller's file and strings.
    unsafe {
        if flags & kOptFlagExpand as uint32_t == 0 {
            return if put_escstr(fd, value_str, EscTarget::SetValue) == FAIL {
                Written::Failed
            } else {
                Written::Value
            };
        }

        let size = strlen(value_str).wrapping_add(1);
        let buf = xmalloc(size).cast::<c_char>();
        home_replace(ptr::null::<buf_T>(), value_str, buf, size, false);

        if !needs_splitting(value_str, flags) {
            let failed = put_escstr(fd, buf, EscTarget::SetValue) == FAIL;
            xfree(buf.cast::<c_void>());
            return if failed {
                Written::Failed
            } else {
                Written::Value
            };
        }

        // Too long for one line: end this one and write an item per `+=`.
        let part = xmalloc(size).cast::<c_char>();
        let mut result = Written::Lines;
        if put_eol(fd) == FAIL {
            result = Written::Failed;
        } else {
            let mut p = buf;
            while *p != NUL as c_char {
                if fprintf(fd, c"%s %s+=".as_ptr(), cmd, name) < 0 {
                    result = Written::Failed;
                    break;
                }
                copy_option_part(&raw mut p, part, size, c",".as_ptr() as *mut c_char);
                if put_escstr(fd, part, EscTarget::SetValue) == FAIL || put_eol(fd) == FAIL {
                    result = Written::Failed;
                    break;
                }
            }
        }
        xfree(buf.cast::<c_void>());
        xfree(part.cast::<c_void>());
        result
    }
}

/// Render an option's value into `NameBuff`.
///
/// # Safety
///
/// `opt` must point into the option table.
pub(crate) unsafe fn option_value2string(opt: *mut vimoption_T, opt_flags: c_int) {
    // SAFETY: the caller's table row, and `NameBuff` is `MAXPATHL` bytes.
    unsafe {
        let varp = get_varp_scope(opt, opt_flags);
        debug_assert!(!varp.is_null());
        let buf = NameBuff.ptr().cast::<c_char>();
        let cap = core::mem::size_of::<[c_char; 4096]>();

        if option_has_type(get_opt_idx(opt), kOptValTypeNumber) {
            let mut wc: OptInt = 0;
            if wc_use_keyname(varp, &mut wc) {
                xstrlcpy(buf, get_special_key_name(wc as c_int, 0), cap);
            } else if wc != 0 {
                // A 'wildchar' that is not a named key still shows as the
                // character rather than as its code.
                xstrlcpy(buf, transchar(wc as c_int), cap);
            } else {
                snprintf(buf, cap, c"%ld".as_ptr(), *varp.cast::<OptInt>());
            }
            return;
        }

        let value = *varp.cast::<*mut c_char>();
        if (*opt).flags & kOptFlagExpand as uint32_t != 0 {
            home_replace(ptr::null::<buf_T>(), value, buf, MAXPATHL as size_t, false);
        } else {
            xstrlcpy(buf, value, MAXPATHL as size_t);
        }
    }
}

/// Whether the variable is 'wildchar' or 'wildcharm' *and* holds a key that
/// has a name; `*wcp` comes back with the value either way for those two.
///
/// # Safety
///
/// `varp` must be an option's variable.
pub(crate) unsafe fn wc_use_keyname(varp: *const c_void, wcp: &mut OptInt) -> bool {
    if varp.cast::<OptInt>() != p_wc.ptr() && varp.cast::<OptInt>() != p_wcm.ptr() {
        return false;
    }
    // SAFETY: the caller's variable, which the test above showed is one of
    // the two `OptInt` cells.
    unsafe {
        *wcp = *varp.cast::<OptInt>();
    }
    // A negative value is a special key code; a positive one may still be a
    // named key such as <Tab>.
    *wcp < 0 || has_key_name(*wcp as c_int)
}
