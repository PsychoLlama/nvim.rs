//! Vimscript typval_T operations for Neovim
//!
//! This crate provides Rust implementations of typval-related functions
//! from `src/nvim/eval/typval.c`. It uses an opaque handle pattern where
//! `typval_T*` pointers are treated as opaque handles, with field access
//! done through C accessor functions.
//!
//! ## Phase 28.1: Type Value Foundation
//!
//! This module provides:
//! - Complete `TypeVal` enum with all VimL types (Number, Float, String, List, Dict, Func, Special, Blob, Partial)
//! - Type conversion traits (`From`, `TryFrom`) between Rust and VimL types
//! - Reference counting infrastructure for compound types
//! - FFI exports for type creation and inspection
//!
//! ## Architecture
//!
//! The typval system uses a two-layer approach:
//! 1. **Opaque handles** (`TypevalHandle`, `ListHandle`, etc.) for C interop
//! 2. **Native Rust types** (`TypeVal`, `VimList`, etc.) for pure Rust operations
//!
//! Most operations go through C accessors to avoid struct layout dependencies.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::similar_names)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::redundant_else)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::if_not_else)]
#![allow(clippy::manual_midpoint)]
#![allow(clippy::module_name_repetitions)]

use std::ffi::{c_char, c_int};
use std::fmt;

/// VarType enum values (matching C's VarType in typval_defs.h).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarType {
    Unknown = 0,
    Number = 1,
    String = 2,
    Func = 3,
    List = 4,
    Dict = 5,
    Float = 6,
    Bool = 7,
    Special = 8,
    Partial = 9,
    Blob = 10,
}

impl VarType {
    /// Convert from C integer to VarType.
    #[inline]
    pub const fn from_c_int(v: c_int) -> Option<Self> {
        match v {
            0 => Some(Self::Unknown),
            1 => Some(Self::Number),
            2 => Some(Self::String),
            3 => Some(Self::Func),
            4 => Some(Self::List),
            5 => Some(Self::Dict),
            6 => Some(Self::Float),
            7 => Some(Self::Bool),
            8 => Some(Self::Special),
            9 => Some(Self::Partial),
            10 => Some(Self::Blob),
            _ => None,
        }
    }

    /// Get a human-readable name for this type.
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Number => "Number",
            Self::String => "String",
            Self::Func => "Funcref",
            Self::List => "List",
            Self::Dict => "Dict",
            Self::Float => "Float",
            Self::Bool => "Boolean",
            Self::Special => "Special",
            Self::Partial => "Partial",
            Self::Blob => "Blob",
        }
    }
}

impl fmt::Display for VarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

// =============================================================================
// Bool and Special value types (matching C enums)
// =============================================================================

/// Bool variable values (matching C's BoolVarValue).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoolVarValue {
    /// v:false
    False = 0,
    /// v:true
    True = 1,
}

impl BoolVarValue {
    /// Convert to a Rust bool.
    #[inline]
    pub const fn as_bool(self) -> bool {
        match self {
            Self::False => false,
            Self::True => true,
        }
    }
}

impl From<bool> for BoolVarValue {
    #[inline]
    fn from(b: bool) -> Self {
        if b {
            Self::True
        } else {
            Self::False
        }
    }
}

impl From<BoolVarValue> for bool {
    #[inline]
    fn from(b: BoolVarValue) -> Self {
        b.as_bool()
    }
}

/// Special variable values (matching C's SpecialVarValue).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecialVarValue {
    /// v:null
    Null = 0,
}

// =============================================================================
// Variable lock status (matching C's VarLockStatus)
// =============================================================================

/// Variable lock status for typval_T.v_lock.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VarLockStatus {
    /// Not locked.
    Unlocked = 0,
    /// User lock, can be unlocked.
    Locked = 1,
    /// Locked forever.
    Fixed = 2,
}

impl VarLockStatus {
    /// Convert from C integer.
    #[inline]
    pub const fn from_c_int(v: c_int) -> Option<Self> {
        match v {
            0 => Some(Self::Unlocked),
            1 => Some(Self::Locked),
            2 => Some(Self::Fixed),
            _ => None,
        }
    }

    /// Check if the variable is locked (Locked or Fixed).
    #[inline]
    pub const fn is_locked(self) -> bool {
        !matches!(self, Self::Unlocked)
    }
}

// =============================================================================
// Type constants for type() function return values
// =============================================================================

/// Type values for the VimL `type()` function.
/// These differ from VarType - they're the values returned by type().
#[repr(i64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeFuncValue {
    /// Number type (0)
    Number = 0,
    /// String type (1)
    String = 1,
    /// Funcref type (2) - covers both VAR_FUNC and VAR_PARTIAL
    Func = 2,
    /// List type (3)
    List = 3,
    /// Dict type (4)
    Dict = 4,
    /// Float type (5)
    Float = 5,
    /// Boolean type (6)
    Bool = 6,
    /// Special type (7) - v:null
    Special = 7,
    /// Blob type (10) - note: not contiguous!
    Blob = 10,
}

impl TypeFuncValue {
    /// Convert from VarType to the type() function return value.
    #[inline]
    pub const fn from_var_type(vt: VarType) -> Option<Self> {
        match vt {
            VarType::Unknown => None,
            VarType::Number => Some(Self::Number),
            VarType::String => Some(Self::String),
            VarType::Func | VarType::Partial => Some(Self::Func),
            VarType::List => Some(Self::List),
            VarType::Dict => Some(Self::Dict),
            VarType::Float => Some(Self::Float),
            VarType::Bool => Some(Self::Bool),
            VarType::Special => Some(Self::Special),
            VarType::Blob => Some(Self::Blob),
        }
    }

    /// Convert to i64 for FFI.
    #[inline]
    pub const fn as_i64(self) -> i64 {
        self as i64
    }
}

// =============================================================================
// TypeVal - Native Rust representation of VimL values
// =============================================================================

/// A VimL value represented in native Rust types.
///
/// This enum provides a safe, native Rust interface to VimL values.
/// For FFI, use the handle types (`TypevalHandle`, etc.) instead.
///
/// Note: List, Dict, Blob, Func, and Partial variants hold opaque handles
/// because these are reference-counted types managed by the C runtime.
#[derive(Debug, Clone)]
pub enum TypeVal {
    /// Unknown/uninitialized value
    Unknown,
    /// Integer number (varnumber_T / i64)
    Number(i64),
    /// Floating-point number
    Float(f64),
    /// String value (owned)
    String(std::string::String),
    /// Boolean value (v:true / v:false)
    Bool(BoolVarValue),
    /// Special value (v:null)
    Special(SpecialVarValue),
    /// List (opaque handle - reference counted in C)
    List(ListHandle),
    /// Dictionary (opaque handle - reference counted in C)
    Dict(DictHandle),
    /// Blob (opaque handle - reference counted in C)
    Blob(BlobHandle),
    /// Function reference (name only)
    Func(std::string::String),
    /// Partial function (opaque handle - reference counted in C)
    Partial(PartialHandle),
}

impl TypeVal {
    /// Get the VarType for this value.
    #[inline]
    pub const fn var_type(&self) -> VarType {
        match self {
            Self::Unknown => VarType::Unknown,
            Self::Number(_) => VarType::Number,
            Self::Float(_) => VarType::Float,
            Self::String(_) => VarType::String,
            Self::Bool(_) => VarType::Bool,
            Self::Special(_) => VarType::Special,
            Self::List(_) => VarType::List,
            Self::Dict(_) => VarType::Dict,
            Self::Blob(_) => VarType::Blob,
            Self::Func(_) => VarType::Func,
            Self::Partial(_) => VarType::Partial,
        }
    }

    /// Get the type() function return value for this value.
    #[inline]
    pub fn type_func_value(&self) -> TypeFuncValue {
        TypeFuncValue::from_var_type(self.var_type()).unwrap_or(TypeFuncValue::Special)
    }

    /// Check if this value is "empty" (falsy in VimScript terms).
    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Unknown => true,
            Self::Number(n) => *n == 0,
            Self::Float(f) => *f == 0.0,
            Self::String(s) => s.is_empty(),
            Self::Bool(b) => !b.as_bool(),
            Self::Special(_) => true, // v:null is always empty
            Self::List(l) => l.is_null(),
            Self::Dict(d) => d.is_null(),
            Self::Blob(b) => b.is_null(),
            Self::Func(s) => s.is_empty(),
            Self::Partial(p) => p.is_null(),
        }
    }

    /// Check if this value is "truthy" (non-empty in VimScript terms).
    #[inline]
    pub fn is_truthy(&self) -> bool {
        !self.is_empty()
    }

    /// Try to convert to a number, returning 0 for incompatible types.
    #[inline]
    pub fn to_number(&self) -> i64 {
        match self {
            Self::Number(n) => *n,
            Self::Bool(b) => i64::from(b.as_bool()),
            Self::Float(f) => *f as i64,
            Self::String(s) => s.parse().unwrap_or(0),
            _ => 0,
        }
    }

    /// Try to convert to a float, returning 0.0 for incompatible types.
    #[inline]
    pub fn to_float(&self) -> f64 {
        match self {
            Self::Float(f) => *f,
            Self::Number(n) => *n as f64,
            Self::String(s) => s.parse().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    /// Create an unknown value.
    #[inline]
    pub const fn unknown() -> Self {
        Self::Unknown
    }

    /// Create a number value.
    #[inline]
    pub const fn number(n: i64) -> Self {
        Self::Number(n)
    }

    /// Create a float value.
    #[inline]
    pub const fn float(f: f64) -> Self {
        Self::Float(f)
    }

    /// Create a boolean value.
    #[inline]
    pub const fn bool(b: bool) -> Self {
        Self::Bool(if b {
            BoolVarValue::True
        } else {
            BoolVarValue::False
        })
    }

    /// Create a v:true value.
    #[inline]
    pub const fn vim_true() -> Self {
        Self::Bool(BoolVarValue::True)
    }

    /// Create a v:false value.
    #[inline]
    pub const fn vim_false() -> Self {
        Self::Bool(BoolVarValue::False)
    }

    /// Create a v:null value.
    #[inline]
    pub const fn vim_null() -> Self {
        Self::Special(SpecialVarValue::Null)
    }

    /// Create a string value.
    #[inline]
    pub fn string(s: impl Into<std::string::String>) -> Self {
        Self::String(s.into())
    }
}

impl Default for TypeVal {
    #[inline]
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<i64> for TypeVal {
    #[inline]
    fn from(n: i64) -> Self {
        Self::Number(n)
    }
}

impl From<i32> for TypeVal {
    #[inline]
    fn from(n: i32) -> Self {
        Self::Number(i64::from(n))
    }
}

impl From<f64> for TypeVal {
    #[inline]
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl From<bool> for TypeVal {
    #[inline]
    fn from(b: bool) -> Self {
        Self::bool(b)
    }
}

impl From<std::string::String> for TypeVal {
    #[inline]
    fn from(s: std::string::String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for TypeVal {
    #[inline]
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl fmt::Display for TypeVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "<unknown>"),
            Self::Number(n) => write!(f, "{n}"),
            Self::Float(fl) => write!(f, "{fl}"),
            Self::String(s) => write!(f, "'{s}'"),
            Self::Bool(b) => write!(f, "{}", if b.as_bool() { "v:true" } else { "v:false" }),
            Self::Special(SpecialVarValue::Null) => write!(f, "v:null"),
            Self::List(_) => write!(f, "[...]"),
            Self::Dict(_) => write!(f, "{{...}}"),
            Self::Blob(_) => write!(f, "0z..."),
            Self::Func(name) => write!(f, "function('{name}')"),
            Self::Partial(_) => write!(f, "<partial>"),
        }
    }
}

// =============================================================================
// Opaque handle to a partial_T
// =============================================================================

/// Opaque handle to a Vimscript partial (`partial_T*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartialHandle(*const std::ffi::c_void);

impl PartialHandle {
    /// Create a new partial handle from a raw pointer.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *const std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *const std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// Opaque Handles (existing code follows)
// =============================================================================

/// Opaque handle to a Vimscript typval (`typval_T*`).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypevalHandle(*const std::ffi::c_void);

impl TypevalHandle {
    /// Create a new typval handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `typval_T*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *const std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *const std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// C accessor functions for typval_T fields.
// These will be defined in eval/typval.c
extern "C" {
    /// Get the v_type field from a typval.
    fn nvim_tv_get_type(tv: TypevalHandle) -> c_int;

    /// Get the v_number field from a typval (only valid if v_type == VAR_NUMBER).
    fn nvim_tv_get_number(tv: TypevalHandle) -> i64;

    /// Get the v_bool field from a typval (only valid if v_type == VAR_BOOL).
    fn nvim_tv_get_bool(tv: TypevalHandle) -> c_int;

    /// Get the v_float field from a typval (only valid if v_type == VAR_FLOAT).
    fn nvim_tv_get_float(tv: TypevalHandle) -> f64;

    /// Get the v_string field from a typval (only valid if v_type == VAR_STRING or VAR_FUNC).
    fn nvim_tv_get_string_ptr(tv: TypevalHandle) -> *const c_char;

    /// Check if v_list is NULL (only valid if v_type == VAR_LIST).
    fn nvim_tv_list_is_null(tv: TypevalHandle) -> c_int;

    /// Check if v_dict is NULL (only valid if v_type == VAR_DICT).
    fn nvim_tv_dict_is_null(tv: TypevalHandle) -> c_int;

    /// Check if v_blob is NULL (only valid if v_type == VAR_BLOB).
    fn nvim_tv_blob_is_null(tv: TypevalHandle) -> c_int;

    /// Check if v_partial is NULL (only valid if v_type == VAR_PARTIAL).
    fn nvim_tv_partial_is_null(tv: TypevalHandle) -> c_int;

    /// Set v_type=VAR_NUMBER and v_number (setter for Rust).
    fn nvim_tv_set_number(tv: TypevalHandle, n: i64);
}

// =============================================================================
// Type checking predicates
// =============================================================================

/// Get the VarType of a typval.
#[inline]
fn tv_type_impl(tv: TypevalHandle) -> VarType {
    if tv.is_null() {
        return VarType::Unknown;
    }
    // SAFETY: We check for null above.
    let type_int = unsafe { nvim_tv_get_type(tv) };
    VarType::from_c_int(type_int).unwrap_or(VarType::Unknown)
}

/// FFI wrapper: get the type of a typval.
#[no_mangle]
pub extern "C" fn rs_tv_type(tv: TypevalHandle) -> c_int {
    tv_type_impl(tv) as c_int
}

/// Check if typval is a number.
#[inline]
fn tv_is_number_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Number
}

/// FFI wrapper for tv_is_number.
#[no_mangle]
pub extern "C" fn rs_tv_is_number(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_number_impl(tv))
}

/// Check if typval is a string.
#[inline]
fn tv_is_string_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::String
}

/// FFI wrapper for tv_is_string.
#[no_mangle]
pub extern "C" fn rs_tv_is_string(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_string_impl(tv))
}

/// Check if typval is a float.
#[inline]
fn tv_is_float_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Float
}

/// FFI wrapper for tv_is_float.
#[no_mangle]
pub extern "C" fn rs_tv_is_float(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_float_impl(tv))
}

/// Check if typval is a bool.
#[inline]
fn tv_is_bool_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Bool
}

/// FFI wrapper for tv_is_bool.
#[no_mangle]
pub extern "C" fn rs_tv_is_bool(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_bool_impl(tv))
}

/// Check if typval is a list.
#[inline]
fn tv_is_list_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::List
}

/// FFI wrapper for tv_is_list.
#[no_mangle]
pub extern "C" fn rs_tv_is_list(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_list_impl(tv))
}

/// Check if typval is a dict.
#[inline]
fn tv_is_dict_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Dict
}

/// FFI wrapper for tv_is_dict.
#[no_mangle]
pub extern "C" fn rs_tv_is_dict(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_dict_impl(tv))
}

/// Check if typval is a blob.
#[inline]
fn tv_is_blob_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Blob
}

/// FFI wrapper for tv_is_blob.
#[no_mangle]
pub extern "C" fn rs_tv_is_blob(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_blob_impl(tv))
}

/// Check if typval is a function reference (VAR_FUNC or VAR_PARTIAL).
#[inline]
fn tv_is_func_impl(tv: TypevalHandle) -> bool {
    let t = tv_type_impl(tv);
    t == VarType::Func || t == VarType::Partial
}

/// FFI wrapper for tv_is_func.
#[no_mangle]
pub extern "C" fn rs_tv_is_func(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_func_impl(tv))
}

/// Check if typval is special (null).
#[inline]
fn tv_is_special_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Special
}

/// FFI wrapper for tv_is_special.
#[no_mangle]
pub extern "C" fn rs_tv_is_special(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_special_impl(tv))
}

/// Check if typval has a numeric type (number or float).
#[inline]
fn tv_is_numeric_impl(tv: TypevalHandle) -> bool {
    let t = tv_type_impl(tv);
    t == VarType::Number || t == VarType::Float
}

/// FFI wrapper for tv_is_numeric.
#[no_mangle]
pub extern "C" fn rs_tv_is_numeric(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_numeric_impl(tv))
}

// =============================================================================
// Value extraction (simple - no error reporting)
// =============================================================================

/// Get the number value from a typval (returns 0 if not a number).
#[inline]
fn tv_get_number_simple_impl(tv: TypevalHandle) -> i64 {
    if tv.is_null() {
        return 0;
    }
    let t = tv_type_impl(tv);
    match t {
        VarType::Number => unsafe { nvim_tv_get_number(tv) },
        VarType::Bool => {
            // v_bool: 0 = false, 1 = true
            let b = unsafe { nvim_tv_get_bool(tv) };
            i64::from(b)
        }
        _ => 0,
    }
}

/// FFI wrapper: get number value from typval (0 if not a number).
#[no_mangle]
pub extern "C" fn rs_tv_get_number_simple(tv: TypevalHandle) -> i64 {
    tv_get_number_simple_impl(tv)
}

/// Get the float value from a typval (returns 0.0 if not a float or number).
#[inline]
fn tv_get_float_simple_impl(tv: TypevalHandle) -> f64 {
    if tv.is_null() {
        return 0.0;
    }
    let t = tv_type_impl(tv);
    match t {
        VarType::Float => unsafe { nvim_tv_get_float(tv) },
        VarType::Number => {
            let n = unsafe { nvim_tv_get_number(tv) };
            n as f64
        }
        _ => 0.0,
    }
}

/// FFI wrapper: get float value from typval (0.0 if not a float).
#[no_mangle]
pub extern "C" fn rs_tv_get_float_simple(tv: TypevalHandle) -> f64 {
    tv_get_float_simple_impl(tv)
}

/// Get the bool value from a typval (returns false if not a bool/number).
#[inline]
fn tv_get_bool_simple_impl(tv: TypevalHandle) -> bool {
    if tv.is_null() {
        return false;
    }
    let t = tv_type_impl(tv);
    match t {
        VarType::Bool => {
            let b = unsafe { nvim_tv_get_bool(tv) };
            b != 0
        }
        VarType::Number => {
            let n = unsafe { nvim_tv_get_number(tv) };
            n != 0
        }
        _ => false,
    }
}

/// FFI wrapper: get bool value from typval (false if not a bool).
#[no_mangle]
pub extern "C" fn rs_tv_get_bool_simple(tv: TypevalHandle) -> c_int {
    c_int::from(tv_get_bool_simple_impl(tv))
}

/// Get the string pointer from a typval (returns NULL if not a string).
#[inline]
fn tv_get_string_ptr_impl(tv: TypevalHandle) -> *const c_char {
    if tv.is_null() {
        return std::ptr::null();
    }
    let t = tv_type_impl(tv);
    if t == VarType::String {
        unsafe { nvim_tv_get_string_ptr(tv) }
    } else {
        std::ptr::null()
    }
}

