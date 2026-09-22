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
#![allow(unsafe_code)]

use crate::cstr;
use crate::ex_cmds::EcmdFlags;
use crate::ex_cmds::newlnum;
use crate::guard::Suppress;
use crate::semsg;
use crate::types::OptStr;
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr;

use crate::arglist::alist_name;
use crate::arglist::state::arg_had_last;
use crate::buffer::state::{
    BLN_LISTED, SEA_DIALOG, SEA_NONE, SEA_QUIT, swap_exists_action, swap_exists_did_quit,
};
use crate::buffer::{
    buf_is_empty, buflist_new, do_modelines, handle_swap_exists, open_buffer, set_buflisted,
    set_curbuf, setfname,
};
use crate::eval::typval::{list_set_lock, tv_list_alloc};
use crate::eval::vars::set_vim_var_list;
use crate::ex_cmds::do_ecmd;
use crate::ex_docmd::do_cmdline_cmd;
use crate::fileio::readfile;
use crate::fileio::state::{READ_NEW, READ_STDIN};
use crate::getchar::state::got_int;
use crate::getchar::vgetc;
use crate::memline::ml_recover;
use crate::memory::{xfree, xstrdup};
use crate::message::msg_putchar;
use crate::message::state::{did_emsg, msg_didany, msg_scroll, no_wait_return};
use crate::option::vars::{P_EF, P_EFM, P_MENC, p_ef, p_fdls, p_shm};
use crate::option::{set_option_direct, set_option_value_give_err};
use crate::os::cshim::snprintf;
use crate::os::input::os_breakcheck;
use crate::path::vim_full_name;
use crate::profile::time_msg_at;
use crate::quickfix::qf_init;
use crate::runtime::state::SID_CARG;
use crate::startup::exit::getout;
use crate::startup::{
    EDIT_QF, MainParams, WIN_HOR, WIN_TABS, WIN_VER, kOptErrorfile, kOptShortmess, recoverymode,
};
use crate::strings::vim_snprintf;
use crate::types::{
    ExArg, Handle, IOSIZE, Integer, LineNr, MAXPATHL, OptInt, OptVal, OptionSetFlags, VarLock, Vv,
    kListLenMayKnow, ptrdiff_t, size_t, ssize_t,
};
use crate::ui::ui_call_error_exit;
use crate::window::{
    goto_tabpage, make_tabpages, make_windows, only_one_window, win_close, win_count, win_enter,
    win_equal,
};

use crate::arglist::global_arglist;
use crate::pos::MAXLNUM;
use crate::startup::exit::os_exit;
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
    getout(1)
}

/// Set `v:argf` to the full paths of the file arguments.
pub(crate) fn set_argf_var() {
    let mut full = [0 as c_char; MAXPATHL as usize];
    let held = tv_list_alloc(kListLenMayKnow as c_int as ptrdiff_t);
    let list = held.as_ptr();
    let alist = global_arglist();
    for i in 0..unsafe { (*alist).al_ga.len() as c_int } {
        let fname = unsafe { alist_name(((*alist).al_ga.as_mut_ptr()).offset(i as isize)) };
        if !fname.is_null() {
            let _ = unsafe { vim_full_name(fname, full.as_mut_ptr(), MAXPATHL as usize, false) };
            unsafe { (*list).push_string(full.as_mut_ptr(), -1 as ssize_t) };
        }
    }
    list_set_lock(unsafe { list.as_mut() }, VarLock::Fixed);
    set_vim_var_list(Vv::Argf, Some(held));
}

/// The first file argument, which is what decides whether `-r` lists the swap
/// files or recovers one.
pub(crate) fn get_fname(_parmp: *mut MainParams) -> *mut c_char {
    // SAFETY: only reached when the argument list is non-empty.
    unsafe { alist_name((*global_arglist()).al_ga.as_mut_ptr()) }
}

