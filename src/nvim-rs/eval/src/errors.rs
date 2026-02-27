//! Error reporting functions for the eval engine.
//!
//! Consolidates all error-message wrappers that were previously C functions in
//! `eval_shim.c`. Each function calls `gettext()` on the English message string
//! (which serves as the gettext msgid) then dispatches to `emsg()`/`semsg()`/etc.
//!
//! The English strings are copied byte-for-byte from the C source so that
//! gettext lookup is identical regardless of where the string literal lives.

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_long};

extern "C" {
    pub fn emsg(s: *const c_char) -> c_int;
    pub fn semsg(fmt: *const c_char, ...) -> c_int;
    // gettext translation
    pub fn gettext(msgid: *const c_char) -> *const c_char;
}

// ---------------------------------------------------------------------------
// Inline helper: translate an English string via gettext at runtime.
// ---------------------------------------------------------------------------
#[inline]
#[must_use]
pub unsafe fn gt(s: *const c_char) -> *const c_char {
    gettext(s)
}

// ---------------------------------------------------------------------------
// Error strings (copied exactly from eval_shim.c statics and errors.h).
// These serve as gettext msgids; any whitespace or character difference would
// silently break translations.
// ---------------------------------------------------------------------------

// From eval_shim.c static definitions:
// eval_shim.c:441  static const char *e_missbrac = N_("E111: Missing ']'");
pub const E_MISSBRAC: &std::ffi::CStr = c"E111: Missing ']'";

// eval_shim.c:442  static const char e_cannot_slice_dictionary[] = N_("E719: Cannot slice a Dictionary");
pub const E_CANNOT_SLICE_DICTIONARY: &std::ffi::CStr = c"E719: Cannot slice a Dictionary";

// eval_shim.c:444  static const char e_cannot_index_special_variable[] = N_("E909: Cannot index a special variable");
pub const E_CANNOT_INDEX_SPECIAL_VARIABLE: &std::ffi::CStr =
    c"E909: Cannot index a special variable";

// eval_shim.c:446  static const char *e_nowhitespace = N_("E274: No white space allowed before parenthesis");
pub const E_NOWHITESPACE: &std::ffi::CStr = c"E274: No white space allowed before parenthesis";

// eval_shim.c:448  static const char e_cannot_index_a_funcref[] = N_("E695: Cannot index a Funcref");
pub const E_CANNOT_INDEX_A_FUNCREF: &std::ffi::CStr = c"E695: Cannot index a Funcref";

// eval_shim.c:450  static const char e_variable_nested_too_deep_for_making_copy[] = N_("E698: Variable nested too deep for making a copy");
pub const E_VARIABLE_NESTED_TOO_DEEP: &std::ffi::CStr =
    c"E698: Variable nested too deep for making a copy";

// eval_shim.c:452  static const char e_dot_can_only_be_used_on_dictionary_str[] = N_("E1203: Dot can only be used on a dictionary: %s");
pub const E_DOT_CAN_ONLY_BE_USED_ON_DICTIONARY_STR: &std::ffi::CStr =
    c"E1203: Dot can only be used on a dictionary: %s";

// eval_shim.c:454  static const char e_empty_function_name[] = N_("E1192: Empty function name");
pub const E_EMPTY_FUNCTION_NAME: &std::ffi::CStr = c"E1192: Empty function name";

// From errors.h EXTERN declarations:
// errors.h:122  e_trailing_arg = N_("E488: Trailing characters: %s")
pub const E_TRAILING_ARG: &std::ffi::CStr = c"E488: Trailing characters: %s";

// errors.h:?  e_invexpr2 = N_("E15: Invalid expression: \"%s\"")
pub const E_INVEXPR2: &std::ffi::CStr = c"E15: Invalid expression: \"%s\"";

// errors.h:94  e_cannot_mod = N_("E995: Cannot modify existing variable")
pub const E_CANNOT_MOD: &std::ffi::CStr = c"E995: Cannot modify existing variable";

// errors.h:92  e_letwrong = N_("E734: Wrong variable type for %s=")
pub const E_LETWRONG: &std::ffi::CStr = c"E734: Wrong variable type for %s=";

