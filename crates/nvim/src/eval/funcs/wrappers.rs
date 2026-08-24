//! The dispatch layer every builtin shares: looking a name up in the
//! generated table, calling a row, and the handful of argument accessors
//! and generic wrappers the rows point at directly.
//!
//! Nothing here belongs to one family. The families themselves live in the
//! sibling modules; this is what the parent module hands them.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::Args;
use super::table::{BUILTINS, builtin_index};
use super::{
    ARENA_EMPTY, ARRAY_DICT_INIT, BASE_LAST, BASE_NONE, FCERR_NONE, FCERR_NOTMETHOD, FCERR_TOOFEW,
    FCERR_TOOMANY, FCERR_UNKNOWN, MAX_FUNC_ARGS, VIML_INTERNAL_CALL, object_data,
};
use crate::api::private::converter::{object_to_vim_take_luaref, vim_to_object};
use crate::api::private::helpers::{api_clear_error, api_free_object};
use crate::buffer::{buflist_findnr, buflist_findpat};
use crate::eval::buffer::find_buffer;
use crate::eval::typval::{tv_check_str_or_nr, tv_get_string};
use crate::eval::userfunc::get_user_func_name;
use crate::eval::vars::{cat_prefix_varname, get_user_var_name};
use crate::eval::window::find_win_by_nr_or_id;
use crate::ex_cmds::check_secure;
use crate::global_cell::GlobalCell;
use crate::guard::Suppress;
use crate::main::{
    IObuff, curbuf, curwin, e_api_error, e_invalwindow, e_toofewarg, e_toomanyarg, lastbuf, p_cpo,
    p_magic,
};
use crate::memory::{arena_finish, arena_mem_free};
use crate::message::emsg;
use crate::optionstr::empty_option;
use crate::os::cshim::{gettext, strncmp};
use crate::types::{
    Arena, Array, Error, EvalFuncData, EvalFuncDef, MsgpackRpcRequestHandler, NUL, Object,
    VAR_BOOL, VAR_FLOAT, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, buf_T, expand_T,
    float_T, kBoolVarTrue, kErrorTypeNone, kObjectTypeNil, typval_T, typval_vval_union, win_T,
};
use crate::winlayer::Win;
use crate::{semsg_c, semsg_multiline_c};
use ::libc::strlen;
use core::ffi::{c_char, c_int};
use core::{ptr, slice};

/// A cleared typval, which is what an unfilled argument slot holds.
const EMPTY_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
    vval: typval_vval_union { v_number: 0 },
};

/// The table row for the builtin `name` spells, or null if there is none.
///
/// # Safety
/// `name` is a NUL-terminated string.
pub unsafe fn find_internal_func(name: *const c_char) -> *const EvalFuncDef {
    // SAFETY: `name` is NUL-terminated, so its first `len` bytes are
    // readable. `from_raw_parts` refuses a null pointer even for an empty
    // slice, and an empty name is not a builtin anyway.
    unsafe {
        let len = strlen(name);
        let key = if len == 0 {
            &[][..]
        } else {
            slice::from_raw_parts(name.cast::<u8>(), len)
        };
        match builtin_index(key) {
            Some(row) => BUILTINS.as_ptr().add(row),
            None => ptr::null::<EvalFuncDef>(),
        }
    }
}

/// Check a call against a row's arity.
///
/// Answers the row's base-argument index for a well-formed call, or -1
/// after reporting E118/E119.
///
/// # Safety
/// `fdef` is a live table row.
pub unsafe fn check_internal_func(fdef: *const EvalFuncDef, argcount: c_int) -> c_int {
    // SAFETY: the caller's obligation; the row's name is a `'static` string
    // in the generated table.
    unsafe {
        let too_many = if argcount < (*fdef).min_argc as c_int {
            false
        } else if argcount > (*fdef).max_argc as c_int {
            true
        } else {
            return (*fdef).base_arg as c_int;
        };
        let message = if too_many {
            e_toomanyarg.as_ptr()
        } else {
            e_toofewarg.as_ptr()
        };
        semsg_c!(gettext(message), (*fdef).name);
        -1
    }
}

