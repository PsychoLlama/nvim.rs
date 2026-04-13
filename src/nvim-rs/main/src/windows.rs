//! Window and buffer creation during startup
//!
//! Implements `rs_create_windows` and `rs_edit_buffers` replacing static
//! C functions in main.c.

use crate::setup::MparmT;
use std::ffi::{c_char, c_int};

// Values for window_layout (must match C enums in main.c)
const WIN_HOR: c_int = 1;
const WIN_VER: c_int = 2;
const WIN_TABS: c_int = 3;

// SEA_* values (from globals.h)
const SEA_NONE: c_int = 0;
const SEA_DIALOG: c_int = 1;
const SEA_QUIT: c_int = 2;

// ECMD_LASTL = 0, ECMD_HIDE = 4 (from ex_cmds.h)
const ECMD_LASTL: c_int = 0;
const ECMD_HIDE: c_int = 4;

unsafe extern "C" {
    fn nvim_al_GARGCOUNT() -> c_int;
    fn make_tabpages(maxcount: c_int) -> c_int;
    fn make_windows(count: c_int, vertical: bool) -> c_int;
    fn rs_win_count() -> c_int;
    fn goto_tabpage(n: c_int);
    fn ml_recover(checkext: bool);
    fn getout(exitval: c_int) -> !;
    fn do_modelines(flags: c_int);
    fn open_buffer(read_stdin: bool, eap: *mut std::ffi::c_void, flags_arg: c_int) -> c_int;
    fn rs_set_buflisted(on: c_int);
    fn handle_swap_exists(old_curbuf: *mut std::ffi::c_void);
    fn os_breakcheck();
    fn vgetc() -> c_int;
    fn rs_only_one_window() -> c_int;
    fn ui_call_error_exit(status: c_int);
    fn rs_win_equal(next_curwin: *mut std::ffi::c_void, current: c_int, dir: c_int);

    // edit_buffers extras
    fn win_close(win: *mut std::ffi::c_void, free_buf: bool, abort_if_last: bool);
    fn win_enter(win: *mut std::ffi::c_void, undo_sync: bool);
    fn do_ecmd(
        fnum: c_int,
        ffname: *mut c_char,
        sfname: *mut c_char,
        eap: *mut std::ffi::c_void,
        newlnum: c_int,
        flags: c_int,
        curwin: *mut std::ffi::c_void,
    ) -> c_int;
    fn os_chdir(path: *const c_char) -> c_int;
    // C helper: calls set_option_value_give_err(kOptShortmess, CSTR_AS_OPTVAL(val), 0)
    fn nvim_set_shortmess_opt(val: *const c_char);
    fn xstrdup(str: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut std::ffi::c_void);
    fn snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
    fn nvim_garglist_name(idx: c_int) -> *mut c_char;

    // Window/tabpage accessors (added to window_shim.c Phase 3+4)
    fn nvim_firstwin_next_null_or_floating() -> c_int;
    fn nvim_set_curwin_to_firstwin();
    fn nvim_curtab_get_tp_next_null() -> c_int;
    fn nvim_advance_curwin_to_next();
    fn nvim_set_curbuf_from_curwin();
    fn nvim_curbuf_get_ml_mfp_null() -> c_int;
    fn nvim_get_p_fdls() -> i64;
    fn nvim_curwin_set_p_fdl(val: c_int);
    fn nvim_get_curwin() -> nvim_window::WinHandle;
    fn nvim_curbuf_setfname_null();
    fn nvim_get_curwin_ptr() -> *mut std::ffi::c_void;
    fn nvim_get_firstwin_ptr() -> *mut std::ffi::c_void;
    fn nvim_curwin_get_next() -> *mut std::ffi::c_void;
    fn nvim_firstwin_get_buffer() -> *mut std::ffi::c_void;
    fn nvim_get_curbuf() -> *mut std::ffi::c_void;
    fn nvim_curbuf_has_ffname() -> c_int;
    fn nvim_win_get_p_pvw(win: *mut std::ffi::c_void) -> c_int;
    fn nvim_win_get_next_in_tab(win: *mut std::ffi::c_void) -> *mut std::ffi::c_void;

    // Globals
    static mut recoverymode: bool;
    static mut swap_exists_action: c_int;
    static mut got_int: bool;
    static mut did_emsg: c_int;
    static mut autocmd_no_enter: c_int;
    static mut autocmd_no_leave: c_int;
    static mut swap_exists_did_quit: bool;
    static mut arg_had_last: bool;
    static mut p_shm: *mut c_char;
}

