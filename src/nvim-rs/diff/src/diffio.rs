//! Diff I/O pipeline: buffer serialization, xdiff invocation, external diff, verification.
//!
//! This module provides Rust implementations for the diff I/O pipeline,
//! replacing the C functions: diff_write_buffer, clear_diffin, clear_diffout,
//! diff_file_internal, diff_file, diff_write, and check_external_diff.

#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::c_char;
use std::os::raw::c_int;

use crate::buffer::BufHandle;

// ============================================================================
// Result constants
// ============================================================================

const OK: c_int = 1;

// ============================================================================
// Diff flags (must match C #define values in diff_shim.c)
// ============================================================================

const DIFF_ICASE: c_int = 0x004;

// ============================================================================
// ML flags (must match C #define values in memline_defs.h)
// ============================================================================

const ML_EMPTY: c_int = 0x02; // empty buffer

// NL (0x0a) is Neovim's internal representation of NUL (0x00) in lines.
const NL: u8 = b'\n';
const NUL: u8 = 0;

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
