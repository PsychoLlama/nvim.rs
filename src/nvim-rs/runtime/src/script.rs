//! Script item management
//!
//! This module handles scriptitem_T tracking for sourced scripts.
//! Phase 2 of the runtime.c migration: script registry functions.

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::constants::{IOSIZE, MAXPATHL};
use crate::globals::{self, ScriptitemT};
use crate::{LinenrT, ScidT, ScriptItemHandle};
use nvim_ex_eval::ExargT;

// =============================================================================
// SID Constants (verified by _Static_assert in runtime_ffi.c)
// =============================================================================

const SID_LUA: ScidT = -8;

// =============================================================================
// C Accessor Extern Declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Allocate script-local variables for a script.
    #[link_name = "new_script_vars"]
    fn nvim_new_script_vars(sid: c_int);

    // path comparison
    #[link_name = "path_fnamecmp"]
    fn nvim_rt_path_fnamecmp(a: *const c_char, b: *const c_char) -> c_int;

    // get_scriptname delegate
    fn nvim_rt_get_scriptname(sc_sid: c_int, sc_chan: u64, should_free: *mut bool) -> *mut c_char;

    // (exarg_T fields accessed directly via nvim_ex_eval::ExargT)
    #[link_name = "expand_env"]
    fn nvim_rt_expand_env(src: *mut c_char, dst: *mut c_char, dstlen: c_int);
    #[link_name = "do_exedit"]
    fn do_exedit_c(eap: *mut c_void, old_curwin: *mut c_void);
    // emsg/gettext for e_invarg (inline)
    fn emsg(s: *const c_char);
    fn gettext(msgid: *const c_char) -> *const c_char;
    // NameBuff and IObuff are declared in statics block below
    fn nvim_rt_home_replace(name: *const c_char, buf: *mut c_char, len: usize);
    // format_script_entry implemented inline in Rust
    #[link_name = "message_filtered"]
    fn nvim_rt_message_filtered(msg: *const c_char) -> bool;
    #[link_name = "msg_putchar"]
    fn nvim_rt_msg_putchar(c: c_int);
    #[link_name = "msg_outtrans"]
    fn nvim_rt_msg_outtrans(msg: *const c_char, hl_id: c_int, hist: bool);
    #[link_name = "line_breakcheck"]
    fn nvim_rt_line_breakcheck();

    // free helpers (in runtime.c, not runtime_ffi.c)
    fn nvim_rt_free_scriptnames();
    fn nvim_rt_ga_clear_loaded();

    // get_sourced_lnum delegate
    fn nvim_rt_get_sourced_lnum(fgetline: *mut c_void, cookie: *mut c_void) -> LinenrT;

    // get_script_local_funcs delegate
    fn nvim_rt_get_script_local_funcs(sid: c_int) -> *mut c_void;

    // f_getscriptinfo helpers
    #[link_name = "tv_list_alloc_ret"]
    fn nvim_rt_list_alloc_ret(rettv: *mut c_void, count: c_int);
    #[link_name = "tv_check_for_opt_dict_arg"]
    fn nvim_rt_check_for_opt_dict_arg(argvars: *const c_void, idx: c_int) -> c_int;
    fn nvim_rt_get_rettv_list(rettv: *mut c_void) -> *mut c_void;
    fn nvim_rt_argvars_is_dict(argvars: *mut c_void) -> bool;
    fn nvim_rt_dict_find_sid(argvars: *mut c_void) -> i64;
    fn nvim_rt_dict_get_name_pat(argvars: *mut c_void) -> *mut c_char;
    fn nvim_rt_vim_regcomp(pat: *const c_char) -> *mut c_void;
    fn nvim_rt_vim_regexec(regmatch: *mut c_void, str: *const c_char) -> bool;
    fn nvim_rt_vim_regfree(regmatch: *mut c_void);
    #[link_name = "tv_dict_alloc"]
    fn nvim_rt_p2_dict_alloc() -> *mut c_void;
    #[link_name = "tv_dict_add_str"]
    fn nvim_rt_dict_add_str(d: *mut c_void, key: *const c_char, keylen: usize, val: *const c_char);
    #[link_name = "tv_dict_add_nr"]
    fn nvim_rt_dict_add_nr(d: *mut c_void, key: *const c_char, keylen: usize, nr: i64);
    fn nvim_rt_dict_add_bool(d: *mut c_void, key: *const c_char, keylen: usize, val: bool);
    #[link_name = "tv_list_append_dict"]
    fn nvim_rt_p2_tv_list_append_dict(l: *mut c_void, d: *mut c_void);
    fn nvim_rt_copy_script_vars(sid: c_int) -> *mut c_void;
    #[link_name = "tv_dict_add_dict"]
    fn nvim_rt_dict_add_dict(d: *mut c_void, key: *const c_char, keylen: usize, val: *mut c_void);
    #[link_name = "tv_dict_add_list"]
    fn nvim_rt_dict_add_list(d: *mut c_void, key: *const c_char, keylen: usize, val: *mut c_void);

    // xfree
    fn xfree(ptr: *mut c_void);
}

