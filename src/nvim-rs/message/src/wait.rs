//! wait_return implementation
//!
//! Implements `wait_return()`: the hit-enter prompt that pauses output
//! and waits for the user to press a key.

use std::ffi::{c_int, c_void};

// ============================================================================
// Key code constants
// ============================================================================

// TERMCAP2KEY(a, b) = -((a) + ((b as i32) << 8))
// KS_EXTRA = 253

const K_UP: c_int = -30059; // TERMCAP2KEY('k', 'u')
const K_DOWN: c_int = -25707; // TERMCAP2KEY('k', 'd')
const K_PAGEUP: c_int = -20587; // TERMCAP2KEY('k', 'P')
const K_PAGEDOWN: c_int = -20075; // TERMCAP2KEY('k', 'N')

// TERMCAP2KEY(KS_EXTRA, KE_xxx) where KS_EXTRA = 253
const K_IGNORE: c_int = -((253) + (53_i32 << 8)); // KE_IGNORE = 53
const K_LEFTMOUSE: c_int = -((253) + (44_i32 << 8)); // KE_LEFTMOUSE = 44
const K_MIDDLEMOUSE: c_int = -((253) + (47_i32 << 8)); // KE_MIDDLEMOUSE = 47
const K_RIGHTMOUSE: c_int = -((253) + (50_i32 << 8)); // KE_RIGHTMOUSE = 50
const K_X1MOUSE: c_int = -((253) + (89_i32 << 8)); // KE_X1MOUSE = 89
const K_X2MOUSE: c_int = -((253) + (92_i32 << 8)); // KE_X2MOUSE = 92
const K_LEFTDRAG: c_int = -((253) + (45_i32 << 8)); // KE_LEFTDRAG = 45
const K_LEFTRELEASE: c_int = -((253) + (46_i32 << 8)); // KE_LEFTRELEASE = 46
const K_MIDDLEDRAG: c_int = -((253) + (48_i32 << 8)); // KE_MIDDLEDRAG = 48
const K_MIDDLERELEASE: c_int = -((253) + (49_i32 << 8)); // KE_MIDDLERELEASE = 49
const K_RIGHTDRAG: c_int = -((253) + (51_i32 << 8)); // KE_RIGHTDRAG = 51
const K_RIGHTRELEASE: c_int = -((253) + (52_i32 << 8)); // KE_RIGHTRELEASE = 52
const K_MOUSELEFT: c_int = -((253) + (77_i32 << 8)); // KE_MOUSELEFT = 77
const K_MOUSERIGHT: c_int = -((253) + (78_i32 << 8)); // KE_MOUSERIGHT = 78
const K_MOUSEDOWN: c_int = -((253) + (75_i32 << 8)); // KE_MOUSEDOWN = 75
const K_MOUSEUP: c_int = -((253) + (76_i32 << 8)); // KE_MOUSEUP = 76
const K_MOUSEMOVE: c_int = -((253) + (100_i32 << 8)); // KE_MOUSEMOVE = 100

const CAR: c_int = 0x0d; // carriage return
const CTRL_B: c_int = 0x02;
const CTRL_C: c_int = 0x03;
const CTRL_F: c_int = 0x06;

/// Mode flags (state_defs.h)
const MODE_HITRETURN: c_int = 0x2000 | 0x01;
const MODE_SETWSIZE: c_int = 0x4000;

/// UIExtension value for kUIMessages
const K_UI_MESSAGES: c_int = 4;

/// Highlight field for "--More--" message (HLF_M = 10)
const HLF_M: c_int = 10;

/// MOUSE_SETPOS flag (mouse.h)
const MOUSE_SETPOS: c_int = 0x08;

/// Redraw type constants (pos_defs.h)
const UPD_NOT_VALID: c_int = 40;
const UPD_VALID: c_int = 10;

/// messagesopt flag: hit-enter prompt (kOptMoptFlagHitEnter)
const K_OPT_MOPT_FLAG_HIT_ENTER: c_int = 0x01;

// ============================================================================
// Extern declarations
// ============================================================================

