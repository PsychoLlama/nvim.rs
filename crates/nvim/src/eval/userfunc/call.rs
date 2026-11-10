//! Calling a user function: the FuncCall's whole life.
//!
//! `call_user_func` fills the `a:` and `l:` scopes of a fresh funccall,
//! evaluates the default arguments in order, runs the body through
//! `do_cmdline` and tears the scopes down again. `call_user_func_check` is
//! the guard in front of it ('maxfuncdepth', the `dict` attribute, deleted
//! functions) and `user_func_error` turns an `FCERR_*` code into the message
//! the user sees.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::charset::vim_strsize_cstr;
use crate::eval::Parsed;
use crate::ex_docmd::{DoCmdOpts, LineSource, do_cmdline};
use crate::guard::{Depth, Lock, Suppress};
use crate::message::cut_middle;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::smsg;
use core::ffi::{CStr, c_int};
use std::rc::Rc;

use super::*;
use crate::eval::typval::DictRef;
use crate::lua::executor::typval_call_lua;
use crate::runtime::sourcing_name_bytes;
use crate::types::{Failed, ProfTime, Refcount};

/// The function of the call `frame` was made from, if it was made from one.
fn caller_func(frame: &FuncCall) -> Option<Rc<UserFunc>> {
    let caller = frame.id.caller()?;
    caller.try_funccall().map(|caller| caller.func.clone())
}

/// Run `body` inside a `:verbose` report frame: no wait-return, scrolled,
/// and terminated with a newline.
fn verbose_report(body: impl FnOnce()) {
    let _no_prompt = Suppress::wait_return();
    verbose_enter_scroll();
    body();
    msg_str(c"\n");
    verbose_leave_scroll();
}

/// `text` as the `:verbose` reports quote a value: cut to `MSG_BUF_CLEN`
/// cells.
fn quoted_value(text: &CStr) -> Vec<u8> {
    if vim_strsize_cstr(text) > MSG_BUF_CLEN {
        cut_middle(
            text.to_bytes(),
            MSG_BUF_CLEN,
            usize::try_from(MSG_BUF_LEN).unwrap_or(0),
        )
    } else {
        text.to_bytes().to_vec()
    }
}

/// The innermost exec-stack entry's name, for a message.
fn sourcing_name_text() -> String {
    msg_bytes(&sourcing_name_bytes().unwrap_or_default()).to_string()
}

/// Put `item` in `dict`; a key already there keeps the old entry and the
/// item goes, upstream's `hash_add` refusal (which reports E685 itself).
fn add_fixed(dict: &DictRef, item: Box<DictItem>) {
    if let Err(refused) = dict.edit().insert(item) {
        drop(refused);
    }
}

