//! Buffer-related builtin vimscript functions: the `buf*()`, `*bufline()` and
//! `prompt_*()` families.
//!
//! The work is split across four submodules:
//!
//! - `lookup` resolves a buffer argument (number, name, `#`, `%`) and answers
//!   the questions about one — `bufnr()`, `bufname()`, `bufwinid()`, ...
//! - `lines` reads and writes buffer text: `getbufline()`, `setbufline()`,
//!   `appendbufline()`, `deletebufline()` and their current-buffer forms.
//! - `info` builds the dictionary `getbufinfo()` returns.
//! - `prompt` is the prompt-buffer surface.
//!
//! This file holds what they share: the "make another buffer current for the
//! duration of a change" dance, and the window walk several of them need.

mod info;
mod lines;
mod lookup;
mod prompt;

pub use info::f_getbufinfo;
pub use lines::{
    f_append, f_appendbufline, f_deletebufline, f_getbufline, f_getbufoneline, f_getline,
    f_setbufline, f_setline,
};
pub use lookup::{
    f_bufadd, f_bufexists, f_buflisted, f_bufload, f_bufloaded, f_bufname, f_bufnr, f_bufwinid,
    f_bufwinnr, find_buffer,
};
pub use prompt::{
    f_prompt_appendbuf, f_prompt_setcallback, f_prompt_setinterrupt, f_prompt_setprompt,
};

use crate::src::nvim::autocmd::{aucmd_prepbuf, aucmd_restbuf};
use crate::src::nvim::buffer::{
    bt_nofilename, bt_prompt, buf_ensure_loaded, buflist_add, buflist_findlnum,
    buflist_findname_exp, buflist_findnr, buflist_new,
};
use crate::src::nvim::change::{
    appended_lines_mark, changed_lines, deleted_lines_mark, inserted_bytes,
};
use crate::src::nvim::cursor::check_cursor_col;
use crate::src::nvim::edit::buf_prompt_text;
use crate::src::nvim::eval::funcs::{get_buf_arg, tv_get_buf, tv_get_buf_from_arg};
use crate::src::nvim::eval::typval::{
    callback_free, tv_check_str_or_nr, tv_clear, tv_dict_add_dict, tv_dict_add_list,
    tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc, tv_dict_find, tv_get_lnum, tv_get_lnum_buf,
    tv_get_number, tv_get_number_chk, tv_get_string, tv_get_string_chk, tv_list_alloc,
    tv_list_alloc_ret, tv_list_append_dict, tv_list_append_number, tv_list_append_string,
    tv_list_item_remove,
};
use crate::src::nvim::eval::window::win_has_winnr;
use crate::src::nvim::eval_1::{callback_from_typval, typval_tostring};
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::extmark::extmark_splice_cols;
use core::ffi::CStr;
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::mem;
use core::ptr;

use crate::src::nvim::main::{
    VIsual_active, cmdwin_buf, curbuf, curtab, curwin, did_emsg, emsg_off, first_tabpage, firstbuf,
    firstwin, swap_exists_action, u_sync_once,
};
use crate::src::nvim::memline::{
    ml_append, ml_delete_flags, ml_get, ml_get_buf, ml_get_buf_len, ml_replace, ml_replace_buf,
};
use crate::src::nvim::memory::{strnequal, xfree, xstrdup};
use crate::src::nvim::r#move::update_topline;
use crate::src::nvim::os::libc::{strcmp, strlen};
use crate::src::nvim::path::path_with_url;
use crate::src::nvim::sign::{buf_has_signs, get_buffer_signs};
use crate::src::nvim::strings::{concat_str, xstrnsave};
pub use crate::src::nvim::types::*;
pub const NUL: c_int = '\0' as c_int;
pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
pub const VAR_UNKNOWN: VarType = 0;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_STRING: VarType = 2;
pub const VAR_LIST: VarType = 4;
pub const VAR_DICT: VarType = 5;
pub const kCallbackNone: CallbackType = 0;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const ML_DEL_MESSAGE: c_uint = 1;
use crate::src::nvim::undo::{bufIsChanged, u_clearallandblockfree, u_save, u_savesub, u_sync};
/// The editor state `change_other_buffer_prepare` saves so that
/// `change_other_buffer_restore` can put it back.
#[derive(Copy, Clone)]
struct SavedBufferState {
    curwin_save: *mut win_T,
    aco: aco_save_T,
    using_aco: bool,
    save_visual_active: bool,
}
/// Visit every window of every tab page, in `FOR_ALL_TAB_WINDOWS` order: the
/// current tab page reports the live window list rather than its saved one.
unsafe fn for_all_tab_windows(mut visit: impl FnMut(*mut win_T)) {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            visit(wp);
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
/// If there is a window for "curbuf", make it the current window.
unsafe fn find_win_for_curbuf() {
    // The b_wininfo list holds the windows that recently contained the
    // buffer, so walking it is cheaper than walking every window. It can name
    // a window that has moved on, hence the second test.
    let wininfo = &(*curbuf.get()).b_wininfo;
    for i in 0..wininfo.size {
        let wip: *mut WinInfo = *wininfo.items.add(i);
        if !(*wip).wi_win.is_null() && (*(*wip).wi_win).w_buffer == curbuf.get() {
            curwin.set((*wip).wi_win);
            break;
        }
    }
}
/// Used before making a change in "buf", which is not the current one: Make
/// "buf" the current buffer and find a window for this buffer, so that side
/// effects are done correctly (e.g., adjusting marks).
///
/// Information is saved in "cob" and MUST be restored by calling
/// change_other_buffer_restore().
unsafe fn change_other_buffer_prepare(cob: *mut SavedBufferState, buf: *mut buf_T) {
    cob.write(mem::zeroed());
    // Set "curbuf" to the buffer being changed. Then make sure there is a
    // window for it to handle any side effects.
    (*cob).save_visual_active = VIsual_active.get();
    VIsual_active.set(false);
    (*cob).curwin_save = curwin.get();
    curbuf.set(buf);
    find_win_for_curbuf();
    if (*curwin.get()).w_buffer != buf {
        // No existing window for this buffer. It is dangerous to have
        // curwin->w_buffer differ from "curbuf", so use the autocmd window.
        curbuf.set((*curwin.get()).w_buffer);
        aucmd_prepbuf(&raw mut (*cob).aco, buf);
        (*cob).using_aco = true;
    }
}
/// Undo what [`change_other_buffer_prepare`] did.
unsafe fn change_other_buffer_restore(cob: *mut SavedBufferState) {
    if (*cob).using_aco {
        aucmd_restbuf(&raw mut (*cob).aco);
    } else {
        curwin.set((*cob).curwin_save);
        curbuf.set((*curwin.get()).w_buffer);
    }
    VIsual_active.set((*cob).save_visual_active);
}
pub const SEA_NONE: c_int = 0;
pub const SEA_READONLY: c_int = 4;
pub const true_0: c_int = 1;
pub const false_0: c_int = 0;
