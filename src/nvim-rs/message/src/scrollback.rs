//! Scrollback buffer management
//!
//! Implements scrollback storage for the "more" and "hit-enter" prompts.
//! Text is stored in a linked list of chunks that can be displayed when
//! scrolling back through message history.

use std::ffi::c_int;
use std::ptr;

/// UIExtension value for kUIMessages (ui_defs.h)
const K_UI_MESSAGES: c_int = 4;

use crate::chunk::MsgChunk;

// C accessor declarations
extern "C" {
    /// xfree wrapper
    fn xfree(ptr: *mut std::ffi::c_void);
}

// ============================================================================
// Rust-owned statics (previously file-local in message.c)
// ============================================================================

/// Last message chunk in scrollback buffer (replaces C static last_msgchunk)
#[no_mangle]
pub static mut last_msgchunk: *mut MsgChunk = std::ptr::null_mut();

/// Scrollback clear state (replaces C static do_clear_sb_text, 0 = SB_CLEAR_NONE)
#[no_mangle]
pub static mut do_clear_sb_text: c_int = 0;

/// Whether to clear temporary history entries (replaces C static do_clear_hist_temp)
#[no_mangle]
pub static mut do_clear_hist_temp: bool = true;

/// Scrollback clear state (mirrors sb_clear_T in C)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SbClearState(pub c_int);

impl SbClearState {
    /// Don't clear anything
    pub const NONE: Self = Self(0);
    /// Clear all scrollback text
    pub const ALL: Self = Self(1);
    /// Command line is busy, don't clear yet
    pub const CMDLINE_BUSY: Self = Self(2);
    /// Command line done, clear on next message
    pub const CMDLINE_DONE: Self = Self(3);
}

/// Get the current scrollback clear state.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_sb_clear_state() -> c_int {
    do_clear_sb_text
}

/// Set the scrollback clear state.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_sb_clear_state(state: c_int) {
    do_clear_sb_text = state;
}

/// Check if scrollback should be cleared on next message.
///
/// Returns true if clear state is ALL or CMDLINE_DONE.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_sb_should_clear() -> c_int {
    let state = do_clear_sb_text;
    c_int::from(state == SbClearState::ALL.0 || state == SbClearState::CMDLINE_DONE.0)
}

/// Check if in command line busy state.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_sb_cmdline_busy() -> c_int {
    c_int::from(do_clear_sb_text == SbClearState::CMDLINE_BUSY.0)
}

/// Get the start of a screen line in the scrollback buffer.
///
/// Walks backwards through chunks until finding one where the previous
/// chunk has sb_eol set (or there is no previous chunk).
///
/// # Safety
/// Direct field access on repr(C) struct.
unsafe fn msg_sb_start(mps: *mut MsgChunk) -> *mut MsgChunk {
    if mps.is_null() {
        return ptr::null_mut();
    }

    let mut mp = mps;
    loop {
        let prev = (*mp).sb_prev;
        if prev.is_null() {
            break;
        }
        if (*prev).sb_eol != 0 {
            break;
        }
        mp = prev;
    }
    mp
}

/// Mark the end of a line in scrollback.
///
/// # Safety
/// Direct field access on repr(C) struct.
#[export_name = "msg_sb_eol"]
pub unsafe extern "C" fn rs_sb_mark_eol() {
    let last = last_msgchunk;
    if !last.is_null() {
        (*last).sb_eol = 1;
    }
}

/// Start entering a command line - prepare scrollback state.
///
/// If already in busy state (nested command line), clears the last
/// unfinished line. Otherwise marks current position and sets busy state.
///
/// # Safety
/// Calls C accessor and mutator functions.
#[export_name = "sb_text_start_cmdline"]
pub unsafe extern "C" fn rs_sb_start_cmdline() {
    let state = do_clear_sb_text;

    if state == SbClearState::CMDLINE_BUSY.0 {
        // Recursive command line: clear last unfinished line
        rs_sb_restart_cmdline();
    } else {
        rs_sb_mark_eol();
        do_clear_sb_text = SbClearState::CMDLINE_BUSY.0;
    }
}