/// Call the user function `func`.
pub(crate) fn call_user_func(
    func: &Rc<UserFunc>,
    args: &[TypVal],
    result: &mut TypVal,
    firstline: LineNr,
    lastline: LineNr,
    selfdict: Option<&DictRef>,
) {
    static DEPTH: GlobalCell<c_int> = GlobalCell::new(0);

    // Don't execute the function when the call depth is getting too high.
    if OptInt::from(DEPTH.get()) >= p_mfd() {
        let deep = c"E132: Function call depth is higher than 'maxfuncdepth'";
        emsg(gettext(deep));
        result.write_number(-1);
        return;
    }
    let call_depth = Depth::of(&DEPTH);

    // Save the search patterns and the redo buffer.
    let mut save_redo = SaveRedo::default();
    let mut did_save_redo = false;
    save_search_patterns();
    if !ins_compl_active() {
        save_redobuff(&mut save_redo);
        did_save_redo = true;
    }
    func.calls.set(func.calls.get() + 1);
    line_breakcheck(); // check for CTRL-C hit

    let frame = open_frame(func, result);
    let body = func.body();
    let islambda = func.name().as_bytes().starts_with(b"<lambda>");
    let scopes = &frame.scopes;

    // Init the l: variables. From here a lookup sees the function's scopes.
    frame.scope_ready.set(true);
    let has_args = !func.has_flag(FuncFlags::NOARGS);
    set_fixed_vars(
        scopes,
        selfdict,
        has_args,
        args.len(),
        &body,
        firstline,
        lastline,
    );
    let bound = bind_arguments(scopes, &body, args, islambda, has_args);

    // Don't redraw while executing the function.
    let redraw_off = Suppress::redraw();

    let sandboxed = func.has_flag(FuncFlags::SANDBOX).then(Lock::sandbox);

    estack_push_ufunc(func, LineNr::ONE);
    if p_verbose() >= 12 {
        report_calling(args);
    }

    let mut profile = CallProfile::start(func, &frame);

    let save_current_sctx = CURRENT_SCTX.get();
    CURRENT_SCTX.set(func.script_ctx.get());
    let save_did_emsg = DID_EMSG.get();
    DID_EMSG.set(0);

    if bound.default_failed && (func.has_flag(FuncFlags::ABORT) || TRYLEVEL.get() > 0) {
        DID_EMSG.set(1);
    } else if islambda {
        run_lambda_body(&frame, &body);
    } else {
        // Call do_cmdline() to execute the lines.
        let opts = DoCmdOpts::NOWAIT | DoCmdOpts::VERBOSE | DoCmdOpts::REPEAT;
        let _ = do_cmdline(None, Some(&mut LineSource::Function(&frame)), opts);
    }

    // Invoke functions added with `:defer`.
    handle_defer_one(&frame);

    drop(redraw_off);

    // The return value is the caller's from here.
    *result = frame.rettv.replace(TypVal::Unknown);
    // When the function was aborted because of an error, return -1.
    if (DID_EMSG.get() != 0 && func.has_flag(FuncFlags::ABORT)) || result.v_type() == VAR_UNKNOWN {
        tv_clear(result);
        result.write_number(-1);
    }

    profile.finish(func, &frame);

    if p_verbose() >= 12 {
        report_returning(result);
    }

    estack_pop();
    CURRENT_SCTX.set(save_current_sctx);
    profile.restore_script();
    drop(sandboxed);

    if p_verbose() >= 12 && sourcing_name_bytes().is_some() {
        verbose_report(|| {
            let name = sourcing_name_text();
            smsg!(0, "continuing in {name}");
        });
    }

    DID_EMSG.set(DID_EMSG.get() | save_did_emsg);
    // The tear-down below is the caller's depth, not this call's.
    drop(call_depth);
    clear_defaults(scopes, &bound.defaults);
    cleanup_function_call(frame);

    func.calls.set(func.calls.get() - 1);
    // Free the function when it was deleted while it was running.
    if func.calls.get() <= 0 && func.refcount.get() <= 0 {
        func_clear_free(func, false);
    }

    if did_save_redo {
        restore_redobuff(&mut save_redo);
    }
    restore_search_patterns();
}

/// A fresh funccall for `func`, its return value seeded with the caller's.
fn open_frame(func: &Rc<UserFunc>, result: &mut TypVal) -> Rc<FuncCall> {
    let frame = create_funccal(func);
    // The funccall's return value starts as what the caller put in its own
    // (upstream's `fc_rettv` *is* the caller's): a plain `:return` leaves it.
    drop(frame.rettv.replace(result.take()));
    frame.breakpoint.set(dbg_find_breakpoint_named(
        false,
        func.name().as_cstr(),
        LineNr::ZERO,
    ));
    frame.dbg_tick.set(DEBUG_TICK.get());
    frame
}

/// The variables every call has whatever its arguments: `l:self` for a
/// dictionary call and, unless the body is known to use none of them, `a:0`,
/// `a:000`, `a:firstline` and `a:lastline`. Locks `a:` against new entries.
fn set_fixed_vars(
    scopes: &FrameScopes,
    selfdict: Option<&DictRef>,
    has_args: bool,
    argcount: usize,
    body: &FuncBody,
    firstline: LineNr,
    lastline: LineNr,
) {
    if let Some(selfdict) = selfdict {
        // Set l:self to "selfdict"; it takes a reference of its own.
        let value = TypVal::dict(Some(selfdict.clone()));
        add_fixed(
            &scopes.l_vars,
            scopes.entry(b"self", value, VarLock::Unlocked),
        );
    }

    if has_args {
        // Set a:0 to the number of arguments past the declared ones.
        let extra =
            VarNumber::try_from(argcount.saturating_sub(body.args.len())).unwrap_or(VarNumber::MAX);
        let item = scopes.entry(b"0", TypVal::Number(extra), VarLock::Fixed);
        add_fixed(&scopes.a_vars, item);
    }
    scopes.a_vars.edit().dv_lock = VarLock::Fixed;
    if has_args {
        // Set a:000 to the list of the extra arguments. The entry *names*
        // the funccall's own list without counting a reference, as
        // upstream's did: `cleanup_function_call` compares the count
        // against the seed to find out whether the list escaped.
        let a000 = TypVal::list(Some(scopes.a_list.clone()));
        add_fixed(&scopes.a_vars, scopes.entry(b"000", a000, VarLock::Fixed));
        scopes.a_list.edit().lv_refcount = Refcount::new(DO_NOT_FREE_CNT);
        // Set a:firstline and a:lastline.
        let first = TypVal::Number(VarNumber::from(firstline));
        add_fixed(
            &scopes.a_vars,
            scopes.entry(b"firstline", first, VarLock::Fixed),
        );
        let last = TypVal::Number(VarNumber::from(lastline));
        add_fixed(
            &scopes.a_vars,
            scopes.entry(b"lastline", last, VarLock::Fixed),
        );
    }
}

