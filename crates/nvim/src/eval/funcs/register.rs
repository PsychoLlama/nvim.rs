//! Registers: `getreg()`, `setreg()`, `getreginfo()` and the
//! recording state.
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::wrappers::{arg_number_chk, dict_alloc_ret};
use super::{kGRegExprSrc, kGRegList, kMTBlockWise, kMTCharWise, kMTLineWise, kMTUnknown};
use crate::cstr;
use crate::eval::typval::{NumBuf, dict_get_number, dict_len, list_iter, tv_list_alloc};
use crate::eval::vars::with_vim_var_str;
use crate::getchar::state::{reg_executing, reg_recorded, reg_recording};
use crate::keycodes::Ctrl_V;
use crate::memory::ThinCString;
use crate::register::{
    format_reg_type, get_reg_contents_list, get_reg_contents_owned, get_reg_type,
    get_register_name, get_unname_register, op_reg_set_previous, point_unnamed_at,
    write_reg_contents_cstr, write_reg_contents_lst,
};
use crate::semsg;
use crate::types::{
    BoolVarValue, Dict, EvalFuncData, Failed, MotionType, NUL, TypVal, VAR_DICT, VAR_LIST, Vv,
    kBoolVarFalse, kBoolVarTrue,
};
use core::ffi::{CStr, c_char, c_int};

