//! vim.c API function implementations in Rust
//!
//! This module provides Rust implementations of top-level nvim_* API functions
//! that were originally in src/nvim/api/vim.c.

#![allow(clippy::missing_safety_doc)]

use crate::{Arena, Array, Dict, NvimString, Object};
use std::ffi::{c_char, c_int};

// ---------------------------------------------------------------------------
// C FFI declarations
// ---------------------------------------------------------------------------

extern "C" {
    // Current buffer/window/tab handles
    fn rs_curbuf_handle() -> c_int;
    fn rs_curwin_handle() -> c_int;
    fn rs_curtab_handle() -> c_int;

    // Buffer/window/tab iteration (callback-based)
    fn rs_for_each_buf_handle(cb: unsafe extern "C" fn(c_int, *mut Array), ud: *mut Array);
    fn rs_for_each_win_handle(cb: unsafe extern "C" fn(c_int, *mut Array), ud: *mut Array);
    fn rs_for_each_tab_handle(cb: unsafe extern "C" fn(c_int, *mut Array), ud: *mut Array);

    // Stats accessors
    fn rs_g_stats_fsync() -> i64;
    fn rs_g_stats_log_skip() -> i64;
    fn rs_g_stats_redraw() -> i64;
    fn rs_arena_alloc_count_get() -> i64;
    fn rs_tslua_query_parse_count_get() -> i64;
    fn nlua_get_global_ref_count() -> c_int;

    // Color table accessor: iterates by index; returns false at end-of-table
    fn rs_color_name_get(idx: usize, name_out: *mut *const c_char, rgb_out: *mut i32) -> bool;

    // Array/Dict allocation
    fn arena_array(arena: *mut Arena, max_size: usize) -> Array;
    fn arena_dict(arena: *mut Arena, max_size: usize) -> Dict;

    // Object helpers already in lib.rs but needed here too
    fn copy_object(obj: Object, arena: *mut Arena) -> Object;
    fn copy_array(arr: Array, arena: *mut Arena) -> Array;
    fn copy_dict(dict: Dict, arena: *mut Arena) -> Dict;

    // String helpers
    fn get_lib_dir() -> *mut c_char;
    fn cstr_as_string(s: *mut c_char) -> NvimString;

    // UI / channel info
    fn ui_array(arena: *mut Arena) -> Array;
    fn channel_all_info(arena: *mut Arena) -> Array;

    // MB string width
    fn mb_string2cells(s: *const c_char) -> usize;

    // Color by name
    fn name_to_color(name: *const c_char, idx: *mut c_int) -> i32;

    // Highlight group
    fn syn_check_group(name: *const c_char, len: usize) -> c_int;

    // error helpers
    fn api_set_error(err: *mut crate::Error, err_type: c_int, fmt: *const c_char, ...);

    // Color table size (number of valid entries)
    fn rs_color_name_table_size() -> usize;

    // C string length
    fn strlen(s: *const c_char) -> usize;
}

// ---------------------------------------------------------------------------
// Phase 1 — trivial getters, listers, color, test helpers
// ---------------------------------------------------------------------------

// Buffer/win/tab handle push callbacks used with rs_for_each_*
unsafe extern "C" fn push_buf_handle(handle: c_int, arr: *mut Array) {
    // Object type 8 = kObjectTypeBuffer
    let obj = Object {
        obj_type: 8,
        data: crate::ObjectData {
            integer: handle as i64,
        },
    };
    (*arr).push(obj);
}

unsafe extern "C" fn push_win_handle(handle: c_int, arr: *mut Array) {
    // Object type 9 = kObjectTypeWindow
    let obj = Object {
        obj_type: 9,
        data: crate::ObjectData {
            integer: handle as i64,
        },
    };
    (*arr).push(obj);
}

unsafe extern "C" fn push_tab_handle(handle: c_int, arr: *mut Array) {
    // Object type 10 = kObjectTypeTabpage
    let obj = Object {
        obj_type: 10,
        data: crate::ObjectData {
            integer: handle as i64,
        },
    };
    (*arr).push(obj);
}

/// Gets the current buffer handle.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_get_current_buf() -> c_int {
    rs_curbuf_handle()
}

/// Gets the current window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_get_current_win() -> c_int {
    rs_curwin_handle()
}

/// Gets the current tabpage handle.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_get_current_tabpage() -> c_int {
    rs_curtab_handle()
}

