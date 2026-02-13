//! Digraph input state machine.
//!
//! Handles digraph composition during character input when the 'digraph' option is set.

use std::sync::atomic::{AtomicI32, Ordering};

use libc::c_int;

// C accessor functions
extern "C" {
    /// Get the value of the 'digraph' option (`p_dg`).
    fn nvim_get_p_dg() -> c_int;

    /// Get a character without mapping.
    fn nvim_digraph_plain_vgetc() -> c_int;

    /// Increment `no_mapping` and `allow_keys`.
    fn nvim_digraph_inc_no_mapping();

    /// Decrement `no_mapping` and `allow_keys`.
    fn nvim_digraph_dec_no_mapping();

    /// Get `cmdline_star` value.
    fn nvim_digraph_get_cmdline_star() -> c_int;

    /// Put a character on the command line.
    fn nvim_digraph_putcmdline(c: c_int, shift: c_int);

    /// Add a character to the showcmd display.
    fn nvim_digraph_add_to_showcmd(c: c_int);

    /// Get display width of a character in cells.
    fn nvim_char2cells(c: c_int) -> c_int;
}

// Import the digraph lookup function from lib
extern "C" {
    fn rs_digraph_get(char1: c_int, char2: c_int, meta_char: c_int) -> c_int;
}

/// `K_BS` key code (matching C's `K_BS`).
/// `K_BS` = `TERMCAP2KEY('k', 'b')` = -('k' + ('b' << 8)) = -(107 + 25088) = -25195
const K_BS: c_int = -25195;

/// Ctrl-H key code.
const CTRL_H: c_int = 8;

/// Static state: character before `K_BS`.
static BACKSPACED: AtomicI32 = AtomicI32::new(-1);

/// Static state: last typed character.
static LASTCHAR: AtomicI32 = AtomicI32::new(0);

/// Handle digraphs after typing a character.
///
/// When the 'digraph' option is set, this function tracks backspace
/// sequences to compose digraphs. For example, typing "a<BS>:" produces "ä".
///
/// # Arguments
/// * `c` - The character that was typed, or -1 to initialize state
///
/// # Returns
/// The resulting character (may be the composed digraph)
fn do_digraph_impl(c: c_int) -> c_int {
    if c == -1 {
        // Initialize state
        BACKSPACED.store(-1, Ordering::SeqCst);
        return c;
    }

    let mut result = c;

    // Check if digraph option is enabled
    let p_dg = unsafe { nvim_get_p_dg() };
    if p_dg != 0 {
        let backspaced = BACKSPACED.load(Ordering::SeqCst);
        if backspaced >= 0 {
            // We have a character before backspace - compose digraph
            result = unsafe { rs_digraph_get(backspaced, c, 0) };
        }
        BACKSPACED.store(-1, Ordering::SeqCst);

        let lastchar = LASTCHAR.load(Ordering::SeqCst);
        if (c == K_BS || c == CTRL_H) && lastchar >= 0 {
            // Backspace pressed - remember the previous character
            BACKSPACED.store(lastchar, Ordering::SeqCst);
        }
    }

    LASTCHAR.store(c, Ordering::SeqCst);
    result
}

/// Handle digraphs after typing a character (FFI export).
#[no_mangle]
pub extern "C" fn rs_do_digraph(c: c_int) -> c_int {
    do_digraph_impl(c)
}

/// ESC key code.
const ESC: c_int = 27;

/// Check if a character is the ESC key.
///
/// # Arguments
/// * `c` - The character to check
///
/// # Returns
/// 1 if c is ESC (27), 0 otherwise.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_digraph_is_esc(c: c_int) -> c_int {
    c_int::from(c == ESC)
}

/// Check if a character should cancel digraph input.
///
/// Returns true for ESC and special keys.
///
/// # Arguments
/// * `c` - The character to check
///
/// # Returns
/// 1 if digraph input should be canceled, 0 otherwise.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_digraph_should_cancel(c: c_int) -> c_int {
    // Special keys have negative values (TERMCAP2KEY produces negative values)
    c_int::from(c == ESC || c < 0)
}

