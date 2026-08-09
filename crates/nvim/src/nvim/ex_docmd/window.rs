//! Splitting, resizing, moving between and listing windows and tab pages.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::autocmd::{EVENT_TABNEWENTERED, apply_autocmds};
use crate::src::nvim::buffer::{bt_quickfix, buf_spname};
use crate::src::nvim::charset::{getdigits, getdigits_int, skipwhite};
use crate::src::nvim::drawscreen::{UPD_CLEAR, UPD_VALID, redraw_later, screen_resize};
use crate::src::nvim::ex_cmds::prepare_tagpreview;
use crate::src::nvim::ex_docmd::argopt::get_tabpage_arg;
use crate::src::nvim::ex_docmd::display::ex_redraw;
use crate::src::nvim::ex_docmd::file::{do_exbuffer, do_exedit};
use crate::src::nvim::ex_docmd::onecmd::fresh_exarg;
use crate::src::nvim::ex_docmd::path::findfunc_find_file;
use crate::src::nvim::ex_docmd::scan::check_nextcmd;
use crate::src::nvim::ex_docmd::source::ex_errmsg;
use crate::src::nvim::ex_docmd::tags::ex_findpat;
use crate::src::nvim::ex_docmd::{FAIL, FNAME_MESS, IOSIZE, NUL};
use crate::src::nvim::file_search::{find_file_in_path, vim_findfile_cleanup};
use crate::src::nvim::highlight_group::HLF_T;
use crate::src::nvim::keycodes::Ctrl_G;
use crate::src::nvim::main::{
    Columns, IObuff, Rows, cmdmod, curbuf, curtab, curwin, e_invarg, e_invarg2, e_invcmd,
    e_invrange, e_screenmode, first_tabpage, firstwin, g_do_tagpreview, got_int, lastused_tabpage,
    msg_col, msg_scroll, must_redraw, p_pvh, postponed_split_flags, postponed_split_tab,
};
use crate::src::nvim::memory::{xfree, xstrlcpy};
use crate::src::nvim::message::{emsg, msg_ext_set_kind, msg_outtrans, msg_putchar, msg_start};
use crate::src::nvim::r#move::validate_cursor;
use crate::src::nvim::normal::do_check_scrollbind;
use crate::src::nvim::option::get_findfunc;
use crate::src::nvim::os::env::home_replace;
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::libc::{atol, gettext, strlen};
use crate::src::nvim::popupmenu::pum_make_popup;
use crate::src::nvim::strings::vim_snprintf;
use crate::src::nvim::types::{
    CMD_new, CMD_sfind, CMD_split, CMD_tabNext, CMD_tabedit, CMD_tabfind, CMD_tabfirst,
    CMD_tablast, CMD_tabnew, CMD_tabprevious, CMD_tabrewind, CMD_vnew, CMD_vsplit, CMOD_KEEPALT,
    exarg_T, intmax_t, size_t, tabpage_T, uint8_t, win_T,
};
use crate::src::nvim::undo::bufIsChanged;
use crate::src::nvim::window::{
    WSP_VERT, do_window, goto_tabpage, tabpage_move, valid_tabpage, win_enter, win_new_tabpage,
    win_setheight_win, win_setwidth_win, win_split, win_valid,
};

/// The number of `win` in the current tab page, counting from one.
///
/// A window not in the list answers the number of windows, which is what
/// `winnr()` reports for one that has just been closed.
pub(crate) unsafe fn current_win_nr(win: *const win_T) -> c_int {
    unsafe {
        let mut nr = 0;
        let mut wp = firstwin.get();
        while !wp.is_null() {
            nr += 1;
            if wp == win as *mut win_T {
                break;
            }
            wp = (*wp).w_next;
        }
        nr
    }
}

/// The same for tab pages. `current_tab_nr(NULL)` is the count.
pub(crate) unsafe fn current_tab_nr(tab: *mut tabpage_T) -> c_int {
    unsafe {
        let mut nr = 0;
        let mut tp = first_tabpage.get();
        while !tp.is_null() {
            nr += 1;
            if tp == tab {
                break;
            }
            tp = (*tp).tp_next;
        }
        nr
    }
}

