//! High-level sign operations
//!
//! This module provides the high-level API for sign operations:
//! - Sign placement (sign_place)
//! - Sign unplacement (sign_unplace)
//! - Sign jump (sign_jump)
//!
//! These functions orchestrate the lower-level placement and query functions.

use std::ffi::{c_char, c_int};

use crate::{bref, LinenrT, SignBufHandle, SignHandle, SIGN_DEF_PRIO};

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    /// Get sign by name from the sign map
    fn nvim_sign_map_get(name: *const c_char) -> SignHandle;

    // Error reporting
    fn semsg(fmt: *const c_char, ...);
    fn emsg(msg: *const c_char) -> c_int;

    // Sign placement helpers
    fn rs_buf_set_sign(
        buf: crate::SignBufHandle,
        id: *mut u32,
        group: *const c_char,
        prio: c_int,
        lnum: LinenrT,
        sp: SignHandle,
    );
    fn rs_buf_mod_sign(
        buf: crate::SignBufHandle,
        id: *mut u32,
        group: *const c_char,
        prio: c_int,
        sp: SignHandle,
    ) -> LinenrT;
    fn rs_sign_effective_priority(prio: c_int) -> c_int;

    // Sign unplace helpers
    fn rs_sign_buffer_has_signs(buf: crate::SignBufHandle) -> bool;
    fn rs_buf_delete_signs(
        buf: crate::SignBufHandle,
        group: *const c_char,
        id: c_int,
        atlnum: LinenrT,
    ) -> c_int;
    fn extmark_del_id(buf: crate::SignBufHandle, ns: u32, id: u32) -> bool;
    fn group_get_ns(group: *const c_char) -> i64;

    // Buffer iteration for all-buffer operations
    fn nvim_get_firstbuf() -> crate::SignBufHandle;

    // Jump accessors (Phase 3)
    fn rs_buf_findsign(buf: crate::SignBufHandle, id: c_int, group: *const c_char) -> LinenrT;
    fn rs_foldOpenCursor();
    fn nvim_buf_jump_open_win(buf: crate::SignBufHandle) -> *mut std::ffi::c_void; // win_T*
    fn nvim_curwin_check_and_beginline();
    fn nvim_curwin_set_cursor_lnum(lnum: LinenrT); // from ex_cmds_shim.c
    fn nvim_do_cmdline_cmd_str(cmd: *const c_char);
    fn xmallocz(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut std::ffi::c_void);
    fn snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
}

// Error format strings
const E155_FMT: &[u8] = b"E155: Unknown sign: %s\0";
const E885_FMT: &[u8] = b"E885: Not possible to change sign %s\0";

// =============================================================================
// Sign Place Operation
// =============================================================================

/// Result of a sign_place operation.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignPlaceOpResult {
    /// Success - sign was placed
    Ok = 0,
    /// Invalid group name (reserved character)
    InvalidGroup = 1,
    /// Unknown sign name
    UnknownSign = 2,
    /// Could not modify existing sign
    ModifyFailed = 3,
    /// General failure
    Failed = 4,
}

/// Parameters validated for sign_place operation.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SignPlaceOpParams {
    /// Sign definition handle
    pub sp: SignHandle,
    /// Effective priority
    pub priority: c_int,
    /// Whether this is a new placement (lnum > 0) or modification (lnum == 0)
    pub is_new_placement: bool,
}

impl std::fmt::Debug for SignPlaceOpParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignPlaceOpParams")
            .field("sp", &"SignHandle")
            .field("priority", &self.priority)
            .field("is_new_placement", &self.is_new_placement)
            .finish()
    }
}

