//! Option default value handling.
//!
//! This module provides helpers for option defaults:
//! - Default value retrieval
//! - Default value comparison
//! - Reset to defaults

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::c_int;

// =============================================================================
// Phase 8 Option Default Value Management (Phase 2)
// =============================================================================

use crate::opt_index::K_OPT_COUNT;
use crate::storage::OptVal;

extern "C" {
    /// Get options[opt_idx].def_val
    fn nvim_option_get_def_val(opt_idx: c_int) -> OptVal;
    /// Set options[opt_idx].def_val
    fn nvim_set_option_def_val(opt_idx: c_int, val: OptVal);
    /// Copy an OptVal (allocates new storage for strings)
    fn rs_optval_copy(o: OptVal) -> OptVal;
    /// Free an OptVal (releases string storage)
    fn rs_optval_free(o: OptVal);
    /// Get options[opt_idx].type as c_int (kOptValTypeString == 2)
    fn nvim_get_option_type(opt_idx: c_int) -> c_int;
    /// Get options[opt_idx].var pointer
    fn nvim_get_option_var(opt_idx: c_int) -> *mut std::ffi::c_void;
    /// Returns get_varp(&options[opt_idx]) for check_options loop
    fn nvim_get_option_varp_for_check(opt_idx: c_int) -> *mut std::ffi::c_void;
    /// Call check_string_option on the given pointer
    fn nvim_call_check_string_option(ptr: *mut *mut std::ffi::c_char);
    /// Get cmdheight default value as number
    fn nvim_get_cmdheight_def_number() -> i64;
    /// Set p_ch (cmdheight) global variable
    fn nvim_set_p_ch(value: i64);
    /// Copy a C string (xstrdup equivalent via copy_option_val)
    fn nvim_call_copy_option_val(val: *const std::ffi::c_char) -> *mut std::ffi::c_char;
}

/// kOptValTypeString constant (must match C kOptValTypeString = 2)
const K_OPT_VAL_TYPE_STRING: c_int = 2;

/// Allocate (copy) all option default values.
///
/// Mirrors C `alloc_options_default`.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_alloc_options_default() {
    let count = K_OPT_COUNT;
    for opt_idx in 0..count {
        let def_val = nvim_option_get_def_val(opt_idx);
        let copied = rs_optval_copy(def_val);
        nvim_set_option_def_val(opt_idx, copied);
    }
}

/// Change the default value for an option.
///
/// Mirrors C `change_option_default`.
/// Takes ownership of `value`.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_change_option_default(opt_idx: c_int, value: OptVal) {
    let old = nvim_option_get_def_val(opt_idx);
    rs_optval_free(old);
    nvim_set_option_def_val(opt_idx, value);
}

/// Set the Vi-default value of a string option.
///
/// Mirrors C `set_string_default`.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_set_string_default_opt(
    opt_idx: c_int,
    val: *mut std::ffi::c_char,
    allocated: c_int,
) {
    // Build an OptVal from the string. If not already allocated, copy it.
    let ptr = if allocated != 0 {
        val
    } else {
        nvim_call_copy_option_val(val)
    };
    // Build a string OptVal wrapping ptr. This mirrors CSTR_AS_OPTVAL(ptr).
    // OptVal layout: type=2 (String), data.string = { ptr, strlen(ptr) }
    let len = if ptr.is_null() { 0 } else { libc_strlen(ptr) };
    let val = build_string_optval(ptr, len);
    rs_change_option_default(opt_idx, val);
}

/// Set p_ch to the cmdheight default value.
///
/// Mirrors C `set_init_tablocal`.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_set_init_tablocal() {
    let ch_default = nvim_get_cmdheight_def_number();
    nvim_set_p_ch(ch_default);
}

/// Check all string options for NULL values.
///
/// Mirrors C `check_options`.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_check_options() {
    let count = K_OPT_COUNT;
    for opt_idx in 0..count {
        if nvim_get_option_type(opt_idx) == K_OPT_VAL_TYPE_STRING
            && !nvim_get_option_var(opt_idx).is_null()
        {
            let varp = nvim_get_option_varp_for_check(opt_idx);
            nvim_call_check_string_option(varp.cast());
        }
    }
}

// =============================================================================
// Phase 8 validate_option_value cluster (Phase 3)
// =============================================================================

