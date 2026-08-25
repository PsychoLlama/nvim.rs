//! The working directory, and 'findfunc' — the callback `:find` and `gf`
//! resolve a name through.
//!
//! A directory is remembered at three scopes (window, tab page, global),
//! and `:cd -` goes back to the previous one *at that scope*. `post_chdir`
//! is what keeps the three in step after any of them changes.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::guard::Lock;
use crate::{semsg_c, smsg_c};
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;

use crate::eval::typval::{
    callback_free, tv_clear, tv_list_copy, tv_list_find, tv_list_free, tv_list_len,
};
use crate::eval::userfunc::get_scriptlocal_funcname;
use crate::eval::{callback_call, get_copy_id, set_ref_in_callback};
use crate::ex_docmd::{ffu_cb, kCdCauseManual, prev_dir};
use crate::ex_getln::allbuf_locked;
use crate::file_search::{do_autocmd_dirchanged, vim_chdir};
use crate::fileio::shorten_fnames;
use crate::main::{
    KeyTyped, NameBuff, curbuf, current_sctx, curtab, curwin, e_cant_find_file_str_in_path,
    e_failed, e_invalid_return_type_from_findfunc, e_invarg, e_no_more_file_str_found_in_path,
    globaldir, last_chdir_reason, p_cdh, p_ffu, p_verbose,
};
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::message::{emsg, msg};
use crate::option::{cpo_has, option_last_set, option_set_callback_func};
use crate::options::kOptFindfunc;
use crate::optionstr::free_string_option;
use crate::os::cshim::gettext;
use crate::os::env::expand_env;
use crate::os::fs::os_dirname;
use crate::path::pathcmp;
use crate::types::{
    BoolVarValue, CMD_lcd, CMD_lchdir, CMD_tcd, CMD_tchdir, Callback, CdScope, CpoFlag, FAIL,
    MAXPATHL, NUL, OK, OptInt, OptionSetFlags, VAR_BOOL, VAR_LIST, VAR_STRING, VAR_UNKNOWN,
    VAR_UNLOCKED, buf_T, exarg_T, kBoolVarFalse, kBoolVarTrue, kCdScopeGlobal, kCdScopeTabpage,
    kCdScopeWindow, list_T, listitem_T, optset_T, sctx_T, size_t, typval_T,
};
use ::libc::strcmp;

/// The buffer-local 'findfunc' if it is set, and the global one otherwise.
pub(crate) unsafe fn get_findfunc_callback() -> *mut Callback {
    unsafe {
        if *(*curbuf.get()).b_p_ffu as c_int != NUL {
            &raw mut (*curbuf.get()).b_ffu_cb
        } else {
            ffu_cb.ptr()
        }
    }
}

/// Call 'findfunc' with the pattern, and answer the List it returned.
///
/// `cmdcomplete` tells the callback whether this is completion (which may
/// answer many names) or a real `:find` (which wants the one at `count`).
/// The text lock is held across the call: the callback must not edit.
pub(crate) unsafe fn call_findfunc(pat: *mut c_char, cmdcomplete: BoolVarValue) -> *mut list_T {
    unsafe {
        let saved_sctx: sctx_T = current_sctx.get();
        let mut args: [typval_T; 3] = core::mem::zeroed();
        args[0].v_type = VAR_STRING;
        args[0].v_lock = VAR_UNLOCKED;
        args[0].vval.v_string = pat;
        args[1].v_type = VAR_BOOL;
        args[1].v_lock = VAR_UNLOCKED;
        args[1].vval.v_bool = cmdcomplete;
        args[2].v_type = VAR_UNKNOWN;
        args[2].v_lock = VAR_UNLOCKED;

        let locked = Lock::text();
        // Errors are reported against the script that *set* the option, not
        // against whatever is running now.
        current_sctx.set(option_last_set(kOptFindfunc));
        let cb = get_findfunc_callback();
        let mut rettv: typval_T = core::mem::zeroed();
        rettv.v_type = VAR_UNKNOWN;
        let called = callback_call(cb, 2, &raw mut args as *mut typval_T, &raw mut rettv);
        current_sctx.set(saved_sctx);
        drop(locked);

        let mut retlist: *mut list_T = ptr::null_mut();
        if called as c_int == OK {
            if rettv.v_type as c_uint == VAR_LIST as c_uint {
                retlist = tv_list_copy(ptr::null(), rettv.vval.v_list, false, get_copy_id());
            } else {
                emsg(gettext(
                    &raw const e_invalid_return_type_from_findfunc as *const c_char,
                ));
            }
            tv_clear(&raw mut rettv);
        }
        retlist
    }
}

