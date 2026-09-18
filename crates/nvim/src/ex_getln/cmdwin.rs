//! The command-line window (`q:`, `q/`, `'cedit'`).
//!
//! [`open_cmdwin`] opens a real buffer holding the history, runs a nested
//! `main_loop` over it, and turns whatever line the cursor was on into the
//! command line's answer.  The `*_locked` guards are here because they are
//! what stops that window being opened from somewhere it cannot unwind.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::buffer::{BufRef, current_buf};
use crate::ex_docmd::{cmdmod_add_flags, cmdmod_set_tab};
use crate::guard::Allow;
use crate::keycodes::{Ctrl_C, Key};
use crate::types::{CmdModFlags, NUL, OptError, OptionSetFlags};
use crate::window::valid_win;
use crate::winlayer::{Buf, Win};
use core::ffi::CStr;

/// True when the text must not be changed and we cannot switch to another
/// window or buffer — editing the command line, and the like.
pub fn text_locked() -> bool {
    if cmdwin_type.get() != 0 {
        return true;
    }
    if expr_map_locked() {
        return true;
    }
    textlock.get() != 0
}

/// Report a command that is not allowed while the cmdline window is open or
/// the command line is being edited another way.
pub fn text_locked_msg() {
    emsg(gettext(get_text_locked_msg()));
}

/// The message [`text_locked_msg`] gives: which of the two locks is on.
pub fn get_text_locked_msg() -> &'static CStr {
    if cmdwin_type.get() != 0 {
        e_cmdwin
    } else {
        e_textlock
    }
}

/// Check for text, window or buffer locked; report and answer true if it is.
pub fn text_or_buf_locked() -> bool {
    if text_locked() {
        text_locked_msg();
        return true;
    }
    curbuf_locked()
}

/// Check `curbuf->b_ro_locked` and `allbuf_lock`; report and answer true if
/// either is set.
pub fn curbuf_locked() -> bool {
    if Buf::current().b_ro_locked > 0 {
        emsg(gettext(e_cannot_edit_other_buf));
        return true;
    }
    allbuf_locked()
}

/// Check `allbuf_lock`; report and answer true if it is set.
pub fn allbuf_locked() -> bool {
    if allbuf_lock.get() > 0 {
        emsg(gettext(
            c"E811: Not allowed to change buffer information now",
        ));
        return true;
    }
    false
}

/// Zero the command-line state at startup.
pub fn cmdline_init() {
    ccline.set(CMDLINE_INFO_INIT);
}

/// `'cedit'` changed: re-derive the key that opens the command-line window.
pub fn did_set_cedit(_args: &mut OptSet) -> Result<(), OptError> {
    derive_cedit_key()
}

/// The `'cedit'` key itself, for the startup sweep, which has no option
/// frame to hand a callback.
///
/// Safe: the option's value is a C string from the moment the option table
/// is initialised, which is the whole of the precondition.
pub(crate) fn derive_cedit_key() -> Result<(), OptError> {
    if p_cedit(CStr::is_empty) {
        cedit_key.set(-1);
    } else {
        let n = p_cedit(|value| unsafe { string_to_key(value.as_ptr().cast_mut()) });
        if n == 0 || vim_isprintc(n) {
            return Err(e_invarg.into());
        }
        cedit_key.set(n);
    }
    Ok(())
}

