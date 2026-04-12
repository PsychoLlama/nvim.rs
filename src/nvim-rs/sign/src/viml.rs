//! VimL function support for signs
//!
//! This module provides structures and helpers for the VimL sign_*() functions:
//! - sign_define()
//! - sign_getdefined()
//! - sign_getplaced()
//! - sign_jump()
//! - sign_place()
//! - sign_placelist()
//! - sign_undefine()
//! - sign_unplace()
//! - sign_unplacelist()

#![allow(
    clippy::manual_c_str_literals,
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::if_not_else
)]

use std::ffi::{c_char, c_int, c_void};

use crate::{LinenrT, MTKeyHandle, SignBufHandle, SignHandle, SIGN_DEF_PRIO};

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    /// Get sign by name from the sign map
    fn nvim_sign_map_get(name: *const c_char) -> SignHandle;
}

// =============================================================================
// VimL Function Result Types
// =============================================================================

/// Result from sign_define() function.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignDefineResult {
    /// Success (returns 0)
    Success = 0,
    /// Invalid argument
    InvalidArg = -1,
    /// Sign already exists (update may have occurred)
    Updated = 1,
}

/// Result from sign_place() function.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignPlaceResultCode {
    /// Success - returns sign ID
    Success = 0,
    /// Invalid argument
    InvalidArg = -1,
    /// Sign not defined
    NotDefined = -2,
    /// Buffer not found
    NoBuffer = -3,
    /// Invalid line number
    InvalidLine = -4,
}

/// Result from sign_unplace() function.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignUnplaceResult {
    /// Success
    Success = 0,
    /// Sign not found
    NotFound = -1,
    /// Invalid argument
    InvalidArg = -2,
}

// =============================================================================
// sign_define() Support
// =============================================================================

/// Parameters for sign_define() VimL function.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SignDefineVimlParams {
    /// Sign name (required)
    pub name: *const c_char,
    /// Icon path (from 'icon' key)
    pub icon: *const c_char,
    /// Sign text (from 'text' key)
    pub text: *const c_char,
    /// Line highlight group (from 'linehl' key)
    pub linehl: *const c_char,
    /// Text highlight group (from 'texthl' key)
    pub texthl: *const c_char,
    /// Cursorline highlight (from 'culhl' key)
    pub culhl: *const c_char,
    /// Number column highlight (from 'numhl' key)
    pub numhl: *const c_char,
    /// Priority (from 'priority' key, -1 for unset)
    pub priority: c_int,
}

impl Default for SignDefineVimlParams {
    fn default() -> Self {
        Self {
            name: std::ptr::null(),
            icon: std::ptr::null(),
            text: std::ptr::null(),
            linehl: std::ptr::null(),
            texthl: std::ptr::null(),
            culhl: std::ptr::null(),
            numhl: std::ptr::null(),
            priority: -1,
        }
    }
}

/// Create default sign_define params.
#[no_mangle]
pub extern "C" fn rs_sign_define_viml_params_default() -> SignDefineVimlParams {
    SignDefineVimlParams::default()
}

/// Check if sign_define params are valid.
///
/// # Safety
/// All string pointers must be null or valid C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_define_viml_params_valid(
    params: *const SignDefineVimlParams,
) -> bool {
    if params.is_null() {
        return false;
    }
    let p = &*params;
    // Name is required and must not be empty
    !p.name.is_null() && *p.name.cast::<u8>() != 0
}

// =============================================================================
// sign_place() Support
// =============================================================================

/// Parameters for sign_place() VimL function.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SignPlaceVimlParams {
    /// Sign ID (0 for auto-generate)
    pub id: c_int,
    /// Sign group (null for global)
    pub group: *const c_char,
    /// Sign name
    pub name: *const c_char,
    /// Buffer handle
    pub buf: SignBufHandle,
    /// Line number (from 'lnum' key)
    pub lnum: LinenrT,
    /// Priority (from 'priority' key, -1 for default)
    pub priority: c_int,
}

impl Default for SignPlaceVimlParams {
    fn default() -> Self {
        Self {
            id: 0,
            group: std::ptr::null(),
            name: std::ptr::null(),
            buf: SignBufHandle::null(),
            lnum: 0,
            priority: -1,
        }
    }
}

/// Create default sign_place params.
#[no_mangle]
pub extern "C" fn rs_sign_place_viml_params_default() -> SignPlaceVimlParams {
    SignPlaceVimlParams::default()
}

/// Validate sign_place params.
///
/// Returns 0 if valid, or a negative error code.
///
/// # Safety
/// All pointers must be null or valid.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_place_viml_validate(
    params: *const SignPlaceVimlParams,
) -> SignPlaceResultCode {
    if params.is_null() {
        return SignPlaceResultCode::InvalidArg;
    }

    let p = &*params;

    // ID must be >= 0
    if p.id < 0 {
        return SignPlaceResultCode::InvalidArg;
    }

    // Name is required
    if p.name.is_null() || *p.name.cast::<u8>() == 0 {
        return SignPlaceResultCode::InvalidArg;
    }

    // Sign must be defined
    let sp = nvim_sign_map_get(p.name);
    if sp.is_null() {
        return SignPlaceResultCode::NotDefined;
    }

    // Buffer is required
    if p.buf.is_null() {
        return SignPlaceResultCode::NoBuffer;
    }

    // Line number must be valid for new placements
    if p.id == 0 && p.lnum <= 0 {
        return SignPlaceResultCode::InvalidLine;
    }

    SignPlaceResultCode::Success
}

/// Get effective priority for sign_place.
#[no_mangle]
pub extern "C" fn rs_sign_place_viml_priority(prio: c_int, def_prio: c_int) -> c_int {
    if prio >= 0 {
        prio
    } else if def_prio >= 0 {
        def_prio
    } else {
        SIGN_DEF_PRIO
    }
}

