// VimL token representation — Rust replacement for viml_pexpr_repr_token().
//
// The C function uses a static 1024-byte buffer and goto for early exit.
// The Rust version uses a write-cursor with an early-return pattern instead.

#![allow(clippy::too_many_lines)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::write_with_newline)]

use std::ffi::{c_char, c_int};
use std::fmt::Write as FmtWrite;

use crate::expr_types::{ExprCaseCompareStrategy, ExprOptScope, LexExprToken, LexExprTokenType};
use crate::lexer::ParserState;

// ---------------------------------------------------------------------------
// String tables used only by repr_token (local, not exported).
// ---------------------------------------------------------------------------

static ELTKN_TYPE_TAB: &[&str] = &[
    "Invalid",            // kExprLexInvalid = 0
    "Missing",            // kExprLexMissing
    "Spacing",            // kExprLexSpacing
    "EOC",                // kExprLexEOC
    "Question",           // kExprLexQuestion
    "Colon",              // kExprLexColon
    "Or",                 // kExprLexOr
    "And",                // kExprLexAnd
    "Comparison",         // kExprLexComparison
    "Plus",               // kExprLexPlus
    "Minus",              // kExprLexMinus
    "Dot",                // kExprLexDot
    "Multiplication",     // kExprLexMultiplication
    "Not",                // kExprLexNot
    "Number",             // kExprLexNumber
    "SingleQuotedString", // kExprLexSingleQuotedString
    "DoubleQuotedString", // kExprLexDoubleQuotedString
    "Option",             // kExprLexOption
    "Register",           // kExprLexRegister
    "Env",                // kExprLexEnv
    "PlainIdentifier",    // kExprLexPlainIdentifier
    "Bracket",            // kExprLexBracket
    "FigureBrace",        // kExprLexFigureBrace
    "Parenthesis",        // kExprLexParenthesis
    "Comma",              // kExprLexComma
    "Arrow",              // kExprLexArrow
    "Assignment",         // kExprLexAssignment
];

static ELTKN_CMP_TYPE_TAB: &[&str] = &[
    "Equal",          // kExprCmpEqual = 0
    "Matches",        // kExprCmpMatches
    "Greater",        // kExprCmpGreater
    "GreaterOrEqual", // kExprCmpGreaterOrEqual
    "Identical",      // kExprCmpIdentical
];

static CCS_TAB: [&str; 64] = {
    let mut arr = [""; 64];
    arr[0] = "UseOption"; // kCCStrategyUseOption = 0
    arr[35] = "MatchCase"; // kCCStrategyMatchCase = '#'
    arr[63] = "IgnoreCase"; // kCCStrategyIgnoreCase = '?'
    arr
};

static ELTKN_MUL_TYPE_TAB: &[&str] = &[
    "Mul", // kExprLexMulMul = 0
    "Div", // kExprLexMulDiv
    "Mod", // kExprLexMulMod
];

static ELTKN_OPT_SCOPE_TAB: [&str; 256] = {
    // Indexed by ExprOptScope discriminant value (b'g'=103, b'l'=108, 0=Unspecified)
    let mut arr = [""; 256];
    arr[0] = "Unspecified";
    arr[b'g' as usize] = "Global";
    arr[b'l' as usize] = "Local";
    arr
};

static EXPR_ASGN_TYPE_TAB: &[&str] = &[
    "Plain",    // kExprAsgnPlain = 0
    "Add",      // kExprAsgnAdd
    "Subtract", // kExprAsgnSubtract
    "Concat",   // kExprAsgnConcat
];

// ---------------------------------------------------------------------------
// Extern "C" accessor for pstate->reader.lines
// ---------------------------------------------------------------------------

extern "C" {
    /// Get the data pointer and size for a reader line at `line_idx`.
    fn nvim_parser_get_line_data(
        pstate: *const ParserState,
        line_idx: usize,
        data_out: *mut *const c_char,
        size_out: *mut usize,
    );
}

// ---------------------------------------------------------------------------
// intchar2str helper
// ---------------------------------------------------------------------------

