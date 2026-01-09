//! Mouse input handling for `VTerm`
//!
//! This module provides mouse input encoding for terminal emulation,
//! supporting multiple mouse protocols (X10, UTF8, SGR, RXVT).

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_lossless)]

use std::ffi::c_int;

use crate::state::MouseProtocol;

// =============================================================================
// Mouse Flags
// =============================================================================

/// Mouse tracking flag: report clicks
pub const MOUSE_WANT_CLICK: u8 = 0x01;
/// Mouse tracking flag: report drags
pub const MOUSE_WANT_DRAG: u8 = 0x02;
/// Mouse tracking flag: report all movement
pub const MOUSE_WANT_MOVE: u8 = 0x04;

// =============================================================================
// Mouse State
// =============================================================================

/// Mouse tracking state
#[derive(Clone, Copy, Debug, Default)]
pub struct MouseState {
    /// Current mouse column (0-indexed)
    pub col: c_int,
    /// Current mouse row (0-indexed)
    pub row: c_int,
    /// Current button state (bitmask)
    pub buttons: c_int,
    /// Mouse tracking flags
    pub flags: u8,
    /// Mouse encoding protocol
    pub protocol: MouseProtocol,
}

impl MouseState {
    /// Create a new mouse state
    pub const fn new() -> Self {
        Self {
            col: 0,
            row: 0,
            buttons: 0,
            flags: 0,
            protocol: MouseProtocol::X10,
        }
    }

    /// Check if mouse tracking wants click events
    #[inline]
    pub const fn wants_click(&self) -> bool {
        (self.flags & MOUSE_WANT_CLICK) != 0
    }

    /// Check if mouse tracking wants drag events
    #[inline]
    pub const fn wants_drag(&self) -> bool {
        (self.flags & MOUSE_WANT_DRAG) != 0
    }

    /// Check if mouse tracking wants move events
    #[inline]
    pub const fn wants_move(&self) -> bool {
        (self.flags & MOUSE_WANT_MOVE) != 0
    }

    /// Check if any mouse tracking is enabled
    #[inline]
    pub const fn is_tracking(&self) -> bool {
        self.flags != 0
    }

    /// Update mouse position
    ///
    /// Returns true if the position changed
    pub fn set_position(&mut self, row: c_int, col: c_int) -> bool {
        if self.col == col && self.row == row {
            return false;
        }
        self.col = col;
        self.row = row;
        true
    }

    /// Update button state
    ///
    /// Returns true if the button state changed
    pub fn set_button(&mut self, button: u8, pressed: bool) -> bool {
        // Valid buttons are 1-3 (main) and 8-11 (extra)
        if !((1..=3).contains(&button) || (8..=11).contains(&button)) {
            return false;
        }

        let old = self.buttons;
        let mask = 1 << (button - 1);

        if pressed {
            self.buttons |= mask;
        } else {
            self.buttons &= !mask;
        }

        self.buttons != old
    }

    /// Get the lowest set button number (1-indexed), or None if no buttons pressed
    pub fn first_button(&self) -> Option<u8> {
        if self.buttons == 0 {
            return None;
        }
        // Count trailing zeros to find the lowest set bit
        Some((self.buttons.trailing_zeros() + 1) as u8)
    }
}

// =============================================================================
// Mouse Output Generation
// =============================================================================

/// Result of encoding a mouse event
#[derive(Clone, Debug)]
pub enum MouseOutput {
    /// CSI M sequence with raw bytes (X10, UTF8)
    CsiM(Vec<u8>),
    /// CSI < sequence with decimal params (SGR)
    /// Format: (code, col, row, pressed)
    Sgr(u16, c_int, c_int, bool),
    /// CSI sequence with decimal params ending in M (RXVT)
    /// Format: (code, col, row)
    Rxvt(u16, c_int, c_int),
    /// No output (position unchanged or tracking disabled)
    None,
}

/// Encode a mouse button event
///
/// # Arguments
/// * `state` - Current mouse state
/// * `button` - Button number (1-3 for main, 4-7 for scroll, 8-11 for extra)
/// * `pressed` - Whether the button is pressed or released
/// * `modifiers` - Keyboard modifiers (shift=1, alt=2, ctrl=4)
///
/// # Returns
/// The encoded mouse output
pub fn encode_button(state: &MouseState, button: u8, pressed: bool, modifiers: u8) -> MouseOutput {
    if !state.is_tracking() {
        return MouseOutput::None;
    }

    let code = button_to_code(button);
    encode_mouse_output(
        state.protocol,
        code,
        pressed,
        modifiers,
        state.col,
        state.row,
    )
}

