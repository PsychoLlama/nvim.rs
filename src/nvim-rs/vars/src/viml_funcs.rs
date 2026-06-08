//! VimL f_ functions: getbufvar/setbufvar/getwinvar/setwinvar/etc.
//!
//! Phase 2: Migrated from `src/nvim/eval/vars.c`.
//!
//! Functions:
//! - `rs_get_var_from`: core logic for getwinvar/gettabvar/getbufvar
//! - `rs_f_gettabvar`: gettabvar() VimL function
//! - `rs_f_gettabwinvar`: gettabwinvar() VimL function
//! - `rs_f_getwinvar`: getwinvar() VimL function
//! - `rs_f_getbufvar`: getbufvar() VimL function
//! - `rs_f_settabvar`: settabvar() VimL function
//! - `rs_f_settabwinvar`: settabwinvar() VimL function
//! - `rs_f_setwinvar`: setwinvar() VimL function
//! - `rs_f_setbufvar`: setbufvar() VimL function

#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ref_as_ptr)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::ptr_cast_constness)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::if_not_else)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Opaque handles
// =============================================================================

/// Opaque handle to a `win_T`.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WinHandle(*mut c_void);

impl WinHandle {
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a `buf_T`.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BufHandle(*mut c_void);

impl BufHandle {
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a `tabpage_T`.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TabHandle(*mut c_void);

impl TabHandle {
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a `dict_T`.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct DictHandle(*mut c_void);

impl DictHandle {
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a `hashtab_T`.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct HashtabHandle(*mut c_void);

/// Opaque handle to a `typval_T`.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct TvHandle(*mut c_void);

impl TvHandle {
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// VarType constants (matching C's VAR_* in typval_defs.h)
// =============================================================================

const VAR_UNKNOWN: c_int = 0;
const VAR_STRING: c_int = 2;

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    // --- typval operations ---
    fn tv_get_string_chk(tv: *const c_void) -> *const c_char;
    // tv_get_number_chk uses *mut (TvPtr) in option_conv.rs, so match that signature
    fn tv_get_number_chk(tv: *mut c_void, ret_error: *mut bool) -> i64;
    // tv_copy uses *mut for both args in vimvar_accessors.rs
    fn tv_copy(from: *mut c_void, to: *mut c_void);
    fn tv_check_str_or_nr(tv: *const c_void) -> bool;
    // nvim_tv_get_type uses *mut in option_conv.rs
    fn nvim_tv_get_type(tv: *mut c_void) -> c_int;
    fn nvim_tv_set_type(tv: *mut c_void, t: c_int);
    fn nvim_tv_set_string_val(tv: *mut c_void, s: *mut c_char);
    fn nvim_tv_dict_set_ret(tv: *mut c_void, d: *mut c_void);

    // --- dict/hashtab operations ---
    fn nvim_dict_get_hashtab(dict: *mut c_void) -> *mut c_void;
    // find_var_in_ht: use c_int for no_autoload to match lookup.rs declaration
    fn find_var_in_ht(
        ht: *mut c_void,
        htname: c_int,
        varname: *const c_char,
        varname_len: usize,
        no_autoload: c_int,
    ) -> *mut c_void;
    fn nvim_dictitem_get_tv(di: *mut c_void) -> *mut c_void;

    // --- window/tab/buf accessor shims (struct field accessors in vars.c) ---
    fn nvim_buf_get_vars(buf: *mut c_void) -> *mut c_void;
    fn nvim_win_get_vars(win: *mut c_void) -> *mut c_void;
    fn nvim_tab_get_vars(tp: *mut c_void) -> *mut c_void;
    fn nvim_buf_get_bufvar_tv(buf: *mut c_void) -> *mut c_void;
    fn nvim_win_get_winvar_tv(win: *mut c_void) -> *mut c_void;
    fn nvim_tab_get_winvar_tv(tp: *mut c_void) -> *mut c_void;
    fn nvim_tab_get_firstwin(tp: *mut c_void) -> *mut c_void;

    // --- globals: exported from nvim_window Rust crate (globals.rs) ---
    fn nvim_get_firstwin() -> *mut c_void;
    fn nvim_get_curbuf() -> *mut c_void;
    fn nvim_set_curbuf(buf: *mut c_void);
    fn nvim_get_curwin() -> *mut c_void;
    fn nvim_get_curtab() -> *mut c_void;
    fn nvim_get_lastused_tabpage() -> *mut c_void;
    /// Set lastused_tabpage (exported as nvim_set_lastused_tabpage_from_rust in globals.rs)
    #[link_name = "nvim_set_lastused_tabpage_from_rust"]
    fn nvim_set_lastused_tabpage(tp: *mut c_void);