// =============================================================================
// sign_getplaced() Support
// =============================================================================

/// Filter parameters for sign_getplaced() function.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SignGetPlacedFilter {
    /// Buffer to query (null for all buffers)
    pub buf: SignBufHandle,
    /// Line number filter (0 for all lines)
    pub lnum: LinenrT,
    /// Sign ID filter (0 for all signs)
    pub id: c_int,
    /// Group filter (null for global, "*" for all)
    pub group: *const c_char,
}

impl Default for SignGetPlacedFilter {
    fn default() -> Self {
        Self {
            buf: SignBufHandle::null(),
            lnum: 0,
            id: 0,
            group: std::ptr::null(),
        }
    }
}

/// Create default sign_getplaced filter.
#[no_mangle]
pub extern "C" fn rs_sign_get_placed_filter_default() -> SignGetPlacedFilter {
    SignGetPlacedFilter::default()
}

/// Check if filter specifies all signs (no restrictions).
#[no_mangle]
pub extern "C" fn rs_sign_get_placed_filter_is_all(filter: &SignGetPlacedFilter) -> bool {
    filter.buf.is_null() && filter.lnum == 0 && filter.id == 0 && filter.group.is_null()
}

/// Check if filter restricts to specific buffer.
#[no_mangle]
pub extern "C" fn rs_sign_get_placed_filter_has_buf(filter: &SignGetPlacedFilter) -> bool {
    !filter.buf.is_null()
}

// =============================================================================
// sign_unplace() Support
// =============================================================================

/// Parameters for sign_unplace() VimL function.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SignUnplaceVimlParams {
    /// Sign group ("*" for all groups)
    pub group: *const c_char,
    /// Buffer handle (null for all buffers)
    pub buf: SignBufHandle,
    /// Sign ID (0 for all)
    pub id: c_int,
}

impl Default for SignUnplaceVimlParams {
    fn default() -> Self {
        Self {
            group: std::ptr::null(),
            buf: SignBufHandle::null(),
            id: 0,
        }
    }
}

/// Create default sign_unplace params.
#[no_mangle]
pub extern "C" fn rs_sign_unplace_viml_params_default() -> SignUnplaceVimlParams {
    SignUnplaceVimlParams::default()
}

/// Check if unplace params specify "all" groups ("*").
///
/// # Safety
/// `group` must be null or a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_unplace_viml_is_all_groups(
    params: *const SignUnplaceVimlParams,
) -> bool {
    if params.is_null() {
        return false;
    }
    let p = &*params;
    !p.group.is_null() && *p.group.cast::<u8>() == b'*'
}

// =============================================================================
// sign_jump() Support
// =============================================================================

/// Parameters for sign_jump() VimL function.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SignJumpVimlParams {
    /// Sign ID (required, must be > 0)
    pub id: c_int,
    /// Sign group (empty string means global)
    pub group: *const c_char,
    /// Buffer to search in
    pub buf: SignBufHandle,
}

impl Default for SignJumpVimlParams {
    fn default() -> Self {
        Self {
            id: 0,
            group: std::ptr::null(),
            buf: SignBufHandle::null(),
        }
    }
}

/// Create default sign_jump params.
#[no_mangle]
pub extern "C" fn rs_sign_jump_viml_params_default() -> SignJumpVimlParams {
    SignJumpVimlParams::default()
}

/// Validate sign_jump params.
///
/// # Safety
/// All pointers must be null or valid.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_jump_viml_validate(params: *const SignJumpVimlParams) -> bool {
    if params.is_null() {
        return false;
    }
    let p = &*params;

    // ID must be > 0
    if p.id <= 0 {
        return false;
    }

    // Buffer is required
    if p.buf.is_null() {
        return false;
    }

    true
}

// =============================================================================
// C Accessor Extern Declarations (Phase 2)
// =============================================================================

// VAR_* type constants from typval_defs.h
const VAR_UNKNOWN: c_int = 0;
const VAR_LIST: c_int = 4;
const VAR_DICT: c_int = 5;

// kListLen* constants (from typval_defs.h)
const K_LIST_LEN_MAY_KNOW: i64 = -1;

