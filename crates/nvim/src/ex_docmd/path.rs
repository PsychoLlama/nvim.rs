//! The working directory, and 'findfunc' — the callback `:find` and `gf`
//! resolve a name through.
//!
//! A directory is remembered at three scopes (window, tab page, global),
//! and `:cd -` goes back to the previous one *at that scope*. `post_chdir`
//! is what keeps the three in step after any of them changes.
#![forbid(unsafe_code)]

use crate::cstr;
use crate::eval::callback_copy;
use crate::file_search::do_autocmd_dirchanged;
use crate::guard::Lock;
use crate::memory::ThinCString;
use crate::memory::XString;
use crate::message::emsg;
use crate::option::callback_from_option;
use crate::optionstr::did_set_optexpr;
use crate::os::cshim::gettext;
use crate::os::env::expand_env_save_opt_of;
use crate::os::fs::current_dir;
use crate::path::pathcmp_cstr;
use crate::semsg;
use crate::smsg;
use crate::types::CmdIdx;
use crate::window::DirOwner;
use crate::window::has_own_dir;
use crate::window::prev_dir_of;
use crate::window::set_own_dir;
use crate::window::set_prev_dir;
use crate::winlayer::TabPage;
use crate::winlayer::{Buf, Win};
use core::ffi::CStr;
use core::ffi::{c_int, c_uint};

use crate::eval::typval::{
    ListRef, TV_INITIAL_VALUE, callback_free, list_copy, list_find, list_iter, list_len, tv_clear,
};

use crate::eval::{callback_call, get_copy_id, set_ref_in_callback};
use crate::ex_docmd::{ffu_cb, kCdCauseManual, prev_dir};
use crate::ex_getln::allbuf_locked;
use crate::file_search::vim_chdir;

use crate::fileio::shorten_fnames;
use crate::getchar::state::KeyTyped;
use crate::message::{e_failed, e_invalid_return_type_from_findfunc, e_invarg};
use crate::option::vars::{p_cdh, p_ffu, p_verbose};
use crate::os::state::last_chdir_reason;
use crate::runtime::state::current_sctx;

use crate::message::msg;
use crate::message_fmt::msg_cstr;

use crate::option::{cpo_has, option_last_set};

use crate::options::kOptFindfunc;

use crate::optionstr::OptString;
use crate::types::{
    BoolVarValue, Callback, CdScope, CpoFlag, ExArg, Failed, NUL, OK, OptError, OptInt, OptSet,
    OptionSetFlags, ScriptCtx, TypVal, VAR_LIST, VAR_STRING, VAR_UNKNOWN, kBoolVarFalse,
    kBoolVarTrue, kCdScopeGlobal, kCdScopeTabpage, kCdScopeWindow,
};

/// A copy of the buffer-local 'findfunc' callback if the option is set
/// there, and of the global one otherwise. A copy, because calling it runs
/// user code, which may set the option again.
fn findfunc_callback() -> Callback {
    let mut cb = Callback::None;
    let buf = Buf::current();
    if buf.b_p_ffu.first_byte() != NUL as u8 {
        callback_copy(&mut cb, &buf.b_ffu_cb);
    } else {
        ffu_cb.with(|global| callback_copy(&mut cb, global));
    }
    cb
}

/// Replace `slot` with the callback `value` names: cleared for an empty
/// value, left alone (and `Err`) for one that names none.
fn set_callback_from_option(slot: &mut Callback, value: Option<Callback>) {
    callback_free(slot);
    if let Some(cb) = value {
        *slot = cb;
    }
}

/// Call 'findfunc' with the pattern, and answer the List it returned.
///
/// `cmdcomplete` tells the callback whether this is completion (which may
/// answer many names) or a real `:find` (which wants the one at `count`).
/// The text lock is held across the call: the callback must not edit.
pub(crate) fn call_findfunc(pat: &[u8], cmdcomplete: BoolVarValue) -> Option<ListRef> {
    let saved_sctx: ScriptCtx = current_sctx.get();
    // The arguments own a copy of the pattern, released when they drop.
    let args = [TypVal::string_from(pat), TypVal::Bool(cmdcomplete)];

    let locked = Lock::text();
    // Errors are reported against the script that *set* the option, not
    // against whatever is running now.
    current_sctx.set(option_last_set(kOptFindfunc));
    let mut cb = findfunc_callback();
    let mut rettv = TV_INITIAL_VALUE;
    rettv.write_empty(VAR_UNKNOWN);
    let called = callback_call(&cb, &args, &mut rettv);
    callback_free(&mut cb);
    current_sctx.set(saved_sctx);
    drop(locked);

    let mut retlist = None;
    if called as c_int == OK {
        if rettv.v_type() as c_uint == VAR_LIST as c_uint {
            // No conversion, a fresh copyID.
            retlist = rettv
                .list_shared()
                .and_then(|list| list_copy(None, list, false, get_copy_id()));
        } else {
            emsg(gettext(e_invalid_return_type_from_findfunc));
        }
        tv_clear(&mut rettv);
    }
    retlist
}

