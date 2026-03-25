//! Complete info support (complete_info() VimL function).
//!
//! This module provides the Rust wrapper for get_complete_info(), which builds
//! a VimL dictionary with current completion state. The heavy VimL type
//! interaction (dict_T, list_T) remains in C via compound accessor.

use std::os::raw::{c_char, c_int, c_void};

// typval_T layout for argument parsing
// sizeof = 16; v_type at offset 0, vval (union) at offset 8
#[repr(C)]
union TypvalVvalInfo {
    v_number: i64,
    v_list: *mut c_void,
    v_dict: *mut c_void,
}

#[repr(C)]
struct TypvalTInfo {
    v_type: c_int,
    v_lock: c_int,
    vval: TypvalVvalInfo,
}

const VAR_UNKNOWN_INFO: c_int = 0;
const VAR_LIST_INFO: c_int = 4;

extern "C" {
    /// Compound accessor: performs the full complete_info() implementation in C.
    /// Takes opaque handles for the what_list (nullable) and retdict.
    fn nvim_get_complete_info_impl(what_list: *mut c_void, retdict: *mut c_void);

    // nvim_f_complete_info_impl: deleted (Phase 32), inlined in rs_f_complete_info below
    fn tv_dict_alloc_ret(rettv: *mut TypvalTInfo);
    #[link_name = "emsg"]
    fn emsg_info(s: *const c_char);
    #[link_name = "gettext"]
    fn gettext_info(msgid: *const c_char) -> *const c_char;
    #[link_name = "e_listreq"]
    static e_listreq_info: [c_char; 0];
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
    // nvim_f_complete_info_impl: deleted (Phase 32), inlined here
    let rettv = rettv.cast::<TypvalTInfo>();
    tv_dict_alloc_ret(rettv);

    let argvars = argvars.cast::<TypvalTInfo>();
    let what_list = if (*argvars).v_type == VAR_UNKNOWN_INFO {
        core::ptr::null_mut()
    } else {
        if (*argvars).v_type != VAR_LIST_INFO {
            emsg_info(gettext_info(e_listreq_info.as_ptr()));
            return;
        }
        (*argvars).vval.v_list
    };
    rs_get_complete_info(what_list, (*rettv).vval.v_dict);
}
