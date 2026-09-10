//! The dispatch layer every builtin shares: looking a name up in the
//! generated table, calling a row, and the handful of argument accessors
//! and generic wrappers the rows point at directly.
//!
//! Nothing here belongs to one family. The families themselves live in the
//! sibling modules; this is what the parent module hands them.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::table::{BUILTINS, builtin_index};
use super::{
    ARENA_EMPTY, ARRAY_DICT_INIT, FCERR_NONE, FCERR_NOTMETHOD, FCERR_TOOFEW, FCERR_TOOMANY,
    FCERR_UNKNOWN, MAX_FUNC_ARGS, VIML_INTERNAL_CALL,
};
use crate::api::private::converter::{object_to_vim_take_luaref, vim_to_object};
use crate::api::private::helpers::api_free_object;
use crate::buffer::{buflist_findpat, find_buf};
use crate::cstr;
use crate::eval::buffer::find_buffer;
use crate::eval::typval::{
    CallFrame, ListRef, NumBuf, tv_blob_alloc_ret, tv_check_str_or_nr, tv_copy, tv_dict_alloc_ret,
    tv_get_bool, tv_get_bool_chk, tv_get_lnum, tv_get_number, tv_get_number_chk, tv_list_alloc_ret,
};
use crate::eval::userfunc::get_user_func_name;
use crate::eval::vars::{cat_prefix_varname, get_user_var_name};
use crate::eval::window::find_win_by_nr_or_id;
use crate::ex_cmds::check_secure;
use crate::global_cell::GlobalCell;
use crate::guard::Suppress;
use crate::memory::{arena_finish, arena_mem_free};
use crate::message::e_invalwindow;
use crate::message::emsg;
use crate::message_fmt::c_str;
use crate::option::vars::{p_cpo, p_magic};
use crate::optionstr::empty_option;
use crate::os::cshim::gettext;
use crate::semsg;
use crate::semsg_multiline;
use crate::types::{
    Arena, Array, Blob, Error, EvalFuncData, EvalFuncDef, Expand, Failed, Float, LineNr, List,
    MsgpackRpcRequestHandler, NUL, Object, TypVal, VAR_BOOL, VAR_FLOAT, VAR_NUMBER, VAR_STRING,
    VarNumber, WrongArity, kBoolVarTrue, ptrdiff_t,
};
use crate::winlayer::{Buf, Win, last_buffer};
use core::ffi::{c_char, c_int};
use core::{ptr, slice};

// -- Reading an argument, writing a return value ----------------------------
//
// The `tv_*` entry points in `eval::typval` take raw pointers, so every
// builtin that reads an argument or fills in its return value used to pay an
// `unsafe` for it -- some three hundred sites across the families, every one
// of them discharging the same obligation: *this is one of the frame's live
// values*. A reference proves exactly that, so the promise is made once,
// here, and the call sites are ordinary checked code.
//
// Nothing below adds behaviour. Each is the C entry point with the frame's
// guarantee spelled in its signature.

/// Argument `tv` as a Number, reporting for a value that has none.
pub(crate) fn arg_number(tv: &TypVal) -> VarNumber {
    // SAFETY: a reference is a live, initialised value, which is the whole
    // of what the coercion asks for.
    unsafe { tv_get_number(tv) }
}

/// Argument `tv` as a Number.
///
/// With an `error` the failure answer is 0 and the flag is set; without one
/// it is -1, which is what makes the reading usable as a tri-state.
pub(crate) fn arg_number_chk(tv: &TypVal, error: Option<&mut bool>) -> VarNumber {
    let error = error.map_or(ptr::null_mut(), ptr::from_mut);
    // SAFETY: as [`arg_number`]; `error` is null or a live `bool`.
    unsafe { tv_get_number_chk(tv, error) }
}

/// Argument `tv` as a boolean Number: -1 when it has no numeric form.
pub(crate) fn arg_bool(tv: &TypVal) -> VarNumber {
    // SAFETY: as [`arg_number`].
    unsafe { tv_get_bool(tv) }
}

/// Argument `tv` as a boolean Number, setting `error` when it has none.
pub(crate) fn arg_bool_chk(tv: &TypVal, error: &mut bool) -> VarNumber {
    // SAFETY: as [`arg_number_chk`].
    unsafe { tv_get_bool_chk(tv, error) }
}

