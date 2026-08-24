//! `vgetorpeek` and `inchar`: the bottom of the input stack.
//!
//! [`vgetorpeek`] is the loop that turns "something is in the typeahead" into
//! "a byte to hand out": it consults the stuff buffers, runs the mapping
//! match, and blocks in [`inchar`] when neither has anything. [`inchar`] is
//! the only place that reads the script stack or the OS input buffer.
//!
//! Two upstream properties this file is faithful to, both reachable and both
//! on the divergence docket (D-B13-1, D-B13-2):
//!
//! - Under `ex_normal_busy` the "get it from the user" arm never reaches
//!   [`inchar`]; it answers a synthetic ESC (or CTRL-C on the command line)
//!   so that `:normal` cannot block. `nvim_feedkeys(k, "Lx")` puts its bytes
//!   in the *low-level* input buffer and then runs `exec_normal` with
//!   `use_vpeekc` under exactly that flag, so `vpeekc()` answers ESC forever
//!   and the loop spins. Upstream v0.12.4 has the identical structure
//!   (getchar.c:2812-2841, api/vim.c:323-350, ex_docmd.c:7451-7468); the
//!   comment on `exec_normal`'s loop shows the author expected CTRL-C here.
//! - In `--headless -l` (which sets `silent_mode`) a blocking [`inchar`]
//!   reaches `input_get`, which drains the event loop and then calls
//!   `read_error_exit()` -> `getout(0)`. That is upstream's designed exit for
//!   batch mode, not a tty-dependent accident.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::keycodes::{Ctrl_C, key_escape};
use crate::types::NUL;
use core::ffi::{c_char, c_int, c_long};
use core::ptr;

/// Longest partial mapping `'showcmd'` will display.
const SHOWCMD_COLS: c_int = 10;

/// Where the cursor was left after the `<Esc>`-in-Insert peek, which is where
/// `'showcmd'` has to draw the partially matched keys.
#[derive(Clone, Copy)]
struct CursorAt {
    wcol: c_int,
    wrow: c_int,
}

/// What [`show_partial_key`] put on the screen and has to take back off.
struct Partial {
    /// Index into the typeahead the showcmd display started at; 0 when
    /// nothing was pushed.
    showcmd_idx: c_int,
    /// Whether a character was drawn into the text or the command line.
    showing: bool,
}

/// Peek 25ms for more input after an `<Esc>` in Insert mode and, when none
/// comes, move the cursor as if Insert mode had been left.
///
/// This is what avoids the one-second pause after typing `<Esc>`: the mode
/// message is taken down and the cursor moved left straight away, and if a
/// key does arrive after all the display is simply redrawn. Answers the
/// cursor position to draw `'showcmd'` at, and whether the mode message was
/// removed.
///
/// # Safety
/// Callable at any time; `typebuf` must have room for three more bytes.
unsafe fn esc_leaves_insert(at: &mut CursorAt) -> bool {
    unsafe {
        let deleted = mode_displayed.get();
        if deleted {
            unshowmode(true);
        }
        validate_cursor(curwin.get());
        let win = curwin.get();
        let old_wcol = (*win).w_wcol;
        let old_wrow = (*win).w_wrow;

        // Move the cursor left, if that is possible.
        if (*win).w_cursor.col != 0 {
            let mut col: colnr_T = 0;
            if (*win).w_wcol > 0 {
                if did_ai.get()
                    && c_int::from(*skipwhite(
                        get_cursor_line_ptr().offset((*win).w_cursor.col as isize),
                    )) == NUL
                {
                    // After auto-indenting with no text following, the
                    // trailing white space is about to be truncated, so the
                    // cursor belongs after the last non-white character.
                    (*win).w_wcol = 0;
                    let ptr = get_cursor_line_ptr();
                    let endptr = ptr.offset((*win).w_cursor.col as isize);

                    let mut csarg = CharsizeArg::default();
                    let cstype = init_charsize_arg(&mut csarg, win, (*win).w_cursor.lnum, ptr);
                    let mut ci = utf_ptr2str_char_info(ptr);
                    let mut vcol = 0;
                    while ci.ptr < endptr {
                        if !ascii_iswhite(ci.chr.value as c_int) {
                            (*win).w_wcol = vcol;
                        }
                        vcol += win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &mut csarg).width;
                        ci = utfc_next(ci);
                    }

                    (*win).w_wrow = (*win).w_cline_row + (*win).w_wcol / (*win).w_view_width;
                    (*win).w_wcol %= (*win).w_view_width;
                    (*win).w_wcol += win_col_off(win);
                    col = 0; // no correction needed
                } else {
                    (*win).w_wcol -= 1;
                    col = (*win).w_cursor.col - 1;
                }
            } else if (*win).w_onebuf_opt.wo_wrap != 0 && (*win).w_wrow != 0 {
                (*win).w_wrow -= 1;
                (*win).w_wcol = (*win).w_view_width - 1;
                col = (*win).w_cursor.col - 1;
            }
            if col > 0 && (*win).w_wcol > 0 {
                // Correct for the cursor sitting on the right half of a
                // double-width character.
                let ptr = get_cursor_line_ptr();
                col -= utf_head_off(ptr, ptr.offset(col as isize));
                if utf_ptr2cells(ptr.offset(col as isize)) > 1 {
                    (*win).w_wcol -= 1;
                }
            }
        }
        setcursor();
        ui_flush();

        at.wcol = (*win).w_wcol;
        at.wrow = (*win).w_wrow;
        (*win).w_wcol = old_wcol;
        (*win).w_wrow = old_wrow;
        deleted
    }
}

