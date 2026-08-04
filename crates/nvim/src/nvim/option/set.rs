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

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::autocmd::{EVENT_OPTIONSET, apply_autocmds, do_filetype_autocmd};
use crate::src::nvim::charset::buf_init_chartab;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, comp_col, redraw_all_later};
use crate::src::nvim::eval::vars::{
    get_vim_var_str, optval_as_tv, reset_v_option_vars, set_vim_var_string, set_vim_var_tv,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::lua::executor::nlua_set_sctx;
use crate::src::nvim::main::{
    curbuf, current_sctx, curwin, e_invarg, e_sandbox, e_secure, e_unknown_option2,
    e_unsupportedoption, p_flp, p_mouse, p_wbr, sandbox, secure, starting, t_colors,
};
use crate::src::nvim::memory::{strequal, xfree, xmalloc, xstrdup};
use crate::src::nvim::message::emsg;
use crate::src::nvim::mouse::setmouse;

use crate::src::nvim::options::{
    find_option_index, kOptAutocomplete, kOptAutoread, kOptFsync, kOptInvalid, kOptScrolloff,
    kOptSidescrolloff, kOptUndolevels, options,
};
use crate::src::nvim::optionstr::check_illegal_path_names;
use crate::src::nvim::os::libc::{gettext, snprintf, strlen};
use crate::src::nvim::runtime::exestack;
use crate::src::nvim::types::{
    OptIndex, OptVal, OptValData, String_0, VV_OPTION_COMMAND, VV_OPTION_NEW, VV_OPTION_OLD,
    VV_OPTION_OLDGLOBAL, VV_OPTION_OLDLOCAL, VV_OPTION_TYPE, estack_T, kFalse, kNone, optset_T,
    ptrdiff_t, scid_T, sctx_T, size_t, uint32_t, vimoption_T,
};
use crate::src::nvim::ui::ui_call_option_set;
use crate::src::nvim::window::set_winbar;

use super::{
    IOSIZE, MAXCOL, NO_LOCAL_UNDOLEVEL, NUL, NUMBUFLEN, OPT_GLOBAL, OPT_LOCAL, OPT_MODELINE,
    SID_NONE, check_redraw, do_spelllang_source, do_syntax_autocmd, find_tty_option_end, get_varp,
    get_varp_scope, insecure_flag, is_option_hidden, kOptFlagCurswant, kOptFlagHLOnly,
    kOptFlagInsecure, kOptFlagRedrAll, kOptFlagSecure, kOptFlagUIOption, kOptFlagWasSet,
    kOptScopeBuf, kOptScopeWin, kOptValTypeBoolean, kOptValTypeNil, kOptValTypeNumber,
    kOptValTypeString, option_has_scope, option_has_type, option_is_global_local,
    option_is_global_only, option_scope_idx, optval_copy, optval_equal, optval_free,
    optval_from_varp, set_option_varp, validate_option_value,
};

/// The message buffer the callbacks format into. `set_option` hands its
/// address to every `did_set_*`, so it cannot live on the stack of a
/// function whose frame the autocommands may outlive.
static ERRBUF: GlobalCell<[c_char; IOSIZE as usize]> = GlobalCell::new([0; IOSIZE as usize]);

/// The script context recorded for the global value of an option.
pub fn get_option_sctx(opt_idx: OptIndex) -> *mut sctx_T {
    assert!(opt_idx != kOptInvalid);
    // SAFETY: the option table is a plain array; nothing holds a borrow.
    unsafe { &raw mut (*options.ptr())[opt_idx as usize].script_ctx }
}

/// Record where the option was just set, in every scope the flags name.
pub fn set_option_sctx(opt_idx: OptIndex, opt_flags: c_int, mut script_ctx: sctx_T) {
    let both = opt_flags & (OPT_LOCAL | OPT_GLOBAL) == 0;

    // A modeline already carries the line it was found on.
    if opt_flags & OPT_MODELINE == 0 {
        // SAFETY: the execution stack always has the frame being sourced on
        // top while anything can set an option.
        script_ctx.sc_lnum += unsafe {
            let stack = *exestack.ptr();
            (*stack
                .ga_data
                .cast::<estack_T>()
                .offset(stack.ga_len as isize - 1))
            .es_lnum
        };
    }
    // SAFETY: `nlua_set_sctx` only reads and rewrites the context in place.
    unsafe { nlua_set_sctx(&raw mut script_ctx) };

    if both || opt_flags & OPT_GLOBAL != 0 || option_is_global_only(opt_idx) {
        // SAFETY: the option table is a plain array.
        unsafe { (*options.ptr())[opt_idx as usize].script_ctx = script_ctx };
    }
    if !both && opt_flags & OPT_LOCAL == 0 {
        return;
    }
    // SAFETY: `curbuf`/`curwin` are live for as long as the editor is, and
    // the scope index the table gives is in range of the per-scope array.
    unsafe {
        if option_has_scope(opt_idx, kOptScopeBuf) {
            let at = option_scope_idx(opt_idx, kOptScopeBuf) as usize;
            (*curbuf.get()).b_p_script_ctx[at] = script_ctx;
        } else if option_has_scope(opt_idx, kOptScopeWin) {
            let at = option_scope_idx(opt_idx, kOptScopeWin) as usize;
            (*curwin.get()).w_onebuf_opt.wo_script_ctx[at] = script_ctx;
            if both {
                // A bare `:set` also writes the "all buffers" copy.
                (*curwin.get()).w_allbuf_opt.wo_script_ctx[at] = script_ctx;
            }
        }
    }
}

/// Fire the `OptionSet` autocommand for an option that has just changed.
///
/// The `v:option_*` variables it reads are also the re-entrancy guard: a
/// non-empty `v:option_type` means we are already inside one of these.
fn apply_optionset_autocmd(
    opt_idx: OptIndex,
    opt_flags: c_int,
    oldval: OptVal,
    oldval_g: OptVal,
    oldval_l: OptVal,
    newval: OptVal,
) {
    // SAFETY: all of these only read globals or the values handed in.
    unsafe {
        if starting.get() != 0 || *get_vim_var_str(VV_OPTION_TYPE) != NUL as c_char {
            return;
        }

        let mut oldval_tv = optval_as_tv(oldval, false);
        let mut oldval_g_tv = optval_as_tv(oldval_g, false);
        let mut oldval_l_tv = optval_as_tv(oldval_l, false);
        let mut newval_tv = optval_as_tv(newval, false);

        set_vim_var_tv(VV_OPTION_OLD, &raw mut oldval_tv);
        set_vim_var_tv(VV_OPTION_NEW, &raw mut newval_tv);

        let type_str: &CStr = if opt_flags & OPT_LOCAL != 0 {
            c"local"
        } else {
            c"global"
        };
        set_vim_var_string(
            VV_OPTION_TYPE,
            type_str.as_ptr(),
            type_str.count_bytes() as ptrdiff_t,
        );

        // The command spellings are not exclusive: `:setlocal` on a
        // global-local option arrives with both scope bits clear only for a
        // bare `:set`, and a modeline overrides whatever came before it.
        let mut command = |name: &CStr| {
            set_vim_var_string(
                VV_OPTION_COMMAND,
                name.as_ptr(),
                name.count_bytes() as ptrdiff_t,
            );
        };
        if opt_flags & OPT_LOCAL != 0 {
            command(c"setlocal");
            set_vim_var_tv(VV_OPTION_OLDLOCAL, &raw mut oldval_tv);
        }
        if opt_flags & OPT_GLOBAL != 0 {
            command(c"setglobal");
            set_vim_var_tv(VV_OPTION_OLDGLOBAL, &raw mut oldval_tv);
        }
        if opt_flags & (OPT_LOCAL | OPT_GLOBAL) == 0 {
            command(c"set");
            set_vim_var_tv(VV_OPTION_OLDLOCAL, &raw mut oldval_l_tv);
            set_vim_var_tv(VV_OPTION_OLDGLOBAL, &raw mut oldval_g_tv);
        }
        if opt_flags & OPT_MODELINE != 0 {
            command(c"modeline");
            set_vim_var_tv(VV_OPTION_OLDLOCAL, &raw mut oldval_tv);
        }

        apply_autocmds(
            EVENT_OPTIONSET,
            (*options.ptr())[opt_idx as usize].fullname,
            ptr::null_mut(),
            false,
            ptr::null_mut(),
        );
        reset_v_option_vars();
    }
}

/// Whether the name is one of the terminal options nvim keeps only to stay
/// compatible with scripts that set them.
///
/// # Safety
///
/// `name` must be a NUL-terminated string.
pub unsafe fn is_tty_option(name: *const c_char) -> bool {
    // SAFETY: the caller's `name` is NUL-terminated.
    !unsafe { find_tty_option_end(name) }.is_null()
}

/// What a terminal option reads back as. Only `t_Co`, `term` and `ttytype`
/// have anything to say; the rest answer with the empty string.
///
/// # Safety
///
/// `name` must be a NUL-terminated string.
pub unsafe fn get_tty_option(name: *const c_char) -> OptVal {
    // SAFETY: the caller's `name` is NUL-terminated.
    let value = unsafe {
        if strequal(name, c"t_Co".as_ptr()) {
            if t_colors.get() <= 1 {
                xstrdup(c"".as_ptr())
            } else {
                let buf = xmalloc(NUMBUFLEN as size_t).cast::<c_char>();
                snprintf(buf, NUMBUFLEN as size_t, c"%d".as_ptr(), t_colors.get());
                buf
            }
        } else if strequal(name, c"term".as_ptr()) {
            xstrdup(TERM.or(c"nvim"))
        } else if strequal(name, c"ttytype".as_ptr()) {
            xstrdup(TTYTYPE.or(c"nvim"))
        } else if is_tty_option(name) {
            xstrdup(c"".as_ptr())
        } else {
            return OptVal {
                type_0: kOptValTypeNil,
                data: OptValData { boolean: kFalse },
            };
        }
    };
    OptVal {
        type_0: kOptValTypeString,
        // SAFETY: every arm above allocated a NUL-terminated string.
        data: OptValData {
            string: unsafe { cstr_as_string(value) },
        },
    }
}

/// Remember what a script set `term` or `ttytype` to, so it reads back.
/// Nothing else is stored, and `false` says so.
///
/// # Safety
///
/// `name` must be NUL-terminated. On success the option takes ownership of
/// `value`, which must be an allocation the option module may free.
pub unsafe fn set_tty_option(name: *const c_char, value: *mut c_char) -> bool {
    // SAFETY: the caller's `name` is NUL-terminated and `value` is ours now.
    unsafe {
        for (spelling, cell) in [(c"term", &TERM), (c"ttytype", &TTYTYPE)] {
            if strequal(name, spelling.as_ptr()) {
                cell.replace(value);
                return true;
            }
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

/// The table index of `len` bytes of option name, or `kOptInvalid`.
///
/// # Safety
///
/// `name` must be readable for `len` bytes.
pub unsafe fn find_option_len(name: *const c_char, len: size_t) -> OptIndex {
    if len == 0 {
        return kOptInvalid;
    }
    // SAFETY: the caller passes `len` readable bytes at `name`.
    find_option_index(unsafe { ::core::slice::from_raw_parts(name.cast::<u8>(), len) })
}

/// The table index of an option name, or `kOptInvalid`.
///
/// # Safety
///
/// `name` must be a NUL-terminated string.
pub unsafe fn find_option(name: *const c_char) -> OptIndex {
    // SAFETY: the caller's `name` is NUL-terminated.
    unsafe { find_option_len(name, strlen(name)) }
}

/// An owned copy of the option's value in the scope the flags name. An
/// unknown option answers nil rather than failing.
pub fn get_option_value(opt_idx: OptIndex, opt_flags: c_int) -> OptVal {
    if opt_idx == kOptInvalid {
        return OptVal {
            type_0: kOptValTypeNil,
            data: OptValData { boolean: kFalse },
        };
    }
    // SAFETY: `opt` points into the option table, which is what both of
    // these want.
    unsafe {
        let opt = get_option(opt_idx);
        optval_copy(optval_from_varp(opt_idx, get_varp_scope(opt, opt_flags)))
    }
}

/// The option table's row for an option.
pub fn get_option(opt_idx: OptIndex) -> *mut vimoption_T {
    assert!(opt_idx != kOptInvalid);
    // SAFETY: the option table is a plain array; nothing holds a borrow.
    unsafe { &raw mut (*options.ptr())[opt_idx as usize] }
}

/// The value that stands for "this window or buffer has no local value".
///
/// Only a global-local option has one to speak of; for everything else the
/// global value plays the part. The four shapes below are the ones the
/// global-local options actually use — a new global-local option has to
/// choose one of them, which is what the final arm asserts.
pub(crate) fn get_option_unset_value(opt_idx: OptIndex) -> OptVal {
    assert!(opt_idx != kOptInvalid);

    if !option_is_global_local(opt_idx) {
        // SAFETY: `get_varp_scope` wants a row of the option table.
        return unsafe {
            optval_from_varp(opt_idx, get_varp_scope(get_option(opt_idx), OPT_GLOBAL))
        };
    }
    // A string global-local option is unset when it is empty.
    if option_has_type(opt_idx, kOptValTypeString) {
        return OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: c"".as_ptr() as *mut c_char,
                    size: 0,
                },
            },
        };
    }
    let (type_0, data) = match opt_idx {
        kOptAutocomplete | kOptAutoread | kOptFsync => {
            (kOptValTypeBoolean, OptValData { boolean: kNone })
        }
        kOptScrolloff | kOptSidescrolloff => (kOptValTypeNumber, OptValData { number: -1 }),
        kOptUndolevels => (
            kOptValTypeNumber,
            OptValData {
                number: NO_LOCAL_UNDOLEVEL as _,
            },
        ),
        _ => unreachable!("global-local option {opt_idx} has no unset value"),
    };
    OptVal { type_0, data }
}

/// Whether the current window or buffer has left a global-local option's
/// local value unset. Always false for an option that is not global-local.
pub(crate) fn is_option_local_value_unset(opt_idx: OptIndex) -> bool {
    if !option_is_global_local(opt_idx) {
        return false;
    }
    // SAFETY: `get_varp_scope` wants a row of the option table.
    let local = unsafe {
        let varp_local = get_varp_scope(get_option(opt_idx), OPT_LOCAL);
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
    varp: *mut c_void,
    old_value: OptVal,
    new_value: OptVal,
    opt_flags: c_int,
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
    unsafe {
        let mut args = optset_T {
            os_varp: varp,
            os_idx: opt_idx,
            os_flags: opt_flags,
            os_oldval: old_value.data,
            os_newval: new_value.data,
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
        } else if (*opt).immutable && !optval_equal(old_value, new_value) {
            errmsg = e_unsupportedoption.ptr().cast::<c_char>();
        } else if (secure.get() != 0 || sandbox.get() != 0)
            && (*opt).flags & kOptFlagSecure as uint32_t != 0
        {
            errmsg = e_secure.ptr().cast::<c_char>();
        } else if new_value.type_0 == kOptValTypeString
            && check_illegal_path_names(*varp.cast::<*mut c_char>(), (*opt).flags)
        {
            errmsg = e_invarg.ptr().cast::<c_char>();
        } else if let Some(did_set_cb) = (*opt).opt_did_set_cb {
            errmsg = did_set_cb(&raw mut args);
            // 'filetype' and 'syntax' report whether the value really moved;
            // they, 'keymap' and the character-class options report whether
            // they have vetted it themselves; and the character-class
            // options leave the table needing a rebuild if they failed.
            value_changed = args.os_value_changed;
            value_checked = args.os_value_checked;
            restore_chartab = args.os_restore_chartab;
        }

        if !errmsg.is_null() {
            set_option_varp(opt_idx, varp, old_value, true);
            if restore_chartab {
                buf_init_chartab(curbuf.get(), true);
            }
            return errmsg;
        }

        // The callback may have freed or rewritten what it was handed, so
        // read the value back rather than trusting `new_value`.
        let new_value = optval_from_varp(opt_idx, varp);

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

        let scope_both = opt_flags & (OPT_LOCAL | OPT_GLOBAL) == 0;
        if scope_both {
            if option_is_global_local(opt_idx) {
                // A bare `:set` on a global-local option drops the local
                // value rather than assigning it too.
                let varp_local = get_varp_scope(opt, OPT_LOCAL);
                let unset = optval_copy(get_option_unset_value(opt_idx));
                set_option_varp(opt_idx, varp_local, unset, true);
            } else {
                let varp_global = get_varp_scope(opt, OPT_GLOBAL);
                set_option_varp(opt_idx, varp_global, optval_copy(new_value), true);
            }
        }

        if direct {
            return errmsg;
        }

        // The autocommands go last, once every flag they might read is set.
        if varp == (&raw mut (*curbuf.get()).b_p_syn).cast::<c_void>() {
            do_syntax_autocmd(curbuf.get(), value_changed);
        } else if varp == (&raw mut (*curbuf.get()).b_p_ft).cast::<c_void>() {
            // A modeline only forces the FileType autocommand when the
            // filetype really changed.
            if opt_flags & OPT_MODELINE == 0 || value_changed {
                do_filetype_autocmd(curbuf.get(), value_changed);
            }
        } else if varp == (&raw mut (*(*curwin.get()).w_s).b_p_spl).cast::<c_void>() {
            do_spelllang_source(curwin.get());
        }

        // 'ruler', 'showcmd', 'columns' and 'laststatus' all move it.
        comp_col();

        if varp == p_mouse.ptr().cast::<c_void>() {
            setmouse();
        } else if (varp == p_flp.ptr().cast::<c_void>()
            || varp == (&raw mut (*curbuf.get()).b_p_flp).cast::<c_void>())
            && (*curwin.get()).w_briopt_list != 0
        {
            // 'formatlistpat' is what 'breakindentopt' list mode indents by.
            redraw_all_later(UPD_NOT_VALID);
        } else if varp == p_wbr.ptr().cast::<c_void>()
            || varp == (&raw mut (*curwin.get()).w_onebuf_opt.wo_wbr).cast::<c_void>()
        {
            set_winbar(true);
        }

        if (*curwin.get()).w_curswant != MAXCOL as c_int
            && (*opt).flags & (kOptFlagCurswant | kOptFlagRedrAll) as uint32_t != 0
            && (*opt).flags & kOptFlagHLOnly as uint32_t == 0
        {
            (*curwin.get()).w_set_curswant = 1;
        }

        check_redraw((*opt).flags);

        (*opt).flags |= kOptFlagWasSet as uint32_t;

        // Anything set from a modeline, from the sandbox or in secure mode
        // is insecure unless the callback vetted it; replacing a value
        // outright clears the mark again.
        let flagsp = insecure_flag(curwin.get(), opt_idx, opt_flags);
        let flagsp_local = scope_both.then(|| insecure_flag(curwin.get(), opt_idx, OPT_LOCAL));
        if !value_checked
            && (secure.get() != 0 || sandbox.get() != 0 || opt_flags & OPT_MODELINE != 0)
        {
            *flagsp |= kOptFlagInsecure as uint32_t;
            if let Some(local) = flagsp_local {
                *local |= kOptFlagInsecure as uint32_t;
            }
        } else if value_replaced {
            *flagsp &= !(kOptFlagInsecure as uint32_t);
            if let Some(local) = flagsp_local {
                *local &= !(kOptFlagInsecure as uint32_t);
            }
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
    opt_flags: c_int,
    set_sid: scid_T,
    direct: bool,
    value_replaced: bool,
    errbuf: *mut c_char,
    errbuflen: size_t,
) -> *const c_char {
    assert!(opt_idx != kOptInvalid);

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
    let scope_local = opt_flags & OPT_LOCAL != 0;
    let scope_global = opt_flags & OPT_GLOBAL != 0;
    let scope_both = !scope_local && !scope_global;
    // True only for a global-local option, by construction.
    let is_opt_local_unset = is_option_local_value_unset(opt_idx);

    // SAFETY: every pointer below comes from the option table, and `errbuf`
    // is the caller's writable buffer.
    unsafe {
        // `:set opt=val` on a global-local option resets the local value, so
        // it is the global variable that is being written.
        let varp = if scope_both && option_is_global_local(opt_idx) {
            (*opt).var
        } else {
            get_varp_scope(opt, opt_flags)
        };
        let varp_local = get_varp_scope(opt, OPT_LOCAL);
        let varp_global = get_varp_scope(opt, OPT_GLOBAL);

        let old_value = optval_from_varp(opt_idx, varp);
        let old_global_value = optval_from_varp(opt_idx, varp_global);
        // An unset local value reads as the global one.
        let old_local_value = if is_opt_local_unset {
            old_global_value
        } else {
            optval_from_varp(opt_idx, varp_local)
        };
        // What was actually in effect, which is what `OptionSet` reports as
        // `v:option_old`: for `:setlocal` on a global-local option with no
        // local value, that is whatever the option reads through.
        let used_old_value = if scope_local && is_opt_local_unset {
            optval_from_varp(opt_idx, get_varp(opt))
        } else {
            old_value
        };

        // The autocommand may close the buffer these live in, so it gets
        // copies that outlive the variables.
        let saved_used_value = optval_copy(used_old_value);
        let saved_old_global_value = optval_copy(old_global_value);
        let saved_old_local_value = optval_copy(old_local_value);
        let saved_new_value = optval_copy(value);

        let insecure = *insecure_flag(curwin.get(), opt_idx, opt_flags);
        let secure_saved = secure.get();
        // Deal with the side effects of a modeline, of the sandbox, or of a
        // value amended rather than replaced, in secure mode.
        if opt_flags & OPT_MODELINE != 0
            || sandbox.get() != 0
            || (!value_replaced && insecure & kOptFlagInsecure as uint32_t != 0)
        {
            secure.set(1);
        }

        set_option_varp(opt_idx, varp, value, false);
        let errmsg = did_set_option(
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
        );

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
            if (*opt).flags & kOptFlagUIOption as uint32_t != 0 {
                ui_call_option_set(
                    cstr_as_string((*opt).fullname),
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
}

/// Write a value with no side effects at all: no callback, no autocommand,
/// no validation. Only for values the editor itself computed.
pub fn set_option_direct(opt_idx: OptIndex, value: OptVal, opt_flags: c_int, set_sid: scid_T) {
    if is_option_hidden(opt_idx) {
        return;
    }
    // SAFETY: `ERRBUF` is `IOSIZE` writable bytes. Nothing can report an
    // error on this path, which is what the assertion says.
    let errmsg = unsafe {
        set_option(
            opt_idx,
            optval_copy(value),
            opt_flags,
            set_sid,
            true,
            true,
            ERRBUF.ptr().cast::<c_char>(),
            IOSIZE as size_t,
        )
    };
    assert!(errmsg.is_null());
}

/// Give an option a new value the way a script would. Takes ownership of
/// nothing: the caller keeps `value`.
///
/// Returns an untranslated error message, or null.
pub fn set_option_value(opt_idx: OptIndex, value: OptVal, opt_flags: c_int) -> *const c_char {
    assert!(opt_idx != kOptInvalid);

    // SAFETY: the option table is a plain array, and `ERRBUF` is `IOSIZE`
    // writable bytes.
    unsafe {
        if sandbox.get() > 0 && (*options.ptr())[opt_idx as usize].flags & kOptFlagSecure != 0 {
            return gettext(e_sandbox.ptr().cast::<c_char>());
        }
        set_option(
            opt_idx,
            optval_copy(value),
            opt_flags,
            0,
            false,
            true,
            ERRBUF.ptr().cast::<c_char>(),
            IOSIZE as size_t,
        )
    }
}

/// Drop a global-local option's local value, so it reads through to the
/// global one again.
pub(crate) fn unset_option_local_value(opt_idx: OptIndex) -> *const c_char {
    assert!(option_is_global_local(opt_idx));
    set_option_value(opt_idx, get_option_unset_value(opt_idx), OPT_LOCAL)
}

/// [`set_option_value`] for a name that may be one of the terminal options,
/// which are accepted and discarded rather than reported as unknown.
///
/// # Safety
///
/// `name` must be a NUL-terminated string.
pub unsafe fn set_option_value_handle_tty(
    name: *const c_char,
    opt_idx: OptIndex,
    value: OptVal,
    opt_flags: c_int,
) -> *const c_char {
    if opt_idx != kOptInvalid {
        return set_option_value(opt_idx, value, opt_flags);
    }
    // SAFETY: the caller's `name` is NUL-terminated, and `ERRBUF` is
    // `IOSIZE` writable bytes.
    unsafe {
        if is_tty_option(name) {
            return ptr::null();
        }
        snprintf(
            ERRBUF.ptr().cast::<c_char>(),
            IOSIZE as size_t,
            gettext(e_unknown_option2.ptr().cast::<c_char>()),
            name,
        );
    }
    ERRBUF.ptr().cast::<c_char>()
}

/// [`set_option_value`], reporting a rejection as an error message.
pub fn set_option_value_give_err(opt_idx: OptIndex, value: OptVal, opt_flags: c_int) {
    let errmsg = set_option_value(opt_idx, value, opt_flags);
    if !errmsg.is_null() {
        // SAFETY: `set_option_value` returns a NUL-terminated message.
        unsafe { emsg(gettext(errmsg)) };
    }
}

/// Attribute a list of options to the script currently running, for the
/// options another option's callback has just overridden on its behalf.
pub(crate) fn didset_options_sctx(opt_flags: c_int, opts: &[OptIndex]) {
    for &opt_idx in opts {
        set_option_sctx(opt_idx, opt_flags, current_sctx.get());
    }
}
