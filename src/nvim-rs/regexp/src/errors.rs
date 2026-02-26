//! Error reporting functions for the regexp engine.
//!
//! Consolidates all error-message wrappers that were previously C functions in
//! `regexp_shim.c`. Each function calls `gettext()` on the English message string
//! (which serves as the gettext msgid) then dispatches to `emsg()`/`semsg()`/etc.
//!
//! The English strings are copied byte-for-byte from the C source so that
//! gettext lookup is identical regardless of where the string literal lives.

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_long};

extern "C" {
    fn emsg(s: *const c_char);
    fn semsg(fmt: *const c_char, ...);
    fn iemsg(s: *const c_char);
    fn siemsg(fmt: *const c_char, ...);
    fn internal_error(msg: *const c_char);
    // gettext translation
    fn gettext(msgid: *const c_char) -> *const c_char;
    // rc_did_emsg flag accessor (keeps the flag-setter in C where rc_did_emsg lives)
    pub fn nvim_regexp_set_rc_did_emsg(v: c_int);
}

// ---------------------------------------------------------------------------
// Inline helper: translate an English string via gettext at runtime.
// ---------------------------------------------------------------------------
#[inline]
pub unsafe fn gt(s: *const c_char) -> *const c_char {
    gettext(s)
}

// ---------------------------------------------------------------------------
// Error strings (copied exactly from regexp_shim.c and errors.h).
// These serve as gettext msgids; any whitespace or character difference would
// silently break translations.
// ---------------------------------------------------------------------------

// From regexp_shim.c static definitions:
pub const E_INVALID_CHARACTER_AFTER_STR_AT: &std::ffi::CStr = c"E59: Invalid character after %s@";
pub const E_INVALID_USE_OF_UNDERSCORE: &std::ffi::CStr = c"E63: Invalid use of \\_";
pub const E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN: &std::ffi::CStr =
    c"E363: Pattern uses more memory than 'maxmempattern'";
pub const E_INVALID_ITEM_IN_STR_BRACKETS: &std::ffi::CStr = c"E369: Invalid item in %s%%[]";
pub const E_MISSING_DELIMITER_AFTER_SEARCH_PATTERN_STR: &std::ffi::CStr =
    c"E654: Missing delimiter after search pattern: %s";
pub const E_MISSINGBRACKET: &std::ffi::CStr = c"E769: Missing ] after %s[";
pub const E_REVERSE_RANGE: &std::ffi::CStr = c"E944: Reverse range in character class";
pub const E_LARGE_CLASS: &std::ffi::CStr = c"E945: Range too large in character class";
pub const E_UNMATCHEDPP: &std::ffi::CStr = c"E53: Unmatched %s%%(";
pub const E_UNMATCHEDP: &std::ffi::CStr = c"E54: Unmatched %s(";
pub const E_UNMATCHEDPAR: &std::ffi::CStr = c"E55: Unmatched %s)";
pub const E_Z_NOT_ALLOWED: &std::ffi::CStr = c"E66: \\z( not allowed here";
pub const E_Z1_NOT_ALLOWED: &std::ffi::CStr = c"E67: \\z1 - \\z9 not allowed here";
pub const E_MISSING_SB: &std::ffi::CStr = c"E69: Missing ] after %s%%[";
pub const E_EMPTY_SB: &std::ffi::CStr = c"E70: Empty %s%%[]";
pub const E_RECURSIVE: &std::ffi::CStr = c"E956: Cannot use pattern recursively";
pub const E_REGEXP_NUMBER_AFTER_DOT_POS_SEARCH_CHR: &std::ffi::CStr =
    c"E1204: No Number allowed after .: '\\%%%c'";
pub const E_NFA_REGEXP_MISSING_VALUE_IN_CHR: &std::ffi::CStr =
    c"E1273: (NFA regexp) missing value in '\\%%%c'";