/// Call the builtin `fname` spells.
///
/// # Safety
/// `fname` is a NUL-terminated string; `argvars` points at an array of at
/// least `MAX_FUNC_ARGS + 1` typvals of which the first `argcount` are
/// filled; `rettv` is the cleared return value.
pub unsafe fn call_internal_func(
    fname: *const c_char,
    argcount: c_int,
    argvars: *mut typval_T,
    rettv: *mut typval_T,
) -> c_int {
    // SAFETY: the caller's obligation. Writing the terminator at `argcount`
    // is what makes `Args` total for the body about to run.
    unsafe {
        let fdef = find_internal_func(fname);
        if fdef.is_null() {
            return FCERR_UNKNOWN as c_int;
        }
        if argcount < (*fdef).min_argc as c_int {
            return FCERR_TOOFEW as c_int;
        }
        if argcount > (*fdef).max_argc as c_int {
            return FCERR_TOOMANY as c_int;
        }
        (*argvars.add(argcount as usize)).v_type = VAR_UNKNOWN;
        (*fdef).func.expect("non-null function pointer")(argvars, rettv, (*fdef).data);
        FCERR_NONE as c_int
    }
}

/// Call the builtin `fname` spells as a method: `base->fname(args)`.
///
/// The row says where the base value goes among the arguments, so this
/// builds a fresh argument array with it spliced in rather than asking the
/// body to know about methods at all.
///
/// # Safety
/// As [`call_internal_func`], plus `basetv` is a live typval.
pub unsafe fn call_internal_method(
    fname: *const c_char,
    argcount: c_int,
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    basetv: *mut typval_T,
) -> c_int {
    // SAFETY: the caller's obligation; `argv` is `MAX_FUNC_ARGS + 1` long
    // and the arity checks above bound every index written into it.
    unsafe {
        let fdef = find_internal_func(fname);
        if fdef.is_null() {
            return FCERR_UNKNOWN as c_int;
        }
        if (*fdef).base_arg as c_int == BASE_NONE as c_int {
            return FCERR_NOTMETHOD as c_int;
        }
        // The base counts as one of the arguments.
        if argcount + 1 < (*fdef).min_argc as c_int {
            return FCERR_TOOFEW as c_int;
        }
        if argcount + 1 > (*fdef).max_argc as c_int {
            return FCERR_TOOMANY as c_int;
        }

        // `base_arg` is one-based, or `BASE_LAST` for "after everything".
        let base_index = if (*fdef).base_arg as c_int == BASE_LAST as c_int {
            argcount
        } else {
            (*fdef).base_arg as c_int - 1
        };
        if argcount < base_index {
            return FCERR_TOOFEW as c_int;
        }

        let mut argv = [EMPTY_TV; MAX_FUNC_ARGS as usize + 1];
        let out = argv.as_mut_ptr();
        ptr::copy_nonoverlapping(argvars, out, base_index as usize);
        *out.add(base_index as usize) = *basetv;
        ptr::copy_nonoverlapping(
            argvars.add(base_index as usize),
            out.add(base_index as usize + 1),
            (argcount - base_index) as usize,
        );
        (*out.add(argcount as usize + 1)).v_type = VAR_UNKNOWN;

        (*fdef).func.expect("non-null function pointer")(out, rettv, (*fdef).data);
        FCERR_NONE as c_int
    }
}

/// Command-line completion over builtin function names.
///
/// The user's own functions come first, then the builtins, and `idx == 0`
/// starts the walk over. The answer for a builtin is `name(` -- or `name()`
/// when it takes no arguments -- in the shared [`IObuff`].
///
/// # Safety
/// `xp` is a live expansion context.
pub unsafe fn get_function_name(xp: *mut expand_T, idx: c_int) -> *mut c_char {
    /// How far into the builtin table the walk has got. Negative while the
    /// user's own functions are still being offered.
    static BUILTIN_IDX: GlobalCell<c_int> = GlobalCell::new(-1);

    // SAFETY: the caller's obligation; `IObuff` is the shared scratch buffer
    // and every builtin name plus three bytes fits in it.
    unsafe {
        if idx == 0 {
            BUILTIN_IDX.set(-1);
        }
        if BUILTIN_IDX.get() < 0 {
            let name = get_user_func_name(xp, idx);
            if !name.is_null() {
                // A plain global name completed after a `g:` prefix has to
                // come back with the prefix on it.
                if *name as c_int != NUL
                    && *name as u8 != b'<'
                    && strncmp(c"g:".as_ptr(), (*xp).xp_pattern, 2) == 0
                {
                    return cat_prefix_varname('g' as c_int, name);
                }
                return name;
            }
        }

        *BUILTIN_IDX.ptr() += 1;
        let key = BUILTINS[BUILTIN_IDX.get() as usize].name;
        if key.is_null() {
            return ptr::null_mut();
        }
        let key_len = strlen(key);
        let buf = IObuff.ptr();
        ptr::copy_nonoverlapping(key, buf as *mut c_char, key_len);
        (*buf)[key_len] = b'(' as c_char;
        if BUILTINS[BUILTIN_IDX.get() as usize].max_argc == 0 {
            (*buf)[key_len + 1] = b')' as c_char;
            (*buf)[key_len + 2] = NUL as c_char;
        } else {
            (*buf)[key_len + 1] = NUL as c_char;
        }
        buf as *mut c_char
    }
}

