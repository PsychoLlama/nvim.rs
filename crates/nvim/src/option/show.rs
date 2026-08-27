//! `:set` with no value, `:mkvimrc`, and the UI's option broadcast.
//!
//! Three ways of rendering an option's value: [`showoneopt`] writes it to
//! the message area, [`put_set`] writes a `:set` command that would restore
//! it to a session file, and [`ui_refresh_options`] hands the ones a UI
//! cares about over the RPC channel.
//!
//! [`option_value2string`] is the shared bottom of the first two. Upstream
//! renders into the shared `NameBuff`, so nothing there could hold a
//! previous rendering across a call to it; here the caller passes the
//! buffer it wants filled.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::api::private::helpers::cstr_as_string;
use crate::charset::{transchar, vim_strsize};
use crate::ex_session::{put_eol, put_line};
use crate::keycodes::{get_special_key_name, has_key_name};
use crate::main::{Columns, curbuf, curwin, got_int, info_message, p_mouse, silent_mode};
use crate::mapping::{EscTarget, put_escstr};
use crate::memory::{xfree, xmalloc, xstrlcpy};
use crate::message::{
    message_filtered, msg_advance, msg_ext_set_kind, msg_outtrans, msg_putchar, msg_puts,
    msg_puts_title,
};
use crate::mouse::setmouse;
use crate::options::{
    kOptAleph, kOptCount, kOptFiletype, kOptFoldenable, kOptFoldexpr, kOptFoldignore,
    kOptFoldlevel, kOptFoldmarker, kOptFoldmethod, kOptFoldminlines, kOptFoldnestmax, kOptPackpath,
    kOptRuntimepath, kOptSyntax, kOptWildchar, kOptWildcharm,
};
use crate::os::cshim::{gettext, snprintf};
use crate::os::env::home_replace;
use crate::os::input::os_breakcheck;
use crate::strings::vim_strchr;
use crate::types::{
    FAIL, FILE, MAXPATHL, NUL, OK, OptIndex, OptInt, OptVal, OptionSetFlags, buf_T, size_t,
    uint32_t,
};
use crate::ui::ui_call_option_set;
use crate::undo::curbuf_is_changed;
use ::libc::{fprintf, fputs, strlen};

