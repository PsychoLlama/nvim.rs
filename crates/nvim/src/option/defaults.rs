//! Where an option's default comes from, and the three startup passes that
//! install it.
//!
//! The generated table's `def_val` is a compile-time constant, which is not
//! enough for a default that depends on the machine — `$SHELL`, `$CDPATH`,
//! the state directory, the locale. [`set_init_1`] replaces those in the
//! table itself (through [`change_option_default`], so that a later `:set
//! opt&` sees them too) before installing every default, and the two later
//! passes fix up what could not be known that early: [`set_init_2`] needs
//! the screen size, [`set_init_3`] needs to have looked at 'shell'.
//!
//! This all runs before much of the editor exists, so a mistake here breaks
//! everything identically.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;
use std::ffi::CString;

use crate::api::private::helpers::{cstr_as_string, cstr_to_string};
use crate::buffer::buf_is_empty;
use crate::change::save_file_ff;
use crate::cursor_shape::{SHAPE_CURSOR, parse_shape_opt};
use crate::drawscreen::comp_col;
use crate::garray::{ga_grow, ga_init};
use crate::indent_c::parse_cino;
use crate::log::{LOGLVL_INF, logmsg_c};
use crate::main::{
    Rows, curbuf, current_sctx, curwin, fenc_default, p_ch, p_enc, p_hlg, p_icon, p_rtp, p_sh,
    p_title, p_window,
};
use crate::mapping::langmap_init;
use crate::mbyte::enc_locale;
use crate::memory::{xfree, xmalloc, xmemdupz, xrealloc, xstrdup};
use crate::options::{
    kOptAleph, kOptBackupdir, kOptBackupskip, kOptCdpath, kOptCmdheight, kOptCount, kOptDirectory,
    kOptFileformats, kOptHelplang, kOptIcon, kOptInvalid, kOptModeline, kOptPackpath,
    kOptRuntimepath, kOptScroll, kOptShell, kOptShellpipe, kOptShellredir, kOptTermbidi, kOptTitle,
    kOptTtyfast, kOptUndodir, kOptViewdir, kOptWindow,
};
use crate::optionstr::{check_buf_options, free_string_option};
use crate::os::cshim::{bind_textdomain_codeset, gettext_ptr, snprintf, strncasecmp};
use crate::os::env::{os_env_exists, os_getenv, vim_getenv};
use crate::os::lang::{get_mess_lang, lang_init};
use crate::os::stdpaths::stdpaths_user_state_subpath;
use crate::path::{after_pathsep, invocation_path_tail, path_fnamecmp, vim_ispathlistsep};
use crate::runtime::runtimepath_default;
use crate::spell::init_spell_chartab;
use crate::strings::{vim_snprintf, vim_strchr};
use crate::types::{
    NUL, OptIndex, OptInt, OptVal, OptValData, OptionSetFlags, PATHSEPSTR, String_0, garray_T,
    size_t, uint32_t,
};
use crate::window::{last_status, win_comp_scroll};
use crate::winlayer::{self, Buf};
use ::libc::getuid;

use super::{
    NO_LOCAL_UNDOLEVEL, PROJECT_NAME, ROOT_UID, SID_NONE, boolean_optval, check_options,
    check_win_options, default_fileformat, didset_options, didset_options2, get_option,
    get_option_unset_value, insecure_flag, kOptFlagComma, kOptFlagGettext, kOptFlagNoDefExp,
    kOptFlagNoDefault, kOptValTypeNumber, kOptValTypeString, option_default, option_expand,
    option_has_type, option_is_global_local, option_var, option_was_set, optval_copy, optval_free,
    set_fileformat, set_option_direct, set_option_value_give_err, set_option_varp,
    store_option_default,
};

/// Every option, in table order.
fn all_options() -> impl Iterator<Item = OptIndex> {
    kOptAleph..kOptCount as OptIndex
}

