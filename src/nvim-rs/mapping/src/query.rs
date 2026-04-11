//! mapblock_fill_dict, get_maparg and VimL query functions.
//!
//! Provides Rust implementations of `mapblock_fill_dict` (fill an Arena Dict
//! with maparg-like entries from a mapblock) and `get_maparg` (implement
//! maparg()/mapcheck() lookup).

use std::ffi::{c_char, c_int, c_void};

use nvim_api::{Dict, KeyValuePair, NvimString, Object, ObjectType};

use crate::MapblockHandle;

// =============================================================================
// Constants
// =============================================================================

/// LUA_NOREF: Lua reference that is not set.
const LUA_NOREF: c_int = -2;

/// REMAP_SCRIPT: remap via <SID>.
const REMAP_SCRIPT: c_int = -2;

/// REPTERM_FROM_PART
const REPTERM_FROM_PART: c_int = 1;

/// REPTERM_DO_LT
const REPTERM_DO_LT: c_int = 2;

/// REPTERM_NO_SIMPLIFY
const REPTERM_NO_SIMPLIFY: c_int = 8;

/// VAR_NUMBER: typval_T v_type for a number value.
const VAR_NUMBER: c_int = 1;

/// VAR_STRING: typval_T v_type for a string value.
const VAR_STRING: c_int = 2;

/// VAR_FUNC: typval_T v_type for a function reference.
const VAR_FUNC: c_int = 3;

/// VAR_DICT: typval_T v_type for a dict value.
const VAR_DICT: c_int = 5;

/// VAR_UNKNOWN: typval_T v_type for "no value".
const VAR_UNKNOWN: c_int = 0;

/// ObjectType::LuaRef discriminant
const K_OBJECT_TYPE_LUAREF: c_int = ObjectType::LuaRef as c_int;

/// ObjectType::Dict discriminant
const K_OBJECT_TYPE_DICT: c_int = ObjectType::Dict as c_int;

// =============================================================================
// Arena type (matches C memory_defs.h)
// =============================================================================

/// Arena struct matching C definition from memory_defs.h.
/// Layout: { char *cur_blk; size_t pos; size_t size; }
#[repr(C)]
pub struct CArena {
    cur_blk: *mut c_char,
    pos: usize,
    size: usize,
}

impl CArena {
    const fn empty() -> Self {
        Self {
            cur_blk: std::ptr::null_mut(),
            pos: 0,
            size: 0,
        }
    }
}

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // str2special: convert key string to printable form
    fn str2special_arena(
        str_: *const c_char,
        replace_spaces: bool,
        replace_lt: bool,
        arena: *mut CArena,
    ) -> *mut c_char;

    fn str2special_save(str_: *const c_char, replace_spaces: bool, replace_lt: bool)
        -> *mut c_char;

    // map_mode_to_chars: fill a 7-byte buffer with mode chars
    fn map_mode_to_chars(mode: c_int, buf: *mut c_char);

    // api_new_luaref: duplicate a LuaRef
    fn api_new_luaref(original_ref: c_int) -> c_int;

    // arena_alloc: allocate bytes from an arena
    fn arena_alloc(arena: *mut CArena, size: usize, align: bool) -> *mut c_char;

    // arena_finish/arena_mem_free: finish and free arena memory
    fn arena_finish(arena: *mut CArena) -> *mut c_void;
    fn arena_mem_free(mem: *mut c_void);

    // object_to_vim_take_luaref: convert Object to typval_T, consuming luarefs
    fn object_to_vim_take_luaref(
        obj: *const Object,
        tv: *mut c_void,
        take_luaref: bool,
        err: *mut c_void,
    );

    // tv_dict_alloc_ret: set rettv to an empty dict
    fn tv_dict_alloc_ret(rettv: *mut c_void);

    // tv accessors (typval_T is opaque, 16-byte struct)
    fn tv_get_string(tv: *const c_void) -> *const c_char;
    fn tv_get_string_buf_chk(tv: *const c_void, buf: *mut c_char) -> *const c_char;
    fn tv_get_number(tv: *const c_void) -> i64;

    // replace_termcodes: process escape sequences
    fn replace_termcodes(
        from: *const c_char,
        from_len: usize,
        bufp: *mut *mut c_char,
        sid_arg: c_int,
        flags: c_int,
        did_simplify: *mut bool,
        cpo_val: *const c_char,
    ) -> *mut c_char;

    // check_map: find a mapping (already in Rust, callable as C)
    fn check_map(
        keys: *mut c_char,
        mode: c_int,
        exact: c_int,
        ign_mod: c_int,
        abbr: c_int,
        mp_ptr: *mut MapblockHandle,
        local_ptr: *mut c_int,
        rhs_lua: *mut c_int,
    ) -> *mut c_char;

    // nlua_funcref_str: describe a Lua function ref as string
    fn nlua_funcref_str(luaref: c_int, arena: *mut c_void) -> *mut c_char;

    // xfree, xstrdup
    fn xfree(ptr: *mut c_char);
    fn xstrdup(s: *const c_char) -> *mut c_char;

    // p_cpo accessor
    fn nvim_mapping_get_p_cpo() -> *const c_char;

    // rs_get_map_mode: parse mode string
    fn rs_get_map_mode(cmdp: *mut *mut c_char, forceit: c_int) -> c_int;
}

