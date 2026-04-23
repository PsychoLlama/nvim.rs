// VimL expression lexer — Rust replacement for viml_pexpr_next_token().
//
// Mirrors the C implementation in expressions.c exactly, including the
// data structures written into LexExprToken's union fields.
//
// This module is a mechanical translation of heavily macro-driven C code.
// Several clippy lints are suppressed because the patterns are intentional.
#![allow(clippy::too_many_lines)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ref_as_ptr)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::same_functions_in_if_condition)]

use std::ffi::{c_char, c_int, c_uchar};
use std::mem::MaybeUninit;

use crate::expr_types::{
    ExprAssignmentType, ExprCaseCompareStrategy, ExprComparisonType, ExprOptScope, ExprVarScope,
    LexExprMulType, LexExprToken, LexExprTokenType, LexTknData, LexTknDataAss, LexTknDataBrc,
    LexTknDataCmp, LexTknDataErr, LexTknDataMul, LexTknDataNum, LexTknDataOpt, LexTknDataReg,
    LexTknDataStr, LexTknDataVar, LexTknNumVal, ParserLine, ParserPosition,
};

// ---------------------------------------------------------------------------
// Opaque handle for ParserState (defined in C).
// ---------------------------------------------------------------------------

/// Opaque type for the C `ParserState` struct.
/// Rust never constructs or sizes this; it only passes pointers through.
#[repr(C)]
pub struct ParserState {
    _opaque: [u8; 0],
}

// ---------------------------------------------------------------------------
// Extern "C" declarations for C functions called by the lexer.
// ---------------------------------------------------------------------------

extern "C" {
    /// Non-inline wrapper around viml_parser_advance().
    fn nvim_viml_parser_advance(pstate: *mut ParserState, len: usize);

    /// Non-inline wrapper around viml_parser_get_remaining_line().
    fn nvim_viml_parser_get_remaining_line(
        pstate: *mut ParserState,
        ret_pline: *mut ParserLine,
    ) -> bool;

    /// Parse a number string. See charset.h for full signature.
    pub(crate) fn vim_str2nr(
        start: *const c_char,
        prep: *mut c_int,
        len: *mut c_int,
        what: c_int,
        nptr: *mut i64,
        unptr: *mut u64,
        maxlen: c_int,
        strict: bool,
        overflow: *mut bool,
    );

    /// Get multibyte character length up to `size` bytes.
    pub(crate) fn utfc_ptr2len_len(p: *const c_char, size: c_int) -> c_int;
}

// ---------------------------------------------------------------------------
// ASCII predicates (trivial, implement inline rather than calling C).
// ---------------------------------------------------------------------------

#[inline]
fn ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

#[inline]
fn ascii_isdigit(c: u8) -> bool {
    c.is_ascii_digit()
}

#[allow(dead_code)]
#[inline]
pub(crate) fn ascii_isxdigit(c: u8) -> bool {
    c.is_ascii_hexdigit()
}

#[inline]
fn ascii_isident(c: u8) -> bool {
    // C implementation: CT_ID_CHAR flag in chartab. For ASCII the rule is:
    // alphanumeric or underscore (i.e. \w in regex).
    c.is_ascii_alphanumeric() || c == b'_'
}

#[inline]
fn ascii_isalpha(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

/// AUTOLOAD_CHAR is '#'
const AUTOLOAD_CHAR: u8 = b'#';

/// STR2NR_ALL from charset.h
const STR2NR_ALL: c_int = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3); // BIN|OCT|HEX|OOCT

// ---------------------------------------------------------------------------
// EXPR_VAR_SCOPE_LIST and EXPR_OPT_SCOPE_LIST
// ---------------------------------------------------------------------------

const EXPR_VAR_SCOPE_LIST: &[u8] = b"sgvbwtla";
const EXPR_OPT_SCOPE_LIST: &[u8] = b"gl";

// ---------------------------------------------------------------------------
// scale_number (private helper, mirrors the C static inline)
// ---------------------------------------------------------------------------