/// FFI wrapper: get string pointer from typval (NULL if not a string).
#[no_mangle]
pub extern "C" fn rs_tv_get_string_ptr(tv: TypevalHandle) -> *const c_char {
    tv_get_string_ptr_impl(tv)
}

// =============================================================================
// Emptiness/truthiness checks
// =============================================================================

/// Check if a typval is "empty" (falsy in Vimscript terms).
///
/// - Numbers: 0 is empty
/// - Strings: empty string or NULL is empty
/// - Lists: NULL or empty list is empty
/// - Dicts: NULL or empty dict is empty
/// - Blobs: NULL or empty blob is empty
/// - Bools: false is empty
/// - Special: always empty (v:null)
/// - Floats: 0.0 is empty
/// - Funcs/Partials: never empty (always truthy if set)
#[inline]
fn tv_is_empty_impl(tv: TypevalHandle) -> bool {
    if tv.is_null() {
        return true;
    }
    let t = tv_type_impl(tv);
    match t {
        VarType::Unknown => true,
        VarType::Number => unsafe { nvim_tv_get_number(tv) == 0 },
        VarType::String => {
            let s = unsafe { nvim_tv_get_string_ptr(tv) };
            if s.is_null() {
                true
            } else {
                // Check if first byte is NUL (empty string)
                unsafe { *s == 0 }
            }
        }
        VarType::Float => unsafe { nvim_tv_get_float(tv) == 0.0 },
        VarType::Bool => unsafe { nvim_tv_get_bool(tv) == 0 },
        VarType::Special => true, // v:null is always empty
        VarType::List => unsafe { nvim_tv_list_is_null(tv) != 0 },
        VarType::Dict => unsafe { nvim_tv_dict_is_null(tv) != 0 },
        VarType::Blob => unsafe { nvim_tv_blob_is_null(tv) != 0 },
        VarType::Func => {
            // Function reference is non-empty if string is set
            let s = unsafe { nvim_tv_get_string_ptr(tv) };
            s.is_null() || unsafe { *s == 0 }
        }
        VarType::Partial => unsafe { nvim_tv_partial_is_null(tv) != 0 },
    }
}

/// FFI wrapper: check if typval is empty/falsy.
#[no_mangle]
pub extern "C" fn rs_tv_is_empty(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_empty_impl(tv))
}

/// Check if a typval is "truthy" (non-empty in Vimscript terms).
#[inline]
fn tv_is_truthy_impl(tv: TypevalHandle) -> bool {
    !tv_is_empty_impl(tv)
}

/// FFI wrapper: check if typval is truthy.
#[no_mangle]
pub extern "C" fn rs_tv_is_truthy(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_truthy_impl(tv))
}

// =============================================================================
// Opaque handle types for list_T, dict_T, blob_T
// =============================================================================

/// Opaque handle to a Vimscript list (`list_T*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListHandle(*const std::ffi::c_void);

impl ListHandle {
    /// Create a null handle.
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null())
    }

    /// Create a new list handle from a raw pointer.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *const std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *const std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a Vimscript dictitem (`dictitem_T*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DictItemHandle(*const std::ffi::c_void);

impl DictItemHandle {
    /// Create a null handle.
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null())
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a Vimscript dict (`dict_T*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DictHandle(*const std::ffi::c_void);

impl DictHandle {
    /// Create a null handle.
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null())
    }

    /// Create a new dict handle from a raw pointer.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *const std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *const std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a Vimscript blob (`blob_T*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobHandle(*const std::ffi::c_void);

impl BlobHandle {
    /// Create a null handle.
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null())
    }

    /// Create a new blob handle from a raw pointer.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *const std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *const std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a list item (`listitem_T*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListItemHandle(*const std::ffi::c_void);

impl ListItemHandle {
    /// Create a null handle.
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null())
    }

    /// Create a new list item handle from a raw pointer.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *const std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *const std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// C accessor functions for list_T, dict_T, blob_T
