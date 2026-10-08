//! Scanning one token out of the Vimscript expression input.
//!
//! `viml_pexpr_next_token` is the module's only export. It asks the reader for
//! the rest of the current line and hands those bytes to [`scan`] as a slice;
//! everything below that point is ordinary indexing rather than the C's
//! `const char *` walk, so the bounds are the compiler's problem.
//!
#![forbid(unsafe_code)]

use crate::charset::{Str2NrBases, str2nr_in};
use crate::mbyte::cluster_len;
use core::ffi::{CStr, c_int};

use super::*;
use crate::ascii::{ascii_isident, ascii_iswhite};

/// The character that separates the parts of an autoload name, `foo#bar`.
const AUTOLOAD_CHAR: u8 = b'#';

/// The scope letters an identifier may carry before its colon, as in `g:foo`.
/// (The C's `EXPR_VAR_SCOPE_LIST` lists `b` twice; the duplicate is inert.)
const VAR_SCOPES: [ExprVarScope; 8] = [
    kExprVarScopeScript,
    kExprVarScopeGlobal,
    kExprVarScopeVim,
    kExprVarScopeBuffer,
    kExprVarScopeWindow,
    kExprVarScopeTabpage,
    kExprVarScopeLocal,
    kExprVarScopeArguments,
];

/// The scope letters an option may carry before its colon, as in `&g:sw`.
const OPT_SCOPES: [ExprOptScope; 2] = [kExprOptScopeGlobal, kExprOptScopeLocal];

use super::ast::translate;

/// How many bytes the first character of `line` occupies, composing marks
/// included, without reading past the line's end. `line` does not start
/// with a NUL (the scanner's own arm takes that), which is the one byte the
/// pointer form measured differently.
fn first_char_len(line: &[u8]) -> size_t {
    cluster_len(line)
}

/// The number at the start of `bytes`: its unsigned value, the prefix letter
/// that named its base and how many bytes it took.
fn str2nr(bytes: &[u8], what: Str2NrBases) -> (UVarNumber, c_int, size_t) {
    debug_assert!(!bytes.is_empty(), "vim_str2nr reads the first byte eagerly");
    let parsed = str2nr_in(bytes, what, false);
    (parsed.magnitude, parsed.prefix, parsed.len)
}

/// A token that has consumed nothing, positioned at `start`.
///
/// The C's partial initializer (`LexExprToken ret = { .type = ..., .start =
/// ... }`) zeroes the entire union, and a scanner that then fills in one
/// member leaves the rest of the larger ones zeroed — which the parser reads
/// back in the two places [`LexExprTokenData`] describes. Carrying no
/// payload at all says the same thing, and the accessors answer the same
/// zeroes for it.
fn blank_token(start: ParserPosition) -> LexExprToken {
    LexExprToken {
        start,
        len: 0,
        type_0: kExprLexInvalid,
        data: LexExprTokenData::Blank,
    }
}

