//! Setting and reading an option's value programmatically: the path every
//! `:set`, every `nvim_set_option_value` and every internal assignment ends
//! up in.
//!
//! [`set_option`] is the whole of it. It writes the new value through the
//! variable the table names, hands the option's own callback a chance to
//! reject or adjust it, and then either keeps the result or puts the old
//! value back. Three things about that order are load-bearing:
//!
//! * the callback runs with the *new* value already in the variable, because
//!   most of them read it back through `args->os_varp`;
//! * the old value is freed only once the callback has accepted it, and the
//!   copies handed to the `OptionSet` autocommand outlive both, because that
//!   autocommand may close the buffer the variable lives in;
//! * `secure` is raised around the callback for a value that came from a
//!   modeline, the sandbox, or an option previously marked insecure, and
//!   lowered again whatever the callback did.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::tr;
use crate::types::AutoEvent;
use crate::winlayer::Win;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;
use std::ffi::CString;

use crate::api::private::helpers::cstr_as_string;
use crate::autocmd::{apply_autocmds, do_filetype_autocmd};
use crate::charset::buf_init_chartab;
use crate::cstr;
use crate::drawscreen::{UPD_NOT_VALID, comp_col, redraw_all_later};
use crate::eval::vars::{
    get_vim_var_str, optval_as_tv, reset_v_option_vars, set_vim_var_string, set_vim_var_tv,
};
use crate::global_cell::GlobalCell;
use crate::lua::executor::nlua_set_sctx;
use crate::main::{
    curbuf, current_sctx, curwin, e_invarg, e_sandbox, e_secure, e_unsupportedoption, sandbox,
    secure, starting, t_colors,
};
use crate::memory::{xfree, xmalloc, xstrdup, xstrlcpy};
use crate::message::emsg;
use crate::message_fmt::msg_cstr;
use crate::mouse::setmouse;

use crate::options::{
    find_option_index, kOptAutocomplete, kOptAutoread, kOptFiletype, kOptFormatlistpat, kOptFsync,
    kOptInvalid, kOptMouse, kOptScrolloff, kOptSidescrolloff, kOptSpelllang, kOptSyntax,
    kOptUndolevels, kOptWinbar, options,
};
use crate::optionstr::check_illegal_path_names;
use crate::os::cshim::{gettext, gettext_owned, snprintf};
use crate::types::{
    IOSIZE, NUL, OptIndex, OptVal, OptionSetFlags, String_0, Vv, optset_T, ptrdiff_t, scid_T,
    sctx_T, size_t, uint32_t, vimoption_T,
};
use crate::ui::ui_call_option_set;
use crate::window::set_winbar;

use super::{
    NO_LOCAL_UNDOLEVEL, NUMBUFLEN, OptSlot, SID_NONE, boolean_optval, check_redraw,
    do_spelllang_source, do_syntax_autocmd, find_tty_option_end, get_varp, get_varp_scope,
    insecure_flag, is_option_hidden, kOptFlagCurswant, kOptFlagHLOnly, kOptFlagRedrAll,
    kOptFlagSecure, kOptFlagUIOption, kOptScopeBuf, kOptScopeWin, kOptValTypeString,
    mark_option_was_set, option_has_scope, option_has_type, option_is_global_local,
    option_is_global_only, option_scope_idx, option_var, optval_copy, optval_equal, optval_free,
    optval_from_varp, set_option_last_set, set_option_varp, validate_option_value,
};
use crate::pos::MAXCOL;
use crate::winlayer::Buf;