/// Convert an int character to a printable string representation.
///
/// - ASCII digits → `'{digit}'`
/// - Other printable ASCII → single character string
/// - Everything else → decimal integer string
///
/// Uses a thread-local buffer to avoid `static mut` unsafety.
fn intchar2str(ch: c_int) -> String {
    if ch >= 0x20 && ch < 0x7f {
        let c = ch as u8;
        if c.is_ascii_digit() {
            format!("'{}'", c as char)
        } else {
            format!("{}", c as char)
        }
    } else {
        format!("{ch}")
    }
}

// ---------------------------------------------------------------------------
// viml_pexpr_repr_token
// ---------------------------------------------------------------------------

/// Represent token as a string (for testing/debugging).
///
/// Returns a pointer into a static 1024-byte buffer (overwritten each call).
/// When `pstate` is NULL, appends `::{len}` instead of the token text.
///
/// # Safety
/// - `token` must be a valid `LexExprToken`.
/// - `pstate` may be NULL; if non-NULL, must point to a valid C `ParserState`.
/// - `ret_size` may be NULL; if non-NULL, receives the byte count of the result.
#[unsafe(export_name = "viml_pexpr_repr_token")]
pub unsafe extern "C" fn viml_pexpr_repr_token(
    pstate: *const ParserState,
    token: LexExprToken,
    ret_size: *mut usize,
) -> *const c_char {
    // Static buffer, matching C's `static char ret[1024]`.
    static mut RET: [u8; 1024] = [0u8; 1024];

    // Build the string into a Rust String first, then copy into the static buffer.
    let mut s = String::with_capacity(256);

    let typ_idx = token.typ as usize;
    let typ_str = if typ_idx < ELTKN_TYPE_TAB.len() {
        ELTKN_TYPE_TAB[typ_idx]
    } else {
        "Unknown"
    };

    let _ = write!(s, "{}:{}:{}", token.start.line, token.start.col, typ_str);

    // Token-type-specific data.
    match token.typ {
        LexExprTokenType::Comparison => {
            let cmp = unsafe { token.data.cmp };
            let cmp_type = cmp.typ as usize;
            let cmp_str = if cmp_type < ELTKN_CMP_TYPE_TAB.len() {
                ELTKN_CMP_TYPE_TAB[cmp_type]
            } else {
                "Unknown"
            };
            let ccs_idx = cmp.ccs as usize;
            let ccs_str = if ccs_idx < CCS_TAB.len() {
                CCS_TAB[ccs_idx]
            } else {
                ""
            };
            let _ = write!(s, "(type={cmp_str},ccs={ccs_str},inv={})", cmp.inv as c_int);
        }
        LexExprTokenType::Multiplication => {
            let mul = unsafe { token.data.mul };
            let mul_idx = mul.typ as usize;
            let mul_str = if mul_idx < ELTKN_MUL_TYPE_TAB.len() {
                ELTKN_MUL_TYPE_TAB[mul_idx]
            } else {
                "Unknown"
            };
            let _ = write!(s, "(type={mul_str})");
        }
        LexExprTokenType::Assignment => {
            let ass = unsafe { token.data.ass };
            let ass_idx = ass.typ as usize;
            let ass_str = if ass_idx < EXPR_ASGN_TYPE_TAB.len() {
                EXPR_ASGN_TYPE_TAB[ass_idx]
            } else {
                "Unknown"
            };
            let _ = write!(s, "(type={ass_str})");
        }
        LexExprTokenType::Register => {
            let reg = unsafe { token.data.reg };
            let _ = write!(s, "(name={})", intchar2str(reg.name));
        }
        LexExprTokenType::SingleQuotedString | LexExprTokenType::DoubleQuotedString => {
            let str_ = unsafe { token.data.str_ };
            let _ = write!(s, "(closed={})", str_.closed as c_int);
        }
        LexExprTokenType::Option => {
            let opt = unsafe { token.data.opt };
            let scope_idx = opt.scope as usize;
            let scope_str = if scope_idx < ELTKN_OPT_SCOPE_TAB.len() {
                ELTKN_OPT_SCOPE_TAB[scope_idx]
            } else {
                "Unknown"
            };
            // opt.name is a pointer into the parser line data (not NUL-terminated beyond opt.len)
            let name_slice = unsafe { std::slice::from_raw_parts(opt.name.cast::<u8>(), opt.len) };
            let name_str = std::str::from_utf8(name_slice).unwrap_or("<invalid>");
            let _ = write!(s, "(scope={scope_str},name={name_str})");
        }
        LexExprTokenType::PlainIdentifier => {
            let var = unsafe { token.data.var };
            let scope_val = var.scope as c_int;
            let _ = write!(
                s,
                "(scope={},autoload={})",
                intchar2str(scope_val),
                var.autoload as c_int
            );
        }
        LexExprTokenType::Number => {
            let num = unsafe { token.data.num };
            let val_f64: f64 = if num.is_float {
                unsafe { num.val.floating }
            } else {
                unsafe { num.val.integer as f64 }
            };
            let _ = write!(
                s,
                "(is_float={},base={},val={})",
                num.is_float as c_int, num.base, val_f64
            );
        }
        LexExprTokenType::Invalid => {
            let err = unsafe { token.data.err };
            // err.msg is a C string literal
            let msg = if err.msg.is_null() {
                ""
            } else {
                let cstr = unsafe { std::ffi::CStr::from_ptr(err.msg) };
                cstr.to_str().unwrap_or("<invalid>")
            };
            let _ = write!(s, "(msg={msg})");
        }
        _ => {
            // No additional arguments.
        }
    }

    // Append the token text (or length if pstate is NULL).
    if pstate.is_null() {
        let _ = write!(s, "::{}", token.len);
    } else {
        s.push(':');
        if token.len > 0 {
            let mut data_ptr: *const c_char = std::ptr::null();
            let mut line_size: usize = 0;
            unsafe {
                nvim_parser_get_line_data(
                    pstate,
                    token.start.line,
                    &raw mut data_ptr,
                    &raw mut line_size,
                );
            }
            let _ = line_size; // used as out-param, value not needed here
            if !data_ptr.is_null() {
                let token_data = unsafe {
                    std::slice::from_raw_parts(
                        data_ptr.add(token.start.col).cast::<u8>(),
                        token.len,
                    )
                };
                // Append as raw bytes (may not be valid UTF-8).
                s.extend(token_data.iter().map(|&b| b as char));
            }
        }
    }

    // Copy into static buffer, truncating at 1023 to leave room for NUL.
    let bytes = s.as_bytes();
    let copy_len = bytes.len().min(1023);
    // Use raw pointer operations to avoid creating references to static mut.
    let ret_ptr: *mut u8 = std::ptr::addr_of_mut!(RET).cast::<u8>();
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ret_ptr, copy_len);
        ret_ptr.add(copy_len).write(0u8);
        if !ret_size.is_null() {
            *ret_size = copy_len;
        }
        ret_ptr.cast::<c_char>()
    }
}

// ---------------------------------------------------------------------------
// eltkn_type_tab and eltkn_mul_type_tab — also used by ExprOptScope.
// Exported as module-level items but not as #[no_mangle] (C doesn't need them).
// ---------------------------------------------------------------------------

/// Get the string name of a token type (for use by tests/repr).
#[must_use]
#[inline]
pub fn token_type_name(typ: LexExprTokenType) -> &'static str {
    let idx = typ as usize;
    if idx < ELTKN_TYPE_TAB.len() {
        ELTKN_TYPE_TAB[idx]
    } else {
        "Unknown"
    }
}

/// Get the string name of an option scope.
#[must_use]
#[inline]
pub fn opt_scope_name(scope: ExprOptScope) -> &'static str {
    let idx = scope as usize;
    if idx < ELTKN_OPT_SCOPE_TAB.len() {
        ELTKN_OPT_SCOPE_TAB[idx]
    } else {
        "Unknown"
    }
}

/// Get the string name of a comparison case strategy.
#[must_use]
#[inline]
pub fn ccs_name(ccs: ExprCaseCompareStrategy) -> &'static str {
    let idx = ccs as usize;
    if idx < CCS_TAB.len() {
        CCS_TAB[idx]
    } else {
        ""
    }
}
