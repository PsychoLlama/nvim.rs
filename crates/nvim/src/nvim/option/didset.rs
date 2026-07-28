//! The callbacks the option table names for a boolean or numeric option
//! that just changed. The string options' callbacks live in
//! [`crate::src::nvim::optionstr`].
//!
//! Each one runs *after* the new value is already in the option's variable,
//! and reports a message when it does not like it — [`super::did_set_option`]
//! then puts the old value back. So a callback that only validates can
//! return early, but one that also updates derived state has to validate
//! first.
//!
//! They stay `pub unsafe extern "C" fn` because the generated table holds
//! them as `opt_did_set_cb_T` function pointers; nothing else in this module
//! needs that shape.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::autocmd::apply_autocmds;
use crate::src::nvim::buffer::do_autochdir;
use crate::src::nvim::change::save_file_ff;
use crate::src::nvim::charset::buf_init_chartab;
use crate::src::nvim::diff::diff_buf_adjust;
use crate::src::nvim::drawscreen::{
    check_screensize, redraw_all_later, screen_resize, showmode, status_redraw_curbuf,
};
use crate::src::nvim::eval::vars::set_vim_var_string;
use crate::src::nvim::ex_docmd::set_no_hlsearch;
use crate::src::nvim::fold::{
    foldUpdateAll, foldmethodIsDiff, foldmethodIsIndent, foldmethodIsSyntax, newFoldLevel,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight::hl_invalidate_blends;
use crate::src::nvim::indent_c::parse_cino;
use crate::src::nvim::main::{
    Columns, Rows, clear_cmdline, cmdline_row, curbuf, curtab, curwin, e_invarg, first_tabpage,
    firstbuf, firstwin, full_screen, lastwin, need_maketitle, p_arshape, p_ch, p_chi, p_columns,
    p_deco, p_ea, p_enc, p_hh, p_hls, p_lines, p_lnr, p_lrm, p_sj, p_tbidi, p_titlelen, p_uc,
    p_udf, p_ul, p_wh, p_window, p_wiw, readonlymode, starting, topframe, updating_screen,
};
use crate::src::nvim::memfile::mf_close_file;
use crate::src::nvim::memline::{ml_open_file, ml_open_files};
use crate::src::nvim::message::{msg, msg_source};
use crate::src::nvim::r#move::changed_window_setting;
use crate::src::nvim::normal::{do_check_scrollbind, get_vtopline};
use crate::src::nvim::options::{kOptKeymap, kOptWindow};
use crate::src::nvim::optionstr::check_signcolumn;
use crate::src::nvim::os::libc::{gettext, strcmp, strncmp};
use crate::src::nvim::popupmenu::{pum_drawn, pum_redraw};
use crate::src::nvim::quickfix::{ll_resize_stack, qf_resize_stack};
use crate::src::nvim::runtime::source_runtime_vim_lua;
use crate::src::nvim::spell::parse_spelllang;
use crate::src::nvim::strings::vim_snprintf;
use crate::src::nvim::terminal::on_scrollback_option_changed;
use crate::src::nvim::types::{
    OptIndex, OptInt, OptVal, OptValData, String_0, TriState, buf_T, colnr_T, event_T, linenr_T,
    optset_T, ptrdiff_t, size_t, tabpage_T, uint8_t, win_T,
};
use crate::src::nvim::undo::{bufIsChanged, u_compute_hash, u_read_undo, u_sync};
use crate::src::nvim::window::{
    check_colorcolumn, command_height, frame_new_height, global_stl_height, last_status, min_rows,
    tabline_height, win_comp_pos, win_equal, win_new_screen_rows, win_setheight, win_setwidth,
};
use crate::src::nvim::winfloat::win_float_update_statusline;

use super::{
    B_IMODE_NONE, B_IMODE_USE_INSERT, BF_SYN_SET, Ctrl_C, DIP_ALL, EVENT_BUFADD, EVENT_BUFDELETE,
    EVENT_SYNTAX, HLF_W, K_KENTER, NO_SCREEN, NUL, OPT_GLOBAL, OPT_LOCAL, STATUS_HEIGHT,
    UPD_NOT_VALID, UPD_SOME_VALID, VV_WARNINGMSG, check_blending, did_set_title, kFalse,
    kOptValTypeNumber, kOptValTypeString, option_was_set, redraw_titles, set_option_value,
    set_option_varp, set_options_bin,
};

/// "E590", the one message a callback in this module reports.
const E_PREVIEW_WINDOW_EXISTS: &CStr = c"E590: A preview window already exists";

/// What the option table hands a callback, read out of the frame once so
/// that the rest of a callback is plain Rust.
///
/// The union arms are only read through the accessors below, each of which
/// is used by a callback whose option the table declares as that type.
#[derive(Clone, Copy)]
struct Frame {
    /// The variable that already holds the new value.
    varp: *mut c_void,
    idx: OptIndex,
    flags: c_int,
    old: OptValData,
    new: OptValData,
    /// The window the set is happening in.
    win: *mut win_T,
    /// The buffer the set is happening in.
    buf: *mut buf_T,
}

impl Frame {
    /// # Safety
    ///
    /// `args` must be the option table's call frame.
    unsafe fn read(args: *mut optset_T) -> Self {
        // SAFETY: the caller's frame.
        unsafe {
            Frame {
                varp: (*args).os_varp,
                idx: (*args).os_idx,
                flags: (*args).os_flags,
                old: (*args).os_oldval,
                new: (*args).os_newval,
                win: (*args).os_win.cast::<win_T>(),
                buf: (*args).os_buf.cast::<buf_T>(),
            }
        }
    }

    fn old_number(self) -> OptInt {
        // SAFETY: the table declares this option numeric.
        unsafe { self.old.number }
    }

    fn new_number(self) -> OptInt {
        // SAFETY: the table declares this option numeric.
        unsafe { self.new.number }
    }

    fn old_boolean(self) -> TriState {
        // SAFETY: the table declares this option boolean.
        unsafe { self.old.boolean }
    }

    fn new_boolean(self) -> TriState {
        // SAFETY: the table declares this option boolean.
        unsafe { self.new.boolean }
    }
}

/// 'arabic': a bundle of other settings, plus the Arabic keymap.
pub unsafe extern "C" fn did_set_arabic(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the window it names is live.
    unsafe {
        let win = Frame::read(args).win;
        if (*win).w_onebuf_opt.wo_arab == 0 {
            if p_tbidi.get() == 0 && (*win).w_onebuf_opt.wo_rl != 0 {
                (*win).w_onebuf_opt.wo_rl = 0;
                changed_window_setting(win);
            }
            (*(*win).w_buffer).b_p_iminsert = B_IMODE_NONE as OptInt;
            (*(*win).w_buffer).b_p_imsearch = B_IMODE_USE_INSERT as OptInt;
            return ptr::null();
        }

        if p_tbidi.get() == 0 {
            // Right-to-left mode, and the shaping that is most of what
            // Arabic needs.
            if (*win).w_onebuf_opt.wo_rl == 0 {
                (*win).w_onebuf_opt.wo_rl = 1;
                changed_window_setting(win);
            }
            if p_arshape.get() == 0 {
                p_arshape.set(1);
                redraw_all_later(UPD_NOT_VALID as c_int);
            }
        }
        if strcmp(p_enc.get(), c"utf-8".as_ptr()) != 0 {
            let warning = c"W17: Arabic requires UTF-8, do ':set encoding=utf-8'";
            msg_source(HLF_W as c_int);
            msg(gettext(warning.as_ptr()), HLF_W as c_int);
            set_vim_var_string(VV_WARNINGMSG, gettext(warning.as_ptr()), -1 as ptrdiff_t);
        }
        p_deco.set(1);
        set_option_value(kOptKeymap, cstr_optval(c"arabic"), OPT_LOCAL)
    }
}

/// A borrowed string option value. The callee copies it if it keeps it.
fn cstr_optval(value: &'static CStr) -> OptVal {
    OptVal {
        type_0: kOptValTypeString,
        data: OptValData {
            string: String_0 {
                data: value.as_ptr() as *mut c_char,
                size: value.count_bytes() as size_t,
            },
        },
    }
}

/// 'autochdir': follow the current file's directory from now on.
pub unsafe extern "C" fn did_set_autochdir(_args: *mut optset_T) -> *const c_char {
    // SAFETY: `curbuf` is live.
    unsafe { do_autochdir() };
    ptr::null()
}

/// 'binary': override four text options for as long as it is on.
pub unsafe extern "C" fn did_set_binary(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the buffer it names is live.
    unsafe {
        let f = Frame::read(args);
        set_options_bin(f.old_boolean(), (*f.buf).b_p_bin, f.flags);
    }
    redraw_titles();
    ptr::null()
}

/// 'buflisted': entering or leaving the buffer list is an event.
pub unsafe extern "C" fn did_set_buflisted(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the buffer it names is live.
    unsafe {
        let f = Frame::read(args);
        if f.old_boolean() != (*f.buf).b_p_bl {
            let event = if (*f.buf).b_p_bl != 0 {
                EVENT_BUFADD
            } else {
                EVENT_BUFDELETE
            };
            apply_autocmds(
                event as event_T,
                ptr::null_mut(),
                ptr::null_mut(),
                true,
                f.buf,
            );
        }
    }
    ptr::null()
}

/// 'cmdheight': the command line cannot be taller than the screen leaves
/// room for, and growing or shrinking it moves every window below it.
pub unsafe extern "C" fn did_set_cmdheight(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame; the rest reads globals.
    unsafe {
        let old_value = Frame::read(args).old_number();
        let room = (Rows.get() - min_rows(curtab.get()) + 1) as OptInt;
        if p_ch.get() > room {
            p_ch.set(room);
        }
        let laid_out =
            (tabline_height() + global_stl_height() + (*topframe.get()).fr_height) as OptInt;
        if (p_ch.get() != old_value || laid_out != Rows.get() as OptInt - p_ch.get())
            && full_screen.get()
        {
            command_height();
        }
    }
    ptr::null()
}

/// 'diff': joining or leaving the diff set redoes the folds.
pub unsafe extern "C" fn did_set_diff(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the window it names is live.
    unsafe {
        let win = Frame::read(args).win;
        diff_buf_adjust(win);
        if foldmethodIsDiff(win) {
            foldUpdateAll(win);
        }
    }
    ptr::null()
}

/// 'endoffile', 'endofline', 'fixendofline', 'bomb': all four show in the
/// window title.
pub unsafe extern "C" fn did_set_eof_eol_fixeol_bomb(_args: *mut optset_T) -> *const c_char {
    redraw_titles();
    ptr::null()
}

/// 'equalalways': switching it on evens the windows out once.
pub unsafe extern "C" fn did_set_equalalways(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the window it names is live.
    unsafe {
        let f = Frame::read(args);
        if p_ea.get() != 0 && f.old_boolean() == kFalse {
            win_equal(f.win, false, 0);
        }
    }
    ptr::null()
}

/// 'foldlevel': open or close folds to match.
pub unsafe extern "C" fn did_set_foldlevel(_args: *mut optset_T) -> *const c_char {
    // SAFETY: `curwin` is live.
    unsafe { newFoldLevel() };
    ptr::null()
}

/// 'foldminlines': the fold sizes all change.
pub unsafe extern "C" fn did_set_foldminlines(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the window it names is live.
    unsafe { foldUpdateAll(Frame::read(args).win) };
    ptr::null()
}

/// 'foldnestmax': only the two computed fold methods have nesting to cap.
pub unsafe extern "C" fn did_set_foldnestmax(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the window it names is live.
    unsafe {
        let win = Frame::read(args).win;
        if foldmethodIsSyntax(win) || foldmethodIsIndent(win) {
            foldUpdateAll(win);
        }
    }
    ptr::null()
}

/// 'helpheight': grow the current window if it is a help window and now too
/// short.
pub unsafe extern "C" fn did_set_helpheight(_args: *mut optset_T) -> *const c_char {
    // SAFETY: `curbuf`/`curwin` are live.
    unsafe {
        if firstwin.get() != lastwin.get()
            && (*curbuf.get()).b_help
            && ((*curwin.get()).w_height as OptInt) < p_hh.get()
        {
            win_setheight(p_hh.get() as c_int);
        }
    }
    ptr::null()
}

/// 'hlsearch': switching it on un-suppresses the current match highlight.
pub unsafe extern "C" fn did_set_hlsearch(_args: *mut optset_T) -> *const c_char {
    // SAFETY: reads globals only.
    unsafe { set_no_hlsearch(false) };
    ptr::null()
}

/// 'ignorecase': what the search highlight matches changes with it.
pub unsafe extern "C" fn did_set_ignorecase(_args: *mut optset_T) -> *const c_char {
    if p_hls.get() != 0 {
        // SAFETY: the screen is the editor's own.
        unsafe { redraw_all_later(UPD_SOME_VALID as c_int) };
    }
    ptr::null()
}

/// 'iminsert': the mode message names the input method.
pub unsafe extern "C" fn did_set_iminsert(_args: *mut optset_T) -> *const c_char {
    // SAFETY: the screen is the editor's own.
    unsafe {
        showmode();
        status_redraw_curbuf();
    }
    ptr::null()
}

/// 'langnoremap': the deprecated inverse of 'langremap'.
pub unsafe extern "C" fn did_set_langnoremap(_args: *mut optset_T) -> *const c_char {
    p_lrm.set((p_lnr.get() == 0) as c_int);
    ptr::null()
}

/// 'langremap': keeps the deprecated 'langnoremap' in step.
pub unsafe extern "C" fn did_set_langremap(_args: *mut optset_T) -> *const c_char {
    p_lnr.set((p_lrm.get() == 0) as c_int);
    ptr::null()
}

/// 'laststatus': the global status line is a row the frame tree does not
/// own, so entering or leaving value 3 resizes the top frame by hand.
pub unsafe extern "C" fn did_set_laststatus(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame; the rest is the window layout.
    unsafe {
        let f = Frame::read(args);
        let (old_value, value) = (f.old_number(), f.new_number());
        if value == 3 && old_value != 3 {
            let height = (*topframe.get()).fr_height - STATUS_HEIGHT as c_int;
            frame_new_height(topframe.get(), height, false, false, false);
            win_comp_pos();
            clear_cmdline.set(true);
        }
        if old_value == 3 && value != 3 {
            let height = (*topframe.get()).fr_height + STATUS_HEIGHT as c_int;
            frame_new_height(topframe.get(), height, false, false, false);
            win_comp_pos();
        }
        status_redraw_curbuf();
        last_status(false);
        win_float_update_statusline();
    }
    ptr::null()
}

/// 'lines'/'columns': a script asking for a different screen size.
///
/// Mid-redraw the request is refused outright — the value is put straight
/// back — because the grid the redraw is writing into cannot be resized
/// under it.
pub unsafe extern "C" fn did_set_lines_or_columns(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, whose `varp` is this option's own
    // variable; the rest is the screen.
    unsafe {
        let f = Frame::read(args);
        if p_lines.get() != Rows.get() as OptInt || p_columns.get() != Columns.get() as OptInt {
            if updating_screen.get() {
                let oldval = OptVal {
                    type_0: kOptValTypeNumber,
                    data: f.old,
                };
                set_option_varp(f.idx, f.varp, oldval, false);
            } else if full_screen.get() {
                screen_resize(p_columns.get() as c_int, p_lines.get() as c_int);
            } else {
                // Before the screen exists there is nothing to resize; just
                // record the size and keep the command line on screen.
                Rows.set(p_lines.get() as c_int);
                Columns.set(p_columns.get() as c_int);
                check_screensize();
                let new_row = (Rows.get() as OptInt - p_ch.get().max(1)) as c_int;
                if cmdline_row.get() > new_row && Rows.get() as OptInt > p_ch.get() {
                    assert!(p_ch.get() >= 0);
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
    }
    ptr::null()
}

/// 'lisp': the word characters change with it.
pub unsafe extern "C" fn did_set_lisp(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the buffer it names is live.
    unsafe { buf_init_chartab(Frame::read(args).buf, false) };
    ptr::null()
}

/// 'modifiable': shows in the window title.
pub unsafe extern "C" fn did_set_modifiable(_args: *mut optset_T) -> *const c_char {
    redraw_titles();
    ptr::null()
}

/// 'modified': clearing it makes the buffer's current file format the one it
/// will be written back in.
pub unsafe extern "C" fn did_set_modified(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the buffer it names is live.
    unsafe {
        let f = Frame::read(args);
        if f.new_boolean() == kFalse {
            save_file_ff(f.buf);
        }
        redraw_titles();
        (*f.buf).b_modified_was_set = f.new_boolean() != 0;
    }
    ptr::null()
}

/// 'number'/'relativenumber': both change how wide the number column is, and
/// a 'signcolumn' of `number` rides on it.
pub unsafe extern "C" fn did_set_number_relativenumber(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the window it names is live.
    unsafe {
        let win = Frame::read(args).win;
        // A 'statuscolumn' draws the number itself, so the cached width has
        // to be recomputed rather than reused.
        if *(*win).w_onebuf_opt.wo_stc != NUL as c_char {
            (*win).w_nrwidth_line_count = 0 as linenr_T;
        }
        check_signcolumn(ptr::null_mut(), win);
    }
    ptr::null()
}

/// 'numberwidth': the cached number-column width is stale.
pub unsafe extern "C" fn did_set_numberwidth(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the window it names is live.
    unsafe { (*Frame::read(args).win).w_nrwidth_line_count = 0 as linenr_T };
    ptr::null()
}

/// Every buffer, oldest first.
pub(crate) fn buffers() -> impl Iterator<Item = *mut buf_T> {
    let mut next = firstbuf.get();
    core::iter::from_fn(move || {
        let buf = next;
        if buf.is_null() {
            return None;
        }
        // SAFETY: the buffer list is the editor's own and is not being
        // rebuilt while an option callback runs.
        next = unsafe { (*buf).b_next };
        Some(buf)
    })
}

/// 'previewwindow': there can be only one, in the current tab page.
pub unsafe extern "C" fn did_set_previewwindow(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the window list is the editor's.
    unsafe {
        let win = Frame::read(args).win;
        if (*win).w_onebuf_opt.wo_pvw == 0 {
            return ptr::null();
        }
        let mut wp = firstwin.get();
        while !wp.is_null() {
            if (*wp).w_onebuf_opt.wo_pvw != 0 && wp != win {
                (*win).w_onebuf_opt.wo_pvw = 0;
                return E_PREVIEW_WINDOW_EXISTS.as_ptr();
            }
            wp = (*wp).w_next;
        }
    }
    ptr::null()
}

/// 'pumblend': the blend of every highlight group is derived from it.
pub unsafe extern "C" fn did_set_pumblend(_args: *mut optset_T) -> *const c_char {
    // SAFETY: the highlight table and the popup menu are the editor's own.
    unsafe {
        hl_invalidate_blends();
        if pum_drawn() {
            pum_redraw();
        }
    }
    ptr::null()
}

/// 'readonly': clearing it globally also clears the `-R` command-line flag,
/// and setting it re-arms the "changing a readonly file" warning.
pub unsafe extern "C" fn did_set_readonly(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the buffer it names is live.
    unsafe {
        let f = Frame::read(args);
        if (*f.buf).b_p_ro == 0 && f.flags & OPT_LOCAL == 0 {
            readonlymode.set(false);
        }
        if (*f.buf).b_p_ro != 0 {
            (*f.buf).b_did_warn = false;
        }
    }
    redraw_titles();
    ptr::null()
}

/// 'scrollback': only shrinking one has anything for the terminal to do.
pub unsafe extern "C" fn did_set_scrollback(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the buffer it names is live.
    unsafe {
        let f = Frame::read(args);
        if !(*f.buf).terminal.is_null() && f.new_number() < f.old_number() {
            on_scrollback_option_changed((*f.buf).terminal);
        }
    }
    ptr::null()
}

/// 'scrollbind': line the window up with the others straight away.
pub unsafe extern "C" fn did_set_scrollbind(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the window it names is live.
    unsafe {
        let win = Frame::read(args).win;
        if (*win).w_onebuf_opt.wo_scb == 0 {
            return ptr::null();
        }
        do_check_scrollbind(false);
        (*win).w_scbind_pos = get_vtopline(win);
    }
    ptr::null()
}

/// 'shiftwidth'/'tabstop': indent folds and the C indenter both read them.
pub unsafe extern "C" fn did_set_shiftwidth_tabstop(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the window and buffer it names.
    unsafe {
        let f = Frame::read(args);
        if foldmethodIsIndent(f.win) {
            foldUpdateAll(f.win);
        }
        // A zero 'shiftwidth' means "use 'tabstop'", so 'tabstop' feeds the
        // C indent options too.
        if f.varp.cast::<OptInt>() == &raw mut (*f.buf).b_p_sw || (*f.buf).b_p_sw == 0 {
            parse_cino(f.buf);
        }
    }
    ptr::null()
}

/// 'showtabline': the tab line takes a screen row from the windows.
pub unsafe extern "C" fn did_set_showtabline(_args: *mut optset_T) -> *const c_char {
    // SAFETY: the window layout is the editor's own.
    unsafe { win_new_screen_rows() };
    ptr::null()
}

/// 'smoothscroll': switching it off drops any partial-line scroll.
pub unsafe extern "C" fn did_set_smoothscroll(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the window it names is live.
    unsafe {
        let win = Frame::read(args).win;
        if (*win).w_onebuf_opt.wo_sms == 0 {
            (*win).w_skipcol = 0 as colnr_T;
        }
    }
    ptr::null()
}

/// 'spell': switching it on is what loads the word lists.
pub unsafe extern "C" fn did_set_spell(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the window it names is live.
    unsafe {
        let win = Frame::read(args).win;
        if (*win).w_onebuf_opt.wo_spell != 0 {
            return parse_spelllang(win);
        }
    }
    ptr::null()
}

/// 'swapfile': open or close the buffer's swap file to match.
pub unsafe extern "C" fn did_set_swapfile(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the buffer it names is live.
    unsafe {
        let buf = Frame::read(args).buf;
        if (*buf).b_p_swf != 0 && p_uc.get() != 0 {
            ml_open_file(buf);
        } else {
            mf_close_file(buf, true);
        }
    }
    ptr::null()
}

/// 'textwidth': a 'colorcolumn' entry may be relative to it, in any window.
pub unsafe extern "C" fn did_set_textwidth(_args: *mut optset_T) -> *const c_char {
    // SAFETY: the tab page and window lists are the editor's own.
    unsafe {
        let mut tp: *mut tabpage_T = first_tabpage.get();
        while !tp.is_null() {
            let mut wp = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                check_colorcolumn(ptr::null_mut(), wp);
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next;
        }
    }
    ptr::null()
}

/// 'title'/'icon': rebuild what the terminal is showing.
pub unsafe extern "C" fn did_set_title_icon(_args: *mut optset_T) -> *const c_char {
    did_set_title();
    ptr::null()
}

/// 'titlelen': the title is a percentage of the window width, so it has to
/// be rebuilt — but not before there is a screen to show it on.
pub unsafe extern "C" fn did_set_titlelen(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame.
    let old_value = unsafe { Frame::read(args) }.old_number();
    if starting.get() != NO_SCREEN && old_value != p_titlelen.get() {
        need_maketitle.set(true);
    }
    ptr::null()
}

/// 'undofile': switching it on reads back the undo file of every unmodified
/// buffer it now applies to.
pub unsafe extern "C" fn did_set_undofile(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the buffer list is the editor's.
    unsafe {
        let f = Frame::read(args);
        if (*f.buf).b_p_udf == 0 && p_udf.get() == 0 {
            return ptr::null();
        }
        let mut hash: [uint8_t; 32] = [0; 32];
        for bp in buffers() {
            // A `:setlocal` only reaches its own buffer; `:set` and
            // `:setglobal` reach all of them.
            let reaches = bp == f.buf || f.flags & OPT_GLOBAL != 0 || f.flags == 0;
            if reaches && !bufIsChanged(bp) && !(*bp).b_ml.ml_mfp.is_null() {
                u_compute_hash(bp, hash.as_mut_ptr());
                u_read_undo(ptr::null_mut(), hash.as_mut_ptr(), (*bp).b_fname);
            }
        }
    }
    ptr::null()
}

/// 'undolevels': the pending change has to be closed off under the *old*
/// limit before the new one takes effect.
pub unsafe extern "C" fn did_set_undolevels(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the buffer it names is live.
    unsafe {
        let f = Frame::read(args);
        let pp = f.varp.cast::<OptInt>();
        let (value, old_value) = (f.new_number(), f.old_number());
        if pp == p_ul.ptr() {
            p_ul.set(old_value);
            u_sync(true);
            p_ul.set(value);
        } else if pp == &raw mut (*f.buf).b_p_ul {
            (*f.buf).b_p_ul = old_value;
            u_sync(true);
            (*f.buf).b_p_ul = value;
        }
    }
    ptr::null()
}

/// 'updatecount': switching it on from zero is what opens the swap files.
pub unsafe extern "C" fn did_set_updatecount(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the buffer list is the editor's.
    unsafe {
        if p_uc.get() != 0 && Frame::read(args).old_number() == 0 {
            ml_open_files();
        }
    }
    ptr::null()
}

/// 'wildchar'/'wildcharm': a key that already means something on the command
/// line cannot also start completion.
pub unsafe extern "C" fn did_set_wildchar(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, whose `varp` is this numeric option's
    // own variable.
    let c = unsafe { *Frame::read(args).varp.cast::<OptInt>() };
    if c == Ctrl_C as OptInt
        || c == '\n' as OptInt
        || c == '\r' as OptInt
        || c == K_KENTER as OptInt
    {
        return e_invarg.ptr().cast::<c_char>();
    }
    ptr::null()
}

/// 'winblend': the window's own blend, clamped, plus the highlight groups
/// that depend on it.
pub unsafe extern "C" fn did_set_winblend(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the window it names is live.
    unsafe {
        let f = Frame::read(args);
        if f.new_number() != f.old_number() {
            let win = f.win;
            (*win).w_onebuf_opt.wo_winbl = (*win).w_onebuf_opt.wo_winbl.clamp(0, 100);
            (*win).w_hl_needs_update = 1;
            check_blending(win);
        }
    }
    ptr::null()
}

/// 'window': the scroll amount `CTRL-F` uses, capped at the screen.
pub unsafe extern "C" fn did_set_window(_args: *mut optset_T) -> *const c_char {
    if p_window.get() < 1 || p_window.get() >= Rows.get() as OptInt {
        p_window.set((Rows.get() - 1) as OptInt);
    }
    ptr::null()
}

/// 'winheight': grow the current window if it is now too short.
pub unsafe extern "C" fn did_set_winheight(_args: *mut optset_T) -> *const c_char {
    // SAFETY: `curwin` is live.
    unsafe {
        if firstwin.get() != lastwin.get() && ((*curwin.get()).w_height as OptInt) < p_wh.get() {
            win_setheight(p_wh.get() as c_int);
        }
    }
    ptr::null()
}

/// 'winwidth': widen the current window if it is now too narrow.
pub unsafe extern "C" fn did_set_winwidth(_args: *mut optset_T) -> *const c_char {
    // SAFETY: `curwin` is live.
    unsafe {
        if firstwin.get() != lastwin.get() && ((*curwin.get()).w_width as OptInt) < p_wiw.get() {
            win_setwidth(p_wiw.get() as c_int);
        }
    }
    ptr::null()
}

/// 'wrap': the two scroll offsets are exclusive — a wrapped window scrolls
/// within a line, an unwrapped one scrolls sideways.
pub unsafe extern "C" fn did_set_wrap(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, and the window it names is live.
    unsafe {
        let win = Frame::read(args).win;
        if (*win).w_onebuf_opt.wo_wrap != 0 {
            (*win).w_leftcol = 0 as colnr_T;
        } else {
            (*win).w_skipcol = 0 as colnr_T;
        }
    }
    ptr::null()
}

/// 'chistory'/'lhistory': how many quickfix or location lists to keep.
pub unsafe extern "C" fn did_set_xhistory(args: *mut optset_T) -> *const c_char {
    // SAFETY: the table's call frame, whose `varp` is this numeric option's
    // own variable, and the window it names is live.
    unsafe {
        let f = Frame::read(args);
        let arg = f.varp.cast::<OptInt>();
        if arg == p_chi.ptr() {
            qf_resize_stack(*arg as c_int);
        } else {
            ll_resize_stack(f.win, *arg as c_int);
        }
    }
    ptr::null()
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

    syn_recursive.set(syn_recursive.get() + 1);
    // SAFETY: the caller's buffer is live.
    unsafe {
        (*buf).b_flags |= BF_SYN_SET;
        apply_autocmds(
            EVENT_SYNTAX,
            (*buf).b_p_syn,
            (*buf).b_fname,
            value_changed || syn_recursive.get() == 1,
            buf,
        );
    }
    syn_recursive.set(syn_recursive.get() - 1);
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
    unsafe {
        let mut q = (*(*win).w_s).b_p_spl;
        // "cjk" is a modifier, not a language.
        if strncmp(q, c"cjk,".as_ptr(), 4) == 0 {
            q = q.add(4);
        }
        let mut p = q;
        while *p != NUL as c_char {
            let c = *p as c_int;
            if !((*p as u8).is_ascii_alphabetic() || ascii_isdigit(c) || c == '-' as c_int) {
                break;
            }
            p = p.add(1);
        }
        if p > q {
            vim_snprintf(
                fname.as_mut_ptr(),
                core::mem::size_of::<[c_char; 200]>(),
                c"spell/%.*s.*".as_ptr(),
                p.offset_from(q) as c_int,
                q,
            );
            source_runtime_vim_lua(fname.as_mut_ptr(), DIP_ALL as c_int);
        }
    }
}
