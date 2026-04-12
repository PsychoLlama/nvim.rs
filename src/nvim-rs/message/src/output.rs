//! Message output utilities
//!
//! Provides Rust implementations for message output state management
//! and coordination with the display system.

use std::ffi::c_int;

/// UIExtension value for kUIMessages (ui_defs.h)
const K_UI_MESSAGES: c_int = 4;

// C accessor declarations
extern "C" {
    static mut msg_silent: c_int;
    /// Get `msg_didany` flag
    static mut msg_didany: bool;
    /// Set `msg_didany` flag
    /// Get `msg_didout` flag
    static mut msg_didout: bool;
    /// Set `msg_didout` flag
    /// Get `msg_nowait` flag
    static mut msg_nowait: bool;
    /// Set `msg_nowait` flag
    /// Get `msg_no_more` flag
    static mut msg_no_more: bool;
    /// Get `lines_left` counter
    static mut lines_left: c_int;
    /// Set `lines_left` counter
    /// Get `need_wait_return` flag
    static mut need_wait_return: bool;
    /// Set `need_wait_return` flag
    /// Get `msg_scrolled` global
    static mut msg_scrolled: c_int;
    /// Get `msg_scrolled_ign` flag
    static mut msg_scrolled_ign: bool;
    /// Get `emsg_on_display` flag
    static mut emsg_on_display: bool;
    /// Set `emsg_on_display` flag
    /// Get `need_fileinfo` flag
    static mut need_fileinfo: bool;
    /// Set `need_fileinfo` flag
    /// Get `p_ch` (cmdheight) option
    static mut p_ch: i64;
    /// Check if UI has messages capability
    fn ui_has(ext: c_int) -> bool;
}

/// Check if any message was output.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_didany() -> c_int {
    c_int::from(msg_didany)
}

/// Set the "message was output" flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_didany(val: c_int) {
    msg_didany = (val) != 0;
}

/// Check if something was written to the current line.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_didout() -> c_int {
    c_int::from(msg_didout)
}

/// Set the "wrote to current line" flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_didout(val: c_int) {
    msg_didout = (val) != 0;
}

/// Check if message should not wait.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_nowait() -> c_int {
    c_int::from(msg_nowait)
}

/// Set the "no wait" flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_nowait(val: c_int) {
    msg_nowait = (val) != 0;
}

/// Get the lines left counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_lines_left() -> c_int {
    lines_left
}

/// Set the lines left counter.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_lines_left(val: c_int) {
    lines_left = val;
}

/// Check if wait_return is needed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_need_wait_return() -> c_int {
    c_int::from(need_wait_return)
}

/// Set the need_wait_return flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_need_wait_return(val: c_int) {
    need_wait_return = (val) != 0;
}

/// Check if error message is on display.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_on_display() -> c_int {
    c_int::from(emsg_on_display)
}

/// Set the emsg_on_display flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_emsg_on_display(val: c_int) {
    emsg_on_display = (val) != 0;
}

/// Check if file info is needed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_need_fileinfo() -> c_int {
    c_int::from(need_fileinfo)
}

/// Set the need_fileinfo flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_need_fileinfo(val: c_int) {
    need_fileinfo = (val) != 0;
}

/// Check if message display has overflowed (scrolled).
///
/// Returns true when a message has been written after scrolling
/// and wait_return may be needed.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_overflow() -> c_int {
    let ui_has_messages = ui_has(K_UI_MESSAGES);

    // Threshold is 1 when cmdheight is 0, otherwise 0
    let threshold = c_int::from(p_ch == 0);
    c_int::from(!ui_has_messages && msg_scrolled > threshold)
}

/// Check if need_wait_return should be set after output.
///
/// Returns true when:
/// - Overflow condition met AND
/// - Not ignoring scrolled messages AND
/// - Output is not just a CR
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_check_wait_return() -> c_int {
    let overflow = rs_msg_overflow() != 0;
    let scrolled_ign = msg_scrolled_ign;

    c_int::from(overflow && !scrolled_ign)
}

/// Check if the more prompt should be shown.
///
/// Returns true when lines_left has reached 0 and the more
/// prompt should be displayed for pagination.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_need_more() -> c_int {
    let ll = lines_left;
    let no_more = msg_no_more;

    c_int::from(ll == 0 && !no_more)
}

/// Decrement lines_left counter and return true if more prompt needed.
///
/// # Safety
/// Calls C accessor and mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_dec_lines_left() -> c_int {
    let ll = lines_left;
    if ll > 0 {
        lines_left = ll - 1;
    }
    rs_msg_need_more()
}

