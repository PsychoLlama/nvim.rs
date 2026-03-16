//! `:write` command implementation.
//!
//! The `:write` command writes buffer content to a file.
//!
//! ## Usage
//! - `:w[rite]` - Write current buffer
//! - `:w[rite] {file}` - Write to specified file
//! - `:w[rite]!` - Force write (overwrite readonly)
//! - `:{range}w[rite] [file]` - Write specified lines
//! - `:w[rite] >> {file}` - Append to file
//! - `:w[rite] !{cmd}` - Write to shell command (filter)
//! - `:up[date]` - Write only if modified
//! - `:sav[eas] {file}` - Write to file and change buffer name
//!
//! ## Implementation Notes
//!
//! The actual file writing is performed by Neovim's `buf_write()` function.
//! This module provides:
//! - Type definitions for write operations
//! - Validation utilities
//! - Helper functions for the C implementation

use std::ffi::{c_char, c_int, CStr, CString};

use crate::range::{LineNr, LineRange};
use crate::{BufHandle, ExArgHandle, WinHandle};

/// Result of a write operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum WriteResult {
    /// Write succeeded
    Ok = 0,
    /// File is readonly
    Readonly = 1,
    /// Directory does not exist
    NoDirectory = 2,
    /// Permission denied
    PermissionDenied = 3,
    /// File already exists (and no ! given)
    FileExists = 4,
    /// Write was interrupted
    Interrupted = 5,
    /// Disk full or quota exceeded
    NoSpace = 6,
    /// Buffer not modified (for :update)
    NotModified = 7,
    /// Other error
    Error = 99,
}

impl WriteResult {
    /// Check if the write was successful.
    #[inline]
    #[must_use]
    pub const fn is_ok(self) -> bool {
        matches!(self, WriteResult::Ok)
    }

    /// Check if this result indicates the write was skipped (not an error).
    #[inline]
    #[must_use]
    pub const fn is_skipped(self) -> bool {
        matches!(self, WriteResult::NotModified)
    }

    /// Convert from C integer return value (0 = success, non-zero = error).
    #[inline]
    #[must_use]
    pub fn from_c_ok_fail(value: c_int) -> Self {
        if value == 0 {
            WriteResult::Ok
        } else {
            WriteResult::Error
        }
    }

    /// Convert to C integer for return.
    #[inline]
    #[must_use]
    pub fn to_c(self) -> c_int {
        self as c_int
    }
}

/// Options for the `:write` command.
#[derive(Debug, Clone, Default)]
pub struct WriteOptions {
    /// Range of lines to write.
    pub range: LineRange,
    /// Force write (ignore readonly, overwrite existing).
    pub force: bool,
    /// Append to file (>>).
    pub append: bool,
    /// Write to filter command.
    pub filter: bool,
    /// Force binary mode.
    pub force_binary: bool,
    /// Force text mode.
    pub force_text: bool,
    /// Specific encoding to use.
    pub encoding: Option<String>,
    /// Create parent directories if needed (++p).
    pub mkdir_p: bool,
}

impl WriteOptions {
    /// Create options for writing the whole buffer.
    #[must_use]
    pub fn whole_buffer(line_count: LineNr) -> Self {
        Self {
            range: LineRange::whole_buffer(line_count),
            ..Default::default()
        }
    }

    /// Create options for writing a specific range.
    #[must_use]
    pub fn with_range(range: LineRange) -> Self {
        Self {
            range,
            ..Default::default()
        }
    }

    /// Create options for appending to a file.
    #[must_use]
    pub fn append_to(range: LineRange) -> Self {
        Self {
            range,
            append: true,
            ..Default::default()
        }
    }

    /// Create options for force-writing.
    #[must_use]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Set the range.
    #[must_use]
    pub fn range(mut self, range: LineRange) -> Self {
        self.range = range;
        self
    }
}

/// Mode for the write operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WriteMode {
    /// Normal write
    #[default]
    Normal,
    /// Only write if buffer is modified (:update)
    Update,
    /// Write and change buffer name (:saveas)
    SaveAs,
}

/// Error type for write operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteError {
    /// Invalid line range.
    InvalidRange,
    /// File path is empty.
    EmptyPath,
    /// Buffer is readonly.
    Readonly,
    /// File already exists.
    FileExists(String),
    /// Permission denied.
    PermissionDenied(String),
    /// Parent directory does not exist.
    NoDirectory(String),
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteError::InvalidRange => write!(f, "invalid range"),
            WriteError::EmptyPath => write!(f, "empty file path"),
            WriteError::Readonly => write!(f, "buffer is readonly"),
            WriteError::FileExists(path) => write!(f, "file already exists: {path}"),
            WriteError::PermissionDenied(path) => write!(f, "permission denied: {path}"),
            WriteError::NoDirectory(path) => write!(f, "directory does not exist: {path}"),
        }
    }
}

impl std::error::Error for WriteError {}

/// Validate a write range against buffer bounds.
///
/// # Arguments
/// * `range` - The line range to validate
/// * `line_count` - Total lines in the buffer
///
/// # Returns
/// A clamped valid range, or an error if the range is completely invalid.
pub fn validate_write_range(range: LineRange, line_count: LineNr) -> Result<LineRange, WriteError> {
    if line_count == 0 {
        // Empty buffer - any write is technically valid (writes nothing)
        return Ok(LineRange::empty());
    }

    let clamped = range.clamp(line_count);
    if clamped.is_empty() && !range.is_empty() {
        // The range was non-empty but clamped to empty - that's invalid
        return Err(WriteError::InvalidRange);
    }

    Ok(clamped)
}

