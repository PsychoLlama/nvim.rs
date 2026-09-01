//! The `:diff*` commands that turn diff mode on and off.
//!
//! `:diffthis`, `:diffsplit`, `:diffoff` and `:diffpatch`, plus
//! [`diff_win_options`], which is the option set every window entering diff
//! mode takes (and, through the saved `w_p_*_save` fields, gives back on the
//! way out).
//!
//! The save/restore pair is not symmetric, and both halves say why at their
//! site: the save happens **once**, guarded by `wo_diff_saved`, so a second
//! `:diffthis` cannot record the diff-mode values as the ones to go back to;
//! and the restore only applies where the window still holds the value diff
//! mode gave it, so an option the user changed in between survives
//! `:diffoff`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::buffer::BufRef;
use crate::cstr;
use crate::ex_docmd::cmdmod_set_tab;
use crate::option::boolean_optval;
use crate::os::cshim::gettext_ptr;
use crate::types::{Failed, MAXPATHL, NUL, OptionSetFlags};
use crate::winlayer::{Buf, Live, TabPage, Win, windows};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// Release one of this module's owned strings; a null is fine, as `xfree`'s
/// own contract says.
fn free_str(p: *mut c_char) {
    // SAFETY: this module's own allocation, or null.
    unsafe { xfree(p.cast::<c_void>()) };
}

/// Delete the file `path` names, if there is one, and release the name.
fn remove_and_free(path: *mut c_char) {
    if !path.is_null() {
        // SAFETY: one of this module's own temp file names.
        unsafe { os_remove(path) };
    }
    free_str(path);
}

/// `emsg(gettext(msg))`, the pair every error here is reported through.
fn emsg_gettext(msg: *const c_char) {
    // SAFETY: a static message string, and the editor exists.
    unsafe { emsg(gettext_ptr(msg)) };
}