extern "C" {
    // Dict/list construction
    fn tv_dict_alloc() -> *mut c_void;
    fn tv_dict_add_str(d: *mut c_void, key: *const c_char, key_len: usize, val: *const c_char);
    fn tv_dict_add_nr(d: *mut c_void, key: *const c_char, key_len: usize, nr: i64);
    fn tv_dict_find(d: *const c_void, key: *const c_char, key_len: i64) -> *mut c_void;
    fn tv_dict_get_string(d: *const c_void, key: *const c_char, save: bool) -> *mut c_char;
    fn tv_dict_get_number(d: *const c_void, key: *const c_char) -> i64;
    fn tv_dict_get_number_def(d: *const c_void, key: *const c_char, def: c_int) -> i64;
    fn tv_list_alloc_ret(rettv: *mut c_void, len: i64);
    fn tv_list_append_dict(l: *mut c_void, d: *mut c_void);
    fn tv_list_append_number(l: *mut c_void, n: i64);
    fn tv_get_number_chk(tv: *const c_void, error: *mut bool) -> i64;
    fn tv_get_string_chk(tv: *const c_void) -> *const c_char;
    fn tv_get_string(tv: *const c_void) -> *const c_char;
    fn tv_get_lnum(tv: *const c_void) -> LinenrT;
    fn tv_check_for_opt_dict_arg(argvars: *const c_void, idx: c_int) -> c_int;
    fn tv_check_for_nonnull_dict_arg(argvars: *const c_void, idx: c_int) -> c_int;
    fn tv_check_for_string_arg(argvars: *const c_void, idx: c_int) -> c_int;
    fn get_buf_arg(tv: *mut c_void) -> SignBufHandle;
    fn nvim_di_get_tv(di: *mut c_void) -> *mut c_void; // dictitem_T* → typval_T*

    // Typval field accessors (in sign.c)
    fn nvim_tv_get_type(tv: *const c_void) -> c_int;
    fn nvim_tv_get_dict(tv: *const c_void) -> *mut c_void;
    fn nvim_tv_get_list(tv: *const c_void) -> *mut c_void;
    fn nvim_rettv_set_number(rettv: *mut c_void, num: i64);
    fn nvim_rettv_get_list(rettv: *mut c_void) -> *mut c_void;
    fn nvim_argvars_at(argvars: *mut c_void, idx: c_int) -> *mut c_void;

    // Sign map iteration
    fn nvim_sign_map_get_nth_value(idx: c_int) -> SignHandle;
    fn nvim_sign_map_get_nth_key(idx: c_int) -> *mut c_char;
    fn nvim_sign_map_size() -> c_int;

    // TV list length + indexed access (in sign.c)
    fn nvim_tv_list_len(l: *const c_void) -> c_int;
    fn nvim_tv_list_item_tv_at(l: *mut c_void, idx: c_int) -> *mut c_void;

    // Buffer iteration
    fn nvim_get_firstbuf() -> SignBufHandle;
    fn nvim_buf_get_next(buf: SignBufHandle) -> SignBufHandle;
    fn rs_sign_buffer_has_signs(buf: SignBufHandle) -> bool;

    // Highlight name lookup
    fn get_highlight_name_ext(xp: *mut c_void, idx: c_int, skip_cleared: bool) -> *const c_char;

    // Sign operations (other Rust crate exports)
    fn rs_sign_place(
        id: *mut u32,
        group: *const c_char,
        name: *const c_char,
        buf: SignBufHandle,
        lnum: LinenrT,
        prio: c_int,
    ) -> c_int;
    fn rs_sign_unplace(
        buf: SignBufHandle,
        id: c_int,
        group: *const c_char,
        atlnum: LinenrT,
    ) -> c_int;
    fn rs_sign_jump(id: c_int, group: *const c_char, buf: SignBufHandle) -> LinenrT;

    // Error/message functions
    fn emsg(msg: *const c_char) -> c_int;

    // Complex dict/list ops that stay in C for now
    fn nvim_sign_get_placed_info_dict_impl(mark: MTKeyHandle) -> *mut c_void;
    fn nvim_get_buffer_signs_impl(buf: SignBufHandle) -> *mut c_void;
}

// =============================================================================
// Phase 2: Migrated VimL Function Implementations
// =============================================================================

// Error strings (mirrors C e_* constants with gettext)
static E_INVARG: &[u8] = b"E474: Invalid argument\0";
static E_LISTREQ: &[u8] = b"E714: List required\0";
static E_DICTREQ: &[u8] = b"E715: Dictionary required\0";

const OK: c_int = 1;
const FAIL: c_int = 0;

/// Helper: dict_T as *mut c_void (typval dict returns)
#[inline]
unsafe fn alloc_info_dict(sp: SignHandle) -> *mut c_void {
    use crate::text::describe_sign_text_impl;
    if sp.is_null() {
        return std::ptr::null_mut();
    }
    let s = &*sp;
    let d = tv_dict_alloc();
    // name
    tv_dict_add_str(d, b"name\0".as_ptr().cast(), 4, s.sn_name);
    // icon (optional)
    if !s.sn_icon.is_null() {
        tv_dict_add_str(d, b"icon\0".as_ptr().cast(), 4, s.sn_icon);
    }
    // text (if set)
    if s.sn_text[0] != 0 {
        let mut buf = [0u8; crate::SIGN_WIDTH * crate::text::MAX_SCHAR_SIZE];
        describe_sign_text_impl(&mut buf, &s.sn_text);
        tv_dict_add_str(d, b"text\0".as_ptr().cast(), 4, buf.as_ptr().cast());
    }
    // priority (if > 0)
    if s.sn_priority > 0 {
        tv_dict_add_nr(
            d,
            b"priority\0".as_ptr().cast(),
            8,
            i64::from(s.sn_priority),
        );
    }
    // highlight groups
    let hl_keys: [&[u8]; 4] = [b"linehl\0", b"texthl\0", b"culhl\0", b"numhl\0"];
    let hl_ids = [s.sn_line_hl, s.sn_text_hl, s.sn_cul_hl, s.sn_num_hl];
    for (key, &hl_id) in hl_keys.iter().zip(hl_ids.iter()) {
        if hl_id > 0 {
            let p = get_highlight_name_ext(std::ptr::null_mut(), hl_id - 1, false);
            let name_ptr = if p.is_null() {
                b"NONE\0".as_ptr().cast()
            } else {
                p
            };
            tv_dict_add_str(d, key.as_ptr().cast(), key.len() - 1, name_ptr);
        }
    }
    d
}

/// Get sign info as a dictionary (migrated from C nvim_sign_get_info_dict_impl).
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[unsafe(export_name = "sign_get_info_dict")]
pub unsafe extern "C" fn rs_sign_get_info_dict(sp: SignHandle) -> *mut c_void {
    if sp.is_null() {
        return std::ptr::null_mut();
    }
    alloc_info_dict(sp)
}

/// Get placed sign info as a dictionary.
///
/// Delegates to C composite accessor for dict construction from MTKey.
///
/// # Safety
///
/// `mark_ptr` must be a valid pointer to an MTKey.
#[unsafe(export_name = "sign_get_placed_info_dict")]
pub unsafe extern "C" fn rs_sign_get_placed_info_dict(mark_ptr: MTKeyHandle) -> *mut c_void {
    if mark_ptr.is_null() {
        return std::ptr::null_mut();
    }
    nvim_sign_get_placed_info_dict_impl(mark_ptr)
}

