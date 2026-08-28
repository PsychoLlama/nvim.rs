//! Turning the file arguments into buffers and windows.
//!
//! The argument list is already built by the time these run; they decide
//! which buffers exist, how many windows and tab pages hold them, and which
//! one the cursor starts in.
//!
//! Every one of them can meet the swap-file ATTENTION prompt, and the user
//! answering "quit" to it is the reason so many of them end in
//! [`quit_on_swap_exists`].

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr;

use crate::api::private::helpers::cstr_as_string;
use crate::arglist::alist_name;
use crate::buffer::{
    buf_is_empty, buflist_new, do_modelines, handle_swap_exists, open_buffer, set_buflisted,
    set_curbuf, setfname,
};
use crate::eval::typval::{tv_list_alloc, tv_list_append_string, tv_list_set_lock};
use crate::eval::vars::set_vim_var_list;
use crate::ex_cmds::do_ecmd;
use crate::ex_docmd::do_cmdline_cmd;
use crate::fileio::readfile;
use crate::getchar::vgetc;
use crate::main::exit::getout;
use crate::main::{
    BLN_LISTED, ECMD_HIDE, ECMD_LASTL, EDIT_QF, READ_NEW, READ_STDIN, SEA_DIALOG, SEA_NONE,
    SEA_QUIT, SID_CARG, WIN_HOR, WIN_TABS, WIN_VER, arg_had_last, autocmd_no_enter,
    autocmd_no_leave, curbuf, curwin, did_emsg, got_int, kOptErrorfile, kOptShortmess,
    kOptValTypeString, mparm_T, msg_didany, msg_scroll, no_wait_return, p_ef, p_efm, p_fdls,
    p_menc, p_shm, recoverymode, swap_exists_action, swap_exists_did_quit, time_msg_at,
};
use crate::memline::ml_recover;
use crate::memory::{xfree, xstrdup};
use crate::message::msg_putchar;
use crate::option::{set_option_direct, set_option_value_give_err};
use crate::os::cshim::snprintf;
use crate::os::input::os_breakcheck;
use crate::path::vim_full_name;
use crate::quickfix::qf_init;
use crate::strings::vim_snprintf;
use crate::types::{
    IOSIZE, Integer, MAXPATHL, OptInt, OptVal, OptValData, OptionSetFlags, VarLock, Vv, aentry_T,
    exarg_T, handle_T, kListLenMayKnow, linenr_T, list_T, ptrdiff_t, size_t, ssize_t,
};
use crate::ui::ui_call_error_exit;
use crate::window::{
    goto_tabpage, make_tabpages, make_windows, only_one_window, win_close, win_count, win_enter,
    win_equal,
};

use crate::arglist::global_arglist;
use crate::main::exit::os_exit;
use crate::pos::MAXLNUM;
use crate::winlayer::{Buf, Live, TabPage, Win, first_window};

/// The user answered "quit" to the swap-file ATTENTION prompt: leave with
/// status 1.
///
/// `clear_hit_enter` drops `did_emsg` first, so the process does not stop to
/// ask the user to press ENTER for a message on its way out. The tag jump is
/// the one caller that does *not* want that -- it has just printed something
/// worth reading.
fn quit_on_swap_exists(clear_hit_enter: bool) -> ! {
    // SAFETY: clears one flag and leaves; `getout` does not return.
    if clear_hit_enter {
        did_emsg.set(0);
    }
    ui_call_error_exit(1 as Integer);
    unsafe { getout(1) }
}