/// Record where the option was just set, in every scope the flags name.
pub(crate) fn set_option_sctx(
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
    mut script_ctx: sctx_T,
) {
    let both = !opt_flags.has(OptionSetFlags::LOCAL | OptionSetFlags::GLOBAL);

    // A modeline already carries the line it was found on.
    if !opt_flags.has(OptionSetFlags::MODELINE) {
        // The execution stack always has the frame being sourced on top
        // while anything can set an option.
        script_ctx.sc_lnum += crate::runtime::innermost_frame().es_lnum;
    }
    // SAFETY: `nlua_set_sctx` only reads and rewrites the context in place.
    unsafe { nlua_set_sctx(&raw mut script_ctx) };

    if both || opt_flags.has(OptionSetFlags::GLOBAL) || option_is_global_only(opt_idx) {
        set_option_last_set(opt_idx, script_ctx);
    }
    if !both && !opt_flags.has(OptionSetFlags::LOCAL) {
        return;
    }
    // SAFETY: `curbuf`/`curwin` are live for as long as the editor is, and
    // the scope index the table gives is in range of the per-scope array.
    if option_has_scope(opt_idx, kOptScopeBuf) {
        let at = option_scope_idx(opt_idx, kOptScopeBuf) as usize;
        cur_buf().b_p_script_ctx[at] = script_ctx;
    } else if option_has_scope(opt_idx, kOptScopeWin) {
        let at = option_scope_idx(opt_idx, kOptScopeWin) as usize;
        cur_win().w_onebuf_opt.wo_script_ctx[at] = script_ctx;
        if both {
            // A bare `:set` also writes the "all buffers" copy.
            cur_win().w_allbuf_opt.wo_script_ctx[at] = script_ctx;
        }
    }
}

/// Fire the `OptionSet` autocommand for an option that has just changed.
///
/// The `v:option_*` variables it reads are also the re-entrancy guard: a
/// non-empty `v:option_type` means we are already inside one of these.
fn apply_optionset_autocmd(
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
    oldval: OptVal,
    oldval_g: OptVal,
    oldval_l: OptVal,
    newval: OptVal,
) {
    // SAFETY: all of these only read globals or the values handed in.
    if starting.get() != 0 || unsafe { *get_vim_var_str(Vv::OptionType) } != NUL as c_char {
        return;
    }

    let mut oldval_tv = unsafe { optval_as_tv(oldval, false) };
    let mut oldval_g_tv = unsafe { optval_as_tv(oldval_g, false) };
    let mut oldval_l_tv = unsafe { optval_as_tv(oldval_l, false) };
    let mut newval_tv = unsafe { optval_as_tv(newval, false) };

    unsafe { set_vim_var_tv(Vv::OptionOld, &raw mut oldval_tv) };
    unsafe { set_vim_var_tv(Vv::OptionNew, &raw mut newval_tv) };

    let type_str: &CStr = if opt_flags.has(OptionSetFlags::LOCAL) {
        c"local"
    } else {
        c"global"
    };
    unsafe {
        set_vim_var_string(
            Vv::OptionType,
            type_str.as_ptr(),
            type_str.count_bytes() as ptrdiff_t,
        )
    };

    // The command spellings are not exclusive: `:setlocal` on a
    // global-local option arrives with both scope bits clear only for a
    // bare `:set`, and a modeline overrides whatever came before it.
    let mut command = |name: &CStr| {
        unsafe {
            set_vim_var_string(
                Vv::OptionCommand,
                name.as_ptr(),
                name.count_bytes() as ptrdiff_t,
            )
        };
    };
    if opt_flags.has(OptionSetFlags::LOCAL) {
        command(c"setlocal");
        unsafe { set_vim_var_tv(Vv::OptionOldlocal, &raw mut oldval_tv) };
    }
    if opt_flags.has(OptionSetFlags::GLOBAL) {
        command(c"setglobal");
        unsafe { set_vim_var_tv(Vv::OptionOldglobal, &raw mut oldval_tv) };
    }
    if !opt_flags.has(OptionSetFlags::LOCAL | OptionSetFlags::GLOBAL) {
        command(c"set");
        unsafe { set_vim_var_tv(Vv::OptionOldlocal, &raw mut oldval_l_tv) };
        unsafe { set_vim_var_tv(Vv::OptionOldglobal, &raw mut oldval_g_tv) };
    }
    if opt_flags.has(OptionSetFlags::MODELINE) {
        command(c"modeline");
        unsafe { set_vim_var_tv(Vv::OptionOldlocal, &raw mut oldval_tv) };
    }

    unsafe {
        apply_autocmds(
            AutoEvent::OptionSet,
            get_option(opt_idx).fullname,
            ptr::null_mut(),
            false,
            ptr::null_mut(),
        )
    };
    unsafe { reset_v_option_vars() };
}

/// Whether the name is one of the terminal options nvim keeps only to stay
/// compatible with scripts that set them.
///
pub(crate) fn is_tty_option(name: &CStr) -> bool {
    // SAFETY: `name` is NUL-terminated, which is all the walk needs.
    !unsafe { find_tty_option_end(name.as_ptr()) }.is_null()
}

