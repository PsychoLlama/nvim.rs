//! `:set` with no value, `:mkvimrc`, and the UI's option broadcast.
//!
//! Three ways of rendering an option's value: [`showoneopt`] writes it to
//! the message area, [`put_set`] writes a `:set` command that would restore
//! it to a session file, and [`ui_refresh_options`] hands the ones a UI
//! cares about over the RPC channel.
//!
//! [`option_value_text`] is the shared bottom of the first two. Upstream
//! renders into the shared `NameBuff`, so nothing there could hold a
//! previous rendering across a call to it; here the rendering is the
//! caller's own, cut where that buffer would have cut it.
//!
//! The session writers render each line into bytes and hand them to an
//! [`std::io::Write`]: the bytes are exactly the ones upstream's `fprintf`s
//! wrote, and a write error is a `Failed` where upstream's was a negative
//! count.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use crate::global_cell::field;
use crate::keycodes::{Ctrl_V, ModMask};
use crate::mbyte::unescape_at;
use crate::option::vars::P_MOUSE;
use crate::optionstr::OptString;
use crate::winlayer::Win;
use core::ffi::c_int;
use std::io::Write;

use crate::ascii::ascii_iswhite;
use crate::charset::{transchar, vim_strsize_cstr};
use crate::getchar::state::got_int;
use crate::keycodes::{get_special_key_name, has_key_name};
use crate::memory::XString;
use crate::message::state::info_message;
use crate::message::{
    message_filtered, msg_advance, msg_display, msg_ext_set_kind, msg_putchar, msg_str, msg_title,
};
use crate::mouse::setmouse;
use crate::options::{
    kOptAleph, kOptCount, kOptFiletype, kOptFoldenable, kOptFoldexpr, kOptFoldignore,
    kOptFoldlevel, kOptFoldmarker, kOptFoldmethod, kOptFoldminlines, kOptFoldnestmax, kOptPackpath,
    kOptRuntimepath, kOptSyntax, kOptWildchar, kOptWildcharm,
};
use crate::os::cshim::gettext;
use crate::os::env::home_replace_in;
use crate::os::input::os_breakcheck;
use crate::startup::silent_mode;
use crate::types::{
    Failed, MAXPATHL, NL, OptIndex, OptInt, OptVal, OptionSetFlags, String_0, WinOpt, uint32_t,
};
use crate::ui::state::Columns;
use crate::ui::ui_call_option_set;
use crate::undo::curbuf_is_changed;