use super::{
    OptSlot, copy_option_part, get_option, get_option_unset_value, get_varp, get_varp_scope,
    kOptFlagComma, kOptFlagExpand, kOptFlagNoGlob, kOptFlagNoMkrc, kOptFlagPriMkrc,
    kOptFlagUIOption, kOptValTypeBoolean, kOptValTypeNumber, kOptValTypeString, option_has_type,
    option_is_global_local, option_is_global_only, option_is_window_local, option_var,
    optval_as_object, optval_boolean, optval_equal, optval_from_varp, optval_is_default,
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
pub(crate) unsafe fn showoptions(all: bool, opt_flags: OptionSetFlags) {
    let mut rendered = [0 as c_char; MAXPATHL as usize];
    // SAFETY: the option table, the message area, and the current window
    // and buffer.
    let mut items: Vec<OptIndex> = Vec::with_capacity(kOptCount as usize);

    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    let title = if opt_flags.has(OptionSetFlags::GLOBAL) {
        c"\n--- Global option values ---"
    } else if opt_flags.has(OptionSetFlags::LOCAL) {
        c"\n--- Local option values ---"
    } else {
        c"\n--- Options ---"
    };
    unsafe { msg_puts_title(gettext(title.as_ptr())) };

    for run in 1..=2 {
        if got_int.get() {
            break;
        }
        items.clear();
        for opt_idx in all_options() {
            let opt = get_option(opt_idx);
            if unsafe { message_filtered(opt.fullname) } {
                continue;
            }
            // An explicit `:setlocal`/`:setglobal` listing skips the
            // options that only exist globally.
            let varp = if opt_flags.has(OptionSetFlags::LOCAL | OptionSetFlags::GLOBAL) {
                if option_is_global_only(opt_idx) {
                    OptSlot::None
                } else {
                    get_varp_scope(opt_idx, opt_flags)
                }
            } else {
                get_varp(opt_idx)
            };
            if varp.is_none() || (!all && unsafe { optval_is_default(opt_idx, varp) }) {
                continue;
            }
            // `:set!` gives every option a line of its own.
            let len = if opt_flags.has(OptionSetFlags::ONECOLUMN) {
                Columns.get()
            } else if option_has_type(opt_idx, kOptValTypeBoolean) {
                1
            } else {
                unsafe { option_value2string(opt_idx, opt_flags, &mut rendered) };
                unsafe { strlen(opt.fullname) as c_int + vim_strsize(rendered.as_mut_ptr()) + 1 }
            };
            let fits = len <= INC - GAP;
            if fits == (run == 1) {
                items.push(opt_idx);
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
            unsafe { msg_putchar('\n' as c_int) };
            if got_int.get() {
                break;
            }
            let mut col = 0;
            let mut i = row as usize;
            while i < items.len() {
                unsafe { msg_advance(col) };
                unsafe { showoneopt(items[i], opt_flags) };
                col += INC;
                i += rows as usize;
            }
            os_breakcheck();
            row += 1;
        }
    }
}

/// Tell the UI every option it declared an interest in.
pub(crate) fn ui_refresh_options() {
    // SAFETY: the option table is a plain array, and each `var` is the
    // option's own global variable.
    for opt_idx in all_options() {
        let opt = get_option(opt_idx);
        if opt.flags & kOptFlagUIOption as uint32_t == 0 {
            continue;
        }
        let name = unsafe { cstr_as_string(opt.fullname) };
        let value = optval_as_object(unsafe { optval_from_varp(opt_idx, option_var(opt_idx)) });
        ui_call_option_set(name, value);
    }
    // 'mouse' is not a UI option, but the UI has to be told about it
    // all the same.
    if !p_mouse.get().is_null() {
        setmouse();
    }
}

/// Write one option to the message area, the way `:set opt?` shows it.
///
/// # Safety
///
/// The current window and buffer must be live.
pub(crate) unsafe fn showoneopt(opt_idx: OptIndex, opt_flags: OptionSetFlags) {
    // `:set` output is a message even under `-s`, which otherwise
    // suppresses everything.
    let save_silent = silent_mode.get();
    silent_mode.set(false);
    info_message.set(true);
    let mut rendered = [0 as c_char; MAXPATHL as usize];

    // SAFETY: the current window and buffer.
    let opt = get_option(opt_idx);
    let varp = get_varp_scope(opt_idx, opt_flags);
    let boolean = option_has_type(opt_idx, kOptValTypeBoolean);

    // The variable is only read for a boolean option, because for
    // anything else it is not an `int` at all. 'modified' has no
    // variable worth reading either; the undo state decides.
    let word = || unsafe { *varp.boolean_var() };
    let is_off = || match varp == OptSlot::Boolean(unsafe { &raw mut (*curbuf.get()).b_changed }) {
        true => !curbuf_is_changed(),
        false => word() == 0,
    };
    let prefix = if boolean && is_off() {
        c"no"
    } else if boolean && word() < 0 {
        // A global-local boolean with no local value.
        c"--"
    } else {
        c"  "
    };
    unsafe { msg_puts(prefix.as_ptr()) };
    unsafe { msg_puts(opt.fullname) };

    if !boolean {
        unsafe { msg_putchar('=' as c_int) };
        unsafe { option_value2string(opt_idx, opt_flags, &mut rendered) };
        if rendered[0] != NUL as c_char {
            unsafe { msg_outtrans(rendered.as_mut_ptr(), 0, false) };
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
pub(crate) unsafe fn makeset(fd: *mut FILE, opt_flags: OptionSetFlags, local_only: c_int) -> c_int {
    // SAFETY: the caller's file, and the option table.
    for priority_pass in [true, false] {
        for opt_idx in all_options() {
            let flags = get_option(opt_idx).flags;

            if flags & kOptFlagNoMkrc as uint32_t != 0 {
                continue;
            }
            if priority_pass != (flags & kOptFlagPriMkrc as uint32_t != 0) {
                continue;
            }
            // A global-only option is written only by `:mkvimrc`, which
            // asks for the global scope.
            if option_is_global_only(opt_idx) && !opt_flags.has(OptionSetFlags::GLOBAL) {
                continue;
            }
            if opt_flags.has(OptionSetFlags::GLOBAL) && flags & kOptFlagNoGlob as uint32_t != 0 {
                continue;
            }
            let mut varp = get_varp_scope(opt_idx, opt_flags);
            if varp.is_none() {
                continue;
            }
            // A global value still at its default needs no command.
            if opt_flags.has(OptionSetFlags::GLOBAL) && unsafe { optval_is_default(opt_idx, varp) }
            {
                continue;
            }
            // `:mksession` skips the runtime paths, which belong to the
            // installation rather than the session.
            if opt_flags.has(OptionSetFlags::SKIPRTP)
                && matches!(opt_idx, kOptRuntimepath | kOptPackpath)
            {
                continue;
            }

            // A window-local option whose global value is not the
            // default needs two commands: a `:set` for the global one,
            // then a `:setlocal` for this window's.
            let mut varp_local = OptSlot::None;
            let mut round = 2;
            if option_is_window_local(opt_idx) {
                if !opt_flags.has(OptionSetFlags::LOCAL) {
                    continue;
                }
                if !opt_flags.has(OptionSetFlags::GLOBAL) && local_only == 0 {
                    let varp_global = get_varp_scope(opt_idx, OptionSetFlags::GLOBAL);
                    if !unsafe { optval_is_default(opt_idx, varp_global) } {
                        round = 1;
                        varp_local = varp;
                        varp = varp_global;
                    }
                }
            }

            while round <= 2 {
                let cmd = if round == 1 || opt_flags.has(OptionSetFlags::GLOBAL) {
                    c"set".as_ptr() as *mut c_char
                } else {
                    c"setlocal".as_ptr() as *mut c_char
                };
                // 'syntax' and 'filetype' fire autocommands that would
                // undo the rest of the session, so they are only set
                // when they are not already right.
                let guarded = opt_idx == kOptSyntax || opt_idx == kOptFiletype;
                let (guard, name) = (c"if &%s != '%s'".as_ptr(), get_option(opt_idx).fullname);
                if guarded
                    && (unsafe { fprintf(fd, guard, name, *varp.string_var()) } < 0
                        || unsafe { put_eol(fd) } < 0)
                {
                    return FAIL;
                }
                if unsafe { put_set(fd, cmd, opt_idx, varp) } == FAIL {
                    return FAIL;
                }
                if guarded && unsafe { put_line(fd, c"endif".as_ptr() as *mut c_char) } == FAIL {
                    return FAIL;
                }
                varp = varp_local;
                round += 1;
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
pub(crate) unsafe fn makefoldset(fd: *mut FILE) -> c_int {
    // SAFETY: the caller's file, and `curwin` is live.
    let wo = unsafe { &raw mut (*curwin.get()).w_onebuf_opt };
    let fields: [(OptIndex, OptSlot); 8] = [
        (
            kOptFoldmethod,
            OptSlot::from(unsafe { &raw mut (*wo).wo_fdm }),
        ),
        (
            kOptFoldexpr,
            OptSlot::from(unsafe { &raw mut (*wo).wo_fde }),
        ),
        (
            kOptFoldmarker,
            OptSlot::from(unsafe { &raw mut (*wo).wo_fmr }),
        ),
        (
            kOptFoldignore,
            OptSlot::from(unsafe { &raw mut (*wo).wo_fdi }),
        ),
        (
            kOptFoldlevel,
            OptSlot::from(unsafe { &raw mut (*wo).wo_fdl }),
        ),
        (
            kOptFoldminlines,
            OptSlot::from(unsafe { &raw mut (*wo).wo_fml }),
        ),
        (
            kOptFoldnestmax,
            OptSlot::from(unsafe { &raw mut (*wo).wo_fdn }),
        ),
        (
            kOptFoldenable,
            OptSlot::from(unsafe { &raw mut (*wo).wo_fen }),
        ),
    ];
    for (opt_idx, varp) in fields {
        if unsafe { put_set(fd, c"setlocal".as_ptr() as *mut c_char, opt_idx, varp) } == FAIL {
            return FAIL;
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
    varp: OptSlot,
) -> c_int {
    // SAFETY: the caller's file and variable, and the option table.
    let value: OptVal = unsafe { optval_from_varp(opt_idx, varp) };
    let opt = get_option(opt_idx);
    let name = opt.fullname;
    let flags = opt.flags;

    // A global-local option with no local value has nothing to say.
    if option_is_global_local(opt_idx)
        && varp != option_var(opt_idx)
        && optval_equal(value, get_option_unset_value(opt_idx))
    {
        return OK;
    }

    match value.type_0 {
        kOptValTypeBoolean => {
            debug_assert!(unsafe { optval_boolean(value.data) }.is_some());
            let prefix = if unsafe { optval_boolean(value.data) } == Some(true) {
                c"".as_ptr()
            } else {
                c"no".as_ptr()
            };
            if unsafe { fprintf(fd, c"%s %s%s".as_ptr(), cmd, prefix, name) } < 0 {
                return FAIL;
            }
        }
        kOptValTypeNumber => {
            if unsafe { fprintf(fd, c"%s %s=".as_ptr(), cmd, name) } < 0 {
                return FAIL;
            }
            // 'wildchar' and 'wildcharm' hold a key, which reads back
            // as its name.
            let mut wc: OptInt = 0;
            if unsafe { wc_use_keyname(opt_idx, varp, &mut wc) } {
                if unsafe { fputs(get_special_key_name(wc as c_int, 0).as_ptr(), fd) } < 0 {
                    return FAIL;
                }
            } else if unsafe { fprintf(fd, c"%ld".as_ptr(), value.data.number) } < 0 {
                return FAIL;
            }
        }
        kOptValTypeString => {
            if unsafe { fprintf(fd, c"%s %s=".as_ptr(), cmd, name) } < 0 {
                return FAIL;
            }
            let value_str = unsafe { value.data.string }.data();
            if !value_str.is_null() {
                match unsafe { put_string_value(fd, cmd, name, value_str, flags) } {
                    Written::Failed => return FAIL,
                    // Each `+=` line already carries its terminator.
                    Written::Lines => return OK,
                    Written::Value => {}
                }
            }
        }
        _ => unreachable!("an option with no type has no value to write"),
    }

    if unsafe { put_eol(fd) } < 0 {
        return FAIL;
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
    unsafe { strlen(value_str) }.wrapping_add(1) >= MAXPATHL as size_t
        && flags & kOptFlagComma as uint32_t != 0
        && !unsafe { vim_strchr(value_str, ',' as c_int) }.is_null()
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
    if flags & kOptFlagExpand as uint32_t == 0 {
        return if unsafe { put_escstr(fd, value_str, EscTarget::SetValue) } == FAIL {
            Written::Failed
        } else {
            Written::Value
        };
    }

    let size = unsafe { strlen(value_str) }.wrapping_add(1);
    let buf = unsafe { xmalloc(size) }.cast::<c_char>();
    unsafe { home_replace(ptr::null::<buf_T>(), value_str, buf, size, false) };

    if !unsafe { needs_splitting(value_str, flags) } {
        let failed = unsafe { put_escstr(fd, buf, EscTarget::SetValue) } == FAIL;
        unsafe { xfree(buf.cast::<c_void>()) };
        return if failed {
            Written::Failed
        } else {
            Written::Value
        };
    }

    // Too long for one line: end this one and write an item per `+=`.
    let part = unsafe { xmalloc(size) }.cast::<c_char>();
    let mut result = Written::Lines;
    if unsafe { put_eol(fd) } == FAIL {
        result = Written::Failed;
    } else {
        let mut p = buf;
        while unsafe { *p } != NUL as c_char {
            if unsafe { fprintf(fd, c"%s %s+=".as_ptr(), cmd, name) } < 0 {
                result = Written::Failed;
                break;
            }
            unsafe { copy_option_part(&raw mut p, part, size, c",".as_ptr() as *mut c_char) };
            if unsafe { put_escstr(fd, part, EscTarget::SetValue) } == FAIL
                || unsafe { put_eol(fd) } == FAIL
            {
                result = Written::Failed;
                break;
            }
        }
    }
    unsafe { xfree(buf.cast::<c_void>()) };
    unsafe { xfree(part.cast::<c_void>()) };
    result
}

/// Render an option's value into `out`.
///
/// # Safety
///
/// The current window and buffer must be live.
pub(crate) unsafe fn option_value2string(
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
    out: &mut [c_char; MAXPATHL as usize],
) {
    // SAFETY: the current window and buffer, and `out` is `MAXPATHL` bytes.
    let varp = get_varp_scope(opt_idx, opt_flags);
    debug_assert!(!varp.is_none());
    let buf = out.as_mut_ptr();
    let cap = out.len();

    if option_has_type(opt_idx, kOptValTypeNumber) {
        let mut wc: OptInt = 0;
        if unsafe { wc_use_keyname(opt_idx, varp, &mut wc) } {
            unsafe { xstrlcpy(buf, get_special_key_name(wc as c_int, 0).as_ptr(), cap) };
        } else if wc != 0 {
            // A 'wildchar' that is not a named key still shows as the
            // character rather than as its code.
            unsafe { xstrlcpy(buf, transchar(wc as c_int).as_ptr(), cap) };
        } else {
            unsafe { snprintf(buf, cap, c"%ld".as_ptr(), *varp.number_var()) };
        }
        return;
    }

    let value = unsafe { *varp.string_var() };
    if get_option(opt_idx).flags & kOptFlagExpand as uint32_t != 0 {
        unsafe { home_replace(ptr::null::<buf_T>(), value, buf, MAXPATHL as size_t, false) };
    } else {
        unsafe { xstrlcpy(buf, value, MAXPATHL as size_t) };
    }
}

/// Whether the option is 'wildchar' or 'wildcharm' *and* holds a key that
/// has a name; `*wcp` comes back with the value either way for those two.
///
/// # Safety
///
/// `slot` must be the option's variable.
pub(crate) unsafe fn wc_use_keyname(opt_idx: OptIndex, slot: OptSlot, wcp: &mut OptInt) -> bool {
    if !matches!(opt_idx, kOptWildchar | kOptWildcharm) {
        return false;
    }
    // SAFETY: both options are numeric and global-only, so the slot is the
    // `OptInt` cell the table names.
    *wcp = unsafe { *slot.number_var() };
    // A negative value is a special key code; a positive one may still be a
    // named key such as <Tab>.
    *wcp < 0 || has_key_name(*wcp as c_int)
}
