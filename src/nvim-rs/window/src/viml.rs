//! Window VimL built-in function implementations (f_* functions).
//!
//! Phase 1: Simple window VimL functions (winnr, winheight, tabpagenr, etc.)
//! Phase 2: Window info and view functions (getwininfo, gettabinfo, winsaveview, winrestview)

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::too_many_lines)]

use std::ffi::{c_char, c_int, c_void};

use crate::{win_struct::win_ref, TabpageHandle, WinHandle};

// =============================================================================
// Types
// =============================================================================

/// Opaque pointer to typval_T.
type TypvalPtr = *mut c_void;

/// Opaque handle for EvalFuncData union.
type EvalFuncData = *mut c_void;

/// Opaque pointer to list_T.
type ListPtr = *mut c_void;

/// Opaque pointer to dict_T.
type DictPtr = *mut c_void;

/// Opaque pointer to dictitem_T.
type DictItemPtr = *mut c_void;

/// varnumber_T (matches C int64_t).
type VarNumber = i64;

// =============================================================================
// Constants
// =============================================================================

const VAR_UNKNOWN: c_int = 0;
const VAR_STRING: c_int = 2;
/// kListLenMayKnow sentinel value.
const K_LIST_LEN_MAY_KNOW: c_int = -2;

// =============================================================================
// repr(C) typval_T mirror (layout validated by _Static_assert in eval_shim.c)
// =============================================================================

#[repr(C)]
union TypvalVval {
    v_number: i64,
    v_float: f64,
    v_string: *mut c_char,
    v_list: *mut c_void,
    v_dict: *mut c_void,
    v_partial: *mut c_void,
    v_blob: *mut c_void,
}