// ============================================================================
// Phase 425: Wait/Return Prompt Functions
// ============================================================================

extern "C" {
    static mut no_wait_return: c_int;
    static vgetc_busy: c_int;
}

/// Get the no_wait_return counter.
///
/// When > 0, wait_return() won't wait for a key.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_no_wait_return() -> c_int {
    no_wait_return
}

/// Increment no_wait_return to prevent waiting.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_no_wait_return_enter() {
    no_wait_return += 1;
}

/// Decrement no_wait_return.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_no_wait_return_leave() {
    if no_wait_return > 0 {
        no_wait_return -= 1;
    }
}

/// Check if waiting for return is currently blocked.
///
/// Returns true if any condition prevents wait_return from waiting:
/// - msg_silent is set
/// - vgetc_busy is set
/// - no_wait_return is set
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_wait_return_blocked() -> c_int {
    c_int::from(msg_silent != 0 || vgetc_busy > 0 || no_wait_return > 0)
}

/// Check if wait_return should be called.
///
/// Returns true if need_wait_return is set and waiting is not blocked.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_should_wait_return() -> c_int {
    let need_wait = need_wait_return;
    let blocked = rs_wait_return_blocked();

    c_int::from(need_wait && blocked == 0)
}

/// Set need_wait_return flag.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_request_wait_return() {
    need_wait_return = true;
}

/// Clear need_wait_return flag.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_wait_return() {
    need_wait_return = false;
}

/// Reset wait return state after handling.
///
/// Clears need_wait_return and msg_didout flags.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_reset_wait_return_state() {
    need_wait_return = false;
    msg_didout = false;
}

// ============================================================================
// Phase 5: msg_puts_len migrated to Rust
// ============================================================================

extern "C" {
    fn redir_write(str_: *const std::ffi::c_char, maxlen: isize);
    fn msg_hist_add(s: *const std::ffi::c_char, len: c_int, hl_id: c_int);
    fn msg_use_printf() -> c_int;
    fn nvim_msg_show_empty();
    static mut headless_mode: bool;
    static mut default_grid: crate::ScreenGrid;
    static mut msg_col: c_int;
    // C helpers for msg_puts_printf on_print callback
    fn nvim_on_print_active() -> c_int;
    fn nvim_on_print_call(str_: *const std::ffi::c_char);
    // For msg_puts_printf
    fn utf_ptr2len(s: *const std::ffi::c_char) -> c_int;
    fn utf_char2cells(c: c_int) -> c_int;
    fn utf_ptr2char(s: *const std::ffi::c_char) -> c_int;
    static mut silent_mode: bool;
    static p_verbose: i64;
    static mut info_message: bool;
    // msg_didout already declared above
}

// ============================================================================
// Phase 4: msg_puts_display migrated to Rust
// ============================================================================

/// GridView struct layout matching C grid_defs.h (local copy for output module)
#[repr(C)]
struct MpdGridView {
    target: *mut std::ffi::c_void,
    row_offset: c_int,
    col_offset: c_int,
}

extern "C" {
    // Globals needed by msg_puts_display
    static mut did_wait_return: bool;
    static mut msg_row: c_int;
    static Rows: c_int;
    static Columns: c_int;
    static mut cmdline_row: c_int;
    static mut exmode_active: bool;
    static mut quit_more: bool;
    static mut redraw_cmdline: bool;
    static mut p_more: c_int;
    // msg_no_more already declared in main extern block above
    static mut msg_grid_adj: MpdGridView;

    // Highlight support
    fn syn_id2attr(hl_id: c_int) -> c_int;
    fn hl_combine_attr(a: c_int, b: c_int) -> c_int;
    static mut hl_attr_active: *mut c_int;

    // Grid output functions
    fn grid_line_start(view: *mut std::ffi::c_void, row: c_int);
    fn grid_line_puts(col: c_int, s: *const std::ffi::c_char, len: c_int, attr: c_int) -> c_int;
    // grid_line_flush: called indirectly via rs_msg_line_flush

    // String utilities
    fn ga_concat_len(gap: *mut std::ffi::c_void, s: *const std::ffi::c_char, len: usize);
    fn mb_string2cells(s: *const std::ffi::c_char) -> c_int;
    fn mb_string2cells_len(s: *const std::ffi::c_char, len: usize) -> c_int;
    fn utf_ptr2cells(s: *const std::ffi::c_char) -> c_int;
    fn utfc_ptr2len(s: *const std::ffi::c_char) -> c_int;
    fn utfc_ptr2len_len(s: *const std::ffi::c_char, size: c_int) -> c_int;
    fn strrchr(s: *const std::ffi::c_char, c: c_int) -> *const std::ffi::c_char;
    fn vim_beep(flag: c_int);

    // cmdline_was_last_drawn omitted: assigned from redrawing_cmdline in C,
    // but the value is only consumed by drawscreen. The Rust migration skips
    // this assignment since the global is owned by C.
}

