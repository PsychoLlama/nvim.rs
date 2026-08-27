//! Registers: `getreg()`, `setreg()`, `getreginfo()` and the
//! recording state.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::{Args, frame};
use super::wrappers::{arg_number_chk, arg_string_chk, dict_alloc_ret};
use super::{
    YREG_YANK, kGRegExprSrc, kGRegList, kMTBlockWise, kMTCharWise, kMTLineWise, kMTUnknown,
};
use crate::ascii::ascii_isdigit;
use crate::charset::getdigits_int;
use crate::eval::typval::{
    NumBuf, tv_dict_add_bool, tv_dict_add_list, tv_dict_add_str, tv_dict_find, tv_dict_get_number,
    tv_dict_len, tv_get_string_buf_chk, tv_list_alloc, tv_list_len, tv_list_ref,
};
use crate::eval::vars::get_vim_var_str;
use crate::keycodes::Ctrl_V;
use crate::main::{e_invargval, e_toomanyarg, reg_executing, reg_recorded, reg_recording};
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::os::cshim::gettext;
use crate::register::{
    format_reg_type, get_reg_contents, get_reg_type, get_register_name, get_unname_register,
    get_yank_register, op_reg_set_previous, write_reg_contents_ex, write_reg_contents_lst,
};
use crate::semsg_c;
use crate::strings::vim_snprintf;
use crate::types::{
    BoolVarValue, EvalFuncData, FAIL, MotionType, NUL, OK, VAR_DICT, VAR_LIST, VAR_STRING, Vv,
    colnr_T, dict_T, kBoolVarFalse, kBoolVarTrue, list_T, listitem_T, typval_T,
};
use ::libc::strlen;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// The buffer `format_reg_type` and `getreginfo()` build a register type in.
/// `NUMBUFLEN + 2` in the C: a CTRL-V plus the widest decimal width.
type TypeBuf = [c_char; 67];

/// Which register a builtin was asked about, or `None` if the argument was
/// not a String. An omitted argument means `v:register`, and an empty name
/// means the unnamed register.
///
/// # Safety
/// `args.ptr(0)` is a live typval.
unsafe fn regname(args: Args<'_>) -> Option<c_int> {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: the caller's obligation; both sources are NUL-terminated.
    let name = if args.has(0) {
        let name = arg_string_chk(&mut numbuf, args.get(0));
        if name.is_null() {
            return None;
        }
        name
    } else {
        unsafe { get_vim_var_str(Vv::Register) }
    };
    Some(match unsafe { *name } {
        0 => b'"' as c_int,
        c => c as u8 as c_int,
    })
}

/// `getreg([{regname} [, 1 [, {list}]]])`.
pub unsafe fn f_getreg(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    let Some(regname) = (unsafe { regname(args) }) else {
        return;
    };
    // The two flag arguments are only read when a register was named:
    // `getreg()` alone cannot have them.
    let (mut expr_src, mut return_list) = (false, false);
    if args.has(0) && args.has(1) {
        let mut error = false;
        expr_src = arg_number_chk(args.get(1), Some(&mut error)) != 0;
        if !error && args.has(2) {
            return_list = arg_number_chk(args.get(2), Some(&mut error)) != 0;
        }
        if error {
            return;
        }
    }
    let mut flags = if expr_src { kGRegExprSrc as c_int } else { 0 };
    if return_list {
        flags |= kGRegList as c_int;
        rettv.v_type = VAR_LIST;
        let mut l = unsafe { get_reg_contents(regname, flags) } as *mut list_T;
        if l.is_null() {
            l = unsafe { tv_list_alloc(0) };
        }
        rettv.vval.v_list = l;
        unsafe { tv_list_ref(l) };
    } else {
        rettv.v_type = VAR_STRING;
        rettv.vval.v_string = unsafe { get_reg_contents(regname, flags) } as *mut c_char;
    }
}