/// The handler every command modifier carries in the table, for the case
/// where it was typed as a command in its own right.
pub(crate) unsafe fn ex_wrongmodifier(eap: *mut exarg_T) {
    unsafe {
        (*eap).errmsg = gettext(&raw const e_invcmd as *const c_char);
    }
}

/// `:split`, `:vsplit`, `:new`, `:sfind`, `:tabedit`, `:tabnew`,
/// `:tabfind` — open a window or a tab page, then edit into it.
pub unsafe fn ex_splitview(eap: *mut exarg_T) {
    unsafe {
        let ea = &mut *eap;
        let old_curwin = curwin.get();
        let mut fname: *mut c_char = ptr::null_mut();
        let use_tab = ea.cmdidx as c_int == CMD_tabedit as c_int
            || ea.cmdidx as c_int == CMD_tabfind as c_int
            || ea.cmdidx as c_int == CMD_tabnew as c_int;

        // Splitting a quickfix window gives a plain window, not a second
        // quickfix one — unless `:tab` asked for a tab page.
        if bt_quickfix(curbuf.get()) && (*cmdmod.ptr()).cmod_tab == 0 {
            if ea.cmdidx as c_int == CMD_split as c_int {
                ea.cmdidx = CMD_new;
            }
            if ea.cmdidx as c_int == CMD_vsplit as c_int {
                ea.cmdidx = CMD_vnew;
            }
        }

        'theend: {
            // `:sfind`/`:tabfind` resolve the name through 'findfunc' or
            // 'path' before anything is opened.
            if ea.cmdidx as c_int == CMD_sfind as c_int
                || ea.cmdidx as c_int == CMD_tabfind as c_int
            {
                let count = if ea.addr_count > 0 {
                    ea.line2 as c_int
                } else {
                    1
                };
                fname = if *get_findfunc() as c_int != NUL {
                    findfunc_find_file(ea.arg, strlen(ea.arg), count)
                } else {
                    let mut file_to_find: *mut c_char = ptr::null_mut();
                    let mut search_ctx: *mut c_char = ptr::null_mut();
                    let found = find_file_in_path(
                        ea.arg,
                        strlen(ea.arg),
                        FNAME_MESS as c_int,
                        true,
                        (*curbuf.get()).b_ffname,
                        &raw mut file_to_find,
                        &raw mut search_ctx,
                    );
                    xfree(file_to_find as *mut c_void);
                    vim_findfile_cleanup(search_ctx as *mut c_void);
                    found
                };
                if fname.is_null() {
                    break 'theend;
                }
                ea.arg = fname;
            }

            if use_tab {
                // `win_new_tabpage` answers the new window, or null when
                // it could not make one — and then nothing more happens:
                // the file is not edited anywhere.
                let where_ = if (*cmdmod.ptr()).cmod_tab != 0 {
                    (*cmdmod.ptr()).cmod_tab
                } else if ea.addr_count == 0 {
                    0
                } else {
                    ea.line2 as c_int + 1
                };
                if !win_new_tabpage(where_, ea.arg, true, ptr::null_mut()).is_null() {
                    do_exedit(eap, old_curwin);
                    apply_autocmds(
                        EVENT_TABNEWENTERED,
                        ptr::null_mut(),
                        ptr::null_mut(),
                        false,
                        curbuf.get(),
                    );
                    // The window left behind gets the new buffer as its
                    // alternate file.
                    if curwin.get() != old_curwin
                        && win_valid(old_curwin)
                        && (*old_curwin).w_buffer != curbuf.get()
                        && (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as c_int == 0
                    {
                        (*old_curwin).w_alt_fnum = (*curbuf.get()).handle as c_int;
                    }
                }
            } else if win_split(
                if ea.addr_count > 0 {
                    ea.line2 as c_int
                } else {
                    0
                },
                if *ea.cmd as c_int == 'v' as c_int {
                    WSP_VERT as c_int
                } else {
                    0
                },
            ) != FAIL
            {
                // A split that will show a *different* file must not stay
                // bound to the one it came from.
                if *ea.arg as c_int != NUL {
                    (*curwin.get()).w_onebuf_opt.wo_scb = 0;
                    (*curwin.get()).w_onebuf_opt.wo_crb = 0;
                } else {
                    do_check_scrollbind(false);
                }
                do_exedit(eap, old_curwin);
            }
        }
        xfree(fname as *mut c_void);
    }
}