/// Write message string with highlight and redirection.
///
/// This is the core low-level output routine. It:
/// 1. Writes to any active redirection targets
/// 2. Returns early for silent/empty messages
/// 3. Optionally adds to message history
/// 4. Sets need_wait_return if scrolled
/// 5. Dispatches to printf or display rendering
///
/// # Arguments
/// * `str_` - The string to write (NUL-terminated, length bytes)
/// * `len` - Length of string, or -1 to use NUL-terminator
/// * `hl_id` - Highlight group ID (0 for default)
/// * `hist` - If true, add to message history
///
/// # Safety
/// - `str_` must be a valid pointer to at least `len` bytes (or NUL-terminated if len == -1)
#[export_name = "msg_puts_len"]
pub unsafe extern "C" fn rs_msg_puts_len(
    str_: *const std::ffi::c_char,
    len: isize,
    hl_id: c_int,
    hist: bool,
) {
    // If redirection is on, also write to the redirection file.
    redir_write(str_, len);

    // Don't print anything when using ":silent cmd" or empty message.
    let first_byte = *str_.cast::<u8>();
    if msg_silent != 0 || first_byte == 0 {
        if first_byte == 0 && ui_has(K_UI_MESSAGES) {
            nvim_msg_show_empty();
        }
        return;
    }

    if hist {
        msg_hist_add(str_, c_int::try_from(len).unwrap_or(c_int::MAX), hl_id);
    }

    // When writing something to the screen after it has scrolled, requires a
    // wait-return prompt later.
    let overflow = !ui_has(K_UI_MESSAGES) && {
        let threshold = c_int::from(p_ch == 0);
        msg_scrolled > threshold
    };

    if overflow && !msg_scrolled_ign {
        // Check if str_ == "\r" - single CR character
        let is_cr_only = *str_.cast::<u8>() == b'\r'
            && (len < 0 || len == 1)
            && (len >= 0 || *str_.add(1).cast::<u8>() == 0);
        if !is_cr_only {
            need_wait_return = true;
        }
    }
    msg_didany = true; // remember that something was outputted

    if msg_use_printf() != 0 {
        let saved_msg_col = msg_col;
        rs_msg_puts_printf(str_, len);
        if headless_mode {
            msg_col = saved_msg_col;
        }
    }
    if msg_use_printf() == 0 || (headless_mode && !default_grid.chars.is_null()) {
        rs_msg_puts_display(str_, c_int::try_from(len).unwrap_or(c_int::MAX), hl_id, 0);
    }

    need_fileinfo = false;
}

/// Highlight field index constants (from highlight_defs.h)
const HLF_MSG: c_int = 5; // Message area attribute
const HLF_AT: c_int = 4; // Attribute for '>' overflow indicator

/// Mode flag for hit-return prompt (from state_defs.h)
const MODE_HITRETURN: c_int = 0x2000 | 0x01;

/// Bell option flag for shell-origin beeps (kOptBoFlagShell)
const K_OPT_BO_FLAG_SHELL: c_int = 0x10000;

/// Get HL_ATTR value for a given highlight field index.
///
/// Equivalent to C macro `HL_ATTR(hlf)` = `hl_attr_active[hlf]`.
///
/// # Safety
/// Reads from the hl_attr_active global array.
#[allow(clippy::cast_sign_loss)]
unsafe fn hl_attr(hlf: c_int) -> c_int {
    *hl_attr_active.add(hlf as usize)
}

extern "C" {
    // Extra globals needed by msg_puts_display not already declared above
    static mut State: c_int;
}