/// Validate and prepare sign_place operation.
///
/// Returns the validated parameters for the operation, or an error.
///
/// # Safety
/// `group` and `name` must be null or valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_place_prepare(
    group: *const c_char,
    name: *const c_char,
    lnum: LinenrT,
    prio: c_int,
) -> SignPlaceOpParams {
    // Default error result
    let error_result = SignPlaceOpParams {
        sp: std::ptr::null_mut(),
        priority: SIGN_DEF_PRIO,
        is_new_placement: false,
    };

    // Name must be provided
    if name.is_null() {
        return error_result;
    }

    // Check for reserved character '*' in group name
    if !group.is_null() {
        let group_byte = *group.cast::<u8>();
        if group_byte == b'*' || group_byte == 0 {
            return error_result;
        }
    }

    // Look up sign definition
    let sp = nvim_sign_map_get(name);
    if sp.is_null() {
        return error_result;
    }

    // Calculate effective priority
    let sign_prio = (*sp).sn_priority;
    let effective_prio = if prio == -1 && sign_prio != -1 {
        if sign_prio == -1 {
            SIGN_DEF_PRIO
        } else {
            sign_prio
        }
    } else if prio == -1 {
        SIGN_DEF_PRIO
    } else {
        prio
    };

    SignPlaceOpParams {
        sp,
        priority: effective_prio,
        is_new_placement: lnum > 0,
    }
}

/// Check if sign_place preparation was successful.
///
/// # Safety
/// `params` must be null or a valid pointer to `SignPlaceOpParams`.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_place_params_valid(params: *const SignPlaceOpParams) -> c_int {
    if params.is_null() {
        return 0;
    }
    c_int::from(!(*params).sp.is_null())
}

// =============================================================================
// Sign Unplace Operation (Batch vs Single)
// =============================================================================

/// Batch mode for sign unplace operation.
///
/// Determines whether to delete a single sign or multiple signs.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignUnplaceBatch {
    /// Delete a single sign by ID
    Single = 0,
    /// Delete multiple signs (by line, group, or all)
    Multiple = 1,
}

/// Determine the unplace batch mode based on parameters.
#[no_mangle]
pub extern "C" fn rs_sign_unplace_batch_mode(
    id: c_int,
    atlnum: LinenrT,
    group_is_all: c_int,
) -> SignUnplaceBatch {
    if id == 0 || atlnum > 0 || group_is_all != 0 {
        SignUnplaceBatch::Multiple
    } else {
        SignUnplaceBatch::Single
    }
}

// =============================================================================
// Sign Jump Operation
// =============================================================================

/// Result of a sign_jump operation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SignJumpResult {
    /// Line number (0 or -1 on error)
    pub lnum: LinenrT,
    /// Whether the sign was found
    pub found: bool,
    /// Whether the buffer has a name (needed for jumping to unopened buffers)
    pub buffer_has_name: bool,
}

/// Create a sign jump result for a found sign.
#[no_mangle]
pub extern "C" fn rs_sign_jump_found(lnum: LinenrT, buffer_has_name: c_int) -> SignJumpResult {
    SignJumpResult {
        lnum,
        found: true,
        buffer_has_name: buffer_has_name != 0,
    }
}

/// Create a sign jump result for a not-found sign.
#[no_mangle]
pub extern "C" fn rs_sign_jump_not_found() -> SignJumpResult {
    SignJumpResult {
        lnum: -1,
        found: false,
        buffer_has_name: false,
    }
}

/// Jump target type for sign_jump.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignJumpTarget {
    /// Jump within current window
    CurrentWindow = 0,
    /// Need to open buffer in new/existing window
    OpenBuffer = 1,
    /// Cannot jump - buffer has no name
    NoName = 2,
    /// Sign not found
    NotFound = 3,
}

/// Determine jump target type.
#[no_mangle]
pub extern "C" fn rs_sign_jump_target(
    sign_found: c_int,
    win_is_current: c_int,
    buffer_has_name: c_int,
) -> SignJumpTarget {
    if sign_found == 0 {
        SignJumpTarget::NotFound
    } else if win_is_current != 0 {
        SignJumpTarget::CurrentWindow
    } else if buffer_has_name != 0 {
        SignJumpTarget::OpenBuffer
    } else {
        SignJumpTarget::NoName
    }
}

// =============================================================================
// Sign Command Dispatching
// =============================================================================

/// Sign command to execute.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignCommand {
    /// Place a sign
    Place = 0,
    /// Unplace a sign
    Unplace = 1,
    /// Jump to a sign
    Jump = 2,
    /// Define a sign
    Define = 3,
    /// Undefine a sign
    Undefine = 4,
    /// List signs
    List = 5,
}