use super::{
    Local, OptSlot, WinOptSet, get_option, get_option_unset_value, get_varp, get_varp_scope,
    kOptFlagComma, kOptFlagExpand, kOptFlagNoGlob, kOptFlagNoMkrc, kOptFlagPriMkrc,
    kOptFlagUIOption, kOptValTypeBoolean, kOptValTypeNumber, option_has_type,
    option_is_global_local, option_is_global_only, option_is_window_local, option_parts,
    option_var, optval_as_object, optval_is_default, slot_holds, slot_value,
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
pub(crate) fn showoptions(all: bool, opt_flags: OptionSetFlags) {
    let mut items: Vec<OptIndex> = Vec::with_capacity(kOptCount as usize);

    msg_ext_set_kind(c"list_cmd");
    let title = if opt_flags.has(OptionSetFlags::GLOBAL) {
        c"\n--- Global option values ---"
    } else if opt_flags.has(OptionSetFlags::LOCAL) {
        c"\n--- Local option values ---"
    } else {
        c"\n--- Options ---"
    };
    msg_title(gettext(title));

    for run in 1..=2 {
        if got_int.get() {
            break;
        }
        items.clear();
        for opt_idx in all_options() {
            let opt = get_option(opt_idx);
            if message_filtered(opt.fullname) {
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
            if varp.is_none() || (!all && optval_is_default(opt_idx, varp)) {
                continue;
            }
            // `:set!` gives every option a line of its own.
            let len = if opt_flags.has(OptionSetFlags::ONECOLUMN) {
                Columns.get()
            } else if option_has_type(opt_idx, kOptValTypeBoolean) {
                1
            } else {
                let rendered = option_value_text(opt_idx, opt_flags);
                let fullname_len =
                    c_int::try_from(opt.fullname.count_bytes()).expect("an option name is short");
                fullname_len + vim_strsize_cstr(rendered.as_cstr()) + 1
            };
            let fits = len <= INC - GAP;
            if fits == (run == 1) {
                items.push(opt_idx);
            }
        }

        let count = c_int::try_from(items.len()).expect("one entry per option");
        let rows = if run == 1 {
            let cols = ((Columns.get() + GAP - 3) / INC).max(1);
            (count + cols - 1) / cols
        } else {
            count
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
            let step = usize::try_from(rows).expect("a positive row count");
            let mut i = usize::try_from(row).expect("a row index");
            while i < items.len() {
                msg_advance(col);
                showoneopt(items[i], opt_flags);
                col += INC;
                i += step;
            }
            os_breakcheck();
            row += 1;
        }
    }
}

/// Tell the UI every option it declared an interest in.
pub(crate) fn ui_refresh_options() {
    for opt_idx in all_options() {
        let opt = get_option(opt_idx);
        if opt.flags & kOptFlagUIOption as uint32_t == 0 {
            continue;
        }
        let name = String_0::from_cstr(opt.fullname);
        let value = optval_as_object(slot_value(opt_idx, option_var(opt_idx)));
        ui_call_option_set(name, value);
    }
    // 'mouse' is not a UI option, but the UI has to be told about it
    // all the same.
    if !P_MOUSE.is_uninit() {
        setmouse();
    }
}

/// Write one option to the message area, the way `:set opt?` shows it.
pub(crate) fn showoneopt(opt_idx: OptIndex, opt_flags: OptionSetFlags) {
    // `:set` output is a message even under `-s`, which otherwise
    // suppresses everything.
    let save_silent = silent_mode.get();
    silent_mode.set(false);
    info_message.set(true);

    let opt = get_option(opt_idx);
    let varp = get_varp_scope(opt_idx, opt_flags);
    let boolean = option_has_type(opt_idx, kOptValTypeBoolean);

    // The variable is only read for a boolean option, because for
    // anything else it is not an `int` at all. 'modified' has no
    // variable worth reading either; the undo state decides.
    let word = || varp.boolean_var().get();
    let is_off = || match varp.is_current_modified() {
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
    msg_str(prefix);
    msg_str(opt.fullname);

    if !boolean {
        msg_putchar('=' as c_int);
        let rendered = option_value_text(opt_idx, opt_flags);
        if !rendered.is_empty() {
            msg_display(rendered.as_cstr(), 0, false);
        }
    }

    silent_mode.set(save_silent);
    info_message.set(false);
}

/// Write `bytes`, a write error being a `Failed`.
fn put(out: &mut dyn Write, bytes: &[u8]) -> Result<(), Failed> {
    out.write_all(bytes).map_err(|_| Failed)
}

/// End the line, dropping a write error: upstream's `put_eol(fd) < 0` was
/// never true: `put_eol` answers `OK` or `FAIL`, 1 or 0, and upstream
/// claims success when the file could not be written.
fn put_eol_unchecked(out: &mut dyn Write) {
    let _ = out.write_all(b"\n");
}

/// Write the `:set` commands that would restore the current options to a
/// session or vimrc file.
///
/// Two priority passes: `kOptFlagPriMkrc` options are written first,
/// because the others may depend on them.
pub(crate) fn makeset(
    out: &mut dyn Write,
    opt_flags: OptionSetFlags,
    local_only: c_int,
) -> Result<(), Failed> {
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
            if opt_flags.has(OptionSetFlags::GLOBAL) && optval_is_default(opt_idx, varp) {
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
                    if !optval_is_default(opt_idx, varp_global) {
                        round = 1;
                        varp_local = varp;
                        varp = varp_global;
                    }
                }
            }

            while round <= 2 {
                let cmd: &[u8] = if round == 1 || opt_flags.has(OptionSetFlags::GLOBAL) {
                    b"set"
                } else {
                    b"setlocal"
                };
                // 'syntax' and 'filetype' fire autocommands that would
                // undo the rest of the session, so they are only set
                // when they are not already right.
                let guarded = opt_idx == kOptSyntax || opt_idx == kOptFiletype;
                if guarded {
                    let mut line = b"if &".to_vec();
                    line.extend_from_slice(get_option(opt_idx).fullname.to_bytes());
                    line.extend_from_slice(b" != '");
                    varp.string_var()
                        .with_bytes(|value| line.extend_from_slice(value));
                    line.push(b'\'');
                    put(out, &line)?;
                    put_eol_unchecked(out);
                }
                put_set(out, cmd, opt_idx, varp)?;
                if guarded {
                    put(out, b"endif\n")?;
                }
                varp = varp_local;
                round += 1;
            }
        }
    }
    Ok(())
}

