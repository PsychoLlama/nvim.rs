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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::buffer::BufRef;
use crate::ex_docmd::cmdmod_set_tab;
use crate::memory::XString;
use crate::option::boolean_optval;
use crate::option::vars::P_PEX;
use crate::os::fs::{current_dir, file_info};
use crate::os::shell::call_shell_text;
use crate::path::full_name_save;
use crate::strings::has_char;
use crate::types::CmdIdx;
use crate::types::{Failed, OptionSetFlags};
use crate::winlayer::graph::{switch_to, switch_window};
use crate::winlayer::{Buf, TabPage, Win, windows};
use core::ffi::{CStr, c_int};

/// `emsg(gettext(msg))`, the pair every error here is reported through.
fn emsg_gettext(msg: &'static CStr) {
    emsg(gettext(msg));
}

/// `:diffpatch {file}`: apply a patch to a copy of the current buffer and
/// open the result beside it.
///
/// The patch runs in the temp directory, because `patch(1)` writes its
/// `.orig` and `.rej` files next to its output and the user's directory is
/// not the place for them; the original directory is restored afterwards.
/// `'patchexpr'` replaces the shell-out entirely.
pub fn ex_diffpatch(excmd: &mut ExArg) {
    let (tmp_orig, tmp_new) = (temp_name(), temp_name());
    if let (Some(orig), Some(new)) = (&tmp_orig, &tmp_new)
        && write_orig(orig.as_cstr()).is_ok()
    {
        patch_into(excmd, orig.as_cstr(), new.as_cstr());
    }
    for name in [&tmp_orig, &tmp_new].into_iter().flatten() {
        os_remove(name.as_cstr());
    }
}

/// [`ex_diffpatch`] once the buffer is written to `tmp_orig`: patch it into
/// `tmp_new` and open that beside the current window.
fn patch_into(excmd: &mut ExArg, tmp_orig: &CStr, tmp_new: &CStr) {
    let old_curwin = Win::current();
    // Owned: the command's line is replaced before the name is done
    // with. A name that cannot be made absolute comes back as itself.
    let fullname = full_name_save(excmd.line.cstr_from(excmd.line.arg), false);
    let esc_name = shellescape(&fullname, true, true);
    // Run the patch from the temp directory, so its `.orig`/`.rej`
    // droppings do not land in the user's.
    let saved_dir = save_cwd();
    if saved_dir.is_some() {
        let tempdir = vim_gettempdir();
        os_chdir(tempdir.as_deref().unwrap_or(c"/tmp"));
        shorten_fnames(true);
    }

    if P_PEX.first_byte() != 0 {
        eval_patch(tmp_orig, fullname.as_cstr(), tmp_new);
    } else {
        let mut command = XString::from_bytes(b"patch -o ");
        command.push_bytes(tmp_new.to_bytes());
        command.push_byte(b' ');
        command.push_bytes(tmp_orig.to_bytes());
        command.push_bytes(b" < ");
        command.push_bytes(&esc_name);
        block_autocmds();
        call_shell_text(command.as_cstr(), ShellOpts::FILTER);
        unblock_autocmds();
    }

    if let Some(dir) = &saved_dir {
        if os_chdir(dir.as_cstr()) != 0 {
            emsg_gettext(e_prev_dir);
        }
        shorten_fnames(true);
    }
    remove_suffixed(tmp_new, c".orig");
    remove_suffixed(tmp_new, c".rej");

    let filesize = file_info(tmp_new).map(|info| info.stat.st_size);
    if filesize.is_none_or(|size| size == 0) {
        emsg_gettext(c"E816: Cannot read patch output");
        return;
    }
    let mut newname = None;
    if !Buf::current().name.is_unnamed() {
        let mut patched = Buf::current()
            .name
            .shown()
            .map_or_else(XString::new, XString::from_cstr);
        patched.push_str(".new");
        newname = Some(patched);
    }
    cmdmod_set_tab(0);
    let vertical = diff_flags.get() & DIFF_VERTICAL != 0;
    let flags = if vertical { WSP_VERT.cast_signed() } else { 0 };
    if win_split(0, flags).is_err() {
        return;
    }
    excmd.cmdidx = CmdIdx::split;
    excmd.set_arg_text(tmp_new.to_bytes());
    do_exedit(excmd, Some(old_curwin.id()));
    if !old_curwin.is_current() && win_valid(old_curwin.id()) {
        diff_win_options(Win::current(), true);
        diff_win_options(old_curwin, true);
        if let Some(patched) = newname.as_mut() {
            excmd.set_arg_text(patched);
            ex_file(excmd);
            if augroup_exists(b"filetypedetect") {
                let _ = do_cmdline_cmd(c":doau filetypedetect BufRead");
            }
        }
    }
}

