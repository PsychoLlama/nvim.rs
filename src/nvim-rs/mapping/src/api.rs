//! API entry points: modify_keymap and keymap_array.
//!
//! Implements `modify_keymap` (set/delete a mapping via the nvim API) and
//! `keymap_array` (enumerate mappings for nvim_get_keymap / nvim_buf_get_keymap).

use std::ffi::{c_char, c_int, c_void};

use nvim_api::{Array, Dict, NvimString, ObjectType};

use crate::{
    args::MapArguments, BufHandle, MapblockHandle, MapblockT, SctxT, MODE_CMDLINE, MODE_INSERT,
};

// =============================================================================
// Constants
// =============================================================================

/// LUA_NOREF: no Lua reference set.
const LUA_NOREF: c_int = -2;

/// MAPTYPE_MAP
const MAPTYPE_MAP: c_int = 0;

/// MAPTYPE_NOREMAP
const MAPTYPE_NOREMAP: c_int = 2;

/// MAPTYPE_UNMAP
const MAPTYPE_UNMAP: c_int = 1;

/// ObjectType::Dict discriminant
const K_OBJECT_TYPE_DICT: c_int = ObjectType::Dict as c_int;

// =============================================================================
// FFI declarations
// =============================================================================

/// Error struct (repr(C) matches C Error type).
#[repr(C)]
pub struct ApiError {
    pub err_type: c_int,
    pub msg: *mut c_char,
}

extern "C" {
    // Buffer and sctx helpers (mapping.c Phase 4 additions)
    fn nvim_mapping_find_buffer_by_handle(buffer: c_int, err: *mut ApiError) -> BufHandle;
    fn nvim_mapping_api_set_sctx(channel_id: u64) -> SctxT;
    fn nvim_mapping_restore_sctx(sctx: SctxT);
    fn nvim_mapping_buf_handle(buf: BufHandle) -> c_int;

    // Dict(keymap)* accessors (all return values from opts fields)
    fn nvim_mapping_keymap_opts_get_noremap(opts: *const c_void) -> bool;
    fn nvim_mapping_keymap_opts_get_nowait(opts: *const c_void) -> bool;
    fn nvim_mapping_keymap_opts_get_silent(opts: *const c_void) -> bool;
    fn nvim_mapping_keymap_opts_get_script(opts: *const c_void) -> bool;
    fn nvim_mapping_keymap_opts_get_expr(opts: *const c_void) -> bool;
    fn nvim_mapping_keymap_opts_get_unique(opts: *const c_void) -> bool;
    fn nvim_mapping_keymap_opts_get_replace_keycodes(opts: *const c_void) -> bool;
    fn nvim_mapping_keymap_opts_has_callback(opts: *const c_void) -> bool;
    fn nvim_mapping_keymap_opts_take_callback(opts: *mut c_void) -> c_int;
    fn nvim_mapping_keymap_opts_has_desc(opts: *const c_void) -> bool;
    fn nvim_mapping_keymap_opts_get_desc_data(opts: *const c_void) -> *const c_char;
    fn nvim_mapping_keymap_opts_get_desc_size(opts: *const c_void) -> usize;
    fn nvim_mapping_string_to_cstr_len(data: *const c_char, size: usize) -> *mut c_char;

    // api_set_error wrappers
    fn nvim_mapping_api_set_error_validation(err: *mut ApiError, msg: *const c_char);
    fn nvim_mapping_api_set_error_validation_lhs(err: *mut ApiError, lhs: *const c_char);
    fn nvim_mapping_api_set_error_validation_mode(err: *mut ApiError, mode: *const c_char);
    fn nvim_mapping_api_set_error_validation_rhs_lhs(err: *mut ApiError, rhs: *const c_char);
    fn nvim_mapping_api_set_error_exception_invarg(err: *mut ApiError);
    fn nvim_mapping_api_set_error_exception_nomap(err: *mut ApiError);
    fn nvim_mapping_api_set_error_exception_abbr(
        err: *mut ApiError,
        is_abbrev: c_int,
        is_global: c_int,
        lhs: *const c_char,
    );

    // rs_* Rust functions (callable as extern "C")
    fn rs_get_map_mode(cmdp: *mut *mut c_char, forceit: c_int) -> c_int;
    fn rs_set_maparg_lhs_rhs(
        orig_lhs: *const c_char,
        orig_lhs_len: usize,
        orig_rhs: *const c_char,
        orig_rhs_len: usize,
        rhs_lua: c_int,
        cpo_val: *const c_char,
        mapargs: *mut MapArguments,
    ) -> c_int;
    fn rs_buf_do_map(
        maptype: c_int,
        args: *mut MapArguments,
        mode: c_int,
        is_abbrev: c_int,
        buf: BufHandle,
    ) -> c_int;
    fn nvim_mapping_get_p_cpo() -> *const c_char;

    // Memory
    fn xfree(ptr: *mut c_char);

    // LuaRef
    fn api_free_luaref(r: c_int);

    // ArrayBuilder wrappers (mapping.c Phase 4 additions)
    fn nvim_mapping_array_builder_new() -> *mut c_void;
    fn nvim_mapping_array_builder_push_dict(b: *mut c_void, d: Dict);
    fn nvim_mapping_array_builder_finish(arena: *mut c_void, b: *mut c_void) -> Array;

    // mapblock accessors (already in lib.rs but accessible via C)
    fn nvim_get_maphash_entry(index: c_int) -> MapblockHandle;
    fn nvim_get_first_abbr() -> MapblockHandle;
    fn nvim_buf_get_maphash_entry(buf: BufHandle, index: c_int) -> MapblockHandle;
    fn nvim_buf_get_first_abbr(buf: BufHandle) -> MapblockHandle;
}

