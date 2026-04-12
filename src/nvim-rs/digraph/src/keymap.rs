//! Keymap functions migrated from digraph.c.
//!
//! These functions implement the `'keymap'` option support, historically
//! located in `digraph.c` but logically distinct from digraph operations.

use libc::{c_char, c_int, c_void, size_t};

/// Opaque handle for a buffer (`buf_T*`).
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct BufHandle(*mut c_void);

/// Opaque handle for a window (`win_T*`).
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct WinHandle(*mut c_void);

/// `GarrayT` matching C's `garray_T` layout.
#[repr(C)]
pub struct GarrayT {
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
    pub ga_data: *mut c_void,
}

extern "C" {
    // kmap_T field accessors (added to digraph.c)
    fn nvim_kmap_entry_get_from(entry: *mut c_void) -> *mut c_char;
    fn nvim_kmap_entry_get_to(entry: *mut c_void) -> *mut c_char;
    fn nvim_kmap_entry_size() -> size_t;

    // Buffer field accessors (for get_keymap_str)
    fn nvim_buf_get_b_p_iminsert(buf: BufHandle) -> i64;
    fn nvim_buf_get_b_kmap_state(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_p_keymap(buf: BufHandle) -> *const c_char;

    // Window field accessor
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;

    // Global state accessors (for get_keymap_str)
    fn nvim_get_curbuf_ptr() -> BufHandle;
    fn nvim_set_curbuf_ptr(buf: BufHandle);
    fn nvim_get_curwin_ptr() -> WinHandle;
    fn nvim_set_curwin_ptr(win: WinHandle);

    // emsg_skip increment/decrement
    fn nvim_syn_emsg_skip_inc();
    fn nvim_syn_emsg_skip_dec();

    // eval_to_string(arg, join_list, use_simple_function) -> heap-allocated string
    fn eval_to_string(arg: *mut c_char, join_list: bool, use_simple_function: bool) -> *mut c_char;

    // vim_snprintf(dst, size, fmt, ...)
    fn vim_snprintf(dst: *mut c_char, size: size_t, fmt: *const c_char, ...) -> c_int;

    // xfree / xmalloc
    fn xfree(ptr: *mut c_void);
    fn xmalloc(size: size_t) -> *mut c_void;

    // curbuf kmap accessors (added to digraph.c)
    fn nvim_curbuf_get_b_kmap_state() -> c_int;
    fn nvim_curbuf_clear_b_kmap_state_bits(mask: c_int);
    fn nvim_curbuf_get_b_kmap_ga() -> *mut GarrayT;
    fn nvim_curbuf_get_b_p_keymap() -> *const c_char;

    // p_cpo: getter from Rust window crate, setter added to digraph.c
    fn nvim_get_p_cpo() -> *mut c_char;
    fn nvim_set_p_cpo(val: *mut c_char);
    // ga_clear: already in fold_shim.c as nvim_ga_clear

    // p_enc (added to digraph.c)
    fn nvim_get_p_enc() -> *const c_char;

    // do_map wrapper for lmap operations (added to digraph.c)
    fn nvim_do_map_keymap(maptype: c_int, arg: *mut c_char) -> c_int;

    // ga_clear: already exists in fold_shim.c
    fn nvim_ga_clear(ga: *mut GarrayT);

    // do_cmdline_cmd (already in sign.c as nvim_do_cmdline_cmd_str, void return)
    fn nvim_do_cmdline_cmd_str(cmd: *const c_char);

    // source_runtime(name, flags) -> FAIL/OK (already exported from Rust runtime crate)
    fn source_runtime(fname: *const c_char, flags: c_int) -> c_int;

    // status_redraw_curbuf (already exported from Rust drawscreen crate)
    fn status_redraw_curbuf();
}

/// `B_IMODE_LMAP = 1` (from `buffer_defs.h`).
const B_IMODE_LMAP: i64 = 1;

/// `KEYMAP_LOADED = 2` (from `buffer_defs.h`).
const KEYMAP_LOADED: c_int = 2;

/// `KEYMAP_INIT = 1` (from `buffer_defs.h`).
const KEYMAP_INIT: c_int = 1;

/// `MAPTYPE_MAP = 0` (from `mapping.h`). Used in `ex_loadkeymap` (Phase 3).
#[allow(dead_code)]
const MAPTYPE_MAP: c_int = 0;

/// `MAPTYPE_UNMAP = 1` (from `mapping.h`).
const MAPTYPE_UNMAP: c_int = 1;

/// Maximum length of a keymap `from` or `to` string.
const KMAP_MAXLEN: usize = 20;

/// Return value for `source_runtime` failure.
const FAIL: c_int = 0;

/// Frees the `buf_T.b_kmap_ga` field of a buffer.
///
/// Exported as `keymap_ga_clear` replacing the C symbol.
/// Called from `buffer_shim.c`.
///
/// # Safety
///
/// `kmap_ga` must point to a valid `garray_T` containing `kmap_T` entries, or be null.
#[unsafe(export_name = "keymap_ga_clear")]
pub unsafe extern "C" fn rs_keymap_ga_clear(kmap_ga: *mut GarrayT) {
    if kmap_ga.is_null() {
        return;
    }
    let ga = &*kmap_ga;
    if ga.ga_data.is_null() || ga.ga_len <= 0 {
        return;
    }
    let entry_size = nvim_kmap_entry_size();
    #[allow(clippy::cast_sign_loss)]
    let len = ga.ga_len as usize;
    for i in 0..len {
        // SAFETY: entry_size matches `sizeof(kmap_T)` as returned by the C helper.
        let entry = ga.ga_data.cast::<u8>().add(i * entry_size).cast::<c_void>();
        let from = nvim_kmap_entry_get_from(entry);
        let to = nvim_kmap_entry_get_to(entry);
        if !from.is_null() {
            xfree(from.cast::<c_void>());
        }
        if !to.is_null() {
            xfree(to.cast::<c_void>());
        }
    }
}

/// Stops using `'keymap'`.
///
/// Exported as `keymap_unload` so the still-C `ex_loadkeymap` can call it.
/// Called from `keymap_init` and `ex_loadkeymap`.
///
/// # Safety
///
/// Must be called with `curbuf` pointing to a valid buffer.
#[unsafe(export_name = "keymap_unload")]
pub unsafe extern "C" fn rs_keymap_unload() {
    if nvim_curbuf_get_b_kmap_state() & KEYMAP_LOADED == 0 {
        return;
    }

    // Save p_cpo and set to "C" to avoid line continuation.
    let save_cpo = nvim_get_p_cpo();
    nvim_set_p_cpo(c"C".as_ptr().cast_mut());

    // Clear the ":lmap"s.
    let ga = &*nvim_curbuf_get_b_kmap_ga();
    let entry_size = nvim_kmap_entry_size();
    // Buffer: "<buffer> " (9 chars) + from (KMAP_MAXLEN) + NUL = 30 bytes
    let buf_size = KMAP_MAXLEN + 10;
    #[allow(clippy::cast_sign_loss)]
    let len = ga.ga_len as usize;
    for i in 0..len {
        let entry = ga.ga_data.cast::<u8>().add(i * entry_size).cast::<c_void>();
        let from = nvim_kmap_entry_get_from(entry);
        // Build "<buffer> <from>" and unmap it.
        let cmd_buf = xmalloc(buf_size + 1).cast::<c_char>();
        vim_snprintf(cmd_buf, buf_size + 1, c"<buffer> %s".as_ptr(), from);
        nvim_do_map_keymap(MAPTYPE_UNMAP, cmd_buf);
        xfree(cmd_buf.cast::<c_void>());
    }

    // Free from/to strings in the garray.
    rs_keymap_ga_clear(nvim_curbuf_get_b_kmap_ga());

    nvim_set_p_cpo(save_cpo);

    nvim_ga_clear(nvim_curbuf_get_b_kmap_ga());
    nvim_curbuf_clear_b_kmap_state_bits(KEYMAP_LOADED);
    status_redraw_curbuf();
}

/// Sets up key mapping tables for the `'keymap'` option.
///
/// Exported as `keymap_init` replacing the C symbol.
/// Called from the `buffer`, `optionstr`, and `ex_cmds` Rust crates.
///
/// # Returns
/// Null pointer on success, or a pointer to a static error string on failure.
///
/// # Safety
///
/// Must be called with `curbuf` pointing to a valid buffer.
#[must_use]
#[unsafe(export_name = "keymap_init")]
pub unsafe extern "C" fn rs_keymap_init() -> *mut c_char {
    nvim_curbuf_clear_b_kmap_state_bits(KEYMAP_INIT);

    let b_p_keymap = nvim_curbuf_get_b_p_keymap();
    if b_p_keymap.is_null() || *b_p_keymap == 0 {
        // Stop any active keymap and clear the table.
        // Also remove b:keymap_name, as no keymap is active now.
        rs_keymap_unload();
        nvim_do_cmdline_cmd_str(c"unlet! b:keymap_name".as_ptr());
    } else {
        // Source the keymap file. It will contain a ":loadkeymap" command.
        let p_enc = nvim_get_p_enc();
        // Calculate buffer size: "keymap/" + keymap_name + "_" + encoding + ".vim" + NUL
        let keymap_len = libc::strlen(b_p_keymap);
        let enc_len = libc::strlen(p_enc);
        // "keymap/" (7) + keymap_name + "_" (1) + encoding + ".vim" (4) + NUL (1) = +13
        let buflen = keymap_len + enc_len + 14;
        let buf = xmalloc(buflen).cast::<c_char>();

        // Try "keymap/<keymap>_<encoding>.vim" first.
        vim_snprintf(buf, buflen, c"keymap/%s_%s.vim".as_ptr(), b_p_keymap, p_enc);

        if source_runtime(buf, 0) == FAIL {
            // Try "keymap/<keymap>.vim" without encoding.
            vim_snprintf(buf, buflen, c"keymap/%s.vim".as_ptr(), b_p_keymap);

            if source_runtime(buf, 0) == FAIL {
                xfree(buf.cast::<c_void>());
                // Return static error string (not heap-allocated; caller must not free).
                return c"E544: Keymap file not found".as_ptr().cast_mut();
            }
        }
        xfree(buf.cast::<c_void>());
    }

    std::ptr::null_mut()
}

/// Gets the value to show for the language mappings / active `'keymap'`.
///
/// Exported as `get_keymap_str` replacing the C symbol.
/// Called from `statusline.c` and the `drawscreen` Rust crate.
///
/// # Parameters
/// - `wp`: window handle
/// - `fmt`: format string containing one `%s` item
/// - `buf`: output buffer
/// - `len`: length of output buffer
///
/// # Returns
/// Length of the formatted string, or 0 if inactive.
///
/// # Safety
///
/// `wp`, `fmt`, and `buf` must be valid non-null pointers. `buf` must have
/// capacity of at least `len` bytes.
#[unsafe(export_name = "get_keymap_str")]
pub unsafe extern "C" fn rs_get_keymap_str(
    wp: WinHandle,
    fmt: *const c_char,
    buf: *mut c_char,
    len: c_int,
) -> c_int {
    let win_buf = nvim_win_get_buffer(wp);
    if nvim_buf_get_b_p_iminsert(win_buf) != B_IMODE_LMAP {
        return 0;
    }

    let old_curbuf = nvim_get_curbuf_ptr();
    let old_curwin = nvim_get_curwin_ptr();

    nvim_set_curbuf_ptr(win_buf);
    nvim_set_curwin_ptr(wp);
    nvim_syn_emsg_skip_inc();

    // eval_to_string takes a *mut c_char; the C string is not modified in practice.
    let to_evaluate = c"b:keymap_name".as_ptr().cast_mut();
    let s = eval_to_string(to_evaluate, false, false);

    nvim_syn_emsg_skip_dec();
    nvim_set_curbuf_ptr(old_curbuf);
    nvim_set_curwin_ptr(old_curwin);

    // Choose which name to display.
    let p: *const c_char = if s.is_null() || *s == 0 {
        let kmap_state = nvim_buf_get_b_kmap_state(win_buf);
        if kmap_state & KEYMAP_LOADED != 0 {
            nvim_buf_get_b_p_keymap(win_buf)
        } else {
            c"lang".as_ptr()
        }
    } else {
        s
    };

    #[allow(clippy::cast_sign_loss)]
    let plen = vim_snprintf(buf, len as size_t, fmt, p);
    xfree(s.cast::<c_void>());

    if plen < 0 || plen > len - 1 {
        *buf = 0;
        return 0;
    }
    plen
}
