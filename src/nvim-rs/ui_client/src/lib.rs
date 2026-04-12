//! UI Client Protocol
//!
//! This crate provides UI client protocol infrastructure for Neovim,
//! including event serialization, handlers, and client attachment.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)] // FFI functions cannot be const
#![allow(clippy::cast_possible_wrap)] // u64 to i64 in statistics

pub mod attach;
pub mod events;
pub mod handler;
pub mod protocol;

use std::ffi::c_int;

// =============================================================================
// UI Client State
// =============================================================================

/// State of UI client connection.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiClientState {
    /// Not connected
    #[default]
    Disconnected = 0,
    /// Connection in progress
    Connecting = 1,
    /// Connected and ready
    Connected = 2,
    /// Disconnecting
    Disconnecting = 3,
    /// Connection failed
    Failed = 4,
}

impl UiClientState {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Connecting,
            2 => Self::Connected,
            3 => Self::Disconnecting,
            4 => Self::Failed,
            _ => Self::Disconnected,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if connected.
    #[must_use]
    pub const fn is_connected(self) -> bool {
        matches!(self, Self::Connected)
    }

    /// Check if in transition.
    #[must_use]
    pub const fn is_transitioning(self) -> bool {
        matches!(self, Self::Connecting | Self::Disconnecting)
    }
}

/// FFI: Check if connected.
#[no_mangle]
pub extern "C" fn rs_ui_client_state_is_connected(state: c_int) -> c_int {
    c_int::from(UiClientState::from_c_int(state).is_connected())
}

// =============================================================================
// UI Client Options
// =============================================================================

/// UI client options.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct UiClientOptions {
    /// Support RGB colors
    pub rgb: bool,
    /// Support `ext_cmdline`
    pub ext_cmdline: bool,
    /// Support `ext_popupmenu`
    pub ext_popupmenu: bool,
    /// Support `ext_tabline`
    pub ext_tabline: bool,
    /// Support `ext_wildmenu`
    pub ext_wildmenu: bool,
    /// Support `ext_messages`
    pub ext_messages: bool,
    /// Support `ext_linegrid`
    pub ext_linegrid: bool,
    /// Support `ext_multigrid`
    pub ext_multigrid: bool,
    /// Support `ext_hlstate`
    pub ext_hlstate: bool,
    /// Support `ext_termcolors`
    pub ext_termcolors: bool,
    /// Override UI
    pub override_ui: bool,
}

impl UiClientOptions {
    /// Create new default options.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            rgb: true,
            ext_cmdline: false,
            ext_popupmenu: false,
            ext_tabline: false,
            ext_wildmenu: false,
            ext_messages: false,
            ext_linegrid: true,
            ext_multigrid: false,
            ext_hlstate: false,
            ext_termcolors: false,
            override_ui: false,
        }
    }

    /// Create TUI-style options.
    #[must_use]
    pub const fn tui() -> Self {
        Self {
            rgb: true,
            ext_cmdline: false,
            ext_popupmenu: false,
            ext_tabline: false,
            ext_wildmenu: false,
            ext_messages: false,
            ext_linegrid: true,
            ext_multigrid: false,
            ext_hlstate: false,
            ext_termcolors: true,
            override_ui: false,
        }
    }

    /// Create GUI-style options.
    #[must_use]
    pub const fn gui() -> Self {
        Self {
            rgb: true,
            ext_cmdline: true,
            ext_popupmenu: true,
            ext_tabline: true,
            ext_wildmenu: true,
            ext_messages: true,
            ext_linegrid: true,
            ext_multigrid: true,
            ext_hlstate: true,
            ext_termcolors: false,
            override_ui: false,
        }
    }

    /// Check if any ext feature is enabled.
    #[must_use]
    pub const fn has_ext_features(&self) -> bool {
        self.ext_cmdline
            || self.ext_popupmenu
            || self.ext_tabline
            || self.ext_wildmenu
            || self.ext_messages
            || self.ext_multigrid
            || self.ext_hlstate
    }
}