/// Command-line completion over anything an expression may name: the
/// functions above, then the user's variables.
///
/// # Safety
/// `xp` is a live expansion context.
pub unsafe fn get_expr_name(xp: *mut expand_T, idx: c_int) -> *mut c_char {
    /// How far into the variable list the walk has got. Negative while the
    /// functions are still being offered.
    static VAR_IDX: GlobalCell<c_int> = GlobalCell::new(-1);

    // SAFETY: the caller's obligation.
    unsafe {
        if idx == 0 {
            VAR_IDX.set(-1);
        }
        if VAR_IDX.get() < 0 {
            let name = get_function_name(xp, idx);
            if !name.is_null() {
                return name;
            }
        }
        *VAR_IDX.ptr() += 1;
        get_user_var_name(xp, VAR_IDX.get())
    }
}

/// Whether a builtin's first argument is "true" in the loose sense the
/// optional flags of `mode()`, `visualmode()` and friends use.
///
/// Deliberately not `tv_get_bool`: only these three types count, and
/// anything else -- a List, a Float, a missing argument -- is false rather
/// than an error.
///
/// # Safety
/// `argvars` is a live call frame's argument array.
pub(crate) unsafe fn non_zero_arg(argvars: *mut typval_T) -> bool {
    // SAFETY: the caller's obligation; each union read is guarded by the
    // type tag that names it.
    unsafe {
        let tv = &*argvars;
        match tv.v_type {
            VAR_NUMBER => tv.vval.v_number != 0,
            VAR_BOOL => tv.vval.v_bool == kBoolVarTrue,
            VAR_STRING => !tv.vval.v_string.is_null() && *tv.vval.v_string as c_int != NUL,
            _ => false,
        }
    }
}

/// A Float or a Number as a Float, reporting E808 for anything else.
///
/// # Safety
/// `tv` is a live typval.
pub(crate) unsafe fn tv_get_float_chk(tv: *const typval_T, ret_f: *mut float_T) -> bool {
    // SAFETY: the caller's obligation; each union read is guarded by the
    // type tag that names it.
    unsafe {
        match (*tv).v_type {
            VAR_FLOAT => *ret_f = (*tv).vval.v_float,
            VAR_NUMBER => *ret_f = (*tv).vval.v_number as float_T,
            _ => {
                semsg_c!(
                    c"%s".as_ptr(),
                    gettext(c"E808: Number or Float required".as_ptr()),
                );
                return false;
            }
        }
        true
    }
}

/// The body every one-argument float builtin shares. The generated table
/// puts the libm function in the row's payload.
pub unsafe fn float_op_wrapper(argvars: *mut typval_T, rettv: *mut typval_T, fptr: EvalFuncData) {
    // SAFETY: the dispatcher's argument array and return value; the row's
    // payload is the float function for exactly these rows.
    unsafe {
        let mut f: float_T = 0.0;
        (*rettv).v_type = VAR_FLOAT;
        (*rettv).vval.v_float = if tv_get_float_chk(argvars, &raw mut f) {
            fptr.float_func.expect("non-null function pointer")(f)
        } else {
            0.0
        };
    }
}