pub const E_ATOM_ENGINE_MUST_BE_AT_START_OF_PATTERN: &std::ffi::CStr =
    c"E1281: Atom '\\%%#=%c' must be at the start of the pattern";
pub const E_SUBSTITUTE_NESTING_TOO_DEEP: &std::ffi::CStr = c"E1290: substitute nesting too deep";
pub const E_UNICODE_VAL_TOO_LARGE: &std::ffi::CStr =
    c"E1541: Value too large, max Unicode codepoint is U+10FFFF";
pub const E_NUL_FOUND: &std::ffi::CStr = c"E865: (NFA) Regexp end encountered prematurely";
pub const E_MISPLACED: &std::ffi::CStr = c"E866: (NFA regexp) Misplaced %c";
// PRId64 on Linux 64-bit expands to "ld", so the gettext key is:
// "E877: (NFA regexp) Invalid character class: %ld"
pub const E_ILL_CHAR_CLASS_FMT: &std::ffi::CStr =
    c"E877: (NFA regexp) Invalid character class: %ld";
pub const E_VALUE_TOO_LARGE: &std::ffi::CStr = c"E951: \\% value too large";

// From errors.h (duplicated here for gettext key identity):
pub const E_INTERNAL_ERROR_IN_REGEXP: &std::ffi::CStr = c"E473: Internal error in regexp";
pub const E_NOPRESUB: &std::ffi::CStr = c"E33: No previous substitute regular expression";
pub const E_NULL: &std::ffi::CStr = c"E38: Null argument";
pub const E_RE_DAMG: &std::ffi::CStr = c"E43: Damaged match string";
pub const E_RE_CORR: &std::ffi::CStr = c"E44: Corrupted regexp program";
pub const E_TOOMSBRA: &std::ffi::CStr = c"E76: Too many [";
pub const E_TRAILING: &std::ffi::CStr = c"E488: Trailing characters";
pub const E_RESULTING_TEXT_TOO_LONG: &std::ffi::CStr = c"E1240: Resulting text too long";

// ---------------------------------------------------------------------------
// Helper: the `m != 0` -> "" / "\" magic/nomagic prefix for many errors.
// ---------------------------------------------------------------------------
#[inline]
pub const fn magic_prefix(m: c_int) -> *const c_char {
    if m != 0 {
        c"".as_ptr()
    } else {
        c"\\".as_ptr()
    }
}

// ---------------------------------------------------------------------------
// Error wrapper functions — direct Rust replacements for the C wrappers.
// ---------------------------------------------------------------------------

/// E654: missing delimiter after search pattern.
pub unsafe fn emsg_semsg_e654(startp: *const c_char) {
    semsg(
        gt(E_MISSING_DELIMITER_AFTER_SEARCH_PATTERN_STR.as_ptr()),
        startp,
    );
}

/// E59: invalid character after @.
pub unsafe fn emsg2_e59(m: c_int) {
    semsg(
        gt(E_INVALID_CHARACTER_AFTER_STR_AT.as_ptr()),
        magic_prefix(m),
    );
    nvim_regexp_set_rc_did_emsg(1);
}

/// E60: too many complex {…}s.
pub unsafe fn emsg2_e60(m: c_int) {
    semsg(
        gt(c"E60: Too many complex %s{...}s".as_ptr()),
        magic_prefix(m),
    );
    nvim_regexp_set_rc_did_emsg(1);
}