/// Get all signs placed in a buffer as a list.
///
/// Delegates to C composite accessor for list construction.
///
/// # Safety
///
/// `buf` must be a valid buffer handle.
#[unsafe(export_name = "get_buffer_signs")]
pub unsafe extern "C" fn rs_get_buffer_signs(buf: SignBufHandle) -> *mut c_void {
    if buf.is_null() {
        return std::ptr::null_mut();
    }
    nvim_get_buffer_signs_impl(buf)
}

/// Get placed signs in a buffer, filtered by parameters.
///
/// Delegates to C composite accessor for marktree iteration and list building.
///
/// # Safety
///
/// `buf` must be a valid buffer handle. `retlist` must be a valid list handle.
/// `group` must be null or a valid C string.
#[unsafe(export_name = "sign_get_placed_in_buf")]
pub unsafe extern "C" fn rs_sign_get_placed_in_buf(
    buf: SignBufHandle,
    lnum: LinenrT,
    sign_id: c_int,
    group: *const c_char,
    retlist: *mut c_void,
) {
    if buf.is_null() || retlist.is_null() {
        return;
    }
    crate::query::rs_nvim_sign_get_placed_in_buf_impl(buf, lnum, sign_id, group, retlist);
}

/// Get placed signs across buffers.
///
/// If `buf` is non-null, gets signs for that buffer only.
/// Otherwise gets signs for all buffers that have signs.
///
/// # Safety
///
/// `buf` must be a valid buffer handle or null. `retlist` must be a valid list.
/// `group` must be null or a valid C string.
#[unsafe(export_name = "sign_get_placed")]
pub unsafe extern "C" fn rs_sign_get_placed(
    buf: SignBufHandle,
    lnum: LinenrT,
    id: c_int,
    group: *const c_char,
    retlist: *mut c_void,
) {
    if retlist.is_null() {
        return;
    }
    if buf.is_null() {
        let mut cbuf = nvim_get_firstbuf();
        while !cbuf.is_null() {
            if rs_sign_buffer_has_signs(cbuf) {
                crate::query::rs_nvim_sign_get_placed_in_buf_impl(cbuf, 0, id, group, retlist);
            }
            cbuf = nvim_buf_get_next(cbuf);
        }
    } else {
        crate::query::rs_nvim_sign_get_placed_in_buf_impl(buf, lnum, id, group, retlist);
    }
}

// =============================================================================
// Helper: define_from_dict — extract sign params from a VimL dict
// =============================================================================

/// Define a sign from a VimL dictionary.
/// Returns 0 on success, -1 on error (matches VimL convention).
///
/// # Safety
/// `name` must be null or valid C string. `dict` must be a valid dict handle or null.
#[unsafe(export_name = "sign_define_from_dict")]
pub unsafe extern "C" fn rs_sign_define_from_dict(name: *mut c_char, dict: *mut c_void) -> c_int {
    let mut effective_name = name;
    if effective_name.is_null() {
        // Get name from dict
        if dict.is_null() {
            return -1;
        }
        effective_name = tv_dict_get_string(dict, b"name\0".as_ptr().cast(), false);
        if effective_name.is_null() || *effective_name == 0 {
            return -1;
        }
    }
    // Extract optional fields from dict
    let icon = if dict.is_null() {
        std::ptr::null_mut()
    } else {
        tv_dict_get_string(dict, b"icon\0".as_ptr().cast(), false)
    };
    let linehl = if dict.is_null() {
        std::ptr::null_mut()
    } else {
        tv_dict_get_string(dict, b"linehl\0".as_ptr().cast(), false)
    };
    let text = if dict.is_null() {
        std::ptr::null_mut()
    } else {
        tv_dict_get_string(dict, b"text\0".as_ptr().cast(), false)
    };
    let texthl = if dict.is_null() {
        std::ptr::null_mut()
    } else {
        tv_dict_get_string(dict, b"texthl\0".as_ptr().cast(), false)
    };
    let culhl = if dict.is_null() {
        std::ptr::null_mut()
    } else {
        tv_dict_get_string(dict, b"culhl\0".as_ptr().cast(), false)
    };
    let numhl = if dict.is_null() {
        std::ptr::null_mut()
    } else {
        tv_dict_get_string(dict, b"numhl\0".as_ptr().cast(), false)
    };
    #[allow(clippy::cast_possible_truncation)]
    let prio: c_int = if dict.is_null() {
        -1
    } else {
        tv_dict_get_number_def(dict, b"priority\0".as_ptr().cast(), -1) as c_int
    };
    // Call Rust define (returns OK=1 or FAIL=0), convert to 0/-1 for VimL
    let result = crate::define::rs_sign_define_by_name(
        effective_name,
        icon,
        text,
        linehl,
        texthl,
        culhl,
        numhl,
        prio,
    );
    if result == OK {
        0
    } else {
        -1
    }
}

/// Define multiple signs from a VimL list (migrated from C nvim_sign_define_multiple_impl).
///
/// # Safety
/// `l` and `retlist` must be valid list handles.
#[unsafe(export_name = "sign_define_multiple")]
pub unsafe extern "C" fn rs_sign_define_multiple(l: *mut c_void, retlist: *mut c_void) {
    if l.is_null() || retlist.is_null() {
        return;
    }
    let len = nvim_tv_list_len(l);
    for i in 0..len {
        let item_tv = nvim_tv_list_item_tv_at(l, i);
        let retval = if !item_tv.is_null() && nvim_tv_get_type(item_tv) == VAR_DICT {
            rs_sign_define_from_dict(std::ptr::null_mut(), nvim_tv_get_dict(item_tv))
        } else {
            emsg(E_DICTREQ.as_ptr().cast());
            -1
        };
        tv_list_append_number(retlist, i64::from(retval));
    }
}

