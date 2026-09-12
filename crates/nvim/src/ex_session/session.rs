//! `:mksession`: the whole editor as a script.
//!
//! [`makeopens`] writes, in this order: a preamble that sets
//! `v:this_session` and fires `SessionLoadPre`, the session globals, a `:cd`,
//! the buffer list as `badd` lines, the global argument list, the window and
//! tab page layout, one [`put_view`](super::view::put_view) per window, and
//! an epilogue that undoes the temporary option changes the preamble made.
//!
//! Two things in there are subtle enough to be worth naming.
//!
//! The layout is written *before* the views, and the buffer list before
//! both, because a restore that creates windows lazily ends up with the
//! buffers in a different order -- an `:edit` or `:tabedit` partway through
//! would reorder them. [`ses_win_rec`] walks the frame tree and emits
//! `split`/`vsplit` with `wincmd`s between, leaving the last window of each
//! frame current.
//!
//! Window sizes are written as arithmetic against `&lines` and `&columns`
//! ([`ses_winsizes`]) so that a session restored into a differently-sized
//! terminal keeps its proportions -- and only when no window was skipped,
//! since a missing window makes the remaining sizes meaningless.
//!
//! Original: `src/nvim/ex_session.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::flag::{FR_COL, FR_LEAF};
use super::view::put_view;
use super::{
    SessionFile, SessionOpts, did_lcd, ses_arglist, ses_do_win, ses_escape_fname, ses_fname,
    ses_put_fname,
};
use crate::arglist::global_arglist;
use crate::buffer::{buf_is_help, buf_is_nofilename, buf_is_terminal};
use crate::eval::typval::NumBuf;
use crate::eval::var_flavour;
use crate::eval::vars::get_globvar_dict;
use crate::memory::xfree;
use crate::option::vars::{p_shm, p_stal, p_wh, p_wiw};
use crate::options::{
    kOptSsopFlagBuffers, kOptSsopFlagCurdir, kOptSsopFlagGlobals, kOptSsopFlagHelp,
    kOptSsopFlagOptions, kOptSsopFlagResize, kOptSsopFlagSesdir, kOptSsopFlagTabpages,
    kOptSsopFlagTerminal, kOptSsopFlagWinsize,
};
use crate::os::env::home_replace_save;
use crate::os::state::globaldir;
use crate::strings::vim_strsave_escaped;
use crate::types::{
    NUL, TypVal, VAR_FLAVOUR_SESSION, VAR_FLOAT, VAR_NUMBER, VAR_STRING, VarType, Window, int64_t,
};
use crate::ui::state::{Columns, Rows};
use crate::window::tab_index;
use crate::winlayer::graph::firstwin;
use crate::winlayer::{
    FrameRef, TabPage, Win, WinId, buffers, current_topframe, first_tab, tabs, windows_in_tab,
};
use ::libc::fprintf;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// Write everything that restores the current editor state.
///
/// `dirnow` is the working directory as it was before [`super::ex_mkrc`]
/// changed into the session file's own directory.
///
/// # Safety
/// Main thread; the buffer, window and tab page lists are live. `dirnow` is
/// NUL-terminated.
pub(crate) unsafe fn makeopens(out: SessionFile, dirnow: *mut c_char) -> bool {
    let opts = SessionOpts::Session;
    // "buffers" means save every buffer, not only the ones in a window.
    let only_save_windows = !opts.has(kOptSsopFlagBuffers);

    if !out.line(c"let v:this_session=expand(\"<sfile>:p\")")
        || !out.line(c"doautoall SessionLoadPre")
    {
        return false;
    }
    // SAFETY: the global variable dict is live on the main thread.
    if opts.has(kOptSsopFlagGlobals) && !store_session_globals(out) {
        return false;
    }

    // Close all windows and tab pages but one.
    if !out.line(c"silent only") {
        return false;
    }
    if opts.has(kOptSsopFlagTabpages) && !out.line(c"silent tabonly") {
        return false;
    }

    // Then a :cd to the session directory or to the current one.
    // SAFETY: `dirnow` and `globaldir` are NUL-terminated.
    if !unsafe { put_cd(out, dirnow) } {
        return false;
    }

    if !out.line(
        c"if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''\n  let s:wipebuf = bufnr('%')\nendif",
    ) {
        return false;
    }

    // 'shortmess' is set for the load and restored at the end; save the old
    // value here unless the file carries options anyway.
    if !opts.has(kOptSsopFlagOptions) && !out.line(c"let s:shortmess_save = &shortmess") {
        return false;
    }
    if !out.line(c"set shortmess+=aoO") {
        return false;
    }

    // Put every buffer into the buffer list, very early, so that loading the
    // session cannot disturb their order.
    // SAFETY: the buffer list is live.
    if !put_buffer_list(out, only_save_windows) {
        return false;
    }

    // SAFETY: the global argument list is live for the call.
    if !unsafe {
        ses_arglist(
            out,
            c"argglobal",
            &(*global_arglist()).al_ga,
            !opts.has(kOptSsopFlagCurdir),
        )
    } {
        return false;
    }

    if opts.has(kOptSsopFlagResize)
        && !out.write(format_args!(
            "set lines={} columns={}\n",
            Rows.get() as int64_t,
            Columns.get() as int64_t
        ))
    {
        return false;
    }

    // With two or more tab pages and 'showtabline' at 1, the tabline appears
    // when the next tab is created, which resizes the first tab's windows.
    let restore_stal = p_stal.get() == 1 && first_tab().is_some_and(|tp| tp.next().is_some());
    if restore_stal && !out.line(c"set stal=2") {
        return false;
    }

    // Whether the tab pass pinned 'winminheight'/'winminwidth', which the
    // epilogue then has to put back.
    let mut restore_height_width = false;
    // SAFETY: the tab page and window lists are live; `put_tabs` runs no
    // Vimscript, so nothing can change them under it.
    if !put_tabs(out, &mut restore_height_width) {
        return false;
    }

    if opts.has(kOptSsopFlagTabpages) {
        let index = tab_index(TabPage::current());
        if !out.write(format_args!("tabnext {index}\n")) {
            return false;
        }
    }
    if restore_stal && !out.line(c"set stal=1") {
        return false;
    }

    // Wipe out the empty unnamed buffer we started in.
    if !out.line(
        c"if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'\n  silent exe 'bwipe ' . s:wipebuf\nendif\nunlet! s:wipebuf",
    ) {
        return false;
    }

    // Re-apply 'winheight' and 'winwidth', which the layout pass set to 1.
    if !out.write(format_args!(
        "set winheight={} winwidth={}\n",
        p_wh.get() as int64_t,
        p_wiw.get() as int64_t
    )) {
        return false;
    }

    // Restore 'shortmess'.
    if opts.has(kOptSsopFlagOptions) {
        // SAFETY: 'shortmess' is a NUL-terminated option string, and its
        // bytes go out verbatim.
        if !out.puts(c"set shortmess=") || !unsafe { out.bytes(p_shm.get()) } || !out.eol() {
            return false;
        }
    } else if !out.line(c"let &shortmess = s:shortmess_save") {
        return false;
    }

    if restore_height_width
        && (!out.line(c"let &winminheight = s:save_winminheight")
            || !out.line(c"let &winminwidth = s:save_winminwidth"))
    {
        return false;
    }

    // Lastly, source the companion x.vim if there is one.
    out.line(
        c"let s:sx = expand(\"<sfile>:p:r\").\"x.vim\"\nif filereadable(s:sx)\n  exe \"source \" . fnameescape(s:sx)\nendif",
    )
}

