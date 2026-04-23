//! Miscellaneous message functions
//!
//! Provides Rust implementations for various message-related utilities
//! that don't fit neatly into other modules.

use std::ffi::{c_char, c_int, c_void};
use std::ptr::addr_of_mut;

/// UIExtension value for kUIMessages (ui_defs.h)
const K_UI_MESSAGES: c_int = 4;
/// UIExtension value for kUIMultigrid (ui_defs.h)
const K_UI_MULTIGRID: c_int = 6;

// ============================================================================
// C Type Definitions
// ============================================================================

/// C GridView struct layout (grid_defs.h)
#[repr(C)]
struct GridView {
    target: *mut c_void,
    row_offset: c_int,
    col_offset: c_int,
}

// ============================================================================
// Rust-owned statics (previously file-local in message.c)
// ============================================================================

/// Row position when message grid was last flushed (replaces C static msg_grid_pos_at_flush)
#[no_mangle]
pub static mut msg_grid_pos_at_flush: c_int = 0;

/// Whether current message uses multiple highlight regions (replaces C static is_multihl)
#[no_mangle]
pub static mut is_multihl: bool = false;

/// messagesopt flags (replaces C static msg_flags)
/// Default: kOptMoptFlagHitEnter (0x01) | kOptMoptFlagHistory (0x04)
#[no_mangle]
pub static mut msg_flags: c_int = 0x01 | 0x04;

/// messagesopt wait time in ms (replaces C static msg_wait)
#[no_mangle]
pub static mut msg_wait: c_int = 0;

/// Maximum message history entries (replaces C static msg_hist_max)
#[no_mangle]
pub static mut msg_hist_max: c_int = 500;

/// Whether keep_msg was set by msgmore() (replaces C global keep_msg_more)
#[no_mangle]
pub static mut keep_msg_more: bool = false;

// ============================================================================
// C Function Declarations
// ============================================================================