/// A borrowed string default. Only for a literal: nothing frees it.
fn borrowed(value: &'static CStr) -> OptVal {
    OptVal {
        type_0: kOptValTypeString,
        data: OptValData {
            string: String_0::from_raw_parts(
                value.as_ptr() as *mut c_char,
                value.count_bytes() as size_t,
            ),
        },
    }
}

/// An owned string value, taking ownership of `value`.
///
/// # Safety
///
/// `value` must be a NUL-terminated allocation the option module may free.
unsafe fn owned(value: *mut c_char) -> OptVal {
    OptVal {
        type_0: kOptValTypeString,
        // SAFETY: the caller's `value` is NUL-terminated.
        data: OptValData {
            string: unsafe { cstr_as_string(value) },
        },
    }
}

/// The two boolean option values this module installs.
const OFF: OptVal = boolean_optval(Some(false));
const ON: OptVal = boolean_optval(Some(true));

/// A new tab page starts with the global 'cmdheight', not with whatever the
/// tab page it was opened from had.
pub(crate) fn set_init_tablocal() {
    // SAFETY: 'cmdheight' is numeric, so the union holds its number.
    p_ch.set(unsafe { option_default(kOptCmdheight).data.number });
}

/// 'shell' defaults to `$SHELL`. A path with a space in it is quoted, since
/// the value is a command line rather than a file name.
fn set_init_default_shell() {
    // SAFETY: `os_getenv` hands back an owned NUL-terminated string.
    let shell = unsafe { os_getenv(c"SHELL".as_ptr()) };
    if shell.is_null() {
        return;
    }
    if unsafe { vim_strchr(shell, ' ' as c_int) }.is_null() {
        unsafe { set_string_default(kOptShell, shell, false) };
    } else {
        let len = unsafe { cstr::bytes_at(shell) }.len().wrapping_add(3);
        let quoted = unsafe { xmalloc(len) }.cast::<c_char>();
        unsafe { snprintf(quoted, len, c"\"%s\"".as_ptr(), shell) };
        unsafe { set_string_default(kOptShell, quoted, true) };
    }
    unsafe { xfree(shell.cast::<c_void>()) };
}

/// 'backupskip' defaults to a `*` pattern under each temporary directory the
/// environment names, `/tmp` included, with the duplicates dropped.
fn set_init_default_backupskip() {
    // An empty name stands for `/tmp`, which has no environment variable.
    const SOURCES: [&CStr; 4] = [c"", c"TMPDIR", c"TEMP", c"TMP"];

    let mut ga = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ptr::null_mut(),
    };
    // SAFETY: the garray is ours, and every string here is either a literal
    // or an owned result of `vim_getenv`.
    unsafe { ga_init(&raw mut ga, 1, 100) };
    let flags = get_option(kOptBackupskip).flags;

    for name in SOURCES {
        let (dir, mut dirlen, owned) = if name.is_empty() {
            (c"/tmp".as_ptr() as *mut c_char, 4 as size_t, false)
        } else {
            (unsafe { vim_getenv(name.as_ptr()) }, 0, true)
        };
        if !dir.is_null() && unsafe { *dir } != NUL as c_char {
            let mut trailing_sep = false;
            if dirlen == 0 {
                dirlen = unsafe { cstr::bytes_at(dir) }.len();
                trailing_sep = unsafe { after_pathsep(dir, dir.add(dirlen)) } != 0;
            }
            let itemsize = dirlen + usize::from(!trailing_sep) + 2;
            let item = unsafe { xmalloc(itemsize) }.cast::<c_char>();
            let sep = if trailing_sep {
                c"".as_ptr()
            } else {
                PATHSEPSTR.as_ptr()
            };
            let itemlen =
                unsafe { vim_snprintf(item, itemsize, c"%s%s*".as_ptr(), dir, sep) } as size_t;

            if unsafe { find_dup_item(ga.ga_data.cast::<c_char>(), item, itemlen, flags) }.is_null()
            {
                let seplen = size_t::from(ga.ga_len != 0);
                unsafe { ga_grow(&raw mut ga, (seplen + itemlen + 1) as c_int) };
                let comma = if seplen > 0 { c"," } else { c"" };
                let at = unsafe { ga.ga_data.cast::<c_char>().offset(ga.ga_len as isize) };
                let (room, sep) = (seplen + itemlen + 1, comma.as_ptr());
                ga.ga_len += unsafe { vim_snprintf(at, room, c"%s%s".as_ptr(), sep, item) };
            }
            unsafe { xfree(item.cast::<c_void>()) };
        }
        if owned {
            unsafe { xfree(dir.cast::<c_void>()) };
        }
    }

    if !ga.ga_data.is_null() {
        unsafe { set_string_default(kOptBackupskip, ga.ga_data.cast::<c_char>(), true) };
    }
}

