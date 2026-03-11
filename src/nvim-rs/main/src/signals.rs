//! Signal handling infrastructure
//!
//! This module provides Rust implementations for Neovim's
//! signal handling (SIGINT, SIGTERM, etc.).

use std::ffi::c_int;

// =============================================================================
// Signal Types
// =============================================================================

/// Signal types that Neovim handles.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalType {
    /// Unknown signal
    Unknown = 0,
    /// Interrupt (Ctrl-C)
    Int = 1,
    /// Termination request
    Term = 2,
    /// Hangup
    Hup = 3,
    /// Quit (core dump)
    Quit = 4,
    /// User signal 1
    Usr1 = 5,
    /// User signal 2
    Usr2 = 6,
    /// Window resize
    Winch = 7,
    /// Child process status changed
    Chld = 8,
    /// Continue after stop
    Cont = 9,
    /// Pipe broken
    Pipe = 10,
    /// Alarm
    Alrm = 11,
    /// Segmentation fault
    Segv = 12,
    /// Bus error
    Bus = 13,
    /// Floating point exception
    Fpe = 14,
    /// Illegal instruction
    Ill = 15,
}

impl SignalType {
    /// Create from C signal number.
    /// Note: Signal numbers vary by platform!
    #[must_use]
    pub const fn from_signum(signum: c_int) -> Self {
        // Common POSIX signal numbers (Linux/macOS)
        match signum {
            2 => Self::Int,    // SIGINT
            15 => Self::Term,  // SIGTERM
            1 => Self::Hup,    // SIGHUP
            3 => Self::Quit,   // SIGQUIT
            10 => Self::Usr1,  // SIGUSR1
            12 => Self::Usr2,  // SIGUSR2
            28 => Self::Winch, // SIGWINCH
            17 => Self::Chld,  // SIGCHLD
            18 => Self::Cont,  // SIGCONT
            13 => Self::Pipe,  // SIGPIPE
            14 => Self::Alrm,  // SIGALRM
            11 => Self::Segv,  // SIGSEGV
            7 => Self::Bus,    // SIGBUS
            8 => Self::Fpe,    // SIGFPE
            4 => Self::Ill,    // SIGILL
            _ => Self::Unknown,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if signal is fatal.
    #[must_use]
    pub const fn is_fatal(self) -> bool {
        matches!(
            self,
            Self::Segv | Self::Bus | Self::Fpe | Self::Ill | Self::Quit
        )
    }

    /// Check if signal should cause exit.
    #[must_use]
    pub const fn causes_exit(self) -> bool {
        matches!(self, Self::Term | Self::Hup) || self.is_fatal()
    }

    /// Check if signal should be ignored.
    #[must_use]
    pub const fn should_ignore(self) -> bool {
        matches!(self, Self::Pipe)
    }
}

// =============================================================================
// Signal Handler State
// =============================================================================

/// State for signal handling.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SignalState {
    /// SIGINT received count
    pub int_count: c_int,
    /// Last signal received
    pub last_signal: c_int,
    /// In signal handler
    pub in_handler: bool,
    /// Signals are blocked
    pub blocked: bool,
    /// Pending signal to process
    pub pending: c_int,
    /// Deadly signal received
    pub deadly_received: bool,
}

impl SignalState {
    /// Create new signal state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            int_count: 0,
            last_signal: 0,
            in_handler: false,
            blocked: false,
            pending: 0,
            deadly_received: false,
        }
    }

    /// Record a signal.
    pub fn record(&mut self, signum: c_int) {
        self.last_signal = signum;
        let sig_type = SignalType::from_signum(signum);

        if matches!(sig_type, SignalType::Int) {
            self.int_count += 1;
        }

        if sig_type.is_fatal() {
            self.deadly_received = true;
        }
    }

    /// Check if user wants to interrupt (multiple SIGINT).
    #[must_use]
    pub const fn wants_interrupt(&self) -> bool {
        self.int_count >= 3
    }

    /// Reset interrupt count.
    pub fn reset_interrupt(&mut self) {
        self.int_count = 0;
    }

    /// Check if there's a pending signal.
    #[must_use]
    pub const fn has_pending(&self) -> bool {
        self.pending != 0
    }
}