/// Set `v:argf` to the full paths of the file arguments.
pub(crate) unsafe fn set_argf_var() {
    let mut full = [0 as c_char; MAXPATHL as usize];
    // SAFETY: the global argument list is initialised by `early_init`.
    let list: *mut list_T = unsafe { tv_list_alloc(kListLenMayKnow as c_int as ptrdiff_t) };
    let alist = global_arglist();
    for i in 0..unsafe { (*alist).al_ga.ga_len } {
        let fname =
            unsafe { alist_name(((*alist).al_ga.ga_data as *mut aentry_T).offset(i as isize)) };
        if !fname.is_null() {
            unsafe { vim_full_name(fname, full.as_mut_ptr(), MAXPATHL as usize, false) };
            unsafe { tv_list_append_string(list, full.as_mut_ptr(), -1 as ssize_t) };
        }
    }
    unsafe { tv_list_set_lock(list, VarLock::Fixed) };
    unsafe { set_vim_var_list(Vv::Argf, list) };
}

/// The first file argument, which is what decides whether `-r` lists the swap
/// files or recovers one.
pub(crate) unsafe fn get_fname(_parmp: *mut mparm_T) -> *mut c_char {
    // SAFETY: only reached when the argument list is non-empty.
    unsafe { alist_name((*global_arglist()).al_ga.ga_data as *mut aentry_T) }
}

/// `-q`: read the errorfile and set up the quickfix list.
///
/// A quickfix list that cannot be built is fatal, with status 3.
pub(crate) unsafe fn handle_quickfix(paramp: *mut mparm_T) {
    let mut title = [0 as c_char; IOSIZE as usize];
    // SAFETY: `paramp` is the caller's live parameter block, and `title`
    // outlives the `qf_init` that reads it.
    let parm = unsafe { Mp::new(paramp) };
    if parm.edit_type != EDIT_QF as c_int {
        return;
    }
    if !parm.use_ef.is_null() {
        set_option_direct(
            kOptErrorfile,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: unsafe { cstr_as_string(parm.use_ef) },
                },
            },
            OptionSetFlags::NONE,
            SID_CARG,
        );
    }
    // The title of the list is the command that would have made it.
    let into = title.as_mut_ptr();
    let fmt = c"cfile %s".as_ptr();
    unsafe { vim_snprintf(into, IOSIZE as size_t, fmt, p_ef.get()) };
    let (ef, efm, enc) = (p_ef.get(), p_efm.get(), p_menc.get());
    if unsafe { qf_init(None, ef, efm, 1, title.as_mut_ptr(), enc) } < 0 {
        unsafe { msg_putchar('\n' as c_int) };
        unsafe { os_exit(3) };
    }
    time_msg_at(c"reading errorfile");
}

/// `-t`: jump to a tag instead of opening a file.
pub(crate) unsafe fn handle_tag(tagname: *mut c_char) {
    let mut cmd = [0 as c_char; IOSIZE as usize];
    // SAFETY: `tagname`, when non-null, points into argv.
    if tagname.is_null() {
        return;
    }
    swap_exists_did_quit.set(false);
    let into = cmd.as_mut_ptr();
    unsafe { vim_snprintf(into, IOSIZE as size_t, c"ta %s".as_ptr(), tagname) };
    unsafe { do_cmdline_cmd(cmd.as_mut_ptr()) };
    time_msg_at(c"jumping to tag");
    if swap_exists_did_quit.get() {
        quit_on_swap_exists(false);
    }
}