/// `:diffpatch {file}`: apply a patch to a copy of the current buffer and
/// open the result beside it.
///
/// The patch runs in the temp directory, because `patch(1)` writes its
/// `.orig` and `.rej` files next to its output and the user's directory is
/// not the place for them; the original directory is restored afterwards.
/// `'patchexpr'` replaces the shell-out entirely.
///
/// # Safety
/// `eap` must be a live command.
pub unsafe fn ex_diffpatch(eap: *mut exarg_T) {
    // SAFETY: the caller's command.
    let mut eap = unsafe { Live::<exarg_T>::new(eap) };
    let old_curwin: *mut win_T = curwin.get();
    let mut newname: *mut c_char = ptr::null_mut();
    let mut esc_name: *mut c_char = ptr::null_mut();
    let mut fullname: *mut c_char = ptr::null_mut();
    let mut buf: *mut c_char = ptr::null_mut();
    // SAFETY: the editor exists, for both names.
    let (tmp_orig, tmp_new) = unsafe { (vim_tempname(), vim_tempname()) };

    if !(tmp_orig.is_null() || tmp_new.is_null()) && write_orig(tmp_orig).is_ok() {
        // SAFETY: `eap.arg` is the command's own argument string.
        fullname = unsafe { full_name_save(eap.arg, false) };
        let name = if fullname.is_null() {
            eap.arg
        } else {
            fullname
        };
        // SAFETY: a NUL-terminated file name.
        esc_name = unsafe { vim_strsave_shellescape(name, true, true) };
        // SAFETY: three NUL-terminated strings.
        let orig_len = unsafe { cstr::bytes_at(tmp_orig) }.len();
        let name_len = unsafe { cstr::bytes_at(esc_name) }.len();
        let new_len = unsafe { cstr::bytes_at(tmp_new) }.len();
        let buflen = orig_len + name_len + new_len + 16;
        // SAFETY: `xmalloc` aborts rather than answer null.
        buf = unsafe { xmalloc(buflen) }.cast::<c_char>();

        // Run the patch from the temp directory, so its `.orig`/`.rej`
        // droppings do not land in the user's.
        let mut dirbuf: [c_char; 4096] = [0; 4096];
        let saved_dir = save_cwd(&mut dirbuf);
        if saved_dir {
            // SAFETY: the editor's own temp directory, else the fallback.
            let tempdir = unsafe { vim_gettempdir() };
            let tempdir = if tempdir.is_null() {
                c"/tmp".as_ptr() as *mut c_char
            } else {
                tempdir
            };
            // SAFETY: a NUL-terminated directory name; the editor exists.
            unsafe { os_chdir(tempdir) };
            unsafe { shorten_fnames(1) };
        }

        // SAFETY: `p_pex` is the `'patchexpr'` option string.
        if unsafe { *p_pex.get() } as c_int != NUL {
            // SAFETY: three NUL-terminated file names.
            unsafe { eval_patch(tmp_orig, name, tmp_new) };
        } else {
            let fmt = c"patch -o %s %s < %s".as_ptr();
            // SAFETY: `buf` holds `buflen` bytes, and the three `%s` are
            // matched by the three NUL-terminated names.
            unsafe { vim_snprintf(buf, buflen, fmt, tmp_new, tmp_orig, esc_name) };
            // SAFETY: the editor exists, in all three calls.
            unsafe { block_autocmds() };
            unsafe { call_shell(buf, ShellOpts::FILTER, ptr::null_mut::<c_char>()) };
            unsafe { unblock_autocmds() };
        }

        if saved_dir {
            // SAFETY: the directory name `save_cwd` filled in.
            if unsafe { os_chdir(dirbuf.as_mut_ptr()) } != 0 {
                emsg_gettext(e_prev_dir.as_ptr());
            }
            // SAFETY: the editor exists.
            unsafe { shorten_fnames(1) };
        }
        remove_suffixed(buf, tmp_new, c".orig".as_ptr());
        remove_suffixed(buf, tmp_new, c".rej".as_ptr());

        let mut file_info = FileInfo::default();
        // SAFETY: a NUL-terminated file name and a live `FileInfo`.
        let info_ok = unsafe { os_fileinfo(tmp_new, &raw mut file_info) };
        // SAFETY: as above.
        let filesize = unsafe { os_fileinfo_size(&raw mut file_info) };
        if !info_ok || filesize == 0 {
            emsg_gettext(c"E816: Cannot read patch output".as_ptr());
        } else {
            if !cur_buf().b_fname.is_null() {
                let fname = cur_buf().b_fname;
                // SAFETY: the buffer's own file name, NUL-terminated; the
                // four extra bytes are for the `.new` appended next.
                newname = unsafe { xstrnsave(fname, cstr::bytes_at(fname).len() + 4) };
                unsafe { strcat(newname, c".new".as_ptr()) };
            }
            cmdmod_set_tab(0);
            let vertical = diff_flags.get() & DIFF_VERTICAL != 0;
            let flags = if vertical { WSP_VERT as c_int } else { 0 };
            if win_split(0, flags).is_ok() {
                eap.cmdidx = CMD_split;
                eap.arg = tmp_new;
                // SAFETY: the caller's command, and a window that was live
                // when it was read.
                unsafe { do_exedit(eap.raw(), old_curwin) };
                // SAFETY: `win_valid` takes any pointer and compares it
                // against the live window list.
                if curwin.get() != old_curwin && win_valid(old_curwin) {
                    // SAFETY: both windows are live, as just checked.
                    diff_win_options(cur_win(), true);
                    diff_win_options(unsafe { Win::new(old_curwin) }, true);
                    if !newname.is_null() {
                        eap.arg = newname;
                        // SAFETY: the caller's command; the group name and
                        // the command line are static strings.
                        unsafe { ex_file(eap.raw()) };
                        if unsafe { augroup_exists(c"filetypedetect".as_ptr()) } {
                            let _ =
                                unsafe { do_cmdline_cmd(c":doau filetypedetect BufRead".as_ptr()) };
                        }
                    }
                }
            }
        }
    }
    remove_and_free(tmp_orig);
    remove_and_free(tmp_new);
    free_str(newname);
    free_str(buf);
    free_str(fullname);
    free_str(esc_name);
}

/// Write the current buffer out to `tmp_orig`, the patch's input.
fn write_orig(tmp_orig: *mut c_char) -> Result<(), Failed> {
    let cb = curbuf.get();
    let end = cur_buf().b_ml.ml_line_count;
    let req = WriteRequest::filter();
    // SAFETY: the current buffer is live and the name is our own temp file;
    // no shortname and no `exarg_T` are wanted.
    unsafe { buf_write(cb, tmp_orig, ptr::null_mut(), 1, end, ptr::null_mut(), req) }
}