// =============================================================================
// Signal Mask
// =============================================================================

/// Bit mask for signal sets.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SignalMask {
    /// Bitmask of blocked signals
    pub bits: u64,
}

impl SignalMask {
    /// Create empty mask.
    #[must_use]
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    /// Create mask with all signals.
    #[must_use]
    pub const fn all() -> Self {
        Self { bits: !0 }
    }

    /// Add a signal to the mask.
    pub fn add(&mut self, sig: SignalType) {
        let bit = sig.to_c_int() as u64;
        if bit < 64 {
            self.bits |= 1 << bit;
        }
    }

    /// Remove a signal from the mask.
    pub fn remove(&mut self, sig: SignalType) {
        let bit = sig.to_c_int() as u64;
        if bit < 64 {
            self.bits &= !(1 << bit);
        }
    }

    /// Check if signal is in mask.
    #[must_use]
    pub const fn contains(&self, sig: SignalType) -> bool {
        let bit = sig.to_c_int() as u64;
        if bit < 64 {
            (self.bits & (1 << bit)) != 0
        } else {
            false
        }
    }

    /// Check if mask is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bits == 0
    }
}

// =============================================================================
// Signal Handler Actions
// =============================================================================

/// Action to take for a signal.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignalAction {
    /// Use default handler
    #[default]
    Default = 0,
    /// Ignore the signal
    Ignore = 1,
    /// Custom handler
    Handle = 2,
    /// Block the signal
    Block = 3,
}

impl SignalAction {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Ignore,
            2 => Self::Handle,
            3 => Self::Block,
            _ => Self::Default,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

/// Get recommended action for a signal.
#[must_use]
pub const fn recommended_action(sig: SignalType) -> SignalAction {
    match sig {
        SignalType::Pipe => SignalAction::Ignore,
        SignalType::Winch | SignalType::Chld | SignalType::Cont => SignalAction::Handle,
        SignalType::Int | SignalType::Term | SignalType::Hup => SignalAction::Handle,
        SignalType::Segv | SignalType::Bus | SignalType::Fpe | SignalType::Ill => {
            SignalAction::Default
        }
        _ => SignalAction::Default,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_type() {
        assert_eq!(SignalType::from_signum(2), SignalType::Int);
        assert_eq!(SignalType::from_signum(15), SignalType::Term);
        assert_eq!(SignalType::from_signum(999), SignalType::Unknown);

        assert!(SignalType::Segv.is_fatal());
        assert!(!SignalType::Int.is_fatal());

        assert!(SignalType::Term.causes_exit());
        assert!(!SignalType::Winch.causes_exit());

        assert!(SignalType::Pipe.should_ignore());
        assert!(!SignalType::Int.should_ignore());
    }

    #[test]
    fn test_signal_state() {
        let mut state = SignalState::new();
        assert_eq!(state.int_count, 0);
        assert!(!state.wants_interrupt());

        state.record(2); // SIGINT
        assert_eq!(state.int_count, 1);

        state.record(2);
        state.record(2);
        assert!(state.wants_interrupt());

        state.reset_interrupt();
        assert_eq!(state.int_count, 0);
    }

    #[test]
    fn test_signal_mask() {
        let mut mask = SignalMask::empty();
        assert!(mask.is_empty());

        mask.add(SignalType::Int);
        assert!(!mask.is_empty());
        assert!(mask.contains(SignalType::Int));
        assert!(!mask.contains(SignalType::Term));

        mask.remove(SignalType::Int);
        assert!(mask.is_empty());
    }

    #[test]
    fn test_signal_action() {
        assert_eq!(recommended_action(SignalType::Pipe), SignalAction::Ignore);
        assert_eq!(recommended_action(SignalType::Int), SignalAction::Handle);
        assert_eq!(recommended_action(SignalType::Segv), SignalAction::Default);
    }
}