/// `getregtype([{regname}])`.
pub unsafe fn f_getregtype(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = ptr::null_mut();
    // SAFETY: the arguments are live typvals and `buf` outlives the call
    // that fills it.
    let Some(regname) = (unsafe { regname(args) }) else {
        return;
    };
    let mut reglen: colnr_T = 0;
    let mut buf: TypeBuf = [0; 67];
    let reg_type = unsafe { get_reg_type(regname, &raw mut reglen) };
    unsafe { format_reg_type(reg_type, reglen, buf.as_mut_ptr(), buf.len()) };
    rettv.vval.v_string = unsafe { xstrdup(buf.as_ptr()) };
}

/// `getreginfo([{regname}])`.
pub unsafe fn f_getreginfo(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals; `buf` outlives
    // the two `tv_dict_add_str` calls that copy from it.
    let Some(mut regname) = (unsafe { regname(args) }) else {
        return;
    };
    if regname == b'@' as c_int {
        regname = b'"' as c_int;
    }
    dict_alloc_ret(rettv);
    let dict: *mut dict_T = unsafe { rettv.vval.v_dict };
    let list = unsafe { get_reg_contents(regname, kGRegExprSrc as c_int | kGRegList as c_int) }
        as *mut list_T;
    // An unset register has no `regcontents`, and no other key either.
    if list.is_null() {
        return;
    }
    unsafe { tv_dict_add_list(dict, c"regcontents".as_ptr(), 11, list) };

    let mut buf: TypeBuf = [0; 67];
    let mut reglen: colnr_T = 0;
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
    unsafe { tv_dict_add_str(dict, c"regtype".as_ptr(), 7, buf.as_ptr()) };

    // The unnamed register reports what it points at; every other one
    // reports whether it is what the unnamed register points at.
    buf[0] = get_register_name(unsafe { get_unname_register() }) as c_char;
    buf[1] = NUL as c_char;
    if regname == b'"' as c_int {
        unsafe { tv_dict_add_str(dict, c"points_to".as_ptr(), 9, buf.as_ptr()) };
    } else {
        let unnamed = regname == buf[0] as c_int;
        let flag = if unnamed { kBoolVarTrue } else { kBoolVarFalse } as BoolVarValue;
        unsafe { tv_dict_add_bool(dict, c"isunnamed".as_ptr(), 9, flag) };
    }
}

/// The single-character String the three recording-state builtins return.
///
/// # Safety
/// `rettv` is the dispatcher's cleared return value.
unsafe fn return_register(regname: c_int, rettv: &mut typval_T) {
    let buf: [c_char; 2] = [regname as c_char, 0];
    rettv.v_type = VAR_STRING;
    // SAFETY: `buf` is NUL-terminated and outlives the copy.
    rettv.vval.v_string = unsafe { xstrdup(buf.as_ptr()) };
}

/// `reg_executing()` — the register a macro is being played from.
pub unsafe fn f_reg_executing(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the dispatcher's cleared return value.
    unsafe { return_register(reg_executing.get(), &mut *rettv) };
}

/// `reg_recording()` — the register `q` is recording into.
pub unsafe fn f_reg_recording(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the dispatcher's cleared return value.
    unsafe { return_register(reg_recording.get(), &mut *rettv) };
}

/// `reg_recorded()` — the register the last recording went into.
pub unsafe fn f_reg_recorded(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: `rettv` is the dispatcher's cleared return value.
    unsafe { return_register(reg_recorded.get(), &mut *rettv) };
}