/// Show the partially matched keys with `'showcmd'` while we wait for the
/// rest of a mapping.
///
/// # Safety
/// Callable at any time.
unsafe fn show_partial_key(at: CursorAt) -> Partial {
    unsafe {
        let tb = typeahead();
        let mut partial = Partial {
            showcmd_idx: 0,
            showing: false,
        };

        if (State.get() & (MODE_NORMAL | MODE_INSERT) != 0 || State.get() == MODE_LANGMAP)
            && State.get() != MODE_HITRETURN
        {
            let last = tb.at(tb.len() - 1);
            if State.get() & MODE_INSERT != 0 && ptr2cells(last.cast()) == 1 {
                // This looks nice when typing a dead-character mapping.
                edit_putchar(c_int::from(*last), false);
                setcursor(); // put the cursor back where it belongs
                partial.showing = true;
            }
            // The showcmd area is drawn relative to the cursor position the
            // <Esc> peek above left, not the one the cursor is at now.
            let win = curwin.get();
            let old_wcol = (*win).w_wcol;
            let old_wrow = (*win).w_wrow;
            (*win).w_wcol = at.wcol;
            (*win).w_wrow = at.wrow;
            push_showcmd();
            if tb.len() > SHOWCMD_COLS {
                partial.showcmd_idx = tb.len() - SHOWCMD_COLS;
            }
            while partial.showcmd_idx < tb.len() {
                add_byte_to_showcmd(*tb.at(partial.showcmd_idx));
                partial.showcmd_idx += 1;
            }
            (*win).w_wcol = old_wcol;
            (*win).w_wrow = old_wrow;
        }

        // The same on the command line, where `get_number()` has none.
        if State.get() & MODE_CMDLINE != 0
            && !(*get_cmdline_info()).cmdbuff.is_null()
            && cmdline_star.get() == 0
        {
            let p = tb.at(tb.len() - 1);
            if ptr2cells(p.cast()) == 1 && c_int::from(*p) < 128 {
                putcmdline(*p as c_char, false);
                partial.showing = true;
            }
        }
        partial
    }
}

/// Take back what [`show_partial_key`] drew.
///
/// # Safety
/// `partial` must be what the matching [`show_partial_key`] answered.
unsafe fn unshow_partial_key(partial: &Partial) {
    unsafe {
        if partial.showcmd_idx != 0 {
            pop_showcmd();
        }
        if partial.showing {
            if State.get() & MODE_INSERT != 0 {
                edit_unputchar();
            }
            if State.get() & MODE_CMDLINE != 0 && !(*get_cmdline_info()).cmdbuff.is_null() {
                unputcmdline();
            } else {
                setcursor(); // put the cursor back where it belongs
            }
        }
    }
}

