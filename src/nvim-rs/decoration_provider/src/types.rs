//! Type definitions for decoration provider system
//!
//! These types match the structures in `decoration_defs.h` and provide
//! Rust-safe wrappers for FFI interaction.

use std::ffi::{c_int, c_void};

use crate::constants::{
    KSH_CONCEAL, KSH_CONCEAL_LINES, KSH_HL_EOL, KSH_IS_SIGN, KSH_SPELL_OFF, KSH_SPELL_ON,
    KSH_UI_WATCHED, KSH_UI_WATCHED_OVERLAY, KVT_HIDE, KVT_IS_LINES, KVT_LINES_ABOVE,
    KVT_REPEAT_LINEBREAK,
};

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to DecorProvider struct in C.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct DecorProviderHandle(*mut c_void);

impl DecorProviderHandle {
    /// Create a null handle.
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if the handle is null.
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get raw pointer.
    #[must_use]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to buffer (buf_T).
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct BufHandle(*mut c_void);

impl BufHandle {
    /// Create a null handle.
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if the handle is null.
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get raw pointer.
    #[must_use]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to window (win_T).
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct WinHandle(*mut c_void);

impl WinHandle {
    /// Create a null handle.
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if the handle is null.
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get raw pointer.
    #[must_use]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

// =============================================================================
// Provider State Enum
// =============================================================================

/// State of a decoration provider.
/// Matches the enum in DecorProvider struct.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DecorProviderState {
    /// Provider is active and will be invoked.
    Active = 1,
    /// Provider is disabled for current window only.
    WinDisabled = 2,
    /// Provider is disabled for current redraw cycle.
    RedrawDisabled = 3,
    /// Provider is fully disabled (default).
    #[default]
    Disabled = 4,
}

impl DecorProviderState {
    /// Create from C int value.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Active,
            2 => Self::WinDisabled,
            3 => Self::RedrawDisabled,
            _ => Self::Disabled,
        }
    }

    /// Convert to C int value.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if provider can receive callbacks.
    #[must_use]
    pub const fn can_receive_callbacks(self) -> bool {
        matches!(self, Self::Active)
    }

    /// Check if provider can receive window callbacks.
    #[must_use]
    pub const fn can_receive_win_callbacks(self) -> bool {
        matches!(self, Self::Active)
    }

    /// Check if provider is disabled.
    #[must_use]
    pub const fn is_disabled(self) -> bool {
        matches!(self, Self::Disabled)
    }

    /// Check if provider is temporarily disabled (win or redraw).
    #[must_use]
    pub const fn is_temporarily_disabled(self) -> bool {
        matches!(self, Self::WinDisabled | Self::RedrawDisabled)
    }
}

// =============================================================================
// Virtual Text Position
// =============================================================================

/// Position type for virtual text.
/// Matches VirtTextPos enum in decoration_defs.h.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VirtTextPos {
    /// Virtual text at end of line.
    #[default]
    EndOfLine = 0,
    /// Virtual text at end of line, right-aligned.
    EndOfLineRightAlign = 1,
    /// Virtual text inline (inserted into text).
    Inline = 2,
    /// Virtual text overlaid on existing text.
    Overlay = 3,
    /// Virtual text right-aligned in window.
    RightAlign = 4,
    /// Virtual text at specific window column.
    WinCol = 5,
}

impl VirtTextPos {
    /// Create from C int value.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::EndOfLine),
            1 => Some(Self::EndOfLineRightAlign),
            2 => Some(Self::Inline),
            3 => Some(Self::Overlay),
            4 => Some(Self::RightAlign),
            5 => Some(Self::WinCol),
            _ => None,
        }
    }

    /// Convert to C int value.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if position is at end of line (either variant).
    #[must_use]
    pub const fn is_eol(self) -> bool {
        matches!(self, Self::EndOfLine | Self::EndOfLineRightAlign)
    }

    /// Check if position is right-aligned (any variant).
    #[must_use]
    pub const fn is_right_aligned(self) -> bool {
        matches!(self, Self::EndOfLineRightAlign | Self::RightAlign)
    }

    /// Check if position is inline or overlay.
    #[must_use]
    pub const fn is_inline_or_overlay(self) -> bool {
        matches!(self, Self::Inline | Self::Overlay)
    }
}

// =============================================================================
// Highlight Mode
// =============================================================================

/// Mode for how highlights are applied.
/// Matches HlMode enum in decoration_defs.h.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HlMode {
    /// Mode unknown/unset.
    #[default]
    Unknown = 0,
    /// Replace existing highlight.
    Replace = 1,
    /// Combine with existing highlight.
    Combine = 2,
    /// Blend with existing highlight.
    Blend = 3,
}