/// What a terminal option reads back as. Only `t_Co`, `term` and `ttytype`
/// have anything to say; the rest answer with the empty string.
///
pub(crate) fn get_tty_option(name: &CStr) -> OptVal {
    // SAFETY: `name` is NUL-terminated, and every arm allocates its answer.
    let value = unsafe {
        if name == c"t_Co" {
            if t_colors.get() <= 1 {
                xstrdup(c"".as_ptr())
            } else {
                let buf = xmalloc(NUMBUFLEN as size_t).cast::<c_char>();
                snprintf(buf, NUMBUFLEN as size_t, c"%d".as_ptr(), t_colors.get());
                buf
            }
        } else if name == c"term" {
            xstrdup(TERM.or(c"nvim"))
        } else if name == c"ttytype" {
            xstrdup(TTYTYPE.or(c"nvim"))
        } else if is_tty_option(name) {
            xstrdup(c"".as_ptr())
        } else {
            return OptVal::Nil;
        }
    };
    // SAFETY: every arm above allocated a NUL-terminated string.
    OptVal::String(unsafe { cstr_as_string(value) })
}

/// Remember what a script set `term` or `ttytype` to, so it reads back.
/// Nothing else is stored, and `false` says so.
///
/// # Safety
///
/// On success the option takes ownership of `value`, which must be an
/// allocation the option module may free.
pub(crate) unsafe fn set_tty_option(name: &CStr, value: *mut c_char) -> bool {
    for (spelling, cell) in [(c"term", &TERM), (c"ttytype", &TTYTYPE)] {
        if name == spelling {
            // SAFETY: `value` is ours now.
            unsafe { cell.replace(value) };
            return true;
        }
    }
    false
}

/// The `term`/`ttytype` a script last set, if any.
static TERM: TtyName = TtyName(GlobalCell::new(ptr::null_mut()));
static TTYTYPE: TtyName = TtyName(GlobalCell::new(ptr::null_mut()));

/// An owned, nullable C string kept only so that a script setting `term` or
/// `ttytype` reads back what it wrote.
struct TtyName(GlobalCell<*mut c_char>);

impl TtyName {
    /// What was stored, or `fallback` if nothing ever was.
    fn or(&self, fallback: &'static CStr) -> *const c_char {
        let stored = self.0.get();
        if stored.is_null() {
            fallback.as_ptr()
        } else {
            stored
        }
    }

    /// Store `value`, releasing whatever was there.
    ///
    /// # Safety
    ///
    /// `value` must be an allocation this module may free.
    unsafe fn replace(&self, value: *mut c_char) {
        let stored = self.0.get();
        if !stored.is_null() {
            // SAFETY: only this function writes the cell, and only an
            // allocation we own.
            unsafe { xfree(stored.cast::<c_void>()) };
        }
        self.0.set(value);
    }
}

/// The table index of an option name given as bytes, or `kOptInvalid`.
pub(crate) fn find_option_len(name: &[u8]) -> OptIndex {
    if name.is_empty() {
        return kOptInvalid;
    }
    find_option_index(name)
}

/// The table index of an option name, or `kOptInvalid`.
pub(crate) fn find_option(name: &CStr) -> OptIndex {
    find_option_len(name.to_bytes())
}

/// An owned copy of the option's value in the scope the flags name. An
/// unknown option answers nil rather than failing.
pub(crate) fn get_option_value(opt_idx: OptIndex, opt_flags: OptionSetFlags) -> OptVal {
    if opt_idx == kOptInvalid {
        return OptVal::Nil;
    }
    // SAFETY: `opt` points into the option table, which is what both of
    // these want.
    optval_copy(unsafe { optval_from_varp(opt_idx, get_varp_scope(opt_idx, opt_flags)) })
}

/// The option table's row for an option: everything the option *is*, all
/// of it immutable. What changes about it is `super::state`'s business.
pub(crate) fn get_option(opt_idx: OptIndex) -> &'static vimoption_T {
    debug_assert!(opt_idx != kOptInvalid);
    &options[opt_idx as usize]
}

