//! The command-line window (`q:`, `q/`, `'cedit'`).
//!
//! [`open_cmdwin`] opens a real buffer holding the history, runs a nested
//! `MAIN_LOOP` over it, and turns whatever line the cursor was on into the
//! command line's answer.  The `*_locked` guards are here because they are
//! what stops that window being opened from somewhere it cannot unwind.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use super::*;
use crate::buffer::DOBUF_WIPE;
use crate::buffer::{BufRef, current_buf};
use crate::cmdhist::HistoryType;
use crate::cstr;
use crate::ex_docmd::{cmdmod_add_flags, cmdmod_set_tab};
use crate::guard::Allow;
use crate::keycodes::SpecialKey;
use crate::mbyte::Codepoint;
use crate::option::defs::{OptError, OptionSetFlags};
use crate::types::CmdModFlags;
use crate::window::valid_win;
use crate::winlayer::{Buf, Win};
use core::ffi::CStr;

/// True when the text must not be changed and we cannot switch to another
/// window or buffer — editing the command line, and the like.
pub fn text_locked() -> bool {
    if CMDWIN_TYPE.get() != 0 {
        return true;
    }
    if expr_map_locked() {
        return true;
    }
    TEXTLOCK.get() != 0
}

/// Report a command that is not allowed while the cmdline window is open or
/// the command line is being edited another way.
pub fn text_locked_msg() {
    emsg(gettext(get_text_locked_msg()));
}

/// The message [`text_locked_msg`] gives: which of the two locks is on.
pub fn get_text_locked_msg() -> &'static CStr {
    if CMDWIN_TYPE.get() != 0 {
        E_CMDWIN
    } else {
        E_TEXTLOCK
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

/// Check `CURBUF->b_ro_locked` and `ALLBUF_LOCK`; report and answer true if
/// either is set.
pub fn curbuf_locked() -> bool {
    if Buf::current().b_ro_locked > 0 {
        emsg(gettext(E_CANNOT_EDIT_OTHER_BUF));
        return true;
    }
    allbuf_locked()
}

/// Check `ALLBUF_LOCK`; report and answer true if it is set.
pub fn allbuf_locked() -> bool {
    if ALLBUF_LOCK.get() > 0 {
        emsg(gettext(
            c"E811: Not allowed to change buffer information now",
        ));
        return true;
    }
    false
}

/// Zero the command-line state at startup.
pub fn cmdline_init() {
    CCLINE.set(CMDLINE_INFO_INIT);
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
        CEDIT_KEY.set(None);
    } else {
        // The option names the key the way `'wildchar'` does: by its code.
        let n = p_cedit(|value| string_to_key(value.to_bytes()));
        if n == 0 || vim_isprintc(Codepoint::new(n)) {
            return Err(E_INVARG.into());
        }
        CEDIT_KEY.set(Some(Key::from_raw(n)));
    }
    Ok(())
}

/// Open a window on the current command line and its history, and edit in it.
///
/// Returns when the window is closed, with `CAR` if the command is to be
/// executed, `CTRL-C` if it is to be abandoned, and `<Ignore>` if editing
/// continues.
pub(crate) fn open_cmdwin() -> Key {
    let old_curwin = Win::current().id();
    let save_restart_edit = RESTART_EDIT.get();
    let save_state = STATE.get();
    let save_exmode = EXMODE_ACTIVE.get();
    let save_cmdmsg_rl = CMDMSG_RL.get();

    // Can't do this when text or buffer is locked, can't do it
    // recursively, and can't do it when typing a password.
    if text_or_buf_locked() || CMDWIN_TYPE.get() != 0 || CMDLINE_STAR.get() > 0 {
        beep_flush();
        return Key::from(SpecialKey::Ignore);
    }

    let old_curbuf = BufRef::of_opt(current_buf());

    // Save current window sizes.
    let winsizes = win_size_save();

    if let Err(answer) = split_cmdwin(old_curwin, &old_curbuf) {
        return answer;
    }
    let histtype = set_up_cmdwin_buffer();
    fill_cmdwin_with_history(histtype);
    put_command_line_in_cmdwin();

    // No Ex mode here.
    EXMODE_ACTIVE.set(false);
    edit_in_cmdwin();
    EXMODE_ACTIVE.set(save_exmode);

    // Safety check: the old window or buffer was changed or deleted.
    // It is a bug when this happens.
    if valid_win(old_curwin).is_none_or(|w| !old_curbuf.is(w.buffer_or_none()))
        || !old_curbuf.valid()
    {
        CMDWIN_RESULT.set(Key::CTRL_C);
        emsg(gettext(E_ACTIVE_WINDOW_OR_BUFFER_CHANGED_OR_DELETED));
    } else {
        take_cmdwin_line(histtype);
        close_cmdwin(old_curwin, &winsizes);
    }

    RESTART_EDIT.set(save_restart_edit);
    CMDMSG_RL.set(save_cmdmsg_rl);

    STATE.set(save_state);
    may_trigger_modechanged();
    setmouse();
    setcursor();

    CMDWIN_RESULT.get()
}