/// Read the standard input into a buffer, for `nvim -` and for a pipe.
///
/// When a file argument already claimed the current buffer, stdin gets a
/// buffer of its own and the file argument's is restored underneath it -- and
/// the stdin buffer is wiped again if nothing came down the pipe (#8561).
pub(crate) unsafe fn read_stdin() {
    // SAFETY: creates and switches buffers, all of which are live for the
    // duration.
    // Use a dialog for the ATTENTION prompt, not a message.
    swap_exists_action.set(SEA_DIALOG);
    no_wait_return.set(1);
    let save_msg_didany = msg_didany.get();

    if !cur_buf().b_ffname.is_null() {
        let stdin_buf =
            unsafe { buflist_new(ptr::null_mut(), ptr::null_mut(), 0, BLN_LISTED as c_int) };
        if stdin_buf.is_null() {
            unsafe { semsg_c!(c"Failed to create buffer for stdin".as_ptr()) };
            return;
        }
        let initial_buf_handle: handle_T = cur_buf().handle;
        unsafe { set_curbuf(Buf::new(stdin_buf), 0, false) };
        let last = MAXLNUM as c_int as linenr_T;
        let null_ea = ptr::null_mut::<exarg_T>();
        let flags = READ_NEW as c_int + READ_STDIN as c_int;
        let (no_fname, no_sfname) = (ptr::null_mut(), ptr::null_mut());
        unsafe { readfile(no_fname, no_sfname, 0, 0, last, null_ea, flags, true) };
        let stdin_buf_handle: handle_T = unsafe { (*stdin_buf).handle };
        let stdin_buf_empty = unsafe { buf_is_empty(curbuf.get()) };

        // Done as commands rather than calls so the autocommands and the
        // window bookkeeping happen as they would for the user.
        let mut cmd: [c_char; 100] = [0; 100];
        let (into, size) = (cmd.as_mut_ptr(), size_of::<[c_char; 100]>());
        let fmt = c"silent! buffer %d".as_ptr();
        unsafe { vim_snprintf(into, size, fmt, initial_buf_handle) };
        unsafe { do_cmdline_cmd(cmd.as_mut_ptr()) };
        if stdin_buf_empty {
            let (into, size) = (cmd.as_mut_ptr(), size_of::<[c_char; 100]>());
            let fmt = c"silent! bwipeout! %d".as_ptr();
            unsafe { vim_snprintf(into, size, fmt, stdin_buf_handle) };
            unsafe { do_cmdline_cmd(cmd.as_mut_ptr()) };
        }
    } else {
        unsafe { set_buflisted(1) };
        unsafe { open_buffer(true, ptr::null_mut::<exarg_T>(), 0) };
        if unsafe { buf_is_empty(curbuf.get()) } && unsafe { Buf::current() }.b_next.is_some() {
            unsafe { do_cmdline_cmd(c"silent! bnext".as_ptr()) };
            unsafe { do_cmdline_cmd(c"silent! bwipeout 1".as_ptr()) };
        }
    }

    no_wait_return.set(0);
    msg_didany.set(save_msg_didany);
    time_msg_at(c"reading stdin");
    unsafe { check_swap_exists_action() };
}

/// How many times the "open a buffer for every window" loop below may start
/// over before giving up. An autocommand that keeps splitting would otherwise
/// keep it going forever.
const MAX_WINDOW_PASSES: c_int = 1000;

/// The parameter block `main` filled in, which outlives every call here.
type Mp = Live<mparm_T>;