extern "C" {
    /// NameBuff[MAXPATHL] buffer for expanding file names
    #[link_name = "NameBuff"]
    static nvim_rt_namebuff: [c_char; 4096]; // MAXPATHL

    /// IObuff[IOSIZE] buffer for sprintf, I/O
    #[link_name = "IObuff"]
    static nvim_rt_iobuff_arr: [c_char; 1025]; // IOSIZE

    /// e_invarg error string
    #[link_name = "e_invarg"]
    static e_invarg: [c_char; 1];
}

// =============================================================================
// ScriptItemHandle <-> *mut ScriptitemT conversions
// =============================================================================

/// Convert a `*mut ScriptitemT` to a `ScriptItemHandle`.
#[inline]
fn si_handle(si: *mut ScriptitemT) -> ScriptItemHandle {
    ScriptItemHandle(si.cast::<c_void>())
}

/// Convert a `ScriptItemHandle` to a `*mut ScriptitemT`.
#[inline]
fn si_ptr(handle: ScriptItemHandle) -> *mut ScriptitemT {
    handle.as_ptr().cast::<ScriptitemT>()
}

// =============================================================================
// Script Item Access (existing helpers)
// =============================================================================

/// Get a script item by ID.
///
/// Returns null handle if ID is invalid.
pub unsafe fn rs_script_item_get(id: ScidT) -> ScriptItemHandle {
    si_handle(globals::script_item_get(id))
}

/// Get the name of a script item.
pub unsafe fn rs_script_item_name(si: ScriptItemHandle) -> *const c_char {
    if si.is_null() {
        return ptr::null();
    }
    (*si_ptr(si)).sn_name
}

/// Check if a script item is a Lua script.
pub unsafe fn rs_script_item_is_lua(si: ScriptItemHandle) -> bool {
    if si.is_null() {
        return false;
    }
    (*si_ptr(si)).sn_lua
}

/// Check if a script item has profiling enabled.
pub unsafe fn rs_script_item_profiling(si: ScriptItemHandle) -> bool {
    if si.is_null() {
        return false;
    }
    (*si_ptr(si)).sn_prof_on
}

// =============================================================================
// Script ID Utilities (existing helpers)
// =============================================================================

/// Get the total number of sourced scripts.
pub unsafe fn rs_script_count() -> c_int {
    globals::script_items_get_len()
}

/// Check if a script ID is valid.
pub unsafe fn rs_script_id_is_valid(id: ScidT) -> bool {
    id > 0 && id <= globals::script_items_get_len()
}

/// Get the name of a script by ID.
///
/// Returns null if ID is invalid.
pub unsafe fn rs_script_name_by_id(id: ScidT) -> *const c_char {
    let si = rs_script_item_get(id);
    rs_script_item_name(si)
}

