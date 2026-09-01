//! `:let` -- parsing the targets and performing the assignment.
//!
//! [`ex_let`] splits the command, [`ex_let_vars`] deals with the
//! `[a, b; rest]` unpack, and the four `ex_let_*` below it are one per kind
//! of target: a variable, an environment variable, an option and a register.
//! The last three implement the compound operators themselves and never
//! reach `set_var_lval`.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::guard::Suppress;
use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use super::*;
use crate::eval::typval::NumBuf;
use crate::option::boolean_optval;
use crate::os::cshim::gettext_owned;
use crate::types::{Failed, NUL, OptionSetFlags};

/// The compound assignment operators, as they appear before the `=`.
const OPERATORS: &CStr = c"+-*/%.";

/// The arithmetic ones, which an environment variable and a register refuse.
const ARITHMETIC: &CStr = c"+-*/%";

/// One `:let` target parser, dispatched on the sigil the target starts with.
type LetTarget =
    unsafe fn(*mut c_char, *mut typval_T, bool, *const c_char, *const c_char) -> *mut c_char;

/// The assignment's operator character: `None` when there is none, which is
/// the `op == NULL` every caller below tests for first.
///
/// # Safety
/// `op` is NULL or points at a readable byte.
unsafe fn op_char(op: *const c_char) -> Option<u8> {
    // SAFETY: the caller's obligation.
    (!op.is_null()).then(|| unsafe { *op } as u8)
}

/// Whether the operator is an arithmetic one, which an environment variable
/// and a register both refuse with E734.
fn is_arithmetic(op: Option<u8>) -> bool {
    // SAFETY: `ARITHMETIC` is a NUL-terminated literal.
    op.is_some_and(|c| !unsafe { vim_strchr(ARITHMETIC.as_ptr(), c.into()) }.is_null())
}

/// Whether what follows the target is one of the characters that may.
///
/// # Safety
/// `endchars` is NULL or NUL-terminated, and `p` is NUL-terminated.
unsafe fn ends_target(endchars: *const c_char, p: *const c_char) -> bool {
    // SAFETY: the caller's obligation; `skipwhite` stops at the NUL.
    endchars.is_null()
        || !unsafe { vim_strchr(endchars, *skipwhite(p) as uint8_t as c_int) }.is_null()
}

