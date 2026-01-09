//! `VTerm` Parser Module
//!
//! This module implements the escape sequence parser for `VTerm`.
//! It handles parsing of:
//! - Control characters (C0 and C1)
//! - CSI (Control Sequence Introducer) sequences
//! - OSC (Operating System Command) sequences
//! - DCS (Device Control String) sequences
//! - APC, PM, SOS strings
//! - Escape sequences

#![allow(clippy::cast_sign_loss)] // Index conversions are always non-negative

use std::ffi::{c_char, c_int, c_long, c_void};

use crate::{
    VTermStringFragment, CSI_ARG_FLAG_MORE, CSI_ARG_MISSING, VTERM_CSI_ARGS_MAX,
    VTERM_CSI_LEADER_MAX, VTERM_INTERMED_MAX,
};

// =============================================================================
// Parser State
// =============================================================================

/// Parser state machine states
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ParserState {
    /// Normal text processing
    #[default]
    Normal = 0,
    /// CSI leader bytes
    CsiLeader = 1,
    /// CSI numeric arguments
    CsiArgs = 2,
    /// CSI intermediate bytes
    CsiIntermed = 3,
    /// DCS command
    DcsCommand = 4,
    /// OSC command number (string states start here)
    OscCommand = 5,
    /// OSC string data
    Osc = 6,
    /// DCS vterm-specific
    DcsVterm = 7,
    /// APC string
    Apc = 8,
    /// PM string
    Pm = 9,
    /// SOS string
    Sos = 10,
}

impl ParserState {
    /// Check if this is a string state (OSC, DCS, APC, PM, SOS)
    #[inline]
    pub const fn is_string_state(self) -> bool {
        matches!(
            self,
            Self::OscCommand | Self::Osc | Self::DcsVterm | Self::Apc | Self::Pm | Self::Sos
        )
    }

    /// Check if this is an active string state (collecting string data)
    #[inline]
    pub const fn is_active_string_state(self) -> bool {
        matches!(
            self,
            Self::Osc | Self::DcsVterm | Self::Apc | Self::Pm | Self::Sos
        )
    }
}

// =============================================================================
// CSI Parser State
// =============================================================================

/// CSI sequence parser state
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CsiParserState {
    /// Leader bytes (e.g., "?" for private mode)
    pub leader: [c_char; VTERM_CSI_LEADER_MAX],
    /// Number of leader bytes
    pub leader_len: c_int,
    /// Argument values
    pub args: [c_long; VTERM_CSI_ARGS_MAX],
    /// Number of arguments
    pub arg_count: c_int,
}

impl Default for CsiParserState {
    fn default() -> Self {
        Self {
            leader: [0; VTERM_CSI_LEADER_MAX],
            leader_len: 0,
            args: [CSI_ARG_MISSING; VTERM_CSI_ARGS_MAX],
            arg_count: 0,
        }
    }
}

impl CsiParserState {
    /// Reset the CSI parser state
    #[inline]
    pub fn reset(&mut self) {
        self.leader_len = 0;
        self.arg_count = 0;
        self.args[0] = CSI_ARG_MISSING;
    }

    /// Add a leader character
    #[inline]
    pub fn add_leader(&mut self, c: c_char) -> bool {
        if (self.leader_len as usize) < VTERM_CSI_LEADER_MAX - 1 {
            self.leader[self.leader_len as usize] = c;
            self.leader_len += 1;
            true
        } else {
            false
        }
    }

    /// Terminate the leader string
    #[inline]
    pub fn terminate_leader(&mut self) {
        self.leader[self.leader_len as usize] = 0;
    }

    /// Get current argument value
    #[inline]
    pub fn current_arg(&self) -> c_long {
        self.args[self.arg_count as usize]
    }

    /// Set current argument value
    #[inline]
    pub fn set_current_arg(&mut self, val: c_long) {
        self.args[self.arg_count as usize] = val;
    }