/// Split a window off below for the command-line window and give it a fresh
/// scratch buffer, setting the `CMDWIN_*` state.
///
/// `Err` holds [`open_cmdwin`]'s answer when it cannot: `<Ignore>` when the
/// split fails, CTRL-C when autocommands messed with the old window or
/// buffer (treated as abandoning the command line).
fn split_cmdwin(old_curwin: WinId, old_curbuf: &BufRef) -> Result<(), Key> {
    // When using completion in Insert mode with <C-R>=<C-F> one can open
    // the command line window, but we don't want the popup menu then.
    pum_undisplay(true);

    // Don't use a new tab page.
    cmdmod_set_tab(0);
    cmdmod_add_flags(CmdModFlags::NOSWAPFILE);

    // Create a window for the command-line buffer.
    if win_split(
        ::core::ffi::c_int::try_from(p_cwh()).unwrap_or(::core::ffi::c_int::MAX),
        WSP_BOT,
    )
    .is_err()
    {
        beep_flush();
        return Err(Key::from(SpecialKey::Ignore));
    }
    // win_split() autocommands may have messed with the old window or
    // buffer. Treat it as abandoning this command line.
    let live = valid_win(old_curwin);
    if live.is_none_or(|w| w.is_current() || !old_curbuf.is(w.buffer_or_none()))
        || !old_curbuf.valid()
    {
        beep_flush();
        return Err(Key::CTRL_C);
    }
    // Don't let quitting the More prompt make this fail.
    GOT_INT.set(false);

    // Set the "cmdwin_*" variables before any autocommand can mess
    // things up.
    CMDWIN_TYPE.set(get_cmdline_type());
    CMDWIN_LEVEL.set(Cc::current().level);
    CMDWIN_WIN.set(Some(Win::current().id()));
    CMDWIN_OLD_CURWIN.set(Some(old_curwin));

    // Create the empty command-line buffer. Be especially cautious of
    // BufLeave autocommands from do_ecmd(): the cmdwin restrictions do
    // not apply to them.
    let newbuf_status = buf_open_scratch(Handle::NONE, None);
    let cmdwin = CMDWIN_WIN.get().and_then(valid_win);
    if newbuf_status.is_err()
        || cmdwin.is_none_or(|w| !w.is_current())
        || !win_valid(old_curwin)
        || !old_curbuf.valid()
        || old_curwin
            .get()
            .is_none_or(|w| !old_curbuf.is(w.buffer_or_none()))
    {
        let bufref = if newbuf_status.is_ok() {
            BufRef::of_opt(current_buf())
        } else {
            BufRef::NONE
        };
        if let Some(cw) = cmdwin
            && !last_window(cw)
        {
            let _ = win_close(cw, true, false);
        }
        // win_close() autocommands may have already deleted the buffer.
        if newbuf_status.is_ok()
            && let Some(buffer) = bufref.get()
            && buffer.raw() != Buf::current_raw()
        {
            wipe_buffer(buffer);
        }

        CMDWIN_TYPE.set(0);
        CMDWIN_LEVEL.set(0);
        CMDWIN_WIN.set(None);
        CMDWIN_OLD_CURWIN.set(None);
        beep_flush();
        return Err(Key::CTRL_C);
    }
    CMDWIN_BUF.set(Some(Buf::current().id()));
    Ok(())
}