#[inline]
fn scale_number(num: f64, base: u8, exponent: u64, exponent_negative: bool) -> f64 {
    if num == 0.0 || exponent == 0 {
        return num;
    }
    debug_assert!(base != 0);
    let mut exp = exponent;
    let mut p_base = base as f64;
    let mut ret = num;
    while exp != 0 {
        if exp & 1 != 0 {
            if exponent_negative {
                ret /= p_base;
            } else {
                ret *= p_base;
            }
        }
        exp >>= 1;
        p_base *= p_base;
    }
    ret
}

// ---------------------------------------------------------------------------
// viml_pexpr_next_token
// ---------------------------------------------------------------------------

/// Get the next token for a VimL expression input.
///
/// Exported with the original C name so existing C callers link without change.
///
/// # Safety
/// `pstate` must be a valid non-null pointer to a live `ParserState`.
#[unsafe(export_name = "viml_pexpr_next_token")]
pub unsafe extern "C" fn viml_pexpr_next_token(
    pstate: *mut ParserState,
    flags: c_int,
) -> LexExprToken {
    // SAFETY: caller guarantees pstate is valid.
    let pos = unsafe { parser_get_pos_raw(pstate) };

    let mut ret = LexExprToken {
        typ: LexExprTokenType::Invalid,
        start: pos,
        len: 0,
        data: unsafe { MaybeUninit::zeroed().assume_init() },
    };

    let mut pline = unsafe { MaybeUninit::<ParserLine>::zeroed().assume_init() };
    if !unsafe { nvim_viml_parser_get_remaining_line(pstate, &mut pline) } {
        ret.typ = LexExprTokenType::EOC;
        return ret;
    }

    if pline.size == 0 {
        ret.len = 0;
        ret.typ = LexExprTokenType::EOC;
        // goto viml_pexpr_next_token_adv_return
        return adv_return(pstate, ret, flags);
    }

    ret.len = 1;
    let data: &[u8] = unsafe { std::slice::from_raw_parts(pline.data as *const u8, pline.size) };
    let schar = data[0];

    // Main dispatch switch — mirrors the C switch(schar) { ... }
    match schar {
        // Paired brackets.
        b'(' | b')' => {
            ret.typ = LexExprTokenType::Parenthesis;
            ret.data = LexTknData {
                brc: LexTknDataBrc {
                    closing: schar == b')',
                },
            };
        }
        b'[' | b']' => {
            ret.typ = LexExprTokenType::Bracket;
            ret.data = LexTknData {
                brc: LexTknDataBrc {
                    closing: schar == b']',
                },
            };
        }
        b'{' | b'}' => {
            ret.typ = LexExprTokenType::FigureBrace;
            ret.data = LexTknData {
                brc: LexTknDataBrc {
                    closing: schar == b'}',
                },
            };
        }

        // Single-character tokens without data.
        b'?' => {
            ret.typ = LexExprTokenType::Question;
        }
        b':' => {
            ret.typ = LexExprTokenType::Colon;
        }
        b',' => {
            ret.typ = LexExprTokenType::Comma;
        }

        // Multiplication / division / modulo.
        b'*' => {
            ret.typ = LexExprTokenType::Multiplication;
            ret.data = LexTknData {
                mul: LexTknDataMul {
                    typ: LexExprMulType::Mul,
                },
            };
        }
        b'/' => {
            ret.typ = LexExprTokenType::Multiplication;
            ret.data = LexTknData {
                mul: LexTknDataMul {
                    typ: LexExprMulType::Div,
                },
            };
        }
        b'%' => {
            ret.typ = LexExprTokenType::Multiplication;
            ret.data = LexTknData {
                mul: LexTknDataMul {
                    typ: LexExprMulType::Mod,
                },
            };
        }

        // Whitespace.
        b' ' | b'\t' => {
            ret.typ = LexExprTokenType::Spacing;
            while ret.len < pline.size && ascii_iswhite(data[ret.len]) {
                ret.len += 1;
            }
        }

        // Control characters (except NUL=0, NL=10, TAB=9).
        c if c < b' ' && c != b'\t' && c != 0 && c != b'\n' => {
            ret.typ = LexExprTokenType::Invalid;
            while ret.len < pline.size && data[ret.len] < b' ' {
                ret.len += 1;
            }
            ret.data = LexTknData {
                err: LexTknDataErr {
                    typ: LexExprTokenType::Spacing,
                    msg: b"E15: Invalid control character present in input: %.*s\0"
                        .as_ptr()
                        .cast::<c_char>(),
                },
            };
        }

        // Number literal.
        b'0'..=b'9' => {
            ret = lex_number(ret, data, pline.size, flags);
        }

        // Environment variable.
        b'$' => {
            ret.typ = LexExprTokenType::Env;
            while ret.len < pline.size && ascii_isident(data[ret.len]) {
                ret.len += 1;
            }
        }

        // Identifier or keyword.
        b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
            ret = lex_ident(ret, schar, data, pline.size, flags);
        }

        // Option.
        b'&' => {
            ret = lex_option(ret, data, pline.size);
        }

        // Register.
        b'@' => {
            ret.typ = LexExprTokenType::Register;
            if pline.size > 1 {
                ret.len += 1;
                ret.data = LexTknData {
                    reg: LexTknDataReg {
                        name: data[1] as c_int,
                    },
                };
            } else {
                ret.data = LexTknData {
                    reg: LexTknDataReg { name: -1 },
                };
            }
        }

        // Single-quoted string.
        b'\'' => {
            ret.typ = LexExprTokenType::SingleQuotedString;
            ret.data = LexTknData {
                str_: LexTknDataStr { closed: false },
            };
            while ret.len < pline.size && !unsafe { ret.data.str_.closed } {
                if data[ret.len] == b'\'' {
                    if ret.len + 1 < pline.size && data[ret.len + 1] == b'\'' {
                        ret.len += 1;
                    } else {
                        ret.data = LexTknData {
                            str_: LexTknDataStr { closed: true },
                        };
                    }
                }
                ret.len += 1;
            }
        }

        // Double-quoted string.
        b'"' => {
            ret.typ = LexExprTokenType::DoubleQuotedString;
            ret.data = LexTknData {
                str_: LexTknDataStr { closed: false },
            };
            while ret.len < pline.size && !unsafe { ret.data.str_.closed } {
                if data[ret.len] == b'\\' {
                    if ret.len + 1 < pline.size {
                        ret.len += 1;
                    }
                } else if data[ret.len] == b'"' {
                    ret.data = LexTknData {
                        str_: LexTknDataStr { closed: true },
                    };
                }
                ret.len += 1;
            }
        }

        // Not (!) or equality/match operators.
        b'!' | b'=' => {
            if pline.size == 1 {
                if schar == b'!' {
                    ret.typ = LexExprTokenType::Not;
                } else {
                    ret.typ = LexExprTokenType::Assignment;
                    ret.data = LexTknData {
                        ass: LexTknDataAss {
                            typ: ExprAssignmentType::Plain,
                        },
                    };
                }
            } else {
                ret.typ = LexExprTokenType::Comparison;
                let inv = schar == b'!';
                if data[1] == b'=' {
                    ret.data = LexTknData {
                        cmp: LexTknDataCmp {
                            typ: ExprComparisonType::Equal,
                            ccs: ExprCaseCompareStrategy::UseOption,
                            inv,
                        },
                    };
                    ret.len += 1;
                } else if data[1] == b'~' {
                    ret.data = LexTknData {
                        cmp: LexTknDataCmp {
                            typ: ExprComparisonType::Matches,
                            ccs: ExprCaseCompareStrategy::UseOption,
                            inv,
                        },
                    };
                    ret.len += 1;
                } else if schar == b'!' {
                    ret.typ = LexExprTokenType::Not;
                } else {
                    ret.typ = LexExprTokenType::Assignment;
                    ret.data = LexTknData {
                        ass: LexTknDataAss {
                            typ: ExprAssignmentType::Plain,
                        },
                    };
                }
                // GET_CCS for comparison tokens.
                if ret.typ == LexExprTokenType::Comparison {
                    let ccs = get_ccs(&mut ret.len, data, pline.size);
                    ret.data.cmp.ccs = ccs;
                }
            }
        }

        // Less/greater [or equal to] comparison operators.
        b'>' | b'<' => {
            ret.typ = LexExprTokenType::Comparison;
            let haseqsign = pline.size > 1 && data[1] == b'=';
            if haseqsign {
                ret.len += 1;
            }
            let ccs = get_ccs(&mut ret.len, data, pline.size);
            let inv = schar == b'<';
            let cmp_type = if inv ^ haseqsign {
                ExprComparisonType::GreaterOrEqual
            } else {
                ExprComparisonType::Greater
            };
            ret.data = LexTknData {
                cmp: LexTknDataCmp {
                    typ: cmp_type,
                    ccs,
                    inv,
                },
            };
        }

        // Minus, arrow, or subtraction-assignment.
        b'-' => {
            if pline.size > 1 && data[1] == b'>' {
                ret.len += 1;
                ret.typ = LexExprTokenType::Arrow;
            } else if pline.size > 1 && data[1] == b'=' {
                ret.len += 1;
                ret.typ = LexExprTokenType::Assignment;
                ret.data = LexTknData {
                    ass: LexTknDataAss {
                        typ: ExprAssignmentType::Subtract,
                    },
                };
            } else {
                ret.typ = LexExprTokenType::Minus;
            }
        }

        // Plus or addition-assignment.
        b'+' => {
            if pline.size > 1 && data[1] == b'=' {
                ret.len += 1;
                ret.typ = LexExprTokenType::Assignment;
                ret.data = LexTknData {
                    ass: LexTknDataAss {
                        typ: ExprAssignmentType::Add,
                    },
                };
            } else {
                ret.typ = LexExprTokenType::Plus;
            }
        }

        // Dot or concatenation-assignment.
        b'.' => {
            if pline.size > 1 && data[1] == b'=' {
                ret.len += 1;
                ret.typ = LexExprTokenType::Assignment;
                ret.data = LexTknData {
                    ass: LexTknDataAss {
                        typ: ExprAssignmentType::Concat,
                    },
                };
            } else {
                ret.typ = LexExprTokenType::Dot;
            }
        }

        // EOC characters.
        0 | b'\n' => {
            if flags & (1 << 4) != 0 {
                // kELFlagForbidEOC
                ret.typ = LexExprTokenType::Invalid;
                ret.data = LexTknData {
                    err: LexTknDataErr {
                        typ: LexExprTokenType::Spacing,
                        msg: b"E15: Unexpected EOC character: %.*s\0"
                            .as_ptr()
                            .cast::<c_char>(),
                    },
                };
            } else {
                ret.typ = LexExprTokenType::EOC;
            }
        }

        // Or operator or EOC.
        b'|' => {
            if pline.size >= 2 && data[1] == b'|' {
                ret.len += 1;
                ret.typ = LexExprTokenType::Or;
            } else if flags & (1 << 4) != 0 {
                // kELFlagForbidEOC
                ret.typ = LexExprTokenType::Invalid;
                ret.data = LexTknData {
                    err: LexTknDataErr {
                        typ: LexExprTokenType::Or,
                        msg: b"E15: Unexpected EOC character: %.*s\0"
                            .as_ptr()
                            .cast::<c_char>(),
                    },
                };
            } else {
                ret.typ = LexExprTokenType::EOC;
            }
        }

        // Everything else is invalid.
        _ => {
            ret.len = unsafe { utfc_ptr2len_len(pline.data, pline.size as c_int) as usize };
            ret.typ = LexExprTokenType::Invalid;
            ret.data = LexTknData {
                err: LexTknDataErr {
                    typ: LexExprTokenType::PlainIdentifier,
                    msg: b"E15: Unidentified character: %.*s\0"
                        .as_ptr()
                        .cast::<c_char>(),
                },
            };
        }
    }

    adv_return(pstate, ret, flags)
}

