//! The callbacks the option table names for a boolean or numeric option
//! that just changed. The string options' callbacks live in
//! [`crate::optionstr`].
//!
//! Each one runs *after* the new value is already in the option's variable,
//! and reports a message when it does not like it — [`super::did_set_option`]
//! then puts the old value back. So a callback that only validates can
//! return early, but one that also updates derived state has to validate
//! first.
//!
//! The generated option table holds them as `opt_did_set_cb_T` function
//! pointers, which is the only reason they are `pub`.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::keycodes::{Ctrl_C, Key};
use crate::types::AutoEvent;
use core::ffi::{CStr, c_char, c_int};
use core::mem::offset_of;
use core::ptr;

use crate::ascii::ascii_isdigit;
use crate::autocmd::apply_autocmds;
use crate::buffer::{BufFlags, do_autochdir};
use crate::change::save_file_ff;
use crate::charset::buf_init_chartab;
use crate::diff::diff_buf_adjust;
use crate::drawscreen::{
    UPD_NOT_VALID, UPD_SOME_VALID, check_screensize, redraw_all_later, screen_resize, showmode,
    status_redraw_curbuf,
};
use crate::eval::vars::set_vim_var_string;
use crate::ex_docmd::set_no_hlsearch;
use crate::fold::{
    fold_update_all, foldmethod_is_diff, foldmethod_is_indent, foldmethod_is_syntax, new_fold_level,
};
use crate::global_cell::GlobalCell;
use crate::guard::Depth;
use crate::highlight::hl_invalidate_blends;
use crate::indent_c::parse_cino;
use crate::main::{
    Columns, Rows, clear_cmdline, cmdline_row, curtab, e_invarg, firstwin, full_screen, lastwin,
    need_maketitle, p_arshape, p_ch, p_columns, p_deco, p_ea, p_enc, p_hh, p_hls, p_lines, p_lnr,
    p_lrm, p_sj, p_tbidi, p_titlelen, p_uc, p_udf, p_ul, p_wh, p_window, p_wiw, readonlymode,
    starting, topframe, updating_screen,
};
use crate::memfile::mf_close_file;
use crate::memline::{ml_open_file, ml_open_files};
use crate::message::{msg, msg_source};
use crate::r#move::changed_window_setting;
use crate::normal::{do_check_scrollbind, get_vtopline};
use crate::options::{kOptChistory, kOptKeymap, kOptUndolevels, kOptWindow};
use crate::optionstr::check_signcolumn;
use crate::os::cshim::gettext;
use crate::popupmenu::{pum_drawn, pum_redraw};
use crate::quickfix::{ll_resize_stack, qf_resize_stack};
use crate::runtime::{RuntimeOpts, source_runtime_vim_lua};
use crate::spell::parse_spelllang;
use crate::strings::vim_snprintf;
use crate::terminal::on_scrollback_option_changed;
use crate::types::{
    NUL, OptIndex, OptInt, OptVal, OptionSetFlags, String_0, Vv, buf_T, colnr_T, linenr_T,
    optset_T, ptrdiff_t, size_t, uint8_t, win_T,
};
use crate::undo::{buf_is_changed, u_compute_hash, u_read_undo, u_sync};
use crate::window::{
    check_colorcolumn, command_height, frame_new_height, global_stl_height, last_status, min_rows,
    tabline_height, win_comp_pos, win_equal, win_new_screen_rows, win_setheight, win_setwidth,
};
use crate::winfloat::win_float_update_statusline;

use super::{
    B_IMODE_NONE, B_IMODE_USE_INSERT, NO_SCREEN, OptSlot, STATUS_HEIGHT, answer_err,
    check_blending, did_set_title, option_var, option_was_set, redraw_titles, set_option_value,
    set_option_varp, set_options_bin,
};
use crate::highlight_group::HLF_W;
use crate::winlayer::{self, Buf, Win};

use super::field_ptr;

/// "E590", the one message a callback in this module reports.
const E_PREVIEW_WINDOW_EXISTS: &CStr = c"E590: A preview window already exists";

/// What the option table hands a callback, read out of the frame once so
/// that the rest of a callback is plain Rust.
///
/// The accessors below each belong to a callback whose option the table
/// declares as that type; the values now carry their own kind, so a
/// mismatch is a panic naming the option rather than a bad read.
#[derive(Clone, Copy)]
struct Frame {
    /// The variable that already holds the new value.
    varp: OptSlot,
    idx: OptIndex,
    flags: OptionSetFlags,
    old: OptVal,
    new: OptVal,
    /// The window the set is happening in.
    win: Win,
    /// The buffer the set is happening in.
    buf: Buf,
}