/// Check if a write should proceed for :update command.
///
/// # Arguments
/// * `is_modified` - Whether the buffer has been modified
/// * `mode` - The write mode
///
/// # Returns
/// `true` if the write should proceed, `false` if it should be skipped.
#[inline]
#[must_use]
pub fn should_write(is_modified: bool, mode: WriteMode) -> bool {
    match mode {
        WriteMode::Update => is_modified,
        WriteMode::Normal | WriteMode::SaveAs => true,
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Validate write range and return validity.
///
/// Returns 1 if valid, 0 if invalid.
pub extern "C" fn rs_validate_write_range(start: c_int, end: c_int, line_count: c_int) -> c_int {
    let range = LineRange::new(start, end);
    c_int::from(validate_write_range(range, line_count).is_ok())
}

/// Check if write should proceed for update command.
///
/// Returns 1 if should write, 0 if should skip.
pub extern "C" fn rs_should_write_update(is_modified: c_int) -> c_int {
    c_int::from(should_write(is_modified != 0, WriteMode::Update))
}

// =============================================================================
// Phase 1: Write Validation Helpers FFI declarations
// =============================================================================

extern "C" {
    fn nvim_excmds_os_nodetype(fname: *const c_char) -> c_int;
    fn nvim_excmds_eap_get_mkdir_p(eap: *const ExArgHandle) -> c_int;
    fn nvim_excmds_os_file_mkdir(fname: *const c_char) -> c_int;
    fn nvim_excmds_buf_get_b_p_ro(buf: *const BufHandle) -> c_int;
    fn nvim_excmds_buf_get_b_fname(buf: *const BufHandle) -> *const c_char;
    fn nvim_excmds_buf_ffname_path_exists(buf: *const BufHandle) -> c_int;
    fn nvim_excmds_buf_ffname_is_writable(buf: *const BufHandle) -> c_int;
    fn nvim_excmds_p_confirm_or_cmod_confirm() -> c_int;
    fn nvim_excmds_vim_dialog_yesno_question(msg: *const c_char) -> c_int;
    fn nvim_excmds_dialog_msg_readonly(fmt_id: c_int, arg: *const c_char) -> *mut c_char;
    fn nvim_excmds_error_msg(error_id: c_int, arg: *const c_char);
    fn nvim_excmds_set_forceit(eap: *mut ExArgHandle, val: c_int);
    fn nvim_exarg_get_forceit(eap: *const ExArgHandle) -> c_int;
    fn xfree(ptr: *mut std::ffi::c_void);
}

// =============================================================================
// Phase 1: Write Validation Helpers (Rust implementations)
// =============================================================================

/// Check 'write' option. Returns true (1) if writing is disabled (error printed).
///
/// # Safety
/// No pointers involved.
#[no_mangle]
pub unsafe extern "C" fn rs_not_writing() -> c_int {
    if crate::p_write != 0 {
        return 0; // writing is enabled, no error
    }
    nvim_excmds_error_msg(ERR_E142, std::ptr::null());
    1 // writing is disabled
}

/// Check if fname is a writable device (Unix only). Returns FAIL (0) or OK (1).
///
/// # Safety
/// `fname` must be a valid C string pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_check_writable(fname: *const c_char) -> c_int {
    #[cfg(unix)]
    {
        if nvim_excmds_os_nodetype(fname) == NODE_OTHER_VAL {
            nvim_excmds_error_msg(ERR_E503, fname);
            return 0; // FAIL
        }
    }
    #[cfg(not(unix))]
    {
        let _ = fname;
    }
    1 // OK
}

/// Handle ++p (mkdir -p) argument for write command.
/// Returns OK (1) on success, FAIL (0) if mkdir failed.
///
/// # Safety
/// `eap` and `fname` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_handle_mkdir_p_arg(
    eap: *mut ExArgHandle,
    fname: *const c_char,
) -> c_int {
    if nvim_excmds_eap_get_mkdir_p(eap) != 0 && nvim_excmds_os_file_mkdir(fname) < 0 {
        return 0; // FAIL
    }
    1 // OK
}

/// Check if buffer is readonly, possibly prompting with a dialog.
/// Returns true (1) if readonly (writing not allowed), false (0) if writing is allowed.
/// May set eap->forceit to true if the user confirms override.
///
/// # Safety
/// `eap` and `buf` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_check_readonly(eap: *mut ExArgHandle, buf: *mut BufHandle) -> c_int {
    let forceit = nvim_exarg_get_forceit(eap);
    if forceit != 0 {
        return 0; // not readonly when forced
    }

    let b_p_ro = nvim_excmds_buf_get_b_p_ro(buf);
    let ffname_exists = nvim_excmds_buf_ffname_path_exists(buf);
    let ffname_writable = nvim_excmds_buf_ffname_is_writable(buf);

    // Check: buffer has 'readonly' set OR (file exists AND is not writable)
    let is_readonly = b_p_ro != 0 || (ffname_exists != 0 && ffname_writable == 0);

    if !is_readonly {
        return 0; // not readonly
    }

    let b_fname = nvim_excmds_buf_get_b_fname(buf);
    if nvim_excmds_p_confirm_or_cmod_confirm() != 0 && !b_fname.is_null() {
        // Show a dialog
        let fmt_id = if b_p_ro != 0 { 0 } else { 1 };
        let buff = nvim_excmds_dialog_msg_readonly(fmt_id, b_fname);
        let yes = nvim_excmds_vim_dialog_yesno_question(buff);
        xfree(buff.cast());
        if yes != 0 {
            // User confirmed: set forceit and allow write
            nvim_excmds_set_forceit(eap, 1);
            return 0; // not readonly (user overrode)
        }
        return 1; // readonly (user declined)
    }

    // No dialog: emit error
    if b_p_ro != 0 {
        nvim_excmds_error_msg(ERR_E_READONLY, std::ptr::null());
    } else {
        nvim_excmds_error_msg(ERR_E505, b_fname);
    }
    1 // readonly
}

