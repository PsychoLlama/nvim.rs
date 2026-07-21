// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct VimMenu {
    pub modes: ::core::ffi::c_int,
    pub enabled: ::core::ffi::c_int,
    pub name: *mut ::core::ffi::c_char,
    pub dname: *mut ::core::ffi::c_char,
    pub en_name: *mut ::core::ffi::c_char,
    pub en_dname: *mut ::core::ffi::c_char,
    pub mnemonic: ::core::ffi::c_int,
    pub actext: *mut ::core::ffi::c_char,
    pub priority: ::core::ffi::c_int,
    pub strings: [*mut ::core::ffi::c_char; 8],
    pub noremap: [::core::ffi::c_int; 8],
    pub silent: [bool; 8],
    pub children: *mut vimmenu_T,
    pub parent: *mut vimmenu_T,
    pub next: *mut vimmenu_T,
}
pub type vimmenu_T = VimMenu;
