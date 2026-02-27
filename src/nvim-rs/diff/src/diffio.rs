//! Diff I/O pipeline: buffer serialization, xdiff invocation, external diff, verification.
//!
//! This module provides Rust implementations for the diff I/O pipeline,
//! replacing the C functions: diff_write_buffer, clear_diffin, clear_diffout,
//! diff_file_internal, diff_file, diff_write, and check_external_diff.

#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::c_char;
use std::os::raw::c_int;

use crate::buffer::{BufHandle, DiffioHandle};

// ============================================================================
// Result constants
// ============================================================================

const OK: c_int = 1;
const FAIL: c_int = 0;

// ============================================================================
// Diff flags (must match C #define values in diff_shim.c)
// ============================================================================

const DIFF_ICASE: c_int = 0x004;
const DIFF_IWHITE: c_int = 0x008;
const DIFF_IWHITEALL: c_int = 0x010;
const DIFF_IWHITEEOL: c_int = 0x020;
const DIFF_IBLANK: c_int = 0x002;

// ============================================================================
// ML flags (must match C #define values in memline_defs.h)
// ============================================================================

const ML_EMPTY: c_int = 0x02; // empty buffer

// NL (0x0a) is Neovim's internal representation of NUL (0x00) in lines.
const NL: u8 = b'\n';
const NUL: u8 = 0;

// ============================================================================
// XDF flags (must match xdiff/xdiff.h)
// ============================================================================

const XDF_IGNORE_WHITESPACE_CHANGE: c_ulong = 1 << 2;
const XDF_IGNORE_WHITESPACE: c_ulong = 1 << 1;
const XDF_IGNORE_WHITESPACE_AT_EOL: c_ulong = 1 << 3;
const XDF_IGNORE_BLANK_LINES: c_ulong = 1 << 7;

// ============================================================================
// xdiff struct declarations (must match xdiff/xdiff.h exactly)
// ============================================================================

use libc::c_ulong;

/// Memory-mapped file (xdiff mmfile_t).
/// Must match: `typedef struct s_mmfile { char *ptr; int size; } mmfile_t;`
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct MmFileInternal {
    pub ptr: *mut c_char,
    pub size: c_int,
}

/// Hunk-consume callback type.
type XdlEmitHunkConsumeFuncT =
    Option<unsafe extern "C" fn(c_int, c_int, c_int, c_int, *mut std::ffi::c_void) -> c_int>;

/// xdiff emit callback (xdemitcb_t).
/// `out_hunk` and `out_line` function pointers must be NULL (zeroed) for our usage.
#[repr(C)]
#[derive(Default)]
struct XdemitCb {
    priv_: *mut std::ffi::c_void,
    out_hunk: Option<
        unsafe extern "C" fn(
            *mut std::ffi::c_void,
            libc::c_long,
            libc::c_long,
            libc::c_long,
            libc::c_long,
            *const c_char,
            libc::c_long,
        ) -> c_int,
    >,
    out_line: Option<unsafe extern "C" fn(*mut std::ffi::c_void, *mut MmBufferT, c_int) -> c_int>,
}

/// mmbuffer_t (used in out_line callback signature only)
#[repr(C)]
struct MmBufferT {
    ptr: *mut c_char,
    size: c_int,
}

/// xdiff emit config (xdemitconf_t).
/// Must match: `typedef struct s_xdemitconf { long ctxlen; long interhunkctxlen;
///              unsigned long flags; find_func_t find_func; void *find_func_priv;
///              xdl_emit_hunk_consume_func_t hunk_func; } xdemitconf_t;`
#[repr(C)]
#[derive(Default)]
struct XdemitConf {
    ctxlen: libc::c_long,
    interhunkctxlen: libc::c_long,
    flags: c_ulong,
    find_func: Option<
        unsafe extern "C" fn(
            *const c_char,
            libc::c_long,
            *mut c_char,
            libc::c_long,
            *mut std::ffi::c_void,
        ) -> libc::c_long,
    >,
    find_func_priv: *mut std::ffi::c_void,
    hunk_func: XdlEmitHunkConsumeFuncT,
}