/// Record the working directory into `dirbuf`, answering whether it can be
/// changed back later.
///
/// A directory that cannot be named, or cannot be re-entered right now, is
/// recorded as the empty string and the caller leaves the cwd alone.
fn save_cwd(dirbuf: &mut [c_char; 4096]) -> bool {
    let at = dirbuf.as_mut_ptr();
    // SAFETY: `dirbuf` holds `MAXPATHL` bytes, which is what `os_dirname` is
    // told, and `os_chdir` gets the NUL-terminated name it wrote there.
    let ok = unsafe { os_dirname(at, MAXPATHL as size_t).is_ok() && os_chdir(at) == 0 };
    if !ok {
        dirbuf[0] = NUL as c_char;
    }
    ok
}

/// Delete `name` with `suffix` glued on, using `buf` as the scratch space --
/// `patch(1)`'s `.orig` and `.rej` leftovers.
fn remove_suffixed(buf: *mut c_char, name: *mut c_char, suffix: *const c_char) {
    // SAFETY: `buf` was sized for `name` plus the longest suffix, and both
    // inputs are NUL-terminated.
    unsafe { strcpy(buf, name) };
    unsafe { strcat(buf, suffix) };
    unsafe { os_remove(buf) };
}

/// `:diffsplit {file}`: open `file` in a new window and diff it against the
/// current buffer.
///
/// # Safety
/// `eap` must be a live command.
pub unsafe fn ex_diffsplit(eap: *mut exarg_T) {
    // SAFETY: the caller's command.
    let mut eap = unsafe { Live::<exarg_T>::new(eap) };
    let old_curwin: *mut win_T = curwin.get();
    let old_curbuf = BufRef::of_opt(current_buf());
    // SAFETY: the current window is live, in both calls.
    validate_cursor(unsafe { Win::new(old_curwin) });
    unsafe { set_fraction(old_curwin) };
    cmdmod_set_tab(0);
    let vertical = diff_flags.get() & DIFF_VERTICAL != 0;
    let flags = if vertical { WSP_VERT as c_int } else { 0 };
    if win_split(0, flags).is_err() {
        return;
    }
    eap.cmdidx = CMD_split;
    cur_win().w_onebuf_opt.wo_diff = 1;
    // SAFETY: the caller's command, and a window that was live when read.
    unsafe { do_exedit(eap.raw(), old_curwin) };
    if curwin.get() == old_curwin {
        return;
    }
    // SAFETY: the current window is live.
    diff_win_options(cur_win(), true);
    // SAFETY: `win_valid` compares against the live window list.
    if win_valid(old_curwin) {
        // SAFETY: the window is live, as just checked.
        diff_win_options(unsafe { Win::new(old_curwin) }, true);
        if let Some(old_buf) = old_curbuf.get() {
            // SAFETY: the old window is live and its buffer reference valid.
            let lnum = unsafe { diff_get_corresponding_line(old_buf, (*old_curwin).w_cursor.lnum) };
            cur_win().w_cursor.lnum = lnum;
        }
    }
    let height = cur_win().w_height;
    // SAFETY: the current window is live.
    unsafe { scroll_to_fraction(curwin.get(), height) };
}

/// `:diffthis`: put the current window in diff mode.
///
/// # Safety
/// The editor must be running.
pub unsafe fn ex_diffthis(_eap: *mut exarg_T) {
    // SAFETY: the current window is live.
    diff_win_options(cur_win(), true);
}

/// Set `'diff'` in `wp` without letting the option's side effects run.
///
/// `curwin` is moved to `wp` for the call because the option code reads it,
/// and `diff_buf_adjust` is suppressed so that the caller stays in charge of
/// the buffer registry.
fn set_diff_option(wp: Win, value: bool) {
    let old_curwin = curwin.get();
    curwin.set(wp.raw());
    curbuf.set(cur_win().w_buffer);
    cur_buf().b_ro_locked += 1;
    // `curwin`/`curbuf` name `wp` and its buffer, which is what the option
    // code reads; the buffer is locked against a `:set` side effect.
    let val = boolean_optval(Some(value));
    set_option_value_give_err(kOptDiff, val, OptionSetFlags::LOCAL);
    cur_buf().b_ro_locked -= 1;
    curwin.set(old_curwin);
    curbuf.set(cur_win().w_buffer);
}