use crate::index::OptIndex;
use crate::opt_index::{
    K_OPT_AUTOCOMPLETE, K_OPT_AUTOREAD, K_OPT_MODELINE, K_OPT_SCROLLOFF, K_OPT_SIDESCROLLOFF,
    K_OPT_UNDOLEVELS,
};

extern "C" {
    /// Check if running as root user (getuid() == ROOT_UID)
    fn nvim_is_root_user() -> c_int;
    /// Get NO_LOCAL_UNDOLEVEL constant (-123456)
    fn nvim_get_no_local_undolevel() -> i64;
    /// Check if option is global-local (scope_flags is not a power of two)
    fn rs_option_is_global_local(opt_idx: c_int) -> c_int;
    /// Check if option has the given type
    fn rs_option_has_type(opt_idx: c_int, type_: c_int) -> c_int;
    /// Get options[opt_idx].flags (as u32)
    fn nvim_get_option_flags(opt_idx: c_int) -> u32;
    /// Get options[opt_idx].def_val.data.string.data
    fn nvim_option_expand(opt_idx: c_int, val: *const std::ffi::c_char) -> *mut std::ffi::c_char;
    /// Get the varp for the given scope (OPT_GLOBAL=1, OPT_LOCAL=2)
    fn nvim_get_varp_scope_by_idx(opt_idx: c_int, opt_flags: c_int) -> *mut std::ffi::c_void;
    /// Convert varp to OptVal
    fn nvim_optval_from_varp(opt_idx: c_int, varp: *mut std::ffi::c_void) -> OptVal;
    /// Get empty_string_option pointer (for STATIC_CSTR_AS_OPTVAL(""))
    fn nvim_get_empty_string_option() -> *mut std::ffi::c_char;
}

/// kOptValTypeString constant (matches C kOptValTypeString = 2)
const K_OPT_VAL_TYPE_STRING_P3: c_int = 2;
/// kOptFlagNoDefExp: bit 1 - don't expand default value
const K_OPT_FLAG_NO_DEF_EXP: u32 = 1 << 1;
/// OPT_LOCAL flag (must match C OPT_LOCAL = 0x02)
const OPT_LOCAL: c_int = 0x02;
/// OPT_GLOBAL flag (must match C OPT_GLOBAL = 0x01)
const OPT_GLOBAL: c_int = 0x01;

/// Get the "unset" sentinel value for a global-local option's local scope.
///
/// Mirrors C `get_option_unset_value`.
///
/// # Panics
/// Panics if `opt_idx` is a global-local option not handled by the switch
/// (i.e., not autocomplete, autoread, scrolloff, sidescrolloff, or undolevels).
///
/// # Safety
/// Calls C accessor functions. opt_idx must be valid (not kOptInvalid).
#[no_mangle]
pub unsafe extern "C" fn rs_get_option_unset_value(opt_idx: OptIndex) -> OptVal {
    // For global-local options, return the unset sentinel for the local scope.
    if rs_option_is_global_local(opt_idx) != 0 {
        // String global-local options use empty string as the unset value.
        if rs_option_has_type(opt_idx, K_OPT_VAL_TYPE_STRING_P3) != 0 {
            let empty = nvim_get_empty_string_option();
            return OptVal {
                type_: OptValType::String,
                data: OptValData {
                    string: String_ {
                        data: empty,
                        size: 0,
                    },
                },
            };
        }

        // Non-string global-local options have specific unset sentinels.
        if opt_idx == K_OPT_AUTOCOMPLETE || opt_idx == K_OPT_AUTOREAD {
            // kNone = -1 as TriState boolean
            return OptVal {
                type_: OptValType::Boolean,
                data: OptValData { boolean: -1 },
            };
        }
        if opt_idx == K_OPT_SCROLLOFF || opt_idx == K_OPT_SIDESCROLLOFF {
            return OptVal {
                type_: OptValType::Number,
                data: OptValData { number: -1 },
            };
        }
        if opt_idx == K_OPT_UNDOLEVELS {
            return OptVal {
                type_: OptValType::Number,
                data: OptValData {
                    number: nvim_get_no_local_undolevel(),
                },
            };
        }
        // Unknown global-local option - this should not happen.
        // Mirror C's abort() behavior via panic.
        panic!("rs_get_option_unset_value: unhandled global-local opt_idx {opt_idx}");
    }

    // For non-global-local options, return the global value as the unset sentinel.
    let varp = nvim_get_varp_scope_by_idx(opt_idx, OPT_GLOBAL);
    nvim_optval_from_varp(opt_idx, varp)
}

