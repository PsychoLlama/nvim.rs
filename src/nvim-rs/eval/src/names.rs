//! Name parsing utilities migrated from eval.c.
//!
//! - `get_env_len`: Parse environment variable name length
//! - `get_id_len`: Parse function/variable name length with namespace
//! - `to_name_end`: Find end of name without magic braces
//! - `find_name_end`: Find end of name with magic brace tracking

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr,
    clippy::borrow_as_ptr
)]

use std::ffi::{c_char, c_int};

/// namespace_char = "abglstvw" (verified by _Static_assert in eval.c)
const NAMESPACE_CHARS: &[u8] = b"abglstvw";

/// FNE_INCL_BR flag (verified by _Static_assert in eval.c)
const FNE_INCL_BR: c_int = 1;
/// FNE_CHECK_START flag (verified by _Static_assert in eval.c)
const FNE_CHECK_START: c_int = 2;

extern "C" {
    #[link_name = "vim_isIDc"]
    fn rs_vim_isIDc(c: c_int) -> bool;
    fn rs_eval_isnamec(c: c_int) -> bool;
    fn rs_eval_isnamec1(c: c_int) -> bool;
    fn rs_eval_isdictc(c: c_int) -> bool;
    fn rs_vim_strchr(string: *const c_char, c: c_int) -> *const c_char;
    #[link_name = "skipwhite"]
    fn rs_skipwhite(p: *const c_char) -> *const c_char;
    fn rs_utfc_ptr2len(p: *const c_char) -> c_int;
}

/// Advance pointer by one multi-byte character (at least 1 byte).
///
/// # Safety
///
/// `p` must point to a valid null-terminated C string.
#[inline]
unsafe fn mb_ptr_adv(p: *const c_char) -> *const c_char {
    let len = rs_utfc_ptr2len(p);
    p.add(if len > 0 { len as usize } else { 1 })
}

/// Get the length of the name of an environment variable.
///
/// Advances `*arg` past the name. Returns 0 if no valid name found.
///
/// # Safety
///
/// `arg` must be a valid pointer to a non-null pointer to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_env_len(arg: *mut *const c_char) -> c_int {
    let mut p = *arg;
    while rs_vim_isIDc(c_int::from(*p as u8)) {
        p = p.add(1);
    }
    if p == *arg {
        return 0;
    }
    let len = p.offset_from(*arg) as c_int;
    *arg = p;
    len
}

/// Get the length of the name of a function or internal variable.
///
/// Advances `*arg` to the first non-white character after the name.
/// Returns 0 if no valid name found.
///
/// # Safety
///
/// `arg` must be a valid pointer to a non-null pointer to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_id_len(arg: *mut *const c_char) -> c_int {
    let start = *arg;
    let mut p = start;

    // Find the end of the name
    while rs_eval_isnamec(c_int::from(*p as u8)) {
        if *p as u8 == b':' {
            // "s:" is start of "s:var", but "n:" is not and can be used in
            // slice "[n:]". Also "xx:" is not a namespace.
            let len = p.offset_from(start) as c_int;
            if len > 1
                || (len == 1
                    && rs_vim_strchr(
                        NAMESPACE_CHARS.as_ptr().cast::<c_char>(),
                        c_int::from(*start as u8),
                    )
                    .is_null())
            {
                break;
            }
        }
        p = p.add(1);
    }
    if p == start {
        return 0;
    }

    let len = p.offset_from(start) as c_int;
    *arg = rs_skipwhite(p);
    len
}

/// Find the end of a variable or function name.  Unlike `find_name_end` this
/// does not recognize magic braces.
///
/// When `use_namespace` is true, recognize "b:", "s:", etc.
///
/// Returns a pointer to just after the name. Equal to `arg` if there is no
/// valid name.
///
/// # Safety
///
/// `arg` must be a valid pointer to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_to_name_end(arg: *const c_char, use_namespace: bool) -> *const c_char {
    // Quick check for valid starting character
    if !rs_eval_isnamec1(c_int::from(*arg as u8)) {
        return arg;
    }

    let mut p = arg.add(1);
    while *p != 0 && rs_eval_isnamec(c_int::from(*p as u8)) {
        // Include a namespace such as "s:var" and "v:var".  But "n:" is not
        // and can be used in slice "[n:]".
        if *p as u8 == b':'
            && (p != arg.add(1)
                || !use_namespace
                || rs_vim_strchr(c"bgstvw".as_ptr(), c_int::from(*arg as u8)).is_null())
        {
            break;
        }
        p = mb_ptr_adv(p);
    }
    p
}

