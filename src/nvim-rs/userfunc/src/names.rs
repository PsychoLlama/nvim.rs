//! Function name translation and existence checks for VimL.
//!
//! Migrated from `src/nvim/eval/userfunc.c` Phase 2.

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_char, c_int, c_void};

// K_SPECIAL / KS_EXTRA / KE_SNR constants (keycodes.h)
const K_SPECIAL: u8 = 0x80;
const KS_EXTRA: u8 = 253;
const KE_SNR: u8 = 82;

// AUTOLOAD_CHAR is '#'
const AUTOLOAD_CHAR: u8 = b'#';

// FLEN_FIXED — must match the C define
const FLEN_FIXED: usize = 40;

// FnameTransError values (must match C enum)
const FCERR_SCRIPT: c_int = 4;

extern "C" {
    fn mb_strnicmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn find_func(name: *const c_char) -> *mut c_void;
    fn find_internal_func(name: *const c_char) -> *const c_void;
    fn trans_function_name(
        pp: *mut *mut c_char,
        skip: c_int,
        flags: c_int,
        fudi: *mut c_void,
        partial: *mut c_void,
    ) -> *mut c_char;
    fn getdigits(pp: *mut *mut c_char, strict: c_int, def: i64) -> i64;
    fn xmalloc(size: usize) -> *mut c_void;
    fn xmemdupz(src: *const c_void, len: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn nvim_get_current_sctx_sid() -> c_int;
    fn nvim_script_id_valid(sid: c_int) -> c_int;
    fn nvim_emsg_usingsid();
}

// =============================================================================
// eval_fname_script
// =============================================================================

/// Returns 5 if `p` starts with `<SID>` or `<SNR>` (case-insensitive).
/// Returns 2 if `p` starts with `s:`.
/// Returns 0 otherwise.
///
/// # Safety
/// `p` must be a valid NUL-terminated C string.
#[unsafe(export_name = "eval_fname_script")]
pub unsafe extern "C" fn rs_eval_fname_script(p: *const c_char) -> c_int {
    if p.is_null() {
        return 0;
    }
    let bytes = p.cast::<u8>();
    // Safety: caller guarantees valid C string
    if unsafe { *bytes } == b'<' {
        // Check for <SID> or <SNR>
        let sid = c"SID>".as_ptr();
        let snr = c"SNR>".as_ptr();
        if unsafe { mb_strnicmp(p.add(1), sid, 4) == 0 || mb_strnicmp(p.add(1), snr, 4) == 0 } {
            return 5;
        }
    }
    if unsafe { *bytes == b's' && *bytes.add(1) == b':' } {
        return 2;
    }
    0
}

// =============================================================================
// eval_fname_sid (inlined into fname_trans_sid; also exported for C callers)
// =============================================================================

/// Returns true if `name` starts with `<SID>` or `s:`.
/// Only valid for names already checked by `eval_fname_script` returning non-zero.
///
/// # Safety
/// `name` must be valid NUL-terminated C string.
#[inline]
unsafe fn eval_fname_sid_inner(name: *const u8) -> bool {
    unsafe { *name == b's' || (*name.add(2)).eq_ignore_ascii_case(&b'I') }
}

// =============================================================================
// fname_trans_sid
// =============================================================================

/// Translate `<SID>` / `s:` / `<SNR>` prefix to K_SNR form.
///
/// Fills `fname_buf` (of at least FLEN_FIXED+1 bytes) or allocates.
/// Sets `*tofree` to the allocated pointer when allocation happened.
/// Sets `*error` to FCERR_SCRIPT on error.
///
/// # Safety
/// All pointers must be valid and non-null; `fname_buf` must have ≥ FLEN_FIXED+1 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_fname_trans_sid(
    name: *const c_char,
    fname_buf: *mut c_char,
    tofree: *mut *mut c_char,
    error: *mut c_int,
) -> *mut c_char {
    let name_bytes = name.cast::<u8>();
    let script_offset = unsafe { rs_eval_fname_script(name) } as usize;
    if script_offset == 0 {
        // No prefix — return as-is
        return name.cast_mut();
    }
    let script_name_ptr = unsafe { name.add(script_offset) };

    // Write K_SPECIAL KS_EXTRA KE_SNR header
    let buf_bytes = fname_buf.cast::<u8>();
    unsafe {
        *buf_bytes = K_SPECIAL;
        *buf_bytes.add(1) = KS_EXTRA;
        *buf_bytes.add(2) = KE_SNR;
    }
    let mut fname_buflen: usize = 3;

    if unsafe { !eval_fname_sid_inner(name_bytes) } {
        // <SNR> prefix: just the header, NUL-terminate
        unsafe { *buf_bytes.add(fname_buflen) = 0 };
    } else {
        // <SID> or s: prefix: append script ID
        let sid = unsafe { nvim_get_current_sctx_sid() };
        if unsafe { nvim_script_id_valid(sid) } == 0 {
            unsafe { *error = FCERR_SCRIPT };
        } else {
            // snprintf into fname_buf after the header
            let remaining = FLEN_FIXED + 1 - fname_buflen;
            let written = unsafe {
                libc_snprintf(buf_bytes.add(fname_buflen).cast::<c_char>(), remaining, sid)
            };
            fname_buflen += written;
        }
    }

    // Compute total length: header + suffix
    let suffix_len = unsafe { strlen_c(script_name_ptr) };
    let fnamelen = fname_buflen + suffix_len;

    let fname: *mut c_char;
    if fnamelen < FLEN_FIXED {
        // Fits in buffer
        unsafe {
            std::ptr::copy_nonoverlapping(
                script_name_ptr.cast::<u8>(),
                buf_bytes.add(fname_buflen),
                suffix_len + 1,
            );
        }
        fname = fname_buf;
    } else {
        // Allocate
        let alloc = unsafe { xmalloc(fnamelen + 1) }.cast::<u8>();
        unsafe {
            std::ptr::copy_nonoverlapping(buf_bytes, alloc, fname_buflen);
            std::ptr::copy_nonoverlapping(
                script_name_ptr.cast::<u8>(),
                alloc.add(fname_buflen),
                suffix_len + 1,
            );
            *tofree = alloc.cast::<c_char>();
        }
        fname = alloc.cast::<c_char>();
    }
    fname
}

