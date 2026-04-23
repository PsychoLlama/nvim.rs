//! Function listing and display for VimL.
//!
//! Implements `:function`, `:function FuncName`, `:function /pat` display.
//! Phase 1 migration from userfunc.c.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use nvim_ex_cmds_types::ExArg;
use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Opaque C types
// =============================================================================

/// Opaque handle for ufunc_T (C type).
pub type UfuncHandle = *mut c_void;

/// Opaque handle for exarg_T (C type).
pub type ExargHandle = *mut ExArg;

/// Rust mirror of C `sctx_T` (24 bytes, matches eval/display.rs SctxT).
#[repr(C)]
pub struct SctxT {
    pub sc_sid: c_int,
    pub sc_seq: c_int,
    pub sc_lnum: i32,
    // 4 bytes implicit padding
    pub sc_chan: u64,
}

// =============================================================================
// C Extern Declarations
// =============================================================================

extern "C" {
    // ufunc_T field accessors (added to userfunc.c Phase 1)
    fn nvim_ufunc_get_name(fp: UfuncHandle) -> *const c_char;
    fn nvim_ufunc_get_name_exp(fp: UfuncHandle) -> *const c_char;
    fn nvim_ufunc_get_namelen(fp: UfuncHandle) -> usize;
    fn nvim_ufunc_get_args_len(fp: UfuncHandle) -> c_int;
    fn nvim_ufunc_get_arg(fp: UfuncHandle, i: c_int) -> *const c_char;
    fn nvim_ufunc_get_def_args_len(fp: UfuncHandle) -> c_int;
    fn nvim_ufunc_get_def_arg(fp: UfuncHandle, i: c_int) -> *const c_char;
    fn nvim_ufunc_get_varargs(fp: UfuncHandle) -> c_int;
    fn nvim_ufunc_get_flags(fp: UfuncHandle) -> c_int;
    fn nvim_ufunc_get_script_ctx(fp: UfuncHandle) -> SctxT;
    fn nvim_ufunc_get_funcline(fp: UfuncHandle, i: c_int) -> *const c_char;
    fn nvim_ufunc_get_lines_len(fp: UfuncHandle) -> c_int;
    fn nvim_ufunc_name_refcount(name: *const c_char) -> c_int;

    // func_hashtab accessors
    fn nvim_func_ht_changed() -> c_int;
    fn nvim_func_ht_foreach(
        cb: unsafe extern "C" fn(fp: UfuncHandle, ctx: *mut c_void),
        ctx: *mut c_void,
    );

    // list_functions_matching_pat shim (uses C's list_functions with regmatch)
    fn nvim_list_functions_matching_pat(pat: *const c_char, ic: bool);

    // find_func
    fn find_func(name: *const c_char) -> UfuncHandle;

    // message functions
    fn msg_start();
    fn msg_puts(s: *const c_char);
    fn msg_putchar(c: c_int);
    fn msg_clr_eos();
    fn msg_prt_line(s: *const c_char, list: c_int);
    fn msg_outnum(n: c_int);

    // last_set_msg (implemented in eval/display.rs, exported as last_set_msg)
    fn last_set_msg(ctx: SctxT);

    // misc
    fn message_filtered(name: *const c_char) -> c_int;
    fn skip_regexp(p: *const c_char, delim: c_int, magic: c_int) -> *mut c_char;
    fn check_nextcmd(p: *const c_char) -> *mut c_char;
    fn ends_excmd(c: c_int) -> c_int;
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn line_breakcheck();

    // Translated error wrappers (defined in userfunc.c)
    fn nvim_emsg_function_list_modified();
    fn nvim_emsg_undefined_function(name: *const c_char);
    fn nvim_emsg_trailing_arg(name: *const c_char);

    // exarg_T accessors (already in ex_docmd.c / indent_ffi.c)
    fn nvim_eap_get_forceit(eap: ExargHandle) -> c_int;

    // globals
    static mut got_int: c_int;
    static p_verbose: c_int;
    static p_ic: c_int;
}

// FC_* flag constants (from userfunc.h)
const FC_ABORT: c_int = 0x01;
const FC_RANGE: c_int = 0x02;
const FC_DICT: c_int = 0x04;
const FC_CLOSURE: c_int = 0x08;

// =============================================================================
// rs_printable_func_name
// =============================================================================

/// Return the display name for a ufunc (expanded <SNR> form if available).
///
/// # Safety
/// `fp` must be a valid non-null ufunc_T pointer.
#[unsafe(export_name = "printable_func_name")]
pub unsafe extern "C" fn rs_printable_func_name(fp: UfuncHandle) -> *const c_char {
    let exp = nvim_ufunc_get_name_exp(fp);
    if exp.is_null() {
        nvim_ufunc_get_name(fp)
    } else {
        exp
    }
}

// =============================================================================
// rs_cat_func_name
// =============================================================================

