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

use super::flag::{FR_COL, FR_LEAF};
use super::view::put_view;
use super::{
    SessionFile, SessionOpts, did_lcd, ses_arglist, ses_do_win, ses_escape_fname, ses_fname,
    ses_put_fname,
};
use crate::buffer::{bt_help, bt_nofilename, bt_terminal};
use crate::eval::typval::tv_get_string;
use crate::eval::var_flavour;
use crate::eval::vars::get_globvar_dict;
use crate::hashtab::hash_removed;
use crate::main::{
    Columns, Rows, curtab, curwin, first_tabpage, firstbuf, firstwin, global_alist, globaldir,
    p_shm, p_stal, p_wh, p_wiw, topframe,
};
use crate::memory::xfree;
use crate::options::{
    kOptSsopFlagBuffers, kOptSsopFlagCurdir, kOptSsopFlagGlobals, kOptSsopFlagHelp,
    kOptSsopFlagOptions, kOptSsopFlagResize, kOptSsopFlagSesdir, kOptSsopFlagTabpages,
    kOptSsopFlagTerminal, kOptSsopFlagWinsize,
};
use crate::os::env::home_replace_save;
use crate::strings::vim_strsave_escaped;
use crate::types::{
    NUL, VAR_FLAVOUR_SESSION, VAR_FLOAT, VAR_NUMBER, VAR_STRING, VarType, buf_T, dictitem_T,
    frame_T, hashitem_T, int64_t, typval_T, win_T,
};
use crate::window::tabpage_index;
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
    if opts.has(kOptSsopFlagGlobals) && !unsafe { store_session_globals(out) } {
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
    if !unsafe { put_buffer_list(out, only_save_windows) } {
        return false;
    }

    // SAFETY: the global argument list is live for the call.
    if !unsafe {
        ses_arglist(
            out,
            c"argglobal",
            &raw mut (*global_alist.ptr()).al_ga,
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
    // SAFETY: the tab page list is live.
    let restore_stal = unsafe { p_stal.get() == 1 && !(*first_tabpage.get()).tp_next.is_null() };
    if restore_stal && !out.line(c"set stal=2") {
        return false;
    }

    // Whether the tab pass pinned 'winminheight'/'winminwidth', which the
    // epilogue then has to put back.
    let mut restore_height_width = false;
    // SAFETY: the tab page and window lists are live; `put_tabs` runs no
    // Vimscript, so nothing can change them under it.
    if !unsafe { put_tabs(out, &mut restore_height_width) } {
        return false;
    }

    if opts.has(kOptSsopFlagTabpages) {
        let index = tabpage_index(curtab.get());
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
    unsafe {
        let dir = if globaldir.get().is_null() {
            dirnow
        } else {
            globaldir.get()
        };
        let sname = home_replace_save(ptr::null_mut::<buf_T>(), dir);
        let fname_esc = ses_escape_fname(sname);
        let ok = out.puts(c"cd ") && out.bytes(fname_esc) && out.eol();
        xfree(fname_esc.cast::<c_void>());
        xfree(sname.cast::<c_void>());
        ok
    }
}

/// One `badd +<lnum> <name>` per buffer worth restoring.
///
/// # Safety
/// The buffer list is live.
unsafe fn put_buffer_list(out: SessionFile, only_save_windows: bool) -> bool {
    let opts = SessionOpts::Session;
    // SAFETY: caller contract; each buffer's window info is its own kvec.
    unsafe {
        let mut buf = firstbuf.get();
        while !buf.is_null() {
            let wanted = !(only_save_windows && (*buf).b_nwindows == 0)
                && !((*buf).b_help && !opts.has(kOptSsopFlagHelp))
                && !(bt_terminal(buf) && !opts.has(kOptSsopFlagTerminal))
                && !(*buf).b_fname.is_null()
                && (*buf).b_p_bl != 0;
            if wanted {
                let lnum = if (*buf).b_wininfo.size == 0 {
                    1 as int64_t
                } else {
                    (**(*buf).b_wininfo.items).wi_mark.mark.lnum as int64_t
                };
                if !out.write(format_args!("badd +{lnum} ")) || !ses_fname(out, buf, opts, true) {
                    return false;
                }
            }
            buf = (*buf).b_next;
        }
    }
    true
}

/// The per-tab-page pass: the window layout, the sizes, and one
/// [`put_view`] per window. When "tabpages" is not in 'sessionoptions' this
/// does the current tab page only.
///
/// # Safety
/// The tab page and window lists are live.
unsafe fn put_tabs(out: SessionFile, restore_height_width: &mut bool) -> bool {
    let opts = SessionOpts::Session;
    let with_tabs = opts.has(kOptSsopFlagTabpages);

    // Populate the tab pages first, as the window layout is populated first
    // below, so that local options set later are not copied into new tabs.
    // `bufhidden=wipe` drops the placeholder buffers once they are unneeded
    // (Vim patch 8.1.0829).
    // SAFETY: caller contract.
    unsafe {
        if with_tabs {
            let mut tp = first_tabpage.get();
            while !tp.is_null() {
                if !(*tp).tp_next.is_null() && !out.line(c"tabnew +setlocal\\ bufhidden=wipe") {
                    return false;
                }
                tp = (*tp).tp_next;
            }
            if !(*first_tabpage.get()).tp_next.is_null() && !out.line(c"tabrewind") {
                return false;
            }
        }
    }

    let mut restore_size = true;
    let mut cur_arg_idx = 0;
    let mut next_arg_idx = 0;
    // The one window whose file `makeopens` already `:edit`ed, so that
    // `put_view` need not do it again.
    let mut edited_win = ptr::null_mut::<win_T>();

    // SAFETY: caller contract; nothing here runs Vimscript, so the lists
    // cannot change under the walk.
    unsafe {
        let mut tp = first_tabpage.get();
        while !tp.is_null() {
            let mut need_tabnext = false;
            let (tab_firstwin, tab_topframe) = if with_tabs {
                need_tabnext = tp != first_tabpage.get();
                if tp == curtab.get() {
                    (firstwin.get(), topframe.get())
                } else {
                    ((*tp).tp_firstwin, (*tp).tp_topframe)
                }
            } else {
                tp = curtab.get();
                (firstwin.get(), topframe.get())
            };

            // Before creating the layout, try loading one file: if that is
            // aborted we do not end up with a pile of useless windows. This
            // may have side effects (a compressed or network file).
            let mut wp = tab_firstwin;
            while !wp.is_null() {
                if ses_do_win(wp)
                    && !(*(*wp).w_buffer).b_ffname.is_null()
                    && !bt_help((*wp).w_buffer)
                    && !bt_nofilename((*wp).w_buffer)
                {
                    if need_tabnext && !out.line(c"tabnext") {
                        return false;
                    }
                    need_tabnext = false;
                    if !out.puts(c"edit ") || !ses_fname(out, (*wp).w_buffer, opts, true) {
                        return false;
                    }
                    if (*wp).w_arg_idx_invalid == 0 {
                        edited_win = wp;
                    }
                    break;
                }
                wp = (*wp).w_next;
            }
            // No file got edited: create an empty tab page.
            if need_tabnext && !out.line(c"tabnext") {
                return false;
            }

            if (*tab_topframe).fr_layout != FR_LEAF {
                if !out.line(c"let s:save_splitbelow = &splitbelow")
                    || !out.line(c"let s:save_splitright = &splitright")
                    || !out.line(c"set splitbelow splitright")
                    || !ses_win_rec(out, tab_topframe)
                    || !out.line(c"let &splitbelow = s:save_splitbelow")
                    || !out.line(c"let &splitright = s:save_splitright")
                {
                    return false;
                }
            }

            // Can the window sizes be restored -- that is, was no window
            // omitted? And which window number is the current one?
            let mut nr = 0;
            let mut cnr = 1;
            let mut wp = tab_firstwin;
            while !wp.is_null() {
                if ses_do_win(wp) {
                    nr += 1;
                } else if !(*wp).w_floating {
                    restore_size = false;
                }
                if curwin.get() == wp {
                    cnr = nr;
                }
                wp = (*wp).w_next;
            }

            if !tab_firstwin.is_null() && !(*tab_firstwin).w_next.is_null() {
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
                if !out
                    .line(c"set winminheight=0\nset winheight=1\nset winminwidth=0\nset winwidth=1")
                {
                    return false;
                }
                *restore_height_width = true;
            }
            if nr > 1 && !ses_winsizes(out, restore_size, tab_firstwin) {
                return false;
            }

            // The tab-local working directory goes before the windows, so a
            // window-local one can override it.
            if opts.has(kOptSsopFlagCurdir) && !(*tp).tp_localdir.is_null() {
                if !out.puts(c"tcd ") || !ses_put_fname(out, (*tp).tp_localdir) || !out.eol() {
                    return false;
                }
                did_lcd.set(true);
            }

            // Each window's view.
            let mut wp = tab_firstwin;
            while !wp.is_null() {
                if ses_do_win(wp) {
                    if !put_view(out, wp, tp, wp != edited_win, opts, cur_arg_idx) {
                        return false;
                    }
                    if nr > 1 && !out.line(c"wincmd w") {
                        return false;
                    }
                    next_arg_idx = (*wp).w_arg_idx;
                }
                wp = (*wp).w_next;
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
            if nr > 1 && !ses_winsizes(out, restore_size, tab_firstwin) {
                return false;
            }

            if !with_tabs {
                break;
            }
            tp = (*tp).tp_next;
        }
    }
    true
}

/// The window sizes for one tab page, as arithmetic against `&lines` and
/// `&columns`. When a window was omitted the numbers would not add up, so
/// the sizes are just equalised instead.
///
/// # Safety
/// `tab_firstwin` heads a live window list.
unsafe fn ses_winsizes(out: SessionFile, restore_size: bool, tab_firstwin: *mut win_T) -> bool {
    if !restore_size || !SessionOpts::Session.has(kOptSsopFlagWinsize) {
        return out.line(c"wincmd =");
    }
    // SAFETY: caller contract; `topframe` is the current tab's frame tree.
    unsafe {
        let mut n = 0;
        let mut wp = tab_firstwin;
        while !wp.is_null() {
            if ses_do_win(wp) {
                n += 1;
                // Restore the height when the window is not full height.
                if (*wp).w_height + (*wp).w_hsep_height + (*wp).w_status_height
                    < (*topframe.get()).fr_height
                    && !out.write(format_args!(
                        "exe '{n}resize ' . ((&lines * {} + {}) / {})\n",
                        (*wp).w_height as int64_t,
                        Rows.get() as int64_t / 2,
                        Rows.get() as int64_t,
                    ))
                {
                    return false;
                }
                // And the width when it is not full width.
                if (*wp).w_width < Columns.get()
                    && !out.write(format_args!(
                        "exe 'vert {n}resize ' . ((&columns * {} + {}) / {})\n",
                        (*wp).w_width as int64_t,
                        Columns.get() as int64_t / 2,
                        Columns.get() as int64_t,
                    ))
                {
                    return false;
                }
            }
            wp = (*wp).w_next;
        }
    }
    true
}

/// Write the splits that recreate the windows of frame `fr`, recursively.
/// Afterwards the last window in the frame is the current one.
///
/// # Safety
/// `fr` is a live frame.
unsafe fn ses_win_rec(out: SessionFile, fr: *mut frame_T) -> bool {
    // SAFETY: caller contract; the frame tree is live.
    unsafe {
        if (*fr).fr_layout == FR_LEAF {
            return true;
        }
        let column = (*fr).fr_layout == FR_COL;

        // Find the first frame that is not skipped, then create a window for
        // each one after it -- the first window is already there.
        let mut count = 0;
        let mut frc = ses_skipframe((*fr).fr_child);
        if !frc.is_null() {
            loop {
                frc = ses_skipframe((*frc).fr_next);
                if frc.is_null() {
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
        }

        // Go back to the first window.
        if count > 0 {
            let direction = if column { 'k' } else { 'h' };
            if !out.write(format_args!("{count}wincmd {direction}\n")) {
                return false;
            }
        }

        // Then recurse into each window of this column or row.
        frc = ses_skipframe((*fr).fr_child);
        while !frc.is_null() {
            ses_win_rec(out, frc);
            frc = ses_skipframe((*frc).fr_next);
            if !frc.is_null() && !out.line(c"wincmd w") {
                return false;
            }
        }
    }
    true
}

/// The first frame at or after `fr` holding a window worth saving, or null.
///
/// # Safety
/// `fr` is null or a live frame.
unsafe fn ses_skipframe(fr: *mut frame_T) -> *mut frame_T {
    // SAFETY: caller contract.
    unsafe {
        let mut frc = fr;
        while !frc.is_null() && !ses_do_frame(frc) {
            frc = (*frc).fr_next;
        }
        frc
    }
}

/// Whether frame `fr` holds a window worth saving anywhere below it.
///
/// # Safety
/// `fr` is a live frame.
unsafe fn ses_do_frame(fr: *const frame_T) -> bool {
    // SAFETY: caller contract.
    unsafe {
        if (*fr).fr_layout == FR_LEAF {
            return ses_do_win((*fr).fr_win);
        }
        let mut frc = (*fr).fr_child;
        while !frc.is_null() {
            if ses_do_frame(frc) {
                return true;
            }
            frc = (*frc).fr_next;
        }
        false
    }
}

/// Write the `g:` variables 'sessionoptions' calls sessionable: the Number,
/// String and Float ones whose name says they belong in a session (an
/// uppercase first letter and a lowercase one after it).
///
/// # Safety
/// Main thread; the global variable dict is live.
unsafe fn store_session_globals(out: SessionFile) -> bool {
    // SAFETY: caller contract. The hashtab walk is upstream's: skip the
    // empty and the tombstone slots, and step back from the key to the item
    // it is embedded in.
    unsafe {
        let ht = &raw mut (*get_globvar_dict()).dv_hashtab;
        let mut todo = (*ht).ht_used;
        let mut hi: *mut hashitem_T = (*ht).ht_array;
        while todo != 0 {
            let key_ptr = (*hi).hi_key;
            let live = !key_ptr.is_null() && key_ptr != (&raw const hash_removed).cast_mut().cast();
            if !live {
                hi = hi.offset(1);
                continue;
            }
            todo -= 1;
            let item = key_ptr.byte_sub(DI_KEY_OFFSET).cast::<dictitem_T>();
            let key = (&raw mut (*item).di_key).cast::<c_char>();
            let kind = (*item).di_tv.v_type;
            let sessionable = var_flavour(key) == VAR_FLAVOUR_SESSION;
            if (kind == VAR_NUMBER || kind == VAR_STRING) && sessionable {
                if !put_session_global(out, key, kind, &raw mut (*item).di_tv) {
                    return false;
                }
            } else if kind == VAR_FLOAT && sessionable {
                let f = (*item).di_tv.vval.v_float;
                let sign = if f < 0.0 { b'-' } else { b' ' } as c_int;
                if fprintf(
                    out.raw(),
                    c"let %s = %c%f".as_ptr(),
                    key,
                    sign,
                    if f < 0.0 { -f } else { f },
                ) < 0
                    || !out.eol()
                {
                    return false;
                }
            }
            hi = hi.offset(1);
        }
    }
    true
}

/// A `dictitem_T`'s key sits at a fixed offset inside it, which is how the
/// hashtab walk gets from one back to the other.
const DI_KEY_OFFSET: usize = 17;

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
    tv: *mut typval_T,
) -> bool {
    // SAFETY: caller contract; `escaped` is owned and freed here.
    unsafe {
        let escaped = vim_strsave_escaped(tv_get_string(tv), c"\\\"\n\r".as_ptr());
        let mut t = escaped;
        while *t != NUL as c_char {
            if *t == b'\n' as c_char {
                *t = b'n' as c_char;
            } else if *t == b'\r' as c_char {
                *t = b'r' as c_char;
            }
            t = t.offset(1);
        }
        // A String is quoted; a Number is surrounded by spaces instead.
        let quote = if kind == VAR_STRING { c"\"" } else { c" " };
        let ok = out.puts(c"let ")
            && out.bytes(key)
            && out.puts(c" = ")
            && out.puts(quote)
            && out.bytes(escaped)
            && out.puts(quote)
            && out.eol();
        xfree(escaped.cast::<c_void>());
        ok
    }
}
