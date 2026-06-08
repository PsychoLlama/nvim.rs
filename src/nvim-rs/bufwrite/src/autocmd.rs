//! Autocommand handling for buffer writing.
//!
//! Mirrors the C `buf_write_do_autocmds` and `buf_write_do_post_autocmds` functions.

#![allow(clippy::too_many_arguments)]
#![allow(clippy::fn_params_excessive_bools)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::similar_names)]

use std::ffi::{c_char, c_int};
use std::ptr;

use crate::ffi::{BufHandle, ExargHandle, FAIL, NOTDONE, OK};

// EVENT_* constants (from auevents_enum.generated.h)
const EVENT_FILEAPPENDCMD: c_int = 48;
const EVENT_FILEAPPENDPRE: c_int = 50;
const EVENT_FILEAPPENDPOST: c_int = 49;
const EVENT_FILTERWRITEPRE: c_int = 65;
const EVENT_FILTERWRITEPOST: c_int = 64;
const EVENT_BUFWRITECMD: c_int = 20;
const EVENT_BUFWRITEPRE: c_int = 22;
const EVENT_BUFWRITEPOST: c_int = 21;
const EVENT_FILEWRITECMD: c_int = 59;
const EVENT_FILEWRITEPRE: c_int = 61;
const EVENT_FILEWRITEPOST: c_int = 60;

// Buffer flags
const BF_NEW: c_int = 0x10;
const BF_WRITE_MASK: c_int = 0x58; // BF_NOTEDITED(0x08) + BF_NEW(0x10) + BF_READERR(0x40)

// CPO_PLUS
const CPO_PLUS: c_int = b'+' as c_int;

// CMOD_LOCKMARKS
const CMOD_LOCKMARKS: c_int = 0x0800;

/// Matches C `pos_T` layout: { linenr_T lnum; colnr_T col; colnr_T coladd; }
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PosT {
    pub lnum: i32,
    pub col: c_int,
    pub coladd: c_int,
}

/// Opaque handle to a C `aco_save_T` struct.
type AcoHandle = *mut std::ffi::c_void;

/// Opaque handle to a C `bufref_T` struct.
type BufrefHandle = *mut std::ffi::c_void;

unsafe extern "C" {
    static mut msg_scroll: c_int;
    static mut no_wait_return: c_int;
    static mut curbuf: BufHandle;
}

extern "C" {
    // Autocmd operations (use opaque handles)
    fn nvim_bw_aucmd_prepbuf(aco: AcoHandle, buf: BufHandle);
    fn nvim_bw_aucmd_restbuf(aco: AcoHandle);
    fn nvim_bw_set_bufref(bufref: BufrefHandle, buf: BufHandle);
    #[link_name = "bufref_valid"]
    fn bufref_valid(bufref: BufrefHandle) -> bool;
    #[link_name = "apply_autocmds_exarg"]
    fn apply_autocmds_exarg(
        event: c_int,
        fname: *const c_char,
        fname_io: *const c_char,
        force: bool,
        buf: BufHandle,
        eap: ExargHandle,
    ) -> bool;

    // Buffer queries
    fn nvim_bw_bt_nofilename(buf: BufHandle) -> c_int;
    #[link_name = "curbufIsChanged"]
    fn curbufIsChanged() -> bool;
    #[link_name = "aborting"]
    fn aborting() -> bool;

    // Buffer field accessors (names match C functions exactly)
    fn nvim_bw_buf_get_ml_line_count(buf: BufHandle) -> i32;
    fn nvim_bw_buf_get_ml_mfp_nonnull(buf: BufHandle) -> c_int;
    fn nvim_bw_buf_get_ffname(buf: BufHandle) -> *mut c_char;
    fn nvim_bw_buf_get_sfname(buf: BufHandle) -> *mut c_char;
    fn nvim_bw_buf_get_changed(buf: BufHandle) -> c_int;
    fn nvim_bw_buf_get_flags(buf: BufHandle) -> c_int;
    fn nvim_bw_buf_set_flags(buf: BufHandle, val: c_int);
    fn nvim_bw_buf_set_op_start(buf: BufHandle, pos: PosT);
    fn nvim_bw_buf_set_op_end(buf: BufHandle, pos: PosT);
    fn nvim_bw_buf_set_no_eol_lnum(buf: BufHandle, val: i32);

    // Undo
    #[link_name = "u_unchanged"]
    fn u_unchanged(buf: BufHandle);
    #[link_name = "u_update_save_nr"]
    fn u_update_save_nr(buf: BufHandle);
    #[link_name = "ml_timestamp"]
    fn ml_timestamp(buf: BufHandle);

    // Globals
    fn nvim_bw_get_cmdmod_cmod_flags() -> c_int;
    fn nvim_bw_cpo_contains(c: c_int) -> c_int;

    // Error/message
    #[link_name = "emsg"]
    fn emsg(msg: *const c_char) -> c_int;
    fn nvim_bw_gettext(s: *const c_char) -> *const c_char;
    fn nvim_bw_semsg_nofile_err(buf: BufHandle);

    // Struct sizes
    fn nvim_bw_sizeof_aco_save() -> usize;
    fn nvim_bw_sizeof_bufref() -> usize;
}

