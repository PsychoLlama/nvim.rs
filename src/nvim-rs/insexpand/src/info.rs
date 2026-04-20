//! Complete info support (complete_info() VimL function).
//!
//! This module provides the Rust implementation of complete_info(), replacing
//! the C nvim_get_complete_info_impl function.

use std::os::raw::{c_char, c_int, c_void};

use crate::match_list::{
    is_first_match, nvim_compl_get_curr_match, nvim_compl_get_first_match, ComplMatch,
};

// what_flag bitmasks
const CI_WHAT_MODE: c_int = 0x01;
const CI_WHAT_PUM_VISIBLE: c_int = 0x02;
const CI_WHAT_ITEMS: c_int = 0x04;
const CI_WHAT_SELECTED: c_int = 0x08;
const CI_WHAT_COMPLETED: c_int = 0x10;
const CI_WHAT_MATCHES: c_int = 0x20;
const CI_WHAT_PREINSERTED_TEXT: c_int = 0x40;
const CI_WHAT_ALL: c_int = 0xff;

// VimType enum value for VAR_UNKNOWN
const VAR_UNKNOWN: c_int = 0;

// typval_T layout for argument parsing
// sizeof = 16; v_type at offset 0, vval (union) at offset 8
#[repr(C)]
union TypvalVvalInfo {
    v_number: i64,
    v_list: *mut c_void,
    v_dict: *mut c_void,
}

#[repr(C)]
struct TypvalTInfo {
    v_type: c_int,
    v_lock: c_int,
    vval: TypvalVvalInfo,
}