/// Complete a `:find` argument through 'findfunc'.
pub unsafe fn expand_findfunc(
    pat: *mut c_char,
    files: *mut *mut *mut c_char,
    num_matches: *mut c_int,
) -> c_int {
    unsafe {
        *num_matches = 0;
        *files = ptr::null_mut();
        let l = call_findfunc(pat, kBoolVarTrue);
        if l.is_null() {
            return FAIL;
        }
        let len = tv_list_len(l);
        if len == 0 {
            tv_list_free(l);
            return FAIL;
        }
        // Sized by the list length, filled only with the entries that are
        // strings — so the count answered may be smaller.
        *files = xmalloc(size_of::<*mut c_char>() * len as size_t) as *mut *mut c_char;
        let mut idx = 0;
        let mut li: *const listitem_T = (*l).lv_first;
        while !li.is_null() {
            if (*li).li_tv.v_type as c_uint == VAR_STRING as c_uint {
                *(*files).offset(idx as isize) = xstrdup((*li).li_tv.vval.v_string);
                idx += 1;
            }
            li = (*li).li_next;
        }
        *num_matches = idx;
        tv_list_free(l);
        OK
    }
}

/// Resolve the `count`'th name 'findfunc' answers for `findarg`.
///
/// `findarg` is not NUL-terminated at `findarg_len`; the byte there is
/// saved, overwritten and put back, because the caller owns a longer line.
pub(crate) unsafe fn findfunc_find_file(
    findarg: *mut c_char,
    findarg_len: size_t,
    count: c_int,
) -> *mut c_char {
    unsafe {
        let mut ret_fname: *mut c_char = ptr::null_mut();
        let saved = *findarg.add(findarg_len);
        *findarg.add(findarg_len) = NUL as c_char;

        let fname_list = call_findfunc(findarg, kBoolVarFalse);
        let fname_count = tv_list_len(fname_list);
        if fname_count == 0 {
            semsg_c!(
                gettext(&raw const e_cant_find_file_str_in_path as *const c_char),
                findarg,
            );
        } else if count > fname_count {
            semsg_c!(
                gettext(&raw const e_no_more_file_str_found_in_path as *const c_char),
                findarg,
            );
        } else {
            let li = tv_list_find(fname_list, count - 1);
            if !li.is_null() && (*li).li_tv.v_type as c_uint == VAR_STRING as c_uint {
                ret_fname = xstrdup((*li).li_tv.vval.v_string);
            }
        }
        if !fname_list.is_null() {
            tv_list_free(fname_list);
        }
        *findarg.add(findarg_len) = saved;
        ret_fname
    }
}