// ---------------------------------------------------------------------------
// Helper: advance pointer and return (replaces the goto label in C).
// ---------------------------------------------------------------------------

#[inline]
unsafe fn adv_return(pstate: *mut ParserState, ret: LexExprToken, flags: c_int) -> LexExprToken {
    const KELFLAG_PEEK: c_int = 1 << 0;
    if flags & KELFLAG_PEEK == 0 {
        unsafe { nvim_viml_parser_advance(pstate, ret.len) };
    }
    ret
}

// ---------------------------------------------------------------------------
// Helper: read current parser position directly.
// We replicate the trivial pstate->pos read without a full C accessor to avoid
// overhead on the hot path; ParserState's pos field is at a fixed offset.
//
// But we do it via the accessor to keep it safe:
// ---------------------------------------------------------------------------

#[inline]
unsafe fn parser_get_pos_raw(pstate: *mut ParserState) -> ParserPosition {
    // We call the C accessor declared in parser.c.
    extern "C" {
        fn nvim_parser_get_pos(pstate: *const ParserState) -> ParserPosition;
    }
    unsafe { nvim_parser_get_pos(pstate.cast_const()) }
}

// ---------------------------------------------------------------------------
// GET_CCS helper: read optional case-comparison suffix.
// ---------------------------------------------------------------------------