/// `:let`, `:const` and (with no `=`) the listing forms.
///
/// # Safety
/// `eap` is a live `:let`/`:const` command.
pub unsafe fn ex_let(eap: *mut exarg_T) {
    // SAFETY: the caller's obligation -- a live `:let`, which the
    // `do_cmdline` frame that owns the `exarg_T` outlives.
    let mut ea = unsafe { Ea::new(eap) };
    let is_const = ea.cmdidx as c_int == CMD_const as c_int;
    let mut arg = ea.arg;
    let mut var_count = 0;
    let mut semicolon = 0;
    let mut first: c_int = 1;

    // SAFETY: `arg` is the command's own NUL-terminated argument text, and
    // the two counters are live locals of this frame.
    let argend = unsafe { skip_var_list(arg, &raw mut var_count, &raw mut semicolon, false) };
    if argend.is_null() {
        return;
    }
    // SAFETY: `argend` points inside `arg`, so it is NUL-terminated too.
    let mut expr = unsafe { skipwhite(argend) };
    let concat = unsafe { cstr::starts_with(expr, b"..=") };
    let lead = unsafe { *expr } as u8;
    let has_assign = lead == b'='
        || (!unsafe { vim_strchr(OPERATORS.as_ptr(), lead.into()) }.is_null()
            && unsafe { *expr.add(1) } == b'=' as c_char);

    if !has_assign && !concat {
        // ":let" with no "=": list variables.
        // SAFETY: `arg` is NUL-terminated and every lister below walks the
        // editor's own scope dictionaries.
        let head = unsafe { *arg } as u8;
        if head == b'[' {
            emsg_static(e_invarg);
        } else if ends_excmd(c_int::from(head.cast_signed())) == 0 {
            // ":let var1 var2"
            arg = unsafe { list_arg_vars(eap, arg, &raw mut first) } as *mut c_char;
        } else if ea.skip == 0 {
            // ":let" on its own.
            const SCOPES: [ScopeLister; 7] = [
                list_glob_vars,
                list_buf_vars,
                list_win_vars,
                list_tab_vars,
                list_script_vars,
                list_func_vars,
                list_vim_vars,
            ];
            for lister in SCOPES {
                // SAFETY: `first` is a live local of this frame, and each
                // lister walks the editor's own scope dictionary.
                unsafe { lister(&raw mut first) };
            }
        }
        ea.nextcmd = unsafe { check_nextcmd(arg) };
        return;
    }

    // Assign to the target or targets, whatever produced the value. The
    // command's argument text is re-read here rather than reused from above
    // because `heredoc_get` moves it.
    let assign = |tv: *mut typval_T, op: *const c_char| {
        let a = ea.arg;
        // SAFETY: the command's own argument text, and a live value.
        let _ = unsafe { ex_let_vars(a, tv, false, semicolon, var_count, is_const, op) };
    };

    let mut rettv = TV_INITIAL_VALUE;
    // SAFETY: `expr` is NUL-terminated, so a byte past a NUL is never read.
    if lead == b'='
        && unsafe { *expr.add(1) } == b'<' as c_char
        && unsafe { *expr.add(2) } == b'<' as c_char
    {
        // A here-document.
        // SAFETY: a live command and the text past the "=<<".
        let l = unsafe { heredoc_get(eap, expr.add(3), false) };
        if !l.is_null() {
            // SAFETY: a live local and the list just built.
            unsafe { tv_list_set_ret(&raw mut rettv, l) };
            if ea.skip == 0 {
                let op = [b'=' as c_char, NUL as c_char];
                assign(&raw mut rettv, op.as_ptr());
            }
            // SAFETY: a live local.
            clear_local(&mut rettv);
        }
        return;
    }

    // The operator, if any, and the expression past it.
    let mut op = [b'=' as c_char, NUL as c_char];
    if lead != b'=' {
        // SAFETY: as above -- `expr` is NUL-terminated.
        if !unsafe { vim_strchr(OPERATORS.as_ptr(), lead.into()) }.is_null() {
            // "+=", "-=", "*=", "/=", "%=" or ".="
            op[0] = lead as c_char;
            if lead == b'.' && unsafe { *expr.add(1) } == b'.' as c_char {
                // "..=" -- one character longer than the rest.
                expr = unsafe { expr.add(1) };
            }
        }
        expr = unsafe { expr.add(2) };
    } else {
        expr = unsafe { expr.add(1) };
    }
    expr = unsafe { skipwhite(expr) };

    let skipping = (ea.skip != 0).then(Suppress::emsg_skip);
    let mut evalarg = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ptr::null_mut(),
        eval_tofree: ptr::null_mut(),
    };
    let skip = ea.skip != 0;
    // SAFETY: a live command, a live local `evalarg`, and `expr` inside the
    // command's own argument text.
    unsafe { fill_evalarg_from_eap(&raw mut evalarg, eap, skip) };
    let eval_res = unsafe { eval0(expr, &raw mut rettv, eap, &raw mut evalarg) };
    drop(skipping);
    unsafe { clear_evalarg(&raw mut evalarg, eap) };

    if ea.skip == 0 && eval_res.is_ok() {
        assign(&raw mut rettv, op.as_ptr());
    }
    if eval_res.is_ok() {
        // SAFETY: a live local.
        clear_local(&mut rettv);
    }
}