    /// Add digit to current argument
    #[inline]
    pub fn add_digit(&mut self, digit: u8) {
        let current = self.args[self.arg_count as usize];
        if current == CSI_ARG_MISSING {
            self.args[self.arg_count as usize] = c_long::from(digit);
        } else {
            self.args[self.arg_count as usize] = current * 10 + c_long::from(digit);
        }
    }

    /// Mark current argument as having more subparameters
    #[inline]
    pub fn mark_more(&mut self) {
        self.args[self.arg_count as usize] |= CSI_ARG_FLAG_MORE;
    }

    /// Advance to next argument
    #[inline]
    pub fn next_arg(&mut self) {
        self.arg_count += 1;
        if (self.arg_count as usize) < VTERM_CSI_ARGS_MAX {
            self.args[self.arg_count as usize] = CSI_ARG_MISSING;
        }
    }

    /// Finalize argument count (increment to get total count)
    #[inline]
    pub fn finalize_args(&mut self) {
        self.arg_count += 1;
    }
}

// =============================================================================
// OSC Parser State
// =============================================================================

/// OSC sequence parser state
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct OscParserState {
    /// OSC command number (-1 if not yet parsed)
    pub command: c_int,
}

impl OscParserState {
    /// Reset the OSC parser state
    #[inline]
    pub fn reset(&mut self) {
        self.command = -1;
    }

    /// Add digit to command number
    #[inline]
    pub fn add_digit(&mut self, digit: u8) {
        if self.command == -1 {
            self.command = c_int::from(digit);
        } else {
            self.command = self.command * 10 + c_int::from(digit);
        }
    }
}

// =============================================================================
// DCS Parser State
// =============================================================================

/// DCS sequence parser state
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DcsParserState {
    /// DCS command bytes
    pub command: [c_char; VTERM_CSI_LEADER_MAX],
    /// Number of command bytes
    pub command_len: c_int,
}

impl Default for DcsParserState {
    fn default() -> Self {
        Self {
            command: [0; VTERM_CSI_LEADER_MAX],
            command_len: 0,
        }
    }
}

impl DcsParserState {
    /// Reset the DCS parser state
    #[inline]
    pub fn reset(&mut self) {
        self.command_len = 0;
    }

    /// Add a command character
    #[inline]
    pub fn add_command(&mut self, c: c_char) -> bool {
        if (self.command_len as usize) < VTERM_CSI_LEADER_MAX {
            self.command[self.command_len as usize] = c;
            self.command_len += 1;
            true
        } else {
            false
        }
    }
}

// =============================================================================
// Parser Variant Data
// =============================================================================

/// Parser variant data (union of sequence-specific state)
#[repr(C)]
#[derive(Clone, Copy)]
pub union ParserVariant {
    /// CSI sequence state
    pub csi: CsiParserState,
    /// OSC sequence state
    pub osc: OscParserState,
    /// DCS sequence state
    pub dcs: DcsParserState,
}

impl Default for ParserVariant {
    fn default() -> Self {
        Self {
            csi: CsiParserState::default(),
        }
    }
}

// =============================================================================
// Parser Callbacks
// =============================================================================

/// Text callback type - returns number of bytes consumed
pub type ParserTextCallback =
    unsafe extern "C" fn(bytes: *const c_char, len: usize, user: *mut c_void) -> c_int;

/// Control character callback type - returns 1 if handled
pub type ParserControlCallback = unsafe extern "C" fn(control: u8, user: *mut c_void) -> c_int;

/// Escape sequence callback type - returns 1 if handled
pub type ParserEscapeCallback =
    unsafe extern "C" fn(seq: *const c_char, len: usize, user: *mut c_void) -> c_int;

/// CSI sequence callback type - returns 1 if handled
pub type ParserCsiCallback = unsafe extern "C" fn(
    leader: *const c_char,
    args: *const c_long,
    arg_count: c_int,
    intermed: *const c_char,
    command: c_char,
    user: *mut c_void,
) -> c_int;

