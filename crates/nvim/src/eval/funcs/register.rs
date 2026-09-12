//! Registers: `getreg()`, `setreg()`, `getreginfo()` and the
//! recording state.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::wrappers::{arg_number_chk, arg_string_chk, dict_alloc_ret};
use super::{
    YREG_YANK, kGRegExprSrc, kGRegList, kMTBlockWise, kMTCharWise, kMTLineWise, kMTUnknown,
};
use crate::ascii::ascii_isdigit;
use crate::charset::getdigits_int;
use crate::cstr;
use crate::eval::typval::{
    ListRef, NumBuf, tv_dict_add_bool, tv_dict_add_list, tv_dict_add_str, tv_dict_find,
    tv_dict_get_number, tv_dict_len, tv_get_string_buf_chk, tv_list_alloc, tv_list_iter,
    tv_list_len,
};
use crate::eval::vars::get_vim_var_str;
use crate::getchar::state::{reg_executing, reg_recorded, reg_recording};
use crate::keycodes::Ctrl_V;
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::register::{
    format_reg_type, get_reg_contents, get_reg_type, get_register_name, get_unname_register,
    get_yank_register, op_reg_set_previous, write_reg_contents_ex, write_reg_contents_lst,
};
use crate::semsg;
use crate::strings::vim_snprintf;
use crate::types::{
    BoolVarValue, ColNr, Dict, EvalFuncData, Failed, List, MotionType, NUL, TypVal, VAR_DICT,
    VAR_LIST, Vv, kBoolVarFalse, kBoolVarTrue,
};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// The buffer `format_reg_type` and `getreginfo()` build a register type in.
/// `NUMBUFLEN + 2` in the C: a CTRL-V plus the widest decimal width.
type TypeBuf = [c_char; 67];

/// Which register a builtin was asked about, or `None` if the argument was
/// not a String. An omitted argument means `v:register`, and an empty name
/// means the unnamed register.
fn regname(args: &[TypVal]) -> Option<c_int> {
    let mut numbuf = NumBuf::new();
    let name = if !args.is_empty() {
        let name = arg_string_chk(&mut numbuf, &args[0]);
        if name.is_null() {
            return None;
        }
        name
    } else {
        get_vim_var_str(Vv::Register)
    };
    Some(match unsafe { *name } {
        0 => b'"' as c_int,
        c => c as u8 as c_int,
    })
}

/// `getreg([{regname} [, 1 [, {list}]]])`.
pub fn f_getreg(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let Some(regname) = regname(args) else {
        return;
    };
    // The two flag arguments are only read when a register was named:
    // `getreg()` alone cannot have them.
    let (mut expr_src, mut return_list) = (false, false);
    if !args.is_empty() && args.len() > 1 {
        let mut error = false;
        expr_src = arg_number_chk(&args[1], Some(&mut error)) != 0;
        if !error && args.len() > 2 {
            return_list = arg_number_chk(&args[2], Some(&mut error)) != 0;
        }
        if error {
            return;
        }
    }
    let mut flags = if expr_src { kGRegExprSrc as c_int } else { 0 };
    if return_list {
        flags |= kGRegList as c_int;
        result.write_empty(VAR_LIST);
        let l = unsafe { get_reg_contents(regname, flags) } as *mut List;
        // `get_reg_contents` hands back a list at one reference, which the
        // answer takes over; an unset register gets a fresh empty one.
        // SAFETY: the register's list, whose reference this takes over.
        let held = unsafe { ListRef::owning(l) }.unwrap_or_else(|| tv_list_alloc(0));
        result.write_list(Some(held));
    } else {
        result.write_string(unsafe { get_reg_contents(regname, flags) } as *mut c_char);
    }
}