// =============================================================================
// Phase 5: getfile, set_swapcommand, delbuf_msg FFI declarations and implementations
// =============================================================================

// GETFILE_* constants -- verified by _Static_assert in ex_cmds_shim.c
const GETFILE_ERROR_VAL: c_int = 1;
const GETFILE_NOT_WRITTEN_VAL: c_int = 2;
const GETFILE_SAME_FILE_VAL: c_int = 0;
const GETFILE_OPEN_OTHER_VAL: c_int = -1;

// ECMD_* constants -- verified by _Static_assert in ex_cmds_shim.c
const ECMD_HIDE_VAL: c_int = 0x01;
const ECMD_FORCEIT_VAL: c_int = 0x08;

// BF_* constants -- verified by _Static_assert in ex_cmds_shim.c
const BF_NOTEDITED_VAL: c_int = 0x08;
const BF_NEW_VAL: c_int = 0x10;
const BF_READERR_VAL: c_int = 0x40;

// BL_SOL | BL_FIX -- verified by _Static_assert in ex_cmds_shim.c
const BL_SOL_FIX_VAL: c_int = 6;

// NODE_OTHER -- verified by _Static_assert in ex_cmds_shim.c
const NODE_OTHER_VAL: c_int = 2;

// Error IDs for nvim_excmds_error_msg dispatcher (see ex_cmds_shim.c)
const ERR_E142: c_int = 5; // E142 Writing is disabled
const ERR_E503: c_int = 6; // E503 not a file or writable device
const ERR_E505: c_int = 7; // E505 read-only
const ERR_E_READONLY: c_int = 8; // e_readonly
const ERR_ISADIR2: c_int = 9; // e_isadir2 (fname)
const ERR_E_EXISTS: c_int = 10; // e_exists
const ERR_E768: c_int = 11; // E768 Swap file exists
const ERR_E140: c_int = 12; // E140 partial buffer
const ERR_E_ARGREQ: c_int = 13; // e_argreq
const ERR_E143: c_int = 14; // E143 autocommands deleted new buffer + clear au_new_curbuf

extern "C" {
    fn nvim_excmds_check_can_set_curbuf_forceit(forceit: c_int) -> c_int;
    fn nvim_excmds_text_locked() -> c_int;
    fn nvim_excmds_curbuf_locked() -> c_int;
    fn nvim_excmds_fname_expand(
        ffname_in: *mut c_char,
        sfname_in: *mut c_char,
        out_ffname: *mut *mut c_char,
        out_sfname: *mut *mut c_char,
    );
    fn nvim_excmds_curbuf_get_b_fnum() -> c_int;
    fn nvim_excmds_curbuf_get_b_nwindows() -> c_int;
    fn nvim_excmds_buf_hide_curbuf() -> c_int;
    fn nvim_excmds_autowrite_curbuf(forceit: c_int) -> c_int;
    fn nvim_excmds_dialog_changed_curbuf();
    fn nvim_excmds_no_write_message();
    fn nvim_excmds_no_wait_return_inc();
    fn nvim_excmds_no_wait_return_dec();
    fn setpcmark();
    fn nvim_curwin_set_cursor_lnum(lnum: c_int);
    fn nvim_get_curwin() -> *mut WinHandle;
    fn nvim_excmds_get_vim_var_str_swapcommand() -> *const c_char;
    fn nvim_excmds_set_vim_var_string_swapcommand(p: *const c_char);
    fn beginline(flags: c_int);
}

