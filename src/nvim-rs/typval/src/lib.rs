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
    fn nvim_semsg_list_index_out_of_range(idx: c_int);
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
        unsafe { nvim_semsg_list_index_out_of_range(n) };
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