extern "C" {
    static Rows: c_int;
    static Columns: c_int;
    static mut msg_silent: c_int;
    // Home directory handling (Phase 77: now implemented in Rust)
    fn home_replace_save(buf: *mut std::ffi::c_void, src: *const c_char) -> *mut c_char;
    fn msg_outtrans(str: *const c_char, hl_id: c_int, hist: bool) -> c_int;

    // For msg_outtrans_long (Phase 80)
    fn msg_outtrans_len(msgstr: *const c_char, len: c_int, hl_id: c_int, hist: bool) -> c_int;
    fn msg_puts_hl(s: *const c_char, hl_id: c_int, hist: bool);
    static mut msg_col: c_int;

    // Note: msg_source is wrapped in error.rs

    // Phase 4: msg_scroll_flush accessors
    static mut msg_grid: crate::ScreenGrid;
    static mut msg_grid_pos: c_int;
    static mut msg_scrolled: c_int;
    static mut msg_scrolled_at_flush: c_int;
    static mut msg_grid_scroll_discount: c_int;
    // nvim_ui_ext_msg_set_pos is now implemented in Rust (below)
    fn nvim_curwin_get_fcs_msgsep() -> u32;
    fn nvim_ui_call_msg_set_pos_impl(
        handle: c_int,
        row: c_int,
        scrolled: bool,
        buf: *const c_char,
        size: usize,
        zindex: c_int,
        comp_index: c_int,
    );
    fn nvim_schar_get_impl(buf_out: *mut c_char, sc: u32) -> usize;
    fn ui_call_grid_resize(grid: i64, width: i64, height: i64);
    fn ui_call_grid_scroll(
        grid: i64,
        top: i64,
        bot: i64,
        left: i64,
        right: i64,
        rows: i64,
        cols: i64,
    );
    fn ui_line(
        grid: *mut crate::ScreenGrid,
        row: c_int,
        clear_clear: bool,
        start_col: c_int,
        end_col: c_int,
        clear_to_col: c_int,
        bg_attr: c_int,
        wrap: bool,
    );

    // Phase 2: cmdmod filter accessors (replacing nvim_message_filtered_impl)
    fn nvim_cmdmod_has_filter() -> bool;
    fn nvim_cmdmod_vim_regexec(msg: *const c_char) -> bool;
    fn nvim_cmdmod_filter_force() -> bool;

    // For msg_clr_cmdline
    static mut cmdline_row: c_int;
    static mut msg_row: c_int;

    // Cursor positioning (grid_adjust takes *mut c_void = GridView*)
    fn grid_adjust(view: *mut c_void, row_off: *mut c_int, col_off: *mut c_int) -> *mut c_void;
    fn nvim_screengrid_get_handle(grid: *mut c_void) -> c_int;
    fn nvim_ui_grid_cursor_goto(handle: c_int, row: c_int, col: c_int);
    static mut cmdmsg_rl: bool;
    static mut msg_grid_adj: GridView;

    // State accessors
    static mut emsg_on_display: bool;
    static mut msg_scroll: c_int;
    static mut did_wait_return: bool;
    static mut emsg_silent: c_int;
    fn ui_has(ext: c_int) -> bool;
    // ui_flush is defined in change_ffi.c
    fn ui_flush();
    // os_delay is defined in change_ffi.c (long ms, bool allow_input)
    fn os_delay(ms: u64, allow_input: bool);

    // keep_msg state
    static mut keep_msg_hl_id: c_int;
    static mut keep_msg: *mut c_char;
    fn xfree(ptr: *mut std::ffi::c_void);
    fn xstrdup(s: *const c_char) -> *mut c_char;

    // For messaging()
    static mut p_lz: c_int;
    fn nvim_char_avail() -> c_int;
    fn nvim_get_key_typed() -> c_int;
    static mut p_ch: i64;

    // For msg_make
    fn skipwhite(s: *const c_char) -> *mut c_char;
    fn msg_putchar(c: c_int);

    // For messagesopt_changed (Phase 86)
    static mut p_mopt: *mut c_char;
    fn strnequal(s1: *const c_char, s2: *const c_char, n: usize) -> bool;
    fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int;
    fn msg_hist_clear(keep: c_int);

    // For msgmore (Phase 6)
    fn nvim_get_global_busy() -> c_int;
    fn nvim_get_p_report() -> i64;
    // msg_buf is EXTERN char msg_buf[480] in globals.h
    static mut msg_buf: [c_char; 480];
    static mut got_int: bool;
    fn ngettext(s1: *const c_char, s2: *const c_char, n: std::ffi::c_ulong) -> *const c_char;
    fn xstrlcat(dst: *mut c_char, src: *const c_char, maxlen: usize) -> usize;

    // For repeat_message (Phase 22)
    static mut State: c_int;
    static mut msg_didout: bool;
    fn ui_cursor_goto(row: c_int, col: c_int);
    fn msg_clr_eos();

    // For hit_return_msg
    static mut p_more: c_int;
    fn msg_puts(s: *const c_char);
    fn msg_use_printf() -> c_int;

    // For msg_moremsg (Phase 24)
    fn hl_combine_attr(char_attr: c_int, prim_attr: c_int) -> c_int;
    static mut hl_attr_active: *mut c_int;
    fn grid_line_start(view: *mut c_void, row: c_int);
    fn grid_line_puts(col: c_int, s: *const c_char, len: c_int, attr: c_int) -> c_int;
    fn grid_line_cursor_goto(col: c_int);
    fn grid_line_flush();
    fn gettext(s: *const c_char) -> *const c_char;
}

// nvim_get_in_assert_fails returns bool in C (normal_shim.c) but other modules
// declare it as c_int. Use bool here to match the actual signature.
#[allow(clashing_extern_declarations)]
extern "C" {
    static in_assert_fails: bool;
}

// ============================================================================
// Easter Egg
// ============================================================================

/// Show a special message if the argument matches a secret phrase.
///
/// # Safety
/// - `arg` must be a valid NUL-terminated C string
#[export_name = "msg_make"]
pub unsafe extern "C" fn rs_msg_make(arg: *const c_char) {
    const STR: &[u8] = b"eeffoc";
    const RS: &[u8] = b"Plon#dqg#vxjduB";

    let arg = skipwhite(arg).cast_const();
    let mut idx: usize = 5;
    let mut p = arg;
    let mut matched = true;
    loop {
        if *p == 0 {
            break;
        }
        // Compare *p (i8) with STR[idx] (u8) — STR chars are all ASCII (<128)
        #[allow(clippy::cast_possible_wrap)]
        if *p != STR[idx] as i8 {
            matched = false;
            break;
        }
        p = p.add(1);
        if idx == 0 {
            break;
        }
        idx -= 1;
    }
    if matched && idx == 0 && *p == 0 {
        msg_putchar(c_int::from(b'\n'));
        for &b in RS {
            msg_putchar(c_int::from(b) - 3);
        }
    }
}

// ============================================================================
// Output Translation Functions
// ============================================================================

// Note: rs_msg_outtrans() and rs_msg_outtrans_len() are defined in format.rs

/// Highlight face for special characters (HLF_8 = 8)
const HLF_8: c_int = 8;