extern "C" {
    // Typval -> container getters
    fn nvim_tv_get_list(tv: TypevalHandle) -> ListHandle;
    fn nvim_tv_get_dict(tv: TypevalHandle) -> DictHandle;
    fn nvim_tv_get_blob(tv: TypevalHandle) -> BlobHandle;

    // List accessors
    fn nvim_list_get_len(l: ListHandle) -> c_int;
    fn nvim_list_get_lock(l: ListHandle) -> c_int;
    fn nvim_list_has_watchers(l: ListHandle) -> c_int;
    fn nvim_list_get_first(l: ListHandle) -> ListItemHandle;
    fn nvim_list_get_last(l: ListHandle) -> ListItemHandle;

    // List cache accessors (for tv_list_find optimization)
    fn nvim_list_get_idx(l: ListHandle) -> c_int;
    fn nvim_list_get_idx_item(l: ListHandle) -> ListItemHandle;
    fn nvim_list_set_idx(l: ListHandle, idx: c_int);
    fn nvim_list_set_idx_item(l: ListHandle, item: ListItemHandle);
    fn nvim_list_get_copyid(l: ListHandle) -> c_int;
    fn nvim_list_get_copylist(l: ListHandle) -> ListHandle;

    // Listitem accessors
    fn nvim_listitem_get_next(li: ListItemHandle) -> ListItemHandle;
    fn nvim_listitem_get_prev(li: ListItemHandle) -> ListItemHandle;
    fn nvim_listitem_get_tv(li: ListItemHandle) -> TypevalHandle;

    // List setters (for mutation operations)
    fn nvim_list_set_first(l: ListHandle, item: ListItemHandle);
    fn nvim_list_set_last(l: ListHandle, item: ListItemHandle);

    // Listitem setters (for mutation operations)
    fn nvim_listitem_set_next(li: ListItemHandle, next: ListItemHandle);
    fn nvim_listitem_set_prev(li: ListItemHandle, prev: ListItemHandle);

    // Dict accessors
    fn nvim_dict_get_ht_used(d: DictHandle) -> usize;
    fn nvim_dict_get_lock(d: DictHandle) -> c_int;
    fn nvim_dict_has_watchers(d: DictHandle) -> c_int;

    // Dict item accessors (Phase 4)
    fn nvim_dictitem_get_tv(di: DictItemHandle) -> TypevalHandle;
    fn nvim_dict_find(d: DictHandle, key: *const c_char, len: isize) -> DictItemHandle;
    fn nvim_tv_get_string_buf(tv: TypevalHandle, buf: *mut c_char) -> *const c_char;
    fn nvim_tv_get_string_buf_chk(tv: TypevalHandle, buf: *mut c_char) -> *const c_char;
    fn tv_copy(from: *const std::ffi::c_void, to: *mut std::ffi::c_void);
    fn nvim_tv_get_number_simple(tv: TypevalHandle) -> i64;
    fn nvim_tv_get_bool_simple(tv: TypevalHandle) -> c_int;

    // Blob accessors
    fn nvim_blob_get_len(b: BlobHandle) -> c_int;
    fn nvim_blob_get_lock(b: BlobHandle) -> c_int;
    fn nvim_blob_get_byte(b: BlobHandle, idx: c_int) -> u8;
    fn nvim_blob_set_byte(b: BlobHandle, idx: c_int, c: u8);

    // Blob mutation accessors (Phase 1)
    fn nvim_blob_get_ga_data(b: BlobHandle) -> *mut u8;
    fn nvim_blob_set_ga_len(b: BlobHandle, len: c_int);
    fn nvim_blob_ga_grow(b: BlobHandle, n: c_int);
    fn nvim_tv_set_blob(tv: TypevalHandle, b: BlobHandle);
    fn nvim_value_check_lock_translated(lock: c_int, name: *const c_char) -> bool;
    fn nvim_semsg_blobidx(idx: i64);
    fn nvim_emsg_blob_wrong_bytes();

    // Functions called by blob ops
    fn tv_blob_alloc() -> BlobHandle;
    fn tv_clear(tv: TypevalHandle);
    fn tv_get_number_chk(tv: TypevalHandle, error: *mut bool) -> i64;

    // Functions for Phase 2
    fn tv_equal(tv1: TypevalHandle, tv2: TypevalHandle, ic: bool) -> bool;
    fn nvim_tv_get_string(tv: TypevalHandle, out_len: *mut usize) -> *const c_char;
    // nvim_semsg_list_index_out_of_range: deleted (Phase 1), use semsg directly
    #[link_name = "semsg"]
    fn semsg_typval(fmt: *const c_char, ...) -> c_int;
    #[link_name = "e_list_index_out_of_range_nr"]
    static e_list_index_out_of_range_nr_tv: [u8; 0];

    // Functions for Phase 3 (tv_get_float, value_check_lock, tv_check_lock)
    fn nvim_emsg_float_funcref();
    fn nvim_emsg_float_string();
    fn nvim_emsg_float_list();
    fn nvim_emsg_float_dict();
    fn nvim_emsg_float_bool();
    fn nvim_emsg_float_special();
    fn nvim_emsg_float_blob();
    fn nvim_emsg_float_unknown();
    fn nvim_value_check_lock(lock: c_int, name: *const c_char, name_len: usize) -> bool;
    fn nvim_tv_get_v_lock(tv: TypevalHandle) -> c_int;

    // Phase 1 accessor helpers for get functions
    fn nvim_format_number(n: i64, buf: *mut c_char, buflen: c_int);
    fn nvim_format_float(f: f64, buf: *mut c_char, buflen: c_int);
    fn nvim_get_bool_var_name(b: c_int) -> *const c_char;
    fn nvim_get_special_var_name(s: c_int) -> *const c_char;
    fn nvim_vim_str2nr(s: *const c_char, out: *mut i64);
    fn nvim_tv_to_lnum_pos(tv: TypevalHandle, ret_fnum: *mut c_int) -> i32;
    fn nvim_did_emsg_check() -> c_int;
    fn nvim_buf_get_ml_line_count(buf: *const std::ffi::c_void) -> i32;
    fn nvim_emsg_get_number_unknown();

    // Phase 2 accessor helpers for blob alloc/free/copy/f_blob2list/f_list2blob
    fn nvim_blob_alloc_impl() -> BlobHandle;
    fn nvim_blob_free_impl(b: BlobHandle);
    fn nvim_blob_dec_refcount(b: BlobHandle) -> c_int;
    fn nvim_blob_set_ga_maxlen(b: BlobHandle, n: c_int);
    fn nvim_blob_xmemdup_ga_data(from: BlobHandle, len: c_int) -> *mut u8;
    fn nvim_blob_set_ga_data(b: BlobHandle, data: *mut u8);
    fn nvim_tv_list_alloc_ret(ret_tv: TypevalHandle, len: isize) -> ListHandle;
    fn nvim_tv_list_append_number(l: *mut std::ffi::c_void, n: c_int);
    fn nvim_blob_ga_append(b: BlobHandle, c: u8);
    fn nvim_blob_ga_clear_only(b: BlobHandle);
    fn nvim_semsg_blob_invalid_value(n: i64);
    fn nvim_tv_set_lock(tv: TypevalHandle, lock: c_int);

    // Phase 3 accessor helpers for dict item alloc/free/add
    fn nvim_dict_item_alloc_len(key: *const c_char, key_len: usize) -> DictItemHandle;
    fn nvim_dict_item_free(item: DictItemHandle);
    fn nvim_dictitem_di_tv(di: DictItemHandle) -> TypevalHandle;
    fn nvim_dict_add_item(d: DictHandle, item: DictItemHandle) -> c_int;
    fn nvim_dict_alloc_impl() -> DictHandle;
    fn nvim_dict_inc_refcount(d: DictHandle);
    fn nvim_tv_set_dict(tv: TypevalHandle, d: DictHandle);
    fn nvim_tv_dict_alloc_ret(ret_tv: TypevalHandle);
    fn nvim_list_ref(l: ListHandle);
    fn nvim_func_ref(name: *mut c_char);
    fn nvim_xmemdupz(s: *const c_char, len: usize) -> *mut c_char;
    fn nvim_ufunc_get_name(fp: *const std::ffi::c_void) -> *const c_char;
    fn nvim_ufunc_get_namelen(fp: *const std::ffi::c_void) -> usize;
    fn nvim_tv_copy(from: TypevalHandle, to: TypevalHandle);
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_xstrndup(s: *const c_char, len: usize) -> *mut c_char;

    // Phase 3 additional setters
    fn nvim_tv_set_type(tv: TypevalHandle, v_type: c_int);
    fn nvim_tv_set_list(tv: TypevalHandle, l: ListHandle);
    fn nvim_tv_set_bool(tv: TypevalHandle, val: c_int);
    fn nvim_tv_set_float(tv: TypevalHandle, f: f64);
    fn nvim_dict_set_lock(d: DictHandle, lock: c_int);
    fn nvim_dict_remove_key(d: DictHandle, key: *const c_char);
    fn nvim_tv_set_string(tv: TypevalHandle, s: *mut c_char);
    fn nvim_dictitem_get_key(di: DictItemHandle) -> *const c_char;

    // Phase 5: list item alloc/append infrastructure
    fn nvim_list_item_alloc() -> ListItemHandle;
    fn nvim_tv_list_append_item(l: ListHandle, item: ListItemHandle);
    fn nvim_list_inc_len(l: ListHandle);
    fn nvim_list_dec_len(l: ListHandle);
    fn nvim_list_set_len(l: ListHandle, len: c_int);
    fn nvim_list_dec_refcount(l: ListHandle) -> c_int;

    // Phase 5: list alloc/free/watcher infrastructure
    fn nvim_list_get_watch(l: ListHandle) -> *mut std::ffi::c_void;
    fn nvim_list_set_watch(l: ListHandle, lw: *mut std::ffi::c_void);
    fn nvim_listwatch_get_next(lw: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn nvim_listwatch_set_next(lw: *mut std::ffi::c_void, next: *mut std::ffi::c_void);
    fn nvim_get_tv_in_free_unref_items() -> c_int;
    fn nvim_list_alloc_impl() -> ListHandle;
    fn nvim_list_free_list_impl(l: ListHandle);
    fn nvim_list_watch_fix(l: ListHandle, item: ListItemHandle);
    fn nvim_list_item_clear_free(li: ListItemHandle);
    fn nvim_tv_clear(tv: TypevalHandle);
    fn nvim_list_item_free(li: ListItemHandle);
    fn nvim_list_init_static_impl(l: ListHandle);
    fn nvim_list_copy_shallow(l: ListHandle) -> ListHandle;
    fn nvim_tv_set_list_vval(tv: TypevalHandle, l: ListHandle);

    // Phase 6c accessor helpers for slice/range/flatten/remove
    fn nvim_tv_list_set_ret(tv: TypevalHandle, l: ListHandle);
    fn nvim_xfree(ptr: *mut std::ffi::c_void);
    fn nvim_got_int() -> c_int;
    fn nvim_fast_breakcheck();
    fn nvim_emsg_invrange();
    fn nvim_tv_list_index_into_rettv(rettv: TypevalHandle, item: ListItemHandle);
    fn nvim_tv_listitem_move_to_rettv(rettv: TypevalHandle, item: ListItemHandle);
}

// =============================================================================
// List operations
// =============================================================================

/// Get the number of items in a list.
/// Returns 0 if the list is NULL.
#[inline]
fn tv_list_len_impl(l: ListHandle) -> c_int {
    if l.is_null() {
        return 0;
    }
    unsafe { nvim_list_get_len(l) }
}

/// FFI wrapper: get list length.
#[no_mangle]
pub extern "C" fn rs_tv_list_len(l: ListHandle) -> c_int {
    tv_list_len_impl(l)
}

/// Get list lock status.
/// Returns VAR_FIXED (2) for NULL lists.
#[inline]
fn tv_list_locked_impl(l: ListHandle) -> c_int {
    if l.is_null() {
        return 2; // VAR_FIXED
    }
    unsafe { nvim_list_get_lock(l) }
}

/// FFI wrapper: get list lock status.
#[no_mangle]
pub extern "C" fn rs_tv_list_locked(l: ListHandle) -> c_int {
    tv_list_locked_impl(l)
}

/// Check whether list has watchers.
#[inline]
fn tv_list_has_watchers_impl(l: ListHandle) -> bool {
    if l.is_null() {
        return false;
    }
    unsafe { nvim_list_has_watchers(l) != 0 }
}

/// FFI wrapper: check if list has watchers.
#[no_mangle]
pub extern "C" fn rs_tv_list_has_watchers(l: ListHandle) -> c_int {
    c_int::from(tv_list_has_watchers_impl(l))
}

/// Get first list item.
/// Returns NULL for NULL or empty lists.
#[inline]
fn tv_list_first_impl(l: ListHandle) -> ListItemHandle {
    if l.is_null() {
        return ListItemHandle(std::ptr::null());
    }
    unsafe { nvim_list_get_first(l) }
}

/// FFI wrapper: get first list item.
#[no_mangle]
pub extern "C" fn rs_tv_list_first(l: ListHandle) -> ListItemHandle {
    tv_list_first_impl(l)
}

/// Get last list item.
/// Returns NULL for NULL or empty lists.
#[inline]
fn tv_list_last_impl(l: ListHandle) -> ListItemHandle {
    if l.is_null() {
        return ListItemHandle(std::ptr::null());
    }
    unsafe { nvim_list_get_last(l) }
}

/// FFI wrapper: get last list item.
#[no_mangle]
pub extern "C" fn rs_tv_list_last(l: ListHandle) -> ListItemHandle {
    tv_list_last_impl(l)
}

/// Normalize list index: return either -1 or non-negative index.
#[inline]
fn tv_list_uidx_impl(l: ListHandle, mut n: c_int) -> c_int {
    let len = tv_list_len_impl(l);
    // Negative index is relative to the end.
    if n < 0 {
        n += len;
    }
    // Check for index out of range.
    if n < 0 || n >= len {
        return -1;
    }
    n
}

/// FFI wrapper: normalize list index.
#[no_mangle]
pub extern "C" fn rs_tv_list_uidx(l: ListHandle, n: c_int) -> c_int {
    tv_list_uidx_impl(l, n)
}

/// Get copy ID of a list (used for cycle detection during copy).
#[inline]
fn tv_list_copyid_impl(l: ListHandle) -> c_int {
    if l.is_null() {
        return 0;
    }
    unsafe { nvim_list_get_copyid(l) }
}

/// FFI wrapper: get list copy ID.
#[no_mangle]
pub extern "C" fn rs_tv_list_copyid(l: ListHandle) -> c_int {
    tv_list_copyid_impl(l)
}

/// Get the latest copy of a list (set during tv_list_copy).
#[inline]
fn tv_list_latest_copy_impl(l: ListHandle) -> ListHandle {
    if l.is_null() {
        return ListHandle(std::ptr::null());
    }
    unsafe { nvim_list_get_copylist(l) }
}

/// FFI wrapper: get list's latest copy.
#[no_mangle]
pub extern "C" fn rs_tv_list_latest_copy(l: ListHandle) -> ListHandle {
    tv_list_latest_copy_impl(l)
}

// =============================================================================
// Listitem operations
// =============================================================================

/// Get next list item.
#[inline]
fn tv_listitem_next_impl(li: ListItemHandle) -> ListItemHandle {
    if li.is_null() {
        return ListItemHandle(std::ptr::null());
    }
    unsafe { nvim_listitem_get_next(li) }
}

/// FFI wrapper: get next list item.
#[no_mangle]
pub extern "C" fn rs_tv_listitem_next(li: ListItemHandle) -> ListItemHandle {
    tv_listitem_next_impl(li)
}

/// Get previous list item.
#[inline]
fn tv_listitem_prev_impl(li: ListItemHandle) -> ListItemHandle {
    if li.is_null() {
        return ListItemHandle(std::ptr::null());
    }
    unsafe { nvim_listitem_get_prev(li) }
}

/// FFI wrapper: get previous list item.
#[no_mangle]
pub extern "C" fn rs_tv_listitem_prev(li: ListItemHandle) -> ListItemHandle {
    tv_listitem_prev_impl(li)
}

/// Get typval from list item.
#[inline]
fn tv_listitem_tv_impl(li: ListItemHandle) -> TypevalHandle {
    if li.is_null() {
        return TypevalHandle(std::ptr::null());
    }
    unsafe { nvim_listitem_get_tv(li) }
}

/// FFI wrapper: get typval from list item.
#[no_mangle]
pub extern "C" fn rs_tv_listitem_tv(li: ListItemHandle) -> TypevalHandle {
    tv_listitem_tv_impl(li)
}

// =============================================================================
// List find operation (tv_list_find)
// =============================================================================

/// Find list item at index n.
///
/// This is a full implementation of `tv_list_find` from C.
/// It uses the list's cached index for optimization.
#[inline]
fn tv_list_find_impl(l: ListHandle, n: c_int) -> ListItemHandle {
    if l.is_null() {
        return ListItemHandle(std::ptr::null());
    }

    // Normalize index
    let n = tv_list_uidx_impl(l, n);
    if n == -1 {
        return ListItemHandle(std::ptr::null());
    }

    let len = tv_list_len_impl(l);
    let cached_item = unsafe { nvim_list_get_idx_item(l) };
    let cached_idx = unsafe { nvim_list_get_idx(l) };

    let (mut item, mut idx) = if !cached_item.is_null() {
        // Use cached index for optimization
        if n < cached_idx / 2 {
            // Closest to start
            (tv_list_first_impl(l), 0)
        } else if n > (cached_idx + len) / 2 {
            // Closest to end
            (tv_list_last_impl(l), len - 1)
        } else {
            // Closest to cached
            (cached_item, cached_idx)
        }
    } else {
        // No cache, choose start or end
        if n < len / 2 {
            (tv_list_first_impl(l), 0)
        } else {
            (tv_list_last_impl(l), len - 1)
        }
    };

    // Search forward
    while n > idx {
        item = tv_listitem_next_impl(item);
        idx += 1;
    }

    // Search backward
    while n < idx {
        item = tv_listitem_prev_impl(item);
        idx -= 1;
    }

    // Update cache
    unsafe {
        nvim_list_set_idx(l, idx);
        nvim_list_set_idx_item(l, item);
    }

    item
}

/// FFI wrapper: find list item at index.
#[export_name = "tv_list_find"]
pub extern "C" fn rs_tv_list_find(l: ListHandle, n: c_int) -> ListItemHandle {
    tv_list_find_impl(l, n)
}

/// Get the index of a list item within a list.
/// Returns -1 if the list is NULL or item is not in the list.
#[inline]
fn tv_list_idx_of_item_impl(l: ListHandle, item: ListItemHandle) -> c_int {
    if l.is_null() {
        return -1;
    }

    let mut idx = 0;
    let mut li = tv_list_first_impl(l);
    while !li.is_null() {
        if li.0 == item.0 {
            return idx;
        }
        li = tv_listitem_next_impl(li);
        idx += 1;
    }
    -1
}

/// FFI wrapper: get index of list item in list.
#[export_name = "tv_list_idx_of_item"]
pub extern "C" fn rs_tv_list_idx_of_item(l: ListHandle, item: ListItemHandle) -> c_int {
    tv_list_idx_of_item_impl(l, item)
}

/// Reverse a list in-place by swapping next/prev pointers.
#[inline]
fn tv_list_reverse_impl(l: ListHandle) {
    if l.is_null() {
        return;
    }

    let len = tv_list_len_impl(l);
    if len <= 1 {
        return;
    }

    // Swap lv_first and lv_last
    let first = tv_list_first_impl(l);
    let last = tv_list_last_impl(l);
    unsafe {
        nvim_list_set_first(l, last);
        nvim_list_set_last(l, first);
    }

    // Iterate through and swap li_next and li_prev for each item.
    // After swapping first/last, lv_first now points to old last.
    // We traverse using li_next AFTER swapping it (which points to old li_prev).
    let mut li = unsafe { nvim_list_get_first(l) };
    while !li.is_null() {
        let next = tv_listitem_next_impl(li);
        let prev = tv_listitem_prev_impl(li);
        unsafe {
            nvim_listitem_set_next(li, prev);
            nvim_listitem_set_prev(li, next);
        }
        // After swap, li_next now points to what was li_prev.
        // We need to follow that to continue "backwards" through original list.
        li = tv_listitem_next_impl(li);
    }

    // Update the cached index: new_idx = len - old_idx - 1
    let old_idx = unsafe { nvim_list_get_idx(l) };
    let new_idx = len - old_idx - 1;
    unsafe {
        nvim_list_set_idx(l, new_idx);
    }
}

/// FFI wrapper: reverse list in-place.
#[export_name = "tv_list_reverse"]
pub extern "C" fn rs_tv_list_reverse(l: ListHandle) {
    tv_list_reverse_impl(l);
}

// =============================================================================
// List append operations (Phase 5)
// =============================================================================

/// FFI export: tv_list_append - append item to end of list.
#[export_name = "tv_list_append"]
pub unsafe extern "C" fn rs_tv_list_append(l: ListHandle, item: ListItemHandle) {
    unsafe { nvim_tv_list_append_item(l, item) };
}

/// FFI export: tv_list_append_tv - append a copy of typval to list.
#[export_name = "tv_list_append_tv"]
pub unsafe extern "C" fn rs_tv_list_append_tv(l: ListHandle, tv: TypevalHandle) {
    let li = unsafe { nvim_list_item_alloc() };
    let li_tv = unsafe { nvim_listitem_get_tv(li) };
    unsafe { nvim_tv_copy(tv, li_tv) };
    unsafe { nvim_tv_list_append_item(l, li) };
}

/// FFI export: tv_list_append_list - append a list as one item.
#[export_name = "tv_list_append_list"]
pub unsafe extern "C" fn rs_tv_list_append_list(l: ListHandle, itemlist: ListHandle) {
    let li = unsafe { nvim_list_item_alloc() };
    let li_tv = unsafe { nvim_listitem_get_tv(li) };
    unsafe { nvim_tv_set_type(li_tv, VAR_LIST) };
    unsafe { nvim_tv_set_lock(li_tv, VarLockStatus::Unlocked as c_int) };
    unsafe { nvim_tv_set_list(li_tv, itemlist) };
    unsafe { nvim_list_ref(itemlist) };
    unsafe { nvim_tv_list_append_item(l, li) };
}

/// FFI export: tv_list_append_dict - append a dict as one item.
#[export_name = "tv_list_append_dict"]
pub unsafe extern "C" fn rs_tv_list_append_dict(l: ListHandle, dict: DictHandle) {
    let li = unsafe { nvim_list_item_alloc() };
    let li_tv = unsafe { nvim_listitem_get_tv(li) };
    unsafe { nvim_tv_set_type(li_tv, VAR_DICT) };
    unsafe { nvim_tv_set_lock(li_tv, VarLockStatus::Unlocked as c_int) };
    unsafe { nvim_tv_set_dict(li_tv, dict) };
    if !dict.is_null() {
        unsafe { nvim_dict_inc_refcount(dict) };
    }
    unsafe { nvim_tv_list_append_item(l, li) };
}

/// FFI export: tv_list_append_string - append a copy of string to list.
#[export_name = "tv_list_append_string"]
pub unsafe extern "C" fn rs_tv_list_append_string(
    l: ListHandle,
    str_ptr: *const c_char,
    len: isize,
) {
    let s: *mut c_char = if str_ptr.is_null() {
        std::ptr::null_mut()
    } else if len >= 0 {
        unsafe { nvim_xmemdupz(str_ptr, len as usize) }
    } else {
        unsafe { nvim_xstrdup(str_ptr) }
    };
    let li = unsafe { nvim_list_item_alloc() };
    let li_tv = unsafe { nvim_listitem_get_tv(li) };
    unsafe { nvim_tv_set_type(li_tv, VAR_STRING) };
    unsafe { nvim_tv_set_lock(li_tv, VarLockStatus::Unlocked as c_int) };
    unsafe { nvim_tv_set_string(li_tv, s) };
    unsafe { nvim_tv_list_append_item(l, li) };
}

/// FFI export: tv_list_append_allocated_string - append owned string to list.
#[export_name = "tv_list_append_allocated_string"]
pub unsafe extern "C" fn rs_tv_list_append_allocated_string(l: ListHandle, str_ptr: *mut c_char) {
    let li = unsafe { nvim_list_item_alloc() };
    let li_tv = unsafe { nvim_listitem_get_tv(li) };
    unsafe { nvim_tv_set_type(li_tv, VAR_STRING) };
    unsafe { nvim_tv_set_lock(li_tv, VarLockStatus::Unlocked as c_int) };
    unsafe { nvim_tv_set_string(li_tv, str_ptr) };
    unsafe { nvim_tv_list_append_item(l, li) };
}

/// FFI export: tv_list_append_number - append number to list.
#[export_name = "tv_list_append_number"]
pub unsafe extern "C" fn rs_tv_list_append_number(l: ListHandle, n: i64) {
    let li = unsafe { nvim_list_item_alloc() };
    let li_tv = unsafe { nvim_listitem_get_tv(li) };
    unsafe { nvim_tv_set_type(li_tv, VAR_NUMBER) };
    unsafe { nvim_tv_set_lock(li_tv, VarLockStatus::Unlocked as c_int) };
    unsafe { nvim_tv_set_number(li_tv, n) };
    unsafe { nvim_tv_list_append_item(l, li) };
}

// =============================================================================
// Phase 5: List infrastructure (alloc, free, watch, item remove, insert)
// =============================================================================

/// FFI export: tv_list_item_remove - remove and free list item.
/// Returns the item that was after the removed one, or NULL.
#[export_name = "tv_list_item_remove"]
pub unsafe extern "C" fn rs_tv_list_item_remove(
    l: ListHandle,
    item: ListItemHandle,
) -> ListItemHandle {
    let next_item = unsafe { nvim_listitem_get_next(item) };
    rs_tv_list_drop_items(l, item, item);
    unsafe { nvim_list_item_clear_free(item) };
    next_item
}

/// FFI export: tv_list_watch_add - prepend a watcher to the list's watch list.
#[export_name = "tv_list_watch_add"]
pub unsafe extern "C" fn rs_tv_list_watch_add(l: ListHandle, lw: *mut std::ffi::c_void) {
    let old_watch = unsafe { nvim_list_get_watch(l) };
    unsafe { nvim_listwatch_set_next(lw, old_watch) };
    unsafe { nvim_list_set_watch(l, lw) };
}

/// FFI export: tv_list_watch_remove - remove a watcher from the list's watch list.
#[export_name = "tv_list_watch_remove"]
pub unsafe extern "C" fn rs_tv_list_watch_remove(l: ListHandle, lwrem: *mut std::ffi::c_void) {
    // We walk via raw pointer equality: scan lv_watch chain looking for lwrem.
    // Use a pointer to the pointer: start with &l->lv_watch.
    // Since we can't take a Rust reference to C-allocated memory easily,
    // we track prev and current.
    let mut prev: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut lw = unsafe { nvim_list_get_watch(l) };
    while !lw.is_null() {
        if std::ptr::eq(lw, lwrem) {
            let next = unsafe { nvim_listwatch_get_next(lw) };
            if prev.is_null() {
                unsafe { nvim_list_set_watch(l, next) };
            } else {
                unsafe { nvim_listwatch_set_next(prev, next) };
            }
            break;
        }
        prev = lw;
        lw = unsafe { nvim_listwatch_get_next(lw) };
    }
}

/// FFI export: tv_list_alloc - allocate an empty list.
#[export_name = "tv_list_alloc"]
pub unsafe extern "C" fn rs_tv_list_alloc(_len: isize) -> ListHandle {
    unsafe { nvim_list_alloc_impl() }
}

/// FFI export: tv_list_init_static - initialize a static list (DO_NOT_FREE_CNT).
#[export_name = "tv_list_init_static"]
pub unsafe extern "C" fn rs_tv_list_init_static(l: ListHandle) {
    unsafe { nvim_list_init_static_impl(l) };
}

/// FFI export: tv_list_free_contents - free all items in a list.
#[export_name = "tv_list_free_contents"]
pub unsafe extern "C" fn rs_tv_list_free_contents(l: ListHandle) {
    loop {
        let item = unsafe { nvim_list_get_first(l) };
        if item.is_null() {
            break;
        }
        let next = unsafe { nvim_listitem_get_next(item) };
        unsafe { nvim_list_set_first(l, next) };
        let li_tv = unsafe { nvim_listitem_get_tv(item) };
        unsafe { nvim_tv_clear(li_tv) };
        unsafe { nvim_list_item_free(item) };
    }
    unsafe { nvim_list_set_len(l, 0) };
    unsafe { nvim_list_set_idx_item(l, ListItemHandle::null()) };
    unsafe { nvim_list_set_last(l, ListItemHandle::null()) };
}

/// FFI export: tv_list_free_list - free the list struct itself (removes from GC).
#[export_name = "tv_list_free_list"]
pub unsafe extern "C" fn rs_tv_list_free_list(l: ListHandle) {
    unsafe { nvim_list_free_list_impl(l) };
}

/// FFI export: tv_list_free - free a list including its contents.
#[export_name = "tv_list_free"]
pub unsafe extern "C" fn rs_tv_list_free(l: ListHandle) {
    if unsafe { nvim_get_tv_in_free_unref_items() } != 0 {
        return;
    }
    unsafe { rs_tv_list_free_contents(l) };
    unsafe { rs_tv_list_free_list(l) };
}

/// FFI export: tv_list_unref - decrement refcount and free if <= 0.
#[export_name = "tv_list_unref"]
pub unsafe extern "C" fn rs_tv_list_unref(l: ListHandle) {
    if l.is_null() {
        return;
    }
    let new_rc = unsafe { nvim_list_dec_refcount(l) };
    if new_rc <= 0 {
        unsafe { rs_tv_list_free(l) };
    }
}

/// FFI export: tv_list_drop_items - unlink items item..item2 from list (does NOT free).
#[export_name = "tv_list_drop_items"]
pub unsafe extern "C" fn rs_tv_list_drop_items(
    l: ListHandle,
    item: ListItemHandle,
    item2: ListItemHandle,
) {
    // Notify watchers and decrement len for each item being dropped.
    let mut ip = item;
    loop {
        unsafe { nvim_list_dec_len(l) };
        unsafe { nvim_list_watch_fix(l, ip) };
        let next_ip = unsafe { nvim_listitem_get_next(ip) };
        let next_item2 = unsafe { nvim_listitem_get_next(item2) };
        if std::ptr::eq(ip.as_ptr(), item2.as_ptr()) {
            break;
        }
        // Advance: ip = ip->li_next (but stop when ip == item2)
        // The loop condition is ip != item2->li_next
        // Actually we need: for ip in item..=item2
        // The C does: for ip = item; ip != item2->li_next; ip = ip->li_next
        // We already handle ip == item2 case above, so advance to next.
        let _ = next_item2; // not used here
        ip = next_ip;
    }

    let next_item2 = unsafe { nvim_listitem_get_next(item2) };
    let prev_item = unsafe { nvim_listitem_get_prev(item) };

    if next_item2.is_null() {
        // item2 was last; update lv_last to item->li_prev
        unsafe { nvim_list_set_last(l, prev_item) };
    } else {
        // item2->li_next->li_prev = item->li_prev
        unsafe { nvim_listitem_set_prev(next_item2, prev_item) };
    }

    if prev_item.is_null() {
        // item was first; update lv_first to item2->li_next
        unsafe { nvim_list_set_first(l, next_item2) };
    } else {
        // item->li_prev->li_next = item2->li_next
        unsafe { nvim_listitem_set_next(prev_item, next_item2) };
    }

    unsafe { nvim_list_set_idx_item(l, ListItemHandle::null()) };
}

/// FFI export: tv_list_remove_items - unlink and free items item..item2.
#[export_name = "tv_list_remove_items"]
pub unsafe extern "C" fn rs_tv_list_remove_items(
    l: ListHandle,
    item: ListItemHandle,
    item2: ListItemHandle,
) {
    unsafe { rs_tv_list_drop_items(l, item, item2) };
    let mut li = item;
    loop {
        let li_tv = unsafe { nvim_listitem_get_tv(li) };
        unsafe { nvim_tv_clear(li_tv) };
        let nli = unsafe { nvim_listitem_get_next(li) };
        if std::ptr::eq(li.as_ptr(), item2.as_ptr()) {
            unsafe { nvim_list_item_free(li) };
            break;
        }
        unsafe { nvim_list_item_free(li) };
        li = nli;
    }
}

/// FFI export: tv_list_move_items - move items from l to end of tgt_l.
#[export_name = "tv_list_move_items"]
pub unsafe extern "C" fn rs_tv_list_move_items(
    l: ListHandle,
    item: ListItemHandle,
    item2: ListItemHandle,
    tgt_l: ListHandle,
    cnt: c_int,
) {
    unsafe { rs_tv_list_drop_items(l, item, item2) };

    let tgt_last = unsafe { nvim_list_get_last(tgt_l) };
    unsafe { nvim_listitem_set_prev(item, tgt_last) };
    unsafe { nvim_listitem_set_next(item2, ListItemHandle::null()) };

    if tgt_last.is_null() {
        unsafe { nvim_list_set_first(tgt_l, item) };
    } else {
        unsafe { nvim_listitem_set_next(tgt_last, item) };
    }
    unsafe { nvim_list_set_last(tgt_l, item2) };

    let tgt_len = unsafe { nvim_list_get_len(tgt_l) };
    unsafe { nvim_list_set_len(tgt_l, tgt_len + cnt) };
}

/// FFI export: tv_list_insert - insert item before another item (NULL = append).
#[export_name = "tv_list_insert"]
pub unsafe extern "C" fn rs_tv_list_insert(
    l: ListHandle,
    ni: ListItemHandle,
    item: ListItemHandle,
) {
    if item.is_null() {
        // Append at end.
        unsafe { rs_tv_list_append(l, ni) };
    } else {
        // Insert before item.
        let prev = unsafe { nvim_listitem_get_prev(item) };
        unsafe { nvim_listitem_set_prev(ni, prev) };
        unsafe { nvim_listitem_set_next(ni, item) };
        if prev.is_null() {
            unsafe { nvim_list_set_first(l, ni) };
            let idx = unsafe { nvim_list_get_idx(l) };
            unsafe { nvim_list_set_idx(l, idx + 1) };
        } else {
            unsafe { nvim_listitem_set_next(prev, ni) };
            unsafe { nvim_list_set_idx_item(l, ListItemHandle::null()) };
        }
        unsafe { nvim_listitem_set_prev(item, ni) };
        unsafe { nvim_list_inc_len(l) };
    }
}

/// FFI export: tv_list_insert_tv - insert a copy of tv before item.
#[export_name = "tv_list_insert_tv"]
pub unsafe extern "C" fn rs_tv_list_insert_tv(
    l: ListHandle,
    tv: TypevalHandle,
    item: ListItemHandle,
) {
    let ni = unsafe { nvim_list_item_alloc() };
    let ni_tv = unsafe { nvim_listitem_get_tv(ni) };
    unsafe { nvim_tv_copy(tv, ni_tv) };
    unsafe { rs_tv_list_insert(l, ni, item) };
}

/// FFI export: tv_list_alloc_ret - alloc list and set as return value.
#[export_name = "tv_list_alloc_ret"]
pub unsafe extern "C" fn rs_tv_list_alloc_ret(ret_tv: TypevalHandle, len: isize) -> ListHandle {
    let l = unsafe { rs_tv_list_alloc(len) };
    unsafe { nvim_list_ref(l) };
    unsafe { nvim_tv_set_list(ret_tv, l) };
    l
}

// =============================================================================
// Phase 6: List operations and VimL functions
// =============================================================================

/// FFI export: tv_list_extend - extend l1 with items from l2, inserting before bef.
///
/// If bef is NULL, items are appended at the end.
/// Stops after the original item count of l2 to prevent infinite loop if l1 == l2.
#[export_name = "tv_list_extend"]
pub unsafe extern "C" fn rs_tv_list_extend(l1: ListHandle, l2: ListHandle, bef: ListItemHandle) {
    let mut todo = unsafe { nvim_list_get_len(l2) };
    // Save the item just before `bef` and its next pointer to handle the case
    // where we're inserting into the middle of l1 and l2 == l1 (self-extend).
    let befbef = if bef.is_null() {
        ListItemHandle::null()
    } else {
        unsafe { nvim_listitem_get_prev(bef) }
    };
    let saved_next = if befbef.is_null() {
        ListItemHandle::null()
    } else {
        unsafe { nvim_listitem_get_next(befbef) }
    };

    let mut item = unsafe { nvim_list_get_first(l2) };
    while !item.is_null() && todo > 0 {
        todo -= 1;
        let li_tv = unsafe { nvim_listitem_get_tv(item) };
        unsafe { rs_tv_list_insert_tv(l1, li_tv, bef) };
        // Advance: if item == befbef, skip to saved_next to avoid re-visiting
        // already-inserted items (relevant when extending list with itself).
        item = if std::ptr::eq(item.as_ptr(), befbef.as_ptr()) {
            saved_next
        } else {
            unsafe { nvim_listitem_get_next(item) }
        };
    }
}

/// FFI export: tv_list_concat - concatenate two lists into a new list stored in tv.
#[export_name = "tv_list_concat"]
pub unsafe extern "C" fn rs_tv_list_concat(
    l1: ListHandle,
    l2: ListHandle,
    tv: TypevalHandle,
) -> c_int {
    // Set return type up front.
    unsafe { nvim_tv_set_type(tv, VAR_LIST) };
    unsafe { nvim_tv_set_lock(tv, VarLockStatus::Unlocked as c_int) };

    let l = if l1.is_null() && l2.is_null() {
        ListHandle::null()
    } else if l1.is_null() {
        // Shallow copy of l2 (tv_list_copy(NULL, l2, false, 0)).
        unsafe { nvim_list_copy_shallow(l2) }
    } else {
        // Shallow copy of l1, then extend with l2.
        let copy = unsafe { nvim_list_copy_shallow(l1) };
        if !copy.is_null() && !l2.is_null() {
            unsafe { rs_tv_list_extend(copy, l2, ListItemHandle::null()) };
        }
        copy
    };

    if l.is_null() && !(l1.is_null() && l2.is_null()) {
        return FAIL;
    }

    // Store the list pointer directly (type/lock already set above).
    unsafe { nvim_tv_set_list_vval(tv, l) };
    OK
}

// =============================================================================
// Phase 6c: list range/slice/flatten/remove operations
// =============================================================================

/// Find list item with index fixup: if negative index not found, try index 0.
/// Used as first index of a range.
fn tv_list_find_index_impl(l: ListHandle, idx: &mut c_int) -> ListItemHandle {
    let li = tv_list_find_impl(l, *idx);
    if !li.is_null() {
        return li;
    }
    if *idx < 0 {
        *idx = 0;
        return tv_list_find_impl(l, *idx);
    }
    ListItemHandle::null()
}

/// FFI export: tv_list_check_range_index_one - validate first range index.
#[export_name = "tv_list_check_range_index_one"]
pub unsafe extern "C" fn rs_tv_list_check_range_index_one(
    l: ListHandle,
    n1: *mut c_int,
    quiet: bool,
) -> ListItemHandle {
    let mut idx = unsafe { *n1 };
    let li = tv_list_find_index_impl(l, &mut idx);
    unsafe { *n1 = idx };
    if !li.is_null() {
        return li;
    }
    if !quiet {
        unsafe {
            semsg_typval(
                e_list_index_out_of_range_nr_tv.as_ptr().cast(),
                i64::from(idx),
            )
        };
    }
    ListItemHandle::null()
}

/// FFI export: tv_list_check_range_index_two - validate second range index.
#[export_name = "tv_list_check_range_index_two"]
pub unsafe extern "C" fn rs_tv_list_check_range_index_two(
    l: ListHandle,
    n1: *mut c_int,
    li1: ListItemHandle,
    n2: *mut c_int,
    quiet: bool,
) -> c_int {
    let mut n2_val = unsafe { *n2 };
    if n2_val < 0 {
        let ni = tv_list_find_impl(l, n2_val);
        if ni.is_null() {
            if !quiet {
                unsafe {
                    semsg_typval(
                        e_list_index_out_of_range_nr_tv.as_ptr().cast(),
                        i64::from(n2_val),
                    )
                };
            }
            return FAIL;
        }
        n2_val = tv_list_idx_of_item_impl(l, ni);
    }
    unsafe { *n2 = n2_val };

    // Fix up n1 if negative.
    let mut n1_val = unsafe { *n1 };
    if n1_val < 0 {
        n1_val = tv_list_idx_of_item_impl(l, li1);
        unsafe { *n1 = n1_val };
    }

    if n2_val < n1_val {
        if !quiet {
            unsafe {
                semsg_typval(
                    e_list_index_out_of_range_nr_tv.as_ptr().cast(),
                    i64::from(n2_val),
                )
            };
        }
        return FAIL;
    }
    OK
}

/// Build a new list from items [n1..=n2] of ol.
unsafe fn tv_list_slice_impl(ol: ListHandle, n1: i64, n2: i64) -> ListHandle {
    let l = unsafe { nvim_list_alloc_impl() };
    if l.is_null() {
        return l;
    }
    unsafe { nvim_list_set_len(l, 0) };
    let mut item = tv_list_find_impl(ol, n1 as c_int);
    let mut i = n1;
    while i <= n2 && !item.is_null() {
        let tv = unsafe { nvim_listitem_get_tv(item) };
        unsafe { rs_tv_list_append_tv(l, tv) };
        item = unsafe { nvim_listitem_get_next(item) };
        i += 1;
    }
    l
}

/// FFI export: tv_list_slice_or_index - slice or index into a list.
#[export_name = "tv_list_slice_or_index"]
pub unsafe extern "C" fn rs_tv_list_slice_or_index(
    _list: ListHandle,
    range: bool,
    n1_arg: i64,
    n2_arg: i64,
    exclusive: bool,
    rettv: TypevalHandle,
    verbose: bool,
) -> c_int {
    // NOTE: The C function ignores the `list` parameter and reads from rettv->vval.v_list.
    // We do the same via nvim_tv_get_list.
    let list = unsafe { nvim_tv_get_list(rettv) };
    let len = i64::from(tv_list_len_impl(list));
    let mut n1 = n1_arg;
    let mut n2 = n2_arg;

    if n1 < 0 {
        n1 += len;
    }
    if n1 < 0 || n1 >= len {
        if !range {
            if verbose {
                unsafe { semsg_typval(e_list_index_out_of_range_nr_tv.as_ptr().cast(), n1_arg) };
            }
            return FAIL;
        }
        n1 = len;
    }

    if range {
        if n2 < 0 {
            n2 += len;
        } else if n2 >= len {
            n2 = len - i64::from(!exclusive);
        }
        if exclusive {
            n2 -= 1;
        }
        if n2 < 0 || n2 + 1 < n1 {
            n2 = -1;
        }
        let l = unsafe { tv_list_slice_impl(list, n1, n2) };
        unsafe { nvim_tv_clear(rettv) };
        unsafe { nvim_tv_list_set_ret(rettv, l) };
    } else {
        // Copy item[n1]'s TV into rettv via C accessor (handles stack alloc and clear).
        let item = tv_list_find_impl(list, n1 as c_int);
        unsafe { nvim_tv_list_index_into_rettv(rettv, item) };
    }
    OK
}

/// FFI export: tv_list_flatten - flatten nested lists in-place.
#[export_name = "tv_list_flatten"]
pub unsafe extern "C" fn rs_tv_list_flatten(
    list: ListHandle,
    first: ListItemHandle,
    maxitems: i64,
    maxdepth: i64,
) {
    if maxdepth == 0 {
        return;
    }

    let mut item = if first.is_null() {
        unsafe { nvim_list_get_first(list) }
    } else {
        first
    };

    let mut done: i64 = 0;
    while !item.is_null() && done < maxitems {
        let next = unsafe { nvim_listitem_get_next(item) };

        unsafe { nvim_fast_breakcheck() };
        if unsafe { nvim_got_int() } != 0 {
            return;
        }

        let item_tv = unsafe { nvim_listitem_get_tv(item) };
        let item_type = unsafe { nvim_tv_get_type(item_tv) };
        if item_type == VAR_LIST {
            let itemlist = unsafe { nvim_tv_get_list(item_tv) };
            let itemlist_len = i64::from(tv_list_len_impl(itemlist));

            // Unlink item from list.
            unsafe { rs_tv_list_drop_items(list, item, item) };
            // Insert itemlist's contents before `next`.
            unsafe { rs_tv_list_extend(list, itemlist, next) };

            // Recursively flatten the newly inserted items.
            if maxdepth > 0 {
                let prev_of_next = if next.is_null() {
                    unsafe { nvim_list_get_last(list) }
                } else {
                    unsafe { nvim_listitem_get_prev(next) }
                };
                let recurse_start = if prev_of_next.is_null() {
                    unsafe { nvim_list_get_first(list) }
                } else {
                    unsafe { nvim_listitem_get_next(prev_of_next) }
                };
                unsafe {
                    rs_tv_list_flatten(list, recurse_start, itemlist_len, maxdepth - 1);
                };
            }

            // Free the item (its tv was a list, extended above).
            unsafe { nvim_tv_clear(item_tv) };
            unsafe { nvim_xfree(item.as_ptr().cast_mut()) };
        }

        done += 1;
        item = next;
    }
}

/// FFI export: tv_list_remove - VimL remove() for lists.
#[export_name = "tv_list_remove"]
pub unsafe extern "C" fn rs_tv_list_remove(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    arg_errmsg: *const c_char,
) {
    let arg0 = unsafe { nvim_typval_array_get(argvars, 0) };
    let l = unsafe { nvim_tv_get_list(arg0) };
    let lock = tv_list_locked_impl(l);
    if unsafe { nvim_value_check_lock_translated(lock, arg_errmsg) } {
        return;
    }

    let arg1 = unsafe { nvim_typval_array_get(argvars, 1) };
    let mut error = false;
    let idx = unsafe { tv_get_number_chk(arg1, &raw mut error) };

    if error {
        // Type error: do nothing, errmsg already given.
        return;
    }

    let item = tv_list_find_impl(l, idx as c_int);
    if item.is_null() {
        unsafe { semsg_typval(e_list_index_out_of_range_nr_tv.as_ptr().cast(), idx) };
        return;
    }

    let arg2 = unsafe { nvim_typval_array_get(argvars, 2) };
    let arg2_type = unsafe { nvim_tv_get_type(arg2) };
    if arg2_type == VAR_UNKNOWN {
        // Remove one item, return its value (bitwise move via C accessor).
        unsafe { rs_tv_list_drop_items(l, item, item) };
        unsafe { nvim_tv_listitem_move_to_rettv(rettv, item) };
    } else {
        // Remove range of items, return list with values.
        let end = unsafe { tv_get_number_chk(arg2, &raw mut error) };
        if error {
            return;
        }
        let item2 = tv_list_find_impl(l, end as c_int);
        if item2.is_null() {
            unsafe { semsg_typval(e_list_index_out_of_range_nr_tv.as_ptr().cast(), end) };
            return;
        }

        // Count items from item to item2 inclusive.
        let mut cnt: c_int = 0;
        let mut li = item;
        loop {
            cnt += 1;
            if std::ptr::eq(li.as_ptr(), item2.as_ptr()) {
                break;
            }
            li = unsafe { nvim_listitem_get_next(li) };
            if li.is_null() {
                break;
            }
        }

        if li.is_null() {
            // item2 not found after item.
            unsafe { nvim_emsg_invrange() };
        } else {
            let ret_list = unsafe { rs_tv_list_alloc_ret(rettv, cnt as isize) };
            unsafe { rs_tv_list_move_items(l, item, item2, ret_list, cnt) };
        }
    }
}

/// FFI export: f_has_key - VimL has_key() function.
#[export_name = "f_has_key"]
pub unsafe extern "C" fn rs_f_has_key(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    _fptr: *const std::ffi::c_void,
) {
    let arg0 = unsafe { nvim_typval_array_get(argvars, 0) };
    if tv_check_for_dict_arg_impl(argvars, 0) == FAIL {
        return;
    }
    let d = unsafe { nvim_tv_get_dict(arg0) };
    if d.is_null() {
        return;
    }
    let arg1 = unsafe { nvim_typval_array_get(argvars, 1) };
    // Use nvim_tv_get_string (wraps tv_get_string) so numbers are converted
    // to their string representation via the static buffer. tv_get_string_ptr_impl
    // only handles VAR_STRING and returns NULL otherwise, which would crash
    // hash_find on a numeric key like has_key(d, 1).
    let key = unsafe { nvim_tv_get_string(arg1, std::ptr::null_mut()) };
    let found = unsafe { nvim_dict_find(d, key, -1) };
    unsafe { nvim_tv_set_number(rettv, i64::from(!found.is_null())) };
}

// =============================================================================
// Dict operations
// =============================================================================

/// Get the number of items in a dictionary.
/// Returns 0 if the dict is NULL.
#[inline]
fn tv_dict_len_impl(d: DictHandle) -> i64 {
    if d.is_null() {
        return 0;
    }
    unsafe { nvim_dict_get_ht_used(d) as i64 }
}

/// FFI wrapper: get dict length.
#[no_mangle]
pub extern "C" fn rs_tv_dict_len(d: DictHandle) -> i64 {
    tv_dict_len_impl(d)
}

/// Get dict lock status.
#[inline]
fn tv_dict_locked_impl(d: DictHandle) -> c_int {
    if d.is_null() {
        return 2; // VAR_FIXED
    }
    unsafe { nvim_dict_get_lock(d) }
}

/// FFI wrapper: get dict lock status.
#[no_mangle]
pub extern "C" fn rs_tv_dict_locked(d: DictHandle) -> c_int {
    tv_dict_locked_impl(d)
}

/// Check if dictionary is watched.
#[inline]
fn tv_dict_is_watched_impl(d: DictHandle) -> bool {
    if d.is_null() {
        return false;
    }
    unsafe { nvim_dict_has_watchers(d) != 0 }
}

/// FFI wrapper: check if dict is watched.
#[no_mangle]
pub extern "C" fn rs_tv_dict_is_watched(d: DictHandle) -> c_int {
    c_int::from(tv_dict_is_watched_impl(d))
}

// =============================================================================
// Dict item alloc/free/add (Phase 3)
// =============================================================================

// VarType integer constants for use with nvim_tv_set_type.
const VAR_UNKNOWN: c_int = VarType::Unknown as c_int;
const VAR_LIST: c_int = VarType::List as c_int;
const VAR_DICT: c_int = VarType::Dict as c_int;
const VAR_NUMBER: c_int = VarType::Number as c_int;
const VAR_FLOAT: c_int = VarType::Float as c_int;
const VAR_BOOL: c_int = VarType::Bool as c_int;
const VAR_STRING: c_int = VarType::String as c_int;
const VAR_FUNC: c_int = VarType::Func as c_int;

/// Allocate a dict item, set its value, add to dict.
/// Returns OK(1) on success, FAIL(0) on duplicate key.
unsafe fn dict_add_item_with_tv_setup(
    d: DictHandle,
    key: *const c_char,
    key_len: usize,
    setup: impl FnOnce(TypevalHandle),
) -> c_int {
    let item = unsafe { nvim_dict_item_alloc_len(key, key_len) };
    setup(unsafe { nvim_dictitem_di_tv(item) });
    if unsafe { nvim_dict_add_item(d, item) } == 0 {
        unsafe { nvim_dict_item_free(item) };
        return 0; // FAIL
    }
    1 // OK
}

/// FFI export: tv_dict_item_alloc_len - allocate dict item with key.
#[export_name = "tv_dict_item_alloc_len"]
pub unsafe extern "C" fn rs_tv_dict_item_alloc_len(
    key: *const c_char,
    key_len: usize,
) -> DictItemHandle {
    unsafe { nvim_dict_item_alloc_len(key, key_len) }
}

/// FFI export: tv_dict_item_alloc - allocate dict item (NUL-terminated key).
#[export_name = "tv_dict_item_alloc"]
pub unsafe extern "C" fn rs_tv_dict_item_alloc(key: *const c_char) -> DictItemHandle {
    let key_len = unsafe { libc_strlen(key) };
    unsafe { nvim_dict_item_alloc_len(key, key_len) }
}

/// FFI export: tv_dict_item_free - free a dict item.
#[export_name = "tv_dict_item_free"]
pub unsafe extern "C" fn rs_tv_dict_item_free(item: DictItemHandle) {
    unsafe { nvim_dict_item_free(item) };
}

/// FFI export: tv_dict_item_copy - make a copy of a dict item.
#[export_name = "tv_dict_item_copy"]
pub unsafe extern "C" fn rs_tv_dict_item_copy(di: DictItemHandle) -> DictItemHandle {
    let di_key = unsafe { nvim_dictitem_get_key(di) };
    let key_len = unsafe { libc_strlen(di_key) };
    let new_di = unsafe { nvim_dict_item_alloc_len(di_key, key_len) };
    let src_tv = unsafe { nvim_dictitem_get_tv(di) };
    let dst_tv = unsafe { nvim_dictitem_di_tv(new_di) };
    unsafe { nvim_tv_copy(src_tv, dst_tv) };
    new_di
}

/// FFI export: tv_dict_item_remove - remove and free a dict item.
#[export_name = "tv_dict_item_remove"]
pub unsafe extern "C" fn rs_tv_dict_item_remove(dict: DictHandle, item: DictItemHandle) {
    // Find the hash item and remove it from the hash table.
    let key = unsafe { nvim_dictitem_get_key(item) };
    // Use tv_dict_find to check if item is present (needed to get hashitem for removal).
    // We delegate the hash removal to nvim_dict_remove_key wrapper.
    unsafe { nvim_dict_remove_key(dict, key) };
    unsafe { nvim_dict_item_free(item) };
}

/// FFI export: tv_dict_alloc - allocate an empty dict.
#[export_name = "tv_dict_alloc"]
pub unsafe extern "C" fn rs_tv_dict_alloc() -> DictHandle {
    unsafe { nvim_dict_alloc_impl() }
}

/// FFI export: tv_dict_alloc_lock - allocate dict with given lock status.
#[export_name = "tv_dict_alloc_lock"]
pub unsafe extern "C" fn rs_tv_dict_alloc_lock(lock: c_int) -> DictHandle {
    let d = unsafe { nvim_dict_alloc_impl() };
    unsafe { nvim_dict_set_lock(d, lock) };
    d
}

/// FFI export: tv_dict_alloc_ret - allocate dict for return value.
#[export_name = "tv_dict_alloc_ret"]
pub unsafe extern "C" fn rs_tv_dict_alloc_ret(ret_tv: TypevalHandle) {
    unsafe { nvim_tv_dict_alloc_ret(ret_tv) };
}

/// FFI export: tv_dict_find - find item in dict by key.
#[export_name = "tv_dict_find"]
pub unsafe extern "C" fn rs_tv_dict_find(
    d: DictHandle,
    key: *const c_char,
    len: isize,
) -> DictItemHandle {
    unsafe { nvim_dict_find(d, key, len) }
}

/// FFI export: tv_dict_add - add item to dict.
#[export_name = "tv_dict_add"]
pub unsafe extern "C" fn rs_tv_dict_add(d: DictHandle, item: DictItemHandle) -> c_int {
    unsafe { nvim_dict_add_item(d, item) }
}

/// FFI export: tv_dict_add_list - add list entry to dict.
#[export_name = "tv_dict_add_list"]
pub unsafe extern "C" fn rs_tv_dict_add_list(
    d: DictHandle,
    key: *const c_char,
    key_len: usize,
    list: ListHandle,
) -> c_int {
    unsafe {
        dict_add_item_with_tv_setup(d, key, key_len, |tv| {
            nvim_tv_set_type(tv, VAR_LIST);
            nvim_tv_set_list(tv, list);
            nvim_list_ref(list);
        })
    }
}

/// FFI export: tv_dict_add_tv - add typval entry to dict.
#[export_name = "tv_dict_add_tv"]
pub unsafe extern "C" fn rs_tv_dict_add_tv(
    d: DictHandle,
    key: *const c_char,
    key_len: usize,
    tv: TypevalHandle,
) -> c_int {
    unsafe {
        dict_add_item_with_tv_setup(d, key, key_len, |item_tv| {
            nvim_tv_copy(tv, item_tv);
        })
    }
}

/// FFI export: tv_dict_add_dict - add dict entry to dict.
#[export_name = "tv_dict_add_dict"]
pub unsafe extern "C" fn rs_tv_dict_add_dict(
    d: DictHandle,
    key: *const c_char,
    key_len: usize,
    dict: DictHandle,
) -> c_int {
    unsafe {
        dict_add_item_with_tv_setup(d, key, key_len, |tv| {
            nvim_tv_set_type(tv, VAR_DICT);
            nvim_tv_set_dict(tv, dict);
            nvim_dict_inc_refcount(dict);
        })
    }
}

/// FFI export: tv_dict_add_nr - add number entry to dict.
#[export_name = "tv_dict_add_nr"]
pub unsafe extern "C" fn rs_tv_dict_add_nr(
    d: DictHandle,
    key: *const c_char,
    key_len: usize,
    nr: i64,
) -> c_int {
    unsafe {
        dict_add_item_with_tv_setup(d, key, key_len, |tv| {
            nvim_tv_set_type(tv, VAR_NUMBER);
            nvim_tv_set_number(tv, nr);
        })
    }
}

/// FFI export: tv_dict_add_float - add float entry to dict.
#[export_name = "tv_dict_add_float"]
pub unsafe extern "C" fn rs_tv_dict_add_float(
    d: DictHandle,
    key: *const c_char,
    key_len: usize,
    nr: f64,
) -> c_int {
    unsafe {
        dict_add_item_with_tv_setup(d, key, key_len, |tv| {
            nvim_tv_set_type(tv, VAR_FLOAT);
            nvim_tv_set_float(tv, nr);
        })
    }
}

/// FFI export: tv_dict_add_bool - add bool entry to dict.
#[export_name = "tv_dict_add_bool"]
pub unsafe extern "C" fn rs_tv_dict_add_bool(
    d: DictHandle,
    key: *const c_char,
    key_len: usize,
    val: c_int,
) -> c_int {
    unsafe {
        dict_add_item_with_tv_setup(d, key, key_len, |tv| {
            nvim_tv_set_type(tv, VAR_BOOL);
            nvim_tv_set_bool(tv, val);
        })
    }
}

/// FFI export: tv_dict_add_str - add string entry to dict.
#[export_name = "tv_dict_add_str"]
pub unsafe extern "C" fn rs_tv_dict_add_str(
    d: DictHandle,
    key: *const c_char,
    key_len: usize,
    val: *const c_char,
) -> c_int {
    unsafe { rs_tv_dict_add_str_len(d, key, key_len, val, -1) }
}

/// FFI export: tv_dict_add_str_len - add string entry with length to dict.
#[export_name = "tv_dict_add_str_len"]
pub unsafe extern "C" fn rs_tv_dict_add_str_len(
    d: DictHandle,
    key: *const c_char,
    key_len: usize,
    val: *const c_char,
    len: c_int,
) -> c_int {
    let s: *mut c_char = if val.is_null() {
        std::ptr::null_mut()
    } else if len < 0 {
        unsafe { nvim_xstrdup(val) }
    } else {
        unsafe { nvim_xstrndup(val, len as usize) }
    };
    unsafe { rs_tv_dict_add_allocated_str(d, key, key_len, s) }
}

/// FFI export: tv_dict_add_allocated_str - add pre-allocated string to dict.
#[export_name = "tv_dict_add_allocated_str"]
pub unsafe extern "C" fn rs_tv_dict_add_allocated_str(
    d: DictHandle,
    key: *const c_char,
    key_len: usize,
    val: *mut c_char,
) -> c_int {
    unsafe {
        dict_add_item_with_tv_setup(d, key, key_len, |tv| {
            nvim_tv_set_type(tv, VAR_STRING);
            nvim_tv_set_string(tv, val);
        })
    }
}

/// FFI export: tv_dict_add_func - add function entry to dict.
#[export_name = "tv_dict_add_func"]
pub unsafe extern "C" fn rs_tv_dict_add_func(
    d: DictHandle,
    key: *const c_char,
    key_len: usize,
    fp: *const std::ffi::c_void,
) -> c_int {
    let name = unsafe { nvim_ufunc_get_name(fp) };
    let namelen = unsafe { nvim_ufunc_get_namelen(fp) };
    let s = unsafe { nvim_xmemdupz(name, namelen) };
    let item = unsafe { nvim_dict_item_alloc_len(key, key_len) };
    let tv = unsafe { nvim_dictitem_di_tv(item) };
    unsafe { nvim_tv_set_type(tv, VAR_FUNC) };
    unsafe { nvim_tv_set_string(tv, s) };
    if unsafe { nvim_dict_add_item(d, item) } == 0 {
        unsafe { nvim_dict_item_free(item) };
        return 0; // FAIL
    }
    unsafe { nvim_func_ref(s) };
    1 // OK
}

// =============================================================================
// Dict lookup operations (Phase 4)
// =============================================================================

/// Check if a key is present in a dictionary.
#[export_name = "tv_dict_has_key"]
pub unsafe extern "C" fn rs_tv_dict_has_key(d: DictHandle, key: *const c_char) -> bool {
    !unsafe { nvim_dict_find(d, key, -1) }.is_null()
}

/// Get a typval item from a dictionary and copy it into rettv.
/// Returns OK (1) on success, FAIL (0) if key not found.
#[export_name = "tv_dict_get_tv"]
pub unsafe extern "C" fn rs_tv_dict_get_tv(
    d: DictHandle,
    key: *const c_char,
    rettv: TypevalHandle,
) -> c_int {
    let di = unsafe { nvim_dict_find(d, key, -1) };
    if di.is_null() {
        return 0; // FAIL
    }
    let di_tv = unsafe { nvim_dictitem_get_tv(di) };
    unsafe { tv_copy(di_tv.0, rettv.0.cast_mut()) };
    1 // OK
}

/// Gets a number item from a dictionary, or a given default value.
unsafe fn tv_dict_get_number_def_impl(d: DictHandle, key: *const c_char, def: i64) -> i64 {
    let di = unsafe { nvim_dict_find(d, key, -1) };
    if di.is_null() {
        return def;
    }
    let di_tv = unsafe { nvim_dictitem_get_tv(di) };
    unsafe { nvim_tv_get_number_simple(di_tv) }
}

/// Gets a number item from a dictionary.
/// Returns 0 if the item does not exist.
#[export_name = "tv_dict_get_number"]
pub unsafe extern "C" fn rs_tv_dict_get_number(d: DictHandle, key: *const c_char) -> i64 {
    unsafe { tv_dict_get_number_def_impl(d, key, 0) }
}

/// Gets a number item from a dictionary, or a given default value.
#[export_name = "tv_dict_get_number_def"]
pub unsafe extern "C" fn rs_tv_dict_get_number_def(
    d: DictHandle,
    key: *const c_char,
    def: c_int,
) -> i64 {
    unsafe { tv_dict_get_number_def_impl(d, key, i64::from(def)) }
}

/// Gets a bool item from a dictionary, or a given default value.
#[export_name = "tv_dict_get_bool"]
pub unsafe extern "C" fn rs_tv_dict_get_bool(d: DictHandle, key: *const c_char, def: c_int) -> i64 {
    let di = unsafe { nvim_dict_find(d, key, -1) };
    if di.is_null() {
        return i64::from(def);
    }
    let di_tv = unsafe { nvim_dictitem_get_tv(di) };
    i64::from(unsafe { nvim_tv_get_bool_simple(di_tv) })
}

/// Get a string item from a dictionary with a caller-provided buffer.
/// Returns NULL if key does not exist.
#[export_name = "tv_dict_get_string_buf"]
pub unsafe extern "C" fn rs_tv_dict_get_string_buf(
    d: DictHandle,
    key: *const c_char,
    numbuf: *mut c_char,
) -> *const c_char {
    let di = unsafe { nvim_dict_find(d, key, -1) };
    if di.is_null() {
        return std::ptr::null();
    }
    let di_tv = unsafe { nvim_dictitem_get_tv(di) };
    unsafe { nvim_tv_get_string_buf(di_tv, numbuf) }
}

/// Get a string item from a dictionary with key length and default.
/// Returns def if key does not exist, NULL on type error.
#[export_name = "tv_dict_get_string_buf_chk"]
pub unsafe extern "C" fn rs_tv_dict_get_string_buf_chk(
    d: DictHandle,
    key: *const c_char,
    key_len: isize,
    numbuf: *mut c_char,
    def: *const c_char,
) -> *const c_char {
    let di = unsafe { nvim_dict_find(d, key, key_len) };
    if di.is_null() {
        return def;
    }
    let di_tv = unsafe { nvim_dictitem_get_tv(di) };
    unsafe { nvim_tv_get_string_buf_chk(di_tv, numbuf) }
}

// =============================================================================
// Blob operations
// =============================================================================

/// Get the length of the data in the blob, in bytes.
/// Returns 0 if the blob is NULL.
#[inline]
fn tv_blob_len_impl(b: BlobHandle) -> c_int {
    if b.is_null() {
        return 0;
    }
    unsafe { nvim_blob_get_len(b) }
}

/// FFI wrapper: get blob length.
#[no_mangle]
pub extern "C" fn rs_tv_blob_len(b: BlobHandle) -> c_int {
    tv_blob_len_impl(b)
}

/// Get blob lock status.
#[inline]
fn tv_blob_locked_impl(b: BlobHandle) -> c_int {
    if b.is_null() {
        return 2; // VAR_FIXED
    }
    unsafe { nvim_blob_get_lock(b) }
}

/// FFI wrapper: get blob lock status.
#[no_mangle]
pub extern "C" fn rs_tv_blob_locked(b: BlobHandle) -> c_int {
    tv_blob_locked_impl(b)
}

/// Get the byte at index `idx` in the blob.
/// Caller must ensure blob is non-NULL and idx is valid.
#[inline]
fn tv_blob_get_impl(b: BlobHandle, idx: c_int) -> u8 {
    unsafe { nvim_blob_get_byte(b, idx) }
}

/// FFI wrapper: get byte from blob.
#[no_mangle]
pub extern "C" fn rs_tv_blob_get(b: BlobHandle, idx: c_int) -> u8 {
    tv_blob_get_impl(b, idx)
}

/// Set the byte at index `idx` in the blob.
/// Caller must ensure blob is non-NULL and idx is valid.
#[inline]
fn tv_blob_set_impl(b: BlobHandle, idx: c_int, c: u8) {
    unsafe { nvim_blob_set_byte(b, idx, c) }
}

/// FFI wrapper: set byte in blob.
#[no_mangle]
pub extern "C" fn rs_tv_blob_set(b: BlobHandle, idx: c_int, c: u8) {
    tv_blob_set_impl(b, idx, c);
}

/// Check if two blobs are equal (byte-by-byte comparison).
/// Empty and NULL blobs are considered equal.
#[inline]
fn tv_blob_equal_impl(b1: BlobHandle, b2: BlobHandle) -> bool {
    let len1 = tv_blob_len_impl(b1);
    let len2 = tv_blob_len_impl(b2);

    // empty and NULL are considered the same
    if len1 == 0 && len2 == 0 {
        return true;
    }
    if b1.0 == b2.0 {
        return true;
    }
    if len1 != len2 {
        return false;
    }

    // Compare byte by byte
    for i in 0..len1 {
        if tv_blob_get_impl(b1, i) != tv_blob_get_impl(b2, i) {
            return false;
        }
    }
    true
}

/// FFI wrapper: check if two blobs are equal.
#[export_name = "tv_blob_equal"]
pub extern "C" fn rs_tv_blob_equal(b1: BlobHandle, b2: BlobHandle) -> bool {
    tv_blob_equal_impl(b1, b2)
}

// =============================================================================
// Blob operations: slice, index, check, set, remove (Phase 1)
// =============================================================================

/// Returns a slice of `blob` from index `n1` to `n2` in `rettv`.
/// Returns empty blob if indexes are out of range.
unsafe fn tv_blob_slice_impl(
    _blob: BlobHandle,
    len: c_int,
    mut n1: i64,
    mut n2: i64,
    exclusive: bool,
    rettv: TypevalHandle,
) -> c_int {
    let len = i64::from(len);
    if n1 < 0 {
        n1 += len;
        if n1 < 0 {
            n1 = 0;
        }
    }
    if n2 < 0 {
        n2 += len;
    } else if n2 >= len {
        n2 = len - i64::from(!exclusive);
    }
    if exclusive {
        n2 -= 1;
    }
    if n1 >= len || n2 < 0 || n1 > n2 {
        unsafe {
            tv_clear(rettv);
            // Set v_type=VAR_BLOB, v_blob=NULL
            nvim_tv_set_blob(rettv, BlobHandle(std::ptr::null()));
        }
    } else {
        let new_blob = unsafe { tv_blob_alloc() };
        let count = (n2 - n1 + 1) as c_int;
        unsafe {
            nvim_blob_ga_grow(new_blob, count);
            nvim_blob_set_ga_len(new_blob, count);
        }
        // Get the source data from rettv's blob
        let src_blob = unsafe { nvim_tv_get_blob(rettv) };
        for i in n1..=n2 {
            let byte = unsafe { nvim_blob_get_byte(src_blob, i as c_int) };
            unsafe { nvim_blob_set_byte(new_blob, (i - n1) as c_int, byte) };
        }
        unsafe {
            tv_clear(rettv);
            nvim_tv_set_blob(rettv, new_blob);
        }
    }
    OK
}

/// Return the byte value in `blob` at index `idx` in `rettv`.
unsafe fn tv_blob_index_impl(
    _blob: BlobHandle,
    len: c_int,
    mut idx: i64,
    rettv: TypevalHandle,
) -> c_int {
    let len = i64::from(len);
    if idx < 0 {
        idx += len;
    }
    if idx < len && idx >= 0 {
        let src_blob = unsafe { nvim_tv_get_blob(rettv) };
        let v = i64::from(unsafe { nvim_blob_get_byte(src_blob, idx as c_int) });
        unsafe { tv_clear(rettv) };
        // Set to number - need mutable pointer to rettv
        let rettv_ptr = rettv.as_ptr().cast_mut();
        unsafe { nvim_tv_set_number(TypevalHandle(rettv_ptr), v) };
    } else {
        unsafe { nvim_semsg_blobidx(idx) };
        return FAIL;
    }
    OK
}

/// Dispatch between blob slice and blob index operations.
#[export_name = "tv_blob_slice_or_index"]
pub unsafe extern "C" fn rs_tv_blob_slice_or_index(
    blob: BlobHandle,
    is_range: bool,
    n1: i64,
    n2: i64,
    exclusive: bool,
    rettv: TypevalHandle,
) -> c_int {
    let len = tv_blob_len_impl(unsafe { nvim_tv_get_blob(rettv) });
    if is_range {
        unsafe { tv_blob_slice_impl(blob, len, n1, n2, exclusive, rettv) }
    } else {
        unsafe { tv_blob_index_impl(blob, len, n1, rettv) }
    }
}

/// Check if `n1` is a valid index for a blob with length `bloblen`.
#[export_name = "tv_blob_check_index"]
pub unsafe extern "C" fn rs_tv_blob_check_index(bloblen: c_int, n1: i64, quiet: bool) -> c_int {
    if n1 < 0 || n1 > i64::from(bloblen) {
        if !quiet {
            unsafe { nvim_semsg_blobidx(n1) };
        }
        return FAIL;
    }
    OK
}

/// Check if `n1`-`n2` is a valid range for a blob with length `bloblen`.
#[export_name = "tv_blob_check_range"]
pub unsafe extern "C" fn rs_tv_blob_check_range(
    bloblen: c_int,
    n1: i64,
    n2: i64,
    quiet: bool,
) -> c_int {
    if n2 < 0 || n2 >= i64::from(bloblen) || n2 < n1 {
        if !quiet {
            unsafe { nvim_semsg_blobidx(n2) };
        }
        return FAIL;
    }
    OK
}

/// Set bytes `n1` to `n2` (inclusive) in `dest` to the value of `src` blob.
/// Caller must make sure `src` is a blob.
/// Returns FAIL if the number of bytes does not match.
#[export_name = "tv_blob_set_range"]
pub unsafe extern "C" fn rs_tv_blob_set_range(
    dest: BlobHandle,
    n1: i64,
    n2: i64,
    src: TypevalHandle,
) -> c_int {
    let src_blob = unsafe { nvim_tv_get_blob(src) };
    let src_len = i64::from(tv_blob_len_impl(src_blob));
    if n2 - n1 + 1 != src_len {
        unsafe { nvim_emsg_blob_wrong_bytes() };
        return FAIL;
    }
    let mut ir = 0i64;
    let mut il = n1;
    while il <= n2 {
        let byte = unsafe { nvim_blob_get_byte(src_blob, ir as c_int) };
        unsafe { nvim_blob_set_byte(dest, il as c_int, byte) };
        ir += 1;
        il += 1;
    }
    OK
}

/// Store one byte `byte` in blob `blob` at `idx`.
/// Append one byte if needed.
#[export_name = "tv_blob_set_append"]
pub unsafe extern "C" fn rs_tv_blob_set_append(blob: BlobHandle, idx: c_int, byte: u8) {
    let ga_len = tv_blob_len_impl(blob);
    if idx <= ga_len {
        if idx == ga_len {
            unsafe { nvim_blob_ga_grow(blob, 1) };
            unsafe { nvim_blob_set_ga_len(blob, ga_len + 1) };
        }
        unsafe { nvim_blob_set_byte(blob, idx, byte) };
    }
}

/// "remove({blob})" function implementation.
#[export_name = "tv_blob_remove"]
pub unsafe extern "C" fn rs_tv_blob_remove(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    arg_errmsg: *const c_char,
) {
    let arg0 = unsafe { nvim_typval_array_get(argvars, 0) };
    let b = unsafe { nvim_tv_get_blob(arg0) };

    if !b.is_null() {
        let lock = unsafe { nvim_blob_get_lock(b) };
        if unsafe { nvim_value_check_lock_translated(lock, arg_errmsg) } {
            return;
        }
    }

    let mut error = false;
    let arg1 = unsafe { nvim_typval_array_get(argvars, 1) };
    let mut idx = unsafe { tv_get_number_chk(arg1, std::ptr::addr_of_mut!(error)) };

    if !error {
        let len = i64::from(tv_blob_len_impl(b));

        if idx < 0 {
            idx += len;
        }
        if idx < 0 || idx >= len {
            unsafe { nvim_semsg_blobidx(idx) };
            return;
        }

        let arg2 = unsafe { nvim_typval_array_get(argvars, 2) };
        let arg2_type = tv_type_impl(arg2);
        if arg2_type == VarType::Unknown {
            // Remove one item, return its value.
            let p = unsafe { nvim_blob_get_ga_data(b) };
            let val = i64::from(unsafe { *p.offset(idx as isize) });
            unsafe {
                std::ptr::copy(
                    p.offset(idx as isize + 1),
                    p.offset(idx as isize),
                    (len - idx - 1) as usize,
                );
                nvim_blob_set_ga_len(b, (len - 1) as c_int);
            }
            let rettv_ptr = rettv.as_ptr().cast_mut();
            unsafe { nvim_tv_set_number(TypevalHandle(rettv_ptr), val) };
        } else {
            // Remove range of items, return blob with values.
            let mut end = unsafe { tv_get_number_chk(arg2, std::ptr::addr_of_mut!(error)) };
            if error {
                return;
            }
            if end < 0 {
                end += len;
            }
            if end >= len || idx > end {
                unsafe { nvim_semsg_blobidx(end) };
                return;
            }
            let blob = unsafe { tv_blob_alloc() };
            let count = (end - idx + 1) as c_int;
            unsafe {
                nvim_blob_set_ga_len(blob, count);
                nvim_blob_ga_grow(blob, count);
            }

            let p = unsafe { nvim_blob_get_ga_data(b) };
            let dst = unsafe { nvim_blob_get_ga_data(blob) };
            unsafe {
                std::ptr::copy_nonoverlapping(p.offset(idx as isize), dst, count as usize);
                nvim_tv_set_blob(rettv, blob);
            }

            let remaining = len - end - 1;
            if remaining > 0 {
                unsafe {
                    std::ptr::copy(
                        p.offset(end as isize + 1),
                        p.offset(idx as isize),
                        remaining as usize,
                    );
                }
            }
            unsafe { nvim_blob_set_ga_len(b, (len - i64::from(count)) as c_int) };
        }
    }
}

// =============================================================================
// Phase 2: List equality, find, tv2bool
// =============================================================================

/// Compare two lists for equality.
fn tv_list_equal_impl(l1: ListHandle, l2: ListHandle, ic: bool) -> bool {
    // Same pointer => equal
    if l1.0 == l2.0 {
        return true;
    }
    let len1 = tv_list_len_impl(l1);
    let len2 = tv_list_len_impl(l2);
    if len1 != len2 {
        return false;
    }
    // empty and NULL list are considered equal
    if len1 == 0 {
        return true;
    }
    if l1.is_null() || l2.is_null() {
        return false;
    }

    let mut item1 = tv_list_first_impl(l1);
    let mut item2 = tv_list_first_impl(l2);
    while !item1.is_null() && !item2.is_null() {
        let tv1 = tv_listitem_tv_impl(item1);
        let tv2 = tv_listitem_tv_impl(item2);
        if !unsafe { tv_equal(tv1, tv2, ic) } {
            return false;
        }
        item1 = tv_listitem_next_impl(item1);
        item2 = tv_listitem_next_impl(item2);
    }
    true
}

/// FFI wrapper: compare two lists for equality.
#[export_name = "tv_list_equal"]
pub extern "C" fn rs_tv_list_equal(l1: ListHandle, l2: ListHandle, ic: bool) -> bool {
    tv_list_equal_impl(l1, l2, ic)
}

/// Get list item l[n] as a number.
fn tv_list_find_nr_impl(l: ListHandle, n: c_int, ret_error: *mut bool) -> i64 {
    let li = tv_list_find_impl(l, n);
    if li.is_null() {
        if !ret_error.is_null() {
            unsafe { *ret_error = true };
        }
        return -1;
    }
    let tv = tv_listitem_tv_impl(li);
    unsafe { tv_get_number_chk(tv, ret_error) }
}

/// FFI wrapper: get list item as number.
#[export_name = "tv_list_find_nr"]
pub extern "C" fn rs_tv_list_find_nr(l: ListHandle, n: c_int, ret_error: *mut bool) -> i64 {
    tv_list_find_nr_impl(l, n, ret_error)
}

/// Get list item l[n] as a string.
unsafe fn tv_list_find_str_impl(l: ListHandle, n: c_int) -> *const c_char {
    let li = tv_list_find_impl(l, n);
    if li.is_null() {
        semsg_typval(
            e_list_index_out_of_range_nr_tv.as_ptr().cast(),
            i64::from(n),
        );
        return std::ptr::null();
    }
    let tv = tv_listitem_tv_impl(li);
    unsafe { nvim_tv_get_string(tv, std::ptr::null_mut()) }
}

/// FFI wrapper: get list item as string.
#[export_name = "tv_list_find_str"]
pub unsafe extern "C" fn rs_tv_list_find_str(l: ListHandle, n: c_int) -> *const c_char {
    unsafe { tv_list_find_str_impl(l, n) }
}

/// Check if typval is truthy (non-zero, non-empty, non-null).
fn tv2bool_impl(tv: TypevalHandle) -> bool {
    match tv_type_impl(tv) {
        VarType::Number => {
            let n = unsafe { nvim_tv_get_number(tv) };
            n != 0
        }
        VarType::Float => {
            let f = unsafe { nvim_tv_get_float(tv) };
            f != 0.0
        }
        VarType::Partial => {
            let is_null = unsafe { nvim_tv_partial_is_null(tv) };
            is_null == 0
        }
        VarType::Func | VarType::String => {
            let s = unsafe { nvim_tv_get_string_ptr(tv) };
            !s.is_null() && unsafe { *s != 0 }
        }
        VarType::List => {
            let l = unsafe { nvim_tv_get_list(tv) };
            !l.is_null() && tv_list_len_impl(l) > 0
        }
        VarType::Dict => {
            let d = unsafe { nvim_tv_get_dict(tv) };
            !d.is_null() && tv_dict_len_impl(d) > 0
        }
        VarType::Bool => {
            let b = unsafe { nvim_tv_get_bool(tv) };
            b != 0
        }
        VarType::Special => false,
        VarType::Blob => {
            let b = unsafe { nvim_tv_get_blob(tv) };
            !b.is_null() && tv_blob_len_impl(b) > 0
        }
        VarType::Unknown => false,
    }
}

/// FFI wrapper: check if typval is truthy.
#[export_name = "tv2bool"]
pub extern "C" fn rs_tv2bool(tv: TypevalHandle) -> bool {
    tv2bool_impl(tv)
}

// =============================================================================
// Phase 3: tv_get_float, value_check_lock, tv_check_lock
// =============================================================================

/// Get the float value of a Vimscript object.
/// Raises an error if object is not number or floating-point.
fn tv_get_float_impl(tv: TypevalHandle) -> f64 {
    match tv_type_impl(tv) {
        VarType::Number => {
            let n = unsafe { nvim_tv_get_number(tv) };
            n as f64
        }
        VarType::Float => unsafe { nvim_tv_get_float(tv) },
        VarType::Partial | VarType::Func => {
            unsafe { nvim_emsg_float_funcref() };
            0.0
        }
        VarType::String => {
            unsafe { nvim_emsg_float_string() };
            0.0
        }
        VarType::List => {
            unsafe { nvim_emsg_float_list() };
            0.0
        }
        VarType::Dict => {
            unsafe { nvim_emsg_float_dict() };
            0.0
        }
        VarType::Bool => {
            unsafe { nvim_emsg_float_bool() };
            0.0
        }
        VarType::Special => {
            unsafe { nvim_emsg_float_special() };
            0.0
        }
        VarType::Blob => {
            unsafe { nvim_emsg_float_blob() };
            0.0
        }
        VarType::Unknown => {
            unsafe { nvim_emsg_float_unknown() };
            0.0
        }
    }
}

/// FFI wrapper: get float from typval.
#[export_name = "tv_get_float"]
pub extern "C" fn rs_tv_get_float(tv: TypevalHandle) -> f64 {
    tv_get_float_impl(tv)
}

/// Check if variable "name" has a locked (immutable) value.
fn value_check_lock_impl(lock: c_int, name: *const c_char, name_len: usize) -> bool {
    unsafe { nvim_value_check_lock(lock, name, name_len) }
}

/// FFI wrapper: check value lock status.
#[export_name = "value_check_lock"]
pub unsafe extern "C" fn rs_value_check_lock(
    lock: c_int,
    name: *const c_char,
    name_len: usize,
) -> bool {
    value_check_lock_impl(lock, name, name_len)
}

/// Check typval's own lock and container lock.
fn tv_check_lock_impl(tv: TypevalHandle, name: *const c_char, name_len: usize) -> bool {
    let container_lock = match tv_type_impl(tv) {
        VarType::Blob => {
            let b = unsafe { nvim_tv_get_blob(tv) };
            if b.is_null() {
                0
            } else {
                unsafe { nvim_blob_get_lock(b) }
            }
        }
        VarType::List => {
            let l = unsafe { nvim_tv_get_list(tv) };
            if l.is_null() {
                0
            } else {
                unsafe { nvim_list_get_lock(l) }
            }
        }
        VarType::Dict => {
            let d = unsafe { nvim_tv_get_dict(tv) };
            if d.is_null() {
                0
            } else {
                unsafe { nvim_dict_get_lock(d) }
            }
        }
        _ => 0,
    };
    let v_lock = unsafe { nvim_tv_get_v_lock(tv) };
    value_check_lock_impl(v_lock, name, name_len)
        || (container_lock != 0 && value_check_lock_impl(container_lock, name, name_len))
}

/// FFI wrapper: check typval lock status.
#[export_name = "tv_check_lock"]
pub unsafe extern "C" fn rs_tv_check_lock(
    tv: TypevalHandle,
    name: *const c_char,
    name_len: usize,
) -> bool {
    tv_check_lock_impl(tv, name, name_len)
}

/// Check whether a Vimscript value is locked itself or refers to a locked container.
/// VAR_LOCKED (not VAR_FIXED) is the criterion.
#[export_name = "tv_islocked"]
pub extern "C" fn rs_tv_islocked(tv: TypevalHandle) -> bool {
    // VAR_LOCKED = 1
    let v_lock = unsafe { nvim_tv_get_v_lock(tv) };
    if v_lock == 1 {
        return true;
    }
    match tv_type_impl(tv) {
        VarType::List => {
            let l = unsafe { nvim_tv_get_list(tv) };
            tv_list_locked_impl(l) == 1 // VAR_LOCKED
        }
        VarType::Dict => {
            let d = unsafe { nvim_tv_get_dict(tv) };
            if d.is_null() {
                false
            } else {
                let lock = unsafe { nvim_dict_get_lock(d) };
                lock == 1 // VAR_LOCKED
            }
        }
        _ => false,
    }
}

// =============================================================================
// Typval -> container conversions
// =============================================================================

/// Get the list from a typval (returns NULL handle if not a list or NULL).
#[inline]
fn tv_get_list_impl(tv: TypevalHandle) -> ListHandle {
    if tv.is_null() || tv_type_impl(tv) != VarType::List {
        return ListHandle(std::ptr::null());
    }
    unsafe { nvim_tv_get_list(tv) }
}

/// FFI wrapper: get list from typval.
#[no_mangle]
pub extern "C" fn rs_tv_get_list(tv: TypevalHandle) -> ListHandle {
    tv_get_list_impl(tv)
}

/// Get the dict from a typval (returns NULL handle if not a dict or NULL).
#[inline]
fn tv_get_dict_impl(tv: TypevalHandle) -> DictHandle {
    if tv.is_null() || tv_type_impl(tv) != VarType::Dict {
        return DictHandle(std::ptr::null());
    }
    unsafe { nvim_tv_get_dict(tv) }
}

/// FFI wrapper: get dict from typval.
#[no_mangle]
pub extern "C" fn rs_tv_get_dict(tv: TypevalHandle) -> DictHandle {
    tv_get_dict_impl(tv)
}

/// Get the blob from a typval (returns NULL handle if not a blob or NULL).
#[inline]
fn tv_get_blob_impl(tv: TypevalHandle) -> BlobHandle {
    if tv.is_null() || tv_type_impl(tv) != VarType::Blob {
        return BlobHandle(std::ptr::null());
    }
    unsafe { nvim_tv_get_blob(tv) }
}

/// FFI wrapper: get blob from typval.
#[no_mangle]
pub extern "C" fn rs_tv_get_blob(tv: TypevalHandle) -> BlobHandle {
    tv_get_blob_impl(tv)
}

// =============================================================================
// Type checking functions (tv_check_for_* family)
// =============================================================================

// C accessor functions for error message reporting.
// These wrap semsg() calls since semsg is variadic and hard to call from Rust.
extern "C" {
    /// Get a pointer to args[idx] in a typval array.
    /// This avoids needing to know sizeof(typval_T) in Rust.
    fn nvim_typval_array_get(args: TypevalHandle, idx: c_int) -> TypevalHandle;

    fn nvim_typval_error_string_required(idx: c_int);
    fn nvim_typval_error_nonempty_string_required(idx: c_int);
    fn nvim_typval_error_number_required(idx: c_int);
    fn nvim_typval_error_float_or_number_required(idx: c_int);
    fn nvim_typval_error_bool_required(idx: c_int);
    fn nvim_typval_error_blob_required(idx: c_int);
    fn nvim_typval_error_list_required(idx: c_int);
    fn nvim_typval_error_dict_required(idx: c_int);
    fn nvim_typval_error_nonnull_dict_required(idx: c_int);
    fn nvim_typval_error_string_or_number_required(idx: c_int);
    fn nvim_typval_error_string_or_list_required(idx: c_int);
    fn nvim_typval_error_string_list_or_blob_required(idx: c_int);
    fn nvim_typval_error_string_list_or_dict_required(idx: c_int);
    fn nvim_typval_error_string_or_func_required(idx: c_int);
    fn nvim_typval_error_list_or_blob_required(idx: c_int);

    // tv_check_num error messages (type-specific)
    fn nvim_typval_error_using_funcref_as_number();
    fn nvim_typval_error_using_list_as_number();
    fn nvim_typval_error_using_dict_as_number();
    fn nvim_typval_error_using_float_as_number();
    fn nvim_typval_error_using_blob_as_number();
    fn nvim_typval_error_using_invalid_as_number();

    // tv_check_str error messages (type-specific)
    fn nvim_typval_error_using_funcref_as_string();
    fn nvim_typval_error_using_list_as_string();
    fn nvim_typval_error_using_dict_as_string();
    fn nvim_typval_error_using_blob_as_string();
    fn nvim_typval_error_using_invalid_as_string();

    // tv_check_str_or_nr error messages (type-specific)
    fn nvim_typval_error_str_or_nr_float();
    fn nvim_typval_error_str_or_nr_funcref();
    fn nvim_typval_error_str_or_nr_list();
    fn nvim_typval_error_str_or_nr_dict();
    fn nvim_typval_error_str_or_nr_blob();
    fn nvim_typval_error_str_or_nr_bool();
    fn nvim_typval_error_str_or_nr_special();
    fn nvim_typval_error_str_or_nr_unknown();
}

/// OK return value (1) matching C's OK from vim_defs.h.
const OK: c_int = 1;
/// FAIL return value (0) matching C's FAIL from vim_defs.h.
const FAIL: c_int = 0;

/// Get the typval at args[idx] using the C accessor.
/// This avoids needing to know sizeof(typval_T) in Rust.
#[inline]
fn get_arg(args: TypevalHandle, idx: c_int) -> TypevalHandle {
    unsafe { nvim_typval_array_get(args, idx) }
}

/// Check if args[idx] is a string. Return OK if valid, FAIL if not.
#[inline]
fn tv_check_for_string_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    if tv_type_impl(tv) == VarType::String {
        OK
    } else {
        unsafe { nvim_typval_error_string_required(idx + 1) };
        FAIL
    }
}