/// How long to wait in [`inchar`] for the rest of a mapping or key code.
fn wait_time_for(advance: bool, keylen: c_int) -> c_long {
    if !advance {
        return 0;
    }
    if typeahead().is_empty()
        || !(p_timeout.get() != 0 || (p_ttimeout.get() != 0 && keylen == KEYLEN_PART_KEY))
    {
        -1 // blocking wait
    } else if keylen == KEYLEN_PART_KEY && p_ttm.get() >= 0 {
        p_ttm.get() as c_long
    } else {
        p_tm.get() as c_long
    }
}

/// Everything that got read after a CTRL-C, and the key to answer with.
///
/// # Safety
/// Callable at any time.
unsafe fn interrupted(advance: bool) -> c_int {
    unsafe {
        let tb = typeahead();
        // Flush all input.
        let got = inchar(tb.storage(), tb.buflen() - 1, 0);

        // If `inchar` answered true (a script file was active) or we are
        // inside a mapping, get out of Insert mode; otherwise behave as if a
        // CTRL-C had been typed, so that typing CTRL-C in Insert mode really
        // inserts one.
        let c = if (got != 0 || tb.maplen() != 0) && State.get() & (MODE_INSERT | MODE_CMDLINE) != 0
        {
            ESC
        } else {
            Ctrl_C
        };
        flush_buffers(FLUSH_INPUT); // flush all typeahead

        if advance {
            // Record this character too; it may be needed to get out of
            // Insert mode.
            *tb.storage() = c as u8;
            gotchars(tb.storage(), 1);
        }
        cmd_silent.set(false);
        c
    }
}