/// Output a potentially long string with truncation.
///
/// If the string is too long for the screen, shows "..." at the middle.
/// Truncates by showing the start and end with "..." in the middle.
///
/// # Arguments
/// * `longstr` - The string to output
/// * `hl_id` - Highlight group ID
///
/// # Safety
/// - `longstr` must be a valid NUL-terminated C string
#[export_name = "msg_outtrans_long"]
pub unsafe extern "C" fn rs_msg_outtrans_long(longstr: *const c_char, hl_id: c_int) {
    // Calculate strlen
    let mut p = longstr;
    while *p != 0 {
        p = p.offset(1);
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let len = (p as usize - longstr as usize) as c_int;
    let mut slen = len;
    let room = Columns - msg_col;
    if !ui_has(K_UI_MESSAGES) && len > room && room >= 20 {
        slen = (room - 3) / 2;
        msg_outtrans_len(longstr, slen, hl_id, false);
        msg_puts_hl(c"...".as_ptr(), HLF_8, false);
    }
    msg_outtrans_len(longstr.offset((len - slen) as isize), slen, hl_id, false);
}

// ============================================================================
// Path Display Functions
// ============================================================================

/// Display a filename with home directory replaced by ~.
///
/// Replaces the home directory prefix with ~ and outputs with highlight 0.
///
/// # Arguments
/// * `fname` - The filename to display
///
/// # Safety
/// - `fname` must be a valid NUL-terminated C string
#[export_name = "msg_home_replace"]
pub unsafe extern "C" fn rs_msg_home_replace(fname: *const c_char) {
    rs_msg_home_replace_hl(fname, 0);
}

/// Display a filename with home directory replaced by ~ and given highlight.
///
/// # Arguments
/// * `fname` - The filename to display
/// * `hl_id` - Highlight group ID
///
/// # Safety
/// - `fname` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_msg_home_replace_hl(fname: *const c_char, hl_id: c_int) {
    let name = home_replace_save(std::ptr::null_mut(), fname);
    msg_outtrans(name.cast_const(), hl_id, false);
    xfree(name.cast());
}

// ============================================================================
// Source Location Functions
// ============================================================================

// Note: rs_msg_source() is defined in error.rs

// ============================================================================
// Prompt and Delay Functions
// ============================================================================

/// Check if a delay is needed before next message.
///
/// Used to ensure messages are visible before proceeding.
///
/// # Arguments
/// * `check_msg_scroll` - If true, also check msg_scroll state
///
/// # Safety
/// Calls C accessor functions and may block.
#[export_name = "msg_check_for_delay"]
pub unsafe extern "C" fn rs_msg_check_for_delay(check_msg_scroll: c_int) {
    let check = check_msg_scroll != 0;
    if (c_int::from(emsg_on_display) != 0 || (check && msg_scroll != 0))
        && !did_wait_return
        && emsg_silent == 0
        && !in_assert_fails
        && !ui_has(K_UI_MESSAGES)
    {
        ui_flush();
        os_delay(1006, true);
        emsg_on_display = false;
        if check {
            msg_scroll = 0;
        }
    }
}

// ============================================================================
// Keep Message Functions
// ============================================================================

/// Set the "keep_msg" string that is re-displayed after redraw.
///
/// Frees the old value. Skips when ext_messages UI is active.
/// Sets keep_msg_more to false and updates highlight.
///
/// # Arguments
/// * `s` - The message string, or NULL to clear
/// * `hl_id` - Highlight group ID for the message
///
/// # Safety
/// Calls C accessor/mutator functions that manage allocated memory.
#[export_name = "set_keep_msg"]
pub unsafe extern "C" fn rs_set_keep_msg(s: *const c_char, hl_id: c_int) {
    // Kept message is not cleared and re-emitted with ext_messages: #20416.
    if ui_has(K_UI_MESSAGES) {
        return;
    }

    xfree(keep_msg.cast());
    if s.is_null() || msg_silent != 0 {
        keep_msg = std::ptr::null_mut();
    } else {
        keep_msg = xstrdup(s);
    }
    keep_msg_more = false;
    keep_msg_hl_id = hl_id;
}

// ============================================================================
// UI Coordination Functions
// ============================================================================

/// Refresh the message area UI.
///
/// Calls ui_call_grid_resize and ui_ext_msg_set_pos when kUIMultigrid is active.
///
/// # Safety
/// Calls C accessor that modifies UI state.
#[export_name = "msg_ui_refresh"]
pub unsafe extern "C" fn rs_msg_ui_refresh() {
    if ui_has(K_UI_MULTIGRID) && !msg_grid.chars.is_null() {
        ui_call_grid_resize(
            i64::from(msg_grid.handle),
            i64::from(msg_grid.cols),
            i64::from(msg_grid.rows),
        );
        rs_nvim_ui_ext_msg_set_pos(msg_grid_pos, msg_scrolled != 0);
    }
}