    fn nvim_emsg_off_inc();
    fn nvim_emsg_off_dec();

    // --- switch_win / restore_win shims ---
    /// Heap-alloc switchwin_T and switch to win/tp; returns NULL on fail.
    fn nvim_vars_switch_win(win: *mut c_void, tp: *mut c_void) -> *mut c_void;
    /// Restore and free heap-allocated switchwin_T.
    fn nvim_vars_switch_win_restore(sw: *mut c_void);
    /// Check if win == curwin && tp == curtab.
    fn nvim_is_curwin_curtab(win: *mut c_void, tp: *mut c_void) -> bool;

    // --- aucmd_prepbuf / restbuf shims (from terminal_shim.c) ---
    fn nvim_aucmd_prepbuf_alloc(buf: *mut c_void) -> *mut c_void;
    fn nvim_aucmd_restbuf_free(aco: *mut c_void);

    // --- goto_tabpage_tp shim ---
    fn nvim_goto_tabpage_tp(tp: *mut c_void, trigger_enter: c_int, trigger_leave: c_int);

    // --- option / other eval functions ---
    fn eval_option(arg: *mut *const c_char, rettv: *mut c_void, evaluate: bool) -> c_int;
    fn get_winbuf_options(bufopt: c_int) -> *mut c_void;
    fn find_win_by_nr(vp: *const c_void, tp: *mut c_void) -> *mut c_void;
    fn tv_get_buf(tv: *const c_void, curtab_only: c_int) -> *mut c_void;
    fn tv_get_buf_from_arg(tv: *const c_void) -> *mut c_void;
    fn set_var(name: *const c_char, name_len: usize, tv: *const c_void, copy: bool);
    fn rs_find_tabpage(n: c_int) -> *mut c_void;
    // rs_check_secure returns c_int (matches eval_helpers.rs declaration)
    fn rs_check_secure() -> c_int;
    fn rs_set_option_from_tv(varname: *const c_char, varp: *mut c_void);
    fn rs_valid_tabpage(tp: *mut c_void) -> c_int;

    // --- memory ---
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);