/// Restart command line - clear last unfinished line.
///
/// Called when redrawing the command line.
///
/// # Safety
/// Direct field access on repr(C) struct, frees memory.
#[export_name = "sb_text_restart_cmdline"]
pub unsafe extern "C" fn rs_sb_restart_cmdline() {
    do_clear_sb_text = SbClearState::CMDLINE_BUSY.0;

    let last = last_msgchunk;
    if last.is_null() || (*last).sb_eol != 0 {
        // No unfinished line
        return;
    }

    // Find start of the unfinished line and free it
    let tofree = msg_sb_start(last);
    if !tofree.is_null() {
        let prev = (*tofree).sb_prev;
        last_msgchunk = prev;

        if !prev.is_null() {
            (*prev).sb_next = ptr::null_mut();
        }

        // Free all chunks in this line
        let mut current = tofree;
        while !current.is_null() {
            let next = (*current).sb_next;
            xfree(current.cast());
            current = next;
        }
    }
}

/// End command line - schedule cleanup of old lines.
///
/// # Safety
/// Calls C mutator function.
#[export_name = "sb_text_end_cmdline"]
pub unsafe extern "C" fn rs_sb_end_cmdline() {
    do_clear_sb_text = SbClearState::CMDLINE_DONE.0;
}

/// Schedule clearing all scrollback text.
///
/// Called when done showing messages and screen will be redrawn.
/// Also sets do_clear_hist_temp to clear temporary history entries.
///
/// # Safety
/// Calls C mutator functions.
#[export_name = "may_clear_sb_text"]
pub unsafe extern "C" fn rs_sb_schedule_clear() {
    do_clear_sb_text = SbClearState::ALL.0;
    do_clear_hist_temp = true;
}

/// Clear scrollback text.
///
/// If `all` is true (non-zero), clears all text.
/// If `all` is false (zero), keeps the last line.
///
/// # Safety
/// Calls C accessor and mutator functions, frees memory.
#[export_name = "clear_sb_text"]
pub unsafe extern "C" fn rs_sb_clear(all: c_int) {
    let last = last_msgchunk;

    if all != 0 {
        // Clear everything
        let mut current = last;
        while !current.is_null() {
            let prev = (*current).sb_prev;
            xfree(current.cast());
            current = prev;
        }
        last_msgchunk = ptr::null_mut();
    } else {
        // Keep the last line, clear everything before it
        if last.is_null() {
            return;
        }

        let line_start = msg_sb_start(last);
        if line_start.is_null() {
            return;
        }

        let before_start = (*line_start).sb_prev;
        if before_start.is_null() {
            // Only one line, nothing to clear
            return;
        }

        // Clear everything before the last line
        let mut current = before_start;
        while !current.is_null() {
            let prev = (*current).sb_prev;
            xfree(current.cast());
            current = prev;
        }

        // Note: line_start's prev is now invalid, but we don't need to update it
        // since we iterate from last_msgchunk going backwards
    }
}

/// Count the number of lines in scrollback.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_sb_line_count() -> c_int {
    let last = last_msgchunk;
    if last.is_null() {
        return 0;
    }

    let mut count = 1;
    let mut mp = msg_sb_start(last);

    while !mp.is_null() {
        let prev = (*mp).sb_prev;
        if prev.is_null() {
            break;
        }
        mp = msg_sb_start(prev);
        count += 1;
    }

    count
}

/// Check if there is content in scrollback that can be shown.
///
/// Returns true if there is more than one line (single line looks weird).
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_sb_has_content() -> c_int {
    let last = last_msgchunk;
    if last.is_null() {
        return 0;
    }

    let start = msg_sb_start(last);
    if start.is_null() {
        return 0;
    }

    let prev = (*start).sb_prev;
    c_int::from(!prev.is_null())
}

/// Check if scrollback buffer is empty.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_sb_empty() -> c_int {
    c_int::from(last_msgchunk.is_null())
}

/// Get the SB_CLEAR_NONE constant.
#[no_mangle]
pub const extern "C" fn rs_sb_clear_none() -> c_int {
    SbClearState::NONE.0
}

