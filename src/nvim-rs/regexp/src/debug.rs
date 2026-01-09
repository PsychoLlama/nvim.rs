//! Debug and profiling support for regex operations.
//!
//! This module provides:
//! - Debug output for pattern dumps
//! - Profiling hooks for performance analysis
//! - State inspection utilities
//!
//! Debug output is controlled by REGEXP_DEBUG feature flag.

use std::ffi::c_int;

// =============================================================================
// Debug Configuration
// =============================================================================

/// Debug output level.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DebugLevel {
    /// No debug output.
    #[default]
    Off = 0,
    /// Basic match information.
    Basic = 1,
    /// Detailed state transitions.
    Verbose = 2,
    /// Full execution trace.
    Trace = 3,
}

impl From<c_int> for DebugLevel {
    fn from(level: c_int) -> Self {
        match level {
            0 => Self::Off,
            1 => Self::Basic,
            2 => Self::Verbose,
            _ => Self::Trace,
        }
    }
}

impl From<DebugLevel> for c_int {
    fn from(level: DebugLevel) -> Self {
        level as c_int
    }
}

// =============================================================================
// Debug State
// =============================================================================

/// Global debug state.
static mut DEBUG_LEVEL: DebugLevel = DebugLevel::Off;

/// Get current debug level.
///
/// # Safety
/// Must be called from single-threaded context or with proper synchronization.
#[inline]
pub unsafe fn get_debug_level() -> DebugLevel {
    // SAFETY: Caller guarantees single-threaded access
    std::ptr::addr_of!(DEBUG_LEVEL).read()
}

/// Set debug level.
///
/// # Safety
/// Must be called from single-threaded context or with proper synchronization.
#[inline]
pub unsafe fn set_debug_level(level: DebugLevel) {
    // SAFETY: Caller guarantees single-threaded access
    std::ptr::addr_of_mut!(DEBUG_LEVEL).write(level);
}

/// Check if debug output is enabled.
///
/// # Safety
/// Must be called from single-threaded context.
#[inline]
pub unsafe fn is_debug_enabled() -> bool {
    DEBUG_LEVEL != DebugLevel::Off
}

// =============================================================================
// Pattern Dump
// =============================================================================

/// Dump format for pattern output.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DumpFormat {
    /// Human-readable format.
    #[default]
    Text = 0,
    /// Hex dump of bytecode.
    Hex = 1,
    /// Graphviz DOT format (for NFA).
    Dot = 2,
}

/// Pattern dump options.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DumpOptions {
    /// Output format.
    pub format: DumpFormat,
    /// Include line numbers.
    pub line_numbers: bool,
    /// Include state IDs.
    pub state_ids: bool,
    /// Maximum output length (0 = unlimited).
    pub max_len: usize,
}

impl DumpOptions {
    /// Create new dump options with defaults.
    pub const fn new() -> Self {
        Self {
            format: DumpFormat::Text,
            line_numbers: true,
            state_ids: true,
            max_len: 0,
        }
    }

    /// Set format.
    pub const fn with_format(mut self, format: DumpFormat) -> Self {
        self.format = format;
        self
    }

    /// Set line numbers flag.
    pub const fn with_line_numbers(mut self, enabled: bool) -> Self {
        self.line_numbers = enabled;
        self
    }

    /// Set state IDs flag.
    pub const fn with_state_ids(mut self, enabled: bool) -> Self {
        self.state_ids = enabled;
        self
    }

    /// Set max length.
    pub const fn with_max_len(mut self, max_len: usize) -> Self {
        self.max_len = max_len;
        self
    }
}

// =============================================================================
// Profiling Support
// =============================================================================

/// Profile counters for regex operations.
#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct ProfileCounters {
    /// Number of pattern compilations.
    pub compilations: u64,
    /// Number of pattern executions.
    pub executions: u64,
    /// Number of successful matches.
    pub matches: u64,
    /// Number of failed matches.
    pub failures: u64,
    /// Number of backtracks (BT engine).
    pub backtracks: u64,
    /// Number of state transitions (NFA engine).
    pub transitions: u64,
    /// Total characters processed.
    pub chars_processed: u64,
    /// Total bytes allocated.
    pub bytes_allocated: u64,
}

impl ProfileCounters {
    /// Create new counters.
    pub const fn new() -> Self {
        Self {
            compilations: 0,
            executions: 0,
            matches: 0,
            failures: 0,
            backtracks: 0,
            transitions: 0,
            chars_processed: 0,
            bytes_allocated: 0,
        }
    }

    /// Reset all counters.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Increment compilation count.
    pub fn inc_compilations(&mut self) {
        self.compilations += 1;
    }

    /// Increment execution count.
    pub fn inc_executions(&mut self) {
        self.executions += 1;
    }

    /// Increment match count.
    pub fn inc_matches(&mut self) {
        self.matches += 1;
    }

    /// Increment failure count.
    pub fn inc_failures(&mut self) {
        self.failures += 1;
    }

