//! Heredoc and script_get functions for VimL.
//!
//! Phase 4: Migrated from `src/nvim/eval/vars.c`.
//!
//! Functions:
//! - `rs_heredoc_get`: Parse HERE document for :let =<<

#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::manual_c_str_literals)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    // --- eap accessors ---
    fn nvim_eap_has_getline(eap: *const c_void) -> c_int;
    fn nvim_eap_call_getline(eap: *mut c_void, c: c_int, indent: c_int) -> *mut c_char;
    fn nvim_eap_get_cmdlinep_str(eap: *const c_void) -> *const c_char;
    fn nvim_eap_get_skip(eap: *const c_void) -> c_int;
    fn nvim_eap_set_nextcmd(eap: *mut c_void, val: *mut c_char);

    // --- string ops ---
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;

    // --- char classification ---
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn skiptowhite(p: *const c_char) -> *mut c_char;
    fn rs_ascii_iswhite(c: c_int) -> c_int;
    fn islower(c: c_int) -> c_int;

    // --- list ops ---
    fn nvim_tv_list_alloc_wrapper(count: c_int) -> *mut c_void;
    #[link_name = "tv_list_append_string"]
    fn vars_tv_list_append_string(l: *mut c_void, s: *const c_char, len: i64);
    #[link_name = "tv_list_append_allocated_string"]
    fn vars_tv_list_append_allocated_string(l: *mut c_void, s: *mut c_char);
    fn tv_list_free(l: *mut c_void);

    // --- eval ---
    fn rs_eval_all_expr_in_str(str_: *mut c_char) -> *mut c_char;

    // --- memory ---
    fn xmemdupz(src: *const c_char, len: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);

    // --- error messages ---
    fn emsg(msg: *const c_char) -> c_int;
    fn semsg(fmt: *const c_char, ...) -> c_int;
}

// Error message string constants (must match gettext keys exactly)
const E_CANNOT_HEREDOC_HERE: &std::ffi::CStr = c"E991: Cannot use =<< here";
const E_MISSING_END_MARKER: &std::ffi::CStr = c"E990: Missing end marker '%s'";
const E_TRAILING: &std::ffi::CStr = c"E488: Trailing characters: %s";
const E_LOWERCASE_MARKER: &std::ffi::CStr = c"E221: Marker cannot start with lower case letter";
const E_MISSING_MARKER: &std::ffi::CStr = c"E172: Missing marker";