    // --- string ---
    fn strlen(s: *const c_char) -> usize;
    fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

// =============================================================================
// Constants
// =============================================================================

const OK: c_int = 1;

// =============================================================================
// Implementation
// =============================================================================

/// Core logic for getwinvar/gettabvar/getbufvar.
///
/// Matches C `get_var_from`.
///
/// # Safety
/// All pointer arguments must be valid (or NULL where permitted).
#[no_mangle]
pub unsafe extern "C" fn rs_get_var_from(
    varname: *const c_char,
    rettv: *mut c_void,
    deftv: *mut c_void,
    htname: c_int,
    tp: *mut c_void,
    win: *mut c_void,
    buf: *mut c_void,
) {
    let mut done = false;
    let do_change_curbuf = !buf.is_null() && htname == c_int::from(b'b');

    nvim_emsg_off_inc();

    // rettv->v_type = VAR_STRING; rettv->vval.v_string = NULL;
    nvim_tv_set_type(rettv, VAR_STRING);
    nvim_tv_set_string_val(rettv, std::ptr::null_mut());

    if !varname.is_null()
        && !tp.is_null()
        && !win.is_null()
        && (htname != c_int::from(b'b') || !buf.is_null())
    {
        // Set curwin to be our win, temporarily.  Also set the tabpage,
        // otherwise the window is not valid. Only do this when needed,
        // autocommands get blocked.
        let need_switch_win = !nvim_is_curwin_curtab(win, tp) && !do_change_curbuf;
        let sw: *mut c_void = if need_switch_win {
            nvim_vars_switch_win(win, tp)
        } else {
            std::ptr::null_mut()
        };

        // Proceed if we didn't need to switch, or switch succeeded
        if !need_switch_win || !sw.is_null() {
            let first_char = *varname as u8;
            if first_char == b'&' && htname != c_int::from(b't') {
                let save_curbuf = nvim_get_curbuf();

                // Change curbuf so the option is read from the correct buffer.
                if do_change_curbuf {
                    nvim_set_curbuf(buf);
                }

                let second_char = *varname.add(1) as u8;
                if second_char == 0 {
                    // get all window-local or buffer-local options in a dict
                    let opts = get_winbuf_options((htname == c_int::from(b'b')) as c_int);
                    if !opts.is_null() {
                        nvim_tv_dict_set_ret(rettv, opts);
                        done = true;
                    }
                } else {
                    // Local option - eval_option advances varname
                    let mut vn_ptr: *const c_char = varname;
                    if eval_option(&raw mut vn_ptr, rettv, true) == OK {
                        done = true;
                    }
                }

                nvim_set_curbuf(save_curbuf);
            } else if first_char == 0 {
                // Empty string: return a dict with all the local variables.
                let scope_tv: *mut c_void = if htname == c_int::from(b'b') {
                    nvim_buf_get_bufvar_tv(buf)
                } else if htname == c_int::from(b'w') {
                    nvim_win_get_winvar_tv(win)
                } else {
                    nvim_tab_get_winvar_tv(tp)
                };
                if !scope_tv.is_null() {
                    tv_copy(scope_tv, rettv);
                    done = true;
                }
            } else {
                // Look up the variable.
                let dict: *mut c_void = if htname == c_int::from(b'b') {
                    nvim_buf_get_vars(buf)
                } else if htname == c_int::from(b'w') {
                    nvim_win_get_vars(win)
                } else {
                    nvim_tab_get_vars(tp)
                };
                let ht = nvim_dict_get_hashtab(dict);
                let vn_len = strlen(varname);
                let v = find_var_in_ht(ht, htname, varname, vn_len, 0);
                if !v.is_null() {
                    let tv_ptr = nvim_dictitem_get_tv(v);
                    if !tv_ptr.is_null() {
                        tv_copy(tv_ptr, rettv);
                        done = true;
                    }
                }
            }
        }

        if need_switch_win {
            nvim_vars_switch_win_restore(sw);
        }
    }

    if !done {
        let deftv_type = nvim_tv_get_type(deftv);
        if deftv_type != VAR_UNKNOWN {
            tv_copy(deftv, rettv);
        }
    }

    nvim_emsg_off_dec();
}

/// getwinvar() and gettabwinvar() implementation (off=0 for getwinvar, off=1 for gettabwinvar).
unsafe fn getwinvar(argvars: *mut c_void, rettv: *mut c_void, off: usize) {
    // argvars is an array of typval_T; each is sizeof(typval_T) bytes apart.
    // We access them by offset. But since typval_T is opaque, we use C helpers.
    // sizeof(typval_T) == 16: confirmed by _Static_assert in testing_shim.c.
    const TV_SIZE: usize = 16;

    let tp: *mut c_void = if off == 1 {
        let arg0 = (argvars as *mut u8).add(0) as *mut c_void;
        let n = tv_get_number_chk(arg0, std::ptr::null_mut());
        rs_find_tabpage(n as c_int)
    } else {
        nvim_get_curtab()
    };

    let arg_off = (argvars as *mut u8).add(off * TV_SIZE) as *const c_void;
    let win = find_win_by_nr(arg_off, tp);

    let arg_off_p1 = (argvars as *mut u8).add((off + 1) * TV_SIZE) as *const c_void;
    let varname = tv_get_string_chk(arg_off_p1);

    let arg_off_p2 = (argvars as *mut u8).add((off + 2) * TV_SIZE) as *mut c_void;

    rs_get_var_from(
        varname,
        rettv,
        arg_off_p2,
        c_int::from(b'w'),
        tp,
        win,
        std::ptr::null_mut(),
    );
}

/// setwinvar() and settabwinvar() implementation (off=0 for setwinvar, off=1 for settabwinvar).
unsafe fn setwinvar(argvars: *mut c_void, off: usize) {
    if rs_check_secure() != 0 {
        return;
    }

    // sizeof(typval_T) == 16: confirmed by _Static_assert in testing_shim.c.
    const TV_SIZE: usize = 16;

    let tp: *mut c_void = if off == 1 {
        let arg0 = (argvars as *mut u8).add(0) as *mut c_void;
        let n = tv_get_number_chk(arg0, std::ptr::null_mut());
        rs_find_tabpage(n as c_int)
    } else {
        nvim_get_curtab()
    };

    let arg_off = (argvars as *mut u8).add(off * TV_SIZE) as *const c_void;
    let win = find_win_by_nr(arg_off, tp);

    let arg_off_p1 = (argvars as *mut u8).add((off + 1) * TV_SIZE) as *const c_void;
    let varname = tv_get_string_chk(arg_off_p1);

    let arg_off_p2 = (argvars as *mut u8).add((off + 2) * TV_SIZE) as *mut c_void;
    let varp = arg_off_p2;

    if win.is_null() || varname.is_null() {
        return;
    }

    let need_switch_win = !nvim_is_curwin_curtab(win, tp);
    let sw: *mut c_void = if need_switch_win {
        nvim_vars_switch_win(win, tp)
    } else {
        std::ptr::null_mut()
    };

    if !need_switch_win || !sw.is_null() {
        let first_char = *varname as u8;
        if first_char == b'&' {
            rs_set_option_from_tv(varname.add(1), varp);
        } else {
            let varname_len = strlen(varname);
            // Build "w:<varname>" string
            let winvarname = xmalloc(varname_len + 3) as *mut c_char;
            let prefix = b"w:\0";
            memcpy(
                winvarname as *mut c_void,
                prefix.as_ptr() as *const c_void,
                2,
            );
            memcpy(
                (winvarname as *mut u8).add(2) as *mut c_void,
                varname as *const c_void,
                varname_len + 1,
            );
            set_var(winvarname, varname_len + 2, varp, true);
            xfree(winvarname as *mut c_void);
        }
    }

    if need_switch_win {
        nvim_vars_switch_win_restore(sw);
    }
}

/// "gettabvar()" function.
///
/// # Safety
/// argvars must be a valid typval_T array with at least 3 elements.
#[no_mangle]
pub unsafe extern "C" fn rs_f_gettabvar(argvars: *mut c_void, rettv: *mut c_void) {
    // sizeof(typval_T) == 16: confirmed by _Static_assert in testing_shim.c.
    const TV_SIZE: usize = 16;

    let arg0 = argvars as *mut c_void;
    let arg1 = (argvars as *mut u8).add(TV_SIZE) as *const c_void;
    let arg2 = (argvars as *mut u8).add(2 * TV_SIZE) as *mut c_void;

    let varname = tv_get_string_chk(arg1);
    let tp = rs_find_tabpage(tv_get_number_chk(arg0, std::ptr::null_mut()) as c_int);

    let win: *mut c_void = if !tp.is_null() {
        let curtab = nvim_get_curtab();
        let firstwin_of_tp = nvim_tab_get_firstwin(tp);
        if tp == curtab || firstwin_of_tp.is_null() {
            nvim_get_firstwin()
        } else {
            firstwin_of_tp
        }
    } else {
        std::ptr::null_mut()
    };

    rs_get_var_from(
        varname,
        rettv,
        arg2,
        c_int::from(b't'),
        tp,
        win,
        std::ptr::null_mut(),
    );
}

/// "gettabwinvar()" function.
///
/// # Safety
/// argvars must be a valid typval_T array with at least 4 elements.
#[no_mangle]
pub unsafe extern "C" fn rs_f_gettabwinvar(argvars: *mut c_void, rettv: *mut c_void) {
    getwinvar(argvars, rettv, 1);
}

/// "getwinvar()" function.
///
/// # Safety
/// argvars must be a valid typval_T array with at least 3 elements.
#[no_mangle]
pub unsafe extern "C" fn rs_f_getwinvar(argvars: *mut c_void, rettv: *mut c_void) {
    getwinvar(argvars, rettv, 0);
}

/// "getbufvar()" function.
///
/// # Safety
/// argvars must be a valid typval_T array with at least 3 elements.
#[no_mangle]
pub unsafe extern "C" fn rs_f_getbufvar(argvars: *mut c_void, rettv: *mut c_void) {
    // sizeof(typval_T) == 16: confirmed by _Static_assert in testing_shim.c.
    const TV_SIZE: usize = 16;

    let arg0 = argvars as *const c_void;
    let arg1 = (argvars as *mut u8).add(TV_SIZE) as *const c_void;
    let arg2 = (argvars as *mut u8).add(2 * TV_SIZE) as *mut c_void;

    let varname = tv_get_string_chk(arg1);
    let buf = tv_get_buf_from_arg(arg0);

    let curtab = nvim_get_curtab();
    let curwin = nvim_get_curwin();

    rs_get_var_from(varname, rettv, arg2, c_int::from(b'b'), curtab, curwin, buf);
}

/// "settabvar()" function.
///
/// # Safety
/// argvars must be a valid typval_T array with at least 3 elements.
#[no_mangle]
pub unsafe extern "C" fn rs_f_settabvar(argvars: *mut c_void) {
    if rs_check_secure() != 0 {
        return;
    }

    // sizeof(typval_T) == 16: confirmed by _Static_assert in testing_shim.c.
    const TV_SIZE: usize = 16;
    let arg0 = argvars as *mut c_void;
    let arg1 = (argvars as *mut u8).add(TV_SIZE) as *const c_void;
    let arg2 = (argvars as *mut u8).add(2 * TV_SIZE) as *mut c_void;

    let tp = rs_find_tabpage(tv_get_number_chk(arg0, std::ptr::null_mut()) as c_int);
    let varname = tv_get_string_chk(arg1);
    let varp = arg2;

    if varname.is_null() || tp.is_null() {
        return;
    }

    let save_curtab = nvim_get_curtab();
    let save_lu_tp = nvim_get_lastused_tabpage();
    nvim_goto_tabpage_tp(tp, 0, 0);

    let varname_len = strlen(varname);
    // Build "t:<varname>" string
    let tabvarname = xmalloc(varname_len + 3) as *mut c_char;
    let prefix = b"t:\0";
    memcpy(
        tabvarname as *mut c_void,
        prefix.as_ptr() as *const c_void,
        2,
    );
    memcpy(
        (tabvarname as *mut u8).add(2) as *mut c_void,
        varname as *const c_void,
        varname_len + 1,
    );
    set_var(tabvarname, varname_len + 2, varp, true);
    xfree(tabvarname as *mut c_void);

    // Restore current tabpage and last accessed tabpage.
    if rs_valid_tabpage(save_curtab) != 0 {
        nvim_goto_tabpage_tp(save_curtab, 0, 0);
        if rs_valid_tabpage(save_lu_tp) != 0 {
            nvim_set_lastused_tabpage(save_lu_tp);
        }
    }
}

/// "settabwinvar()" function.
///
/// # Safety
/// argvars must be a valid typval_T array with at least 4 elements.
#[no_mangle]
pub unsafe extern "C" fn rs_f_settabwinvar(argvars: *mut c_void) {
    setwinvar(argvars, 1);
}

/// "setwinvar()" function.
///
/// # Safety
/// argvars must be a valid typval_T array with at least 3 elements.
#[no_mangle]
pub unsafe extern "C" fn rs_f_setwinvar(argvars: *mut c_void) {
    setwinvar(argvars, 0);
}

/// "setbufvar()" function.
///
/// # Safety
/// argvars must be a valid typval_T array with at least 3 elements.
#[no_mangle]
pub unsafe extern "C" fn rs_f_setbufvar(argvars: *mut c_void) {
    // sizeof(typval_T) == 16: confirmed by _Static_assert in testing_shim.c.
    const TV_SIZE: usize = 16;
    let arg0 = argvars as *const c_void;
    let arg1 = (argvars as *mut u8).add(TV_SIZE) as *const c_void;
    let arg2 = (argvars as *mut u8).add(2 * TV_SIZE) as *mut c_void;

    if rs_check_secure() != 0 || !tv_check_str_or_nr(arg0) {
        return;
    }
    let varname = tv_get_string_chk(arg1);
    let buf = tv_get_buf(arg0, 0);
    let varp = arg2;

    if buf.is_null() || varname.is_null() {
        return;
    }

    let first_char = *varname as u8;
    if first_char == b'&' {
        // Set curbuf to be our buf, temporarily.
        let aco = nvim_aucmd_prepbuf_alloc(buf);
        rs_set_option_from_tv(varname.add(1), varp);
        nvim_aucmd_restbuf_free(aco);
    } else {
        let varname_len = strlen(varname);
        // Build "b:<varname>" string
        let bufvarname = xmalloc(varname_len + 3) as *mut c_char;
        let save_curbuf = nvim_get_curbuf();
        nvim_set_curbuf(buf);
        let prefix = b"b:\0";
        memcpy(
            bufvarname as *mut c_void,
            prefix.as_ptr() as *const c_void,
            2,
        );
        memcpy(
            (bufvarname as *mut u8).add(2) as *mut c_void,
            varname as *const c_void,
            varname_len + 1,
        );
        set_var(bufvarname, varname_len + 2, varp, true);
        xfree(bufvarname as *mut c_void);
        nvim_set_curbuf(save_curbuf);
    }
}
