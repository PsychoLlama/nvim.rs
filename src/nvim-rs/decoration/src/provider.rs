//! Decoration provider registration and lifecycle
//!
//! This module provides infrastructure for managing decoration providers,
//! which are plugins/extensions that dynamically provide decorations.

use std::ffi::c_int;

// =============================================================================
// Provider State
// =============================================================================

/// State of a decoration provider.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProviderState {
    /// Provider not registered
    #[default]
    Unregistered = 0,
    /// Provider is registering
    Registering = 1,
    /// Provider is active
    Active = 2,
    /// Provider is suspended
    Suspended = 3,
    /// Provider is being removed
    Removing = 4,
    /// Provider encountered error
    Error = 5,
}

impl ProviderState {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Registering,
            2 => Self::Active,
            3 => Self::Suspended,
            4 => Self::Removing,
            5 => Self::Error,
            _ => Self::Unregistered,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if provider can provide decorations.
    #[must_use]
    pub const fn can_provide(self) -> bool {
        matches!(self, Self::Active)
    }

    /// Check if provider is in transition.
    #[must_use]
    pub const fn is_transitioning(self) -> bool {
        matches!(self, Self::Registering | Self::Removing)
    }
}

/// FFI: Check if provider can provide.
#[no_mangle]
pub extern "C" fn rs_provider_state_can_provide(state: c_int) -> c_int {
    c_int::from(ProviderState::from_c_int(state).can_provide())
}

// =============================================================================
// Provider Flags
// =============================================================================

/// Flags for decoration providers.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ProviderFlags {
    /// Provider provides highlights
    pub highlights: bool,
    /// Provider provides virtual text
    pub virt_text: bool,
    /// Provider provides virtual lines
    pub virt_lines: bool,
    /// Provider provides signs
    pub signs: bool,
    /// Provider is ephemeral (decorations cleared on redraw)
    pub ephemeral: bool,
    /// Provider is legacy (using old API)
    pub legacy: bool,
    /// Provider wants spell checking info
    pub spell: bool,
}

impl ProviderFlags {
    /// Create default flags (all false).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            highlights: false,
            virt_text: false,
            virt_lines: false,
            signs: false,
            ephemeral: false,
            legacy: false,
            spell: false,
        }
    }

    /// Create flags for highlight-only provider.
    #[must_use]
    pub const fn highlights_only() -> Self {
        Self {
            highlights: true,
            virt_text: false,
            virt_lines: false,
            signs: false,
            ephemeral: false,
            legacy: false,
            spell: false,
        }
    }

    /// Create flags for full provider.
    #[must_use]
    pub const fn full() -> Self {
        Self {
            highlights: true,
            virt_text: true,
            virt_lines: true,
            signs: true,
            ephemeral: false,
            legacy: false,
            spell: false,
        }
    }

    /// Check if any decoration type is enabled.
    #[must_use]
    pub const fn has_any(&self) -> bool {
        self.highlights || self.virt_text || self.virt_lines || self.signs
    }
}

/// FFI: Create default provider flags.
#[no_mangle]
pub extern "C" fn rs_provider_flags_new() -> ProviderFlags {
    ProviderFlags::new()
}

/// FFI: Create highlight-only flags.
#[no_mangle]
pub extern "C" fn rs_provider_flags_highlights_only() -> ProviderFlags {
    ProviderFlags::highlights_only()
}

/// FFI: Create full provider flags.
#[no_mangle]
pub extern "C" fn rs_provider_flags_full() -> ProviderFlags {
    ProviderFlags::full()
}

// =============================================================================
// Provider Info
// =============================================================================

/// Information about a decoration provider.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ProviderInfo {
    /// Provider ID
    pub id: c_int,
    /// Namespace ID
    pub ns_id: u64,
    /// Current state
    pub state: c_int,
    /// Provider flags
    pub flags: ProviderFlags,
    /// Priority (higher = rendered later/on top)
    pub priority: c_int,
    /// Number of times called
    pub call_count: u64,
    /// Total time in provider (microseconds)
    pub total_time_us: i64,
    /// Last error code
    pub last_error: c_int,
}

impl ProviderInfo {
    /// Create new provider info.
    #[must_use]
    pub const fn new(id: c_int, ns_id: u64) -> Self {
        Self {
            id,
            ns_id,
            state: ProviderState::Unregistered as c_int,
            flags: ProviderFlags::new(),
            priority: 0,
            call_count: 0,
            total_time_us: 0,
            last_error: 0,
        }
    }

    /// Get state.
    #[must_use]
    pub const fn get_state(&self) -> ProviderState {
        ProviderState::from_c_int(self.state)
    }

    /// Set state.
    pub fn set_state(&mut self, state: ProviderState) {
        self.state = state as c_int;
    }

    /// Record a call.
    pub fn record_call(&mut self, time_us: i64) {
        self.call_count += 1;
        self.total_time_us += time_us;
    }

    /// Record error.
    pub fn record_error(&mut self, error: c_int) {
        self.last_error = error;
        self.state = ProviderState::Error as c_int;
    }

    /// Check if can provide.
    #[must_use]
    pub const fn can_provide(&self) -> bool {
        self.get_state().can_provide()
    }

    /// Activate provider.
    pub fn activate(&mut self) {
        self.state = ProviderState::Active as c_int;
    }

    /// Suspend provider.
    pub fn suspend(&mut self) {
        self.state = ProviderState::Suspended as c_int;
    }
}

/// FFI: Create provider info.
#[no_mangle]
pub extern "C" fn rs_provider_info_new(id: c_int, ns_id: u64) -> ProviderInfo {
    ProviderInfo::new(id, ns_id)
}

