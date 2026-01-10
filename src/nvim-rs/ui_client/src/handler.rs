//! UI event handlers
//!
//! This module provides handler infrastructure for processing
//! UI events from Neovim.

use std::ffi::c_int;

use crate::protocol::UiEventType;

// =============================================================================
// Handler Status
// =============================================================================

/// Status of event handler execution.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HandlerStatus {
    /// Handler executed successfully
    #[default]
    Ok = 0,
    /// Handler wants event to be propagated
    Propagate = 1,
    /// Handler encountered an error
    Error = 2,
    /// Handler is not registered for this event
    NotRegistered = 3,
    /// Handler is disabled
    Disabled = 4,
    /// Handler timed out
    Timeout = 5,
}

impl HandlerStatus {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Ok,
            1 => Self::Propagate,
            3 => Self::NotRegistered,
            4 => Self::Disabled,
            5 => Self::Timeout,
            // 2 and unrecognized values
            _ => Self::Error,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if handler succeeded.
    #[must_use]
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Ok | Self::Propagate)
    }
}

/// FFI: Check if handler succeeded.
#[no_mangle]
pub extern "C" fn rs_handler_status_is_success(status: c_int) -> c_int {
    c_int::from(HandlerStatus::from_c_int(status).is_success())
}

// =============================================================================
// Handler Flags
// =============================================================================

/// Flags for handler registration.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct HandlerFlags {
    /// Handler is enabled
    pub enabled: bool,
    /// Handler should be called synchronously
    pub synchronous: bool,
    /// Handler consumes the event (no propagation)
    pub exclusive: bool,
    /// Handler should receive all events (including filtered)
    pub receive_all: bool,
    /// Handler is internal (not from plugin)
    pub internal: bool,
    /// Handler has priority
    pub priority: c_int,
}

impl HandlerFlags {
    /// Create default flags.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            enabled: true,
            synchronous: false,
            exclusive: false,
            receive_all: false,
            internal: false,
            priority: 0,
        }
    }

    /// Create flags for synchronous handler.
    #[must_use]
    pub const fn synchronous() -> Self {
        Self {
            enabled: true,
            synchronous: true,
            exclusive: false,
            receive_all: false,
            internal: false,
            priority: 0,
        }
    }

    /// Create flags for exclusive handler.
    #[must_use]
    pub const fn exclusive() -> Self {
        Self {
            enabled: true,
            synchronous: false,
            exclusive: true,
            receive_all: false,
            internal: false,
            priority: 0,
        }
    }

    /// Create flags for internal handler.
    #[must_use]
    pub const fn internal() -> Self {
        Self {
            enabled: true,
            synchronous: true,
            exclusive: false,
            receive_all: false,
            internal: true,
            priority: 100,
        }
    }
}

/// FFI: Create default handler flags.
#[no_mangle]
pub extern "C" fn rs_handler_flags_new() -> HandlerFlags {
    HandlerFlags::new()
}

/// FFI: Create synchronous handler flags.
#[no_mangle]
pub extern "C" fn rs_handler_flags_synchronous() -> HandlerFlags {
    HandlerFlags::synchronous()
}

// =============================================================================
// Handler Registration
// =============================================================================

/// Information about a registered handler.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct HandlerInfo {
    /// Handler ID
    pub id: c_int,
    /// Event type this handler handles
    pub event_type: c_int,
    /// Handler flags
    pub flags: HandlerFlags,
    /// Number of times called
    pub call_count: u64,
    /// Total execution time (microseconds)
    pub total_time_us: i64,
    /// Last execution time (microseconds)
    pub last_time_us: i64,
    /// Last error code
    pub last_error: c_int,
}

impl HandlerInfo {
    /// Create new handler info.
    #[must_use]
    pub const fn new(id: c_int, event_type: UiEventType) -> Self {
        Self {
            id,
            event_type: event_type as c_int,
            flags: HandlerFlags::new(),
            call_count: 0,
            total_time_us: 0,
            last_time_us: 0,
            last_error: 0,
        }
    }

    /// Get event type.
    #[must_use]
    pub const fn get_event_type(&self) -> UiEventType {
        UiEventType::from_c_int(self.event_type)
    }

    /// Record a call.
    pub fn record_call(&mut self, duration_us: i64, error: c_int) {
        self.call_count += 1;
        self.total_time_us += duration_us;
        self.last_time_us = duration_us;
        self.last_error = error;
    }