/// Place a sign from VimL typval parameters (migrated from C nvim_sign_place_from_dict_impl).
///
/// All of `id_tv`, `group_tv`, `name_tv`, `buf_tv` may be null — in that case the
/// corresponding field is looked up from `dict`.
///
/// # Safety
/// All pointer arguments must be valid or null.
#[unsafe(export_name = "sign_place_from_dict")]
pub unsafe extern "C" fn rs_sign_place_from_dict(
    id_tv: *mut c_void,
    group_tv: *mut c_void,
    name_tv: *mut c_void,
    buf_tv: *mut c_void,
    dict: *mut c_void,
) -> c_int {
    // ---- id ----
    let mut id: c_int = 0;
    let mut notanum = false;
    let effective_id_tv = if !id_tv.is_null() {
        id_tv
    } else if !dict.is_null() {
        let di = tv_dict_find(dict, b"id\0".as_ptr().cast(), -1);
        if di.is_null() {
            std::ptr::null_mut()
        } else {
            nvim_di_get_tv(di)
        }
    } else {
        std::ptr::null_mut()
    };
    if !effective_id_tv.is_null() {
        id = tv_get_number_chk(effective_id_tv, std::ptr::addr_of_mut!(notanum)) as c_int;
        if notanum {
            return -1;
        }
        if id < 0 {
            emsg(E_INVARG.as_ptr().cast());
            return -1;
        }
    }

    // ---- group ----
    let mut group: *const c_char = std::ptr::null();
    let effective_group_tv = if !group_tv.is_null() {
        group_tv
    } else if !dict.is_null() {
        let di = tv_dict_find(dict, b"group\0".as_ptr().cast(), -1);
        if di.is_null() {
            std::ptr::null_mut()
        } else {
            nvim_di_get_tv(di)
        }
    } else {
        std::ptr::null_mut()
    };
    if !effective_group_tv.is_null() {
        let g = tv_get_string_chk(effective_group_tv);
        if g.is_null() {
            return -1;
        }
        group = if *g == 0 { std::ptr::null() } else { g };
    }

    // ---- name ----
    let effective_name_tv = if !name_tv.is_null() {
        name_tv
    } else if !dict.is_null() {
        let di = tv_dict_find(dict, b"name\0".as_ptr().cast(), -1);
        if di.is_null() {
            std::ptr::null_mut()
        } else {
            nvim_di_get_tv(di)
        }
    } else {
        std::ptr::null_mut()
    };
    if effective_name_tv.is_null() {
        return -1;
    }
    let name = tv_get_string_chk(effective_name_tv);
    if name.is_null() {
        return -1;
    }

    // ---- buf ----
    let effective_buf_tv = if !buf_tv.is_null() {
        buf_tv
    } else if !dict.is_null() {
        let di = tv_dict_find(dict, b"buffer\0".as_ptr().cast(), -1);
        if di.is_null() {
            std::ptr::null_mut()
        } else {
            nvim_di_get_tv(di)
        }
    } else {
        std::ptr::null_mut()
    };
    if effective_buf_tv.is_null() {
        return -1;
    }
    let buf = get_buf_arg(effective_buf_tv);
    if buf.is_null() {
        return -1;
    }

    // ---- lnum ----
    let mut lnum: LinenrT = 0;
    if !dict.is_null() {
        let di = tv_dict_find(dict, b"lnum\0".as_ptr().cast(), -1);
        if !di.is_null() {
            let dtv = nvim_di_get_tv(di);
            lnum = tv_get_lnum(dtv);
            if lnum <= 0 {
                emsg(E_INVARG.as_ptr().cast());
                return -1;
            }
        }
    }

    // ---- priority ----
    let prio: c_int = if !dict.is_null() {
        let di = tv_dict_find(dict, b"priority\0".as_ptr().cast(), -1);
        if !di.is_null() {
            let dtv = nvim_di_get_tv(di);
            let p = tv_get_number_chk(dtv, std::ptr::addr_of_mut!(notanum)) as c_int;
            if notanum {
                return -1;
            }
            p
        } else {
            -1
        }
    } else {
        -1
    };

    // ---- place ----
    #[allow(clippy::cast_sign_loss)]
    let mut uid = id as u32;
    if rs_sign_place(std::ptr::addr_of_mut!(uid), group, name, buf, lnum, prio) == OK {
        uid as c_int
    } else {
        -1
    }
}

/// Unplace a sign from VimL typval parameters (migrated from C nvim_sign_unplace_from_dict_impl).
///
/// # Safety
/// `group_tv` must be a valid typval pointer or null. `dict` must be valid or null.
#[unsafe(export_name = "sign_unplace_from_dict")]
pub unsafe extern "C" fn rs_sign_unplace_from_dict(
    group_tv: *mut c_void,
    dict: *mut c_void,
) -> c_int {
    let mut id: c_int = 0;
    let mut buf = SignBufHandle::null();

    // Determine group
    let group: *const c_char = if !group_tv.is_null() {
        let g = tv_get_string(group_tv);
        if *g == 0 {
            std::ptr::null()
        } else {
            g
        }
    } else if !dict.is_null() {
        let g = tv_dict_get_string(dict, b"group\0".as_ptr().cast(), false);
        if g.is_null() || *g == 0 {
            std::ptr::null()
        } else {
            g
        }
    } else {
        std::ptr::null()
    };

    if !dict.is_null() {
        // buffer
        let di = tv_dict_find(dict, b"buffer\0".as_ptr().cast(), -1);
        if !di.is_null() {
            buf = get_buf_arg(nvim_di_get_tv(di));
            if buf.is_null() {
                return -1;
            }
        }
        // id
        let di2 = tv_dict_find(dict, b"id\0".as_ptr().cast(), -1);
        if !di2.is_null() {
            id = tv_dict_get_number(dict, b"id\0".as_ptr().cast()) as c_int;
            if id <= 0 {
                emsg(E_INVARG.as_ptr().cast());
                return -1;
            }
        }
    }

    (rs_sign_unplace(buf, id, group, 0) - 1) as c_int
}

