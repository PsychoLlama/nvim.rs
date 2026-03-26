//! High-level input orchestrator functions
//!
//! This module implements the top-level character input functions that
//! coordinate between the typeahead buffer, mappings, and the terminal.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use std::ffi::{c_int, c_void};

// =============================================================================
// C FFI Declarations
// =============================================================================

extern "C" {
    /// vgetorpeek: get next character from typeahead or keyboard
    fn vgetorpeek(advance: bool) -> c_int;

    /// get_keystroke: get a keystroke directly from the user
    fn get_keystroke(argvars: *mut c_void) -> c_int;

    /// garbage_collect: run the garbage collector
    fn garbage_collect(testing: bool);

    /// nvim_call_updatescript: call updatescript(c) from C
    fn nvim_call_updatescript(c: c_int);

    /// can_get_old_char: check if old_char is available
    fn rs_can_get_old_char() -> c_int;
    /// get_old_char: retrieve old_char
    fn rs_get_old_char() -> c_int;
    /// restore_old_char_state: restore state after consuming old_char
    fn rs_restore_old_char_state();

    /// Get typebuf.tb_len
    fn nvim_get_typebuf_len() -> c_int;

    /// Get no_mapping global
    fn nvim_get_no_mapping() -> c_int;
    /// Set no_mapping global
    fn nvim_set_no_mapping(val: c_int);

    /// Get allow_keys global
    fn nvim_get_allow_keys() -> c_int;
    /// Set allow_keys global
    fn nvim_set_allow_keys(val: c_int);

    /// Get mod_mask global
    fn nvim_get_mod_mask() -> c_int;
    /// Set mod_mask global
    fn nvim_set_mod_mask(val: c_int);

    /// vgetc_mod_mask global (direct access)
    static mut vgetc_mod_mask: c_int;

    /// Get vgetc_char global
    fn nvim_get_vgetc_char() -> c_int;
    /// vgetc_char global (direct access)
    static mut vgetc_char: c_int;

    /// Get KeyTyped global
    fn nvim_get_keytyped() -> c_int;

    /// Get State global (for MODE_TERMINAL check)
    fn nvim_get_state() -> c_int;

    /// Get last_recorded_len
    fn nvim_get_last_recorded_len() -> usize;
    /// Set last_recorded_len
    fn nvim_set_last_recorded_len(val: usize);

    /// ins_char_typebuf: insert a character into the typeahead buffer
    fn ins_char_typebuf(c: c_int, modifiers: c_int, on_key_ignore: bool) -> c_int;

    /// ungetchars: un-get characters from the typeahead buffer
    fn ungetchars(len: c_int);

    /// state_no_longer_safe wrapper
    fn nvim_state_no_longer_safe();

    /// utf_ptr2char: convert UTF-8 bytes to codepoint
    fn utf_ptr2char(p: *const u8) -> c_int;

    /// is_mouse_key: check if a key code is a mouse key
    fn is_mouse_key(c: c_int) -> bool;

    /// MB_BYTE2LEN_CHECK wrapper
    fn nvim_mb_byte2len_check(b: c_int) -> c_int;

    /// getcmdkeycmd: get command after <Cmd> key (now in Rust, but usable via FFI)
    fn getcmdkeycmd(
        promptc: c_int,
        cookie: *mut c_void,
        indent: c_int,
        do_concat: bool,
    ) -> *mut std::ffi::c_char;

    /// xfree: free memory allocated by C allocator
    fn xfree(ptr: *mut c_void);

    /// map_execute_lua(false, true): execute Lua map with discard=true
    fn nvim_map_execute_lua_discard() -> bool;

    /// paste_repeat(0): repeat paste for K_PASTE_START discard path
    fn nvim_paste_repeat_discard();

    /// may_garbage_collect: set after garbagecollect() is called
    static may_garbage_collect: bool;

    /// want_garbage_collect: set by garbagecollect() function
    static want_garbage_collect: bool;

    /// test_disable_char_avail: disables char_avail() for testing
    static test_disable_char_avail: bool;

    // Phase 2: garray operations (opaque handle pattern)
    fn ga_init(gap: *mut GarrayT, itemsize: c_int, growsize: c_int);
    fn ga_grow(gap: *mut GarrayT, n: c_int);
    fn ga_concat(gap: *mut GarrayT, s: *const u8);
    fn ga_append(gap: *mut GarrayT, c: u8);
    fn ga_clear(gap: *mut GarrayT);

    // Error message and translation
    fn emsg(s: *const std::ffi::c_char);
    fn gettext(msgid: *const std::ffi::c_char) -> *const std::ffi::c_char;

    // got_int global
    static mut got_int: bool;

    // Phase 4: on_key_buf wrappers
    fn nvim_on_key_buf_push_nul();
    fn nvim_on_key_buf_execute_and_reset(c: c_int) -> bool;
    fn nvim_on_key_buf_shrink(len: usize);
}