/// The value that stands for "this window or buffer has no local value".
///
/// Only a global-local option has one to speak of; for everything else the
/// global value plays the part. The four shapes below are the ones the
/// global-local options actually use — a new global-local option has to
/// choose one of them, which is what the final arm asserts.
pub(crate) fn get_option_unset_value(opt_idx: OptIndex) -> OptVal {
    debug_assert!(opt_idx != kOptInvalid);

    if !option_is_global_local(opt_idx) {
        // SAFETY: `optval_from_varp` reads the variable as its own type.
        return unsafe {
            optval_from_varp(opt_idx, get_varp_scope(opt_idx, OptionSetFlags::GLOBAL))
        };
    }
    // A string global-local option is unset when it is empty.
    if option_has_type(opt_idx, kOptValTypeString) {
        return OptVal::String(String_0::from_raw_parts(c"".as_ptr() as *mut c_char, 0));
    }
    match opt_idx {
        kOptAutocomplete | kOptAutoread | kOptFsync => boolean_optval(None),
        kOptScrolloff | kOptSidescrolloff => OptVal::Number(-1),
        kOptUndolevels => OptVal::Number(NO_LOCAL_UNDOLEVEL as _),
        _ => unreachable!("global-local option {opt_idx} has no unset value"),
    }
}

/// Whether the current window or buffer has left a global-local option's
/// local value unset. Always false for an option that is not global-local.
pub(crate) fn is_option_local_value_unset(opt_idx: OptIndex) -> bool {
    if !option_is_global_local(opt_idx) {
        return false;
    }
    // SAFETY: `get_varp_scope` wants a row of the option table.
    let local = unsafe {
        let varp_local = get_varp_scope(opt_idx, OptionSetFlags::LOCAL);
        optval_from_varp(opt_idx, varp_local)
    };
    optval_equal(local, get_option_unset_value(opt_idx))
}