/// Gets the list of all buffers (including unlisted/unloaded).
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_list_bufs(arena: *mut Arena) -> Array {
    // Count first pass via a temporary counter
    let mut count: usize = 0;
    unsafe extern "C" fn count_buf(_handle: c_int, ud: *mut Array) {
        unsafe {
            let c = &mut *(ud as *mut usize);
            *c += 1;
        }
    }
    rs_for_each_buf_handle(count_buf, &mut count as *mut usize as *mut Array);

    let mut rv = arena_array(arena, count);
    rs_for_each_buf_handle(push_buf_handle, &mut rv);
    rv
}

/// Gets the current list of all window IDs in all tabpages.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_list_wins(arena: *mut Arena) -> Array {
    let mut count: usize = 0;
    unsafe extern "C" fn count_win(_handle: c_int, ud: *mut Array) {
        unsafe {
            let c = &mut *(ud as *mut usize);
            *c += 1;
        }
    }
    rs_for_each_win_handle(count_win, &mut count as *mut usize as *mut Array);

    let mut rv = arena_array(arena, count);
    rs_for_each_win_handle(push_win_handle, &mut rv);
    rv
}

/// Gets the current list of all tabpage IDs.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_list_tabpages(arena: *mut Arena) -> Array {
    let mut count: usize = 0;
    unsafe extern "C" fn count_tab(_handle: c_int, ud: *mut Array) {
        unsafe {
            let c = &mut *(ud as *mut usize);
            *c += 1;
        }
    }
    rs_for_each_tab_handle(count_tab, &mut count as *mut usize as *mut Array);

    let mut rv = arena_array(arena, count);
    rs_for_each_tab_handle(push_tab_handle, &mut rv);
    rv
}

/// Gets a list of dicts representing attached UIs.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_list_uis(arena: *mut Arena) -> Array {
    ui_array(arena)
}

/// Gets information about all open channels.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_list_chans(arena: *mut Arena) -> Array {
    channel_all_info(arena)
}

/// Calculates display width of text.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_strwidth(
    text_data: *const c_char,
    text_size: usize,
    err: *mut crate::Error,
) -> i64 {
    if text_size > i32::MAX as usize {
        // VALIDATE_S equivalent
        api_set_error(
            err,
            1, // kErrorTypeValidation
            c"Invalid '%s': '%s'".as_ptr(),
            c"text length".as_ptr(),
            c"(too long)".as_ptr(),
        );
        return 0;
    }
    mb_string2cells(text_data) as i64
}

/// Returns the 24-bit RGB value of a color name.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_get_color_by_name(name: *const c_char) -> i64 {
    let mut dummy: c_int = 0;
    name_to_color(name, &mut dummy) as i64
}

/// Returns a map of color names and RGB values.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_get_color_map(arena: *mut Arena) -> Dict {
    let count = rs_color_name_table_size();
    let mut colors = arena_dict(arena, count);
    let mut i: usize = 0;
    loop {
        let mut name: *const c_char = std::ptr::null();
        let mut rgb: i32 = 0;
        if !rs_color_name_get(i, &mut name, &mut rgb) {
            break;
        }
        let obj = Object {
            obj_type: 2, // kObjectTypeInteger
            data: crate::ObjectData {
                integer: rgb as i64,
            },
        };
        colors.put_static(name, obj);
        i += 1;
    }
    colors
}

/// Returns object given as argument (testing).
#[no_mangle]
pub unsafe extern "C" fn rs_nvim__id(obj: Object, arena: *mut Arena) -> Object {
    copy_object(obj, arena)
}

/// Returns array given as argument (testing).
#[no_mangle]
pub unsafe extern "C" fn rs_nvim__id_array(arr: Array, arena: *mut Arena) -> Array {
    copy_array(arr, arena)
}

/// Returns dict given as argument (testing).
#[no_mangle]
pub unsafe extern "C" fn rs_nvim__id_dict(dct: Dict, arena: *mut Arena) -> Dict {
    copy_dict(dct, arena)
}

/// Returns floating-point value given as argument (testing).
#[no_mangle]
pub extern "C" fn rs_nvim__id_float(flt: f64) -> f64 {
    flt
}