// =============================================================================
// modify_keymap
// =============================================================================

/// Set, tweak, or remove a mapping. Implements nvim_set_keymap/nvim_buf_set_keymap/etc.
///
/// # Safety
/// All pointer parameters must be valid. `opts` may be null (no options).
///
/// # Panics
/// Panics if RHS is non-empty and should never have been set (internal invariant violation).
#[unsafe(export_name = "modify_keymap")]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_modify_keymap(
    channel_id: u64,
    buffer: c_int, // Buffer = handle_T = int
    is_unmap: bool,
    mode: NvimString,
    lhs: NvimString,
    rhs: NvimString,
    opts: *mut c_void, // Dict(keymap)*
    err: *mut ApiError,
) {
    let mut lua_funcref: c_int = LUA_NOREF;
    let global = buffer == -1;
    let buffer_handle: c_int = if global { 0 } else { buffer };

    let target_buf = nvim_mapping_find_buffer_by_handle(buffer_handle, err);
    if target_buf.is_null() {
        return;
    }

    let save_sctx = nvim_mapping_api_set_sctx(channel_id);

    let mut parsed_args: MapArguments = std::mem::zeroed();
    if !opts.is_null() {
        parsed_args.nowait = nvim_mapping_keymap_opts_get_nowait(opts);
        parsed_args.noremap = nvim_mapping_keymap_opts_get_noremap(opts);
        parsed_args.silent = nvim_mapping_keymap_opts_get_silent(opts);
        parsed_args.script = nvim_mapping_keymap_opts_get_script(opts);
        parsed_args.expr = nvim_mapping_keymap_opts_get_expr(opts);
        parsed_args.unique = nvim_mapping_keymap_opts_get_unique(opts);
        parsed_args.replace_keycodes = nvim_mapping_keymap_opts_get_replace_keycodes(opts);
        if nvim_mapping_keymap_opts_has_callback(opts) {
            lua_funcref = nvim_mapping_keymap_opts_take_callback(opts);
        }
        if nvim_mapping_keymap_opts_has_desc(opts) {
            let desc_data = nvim_mapping_keymap_opts_get_desc_data(opts);
            let desc_size = nvim_mapping_keymap_opts_get_desc_size(opts);
            parsed_args.desc = nvim_mapping_string_to_cstr_len(desc_data, desc_size);
        }
    }
    parsed_args.buffer = !global;

    if parsed_args.replace_keycodes && !parsed_args.expr {
        nvim_mapping_api_set_error_validation(
            err,
            c"\"replace_keycodes\" requires \"expr\"".as_ptr(),
        );
        cleanup(&raw mut parsed_args, save_sctx);
        return;
    }

    let p_cpo = nvim_mapping_get_p_cpo();
    if rs_set_maparg_lhs_rhs(
        lhs.data,
        lhs.size,
        rhs.data,
        rhs.size,
        lua_funcref,
        p_cpo,
        &raw mut parsed_args,
    ) == 0
    {
        nvim_mapping_api_set_error_validation_lhs(err, lhs.data);
        cleanup(&raw mut parsed_args, save_sctx);
        return;
    }

    if parsed_args.lhs_len > crate::MAXMAPLEN || parsed_args.alt_lhs_len > crate::MAXMAPLEN {
        nvim_mapping_api_set_error_validation_lhs(err, lhs.data);
        cleanup(&raw mut parsed_args, save_sctx);
        return;
    }

    // Parse mode string.
    let m_char = b'm' as c_char;
    let mut p: *mut c_char = if mode.size > 0 {
        mode.data
    } else {
        std::ptr::addr_of!(m_char).cast_mut()
    };
    let forceit = *p == b'!' as c_char;
    let mode_val = rs_get_map_mode(std::ptr::addr_of_mut!(p), c_int::from(forceit));
    if forceit {
        p = p.add(1);
    }
    let is_abbrev = (mode_val & (MODE_INSERT | MODE_CMDLINE)) != 0 && *p == b'a' as c_char;
    if is_abbrev {
        p = p.add(1);
    }
    if mode.size > 0 && p.offset_from(mode.data) as usize != mode.size {
        nvim_mapping_api_set_error_validation_mode(err, mode.data);
        cleanup(&raw mut parsed_args, save_sctx);
        return;
    }

    if parsed_args.lhs_len == 0 {
        nvim_mapping_api_set_error_validation(err, c"Invalid (empty) LHS".as_ptr());
        cleanup(&raw mut parsed_args, save_sctx);
        return;
    }

    let is_noremap = parsed_args.noremap;
    debug_assert!(!(is_unmap && is_noremap));

    if !is_unmap && lua_funcref == LUA_NOREF && parsed_args.rhs_len == 0 && !parsed_args.rhs_is_noop
    {
        if rhs.size == 0 {
            parsed_args.rhs_is_noop = true;
        } else {
            // Should never happen; C code calls abort() here.
            panic!("modify_keymap: non-empty RHS with no lhs_is_noop");
        }
    } else if is_unmap && (parsed_args.rhs_len != 0 || parsed_args.rhs_lua != LUA_NOREF) {
        if parsed_args.rhs_len != 0 {
            nvim_mapping_api_set_error_validation_rhs_lhs(err, parsed_args.rhs);
        } else {
            nvim_mapping_api_set_error_validation(err, c"Gave nonempty RHS for unmap".as_ptr());
        }
        cleanup(&raw mut parsed_args, save_sctx);
        return;
    }

    let maptype_val = if is_unmap {
        MAPTYPE_UNMAP
    } else if is_noremap {
        MAPTYPE_NOREMAP
    } else {
        MAPTYPE_MAP
    };

    let rc = rs_buf_do_map(
        maptype_val,
        &raw mut parsed_args,
        mode_val,
        c_int::from(is_abbrev),
        target_buf,
    );

    match rc {
        0 => {}
        1 => {
            nvim_mapping_api_set_error_exception_invarg(err);
        }
        2 => {
            nvim_mapping_api_set_error_exception_nomap(err);
        }
        5 => {
            nvim_mapping_api_set_error_exception_abbr(err, c_int::from(is_abbrev), 0, lhs.data);
        }
        6 => {
            nvim_mapping_api_set_error_exception_abbr(err, c_int::from(is_abbrev), 1, lhs.data);
        }
        _ => {
            debug_assert!(false, "Unrecognized rs_buf_do_map return code: {rc}");
        }
    }

    cleanup(&raw mut parsed_args, save_sctx);
}

