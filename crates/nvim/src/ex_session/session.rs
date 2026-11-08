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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::flag::{FR_COL, FR_LEAF};
use super::view::put_view;
use super::{
    SessionFile, SessionOpts, Written, did_lcd, ses_arglist, ses_do_win, ses_escape_fname,
    ses_fname, ses_put_fname,
};
use crate::arglist::ArgListId;
use crate::buffer::first_wininfo_lnum;
use crate::buffer::{buf_is_help, buf_is_nofilename, buf_is_terminal};
use crate::env::home_to_tilde;
use crate::eval::typval::NumBuf;
use crate::eval::var_flavour;
use crate::eval::vars::globvar_dict;
use crate::memory::XString;
use crate::option::vars::{p_shm, p_stal, p_wh, p_wiw};
use crate::options::{
    kOptSsopFlagBuffers, kOptSsopFlagCurdir, kOptSsopFlagGlobals, kOptSsopFlagHelp,
    kOptSsopFlagOptions, kOptSsopFlagResize, kOptSsopFlagSesdir, kOptSsopFlagTabpages,
    kOptSsopFlagTerminal, kOptSsopFlagWinsize,
};
use crate::os::libc::int64_t;
use crate::strings::escaped_bytes;
use crate::types::{
    TypVal, VAR_FLAVOUR_SESSION, VAR_FLOAT, VAR_NUMBER, VAR_STRING, VarType, Window,
};
use crate::ui::state::{Columns, Rows};
use crate::window::tab_index;
use crate::window::{DirOwner, own_dir};
use crate::winlayer::graph::firstwin;
use crate::winlayer::{
    FrameRef, TabPage, Win, WinId, buffers, current_topframe, first_tab, tabs, windows_in_tab,
};
use core::ffi::CStr;
use core::ptr;

/// Write everything that restores the current editor state.
///
/// `dirnow` is the working directory as it was before [`super::ex_mkrc`]
/// changed into the session file's own directory, empty when unknown.
pub(crate) fn makeopens(out: &mut SessionFile, dirnow: &XString) -> Written {
    let opts = SessionOpts::Session;
    // "buffers" means save every buffer, not only the ones in a window.
    let only_save_windows = !opts.has(kOptSsopFlagBuffers);

    out.line(c"let v:this_session=expand(\"<sfile>:p\")")?;
    out.line(c"doautoall SessionLoadPre")?;
    if opts.has(kOptSsopFlagGlobals) {
        store_session_globals(out)?;
    }

    // Close all windows and tab pages but one.
    out.line(c"silent only")?;
    if opts.has(kOptSsopFlagTabpages) {
        out.line(c"silent tabonly")?;
    }

    // Then a :cd to the session directory or to the current one.
    put_cd(out, dirnow)?;

    out.line(
        c"if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''\n  let s:wipebuf = bufnr('%')\nendif",
    )?;

    // 'shortmess' is set for the load and restored at the end; save the old
    // value here unless the file carries options anyway.
    if !opts.has(kOptSsopFlagOptions) {
        out.line(c"let s:shortmess_save = &shortmess")?;
    }
    out.line(c"set shortmess+=aoO")?;

    // Put every buffer into the buffer list, very early, so that loading the
    // session cannot disturb their order.
    put_buffer_list(out, only_save_windows)?;

    let names = crate::arglist::arg_names(ArgListId::GLOBAL);
    ses_arglist(out, c"argglobal", &names, !opts.has(kOptSsopFlagCurdir))?;

    if opts.has(kOptSsopFlagResize) {
        out.write(format_args!(
            "set lines={} columns={}\n",
            int64_t::from(Rows.get()),
            int64_t::from(Columns.get())
        ))?;
    }

    // With two or more tab pages and 'showtabline' at 1, the tabline appears
    // when the next tab is created, which resizes the first tab's windows.
    let restore_stal = p_stal() == 1 && first_tab().is_some_and(|tp| tp.next().is_some());
    if restore_stal {
        out.line(c"set stal=2")?;
    }

    // Whether the tab pass pinned 'winminheight'/'winminwidth', which the
    // epilogue then has to put back.
    let mut restore_height_width = false;
    // `put_tabs` runs no Vimscript, so nothing can change the tab page and
    // window lists under it.
    put_tabs(out, &mut restore_height_width)?;

    if opts.has(kOptSsopFlagTabpages) {
        let index = tab_index(TabPage::current());
        out.write(format_args!("tabnext {index}\n"))?;
    }
    if restore_stal {
        out.line(c"set stal=1")?;
    }

    // Wipe out the empty unnamed buffer we started in.
    out.line(
        c"if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'\n  silent exe 'bwipe ' . s:wipebuf\nendif\nunlet! s:wipebuf",
    )?;

    // Re-apply 'winheight' and 'winwidth', which the layout pass set to 1.
    out.write(format_args!(
        "set winheight={} winwidth={}\n",
        p_wh() as int64_t,
        p_wiw() as int64_t
    ))?;

    // Restore 'shortmess'.
    if opts.has(kOptSsopFlagOptions) {
        // 'shortmess''s bytes go out verbatim.
        out.puts(c"set shortmess=")?;
        p_shm(|value| out.put_bytes(value.to_bytes()))?;
        out.eol()?;
    } else {
        out.line(c"let &shortmess = s:shortmess_save")?;
    }

    if restore_height_width {
        out.line(c"let &winminheight = s:save_winminheight")?;
        out.line(c"let &winminwidth = s:save_winminwidth")?;
    }

    // Lastly, source the companion x.vim if there is one.
    out.line(
        c"let s:sx = expand(\"<sfile>:p:r\").\"x.vim\"\nif filereadable(s:sx)\n  exe \"source \" . fnameescape(s:sx)\nendif",
    )
}