/// `getregtype([{regname}])`.
pub fn f_getregtype(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(ptr::null_mut());
    let Some(regname) = regname(args) else {
        return;
    };
    let mut reglen: ColNr = 0;
    let mut buf: TypeBuf = [0; 67];
    let reg_type = unsafe { get_reg_type(regname, &raw mut reglen) };
    unsafe { format_reg_type(reg_type, reglen, buf.as_mut_ptr(), buf.len()) };
    result.write_string(unsafe { xstrdup(buf.as_ptr()) });
}

/// `getreginfo([{regname}])`.
pub fn f_getreginfo(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let Some(mut regname) = regname(args) else {
        return;
    };
    if regname == b'@' as c_int {
        regname = b'"' as c_int;
    }
    dict_alloc_ret(result);
    let dict: *mut Dict = result.dict_or_null();
    let list = unsafe { get_reg_contents(regname, kGRegExprSrc as c_int | kGRegList as c_int) }
        as *mut List;
    // An unset register has no `regcontents`, and no other key either.
    if list.is_null() {
        return;
    }
    // SAFETY: the register's list, whose reference the dictionary takes over.
    let _ = unsafe { tv_dict_add_list(dict, c"regcontents".as_ptr(), 11, ListRef::owning(list)) };

    let mut buf: TypeBuf = [0; 67];
    let mut reglen: ColNr = 0;
    match unsafe { get_reg_type(regname, &raw mut reglen) } {
        kMTLineWise => buf[0] = b'V' as c_char,
        kMTCharWise => buf[0] = b'v' as c_char,
        kMTBlockWise => {
            let (out, cap) = (buf.as_mut_ptr(), buf.len());
            let fmt = c"%c%d".as_ptr();
            // SAFETY: `buf` is the caller's, `cap` bytes long, and the two
            // operands match the two conversions.
            unsafe { vim_snprintf(out, cap, fmt, Ctrl_V, reglen + 1) };
        }
        // `kMTUnknown` cannot come back for a register that has
        // contents, which the null check above established.
        _ => unreachable!("register {regname} has contents but no type"),
    }
    let _ = unsafe { tv_dict_add_str(dict, c"regtype".as_ptr(), 7, buf.as_ptr()) };

    // The unnamed register reports what it points at; every other one
    // reports whether it is what the unnamed register points at.
    buf[0] = get_register_name(unsafe { get_unname_register() }) as c_char;
    buf[1] = NUL as c_char;
    if regname == b'"' as c_int {
        let _ = unsafe { tv_dict_add_str(dict, c"points_to".as_ptr(), 9, buf.as_ptr()) };
    } else {
        let unnamed = regname == buf[0] as c_int;
        let flag = if unnamed { kBoolVarTrue } else { kBoolVarFalse } as BoolVarValue;
        let _ = unsafe { tv_dict_add_bool(dict, c"isunnamed".as_ptr(), 9, flag) };
    }
}

/// The single-character String the three recording-state builtins return.
fn return_register(regname: c_int, result: &mut TypVal) {
    let buf: [c_char; 2] = [regname as c_char, 0];
    // SAFETY: `buf` is NUL-terminated and outlives the copy.
    result.write_string(unsafe { xstrdup(buf.as_ptr()) });
}

/// `reg_executing()` — the register a macro is being played from.
pub fn f_reg_executing(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    return_register(reg_executing.get(), &mut *result);
}

/// `reg_recording()` — the register `q` is recording into.
pub fn f_reg_recording(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    return_register(reg_recording.get(), &mut *result);
}

/// `reg_recorded()` — the register the last recording went into.
pub fn f_reg_recorded(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    return_register(reg_recorded.get(), &mut *result);
}

