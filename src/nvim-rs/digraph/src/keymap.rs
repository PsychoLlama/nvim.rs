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

    // Buffer field accessors
    fn nvim_buf_get_b_p_iminsert(buf: BufHandle) -> i64;
    fn nvim_buf_get_b_kmap_state(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_p_keymap(buf: BufHandle) -> *const c_char;

    // Window field accessors
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;

    // Global state accessors
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

    // xfree
    fn xfree(ptr: *mut c_void);
}

/// `B_IMODE_LMAP = 1` (from `buffer_defs.h`).
const B_IMODE_LMAP: i64 = 1;

/// `KEYMAP_LOADED = 2` (from `buffer_defs.h`).
const KEYMAP_LOADED: c_int = 2;

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