/// The `:cd` that fixes what relative file names below are relative to.
fn put_cd(out: &mut SessionFile, dirnow: &XString) -> Written {
    let opts = SessionOpts::Session;
    if opts.has(kOptSsopFlagSesdir) {
        return out.line(c"exe \"cd \" . escape(expand(\"<sfile>:p:h\"), ' ')");
    }
    if !opts.has(kOptSsopFlagCurdir) {
        return Ok(());
    }
    let globaldir = own_dir(DirOwner::Global);
    let dir: &CStr = globaldir.as_ref().unwrap_or(dirnow).as_cstr();
    let sname = home_to_tilde(None, dir, dir.count_bytes() + 3, true);
    let fname_esc = ses_escape_fname(sname.as_cstr());
    out.puts(c"cd ")?;
    out.put_bytes(&fname_esc)?;
    out.eol()
}

/// One `badd +<lnum> <name>` per buffer worth restoring.
fn put_buffer_list(out: &mut SessionFile, only_save_windows: bool) -> Written {
    let opts = SessionOpts::Session;
    for buf in buffers() {
        let wanted = !(only_save_windows && buf.b_nwindows == 0)
            && !(buf.b_help && !opts.has(kOptSsopFlagHelp))
            && !(buf_is_terminal(Some(buf)) && !opts.has(kOptSsopFlagTerminal))
            && !buf.name.is_unnamed()
            && buf.b_p_bl != 0;
        if wanted {
            let lnum = first_wininfo_lnum(buf).map_or(1, int64_t::from);
            out.write(format_args!("badd +{lnum} "))?;
            ses_fname(out, buf, opts, true)?;
        }
    }
    Ok(())
}