/// Make the windows and tab pages the command line asked for, and give every
/// one of them a buffer.
pub(crate) unsafe fn create_windows(parmp: *mut mparm_T) {
    // SAFETY: `parmp` is the caller's live parameter block; the window and
    // buffer lists are global and may be rearranged by the autocommands the
    // buffer loading fires.
    let mut parm = unsafe { Mp::new(parmp) };
    if parm.window_count == -1 {
        // Not set: one window.
        parm.window_count = 1;
    }
    if parm.window_count == 0 {
        // `-o`/`-O`/`-p` with no count: one per file.
        unsafe { parm.window_count = (*global_arglist()).al_ga.ga_len };
    }
    if parm.window_count > 1 {
        // Leave the layout alone if a vimrc command already split it.
        if parm.window_layout == 0 {
            parm.window_layout = WIN_HOR as c_int;
        }
        if parm.window_layout == WIN_TABS as c_int {
            unsafe { parm.window_count = make_tabpages(parm.window_count) };
            time_msg_at(c"making tab pages");
        } else if first_win().next().is_none_or(|next| next.w_floating) {
            let (count, vertical) = (parm.window_count, parm.window_layout == WIN_VER as c_int);
            parm.window_count = unsafe { make_windows(count, vertical) };
            time_msg_at(c"making windows");
        } else {
            parm.window_count = win_count();
        }
    } else {
        parm.window_count = 1;
    }

    if recoverymode.get() {
        msg_scroll.set(1);
        unsafe { ml_recover(true) };
        if cur_buf().b_ml.ml_mfp.is_null() {
            // Recovery failed; there is nothing to edit.
            unsafe { getout(1) };
        }
        do_modelines(OptionSetFlags::NONE);
        return;
    }

    // Open a buffer for the windows that do not have one yet. Commands in
    // the vimrc may have loaded a file or split the window, and an
    // autocommand may delete one while we walk -- hence the rewind.
    autocmd_no_enter.set(autocmd_no_enter.get() + 1);
    autocmd_no_leave.set(autocmd_no_leave.get() + 1);

    let mut dorewind = true;
    let mut passes = 0;
    while passes < MAX_WINDOW_PASSES {
        passes += 1;
        if dorewind {
            if parm.window_layout == WIN_TABS as c_int {
                goto_tabpage(1);
            } else {
                curwin.set(first_win().raw());
            }
        } else if parm.window_layout == WIN_TABS as c_int {
            if unsafe { TabPage::current() }.next().is_none() {
                break;
            }
            goto_tabpage(0);
        } else {
            let Some(next) = unsafe { Win::current() }.next() else {
                break;
            };
            curwin.set(next.raw());
        }
        dorewind = false;
        curbuf.set(cur_win().w_buffer);

        if cur_buf().b_ml.ml_mfp.is_null() {
            if p_fdls.get() >= 0 as OptInt {
                cur_win().w_onebuf_opt.wo_fdl = p_fdls.get();
            }
            // Ask, rather than print, if the swap file is in the way.
            swap_exists_action.set(SEA_DIALOG);
            unsafe { set_buflisted(1) };
            unsafe { open_buffer(false, ptr::null_mut::<exarg_T>(), 0) };

            if swap_exists_action.get() == SEA_QUIT {
                if got_int.get() || unsafe { only_one_window() } {
                    quit_on_swap_exists(true);
                }
                // The window cannot be closed here without disturbing
                // what comes next: clear the name and mark the argument
                // index so it is deleted later.
                unsafe { setfname(Buf::current(), ptr::null_mut(), ptr::null_mut(), false) };
                cur_win().w_arg_idx = -1;
                swap_exists_action.set(SEA_NONE);
            } else {
                handle_swap_exists(None);
            }
            // The lists may have moved under us.
            dorewind = true;
        }

        os_breakcheck();
        if got_int.get() {
            // Interrupt the file loading, not the rest of the startup.
            vgetc();
            break;
        }
    }

    if parm.window_layout == WIN_TABS as c_int {
        goto_tabpage(1);
    } else {
        curwin.set(first_win().raw());
    }
    curbuf.set(cur_win().w_buffer);
    autocmd_no_enter.set(autocmd_no_enter.get() - 1);
    autocmd_no_leave.set(autocmd_no_leave.get() - 1);
}