impl Frame {
    /// # Safety
    ///
    /// `args` must be the option table's call frame, whose `os_win` and
    /// `os_buf` are the live window and buffer the set is happening in —
    /// `set_option` fills them from `curwin`/`curbuf`.
    unsafe fn read(args: *mut optset_T) -> Self {
        // SAFETY: the caller's frame, and the window and buffer it names.
        Frame {
            varp: unsafe { (*args).os_varp },
            idx: unsafe { (*args).os_idx },
            flags: unsafe { (*args).os_flags },
            old: unsafe { (*args).os_oldval },
            new: unsafe { (*args).os_newval },
            win: unsafe { Win::new((*args).os_win.cast::<win_T>()) },
            buf: unsafe { Buf::new((*args).os_buf.cast::<buf_T>()) },
        }
    }

    fn old_number(self) -> OptInt {
        self.old
            .as_number()
            .expect("the table declares this option numeric")
    }

    fn new_number(self) -> OptInt {
        self.new
            .as_number()
            .expect("the table declares this option numeric")
    }

    /// `None` is the unset global-local marker, not "false".
    fn old_boolean(self) -> Option<bool> {
        self.old.as_boolean()
    }

    fn new_boolean(self) -> Option<bool> {
        self.new.as_boolean()
    }
}

/// 'arabic': a bundle of other settings, plus the Arabic keymap.
pub(crate) unsafe fn did_set_arabic(args: &mut optset_T) -> Option<&CStr> {
    let (keymap, local) = (cstr_optval(c"arabic"), OptionSetFlags::LOCAL);
    // SAFETY: the table's call frame, and the window it names is live.
    let mut win = unsafe { Frame::read(args) }.win;
    if win.w_onebuf_opt.wo_arab == 0 {
        if p_tbidi.get() == 0 && win.w_onebuf_opt.wo_rl != 0 {
            win.w_onebuf_opt.wo_rl = 0;
            changed_window_setting(win);
        }
        // SAFETY: a live window's buffer is live.
        let mut buf = unsafe { Buf::new(win.w_buffer) };
        buf.b_p_iminsert = B_IMODE_NONE as OptInt;
        buf.b_p_imsearch = B_IMODE_USE_INSERT as OptInt;
        return None;
    }

    if p_tbidi.get() == 0 {
        // Right-to-left mode, and the shaping that is most of what
        // Arabic needs.
        if win.w_onebuf_opt.wo_rl == 0 {
            win.w_onebuf_opt.wo_rl = 1;
            changed_window_setting(win);
        }
        if p_arshape.get() == 0 {
            p_arshape.set(1);
            unsafe { redraw_all_later(UPD_NOT_VALID) };
        }
    }
    if unsafe { cstr::bytes_at(p_enc.get()) != b"utf-8" } {
        let warning = c"W17: Arabic requires UTF-8, do ':set encoding=utf-8'";
        unsafe { msg_source(HLF_W) };
        msg(gettext(warning), HLF_W);
        unsafe { set_vim_var_string(Vv::Warningmsg, gettext(warning).as_ptr(), -1 as ptrdiff_t) };
    }
    p_deco.set(1);
    unsafe { answer_err(args, set_option_value(kOptKeymap, keymap, local)) }
}

/// A borrowed string option value. The callee copies it if it keeps it.
fn cstr_optval(value: &'static CStr) -> OptVal {
    OptVal::String(String_0::from_raw_parts(
        value.as_ptr() as *mut c_char,
        value.count_bytes() as size_t,
    ))
}

/// 'autochdir': follow the current file's directory from now on.
pub(crate) unsafe fn did_set_autochdir(_args: &mut optset_T) -> Option<&CStr> {
    do_autochdir();
    None
}

/// 'binary': override four text options for as long as it is on.
pub(crate) unsafe fn did_set_binary(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the buffer it names is live.
    let f = unsafe { Frame::read(args) };
    let bin = f.buf.b_p_bin != 0;
    set_options_bin(f.old_boolean() == Some(true), bin, f.flags);
    redraw_titles();
    None
}