/// The body every builtin that is really an API function shares. The
/// generated table puts the RPC handler in the row's payload.
pub unsafe fn api_wrapper(argvars: *mut typval_T, rettv: *mut typval_T, fptr: EvalFuncData) {
    // SAFETY: the dispatcher's argument array and return value; `items`
    // outlives the `Array` that borrows it, and the arena owns what the
    // conversion allocates until it is freed below.
    unsafe {
        if check_secure() {
            return;
        }
        let handler: MsgpackRpcRequestHandler = *fptr.api_handler;

        let mut items = [Object {
            type_0: kObjectTypeNil,
            data: object_data { boolean: false },
        }; MAX_FUNC_ARGS as usize];
        let mut args: Array = ARRAY_DICT_INIT;
        args.capacity = MAX_FUNC_ARGS as usize;
        args.items = items.as_mut_ptr();
        let mut arena: Arena = ARENA_EMPTY;

        let frame = Args::new(argvars);
        let mut i = 0;
        while frame.has(i) {
            *args.items.add(args.size) = vim_to_object(frame.ptr(i), &raw mut arena, false);
            args.size += 1;
            i += 1;
        }

        let mut err = Error {
            type_0: kErrorTypeNone,
            msg: ptr::null_mut(),
        };
        let mut result = handler.fn_0.expect("non-null function pointer")(
            VIML_INTERNAL_CALL,
            args,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 != kErrorTypeNone {
            semsg_multiline_c!(c"emsg".as_ptr(), e_api_error.as_ptr(), err.msg,);
        } else {
            object_to_vim_take_luaref(&raw mut result, rettv, true, &raw mut err);
        }
        // Only some handlers allocate their result; the row's handler says
        // which.
        if handler.ret_alloc {
            api_free_object(result);
        }
        arena_mem_free(arena_finish(&raw mut arena));
        api_clear_error(&raw mut err);
    }
}

/// The buffer a typval names: a buffer number, or a name matched as a
/// pattern the way `:buffer` matches one.
///
/// # Safety
/// `tv` is a live typval.
pub unsafe fn tv_get_buf(tv: *mut typval_T, curtab_only: c_int) -> *mut buf_T {
    // SAFETY: the caller's obligation; the name is the string the typval
    // owns and outlives the match.
    unsafe {
        if (*tv).v_type == VAR_NUMBER {
            return buflist_findnr((*tv).vval.v_number as c_int);
        }
        if (*tv).v_type != VAR_STRING {
            return ptr::null_mut();
        }
        let name = (*tv).vval.v_string;
        // The empty string is the current buffer, `$` the last one.
        if name.is_null() || *name as c_int == NUL {
            return curbuf.get();
        }
        if *name as u8 == b'$' && *name.add(1) as c_int == NUL {
            return lastbuf.get();
        }

        // The pattern is matched with 'magic' on and 'cpoptions' empty, so
        // that neither setting can change what a buffer name means.
        let save_magic = p_magic.get();
        let save_cpo = p_cpo.get();
        p_magic.set(1);
        p_cpo.set(empty_option());
        let mut buf = buflist_findnr(buflist_findpat(
            name,
            name.add(strlen(name)),
            true,
            false,
            curtab_only != 0,
        ));
        p_magic.set(save_magic);
        p_cpo.set(save_cpo);

        // A name no buffer matches may still be a *file* name we know.
        if buf.is_null() {
            buf = find_buffer(tv);
        }
        buf
    }
}

/// [`tv_get_buf`] for a builtin's own `{buf}` argument: type-check it, then
/// resolve it silently.
///
/// # Safety
/// `tv` is a live typval.
pub unsafe fn tv_get_buf_from_arg(tv: *mut typval_T) -> *mut buf_T {
    // SAFETY: the caller's obligation.
    unsafe {
        if !tv_check_str_or_nr(tv) {
            return ptr::null_mut();
        }
        let _no_emsg = Suppress::emsg();
        tv_get_buf(tv, 0)
    }
}

/// [`tv_get_buf`] for a builtin that must report a bad buffer itself.
///
/// # Safety
/// `arg` is a live typval.
pub unsafe fn get_buf_arg(arg: *mut typval_T) -> *mut buf_T {
    // SAFETY: the caller's obligation. The guard is what makes E158 the
    // *only* message this can produce.
    unsafe {
        let no_emsg = Suppress::emsg();
        let buf = tv_get_buf(arg, 0);
        drop(no_emsg);
        if buf.is_null() {
            semsg_c!(
                gettext(c"E158: Invalid buffer name: %s".as_ptr()),
                tv_get_string(arg),
            );
        }
        buf
    }
}

/// The window a builtin's optional `{winid}` argument names, defaulting to
/// the current one. Null after reporting E957.
///
/// # Safety
/// `argvars` is a live call frame's argument array and `idx` is within it.
pub unsafe fn get_optional_window(argvars: *mut typval_T, idx: c_int) -> *mut win_T {
    // SAFETY: the caller's obligation.
    unsafe {
        if (*argvars.add(idx as usize)).v_type == VAR_UNKNOWN {
            return curwin.get();
        }
        let win = find_win_by_nr_or_id(argvars.add(idx as usize));
        if win.is_none() {
            emsg(gettext(e_invalwindow.as_ptr()));
        }
        win.map_or(ptr::null_mut(), Win::raw)
    }
}