/// OSC sequence callback type
pub type ParserOscCallback =
    unsafe extern "C" fn(command: c_int, frag: VTermStringFragment, user: *mut c_void) -> c_int;

/// DCS sequence callback type
pub type ParserDcsCallback = unsafe extern "C" fn(
    command: *const c_char,
    command_len: usize,
    frag: VTermStringFragment,
    user: *mut c_void,
) -> c_int;

/// APC string callback type
pub type ParserApcCallback =
    unsafe extern "C" fn(frag: VTermStringFragment, user: *mut c_void) -> c_int;

/// PM string callback type
pub type ParserPmCallback =
    unsafe extern "C" fn(frag: VTermStringFragment, user: *mut c_void) -> c_int;

/// SOS string callback type
pub type ParserSosCallback =
    unsafe extern "C" fn(frag: VTermStringFragment, user: *mut c_void) -> c_int;

/// Parser callback function table
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VTermParserCallbacks {
    /// Text callback
    pub text: Option<ParserTextCallback>,
    /// Control character callback
    pub control: Option<ParserControlCallback>,
    /// Escape sequence callback
    pub escape: Option<ParserEscapeCallback>,
    /// CSI sequence callback
    pub csi: Option<ParserCsiCallback>,
    /// OSC sequence callback
    pub osc: Option<ParserOscCallback>,
    /// DCS sequence callback
    pub dcs: Option<ParserDcsCallback>,
    /// APC string callback
    pub apc: Option<ParserApcCallback>,
    /// PM string callback
    pub pm: Option<ParserPmCallback>,
    /// SOS string callback
    pub sos: Option<ParserSosCallback>,
}

// =============================================================================
// Parser Core
// =============================================================================

/// `VTerm` parser state
#[repr(C)]
pub struct Parser {
    /// Current parser state
    pub state: ParserState,
    /// Whether we're in an ESC sequence
    pub in_esc: bool,
    /// Intermediate bytes for current sequence
    pub intermed: [c_char; VTERM_INTERMED_MAX],
    /// Number of intermediate bytes
    pub intermed_len: c_int,
    /// Variant data for current sequence type
    pub v: ParserVariant,
    /// Callbacks for parser events
    pub callbacks: *const VTermParserCallbacks,
    /// User data for callbacks
    pub cbdata: *mut c_void,
    /// Whether current string is the initial fragment
    pub string_initial: bool,
    /// Whether to emit NUL characters
    pub emit_nul: bool,
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            state: ParserState::Normal,
            in_esc: false,
            intermed: [0; VTERM_INTERMED_MAX],
            intermed_len: 0,
            v: ParserVariant::default(),
            callbacks: std::ptr::null(),
            cbdata: std::ptr::null_mut(),
            string_initial: false,
            emit_nul: false,
        }
    }
}

impl Parser {
    /// Create a new parser
    #[inline]
    pub const fn new() -> Self {
        Self {
            state: ParserState::Normal,
            in_esc: false,
            intermed: [0; VTERM_INTERMED_MAX],
            intermed_len: 0,
            v: ParserVariant {
                csi: CsiParserState {
                    leader: [0; VTERM_CSI_LEADER_MAX],
                    leader_len: 0,
                    args: [CSI_ARG_MISSING; VTERM_CSI_ARGS_MAX],
                    arg_count: 0,
                },
            },
            callbacks: std::ptr::null(),
            cbdata: std::ptr::null_mut(),
            string_initial: false,
            emit_nul: false,
        }
    }

    /// Reset intermediate bytes
    #[inline]
    pub fn reset_intermed(&mut self) {
        self.intermed_len = 0;
    }