/// Which register a builtin was asked about, or `None` if the argument was
/// not a String. An omitted argument means `v:register`, and an empty name
/// means the unnamed register.
fn regname(args: &[TypVal]) -> Option<c_int> {
    let mut numbuf = NumBuf::new();
    let first = if let Some(arg) = args.first() {
        numbuf.bytes_chk(arg)?.first().copied().unwrap_or(0)
    } else {
        with_vim_var_str(Vv::Register, cstr::first)
    };
    Some(match first {
        0 => b'"' as c_int,
        c => c_int::from(c),
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
        // An unset register gets a fresh empty list.
        let held = get_reg_contents_list(regname, flags).unwrap_or_else(|| tv_list_alloc(0));
        result.write_list(Some(held));
    } else {
        result.write_string(get_reg_contents_owned(regname, flags).map(ThinCString::from));
    }
}

/// `getregtype([{regname}])`.
pub fn f_getregtype(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(None);
    let Some(regname) = regname(args) else {
        return;
    };
    let (reg_type, width) = get_reg_type(regname);
    result.write_string(Some(format_reg_type(reg_type, width)));
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
    // Reading `"*`/`"+` runs the provider, so the answer's dictionary is
    // only borrowed between the reads.
    let list = get_reg_contents_list(regname, kGRegExprSrc as c_int);
    // An unset register has no `regcontents`, and no other key either.
    let Some(list) = list else {
        return;
    };
    let _ = dict(result).add_list(b"regcontents", Some(list));

    // A register that has contents has a type, which the check above
    // established.
    let (reg_type, width) = get_reg_type(regname);
    debug_assert!(matches!(reg_type, kMTLineWise | kMTCharWise | kMTBlockWise));
    let regtype = format_reg_type(reg_type, width);
    let _ = dict(result).add_str(b"regtype", Some(&regtype));

    // The unnamed register reports what it points at; every other one
    // reports whether it is what the unnamed register points at.
    let unnamed = get_register_name(get_unname_register());
    if regname == b'"' as c_int {
        let name = ThinCString::from_bytes(&[unnamed as c_char as u8]);
        let _ = dict(result).add_str(b"points_to", Some(&name));
    } else {
        let flag = if regname == unnamed {
            kBoolVarTrue
        } else {
            kBoolVarFalse
        };
        let _ = dict(result).add_bool(b"isunnamed", flag as BoolVarValue);
    }
}

/// The dictionary `getreginfo()` is filling in.
fn dict(result: &mut TypVal) -> &mut Dict {
    result.dict_mut().expect("just allocated")
}

/// The single-character String the three recording-state builtins return.
fn return_register(regname: c_int, result: &mut TypVal) {
    // A NUL register name answers the empty string.
    result.write_string(Some(ThinCString::from_bytes(&[regname as c_char as u8])));
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

/// Read the register-type letter at `text[*at]`, moving `at` past the width
/// digits a blockwise type may carry.
///
/// `at` is left on the *last* byte consumed, not one past it, because both
/// callers step it forward themselves.
fn get_yank_type(
    text: &[u8],
    at: &mut usize,
    yank_type: &mut MotionType,
    block_len: &mut c_int,
) -> Result<(), Failed> {
    match cstr::byte_at(text, *at) {
        b'v' | b'c' => *yank_type = kMTCharWise,
        b'V' | b'l' => *yank_type = kMTLineWise,
        b'b' => *yank_type = kMTBlockWise,
        c if c_int::from(c) == Ctrl_V => *yank_type = kMTBlockWise,
        _ => return Err(Failed),
    }
    let digits = &text[(*at + 1).min(text.len())..];
    let count = crate::charset::skip::digits(digits);
    if *yank_type == kMTBlockWise && count > 0 {
        // `getdigits_int` with no default: a width that does not fit an
        // `int` reads as 0.
        let width = digits[..count].iter().try_fold(0 as c_int, |n, &d| {
            n.checked_mul(10)?.checked_add(c_int::from(d - b'0'))
        });
        *block_len = width.unwrap_or(0) - 1;
        *at += count;
    }
    Ok(())
}

/// `setreg({regname}, {value} [, {options}])`.
pub fn f_setreg(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    let mut numbuf4 = NumBuf::new();
    let mut numbuf5 = NumBuf::new();
    // Non-zero means "did not set anything", which is what every early
    // return leaves behind.
    result.write_number(1);
    let Some(strregname) = numbuf.bytes_chk(&args[0]) else {
        return;
    };
    let mut regname = match strregname.first().copied().unwrap_or(0) {
        0 | b'@' => b'"' as c_char,
        c => c as c_char,
    };

    let mut yank_type: MotionType = kMTUnknown;
    let mut block_len: c_int = -1;
    let regcontents: Option<&TypVal>;
    let mut pointreg: c_char = 0;

    if args[1].v_type() == VAR_DICT {
        let d_ref = args[1].dict_ref();
        // An empty dict clears the register outright.
        if dict_len(d_ref) == 0 {
            write_reg_contents_lst(regname as c_int, &[], false, kMTUnknown, -1);
            return;
        }
        // The value is copied out before the register writer runs, which
        // for `"*`/`"+` calls the provider.
        regcontents = d_ref
            .and_then(|dict| dict.find(b"regcontents"))
            .map(|item| &item.di_tv);
        if let Some(stropt) = numbuf2.dict_string(d_ref, b"regtype") {
            let text = stropt.to_bytes();
            let mut at = 0;
            // The type must be exactly one letter (plus a width), so
            // the byte after what was consumed has to be the
            // terminator.
            if get_yank_type(text, &mut at, &mut yank_type, &mut block_len).is_err()
                || cstr::byte_at(text, at + 1) != NUL as u8
            {
                let arg0 = "value";
                semsg!("E475: Invalid value for argument {arg0}");
                return;
            }
        }
        if regname == b'"' as c_char {
            if let Some(stropt) = numbuf3.dict_string(d_ref, b"points_to") {
                pointreg = cstr::byte_at(stropt.to_bytes(), 0) as c_char;
                regname = pointreg;
            }
        } else if dict_get_number(d_ref, b"isunnamed") != 0 {
            pointreg = regname;
        }
    } else {
        regcontents = Some(&args[1]);
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
        let Some(opts) = numbuf4.bytes_chk(&args[2]) else {
            return;
        };
        let mut at = 0;
        while at < opts.len() {
            match opts[at] {
                b'a' | b'A' => append = true,
                b'u' | b'"' => set_unnamed = true,
                // Anything else is a register type, and an
                // unrecognised one is silently ignored.
                _ => {
                    let _ = get_yank_type(opts, &mut at, &mut yank_type, &mut block_len);
                }
            }
            at += 1;
        }
    }

    if let Some(contents) = regcontents
        && contents.v_type() == VAR_LIST
    {
        // An item that is not a String does not make the call fail: it
        // only leaves the register alone.
        if let Some(lines) = list_lines(contents) {
            let lines: Vec<&CStr> = lines.iter().map(|line| line.as_cstr()).collect();
            write_reg_contents_lst(regname as c_int, &lines, append, yank_type, block_len);
        }
    } else if let Some(contents) = regcontents {
        let Some(strval) = numbuf5.string_chk(contents) else {
            return;
        };
        let text = ThinCString::from_cstr(strval);
        write_reg_contents_cstr(regname as c_int, &text, append, yank_type, block_len);
    }
    if pointreg != 0 {
        point_unnamed_at(c_int::from(pointreg));
    }
    result.write_number(0);
    if set_unnamed {
        op_reg_set_previous(regname);
    }
}

/// A List value's items as owned strings, or `None` -- having given the
/// conversion's error -- at the first item that is not one.
fn list_lines(contents: &TypVal) -> Option<Vec<ThinCString>> {
    let mut lines = Vec::new();
    for item in list_iter(contents.list_ref()) {
        let mut buf = NumBuf::new();
        lines.push(ThinCString::from_cstr(buf.string_chk(&item.li_tv)?));
    }
    Some(lines)
}