/// xdiff parameter struct (xpparam_t).
/// Note: `#if 0` block in xdiff.h removes ignore_regex fields, leaving only:
///   `unsigned long flags; char **anchors; size_t anchors_nr;`
#[repr(C)]
#[derive(Default)]
struct XpParam {
    flags: c_ulong,
    anchors: *mut *mut c_char,
    anchors_nr: usize,
}

// ============================================================================
// C FFI declarations
// ============================================================================

extern "C" {
    /// Get the line count for a buffer (b_ml.ml_line_count).
    fn nvim_diff_buf_get_ml_line_count(buf: BufHandle) -> i32;

    /// Get the ml_flags for a buffer (b_ml.ml_flags).
    fn nvim_diff_buf_get_ml_flags(buf: BufHandle) -> c_int;

    /// Get the text of line `lnum` in `buf` (ml_get_buf).
    fn nvim_diff_ml_get_buf(buf: BufHandle, lnum: i32) -> *const c_char;

    /// Get the byte length of line `lnum` in `buf` (ml_get_buf_len).
    fn nvim_diff_ml_get_buf_len(buf: BufHandle, lnum: i32) -> c_int;

    /// Allocate memory via xmalloc.
    fn nvim_diff_xmalloc(size: usize) -> *mut c_char;

    // UTF-8 helpers (same as in inline_compute.rs)
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_fold(c: c_int) -> c_int;
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn utf_char2len(c: c_int) -> c_int;
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;

    // Diff flags and algorithm accessors
    fn nvim_diff_get_diff_flags() -> c_int;
    fn nvim_diff_get_algorithm() -> c_int;

    // Diffio mmfile accessors (from diff_shim.c)
    fn nvim_diffio_get_orig_ptr(dio: DiffioHandle) -> *mut c_char;
    fn nvim_diffio_get_orig_size(dio: DiffioHandle) -> c_int;
    fn nvim_diffio_get_new_ptr(dio: DiffioHandle) -> *mut c_char;
    fn nvim_diffio_get_new_size(dio: DiffioHandle) -> c_int;
    fn nvim_diffio_get_dout(dio: DiffioHandle) -> *mut std::ffi::c_void;

    // Error message
    fn nvim_diff_emsg_e960();

    /// rs_xdiff_out -- the Rust xdiff hunk callback (already exported from viml.rs).
    /// We call it directly to avoid needing to access the static C xdiff_out wrapper.
    fn rs_xdiff_out(
        start_a: c_int,
        count_a: c_int,
        start_b: c_int,
        count_b: c_int,
        priv_: *mut std::ffi::c_void,
    ) -> c_int;

    // Phase 3 (diff_file, diff_write) accessors
    fn nvim_diffio_get_orig_fname(dio: DiffioHandle) -> *const c_char;
    fn nvim_diffio_is_internal(dio: DiffioHandle) -> bool;
    fn nvim_diff_eval_diff(orig: *const c_char, new_f: *const c_char, diff: *const c_char);
    fn nvim_diffio_get_diff_fname(dio: DiffioHandle) -> *const c_char;

    // Phase 3b: run_external_shell accessors
    fn nvim_diff_get_p_srr() -> *const c_char;
    fn nvim_diff_env_clear_diff_options();
    fn nvim_diff_call_shell_diff(cmd: *const c_char);
    fn nvim_diff_get_a_works() -> c_int;
    fn nvim_diff_set_a_works(val: c_int);
    fn nvim_diff_xfree(p: *mut std::ffi::c_void);

    /// rs_append_redir -- the Rust append_redir implementation (from ex_cmds crate).
    fn rs_append_redir(buf: *mut c_char, buflen: usize, opt: *const c_char, fname: *const c_char);

    /// xdl_diff -- the xdiff library entry point.
    fn xdl_diff(
        mf1: *const MmFileInternal,
        mf2: *const MmFileInternal,
        xpp: *const XpParam,
        xecfg: *const XdemitConf,
        ecb: *mut XdemitCb,
    ) -> c_int;
}