/// 'buflisted': entering or leaving the buffer list is an event.
pub(crate) unsafe fn did_set_buflisted(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the buffer it names is live.
    let f = unsafe { Frame::read(args) };
    if f.old_boolean() != Some(f.buf.b_p_bl != 0) {
        let event = if f.buf.b_p_bl != 0 {
            AutoEvent::BufAdd
        } else {
            AutoEvent::BufDelete
        };
        unsafe {
            apply_autocmds(
                event as AutoEvent,
                ptr::null_mut(),
                ptr::null_mut(),
                true,
                f.buf.raw(),
            )
        };
    }
    None
}

/// 'cmdheight': the command line cannot be taller than the screen leaves
/// room for, and growing or shrinking it moves every window below it.
pub(crate) unsafe fn did_set_cmdheight(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame; the rest reads globals.
    let old_value = unsafe { Frame::read(args) }.old_number();
    let room = (Rows.get() - unsafe { min_rows(curtab.get()) } + 1) as OptInt;
    if p_ch.get() > room {
        p_ch.set(room);
    }
    let laid_out =
        (tabline_height() + global_stl_height() + unsafe { (*topframe.get()).fr_height }) as OptInt;
    if (p_ch.get() != old_value || laid_out != Rows.get() as OptInt - p_ch.get())
        && full_screen.get()
    {
        unsafe { command_height() };
    }
    None
}

/// 'diff': joining or leaving the diff set redoes the folds.
pub(crate) unsafe fn did_set_diff(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the window it names is live.
    let mut win = unsafe { Frame::read(args) }.win;
    diff_buf_adjust(win);
    if foldmethod_is_diff(win) {
        fold_update_all(win);
    }
    None
}

/// 'endoffile', 'endofline', 'fixendofline', 'bomb': all four show in the
/// window title.
pub(crate) unsafe fn did_set_eof_eol_fixeol_bomb(_args: &mut optset_T) -> Option<&CStr> {
    redraw_titles();
    None
}

/// 'equalalways': switching it on evens the windows out once.
pub(crate) unsafe fn did_set_equalalways(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the window it names is live.
    let f = unsafe { Frame::read(args) };
    if p_ea.get() != 0 && f.old_boolean() == Some(false) {
        unsafe { win_equal(f.win.raw(), false, 0) };
    }
    None
}

/// 'foldlevel': open or close folds to match.
pub(crate) unsafe fn did_set_foldlevel(_args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: `curwin` is live.
    unsafe { new_fold_level() };
    None
}

/// 'foldminlines': the fold sizes all change.
pub(crate) unsafe fn did_set_foldminlines(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the window it names is live.
    // SAFETY: the table's call frame, and the window it names is live.
    fold_update_all(unsafe { Frame::read(args) }.win);
    None
}

/// 'foldnestmax': only the two computed fold methods have nesting to cap.
pub(crate) unsafe fn did_set_foldnestmax(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the window it names is live.
    let mut win = unsafe { Frame::read(args) }.win;
    if foldmethod_is_syntax(win) || foldmethod_is_indent(win) {
        fold_update_all(win);
    }
    None
}

/// 'helpheight': grow the current window if it is a help window and now too
/// short.
pub(crate) unsafe fn did_set_helpheight(_args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: `curbuf`/`curwin` are live.
    if firstwin.get() != lastwin.get()
        && cur_buf().b_help
        && (cur_win().w_height as OptInt) < p_hh.get()
    {
        win_setheight(p_hh.get() as c_int);
    }
    None
}

/// 'hlsearch': switching it on un-suppresses the current match highlight.
pub(crate) unsafe fn did_set_hlsearch(_args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: reads globals only.
    unsafe { set_no_hlsearch(false) };
    None
}

/// 'ignorecase': what the search highlight matches changes with it.
pub(crate) unsafe fn did_set_ignorecase(_args: &mut optset_T) -> Option<&CStr> {
    if p_hls.get() != 0 {
        // SAFETY: the screen is the editor's own.
        unsafe { redraw_all_later(UPD_SOME_VALID) };
    }
    None
}

/// 'iminsert': the mode message names the input method.
pub(crate) unsafe fn did_set_iminsert(_args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the screen is the editor's own.
    unsafe { showmode() };
    unsafe { status_redraw_curbuf() };
    None
}