/// The per-tab-page pass: the window layout, the sizes, and one
/// [`put_view`] per window. When "tabpages" is not in 'sessionoptions' this
/// does the current tab page only.
fn put_tabs(out: &mut SessionFile, restore_height_width: &mut bool) -> Written {
    let opts = SessionOpts::Session;
    let with_tabs = opts.has(kOptSsopFlagTabpages);

    // Populate the tab pages first, as the window layout is populated first
    // below, so that local options set later are not copied into new tabs.
    // `bufhidden=wipe` drops the placeholder buffers once they are unneeded
    // (Vim patch 8.1.0829).
    if with_tabs {
        for tp in tabs() {
            if tp.next().is_some() {
                out.line(c"tabnew +setlocal\\ bufhidden=wipe")?;
            }
        }
        if first_tab().is_some_and(|tp| tp.next().is_some()) {
            out.line(c"tabrewind")?;
        }
    }

    let mut restore_size = true;
    let mut cur_arg_idx = 0;
    let mut next_arg_idx = 0;
    // The one window whose file `makeopens` already `:edit`ed, so that
    // `put_view` need not do it again.
    let mut edited_win = ptr::null_mut::<Window>();

    // Nothing here runs Vimscript, so the lists cannot change under the
    // walk.
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
                && !buffer.name.full().is_none()
                && !buf_is_help(Some(buffer))
                && !buf_is_nofilename(Some(buffer))
            {
                if need_tabnext {
                    out.line(c"tabnext")?;
                }
                need_tabnext = false;
                out.puts(c"edit ")?;
                ses_fname(out, buffer, opts, true)?;
                if !window.w_arg_idx_invalid {
                    edited_win = window.raw();
                }
                break;
            }
        }
        // No file got edited: create an empty tab page.
        if need_tabnext {
            out.line(c"tabnext")?;
        }

        if tab_topframe.fr_layout != FR_LEAF {
            out.line(c"let s:save_splitbelow = &splitbelow")?;
            out.line(c"let s:save_splitright = &splitright")?;
            out.line(c"set splitbelow splitright")?;
            ses_win_rec(out, tab_topframe)?;
            out.line(c"let &splitbelow = s:save_splitbelow")?;
            out.line(c"let &splitright = s:save_splitright")?;
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
            out.line(c"wincmd t")?;
            if !*restore_height_width {
                out.line(c"let s:save_winminheight = &winminheight")?;
                out.line(c"let s:save_winminwidth = &winminwidth")?;
            }
            out.line(c"set winminheight=0\nset winheight=1\nset winminwidth=0\nset winwidth=1")?;
            *restore_height_width = true;
        }
        if nr > 1 {
            ses_winsizes(out, restore_size, tab)?;
        }

        // The tab-local working directory goes before the windows, so a
        // window-local one can override it.
        if opts.has(kOptSsopFlagCurdir)
            && let Some(localdir) = own_dir(DirOwner::TabPage(tab))
        {
            out.puts(c"tcd ")?;
            ses_put_fname(out, localdir.as_cstr())?;
            out.eol()?;
            did_lcd.set(true);
        }

        // Each window's view.
        for window in windows_in_tab(tab) {
            if ses_do_win(window) {
                let add_edit = window.raw() != edited_win;
                put_view(out, window, tab, add_edit, opts, cur_arg_idx)?;
                if nr > 1 {
                    out.line(c"wincmd w")?;
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
        if cnr > 1 {
            out.write(format_args!("{cnr}wincmd w\n"))?;
        }
        // And restore the sizes again: jumping around gives the current
        // window a minimum size the others may not have.
        if nr > 1 {
            ses_winsizes(out, restore_size, tab)?;
        }

        if !with_tabs {
            break;
        }
        next = tab.next();
    }
    Ok(())
}

/// The window sizes for one tab page, as arithmetic against `&lines` and
/// `&columns`. When a window was omitted the numbers would not add up, so
/// the sizes are just equalised instead.
fn ses_winsizes(out: &mut SessionFile, restore_size: bool, tab: TabPage) -> Written {
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
            if window.w_height + window.w_hsep_height + window.w_status_height < top.fr_height {
                out.write(format_args!(
                    "exe '{n}resize ' . ((&lines * {} + {}) / {})\n",
                    int64_t::from(window.w_height),
                    int64_t::from(Rows.get()) / 2,
                    int64_t::from(Rows.get()),
                ))?;
            }
            // And the width when it is not full width.
            if window.w_width < Columns.get() {
                out.write(format_args!(
                    "exe 'vert {n}resize ' . ((&columns * {} + {}) / {})\n",
                    int64_t::from(window.w_width),
                    int64_t::from(Columns.get()) / 2,
                    int64_t::from(Columns.get()),
                ))?;
            }
        }
    }
    Ok(())
}