// ============================================================================
// Phase 1: diff_write_buffer
// ============================================================================

/// Write buffer lines into a contiguous memory block for xdiff consumption.
///
/// Replicates C `diff_write_buffer(buf, m, start, end)`.
///
/// On success returns `OK` and sets `*m_ptr` and `*m_size`.
/// On empty-buffer / empty-range returns `OK` with NULL ptr and size 0.
///
/// # Safety
/// All pointers must be valid. `m_ptr` and `m_size` must be non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_write_buffer(
    buf: BufHandle,
    m_ptr: *mut *mut c_char,
    m_size: *mut c_int,
    start: i32,
    end: i32,
    diff_flags: c_int,
) -> c_int {
    // Determine effective end
    let end = if end < 0 {
        nvim_diff_buf_get_ml_line_count(buf)
    } else {
        end
    };

    // Empty buffer or inverted range: return empty mmfile
    let ml_flags = nvim_diff_buf_get_ml_flags(buf);
    if (ml_flags & ML_EMPTY) != 0 || end < start {
        *m_ptr = std::ptr::null_mut();
        *m_size = 0;
        return OK;
    }

    // First pass: compute total size needed.
    let mut total_len: usize = 0;
    for lnum in start..=end {
        let line_len = nvim_diff_ml_get_buf_len(buf, lnum);
        total_len += (line_len as usize) + 1; // +1 for NL separator
    }

    // Allocate the block.
    let ptr = nvim_diff_xmalloc(total_len);
    *m_ptr = ptr;
    *m_size = total_len as c_int;

    // Second pass: copy line data.
    let mut write_pos: usize = 0;
    for lnum in start..=end {
        let s = nvim_diff_ml_get_buf(buf, lnum);
        let line_len = nvim_diff_ml_get_buf_len(buf, lnum) as usize;

        if (diff_flags & DIFF_ICASE) != 0 {
            // Case-folding path: fold each character using utf_fold.
            let mut src = s;
            let mut remaining = line_len;
            while remaining > 0 {
                let byte = *src.cast::<u8>();
                if byte == NL {
                    // NL represents internal NUL; write NUL (0)
                    *ptr.add(write_pos).cast::<u8>() = NUL;
                    src = src.add(1);
                    write_pos += 1;
                    remaining -= 1;
                } else {
                    // Fold this character.
                    let c = utf_ptr2char(src);
                    let c_len = utf_char2len(c) as usize;
                    let c_folded = utf_fold(c);
                    let orig_len = utfc_ptr2len(src) as usize;

                    let mut cbuf = [0u8; 8]; // MB_MAXBYTES + 1
                    let c_fold_len = utf_char2bytes(c_folded, cbuf.as_mut_ptr().cast()) as usize;

                    if c_fold_len == c_len {
                        // Write folded base character.
                        std::ptr::copy_nonoverlapping(
                            cbuf.as_ptr(),
                            ptr.add(write_pos).cast::<u8>(),
                            c_fold_len,
                        );
                        // Copy remaining composing characters unchanged.
                        if orig_len > c_len {
                            std::ptr::copy_nonoverlapping(
                                src.add(c_len).cast::<u8>(),
                                ptr.add(write_pos + c_fold_len).cast::<u8>(),
                                orig_len - c_len,
                            );
                        }
                    } else {
                        // TODO(Bram): handle byte length difference
                        // One example is Å (3 bytes) and å (2 bytes).
                        // Preserve original bytes unchanged.
                        std::ptr::copy_nonoverlapping(
                            src.cast::<u8>(),
                            ptr.add(write_pos).cast::<u8>(),
                            orig_len,
                        );
                    }

                    src = src.add(orig_len);
                    write_pos += orig_len;
                    remaining = remaining.saturating_sub(orig_len);
                }
            }
        } else {
            // Non-case-folding path: copy bytes and substitute NL -> NUL.
            let src_slice = std::slice::from_raw_parts(s.cast::<u8>(), line_len);
            let dst_slice =
                std::slice::from_raw_parts_mut(ptr.add(write_pos).cast::<u8>(), line_len);
            dst_slice.copy_from_slice(src_slice);
            // Neovim stores NUL bytes as NL (0x0a); xdiff wants real NUL (0x00).
            for b in dst_slice.iter_mut() {
                if *b == NL {
                    *b = NUL;
                }
            }
            write_pos += line_len;
        }

        // Append the NL line separator.
        *ptr.add(write_pos).cast::<u8>() = NL;
        write_pos += 1;
    }

    OK
}