/// The `:cd` that fixes what relative file names below are relative to.
///
/// # Safety
/// `dirnow` is NUL-terminated.
unsafe fn put_cd(out: SessionFile, dirnow: *mut c_char) -> bool {
    let opts = SessionOpts::Session;
    if opts.has(kOptSsopFlagSesdir) {
        return out.line(c"exe \"cd \" . escape(expand(\"<sfile>:p:h\"), ' ')");
    }
    if !opts.has(kOptSsopFlagCurdir) {
        return true;
    }
    // SAFETY: caller contract; both copies are owned and freed here.
    let dir = if globaldir.get().is_null() {
        dirnow
    } else {
        globaldir.get()
    };
    let sname = unsafe { home_replace_save(None, dir) };
    let fname_esc = unsafe { ses_escape_fname(sname) };
    let ok = out.puts(c"cd ") && unsafe { out.bytes(fname_esc) } && out.eol();
    unsafe { xfree(fname_esc.cast::<c_void>()) };
    unsafe { xfree(sname.cast::<c_void>()) };
    ok
}

/// One `badd +<lnum> <name>` per buffer worth restoring.
fn put_buffer_list(out: SessionFile, only_save_windows: bool) -> bool {
    let opts = SessionOpts::Session;
    // SAFETY: caller contract; each buffer's window info is its own kvec.
    for buf in buffers() {
        let wanted = !(only_save_windows && buf.b_nwindows == 0)
            && !(buf.b_help && !opts.has(kOptSsopFlagHelp))
            && !(buf_is_terminal(Some(buf)) && !opts.has(kOptSsopFlagTerminal))
            && !buf.b_fname.is_null()
            && buf.b_p_bl != 0;
        if wanted {
            let lnum = if buf.b_wininfo.size == 0 {
                1 as int64_t
            } else {
                unsafe { (**buf.b_wininfo.items).wi_mark.mark.lnum as int64_t }
            };
            if !out.write(format_args!("badd +{lnum} ")) || !ses_fname(out, buf, opts, true) {
                return false;
            }
        }
    }
    true
}