/// Set the command-line window's buffer options — and, for Ex commands, the
/// `<Tab>` completion maps and 'filetype' `vim`.  Answers the history the
/// window edits.
fn set_up_cmdwin_buffer() -> HistoryType {
    // The command-line buffer has bufhidden=wipe, unlike a true
    // "scratch" buffer.
    set_option_value_give_err(
        OPT_BUFHIDDEN,
        OptVal::static_string(c"wipe"),
        OptionSetFlags::LOCAL,
    );
    Buf::current().b_p_ma = 1;
    Win::current().w_onebuf_opt.wo_fen = 0;
    Win::current().w_onebuf_opt.wo_rl = ::core::ffi::c_int::from(CMDMSG_RL.get());
    CMDMSG_RL.set(false);

    // Don't allow switching to another buffer.
    Buf::current().b_ro_locked += 1;

    // Showing the prompt may have set need_wait_return; reset it.
    NEED_WAIT_RETURN.set(false);

    let histtype = hist_char2type(CMDWIN_TYPE.get());
    if histtype == HIST_CMD || histtype == HIST_DEBUG {
        if wildchar() == Key::TAB {
            add_map(b"<Tab>", b"<C-X><C-V>", MODE_INSERT, true);
            add_map(b"<Tab>", b"a<C-X><C-V>", MODE_NORMAL, true);
        }
        set_option_value_give_err(
            OPT_FILETYPE,
            OptVal::static_string(c"vim"),
            OptionSetFlags::LOCAL,
        );
    }
    Buf::current().b_ro_locked -= 1;

    // Reset 'textwidth' after setting 'filetype' (the Vim filetype plugin
    // sets 'textwidth' to 78).
    Buf::current().b_p_tw = 0;
    histtype
}

/// Fill the command-line window's buffer with the history, oldest first.
fn fill_cmdwin_with_history(histtype: HistoryType) {
    init_history();
    if get_hislen() == 0 || histtype == HIST_INVALID {
        return;
    }
    let Some(mut i) = get_hisidx(histtype) else {
        return;
    };
    let mut lnum: LineNr = LineNr::ZERO;
    // C's do-while: `get_hisidx` is re-read at the test, because
    // `ml_append`'s autocommands can move it.
    loop {
        i += 1;
        if i == get_hislen() {
            i = 0;
        }
        if let Some(entry) = hist_entry_text(histtype, i) {
            let _ = ml_append_cstr(lnum, &entry.text);
            lnum += 1;
        }
        if Some(i) == get_hisidx(histtype) {
            break;
        }
    }
}

/// Replace the empty last line with the current command line and put the
/// cursor there.
fn put_command_line_in_cmdwin() {
    let last = Buf::current().b_ml.ml_line_count;
    let cc = Cc::current();
    if cc.in_use() {
        // A copy of the command line's text, up to its first NUL.
        let text = cstr::until_nul(cc.bytes()).to_vec();
        let _ = ml_replace_buf_text(Buf::current(), last, &text);
    }
    Win::current().w_cursor.lnum = Buf::current().b_ml.ml_line_count;
    Win::current().w_cursor.col = ColNr::new(Cc::current().cmdpos);
    changed_line_abv_curs();
    invalidate_botline_win(Win::current());
    ui_ext_cmdline_hide(false);
    redraw_later(Win::current(), UPD_SOME_VALID);
}

/// Run Normal mode in the command-line window until `<CR>` or CTRL-C is
/// typed, between its `CmdwinEnter` and `CmdwinLeave`, then clear the
/// `CMDWIN_*` state.
fn edit_in_cmdwin() {
    STATE.set(MODE_NORMAL);
    setmouse();
    clear_showcmd();

    // Reset here so a CmdwinEnter autocommand can set it.
    CMDWIN_RESULT.set(Key::NUL);

    trigger_cmd_autocmd(CMDWIN_TYPE.get(), AutoEvent::CmdwinEnter);
    if !RESTART_EDIT.get().is_nul() {
        // An autocmd ran ":startinsert".
        stuff_readbuf_char(Key::from(SpecialKey::Nop));
    }

    let redraw = Allow::redraw();
    let save_count = crate::clipboard::save_batch_count();

    // Call the main loop until <CR> or CTRL-C is typed.
    normal_enter(true, false);

    drop(redraw);
    crate::clipboard::restore_batch_count(save_count);

    let save_key_typed = KEY_TYPED.get();
    trigger_cmd_autocmd(CMDWIN_TYPE.get(), AutoEvent::CmdwinLeave);
    // Restore KeyTyped in case an autocommand modified it.
    KEY_TYPED.set(save_key_typed);

    CMDWIN_TYPE.set(0);
    CMDWIN_LEVEL.set(0);
    CMDWIN_BUF.set(None);
    CMDWIN_WIN.set(None);
    CMDWIN_OLD_CURWIN.set(None);
}