/// Encode a mouse move event
///
/// # Arguments
/// * `state` - Current mouse state
/// * `col` - New column position
/// * `row` - New row position
/// * `modifiers` - Keyboard modifiers
///
/// # Returns
/// The encoded mouse output, or None if position unchanged
pub fn encode_move(state: &MouseState, col: c_int, row: c_int, modifiers: u8) -> MouseOutput {
    // Check if we need to report this move
    let should_report = if let Some(button) = state.first_button() {
        // Dragging with button pressed
        if state.wants_drag() {
            // Encode drag with button info
            let code = if button < 4 {
                (button - 1) as u16 + 0x20
            } else if (8..12).contains(&button) {
                (button - 8) as u16 + 0x80 + 0x20
            } else {
                return MouseOutput::None;
            };
            return encode_mouse_output(state.protocol, code, true, modifiers, col, row);
        }
        false
    } else if state.wants_move() {
        // Moving without button (motion tracking)
        true
    } else {
        false
    };

    if should_report {
        // No button pressed, report with code 3 + 0x20 = 35
        encode_mouse_output(state.protocol, 0x23, true, modifiers, col, row)
    } else {
        MouseOutput::None
    }
}

/// Convert button number to protocol code
///
/// Returns the button code. For protocols that use code 3 for release
/// (X10, UTF8, RXVT), this is handled in `encode_mouse_output`.
fn button_to_code(button: u8) -> u16 {
    match button {
        1..=3 => (button - 1) as u16,
        4..=7 => {
            // Scroll buttons (wheel up/down/left/right)
            (button - 4) as u16 + 0x40
        }
        8..=11 => {
            // Extra buttons
            (button - 8) as u16 + 0x80
        }
        _ => 3,
    }
}

/// Encode mouse output according to protocol
fn encode_mouse_output(
    protocol: MouseProtocol,
    code: u16,
    pressed: bool,
    modifiers: u8,
    col: c_int,
    row: c_int,
) -> MouseOutput {
    // Modifiers are shifted left by 2 bits in the protocol
    let mod_bits = (modifiers as u16) << 2;

    match protocol {
        MouseProtocol::X10 => {
            // X10 protocol: CSI M <code+mods+32> <col+33> <row+33>
            // Coordinates are limited to 0xFF - 0x21 = 222
            let mut c = col;
            let mut r = row;

            if c + 0x21 > 0xff {
                c = 0xff - 0x21;
            }
            if r + 0x21 > 0xff {
                r = 0xff - 0x21;
            }

            let button_code = if pressed { code } else { 3 };

            // High bit codes not supported in X10
            if button_code & 0x80 != 0 {
                return MouseOutput::None;
            }

            let bytes = vec![
                ((button_code | mod_bits) + 0x20) as u8,
                (c + 0x21) as u8,
                (r + 0x21) as u8,
            ];
            MouseOutput::CsiM(bytes)
        }

        MouseProtocol::Utf8 => {
            // UTF-8 protocol: like X10 but coordinates can be UTF-8 encoded
            let button_code = if pressed { code } else { 3 };

            let mut bytes = Vec::with_capacity(12);

            // Encode code + modifiers as UTF-8
            encode_utf8_char(u32::from((button_code | mod_bits) + 0x20), &mut bytes);
            // Encode column as UTF-8
            encode_utf8_char((col as u32) + 0x21, &mut bytes);
            // Encode row as UTF-8
            encode_utf8_char((row as u32) + 0x21, &mut bytes);

            MouseOutput::CsiM(bytes)
        }

        MouseProtocol::Sgr => {
            // SGR protocol: CSI < <code+mods> ; <col+1> ; <row+1> M|m
            MouseOutput::Sgr(code | mod_bits, col + 1, row + 1, pressed)
        }

        MouseProtocol::Rxvt => {
            // RXVT protocol: CSI <code+mods> ; <col+1> ; <row+1> M
            let button_code = if pressed { code } else { 3 };
            MouseOutput::Rxvt(button_code | mod_bits, col + 1, row + 1)
        }
    }
}

/// Encode a codepoint as UTF-8 and append to buffer
fn encode_utf8_char(c: u32, buf: &mut Vec<u8>) {
    if c < 0x80 {
        buf.push(c as u8);
    } else if c < 0x800 {
        buf.push((0xC0 | (c >> 6)) as u8);
        buf.push((0x80 | (c & 0x3F)) as u8);
    } else if c < 0x1_0000 {
        buf.push((0xE0 | (c >> 12)) as u8);
        buf.push((0x80 | ((c >> 6) & 0x3F)) as u8);
        buf.push((0x80 | (c & 0x3F)) as u8);
    } else {
        buf.push((0xF0 | (c >> 18)) as u8);
        buf.push((0x80 | ((c >> 12) & 0x3F)) as u8);
        buf.push((0x80 | ((c >> 6) & 0x3F)) as u8);
        buf.push((0x80 | (c & 0x3F)) as u8);
    }
}