/// FFI wrapper: check if args[idx] is a string.
#[export_name = "tv_check_for_string_arg"]
pub extern "C" fn rs_tv_check_for_string_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_string_arg_impl(args, idx)
}

/// Check if args[idx] is a non-empty string. Return OK if valid, FAIL if not.
#[inline]
fn tv_check_for_nonempty_string_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    if tv_check_for_string_arg_impl(args, idx) == FAIL {
        return FAIL;
    }
    let tv = get_arg(args, idx);
    let s = unsafe { nvim_tv_get_string_ptr(tv) };
    if s.is_null() || unsafe { *s == 0 } {
        unsafe { nvim_typval_error_nonempty_string_required(idx + 1) };
        return FAIL;
    }
    OK
}

/// FFI wrapper: check if args[idx] is a non-empty string.
#[export_name = "tv_check_for_nonempty_string_arg"]
pub extern "C" fn rs_tv_check_for_nonempty_string_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_nonempty_string_arg_impl(args, idx)
}

/// Check for optional string at args[idx]. VAR_UNKNOWN is OK.
#[inline]
fn tv_check_for_opt_string_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    if tv_type_impl(tv) == VarType::Unknown {
        OK
    } else {
        tv_check_for_string_arg_impl(args, idx)
    }
}

/// FFI wrapper: check for optional string at args[idx].
#[export_name = "tv_check_for_opt_string_arg"]
pub extern "C" fn rs_tv_check_for_opt_string_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_opt_string_arg_impl(args, idx)
}