/// E61: nested star.
pub unsafe fn emsg2_e61(m: c_int) {
    semsg(gt(c"E61: Nested %s*".as_ptr()), magic_prefix(m));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E62: nested multi.
pub unsafe fn emsg3_e62(m: c_int, c: c_int) {
    semsg(gt(c"E62: Nested %s%c".as_ptr()), magic_prefix(m), c);
    nvim_regexp_set_rc_did_emsg(1);
}

/// E50: too many \z(.
pub unsafe fn emsg_e50() {
    emsg(gt(c"E50: Too many \\z(".as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E51: too many groups.
pub unsafe fn emsg2_e51(m: c_int) {
    semsg(gt(c"E51: Too many %s(".as_ptr()), magic_prefix(m));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E52: unmatched \z(.
pub unsafe fn emsg_e52() {
    emsg(gt(c"E52: Unmatched \\z(".as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E53: unmatched %%(  .
pub unsafe fn emsg2_e53(m: c_int) {
    semsg(gt(E_UNMATCHEDPP.as_ptr()), magic_prefix(m));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E54: unmatched (.
pub unsafe fn emsg2_e54(m: c_int) {
    semsg(gt(E_UNMATCHEDP.as_ptr()), magic_prefix(m));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E55: unmatched ).
pub unsafe fn emsg2_e55(m: c_int) {
    semsg(gt(E_UNMATCHEDPAR.as_ptr()), magic_prefix(m));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E488: trailing characters.
pub unsafe fn emsg_e488() {
    emsg(gt(E_TRAILING.as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E63: invalid use of \_.
pub unsafe fn emsg_e63_underscore() {
    emsg(gt(E_INVALID_USE_OF_UNDERSCORE.as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E64: follows nothing.
pub unsafe fn emsg3_e64(m: c_int, c: c_int) {
    semsg(
        gt(c"E64: %s%c follows nothing".as_ptr()),
        magic_prefix(m),
        c,
    );
    nvim_regexp_set_rc_did_emsg(1);
}

/// E65: illegal back reference.
pub unsafe fn emsg_e65() {
    emsg(gt(c"E65: Illegal back reference".as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E66: \z( not allowed here.
pub unsafe fn emsg_e66() {
    emsg(gt(E_Z_NOT_ALLOWED.as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E67: \z1-\z9 not allowed here.
pub unsafe fn emsg_e67() {
    emsg(gt(E_Z1_NOT_ALLOWED.as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E68: invalid character after \z.
pub unsafe fn emsg_e68() {
    emsg(gt(c"E68: Invalid character after \\z".as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E69: missing ].
pub unsafe fn emsg2_e69(m: c_int) {
    semsg(gt(E_MISSING_SB.as_ptr()), magic_prefix(m));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E70: empty %[].
pub unsafe fn emsg2_e70(m: c_int) {
    semsg(gt(E_EMPTY_SB.as_ptr()), magic_prefix(m));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E71: invalid character after %%.
pub unsafe fn emsg2_e71(m: c_int) {
    semsg(
        gt(c"E71: Invalid character after %s%%".as_ptr()),
        magic_prefix(m),
    );
    nvim_regexp_set_rc_did_emsg(1);
}

/// E678: invalid character after %[dxouU].
pub unsafe fn emsg2_e678(m: c_int) {
    semsg(
        gt(c"E678: Invalid character after %s%%[dxouU]".as_ptr()),
        magic_prefix(m),
    );
    nvim_regexp_set_rc_did_emsg(1);
}

/// E769: missing ] after [...].
pub unsafe fn emsg2_e769(m: c_int) {
    semsg(gt(E_MISSINGBRACKET.as_ptr()), magic_prefix(m));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E944: reverse range in character class.
pub unsafe fn emsg_e944() {
    emsg(gt(E_REVERSE_RANGE.as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E945: range too large in character class.
pub unsafe fn emsg_e945() {
    emsg(gt(E_LARGE_CLASS.as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E1541: value too large (unicode codepoint).
pub unsafe fn emsg_e949() {
    emsg(gt(E_UNICODE_VAL_TOO_LARGE.as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E76: too many subexpressions.
pub unsafe fn emsg_toomsbra() {
    emsg(gt(E_TOOMSBRA.as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E1281: atom engine position.
pub unsafe fn semsg_e_atom_engine(c: c_int) {
    semsg(gt(E_ATOM_ENGINE_MUST_BE_AT_START_OF_PATTERN.as_ptr()), c);
    nvim_regexp_set_rc_did_emsg(1);
}

/// E1204: number after dot.
pub unsafe fn semsg_e_dot_pos(c: c_int) {
    semsg(gt(E_REGEXP_NUMBER_AFTER_DOT_POS_SEARCH_CHR.as_ptr()), c);
    nvim_regexp_set_rc_did_emsg(1);
}

/// E369: invalid item in %[].
pub unsafe fn emsg2_e369(m: c_int) {
    semsg(gt(E_INVALID_ITEM_IN_STR_BRACKETS.as_ptr()), magic_prefix(m));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E363: maxmempattern exceeded.
pub unsafe fn emsg_maxmempattern() {
    emsg(gt(E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN.as_ptr()));
}

/// E865: NFA regexp end prematurely.
pub unsafe fn emsg_nul_found() {
    emsg(gt(E_NUL_FOUND.as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E866: misplaced character.
#[allow(clippy::cast_possible_truncation)]
pub unsafe fn semsg_misplaced(c: c_int) {
    semsg(gt(E_MISPLACED.as_ptr()), c as c_char as c_int);
}

/// E877: invalid character class.
pub unsafe fn semsg_ill_char_class(c: i64) {
    semsg(gt(E_ILL_CHAR_CLASS_FMT.as_ptr()), c as c_long);
    nvim_regexp_set_rc_did_emsg(1);
}

/// Internal: unknown character class (siemsg, no gettext).
pub unsafe fn siemsg_unknown_class(c: i64) {
    siemsg(
        c"INTERNAL: Unknown character class char: %ld".as_ptr(),
        c as c_long,
    );
}

/// E867: unknown \z operator.
pub unsafe fn semsg_e867_z(c: c_int) {
    semsg(gt(c"E867: (NFA) Unknown operator '\\z%c'".as_ptr()), c);
}

/// E867: unknown %% operator.
pub unsafe fn semsg_e867_pct(c: c_int) {
    semsg(gt(c"E867: (NFA) Unknown operator '\\%%%c'".as_ptr()), c);
}

/// E951: \% value too large.
pub unsafe fn emsg_value_too_large() {
    emsg(gt(E_VALUE_TOO_LARGE.as_ptr()));
}

/// E1273: missing value in \%X.
pub unsafe fn semsg_missing_value(c: c_int) {
    semsg(gt(E_NFA_REGEXP_MISSING_VALUE_IN_CHR.as_ptr()), c);
}

/// Set `rc_did_emsg` to true (used by NFA code after certain errors).
pub unsafe fn set_rc_did_emsg_true() {
    nvim_regexp_set_rc_did_emsg(1);
}

/// E869: unknown \@ operator.
pub unsafe fn semsg_e869(op: c_int) {
    semsg(gt(c"E869: (NFA) Unknown operator '\\@%c'".as_ptr()), op);
}

/// E870: error reading repetition limits.
pub unsafe fn emsg_e870() {
    emsg(gt(
        c"E870: (NFA regexp) Error reading repetition limits".as_ptr()
    ));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E871: multi follow multi.
pub unsafe fn emsg_e871() {
    emsg(gt(
        c"E871: (NFA regexp) Can't have a multi follow a multi".as_ptr()
    ));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E872: too many (.
pub unsafe fn emsg_e872() {
    emsg(gt(c"E872: (NFA regexp) Too many '('".as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E879: too many \z(.
pub unsafe fn emsg_e879() {
    emsg(gt(c"E879: (NFA regexp) Too many \\z(".as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E873: proper termination error.
pub unsafe fn emsg_e873() {
    emsg(gt(c"E873: (NFA regexp) proper termination error".as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E874: could not pop stack.
pub unsafe fn emsg_e874() {
    emsg(gt(c"E874: (NFA) Could not pop the stack!".as_ptr()));
}

/// E875: too many states on stack.
pub unsafe fn emsg_e875() {
    emsg(gt(
        c"E875: (NFA regexp) (While converting from postfix to NFA),too many states left on stack"
            .as_ptr(),
    ));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E876: not enough space for NFA.
pub unsafe fn emsg_e876() {
    emsg(gt(
        c"E876: (NFA regexp) Not enough space to store the whole NFA ".as_ptr(),
    ));
    nvim_regexp_set_rc_did_emsg(1);
}

/// Duplicate of `semsg_ill_char_class` (for `check_char_class`).
pub unsafe fn siemsg_ill_char_class(cls: i64) {
    siemsg(gt(E_ILL_CHAR_CLASS_FMT.as_ptr()), cls as c_long);
}

// ---------------------------------------------------------------------------
// Error wrappers using globals from errors.h (duplicated string literals).
// ---------------------------------------------------------------------------

/// E1240: resulting text too long.
pub unsafe fn emsg_resulting_text_too_long() {
    emsg(gt(E_RESULTING_TEXT_TOO_LONG.as_ptr()));
}

/// E44: corrupted regexp program (iemsg).
pub unsafe fn iemsg_re_corr() {
    iemsg(gt(E_RE_CORR.as_ptr()));
}

/// E43: damaged match string (iemsg).
pub unsafe fn call_iemsg_re_damg() {
    iemsg(gt(E_RE_DAMG.as_ptr()));
}

/// E38: null argument.
pub unsafe fn emsg_e_null() {
    emsg(gt(E_NULL.as_ptr()));
}

/// E1290: substitute nesting too deep.
pub unsafe fn emsg_e_substitute_nesting() {
    emsg(gt(E_SUBSTITUTE_NESTING_TOO_DEEP.as_ptr()));
}

/// E473: internal error in regexp (iemsg).
pub unsafe fn iemsg_internal() {
    iemsg(gt(E_INTERNAL_ERROR_IN_REGEXP.as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// E33: no previous substitute regexp.
pub unsafe fn emsg_nopresub() {
    emsg(gt(E_NOPRESUB.as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// Internal: not enough space in `vim_regsub_both` (iemsg, not gettext).
pub unsafe fn call_iemsg_not_enough_space() {
    iemsg(c"vim_regsub_both(): not enough space".as_ptr());
}

/// E38: null argument (iemsg variant for `nfa_regexec_both`).
pub unsafe fn call_iemsg_null() {
    iemsg(gt(E_NULL.as_ptr()));
}

/// E956: cannot use pattern recursively.
pub unsafe fn call_emsg_recursive() {
    emsg(gt(E_RECURSIVE.as_ptr()));
}

/// E864: \%#= can only be followed by 0, 1, or 2.
pub unsafe fn call_emsg_e864() {
    emsg(gt(
        c"E864: \\%#= can only be followed by 0, 1, or 2. The automatic engine will be used "
            .as_ptr(),
    ));
}

/// E339: pattern too long.
pub unsafe fn call_emsg_e339() {
    emsg(gt(c"E339: Pattern too long".as_ptr()));
    nvim_regexp_set_rc_did_emsg(1);
}

/// Wrapper for `internal_error()`.
pub unsafe fn regexp_internal_error(msg: *const c_char) {
    internal_error(msg);
}

/// E888: (NFA regexp) cannot repeat.
pub unsafe fn semsg_e888(what: *const c_char) {
    semsg(gt(c"E888: (NFA regexp) cannot repeat %s".as_ptr()), what);
}

/// `nvim_regexp_emsg2_fail`: compound error (`semsg` + set `rc_did_emsg` + return FAIL).
/// Returns FAIL (0 in Neovim).
pub unsafe fn emsg2_fail(msg: *const c_char, is_magic_all: c_int) -> c_int {
    semsg(msg, magic_prefix(is_magic_all));
    nvim_regexp_set_rc_did_emsg(1);
    // FAIL = 0 in Neovim
    0
}