    /// Add backtrack count.
    pub fn add_backtracks(&mut self, count: u64) {
        self.backtracks += count;
    }

    /// Add transition count.
    pub fn add_transitions(&mut self, count: u64) {
        self.transitions += count;
    }

    /// Add chars processed.
    pub fn add_chars_processed(&mut self, count: u64) {
        self.chars_processed += count;
    }

    /// Add bytes allocated.
    pub fn add_bytes_allocated(&mut self, count: u64) {
        self.bytes_allocated += count;
    }
}

/// Global profile counters.
static mut PROFILE_COUNTERS: ProfileCounters = ProfileCounters::new();

/// Get profile counters.
///
/// # Safety
/// Must be called from single-threaded context.
#[inline]
pub unsafe fn get_profile_counters() -> &'static ProfileCounters {
    // SAFETY: Caller guarantees single-threaded access
    std::ptr::addr_of!(PROFILE_COUNTERS).as_ref().unwrap()
}

/// Get mutable profile counters.
///
/// # Safety
/// Must be called from single-threaded context.
#[inline]
pub unsafe fn get_profile_counters_mut() -> &'static mut ProfileCounters {
    // SAFETY: Caller guarantees single-threaded access
    std::ptr::addr_of_mut!(PROFILE_COUNTERS).as_mut().unwrap()
}

/// Reset profile counters.
///
/// # Safety
/// Must be called from single-threaded context.
#[inline]
pub unsafe fn reset_profile_counters() {
    *std::ptr::addr_of_mut!(PROFILE_COUNTERS) = ProfileCounters::new();
}

// =============================================================================
// Execution Tracing
// =============================================================================

/// Trace event type.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraceEvent {
    /// Match started.
    MatchStart = 0,
    /// Match ended (success or failure).
    MatchEnd = 1,
    /// State transition.
    Transition = 2,
    /// Backtrack occurred.
    Backtrack = 3,
    /// Submatch opened.
    SubmatchOpen = 4,
    /// Submatch closed.
    SubmatchClose = 5,
    /// Character consumed.
    CharConsumed = 6,
    /// Lookahead started.
    LookaheadStart = 7,
    /// Lookahead ended.
    LookaheadEnd = 8,
}

/// Trace entry.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TraceEntry {
    /// Event type.
    pub event: TraceEvent,
    /// Position in input (byte offset).
    pub position: usize,
    /// State or opcode.
    pub state: u32,
    /// Additional data (context-dependent).
    pub data: u32,
}

impl TraceEntry {
    /// Create new trace entry.
    pub const fn new(event: TraceEvent, position: usize, state: u32, data: u32) -> Self {
        Self {
            event,
            position,
            state,
            data,
        }
    }
}

/// Trace buffer for execution history.
pub struct TraceBuffer {
    /// Entries.
    entries: Vec<TraceEntry>,
    /// Maximum entries.
    max_entries: usize,
    /// Whether buffer is enabled.
    enabled: bool,
}