#[repr(C)]
struct TypvalT {
    v_type: c_int,
    v_lock: c_int,
    vval: TypvalVval,
}

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    // typval operations
    fn tv_get_number(tv: TypvalPtr) -> VarNumber;
    fn tv_get_number_chk(tv: TypvalPtr, error: *mut bool) -> VarNumber;
    fn tv_get_string_chk(tv: TypvalPtr) -> *const c_char;
    fn tv_list_alloc_ret(rettv: TypvalPtr, len: c_int);
    fn tv_list_append_number(list: ListPtr, nr: VarNumber);
    fn tv_dict_alloc_ret(rettv: TypvalPtr);
    fn tv_dict_alloc() -> DictPtr;
    fn tv_dict_add_nr(dict: DictPtr, key: *const c_char, key_len: usize, nr: VarNumber);
    fn tv_dict_add_dict(dict: DictPtr, key: *const c_char, key_len: usize, val: DictPtr);
    fn tv_dict_add_list(dict: DictPtr, key: *const c_char, key_len: usize, val: ListPtr);
    fn tv_dict_find(dict: DictPtr, key: *const c_char, key_len: c_int) -> DictItemPtr;
    fn tv_list_alloc(len: c_int) -> ListPtr;
    fn tv_list_append_dict(list: ListPtr, dict: DictPtr);
    fn tv_check_for_nonnull_dict_arg(argvars: TypvalPtr, idx: c_int) -> c_int;

    // memory
    fn xmallocz(size: usize) -> *mut c_void;

    // messaging
    fn semsg(fmt: *const c_char, ...) -> bool;

    // window-viml typval shims (window_shim.c)
    fn nvim_eval_tv_idx(argvars: TypvalPtr, i: c_int) -> TypvalPtr;
    fn nvim_eval_tv_set_number(tv: TypvalPtr, n: VarNumber);
    fn nvim_eval_tv_set_string(tv: TypvalPtr, s: *mut c_char);
    fn nvim_eval_tv_set_type(tv: TypvalPtr, t: c_int);
    fn nvim_dictitem_get_tv(di: DictItemPtr) -> TypvalPtr;

    // window/tabpage accessors
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_get_lastwin() -> WinHandle;
    fn nvim_get_curtab() -> TabpageHandle;
    fn nvim_get_first_tabpage() -> TabpageHandle;
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;
    fn nvim_win_get_handle(wp: WinHandle) -> c_int;
    fn nvim_win_get_buffer(wp: WinHandle) -> crate::BufHandle;
    fn nvim_win_get_view_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_view_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_winrow(wp: WinHandle) -> c_int;
    fn nvim_win_get_wincol(wp: WinHandle) -> c_int;
    fn nvim_win_get_wcol(wp: WinHandle) -> c_int;
    fn nvim_win_get_wrow(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_hide(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_focusable(wp: WinHandle) -> c_int;
    fn nvim_win_get_winbar_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_topline(wp: WinHandle) -> c_int;
    fn nvim_win_get_botline(wp: WinHandle) -> c_int;
    fn nvim_win_get_leftcol(wp: WinHandle) -> c_int;
    fn nvim_win_get_skipcol(wp: WinHandle) -> c_int;
    fn nvim_win_get_topfill(wp: WinHandle) -> c_int;
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> c_int;
    fn nvim_win_get_cursor_col(wp: WinHandle) -> c_int;
    fn nvim_win_get_cursor_coladd(wp: WinHandle) -> c_int;
    fn nvim_win_get_curswant(wp: WinHandle) -> c_int;
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;
    fn nvim_tabpage_get_next(tp: TabpageHandle) -> TabpageHandle;
    fn nvim_tabpage_get_curwin(tp: TabpageHandle) -> WinHandle;
    fn nvim_tabpage_get_prevwin(tp: TabpageHandle) -> WinHandle;
    fn nvim_tabpage_get_lastwin(tp: TabpageHandle) -> WinHandle;

    // window vars / tabpage vars (window_shim.c)
    fn nvim_win_get_llist_ref(wp: WinHandle) -> *mut c_void;
    fn nvim_tabpage_get_vars(tp: TabpageHandle) -> DictPtr;

    // buffer accessors
    fn nvim_buf_get_fnum(buf: crate::BufHandle) -> c_int;
    fn rs_bt_terminal(buf: crate::BufHandle) -> bool;
    fn rs_bt_quickfix(buf: crate::BufHandle) -> bool;

    // navigation/update helpers
    fn win_col_off(wp: WinHandle) -> c_int;
    fn nvim_validate_cursor();
    #[link_name = "validate_botline"]
    fn nvim_validate_botline(wp: WinHandle);
    fn update_curswant();

    // view restoration helpers
    #[link_name = "set_topline"]
    fn nvim_set_topline(wp: WinHandle, lnum: c_int);
    fn nvim_check_cursor_win_wrapper(wp: WinHandle);
    #[link_name = "check_topfill"]
    fn nvim_check_topfill(wp: WinHandle, down: c_int);
    fn changed_window_setting(wp: WinHandle);
    fn nvim_win_set_cursor_lnum(wp: WinHandle, lnum: c_int);
    fn nvim_win_set_cursor_col(wp: WinHandle, col: c_int);
    fn nvim_win_set_cursor_coladd(wp: WinHandle, coladd: c_int);
    fn nvim_win_set_curswant(wp: WinHandle, val: c_int);
    fn nvim_win_set_set_curswant(wp: WinHandle, val: c_int);
    fn nvim_win_set_leftcol(wp: WinHandle, val: c_int);
    fn nvim_win_set_skipcol(wp: WinHandle, val: c_int);
    fn nvim_win_set_topfill(wp: WinHandle, val: c_int);
    fn nvim_curbuf_get_ml_line_count() -> c_int;

    // globals
    static cmdwin_type: c_int;
    fn nvim_get_lastused_tabpage() -> TabpageHandle;

    // find window by nr or id (direct call)
    fn find_win_by_nr_or_id(argvars: TypvalPtr) -> WinHandle;

    // tab/window navigation (existing Rust exports callable via #[link_name])
    fn rs_find_tabpage(n: c_int) -> TabpageHandle;
    fn rs_tabpage_index(ftp: TabpageHandle) -> c_int;
    fn rs_valid_tabpage(tpc: TabpageHandle) -> c_int;
    fn rs_win_get_tabwin(id: c_int, tabnr: *mut c_int, winnr: *mut c_int);
    fn rs_win_vert_neighbor(tp: TabpageHandle, wp: WinHandle, up: c_int, count: c_int)
        -> WinHandle;
    fn rs_win_horz_neighbor(
        tp: TabpageHandle,
        wp: WinHandle,
        left: c_int,
        count: c_int,
    ) -> WinHandle;
    fn rs_win_new_height(wp: WinHandle, height: c_int);
    fn rs_win_new_width(wp: WinHandle, width: c_int);

    // error strings
    static e_invexpr2: [c_char; 0];
}

// =============================================================================
// Internal helpers
// =============================================================================

/// Get a pointer to `argvars[i]`.
///
/// # Safety
/// `argvars` must be a valid typval_T array with at least `i+1` entries.
#[inline]
unsafe fn argvar_at(argvars: TypvalPtr, i: c_int) -> TypvalPtr {
    unsafe { nvim_eval_tv_idx(argvars, i) }
}

/// Get the first window for a tabpage, equivalent to `FOR_ALL_WINDOWS_IN_TAB` init.
///
/// For curtab, returns `firstwin`; for other tabs, returns `tp->tp_firstwin`.
///
/// # Safety
/// `tp` must be a valid tabpage handle.
#[inline]
unsafe fn tabpage_firstwin(tp: TabpageHandle) -> WinHandle {
    unsafe {
        if tp == nvim_get_curtab() {
            nvim_get_firstwin()
        } else {
            nvim_tabpage_get_firstwin(tp)
        }
    }
}

/// Return true if `wp` has a visible window number in tabpage `tp`.
///
/// A window has a number if it is the current window of the tab, OR
/// if it is not hidden and is focusable. Replicates `win_has_winnr()` from eval/window.c.
///
/// # Safety
/// `wp` and `tp` must be valid handles.
#[inline]
unsafe fn win_has_winnr(wp: WinHandle, tp: TabpageHandle) -> bool {
    unsafe {
        let curwin_for_tab = if tp == nvim_get_curtab() {
            nvim_get_curwin()
        } else {
            nvim_tabpage_get_curwin(tp)
        };
        wp == curwin_for_tab
            || (nvim_win_get_config_hide(wp) == 0 && nvim_win_get_config_focusable(wp) != 0)
    }
}

/// Common implementation for `get_winnr` (shared by `f_winnr` and `f_tabpagewinnr`).
///
/// Replicates the static `get_winnr()` function from eval/window.c.
///
/// # Safety
/// `tp` and `argvar` must be valid pointers.
unsafe fn get_winnr_impl(tp: TabpageHandle, argvar: TypvalPtr) -> c_int {
    unsafe {
        let mut nr: c_int = 1;

        let curtab = nvim_get_curtab();
        let curwin_for_tab = if tp == curtab {
            nvim_get_curwin()
        } else {
            nvim_tabpage_get_curwin(tp)
        };
        let mut twin = curwin_for_tab;

        if (*argvar.cast::<TypvalT>()).v_type != VAR_UNKNOWN {
            let mut invalid_arg = false;
            let arg = tv_get_string_chk(argvar);
            if arg.is_null() {
                nr = 0; // type error; errmsg already given
            } else {
                use std::ffi::CStr;
                let arg_str = CStr::from_ptr(arg).to_bytes();
                if arg_str == b"$" {
                    twin = if tp == curtab {
                        nvim_get_lastwin()
                    } else {
                        nvim_tabpage_get_lastwin(tp)
                    };
                } else if arg_str == b"#" {
                    twin = nvim_tabpage_get_prevwin(tp);
                    if twin.is_null() {
                        nr = 0;
                    }
                } else {
                    // Parse optional count prefix, e.g. "3j"
                    let mut endp: *mut c_char = std::ptr::null_mut();
                    let count_val = strtol(arg, std::ptr::addr_of_mut!(endp), 10);
                    let count = if count_val <= 0 {
                        1
                    } else {
                        count_val as c_int
                    };

                    if !endp.is_null() && *endp != 0 {
                        let dir_byte = *endp as u8;
                        if dir_byte == b'j' {
                            twin = rs_win_vert_neighbor(tp, twin, 0, count);
                        } else if dir_byte == b'k' {
                            twin = rs_win_vert_neighbor(tp, twin, 1, count);
                        } else if dir_byte == b'h' {
                            twin = rs_win_horz_neighbor(tp, twin, 1, count);
                        } else if dir_byte == b'l' {
                            twin = rs_win_horz_neighbor(tp, twin, 0, count);
                        } else {
                            invalid_arg = true;
                        }
                    } else {
                        invalid_arg = true;
                    }
                }
            }

            if invalid_arg {
                semsg(e_invexpr2.as_ptr(), arg);
                nr = 0;
            }
        } else if !win_has_winnr(twin, tp) {
            nr = 0;
        }

        if nr <= 0 {
            return 0;
        }

        nr = 0;
        let mut wp = tabpage_firstwin(tp);
        while !wp.is_null() {
            nr += win_has_winnr(wp, tp) as c_int;
            if wp == twin {
                break;
            }
            wp = nvim_win_get_next(wp);
        }
        if wp.is_null() {
            nr = 0; // didn't find it in this tabpage
        }
        nr
    }
}

// =============================================================================
// Phase 1: Simple window VimL functions
// =============================================================================

/// "getwinpos({timeout})" function -- always returns [-1, -1] (terminal emulator).
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_getwinpos"]
pub unsafe extern "C" fn rs_f_getwinpos(
    _argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe {
        tv_list_alloc_ret(rettv, 2);
        let list = (*rettv.cast::<TypvalT>()).vval.v_list;
        tv_list_append_number(list, -1);
        tv_list_append_number(list, -1);
    }
}

/// "getwinposx()" function -- always returns -1 (terminal emulator).
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_getwinposx"]
pub unsafe extern "C" fn rs_f_getwinposx(
    _argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe { nvim_eval_tv_set_number(rettv, -1) }
}

/// "getwinposy()" function -- always returns -1 (terminal emulator).
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_getwinposy"]
pub unsafe extern "C" fn rs_f_getwinposy(
    _argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe { nvim_eval_tv_set_number(rettv, -1) }
}

/// "wincol()" function -- returns `curwin->w_wcol + 1`.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_wincol"]
pub unsafe extern "C" fn rs_f_wincol(_argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        nvim_validate_cursor();
        let curwin = nvim_get_curwin();
        nvim_eval_tv_set_number(rettv, i64::from(nvim_win_get_wcol(curwin) + 1));
    }
}