    /// Get average call time.
    #[must_use]
    pub const fn avg_time_us(&self) -> i64 {
        if self.call_count == 0 {
            0
        } else {
            self.total_time_us / self.call_count as i64
        }
    }

    /// Enable the handler.
    pub fn enable(&mut self) {
        self.flags.enabled = true;
    }

    /// Disable the handler.
    pub fn disable(&mut self) {
        self.flags.enabled = false;
    }
}

/// FFI: Create handler info.
#[no_mangle]
pub extern "C" fn rs_handler_info_new(id: c_int, event_type: c_int) -> HandlerInfo {
    HandlerInfo::new(id, UiEventType::from_c_int(event_type))
}

/// FFI: Record handler call.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_handler_record_call(
    info: *mut HandlerInfo,
    duration_us: i64,
    error: c_int,
) {
    if !info.is_null() {
        (*info).record_call(duration_us, error);
    }
}

/// FFI: Get average call time.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_handler_avg_time(info: *const HandlerInfo) -> i64 {
    if info.is_null() {
        return 0;
    }
    (*info).avg_time_us()
}

/// FFI: Enable handler.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_handler_enable(info: *mut HandlerInfo) {
    if !info.is_null() {
        (*info).enable();
    }
}

/// FFI: Disable handler.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_handler_disable(info: *mut HandlerInfo) {
    if !info.is_null() {
        (*info).disable();
    }
}

// =============================================================================
// Event Filter
// =============================================================================

/// Filter for UI events.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct EventFilter {
    /// Filter grid events
    pub grid_events: bool,
    /// Filter window events
    pub window_events: bool,
    /// Filter cmdline events
    pub cmdline_events: bool,
    /// Filter message events
    pub message_events: bool,
    /// Filter popupmenu events
    pub popupmenu_events: bool,
    /// Specific grid ID to filter (0 = all)
    pub grid_id: c_int,
    /// Minimum priority to pass
    pub min_priority: c_int,
}

impl EventFilter {
    /// Create filter that passes all events.
    #[must_use]
    pub const fn pass_all() -> Self {
        Self {
            grid_events: true,
            window_events: true,
            cmdline_events: true,
            message_events: true,
            popupmenu_events: true,
            grid_id: 0,
            min_priority: 0,
        }
    }

    /// Create filter that only passes grid events.
    #[must_use]
    pub const fn grid_only() -> Self {
        Self {
            grid_events: true,
            window_events: false,
            cmdline_events: false,
            message_events: false,
            popupmenu_events: false,
            grid_id: 0,
            min_priority: 0,
        }
    }

    /// Create filter for specific grid.
    #[must_use]
    pub const fn for_grid(grid_id: c_int) -> Self {
        Self {
            grid_events: true,
            window_events: true,
            cmdline_events: false,
            message_events: false,
            popupmenu_events: false,
            grid_id,
            min_priority: 0,
        }
    }

    /// Check if event type passes filter.
    #[must_use]
    pub const fn passes(&self, event_type: UiEventType) -> bool {
        if event_type.is_grid_event() {
            return self.grid_events;
        }
        if event_type.is_window_event() {
            return self.window_events;
        }
        if event_type.is_cmdline_event() {
            return self.cmdline_events;
        }
        if event_type.is_message_event() {
            return self.message_events;
        }
        if event_type.is_popupmenu_event() {
            return self.popupmenu_events;
        }
        // Pass other events by default
        true
    }
}

/// FFI: Create pass-all filter.
#[no_mangle]
pub extern "C" fn rs_event_filter_pass_all() -> EventFilter {
    EventFilter::pass_all()
}

/// FFI: Create grid-only filter.
#[no_mangle]
pub extern "C" fn rs_event_filter_grid_only() -> EventFilter {
    EventFilter::grid_only()
}

/// FFI: Check if event passes filter.
///
/// # Safety
/// `filter` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_event_filter_passes(
    filter: *const EventFilter,
    event_type: c_int,
) -> c_int {
    if filter.is_null() {
        return 1; // Pass by default if no filter
    }
    c_int::from((*filter).passes(UiEventType::from_c_int(event_type)))
}

// =============================================================================
// Handler Statistics
// =============================================================================

/// Statistics for all handlers.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct HandlerStats {
    /// Total handlers registered
    pub total_handlers: c_int,
    /// Active handlers
    pub active_handlers: c_int,
    /// Total events processed
    pub events_processed: u64,
    /// Total events filtered
    pub events_filtered: u64,
    /// Total handler errors
    pub handler_errors: u64,
    /// Total processing time (microseconds)
    pub total_time_us: i64,
}

