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
//! A default the table declares is a literal, and stays one until something
//! computes another: nothing is copied for an option whose default is left
//! alone, and a computed one is owned by the option's state.
//!
//! This all runs before much of the editor exists, so a mistake here breaks
//! everything identically.

#![forbid(unsafe_code)]

use crate::memory::XString;
use crate::optionstr::OptString;
use crate::winlayer::Win;
use core::ffi::CStr;

use crate::buffer::buf_is_empty;
use crate::change::save_file_ff;
use crate::cursor_shape::{SHAPE_CURSOR, parse_shape_opt};
use crate::drawscreen::comp_col;
use crate::indent_c::parse_cino_for;
use crate::log::{LOGLVL_INF, logmsg};
use crate::mapping::langmap_init;
use crate::mbyte::enc_locale;
use crate::message_fmt::msg_cstr;
use crate::option::vars::{P_CH, P_HLG, P_ICON, P_TITLE, P_WINDOW, fenc_default};
use crate::option::vars::{mark_option_defaults_set, p_enc, p_rtp, p_sh};
use crate::options::{
    kOptAleph, kOptBackupdir, kOptBackupskip, kOptCdpath, kOptCmdheight, kOptCount, kOptDirectory,
    kOptFileformats, kOptHelplang, kOptIcon, kOptModeline, kOptPackpath, kOptRuntimepath,
    kOptScroll, kOptShell, kOptShellpipe, kOptShellredir, kOptTermbidi, kOptTitle, kOptTtyfast,
    kOptUndodir, kOptViewdir, kOptWindow,
};
use crate::optionstr::check_buf_options;
use crate::os::cshim::{bind_textdomain_codeset_of, gettext_owned};
use crate::os::env::{env_exists, getenv_owned, vim_getenv_owned};
use crate::os::lang::{lang_init, mess_lang_of};
use crate::os::stdpaths::stdpaths_user_state_subpath_of;
use crate::os::users::os_getuid;
use crate::path::{invocation_path_tail, path_fnamecmp, vim_ispathlistsep};
use crate::runtime::runtimepath_default;
use crate::runtime::state::current_sctx;
use crate::spell::init_spell_chartab;
use crate::types::{OptIndex, OptInt, OptVal, OptionSetFlags, PATHSEPSTR, uint32_t};
use crate::ui::state::Rows;
use crate::window::{last_status, win_comp_scroll};
use crate::winlayer::{self, Buf};

use super::{
    NO_LOCAL_UNDOLEVEL, PROJECT_NAME, ROOT_UID, SID_NONE, boolean_optval, default_fileformat,
    didset_options, didset_options2, find_dup_item, get_option, get_option_unset_value,
    insecure_flag, kOptFlagGettext, kOptFlagNoDefExp, kOptFlagNoDefault, option_default,
    option_expand, option_is_global_local, option_var, option_was_set, replace_option_default,
    set_fileformat, set_option_direct, set_option_value_give_err, write_slot,
};

/// Every option, in table order.
fn all_options() -> impl Iterator<Item = OptIndex> {
    kOptAleph..kOptCount as OptIndex
}

/// A borrowed string default. Only for a literal.
const fn borrowed(value: &'static CStr) -> OptVal {
    OptVal::static_string(value)
}

/// The two boolean option values this module installs.
const OFF: OptVal = boolean_optval(Some(false));
const ON: OptVal = boolean_optval(Some(true));

/// A new tab page starts with the global 'cmdheight', not with whatever the
/// tab page it was opened from had.
pub(crate) fn set_init_tablocal() {
    P_CH.set(
        option_default(kOptCmdheight)
            .as_number()
            .expect("'cmdheight' is a number option"),
    );
}

/// 'shell' defaults to `$SHELL`. A path with a space in it is quoted, since
/// the value is a command line rather than a file name.
fn set_init_default_shell() {
    let Some(shell) = getenv_owned(c"SHELL") else {
        return;
    };
    let value = if shell.contains(&b' ') {
        XString::from_vec([b"\"", &shell[..], b"\""].concat())
    } else {
        shell
    };
    change_option_default(kOptShell, OptVal::from_xstring(value));
}