    /// Add an intermediate character
    #[inline]
    pub fn add_intermed(&mut self, c: c_char) -> bool {
        if (self.intermed_len as usize) < VTERM_INTERMED_MAX - 1 {
            self.intermed[self.intermed_len as usize] = c;
            self.intermed_len += 1;
            true
        } else {
            false
        }
    }

    /// Terminate the intermediate string
    #[inline]
    pub fn terminate_intermed(&mut self) {
        self.intermed[self.intermed_len as usize] = 0;
    }

    /// Enter a new state
    #[inline]
    pub fn enter_state(&mut self, state: ParserState) {
        self.state = state;
    }

    /// Enter normal state
    #[inline]
    pub fn enter_normal(&mut self) {
        self.state = ParserState::Normal;
    }

    /// Check if a byte is an intermediate byte (0x20-0x2f)
    #[inline]
    pub const fn is_intermed(c: u8) -> bool {
        c >= 0x20 && c <= 0x2f
    }

    /// Set callbacks
    pub fn set_callbacks(&mut self, callbacks: *const VTermParserCallbacks, user: *mut c_void) {
        self.callbacks = callbacks;
        self.cbdata = user;
    }

    /// Get CSI state mutably (unsafe - must ensure correct state)
    ///
    /// # Safety
    /// Caller must ensure parser is in CSI-related state.
    #[inline]
    pub unsafe fn csi_mut(&mut self) -> &mut CsiParserState {
        &mut self.v.csi
    }

    /// Get CSI state (unsafe - must ensure correct state)
    ///
    /// # Safety
    /// Caller must ensure parser is in CSI-related state.
    #[inline]
    pub unsafe fn csi(&self) -> &CsiParserState {
        &self.v.csi
    }

    /// Get OSC state mutably (unsafe - must ensure correct state)
    ///
    /// # Safety
    /// Caller must ensure parser is in OSC-related state.
    #[inline]
    pub unsafe fn osc_mut(&mut self) -> &mut OscParserState {
        &mut self.v.osc
    }

    /// Get OSC state (unsafe - must ensure correct state)
    ///
    /// # Safety
    /// Caller must ensure parser is in OSC-related state.
    #[inline]
    pub unsafe fn osc(&self) -> &OscParserState {
        &self.v.osc
    }

    /// Get DCS state mutably (unsafe - must ensure correct state)
    ///
    /// # Safety
    /// Caller must ensure parser is in DCS-related state.
    #[inline]
    pub unsafe fn dcs_mut(&mut self) -> &mut DcsParserState {
        &mut self.v.dcs
    }