/// The per-tab-page pass: the window layout, the sizes, and one
/// [`put_view`] per window. When "tabpages" is not in 'sessionoptions' this
/// does the current tab page only.
fn put_tabs(out: SessionFile, restore_height_width: &mut bool) -> bool {
    let opts = SessionOpts::Session;
    let with_tabs = opts.has(kOptSsopFlagTabpages);

    // Populate the tab pages first, as the window layout is populated first
    // below, so that local options set later are not copied into new tabs.
    // `bufhidden=wipe` drops the placeholder buffers once they are unneeded
    // (Vim patch 8.1.0829).
    if with_tabs {
        for tp in tabs() {
            if tp.next().is_some() && !out.line(c"tabnew +setlocal\\ bufhidden=wipe") {
                return false;
            }
        }
        if first_tab().is_some_and(|tp| tp.next().is_some()) && !out.line(c"tabrewind") {
            return false;
        }
    }

    let mut restore_size = true;
    let mut cur_arg_idx = 0;
    let mut next_arg_idx = 0;
    // The one window whose file `makeopens` already `:edit`ed, so that
    // `put_view` need not do it again.
    let mut edited_win = ptr::null_mut::<Window>();

    // SAFETY: caller contract; nothing here runs Vimscript, so the lists
    // cannot change under the walk.
    let mut next = first_tab();
    while let Some(mut tab) = next {
        let mut need_tabnext = false;
        let (tab_firstwin, tab_topframe) = if with_tabs {
            need_tabnext = Some(tab) != first_tab();
            if tab.is_current() {
                (firstwin.get(), current_topframe())
            } else {
                (tab.tp_firstwin, tab.topframe())
            }
        } else {
            tab = TabPage::current();
            (firstwin.get(), current_topframe())
        };

        // Before creating the layout, try loading one file: if that is
        // aborted we do not end up with a pile of useless windows. This
        // may have side effects (a compressed or network file).
        for window in windows_in_tab(tab) {
            let buffer = window.buffer();
            if ses_do_win(window)
                && !buffer.b_ffname.is_null()
                && !buf_is_help(Some(buffer))
                && !buf_is_nofilename(Some(buffer))
            {
                if need_tabnext && !out.line(c"tabnext") {
                    return false;
                }
                need_tabnext = false;
                if !out.puts(c"edit ") || !ses_fname(out, buffer, opts, true) {
                    return false;
                }
                if !window.w_arg_idx_invalid {
                    edited_win = window.raw();
                }
                break;
            }
        }
        // No file got edited: create an empty tab page.
        if need_tabnext && !out.line(c"tabnext") {
            return false;
        }

        if tab_topframe.fr_layout != FR_LEAF
            && (!out.line(c"let s:save_splitbelow = &splitbelow")
                || !out.line(c"let s:save_splitright = &splitright")
                || !out.line(c"set splitbelow splitright")
                || !ses_win_rec(out, tab_topframe)
                || !out.line(c"let &splitbelow = s:save_splitbelow")
                || !out.line(c"let &splitright = s:save_splitright"))
        {
            return false;
        }

        // Can the window sizes be restored -- that is, was no window
        // omitted? And which window number is the current one?
        let mut nr = 0;
        let mut cnr = 1;
        for window in windows_in_tab(tab) {
            if ses_do_win(window) {
                nr += 1;
            } else if !window.w_floating {
                restore_size = false;
            }
            if Win::current_raw() == window.raw() {
                cnr = nr;
            }
        }

        if tab_firstwin
            .and_then(WinId::get)
            .is_some_and(|w| w.next().is_some())
        {
            // Go to the first window, then pin 'winheight'/'winwidth' to
            // 1 so that moving between windows does not resize them --
            // before restoring the views, so that the topline and the
            // cursor can be set. Done again at the end.
            // 'winminheight'/'winminwidth' go to 0 as well, or a user
            // 'winheight' would make this an error.
            if !out.line(c"wincmd t") {
                return false;
            }
            if !*restore_height_width
                && (!out.line(c"let s:save_winminheight = &winminheight")
                    || !out.line(c"let s:save_winminwidth = &winminwidth"))
            {
                return false;
            }
            if !out.line(c"set winminheight=0\nset winheight=1\nset winminwidth=0\nset winwidth=1")
            {
                return false;
            }
            *restore_height_width = true;
        }
        if nr > 1 && !ses_winsizes(out, restore_size, tab) {
            return false;
        }

        // The tab-local working directory goes before the windows, so a
        // window-local one can override it.
        if opts.has(kOptSsopFlagCurdir) && !tab.tp_localdir.is_null() {
            if !out.puts(c"tcd ") || !unsafe { ses_put_fname(out, tab.tp_localdir) } || !out.eol() {
                return false;
            }
            did_lcd.set(true);
        }

        // Each window's view.
        for window in windows_in_tab(tab) {
            if ses_do_win(window) {
                if !put_view(
                    out,
                    window,
                    tab,
                    window.raw() != edited_win,
                    opts,
                    cur_arg_idx,
                ) {
                    return false;
                }
                if nr > 1 && !out.line(c"wincmd w") {
                    return false;
                }
                next_arg_idx = window.w_arg_idx;
            }
        }
        // The argument index is zero in the first tab page and has to be
        // set per window; for later tab pages it is the window the
        // `:tabedit` happened in.
        cur_arg_idx = next_arg_idx;

        // Put the cursor back in the current window when it is not the
        // first.
        if cnr > 1 && !out.write(format_args!("{cnr}wincmd w\n")) {
            return false;
        }
        // And restore the sizes again: jumping around gives the current
        // window a minimum size the others may not have.
        if nr > 1 && !ses_winsizes(out, restore_size, tab) {
            return false;
        }

        if !with_tabs {
            break;
        }
        next = tab.next();
    }
    true
}