/// Flush pending UI updates for messages.
///
/// Updates comp index position when kUIMultigrid is active.
///
/// # Safety
/// Calls C accessor that emits UI events.
#[export_name = "msg_ui_flush"]
pub unsafe extern "C" fn rs_msg_ui_flush() {
    if ui_has(K_UI_MULTIGRID) && !msg_grid.chars.is_null() && msg_grid.pending_comp_index_update {
        rs_nvim_ui_ext_msg_set_pos(msg_grid_pos, msg_scrolled != 0);
    }
}

/// Flush scroll-related UI updates to clients.
///
/// Coalesces throttled message grid scrolling into a single grid_scroll
/// event per screen update.
///
/// # Panics
/// Panics if the scroll accounting invariants are violated (pos_delta or
/// to_scroll is negative), which indicates a bug in the scroll bookkeeping.
///
/// # Safety
/// Calls C accessor functions that modify UI state.
#[allow(clippy::cast_sign_loss)]
#[export_name = "msg_scroll_flush"]
pub unsafe extern "C" fn rs_msg_scroll_flush() {
    if msg_grid.throttled {
        msg_grid.throttled = false;
        let pos_delta = msg_grid_pos_at_flush - msg_grid_pos;
        assert!(pos_delta >= 0);
        let delta = (msg_scrolled - msg_scrolled_at_flush).min(msg_grid.rows);

        if pos_delta > 0 {
            rs_nvim_ui_ext_msg_set_pos(msg_grid_pos, true);
        }

        let to_scroll = delta - pos_delta - msg_grid_scroll_discount;
        assert!(to_scroll >= 0);

        if to_scroll > 0 && msg_grid_pos == 0 {
            // Inlined nvim_msg_grid_scroll_up:
            ui_call_grid_scroll(
                i64::from(msg_grid.handle),
                0,
                i64::from(Rows),
                0,
                i64::from(Columns),
                i64::from(to_scroll),
                0,
            );
        }

        let rows = Rows;
        let start = (rows - delta.max(1)).max(0);
        for i in start..rows {
            let row = i - msg_grid_pos;
            assert!(row >= 0);
            // Inlined nvim_msg_grid_flush_dirty_line:
            let dirty_end = *msg_grid.dirty_col.add(row as usize);
            ui_line(
                std::ptr::addr_of_mut!(msg_grid),
                row,
                false,
                0,
                dirty_end,
                msg_grid.cols,
                hl_attr(HLF_MSG),
                false,
            );
            *msg_grid.dirty_col.add(row as usize) = 0;
        }
    }
    msg_scrolled_at_flush = msg_scrolled;
    msg_grid_scroll_discount = 0;
    msg_grid_pos_at_flush = msg_grid_pos;
}

// Note: rs_msg_reset_scroll() is defined in scrollback.rs

// ============================================================================
// Cursor Functions
// ============================================================================

/// Position the cursor in the message area.
///
/// # Arguments
/// * `row` - Target row
/// * `col` - Target column
///
/// # Safety
/// Calls C functions that modify display state.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_cursor_goto(row: c_int, col: c_int) {
    rs_msg_cursor_goto_impl(row, col);
}

/// Position the cursor in the message area (C-exported implementation).
///
/// Adjusts column for right-to-left cmdline mode, then calls grid cursor goto.
///
/// # Safety
/// Calls C functions that modify display state.
#[export_name = "msg_cursor_goto"]
pub unsafe extern "C" fn rs_msg_cursor_goto_impl(mut row: c_int, mut col: c_int) {
    if cmdmsg_rl {
        col = Columns - 1 - col;
    }
    let grid = grid_adjust(
        addr_of_mut!(msg_grid_adj).cast::<c_void>(),
        &raw mut row,
        &raw mut col,
    );
    let handle = nvim_screengrid_get_handle(grid);
    nvim_ui_grid_cursor_goto(handle, row, col);
}

// ============================================================================
// Clearing Functions
// ============================================================================

/// Clear the command line area.
///
/// # Safety
/// Calls C accessor functions that modify display state.
#[export_name = "msg_clr_cmdline"]
pub unsafe extern "C" fn rs_msg_clr_cmdline() {
    msg_row = cmdline_row;
    msg_col = 0;
    crate::output_core::rs_msg_clr_eos_force_exported();
}