/// Argument `tv` as a line number, resolving `"$"` and `"."` the way
/// `line()` does.
pub(crate) fn arg_lnum(tv: &TypVal) -> LineNr {
    // SAFETY: as [`arg_number`].
    unsafe { tv_get_lnum(tv) }
}

/// Argument `tv` as a string, the empty string for a value that has none.
pub(crate) fn arg_string(buf: &mut NumBuf, tv: &TypVal) -> *const c_char {
    // SAFETY: as [`arg_number`]; a Number is formatted into `buf`, which
    // outlives the borrow the caller holds it through.
    unsafe { buf.string(tv) }
}

/// As [`arg_string`], but NULL rather than the empty string for a value that
/// has none.
pub(crate) fn arg_string_chk(buf: &mut NumBuf, tv: &TypVal) -> *const c_char {
    // SAFETY: as [`arg_string`].
    unsafe { buf.string_chk(tv) }
}

/// Copy argument `tv` into `to`, taking a reference on what it points at.
pub(crate) fn arg_copy(tv: &TypVal, to: &mut TypVal) {
    // SAFETY: both are live values; `to` is the caller's cleared return
    // value or its own local.
    unsafe { tv_copy(tv, to) }
}

/// Make `result` a fresh List of `len` items, or of unknown length for one of
/// the `kListLen*` hints. The list the builtin then fills in.
pub(crate) fn list_alloc_ret(result: &mut TypVal, len: ptrdiff_t) -> *mut List {
    tv_list_alloc_ret(result, len)
}

/// Make `result` the List `l`, which may be null for an empty one.
///
/// The answer takes a reference of its own; the caller keeps the one it had.
pub(crate) fn list_set_ret(result: &mut TypVal, l: *mut List) {
    // SAFETY: `l` is null or a list the caller holds a reference to.
    result.write_list(unsafe { ListRef::retained(l) });
}

/// Make `result` a fresh, empty Dictionary.
pub(crate) fn dict_alloc_ret(result: &mut TypVal) {
    tv_dict_alloc_ret(result)
}

/// Make `result` a fresh, empty Blob.
pub(crate) fn blob_alloc_ret(result: &mut TypVal) -> *mut Blob {
    // SAFETY: `result` is the caller's cleared return value.
    unsafe { tv_blob_alloc_ret(result) }
}

/// The table row for the builtin `name` spells, or null if there is none.
///
/// # Safety
/// `name` is a NUL-terminated string.
pub unsafe fn find_internal_func(name: *const c_char) -> *const EvalFuncDef {
    // SAFETY: `name` is NUL-terminated, so its first `len` bytes are
    // readable. `from_raw_parts` refuses a null pointer even for an empty
    // slice, and an empty name is not a builtin anyway.
    let len = unsafe { cstr::bytes_at(name) }.len();
    let key = if len == 0 {
        &[][..]
    } else {
        unsafe { slice::from_raw_parts(name.cast::<u8>(), len) }
    };
    match builtin_index(key) {
        Some(row) => unsafe { BUILTINS.as_ptr().add(row) },
        None => ptr::null::<EvalFuncDef>(),
    }
}

/// Check a call against a row's arity, reporting E118/E119 if it does not
/// fit.
///
/// # Safety
/// `fdef` is a live table row.
pub unsafe fn check_internal_func(fdef: *const EvalFuncDef, argcount: c_int) -> Result<(), Failed> {
    // SAFETY: the caller's obligation; the row's name is a `'static` string
    // in the generated table.
    let wrong = match unsafe { (*fdef).arity }.accepts(argcount.cast_unsigned() as usize) {
        Ok(()) => return Ok(()),
        Err(wrong) => wrong,
    };
    // SAFETY: the builtin's own name, NUL-terminated.
    let name = unsafe { c_str((*fdef).name) };
    match wrong {
        WrongArity::TooMany => {
            semsg!("E118: Too many arguments for function: {name}");
        }
        WrongArity::TooFew => {
            semsg!("E119: Not enough arguments for function: {name}");
        }
    }
    Err(Failed)
}