/// React to an option whose variable already holds its new value: run the
/// option's own callback, and on rejection put `old_value` back.
///
/// `set_sid` is the script the change is attributed to — 0 for the one
/// currently running, `SID_NONE` to leave the attribution alone. `direct`
/// skips every side effect, `value_replaced` says the whole value was
/// written rather than amended.
///
/// Returns an untranslated error message, or null.
///
/// # Safety
///
/// `varp` must be `opt_idx`'s variable in the scope `opt_flags` names, and
/// `errbuf` writable for `errbuflen` bytes.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn did_set_option(
    opt_idx: OptIndex,
    varp: OptSlot,
    old_value: OptVal,
    new_value: OptVal,
    opt_flags: OptionSetFlags,
    set_sid: scid_T,
    direct: bool,
    value_replaced: bool,
    errbuf: *mut c_char,
    errbuflen: size_t,
) -> *const c_char {
    let opt = get_option(opt_idx);
    let mut errmsg: *const c_char = ptr::null();
    let mut restore_chartab = false;
    let mut value_changed = false;
    let mut value_checked = false;

    // SAFETY: the caller's `varp` is this option's variable, `opt` a row of
    // the table, and `errbuf` writable for `errbuflen` bytes.
    let mut args = optset_T {
        os_varp: varp,
        os_idx: opt_idx,
        os_flags: opt_flags,
        os_oldval: old_value,
        os_newval: new_value,
        os_value_checked: false,
        os_value_changed: false,
        os_restore_chartab: false,
        os_errbuf: errbuf,
        os_errbuflen: errbuflen,
        os_win: curwin.get().cast::<c_void>(),
        os_buf: curbuf.get().cast::<c_void>(),
    };

    if direct {
        // Nothing to vet: the caller is putting a value back.
    } else if opt.immutable && !optval_equal(old_value, new_value) {
        errmsg = e_unsupportedoption.as_ptr();
    } else if (secure.get() != 0 || sandbox.get() != 0)
        && opt.flags & kOptFlagSecure as uint32_t != 0
    {
        errmsg = e_secure.as_ptr();
    } else if new_value.as_string().is_some()
        && check_illegal_path_names(unsafe { CStr::from_ptr(*varp.string_var()) }, opt.flags)
    {
        errmsg = e_invarg.as_ptr();
    } else if let Some(did_set_cb) = opt.opt_did_set_cb {
        errmsg = unsafe { did_set_cb(&raw mut args) };
        // 'filetype' and 'syntax' report whether the value really moved;
        // they, 'keymap' and the character-class options report whether
        // they have vetted it themselves; and the character-class
        // options leave the table needing a rebuild if they failed.
        value_changed = args.os_value_changed;
        value_checked = args.os_value_checked;
        restore_chartab = args.os_restore_chartab;
    }

    if !errmsg.is_null() {
        unsafe { set_option_varp(opt_idx, varp, old_value, true) };
        if restore_chartab {
            unsafe { buf_init_chartab(curbuf.get(), true) };
        }
        return errmsg;
    }

    // The callback may have freed or rewritten what it was handed, so
    // read the value back rather than trusting `new_value`.
    let new_value = unsafe { optval_from_varp(opt_idx, varp) };

    if set_sid != SID_NONE {
        let script_ctx = if set_sid == 0 {
            current_sctx.get()
        } else {
            sctx_T {
                sc_sid: set_sid,
                sc_seq: 0,
                sc_lnum: 0,
                sc_chan: 0,
            }
        };
        set_option_sctx(opt_idx, opt_flags, script_ctx);
    }

    optval_free(old_value);

    let scope_both = !opt_flags.has(OptionSetFlags::LOCAL | OptionSetFlags::GLOBAL);
    if scope_both {
        if option_is_global_local(opt_idx) {
            // A bare `:set` on a global-local option drops the local
            // value rather than assigning it too.
            let varp_local = get_varp_scope(opt_idx, OptionSetFlags::LOCAL);
            let unset = optval_copy(get_option_unset_value(opt_idx));
            unsafe { set_option_varp(opt_idx, varp_local, unset, true) };
        } else {
            let varp_global = get_varp_scope(opt_idx, OptionSetFlags::GLOBAL);
            unsafe { set_option_varp(opt_idx, varp_global, optval_copy(new_value), true) };
        }
    }

    if direct {
        return errmsg;
    }

    // The autocommands go last, once every flag they might read is set.
    match opt_idx {
        kOptSyntax => unsafe { do_syntax_autocmd(curbuf.get(), value_changed) },
        // A modeline only forces the FileType autocommand when the
        // filetype really changed.
        kOptFiletype if !opt_flags.has(OptionSetFlags::MODELINE) || value_changed => {
            do_filetype_autocmd(unsafe { Buf::current() }, value_changed);
        }
        kOptSpelllang => unsafe { do_spelllang_source(curwin.get()) },
        _ => {}
    }

    // 'ruler', 'showcmd', 'columns' and 'laststatus' all move it.
    unsafe { comp_col() };

    match opt_idx {
        kOptMouse => setmouse(),
        // 'formatlistpat' is what 'breakindentopt' list mode indents by.
        kOptFormatlistpat if cur_win().w_briopt_list != 0 => {
            unsafe { redraw_all_later(UPD_NOT_VALID) };
        }
        kOptWinbar => set_winbar(true),
        _ => {}
    }

    if cur_win().w_curswant != MAXCOL as c_int
        && opt.flags & (kOptFlagCurswant | kOptFlagRedrAll) as uint32_t != 0
        && opt.flags & kOptFlagHLOnly as uint32_t == 0
    {
        cur_win().w_set_curswant = true;
    }

    check_redraw(opt.flags);

    mark_option_was_set(opt_idx);

    // Anything set from a modeline, from the sandbox or in secure mode
    // is insecure unless the callback vetted it; replacing a value
    // outright clears the mark again.
    let flagsp = unsafe { insecure_flag(curwin.get(), opt_idx, opt_flags) };
    let flagsp_local =
        scope_both.then(|| unsafe { insecure_flag(curwin.get(), opt_idx, OptionSetFlags::LOCAL) });
    if !value_checked
        && (secure.get() != 0 || sandbox.get() != 0 || opt_flags.has(OptionSetFlags::MODELINE))
    {
        flagsp.set(true);
        if let Some(local) = flagsp_local {
            local.set(true);
        }
    } else if value_replaced {
        flagsp.set(false);
        if let Some(local) = flagsp_local {
            local.set(false);
        }
    }

    errmsg
}