/// Force clear to end of screen even if not needed (rs_ alias).
///
/// # Safety
/// Delegates to the Rust msg_clr_eos_force implementation.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_clr_eos_force() {
    crate::output_core::rs_msg_clr_eos_force_exported();
}

// ============================================================================
// Message Enable Check
// ============================================================================

/// Return true if printing messages should currently be done.
///
/// Checks 'lazyredraw', character availability, and cmdheight/ext_messages.
///
/// # Safety
/// Calls C accessor functions.
#[export_name = "messaging"]
#[must_use]
pub unsafe extern "C" fn rs_messaging() -> bool {
    // TODO(bfredl): with general support for "async" messages with p_ch,
    // this should be re-enabled.
    !(p_lz != 0 && nvim_char_avail() != 0 && nvim_get_key_typed() == 0)
        && (p_ch > 0 || ui_has(K_UI_MESSAGES))
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Check if message output should be suppressed.
///
/// Returns true if msg_silent is set.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_should_suppress() -> c_int {
    c_int::from(msg_silent != 0)
}

// ============================================================================
// Messages Option Parsing (Phase 86)
// ============================================================================

// kOptMoptFlag constants (from build/src/nvim/auto/option_vars.generated.h)
const K_OPT_MOPT_FLAG_HIT_ENTER: c_int = 0x01;
const K_OPT_MOPT_FLAG_WAIT: c_int = 0x02;
const K_OPT_MOPT_FLAG_HISTORY: c_int = 0x04;

/// Parse and apply the 'messagesopt' option.
///
/// Returns OK (0) on success or FAIL (2) on error.
///
/// # Safety
/// Reads the global `p_mopt` option and calls C accessors.
#[export_name = "messagesopt_changed"]
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub unsafe extern "C" fn rs_messagesopt_changed() -> c_int {
    const OK: c_int = 1;
    const FAIL: c_int = 0;

    const OPT_HIT_ENTER: &[u8] = b"hit-enter";
    const OPT_WAIT: &[u8] = b"wait:";
    const OPT_HISTORY: &[u8] = b"history:";

    let mut messages_flags_new: c_int = 0;
    let mut messages_wait_new: c_int = 0;
    let mut messages_history_new: c_int = 0;

    let mut p: *mut c_char = p_mopt;

    while *p != 0 {
        if strnequal(p, OPT_HIT_ENTER.as_ptr().cast(), OPT_HIT_ENTER.len()) {
            p = p.add(OPT_HIT_ENTER.len());
            messages_flags_new |= K_OPT_MOPT_FLAG_HIT_ENTER;
        } else if strnequal(p, OPT_WAIT.as_ptr().cast(), OPT_WAIT.len())
            && (*p.add(OPT_WAIT.len()) as u8).is_ascii_digit()
        {
            p = p.add(OPT_WAIT.len());
            messages_wait_new = getdigits_int(std::ptr::addr_of_mut!(p), false, c_int::MAX);
            messages_flags_new |= K_OPT_MOPT_FLAG_WAIT;
        } else if strnequal(p, OPT_HISTORY.as_ptr().cast(), OPT_HISTORY.len())
            && (*p.add(OPT_HISTORY.len()) as u8).is_ascii_digit()
        {
            p = p.add(OPT_HISTORY.len());
            messages_history_new = getdigits_int(std::ptr::addr_of_mut!(p), false, c_int::MAX);
            messages_flags_new |= K_OPT_MOPT_FLAG_HISTORY;
        }

        if *p != b',' as c_char && *p != 0 {
            return FAIL;
        }
        if *p == b',' as c_char {
            p = p.add(1);
        }
    }

    // Either "wait" or "hit-enter" is required
    if messages_flags_new & (K_OPT_MOPT_FLAG_HIT_ENTER | K_OPT_MOPT_FLAG_WAIT) == 0 {
        return FAIL;
    }

    // "history" must be set
    if messages_flags_new & K_OPT_MOPT_FLAG_HISTORY == 0 {
        return FAIL;
    }

    // "history" must be <= 10000
    if messages_history_new > 10000 {
        return FAIL;
    }

    // "wait" must be <= 10000
    if messages_wait_new > 10000 {
        return FAIL;
    }

    msg_flags = messages_flags_new;
    msg_wait = messages_wait_new;
    msg_hist_max = messages_history_new;
    msg_hist_clear(messages_history_new);

    OK
}

// ============================================================================
// Filter Check Function (Phase 1)
// ============================================================================