// errors.h:101  e_dictkey = N_("E716: Key not present in Dictionary: \"%s\"")
pub const E_DICTKEY: &std::ffi::CStr = c"E716: Key not present in Dictionary: \"%s\"";

// errors.h:102  e_dictkey_len = N_("E716: Key not present in Dictionary: \"%.*s\"")
pub const E_DICTKEY_LEN: &std::ffi::CStr = c"E716: Key not present in Dictionary: \"%.*s\"";

// errors.h:93  e_illvar = N_("E461: Illegal variable name: %s")
pub const E_ILLVAR: &std::ffi::CStr = c"E461: Illegal variable name: %s";

// errors.h:37  e_invexpr2 already above; missingparen:
// errors.h:130  e_missingparen = N_("E107: Missing parentheses: %s")
pub const E_MISSINGPAREN: &std::ffi::CStr = c"E107: Missing parentheses: %s";

// errors.h:150  e_using_float_as_string = N_("E806: Using a Float as a String")
pub const E_USING_FLOAT_AS_STRING: &std::ffi::CStr = c"E806: Using a Float as a String";

// errors.h:43  e_invchan = N_("E900: Invalid channel id")
pub const E_INVCHAN: &std::ffi::CStr = c"E900: Invalid channel id";

// errors.h:44  e_invchanjob = N_("E900: Invalid channel id: not a job")
pub const E_INVCHANJOB: &std::ffi::CStr = c"E900: Invalid channel id: not a job";

// errors.h:132  e_nobufnr = N_("E86: Buffer %" PRId64 " does not exist")
// PRId64 on Linux 64-bit expands to "ld", so the gettext key is:
pub const E_NOBUFNR: &std::ffi::CStr = c"E86: Buffer %ld does not exist";

// errors.h:32  e_invarg = N_("E474: Invalid argument")
pub const E_INVARG: &std::ffi::CStr = c"E474: Invalid argument";

// errors.h:33  e_invarg2 = N_("E475: Invalid argument: %s")
pub const E_INVARG2: &std::ffi::CStr = c"E475: Invalid argument: %s";

// errors.h:35  e_invargNval = N_("E475: Invalid value for argument %s: %s")
pub const E_INVARG_NVAL: &std::ffi::CStr = c"E475: Invalid value for argument %s: %s";

// errors.h:158  e_fast_api_disabled = N_("E5560: %s must not be called in a fast event context")
pub const E_FAST_API_DISABLED: &std::ffi::CStr =
    c"E5560: %s must not be called in a fast event context";

// Inline error strings only used in eval_shim.c (never in errors.h):
// eval_shim.c:1012  _("E996: Cannot lock a range")
pub const E_CANNOT_LOCK_RANGE: &std::ffi::CStr = c"E996: Cannot lock a range";

// eval_shim.c:1017  _("E996: Cannot lock a list or dict")
pub const E_CANNOT_LOCK_LIST_OR_DICT: &std::ffi::CStr = c"E996: Cannot lock a list or dict";

// eval_shim.c:1222  _("E689: Can only index a List, Dictionary or Blob")
pub const E_E689: &std::ffi::CStr = c"E689: Can only index a List, Dictionary or Blob";

// eval_shim.c:1227  _("E708: [:] must come last")
pub const E_E708: &std::ffi::CStr = c"E708: [:] must come last";

// eval_shim.c:1233  _("E713: Cannot use empty key after .")
pub const E_E713: &std::ffi::CStr = c"E713: Cannot use empty key after .";

// eval_shim.c:1239  _("E709: [:] requires a List or Blob value")
pub const E_E709: &std::ffi::CStr = c"E709: [:] requires a List or Blob value";

// eval_shim.c:2408  e_fast_api_disabled with "Vimscript function" arg (non-translatable fmt)
// Provider format strings (non-translatable):
pub const PROVIDER_MISSING_VAR_FMT: &std::ffi::CStr =
    c"provider: %s: missing required variable g:loaded_%s_provider";
pub const PROVIDER_NO_CALL_FMT: &std::ffi::CStr =
    c"provider: %s: g:loaded_%s_provider=2 but %s is not defined";