/// 'backupskip' defaults to a `*` pattern under each temporary directory the
/// environment names, `/tmp` included, with the duplicates dropped.
fn set_init_default_backupskip() {
    // An empty name stands for `/tmp`, which has no environment variable.
    const SOURCES: [&CStr; 4] = [c"", c"TMPDIR", c"TEMP", c"TMP"];

    // The value built so far; nothing until the first directory is found.
    let mut value: Option<Vec<u8>> = None;
    let flags = get_option(kOptBackupskip).flags;

    for name in SOURCES {
        let dir = if name.is_empty() {
            Some(XString::from_cstr(c"/tmp"))
        } else {
            vim_getenv_owned(name)
        };
        let Some(dir) = dir.filter(|dir| !dir.is_empty()) else {
            continue;
        };
        // A separator is only added where the directory does not end in
        // one; `/tmp` is known not to.
        let trailing_sep = !name.is_empty() && dir.last() == Some(&b'/');
        let sep: &[u8] = if trailing_sep {
            b""
        } else {
            PATHSEPSTR.to_bytes()
        };
        let item = [&dir[..], sep, b"*"].concat();
        let so_far = value.as_deref().unwrap_or_default();
        if value.is_none() || find_dup_item(so_far, &item, flags).is_none() {
            let value = value.get_or_insert_with(Vec::new);
            if !value.is_empty() {
                value.push(b',');
            }
            value.extend_from_slice(&item);
        }
    }

    if let Some(value) = value {
        change_option_default(
            kOptBackupskip,
            OptVal::from_xstring(XString::from_vec(value)),
        );
    }
}

/// 'cdpath' defaults to `$CDPATH` with its separators turned into commas,
/// and a leading comma so that `:cd` still tries the current directory
/// first. A space or comma in a component is escaped.
fn set_init_default_cdpath() {
    let Some(cdpath) = vim_getenv_owned(c"CDPATH") else {
        return;
    };
    let mut value = Vec::with_capacity(2 * cdpath.len() + 1);
    value.push(b',');
    for &byte in cdpath.iter() {
        if vim_ispathlistsep(byte.into()) {
            value.push(b',');
        } else {
            if byte == b' ' || byte == b',' {
                value.push(b'\\');
            }
            value.push(byte);
        }
    }
    change_option_default(kOptCdpath, OptVal::from_xstring(XString::from_vec(value)));
}

/// Expand the environment variables in every string default, in the option
/// itself as well as in the default it can be reset to.
fn set_init_expand_env() {
    for opt_idx in all_options() {
        let opt = get_option(opt_idx);
        if opt.flags & kOptFlagNoDefExp as uint32_t != 0 {
            continue;
        }
        // A `kOptFlagGettext` default is a translatable message rather
        // than a path; there is nothing in it to expand.
        let translated = opt.flags & kOptFlagGettext as uint32_t != 0 && opt.var.has_global();
        let value = if translated {
            let own = option_var(opt_idx).string_var().get();
            XString::from_cstr(&gettext_owned(own.as_cstr()))
        } else {
            match option_expand(opt_idx, None) {
                Some(expanded) => expanded,
                None => continue,
            }
        };
        // The variable takes one copy and the default the other.
        write_slot(
            opt_idx,
            option_var(opt_idx),
            OptVal::from_xstring(value.clone()),
        );
        change_option_default(opt_idx, OptVal::from_xstring(value));
    }
}

/// The encoding a file with no 'fileencodings' match is read as, taken from
/// the locale.
fn set_init_fenc_default() {
    let enc = enc_locale().unwrap_or_else(|| XString::from("utf-8"));
    fenc_default.set(Some(enc));
}