/// Check if a message is filtered by the current `:filter` pattern.
///
/// Returns true when `:filter pattern` was used and `msg` does not match
/// `pattern` (or matches if `filter!` was used).
///
/// # Arguments
/// * `msg` - The message string to test against the filter
///
/// # Safety
/// - `msg` must be a valid NUL-terminated C string
/// - Calls C accessors that perform regex matching
#[must_use]
#[export_name = "message_filtered"]
pub unsafe extern "C" fn rs_message_filtered(msg: *const c_char) -> bool {
    if !nvim_cmdmod_has_filter() {
        return false;
    }
    let matches = nvim_cmdmod_vim_regexec(msg);
    if nvim_cmdmod_filter_force() {
        matches
    } else {
        !matches
    }
}

/// Format "N more/fewer lines" into msg_buf and return pointer to it.
///
/// Replaces nvim_format_msgmore (message.c).
///
/// # Safety
/// Writes to the C global msg_buf[480]; calls ngettext, gettext.
unsafe fn format_msgmore(n: c_int) -> *const c_char {
    const MSG_BUF_LEN: usize = 480;
    let pn_u32 = n.unsigned_abs();
    let pn = std::ffi::c_ulong::from(pn_u32);
    let fmt_ptr = if n > 0 {
        ngettext(c"%d more line".as_ptr(), c"%d more lines".as_ptr(), pn)
    } else {
        ngettext(c"%d line less".as_ptr(), c"%d fewer lines".as_ptr(), pn)
    };
    let fmt = std::ffi::CStr::from_ptr(fmt_ptr).to_string_lossy();
    let result = fmt.replacen("%d", &pn_u32.to_string(), 1);
    let bytes = result.as_bytes();
    let len = bytes.len().min(MSG_BUF_LEN - 1);
    let buf_ptr = std::ptr::addr_of_mut!(msg_buf).cast::<u8>();
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf_ptr, len);
    *buf_ptr.add(len).cast::<c_char>() = 0;
    if got_int {
        let interrupted = gettext(c" (Interrupted)".as_ptr());
        xstrlcat(
            std::ptr::addr_of_mut!(msg_buf).cast::<c_char>(),
            interrupted,
            MSG_BUF_LEN,
        );
    }
    std::ptr::addr_of!(msg_buf).cast::<c_char>()
}

/// Display "N more/fewer lines" message.
///
/// Shows the number of lines added or removed by a command.
/// Does nothing if messages are suppressed or another important message
/// is being kept.
///
/// # Safety
/// Calls C accessor and display functions.
#[export_name = "msgmore"]
pub unsafe extern "C" fn rs_msgmore(n: c_int) {
    // No messages now - wait until global is finished, or lazyredraw is set.
    if nvim_get_global_busy() != 0 || !rs_messaging() {
        return;
    }

    // Don't overwrite another important message, but do overwrite a previous
    // "more lines" or "fewer lines" message.
    if !keep_msg.is_null() && !keep_msg_more {
        return;
    }

    let pn = c_int::try_from(n.unsigned_abs()).unwrap_or(c_int::MAX);
    if i64::from(pn) > nvim_get_p_report() {
        let buf = format_msgmore(n);
        if crate::output_core::rs_msg(buf, 0) != 0 {
            rs_set_keep_msg(buf, 0);
            keep_msg_more = true;
        }
    }
}

// ============================================================================
// Phase 24: msg_moremsg migrated from C
// ============================================================================

/// Highlight field for more prompt (HLF_M = 6)
const HLF_M: c_int = 6;
/// Highlight field for message area (HLF_MSG = 5)
const HLF_MSG: c_int = 5;

/// Get HL_ATTR value for a given highlight field index.
///
/// # Safety
/// Reads from the hl_attr_active global array.
#[allow(clippy::cast_sign_loss)]
unsafe fn hl_attr(hlf: c_int) -> c_int {
    *hl_attr_active.add(hlf as usize)
}

/// Display the "--More--" prompt (and optionally scrolling help).
///
/// Equivalent to C `msg_moremsg()`.
///
/// # Safety
/// Accesses global message and grid state.
#[export_name = "msg_moremsg"]
pub unsafe extern "C" fn rs_msg_moremsg(full: bool) {
    msg_moremsg_impl(full);
}

unsafe fn msg_moremsg_impl(full: bool) {
    let attr = hl_combine_attr(hl_attr(HLF_MSG), hl_attr(HLF_M));
    grid_line_start(addr_of_mut!(msg_grid_adj).cast::<c_void>(), Rows - 1);
    let len = grid_line_puts(0, gettext(c"-- More --".as_ptr()), -1, attr);
    if full {
        let more_help = gettext(c" SPACE/d/j: screen/page/line down, b/u/k: up, q: quit ".as_ptr());
        grid_line_puts(len, more_help, -1, attr);
    }
    grid_line_cursor_goto(len);
    grid_line_flush();
}

