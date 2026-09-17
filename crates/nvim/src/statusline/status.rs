//! The plain status line, and the buffer name behind it.
//!
//! [`win_redr_status`] is the entry point the drawing layer calls once per
//! window: it hands `'statusline'` to [`win_redr_custom`] and then draws the
//! one cell below the vertical separator itself, which is the only part of a
//! status line the format language has nothing to say about.
//! [`stl_connected`] answers whether a window's status line runs on into the
//! window right of it, which decides whether that cell is a fill character or
//! a separator. [`get_trans_bufname`] puts a buffer's displayable name in
//! `NameBuff`, for the tab line and for `:ls`.
//!
//! Original: `src/nvim/statusline.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::winlayer::Buf;
use core::ffi::c_int;

use super::*;
use crate::buffer::buf_spname;
use crate::charset::trans_characters;
use crate::cmdexpand::state::wild_menu_showing;
use crate::drawscreen::state::redraw_cmdline;
use crate::global_cell::GlobalCell;
use crate::grid::default_gridview;
use crate::highlight_group::HLF_C;
use crate::memory::xstrlcpy;
use crate::optionstr::LocalOptStr;
use crate::os::env::home_replace;
use crate::types::MAXPATHL;
use crate::types::ui::kUIWildmenu;
use crate::ui::ui_has;

/// Redraw the status line of window `window`.
pub fn win_redr_status(window: Win) {
    let mut win = window;
    let is_stl_global = stl_is_global();

    static BUSY: GlobalCell<bool> = GlobalCell::new(false);
    // Reached recursively when 'statusline' (indirectly) invokes
    // ":redrawstatus"; ignore the call then. Also ignore it while the
    // wildmenu is showing, which may be drawn over the status line.
    if BUSY.get() || (wild_menu_showing.get() != 0 && !ui_has(kUIWildmenu)) {
        return;
    }
    BUSY.set(true);

    win.w_redr_status = false;
    if win.w_status_height == 0 && !(is_stl_global && win.is_current()) {
        // No status line: either 'laststatus' is 3 or this is the last
        // window, so the command line is what has to be refreshed.
        redraw_cmdline.set(true);
    } else if !is_redrawing() {
        // Not now -- the popup menu may be drawn over it.
        win.w_redr_status = true;
    } else if !win.w_onebuf_opt.wo_stl.bytes().is_empty()
        || !win.w_floating
        || (is_stl_global && win.is_current())
    {
        redraw_custom_statusline(window);
    }

    // May need to draw the character below the vertical separator.
    if win.w_vsep_width != 0 && win.w_status_height != 0 && is_redrawing() {
        let mut group = HLF_C;
        let fillchar = if stl_connected(window) {
            let (g, fillchar) = fillchar_status_of(win);
            group = g;
            fillchar
        } else {
            win.w_p_fcs_chars.vert
        };
        let attr = win_hl(win, group as c_int);
        view_line_start(default_gridview(), win.w_winrow + win.w_height);
        paint_schar(win.w_wincol + win.w_width, fillchar, attr);
        paint_flush();
    }
    BUSY.set(false);
}

/// Whether the status line of `window` is connected to the status line of the
/// window right of it -- as opposed to meeting a vertical separator there.
///
/// Only meaningful when `window.w_vsep_width != 0`.
pub fn stl_connected(window: Win) -> bool {
    let mut fr = window.frame();
    while let Some(parent) = fr.parent() {
        if c_int::from(parent.fr_layout) == FR_COL {
            // A row below this one ends the run.
            if fr.next().is_some() {
                break;
            }
        } else if fr.next().is_some() {
            // Another window beside this one, at the same height.
            return true;
        }
        fr = parent;
    }
    false
}

/// Put the displayable name of `buffer` in `name`: its special name if it has
/// one, else its file name with `$HOME` folded back to `~`, with the
/// unprintable characters replaced by their display forms.
///
/// Upstream fills the shared `NameBuff` and leaves the readers to find it
/// there, which is why the fill and each read had to be separate borrows.
pub(crate) fn get_trans_bufname(buffer: Buf, name: &mut [c_char; MAXPATHL as usize]) {
    let spname = buf_spname(buffer);
    let (out, room) = (name.as_mut_ptr(), MAXPATHL as size_t);
    // SAFETY: the caller's promise, and `name` is `MAXPATHL` bytes, which
    // each of the three writes below is told.
    unsafe {
        if spname.is_null() {
            home_replace(Some(buffer), buffer.b_fname, out, room, true);
        } else {
            xstrlcpy(out, spname, room);
        }
        trans_characters(out, MAXPATHL);
    }
}
