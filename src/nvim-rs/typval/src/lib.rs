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

use std::ffi::{c_char, c_int, c_void};
use std::fmt;

/// Rust mirror of C `garray_T` (growing array).
///
/// Must match the layout in `src/nvim-rs/collections/src/garray.rs` and
/// `src/nvim/garray.h` exactly.
#[repr(C)]
#[allow(clippy::struct_field_names)]
struct GArrayT {
    ga_len: c_int,
    ga_maxlen: c_int,
    ga_itemsize: c_int,
    ga_growsize: c_int,
    ga_data: *mut c_void,
}

impl GArrayT {
    const fn new() -> Self {
        Self {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 1,
            ga_data: std::ptr::null_mut(),
        }
    }
}

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

/// Opaque handle for `hashitem_T*` (hashtab item).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HashItemHandle(*const std::ffi::c_void);

impl HashItemHandle {
    /// Create a new hash item handle from a raw pointer.
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
    // nvim_value_check_lock_translated removed: value_check_lock_impl is now native Rust (Phase 3)
    // nvim_semsg_blobidx: deleted (Phase 9), use typval_err_blobidx directly
    // nvim_emsg_blob_wrong_bytes: deleted (Phase 9), use typval_err_blob_wrong_bytes directly

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
    // nvim_emsg_float_*: deleted (Phase 9), use typval_err_float_* inline helpers directly
    // nvim_value_check_lock removed: value_check_lock is now native Rust (Phase 3)
    fn nvim_tv_get_v_lock(tv: TypevalHandle) -> c_int;

    // Phase 3: tv_item_lock / value_check_lock_impl native accessors
    fn nvim_gettext_value_locked() -> *const c_char;
    fn nvim_gettext_value_fixed() -> *const c_char;
    fn nvim_gettext_unknown() -> *const c_char;
    // nvim_emsg_item_lock_nested: deleted (Phase 9), use typval_err_item_lock_nested directly
    fn nvim_list_set_lock(l: ListHandle, lock: c_int);
    fn nvim_list_set_refcount(l: ListHandle, rc: c_int);
    fn nvim_blob_set_lock(b: BlobHandle, lock: c_int);
    fn nvim_list_get_refcount(l: ListHandle) -> c_int;
    fn nvim_dict_get_refcount(d: DictHandle) -> c_int;
    fn nvim_blob_get_bv_refcount(b: BlobHandle) -> c_int;

    // Phase 1 accessor helpers for get functions
    fn nvim_format_number(n: i64, buf: *mut c_char, buflen: c_int);
    fn nvim_format_float(f: f64, buf: *mut c_char, buflen: c_int);
    fn nvim_get_bool_var_name(b: c_int) -> *const c_char;
    fn nvim_get_special_var_name(s: c_int) -> *const c_char;
    fn nvim_vim_str2nr(s: *const c_char, out: *mut i64);
    fn nvim_tv_to_lnum_pos(tv: TypevalHandle, ret_fnum: *mut c_int) -> i32;
    static mut did_emsg: c_int;
    fn nvim_buf_get_ml_line_count(buf: *const std::ffi::c_void) -> i32;
    // nvim_emsg_get_number_unknown: deleted (Phase 9), use typval_err_get_number_unknown directly

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
    // nvim_semsg_blob_invalid_value: deleted (Phase 9), use typval_err_blob_invalid_value directly
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
    #[link_name = "xstrdup"]
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
    // nvim_tv_clear deleted (Phase 2): tv_clear migrated to Rust, call rs_tv_clear directly
    fn nvim_list_item_free(li: ListItemHandle);
    fn nvim_list_init_static_impl(l: ListHandle);
    fn nvim_staticlist10_clear(sl: *mut c_void);
    fn nvim_staticlist10_get_item(sl: *mut c_void, i: c_int) -> ListItemHandle;
    fn nvim_staticlist10_get_list(sl: *mut c_void) -> ListHandle;
    fn nvim_do_not_free_cnt() -> c_int;
    fn nvim_list_copy_shallow(l: ListHandle) -> ListHandle;
    fn nvim_tv_set_list_vval(tv: TypevalHandle, l: ListHandle);

    // Phase 6c accessor helpers for slice/range/flatten/remove
    fn nvim_tv_list_set_ret(tv: TypevalHandle, l: ListHandle);
    #[link_name = "xfree"]
    fn nvim_xfree(ptr: *mut std::ffi::c_void);
    fn nvim_got_int() -> c_int;
    fn nvim_fast_breakcheck();
    // nvim_emsg_invrange: deleted (Phase 9), use typval_err_invrange directly
    fn nvim_tv_list_index_into_rettv(rettv: TypevalHandle, item: ListItemHandle);
    fn nvim_tv_listitem_move_to_rettv(rettv: TypevalHandle, item: ListItemHandle);

    // Phase 6d: tv_list_assign_range
    // nvim_emsg_list_more_items: deleted (Phase 9), use typval_err_list_more_items directly
    // nvim_emsg_list_not_enough_items: deleted (Phase 9), use typval_err_list_not_enough_items directly
    fn nvim_listitem_get_v_lock(li: ListItemHandle) -> c_int;
    fn eexe_mod_op(tv1: TypevalHandle, tv2: TypevalHandle, op: *const c_char) -> c_int;

    // Phase 6e: tv_list_copy, tv_list2items, tv_string2items, f_items
    fn var_item_copy(
        conv: *const std::ffi::c_void,
        from: TypevalHandle,
        to: TypevalHandle,
        deep: bool,
        copy_id: c_int,
    ) -> c_int;
    fn nvim_list_set_copyid(l: ListHandle, copyid: c_int);
    fn nvim_list_set_copylist(l: ListHandle, copy: ListHandle);
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    // Phase 6f: tv_dict_remove
    // nvim_di_check_fixed_translate inlined: var_check_fixed(di_flags at offset 16, name, TV_TRANSLATE)
    fn var_check_fixed(flags: c_int, name: *const c_char, name_len: usize) -> bool;
    // nvim_di_check_ro_translate inlined: var_check_ro(di_flags at offset 16, name, TV_TRANSLATE)
    fn var_check_ro(flags: c_int, name: *const c_char, name_len: usize) -> bool;
    // nvim_dictitem_move_tv_to_rettv inlined: memcpy(rettv, di_tv@offset0, 16); zero di_tv
    fn nvim_semsg_dictkey(key: *const c_char);
    fn nvim_semsg_toomanyarg(fname: *const c_char);
    // Phase 6h: tv_dict_set_keys_readonly hashtab iteration
    fn nvim_dict_get_ht_array(d: DictHandle) -> HashItemHandle;
    fn nvim_hashitem_get_key(hi: HashItemHandle) -> *const c_char;
    fn nvim_hash_removed_ptr() -> *const c_char;
    fn nvim_hashitem_set_ro_fix(hi: HashItemHandle);
    fn nvim_hashitem_next(hi: HashItemHandle) -> HashItemHandle;

    // Phase 6j: tv_dict_to_env (now Rust, kept as reference for callers)
    fn nvim_dictitem_format_env(di: DictItemHandle) -> *mut c_char;
    #[link_name = "xmalloc"]
    fn nvim_xmalloc(size: usize) -> *mut std::ffi::c_void;
    fn nvim_hashitem_to_dictitem(hi: HashItemHandle) -> DictItemHandle;

    // Phase 3 (f39b5673): new accessor helpers for migrating nvim_* functions to Rust
    fn nvim_dict_hash_find(d: DictHandle, key: *const c_char) -> HashItemHandle;
    fn nvim_dict_hash_find_len(d: DictHandle, key: *const c_char, len: usize) -> HashItemHandle;
    fn nvim_hashitem_is_empty(hi: HashItemHandle) -> c_int;
    fn nvim_listwatch_get_item(lw: *mut std::ffi::c_void) -> ListItemHandle;
    fn nvim_listwatch_set_item(lw: *mut std::ffi::c_void, item: ListItemHandle);
}

// =============================================================================
// Migrated from typval.c: Phase 3 (f39b5673)
//
// These functions were C-only ("accessor for Rust") wrappers with no C callers.
// Moving the logic to Rust eliminates the C bodies entirely.
// =============================================================================

/// Find a dictitem by key in a dict's hashtab.
/// Returns null handle if dict is NULL or key not found.
/// Exported as `nvim_dict_find` for existing `extern "C"` callers in this crate.
#[export_name = "nvim_dict_find"]
pub unsafe extern "C" fn rs_nvim_dict_find(
    d: DictHandle,
    key: *const c_char,
    len: isize,
) -> DictItemHandle {
    if d.is_null() {
        return DictItemHandle::null();
    }
    let hi = if len < 0 {
        unsafe { nvim_dict_hash_find(d, key) }
    } else {
        unsafe { nvim_dict_hash_find_len(d, key, len as usize) }
    };
    if unsafe { nvim_hashitem_is_empty(hi) } != 0 {
        return DictItemHandle::null();
    }
    unsafe { nvim_hashitem_to_dictitem(hi) }
}

/// Remove a key from a dict's hashtab (does NOT free the dictitem).
/// Exported as `nvim_dict_remove_key` for existing `extern "C"` callers in this crate.
#[export_name = "nvim_dict_remove_key"]
pub unsafe extern "C" fn rs_nvim_dict_remove_key(d: DictHandle, key: *const c_char) {
    let hi = unsafe { nvim_dict_hash_find(d, key) };
    if unsafe { nvim_hashitem_is_empty(hi) } == 0 {
        // Use nvim_dict_hash_remove (wraps hash_remove on d->dv_hashtab)
        unsafe { nvim_dict_hash_remove(d, hi) };
    }
}

/// Advance all list watchers that point to `item` past it to `item->li_next`.
/// Exported as `nvim_list_watch_fix` for existing `extern "C"` callers in this crate.
#[export_name = "nvim_list_watch_fix"]
pub unsafe extern "C" fn rs_nvim_list_watch_fix(l: ListHandle, item: ListItemHandle) {
    let mut lw = unsafe { nvim_list_get_watch(l) };
    while !lw.is_null() {
        let lw_item = unsafe { nvim_listwatch_get_item(lw) };
        if lw_item.as_ptr() == item.as_ptr() {
            let next = unsafe { nvim_listitem_get_next(item) };
            unsafe { nvim_listwatch_set_item(lw, next) };
        }
        lw = unsafe { nvim_listwatch_get_next(lw) };
    }
}

/// Clear a list item's tv payload and free the item.
/// Exported as `nvim_list_item_clear_free` for existing `extern "C"` callers in this crate.
#[export_name = "nvim_list_item_clear_free"]
pub unsafe extern "C" fn rs_nvim_list_item_clear_free(li: ListItemHandle) {
    let tv = unsafe { nvim_listitem_get_tv(li) };
    unsafe { tv_clear(tv) };
    unsafe { nvim_list_item_free(li) };
}

/// Format a dictitem as "key=value" for environment variable use.
/// Returns an xmalloc'd string; caller must free.
/// Exported as `nvim_dictitem_format_env` for existing `extern "C"` callers in this crate.
#[export_name = "nvim_dictitem_format_env"]
pub unsafe extern "C" fn rs_nvim_dictitem_format_env(di: DictItemHandle) -> *mut c_char {
    let key_ptr = unsafe { nvim_dictitem_get_key(di) };
    let tv = unsafe { nvim_dictitem_di_tv(di) };
    // Use nvim_tv_get_string which does type coercion (same as tv_get_string in C).
    let val_ptr = unsafe { nvim_tv_get_string(tv, std::ptr::null_mut()) };

    let key = std::ffi::CStr::from_ptr(key_ptr).to_bytes();
    let val = if val_ptr.is_null() {
        b"" as &[u8]
    } else {
        std::ffi::CStr::from_ptr(val_ptr).to_bytes()
    };

    // Allocate: key + '=' + val + NUL
    let total = key.len() + 1 + val.len() + 1;
    let buf = unsafe { nvim_xmalloc(total) }.cast::<u8>();
    let slice = std::slice::from_raw_parts_mut(buf, total);
    slice[..key.len()].copy_from_slice(key);
    slice[key.len()] = b'=';
    slice[key.len() + 1..key.len() + 1 + val.len()].copy_from_slice(val);
    slice[total - 1] = 0;
    buf.cast::<c_char>()
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

/// FFI export: tv_list_init_static10 - initialize a static list with 10 pre-allocated items.
///
/// Migrated from C. Zeros the `staticList10_T` struct, sets up the list header,
/// and chains the 10 `listitem_T` items in a doubly-linked list.
///
/// # Safety
///
/// `sl` must be a valid pointer to a `staticList10_T`.
#[export_name = "tv_list_init_static10"]
pub unsafe extern "C" fn rs_tv_list_init_static10(sl: *mut c_void) {
    // Zero the whole struct (CLEAR_POINTER)
    unsafe { nvim_staticlist10_clear(sl) };

    // Get pointers to the list and items array
    let l = unsafe { nvim_staticlist10_get_list(sl) };
    let do_not_free = unsafe { nvim_do_not_free_cnt() };

    // Set up list header
    let item0 = unsafe { nvim_staticlist10_get_item(sl, 0) };
    let item9 = unsafe { nvim_staticlist10_get_item(sl, 9) };
    unsafe { nvim_list_set_first(l, item0) };
    unsafe { nvim_list_set_last(l, item9) };
    unsafe { nvim_list_set_refcount(l, do_not_free) };
    unsafe { nvim_list_set_lock(l, 2) }; // VAR_FIXED = 2
    unsafe { nvim_list_set_len(l, 10) };

    // Pre-fetch all 10 item handles.
    let items: [ListItemHandle; 10] =
        core::array::from_fn(|i| unsafe { nvim_staticlist10_get_item(sl, i as c_int) });

    // Chain item 0: prev=NULL, next=item[1]
    unsafe { nvim_listitem_set_prev(items[0], ListItemHandle::null()) };
    unsafe { nvim_listitem_set_next(items[0], items[1]) };

    // Chain items 1..8: prev=item[i-1], next=item[i+1]
    for i in 1..9_usize {
        unsafe { nvim_listitem_set_prev(items[i], items[i - 1]) };
        unsafe { nvim_listitem_set_next(items[i], items[i + 1]) };
    }

    // Chain item 9: prev=item[8], next=NULL
    unsafe { nvim_listitem_set_prev(items[9], items[8]) };
    unsafe { nvim_listitem_set_next(items[9], ListItemHandle::null()) };
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
        unsafe { rs_tv_clear(li_tv) };
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
        unsafe { rs_tv_clear(li_tv) };
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
        unsafe { rs_tv_clear(rettv) };
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
            unsafe { rs_tv_clear(item_tv) };
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
    if unsafe { value_check_lock_impl(lock, arg_errmsg, TV_TRANSLATE) } {
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
            unsafe { typval_err_invrange() };
        } else {
            let ret_list = unsafe { rs_tv_list_alloc_ret(rettv, cnt as isize) };
            unsafe { rs_tv_list_move_items(l, item, item2, ret_list, cnt) };
        }
    }
}

// =============================================================================
// Phase 6d: tv_list_assign_range and sort/uniq
// =============================================================================