/// Assign `tv` to the target or targets at `arg_start`: one name, or the
/// `[v1, v2]` / `[v1, v2; rest]` unpack of a List.
///
/// `op` points at the characters that must follow the target(s), and names
/// the operator: `"+"`, `"-"` or `"."` for add, subtract or concatenate.
/// `semicolon` and `var_count` come from [`skip_var_list`].
///
/// # Safety
/// `arg_start` is a NUL-terminated string and `tv` a live value.
pub unsafe fn ex_let_vars(
    arg_start: *mut c_char,
    tv: *mut typval_T,
    copy: bool,
    semicolon: c_int,
    var_count: c_int,
    is_const: bool,
    op: *const c_char,
) -> Result<(), Failed> {
    let mut arg = arg_start;
    // SAFETY: the caller's obligation -- `arg` is NUL-terminated and `tv` is
    // a live value.
    if unsafe { *arg } != b'[' as c_char {
        // ":let var = expr" or ":for var in list"
        if unsafe { ex_let_one(arg, tv, copy, is_const, op, op) }.is_null() {
            return Err(Failed);
        }
        return Ok(());
    }

    // ":let [v1, v2] = list" or ":for [v1, v2] in listlist"
    // SAFETY: the caller's obligation -- a live value.
    let tv = unsafe { Tv::new(tv) };
    if tv.v_type != VAR_LIST {
        emsg_static(e_listreq);
        return Err(Failed);
    }
    // SAFETY: the type tag says the union holds the List arm, and the list
    // is the caller's for the whole walk below.
    let l = tv.list_or_null();
    let len = unsafe { tv_list_len(l) };
    if semicolon == 0 && var_count < len {
        emsg_static(c"E687: Less targets than List items");
        return Err(Failed);
    }
    if var_count - semicolon > len {
        emsg_static(c"E688: More targets than List items");
        return Err(Failed);
    }
    // `l` may really be NULL, but `:let [] = v:_null_list` fails with
    // E688 or earlier before it can get here.
    debug_assert!(!l.is_null());

    // SAFETY: a live list, whose items stay live for the walk.
    let mut item = unsafe { tv_list_first(l) };
    let mut rest_len = unsafe { tv_list_len(l) } as size_t;
    while unsafe { *arg } != b']' as c_char {
        // Skip the whitespace after the '[', ',' or ';'.
        // SAFETY: `arg` is inside the caller's NUL-terminated string, and
        // `item` is a live item of `l` -- the length checks above are what
        // keep the walk inside it.
        let (next, itv) = unsafe { (skipwhite(arg.add(1)), &raw mut (*item).li_tv) };
        arg = unsafe { ex_let_one(next, itv, true, is_const, c",;]".as_ptr(), op) };
        if arg.is_null() {
            return Err(Failed);
        }
        rest_len -= 1;
        item = unsafe { (*item).li_next };

        arg = unsafe { skipwhite(arg) };
        let sep = unsafe { *arg } as u8;
        if sep == b';' {
            // The rest of the list, which may be empty, goes to the
            // variable after the ';', as a list of its own.
            // SAFETY: the list just allocated, and the items left in `l`.
            let rest_list = unsafe { tv_list_alloc(rest_len as ptrdiff_t) };
            while !item.is_null() {
                unsafe { tv_list_append_tv(rest_list, &raw mut (*item).li_tv) };
                item = unsafe { (*item).li_next };
            }
            let mut ltv = typval_T {
                v_type: VAR_LIST,
                v_lock: VarLock::Unlocked,
                vval: typval_vval_union { v_list: rest_list },
            };
            unsafe { tv_list_ref(rest_list) };

            // SAFETY: `arg` is inside the caller's string and `ltv` a live
            // local.
            let rest_arg = unsafe { skipwhite(arg.add(1)) };
            let ltvp = &raw mut ltv;
            arg = unsafe { ex_let_one(rest_arg, ltvp, false, is_const, c"]".as_ptr(), op) };
            unsafe { tv_clear(ltvp) };
            if arg.is_null() {
                return Err(Failed);
            }
            break;
        } else if sep != b',' && sep != b']' {
            unsafe { internal_error(c"ex_let_vars()".as_ptr()) };
            return Err(Failed);
        }
    }
    Ok(())
}