// =============================================================================
// Phase 2: Migrated Functions
// =============================================================================

// --- 1. new_script_item ---

/// Create a new script item and allocate script-local vars.
///
/// Maintains a static counter `last_current_SID` to assign unique script IDs.
///
/// # Safety
///
/// `name` may be NULL for anonymous :source.
/// `sid_out` may be NULL if the caller doesn't need the SID.
#[export_name = "new_script_item"]
pub unsafe extern "C" fn rs_new_script_item(
    name: *mut c_char,
    sid_out: *mut ScidT,
) -> ScriptItemHandle {
    // Static counter persists across calls (mirrors C `static scid_T last_current_SID = 0`)
    static mut LAST_CURRENT_SID: ScidT = 0;

    LAST_CURRENT_SID += 1;
    let sid = LAST_CURRENT_SID;

    if !sid_out.is_null() {
        *sid_out = sid;
    }

    // Grow the garray to accommodate the new sid
    let ga_len = globals::script_items_get_len();
    globals::script_items_ga_grow(sid - ga_len);

    // Fill in any gaps between ga_len and sid
    while globals::script_items_get_len() < sid {
        let si = globals::xcalloc_scriptitem();
        globals::script_items_inc_len();
        let current_len = globals::script_items_get_len();
        globals::script_item_set(current_len, si);
        (*si).sn_name = ptr::null_mut();

        // Allocate the local script variables to use for this script.
        nvim_new_script_vars(current_len);

        (*si).sn_prof_on = false;
    }

    let si = globals::script_item_get(sid);
    (*si).sn_name = name;
    si_handle(si)
}

// --- 2. find_script_by_name ---

/// Find an already loaded script by name.
///
/// Returns its script ID if found, -1 if not found.
/// Uses `path_fnamecmp` for platform-correct filename comparison.
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[export_name = "find_script_by_name"]
pub unsafe extern "C" fn rs_find_script_by_name(name: *const c_char) -> c_int {
    if name.is_null() {
        return -1;
    }

    let count = globals::script_items_get_len();
    // Search from the end (most recently loaded scripts first), matching C behavior
    let mut sid = count;
    while sid > 0 {
        let si = globals::script_item_get(sid);
        let si_name = (*si).sn_name.cast_const();

        if !si_name.is_null() && nvim_rt_path_fnamecmp(si_name, name) == 0 {
            return sid;
        }
        sid -= 1;
    }

    -1
}

// --- 3. script_is_lua ---

/// Check if the script with the given script ID is a Lua script.
///
/// Returns true if sid == SID_LUA, or if the script item has sn_lua set.
/// Returns false if the sid is not a valid script ID.
#[export_name = "script_is_lua"]
pub unsafe extern "C" fn rs_script_is_lua(sid: ScidT) -> bool {
    if sid == SID_LUA {
        return true;
    }
    if !rs_script_id_is_valid(sid) {
        return false;
    }
    let si = globals::script_item_get(sid);
    (*si).sn_lua
}

// --- 4. get_scriptname ---

/// Get a pointer to a script name. Used for ":verbose set".
///
/// Delegates entirely to C because the return value may be a static string,
/// IObuff, or an allocated string, which is complex to manage across FFI.
///
/// # Safety
///
/// `should_free` may be NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_get_scriptname(
    sc_sid: c_int,
    sc_chan: u64,
    should_free: *mut bool,
) -> *mut c_char {
    nvim_rt_get_scriptname(sc_sid, sc_chan, should_free)
}

// --- 5. ex_scriptnames ---

