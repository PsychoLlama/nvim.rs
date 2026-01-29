//! Digraph input state machine.
//!
//! Handles digraph composition during character input when the 'digraph' option is set.

use std::sync::atomic::{AtomicI32, Ordering};

use libc::c_int;

// C accessor for 'digraph' option
extern "C" {
    /// Get the value of the 'digraph' option (`p_dg`).
    fn nvim_get_p_dg() -> c_int;
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
}