/// Open a new tab page, as `:tabnew` would.
pub unsafe fn tabpage_new() {
    unsafe {
        let mut ea = fresh_exarg();
        ea.line1 = 0;
        ea.line2 = 0;
        ea.arg = c"".as_ptr() as *mut c_char;
        // `ex_splitview` reads the first byte of `cmd` to tell a vertical
        // split from a horizontal one.
        ea.cmd = c"tabn".as_ptr() as *mut c_char;
        ea.cmdidx = CMD_tabnew;
        ex_splitview(&raw mut ea);
    }
}

/// `:tabnext` and its seven siblings.
///
/// `:tabprevious`/`:tabNext` count *backwards*, which `goto_tabpage`
/// spells as a negative argument; the rest go to an absolute number that
/// `get_tabpage_arg` works out.
pub(crate) unsafe fn ex_tabnext(eap: *mut exarg_T) {
    unsafe {
        let ea = &mut *eap;
        let idx = ea.cmdidx as c_int;
        if idx == CMD_tabfirst as c_int || idx == CMD_tabrewind as c_int {
            goto_tabpage(1);
            return;
        }
        if idx == CMD_tablast as c_int {
            // Larger than any tab count.
            goto_tabpage(9999);
            return;
        }
        if idx != CMD_tabprevious as c_int && idx != CMD_tabNext as c_int {
            let tab_number = get_tabpage_arg(eap);
            if ea.errmsg.is_null() {
                goto_tabpage(tab_number);
            }
            return;
        }

        // A count for `:tabprevious` may be an argument or a range, but a
        // *signed* argument is not a count of places to go back — `:tabp -1`
        // is an error, not `:tabp 1`.
        let tab_number;
        if !ea.arg.is_null() && *ea.arg as c_int != NUL {
            let mut p = ea.arg;
            let p_save = p;
            tab_number = getdigits(&raw mut p, false, 0 as intmax_t) as c_int;
            if p == p_save
                || *p_save as c_int == '-' as c_int
                || *p_save as c_int == '+' as c_int
                || *p as c_int != NUL
                || tab_number == 0
            {
                ea.errmsg = ex_errmsg(&raw const e_invarg2 as *const c_char, ea.arg);
                return;
            }
        } else if ea.addr_count == 0 {
            tab_number = 1;
        } else {
            tab_number = ea.line2 as c_int;
            if tab_number < 1 {
                ea.errmsg = gettext(&raw const e_invrange as *const c_char);
                return;
            }
        }
        goto_tabpage(-tab_number);
    }
}

/// `:tabmove`.
pub(crate) unsafe fn ex_tabmove(eap: *mut exarg_T) {
    unsafe {
        let tab_number = get_tabpage_arg(eap);
        if (*eap).errmsg.is_null() {
            tabpage_move(tab_number);
        }
    }
}

