//! Migrated orchestrator functions from eval/vars.c.
//!
//! Phase 1: Pure orchestrator functions with zero struct access.
//!
//! - `skip_var_one` (private helper): skip one assignable variable name
//! - `rs_skip_var_list` (FFI export): skip variable list `[var, var]` syntax
//! - `eval_all_expr_in_str` (private helper): evaluate all `{expr}` blocks in a string
//! - `rs_eval_one_expr_in_str` (FFI export): evaluate one `{expr}` in a string
//! - `rs_get_spellword` (FFI export): get spell word and score from suggestion list

#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::ref_as_ptr)]
#![allow(clippy::too_long_first_doc_paragraph)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// GArray type (matches garray_T layout exactly)
// =============================================================================

/// Mirror of `garray_T` from C (must match layout exactly).
#[repr(C)]
pub struct GArray {
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
    pub ga_data: *mut c_void,
}

// =============================================================================
// ListHandle (opaque pointer to list_T)
// =============================================================================

/// Opaque handle to a `list_T`.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ListHandle(*const c_void);

impl ListHandle {
    /// Check if the handle is null.
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    // rs_find_name_end: already a Rust export, declared here for internal use.
    fn rs_find_name_end(
        arg: *const c_char,
        expr_start: *mut *const c_char,
        expr_end: *mut *const c_char,
        flags: c_int,
    ) -> *const c_char;

    // skipwhite: skip whitespace, returns pointer past whitespace
    fn skipwhite(p: *const c_char) -> *mut c_char;

    // skip_expr: skip over an expression, advances *pp
    fn skip_expr(pp: *mut *mut c_char, evalarg: *mut c_void) -> c_int;

    // eval_to_string: evaluate expression to string (allocated, must be xfree'd)
    fn eval_to_string(arg: *mut c_char, join_list: bool, use_simple_function: bool) -> *mut c_char;

    // ga_concat: append a string to a growing array
    fn ga_concat(gap: *mut GArray, s: *const c_char);

    // ga_concat_len: append len bytes to a growing array
    fn ga_concat_len(gap: *mut GArray, s: *const c_char, len: usize);

    // ga_append: append a single byte to a growing array
    fn ga_append(gap: *mut GArray, c: c_int);

    // ga_init: initialize a growing array
    fn ga_init(gap: *mut GArray, itemsize: c_int, growsize: c_int);

    // ga_clear: free the data of a growing array
    fn ga_clear(gap: *mut GArray);

    // xfree: free allocated memory
    fn xfree(ptr: *mut c_void);

    // emsg: print error message
    fn emsg(s: *const c_char) -> c_int;

    // semsg: print formatted error message
    fn semsg(fmt: *const c_char, ...) -> c_int;

    // rs_tv_list_len: get length of a VimL list (Rust export from typval crate)
    fn rs_tv_list_len(l: ListHandle) -> c_int;

    // tv_list_find_str: get list item as string (Rust export via export_name)
    fn tv_list_find_str(l: ListHandle, n: c_int) -> *const c_char;

    // tv_list_find_nr: get list item as number (Rust export via export_name)
    fn tv_list_find_nr(l: ListHandle, n: c_int, ret_error: *mut bool) -> i64;

    // Error string globals
    static e_stray_closing_curly_str: c_char;
    static e_missing_close_curly_str: c_char;

    // nvim_get_e_invarg2: returns e_invarg2 pointer (from ex_docmd.c)
    fn nvim_get_e_invarg2() -> *const c_char;

    // nvim_vars_emsg_e5700: emit E5700 error for spellsuggest
    fn nvim_vars_emsg_e5700();
}

// FNE flags (match C defines in eval.h)
const FNE_INCL_BR: c_int = 1;
const FNE_CHECK_START: c_int = 2;

// C return value for FAIL
const FAIL: c_int = 0;

// =============================================================================
// Private helper: skip_var_one
// =============================================================================