/// Put `wp` into diff mode: the option set, and optionally its buffer.
///
/// Every option this changes is saved into the matching `w_p_*_save` field
/// first, but **only on the first call** -- `wo_diff_saved` is what stops a
/// second `:diffthis` from saving the diff-mode values as the ones to
/// restore.
///
/// Safe: a [`Win`] carries the whole of the promise this needs.
pub fn diff_win_options(mut wp: Win, addbuf: bool) {
    let old_curwin = curwin.get();
    curwin.set(wp.raw());
    // SAFETY: `curwin` is `wp`, which is live.
    unsafe { new_fold_level() };
    curwin.set(old_curwin);

    // Each option is saved only while the window is not already in diff
    // mode, so a second `:diffthis` cannot overwrite the saved values.
    let first_time = wp.w_onebuf_opt.wo_diff == 0;
    if first_time {
        wp.w_onebuf_opt.wo_scb_save = wp.w_onebuf_opt.wo_scb;
    }
    wp.w_onebuf_opt.wo_scb = 1;
    if first_time {
        wp.w_onebuf_opt.wo_crb_save = wp.w_onebuf_opt.wo_crb;
    }
    wp.w_onebuf_opt.wo_crb = 1;
    if diff_flags.get() & DIFF_FOLLOWWRAP == 0 {
        if first_time {
            wp.w_onebuf_opt.wo_wrap_save = wp.w_onebuf_opt.wo_wrap;
        }
        wp.w_onebuf_opt.wo_wrap = 0;
        wp.w_skipcol = 0 as colnr_T;
    }
    if first_time {
        if wp.w_onebuf_opt.wo_diff_saved != 0 {
            free_string_option_of(wp.w_onebuf_opt.wo_fdm_save);
        }
        wp.w_onebuf_opt.wo_fdm_save = strdup_of(wp.w_onebuf_opt.wo_fdm);
    }
    let foldmethod = OptVal::String(String_0::from_raw_parts(c"diff".as_ptr() as *mut c_char, 4));
    let scope = OptionSetFlags::LOCAL;
    // SAFETY: a live window as the option's scope, and a static string as
    // its value.
    unsafe {
        set_option_direct_for(
            kOptFoldmethod,
            foldmethod,
            scope,
            0 as scid_T,
            kOptScopeWin,
            wp.raw().cast::<c_void>(),
        )
    };
    if first_time {
        wp.w_onebuf_opt.wo_fen_save = wp.w_onebuf_opt.wo_fen;
        wp.w_onebuf_opt.wo_fdl_save = wp.w_onebuf_opt.wo_fdl;
        if wp.w_onebuf_opt.wo_diff_saved != 0 {
            free_string_option_of(wp.w_onebuf_opt.wo_fdc_save);
        }
        wp.w_onebuf_opt.wo_fdc_save = strdup_of(wp.w_onebuf_opt.wo_fdc);
    }
    free_string_option_of(wp.w_onebuf_opt.wo_fdc);
    wp.w_onebuf_opt.wo_fdc = strdup_of(c"2".as_ptr());
    // A single digit, because the option's buffer is one byte plus the
    // NUL. C's `assert()` is `debug_assert!`: it vanishes under NDEBUG.
    debug_assert!((0..=9).contains(&diff_foldcolumn.get()));
    let fdc = wp.w_onebuf_opt.wo_fdc;
    let width = diff_foldcolumn.get();
    // SAFETY: `fdc` is the one-digit string just allocated, and `strlen + 1`
    // is exactly the room it has.
    unsafe { snprintf(fdc, cstr::bytes_at(fdc).len() + 1, c"%d".as_ptr(), width) };
    wp.w_onebuf_opt.wo_fen = 1;
    wp.w_onebuf_opt.wo_fdl = 0 as OptInt;
    // SAFETY: a live window, in all three calls.
    fold_update_all(wp);
    changed_window_setting(wp);
    if unsafe { vim_strchr(p_sbo.get(), 'h' as c_int) }.is_null() {
        let _ = unsafe { do_cmdline_cmd(c"set sbo+=hor".as_ptr()) };
    }
    wp.w_onebuf_opt.wo_diff_saved = 1;
    set_diff_option(wp, true);
    if addbuf {
        // SAFETY: a live window's buffer is live.
        diff_buf_add(wp.buffer());
    }
    wp.redraw_later(UPD_NOT_VALID);
}

/// `free_string_option`, for one of the window's own option strings.
fn free_string_option_of(p: *mut c_char) {
    // SAFETY: an option string the option code itself allocated, or null.
    unsafe { free_string_option(p) };
}

