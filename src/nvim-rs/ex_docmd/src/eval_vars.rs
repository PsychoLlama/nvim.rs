//! Phase 3: eval_vars_impl migrated from ex_docmd.c.
//!
//! Evaluates cmdline special variables: `%`, `#`, `<cword>`, `<cWORD>`,
//! `<cexpr>`, `<cfile>`, `<sfile>`, `<stack>`, `<script>`, `<slnum>`,
//! `<sflnum>`, `<afile>`, `<abuf>`, `<amatch>`, `<SID>`.

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

// ---------------------------------------------------------------------------
// SPEC_* indices — must match SPEC_STRINGS order in commands.rs
// ---------------------------------------------------------------------------
const SPEC_PERC: isize = 0;
const SPEC_HASH: isize = 1;
const SPEC_CWORD: isize = 2;
const SPEC_CCWORD: isize = 3;
const SPEC_CEXPR: isize = 4;
const SPEC_CFILE: isize = 5;
const SPEC_SFILE: isize = 6;
const SPEC_SLNUM: isize = 7;
const SPEC_STACK: isize = 8;
const SPEC_SCRIPT: isize = 9;
const SPEC_AFILE: isize = 10;
const SPEC_ABUF: isize = 11;
const SPEC_AMATCH: isize = 12;
const SPEC_SFLNUM: isize = 13;
const SPEC_SID: isize = 14;

// VALID_PATH / VALID_HEAD from ex_docmd.h
const VALID_PATH: c_int = 1;
const VALID_HEAD: c_int = 2;

// ECMD_LAST from ex_cmds.h
const ECMD_LAST: i32 = -1;

// FIND_IDENT / FIND_STRING / FIND_EVAL from normal.h
const FIND_IDENT: c_int = 1;
const FIND_STRING: c_int = 2;
const FIND_EVAL: c_int = 4;

// ESTACK_* from runtime_defs.h (estack_arg_T enum: NONE=0, SFILE=1, STACK=2, SCRIPT=3)
const ESTACK_SFILE: c_int = 1;
const ESTACK_STACK: c_int = 2;
const ESTACK_SCRIPT: c_int = 3;

// ---------------------------------------------------------------------------
// FFI declarations
// ---------------------------------------------------------------------------

/// Rust mirror of C `sctx_T`.
///
/// Layout matches: { int sc_sid; int sc_seq; int32_t sc_lnum; uint64_t sc_chan; }
#[repr(C)]
struct SctxT {
    sc_sid: c_int,
    sc_seq: c_int,
    sc_lnum: i32,
    _sc_chan: u64,
}

extern "C" {
    /// find_cmdline_var — exported from Rust (commands.rs)
    fn find_cmdline_var(src: *const c_char, usedlen: *mut usize) -> isize;

    /// rs_find_ident_under_cursor — Rust normal-mode function
    fn rs_find_ident_under_cursor(text: *mut *mut c_char, find_type: c_int) -> usize;

    // memory helpers
    fn xfree(p: *mut c_void);
    fn xmemdupz(data: *const c_char, len: usize) -> *mut c_char;

    // string helpers
    fn strlen(s: *const c_char) -> usize;
    fn strrchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    fn snprintf(s: *mut c_char, n: usize, format: *const c_char, ...) -> c_int;
    fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;

    // path helpers
    fn path_tail(p: *const c_char) -> *const c_char;
    fn path_try_shorten_fname(full_path: *mut c_char) -> *mut c_char;

    // modify_fname
    fn modify_fname(
        src: *mut c_char,
        tilde_file: bool,
        usedlen: *mut usize,
        fnamep: *mut *mut c_char,
        bufp: *mut *mut c_char,
        fnamelen: *mut usize,
    ) -> c_int;

    // curbuf accessors
    fn nvim_docmd_eval_curbuf_fname() -> *mut c_char;

    // buflist_findnr (Rust-implemented; returns buf_T*)
    fn rs_buflist_findnr(nr: c_int) -> *mut c_void;

    // buf_T->b_fname accessor
    fn nvim_buf_get_b_fname(buf: *mut c_void) -> *const c_char;

    // file_name_at_cursor (FNAME_MESS|FNAME_HYP=5, count=1, lnum=NULL)
    fn file_name_at_cursor(options: c_int, count: c_int, file_lnum: *mut c_void) -> *mut c_char;

    // FullName_save
    fn FullName_save(fname: *const c_char, force: bool) -> *mut c_char;

    // autocmd variable accessors (EXTERN globals)
    static mut autocmd_fname: *mut c_char;
    static mut autocmd_fname_full: bool;
    static autocmd_match: *const c_char;
    fn nvim_get_autocmd_bufnr() -> c_int;
    // xstrlcpy (Rust-exported from strings crate)
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize;

    // estack_sfile
    fn estack_sfile(which: c_int) -> *mut c_char;

    // SOURCING_NAME / SOURCING_LNUM
    fn nvim_get_sourcing_name() -> *const c_char;
    fn nvim_get_sourcing_lnum() -> c_int;

    // current_sctx (direct access)
    static current_sctx: SctxT;

    // oldfiles (wraps tv_list_find_str(get_vim_var_list(VV_OLDFILES), idx))
    fn nvim_excmds_oldfiles_find_str(idx: c_int) -> *const c_char;

    // arg_all
    fn arg_all() -> *mut c_char;

    // getdigits_int (direct C function)
    fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int;

}