/// What binding the arguments left for the rest of the call.
struct BoundArguments {
    /// A default expression failed to evaluate; the binding stopped there.
    default_failed: bool,
    /// The defaults evaluated for this call: (in `l:`, name). Upstream
    /// clears their values before the scopes are torn down.
    defaults: Vec<(bool, Vec<u8>)>,
}

/// Bind the call's arguments: each declared parameter (its default
/// evaluated when no argument was given for it), then `a:1`, `a:2`, … for the
/// extra ones, which also go into `a:000`. The order is important: the
/// parameters are named first, so that a default expression may refer to
/// the parameter to its left.
fn bind_arguments(
    scopes: &FrameScopes,
    body: &FuncBody,
    args: &[TypVal],
    islambda: bool,
    has_args: bool,
) -> BoundArguments {
    let argcount = c_int::try_from(args.len()).unwrap_or(c_int::MAX);
    let declared = c_int::try_from(body.args.len()).unwrap_or(c_int::MAX);
    let mut bound = BoundArguments {
        default_failed: false,
        defaults: Vec::new(),
    };
    let mut i = 0;
    while i < argcount || i < declared {
        let mut addlocal = false;
        let mut isdefault = false;
        let mut def_rettv = TV_INITIAL_VALUE;
        let name: Vec<u8>;
        let at = usize::try_from(i).unwrap_or(0);

        // "ai" is the index in "argvars" past the declared arguments.
        let ai = i - declared;
        if ai < 0 {
            // A declared argument: use its name.
            name = body.args[at].to_vec();
            if islambda {
                addlocal = true;
            }
            // Evaluate the default expression when there is one and no
            // argument was given for it.
            let defaults = c_int::try_from(body.def_args.len()).unwrap_or(c_int::MAX);
            isdefault = ai + defaults >= 0 && i >= argcount;
            if isdefault {
                def_rettv.write_number(-1);
                let default = &body.def_args[usize::try_from(ai + defaults).unwrap_or(0)];
                if eval1(&mut Cursor::new(default), &mut def_rettv, true).is_err() {
                    bound.default_failed = true;
                    break;
                }
            }
        } else {
            if !has_args {
                break;
            }
            // An extra argument: a:1, a:2, ...
            name = (ai + 1).to_string().into_bytes();
        }

        // Note: the argument is not copied, so its value is shared with the
        // caller's -- the `a:` item *names* it for the length of the call,
        // and gives it up unreleased at the end. A default's value is this
        // call's own and moves in; it is cleared before the tear-down.
        let value = if isdefault {
            def_rettv.take()
        } else {
            // The caller keeps the value for the length of the call, and
            // the `a:` item releases nothing: `cleanup_function_call` gives
            // it up, or upgrades it to a copy of its own when the scope
            // outlives the call.
            args[at].named_for_call()
        };
        let mut item = scopes.entry(&name, value, VarLock::Fixed);
        if isdefault {
            bound.defaults.push((addlocal, name));
        }

        if addlocal {
            // A lambda sees its arguments as l: variables, each a value of
            // its own.
            let owned = item.di_tv.clone();
            item.di_tv.overwrite(owned);
            add_fixed(&scopes.l_vars, item);
        } else {
            add_fixed(&scopes.a_vars, item);
        }

        if (0..MAX_FUNC_ARGS).contains(&ai) {
            // Add the extra argument to a:000. As `a:name` above, the item
            // *names* the caller's value; `List::disown_items` gives it up.
            let value = args[at].named_for_call();
            scopes.a_list.edit().lv_items.push(ListItem {
                li_tv: value,
                li_lock: VarLock::Fixed,
            });
        }
        i += 1;
    }
    bound
}