/// Skip one (assignable) variable name, including @r, $VAR, &option, d.key, l[idx].
///
/// Mirrors C `skip_var_one` (static in vars.c).
///
/// # Safety
/// `arg` must be a valid null-terminated C string.
unsafe fn skip_var_one_impl(arg: *const c_char) -> *const c_char {
    let c = *arg as u8;
    if c == b'@' {
        let next = *arg.add(1) as u8;
        if next != 0 {
            return arg.add(2);
        }
    }
    let start = if c == b'$' || c == b'&' {
        arg.add(1)
    } else {
        arg
    };
    rs_find_name_end(
        start,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        FNE_INCL_BR | FNE_CHECK_START,
    )
}

// =============================================================================
// rs_skip_var_list
// =============================================================================

/// Skip over assignable variable "var" or list of variables "[var, var]".
///
/// Used for ":let varvar = expr" and ":for varvar in expr".
/// For "[var, var]" increments `*var_count` for each variable.
/// For "[var, var; var]" sets `*semicolon` to 1.
/// If `silent` is true, does not give an "invalid argument" error message.
///
/// Returns NULL for an error. Equivalent to C `skip_var_list`.
///
/// # Safety
/// - `arg` must be a valid null-terminated C string.
/// - `var_count` and `semicolon` must be valid mutable integer pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_var_list(
    arg: *const c_char,
    var_count: *mut c_int,
    semicolon: *mut c_int,
    silent: bool,
) -> *const c_char {
    if *arg as u8 == b'[' {
        // "[var, var]": find the matching ']'.
        let mut p = arg;
        loop {
            p = skipwhite(p.add(1)).cast::<c_char>();
            let s = skip_var_one_impl(p);
            if s == p {
                if !silent {
                    semsg(nvim_get_e_invarg2(), p);
                }
                return std::ptr::null();
            }
            *var_count += 1;

            p = skipwhite(s).cast::<c_char>();
            let pc = *p as u8;
            if pc == b']' {
                break;
            } else if pc == b';' {
                if *semicolon == 1 {
                    if !silent {
                        emsg(E_DOUBLE_SEMICOLON.as_ptr().cast::<c_char>());
                    }
                    return std::ptr::null();
                }
                *semicolon = 1;
            } else if pc != b',' {
                if !silent {
                    semsg(nvim_get_e_invarg2(), p);
                }
                return std::ptr::null();
            }
        }
        return p.add(1);
    }
    skip_var_one_impl(arg)
}

// Error string for E452 (double semicolon in list of variables).
// Matches C: `static const char e_double_semicolon_in_list_of_variables[]
//              = N_("E452: Double ; in list of variables");`
static E_DOUBLE_SEMICOLON: &[u8] = b"E452: Double ; in list of variables\0";

// =============================================================================
// Private helper: eval_all_expr_in_str
// =============================================================================

/// Evaluate all the Vim expressions `{expr}` in "str" and return the resulting
/// string in allocated memory.
///
/// `{{` is reduced to `{` and `}}` to `}`. Used for a heredoc assignment.
/// Returns NULL for an error. Mirrors C `eval_all_expr_in_str` (static in vars.c).
///
/// # Safety
/// `str` must be a valid null-terminated C string.
unsafe fn eval_all_expr_in_str_impl(str: *mut c_char) -> *mut c_char {
    let mut ga = GArray {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: std::ptr::null_mut(),
    };
    ga_init(&mut ga, 1, 80);

    let mut p = str;

    loop {
        if *p as u8 == 0 {
            break;
        }
        let mut escaped_brace = false;

        // Look for a block start.
        let lit_start = p;
        while *p as u8 != b'{' && *p as u8 != b'}' && *p as u8 != 0 {
            p = p.add(1);
        }

        // Check for escaped brace ({{ or }})
        if *p as u8 != 0 && *p as u8 == *p.add(1) as u8 {
            // Escaped brace, unescape and continue.
            p = p.add(1);
            escaped_brace = true;
        } else if *p as u8 == b'}' {
            semsg(&e_stray_closing_curly_str as *const c_char, str);
            ga_clear(&mut ga);
            return std::ptr::null_mut();
        }

        // Append the literal part.
        let lit_len = p.offset_from(lit_start) as usize;
        ga_concat_len(&mut ga, lit_start, lit_len);

        if *p as u8 == 0 {
            break;
        }

        if escaped_brace {
            // Skip the second brace.
            p = p.add(1);
            continue;
        }

        // Evaluate the expression and append the result.
        p = rs_eval_one_expr_in_str_impl(p, &mut ga, true);
        if p.is_null() {
            ga_clear(&mut ga);
            return std::ptr::null_mut();
        }
    }
    ga_append(&mut ga, 0); // NUL terminator

    ga.ga_data.cast::<c_char>()
}

