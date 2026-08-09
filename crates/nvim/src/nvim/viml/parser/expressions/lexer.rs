use super::*;
use crate::src::nvim::ascii::{ascii_isdigit, ascii_isident, ascii_iswhite};
use crate::src::nvim::keycodes::{
    Ctrl_A, Ctrl_B, Ctrl_C, Ctrl_D, Ctrl_E, Ctrl_F, Ctrl_G, Ctrl_H, Ctrl_K, Ctrl_L, Ctrl_M, Ctrl_N,
    Ctrl_O, Ctrl_P, Ctrl_Q, Ctrl_R, Ctrl_S, Ctrl_T, Ctrl_U, Ctrl_V, Ctrl_W, Ctrl_X, Ctrl_Y, Ctrl_Z,
};

pub const AUTOLOAD_CHAR: ::core::ffi::c_int = '#' as ::core::ffi::c_int;
#[inline(always)]
pub(super) fn scale_number(
    num: float_T,
    base: uint8_t,
    exponent: uvarnumber_T,
    exponent_negative: bool,
) -> float_T {
    if num == 0 as ::core::ffi::c_int as float_T || exponent == 0 as uvarnumber_T {
        return num;
    }
    debug_assert!(base != 0, "base");
    let mut exp: uvarnumber_T = exponent;
    let mut p_base: float_T = base as float_T;
    let mut ret: float_T = num;
    while exp != 0 {
        if exp & 1 as uvarnumber_T != 0 {
            if exponent_negative {
                ret /= p_base;
            } else {
                ret *= p_base;
            }
        }
        exp >>= 1 as ::core::ffi::c_int;
        p_base *= p_base;
    }
    return ret;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn viml_pexpr_next_token(
    pstate: *mut ParserState,
    flags: ::core::ffi::c_int,
) -> LexExprToken {
    let mut schar: uint8_t = 0;
    // The C partial initializer (`LexExprToken ret = { .type = ..., .start =
    // ... }`) zeroes the entire union; initializing only the `cmp` variant
    // leaves the tail of larger variants (e.g. `opt.scope`) as stack garbage,
    // which the parser later reads through for invalid option tokens.
    let mut ret: LexExprToken = ::core::mem::zeroed();
    ret.start = (*pstate).pos;
    ret.type_0 = kExprLexInvalid;
    let Some(pline) = viml_parser_get_remaining_line(pstate) else {
        ret.type_0 = kExprLexEOC;
        return ret;
    };
    if pline.size == 0 {
        ret.len = 0;
        ret.type_0 = kExprLexEOC;
    } else {
        ret.len = 1;
        schar = *pline.data.offset(0 as ::core::ffi::c_int as isize) as uint8_t;
        match schar as ::core::ffi::c_int {
            40 | 41 => {
                ret.type_0 = kExprLexParenthesis;
                ret.data.brc.closing = schar as ::core::ffi::c_int == ')' as ::core::ffi::c_int;
            }
            91 | 93 => {
                ret.type_0 = kExprLexBracket;
                ret.data.brc.closing = schar as ::core::ffi::c_int == ']' as ::core::ffi::c_int;
            }
            123 | 125 => {
                ret.type_0 = kExprLexFigureBrace;
                ret.data.brc.closing = schar as ::core::ffi::c_int == '}' as ::core::ffi::c_int;
            }
            63 => {
                ret.type_0 = kExprLexQuestion;
            }
            58 => {
                ret.type_0 = kExprLexColon;
            }
            44 => {
                ret.type_0 = kExprLexComma;
            }
            42 => {
                ret.type_0 = kExprLexMultiplication;
                ret.data.mul.type_0 = kExprLexMulMul;
            }
            47 => {
                ret.type_0 = kExprLexMultiplication;
                ret.data.mul.type_0 = kExprLexMulDiv;
            }
            37 => {
                ret.type_0 = kExprLexMultiplication;
                ret.data.mul.type_0 = kExprLexMulMod;
            }
            32 | TAB => {
                ret.type_0 = kExprLexSpacing;
                while ret.len < pline.size
                    && ascii_iswhite(*pline.data.add(ret.len) as ::core::ffi::c_int)
                {
                    ret.len = ret.len.wrapping_add(1);
                }
            }
            Ctrl_A | Ctrl_B | Ctrl_C | Ctrl_D | Ctrl_E | Ctrl_F | Ctrl_G | Ctrl_H | Ctrl_K
            | Ctrl_L | Ctrl_M | Ctrl_N | Ctrl_O | Ctrl_P | Ctrl_Q | Ctrl_R | Ctrl_S | Ctrl_T
            | Ctrl_U | Ctrl_V | Ctrl_W | Ctrl_X | Ctrl_Y | Ctrl_Z => {
                ret.type_0 = kExprLexInvalid;
                while ret.len < pline.size
                    && (*pline.data.add(ret.len) as ::core::ffi::c_int) < ' ' as ::core::ffi::c_int
                {
                    ret.len = ret.len.wrapping_add(1);
                }
                ret.data.err.type_0 = kExprLexSpacing;
                ret.data.err.msg =
                    gettext(c"E15: Invalid control character present in input: %.*s".as_ptr());
            }
            48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
                ret.data.num.is_float = false;
                ret.data.num.base = 10 as uint8_t;
                let mut frac_start: size_t = 0;
                let mut exp_start: size_t = 0;
                let mut frac_end: size_t = 0;
                let mut exp_negative: bool = false;
                ret.type_0 = kExprLexNumber;
                while ret.len < pline.size
                    && ascii_isdigit(*pline.data.add(ret.len) as ::core::ffi::c_int)
                {
                    ret.len = ret.len.wrapping_add(1);
                }
                if flags & kELFlagAllowFloat as ::core::ffi::c_int != 0 {
                    let non_float_ret: LexExprToken = ret;
                    if pline.size > ret.len.wrapping_add(1)
                        && *pline.data.add(ret.len) as ::core::ffi::c_int
                            == '.' as ::core::ffi::c_int
                        && ascii_isdigit(
                            *pline.data.add(ret.len.wrapping_add(1)) as ::core::ffi::c_int
                        )
                    {
                        ret.len = ret.len.wrapping_add(1);
                        frac_start = ret.len;
                        frac_end = ret.len;
                        ret.data.num.is_float = true;
                        while ret.len < pline.size
                            && ascii_isdigit(*pline.data.add(ret.len) as ::core::ffi::c_int)
                        {
                            if *pline.data.add(ret.len) as ::core::ffi::c_int
                                != '0' as ::core::ffi::c_int
                            {
                                frac_end = ret.len.wrapping_add(1);
                            }
                            ret.len = ret.len.wrapping_add(1);
                        }
                        if pline.size > ret.len.wrapping_add(1)
                            && (*pline.data.add(ret.len) as ::core::ffi::c_int
                                == 'e' as ::core::ffi::c_int
                                || *pline.data.add(ret.len) as ::core::ffi::c_int
                                    == 'E' as ::core::ffi::c_int)
                            && (pline.size > ret.len.wrapping_add(2)
                                && (*pline.data.add(ret.len.wrapping_add(1)) as ::core::ffi::c_int
                                    == '+' as ::core::ffi::c_int
                                    || *pline.data.add(ret.len.wrapping_add(1))
                                        as ::core::ffi::c_int
                                        == '-' as ::core::ffi::c_int)
                                && ascii_isdigit(
                                    *pline.data.add(ret.len.wrapping_add(2)) as ::core::ffi::c_int
                                )
                                || ascii_isdigit(
                                    *pline.data.add(ret.len.wrapping_add(1)) as ::core::ffi::c_int
                                ))
                        {
                            ret.len = ret.len.wrapping_add(1);
                            if *pline.data.add(ret.len) as ::core::ffi::c_int
                                == '+' as ::core::ffi::c_int
                                || {
                                    exp_negative = *pline.data.add(ret.len) as ::core::ffi::c_int
                                        == '-' as ::core::ffi::c_int;
                                    exp_negative
                                }
                            {
                                ret.len = ret.len.wrapping_add(1);
                            }
                            exp_start = ret.len;
                            ret.type_0 = kExprLexNumber;
                            while ret.len < pline.size
                                && ascii_isdigit(*pline.data.add(ret.len) as ::core::ffi::c_int)
                            {
                                ret.len = ret.len.wrapping_add(1);
                            }
                        }
                    }
                    if pline.size > ret.len
                        && (*pline.data.add(ret.len) as ::core::ffi::c_int
                            == '.' as ::core::ffi::c_int
                            || (*pline.data.add(ret.len) as ::core::ffi::c_uint
                                >= 'A' as ::core::ffi::c_uint
                                && *pline.data.add(ret.len) as ::core::ffi::c_uint
                                    <= 'Z' as ::core::ffi::c_uint
                                || *pline.data.add(ret.len) as ::core::ffi::c_uint
                                    >= 'a' as ::core::ffi::c_uint
                                    && *pline.data.add(ret.len) as ::core::ffi::c_uint
                                        <= 'z' as ::core::ffi::c_uint))
                    {
                        ret = non_float_ret;
                    }
                }
                if ret.data.num.is_float {
                    let mut significand_part: float_T = 0 as ::core::ffi::c_int as float_T;
                    let mut exp_part: uvarnumber_T = 0 as uvarnumber_T;
                    let frac_size: size_t = frac_end.wrapping_sub(frac_start);
                    let mut i: size_t = 0;
                    while i < frac_end {
                        if i != frac_start.wrapping_sub(1) {
                            significand_part = significand_part
                                * 10 as ::core::ffi::c_int as float_T
                                + (*pline.data.add(i) as ::core::ffi::c_int
                                    - '0' as ::core::ffi::c_int)
                                    as float_T;
                        }
                        i = i.wrapping_add(1);
                    }
                    if exp_start != 0 {
                        vim_str2nr(
                            pline.data.add(exp_start),
                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                            0 as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<varnumber_T>(),
                            &raw mut exp_part,
                            ret.len.wrapping_sub(exp_start) as ::core::ffi::c_int,
                            false,
                            ::core::ptr::null_mut::<bool>(),
                        );
                    }
                    if exp_negative {
                        exp_part = (exp_part as ::core::ffi::c_ulong)
                            .wrapping_add(frac_size as ::core::ffi::c_ulong)
                            as uvarnumber_T;
                    } else if exp_part < frac_size as uvarnumber_T {
                        exp_negative = true;
                        exp_part = frac_size.wrapping_sub(exp_part as size_t) as uvarnumber_T;
                    } else {
                        exp_part = (exp_part as ::core::ffi::c_ulong)
                            .wrapping_sub(frac_size as ::core::ffi::c_ulong)
                            as uvarnumber_T;
                    }
                    ret.data.num.val.floating =
                        scale_number(significand_part, 10 as uint8_t, exp_part, exp_negative);
                } else {
                    let mut len: ::core::ffi::c_int = 0;
                    let mut prep: ::core::ffi::c_int = 0;
                    vim_str2nr(
                        pline.data,
                        &raw mut prep,
                        &raw mut len,
                        STR2NR_ALL as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<varnumber_T>(),
                        &raw mut ret.data.num.val.integer,
                        pline.size as ::core::ffi::c_int,
                        false,
                        ::core::ptr::null_mut::<bool>(),
                    );
                    ret.len = len as size_t;
                    let bases: [uint8_t; 121] = [
                        10 as uint8_t,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        8 as uint8_t,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        2 as uint8_t,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        16 as uint8_t,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        2 as uint8_t,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        16 as uint8_t,
                    ];
                    ret.data.num.base = bases[prep as usize];
                }
            }
            36 => {
                ret.type_0 = kExprLexEnv;
                while ret.len < pline.size
                    && ascii_isident(*pline.data.add(ret.len) as ::core::ffi::c_int)
                {
                    ret.len = ret.len.wrapping_add(1);
                }
            }
            97 | 98 | 99 | 100 | 101 | 102 | 103 | 104 | 105 | 106 | 107 | 108 | 109 | 110
            | 111 | 112 | 113 | 114 | 115 | 116 | 117 | 118 | 119 | 120 | 121 | 122 | 65 | 66
            | 67 | 68 | 69 | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 80 | 81 | 82
            | 83 | 84 | 85 | 86 | 87 | 88 | 89 | 90 | 95 => {
                ret.data.var.scope = kExprVarScopeMissing;
                ret.data.var.autoload = false;
                ret.type_0 = kExprLexPlainIdentifier;
                while ret.len < pline.size
                    && ascii_isident(*pline.data.add(ret.len) as ::core::ffi::c_int)
                {
                    ret.len = ret.len.wrapping_add(1);
                }
                if flags & kELFlagIsNotCmp as ::core::ffi::c_int == 0
                    && (ret.len == 2
                        && memcmp(
                            pline.data as *const ::core::ffi::c_void,
                            c"is".as_ptr() as *const ::core::ffi::c_void,
                            2,
                        ) == 0 as ::core::ffi::c_int
                        || ret.len == 5
                            && memcmp(
                                pline.data as *const ::core::ffi::c_void,
                                c"isnot".as_ptr() as *const ::core::ffi::c_void,
                                5,
                            ) == 0 as ::core::ffi::c_int)
                {
                    ret.type_0 = kExprLexComparison;
                    ret.data.cmp.type_0 = kExprCmpIdentical;
                    ret.data.cmp.inv = ret.len == 5;
                    if ret.len < pline.size
                        && !strchr(
                            c"?#".as_ptr(),
                            *pline.data.add(ret.len) as ::core::ffi::c_int,
                        )
                        .is_null()
                    {
                        ret.data.cmp.ccs = *pline.data.add(ret.len) as ExprCaseCompareStrategy;
                        ret.len = ret.len.wrapping_add(1);
                    } else {
                        ret.data.cmp.ccs = kCCStrategyUseOption;
                    }
                } else if ret.len == 1
                    && pline.size > 1
                    && {
                        let mut c2rust_lvalue: [::core::ffi::c_char; 9] = [
                            kExprVarScopeScript as ::core::ffi::c_int as ::core::ffi::c_char,
                            kExprVarScopeGlobal as ::core::ffi::c_int as ::core::ffi::c_char,
                            kExprVarScopeVim as ::core::ffi::c_int as ::core::ffi::c_char,
                            kExprVarScopeBuffer as ::core::ffi::c_int as ::core::ffi::c_char,
                            kExprVarScopeWindow as ::core::ffi::c_int as ::core::ffi::c_char,
                            kExprVarScopeTabpage as ::core::ffi::c_int as ::core::ffi::c_char,
                            kExprVarScopeLocal as ::core::ffi::c_int as ::core::ffi::c_char,
                            kExprVarScopeBuffer as ::core::ffi::c_int as ::core::ffi::c_char,
                            kExprVarScopeArguments as ::core::ffi::c_int as ::core::ffi::c_char,
                        ];
                        !memchr(
                            &raw mut c2rust_lvalue as *mut ::core::ffi::c_char
                                as *const ::core::ffi::c_void,
                            schar as ::core::ffi::c_int,
                            ::core::mem::size_of::<[::core::ffi::c_char; 9]>(),
                        )
                        .is_null()
                    }
                    && *pline.data.add(ret.len) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                    && flags & kELFlagForbidScope as ::core::ffi::c_int == 0
                {
                    ret.len = ret.len.wrapping_add(1);
                    ret.data.var.scope = schar as ExprVarScope;
                    ret.type_0 = kExprLexPlainIdentifier;
                    while ret.len < pline.size
                        && (ascii_isident(*pline.data.add(ret.len) as ::core::ffi::c_int)
                            || *pline.data.add(ret.len) as ::core::ffi::c_int == AUTOLOAD_CHAR)
                    {
                        ret.len = ret.len.wrapping_add(1);
                    }
                    ret.data.var.autoload = !memchr(
                        pline.data.offset(2 as ::core::ffi::c_int as isize)
                            as *const ::core::ffi::c_void,
                        AUTOLOAD_CHAR,
                        ret.len.wrapping_sub(2),
                    )
                    .is_null();
                } else if pline.size > ret.len
                    && *pline.data.add(ret.len) as ::core::ffi::c_int == AUTOLOAD_CHAR
                {
                    ret.data.var.autoload = true;
                    ret.type_0 = kExprLexPlainIdentifier;
                    while ret.len < pline.size
                        && (ascii_isident(*pline.data.add(ret.len) as ::core::ffi::c_int)
                            || *pline.data.add(ret.len) as ::core::ffi::c_int == AUTOLOAD_CHAR)
                    {
                        ret.len = ret.len.wrapping_add(1);
                    }
                }
            }
            38 => {
                if pline.size > 1
                    && *pline.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '&' as ::core::ffi::c_int
                {
                    ret.type_0 = kExprLexAnd;
                    ret.len = ret.len.wrapping_add(1);
                } else if pline.size == 1
                    || !(*pline.data.offset(1 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_uint
                        >= 'A' as ::core::ffi::c_uint
                        && *pline.data.offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_uint
                            <= 'Z' as ::core::ffi::c_uint
                        || *pline.data.offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_uint
                            >= 'a' as ::core::ffi::c_uint
                            && *pline.data.offset(1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_uint
                                <= 'z' as ::core::ffi::c_uint)
                {
                    ret.type_0 = kExprLexInvalid;
                    ret.data.err.type_0 = kExprLexOption;
                    ret.data.err.msg = gettext(c"E112: Option name missing: %.*s".as_ptr());
                } else {
                    ret.type_0 = kExprLexOption;
                    if pline.size > 2
                        && *pline.data.offset(2 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == ':' as ::core::ffi::c_int
                        && {
                            let mut c2rust_lvalue_0: [::core::ffi::c_char; 2] = [
                                kExprOptScopeGlobal as ::core::ffi::c_int as ::core::ffi::c_char,
                                kExprOptScopeLocal as ::core::ffi::c_int as ::core::ffi::c_char,
                            ];
                            !memchr(
                                &raw mut c2rust_lvalue_0 as *mut ::core::ffi::c_char
                                    as *const ::core::ffi::c_void,
                                *pline.data.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int,
                                ::core::mem::size_of::<[::core::ffi::c_char; 2]>(),
                            )
                            .is_null()
                        }
                    {
                        ret.len = ret.len.wrapping_add(2);
                        ret.data.opt.scope =
                            *pline.data.offset(1 as ::core::ffi::c_int as isize) as ExprOptScope;
                        ret.data.opt.name = pline.data.offset(3 as ::core::ffi::c_int as isize);
                    } else {
                        ret.data.opt.scope = kExprOptScopeUnspecified;
                        ret.data.opt.name = pline.data.offset(1 as ::core::ffi::c_int as isize);
                    }
                    let mut p: *const ::core::ffi::c_char = ret.data.opt.name;
                    let e: *const ::core::ffi::c_char = pline.data.add(pline.size);
                    if e.offset_from(p) >= 4 as isize
                        && *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 't' as ::core::ffi::c_int
                        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '_' as ::core::ffi::c_int
                    {
                        ret.data.opt.len = 4;
                        ret.len = ret.len.wrapping_add(4);
                    } else {
                        while p < e
                            && (*p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                                && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                                || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                                    && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
                        {
                            p = p.offset(1);
                        }
                        ret.data.opt.len = p.offset_from(ret.data.opt.name) as size_t;
                        if ret.data.opt.len == 0 {
                            ret.type_0 = kExprLexInvalid;
                            ret.data.err.type_0 = kExprLexOption;
                            ret.data.err.msg = gettext(c"E112: Option name missing: %.*s".as_ptr());
                        } else {
                            ret.len = ret.len.wrapping_add(ret.data.opt.len);
                        }
                    }
                }
            }
            64 => {
                ret.type_0 = kExprLexRegister;
                if pline.size > 1 {
                    ret.len = ret.len.wrapping_add(1);
                    ret.data.reg.name = *pline.data.offset(1 as ::core::ffi::c_int as isize)
                        as uint8_t as ::core::ffi::c_int;
                } else {
                    ret.data.reg.name = -1 as ::core::ffi::c_int;
                }
            }
            39 => {
                ret.type_0 = kExprLexSingleQuotedString;
                ret.data.str.closed = false;
                while ret.len < pline.size && !ret.data.str.closed {
                    if *pline.data.add(ret.len) as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
                    {
                        if ret.len.wrapping_add(1) < pline.size
                            && *pline.data.add(ret.len.wrapping_add(1)) as ::core::ffi::c_int
                                == '\'' as ::core::ffi::c_int
                        {
                            ret.len = ret.len.wrapping_add(1);
                        } else {
                            ret.data.str.closed = true;
                        }
                    }
                    ret.len = ret.len.wrapping_add(1);
                }
            }
            34 => {
                ret.type_0 = kExprLexDoubleQuotedString;
                ret.data.str.closed = false;
                while ret.len < pline.size && !ret.data.str.closed {
                    if *pline.data.add(ret.len) as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                    {
                        if ret.len.wrapping_add(1) < pline.size {
                            ret.len = ret.len.wrapping_add(1);
                        }
                    } else if *pline.data.add(ret.len) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
                    {
                        ret.data.str.closed = true;
                    }
                    ret.len = ret.len.wrapping_add(1);
                }
            }
            33 | 61 => {
                if pline.size == 1 {
                    ret.type_0 = (if schar as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                        kExprLexNot as ::core::ffi::c_int
                    } else {
                        kExprLexAssignment as ::core::ffi::c_int
                    }) as LexExprTokenType;
                    ret.data.ass.type_0 = kExprAsgnPlain;
                } else {
                    ret.type_0 = kExprLexComparison;
                    ret.data.cmp.inv = schar as ::core::ffi::c_int == '!' as ::core::ffi::c_int;
                    if *pline.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int
                    {
                        ret.data.cmp.type_0 = kExprCmpEqual;
                        ret.len = ret.len.wrapping_add(1);
                    } else if *pline.data.offset(1 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == '~' as ::core::ffi::c_int
                    {
                        ret.data.cmp.type_0 = kExprCmpMatches;
                        ret.len = ret.len.wrapping_add(1);
                    } else if schar as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                        ret.type_0 = kExprLexNot;
                    } else {
                        ret.type_0 = kExprLexAssignment;
                        ret.data.ass.type_0 = kExprAsgnPlain;
                    }
                    if ret.len < pline.size
                        && !strchr(
                            c"?#".as_ptr(),
                            *pline.data.add(ret.len) as ::core::ffi::c_int,
                        )
                        .is_null()
                    {
                        ret.data.cmp.ccs = *pline.data.add(ret.len) as ExprCaseCompareStrategy;
                        ret.len = ret.len.wrapping_add(1);
                    } else {
                        ret.data.cmp.ccs = kCCStrategyUseOption;
                    }
                }
            }
            62 | 60 => {
                ret.type_0 = kExprLexComparison;
                let haseqsign: bool = pline.size > 1
                    && *pline.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int;
                if haseqsign {
                    ret.len = ret.len.wrapping_add(1);
                }
                if ret.len < pline.size
                    && !strchr(
                        c"?#".as_ptr(),
                        *pline.data.add(ret.len) as ::core::ffi::c_int,
                    )
                    .is_null()
                {
                    ret.data.cmp.ccs = *pline.data.add(ret.len) as ExprCaseCompareStrategy;
                    ret.len = ret.len.wrapping_add(1);
                } else {
                    ret.data.cmp.ccs = kCCStrategyUseOption;
                }
                ret.data.cmp.inv = schar as ::core::ffi::c_int == '<' as ::core::ffi::c_int;
                ret.data.cmp.type_0 = (if ret.data.cmp.inv ^ haseqsign {
                    kExprCmpGreaterOrEqual as ::core::ffi::c_int
                } else {
                    kExprCmpGreater as ::core::ffi::c_int
                }) as ExprComparisonType;
            }
            45 => {
                if pline.size > 1
                    && *pline.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '>' as ::core::ffi::c_int
                {
                    ret.len = ret.len.wrapping_add(1);
                    ret.type_0 = kExprLexArrow;
                } else if pline.size > 1
                    && *pline.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int
                {
                    ret.len = ret.len.wrapping_add(1);
                    ret.type_0 = kExprLexAssignment;
                    ret.data.ass.type_0 = kExprAsgnSubtract;
                } else {
                    ret.type_0 = kExprLexMinus;
                }
            }
            43 => {
                if pline.size > 1
                    && *pline.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int
                {
                    ret.len = ret.len.wrapping_add(1);
                    ret.type_0 = kExprLexAssignment;
                    ret.data.ass.type_0 = kExprAsgnAdd;
                } else {
                    ret.type_0 = kExprLexPlus;
                }
            }
            46 => {
                if pline.size > 1
                    && *pline.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int
                {
                    ret.len = ret.len.wrapping_add(1);
                    ret.type_0 = kExprLexAssignment;
                    ret.data.ass.type_0 = kExprAsgnConcat;
                } else {
                    ret.type_0 = kExprLexDot;
                }
            }
            NUL | NL => {
                if flags & kELFlagForbidEOC as ::core::ffi::c_int != 0 {
                    ret.type_0 = kExprLexInvalid;
                    ret.data.err.msg = gettext(c"E15: Unexpected EOC character: %.*s".as_ptr());
                    ret.data.err.type_0 = kExprLexSpacing;
                } else {
                    ret.type_0 = kExprLexEOC;
                }
            }
            124 => {
                if pline.size >= 2
                    && *pline.data.add(ret.len) as ::core::ffi::c_int == '|' as ::core::ffi::c_int
                {
                    ret.len = ret.len.wrapping_add(1);
                    ret.type_0 = kExprLexOr;
                } else if flags & kELFlagForbidEOC as ::core::ffi::c_int != 0 {
                    ret.type_0 = kExprLexInvalid;
                    ret.data.err.msg = gettext(c"E15: Unexpected EOC character: %.*s".as_ptr());
                    ret.data.err.type_0 = kExprLexOr;
                } else {
                    ret.type_0 = kExprLexEOC;
                }
            }
            _ => {
                ret.len = utfc_ptr2len_len(pline.data, pline.size as ::core::ffi::c_int) as size_t;
                ret.type_0 = kExprLexInvalid;
                ret.data.err.type_0 = kExprLexPlainIdentifier;
                ret.data.err.msg = gettext(c"E15: Unidentified character: %.*s".as_ptr());
            }
        }
    }
    if flags & kELFlagPeek as ::core::ffi::c_int == 0 {
        viml_parser_advance(&mut (*pstate).pos, &mut (*pstate).reader, ret.len);
    }
    return ret;
}