/// The window sizes for one tab page, as arithmetic against `&lines` and
/// `&columns`. When a window was omitted the numbers would not add up, so
/// the sizes are just equalised instead.
fn ses_winsizes(out: SessionFile, restore_size: bool, tab: TabPage) -> bool {
    if !restore_size || !SessionOpts::Session.has(kOptSsopFlagWinsize) {
        return out.line(c"wincmd =");
    }
    // `topframe` is the current tab's frame tree.
    let top = current_topframe();
    let mut n = 0;
    for window in windows_in_tab(tab) {
        if ses_do_win(window) {
            n += 1;
            // Restore the height when the window is not full height.
            if window.w_height + window.w_hsep_height + window.w_status_height < top.fr_height
                && !out.write(format_args!(
                    "exe '{n}resize ' . ((&lines * {} + {}) / {})\n",
                    window.w_height as int64_t,
                    Rows.get() as int64_t / 2,
                    Rows.get() as int64_t,
                ))
            {
                return false;
            }
            // And the width when it is not full width.
            if window.w_width < Columns.get()
                && !out.write(format_args!(
                    "exe 'vert {n}resize ' . ((&columns * {} + {}) / {})\n",
                    window.w_width as int64_t,
                    Columns.get() as int64_t / 2,
                    Columns.get() as int64_t,
                ))
            {
                return false;
            }
        }
    }
    true
}

