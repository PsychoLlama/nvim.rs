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
    autocmd_no_leave, curbuf, curtab, curwin, did_emsg, firstwin, got_int, kOptErrorfile,
    kOptShortmess, kOptValTypeString, mparm_T, msg_didany, msg_scroll, no_wait_return, p_ef, p_efm,
    p_fdls, p_menc, p_shm, recoverymode, swap_exists_action, swap_exists_did_quit, time_msg_at,
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
    IOSIZE, Integer, MAXPATHL, OptInt, OptVal, OptValData, OptionSetFlags, VAR_FIXED, Vv, aentry_T,
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
use crate::winlayer::Buf;

/// The user answered "quit" to the swap-file ATTENTION prompt: leave with
/// status 1.
///
/// `clear_hit_enter` drops `did_emsg` first, so the process does not stop to
/// ask the user to press ENTER for a message on its way out. The tag jump is
/// the one caller that does *not* want that -- it has just printed something
/// worth reading.
fn quit_on_swap_exists(clear_hit_enter: bool) -> ! {
    // SAFETY: clears one flag and leaves; `getout` does not return.
    unsafe {
        if clear_hit_enter {
            did_emsg.set(0);
        }
        ui_call_error_exit(1 as Integer);
        getout(1)
    }
}

/// Set `v:argf` to the full paths of the file arguments.
pub(crate) unsafe fn set_argf_var() {
    let mut full = [0 as c_char; MAXPATHL as usize];
    // SAFETY: the global argument list is initialised by `early_init`.
    unsafe {
        let list: *mut list_T = tv_list_alloc(kListLenMayKnow as c_int as ptrdiff_t);
        let alist = global_arglist();
        for i in 0..(*alist).al_ga.ga_len {
            let fname = alist_name(((*alist).al_ga.ga_data as *mut aentry_T).offset(i as isize));
            if !fname.is_null() {
                vim_full_name(fname, full.as_mut_ptr(), MAXPATHL as usize, false);
                tv_list_append_string(list, full.as_mut_ptr(), -1 as ssize_t);
            }
        }
        tv_list_set_lock(list, VAR_FIXED);
        set_vim_var_list(Vv::Argf, list);
    }
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
    unsafe {
        if (*paramp).edit_type != EDIT_QF as c_int {
            return;
        }
        if !(*paramp).use_ef.is_null() {
            set_option_direct(
                kOptErrorfile,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string((*paramp).use_ef),
                    },
                },
                OptionSetFlags::NONE,
                SID_CARG,
            );
        }
        // The title of the list is the command that would have made it.
        vim_snprintf(
            title.as_mut_ptr(),
            IOSIZE as size_t,
            c"cfile %s".as_ptr(),
            p_ef.get(),
        );
        if qf_init(
            None,
            p_ef.get(),
            p_efm.get(),
            1,
            title.as_mut_ptr(),
            p_menc.get(),
        ) < 0
        {
            msg_putchar('\n' as c_int);
            os_exit(3);
        }
        time_msg_at(c"reading errorfile");
    }
}