/// Check if args[idx] is a number. Return OK if valid, FAIL if not.
#[inline]
fn tv_check_for_number_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    if tv_type_impl(tv) == VarType::Number {
        OK
    } else {
        unsafe { nvim_typval_error_number_required(idx + 1) };
        FAIL
    }
}

/// FFI wrapper: check if args[idx] is a number.
#[export_name = "tv_check_for_number_arg"]
pub extern "C" fn rs_tv_check_for_number_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_number_arg_impl(args, idx)
}

/// Check for optional number at args[idx]. VAR_UNKNOWN is OK.
#[inline]
fn tv_check_for_opt_number_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    if tv_type_impl(tv) == VarType::Unknown {
        OK
    } else {
        tv_check_for_number_arg_impl(args, idx)
    }
}

/// FFI wrapper: check for optional number at args[idx].
#[export_name = "tv_check_for_opt_number_arg"]
pub extern "C" fn rs_tv_check_for_opt_number_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_opt_number_arg_impl(args, idx)
}

/// Check if args[idx] is a float or number. Return OK if valid, FAIL if not.
#[inline]
fn tv_check_for_float_or_nr_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    let t = tv_type_impl(tv);
    if t == VarType::Float || t == VarType::Number {
        OK
    } else {
        unsafe { nvim_typval_error_float_or_number_required(idx + 1) };
        FAIL
    }
}