/// Create windows during startup.
///
/// # Safety
/// `parmp` must be a valid pointer to an mparm_T.
#[no_mangle]
pub unsafe extern "C" fn rs_create_windows(parmp: *mut MparmT) {
    let p = &mut *parmp;

    // Determine window count
    if p.window_count == -1 {
        p.window_count = 1;
    }
    if p.window_count == 0 {
        p.window_count = nvim_al_GARGCOUNT();
    }
    if p.window_count > 1 {
        if p.window_layout == 0 {
            p.window_layout = WIN_HOR;
        }
        if p.window_layout == WIN_TABS {
            p.window_count = make_tabpages(p.window_count);
        } else if nvim_firstwin_next_null_or_floating() != 0 {
            p.window_count = make_windows(p.window_count, p.window_layout == WIN_VER);
        } else {
            p.window_count = rs_win_count();
        }
    } else {
        p.window_count = 1;
    }

    if recoverymode {
        // do recover
        ml_recover(true);
        if nvim_curbuf_get_ml_mfp_null() != 0 {
            // failed
            getout(1);
        }
        do_modelines(0);
    } else {
        let mut done = 0;
        autocmd_no_enter += 1;
        autocmd_no_leave += 1;
        let mut dorewind = true;
        while done < 1000 {
            done += 1;
            if dorewind {
                if p.window_layout == WIN_TABS {
                    goto_tabpage(1);
                } else {
                    nvim_set_curwin_to_firstwin();
                }
            } else if p.window_layout == WIN_TABS {
                if nvim_curtab_get_tp_next_null() != 0 {
                    break;
                }
                goto_tabpage(0);
            } else {
                if nvim_window::win_struct::win_ref(nvim_get_curwin())
                    .w_next
                    .is_null()
                {
                    break;
                }
                nvim_advance_curwin_to_next();
            }
            dorewind = false;
            nvim_set_curbuf_from_curwin();
            if nvim_curbuf_get_ml_mfp_null() != 0 {
                // Set 'foldlevel' to 'foldlevelstart' if not negative
                let fdls = nvim_get_p_fdls();
                if fdls >= 0 {
                    nvim_curwin_set_p_fdl(fdls as c_int);
                }
                // When getting the ATTENTION prompt here, use a dialog.
                swap_exists_action = SEA_DIALOG;
                rs_set_buflisted(1);
                // create memfile, read file
                open_buffer(false, std::ptr::null_mut(), 0);

                if swap_exists_action == SEA_QUIT {
                    if got_int || rs_only_one_window() != 0 {
                        // abort selected or quit and only one window
                        did_emsg = 0; // avoid hit-enter prompt
                        ui_call_error_exit(1);
                        getout(1);
                    }
                    // We can't close the window, it would disturb what
                    // happens next. Clear the file name and set the arg
                    // index to -1 to delete it later.
                    nvim_curbuf_setfname_null();
                    nvim_window::win_struct::win_mut(nvim_get_curwin()).w_arg_idx = -1;
                    swap_exists_action = SEA_NONE;
                } else {
                    handle_swap_exists(std::ptr::null_mut());
                }
                dorewind = true; // start again
            }
            os_breakcheck();
            if got_int {
                vgetc(); // only break the file loading, not the rest
                break;
            }
        }
        if p.window_layout == WIN_TABS {
            goto_tabpage(1);
        } else {
            nvim_set_curwin_to_firstwin();
        }
        nvim_set_curbuf_from_curwin();
        autocmd_no_enter -= 1;
        autocmd_no_leave -= 1;
    }
}