/// Call the builtin `fname` spells.
///
/// # Safety
/// `fname` is a NUL-terminated string.
pub unsafe fn call_internal_func(
    fname: *const c_char,
    args: &[TypVal],
    result: &mut TypVal,
) -> c_int {
    // SAFETY: the caller's obligation.
    let fdef = unsafe { find_internal_func(fname) };
    if fdef.is_null() {
        return FCERR_UNKNOWN as c_int;
    }
    // SAFETY: `find_internal_func` answers a live row of the generated table.
    let fdef = unsafe { &*fdef };
    match fdef.arity.accepts(args.len()) {
        Ok(()) => {}
        Err(WrongArity::TooFew) => return FCERR_TOOFEW as c_int,
        Err(WrongArity::TooMany) => return FCERR_TOOMANY as c_int,
    }
    let func = fdef.func.expect("non-null function pointer");
    func(args, result, fdef.data);
    FCERR_NONE as c_int
}

/// Call the builtin `fname` spells as a method: `base->fname(args)`.
///
/// The row says where the base value goes among the arguments, so this
/// splices it in rather than asking the body to know about methods at all.
/// The spliced frame borrows: every slot names a value the caller owns for
/// the length of the call, which is why it is `ManuallyDrop`.
///
/// # Safety
/// As [`call_internal_func`], plus `base` is a live typval.
pub unsafe fn call_internal_method(
    fname: *const c_char,
    args: &[TypVal],
    result: &mut TypVal,
    base: &mut TypVal,
) -> c_int {
    // SAFETY: the caller's obligation.
    let fdef = unsafe { find_internal_func(fname) };
    if fdef.is_null() {
        return FCERR_UNKNOWN as c_int;
    }
    // SAFETY: `find_internal_func` answers a live row of the generated table.
    let fdef = unsafe { &*fdef };
    let Some(base_index) = fdef.base_arg.index() else {
        return FCERR_NOTMETHOD as c_int;
    };
    // The base counts as one of the arguments.
    match fdef.arity.accepts(args.len() + 1) {
        Ok(()) => {}
        Err(WrongArity::TooFew) => return FCERR_TOOFEW as c_int,
        Err(WrongArity::TooMany) => return FCERR_TOOMANY as c_int,
    }
    if args.len() < base_index {
        return FCERR_TOOFEW as c_int;
    }

    let mut frame = CallFrame::<{ MAX_FUNC_ARGS as usize + 1 }>::new();
    let (before, after) = args.split_at(base_index);
    frame.extend_borrowed(before);
    // SAFETY: the caller's promise -- `base` is a live typval, named the
    // same way.
    frame.push_borrowed(&*base);
    frame.extend_borrowed(after);

    let func = fdef.func.expect("non-null function pointer");
    func(frame.args(), result, fdef.data);
    FCERR_NONE as c_int
}

/// Command-line completion over builtin function names.
///
/// The user's own functions come first, then the builtins, and `idx == 0`
/// starts the walk over. The answer for a builtin is `name(` -- or `name()`
/// when it takes no arguments -- in the expansion context's own scratch.
///
/// # Safety
/// `expand` is a live expansion context.
pub unsafe fn get_function_name(expand: *mut Expand, idx: c_int) -> *mut c_char {
    /// How far into the builtin table the walk has got. Negative while the
    /// user's own functions are still being offered.
    static BUILTIN_IDX: GlobalCell<c_int> = GlobalCell::new(-1);

    // SAFETY throughout: the caller's obligation; `xp_buf` is the context's own scratch
    // and every builtin name plus three bytes fits in it.
    if idx == 0 {
        BUILTIN_IDX.set(-1);
    }
    if BUILTIN_IDX.get() < 0 {
        let name = unsafe { get_user_func_name(expand, idx) };
        if !name.is_null() {
            // A plain global name completed after a `g:` prefix has to
            // come back with the prefix on it.
            if unsafe { *name } as c_int != NUL
                && unsafe { *name } as u8 != b'<'
                && unsafe { cstr::starts_with((*expand).xp_pattern, b"g:") }
            {
                return unsafe { cat_prefix_varname('g' as c_int, name) };
            }
            return name;
        }
    }

    BUILTIN_IDX.set(BUILTIN_IDX.get() + 1);
    let key = BUILTINS[BUILTIN_IDX.get() as usize].name;
    if key.is_null() {
        return ptr::null_mut();
    }
    let key_len = unsafe { cstr::bytes_at(key) }.len();
    let buf = unsafe { &raw mut (*expand).xp_buf };
    unsafe { ptr::copy_nonoverlapping(key, buf as *mut c_char, key_len) };
    unsafe { (*buf)[key_len] = b'(' as c_char };
    if BUILTINS[BUILTIN_IDX.get() as usize].arity.max() == Some(0) {
        unsafe { (*buf)[key_len + 1] = b')' as c_char };
        unsafe { (*buf)[key_len + 2] = NUL as c_char };
    } else {
        unsafe { (*buf)[key_len + 1] = NUL as c_char };
    }
    buf as *mut c_char
}