/// Try to abandon the current file and edit a new or existing file.
///
/// Returns GETFILE_ERROR, GETFILE_NOT_WRITTEN, GETFILE_SAME_FILE, or GETFILE_OPEN_OTHER.
///
/// # Safety
/// All pointer arguments must be valid or null.
#[allow(clippy::must_use_candidate)]
#[export_name = "getfile"]
pub unsafe extern "C" fn rs_getfile(
    fnum: c_int,
    ffname_arg: *mut c_char,
    sfname_arg: *mut c_char,
    setpm: c_int,
    lnum: c_int,
    forceit: c_int,
) -> c_int {
    if nvim_excmds_check_can_set_curbuf_forceit(forceit) == 0 {
        return GETFILE_ERROR_VAL;
    }
    if nvim_excmds_text_locked() != 0 {
        return GETFILE_ERROR_VAL;
    }
    if nvim_excmds_curbuf_locked() != 0 {
        return GETFILE_ERROR_VAL;
    }

    let (ffname, sfname, free_me, other) = if fnum == 0 {
        // Expand filename
        let mut out_ffname: *mut c_char = ffname_arg;
        let mut out_sfname: *mut c_char = sfname_arg;
        nvim_excmds_fname_expand(ffname_arg, sfname_arg, &mut out_ffname, &mut out_sfname);
        let other = nvim_excmds_otherfile(out_ffname);
        // out_ffname is the newly allocated expanded name
        let free_me = if out_ffname != ffname_arg {
            out_ffname
        } else {
            std::ptr::null_mut()
        };
        (out_ffname, out_sfname, free_me, other)
    } else {
        let other = if fnum != nvim_excmds_curbuf_get_b_fnum() {
            1
        } else {
            0
        };
        (
            ffname_arg,
            sfname_arg,
            std::ptr::null_mut::<c_char>(),
            other,
        )
    };

    if other != 0 {
        nvim_excmds_no_wait_return_inc();
    }

    let retval;

    if other != 0
        && forceit == 0
        && nvim_excmds_curbuf_get_b_nwindows() == 1
        && nvim_excmds_buf_hide_curbuf() == 0
        && nvim_excmds_curbufIsChanged() != 0
        && nvim_excmds_autowrite_curbuf(forceit) == 0
    {
        if crate::p_confirm != 0 && crate::p_write != 0 {
            nvim_excmds_dialog_changed_curbuf();
        }
        if nvim_excmds_curbufIsChanged() != 0 {
            nvim_excmds_no_wait_return_dec();
            nvim_excmds_no_write_message();
            xfree(free_me.cast());
            return GETFILE_NOT_WRITTEN_VAL;
        }
    }

    if other != 0 {
        nvim_excmds_no_wait_return_dec();
    }
    if setpm != 0 {
        setpcmark();
    }

    if other == 0 {
        if lnum != 0 {
            nvim_curwin_set_cursor_lnum(lnum);
        }
        crate::nvim_check_cursor_lnum_call();
        beginline(BL_SOL_FIX_VAL);
        retval = GETFILE_SAME_FILE_VAL;
    } else {
        let hide_flag = if nvim_excmds_buf_hide_curbuf() != 0 {
            ECMD_HIDE_VAL
        } else {
            0
        };
        let force_flag = if forceit != 0 { ECMD_FORCEIT_VAL } else { 0 };
        let flags = hide_flag + force_flag;
        let curwin = nvim_get_curwin();
        if crate::edit::rs_do_ecmd(
            fnum,
            ffname,
            sfname,
            std::ptr::null_mut(),
            lnum,
            flags,
            curwin,
        ) != 0
        {
            retval = GETFILE_OPEN_OTHER_VAL;
        } else {
            retval = GETFILE_ERROR_VAL;
        }
    }

    xfree(free_me.cast());
    retval
}

/// Set v:swapcommand for SwapExists autocommands.
///
/// Returns 1 if the swapcommand was set, 0 otherwise.
///
/// # Safety
/// `command` must be a valid C string pointer or null.
#[export_name = "set_swapcommand"]
#[allow(clippy::must_use_candidate)]
pub unsafe extern "C" fn rs_set_swapcommand(command: *const c_char, newlnum: c_int) -> bool {
    // Don't set if both command is null and newlnum <= 0
    if command.is_null() && newlnum <= 0 {
        return false;
    }
    // Don't set if v:swapcommand is already set
    let existing = nvim_excmds_get_vim_var_str_swapcommand();
    if !existing.is_null() && *existing != 0 {
        return false;
    }

    // Format swapcommand string: ":%s\r" if command is set, else "<lnum>G".
    let s = if !command.is_null() {
        let cmd = CStr::from_ptr(command).to_string_lossy();
        CString::new(format!(":{cmd}\r")).unwrap_or_default()
    } else {
        CString::new(format!("{}G", newlnum)).unwrap_or_default()
    };
    nvim_excmds_set_vim_var_string_swapcommand(s.as_ptr());
    true
}

/// Emit E143 error and clear au_new_curbuf.
///
/// Frees `name` (C-allocated string).
///
/// # Safety
/// `name` must be a valid C-allocated string pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_delbuf_msg(name: *mut c_char) {
    nvim_excmds_error_msg(ERR_E143, name);
    xfree(name.cast());
}

// =============================================================================
// Phase 4: do_wqall FFI declarations and implementation
// =============================================================================

extern "C" {
    fn nvim_excmds_cmd_xall() -> c_int;
    fn nvim_excmds_cmd_wqall() -> c_int;
    fn nvim_excmds_before_quit_all(eap: *mut ExArgHandle) -> c_int;
    fn nvim_excmds_getout(code: c_int);
    fn nvim_excmds_not_exiting();
    fn nvim_excmds_buf_get_next(buf: *const BufHandle) -> *mut BufHandle;
    fn nvim_excmds_buf_has_running_job(buf: *const BufHandle) -> c_int;
    fn nvim_excmds_no_write_message_nobang(buf: *mut BufHandle);
    fn nvim_excmds_bufIsChanged(buf: *mut BufHandle) -> c_int;
    fn nvim_excmds_bt_dontwrite(buf: *const BufHandle) -> c_int;
    fn nvim_excmds_buf_get_b_ffname_ptr(buf: *const BufHandle) -> *const c_char;
    fn nvim_excmds_buf_get_b_fnum(buf: *const BufHandle) -> c_int;
    fn nvim_excmds_semsg_e141(fnum: i64);
    fn nvim_excmds_check_readonly_buf(
        forceit_in: c_int,
        buf: *mut BufHandle,
        forceit_out: *mut c_int,
    ) -> c_int;

    fn nvim_excmds_new_bufref(buf: *mut BufHandle) -> *mut std::ffi::c_void;
    fn nvim_excmds_bufref_valid(refp: *mut std::ffi::c_void) -> c_int;
    fn nvim_excmds_free_bufref(refp: *mut std::ffi::c_void);

    fn nvim_excmds_buf_write_all(buf: *mut BufHandle, forceit: c_int) -> c_int;
    fn nvim_exarg_get_cmdidx(eap: *mut ExArgHandle) -> c_int;
}