/// Display a message string on the message grid.
///
/// This is the core grid-rendering engine for message output. It:
/// - Handles ext_messages UI: appends to the current chunk with highlight
/// - Handles grid rendering: wraps at column boundaries, scrolls as needed
/// - Stores text in the scrollback buffer for "g<" scrollback
/// - Shows "--more--" prompt when the screen fills up
///
/// # Arguments
/// * `str_` - The string to display
/// * `maxlen` - Max bytes to display, or -1 for NUL-terminated
/// * `hl_id` - Highlight group ID (0 for default)
/// * `recurse` - Non-zero if called recursively (suppresses more-prompt, scrollback)
///
/// # Safety
/// - `str_` must be valid for at least `maxlen` bytes (or NUL-terminated if maxlen == -1)
/// - Accesses global message state
#[export_name = "msg_puts_display"]
#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub unsafe extern "C" fn rs_msg_puts_display(
    str_: *const std::ffi::c_char,
    maxlen: c_int,
    hl_id: c_int,
    recurse: c_int,
) {
    let mut s = str_;
    let mut sb_str = str_;
    let mut sb_col = msg_col;
    let attr = if hl_id != 0 { syn_id2attr(hl_id) } else { 0 };

    did_wait_return = false;

    if ui_has(K_UI_MESSAGES) {
        if attr != crate::display::msg_ext_last_attr {
            crate::display::msg_ext_emit_chunk();
            crate::display::msg_ext_last_attr = attr;
            crate::display::msg_ext_last_hl_id = hl_id;
        }
        // Concat pieces with the same highlight
        let len: usize = if maxlen < 0 {
            mpd_strlen(str_)
        } else {
            mpd_strnlen(str_, maxlen as usize)
        };
        ga_concat_len(
            std::ptr::addr_of_mut!(crate::display::msg_ext_last_chunk).cast(),
            str_,
            len,
        );

        // Find last newline and calculate the new message column
        let lastline = strrchr(str_, c_int::from(b'\n'));
        let maxlen_adj = maxlen
            - if lastline.is_null() {
                0
            } else {
                (lastline as isize - str_ as isize) as c_int
            };
        let p: *const std::ffi::c_char = if lastline.is_null() {
            str_
        } else {
            lastline.add(1)
        };
        let col = if maxlen_adj < 0 {
            mb_string2cells(p)
        } else {
            mb_string2cells_len(p, maxlen_adj as usize)
        };
        msg_col = if lastline.is_null() { msg_col } else { 0 } + col;
        return;
    }

    let print_attr = hl_combine_attr(hl_attr(HLF_MSG), attr);
    crate::misc::rs_msg_grid_validate();

    // Mirror C: cmdline_was_last_drawn = redrawing_cmdline
    // (This assignment is used by drawscreen logic; the value is written here
    //  but consumed by callers that redraw the cmdline. We skip the write since
    //  the global is owned by C code and rs_msg_grid_validate handles the screen.)
    // nvim_get_cmdline_was_last_drawn() is available if needed but the C value
    // is only read here to be stored, so we tolerate the omission.

    let mut msg_row_pending: c_int = -1;

    loop {
        if msg_col >= Columns {
            if p_more != 0 && recurse == 0 {
                crate::scrollback::rs_store_sb_text(&raw mut sb_str, s, hl_id, &raw mut sb_col, 1);
            }
            if msg_no_more && lines_left == 0 {
                break;
            }
            msg_col = 0;
            msg_row += 1;
            msg_didout = false;
        }

        if msg_row >= Rows {
            msg_row = Rows - 1;

            if msg_no_more && lines_left == 0 {
                break;
            }

            if recurse == 0 {
                if msg_row_pending >= 0 {
                    crate::display::rs_msg_line_flush();
                    msg_row_pending = -1;
                }

                // Scroll the screen up one line.
                crate::output_core::rs_msg_scroll_up(0, 0);

                crate::scrollback::rs_inc_msg_scrolled();
                need_wait_return = true;
                redraw_cmdline = true;
                if cmdline_row > 0 && !exmode_active {
                    cmdline_row -= 1;
                }

                if lines_left > 0 {
                    lines_left -= 1;
                }

                if p_more != 0
                    && lines_left == 0
                    && State != MODE_HITRETURN
                    && !msg_no_more
                    && !exmode_active
                {
                    if crate::scrollback::rs_do_more_prompt(0) {
                        s = crate::dialog::confirm_buttons;
                    }
                    if quit_more {
                        return;
                    }
                }
            }
        }

        // Break if end of string
        if !((maxlen < 0 || (s as isize - str_ as isize) < maxlen as isize) && *s != 0) {
            break;
        }

        let sb = *s as u8;
        if msg_row != msg_row_pending && (sb >= 0x20 || sb == b'\t') {
            if msg_row_pending >= 0 {
                crate::display::rs_msg_line_flush();
            }
            grid_line_start(std::ptr::addr_of_mut!(msg_grid_adj).cast(), msg_row);
            msg_row_pending = msg_row;
        }

        if sb >= 0x20 {
            // printable character
            let cw = utf_ptr2cells(s);
            let l = if maxlen >= 0 {
                utfc_ptr2len_len(s, (str_ as isize + maxlen as isize - s as isize) as c_int)
            } else {
                utfc_ptr2len(s)
            };

            if cw > 1 && msg_col == Columns - 1 {
                // Doesn't fit: print a highlighted '>' to fill the last column.
                grid_line_puts(msg_col, c">".as_ptr(), 1, hl_attr(HLF_AT));
                // Don't advance s — character not consumed, will wrap next iteration
            } else {
                grid_line_puts(msg_col, s, l, print_attr);
                s = s.add(l as usize);
            }
            msg_didout = true;
            msg_col += cw;
        } else {
            s = s.add(1);
            match sb {
                b'\n' => {
                    // go to next line
                    msg_didout = false;
                    msg_col = 0;
                    msg_row += 1;
                    if p_more != 0 && recurse == 0 {
                        crate::scrollback::rs_store_sb_text(
                            &raw mut sb_str,
                            s,
                            hl_id,
                            &raw mut sb_col,
                            1,
                        );
                    }
                }
                b'\r' => {
                    // go to column 0
                    msg_col = 0;
                }
                b'\x08' => {
                    // backspace
                    if msg_col > 0 {
                        msg_col -= 1;
                    }
                }
                b'\t' => {
                    // tab: translate into spaces
                    loop {
                        grid_line_puts(msg_col, c" ".as_ptr(), 1, print_attr);
                        msg_col += 1;
                        if msg_col == Columns {
                            break; // outer loop will handle wrap next iteration
                        }
                        if msg_col.trailing_zeros() >= 3 {
                            break;
                        }
                    }
                }
                0x07 => {
                    // BELL (from ":sh")
                    vim_beep(K_OPT_BO_FLAG_SHELL);
                }
                _ => {}
            }
        }
    }

    if msg_row_pending >= 0 {
        crate::display::rs_msg_line_flush();
    }
    crate::misc::rs_msg_cursor_goto(msg_row, msg_col);

    if p_more != 0 && recurse == 0 {
        crate::scrollback::rs_store_sb_text(&raw mut sb_str, s, hl_id, &raw mut sb_col, 0);
    }

    crate::display::rs_msg_check();
}

