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

#![forbid(unsafe_code)]

use super::{
    SessionFile, SessionOpts, did_lcd, ses_arglist, ses_escape_fname, ses_fname, ses_get_fname,
    ses_put_fname,
};
use crate::buffer::{buf_is_help, buf_is_nofilename, buf_is_normal, buf_is_terminal, find_buf};
use crate::fold::put_folds;
use crate::mapping::makemap;
use crate::option::vars::ssop_flags;
use crate::option::{makefoldset, makeset};
use crate::options::{
    kOptSsopFlagCurdir, kOptSsopFlagCursor, kOptSsopFlagFolds, kOptSsopFlagLocaloptions,
    kOptSsopFlagOptions, kOptSsopFlagTerminal,
};
use crate::pos::MAXCOL;
use crate::types::ArgListId;
use crate::types::{OptionSetFlags, int64_t};
use crate::window::{DirOwner, has_own_dir, own_dir};
use crate::winlayer::graph::switch_to;
use crate::winlayer::{TabPage, Win};
use core::ffi::c_int;

/// Write the commands that restore `window`'s view.
///
/// `add_edit` asks for the `:edit` that loads the file; `:mksession` clears
/// it for the one window whose file it already edited, and `:mkview` clears
/// it when writing into 'viewdir'. `current_arg_idx` is the argument index
/// already in effect, or -1 when unknown.
///
/// The caller must have set 'scrolloff' to zero.
pub(crate) fn put_view(
    out: &mut SessionFile,
    window: Win,
    tabpage: TabPage,
    add_edit: bool,
    opts: SessionOpts,
    current_arg_idx: c_int,
) -> bool {
    // The cursor position is always restored for a session; for a view only
    // when 'viewoptions' asks.
    let mut do_cursor = opts.is_session() || opts.has(kOptSsopFlagCursor);

    // The argument list: the global one, or a local copy written out.
    if window.w_alist == ArgListId::GLOBAL {
        if !out.line(c"argglobal") {
            return false;
        }
    } else {
        // Full paths unless the session knows where it will be sourced
        // and no directory below it overrides that.
        let fullname = !opts.is_session()
            || !opts.has(kOptSsopFlagCurdir)
            || has_own_dir(DirOwner::TabPage(tabpage))
            || has_own_dir(DirOwner::Window(window));
        let names = crate::arglist::arg_names(window.w_alist);
        if !ses_arglist(out, c"arglocal", &names, fullname) {
            return false;
        }
    }

    // Restore the argument index, but only as part of a session and only
    // when it still points at something: arguments may have been deleted.
    let mut did_next = false;
    if window.w_arg_idx != current_arg_idx
        && window.w_arg_idx < crate::arglist::wargcount(window)
        && opts.is_session()
    {
        if !out.write(format_args!("{}argu\n", window.w_arg_idx as int64_t + 1)) {
            return false;
        }
        did_next = true;
    }

    // Edit the file, unless the `:next` above already did.
    if add_edit && (!did_next || window.w_arg_idx_invalid) {
        match put_edit(out, window, opts) {
            Some(keep_cursor) => do_cursor &= keep_cursor,
            None => return false,
        }
    }

    if window.w_alt_fnum != 0 && !put_alternate(out, window, opts) {
        return false;
    }

    // Local mappings and abbreviations.
    if opts.has(kOptSsopFlagOptions | kOptSsopFlagLocaloptions)
        && makemap(out, window.buffer_or_none()).is_err()
    {
        return false;
    }

    if !put_local_options(out, window, opts) {
        return false;
    }

    // Folds, when 'buftype' is empty and for help files.
    let buf = window.buffer();
    if opts.has(kOptSsopFlagFolds)
        && !buf.name.full().is_none()
        && (buf_is_normal(Some(buf)) || buf_is_help(Some(buf)))
        && put_folds(out, window).is_err()
    {
        return false;
    }

    // The cursor goes last: creating folds moves it.
    if do_cursor && !put_cursor(out, window) {
        return false;
    }

    // The window-local directory, unless this is a view that was not
    // asked for directories.
    if let Some(localdir) = own_dir(DirOwner::Window(window))
        && (opts.is_session() || opts.has(kOptSsopFlagCurdir))
    {
        if !out.puts(c"lcd ") || !ses_put_fname(out, localdir.as_cstr()) || !out.eol() {
            return false;
        }
        did_lcd.set(true);
    }
    true
}

/// Write the command that loads `window`'s file. Answers whether the cursor
/// position is still worth restoring afterwards -- an empty buffer has no
/// position -- or `None` when a write failed.
fn put_edit(out: &mut SessionFile, window: Win, opts: SessionOpts) -> Option<bool> {
    let buffer = window.buffer();
    let name = ses_get_fname(buffer, opts).unwrap_or_default();
    let fname_esc = ses_escape_fname(name.as_cstr());
    if buf_is_help(window.buffer_or_none()) {
        return put_help_edit(out, window).then_some(true);
    }
    if !buffer.name.full().is_none()
        && (!buf_is_nofilename(window.buffer_or_none()) || !buffer.terminal.is_null())
    {
        // Editing a file. This may have side effects -- a compressed or
        // network file -- and if a buffer for it already exists we
        // `:buffer` it instead, because `:edit` resets the folds of
        // other buffers.
        let ok = out.puts(c"if bufexists(fnamemodify(\"")
            && out.put_bytes(&fname_esc)
            && out.puts(c"\", \":p\")) | buffer ")
            && out.put_bytes(&fname_esc)
            && out.puts(c" | else | edit ")
            && out.put_bytes(&fname_esc)
            && out.puts(c" | endif\nif &buftype ==# 'terminal'\n  silent file ")
            && out.put_bytes(&fname_esc)
            && out.puts(c"\nendif\n");
        return ok.then_some(true);
    }
    // No file in this buffer: make it empty. It may still have a name that
    // is not a file name.
    let named = !buffer.name.full().is_none();
    let ok = out.line(c"enew")
        && (!named || (out.puts(c"file ") && out.put_bytes(&fname_esc) && out.eol()));
    ok.then_some(false)
}

