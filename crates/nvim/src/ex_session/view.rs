//! One window's worth of a session file: the argument list it uses, the
//! file it is editing, its local options and mappings, its folds, and where
//! its cursor sits.
//!
//! This is the whole of `:mkview` and the per-window part of `:mksession`.
//! The difference between the two is [`SessionOpts`]: a view is filtered by
//! 'viewoptions' and does not know what the working directory will be when
//! it is read back, so it writes full paths and only restores the cursor
//! when asked to.
//!
//! **The cursor block is arithmetic on purpose.** Rather than a line number
//! the file carries `let s:l = <lnum> - ((<offset> * winheight(0) + <h/2>) /
//! <h>)`, so the same line ends up the same distance down a window of a
//! different height; the column block does the same against `winwidth(0)`
//! when 'wrap' is off. Those digits are the format, not an implementation
//! detail.
//!
//! Original: `src/nvim/ex_session.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{
    SessionFile, SessionOpts, did_lcd, ses_arglist, ses_escape_fname, ses_fname, ses_get_fname,
    ses_put_fname,
};
use crate::buffer::{bt_help, bt_nofilename, bt_normal, bt_terminal, buflist_findnr};
use crate::fold::put_folds;
use crate::main::{curbuf, curwin, global_alist, ssop_flags};
use crate::mapping::makemap;
use crate::memory::xfree;
use crate::option::{makefoldset, makeset};
use crate::options::{
    kOptSsopFlagCurdir, kOptSsopFlagCursor, kOptSsopFlagFolds, kOptSsopFlagLocaloptions,
    kOptSsopFlagOptions, kOptSsopFlagTerminal,
};
use crate::pos::MAXCOL;
use crate::types::{FAIL, NUL, OK, OptionSetFlags, int64_t, tabpage_T, win_T};
use ::libc::fprintf;
use core::ffi::{c_char, c_int, c_void};

/// Write the commands that restore `wp`'s view.
///
/// `add_edit` asks for the `:edit` that loads the file; `:mksession` clears
/// it for the one window whose file it already edited, and `:mkview` clears
/// it when writing into 'viewdir'. `current_arg_idx` is the argument index
/// already in effect, or -1 when unknown.
///
/// The caller must have set 'scrolloff' to zero.
///
/// # Safety
/// `wp` and `tp` are live; `wp` belongs to `tp`. Main thread: this makes
/// `wp` current for the duration of the option writers.
pub(crate) unsafe fn put_view(
    out: SessionFile,
    wp: *mut win_T,
    tp: *mut tabpage_T,
    add_edit: bool,
    opts: SessionOpts,
    current_arg_idx: c_int,
) -> bool {
    // The cursor position is always restored for a session; for a view only
    // when 'viewoptions' asks.
    let mut do_cursor = opts.is_session() || opts.has(kOptSsopFlagCursor);

    // SAFETY: caller contract; a window always has a buffer and an argument
    // list.
    unsafe {
        // The argument list: the global one, or a local copy written out.
        if (*wp).w_alist == global_alist.ptr() {
            if !out.line(c"argglobal") {
                return false;
            }
        } else {
            // Full paths unless the session knows where it will be sourced
            // and no directory below it overrides that.
            let fullname = !opts.is_session()
                || !opts.has(kOptSsopFlagCurdir)
                || !(*tp).tp_localdir.is_null()
                || !(*wp).w_localdir.is_null();
            if !ses_arglist(out, c"arglocal", &raw mut (*(*wp).w_alist).al_ga, fullname) {
                return false;
            }
        }

        // Restore the argument index, but only as part of a session and only
        // when it still points at something: arguments may have been deleted.
        let mut did_next = false;
        if (*wp).w_arg_idx != current_arg_idx
            && (*wp).w_arg_idx < (*(*wp).w_alist).al_ga.ga_len
            && opts.is_session()
        {
            if !out.write(format_args!("{}argu\n", (*wp).w_arg_idx as int64_t + 1)) {
                return false;
            }
            did_next = true;
        }

        // Edit the file, unless the `:next` above already did.
        if add_edit && (!did_next || (*wp).w_arg_idx_invalid != 0) {
            match put_edit(out, wp, opts) {
                Some(keep_cursor) => do_cursor &= keep_cursor,
                None => return false,
            }
        }

        if (*wp).w_alt_fnum != 0 && !put_alternate(out, wp, opts) {
            return false;
        }

        // Local mappings and abbreviations.
        if opts.has(kOptSsopFlagOptions | kOptSsopFlagLocaloptions)
            && makemap(out.raw(), (*wp).w_buffer) == FAIL
        {
            return false;
        }

        if !put_local_options(out, wp, opts) {
            return false;
        }

        // Folds, when 'buftype' is empty and for help files.
        if opts.has(kOptSsopFlagFolds)
            && !(*(*wp).w_buffer).b_ffname.is_null()
            && (bt_normal((*wp).w_buffer) || bt_help((*wp).w_buffer))
            && put_folds(out.raw(), wp) == FAIL
        {
            return false;
        }

        // The cursor goes last: creating folds moves it.
        if do_cursor && !put_cursor(out, wp) {
            return false;
        }

        // The window-local directory, unless this is a view that was not
        // asked for directories.
        if !(*wp).w_localdir.is_null() && (opts.is_session() || opts.has(kOptSsopFlagCurdir)) {
            if !out.puts(c"lcd ") || !ses_put_fname(out, (*wp).w_localdir) || !out.eol() {
                return false;
            }
            did_lcd.set(true);
        }
    }
    true
}