/// FFI: Record call.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_provider_record_call(info: *mut ProviderInfo, time_us: i64) {
    if !info.is_null() {
        (*info).record_call(time_us);
    }
}

/// FFI: Record error.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_provider_record_error(info: *mut ProviderInfo, error: c_int) {
    if !info.is_null() {
        (*info).record_error(error);
    }
}

/// FFI: Activate provider.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_provider_activate(info: *mut ProviderInfo) {
    if !info.is_null() {
        (*info).activate();
    }
}

/// FFI: Suspend provider.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_provider_suspend(info: *mut ProviderInfo) {
    if !info.is_null() {
        (*info).suspend();
    }
}

/// FFI: Check if can provide.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_provider_can_provide(info: *const ProviderInfo) -> c_int {
    if info.is_null() {
        return 0;
    }
    c_int::from((*info).can_provide())
}

// =============================================================================
// Provider Registry Stats
// =============================================================================

/// Statistics for provider registry.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ProviderRegistryStats {
    /// Total providers registered
    pub total_providers: c_int,
    /// Active providers
    pub active_providers: c_int,
    /// Suspended providers
    pub suspended_providers: c_int,
    /// Providers in error state
    pub error_providers: c_int,
    /// Total invocations
    pub total_invocations: u64,
    /// Total errors
    pub total_errors: u64,
}

impl ProviderRegistryStats {
    /// Create new stats.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_providers: 0,
            active_providers: 0,
            suspended_providers: 0,
            error_providers: 0,
            total_invocations: 0,
            total_errors: 0,
        }
    }

    /// Add provider.
    pub fn add_provider(&mut self) {
        self.total_providers += 1;
        self.active_providers += 1;
    }

    /// Remove provider.
    pub fn remove_provider(&mut self, was_active: bool) {
        if was_active && self.active_providers > 0 {
            self.active_providers -= 1;
        }
    }

    /// Record state change.
    pub fn record_state_change(&mut self, from: ProviderState, to: ProviderState) {
        match from {
            ProviderState::Active => {
                if self.active_providers > 0 {
                    self.active_providers -= 1;
                }
            }
            ProviderState::Suspended => {
                if self.suspended_providers > 0 {
                    self.suspended_providers -= 1;
                }
            }
            ProviderState::Error => {
                if self.error_providers > 0 {
                    self.error_providers -= 1;
                }
            }
            _ => {}
        }

        match to {
            ProviderState::Active => self.active_providers += 1,
            ProviderState::Suspended => self.suspended_providers += 1,
            ProviderState::Error => self.error_providers += 1,
            _ => {}
        }
    }

    /// Record invocation.
    pub fn record_invocation(&mut self, had_error: bool) {
        self.total_invocations += 1;
        if had_error {
            self.total_errors += 1;
        }
    }
}

/// FFI: Create registry stats.
#[no_mangle]
pub extern "C" fn rs_provider_registry_stats_new() -> ProviderRegistryStats {
    ProviderRegistryStats::new()
}

/// FFI: Add provider to stats.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_provider_registry_add(stats: *mut ProviderRegistryStats) {
    if !stats.is_null() {
        (*stats).add_provider();
    }
}

/// FFI: Record invocation.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_provider_registry_record_invocation(
    stats: *mut ProviderRegistryStats,
    had_error: c_int,
) {
    if !stats.is_null() {
        (*stats).record_invocation(had_error != 0);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_state() {
        assert!(!ProviderState::Unregistered.can_provide());
        assert!(ProviderState::Active.can_provide());
        assert!(!ProviderState::Suspended.can_provide());

        assert!(ProviderState::Registering.is_transitioning());
        assert!(ProviderState::Removing.is_transitioning());
        assert!(!ProviderState::Active.is_transitioning());
    }

    #[test]
    fn test_provider_flags() {
        let flags = ProviderFlags::new();
        assert!(!flags.has_any());

        let hl_only = ProviderFlags::highlights_only();
        assert!(hl_only.has_any());
        assert!(hl_only.highlights);
        assert!(!hl_only.virt_text);

        let full = ProviderFlags::full();
        assert!(full.has_any());
        assert!(full.highlights);
        assert!(full.virt_text);
        assert!(full.signs);
    }

    #[test]
    fn test_provider_info() {
        let mut info = ProviderInfo::new(1, 100);
        assert_eq!(info.id, 1);
        assert_eq!(info.ns_id, 100);
        assert!(!info.can_provide());

        info.activate();
        assert!(info.can_provide());

        info.record_call(50);
        assert_eq!(info.call_count, 1);
        assert_eq!(info.total_time_us, 50);

        info.suspend();
        assert!(!info.can_provide());
        assert_eq!(info.get_state(), ProviderState::Suspended);

        info.record_error(42);
        assert_eq!(info.last_error, 42);
        assert_eq!(info.get_state(), ProviderState::Error);
    }

    #[test]
    fn test_registry_stats() {
        let mut stats = ProviderRegistryStats::new();
        assert_eq!(stats.total_providers, 0);

        stats.add_provider();
        stats.add_provider();
        assert_eq!(stats.total_providers, 2);
        assert_eq!(stats.active_providers, 2);

        stats.record_state_change(ProviderState::Active, ProviderState::Suspended);
        assert_eq!(stats.active_providers, 1);
        assert_eq!(stats.suspended_providers, 1);

        stats.record_invocation(false);
        stats.record_invocation(true);
        assert_eq!(stats.total_invocations, 2);
        assert_eq!(stats.total_errors, 1);
    }
}