// =============================================================================
// typval helpers
// =============================================================================

/// Size of typval_T: v_type(4) + v_lock(4) + vval(8) = 16 bytes.
const TYPVAL_SIZE: usize = 16;

/// NUMBUFLEN for tv_get_string_buf_chk
const NUMBUFLEN: usize = 65;

/// Get a pointer to argvars[i] in a typval_T array.
///
/// # Safety
/// `argvars` must point to an array of at least `i+1` typval_T values.
#[inline]
unsafe fn tv_idx(argvars: *const c_void, i: usize) -> *const c_void {
    argvars.cast::<u8>().add(i * TYPVAL_SIZE).cast()
}

/// Get the v_type field of a typval_T.
///
/// # Safety
/// `tv` must be a valid non-null typval_T pointer.
#[inline]
unsafe fn tv_type(tv: *const c_void) -> c_int {
    *(tv.cast::<c_int>())
}

/// Set the v_type field of a typval_T.
///
/// # Safety
/// `tv` must be a valid non-null typval_T pointer.
#[inline]
unsafe fn tv_set_type(tv: *mut c_void, t: c_int) {
    *(tv.cast::<c_int>()) = t;
}

/// Set the vval.v_string field of a typval_T (offset 8, *mut c_char).
///
/// # Safety
/// `tv` must be a valid non-null typval_T pointer.
#[inline]
#[allow(clippy::cast_ptr_alignment)]
unsafe fn tv_set_vstring(tv: *mut c_void, s: *mut c_char) {
    *tv.cast::<u8>().add(8).cast::<*mut c_char>() = s;
}

// =============================================================================
// Dict helpers
// =============================================================================

/// Allocate a Dict from an arena with pre-set capacity.
///
/// # Safety
/// `arena` must be a valid arena pointer.
unsafe fn make_arena_dict(arena: *mut CArena, max_size: usize) -> Dict {
    #[allow(clippy::cast_ptr_alignment)]
    let items = arena_alloc(arena, max_size * std::mem::size_of::<KeyValuePair>(), true)
        .cast::<KeyValuePair>();
    Dict {
        size: 0,
        capacity: max_size,
        items,
    }
}

/// Build an Object from a C string (CSTR_AS_OBJ equivalent).
/// Points into the string without copying. The string must outlive the Object.
///
/// # Safety
/// `s` must be a valid NUL-terminated C string.
#[inline]
unsafe fn cstr_obj(s: *const c_char) -> Object {
    Object::string(NvimString {
        data: s.cast_mut(),
        size: libc::strlen(s),
    })
}

/// Build an Object from a C string with known length (CSTR_AS_OBJ equivalent).
///
/// # Safety
/// `s` must be valid for `size` bytes.
#[inline]
unsafe fn cstr_obj_len(s: *const c_char, size: usize) -> Object {
    Object::string(NvimString {
        data: s.cast_mut(),
        size,
    })
}