/// Cleanup helper: restore sctx and free parsed_args allocations.
///
/// # Safety
/// `parsed_args` must be a valid pointer to MapArguments with any allocated fields.
unsafe fn cleanup(parsed_args: *mut MapArguments, save_sctx: SctxT) {
    nvim_mapping_restore_sctx(save_sctx);
    // NLUA_CLEAR_REF equivalent: free the lua ref if set.
    if (*parsed_args).rhs_lua != LUA_NOREF {
        api_free_luaref((*parsed_args).rhs_lua);
        (*parsed_args).rhs_lua = LUA_NOREF;
    }
    xfree((*parsed_args).rhs);
    xfree((*parsed_args).orig_rhs);
    xfree((*parsed_args).desc);
}

// =============================================================================
// keymap_array
// =============================================================================

/// Get an array of dicts describing all mappings for a mode and buffer.
///
/// # Safety
/// `buf` may be null (global maps). `arena` must be a valid arena pointer.
#[unsafe(export_name = "keymap_array")]
pub unsafe extern "C" fn rs_keymap_array(
    mode: NvimString,
    buf: BufHandle,
    arena: *mut c_void,
) -> Array {
    let m_char = b'm' as c_char;
    let mut p: *mut c_char = if mode.size > 0 {
        mode.data
    } else {
        std::ptr::addr_of!(m_char).cast_mut()
    };
    let forceit = *p == b'!' as c_char;
    let int_mode = rs_get_map_mode(std::ptr::addr_of_mut!(p), c_int::from(forceit));
    if forceit {
        p = p.add(1);
    }
    let is_abbrev = (int_mode & (MODE_INSERT | MODE_CMDLINE)) != 0 && *p == b'a' as c_char;

    let buffer_value: c_int = if buf.is_null() {
        0
    } else {
        nvim_mapping_buf_handle(buf)
    };

    let builder = nvim_mapping_array_builder_new();

    let max_hash = if is_abbrev { 1 } else { crate::MAX_MAPHASH };
    for i in 0..max_hash {
        let mut mp: MapblockHandle = if is_abbrev {
            if buf.is_null() {
                nvim_get_first_abbr()
            } else {
                nvim_buf_get_first_abbr(buf)
            }
        } else if buf.is_null() {
            nvim_get_maphash_entry(i)
        } else {
            nvim_buf_get_maphash_entry(buf, i)
        };

        while !mp.is_null() {
            let cur: &MapblockT = &*mp;
            if cur.m_simplified == 0 && (int_mode & cur.m_mode) != 0 {
                let lhsrawalt: *const c_char = if cur.m_alt.is_null() {
                    std::ptr::null()
                } else {
                    (*cur.m_alt).m_keys
                };
                // Use a local arena per entry - actually, keymap_array shares the caller's arena.
                let dict = crate::query::rs_mapblock_fill_dict(
                    mp,
                    lhsrawalt,
                    buffer_value,
                    is_abbrev,
                    false,
                    arena.cast(),
                );
                nvim_mapping_array_builder_push_dict(builder, dict);
            }
            mp = cur.m_next;
        }
    }

    nvim_mapping_array_builder_finish(arena, builder)
}