/// `-q`: read the errorfile and set up the quickfix list.
///
/// A quickfix list that cannot be built is fatal, with status 3.
///
/// # Safety
///
/// `paramp` must point at the startup parameters.
pub(crate) unsafe fn handle_quickfix(paramp: *mut MainParams) {
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
            // SAFETY: NUL-terminated and outlives the call, which copies.
            OptVal::String(unsafe { OptStr::borrowing(parm.use_ef) }),
            OptionSetFlags::NONE,
            SID_CARG,
        );
    }
    // The title of the list is the command that would have made it.
    let into = title.as_mut_ptr();
    let fmt = c"cfile %s".as_ptr();
    p_ef(|value| unsafe { vim_snprintf(into, IOSIZE as size_t, fmt, value.as_ptr().cast_mut()) });
    // Copies: `qf_init` reads a file and fires autocommands.
    let (ef, efm, enc) = (P_EF.get(), P_EFM.get(), P_MENC.get());
    let (ef, efm, enc) = (
        ef.as_ptr().cast_mut(),
        efm.as_ptr().cast_mut(),
        enc.as_ptr().cast_mut(),
    );
    if unsafe { qf_init(None, ef, efm, true, 1, title.as_mut_ptr(), enc) } < 0 {
        msg_putchar('\n' as c_int);
        os_exit(3);
    }
    time_msg_at(c"reading errorfile");
}

