#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

pub struct AutoCmdVec {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut AutoCmd,
}
/// The khash-derived maps and sets, one monomorphisation per key/value pair.
///
/// None of them is `Copy`. Each owns its [`MapHash`]'s bucket table and the
/// `keys` (and, for a map, `values`) array beside it, all of which
/// `map_destroy`/`set_destroy` free exactly once. `Clone` stays, so the few
/// places that really do want a second handle say so.
#[derive(Clone)]
pub struct Map_String_int {
    pub set: Set_String,
    pub values: *mut ::core::ffi::c_int,
}
#[derive(Clone)]
pub struct Map_cstr_t_int {
    pub set: Set_cstr_t,
    pub values: *mut ::core::ffi::c_int,
}
#[derive(Clone)]
#[repr(C)]
pub struct Map_cstr_t_ptr_t {
    pub set: Set_cstr_t,
    pub values: *mut ptr_t,
}
#[derive(Clone)]
pub struct Map_int64_t_int64_t {
    pub set: Set_int64_t,
    pub values: *mut int64_t,
}
#[derive(Clone)]
pub struct Map_int64_t_ptr_t {
    pub set: Set_int64_t,
    pub values: *mut ptr_t,
}
#[derive(Clone)]
pub struct Map_int_String {
    pub set: Set_int,
    pub values: *mut String_0,
}
#[derive(Clone)]
pub struct Map_ptr_t_ptr_t {
    pub set: Set_ptr_t,
    pub values: *mut ptr_t,
}
#[derive(Clone)]
pub struct Map_uint32_t_ptr_t {
    pub set: Set_uint32_t,
    pub values: *mut ptr_t,
}
pub struct Map_uint32_t_uint32_t {
    pub set: Set_uint32_t,
    pub values: *mut uint32_t,
}
pub struct Map_uint64_t_MTDamagePair {
    pub set: Set_uint64_t,
    pub values: *mut MTDamagePair,
}
pub struct Map_uint64_t_int {
    pub set: Set_uint64_t,
    pub values: *mut ::core::ffi::c_int,
}
pub struct Map_uint64_t_ptr_t {
    pub set: Set_uint64_t,
    pub values: *mut ptr_t,
}
pub type OptIndex = ::core::ffi::c_int;
pub struct ParserHighlight {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ParserHighlightChunk,
    pub init_array: [ParserHighlightChunk; 16],
}
#[repr(C)]
pub struct ScopeDictDictItem {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 1],
}
#[derive(Clone)]
pub struct Set_String {
    pub h: MapHash,
    pub keys: *mut String_0,
}
#[derive(Clone)]
#[repr(C)]
pub struct Set_cstr_t {
    pub h: MapHash,
    pub keys: *mut cstr_t,
}
#[derive(Clone)]
pub struct Set_glyph {
    pub h: MapHash,
    pub keys: *mut ::core::ffi::c_char,
}
#[derive(Clone)]
pub struct Set_int {
    pub h: MapHash,
    pub keys: *mut ::core::ffi::c_int,
}
#[derive(Clone)]
pub struct Set_int64_t {
    pub h: MapHash,
    pub keys: *mut int64_t,
}
#[derive(Clone)]
pub struct Set_ptr_t {
    pub h: MapHash,
    pub keys: *mut ptr_t,
}
#[derive(Clone)]
pub struct Set_uint32_t {
    pub h: MapHash,
    pub keys: *mut uint32_t,
}
pub struct Set_uint64_t {
    pub h: MapHash,
    pub keys: *mut uint64_t,
}
#[repr(C)]
pub struct StringArray {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut String_0,
}
#[derive(Copy, Clone)]
pub struct StringBuilder {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_char,
}
pub type TermKey_Terminfo_Getstr_Hook = unsafe extern "C" fn(
    *const ::core::ffi::c_char,
    *const ::core::ffi::c_char,
    *mut ::core::ffi::c_void,
) -> *const ::core::ffi::c_char;
pub type VTermOutputCallback =
    unsafe extern "C" fn(*const ::core::ffi::c_char, size_t, *mut ::core::ffi::c_void) -> ();
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VirtLines {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut virt_line,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VirtText {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut VirtTextChunk,
}
#[derive(Copy, Clone)]
pub struct caller_scope {
    pub script_ctx: sctx_T,
    pub es_entry: estack_T,
    pub autocmd_fname: *mut ::core::ffi::c_char,
    pub autocmd_match: *mut ::core::ffi::c_char,
    pub autocmd_fname_full: bool,
    pub autocmd_bufnr: ::core::ffi::c_int,
    pub funccalp: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictitem_T {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 0],
}
pub struct mod_entry_T {
    pub flag: ::core::ffi::c_int,
    pub name: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
pub struct nvim_stats_s {
    pub fsync: int64_t,
    pub redraw: int64_t,
    pub log_skip: int16_t,
}
#[derive(Copy, Clone)]
pub struct virt_line {
    pub line: VirtText,
    pub flags: ::core::ffi::c_int,
}