/// Write the command that loads `wp`'s file. Answers whether the cursor
/// position is still worth restoring afterwards -- an empty buffer has no
/// position -- or `None` when a write failed.
///
/// # Safety
/// `wp` is live.
unsafe fn put_edit(out: SessionFile, wp: *mut win_T, opts: SessionOpts) -> Option<bool> {
    // SAFETY: caller contract; `fname_esc` is owned and freed on every path.
    unsafe {
        let buf = (*wp).w_buffer;
        let fname_esc = ses_escape_fname(ses_get_fname(buf, opts));
        let outcome = if bt_help(buf) {
            put_help_edit(out, wp).then_some(true)
        } else if !(*buf).b_ffname.is_null() && (!bt_nofilename(buf) || !(*buf).terminal.is_null())
        {
            // Editing a file. This may have side effects -- a compressed or
            // network file -- and if a buffer for it already exists we
            // `:buffer` it instead, because `:edit` resets the folds of
            // other buffers.
            let ok = fprintf(
                out.raw(),
                c"if bufexists(fnamemodify(\"%s\", \":p\")) | buffer %s | else | edit %s | endif\nif &buftype ==# 'terminal'\n  silent file %s\nendif\n"
                    .as_ptr(),
                fname_esc,
                fname_esc,
                fname_esc,
                fname_esc,
            ) >= 0;
            ok.then_some(true)
        } else {
            // No file in this buffer: make it empty. It may still have a
            // name that is not a file name.
            let named = !(*buf).b_ffname.is_null();
            let ok = out.line(c"enew")
                && (!named || (out.puts(c"file ") && out.bytes(fname_esc) && out.eol()));
            ok.then_some(false)
        };
        xfree(fname_esc.cast::<c_void>());
        outcome
    }
}

/// A help window: create an empty `'buftype'=help` buffer and let `:help`
/// re-use both it and the window, which sets the options a help buffer needs
/// even when "options" is not in 'sessionoptions'.
///
/// # Safety
/// `wp` is live.
unsafe fn put_help_edit(out: SessionFile, wp: *mut win_T) -> bool {
    // SAFETY: caller contract; a tag stack entry's name is NUL-terminated.
    unsafe {
        let curtag = if 0 < (*wp).w_tagstackidx && (*wp).w_tagstackidx <= (*wp).w_tagstacklen {
            (*wp).w_tagstack[((*wp).w_tagstackidx - 1) as usize].tagname
        } else {
            c"".as_ptr().cast_mut()
        };
        out.line(c"enew | setl bt=help") && out.puts(c"help ") && out.bytes(curtag) && out.eol()
    }
}