/// `-t`: jump to a tag instead of opening a file.
pub(crate) unsafe fn handle_tag(tagname: *mut c_char) {
    let mut cmd = [0 as c_char; IOSIZE as usize];
    // SAFETY: `tagname`, when non-null, points into argv.
    unsafe {
        if tagname.is_null() {
            return;
        }
        swap_exists_did_quit.set(false);
        vim_snprintf(
            cmd.as_mut_ptr(),
            IOSIZE as size_t,
            c"ta %s".as_ptr(),
            tagname,
        );
        do_cmdline_cmd(cmd.as_mut_ptr());
        time_msg_at(c"jumping to tag");
        if swap_exists_did_quit.get() {
            quit_on_swap_exists(false);
        }
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
    unsafe {
        // Use a dialog for the ATTENTION prompt, not a message.
        swap_exists_action.set(SEA_DIALOG);
        no_wait_return.set(1);
        let save_msg_didany = msg_didany.get();

        if !(*curbuf.get()).b_ffname.is_null() {
            let stdin_buf = buflist_new(ptr::null_mut(), ptr::null_mut(), 0, BLN_LISTED as c_int);
            if stdin_buf.is_null() {
                semsg_c!(c"Failed to create buffer for stdin".as_ptr());
                return;
            }
            let initial_buf_handle: handle_T = (*curbuf.get()).handle;
            set_curbuf(Buf::new(stdin_buf), 0, false);
            readfile(
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                0,
                MAXLNUM as c_int as linenr_T,
                ptr::null_mut::<exarg_T>(),
                READ_NEW as c_int + READ_STDIN as c_int,
                true,
            );
            let stdin_buf_handle: handle_T = (*stdin_buf).handle;
            let stdin_buf_empty = buf_is_empty(curbuf.get());

            // Done as commands rather than calls so the autocommands and the
            // window bookkeeping happen as they would for the user.
            let mut cmd: [c_char; 100] = [0; 100];
            vim_snprintf(
                cmd.as_mut_ptr(),
                size_of::<[c_char; 100]>(),
                c"silent! buffer %d".as_ptr(),
                initial_buf_handle,
            );
            do_cmdline_cmd(cmd.as_mut_ptr());
            if stdin_buf_empty {
                vim_snprintf(
                    cmd.as_mut_ptr(),
                    size_of::<[c_char; 100]>(),
                    c"silent! bwipeout! %d".as_ptr(),
                    stdin_buf_handle,
                );
                do_cmdline_cmd(cmd.as_mut_ptr());
            }
        } else {
            set_buflisted(1);
            open_buffer(true, ptr::null_mut::<exarg_T>(), 0);
            if buf_is_empty(curbuf.get()) && Buf::current().b_next.is_some() {
                do_cmdline_cmd(c"silent! bnext".as_ptr());
                do_cmdline_cmd(c"silent! bwipeout 1".as_ptr());
            }
        }

        no_wait_return.set(0);
        msg_didany.set(save_msg_didany);
        time_msg_at(c"reading stdin");
        check_swap_exists_action();
    }
}

/// How many times the "open a buffer for every window" loop below may start
/// over before giving up. An autocommand that keeps splitting would otherwise
/// keep it going forever.
const MAX_WINDOW_PASSES: c_int = 1000;

/// Make the windows and tab pages the command line asked for, and give every
/// one of them a buffer.
pub(crate) unsafe fn create_windows(parmp: *mut mparm_T) {
    // SAFETY: `parmp` is the caller's live parameter block; the window and
    // buffer lists are global and may be rearranged by the autocommands the
    // buffer loading fires.
    unsafe {
        if (*parmp).window_count == -1 {
            // Not set: one window.
            (*parmp).window_count = 1;
        }
        if (*parmp).window_count == 0 {
            // `-o`/`-O`/`-p` with no count: one per file.
            (*parmp).window_count = (*global_arglist()).al_ga.ga_len;
        }
        if (*parmp).window_count > 1 {
            // Leave the layout alone if a vimrc command already split it.
            if (*parmp).window_layout == 0 {
                (*parmp).window_layout = WIN_HOR as c_int;
            }
            if (*parmp).window_layout == WIN_TABS as c_int {
                (*parmp).window_count = make_tabpages((*parmp).window_count);
                time_msg_at(c"making tab pages");
            } else if (*firstwin.get()).w_next.is_null() || (*(*firstwin.get()).w_next).w_floating {
                (*parmp).window_count = make_windows(
                    (*parmp).window_count,
                    (*parmp).window_layout == WIN_VER as c_int,
                );
                time_msg_at(c"making windows");
            } else {
                (*parmp).window_count = win_count();
            }
        } else {
            (*parmp).window_count = 1;
        }

        if recoverymode.get() {
            msg_scroll.set(1);
            ml_recover(true);
            if (*curbuf.get()).b_ml.ml_mfp.is_null() {
                // Recovery failed; there is nothing to edit.
                getout(1);
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
                if (*parmp).window_layout == WIN_TABS as c_int {
                    goto_tabpage(1);
                } else {
                    curwin.set(firstwin.get());
                }
            } else if (*parmp).window_layout == WIN_TABS as c_int {
                if (*curtab.get()).tp_next.is_null() {
                    break;
                }
                goto_tabpage(0);
            } else {
                if (*curwin.get()).w_next.is_null() {
                    break;
                }
                curwin.set((*curwin.get()).w_next);
            }
            dorewind = false;
            curbuf.set((*curwin.get()).w_buffer);

            if (*curbuf.get()).b_ml.ml_mfp.is_null() {
                if p_fdls.get() >= 0 as OptInt {
                    (*curwin.get()).w_onebuf_opt.wo_fdl = p_fdls.get();
                }
                // Ask, rather than print, if the swap file is in the way.
                swap_exists_action.set(SEA_DIALOG);
                set_buflisted(1);
                open_buffer(false, ptr::null_mut::<exarg_T>(), 0);

                if swap_exists_action.get() == SEA_QUIT {
                    if got_int.get() || only_one_window() {
                        quit_on_swap_exists(true);
                    }
                    // The window cannot be closed here without disturbing
                    // what comes next: clear the name and mark the argument
                    // index so it is deleted later.
                    setfname(Buf::current(), ptr::null_mut(), ptr::null_mut(), false);
                    (*curwin.get()).w_arg_idx = -1;
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

        if (*parmp).window_layout == WIN_TABS as c_int {
            goto_tabpage(1);
        } else {
            curwin.set(firstwin.get());
        }
        curbuf.set((*curwin.get()).w_buffer);
        autocmd_no_enter.set(autocmd_no_enter.get() - 1);
        autocmd_no_leave.set(autocmd_no_leave.get() - 1);
    }
}

/// Load the remaining file arguments into the windows [`create_windows`]
/// made, and leave the cursor in the first non-preview one.
pub(crate) unsafe fn edit_buffers(parmp: *mut mparm_T) {
    // SAFETY: `parmp` is the caller's live parameter block; the window list
    // is global and `do_ecmd` may close windows through autocommands.
    unsafe {
        autocmd_no_enter.set(autocmd_no_enter.get() + 1);
        autocmd_no_leave.set(autocmd_no_leave.get() + 1);

        // `create_windows` marks a window whose file could not be opened.
        let mut advance = true;
        if (*curwin.get()).w_arg_idx == -1 {
            win_close(curwin.get(), true, false);
            advance = false;
        }

        // 'shortmess' with F added, saved for restoring after the tab pages
        // are filled: the per-file messages are noise when there are many.
        let mut p_shm_save: *mut c_char = ptr::null_mut();

        let mut arg_idx: c_int = 1;
        for i in 1..(*parmp).window_count {
            if (*curwin.get()).w_arg_idx == -1 {
                arg_idx += 1;
                win_close(curwin.get(), true, false);
                advance = false;
                continue;
            }

            if advance {
                if (*parmp).window_layout == WIN_TABS as c_int {
                    if (*curtab.get()).tp_next.is_null() {
                        break;
                    }
                    goto_tabpage(0);
                    if i == 1 {
                        p_shm_save = xstrdup(p_shm.get());
                        let mut shm: [c_char; 100] = [0; 100];
                        snprintf(
                            shm.as_mut_ptr(),
                            size_of::<[c_char; 100]>(),
                            c"F%s".as_ptr(),
                            p_shm.get(),
                        );
                        set_shortmess(shm.as_mut_ptr());
                    }
                } else {
                    if (*curwin.get()).w_next.is_null() {
                        break;
                    }
                    win_enter((*curwin.get()).w_next, false);
                }
            }
            advance = true;

            // Only load a file into a window that is still showing the first
            // window's buffer, or an unnamed one.
            if curbuf.get() == (*firstwin.get()).w_buffer || (*curbuf.get()).b_ffname.is_null() {
                (*curwin.get()).w_arg_idx = arg_idx;
                swap_exists_did_quit.set(false);
                let alist = global_arglist();
                do_ecmd(
                    0,
                    if arg_idx < (*alist).al_ga.ga_len {
                        alist_name(
                            ((*alist).al_ga.ga_data as *mut aentry_T).offset(arg_idx as isize),
                        )
                    } else {
                        ptr::null_mut()
                    },
                    ptr::null_mut(),
                    ptr::null_mut::<exarg_T>(),
                    ECMD_LASTL as c_int as linenr_T,
                    ECMD_HIDE as c_int,
                    curwin.get(),
                );
                if swap_exists_did_quit.get() {
                    if got_int.get() || only_one_window() {
                        quit_on_swap_exists(true);
                    }
                    win_close(curwin.get(), true, false);
                    advance = false;
                }
                if arg_idx == (*alist).al_ga.ga_len - 1 {
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
            set_shortmess(p_shm_save);
            xfree(p_shm_save as *mut c_void);
        }

        if (*parmp).window_layout == WIN_TABS as c_int {
            goto_tabpage(1);
        }
        autocmd_no_enter.set(autocmd_no_enter.get() - 1);

        // Start in the first window that is not a preview.
        let mut win = firstwin.get();
        while (*win).w_onebuf_opt.wo_pvw != 0 {
            win = (*win).w_next;
            if win.is_null() {
                win = firstwin.get();
                break;
            }
        }
        win_enter(win, false);
        autocmd_no_leave.set(autocmd_no_leave.get() - 1);

        time_msg_at(c"editing files in windows");
        if (*parmp).window_count > 1 && (*parmp).window_layout != WIN_TABS as c_int {
            win_equal(curwin.get(), false, 'b' as c_int);
        }
    }
}

/// Set 'shortmess' to `value`, reporting an error the way `:set` would.
unsafe fn set_shortmess(value: *mut c_char) {
    // SAFETY: `value` is a NUL-terminated string that outlives the call; the
    // option layer copies it.
    unsafe {
        set_option_value_give_err(
            kOptShortmess,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(value),
                },
            },
            OptionSetFlags::NONE,
        );
    }
}

/// Act on the ATTENTION prompt's answer after a buffer was loaded.
pub(crate) unsafe fn check_swap_exists_action() {
    if swap_exists_action.get() == SEA_QUIT {
        quit_on_swap_exists(false);
    }
    handle_swap_exists(None);
}