impl SignCommand {
    /// Convert from command index.
    pub const fn from_index(idx: c_int) -> Option<Self> {
        match idx {
            3 => Some(Self::Place),
            4 => Some(Self::Unplace),
            5 => Some(Self::Jump),
            0 => Some(Self::Define),
            1 => Some(Self::Undefine),
            2 => Some(Self::List),
            _ => None,
        }
    }
}

/// FFI export: Convert command index to SignCommand enum.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub extern "C" fn rs_sign_cmd_from_index(idx: c_int) -> c_int {
    SignCommand::from_index(idx).map_or(-1, |cmd| cmd as c_int)
}

// =============================================================================
// Sign Placement Execution
// =============================================================================

/// Parameters for sign placement execution.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SignPlaceExecParams {
    /// Buffer handle
    pub buf: crate::SignBufHandle,
    /// Namespace ID (0 for global)
    pub ns_id: u32,
    /// Sign ID (0 for auto-assign)
    pub id: u32,
    /// Line number (1-based)
    pub lnum: LinenrT,
    /// Priority
    pub priority: c_int,
    /// Sign definition handle
    pub sp: SignHandle,
}

impl Default for SignPlaceExecParams {
    fn default() -> Self {
        Self {
            buf: crate::SignBufHandle::null(),
            ns_id: 0,
            id: 0,
            lnum: 0,
            priority: SIGN_DEF_PRIO,
            sp: std::ptr::null_mut(),
        }
    }
}

/// Create default sign placement execution params.
#[no_mangle]
pub extern "C" fn rs_sign_place_exec_params_default() -> SignPlaceExecParams {
    SignPlaceExecParams::default()
}

/// Check if sign placement exec params are valid.
#[no_mangle]
pub extern "C" fn rs_sign_place_exec_valid(params: &SignPlaceExecParams) -> c_int {
    // Need buffer, sign definition, valid line
    c_int::from(!params.buf.is_null() && !params.sp.is_null() && params.lnum > 0)
}

// =============================================================================
// Sign Unplace Execution
// =============================================================================

/// Parameters for sign unplace execution.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SignUnplaceExecParams {
    /// Buffer handle (null for all buffers)
    pub buf: crate::SignBufHandle,
    /// Namespace ID (0 for global, u32::MAX for all)
    pub ns_id: u64,
    /// Sign ID (0 for all in namespace)
    pub id: u32,
    /// Line number filter (0 for all lines)
    pub atlnum: LinenrT,
}

impl Default for SignUnplaceExecParams {
    fn default() -> Self {
        Self {
            buf: crate::SignBufHandle::null(),
            ns_id: 0,
            id: 0,
            atlnum: 0,
        }
    }
}

/// Create default sign unplace execution params.
#[no_mangle]
pub extern "C" fn rs_sign_unplace_exec_params_default() -> SignUnplaceExecParams {
    SignUnplaceExecParams::default()
}

/// Check if unplace should affect all buffers.
#[no_mangle]
pub extern "C" fn rs_sign_unplace_all_buffers(params: &SignUnplaceExecParams) -> c_int {
    c_int::from(params.buf.is_null())
}

/// Check if unplace should affect all namespaces.
#[no_mangle]
pub extern "C" fn rs_sign_unplace_all_namespaces(params: &SignUnplaceExecParams) -> c_int {
    c_int::from(params.ns_id == u64::from(u32::MAX))
}

/// Check if unplace should affect all signs.
#[no_mangle]
pub extern "C" fn rs_sign_unplace_all_signs(params: &SignUnplaceExecParams) -> c_int {
    c_int::from(params.id == 0)
}

// =============================================================================
// Sign Define Execution
// =============================================================================