/// Reads an optional '#' or '?' suffix from the token and returns the
/// corresponding ExprCaseCompareStrategy.  Updates `len` in place.
#[inline]
fn get_ccs(len: &mut usize, data: &[u8], size: usize) -> ExprCaseCompareStrategy {
    if *len < size {
        match data[*len] {
            b'#' => {
                *len += 1;
                ExprCaseCompareStrategy::MatchCase
            }
            b'?' => {
                *len += 1;
                ExprCaseCompareStrategy::IgnoreCase
            }
            _ => ExprCaseCompareStrategy::UseOption,
        }
    } else {
        ExprCaseCompareStrategy::UseOption
    }
}

// ---------------------------------------------------------------------------
// lex_number — handles number tokens (integer and float).
// ---------------------------------------------------------------------------

fn lex_number(mut ret: LexExprToken, data: &[u8], size: usize, flags: c_int) -> LexExprToken {
    ret.data = LexTknData {
        num: LexTknDataNum {
            val: LexTknNumVal { integer: 0 },
            base: 10,
            is_float: false,
        },
    };

    // Consume integer digits first.
    while ret.len < size && ascii_isdigit(data[ret.len]) {
        ret.len += 1;
    }

    const KELFLAG_ALLOW_FLOAT: c_int = 1 << 2;
    if flags & KELFLAG_ALLOW_FLOAT != 0 {
        let non_float_ret = ret;

        // Check for decimal point followed by a digit.
        if size > ret.len + 1 && data[ret.len] == b'.' && ascii_isdigit(data[ret.len + 1]) {
            ret.len += 1;
            let frac_start = ret.len;
            let mut frac_end = ret.len;
            ret.data.num.is_float = true;

            while ret.len < size && ascii_isdigit(data[ret.len]) {
                if data[ret.len] != b'0' {
                    frac_end = ret.len + 1;
                }
                ret.len += 1;
            }

            // Optional exponent.
            let mut exp_start = 0usize;
            let mut exp_negative = false;
            if size > ret.len + 1
                && (data[ret.len] == b'e' || data[ret.len] == b'E')
                && ((size > ret.len + 2
                    && (data[ret.len + 1] == b'+' || data[ret.len + 1] == b'-')
                    && ascii_isdigit(data[ret.len + 2]))
                    || ascii_isdigit(data[ret.len + 1]))
            {
                ret.len += 1;
                if data[ret.len] == b'+' || {
                    exp_negative = data[ret.len] == b'-';
                    exp_negative
                } {
                    ret.len += 1;
                }
                exp_start = ret.len;
                while ret.len < size && ascii_isdigit(data[ret.len]) {
                    ret.len += 1;
                }
            }

            // Reject float if followed by dot or alpha (e.g. "1.0.0" or "1.0x").
            if size > ret.len && (data[ret.len] == b'.' || ascii_isalpha(data[ret.len])) {
                ret = non_float_ret;
            } else {
                // Parse the float value.
                let frac_size = frac_end - frac_start;
                let mut significand_part: f64 = 0.0;
                for i in 0..frac_end {
                    if i == frac_start - 1 {
                        continue;
                    }
                    significand_part = significand_part.mul_add(10.0, f64::from(data[i] - b'0'));
                }

                let mut exp_part: u64 = 0;
                if exp_start != 0 {
                    // Parse exponent with vim_str2nr.
                    unsafe {
                        vim_str2nr(
                            data.as_ptr().add(exp_start).cast::<c_char>(),
                            std::ptr::null_mut(),
                            std::ptr::null_mut(),
                            0,
                            std::ptr::null_mut(),
                            &mut exp_part,
                            (ret.len - exp_start) as c_int,
                            false,
                            std::ptr::null_mut(),
                        );
                    }
                }

                if exp_negative {
                    exp_part += frac_size as u64;
                } else if exp_part < frac_size as u64 {
                    exp_negative = true;
                    exp_part = frac_size as u64 - exp_part;
                } else {
                    exp_part -= frac_size as u64;
                }

                let floating = scale_number(significand_part, 10, exp_part, exp_negative);
                ret.data.num.val.floating = floating;
                return ret;
            }
        }
    }

    // Integer: use vim_str2nr.
    let mut prep: c_int = 0;
    let mut len: c_int = 0;
    let mut integer: u64 = 0;
    unsafe {
        vim_str2nr(
            data.as_ptr().cast::<c_char>(),
            &mut prep,
            &mut len,
            STR2NR_ALL,
            std::ptr::null_mut(),
            &mut integer,
            size as c_int,
            false,
            std::ptr::null_mut(),
        );
    }
    ret.len = len as usize;
    let base: c_uchar = match prep {
        48 => 8,        // b'0' => octal
        120 | 88 => 16, // b'x' | b'X' => hex
        98 | 66 => 2,   // b'b' | b'B' => binary
        _ => 10,        // 0 (no prefix) or anything else => decimal
    };
    ret.data = LexTknData {
        num: LexTknDataNum {
            val: LexTknNumVal { integer },
            base,
            is_float: false,
        },
    };
    ret
}