/// 'cdpath' defaults to `$CDPATH` with its separators turned into commas,
/// and a leading comma so that `:cd` still tries the current directory
/// first. A space or comma in a component is escaped.
fn set_init_default_cdpath() {
    // SAFETY: `vim_getenv` hands back an owned NUL-terminated string, and
    // the buffer below is two bytes per source byte plus the comma and the
    // terminator, which is the most the escaping can need.
    let cdpath = unsafe { vim_getenv(c"CDPATH".as_ptr()) };
    if cdpath.is_null() {
        return;
    }
    let cdpath_len = unsafe { cstr::bytes_at(cdpath) }.len();
    let buf = unsafe { xmalloc(2usize.wrapping_mul(cdpath_len).wrapping_add(2)) }.cast::<c_char>();
    unsafe { *buf = ',' as c_char };
    let mut at = 1isize;
    let mut src = cdpath;
    while unsafe { *src } != NUL as c_char {
        if vim_ispathlistsep(unsafe { *src } as c_int) {
            unsafe { *buf.offset(at) = ',' as c_char };
            at += 1;
        } else {
            if unsafe { *src } as c_int == ' ' as c_int || unsafe { *src } as c_int == ',' as c_int
            {
                unsafe { *buf.offset(at) = '\\' as c_char };
                at += 1;
            }
            unsafe { *buf.offset(at) = *src };
            at += 1;
        }
        src = unsafe { src.add(1) };
    }
    unsafe { *buf.offset(at) = NUL as c_char };
    change_option_default(kOptCdpath, unsafe { owned(buf) });
    unsafe { xfree(cdpath.cast::<c_void>()) };
}

/// Expand the environment variables in every string default, in the option
/// itself as well as in the default it can be reset to.
fn set_init_expand_env() {
    // SAFETY: the option table is a plain array, and each `var` is the
    // option's own variable.
    for opt_idx in all_options() {
        let opt = get_option(opt_idx);
        if opt.flags & kOptFlagNoDefExp as uint32_t != 0 {
            continue;
        }
        // A `kOptFlagGettext` default is a translatable message rather
        // than a path; there is nothing in it to expand.
        let translated = opt.flags & kOptFlagGettext as uint32_t != 0 && opt.var.has_global();
        let expansion = (!translated).then(|| unsafe { option_expand(opt_idx, ptr::null()) });
        let expanded = match &expansion {
            None => unsafe { gettext_ptr(*option_var(opt_idx).string_var()) }.as_ptr(),
            Some(Some(expanded)) => expanded.as_ptr(),
            Some(None) => continue,
        };
        let value = OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: unsafe { cstr_to_string(expanded) },
            },
        };
        unsafe { set_option_varp(opt_idx, option_var(opt_idx), value, true) };
        change_option_default(
            opt_idx,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: unsafe { cstr_to_string(expanded) },
                },
            },
        );
    }
}