/// Implement `:wall`, `:wqall`, `:xall`. Replaces C `do_wqall`.
///
/// Write all changed buffers. For :wqall/:xall, also exit after writing.
///
/// # Safety
/// `eap` must be a valid exarg_T pointer.
#[export_name = "do_wqall"]
pub unsafe extern "C" fn rs_do_wqall(eap: *mut ExArgHandle) {
    let mut error: c_int = 0;
    let save_forceit = nvim_exarg_get_forceit(eap);

    let cmdidx = nvim_exarg_get_cmdidx(eap);
    let cmd_xall = nvim_excmds_cmd_xall();
    let cmd_wqall = nvim_excmds_cmd_wqall();

    if cmdidx == cmd_xall || cmdidx == cmd_wqall {
        if nvim_excmds_before_quit_all(eap) == 0 {
            return; // FAIL from before_quit_all
        }
        crate::exiting = true;
    }

    // Iterate all buffers (manually walk the linked list)
    let mut buf = crate::firstbuf;
    while !buf.is_null() {
        let exiting = crate::exiting;

        if exiting && nvim_excmds_buf_has_running_job(buf) != 0 {
            nvim_excmds_no_write_message_nobang(buf);
            error += 1;
        } else if nvim_excmds_bufIsChanged(buf) == 0 || nvim_excmds_bt_dontwrite(buf) != 0 {
            buf = nvim_excmds_buf_get_next(buf);
            continue;
        } else {
            // Buffer needs writing
            if rs_not_writing() != 0 {
                error += 1;
                break; // 'write' option disabled, stop processing
            }

            let ffname = nvim_excmds_buf_get_b_ffname_ptr(buf);
            if ffname.is_null() {
                let fnum = nvim_excmds_buf_get_b_fnum(buf) as i64;
                nvim_excmds_semsg_e141(fnum);
                error += 1;
            } else {
                // Check readonly and overwrite
                let mut forceit_out: c_int = save_forceit;
                let readonly = nvim_excmds_check_readonly_buf(save_forceit, buf, &mut forceit_out);
                nvim_excmds_set_forceit(eap, forceit_out);

                let buf_fname = nvim_excmds_buf_get_b_fname(buf);
                if readonly != 0 || rs_check_overwrite(eap, buf, buf_fname, ffname, 0) == 0 {
                    error += 1;
                } else {
                    // Track buffer ref in case autocmds delete it
                    let bufref = nvim_excmds_new_bufref(buf);
                    let forceit = nvim_exarg_get_forceit(eap);
                    if rs_handle_mkdir_p_arg(eap, buf_fname) == 0
                        || nvim_excmds_buf_write_all(buf, forceit) == 0
                    {
                        error += 1;
                    }
                    // An autocommand may have deleted the buffer.
                    if nvim_excmds_bufref_valid(bufref) == 0 {
                        nvim_excmds_free_bufref(bufref);
                        // Reset to start of list
                        buf = crate::firstbuf;
                        nvim_excmds_set_forceit(eap, save_forceit);
                        continue;
                    }
                    nvim_excmds_free_bufref(bufref);
                }
            }
            nvim_excmds_set_forceit(eap, save_forceit); // check_overwrite may have set it
        }

        buf = nvim_excmds_buf_get_next(buf);
    }

    if crate::exiting {
        if error == 0 {
            nvim_excmds_getout(0); // exit Vim (diverges)
        }
        nvim_excmds_not_exiting();
    }
}

// =============================================================================
// Phase 3: check_overwrite FFI declarations and implementation
// =============================================================================

extern "C" {
    fn nvim_excmds_bt_nofilename(buf: *const BufHandle) -> c_int;
    fn nvim_excmds_buf_get_b_flags(buf: *const BufHandle) -> c_int;
    fn nvim_excmds_cpo_no_overnew() -> c_int;
    fn nvim_excmds_os_path_exists(ffname: *const c_char) -> c_int;
    fn nvim_excmds_os_isdir(ffname: *const c_char) -> c_int;
    fn nvim_excmds_dialog_overwrite(eap: *mut ExArgHandle, fname: *const c_char) -> c_int;
    fn nvim_excmds_get_first_dir() -> *mut c_char;
    fn nvim_excmds_makeswapname(
        fname: *const c_char,
        ffname: *const c_char,
        dir: *const c_char,
    ) -> *mut c_char;
    fn nvim_excmds_dialog_swapfile(eap: *mut ExArgHandle, swapname: *const c_char) -> c_int;
}