/// FFI wrapper: check if args[idx] is a float or number.
#[export_name = "tv_check_for_float_or_nr_arg"]
pub extern "C" fn rs_tv_check_for_float_or_nr_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_float_or_nr_arg_impl(args, idx)
}

/// Check if args[idx] is a bool (VAR_BOOL or NUMBER 0/1). Return OK if valid, FAIL if not.
#[inline]
fn tv_check_for_bool_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    let t = tv_type_impl(tv);
    if t == VarType::Bool {
        return OK;
    }
    // Also accept numbers 0 and 1 as bool values
    if t == VarType::Number {
        let n = unsafe { nvim_tv_get_number(tv) };
        if n == 0 || n == 1 {
            return OK;
        }
    }
    unsafe { nvim_typval_error_bool_required(idx + 1) };
    FAIL
}

/// FFI wrapper: check if args[idx] is a bool.
#[export_name = "tv_check_for_bool_arg"]
pub extern "C" fn rs_tv_check_for_bool_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_bool_arg_impl(args, idx)
}

/// Check for optional bool at args[idx]. VAR_UNKNOWN is OK.
#[inline]
fn tv_check_for_opt_bool_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    if tv_type_impl(tv) == VarType::Unknown {
        OK
    } else {
        tv_check_for_bool_arg_impl(args, idx)
    }
}