/// Command-line completion over anything an expression may name: the
/// functions above, then the user's variables.
///
/// # Safety
/// `expand` is a live expansion context.
pub unsafe fn get_expr_name(expand: *mut Expand, idx: c_int) -> *mut c_char {
    /// How far into the variable list the walk has got. Negative while the
    /// functions are still being offered.
    static VAR_IDX: GlobalCell<c_int> = GlobalCell::new(-1);

    // SAFETY throughout: the caller's obligation.
    if idx == 0 {
        VAR_IDX.set(-1);
    }
    if VAR_IDX.get() < 0 {
        let name = unsafe { get_function_name(expand, idx) };
        if !name.is_null() {
            return name;
        }
    }
    VAR_IDX.set(VAR_IDX.get() + 1);
    unsafe { get_user_var_name(expand, VAR_IDX.get()) }
}

/// Whether a builtin's first argument is "true" in the loose sense the
/// optional flags of `mode()`, `visualmode()` and friends use.
///
/// Deliberately not `tv_get_bool`: only these three types count, and
/// anything else -- a List, a Float, a missing argument -- is false rather
/// than an error.
pub(crate) fn non_zero_arg(tv: &TypVal) -> bool {
    match tv.v_type() {
        VAR_NUMBER => tv.number_or_zero() != 0,
        VAR_BOOL => tv.as_bool() == Some(kBoolVarTrue),
        VAR_STRING => {
            !tv.string_or_null().is_null() && unsafe { *tv.string_or_null() } as c_int != NUL
        }
        _ => false,
    }
}

/// A Float or a Number as a Float, reporting E808 for anything else.
///
/// # Safety
/// `tv` is a live typval.
pub(crate) unsafe fn tv_get_float_chk(tv: &TypVal, ret_f: *mut Float) -> bool {
    // SAFETY: the caller's obligation; each union read is guarded by the
    // type tag that names it.
    match (*tv).v_type() {
        VAR_FLOAT => unsafe { *ret_f = (*tv).float_or_zero() },
        VAR_NUMBER => unsafe { *ret_f = (*tv).number_or_zero() as Float },
        _ => {
            let msg = c"E808: Number or Float required";
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg0 = unsafe { c_str(gettext(msg).as_ptr()) };
            semsg!("{arg0}");
            return false;
        }
    }
    true
}

/// The body every one-argument float builtin shares. The generated table
/// puts the libm function in the row's payload.
///
pub fn float_op_wrapper(args: &[TypVal], result: &mut TypVal, fptr: EvalFuncData) {
    let mut f: Float = 0.0;
    result.write_empty(VAR_FLOAT);
    // SAFETY: an argument is a live value and `f` is this frame's local.
    let value = if unsafe { tv_get_float_chk(&args[0], &raw mut f) } {
        let EvalFuncData::Float(op) = fptr else {
            unreachable!("a float builtin's row carries its operation")
        };
        op.expect("non-null function pointer")(f)
    } else {
        0.0
    };
    result.write_float(value);
}