/// The first startup pass: compute every default that depends on the
/// machine, install them all, and run the checks that turn them into the
/// derived state the editor reads.
///
/// `clean_arg` is `--clean`, which keeps the user's directories out of
/// 'runtimepath'.
pub(crate) fn set_init_1(clean_arg: bool) {
    langmap_init();

    set_init_default_shell();
    set_init_default_backupskip();
    set_init_default_cdpath();

    // 'backupdir' is the state directory's backup subdirectory, with
    // "." in front of it so a backup lands next to the file when it can.
    let subpath = stdpaths_user_state_subpath_of(c"backup", 2, true);
    let backupdir = XString::from_vec([&b".,"[..], &subpath[..]].concat());
    change_option_default(kOptBackupdir, OptVal::from_xstring(backupdir));

    for (opt_idx, name) in [
        (kOptViewdir, c"view"),
        (kOptDirectory, c"swap"),
        (kOptUndodir, c"undo"),
    ] {
        let dir = stdpaths_user_state_subpath_of(name, 2, true);
        change_option_default(opt_idx, OptVal::from_xstring(dir));
    }

    if let Some(rtp) = runtimepath_default(clean_arg) {
        // 'packpath' gets its own copy.
        change_option_default(kOptPackpath, OptVal::from_xstring(rtp.clone()));
        change_option_default(kOptRuntimepath, OptVal::from_xstring(rtp));
    }

    set_options_default(OptionSetFlags::NONE);
    // From here no string option is upstream's NULL.
    mark_option_defaults_set();

    Buf::current().b_p_initialized = true;
    // The four global-local options the first buffer starts unset.
    Buf::current().b_p_ac = -1;
    Buf::current().b_p_ar = -1;
    Buf::current().b_p_fs = -1;
    Buf::current().b_p_ul = NO_LOCAL_UNDOLEVEL as OptInt;

    check_buf_options(Buf::current());
    last_status(false);
    didset_options();
    init_spell_chartab();

    set_init_expand_env();

    if env_exists(c"NVIM_NOTTYFAST", false) {
        set_option_value_give_err(kOptTtyfast, OFF, OptionSetFlags::NONE);
    }
    save_file_ff(Buf::current());
    if env_exists(c"MLTERM", false) {
        set_option_value_give_err(kOptTermbidi, ON, OptionSetFlags::NONE);
    }

    didset_options2();
    lang_init();
    set_init_fenc_default();
    p_enc(|value| bind_textdomain_codeset_of(PROJECT_NAME, value));
    if let Some(lang) = mess_lang_of() {
        set_helplang_default(&lang);
    }
}

/// The default `:set opt&` would install: the table's, with the environment
/// expanded, or the unset value for a `:setlocal` on a global-local option.
pub(crate) fn get_option_default(opt_idx: OptIndex, opt_flags: OptionSetFlags) -> OptVal {
    // Running as root, 'modeline' defaults off: a modeline is arbitrary
    // code from whoever wrote the file.
    if opt_idx == kOptModeline && os_getuid() == ROOT_UID as _ {
        return OFF;
    }
    if opt_flags.has(OptionSetFlags::LOCAL) && option_is_global_local(opt_idx) {
        return get_option_unset_value(opt_idx);
    }
    let default = option_default(opt_idx);
    let Some(string) = default.as_string() else {
        return default;
    };
    if get_option(opt_idx).flags & kOptFlagNoDefExp as uint32_t != 0 {
        return default;
    }
    match option_expand(opt_idx, Some(string.as_cstr())) {
        None => default,
        Some(expanded) => OptVal::from_xstring(expanded),
    }
}

/// Replace an option's default, releasing the one it had. Takes ownership.
pub(crate) fn change_option_default(opt_idx: OptIndex, value: OptVal) {
    drop(replace_option_default(opt_idx, value));
}

/// Put an option back to its default, and clear the insecure mark with it —
/// a default cannot have come from a modeline.
fn set_option_default(opt_idx: OptIndex, opt_flags: OptionSetFlags) {
    let both = !opt_flags.has(OptionSetFlags::LOCAL | OptionSetFlags::GLOBAL);
    let def_val = get_option_default(opt_idx, opt_flags);
    set_option_direct(opt_idx, def_val, opt_flags, current_sctx.get().sc_sid);
    // 'scroll' is half the window height, which the option table cannot
    // know.
    if opt_idx == kOptScroll {
        win_comp_scroll(Win::current());
    }
    insecure_flag(Win::current_or_none(), opt_idx, opt_flags).set(false);
    if both {
        insecure_flag(Win::current_or_none(), opt_idx, OptionSetFlags::LOCAL).set(false);
    }
}