/// Gets internal stats.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim__stats(arena: *mut Arena) -> Dict {
    let mut rv = arena_dict(arena, 6);

    rv.put_static(c"fsync".as_ptr(), Object::integer(rs_g_stats_fsync()));
    rv.put_static(c"log_skip".as_ptr(), Object::integer(rs_g_stats_log_skip()));
    rv.put_static(
        c"lua_refcount".as_ptr(),
        Object::integer(nlua_get_global_ref_count() as i64),
    );
    rv.put_static(c"redraw".as_ptr(), Object::integer(rs_g_stats_redraw()));
    rv.put_static(
        c"arena_alloc_count".as_ptr(),
        Object::integer(rs_arena_alloc_count_get()),
    );
    rv.put_static(
        c"ts_query_parse_count".as_ptr(),
        Object::integer(rs_tslua_query_parse_count_get()),
    );

    rv
}

/// Gets a highlight group by name (allocates new ID if not present).
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_get_hl_id_by_name(
    name_data: *const c_char,
    name_size: usize,
) -> i64 {
    syn_check_group(name_data, name_size) as i64
}

/// Gets the lib directory path.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim__get_lib_dir() -> NvimString {
    cstr_as_string(get_lib_dir())
}

// ---------------------------------------------------------------------------
// Phase 2 — variables, current line/dir, mode, keymap, mark, chan_info
// ---------------------------------------------------------------------------

extern "C" {
    // g:var / v:var C thunks
    fn rs_nvim_get_var_impl(name: NvimString, arena: *mut Arena, err: *mut crate::Error) -> Object;
    fn rs_nvim_set_var_impl(name: NvimString, value: Object, err: *mut crate::Error);
    fn rs_nvim_del_var_impl(name: NvimString, err: *mut crate::Error);
    fn rs_nvim_get_vvar_impl(name: NvimString, arena: *mut Arena, err: *mut crate::Error)
        -> Object;
    fn rs_nvim_set_vvar_impl(name: NvimString, value: Object, err: *mut crate::Error);

    // current dir
    fn rs_nvim_set_current_dir_impl(
        dir_data: *const c_char,
        dir_size: usize,
        err: *mut crate::Error,
    );

    // current line C thunks
    fn rs_nvim_get_current_line_impl(arena: *mut Arena, err: *mut crate::Error) -> NvimString;
    fn rs_nvim_set_current_line_impl(line: NvimString, arena: *mut Arena, err: *mut crate::Error);
    fn rs_nvim_del_current_line_impl(arena: *mut Arena, err: *mut crate::Error);

    // mode
    fn rs_nvim_get_mode_impl(mode_out: *mut c_char);
    fn rs_nvim_input_blocking() -> bool;
    fn rs_mode_max_length() -> c_int;

    // keymap
    fn rs_nvim_get_keymap_impl(mode: NvimString, arena: *mut Arena) -> Array;
    fn rs_nvim_set_keymap_impl(
        channel_id: u64,
        mode: NvimString,
        lhs: NvimString,
        rhs: NvimString,
        opts: *mut KeymapOpts,
        err: *mut crate::Error,
    );
    fn rs_nvim_del_keymap_impl(
        channel_id: u64,
        mode: NvimString,
        lhs: NvimString,
        err: *mut crate::Error,
    );

    // marks
    fn rs_nvim_del_mark_impl(name: NvimString, err: *mut crate::Error) -> bool;
    fn rs_nvim_get_mark_impl(name: NvimString, arena: *mut Arena, err: *mut crate::Error) -> Array;

    // channel info
    fn rs_nvim_get_chan_info_impl(channel_id: u64, chan: i64, arena: *mut Arena) -> Dict;

    // arena alloc (for mode string)
    fn rs_arena_alloc_raw(arena: *mut Arena, size: usize) -> *mut c_char;
}

// Opaque type for Dict(keymap) — only passed through as pointer
#[repr(C)]
pub struct KeymapOpts {
    _private: [u8; 0],
}

/// Gets a global (g:) variable.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_get_var(
    name: NvimString,
    arena: *mut Arena,
    err: *mut crate::Error,
) -> Object {
    rs_nvim_get_var_impl(name, arena, err)
}

/// Sets a global (g:) variable.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_set_var(name: NvimString, value: Object, err: *mut crate::Error) {
    rs_nvim_set_var_impl(name, value, err);
}

/// Removes a global (g:) variable.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_del_var(name: NvimString, err: *mut crate::Error) {
    rs_nvim_del_var_impl(name, err);
}

/// Gets a v: variable.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_get_vvar(
    name: NvimString,
    arena: *mut Arena,
    err: *mut crate::Error,
) -> Object {
    rs_nvim_get_vvar_impl(name, arena, err)
}