/// Parameters for sign define execution.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SignDefineExecParams {
    /// Sign name
    pub name: *const c_char,
    /// Icon path (null if not set)
    pub icon: *const c_char,
    /// Sign text (null if not set)
    pub text: *const c_char,
    /// Text highlight ID (0 if not set)
    pub text_hl: c_int,
    /// Line highlight ID (0 if not set)
    pub line_hl: c_int,
    /// Number highlight ID (0 if not set)
    pub num_hl: c_int,
    /// Cursorline highlight ID (0 if not set)
    pub cul_hl: c_int,
    /// Priority (-1 for default)
    pub priority: c_int,
}

impl Default for SignDefineExecParams {
    fn default() -> Self {
        Self {
            name: std::ptr::null(),
            icon: std::ptr::null(),
            text: std::ptr::null(),
            text_hl: 0,
            line_hl: 0,
            num_hl: 0,
            cul_hl: 0,
            priority: -1,
        }
    }
}

/// Create default sign define execution params.
#[no_mangle]
pub extern "C" fn rs_sign_define_exec_params_default() -> SignDefineExecParams {
    SignDefineExecParams::default()
}

/// Check if sign define exec params have a valid name.
///
/// # Safety
/// `params.name` must be null or a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_define_exec_valid(params: &SignDefineExecParams) -> c_int {
    if params.name.is_null() {
        return 0;
    }
    // Check name is not empty
    c_int::from(*params.name.cast::<u8>() != 0)
}

/// Check if sign define exec has any visual attributes.
#[no_mangle]
pub extern "C" fn rs_sign_define_exec_has_attrs(params: &SignDefineExecParams) -> c_int {
    c_int::from(
        !params.icon.is_null()
            || !params.text.is_null()
            || params.text_hl > 0
            || params.line_hl > 0
            || params.num_hl > 0
            || params.cul_hl > 0
            || params.priority >= 0,
    )
}

// =============================================================================
// Sign Undefine Execution
// =============================================================================

/// Result of sign undefine operation.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignUndefineResult {
    /// Success
    Ok = 0,
    /// Sign not found
    NotFound = 1,
    /// Invalid name
    InvalidName = 2,
}

/// Check if a sign can be undefined.
///
/// # Safety
/// `name` must be null or a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_can_undefine(name: *const c_char) -> SignUndefineResult {
    if name.is_null() {
        return SignUndefineResult::InvalidName;
    }

    // Check name is not empty
    if *name.cast::<u8>() == 0 {
        return SignUndefineResult::InvalidName;
    }

    // Look up the sign
    let sp = nvim_sign_map_get(name);
    if sp.is_null() {
        SignUndefineResult::NotFound
    } else {
        SignUndefineResult::Ok
    }
}

// =============================================================================
// Core High-Level Operations
// =============================================================================

/// Inline implementation of unplace_inner: unplace sign(s) from a single buffer.
///
/// Returns OK (1) on success, FAIL (0) on failure.
///
/// # Safety
/// All pointer parameters must be valid.
unsafe fn unplace_inner(
    buf: crate::SignBufHandle,
    id: c_int,
    group: *const c_char,
    atlnum: LinenrT,
) -> c_int {
    if !rs_sign_buffer_has_signs(buf) {
        return 0; // FAIL
    }

    let group_is_star = !group.is_null() && *group.cast::<u8>() == b'*';
    if id == 0 || atlnum > 0 || group_is_star {
        if rs_buf_delete_signs(buf, group, id, atlnum) == 0 {
            return 0; // FAIL
        }
    } else {
        let ns = group_get_ns(group);
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        if ns < 0 || !extmark_del_id(buf, ns as u32, id as u32) {
            return 0; // FAIL
        }
    }

    1 // OK
}

/// C-visible export: unplace sign(s) from a single buffer (replaces C impl).
///
/// # Safety
/// All pointer parameters must be valid.
#[unsafe(export_name = "nvim_sign_unplace_inner_impl")]
pub unsafe extern "C" fn rs_sign_unplace_inner_impl(
    buf: crate::SignBufHandle,
    id: c_int,
    group: *const c_char,
    atlnum: LinenrT,
) -> c_int {
    unplace_inner(buf, id, group, atlnum)
}