/// Execute pre-write autocommands.
///
/// Replaces C `buf_write_do_autocmds`.
///
/// # Safety
///
/// All pointers must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_write_do_autocmds(
    buf: BufHandle,
    fnamep: *mut *mut c_char,
    sfnamep: *mut *mut c_char,
    ffnamep: *mut *mut c_char,
    start: i32,
    endp: *mut i32,
    eap: ExargHandle,
    append: c_int,
    filtering: c_int,
    reset_changed: c_int,
    overwriting: c_int,
    whole: c_int,
    orig_start: PosT,
    orig_end: PosT,
) -> c_int {
    let old_line_count = unsafe { nvim_bw_buf_get_ml_line_count(buf) };
    let msg_save = unsafe { msg_scroll };

    // Heap-allocate aco_save_T and bufref_T as opaque blobs
    let aco_size = unsafe { nvim_bw_sizeof_aco_save() };
    let bufref_size = unsafe { nvim_bw_sizeof_bufref() };
    let mut aco_buf = vec![0u8; aco_size];
    let mut bufref_buf = vec![0u8; bufref_size];
    let aco: AcoHandle = aco_buf.as_mut_ptr().cast();
    let bufref: BufrefHandle = bufref_buf.as_mut_ptr().cast();

    let mut did_cmd = false;
    let mut nofile_err = false;
    let empty_memline = unsafe { nvim_bw_buf_get_ml_mfp_nonnull(buf) } == 0;

    let sfname = unsafe { *sfnamep };

    // Save pointer identity with buffer fields
    let buf_ffname = unsafe { *ffnamep == nvim_bw_buf_get_ffname(buf) };
    let buf_sfname = unsafe { sfname == nvim_bw_buf_get_sfname(buf) };
    let buf_fname_f = unsafe { *fnamep == nvim_bw_buf_get_ffname(buf) };
    let buf_fname_s = unsafe { *fnamep == nvim_bw_buf_get_sfname(buf) };

    // Set curwin/curbuf to buf and save a few things
    unsafe { nvim_bw_aucmd_prepbuf(aco, buf) };
    unsafe { nvim_bw_set_bufref(bufref, buf) };

    if append != 0 {
        did_cmd = unsafe {
            apply_autocmds_exarg(EVENT_FILEAPPENDCMD, sfname, sfname, false, curbuf, eap)
        };
        if !did_cmd {
            if overwriting != 0 && unsafe { nvim_bw_bt_nofilename(curbuf) } != 0 {
                nofile_err = true;
            } else {
                unsafe {
                    apply_autocmds_exarg(EVENT_FILEAPPENDPRE, sfname, sfname, false, curbuf, eap);
                }
            }
        }
    } else if filtering != 0 {
        unsafe {
            apply_autocmds_exarg(
                EVENT_FILTERWRITEPRE,
                ptr::null(),
                sfname,
                false,
                curbuf,
                eap,
            );
        }
    } else if reset_changed != 0 && whole != 0 {
        let was_changed = unsafe { curbufIsChanged() };

        did_cmd =
            unsafe { apply_autocmds_exarg(EVENT_BUFWRITECMD, sfname, sfname, false, curbuf, eap) };
        if did_cmd {
            if was_changed && !unsafe { curbufIsChanged() } {
                unsafe {
                    u_unchanged(curbuf);
                    u_update_save_nr(curbuf);
                }
            }
        } else if overwriting != 0 && unsafe { nvim_bw_bt_nofilename(curbuf) } != 0 {
            nofile_err = true;
        } else {
            unsafe {
                apply_autocmds_exarg(EVENT_BUFWRITEPRE, sfname, sfname, false, curbuf, eap);
            }
        }
    } else {
        did_cmd =
            unsafe { apply_autocmds_exarg(EVENT_FILEWRITECMD, sfname, sfname, false, curbuf, eap) };
        if !did_cmd {
            if overwriting != 0 && unsafe { nvim_bw_bt_nofilename(curbuf) } != 0 {
                nofile_err = true;
            } else {
                unsafe {
                    apply_autocmds_exarg(EVENT_FILEWRITEPRE, sfname, sfname, false, curbuf, eap);
                }
            }
        }
    }

    // Restore curwin/curbuf
    unsafe { nvim_bw_aucmd_restbuf(aco) };

    // Check if buffer was deleted/unloaded
    let buf = if unsafe { bufref_valid(bufref) } {
        buf
    } else {
        ptr::null_mut()
    };

    // C condition: buf == NULL
    //   || (buf->b_ml.ml_mfp == NULL && !empty_memline)
    //   || did_cmd || nofile_err || aborting()
    let should_return = buf.is_null()
        || (!buf.is_null()
            && unsafe { nvim_bw_buf_get_ml_mfp_nonnull(buf) } == 0
            && !empty_memline)
        || did_cmd
        || nofile_err
        || unsafe { aborting() };

    if should_return {
        if !buf.is_null() && (unsafe { nvim_bw_get_cmdmod_cmod_flags() } & CMOD_LOCKMARKS != 0) {
            unsafe {
                nvim_bw_buf_set_op_start(buf, orig_start);
                nvim_bw_buf_set_op_end(buf, orig_end);
            }
        }

        unsafe {
            no_wait_return -= 1;
            msg_scroll = msg_save;
        }

        if nofile_err {
            unsafe { nvim_bw_semsg_nofile_err(curbuf) };
        }

        if nofile_err || unsafe { aborting() } {
            return FAIL;
        }
        if did_cmd {
            if buf.is_null() {
                return OK;
            }
            if overwriting != 0 {
                unsafe { ml_timestamp(buf) };
                let flags = unsafe { nvim_bw_buf_get_flags(buf) };
                if append != 0 {
                    unsafe { nvim_bw_buf_set_flags(buf, flags & !BF_NEW) };
                } else {
                    unsafe { nvim_bw_buf_set_flags(buf, flags & !BF_WRITE_MASK) };
                }
            }
            if reset_changed != 0
                && unsafe { nvim_bw_buf_get_changed(buf) } != 0
                && append == 0
                && (overwriting != 0 || unsafe { nvim_bw_cpo_contains(CPO_PLUS) } != 0)
            {
                return FAIL;
            }
            return OK;
        }
        if !unsafe { aborting() } {
            unsafe {
                emsg(nvim_bw_gettext(
                    c"E203: Autocommands deleted or unloaded buffer to be written".as_ptr(),
                ));
            }
        }
        return FAIL;
    }

    // The autocommands may have changed the number of lines
    let new_line_count = unsafe { nvim_bw_buf_get_ml_line_count(buf) };
    if new_line_count != old_line_count {
        let end = unsafe { *endp };
        if whole != 0 {
            unsafe { *endp = new_line_count };
        } else if new_line_count > old_line_count {
            unsafe { *endp = end + (new_line_count - old_line_count) };
        } else {
            let new_end = end - (old_line_count - new_line_count);
            if new_end < start {
                unsafe {
                    no_wait_return -= 1;
                    msg_scroll = msg_save;
                    emsg(nvim_bw_gettext(
                        c"E204: Autocommand changed number of lines in unexpected way".as_ptr(),
                    ));
                }
                return FAIL;
            }
            unsafe { *endp = new_end };
        }
    }

    // The autocommands may have changed the buffer name
    if buf_ffname {
        unsafe { *ffnamep = nvim_bw_buf_get_ffname(buf) };
    }
    if buf_sfname {
        unsafe { *sfnamep = nvim_bw_buf_get_sfname(buf) };
    }
    if buf_fname_f {
        unsafe { *fnamep = nvim_bw_buf_get_ffname(buf) };
    }
    if buf_fname_s {
        unsafe { *fnamep = nvim_bw_buf_get_sfname(buf) };
    }
    NOTDONE
}