/// Undefine multiple signs from a VimL list (migrated from C nvim_sign_undefine_multiple_impl).
///
/// # Safety
/// `l` and `retlist` must be valid list handles.
#[unsafe(export_name = "sign_undefine_multiple")]
pub unsafe extern "C" fn rs_sign_undefine_multiple(l: *mut c_void, retlist: *mut c_void) {
    if l.is_null() || retlist.is_null() {
        return;
    }
    let len = nvim_tv_list_len(l);
    for i in 0..len {
        let item_tv = nvim_tv_list_item_tv_at(l, i);
        let retval: c_int = if !item_tv.is_null() {
            let name = tv_get_string_chk(item_tv);
            if !name.is_null() && crate::define::rs_sign_undefine_by_name(name) == OK {
                0
            } else {
                -1
            }
        } else {
            -1
        };
        tv_list_append_number(retlist, i64::from(retval));
    }
}

// =============================================================================
// f_sign_* VimL Function Wrappers (migrated from C)
// =============================================================================

/// VimL sign_define() — migrated from C nvim_f_sign_define_impl.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[unsafe(export_name = "f_sign_define")]
pub unsafe extern "C" fn rs_f_sign_define(
    argvars: *mut c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let av0 = nvim_argvars_at(argvars, 0);
    // If first arg is a list and no second arg, define multiple
    if nvim_tv_get_type(av0) == VAR_LIST
        && nvim_tv_get_type(nvim_argvars_at(argvars, 1)) == VAR_UNKNOWN
    {
        tv_list_alloc_ret(rettv, K_LIST_LEN_MAY_KNOW);
        let l = nvim_tv_get_list(av0);
        let retl = nvim_rettv_get_list(rettv);
        rs_sign_define_multiple(l, retl);
        return;
    }
    nvim_rettv_set_number(rettv, -1);
    let name = tv_get_string_chk(av0).cast_mut();
    if name.is_null() {
        return;
    }
    if tv_check_for_opt_dict_arg(argvars, 1) == FAIL {
        return;
    }
    let av1 = nvim_argvars_at(argvars, 1);
    let d = if nvim_tv_get_type(av1) == VAR_DICT {
        nvim_tv_get_dict(av1)
    } else {
        std::ptr::null_mut()
    };
    nvim_rettv_set_number(rettv, i64::from(rs_sign_define_from_dict(name, d)));
}

/// VimL sign_getdefined() — migrated from C nvim_f_sign_getdefined_impl.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[unsafe(export_name = "f_sign_getdefined")]
pub unsafe extern "C" fn rs_f_sign_getdefined(
    argvars: *mut c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    tv_list_alloc_ret(rettv, 0);
    let retl = nvim_rettv_get_list(rettv);
    let av0 = nvim_argvars_at(argvars, 0);
    if nvim_tv_get_type(av0) == VAR_UNKNOWN {
        // List all defined signs
        let mut i = 0;
        loop {
            let sp = nvim_sign_map_get_nth_value(i);
            if sp.is_null() {
                break;
            }
            tv_list_append_dict(retl, alloc_info_dict(sp));
            i += 1;
        }
    } else {
        // List single sign
        let name = tv_get_string(av0);
        let sp = crate::define::rs_sign_get_by_name(name);
        if !sp.is_null() {
            tv_list_append_dict(retl, alloc_info_dict(sp));
        }
    }
}

/// VimL sign_getplaced() — migrated from C nvim_f_sign_getplaced_impl.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[unsafe(export_name = "f_sign_getplaced")]
pub unsafe extern "C" fn rs_f_sign_getplaced(
    argvars: *mut c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let mut buf = SignBufHandle::null();
    let mut lnum: LinenrT = 0;
    let mut sign_id: c_int = 0;
    let mut group: *const c_char = std::ptr::null();
    let mut notanum = false;
    tv_list_alloc_ret(rettv, 0);
    let retl = nvim_rettv_get_list(rettv);

    let av0 = nvim_argvars_at(argvars, 0);
    if nvim_tv_get_type(av0) != VAR_UNKNOWN {
        buf = get_buf_arg(av0);
        if buf.is_null() {
            return;
        }
        let av1 = nvim_argvars_at(argvars, 1);
        if nvim_tv_get_type(av1) != VAR_UNKNOWN {
            if tv_check_for_nonnull_dict_arg(argvars, 1) == FAIL {
                return;
            }
            let dict = nvim_tv_get_dict(av1);
            let di_lnum = tv_dict_find(dict, b"lnum\0".as_ptr().cast(), -1);
            if !di_lnum.is_null() {
                lnum = tv_get_lnum(nvim_di_get_tv(di_lnum));
                if lnum <= 0 {
                    return;
                }
            }
            let di_id = tv_dict_find(dict, b"id\0".as_ptr().cast(), -1);
            if !di_id.is_null() {
                sign_id = tv_get_number_chk(nvim_di_get_tv(di_id), std::ptr::addr_of_mut!(notanum))
                    as c_int;
                if notanum {
                    return;
                }
            }
            let di_grp = tv_dict_find(dict, b"group\0".as_ptr().cast(), -1);
            if !di_grp.is_null() {
                let g = tv_get_string_chk(nvim_di_get_tv(di_grp));
                if g.is_null() {
                    return;
                }
                group = if *g == 0 { std::ptr::null() } else { g };
            }
        }
    }
    // Delegate to existing Rust sign_get_placed
    crate::viml::rs_sign_get_placed(buf, lnum, sign_id, group, retl);
}