/// Copy function name into buf, converting SNR prefix to <SNR>NNN_ form.
/// Returns the number of bytes written (not including NUL), or -1 on error.
///
/// # Safety
/// `fp` must be a valid non-null ufunc_T pointer.
/// `buf` must point to at least `bufsize` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_cat_func_name(
    buf: *mut c_char,
    bufsize: usize,
    fp: UfuncHandle,
) -> c_int {
    if fp.is_null() || buf.is_null() || bufsize == 0 {
        return -1;
    }

    let name = nvim_ufunc_get_name(fp);
    let namelen = nvim_ufunc_get_namelen(fp);

    if name.is_null() || namelen == 0 {
        return -1;
    }

    // K_SPECIAL byte is 0x80
    let first_byte = *name as u8;
    let buf_slice = std::slice::from_raw_parts_mut(buf.cast::<u8>(), bufsize);

    if first_byte == 0x80 && namelen > 3 {
        // SNR prefix: write "<SNR>" + name+3
        let name_bytes = std::slice::from_raw_parts(name.cast::<u8>(), namelen);
        let rest = &name_bytes[3..];
        let prefix = b"<SNR>";
        let copy_prefix = prefix.len().min(bufsize.saturating_sub(1));
        buf_slice[..copy_prefix].copy_from_slice(&prefix[..copy_prefix]);
        if copy_prefix < bufsize - 1 {
            let copy_rest = rest.len().min(bufsize - 1 - copy_prefix);
            buf_slice[copy_prefix..copy_prefix + copy_rest].copy_from_slice(&rest[..copy_rest]);
            buf_slice[copy_prefix + copy_rest] = 0;
            (copy_prefix + copy_rest) as c_int
        } else {
            buf_slice[copy_prefix] = 0;
            copy_prefix as c_int
        }
    } else {
        // Plain name copy
        let name_bytes = std::slice::from_raw_parts(name.cast::<u8>(), namelen);
        let copy_len = namelen.min(bufsize - 1);
        buf_slice[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
        buf_slice[copy_len] = 0;
        copy_len as c_int
    }
}

// =============================================================================
// rs_function_list_modified
// =============================================================================

/// Return 1 if the function hash table was modified since `prev_ht_changed`,
/// also emitting an error message. Returns 0 if not modified.
///
/// # Safety
/// Accesses global C state.
#[no_mangle]
pub unsafe extern "C" fn rs_function_list_modified(prev_ht_changed: c_int) -> c_int {
    let current = nvim_func_ht_changed();
    if prev_ht_changed != current {
        nvim_emsg_function_list_modified();
        return 1;
    }
    0
}

// =============================================================================
// rs_list_func_head
// =============================================================================

/// List the function header: "function name(arg1, arg2) abort range dict closure"
/// Returns 0 (OK) or 1 (FAIL).
///
/// # Safety
/// `fp` must be a valid non-null ufunc_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_list_func_head(fp: UfuncHandle, indent: c_int, force: c_int) -> c_int {
    let prev_ht_changed = nvim_func_ht_changed();

    msg_start();

    // A callback at the more prompt may have deleted the function
    if rs_function_list_modified(prev_ht_changed) != 0 {
        return 1; // FAIL
    }

    if indent != 0 {
        msg_puts(c"   ".as_ptr());
    }
    if force != 0 {
        msg_puts(c"function! ".as_ptr());
    } else {
        msg_puts(c"function ".as_ptr());
    }

    let name_exp = nvim_ufunc_get_name_exp(fp);
    if name_exp.is_null() {
        msg_puts(nvim_ufunc_get_name(fp));
    } else {
        msg_puts(name_exp);
    }

    msg_putchar(c_int::from(b'('));

    let args_len = nvim_ufunc_get_args_len(fp);
    let def_args_len = nvim_ufunc_get_def_args_len(fp);

    for j in 0..args_len {
        if j > 0 {
            msg_puts(c", ".as_ptr());
        }
        let arg = nvim_ufunc_get_arg(fp, j);
        if !arg.is_null() {
            msg_puts(arg);
        }
        // Default argument when j >= args_len - def_args_len
        if j >= args_len - def_args_len {
            let def_idx = j - (args_len - def_args_len);
            msg_puts(c" = ".as_ptr());
            let def_arg = nvim_ufunc_get_def_arg(fp, def_idx);
            if !def_arg.is_null() {
                msg_puts(def_arg);
            }
        }
    }

    if nvim_ufunc_get_varargs(fp) != 0 {
        if args_len > 0 {
            msg_puts(c", ".as_ptr());
        }
        msg_puts(c"...".as_ptr());
    }

    msg_putchar(c_int::from(b')'));

    let flags = nvim_ufunc_get_flags(fp);
    if flags & FC_ABORT != 0 {
        msg_puts(c" abort".as_ptr());
    }
    if flags & FC_RANGE != 0 {
        msg_puts(c" range".as_ptr());
    }
    if flags & FC_DICT != 0 {
        msg_puts(c" dict".as_ptr());
    }
    if flags & FC_CLOSURE != 0 {
        msg_puts(c" closure".as_ptr());
    }

    msg_clr_eos();

    if p_verbose > 0 {
        let sctx = nvim_ufunc_get_script_ctx(fp);
        last_set_msg(sctx);
    }

    0 // OK
}