/// Give an option a new value, with everything that entails.
///
/// Takes ownership of `value`. See the module docs for the ordering; see
/// [`did_set_option`] for `set_sid`, `direct` and `value_replaced`.
///
/// Returns an untranslated error message, or null.
///
/// # Safety
///
/// `errbuf` must be writable for `errbuflen` bytes.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn set_option(
    opt_idx: OptIndex,
    mut value: OptVal,
    opt_flags: OptionSetFlags,
    set_sid: scid_T,
    direct: bool,
    value_replaced: bool,
    errbuf: *mut c_char,
    errbuflen: size_t,
) -> *const c_char {
    debug_assert!(opt_idx != kOptInvalid);

    if !direct {
        // SAFETY: the caller's `errbuf` is writable for `errbuflen` bytes.
        let errmsg =
            unsafe { validate_option_value(opt_idx, &mut value, opt_flags, errbuf, errbuflen) };
        if !errmsg.is_null() {
            optval_free(value);
            return errmsg;
        }
    }

    let opt = get_option(opt_idx);
    let scope_local = opt_flags.has(OptionSetFlags::LOCAL);
    let scope_global = opt_flags.has(OptionSetFlags::GLOBAL);
    let scope_both = !scope_local && !scope_global;
    // True only for a global-local option, by construction.
    let is_opt_local_unset = is_option_local_value_unset(opt_idx);

    // SAFETY: every pointer below comes from the option table, and `errbuf`
    // is the caller's writable buffer.
    // `:set opt=val` on a global-local option resets the local value, so
    // it is the global variable that is being written.
    let varp = if scope_both && option_is_global_local(opt_idx) {
        option_var(opt_idx)
    } else {
        get_varp_scope(opt_idx, opt_flags)
    };
    let varp_local = get_varp_scope(opt_idx, OptionSetFlags::LOCAL);
    let varp_global = get_varp_scope(opt_idx, OptionSetFlags::GLOBAL);

    let old_value = unsafe { optval_from_varp(opt_idx, varp) };
    let old_global_value = unsafe { optval_from_varp(opt_idx, varp_global) };
    // An unset local value reads as the global one.
    let old_local_value = if is_opt_local_unset {
        old_global_value
    } else {
        unsafe { optval_from_varp(opt_idx, varp_local) }
    };
    // What was actually in effect, which is what `OptionSet` reports as
    // `v:option_old`: for `:setlocal` on a global-local option with no
    // local value, that is whatever the option reads through.
    let used_old_value = if scope_local && is_opt_local_unset {
        unsafe { optval_from_varp(opt_idx, get_varp(opt_idx)) }
    } else {
        old_value
    };

    // The autocommand may close the buffer these live in, so it gets
    // copies that outlive the variables.
    let saved_used_value = optval_copy(used_old_value);
    let saved_old_global_value = optval_copy(old_global_value);
    let saved_old_local_value = optval_copy(old_local_value);
    let saved_new_value = optval_copy(value);

    let insecure = unsafe { insecure_flag(curwin.get(), opt_idx, opt_flags) }.is_set();
    let secure_saved = secure.get();
    // Deal with the side effects of a modeline, of the sandbox, or of a
    // value amended rather than replaced, in secure mode.
    if opt_flags.has(OptionSetFlags::MODELINE)
        || sandbox.get() != 0
        || (!value_replaced && insecure)
    {
        secure.set(1);
    }

    unsafe { set_option_varp(opt_idx, varp, value, false) };
    let errmsg = unsafe {
        did_set_option(
            opt_idx,
            varp,
            old_value,
            value,
            opt_flags,
            set_sid,
            direct,
            value_replaced,
            errbuf,
            errbuflen,
        )
    };

    secure.set(secure_saved);

    if errmsg.is_null() && !direct {
        if starting.get() == 0 {
            apply_optionset_autocmd(
                opt_idx,
                opt_flags,
                saved_used_value,
                saved_old_global_value,
                saved_old_local_value,
                saved_new_value,
            );
        }
        if opt.flags & kOptFlagUIOption as uint32_t != 0 {
            ui_call_option_set(
                unsafe { cstr_as_string(opt.fullname) },
                super::optval_as_object(saved_new_value),
            );
        }
    }

    optval_free(saved_used_value);
    optval_free(saved_old_local_value);
    optval_free(saved_old_global_value);
    optval_free(saved_new_value);

    errmsg
}

/// Write a value with no side effects at all: no callback, no autocommand,
/// no validation. Only for values the editor itself computed.
pub(crate) fn set_option_direct(
    opt_idx: OptIndex,
    value: OptVal,
    opt_flags: OptionSetFlags,
    set_sid: scid_T,
) {
    if is_option_hidden(opt_idx) {
        return;
    }
    let mut errbuf = [0 as c_char; IOSIZE as usize];
    // SAFETY: `errbuf` is `IOSIZE` writable bytes. Nothing can report an
    // error on this path, which is what the assertion says.
    let errmsg = unsafe {
        set_option(
            opt_idx,
            optval_copy(value),
            opt_flags,
            set_sid,
            true,
            true,
            errbuf.as_mut_ptr(),
            IOSIZE as size_t,
        )
    };
    debug_assert!(errmsg.is_null());
}