/// Write the splits that recreate the windows of frame `fr`, recursively.
/// Afterwards the last window in the frame is the current one.
fn ses_win_rec(out: SessionFile, fr: FrameRef) -> bool {
    if fr.fr_layout == FR_LEAF {
        return true;
    }
    let column = fr.fr_layout == FR_COL;

    // Find the first frame that is not skipped, then create a window for
    // each one after it -- the first window is already there.
    let mut count = 0;
    let mut frc = ses_skipframe(fr.child());
    while let Some(current) = frc {
        frc = ses_skipframe(current.next());
        if frc.is_none() {
            break;
        }
        // Make the window as big as possible, for room to split.
        if !out.puts(c"wincmd _ | wincmd |")
            || !out.eol()
            || !out.line(if column { c"split" } else { c"vsplit" })
        {
            return false;
        }
        count += 1;
    }

    // Go back to the first window.
    if count > 0 {
        let direction = if column { 'k' } else { 'h' };
        if !out.write(format_args!("{count}wincmd {direction}\n")) {
            return false;
        }
    }

    // Then recurse into each window of this column or row.
    frc = ses_skipframe(fr.child());
    while let Some(current) = frc {
        ses_win_rec(out, current);
        frc = ses_skipframe(current.next());
        if frc.is_some() && !out.line(c"wincmd w") {
            return false;
        }
    }
    true
}

/// The first frame at or after `fr` holding a window worth saving.
fn ses_skipframe(fr: Option<FrameRef>) -> Option<FrameRef> {
    let mut frc = fr;
    while let Some(current) = frc.filter(|frc| !ses_do_frame(*frc)) {
        frc = current.next();
    }
    frc
}

/// Whether frame `fr` holds a window worth saving anywhere below it.
fn ses_do_frame(fr: FrameRef) -> bool {
    match fr.win() {
        Some(win) => ses_do_win(win),
        None => fr.children().any(ses_do_frame),
    }
}

/// Write the `g:` variables 'sessionoptions' calls sessionable: the Number,
/// String and Float ones whose name says they belong in a session (an
/// uppercase first letter and a lowercase one after it).
fn store_session_globals(out: SessionFile) -> bool {
    // SAFETY: caller contract -- the global variable dictionary is live, so
    // the walk of it is ordinary code.
    let globals = unsafe { &mut *get_globvar_dict() };
    for item in globals.items_mut() {
        let key = item.di_key.as_ptr();
        let kind = item.di_tv.v_type();
        // SAFETY: the key is the item's own, NUL-terminated.
        let sessionable = unsafe { var_flavour(key.cast_mut()) } == VAR_FLAVOUR_SESSION;
        if (kind == VAR_NUMBER || kind == VAR_STRING) && sessionable {
            // SAFETY: the session file, and the item's own key and value.
            if !unsafe { put_session_global(out, key, kind, &mut item.di_tv) } {
                return false;
            }
        } else if kind == VAR_FLOAT && sessionable {
            let f = item.di_tv.float_or_zero();
            let sign = if f < 0.0 { b'-' } else { b' ' } as c_int;
            if unsafe {
                fprintf(
                    out.raw(),
                    c"let %s = %c%f".as_ptr(),
                    key,
                    sign,
                    if f < 0.0 { -f } else { f },
                )
            } < 0
                || !out.eol()
            {
                return false;
            }
        }
    }
    true
}

/// `let <name> = <value>` for one Number or String global. The value is
/// escaped so that the script reads it back unchanged: backslash, quote, and
/// a literal LF or CR turned into `\n`/`\r`.
///
/// # Safety
/// `key` is NUL-terminated and `tv` a Number or String.
unsafe fn put_session_global(
    out: SessionFile,
    key: *const c_char,
    kind: VarType,
    tv: &mut TypVal,
) -> bool {
    let mut numbuf = NumBuf::new();
    // SAFETY: caller contract; `escaped` is owned and freed here.
    let escaped = unsafe { vim_strsave_escaped(numbuf.string(tv), c"\\\"\n\r".as_ptr()) };
    let mut t = escaped;
    while unsafe { *t } != NUL as c_char {
        if unsafe { *t } == b'\n' as c_char {
            unsafe { *t = b'n' as c_char };
        } else if unsafe { *t } == b'\r' as c_char {
            unsafe { *t = b'r' as c_char };
        }
        t = unsafe { t.offset(1) };
    }
    // A String is quoted; a Number is surrounded by spaces instead.
    let quote = if kind == VAR_STRING { c"\"" } else { c" " };
    let ok = out.puts(c"let ")
        && unsafe { out.bytes(key) }
        && out.puts(c" = ")
        && out.puts(quote)
        && unsafe { out.bytes(escaped) }
        && out.puts(quote)
        && out.eol();
    unsafe { xfree(escaped.cast::<c_void>()) };
    ok
}