/// Parse HERE document for :let =<<
///
/// Matches C `heredoc_get`. Returns an allocated list_T* on success, NULL on failure.
///
/// # Safety
/// `eap` must be a valid `exarg_T*`. `cmd` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_heredoc_get(
    eap: *mut c_void,
    cmd: *mut c_char,
    script_get: c_int,
) -> *mut c_void {
    let mut cmd = cmd;
    let mut marker_indent_len: c_int = 0;
    let mut text_indent_len: c_int = 0;
    let mut text_indent: *mut c_char = std::ptr::null_mut();
    // Use a dot marker for script_get fallback
    let dot_buf = b".\0";
    let mut heredoc_in_string = false;
    let mut line_arg: *mut c_char = std::ptr::null_mut();

    // Check for newline (heredoc embedded in a string)
    let nl_ptr = vim_strchr(cmd, b'\n' as c_int);
    if !nl_ptr.is_null() {
        heredoc_in_string = true;
        line_arg = nl_ptr.add(1);
        *nl_ptr = 0; // NUL-terminate at newline
    } else if nvim_eap_has_getline(eap) == 0 {
        emsg(E_CANNOT_HEREDOC_HERE.as_ptr());
        return std::ptr::null_mut();
    }

    // Skip optional 'trim' / 'eval' keywords
    cmd = skipwhite(cmd);
    let mut evalstr = false;
    let mut eval_failed = false;
    loop {
        if strncmp(cmd, b"trim\0".as_ptr() as *const c_char, 4) == 0
            && (rs_ascii_iswhite(*cmd.add(4) as c_int) != 0 || *cmd.add(4) == 0)
        {
            cmd = skipwhite(cmd.add(4));
            // Trim the indentation. marker_indent_len = indent of the :let line.
            let cmdlinep_str = nvim_eap_get_cmdlinep_str(eap);
            let mut p = cmdlinep_str;
            while rs_ascii_iswhite(*p as c_int) != 0 {
                p = p.add(1);
                marker_indent_len += 1;
            }
            text_indent_len = -1;
            continue;
        }
        if strncmp(cmd, b"eval\0".as_ptr() as *const c_char, 4) == 0
            && (rs_ascii_iswhite(*cmd.add(4) as c_int) != 0 || *cmd.add(4) == 0)
        {
            cmd = skipwhite(cmd.add(4));
            evalstr = true;
            continue;
        }
        break;
    }

    // Determine the end marker
    let comment_char = b'"';
    let marker: *const c_char;
    if *cmd != 0 && *cmd as u8 != comment_char {
        marker = skipwhite(cmd);
        let p = skiptowhite(marker);
        let after = skipwhite(p);
        if *after != 0 && *after as u8 != comment_char {
            semsg(E_TRAILING.as_ptr(), p);
            return std::ptr::null_mut();
        }
        *(p as *mut c_char) = 0; // NUL-terminate marker
        if script_get == 0 && islower(*marker as c_int) != 0 {
            emsg(E_LOWERCASE_MARKER.as_ptr());
            return std::ptr::null_mut();
        }
    } else if script_get != 0 {
        marker = dot_buf.as_ptr() as *const c_char;
    } else {
        emsg(E_MISSING_MARKER.as_ptr());
        return std::ptr::null_mut();
    }

    let l = nvim_tv_list_alloc_wrapper(0);
    let mut theline: *mut c_char = std::ptr::null_mut();

    loop {
        let mut mi: usize = 0;
        let mut ti: usize = 0;

        if heredoc_in_string {
            // Get next line from embedded string
            if *line_arg == 0 {
                if script_get == 0 {
                    semsg(E_MISSING_END_MARKER.as_ptr(), marker);
                }
                break;
            }
            theline = line_arg;
            let next_line = vim_strchr(theline, b'\n' as c_int);
            if next_line.is_null() {
                line_arg = line_arg.add(strlen(line_arg));
            } else {
                *next_line = 0;
                line_arg = next_line.add(1);
            }
        } else {
            xfree(theline as *mut c_void);
            theline = nvim_eap_call_getline(eap, b'\0' as c_int, 0);
            if theline.is_null() {
                if script_get == 0 {
                    semsg(E_MISSING_END_MARKER.as_ptr(), marker);
                }
                break;
            }
        }

        // With "trim": skip the indent matching the :let line to find the marker
        if marker_indent_len > 0 {
            let cmdlinep_str = nvim_eap_get_cmdlinep_str(eap);
            if strncmp(theline, cmdlinep_str, marker_indent_len as usize) == 0 {
                mi = marker_indent_len as usize;
            }
        }
        if strcmp(marker, theline.add(mi)) == 0 {
            break;
        }

        // If expression evaluation failed, skip till end marker
        if eval_failed {
            continue;
        }

        if text_indent_len == -1 && *theline != 0 {
            // Set text indent from the first line
            let mut p = theline;
            text_indent_len = 0;
            while rs_ascii_iswhite(*p as c_int) != 0 {
                p = p.add(1);
                text_indent_len += 1;
            }
            text_indent = xmemdupz(theline, text_indent_len as usize) as *mut c_char;
        }
        // With "trim": skip the indent matching the first line
        if !text_indent.is_null() {
            let tilen = text_indent_len as usize;
            let mut j = 0usize;
            while j < tilen {
                if *theline.add(j) != *text_indent.add(j) {
                    break;
                }
                j += 1;
            }
            ti = j;
        }

        let str_ptr = theline.add(ti);
        if evalstr && nvim_eap_get_skip(eap) == 0 {
            let evaled = rs_eval_all_expr_in_str(str_ptr);
            if evaled.is_null() {
                eval_failed = true;
                continue;
            }
            vars_tv_list_append_allocated_string(l, evaled);
        } else {
            vars_tv_list_append_string(l, str_ptr, -1);
        }
    }

    if heredoc_in_string {
        nvim_eap_set_nextcmd(eap, line_arg);
    } else {
        xfree(theline as *mut c_void);
    }
    xfree(text_indent as *mut c_void);

    if eval_failed {
        tv_list_free(l);
        return std::ptr::null_mut();
    }
    l
}