/// Build an integer Object (INTEGER_OBJ equivalent).
#[inline]
fn int_obj(i: i64) -> Object {
    Object::integer(i)
}

/// Build a LuaRef Object (LUAREF_OBJ equivalent).
#[inline]
fn luaref_obj(r: c_int) -> Object {
    Object {
        obj_type: K_OBJECT_TYPE_LUAREF,
        data: nvim_api::ObjectData {
            luaref: i64::from(r),
        },
    }
}

/// Add a key-value pair to a dict using a NUL-terminated key.
///
/// # Safety
/// - `dict` must have been allocated with sufficient capacity.
/// - `key` must be a valid NUL-terminated C string.
#[inline]
unsafe fn dict_put(dict: &mut Dict, key: *const c_char, value: Object) {
    debug_assert!(dict.size < dict.capacity);
    let pair = &mut *dict.items.add(dict.size);
    *pair = KeyValuePair {
        key: NvimString {
            data: key.cast_mut(),
            size: libc::strlen(key),
        },
        value,
    };
    dict.size += 1;
}

// =============================================================================
// mapblock_fill_dict
// =============================================================================

/// Fill an Arena-allocated Dict with maparg()-like dictionary entries.
///
/// Equivalent to the C `mapblock_fill_dict` static function.
///
/// # Safety
/// `mp` must be a valid non-null mapblock pointer.
/// `arena` must be a valid arena pointer.
#[unsafe(export_name = "mapblock_fill_dict")]
pub unsafe extern "C" fn rs_mapblock_fill_dict(
    mp: *const crate::MapblockT,
    lhsrawalt: *const c_char,
    buffer_value: c_int,
    abbr: bool,
    compatible: bool,
    arena: *mut CArena,
) -> Dict {
    let mut dict = make_arena_dict(arena, 19);

    let lhs = str2special_arena((*mp).m_keys, compatible, !compatible, arena);
    let mapmode_buf = arena_alloc(arena, 7, false);
    map_mode_to_chars((*mp).m_mode, mapmode_buf);

    let noremap_value: i64 = if compatible {
        i64::from((*mp).m_noremap != 0)
    } else if (*mp).m_noremap == REMAP_SCRIPT {
        2
    } else {
        i64::from((*mp).m_noremap != 0)
    };

    if (*mp).m_luaref == LUA_NOREF {
        let rhs_str: *const c_char = if compatible {
            (*mp).m_orig_str
        } else {
            str2special_arena((*mp).m_str, false, true, arena)
        };
        if rhs_str.is_null() {
            dict_put(&mut dict, c"rhs".as_ptr(), cstr_obj_len(c"".as_ptr(), 0));
        } else {
            dict_put(&mut dict, c"rhs".as_ptr(), cstr_obj(rhs_str));
        }
    } else {
        let new_ref = api_new_luaref((*mp).m_luaref);
        dict_put(&mut dict, c"callback".as_ptr(), luaref_obj(new_ref));
    }

    if !(*mp).m_desc.is_null() {
        dict_put(&mut dict, c"desc".as_ptr(), cstr_obj((*mp).m_desc));
    }
    dict_put(&mut dict, c"lhs".as_ptr(), cstr_obj(lhs));
    dict_put(&mut dict, c"lhsraw".as_ptr(), cstr_obj((*mp).m_keys));
    if !lhsrawalt.is_null() {
        dict_put(&mut dict, c"lhsrawalt".as_ptr(), cstr_obj(lhsrawalt));
    }
    dict_put(&mut dict, c"noremap".as_ptr(), int_obj(noremap_value));
    dict_put(
        &mut dict,
        c"script".as_ptr(),
        int_obj(i64::from((*mp).m_noremap == REMAP_SCRIPT)),
    );
    dict_put(
        &mut dict,
        c"expr".as_ptr(),
        int_obj(i64::from((*mp).m_expr != 0)),
    );
    dict_put(
        &mut dict,
        c"silent".as_ptr(),
        int_obj(i64::from((*mp).m_silent != 0)),
    );
    dict_put(
        &mut dict,
        c"sid".as_ptr(),
        int_obj(i64::from((*mp).m_script_ctx.sc_sid)),
    );
    dict_put(&mut dict, c"scriptversion".as_ptr(), int_obj(1));
    dict_put(
        &mut dict,
        c"lnum".as_ptr(),
        int_obj(i64::from((*mp).m_script_ctx.sc_lnum)),
    );
    dict_put(
        &mut dict,
        c"buffer".as_ptr(),
        int_obj(i64::from(buffer_value)),
    );
    dict_put(
        &mut dict,
        c"nowait".as_ptr(),
        int_obj(i64::from((*mp).m_nowait != 0)),
    );
    if (*mp).m_replace_keycodes {
        dict_put(&mut dict, c"replace_keycodes".as_ptr(), int_obj(1));
    }
    let mapmode_len = libc::strlen(mapmode_buf);
    dict_put(
        &mut dict,
        c"mode".as_ptr(),
        cstr_obj_len(mapmode_buf, mapmode_len),
    );
    dict_put(&mut dict, c"abbr".as_ptr(), int_obj(i64::from(abbr)));
    dict_put(
        &mut dict,
        c"mode_bits".as_ptr(),
        int_obj(i64::from((*mp).m_mode)),
    );

    dict
}

