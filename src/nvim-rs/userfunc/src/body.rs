//! Function body parser for VimL.
//!
//! Phase 7 (plan db85cc6b) from `src/nvim/eval/userfunc.c`:
//! - `get_function_body` — reads lines until `:endfunction`, managing nesting,
//!   heredocs, and line continuation via SOURCING_LNUM.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::if_not_else)]
#![allow(clippy::cast_lossless)]

use std::ffi::{c_char, c_int, c_void};

use super::parsing::GarrayT;

// =============================================================================
// Return codes (matching C defines in vim_defs.h)
// =============================================================================

const OK: c_int = 1;
const FAIL: c_int = 0;

// Maximum function nesting depth (matches #define MAX_FUNC_NESTING 50)
const MAX_FUNC_NESTING: c_int = 50;

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    // --- memory ---
    fn xfree(ptr: *mut c_void);
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xmemdupz(src: *const c_void, len: usize) -> *mut c_void;

    // --- string ops ---
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    fn strncmp(a: *const c_char, b: *const c_char, n: usize) -> c_int;
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn skiptowhite(p: *const c_char) -> *mut c_char;

    // --- cmdline input ---
    fn getcmdline(firstc: c_int, count: c_int, indent: c_int, do_concat: bool) -> *mut c_char;

    // --- garray ---
    fn ga_grow(gap: *mut GarrayT, n: c_int);

    // --- ex_docmd helpers ---
    #[link_name = "checkforcmd"]
    fn rs_checkforcmd(pp: *mut *mut c_char, cmd: *const c_char, len: c_int) -> bool;
    fn skip_range(cmd: *const c_char, ctx: *mut c_int) -> *const c_char;
    fn skip_var_list(
        arg: *const c_char,
        var_count: *mut c_int,
        semicolon: *mut c_int,
        silent: bool,
    ) -> *const c_char;

    // --- function name helpers ---
    fn trans_function_name(
        pp: *mut *mut c_char,
        skip: c_int,
        flags: c_int,
        fdp: *mut c_void,
        partial: *mut c_void,
    ) -> *mut c_char;

    // --- eap accessors ---
    fn nvim_eap_has_getline(eap: *const c_void) -> c_int;
    fn nvim_eap_call_getline_concat(
        eap: *mut c_void,
        c: c_int,
        indent: c_int,
        do_concat: bool,
    ) -> *mut c_char;
    fn nvim_eap_get_sourced_lnum(eap: *const c_void) -> i32;
    fn nvim_eap_get_cmdlinep(eap: *mut c_void) -> *mut *mut c_char;
    fn nvim_eap_set_nextcmd(eap: *mut c_void, val: *mut c_char);

    // --- eval_fname_script (Rust export from names.rs in same crate) ---
    fn eval_fname_script(p: *const c_char) -> c_int;

    // --- ui ---
    fn ui_ext_cmdline_block_append(indent: usize, s: *const c_char);

    // --- error messages (wrappers in userfunc.c) ---
    fn nvim_userfunc_swmsg_w22(p: *const c_char);
    fn nvim_userfunc_emsg_e126();
    fn nvim_userfunc_emsg_e1058();
    fn nvim_userfunc_semsg_e1145(marker: *const c_char);

    // --- globals ---
    static mut KeyTyped: bool;
    static mut msg_scroll: c_int;
    static mut need_wait_return: bool;
    static mut did_emsg: c_int;
    static mut lines_left: c_int;
    static Rows: c_int;
    fn nvim_get_p_verbose() -> c_int;
    fn nvim_rt_get_sourcing_lnum() -> c_int;
}

// =============================================================================
// Helper: ASCII character classification (inline, no C call needed)
// =============================================================================

#[inline]
fn ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

#[inline]
fn ascii_iswhite_nl_or_nul(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\n' || c == 0
}

#[inline]
fn ascii_isalnum(c: u8) -> bool {
    c.is_ascii_alphanumeric()
}