/// Scale `num` by `base` raised to `exponent`, by repeated squaring.
///
/// `base` must not be zero.
#[inline(always)]
fn scale_number(num: Float, base: uint8_t, exponent: UVarNumber, exponent_negative: bool) -> Float {
    if num == 0.0 || exponent == 0 {
        return num;
    }
    debug_assert!(base != 0, "base");
    let mut exp = exponent;
    let mut p_base = Float::from(base);
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

/// The index of the first byte at or after `from` that `accept` rejects, or
/// the end of the line.
fn scan_while(line: &[u8], from: size_t, accept: impl Fn(u8) -> bool) -> size_t {
    let mut i = from;
    while i < line.len() && accept(line[i]) {
        i += 1;
    }
    i
}

/// The base `vim_str2nr` reports through its prefix letter.
///
/// The C indexes a designated-initializer table that names only `0`, `x`/`X`
/// and `b`/`B`, so an `0o…` literal answers **zero** rather than eight. Kept:
/// the only reader is `values.rs`'s `base_to_prefix_length`, where zero and
/// ten mean the same thing (no prefix to highlight).
fn base_for_prefix(prefix: c_int) -> uint8_t {
    match u8::try_from(prefix).unwrap_or(0) {
        0 => 10,
        b'0' => 8,
        b'x' | b'X' => 16,
        b'b' | b'B' => 2,
        _ => 0,
    }
}

/// Consume the case-sensitivity marker a comparison operator may end with,
/// and answer the strategy it names.
///
/// The C tests `strchr("?#", c)`, which also answers non-NULL for the
/// terminating NUL: a NUL byte here is consumed as a strategy of zero, which
/// is what `kCCStrategyUseOption` is anyway. Only the token's length differs,
/// and it differs in the C too.
///
/// `!` and `=` reach this with no comparison to put the answer in, and the
/// C writes it into the union regardless; only the length it consumes is
/// read back, so those callers drop it.
fn scan_ccs(ret: &mut LexExprToken, line: &[u8]) -> ExprCaseCompareStrategy {
    if ret.len < line.len() && matches!(line[ret.len], b'?' | b'#' | b'\0') {
        let ccs = ExprCaseCompareStrategy::from(line[ret.len]);
        ret.len += 1;
        ccs
    } else {
        kCCStrategyUseOption
    }
}

/// Report that `&` was not followed by an option name.
///
/// `scope` is what the `&g:` prefix named, which the highlight still covers:
/// see [`LexExprTokenError::opt_scope`].
fn option_name_missing(ret: &mut LexExprToken, scope: ExprOptScope) {
    ret.type_0 = kExprLexInvalid;
    ret.data = LexExprTokenData::Error(LexExprTokenError {
        type_0: kExprLexOption,
        msg: translate(c"E112: Option name missing: %.*s"),
        opt_scope: scope,
    });
}

/// A number literal: an integer in whichever base its prefix names, or a
/// float when `kELFlagAllowFloat` lets one through.
fn scan_number(ret: &mut LexExprToken, line: &[u8], flags: c_int) {
    let mut is_float = false;
    let mut base: uint8_t = 10;
    let mut frac_start: size_t = 0;
    let mut frac_end: size_t = 0;
    let mut exp_start: size_t = 0;
    let mut exp_negative = false;
    ret.type_0 = kExprLexNumber;
    ret.len = scan_while(line, ret.len, |b| b.is_ascii_digit());
    if flags & kELFlagAllowFloat as c_int != 0 {
        let non_float = *ret;
        if line.len() > ret.len + 1 && line[ret.len] == b'.' && line[ret.len + 1].is_ascii_digit() {
            ret.len += 1;
            frac_start = ret.len;
            frac_end = ret.len;
            is_float = true;
            while ret.len < line.len() && line[ret.len].is_ascii_digit() {
                // Trailing zeroes in the fraction add nothing to the
                // significand, so they are left out of `frac_end`.
                if line[ret.len] != b'0' {
                    frac_end = ret.len + 1;
                }
                ret.len += 1;
            }
            let has_exponent = line.len() > ret.len + 1
                && (line[ret.len] == b'e' || line[ret.len] == b'E')
                && ((line.len() > ret.len + 2
                    && (line[ret.len + 1] == b'+' || line[ret.len + 1] == b'-')
                    && line[ret.len + 2].is_ascii_digit())
                    || line[ret.len + 1].is_ascii_digit());
            if has_exponent {
                ret.len += 1;
                exp_negative = line[ret.len] == b'-';
                if exp_negative || line[ret.len] == b'+' {
                    ret.len += 1;
                }
                exp_start = ret.len;
                ret.type_0 = kExprLexNumber;
                ret.len = scan_while(line, ret.len, |b| b.is_ascii_digit());
            }
        }
        // A `.` or a letter right after the number means this was not a
        // float after all: `1.2.3` is a concatenation, `1.2x` a syntax error.
        if line.len() > ret.len && (line[ret.len] == b'.' || line[ret.len].is_ascii_alphabetic()) {
            *ret = non_float;
            is_float = false;
        }
    }
    // TODO(ZyX-I): detect overflows
    let val = if is_float {
        // Vim used to call string2float, i.e. strtod(), which is
        // locale-dependent and takes no length. This is uClibc's approach
        // instead: accumulate the digits ignoring the decimal point, then use
        // the point's position to scale the result when applying the
        // exponent.
        let mut significand: Float = 0.0;
        let frac_size = frac_end - frac_start;
        for (i, &byte) in line[..frac_end].iter().enumerate() {
            if i == frac_start - 1 {
                continue; // the decimal point
            }
            significand = significand * 10.0 + Float::from(byte - b'0');
        }
        let mut exp_part: UVarNumber = if exp_start != 0 {
            str2nr(&line[exp_start..ret.len], Str2NrBases::NONE).0
        } else {
            0
        };
        if exp_negative {
            exp_part = exp_part.wrapping_add(frac_size as UVarNumber);
        } else if exp_part < frac_size as UVarNumber {
            exp_negative = true;
            exp_part = (frac_size as UVarNumber).wrapping_sub(exp_part);
        } else {
            exp_part = exp_part.wrapping_sub(frac_size as UVarNumber);
        }
        LexExprTokenNumberValue::Floating(scale_number(significand, 10, exp_part, exp_negative))
    } else {
        let (value, prefix, len) = str2nr(line, Str2NrBases::ALL);
        ret.len = len;
        base = base_for_prefix(prefix);
        LexExprTokenNumberValue::Integer(value)
    };
    ret.data = LexExprTokenData::Number(LexExprTokenNumber { val, base });
}

/// A name: a variable or function, possibly scoped (`g:foo`) or autoloaded
/// (`foo#bar`) — or the `is`/`isnot` comparison operators, which are spelled
/// like one.
fn scan_identifier(ret: &mut LexExprToken, line: &[u8], flags: c_int) {
    let schar = line[0];
    let mut scope = kExprVarScopeMissing;
    let mut autoload = false;
    ret.type_0 = kExprLexPlainIdentifier;
    ret.len = scan_while(line, ret.len, |b| ascii_isident(b.into()));
    if flags & kELFlagIsNotCmp as c_int == 0
        && ((ret.len == 2 && &line[..2] == b"is") || (ret.len == 5 && &line[..5] == b"isnot"))
    {
        ret.type_0 = kExprLexComparison;
        let inv = ret.len == 5;
        let ccs = scan_ccs(ret, line);
        ret.data = LexExprTokenData::Comparison(LexExprTokenComparison {
            type_0: kExprCmpIdentical,
            ccs,
            inv,
        });
        return;
    }
    if ret.len == 1
        && line.len() > 1
        && VAR_SCOPES.contains(&ExprVarScope::from(schar))
        && line[1] == b':'
        && flags & kELFlagForbidScope as c_int == 0
    {
        ret.len += 1;
        scope = ExprVarScope::from(schar);
        ret.type_0 = kExprLexPlainIdentifier;
        // The scan above stopped at the first `#` so that `is#` could be read
        // as a comparison; from here autoload characters belong to the name.
        //
        // The ambiguity that leaves is the lexer's to hand on: `is#Foo(1)` is
        // a call of `is#Foo()`, `1is#Foo(1)` is `1 is# Foo(1)`. Only the
        // parser, which has the context, can tell them apart.
        ret.len = scan_while(line, ret.len, |b| {
            ascii_isident(b.into()) || b == AUTOLOAD_CHAR
        });
        autoload = line[2..ret.len].contains(&AUTOLOAD_CHAR);
    } else if line.len() > ret.len && line[ret.len] == AUTOLOAD_CHAR {
        autoload = true;
        ret.type_0 = kExprLexPlainIdentifier;
        ret.len = scan_while(line, ret.len, |b| {
            ascii_isident(b.into()) || b == AUTOLOAD_CHAR
        });
    }
    ret.data = LexExprTokenData::Var(LexExprTokenVar { scope, autoload });
}

/// `&&`, or an option name with an optional `g:`/`l:` scope.
fn scan_option(ret: &mut LexExprToken, line: &[u8]) {
    if line.len() > 1 && line[1] == b'&' {
        ret.type_0 = kExprLexAnd;
        ret.len += 1;
        return;
    }
    if line.len() == 1 || !line[1].is_ascii_alphabetic() {
        option_name_missing(ret, kExprOptScopeUnspecified);
        return;
    }
    ret.type_0 = kExprLexOption;
    let (name_at, scope): (size_t, ExprOptScope) =
        if line.len() > 2 && line[2] == b':' && OPT_SCOPES.contains(&ExprOptScope::from(line[1])) {
            ret.len += 2;
            (3, ExprOptScope::from(line[1]))
        } else {
            (1, kExprOptScopeUnspecified)
        };
    let name = &line[name_at..];
    let name_len = if name.len() >= 4 && name[0] == b't' && name[1] == b'_' {
        // `t_XY`: a termcap option, whose name is always two bytes after the
        // prefix whether or not they are letters.
        ret.len += 4;
        4
    } else {
        let name_len = scan_while(name, 0, |b| b.is_ascii_alphabetic());
        if name_len == 0 {
            // Replaces the payload `opt` would have had, exactly as the C's
            // `OPTNAMEMISS` does — keeping the scope, which is the one thing
            // the C's union leaves standing.
            option_name_missing(ret, scope);
            return;
        }
        ret.len += name_len;
        name_len
    };
    ret.data = LexExprTokenData::Option(LexExprTokenOption {
        name_offset: name_at,
        len: name_len,
        scope,
    });
}

/// A single-quoted string, which ends at the first `'` that is not doubled.
fn scan_single_quoted(ret: &mut LexExprToken, line: &[u8]) {
    ret.type_0 = kExprLexSingleQuotedString;
    let mut closed = false;
    while ret.len < line.len() && !closed {
        if line[ret.len] == b'\'' {
            if ret.len + 1 < line.len() && line[ret.len + 1] == b'\'' {
                ret.len += 1;
            } else {
                closed = true;
            }
        }
        ret.len += 1;
    }
    ret.data = LexExprTokenData::Str(LexExprTokenString { closed });
}

/// A double-quoted string, which ends at the first `"` that is not escaped.
fn scan_double_quoted(ret: &mut LexExprToken, line: &[u8]) {
    ret.type_0 = kExprLexDoubleQuotedString;
    let mut closed = false;
    while ret.len < line.len() && !closed {
        if line[ret.len] == b'\\' {
            if ret.len + 1 < line.len() {
                ret.len += 1;
            }
        } else if line[ret.len] == b'"' {
            closed = true;
        }
        ret.len += 1;
    }
    ret.data = LexExprTokenData::Str(LexExprTokenString { closed });
}

/// `!` and `=`: unary not, assignment, and the (in)equality and regex-match
/// comparisons the two of them begin.
fn scan_bang_or_equals(ret: &mut LexExprToken, line: &[u8]) {
    let schar = line[0];
    let inv = schar == b'!';
    if line.len() == 1 {
        ret.type_0 = if inv { kExprLexNot } else { kExprLexAssignment };
        ret.data = LexExprTokenData::Assignment(LexExprTokenAssignment {
            type_0: kExprAsgnPlain,
        });
        return;
    }
    let comparison = if line[1] == b'=' {
        ret.len += 1;
        Some(kExprCmpEqual)
    } else if line[1] == b'~' {
        ret.len += 1;
        Some(kExprCmpMatches)
    } else {
        None
    };
    match comparison {
        Some(type_0) => {
            ret.type_0 = kExprLexComparison;
            let ccs = scan_ccs(ret, line);
            ret.data = LexExprTokenData::Comparison(LexExprTokenComparison { type_0, ccs, inv });
        }
        // `!?` and `!#`: the marker is consumed, nothing reads it back, and
        // the token's length is what the consumption was for.
        None if inv => {
            ret.type_0 = kExprLexNot;
            scan_ccs(ret, line);
        }
        None => {
            ret.type_0 = kExprLexAssignment;
            scan_ccs(ret, line);
            ret.data = LexExprTokenData::Assignment(LexExprTokenAssignment {
                type_0: kExprAsgnPlain,
            });
        }
    }
}

/// `<` and `>`, with or without a trailing `=`.
fn scan_ordering(ret: &mut LexExprToken, line: &[u8]) {
    ret.type_0 = kExprLexComparison;
    let has_eq_sign = line.len() > 1 && line[1] == b'=';
    if has_eq_sign {
        ret.len += 1;
    }
    let ccs = scan_ccs(ret, line);
    let inv = line[0] == b'<';
    ret.data = LexExprTokenData::Comparison(LexExprTokenComparison {
        type_0: if inv ^ has_eq_sign {
            kExprCmpGreaterOrEqual
        } else {
            kExprCmpGreater
        },
        ccs,
        inv,
    });
}

/// An opening or closing bracket, brace or parenthesis.
fn brace(closing: bool) -> LexExprTokenData {
    LexExprTokenData::Brace(LexExprTokenBrace { closing })
}

/// One of the augmented assignments, or plain `=`.
fn assignment(type_0: ExprAssignmentType) -> LexExprTokenData {
    LexExprTokenData::Assignment(LexExprTokenAssignment { type_0 })
}

/// An invalid token: what it was trying to be, and the message saying why it
/// is not. No option scope was read, so none is carried.
fn lex_error(type_0: LexExprTokenType, msg: &'static CStr) -> LexExprTokenData {
    LexExprTokenData::Error(LexExprTokenError {
        type_0,
        msg: translate(msg),
        opt_scope: kExprOptScopeUnspecified,
    })
}

/// Scan one token out of `line`, the input from the cursor to the end of the
/// current line. `start` is where the cursor stands; `flags` is a set of
/// `kELFlag*`.
fn scan(line: &[u8], start: ParserPosition, flags: c_int) -> LexExprToken {
    let mut ret = blank_token(start);
    if line.is_empty() {
        ret.type_0 = kExprLexEOC;
        return ret;
    }
    ret.len = 1;
    let schar = line[0];
    match schar {
        b'(' | b')' => {
            ret.type_0 = kExprLexParenthesis;
            ret.data = brace(schar == b')');
        }
        b'[' | b']' => {
            ret.type_0 = kExprLexBracket;
            ret.data = brace(schar == b']');
        }
        b'{' | b'}' => {
            ret.type_0 = kExprLexFigureBrace;
            ret.data = brace(schar == b'}');
        }
        b'?' => ret.type_0 = kExprLexQuestion,
        b':' => ret.type_0 = kExprLexColon,
        b',' => ret.type_0 = kExprLexComma,
        b'*' | b'/' | b'%' => {
            ret.type_0 = kExprLexMultiplication;
            ret.data = LexExprTokenData::Multiplication(LexExprTokenMultiplication {
                type_0: match schar {
                    b'*' => kExprLexMulMul,
                    b'/' => kExprLexMulDiv,
                    _ => kExprLexMulMod,
                },
            });
        }
        b' ' | b'\t' => {
            ret.type_0 = kExprLexSpacing;
            ret.len = scan_while(line, ret.len, |b| ascii_iswhite(b.into()));
        }
        // The control characters, less NUL, TAB and NL, which have their own
        // arms. The C's case list stops at 0x1a, so ESC and the four above it
        // fall through to "unidentified character" — as they do here.
        0x01..=0x08 | 0x0b..=0x1a => {
            ret.type_0 = kExprLexInvalid;
            ret.len = scan_while(line, ret.len, |b| b < b' ');
            ret.data = lex_error(
                kExprLexSpacing,
                c"E15: Invalid control character present in input: %.*s",
            );
        }
        b'0'..=b'9' => scan_number(&mut ret, line, flags),
        b'$' => {
            ret.type_0 = kExprLexEnv;
            ret.len = scan_while(line, ret.len, |b| ascii_isident(b.into()));
        }
        b'a'..=b'z' | b'A'..=b'Z' | b'_' => scan_identifier(&mut ret, line, flags),
        b'&' => scan_option(&mut ret, line),
        b'@' => {
            ret.type_0 = kExprLexRegister;
            let name = if line.len() > 1 {
                ret.len += 1;
                c_int::from(line[1])
            } else {
                -1
            };
            ret.data = LexExprTokenData::Register(LexExprTokenRegister { name });
        }
        b'\'' => scan_single_quoted(&mut ret, line),
        b'"' => scan_double_quoted(&mut ret, line),
        b'!' | b'=' => scan_bang_or_equals(&mut ret, line),
        b'>' | b'<' => scan_ordering(&mut ret, line),
        b'-' => {
            if line.len() > 1 && line[1] == b'>' {
                ret.len += 1;
                ret.type_0 = kExprLexArrow;
            } else if line.len() > 1 && line[1] == b'=' {
                ret.len += 1;
                ret.type_0 = kExprLexAssignment;
                ret.data = assignment(kExprAsgnSubtract);
            } else {
                ret.type_0 = kExprLexMinus;
            }
        }
        b'+' | b'.' => {
            let (plain, augmented) = if schar == b'+' {
                (kExprLexPlus, kExprAsgnAdd)
            } else {
                (kExprLexDot, kExprAsgnConcat)
            };
            if line.len() > 1 && line[1] == b'=' {
                ret.len += 1;
                ret.type_0 = kExprLexAssignment;
                ret.data = assignment(augmented);
            } else {
                ret.type_0 = plain;
            }
        }
        // The Ex command ended, so the expression does too.
        b'\0' | b'\n' => {
            if flags & kELFlagForbidEOC as c_int != 0 {
                ret.type_0 = kExprLexInvalid;
                ret.data = lex_error(kExprLexSpacing, c"E15: Unexpected EOC character: %.*s");
            } else {
                ret.type_0 = kExprLexEOC;
            }
        }
        b'|' => {
            if line.len() >= 2 && line[1] == b'|' {
                ret.len += 1;
                ret.type_0 = kExprLexOr;
            } else if flags & kELFlagForbidEOC as c_int != 0 {
                // Note: `<C-r>=1 | 2<CR>` yields 1 in Vim, with no error at
                // all. That is deliberately not what happens here.
                ret.type_0 = kExprLexInvalid;
                ret.data = lex_error(kExprLexOr, c"E15: Unexpected EOC character: %.*s");
            } else {
                ret.type_0 = kExprLexEOC;
            }
        }
        _ => {
            ret.len = first_char_len(line);
            ret.type_0 = kExprLexInvalid;
            ret.data = lex_error(
                kExprLexPlainIdentifier,
                c"E15: Unidentified character: %.*s",
            );
        }
    }
    ret
}

/// The next token of the Vimscript expression `pstate` is reading, advancing
/// the cursor past it unless `kELFlagPeek` is set.
pub fn viml_pexpr_next_token(pstate: &mut ParserState<'_>, flags: c_int) -> LexExprToken {
    let start = pstate.pos;
    let Some(line) = pstate.remaining_line() else {
        let mut ret = blank_token(start);
        ret.type_0 = kExprLexEOC;
        return ret;
    };
    let ret = scan(line, start, flags);
    if flags & kELFlagPeek as c_int == 0 {
        pstate.advance(ret.len);
    }
    ret
}