/// 'findfunc' changed: recompile the callback, and shorten a
/// script-local function name to its `<SNR>` form.
///
/// The generated option table holds it as an `opt_did_set_cb` fn pointer.
pub unsafe fn did_set_findfunc(args: *mut optset_T) -> *const c_char {
    unsafe {
        let buf = (*args).os_buf as *mut buf_T;
        let retval = if (*args).os_flags.has(OptionSetFlags::LOCAL) {
            option_set_callback_func((*buf).b_p_ffu, &raw mut (*buf).b_ffu_cb)
        } else {
            let r = option_set_callback_func(p_ffu.get(), ffu_cb.ptr());
            // Setting it globally without `:setglobal` clears the local one.
            if !(*args).os_flags.has(OptionSetFlags::GLOBAL) {
                callback_free(&raw mut (*buf).b_ffu_cb);
            }
            r
        };
        if retval == FAIL {
            return &raw const e_invarg as *const c_char;
        }
        let varp = (*args).os_varp as *mut *mut c_char;
        let name = get_scriptlocal_funcname(*varp);
        if !name.is_null() {
            free_string_option(*varp);
            *varp = name;
        }
        ptr::null()
    }
}

/// Mark what the global 'findfunc' callback holds, for the garbage
/// collector.
pub unsafe fn set_ref_in_findfunc(copy_id: c_int) -> bool {
    unsafe { set_ref_in_callback(ffu_cb.ptr(), copy_id, ptr::null_mut(), ptr::null_mut()) }
}

/// The directory `:cd -` would go back to, at this scope.
pub(crate) unsafe fn get_prevdir(scope: CdScope) -> *mut c_char {
    unsafe {
        match scope as c_int {
            s if s == kCdScopeTabpage as c_int => (*curtab.get()).tp_prevdir,
            s if s == kCdScopeWindow as c_int => (*curwin.get()).w_prevdir,
            _ => prev_dir.get(),
        }
    }
}

/// Record the directory just changed to at `scope`, and drop the
/// narrower scopes' overrides.
///
/// A window-local or tab-local directory that is still set would win over
/// the change that has just happened, so both are cleared. `globaldir`
/// remembers what the global directory was, so that leaving a local
/// directory can go back to it.
pub(crate) unsafe fn post_chdir(scope: CdScope, trigger_dirchanged: bool) {
    unsafe {
        xfree((*curwin.get()).w_localdir as *mut c_void);
        (*curwin.get()).w_localdir = ptr::null_mut();
        if scope as c_int >= kCdScopeTabpage as c_int {
            xfree((*curtab.get()).tp_localdir as *mut c_void);
            (*curtab.get()).tp_localdir = ptr::null_mut();
        }
        if (scope as c_int) < kCdScopeGlobal as c_int {
            let pdir = get_prevdir(scope);
            if globaldir.get().is_null() && !pdir.is_null() {
                globaldir.set(xstrdup(pdir));
            }
        }

        let mut cwd: [c_char; 4096] = [0; 4096];
        if os_dirname(&raw mut cwd as *mut c_char, MAXPATHL as size_t) != OK {
            return;
        }
        match scope as c_int {
            s if s == kCdScopeGlobal as c_int => {
                // The global directory *is* the process's now, so there is
                // nothing left to remember.
                xfree(globaldir.get() as *mut c_void);
                globaldir.set(ptr::null_mut());
            }
            s if s == kCdScopeTabpage as c_int => {
                (*curtab.get()).tp_localdir = xstrdup(&raw mut cwd as *mut c_char);
            }
            s if s == kCdScopeWindow as c_int => {
                (*curwin.get()).w_localdir = xstrdup(&raw mut cwd as *mut c_char);
            }
            // `kCdScopeInvalid`. Upstream aborts here; so does this.
            _ => unreachable!("post_chdir with an invalid scope"),
        }
        last_chdir_reason.set(ptr::null_mut());
        shorten_fnames(!cpo_has(CpoFlag::NOSYMLINKS) as c_int);
        if trigger_dirchanged {
            do_autocmd_dirchanged(&raw mut cwd as *mut c_char, scope, kCdCauseManual, false);
        }
    }
}