/// "winline()" function -- returns `curwin->w_wrow + 1`.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_winline"]
pub unsafe extern "C" fn rs_f_winline(_argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        nvim_validate_cursor();
        let curwin = nvim_get_curwin();
        nvim_eval_tv_set_number(rettv, i64::from(nvim_win_get_wrow(curwin) + 1));
    }
}

/// "winheight(nr)" function -- returns window's view height or -1.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_winheight"]
pub unsafe extern "C" fn rs_f_winheight(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let wp = find_win_by_nr_or_id(argvars);
        nvim_eval_tv_set_number(
            rettv,
            if wp.is_null() {
                -1
            } else {
                i64::from(nvim_win_get_view_height(wp))
            },
        );
    }
}

/// "winwidth(nr)" function -- returns window's view width or -1.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_winwidth"]
pub unsafe extern "C" fn rs_f_winwidth(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let wp = find_win_by_nr_or_id(argvars);
        nvim_eval_tv_set_number(
            rettv,
            if wp.is_null() {
                -1
            } else {
                i64::from(nvim_win_get_view_width(wp))
            },
        );
    }
}

/// "winbufnr(nr)" function -- returns buffer number for window or -1.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_winbufnr"]
pub unsafe extern "C" fn rs_f_winbufnr(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let wp = find_win_by_nr_or_id(argvars);
        nvim_eval_tv_set_number(
            rettv,
            if wp.is_null() {
                -1
            } else {
                let buf = nvim_win_get_buffer(wp);
                i64::from(nvim_buf_get_fnum(buf))
            },
        );
    }
}

