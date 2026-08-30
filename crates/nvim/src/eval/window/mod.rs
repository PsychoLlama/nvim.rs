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
//! This file holds only what they share: the tab-page-scoped window lookups
//! the numbering questions rest on, hung off [`winlayer`](crate::winlayer)'s
//! handles rather than written out as a pointer test at every site.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

mod info;
mod resolve;
mod switch;
mod view;

pub use info::{f_getcmdwintype, f_gettabinfo, f_getwininfo, f_win_gettype, f_winlayout};
pub use resolve::{
    f_tabpagenr, f_tabpagewinnr, f_win_findbuf, f_win_getid, f_win_gotoid, f_win_id2tabwin,
    f_win_id2win, f_winbufnr, f_winnr, find_tabwin, find_win_by_nr, find_win_by_nr_or_id,
    win_and_tab_by_id, win_by_id,
};
// The family's shared views of the editor's roots and of one argument. They
// live in `resolve` because resolving an argument to a window is its job,
// and are named here so that `use super::*` finds them.
pub(crate) use resolve::{arg_number, arg_number_chk, arg_win, cur_buf, cur_tab, cur_win};
pub use switch::{
    f_win_execute, restore_win, restore_win_noblock, switch_win, switch_win_noblock,
    win_execute_after, win_execute_before,
};
pub use view::{
    f_getwinpos, f_getwinposx, f_getwinposy, f_win_move_separator, f_win_move_statusline,
    f_win_screenpos, f_win_splitmove, f_wincol, f_winheight, f_winline, f_winrestcmd,
    f_winrestview, f_winsaveview, f_winwidth,
};

use crate::autocmd::{block_autocmds, is_aucmd_win, unblock_autocmds};
use crate::buffer::{buf_is_quickfix, buf_is_terminal, do_autochdir};
use crate::cursor::{check_cursor, check_pos};
use crate::eval::funcs::args::{Args, frame};
use crate::eval::funcs::execute_common;
use crate::eval::typval::{
    tv_check_for_nonnull_dict_arg, tv_dict_add_dict, tv_dict_add_list, tv_dict_add_nr,
    tv_dict_alloc, tv_dict_alloc_ret, tv_dict_find, tv_dict_get_number, tv_get_number,
    tv_get_number_chk, tv_list_alloc, tv_list_alloc_ret, tv_list_append_dict, tv_list_append_list,
    tv_list_append_number, tv_list_append_string,
};
use crate::ex_getln::text_or_buf_locked;
use crate::garray::{ga_append, ga_concat_len, ga_init};
use crate::main::{
    cmdwin_type, cmdwin_win, curbuf, curtab, curwin, lastused_tabpage, p_acd, prevwin,
};
use crate::memory::{xfree, xmallocz, xstrdup};
use crate::r#move::{
    changed_window_setting, check_topfill, set_topline, update_curswant, validate_botline_win,
    validate_cursor, win_col_off,
};
use crate::narrow::number_as_int;
use crate::normal::end_visual_mode;
use crate::os::fs::{os_chdir, os_dirname};
use crate::strings::vim_snprintf_safelen;
use crate::types::*;
use crate::winlayer::{
    Buf, Frame, TabPage, Win, WinId, last_window, tab_windows, tabs, windows_in_tab,
};
use ::libc::strtol;
use core::ffi::{CStr, c_char, c_int};
use core::{mem, ptr};
pub const FR_LEAF: c_int = 0;
pub const FR_ROW: c_int = 1;
/// Window handles start here, so a `winnr()`-shaped argument at or above it is
/// a window id rather than a window number.
pub const LOWEST_WIN_ID: c_int = 1000;
use crate::window::{
    check_split_disallowed, find_tabpage, goto_tabpage_tp, goto_tabpage_win, tabpage_index,
    unuse_tabpage, use_tabpage, valid_tabpage, win_drag_status_line, win_drag_vsep_line,
    win_get_tabwin, win_goto, win_horz_neighbor, win_new_height, win_new_width, win_splitmove,
    win_valid, win_vert_neighbor,
};
/// The three window pointers a tab page keeps, read the way upstream reads
/// them.
///
/// While a tab page is the current one its own `tp_curwin`/`tp_lastwin`/
/// `tp_prevwin` are stale — the live values are in the globals, and the tab
/// page's fields are only written back when it is left. The C spells the
/// `tp == curtab ? global : tp->field` test out at every site; here it is
/// written once per field. (The window *list* has the same rule and is
/// [`windows_in_tab`], which winlayer already provides.)
impl TabPage {
    /// The window that is current in this tab page.
    fn curwin(self) -> Win {
        let wp = if self.is_current() {
            curwin.get()
        } else {
            self.tp_curwin
        };
        // SAFETY: a live tab page's current window is live, and `curwin` is
        // set from startup to exit.
        unsafe { Win::new(wp) }
    }

    /// The last window of this tab page.
    fn lastwin(self) -> Win {
        let wp = match self.is_current() {
            true => last_window(),
            false => self.tp_lastwin.and_then(WinId::get),
        };
        wp.expect("a live tab page has a last window")
    }

    /// The window that was current before this tab page's current one — `None`
    /// until something has been left, which is what `winnr("#")` reports as 0.
    fn prevwin(self) -> Option<Win> {
        let wp = if self.is_current() {
            prevwin.get()
        } else {
            self.tp_prevwin
        };
        // SAFETY: a live tab page's previous window is live or null.
        unsafe { Win::from_raw(wp) }
    }
}

impl Win {
    /// Whether `winnr()`'s numbering counts this window in tab page `tp`.
    ///
    /// Every window has a number except a hidden or unfocusable float — and
    /// even one of those keeps its number while it is the tab page's current
    /// window, which is the only way the cursor can be inside it.
    pub fn has_winnr(self, tp: TabPage) -> bool {
        self == tp.curwin() || !self.w_config.hide && self.w_config.focusable
    }
}
