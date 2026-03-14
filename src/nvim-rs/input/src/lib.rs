//! Input handling utilities for Neovim
//!
//! This module provides types and utilities for input handling,
//! including terminal detection, input buffer management, and input state.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::borrow_as_ptr)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Buffer Size Constants
// =============================================================================

/// Read buffer size
pub const READ_BUFFER_SIZE: usize = 0xfff;

/// Input buffer size (read buffer * 4 + max key code len)
pub const INPUT_BUFFER_SIZE: usize = READ_BUFFER_SIZE * 4 + MAX_KEY_CODE_LEN;

/// Maximum key code length
pub const MAX_KEY_CODE_LEN: usize = 21;

/// Get read buffer size
#[no_mangle]
pub extern "C" fn rs_read_buffer_size() -> usize {
    READ_BUFFER_SIZE
}

/// Get input buffer size
#[no_mangle]
pub extern "C" fn rs_input_buffer_size() -> usize {
    INPUT_BUFFER_SIZE
}

/// Get max key code length
#[no_mangle]
pub extern "C" fn rs_max_key_code_len() -> usize {
    MAX_KEY_CODE_LEN
}

// =============================================================================
// Input State
// =============================================================================

/// Input state flags
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputState {
    /// Whether we've reached EOF
    pub eof: bool,
    /// Whether we're blocking on input
    pub blocking: bool,
    /// Whether stdin was used
    pub used_stdin: bool,
}

/// Create default input state
#[no_mangle]
pub extern "C" fn rs_input_state_default() -> InputState {
    InputState {
        eof: false,
        blocking: false,
        used_stdin: false,
    }
}

/// Check if input is at EOF
#[no_mangle]
pub unsafe extern "C" fn rs_input_is_eof(state: *const InputState) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).eof
}

/// Check if input is blocking
#[no_mangle]
pub unsafe extern "C" fn rs_input_is_blocking(state: *const InputState) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).blocking
}

// =============================================================================
// Input Buffer Ring State
// =============================================================================

/// Ring buffer state for input
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InputRingBuffer {
    /// Read position offset
    pub read_offset: usize,
    /// Write position offset
    pub write_offset: usize,
    /// Buffer capacity
    pub capacity: usize,
}

/// Initialize input ring buffer state
#[no_mangle]
pub extern "C" fn rs_input_ring_init(capacity: usize) -> InputRingBuffer {
    InputRingBuffer {
        read_offset: 0,
        write_offset: 0,
        capacity,
    }
}

/// Get available bytes in ring buffer
#[no_mangle]
pub unsafe extern "C" fn rs_input_ring_available(buf: *const InputRingBuffer) -> usize {
    if buf.is_null() {
        return 0;
    }

    let buf = &*buf;
    if buf.write_offset >= buf.read_offset {
        buf.write_offset - buf.read_offset
    } else {
        buf.capacity - buf.read_offset + buf.write_offset
    }
}

/// Get free space in ring buffer
#[no_mangle]
pub unsafe extern "C" fn rs_input_ring_free_space(buf: *const InputRingBuffer) -> usize {
    if buf.is_null() {
        return 0;
    }

    let buf = &*buf;
    buf.capacity - rs_input_ring_available(buf) - 1
}

/// Check if ring buffer is empty
#[no_mangle]
pub unsafe extern "C" fn rs_input_ring_is_empty(buf: *const InputRingBuffer) -> bool {
    if buf.is_null() {
        return true;
    }

    let buf = &*buf;
    buf.read_offset == buf.write_offset
}

// =============================================================================
// CursorHold Event State
// =============================================================================

/// State for CursorHold event timing
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CursorHoldState {
    /// Time waiting for CursorHold event (in ms)
    pub time: c_int,
    /// tb_change_cnt when waiting started
    pub tb_change_cnt: c_int,
}

/// Initialize CursorHold state
#[no_mangle]
pub extern "C" fn rs_cursorhold_state_init() -> CursorHoldState {
    CursorHoldState {
        time: 0,
        tb_change_cnt: 0,
    }
}