/// The loop that fills the typeahead and matches mappings against it.
///
/// Answers the byte to hand out, or a negative value when the input script
/// ended and the caller should start over.
///
/// # Safety
/// Callable at any time; may block waiting for input.
unsafe fn read_from_typeahead(
    advance: bool,
    timedout: &mut bool,
    mapdepth: &mut c_int,
    mode_deleted: &mut bool,
) -> c_int {
    unsafe {
        loop {
            check_end_reg_executing(advance);

            // `os_breakcheck` is slow; inside a mapping do not use it every
            // time round, but do for every typed character.
            if typeahead().maplen() != 0 {
                line_breakcheck();
            } else {
                // os_breakcheck() can call input_enqueue().
                if (mapped_ctrl_c.get() | (*curbuf.get()).b_mapped_ctrl_c) & get_real_state() != 0 {
                    ctrl_c_interrupts.set(false);
                }
                os_breakcheck(); // check for CTRL-C
                ctrl_c_interrupts.set(true);
            }

            let mut keylen = 0;
            if got_int.get() {
                return interrupted(advance);
            } else if !typeahead().is_empty() {
                // Check for a mapping in the typeahead.
                match handle_mapping(&raw mut keylen, timedout, mapdepth) as map_result_T {
                    map_result_retry => continue, // try mapping again
                    map_result_fail => return -1, // failed; use the outer loop
                    map_result_get => {
                        // Take the character from the typeahead.
                        let tb = typeahead();
                        let c = tb.byte(0);
                        if advance {
                            cmd_silent.set(tb.silent() > 0);
                            if tb.maplen() > 0 {
                                KeyTyped.set(false);
                            } else {
                                KeyTyped.set(true);
                                // Write the character to the script file(s).
                                gotchars(tb.at(0), 1);
                            }
                            KeyNoremap.set(tb.noremap(0));
                            del_typebuf(1, 0);
                        }
                        return c;
                    }
                    _ => {} // not enough characters; get more
                }
            }

            // Get a character from the user, handling <Esc> in Insert mode.
            //
            // Special case: an <ESC> in Insert mode with nothing else
            // immediately available means we pretend to leave Insert mode,
            // which avoids the one-second delay after typing <ESC>. If
            // something does arrive after all the mode may have to be
            // redisplayed; that the cursor is in the wrong place until then
            // does not matter.
            let mut c = 0;
            let win = curwin.get();
            let mut at = CursorAt {
                wcol: (*win).w_wcol,
                wrow: (*win).w_wrow,
            };
            let tb = typeahead();
            if advance
                && tb.len() == 1
                && tb.byte(0) == ESC
                && no_mapping.get() == 0
                && ex_normal_busy.get() == 0
                && tb.maplen() == 0
                && State.get() & MODE_INSERT != 0
                && (p_timeout.get() != 0 || (keylen == KEYLEN_PART_KEY && p_ttimeout.get() != 0))
                && {
                    c = inchar(tb.tail(), 3, 25);
                    c == 0
                }
                && esc_leaves_insert(&mut at)
            {
                *mode_deleted = true;
            }
            if c < 0 {
                continue; // end of the input script reached
            }

            // Allow mapping for the characters just typed. `c` is the number
            // of extra bytes and `tb_len` is 1.
            for n in 1..=c {
                tb.set_noremap(n, RM_YES as u8);
            }
            tb.grow(c);

            if tb.len() >= tb.maplen() + MAXMAPLEN as c_int {
                // The buffer is full, so don't map.
                *timedout = true;
                continue;
            }

            if ex_normal_busy.get() > 0 {
                /// The key the previous forced answer used, so that the
                /// cmdline window alternates between ESC and CTRL-C.
                static tc: GlobalCell<c_int> = GlobalCell::new(0);

                // No typeahead left and inside `:normal`: something has to be
                // answered to avoid getting stuck. With an incomplete mapping
                // present, behave as if it timed out.
                if !tb.is_empty() {
                    *timedout = true;
                    continue;
                }

                // On the command line only CTRL-C breaks it; for the cmdline
                // window alternate between ESC (for most situations) and
                // CTRL-C (which closes the window).
                let c = if State.get() & MODE_CMDLINE != 0
                    || (cmdwin_type.get() > 0 && tc.get() == ESC)
                {
                    Ctrl_C
                } else {
                    ESC
                };
                tc.set(c);

                // A flag saying this was not a normal character.
                if advance {
                    typebuf_was_empty.set(true);
                }
                // Answer 0 in normal_check().
                if pending_exmode_active.get() {
                    exmode_active.set(true);
                }
                // No characters to block abbreviations for.
                tb.set_no_abbr_cnt(0);
                return c;
            }

            // In Insert mode a screen update is skipped while characters are
            // still available. But when those characters are part of a
            // mapping we are about to block here, so the changed text has to
            // be shown. Same for a redraw 'lazyredraw' postponed because
            // there was something in the input buffer (a termresponse, say).
            if (State.get() & MODE_INSERT != 0 || p_lz.get() != 0)
                && State.get() & MODE_CMDLINE == 0
                && advance
                && must_redraw.get() != 0
                && !need_wait_return.get()
            {
                update_screen();
                setcursor(); // put the cursor back where it belongs
            }

            let partial = if !tb.is_empty() && advance && !exmode_active.get() {
                show_partial_key(at)
            } else {
                Partial {
                    showcmd_idx: 0,
                    showing: false,
                }
            };

            if tb.is_empty() {
                // `timedout` may have been set when a mapping with an empty
                // RHS fully matched while longer mappings timed out.
                *timedout = false;
            }

            let wait_tb_len = tb.len();
            c = inchar(tb.tail(), tb.room(), wait_time_for(advance, keylen));

            unshow_partial_key(&partial);

            if c < 0 {
                continue; // end of the input script reached
            }
            if c == NUL {
                // No character is available.
                if !advance {
                    return NUL;
                }
                if wait_tb_len > 0 {
                    *timedout = true; // timed out
                    continue;
                }
            } else {
                // Allow mapping for the characters just typed.
                while c_int::from(*tb.tail()) != NUL {
                    tb.set_noremap(tb.len(), RM_YES as u8);
                    tb.grow(1);
                }
            }
        }
    }
}