/// Read a register-type letter, advancing `cursor` past the width digits a
/// blockwise type may carry.
///
/// `cursor` is left on the *last* byte consumed, not one past it, because both
/// callers step it forward themselves.
///
/// # Safety
/// `*pp` points into a NUL-terminated string.
unsafe fn get_yank_type(
    cursor: &mut *const c_char,
    yank_type: &mut MotionType,
    block_len: &mut c_int,
) -> Result<(), Failed> {
    // SAFETY throughout: the caller's obligation; `getdigits_int` only walks forward
    // and stops at the first non-digit.
    let mut p = *cursor;
    match unsafe { *p } as u8 {
        b'v' | b'c' => *yank_type = kMTCharWise,
        b'V' | b'l' => *yank_type = kMTLineWise,
        b'b' => *yank_type = kMTBlockWise,
        c if c as c_int == Ctrl_V => *yank_type = kMTBlockWise,
        _ => return Err(Failed),
    }
    if *yank_type == kMTBlockWise && ascii_isdigit(unsafe { *p.add(1) } as c_int) {
        let mut q = unsafe { p.add(1) } as *mut c_char;
        *block_len = unsafe { getdigits_int(&raw mut q, false, 0) } - 1;
        p = unsafe { q.sub(1) };
    }
    *cursor = p;
    Ok(())
}

/// `setreg({regname}, {value} [, {options}])`.
pub fn f_setreg(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    let mut numbuf4 = NumBuf::new();
    let mut numbuf5 = NumBuf::new();
    // SAFETY throughout: the arguments and `result` are live typvals; every string
    // read below is NUL-terminated and outlives its use.
    // Non-zero means "did not set anything", which is what every early
    // return leaves behind.
    result.write_number(1);
    let strregname = arg_string_chk(&mut numbuf, &args[0]);
    if strregname.is_null() {
        return;
    }
    let mut regname = match unsafe { *strregname } as u8 {
        0 | b'@' => b'"' as c_char,
        _ => unsafe { *strregname },
    };

    let mut yank_type: MotionType = kMTUnknown;
    let mut block_len: c_int = -1;
    let mut regcontents: *const TypVal = ptr::null();
    let mut pointreg: c_char = 0;

    if args[1].v_type() == VAR_DICT {
        let d = args[1].dict_or_null();
        // An empty dict clears the register outright.
        if unsafe { tv_dict_len(d) } == 0 {
            let mut empty: [*mut c_char; 2] = [ptr::null_mut(); 2];
            let lines = empty.as_mut_ptr();
            let reg = regname as c_int;
            unsafe { write_reg_contents_lst(reg, lines, false, kMTUnknown, -1) };
            return;
        }
        let di = unsafe { tv_dict_find(d, c"regcontents".as_ptr(), -1) };
        if !di.is_null() {
            regcontents = unsafe { &raw mut (*di).di_tv };
        }
        let stropt = unsafe { numbuf2.dict_string(d, c"regtype".as_ptr()) };
        if !stropt.is_null() {
            let mut p: *const c_char = stropt;
            // The type must be exactly one letter (plus a width), so
            // the byte after what was consumed has to be the
            // terminator.
            if unsafe { get_yank_type(&mut p, &mut yank_type, &mut block_len) }.is_err()
                || unsafe { *p.add(1) } != NUL as c_char
            {
                let arg0 = "value";
                semsg!("E475: Invalid value for argument {arg0}");
                return;
            }
        }
        if regname == b'"' as c_char {
            let stropt = unsafe { numbuf3.dict_string(d, c"points_to".as_ptr()) };
            if !stropt.is_null() {
                pointreg = unsafe { *stropt };
                regname = pointreg;
            }
        } else if unsafe { tv_dict_get_number(d, c"isunnamed".as_ptr()) } != 0 {
            pointreg = regname;
        }
    } else {
        regcontents = &args[1];
    }

    let mut append = false;
    let mut set_unnamed = false;
    if args.len() > 2 {
        // A dict value already carried the type; a third argument on
        // top of it is one argument too many.
        if yank_type != kMTUnknown {
            let arg0 = "setreg";
            semsg!("E118: Too many arguments for function: {arg0}");
            return;
        }
        let opts = arg_string_chk(&mut numbuf4, &args[2]);
        if opts.is_null() {
            return;
        }
        let mut p = opts;
        while unsafe { *p } != NUL as c_char {
            match unsafe { *p } as u8 {
                b'a' | b'A' => append = true,
                b'u' | b'"' => set_unnamed = true,
                // Anything else is a register type, and an
                // unrecognised one is silently ignored.
                _ => {
                    let _ = unsafe { get_yank_type(&mut p, &mut yank_type, &mut block_len) };
                }
            }
            p = unsafe { p.add(1) };
        }
    }

    if !regcontents.is_null() && unsafe { (*regcontents).v_type() } == VAR_LIST {
        let list = unsafe { (*regcontents).list_or_null() };
        unsafe { write_list(regname, list, append, yank_type, block_len) };
    } else if !regcontents.is_null() {
        let strval = unsafe { numbuf5.string_chk(&*regcontents) };
        if strval.is_null() {
            return;
        }
        let reg = regname as c_int;
        let len = unsafe { cstr::bytes_at(strval) }.len() as isize;
        unsafe { write_reg_contents_ex(reg, strval, len, append, yank_type, block_len) };
    }
    if pointreg != 0 {
        unsafe { get_yank_register(pointreg as c_int, YREG_YANK as c_int) };
    }
    result.write_number(0);
    if set_unnamed {
        unsafe { op_reg_set_previous(regname) };
    }
}