/// Get the default value for an option, with scope and root-user awareness.
///
/// Mirrors C `get_option_default`.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_get_option_default(opt_idx: OptIndex, opt_flags: c_int) -> OptVal {
    // On Unix, modeline defaults to off for root.
    if opt_idx == K_OPT_MODELINE && nvim_is_root_user() != 0 {
        return OptVal {
            type_: OptValType::Boolean,
            data: OptValData { boolean: 0 }, // false
        };
    }

    let is_global_local = rs_option_is_global_local(opt_idx) != 0;

    if (opt_flags & OPT_LOCAL) != 0 && is_global_local {
        // Use unset local value instead of default for local scope of global-local options.
        rs_get_option_unset_value(opt_idx)
    } else if rs_option_has_type(opt_idx, K_OPT_VAL_TYPE_STRING_P3) != 0
        && (nvim_get_option_flags(opt_idx) & K_OPT_FLAG_NO_DEF_EXP) == 0
    {
        // For string options, expand environment variables and ~ if needed.
        let def_val = nvim_option_get_def_val(opt_idx);
        let def_str = def_val.data.string.data;
        let s = nvim_option_expand(opt_idx, def_str);
        if s.is_null() {
            return def_val;
        }
        // Build a string OptVal from the expanded (xmalloc'd) string.
        let len = {
            let mut p = s.cast_const();
            while *p != 0 {
                p = p.add(1);
            }
            p.offset_from(s.cast_const()) as usize
        };
        OptVal {
            type_: OptValType::String,
            data: OptValData {
                string: String_ { data: s, size: len },
            },
        }
    } else {
        nvim_option_get_def_val(opt_idx)
    }
}

// =============================================================================
// Internal helpers
// =============================================================================

use crate::storage::{OptValData, String_};
use crate::OptValType;

/// Build a string OptVal from a C string pointer and length.
unsafe fn build_string_optval(ptr: *mut std::ffi::c_char, len: usize) -> OptVal {
    OptVal {
        type_: OptValType::String,
        data: OptValData {
            string: String_ {
                data: ptr,
                size: len,
            },
        },
    }
}

/// Portable strlen for C strings.
unsafe fn libc_strlen(s: *const std::ffi::c_char) -> usize {
    let mut p = s;
    while *p != 0 {
        p = p.add(1);
    }
    p.offset_from(s) as usize
}

// =============================================================================
// Default Source Constants
// =============================================================================

/// Default is hard-coded in source.
pub const DEFAULT_BUILTIN: c_int = 0;
/// Default is from modeline.
pub const DEFAULT_MODELINE: c_int = 1;
/// Default is from vimrc.
pub const DEFAULT_VIMRC: c_int = 2;
/// Default is from environment variable.
pub const DEFAULT_ENV: c_int = 3;
/// Default is from system config.
pub const DEFAULT_SYSTEM: c_int = 4;

// =============================================================================
// Default Flags
// =============================================================================

/// Option has been changed from default.
pub const OPT_CHANGED: c_int = 0x01;
/// Option was set by user.
pub const OPT_USER_SET: c_int = 0x02;
/// Option was set by modeline.
pub const OPT_MODELINE_SET: c_int = 0x04;
/// Option was set by script.
pub const OPT_SCRIPT_SET: c_int = 0x08;
/// Option is at factory default.
pub const OPT_FACTORY: c_int = 0x10;

// =============================================================================
// Default Value Helpers
// =============================================================================

/// Check if option has been changed from default.
fn is_changed_from_default(flags: c_int) -> bool {
    (flags & OPT_CHANGED) != 0
}

/// Check if option was user-set.
fn is_user_set(flags: c_int) -> bool {
    (flags & OPT_USER_SET) != 0
}

/// Check if option was set by modeline.
fn is_modeline_set(flags: c_int) -> bool {
    (flags & OPT_MODELINE_SET) != 0
}

/// Check if option was set by script.
fn is_script_set(flags: c_int) -> bool {
    (flags & OPT_SCRIPT_SET) != 0
}

/// Check if option is at factory default.
fn is_factory_default(flags: c_int) -> bool {
    (flags & OPT_FACTORY) != 0 && !is_changed_from_default(flags)
}

/// Get priority of default source.
fn default_source_priority(source: c_int) -> c_int {
    match source {
        DEFAULT_BUILTIN => 0,
        DEFAULT_SYSTEM => 1,
        DEFAULT_ENV => 2,
        DEFAULT_VIMRC => 3,
        DEFAULT_MODELINE => 4,
        _ => -1,
    }
}