#[inline]
fn ascii_isalpha(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

// =============================================================================
// rs_get_function_body
// =============================================================================

/// Read the body of a `:function` definition, collecting lines into `newlines`.
///
/// Stops at `:endfunction`. Manages nesting (`if`/`while`/`for`/`try`),
/// heredocs (`:python <<EOF`, `:let v =<< trim EOF`), and line continuation.
///
/// Returns `OK` on success, `FAIL` on error (including missing endfunction).
///
/// Mirrors `get_function_body()` from `src/nvim/eval/userfunc.c`.
///
/// # Safety
/// All pointers must be valid. `eap`, `newlines`, `line_to_free` must be
/// non-null. `line_arg_in` may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_get_function_body(
    eap: *mut c_void,
    newlines: *mut GarrayT,
    line_arg_in: *mut c_char,
    line_to_free: *mut *mut c_char,
    show_block: bool,
) -> c_int {
    let saved_wait_return = need_wait_return;
    let mut line_arg: *mut c_char = line_arg_in;
    let mut indent: c_int = 2;
    let mut nesting: c_int = 0;
    let mut skip_until: *mut c_char = std::ptr::null_mut();
    let mut ret = FAIL;
    let mut is_heredoc = false;
    let mut heredoc_trimmed: *mut c_char = std::ptr::null_mut();
    let mut heredoc_trimmedlen: usize = 0;
    let mut do_concat = true;

    // found_end is set to true when the matching :endfunction is found.
    let mut found_end = false;

    'mainloop: loop {
        // --- per-iteration KeyTyped check ---
        if KeyTyped {
            msg_scroll = 1;
            need_wait_return = false;
        }
        need_wait_return = false;

        // --- get next line ---
        let theline: *mut c_char;

        if !line_arg.is_null() {
            // Use eap->arg split by '\n'.
            theline = line_arg;
            let nl = vim_strchr(theline, b'\n' as c_int);
            if nl.is_null() {
                let len = strlen(theline);
                line_arg = line_arg.add(len);
            } else {
                *nl = 0;
                line_arg = nl.add(1);
            }
        } else {
            xfree(*line_to_free as *mut c_void);
            let fetched = if nvim_eap_has_getline(eap) == 0 {
                getcmdline(b':' as c_int, 0, indent, do_concat)
            } else {
                nvim_eap_call_getline_concat(eap, b':' as c_int, indent, do_concat)
            };
            *line_to_free = fetched;
            theline = fetched;
        }

        if KeyTyped {
            lines_left = Rows - 1;
        }

        if theline.is_null() {
            // Missing :endfunction (or heredoc end marker).
            if !skip_until.is_null() {
                nvim_userfunc_semsg_e1145(skip_until);
            } else {
                nvim_userfunc_emsg_e126();
            }
            break 'mainloop; // goto theend
        }

        if show_block {
            debug_assert!(indent >= 0);
            ui_ext_cmdline_block_append(indent as usize, theline);
        }

        // --- detect line continuation via SOURCING_LNUM ---
        let sourcing_lnum_now = nvim_rt_get_sourcing_lnum();
        let raw_sourced = nvim_eap_get_sourced_lnum(eap);
        let sourcing_lnum_off: usize = if sourcing_lnum_now < raw_sourced {
            (raw_sourced - sourcing_lnum_now) as usize
        } else {
            0
        };

        // --- handle skip_until (inside heredoc / append block) ---
        if !skip_until.is_null() {
            let should_check = heredoc_trimmed.is_null()
                || (is_heredoc && skipwhite(theline) == theline)
                || strncmp(theline, heredoc_trimmed, heredoc_trimmedlen) == 0;

            if should_check {
                let p: *const c_char = if heredoc_trimmed.is_null() {
                    theline
                } else if is_heredoc {
                    if skipwhite(theline) == theline {
                        theline
                    } else {
                        theline.add(heredoc_trimmedlen)
                    }
                } else {
                    theline.add(heredoc_trimmedlen)
                };

                if strcmp(p, skip_until) == 0 {
                    xfree(skip_until as *mut c_void);
                    skip_until = std::ptr::null_mut();
                    xfree(heredoc_trimmed as *mut c_void);
                    heredoc_trimmed = std::ptr::null_mut();
                    heredoc_trimmedlen = 0;
                    do_concat = true;
                    is_heredoc = false;
                }
            }
        } else {
            // --- normal line: skip ':' and blanks ---
            let mut p: *mut c_char = theline;
            while *p != 0 && (ascii_iswhite(*p as u8) || *p == b':' as c_char) {
                p = p.add(1);
            }

            // Check for "endfunction".
            // C: if (checkforcmd(&p, "endfunction", 4) && nesting-- == 0)
            // nesting-- returns the old value; we check if old value == 0.
            if rs_checkforcmd(std::ptr::addr_of_mut!(p), c"endfunction".as_ptr(), 4) {
                let old_nesting = nesting;
                nesting -= 1;
                if old_nesting == 0 {
                    // This is the matching :endfunction.
                    if *p == b'!' as c_char {
                        p = p.add(1);
                    }
                    let mut nextcmd: *mut c_char = std::ptr::null_mut();
                    if *p == b'|' as c_char {
                        nextcmd = p.add(1);
                    } else if !line_arg.is_null() && *skipwhite(line_arg) != 0 {
                        nextcmd = line_arg;
                    } else if *p != 0 && *p != b'"' as c_char && nvim_get_p_verbose() > 0 {
                        nvim_userfunc_swmsg_w22(p);
                    }
                    if !nextcmd.is_null() {
                        nvim_eap_set_nextcmd(eap, nextcmd);
                        if !(*line_to_free).is_null() {
                            let cmdlinep = nvim_eap_get_cmdlinep(eap);
                            if !cmdlinep.is_null() {
                                xfree(*cmdlinep as *mut c_void);
                                *cmdlinep = *line_to_free;
                            }
                            *line_to_free = std::ptr::null_mut();
                        }
                    }
                    found_end = true;
                    break 'mainloop;
                }
                // else: nesting was > 0, now one less; continue loop
            } else {
                // Increase/decrease indent for block keywords.
                if indent > 2 && strncmp(p, c"end".as_ptr(), 3) == 0 {
                    indent -= 2;
                } else if strncmp(p, c"if".as_ptr(), 2) == 0
                    || strncmp(p, c"wh".as_ptr(), 2) == 0
                    || strncmp(p, c"for".as_ptr(), 3) == 0
                    || strncmp(p, c"try".as_ptr(), 3) == 0
                {
                    indent += 2;
                }

                // Check for nested function definition.
                let mut pf = p;
                if rs_checkforcmd(std::ptr::addr_of_mut!(pf), c"function".as_ptr(), 2) {
                    if *pf == b'!' as c_char {
                        pf = skipwhite(pf.add(1));
                    }
                    let script_off = eval_fname_script(pf) as usize;
                    pf = pf.add(script_off);
                    let freed = trans_function_name(
                        std::ptr::addr_of_mut!(pf),
                        1, // skip = true
                        0,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    );
                    xfree(freed as *mut c_void);
                    if *skipwhite(pf) == b'(' as c_char {
                        if nesting == MAX_FUNC_NESTING - 1 {
                            nvim_userfunc_emsg_e1058();
                        } else {
                            nesting += 1;
                            indent += 2;
                        }
                    }
                }

                // Check for ":append", ":change", ":insert" (body until ".").
                // C: char *const tp = p = skip_range(p, NULL);
                let tp: *mut c_char = skip_range(p, std::ptr::null_mut()).cast_mut();
                let mut pa: *mut c_char = tp;
                if (rs_checkforcmd(std::ptr::addr_of_mut!(pa), c"append".as_ptr(), 1)
                    || {
                        pa = tp;
                        rs_checkforcmd(std::ptr::addr_of_mut!(pa), c"change".as_ptr(), 1)
                    }
                    || {
                        pa = tp;
                        rs_checkforcmd(std::ptr::addr_of_mut!(pa), c"insert".as_ptr(), 1)
                    })
                    && (*pa == b'!' as c_char
                        || *pa == b'|' as c_char
                        || ascii_iswhite_nl_or_nul(*pa as u8))
                {
                    skip_until = xmemdupz(c".".as_ptr().cast::<c_void>(), 1).cast::<c_char>();
                } else {
                    // p = tp (but tp is already past range; p not used further in
                    // this branch so this is a no-op for our purposes)
                }

                // heredoc: ":python <<EOF", ":lua <<EOF", etc.
                // C uses p = tp first; we use tp directly.
                let arg = skipwhite(skiptowhite(tp));
                if !arg.is_null()
                    && *arg == b'<' as c_char
                    && *arg.add(1) == b'<' as c_char
                    && is_embed_script_cmd(tp)
                {
                    let mut ph = skipwhite(arg.add(2));
                    if strncmp(ph, c"trim".as_ptr(), 4) == 0
                        && (*ph.add(4) == 0 || ascii_iswhite(*ph.add(4) as u8))
                    {
                        ph = skipwhite(ph.add(4));
                        heredoc_trimmedlen = (skipwhite(theline) as usize) - (theline as usize);
                        heredoc_trimmed =
                            xmemdupz(theline as *const c_void, heredoc_trimmedlen).cast::<c_char>();
                    }
                    if *ph == 0 {
                        skip_until = xmemdupz(c".".as_ptr().cast::<c_void>(), 1).cast::<c_char>();
                    } else {
                        let end = skiptowhite(ph);
                        let marker_len = (end as usize) - (ph as usize);
                        skip_until = xmemdupz(ph as *const c_void, marker_len).cast::<c_char>();
                    }
                    do_concat = false;
                    is_heredoc = true;
                }

                if !is_heredoc {
                    // Check for ":let v =<< [trim] EOF" heredoc.
                    // C: arg = p; if (checkforcmd(&arg, "let", 2)) { ... }
                    // p was set to tp above; use tp here.
                    let mut arg2 = tp;
                    if rs_checkforcmd(std::ptr::addr_of_mut!(arg2), c"let".as_ptr(), 2) {
                        let mut var_count: c_int = 0;
                        let mut semicolon: c_int = 0;
                        let after_vars = skip_var_list(
                            arg2,
                            std::ptr::addr_of_mut!(var_count),
                            std::ptr::addr_of_mut!(semicolon),
                            true,
                        );
                        let mut av: *mut c_char = if after_vars.is_null() {
                            std::ptr::null_mut()
                        } else {
                            skipwhite(after_vars)
                        };
                        if !av.is_null() && strncmp(av, c"=<<".as_ptr(), 3) == 0 {
                            av = skipwhite(av.add(3));
                            let mut has_trim = false;
                            loop {
                                if strncmp(av, c"trim".as_ptr(), 4) == 0
                                    && (*av.add(4) == 0 || ascii_iswhite(*av.add(4) as u8))
                                {
                                    av = skipwhite(av.add(4));
                                    has_trim = true;
                                    continue;
                                }
                                if strncmp(av, c"eval".as_ptr(), 4) == 0
                                    && (*av.add(4) == 0 || ascii_iswhite(*av.add(4) as u8))
                                {
                                    av = skipwhite(av.add(4));
                                    continue;
                                }
                                break;
                            }
                            if has_trim {
                                heredoc_trimmedlen =
                                    (skipwhite(theline) as usize) - (theline as usize);
                                heredoc_trimmed =
                                    xmemdupz(theline as *const c_void, heredoc_trimmedlen)
                                        .cast::<c_char>();
                            }
                            xfree(skip_until as *mut c_void);
                            let end = skiptowhite(av);
                            let marker_len = (end as usize) - (av as usize);
                            skip_until = xmemdupz(av as *const c_void, marker_len).cast::<c_char>();
                            do_concat = false;
                            is_heredoc = true;
                        }
                    }
                }
            }
        }

        // --- add the line to the function body ---
        ga_grow(newlines, 1 + sourcing_lnum_off as c_int);

        let copy = xstrdup(theline);
        let data_ptr: *mut *mut c_char = (*newlines).ga_data.cast::<*mut c_char>();
        let idx = (*newlines).ga_len as usize;
        *data_ptr.add(idx) = copy;
        (*newlines).ga_len += 1;

        // Add NULL lines for continuation lines (keeps line indices correct).
        let mut off = sourcing_lnum_off;
        while off > 0 {
            off -= 1;
            let idx2 = (*newlines).ga_len as usize;
            *data_ptr.add(idx2) = std::ptr::null_mut();
            (*newlines).ga_len += 1;
        }

        // Check for end of eap->arg inline content.
        if !line_arg.is_null() && *line_arg == 0 {
            line_arg = std::ptr::null_mut();
        }
    } // 'mainloop

    // Return OK when no error was detected (and we found :endfunction).
    if found_end && did_emsg == 0 {
        ret = OK;
    }

    // theend:
    xfree(skip_until as *mut c_void);
    xfree(heredoc_trimmed as *mut c_void);
    need_wait_return |= saved_wait_return;
    ret
}