/// Write a List value into a register.
///
/// The C builds one allocation holding both the NULL-terminated array of
/// item pointers and, past it, the subset of those that had to be copied
/// out of `tv_get_string_buf_chk`'s scratch buffer. That layout is kept:
/// it is one `xmalloc`/`xfree` pair for the whole operation, and the
/// copies are freed in reverse.
///
/// # Safety
/// `l` is a List pointer or null.
unsafe fn write_list(
    regname: c_char,
    l: *mut List,
    append: bool,
    yank_type: MotionType,
    block_len: c_int,
) {
    // SAFETY: the caller's obligation. The allocation has room for
    // `len + 1` pointers of value plus `len + 1` of copies, which is the
    // most either half can need.
    let len = unsafe { tv_list_len(l) } as usize;
    let base = unsafe { xmalloc(size_of::<*mut c_char>() * (len + 1) * 2) }.cast::<*mut c_char>();
    let allocated = unsafe { base.add(len + 2) };
    let mut curval = base;
    let mut curalloc = allocated;

    let mut complete = true;
    if !l.is_null() {
        for li in tv_list_iter(unsafe { l.as_ref() }) {
            let mut buf: [c_char; 65] = [0; 65];
            let s = unsafe { tv_get_string_buf_chk(&li.li_tv, buf.as_mut_ptr()) };
            if s.is_null() {
                complete = false;
                break;
            }
            // A value that is not already a String was rendered into
            // the scratch buffer, which the next item reuses, so it is
            // copied out and the copy remembered for the free below.
            let value = if s == buf.as_ptr() {
                // SAFETY: `curalloc` is inside the copies half of the
                // allocation, which has room for one per item.
                let copy = unsafe { xstrdup(s) };
                unsafe { *curalloc = copy };
                curalloc = unsafe { curalloc.add(1) };
                copy
            } else {
                s as *mut c_char
            };
            // SAFETY: `curval` is inside the values half, which has room
            // for one per item plus the terminator.
            unsafe { *curval = value };
            curval = unsafe { curval.add(1) };
        }
    }
    if complete {
        unsafe { *curval = ptr::null_mut() };
        unsafe { write_reg_contents_lst(regname as c_int, base, append, yank_type, block_len) };
    }
    while curalloc > allocated {
        curalloc = unsafe { curalloc.sub(1) };
        unsafe { xfree((*curalloc).cast::<c_void>()) };
    }
    unsafe { xfree(base.cast::<c_void>()) };
}