/// 'langnoremap': the deprecated inverse of 'langremap'.
pub(crate) unsafe fn did_set_langnoremap(_args: &mut optset_T) -> Option<&CStr> {
    p_lrm.set((p_lnr.get() == 0) as c_int);
    None
}

/// 'langremap': keeps the deprecated 'langnoremap' in step.
pub(crate) unsafe fn did_set_langremap(_args: &mut optset_T) -> Option<&CStr> {
    p_lnr.set((p_lrm.get() == 0) as c_int);
    None
}

/// 'laststatus': the global status line is a row the frame tree does not
/// own, so entering or leaving value 3 resizes the top frame by hand.
pub(crate) unsafe fn did_set_laststatus(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame; the rest is the window layout.
    let f = unsafe { Frame::read(args) };
    let (old_value, value) = (f.old_number(), f.new_number());
    if value == 3 && old_value != 3 {
        let height = unsafe { (*topframe.get()).fr_height } - STATUS_HEIGHT as c_int;
        unsafe { frame_new_height(topframe.get(), height, false, false, false) };
        win_comp_pos();
        clear_cmdline.set(true);
    }
    if old_value == 3 && value != 3 {
        let height = unsafe { (*topframe.get()).fr_height } + STATUS_HEIGHT as c_int;
        unsafe { frame_new_height(topframe.get(), height, false, false, false) };
        win_comp_pos();
    }
    unsafe { status_redraw_curbuf() };
    last_status(false);
    win_float_update_statusline();
    None
}

/// 'lines'/'columns': a script asking for a different screen size.
///
/// Mid-redraw the request is refused outright — the value is put straight
/// back — because the grid the redraw is writing into cannot be resized
/// under it.
pub(crate) unsafe fn did_set_lines_or_columns(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, whose `varp` is this option's own
    // variable; the rest is the screen.
    let f = unsafe { Frame::read(args) };
    if p_lines.get() != Rows.get() as OptInt || p_columns.get() != Columns.get() as OptInt {
        if updating_screen.get() {
            unsafe { set_option_varp(f.idx, f.varp, f.old, false) };
        } else if full_screen.get() {
            unsafe { screen_resize(p_columns.get() as c_int, p_lines.get() as c_int) };
        } else {
            // Before the screen exists there is nothing to resize; just
            // record the size and keep the command line on screen.
            Rows.set(p_lines.get() as c_int);
            Columns.set(p_columns.get() as c_int);
            unsafe { check_screensize() };
            let new_row = (Rows.get() as OptInt - p_ch.get().max(1)) as c_int;
            if cmdline_row.get() > new_row && Rows.get() as OptInt > p_ch.get() {
                debug_assert!(p_ch.get() >= 0);
                cmdline_row.set(new_row);
            }
        }
        if p_window.get() >= Rows.get() as OptInt || !option_was_set(kOptWindow) {
            p_window.set((Rows.get() - 1) as OptInt);
        }
    }
    if p_sj.get() >= Rows.get() as OptInt && full_screen.get() {
        p_sj.set((Rows.get() / 2) as OptInt);
    }
    None
}

/// 'lisp': the word characters change with it.
pub(crate) unsafe fn did_set_lisp(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the buffer it names is live.
    unsafe { buf_init_chartab(Frame::read(args).buf.raw(), false) };
    None
}

/// 'modifiable': shows in the window title.
pub(crate) unsafe fn did_set_modifiable(_args: &mut optset_T) -> Option<&CStr> {
    redraw_titles();
    None
}

/// 'modified': clearing it makes the buffer's current file format the one it
/// will be written back in.
pub(crate) unsafe fn did_set_modified(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the buffer it names is live.
    let mut f = unsafe { Frame::read(args) };
    if f.new_boolean() == Some(false) {
        save_file_ff(f.buf);
    }
    redraw_titles();
    f.buf.b_modified_was_set = f.new_boolean() != Some(false);
    None
}

/// 'number'/'relativenumber': both change how wide the number column is, and
/// a 'signcolumn' of `number` rides on it.
pub(crate) unsafe fn did_set_number_relativenumber(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the window it names is live.
    let mut win = unsafe { Frame::read(args) }.win;
    // A 'statuscolumn' draws the number itself, so the cached width has
    // to be recomputed rather than reused.
    if (unsafe { *win.w_onebuf_opt.wo_stc }) != NUL as c_char {
        win.w_nrwidth_line_count = 0 as linenr_T;
    }
    let _ = unsafe { check_signcolumn(ptr::null_mut(), win.raw()) };
    None
}