/// Sets a v: variable (if not readonly).
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_set_vvar(name: NvimString, value: Object, err: *mut crate::Error) {
    rs_nvim_set_vvar_impl(name, value, err);
}

/// Changes the global working directory.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_set_current_dir(dir: NvimString, err: *mut crate::Error) {
    rs_nvim_set_current_dir_impl(dir.data, dir.size, err);
}

/// Gets the current line.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_get_current_line(
    arena: *mut Arena,
    err: *mut crate::Error,
) -> NvimString {
    rs_nvim_get_current_line_impl(arena, err)
}

/// Sets the text on the current line.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_set_current_line(
    line: NvimString,
    arena: *mut Arena,
    err: *mut crate::Error,
) {
    rs_nvim_set_current_line_impl(line, arena, err);
}

/// Deletes the current line.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_del_current_line(arena: *mut Arena, err: *mut crate::Error) {
    rs_nvim_del_current_line_impl(arena, err);
}

/// Gets the current mode.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_get_mode(arena: *mut Arena) -> Dict {
    let len = rs_mode_max_length() as usize;
    let modestr = rs_arena_alloc_raw(arena, len);
    rs_nvim_get_mode_impl(modestr);
    let blocked = rs_nvim_input_blocking();

    let mut rv = arena_dict(arena, 2);
    let mode_obj = Object {
        obj_type: 4, // kObjectTypeString
        data: crate::ObjectData {
            string: NvimString {
                data: modestr,
                size: strlen(modestr),
            },
        },
    };
    rv.put_static(c"mode".as_ptr(), mode_obj);
    rv.put_static(
        c"blocking".as_ptr(),
        Object {
            obj_type: 1, // kObjectTypeBoolean
            data: crate::ObjectData { boolean: blocked },
        },
    );
    rv
}

/// Gets a list of global (non-buffer-local) mapping definitions.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_get_keymap(mode: NvimString, arena: *mut Arena) -> Array {
    rs_nvim_get_keymap_impl(mode, arena)
}

/// Sets a global mapping for the given mode.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_set_keymap(
    channel_id: u64,
    mode: NvimString,
    lhs: NvimString,
    rhs: NvimString,
    opts: *mut KeymapOpts,
    err: *mut crate::Error,
) {
    rs_nvim_set_keymap_impl(channel_id, mode, lhs, rhs, opts, err);
}

/// Unmaps a global mapping for the given mode.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_del_keymap(
    channel_id: u64,
    mode: NvimString,
    lhs: NvimString,
    err: *mut crate::Error,
) {
    rs_nvim_del_keymap_impl(channel_id, mode, lhs, err);
}

/// Deletes an uppercase/file named mark.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_del_mark(name: NvimString, err: *mut crate::Error) -> bool {
    // Validation: must be a single char, uppercase or digit
    if name.size != 1 {
        api_set_error(
            err,
            1, // kErrorTypeValidation
            c"Invalid 'mark name (must be a single char)': '%s'".as_ptr(),
            name.data,
        );
        return false;
    }
    let ch = *name.data as u8;
    if !ch.is_ascii_uppercase() && !ch.is_ascii_digit() {
        api_set_error(
            err,
            1,
            c"Invalid 'mark name (must be file/uppercase)': '%s'".as_ptr(),
            name.data,
        );
        return false;
    }
    rs_nvim_del_mark_impl(name, err)
}

/// Returns a (row, col, buffer, buffername) tuple for the named mark.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_get_mark(
    name: NvimString,
    arena: *mut Arena,
    err: *mut crate::Error,
) -> Array {
    let rv = Array::empty();
    if name.size != 1 {
        api_set_error(
            err,
            1,
            c"Invalid 'mark name (must be a single char)': '%s'".as_ptr(),
            name.data,
        );
        return rv;
    }
    let ch = *name.data as u8;
    if !ch.is_ascii_uppercase() && !ch.is_ascii_digit() {
        api_set_error(
            err,
            1,
            c"Invalid 'mark name (must be file/uppercase)': '%s'".as_ptr(),
            name.data,
        );
        return rv;
    }
    rs_nvim_get_mark_impl(name, arena, err)
}

/// Gets information about a channel.
#[no_mangle]
pub unsafe extern "C" fn rs_nvim_get_chan_info(
    channel_id: u64,
    chan: i64,
    arena: *mut Arena,
    _err: *mut crate::Error,
) -> Dict {
    if chan < 0 {
        return Dict::empty();
    }
    rs_nvim_get_chan_info_impl(channel_id, chan, arena)
}
