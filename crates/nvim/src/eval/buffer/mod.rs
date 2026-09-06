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

#![deny(unsafe_op_in_unsafe_fn)]
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

use crate::autocmd::{aucmd_prepbuf, aucmd_restbuf};
use crate::buffer::{
    buf_ensure_loaded, buf_is_nofilename, buf_is_prompt, buflist_add, buflist_findlnum,
    buflist_findname_exp, buflist_new, find_buf,
};
use crate::change::{appended_lines_mark, changed_lines, deleted_lines_mark, inserted_bytes};
use crate::cursor::check_cursor_col;
use crate::edit::buf_prompt_text;
use crate::eval::funcs::args::{Args, frame};
use crate::eval::funcs::{get_buf_arg, tv_get_buf, tv_get_buf_from_arg};
use crate::eval::typval::{
    callback_free, tv_check_str_or_nr, tv_clear, tv_dict_add_dict, tv_dict_add_list,
    tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc, tv_dict_find, tv_get_lnum, tv_get_lnum_buf,
    tv_get_number, tv_get_number_chk, tv_list_alloc, tv_list_alloc_ret, tv_list_append_dict,
    tv_list_append_number, tv_list_append_string, tv_list_item_remove,
};
use crate::eval::{callback_from_typval, typval_tostring};
use crate::ex_cmds::check_secure;
use crate::extmark::extmark_splice_cols;
use crate::narrow::number_as_int;
use core::ffi::{CStr, c_char, c_int};
use core::{mem, ptr};

use crate::buffer::state::swap_exists_action;
use crate::memline::{ml_append, ml_delete_flags, ml_get, ml_replace, ml_replace_buf};
use crate::memory::{strnequal, xfree, xstrdup};
use crate::message::state::did_emsg;
use crate::r#move::update_topline;
use crate::path::path_with_url;
use crate::sign::{buf_has_signs, get_buffer_signs};
use crate::strings::{concat_str, xstrnsave};
use crate::types::*;
use crate::undo::u_sync_once;
use crate::winlayer::graph::cmdwin_buf;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
use crate::memline::ML_DEL_MESSAGE;
use crate::normal::{set_visual_active, visual_active};
use crate::undo::{buf_is_changed, u_clearallandblockfree, u_save, u_savesub, u_sync};
use crate::winlayer::{Buf, Live, TabPage, Win, buffers, tab_windows, windows_in_tab};

/// A value whose caller has promised it outlives the handle.
///
/// The builtins here are handed `TypVal`s and `ListItem`s that belong to
/// the evaluator's own argument frame, which outlives the call. Wrapping is
/// the unsafe step, once; every `(*p).field` after it is checked code.
pub(super) type Tv = Live<TypVal>;

/// One item of a live list, whose caller has promised the list outlives it.
pub(super) type Li = Live<ListItem>;

/// Argument `i` as a Number.
///
/// Argument `i` as a line number in the current buffer, reported and clamped.
pub(super) fn arg_lnum(args: Args<'_>, i: usize) -> LineNr {
    // SAFETY: as [`arg_number`].
    unsafe { tv_get_lnum(args.ptr(i)) }
}

/// Argument `i` as a line number in `buffer`.
///
/// # Safety
/// `buffer` is a live buffer or NULL.
pub(super) unsafe fn arg_lnum_buf(args: Args<'_>, i: usize, buffer: Buf) -> LineNr {
    // SAFETY: the caller's obligation, and [`arg_number`]'s for the typval.
    unsafe { tv_get_lnum_buf(args.ptr(i), Some(buffer)) }
}

/// The buffer argument `i` names, or NULL -- the `bufnr()`-shaped spelling,
/// which takes a number, a name or a pattern.
pub(super) fn arg_buf(args: Args<'_>, i: usize, curtab_only: c_int) -> Option<Buf> {
    // SAFETY: as [`arg_number`].
    unsafe { tv_get_buf(args.ptr(i), curtab_only) }
}

/// The buffer argument `i` names, reporting for a type that names none.
pub(super) fn arg_buf_chk(args: Args<'_>, i: usize) -> Option<Buf> {
    // SAFETY: as [`arg_number`].
    unsafe { tv_get_buf_from_arg(args.ptr(i)) }
}

/// The editor state [`SavedBufferState::prepare`] saves so that
/// [`SavedBufferState::restore`] can put it back.
struct SavedBufferState {
    curwin_save: Win,
    aco: AcoSave,
    using_aco: bool,
    save_visual_active: bool,
}
impl SavedBufferState {
    /// The all-zero state the two halves below start from — `AcoSave`'s
    /// own initial value, which `aucmd_prepbuf` overwrites in full.
    fn new() -> Self {
        // SAFETY: every field is a raw pointer, an integer, a `bool` or a
        // `Win` (one pointer), for all of which all-zero is a valid value.
        // A null `Win` names no window and `prepare` overwrites it before
        // anything reads it, exactly as the raw pointer it replaced was.
        unsafe { mem::zeroed() }
    }

    /// Make `buffer` the current buffer, with a window showing it, so that a
    /// change to it has its side effects (mark adjustment and the rest) done
    /// where they belong.
    ///
    /// MUST be undone with [`SavedBufferState::restore`].
    ///
    /// # Safety
    /// `curwin`/`curbuf` must be set, which they are from startup to exit.
    unsafe fn prepare(&mut self, buffer: Buf) {
        self.save_visual_active = visual_active();
        set_visual_active(false);
        self.curwin_save = Win::current();
        buffer.make_current();
        // SAFETY: `curbuf` was just set to the caller's live buffer.
        unsafe { find_win_for_curbuf() };
        let current = Win::current();
        if current.w_buffer != buffer.raw() {
            // No existing window for this buffer. It is dangerous to have
            // `curwin->w_buffer` differ from `curbuf`, so use the autocmd
            // window.
            current.buffer().make_current();
            // SAFETY: `self.aco` is this frame's, and the buffer is live.
            unsafe { aucmd_prepbuf(&raw mut self.aco, buffer) };
            self.using_aco = true;
        }
    }

    /// Undo what [`SavedBufferState::prepare`] did.
    ///
    /// # Safety
    /// `self` must be the state `prepare` filled in.
    unsafe fn restore(&mut self) {
        if self.using_aco {
            // SAFETY: the caller's obligation — `aco` is what `prepare` left.
            unsafe { aucmd_restbuf(&raw mut self.aco) };
        } else {
            // The saved window is live and so is its buffer.
            self.curwin_save.make_current();
            self.curwin_save.buffer().make_current();
        }
        set_visual_active(self.save_visual_active);
    }
}

/// If there is a window for `curbuf`, make it the current window.
///
/// # Safety
/// `curbuf` must be set, which it is from startup to exit.
unsafe fn find_win_for_curbuf() {
    // The b_wininfo list holds the windows that recently contained the
    // buffer, so walking it is cheaper than walking every window. It can name
    // a window that has moved on, hence the second test.
    // SAFETY: `curbuf` is live and its window-info vector holds `size` live
    // entries.
    let buf = Buf::current();
    let wininfo = &buf.b_wininfo;
    for i in 0..wininfo.size {
        let wip: *mut WinInfo = unsafe { *wininfo.items.add(i) };
        // SAFETY: an entry's `wi_win` is a live window or null.
        let Some(win) = (unsafe { Win::from_raw((*wip).wi_win) }) else {
            continue;
        };
        if win.w_buffer == Buf::current_raw() {
            win.make_current();
            break;
        }
    }
}
pub const SEA_NONE: c_int = 0;
pub const SEA_READONLY: c_int = 4;