/// Load the remaining file arguments into the windows [`create_windows`]
/// made, and leave the cursor in the first non-preview one.
pub(crate) unsafe fn edit_buffers(parmp: *mut mparm_T) {
    // SAFETY: `parmp` is the caller's live parameter block; the window list
    // is global and `do_ecmd` may close windows through autocommands.
    let parm = unsafe { Mp::new(parmp) };
    autocmd_no_enter.set(autocmd_no_enter.get() + 1);
    autocmd_no_leave.set(autocmd_no_leave.get() + 1);

    // `create_windows` marks a window whose file could not be opened.
    let mut advance = true;
    if cur_win().w_arg_idx == -1 {
        unsafe { win_close(curwin.get(), true, false) };
        advance = false;
    }

    // 'shortmess' with F added, saved for restoring after the tab pages
    // are filled: the per-file messages are noise when there are many.
    let mut p_shm_save: *mut c_char = ptr::null_mut();

    let mut arg_idx: c_int = 1;
    for i in 1..parm.window_count {
        if cur_win().w_arg_idx == -1 {
            arg_idx += 1;
            unsafe { win_close(curwin.get(), true, false) };
            advance = false;
            continue;
        }

        if advance {
            if parm.window_layout == WIN_TABS as c_int {
                if unsafe { TabPage::current() }.next().is_none() {
                    break;
                }
                goto_tabpage(0);
                if i == 1 {
                    p_shm_save = unsafe { xstrdup(p_shm.get()) };
                    let mut shm: [c_char; 100] = [0; 100];
                    let (into, size) = (shm.as_mut_ptr(), size_of::<[c_char; 100]>());
                    unsafe { snprintf(into, size, c"F%s".as_ptr(), p_shm.get()) };
                    unsafe { set_shortmess(shm.as_mut_ptr()) };
                }
            } else {
                let Some(next) = unsafe { Win::current() }.next() else {
                    break;
                };
                unsafe { win_enter(next.raw(), false) };
            }
        }
        advance = true;

        // Only load a file into a window that is still showing the first
        // window's buffer, or an unnamed one.
        if curbuf.get() == first_win().w_buffer || cur_buf().b_ffname.is_null() {
            cur_win().w_arg_idx = arg_idx;
            swap_exists_did_quit.set(false);
            let alist = global_arglist();
            let name = if arg_idx < unsafe { (*alist).al_ga.ga_len } {
                let entries = unsafe { (*alist).al_ga.ga_data } as *mut aentry_T;
                unsafe { alist_name(entries.offset(arg_idx as isize)) }
            } else {
                ptr::null_mut()
            };
            let (last, hide) = (ECMD_LASTL as c_int as linenr_T, ECMD_HIDE as c_int);
            let null_ea = ptr::null_mut::<exarg_T>();
            unsafe { do_ecmd(0, name, ptr::null_mut(), null_ea, last, hide, curwin.get()) };
            if swap_exists_did_quit.get() {
                if got_int.get() || unsafe { only_one_window() } {
                    quit_on_swap_exists(true);
                }
                unsafe { win_close(curwin.get(), true, false) };
                advance = false;
            }
            if arg_idx == unsafe { (*alist).al_ga.ga_len } - 1 {
                arg_had_last.set(true);
            }
            arg_idx += 1;
        }

        os_breakcheck();
        if got_int.get() {
            vgetc();
            break;
        }
    }

    if !p_shm_save.is_null() {
        unsafe { set_shortmess(p_shm_save) };
        unsafe { xfree(p_shm_save as *mut c_void) };
    }

    if parm.window_layout == WIN_TABS as c_int {
        goto_tabpage(1);
    }
    autocmd_no_enter.set(autocmd_no_enter.get() - 1);

    // Start in the first window that is not a preview.
    let mut win = first_win();
    while win.w_onebuf_opt.wo_pvw != 0 {
        let Some(next) = win.next() else {
            win = first_win();
            break;
        };
        win = next;
    }
    unsafe { win_enter(win.raw(), false) };
    autocmd_no_leave.set(autocmd_no_leave.get() - 1);

    time_msg_at(c"editing files in windows");
    if parm.window_count > 1 && parm.window_layout != WIN_TABS as c_int {
        unsafe { win_equal(curwin.get(), false, 'b' as c_int) };
    }
}

/// Set 'shortmess' to `value`, reporting an error the way `:set` would.
unsafe fn set_shortmess(value: *mut c_char) {
    // SAFETY: `value` is a NUL-terminated string that outlives the call; the
    // option layer copies it.
    set_option_value_give_err(
        kOptShortmess,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: unsafe { cstr_as_string(value) },
            },
        },
        OptionSetFlags::NONE,
    );
}

/// Act on the ATTENTION prompt's answer after a buffer was loaded.
pub(crate) unsafe fn check_swap_exists_action() {
    if swap_exists_action.get() == SEA_QUIT {
        quit_on_swap_exists(false);
    }
    handle_swap_exists(None);
}

/// The first window of the current tab page, which exists from the moment
/// startup makes it until exit.
fn first_win() -> Win {
    first_window().expect("the editor always has a window")
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
