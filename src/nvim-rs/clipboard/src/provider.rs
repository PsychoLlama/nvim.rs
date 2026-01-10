//! Clipboard provider abstraction
//!
//! This module provides types for clipboard provider management,
//! including capability detection and status tracking.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Provider Type
// =============================================================================

/// Type of clipboard provider
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClipboardProvider {
    /// No provider available
    #[default]
    None = 0,
    /// System provider (xclip, xsel, pbcopy, etc.)
    System = 1,
    /// OSC 52 terminal sequence
    Osc52 = 2,
    /// Wayland (wl-copy/wl-paste)
    Wayland = 3,
    /// X11 (xclip, xsel)
    X11 = 4,
    /// Windows (clip.exe, powershell)
    Windows = 5,
    /// macOS (pbcopy/pbpaste)
    MacOS = 6,
    /// tmux buffer
    Tmux = 7,
    /// Custom Lua provider
    Custom = 8,
}

impl ClipboardProvider {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::System),
            2 => Some(Self::Osc52),
            3 => Some(Self::Wayland),
            4 => Some(Self::X11),
            5 => Some(Self::Windows),
            6 => Some(Self::MacOS),
            7 => Some(Self::Tmux),
            8 => Some(Self::Custom),
            _ => None,
        }
    }

    /// Convert to raw value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Check if provider is available
    pub const fn is_available(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Check if provider supports async operations
    pub const fn supports_async(self) -> bool {
        matches!(self, Self::Osc52 | Self::Custom)
    }

    /// Check if provider supports selection types
    pub const fn supports_selections(self) -> bool {
        matches!(self, Self::X11 | Self::Wayland)
    }

    /// Get provider name
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::System => "system",
            Self::Osc52 => "osc52",
            Self::Wayland => "wayland",
            Self::X11 => "x11",
            Self::Windows => "windows",
            Self::MacOS => "macos",
            Self::Tmux => "tmux",
            Self::Custom => "custom",
        }
    }
}

// =============================================================================
// Provider Capabilities
// =============================================================================

/// Provider capability flags
pub const CAP_READ: u32 = 0x0001;
pub const CAP_WRITE: u32 = 0x0002;
pub const CAP_PRIMARY: u32 = 0x0004;
pub const CAP_CLIPBOARD: u32 = 0x0008;
pub const CAP_ASYNC: u32 = 0x0010;
pub const CAP_BATCH: u32 = 0x0020;

/// Provider capabilities
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ProviderCapabilities {
    caps: u32,
}

impl ProviderCapabilities {
    /// Create with no capabilities
    pub const fn none() -> Self {
        Self { caps: 0 }
    }

    /// Create from raw value
    pub const fn from_raw(caps: u32) -> Self {
        Self { caps }
    }

    /// Get raw value
    pub const fn as_raw(self) -> u32 {
        self.caps
    }

    /// Create with basic read/write for clipboard
    pub const fn basic() -> Self {
        Self {
            caps: CAP_READ | CAP_WRITE | CAP_CLIPBOARD,
        }
    }

    /// Create with full X11/Wayland capabilities
    pub const fn full() -> Self {
        Self {
            caps: CAP_READ | CAP_WRITE | CAP_PRIMARY | CAP_CLIPBOARD,
        }
    }

    /// Check if can read
    pub const fn can_read(self) -> bool {
        (self.caps & CAP_READ) != 0
    }

    /// Check if can write
    pub const fn can_write(self) -> bool {
        (self.caps & CAP_WRITE) != 0
    }

    /// Check if supports PRIMARY selection
    pub const fn has_primary(self) -> bool {
        (self.caps & CAP_PRIMARY) != 0
    }

    /// Check if supports CLIPBOARD selection
    pub const fn has_clipboard(self) -> bool {
        (self.caps & CAP_CLIPBOARD) != 0
    }

    /// Check if supports async operations
    pub const fn has_async(self) -> bool {
        (self.caps & CAP_ASYNC) != 0
    }

    /// Check if supports batch operations
    pub const fn has_batch(self) -> bool {
        (self.caps & CAP_BATCH) != 0
    }

    /// Set capability
    pub fn set_capability(&mut self, cap: u32, value: bool) {
        if value {
            self.caps |= cap;
        } else {
            self.caps &= !cap;
        }
    }
}

// =============================================================================
// Provider Status
// =============================================================================

/// Status of clipboard provider
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProviderStatus {
    /// Not initialized
    #[default]
    Unknown = 0,
    /// Provider available and working
    Available = 1,
    /// Provider not available
    Unavailable = 2,
    /// Provider error
    Error = 3,
    /// Checking provider
    Checking = 4,
}

impl ProviderStatus {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Unknown),
            1 => Some(Self::Available),
            2 => Some(Self::Unavailable),
            3 => Some(Self::Error),
            4 => Some(Self::Checking),
            _ => None,
        }
    }

    /// Check if provider is usable
    pub const fn is_usable(self) -> bool {
        matches!(self, Self::Available)
    }

    /// Check if status is terminal
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Available | Self::Unavailable | Self::Error)
    }
}

// =============================================================================
// Provider State
// =============================================================================

/// Full provider state
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProviderState {
    /// Current provider type
    pub provider: ClipboardProvider,
    /// Provider status
    pub status: ProviderStatus,
    /// Provider capabilities
    pub capabilities: ProviderCapabilities,
    /// Whether warning has been shown
    pub warned: bool,
}