// ---------------------------------------------------------------------------
// lex_ident — handles identifiers, scoped vars, "is"/"isnot" operators.
// ---------------------------------------------------------------------------

fn lex_ident(
    mut ret: LexExprToken,
    schar: u8,
    data: &[u8],
    size: usize,
    flags: c_int,
) -> LexExprToken {
    ret.data = LexTknData {
        var: LexTknDataVar {
            scope: ExprVarScope::Missing,
            autoload: false,
        },
    };
    ret.typ = LexExprTokenType::PlainIdentifier;

    // Consume ident chars.
    while ret.len < size && ascii_isident(data[ret.len]) {
        ret.len += 1;
    }

    const KELFLAG_IS_NOT_CMP: c_int = 1 << 3;
    const KELFLAG_FORBID_SCOPE: c_int = 1 << 1;

    // "is" / "isnot" comparison operators.
    if flags & KELFLAG_IS_NOT_CMP == 0 {
        if (ret.len == 2 && data[..2] == *b"is") || (ret.len == 5 && data[..5] == *b"isnot") {
            ret.typ = LexExprTokenType::Comparison;
            let inv = ret.len == 5;
            let ccs = get_ccs(&mut ret.len, data, size);
            ret.data = LexTknData {
                cmp: LexTknDataCmp {
                    typ: ExprComparisonType::Identical,
                    ccs,
                    inv,
                },
            };
            return ret;
        }
    }

    // Scope character: s:, g:, etc.
    if ret.len == 1
        && size > 1
        && EXPR_VAR_SCOPE_LIST.contains(&schar)
        && data[1] == b':'
        && flags & KELFLAG_FORBID_SCOPE == 0
    {
        ret.len += 1; // consume ':'
        let scope = match schar {
            b's' => ExprVarScope::Script,
            b'g' => ExprVarScope::Global,
            b'v' => ExprVarScope::Vim,
            b'b' => ExprVarScope::Buffer,
            b'w' => ExprVarScope::Window,
            b't' => ExprVarScope::Tabpage,
            b'l' => ExprVarScope::Local,
            b'a' => ExprVarScope::Arguments,
            _ => ExprVarScope::Missing,
        };
        // Consume identifier chars (including autoload #).
        while ret.len < size && (ascii_isident(data[ret.len]) || data[ret.len] == AUTOLOAD_CHAR) {
            ret.len += 1;
        }
        let autoload = data[2..ret.len].contains(&AUTOLOAD_CHAR);
        ret.data = LexTknData {
            var: LexTknDataVar { scope, autoload },
        };
        return ret;
    }

    // Plain ident with autoload character.
    if size > ret.len && data[ret.len] == AUTOLOAD_CHAR {
        while ret.len < size && (ascii_isident(data[ret.len]) || data[ret.len] == AUTOLOAD_CHAR) {
            ret.len += 1;
        }
        ret.data = LexTknData {
            var: LexTknDataVar {
                scope: ExprVarScope::Missing,
                autoload: true,
            },
        };
    }

    ret
}