/// Get the SB_CLEAR_ALL constant.
#[no_mangle]
pub const extern "C" fn rs_sb_clear_all() -> c_int {
    SbClearState::ALL.0
}

/// Get the SB_CLEAR_CMDLINE_BUSY constant.
#[no_mangle]
pub const extern "C" fn rs_sb_clear_cmdline_busy() -> c_int {
    SbClearState::CMDLINE_BUSY.0
}

/// Get the SB_CLEAR_CMDLINE_DONE constant.
#[no_mangle]
pub const extern "C" fn rs_sb_clear_cmdline_done() -> c_int {
    SbClearState::CMDLINE_DONE.0
}

/// Reset scrollback clear state to NONE.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_sb_reset_clear_state() {
    do_clear_sb_text = SbClearState::NONE.0;
}

// ============================================================================
// Phase 426: Additional Scroll Operations
// ============================================================================

extern "C" {
    static mut msg_scrolled: c_int;
    static mut msg_did_scroll: bool;
    fn ui_has(ext: c_int) -> bool;

    // For inlined nvim_msg_reset_scroll_grid
    static mut msg_grid: crate::ScreenGrid;
    static mut msg_scrolled_at_flush: c_int;
    static mut msg_grid_scroll_discount: c_int;
    static mut clear_cmdline: bool;
    static mut p_ch: i64;
    static Rows: c_int;
    fn msg_grid_set_pos(row: c_int, scrolled: bool);
    fn grid_clear_line(grid: *mut crate::ScreenGrid, off: usize, width: c_int, valid: bool);
    fn msg_scrollsize() -> c_int;
}

/// Scroll message display up (normal case).
///
/// Convenience wrapper without throttling or zerocmd handling.
///
/// # Safety
/// Calls Rust msg_scroll_up implementation.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_scroll_up_simple() {
    crate::output_core::rs_msg_scroll_up(0, 0);
}

// Note: rs_msg_scrolled() is defined in lib.rs

/// Set the msg_scrolled counter.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_scrolled(val: c_int) {
    msg_scrolled = val;
}

/// Increment the msg_scrolled counter.
///
/// # Safety
/// Calls C accessor/mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_inc_msg_scrolled() {
    let val = msg_scrolled;
    msg_scrolled = val + 1;
}

/// Check if message display has scrolled.
///
/// Returns true if msg_scrolled > 0.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_has_msg_scrolled() -> c_int {
    c_int::from(msg_scrolled > 0)
}

/// Check if msg_did_scroll flag is set.
///
/// Returns true if scrolling occurred during current message.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_did_scroll() -> c_int {
    c_int::from(msg_did_scroll)
}

/// Set the msg_did_scroll flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_did_scroll(val: c_int) {
    msg_did_scroll = val != 0;
}

/// Reset scroll state and message grid position.
///
/// Called when the message grid should be collapsed (e.g., after redraw).
///
/// # Safety
/// Modifies grid state globals; calls msg_grid_set_pos and grid_clear_line.
#[export_name = "msg_reset_scroll"]
pub unsafe extern "C" fn rs_msg_reset_scroll() {
    if ui_has(K_UI_MESSAGES) {
        return;
    }
    // Inlined from nvim_msg_reset_scroll_grid:
    msg_grid.throttled = false;
    let new_pos = Rows - c_int::try_from(p_ch).unwrap_or(c_int::MAX);
    msg_grid_set_pos(new_pos, false);
    clear_cmdline = true;
    if !msg_grid.chars.is_null() {
        let limit = msg_scrollsize().min(msg_grid.rows);
        for i in 0..limit {
            #[allow(clippy::cast_sign_loss)]
            let off = *msg_grid.line_offset.add(i as usize);
            grid_clear_line(std::ptr::addr_of_mut!(msg_grid), off, msg_grid.cols, false);
        }
    }
    msg_scrolled = 0;
    msg_scrolled_at_flush = 0;
    msg_grid_scroll_discount = 0;
}

// ============================================================================
// Phase 13: store_sb_text and inc_msg_scrolled migrated from C
// ============================================================================