/// Copy a callback's message into the option frame's error buffer and
/// answer it, or answer null.
///
/// A `did_set_*` callback answers a pointer, and `set_option` handed it the
/// buffer that pointer has to live in; a callback whose message came from
/// an owned source reports it through here.
///
/// # Safety
///
/// `args` must be the option table's call frame.
pub(crate) unsafe fn answer_err(args: *mut optset_T, msg: Option<CString>) -> *const c_char {
    let Some(msg) = msg else {
        return ptr::null();
    };
    // SAFETY: the caller's frame names a buffer of `os_errbuflen` bytes,
    // and `msg` is NUL-terminated.
    unsafe { xstrlcpy((*args).os_errbuf, msg.as_ptr(), (*args).os_errbuflen) };
    // SAFETY: the same frame.
    unsafe { (*args).os_errbuf }
}

/// Give an option a new value the way a script would. Takes ownership of
/// nothing: the caller keeps `value`.
///
/// Returns an untranslated error message.
///
/// The message is the caller's: the `did_set_*` callbacks format into a
/// buffer belonging to this frame, and the answer is copied out of it.
/// Upstream answers a pointer into one static buffer shared by every
/// setter, which a second rejection — an autocommand's, say — overwrites.
pub(crate) fn set_option_value(
    opt_idx: OptIndex,
    value: OptVal,
    opt_flags: OptionSetFlags,
) -> Option<CString> {
    debug_assert!(opt_idx != kOptInvalid);
    let mut errbuf = [0 as c_char; IOSIZE as usize];

    if sandbox.get() > 0 && get_option(opt_idx).flags & kOptFlagSecure != 0 {
        // SAFETY: a NUL-terminated message static.
        return Some(unsafe { CStr::from_ptr(gettext(e_sandbox).as_ptr()) }.to_owned());
    }
    // SAFETY: the option table is a plain array, and `errbuf` is `IOSIZE`
    // writable bytes.
    let errmsg = unsafe {
        set_option(
            opt_idx,
            optval_copy(value),
            opt_flags,
            0,
            false,
            true,
            errbuf.as_mut_ptr(),
            IOSIZE as size_t,
        )
    };
    // SAFETY: `set_option` answers null or a NUL-terminated message.
    unsafe { cstr::at_opt(errmsg) }.map(CStr::to_owned)
}

/// Drop a global-local option's local value, so it reads through to the
/// global one again.
pub(crate) fn unset_option_local_value(opt_idx: OptIndex) -> Option<CString> {
    debug_assert!(option_is_global_local(opt_idx));
    set_option_value(
        opt_idx,
        get_option_unset_value(opt_idx),
        OptionSetFlags::LOCAL,
    )
}

/// [`set_option_value`] for a name that may be one of the terminal options,
/// which are accepted and discarded rather than reported as unknown.
///
/// # Safety
///
/// `name` must be a NUL-terminated string.
pub(crate) unsafe fn set_option_value_handle_tty(
    name: *const c_char,
    opt_idx: OptIndex,
    value: OptVal,
    opt_flags: OptionSetFlags,
) -> Option<CString> {
    if opt_idx != kOptInvalid {
        return set_option_value(opt_idx, value, opt_flags);
    }
    // SAFETY: the caller's `name` is NUL-terminated.
    let name = unsafe { CStr::from_ptr(name) };
    if is_tty_option(name) {
        return None;
    }
    let name = msg_cstr(name);
    Some(CString::new(tr!("E355: Unknown option: {name}")).unwrap_or_default())
}

/// [`set_option_value`], reporting a rejection as an error message.
pub(crate) fn set_option_value_give_err(
    opt_idx: OptIndex,
    value: OptVal,
    opt_flags: OptionSetFlags,
) {
    if let Some(errmsg) = set_option_value(opt_idx, value, opt_flags) {
        emsg(&gettext_owned(&errmsg));
    }
}

/// Attribute a list of options to the script currently running, for the
/// options another option's callback has just overridden on its behalf.
pub(crate) fn didset_options_sctx(opt_flags: OptionSetFlags, opts: &[OptIndex]) {
    for &opt_idx in opts {
        set_option_sctx(opt_idx, opt_flags, current_sctx.get());
    }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