extern "C" {
    // List item iteration (from quickfix_shim.c -- already exported)
    fn nvim_tv_list_first(list: *const c_void) -> *mut c_void;
    fn nvim_tv_list_item_next(list: *const c_void, item: *const c_void) -> *mut c_void;
    fn nvim_tv_list_item_string(item: *const c_void) -> *mut c_char;

    // Dict/list operations (nvim_ci_ prefix avoids name conflicts)
    fn nvim_ci_dict_alloc() -> *mut c_void;
    fn nvim_ci_list_alloc_known() -> *mut c_void;
    fn nvim_ci_dict_add_str(
        d: *mut c_void,
        key: *const c_char,
        klen: usize,
        val: *const c_char,
    ) -> c_int;
    fn nvim_ci_dict_add_str_len(
        d: *mut c_void,
        key: *const c_char,
        klen: usize,
        val: *const c_char,
        vlen: c_int,
    ) -> c_int;
    fn nvim_ci_dict_add_nr(d: *mut c_void, key: *const c_char, klen: usize, nr: i64) -> c_int;
    fn nvim_ci_dict_add_bool(d: *mut c_void, key: *const c_char, klen: usize, val: c_int) -> c_int;
    fn nvim_ci_dict_add_tv(
        d: *mut c_void,
        key: *const c_char,
        klen: usize,
        tv: *mut c_void,
    ) -> c_int;
    fn nvim_ci_dict_add_dict(
        d: *mut c_void,
        key: *const c_char,
        klen: usize,
        val: *mut c_void,
    ) -> c_int;
    fn nvim_ci_dict_add_list(
        d: *mut c_void,
        key: *const c_char,
        klen: usize,
        list: *mut c_void,
    ) -> c_int;
    fn nvim_ci_list_append_dict(list: *mut c_void, dict: *mut c_void);

    // Pum / preview
    #[link_name = "pum_visible"]
    fn nvim_pum_visible() -> c_int;
    fn nvim_win_float_find_preview() -> *mut c_void;
    fn nvim_ci_win_get_handle(wp: *mut c_void) -> c_int;
    fn nvim_ci_win_get_buf_handle(wp: *mut c_void) -> c_int;

    // compl_T field accessors
    fn nvim_compl_match_get_cp_str_data(m: ComplMatch) -> *const c_char;
    fn nvim_compl_match_get_cp_text_abbr(m: ComplMatch) -> *const c_char;
    fn nvim_compl_match_get_cp_text_menu(m: ComplMatch) -> *const c_char;
    fn nvim_compl_match_get_cp_text_kind(m: ComplMatch) -> *const c_char;
    fn nvim_compl_match_get_cp_text_info(m: ComplMatch) -> *const c_char;
    fn nvim_compl_match_get_in_match_array(m: ComplMatch) -> c_int;
    fn nvim_compl_match_user_data_is_unknown(m: ComplMatch) -> c_int;
    fn nvim_compl_match_copy_user_data_tv(m: ComplMatch, dest_tv: *mut c_void);
    fn nvim_compl_match_get_cp_number(m: ComplMatch) -> c_int;
    fn nvim_compl_match_get_next(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_at_original_text(m: ComplMatch) -> c_int;

    // Update sequence numbers (when cp_number == -1)
    fn rs_ins_compl_update_sequence_numbers();
    // ins_compl_mode string
    fn rs_ins_compl_mode() -> *const c_char;

    // preinserted_text
    fn nvim_ci_preinserted_text_ptr() -> *const c_char;
    fn nvim_ci_preinserted_text_len() -> c_int;

    // typval for ret dict
    fn tv_dict_alloc_ret(rettv: *mut TypvalTInfo);
    #[link_name = "emsg"]
    fn emsg_info(s: *const c_char);
    #[link_name = "gettext"]
    fn gettext_info(msgid: *const c_char) -> *const c_char;
    #[link_name = "e_listreq"]
    static e_listreq_info: [c_char; 0];
}

/// Build the word/abbr/menu/kind/info/user_data dict for one match entry.
unsafe fn build_match_dict(m: ComplMatch) -> *mut c_void {
    let di = nvim_ci_dict_alloc();
    nvim_ci_dict_add_str(di, c"word".as_ptr(), 4, nvim_compl_match_get_cp_str_data(m));
    nvim_ci_dict_add_str(
        di,
        c"abbr".as_ptr(),
        4,
        nvim_compl_match_get_cp_text_abbr(m),
    );
    nvim_ci_dict_add_str(
        di,
        c"menu".as_ptr(),
        4,
        nvim_compl_match_get_cp_text_menu(m),
    );
    nvim_ci_dict_add_str(
        di,
        c"kind".as_ptr(),
        4,
        nvim_compl_match_get_cp_text_kind(m),
    );
    nvim_ci_dict_add_str(
        di,
        c"info".as_ptr(),
        4,
        nvim_compl_match_get_cp_text_info(m),
    );
    if nvim_compl_match_user_data_is_unknown(m) != 0 {
        nvim_ci_dict_add_str(di, c"user_data".as_ptr(), 9, c"".as_ptr());
    } else {
        // Need a typval-sized buffer to receive user_data
        let mut tv_buf = [0u8; 16]; // sizeof(typval_T)
        nvim_compl_match_copy_user_data_tv(m, tv_buf.as_mut_ptr().cast());
        nvim_ci_dict_add_tv(di, c"user_data".as_ptr(), 9, tv_buf.as_mut_ptr().cast());
    }
    di
}

/// Parse what_list and return a bitmask of CI_WHAT_* flags.
/// If what_list is NULL, returns all flags except MATCHES and COMPLETED.
///
/// # Safety
/// `what_list` must be NULL or a valid VimL list_T pointer.
unsafe fn ci_parse_what_list(what_list: *mut c_void) -> c_int {
    if what_list.is_null() {
        return CI_WHAT_ALL & !(CI_WHAT_MATCHES | CI_WHAT_COMPLETED);
    }
    let mut what_flag: c_int = 0;
    let mut item = nvim_tv_list_first(what_list);
    while !item.is_null() {
        let what = nvim_tv_list_item_string(item);
        if !what.is_null() {
            let s = std::ffi::CStr::from_ptr(what).to_bytes();
            if s == b"mode" {
                what_flag |= CI_WHAT_MODE;
            } else if s == b"pum_visible" {
                what_flag |= CI_WHAT_PUM_VISIBLE;
            } else if s == b"items" {
                what_flag |= CI_WHAT_ITEMS;
            } else if s == b"selected" {
                what_flag |= CI_WHAT_SELECTED;
            } else if s == b"completed" {
                what_flag |= CI_WHAT_COMPLETED;
            } else if s == b"preinserted_text" {
                what_flag |= CI_WHAT_PREINSERTED_TEXT;
            } else if s == b"matches" {
                what_flag |= CI_WHAT_MATCHES;
            }
        }
        item = nvim_tv_list_item_next(what_list, item);
    }
    what_flag
}

/// Implementation of complete_info(), replacing nvim_get_complete_info_impl.
///
/// # Safety
/// Requires valid VimL completion state. Called from rs_get_complete_info.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_get_complete_info(what_list: *mut c_void, retdict: *mut c_void) {
    let what_flag = ci_parse_what_list(what_list);
    let mut ret: c_int = 1; // OK = 1, use nonzero as "success" sentinel

    if (what_flag & CI_WHAT_MODE) != 0 && ret != 0 {
        let mode = rs_ins_compl_mode();
        ret = c_int::from(nvim_ci_dict_add_str(retdict, c"mode".as_ptr(), 4, mode) == 0);
    }

    if (what_flag & CI_WHAT_PUM_VISIBLE) != 0 && ret != 0 {
        ret = c_int::from(
            nvim_ci_dict_add_nr(
                retdict,
                c"pum_visible".as_ptr(),
                11,
                i64::from(nvim_pum_visible()),
            ) == 0,
        );
    }

    if (what_flag & CI_WHAT_PREINSERTED_TEXT) != 0 && ret != 0 {
        let ptr = nvim_ci_preinserted_text_ptr();
        let len = nvim_ci_preinserted_text_len();
        ret = c_int::from(
            nvim_ci_dict_add_str_len(retdict, c"preinserted_text".as_ptr(), 16, ptr, len) == 0,
        );
    }

    let needs_list =
        (what_flag & (CI_WHAT_ITEMS | CI_WHAT_SELECTED | CI_WHAT_MATCHES | CI_WHAT_COMPLETED)) != 0;
    if needs_list && ret != 0 {
        let has_items = (what_flag & CI_WHAT_ITEMS) != 0;
        let has_matches = (what_flag & CI_WHAT_MATCHES) != 0;
        let has_completed = (what_flag & CI_WHAT_COMPLETED) != 0;
        let mut selected_idx: c_int = -1;

        let li: *mut c_void;
        if has_items || has_matches {
            li = nvim_ci_list_alloc_known();
            let (key, klen): (*const c_char, usize) = if has_matches && !has_items {
                (c"matches".as_ptr(), 7)
            } else {
                (c"items".as_ptr(), 5)
            };
            ret = c_int::from(nvim_ci_dict_add_list(retdict, key, klen, li) == 0);
        } else {
            li = std::ptr::null_mut();
        }

        if (what_flag & CI_WHAT_SELECTED) != 0 && ret != 0 {
            let curr = nvim_compl_get_curr_match();
            if !curr.is_null() && nvim_compl_match_get_cp_number(curr) == -1 {
                rs_ins_compl_update_sequence_numbers();
            }
        }

        if ret != 0 {
            let first = nvim_compl_get_first_match();
            if !first.is_null() {
                let mut list_idx: c_int = 0;
                let mut m = first;
                loop {
                    if nvim_compl_match_at_original_text(m) == 0 {
                        let in_array = nvim_compl_match_get_in_match_array(m) != 0;
                        if has_items || (has_matches && in_array) {
                            let di = build_match_dict(m);
                            nvim_ci_list_append_dict(li, di);
                            if has_matches && has_items {
                                nvim_ci_dict_add_bool(
                                    di,
                                    c"match".as_ptr(),
                                    5,
                                    c_int::from(in_array),
                                );
                            }
                        }

                        let curr = nvim_compl_get_curr_match();
                        if !curr.is_null()
                            && nvim_compl_match_get_cp_number(curr)
                                == nvim_compl_match_get_cp_number(m)
                        {
                            selected_idx = list_idx;
                        }
                        if !has_matches || in_array {
                            list_idx += 1;
                        }
                    }
                    let next = nvim_compl_match_get_next(m);
                    if next.is_null() || is_first_match(next) {
                        break;
                    }
                    m = next;
                }
            }
        }

        if (what_flag & CI_WHAT_SELECTED) != 0 && ret != 0 {
            ret = c_int::from(
                nvim_ci_dict_add_nr(retdict, c"selected".as_ptr(), 8, i64::from(selected_idx)) == 0,
            );
            let wp = nvim_win_float_find_preview();
            if !wp.is_null() {
                nvim_ci_dict_add_nr(
                    retdict,
                    c"preview_winid".as_ptr(),
                    13,
                    i64::from(nvim_ci_win_get_handle(wp)),
                );
                nvim_ci_dict_add_nr(
                    retdict,
                    c"preview_bufnr".as_ptr(),
                    13,
                    i64::from(nvim_ci_win_get_buf_handle(wp)),
                );
            }
        }

        if selected_idx != -1 && has_completed && ret != 0 {
            let curr = nvim_compl_get_curr_match();
            let di = build_match_dict(curr);
            ret = c_int::from(nvim_ci_dict_add_dict(retdict, c"completed".as_ptr(), 9, di) == 0);
        }
    }

    let _ = ret;
}

/// VimL `complete_info()` builtin.
///
/// Allocates the return dict, parses the optional what_list argument,
/// and populates the dict with current completion state.
///
/// # Safety
/// `argvars` must be a valid `typval_T[]` pointer; `rettv` a `typval_T*`.
#[export_name = "f_complete_info"]
pub unsafe extern "C" fn rs_f_complete_info(
    argvars: *mut c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let rettv = rettv.cast::<TypvalTInfo>();
    tv_dict_alloc_ret(rettv);

    let argvars = argvars.cast::<TypvalTInfo>();
    let what_list = if (*argvars).v_type == VAR_UNKNOWN {
        core::ptr::null_mut()
    } else {
        if (*argvars).v_type != 4 {
            // VAR_LIST = 4
            emsg_info(gettext_info(e_listreq_info.as_ptr()));
            return;
        }
        (*argvars).vval.v_list
    };
    rs_get_complete_info(what_list, (*rettv).vval.v_dict);
}