/// 'numberwidth': the cached number-column width is stale.
pub(crate) unsafe fn did_set_numberwidth(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the window it names is live.
    unsafe { Frame::read(args).win.w_nrwidth_line_count = 0 as linenr_T };
    None
}

/// 'previewwindow': there can be only one, in the current tab page.
pub(crate) unsafe fn did_set_previewwindow(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the window list is the editor's.
    let mut win = unsafe { Frame::read(args) }.win;
    if win.w_onebuf_opt.wo_pvw == 0 {
        return None;
    }
    for wp in winlayer::windows() {
        if wp.w_onebuf_opt.wo_pvw != 0 && wp != win {
            win.w_onebuf_opt.wo_pvw = 0;
            return Some(E_PREVIEW_WINDOW_EXISTS);
        }
    }
    None
}

/// 'pumblend': the blend of every highlight group is derived from it.
pub(crate) unsafe fn did_set_pumblend(_args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the highlight table and the popup menu are the editor's own.
    unsafe { hl_invalidate_blends() };
    if pum_drawn() {
        unsafe { pum_redraw() };
    }
    None
}

/// 'readonly': clearing it globally also clears the `-R` command-line flag,
/// and setting it re-arms the "changing a readonly file" warning.
pub(crate) unsafe fn did_set_readonly(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the buffer it names is live.
    let mut f = unsafe { Frame::read(args) };
    if f.buf.b_p_ro == 0 && !f.flags.has(OptionSetFlags::LOCAL) {
        readonlymode.set(false);
    }
    if f.buf.b_p_ro != 0 {
        f.buf.b_did_warn = false;
    }
    redraw_titles();
    None
}

/// 'scrollback': only shrinking one has anything for the terminal to do.
pub(crate) unsafe fn did_set_scrollback(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the buffer it names is live.
    let f = unsafe { Frame::read(args) };
    if !f.buf.terminal.is_null() && f.new_number() < f.old_number() {
        unsafe { on_scrollback_option_changed(f.buf.terminal) };
    }
    None
}

/// 'scrollbind': line the window up with the others straight away.
pub(crate) unsafe fn did_set_scrollbind(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the window it names is live.
    let mut win = unsafe { Frame::read(args) }.win;
    if win.w_onebuf_opt.wo_scb == 0 {
        return None;
    }
    unsafe { do_check_scrollbind(false) };
    win.w_scbind_pos = get_vtopline(win);
    None
}

/// 'shiftwidth'/'tabstop': indent folds and the C indenter both read them.
pub(crate) unsafe fn did_set_shiftwidth_tabstop(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the window and buffer it names.
    let f = unsafe { Frame::read(args) };
    if foldmethod_is_indent(f.win) {
        fold_update_all(f.win);
    }
    // A zero 'shiftwidth' means "use 'tabstop'", so 'tabstop' feeds the
    // C indent options too.
    let own_sw = field_ptr(f.buf.raw(), offset_of!(buf_T, b_p_sw), |b: &buf_T| {
        &b.b_p_sw
    });
    if f.varp == OptSlot::Number(own_sw) || f.buf.b_p_sw == 0 {
        unsafe { parse_cino(f.buf) };
    }
    None
}

/// 'showtabline': the tab line takes a screen row from the windows.
pub(crate) unsafe fn did_set_showtabline(_args: &mut optset_T) -> Option<&CStr> {
    win_new_screen_rows();
    None
}

/// 'smoothscroll': switching it off drops any partial-line scroll.
pub(crate) unsafe fn did_set_smoothscroll(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the window it names is live.
    let mut win = unsafe { Frame::read(args) }.win;
    if win.w_onebuf_opt.wo_sms == 0 {
        win.w_skipcol = 0 as colnr_T;
    }
    None
}

/// 'spell': switching it on is what loads the word lists.
pub(crate) unsafe fn did_set_spell(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the window it names is live.
    let mut win = unsafe { Frame::read(args) }.win;
    if win.w_onebuf_opt.wo_spell != 0 {
        return unsafe { parse_spelllang(win.raw()) };
    }
    None
}

/// 'swapfile': open or close the buffer's swap file to match.
pub(crate) unsafe fn did_set_swapfile(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the buffer it names is live.
    let buf = unsafe { Frame::read(args) }.buf;
    if buf.b_p_swf != 0 && p_uc.get() != 0 {
        unsafe { ml_open_file(buf.raw()) };
    } else {
        unsafe { mf_close_file(buf.raw(), true) };
    }
    None
}