/// Write the splits that recreate the windows of frame `fr`, recursively.
/// Afterwards the last window in the frame is the current one.
fn ses_win_rec(out: &mut SessionFile, fr: FrameRef) -> Written {
    if fr.fr_layout == FR_LEAF {
        return Ok(());
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
        out.puts(c"wincmd _ | wincmd |")?;
        out.eol()?;
        out.line(if column { c"split" } else { c"vsplit" })?;
        count += 1;
    }

    // Go back to the first window.
    if count > 0 {
        let direction = if column { 'k' } else { 'h' };
        out.write(format_args!("{count}wincmd {direction}\n"))?;
    }

    // Then recurse into each window of this column or row. A nested
    // frame's write error goes unanswered, as upstream: the next write
    // here meets it again.
    frc = ses_skipframe(fr.child());
    while let Some(current) = frc {
        let _ = ses_win_rec(out, current);
        frc = ses_skipframe(current.next());
        if frc.is_some() {
            out.line(c"wincmd w")?;
        }
    }
    Ok(())
}

/// The first frame at or after `fr` holding a window worth saving.
fn ses_skipframe(fr: Option<FrameRef>) -> Option<FrameRef> {
    let mut frc = fr;
    while let Some(current) = frc.filter(|frc| !(*frc).has_session_window()) {
        frc = current.next();
    }
    frc
}

impl FrameRef {
    /// Whether frame `fr` holds a window worth saving anywhere below it.
    fn has_session_window(self) -> bool {
        match self.win() {
            Some(win) => ses_do_win(win),
            None => self.children().any(FrameRef::has_session_window),
        }
    }
}

/// Write the `g:` variables 'sessionoptions' calls sessionable: the Number,
/// String and Float ones whose name says they belong in a session (an
/// uppercase first letter and a lowercase one after it).
fn store_session_globals(out: &mut SessionFile) -> Written {
    let globals = globvar_dict();
    for item in globals.edit().items_mut() {
        let key = item.di_key.bytes();
        let kind = item.di_tv.v_type();
        let sessionable = var_flavour(key) == VAR_FLAVOUR_SESSION;
        if (kind == VAR_NUMBER || kind == VAR_STRING) && sessionable {
            put_session_global(out, key, kind, &mut item.di_tv)?;
        } else if kind == VAR_FLOAT && sessionable {
            let f = item.di_tv.float_or_zero();
            let (sign, magnitude) = if f < 0.0 { ('-', -f) } else { (' ', f) };
            out.puts(c"let ")?;
            out.put_bytes(key)?;
            out.write(format_args!(" = {sign}{}", printf_f(magnitude)))?;
            out.eol()?;
        }
    }
    Ok(())
}

/// `value` as C's `printf("%f")` writes it: six decimals, and `inf` and
/// `nan` spelled in lower case, with the sign a negative NaN carries.
fn printf_f(value: f64) -> String {
    if value.is_nan() {
        return if value.is_sign_negative() {
            "-nan"
        } else {
            "nan"
        }
        .to_owned();
    }
    if value.is_infinite() {
        return if value < 0.0 { "-inf" } else { "inf" }.to_owned();
    }
    format!("{value:.6}")
}

/// `let <name> = <value>` for one Number or String global. The value is
/// escaped so that the script reads it back unchanged: backslash, quote, and
/// a literal LF or CR turned into `\n`/`\r`.
fn put_session_global(
    out: &mut SessionFile,
    key: &[u8],
    kind: VarType,
    tv: &mut TypVal,
) -> Written {
    let mut numbuf = NumBuf::new();
    let mut escaped = escaped_bytes(numbuf.string(tv).to_bytes(), b"\\\"\n\r").into_vec();
    escaped.pop(); // the terminator
    for byte in &mut escaped {
        match *byte {
            b'\n' => *byte = b'n',
            b'\r' => *byte = b'r',
            _ => {}
        }
    }
    // A String is quoted; a Number is surrounded by spaces instead.
    let quote = if kind == VAR_STRING { c"\"" } else { c" " };
    out.puts(c"let ")?;
    out.put_bytes(key)?;
    out.puts(c" = ")?;
    out.puts(quote)?;
    out.put_bytes(&escaped)?;
    out.puts(quote)?;
    out.eol()
}