/// Open a window on the current command line and its history, and edit in it.
///
/// Returns when the window is closed, with `CAR` if the command is to be
/// executed, `Ctrl_C` if it is to be abandoned, and `K_IGNORE` if editing
/// continues.
pub(crate) fn open_cmdwin() -> ::core::ffi::c_int {
    let mut bufref = BufRef::NONE;
    let old_curwin = Win::current().id();
    // Uninitialised in the C; `win_size_save` below fills it.
    let save_restart_edit = restart_edit.get();
    let save_state = State.get();
    let save_exmode = exmode_active.get();
    let save_cmdmsg_rl = cmdmsg_rl.get();

    // Can't do this when text or buffer is locked, can't do it
    // recursively, and can't do it when typing a password.
    if text_or_buf_locked() || cmdwin_type.get() != 0 || cmdline_star.get() > 0 {
        beep_flush();
        return Key::Ignore.code();
    }

    let old_curbuf = BufRef::of_opt(current_buf());

    // Save current window sizes.
    let winsizes = win_size_save();

    // When using completion in Insert mode with <C-R>=<C-F> one can open
    // the command line window, but we don't want the popup menu then.
    pum_undisplay(true);

    // Don't use a new tab page.
    cmdmod_set_tab(0);
    cmdmod_add_flags(CmdModFlags::NOSWAPFILE);

    // Create a window for the command-line buffer.
    if win_split(p_cwh() as ::core::ffi::c_int, WSP_BOT as ::core::ffi::c_int).is_err() {
        beep_flush();
        return Key::Ignore.code();
    }
    // win_split() autocommands may have messed with the old window or
    // buffer. Treat it as abandoning this command line.
    let live = valid_win(old_curwin);
    if live.is_none_or(|w| w.is_current() || !old_curbuf.is(w.buffer_or_none()))
        || !old_curbuf.valid()
    {
        beep_flush();
        return Ctrl_C;
    }
    // Don't let quitting the More prompt make this fail.
    got_int.set(false);

    // Set the "cmdwin_*" variables before any autocommand can mess
    // things up.
    cmdwin_type.set(get_cmdline_type());
    cmdwin_level.set(Cc::current().level);
    cmdwin_win.set(Some(Win::current().id()));
    cmdwin_old_curwin.set(Some(old_curwin));

    // Create the empty command-line buffer. Be especially cautious of
    // BufLeave autocommands from do_ecmd(): the cmdwin restrictions do
    // not apply to them.
    let newbuf_status =
        unsafe { buf_open_scratch(0, ::core::ptr::null_mut::<::core::ffi::c_char>()) };
    let cmdwin = cmdwin_win.get().and_then(valid_win);
    if newbuf_status.is_err()
        || cmdwin.is_none_or(|w| !w.is_current())
        || !win_valid(old_curwin)
        || !old_curbuf.valid()
        || old_curwin
            .get()
            .is_none_or(|w| !old_curbuf.is(w.buffer_or_none()))
    {
        if newbuf_status.is_ok() {
            bufref = BufRef::of_opt(current_buf());
        }
        if let Some(cw) = cmdwin
            && !last_window(cw)
        {
            win_close(cw, true, false);
        }
        // win_close() autocommands may have already deleted the buffer.
        if newbuf_status.is_ok()
            && let Some(buffer) = bufref.get()
            && buffer.raw() != Buf::current_raw()
        {
            wipe_buffer(buffer);
        }

        cmdwin_type.set(0);
        cmdwin_level.set(0);
        cmdwin_win.set(None);
        cmdwin_old_curwin.set(None);
        beep_flush();
        return Ctrl_C;
    }
    cmdwin_buf.set(Some(Buf::current().id()));

    // The command-line buffer has bufhidden=wipe, unlike a true
    // "scratch" buffer.
    set_option_value_give_err(kOptBufhidden, static_optval(c"wipe"), OptionSetFlags::LOCAL);
    Buf::current().b_p_ma = 1;
    Win::current().w_onebuf_opt.wo_fen = 0;
    Win::current().w_onebuf_opt.wo_rl = cmdmsg_rl.get() as ::core::ffi::c_int;
    cmdmsg_rl.set(false);

    // Don't allow switching to another buffer.
    Buf::current().b_ro_locked += 1;

    // Showing the prompt may have set need_wait_return; reset it.
    need_wait_return.set(false);

    let histtype = hist_char2type(cmdwin_type.get());
    if histtype == HIST_CMD || histtype == HIST_DEBUG {
        if p_wc() == TAB as OptInt {
            let tab = c"<Tab>".as_ptr().cast_mut();
            let (ins, nrm) = (
                c"<C-X><C-V>".as_ptr().cast_mut(),
                c"a<C-X><C-V>".as_ptr().cast_mut(),
            );
            // SAFETY: four static NUL-terminated strings.
            unsafe { add_map(tab, ins, MODE_INSERT, true) };
            unsafe { add_map(tab, nrm, MODE_NORMAL, true) };
        }
        set_option_value_give_err(kOptFiletype, static_optval(c"vim"), OptionSetFlags::LOCAL);
    }
    Buf::current().b_ro_locked -= 1;

    // Reset 'textwidth' after setting 'filetype' (the Vim filetype plugin
    // sets 'textwidth' to 78).
    Buf::current().b_p_tw = 0;

    // Fill the buffer with the history.
    init_history();
    if get_hislen() > 0 && histtype != HIST_INVALID {
        let mut i = get_hisidx(histtype);
        if i >= 0 {
            let mut lnum: LineNr = 0;
            // C's do-while: `get_hisidx` is re-read at the test, because
            // `ml_append`'s autocommands can move it.
            loop {
                i += 1;
                if i == get_hislen() {
                    i = 0;
                }
                if let Some(entry) = hist_entry_ref(histtype, i) {
                    let text = entry.text as *mut ::core::ffi::c_char;
                    let _ = unsafe { ml_append(lnum, text, 0, false) };
                    lnum += 1;
                }
                if i == get_hisidx(histtype) {
                    break;
                }
            }
        }
    }

    // Replace the empty last line with the current command line and put
    // the cursor there.
    let (last, text) = (Buf::current().b_ml.ml_line_count, Cc::current().text());
    let _ = unsafe { ml_replace(last, text, true) };
    Win::current().w_cursor.lnum = Buf::current().b_ml.ml_line_count;
    Win::current().w_cursor.col = Cc::current().cmdpos as ColNr;
    changed_line_abv_curs();
    invalidate_botline_win(Win::current());
    ui_ext_cmdline_hide(false);
    redraw_later(Win::current(), UPD_SOME_VALID);

    // No Ex mode here.
    exmode_active.set(false);

    State.set(MODE_NORMAL);
    setmouse();
    clear_showcmd();

    // Reset here so a CmdwinEnter autocommand can set it.
    cmdwin_result.set(0);

    trigger_cmd_autocmd(cmdwin_type.get(), AutoEvent::CmdwinEnter);
    if restart_edit.get() != 0 {
        // An autocmd ran ":startinsert".
        stuff_readbuf_char(Key::Nop.code());
    }

    let redraw = Allow::redraw();
    let save_count = crate::clipboard::save_batch_count();

    // Call the main loop until <CR> or CTRL-C is typed.
    normal_enter(true, false);

    drop(redraw);
    crate::clipboard::restore_batch_count(save_count);

    let save_key_typed = KeyTyped.get();
    trigger_cmd_autocmd(cmdwin_type.get(), AutoEvent::CmdwinLeave);
    // Restore KeyTyped in case an autocommand modified it.
    KeyTyped.set(save_key_typed);

    cmdwin_type.set(0);
    cmdwin_level.set(0);
    cmdwin_buf.set(None);
    cmdwin_win.set(None);
    cmdwin_old_curwin.set(None);

    exmode_active.set(save_exmode);

    // Safety check: the old window or buffer was changed or deleted.
    // It is a bug when this happens.
    if valid_win(old_curwin).is_none_or(|w| !old_curbuf.is(w.buffer_or_none()))
        || !old_curbuf.valid()
    {
        cmdwin_result.set(Ctrl_C);
        emsg(gettext(e_active_window_or_buffer_changed_or_deleted));
    } else {
        // Autocmds may abort script processing.
        if aborting() && cmdwin_result.get() != Key::Ignore.code() {
            cmdwin_result.set(Ctrl_C);
        }
        // Set the new command line from the cmdline buffer.
        dealloc_cmdbuff();

        if cmdwin_result.get() == Key::Xf1.code() || cmdwin_result.get() == Key::Xf2.code() {
            // ":qa[!]" was typed.
            let p = if cmdwin_result.get() == Key::Xf2.code() {
                c"qa"
            } else {
                c"qa!"
            };

            if histtype == HIST_CMD {
                // Execute the command directly.
                unsafe { Cc::current().set_cstr(p.as_ptr()) };
                cmdwin_result.set(CAR);
            } else {
                // First need to cancel what we were doing.
                stuff_readbuf_char(':' as ::core::ffi::c_int);
                stuff_readbuf(p);
                stuff_readbuf_char(CAR);
            }
        } else if cmdwin_result.get() == Ctrl_C {
            // ":q" or ":close": don't execute any command and don't
            // modify the cmdline window.
            Cc::current().close();
        } else {
            let n = get_cursor_line_len() as ::core::ffi::c_int;
            Cc::current().set_text(unsafe {
                ::core::slice::from_raw_parts(get_cursor_line_ptr(), n.max(0) as usize)
            });
        }

        let mut cc = Cc::current();
        if !cc.in_use() {
            cc.set_text(&[]);
            cc.cmdpos = 0;
            cmdwin_result.set(Ctrl_C);
        } else {
            cc.cmdpos = Win::current().w_cursor.col as ::core::ffi::c_int;
            // If the cursor is on the last character, it probably should
            // be after it.
            if cc.cmdpos == cc.len() - 1 || cc.cmdpos > cc.len() {
                cc.cmdpos = cc.len();
            }
            if cmdwin_result.get() == Key::Ignore.code() {
                cc.cmdspos = cmd_screencol(cc.cmdpos);
                redrawcmd();
            }
        }

        // Avoid the command-line window's first character being
        // concealed.
        Win::current().w_onebuf_opt.wo_cole = 0;
        // First go back to the original window.
        let wp = Win::current().id();
        bufref = BufRef::of_opt(current_buf());
        skip_win_fix_cursor.set(true);
        let old = old_curwin.get().expect("just checked live");
        win_goto(old);

        // win_goto() may trigger an autocommand that already closes the
        // cmdline window.
        if let Some(wp) = valid_win(wp).filter(|w| !w.is_current()) {
            win_close(wp, true, false);
        }

        // win_close() may have already wiped the buffer when 'bh' is set
        // to 'wipe'; autocommands may have closed other windows.
        if let Some(buffer) = bufref.get()
            && buffer.raw() != Buf::current_raw()
        {
            wipe_buffer(buffer);
        }

        // Restore window sizes.
        win_size_restore(&winsizes);
        skip_win_fix_cursor.set(false);
    }

    restart_edit.set(save_restart_edit);
    cmdmsg_rl.set(save_cmdmsg_rl);

    State.set(save_state);
    may_trigger_modechanged();
    setmouse();
    setcursor();

    cmdwin_result.get()
}

/// True when in the cmdwin, and not editing the command line.
pub fn is_in_cmdwin() -> bool {
    cmdwin_type.get() != 0 && get_cmdline_type() == NUL
}

/// C's `close_buffer(NULL, buf, DOBUF_WIPE, false, false)`: wipe the command
/// window's buffer out, window-less and without forcing.
fn wipe_buffer(buffer: Buf) {
    let wipe = DOBUF_WIPE as ::core::ffi::c_int;
    close_buffer(None, buffer, wipe, false, false);
}