/// The encoding a file with no 'fileencodings' match is read as, taken from
/// the locale.
fn set_init_fenc_default() {
    // SAFETY: both branches produce an owned NUL-terminated string.
    let mut enc = unsafe { enc_locale() };
    if enc.is_null() {
        enc = unsafe { xmemdupz(c"utf-8".as_ptr().cast::<c_void>(), 5) }.cast::<c_char>();
    }
    fenc_default.set(enc);
}

/// The first startup pass: compute every default that depends on the
/// machine, install them all, and run the checks that turn them into the
/// derived state the editor reads.
///
/// `clean_arg` is `--clean`, which keeps the user's directories out of
/// 'runtimepath'.
pub(crate) fn set_init_1(clean_arg: bool) {
    // SAFETY: this runs on the main thread before anything else can touch
    // an option, and every string handed to `set_string_default` below is a
    // fresh allocation it takes ownership of.
    langmap_init();
    alloc_options_default();

    set_init_default_shell();
    set_init_default_backupskip();
    set_init_default_cdpath();

    // 'backupdir' is the state directory's backup subdirectory, with
    // "." in front of it so a backup lands next to the file when it can.
    let subpath = unsafe { stdpaths_user_state_subpath(c"backup".as_ptr(), 2, true) };
    let len = unsafe { cstr::bytes_at(subpath) }.len();
    let backupdir =
        unsafe { xrealloc(subpath.cast::<c_void>(), len.wrapping_add(3)) }.cast::<c_char>();
    let after = unsafe { backupdir.add(2) }.cast::<c_void>();
    let into = after.cast::<u8>();
    unsafe { into.copy_from(backupdir.cast(), len.wrapping_add(1)) };
    let dot = c".,".as_ptr().cast::<c_void>();
    unsafe { backupdir.cast::<u8>().copy_from(dot.cast(), 2) };
    unsafe { set_string_default(kOptBackupdir, backupdir, true) };

    for (opt_idx, name) in [
        (kOptViewdir, c"view"),
        (kOptDirectory, c"swap"),
        (kOptUndodir, c"undo"),
    ] {
        let dir = unsafe { stdpaths_user_state_subpath(name.as_ptr(), 2, true) };
        unsafe { set_string_default(opt_idx, dir, true) };
    }

    let rtp = unsafe { runtimepath_default(clean_arg) };
    if !rtp.is_null() {
        // 'packpath' gets its own copy: `set_string_default` only takes
        // ownership when told to.
        unsafe { set_string_default(kOptRuntimepath, rtp, true) };
        unsafe { set_string_default(kOptPackpath, rtp, false) };
    }

    set_options_default(OptionSetFlags::NONE);

    cur_buf().b_p_initialized = true;
    // The four global-local options the first buffer starts unset.
    cur_buf().b_p_ac = -1;
    cur_buf().b_p_ar = -1;
    cur_buf().b_p_fs = -1;
    cur_buf().b_p_ul = NO_LOCAL_UNDOLEVEL as OptInt;

    unsafe { check_buf_options(curbuf.get()) };
    unsafe { check_win_options(curwin.get()) };
    check_options();
    last_status(false);
    didset_options();
    init_spell_chartab();

    set_init_expand_env();

    if unsafe { os_env_exists(c"NVIM_NOTTYFAST".as_ptr(), false) } {
        set_option_value_give_err(kOptTtyfast, OFF, OptionSetFlags::NONE);
    }
    save_file_ff(unsafe { Buf::current() });
    if unsafe { os_env_exists(c"MLTERM".as_ptr(), false) } {
        set_option_value_give_err(kOptTermbidi, ON, OptionSetFlags::NONE);
    }

    didset_options2();
    lang_init();
    set_init_fenc_default();
    unsafe { bind_textdomain_codeset(PROJECT_NAME.as_ptr(), p_enc.get()) };
    unsafe { set_helplang_default(get_mess_lang()) };
}