/// `xstrdup`, for a NUL-terminated option string.
fn strdup_of(p: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated string; `xstrdup` aborts rather than fail.
    unsafe { xstrdup(p) }
}

/// `:diffoff[!]`: leave diff mode in this window, or with `!` in every
/// window of the tabpage.
///
/// Each option goes back to its `w_p_*_save` value, but only where the
/// window still holds the value diff mode gave it: a value the user changed
/// in the meantime is left alone.
///
/// # Safety
/// `eap` must be a live command.
pub unsafe fn ex_diffoff(eap: *mut exarg_T) {
    // SAFETY: the caller's command.
    let eap = unsafe { Live::<exarg_T>::new(eap) };
    let mut diffwin = false;
    // `FOR_ALL_WINDOWS_IN_TAB(wp, curtab)`: always the `firstwin` list.
    for mut wp in windows() {
        let wanted = if eap.forceit != 0 {
            wp.w_onebuf_opt.wo_diff != 0
        } else {
            wp.is_current()
        };
        if wanted {
            set_diff_option(wp, false);
            if wp.w_onebuf_opt.wo_diff_saved != 0 {
                if wp.w_onebuf_opt.wo_scb != 0 {
                    wp.w_onebuf_opt.wo_scb = wp.w_onebuf_opt.wo_scb_save;
                }
                if wp.w_onebuf_opt.wo_crb != 0 {
                    wp.w_onebuf_opt.wo_crb = wp.w_onebuf_opt.wo_crb_save;
                }
                if diff_flags.get() & DIFF_FOLLOWWRAP == 0
                    && wp.w_onebuf_opt.wo_wrap == 0
                    && wp.w_onebuf_opt.wo_wrap_save != 0
                {
                    wp.w_onebuf_opt.wo_wrap = 1;
                    wp.w_leftcol = 0 as colnr_T;
                }
                free_string_option_of(wp.w_onebuf_opt.wo_fdm);
                wp.w_onebuf_opt.wo_fdm =
                    strdup_of(saved_or(wp.w_onebuf_opt.wo_fdm_save, c"manual".as_ptr()));
                free_string_option_of(wp.w_onebuf_opt.wo_fdc);
                wp.w_onebuf_opt.wo_fdc =
                    strdup_of(saved_or(wp.w_onebuf_opt.wo_fdc_save, c"0".as_ptr()));
                if wp.w_onebuf_opt.wo_fdl == 0 as OptInt {
                    wp.w_onebuf_opt.wo_fdl = wp.w_onebuf_opt.wo_fdl_save;
                }
                if wp.w_onebuf_opt.wo_fen != 0 {
                    // SAFETY: a live window.
                    let manual = foldmethod_is_manual(wp);
                    wp.w_onebuf_opt.wo_fen = if manual {
                        0
                    } else {
                        wp.w_onebuf_opt.wo_fen_save
                    };
                }
                // SAFETY: a live window.
                fold_update_all(wp);
            }
            wp.w_topfill = 0;
            changed_window_setting(wp);
            diff_buf_adjust(wp);
        }
        diffwin = diffwin || wp.w_onebuf_opt.wo_diff != 0;
    }
    if eap.forceit != 0 {
        diff_buf_clear();
    }
    let mut tp = cur_tab();
    if !diffwin {
        diff_need_update.set(false);
        tp.tp_diff_invalid = 0;
        tp.tp_diff_update = 0;
        // SAFETY: the current tab page is live.
        diff_clear(tp);
    }
    // SAFETY: `p_sbo` is the `'scrollopt'` option string.
    if !diffwin && !unsafe { vim_strchr(p_sbo.get(), 'h' as c_int) }.is_null() {
        // SAFETY: a static command line.
        let _ = unsafe { do_cmdline_cmd(c"set sbo-=hor".as_ptr()) };
    }
}

/// The saved option string, or `fallback` when nothing was saved.
///
/// Upstream tests the saved value's *first byte*: an empty saved string means
/// the option was never really recorded.
fn saved_or(saved: *mut c_char, fallback: *const c_char) -> *const c_char {
    // SAFETY: a NUL-terminated option string the option code allocated.
    if unsafe { *saved } as c_int != 0 {
        saved.cast_const()
    } else {
        fallback
    }
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

/// The tab page the editor is working in.
fn cur_tab() -> TabPage {
    // SAFETY: `curtab` is set from startup to exit.
    unsafe { TabPage::current() }
}