impl HlMode {
    /// Create from C int value.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Unknown),
            1 => Some(Self::Replace),
            2 => Some(Self::Combine),
            3 => Some(Self::Blend),
            _ => None,
        }
    }

    /// Convert to C int value.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if mode affects existing highlights.
    #[must_use]
    pub const fn affects_existing(self) -> bool {
        matches!(self, Self::Combine | Self::Blend)
    }
}

// =============================================================================
// Decoration Flags
// =============================================================================

/// Flags for sign/highlight decorations.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DecorSignFlags(u16);

impl DecorSignFlags {
    /// Create empty flags.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Create from raw value.
    #[must_use]
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    /// Get raw value.
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// Check if is a sign.
    #[must_use]
    pub const fn is_sign(self) -> bool {
        (self.0 & KSH_IS_SIGN) != 0
    }

    /// Check if highlight extends to EOL.
    #[must_use]
    pub const fn hl_eol(self) -> bool {
        (self.0 & KSH_HL_EOL) != 0
    }

    /// Check if UI is watching.
    #[must_use]
    pub const fn ui_watched(self) -> bool {
        (self.0 & KSH_UI_WATCHED) != 0
    }

    /// Check if UI is watching overlay.
    #[must_use]
    pub const fn ui_watched_overlay(self) -> bool {
        (self.0 & KSH_UI_WATCHED_OVERLAY) != 0
    }

    /// Check if spell is on.
    #[must_use]
    pub const fn spell_on(self) -> bool {
        (self.0 & KSH_SPELL_ON) != 0
    }

    /// Check if spell is off.
    #[must_use]
    pub const fn spell_off(self) -> bool {
        (self.0 & KSH_SPELL_OFF) != 0
    }

    /// Check if conceal is enabled.
    #[must_use]
    pub const fn conceal(self) -> bool {
        (self.0 & KSH_CONCEAL) != 0
    }

    /// Check if conceal lines is enabled.
    #[must_use]
    pub const fn conceal_lines(self) -> bool {
        (self.0 & KSH_CONCEAL_LINES) != 0
    }

    /// Set is_sign flag.
    #[must_use]
    pub const fn with_sign(self) -> Self {
        Self(self.0 | KSH_IS_SIGN)
    }

    /// Set hl_eol flag.
    #[must_use]
    pub const fn with_hl_eol(self) -> Self {
        Self(self.0 | KSH_HL_EOL)
    }

    /// Set ui_watched flag.
    #[must_use]
    pub const fn with_ui_watched(self) -> Self {
        Self(self.0 | KSH_UI_WATCHED)
    }
}

/// Flags for virtual text decorations.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DecorVirtFlags(u8);

impl DecorVirtFlags {
    /// Create empty flags.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Create from raw value.
    #[must_use]
    pub const fn from_raw(raw: u8) -> Self {
        Self(raw)
    }

    /// Get raw value.
    #[must_use]
    pub const fn raw(self) -> u8 {
        self.0
    }

    /// Check if is virtual lines (not text).
    #[must_use]
    pub const fn is_lines(self) -> bool {
        (self.0 & KVT_IS_LINES) != 0
    }

    /// Check if hidden.
    #[must_use]
    pub const fn is_hidden(self) -> bool {
        (self.0 & KVT_HIDE) != 0
    }

    /// Check if lines are above.
    #[must_use]
    pub const fn lines_above(self) -> bool {
        (self.0 & KVT_LINES_ABOVE) != 0
    }

    /// Check if should repeat linebreak.
    #[must_use]
    pub const fn repeat_linebreak(self) -> bool {
        (self.0 & KVT_REPEAT_LINEBREAK) != 0
    }

    /// Set is_lines flag.
    #[must_use]
    pub const fn with_lines(self) -> Self {
        Self(self.0 | KVT_IS_LINES)
    }

    /// Set hidden flag.
    #[must_use]
    pub const fn with_hidden(self) -> Self {
        Self(self.0 | KVT_HIDE)
    }

    /// Set lines_above flag.
    #[must_use]
    pub const fn with_lines_above(self) -> Self {
        Self(self.0 | KVT_LINES_ABOVE)
    }
}

// =============================================================================
// FFI Exports - Provider State
// =============================================================================

/// Convert provider state from C int.
#[no_mangle]
pub extern "C" fn rs_decor_provider_state_from_int(val: c_int) -> c_int {
    DecorProviderState::from_c_int(val).to_c_int()
}

/// Check if provider state can receive callbacks.
#[no_mangle]
pub extern "C" fn rs_decor_provider_state_can_receive(state: c_int) -> bool {
    DecorProviderState::from_c_int(state).can_receive_callbacks()
}