/// Read a register-type letter, advancing `pp` past the width digits a
/// blockwise type may carry.
///
/// `pp` is left on the *last* byte consumed, not one past it, because both
/// callers step it forward themselves.
///
/// # Safety
/// `*pp` points into a NUL-terminated string.
unsafe fn get_yank_type(
    pp: &mut *const c_char,
    yank_type: &mut MotionType,
    block_len: &mut c_int,
) -> c_int {
    // SAFETY throughout: the caller's obligation; `getdigits_int` only walks forward
    // and stops at the first non-digit.
    let mut p = *pp;
    match unsafe { *p } as u8 {
        b'v' | b'c' => *yank_type = kMTCharWise,
        b'V' | b'l' => *yank_type = kMTLineWise,
        b'b' => *yank_type = kMTBlockWise,
        c if c as c_int == Ctrl_V => *yank_type = kMTBlockWise,
        _ => return FAIL,
    }
    if *yank_type == kMTBlockWise && ascii_isdigit(unsafe { *p.add(1) } as c_int) {
        let mut q = unsafe { p.add(1) } as *mut c_char;
        *block_len = unsafe { getdigits_int(&raw mut q, false, 0) } - 1;
        p = unsafe { q.sub(1) };
    }
    *pp = p;
    OK
}

/// `setreg({regname}, {value} [, {options}])`.
pub unsafe fn f_setreg(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    let mut numbuf4 = NumBuf::new();
    let mut numbuf5 = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY throughout: the arguments and `rettv` are live typvals; every string
    // read below is NUL-terminated and outlives its use.
    // Non-zero means "did not set anything", which is what every early
    // return leaves behind.
    rettv.vval.v_number = 1;
    let strregname = arg_string_chk(&mut numbuf, args.get(0));
    if strregname.is_null() {
        return;
    }
    let mut regname = match unsafe { *strregname } as u8 {
        0 | b'@' => b'"' as c_char,
        _ => unsafe { *strregname },
    };

    let mut yank_type: MotionType = kMTUnknown;
    let mut block_len: c_int = -1;
    let mut regcontents: *const typval_T = ptr::null();
    let mut pointreg: c_char = 0;

    if args.ty(1) == VAR_DICT {
        let d = unsafe { args.get(1).vval.v_dict };
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
            if unsafe { get_yank_type(&mut p, &mut yank_type, &mut block_len) } == FAIL
                || unsafe { *p.add(1) } != NUL as c_char
            {
                unsafe { semsg_c!(gettext(e_invargval.as_ptr()), c"value".as_ptr(),) };
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
        regcontents = args.ptr(1);
    }

    let mut append = false;
    let mut set_unnamed = false;
    if args.has(2) {
        // A dict value already carried the type; a third argument on
        // top of it is one argument too many.
        if yank_type != kMTUnknown {
            unsafe { semsg_c!(gettext(e_toomanyarg.as_ptr()), c"setreg".as_ptr(),) };
            return;
        }
        let opts = arg_string_chk(&mut numbuf4, args.get(2));
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
                    unsafe { get_yank_type(&mut p, &mut yank_type, &mut block_len) };
                }
            }
            p = unsafe { p.add(1) };
        }
    }

    if !regcontents.is_null() && unsafe { (*regcontents).v_type } == VAR_LIST {
        let list = unsafe { (*regcontents).vval.v_list };
        unsafe { write_list(regname, list, append, yank_type, block_len) };
    } else if !regcontents.is_null() {
        let strval = unsafe { numbuf5.string_chk(regcontents) };
        if strval.is_null() {
            return;
        }
        let reg = regname as c_int;
        let len = unsafe { strlen(strval) } as isize;
        unsafe { write_reg_contents_ex(reg, strval, len, append, yank_type, block_len) };
    }
    if pointreg != 0 {
        unsafe { get_yank_register(pointreg as c_int, YREG_YANK as c_int) };
    }
    rettv.vval.v_number = 0;
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
    l: *mut list_T,
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
        let mut li: *const listitem_T = unsafe { (*l).lv_first };
        while !li.is_null() {
            let mut buf: [c_char; 65] = [0; 65];
            let s = unsafe { tv_get_string_buf_chk(&raw const (*li).li_tv, buf.as_mut_ptr()) };
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
            li = unsafe { (*li).li_next };
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