/// Complete a `:find` argument through 'findfunc': the strings in the list
/// it answers, the rest skipped.
pub(crate) fn expand_findfunc_matches(pat: &CStr) -> Result<Vec<XString>, Failed> {
    let held = call_findfunc(pat.to_bytes(), kBoolVarTrue).ok_or(Failed)?;
    if list_len(Some(&held)) == 0 {
        return Err(Failed);
    }
    Ok(list_iter(Some(&held))
        .filter_map(|li| li.li_tv.string_cstr())
        .map(XString::from_cstr)
        .collect())
}

/// Resolve the `count`'th name 'findfunc' answers for `findarg`.
///
/// Upstream is handed a pointer into a longer line with the argument's
/// length beside it, and NUL-terminates the argument in place for the call,
/// putting the byte back afterwards. A slice says how long the argument is
/// without borrowing the line's terminator, so the copy here replaces that
/// whole dance.
pub(crate) fn findfunc_find_file(findarg: &[u8], count: c_int) -> Option<ThinCString> {
    let findarg = cstr::owned(findarg);
    let mut held = call_findfunc(findarg.to_bytes(), kBoolVarFalse);
    let fname_count = list_len(held.as_deref());
    if fname_count == 0 {
        let findarg = msg_cstr(&findarg);
        semsg!("E345: Can't find file \"{findarg}\" in path");
    } else if count > fname_count {
        let findarg = msg_cstr(&findarg);
        semsg!("E347: No more file \"{findarg}\" found in path");
    } else if let Some(li) = list_find(held.as_deref_mut(), count - 1)
        && li.li_tv.v_type() as c_uint == VAR_STRING as c_uint
    {
        // The null string, which upstream's `xstrdup` crashed on, reads
        // as the empty one.
        return Some(ThinCString::from_cstr(
            li.li_tv.string_cstr().unwrap_or(c""),
        ));
    }
    None
}

/// 'findfunc' changed: recompile the callback, and shorten a
/// script-local function name to its `<SNR>` form.
///
/// The generated option table holds it as an `opt_did_set_cb` fn pointer.
pub fn did_set_findfunc(args: &mut OptSet) -> Result<(), OptError> {
    let mut buf = args.os_buf;
    let parsed = if args.os_flags.has(OptionSetFlags::LOCAL) {
        callback_from_option(buf.b_p_ffu.get().as_cstr())
            .map(|cb| set_callback_from_option(&mut buf.b_ffu_cb, cb))
    } else {
        // Parsed before the slot is lent: the parse evaluates an
        // expression, which may read the option's callback.
        let parsed = p_ffu(callback_from_option);
        parsed.map(|cb| {
            ffu_cb.with_mut(|slot| set_callback_from_option(slot, cb));
            // Setting it globally without `:setglobal` clears the local
            // one.
            if !args.os_flags.has(OptionSetFlags::GLOBAL) {
                callback_free(&mut buf.b_ffu_cb);
            }
        })
    };
    if parsed.is_err() {
        return Err((e_invarg).into());
    }
    // A script-local function name is shortened to its `<SNR>` form, as
    // 'operatorfunc' and the other expression options are.
    did_set_optexpr(args)
}

/// Mark what the global 'findfunc' callback holds, for the garbage
/// collector.
pub fn set_ref_in_findfunc(copy_id: c_int) -> bool {
    ffu_cb.with(|cb| set_ref_in_callback(cb, copy_id, None, None))
}

/// A copy of the directory a global `:cd -` goes back to.
pub(crate) fn global_prev_dir() -> Option<XString> {
    prev_dir.with(Clone::clone)
}

/// Replace the directory a global `:cd -` goes back to.
pub(crate) fn set_global_prev_dir(dir: Option<XString>) {
    prev_dir.set(dir);
}

/// Who keeps the directory at `scope`.
fn owner_of(scope: CdScope) -> DirOwner {
    match scope as c_int {
        s if s == kCdScopeTabpage as c_int => DirOwner::TabPage(TabPage::current()),
        s if s == kCdScopeWindow as c_int => DirOwner::Window(Win::current()),
        _ => DirOwner::Global,
    }
}