/// Write the current buffer out to `tmp_orig`, the patch's input.
fn write_orig(tmp_orig: &CStr) -> Result<(), Failed> {
    let cb = Buf::current();
    let end = Buf::current().b_ml.ml_line_count;
    let req = WriteRequest::filter();
    // No shortname and no `ExArg` are wanted.
    buf_write(cb, WriteName::Other(tmp_orig), None, 1, end, None, req)
}

/// The working directory, if it can be named and changed back to later.
///
/// A directory that cannot be named, or cannot be re-entered right now, is
/// `None`, and the caller leaves the cwd alone.
fn save_cwd() -> Option<XString> {
    let dir = current_dir()?;
    (os_chdir(dir.as_cstr()) == 0).then_some(dir)
}

/// Delete `name` with `suffix` glued on -- `patch(1)`'s `.orig` and `.rej`
/// leftovers.
fn remove_suffixed(name: &CStr, suffix: &CStr) {
    let mut suffixed = XString::from_cstr(name);
    suffixed.push_cstr(suffix);
    os_remove(suffixed.as_cstr());
}

/// `:diffsplit {file}`: open `file` in a new window and diff it against the
/// current buffer.
pub fn ex_diffsplit(excmd: &mut ExArg) {
    let old_curwin = Win::current();
    let old_curbuf = BufRef::of_opt(current_buf());
    validate_cursor(old_curwin);
    set_fraction(old_curwin);
    cmdmod_set_tab(0);
    let vertical = diff_flags.get() & DIFF_VERTICAL != 0;
    let flags = if vertical { WSP_VERT.cast_signed() } else { 0 };
    if win_split(0, flags).is_err() {
        return;
    }
    excmd.cmdidx = CmdIdx::split;
    Win::current().w_onebuf_opt.wo_diff = 1;
    do_exedit(excmd, Some(old_curwin.id()));
    if old_curwin.is_current() {
        return;
    }
    diff_win_options(Win::current(), true);
    if win_valid(old_curwin.id()) {
        diff_win_options(old_curwin, true);
        if let Some(old_buf) = old_curbuf.get() {
            let lnum = diff_get_corresponding_line(old_buf, old_curwin.w_cursor.lnum);
            Win::current().w_cursor.lnum = lnum;
        }
    }
    let height = Win::current().w_height;
    scroll_to_fraction(Win::current(), height);
}

/// `:diffthis`: put the current window in diff mode.
pub fn ex_diffthis(_excmd: &mut ExArg) {
    diff_win_options(Win::current(), true);
}

/// Set `'diff'` in `window` without letting the option's side effects run.
///
/// `curwin` is moved to `window` for the call because the option code reads it,
/// and `diff_buf_adjust` is suppressed so that the caller stays in charge of
/// the buffer registry.
fn set_diff_option(window: Win, value: bool) {
    let saved = switch_to(window);
    Buf::current().b_ro_locked += 1;
    // `curwin`/`curbuf` name `window` and its buffer, which is what the option
    // code reads; the buffer is locked against a `:set` side effect.
    let val = boolean_optval(Some(value));
    set_option_value_give_err(kOptDiff, val, OptionSetFlags::LOCAL);
    Buf::current().b_ro_locked -= 1;
    saved.restore();
}