/// `-t`: jump to a tag instead of opening a file.
///
/// # Safety
///
/// `tagname` must point at a NUL-terminated string, unaliased for the call.
pub(crate) unsafe fn handle_tag(tagname: *mut c_char) {
    let mut cmd = [0 as c_char; IOSIZE as usize];
    // SAFETY: `tagname`, when non-null, points into argv.
    if tagname.is_null() {
        return;
    }
    swap_exists_did_quit.set(false);
    let into = cmd.as_mut_ptr();
    unsafe { vim_snprintf(into, IOSIZE as size_t, c"ta %s".as_ptr(), tagname) };
    // SAFETY: `vim_snprintf` terminated the buffer above.
    let _ = do_cmdline_cmd(unsafe { cstr::at(cmd.as_ptr()) });
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
pub(crate) fn read_stdin() {
    // SAFETY: creates and switches buffers, all of which are live for the
    // duration.
    // Use a dialog for the ATTENTION prompt, not a message.
    swap_exists_action.set(SEA_DIALOG);
    no_wait_return.set(1);
    let save_msg_didany = msg_didany.get();

    if !Buf::current().name.full().is_none() {
        let stdin_buf =
            unsafe { buflist_new(ptr::null_mut(), ptr::null_mut(), 0, BLN_LISTED as c_int) };
        if stdin_buf.is_none() {
            semsg!("Failed to create buffer for stdin");
            return;
        }
        let initial_buf_handle: Handle = Buf::current().handle;
        set_curbuf(stdin_buf.expect("a live handle"), 0, false);
        let last = MAXLNUM as LineNr;
        let _null_ea = ptr::null_mut::<ExArg>();
        let flags = READ_NEW as c_int + READ_STDIN as c_int;
        let (no_fname, no_sfname) = (ptr::null_mut(), ptr::null_mut());
        let _ = unsafe { readfile(no_fname, no_sfname, 0, 0, last, None, flags, true) };
        let stdin_buf_handle: Handle = stdin_buf.map_or(0, |b| b.handle);
        let stdin_buf_empty = buf_is_empty(Buf::current());

        // Done as commands rather than calls so the autocommands and the
        // window bookkeeping happen as they would for the user.
        let mut cmd: [c_char; 100] = [0; 100];
        let (into, size) = (cmd.as_mut_ptr(), size_of::<[c_char; 100]>());
        let fmt = c"silent! buffer %d".as_ptr();
        unsafe { vim_snprintf(into, size, fmt, initial_buf_handle) };
        // SAFETY: `vim_snprintf` terminated the buffer above.
        let _ = do_cmdline_cmd(unsafe { cstr::at(cmd.as_ptr()) });
        if stdin_buf_empty {
            let (into, size) = (cmd.as_mut_ptr(), size_of::<[c_char; 100]>());
            let fmt = c"silent! bwipeout! %d".as_ptr();
            unsafe { vim_snprintf(into, size, fmt, stdin_buf_handle) };
            // SAFETY: `vim_snprintf` terminated the buffer above.
            let _ = do_cmdline_cmd(unsafe { cstr::at(cmd.as_ptr()) });
        }
    } else {
        set_buflisted(1);
        let _ = open_buffer(true, None, 0);
        if buf_is_empty(Buf::current()) && Buf::current().b_next.is_some() {
            let _ = do_cmdline_cmd(c"silent! bnext");
            let _ = do_cmdline_cmd(c"silent! bwipeout 1");
        }
    }

    no_wait_return.set(0);
    msg_didany.set(save_msg_didany);
    time_msg_at(c"reading stdin");
    check_swap_exists_action();
}

/// How many times the "open a buffer for every window" loop below may start
/// over before giving up. An autocommand that keeps splitting would otherwise
/// keep it going forever.
const MAX_WINDOW_PASSES: c_int = 1000;

/// The parameter block `main` filled in, which outlives every call here.
type Mp = Live<MainParams>;

/// Make the windows and tab pages the command line asked for, and give every
/// one of them a buffer.
///
/// # Safety
///
/// `parmp` must point at the startup parameters.
pub(crate) unsafe fn create_windows(parmp: *mut MainParams) {
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
        unsafe { parm.window_count = (*global_arglist()).al_ga.len() as c_int };
    }
    if parm.window_count > 1 {
        // Leave the layout alone if a vimrc command already split it.
        if parm.window_layout == 0 {
            parm.window_layout = WIN_HOR as c_int;
        }
        if parm.window_layout == WIN_TABS as c_int {
            parm.window_count = make_tabpages(parm.window_count);
            time_msg_at(c"making tab pages");
        } else if first_win().next().is_none_or(|next| next.w_floating) {
            let (count, vertical) = (parm.window_count, parm.window_layout == WIN_VER as c_int);
            parm.window_count = make_windows(count, vertical);
            time_msg_at(c"making windows");
        } else {
            parm.window_count = win_count();
        }
    } else {
        parm.window_count = 1;
    }

    if recoverymode.get() {
        msg_scroll.set(1);
        ml_recover(true);
        if Buf::current().b_ml.ml_mfp.is_null() {
            // Recovery failed; there is nothing to edit.
            getout(1);
        }
        do_modelines(OptionSetFlags::NONE);
        return;
    }

    // Open a buffer for the windows that do not have one yet. Commands in
    // the vimrc may have loaded a file or split the window, and an
    // autocommand may delete one while we walk -- hence the rewind.
    let quiet = Suppress::win_enter_leave_autocmds();

    let mut dorewind = true;
    let mut passes = 0;
    while passes < MAX_WINDOW_PASSES {
        passes += 1;
        if dorewind {
            if parm.window_layout == WIN_TABS as c_int {
                goto_tabpage(1);
            } else {
                first_win().make_current();
            }
        } else if parm.window_layout == WIN_TABS as c_int {
            if TabPage::current().next().is_none() {
                break;
            }
            goto_tabpage(0);
        } else {
            let Some(next) = Win::current().next() else {
                break;
            };
            next.make_current();
        }
        dorewind = false;
        Win::current().buffer().make_current();

        if Buf::current().b_ml.ml_mfp.is_null() {
            if p_fdls() >= 0 as OptInt {
                Win::current().w_onebuf_opt.wo_fdl = p_fdls();
            }
            // Ask, rather than print, if the swap file is in the way.
            swap_exists_action.set(SEA_DIALOG);
            set_buflisted(1);
            let _ = open_buffer(false, None, 0);

            if swap_exists_action.get() == SEA_QUIT {
                if got_int.get() || only_one_window() {
                    quit_on_swap_exists(true);
                }
                // The window cannot be closed here without disturbing
                // what comes next: clear the name and mark the argument
                // index so it is deleted later.
                let _ = setfname(Buf::current(), None, None, false);
                Win::current().w_arg_idx = -1;
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
        first_win().make_current();
    }
    Win::current().buffer().make_current();
    drop(quiet);
}

/// Load the remaining file arguments into the windows [`create_windows`]
/// made, and leave the cursor in the first non-preview one.
///
/// # Safety
///
/// `parmp` must point at the startup parameters.
pub(crate) unsafe fn edit_buffers(parmp: *mut MainParams) {
    // SAFETY: `parmp` is the caller's live parameter block; the window list
    // is global and `do_ecmd` may close windows through autocommands.
    let parm = unsafe { Mp::new(parmp) };
    let no_enter = Suppress::win_enter_autocmds();
    let no_leave = Suppress::win_leave_autocmds();

    // `create_windows` marks a window whose file could not be opened.
    let mut advance = true;
    if Win::current().w_arg_idx == -1 {
        win_close(Win::current(), true, false);
        advance = false;
    }

    // 'shortmess' with F added, saved for restoring after the tab pages
    // are filled: the per-file messages are noise when there are many.
    let mut p_shm_save: *mut c_char = ptr::null_mut();

    let mut arg_idx: c_int = 1;
    for i in 1..parm.window_count {
        if Win::current().w_arg_idx == -1 {
            arg_idx += 1;
            win_close(Win::current(), true, false);
            advance = false;
            continue;
        }

        if advance {
            if parm.window_layout == WIN_TABS as c_int {
                if TabPage::current().next().is_none() {
                    break;
                }
                goto_tabpage(0);
                if i == 1 {
                    p_shm_save = p_shm(|value| unsafe { xstrdup(value.as_ptr().cast_mut()) });
                    let mut shm: [c_char; 100] = [0; 100];
                    let (into, size) = (shm.as_mut_ptr(), size_of::<[c_char; 100]>());
                    p_shm(|value| unsafe {
                        snprintf(into, size, c"F%s".as_ptr(), value.as_ptr().cast_mut())
                    });
                    unsafe { set_shortmess(shm.as_mut_ptr()) };
                }
            } else {
                let Some(next) = Win::current().next() else {
                    break;
                };
                win_enter(next, false);
            }
        }
        advance = true;

        // Only load a file into a window that is still showing the first
        // window's buffer, or an unnamed one.
        if Buf::current_raw() == first_win().w_buffer || Buf::current().name.full().is_none() {
            Win::current().w_arg_idx = arg_idx;
            swap_exists_did_quit.set(false);
            let alist = global_arglist();
            let name = if arg_idx < unsafe { (*alist).al_ga.len() as c_int } {
                let entries = unsafe { (*alist).al_ga.as_mut_ptr() };
                unsafe { alist_name(entries.offset(arg_idx as isize)) }
            } else {
                ptr::null_mut()
            };
            let (last, hide) = (newlnum::LASTL as LineNr, EcmdFlags::HIDE);
            let _null_ea = ptr::null_mut::<ExArg>();
            let win = Win::current().id();
            let _ = unsafe { do_ecmd(0, name, ptr::null_mut(), None, last, hide, Some(win)) };
            if swap_exists_did_quit.get() {
                if got_int.get() || only_one_window() {
                    quit_on_swap_exists(true);
                }
                win_close(Win::current(), true, false);
                advance = false;
            }
            if arg_idx == unsafe { (*alist).al_ga.len() as c_int } - 1 {
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
    // The release order is load-bearing: entering the first non-preview
    // window below fires `WinEnter`/`BufEnter` but still no `WinLeave`.
    drop(no_enter);

    // Start in the first window that is not a preview.
    let mut win = first_win();
    while win.w_onebuf_opt.wo_pvw != 0 {
        let Some(next) = win.next() else {
            win = first_win();
            break;
        };
        win = next;
    }
    win_enter(win, false);
    drop(no_leave);

    time_msg_at(c"editing files in windows");
    if parm.window_count > 1 && parm.window_layout != WIN_TABS as c_int {
        win_equal(Win::current_or_none(), false, 'b' as c_int);
    }
}

/// Set 'shortmess' to `value`, reporting an error the way `:set` would.
///
/// # Safety
///
/// `value` must point at a NUL-terminated string, unaliased for the call.
unsafe fn set_shortmess(value: *mut c_char) {
    // SAFETY: `value` is a NUL-terminated string that outlives the call; the
    // option layer copies it.
    set_option_value_give_err(
        kOptShortmess,
        OptVal::String(unsafe { OptStr::borrowing(value) }),
        OptionSetFlags::NONE,
    );
}

/// Act on the ATTENTION prompt's answer after a buffer was loaded.
pub(crate) fn check_swap_exists_action() {
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