// ============================================================================
// Phase 2: diff_file_internal (xdiff invocation)
// ============================================================================

/// Invoke the xdiff engine on the mmfiles stored in `dio`.
///
/// Replicates C `diff_file_internal(diffio)`.
///
/// # Safety
/// `dio` must be a valid, non-null diffio handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_file_internal(dio: DiffioHandle) -> c_int {
    let diff_flags = nvim_diff_get_diff_flags();
    let diff_algorithm = nvim_diff_get_algorithm();

    let mut param = XpParam::default();
    let mut emit_cfg = XdemitConf::default();
    let mut emit_cb = XdemitCb::default();

    param.flags = diff_algorithm as c_ulong;

    if (diff_flags & DIFF_IWHITE) != 0 {
        param.flags |= XDF_IGNORE_WHITESPACE_CHANGE;
    }
    if (diff_flags & DIFF_IWHITEALL) != 0 {
        param.flags |= XDF_IGNORE_WHITESPACE;
    }
    if (diff_flags & DIFF_IWHITEEOL) != 0 {
        param.flags |= XDF_IGNORE_WHITESPACE_AT_EOL;
    }
    if (diff_flags & DIFF_IBLANK) != 0 {
        param.flags |= XDF_IGNORE_BLANK_LINES;
    }

    emit_cfg.ctxlen = 0; // don't need any diff_context here
    emit_cb.priv_ = nvim_diffio_get_dout(dio);
    emit_cfg.hunk_func = Some(rs_xdiff_out);

    let orig = MmFileInternal {
        ptr: nvim_diffio_get_orig_ptr(dio),
        size: nvim_diffio_get_orig_size(dio),
    };
    let new = MmFileInternal {
        ptr: nvim_diffio_get_new_ptr(dio),
        size: nvim_diffio_get_new_size(dio),
    };

    if xdl_diff(
        std::ptr::addr_of!(orig),
        std::ptr::addr_of!(new),
        std::ptr::addr_of!(param),
        std::ptr::addr_of!(emit_cfg),
        std::ptr::addr_of_mut!(emit_cb),
    ) < 0
    {
        nvim_diff_emsg_e960();
        return FAIL;
    }
    OK
}

// ============================================================================
// Phase 3: diff_file (dispatch between diffexpr, internal xdiff, external shell)
// ============================================================================

/// Dispatch diff computation for files stored in `dio`.
///
/// Replicates C `diff_file(dio)`:
/// - If diffexpr is set (`p_dex != NUL`): run eval_diff
/// - If internal flag set in dio: run rs_diff_file_internal (xdiff)
/// - Otherwise: run external shell diff
///
/// # Safety
/// `dio` must be a valid, non-null diffio handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_file(dio: DiffioHandle) -> c_int {
    if !nvim_is_diffexpr_empty() {
        // Use 'diffexpr' to generate the diff file.
        let orig = nvim_diffio_get_orig_fname(dio);
        let new_f = nvim_diffio_get_new_fname(dio);
        let diff = nvim_diffio_get_diff_fname(dio);
        nvim_diff_eval_diff(orig, new_f, diff);
        return OK;
    }

    // Use xdiff for internal diff.
    if nvim_diffio_is_internal(dio) {
        return rs_diff_file_internal(dio);
    }

    // External shell diff.
    rs_diff_run_external_shell(dio)
}