/// Put `window` into diff mode: the option set, and optionally its buffer.
///
/// Every option this changes is saved into the matching `w_p_*_save` field
/// first, but **only on the first call** -- `wo_diff_saved` is what stops a
/// second `:diffthis` from saving the diff-mode values as the ones to
/// restore.
///
/// Safe: a [`Win`] carries the whole of the promise this needs.
pub fn diff_win_options(mut window: Win, addbuf: bool) {
    let saved = switch_window(window);
    new_fold_level();
    saved.restore();

    // Each option is saved only while the window is not already in diff
    // mode, so a second `:diffthis` cannot overwrite the saved values.
    let first_time = window.w_onebuf_opt.wo_diff == 0;
    if first_time {
        window.w_onebuf_opt.wo_scb_save = window.w_onebuf_opt.wo_scb;
    }
    window.w_onebuf_opt.wo_scb = 1;
    if first_time {
        window.w_onebuf_opt.wo_crb_save = window.w_onebuf_opt.wo_crb;
    }
    window.w_onebuf_opt.wo_crb = 1;
    if diff_flags.get() & DIFF_FOLLOWWRAP == 0 {
        if first_time {
            window.w_onebuf_opt.wo_wrap_save = window.w_onebuf_opt.wo_wrap;
        }
        window.w_onebuf_opt.wo_wrap = 0;
        window.w_skipcol = 0 as ColNr;
    }
    if first_time {
        window.w_onebuf_opt.wo_fdm_save = window.w_onebuf_opt.wo_fdm.clone();
    }
    let foldmethod = OptVal::string(String_0::from_cstr(c"diff"));
    let scope = OptionSetFlags::LOCAL;
    set_option_direct_for(
        kOptFoldmethod,
        foldmethod,
        scope,
        0 as ScriptId,
        OptionTarget::Win(window),
    );
    if first_time {
        window.w_onebuf_opt.wo_fen_save = window.w_onebuf_opt.wo_fen;
        window.w_onebuf_opt.wo_fdl_save = window.w_onebuf_opt.wo_fdl;
        window.w_onebuf_opt.wo_fdc_save = window.w_onebuf_opt.wo_fdc.clone();
    }
    // A single digit, because the option's buffer is one byte plus the
    // NUL. C's `assert()` is `debug_assert!`: it vanishes under NDEBUG.
    let width = diff_foldcolumn.get();
    debug_assert!((0..=9).contains(&width));
    let digit = u8::try_from(width.clamp(0, 9)).unwrap_or(0);
    window.w_onebuf_opt.wo_fdc = Some(XString::from_bytes(&[b'0' + digit]));
    window.w_onebuf_opt.wo_fen = 1;
    window.w_onebuf_opt.wo_fdl = 0 as OptInt;
    fold_update_all(window);
    changed_window_setting(window);
    if !p_sbo(|value| has_char(value, 'h' as c_int)) {
        let _ = do_cmdline_cmd(c"set sbo+=hor");
    }
    window.w_onebuf_opt.wo_diff_saved = 1;
    set_diff_option(window, true);
    if addbuf {
        diff_buf_add(window.buffer());
    }
    window.redraw_later(UPD_NOT_VALID);
}

/// `:diffoff[!]`: leave diff mode in this window, or with `!` in every
/// window of the tabpage.
///
/// Each option goes back to its `w_p_*_save` value, but only where the
/// window still holds the value diff mode gave it: a value the user changed
/// in the meantime is left alone.
pub fn ex_diffoff(excmd: &mut ExArg) {
    let mut diffwin = false;
    // `FOR_ALL_WINDOWS_IN_TAB(wp, curtab)`: always the `firstwin` list.
    for mut wp in windows() {
        let wanted = if excmd.forceit {
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
                    wp.w_leftcol = 0 as ColNr;
                }
                wp.w_onebuf_opt.wo_fdm = saved_or(&wp.w_onebuf_opt.wo_fdm_save, c"manual");
                wp.w_onebuf_opt.wo_fdc = saved_or(&wp.w_onebuf_opt.wo_fdc_save, c"0");
                if wp.w_onebuf_opt.wo_fdl == 0 as OptInt {
                    wp.w_onebuf_opt.wo_fdl = wp.w_onebuf_opt.wo_fdl_save;
                }
                if wp.w_onebuf_opt.wo_fen != 0 {
                    let manual = foldmethod_is_manual(wp);
                    wp.w_onebuf_opt.wo_fen = if manual {
                        0
                    } else {
                        wp.w_onebuf_opt.wo_fen_save
                    };
                }
                fold_update_all(wp);
            }
            wp.w_topfill = 0;
            changed_window_setting(wp);
            diff_buf_adjust(wp);
        }
        diffwin = diffwin || wp.w_onebuf_opt.wo_diff != 0;
    }
    if excmd.forceit {
        diff_buf_clear();
    }
    let mut tp = TabPage::current();
    if !diffwin {
        diff_need_update.set(false);
        tp.tp_diff_invalid = 0;
        tp.tp_diff_update = 0;
        diff_clear(tp);
    }
    if !diffwin && p_sbo(|value| has_char(value, 'h' as c_int)) {
        let _ = do_cmdline_cmd(c"set sbo-=hor");
    }
}

/// The saved option string, or `fallback` when nothing was saved.
///
/// Upstream tests the saved value's *first byte*: an empty saved string means
/// the option was never really recorded.
fn saved_or(saved: &Option<XString>, fallback: &'static CStr) -> Option<XString> {
    match saved {
        Some(value) if !value.is_empty() => Some(value.clone()),
        _ => Some(XString::from_cstr(fallback)),
    }
}