/// Change directory at `scope`, remembering where we came from.
///
/// The DirChangedPre autocommand fires *before* the change and may cancel
/// it by failing; that is why the `chdir` and the event are only reached
/// when the directory really differs.
pub unsafe fn changedir_func(new_dir: *mut c_char, scope: CdScope) -> bool {
    unsafe {
        if new_dir.is_null() || allbuf_locked() {
            return false;
        }
        let mut new_dir = new_dir;
        if strcmp(new_dir, c"-".as_ptr()) == 0 {
            let pdir = get_prevdir(scope);
            if pdir.is_null() {
                emsg(gettext(c"E186: No previous directory".as_ptr()));
                return false;
            }
            new_dir = pdir;
        }

        let pdir = if os_dirname(NameBuff.ptr() as *mut c_char, MAXPATHL as size_t) == OK {
            xstrdup(NameBuff.ptr() as *mut c_char)
        } else {
            ptr::null_mut()
        };

        // `:cd` with no argument means home, when 'cdhome' is set.
        if *new_dir as c_int == NUL && p_cdh.get() != 0 {
            expand_env(
                c"$HOME".as_ptr() as *mut c_char,
                NameBuff.ptr() as *mut c_char,
                MAXPATHL,
            );
            new_dir = NameBuff.ptr() as *mut c_char;
        }

        let dir_differs = pdir.is_null() || pathcmp(pdir, new_dir, -1) != 0;
        if dir_differs {
            do_autocmd_dirchanged(new_dir, scope, kCdCauseManual, true);
            if vim_chdir(new_dir) != 0 {
                emsg(gettext(&raw const e_failed as *const c_char));
                xfree(pdir as *mut c_void);
                return false;
            }
        }

        let pp: *mut *mut c_char = match scope as c_int {
            s if s == kCdScopeTabpage as c_int => &raw mut (*curtab.get()).tp_prevdir,
            s if s == kCdScopeWindow as c_int => &raw mut (*curwin.get()).w_prevdir,
            _ => prev_dir.ptr(),
        };
        xfree(*pp as *mut c_void);
        *pp = pdir;

        post_chdir(scope, dir_differs);
        true
    }
}

/// `:cd`, `:lcd`, `:tcd` and their `…chdir` spellings.
pub unsafe fn ex_cd(eap: *mut exarg_T) {
    unsafe {
        let new_dir = (*eap).arg;
        // Without 'cdhome', a bare `:cd` reports the directory instead of
        // changing it — Vi's behaviour.
        if *new_dir as c_int == NUL && p_cdh.get() == 0 {
            ex_pwd(ptr::null_mut());
            return;
        }
        let idx = (*eap).cmdidx as c_int;
        let scope = if idx == CMD_tcd as c_int || idx == CMD_tchdir as c_int {
            kCdScopeTabpage
        } else if idx == CMD_lcd as c_int || idx == CMD_lchdir as c_int {
            kCdScopeWindow
        } else {
            kCdScopeGlobal
        };
        if changedir_func(new_dir, scope) && (KeyTyped.get() || p_verbose.get() >= 5 as OptInt) {
            ex_pwd(eap);
        }
    }
}

/// `:pwd` — and with 'verbose' set, which scope the directory came from.
pub(crate) unsafe fn ex_pwd(_eap: *mut exarg_T) {
    unsafe {
        if os_dirname(NameBuff.ptr() as *mut c_char, MAXPATHL as size_t) != OK {
            emsg(gettext(c"E187: Unknown".as_ptr()));
            return;
        }
        if p_verbose.get() > 0 as OptInt {
            let context = if !last_chdir_reason.get().is_null() {
                last_chdir_reason.get()
            } else if !(*curwin.get()).w_localdir.is_null() {
                c"window".as_ptr() as *mut c_char
            } else if !(*curtab.get()).tp_localdir.is_null() {
                c"tabpage".as_ptr() as *mut c_char
            } else {
                c"global".as_ptr() as *mut c_char
            };
            smsg_c!(
                0,
                c"[%s] %s".as_ptr(),
                context,
                NameBuff.ptr() as *mut c_char,
            );
        } else {
            msg(NameBuff.ptr() as *mut c_char, 0);
        }
    }
}