pub const PROVIDER_NOT_FOUND_FMT: &std::ffi::CStr =
    c"E319: No \"%s\" provider found. Run \":checkhealth vim.provider\"";

// errors.h:112  e_semsg_e112 = "E112: Option name missing: %s"
pub const E_E112_OPTION_NAME_MISSING: &std::ffi::CStr = c"E112: Option name missing: %s";

// errors.h:113  e_semsg_e113 = "E113: Unknown option: %s"
pub const E_E113_UNKNOWN_OPTION: &std::ffi::CStr = c"E113: Unknown option: %s";

// ---------------------------------------------------------------------------
// Error wrapper functions -- direct Rust replacements for the C wrappers.
// ---------------------------------------------------------------------------

/// E488: Trailing characters: %s
pub unsafe fn semsg_trailing_arg(p: *const c_char) {
    semsg(gt(E_TRAILING_ARG.as_ptr()), p);
}

/// E121: Undefined variable: %.*s
pub unsafe fn semsg_undef_var(len: c_int, name: *const c_char) {
    semsg(gt(c"E121: Undefined variable: %.*s".as_ptr()), len, name);
}

/// E995: Cannot modify existing variable
pub unsafe fn emsg_cannot_mod() {
    emsg(gt(E_CANNOT_MOD.as_ptr()));
}

/// E734: Wrong variable type for %s=
pub unsafe fn semsg_letwrong(op: *const c_char) {
    semsg(gt(E_LETWRONG.as_ptr()), op);
}

/// E996: Cannot lock a range
pub unsafe fn emsg_cannot_lock_range() {
    emsg(gt(E_CANNOT_LOCK_RANGE.as_ptr()));
}

/// E996: Cannot lock a list or dict
pub unsafe fn emsg_cannot_lock_list_or_dict() {
    emsg(gt(E_CANNOT_LOCK_LIST_OR_DICT.as_ptr()));
}

/// E716: Key not present in Dictionary: "%s"
pub unsafe fn semsg_dictkey(key: *const c_char) {
    semsg(gt(E_DICTKEY.as_ptr()), key);
}

/// E111: Missing ']'
pub unsafe fn emsg_missbrac() {
    emsg(gt(E_MISSBRAC.as_ptr()));
}

/// E695: Cannot index a Funcref
pub unsafe fn emsg_cannot_index_funcref() {
    emsg(gt(E_CANNOT_INDEX_A_FUNCREF.as_ptr()));
}

/// E806: Using a Float as a String
pub unsafe fn emsg_using_float_as_string() {
    emsg(gt(E_USING_FLOAT_AS_STRING.as_ptr()));
}

/// E909: Cannot index a special variable
pub unsafe fn emsg_cannot_index_special() {
    emsg(gt(E_CANNOT_INDEX_SPECIAL_VARIABLE.as_ptr()));
}

/// E719: Cannot slice a Dictionary
pub unsafe fn emsg_cannot_slice_dict() {
    emsg(gt(E_CANNOT_SLICE_DICTIONARY.as_ptr()));
}

/// E716: Key not present in Dictionary: "%.*s" (with length)
#[allow(clippy::cast_possible_truncation)]
pub unsafe fn semsg_dictkey_len(keylen: isize, key: *const c_char) {
    semsg(gt(E_DICTKEY_LEN.as_ptr()), keylen as c_int, key);
}

/// E689: Can only index a List, Dictionary or Blob
pub unsafe fn emsg_e689() {
    emsg(gt(E_E689.as_ptr()));
}

/// E708: [:] must come last
pub unsafe fn emsg_e708() {
    emsg(gt(E_E708.as_ptr()));
}

/// E713: Cannot use empty key after .
pub unsafe fn emsg_e713() {
    emsg(gt(E_E713.as_ptr()));
}

/// E709: [:] requires a List or Blob value
pub unsafe fn emsg_e709() {
    emsg(gt(E_E709.as_ptr()));
}

/// E1203: Dot can only be used on a dictionary: %s
pub unsafe fn semsg_e_dot_dict(name: *const c_char) {
    semsg(gt(E_DOT_CAN_ONLY_BE_USED_ON_DICTIONARY_STR.as_ptr()), name);
}