/// Put every option back to its default. `kOptFlagNoDefault` names the ones
/// that have already been given a computed value and must keep it.
pub(crate) fn set_options_default(opt_flags: OptionSetFlags) {
    for opt_idx in all_options() {
        if get_option(opt_idx).flags & kOptFlagNoDefault as uint32_t == 0 {
            set_option_default(opt_idx, opt_flags);
        }
    }
    // 'scroll' again, this time for every window there is.
    for wp in winlayer::tab_windows() {
        win_comp_scroll(wp);
    }
    parse_cino_for(Buf::current());
}

/// The second startup pass, once the screen size is known.
pub(crate) fn set_init_2(_headless: bool) {
    logmsg!(
        LOGLVL_INF,
        c"set_init_2",
        613,
        "startup runtimepath/packpath value: {}",
        msg_cstr(&p_rtp(CStr::to_owned))
    );
    // 'scroll' is half the window height, so it could not be defaulted
    // before there was a window.
    if !option_was_set(kOptScroll) {
        set_option_default(kOptScroll, OptionSetFlags::LOCAL);
    }
    comp_col();
    // Same for 'window', which is one screen's worth of lines.
    if !option_was_set(kOptWindow) {
        P_WINDOW.set((Rows.get() - 1) as OptInt);
    }
    change_option_default(kOptWindow, OptVal::Number((Rows.get() - 1) as OptInt));
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

    let _ = parse_shape_opt(SHAPE_CURSOR);

    let do_srr = !option_was_set(kOptShellredir);
    let do_sp = !option_was_set(kOptShellpipe);

    // The program's own name: its tail, without the arguments.
    let shell = p_sh(|value| {
        let tail = invocation_path_tail(value).to_bytes();
        let name = tail.split(|&b| b == b' ').next().unwrap_or_default();
        XString::from_bytes(name)
    });
    let named = |names: &[&CStr]| {
        names
            .iter()
            .any(|n| path_fnamecmp(&shell, n.to_bytes()) == 0)
    };
    let is_csh = named(&CSH_LIKE);

    if is_csh || named(&POSIX_LIKE) {
        if do_sp {
            let sp = if is_csh { c"|& tee" } else { c"2>&1| tee" };
            set_option_direct(kOptShellpipe, borrowed(sp), OptionSetFlags::NONE, SID_NONE);
            change_option_default(kOptShellpipe, borrowed(sp));
        }
        if do_srr {
            let srr = if is_csh { c">&" } else { c">%s 2>&1" };
            set_option_direct(
                kOptShellredir,
                borrowed(srr),
                OptionSetFlags::NONE,
                SID_NONE,
            );
            change_option_default(kOptShellredir, borrowed(srr));
        }
    }

    // An empty buffer has no line endings to have detected a format
    // from, so it takes the first of 'fileformats' — but only if the
    // user gave that option a value; otherwise its own default stands.
    if buf_is_empty(Buf::current()) && option_was_set(kOptFileformats) {
        set_fileformat(default_fileformat(), OptionSetFlags::LOCAL);
    }
    set_title_defaults();
}

/// 'helplang' defaults to the message language's two-letter code.
pub(crate) fn set_helplang_default(lang: &[u8]) {
    // Two letters is what the option holds; anything shorter cannot be
    // a language code.
    if lang.len() < 2 || option_was_set(kOptHelplang) {
        return;
    }
    let lower = |c: u8| c.to_ascii_lowercase();
    let code = if lang[..3.min(lang.len())].eq_ignore_ascii_case(b"zh_") && lang.len() >= 5 {
        // zh_CN becomes "cn", zh_TW becomes "tw".
        [lower(lang[3]), lower(lang[4])]
    } else if lang[0] == b'C' {
        // Any C-like setting, C.UTF-8 included, becomes "en".
        *b"en"
    } else {
        [lang[0], lang[1]]
    };
    P_HLG.set(XString::from_bytes(&code));
}

/// 'title' and 'icon' default off unless the user asked for them, so that
/// nvim does not have to ask the terminal what its title was.
pub(crate) fn set_title_defaults() {
    for (opt_idx, field) in [(kOptTitle, P_TITLE), (kOptIcon, P_ICON)] {
        if !option_was_set(opt_idx) {
            change_option_default(opt_idx, OFF);
            field.set(false);
        }
    }
}
