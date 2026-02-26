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