/// Execute post-write autocommands.
///
/// Replaces C `buf_write_do_post_autocmds`.
///
/// # Safety
///
/// All pointers must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_write_do_post_autocmds(
    buf: BufHandle,
    fname: *const c_char,
    eap: ExargHandle,
    append: c_int,
    filtering: c_int,
    reset_changed: c_int,
    whole: c_int,
) {
    let aco_size = unsafe { nvim_bw_sizeof_aco_save() };
    let mut aco_buf = vec![0u8; aco_size];
    let aco: AcoHandle = aco_buf.as_mut_ptr().cast();

    unsafe { nvim_bw_buf_set_no_eol_lnum(curbuf, 0) };

    // Set curwin/curbuf to buf
    unsafe { nvim_bw_aucmd_prepbuf(aco, buf) };

    if append != 0 {
        unsafe {
            apply_autocmds_exarg(EVENT_FILEAPPENDPOST, fname, fname, false, curbuf, eap);
        }
    } else if filtering != 0 {
        unsafe {
            apply_autocmds_exarg(
                EVENT_FILTERWRITEPOST,
                ptr::null(),
                fname,
                false,
                curbuf,
                eap,
            );
        }
    } else if reset_changed != 0 && whole != 0 {
        unsafe {
            apply_autocmds_exarg(EVENT_BUFWRITEPOST, fname, fname, false, curbuf, eap);
        }
    } else {
        unsafe {
            apply_autocmds_exarg(EVENT_FILEWRITEPOST, fname, fname, false, curbuf, eap);
        }
    }

    // Restore curwin/curbuf
    unsafe { nvim_bw_aucmd_restbuf(aco) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_t_layout() {
        assert_eq!(std::mem::size_of::<PosT>(), 12);
        assert_eq!(std::mem::align_of::<PosT>(), 4);
    }

    #[test]
    fn test_event_constants() {
        // Just ensure they're distinct and in expected range
        let events = [
            EVENT_FILEAPPENDCMD,
            EVENT_FILEAPPENDPRE,
            EVENT_FILEAPPENDPOST,
            EVENT_FILTERWRITEPRE,
            EVENT_FILTERWRITEPOST,
            EVENT_BUFWRITECMD,
            EVENT_BUFWRITEPRE,
            EVENT_BUFWRITEPOST,
            EVENT_FILEWRITECMD,
            EVENT_FILEWRITEPRE,
            EVENT_FILEWRITEPOST,
        ];
        for &e in &events {
            assert!(e >= 0);
        }
        // All distinct
        for i in 0..events.len() {
            for j in (i + 1)..events.len() {
                assert_ne!(events[i], events[j]);
            }
        }
    }

    #[test]
    fn test_buffer_flag_constants() {
        assert_eq!(BF_NEW, 0x10);
        assert_eq!(BF_WRITE_MASK, 0x58);
        // BF_WRITE_MASK should include BF_NEW
        assert_ne!(BF_WRITE_MASK & BF_NEW, 0);
    }
}
