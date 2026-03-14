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
    /// vgetc: the main keyboard input handler (stays in C until Phase 4)
    fn vgetc() -> c_int;

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

    /// Get typebuf.tb_len
    fn nvim_get_typebuf_len() -> c_int;

    /// Get no_mapping global
    fn nvim_get_no_mapping() -> c_int;
    /// Set no_mapping global
    fn nvim_set_no_mapping(val: c_int);

    /// may_garbage_collect: set after garbagecollect() is called
    static may_garbage_collect: bool;

    /// test_disable_char_avail: disables char_avail() for testing
    static test_disable_char_avail: bool;

    // Phase 2: garray operations (opaque handle pattern)
    fn ga_init(gap: *mut GarrayT, itemsize: c_int, growsize: c_int);
    fn ga_grow(gap: *mut GarrayT, n: c_int);
    fn ga_concat(gap: *mut GarrayT, s: *const u8);
    fn ga_append(gap: *mut GarrayT, c: u8);
    fn ga_clear(gap: *mut GarrayT);

    // Phase 2: error message wrappers for getcmdkeycmd
    fn nvim_emsg_cmd_mapping_must_end_with_cr();
    fn nvim_emsg_cmd_mapping_before_second_cmd();

    // Phase 2: got_int global
    static mut got_int: bool;
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
    let c = vgetc();
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

    got_int = false;

    while c1 != NUL && !aborted {
        ga_grow(ga_ptr, 32);

        if vgetorpeek(false) == NUL {
            // incomplete <Cmd> is an error
            nvim_emsg_cmd_mapping_must_end_with_cr();
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

        if got_int {
            aborted = true;
        } else if c1 == c_int::from(b'\r') || c1 == c_int::from(b'\n') {
            c1 = NUL; // end the line
        } else if c1 == ESC {
            aborted = true;
        } else if c1 == K_COMMAND {
            // give a nicer error message for this special case
            nvim_emsg_cmd_mapping_before_second_cmd();
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