/// Reset CursorHold state
#[no_mangle]
pub unsafe extern "C" fn rs_cursorhold_reset(state: *mut CursorHoldState) {
    if state.is_null() {
        return;
    }

    (*state).time = 0;
    (*state).tb_change_cnt = 0;
}

// =============================================================================
// Breakcheck Level
// =============================================================================

/// Breakcheck levels for checking user interrupts
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakcheckLevel {
    /// Normal breakcheck (100 lines)
    Normal = 0,
    /// Line breakcheck (32 lines)
    Line = 1,
    /// Fast breakcheck (4000 bytes)
    Fast = 2,
    /// Very fast breakcheck (4000 bytes, no events)
    VeryFast = 3,
}

/// Line count for normal breakcheck
pub const BREAKCHECK_NORMAL_LINES: c_int = 100;

/// Line count for line breakcheck
pub const BREAKCHECK_LINE_LINES: c_int = 32;

/// Byte count for fast breakcheck
pub const BREAKCHECK_FAST_BYTES: c_int = 4000;

/// Get line count for normal breakcheck
#[no_mangle]
pub extern "C" fn rs_breakcheck_normal_lines() -> c_int {
    BREAKCHECK_NORMAL_LINES
}

/// Get line count for line breakcheck
#[no_mangle]
pub extern "C" fn rs_breakcheck_line_lines() -> c_int {
    BREAKCHECK_LINE_LINES
}

/// Get byte count for fast breakcheck
#[no_mangle]
pub extern "C" fn rs_breakcheck_fast_bytes() -> c_int {
    BREAKCHECK_FAST_BYTES
}

// =============================================================================
// Key Code Constants (verified against C headers)
// =============================================================================

const CTRL_C: c_int = 3; // ascii_defs.h
const ESC: c_int = 0x1b; // ascii_defs.h
const NUL: u8 = 0; // ascii_defs.h
const K_SPECIAL: u8 = 0x80; // keycodes.h:19
const KS_MODIFIER: u8 = 252; // keycodes.h:45
const KS_EXTRA: u8 = 253; // keycodes.h:41
const KS_SPECIAL: u8 = 254; // keycodes.h:37
const KS_ZERO: u8 = 255; // keycodes.h:33
const KE_FILLER: u8 = b'X'; // keycodes.h:70 (88)
const KE_IGNORE: u8 = 53; // keycodes.h:157
const KE_LEFTMOUSE: u8 = 44; // keycodes.h:147
const HLF_R: c_int = 18; // highlight_defs.h (enum position 18)
const EXPAND_NOTHING: c_int = 0; // cmdexpand_defs.h
const IOSIZE: usize = 1025; // vim_defs.h (1024 + 1)

/// Matches TERMCAP2KEY macro: keycodes.h:74
const fn termcap2key(a: u8, b: u8) -> c_int {
    -((a as c_int) + ((b as c_int) << 8))
}

/// Matches TO_SPECIAL macro: keycodes.h:87-88
const fn to_special(a: u8, b: u8) -> c_int {
    if a == KS_SPECIAL {
        K_SPECIAL as c_int
    } else if a == KS_ZERO {
        termcap2key(KS_ZERO, KE_FILLER) // K_ZERO
    } else {
        termcap2key(a, b)
    }
}

/// Matches MB_BYTE2LEN macro: reads utf8len_tab[b]
unsafe fn mb_byte2len(b: u8) -> u8 {
    utf8len_tab[b as usize]
}

// Derived key codes
const K_IGNORE: c_int = termcap2key(KS_EXTRA, KE_IGNORE);
const K_LEFTMOUSE: c_int = termcap2key(KS_EXTRA, KE_LEFTMOUSE);

// =============================================================================
// FFI Extern Declarations
// =============================================================================