/// The body every builtin that is really an API function shares. The
/// generated table puts the RPC handler in the row's payload.
///
pub fn api_wrapper(args: &[TypVal], result: &mut TypVal, fptr: EvalFuncData) {
    // SAFETY throughout: the dispatcher's arguments and return value; `items`
    // outlives the `Array` that borrows it, and the arena owns what the
    // conversion allocates until it is freed below.
    if check_secure() {
        return;
    }
    let EvalFuncData::Api(row) = fptr else {
        unreachable!("an API builtin's row carries its handler")
    };
    let handler: MsgpackRpcRequestHandler = unsafe { *row };

    let mut items = [Object::Nil; MAX_FUNC_ARGS as usize];
    let mut array: Array = ARRAY_DICT_INIT;
    array.capacity = MAX_FUNC_ARGS as usize;
    array.items = items.as_mut_ptr();
    let mut arena: Arena = ARENA_EMPTY;

    for arg in args {
        unsafe { *array.items.add(array.size) = vim_to_object(arg, &raw mut arena, false) };
        array.size += 1;
    }

    let mut err = Error::none();
    let call = handler.fn_0.expect("non-null function pointer");
    let mem = &raw mut arena;
    // SAFETY: `array` is the Array built above and both are locals.
    let mut answer = unsafe { call(VIML_INTERNAL_CALL, array, mem, &mut err) };
    if err.is_set() {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let msg = unsafe { c_str(err.message_or_empty().as_ptr()) };
        semsg_multiline!(c"emsg", "E5555: API call: {msg}");
    } else {
        unsafe { object_to_vim_take_luaref(&raw mut answer, result, true) };
    }
    // Only some handlers allocate their result; the row's handler says
    // which.
    if handler.ret_alloc {
        unsafe { api_free_object(answer) };
    }
    unsafe { arena_mem_free(arena_finish(&raw mut arena)) };
    err.clear();
}

/// The buffer a typval names: a buffer number, or a name matched as a
/// pattern the way `:buffer` matches one.
///
/// # Safety
/// `tv` is a live typval.
pub unsafe fn tv_get_buf(tv: &TypVal, curtab_only: c_int) -> Option<Buf> {
    // SAFETY: the caller's obligation; the name is the string the typval
    // owns and outlives the match.
    if (*tv).v_type() == VAR_NUMBER {
        return find_buf((*tv).number_or_zero() as c_int);
    }
    if (*tv).v_type() != VAR_STRING {
        return None;
    }
    let name = (*tv).string_or_null();
    // The empty string is the current buffer, `$` the last one.
    if name.is_null() || unsafe { *name } as c_int == NUL {
        return Buf::current_or_none();
    }
    if unsafe { *name } as u8 == b'$' && unsafe { *name.add(1) } as c_int == NUL {
        return last_buffer();
    }

    // The pattern is matched with 'magic' on and 'cpoptions' empty, so
    // that neither setting can change what a buffer name means.
    let save_magic = p_magic.get();
    let save_cpo = p_cpo.get();
    p_magic.set(1);
    p_cpo.set(empty_option());
    let end = unsafe { name.add(cstr::bytes_at(name).len()) };
    let only = curtab_only != 0;
    let buf = unsafe { buflist_findpat(name, end, true, false, only) };
    let found = find_buf(buf);
    p_magic.set(save_magic);
    p_cpo.set(save_cpo);

    // A name no buffer matches may still be a *file* name we know.
    match found {
        Some(buf) => Some(buf),
        None => unsafe { find_buffer(tv) },
    }
}

/// [`tv_get_buf`] for a builtin's own `{buf}` argument: type-check it, then
/// resolve it silently.
///
/// # Safety
/// `tv` is a live typval.
pub unsafe fn tv_get_buf_from_arg(tv: &TypVal) -> Option<Buf> {
    // SAFETY: the caller's obligation.
    if !unsafe { tv_check_str_or_nr(tv) } {
        return None;
    }
    let _no_emsg = Suppress::emsg();
    unsafe { tv_get_buf(tv, 0) }
}

/// [`tv_get_buf`] for a builtin that must report a bad buffer itself.
///
/// # Safety
/// `arg` is a live typval.
pub unsafe fn get_buf_arg(arg: &TypVal) -> Option<Buf> {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: the caller's obligation. The guard is what makes E158 the
    // *only* message this can produce.
    let no_emsg = Suppress::emsg();
    let buf = unsafe { tv_get_buf(arg, 0) };
    drop(no_emsg);
    if buf.is_none() {
        let what = unsafe { numbuf.string(arg) };
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let what = unsafe { c_str(what) };
        semsg!("E158: Invalid buffer name: {what}");
    }
    buf
}

/// The window a builtin's optional `{winid}` argument names, defaulting to
/// the current one. Null after reporting E957.
///
/// # Safety
/// `args` is a live call frame's argument array and `idx` is within it.
pub unsafe fn get_optional_window(args: &[TypVal], idx: usize) -> Option<Win> {
    let Some(arg) = args.get(idx) else {
        return Win::current_or_none();
    };
    // SAFETY: the caller's obligation.
    let win = unsafe { find_win_by_nr_or_id(arg) };
    if win.is_none() {
        emsg(gettext(e_invalwindow));
    }
    win
}