/// "getcmdwintype()" function -- returns the command-window type char as a string.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_getcmdwintype"]
pub unsafe extern "C" fn rs_f_getcmdwintype(
    _argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe {
        let s = xmallocz(1).cast::<c_char>();
        *s = cmdwin_type as c_char;
        nvim_eval_tv_set_type(rettv, VAR_STRING);
        nvim_eval_tv_set_string(rettv, s);
    }
}

/// "win_screenpos({nr})" function -- returns [winrow+1, wincol+1] for window.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_win_screenpos"]
pub unsafe extern "C" fn rs_f_win_screenpos(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe {
        tv_list_alloc_ret(rettv, 2);
        let list = (*rettv.cast::<TypvalT>()).vval.v_list;
        let wp = find_win_by_nr_or_id(argvars);
        tv_list_append_number(
            list,
            if wp.is_null() {
                0
            } else {
                i64::from(nvim_win_get_winrow(wp) + 1)
            },
        );
        tv_list_append_number(
            list,
            if wp.is_null() {
                0
            } else {
                i64::from(nvim_win_get_wincol(wp) + 1)
            },
        );
    }
}

/// "tabpagenr()" function -- returns current/last/prev tabpage number.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_tabpagenr"]
pub unsafe extern "C" fn rs_f_tabpagenr(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let tv0 = argvar_at(argvars, 0);
        let nr = if (*tv0.cast::<TypvalT>()).v_type == VAR_UNKNOWN {
            rs_tabpage_index(nvim_get_curtab())
        } else {
            let arg = tv_get_string_chk(tv0);
            if arg.is_null() {
                0
            } else {
                use std::ffi::CStr;
                let arg_str = CStr::from_ptr(arg).to_bytes();
                if arg_str == b"$" {
                    // rs_tabpage_index(NULL) returns total count + 1
                    rs_tabpage_index(TabpageHandle::null()) - 1
                } else if arg_str == b"#" {
                    let last = nvim_get_lastused_tabpage();
                    if rs_valid_tabpage(last) != 0 {
                        rs_tabpage_index(last)
                    } else {
                        0
                    }
                } else {
                    semsg(e_invexpr2.as_ptr(), arg);
                    0
                }
            }
        };
        nvim_eval_tv_set_number(rettv, i64::from(nr));
    }
}