// =============================================================================
// FFI Functions
// =============================================================================

/// Get mouse tracking flags from mode value
///
/// Converts `VTermProp` mouse mode to tracking flags
#[no_mangle]
pub extern "C" fn rs_vterm_mouse_mode_to_flags(mode: c_int) -> c_int {
    match mode {
        1 => MOUSE_WANT_CLICK as c_int,                     // Click
        2 => (MOUSE_WANT_CLICK | MOUSE_WANT_DRAG) as c_int, // Drag
        3 => (MOUSE_WANT_CLICK | MOUSE_WANT_DRAG | MOUSE_WANT_MOVE) as c_int, // Move
        _ => 0,                                             // None or invalid
    }
}

/// Check if mouse tracking is enabled
#[no_mangle]
pub extern "C" fn rs_vterm_mouse_is_tracking(flags: c_int) -> c_int {
    c_int::from(flags != 0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_state_new() {
        let state = MouseState::new();
        assert_eq!(state.col, 0);
        assert_eq!(state.row, 0);
        assert_eq!(state.buttons, 0);
        assert_eq!(state.flags, 0);
        assert_eq!(state.protocol, MouseProtocol::X10);
    }

    #[test]
    fn test_mouse_state_tracking_flags() {
        let mut state = MouseState::new();

        assert!(!state.is_tracking());
        assert!(!state.wants_click());
        assert!(!state.wants_drag());
        assert!(!state.wants_move());

        state.flags = MOUSE_WANT_CLICK;
        assert!(state.is_tracking());
        assert!(state.wants_click());
        assert!(!state.wants_drag());

        state.flags = MOUSE_WANT_CLICK | MOUSE_WANT_DRAG;
        assert!(state.wants_click());
        assert!(state.wants_drag());
        assert!(!state.wants_move());

        state.flags = MOUSE_WANT_CLICK | MOUSE_WANT_DRAG | MOUSE_WANT_MOVE;
        assert!(state.wants_click());
        assert!(state.wants_drag());
        assert!(state.wants_move());
    }

    #[test]
    fn test_mouse_state_position() {
        let mut state = MouseState::new();

        // First position change should return true
        assert!(state.set_position(10, 20));
        assert_eq!(state.row, 10);
        assert_eq!(state.col, 20);

        // Same position should return false
        assert!(!state.set_position(10, 20));

        // Different position should return true
        assert!(state.set_position(15, 25));
        assert_eq!(state.row, 15);
        assert_eq!(state.col, 25);
    }

    #[test]
    fn test_mouse_state_buttons() {
        let mut state = MouseState::new();

        // Button 1 press
        assert!(state.set_button(1, true));
        assert_eq!(state.buttons, 1);
        assert_eq!(state.first_button(), Some(1));

        // Button 2 press (both 1 and 2 pressed)
        assert!(state.set_button(2, true));
        assert_eq!(state.buttons, 3);
        assert_eq!(state.first_button(), Some(1)); // First button is still 1

        // Button 1 release
        assert!(state.set_button(1, false));
        assert_eq!(state.buttons, 2);
        assert_eq!(state.first_button(), Some(2));

        // Button 2 release
        assert!(state.set_button(2, false));
        assert_eq!(state.buttons, 0);
        assert_eq!(state.first_button(), None);
    }

    #[test]
    fn test_mouse_state_extra_buttons() {
        let mut state = MouseState::new();

        // Button 8 (extra button)
        assert!(state.set_button(8, true));
        assert_eq!(state.first_button(), Some(8));

        // Invalid button (out of range)
        assert!(!state.set_button(15, true));
    }

    #[test]
    fn test_button_to_code() {
        // Main buttons
        assert_eq!(button_to_code(1), 0);
        assert_eq!(button_to_code(2), 1);
        assert_eq!(button_to_code(3), 2);

        // Scroll buttons
        assert_eq!(button_to_code(4), 0x40);
        assert_eq!(button_to_code(5), 0x41);

        // Extra buttons
        assert_eq!(button_to_code(8), 0x80);
        assert_eq!(button_to_code(9), 0x81);
    }

    #[test]
    fn test_encode_button_x10() {
        let mut state = MouseState::new();
        state.flags = MOUSE_WANT_CLICK;
        state.protocol = MouseProtocol::X10;
        state.col = 10;
        state.row = 5;

        // Button 1 press at (10, 5)
        let output = encode_button(&state, 1, true, 0);
        match output {
            MouseOutput::CsiM(bytes) => {
                assert_eq!(bytes.len(), 3);
                assert_eq!(bytes[0], 0x20); // code 0 + 0x20
                assert_eq!(bytes[1], 10 + 0x21); // col + 33
                assert_eq!(bytes[2], 5 + 0x21); // row + 33
            }
            _ => panic!("Expected CsiM output"),
        }
    }

    #[test]
    fn test_encode_button_sgr() {
        let mut state = MouseState::new();
        state.flags = MOUSE_WANT_CLICK;
        state.protocol = MouseProtocol::Sgr;
        state.col = 10;
        state.row = 5;

        // Button 1 press
        let output = encode_button(&state, 1, true, 0);
        match output {
            MouseOutput::Sgr(code, col, row, pressed) => {
                assert_eq!(code, 0);
                assert_eq!(col, 11); // col + 1
                assert_eq!(row, 6); // row + 1
                assert!(pressed);
            }
            _ => panic!("Expected Sgr output"),
        }

        // Button 1 release
        let output = encode_button(&state, 1, false, 0);
        match output {
            MouseOutput::Sgr(code, col, row, pressed) => {
                assert_eq!(code, 0);
                assert_eq!(col, 11);
                assert_eq!(row, 6);
                assert!(!pressed);
            }
            _ => panic!("Expected Sgr output"),
        }
    }

    #[test]
    fn test_encode_button_with_modifiers() {
        let mut state = MouseState::new();
        state.flags = MOUSE_WANT_CLICK;
        state.protocol = MouseProtocol::Sgr;
        state.col = 0;
        state.row = 0;

        // Button 1 with Shift (modifier 1)
        let output = encode_button(&state, 1, true, 1);
        match output {
            MouseOutput::Sgr(code, _, _, _) => {
                // Modifiers shifted left by 2: 1 << 2 = 4
                assert_eq!(code, 4); // button 1 (code 0) | shift mod (1 << 2)
            }
            _ => panic!("Expected Sgr output"),
        }

        // Button 1 with Ctrl (modifier 4)
        let output = encode_button(&state, 1, true, 4);
        match output {
            MouseOutput::Sgr(code, _, _, _) => {
                // Modifiers shifted left by 2: 4 << 2 = 16
                assert_eq!(code, 16); // button 1 (code 0) | ctrl mod (4 << 2)
            }
            _ => panic!("Expected Sgr output"),
        }
    }

    #[test]
    fn test_encode_button_no_tracking() {
        let state = MouseState::new(); // flags = 0

        let output = encode_button(&state, 1, true, 0);
        assert!(matches!(output, MouseOutput::None));
    }

    #[test]
    fn test_encode_move_no_tracking() {
        let state = MouseState::new();

        let output = encode_move(&state, 10, 20, 0);
        assert!(matches!(output, MouseOutput::None));
    }

    #[test]
    fn test_encode_move_with_button_drag() {
        let mut state = MouseState::new();
        state.flags = MOUSE_WANT_CLICK | MOUSE_WANT_DRAG;
        state.protocol = MouseProtocol::Sgr;
        state.buttons = 1; // Button 1 pressed

        let output = encode_move(&state, 10, 5, 0);
        match output {
            MouseOutput::Sgr(code, col, row, pressed) => {
                // Drag code for button 1: 0 + 0x20 = 32
                assert_eq!(code, 0x20);
                assert_eq!(col, 11);
                assert_eq!(row, 6);
                assert!(pressed);
            }
            _ => panic!("Expected Sgr output"),
        }
    }

    #[test]
    fn test_ffi_mouse_mode_to_flags() {
        assert_eq!(rs_vterm_mouse_mode_to_flags(0), 0);
        assert_eq!(rs_vterm_mouse_mode_to_flags(1), MOUSE_WANT_CLICK as c_int);
        assert_eq!(
            rs_vterm_mouse_mode_to_flags(2),
            (MOUSE_WANT_CLICK | MOUSE_WANT_DRAG) as c_int
        );
        assert_eq!(
            rs_vterm_mouse_mode_to_flags(3),
            (MOUSE_WANT_CLICK | MOUSE_WANT_DRAG | MOUSE_WANT_MOVE) as c_int
        );
    }

    #[test]
    fn test_ffi_mouse_is_tracking() {
        assert_eq!(rs_vterm_mouse_is_tracking(0), 0);
        assert_eq!(rs_vterm_mouse_is_tracking(1), 1);
        assert_eq!(rs_vterm_mouse_is_tracking(7), 1);
    }
}