// Additional accessor used in rs_diff_file but declared in buffer.rs for other uses.
// Re-declare here to avoid cross-module dependencies.
extern "C" {
    fn nvim_is_diffexpr_empty() -> bool;
    fn nvim_diffio_get_new_fname(dio: DiffioHandle) -> *const c_char;
}

// ============================================================================
// Phase 3b: run_external_shell (build diff command and shell out)
// ============================================================================

// TriState value for diff_a_works = kFalse (diff -a is known to not work)
const K_FALSE_A: c_int = 0;

/// Run the external diff command for the given diffio.
///
/// Replicates C `nvim_diff_run_external_shell(dio)`:
/// - Clears DIFF_OPTIONS environment variable if set
/// - Builds `diff [-a] [-b] [-w] [-Z] [-B] [-i] orig new` command string
/// - Appends shell redirect via rs_append_redir
/// - Calls the shell with filter+silent+doout flags
///
/// # Safety
/// `dio` must be a valid, non-null diffio handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_run_external_shell(dio: DiffioHandle) -> c_int {
    let tmp_orig = nvim_diffio_get_orig_fname(dio);
    let tmp_new = nvim_diffio_get_new_fname(dio);
    let tmp_diff = nvim_diffio_get_diff_fname(dio);
    let p_srr = nvim_diff_get_p_srr();

    // Compute length just like C: sum of fname lengths + srr length + 27
    let orig_len = libc::strlen(tmp_orig);
    let new_len = libc::strlen(tmp_new);
    let diff_len = libc::strlen(tmp_diff);
    let srr_len = libc::strlen(p_srr);
    let len = orig_len + new_len + diff_len + srr_len + 27;

    // Allocate via xmalloc so append_redir / xfree are consistent
    let cmd = nvim_diff_xmalloc(len);

    // Clear DIFF_OPTIONS env var if set (prevents unexpected behavior)
    nvim_diff_env_clear_diff_options();

    // Build the diff command string
    let diff_flags = nvim_diff_get_diff_flags();
    let a_works = nvim_diff_get_a_works();

    let a_flag = if a_works == K_FALSE_A { "" } else { "-a " };
    let b_flag = if (diff_flags & DIFF_IWHITE) != 0 {
        "-b "
    } else {
        ""
    };
    let w_flag = if (diff_flags & DIFF_IWHITEALL) != 0 {
        "-w "
    } else {
        ""
    };
    let z_flag = if (diff_flags & DIFF_IWHITEEOL) != 0 {
        "-Z "
    } else {
        ""
    };
    let cap_b_flag = if (diff_flags & DIFF_IBLANK) != 0 {
        "-B "
    } else {
        ""
    };
    let i_flag = if (diff_flags & DIFF_ICASE) != 0 {
        "-i "
    } else {
        ""
    };

    // Convert tmp_orig and tmp_new to Rust str slices for format!
    let orig_str = std::ffi::CStr::from_ptr(tmp_orig).to_string_lossy();
    let new_str = std::ffi::CStr::from_ptr(tmp_new).to_string_lossy();

    let cmd_str =
        format!("diff {a_flag}{b_flag}{w_flag}{z_flag}{cap_b_flag}{i_flag}{orig_str} {new_str}\0");

    // Copy into the C-allocated buffer (must fit; len was computed to be sufficient)
    let bytes = cmd_str.as_bytes();
    std::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), cmd, bytes.len());

    // Append redirect (modifies cmd buffer in place)
    rs_append_redir(cmd, len, p_srr, tmp_diff);

    // Run the shell command
    nvim_diff_call_shell_diff(cmd);

    // Free the C-allocated buffer
    nvim_diff_xfree(cmd.cast());

    OK
}

// ============================================================================
// Phase 4: check_external_diff (verify external diff works, manage -a flag)
// ============================================================================