extern "C" {
    fn xmalloc(size: usize) -> *mut std::ffi::c_char;
    fn gettext(s: *const std::ffi::c_char) -> *const std::ffi::c_char;
    fn get_vim_var_str(idx: c_int) -> *mut std::ffi::c_char;
    fn set_vim_var_string(idx: c_int, val: *const std::ffi::c_char, len: c_int);
    fn nvim_get_sourcing_name() -> *const std::ffi::c_char;
    fn nvim_get_sourcing_lnum() -> c_int;
    fn set_must_redraw(upd_type: c_int);
}

/// Constant: UPD_VALID redraw level (drawscreen.h)
const UPD_VALID: c_int = 10;
/// Constant: VV_SCROLLSTART vim var index (eval_defs.h)
const VV_SCROLLSTART: c_int = 46;

/// Store part of a printed message for displaying when scrolling back.
///
/// Called from msg_puts_display() (which stays in C) with pointer-to-pointer
/// args so it can advance the string position.
///
/// # Safety
/// Reads/writes through raw C pointers.
#[export_name = "store_sb_text"]
pub unsafe extern "C" fn rs_store_sb_text(
    sb_str: *mut *const std::ffi::c_char,
    s: *const std::ffi::c_char,
    hl_id: c_int,
    sb_col: *mut c_int,
    finish: c_int,
) {
    use crate::chunk::MsgChunk;

    // Handle pending clear
    if do_clear_sb_text == SbClearState::ALL.0 || do_clear_sb_text == SbClearState::CMDLINE_DONE.0 {
        rs_sb_clear(c_int::from(do_clear_sb_text == SbClearState::ALL.0));
        rs_sb_mark_eol();
        if do_clear_sb_text == SbClearState::CMDLINE_DONE.0 && s > *sb_str && *(*sb_str) == 10
        // '\n' as c_char
        {
            *sb_str = (*sb_str).add(1);
        }
        do_clear_sb_text = SbClearState::NONE.0;
    }

    if s > *sb_str {
        let text_len = s as usize - (*sb_str) as usize;
        // Allocate space for the MsgChunk header + text + NUL
        let chunk_size = std::mem::size_of::<MsgChunk>();
        // xmalloc returns *mut c_char; alignment is guaranteed by the allocator
        #[allow(clippy::cast_possible_wrap, clippy::cast_ptr_alignment)]
        let mp = xmalloc(chunk_size + text_len + 1).cast::<MsgChunk>();
        (*mp).sb_eol = i8::from(finish != 0); // sb_eol is char (bool-like)
        (*mp).sb_msg_col = *sb_col;
        (*mp).sb_hl_id = hl_id;
        // Copy text into flexible array at end of struct
        let text_ptr = mp.cast::<u8>().add(chunk_size);
        std::ptr::copy_nonoverlapping((*sb_str).cast::<u8>(), text_ptr, text_len);
        *text_ptr.add(text_len) = 0u8; // NUL terminate

        // Link into the scrollback list
        if last_msgchunk.is_null() {
            last_msgchunk = mp;
            (*mp).sb_prev = std::ptr::null_mut();
        } else {
            (*mp).sb_prev = last_msgchunk;
            (*last_msgchunk).sb_next = mp;
            last_msgchunk = mp;
        }
        (*mp).sb_next = std::ptr::null_mut();
    } else if finish != 0 && !last_msgchunk.is_null() {
        (*last_msgchunk).sb_eol = 1;
    }

    *sb_str = s;
    *sb_col = 0;
}