    /// Get DCS state (unsafe - must ensure correct state)
    ///
    /// # Safety
    /// Caller must ensure parser is in DCS-related state.
    #[inline]
    pub unsafe fn dcs(&self) -> &DcsParserState {
        &self.v.dcs
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if a character is a C0 control character (0x00-0x1f)
#[inline]
pub const fn is_c0_control(c: u8) -> bool {
    c < 0x20
}

/// Check if a character is a C1 control character (0x80-0x9f)
#[inline]
pub const fn is_c1_control(c: u8) -> bool {
    c >= 0x80 && c < 0xa0
}

/// Check if a character is a CSI final byte (0x40-0x7e)
#[inline]
pub const fn is_csi_final(c: u8) -> bool {
    c >= 0x40 && c <= 0x7e
}

/// Check if a character is a CSI leader byte (0x3c-0x3f)
#[inline]
pub const fn is_csi_leader(c: u8) -> bool {
    c >= 0x3c && c <= 0x3f
}

/// Check if a character is a digit (0x30-0x39)
#[inline]
pub const fn is_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// Convert a digit character to its numeric value
#[inline]
pub const fn digit_value(c: u8) -> u8 {
    c - b'0'
}

// =============================================================================
// C1 Control Codes
// =============================================================================

/// C1 control code constants
pub mod c1 {
    /// Single Shift 3
    pub const SS3: u8 = 0x8f;
    /// Device Control String
    pub const DCS: u8 = 0x90;
    /// String Terminator
    pub const ST: u8 = 0x9c;
    /// Control Sequence Introducer
    pub const CSI: u8 = 0x9b;
    /// Operating System Command
    pub const OSC: u8 = 0x9d;
    /// Start of String
    pub const SOS: u8 = 0x98;
    /// Privacy Message
    pub const PM: u8 = 0x9e;
    /// Application Program Command
    pub const APC: u8 = 0x9f;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::cast_possible_wrap)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_state() {
        assert!(!ParserState::Normal.is_string_state());
        assert!(!ParserState::CsiLeader.is_string_state());
        assert!(ParserState::Osc.is_string_state());
        assert!(ParserState::OscCommand.is_string_state());
        assert!(ParserState::DcsVterm.is_string_state());
        assert!(ParserState::Apc.is_string_state());
        assert!(ParserState::Pm.is_string_state());
        assert!(ParserState::Sos.is_string_state());
    }

    #[test]
    fn test_csi_parser_state() {
        let mut csi = CsiParserState::default();

        // Test leader
        assert!(csi.add_leader(b'?' as c_char));
        assert_eq!(csi.leader_len, 1);
        csi.terminate_leader();

        // Test digits
        csi.reset();
        assert_eq!(csi.current_arg(), CSI_ARG_MISSING);
        csi.add_digit(1);
        assert_eq!(csi.current_arg(), 1);
        csi.add_digit(2);
        assert_eq!(csi.current_arg(), 12);

        // Test next arg
        csi.next_arg();
        assert_eq!(csi.arg_count, 1);
        assert_eq!(csi.current_arg(), CSI_ARG_MISSING);

        // Test finalize
        csi.finalize_args();
        assert_eq!(csi.arg_count, 2);
    }

    #[test]
    fn test_osc_parser_state() {
        let mut osc = OscParserState::default();
        assert_eq!(osc.command, 0); // default is 0 in Rust

        osc.reset();
        assert_eq!(osc.command, -1);

        osc.add_digit(5);
        assert_eq!(osc.command, 5);

        osc.add_digit(2);
        assert_eq!(osc.command, 52);
    }

    #[test]
    fn test_parser_intermed() {
        let mut parser = Parser::new();

        assert!(parser.add_intermed(b'(' as c_char));
        assert_eq!(parser.intermed_len, 1);

        parser.terminate_intermed();
        assert_eq!(parser.intermed[1], 0);

        parser.reset_intermed();
        assert_eq!(parser.intermed_len, 0);
    }

    #[test]
    fn test_helper_functions() {
        assert!(is_c0_control(0x00));
        assert!(is_c0_control(0x1b)); // ESC
        assert!(!is_c0_control(0x20));

        assert!(!is_c1_control(0x7f));
        assert!(is_c1_control(0x80));
        assert!(is_c1_control(0x9f));
        assert!(!is_c1_control(0xa0));

        assert!(!is_csi_final(0x3f));
        assert!(is_csi_final(0x40));
        assert!(is_csi_final(0x7e));
        assert!(!is_csi_final(0x7f));

        assert!(!is_csi_leader(0x3b));
        assert!(is_csi_leader(0x3c));
        assert!(is_csi_leader(0x3f));
        assert!(!is_csi_leader(0x40));

        assert!(is_digit(b'0'));
        assert!(is_digit(b'9'));
        assert!(!is_digit(b'a'));

        assert_eq!(digit_value(b'0'), 0);
        assert_eq!(digit_value(b'5'), 5);
        assert_eq!(digit_value(b'9'), 9);
    }

    #[test]
    fn test_parser_is_intermed() {
        assert!(!Parser::is_intermed(0x1f));
        assert!(Parser::is_intermed(0x20));
        assert!(Parser::is_intermed(0x2f));
        assert!(!Parser::is_intermed(0x30));
    }
}