extern "C" {
    // UTF-8 length table (mbyte.c)
    static utf8len_tab: [u8; 256];

    // Direct C globals
    static mut no_wait_return: c_int;

    // Global state accessors
    fn nvim_get_State() -> c_int;
    fn nvim_set_State(val: c_int);
    fn nvim_set_need_wait_return(val: c_int);
    fn nvim_get_msg_scrolled() -> c_int;
    fn nvim_get_msg_row() -> c_int;
    fn nvim_set_cmdline_row(val: c_int);
    fn nvim_get_mod_mask() -> c_int;
    fn nvim_set_mod_mask(val: c_int);
    fn nvim_get_no_mapping() -> c_int;
    fn nvim_set_no_mapping(val: c_int);
    fn nvim_get_allow_keys() -> c_int;
    fn nvim_set_allow_keys(val: c_int);
    fn nvim_get_mapped_ctrl_c() -> c_int;
    fn nvim_set_mapped_ctrl_c(val: c_int);
    fn nvim_ui_has_messages() -> c_int;
    fn nvim_get_keep_msg() -> *const c_char;
    fn nvim_get_keep_msg_hl_id() -> c_int;

    // Display / UI
    fn ui_flush();
    fn msg_putchar(c: c_int);
    fn setmouse();
    fn set_keep_msg(s: *const c_char, hl_id: c_int);

    // Input processing
    fn input_get(
        buf: *mut u8,
        maxlen: c_int,
        ms: c_int,
        tb_change_cnt: c_int,
        events: *mut c_void,
    ) -> c_int;
    fn fix_input_buffer(buf: *mut u8, len: c_int) -> c_int;
    fn is_mouse_key(c: c_int) -> bool;
    fn merge_modifiers(c: c_int, modifiers: *mut c_int) -> c_int;

    // String / char helpers
    fn nvim_utf_ptr2char(p: *const c_char) -> c_int;
    fn nvim_xstrdup(str: *const c_char) -> *mut c_char;
    fn nvim_xfree(ptr: *mut c_void);

    // Cmdline prompt wrapper (avoids Callback union across FFI)
    fn nvim_getcmdline_prompt_simple(
        firstc: c_int,
        prompt: *const c_char,
        hl_id: c_int,
        xp_context: c_int,
        one_key: bool,
        mouse_used: *mut bool,
    ) -> *mut c_char;

    // gettext for i18n
    fn gettext(msgid: *const c_char) -> *const c_char;
}

// =============================================================================
// Migrated Functions
// =============================================================================

/// Ask the user for input through a cmdline prompt.
///
/// Port of `prompt_for_input` from input.c.
unsafe fn prompt_for_input_impl(
    prompt: *mut c_char,
    hl_id: c_int,
    one_key: bool,
    mouse_used: *mut bool,
) -> c_int {
    let mut ret: c_int = if one_key { ESC } else { 0 };

    // Save keep_msg
    let keep_msg_ptr = nvim_get_keep_msg();
    let kmsg = if keep_msg_ptr.is_null() {
        std::ptr::null_mut()
    } else {
        nvim_xstrdup(keep_msg_ptr)
    };

    let prompt = if prompt.is_null() {
        if mouse_used.is_null() {
            gettext(c"Type number and <Enter> (q or empty cancels): ".as_ptr())
        } else {
            gettext(
                c"Type number and <Enter> or click with the mouse (q or empty cancels): ".as_ptr(),
            )
        }
    } else {
        prompt.cast_const()
    };

    nvim_set_cmdline_row(nvim_get_msg_row());
    ui_flush();

    // Don't map prompt input
    nvim_set_no_mapping(nvim_get_no_mapping() + 1);
    // Allow special keys
    nvim_set_allow_keys(nvim_get_allow_keys() + 1);

    let resp =
        nvim_getcmdline_prompt_simple(-1, prompt, hl_id, EXPAND_NOTHING, one_key, mouse_used);

    nvim_set_allow_keys(nvim_get_allow_keys() - 1);
    nvim_set_no_mapping(nvim_get_no_mapping() - 1);

    if !resp.is_null() {
        if one_key {
            ret = *resp as c_int;
        } else {
            ret = libc::atoi(resp);
        }
        nvim_xfree(resp.cast::<c_void>());
    }

    if !kmsg.is_null() {
        let keep_hl = nvim_get_keep_msg_hl_id();
        set_keep_msg(kmsg, keep_hl);
        nvim_xfree(kmsg.cast::<c_void>());
    }

    ret
}

