//! Complete info support (complete_info() VimL function).
//!
//! This module provides the Rust wrapper for get_complete_info(), which builds
//! a VimL dictionary with current completion state. The heavy VimL type
//! interaction (dict_T, list_T) remains in C via compound accessor.

use std::os::raw::c_void;

extern "C" {
    /// Compound accessor: performs the full complete_info() implementation in C.
    /// Takes opaque handles for the what_list (nullable) and retdict.
    fn nvim_get_complete_info_impl(what_list: *mut c_void, retdict: *mut c_void);

    /// Compound accessor: f_complete_info argument parsing + dispatch.
    /// Handles tv_dict_alloc_ret, argvars[0] type-checking, and calls
    /// rs_get_complete_info with the extracted what_list and retdict.
    fn nvim_f_complete_info_impl(argvars: *mut c_void, rettv: *mut c_void);
}

/// Get complete information for the complete_info() VimL function.
///
/// Dispatches the what_flag parsing and dictionary population to the C
/// compound accessor, which handles all VimL type interactions.
///
/// # Safety
/// Requires valid VimL typval state. Called from f_complete_info.
#[no_mangle]
pub unsafe extern "C" fn rs_get_complete_info(what_list: *mut c_void, retdict: *mut c_void) {
    nvim_get_complete_info_impl(what_list, retdict);
}

/// VimL `complete_info()` builtin.
///
/// Allocates the return dict, parses the optional what_list argument,
/// and populates the dict with current completion state.
///
/// # Safety
/// `argvars` must be a valid `typval_T[]` pointer; `rettv` a `typval_T*`.
#[export_name = "f_complete_info"]
pub unsafe extern "C" fn rs_f_complete_info(
    argvars: *mut c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_f_complete_info_impl(argvars, rettv);
}