/// `:tabs` — every tab page, with its windows.
pub(crate) unsafe fn ex_tabs(_eap: *mut exarg_T) {
    unsafe {
        msg_ext_set_kind(c"list_cmd".as_ptr());
        msg_start();
        msg_scroll.set(1);

        let lastused_win = if valid_tabpage(lastused_tabpage.get()) {
            (*lastused_tabpage.get()).tp_curwin
        } else {
            ptr::null_mut()
        };

        let mut tabcount = 1;
        let mut tp = first_tabpage.get();
        while !tp.is_null() && !got_int.get() {
            if msg_col.get() > 0 {
                msg_putchar('\n' as c_int);
            }
            vim_snprintf(
                IObuff.ptr() as *mut c_char,
                IOSIZE as size_t,
                gettext(c"Tab page %d".as_ptr()),
                tabcount,
            );
            tabcount += 1;
            msg_outtrans(IObuff.ptr() as *mut c_char, HLF_T, false);
            os_breakcheck();

            // The current tab page's window list lives in the globals, not
            // in the `tabpage_T`.
            let mut wp = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() && !got_int.get() {
                // A hidden or unfocusable floating window is not listed.
                if (*wp).w_config.focusable && !(*wp).w_config.hide {
                    msg_putchar('\n' as c_int);
                    msg_putchar(if wp == curwin.get() {
                        '>' as c_int
                    } else if wp == lastused_win {
                        '#' as c_int
                    } else {
                        ' ' as c_int
                    });
                    msg_putchar(' ' as c_int);
                    msg_putchar(if bufIsChanged((*wp).w_buffer) {
                        '+' as c_int
                    } else {
                        ' ' as c_int
                    });
                    msg_putchar(' ' as c_int);
                    let special = buf_spname((*wp).w_buffer);
                    if special.is_null() {
                        home_replace(
                            (*wp).w_buffer,
                            (*(*wp).w_buffer).b_fname,
                            IObuff.ptr() as *mut c_char,
                            IOSIZE as size_t,
                            true,
                        );
                    } else {
                        xstrlcpy(IObuff.ptr() as *mut c_char, special, IOSIZE as size_t);
                    }
                    msg_outtrans(IObuff.ptr() as *mut c_char, 0, false);
                    os_breakcheck();
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next;
        }
    }
}

/// `:mode` — a redraw; the Vim spelling that took a terminal mode name is
/// refused.
pub(crate) unsafe fn ex_mode(eap: *mut exarg_T) {
    unsafe {
        if *(*eap).arg as c_int == NUL {
            must_redraw.set(UPD_CLEAR);
            ex_redraw(eap);
        } else {
            emsg(gettext(&raw const e_screenmode as *const c_char));
        }
    }
}

/// `:resize`, and `:vertical resize`.
///
/// A leading `-` or `+` makes the argument relative — `atol` already read
/// the sign, so the current size is simply added. No argument at all means
/// "as large as possible".
pub(crate) unsafe fn ex_resize(eap: *mut exarg_T) {
    unsafe {
        let ea = &mut *eap;
        let mut wp = curwin.get();
        if ea.addr_count > 0 {
            let mut n = ea.line2 as c_int;
            wp = firstwin.get();
            while !(*wp).w_next.is_null() && {
                n -= 1;
                n > 0
            } {
                wp = (*wp).w_next;
            }
        }

        let relative = *ea.arg as c_int == '-' as c_int || *ea.arg as c_int == '+' as c_int;
        let empty = *ea.arg as c_int == NUL;
        let mut n = atol(ea.arg) as c_int;
        if (*cmdmod.ptr()).cmod_split & WSP_VERT as c_int != 0 {
            if relative {
                n += (*wp).w_width;
            } else if n == 0 && empty {
                n = Columns.get();
            }
            win_setwidth_win(n, wp);
        } else {
            if relative {
                n += (*wp).w_height;
            } else if n == 0 && empty {
                n = Rows.get() - 1;
            }
            win_setheight_win(n, wp);
        }
    }
}

/// The Vim commands that only make sense with a built-in GUI.
pub(crate) unsafe fn ex_nogui(eap: *mut exarg_T) {
    unsafe {
        (*eap).errmsg = gettext(c"E25: Nvim does not have a built-in GUI".as_ptr());
    }
}

/// `:popup`.
pub(crate) unsafe fn ex_popup(eap: *mut exarg_T) {
    unsafe {
        pum_make_popup((*eap).arg, (*eap).forceit);
    }
}

/// `:winsize` — two numbers, and nothing else.
pub(crate) unsafe fn ex_winsize(eap: *mut exarg_T) {
    unsafe {
        let mut arg = (*eap).arg;
        if !ascii_isdigit(*arg as c_int) {
            semsg_c!(gettext(&raw const e_invarg2 as *const c_char), arg);
            return;
        }
        let w = getdigits_int(&raw mut arg, false, 10);
        arg = skipwhite(arg);
        let second = arg;
        let h = getdigits_int(&raw mut arg, false, 10);
        // `second` still pointing at something means there *was* a second
        // number; `arg` at the end means there was nothing after it.
        if *second as c_int != NUL && *arg as c_int == NUL {
            screen_resize(w, h);
        } else {
            emsg(gettext(
                c"E465: :winsize requires two number arguments".as_ptr(),
            ));
        }
    }
}

/// `:wincmd` — one window command, spelled as a command line.
pub(crate) unsafe fn ex_wincmd(eap: *mut exarg_T) {
    unsafe {
        let ea = &mut *eap;
        // `CTRL-W g` takes a second character.
        let mut xchar = NUL;
        let mut p;
        if *ea.arg as c_int == 'g' as c_int || *ea.arg as c_int == Ctrl_G {
            if *ea.arg.add(1) as c_int == NUL {
                emsg(gettext(&raw const e_invarg as *const c_char));
                return;
            }
            xchar = *ea.arg.add(1) as uint8_t as c_int;
            p = ea.arg.add(2);
        } else {
            p = ea.arg.add(1);
        }

        ea.nextcmd = check_nextcmd(p);
        p = skipwhite(p);
        if *p as c_int != NUL && *p as c_int != '"' as c_int && ea.nextcmd.is_null() {
            emsg(gettext(&raw const e_invarg as *const c_char));
        } else if ea.skip == 0 {
            // A `:vertical`/`:tab` in front applies to the split the window
            // command is about to make.
            postponed_split_flags.set((*cmdmod.ptr()).cmod_split);
            postponed_split_tab.set((*cmdmod.ptr()).cmod_tab);
            do_window(
                *ea.arg as c_int,
                if ea.addr_count > 0 {
                    ea.line2 as c_int
                } else {
                    0
                },
                xchar,
            );
            postponed_split_flags.set(0);
            postponed_split_tab.set(0);
        }
    }
}

/// `:psearch` — `:isearch` with the result shown in the preview window.
pub(crate) unsafe fn ex_psearch(eap: *mut exarg_T) {
    unsafe {
        g_do_tagpreview.set(p_pvh.get() as c_int);
        ex_findpat(eap);
        g_do_tagpreview.set(0);
    }
}

/// `:pedit`.
pub(crate) unsafe fn ex_pedit(eap: *mut exarg_T) {
    unsafe {
        let curwin_save = curwin.get();
        prepare_preview_window();
        do_exedit(eap, ptr::null_mut());
        back_to_current_window(curwin_save);
    }
}

/// `:pbuffer`.
pub(crate) unsafe fn ex_pbuffer(eap: *mut exarg_T) {
    unsafe {
        let curwin_save = curwin.get();
        prepare_preview_window();
        do_exbuffer(eap);
        back_to_current_window(curwin_save);
    }
}

/// Open or reuse the preview window, and make it current.
pub(crate) unsafe fn prepare_preview_window() {
    unsafe {
        g_do_tagpreview.set(p_pvh.get() as c_int);
        prepare_tagpreview(true);
    }
}

/// Go back to the window `:pedit` was run from, if it is still there.
pub(crate) unsafe fn back_to_current_window(curwin_save: *mut win_T) {
    unsafe {
        if curwin.get() != curwin_save && win_valid(curwin_save) {
            // The preview window is left drawn but not current.
            validate_cursor(curwin.get());
            redraw_later(curwin.get(), UPD_VALID);
            win_enter(curwin_save, true);
        }
        g_do_tagpreview.set(0);
    }
}