/// Ask for a reply from the user, a 'y' or a 'n'.
///
/// Port of `ask_yesno` from input.c.
#[export_name = "ask_yesno"]
pub unsafe extern "C" fn rs_ask_yesno(str: *const c_char) -> c_int {
    let save_state = nvim_get_State();

    no_wait_return += 1;

    // Format prompt: "%s (y/n)?"
    let mut iobuff = [0u8; IOSIZE];
    let fmt = gettext(c"%s (y/n)?".as_ptr());
    libc::snprintf(iobuff.as_mut_ptr().cast::<c_char>(), IOSIZE, fmt, str);
    let prompt = nvim_xstrdup(iobuff.as_ptr().cast::<c_char>());

    let mut r: c_int = b' ' as c_int;
    while r != b'y' as c_int && r != b'n' as c_int {
        // Same highlighting as for wait_return()
        r = prompt_for_input_impl(prompt, HLF_R, true, std::ptr::null_mut());
        if r == CTRL_C || r == ESC {
            r = b'n' as c_int;
            if nvim_ui_has_messages() == 0 {
                msg_putchar(r);
            }
        }
    }

    let msg_scrolled = nvim_get_msg_scrolled();
    nvim_set_need_wait_return(c_int::from(msg_scrolled != 0));
    no_wait_return -= 1;
    nvim_set_State(save_state);
    setmouse();
    nvim_xfree(prompt.cast::<c_void>());

    r
}

/// Get a key stroke directly from the user.
///
/// Port of `get_keystroke` from input.c.
#[export_name = "get_keystroke"]
pub unsafe extern "C" fn rs_get_keystroke(events: *mut c_void) -> c_int {
    let mut buflen: usize = 150;
    let mut buf: Vec<u8> = Vec::with_capacity(buflen);
    let mut len: usize = 0;
    let save_mapped_ctrl_c = nvim_get_mapped_ctrl_c();

    nvim_set_mod_mask(0);
    nvim_set_mapped_ctrl_c(0); // mappings are not used here

    let n;
    loop {
        // Flush output before waiting
        ui_flush();

        // Leave room for fix_input_buffer to triple bytes, plus 6 for key code + NUL
        let maxlen = (buflen as c_int - 6 - len as c_int) / 3;
        if buf.is_empty() {
            buf.resize(buflen, 0);
        } else if maxlen < 10 {
            // Need more space for long escape sequences
            buflen += 100;
            buf.resize(buflen, 0);
            // Recalculate not needed; loop will recompute at top
            continue;
        }

        let maxlen = (buflen as c_int - 6 - len as c_int) / 3;

        // First time: blocking wait. Second time: wait up to 100ms.
        let got = input_get(
            buf.as_mut_ptr().add(len),
            maxlen,
            if len == 0 { -1 } else { 100 },
            0,
            events,
        );
        if got > 0 {
            // Replace zero and K_SPECIAL by a special key code.
            let fixed = fix_input_buffer(buf.as_mut_ptr().add(len), got);
            len += fixed as usize;
        }

        if got > 0 {
            // found a termcode: adjust length
            len = got as usize;
        }
        if len == 0 {
            // nothing typed yet
            continue;
        }

        // Handle modifier and/or special key code.
        let code;
        if buf[0] == K_SPECIAL {
            code = to_special(buf[1], buf[2]);
            if buf[1] == KS_MODIFIER
                || code == K_IGNORE
                || (is_mouse_key(code) && code != K_LEFTMOUSE)
            {
                if buf[1] == KS_MODIFIER {
                    nvim_set_mod_mask(buf[2] as c_int);
                }
                if len >= 3 {
                    len -= 3;
                }
                if len > 0 {
                    buf.copy_within(3..3 + len, 0);
                }
                continue;
            }
            n = code;
            break;
        }
        if mb_byte2len(buf[0]) as usize > len {
            // more bytes to get
            continue;
        }
        let idx = if len >= buflen { buflen - 1 } else { len };
        buf[idx] = NUL;
        n = nvim_utf_ptr2char(buf.as_ptr().cast::<c_char>());
        break;
    }

    nvim_set_mapped_ctrl_c(save_mapped_ctrl_c);
    let mut mod_mask = nvim_get_mod_mask();
    let result = merge_modifiers(n, &mut mod_mask);
    nvim_set_mod_mask(mod_mask);
    result
}