/// A help window: create an empty `'buftype'=help` buffer and let `:help`
/// re-use both it and the window, which sets the options a help buffer needs
/// even when "options" is not in 'sessionoptions'.
fn put_help_edit(out: &mut SessionFile, window: Win) -> bool {
    let curtag = if 0 < window.w_tagstackidx && window.w_tagstackidx <= window.w_tagstacklen {
        window.w_tagstack[(window.w_tagstackidx - 1) as usize]
            .tagname
            .as_ref()
            .map_or(&b""[..], |tagname| tagname.as_bytes())
    } else {
        b""
    };
    out.line(c"enew | setl bt=help") && out.puts(c"help ") && out.put_bytes(curtag) && out.eol()
}

/// Write `balt` for the window's alternate file, when a session is being
/// written and the alternate buffer is one a restore could find again.
fn put_alternate(out: &mut SessionFile, window: Win, opts: SessionOpts) -> bool {
    let alt = find_buf(window.w_alt_fnum);
    let restorable = alt.is_some_and(|b| {
        !b.name.is_unnamed() && b.name.shown().is_some_and(|name| !name.is_empty()) && b.b_p_bl != 0
    });
    let wanted = opts.is_session()
        && restorable
        // Not a terminal, unless terminals are in 'sessionoptions'.
        && !(buf_is_terminal(alt) && ssop_flags.get() & kOptSsopFlagTerminal == 0);
    let Some(alt) = alt.filter(|_| wanted) else {
        return true;
    };
    out.puts(c"balt ") && ses_fname(out, alt, opts, true)
}

/// Write the window's local options or, when options are not wanted at all,
/// just the fold options that folds could not be restored without.
///
/// The writers read `curwin`/`curbuf`, so the window has to be made current
/// for the duration. Nothing between the two assignments runs Vimscript.
fn put_local_options(out: &mut SessionFile, window: Win, opts: SessionOpts) -> bool {
    // `curwin`/`curbuf` are restored before returning either way.
    let saved = switch_to(window);
    let f = if opts.has(kOptSsopFlagOptions | kOptSsopFlagLocaloptions) {
        // Store only the local values for a view, and for a session
        // whose 'sessionoptions' has no "options".
        let local_only = !opts.is_session() || !opts.has(kOptSsopFlagOptions);
        makeset(out, OptionSetFlags::LOCAL, c_int::from(local_only))
    } else if opts.has(kOptSsopFlagFolds) {
        makefoldset(out)
    } else {
        Ok(())
    };
    saved.restore();
    f.is_ok()
}

/// Restore the cursor line -- both in the file and relative to the top of
/// the window -- and then the column. `G` is deliberately not used: it would
/// change the jumplist.
fn put_cursor(out: &mut SessionFile, window: Win) -> bool {
    let height = window.w_view_height;
    let lnum = window.w_cursor.lnum;
    let placed = if height <= 0 {
        out.write(format_args!("let s:l = {lnum}\n"))
    } else {
        out.write(format_args!(
            "let s:l = {lnum} - (({} * winheight(0) + {}) / {height})\n",
            lnum - window.w_topline,
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
    if window.w_cursor.col == 0 {
        return out.line(c"normal! 0");
    }
    let width = window.w_width;
    if window.w_onebuf_opt.wo_wrap == 0 && window.w_leftcol > 0 && width > 0 {
        let virtcol = window.w_virtcol as int64_t;
        return out.write(format_args!(
            "let s:c = {} - (({} * winwidth(0) + {}) / {})\nif s:c > 0\n  exe 'normal! ' . s:c . '|zs' . {} . '|'\nelse\n",
            virtcol + 1,
            (window.w_virtcol - window.w_leftcol) as int64_t,
            (width / 2) as int64_t,
            width as int64_t,
            virtcol + 1,
        )) && put_view_curpos(out, window, "  ")
            && out.line(c"endif");
    }
    put_view_curpos(out, window, "")
}

/// The `normal!` command that puts the cursor on its column. `$` when the
/// cursor was at end-of-line ('curswant' is `MAXCOL`), otherwise the virtual
/// column, one-based.
fn put_view_curpos(out: &mut SessionFile, window: Win, spaces: &str) -> bool {
    if window.w_curswant == MAXCOL {
        out.write(format_args!("{spaces}normal! $\n"))
    } else {
        out.write(format_args!("{spaces}normal! 0{}|\n", window.w_virtcol + 1))
    }
}