/// E461: Illegal variable name: %s (raw, no translation -- used for v:lua case)
pub unsafe fn semsg_e_illvar_raw(name: *const c_char) {
    semsg(E_ILLVAR.as_ptr(), name);
}

/// E461: Illegal variable name: %s (with translation)
pub unsafe fn semsg_e_illvar(name: *const c_char) {
    semsg(gt(E_ILLVAR.as_ptr()), name);
}

/// E15: Invalid expression: "%s"
pub unsafe fn semsg_invexpr2(p: *const c_char) {
    semsg(gt(E_INVEXPR2.as_ptr()), p);
}

/// E274: No white space allowed before parenthesis
pub unsafe fn emsg_e_nowhitespace() {
    emsg(gt(E_NOWHITESPACE.as_ptr()));
}

/// E107: Missing parentheses: %s
pub unsafe fn semsg_e_missingparen(name: *const c_char) {
    semsg(gt(E_MISSINGPAREN.as_ptr()), name);
}

/// E1192: Empty function name
pub unsafe fn emsg_e_empty_function_name() {
    emsg(gt(E_EMPTY_FUNCTION_NAME.as_ptr()));
}

/// E112: Option name missing: %s
pub unsafe fn semsg_e112_option_name_missing(arg: *const c_char) {
    semsg(gt(E_E112_OPTION_NAME_MISSING.as_ptr()), arg);
}

/// E113: Unknown option: %s
pub unsafe fn semsg_e113_unknown_option(arg: *const c_char) {
    semsg(gt(E_E113_UNKNOWN_OPTION.as_ptr()), arg);
}

/// E698: Variable nested too deep for making a copy
pub unsafe fn emsg_nested_too_deep() {
    emsg(gt(E_VARIABLE_NESTED_TOO_DEEP.as_ptr()));
}

/// E86: Buffer %ld does not exist
pub unsafe fn semsg_e_nobufnr(nr: i64) {
    semsg(gt(E_NOBUFNR.as_ptr()), nr as c_long);
}

/// E474: Invalid argument (list must have at least one item)
pub unsafe fn emsg_tv_to_argv_empty() {
    emsg(gt(E_INVARG.as_ptr()));
}

/// E475: Invalid argument: %s (expected String or List)
pub unsafe fn semsg_tv_to_argv_type() {
    semsg(gt(E_INVARG2.as_ptr()), c"expected String or List".as_ptr());
}

/// E475: Invalid value for argument %s: %s (not executable)
pub unsafe fn semsg_tv_to_argv_notexe(msg: *const c_char) {
    semsg(gt(E_INVARG_NVAL.as_ptr()), c"cmd".as_ptr(), msg);
}

/// E5560: %s must not be called in a fast event context (Vimscript function)
pub unsafe fn semsg_fast_api_disabled() {
    semsg(
        gt(E_FAST_API_DISABLED.as_ptr()),
        c"Vimscript function".as_ptr(),
    );
}

/// provider: %s: missing required variable g:loaded_%s_provider
pub unsafe fn semsg_provider_missing_var(name: *const c_char) {
    semsg(PROVIDER_MISSING_VAR_FMT.as_ptr(), name, name);
}

/// provider: %s: g:loaded_%s_provider=2 but %s is not defined
pub unsafe fn semsg_provider_no_call(name: *const c_char, funcname: *const c_char) {
    semsg(PROVIDER_NO_CALL_FMT.as_ptr(), name, name, funcname);
}

/// E319: No "%s" provider found. Run ":checkhealth vim.provider"
pub unsafe fn semsg_no_provider(provider: *const c_char) {
    semsg(PROVIDER_NOT_FOUND_FMT.as_ptr(), provider);
}

/// E900: Invalid channel id
pub unsafe fn emsg_invchan() {
    emsg(gt(E_INVCHAN.as_ptr()));
}

/// E900: Invalid channel id: not a job
pub unsafe fn emsg_invchanjob() {
    emsg(gt(E_INVCHANJOB.as_ptr()));
}