// =============================================================================
// get_maparg (shared impl for maparg/mapcheck)
// =============================================================================

/// Shared implementation for maparg() and mapcheck().
///
/// Equivalent to the C `get_maparg` static function.
///
/// # Safety
/// `argvars` must be a valid pointer to at least 4 typval_T values.
/// `rettv` must be a valid pointer to a typval_T.
#[allow(clippy::too_many_lines)]
unsafe fn get_maparg_impl(argvars: *const c_void, rettv: *mut c_void, exact: c_int) {
    // Return empty string for failure.
    tv_set_type(rettv, VAR_STRING);
    tv_set_vstring(rettv, std::ptr::null_mut());

    let keys = tv_get_string(tv_idx(argvars, 0)).cast_mut();
    if keys.is_null() || *keys == 0 {
        return;
    }

    let which: *const c_char;
    let mut buf = [0u8; NUMBUFLEN];
    let mut abbr = false;
    let mut get_dict = false;

    if tv_type(tv_idx(argvars, 1)) == VAR_UNKNOWN {
        which = c"".as_ptr();
    } else {
        which = tv_get_string_buf_chk(tv_idx(argvars, 1), buf.as_mut_ptr().cast());
        if tv_type(tv_idx(argvars, 2)) != VAR_UNKNOWN {
            abbr = tv_get_number(tv_idx(argvars, 2)) != 0;
            if tv_type(tv_idx(argvars, 3)) != VAR_UNKNOWN {
                get_dict = tv_get_number(tv_idx(argvars, 3)) != 0;
            }
        }
    }

    if which.is_null() {
        return;
    }

    let mut which_mut = which.cast_mut();
    let mode = rs_get_map_mode(std::ptr::addr_of_mut!(which_mut), 0);

    let mut keys_buf: *mut c_char = std::ptr::null_mut();
    let mut alt_keys_buf: *mut c_char = std::ptr::null_mut();
    let mut did_simplify = false;
    let flags = REPTERM_FROM_PART | REPTERM_DO_LT;
    let p_cpo = nvim_mapping_get_p_cpo();

    let keys_simplified = replace_termcodes(
        keys,
        libc::strlen(keys.cast()),
        std::ptr::addr_of_mut!(keys_buf),
        0,
        flags,
        std::ptr::addr_of_mut!(did_simplify),
        p_cpo,
    );

    let mut mp: MapblockHandle = std::ptr::null_mut();
    let mut buffer_local: c_int = 0;
    let mut rhs_lua: c_int = LUA_NOREF;

    let rhs = if did_simplify {
        // When the lhs is being simplified, the not-simplified keys are preferred
        // for printing, like in do_map().
        replace_termcodes(
            keys,
            libc::strlen(keys.cast()),
            std::ptr::addr_of_mut!(alt_keys_buf),
            0,
            flags | REPTERM_NO_SIMPLIFY,
            std::ptr::null_mut(),
            p_cpo,
        );
        check_map(
            alt_keys_buf,
            mode,
            exact,
            0,
            c_int::from(abbr),
            std::ptr::addr_of_mut!(mp),
            std::ptr::addr_of_mut!(buffer_local),
            std::ptr::addr_of_mut!(rhs_lua),
        )
    } else {
        check_map(
            keys_simplified,
            mode,
            exact,
            0,
            c_int::from(abbr),
            std::ptr::addr_of_mut!(mp),
            std::ptr::addr_of_mut!(buffer_local),
            std::ptr::addr_of_mut!(rhs_lua),
        )
    };

    if get_dict {
        // Return a dictionary.
        if !mp.is_null() && (!rhs.is_null() || rhs_lua != LUA_NOREF) {
            let mut arena = CArena::empty();
            let dict = rs_mapblock_fill_dict(
                mp,
                if did_simplify {
                    keys_simplified
                } else {
                    std::ptr::null()
                },
                buffer_local,
                abbr,
                true,
                std::ptr::addr_of_mut!(arena),
            );
            let dict_obj = Object {
                obj_type: K_OBJECT_TYPE_DICT,
                data: nvim_api::ObjectData { dict },
            };
            object_to_vim_take_luaref(
                std::ptr::addr_of!(dict_obj),
                rettv,
                true,
                std::ptr::null_mut(),
            );
            let mem = arena_finish(std::ptr::addr_of_mut!(arena));
            arena_mem_free(mem);
        } else {
            // Return an empty dictionary.
            tv_dict_alloc_ret(rettv);
        }
    } else {
        // Return a string.
        if !rhs.is_null() {
            tv_set_type(rettv, VAR_STRING);
            if *rhs == 0 {
                tv_set_vstring(rettv, xstrdup(c"<Nop>".as_ptr()));
            } else {
                tv_set_vstring(rettv, str2special_save(rhs, false, false));
            }
        } else if rhs_lua != LUA_NOREF {
            tv_set_type(rettv, VAR_STRING);
            tv_set_vstring(
                rettv,
                nlua_funcref_str((*mp).m_luaref, std::ptr::null_mut()),
            );
        }
    }

    xfree(keys_buf);
    xfree(alt_keys_buf);
}