/// FFI: Create default UI options.
#[no_mangle]
pub extern "C" fn rs_ui_client_options_new() -> UiClientOptions {
    UiClientOptions::new()
}

/// FFI: Create TUI options.
#[no_mangle]
pub extern "C" fn rs_ui_client_options_tui() -> UiClientOptions {
    UiClientOptions::tui()
}

/// FFI: Create GUI options.
#[no_mangle]
pub extern "C" fn rs_ui_client_options_gui() -> UiClientOptions {
    UiClientOptions::gui()
}

// =============================================================================
// UI Client Info
// =============================================================================

/// Information about a UI client.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct UiClientInfo {
    /// Client ID
    pub client_id: c_int,
    /// Channel ID
    pub channel_id: u64,
    /// Connection state
    pub state: c_int,
    /// Client options
    pub options: UiClientOptions,
    /// Width in cells
    pub width: c_int,
    /// Height in cells
    pub height: c_int,
    /// Rows in pum (popup menu)
    pub pum_row: c_int,
    /// Pum visible
    pub pum_visible: bool,
    /// Is stdin client
    pub stdin_client: bool,
    /// Is embedded UI
    pub embedded: bool,
}

impl UiClientInfo {
    /// Create new client info.
    #[must_use]
    pub const fn new(client_id: c_int, channel_id: u64) -> Self {
        Self {
            client_id,
            channel_id,
            state: UiClientState::Disconnected as c_int,
            options: UiClientOptions::new(),
            width: 80,
            height: 24,
            pum_row: 0,
            pum_visible: false,
            stdin_client: false,
            embedded: false,
        }
    }

    /// Get connection state.
    #[must_use]
    pub const fn get_state(&self) -> UiClientState {
        UiClientState::from_c_int(self.state)
    }

    /// Set connection state.
    pub fn set_state(&mut self, state: UiClientState) {
        self.state = state as c_int;
    }

    /// Set dimensions.
    pub fn set_dimensions(&mut self, width: c_int, height: c_int) {
        self.width = width;
        self.height = height;
    }
}

/// FFI: Create client info.
#[no_mangle]
pub extern "C" fn rs_ui_client_info_new(client_id: c_int, channel_id: u64) -> UiClientInfo {
    UiClientInfo::new(client_id, channel_id)
}

/// FFI: Set client dimensions.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_client_set_dimensions(
    info: *mut UiClientInfo,
    width: c_int,
    height: c_int,
) {
    if !info.is_null() {
        (*info).set_dimensions(width, height);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_client_state() {
        assert!(!UiClientState::Disconnected.is_connected());
        assert!(UiClientState::Connected.is_connected());
        assert!(UiClientState::Connecting.is_transitioning());
        assert!(UiClientState::Disconnecting.is_transitioning());
    }

    #[test]
    fn test_ui_client_options() {
        let default = UiClientOptions::new();
        assert!(default.rgb);
        assert!(default.ext_linegrid);
        assert!(!default.ext_cmdline);

        let tui = UiClientOptions::tui();
        assert!(tui.ext_termcolors);
        assert!(!tui.ext_multigrid);

        let gui = UiClientOptions::gui();
        assert!(gui.has_ext_features());
        assert!(gui.ext_multigrid);
    }

    #[test]
    fn test_ui_client_info() {
        let mut info = UiClientInfo::new(1, 100);
        assert_eq!(info.client_id, 1);
        assert_eq!(info.channel_id, 100);
        assert_eq!(info.get_state(), UiClientState::Disconnected);

        info.set_state(UiClientState::Connected);
        assert_eq!(info.get_state(), UiClientState::Connected);

        info.set_dimensions(120, 40);
        assert_eq!(info.width, 120);
        assert_eq!(info.height, 40);
    }
}