// ---------------------------------------------------------------------------
// Public export: nvim_docmd_eval_vars_impl
// ---------------------------------------------------------------------------

/// Evaluate a cmdline special variable starting at `src`.
///
/// This is the Rust port of the C `nvim_docmd_eval_vars_impl` function.
///
/// # Safety
///
/// All pointer arguments must be valid as described in the original C documentation.
#[export_name = "eval_vars"]
pub unsafe extern "C" fn rs_eval_vars_impl(
    src: *mut c_char,
    srcstart: *const c_char,
    usedlen: *mut usize,
    lnump: *mut i32,
    errormsg: *mut *const c_char,
    escaped: *mut c_int,
    empty_is_error: bool,
) -> *mut c_char {
    *errormsg = ptr::null();
    if !escaped.is_null() {
        *escaped = 0;
    }

    // Check if there is something to do.
    let spec_idx = find_cmdline_var(src, usedlen);
    if spec_idx < 0 {
        // no match
        *usedlen = 1;
        return ptr::null_mut();
    }

    // Skip when preceded by a backslash: "\%" and "\#".
    // Note: In "\\%" the % is also not recognised!
    if src > srcstart as *mut c_char && *src.sub(1) as u8 == b'\\' {
        *usedlen = 0;
        // STRMOVE(src - 1, src) — remove backslash
        let dst = src.sub(1) as *mut c_void;
        let s = src as *const c_void;
        let len = strlen(src) + 1;
        memmove(dst, s, len);
        return ptr::null_mut();
    }

    let mut result: *mut c_char = c"".as_ptr().cast_mut();
    let mut resultbuf: *mut c_char = ptr::null_mut();
    let mut resultlen: usize;
    let mut valid: c_int = VALID_HEAD | VALID_PATH;
    let mut tilde_file = false;
    let mut skip_mod = false;
    // Stack buffer for number formatting (30 bytes, like strbuf[30] in C)
    let mut strbuf = [0u8; 30];

    // word or WORD under cursor
    if spec_idx == SPEC_CWORD || spec_idx == SPEC_CCWORD || spec_idx == SPEC_CEXPR {
        let find_type = if spec_idx == SPEC_CWORD {
            FIND_IDENT | FIND_STRING
        } else if spec_idx == SPEC_CEXPR {
            FIND_IDENT | FIND_STRING | FIND_EVAL
        } else {
            FIND_STRING
        };
        resultlen = rs_find_ident_under_cursor(&raw mut result, find_type);
        if resultlen == 0 {
            *errormsg = c"".as_ptr();
            return ptr::null_mut();
        }
        // C skips the else branch entirely for cword/ccword/cexpr:
        // no extension stripping, no modify_fname. Fall straight through to
        // the final validity check.
        return finish_result(
            result,
            resultbuf,
            resultlen,
            valid,
            empty_is_error,
            errormsg,
        );
    }

    // All other spec indices go through the switch.
    match spec_idx {
        SPEC_PERC => {
            // '%': current file
            let fname = nvim_docmd_eval_curbuf_fname();
            if fname.is_null() {
                result = c"".as_ptr().cast_mut();
                valid = 0; // Must have ":p:h" to be valid
            } else {
                result = fname;
                tilde_file = strcmp(result, c"~".as_ptr()) == 0;
            }
        }

        SPEC_HASH => {
            // '#' or "#99": alternate file
            if *src.add(1) as u8 == b'#' {
                // "##": the argument list
                result = arg_all();
                resultbuf = result;
                *usedlen = 2;
                if !escaped.is_null() {
                    *escaped = 1;
                }
                skip_mod = true;
                // Falls through to the common resultlen / extension-strip block.
            } else {
                // Parse optional number after '#'
                let mut s = src.add(1);
                let use_oldfiles = *s as u8 == b'<';
                if use_oldfiles {
                    s = s.add(1);
                }
                let i = getdigits_int(&raw mut s, false, 0);
                // If s == src+2 and src[1] == '-': just a minus sign, don't skip over it
                if s == src.add(2) && *src.add(1) as u8 == b'-' {
                    s = s.sub(1);
                }
                *usedlen = (s as usize) - (src as usize);

                if use_oldfiles && i != 0 {
                    if *usedlen < 2 {
                        // Should we give an error for #<text?
                        *usedlen = 1;
                        return ptr::null_mut();
                    }
                    let r = nvim_excmds_oldfiles_find_str(i - 1);
                    if r.is_null() {
                        *errormsg = c"".as_ptr();
                        return ptr::null_mut();
                    }
                    result = r as *mut c_char;
                } else {
                    if i == 0 && use_oldfiles && *usedlen > 1 {
                        *usedlen = 1;
                    }
                    let buf = rs_buflist_findnr(i);
                    if buf.is_null() {
                        *errormsg = crate::gt(crate::E_NO_ALT_FILE_STR.as_ptr());
                        return ptr::null_mut();
                    }
                    if !lnump.is_null() {
                        *lnump = ECMD_LAST;
                    }
                    let fname = nvim_buf_get_b_fname(buf);
                    if fname.is_null() {
                        result = c"".as_ptr().cast_mut();
                        valid = 0; // Must have ":p:h" to be valid
                    } else {
                        result = fname as *mut c_char;
                        tilde_file = strcmp(result, c"~".as_ptr()) == 0;
                    }
                }
            }
        }

        SPEC_CFILE => {
            // file name under cursor
            result = file_name_at_cursor(5, 1, std::ptr::null_mut());
            if result.is_null() {
                *errormsg = c"".as_ptr();
                return ptr::null_mut();
            }
            resultbuf = result; // remember allocated string
        }

        SPEC_AFILE => {
            // file name for autocommand
            let fname = autocmd_fname;
            if !fname.is_null() && !autocmd_fname_full {
                // Still need to turn the fname into a full path.
                // Postponed to avoid a delay when <afile> is not used.
                autocmd_fname_full = true;
                let full = FullName_save(fname, false);
                // Copy into autocmd_fname, don't reassign it. #8165
                xstrlcpy(fname, full, 4096); // MAXPATHL
                xfree(full as *mut c_void);
            }
            result = autocmd_fname;
            if result.is_null() {
                *errormsg = crate::gt(crate::E_NO_AFILE_STR.as_ptr());
                return ptr::null_mut();
            }
            result = path_try_shorten_fname(result);
        }

        SPEC_ABUF => {
            // buffer number for autocommand
            let bufnr = nvim_get_autocmd_bufnr();
            if bufnr <= 0 {
                *errormsg = crate::gt(crate::E_NO_ABUF_STR.as_ptr());
                return ptr::null_mut();
            }
            snprintf(
                strbuf.as_mut_ptr() as *mut c_char,
                strbuf.len(),
                c"%d".as_ptr(),
                bufnr,
            );
            result = strbuf.as_mut_ptr() as *mut c_char;
        }

        SPEC_AMATCH => {
            // match name for autocommand
            result = autocmd_match as *mut c_char;
            if result.is_null() {
                *errormsg = crate::gt(crate::E_NO_AMATCH_STR.as_ptr());
                return ptr::null_mut();
            }
        }

        SPEC_SFILE => {
            // file name for ":so" command
            result = estack_sfile(ESTACK_SFILE);
            if result.is_null() {
                *errormsg = crate::gt(crate::E_NO_SFILE_STR.as_ptr());
                return ptr::null_mut();
            }
            resultbuf = result; // remember allocated string
        }

        SPEC_STACK => {
            // call stack
            result = estack_sfile(ESTACK_STACK);
            if result.is_null() {
                *errormsg = crate::gt(crate::E_NO_STACK_STR.as_ptr());
                return ptr::null_mut();
            }
            resultbuf = result;
        }

        SPEC_SCRIPT => {
            // script file name
            result = estack_sfile(ESTACK_SCRIPT);
            if result.is_null() {
                *errormsg = crate::gt(crate::E_NO_SCRIPT_STR.as_ptr());
                return ptr::null_mut();
            }
            resultbuf = result;
        }

        SPEC_SLNUM => {
            // line in file for ":so" command
            let sourcing_name = nvim_get_sourcing_name();
            let sourcing_lnum = nvim_get_sourcing_lnum();
            if sourcing_name.is_null() || sourcing_lnum == 0 {
                *errormsg = crate::gt(crate::E_NO_SLNUM_STR.as_ptr());
                return ptr::null_mut();
            }
            snprintf(
                strbuf.as_mut_ptr() as *mut c_char,
                strbuf.len(),
                c"%d".as_ptr(),
                sourcing_lnum,
            );
            result = strbuf.as_mut_ptr() as *mut c_char;
        }

        SPEC_SFLNUM => {
            // line in script file
            let sc_lnum = current_sctx.sc_lnum;
            let sourcing_lnum = nvim_get_sourcing_lnum();
            if sc_lnum + sourcing_lnum == 0 {
                *errormsg = crate::gt(crate::E_NO_SFLNUM_STR.as_ptr());
                return ptr::null_mut();
            }
            snprintf(
                strbuf.as_mut_ptr() as *mut c_char,
                strbuf.len(),
                c"%d".as_ptr(),
                sc_lnum + sourcing_lnum,
            );
            result = strbuf.as_mut_ptr() as *mut c_char;
        }

        SPEC_SID => {
            let sc_sid = current_sctx.sc_sid;
            if sc_sid <= 0 {
                *errormsg = crate::gt(crate::E_USINGSID_STR.as_ptr());
                return ptr::null_mut();
            }
            snprintf(
                strbuf.as_mut_ptr() as *mut c_char,
                strbuf.len(),
                c"<SNR>%d_".as_ptr(),
                sc_sid,
            );
            result = strbuf.as_mut_ptr() as *mut c_char;
        }

        _ => {
            // should not happen
            *errormsg = c"".as_ptr();
        }
    }

    // Length of new string.
    resultlen = strlen(result);

    // Remove the file name extension.
    if *src.add(*usedlen) as u8 == b'<' {
        *usedlen += 1;
        let dot = strrchr(result, b'.' as c_int);
        if !dot.is_null() && dot >= path_tail(result) as *mut c_char {
            resultlen = (dot as usize) - (result as usize);
        }
    } else if !skip_mod {
        valid |= modify_fname(
            src,
            tilde_file,
            usedlen,
            &raw mut result,
            &raw mut resultbuf,
            &raw mut resultlen,
        );
        if result.is_null() {
            *errormsg = c"".as_ptr();
            xfree(resultbuf as *mut c_void);
            return ptr::null_mut();
        }
    }

    finish_result(
        result,
        resultbuf,
        resultlen,
        valid,
        empty_is_error,
        errormsg,
    )
}

/// Final result validation: duplicates `result` on success or sets `errormsg` on failure.
///
/// # Safety
/// All pointer arguments must be valid.
#[inline]
unsafe fn finish_result(
    result: *mut c_char,
    resultbuf: *mut c_char,
    resultlen: usize,
    valid: c_int,
    empty_is_error: bool,
    errormsg: *mut *const c_char,
) -> *mut c_char {
    if resultlen == 0 || valid != VALID_HEAD + VALID_PATH {
        if empty_is_error {
            if valid != VALID_HEAD + VALID_PATH {
                // xgettext:no-c-format
                *errormsg = crate::gt(crate::E_EMPTY_FNAME_STR.as_ptr());
            } else {
                *errormsg = crate::gt(crate::E_EMPTY_STRING_STR.as_ptr());
            }
        }
        xfree(resultbuf as *mut c_void);
        ptr::null_mut()
    } else {
        let dup = xmemdupz(result, resultlen);
        xfree(resultbuf as *mut c_void);
        dup
    }
}