/// Check if overwriting a file is allowed.
///
/// Returns OK (1) if it's OK to write, FAIL (0) if not.
/// May set eap->forceit if a dialog says it's OK to overwrite.
///
/// # Safety
/// All pointer arguments must be valid.
#[allow(clippy::must_use_candidate)]
#[export_name = "check_overwrite"]
pub unsafe extern "C" fn rs_check_overwrite(
    eap: *mut ExArgHandle,
    buf: *mut BufHandle,
    fname: *const c_char,
    ffname: *const c_char,
    other: c_int,
) -> c_int {
    // Check if overwrite check is needed
    let b_flags = nvim_excmds_buf_get_b_flags(buf);
    let is_nofilename = nvim_excmds_bt_nofilename(buf) != 0;

    let needs_check = other != 0
        || (!is_nofilename
            && ((b_flags & BF_NOTEDITED_VAL) != 0
                || ((b_flags & BF_NEW_VAL) != 0 && nvim_excmds_cpo_no_overnew() != 0)
                || (b_flags & BF_READERR_VAL) != 0));

    if !needs_check || crate::p_wa != 0 || nvim_excmds_os_path_exists(ffname) == 0 {
        return 1; // OK
    }

    let forceit = nvim_exarg_get_forceit(eap);
    let append = nvim_excmds_eap_get_append(eap);

    if forceit == 0 && append == 0 {
        // Check if target is a directory (Unix only)
        #[cfg(unix)]
        {
            if nvim_excmds_os_isdir(ffname) != 0 {
                nvim_excmds_error_msg(ERR_ISADIR2, ffname);
                return 0; // FAIL
            }
        }

        if nvim_excmds_p_confirm_or_cmod_confirm() != 0 {
            if nvim_excmds_dialog_overwrite(eap, fname) == 0 {
                return 0; // FAIL (user declined)
            }
            // forceit is set by nvim_excmds_dialog_overwrite
        } else {
            nvim_excmds_error_msg(ERR_E_EXISTS, std::ptr::null());
            return 0; // FAIL
        }
    }

    // For ":w! filename" check that no swap file exists for "filename".
    if other != 0 && crate::emsg_silent == 0 {
        let dir = nvim_excmds_get_first_dir();
        let swapname = nvim_excmds_makeswapname(fname, ffname, dir);
        xfree(dir.cast());

        if nvim_excmds_os_path_exists(swapname) != 0 {
            if nvim_excmds_p_confirm_or_cmod_confirm() != 0 {
                if nvim_excmds_dialog_swapfile(eap, swapname) == 0 {
                    xfree(swapname.cast());
                    return 0; // FAIL
                }
                // forceit set by dialog_swapfile
            } else {
                nvim_excmds_error_msg(ERR_E768, swapname);
                xfree(swapname.cast());
                return 0; // FAIL
            }
        }
        xfree(swapname.cast());
    }

    1 // OK
}

// =============================================================================
// Phase 2: do_write FFI declarations and implementation
// =============================================================================

extern "C" {
    fn nvim_excmds_get_arg_mut(eap: *mut ExArgHandle) -> *mut c_char;
    fn nvim_excmds_eap_get_append(eap: *const ExArgHandle) -> c_int;
    fn nvim_excmds_eap_get_line1(eap: *const ExArgHandle) -> c_int;
    fn nvim_excmds_eap_get_line2_val(eap: *const ExArgHandle) -> c_int;
    fn nvim_excmds_fix_fname(ffname: *const c_char) -> *mut c_char;
    fn nvim_excmds_otherfile(ffname: *const c_char) -> c_int;
    fn nvim_excmds_vim_strchr_cpo_altwrite() -> c_int;
    fn nvim_excmds_setaltfname(
        ffname: *const c_char,
        fname: *const c_char,
        lnum: c_int,
    ) -> *mut BufHandle;
    fn nvim_excmds_buflist_findname(ffname: *const c_char) -> *mut BufHandle;
    fn nvim_excmds_buf_has_mfp(buf: *const BufHandle) -> c_int;
    fn nvim_excmds_emsg_e_bufloaded();
    fn nvim_excmds_bt_dontwrite_msg_curbuf() -> c_int;
    fn nvim_excmds_check_fname() -> c_int;
    fn nvim_excmds_curbuf_check_writable() -> c_int;
    fn nvim_excmds_dialog_write_partial() -> c_int;
    fn nvim_excmds_curbuf_get_ffname() -> *mut c_char;
    fn nvim_excmds_curbuf_get_fname() -> *mut c_char;
    fn nvim_get_curbuf() -> *mut BufHandle;

    fn nvim_excmds_do_saveas_swap(alt_buf: *mut BufHandle, out_sfname: *mut *const c_char)
        -> c_int;
    fn nvim_excmds_buf_write_do_write(
        ffname: *const c_char,
        fname: *const c_char,
        line1: c_int,
        line2: c_int,
        eap: *mut ExArgHandle,
        append: c_int,
        forceit: c_int,
    ) -> c_int;
    fn nvim_excmds_saveas_post_success();
    fn nvim_excmds_curbuf_ffname_null() -> c_int;
    fn nvim_excmds_do_autochdir();
    fn nvim_exarg_cmdidx_is_saveas(eap: *const ExArgHandle) -> c_int;
}