/// 'textwidth': a 'colorcolumn' entry may be relative to it, in any window.
pub(crate) unsafe fn did_set_textwidth(_args: &mut optset_T) -> Option<&CStr> {
    for wp in winlayer::tab_windows() {
        // SAFETY: `wp` is a live window of the editor's own list.
        unsafe { check_colorcolumn(ptr::null_mut(), wp.raw()) };
    }
    None
}

/// 'title'/'icon': rebuild what the terminal is showing.
pub(crate) unsafe fn did_set_title_icon(_args: &mut optset_T) -> Option<&CStr> {
    did_set_title();
    None
}

/// 'titlelen': the title is a percentage of the window width, so it has to
/// be rebuilt — but not before there is a screen to show it on.
pub(crate) unsafe fn did_set_titlelen(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame.
    let old_value = unsafe { Frame::read(args) }.old_number();
    if starting.get() != NO_SCREEN && old_value != p_titlelen.get() {
        need_maketitle.set(true);
    }
    None
}

/// 'undofile': switching it on reads back the undo file of every unmodified
/// buffer it now applies to.
pub(crate) unsafe fn did_set_undofile(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the buffer list is the editor's.
    let f = unsafe { Frame::read(args) };
    if f.buf.b_p_udf == 0 && p_udf.get() == 0 {
        return None;
    }
    let mut hash: [uint8_t; 32] = [0; 32];
    for bp in winlayer::buffers() {
        // A `:setlocal` only reaches its own buffer; `:set` and
        // `:setglobal` reach all of them.
        let reaches = bp == f.buf || f.flags.has(OptionSetFlags::GLOBAL) || f.flags.is_empty();
        if reaches && !buf_is_changed(bp) && !bp.b_ml.ml_mfp.is_null() {
            let (hash, fname) = (hash.as_mut_ptr(), bp.b_fname);
            // SAFETY: `hash` is 32 bytes, which is what both want, and
            // `b_fname` is the buffer's own name.
            unsafe { u_compute_hash(bp, hash) };
            unsafe { u_read_undo(ptr::null_mut(), hash, fname) };
        }
    }
    None
}

/// 'undolevels': the pending change has to be closed off under the *old*
/// limit before the new one takes effect.
pub(crate) unsafe fn did_set_undolevels(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the buffer it names is live.
    let mut f = unsafe { Frame::read(args) };
    let pp = f.varp.number_var();
    let own_ul = field_ptr(f.buf.raw(), offset_of!(buf_T, b_p_ul), |b: &buf_T| {
        &b.b_p_ul
    });
    let (value, old_value) = (f.new_number(), f.old_number());
    if f.varp == option_var(kOptUndolevels) {
        p_ul.set(old_value);
        u_sync(true);
        p_ul.set(value);
    } else if pp == own_ul {
        f.buf.b_p_ul = old_value;
        u_sync(true);
        f.buf.b_p_ul = value;
    }
    None
}

/// 'updatecount': switching it on from zero is what opens the swap files.
pub(crate) unsafe fn did_set_updatecount(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the buffer list is the editor's.
    if p_uc.get() != 0 && unsafe { Frame::read(args) }.old_number() == 0 {
        unsafe { ml_open_files() };
    }
    None
}

/// 'wildchar'/'wildcharm': a key that already means something on the command
/// line cannot also start completion.
pub(crate) unsafe fn did_set_wildchar(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, whose `varp` is this numeric option's
    // own variable.
    let c = unsafe { *Frame::read(args).varp.number_var() };
    if c == Ctrl_C as OptInt
        || c == '\n' as OptInt
        || c == '\r' as OptInt
        || c == Key::Kenter.code() as OptInt
    {
        return Some(e_invarg);
    }
    None
}

/// 'winblend': the window's own blend, clamped, plus the highlight groups
/// that depend on it.
pub(crate) unsafe fn did_set_winblend(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the window it names is live.
    let f = unsafe { Frame::read(args) };
    if f.new_number() != f.old_number() {
        let mut win = f.win;
        win.w_onebuf_opt.wo_winbl = win.w_onebuf_opt.wo_winbl.clamp(0, 100);
        win.w_hl_needs_update = true;
        unsafe { check_blending(win.raw()) };
    }
    None
}