impl HandlerStats {
    /// Create new stats.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_handlers: 0,
            active_handlers: 0,
            events_processed: 0,
            events_filtered: 0,
            handler_errors: 0,
            total_time_us: 0,
        }
    }

    /// Record processed event.
    pub fn record_processed(&mut self, time_us: i64, had_error: bool) {
        self.events_processed += 1;
        self.total_time_us += time_us;
        if had_error {
            self.handler_errors += 1;
        }
    }

    /// Record filtered event.
    pub fn record_filtered(&mut self) {
        self.events_filtered += 1;
    }

    /// Add handler.
    pub fn add_handler(&mut self) {
        self.total_handlers += 1;
        self.active_handlers += 1;
    }

    /// Remove handler.
    pub fn remove_handler(&mut self) {
        if self.active_handlers > 0 {
            self.active_handlers -= 1;
        }
    }

    /// Reset statistics.
    pub fn reset(&mut self) {
        self.events_processed = 0;
        self.events_filtered = 0;
        self.handler_errors = 0;
        self.total_time_us = 0;
    }
}

/// FFI: Create handler stats.
#[no_mangle]
pub extern "C" fn rs_handler_stats_new() -> HandlerStats {
    HandlerStats::new()
}

/// FFI: Record processed event.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_handler_stats_record_processed(
    stats: *mut HandlerStats,
    time_us: i64,
    had_error: c_int,
) {
    if !stats.is_null() {
        (*stats).record_processed(time_us, had_error != 0);
    }
}

/// FFI: Reset handler stats.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_handler_stats_reset(stats: *mut HandlerStats) {
    if !stats.is_null() {
        (*stats).reset();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_status() {
        assert!(HandlerStatus::Ok.is_success());
        assert!(HandlerStatus::Propagate.is_success());
        assert!(!HandlerStatus::Error.is_success());
        assert!(!HandlerStatus::NotRegistered.is_success());
    }

    #[test]
    fn test_handler_flags() {
        let flags = HandlerFlags::new();
        assert!(flags.enabled);
        assert!(!flags.synchronous);

        let sync = HandlerFlags::synchronous();
        assert!(sync.synchronous);

        let excl = HandlerFlags::exclusive();
        assert!(excl.exclusive);

        let internal = HandlerFlags::internal();
        assert!(internal.internal);
        assert_eq!(internal.priority, 100);
    }

    #[test]
    fn test_handler_info() {
        let mut info = HandlerInfo::new(1, UiEventType::GridLine);
        assert_eq!(info.id, 1);
        assert_eq!(info.get_event_type(), UiEventType::GridLine);
        assert_eq!(info.call_count, 0);

        info.record_call(100, 0);
        assert_eq!(info.call_count, 1);
        assert_eq!(info.total_time_us, 100);
        assert_eq!(info.avg_time_us(), 100);

        info.record_call(200, 0);
        assert_eq!(info.call_count, 2);
        assert_eq!(info.avg_time_us(), 150);

        info.disable();
        assert!(!info.flags.enabled);

        info.enable();
        assert!(info.flags.enabled);
    }

    #[test]
    fn test_event_filter() {
        let all = EventFilter::pass_all();
        assert!(all.passes(UiEventType::GridLine));
        assert!(all.passes(UiEventType::WinPos));
        assert!(all.passes(UiEventType::CmdlineShow));

        let grid_only = EventFilter::grid_only();
        assert!(grid_only.passes(UiEventType::GridLine));
        assert!(!grid_only.passes(UiEventType::WinPos));
        assert!(!grid_only.passes(UiEventType::CmdlineShow));
    }

    #[test]
    fn test_handler_stats() {
        let mut stats = HandlerStats::new();
        assert_eq!(stats.events_processed, 0);

        stats.add_handler();
        assert_eq!(stats.total_handlers, 1);
        assert_eq!(stats.active_handlers, 1);

        stats.record_processed(100, false);
        assert_eq!(stats.events_processed, 1);
        assert_eq!(stats.total_time_us, 100);

        stats.record_processed(50, true);
        assert_eq!(stats.events_processed, 2);
        assert_eq!(stats.handler_errors, 1);

        stats.record_filtered();
        assert_eq!(stats.events_filtered, 1);

        stats.remove_handler();
        assert_eq!(stats.active_handlers, 0);

        stats.reset();
        assert_eq!(stats.events_processed, 0);
    }
}