/// Increment "msg_scrolled" and set v:scrollstart if not already set.
///
/// Mirrors the C static `inc_msg_scrolled()` in message.c.
///
/// # Safety
/// Reads/writes global state via C extern functions.
#[export_name = "inc_msg_scrolled"]
pub unsafe extern "C" fn rs_inc_msg_scrolled_full() {
    if *get_vim_var_str(VV_SCROLLSTART) == 0 {
        let p = nvim_get_sourcing_name();
        if p.is_null() {
            let unknown = gettext(c"Unknown".as_ptr());
            set_vim_var_string(VV_SCROLLSTART, unknown, -1);
        } else {
            // Format "%s line <lnum>" using Rust string formatting
            let p_str = std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned();
            let lnum = nvim_get_sourcing_lnum();
            let fmt = format!("{p_str} line {lnum}\0");
            set_vim_var_string(VV_SCROLLSTART, fmt.as_ptr().cast::<std::ffi::c_char>(), -1);
            // fmt stays live until end of this scope
        }
    }
    msg_scrolled += 1;
    set_must_redraw(UPD_VALID);
}

// ============================================================================
// Phase 3: do_more_prompt, disp_sb_line, show_sb_text
// ============================================================================

/// Special key code constants (from keycodes.h via TERMCAP2KEY macro)
const K_BS: c_int = -25195; // TERMCAP2KEY('k','b')
const K_UP: c_int = -30059; // TERMCAP2KEY('k','u')
const K_DOWN: c_int = -25707; // TERMCAP2KEY('k','d')
const K_PAGEUP: c_int = -20587; // TERMCAP2KEY('k','P')
const K_PAGEDOWN: c_int = -20075; // TERMCAP2KEY('k','N')
const K_LEFTMOUSE: c_int = -11517; // TERMCAP2KEY(253, 44)
const K_EVENT: c_int = -26365; // TERMCAP2KEY(KS_EXTRA, 102)

/// ASCII/control key constants
const BS: c_int = 0x08; // backspace
const CAR: c_int = 0x0d; // carriage return
const NL: c_int = 0x0a; // newline (Ctrl-J)
const CTRL_B: c_int = 0x02;
const CTRL_C: c_int = 0x03;
const CTRL_F: c_int = 0x06;
const ESC: c_int = 0x1b;

/// Mode flags (state_defs.h)
const MODE_HITRETURN: c_int = 0x2000 | 0x01; // MODE_HITRETURN = 0x2000 | MODE_NORMAL
const MODE_ASKMORE: c_int = 0x3000; // MODE_ASKMORE

/// Option bell flag values (option_vars.generated.h)
const K_OPT_BO_FLAG_MESS: c_int = 0x1000; // kOptBoFlagMess

// UIExtension for kUIMessages is already defined as K_UI_MESSAGES above.

/// GridView struct layout (grid_defs.h) - local copy for scrollback module
#[repr(C)]
struct SbGridView {
    target: *mut std::ffi::c_void,
    row_offset: c_int,
    col_offset: c_int,
}

extern "C" {
    // For do_more_prompt
    fn get_keystroke(events: *mut std::ffi::c_void) -> c_int;
    fn multiqueue_process_events(q: *mut std::ffi::c_void);
    fn grid_ins_lines(
        grid: *mut crate::ScreenGrid,
        row: c_int,
        line_count: c_int,
        end: c_int,
        col: c_int,
        width: c_int,
    );
    fn setmouse();
    fn vim_beep(flag: c_int);
    fn typeahead_noflush(c: c_int);

    static mut resize_events: *mut std::ffi::c_void;
    static mut headless_mode: bool;
    static mut embedded_mode: bool;
    static mut State: c_int;
    static mut got_int: bool;
    static mut quit_more: bool;
    static mut skip_redraw: bool;
    static mut need_wait_return: bool;
    static mut lines_left: c_int;
    static mut cmdline_row: c_int;
    // Rows already declared above
    static Columns: c_int;

    fn ui_active() -> c_int;
    // ui_has already declared above
    fn msg_scroll_up(may_throttle: bool, zerocmd: bool);
    fn msg_do_throttle() -> bool;
    fn inc_msg_scrolled();
    fn msg_moremsg(full: bool);

    // Grid state (msg_grid, msg_scrolled, msg_scrolled_at_flush, msg_grid_scroll_discount
    // already declared above)
    static mut msg_grid_adj: SbGridView;
    static mut msg_row: c_int;
    static mut msg_col: c_int;

    // hl_attr_active for HLF_MSG
    static mut hl_attr_active: *mut c_int;

    // msg_puts_display for disp_sb_line
    fn msg_puts_display(str_: *const std::ffi::c_char, len: c_int, hl_id: c_int, recurse: c_int);

    // Grid clear operations
    fn grid_clear(
        grid: *mut SbGridView,
        start_row: c_int,
        end_row: c_int,
        start_col: c_int,
        end_col: c_int,
        attr: c_int,
    );

    // nvim_set_ wrappers for C globals
    fn nvim_set_clear_cmdline(val: bool);
    fn nvim_set_mode_displayed(val: bool);
    static mut redraw_cmdline: bool;
}