/// The default `:set opt&` would install: the table's, with the environment
/// expanded, or the unset value for a `:setlocal` on a global-local option.
///
/// The answer *borrows*: from the option table, or — when the default held
/// an environment variable — from `expansion`, which the caller must keep
/// alive for as long as it reads the value. Upstream borrows the shared
/// `NameBuff` for that case instead.
pub(crate) fn get_option_default(
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
    expansion: &mut Option<CString>,
) -> OptVal {
    // Running as root, 'modeline' defaults off: a modeline is arbitrary
    // code from whoever wrote the file.
    // SAFETY: `getuid` and the option table.
    if opt_idx == kOptModeline && unsafe { getuid() } == ROOT_UID as _ {
        return OFF;
    }
    if opt_flags.has(OptionSetFlags::LOCAL) && option_is_global_local(opt_idx) {
        return get_option_unset_value(opt_idx);
    }
    let default = option_default(opt_idx);
    // SAFETY: the string arm is only read once the type has been tested.
    if !option_has_type(opt_idx, kOptValTypeString)
        || get_option(opt_idx).flags & kOptFlagNoDefExp as uint32_t != 0
    {
        return default;
    }
    match unsafe { option_expand(opt_idx, default.data.string.data()) } {
        None => default,
        // Borrowed: the bytes are `expansion`'s, and nothing frees this.
        Some(e) => unsafe { owned(expansion.insert(e).as_ptr().cast_mut()) },
    }
}

/// Give every default in the table an allocation of its own, so that the
/// computed ones below can free what they replace.
fn alloc_options_default() {
    for opt_idx in all_options() {
        store_option_default(opt_idx, optval_copy(option_default(opt_idx)));
    }
}

/// Replace an option's default, releasing the one it had. Takes ownership.
pub(crate) fn change_option_default(opt_idx: OptIndex, value: OptVal) {
    // Every default is owned by the option after `alloc_options_default`.
    optval_free(option_default(opt_idx));
    store_option_default(opt_idx, value);
}

/// Put an option back to its default, and clear the insecure mark with it —
/// a default cannot have come from a modeline.
fn set_option_default(opt_idx: OptIndex, opt_flags: OptionSetFlags) {
    let both = !opt_flags.has(OptionSetFlags::LOCAL | OptionSetFlags::GLOBAL);
    // `get_option_default`'s answer borrows this when the default expands.
    let mut expansion = None;
    let def_val = get_option_default(opt_idx, opt_flags, &mut expansion);
    // SAFETY: `current_sctx`, `curwin` and the option table are the
    // editor's own.
    set_option_direct(opt_idx, def_val, opt_flags, current_sctx.get().sc_sid);
    // 'scroll' is half the window height, which the option table cannot
    // know.
    if opt_idx == kOptScroll {
        unsafe { win_comp_scroll(curwin.get()) };
    }
    unsafe { insecure_flag(curwin.get(), opt_idx, opt_flags) }.set(false);
    if both {
        unsafe { insecure_flag(curwin.get(), opt_idx, OptionSetFlags::LOCAL) }.set(false);
    }
}

/// Put every option back to its default. `kOptFlagNoDefault` names the ones
/// that have already been given a computed value and must keep it.
pub(crate) fn set_options_default(opt_flags: OptionSetFlags) {
    // SAFETY: the option table and the window lists are the editor's own.
    for opt_idx in all_options() {
        if get_option(opt_idx).flags & kOptFlagNoDefault as uint32_t == 0 {
            set_option_default(opt_idx, opt_flags);
        }
    }
    // 'scroll' again, this time for every window there is.
    for wp in winlayer::tab_windows() {
        unsafe { win_comp_scroll(wp.raw()) };
    }
    unsafe { parse_cino(Buf::current()) };
}