/// Skip an assignable variable, or the `[var, var]` list of them, answering
/// the character past it or NULL on an error.
///
/// `var_count` counts the variables in a list and `semicolon` records
/// whether one carried a `;`.  `silent` suppresses E475.
///
/// # Safety
/// `arg` is a NUL-terminated string; `var_count` and `semicolon` are
/// writable.
pub unsafe fn skip_var_list(
    arg: *const c_char,
    var_count: *mut c_int,
    semicolon: *mut c_int,
    silent: bool,
) -> *const c_char {
    // SAFETY: the caller's obligation -- `arg` is NUL-terminated and the two
    // counters are writable locals of the caller's frame.
    if unsafe { *arg } != b'[' as c_char {
        return unsafe { skip_var_one(arg) };
    }
    // "[var, var]": find the matching ']'.
    let mut p = arg;
    loop {
        // Skip the whitespace after the '[', ';' or ','.
        p = unsafe { skipwhite(p.add(1)) };
        let s = unsafe { skip_var_one(p) };
        if s == p {
            if !silent {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let p = unsafe { c_str(p) };
                semsg!("E475: Invalid argument: {p}");
            }
            return ptr::null();
        }
        unsafe { *var_count += 1 };

        p = unsafe { skipwhite(s) };
        match unsafe { *p } as u8 {
            b']' => return unsafe { p.add(1) },
            b';' if unsafe { *semicolon } == 1 => {
                if !silent {
                    emsg_static(e_double_semicolon_in_list_of_variables);
                }
                return ptr::null();
            }
            b';' => unsafe { *semicolon = 1 },
            b',' => {}
            _ => {
                if !silent {
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let p = unsafe { c_str(p) };
                    semsg!("E475: Invalid argument: {p}");
                }
                return ptr::null();
            }
        }
    }
}

/// Skip one assignable name, including `@r`, `$VAR`, `&option`, `d.key` and
/// `l[idx]`.
///
/// # Safety
/// `arg` is a NUL-terminated string.
unsafe fn skip_var_one(arg: *const c_char) -> *const c_char {
    // SAFETY: the caller's obligation -- `arg` is NUL-terminated, so the
    // second byte is only read once the first has proved not to be one.
    let sigil = unsafe { *arg } as u8;
    if sigil == b'@' && unsafe { *arg.add(1) } != NUL as c_char {
        return unsafe { arg.add(2) };
    }
    let name = if sigil == b'$' || sigil == b'&' {
        unsafe { arg.add(1) }
    } else {
        arg
    };
    let flags = FNE_INCL_BR | FNE_CHECK_START;
    let (nil1, nil2) = (ptr::null_mut(), ptr::null_mut());
    // SAFETY: a NUL-terminated name, with neither out-parameter wanted.
    unsafe { find_name_end(name, nil1, nil2, flags) }
}