/// Write `balt` for the window's alternate file, when a session is being
/// written and the alternate buffer is one a restore could find again.
///
/// # Safety
/// `wp` is live.
unsafe fn put_alternate(out: SessionFile, wp: *mut win_T, opts: SessionOpts) -> bool {
    // SAFETY: caller contract; `buflist_findnr` answers a live buffer or
    // null.
    unsafe {
        let alt = buflist_findnr((*wp).w_alt_fnum);
        let wanted = opts.is_session()
            && !alt.is_null()
            && !(*alt).b_fname.is_null()
            && *(*alt).b_fname != NUL as c_char
            && (*alt).b_p_bl != 0
            // Not a terminal, unless terminals are in 'sessionoptions'.
            && !(bt_terminal(alt) && ssop_flags.get() & kOptSsopFlagTerminal == 0);
        !wanted || (out.puts(c"balt ") && ses_fname(out, alt, opts, true))
    }
}

/// Write the window's local options or, when options are not wanted at all,
/// just the fold options that folds could not be restored without.
///
/// The writers read `curwin`/`curbuf`, so the window has to be made current
/// for the duration. Nothing between the two assignments runs Vimscript.
///
/// # Safety
/// `wp` is live.
unsafe fn put_local_options(out: SessionFile, wp: *mut win_T, opts: SessionOpts) -> bool {
    // SAFETY: caller contract; `curwin`/`curbuf` are restored before
    // returning either way.
    unsafe {
        let save_curwin = curwin.get();
        curwin.set(wp);
        curbuf.set((*curwin.get()).w_buffer);
        let f = if opts.has(kOptSsopFlagOptions | kOptSsopFlagLocaloptions) {
            // Store only the local values for a view, and for a session
            // whose 'sessionoptions' has no "options".
            let local_only = !opts.is_session() || !opts.has(kOptSsopFlagOptions);
            makeset(out.raw(), OptionSetFlags::LOCAL, local_only as c_int)
        } else if opts.has(kOptSsopFlagFolds) {
            makefoldset(out.raw())
        } else {
            OK
        };
        curwin.set(save_curwin);
        curbuf.set((*curwin.get()).w_buffer);
        f != FAIL
    }
}

/// Restore the cursor line -- both in the file and relative to the top of
/// the window -- and then the column. `G` is deliberately not used: it would
/// change the jumplist.
///
/// # Safety
/// `wp` is live.
unsafe fn put_cursor(out: SessionFile, wp: *mut win_T) -> bool {
    // SAFETY: caller contract.
    unsafe {
        let height = (*wp).w_view_height;
        let lnum = (*wp).w_cursor.lnum;
        let placed = if height <= 0 {
            out.write(format_args!("let s:l = {lnum}\n"))
        } else {
            out.write(format_args!(
                "let s:l = {lnum} - (({} * winheight(0) + {}) / {height})\n",
                lnum - (*wp).w_topline,
                height / 2,
            ))
        };
        if !placed
            || !out.write(format_args!(
                "if s:l < 1 | let s:l = 1 | endif\nkeepjumps exe s:l\nnormal! zt\nkeepjumps {lnum}\n"
            ))
        {
            return false;
        }

        // The column, and the left offset when not wrapping.
        if (*wp).w_cursor.col == 0 {
            return out.line(c"normal! 0");
        }
        let width = (*wp).w_width;
        if (*wp).w_onebuf_opt.wo_wrap == 0 && (*wp).w_leftcol > 0 && width > 0 {
            let virtcol = (*wp).w_virtcol as int64_t;
            return out.write(format_args!(
                "let s:c = {} - (({} * winwidth(0) + {}) / {})\nif s:c > 0\n  exe 'normal! ' . s:c . '|zs' . {} . '|'\nelse\n",
                virtcol + 1,
                ((*wp).w_virtcol - (*wp).w_leftcol) as int64_t,
                (width / 2) as int64_t,
                width as int64_t,
                virtcol + 1,
            )) && put_view_curpos(out, wp, "  ")
                && out.line(c"endif");
        }
        put_view_curpos(out, wp, "")
    }
}

/// The `normal!` command that puts the cursor on its column. `$` when the
/// cursor was at end-of-line ('curswant' is `MAXCOL`), otherwise the virtual
/// column, one-based.
///
/// # Safety
/// `wp` is live.
unsafe fn put_view_curpos(out: SessionFile, wp: *const win_T, spaces: &str) -> bool {
    // SAFETY: caller contract.
    unsafe {
        if (*wp).w_curswant == MAXCOL {
            out.write(format_args!("{spaces}normal! $\n"))
        } else {
            out.write(format_args!("{spaces}normal! 0{}|\n", (*wp).w_virtcol + 1))
        }
    }
}