/// Find the end of a variable or function name, including magic braces.
///
/// When `expr_start` and `expr_end` are not NULL, also return pointers to
/// the start and end of magic brace expressions.
///
/// `flags` can have `FNE_INCL_BR` and `FNE_CHECK_START`.
///
/// Returns a pointer to just after the name. Equal to `arg` if there is no
/// valid name.
///
/// # Safety
///
/// `arg` must be a valid pointer to a null-terminated C string.
/// `expr_start` and `expr_end` may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_find_name_end(
    arg: *const c_char,
    expr_start: *mut *const c_char,
    expr_end: *mut *const c_char,
    flags: c_int,
) -> *const c_char {
    if !expr_start.is_null() {
        *expr_start = std::ptr::null();
        *expr_end = std::ptr::null();
    }

    // Quick check for valid starting character
    if (flags & FNE_CHECK_START) != 0
        && !rs_eval_isnamec1(c_int::from(*arg as u8))
        && *arg as u8 != b'{'
    {
        return arg;
    }

    let mut mb_nest: c_int = 0;
    let mut br_nest: c_int = 0;

    let mut p = arg;
    while *p != 0
        && (rs_eval_isnamec(c_int::from(*p as u8))
            || *p as u8 == b'{'
            || ((flags & FNE_INCL_BR) != 0
                && (*p as u8 == b'['
                    || (*p as u8 == b'.' && rs_eval_isdictc(c_int::from(*p.add(1) as u8)))))
            || mb_nest != 0
            || br_nest != 0)
    {
        if *p as u8 == b'\'' {
            // skip over 'string' to avoid counting [ and ] inside it
            p = p.add(1);
            while *p != 0 && *p as u8 != b'\'' {
                p = mb_ptr_adv(p);
            }
            if *p == 0 {
                break;
            }
        } else if *p as u8 == b'"' {
            // skip over "str\"ing" to avoid counting [ and ] inside it
            p = p.add(1);
            while *p != 0 && *p as u8 != b'"' {
                if *p as u8 == b'\\' && *p.add(1) != 0 {
                    p = p.add(1);
                }
                p = mb_ptr_adv(p);
            }
            if *p == 0 {
                break;
            }
        } else if br_nest == 0 && mb_nest == 0 && *p as u8 == b':' {
            // "s:" is start of "s:var", but "n:" is not and can be used in
            // slice "[n:]".  Also "xx:" is not a namespace. But {ns}: is.
            let len = p.offset_from(arg) as c_int;
            if (len > 1 && *p.sub(1) as u8 != b'}')
                || (len == 1
                    && rs_vim_strchr(
                        NAMESPACE_CHARS.as_ptr().cast::<c_char>(),
                        c_int::from(*arg as u8),
                    )
                    .is_null())
            {
                break;
            }
        }

        if mb_nest == 0 {
            if *p as u8 == b'[' {
                br_nest += 1;
            } else if *p as u8 == b']' {
                br_nest -= 1;
            }
        }

        if br_nest == 0 {
            if *p as u8 == b'{' {
                mb_nest += 1;
                if !expr_start.is_null() && (*expr_start).is_null() {
                    *expr_start = p;
                }
            } else if *p as u8 == b'}' {
                mb_nest -= 1;
                if !expr_start.is_null() && mb_nest == 0 && (*expr_end).is_null() {
                    *expr_end = p;
                }
            }
        }

        p = mb_ptr_adv(p);
    }

    p
}

// =============================================================================
// Phase 4 (eval_shim pass 5): get_name_len and make_expanded_name
// =============================================================================

// Key constants from keycodes.h
const K_SPECIAL: u8 = 0x80;
const KS_EXTRA: u8 = 253;
const KE_SNR: u8 = 82;

#[allow(clashing_extern_declarations)]
extern "C" {
    // Phase 4: get_name_len helpers
    fn nvim_eval_fname_script(p: *const c_char) -> c_int;
    // skipwhite already declared above as returning *const c_char (link_name alias)
    fn nvim_semsg_invexpr2(p: *const c_char);

    // Phase 4: make_expanded_name helpers (eval_to_string: renamed Rust export)
    fn eval_to_string(arg: *mut c_char, join_list: bool, use_simple_function: bool) -> *mut c_char;
    fn nvim_snprintf_three(
        buf: *mut c_char,
        bufsize: usize,
        a: *const c_char,
        b: *const c_char,
        c: *const c_char,
    );
    fn xmalloc(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_char);
}

/// Compute strlen of a C string.
///
/// # Safety
/// `s` must be a valid null-terminated C string.
#[inline]
unsafe fn c_strlen(s: *const c_char) -> usize {
    let mut p = s;
    while *p != 0 {
        p = p.add(1);
    }
    p.offset_from(s) as usize
}