/// FFI wrapper: check for optional bool at args[idx].
#[export_name = "tv_check_for_opt_bool_arg"]
pub extern "C" fn rs_tv_check_for_opt_bool_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_opt_bool_arg_impl(args, idx)
}

/// Check if args[idx] is a blob. Return OK if valid, FAIL if not.
#[inline]
fn tv_check_for_blob_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    if tv_type_impl(tv) == VarType::Blob {
        OK
    } else {
        unsafe { nvim_typval_error_blob_required(idx + 1) };
        FAIL
    }
}

/// FFI wrapper: check if args[idx] is a blob.
#[export_name = "tv_check_for_blob_arg"]
pub extern "C" fn rs_tv_check_for_blob_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_blob_arg_impl(args, idx)
}

/// Check if args[idx] is a list. Return OK if valid, FAIL if not.
#[inline]
fn tv_check_for_list_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    if tv_type_impl(tv) == VarType::List {
        OK
    } else {
        unsafe { nvim_typval_error_list_required(idx + 1) };
        FAIL
    }
}

/// FFI wrapper: check if args[idx] is a list.
#[export_name = "tv_check_for_list_arg"]
pub extern "C" fn rs_tv_check_for_list_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_list_arg_impl(args, idx)
}

/// Check if args[idx] is a dict. Return OK if valid, FAIL if not.
#[inline]
fn tv_check_for_dict_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    if tv_type_impl(tv) == VarType::Dict {
        OK
    } else {
        unsafe { nvim_typval_error_dict_required(idx + 1) };
        FAIL
    }
}

/// FFI wrapper: check if args[idx] is a dict.
#[export_name = "tv_check_for_dict_arg"]
pub extern "C" fn rs_tv_check_for_dict_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_dict_arg_impl(args, idx)
}

/// Check if args[idx] is a non-NULL dict. Return OK if valid, FAIL if not.
#[inline]
fn tv_check_for_nonnull_dict_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    if tv_check_for_dict_arg_impl(args, idx) == FAIL {
        return FAIL;
    }
    let tv = get_arg(args, idx);
    if unsafe { nvim_tv_dict_is_null(tv) != 0 } {
        unsafe { nvim_typval_error_nonnull_dict_required(idx + 1) };
        return FAIL;
    }
    OK
}

/// FFI wrapper: check if args[idx] is a non-NULL dict.
#[export_name = "tv_check_for_nonnull_dict_arg"]
pub extern "C" fn rs_tv_check_for_nonnull_dict_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_nonnull_dict_arg_impl(args, idx)
}

/// Check for optional dict at args[idx]. VAR_UNKNOWN is OK.
#[inline]
fn tv_check_for_opt_dict_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    if tv_type_impl(tv) == VarType::Unknown {
        OK
    } else {
        tv_check_for_dict_arg_impl(args, idx)
    }
}

/// FFI wrapper: check for optional dict at args[idx].
#[export_name = "tv_check_for_opt_dict_arg"]
pub extern "C" fn rs_tv_check_for_opt_dict_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_opt_dict_arg_impl(args, idx)
}

/// Check if args[idx] is a string or number. Return OK if valid, FAIL if not.
#[inline]
fn tv_check_for_string_or_number_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    let t = tv_type_impl(tv);
    if t == VarType::String || t == VarType::Number {
        OK
    } else {
        unsafe { nvim_typval_error_string_or_number_required(idx + 1) };
        FAIL
    }
}

/// FFI wrapper: check if args[idx] is a string or number.
#[export_name = "tv_check_for_string_or_number_arg"]
pub extern "C" fn rs_tv_check_for_string_or_number_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_string_or_number_arg_impl(args, idx)
}

/// Check if args[idx] is a buffer (string or number). Return OK if valid, FAIL if not.
#[export_name = "tv_check_for_buffer_arg"]
pub extern "C" fn rs_tv_check_for_buffer_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_string_or_number_arg_impl(args, idx)
}

/// Check if args[idx] is a line number (string or number). Return OK if valid, FAIL if not.
#[export_name = "tv_check_for_lnum_arg"]
pub extern "C" fn rs_tv_check_for_lnum_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_string_or_number_arg_impl(args, idx)
}

/// Check if args[idx] is a string or list. Return OK if valid, FAIL if not.
#[inline]
fn tv_check_for_string_or_list_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    let t = tv_type_impl(tv);
    if t == VarType::String || t == VarType::List {
        OK
    } else {
        unsafe { nvim_typval_error_string_or_list_required(idx + 1) };
        FAIL
    }
}

/// FFI wrapper: check if args[idx] is a string or list.
#[export_name = "tv_check_for_string_or_list_arg"]
pub extern "C" fn rs_tv_check_for_string_or_list_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_string_or_list_arg_impl(args, idx)
}

/// Check for optional string or list at args[idx]. VAR_UNKNOWN is OK.
#[inline]
fn tv_check_for_opt_string_or_list_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    if tv_type_impl(tv) == VarType::Unknown {
        OK
    } else {
        tv_check_for_string_or_list_arg_impl(args, idx)
    }
}

/// FFI wrapper: check for optional string or list at args[idx].
#[export_name = "tv_check_for_opt_string_or_list_arg"]
pub extern "C" fn rs_tv_check_for_opt_string_or_list_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_opt_string_or_list_arg_impl(args, idx)
}

/// Check if args[idx] is a string, list, or blob. Return OK if valid, FAIL if not.
#[inline]
fn tv_check_for_string_or_list_or_blob_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    let t = tv_type_impl(tv);
    if t == VarType::String || t == VarType::List || t == VarType::Blob {
        OK
    } else {
        unsafe { nvim_typval_error_string_list_or_blob_required(idx + 1) };
        FAIL
    }
}

/// FFI wrapper: check if args[idx] is a string, list, or blob.
#[export_name = "tv_check_for_string_or_list_or_blob_arg"]
pub extern "C" fn rs_tv_check_for_string_or_list_or_blob_arg(
    args: TypevalHandle,
    idx: c_int,
) -> c_int {
    tv_check_for_string_or_list_or_blob_arg_impl(args, idx)
}

/// Check if args[idx] is a string, list, or dict. Return OK if valid, FAIL if not.
#[inline]
fn tv_check_for_string_or_list_or_dict_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    let t = tv_type_impl(tv);
    if t == VarType::String || t == VarType::List || t == VarType::Dict {
        OK
    } else {
        unsafe { nvim_typval_error_string_list_or_dict_required(idx + 1) };
        FAIL
    }
}

/// FFI wrapper: check if args[idx] is a string, list, or dict.
#[export_name = "tv_check_for_string_or_list_or_dict_arg"]
pub extern "C" fn rs_tv_check_for_string_or_list_or_dict_arg(
    args: TypevalHandle,
    idx: c_int,
) -> c_int {
    tv_check_for_string_or_list_or_dict_arg_impl(args, idx)
}

/// Check if args[idx] is a string or function reference. Return OK if valid, FAIL if not.
#[inline]
fn tv_check_for_string_or_func_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    let t = tv_type_impl(tv);
    if t == VarType::Partial || t == VarType::Func || t == VarType::String {
        OK
    } else {
        unsafe { nvim_typval_error_string_or_func_required(idx + 1) };
        FAIL
    }
}

/// FFI wrapper: check if args[idx] is a string or function reference.
#[export_name = "tv_check_for_string_or_func_arg"]
pub extern "C" fn rs_tv_check_for_string_or_func_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_string_or_func_arg_impl(args, idx)
}

/// Check if args[idx] is a list or blob. Return OK if valid, FAIL if not.
#[inline]
fn tv_check_for_list_or_blob_arg_impl(args: TypevalHandle, idx: c_int) -> c_int {
    let tv = get_arg(args, idx);
    let t = tv_type_impl(tv);
    if t == VarType::List || t == VarType::Blob {
        OK
    } else {
        unsafe { nvim_typval_error_list_or_blob_required(idx + 1) };
        FAIL
    }
}

/// FFI wrapper: check if args[idx] is a list or blob.
#[export_name = "tv_check_for_list_or_blob_arg"]
pub extern "C" fn rs_tv_check_for_list_or_blob_arg(args: TypevalHandle, idx: c_int) -> c_int {
    tv_check_for_list_or_blob_arg_impl(args, idx)
}

// =============================================================================
// Type validation functions (tv_check_num, tv_check_str, tv_check_str_or_nr)
// =============================================================================

/// Check that given value is a number or can be converted to it.
///
/// Types that can be used as numbers: Number, Bool, Special, String.
/// Types that emit errors: Func, Partial, List, Dict, Float, Blob, Unknown.
#[inline]
fn tv_check_num_impl(tv: TypevalHandle) -> bool {
    let t = tv_type_impl(tv);
    match t {
        VarType::Number | VarType::Bool | VarType::Special | VarType::String => true,
        VarType::Func | VarType::Partial => {
            unsafe { nvim_typval_error_using_funcref_as_number() };
            false
        }
        VarType::List => {
            unsafe { nvim_typval_error_using_list_as_number() };
            false
        }
        VarType::Dict => {
            unsafe { nvim_typval_error_using_dict_as_number() };
            false
        }
        VarType::Float => {
            unsafe { nvim_typval_error_using_float_as_number() };
            false
        }
        VarType::Blob => {
            unsafe { nvim_typval_error_using_blob_as_number() };
            false
        }
        VarType::Unknown => {
            unsafe { nvim_typval_error_using_invalid_as_number() };
            false
        }
    }
}

/// FFI wrapper: check if typval can be used as a number.
#[export_name = "tv_check_num"]
pub extern "C" fn rs_tv_check_num(tv: TypevalHandle) -> bool {
    tv_check_num_impl(tv)
}

/// Check that given value is a string or can be "cast" to it.
///
/// Types that can be used as strings: Number, Bool, Special, String, Float.
/// Types that emit errors: Func, Partial, List, Dict, Blob, Unknown.
#[inline]
fn tv_check_str_impl(tv: TypevalHandle) -> bool {
    let t = tv_type_impl(tv);
    match t {
        VarType::Number | VarType::Bool | VarType::Special | VarType::String | VarType::Float => {
            true
        }
        VarType::Func | VarType::Partial => {
            unsafe { nvim_typval_error_using_funcref_as_string() };
            false
        }
        VarType::List => {
            unsafe { nvim_typval_error_using_list_as_string() };
            false
        }
        VarType::Dict => {
            unsafe { nvim_typval_error_using_dict_as_string() };
            false
        }
        VarType::Blob => {
            unsafe { nvim_typval_error_using_blob_as_string() };
            false
        }
        VarType::Unknown => {
            unsafe { nvim_typval_error_using_invalid_as_string() };
            false
        }
    }
}

/// FFI wrapper: check if typval can be used as a string.
#[export_name = "tv_check_str"]
pub extern "C" fn rs_tv_check_str(tv: TypevalHandle) -> bool {
    tv_check_str_impl(tv)
}

/// Check that given value is a number or string.
///
/// This is stricter than tv_check_num/tv_check_str: only VAR_NUMBER and VAR_STRING
/// are accepted. Other types emit type-specific error messages.
#[inline]
fn tv_check_str_or_nr_impl(tv: TypevalHandle) -> bool {
    let t = tv_type_impl(tv);
    match t {
        VarType::Number | VarType::String => true,
        VarType::Float => {
            unsafe { nvim_typval_error_str_or_nr_float() };
            false
        }
        VarType::Func | VarType::Partial => {
            unsafe { nvim_typval_error_str_or_nr_funcref() };
            false
        }
        VarType::List => {
            unsafe { nvim_typval_error_str_or_nr_list() };
            false
        }
        VarType::Dict => {
            unsafe { nvim_typval_error_str_or_nr_dict() };
            false
        }
        VarType::Blob => {
            unsafe { nvim_typval_error_str_or_nr_blob() };
            false
        }
        VarType::Bool => {
            unsafe { nvim_typval_error_str_or_nr_bool() };
            false
        }
        VarType::Special => {
            unsafe { nvim_typval_error_str_or_nr_special() };
            false
        }
        VarType::Unknown => {
            unsafe { nvim_typval_error_str_or_nr_unknown() };
            false
        }
    }
}

/// FFI wrapper: check if typval is a number or string.
#[export_name = "tv_check_str_or_nr"]
pub extern "C" fn rs_tv_check_str_or_nr(tv: TypevalHandle) -> bool {
    tv_check_str_or_nr_impl(tv)
}

// =============================================================================
// Expression validity checking
// =============================================================================

/// Check if a typval is a valid expression to pass to eval_expr_typval()
/// or eval_expr_to_bool(). An empty string returns false.
///
/// Returns true if:
/// - v_type is not VAR_UNKNOWN AND
/// - Either v_type is not VAR_STRING, OR the string is non-NULL and non-empty
#[inline]
fn eval_expr_valid_arg_impl(tv: TypevalHandle) -> bool {
    if tv.is_null() {
        return false;
    }
    let t = tv_type_impl(tv);
    if t == VarType::Unknown {
        return false;
    }
    if t != VarType::String {
        return true;
    }
    // For strings, check that it's non-NULL and non-empty
    let s = unsafe { nvim_tv_get_string_ptr(tv) };
    if s.is_null() {
        return false;
    }
    // Check first byte is not NUL (empty string)
    unsafe { *s != 0 }
}

/// FFI wrapper: check if typval is a valid expression argument.
#[no_mangle]
pub extern "C" fn rs_eval_expr_valid_arg(tv: TypevalHandle) -> c_int {
    c_int::from(eval_expr_valid_arg_impl(tv))
}

// =============================================================================
// Phase 2: Blob alloc/free/unref/alloc_ret/copy and f_blob2list/f_list2blob
// =============================================================================

/// FFI export: tv_blob_alloc - allocate an empty blob.
#[export_name = "tv_blob_alloc"]
pub extern "C" fn rs_tv_blob_alloc() -> BlobHandle {
    unsafe { nvim_blob_alloc_impl() }
}

/// FFI export: tv_blob_free - free a blob (ignore refcount).
#[export_name = "tv_blob_free"]
pub unsafe extern "C" fn rs_tv_blob_free(b: BlobHandle) {
    unsafe { nvim_blob_free_impl(b) };
}

/// FFI export: tv_blob_unref - decrement refcount and free if zero.
#[export_name = "tv_blob_unref"]
pub unsafe extern "C" fn rs_tv_blob_unref(b: BlobHandle) {
    if !b.is_null() && unsafe { nvim_blob_dec_refcount(b) } <= 0 {
        unsafe { nvim_blob_free_impl(b) };
    }
}

/// FFI export: tv_blob_alloc_ret - allocate a blob and set as return value.
#[export_name = "tv_blob_alloc_ret"]
pub unsafe extern "C" fn rs_tv_blob_alloc_ret(ret_tv: TypevalHandle) -> BlobHandle {
    let b = unsafe { nvim_blob_alloc_impl() };
    unsafe { nvim_tv_set_blob(ret_tv, b) };
    b
}

/// FFI export: tv_blob_copy - copy a blob typval to a new typval.
#[export_name = "tv_blob_copy"]
pub unsafe extern "C" fn rs_tv_blob_copy(from: BlobHandle, to: TypevalHandle) {
    // Set v_type=VAR_BLOB, v_lock=VAR_UNLOCKED
    unsafe { nvim_tv_set_blob(to, BlobHandle::null()) }; // sets type=VAR_BLOB, v_blob=NULL
    unsafe { nvim_tv_set_lock(to, 0) }; // VAR_UNLOCKED = 0
    if !from.is_null() {
        let new_b = unsafe { nvim_blob_alloc_impl() };
        let len = unsafe { nvim_blob_get_len(from) };
        if len > 0 {
            let data = unsafe { nvim_blob_xmemdup_ga_data(from, len) };
            unsafe { nvim_blob_set_ga_data(new_b, data) };
        }
        unsafe { nvim_blob_set_ga_len(new_b, len) };
        unsafe { nvim_blob_set_ga_maxlen(new_b, len) };
        // Increment new_b refcount and set it in 'to'
        unsafe { nvim_tv_set_blob(to, new_b) };
    }
}

/// FFI export: f_blob2list - VimL blob2list() function.
#[export_name = "f_blob2list"]
pub unsafe extern "C" fn rs_f_blob2list(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    _fptr: *const std::ffi::c_void,
) {
    let l = unsafe { nvim_tv_list_alloc_ret(rettv, -3) }; // kListLenMayKnow = -3

    let arg0 = unsafe { nvim_typval_array_get(argvars, 0) };
    if tv_check_for_blob_arg_impl(argvars, 0) == FAIL {
        return;
    }

    let blob = unsafe { nvim_tv_get_blob(arg0) };
    let len = tv_blob_len_impl(blob);
    for i in 0..len {
        let byte = i64::from(unsafe { nvim_blob_get_byte(blob, i) });
        unsafe { nvim_tv_list_append_number(l.0.cast_mut(), byte as c_int) };
    }
}

/// FFI export: f_list2blob - VimL list2blob() function.
#[export_name = "f_list2blob"]
pub unsafe extern "C" fn rs_f_list2blob(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    _fptr: *const std::ffi::c_void,
) {
    let blob = unsafe { nvim_tv_blob_alloc_ret_handle(rettv) };

    if tv_check_for_list_arg_impl(argvars, 0) == FAIL {
        return;
    }

    let arg0 = unsafe { nvim_typval_array_get(argvars, 0) };
    let l = unsafe { nvim_tv_get_list(arg0) };
    if l.is_null() {
        return;
    }

    // Iterate list items
    let mut li = tv_list_first_impl(l);
    while !li.is_null() {
        let item_tv = tv_listitem_tv_impl(li);
        let mut error = false;
        let n = tv_get_number_chk_impl(item_tv, &raw mut error);
        if error || !(0..=255).contains(&n) {
            if !error {
                unsafe { nvim_semsg_blob_invalid_value(n) };
            }
            unsafe { nvim_blob_ga_clear_only(blob) };
            return;
        }
        unsafe { nvim_blob_ga_append(blob, n as u8) };
        li = tv_listitem_next_impl(li);
    }
}

/// Helper: call tv_blob_alloc_ret and return the BlobHandle.
unsafe fn nvim_tv_blob_alloc_ret_handle(ret_tv: TypevalHandle) -> BlobHandle {
    unsafe { rs_tv_blob_alloc_ret(ret_tv) }
}

// =============================================================================
// Phase 1: Get functions (tv_get_number_chk, tv_get_string_buf_chk, etc.)
// =============================================================================

/// NUMBUFLEN constant matching C's value.
const NUMBUFLEN: usize = 65;

/// Get the number value of a Vimscript object with error checking.
///
/// This is the core implementation that handles all type cases.
/// Returns a number value; sets *ret_error=true on type error.
fn tv_get_number_chk_impl(tv: TypevalHandle, ret_error: *mut bool) -> i64 {
    match tv_type_impl(tv) {
        VarType::Func | VarType::Partial => {
            unsafe { nvim_typval_error_using_funcref_as_number() };
        }
        VarType::List => {
            unsafe { nvim_typval_error_using_list_as_number() };
        }
        VarType::Dict => {
            unsafe { nvim_typval_error_using_dict_as_number() };
        }
        VarType::Float => {
            unsafe { nvim_typval_error_using_float_as_number() };
        }
        VarType::Blob => {
            unsafe { nvim_typval_error_using_blob_as_number() };
        }
        VarType::Number => {
            return unsafe { nvim_tv_get_number(tv) };
        }
        VarType::String => {
            let s = unsafe { nvim_tv_get_string_ptr(tv) };
            let mut n: i64 = 0;
            unsafe { nvim_vim_str2nr(s, &raw mut n) };
            return n;
        }
        VarType::Bool => {
            let b = unsafe { nvim_tv_get_bool(tv) };
            return i64::from(b);
        }
        VarType::Special => {
            return 0;
        }
        VarType::Unknown => {
            unsafe { nvim_emsg_get_number_unknown() };
        }
    }
    // Type error path
    if !ret_error.is_null() {
        unsafe { *ret_error = true };
    }
    if ret_error.is_null() {
        -1
    } else {
        0
    }
}