/// VimL sign_jump() — migrated from C nvim_f_sign_jump_impl.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[unsafe(export_name = "f_sign_jump")]
pub unsafe extern "C" fn rs_f_sign_jump(
    argvars: *mut c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_rettv_set_number(rettv, -1);
    let mut notanum = false;
    let av0 = nvim_argvars_at(argvars, 0);
    let id = tv_get_number_chk(av0, std::ptr::addr_of_mut!(notanum)) as c_int;
    if notanum {
        return;
    }
    if id <= 0 {
        emsg(E_INVARG.as_ptr().cast());
        return;
    }
    let av1 = nvim_argvars_at(argvars, 1);
    let group_raw = tv_get_string_chk(av1);
    if group_raw.is_null() {
        return;
    }
    let group: *const c_char = if *group_raw == 0 {
        std::ptr::null()
    } else {
        group_raw
    };
    let av2 = nvim_argvars_at(argvars, 2);
    let buf = get_buf_arg(av2);
    if buf.is_null() {
        return;
    }
    nvim_rettv_set_number(rettv, i64::from(rs_sign_jump(id, group, buf)));
}

/// VimL sign_place() — migrated from C nvim_f_sign_place_impl.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[unsafe(export_name = "f_sign_place")]
pub unsafe extern "C" fn rs_f_sign_place(
    argvars: *mut c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_rettv_set_number(rettv, -1);
    let av4 = nvim_argvars_at(argvars, 4);
    let d = if nvim_tv_get_type(av4) != VAR_UNKNOWN {
        if tv_check_for_nonnull_dict_arg(argvars, 4) == FAIL {
            return;
        }
        nvim_tv_get_dict(av4)
    } else {
        std::ptr::null_mut()
    };
    let av0 = nvim_argvars_at(argvars, 0);
    let av1 = nvim_argvars_at(argvars, 1);
    let av2 = nvim_argvars_at(argvars, 2);
    let av3 = nvim_argvars_at(argvars, 3);
    nvim_rettv_set_number(
        rettv,
        i64::from(rs_sign_place_from_dict(av0, av1, av2, av3, d)),
    );
}

/// VimL sign_placelist() — migrated from C nvim_f_sign_placelist_impl.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[unsafe(export_name = "f_sign_placelist")]
pub unsafe extern "C" fn rs_f_sign_placelist(
    argvars: *mut c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    tv_list_alloc_ret(rettv, K_LIST_LEN_MAY_KNOW);
    let retl = nvim_rettv_get_list(rettv);
    let av0 = nvim_argvars_at(argvars, 0);
    if nvim_tv_get_type(av0) != VAR_LIST {
        emsg(E_LISTREQ.as_ptr().cast());
        return;
    }
    let l = nvim_tv_get_list(av0);
    let len = nvim_tv_list_len(l);
    for i in 0..len {
        let item_tv = nvim_tv_list_item_tv_at(l, i);
        let sign_id = if !item_tv.is_null() && nvim_tv_get_type(item_tv) == VAR_DICT {
            rs_sign_place_from_dict(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                nvim_tv_get_dict(item_tv),
            )
        } else {
            emsg(E_DICTREQ.as_ptr().cast());
            -1
        };
        tv_list_append_number(retl, i64::from(sign_id));
    }
}

/// VimL sign_undefine() — migrated from C nvim_f_sign_undefine_impl.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[unsafe(export_name = "f_sign_undefine")]
pub unsafe extern "C" fn rs_f_sign_undefine(
    argvars: *mut c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let av0 = nvim_argvars_at(argvars, 0);
    // If first arg is a list and no second arg, undefine multiple
    if nvim_tv_get_type(av0) == VAR_LIST
        && nvim_tv_get_type(nvim_argvars_at(argvars, 1)) == VAR_UNKNOWN
    {
        tv_list_alloc_ret(rettv, K_LIST_LEN_MAY_KNOW);
        let retl = nvim_rettv_get_list(rettv);
        rs_sign_undefine_multiple(nvim_tv_get_list(av0), retl);
        return;
    }
    nvim_rettv_set_number(rettv, -1);
    if nvim_tv_get_type(av0) == VAR_UNKNOWN {
        // Undefine all
        let map_size = nvim_sign_map_size();
        // Collect all names first to avoid mutation during iteration
        let mut names = std::vec::Vec::with_capacity(map_size as usize);
        for i in 0..map_size {
            let key = nvim_sign_map_get_nth_key(i);
            if !key.is_null() {
                names.push(key);
            }
        }
        for name in names {
            crate::define::rs_sign_undefine_by_name(name);
        }
        nvim_rettv_set_number(rettv, 0);
    } else {
        let name = tv_get_string_chk(av0);
        if name.is_null() {
            return;
        }
        if crate::define::rs_sign_undefine_by_name(name) == OK {
            nvim_rettv_set_number(rettv, 0);
        }
    }
}

/// VimL sign_unplace() — migrated from C nvim_f_sign_unplace_impl.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[unsafe(export_name = "f_sign_unplace")]
pub unsafe extern "C" fn rs_f_sign_unplace(
    argvars: *mut c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_rettv_set_number(rettv, -1);
    if tv_check_for_string_arg(argvars, 0) == FAIL || tv_check_for_opt_dict_arg(argvars, 1) == FAIL
    {
        return;
    }
    let av1 = nvim_argvars_at(argvars, 1);
    let d = if nvim_tv_get_type(av1) != VAR_UNKNOWN {
        nvim_tv_get_dict(av1)
    } else {
        std::ptr::null_mut()
    };
    let av0 = nvim_argvars_at(argvars, 0);
    nvim_rettv_set_number(rettv, i64::from(rs_sign_unplace_from_dict(av0, d)));
}