// TriState constants (must match C kNone/kFalse/kTrue)
const K_NONE: c_int = -1;
const K_FALSE: c_int = 0;
const K_TRUE: c_int = 1;

// Line buffer length for reading diff output
const LBUFLEN: usize = 50;

extern "C" {
    fn nvim_diff_fopen_write(fname: *const c_char) -> *mut std::ffi::c_void;
    fn nvim_diff_fwrite_line(fd: *mut std::ffi::c_void, data: *const c_char, len: usize) -> bool;
    fn nvim_diff_fopen_read(fname: *const c_char) -> *mut std::ffi::c_void;
    fn nvim_diff_fclose(fd: *mut std::ffi::c_void);
    fn nvim_diff_fgets(fd: *mut std::ffi::c_void, buf: *mut c_char, buflen: c_int) -> bool;
    fn nvim_diff_os_remove(fname: *const c_char);
    fn nvim_diff_emsg_e810();
    fn nvim_diff_emsg_e97();
}

/// Verify that the external diff command works.
///
/// Replicates C `check_external_diff(diffio)`.
/// Writes test files, runs diff, checks output for "1c1" or "@@ -1 +1 @@".
/// Manages retry logic for `-a` flag via `diff_a_works` state.
///
/// # Safety
/// `dio` must be a valid, non-null diffio handle.
#[no_mangle]
pub unsafe extern "C" fn rs_check_external_diff(dio: DiffioHandle) -> c_int {
    // May try twice, first with "-a" and then without.
    let mut io_error = false;
    let mut ok;

    loop {
        ok = K_FALSE;

        let orig_fname = nvim_diffio_get_orig_fname(dio);
        let fd = nvim_diff_fopen_write(orig_fname);

        if fd.is_null() {
            io_error = true;
        } else {
            let line1 = b"line1\n";
            if !nvim_diff_fwrite_line(fd, line1.as_ptr().cast(), line1.len()) {
                io_error = true;
            }
            nvim_diff_fclose(fd);

            let new_fname = nvim_diffio_get_new_fname(dio);
            let fd2 = nvim_diff_fopen_write(new_fname);

            if fd2.is_null() {
                io_error = true;
            } else {
                let line2 = b"line2\n";
                if !nvim_diff_fwrite_line(fd2, line2.as_ptr().cast(), line2.len()) {
                    io_error = true;
                }
                nvim_diff_fclose(fd2);

                let diff_fname = nvim_diffio_get_diff_fname(dio);
                let fd3 = if rs_diff_file(dio) == OK {
                    nvim_diff_fopen_read(diff_fname)
                } else {
                    std::ptr::null_mut()
                };

                if fd3.is_null() {
                    io_error = true;
                } else {
                    let mut linebuf = [0u8; LBUFLEN];
                    loop {
                        if nvim_diff_fgets(fd3, linebuf.as_mut_ptr().cast(), LBUFLEN as c_int) {
                            break;
                        }
                        let line = &linebuf[..];
                        if line.starts_with(b"1c1") || line.starts_with(b"@@ -1 +1 @@") {
                            ok = K_TRUE;
                        }
                    }
                    nvim_diff_fclose(fd3);
                }

                nvim_diff_os_remove(diff_fname);
                nvim_diff_os_remove(new_fname);
            }
            nvim_diff_os_remove(orig_fname);
        }

        // When using 'diffexpr' break here.
        if !nvim_is_diffexpr_empty() {
            break;
        }

        // If we checked if "-a" works already, break here.
        let a_works = nvim_diff_get_a_works();
        if a_works != K_NONE {
            break;
        }
        nvim_diff_set_a_works(ok);

        // If "-a" works break here, otherwise retry without "-a".
        if ok == K_TRUE {
            break;
        }
    }

    if ok != K_TRUE {
        if io_error {
            nvim_diff_emsg_e810();
        }
        nvim_diff_emsg_e97();
        nvim_diff_set_a_works(K_NONE);
        return FAIL;
    }
    OK
}