// Helper: strlen for a C string
unsafe fn strlen_c(s: *const c_char) -> usize {
    let mut len = 0usize;
    while unsafe { *s.add(len) != 0 } {
        len += 1;
    }
    len
}

// Helper: snprintf "<sid>_" into buf, returns bytes written (without NUL)
unsafe fn libc_snprintf(buf: *mut c_char, size: usize, sid: c_int) -> usize {
    // Use write! to format into a stack buffer, then copy
    let mut tmp = [0u8; 32];
    let s = format!("{sid}_");
    let bytes = s.as_bytes();
    let to_copy = bytes.len().min(size.saturating_sub(1));
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), tmp.as_mut_ptr(), to_copy);
        std::ptr::copy_nonoverlapping(tmp.as_ptr(), buf.cast::<u8>(), to_copy);
        *buf.add(to_copy).cast::<u8>() = 0;
    }
    to_copy
}

// =============================================================================
// func_name_refcount
// =============================================================================

/// Returns true if the function name is refcounted (numbered or lambda).
///
/// # Safety
/// `name` must be a valid non-null NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_func_name_refcount(name: *const c_char) -> c_int {
    let first = unsafe { *name.cast::<u8>() };
    c_int::from(first.is_ascii_digit() || first == b'<')
}

// =============================================================================
// builtin_function
// =============================================================================

/// Returns true if `name` looks like a builtin function name.
///
/// Builtins start with a lowercase ASCII letter, don't have `:` as second char,
/// and don't contain `AUTOLOAD_CHAR` (`#`).
///
/// # Safety
/// `name` must be valid; if `len >= 0`, points to `len` bytes; if `len == -1`, NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn rs_builtin_function(name: *const c_char, len: c_int) -> c_int {
    let bytes = name.cast::<u8>();
    let first = unsafe { *bytes };
    if !first.is_ascii_lowercase() || unsafe { *bytes.add(1) == b':' } {
        return 0;
    }
    // Check for AUTOLOAD_CHAR
    let found_hash = if len == -1 {
        // NUL-terminated scan
        let mut i = 0usize;
        loop {
            let c = unsafe { *bytes.add(i) };
            if c == 0 {
                break false;
            }
            if c == AUTOLOAD_CHAR {
                break true;
            }
            i += 1;
        }
    } else {
        let slice = unsafe { std::slice::from_raw_parts(bytes, len as usize) };
        slice.contains(&AUTOLOAD_CHAR)
    };
    c_int::from(!found_hash)
}

// =============================================================================
// translated_function_exists
// =============================================================================

/// Returns true if the (already-translated) function name exists.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string.
#[unsafe(export_name = "translated_function_exists")]
pub unsafe extern "C" fn rs_translated_function_exists(name: *const c_char) -> c_int {
    if unsafe { rs_builtin_function(name, -1) } != 0 {
        c_int::from(!unsafe { find_internal_func(name) }.is_null())
    } else {
        c_int::from(!unsafe { find_func(name) }.is_null())
    }
}

// =============================================================================
// function_exists
// =============================================================================

// TFN_* flags (must match C defines in userfunc.h)
const TFN_INT: c_int = 1;
const TFN_QUIET: c_int = 2;
const TFN_NO_AUTOLOAD: c_int = 4;
const TFN_NO_DEREF: c_int = 8;