impl Default for ProviderState {
    fn default() -> Self {
        Self {
            provider: ClipboardProvider::None,
            status: ProviderStatus::Unknown,
            capabilities: ProviderCapabilities::none(),
            warned: false,
        }
    }
}

impl ProviderState {
    /// Create new empty state
    pub const fn new() -> Self {
        Self {
            provider: ClipboardProvider::None,
            status: ProviderStatus::Unknown,
            capabilities: ProviderCapabilities::none(),
            warned: false,
        }
    }

    /// Check if provider is available
    pub const fn is_available(&self) -> bool {
        self.provider.is_available() && self.status.is_usable()
    }

    /// Check if we should warn about missing provider
    pub fn should_warn(&self) -> bool {
        !self.is_available() && !self.warned
    }

    /// Mark as warned
    pub fn mark_warned(&mut self) {
        self.warned = true;
    }

    /// Set provider
    pub fn set_provider(&mut self, provider: ClipboardProvider, caps: ProviderCapabilities) {
        self.provider = provider;
        self.capabilities = caps;
        self.status = if provider.is_available() {
            ProviderStatus::Available
        } else {
            ProviderStatus::Unavailable
        };
    }

    /// Set error status
    pub fn set_error(&mut self) {
        self.status = ProviderStatus::Error;
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if provider is valid
#[no_mangle]
pub extern "C" fn rs_clipboard_provider_valid(provider: c_int) -> c_int {
    c_int::from(ClipboardProvider::from_raw(provider).is_some())
}

/// FFI export: Check if provider is available
#[no_mangle]
pub extern "C" fn rs_clipboard_provider_available(provider: c_int) -> c_int {
    ClipboardProvider::from_raw(provider).map_or(0, |p| c_int::from(p.is_available()))
}

/// FFI export: Check if provider supports async
#[no_mangle]
pub extern "C" fn rs_clipboard_provider_supports_async(provider: c_int) -> c_int {
    ClipboardProvider::from_raw(provider).map_or(0, |p| c_int::from(p.supports_async()))
}

/// FFI export: Check if capabilities can read
#[no_mangle]
pub extern "C" fn rs_clipboard_caps_can_read(caps: u32) -> c_int {
    c_int::from(ProviderCapabilities::from_raw(caps).can_read())
}

/// FFI export: Check if capabilities can write
#[no_mangle]
pub extern "C" fn rs_clipboard_caps_can_write(caps: u32) -> c_int {
    c_int::from(ProviderCapabilities::from_raw(caps).can_write())
}

/// FFI export: Check if status is usable
#[no_mangle]
pub extern "C" fn rs_clipboard_status_usable(status: c_int) -> c_int {
    ProviderStatus::from_raw(status).map_or(0, |s| c_int::from(s.is_usable()))
}

/// FFI export: Create new provider state
#[no_mangle]
pub extern "C" fn rs_clipboard_provider_state_new() -> ProviderState {
    ProviderState::new()
}

/// FFI export: Check if provider state is available
#[no_mangle]
pub extern "C" fn rs_clipboard_provider_state_available(state: *const ProviderState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*state).is_available() })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::borrow_as_ptr)]
mod tests {
    use super::*;

    #[test]
    fn test_provider() {
        assert_eq!(
            ClipboardProvider::from_raw(0),
            Some(ClipboardProvider::None)
        );
        assert_eq!(ClipboardProvider::from_raw(4), Some(ClipboardProvider::X11));
        assert_eq!(ClipboardProvider::from_raw(100), None);

        assert!(!ClipboardProvider::None.is_available());
        assert!(ClipboardProvider::System.is_available());

        assert!(ClipboardProvider::X11.supports_selections());
        assert!(!ClipboardProvider::MacOS.supports_selections());

        assert!(ClipboardProvider::Osc52.supports_async());
    }

    #[test]
    fn test_capabilities() {
        let caps = ProviderCapabilities::none();
        assert!(!caps.can_read());
        assert!(!caps.can_write());

        let caps = ProviderCapabilities::basic();
        assert!(caps.can_read());
        assert!(caps.can_write());
        assert!(caps.has_clipboard());
        assert!(!caps.has_primary());

        let caps = ProviderCapabilities::full();
        assert!(caps.has_primary());
        assert!(caps.has_clipboard());
    }

    #[test]
    fn test_status() {
        assert_eq!(ProviderStatus::from_raw(0), Some(ProviderStatus::Unknown));
        assert_eq!(ProviderStatus::from_raw(1), Some(ProviderStatus::Available));
        assert_eq!(ProviderStatus::from_raw(100), None);

        assert!(ProviderStatus::Available.is_usable());
        assert!(!ProviderStatus::Unavailable.is_usable());
        assert!(ProviderStatus::Available.is_terminal());
    }

    #[test]
    fn test_provider_state() {
        let mut state = ProviderState::new();
        assert!(!state.is_available());

        state.set_provider(ClipboardProvider::X11, ProviderCapabilities::full());
        assert!(state.is_available());
        assert!(state.capabilities.has_primary());
    }

    #[test]
    fn test_provider_state_warning() {
        let mut state = ProviderState::new();
        assert!(state.should_warn());

        state.mark_warned();
        assert!(!state.should_warn());
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_clipboard_provider_valid(0), 1);
        assert_eq!(rs_clipboard_provider_valid(100), 0);

        assert_eq!(rs_clipboard_provider_available(0), 0);
        assert_eq!(rs_clipboard_provider_available(1), 1);

        assert_eq!(rs_clipboard_caps_can_read(CAP_READ), 1);
        assert_eq!(rs_clipboard_caps_can_read(0), 0);
    }
}