/// Check if provider state is temporarily disabled.
#[no_mangle]
pub extern "C" fn rs_decor_provider_state_temp_disabled(state: c_int) -> bool {
    DecorProviderState::from_c_int(state).is_temporarily_disabled()
}

// =============================================================================
// FFI Exports - VirtTextPos
// =============================================================================

/// Check if virtual text position is at EOL.
#[no_mangle]
pub extern "C" fn rs_virt_text_pos_is_eol(pos: c_int) -> bool {
    VirtTextPos::from_c_int(pos).is_some_and(VirtTextPos::is_eol)
}

/// Check if virtual text position is right-aligned.
#[no_mangle]
pub extern "C" fn rs_virt_text_pos_is_right_aligned(pos: c_int) -> bool {
    VirtTextPos::from_c_int(pos).is_some_and(VirtTextPos::is_right_aligned)
}

/// Check if virtual text position is inline or overlay.
#[no_mangle]
pub extern "C" fn rs_virt_text_pos_is_inline_or_overlay(pos: c_int) -> bool {
    VirtTextPos::from_c_int(pos).is_some_and(VirtTextPos::is_inline_or_overlay)
}

// =============================================================================
// FFI Exports - HlMode
// =============================================================================

/// Check if highlight mode affects existing highlights.
#[no_mangle]
pub extern "C" fn rs_hl_mode_affects_existing(mode: c_int) -> bool {
    HlMode::from_c_int(mode).is_some_and(HlMode::affects_existing)
}

// =============================================================================
// FFI Exports - DecorSignFlags
// =============================================================================

/// Create empty sign flags.
#[no_mangle]
pub extern "C" fn rs_decor_sign_flags_empty() -> u16 {
    DecorSignFlags::empty().raw()
}

/// Check sign flags: is_sign.
#[no_mangle]
pub extern "C" fn rs_decor_sign_flags_is_sign(flags: u16) -> bool {
    DecorSignFlags::from_raw(flags).is_sign()
}

/// Check sign flags: hl_eol.
#[no_mangle]
pub extern "C" fn rs_decor_sign_flags_hl_eol(flags: u16) -> bool {
    DecorSignFlags::from_raw(flags).hl_eol()
}

/// Check sign flags: ui_watched.
#[no_mangle]
pub extern "C" fn rs_decor_sign_flags_ui_watched(flags: u16) -> bool {
    DecorSignFlags::from_raw(flags).ui_watched()
}

/// Check sign flags: spell_on.
#[no_mangle]
pub extern "C" fn rs_decor_sign_flags_spell_on(flags: u16) -> bool {
    DecorSignFlags::from_raw(flags).spell_on()
}

/// Check sign flags: spell_off.
#[no_mangle]
pub extern "C" fn rs_decor_sign_flags_spell_off(flags: u16) -> bool {
    DecorSignFlags::from_raw(flags).spell_off()
}

/// Check sign flags: conceal.
#[no_mangle]
pub extern "C" fn rs_decor_sign_flags_conceal(flags: u16) -> bool {
    DecorSignFlags::from_raw(flags).conceal()
}

// =============================================================================
// FFI Exports - DecorVirtFlags
// =============================================================================

/// Create empty virt flags.
#[no_mangle]
pub extern "C" fn rs_decor_virt_flags_empty() -> u8 {
    DecorVirtFlags::empty().raw()
}

/// Check virt flags: is_lines.
#[no_mangle]
pub extern "C" fn rs_decor_virt_flags_is_lines(flags: u8) -> bool {
    DecorVirtFlags::from_raw(flags).is_lines()
}

/// Check virt flags: is_hidden.
#[no_mangle]
pub extern "C" fn rs_decor_virt_flags_is_hidden(flags: u8) -> bool {
    DecorVirtFlags::from_raw(flags).is_hidden()
}

/// Check virt flags: lines_above.
#[no_mangle]
pub extern "C" fn rs_decor_virt_flags_lines_above(flags: u8) -> bool {
    DecorVirtFlags::from_raw(flags).lines_above()
}