/// Check if first source has higher priority than second.
fn source_has_higher_priority(source1: c_int, source2: c_int) -> bool {
    default_source_priority(source1) > default_source_priority(source2)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get DEFAULT_BUILTIN constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_default_builtin() -> c_int {
    DEFAULT_BUILTIN
}

/// FFI: Get DEFAULT_MODELINE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_default_modeline() -> c_int {
    DEFAULT_MODELINE
}

/// FFI: Get DEFAULT_VIMRC constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_default_vimrc() -> c_int {
    DEFAULT_VIMRC
}

/// FFI: Get DEFAULT_ENV constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_default_env() -> c_int {
    DEFAULT_ENV
}

/// FFI: Get DEFAULT_SYSTEM constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_default_system() -> c_int {
    DEFAULT_SYSTEM
}

/// FFI: Get OPT_CHANGED constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_changed_flag() -> c_int {
    OPT_CHANGED
}

/// FFI: Get OPT_USER_SET constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_user_set_flag() -> c_int {
    OPT_USER_SET
}

/// FFI: Get OPT_MODELINE_SET constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_modeline_set_flag() -> c_int {
    OPT_MODELINE_SET
}

/// FFI: Get OPT_SCRIPT_SET constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_script_set_flag() -> c_int {
    OPT_SCRIPT_SET
}

/// FFI: Get OPT_FACTORY constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_factory_flag() -> c_int {
    OPT_FACTORY
}

/// FFI: Check if changed from default.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_is_changed(flags: c_int) -> c_int {
    c_int::from(is_changed_from_default(flags))
}

/// FFI: Check if user-set.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_is_user_set(flags: c_int) -> c_int {
    c_int::from(is_user_set(flags))
}

/// FFI: Check if modeline-set.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_is_modeline_set(flags: c_int) -> c_int {
    c_int::from(is_modeline_set(flags))
}

/// FFI: Check if script-set.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_is_script_set(flags: c_int) -> c_int {
    c_int::from(is_script_set(flags))
}

/// FFI: Check if factory default.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_is_factory_default(flags: c_int) -> c_int {
    c_int::from(is_factory_default(flags))
}

/// FFI: Get default source priority.
#[unsafe(no_mangle)]
pub extern "C" fn rs_default_source_priority(source: c_int) -> c_int {
    default_source_priority(source)
}

/// FFI: Check if source has higher priority.
#[unsafe(no_mangle)]
pub extern "C" fn rs_source_has_higher_priority(source1: c_int, source2: c_int) -> c_int {
    c_int::from(source_has_higher_priority(source1, source2))
}

// =============================================================================
// Phase 12: option_expand, set_option_default, set_options_default, free_all_options
// =============================================================================

use crate::opt_index::K_OPT_SCROLL;
use crate::OptFlags;
use std::ffi::c_uint;

extern "C" {
    /// rs_option_is_hidden(opt_idx) - returns 1 if option is hidden
    fn rs_option_is_hidden(opt_idx: c_int) -> c_int;
    /// rs_option_is_global_only(opt_idx)
    fn rs_option_is_global_only(opt_idx: c_int) -> c_int;
    /// rs_option_is_window_local(opt_idx)
    fn rs_option_is_window_local(opt_idx: c_int) -> c_int;
    /// rs_optval_from_varp(opt_idx, varp) - convert varp pointer to OptVal
    fn rs_optval_from_varp(opt_idx: c_int, varp: *mut std::ffi::c_void) -> OptVal;
    /// rs_insecure_flag(wp, opt_idx, opt_flags) - get pointer to insecure flag
    fn rs_insecure_flag(wp: *mut std::ffi::c_void, opt_idx: c_int, opt_flags: c_int)
        -> *mut c_uint;
    /// nvim_get_koptflag_insecure() - get kOptFlagInsecure constant
    fn nvim_get_koptflag_insecure() -> c_uint;
    /// nvim_opt_get_curwin() - get current window handle
    fn nvim_opt_get_curwin() -> *mut std::ffi::c_void;
    /// set_option_direct with current_sctx.sc_sid
    fn nvim_call_set_option_direct_with_sctx(opt_idx: c_int, val: OptVal, opt_flags: c_int);
    /// win_comp_scroll(curwin)
    fn nvim_call_win_comp_scroll_curwin();
    /// FOR_ALL_TAB_WINDOWS { win_comp_scroll(wp) }
    fn nvim_call_comp_scroll_all_windows();
    /// parse_cino(curbuf)
    fn nvim_call_parse_cino_curbuf();
    /// free_operatorfunc_option()
    fn nvim_call_free_operatorfunc_option();
    /// free_findfunc_option()
    fn nvim_call_free_findfunc_option();
    /// rs_free_tagfunc_option() - free tagfunc option
    fn rs_free_tagfunc_option();
    /// XFREE_CLEAR(fenc_default)
    fn nvim_call_xfree_clear_fenc_default();
    /// XFREE_CLEAR(p_term)
    fn nvim_call_xfree_clear_p_term();
    /// XFREE_CLEAR(p_ttytype)
    fn nvim_call_xfree_clear_p_ttytype();
    /// option_expand escape kind for opt_idx (0=none, 1=esc, 2=file:)
    fn nvim_option_expand_escape_kind(opt_idx: c_int) -> c_int;
    /// expand_env_esc into NameBuff; returns NameBuff or NULL if no change
    fn nvim_call_expand_env_esc_option(
        val: *const std::ffi::c_char,
        esc_kind: c_int,
    ) -> *const std::ffi::c_char;
}