// =============================================================================
// is_embed_script_cmd
// =============================================================================

/// Check whether the keyword at `p` is a command that accepts `<<EOF` heredoc
/// syntax (python/py/py3/pyx, perl/pe, tcl, lua, ruby/rub, mzscheme/mzs).
///
/// Mirrors the inline expression in `get_function_body`.
unsafe fn is_embed_script_cmd(p: *const c_char) -> bool {
    let b = |off: usize| -> u8 { *p.add(off) as u8 };
    // python / py / py3 / pyx
    (b(0) == b'p'
        && b(1) == b'y'
        && (!ascii_isalnum(b(2))
            || b(2) == b't'
            || ((b(2) == b'3' || b(2) == b'x') && !ascii_isalpha(b(3)))))
        // perl / pe
        || (b(0) == b'p' && b(1) == b'e' && (!ascii_isalpha(b(2)) || b(2) == b'r'))
        // tcl
        || (b(0) == b't' && b(1) == b'c' && (!ascii_isalpha(b(2)) || b(2) == b'l'))
        // lua
        || (b(0) == b'l' && b(1) == b'u' && b(2) == b'a' && !ascii_isalpha(b(3)))
        // ruby / rub
        || (b(0) == b'r'
            && b(1) == b'u'
            && b(2) == b'b'
            && (!ascii_isalpha(b(3)) || b(3) == b'y'))
        // mzscheme / mzs
        || (b(0) == b'm' && b(1) == b'z' && (!ascii_isalpha(b(2)) || b(2) == b's'))
}