/// Returns true if the named function exists.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string.
#[unsafe(export_name = "function_exists")]
pub unsafe extern "C" fn rs_function_exists(name: *const c_char, no_deref: c_int) -> c_int {
    let mut nm = name.cast_mut();
    let mut flag = TFN_INT | TFN_QUIET | TFN_NO_AUTOLOAD;
    if no_deref != 0 {
        flag |= TFN_NO_DEREF;
    }
    let p = unsafe {
        trans_function_name(
            std::ptr::addr_of_mut!(nm),
            0,
            flag,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    let nm_after = unsafe { skipwhite(nm) };
    let result = if p.is_null() {
        0
    } else {
        let c = unsafe { *nm_after };
        if c == 0 || c == b'(' as c_char {
            unsafe { rs_translated_function_exists(p) }
        } else {
            0
        }
    };
    if !p.is_null() {
        unsafe { xfree(p.cast::<c_void>()) };
    }
    result
}

// =============================================================================
// get_scriptlocal_funcname
// =============================================================================

/// Expand `s:` or `<SID>` prefix to `<SNR>nr_` form.
/// Returns allocated string the caller must free, or NULL.
///
/// # Safety
/// `funcname` must be a valid NUL-terminated C string or NULL.
#[unsafe(export_name = "get_scriptlocal_funcname")]
pub unsafe extern "C" fn rs_get_scriptlocal_funcname(funcname: *const c_char) -> *mut c_char {
    if funcname.is_null() {
        return std::ptr::null_mut();
    }
    let bytes = funcname.cast::<u8>();
    // Check for "s:" or "<SID>" prefix
    let has_s_colon = unsafe { *bytes == b's' && *bytes.add(1) == b':' };
    let has_sid = unsafe {
        *bytes == b'<'
            && *bytes.add(1) == b'S'
            && *bytes.add(2) == b'I'
            && *bytes.add(3) == b'D'
            && *bytes.add(4) == b'>'
    };
    if !has_s_colon && !has_sid {
        return std::ptr::null_mut();
    }

    let sid = unsafe { nvim_get_current_sctx_sid() };
    if unsafe { nvim_script_id_valid(sid) } == 0 {
        unsafe { nvim_emsg_usingsid() };
        return std::ptr::null_mut();
    }

    // Format "<SNR>sid_"
    let prefix = format!("<SNR>{sid}_");
    let off: usize = if has_s_colon { 2 } else { 5 };
    let suffix_len = unsafe { strlen_c(funcname.add(off)) };
    let total = prefix.len() + suffix_len + 1;

    let newname = unsafe { xmalloc(total) }.cast::<u8>();
    let pbytes = prefix.as_bytes();
    unsafe {
        std::ptr::copy_nonoverlapping(pbytes.as_ptr(), newname, pbytes.len());
        std::ptr::copy_nonoverlapping(
            funcname.add(off).cast::<u8>(),
            newname.add(pbytes.len()),
            suffix_len + 1,
        );
    }
    newname.cast::<c_char>()
}

// =============================================================================
// save_function_name
// =============================================================================

/// Translate or pass-through lambda names.
/// Returns allocated function name (caller frees), advances `*name`.
///
/// # Safety
/// `name` must point to a valid mutable C string pointer; `fudi` may be null.
#[unsafe(export_name = "save_function_name")]
pub unsafe extern "C" fn rs_save_function_name(
    name: *mut *mut c_char,
    skip: c_int,
    flags: c_int,
    fudi: *mut c_void,
) -> *mut c_char {
    let p: *mut c_char = unsafe { *name };
    let lambda_prefix = b"<lambda>";

    // Check if starts with "<lambda>" (8 bytes)
    let is_lambda = unsafe {
        let pb = p.cast::<u8>();
        *pb == b'<'
            && *pb.add(1) == b'l'
            && *pb.add(2) == b'a'
            && *pb.add(3) == b'm'
            && *pb.add(4) == b'b'
            && *pb.add(5) == b'd'
            && *pb.add(6) == b'a'
            && *pb.add(7) == b'>'
    };
    let _ = lambda_prefix; // suppress unused warning

    if is_lambda {
        // Advance past "<lambda>" and digits
        let mut after = unsafe { p.add(8) };
        unsafe { getdigits(std::ptr::addr_of_mut!(after), 0, 0) };
        let len = unsafe { after.offset_from(p) } as usize;
        let dup = unsafe { xmemdupz(p.cast::<c_void>(), len) };
        if !fudi.is_null() {
            // CLEAR_POINTER(fudi) — zero the funcdict_T
            // funcdict_T is 3 pointers = 24 bytes on 64-bit
            unsafe { std::ptr::write_bytes(fudi.cast::<u8>(), 0, 24) };
        }
        unsafe { *name = after };
        dup.cast::<c_char>()
    } else {
        unsafe { trans_function_name(name, skip, flags, fudi, std::ptr::null_mut()) }
    }
}