// =============================================================================
// rs_list_functions (callback-based hash table iteration)
// =============================================================================

/// Context for list_functions_cb
struct ListFunctionsCtx {
    prev_ht_changed: c_int,
    done: bool,
}

unsafe extern "C" fn list_functions_cb(fp: UfuncHandle, ctx_ptr: *mut c_void) {
    let ctx = &mut *ctx_ptr.cast::<ListFunctionsCtx>();
    if ctx.done || got_int != 0 {
        return;
    }

    let name = nvim_ufunc_get_name(fp);
    if name.is_null() {
        return;
    }

    // No pattern: skip refcounted and filtered functions
    if message_filtered(name) == 0 && nvim_ufunc_name_refcount(name) == 0 {
        if rs_list_func_head(fp, 0, 0) != 0 {
            ctx.done = true;
            return;
        }
        if rs_function_list_modified(ctx.prev_ht_changed) != 0 {
            ctx.done = true;
        }
    }
}

/// List all functions (no pattern filter).
///
/// # Safety
/// Accesses global C state.
#[no_mangle]
pub unsafe extern "C" fn rs_list_functions() {
    let prev_ht_changed = nvim_func_ht_changed();
    let mut ctx = ListFunctionsCtx {
        prev_ht_changed,
        done: false,
    };
    nvim_func_ht_foreach(
        list_functions_cb,
        std::ptr::addr_of_mut!(ctx).cast::<c_void>(),
    );
}

// =============================================================================
// rs_list_functions_matching_pat
// =============================================================================

/// Handle `:function /pat`: list functions matching pattern.
/// Returns pointer after the closing '/', or NULL on error.
///
/// # Safety
/// `eap` must be a valid exarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_list_functions_matching_pat(eap: ExargHandle) -> *mut c_char {
    let arg = (*eap).arg;
    // arg+1 to skip the leading '/'
    let p = skip_regexp(arg.add(1), c_int::from(b'/'), 1);

    if (*eap).skip == 0 {
        // Temporarily NUL-terminate the pattern
        let c = *p;
        *p = 0i8;
        // Let C build regmatch_T and call list_functions()
        nvim_list_functions_matching_pat(arg.add(1), p_ic != 0);
        *p = c;
    }

    if *p == b'/' as c_char {
        p.add(1)
    } else {
        p
    }
}

// =============================================================================
// rs_list_one_function
// =============================================================================

/// List a single function.
/// Returns the ufunc pointer (or NULL on failure).
///
/// # Safety
/// `eap` must be valid. `name` must be a valid C string. `p` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_list_one_function(
    eap: ExargHandle,
    name: *const c_char,
    p: *mut c_char,
) -> UfuncHandle {
    let after_ws = skipwhite(p);
    if ends_excmd(c_int::from(*after_ws)) == 0 {
        nvim_emsg_trailing_arg(p);
        return std::ptr::null_mut();
    }

    let next = check_nextcmd(p);
    (*eap).nextcmd = next;

    if !next.is_null() {
        *p = 0i8; // NUL terminate before nextcmd
    }

    let skip = (*eap).skip;
    if skip != 0 || got_int != 0 {
        return std::ptr::null_mut();
    }

    let fp = find_func(name);
    if fp.is_null() {
        nvim_emsg_undefined_function(name);
        return std::ptr::null_mut();
    }

    let prev_ht_changed = nvim_func_ht_changed();
    let forceit = nvim_eap_get_forceit(eap);

    // list_func_head: indent = !forceit, force = forceit
    let indent = i32::from(forceit == 0);
    if rs_list_func_head(fp, indent, forceit) != 0 {
        return fp;
    }

    let lines_len = nvim_ufunc_get_lines_len(fp);
    let mut j = 0;
    while j < lines_len && got_int == 0 {
        let line = nvim_ufunc_get_funcline(fp, j);
        if line.is_null() {
            j += 1;
            continue;
        }
        msg_putchar(c_int::from(b'\n'));
        if forceit == 0 {
            msg_outnum(j + 1);
            if j < 9 {
                msg_putchar(c_int::from(b' '));
            }
            if j < 99 {
                msg_putchar(c_int::from(b' '));
            }
            if rs_function_list_modified(prev_ht_changed) != 0 {
                break;
            }
        }
        msg_prt_line(line, 0);
        line_breakcheck();
        j += 1;
    }

    if got_int == 0 {
        msg_putchar(c_int::from(b'\n'));
        if rs_function_list_modified(prev_ht_changed) == 0 {
            if forceit != 0 {
                msg_puts(c"endfunction".as_ptr());
            } else {
                msg_puts(c"   endfunction".as_ptr());
            }
        }
    }

    fp
}