/// "tabpagewinnr({tabnr})" function -- returns window number in tabpage.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_tabpagewinnr"]
pub unsafe extern "C" fn rs_f_tabpagewinnr(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe {
        let tv0 = argvar_at(argvars, 0);
        let tp = rs_find_tabpage(tv_get_number(tv0) as c_int);
        let nr = if tp.is_null() {
            0
        } else {
            let tv1 = argvar_at(argvars, 1);
            get_winnr_impl(tp, tv1)
        };
        nvim_eval_tv_set_number(rettv, i64::from(nr));
    }
}

/// "win_getid()" function -- returns the window ID for a given window number.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_win_getid"]
pub unsafe extern "C" fn rs_f_win_getid(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        nvim_eval_tv_set_number(rettv, i64::from(win_getid_impl(argvars)));
    }
}

/// Implementation of win_getid logic (the static `win_getid()` from eval/window.c).
///
/// # Safety
/// `argvars` must be a valid typval array.
unsafe fn win_getid_impl(argvars: TypvalPtr) -> c_int {
    unsafe {
        let tv0 = argvar_at(argvars, 0);
        if (*tv0.cast::<TypvalT>()).v_type == VAR_UNKNOWN {
            return nvim_win_get_handle(nvim_get_curwin());
        }
        let winnr = tv_get_number(tv0) as c_int;
        if winnr <= 0 {
            return 0;
        }

        let tv1 = argvar_at(argvars, 1);
        let (tp, first_wp) = if (*tv1.cast::<TypvalT>()).v_type == VAR_UNKNOWN {
            (nvim_get_curtab(), nvim_get_firstwin())
        } else {
            let tabnr = tv_get_number(tv1) as c_int;
            let mut found_tp = TabpageHandle::null();
            let mut tpiter = nvim_get_first_tabpage();
            let mut remaining = tabnr;
            while !tpiter.is_null() {
                remaining -= 1;
                if remaining == 0 {
                    found_tp = tpiter;
                    break;
                }
                tpiter = nvim_tabpage_get_next(tpiter);
            }
            if found_tp.is_null() {
                return -1;
            }
            let first = tabpage_firstwin(found_tp);
            (found_tp, first)
        };

        let mut remaining_winnr = winnr;
        let mut wp = first_wp;
        while !wp.is_null() {
            remaining_winnr -= win_has_winnr(wp, tp) as c_int;
            if remaining_winnr == 0 {
                return nvim_win_get_handle(wp);
            }
            wp = nvim_win_get_next(wp);
        }
        0
    }
}

/// "win_id2tabwin()" function -- returns [tabnr, winnr] for a window ID.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_win_id2tabwin"]
pub unsafe extern "C" fn rs_f_win_id2tabwin(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe {
        let tv0 = argvar_at(argvars, 0);
        let id = tv_get_number(tv0) as c_int;
        let mut tabnr: c_int = 1;
        let mut winnr: c_int = 1;
        rs_win_get_tabwin(
            id,
            std::ptr::addr_of_mut!(tabnr),
            std::ptr::addr_of_mut!(winnr),
        );
        tv_list_alloc_ret(rettv, 2);
        let list = (*rettv.cast::<TypvalT>()).vval.v_list;
        tv_list_append_number(list, i64::from(tabnr));
        tv_list_append_number(list, i64::from(winnr));
    }
}