/// The `:verbose` (12) report of a call, with its arguments at 14.
fn report_calling(args: &[TypVal]) {
    verbose_report(|| {
        let called = sourcing_name_text();
        smsg!(0, "calling {called}");
        if p_verbose() >= 14 {
            msg_str(c"(");
            for (i, tv) in args.iter().enumerate() {
                if i > 0 {
                    msg_str(c", ");
                }
                if tv.v_type() == VAR_NUMBER {
                    msg_outnum(c_int::try_from(tv.number_or_zero()).unwrap_or(c_int::MAX));
                } else {
                    // Do not want errors such as E724 here.
                    let rendered = {
                        let _no_emsg = Suppress::emsg();
                        encode_tv2string(tv)
                    };
                    crate::cstr::with_terminated(&quoted_value(rendered.as_cstr()), msg_str);
                }
            }
            msg_str(c")");
        }
    });
}

/// The `:verbose` (12) report of what a call returned.
fn report_returning(result: &TypVal) {
    verbose_report(|| {
        let name = sourcing_name_text();
        if aborting() {
            smsg!(0, "{name} aborted");
        } else if result.v_type() == VAR_NUMBER {
            let n = result.number_or_zero();
            smsg!(0, "{name} returning #{}", n);
        } else {
            // Do not want errors such as E724 here.
            let rendered = {
                let _no_emsg = Suppress::emsg();
                encode_tv2string(result)
            };
            let text = msg_bytes(&quoted_value(rendered.as_cstr())).to_string();
            smsg!(0, "{name} returning {text}");
        }
    });
}

/// A lambda's body is one line, "return <expr>": evaluate the expression
/// straight rather than going through `do_cmdline`.
fn run_lambda_body(frame: &FuncCall, body: &FuncBody) {
    let line = body
        .lines
        .first()
        .and_then(Option::as_deref)
        .unwrap_or_default();
    let expr = line.get(c"return ".count_bytes()..).unwrap_or_default();
    let _nesting = Depth::of(EX_NESTING_LEVEL);
    // Into what the caller left there, as upstream evaluates into the
    // caller's own value.
    let mut value = frame.rettv.replace(TypVal::Unknown);
    let _ = eval1(&mut Cursor::new(expr), &mut value, true);
    // Over whatever a `:return` run from inside the expression left.
    let old = frame.rettv.replace(value);
    drop(old);
}

/// The profiler's view of one call: what `:profile` was doing when it
/// started, and the clocks it started.
struct CallProfile {
    /// `:profile` is on (`PROF_YES`).
    profiling: bool,
    /// This call turned on profiling for `func` (`:profile func` named it).
    started: bool,
    /// `func` or its caller is profiled, so the call is timed.
    timed: bool,
    call_start: ProfTime,
    wait_start: ProfTime,
}

impl CallProfile {
    /// Start the clocks for a call of `func` in `frame`.
    fn start(func: &Rc<UserFunc>, frame: &FuncCall) -> Self {
        let profiling = DO_PROFILING.get() == PROF_YES;
        let mut started = false;
        if profiling
            && !func.prof.borrow().profiling
            && has_profiling_named(false, func.name().as_cstr())
        {
            started = true;
            func_do_profile(func);
        }
        let timed = profiling
            && (func.prof.borrow().profiling
                || caller_func(frame).is_some_and(|caller| caller.prof.borrow().profiling));
        let mut call_start = 0;
        let mut wait_start = 0;
        if timed {
            let mut prof = func.prof.borrow_mut();
            prof.tm_count += 1;
            call_start = profile_start();
            prof.tm_children = profile_zero();
        }
        if profiling {
            wait_start = script_prof_save();
        }
        Self {
            profiling,
            started,
            timed,
            call_start,
            wait_start,
        }
    }

    /// Stop the call's clock and charge it to `func` and its caller.
    fn finish(&mut self, func: &UserFunc, frame: &FuncCall) {
        if !self.timed {
            return;
        }
        self.call_start = profile_end(self.call_start);
        self.call_start = profile_sub_wait(self.wait_start, self.call_start);
        let call_time = self.call_start;
        {
            let mut prof = func.prof.borrow_mut();
            prof.tm_total = profile_add(prof.tm_total, call_time);
            prof.tm_self = profile_self(prof.tm_self, call_time, prof.tm_children);
        }
        if let Some(caller) = caller_func(frame)
            && caller.prof.borrow().profiling
        {
            let mut prof = caller.prof.borrow_mut();
            prof.tm_children = profile_add(prof.tm_children, call_time);
            prof.tml_children = profile_add(prof.tml_children, call_time);
        }
        if self.started {
            // Make a `:profdel func` stop profiling the function.
            func.prof.borrow_mut().profiling = false;
        }
    }