/// `:scriptnames` command handler.
///
/// If addr_count > 0 or arg is non-empty, edit the specified script.
/// Otherwise, list all loaded scripts.
#[export_name = "ex_scriptnames"]
pub unsafe extern "C" fn rs_ex_scriptnames(eap: *mut c_void) {
    let eap_ref = &mut *eap.cast::<ExargT>();
    let addr_count = eap_ref.addr_count;
    let arg_is_nul = eap_ref.arg.is_null() || *eap_ref.arg == 0;

    if addr_count > 0 || !arg_is_nul {
        // :script {scriptId}: edit the script
        if addr_count > 0 {
            let line2 = eap_ref.line2;
            if !rs_script_id_is_valid(line2) {
                emsg(gettext(e_invarg.as_ptr()));
                return;
            }
            let si = globals::script_item_get(line2);
            let sn_name = (*si).sn_name.cast_const();
            eap_ref.arg = sn_name.cast_mut();
        } else {
            let arg = eap_ref.arg;
            let namebuff = nvim_rt_namebuff.as_ptr().cast_mut();
            nvim_rt_expand_env(arg, namebuff, MAXPATHL as c_int);
            eap_ref.arg = namebuff;
        }
        do_exedit_c(eap, std::ptr::null_mut());
        return;
    }

    // List all scripts
    let count = globals::script_items_get_len();
    let namebuff = nvim_rt_namebuff.as_ptr().cast_mut();
    let iobuff = nvim_rt_iobuff_arr.as_ptr().cast_mut();

    let mut i: c_int = 1;
    while i <= count && !globals::got_int {
        let si = globals::script_item_get(i);
        let sn_name = (*si).sn_name.cast_const();

        if !sn_name.is_null() {
            nvim_rt_home_replace(sn_name, namebuff, MAXPATHL);
            // format_script_entry: "%3d: %s" into IObuff
            {
                use std::io::Write as _;
                let nb = std::ffi::CStr::from_ptr(namebuff.cast_const()).to_bytes();
                let buf_slice = std::slice::from_raw_parts_mut(
                    nvim_rt_iobuff_arr.as_ptr().cast_mut().cast::<u8>(),
                    IOSIZE,
                );
                let mut cur = std::io::Cursor::new(&mut *buf_slice);
                let _ = write!(cur, "{i:3}: {}\0", std::str::from_utf8_unchecked(nb));
            }
            if !nvim_rt_message_filtered(iobuff) {
                nvim_rt_msg_putchar(c_int::from(b'\n'));
                nvim_rt_msg_outtrans(iobuff, 0, false);
                nvim_rt_line_breakcheck();
            }
        }
        i += 1;
    }
}

// --- 6. free_scriptnames (EXITFREE only) ---

/// Free all script names and associated data.
///
/// Delegates entirely to C since it uses EXITFREE macros and complex cleanup.
#[export_name = "free_scriptnames"]
pub unsafe extern "C" fn rs_free_scriptnames() {
    nvim_rt_free_scriptnames();
}

// --- 7. free_autoload_scriptnames ---

/// Free the autoload script names array.
///
/// Delegates to C since ga_loaded is static in runtime.c.
#[export_name = "free_autoload_scriptnames"]
pub unsafe extern "C" fn rs_free_autoload_scriptnames() {
    nvim_rt_ga_clear_loaded();
}

// --- 8. get_sourced_lnum ---

/// Get the sourced line number.
///
/// Delegates entirely to C since comparing function pointers across FFI is
/// unreliable.
#[unsafe(export_name = "get_sourced_lnum")]
pub unsafe extern "C" fn rs_get_sourced_lnum(
    fgetline: *mut c_void,
    cookie: *mut c_void,
) -> LinenrT {
    nvim_rt_get_sourced_lnum(fgetline, cookie)
}

// --- 9. get_script_local_funcs (static) ---

/// Return a List of script-local functions defined in the script with id `sid`.
///
/// Delegates to C since it needs hashtab iteration with HASHTAB_ITER macro.
#[no_mangle]
pub unsafe extern "C" fn rs_get_script_local_funcs(sid: ScidT) -> *mut c_void {
    nvim_rt_get_script_local_funcs(sid)
}