/// Implement `do_write`. Replaces C `do_write`.
///
/// Write current buffer to file specified in `eap->arg`.
///
/// # Safety
/// `eap` must be a valid exarg_T pointer.
#[allow(clippy::must_use_candidate)]
#[export_name = "do_write"]
pub unsafe extern "C" fn rs_do_write(eap: *mut ExArgHandle) -> c_int {
    let mut retval: c_int = 0; // FAIL

    // Check 'write' option
    if rs_not_writing() != 0 {
        return 0; // FAIL
    }

    let arg = nvim_excmds_get_arg_mut(eap);
    // Determine file names
    let (mut ffname, mut fname, free_fname, other) = {
        // Read first char of arg
        let first_char = if arg.is_null() { 0u8 } else { *arg as u8 };
        if first_char == 0 {
            // No argument
            if nvim_exarg_cmdidx_is_saveas(eap) != 0 {
                nvim_excmds_error_msg(ERR_E_ARGREQ, std::ptr::null());
                return 0; // FAIL (goto theend with free_fname=NULL)
            }
            (
                std::ptr::null_mut::<c_char>(),
                std::ptr::null_mut::<c_char>(),
                std::ptr::null_mut::<c_char>(),
                0,
            )
        } else {
            // Has argument
            let fname_ptr = arg;
            let free_ptr = nvim_excmds_fix_fname(fname_ptr);
            let ff = if !free_ptr.is_null() {
                free_ptr
            } else {
                fname_ptr
            };
            let other = nvim_excmds_otherfile(ff);
            (ff, fname_ptr, free_ptr, other)
        }
    };

    let mut alt_buf: *mut BufHandle = std::ptr::null_mut();

    // If we have a new file, put its name in the list of alternate file names.
    if other != 0 {
        if nvim_excmds_vim_strchr_cpo_altwrite() != 0 || nvim_exarg_cmdidx_is_saveas(eap) != 0 {
            alt_buf = nvim_excmds_setaltfname(ffname, fname, 1);
        } else {
            alt_buf = nvim_excmds_buflist_findname(ffname);
        }
        if !alt_buf.is_null() && nvim_excmds_buf_has_mfp(alt_buf) != 0 {
            nvim_excmds_emsg_e_bufloaded();
            xfree(free_fname.cast());
            return 0; // FAIL
        }
    }

    // Writing to the current file checks
    if other == 0 {
        let curbuf = nvim_get_curbuf();
        if nvim_excmds_bt_dontwrite_msg_curbuf() != 0
            || nvim_excmds_check_fname() == 0
            || nvim_excmds_curbuf_check_writable() == 0
            || rs_check_readonly(eap, curbuf) != 0
        {
            xfree(free_fname.cast());
            return 0; // FAIL
        }
    }

    if other == 0 {
        // Writing to current file; use curbuf's names
        ffname = nvim_excmds_curbuf_get_ffname() as *mut c_char;
        fname = nvim_excmds_curbuf_get_fname() as *mut c_char;

        // Partial write check
        let line1 = nvim_excmds_eap_get_line1(eap);
        let line2 = nvim_excmds_eap_get_line2_val(eap);
        let line_count = nvim_curbuf_get_b_ml_ml_line_count();
        let forceit = nvim_exarg_get_forceit(eap);
        let append = nvim_excmds_eap_get_append(eap);
        let p_wa = crate::p_wa;

        if (line1 != 1 || line2 != line_count) && forceit == 0 && append == 0 && p_wa == 0 {
            if nvim_excmds_p_confirm_or_cmod_confirm() != 0 {
                if nvim_excmds_dialog_write_partial() == 0 {
                    xfree(free_fname.cast());
                    return 0; // FAIL
                }
                nvim_excmds_set_forceit(eap, 1);
            } else {
                nvim_excmds_error_msg(ERR_E140, std::ptr::null());
                xfree(free_fname.cast());
                return 0; // FAIL
            }
        }
    }

    let curbuf = nvim_get_curbuf();
    if rs_check_overwrite(eap, curbuf, fname, ffname, other) != 0 {
        // check_overwrite returned OK
        let is_saveas = nvim_exarg_cmdidx_is_saveas(eap) != 0;

        if is_saveas && !alt_buf.is_null() {
            let mut sfname_out: *const c_char = std::ptr::null();
            if nvim_excmds_do_saveas_swap(alt_buf, &mut sfname_out) == 0 {
                // Buffer changed, abort
                xfree(free_fname.cast());
                return 0; // FAIL
            }
            // Use the updated sfname from curbuf
            fname = sfname_out as *mut c_char;
        }

        if rs_handle_mkdir_p_arg(eap, fname) == 0 {
            xfree(free_fname.cast());
            return 0; // FAIL
        }

        let name_was_missing = nvim_excmds_curbuf_ffname_null();
        let append = nvim_excmds_eap_get_append(eap);
        let forceit = nvim_exarg_get_forceit(eap);
        let line1 = nvim_excmds_eap_get_line1(eap);
        let line2 = nvim_excmds_eap_get_line2_val(eap);

        let write_ok =
            nvim_excmds_buf_write_do_write(ffname, fname, line1, line2, eap, append, forceit);
        retval = write_ok;

        if is_saveas && write_ok != 0 {
            nvim_excmds_saveas_post_success();
        }

        if is_saveas || name_was_missing != 0 {
            nvim_excmds_do_autochdir();
        }
    }

    xfree(free_fname.cast());
    retval
}

// =============================================================================
// ex_update, ex_write, ex_wnext (Phase 3 migration)
// =============================================================================

extern "C" {
    fn nvim_excmds_curbufIsChanged() -> c_int;
    fn nvim_excmds_bt_nofilename_curbuf() -> c_int;
    fn nvim_excmds_curbuf_ffname_not_null() -> c_int;
    fn nvim_excmds_os_path_exists_curbuf_ffname() -> c_int;

    fn nvim_exarg_get_usefilter(eap: *const ExArgHandle) -> c_int;
    fn nvim_exarg_set_line1(eap: *mut ExArgHandle, line1: c_int);
    fn nvim_exarg_set_line2(eap: *mut ExArgHandle, line2: c_int);
    fn nvim_curbuf_get_b_ml_ml_line_count() -> c_int;
    fn nvim_excmds_curwin_get_w_arg_idx() -> c_int;
    fn nvim_exarg_get_cmd_byte1(eap: *const ExArgHandle) -> c_int;
    fn nvim_exarg_get_line2(eap: *const ExArgHandle) -> c_int;
    fn nvim_excmds_do_argfile(eap: *mut ExArgHandle, i: c_int);
}