// ---------------------------------------------------------------------------
// lex_option — handles &optionname tokens.
// ---------------------------------------------------------------------------

fn lex_option(mut ret: LexExprToken, data: &[u8], size: usize) -> LexExprToken {
    // "&&" is logical-and.
    if size > 1 && data[1] == b'&' {
        ret.typ = LexExprTokenType::And;
        ret.len += 1;
        return ret;
    }

    // Must have an alpha char after '&'.
    if size == 1 || !ascii_isalpha(data[1]) {
        ret.typ = LexExprTokenType::Invalid;
        ret.data = LexTknData {
            err: LexTknDataErr {
                typ: LexExprTokenType::Option,
                msg: b"E112: Option name missing: %.*s\0"
                    .as_ptr()
                    .cast::<c_char>(),
            },
        };
        return ret;
    }

    ret.typ = LexExprTokenType::Option;

    // Optional scope prefix: &l: or &g:
    let (scope, name_start) =
        if size > 2 && data[2] == b':' && EXPR_OPT_SCOPE_LIST.contains(&data[1]) {
            ret.len += 2;
            let sc = match data[1] {
                b'g' => ExprOptScope::Global,
                b'l' => ExprOptScope::Local,
                _ => ExprOptScope::Unspecified,
            };
            (sc, 3usize) // name starts at data[3]
        } else {
            (ExprOptScope::Unspecified, 1usize) // name starts at data[1]
        };

    let name_ptr = unsafe { data.as_ptr().add(name_start).cast::<c_char>() };

    // Terminal key option: t_XX (4 chars)
    let opt_len =
        if size >= name_start + 4 && data[name_start] == b't' && data[name_start + 1] == b'_' {
            ret.len += 4;
            4usize
        } else {
            let mut p = name_start;
            while p < size && ascii_isalpha(data[p]) {
                p += 1;
            }
            let l = p - name_start;
            if l == 0 {
                ret.typ = LexExprTokenType::Invalid;
                ret.data = LexTknData {
                    err: LexTknDataErr {
                        typ: LexExprTokenType::Option,
                        msg: b"E112: Option name missing: %.*s\0"
                            .as_ptr()
                            .cast::<c_char>(),
                    },
                };
                return ret;
            }
            ret.len += l;
            l
        };

    ret.data = LexTknData {
        opt: LexTknDataOpt {
            name: name_ptr,
            len: opt_len,
            scope,
        },
    };
    ret
}
