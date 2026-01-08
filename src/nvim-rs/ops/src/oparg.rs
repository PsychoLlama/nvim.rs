//! Operator arguments wrapper
//!
//! This module provides a safe Rust wrapper around the C `oparg_T` struct,
//! using FFI accessor functions to read and write fields.

use std::ffi::c_int;

use crate::types::{MotionType, OpType, Pos};

/// Opaque handle to C's oparg_T struct
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct OpArgHandle(*mut std::ffi::c_void);

impl OpArgHandle {
    /// Create from raw pointer
    ///
    /// # Safety
    /// The pointer must be a valid oparg_T pointer or null.
    #[inline]
    #[must_use]
    pub const unsafe fn from_raw(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get raw pointer
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Check if handle is null
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// FFI accessor functions - these are implemented in C
extern "C" {
    // Getters
    fn nvim_oap_get_op_type_raw(oap: *const std::ffi::c_void) -> c_int;
    fn nvim_oap_get_regname_raw(oap: *const std::ffi::c_void) -> c_int;
    fn nvim_oap_get_motion_type_raw(oap: *const std::ffi::c_void) -> c_int;
    fn nvim_oap_get_motion_force(oap: *const std::ffi::c_void) -> c_int;
    fn nvim_oap_get_inclusive(oap: *const std::ffi::c_void) -> c_int;
    fn nvim_oap_get_use_reg_one(oap: *const std::ffi::c_void) -> c_int;
    fn nvim_oap_get_line_count(oap: *const std::ffi::c_void) -> c_int;
    fn nvim_oap_get_empty(oap: *const std::ffi::c_void) -> c_int;
    fn nvim_oap_get_is_visual(oap: *const std::ffi::c_void) -> c_int;
    fn nvim_oap_get_excl_tr_ws(oap: *const std::ffi::c_void) -> c_int;

    // Position getters
    fn nvim_oap_get_start_lnum(oap: *const std::ffi::c_void) -> c_int;
    fn nvim_oap_get_start_col(oap: *const std::ffi::c_void) -> c_int;
    fn nvim_oap_get_start_coladd(oap: *const std::ffi::c_void) -> c_int;
    fn nvim_oap_get_end_lnum(oap: *const std::ffi::c_void) -> c_int;
    fn nvim_oap_get_end_col(oap: *const std::ffi::c_void) -> c_int;
    fn nvim_oap_get_end_coladd(oap: *const std::ffi::c_void) -> c_int;

    // Virtual column getters
    fn nvim_oap_get_start_vcol(oap: *const std::ffi::c_void) -> c_int;
    fn nvim_oap_get_end_vcol(oap: *const std::ffi::c_void) -> c_int;

    // Setters
    fn nvim_oap_set_op_type(oap: *mut std::ffi::c_void, op_type: c_int);
    fn nvim_oap_set_motion_type(oap: *mut std::ffi::c_void, motion_type: c_int);
    fn nvim_oap_set_inclusive(oap: *mut std::ffi::c_void, inclusive: c_int);
    fn nvim_oap_set_line_count(oap: *mut std::ffi::c_void, line_count: c_int);
    fn nvim_oap_set_empty(oap: *mut std::ffi::c_void, empty: c_int);
}

/// Safe wrapper for reading oparg_T fields
///
/// This provides read-only access to the operator arguments.
pub struct OpArgRef {
    handle: OpArgHandle,
}

impl OpArgRef {
    /// Create a new OpArgRef from a handle
    ///
    /// # Safety
    /// The handle must be valid and not null.
    #[inline]
    #[must_use]
    pub const unsafe fn from_handle(handle: OpArgHandle) -> Self {
        Self { handle }
    }

    /// Get the operator type
    #[inline]
    #[must_use]
    pub fn op_type(&self) -> OpType {
        let raw = unsafe { nvim_oap_get_op_type_raw(self.handle.as_ptr()) };
        OpType::from_raw(raw).unwrap_or(OpType::Nop)
    }

    /// Get the register name
    #[inline]
    #[must_use]
    pub fn regname(&self) -> c_int {
        unsafe { nvim_oap_get_regname_raw(self.handle.as_ptr()) }
    }

    /// Get the motion type
    #[inline]
    #[must_use]
    pub fn motion_type(&self) -> MotionType {
        let raw = unsafe { nvim_oap_get_motion_type_raw(self.handle.as_ptr()) };
        MotionType::from_raw(raw)
    }

    /// Get the motion force character ('v', 'V', or CTRL-V)
    #[inline]
    #[must_use]
    pub fn motion_force(&self) -> c_int {
        unsafe { nvim_oap_get_motion_force(self.handle.as_ptr()) }
    }

    /// Check if the motion is inclusive
    #[inline]
    #[must_use]
    pub fn inclusive(&self) -> bool {
        unsafe { nvim_oap_get_inclusive(self.handle.as_ptr()) != 0 }
    }

    /// Check if delete should use reg 1 even when not linewise
    #[inline]
    #[must_use]
    pub fn use_reg_one(&self) -> bool {
        unsafe { nvim_oap_get_use_reg_one(self.handle.as_ptr()) != 0 }
    }

    /// Get the line count
    #[inline]
    #[must_use]
    pub fn line_count(&self) -> c_int {
        unsafe { nvim_oap_get_line_count(self.handle.as_ptr()) }
    }

    /// Check if op_start and op_end are the same
    #[inline]
    #[must_use]
    pub fn empty(&self) -> bool {
        unsafe { nvim_oap_get_empty(self.handle.as_ptr()) != 0 }
    }

    /// Check if operator is on Visual area
    #[inline]
    #[must_use]
    pub fn is_visual(&self) -> bool {
        unsafe { nvim_oap_get_is_visual(self.handle.as_ptr()) != 0 }
    }

    /// Check if trailing whitespace should be excluded for block yank
    #[inline]
    #[must_use]
    pub fn excl_tr_ws(&self) -> bool {
        unsafe { nvim_oap_get_excl_tr_ws(self.handle.as_ptr()) != 0 }
    }

    /// Get the start position
    #[inline]
    #[must_use]
    pub fn start(&self) -> Pos {
        Pos {
            lnum: unsafe { nvim_oap_get_start_lnum(self.handle.as_ptr()) },
            col: unsafe { nvim_oap_get_start_col(self.handle.as_ptr()) },
            coladd: unsafe { nvim_oap_get_start_coladd(self.handle.as_ptr()) },
        }
    }

    /// Get the end position
    #[inline]
    #[must_use]
    pub fn end(&self) -> Pos {
        Pos {
            lnum: unsafe { nvim_oap_get_end_lnum(self.handle.as_ptr()) },
            col: unsafe { nvim_oap_get_end_col(self.handle.as_ptr()) },
            coladd: unsafe { nvim_oap_get_end_coladd(self.handle.as_ptr()) },
        }
    }

    /// Get the start virtual column (for block mode)
    #[inline]
    #[must_use]
    pub fn start_vcol(&self) -> c_int {
        unsafe { nvim_oap_get_start_vcol(self.handle.as_ptr()) }
    }

    /// Get the end virtual column (for block mode)
    #[inline]
    #[must_use]
    pub fn end_vcol(&self) -> c_int {
        unsafe { nvim_oap_get_end_vcol(self.handle.as_ptr()) }
    }

    /// Check if this is a block-wise operation
    #[inline]
    #[must_use]
    pub fn is_block(&self) -> bool {
        self.motion_type() == MotionType::BlockWise
    }

    /// Check if this is a line-wise operation
    #[inline]
    #[must_use]
    pub fn is_linewise(&self) -> bool {
        self.motion_type() == MotionType::LineWise
    }

    /// Check if this is a character-wise operation
    #[inline]
    #[must_use]
    pub fn is_charwise(&self) -> bool {
        self.motion_type() == MotionType::CharWise
    }
}

/// Safe wrapper for reading and writing oparg_T fields
///
/// This provides mutable access to operator arguments.
pub struct OpArgMut {
    handle: OpArgHandle,
}

impl OpArgMut {
    /// Create a new OpArgMut from a handle
    ///
    /// # Safety
    /// The handle must be valid and not null.
    #[inline]
    #[must_use]
    pub const unsafe fn from_handle(handle: OpArgHandle) -> Self {
        Self { handle }
    }

    /// Get read-only access
    #[inline]
    #[must_use]
    pub fn as_ref(&self) -> OpArgRef {
        OpArgRef {
            handle: self.handle,
        }
    }

    /// Set the operator type
    #[inline]
    pub fn set_op_type(&mut self, op_type: OpType) {
        unsafe { nvim_oap_set_op_type(self.handle.as_ptr(), op_type.as_raw()) }
    }

    /// Set the motion type
    #[inline]
    pub fn set_motion_type(&mut self, motion_type: MotionType) {
        unsafe { nvim_oap_set_motion_type(self.handle.as_ptr(), motion_type.as_raw()) }
    }

    /// Set inclusive flag
    #[inline]
    pub fn set_inclusive(&mut self, inclusive: bool) {
        unsafe { nvim_oap_set_inclusive(self.handle.as_ptr(), c_int::from(inclusive)) }
    }

    /// Set line count
    #[inline]
    pub fn set_line_count(&mut self, line_count: c_int) {
        unsafe { nvim_oap_set_line_count(self.handle.as_ptr(), line_count) }
    }

    /// Set empty flag
    #[inline]
    pub fn set_empty(&mut self, empty: bool) {
        unsafe { nvim_oap_set_empty(self.handle.as_ptr(), c_int::from(empty)) }
    }
}

// Forward common getters from OpArgRef
impl OpArgMut {
    /// Get the operator type
    #[inline]
    #[must_use]
    pub fn op_type(&self) -> OpType {
        self.as_ref().op_type()
    }

    /// Get the register name
    #[inline]
    #[must_use]
    pub fn regname(&self) -> c_int {
        self.as_ref().regname()
    }

    /// Get the motion type
    #[inline]
    #[must_use]
    pub fn motion_type(&self) -> MotionType {
        self.as_ref().motion_type()
    }

    /// Get the motion force character
    #[inline]
    #[must_use]
    pub fn motion_force(&self) -> c_int {
        self.as_ref().motion_force()
    }

    /// Check if the motion is inclusive
    #[inline]
    #[must_use]
    pub fn inclusive(&self) -> bool {
        self.as_ref().inclusive()
    }

    /// Get the line count
    #[inline]
    #[must_use]
    pub fn line_count(&self) -> c_int {
        self.as_ref().line_count()
    }

    /// Check if op_start and op_end are the same
    #[inline]
    #[must_use]
    pub fn empty(&self) -> bool {
        self.as_ref().empty()
    }

    /// Check if operator is on Visual area
    #[inline]
    #[must_use]
    pub fn is_visual(&self) -> bool {
        self.as_ref().is_visual()
    }

    /// Get the start position
    #[inline]
    #[must_use]
    pub fn start(&self) -> Pos {
        self.as_ref().start()
    }

    /// Get the end position
    #[inline]
    #[must_use]
    pub fn end(&self) -> Pos {
        self.as_ref().end()
    }

    /// Get the start virtual column
    #[inline]
    #[must_use]
    pub fn start_vcol(&self) -> c_int {
        self.as_ref().start_vcol()
    }

    /// Get the end virtual column
    #[inline]
    #[must_use]
    pub fn end_vcol(&self) -> c_int {
        self.as_ref().end_vcol()
    }

    /// Check if this is a block-wise operation
    #[inline]
    #[must_use]
    pub fn is_block(&self) -> bool {
        self.as_ref().is_block()
    }

    /// Check if this is a line-wise operation
    #[inline]
    #[must_use]
    pub fn is_linewise(&self) -> bool {
        self.as_ref().is_linewise()
    }

    /// Check if this is a character-wise operation
    #[inline]
    #[must_use]
    pub fn is_charwise(&self) -> bool {
        self.as_ref().is_charwise()
    }
}