/// Check virt flags: repeat_linebreak.
#[no_mangle]
pub extern "C" fn rs_decor_virt_flags_repeat_linebreak(flags: u8) -> bool {
    DecorVirtFlags::from_raw(flags).repeat_linebreak()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_null() {
        let h = DecorProviderHandle::null();
        assert!(h.is_null());

        let b = BufHandle::null();
        assert!(b.is_null());

        let w = WinHandle::null();
        assert!(w.is_null());
    }

    #[test]
    fn test_handle_sizes() {
        assert_eq!(
            std::mem::size_of::<DecorProviderHandle>(),
            std::mem::size_of::<*mut c_void>()
        );
        assert_eq!(
            std::mem::size_of::<BufHandle>(),
            std::mem::size_of::<*mut c_void>()
        );
        assert_eq!(
            std::mem::size_of::<WinHandle>(),
            std::mem::size_of::<*mut c_void>()
        );
    }

    #[test]
    fn test_provider_state() {
        assert_eq!(
            DecorProviderState::from_c_int(1),
            DecorProviderState::Active
        );
        assert_eq!(
            DecorProviderState::from_c_int(2),
            DecorProviderState::WinDisabled
        );
        assert_eq!(
            DecorProviderState::from_c_int(3),
            DecorProviderState::RedrawDisabled
        );
        assert_eq!(
            DecorProviderState::from_c_int(4),
            DecorProviderState::Disabled
        );
        assert_eq!(
            DecorProviderState::from_c_int(100),
            DecorProviderState::Disabled
        );

        assert!(DecorProviderState::Active.can_receive_callbacks());
        assert!(!DecorProviderState::Disabled.can_receive_callbacks());
        assert!(!DecorProviderState::WinDisabled.can_receive_callbacks());

        assert!(DecorProviderState::WinDisabled.is_temporarily_disabled());
        assert!(DecorProviderState::RedrawDisabled.is_temporarily_disabled());
        assert!(!DecorProviderState::Active.is_temporarily_disabled());
        assert!(!DecorProviderState::Disabled.is_temporarily_disabled());
    }

    #[test]
    fn test_virt_text_pos() {
        assert_eq!(VirtTextPos::from_c_int(0), Some(VirtTextPos::EndOfLine));
        assert_eq!(
            VirtTextPos::from_c_int(1),
            Some(VirtTextPos::EndOfLineRightAlign)
        );
        assert_eq!(VirtTextPos::from_c_int(2), Some(VirtTextPos::Inline));
        assert_eq!(VirtTextPos::from_c_int(3), Some(VirtTextPos::Overlay));
        assert_eq!(VirtTextPos::from_c_int(4), Some(VirtTextPos::RightAlign));
        assert_eq!(VirtTextPos::from_c_int(5), Some(VirtTextPos::WinCol));
        assert_eq!(VirtTextPos::from_c_int(100), None);

        assert!(VirtTextPos::EndOfLine.is_eol());
        assert!(VirtTextPos::EndOfLineRightAlign.is_eol());
        assert!(!VirtTextPos::Inline.is_eol());

        assert!(VirtTextPos::EndOfLineRightAlign.is_right_aligned());
        assert!(VirtTextPos::RightAlign.is_right_aligned());
        assert!(!VirtTextPos::EndOfLine.is_right_aligned());

        assert!(VirtTextPos::Inline.is_inline_or_overlay());
        assert!(VirtTextPos::Overlay.is_inline_or_overlay());
        assert!(!VirtTextPos::EndOfLine.is_inline_or_overlay());
    }

    #[test]
    fn test_hl_mode() {
        assert_eq!(HlMode::from_c_int(0), Some(HlMode::Unknown));
        assert_eq!(HlMode::from_c_int(1), Some(HlMode::Replace));
        assert_eq!(HlMode::from_c_int(2), Some(HlMode::Combine));
        assert_eq!(HlMode::from_c_int(3), Some(HlMode::Blend));
        assert_eq!(HlMode::from_c_int(100), None);

        assert!(!HlMode::Unknown.affects_existing());
        assert!(!HlMode::Replace.affects_existing());
        assert!(HlMode::Combine.affects_existing());
        assert!(HlMode::Blend.affects_existing());
    }

    #[test]
    fn test_sign_flags() {
        let flags = DecorSignFlags::empty();
        assert!(!flags.is_sign());
        assert!(!flags.hl_eol());

        let flags = flags.with_sign();
        assert!(flags.is_sign());
        assert!(!flags.hl_eol());

        let flags = flags.with_hl_eol();
        assert!(flags.is_sign());
        assert!(flags.hl_eol());

        let flags = DecorSignFlags::from_raw(KSH_UI_WATCHED | KSH_SPELL_ON);
        assert!(flags.ui_watched());
        assert!(flags.spell_on());
        assert!(!flags.spell_off());
    }

    #[test]
    fn test_virt_flags() {
        let flags = DecorVirtFlags::empty();
        assert!(!flags.is_lines());
        assert!(!flags.is_hidden());

        let flags = flags.with_lines();
        assert!(flags.is_lines());
        assert!(!flags.is_hidden());

        let flags = flags.with_hidden();
        assert!(flags.is_lines());
        assert!(flags.is_hidden());

        let flags = DecorVirtFlags::from_raw(KVT_LINES_ABOVE | KVT_REPEAT_LINEBREAK);
        assert!(flags.lines_above());
        assert!(flags.repeat_linebreak());
        assert!(!flags.is_lines());
    }
}