// =============================================================================
// f_maparg / f_mapcheck
// =============================================================================

/// "maparg()" function: exact=1 means exact match required.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval_T pointers.
#[unsafe(export_name = "f_maparg")]
pub unsafe extern "C" fn rs_f_maparg(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    get_maparg_impl(argvars, rettv, 1);
}

/// "mapcheck()" function: exact=0 means prefix match is OK.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval_T pointers.
#[unsafe(export_name = "f_mapcheck")]
pub unsafe extern "C" fn rs_f_mapcheck(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    get_maparg_impl(argvars, rettv, 0);
}

// =============================================================================
// Phase 3 FFI additions
// =============================================================================

extern "C" {
    // typval helpers (opaque typval_T*) -- from eval/typval.c
    fn nvim_tv_idx(argvars: *const c_void, i: c_int) -> *mut c_void;
    fn nvim_tv_get_type(tv: *const c_void) -> c_int;
    fn nvim_tv_get_dict(tv: *const c_void) -> *mut c_void; // dict_T*
    fn nvim_tv_get_list(tv: *const c_void) -> *mut c_void; // list_T*

    // tv_dict ops -- implemented in Rust (nvim-typval crate)
    fn tv_dict_get_string(d: *mut c_void, key: *const c_char, allocate: bool) -> *mut c_char;
    fn tv_dict_get_bool(d: *mut c_void, key: *const c_char, def: c_int) -> i64;
    fn tv_dict_get_number(d: *mut c_void, key: *const c_char) -> i64;
    fn tv_dict_find(d: *mut c_void, key: *const c_char, len: isize) -> *mut c_void; // dictitem_T*
    fn tv_check_for_dict_arg(argvars: *const c_void, idx: c_int) -> c_int;
    fn tv_get_bool(tv: *const c_void) -> i64;

    // tv_list ops -- implemented in Rust (nvim-typval crate)
    fn tv_list_alloc_ret(rettv: *mut c_void, len: c_int);
    fn tv_list_append_dict(list: *mut c_void, dict: *mut c_void);

    // ufunc_T accessors
    fn find_func(name: *const c_char) -> *mut c_void; // ufunc_T*
    fn nvim_ufunc_get_flags(fp: *mut c_void) -> c_int;
    fn nvim_ufunc_get_luaref(fp: *const c_void) -> c_int;

    // dictitem_T accessors (mapping.c Phase 3 additions)
    fn nvim_mapping_dictitem_tv_type(di: *const c_void) -> c_int;
    fn nvim_mapping_dictitem_tv_vstring(di: *const c_void) -> *const c_char;

    // Error emitters
    fn nvim_mapping_emsg_entries_missing();
    fn nvim_mapping_semsg_illegal_map_mode(which: *const c_char);

    // api_free_luaref: release a Lua reference
    fn api_free_luaref(r: c_int);

    // rs_set_maparg_rhs / rs_set_maparg_lhs_rhs / rs_buf_do_map / rs_map_add
    // are Rust functions; call them directly by their extern "C" names.
    fn rs_get_map_mode_string(mode_string: *const c_char, abbr: c_int) -> c_int;
    fn rs_set_maparg_rhs(
        orig_rhs: *const c_char,
        orig_rhs_len: usize,
        rhs_lua: c_int,
        sid: c_int,
        cpo_val: *const c_char,
        mapargs: *mut crate::args::MapArguments,
    );
    fn rs_set_maparg_lhs_rhs(
        orig_lhs: *const c_char,
        orig_lhs_len: usize,
        orig_rhs: *const c_char,
        orig_rhs_len: usize,
        rhs_lua: c_int,
        cpo_val: *const c_char,
        mapargs: *mut crate::args::MapArguments,
    ) -> c_int;
    fn rs_buf_do_map(
        maptype: c_int,
        args: *mut crate::args::MapArguments,
        mode: c_int,
        is_abbrev: c_int,
        buf: crate::BufHandle,
    ) -> c_int;
    fn rs_map_add(
        buf: crate::BufHandle,
        is_buf_local: c_int,
        keys: *const c_char,
        args: *mut crate::args::MapArguments,
        noremap: c_int,
        mode: c_int,
        is_abbr: c_int,
        sid: c_int,
        lnum: c_int,
        simplified: c_int,
    ) -> crate::MapblockHandle;
}