    /// Give the script's profile back the time the call waited.
    fn restore_script(&self) {
        if self.profiling {
            script_prof_restore(self.wait_start);
        }
    }
}

/// Clear the values of the defaults this call evaluated, as upstream does
/// before the scopes are torn down.
fn clear_defaults(scopes: &FrameScopes, defaults: &[(bool, Vec<u8>)]) {
    for (local, name) in defaults {
        let scope = if *local {
            &scopes.l_vars
        } else {
            &scopes.a_vars
        };
        // Taken out and cleared with the dictionary's borrow over, then put
        // back as `tv_clear` leaves it.
        let taken = scope.edit().find_mut(name).map(|item| item.di_tv.take());
        if let Some(mut value) = taken {
            tv_clear(&mut value);
            if let Some(item) = scope.edit().find_mut(name) {
                item.di_tv = value;
            }
        }
    }
}

/// The guard in front of [`call_user_func`]: a Lua reference is called
/// directly, and a wrong argument count or a missing `self` dictionary is
/// answered as an `FCERR_*` code rather than a call.
pub(crate) fn call_user_func_check(
    func: &Rc<UserFunc>,
    args: &[TypVal],
    result: &mut TypVal,
    with: &mut CallWith<'_>,
    selfdict: Option<&DictRef>,
) -> c_int {
    if func.has_flag(FuncFlags::LUAREF) {
        return typval_exec_lua_callable(func.luaref.get(), args, result);
    }

    if func.has_flag(FuncFlags::RANGE)
        && let Some(doesrange) = with.doesrange.as_deref_mut()
    {
        *doesrange = true;
    }
    let argcount = c_int::try_from(args.len()).unwrap_or(c_int::MAX);
    let error = check_user_func_argcount(func, argcount);
    if error != FCERR_UNKNOWN {
        return error;
    }
    if func.has_flag(FuncFlags::DICT) && selfdict.is_none() {
        return FCERR_DICT;
    }

    let dict = selfdict.filter(|_| func.has_flag(FuncFlags::DICT));
    call_user_func(func, args, result, with.firstline, with.lastline, dict);
    FCERR_NONE
}

/// Report why a call could not be made.
pub(crate) fn user_func_error(error: c_int, name: &[u8], found_var: bool) {
    let template = match error {
        FCERR_UNKNOWN if found_var => {
            let name = msg_bytes(name);
            semsg!("E1085: Not a callable type: {name}");
            return;
        }
        FCERR_UNKNOWN => E_UNKNOWN_FUNCTION_STR,
        FCERR_NOTMETHOD => c"E276: Cannot use function as a method: %s",
        FCERR_DELETED => c"E933: Function was deleted: %s",
        FCERR_TOOMANY => E_TOOMANYARG,
        FCERR_TOOFEW => E_TOOFEWARG,
        FCERR_SCRIPT => c"E120: Using <SID> not in a script context: %s",
        FCERR_DICT => c"E725: Calling dict function without Dictionary: %s",
        _ => return,
    };
    emsg_funcname(template, name);
}

/// Call the Lua function `name` with no arguments.
pub fn call_simple_luafunc(name: &[u8], result: &mut TypVal) -> Result<(), Failed> {
    // the default is number zero
    result.write_number(0);
    typval_call_lua(name, &[], result);
    Ok(())
}

/// Call a user function by name with no arguments, for the internal callers
/// that know there is nothing else to pass. Answers [`Parsed::NotThis`]
/// when there is no such function.
pub fn call_simple_func(funcname: &[u8], result: &mut TypVal) -> Result<Parsed, Failed> {
    let mut ret = Err(Failed);
    // the default is number zero
    result.write_number(0);

    let (fname, mut error) = fname_trans_sid(funcname);

    // Skip "g:" before a function name.
    let rfname = fname.strip_prefix(b"g:").unwrap_or(&fname);
    match find_func(rfname) {
        None => ret = Ok(Parsed::NotThis),
        Some(func) if func.has_flag(FuncFlags::DELETED) => error = FCERR_DELETED,
        Some(func) => {
            let mut with = CallWith::new(true);
            error = call_user_func_check(&func, &[], result, &mut with, None);
            if error == FCERR_NONE {
                ret = Ok(Parsed::Done);
            }
        }
    }

    user_func_error(error, funcname, false);
    ret
}