/// Highlight field for message area (HLF_MSG)
/// Note: value matches the constant in misc.rs
const HLF_MSG_SB: c_int = 5;

/// Get HL_ATTR value for a given highlight field index.
///
/// # Safety
/// Reads from the hl_attr_active global array.
#[allow(clippy::cast_sign_loss)]
unsafe fn hl_attr_sb(hlf: c_int) -> c_int {
    *hl_attr_active.add(hlf as usize)
}

/// Display a screen line from previously displayed text at row `row`.
///
/// Equivalent to C `disp_sb_line()` (was static in message.c).
///
/// # Safety
/// Accesses global message state and chunk pointers.
unsafe fn disp_sb_line(row: c_int, smp: *mut MsgChunk) -> *mut MsgChunk {
    let mut mp = smp;

    loop {
        msg_row = row;
        msg_col = (*mp).sb_msg_col;
        let p = crate::chunk::rs_msgchunk_text(mp);
        msg_puts_display(p, -1, (*mp).sb_hl_id, 1);
        if (*mp).sb_eol != 0 || (*mp).sb_next.is_null() {
            break;
        }
        mp = (*mp).sb_next;
    }

    (*mp).sb_next
}

/// Reentrancy guard for do_more_prompt.
static mut ENTERED: bool = false;

