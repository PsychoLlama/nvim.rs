//! Function name translation and existence checks for VimL.
//!
//! Migrated from `src/nvim/eval/userfunc.c` Phase 2.
//! Wave 2 Phase 3: `trans_function_name` migrated here.

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_ptr_alignment)]

use std::ffi::{c_char, c_int, c_void};

use nvim_eval::typval::TypvalT;
use nvim_eval_exec::lval::LvalT;

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

// VAR_FUNC = 3, VAR_PARTIAL = 9 (from typval_defs.h)
const VAR_FUNC: c_int = 3;
const VAR_PARTIAL: c_int = 9;

// TFN / GLV flags
const TFN_NO_AUTOLOAD: c_int = 4;
const TFN_INT_FLAG: c_int = 1;
const TFN_QUIET_FLAG: c_int = 2;
const TFN_NO_DEREF: c_int = 8;
const GLV_READ_ONLY: c_int = 16;
const FNE_INCL_BR: c_int = 1;
const FNE_CHECK_START: c_int = 2;

// rs_is_luafunc, rs_partial_name, deref_func_name: const vs mut ptr; same ABI.
#[allow(clashing_extern_declarations)]
extern "C" {
    fn mb_strnicmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn find_func(name: *const c_char) -> *mut c_void;
    fn find_internal_func(name: *const c_char) -> *const c_void;
    fn getdigits(pp: *mut *mut c_char, strict: c_int, def: i64) -> i64;
    fn xmalloc(size: usize) -> *mut c_void;
    fn xmallocz(size: usize) -> *mut c_void;
    fn xmemdupz(src: *const c_void, len: usize) -> *mut c_void;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn nvim_get_current_sctx_sid() -> c_int;
    fn nvim_script_id_valid(sid: c_int) -> c_int;
    fn nvim_emsg_usingsid();

    // get_lval / clear_lval (Rust symbols, eval_exec::lval)
    fn get_lval(
        name: *mut c_char,
        rettv: *mut c_void,
        lp: *mut LvalT,
        unlet: bool,
        skip: bool,
        flags: c_int,
        fne_flags: c_int,
    ) -> *mut c_char;
    fn clear_lval(lp: *mut LvalT);

    // Helper Rust functions (from eval crate, same binary)
    fn rs_get_id_len(arg: *mut *const c_char) -> c_int;
    fn rs_find_name_end(
        arg: *const c_char,
        expr_start: *mut *const c_char,
        expr_end: *mut *const c_char,
        flags: c_int,
    ) -> *const c_char;
    fn rs_is_luafunc(pt: *const c_void) -> bool;
    fn rs_check_luafunc_name(str: *const c_char, paren: bool) -> c_int;
    fn rs_partial_name(pt: *const c_void) -> *mut c_char;
    fn deref_func_name(
        name: *const c_char,
        lenp: *mut c_int,
        partialp: *mut *mut c_void,
        no_autoload: bool,
        found_var: *mut bool,
    ) -> *mut c_char;

    // Error messages (Wave 2 Phase 3 accessors added to userfunc.c)
    fn nvim_emsg_e129_funcname_required();
    fn nvim_semsg_e128_func_start_capital(start: *const c_char);
    fn nvim_semsg_e884_func_no_colon(start: *const c_char);
    fn nvim_semsg_invexpr2_vlua();
    fn nvim_emsg_funcref();
    fn nvim_semsg_invarg2(arg: *const c_char);
}

// aborting() — declared separately to allow clashing with other modules
#[allow(clashing_extern_declarations)]
extern "C" {
    fn aborting() -> bool;
}

// =============================================================================
// trans_function_name (Wave 2 Phase 3: migrated from userfunc.c)
// =============================================================================