/// Edit files in multiple windows after creation.
///
/// # Safety
/// `parmp` must be a valid pointer to an mparm_T.
/// `cwd` may be null or a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_edit_buffers(parmp: *mut MparmT, cwd: *mut c_char) {
    let p = &mut *parmp;
    let mut advance = true;
    let mut p_shm_save: *mut c_char = std::ptr::null_mut();

    // Don't execute Win/Buf Enter/Leave autocommands here
    autocmd_no_enter += 1;
    autocmd_no_leave += 1;

    // When w_arg_idx is -1 remove the window (see create_windows()).
    if nvim_window::win_struct::win_ref(nvim_get_curwin()).w_arg_idx == -1 {
        win_close(nvim_get_curwin_ptr(), true, false);
        advance = false;
    }

    let mut arg_idx: c_int = 1;
    let mut i: c_int = 1;
    while i < p.window_count {
        if !cwd.is_null() {
            os_chdir(cwd);
        }
        // When w_arg_idx is -1 remove the window (see create_windows()).
        if nvim_window::win_struct::win_ref(nvim_get_curwin()).w_arg_idx == -1 {
            arg_idx += 1;
            win_close(nvim_get_curwin_ptr(), true, false);
            advance = false;
            i += 1;
            continue;
        }

        if advance {
            if p.window_layout == WIN_TABS {
                if nvim_curtab_get_tp_next_null() != 0 {
                    // just checking
                    break;
                }
                goto_tabpage(0);
                // Temporarily reset 'shm' option to not print fileinfo when
                // loading the other buffers.
                if i == 1 {
                    let mut buf = [0i8; 100];
                    p_shm_save = xstrdup(p_shm);
                    snprintf(buf.as_mut_ptr(), 100, c"F%s".as_ptr(), p_shm);
                    nvim_set_shortmess_opt(buf.as_ptr());
                }
            } else {
                if nvim_window::win_struct::win_ref(nvim_get_curwin())
                    .w_next
                    .is_null()
                {
                    // just checking
                    break;
                }
                win_enter(nvim_curwin_get_next(), false);
            }
        }
        advance = true;

        // Only open the file if there is no file in this window yet.
        let curbuf = nvim_get_curbuf();
        let firstwin_buf = nvim_firstwin_get_buffer();
        if curbuf == firstwin_buf || nvim_curbuf_has_ffname() == 0 {
            nvim_window::win_struct::win_mut(nvim_get_curwin()).w_arg_idx = arg_idx;
            // Edit file from arg list, if there is one.
            swap_exists_did_quit = false;
            let gargcount = nvim_al_GARGCOUNT();
            let fname = if arg_idx < gargcount {
                nvim_garglist_name(arg_idx)
            } else {
                std::ptr::null_mut()
            };
            do_ecmd(
                0,
                fname,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                ECMD_LASTL,
                ECMD_HIDE,
                nvim_get_curwin_ptr(),
            );
            if swap_exists_did_quit {
                // abort or quit selected
                if got_int || rs_only_one_window() != 0 {
                    // abort selected and only one window
                    did_emsg = 0; // avoid hit-enter prompt
                    ui_call_error_exit(1);
                    getout(1);
                }
                win_close(nvim_get_curwin_ptr(), true, false);
                advance = false;
            }
            if arg_idx == gargcount - 1 {
                arg_had_last = true;
            }
            arg_idx += 1;
        }
        os_breakcheck();
        if got_int {
            vgetc(); // only break the file loading, not the rest
            break;
        }
        i += 1;
    }

    if !p_shm_save.is_null() {
        nvim_set_shortmess_opt(p_shm_save);
        xfree(p_shm_save as *mut std::ffi::c_void);
    }

    if p.window_layout == WIN_TABS {
        goto_tabpage(1);
    }
    autocmd_no_enter -= 1;

    // make the first window the current window
    let mut win = nvim_get_firstwin_ptr();
    // Avoid making a preview window the current window.
    loop {
        if nvim_win_get_p_pvw(win) == 0 {
            break;
        }
        win = nvim_win_get_next_in_tab(win);
        if win.is_null() {
            win = nvim_get_firstwin_ptr();
            break;
        }
    }
    win_enter(win, false);

    autocmd_no_leave -= 1;
    // TIME_MSG("editing files in windows") is a no-op in Rust
    if p.window_count > 1 && p.window_layout != WIN_TABS {
        rs_win_equal(nvim_get_curwin_ptr(), 0, c_int::from(b'b'));
    }
}