/// `:let $VAR = …`.  Answers the character past the name, or NULL.
///
/// # Safety
/// `arg` points at the `$`; `tv` is a live value.
unsafe fn ex_let_env(
    mut arg: *mut c_char,
    tv: *mut typval_T,
    is_const: bool,
    endchars: *const c_char,
    op: *const c_char,
) -> *mut c_char {
    let mut numbuf = NumBuf::new();
    if is_const {
        emsg_static(c"E996: Cannot lock an environment variable");
        return ptr::null_mut();
    }
    // SAFETY: the caller's obligation -- `op` is NULL or NUL-terminated.
    let opch = unsafe { op_char(op) };

    // Find the end of the name.
    let mut arg_end: *mut c_char = ptr::null_mut();
    // SAFETY: `arg` points at the `$` of a NUL-terminated name.
    arg = unsafe { arg.add(1) };
    let name = arg;
    let len = unsafe { get_env_len(&raw mut arg as *mut *const c_char) };
    if len == 0 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg0 = unsafe { c_str(name.sub(1)) };
        semsg!("E475: Invalid argument: {arg0}");
    } else if is_arithmetic(opch) {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let op = unsafe { c_str(op) };
        semsg!("E734: Wrong variable type for {op}=");
    } else if !unsafe { ends_target(endchars, arg) } {
        emsg_static(e_letunexp);
    } else if !check_secure() {
        // Terminate the name in place: `arg` has already moved past it.
        let mut tofree: *mut c_char = ptr::null_mut();
        // SAFETY: `len` is the length `get_env_len` measured from `name`, so
        // the byte at it is the name's own terminator or separator.
        let end = unsafe { name.offset(len as isize) };
        let c1 = unsafe { *end };
        unsafe { *end = NUL as c_char };

        // SAFETY: the caller's obligation -- `tv` is a live value.
        let mut p = unsafe { numbuf.string_chk(tv) };
        if !p.is_null() && opch == Some(b'.') {
            // SAFETY: a NUL-terminated name and value.
            let s = unsafe { vim_getenv(name) };
            if !s.is_null() {
                tofree = unsafe { concat_str(s, p) };
                p = tofree;
                unsafe { xfree(s.cast()) };
            }
        }
        if !p.is_null() {
            // SAFETY: a NUL-terminated name and value.
            unsafe { vim_setenv_ext(name, p) };
            arg_end = arg;
        }
        unsafe { *end = c1 };
        unsafe { xfree(tofree.cast()) };
    }
    arg_end
}

