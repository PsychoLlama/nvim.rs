//! Window- and tab-page-related builtin vimscript functions.
//!
//! The work is split across four submodules:
//!
//! - `resolve` turns a vimscript argument into a window or tab page and
//!   answers the numbering questions (`winnr()`, `win_getid()`,
//!   `win_id2win()`, `tabpagenr()`, ...).
//! - `info` builds the dictionaries and lists that describe the layout
//!   (`getwininfo()`, `gettabinfo()`, `winlayout()`, `win_gettype()`).
//! - `view` is geometry: the saved view, the resize commands, and moving a
//!   window or its separators.
//! - `switch` makes another window current for the duration of a call, which
//!   is what `win_execute()` and the API's window-scoped entry points use.
//!
//! This file holds only what they share.

mod info;
mod resolve;
mod switch;
mod view;

pub use info::{f_getcmdwintype, f_gettabinfo, f_getwininfo, f_win_gettype, f_winlayout};
pub use resolve::{
    f_tabpagenr, f_tabpagewinnr, f_win_findbuf, f_win_getid, f_win_gotoid, f_win_id2tabwin,
    f_win_id2win, f_winbufnr, f_winnr, find_tabwin, find_win_by_nr, find_win_by_nr_or_id,
    win_id2wp, win_id2wp_tp,
};
pub use switch::{
    f_win_execute, restore_win, restore_win_noblock, switch_win, switch_win_noblock,
    win_execute_after, win_execute_before,
};
pub use view::{
    f_getwinpos, f_getwinposx, f_getwinposy, f_win_move_separator, f_win_move_statusline,
    f_win_screenpos, f_win_splitmove, f_wincol, f_winheight, f_winline, f_winrestcmd,
    f_winrestview, f_winsaveview, f_winwidth,
};

use crate::src::nvim::autocmd::{block_autocmds, is_aucmd_win, unblock_autocmds};
use crate::src::nvim::buffer::{bt_quickfix, bt_terminal, do_autochdir};
use crate::src::nvim::cursor::{check_cursor, check_pos};
use crate::src::nvim::eval::funcs::execute_common;
use crate::src::nvim::eval::typval::{
    tv_check_for_nonnull_dict_arg, tv_dict_add_dict, tv_dict_add_list, tv_dict_add_nr,
    tv_dict_alloc, tv_dict_alloc_ret, tv_dict_find, tv_dict_get_number, tv_get_number,
    tv_get_number_chk, tv_get_string_chk, tv_list_alloc, tv_list_alloc_ret, tv_list_append_dict,
    tv_list_append_list, tv_list_append_number, tv_list_append_string,
};
use crate::src::nvim::ex_getln::text_or_buf_locked;
use crate::src::nvim::garray::{ga_append, ga_concat_len, ga_init};
use crate::src::nvim::main::{
    VIsual, VIsual_active, cmdwin_type, cmdwin_win, curbuf, curtab, curwin, first_tabpage,
    firstwin, lastused_tabpage, lastwin, p_acd, prevwin,
};
use crate::src::nvim::memory::{strequal, xfree, xmallocz, xstrdup};
use crate::src::nvim::r#move::{
    changed_window_setting, check_topfill, set_topline, update_curswant, validate_botline_win,
    validate_cursor, win_col_off,
};
use crate::src::nvim::normal::end_visual_mode;
use crate::src::nvim::os::fs::{os_chdir, os_dirname};
use crate::src::nvim::os::libc::{memset, strcmp, strtol};
use crate::src::nvim::strings::vim_snprintf_safelen;
pub use crate::src::nvim::types::*;
use core::ffi::CStr;
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::mem;
use core::ptr;
pub const NUL: c_int = '\0' as c_int;
pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
pub const MAXPATHL: c_int = 4096;
pub const FR_LEAF: c_int = 0;
pub const FR_ROW: c_int = 1;
pub const VAR_UNKNOWN: VarType = 0;
pub const VAR_STRING: VarType = 2;
pub const kListLenUnknown: ListLenSpecials = -1;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const LOWEST_WIN_ID: c_uint = 1000;
pub const WSP_VERT: c_uint = 2;
pub const WSP_BELOW: c_uint = 64;
pub const WSP_ABOVE: c_uint = 128;
use crate::src::nvim::window::{
    check_split_disallowed, find_tabpage, goto_tabpage_tp, goto_tabpage_win, tabpage_index,
    unuse_tabpage, use_tabpage, valid_tabpage, win_drag_status_line, win_drag_vsep_line,
    win_get_tabwin, win_goto, win_horz_neighbor, win_new_height, win_new_width, win_splitmove,
    win_valid, win_vert_neighbor,
};
/// The first window of tab page `tp`. The current tab page's window list lives
/// in the globals; a background tab page keeps its own.
unsafe fn tab_firstwin(tp: *mut tabpage_T) -> *mut win_T {
    if tp == curtab.get() {
        firstwin.get()
    } else {
        (*tp).tp_firstwin
    }
}
pub unsafe extern "C" fn win_has_winnr(wp: *mut win_T, mut tp: *mut tabpage_T) -> bool {
    wp == (if tp == curtab.get() {
        curwin.get()
    } else {
        (*tp).tp_curwin
    }) || !(*wp).w_config.hide && (*wp).w_config.focusable
}
pub const true_0: c_int = 1;
pub const false_0: c_int = 0;