/// Record the directory just changed to at `scope`, and drop the
/// narrower scopes' overrides.
///
/// A window-local or tab-local directory that is still set would win over
/// the change that has just happened, so both are cleared. `globaldir`
/// remembers what the global directory was, so that leaving a local
/// directory can go back to it.
pub(crate) fn post_chdir(scope: CdScope, trigger_dirchanged: bool) {
    set_own_dir(DirOwner::Window(Win::current()), None);
    if scope as c_int >= kCdScopeTabpage as c_int {
        set_own_dir(DirOwner::TabPage(TabPage::current()), None);
    }
    if (scope as c_int) < kCdScopeGlobal as c_int
        && !has_own_dir(DirOwner::Global)
        && let Some(pdir) = prev_dir_of(owner_of(scope))
    {
        set_own_dir(DirOwner::Global, Some(pdir));
    }

    let Some(cwd) = current_dir() else {
        return;
    };
    match scope as c_int {
        s if s == kCdScopeGlobal as c_int => {
            // The global directory *is* the process's now, so there is
            // nothing left to remember.
            set_own_dir(DirOwner::Global, None);
        }
        s if s == kCdScopeTabpage as c_int => {
            set_own_dir(DirOwner::TabPage(TabPage::current()), Some(cwd.clone()));
        }
        s if s == kCdScopeWindow as c_int => {
            set_own_dir(DirOwner::Window(Win::current()), Some(cwd.clone()));
        }
        // `kCdScopeInvalid`. Upstream aborts here; so does this.
        _ => unreachable!("post_chdir with an invalid scope"),
    }
    last_chdir_reason.set(None);
    shorten_fnames(!cpo_has(CpoFlag::NOSYMLINKS) as c_int);
    if trigger_dirchanged {
        do_autocmd_dirchanged(cwd.as_cstr(), scope, kCdCauseManual, false);
    }
}

/// Change directory at `scope` to `new_dir`, remembering where we came
/// from.
///
/// The DirChangedPre autocommand fires *before* the change and may cancel
/// it by failing; that is why the `chdir` and the event are only reached
/// when the directory really differs.
pub(crate) fn change_dir(new_dir: &CStr, scope: CdScope) -> bool {
    if allbuf_locked() {
        return false;
    }
    let mut new_dir = XString::from_cstr(new_dir);
    if new_dir.as_cstr() == c"-" {
        let Some(pdir) = prev_dir_of(owner_of(scope)) else {
            emsg(gettext(c"E186: No previous directory"));
            return false;
        };
        new_dir = pdir;
    }

    let pdir = current_dir();

    // `:cd` with no argument means home, when 'cdhome' is set.
    if new_dir.is_empty() && p_cdh() {
        new_dir = expand_env_save_opt_of(c"$HOME", false);
    }

    let dir_differs = pdir
        .as_ref()
        .is_none_or(|pdir| pathcmp_cstr(pdir.as_cstr(), new_dir.as_cstr()) != 0);
    if dir_differs {
        do_autocmd_dirchanged(new_dir.as_cstr(), scope, kCdCauseManual, true);
        if vim_chdir(new_dir.as_cstr()) != 0 {
            emsg(gettext(e_failed));
            return false;
        }
    }

    set_prev_dir(owner_of(scope), pdir);
    post_chdir(scope, dir_differs);
    true
}

/// `:cd`, `:lcd`, `:tcd` and their `…chdir` spellings.
pub fn ex_cd(excmd: &mut ExArg) {
    // A copy: the DirChangedPre autocommand runs while it is still read.
    let new_dir = excmd.line.cstr_from(excmd.line.arg).to_owned();
    // Without 'cdhome', a bare `:cd` reports the directory instead of
    // changing it — Vi's behaviour.
    if new_dir.is_empty() && !p_cdh() {
        report_working_dir();
        return;
    }
    let idx = excmd.cmdidx;
    let scope = if idx == CmdIdx::tcd || idx == CmdIdx::tchdir {
        kCdScopeTabpage
    } else if idx == CmdIdx::lcd || idx == CmdIdx::lchdir {
        kCdScopeWindow
    } else {
        kCdScopeGlobal
    };
    if change_dir(&new_dir, scope) && (KeyTyped.get() || p_verbose() >= 5 as OptInt) {
        report_working_dir();
    }
}

/// `:pwd`.
pub(crate) fn ex_pwd(_excmd: &mut ExArg) {
    report_working_dir();
}

/// Say what the working directory is — and with 'verbose' set, which scope
/// it came from.
///
/// Split from the handler because a bare `:cd` reports the directory too,
/// and has its own command to answer for.
fn report_working_dir() {
    let Some(dir) = current_dir() else {
        emsg(gettext(c"E187: Unknown"));
        return;
    };
    if p_verbose() > 0 as OptInt {
        let context = if let Some(reason) = last_chdir_reason.get() {
            reason
        } else if has_own_dir(DirOwner::Window(Win::current())) {
            c"window"
        } else if has_own_dir(DirOwner::TabPage(TabPage::current())) {
            c"tabpage"
        } else {
            c"global"
        };
        let (context, dir) = (msg_cstr(context), msg_cstr(dir.as_cstr()));
        smsg!(0, "[{context}] {dir}");
    } else {
        msg(dir.as_cstr(), 0);
    }
}