/// `:let &opt = …`.  Answers the character past the name, or NULL.
///
/// The compound operators are implemented here rather than through
/// `eexe_mod_op`, because an option's value is an `OptVal` and not a
/// `typval_T`: the current value is read, combined, and set back.
///
/// # Safety
/// `arg` points at the `&`; `tv` is a live value.
unsafe fn ex_let_option(
    mut arg: *mut c_char,
    tv: *mut typval_T,
    is_const: bool,
    endchars: *const c_char,
    op: *const c_char,
) -> *mut c_char {
    if is_const {
        // SAFETY: a NUL-terminated literal.
        emsg_static(c"E996: Cannot lock an option");
        return ptr::null_mut();
    }
    // SAFETY: the caller's obligation -- `op` is NULL or NUL-terminated.
    let opch = unsafe { op_char(op) };

    // Find the end of the name.
    let mut opt_idx: OptIndex = kOptAleph;
    let mut opt_flags: OptionSetFlags = OptionSetFlags::NONE;
    let namep = &raw mut arg as *mut *const c_char;
    let (idxp, flagsp) = (&raw mut opt_idx, &raw mut opt_flags);
    // SAFETY: `arg` points at the `&` of a NUL-terminated name, and the two
    // out-parameters are live locals of this frame.
    let p = unsafe { find_option_var_end(namep, idxp, flagsp) } as *mut c_char;
    if p.is_null() || !unsafe { ends_target(endchars, p) } {
        emsg_static(e_letunexp);
        return ptr::null_mut();
    }

    // Terminate the name in place; every exit below puts it back.
    let c1 = unsafe { *p };
    unsafe { *p = NUL as c_char };

    let arg_name = unsafe { CStr::from_ptr(arg) };
    let is_tty_opt = is_tty_option(arg_name);
    let hidden = is_option_hidden(opt_idx);
    let curval = if is_tty_opt {
        get_tty_option(arg_name)
    } else {
        get_option_value(opt_idx, opt_flags)
    };
    let mut newval = OptVal::Nil;
    let mut arg_end: *mut c_char = ptr::null_mut();

    'theend: {
        if curval.is_nil() {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = unsafe { c_str(arg) };
            semsg!("E355: Unknown option: {arg}");
            break 'theend;
        }
        let compound = opch.is_some_and(|c| c != b'=');
        let is_string = matches!(curval, OptVal::String(_));
        if compound && opch.is_some_and(|c| (c == b'.') != is_string) {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let op = unsafe { c_str(op) };
            semsg!("E734: Wrong variable type for {op}=");
            break 'theend;
        }

        let mut error = false;
        newval = unsafe { tv_to_optval(tv, opt_idx, arg, &raw mut error) };
        if error {
            break 'theend;
        }
        // The current and the new value must have the same type.
        debug_assert!(curval.kind() == newval.kind());

        if compound && !hidden {
            // A Number or Boolean `OptVal` as a number; a closure, so that
            // the two reads are written once. Only those two variants get
            // this far: the `if` just below is the guard that keeps a
            // String or a Nil out, and both calls are inside it.
            let as_int = |v: OptVal| -> OptInt {
                match v {
                    OptVal::Number(number) => number,
                    // The tri-state word itself, as upstream's union read
                    // of the `boolean` arm answered.
                    OptVal::Boolean(word) => OptInt::from(word),
                    OptVal::Nil | OptVal::String(_) => {
                        unreachable!("guarded to a Number or a Boolean")
                    }
                }
            };
            if matches!(curval, OptVal::Number(_) | OptVal::Boolean(_)) {
                let cur_n = as_int(curval);
                let new_n = as_int(newval);
                let new_n = match opch.unwrap_or(b'=') {
                    b'+' => cur_n + new_n,
                    b'-' => cur_n - new_n,
                    b'*' => cur_n * new_n,
                    b'/' => num_divide(cur_n as varnumber_T, new_n as varnumber_T) as OptInt,
                    b'%' => num_modulus(cur_n as varnumber_T, new_n as varnumber_T) as OptInt,
                    // No other operator reaches here: `.` was refused
                    // above for a non-String option.
                    _ => new_n,
                };
                newval = if matches!(curval, OptVal::Number(_)) {
                    OptVal::Number(new_n)
                } else {
                    boolean_optval(tristate_from_int(new_n))
                };
            } else if let (OptVal::String(cur), OptVal::String(new)) = (curval, newval) {
                // The two are Strings together, the assertion above having
                // given `newval` `curval`'s type.
                let (curval_data, newval_data) = (cur.data(), new.data());
                if !curval_data.is_null() && !newval_data.is_null() {
                    let newval_old = newval;
                    newval = OptVal::String(unsafe {
                        cstr_as_string(concat_str(curval_data, newval_data))
                    });
                    optval_free(newval_old);
                }
            }
        }

        let err = unsafe { set_option_value_handle_tty(arg, opt_idx, newval, opt_flags) };
        arg_end = p;
        if let Some(err) = err {
            emsg(&gettext_owned(&err));
        }
    }

    unsafe { *p = c1 };
    optval_free(curval);
    optval_free(newval);
    arg_end
}

/// Upstream's `TRISTATE_FROM_INT`: anything positive is true, zero is false,
/// and a negative number is "unset".
pub(crate) fn tristate_from_int(n: OptInt) -> Option<bool> {
    if n == 0 {
        Some(false)
    } else if n >= 1 {
        Some(true)
    } else {
        None
    }
}