/// Place a sign at the specified file location or update a sign.
///
/// Returns OK (1) on success, FAIL (0) on failure.
///
/// # Safety
/// All pointer parameters must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_place(
    id: *mut u32,
    group: *const c_char,
    name: *const c_char,
    buf: crate::SignBufHandle,
    lnum: LinenrT,
    prio: c_int,
) -> c_int {
    // Check for reserved character '*' or empty string in group name
    if !group.is_null() {
        let group_byte = *group.cast::<u8>();
        if group_byte == b'*' || group_byte == 0 {
            return 0; // FAIL
        }
    }

    // Look up sign definition
    let sp = nvim_sign_map_get(name);
    if sp.is_null() {
        semsg(E155_FMT.as_ptr().cast(), name);
        return 0; // FAIL
    }

    // Calculate effective priority: if prio == -1 and sign has a priority, use it
    let effective_prio = rs_sign_effective_priority(if prio == -1 && (*sp).sn_priority != -1 {
        (*sp).sn_priority
    } else {
        prio
    });

    let result_lnum = if lnum > 0 {
        rs_buf_set_sign(buf, id, group, effective_prio, lnum, sp);
        lnum
    } else {
        rs_buf_mod_sign(buf, id, group, effective_prio, sp)
    };

    if result_lnum <= 0 {
        semsg(E885_FMT.as_ptr().cast(), name);
        return 0; // FAIL
    }

    1 // OK
}

/// Unplace the specified sign for a single or all buffers.
///
/// Returns OK (1) on success, FAIL (0) on failure.
///
/// # Safety
/// All pointer parameters must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_unplace(
    buf: crate::SignBufHandle,
    id: c_int,
    group: *const c_char,
    atlnum: LinenrT,
) -> c_int {
    if buf.is_null() {
        let mut retval = 1; // OK
        let mut cbuf = nvim_get_firstbuf();
        while !cbuf.is_null() {
            if unplace_inner(cbuf, id, group, atlnum) == 0 {
                retval = 0; // FAIL (at least one failed)
            }
            cbuf = SignBufHandle(bref(cbuf).b_next);
        }
        retval
    } else {
        unplace_inner(buf, id, group, atlnum)
    }
}