extern "C" {
    fn redraw_all_later(r#type: c_int);
    fn ui_has(ext: c_int) -> bool;
    fn prompt_for_input(
        prompt: *mut std::ffi::c_char,
        hl_id: c_int,
        one_key: bool,
        mouse_used: *mut bool,
    ) -> c_int;
    static mut msg_silent: c_int;
    static mut headless_mode: bool;
    fn ui_active() -> c_int;
    static mut vgetc_busy: c_int;
    static mut need_wait_return: bool;
    static mut no_wait_return: c_int;
    static mut exmode_active: bool;
    static mut cmdline_row: c_int;
    static mut msg_row: c_int;
    static mut redir_off: bool;
    static mut State: c_int;
    static mut quit_more: bool;
    static mut got_int: bool;
    fn msg_puts(s: *const std::ffi::c_char);
    fn setmouse();
    static mut need_check_timestamps: bool;
    fn check_timestamps(focus: bool);
    static mut p_ch: i64;
    fn msg_scroll_up(may_throttle: bool, zerocmd: bool);
    static mut msg_scrolled: c_int;
    static Rows: c_int;
    static Columns: c_int;
    fn hit_return_msg(newline_sb: bool);
    fn nvim_inc_no_mapping();
    fn nvim_dec_no_mapping();
    fn nvim_inc_allow_keys();
    fn nvim_dec_allow_keys();
    static mut reg_recording: c_int;
    static mut scriptout: *mut c_void; // FILE* treated as opaque
    fn safe_vgetc() -> c_int;
    static mut global_busy: c_int;
    static mut p_more: c_int;
    static mut msg_didout: bool;
    static mut msg_col: c_int;
    fn os_breakcheck();
    fn jump_to_mouse(flags: c_int, inclusive: *mut bool, which_button: c_int) -> c_int;
    fn vim_strchr(string: *const std::ffi::c_char, c: c_int) -> *mut std::ffi::c_char;
    static mut vgetc_char: c_int;
    static mut vgetc_mod_mask: c_int;
    fn ins_char_typebuf(c: c_int, modifiers: c_int, on_key_ignore: bool) -> c_int;
    static mut do_redraw: bool;
    fn do_sleep(ms: c_int, allow_int: bool);
    static mut did_wait_return: bool;
    static mut emsg_on_display: bool;
    static mut lines_left: c_int;
    static mut keep_msg: *mut std::ffi::c_char;
    fn vim_strsize(s: *const std::ffi::c_char) -> c_int;
    static mut sc_col: c_int;
    fn xfree(ptr: *mut c_void);
    fn ui_refresh();
    static mut skip_redraw: bool;
    fn redraw_later(wp: *mut c_void, r#type: c_int);
    static mut curwin: *mut c_void;
    fn msg_check();
}

/// Wait for the user to hit a key (normally Enter).
///
/// `redraw` controls post-wait redrawing:
/// - true (1): redraw the entire screen (`UPD_NOT_VALID`)
/// - false (0): do a normal redraw
/// - -1: don't redraw at all
///
/// # Safety
/// Accesses global Neovim state and blocks for user input.
#[export_name = "wait_return"]
#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub unsafe extern "C" fn rs_wait_return(redraw: c_int) {
    if redraw == 1 {
        redraw_all_later(UPD_NOT_VALID);
    }

    if ui_has(K_UI_MESSAGES) {
        prompt_for_input(
            c"Press any key to continue".as_ptr().cast_mut(),
            HLF_M,
            true,
            std::ptr::null_mut(),
        );
        return;
    }

    // If using ":silent cmd", don't wait for a return.
    if msg_silent != 0 {
        return;
    }

    if headless_mode && ui_active() == 0 {
        return;
    }

    // When inside vgetc(), we can't wait for a typed character at all.
    if vgetc_busy > 0 {
        return;
    }

    need_wait_return = true;
    if no_wait_return != 0 {
        if !exmode_active {
            cmdline_row = msg_row;
        }
        return;
    }

    redir_off = true; // don't redirect this message
    let old_state = State;
    let c: c_int;

    if quit_more {
        c = CAR; // just pretend CR was hit
        quit_more = false;
        got_int = false;
    } else if exmode_active {
        msg_puts(c" ".as_ptr()); // make sure the cursor is on the right line
        c = CAR; // no need for a return in ex mode
        got_int = false;
    } else {
        State = MODE_HITRETURN;
        setmouse();
        cmdline_row = msg_row;

        // Avoid the file-changed dialog interrupting the hit-return sequence.
        if need_check_timestamps {
            check_timestamps(false);
        }

        // if cmdheight=0, scroll the first line of msg_grid onto the screen
        if p_ch == 0 && !ui_has(K_UI_MESSAGES) && msg_scrolled == 0 {
            crate::misc::rs_msg_grid_validate();
            msg_scroll_up(false, true);
            msg_scrolled += 1;
            cmdline_row = Rows - 1;
        }

        c = if crate::misc::msg_flags & K_OPT_MOPT_FLAG_HIT_ENTER != 0 {
            hit_return_msg(true);

            let mut c_inner: c_int;
            loop {
                let had_got_int = got_int;

                // Don't do mappings; put the character back in typeahead.
                nvim_inc_no_mapping();
                nvim_inc_allow_keys();

                // Temporarily disable Recording.
                let save_reg_recording = reg_recording;
                let save_scriptout = scriptout;
                reg_recording = 0;
                scriptout = std::ptr::null_mut();
                c_inner = safe_vgetc();
                if had_got_int && global_busy == 0 {
                    got_int = false;
                }
                nvim_dec_no_mapping();
                nvim_dec_allow_keys();
                reg_recording = save_reg_recording;
                scriptout = save_scriptout;

                // Allow scrolling back in the messages.
                if p_more != 0 {
                    if c_inner == c_int::from(b'b')
                        || c_inner == CTRL_B
                        || c_inner == c_int::from(b'k')
                        || c_inner == c_int::from(b'u')
                        || c_inner == c_int::from(b'g')
                        || c_inner == K_UP
                        || c_inner == K_PAGEUP
                    {
                        if msg_scrolled > Rows {
                            // scroll back to show older messages
                            crate::scrollback::rs_do_more_prompt(c_inner);
                        } else {
                            msg_didout = false;
                            c_inner = K_IGNORE;
                            msg_col = 0;
                        }
                        if quit_more {
                            c_inner = CAR; // just pretend CR was hit
                            quit_more = false;
                            got_int = false;
                        } else if c_inner != K_IGNORE {
                            c_inner = K_IGNORE;
                            hit_return_msg(false);
                        }
                    } else if msg_scrolled > Rows - 2
                        && (c_inner == c_int::from(b'j')
                            || c_inner == c_int::from(b'd')
                            || c_inner == c_int::from(b'f')
                            || c_inner == CTRL_F
                            || c_inner == K_DOWN
                            || c_inner == K_PAGEDOWN)
                    {
                        c_inner = K_IGNORE;
                    }
                }

                let continue_loop = (got_int && c_inner == CTRL_C)
                    || c_inner == K_IGNORE
                    || c_inner == K_LEFTDRAG
                    || c_inner == K_LEFTRELEASE
                    || c_inner == K_MIDDLEDRAG
                    || c_inner == K_MIDDLERELEASE
                    || c_inner == K_RIGHTDRAG
                    || c_inner == K_RIGHTRELEASE
                    || c_inner == K_MOUSELEFT
                    || c_inner == K_MOUSERIGHT
                    || c_inner == K_MOUSEDOWN
                    || c_inner == K_MOUSEUP
                    || c_inner == K_MOUSEMOVE;
                if !continue_loop {
                    break;
                }
            }
            os_breakcheck();

            // Avoid that the mouse-up event causes visual mode to start.
            if c_inner == K_LEFTMOUSE
                || c_inner == K_MIDDLEMOUSE
                || c_inner == K_RIGHTMOUSE
                || c_inner == K_X1MOUSE
                || c_inner == K_X2MOUSE
            {
                jump_to_mouse(MOUSE_SETPOS, std::ptr::null_mut(), 0);
            } else if vim_strchr(c"\r\n ".as_ptr(), c_inner).is_null() && c_inner != CTRL_C {
                // Put the character back in the typeahead buffer.
                ins_char_typebuf(vgetc_char, vgetc_mod_mask, true);
                do_redraw = true; // need a redraw even though there is typeahead
            }
            c_inner
        } else {
            // No hit-enter: wait to allow user to verify the output.
            do_sleep(crate::misc::msg_wait, true);
            CAR
        };
    }

    redir_off = false;

    // If the user hits ':', '?' or '/' we get a command line from the next line.
    if c == c_int::from(b':') || c == c_int::from(b'?') || c == c_int::from(b'/') {
        if !exmode_active {
            cmdline_row = msg_row;
        }
        skip_redraw = true;
        do_redraw = false;
    }

    // If the screen size changed, screen_resize() will redraw the screen.
    let tmp_state = State;
    State = old_state; // restore State before screen_resize()
    setmouse();
    msg_check();
    need_wait_return = false;
    did_wait_return = true;
    emsg_on_display = false; // can delete error message now
    lines_left = -1; // reset lines_left at next msg_start()
    crate::error::rs_reset_last_sourcing();

    if !keep_msg.is_null() && vim_strsize(keep_msg) >= (Rows - cmdline_row - 1) * Columns + sc_col {
        xfree(keep_msg.cast::<c_void>());
        keep_msg = std::ptr::null_mut();
    }

    if tmp_state == MODE_SETWSIZE {
        // got resize event while in vgetc()
        ui_refresh();
    } else if !skip_redraw && (redraw == 1 || (msg_scrolled != 0 && redraw != -1)) {
        redraw_later(curwin, UPD_VALID);
    }
}