// ============================================================================
// Phase 22: repeat_message migrated from C
// ============================================================================

/// Vim editor state mode constants (state_defs.h)
const MODE_CMDLINE: c_int = 0x08;
const MODE_ASKMORE: c_int = 0x3000;
const MODE_SETWSIZE: c_int = 0x4000;
const MODE_EXTERNCMD: c_int = 0x5000;
const MODE_HITRETURN: c_int = 0x2001; // 0x2000 | MODE_NORMAL (0x01)

/// Repeat the message for the current mode: MODE_ASKMORE, MODE_EXTERNCMD,
/// confirm() prompt or exmode_active.
///
/// Equivalent to C `repeat_message()`.
///
/// # Safety
/// Accesses global state via C extern.
#[export_name = "repeat_message"]
pub unsafe extern "C" fn rs_repeat_message() {
    if ui_has(K_UI_MESSAGES) {
        return;
    }

    if State == MODE_ASKMORE {
        msg_moremsg_impl(true); // display --more-- message again
        msg_row = Rows - 1;
    } else if (State & MODE_CMDLINE) != 0 && !crate::dialog::confirm_msg.is_null() {
        crate::dialog::rs_display_confirm_msg(); // display ":confirm" message again
        msg_row = Rows - 1;
    } else if State == MODE_EXTERNCMD {
        ui_cursor_goto(msg_row, msg_col); // put cursor back
    } else if State == MODE_HITRETURN || State == MODE_SETWSIZE {
        if msg_row == Rows - 1 {
            // Avoid drawing the "hit-enter" prompt below the previous one,
            // overwrite it. Esp. useful when regaining focus and a
            // FocusGained autocmd exists but didn't draw anything.
            msg_didout = false;
            msg_col = 0;
            msg_clr_eos();
        }
        rs_hit_return_msg(false);
        msg_row = Rows - 1;
    }
}

// ============================================================================
// hit_return_msg migrated from C (message.c)
// ============================================================================

/// Highlight for "Press ENTER" prompt (HLF_R = 18 from highlight_defs.h)
const HLF_R: c_int = 18;

/// Write the hit-return prompt.
///
/// @param newline_sb  if starting a new line, add it to the scrollback.
///
/// Equivalent to C `hit_return_msg()`.
///
/// # Safety
/// Accesses global state via C extern.
#[export_name = "hit_return_msg"]
pub unsafe extern "C" fn rs_hit_return_msg(newline_sb: bool) {
    let save_p_more = p_more;
    if !newline_sb {
        p_more = 0; // false
    }
    if msg_didout {
        // start on a new line
        msg_putchar(c_int::from(b'\n'));
    }
    p_more = 0; // don't want to see this message when scrolling back
    if got_int {
        msg_puts(gettext(c"Interrupt: ".as_ptr()));
    }
    msg_puts_hl(
        gettext(c"Press ENTER or type command to continue".as_ptr()),
        HLF_R,
        false,
    );
    if msg_use_printf() == 0 {
        msg_clr_eos();
    }
    p_more = save_p_more;
}

// ============================================================================
// Phase 2: Grid management functions (msg_grid_set_pos, msg_grid_validate)
// ============================================================================

/// z-index for the message grid (kZIndexMessages from grid_defs.h)
const K_Z_INDEX_MESSAGES: c_int = 200;

extern "C" {
    // For msg_grid_validate
    fn grid_assign_handle(grid: *mut crate::ScreenGrid);
    fn grid_alloc(grid: *mut crate::ScreenGrid, rows: c_int, cols: c_int, copy: bool, valid: bool);
    fn grid_free(grid: *mut crate::ScreenGrid);
    fn ui_comp_put_grid(
        grid: *mut crate::ScreenGrid,
        row: c_int,
        col: c_int,
        height: c_int,
        width: c_int,
        valid: bool,
        on_top: bool,
    ) -> bool;
    fn ui_comp_remove_grid(grid: *mut crate::ScreenGrid);
    fn ui_call_grid_destroy(handle: i64);
    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn msg_use_grid() -> bool;
    static mut default_grid: crate::ScreenGrid;
    static mut redraw_cmdline: bool;
    fn grid_clear(
        grid: *mut GridView,
        start_row: c_int,
        end_row: c_int,
        start_col: c_int,
        end_col: c_int,
        attr: c_int,
    );
}