/// Set the new command line from the command-line window: the line under
/// the cursor, `:qa[!]`, or nothing for `:q`/CTRL-C, per `CMDWIN_RESULT`.
fn take_cmdwin_line(histtype: HistoryType) {
    // Autocmds may abort script processing.
    if aborting() && CMDWIN_RESULT.get() != SpecialKey::Ignore {
        CMDWIN_RESULT.set(Key::CTRL_C);
    }
    dealloc_cmdbuff();

    if CMDWIN_RESULT.get() == SpecialKey::Xf1 || CMDWIN_RESULT.get() == SpecialKey::Xf2 {
        // ":qa[!]" was typed.
        let p = if CMDWIN_RESULT.get() == SpecialKey::Xf2 {
            c"qa"
        } else {
            c"qa!"
        };

        if histtype == HIST_CMD {
            // Execute the command directly.
            Cc::current().set_text(p.to_bytes());
            CMDWIN_RESULT.set(Key::CAR);
        } else {
            // First need to cancel what we were doing.
            stuff_readbuf_char(Key::ascii(b':'));
            stuff_readbuf(p.to_bytes());
            stuff_readbuf_char(Key::CAR);
        }
    } else if CMDWIN_RESULT.get() == Key::CTRL_C {
        // ":q" or ":close": don't execute any command and don't modify the
        // cmdline window.
        Cc::current().close();
    } else {
        let lnum = Win::current().w_cursor.lnum;
        Cc::current().set_text(Buf::current().lines().line(lnum));
    }

    let mut cc = Cc::current();
    if !cc.in_use() {
        cc.set_text(&[]);
        cc.cmdpos = 0;
        CMDWIN_RESULT.set(Key::CTRL_C);
    } else {
        cc.cmdpos = Win::current().w_cursor.col.get();
        // If the cursor is on the last character, it probably should be
        // after it.
        if cc.cmdpos == cc.len() - 1 || cc.cmdpos > cc.len() {
            cc.cmdpos = cc.len();
        }
        if CMDWIN_RESULT.get() == SpecialKey::Ignore {
            cc.cmdspos = cmd_screencol(cc.cmdpos);
            redrawcmd();
        }
    }
}

/// Go back to `old_curwin`, close the command-line window and wipe its
/// buffer, and restore the window sizes.
fn close_cmdwin(old_curwin: WinId, winsizes: &[::core::ffi::c_int]) {
    // Avoid the command-line window's first character being concealed.
    Win::current().w_onebuf_opt.wo_cole = 0;
    // First go back to the original window.
    let wp = Win::current().id();
    let bufref = BufRef::of_opt(current_buf());
    SKIP_WIN_FIX_CURSOR.set(true);
    let old = old_curwin.get().expect("just checked live");
    win_goto(old);

    // win_goto() may trigger an autocommand that already closes the cmdline
    // window.
    if let Some(wp) = valid_win(wp).filter(|w| !w.is_current()) {
        let _ = win_close(wp, true, false);
    }

    // win_close() may have already wiped the buffer when 'bh' is set to
    // 'wipe'; autocommands may have closed other windows.
    if let Some(buffer) = bufref.get()
        && buffer.raw() != Buf::current_raw()
    {
        wipe_buffer(buffer);
    }

    win_size_restore(winsizes);
    SKIP_WIN_FIX_CURSOR.set(false);
}

/// True when in the cmdwin, and not editing the command line.
pub fn is_in_cmdwin() -> bool {
    CMDWIN_TYPE.get() != 0 && get_cmdline_type() == 0
}

/// C's `close_buffer(NULL, buf, DOBUF_WIPE, false, false)`: wipe the command
/// window's buffer out, window-less and without forcing.
fn wipe_buffer(buffer: Buf) {
    let wipe = DOBUF_WIPE;
    close_buffer(None, buffer, wipe, false, false);
}