/// Jump to a sign.
///
/// Returns the line number on success, -1 on failure.
///
/// # Safety
/// All pointer parameters must be valid.
#[no_mangle]
#[allow(clippy::manual_c_str_literals)]
pub unsafe extern "C" fn rs_sign_jump(
    id: c_int,
    group: *const c_char,
    buf: crate::SignBufHandle,
) -> LinenrT {
    let lnum = rs_buf_findsign(buf, id, group);
    if lnum <= 0 {
        static E157_FMT: &[u8] = b"E157: Invalid sign ID: %d\0";
        semsg(E157_FMT.as_ptr().cast(), id);
        return -1;
    }
    if nvim_buf_jump_open_win(buf).is_null() {
        // Need to open buffer
        let fname = bref(buf).b_fname;
        if fname.is_null() {
            emsg(
                b"E934: Cannot jump to a buffer that does not have a name\0"
                    .as_ptr()
                    .cast(),
            );
            return -1;
        }
        // Calculate command length: "e +<lnum> <fname>\0"
        let fname_len = {
            let mut len = 0usize;
            while *fname.add(len) != 0 {
                len += 1;
            }
            len
        };
        let cmd_len = fname_len + 24;
        let cmd = xmallocz(cmd_len);
        snprintf(
            cmd,
            cmd_len,
            b"e +%lld %s\0".as_ptr().cast(),
            i64::from(lnum),
            fname,
        );
        nvim_do_cmdline_cmd_str(cmd);
        xfree(cmd.cast());
    } else {
        // Jumped to existing window
        nvim_curwin_set_cursor_lnum(lnum);
        nvim_curwin_check_and_beginline();
    }
    rs_foldOpenCursor();
    lnum
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_place_exec_params_default() {
        let params = SignPlaceExecParams::default();
        assert!(params.buf.is_null());
        assert_eq!(params.ns_id, 0);
        assert_eq!(params.id, 0);
        assert_eq!(params.lnum, 0);
        assert_eq!(params.priority, SIGN_DEF_PRIO);
        assert!(params.sp.is_null());
    }

    #[test]
    fn test_sign_unplace_exec_params_default() {
        let params = SignUnplaceExecParams::default();
        assert!(params.buf.is_null());
        assert_eq!(params.ns_id, 0);
        assert_eq!(params.id, 0);
        assert_eq!(params.atlnum, 0);
    }

    #[test]
    fn test_sign_define_exec_params_default() {
        let params = SignDefineExecParams::default();
        assert!(params.name.is_null());
        assert!(params.icon.is_null());
        assert!(params.text.is_null());
        assert_eq!(params.text_hl, 0);
        assert_eq!(params.line_hl, 0);
        assert_eq!(params.num_hl, 0);
        assert_eq!(params.cul_hl, 0);
        assert_eq!(params.priority, -1);
    }

    #[test]
    fn test_sign_define_exec_has_attrs() {
        let default = SignDefineExecParams::default();
        assert_eq!(rs_sign_define_exec_has_attrs(&default), 0);

        let with_prio = SignDefineExecParams {
            priority: 10,
            ..Default::default()
        };
        assert_eq!(rs_sign_define_exec_has_attrs(&with_prio), 1);
    }

    #[test]
    fn test_sign_undefine_result() {
        assert_eq!(SignUndefineResult::Ok as c_int, 0);
        assert_eq!(SignUndefineResult::NotFound as c_int, 1);
        assert_eq!(SignUndefineResult::InvalidName as c_int, 2);
    }

    #[test]
    fn test_sign_place_op_result() {
        assert_eq!(SignPlaceOpResult::Ok as c_int, 0);
        assert_eq!(SignPlaceOpResult::InvalidGroup as c_int, 1);
        assert_eq!(SignPlaceOpResult::UnknownSign as c_int, 2);
    }

    #[test]
    fn test_sign_unplace_batch_mode() {
        // Single deletion
        assert_eq!(
            rs_sign_unplace_batch_mode(5, 0, 0),
            SignUnplaceBatch::Single
        );

        // Multiple deletion by line
        assert_eq!(
            rs_sign_unplace_batch_mode(0, 10, 0),
            SignUnplaceBatch::Multiple
        );

        // Multiple deletion all
        assert_eq!(
            rs_sign_unplace_batch_mode(0, 0, 1),
            SignUnplaceBatch::Multiple
        );

        // Multiple deletion id=0
        assert_eq!(
            rs_sign_unplace_batch_mode(0, 0, 0),
            SignUnplaceBatch::Multiple
        );
    }

    #[test]
    fn test_sign_jump_result_default() {
        let result = SignJumpResult::default();
        assert_eq!(result.lnum, 0);
        assert!(!result.found);
        assert!(!result.buffer_has_name);
    }

    #[test]
    fn test_sign_jump_found() {
        let result = rs_sign_jump_found(42, 1);
        assert_eq!(result.lnum, 42);
        assert!(result.found);
        assert!(result.buffer_has_name);
    }

    #[test]
    fn test_sign_jump_not_found() {
        let result = rs_sign_jump_not_found();
        assert_eq!(result.lnum, -1);
        assert!(!result.found);
    }

    #[test]
    fn test_sign_jump_target() {
        assert_eq!(rs_sign_jump_target(0, 0, 0), SignJumpTarget::NotFound);
        assert_eq!(rs_sign_jump_target(1, 1, 0), SignJumpTarget::CurrentWindow);
        assert_eq!(rs_sign_jump_target(1, 0, 1), SignJumpTarget::OpenBuffer);
        assert_eq!(rs_sign_jump_target(1, 0, 0), SignJumpTarget::NoName);
    }

    #[test]
    fn test_sign_command_from_index() {
        assert_eq!(SignCommand::from_index(0), Some(SignCommand::Define));
        assert_eq!(SignCommand::from_index(3), Some(SignCommand::Place));
        assert_eq!(SignCommand::from_index(5), Some(SignCommand::Jump));
        assert_eq!(SignCommand::from_index(99), None);
    }
}