/// Write the current window's fold settings, for `:mksession`.
pub(crate) fn makefoldset(out: &mut dyn Write) -> Result<(), Failed> {
    let win = Win::current();
    let one = WinOptSet::One;
    let fields: [(OptIndex, OptSlot); 8] = [
        (
            kOptFoldmethod,
            Local::Win(win, one, field!(WinOpt, wo_fdm)).into(),
        ),
        (
            kOptFoldexpr,
            Local::Win(win, one, field!(WinOpt, wo_fde)).into(),
        ),
        (
            kOptFoldmarker,
            Local::Win(win, one, field!(WinOpt, wo_fmr)).into(),
        ),
        (
            kOptFoldignore,
            Local::Win(win, one, field!(WinOpt, wo_fdi)).into(),
        ),
        (
            kOptFoldlevel,
            Local::Win(win, one, field!(WinOpt, wo_fdl)).into(),
        ),
        (
            kOptFoldminlines,
            Local::Win(win, one, field!(WinOpt, wo_fml)).into(),
        ),
        (
            kOptFoldnestmax,
            Local::Win(win, one, field!(WinOpt, wo_fdn)).into(),
        ),
        (
            kOptFoldenable,
            Local::Win(win, one, field!(WinOpt, wo_fen)).into(),
        ),
    ];
    for (opt_idx, varp) in fields {
        put_set(out, b"setlocal", opt_idx, varp)?;
    }
    Ok(())
}

/// Write one `<cmd> <option>=<value>` line.
pub(crate) fn put_set(
    out: &mut dyn Write,
    cmd: &[u8],
    opt_idx: OptIndex,
    varp: OptSlot,
) -> Result<(), Failed> {
    let opt = get_option(opt_idx);
    let name = opt.fullname.to_bytes();
    let flags = opt.flags;

    // A global-local option with no local value has nothing to say.
    if option_is_global_local(opt_idx)
        && varp != option_var(opt_idx)
        && slot_holds(opt_idx, varp, &get_option_unset_value(opt_idx))
    {
        return Ok(());
    }

    let mut line = cmd.to_vec();
    line.push(b' ');
    match slot_value(opt_idx, varp) {
        OptVal::Boolean(boolean) => {
            debug_assert!(boolean.is_some());
            if boolean != Some(true) {
                line.extend_from_slice(b"no");
            }
            line.extend_from_slice(name);
            put(out, &line)?;
        }
        OptVal::Number(number) => {
            line.extend_from_slice(name);
            line.push(b'=');
            // 'wildchar' and 'wildcharm' hold a key, which reads back
            // as its name.
            let mut wc: OptInt = 0;
            if wc_use_keyname(opt_idx, varp, &mut wc) {
                let key = c_int::try_from(wc).expect("a key code fits an int");
                let key_name = get_special_key_name(key, ModMask::NONE);
                line.extend_from_slice(
                    cstr::as_bytes(&key_name)
                        .split(|&b| b == 0)
                        .next()
                        .unwrap_or_default(),
                );
            } else {
                line.extend_from_slice(number.to_string().as_bytes());
            }
            put(out, &line)?;
        }
        OptVal::String(value) => {
            line.extend_from_slice(name);
            line.push(b'=');
            put(out, &line)?;
            match put_string_value(out, cmd, name, value.as_bytes(), flags)? {
                // Each `+=` line already carries its terminator.
                Written::Lines => return Ok(()),
                Written::Value => {}
            }
        }
        OptVal::Nil => unreachable!("an option with no type has no value to write"),
    }

    // The newline's write error is dropped, as upstream drops it.
    put_eol_unchecked(out);
    Ok(())
}

/// What [`put_string_value`] wrote.
enum Written {
    /// The value is on the current line, which still needs its terminator.
    Value,
    /// The value went out as a run of `+=` lines, each already terminated.
    Lines,
}

/// Whether a `kOptFlagExpand` value is too long to write in one line, so
/// that it has to go out as a run of `+=` commands.
fn needs_splitting(value: &[u8], flags: uint32_t) -> bool {
    value.len() + 1 >= MAXPATHL as usize
        && flags & kOptFlagComma as uint32_t != 0
        && value.contains(&b',')
}