/// Translate a function name, returning an allocated string or NULL on failure.
///
/// Handles `<SID>`, `<SNR>`, `s:`, `g:`, dict references, Funcref/Partial
/// dereffing, and Lua subscript paths.
#[unsafe(export_name = "trans_function_name")]
pub unsafe extern "C" fn rs_trans_function_name(
    pp: *mut *mut c_char,
    skip_int: c_int, // bool in C, passed as int for ABI compat with existing Rust externs
    flags: c_int,
    fdp: *mut c_void,          // funcdict_T*
    partial: *mut *mut c_void, // partial_T**
) -> *mut c_char {
    let skip = skip_int != 0;
    let mut name: *mut c_char = std::ptr::null_mut();

    if !fdp.is_null() {
        // CLEAR_POINTER(fdp) — zero funcdict_T (3 pointers = 24 bytes)
        unsafe { std::ptr::write_bytes(fdp.cast::<u8>(), 0, 24) };
    }

    let start: *const c_char = unsafe { *pp };

    // Check for hard-coded <SNR>: K_SPECIAL KS_EXTRA KE_SNR sequence.
    let b0 = unsafe { *(start.cast::<u8>()) };
    let b1 = unsafe { *(start.cast::<u8>().add(1)) };
    let b2 = unsafe { *(start.cast::<u8>()).add(2) };
    if b0 == K_SPECIAL && b1 == KS_EXTRA && b2 == KE_SNR {
        unsafe { *pp = (*pp).add(3) };
        let mut pp_const: *const c_char = unsafe { *pp };
        let id_len = unsafe { rs_get_id_len(std::ptr::addr_of_mut!(pp_const)) };
        unsafe { *pp = pp_const.cast_mut() };
        let len = id_len + 3;
        return unsafe { xmemdupz(start.cast::<c_void>(), len as usize).cast::<c_char>() };
    }

    // Check for <SID>/<SNR>/s: prefix.
    let lead = unsafe { rs_eval_fname_script(start) };
    let start_after_lead: *const c_char = if lead > 2 {
        unsafe { start.add(lead as usize) }
    } else {
        start
    };

    // Note: TFN_ flags use same values as GLV_ flags.
    let fne_flags = if lead > 2 { 0 } else { FNE_CHECK_START };
    let mut lv = LvalT {
        ll_name: std::ptr::null(),
        ll_name_len: 0,
        ll_exp_name: std::ptr::null_mut(),
        ll_tv: std::ptr::null_mut(),
        ll_li: std::ptr::null_mut(),
        ll_list: std::ptr::null_mut(),
        ll_range: false,
        ll_empty2: false,
        ll_n1: 0,
        ll_n2: 0,
        ll_dict: std::ptr::null_mut(),
        ll_di: std::ptr::null_mut(),
        ll_newkey: std::ptr::null_mut(),
        ll_blob: std::ptr::null_mut(),
    };

    let end: *mut c_char = unsafe {
        get_lval(
            start_after_lead.cast_mut(),
            std::ptr::null_mut(),
            std::ptr::addr_of_mut!(lv),
            false,
            skip,
            flags | GLV_READ_ONLY,
            fne_flags,
        )
    };

    if end == start_after_lead.cast_mut() {
        if !skip {
            unsafe { nvim_emsg_e129_funcname_required() };
        }
        unsafe { clear_lval(std::ptr::addr_of_mut!(lv)) };
        return std::ptr::null_mut();
    }

    if end.is_null() || (!lv.ll_tv.is_null() && (lead > 2 || lv.ll_range)) {
        // Invalid expression unless aborting.
        let is_aborting = unsafe { aborting() };
        if is_aborting {
            unsafe {
                *pp = rs_find_name_end(
                    start,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    FNE_INCL_BR,
                )
                .cast_mut();
            };
        } else if !end.is_null() {
            unsafe { nvim_semsg_invarg2(start_after_lead) };
        }
        unsafe { clear_lval(std::ptr::addr_of_mut!(lv)) };
        return std::ptr::null_mut();
    }

    if !lv.ll_tv.is_null() {
        // Dict/lval case
        if !fdp.is_null() {
            // fdp->fd_dict = lv.ll_dict  (offset 0)
            // fdp->fd_newkey = lv.ll_newkey  (offset 8)
            // fdp->fd_di = lv.ll_di  (offset 16)
            let fdp_dict = fdp.cast::<*mut c_void>();
            unsafe { *fdp_dict = lv.ll_dict };
            let fdp_newkey = unsafe { fdp.cast::<u8>().add(8).cast::<*mut c_char>() };
            unsafe { *fdp_newkey = lv.ll_newkey };
            lv.ll_newkey = std::ptr::null_mut(); // transferred ownership
            let fdp_di = unsafe { fdp.cast::<u8>().add(16).cast::<*mut c_void>() };
            unsafe { *fdp_di = lv.ll_di };
        }

        let tv = lv.ll_tv.cast::<TypvalT>();
        let v_type = unsafe { (*tv).v_type };
        if v_type == VAR_FUNC {
            let v_string = unsafe { (*tv).vval.v_string };
            if !v_string.is_null() {
                name = unsafe { xstrdup(v_string) };
                unsafe { *pp = end };
            }
        } else if v_type == VAR_PARTIAL {
            let v_partial = unsafe { (*tv).vval.v_partial };
            if !v_partial.is_null() {
                if unsafe { rs_is_luafunc(v_partial) } && unsafe { *end == b'.' as c_char } {
                    let len = unsafe { rs_check_luafunc_name(end.add(1), true) };
                    if len == 0 {
                        unsafe { nvim_semsg_invexpr2_vlua() };
                        unsafe { clear_lval(std::ptr::addr_of_mut!(lv)) };
                        return std::ptr::null_mut();
                    }
                    name = unsafe { xmallocz(len as usize).cast::<c_char>() };
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            end.add(1).cast::<u8>(),
                            name.cast::<u8>(),
                            len as usize,
                        );
                    };
                    unsafe { *pp = end.add(1 + len as usize) };
                } else {
                    name = unsafe { xstrdup(rs_partial_name(v_partial)) };
                    unsafe { *pp = end };
                }
                if !partial.is_null() {
                    unsafe { *partial = v_partial };
                }
            }
        } else {
            // Not VAR_FUNC or VAR_PARTIAL
            // C: if (!skip && !(flags & TFN_QUIET) && (fdp==NULL||ll_dict==NULL||fd_newkey==NULL))
            //        emsg(e_funcref)
            //    else
            //        *pp = end
            let should_error = !skip
                && (flags & TFN_QUIET_FLAG == 0)
                && (fdp.is_null() || {
                    let fd_dict = unsafe { *(fdp.cast::<*mut c_void>()) };
                    let fd_newkey = unsafe { *(fdp.cast::<u8>().add(8).cast::<*mut c_char>()) };
                    fd_dict.is_null() || fd_newkey.is_null()
                });
            if should_error {
                unsafe { nvim_emsg_funcref() };
            } else {
                unsafe { *pp = end };
            }
            name = std::ptr::null_mut();
        }
        unsafe { clear_lval(std::ptr::addr_of_mut!(lv)) };
        return name;
    }

    if lv.ll_name.is_null() {
        // Error found, advance past function name.
        unsafe { *pp = end };
        unsafe { clear_lval(std::ptr::addr_of_mut!(lv)) };
        return std::ptr::null_mut();
    }

    // Check if the name is a Funcref; if so, use its value.
    if !lv.ll_exp_name.is_null() {
        let mut len = unsafe { strlen_c(lv.ll_exp_name) } as c_int;
        let deref = unsafe {
            deref_func_name(
                lv.ll_exp_name,
                std::ptr::addr_of_mut!(len),
                if partial.is_null() {
                    std::ptr::null_mut()
                } else {
                    partial
                },
                flags & TFN_NO_AUTOLOAD != 0,
                std::ptr::null_mut(),
            )
        };
        if deref != lv.ll_exp_name {
            name = deref;
        }
    } else if flags & TFN_NO_DEREF == 0 {
        let mut len = unsafe { end.offset_from(*pp) } as c_int;
        let deref = unsafe {
            deref_func_name(
                *pp,
                std::ptr::addr_of_mut!(len),
                if partial.is_null() {
                    std::ptr::null_mut()
                } else {
                    partial
                },
                flags & TFN_NO_AUTOLOAD != 0,
                std::ptr::null_mut(),
            )
        };
        if deref != *pp {
            name = deref;
        }
    }

    if !name.is_null() {
        name = unsafe { xstrdup(name) };
        unsafe { *pp = end };
        // If name starts with "<SNR>" convert to byte sequence
        let nbytes = name.cast::<u8>();
        if unsafe {
            *nbytes == b'<'
                && *nbytes.add(1) == b'S'
                && *nbytes.add(2) == b'N'
                && *nbytes.add(3) == b'R'
                && *nbytes.add(4) == b'>'
        } {
            unsafe {
                *nbytes = K_SPECIAL;
                *nbytes.add(1) = KS_EXTRA;
                *nbytes.add(2) = KE_SNR;
                // memmove name+3 <- name+5
                let suffix = name.add(5);
                let suffix_len = strlen_c(suffix) + 1;
                std::ptr::copy(suffix.cast::<u8>(), nbytes.add(3), suffix_len);
            }
        }
        unsafe { clear_lval(std::ptr::addr_of_mut!(lv)) };
        return name;
    }

    // Compute name from lv.ll_exp_name or lv.ll_name.
    let mut ll_name = lv.ll_name;
    let mut ll_name_len = lv.ll_name_len;
    let mut lead = lead;
    let mut len: c_int;

    if lv.ll_exp_name.is_null() {
        // Skip over "s:" and "g:"
        if lead == 2
            || (unsafe { *ll_name.cast::<u8>() == b'g' && *ll_name.cast::<u8>().add(1) == b':' })
        {
            ll_name = unsafe { ll_name.add(2) };
            ll_name_len -= 2;
        }
        len = unsafe { end.offset_from(ll_name.cast_mut()) } as c_int;
    } else {
        len = unsafe { strlen_c(lv.ll_exp_name) } as c_int;
        if lead <= 2
            && lv.ll_name == lv.ll_exp_name.cast_const()
            && ll_name_len >= 2
            && unsafe { *ll_name.cast::<u8>() == b's' && *ll_name.cast::<u8>().add(1) == b':' }
        {
            // Remove "s:" prefix when it was already there or expanded to it
            ll_name = unsafe { ll_name.add(2) };
            ll_name_len -= 2;
            len -= 2;
            lead = 2;
        }
    }

    let mut sid_buflen: usize = 0;
    let mut sid_buf = [0u8; 20];

    // Copy function name: accept <SID>name(), translate to <SNR>123_name().
    if skip {
        lead = 0; // do nothing with prefix
    } else if lead > 0 {
        lead = 3;
        // Check if it's "<SID>" or "s:" (not "<SNR>")
        let needs_sid = (!lv.ll_exp_name.is_null()
            && unsafe { eval_fname_sid_inner(lv.ll_exp_name.cast::<u8>()) })
            || unsafe { eval_fname_sid_inner((*pp).cast::<u8>()) };
        if needs_sid {
            let sid = unsafe { nvim_get_current_sctx_sid() };
            if sid <= 0 {
                unsafe { nvim_emsg_usingsid() };
                unsafe { clear_lval(std::ptr::addr_of_mut!(lv)) };
                return std::ptr::null_mut();
            }
            let s = format!("{sid}_");
            let bytes = s.as_bytes();
            let copy_len = bytes.len().min(sid_buf.len() - 1);
            sid_buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
            sid_buf[copy_len] = 0;
            sid_buflen = copy_len;
            lead += copy_len as c_int;
        }
    } else if (flags & TFN_INT_FLAG == 0)
        && unsafe { rs_builtin_function(ll_name, ll_name_len as c_int) } != 0
    {
        unsafe { nvim_semsg_e128_func_start_capital(start) };
        unsafe { clear_lval(std::ptr::addr_of_mut!(lv)) };
        return std::ptr::null_mut();
    }

    // Check for colon in name (E884)
    if !skip && (flags & TFN_QUIET_FLAG == 0) && (flags & TFN_NO_DEREF == 0) {
        let name_slice = unsafe { std::slice::from_raw_parts(ll_name.cast::<u8>(), ll_name_len) };
        let cp = name_slice.iter().rposition(|&b| b == b':');
        if let Some(pos) = cp {
            let cp_ptr = unsafe { ll_name.add(pos) };
            if cp_ptr < end {
                unsafe { nvim_semsg_e884_func_no_colon(start) };
                unsafe { clear_lval(std::ptr::addr_of_mut!(lv)) };
                return std::ptr::null_mut();
            }
        }
    }

    // Allocate and build the name
    let total = len as usize + lead as usize + 1;
    name = unsafe { xmalloc(total).cast::<c_char>() };
    if !skip && lead > 0 {
        let nbytes = name.cast::<u8>();
        unsafe {
            *nbytes = K_SPECIAL;
            *nbytes.add(1) = KS_EXTRA;
            *nbytes.add(2) = KE_SNR;
            if sid_buflen > 0 {
                std::ptr::copy_nonoverlapping(sid_buf.as_ptr(), nbytes.add(3), sid_buflen);
            }
        }
    }
    unsafe {
        std::ptr::copy(
            ll_name.cast::<u8>(),
            name.cast::<u8>().add(lead as usize),
            len as usize,
        );
        *name.add(lead as usize + len as usize) = 0;
    }
    unsafe { *pp = end };

    unsafe { clear_lval(std::ptr::addr_of_mut!(lv)) };
    name
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

/// Returns true if the named function exists.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string.
#[unsafe(export_name = "function_exists")]
pub unsafe extern "C" fn rs_function_exists(name: *const c_char, no_deref: c_int) -> c_int {
    let mut nm = name.cast_mut();
    let mut flag = TFN_INT_FLAG | TFN_QUIET_FLAG | TFN_NO_AUTOLOAD;
    if no_deref != 0 {
        flag |= TFN_NO_DEREF;
    }
    let p = unsafe {
        rs_trans_function_name(
            std::ptr::addr_of_mut!(nm),
            0, // skip = false
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
        unsafe { rs_trans_function_name(name, skip, flags, fudi, std::ptr::null_mut()) }
    }
}