/// Exported wrapper for prompt_for_input (called from C).
#[export_name = "prompt_for_input"]
pub unsafe extern "C" fn rs_prompt_for_input(
    prompt: *mut c_char,
    hl_id: c_int,
    one_key: bool,
    mouse_used: *mut bool,
) -> c_int {
    prompt_for_input_impl(prompt, hl_id, one_key, mouse_used)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_sizes() {
        assert_eq!(rs_read_buffer_size(), 0xfff);
        assert_eq!(rs_max_key_code_len(), 21);

        let expected_input_size = 0xfff * 4 + 21;
        assert_eq!(rs_input_buffer_size(), expected_input_size);
    }

    #[test]
    fn test_input_state() {
        let state = rs_input_state_default();
        assert!(!state.eof);
        assert!(!state.blocking);
        assert!(!state.used_stdin);

        unsafe {
            assert!(!rs_input_is_eof(&state));
            assert!(!rs_input_is_blocking(&state));
        }
    }

    #[test]
    fn test_ring_buffer() {
        let buf = rs_input_ring_init(1024);
        assert_eq!(buf.capacity, 1024);
        assert_eq!(buf.read_offset, 0);
        assert_eq!(buf.write_offset, 0);

        unsafe {
            assert!(rs_input_ring_is_empty(&buf));
            assert_eq!(rs_input_ring_available(&buf), 0);
            assert_eq!(rs_input_ring_free_space(&buf), 1023);
        }
    }

    #[test]
    fn test_cursorhold_state() {
        let mut state = rs_cursorhold_state_init();
        assert_eq!(state.time, 0);
        assert_eq!(state.tb_change_cnt, 0);

        state.time = 100;
        state.tb_change_cnt = 5;

        unsafe { rs_cursorhold_reset(&mut state) };
        assert_eq!(state.time, 0);
        assert_eq!(state.tb_change_cnt, 0);
    }

    #[test]
    fn test_breakcheck_constants() {
        assert_eq!(rs_breakcheck_normal_lines(), 100);
        assert_eq!(rs_breakcheck_line_lines(), 32);
        assert_eq!(rs_breakcheck_fast_bytes(), 4000);
    }

    #[test]
    fn test_termcap2key() {
        // TERMCAP2KEY(KS_EXTRA, KE_IGNORE) should equal K_IGNORE
        assert_eq!(termcap2key(KS_EXTRA, KE_IGNORE), K_IGNORE);
        // TERMCAP2KEY(KS_EXTRA, KE_LEFTMOUSE) should equal K_LEFTMOUSE
        assert_eq!(termcap2key(KS_EXTRA, KE_LEFTMOUSE), K_LEFTMOUSE);
        // TERMCAP2KEY(KS_ZERO, KE_FILLER) is K_ZERO
        let k_zero = termcap2key(KS_ZERO, KE_FILLER);
        assert_eq!(k_zero, -((KS_ZERO as c_int) + ((KE_FILLER as c_int) << 8)));
    }

    #[test]
    fn test_to_special() {
        // KS_SPECIAL maps back to K_SPECIAL (0x80)
        assert_eq!(to_special(KS_SPECIAL, KE_FILLER), K_SPECIAL as c_int);
        // KS_ZERO maps to K_ZERO
        assert_eq!(
            to_special(KS_ZERO, KE_FILLER),
            termcap2key(KS_ZERO, KE_FILLER)
        );
        // Other values use TERMCAP2KEY
        assert_eq!(
            to_special(KS_EXTRA, KE_IGNORE),
            termcap2key(KS_EXTRA, KE_IGNORE)
        );
    }

    #[test]
    fn test_constants() {
        assert_eq!(HLF_R, 18);
        assert_eq!(KE_IGNORE, 53);
        assert_eq!(KE_LEFTMOUSE, 44);
        assert_eq!(K_SPECIAL, 0x80);
        assert_eq!(KS_MODIFIER, 252);
        assert_eq!(KS_EXTRA, 253);
        assert_eq!(EXPAND_NOTHING, 0);
        assert_eq!(IOSIZE, 1025);
        assert_eq!(CTRL_C, 3);
        assert_eq!(ESC, 0x1b);
    }
}