/// FFI export: tv_list_assign_range - assign src list values into dest range.
///
/// # Panics
///
/// Panics if the src list has items but dest range is exhausted (logic error in caller).
#[export_name = "tv_list_assign_range"]
pub unsafe extern "C" fn rs_tv_list_assign_range(
    dest: ListHandle,
    src: ListHandle,
    idx1_arg: c_int,
    idx2: c_int,
    empty_idx2: bool,
    op: *const c_char,
    varname: *const c_char,
) -> c_int {
    let mut idx1 = idx1_arg;
    let first_li = tv_list_find_index_impl(dest, &mut idx1);

    // Check whether any of the list items is locked before making any changes.
    let mut idx = idx1;
    let mut dest_li = first_li;
    let mut src_li = unsafe { nvim_list_get_first(src) };
    while !src_li.is_null() && !dest_li.is_null() {
        let v_lock = unsafe { nvim_listitem_get_v_lock(dest_li) };
        if unsafe { value_check_lock_impl(v_lock, varname, TV_CSTRING) } {
            return FAIL;
        }
        src_li = unsafe { nvim_listitem_get_next(src_li) };
        if src_li.is_null() || (!empty_idx2 && idx2 == idx) {
            break;
        }
        dest_li = unsafe { nvim_listitem_get_next(dest_li) };
        idx += 1;
    }

    // Assign the list values to the list items.
    idx = idx1;
    dest_li = first_li;
    src_li = unsafe { nvim_list_get_first(src) };
    while !src_li.is_null() {
        assert!(!dest_li.is_null());
        let dest_tv = unsafe { nvim_listitem_get_tv(dest_li) };
        let src_tv = unsafe { nvim_listitem_get_tv(src_li) };
        if !op.is_null() && unsafe { *op != b'=' as c_char } {
            unsafe { eexe_mod_op(dest_tv, src_tv, op) };
        } else {
            unsafe { rs_tv_clear(dest_tv) };
            unsafe { nvim_tv_copy(src_tv, dest_tv) };
        }
        src_li = unsafe { nvim_listitem_get_next(src_li) };
        if src_li.is_null() || (!empty_idx2 && idx2 == idx) {
            break;
        }
        if unsafe { nvim_listitem_get_next(dest_li) }.is_null() {
            // Need to add an empty item.
            unsafe { rs_tv_list_append_number(dest, 0) };
            dest_li = unsafe { nvim_list_get_last(dest) };
        } else {
            dest_li = unsafe { nvim_listitem_get_next(dest_li) };
        }
        idx += 1;
    }

    if !src_li.is_null() {
        unsafe { typval_err_list_more_items() };
        return FAIL;
    }
    if if empty_idx2 {
        !dest_li.is_null() && !unsafe { nvim_listitem_get_next(dest_li) }.is_null()
    } else {
        idx != idx2
    } {
        unsafe { typval_err_list_not_enough_items() };
        return FAIL;
    }
    OK
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
// Phase 6e: tv_list_copy, tv_list2items, tv_string2items, f_items
// =============================================================================

/// FFI export: tv_list_copy - deep or shallow copy of a list.
///
/// # Panics
///
/// Does not panic under normal usage.
#[export_name = "tv_list_copy"]
pub unsafe extern "C" fn rs_tv_list_copy(
    conv: *const std::ffi::c_void,
    orig: ListHandle,
    deep: bool,
    copy_id: c_int,
) -> ListHandle {
    if orig.is_null() {
        return ListHandle(std::ptr::null());
    }

    let copy = unsafe { rs_tv_list_alloc(tv_list_len_impl(orig) as isize) };
    unsafe { nvim_list_ref(copy) };

    if copy_id != 0 {
        // Record copyID before adding items, so back-references work.
        unsafe { nvim_list_set_copyid(orig, copy_id) };
        unsafe { nvim_list_set_copylist(orig, copy) };
    }

    let mut item = unsafe { nvim_list_get_first(orig) };
    while !item.is_null() {
        if unsafe { nvim_got_int() } != 0 {
            break;
        }
        let ni = unsafe { nvim_list_item_alloc() };
        let item_tv = unsafe { nvim_listitem_get_tv(item) };
        let ni_tv = unsafe { nvim_listitem_get_tv(ni) };
        if deep {
            if unsafe { var_item_copy(conv, item_tv, ni_tv, deep, copy_id) } == FAIL {
                unsafe { nvim_xfree(ni.0.cast_mut()) };
                // Unref/free the partial copy
                unsafe { rs_tv_list_unref(copy) };
                return ListHandle(std::ptr::null());
            }
        } else {
            unsafe { nvim_tv_copy(item_tv, ni_tv) };
        }
        unsafe { rs_tv_list_append(copy, ni) };
        item = unsafe { nvim_listitem_get_next(item) };
    }

    copy
}

/// Helper: "items(list)" implementation.
///
/// # Safety
///
/// argvars[0] must be a VAR_LIST typval.
unsafe fn tv_list2items_impl(argvars: TypevalHandle, rettv: TypevalHandle) {
    let arg0 = unsafe { nvim_typval_array_get(argvars, 0) };
    let l = unsafe { nvim_tv_get_list(arg0) };
    unsafe { nvim_tv_list_alloc_ret(rettv, tv_list_len_impl(l) as isize) };
    if l.is_null() {
        return; // null list behaves like an empty list
    }
    let ret_list = unsafe { nvim_tv_get_list(rettv) };

    let mut idx: i64 = 0;
    let mut li = unsafe { nvim_list_get_first(l) };
    while !li.is_null() {
        let l2 = unsafe { rs_tv_list_alloc(2) };
        unsafe { rs_tv_list_append_list(ret_list, l2) };
        unsafe { rs_tv_list_append_number(l2, idx) };
        let item_tv = unsafe { nvim_listitem_get_tv(li) };
        unsafe { rs_tv_list_append_tv(l2, item_tv) };
        idx += 1;
        li = unsafe { nvim_listitem_get_next(li) };
    }
}

/// Helper: "items(string)" implementation.
///
/// # Safety
///
/// argvars[0] must be a VAR_STRING typval.
unsafe fn tv_string2items_impl(argvars: TypevalHandle, rettv: TypevalHandle) {
    let arg0 = unsafe { nvim_typval_array_get(argvars, 0) };
    let p_start = unsafe { nvim_tv_get_string_ptr(arg0) };
    unsafe {
        nvim_tv_list_alloc_ret(rettv, -3 /* kListLenMayKnow */)
    };
    if p_start.is_null() {
        return; // null string behaves like empty string
    }
    let ret_list = unsafe { nvim_tv_get_list(rettv) };

    let mut p = p_start;
    let mut idx: i64 = 0;
    loop {
        // Check for NUL terminator
        if unsafe { *p } == 0 {
            break;
        }
        let len = unsafe { utfc_ptr2len(p) };
        if len == 0 {
            break;
        }
        let l2 = unsafe { rs_tv_list_alloc(2) };
        unsafe { rs_tv_list_append_list(ret_list, l2) };
        unsafe { rs_tv_list_append_number(l2, idx) };
        unsafe { rs_tv_list_append_string(l2, p, len as isize) };
        p = unsafe { p.add(len as usize) };
        idx += 1;
    }
}

/// FFI export: f_items - VimL items() function.
#[export_name = "f_items"]
pub unsafe extern "C" fn rs_f_items(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    _fptr: *const std::ffi::c_void,
) {
    let arg0 = unsafe { nvim_typval_array_get(argvars, 0) };
    let v_type = tv_type_impl(arg0);
    match v_type {
        VarType::String => unsafe { tv_string2items_impl(argvars, rettv) },
        VarType::List => unsafe { tv_list2items_impl(argvars, rettv) },
        _ => unsafe { tv_dict2list_impl(argvars, rettv, 2) },
    }
}

// =============================================================================
// Phase 6f: tv_dict_remove
// =============================================================================

/// FFI export: tv_dict_remove - VimL remove() for dicts.
#[export_name = "tv_dict_remove"]
pub unsafe extern "C" fn rs_tv_dict_remove(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    arg_errmsg: *const c_char,
) {
    let arg2 = unsafe { nvim_typval_array_get(argvars, 2) };
    if tv_type_impl(arg2) != VarType::Unknown {
        let remove_str = b"remove()\0";
        unsafe { nvim_semsg_toomanyarg(remove_str.as_ptr().cast::<c_char>()) };
        return;
    }
    let arg0 = unsafe { nvim_typval_array_get(argvars, 0) };
    let d = unsafe { nvim_tv_get_dict(arg0) };
    if d.is_null() {
        return;
    }
    let dv_lock = unsafe { nvim_dict_get_lock(d) };
    if unsafe { value_check_lock_impl(dv_lock, arg_errmsg, TV_TRANSLATE) } {
        return;
    }
    let arg1 = unsafe { nvim_typval_array_get(argvars, 1) };
    let key = unsafe { rs_tv_get_string_chk(arg1) };
    if key.is_null() {
        return;
    }
    let di = unsafe { nvim_dict_find(d, key, -1) };
    if di.is_null() {
        unsafe { nvim_semsg_dictkey(key) };
        return;
    }
    // nvim_di_check_fixed_translate inlined: di_flags at offset 16, TV_TRANSLATE = SIZE_MAX
    // nvim_di_check_ro_translate inlined: di_flags at offset 16, TV_TRANSLATE = SIZE_MAX
    let di_flags = c_int::from(unsafe { *di.0.cast::<u8>().add(16) });
    if unsafe { var_check_fixed(di_flags, arg_errmsg, usize::MAX) }
        || unsafe { var_check_ro(di_flags, arg_errmsg, usize::MAX) }
    {
        return;
    }
    // nvim_dictitem_move_tv_to_rettv inlined: di_tv at offset 0, copy 16 bytes then zero
    unsafe {
        std::ptr::copy_nonoverlapping(di.0.cast::<u8>(), rettv.0.cast_mut().cast::<u8>(), 16);
        std::ptr::write_bytes(di.0.cast_mut().cast::<u8>(), 0, 16);
    }
    unsafe { rs_tv_dict_item_remove(d, di) };
    if tv_dict_is_watched_impl(d) {
        unsafe {
            rs_tv_dict_watcher_notify(
                d,
                key,
                TypevalHandle::from_ptr(std::ptr::null()),
                TypevalHandle::from_ptr(rettv.0.cast_mut()),
            );
        }
    }
}

// =============================================================================
// Phase 6g / Phase 8: f_keys, f_values, and tv_dict2list in Rust
// =============================================================================

/// Enumerate dict entries into a list (replaces C tv_dict2list, Phase 8).
///
/// mode: 0 = keys, 1 = values, 2 = items ([key, value] pairs)
///
/// # Safety
/// argvars and rettv must be valid non-null pointers to typval_T arrays.
unsafe fn tv_dict2list_impl(argvars: TypevalHandle, rettv: TypevalHandle, mode: u8) {
    let check_result = if mode == 2 {
        tv_check_for_string_or_list_or_dict_arg_impl(argvars, 0)
    } else {
        tv_check_for_dict_arg_impl(argvars, 0)
    };
    if check_result == FAIL {
        unsafe { nvim_tv_list_alloc_ret(rettv, 0) };
        return;
    }
    let arg0 = unsafe { nvim_typval_array_get(argvars, 0) };
    let d = unsafe { nvim_tv_get_dict(arg0) };
    let ht_used: usize = if d.is_null() {
        0
    } else {
        unsafe { nvim_dict_get_ht_used(d) }
    };
    let ret_list = unsafe { nvim_tv_list_alloc_ret(rettv, ht_used as isize) };
    if d.is_null() || ht_used == 0 {
        return;
    }
    let mut hi = unsafe { nvim_dict_get_ht_array(d) };
    let mut seen = 0usize;
    while seen < ht_used {
        let key = unsafe { nvim_hashitem_get_key(hi) };
        // Empty slots have null key; deleted slots have key starting with '@'.
        if !key.is_null() && unsafe { *key } != b'@' as c_char {
            seen += 1;
            let di = unsafe { nvim_hashitem_to_dictitem(hi) };
            match mode {
                0 => {
                    // keys: append key string
                    unsafe { rs_tv_list_append_string(ret_list, key, -1) };
                }
                1 => {
                    // values: append copy of dict value
                    let di_tv = unsafe { nvim_dictitem_get_tv(di) };
                    unsafe { rs_tv_list_append_tv(ret_list, di_tv) };
                }
                _ => {
                    // items: append [key, value] two-element sub-list
                    let sub_l = unsafe { nvim_list_alloc_impl() };
                    unsafe { rs_tv_list_append_string(sub_l, key, -1) };
                    let di_tv = unsafe { nvim_dictitem_get_tv(di) };
                    unsafe { rs_tv_list_append_tv(sub_l, di_tv) };
                    // rs_tv_list_append_list increments sub_l refcount from 0 to 1
                    unsafe { rs_tv_list_append_list(ret_list, sub_l) };
                }
            }
        }
        hi = unsafe { nvim_hashitem_next(hi) };
    }
}

/// FFI export: f_keys - VimL keys() function.
#[export_name = "f_keys"]
pub unsafe extern "C" fn rs_f_keys(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    _fptr: *const std::ffi::c_void,
) {
    unsafe { tv_dict2list_impl(argvars, rettv, 0) };
}

/// FFI export: f_values - VimL values() function.
#[export_name = "f_values"]
pub unsafe extern "C" fn rs_f_values(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    _fptr: *const std::ffi::c_void,
) {
    unsafe { tv_dict2list_impl(argvars, rettv, 1) };
}

// =============================================================================
// Phase 6i: tv_dict_get_string
// =============================================================================

/// FFI export: tv_dict_get_string - get string from dict, optionally duplicated.
#[export_name = "tv_dict_get_string"]
pub unsafe extern "C" fn rs_tv_dict_get_string(
    d: DictHandle,
    key: *const c_char,
    save: bool,
) -> *mut c_char {
    use std::cell::UnsafeCell;
    struct StaticBuf(UnsafeCell<[u8; NUMBUFLEN]>);
    unsafe impl Sync for StaticBuf {}
    static NUMBUF: StaticBuf = StaticBuf(UnsafeCell::new([0u8; NUMBUFLEN]));
    let buf = unsafe { (*NUMBUF.0.get()).as_mut_ptr().cast::<c_char>() };
    let s = unsafe { rs_tv_dict_get_string_buf(d, key, buf) };
    if save && !s.is_null() {
        unsafe { nvim_xstrdup(s) }
    } else {
        s.cast_mut()
    }
}

// =============================================================================
// Phase 6h: tv_dict_set_keys_readonly (using hashtab iteration)
// =============================================================================

/// FFI export: tv_dict_set_keys_readonly - set all keys read-only and fixed.
#[export_name = "tv_dict_set_keys_readonly"]
pub unsafe extern "C" fn rs_tv_dict_set_keys_readonly(dict: DictHandle) {
    let ht_used = unsafe { nvim_dict_get_ht_used(dict) };
    if ht_used == 0 {
        return;
    }
    let hi_removed = unsafe { nvim_hash_removed_ptr() };
    let mut hi = unsafe { nvim_dict_get_ht_array(dict) };
    let mut todo = ht_used;
    // Replicate HASHTAB_ITER: iterate ht_array, skip empty/removed items.
    loop {
        if todo == 0 {
            break;
        }
        let key = unsafe { nvim_hashitem_get_key(hi) };
        if !key.is_null() && key != hi_removed {
            // Item is live - set RO+FIX flags via C accessor (TV_DICT_HI2DI + |=)
            unsafe { nvim_hashitem_set_ro_fix(hi) };
            todo -= 1;
        }
        hi = unsafe { nvim_hashitem_next(hi) };
    }
}

// =============================================================================
// Phase 6j: tv_dict_to_env
// =============================================================================

/// FFI export: tv_dict_to_env - convert dict to a NULL-terminated env array.
///
/// Each entry is "KEY=VALUE" as a freshly allocated C string.
/// The returned array is also heap-allocated and must be freed by the caller.
/// Returns NULL if the dict is NULL.
///
/// # Safety
/// `denv` must be a valid DictHandle or null.
#[export_name = "tv_dict_to_env"]
pub unsafe extern "C" fn rs_tv_dict_to_env(denv: DictHandle) -> *mut *mut c_char {
    if denv.is_null() {
        return std::ptr::null_mut();
    }
    let env_size = unsafe { nvim_dict_get_ht_used(denv) } as usize;
    let alloc_size = (env_size + 1)
        .checked_mul(std::mem::size_of::<*mut c_char>())
        .unwrap_or(0);
    let env = unsafe { nvim_xmalloc(alloc_size) }.cast::<*mut c_char>();
    if env.is_null() {
        return std::ptr::null_mut();
    }
    let hi_removed = unsafe { nvim_hash_removed_ptr() };
    let mut hi = unsafe { nvim_dict_get_ht_array(denv) };
    let mut todo = env_size;
    let mut i = 0usize;
    loop {
        if todo == 0 {
            break;
        }
        let key = unsafe { nvim_hashitem_get_key(hi) };
        if !key.is_null() && key != hi_removed {
            let di = unsafe { nvim_hashitem_to_dictitem(hi) };
            let entry = unsafe { nvim_dictitem_format_env(di) };
            unsafe { *env.add(i) = entry };
            i += 1;
            todo -= 1;
        }
        hi = unsafe { nvim_hashitem_next(hi) };
    }
    unsafe { *env.add(env_size) = std::ptr::null_mut() };
    env
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
        unsafe { typval_err_blobidx(idx) };
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
            unsafe { typval_err_blobidx(n1) };
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
            unsafe { typval_err_blobidx(n2) };
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
        unsafe { typval_err_blob_wrong_bytes() };
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
        if unsafe { value_check_lock_impl(lock, arg_errmsg, TV_TRANSLATE) } {
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
            unsafe { typval_err_blobidx(idx) };
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
                unsafe { typval_err_blobidx(end) };
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
            unsafe { typval_err_float_funcref() };
            0.0
        }
        VarType::String => {
            unsafe { typval_err_float_string() };
            0.0
        }
        VarType::List => {
            unsafe { typval_err_float_list() };
            0.0
        }
        VarType::Dict => {
            unsafe { typval_err_float_dict() };
            0.0
        }
        VarType::Bool => {
            unsafe { typval_err_float_bool() };
            0.0
        }
        VarType::Special => {
            unsafe { typval_err_float_special() };
            0.0
        }
        VarType::Blob => {
            unsafe { typval_err_float_blob() };
            0.0
        }
        VarType::Unknown => {
            unsafe { typval_err_float_unknown() };
            0.0
        }
    }
}