/// FC_LUAREF flag (userfunc.h)
const FC_LUAREF: c_int = 0x800;

/// MAPTYPE_UNMAP_LHS: unmap by lhs only.
const MAPTYPE_UNMAP_LHS: c_int = 3;

/// REMAP_NONE: disable remapping.
const REMAP_NONE: c_int = -1;

/// FAIL: C FAIL return value.
const FAIL: c_int = 0;

// =============================================================================
// f_mapset
// =============================================================================

/// "mapset()" function -- restore a mapping from a dict (e.g. from maparg()).
///
/// # Safety
/// `argvars` and `rettv` must be valid typval_T pointers.
#[unsafe(export_name = "f_mapset")]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_f_mapset(
    argvars: *const c_void,
    _rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let av0 = nvim_tv_idx(argvars, 0);

    // If first arg is a dict, then that's the only arg permitted.
    let dict_only = nvim_tv_get_type(av0) == VAR_DICT;

    let d: *mut c_void;
    let which: *const c_char;
    let is_abbr: c_int;

    let mut buf = [0u8; NUMBUFLEN];

    if dict_only {
        d = nvim_tv_get_dict(av0);
        which = tv_dict_get_string(d, c"mode".as_ptr(), false);
        let abbr_val = tv_dict_get_bool(d, c"abbr".as_ptr(), -1);
        if which.is_null() || abbr_val < 0 {
            nvim_mapping_emsg_entries_missing();
            return;
        }
        is_abbr = abbr_val as c_int;
    } else {
        which = tv_get_string_buf_chk(av0, buf.as_mut_ptr().cast());
        if which.is_null() {
            return;
        }
        is_abbr = tv_get_bool(nvim_tv_idx(argvars, 1)) as c_int;
        if tv_check_for_dict_arg(argvars, 2) == FAIL {
            return;
        }
        d = nvim_tv_get_dict(nvim_tv_idx(argvars, 2));
    }

    let mode = rs_get_map_mode_string(which, is_abbr);
    if mode == 0 {
        nvim_mapping_semsg_illegal_map_mode(which);
        return;
    }

    // Get the values in the same order as in get_maparg().
    let lhs = tv_dict_get_string(d, c"lhs".as_ptr(), false);
    let lhsraw = tv_dict_get_string(d, c"lhsraw".as_ptr(), false);
    let lhsrawalt = tv_dict_get_string(d, c"lhsrawalt".as_ptr(), false);
    let mut orig_rhs = tv_dict_get_string(d, c"rhs".as_ptr(), false);
    let mut rhs_lua: c_int = LUA_NOREF;

    // Check for a Lua callback in the dict.
    // S_LEN("callback") = key, len = 8
    let callback_di = tv_dict_find(d, c"callback".as_ptr(), 8);
    if !callback_di.is_null() && nvim_mapping_dictitem_tv_type(callback_di) == VAR_FUNC {
        let v_string = nvim_mapping_dictitem_tv_vstring(callback_di);
        if !v_string.is_null() {
            let fp = find_func(v_string);
            if !fp.is_null() && (nvim_ufunc_get_flags(fp) & FC_LUAREF) != 0 {
                rhs_lua = api_new_luaref(nvim_ufunc_get_luaref(fp));
                orig_rhs = c"".as_ptr().cast_mut();
            }
        }
    }

    if lhs.is_null() || lhsraw.is_null() || orig_rhs.is_null() {
        nvim_mapping_emsg_entries_missing();
        api_free_luaref(rhs_lua);
        return;
    }

    let sid = tv_dict_get_number(d, c"sid".as_ptr()) as c_int;
    let lnum = tv_dict_get_number(d, c"lnum".as_ptr()) as c_int;
    let buffer = tv_dict_get_number(d, c"buffer".as_ptr()) != 0;

    let noremap: c_int = if tv_dict_get_number(d, c"script".as_ptr()) != 0 {
        REMAP_SCRIPT
    } else if tv_dict_get_number(d, c"noremap".as_ptr()) != 0 {
        REMAP_NONE
    } else {
        0
    };

    let mut args: crate::args::MapArguments = std::mem::zeroed();
    args.expr = tv_dict_get_number(d, c"expr".as_ptr()) != 0;
    args.silent = tv_dict_get_number(d, c"silent".as_ptr()) != 0;
    args.nowait = tv_dict_get_number(d, c"nowait".as_ptr()) != 0;
    args.replace_keycodes = tv_dict_get_number(d, c"replace_keycodes".as_ptr()) != 0;
    args.desc = tv_dict_get_string(d, c"desc".as_ptr(), true);

    let p_cpo = nvim_mapping_get_p_cpo();
    rs_set_maparg_rhs(
        orig_rhs,
        libc::strlen(orig_rhs),
        rhs_lua,
        sid,
        p_cpo,
        &raw mut args,
    );

    // Delete any existing mapping for this lhs and mode.
    let mut unmap_args: crate::args::MapArguments = std::mem::zeroed();
    rs_set_maparg_lhs_rhs(
        lhs,
        libc::strlen(lhs),
        c"".as_ptr(),
        0,
        LUA_NOREF,
        p_cpo,
        &raw mut unmap_args,
    );
    unmap_args.buffer = buffer;
    let curbuf = crate::nvim_get_curbuf();
    rs_buf_do_map(
        MAPTYPE_UNMAP_LHS,
        &raw mut unmap_args,
        mode,
        is_abbr,
        curbuf,
    );
    xfree(unmap_args.rhs);
    xfree(unmap_args.orig_rhs);

    let mp0 = rs_map_add(
        curbuf,
        c_int::from(buffer),
        lhsraw,
        &raw mut args,
        noremap,
        mode,
        is_abbr,
        sid,
        lnum,
        0,
    );
    let mp1 = if lhsrawalt.is_null() {
        std::ptr::null_mut()
    } else {
        rs_map_add(
            curbuf,
            c_int::from(buffer),
            lhsrawalt,
            &raw mut args,
            noremap,
            mode,
            is_abbr,
            sid,
            lnum,
            1,
        )
    };

    if !mp0.is_null() && !mp1.is_null() {
        (*mp0).m_alt = mp1;
        (*mp1).m_alt = mp0;
    }
}