/// Show the more-prompt and handle the user response.
///
/// Equivalent to C `do_more_prompt()` (was static in message.c).
/// Takes care of scrollback navigation, keyboard input, screen redraw.
/// When at hit-enter prompt `typed_char` is the already typed character,
/// otherwise it's NUL.
///
/// Returns true when jumping ahead to `confirm_buttons`.
///
/// # Safety
/// Accesses global message state and UI.
#[export_name = "do_more_prompt"]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_do_more_prompt(typed_char: c_int) -> bool {
    let mut used_typed_char = typed_char;
    let old_state = State;
    let mut retval = false;
    let mut to_redraw = false;
    let mut mp_last: *mut MsgChunk = std::ptr::null_mut();

    // If headless mode is enabled and no input is required, skip.
    // However if server mode (embedded) is enabled, show "--more--".
    let no_need_more = headless_mode && !embedded_mode && ui_active() == 0;

    // We get called recursively when a timer callback outputs a message.
    // Don't show another prompt. Also when at the hit-Enter prompt and nothing typed.
    if no_need_more || ENTERED || (State == MODE_HITRETURN && typed_char == 0) {
        return false;
    }
    ENTERED = true;

    if typed_char == c_int::from(b'G') {
        // "g<": Find first line on the last page.
        mp_last = crate::chunk::rs_msg_sb_start(last_msgchunk);
        let mut i = 0;
        while i < Rows - 2 && !mp_last.is_null() && !(*mp_last).sb_prev.is_null() {
            mp_last = crate::chunk::rs_msg_sb_start((*mp_last).sb_prev);
            i += 1;
        }
    }

    State = MODE_ASKMORE;
    setmouse();
    if typed_char == 0 {
        msg_moremsg(false);
    }

    'outer: loop {
        // Get a typed character directly from the user.
        let c = if used_typed_char != 0 {
            let ch = used_typed_char;
            used_typed_char = 0;
            ch
        } else {
            get_keystroke(resize_events)
        };

        let mut toscroll: c_int = 0;

        match c {
            // Scroll one line back
            v if v == BS || v == K_BS || v == c_int::from(b'k') || v == K_UP => {
                toscroll = -1;
            }
            // One extra line
            v if v == CAR || v == NL || v == c_int::from(b'j') || v == K_DOWN => {
                toscroll = 1;
            }
            // Up half a page
            v if v == c_int::from(b'u') => {
                toscroll = -(Rows / 2);
            }
            // Down half a page
            v if v == c_int::from(b'd') => {
                toscroll = Rows / 2;
            }
            // One page back
            v if v == c_int::from(b'b') || v == CTRL_B || v == K_PAGEUP => {
                toscroll = -(Rows - 1);
            }
            // One extra page
            v if v == c_int::from(b' ')
                || v == c_int::from(b'f')
                || v == CTRL_F
                || v == K_PAGEDOWN
                || v == K_LEFTMOUSE =>
            {
                toscroll = Rows - 1;
            }
            // All the way back to start
            v if v == c_int::from(b'g') => {
                toscroll = -999_999;
            }
            // All the way to the end
            v if v == c_int::from(b'G') => {
                toscroll = 999_999;
                lines_left = 999_999;
            }
            // Start new command line
            v if v == c_int::from(b':') => {
                if crate::dialog::confirm_msg_used == 0 {
                    // Since got_int is set all typeahead will be flushed, but we
                    // want to keep this ':', remember that in a special way.
                    typeahead_noflush(c_int::from(b':'));
                    cmdline_row = Rows - 1; // put ':' on this line
                    skip_redraw = true; // skip redraw once
                    need_wait_return = false; // don't wait in main()
                }
                // FALLTHROUGH to 'q' case
                if crate::dialog::confirm_msg_used != 0 {
                    retval = true;
                } else {
                    got_int = true;
                    quit_more = true;
                }
                lines_left = Rows - 1;
                break 'outer;
            }
            // Quit
            v if v == c_int::from(b'q') || v == CTRL_C || v == ESC => {
                if crate::dialog::confirm_msg_used != 0 {
                    retval = true;
                } else {
                    got_int = true;
                    quit_more = true;
                }
                lines_left = Rows - 1;
                break 'outer;
            }
            // Process resize events
            v if v == K_EVENT => {
                multiqueue_process_events(resize_events);
                to_redraw = true;
            }
            // No valid response
            _ => {
                msg_moremsg(true);
                continue 'outer;
            }
        }

        // code assumes we only do one at a time
        debug_assert!(toscroll == 0 || !to_redraw);

        if toscroll != 0 || to_redraw {
            if toscroll < 0 || to_redraw {
                // go to start of last line
                let mp = if mp_last.is_null() {
                    crate::chunk::rs_msg_sb_start(last_msgchunk)
                } else if !(*mp_last).sb_prev.is_null() {
                    crate::chunk::rs_msg_sb_start((*mp_last).sb_prev)
                } else {
                    std::ptr::null_mut()
                };

                // go to start of line at top of the screen
                let mut mp = mp;
                let mut i = 0;
                while i < Rows - 2 && !mp.is_null() && !(*mp).sb_prev.is_null() {
                    mp = crate::chunk::rs_msg_sb_start((*mp).sb_prev);
                    i += 1;
                }

                if !mp.is_null() && (!(*mp).sb_prev.is_null() || to_redraw) {
                    // Find line to be displayed at top
                    let mut i = 0;
                    while i > toscroll {
                        if mp.is_null() || (*mp).sb_prev.is_null() {
                            break;
                        }
                        mp = crate::chunk::rs_msg_sb_start((*mp).sb_prev);
                        mp_last = if mp_last.is_null() {
                            crate::chunk::rs_msg_sb_start(last_msgchunk)
                        } else {
                            crate::chunk::rs_msg_sb_start((*mp_last).sb_prev)
                        };
                        i -= 1;
                    }

                    if toscroll == -1 && !to_redraw {
                        grid_ins_lines(&raw mut msg_grid, 0, 1, Rows, 0, Columns);
                        grid_clear(
                            &raw mut msg_grid_adj,
                            0,
                            1,
                            0,
                            Columns,
                            hl_attr_sb(HLF_MSG_SB),
                        );
                        // display line at top
                        disp_sb_line(0, mp);
                    } else {
                        // redisplay all lines
                        grid_clear(
                            &raw mut msg_grid_adj,
                            0,
                            Rows,
                            0,
                            Columns,
                            hl_attr_sb(HLF_MSG_SB),
                        );
                        let mut mp = mp;
                        let mut i = 0;
                        while !mp.is_null() && i < Rows - 1 {
                            mp = disp_sb_line(i, mp);
                            msg_scrolled += 1;
                            i += 1;
                        }
                        to_redraw = false;
                    }
                    toscroll = 0;
                }
            } else {
                // First display any text that we scrolled back.
                // if p_ch=0 we need to allocate a line for "press enter" messages!
                if cmdline_row >= Rows && !ui_has(K_UI_MESSAGES) {
                    msg_scroll_up(true, false);
                    msg_scrolled += 1;
                }
                while toscroll > 0 && !mp_last.is_null() {
                    if msg_do_throttle() && !msg_grid.throttled {
                        // Tricky: we redraw at one line higher than usual.
                        msg_scrolled_at_flush -= 1;
                        msg_grid_scroll_discount += 1;
                    }
                    // scroll up, display line at bottom
                    msg_scroll_up(true, false);
                    inc_msg_scrolled();
                    grid_clear(
                        &raw mut msg_grid_adj,
                        Rows - 2,
                        Rows - 1,
                        0,
                        Columns,
                        hl_attr_sb(HLF_MSG_SB),
                    );
                    mp_last = disp_sb_line(Rows - 2, mp_last);
                    toscroll -= 1;
                }
            }

            if toscroll <= 0 {
                // displayed the requested text, more prompt again
                grid_clear(
                    &raw mut msg_grid_adj,
                    Rows - 1,
                    Rows,
                    0,
                    Columns,
                    hl_attr_sb(HLF_MSG_SB),
                );
                msg_moremsg(false);
                continue 'outer;
            }

            // display more text, return to caller
            lines_left = toscroll;
        }

        break 'outer;
    }

    // clear the --more-- message
    grid_clear(
        &raw mut msg_grid_adj,
        Rows - 1,
        Rows,
        0,
        Columns,
        hl_attr_sb(HLF_MSG_SB),
    );
    nvim_set_clear_cmdline(false);
    nvim_set_mode_displayed(false);

    redraw_cmdline = true;

    State = old_state;
    setmouse();
    if quit_more {
        msg_row = Rows - 1;
        msg_col = 0;
    }

    ENTERED = false;
    retval
}