// --- 10. f_getscriptinfo ---

/// `getscriptinfo()` VimL function.
///
/// Returns a List of Dicts with script information.
#[no_mangle]
pub unsafe extern "C" fn rs_f_getscriptinfo(
    argvars: *mut c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let count = globals::script_items_get_len();
    nvim_rt_list_alloc_ret(rettv, count);

    // FAIL == 0, OK == 1
    if nvim_rt_check_for_opt_dict_arg(argvars.cast_const(), 0) == 0 {
        return;
    }

    let l = nvim_rt_get_rettv_list(rettv);

    let mut regmatch: *mut c_void = ptr::null_mut();
    let mut filterpat = false;
    let mut sid: i64 = -1;
    let mut pat: *mut c_char = ptr::null_mut();

    if nvim_rt_argvars_is_dict(argvars) {
        let found_sid = nvim_rt_dict_find_sid(argvars);
        if found_sid == -2 {
            // error in type conversion
            return;
        } else if found_sid == -3 {
            // invalid sid value (error already emitted)
            return;
        } else if found_sid >= 1 {
            sid = found_sid;
        } else {
            // sid not in dict, try "name" pattern
            pat = nvim_rt_dict_get_name_pat(argvars);
            if !pat.is_null() {
                regmatch = nvim_rt_vim_regcomp(pat);
            }
            if !regmatch.is_null() {
                filterpat = true;
            }
        }
    }

    let start = if sid > 0 { sid as c_int } else { 1 };

    let mut i = start;
    while (i == sid as c_int || sid <= 0) && i <= count {
        let si = globals::script_item_get(i);
        let sn_name = (*si).sn_name.cast_const();

        if sn_name.is_null() {
            i += 1;
            continue;
        }

        if filterpat && !nvim_rt_vim_regexec(regmatch, sn_name) {
            i += 1;
            continue;
        }

        let d = nvim_rt_p2_dict_alloc();
        nvim_rt_p2_tv_list_append_dict(l, d);
        nvim_rt_dict_add_str(d, b"name\0".as_ptr().cast(), 4, sn_name);
        nvim_rt_dict_add_nr(d, b"sid\0".as_ptr().cast(), 3, i64::from(i));
        nvim_rt_dict_add_nr(d, b"version\0".as_ptr().cast(), 7, 1);
        nvim_rt_dict_add_bool(d, b"autoload\0".as_ptr().cast(), 8, false);

        // When a script ID is specified, return information about only the
        // specified script, and add the script-local variables and functions.
        if sid > 0 {
            let var_dict = nvim_rt_copy_script_vars(i);
            nvim_rt_dict_add_dict(d, b"variables\0".as_ptr().cast(), 9, var_dict);
            let funcs = nvim_rt_get_script_local_funcs(sid as ScidT);
            nvim_rt_dict_add_list(d, b"functions\0".as_ptr().cast(), 9, funcs);
        }

        i += 1;
    }

    nvim_rt_vim_regfree(regmatch);
    xfree(pat.cast());
}

// --- 11. scriptnames_slash_adjust ---

/// Fix slashes in script names for 'shellslash' (Windows only).
///
/// On non-Windows platforms, this is a no-op since BACKSLASH_IN_FILENAME
/// is not defined.
#[export_name = "scriptnames_slash_adjust"]
pub unsafe extern "C" fn rs_scriptnames_slash_adjust() {
    // This is a no-op on non-Windows. On Windows, we would iterate and call
    // nvim_rt_slash_adjust. Since Neovim targets Linux/macOS primarily and
    // BACKSLASH_IN_FILENAME is not defined, this function body is empty.
    // The C wrapper is guarded by #if defined(BACKSLASH_IN_FILENAME).
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_null() {
        assert!(ScriptItemHandle::null().is_null());
    }

    #[test]
    fn test_sid_lua_constant() {
        assert_eq!(SID_LUA, -8);
    }
}