impl TraceBuffer {
    /// Create new trace buffer.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries.min(1024)),
            max_entries,
            enabled: false,
        }
    }

    /// Enable tracing.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable tracing.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Clear buffer.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Add entry.
    pub fn push(&mut self, entry: TraceEntry) {
        if self.enabled && self.entries.len() < self.max_entries {
            self.entries.push(entry);
        }
    }

    /// Get entries.
    pub fn entries(&self) -> &[TraceEntry] {
        &self.entries
    }

    /// Get entry count.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for TraceBuffer {
    fn default() -> Self {
        Self::new(10000)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get debug level.
///
/// # Safety
/// Must be called from single-threaded context.
#[no_mangle]
pub unsafe extern "C" fn rs_regexp_get_debug_level() -> c_int {
    get_debug_level().into()
}

/// Set debug level.
///
/// # Safety
/// Must be called from single-threaded context.
#[no_mangle]
pub unsafe extern "C" fn rs_regexp_set_debug_level(level: c_int) {
    set_debug_level(DebugLevel::from(level));
}

/// Check if debug is enabled.
///
/// # Safety
/// Must be called from single-threaded context.
#[no_mangle]
pub unsafe extern "C" fn rs_regexp_debug_enabled() -> c_int {
    c_int::from(is_debug_enabled())
}

/// Reset profile counters.
///
/// # Safety
/// Must be called from single-threaded context.
#[no_mangle]
pub unsafe extern "C" fn rs_regexp_reset_profile() {
    reset_profile_counters();
}

/// Get compilation count.
///
/// # Safety
/// Must be called from single-threaded context.
#[no_mangle]
pub unsafe extern "C" fn rs_regexp_get_compilations() -> u64 {
    get_profile_counters().compilations
}

/// Get execution count.
///
/// # Safety
/// Must be called from single-threaded context.
#[no_mangle]
pub unsafe extern "C" fn rs_regexp_get_executions() -> u64 {
    get_profile_counters().executions
}

/// Get match count.
///
/// # Safety
/// Must be called from single-threaded context.
#[no_mangle]
pub unsafe extern "C" fn rs_regexp_get_matches() -> u64 {
    get_profile_counters().matches
}

/// Get backtrack count.
///
/// # Safety
/// Must be called from single-threaded context.
#[no_mangle]
pub unsafe extern "C" fn rs_regexp_get_backtracks() -> u64 {
    get_profile_counters().backtracks
}

/// Get transition count.
///
/// # Safety
/// Must be called from single-threaded context.
#[no_mangle]
pub unsafe extern "C" fn rs_regexp_get_transitions() -> u64 {
    get_profile_counters().transitions
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_level_conversion() {
        assert_eq!(DebugLevel::from(0), DebugLevel::Off);
        assert_eq!(DebugLevel::from(1), DebugLevel::Basic);
        assert_eq!(DebugLevel::from(2), DebugLevel::Verbose);
        assert_eq!(DebugLevel::from(3), DebugLevel::Trace);
        assert_eq!(DebugLevel::from(99), DebugLevel::Trace); // >= 3
    }

    #[test]
    fn test_debug_level_to_int() {
        assert_eq!(c_int::from(DebugLevel::Off), 0);
        assert_eq!(c_int::from(DebugLevel::Basic), 1);
        assert_eq!(c_int::from(DebugLevel::Verbose), 2);
        assert_eq!(c_int::from(DebugLevel::Trace), 3);
    }

    #[test]
    fn test_dump_options() {
        let opts = DumpOptions::new()
            .with_format(DumpFormat::Hex)
            .with_line_numbers(false)
            .with_max_len(1000);

        assert_eq!(opts.format, DumpFormat::Hex);
        assert!(!opts.line_numbers);
        assert!(opts.state_ids); // default
        assert_eq!(opts.max_len, 1000);
    }

    #[test]
    fn test_profile_counters() {
        let mut counters = ProfileCounters::new();

        assert_eq!(counters.compilations, 0);
        assert_eq!(counters.executions, 0);

        counters.inc_compilations();
        counters.inc_executions();
        counters.inc_matches();
        counters.add_backtracks(10);
        counters.add_transitions(100);
        counters.add_chars_processed(50);

        assert_eq!(counters.compilations, 1);
        assert_eq!(counters.executions, 1);
        assert_eq!(counters.matches, 1);
        assert_eq!(counters.backtracks, 10);
        assert_eq!(counters.transitions, 100);
        assert_eq!(counters.chars_processed, 50);

        counters.reset();
        assert_eq!(counters.compilations, 0);
        assert_eq!(counters.backtracks, 0);
    }

    #[test]
    fn test_trace_entry() {
        let entry = TraceEntry::new(TraceEvent::MatchStart, 42, 1, 0);

        assert_eq!(entry.event, TraceEvent::MatchStart);
        assert_eq!(entry.position, 42);
        assert_eq!(entry.state, 1);
        assert_eq!(entry.data, 0);
    }

    #[test]
    fn test_trace_buffer() {
        let mut buffer = TraceBuffer::new(100);

        assert!(!buffer.is_enabled());
        assert!(buffer.is_empty());

        buffer.enable();
        assert!(buffer.is_enabled());

        buffer.push(TraceEntry::new(TraceEvent::MatchStart, 0, 0, 0));
        buffer.push(TraceEntry::new(TraceEvent::CharConsumed, 1, 0, b'a' as u32));
        buffer.push(TraceEntry::new(TraceEvent::MatchEnd, 2, 0, 1));

        assert_eq!(buffer.len(), 3);
        assert!(!buffer.is_empty());

        let entries = buffer.entries();
        assert_eq!(entries[0].event, TraceEvent::MatchStart);
        assert_eq!(entries[1].event, TraceEvent::CharConsumed);
        assert_eq!(entries[2].event, TraceEvent::MatchEnd);

        buffer.clear();
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_trace_buffer_disabled() {
        let mut buffer = TraceBuffer::new(100);

        // Disabled by default
        buffer.push(TraceEntry::new(TraceEvent::MatchStart, 0, 0, 0));
        assert!(buffer.is_empty()); // Should not add when disabled
    }

    #[test]
    fn test_trace_buffer_max_entries() {
        let mut buffer = TraceBuffer::new(2);
        buffer.enable();

        buffer.push(TraceEntry::new(TraceEvent::MatchStart, 0, 0, 0));
        buffer.push(TraceEntry::new(TraceEvent::CharConsumed, 1, 0, 0));
        buffer.push(TraceEntry::new(TraceEvent::MatchEnd, 2, 0, 0)); // Should be ignored

        assert_eq!(buffer.len(), 2);
    }

    #[test]
    fn test_dump_format_default() {
        let format = DumpFormat::default();
        assert_eq!(format, DumpFormat::Text);
    }
}