/// "g<" command: show scrollback text.
///
/// Equivalent to C `show_sb_text()`.
///
/// # Safety
/// Accesses global message state.
#[export_name = "show_sb_text"]
pub unsafe extern "C" fn rs_show_sb_text() {
    if ui_has(K_UI_MESSAGES) {
        // Call ex_messages with skip=true, arg="", addr_count=0.
        // This shows only temp history entries (since last cmdline).
        crate::display::rs_ex_messages_with_skip();
        return;
    }
    // Only show something if there is more than one line
    let mp = crate::chunk::rs_msg_sb_start(last_msgchunk);
    if mp.is_null() || (*mp).sb_prev.is_null() {
        vim_beep(K_OPT_BO_FLAG_MESS);
    } else {
        rs_do_more_prompt(c_int::from(b'G'));
        crate::wait::rs_wait_return(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sb_clear_state_constants() {
        assert_eq!(SbClearState::NONE.0, 0);
        assert_eq!(SbClearState::ALL.0, 1);
        assert_eq!(SbClearState::CMDLINE_BUSY.0, 2);
        assert_eq!(SbClearState::CMDLINE_DONE.0, 3);
    }

    #[test]
    fn test_sb_clear_state_exports() {
        assert_eq!(rs_sb_clear_none(), 0);
        assert_eq!(rs_sb_clear_all(), 1);
        assert_eq!(rs_sb_clear_cmdline_busy(), 2);
        assert_eq!(rs_sb_clear_cmdline_done(), 3);
    }
}