/// Get a byte from the stuff buffer, the typeahead buffer, or the user.
///
/// With `advance` (what `vgetc` wants) the byte is really consumed, `KeyTyped`
/// is set when the user typed it and `KeyStuffed` when it came from the stuff
/// buffer; without it (what `vpeekc` wants) this only looks, and answers `NUL`
/// when there is nothing.
///
/// Mappings are checked when the global `no_mapping` is zero. Only one byte of
/// a multibyte character comes back, and a `K_SPECIAL` may be escaped — two
/// more bytes have to be fetched then.
///
/// # Safety
/// Callable at any time; may block waiting for input when `advance` is set.
pub(crate) unsafe fn vgetorpeek(advance: bool) -> c_int {
    unsafe {
        // This function does not work well when called recursively, which can
        // happen because `add_to_showcmd` uses `char_avail`, and because a UI
        // callback that writes to the screen can raise a `wait_return`. Using
        // `:normal` can do it too, but that saves the typeahead buffer, so it
        // is allowed -- it just must not read a key from the user.
        if vgetc_busy.get() > 0 && ex_normal_busy.get() == 0 {
            return NUL;
        }
        vgetc_busy.set(vgetc_busy.get() + 1);

        if advance {
            KeyStuffed.set(0);
            typebuf_was_empty.set(false);
        }

        let mut timedout = false; // waited longer than 'timeoutlen' for a
        // mapping to complete, or 'ttimeoutlen' for a key code
        let mut mapdepth = 0; // recursive mapping check
        let mut mode_deleted = false; // the mode message has been taken down

        init_typebuf();
        start_stuff();
        check_end_reg_executing(advance);

        let c = loop {
            // Get a character: 1. from the stuff buffer.
            let c = if typeahead_char.get() != 0 {
                let c = typeahead_char.get();
                if advance {
                    typeahead_char.set(0);
                }
                c
            } else {
                read_readbuffers(advance)
            };

            let c = if c != NUL && !got_int.get() {
                if advance {
                    // KeyTyped is deliberately left alone: when the command
                    // that stuffed something was typed, behave as if the
                    // stuffed command was typed. Needed for CTRL-W CTRL-] to
                    // open a fold, for example.
                    KeyStuffed.set(1);
                }
                if typeahead().no_abbr_cnt() == 0 {
                    typeahead().set_no_abbr_cnt(1); // no abbreviations now
                }
                c
            } else {
                read_from_typeahead(advance, &mut timedout, &mut mapdepth, &mut mode_deleted)
            };

            // With `advance` false, don't loop on NULs.
            if !(c < 0 || (advance && c == NUL)) {
                break c;
            }
        };

        // The "INSERT" message is taken care of here: if an ESC is answered
        // to leave Insert mode the message is deleted, and if we do not
        // answer an ESC but deleted the message before, it is redisplayed.
        if advance && p_smd.get() != 0 && msg_silent.get() == 0 && State.get() & MODE_INSERT != 0 {
            if c == ESC && !mode_deleted && no_mapping.get() == 0 && mode_displayed.get() {
                if !typeahead().is_empty() && !KeyTyped.get() {
                    redraw_cmdline.set(true); // delete the mode later
                } else {
                    unshowmode(false);
                }
            } else if c != ESC && mode_deleted {
                if !typeahead().is_empty() && !KeyTyped.get() {
                    redraw_cmdline.set(true); // show the mode later
                } else {
                    showmode();
                }
            }
        }

        if timedout && c == ESC {
            // When recording there is no timeout. Add an <Ignore> after the
            // ESC so that it cannot form a key code with what follows.
            gotchars_ignore();
        }

        vgetc_busy.set(vgetc_busy.get() - 1);
        c
    }
}