/// Set the message grid row position.
///
/// Equivalent to C `msg_grid_set_pos()`.
///
/// # Safety
/// Modifies global message grid state.
#[export_name = "msg_grid_set_pos"]
pub unsafe extern "C" fn rs_msg_grid_set_pos(row: c_int, scrolled: bool) {
    if !msg_grid.throttled {
        rs_nvim_ui_ext_msg_set_pos(row, scrolled);
        msg_grid_pos_at_flush = row;
    }
    msg_grid_pos = row;
    if !msg_grid.chars.is_null() {
        msg_grid_adj.row_offset = -row;
    }
}

/// Validate (allocate/resize/free) the message grid.
///
/// Equivalent to C `msg_grid_validate()`.
///
/// # Safety
/// Modifies global message grid state and UI.
#[export_name = "msg_grid_validate"]
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
pub unsafe extern "C" fn rs_msg_grid_validate() {
    grid_assign_handle(&raw mut msg_grid);
    let should_alloc = msg_use_grid();
    let max_rows = Rows - p_ch as c_int;

    if should_alloc
        && (msg_grid.rows != Rows || msg_grid.cols != Columns || msg_grid.chars.is_null())
    {
        grid_alloc(&raw mut msg_grid, Rows, Columns, false, true);
        msg_grid.zindex = K_Z_INDEX_MESSAGES;

        xfree(msg_grid.dirty_col.cast());
        msg_grid.dirty_col = xcalloc(Rows as usize, std::mem::size_of::<c_int>()).cast();

        // Tricky: allow resize while pager or ex mode is active
        let pos = if State & MODE_ASKMORE != 0 {
            0
        } else {
            0.max(max_rows - msg_scrolled)
        };
        msg_grid.throttled = false; // don't throttle in 'cmdheight' area
        rs_msg_grid_set_pos(pos, msg_scrolled != 0);
        ui_comp_put_grid(
            &raw mut msg_grid,
            pos,
            0,
            msg_grid.rows,
            msg_grid.cols,
            false,
            true,
        );
        ui_call_grid_resize(
            msg_grid.handle.into(),
            msg_grid.cols.into(),
            msg_grid.rows.into(),
        );

        msg_scrolled_at_flush = msg_scrolled;
        msg_grid.mouse_enabled = false;
        msg_grid_adj.target = (&raw mut msg_grid).cast();
    } else if !should_alloc && !msg_grid.chars.is_null() {
        ui_comp_remove_grid(&raw mut msg_grid);
        grid_free(&raw mut msg_grid);
        // XFREE_CLEAR equivalent: free and set to null
        xfree(msg_grid.dirty_col.cast());
        msg_grid.dirty_col = std::ptr::null_mut();
        ui_call_grid_destroy(msg_grid.handle.into());
        msg_grid.throttled = false;
        msg_grid_adj.row_offset = 0;
        msg_grid_adj.target = (&raw mut default_grid).cast();
        redraw_cmdline = true;
    } else if !msg_grid.chars.is_null() && msg_scrolled == 0 && msg_grid_pos != max_rows {
        let diff = msg_grid_pos - max_rows;
        rs_msg_grid_set_pos(max_rows, false);
        if diff > 0 {
            grid_clear(
                &raw mut msg_grid_adj,
                Rows - diff,
                Rows,
                0,
                Columns,
                hl_attr(HLF_MSG),
            );
        }
    }

    if !msg_grid.chars.is_null() && msg_scrolled == 0 && cmdline_row < msg_grid_pos {
        cmdline_row = msg_grid_pos;
    }
}

// ============================================================================
// nvim_ui_ext_msg_set_pos — migrated from message.c
// ============================================================================

/// Update the message grid position in the UI compositor.
///
/// Reads the current window's msgsep fillchar, gets its UTF-8 bytes via
/// `nvim_schar_get_impl`, and calls `ui_call_msg_set_pos` via C wrapper.
///
/// Replaces C `nvim_ui_ext_msg_set_pos(int row, bool scrolled)`.
///
/// # Safety
/// Accesses global `msg_grid` and calls C accessors.
#[export_name = "nvim_ui_ext_msg_set_pos"]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_nvim_ui_ext_msg_set_pos(row: c_int, scrolled: bool) {
    const MAX_SCHAR_SIZE: usize = 32;
    let mut buf = [0u8; MAX_SCHAR_SIZE];
    let sc = nvim_curwin_get_fcs_msgsep();
    let size = nvim_schar_get_impl(buf.as_mut_ptr().cast::<c_char>(), sc);
    nvim_ui_call_msg_set_pos_impl(
        msg_grid.handle,
        row,
        scrolled,
        buf.as_ptr().cast::<c_char>(),
        size,
        msg_grid.zindex,
        msg_grid.comp_index as c_int,
    );
    msg_grid.pending_comp_index_update = false;
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
}