/// VimL sign_unplacelist() — migrated from C nvim_f_sign_unplacelist_impl.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[unsafe(export_name = "f_sign_unplacelist")]
pub unsafe extern "C" fn rs_f_sign_unplacelist(
    argvars: *mut c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    tv_list_alloc_ret(rettv, K_LIST_LEN_MAY_KNOW);
    let retl = nvim_rettv_get_list(rettv);
    let av0 = nvim_argvars_at(argvars, 0);
    if nvim_tv_get_type(av0) != VAR_LIST {
        emsg(E_LISTREQ.as_ptr().cast());
        return;
    }
    let l = nvim_tv_get_list(av0);
    let len = nvim_tv_list_len(l);
    for i in 0..len {
        let item_tv = nvim_tv_list_item_tv_at(l, i);
        let retval = if !item_tv.is_null() && nvim_tv_get_type(item_tv) == VAR_DICT {
            rs_sign_unplace_from_dict(std::ptr::null_mut(), nvim_tv_get_dict(item_tv))
        } else {
            emsg(E_DICTREQ.as_ptr().cast());
            -1
        };
        tv_list_append_number(retl, i64::from(retval));
    }
}

// =============================================================================
// Return Value Helpers
// =============================================================================

/// Return value type for VimL sign functions.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignVimlReturnType {
    /// Return a number (sign ID or error code)
    Number = 0,
    /// Return a list
    List = 1,
    /// Return a dict
    Dict = 2,
}

/// Determine return type for sign_define().
#[no_mangle]
pub extern "C" fn rs_sign_define_return_type(is_list_arg: c_int) -> SignVimlReturnType {
    if is_list_arg != 0 {
        SignVimlReturnType::List
    } else {
        SignVimlReturnType::Number
    }
}

/// Determine return type for sign_place().
#[no_mangle]
pub extern "C" fn rs_sign_place_return_type(is_list_mode: c_int) -> SignVimlReturnType {
    if is_list_mode != 0 {
        SignVimlReturnType::List
    } else {
        SignVimlReturnType::Number
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_define_viml_params_default() {
        let params = SignDefineVimlParams::default();
        assert!(params.name.is_null());
        assert!(params.icon.is_null());
        assert!(params.text.is_null());
        assert_eq!(params.priority, -1);
    }

    #[test]
    fn test_sign_place_viml_params_default() {
        let params = SignPlaceVimlParams::default();
        assert_eq!(params.id, 0);
        assert!(params.group.is_null());
        assert!(params.name.is_null());
        assert!(params.buf.is_null());
        assert_eq!(params.lnum, 0);
        assert_eq!(params.priority, -1);
    }

    #[test]
    fn test_sign_place_viml_priority() {
        // Explicit priority takes precedence
        assert_eq!(rs_sign_place_viml_priority(5, 10), 5);
        // Fall back to definition priority
        assert_eq!(rs_sign_place_viml_priority(-1, 10), 10);
        // Fall back to default
        assert_eq!(rs_sign_place_viml_priority(-1, -1), SIGN_DEF_PRIO);
    }

    #[test]
    fn test_sign_get_placed_filter_default() {
        let filter = SignGetPlacedFilter::default();
        assert!(filter.buf.is_null());
        assert_eq!(filter.lnum, 0);
        assert_eq!(filter.id, 0);
        assert!(filter.group.is_null());
    }

    #[test]
    fn test_sign_get_placed_filter_is_all() {
        let filter = SignGetPlacedFilter::default();
        assert!(rs_sign_get_placed_filter_is_all(&filter));
    }

    #[test]
    fn test_sign_unplace_viml_params_default() {
        let params = SignUnplaceVimlParams::default();
        assert!(params.group.is_null());
        assert!(params.buf.is_null());
        assert_eq!(params.id, 0);
    }

    #[test]
    fn test_sign_jump_viml_params_default() {
        let params = SignJumpVimlParams::default();
        assert_eq!(params.id, 0);
        assert!(params.group.is_null());
        assert!(params.buf.is_null());
    }

    #[test]
    fn test_sign_define_return_type() {
        assert_eq!(rs_sign_define_return_type(0), SignVimlReturnType::Number);
        assert_eq!(rs_sign_define_return_type(1), SignVimlReturnType::List);
    }

    #[test]
    fn test_sign_place_return_type() {
        assert_eq!(rs_sign_place_return_type(0), SignVimlReturnType::Number);
        assert_eq!(rs_sign_place_return_type(1), SignVimlReturnType::List);
    }

    #[test]
    fn test_sign_define_result_values() {
        assert_eq!(SignDefineResult::Success as c_int, 0);
        assert_eq!(SignDefineResult::InvalidArg as c_int, -1);
        assert_eq!(SignDefineResult::Updated as c_int, 1);
    }

    #[test]
    fn test_sign_place_result_code_values() {
        assert_eq!(SignPlaceResultCode::Success as c_int, 0);
        assert_eq!(SignPlaceResultCode::InvalidArg as c_int, -1);
        assert_eq!(SignPlaceResultCode::NotDefined as c_int, -2);
        assert_eq!(SignPlaceResultCode::NoBuffer as c_int, -3);
        assert_eq!(SignPlaceResultCode::InvalidLine as c_int, -4);
    }

    #[test]
    fn test_sign_unplace_result_values() {
        assert_eq!(SignUnplaceResult::Success as c_int, 0);
        assert_eq!(SignUnplaceResult::NotFound as c_int, -1);
        assert_eq!(SignUnplaceResult::InvalidArg as c_int, -2);
    }
}