/// Implement `:update` command. Replaces C `ex_update`.
///
/// Writes the buffer only if it has been changed or if the file does not exist.
///
/// # Safety
/// `eap` must be a valid exarg_T pointer.
#[export_name = "ex_update"]
pub unsafe extern "C" fn rs_ex_update(eap: *mut ExArgHandle) {
    let is_changed = nvim_excmds_curbufIsChanged() != 0;
    let no_filename = nvim_excmds_bt_nofilename_curbuf() != 0;
    let has_ffname = nvim_excmds_curbuf_ffname_not_null() != 0;
    let path_exists = nvim_excmds_os_path_exists_curbuf_ffname() != 0;

    if is_changed || (!no_filename && has_ffname && !path_exists) {
        rs_do_write(eap);
    }
}

/// Implement `:write` and `:saveas` commands. Replaces C `ex_write`.
///
/// # Safety
/// `eap` must be a valid exarg_T pointer.
#[export_name = "ex_write"]
pub unsafe extern "C" fn rs_ex_write(eap: *mut ExArgHandle) {
    if nvim_exarg_cmdidx_is_saveas(eap) != 0 {
        // :saveas does not take a range, uses all lines.
        nvim_exarg_set_line1(eap, 1);
        let line_count = nvim_curbuf_get_b_ml_ml_line_count();
        nvim_exarg_set_line2(eap, line_count);
    }

    if nvim_exarg_get_usefilter(eap) != 0 {
        // input lines to shell command
        crate::shell::rs_do_bang(1, eap, false, true, false);
    } else {
        rs_do_write(eap);
    }
}

/// Implement `:wnext`, `:wNext`, `:wprevious` commands. Replaces C `ex_wnext`.
///
/// # Safety
/// `eap` must be a valid exarg_T pointer.
#[export_name = "ex_wnext"]
pub unsafe extern "C" fn rs_ex_wnext(eap: *mut ExArgHandle) {
    let cmd_byte1 = nvim_exarg_get_cmd_byte1(eap);
    let line2_count = nvim_exarg_get_line2(eap);
    let w_arg_idx = nvim_excmds_curwin_get_w_arg_idx();

    let i = if cmd_byte1 == b'n' as c_int {
        w_arg_idx + line2_count
    } else {
        w_arg_idx - line2_count
    };

    nvim_exarg_set_line1(eap, 1);
    let line_count = nvim_curbuf_get_b_ml_ml_line_count();
    nvim_exarg_set_line2(eap, line_count);

    if rs_do_write(eap) != 0 {
        nvim_excmds_do_argfile(eap, i);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_result() {
        assert!(WriteResult::Ok.is_ok());
        assert!(!WriteResult::Readonly.is_ok());
        assert!(!WriteResult::Error.is_ok());

        assert!(WriteResult::NotModified.is_skipped());
        assert!(!WriteResult::Ok.is_skipped());
    }

    #[test]
    fn test_write_result_from_c() {
        assert_eq!(WriteResult::from_c_ok_fail(0), WriteResult::Ok);
        assert_eq!(WriteResult::from_c_ok_fail(1), WriteResult::Error);
    }

    #[test]
    fn test_validate_write_range() {
        // Normal range
        let range = LineRange::new(5, 10);
        let result = validate_write_range(range, 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), range);

        // Range extending beyond buffer - gets clamped
        let range = LineRange::new(5, 150);
        let result = validate_write_range(range, 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LineRange::new(5, 100));

        // Empty buffer
        let range = LineRange::new(1, 10);
        let result = validate_write_range(range, 0);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_should_write() {
        // Normal mode always writes
        assert!(should_write(true, WriteMode::Normal));
        assert!(should_write(false, WriteMode::Normal));

        // Update mode only writes if modified
        assert!(should_write(true, WriteMode::Update));
        assert!(!should_write(false, WriteMode::Update));

        // SaveAs always writes
        assert!(should_write(true, WriteMode::SaveAs));
        assert!(should_write(false, WriteMode::SaveAs));
    }

    #[test]
    fn test_write_options_whole_buffer() {
        let opts = WriteOptions::whole_buffer(100);
        assert_eq!(opts.range, LineRange::whole_buffer(100));
        assert!(!opts.force);
        assert!(!opts.append);
    }

    #[test]
    fn test_write_options_with_range() {
        let range = LineRange::new(5, 20);
        let opts = WriteOptions::with_range(range);
        assert_eq!(opts.range, range);
    }

    #[test]
    fn test_write_options_append() {
        let range = LineRange::new(5, 20);
        let opts = WriteOptions::append_to(range);
        assert!(opts.append);
        assert_eq!(opts.range, range);
    }

    #[test]
    fn test_write_options_builder() {
        let opts = WriteOptions::with_range(LineRange::new(1, 10)).force();
        assert!(opts.force);
        assert_eq!(opts.range.start, 1);
        assert_eq!(opts.range.end, 10);
    }

    #[test]
    fn test_write_error_display() {
        let err = WriteError::InvalidRange;
        assert_eq!(format!("{err}"), "invalid range");

        let err = WriteError::EmptyPath;
        assert_eq!(format!("{err}"), "empty file path");

        let err = WriteError::Readonly;
        assert_eq!(format!("{err}"), "buffer is readonly");

        let err = WriteError::FileExists("test.txt".to_string());
        assert_eq!(format!("{err}"), "file already exists: test.txt");
    }

    #[test]
    fn test_rs_validate_write_range() {
        assert_eq!(rs_validate_write_range(1, 10, 100), 1);
        assert_eq!(rs_validate_write_range(5, 150, 100), 1); // Gets clamped
    }

    #[test]
    fn test_rs_should_write_update() {
        assert_eq!(rs_should_write_update(1), 1);
        assert_eq!(rs_should_write_update(0), 0);
    }
}