/// Expand `{expr}` constructs in a variable/function name.
///
/// Equivalent to C static `make_expanded_name`. Mutates the input string
/// temporarily (writes NUL bytes) to extract sub-expressions.
///
/// Returns newly allocated string (caller must free) or NULL on failure.
///
/// # Safety
/// All pointer arguments must be valid. `expr_start` and `expr_end` must
/// be non-null (caller has already checked). `in_end` must be non-null.
unsafe fn make_expanded_name_impl(
    in_start: *const c_char,
    expr_start: *mut c_char,
    expr_end: *mut c_char,
    in_end: *mut c_char,
) -> *mut c_char {
    if expr_end.is_null() || in_end.is_null() {
        return std::ptr::null_mut();
    }

    // Temporarily NUL-terminate at brace positions
    *expr_start = 0; // NUL
    *expr_end = 0; // NUL
    let c1 = *in_end;
    *in_end = 0; // NUL

    // Evaluate the expression between the braces
    let temp_result = eval_to_string(expr_start.add(1), false, false);

    let retval: *mut c_char = if temp_result.is_null() {
        std::ptr::null_mut()
    } else {
        let prefix_len = expr_start.offset_from(in_start) as usize;
        let result_len = c_strlen(temp_result);
        let suffix_len = in_end.offset_from(expr_end) as usize;
        let retvalsize = prefix_len + result_len + suffix_len + 1;
        let buf = xmalloc(retvalsize);
        nvim_snprintf_three(buf, retvalsize, in_start, temp_result, expr_end.add(1));
        buf
    };

    xfree(temp_result);

    // Restore original bytes
    *in_end = c1;
    *expr_start = b'{' as c_char;
    *expr_end = b'}' as c_char;

    if !retval.is_null() {
        // Check for further {expr} in the expanded result
        let mut inner_start: *const c_char = std::ptr::null();
        let mut inner_end: *const c_char = std::ptr::null();
        let next_p = rs_find_name_end(retval, &mut inner_start, &mut inner_end, 0);
        if !inner_start.is_null() {
            // Recursive expansion
            let further = make_expanded_name_impl(
                retval,
                inner_start.cast_mut(),
                inner_end.cast_mut(),
                next_p.cast_mut(),
            );
            xfree(retval);
            return further;
        }
    }

    retval
}

/// Exported wrapper for `make_expanded_name_impl`.
///
/// Called from C `make_expanded_name` and from Rust `lval.rs`.
/// Matches the signature of C `nvim_make_expanded_name`.
///
/// Returns newly allocated string (caller must free) or NULL on failure.
///
/// # Safety
/// All pointer arguments must be valid C pointers (see `make_expanded_name_impl`).
#[no_mangle]
pub unsafe extern "C" fn rs_make_expanded_name(
    in_start: *const c_char,
    expr_start: *mut c_char,
    expr_end: *mut c_char,
    in_end: *mut c_char,
) -> *mut c_char {
    make_expanded_name_impl(in_start, expr_start, expr_end, in_end)
}

/// Get the length of a variable or function name.
///
/// Handles `<SNR>`, `<SID>`, `s:` prefixes and `{expr}` brace expansion.
/// Sets `*alias` to an allocated expanded string if `{expr}` is used.
///
/// Returns -1 on error, 0 if no name found, else the length.
///
/// Equivalent to C `get_name_len`.
///
/// # Safety
/// - `arg` must be a valid pointer to a null-terminated C string pointer.
/// - `alias` must be a valid writable pointer (set to NULL on entry by this fn).
#[export_name = "get_name_len"]
pub unsafe extern "C" fn rs_get_name_len(
    arg: *mut *const c_char,
    alias: *mut *mut c_char,
    evaluate: bool,
    verbose: bool,
) -> c_int {
    *alias = std::ptr::null_mut();

    let start = *arg;

    // Check for hard-coded <SNR> (already translated in the keycode stream)
    if (*start as u8) == K_SPECIAL
        && (*start.add(1) as u8) == KS_EXTRA
        && (*start.add(2) as u8) == KE_SNR
    {
        *arg = start.add(3);
        return rs_get_id_len(arg) + 3;
    }

    // Check for literal "<SID>", "s:", or "<SNR>"
    let mut len = nvim_eval_fname_script(start);
    if len > 0 {
        *arg = start.add(len as usize);
    }

    // Find the end of the name; check for {} construction.
    let mut expr_start: *const c_char = std::ptr::null();
    let mut expr_end: *const c_char = std::ptr::null();
    let p = rs_find_name_end(
        *arg,
        &mut expr_start,
        &mut expr_end,
        if len > 0 { 0 } else { FNE_CHECK_START },
    );

    if !expr_start.is_null() {
        if !evaluate {
            // len = script prefix length; add name portion length
            len += p.offset_from(*arg) as c_int;
            *arg = rs_skipwhite(p);
            return len;
        }

        // Include any <SID> etc in the expanded string; thus -len offset here.
        let temp_string = make_expanded_name_impl(
            (*arg).sub(len as usize),
            expr_start.cast_mut(),
            expr_end.cast_mut(),
            p.cast_mut(),
        );
        if temp_string.is_null() {
            return -1;
        }
        *alias = temp_string;
        *arg = rs_skipwhite(p);
        return c_strlen(temp_string) as c_int;
    }

    let id_len = rs_get_id_len(arg);
    len += id_len;

    // Only give an error when there is something; otherwise reported higher up.
    if len == 0 && verbose && **arg != 0 {
        nvim_semsg_invexpr2(*arg);
    }

    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_chars() {
        assert_eq!(NAMESPACE_CHARS, b"abglstvw");
    }

    #[test]
    fn test_fne_constants() {
        assert_eq!(FNE_INCL_BR, 1);
        assert_eq!(FNE_CHECK_START, 2);
    }
}