/// "win_id2win()" function -- returns window number in current tabpage for window ID.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_win_id2win"]
pub unsafe extern "C" fn rs_f_win_id2win(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe {
        let tv0 = argvar_at(argvars, 0);
        let id = tv_get_number(tv0) as c_int;
        let curtab = nvim_get_curtab();
        let mut nr: c_int = 1;
        let mut wp = tabpage_firstwin(curtab);
        let mut result = 0;
        while !wp.is_null() {
            if nvim_win_get_handle(wp) == id {
                result = if win_has_winnr(wp, curtab) { nr } else { 0 };
                break;
            }
            nr += win_has_winnr(wp, curtab) as c_int;
            wp = nvim_win_get_next(wp);
        }
        nvim_eval_tv_set_number(rettv, i64::from(result));
    }
}

/// "win_findbuf()" function -- returns list of window IDs that display a given buffer.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_win_findbuf"]
pub unsafe extern "C" fn rs_f_win_findbuf(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe {
        tv_list_alloc_ret(rettv, K_LIST_LEN_MAY_KNOW);
        let list = (*rettv.cast::<TypvalT>()).vval.v_list;
        let tv0 = argvar_at(argvars, 0);
        let bufnr = tv_get_number(tv0) as c_int;
        // FOR_ALL_TAB_WINDOWS equivalent
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            let mut wp = tabpage_firstwin(tp);
            while !wp.is_null() {
                let buf = nvim_win_get_buffer(wp);
                if nvim_buf_get_fnum(buf) == bufnr {
                    tv_list_append_number(list, i64::from(nvim_win_get_handle(wp)));
                }
                wp = nvim_win_get_next(wp);
            }
            tp = nvim_tabpage_get_next(tp);
        }
    }
}

/// "winnr()" function -- returns window number in the current tabpage.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_winnr"]
pub unsafe extern "C" fn rs_f_winnr(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let curtab = nvim_get_curtab();
        let tv0 = argvar_at(argvars, 0);
        let nr = get_winnr_impl(curtab, tv0);
        nvim_eval_tv_set_number(rettv, i64::from(nr));
    }
}

// =============================================================================
// Phase 2: Window info and view functions
// =============================================================================

/// Build a dict with window information for `getwininfo()`.
///
/// # Safety
/// `wp` must be a valid window handle.
unsafe fn get_win_info_impl(wp: WinHandle, tpnr: i16, winnr: i16) -> DictPtr {
    unsafe {
        let dict = tv_dict_alloc();

        nvim_validate_botline(wp);

        macro_rules! add_nr {
            ($key:literal, $val:expr) => {
                tv_dict_add_nr(
                    dict,
                    concat!($key, "\0").as_ptr().cast::<c_char>(),
                    $key.len(),
                    $val as VarNumber,
                );
            };
        }

        add_nr!("tabnr", tpnr);
        add_nr!("winnr", winnr);
        add_nr!("winid", nvim_win_get_handle(wp));
        add_nr!("height", nvim_win_get_view_height(wp));
        add_nr!("winrow", nvim_win_get_winrow(wp) + 1);
        add_nr!("topline", nvim_win_get_topline(wp));
        add_nr!("botline", nvim_win_get_botline(wp) - 1);
        add_nr!("leftcol", nvim_win_get_leftcol(wp));
        add_nr!("winbar", nvim_win_get_winbar_height(wp));
        add_nr!("width", nvim_win_get_view_width(wp));
        let buf = nvim_win_get_buffer(wp);
        add_nr!("bufnr", nvim_buf_get_fnum(buf));
        add_nr!("wincol", nvim_win_get_wincol(wp) + 1);
        add_nr!("textoff", win_col_off(wp));
        add_nr!("terminal", rs_bt_terminal(buf));
        add_nr!("quickfix", rs_bt_quickfix(buf));
        let llist_ref = nvim_win_get_llist_ref(wp);
        add_nr!(
            "loclist",
            (rs_bt_quickfix(buf) && !llist_ref.is_null()) as c_int
        );

        tv_dict_add_dict(
            dict,
            c"variables".as_ptr(),
            c"variables".to_bytes().len(),
            win_ref(wp).w_vars,
        );

        dict
    }
}