/// OPT_LOCAL flag (must match C OPT_LOCAL = 0x02)
const OPT_LOCAL_P12: c_int = 0x02;
/// OPT_GLOBAL flag (must match C OPT_GLOBAL = 0x01)
const OPT_GLOBAL_P12: c_int = 0x01;
/// MAXPATHL constant (must match C MAXPATHL = 4096)
const MAXPATHL: usize = 4096;

/// Expand environment variables for a string option.
///
/// Mirrors C `option_expand`.
/// Returns pointer to NameBuff (static buffer), or NULL when not expanded.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_option_expand(
    opt_idx: c_int,
    val: *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    // If option doesn't need expansion, nothing to do.
    if (nvim_get_option_flags(opt_idx) & OptFlags::EXPAND.0) == 0
        || rs_option_is_hidden(opt_idx) != 0
    {
        return std::ptr::null();
    }

    // If val is NULL, read the current string value from options[opt_idx].var.
    let actual_val: *const std::ffi::c_char = if val.is_null() {
        let var_ptr = nvim_get_option_var(opt_idx);
        if var_ptr.is_null() {
            return std::ptr::null();
        }
        // options[opt_idx].var is char** for string options; dereference it.
        *(var_ptr.cast::<*const std::ffi::c_char>())
    } else {
        val
    };

    if actual_val.is_null() {
        return std::ptr::null();
    }

    // If val is longer than MAXPATHL, no meaningful expansion can be done.
    let mut len = 0usize;
    let mut p = actual_val;
    while *p != 0 {
        len += 1;
        if len > MAXPATHL {
            return std::ptr::null();
        }
        p = p.add(1);
    }

    // Get escape kind and expand.
    let esc_kind = nvim_option_expand_escape_kind(opt_idx);
    nvim_call_expand_env_esc_option(actual_val, esc_kind)
}

/// Set one option to its default value.
///
/// Mirrors C `set_option_default`.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_set_option_default(opt_idx: c_int, opt_flags: c_int) {
    let both = (opt_flags & (OPT_LOCAL_P12 | OPT_GLOBAL_P12)) == 0;
    let def_val = rs_get_option_default(opt_idx, opt_flags);
    nvim_call_set_option_direct_with_sctx(opt_idx, def_val, opt_flags);

    if opt_idx == K_OPT_SCROLL {
        nvim_call_win_comp_scroll_curwin();
    }

    // The default value is not insecure.
    let curwin = nvim_opt_get_curwin();
    let insecure_bit = nvim_get_koptflag_insecure();
    let flagsp = rs_insecure_flag(curwin, opt_idx, opt_flags);
    *flagsp &= !insecure_bit;
    if both {
        let flagsp2 = rs_insecure_flag(curwin, opt_idx, OPT_LOCAL_P12);
        *flagsp2 &= !insecure_bit;
    }
}

/// Set all options (except terminal options) to their default value.
///
/// Mirrors C `set_options_default`.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_set_options_default(opt_flags: c_int) {
    let count = K_OPT_COUNT;
    for opt_idx in 0..count {
        let flags = OptFlags(nvim_get_option_flags(opt_idx));
        if !flags.contains(OptFlags::NO_DEFAULT) {
            rs_set_option_default(opt_idx, opt_flags);
        }
    }

    // The 'scroll' option must be computed for all windows.
    nvim_call_comp_scroll_all_windows();

    nvim_call_parse_cino_curbuf();
}