/// `:let @r = …`.  Answers the character past the register, or NULL.
///
/// # Safety
/// `arg` points at the `@`; `tv` is a live value.
unsafe fn ex_let_register(
    mut arg: *mut c_char,
    tv: *mut typval_T,
    is_const: bool,
    endchars: *const c_char,
    op: *const c_char,
) -> *mut c_char {
    let mut numbuf = NumBuf::new();
    if is_const {
        // SAFETY: a NUL-terminated literal.
        emsg_static(c"E996: Cannot lock a register");
        return ptr::null_mut();
    }
    // SAFETY: the caller's obligation -- `op` is NULL or NUL-terminated.
    let opch = unsafe { op_char(op) };

    let mut arg_end: *mut c_char = ptr::null_mut();
    // SAFETY: `arg` points at the `@` of a NUL-terminated name.
    arg = unsafe { arg.add(1) };
    if is_arithmetic(opch) {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let op = unsafe { c_str(op) };
        semsg!("E734: Wrong variable type for {op}=");
        return arg_end;
    }
    // SAFETY: the register name is one byte, so the byte past it is inside
    // the caller's string.
    let past = unsafe { arg.add(1) };
    if !unsafe { ends_target(endchars, past) } {
        emsg_static(e_letunexp);
        return arg_end;
    }

    // A bare "@" is the unnamed register.
    // SAFETY: `arg` is inside the caller's NUL-terminated string.
    let regname = match unsafe { *arg } as u8 {
        b'@' => b'"' as c_int,
        // Sign-extended, as the C's `*arg` is: a register name is ASCII, but
        // the byte is what upstream passes on.
        c => c_int::from(c.cast_signed()),
    };
    let mut ptofree: *mut c_char = ptr::null_mut();
    // SAFETY: the caller's obligation -- `tv` is a live value.
    let mut p = unsafe { numbuf.string_chk(tv) };
    if !p.is_null() && opch == Some(b'.') {
        // SAFETY: a register name and a NUL-terminated value.
        let s = unsafe { get_reg_contents(regname, kGRegExprSrc as c_int) } as *mut c_char;
        if !s.is_null() {
            ptofree = unsafe { concat_str(s, p) };
            p = ptofree;
            unsafe { xfree(s.cast()) };
        }
    }
    if !p.is_null() {
        // SAFETY: a register name and a NUL-terminated value.
        unsafe { write_reg_contents(regname, p, cstr::bytes_at(p).len() as ssize_t, 0) };
        arg_end = past;
    }
    // SAFETY: `ptofree` is NULL or this frame's own allocation.
    unsafe { xfree(ptofree.cast()) };
    arg_end
}

/// One assignment target, dispatched on what it starts with.  Answers the
/// character past it, or NULL on an error.
///
/// # Safety
/// `arg` is a NUL-terminated string; `tv` is a live value.
unsafe fn ex_let_one(
    arg: *mut c_char,
    tv: *mut typval_T,
    copy: bool,
    is_const: bool,
    endchars: *const c_char,
    op: *const c_char,
) -> *mut c_char {
    // SAFETY: the caller's obligation -- `arg` is NUL-terminated.
    let sigil = unsafe { *arg } as u8;
    // The three sigils each have a parser of their own, which reads the
    // sigil back off `arg`.
    let target: Option<LetTarget> = match sigil {
        b'$' => Some(ex_let_env),
        b'&' => Some(ex_let_option),
        b'@' => Some(ex_let_register),
        _ => None,
    };
    if let Some(target) = target {
        // SAFETY: the caller's obligation, passed straight on.
        return unsafe { target(arg, tv, is_const, endchars, op) };
    }
    if !eval_isnamec1(c_int::from(sigil.cast_signed())) && sigil != b'{' {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str(arg) };
        semsg!("E475: Invalid argument: {arg}");
        return ptr::null_mut();
    }

    // A variable, a List or Dict item, or a Blob byte.
    let mut arg_end: *mut c_char = ptr::null_mut();
    let mut lv = LVAL_INITIAL_VALUE;
    let lvp = &raw mut lv;
    // SAFETY: the caller's obligation, and `lv` is a live local.
    let p = unsafe { get_lval(arg, tv, lvp, false, false, 0, FNE_CHECK_START) };
    if !p.is_null() && !lv.ll_name.is_null() {
        if !unsafe { ends_target(endchars, p) } {
            emsg_static(e_letunexp);
        } else {
            unsafe { set_var_lval(lvp, p, tv, copy, is_const, op) };
            arg_end = p;
        }
    }
    unsafe { clear_lval(lvp) };
    arg_end
}
