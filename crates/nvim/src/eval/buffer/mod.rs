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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

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

use crate::autocmd::AucmdBuf;
use crate::buffer::{
    buf_ensure_loaded, buf_is_nofilename, buf_is_prompt, buflist_findlnum, find_buf,
};
use crate::change::{appended_lines_mark, changed_lines, deleted_lines_mark, inserted_bytes};
use crate::cursor::check_cursor_col;
use crate::eval::funcs::{get_buf_arg, tv_get_buf, tv_get_buf_from_arg};
use crate::eval::typval::{
    callback_free, dict_find, tv_check_str_or_nr, tv_clear, tv_dict_alloc, tv_get_lnum,
    tv_get_lnum_buf, tv_get_number, tv_get_number_chk, tv_list_alloc, tv_list_alloc_ret,
};
use crate::eval::{callback_from_typval, typval_tostring};
use crate::ex_cmds::check_secure;
use crate::extmark::extmark_splice_cols;
use crate::narrow::number_as_int;
use core::ffi::{CStr, c_int};

use crate::buffer::state::swap_exists_action;
use crate::memline::ml_delete_flags;
use crate::message::state::did_emsg;
use crate::r#move::update_topline;
use crate::path::path_with_url;
use crate::sign::{buf_has_signs, get_buffer_signs};
use crate::types::*;
use crate::undo::u_sync_once;
use crate::winlayer::graph::cmdwin_buf;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
use crate::buffer::WinInfos;
use crate::memline::ML_DEL_MESSAGE;
use crate::normal::{set_visual_active, visual_active};
use crate::undo::{buf_is_changed, u_clearallandblockfree, u_save, u_savesub, u_sync};
use crate::winlayer::{Buf, TabPage, Win, buffers, tab_windows, windows_in_tab};

/// Argument `i` as a Number.
///
/// Argument `i` as a line number in the current buffer, reported and clamped.
pub(super) fn arg_lnum(args: &[TypVal], i: usize) -> LineNr {
    tv_get_lnum(&args[i])
}

/// Argument `i` as a line number in `buffer`.
pub(super) fn arg_lnum_buf(args: &[TypVal], i: usize, buffer: Option<Buf>) -> LineNr {
    tv_get_lnum_buf(&args[i], buffer)
}

/// The buffer argument `i` names, or NULL -- the `bufnr()`-shaped spelling,
/// which takes a number, a name or a pattern.
pub(super) fn arg_buf(args: &[TypVal], i: usize, curtab_only: c_int) -> Option<Buf> {
    tv_get_buf(&args[i], curtab_only)
}

/// The buffer argument `i` names, reporting for a type that names none.
pub(super) fn arg_buf_chk(args: &[TypVal], i: usize) -> Option<Buf> {
    tv_get_buf_from_arg(&args[i])
}

/// The editor state [`SavedBufferState::prepare`] saves so that
/// [`SavedBufferState::restore`] can put it back.
struct SavedBufferState {
    curwin_save: Win,
    /// The autocommand window the buffer was made current in, when no window
    /// showed it; dropping it restores what it replaced.
    aco: Option<AucmdBuf>,
    save_visual_active: bool,
}

impl SavedBufferState {
    /// Make `buffer` the current buffer, with a window showing it, so that a
    /// change to it has its side effects (mark adjustment and the rest) done
    /// where they belong.
    ///
    /// MUST be undone with [`SavedBufferState::restore`].
    fn prepare(buffer: Buf) -> Self {
        let save_visual_active = visual_active();
        set_visual_active(false);
        let curwin_save = Win::current();
        buffer.make_current();
        find_win_for_curbuf();
        let current = Win::current();
        let aco = (current.w_buffer != buffer).then(|| {
            // No existing window for this buffer. It is dangerous to have
            // `curwin->w_buffer` differ from `curbuf`, so use the autocmd
            // window.
            current.buffer().make_current();
            AucmdBuf::enter(buffer)
        });
        SavedBufferState {
            curwin_save,
            aco,
            save_visual_active,
        }
    }

    /// Undo what [`SavedBufferState::prepare`] did.
    fn restore(self) {
        if let Some(aco) = self.aco {
            drop(aco);
        } else {
            // The saved window is live and so is its buffer.
            self.curwin_save.make_current();
            self.curwin_save.buffer().make_current();
        }
        set_visual_active(self.save_visual_active);
    }
}

/// If there is a window for `curbuf`, make it the current window.
fn find_win_for_curbuf() {
    // The b_wininfo list holds the windows that recently contained the
    // buffer, so walking it is cheaper than walking every window. It can name
    // a window that has moved on, hence the second test.
    let mut buf = Buf::current();
    let found = WinInfos::of(&mut buf)
        .entries_mut()
        .iter()
        .filter_map(|entry| entry.window())
        .find(|win| win.w_buffer.is_current());
    if let Some(win) = found {
        win.make_current();
    }
}

pub const SEA_NONE: c_int = 0;
pub const SEA_READONLY: c_int = 4;