/// Build a dict with tabpage information for `gettabinfo()`.
///
/// # Safety
/// `tp` must be a valid tabpage handle.
unsafe fn get_tabpage_info_impl(tp: TabpageHandle, tp_idx: c_int) -> DictPtr {
    unsafe {
        let dict = tv_dict_alloc();

        tv_dict_add_nr(
            dict,
            c"tabnr".as_ptr(),
            c"tabnr".to_bytes().len(),
            i64::from(tp_idx),
        );

        let l = tv_list_alloc(K_LIST_LEN_MAY_KNOW);
        let mut wp = tabpage_firstwin(tp);
        while !wp.is_null() {
            tv_list_append_number(l, i64::from(nvim_win_get_handle(wp)));
            wp = nvim_win_get_next(wp);
        }
        tv_dict_add_list(dict, c"windows".as_ptr(), c"windows".to_bytes().len(), l);

        tv_dict_add_dict(
            dict,
            c"variables".as_ptr(),
            c"variables".to_bytes().len(),
            nvim_tabpage_get_vars(tp),
        );

        dict
    }
}

/// "gettabinfo()" function -- returns list of tabpage info dicts.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_gettabinfo"]
pub unsafe extern "C" fn rs_f_gettabinfo(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe {
        let tv0 = argvar_at(argvars, 0);
        let tparg = if (*tv0.cast::<TypvalT>()).v_type == VAR_UNKNOWN {
            TabpageHandle::null()
        } else {
            let n = tv_get_number_chk(tv0, std::ptr::null_mut()) as c_int;
            let tp = rs_find_tabpage(n);
            if tp.is_null() {
                // No matching tabpage -- return empty list.
                tv_list_alloc_ret(rettv, 1);
                return;
            }
            tp
        };

        tv_list_alloc_ret(rettv, K_LIST_LEN_MAY_KNOW);
        let list = (*rettv.cast::<TypvalT>()).vval.v_list;

        let mut tpnr: c_int = 0;
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            tpnr += 1;
            if !tparg.is_null() && tp != tparg {
                tp = nvim_tabpage_get_next(tp);
                continue;
            }
            let d = get_tabpage_info_impl(tp, tpnr);
            tv_list_append_dict(list, d);
            if !tparg.is_null() {
                return;
            }
            tp = nvim_tabpage_get_next(tp);
        }
    }
}

/// "getwininfo()" function -- returns list of window info dicts.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_getwininfo"]
pub unsafe extern "C" fn rs_f_getwininfo(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe {
        tv_list_alloc_ret(rettv, K_LIST_LEN_MAY_KNOW);
        let list = (*rettv.cast::<TypvalT>()).vval.v_list;

        let tv0 = argvar_at(argvars, 0);
        let wparg = if (*tv0.cast::<TypvalT>()).v_type == VAR_UNKNOWN {
            WinHandle::null()
        } else {
            let id = tv_get_number(tv0) as c_int;
            let found = win_id2wp_impl(id);
            if found.is_null() {
                return;
            }
            found
        };

        let mut tabnr: i16 = 0;
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            tabnr += 1;
            let mut winnr: i16 = 0;
            let mut wp = tabpage_firstwin(tp);
            while !wp.is_null() {
                winnr += win_has_winnr(wp, tp) as i16;
                if !wparg.is_null() && wp != wparg {
                    wp = nvim_win_get_next(wp);
                    continue;
                }
                let has_nr = win_has_winnr(wp, tp);
                let d = get_win_info_impl(wp, tabnr, if has_nr { winnr } else { 0 });
                tv_list_append_dict(list, d);
                if !wparg.is_null() {
                    return;
                }
                wp = nvim_win_get_next(wp);
            }
            tp = nvim_tabpage_get_next(tp);
        }
    }
}

/// Find a window by handle ID across all tabpages (equivalent to `win_id2wp`).
///
/// # Safety
/// Calls FFI functions.
unsafe fn win_id2wp_impl(id: c_int) -> WinHandle {
    unsafe {
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            let mut wp = tabpage_firstwin(tp);
            while !wp.is_null() {
                if nvim_win_get_handle(wp) == id {
                    return wp;
                }
                wp = nvim_win_get_next(wp);
            }
            tp = nvim_tabpage_get_next(tp);
        }
        WinHandle::null()
    }
}