/// Process digraph input state machine.
///
/// This is the core state machine for handling Ctrl-K digraph input.
/// It manages the two-character sequence for digraph composition.
///
/// # Arguments
/// * `first_char` - The first character entered
/// * `second_char` - The second character entered
///
/// # Returns
/// The composed digraph character, or 0 if `second_char` is ESC.
#[no_mangle]
pub extern "C" fn rs_get_digraph_result(first_char: c_int, second_char: c_int) -> c_int {
    if second_char == ESC {
        return 0;
    }
    unsafe { rs_digraph_get(first_char, second_char, 1) }
}

/// NUL character.
const NUL: c_int = 0;

/// Get a digraph via Ctrl-K two-character input.
///
/// Reads two characters from the user (with `no_mapping` set) and composes
/// a digraph. If the first character is ESC or a special key, returns
/// NUL or the special key respectively. Between characters, displays
/// the first character on the command line or showcmd.
///
/// # Arguments
/// * `cmdline` - 1 if called from command-line mode, 0 otherwise
///
/// # Returns
/// The composed digraph character, or NUL if ESC was pressed.
///
/// # Safety
/// Calls C input functions.
#[no_mangle]
pub unsafe extern "C" fn rs_get_digraph(cmdline: c_int) -> c_int {
    // Read first character with no mapping
    unsafe { nvim_digraph_inc_no_mapping() };
    let c = unsafe { nvim_digraph_plain_vgetc() };
    unsafe { nvim_digraph_dec_no_mapping() };

    // ESC cancels Ctrl-K
    if c == ESC {
        return NUL;
    }

    // Special keys (negative values) are returned as-is
    if c < 0 {
        return c;
    }

    // Show the first character
    if cmdline != 0 {
        if unsafe { nvim_char2cells(c) } == 1
            && c < 128
            && unsafe { nvim_digraph_get_cmdline_star() } == 0
        {
            unsafe { nvim_digraph_putcmdline(c, 1) };
        }
    } else {
        unsafe { nvim_digraph_add_to_showcmd(c) };
    }

    // Read second character with no mapping
    unsafe { nvim_digraph_inc_no_mapping() };
    let cc = unsafe { nvim_digraph_plain_vgetc() };
    unsafe { nvim_digraph_dec_no_mapping() };

    // ESC cancels
    if cc == ESC {
        return NUL;
    }

    // Compose the digraph
    unsafe { rs_digraph_get(c, cc, 1) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_state() {
        // Calling with -1 should initialize state
        let result = do_digraph_impl(-1);
        assert_eq!(result, -1);
        assert_eq!(BACKSPACED.load(Ordering::SeqCst), -1);
    }

    #[test]
    fn test_k_bs_constant() {
        // K_BS = TERMCAP2KEY('k', 'b') = -(107 + (98 << 8))
        assert_eq!(K_BS, -25195);
    }

    #[test]
    fn test_ctrl_h_constant() {
        assert_eq!(CTRL_H, 8);
    }

    #[test]
    fn test_lastchar_tracking() {
        // Reset state
        do_digraph_impl(-1);
        LASTCHAR.store(0, Ordering::SeqCst);

        // After processing a character, lastchar should be updated
        // Note: Without mocking p_dg, we can only test basic state updates
        let _ = do_digraph_impl(c_int::from(b'a'));
        assert_eq!(LASTCHAR.load(Ordering::SeqCst), c_int::from(b'a'));
    }

    #[test]
    fn test_is_esc() {
        assert_eq!(rs_digraph_is_esc(27), 1);
        assert_eq!(rs_digraph_is_esc(0), 0);
        assert_eq!(rs_digraph_is_esc(c_int::from(b'a')), 0);
        assert_eq!(rs_digraph_is_esc(-1), 0);
    }

    #[test]
    fn test_should_cancel() {
        // ESC should cancel
        assert_eq!(rs_digraph_should_cancel(27), 1);
        // Negative values (special keys) should cancel
        assert_eq!(rs_digraph_should_cancel(-1), 1);
        assert_eq!(rs_digraph_should_cancel(K_BS), 1);
        // Normal characters should not cancel
        assert_eq!(rs_digraph_should_cancel(c_int::from(b'a')), 0);
        assert_eq!(rs_digraph_should_cancel(c_int::from(b':')), 0);
    }

    #[test]
    fn test_get_digraph_result_esc_returns_zero() {
        // ESC as second char should return 0
        assert_eq!(rs_get_digraph_result(c_int::from(b'a'), 27), 0);
    }
}