/// Replace a string option's default. With `allocated` the default takes
/// ownership of `val`; without it a copy is made.
///
/// # Safety
///
/// `val` must be NUL-terminated, and an allocation the option module may
/// free when `allocated`.
unsafe fn set_string_default(opt_idx: OptIndex, val: *mut c_char, allocated: bool) {
    debug_assert!(opt_idx != kOptInvalid);
    // SAFETY: the caller's `val` is NUL-terminated.
    let value = unsafe { owned(if allocated { val } else { xstrdup(val) }) };
    change_option_default(opt_idx, value);
}

/// Find `newval` among `origval`'s items, or null.
///
/// For a comma-separated option an item only counts when it starts and ends
/// at a separator. A comma is only a separator when it is not escaped, and
/// the backslash count is what decides that: an even number of backslashes
/// before it, or a single backslash that is itself preceded by a comma.
///
/// # Safety
///
/// `newval` must be readable for `newvallen` bytes and `origval`, if not
/// null, NUL-terminated.
pub(crate) unsafe fn find_dup_item(
    origval: *const c_char,
    newval: *const c_char,
    newvallen: size_t,
    flags: uint32_t,
) -> *const c_char {
    if origval.is_null() {
        return ptr::null();
    }
    let comma_list = flags & kOptFlagComma as uint32_t != 0;
    let mut bs = 0;

    // SAFETY: the caller's strings.
    let mut s = origval;
    while unsafe { *s } != NUL as c_char {
        let starts_item = !comma_list
            || s == origval
            || (unsafe { *s.offset(-1) } as c_int == ',' as c_int && bs & 1 == 0);
        // The tail test must stay behind the comparison: only a match
        // proves `s` has `newvallen` bytes, and so that `s[newvallen]`
        // is at worst the terminator rather than past it.
        let ends_item = || {
            !comma_list
                || unsafe { *s.add(newvallen) } as c_int == ',' as c_int
                || unsafe { *s.add(newvallen) } == NUL as c_char
        };
        if starts_item && unsafe { cstr::prefix_eq(s, newval, newvallen) } && ends_item() {
            return s;
        }
        let escaping = (s > unsafe { origval.add(1) }
            && unsafe { *s.offset(-1) } as c_int == '\\' as c_int
            && unsafe { *s.offset(-2) } as c_int != ',' as c_int)
            || (s == unsafe { origval.add(1) }
                && unsafe { *s.offset(-1) } as c_int == '\\' as c_int);
        bs = if escaping { bs + 1 } else { 0 };
        s = unsafe { s.add(1) };
    }
    ptr::null()
}

/// The second startup pass, once the screen size is known.
pub(crate) fn set_init_2(_headless: bool) {
    // SAFETY: the option table and the screen are the editor's own.
    unsafe {
        logmsg_c!(
            LOGLVL_INF,
            ptr::null(),
            c"set_init_2".as_ptr(),
            613,
            true,
            c"startup runtimepath/packpath value: %s".as_ptr(),
            p_rtp.get(),
        )
    };
    // 'scroll' is half the window height, so it could not be defaulted
    // before there was a window.
    if !option_was_set(kOptScroll) {
        set_option_default(kOptScroll, OptionSetFlags::LOCAL);
    }
    unsafe { comp_col() };
    // Same for 'window', which is one screen's worth of lines.
    if !option_was_set(kOptWindow) {
        p_window.set((Rows.get() - 1) as OptInt);
    }
    change_option_default(
        kOptWindow,
        OptVal {
            type_0: kOptValTypeNumber,
            data: OptValData {
                number: (Rows.get() - 1) as OptInt,
            },
        },
    );
}