/// "winsaveview()" function -- saves cursor/scroll state to a dict.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_winsaveview"]
pub unsafe extern "C" fn rs_f_winsaveview(
    _argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe {
        tv_dict_alloc_ret(rettv);
        let dict = (*rettv.cast::<TypvalT>()).vval.v_dict;
        let curwin = nvim_get_curwin();

        macro_rules! add_nr {
            ($key:literal, $val:expr) => {
                tv_dict_add_nr(
                    dict,
                    concat!($key, "\0").as_ptr().cast::<c_char>(),
                    $key.len(),
                    $val as VarNumber,
                );
            };
        }

        add_nr!("lnum", nvim_win_get_cursor_lnum(curwin));
        add_nr!("col", nvim_win_get_cursor_col(curwin));
        add_nr!("coladd", nvim_win_get_cursor_coladd(curwin));
        update_curswant();
        add_nr!("curswant", nvim_win_get_curswant(curwin));
        add_nr!("topline", nvim_win_get_topline(curwin));
        add_nr!("topfill", nvim_win_get_topfill(curwin));
        add_nr!("leftcol", nvim_win_get_leftcol(curwin));
        add_nr!("skipcol", nvim_win_get_skipcol(curwin));
    }
}

/// "winrestview()" function -- restores cursor/scroll state from a dict.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_winrestview"]
pub unsafe extern "C" fn rs_f_winrestview(
    argvars: TypvalPtr,
    _rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe {
        const FAIL: c_int = -1;
        if tv_check_for_nonnull_dict_arg(argvars, 0) == FAIL {
            return;
        }

        let tv0 = argvar_at(argvars, 0);
        let dict = (*tv0.cast::<TypvalT>()).vval.v_dict;
        let curwin = nvim_get_curwin();

        /// Look up a dict key and call setter if found.
        macro_rules! restore_field {
            ($key:literal, $setter:expr) => {{
                let di = tv_dict_find(
                    dict,
                    concat!($key, "\0").as_ptr().cast::<c_char>(),
                    $key.len() as c_int,
                );
                if !di.is_null() {
                    let di_tv = nvim_dictitem_get_tv(di);
                    $setter(curwin, tv_get_number(di_tv) as c_int);
                }
            }};
        }

        restore_field!("lnum", nvim_win_set_cursor_lnum);
        restore_field!("col", nvim_win_set_cursor_col);
        restore_field!("coladd", nvim_win_set_cursor_coladd);

        // curswant: also clear w_set_curswant flag
        {
            let di = tv_dict_find(
                dict,
                c"curswant".as_ptr(),
                c"curswant".to_bytes().len() as c_int,
            );
            if !di.is_null() {
                let di_tv = nvim_dictitem_get_tv(di);
                nvim_win_set_curswant(curwin, tv_get_number(di_tv) as c_int);
                nvim_win_set_set_curswant(curwin, 0); // false
            }
        }

        restore_field!("topline", nvim_set_topline);
        restore_field!("topfill", nvim_win_set_topfill);
        restore_field!("leftcol", nvim_win_set_leftcol);
        restore_field!("skipcol", nvim_win_set_skipcol);

        nvim_check_cursor_win_wrapper(curwin);
        rs_win_new_height(curwin, nvim_win_get_view_height(curwin));
        rs_win_new_width(curwin, nvim_win_get_view_width(curwin));
        changed_window_setting(curwin);

        // Clamp w_topline to [1, line_count]
        let topline = nvim_win_get_topline(curwin);
        if topline <= 0 {
            nvim_set_topline(curwin, 1);
        } else {
            let line_count = nvim_curbuf_get_ml_line_count();
            if topline > line_count {
                nvim_set_topline(curwin, line_count);
            }
        }
        nvim_check_topfill(curwin, 1); // 1 = true (down)
    }
}

/// Thin wrapper around libc strtol for use in get_winnr_impl.
///
/// # Safety
/// `s` must be a valid C string pointer. `endp` may be null.
unsafe fn strtol(s: *const c_char, endp: *mut *mut c_char, base: c_int) -> i64 {
    extern "C" {
        #[link_name = "strtol"]
        fn c_strtol(nptr: *const c_char, endptr: *mut *mut c_char, base: c_int) -> i64;
    }
    unsafe { c_strtol(s, endp, base) }
}