/// Write a string option's value, with `$HOME` folded back to `~` for the
/// path-like ones, splitting a long comma list into `+=` lines.
fn put_string_value(
    out: &mut dyn Write,
    cmd: &[u8],
    name: &[u8],
    value: &[u8],
    flags: uint32_t,
) -> Result<Written, Failed> {
    if flags & kOptFlagExpand as uint32_t == 0 {
        put(out, &escaped_set_value(value))?;
        return Ok(Written::Value);
    }

    let size = value.len() + 1;
    let replaced = home_replace_in(None, cstr::in_bytes(&[value, b"\0"].concat()), size, false);

    if !needs_splitting(value, flags) {
        put(out, &escaped_set_value(&replaced))?;
        return Ok(Written::Value);
    }

    // Too long for one line: end this one and write an item per `+=`.
    put(out, b"\n")?;
    for part in option_parts(&replaced) {
        let mut line = cmd.to_vec();
        line.push(b' ');
        line.extend_from_slice(name);
        line.extend_from_slice(b"+=");
        put(out, &line)?;
        put(out, &escaped_set_value(&part))?;
        put(out, b"\n")?;
    }
    Ok(Written::Lines)
}

/// `value` escaped for a `:set` command line, as `put_escstr` writes a
/// `:set` value: a multibyte character as itself (a `K_SPECIAL` escape
/// inside it undone), a newline as `\^V^J`, white space, `"` and `\`
/// behind a backslash, and a control character, `|` or a byte that is not
/// a character behind a CTRL-V.
pub(crate) fn escaped_set_value(value: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(value.len());
    let mut i = 0;
    while i < value.len() && value[i] != 0 {
        let c = value[i];
        if c >= 0x80
            && let Some((bytes, used)) = unescape_at(&value[i..])
        {
            out.extend(bytes.iter().copied().take_while(|&b| b != 0));
            i += used;
            continue;
        }
        let code = c_int::from(c);
        if code == NL {
            out.extend_from_slice(b"\\\x16\n");
        } else {
            if ascii_iswhite(code) || c == b'"' || c == b'\\' {
                out.push(b'\\');
            } else if !(b' '..=b'~').contains(&c) || c == b'|' {
                out.push(u8::try_from(Ctrl_V).expect("CTRL-V is a byte"));
            }
            out.push(c);
        }
        i += 1;
    }
    out
}

/// An option's value as `:set opt?` shows it: a key's name for
/// 'wildchar'-like options, `$HOME` folded to `~` for the path-like ones,
/// and cut where upstream's `MAXPATHL` buffer cut it.
pub(crate) fn option_value_text(opt_idx: OptIndex, opt_flags: OptionSetFlags) -> XString {
    let varp = get_varp_scope(opt_idx, opt_flags);
    debug_assert!(!varp.is_none());
    let room = MAXPATHL as usize;

    if option_has_type(opt_idx, kOptValTypeNumber) {
        let mut wc: OptInt = 0;
        let shown = if wc_use_keyname(opt_idx, varp, &mut wc) {
            let key = c_int::try_from(wc).expect("a key code fits an int");
            cstr::in_chars(&get_special_key_name(key, ModMask::NONE))
                .to_bytes()
                .to_vec()
        } else if wc != 0 {
            // A 'wildchar' that is not a named key still shows as the
            // character rather than as its code.
            let key = c_int::try_from(wc).expect("a key code fits an int");
            cstr::in_chars(&transchar(key)).to_bytes().to_vec()
        } else {
            varp.number_var().get().to_string().into_bytes()
        };
        return XString::from_bytes(&shown[..shown.len().min(room - 1)]);
    }

    let value = varp.string_var().get();
    if get_option(opt_idx).flags & kOptFlagExpand as uint32_t != 0 {
        home_replace_in(None, value.as_cstr(), room, false)
    } else {
        XString::from_bytes(&value[..value.len().min(room - 1)])
    }
}

/// Whether the option is 'wildchar' or 'wildcharm' *and* holds a key that
/// has a name; `*wcp` comes back with the value either way for those two.
pub(crate) fn wc_use_keyname(opt_idx: OptIndex, slot: OptSlot, wcp: &mut OptInt) -> bool {
    if !matches!(opt_idx, kOptWildchar | kOptWildcharm) {
        return false;
    }
    // Both options are numeric and global-only, so the slot is the number
    // the table names.
    *wcp = slot.number_var().get();
    // A negative value is a special key code; a positive one may still be a
    // named key such as <Tab>.
    *wcp < 0 || has_key_name(*wcp as c_int)
}