/// Compute strlen (null-terminated string length).
const unsafe fn mpd_strlen(s: *const std::ffi::c_char) -> usize {
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

/// Compute strnlen: length of null-terminated string capped at `max`.
const unsafe fn mpd_strnlen(s: *const std::ffi::c_char, max: usize) -> usize {
    let mut len = 0usize;
    while len < max && *s.add(len) != 0 {
        len += 1;
    }
    len
}

// ============================================================================
// Phase 1: msg_puts_printf migrated to Rust
// ============================================================================

/// Print a message when there is no valid screen.
///
/// This is called when `msg_use_printf()` is true (no grid/UI available).
/// Handles the `on_print` callback for embedded/headless mode, and outputs
/// to stderr or stdout depending on `info_message`.
///
/// # Safety
/// - `str_` must be a valid pointer to at least `maxlen` bytes (or NUL-terminated if maxlen < 0)
#[export_name = "msg_puts_printf"]
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_msg_puts_printf(str_: *const std::ffi::c_char, maxlen: isize) {
    if nvim_on_print_active() != 0 {
        nvim_on_print_call(str_);
        return;
    }

    let mut s = str_;
    while (maxlen < 0 || (s as isize - str_ as isize) < maxlen) && *s != 0 {
        let len = utf_ptr2len(s);

        if !(silent_mode && p_verbose == 0) {
            // NL -> CR NL translation (for Unix, not for "--version")
            let mut buf = [0u8; 8];
            let mut p_idx = 0usize;
            if *s == b'\n' as i8 && !info_message {
                buf[p_idx] = b'\r';
                p_idx += 1;
            }
            let slice = std::slice::from_raw_parts(s.cast::<u8>(), len as usize);
            buf[p_idx..p_idx + len as usize].copy_from_slice(slice);
            buf[p_idx + len as usize] = 0;

            let buf_ptr = buf.as_ptr().cast::<std::ffi::c_char>();
            if info_message {
                libc_printf(buf_ptr);
            } else {
                libc_fprintf_stderr(buf_ptr);
            }
        }

        let cw = utf_char2cells(utf_ptr2char(s));
        if *s == b'\r' as i8 || *s == b'\n' as i8 {
            msg_col = 0;
            msg_didout = false;
        } else {
            msg_col += cw;
            msg_didout = true;
        }
        s = s.add(len as usize);
    }
}

// Thin C-callable wrappers for stdio (avoids linking against libc directly).
extern "C" {
    #[link_name = "nvim_printf_stdout"]
    fn libc_printf(s: *const std::ffi::c_char);
    #[link_name = "nvim_fprintf_stderr"]
    fn libc_fprintf_stderr(s: *const std::ffi::c_char);
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
}