/// 'window': the scroll amount `CTRL-F` uses, capped at the screen.
pub(crate) unsafe fn did_set_window(_args: &mut optset_T) -> Option<&CStr> {
    if p_window.get() < 1 || p_window.get() >= Rows.get() as OptInt {
        p_window.set((Rows.get() - 1) as OptInt);
    }
    None
}

/// 'winheight': grow the current window if it is now too short.
pub(crate) unsafe fn did_set_winheight(_args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: `curwin` is live.
    if firstwin.get() != lastwin.get() && (cur_win().w_height as OptInt) < p_wh.get() {
        win_setheight(p_wh.get() as c_int);
    }
    None
}

/// 'winwidth': widen the current window if it is now too narrow.
pub(crate) unsafe fn did_set_winwidth(_args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: `curwin` is live.
    if firstwin.get() != lastwin.get() && (cur_win().w_width as OptInt) < p_wiw.get() {
        win_setwidth(p_wiw.get() as c_int);
    }
    None
}

/// 'wrap': the two scroll offsets are exclusive — a wrapped window scrolls
/// within a line, an unwrapped one scrolls sideways.
pub(crate) unsafe fn did_set_wrap(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, and the window it names is live.
    let mut win = unsafe { Frame::read(args) }.win;
    if win.w_onebuf_opt.wo_wrap != 0 {
        win.w_leftcol = 0 as colnr_T;
    } else {
        win.w_skipcol = 0 as colnr_T;
    }
    None
}

/// 'chistory'/'lhistory': how many quickfix or location lists to keep.
pub(crate) unsafe fn did_set_xhistory(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the table's call frame, whose `varp` is this numeric option's
    // own variable, and the window it names is live.
    let f = unsafe { Frame::read(args) };
    let arg = f.varp.number_var();
    if f.varp == option_var(kOptChistory) {
        qf_resize_stack(unsafe { *arg } as c_int);
    } else {
        ll_resize_stack(f.win, unsafe { *arg } as c_int);
    }
    None
}

/// Fire the `Syntax` autocommand for a buffer whose 'syntax' just changed.
///
/// The recursion counter is what lets a syntax file set 'syntax' again (to
/// include another one) without the outer pass being treated as a no-op.
///
/// # Safety
///
/// `buf` must be a live buffer.
pub(crate) unsafe fn do_syntax_autocmd(buf: *mut buf_T, value_changed: bool) {
    static syn_recursive: GlobalCell<c_int> = GlobalCell::new(0);

    let _syn_recursive = Depth::of(&syn_recursive);
    // SAFETY: the caller's buffer is live.
    unsafe { (*buf).b_flags |= BufFlags::SYN_SET };
    unsafe {
        apply_autocmds(
            AutoEvent::Syntax,
            (*buf).b_p_syn,
            (*buf).b_fname,
            value_changed || syn_recursive.get() == 1,
            buf,
        )
    };
}

/// Source `spell/<lang>.vim` for a window whose 'spelllang' just changed.
///
/// Only the leading language name is used, and only the letters, digits and
/// hyphens of it — the rest of the value is regions and file names.
///
/// # Safety
///
/// `win` must be a live window.
pub(crate) unsafe fn do_spelllang_source(win: *mut win_T) {
    let mut fname: [c_char; 200] = [0; 200];

    // SAFETY: the caller's window is live, and its 'spelllang' is a
    // NUL-terminated option value.
    let mut q = unsafe { (*(*win).w_s).b_p_spl };
    // "cjk" is a modifier, not a language.
    if unsafe { cstr::starts_with(q, b"cjk,") } {
        q = unsafe { q.add(4) };
    }
    let mut p = q;
    while unsafe { *p } != NUL as c_char {
        let c = unsafe { *p } as c_int;
        if !((unsafe { *p } as u8).is_ascii_alphabetic() || ascii_isdigit(c) || c == '-' as c_int) {
            break;
        }
        p = unsafe { p.add(1) };
    }
    if p > q {
        unsafe {
            vim_snprintf(
                fname.as_mut_ptr(),
                size_of::<[c_char; 200]>(),
                c"spell/%.*s.*".as_ptr(),
                p.offset_from(q) as c_int,
                q,
            )
        };
        let _ = unsafe { source_runtime_vim_lua(fname.as_mut_ptr(), RuntimeOpts::ALL) };
    }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