/// Free all options (EXITFREE path only).
///
/// Mirrors C `free_all_options`.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_free_all_options() {
    let count = K_OPT_COUNT;
    for opt_idx in 0..count {
        let hidden = rs_option_is_hidden(opt_idx) != 0;
        if rs_option_is_global_only(opt_idx) != 0 || hidden {
            // global option: free value and default value.
            // hidden option: free default value only.
            if !hidden {
                let var_ptr = nvim_get_option_var(opt_idx);
                rs_optval_free(rs_optval_from_varp(opt_idx, var_ptr));
            }
        } else if rs_option_is_window_local(opt_idx) == 0 {
            // buffer-local option: free global value.
            let var_ptr = nvim_get_option_var(opt_idx);
            rs_optval_free(rs_optval_from_varp(opt_idx, var_ptr));
        }
        rs_optval_free(nvim_option_get_def_val(opt_idx));
    }
    nvim_call_free_operatorfunc_option();
    rs_free_tagfunc_option();
    nvim_call_free_findfunc_option();
    nvim_call_xfree_clear_fenc_default();
    nvim_call_xfree_clear_p_term();
    nvim_call_xfree_clear_p_ttytype();
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_source_constants() {
        assert_eq!(DEFAULT_BUILTIN, 0);
        assert_eq!(DEFAULT_MODELINE, 1);
        assert_eq!(DEFAULT_VIMRC, 2);
        assert_eq!(DEFAULT_ENV, 3);
        assert_eq!(DEFAULT_SYSTEM, 4);
    }

    #[test]
    fn test_default_flags() {
        assert_eq!(OPT_CHANGED, 0x01);
        assert_eq!(OPT_USER_SET, 0x02);
        assert_eq!(OPT_MODELINE_SET, 0x04);
        assert_eq!(OPT_SCRIPT_SET, 0x08);
        assert_eq!(OPT_FACTORY, 0x10);
    }

    #[test]
    fn test_is_changed_from_default() {
        assert!(is_changed_from_default(OPT_CHANGED));
        assert!(!is_changed_from_default(0));
        assert!(!is_changed_from_default(OPT_USER_SET));
    }

    #[test]
    fn test_is_user_set() {
        assert!(is_user_set(OPT_USER_SET));
        assert!(!is_user_set(OPT_MODELINE_SET));
    }

    #[test]
    fn test_is_modeline_set() {
        assert!(is_modeline_set(OPT_MODELINE_SET));
        assert!(!is_modeline_set(OPT_USER_SET));
    }

    #[test]
    fn test_is_script_set() {
        assert!(is_script_set(OPT_SCRIPT_SET));
        assert!(!is_script_set(OPT_USER_SET));
    }

    #[test]
    fn test_is_factory_default() {
        assert!(is_factory_default(OPT_FACTORY));
        assert!(!is_factory_default(OPT_FACTORY | OPT_CHANGED));
        assert!(!is_factory_default(0));
    }

    #[test]
    fn test_default_source_priority() {
        assert_eq!(default_source_priority(DEFAULT_BUILTIN), 0);
        assert_eq!(default_source_priority(DEFAULT_SYSTEM), 1);
        assert_eq!(default_source_priority(DEFAULT_ENV), 2);
        assert_eq!(default_source_priority(DEFAULT_VIMRC), 3);
        assert_eq!(default_source_priority(DEFAULT_MODELINE), 4);
        assert_eq!(default_source_priority(99), -1);
    }

    #[test]
    fn test_source_has_higher_priority() {
        // modeline > vimrc > env > system > builtin
        assert!(source_has_higher_priority(DEFAULT_MODELINE, DEFAULT_VIMRC));
        assert!(source_has_higher_priority(DEFAULT_VIMRC, DEFAULT_ENV));
        assert!(source_has_higher_priority(DEFAULT_ENV, DEFAULT_SYSTEM));
        assert!(source_has_higher_priority(DEFAULT_SYSTEM, DEFAULT_BUILTIN));

        assert!(!source_has_higher_priority(
            DEFAULT_BUILTIN,
            DEFAULT_MODELINE
        ));
        assert!(!source_has_higher_priority(DEFAULT_SYSTEM, DEFAULT_ENV));
    }
}