/// FFI wrapper: get float from typval.
#[export_name = "tv_get_float"]
pub extern "C" fn rs_tv_get_float(tv: TypevalHandle) -> f64 {
    tv_get_float_impl(tv)
}

/// TV_TRANSLATE sentinel: name should be passed through gettext().
const TV_TRANSLATE: usize = usize::MAX;
/// TV_CSTRING sentinel: name_len should be computed via strlen().
const TV_CSTRING: usize = usize::MAX - 1;

/// Check if variable "name" has a locked (immutable) value.
///
/// Native Rust implementation (Phase 3): no longer delegates to C.
///
/// # Safety
///
/// `name` must be a valid C string pointer, or null.
unsafe fn value_check_lock_impl(lock: c_int, name: *const c_char, name_len: usize) -> bool {
    // VAR_UNLOCKED = 0, VAR_LOCKED = 1, VAR_FIXED = 2
    if lock == 0 {
        return false;
    }
    let fmt = if lock == 1 {
        nvim_gettext_value_locked()
    } else {
        nvim_gettext_value_fixed()
    };
    // Resolve name and name_len
    let (resolved_name, resolved_len) = if name.is_null() {
        let unknown = nvim_gettext_unknown();
        (unknown, libc_strlen(unknown))
    } else if name_len == TV_TRANSLATE {
        // Need to apply gettext
        extern "C" {
            #[link_name = "gettext"]
            fn do_gettext(msgid: *const c_char) -> *const c_char;
        }
        let translated = do_gettext(name);
        (translated, libc_strlen(translated))
    } else if name_len == TV_CSTRING {
        (name, libc_strlen(name))
    } else {
        (name, name_len)
    };
    semsg_typval(fmt, resolved_len as c_int, resolved_name);
    true
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
    if unsafe { value_check_lock_impl(v_lock, name, name_len) } {
        return true;
    }
    container_lock != 0 && unsafe { value_check_lock_impl(container_lock, name, name_len) }
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

// Direct emsg/semsg/gettext bindings used throughout this module.
extern "C" {
    /// Get a pointer to args[idx] in a typval array.
    /// This avoids needing to know sizeof(typval_T) in Rust.
    fn nvim_typval_array_get(args: TypevalHandle, idx: c_int) -> TypevalHandle;

    // Direct error dispatch (Phase 1: replaces ~50 nvim_typval_error_* / nvim_emsg_* wrappers)
    fn emsg(s: *const c_char);
    // gettext for translated error strings
    fn gettext(msgid: *const c_char) -> *const c_char;

    // Translated format strings for argument-index errors (static-local in typval.c;
    // must go through _() in C so these tiny C accessors are kept).
    fn nvim_gettext_e_string_required_for_argument_nr() -> *const c_char;
    fn nvim_gettext_e_nonempty_string_required_for_argument_nr() -> *const c_char;
    fn nvim_gettext_e_number_required_for_argument_nr() -> *const c_char;
    fn nvim_gettext_e_float_or_number_required_for_argument_nr() -> *const c_char;
    fn nvim_gettext_e_bool_required_for_argument_nr() -> *const c_char;
    fn nvim_gettext_e_blob_required_for_argument_nr() -> *const c_char;
    fn nvim_gettext_e_list_required_for_argument_nr() -> *const c_char;
    fn nvim_gettext_e_dict_required_for_argument_nr() -> *const c_char;
    fn nvim_gettext_e_nonnull_dict_required_for_argument_nr() -> *const c_char;
    fn nvim_gettext_e_string_or_number_required_for_argument_nr() -> *const c_char;
    fn nvim_gettext_e_string_or_list_required_for_argument_nr() -> *const c_char;
    fn nvim_gettext_e_string_list_or_blob_required_for_argument_nr() -> *const c_char;
    fn nvim_gettext_e_string_list_or_dict_required_for_argument_nr() -> *const c_char;
    fn nvim_gettext_e_string_or_func_required_for_argument_nr() -> *const c_char;
    fn nvim_gettext_e_list_or_blob_required_for_argument_nr() -> *const c_char;
    // Static-local translated string: e_using_invalid_value_as_string
    fn nvim_gettext_e_using_invalid_value_as_string() -> *const c_char;
    // Static-local translated string: e_variable_nested_too_deep_for_unlock
    fn nvim_gettext_e_variable_nested_too_deep_for_unlock() -> *const c_char;
    // Static-local: e_invalid_value_for_blob_nr (has %d format specifier)
    fn nvim_gettext_e_invalid_value_for_blob_nr() -> *const c_char;
    // Global error strings (from errors.h; translated via gettext at call site)
    #[link_name = "e_intern2"]
    static e_intern2_tv: [u8; 0];
    #[link_name = "e_blobidx"]
    static e_blobidx_tv: [u8; 0];
    #[link_name = "e_invarg"]
    static e_invarg_tv: [u8; 0];
    #[link_name = "e_listreq"]
    static e_listreq_tv: [u8; 0];
    #[link_name = "e_listarg"]
    static e_listarg_tv: [u8; 0];
    #[link_name = "e_invrange"]
    static e_invrange_tv: [u8; 0];
}

// Inline helper: translate a C string via gettext.
#[inline]
unsafe fn gt(s: *const c_char) -> *const c_char {
    gettext(s)
}

// Inline error helpers replacing the ~50 deleted C wrapper functions.
// Argument-index errors (format: semsg(fmt, idx)).
// These use the `semsg_typval` variadic binding that already exists.
#[inline]
unsafe fn typval_err_string_required(idx: c_int) {
    semsg_typval(gt(nvim_gettext_e_string_required_for_argument_nr()), idx);
}
#[inline]
unsafe fn typval_err_nonempty_string_required(idx: c_int) {
    semsg_typval(
        gt(nvim_gettext_e_nonempty_string_required_for_argument_nr()),
        idx,
    );
}
#[inline]
unsafe fn typval_err_number_required(idx: c_int) {
    semsg_typval(gt(nvim_gettext_e_number_required_for_argument_nr()), idx);
}
#[inline]
unsafe fn typval_err_float_or_number_required(idx: c_int) {
    semsg_typval(
        gt(nvim_gettext_e_float_or_number_required_for_argument_nr()),
        idx,
    );
}
#[inline]
unsafe fn typval_err_bool_required(idx: c_int) {
    semsg_typval(gt(nvim_gettext_e_bool_required_for_argument_nr()), idx);
}
#[inline]
unsafe fn typval_err_blob_required(idx: c_int) {
    semsg_typval(gt(nvim_gettext_e_blob_required_for_argument_nr()), idx);
}
#[inline]
unsafe fn typval_err_list_required(idx: c_int) {
    semsg_typval(gt(nvim_gettext_e_list_required_for_argument_nr()), idx);
}
#[inline]
unsafe fn typval_err_dict_required(idx: c_int) {
    semsg_typval(gt(nvim_gettext_e_dict_required_for_argument_nr()), idx);
}
#[inline]
unsafe fn typval_err_nonnull_dict_required(idx: c_int) {
    semsg_typval(
        gt(nvim_gettext_e_nonnull_dict_required_for_argument_nr()),
        idx,
    );
}
#[inline]
unsafe fn typval_err_string_or_number_required(idx: c_int) {
    semsg_typval(
        gt(nvim_gettext_e_string_or_number_required_for_argument_nr()),
        idx,
    );
}
#[inline]
unsafe fn typval_err_string_or_list_required(idx: c_int) {
    semsg_typval(
        gt(nvim_gettext_e_string_or_list_required_for_argument_nr()),
        idx,
    );
}
#[inline]
unsafe fn typval_err_string_list_or_blob_required(idx: c_int) {
    semsg_typval(
        gt(nvim_gettext_e_string_list_or_blob_required_for_argument_nr()),
        idx,
    );
}
#[inline]
unsafe fn typval_err_string_list_or_dict_required(idx: c_int) {
    semsg_typval(
        gt(nvim_gettext_e_string_list_or_dict_required_for_argument_nr()),
        idx,
    );
}
#[inline]
unsafe fn typval_err_string_or_func_required(idx: c_int) {
    semsg_typval(
        gt(nvim_gettext_e_string_or_func_required_for_argument_nr()),
        idx,
    );
}
#[inline]
unsafe fn typval_err_list_or_blob_required(idx: c_int) {
    semsg_typval(
        gt(nvim_gettext_e_list_or_blob_required_for_argument_nr()),
        idx,
    );
}
// Type-mismatch errors (no index argument)
#[inline]
unsafe fn typval_err_funcref_as_number() {
    emsg(gt(c"E703: Using a Funcref as a Number".as_ptr()));
}
#[inline]
unsafe fn typval_err_list_as_number() {
    emsg(gt(c"E745: Using a List as a Number".as_ptr()));
}
#[inline]
unsafe fn typval_err_dict_as_number() {
    emsg(gt(c"E728: Using a Dictionary as a Number".as_ptr()));
}
#[inline]
unsafe fn typval_err_float_as_number() {
    emsg(gt(c"E805: Using a Float as a Number".as_ptr()));
}
#[inline]
unsafe fn typval_err_blob_as_number() {
    emsg(gt(c"E974: Using a Blob as a Number".as_ptr()));
}
#[inline]
unsafe fn typval_err_invalid_as_number() {
    emsg(gt(c"E685: using an invalid value as a Number".as_ptr()));
}
#[inline]
unsafe fn typval_err_funcref_as_string() {
    emsg(gt(c"E729: Using a Funcref as a String".as_ptr()));
}
#[inline]
unsafe fn typval_err_list_as_string() {
    emsg(gt(c"E730: Using a List as a String".as_ptr()));
}
#[inline]
unsafe fn typval_err_dict_as_string() {
    emsg(gt(c"E731: Using a Dictionary as a String".as_ptr()));
}
#[inline]
unsafe fn typval_err_blob_as_string() {
    emsg(gt(c"E976: Using a Blob as a String".as_ptr()));
}
#[inline]
unsafe fn typval_err_invalid_as_string() {
    emsg(gt(nvim_gettext_e_using_invalid_value_as_string()));
}
// str_or_nr errors
#[inline]
unsafe fn typval_err_str_or_nr_float() {
    emsg(gt(
        c"E805: Expected a Number or a String, Float found".as_ptr()
    ));
}
#[inline]
unsafe fn typval_err_str_or_nr_funcref() {
    emsg(gt(
        c"E703: Expected a Number or a String, Funcref found".as_ptr()
    ));
}
#[inline]
unsafe fn typval_err_str_or_nr_list() {
    emsg(gt(
        c"E745: Expected a Number or a String, List found".as_ptr()
    ));
}
#[inline]
unsafe fn typval_err_str_or_nr_dict() {
    emsg(gt(
        c"E728: Expected a Number or a String, Dictionary found".as_ptr()
    ));
}
#[inline]
unsafe fn typval_err_str_or_nr_blob() {
    emsg(gt(
        c"E974: Expected a Number or a String, Blob found".as_ptr()
    ));
}
#[inline]
unsafe fn typval_err_str_or_nr_bool() {
    emsg(gt(
        c"E5299: Expected a Number or a String, Boolean found".as_ptr()
    ));
}
#[inline]
unsafe fn typval_err_str_or_nr_special() {
    emsg(gt(c"E5300: Expected a Number or a String".as_ptr()));
}
#[inline]
unsafe fn typval_err_str_or_nr_unknown() {
    semsg_typval(
        gt(e_intern2_tv.as_ptr().cast()),
        c"tv_check_str_or_nr(UNKNOWN)".as_ptr(),
    );
}
// Float errors
#[inline]
unsafe fn typval_err_float_funcref() {
    emsg(gt(c"E891: Using a Funcref as a Float".as_ptr()));
}
#[inline]
unsafe fn typval_err_float_string() {
    emsg(gt(c"E892: Using a String as a Float".as_ptr()));
}
#[inline]
unsafe fn typval_err_float_list() {
    emsg(gt(c"E893: Using a List as a Float".as_ptr()));
}
#[inline]
unsafe fn typval_err_float_dict() {
    emsg(gt(c"E894: Using a Dictionary as a Float".as_ptr()));
}
#[inline]
unsafe fn typval_err_float_bool() {
    emsg(gt(c"E362: Using a boolean value as a Float".as_ptr()));
}
#[inline]
unsafe fn typval_err_float_special() {
    emsg(gt(c"E907: Using a special value as a Float".as_ptr()));
}
#[inline]
unsafe fn typval_err_float_blob() {
    emsg(gt(c"E975: Using a Blob as a Float".as_ptr()));
}
#[inline]
unsafe fn typval_err_float_unknown() {
    semsg_typval(
        gt(e_intern2_tv.as_ptr().cast()),
        c"tv_get_float(UNKNOWN)".as_ptr(),
    );
}
// Blob / list / sort / misc errors
#[inline]
unsafe fn typval_err_blob_wrong_bytes() {
    emsg(gt(
        c"E972: Blob value does not have the right number of bytes".as_ptr(),
    ));
}
#[inline]
unsafe fn typval_err_item_lock_nested() {
    emsg(gt(nvim_gettext_e_variable_nested_too_deep_for_unlock()));
}
#[inline]
unsafe fn typval_err_invrange() {
    emsg(gt(e_invrange_tv.as_ptr().cast()));
}
#[inline]
unsafe fn typval_err_list_more_items() {
    emsg(gt(c"E710: List value has more items than target".as_ptr()));
}
#[inline]
unsafe fn typval_err_list_not_enough_items() {
    emsg(gt(c"E711: List value has not enough items".as_ptr()));
}
#[inline]
unsafe fn typval_err_sort_failed() {
    emsg(gt(c"E702: Sort compare function failed".as_ptr()));
}
#[inline]
unsafe fn typval_err_uniq_failed() {
    emsg(gt(c"E882: Uniq compare function failed".as_ptr()));
}
#[inline]
unsafe fn typval_err_listarg(fname: *const c_char) {
    semsg_typval(gt(e_listarg_tv.as_ptr().cast()), fname);
}
#[inline]
unsafe fn typval_err_invarg() {
    emsg(gt(e_invarg_tv.as_ptr().cast()));
}
#[inline]
unsafe fn typval_err_e_listreq() {
    emsg(gt(e_listreq_tv.as_ptr().cast()));
}
#[inline]
unsafe fn typval_err_not_func_or_funcname() {
    emsg(gt(
        c"E6000: Argument is not a function or function name".as_ptr()
    ));
}
#[inline]
unsafe fn typval_err_get_number_unknown() {
    semsg_typval(
        gt(e_intern2_tv.as_ptr().cast()),
        c"tv_get_number(UNKNOWN)".as_ptr(),
    );
}
#[inline]
unsafe fn typval_err_blobidx(idx: i64) {
    semsg_typval(gt(e_blobidx_tv.as_ptr().cast()), idx);
}
#[inline]
unsafe fn typval_err_blob_invalid_value(n: i64) {
    semsg_typval(gt(nvim_gettext_e_invalid_value_for_blob_nr()), n as c_int);
}
#[inline]
unsafe fn typval_err_key_exists(key: *const c_char) {
    semsg_typval(gt(c"E737: Key already exists: %s".as_ptr()), key);
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
        unsafe { typval_err_string_required(idx + 1) };
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
        unsafe { typval_err_nonempty_string_required(idx + 1) };
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
        unsafe { typval_err_number_required(idx + 1) };
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
        unsafe { typval_err_float_or_number_required(idx + 1) };
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
    unsafe { typval_err_bool_required(idx + 1) };
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
        unsafe { typval_err_blob_required(idx + 1) };
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
        unsafe { typval_err_list_required(idx + 1) };
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
        unsafe { typval_err_dict_required(idx + 1) };
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
        unsafe { typval_err_nonnull_dict_required(idx + 1) };
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
        unsafe { typval_err_string_or_number_required(idx + 1) };
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
        unsafe { typval_err_string_or_list_required(idx + 1) };
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
        unsafe { typval_err_string_list_or_blob_required(idx + 1) };
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
        unsafe { typval_err_string_list_or_dict_required(idx + 1) };
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
        unsafe { typval_err_string_or_func_required(idx + 1) };
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
        unsafe { typval_err_list_or_blob_required(idx + 1) };
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
            unsafe { typval_err_funcref_as_number() };
            false
        }
        VarType::List => {
            unsafe { typval_err_list_as_number() };
            false
        }
        VarType::Dict => {
            unsafe { typval_err_dict_as_number() };
            false
        }
        VarType::Float => {
            unsafe { typval_err_float_as_number() };
            false
        }
        VarType::Blob => {
            unsafe { typval_err_blob_as_number() };
            false
        }
        VarType::Unknown => {
            unsafe { typval_err_invalid_as_number() };
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
            unsafe { typval_err_funcref_as_string() };
            false
        }
        VarType::List => {
            unsafe { typval_err_list_as_string() };
            false
        }
        VarType::Dict => {
            unsafe { typval_err_dict_as_string() };
            false
        }
        VarType::Blob => {
            unsafe { typval_err_blob_as_string() };
            false
        }
        VarType::Unknown => {
            unsafe { typval_err_invalid_as_string() };
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
            unsafe { typval_err_str_or_nr_float() };
            false
        }
        VarType::Func | VarType::Partial => {
            unsafe { typval_err_str_or_nr_funcref() };
            false
        }
        VarType::List => {
            unsafe { typval_err_str_or_nr_list() };
            false
        }
        VarType::Dict => {
            unsafe { typval_err_str_or_nr_dict() };
            false
        }
        VarType::Blob => {
            unsafe { typval_err_str_or_nr_blob() };
            false
        }
        VarType::Bool => {
            unsafe { typval_err_str_or_nr_bool() };
            false
        }
        VarType::Special => {
            unsafe { typval_err_str_or_nr_special() };
            false
        }
        VarType::Unknown => {
            unsafe { typval_err_str_or_nr_unknown() };
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
                unsafe { typval_err_blob_invalid_value(n) };
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
            unsafe { typval_err_funcref_as_number() };
        }
        VarType::List => {
            unsafe { typval_err_list_as_number() };
        }
        VarType::Dict => {
            unsafe { typval_err_dict_as_number() };
        }
        VarType::Float => {
            unsafe { typval_err_float_as_number() };
        }
        VarType::Blob => {
            unsafe { typval_err_blob_as_number() };
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
            unsafe { typval_err_get_number_unknown() };
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
            unsafe { typval_err_funcref_as_string() };
            std::ptr::null()
        }
        VarType::List => {
            unsafe { typval_err_list_as_string() };
            std::ptr::null()
        }
        VarType::Dict => {
            unsafe { typval_err_dict_as_string() };
            std::ptr::null()
        }
        VarType::Blob => {
            unsafe { typval_err_blob_as_string() };
            std::ptr::null()
        }
        VarType::Unknown => {
            unsafe { typval_err_invalid_as_string() };
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
    let did_emsg_before = unsafe { did_emsg };
    let lnum = tv_get_number_chk_impl(tv, std::ptr::null_mut()) as i32;
    if lnum <= 0 && unsafe { did_emsg } == did_emsg_before && tv_type_impl(tv) != VarType::Number {
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

// =============================================================================
// Phase 1: Callback Operations (migrated from typval.c)
// =============================================================================
//
// The Callback struct layout (16 bytes):
//   offset 0: data union (8 bytes) - funcref (*mut c_char) / partial (*mut c_void) / luaref (c_int)
//   offset 8: type (c_int, CallbackType enum)
//   offset 12: 4 bytes padding
//
// CallbackType enum values:
//   kCallbackNone    = 0
//   kCallbackFuncref = 1
//   kCallbackPartial = 2
//   kCallbackLua     = 3

// c_void already imported at top

const K_CALLBACK_NONE: c_int = 0;
const K_CALLBACK_FUNCREF: c_int = 1;
const K_CALLBACK_PARTIAL: c_int = 2;
const K_CALLBACK_LUA: c_int = 3;

/// Callback struct mirror (16 bytes, layout verified by _Static_assert in eval_shim.c).
#[repr(C)]
struct CbData {
    /// Data union: funcref (*mut c_char), partial (*mut c_void), or luaref (c_int).
    data: CbUnion,
    /// Type discriminant (CallbackType enum value).
    cb_type: c_int,
    _pad: [u8; 4],
}

/// Union for callback data field (8 bytes).
#[repr(C)]
union CbUnion {
    funcref: *mut c_char,
    partial: *mut c_void,
    luaref: c_int,
}

extern "C" {
    // Callback Lua accessors (defined in typval.c Phase 1 section)
    fn nvim_callback_clear_luaref(cb: *mut c_void);
    fn nvim_callback_new_luaref(ref_: c_int) -> c_int;
    fn nvim_callback_funcref_str(luaref: c_int, arena: *mut c_void) -> *mut c_char;

    // Partial accessors
    fn nvim_partial_inc_refcount(pt: *mut c_void);
    fn nvim_partial_dec_refcount(pt: *mut c_void);
    fn nvim_partial_get_refcount(pt: *const c_void) -> c_int;
    fn nvim_partial_get_name(pt: *mut c_void) -> *mut c_char;

    // Memory and string functions
    #[link_name = "xmallocz"]
    fn cb_xmallocz(size: usize) -> *mut c_char;
    fn func_unref(name: *mut c_char);
    fn func_ref(name: *const c_char);
    // partial_unref is exported by the eval crate
    fn partial_unref(pt: *mut c_void);
    fn snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

// Additional setters for callback_put and tv_clear
extern "C" {
    fn nvim_tv_set_partial(tv: *mut c_void, pt: *mut c_void);
    fn nvim_tv_set_vstring_owned(tv: *mut c_void, s: *mut c_char);
    fn nvim_tv_set_special(tv: *mut c_void, val: c_int);
    fn nvim_tv_set_vpartial_null(tv: TypevalHandle);
    fn nvim_tv_set_vblob_null(tv: TypevalHandle);
    fn nvim_tv_set_vlist_null(tv: TypevalHandle);
    fn nvim_tv_set_vdict_null(tv: TypevalHandle);
    // tv_empty_string: address of the global empty-string sentinel
    #[link_name = "tv_empty_string"]
    static tv_empty_string_ptr: *const c_char;
}

/// Compare two Callback structs for equality.
///
/// Migrated from C `tv_callback_equal`.
///
/// # Safety
///
/// Both `cb1` and `cb2` must be valid non-null pointers to `Callback` structs.
#[export_name = "tv_callback_equal"]
pub unsafe extern "C" fn rs_tv_callback_equal(cb1: *const c_void, cb2: *const c_void) -> bool {
    let cb1 = &*cb1.cast::<CbData>();
    let cb2 = &*cb2.cast::<CbData>();

    if cb1.cb_type != cb2.cb_type {
        return false;
    }
    match cb1.cb_type {
        K_CALLBACK_FUNCREF => {
            // Compare funcref strings
            let s1 = cb1.data.funcref;
            let s2 = cb2.data.funcref;
            if s1.is_null() && s2.is_null() {
                return true;
            }
            if s1.is_null() || s2.is_null() {
                return false;
            }
            strcmp(s1, s2) == 0
        }
        K_CALLBACK_PARTIAL => {
            // Compare partial pointers (pointer identity, not deep equality)
            cb1.data.partial == cb2.data.partial
        }
        K_CALLBACK_LUA => {
            // Compare luarefs
            cb1.data.luaref == cb2.data.luaref
        }
        K_CALLBACK_NONE => true,
        _ => false,
    }
}

/// Free callback resources and reset to kCallbackNone.
///
/// Migrated from C `callback_free`.
///
/// # Safety
///
/// `callback` must be a valid non-null pointer to a `Callback` struct.
#[export_name = "callback_free"]
pub unsafe extern "C" fn rs_callback_free(callback: *mut c_void) {
    let cb = &mut *callback.cast::<CbData>();
    match cb.cb_type {
        K_CALLBACK_FUNCREF => {
            func_unref(cb.data.funcref);
            nvim_xfree(cb.data.funcref.cast());
        }
        K_CALLBACK_PARTIAL => {
            partial_unref(cb.data.partial);
        }
        K_CALLBACK_LUA => {
            nvim_callback_clear_luaref(callback);
            return; // nvim_callback_clear_luaref already resets type and data
        }
        _ => {}
    }
    cb.cb_type = K_CALLBACK_NONE;
    cb.data.funcref = std::ptr::null_mut();
}

/// Copy a Callback into a typval_T.
///
/// Migrated from C `callback_put`.
///
/// # Safety
///
/// `cb` must be a valid non-null pointer to a `Callback` struct.
/// `tv` must be a valid non-null pointer to a `typval_T` struct.
#[export_name = "callback_put"]
pub unsafe extern "C" fn rs_callback_put(cb: *mut c_void, tv: TypevalHandle) {
    let cb = &*cb.cast::<CbData>();
    match cb.cb_type {
        K_CALLBACK_PARTIAL => {
            // tv->v_type = VAR_PARTIAL; tv->vval.v_partial = cb->data.partial; refcount++
            nvim_tv_set_partial(tv.as_ptr().cast_mut(), cb.data.partial);
            nvim_partial_inc_refcount(cb.data.partial);
        }
        K_CALLBACK_FUNCREF => {
            // tv->v_type = VAR_FUNC; tv->vval.v_string = xstrdup(cb->data.funcref)
            nvim_tv_set_type(tv, VarType::Func as c_int);
            let s = nvim_xstrdup(cb.data.funcref);
            nvim_tv_set_vstring_owned(tv.as_ptr().cast_mut(), s);
            func_ref(cb.data.funcref);
        }
        _ => {
            // Lua and None: set to v:null (VAR_SPECIAL, kSpecialVarNull=0)
            nvim_tv_set_special(tv.as_ptr().cast_mut(), 0);
        }
    }
}

/// Deep-copy a Callback with refcount bumps.
///
/// Migrated from C `callback_copy`.
///
/// # Safety
///
/// `dest` and `src` must be valid non-null pointers to `Callback` structs.
#[export_name = "callback_copy"]
pub unsafe extern "C" fn rs_callback_copy(dest: *mut c_void, src: *mut c_void) {
    let src = &*src.cast::<CbData>();
    let dest = &mut *dest.cast::<CbData>();
    dest.cb_type = src.cb_type;
    match src.cb_type {
        K_CALLBACK_PARTIAL => {
            dest.data.partial = src.data.partial;
            nvim_partial_inc_refcount(src.data.partial);
        }
        K_CALLBACK_FUNCREF => {
            dest.data.funcref = nvim_xstrdup(src.data.funcref);
            func_ref(src.data.funcref);
        }
        K_CALLBACK_LUA => {
            dest.data.luaref = nvim_callback_new_luaref(src.data.luaref);
        }
        _ => {
            dest.data.funcref = std::ptr::null_mut();
        }
    }
}

/// Format a Callback as a display string.
///
/// Migrated from C `callback_to_string`.
///
/// # Safety
///
/// `cb` must be a valid non-null pointer to a `Callback` struct.
/// `arena` may be null (heap allocation used).
#[export_name = "callback_to_string"]
pub unsafe extern "C" fn rs_callback_to_string(cb: *mut c_void, arena: *mut c_void) -> *mut c_char {
    let cb = &*cb.cast::<CbData>();
    if cb.cb_type == K_CALLBACK_LUA {
        return nvim_callback_funcref_str(cb.data.luaref, arena);
    }

    let msglen: usize = 100;
    let msg = cb_xmallocz(msglen);
    if msg.is_null() {
        return msg;
    }

    match cb.cb_type {
        K_CALLBACK_FUNCREF => {
            let fmt = b"<vim function: %s>\0".as_ptr().cast::<c_char>();
            snprintf(msg, msglen, fmt, cb.data.funcref);
        }
        K_CALLBACK_PARTIAL => {
            let name: *mut c_char = nvim_partial_get_name(cb.data.partial);
            let fmt = b"<vim partial: %s>\0".as_ptr().cast::<c_char>();
            snprintf(msg, msglen, fmt, name);
        }
        _ => {
            *msg = 0; // NUL-terminate: empty string
        }
    }
    msg
}

// =============================================================================
// Phase 2: tv_copy, tv_free, tv_equal (migrated from typval.c)
// =============================================================================

use std::cell::Cell;

// Thread-local state for tv_equal recursion tracking.
// These replicate the C static variables:
//   static int recursive_cnt = 0;
//   static int tv_equal_recurse_limit;
thread_local! {
    static TV_EQUAL_RECURSIVE_CNT: Cell<i32> = const { Cell::new(0) };
    static TV_EQUAL_RECURSE_LIMIT: Cell<i32> = const { Cell::new(1000) };
}

extern "C" {
    // Phase 2 accessors
    fn nvim_tv_get_partial(tv: TypevalHandle) -> *mut c_void;
    fn nvim_tv_get_string_mutable(tv: TypevalHandle) -> *mut c_char;
    fn nvim_tv_copy_vval(to: TypevalHandle, from: TypevalHandle);
    fn nvim_blob_inc_refcount(b: BlobHandle);
    fn nvim_mb_strcmp_ic(ic: bool, s1: *const c_char, s2: *const c_char) -> c_int;

    // tv_dict_unref (still in C, Phase 4 will migrate it)
    fn tv_dict_unref(d: DictHandle);

    // tv_list_equal (in Rust via export_name, callable via extern)
    fn tv_list_equal(l1: ListHandle, l2: ListHandle, ic: bool) -> bool;

    // tv_dict_equal (still in C, called from tv_equal)
    fn tv_dict_equal(d1: DictHandle, d2: DictHandle, ic: bool) -> bool;

    // tv_blob_equal (in Rust via export_name)
    fn tv_blob_equal(b1: BlobHandle, b2: BlobHandle) -> bool;

    // rs_func_equal (in Rust)
    fn rs_func_equal(tv1: TypevalHandle, tv2: TypevalHandle, ic: bool) -> bool;

}

/// Free allocated Vimscript object and value stored inside.
///
/// Migrated from C `tv_free`.
///
/// # Safety
///
/// `tv` must be a valid pointer to a heap-allocated `typval_T`, or null.
#[export_name = "tv_free"]
pub unsafe extern "C" fn rs_tv_free(tv: TypevalHandle) {
    if tv.is_null() {
        return;
    }
    let v_type = nvim_tv_get_type(tv);
    match VarType::from_c_int(v_type) {
        Some(VarType::Partial) => {
            let pt = nvim_tv_get_partial(tv);
            partial_unref(pt);
        }
        Some(VarType::Func) => {
            let s = nvim_tv_get_string_mutable(tv);
            func_unref(s);
            nvim_xfree(s.cast());
        }
        Some(VarType::String) => {
            let s = nvim_tv_get_string_mutable(tv);
            nvim_xfree(s.cast());
        }
        Some(VarType::Blob) => {
            let b = nvim_tv_get_blob(tv);
            // tv_blob_unref is in Rust
            rs_tv_blob_unref(b);
        }
        Some(VarType::List) => {
            let l = nvim_tv_get_list(tv);
            // tv_list_unref is in Rust
            rs_tv_list_unref(l);
        }
        Some(VarType::Dict) => {
            let d = nvim_tv_get_dict(tv);
            tv_dict_unref(d);
        }
        _ => {}
    }
    nvim_xfree(tv.as_ptr().cast_mut());
}

/// Free memory for a variable value and set the value to NULL or 0.
///
/// Differs from `tv_free` in that it does NOT free the typval struct itself,
/// and handles shared (refcount > 1) lists/dicts/partials by only decrementing.
///
/// Migrated from C `tv_clear` (and the encode-nothing framework it used).
///
/// # Safety
///
/// `tv` must be a valid pointer to a `typval_T`, or null.
#[export_name = "tv_clear"]
pub unsafe extern "C" fn rs_tv_clear(tv: TypevalHandle) {
    if tv.is_null() {
        return;
    }
    let v_type = unsafe { nvim_tv_get_type(tv) };
    if v_type == VarType::Unknown as c_int {
        return;
    }
    match VarType::from_c_int(v_type) {
        Some(VarType::Partial) => {
            let pt = unsafe { nvim_tv_get_partial(tv) };
            if !pt.is_null() && unsafe { nvim_partial_get_refcount(pt) } > 1 {
                unsafe { nvim_partial_dec_refcount(pt) };
                unsafe { nvim_tv_set_vpartial_null(tv) };
                unsafe { nvim_tv_set_lock(tv, 0) };
                return;
            }
            unsafe { partial_unref(pt) };
            unsafe { nvim_tv_set_vpartial_null(tv) };
        }
        Some(VarType::Func) => {
            let s = unsafe { nvim_tv_get_string_mutable(tv) };
            unsafe { func_unref(s) };
            // Only xfree if not the global tv_empty_string sentinel
            if s.cast_const() != unsafe { tv_empty_string_ptr } {
                unsafe { nvim_xfree(s.cast()) };
            }
            unsafe { nvim_tv_set_vstring_owned(tv.as_ptr().cast_mut(), core::ptr::null_mut()) };
        }
        Some(VarType::String) => {
            let s = unsafe { nvim_tv_get_string_mutable(tv) };
            unsafe { nvim_xfree(s.cast()) };
            unsafe { nvim_tv_set_vstring_owned(tv.as_ptr().cast_mut(), core::ptr::null_mut()) };
        }
        Some(VarType::Blob) => {
            let b = unsafe { nvim_tv_get_blob(tv) };
            unsafe { rs_tv_blob_unref(b) };
            unsafe { nvim_tv_set_vblob_null(tv) };
        }
        Some(VarType::List) => {
            let l = unsafe { nvim_tv_get_list(tv) };
            unsafe { rs_tv_list_unref(l) };
            unsafe { nvim_tv_set_vlist_null(tv) };
        }
        Some(VarType::Dict) => {
            let d = unsafe { nvim_tv_get_dict(tv) };
            unsafe { rs_tv_dict_unref(d) };
            unsafe { nvim_tv_set_vdict_null(tv) };
        }
        _ => {}
    }
    unsafe { nvim_tv_set_lock(tv, 0) }; // VAR_UNLOCKED
}

/// Copy typval from one location to another (shallow copy with refcount bumps).
///
/// Migrated from C `tv_copy`.
///
/// # Safety
///
/// `from` and `to` must be valid non-null pointers to `typval_T`.
#[export_name = "tv_copy"]
pub unsafe extern "C" fn rs_tv_copy(from: TypevalHandle, to: TypevalHandle) {
    // Copy v_type
    let v_type_int = nvim_tv_get_type(from);
    nvim_tv_set_type(to, v_type_int);
    // Set v_lock = VAR_UNLOCKED (0)
    nvim_tv_set_lock(to, 0);
    // Copy the vval union via memmove
    nvim_tv_copy_vval(to, from);

    // Per-type fixups
    match VarType::from_c_int(v_type_int) {
        Some(VarType::Number | VarType::Float | VarType::Bool | VarType::Special) => {
            // No refcount or pointer fixups needed
        }
        Some(VarType::String | VarType::Func) => {
            let s = nvim_tv_get_string_ptr(from);
            if !s.is_null() {
                let new_s = nvim_xstrdup(s);
                nvim_tv_set_vstring_owned(to.as_ptr().cast_mut(), new_s);
                if v_type_int == VarType::Func as c_int {
                    func_ref(new_s);
                }
            }
        }
        Some(VarType::Partial) => {
            let pt = nvim_tv_get_partial(to);
            if !pt.is_null() {
                nvim_partial_inc_refcount(pt);
            }
        }
        Some(VarType::Blob) => {
            let b = nvim_tv_get_blob(from);
            if !b.is_null() {
                nvim_blob_inc_refcount(b);
            }
        }
        Some(VarType::List) => {
            let l = nvim_tv_get_list(to);
            // tv_list_ref increments refcount
            rs_tv_list_ref(l);
        }
        Some(VarType::Dict) => {
            let d = nvim_tv_get_dict(from);
            if !d.is_null() {
                nvim_dict_inc_refcount(d);
            }
        }
        Some(VarType::Unknown) | None => {
            // Warn about UNKNOWN copy
            let fmt = b"E340: Internal error: %s\0".as_ptr().cast::<c_char>();
            let arg = b"tv_copy(UNKNOWN)\0".as_ptr().cast::<c_char>();
            semsg_typval(fmt, arg);
        }
    }
}

/// Compare two Vimscript values for equality.
///
/// Migrated from C `tv_equal`.
/// Uses thread-local state to track recursion depth.
///
/// # Safety
///
/// `tv1` and `tv2` must be valid non-null pointers to `typval_T`.
#[export_name = "tv_equal"]
pub unsafe extern "C" fn rs_tv_equal(tv1: TypevalHandle, tv2: TypevalHandle, ic: bool) -> bool {
    let t1 = VarType::from_c_int(nvim_tv_get_type(tv1)).unwrap_or(VarType::Unknown);
    let t2 = VarType::from_c_int(nvim_tv_get_type(tv2)).unwrap_or(VarType::Unknown);

    // Type mismatch check (except func/partial both count as func)
    let is_func1 = matches!(t1, VarType::Func | VarType::Partial);
    let is_func2 = matches!(t2, VarType::Func | VarType::Partial);
    if !(is_func1 && is_func2) && t1 != t2 {
        return false;
    }

    // Recursion limit tracking
    let cnt = TV_EQUAL_RECURSIVE_CNT.get();
    if cnt == 0 {
        TV_EQUAL_RECURSE_LIMIT.set(1000);
    }
    let limit = TV_EQUAL_RECURSE_LIMIT.get();
    if cnt >= limit {
        TV_EQUAL_RECURSE_LIMIT.set(limit - 1);
        return true;
    }

    match t1 {
        VarType::List => {
            TV_EQUAL_RECURSIVE_CNT.set(cnt + 1);
            let l1 = nvim_tv_get_list(tv1);
            let l2 = nvim_tv_get_list(tv2);
            let r = tv_list_equal(l1, l2, ic);
            TV_EQUAL_RECURSIVE_CNT.set(cnt);
            r
        }
        VarType::Dict => {
            TV_EQUAL_RECURSIVE_CNT.set(cnt + 1);
            let d1 = nvim_tv_get_dict(tv1);
            let d2 = nvim_tv_get_dict(tv2);
            let r = tv_dict_equal(d1, d2, ic);
            TV_EQUAL_RECURSIVE_CNT.set(cnt);
            r
        }
        VarType::Partial | VarType::Func => {
            // Check for null partial
            let p1_null = t1 == VarType::Partial && nvim_tv_get_partial(tv1).is_null();
            let p2_null = t2 == VarType::Partial && nvim_tv_get_partial(tv2).is_null();
            if p1_null || p2_null {
                return false;
            }
            TV_EQUAL_RECURSIVE_CNT.set(cnt + 1);
            let r = rs_func_equal(tv1, tv2, ic);
            TV_EQUAL_RECURSIVE_CNT.set(cnt);
            r
        }
        VarType::Blob => {
            let b1 = nvim_tv_get_blob(tv1);
            let b2 = nvim_tv_get_blob(tv2);
            tv_blob_equal(b1, b2)
        }
        VarType::Number => nvim_tv_get_number(tv1) == nvim_tv_get_number(tv2),
        #[allow(clippy::float_cmp)] // Match C behavior: exact float equality
        VarType::Float => nvim_tv_get_float(tv1) == nvim_tv_get_float(tv2),
        VarType::String => {
            let mut buf1 = [0u8; NUMBUFLEN];
            let mut buf2 = [0u8; NUMBUFLEN];
            let s1 = nvim_tv_get_string_buf(tv1, buf1.as_mut_ptr().cast());
            let s2 = nvim_tv_get_string_buf(tv2, buf2.as_mut_ptr().cast());
            nvim_mb_strcmp_ic(ic, s1, s2) == 0
        }
        VarType::Bool => nvim_tv_get_bool(tv1) == nvim_tv_get_bool(tv2),
        VarType::Special => nvim_tv_get_bool(tv1) == nvim_tv_get_bool(tv2), // same union offset
        VarType::Unknown => false,
    }
}

// Helpers: call the Rust-exported functions internally by their Rust names
// (avoiding need to go through FFI for functions we own)

// tv_list_ref increments refcount; use the one we already have
unsafe fn rs_tv_list_ref(l: ListHandle) {
    if !l.is_null() {
        nvim_list_ref(l);
    }
}

// =============================================================================
// Phase 3 (typval migration): tv_item_lock
// =============================================================================

thread_local! {
    /// Recursion counter for tv_item_lock (mirrors C static `recurse`).
    static TV_ITEM_LOCK_RECURSE: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}

/// CHANGE_LOCK logic: apply lock/unlock respecting VAR_FIXED.
///
/// VAR_UNLOCKED=0, VAR_LOCKED=1, VAR_FIXED=2
/// If current == VAR_FIXED: stays VAR_FIXED.
/// If lock: becomes VAR_LOCKED.
/// Else: becomes VAR_UNLOCKED.
fn change_lock(current: c_int, lock: bool) -> c_int {
    if current == 2 {
        2 // VAR_FIXED stays fixed
    } else {
        c_int::from(lock) // 1 if lock=true (VAR_LOCKED), 0 if false (VAR_UNLOCKED)
    }
}

/// Lock or unlock an item (and optionally its container contents).
///
/// Migrated from C `tv_item_lock`.
///
/// # Safety
///
/// `tv` must be a valid non-null pointer to `typval_T`.
#[export_name = "tv_item_lock"]
pub unsafe extern "C" fn rs_tv_item_lock(
    tv: TypevalHandle,
    deep: c_int,
    lock: bool,
    check_refcount: bool,
) {
    let recurse = TV_ITEM_LOCK_RECURSE.get();
    if recurse >= 100 {
        // DICT_MAXNEST = 100
        typval_err_item_lock_nested();
        return;
    }
    if deep == 0 {
        return;
    }
    TV_ITEM_LOCK_RECURSE.set(recurse + 1);

    // Lock/unlock the typval itself
    let v_lock = nvim_tv_get_v_lock(tv);
    nvim_tv_set_lock(tv, change_lock(v_lock, lock));

    match VarType::from_c_int(nvim_tv_get_type(tv)) {
        Some(VarType::Blob) => {
            let b = nvim_tv_get_blob(tv);
            if !b.is_null() {
                let bv_refcount = nvim_blob_get_bv_refcount(b);
                if !(check_refcount && bv_refcount > 1) {
                    let bv_lock = nvim_blob_get_lock(b);
                    nvim_blob_set_lock(b, change_lock(bv_lock, lock));
                }
            }
        }
        Some(VarType::List) => {
            let l = nvim_tv_get_list(tv);
            if !l.is_null() {
                let lv_refcount = nvim_list_get_refcount(l);
                if !(check_refcount && lv_refcount > 1) {
                    let lv_lock = nvim_list_get_lock(l);
                    nvim_list_set_lock(l, change_lock(lv_lock, lock));
                    if !(0..=1).contains(&deep) {
                        // Recursively lock list items
                        let mut li = nvim_list_get_first(l);
                        while !li.is_null() {
                            let item_tv = nvim_listitem_get_tv(li);
                            rs_tv_item_lock(item_tv, deep - 1, lock, check_refcount);
                            li = nvim_listitem_get_next(li);
                        }
                    }
                }
            }
        }
        Some(VarType::Dict) => {
            let d = nvim_tv_get_dict(tv);
            if !d.is_null() {
                let dv_refcount = nvim_dict_get_refcount(d);
                if !(check_refcount && dv_refcount > 1) {
                    let dv_lock = nvim_dict_get_lock(d);
                    nvim_dict_set_lock(d, change_lock(dv_lock, lock));
                    if !(0..=1).contains(&deep) {
                        // Recursively lock dict item values
                        let ht_used = nvim_dict_get_ht_used(d);
                        let hi_removed = nvim_hash_removed_ptr();
                        let mut hi = nvim_dict_get_ht_array(d);
                        let mut todo = ht_used;
                        while todo > 0 {
                            let key = nvim_hashitem_get_key(hi);
                            if !key.is_null() && key != hi_removed {
                                let di = nvim_hashitem_to_dictitem(hi);
                                let di_tv = nvim_dictitem_get_tv(di);
                                rs_tv_item_lock(di_tv, deep - 1, lock, check_refcount);
                                todo -= 1;
                            }
                            hi = nvim_hashitem_next(hi);
                        }
                    }
                }
            }
        }
        // Other types: no container to lock, just the typval v_lock (done above)
        _ => {}
    }

    TV_ITEM_LOCK_RECURSE.set(recurse);
}

// =============================================================================
// Phase 4 (typval migration): dict free/unref/equal/extend/copy
// =============================================================================

extern "C" {
    // Phase 4 accessors
    fn nvim_dict_free_hashtab_contents(d: DictHandle);
    fn nvim_dict_free_watchers(d: DictHandle);
    fn nvim_dict_gc_unlink_and_free(d: DictHandle);
    // nvim_get_tv_in_free_unref_items: already in existing extern block
    fn nvim_dict_is_watched(d: DictHandle) -> c_int;
    fn nvim_dict_item_copy_impl(di: DictItemHandle) -> DictItemHandle;
    fn nvim_dict_item_alloc_impl(key: *const c_char) -> DictItemHandle;
    fn nvim_dict_item_alloc_len_impl(key: *const c_char, len: usize) -> DictItemHandle;
    // nvim_dict_watcher_notify was removed (C accessor deleted in Phase 7).
    // tv_dict_watcher_notify is now exported directly from Rust.
    fn nvim_valid_varname(name: *const c_char) -> c_int;
    fn nvim_var_check_ro(flags: c_int, name: *const c_char, name_len: usize) -> bool;
    fn nvim_dictitem_get_flags(di: DictItemHandle) -> c_int;
    fn nvim_dict_hash_lock(d: DictHandle);
    fn nvim_dict_hash_unlock(d: DictHandle);
    fn nvim_dict_hash_remove(d: DictHandle, hi: HashItemHandle);
    fn nvim_dictitem_get_key_ptr(di: DictItemHandle) -> *const c_char;
    // nvim_semsg_key_exists: deleted (Phase 9), use typval_err_key_exists directly
    fn nvim_dict_copy_key_convert(
        conv: *const c_void,
        key: *const c_char,
        len_out: *mut usize,
    ) -> *mut c_char;
    fn nvim_dict_item_free_raw(di: DictItemHandle);
    fn nvim_dict_set_copyid_and_copydict(orig: DictHandle, copy_id: c_int, copy: DictHandle);
    // nvim_got_int already declared in existing extern block
    // nvim_get_tv_in_free_unref_items already declared in existing extern block
    fn nvim_dict_set_refcount(d: DictHandle, rc: c_int);
    fn nvim_dict_get_len_impl(d: DictHandle) -> c_int;
    fn nvim_dict_get_scope_impl(d: DictHandle) -> c_int;
    fn tv_dict_alloc() -> DictHandle;
    fn tv_dict_wrong_func_name(d: DictHandle, tv: TypevalHandle, name: *const c_char) -> c_int;
}

/// Free items contained in a dictionary.
///
/// Migrated from C `tv_dict_free_contents`.
///
/// # Safety
///
/// `d` must be a valid non-null pointer to `dict_T`.
#[export_name = "tv_dict_free_contents"]
pub unsafe extern "C" fn rs_tv_dict_free_contents(d: DictHandle) {
    nvim_dict_free_hashtab_contents(d);
    nvim_dict_free_watchers(d);
}

/// Free a dictionary itself, ignoring items it contains.
///
/// Migrated from C `tv_dict_free_dict`.
///
/// # Safety
///
/// `d` must be a valid non-null pointer to `dict_T`.
#[export_name = "tv_dict_free_dict"]
pub unsafe extern "C" fn rs_tv_dict_free_dict(d: DictHandle) {
    nvim_dict_gc_unlink_and_free(d);
}

/// Free a dictionary, including all items it contains.
///
/// Migrated from C `tv_dict_free`.
///
/// # Safety
///
/// `d` must be a valid non-null pointer to `dict_T`.
#[export_name = "tv_dict_free"]
pub unsafe extern "C" fn rs_tv_dict_free(d: DictHandle) {
    if nvim_get_tv_in_free_unref_items() != 0 {
        return;
    }
    rs_tv_dict_free_contents(d);
    rs_tv_dict_free_dict(d);
}

/// Unreference a dictionary: decrement refcount and free at zero.
///
/// Migrated from C `tv_dict_unref`.
///
/// # Safety
///
/// `d` must be a valid pointer to `dict_T`, or null.
#[export_name = "tv_dict_unref"]
pub unsafe extern "C" fn rs_tv_dict_unref(d: DictHandle) {
    if d.is_null() {
        return;
    }
    let rc = nvim_dict_get_refcount(d);
    let new_rc = rc - 1;
    // Update refcount via increment accessor in reverse:
    // We don't have a decrement accessor, so use nvim_dict_get_refcount/set approach.
    // Actually: dec refcount by calling the C helper directly with a trick:
    // We create a wrapper that decrements. Add nvim_dict_dec_refcount accessor.
    // For now, use the existing pattern: if rc <= 1, free; else decrement via...
    // Actually we need to set the refcount. Let's use:
    //   nvim_dict_inc_refcount sets +1. We need -1.
    // Use: set refcount = rc - 1
    nvim_dict_set_refcount(d, new_rc);
    if new_rc <= 0 {
        rs_tv_dict_free(d);
    }
}

// Note: nvim_dict_get_len_impl / nvim_dict_get_scope_impl are wrapper names used below

/// Compare two dictionaries for equality.
///
/// Migrated from C `tv_dict_equal`.
///
/// # Safety
///
/// `d1` and `d2` must be valid pointers to `dict_T`, or null.
#[export_name = "tv_dict_equal"]
pub unsafe extern "C" fn rs_tv_dict_equal(d1: DictHandle, d2: DictHandle, ic: bool) -> bool {
    if d1.as_ptr() == d2.as_ptr() {
        return true;
    }
    let len1 = nvim_dict_get_len_impl(d1);
    let len2 = nvim_dict_get_len_impl(d2);
    if len1 != len2 {
        return false;
    }
    if len1 == 0 {
        // empty and NULL dicts are considered equal
        return true;
    }
    if d1.is_null() || d2.is_null() {
        return false;
    }

    // Iterate d1, find each key in d2, compare values
    let ht_used = nvim_dict_get_ht_used(d1);
    let hi_removed = nvim_hash_removed_ptr();
    let mut hi = nvim_dict_get_ht_array(d1);
    let mut todo = ht_used;
    while todo > 0 {
        let key = nvim_hashitem_get_key(hi);
        if !key.is_null() && key != hi_removed {
            let di1 = nvim_hashitem_to_dictitem(hi);
            let key_ptr = nvim_dictitem_get_key_ptr(di1);
            let di2 = nvim_dict_find(d2, key_ptr, -1);
            if di2.is_null() {
                return false;
            }
            let tv1 = nvim_dictitem_get_tv(di1);
            let tv2 = nvim_dictitem_get_tv(di2);
            if !rs_tv_equal(tv1, tv2, ic) {
                return false;
            }
            todo -= 1;
        }
        hi = nvim_hashitem_next(hi);
    }
    true
}

/// The extend() argument error message (TV_CSTRING semantics).
static EXTEND_ARG_ERRMSG: &[u8] = b"extend() argument\0";

/// Extend dictionary d1 with items from d2.
///
/// Migrated from C `tv_dict_extend`.
///
/// # Safety
///
/// `d1`, `d2`, `action` must be valid non-null pointers.
#[export_name = "tv_dict_extend"]
pub unsafe extern "C" fn rs_tv_dict_extend(d1: DictHandle, d2: DictHandle, action: *const c_char) {
    let watched = nvim_dict_is_watched(d1) != 0;
    let arg_errmsg = EXTEND_ARG_ERRMSG.as_ptr().cast::<c_char>();
    let arg_errmsg_len = EXTEND_ARG_ERRMSG.len() - 1; // exclude NUL

    let action_char = *action as u8;

    if action_char == b'm' {
        nvim_dict_hash_lock(d2);
    }

    // Iterate d2 hashtab
    let ht_used = nvim_dict_get_ht_used(d2);
    let hi_removed = nvim_hash_removed_ptr();
    let mut hi = nvim_dict_get_ht_array(d2);
    let mut todo = ht_used;
    let mut should_break = false;

    while todo > 0 && !should_break {
        let key = nvim_hashitem_get_key(hi);
        if !key.is_null() && key != hi_removed {
            let di2 = nvim_hashitem_to_dictitem(hi);
            let di2_key = nvim_dictitem_get_key_ptr(di2);
            let di1 = nvim_dict_find(d1, di2_key, -1);

            // Check key validity for scoped dicts
            let scope = nvim_dict_get_scope_impl(d1);
            if scope != 0 && nvim_valid_varname(di2_key) == 0 {
                should_break = true;
            } else if di1.is_null() {
                if action_char == b'm' {
                    // Move item from d2 to d1
                    let new_di = di2;
                    let di2_tv = nvim_dictitem_get_tv(new_di);
                    if nvim_dict_add_item(d1, new_di) != 0 {
                        // OK = 1 (non-zero)
                        nvim_dict_hash_remove(d2, hi);
                        rs_tv_dict_watcher_notify(
                            d1,
                            di2_key,
                            di2_tv,
                            TypevalHandle::from_ptr(std::ptr::null()),
                        );
                    }
                } else {
                    let new_di = nvim_dict_item_copy_impl(di2);
                    let new_di_tv = nvim_dictitem_get_tv(new_di);
                    let new_di_key = nvim_dictitem_get_key_ptr(new_di);
                    if nvim_dict_add_item(d1, new_di) == 0 {
                        // FAIL = 0
                        nvim_dict_item_free(new_di);
                    } else if watched {
                        rs_tv_dict_watcher_notify(
                            d1,
                            new_di_key,
                            new_di_tv,
                            TypevalHandle::from_ptr(std::ptr::null()),
                        );
                    }
                }
            } else if action_char == b'e' {
                typval_err_key_exists(di2_key);
                should_break = true;
            } else if action_char == b'f' && di2 != di1 {
                let di1_tv = nvim_dictitem_get_tv(di1);
                let di1_v_lock = nvim_tv_get_v_lock(di1_tv);
                let di1_flags = nvim_dictitem_get_flags(di1);
                if value_check_lock_impl(di1_v_lock, arg_errmsg, arg_errmsg_len)
                    || nvim_var_check_ro(di1_flags, arg_errmsg, arg_errmsg_len)
                {
                    should_break = true;
                } else {
                    // Check for wrong func name
                    let di2_tv = nvim_dictitem_get_tv(di2);
                    if tv_dict_wrong_func_name(d1, di2_tv, di2_key) != 0 {
                        should_break = true;
                    } else if watched {
                        // Allocate properly aligned stack space for oldtv.
                        // typval_T is 16 bytes; [u64; 2] = 16 bytes with 8-byte alignment.
                        let mut oldtv_buf = [0u64; 2];
                        let oldtv_ptr = oldtv_buf.as_mut_ptr().cast::<c_void>();
                        let oldtv_th = TypevalHandle::from_ptr(oldtv_ptr);
                        rs_tv_copy(di1_tv, oldtv_th);
                        // clear di1_tv and copy di2_tv into it
                        tv_clear(di1_tv);
                        rs_tv_copy(di2_tv, di1_tv);
                        let di1_key = nvim_dictitem_get_key_ptr(di1);
                        rs_tv_dict_watcher_notify(d1, di1_key, di1_tv, oldtv_th);
                        tv_clear(oldtv_th);
                    } else {
                        tv_clear(di1_tv);
                        rs_tv_copy(di2_tv, di1_tv);
                    }
                }
            }
            // 'k' (keep) and other actions: just skip duplicates (do nothing)
            todo -= 1;
        }
        hi = nvim_hashitem_next(hi);
    }

    if action_char == b'm' {
        nvim_dict_hash_unlock(d2);
    }
}

/// Make a copy of dictionary.
///
/// Migrated from C `tv_dict_copy`.
///
/// # Safety
///
/// `conv` must be a valid pointer to `vimconv_T`, or null.
/// `orig` must be a valid pointer to `dict_T`, or null.
#[export_name = "tv_dict_copy"]
pub unsafe extern "C" fn rs_tv_dict_copy(
    conv: *const c_void,
    orig: DictHandle,
    deep: bool,
    copy_id: c_int,
) -> DictHandle {
    if orig.is_null() {
        return DictHandle::from_ptr(std::ptr::null());
    }

    let copy = tv_dict_alloc();

    if copy_id != 0 {
        nvim_dict_set_copyid_and_copydict(orig, copy_id, copy);
    }

    // Iterate orig hashtab
    let ht_used = nvim_dict_get_ht_used(orig);
    let hi_removed = nvim_hash_removed_ptr();
    let mut hi = nvim_dict_get_ht_array(orig);
    let mut todo = ht_used;
    let mut failed = false;

    // CONV_NONE = 0
    let conv_is_none = conv.is_null() || {
        // conv->vc_type is at offset 0 (int)
        let vc_type = *conv.cast::<c_int>();
        vc_type == 0
    };

    while todo > 0 && !failed {
        if nvim_got_int() != 0 {
            break;
        }
        let key = nvim_hashitem_get_key(hi);
        if !key.is_null() && key != hi_removed {
            let di = nvim_hashitem_to_dictitem(hi);
            let di_key = nvim_dictitem_get_key_ptr(di);

            let new_di = if conv_is_none {
                nvim_dict_item_alloc_impl(di_key)
            } else {
                let mut len_out: usize = 0;
                let converted_key =
                    nvim_dict_copy_key_convert(conv, di_key, std::ptr::addr_of_mut!(len_out));
                if converted_key.is_null() {
                    // Use original key with computed length
                    nvim_dict_item_alloc_len_impl(di_key, len_out)
                } else {
                    let new_item =
                        nvim_dict_item_alloc_len_impl(converted_key.cast_const(), len_out);
                    nvim_xfree(converted_key.cast());
                    new_item
                }
            };

            let di_tv = nvim_dictitem_get_tv(di);
            let new_di_tv = nvim_dictitem_get_tv(new_di);

            if deep {
                if var_item_copy(conv, di_tv, new_di_tv, deep, copy_id) == FAIL {
                    // FAIL = 0
                    nvim_dict_item_free_raw(new_di);
                    failed = true;
                }
            } else {
                rs_tv_copy(di_tv, new_di_tv);
            }

            if !failed && nvim_dict_add_item(copy, new_di) == 0 {
                // FAIL = 0
                nvim_dict_item_free(new_di);
                failed = true;
            }

            todo -= 1;
        }
        hi = nvim_hashitem_next(hi);
    }

    // Increment refcount
    nvim_dict_inc_refcount(copy);

    if nvim_got_int() != 0 {
        rs_tv_dict_unref(copy);
        return DictHandle::from_ptr(std::ptr::null());
    }

    copy
}

// =============================================================================
// Phase 5 (typval migration): dict helpers and tv_dict2list
// =============================================================================

/// Opaque handle to a DictWatcher.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DictWatcherHandle(*const std::ffi::c_void);

impl DictWatcherHandle {
    pub const fn null() -> Self {
        Self(std::ptr::null())
    }
    pub const unsafe fn from_ptr(ptr: *const std::ffi::c_void) -> Self {
        Self(ptr)
    }
    pub const fn as_ptr(self) -> *const std::ffi::c_void {
        self.0
    }
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

extern "C" {
    // Phase 5 accessors
    fn nvim_dict_get_hashtab_ptr(d: DictHandle) -> *const std::ffi::c_void;
    fn nvim_get_globvar_dict() -> DictHandle;
    fn nvim_get_funccal_local_ht() -> *const std::ffi::c_void;
    fn nvim_tv_is_func(tv: TypevalHandle) -> bool;
    fn nvim_var_wrong_func_name(name: *const c_char) -> bool;
    fn nvim_watcher_get_key_pattern(w: DictWatcherHandle) -> *const c_char;
    fn nvim_watcher_get_key_pattern_len(w: DictWatcherHandle) -> usize;
    fn nvim_watcher_get_callback_ptr(w: DictWatcherHandle) -> *mut c_void;
    fn nvim_set_selfdict(tv: TypevalHandle, d: DictHandle);
    // nvim_emsg_not_func_or_funcname: deleted (Phase 9), use typval_err_not_func_or_funcname directly
    fn nvim_tv_is_func_or_string(tv: TypevalHandle) -> bool;
    fn nvim_callback_from_typval_impl(result: *mut c_void, tv: TypevalHandle) -> bool;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
}

/// Check if a key matches a watcher's pattern (migrated from C `tv_dict_watcher_matches`).
///
/// # Safety
///
/// `watcher` and `key` must be valid non-null pointers.
#[export_name = "tv_dict_watcher_matches"]
pub unsafe extern "C" fn rs_tv_dict_watcher_matches(
    watcher: DictWatcherHandle,
    key: *const c_char,
) -> bool {
    let pattern = nvim_watcher_get_key_pattern(watcher);
    let pattern_len = nvim_watcher_get_key_pattern_len(watcher);

    if pattern_len > 0 {
        let last_byte = *pattern.add(pattern_len - 1) as u8;
        if last_byte == b'*' {
            return strncmp(key, pattern, pattern_len - 1) == 0;
        }
    }
    strcmp(key, pattern) == 0
}

/// Free a DictWatcher struct (migrated from C `tv_dict_watcher_free`).
///
/// # Safety
///
/// `watcher` must be a valid non-null pointer to a DictWatcher.
#[export_name = "tv_dict_watcher_free"]
pub unsafe extern "C" fn rs_tv_dict_watcher_free(watcher: DictWatcherHandle) {
    let cb_ptr = nvim_watcher_get_callback_ptr(watcher);
    rs_callback_free(cb_ptr);
    let key_pattern = nvim_watcher_get_key_pattern(watcher) as *mut c_void;
    nvim_xfree(key_pattern);
    nvim_xfree(watcher.as_ptr().cast_mut());
}

/// Check for adding a function to g: or l: (migrated from C `tv_dict_wrong_func_name`).
///
/// # Safety
///
/// `d`, `tv`, and `name` must be valid non-null pointers.
#[export_name = "tv_dict_wrong_func_name"]
pub unsafe extern "C" fn rs_tv_dict_wrong_func_name(
    d: DictHandle,
    tv: TypevalHandle,
    name: *const c_char,
) -> c_int {
    let globvar_dict = nvim_get_globvar_dict();
    let funccal_ht = nvim_get_funccal_local_ht();
    let dict_ht = nvim_dict_get_hashtab_ptr(d);

    let is_global_or_local =
        d.as_ptr() == globvar_dict.as_ptr() || dict_ht.cast_mut() == funccal_ht.cast_mut();

    c_int::from(is_global_or_local && nvim_tv_is_func(tv) && nvim_var_wrong_func_name(name))
}

/// Get a callback from a dict key (migrated from C `tv_dict_get_callback`).
///
/// # Safety
///
/// `d`, `key`, and `result` must be valid pointers as required by the C signature.
#[export_name = "tv_dict_get_callback"]
pub unsafe extern "C" fn rs_tv_dict_get_callback(
    d: DictHandle,
    key: *const c_char,
    key_len: isize,
    result: *mut c_void,
) -> bool {
    // Initialize result->type to kCallbackNone (0) at offset 8 (after 8-byte union)
    // result is Callback*: first 8 bytes are data union, next 4 bytes are type int.
    *(result.add(8).cast::<c_int>()) = 0; // kCallbackNone = 0

    let di = nvim_dict_find(d, key, key_len);
    if di.is_null() {
        return true;
    }

    let di_tv = nvim_dictitem_get_tv(di);
    if !nvim_tv_is_func_or_string(di_tv) {
        typval_err_not_func_or_funcname();
        return false;
    }

    // Allocate stack space for a temporary typval_T (16 bytes, 8-byte aligned)
    let mut tv_buf = [0u64; 2];
    let tv_ptr = tv_buf.as_mut_ptr().cast::<c_void>();
    let tv_handle = TypevalHandle::from_ptr(tv_ptr);

    // tv_copy(&di->di_tv, &tv)
    rs_tv_copy(di_tv, tv_handle);

    // set_selfdict(&tv, d)
    nvim_set_selfdict(tv_handle, d);

    // rs_callback_from_typval(result, &tv)
    let ok = nvim_callback_from_typval_impl(result, tv_handle);

    // tv_clear(&tv)
    tv_clear(tv_handle);

    ok
}

// =============================================================================
// Phase 6 (typval migration): sort/uniq, f_join, f_list2str
// =============================================================================

extern "C" {
    // Phase 6 sort/uniq accessors
    fn encode_tv2string(tv: TypevalHandle, len: *mut usize) -> *mut c_char;
    // nvim_emsg_sort_failed: deleted (Phase 9), use typval_err_sort_failed directly
    // nvim_emsg_uniq_failed: deleted (Phase 9), use typval_err_uniq_failed directly
    // nvim_emsg_listarg: deleted (Phase 9), use typval_err_listarg directly
    // nvim_emsg_invarg: deleted (Phase 9), use typval_err_invarg directly
    fn nvim_tv_check_for_dict_arg(argvars: TypevalHandle, idx: c_int) -> c_int;
    fn nvim_tv_get_string_checked(tv: TypevalHandle) -> *const c_char;
    // call_func for sort/uniq item_compare2
    fn call_func(
        funcname: *const c_char,
        len: c_int,
        rettv: *mut c_void,
        argcount: c_int,
        argvars: *mut c_void,
        funcexe: *mut c_void,
    ) -> c_int;
    fn rs_partial_name(pt: *mut c_void) -> *const c_char;
    fn strcasecmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strcoll(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strtod(s: *const c_char, endptr: *mut *mut c_char) -> f64;
    // Phase 6 join/list2str (nvim_list_join_to_string and nvim_f_list2str_from_list deleted Phase 3)
    // encode_tv2echo: stringify a typval for echo/join
    fn encode_tv2echo(tv: TypevalHandle, len: *mut usize) -> *mut c_char;
    // garray_T helpers (ga_* are Rust-exported functions from nvim-collections)
    fn ga_init(gap: *mut GArrayT, itemsize: c_int, growsize: c_int);
    fn ga_concat(gap: *mut GArrayT, s: *const c_char);
    fn ga_append(gap: *mut GArrayT, c: u8);
    fn ga_clear(gap: *mut GArrayT);
    // ga_grow: reserved for future use (not needed by current implementations)
    fn line_breakcheck();
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    // tv_get_number / tv_get_float for sort (C exported versions)
    fn tv_get_number(tv: TypevalHandle) -> i64;
    fn tv_get_float(tv: TypevalHandle) -> f64;
    // nvim_emsg_e_listreq: deleted (Phase 9), use typval_err_e_listreq directly
    // argvars[i] indexing
    fn nvim_tv_idx(argvars: *mut c_void, i: c_int) -> TypevalHandle;
}

/// Rust mirror of funcexe_T (Phase 6 sort).
///
/// Must match C definition in src/nvim/eval/userfunc.h exactly. Size = 64.
#[repr(C)]
struct SortFuncExe {
    fe_argv_func: *mut c_void,
    fe_firstline: i32,
    fe_lastline: i32,
    fe_doesrange: *mut c_void,
    fe_evaluate: bool,
    _pad: [u8; 7],
    fe_partial: *mut c_void,
    fe_selfdict: *mut c_void,
    fe_basetv: *mut c_void,
    fe_found_var: bool,
    _pad2: [u8; 7],
}

impl SortFuncExe {
    fn new() -> Self {
        // Safety: All-zero is valid for this repr(C) struct (NULL ptrs, false bools).
        unsafe { std::mem::zeroed() }
    }
}

/// Captures sort parameters, replacing C's global `sortinfo_T`.
#[allow(clippy::struct_excessive_bools)]
struct SortInfo {
    ic: bool,
    lc: bool,
    numeric: bool,
    numbers: bool,
    float_cmp: bool,
    func: *const c_char,   // borrowed from argvars, NULL if none
    partial: *mut c_void,  // partial_T*, NULL if none
    selfdict: *mut c_void, // dict_T*, NULL if none
    func_err: bool,
}

/// A sort element: list item pointer + original index (for stable sort).
struct SortItem {
    item: ListItemHandle,
    idx: i32,
}

/// Comparison logic for sort (no user function).
/// When `keep_zero` is false, ties are broken by index (stable sort).
unsafe fn item_compare_builtin(
    si1: &SortItem,
    si2: &SortItem,
    info: &SortInfo,
    keep_zero: bool,
) -> std::cmp::Ordering {
    let tv1 = nvim_listitem_get_tv(si1.item);
    let tv2 = nvim_listitem_get_tv(si2.item);

    let res: i32 = if info.numbers {
        let v1 = tv_get_number(tv1);
        let v2 = tv_get_number(tv2);
        match v1.cmp(&v2) {
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Less => -1,
        }
    } else if info.float_cmp {
        let v1 = tv_get_float(tv1);
        let v2 = tv_get_float(tv2);
        match v1.total_cmp(&v2) {
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Less => -1,
        }
    } else {
        let t1 = nvim_tv_get_type(tv1);
        let t2 = nvim_tv_get_type(tv2);
        let var_string = VarType::String as c_int;

        let mut tofree1: *mut c_char = std::ptr::null_mut();
        let mut tofree2: *mut c_char = std::ptr::null_mut();

        let single_quote = b"'\0".as_ptr().cast::<c_char>();

        let p1: *const c_char = if t1 == var_string {
            if t2 != var_string || info.numeric {
                single_quote
            } else {
                let s = nvim_tv_get_string_mutable(tv1);
                if s.is_null() {
                    b"\0".as_ptr().cast::<c_char>()
                } else {
                    s
                }
            }
        } else {
            tofree1 = encode_tv2string(tv1, std::ptr::null_mut());
            if tofree1.is_null() {
                b"\0".as_ptr().cast::<c_char>()
            } else {
                tofree1
            }
        };

        let p2: *const c_char = if t2 == var_string {
            if t1 != var_string || info.numeric {
                single_quote
            } else {
                let s = nvim_tv_get_string_mutable(tv2);
                if s.is_null() {
                    b"\0".as_ptr().cast::<c_char>()
                } else {
                    s
                }
            }
        } else {
            tofree2 = encode_tv2string(tv2, std::ptr::null_mut());
            if tofree2.is_null() {
                b"\0".as_ptr().cast::<c_char>()
            } else {
                tofree2
            }
        };

        let cmp = if !info.numeric {
            if info.lc {
                strcoll(p1, p2)
            } else if info.ic {
                strcasecmp(p1, p2)
            } else {
                strcmp(p1, p2)
            }
        } else {
            let n1 = strtod(p1, std::ptr::null_mut());
            let n2 = strtod(p2, std::ptr::null_mut());
            match n1.total_cmp(&n2) {
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
                std::cmp::Ordering::Less => -1,
            }
        };

        if !tofree1.is_null() {
            nvim_xfree(tofree1.cast::<c_void>());
        }
        if !tofree2.is_null() {
            nvim_xfree(tofree2.cast::<c_void>());
        }
        cmp
    };

    if res == 0 && !keep_zero {
        si1.idx.cmp(&si2.idx)
    } else {
        res.cmp(&0)
    }
}

/// Comparison with user function.
/// Sets info.func_err on failure.
unsafe fn item_compare_userfunc(
    si1: &SortItem,
    si2: &SortItem,
    info: &mut SortInfo,
    keep_zero: bool,
) -> std::cmp::Ordering {
    if info.func_err {
        return std::cmp::Ordering::Equal;
    }

    let partial = info.partial;
    let func_name: *const c_char = if partial.is_null() {
        info.func
    } else {
        rs_partial_name(partial)
    };

    // Stack-allocate two typval_T (16 bytes each) for argv and one for rettv.
    let mut argv_buf = [0u8; 32]; // 2 * sizeof(typval_T)
    let mut rettv_buf = [0u8; 16]; // 1 * sizeof(typval_T)

    let argv_ptr = argv_buf.as_mut_ptr().cast::<c_void>();
    let rettv_ptr = rettv_buf.as_mut_ptr().cast::<c_void>();

    let argv0 = TypevalHandle::from_ptr(argv_ptr);
    let argv1 = TypevalHandle::from_ptr(argv_ptr.add(16));
    let rettv_handle = TypevalHandle::from_ptr(rettv_ptr);

    rs_tv_copy(nvim_listitem_get_tv(si1.item), argv0);
    rs_tv_copy(nvim_listitem_get_tv(si2.item), argv1);

    let mut funcexe = SortFuncExe::new();
    funcexe.fe_evaluate = true;
    funcexe.fe_partial = partial;
    funcexe.fe_selfdict = info.selfdict;

    let res = call_func(
        func_name,
        -1,
        rettv_ptr,
        2,
        argv_ptr,
        std::ptr::addr_of_mut!(funcexe).cast::<c_void>(),
    );

    tv_clear(argv0);
    tv_clear(argv1);

    let ordering = if res == 0 {
        // FAIL
        info.func_err = true;
        std::cmp::Ordering::Equal
    } else {
        let mut func_err = false;
        let n = tv_get_number_chk(rettv_handle, std::ptr::addr_of_mut!(func_err));
        if func_err {
            info.func_err = true;
            std::cmp::Ordering::Equal
        } else if n > 0 {
            std::cmp::Ordering::Greater
        } else if n < 0 {
            std::cmp::Ordering::Less
        } else if keep_zero {
            std::cmp::Ordering::Equal
        } else {
            si1.idx.cmp(&si2.idx)
        }
    };

    tv_clear(rettv_handle);
    ordering
}

/// Sort a list in-place, rebuilding its internal linked list.
unsafe fn do_sort_impl(l: ListHandle, info: &mut SortInfo) {
    let len = tv_list_len_impl(l) as usize;
    if len == 0 {
        return;
    }

    let mut ptrs: Vec<SortItem> = Vec::with_capacity(len);
    let mut li = nvim_list_get_first(l);
    let mut idx = 0i32;
    while !li.is_null() {
        ptrs.push(SortItem { item: li, idx });
        li = nvim_listitem_get_next(li);
        idx += 1;
    }

    info.func_err = false;

    if !info.func.is_null() || !info.partial.is_null() {
        let info_ptr: *mut SortInfo = info;
        ptrs.sort_by(|a, b| item_compare_userfunc(a, b, &mut *info_ptr, false));
    } else {
        ptrs.sort_by(|a, b| item_compare_builtin(a, b, info, false));
    }

    if !info.func_err {
        // Rebuild the linked list in sorted order.
        nvim_list_set_first(l, ListItemHandle::null());
        nvim_list_set_last(l, ListItemHandle::null());
        nvim_list_set_len(l, 0);
        for si in &ptrs {
            rs_tv_list_append(l, si.item);
        }
    }
    if info.func_err {
        typval_err_sort_failed();
    }
}

/// Remove adjacent duplicates from a list (uniq).
unsafe fn do_uniq_impl(l: ListHandle, info: &mut SortInfo) {
    info.func_err = false;
    let use_func = !info.func.is_null() || !info.partial.is_null();

    let mut li = nvim_list_get_first(l);
    if li.is_null() {
        return;
    }
    li = nvim_listitem_get_next(li);

    while !li.is_null() {
        let prev_li = nvim_listitem_get_prev(li);
        let si_prev = SortItem {
            item: prev_li,
            idx: 0,
        };
        let si_curr = SortItem { item: li, idx: 0 };

        let equal = if use_func {
            let info_ptr: *mut SortInfo = info;
            item_compare_userfunc(&si_prev, &si_curr, &mut *info_ptr, true)
                == std::cmp::Ordering::Equal
        } else {
            item_compare_builtin(&si_prev, &si_curr, info, true) == std::cmp::Ordering::Equal
        };

        if equal {
            li = rs_tv_list_item_remove(l, li);
        } else {
            li = nvim_listitem_get_next(li);
        }

        if info.func_err {
            typval_err_uniq_failed();
            break;
        }
    }
}

/// Parse sort/uniq optional arguments into a `SortInfo`.
/// Returns true on success, false on error (emsg already emitted).
unsafe fn parse_sort_uniq_args_impl(argvars: TypevalHandle, info: &mut SortInfo) -> bool {
    info.ic = false;
    info.lc = false;
    info.numeric = false;
    info.numbers = false;
    info.float_cmp = false;
    info.func = std::ptr::null();
    info.partial = std::ptr::null_mut();
    info.selfdict = std::ptr::null_mut();

    let arg1 = nvim_tv_idx(argvars.as_ptr().cast_mut(), 1);
    if nvim_tv_get_type(arg1) == VarType::Unknown as c_int {
        return true;
    }

    if nvim_tv_get_type(arg1) == VarType::Func as c_int {
        info.func = nvim_tv_get_string_mutable(arg1);
    } else if nvim_tv_get_type(arg1) == VarType::Partial as c_int {
        info.partial = nvim_tv_get_partial(arg1);
    } else {
        let mut error = false;
        let nr = tv_get_number_chk(arg1, std::ptr::addr_of_mut!(error)) as i32;
        if error {
            return false;
        }
        if nr == 1 {
            info.ic = true;
        } else if nvim_tv_get_type(arg1) != VarType::Number as c_int {
            let s = nvim_tv_get_string_checked(arg1);
            if s.is_null() {
                return false;
            }
            info.func = s;
        } else if nr != 0 {
            typval_err_invarg();
            return false;
        }

        if !info.func.is_null() {
            if *info.func as u8 == 0 {
                info.func = std::ptr::null();
            } else if strcmp(info.func, b"n\0".as_ptr().cast::<c_char>()) == 0 {
                info.func = std::ptr::null();
                info.numeric = true;
            } else if strcmp(info.func, b"N\0".as_ptr().cast::<c_char>()) == 0 {
                info.func = std::ptr::null();
                info.numbers = true;
            } else if strcmp(info.func, b"f\0".as_ptr().cast::<c_char>()) == 0 {
                info.func = std::ptr::null();
                info.float_cmp = true;
            } else if strcmp(info.func, b"i\0".as_ptr().cast::<c_char>()) == 0 {
                info.func = std::ptr::null();
                info.ic = true;
            } else if strcmp(info.func, b"l\0".as_ptr().cast::<c_char>()) == 0 {
                info.func = std::ptr::null();
                info.lc = true;
            }
        }
    }

    let arg2 = nvim_tv_idx(argvars.as_ptr().cast_mut(), 2);
    if nvim_tv_get_type(arg2) != VarType::Unknown as c_int {
        if nvim_tv_check_for_dict_arg(argvars, 2) == 0 {
            return false;
        }
        info.selfdict = nvim_tv_get_dict(arg2).as_ptr().cast_mut();
    }

    true
}

/// Shared sort/uniq driver (replaces C `do_sort_uniq`).
unsafe fn do_sort_uniq_impl(argvars: TypevalHandle, rettv: TypevalHandle, sort: bool) {
    let arg0 = nvim_tv_idx(argvars.as_ptr().cast_mut(), 0);
    if nvim_tv_get_type(arg0) != VarType::List as c_int {
        let fname = if sort {
            b"sort()\0".as_ptr().cast::<c_char>()
        } else {
            b"uniq()\0".as_ptr().cast::<c_char>()
        };
        typval_err_listarg(fname);
        return;
    }

    let l = nvim_tv_get_list(arg0);
    let arg_errmsg = if sort {
        b"sort() argument\0".as_ptr().cast::<c_char>()
    } else {
        b"uniq() argument\0".as_ptr().cast::<c_char>()
    };

    let lock = tv_list_locked_impl(l);
    if value_check_lock_impl(lock, arg_errmsg, TV_TRANSLATE) {
        return;
    }

    // tv_list_set_ret(rettv, l) - sets rettv as the list return value
    nvim_tv_list_set_ret(rettv, l);

    let len = tv_list_len_impl(l);
    if len <= 1 {
        return;
    }

    let mut info = SortInfo {
        ic: false,
        lc: false,
        numeric: false,
        numbers: false,
        float_cmp: false,
        func: std::ptr::null(),
        partial: std::ptr::null_mut(),
        selfdict: std::ptr::null_mut(),
        func_err: false,
    };

    if !parse_sort_uniq_args_impl(argvars, &mut info) {
        return;
    }

    if sort {
        do_sort_impl(l, &mut info);
    } else {
        do_uniq_impl(l, &mut info);
    }
}

/// "sort({list})" VimL function.
#[export_name = "f_sort"]
pub unsafe extern "C" fn rs_f_sort(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    _fptr: *mut c_void,
) {
    do_sort_uniq_impl(argvars, rettv, true);
}

/// "uniq({list})" VimL function.
#[export_name = "f_uniq"]
pub unsafe extern "C" fn rs_f_uniq(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    _fptr: *mut c_void,
) {
    do_sort_uniq_impl(argvars, rettv, false);
}

/// Join list into a string using given separator (migrated from C `tv_list_join`).
///
/// Writes result into the caller-provided `garray_T` (gap). Returns OK on success, FAIL on
/// error (e.g. encode_tv2echo returns NULL or got_int is set).
///
/// # Safety
///
/// `gap` must be a valid pointer to a `garray_T` initialized for byte output.
/// `l` may be null (returns OK immediately).
/// `sep` must be a valid C string.
#[export_name = "tv_list_join"]
pub unsafe extern "C" fn rs_tv_list_join(
    gap: *mut c_void,
    l: ListHandle,
    sep: *const c_char,
) -> c_int {
    if tv_list_len_impl(l) == 0 {
        return OK;
    }

    // Stringify each list item via encode_tv2echo into a Rust vec.
    let mut strs: Vec<(*mut c_char, usize)> = Vec::new();
    let mut item = unsafe { nvim_list_get_first(l) };
    let mut failed = false;

    while !item.is_null() {
        if unsafe { nvim_got_int() } != 0 {
            failed = true;
            break;
        }
        let tv = unsafe { nvim_listitem_get_tv(item) };
        let mut len: usize = 0;
        let s = unsafe { encode_tv2echo(tv, &raw mut len) };
        if s.is_null() {
            failed = true;
            break;
        }
        strs.push((s, len));
        unsafe { line_breakcheck() };
        item = unsafe { nvim_listitem_get_next(item) };
    }

    if failed || unsafe { nvim_got_int() } != 0 {
        for (s, _) in &strs {
            unsafe { nvim_xfree(s.cast()) };
        }
        return FAIL;
    }

    // Concatenate into the garray with separators.
    let gap = gap.cast::<GArrayT>();
    let mut first = true;
    for (s, _) in &strs {
        if first {
            first = false;
        } else if !sep.is_null() {
            unsafe { ga_concat(gap, sep) };
        }
        if !s.is_null() {
            unsafe { ga_concat(gap, s.cast_const()) };
        }
        if unsafe { nvim_got_int() } != 0 {
            break;
        }
        unsafe { line_breakcheck() };
    }

    // Free all encoded strings.
    for (s, _) in &strs {
        unsafe { nvim_xfree(s.cast()) };
    }

    OK
}

/// "join({list})" VimL function.
#[export_name = "f_join"]
pub unsafe extern "C" fn rs_f_join(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    _fptr: *mut c_void,
) {
    let arg0 = unsafe { nvim_tv_idx(argvars.as_ptr().cast_mut(), 0) };
    if unsafe { nvim_tv_get_type(arg0) } != VarType::List as c_int {
        unsafe { typval_err_e_listreq() };
        return;
    }

    unsafe { nvim_tv_set_type(rettv, VarType::String as c_int) };

    let arg1 = unsafe { nvim_tv_idx(argvars.as_ptr().cast_mut(), 1) };
    let sep: *const c_char = if unsafe { nvim_tv_get_type(arg1) } == VarType::Unknown as c_int {
        b" \0".as_ptr().cast::<c_char>()
    } else {
        let s = unsafe { nvim_tv_get_string_checked(arg1) };
        if s.is_null() {
            unsafe { nvim_tv_set_string(rettv, std::ptr::null_mut()) };
            return;
        }
        s
    };

    let l = unsafe { nvim_tv_get_list(arg0) };

    // Use rs_tv_list_join with a local garray to build the result string.
    let mut gap = GArrayT::new();
    unsafe { ga_init(&raw mut gap, 1, 80) };
    let rc = unsafe { rs_tv_list_join((&raw mut gap).cast(), l, sep) };
    if rc == OK {
        unsafe { ga_append(&raw mut gap, 0) }; // NUL-terminate
                                               // ga_data is the result string (owned by the garray).
        unsafe { nvim_tv_set_string(rettv, gap.ga_data.cast::<c_char>()) };
        // Don't free gap.ga_data - ownership transferred to rettv.
    } else {
        unsafe { ga_clear(&raw mut gap) };
        unsafe { nvim_tv_set_string(rettv, std::ptr::null_mut()) };
    }
}

/// "list2str({list})" VimL function (inlined from C `nvim_f_list2str_from_list`).
#[export_name = "f_list2str"]
pub unsafe extern "C" fn rs_f_list2str(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    _fptr: *mut c_void,
) {
    unsafe { nvim_tv_set_type(rettv, VarType::String as c_int) };
    unsafe { nvim_tv_set_string(rettv, std::ptr::null_mut()) };

    let arg0 = unsafe { nvim_tv_idx(argvars.as_ptr().cast_mut(), 0) };
    if unsafe { nvim_tv_get_type(arg0) } != VarType::List as c_int {
        unsafe { typval_err_invarg() };
        return;
    }

    let l = unsafe { nvim_tv_get_list(arg0) };
    if l.is_null() {
        return;
    }

    // Build UTF-8 string from list of codepoints using a garray.
    let mut ga = GArrayT::new();
    unsafe { ga_init(&raw mut ga, 1, 80) };

    let mut item = unsafe { nvim_list_get_first(l) };
    while !item.is_null() {
        let tv = unsafe { nvim_listitem_get_tv(item) };
        let n = unsafe { nvim_tv_get_number_simple(tv) } as c_int;
        let mut buf = [0u8; 7]; // MB_MAXBYTES + 1
        let len = unsafe { utf_char2bytes(n, buf.as_mut_ptr().cast::<c_char>()) } as usize;
        // Temporarily NUL-terminate for ga_concat.
        buf[len] = 0;
        unsafe { ga_concat(&raw mut ga, buf.as_ptr().cast::<c_char>()) };
        item = unsafe { nvim_listitem_get_next(item) };
    }

    // NUL-terminate the result.
    unsafe { ga_append(&raw mut ga, 0) };
    // Transfer ownership of ga_data to rettv.
    unsafe { nvim_tv_set_string(rettv, ga.ga_data.cast::<c_char>()) };
    // Don't free ga.ga_data - ownership transferred.
}

// tv_list_append_owned_tv remains in C (by-value struct ABI not compatible with TypevalHandle).

// =============================================================================
// Phase 7 (typval migration): dict watcher add/remove/notify
// =============================================================================

/// Raw mirror of C `Callback` struct (16 bytes, verified).
///
/// Layout:
/// ```text
/// offset 0: data (8 bytes, union: char*/partial_T*/LuaRef)
/// offset 8: cb_type (i32, CallbackType)
/// offset 12: _pad (4 bytes)
/// ```
#[repr(C)]
pub struct CallbackRaw {
    data: u64,
    cb_type: i32,
    _pad: u32,
}

/// QUEUE node raw accessor (two pointer-sized fields: next, prev).
/// We use *mut *mut c_void to read next/prev without defining the full QUEUE type.
type QueuePtr = *mut c_void;

extern "C" {
    // Phase 7: C accessors for DictWatcher fields
    fn nvim_dict_get_watchers_head(d: DictHandle) -> QueuePtr;
    fn nvim_watcher_node_data(node: QueuePtr) -> *mut c_void; // -> DictWatcher*
    fn nvim_watcher_get_busy(w: *mut c_void) -> bool;
    fn nvim_watcher_set_busy(w: *mut c_void, v: bool);
    fn nvim_watcher_get_needs_free(w: *mut c_void) -> bool;
    fn nvim_watcher_set_needs_free(w: *mut c_void, v: bool);
    fn nvim_callback_equal_raw(cb1: *const c_void, cb2: *const c_void) -> bool;

    // Phase 7: memory / dict operations
    fn xmalloc(size: usize) -> *mut c_void;
    fn xmemdupz(data: *const c_void, len: usize) -> *mut c_char;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> c_int;
    fn callback_call(
        cb: *const c_void,
        argcount: c_int,
        argv: *mut c_void,
        rettv: *mut c_void,
    ) -> bool;

    // Phase 7: QUEUE link/unlink (from collections crate, already exported as C symbols)
    fn rs_queue_insert_tail(h: QueuePtr, q: QueuePtr);
    fn rs_queue_remove(q: QueuePtr);
}

/// Read the `next` pointer from a QUEUE node (first 8 bytes = pointer to next node).
unsafe fn queue_next(node: QueuePtr) -> QueuePtr {
    *(node.cast::<QueuePtr>())
}

/// Build a typval_T [u8; 16] for a dict value.
/// v_type=VAR_DICT(5) at offset 0, v_lock=VAR_UNLOCKED(0) at offset 4, dict ptr at offset 8.
#[allow(clippy::cast_ptr_alignment)]
unsafe fn make_tv_dict_raw(d: DictHandle) -> [u8; 16] {
    let mut tv = [0u8; 16];
    tv.as_mut_ptr().cast::<i32>().write_unaligned(5); // VAR_DICT = 5
    tv.as_mut_ptr().add(4).cast::<i32>().write_unaligned(0); // VAR_UNLOCKED = 0
    tv.as_mut_ptr()
        .add(8)
        .cast::<*const c_void>()
        .write_unaligned(d.as_ptr());
    tv
}

/// Build a typval_T [u8; 16] for a string value.
/// v_type=VAR_STRING(2) at offset 0, v_lock=VAR_UNLOCKED(0) at offset 4, char* at offset 8.
#[allow(clippy::cast_ptr_alignment)]
unsafe fn make_tv_string_raw(s: *mut c_char) -> [u8; 16] {
    let mut tv = [0u8; 16];
    tv.as_mut_ptr().cast::<i32>().write_unaligned(2); // VAR_STRING = 2
    tv.as_mut_ptr().add(4).cast::<i32>().write_unaligned(0); // VAR_UNLOCKED = 0
    tv.as_mut_ptr()
        .add(8)
        .cast::<*mut c_char>()
        .write_unaligned(s);
    tv
}

/// Add a watcher to a dictionary (migrated from C `tv_dict_watcher_add`).
///
/// DictWatcher layout (56 bytes, verified):
/// - offset  0: Callback callback (16 bytes)
/// - offset 16: char *key_pattern (8 bytes)
/// - offset 24: size_t key_pattern_len (8 bytes)
/// - offset 32: QUEUE node (16 bytes: next ptr + prev ptr)
/// - offset 48: bool busy (1 byte)
/// - offset 49: bool needs_free (1 byte)
///
/// # Safety
///
/// `dict` and `key_pattern` must be valid. `key_pattern` must point to at least
/// `key_pattern_len` bytes.
#[allow(clippy::cast_ptr_alignment, clippy::borrow_as_ptr)]
#[export_name = "tv_dict_watcher_add"]
pub unsafe extern "C" fn rs_tv_dict_watcher_add(
    dict: DictHandle,
    key_pattern: *const c_char,
    key_pattern_len: usize,
    callback: CallbackRaw,
) {
    if dict.is_null() {
        return;
    }
    let watcher = xmalloc(56).cast::<u8>();

    // offset 0: Callback (16 bytes)
    watcher.cast::<CallbackRaw>().write(callback);

    // offset 16: char *key_pattern
    let dup = xmemdupz(key_pattern.cast::<c_void>(), key_pattern_len);
    watcher.add(16).cast::<*mut c_char>().write(dup);

    // offset 24: size_t key_pattern_len
    watcher.add(24).cast::<usize>().write(key_pattern_len);

    // offset 32: QUEUE node — initialize as self-referential (rs_queue_init equivalent)
    let node: QueuePtr = watcher.add(32).cast::<c_void>();
    watcher.add(32).cast::<QueuePtr>().write(node); // node->next = node
    watcher.add(40).cast::<QueuePtr>().write(node); // node->prev = node

    // offset 48: busy = false, offset 49: needs_free = false
    watcher.add(48).write(0);
    watcher.add(49).write(0);

    // Insert into dict's watchers queue tail
    let head = nvim_dict_get_watchers_head(dict);
    rs_queue_insert_tail(head, node);
}

/// Remove a matching watcher from a dictionary (migrated from C `tv_dict_watcher_remove`).
///
/// # Safety
///
/// `dict` and `key_pattern` must be valid. `key_pattern` must point to at least
/// `key_pattern_len` bytes.
#[allow(clippy::cast_ptr_alignment, clippy::borrow_as_ptr)]
#[export_name = "tv_dict_watcher_remove"]
pub unsafe extern "C" fn rs_tv_dict_watcher_remove(
    dict: DictHandle,
    key_pattern: *const c_char,
    key_pattern_len: usize,
    callback: CallbackRaw,
) -> bool {
    if dict.is_null() {
        return false;
    }

    let head = nvim_dict_get_watchers_head(dict);
    let mut matched_node: QueuePtr = std::ptr::null_mut();
    let mut matched_watcher: *mut c_void = std::ptr::null_mut();
    let mut queue_is_busy = false;

    // QUEUE_FOREACH equivalent
    let mut w = queue_next(head);
    while !std::ptr::eq(w, head) {
        let next = queue_next(w);
        let watcher_ptr = nvim_watcher_node_data(w);

        if nvim_watcher_get_busy(watcher_ptr) {
            queue_is_busy = true;
        }

        // Compare callback and key_pattern
        let watcher_cb = watcher_ptr.cast::<c_void>(); // callback at offset 0
        let cb_ref: *const c_void = std::ptr::from_ref::<CallbackRaw>(&callback).cast::<c_void>();
        let watcher_kp = watcher_ptr
            .cast::<u8>()
            .add(16)
            .cast::<*const c_char>()
            .read();
        let watcher_kp_len = watcher_ptr.cast::<u8>().add(24).cast::<usize>().read();

        if nvim_callback_equal_raw(watcher_cb, cb_ref)
            && watcher_kp_len == key_pattern_len
            && memcmp(
                watcher_kp.cast::<c_void>(),
                key_pattern.cast::<c_void>(),
                key_pattern_len,
            ) == 0
        {
            matched_node = w;
            matched_watcher = watcher_ptr;
            break;
        }

        w = next;
    }

    if matched_watcher.is_null() {
        return false;
    }

    if queue_is_busy {
        nvim_watcher_set_needs_free(matched_watcher, true);
    } else {
        rs_queue_remove(matched_node);
        rs_tv_dict_watcher_free(DictWatcherHandle::from_ptr(matched_watcher));
    }
    true
}

/// Notify all matching dict watchers of a key change (migrated from C `tv_dict_watcher_notify`).
///
/// # Safety
///
/// `dict` and `key` must be valid non-null pointers. `newtv` and `oldtv` may be null.
#[allow(clippy::cast_ptr_alignment)]
#[export_name = "tv_dict_watcher_notify"]
pub unsafe extern "C" fn rs_tv_dict_watcher_notify(
    dict: DictHandle,
    key: *const c_char,
    newtv: TypevalHandle,
    oldtv: TypevalHandle,
) {
    // argv[0] = dict typval, argv[1] = key string typval, argv[2] = changes dict typval
    let argv0 = make_tv_dict_raw(dict);
    let key_dup = xstrdup(key);
    let argv1 = make_tv_string_raw(key_dup);

    let changes_dict = tv_dict_alloc();
    nvim_dict_inc_refcount(changes_dict);
    let argv2 = make_tv_dict_raw(changes_dict);

    // Optionally add "new" key to changes dict
    if !newtv.is_null() {
        let di = nvim_dict_item_alloc_len(b"new\0".as_ptr().cast::<c_char>(), 3);
        rs_tv_copy(newtv, nvim_dictitem_get_tv(di));
        nvim_dict_add_item(changes_dict, di);
    }

    // Optionally add "old" key to changes dict (only if oldtv != NULL and not VAR_UNKNOWN=0)
    if !oldtv.is_null() {
        let oldtv_type = oldtv.as_ptr().cast::<i32>().read(); // v_type at offset 0
        if oldtv_type != 0 {
            // VAR_UNKNOWN = 0
            let di = nvim_dict_item_alloc_len(b"old\0".as_ptr().cast::<c_char>(), 3);
            rs_tv_copy(oldtv, nvim_dictitem_get_tv(di));
            nvim_dict_add_item(changes_dict, di);
        }
    }

    // Concatenate the three typvals into a 48-byte array
    let mut argv = [0u8; 48];
    argv[0..16].copy_from_slice(&argv0);
    argv[16..32].copy_from_slice(&argv1);
    argv[32..48].copy_from_slice(&argv2);

    // Increment dict refcount to prevent premature free during iteration
    nvim_dict_inc_refcount(dict);

    let head = nvim_dict_get_watchers_head(dict);
    let mut any_needs_free = false;

    // First pass: call matching, non-busy watchers
    let mut w = queue_next(head);
    while !std::ptr::eq(w, head) {
        let next = queue_next(w);
        let watcher_ptr = nvim_watcher_node_data(w);
        let watcher_handle = DictWatcherHandle::from_ptr(watcher_ptr);

        if !nvim_watcher_get_busy(watcher_ptr) && rs_tv_dict_watcher_matches(watcher_handle, key) {
            let mut rettv = [0u8; 16]; // TV_INITIAL_VALUE (all zeros = VAR_UNKNOWN)
            nvim_watcher_set_busy(watcher_ptr, true);
            // callback is at offset 0 in DictWatcher
            callback_call(
                watcher_ptr.cast::<c_void>(),
                3,
                argv.as_mut_ptr().cast::<c_void>(),
                rettv.as_mut_ptr().cast::<c_void>(),
            );
            nvim_watcher_set_busy(watcher_ptr, false);
            tv_clear(TypevalHandle::from_ptr(rettv.as_mut_ptr().cast::<c_void>()));
            if nvim_watcher_get_needs_free(watcher_ptr) {
                any_needs_free = true;
            }
        }

        w = next;
    }

    // Second pass: free watchers that were marked needs_free during callbacks
    if any_needs_free {
        let mut w2 = queue_next(head);
        while !std::ptr::eq(w2, head) {
            let next = queue_next(w2);
            let watcher_ptr = nvim_watcher_node_data(w2);
            if nvim_watcher_get_needs_free(watcher_ptr) {
                rs_queue_remove(w2);
                rs_tv_dict_watcher_free(DictWatcherHandle::from_ptr(watcher_ptr));
            }
            w2 = next;
        }
    }

    // Decrement the refcount we incremented above
    rs_tv_dict_unref(dict);

    // Clear argv[1] (key string) and argv[2] (changes dict) - argv[0] (dict) is not owned
    tv_clear(TypevalHandle::from_ptr(
        argv[16..32].as_mut_ptr().cast::<c_void>(),
    ));
    tv_clear(TypevalHandle::from_ptr(
        argv[32..48].as_mut_ptr().cast::<c_void>(),
    ));
}

// =============================================================================
// Phase 1: filter/map/mapnew/foreach in Rust
// =============================================================================

/// Crate-internal accessor: get v_lock from a typval_T.
#[inline]
pub(crate) fn get_v_lock(tv: TypevalHandle) -> c_int {
    if tv.is_null() {
        return 0;
    }
    unsafe { nvim_tv_get_v_lock(tv) }
}

pub mod filter_map;

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