/// The third startup pass: the defaults that depend on which shell 'shell'
/// turned out to name.
pub(crate) fn set_init_3() {
    /// The shells whose redirection syntax nvim knows, csh-like first.
    const CSH_LIKE: [&CStr; 2] = [c"csh", c"tcsh"];
    const POSIX_LIKE: [&CStr; 10] = [
        c"sh",
        c"ksh",
        c"mksh",
        c"pdksh",
        c"zsh",
        c"zsh-beta",
        c"bash",
        c"fish",
        c"ash",
        c"dash",
    ];

    // SAFETY: 'shell' is a NUL-terminated option value, and `p` below is an
    // owned copy of its trailing component.
    unsafe { parse_shape_opt(SHAPE_CURSOR) };

    let do_srr = !option_was_set(kOptShellredir);
    let do_sp = !option_was_set(kOptShellpipe);

    let mut len: size_t = 0;
    let tail = unsafe { invocation_path_tail(p_sh.get(), &raw mut len) };
    let shell = unsafe { xmemdupz(tail.cast::<c_void>(), len) }.cast::<c_char>();
    let named = |names: &[&CStr]| {
        names
            .iter()
            .any(|n| unsafe { path_fnamecmp(shell, n.as_ptr()) } == 0)
    };
    let is_csh = named(&CSH_LIKE);

    if is_csh || named(&POSIX_LIKE) {
        if do_sp {
            let sp = borrowed(if is_csh { c"|& tee" } else { c"2>&1| tee" });
            set_option_direct(kOptShellpipe, sp, OptionSetFlags::NONE, SID_NONE);
            change_option_default(kOptShellpipe, optval_copy(sp));
        }
        if do_srr {
            let srr = borrowed(if is_csh { c">&" } else { c">%s 2>&1" });
            set_option_direct(kOptShellredir, srr, OptionSetFlags::NONE, SID_NONE);
            change_option_default(kOptShellredir, optval_copy(srr));
        }
    }
    unsafe { xfree(shell.cast::<c_void>()) };

    // An empty buffer has no line endings to have detected a format
    // from, so it takes the first of 'fileformats' — but only if the
    // user gave that option a value; otherwise its own default stands.
    if unsafe { buf_is_empty(curbuf.get()) } && option_was_set(kOptFileformats) {
        set_fileformat(default_fileformat(), OptionSetFlags::LOCAL);
    }
    set_title_defaults();
}

/// 'helplang' defaults to the message language's two-letter code.
///
/// # Safety
///
/// `lang`, if not null, must be NUL-terminated.
pub(crate) unsafe fn set_helplang_default(lang: *const c_char) {
    if lang.is_null() {
        return;
    }
    // SAFETY: the caller's `lang` is NUL-terminated.
    let lang_len = unsafe { cstr::bytes_at(lang) }.len();
    // Two letters is what the option holds; anything shorter cannot be
    // a language code.
    if lang_len < 2 || option_was_set(kOptHelplang) {
        return;
    }
    unsafe { free_string_option(p_hlg.get()) };
    p_hlg.set(unsafe { xmemdupz(lang.cast::<c_void>(), lang_len) }.cast::<c_char>());

    let hlg = p_hlg.get();
    let lower = |c: c_char| (c as u8).to_ascii_lowercase() as c_char;
    if unsafe { strncasecmp(hlg, c"zh_".as_ptr(), 3) } == 0 && lang_len >= 5 {
        // zh_CN becomes "cn", zh_TW becomes "tw".
        unsafe { *hlg = lower(*hlg.add(3)) };
        unsafe { *hlg.add(1) = lower(*hlg.add(4)) };
    } else if unsafe { *hlg } as c_int == 'C' as c_int {
        // Any C-like setting, C.UTF-8 included, becomes "en".
        unsafe { *hlg = 'e' as c_char };
        unsafe { *hlg.add(1) = 'n' as c_char };
    }
    unsafe { *hlg.add(2) = NUL as c_char };
}

/// 'title' and 'icon' default off unless the user asked for them, so that
/// nvim does not have to ask the terminal what its title was.
pub(crate) fn set_title_defaults() {
    for (opt_idx, cell) in [(kOptTitle, &p_title), (kOptIcon, &p_icon)] {
        if !option_was_set(opt_idx) {
            change_option_default(opt_idx, OFF);
            cell.set(0);
        }
    }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