/// Read up to `maxlen` bytes from a script file or the keyboard into `buf`.
///
/// `buf` must have room for `maxlen + 1` bytes and is NUL-terminated;
/// `maxlen` must be at least 3, because `fix_input_buffer` can triple a byte.
/// `wait_time` is in milliseconds: 0 does not wait, -1 waits forever.
///
/// Answers the number of bytes obtained, or -1 at the end of an input script
/// — which is a distinct answer because closing the script frees
/// `typebuf.tb_buf`, and `buf` may point inside it.
///
/// # Safety
/// `buf` must point at `maxlen + 1` writable bytes.
pub(crate) unsafe fn inchar(buf: *mut u8, maxlen: c_int, wait_time: c_long) -> c_int {
    unsafe {
        let mut len = 0;
        let mut retesc = false; // answer ESC with got_int
        let tb_change_cnt = typeahead().change_cnt();

        if wait_time == -1 || wait_time > 100 {
            ui_flush(); // flush output before waiting
        }

        // Don't reset these at the hit-return prompt, or an endless recursion
        // can result: write error in swapfile, hit-return, timeout on the
        // character wait, flush swapfile, write error, ...
        if State.get() != MODE_HITRETURN {
            did_outofmem_msg.set(false); // display the out-of-memory message again
            did_swapwrite_msg.set(false); // display the swap-write error again
        }

        // Get a character from a script file if there is one. On an
        // interrupt, stop reading script files and close them all.
        let mut read_size: ptrdiff_t = -1;
        while curscript.get() >= 0 && read_size <= 0 && !ignore_script.get() {
            let mut script_char: c_char = 0;
            // Short-circuit deliberately: once interrupted, no further read
            // is attempted and `read_size` keeps the value it had.
            let failed = got_int.get() || {
                read_size = file_read(script_at(curscript.get()), &raw mut script_char, 1);
                read_size != 1
            };
            if failed {
                // EOF, or some error. Careful: closescript() frees
                // typebuf.tb_buf and buf may point inside it, so buf must not
                // be used after this.
                closescript();
                if got_int.get() {
                    // Reading the script was interrupted: answer an ESC to
                    // get back to Normal mode.
                    retesc = true;
                } else {
                    // Otherwise -1, because typebuf.tb_buf has changed.
                    return -1;
                }
            } else {
                *buf = script_char as u8;
                len = 1;
            }
        }

        if read_size <= 0 {
            // Nothing came from a script.
            //
            // On an interrupt, skip everything typed so far and answer
            // whether reading a script file was quit. Don't use buf here:
            // closescript() may have freed typebuf.tb_buf and buf may point
            // inside it.
            if got_int.get() {
                const DUM_LEN: usize = MAXMAPLEN as usize * 3 + 3;
                let mut dum = [0u8; DUM_LEN + 1];
                loop {
                    let got = input_get(
                        dum.as_mut_ptr(),
                        DUM_LEN as c_int,
                        0,
                        0,
                        ptr::null_mut::<MultiQueue>(),
                    );
                    if got == 0 || (got == 1 && c_int::from(dum[0]) == Ctrl_C) {
                        break;
                    }
                }
                return c_int::from(retesc);
            }

            // Always flush the output when reading input from the user, as
            // opposed to just peeking.
            if wait_time == -1 || wait_time > 10 {
                ui_flush();
            }

            // Fill up to a third of the buffer: `fix_input_buffer` can triple
            // each character.
            len = input_get(
                buf,
                maxlen / 3,
                wait_time as c_int,
                tb_change_cnt,
                ptr::null_mut::<MultiQueue>(),
            );
        }

        // If the typeahead was changed further down it is as if nothing was
        // added by this call.
        if typebuf_changed(tb_change_cnt) {
            return 0;
        }

        // Note the change in the typeahead buffer; this matters for when
        // vgetorpeek() is called recursively, e.g. `getchar(1)` in a timer.
        if len > 0 {
            typeahead().note_change();
        }

        fix_input_buffer(buf, len)
    }
}

/// Escape the bytes that cannot appear literally in the typeahead, and answer
/// the new length.
///
/// Only input read from a *script* needs this: keys from the user have
/// already been through `input_enqueue`, which escapes them. `buf` must have
/// room to triple the number of bytes.
///
/// # Safety
/// `buf` must point at `len` readable and `3 * len + 1` writable bytes.
pub unsafe fn fix_input_buffer(buf: *mut u8, mut len: c_int) -> c_int {
    unsafe {
        if using_script() == 0 {
            // K_SPECIAL should not be escaped for input from the user: the
            // key codes are processed in input.c/input_enqueue.
            *buf.offset(len as isize) = 0;
            return len;
        }

        // Two bytes are special: NUL and K_SPECIAL. Both are replaced by
        // their three-byte escape.
        let mut p = buf;
        let mut i = len;
        while i > 0 {
            i -= 1;
            if c_int::from(*p) == NUL
                || (c_int::from(*p) == K_SPECIAL && (i < 2 || c_int::from(*p.add(1)) != KS_EXTRA))
            {
                ptr::copy(p.add(1), p.add(3), i as usize);
                let escape = key_escape(c_int::from(*p));
                *p = escape[0];
                *p.add(1) = escape[1];
                *p.add(2) = escape[2];
                p = p.add(2);
                len += 2;
            }
            p = p.add(1);
        }
        *p = 0; // add the trailing NUL
        len
    }
}