// =============================================================================
// rs_eval_one_expr_in_str
// =============================================================================

/// Internal implementation shared with the C export.
unsafe fn rs_eval_one_expr_in_str_impl(
    p: *mut c_char,
    gap: *mut GArray,
    evaluate: bool,
) -> *mut c_char {
    // skip the opening '{'
    let block_start = skipwhite(p.add(1));

    if *block_start as u8 == 0 {
        semsg(&e_missing_close_curly_str as *const c_char, p);
        return std::ptr::null_mut();
    }

    let mut block_end = block_start;
    if skip_expr(&mut block_end, std::ptr::null_mut()) == FAIL {
        return std::ptr::null_mut();
    }
    block_end = skipwhite(block_end);

    if *block_end as u8 != b'}' {
        semsg(&e_missing_close_curly_str as *const c_char, p);
        return std::ptr::null_mut();
    }

    if evaluate {
        let saved = *block_end;
        *block_end = 0; // NUL-terminate the expression
        let expr_val = eval_to_string(block_start, false, false);
        *block_end = saved; // restore '}'
        if expr_val.is_null() {
            return std::ptr::null_mut();
        }
        ga_concat(gap, expr_val);
        xfree(expr_val.cast::<c_void>());
    }

    block_end.add(1)
}

/// Evaluate one Vim expression `{expr}` in string "p" and append the result to "gap".
///
/// "p" points to the opening `{`. When "evaluate" is false, only skip over the
/// expression. Return a pointer to the character after `}`, NULL for an error.
///
/// Equivalent to C `eval_one_expr_in_str`.
///
/// # Safety
/// - `p` must be a valid pointer into a null-terminated C string (at `{`).
/// - `gap` must be a valid pointer to an initialized garray_T.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_one_expr_in_str(
    p: *mut c_char,
    gap: *mut GArray,
    evaluate: bool,
) -> *mut c_char {
    rs_eval_one_expr_in_str_impl(p, gap, evaluate)
}

/// Evaluate all the Vim expressions `{expr}` in "str", returning allocated string.
///
/// Equivalent to C `eval_all_expr_in_str` (was static, now exported for
/// `heredoc_get` in C until Phase 3 migrates `heredoc_get`).
///
/// # Safety
/// - `str` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_all_expr_in_str(str: *mut c_char) -> *mut c_char {
    eval_all_expr_in_str_impl(str)
}

// =============================================================================
// rs_get_spellword
// =============================================================================

/// Get spell word and score from a spellsuggest entry.
///
/// Entry must be a list with two items: a word and a score. Returns -1 in
/// case of error, score otherwise. Equivalent to C `get_spellword`.
///
/// # Safety
/// - `list` must be a valid list_T pointer (or null for error).
/// - `ret_word` must be a valid pointer to a `*const c_char`.
#[no_mangle]
pub unsafe extern "C" fn rs_get_spellword(list: ListHandle, ret_word: *mut *const c_char) -> c_int {
    if rs_tv_list_len(list) != 2 {
        nvim_vars_emsg_e5700();
        return -1;
    }
    let word = tv_list_find_str(list, 0);
    if word.is_null() {
        return -1;
    }
    *ret_word = word;
    tv_list_find_nr(list, -1, std::ptr::null_mut()) as c_int
}