// =============================================================================
// GArray type (must match C garray_T struct layout exactly)
// =============================================================================

/// Mirror of C garray_T struct (from garray_defs.h)
#[repr(C)]
pub struct GarrayT {
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
    pub ga_data: *mut std::ffi::c_void,
}

impl GarrayT {
    /// Create an empty garray (equivalent to GA_INIT(1, 32))
    const fn new(itemsize: c_int, growsize: c_int) -> Self {
        Self {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: itemsize,
            ga_growsize: growsize,
            ga_data: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// Key code constants
// =============================================================================

// Key code constants
const NUL: c_int = 0;
const ESC: c_int = 0x1b;

/// K_IGNORE: termcap2key(253, 53) = -(253 + (53 << 8))
const K_IGNORE: c_int = -((253) + (53 << 8));
/// K_VER_SCROLLBAR: termcap2key(249, 'X') = -(249 + (0x58 << 8))
const K_VER_SCROLLBAR: c_int = -((249) + (0x58 << 8));
/// K_HOR_SCROLLBAR: termcap2key(248, 'X') = -(248 + (0x58 << 8))
const K_HOR_SCROLLBAR: c_int = -((248) + (0x58 << 8));
/// K_MOUSEMOVE: termcap2key(253, 100) = -(253 + (100 << 8))
const K_MOUSEMOVE: c_int = -((253) + (100 << 8));

// Special key constants for getcmdkeycmd
/// K_SPECIAL byte value (0x80 = 128)
const K_SPECIAL_BYTE: c_int = 0x80;
/// KS_MODIFIER = 252
const KS_MODIFIER: c_int = 252;
/// KS_EXTRA = 253
const KS_EXTRA: c_int = 253;
/// KE_COMMAND = 104 (K_COMMAND = TERMCAP2KEY(KS_EXTRA, KE_COMMAND))
const KE_COMMAND: c_int = 104;
/// KE_SNR = 82 (K_SNR = TERMCAP2KEY(KS_EXTRA, KE_SNR))
const KE_SNR: c_int = 82;
/// K_COMMAND = TERMCAP2KEY(KS_EXTRA, KE_COMMAND) = -(253 + (104 << 8))
const K_COMMAND: c_int = -(KS_EXTRA + (KE_COMMAND << 8));
/// K_SNR = TERMCAP2KEY(KS_EXTRA, KE_SNR) = -(253 + (82 << 8))
const K_SNR: c_int = -(KS_EXTRA + (KE_SNR << 8));
/// KS_ZERO = 255
const KS_ZERO: c_int = 255;
/// KS_SPECIAL = 254
const KS_SPECIAL: c_int = 254;
/// KE_FILLER = 'X' = 88
const KE_FILLER: c_int = b'X' as c_int;

/// TERMCAP2KEY(a, b) = -(a + (b << 8))
const fn termcap2key(a: c_int, b: c_int) -> c_int {
    -(a + (b << 8))
}

/// KEY2TERMCAP0(x) = (-x) & 0xff
const fn key2termcap0(x: c_int) -> c_int {
    (-x) & 0xff
}

/// KEY2TERMCAP1(x) = ((-x) >> 8) & 0xff
const fn key2termcap1(x: c_int) -> c_int {
    ((-x) >> 8) & 0xff
}

/// TO_SPECIAL(a, b): convert KS_SPECIAL/KS_ZERO to K_SPECIAL/NUL else TERMCAP2KEY
const fn to_special_key(a: c_int, b: c_int) -> c_int {
    if a == KS_SPECIAL {
        K_SPECIAL_BYTE
    } else if a == KS_ZERO {
        NUL
    } else {
        termcap2key(a, b)
    }
}

// =============================================================================
// Error message strings (match C msgids exactly for gettext lookup)
// =============================================================================

/// E1255
const E_CMD_MAPPING_MUST_END_WITH_CR: &std::ffi::CStr = c"E1255: <Cmd> mapping must end with <CR>";

/// E1136: <Cmd> mapping must end with <CR> before second <Cmd>
const E_CMD_MAPPING_BEFORE_SECOND_CMD: &std::ffi::CStr =
    c"E1136: <Cmd> mapping must end with <CR> before second <Cmd>";

/// IS_SPECIAL(c): check if c is a special key (negative)
const fn is_special(c: c_int) -> bool {
    c < 0
}

/// K_SECOND(c): get second byte for special key encoding
const fn k_second(c: c_int) -> u8 {
    if c == K_SPECIAL_BYTE {
        KS_SPECIAL as u8
    } else if c == NUL {
        KS_ZERO as u8
    } else {
        key2termcap0(c) as u8
    }
}

/// K_THIRD(c): get third byte for special key encoding
const fn k_third(c: c_int) -> u8 {
    if c == K_SPECIAL_BYTE || c == NUL {
        KE_FILLER as u8
    } else {
        key2termcap1(c) as u8
    }
}

// =============================================================================
// Phase 1: Small pure orchestrators
// =============================================================================

/// Like vgetc(), but never returns NUL when called recursively.
///
/// Gets a key directly from the user if vgetc() returns NUL.
///
/// # Safety
/// Calls C functions.
#[must_use]
#[export_name = "safe_vgetc"]
pub unsafe extern "C" fn rs_safe_vgetc() -> c_int {
    let c = rs_vgetc();
    if c == NUL {
        get_keystroke(std::ptr::null_mut())
    } else {
        c
    }
}

/// Like safe_vgetc(), but loops to handle K_IGNORE.
///
/// Also ignores scrollbar events and mouse move events.
///
/// # Safety
/// Calls C functions.
#[must_use]
#[export_name = "plain_vgetc"]
pub unsafe extern "C" fn rs_plain_vgetc() -> c_int {
    loop {
        let c = rs_safe_vgetc();
        if c != K_IGNORE && c != K_VER_SCROLLBAR && c != K_HOR_SCROLLBAR && c != K_MOUSEMOVE {
            return c;
        }
    }
}

/// Check if a character is available, such that vgetc() will not block.
///
/// Returns NUL if no character is available.
/// If the next character is a special character or multi-byte, the returned
/// character is not valid!
///
/// # Safety
/// Calls C functions.
#[must_use]
#[export_name = "vpeekc"]
pub unsafe extern "C" fn rs_vpeekc() -> c_int {
    if rs_can_get_old_char() != 0 {
        return rs_get_old_char();
    }
    vgetorpeek(false)
}

/// Check if any character is available, also half an escape sequence.
///
/// When no typeahead found, but there is something in the typeahead buffer,
/// it must be an ESC that is recognized as the start of a key code.
///
/// # Safety
/// Calls C functions.
#[must_use]
#[export_name = "vpeekc_any"]
pub unsafe extern "C" fn rs_vpeekc_any() -> c_int {
    let c = rs_vpeekc();
    if c == NUL && nvim_get_typebuf_len() > 0 {
        return ESC;
    }
    c
}

/// Call vpeekc() without causing anything to be mapped.
///
/// Returns true if a character is available, false otherwise.
///
/// # Safety
/// Calls C functions.
#[must_use]
#[export_name = "char_avail"]
pub unsafe extern "C" fn rs_char_avail() -> bool {
    if test_disable_char_avail {
        return false;
    }
    let nm = nvim_get_no_mapping();
    nvim_set_no_mapping(nm + 1);
    let retval = rs_vpeekc();
    nvim_set_no_mapping(nvim_get_no_mapping() - 1);
    retval != NUL
}

/// This function is called just before doing a blocking wait.
///
/// Thus after waiting 'updatetime' for a character to arrive.
///
/// # Safety
/// Calls C functions.
#[export_name = "before_blocking"]
pub unsafe extern "C" fn rs_before_blocking() {
    nvim_call_updatescript(0);
    if may_garbage_collect {
        garbage_collect(false);
    }
}

// =============================================================================
// Phase 2: getcmdkeycmd
// =============================================================================

/// Function passed to do_cmdline() to get the command after a `<Cmd>` key from typeahead.
///
/// Returns a heap-allocated C string (caller must free), or NULL if aborted.
///
/// # Safety
/// Calls C functions. Returns a raw pointer to C heap memory.
#[export_name = "getcmdkeycmd"]
pub unsafe extern "C" fn rs_getcmdkeycmd(
    _promptc: c_int,
    _cookie: *mut c_void,
    _indent: c_int,
    _do_concat: bool,
) -> *mut std::ffi::c_char {
    let mut line_ga = GarrayT::new(1, 32);
    let ga_ptr = std::ptr::addr_of_mut!(line_ga);
    ga_init(ga_ptr, 1, 32);

    let mut c1: c_int = -1;
    let mut cmod: c_int = 0;
    let mut aborted = false;

    let nm = nvim_get_no_mapping();
    nvim_set_no_mapping(nm + 1);

    unsafe {
        got_int = false;
    }

    while c1 != NUL && !aborted {
        ga_grow(ga_ptr, 32);

        if vgetorpeek(false) == NUL {
            // incomplete <Cmd> is an error
            emsg(gettext(E_CMD_MAPPING_MUST_END_WITH_CR.as_ptr()));
            aborted = true;
            break;
        }

        // Get one character at a time.
        c1 = vgetorpeek(true);

        // Get two extra bytes for special keys
        if c1 == K_SPECIAL_BYTE {
            c1 = vgetorpeek(true);
            let c2 = vgetorpeek(true);
            if c1 == KS_MODIFIER {
                cmod = c2;
                continue;
            }
            c1 = to_special_key(c1, c2);
        }

        if unsafe { got_int } {
            aborted = true;
        } else if c1 == c_int::from(b'\r') || c1 == c_int::from(b'\n') {
            c1 = NUL; // end the line
        } else if c1 == ESC {
            aborted = true;
        } else if c1 == K_COMMAND {
            // give a nicer error message for this special case
            emsg(gettext(E_CMD_MAPPING_BEFORE_SECOND_CMD.as_ptr()));
            aborted = true;
        } else if c1 == K_SNR {
            ga_concat(ga_ptr, c"<SNR>".as_ptr().cast::<u8>());
        } else {
            if cmod != 0 {
                ga_append(ga_ptr, K_SPECIAL_BYTE as u8);
                ga_append(ga_ptr, KS_MODIFIER as u8);
                ga_append(ga_ptr, cmod as u8);
            }
            if is_special(c1) {
                ga_append(ga_ptr, K_SPECIAL_BYTE as u8);
                ga_append(ga_ptr, k_second(c1));
                ga_append(ga_ptr, k_third(c1));
            } else {
                ga_append(ga_ptr, c1 as u8);
            }
        }

        cmod = 0;
    }

    nvim_set_no_mapping(nvim_get_no_mapping() - 1);

    if aborted {
        ga_clear(ga_ptr);
    }

    line_ga.ga_data.cast::<std::ffi::c_char>()
}

// =============================================================================
// Phase 4: vgetc
// =============================================================================

// Modifier mask constants
const MOD_MASK_SHIFT: c_int = 0x02;
const MOD_MASK_CTRL: c_int = 0x04;
const MOD_MASK_ALT: c_int = 0x08;

// State flag
const MODE_TERMINAL: c_int = 0x80;

// CAR = carriage return
const CAR: c_int = 13;

// MB_MAXBYTES: maximum bytes in a UTF-8 character
const MB_MAXBYTES: usize = 21;

// Key constants for vgetc keypad normalization
// TERMCAP2KEY(a, b) = -(a + (b << 8))
const K_KPLUS: c_int = -13899; // TERMCAP2KEY('K', '6')
const K_KMINUS: c_int = -14155; // TERMCAP2KEY('K', '7')
const K_KDIVIDE: c_int = -14411; // TERMCAP2KEY('K', '8')
const K_KMULTIPLY: c_int = -14667; // TERMCAP2KEY('K', '9')
const K_KENTER: c_int = -16715; // TERMCAP2KEY('K', 'A')
const K_KPOINT: c_int = -16971; // TERMCAP2KEY('K', 'B')
const K_KCOMMA: c_int = -19787; // TERMCAP2KEY('K', 'M')
const K_KEQUAL: c_int = -20043; // TERMCAP2KEY('K', 'N')
const K_K0: c_int = -17227; // TERMCAP2KEY('K', 'C')
const K_K1: c_int = -17483; // TERMCAP2KEY('K', 'D')
const K_K2: c_int = -17739; // TERMCAP2KEY('K', 'E')
const K_K3: c_int = -17995; // TERMCAP2KEY('K', 'F')
const K_K4: c_int = -18251; // TERMCAP2KEY('K', 'G')
const K_K5: c_int = -18507; // TERMCAP2KEY('K', 'H')
const K_K6: c_int = -18763; // TERMCAP2KEY('K', 'I')
const K_K7: c_int = -19019; // TERMCAP2KEY('K', 'J')
const K_K8: c_int = -19275; // TERMCAP2KEY('K', 'K')
const K_K9: c_int = -19531; // TERMCAP2KEY('K', 'L')
const K_KUP: c_int = -30027; // TERMCAP2KEY('K', 'u')
const K_KDOWN: c_int = -25675; // TERMCAP2KEY('K', 'd')
const K_KLEFT: c_int = -27723; // TERMCAP2KEY('K', 'l')
const K_KRIGHT: c_int = -29259; // TERMCAP2KEY('K', 'r')
const K_XUP: c_int = -16893; // TERMCAP2KEY(KS_EXTRA, KE_XUP=65)
const K_XDOWN: c_int = -17149; // TERMCAP2KEY(KS_EXTRA, KE_XDOWN=66)
const K_XLEFT: c_int = -17405; // TERMCAP2KEY(KS_EXTRA, KE_XLEFT=67)
const K_XRIGHT: c_int = -17661; // TERMCAP2KEY(KS_EXTRA, KE_XRIGHT=68)
const K_XHOME: c_int = -16381; // TERMCAP2KEY(KS_EXTRA, KE_XHOME=63)
const K_ZHOME: c_int = -16637; // TERMCAP2KEY(KS_EXTRA, KE_ZHOME=64)
const K_XEND: c_int = -15869; // TERMCAP2KEY(KS_EXTRA, KE_XEND=61)
const K_ZEND: c_int = -16125; // TERMCAP2KEY(KS_EXTRA, KE_ZEND=62)
const K_HOME: c_int = -26731; // TERMCAP2KEY('k', 'h')
const K_END: c_int = -14144; // TERMCAP2KEY('@', '7')
const K_UP: c_int = -30059; // TERMCAP2KEY('k', 'u')
const K_DOWN: c_int = -25707; // TERMCAP2KEY('k', 'd')
const K_LEFT: c_int = -27755; // TERMCAP2KEY('k', 'l')
const K_RIGHT: c_int = -29291; // TERMCAP2KEY('k', 'r')
const K_S_HOME: c_int = -12835; // TERMCAP2KEY('#', '2')
const K_S_END: c_int = -14122; // TERMCAP2KEY('*', '7')
const K_C_HOME: c_int = -22525; // TERMCAP2KEY(KS_EXTRA, KE_C_HOME=87)
const K_C_END: c_int = -22781; // TERMCAP2KEY(KS_EXTRA, KE_C_END=88)
const K_LUA: c_int = -26621; // TERMCAP2KEY(KS_EXTRA, KE_LUA=103)
const K_PASTE_START: c_int = -21328; // TERMCAP2KEY('P', 'S')

/// number of characters recorded from the last vgetc() call
static mut LAST_VGETC_RECORDED_LEN: usize = 0;

/// Get the next input character.
///
/// Can return a special key or a multi-byte character.
/// Can return NUL when called recursively, use safe_vgetc() if that's not wanted.
/// This translates escaped K_SPECIAL bytes to a K_SPECIAL byte.
/// Collects the bytes of a multibyte character into the whole character.
/// Returns the modifiers in the global "mod_mask".
///
/// # Safety
/// Calls C functions and accesses global state.
#[must_use]
#[export_name = "vgetc"]
pub unsafe extern "C" fn rs_vgetc() -> c_int {
    // Do garbage collection when garbagecollect() was called previously and
    // we are now at the toplevel.
    if may_garbage_collect && want_garbage_collect {
        garbage_collect(false);
    }

    let c;

    // If a character was put back with vungetc, it was already processed.
    // Return it directly.
    if rs_can_get_old_char() != 0 {
        c = rs_get_old_char();
        rs_restore_old_char_state();
    } else {
        // last_recorded_len can be larger than LAST_VGETC_RECORDED_LEN
        // if peeking records more
        let last_recorded = nvim_get_last_recorded_len();
        nvim_set_last_recorded_len(last_recorded.saturating_sub(LAST_VGETC_RECORDED_LEN));

        nvim_set_mod_mask(0);
        vgetc_mod_mask = 0;
        vgetc_char = 0;

        let result = vgetc_inner_loop();
        c = result;

        LAST_VGETC_RECORDED_LEN = nvim_get_last_recorded_len();
    }

    // In the main loop "may_garbage_collect" can be set to do garbage
    // collection in the first next vgetc().  It's disabled after that to
    // avoid internally used Lists and Dicts to be freed.
    // Note: may_garbage_collect is a C extern static - we write via pointer
    let mgc_ptr = std::ptr::addr_of!(may_garbage_collect).cast_mut();
    *mgc_ptr = false;

    // Execute Lua on_key callbacks.
    nvim_on_key_buf_push_nul();
    let out = if nvim_on_key_buf_execute_and_reset(c) {
        // Keys following K_COMMAND/K_LUA/K_PASTE_START aren't normally received by
        // vim.on_key() callbacks, so discard them along with the current key.
        if c == K_COMMAND {
            let s = getcmdkeycmd(NUL, std::ptr::null_mut(), 0, false);
            xfree(s.cast::<c_void>());
        } else if c == K_LUA {
            nvim_map_execute_lua_discard();
        } else if c == K_PASTE_START {
            nvim_paste_repeat_discard();
        }
        // Discard the current key.
        K_IGNORE
    } else {
        c
    };

    // Need to process the character before we know it's safe to do something else.
    if out != K_IGNORE {
        nvim_state_no_longer_safe();
    }

    out
}

/// Inner loop for vgetc: reads and processes one character from typeahead.
///
/// # Safety
/// Calls C functions and accesses global state.
unsafe fn vgetc_inner_loop() -> c_int {
    let mut buf = [0u8; MB_MAXBYTES + 1];

    loop {
        // no mapping after modifier has been read
        let did_inc = if nvim_get_mod_mask() != 0 {
            nvim_set_no_mapping(nvim_get_no_mapping() + 1);
            nvim_set_allow_keys(nvim_get_allow_keys() + 1);
            true
        } else {
            false
        };
        let mut c = vgetorpeek(true);
        if did_inc {
            nvim_set_no_mapping(nvim_get_no_mapping() - 1);
            nvim_set_allow_keys(nvim_get_allow_keys() - 1);
        }

        // Get two extra bytes for special keys
        if c == K_SPECIAL_BYTE {
            let save_allow_keys = nvim_get_allow_keys();
            nvim_set_no_mapping(nvim_get_no_mapping() + 1);
            nvim_set_allow_keys(0); // make sure BS is not found
            let c2 = vgetorpeek(true); // no mapping for these chars
            c = vgetorpeek(true);
            nvim_set_no_mapping(nvim_get_no_mapping() - 1);
            nvim_set_allow_keys(save_allow_keys);
            if c2 == KS_MODIFIER {
                nvim_set_mod_mask(c);
                continue;
            }
            c = to_special_key(c2, c);
        }

        // For a multi-byte character get all the bytes and return the
        // converted character.
        // Note: This will loop until enough bytes are received!
        let n = nvim_mb_byte2len_check(c);
        if n > 1 {
            nvim_set_no_mapping(nvim_get_no_mapping() + 1);
            buf[0] = c as u8;
            #[allow(clippy::needless_range_loop)]
            for i in 1..(n as usize) {
                buf[i] = vgetorpeek(true) as u8;
                if buf[i] == K_SPECIAL_BYTE as u8 {
                    // Must be a K_SPECIAL - KS_SPECIAL - KE_FILLER sequence,
                    // which represents a K_SPECIAL (0x80).
                    vgetorpeek(true); // skip KS_SPECIAL
                    vgetorpeek(true); // skip KE_FILLER
                }
            }
            nvim_set_no_mapping(nvim_get_no_mapping() - 1);
            c = utf_ptr2char(buf.as_ptr());
        }

        // If mappings are enabled (i.e., not i_CTRL-V) and the user directly typed
        // something with MOD_MASK_ALT (<M-/<A- modifier) that was not mapped, interpret
        // <M-x> as <Esc>x rather than as an unbound <M-x> keypress. #8213
        // In Terminal mode, however, this is not desirable. #16202 #16220
        // Also do not do this for mouse keys, as terminals encode mouse events as
        // CSI sequences, and MOD_MASK_ALT has a meaning even for unmapped mouse keys.
        if nvim_get_no_mapping() == 0
            && nvim_get_keytyped() != 0
            && nvim_get_mod_mask() == MOD_MASK_ALT
            && (nvim_get_state() & MODE_TERMINAL) == 0
            && !is_mouse_key(c)
        {
            nvim_set_mod_mask(0);
            let len = ins_char_typebuf(c, 0, false);
            ins_char_typebuf(ESC, 0, false);
            // K_SPECIAL KS_MODIFIER MOD_MASK_ALT takes 3 more bytes
            let old_len = len + 3;
            ungetchars(old_len);
            nvim_on_key_buf_shrink(old_len as usize);
            continue;
        }

        if nvim_get_vgetc_char() == 0 {
            vgetc_mod_mask = nvim_get_mod_mask();
            vgetc_char = c;
        }

        // A keypad or special function key was not mapped, use it like
        // its ASCII equivalent.
        c = normalize_keypad(c, nvim_get_mod_mask());

        break c;
    }
}

/// Normalize keypad keys to their ASCII equivalents.
/// Also normalizes extended home/end/cursor keys based on mod_mask.
/// When mod_mask changes (e.g., K_XHOME + SHIFT -> K_S_HOME), it is updated via the pointer.
unsafe fn normalize_keypad(c: c_int, mod_mask: c_int) -> c_int {
    match c {
        K_KPLUS => c_int::from(b'+'),
        K_KMINUS => c_int::from(b'-'),
        K_KDIVIDE => c_int::from(b'/'),
        K_KMULTIPLY => c_int::from(b'*'),
        K_KENTER => CAR,
        K_KPOINT => c_int::from(b'.'),
        K_KCOMMA => c_int::from(b','),
        K_KEQUAL => c_int::from(b'='),
        K_K0 => c_int::from(b'0'),
        K_K1 => c_int::from(b'1'),
        K_K2 => c_int::from(b'2'),
        K_K3 => c_int::from(b'3'),
        K_K4 => c_int::from(b'4'),
        K_K5 => c_int::from(b'5'),
        K_K6 => c_int::from(b'6'),
        K_K7 => c_int::from(b'7'),
        K_K8 => c_int::from(b'8'),
        K_K9 => c_int::from(b'9'),
        K_XHOME | K_ZHOME => {
            if mod_mask == MOD_MASK_SHIFT {
                nvim_set_mod_mask(0);
                K_S_HOME
            } else if mod_mask == MOD_MASK_CTRL {
                nvim_set_mod_mask(0);
                K_C_HOME
            } else {
                K_HOME
            }
        }
        K_XEND | K_ZEND => {
            if mod_mask == MOD_MASK_SHIFT {
                nvim_set_mod_mask(0);
                K_S_END
            } else if mod_mask == MOD_MASK_CTRL {
                nvim_set_mod_mask(0);
                K_C_END
            } else {
                K_END
            }
        }
        K_KUP | K_XUP => K_UP,
        K_KDOWN | K_XDOWN => K_DOWN,
        K_KLEFT | K_XLEFT => K_LEFT,
        K_KRIGHT | K_XRIGHT => K_RIGHT,
        _ => c,
    }
}