/// FFI export: tv_get_number_chk - get number from typval with error checking.
#[export_name = "tv_get_number_chk"]
pub unsafe extern "C" fn rs_tv_get_number_chk(tv: TypevalHandle, ret_error: *mut bool) -> i64 {
    tv_get_number_chk_impl(tv, ret_error)
}

/// FFI export: tv_get_number - get number from typval (no error output).
#[export_name = "tv_get_number"]
pub extern "C" fn rs_tv_get_number(tv: TypevalHandle) -> i64 {
    tv_get_number_chk_impl(tv, std::ptr::null_mut())
}

/// FFI export: tv_get_bool - get number/bool value from typval.
#[export_name = "tv_get_bool"]
pub unsafe extern "C" fn rs_tv_get_bool(tv: TypevalHandle) -> i64 {
    tv_get_number_chk_impl(tv, std::ptr::null_mut())
}

/// FFI export: tv_get_bool_chk - get number/bool value with error output.
#[export_name = "tv_get_bool_chk"]
pub unsafe extern "C" fn rs_tv_get_bool_chk(tv: TypevalHandle, ret_error: *mut bool) -> i64 {
    tv_get_number_chk_impl(tv, ret_error)
}

/// Get string representation of a typval into buf.
/// Returns NULL on type error, valid string pointer otherwise.
unsafe fn tv_get_string_buf_chk_impl(tv: TypevalHandle, buf: *mut c_char) -> *const c_char {
    match tv_type_impl(tv) {
        VarType::Number => {
            let n = unsafe { nvim_tv_get_number(tv) };
            unsafe { nvim_format_number(n, buf, NUMBUFLEN as c_int) };
            buf
        }
        VarType::Float => {
            let f = unsafe { nvim_tv_get_float(tv) };
            unsafe { nvim_format_float(f, buf, NUMBUFLEN as c_int) };
            buf
        }
        VarType::String => {
            let s = unsafe { nvim_tv_get_string_ptr(tv) };
            if s.is_null() {
                // Return pointer to empty string literal
                b"\0".as_ptr().cast::<c_char>()
            } else {
                s
            }
        }
        VarType::Bool => {
            let b = unsafe { nvim_tv_get_bool(tv) };
            let name = unsafe { nvim_get_bool_var_name(b) };
            // strcpy into buf
            let name_len = unsafe { libc_strlen(name) };
            unsafe {
                std::ptr::copy_nonoverlapping(name, buf, name_len + 1);
            }
            buf
        }
        VarType::Special => {
            let s = unsafe {
                let sv = nvim_tv_get_special(tv);
                nvim_get_special_var_name(sv)
            };
            let s_len = unsafe { libc_strlen(s) };
            unsafe {
                std::ptr::copy_nonoverlapping(s, buf, s_len + 1);
            }
            buf
        }
        VarType::Partial | VarType::Func => {
            unsafe { nvim_typval_error_using_funcref_as_string() };
            std::ptr::null()
        }
        VarType::List => {
            unsafe { nvim_typval_error_using_list_as_string() };
            std::ptr::null()
        }
        VarType::Dict => {
            unsafe { nvim_typval_error_using_dict_as_string() };
            std::ptr::null()
        }
        VarType::Blob => {
            unsafe { nvim_typval_error_using_blob_as_string() };
            std::ptr::null()
        }
        VarType::Unknown => {
            unsafe { nvim_typval_error_using_invalid_as_string() };
            std::ptr::null()
        }
    }
}

/// FFI export: tv_get_string_buf_chk.
#[export_name = "tv_get_string_buf_chk"]
pub unsafe extern "C" fn rs_tv_get_string_buf_chk(
    tv: TypevalHandle,
    buf: *mut c_char,
) -> *const c_char {
    unsafe { tv_get_string_buf_chk_impl(tv, buf) }
}

/// FFI export: tv_get_string_chk - get string with error checking using static buffer.
#[export_name = "tv_get_string_chk"]
pub unsafe extern "C" fn rs_tv_get_string_chk(tv: TypevalHandle) -> *const c_char {
    // Use a thread-local static buffer matching C's mybuf[NUMBUFLEN]
    use std::cell::UnsafeCell;
    struct StaticBuf(UnsafeCell<[u8; NUMBUFLEN]>);
    unsafe impl Sync for StaticBuf {}
    static MYBUF: StaticBuf = StaticBuf(UnsafeCell::new([0u8; NUMBUFLEN]));
    let buf = unsafe { (*MYBUF.0.get()).as_mut_ptr().cast::<c_char>() };
    unsafe { tv_get_string_buf_chk_impl(tv, buf) }
}

/// FFI export: tv_get_string - get string, return empty string on error.
#[export_name = "tv_get_string"]
pub unsafe extern "C" fn rs_tv_get_string(tv: TypevalHandle) -> *const c_char {
    use std::cell::UnsafeCell;
    struct StaticBuf(UnsafeCell<[u8; NUMBUFLEN]>);
    unsafe impl Sync for StaticBuf {}
    static MYBUF: StaticBuf = StaticBuf(UnsafeCell::new([0u8; NUMBUFLEN]));
    let buf = unsafe { (*MYBUF.0.get()).as_mut_ptr().cast::<c_char>() };
    let res = unsafe { tv_get_string_buf_chk_impl(tv, buf) };
    if res.is_null() {
        b"\0".as_ptr().cast::<c_char>()
    } else {
        res
    }
}

/// FFI export: tv_get_string_buf - get string with caller-provided buffer.
#[export_name = "tv_get_string_buf"]
pub unsafe extern "C" fn rs_tv_get_string_buf(
    tv: TypevalHandle,
    buf: *mut c_char,
) -> *const c_char {
    let res = unsafe { tv_get_string_buf_chk_impl(tv, buf) };
    if res.is_null() {
        b"\0".as_ptr().cast::<c_char>()
    } else {
        res
    }
}

/// Get line number from a typval (line number or special string like "$", ".", etc.)
fn tv_get_lnum_impl(tv: TypevalHandle) -> i32 {
    let did_emsg_before = unsafe { nvim_did_emsg_check() };
    let lnum = tv_get_number_chk_impl(tv, std::ptr::null_mut()) as i32;
    if lnum <= 0
        && unsafe { nvim_did_emsg_check() } == did_emsg_before
        && tv_type_impl(tv) != VarType::Number
    {
        let mut fnum: c_int = 0;
        let pos_lnum = unsafe { nvim_tv_to_lnum_pos(tv, &raw mut fnum) };
        if pos_lnum != 0 {
            return pos_lnum;
        }
    }
    lnum
}

/// FFI export: tv_get_lnum - get line number from typval.
#[export_name = "tv_get_lnum"]
pub extern "C" fn rs_tv_get_lnum(tv: TypevalHandle) -> i32 {
    tv_get_lnum_impl(tv)
}

/// Get line number from typval with buffer for "$" handling.
unsafe fn tv_get_lnum_buf_impl(tv: TypevalHandle, buf: *const std::ffi::c_void) -> i32 {
    // Check for "$" special string
    if tv_type_impl(tv) == VarType::String {
        let s = unsafe { nvim_tv_get_string_ptr(tv) };
        if !s.is_null() {
            // Check s == "$\0"
            let b0 = unsafe { *s as u8 };
            let b1 = unsafe { *s.offset(1) as u8 };
            if b0 == b'$' && b1 == b'\0' && !buf.is_null() {
                return unsafe { nvim_buf_get_ml_line_count(buf) };
            }
        }
    }
    tv_get_number_chk_impl(tv, std::ptr::null_mut()) as i32
}

/// FFI export: tv_get_lnum_buf - get line number with buffer for "$".
#[export_name = "tv_get_lnum_buf"]
pub unsafe extern "C" fn rs_tv_get_lnum_buf(
    tv: TypevalHandle,
    buf: *const std::ffi::c_void,
) -> i32 {
    unsafe { tv_get_lnum_buf_impl(tv, buf) }
}

// Helper: strlen for C strings
unsafe fn libc_strlen(s: *const c_char) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut len = 0usize;
    while unsafe { *s.add(len) } != 0 {
        len += 1;
    }
    len
}

// Helper: get v_special field from typval
unsafe fn nvim_tv_get_special(tv: TypevalHandle) -> c_int {
    // v_special is an int field at same offset as v_bool (in the vval union)
    // Reuse the nvim_tv_get_bool accessor (they are the same int in the union)
    // Note: special values use v_special which is typed like BoolVarValue
    unsafe { nvim_tv_get_bool(tv) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vartype_from_c_int() {
        assert_eq!(VarType::from_c_int(0), Some(VarType::Unknown));
        assert_eq!(VarType::from_c_int(1), Some(VarType::Number));
        assert_eq!(VarType::from_c_int(2), Some(VarType::String));
        assert_eq!(VarType::from_c_int(10), Some(VarType::Blob));
        assert_eq!(VarType::from_c_int(99), None);
    }

    #[test]
    fn test_typval_handle_null() {
        let handle = unsafe { TypevalHandle::from_ptr(std::ptr::null()) };
        assert!(handle.is_null());
        assert_eq!(tv_type_impl(handle), VarType::Unknown);
        assert!(!tv_is_number_impl(handle));
        assert!(!tv_is_string_impl(handle));
        assert!(!tv_is_float_impl(handle));
        assert!(tv_is_empty_impl(handle));
    }

    #[test]
    fn test_list_handle_null() {
        let handle = unsafe { ListHandle::from_ptr(std::ptr::null()) };
        assert!(handle.is_null());
        assert_eq!(tv_list_len_impl(handle), 0);
        assert_eq!(tv_list_locked_impl(handle), 2); // VAR_FIXED
        assert!(!tv_list_has_watchers_impl(handle));
        assert!(tv_list_first_impl(handle).is_null());
        assert!(tv_list_last_impl(handle).is_null());
    }

    #[test]
    fn test_dict_handle_null() {
        let handle = unsafe { DictHandle::from_ptr(std::ptr::null()) };
        assert!(handle.is_null());
        assert_eq!(tv_dict_len_impl(handle), 0);
        assert_eq!(tv_dict_locked_impl(handle), 2); // VAR_FIXED
        assert!(!tv_dict_is_watched_impl(handle));
    }

    #[test]
    fn test_blob_handle_null() {
        let handle = unsafe { BlobHandle::from_ptr(std::ptr::null()) };
        assert!(handle.is_null());
        assert_eq!(tv_blob_len_impl(handle), 0);
        assert_eq!(tv_blob_locked_impl(handle), 2); // VAR_FIXED
    }

    #[test]
    fn test_list_uidx() {
        // With null list (len=0), all indices should return -1
        let null_list = unsafe { ListHandle::from_ptr(std::ptr::null()) };
        assert_eq!(tv_list_uidx_impl(null_list, 0), -1);
        assert_eq!(tv_list_uidx_impl(null_list, -1), -1);
    }

    #[test]
    fn test_ok_fail_constants() {
        // OK should be 0, FAIL should be -1
        assert_eq!(OK, 0);
        assert_eq!(FAIL, -1);
    }

    #[test]
    fn test_vartype_enum_values() {
        // VarType enum values should match C definitions
        assert_eq!(VarType::Unknown as i32, 0);
        assert_eq!(VarType::Number as i32, 1);
        assert_eq!(VarType::String as i32, 2);
        assert_eq!(VarType::Func as i32, 3);
        assert_eq!(VarType::List as i32, 4);
        assert_eq!(VarType::Dict as i32, 5);
        assert_eq!(VarType::Float as i32, 6);
        assert_eq!(VarType::Bool as i32, 7);
        assert_eq!(VarType::Special as i32, 8);
        assert_eq!(VarType::Partial as i32, 9);
        assert_eq!(VarType::Blob as i32, 10);
    }

    #[test]
    fn test_listitem_handle_null() {
        let handle = ListItemHandle(std::ptr::null());
        assert!(handle.is_null());
        assert!(tv_listitem_next_impl(handle).is_null());
        assert!(tv_listitem_prev_impl(handle).is_null());
        assert!(tv_listitem_tv_impl(handle).is_null());
    }

    // =============================================================================
    // Phase 28.1 new tests
    // =============================================================================

    #[test]
    fn test_vartype_name() {
        assert_eq!(VarType::Unknown.name(), "unknown");
        assert_eq!(VarType::Number.name(), "Number");
        assert_eq!(VarType::String.name(), "String");
        assert_eq!(VarType::Func.name(), "Funcref");
        assert_eq!(VarType::List.name(), "List");
        assert_eq!(VarType::Dict.name(), "Dict");
        assert_eq!(VarType::Float.name(), "Float");
        assert_eq!(VarType::Bool.name(), "Boolean");
        assert_eq!(VarType::Special.name(), "Special");
        assert_eq!(VarType::Partial.name(), "Partial");
        assert_eq!(VarType::Blob.name(), "Blob");
    }

    #[test]
    fn test_vartype_display() {
        assert_eq!(format!("{}", VarType::Number), "Number");
        assert_eq!(format!("{}", VarType::String), "String");
    }

    #[test]
    fn test_bool_var_value() {
        assert!(!BoolVarValue::False.as_bool());
        assert!(BoolVarValue::True.as_bool());

        let true_val: BoolVarValue = true.into();
        let false_val: BoolVarValue = false.into();
        assert_eq!(true_val, BoolVarValue::True);
        assert_eq!(false_val, BoolVarValue::False);

        let b: bool = BoolVarValue::True.into();
        assert!(b);
        let b: bool = BoolVarValue::False.into();
        assert!(!b);
    }

    #[test]
    fn test_special_var_value() {
        assert_eq!(SpecialVarValue::Null as i32, 0);
    }

    #[test]
    fn test_var_lock_status() {
        assert_eq!(VarLockStatus::from_c_int(0), Some(VarLockStatus::Unlocked));
        assert_eq!(VarLockStatus::from_c_int(1), Some(VarLockStatus::Locked));
        assert_eq!(VarLockStatus::from_c_int(2), Some(VarLockStatus::Fixed));
        assert_eq!(VarLockStatus::from_c_int(99), None);

        assert!(!VarLockStatus::Unlocked.is_locked());
        assert!(VarLockStatus::Locked.is_locked());
        assert!(VarLockStatus::Fixed.is_locked());
    }

    #[test]
    fn test_type_func_value() {
        assert_eq!(TypeFuncValue::Number.as_i64(), 0);
        assert_eq!(TypeFuncValue::String.as_i64(), 1);
        assert_eq!(TypeFuncValue::Func.as_i64(), 2);
        assert_eq!(TypeFuncValue::List.as_i64(), 3);
        assert_eq!(TypeFuncValue::Dict.as_i64(), 4);
        assert_eq!(TypeFuncValue::Float.as_i64(), 5);
        assert_eq!(TypeFuncValue::Bool.as_i64(), 6);
        assert_eq!(TypeFuncValue::Special.as_i64(), 7);
        assert_eq!(TypeFuncValue::Blob.as_i64(), 10);

        // Test from_var_type
        assert_eq!(
            TypeFuncValue::from_var_type(VarType::Number),
            Some(TypeFuncValue::Number)
        );
        assert_eq!(
            TypeFuncValue::from_var_type(VarType::Func),
            Some(TypeFuncValue::Func)
        );
        assert_eq!(
            TypeFuncValue::from_var_type(VarType::Partial),
            Some(TypeFuncValue::Func)
        );
        assert_eq!(TypeFuncValue::from_var_type(VarType::Unknown), None);
    }

    #[test]
    fn test_typeval_basic() {
        // Test number
        let num = TypeVal::number(42);
        assert_eq!(num.var_type(), VarType::Number);
        assert!(!num.is_empty());
        assert!(num.is_truthy());
        assert_eq!(num.to_number(), 42);
        assert!((num.to_float() - 42.0).abs() < f64::EPSILON);

        // Test zero is empty
        let zero = TypeVal::number(0);
        assert!(zero.is_empty());
        assert!(!zero.is_truthy());

        // Test float
        let float = TypeVal::float(2.5);
        assert_eq!(float.var_type(), VarType::Float);
        assert!(!float.is_empty());
        assert!((float.to_float() - 2.5).abs() < f64::EPSILON);
        assert_eq!(float.to_number(), 2);

        // Test zero float is empty
        let zero_float = TypeVal::float(0.0);
        assert!(zero_float.is_empty());
    }

    #[test]
    fn test_typeval_bool() {
        let t = TypeVal::vim_true();
        let f = TypeVal::vim_false();

        assert_eq!(t.var_type(), VarType::Bool);
        assert_eq!(f.var_type(), VarType::Bool);
        assert!(t.is_truthy());
        assert!(!f.is_truthy());
        assert!(!t.is_empty());
        assert!(f.is_empty());
        assert_eq!(t.to_number(), 1);
        assert_eq!(f.to_number(), 0);
    }

    #[test]
    fn test_typeval_special() {
        let null = TypeVal::vim_null();
        assert_eq!(null.var_type(), VarType::Special);
        assert!(null.is_empty());
        assert!(!null.is_truthy());
    }

    #[test]
    fn test_typeval_string() {
        let s = TypeVal::string("hello");
        assert_eq!(s.var_type(), VarType::String);
        assert!(!s.is_empty());
        assert!(s.is_truthy());

        let empty = TypeVal::string("");
        assert!(empty.is_empty());
        assert!(!empty.is_truthy());
    }

    #[test]
    fn test_typeval_from_traits() {
        let n: TypeVal = 42i64.into();
        assert_eq!(n.var_type(), VarType::Number);

        let n32: TypeVal = 42i32.into();
        assert_eq!(n32.var_type(), VarType::Number);

        let f: TypeVal = 2.5f64.into();
        assert_eq!(f.var_type(), VarType::Float);

        let b: TypeVal = true.into();
        assert_eq!(b.var_type(), VarType::Bool);

        let s: TypeVal = "test".into();
        assert_eq!(s.var_type(), VarType::String);

        let s2: TypeVal = std::string::String::from("test2").into();
        assert_eq!(s2.var_type(), VarType::String);
    }

    #[test]
    fn test_typeval_default() {
        let def = TypeVal::default();
        assert_eq!(def.var_type(), VarType::Unknown);
        assert!(def.is_empty());
    }

    #[test]
    fn test_typeval_display() {
        assert_eq!(format!("{}", TypeVal::number(42)), "42");
        assert_eq!(format!("{}", TypeVal::float(2.5)), "2.5");
        assert_eq!(format!("{}", TypeVal::string("hello")), "'hello'");
        assert_eq!(format!("{}", TypeVal::vim_true()), "v:true");
        assert_eq!(format!("{}", TypeVal::vim_false()), "v:false");
        assert_eq!(format!("{}", TypeVal::vim_null()), "v:null");
        assert_eq!(format!("{}", TypeVal::unknown()), "<unknown>");
    }

    #[test]
    fn test_typeval_type_func_value() {
        assert_eq!(TypeVal::number(1).type_func_value(), TypeFuncValue::Number);
        assert_eq!(TypeVal::float(1.0).type_func_value(), TypeFuncValue::Float);
        assert_eq!(
            TypeVal::string("x").type_func_value(),
            TypeFuncValue::String
        );
        assert_eq!(TypeVal::vim_true().type_func_value(), TypeFuncValue::Bool);
        assert_eq!(
            TypeVal::vim_null().type_func_value(),
            TypeFuncValue::Special
        );
    }

    #[test]
    fn test_partial_handle_null() {
        let handle = unsafe { PartialHandle::from_ptr(std::ptr::null()) };
        assert!(handle.is_null());
        assert!(handle.as_ptr().is_null());
    }
}