// =============================================================================
// f_maplist
// =============================================================================

/// "maplist()" function -- return a list of all mappings as dicts.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval_T pointers.
#[unsafe(export_name = "f_maplist")]
pub unsafe extern "C" fn rs_f_maplist(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    const FLAGS: c_int = REPTERM_FROM_PART | REPTERM_DO_LT;
    const K_LIST_LEN_UNKNOWN: c_int = -1;

    let av0 = nvim_tv_idx(argvars, 0);
    let abbr = nvim_tv_get_type(av0) != VAR_UNKNOWN && tv_get_bool(av0) != 0;

    tv_list_alloc_ret(rettv, K_LIST_LEN_UNKNOWN);
    let list = nvim_tv_get_list(rettv);

    let curbuf = crate::nvim_get_curbuf();
    let p_cpo = nvim_mapping_get_p_cpo();

    // Do it twice: once for global maps and once for local maps.
    for buffer_local in 0..=1i32 {
        for hash in 0..crate::MAX_MAPHASH {
            let mut mp = if abbr {
                if hash > 0 {
                    break; // only one abbr list
                }
                if buffer_local != 0 {
                    crate::nvim_buf_get_first_abbr(curbuf)
                } else {
                    crate::nvim_get_first_abbr()
                }
            } else if buffer_local != 0 {
                crate::nvim_buf_get_maphash_entry(curbuf, hash)
            } else {
                crate::nvim_get_maphash_entry(hash)
            };

            while !mp.is_null() {
                if (*mp).m_simplified == 0 {
                    let mut keys_buf: *mut c_char = std::ptr::null_mut();
                    let mut did_simplify = false;
                    let mut arena = CArena::empty();

                    let lhs = str2special_arena((*mp).m_keys, true, false, &raw mut arena);
                    replace_termcodes(
                        lhs,
                        libc::strlen(lhs),
                        std::ptr::addr_of_mut!(keys_buf),
                        0,
                        FLAGS,
                        std::ptr::addr_of_mut!(did_simplify),
                        p_cpo,
                    );

                    let dict = rs_mapblock_fill_dict(
                        mp,
                        if did_simplify {
                            keys_buf
                        } else {
                            std::ptr::null()
                        },
                        buffer_local,
                        abbr,
                        true,
                        &raw mut arena,
                    );
                    let dict_obj = Object {
                        obj_type: K_OBJECT_TYPE_DICT,
                        data: nvim_api::ObjectData { dict },
                    };
                    // Convert Dict to typval_T d, then append its v_dict.
                    let mut d_tv = [0u8; TYPVAL_SIZE];
                    object_to_vim_take_luaref(
                        std::ptr::addr_of!(dict_obj),
                        d_tv.as_mut_ptr().cast(),
                        true,
                        std::ptr::null_mut(),
                    );
                    let d_dict = nvim_tv_get_dict(d_tv.as_ptr().cast());
                    tv_list_append_dict(list, d_dict);

                    let mem = arena_finish(&raw mut arena);
                    arena_mem_free(mem);
                    xfree(keys_buf);
                }
                mp = (*mp).m_next;
            }
        }
    }
}
