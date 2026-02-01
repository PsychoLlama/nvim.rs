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
// NFA Debug Dump Functions
// =============================================================================
//
// Debug code uses libc::fprintf with C strings for compatibility with C debug output.
// Allow manual C string construction for consistency with existing patterns.

use crate::nfa_states::{is_nfa_nl_variant, nfa_opcode_name, NFA_ADD_NL};
use std::ffi::c_void;

/// Opaque handle to nfa_state_T (mutable).
pub type NfaStateHandle = *mut c_void;

/// Opaque handle to nfa_regprog_T (mutable).
pub type NfaRegprogHandle = *mut c_void;

/// Log file path for NFA regexp dumps.
const NFA_REGEXP_DUMP_LOG: &[u8] = b"nfa_regexp_dump.log\0";

// FFI declarations for C accessor functions
// Note: Using *const c_void for consistency with exec_state.rs declarations
extern "C" {
    fn nvim_nfa_state_get_c(state: *const c_void) -> c_int;
    fn nvim_nfa_state_get_out(state: *const c_void) -> *mut c_void;
    fn nvim_nfa_state_get_out1(state: *const c_void) -> *mut c_void;
    fn nvim_nfa_state_get_id(state: *const c_void) -> c_int;
    fn nvim_nfa_state_get_val(state: *const c_void) -> c_int;

    fn nvim_nfa_regprog_get_start(prog: *const c_void) -> *mut c_void;
    fn nvim_nfa_regprog_get_reganch(prog: *const c_void) -> c_int;
    fn nvim_nfa_regprog_get_regstart(prog: *const c_void) -> c_int;
    fn nvim_nfa_regprog_get_match_text(prog: *const c_void) -> *const u8;
}

/// Growing array for indentation tracking.
struct IndentBuffer {
    data: Vec<u8>,
}

impl IndentBuffer {
    fn new() -> Self {
        Self { data: vec![0] }
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn push_str(&mut self, s: &[u8]) {
        // Remove trailing NUL
        if self.data.last() == Some(&0) {
            self.data.pop();
        }
        self.data.extend_from_slice(s);
        self.data.push(0);
    }

    fn shrink(&mut self, by: usize) {
        if self.data.len() > by {
            self.data.truncate(self.data.len() - by);
            if self.data.last() != Some(&0) {
                self.data.push(0);
            }
        }
    }

    fn get_for_branch(&self) -> Vec<u8> {
        if self.len() >= 4 {
            // Replace last "  " or "| " with "+-"
            let mut result = self.data.clone();
            let last = result.len() - 3; // Before the NUL and last two chars
            if last >= 2 {
                result[last] = b'+';
                result[last + 1] = b'-';
            }
            result
        } else {
            self.data.clone()
        }
    }
}

/// Print the NFA state tree recursively.
///
/// This is the Rust implementation of `nfa_print_state2` from C.
///
/// # Safety
/// - `debugf` must be a valid FILE pointer
/// - `state` must be a valid NFA state pointer or null
#[allow(clippy::manual_c_str_literals)]
unsafe fn nfa_print_state2_impl(
    debugf: *mut libc::FILE,
    state: NfaStateHandle,
    indent: &mut IndentBuffer,
) {
    if state.is_null() || debugf.is_null() {
        return;
    }

    let id = nvim_nfa_state_get_id(state);
    let c = nvim_nfa_state_get_c(state);
    let val = nvim_nfa_state_get_val(state);
    let out = nvim_nfa_state_get_out(state);
    let out1 = nvim_nfa_state_get_out1(state);

    // Print state ID
    libc::fprintf(debugf, b"(%2d)\0".as_ptr().cast(), id.abs());

    // Output indent with branch marker
    let branch_indent = indent.get_for_branch();
    libc::fprintf(debugf, b" %s\0".as_ptr().cast(), branch_indent.as_ptr());

    // Get opcode name
    let name = nfa_opcode_name(c);
    let nl_suffix = if is_nfa_nl_variant(c) {
        " + NEWLINE "
    } else {
        ""
    };

    // Print state info
    libc::fprintf(
        debugf,
        b"%s%s (%d) (id=%d) val=%d\n\0".as_ptr().cast(),
        name.as_ptr(),
        nl_suffix.as_ptr(),
        c,
        id.abs(),
        val,
    );

    // If already visited (negative id), stop
    if id < 0 {
        return;
    }

    // Mark as visited by negating the ID (done in C side)
    // Note: We can't modify the state here since we only have accessors
    // This function should only be called from the C wrapper that handles state marking

    // Grow indent for state->out
    if !out1.is_null() {
        indent.push_str(b"| ");
    } else {
        indent.push_str(b"  ");
    }

    nfa_print_state2_impl(debugf, out, indent);

    // Replace last part of indent for state->out1
    indent.shrink(3);
    indent.push_str(b"  ");

    nfa_print_state2_impl(debugf, out1, indent);

    // Shrink indent back
    indent.shrink(3);
}

/// Print the NFA starting with a root node "state".
///
/// This is the Rust implementation of `nfa_print_state` from C.
///
/// # Safety
/// - `debugf` must be a valid FILE pointer
/// - `state` must be a valid NFA state pointer or null
pub unsafe fn nfa_print_state_impl(debugf: *mut libc::FILE, state: NfaStateHandle) {
    let mut indent = IndentBuffer::new();
    nfa_print_state2_impl(debugf, state, &mut indent);
}

/// Print the NFA state machine.
///
/// This is the Rust implementation of `nfa_dump` from C.
///
/// # Safety
/// - `prog` must be a valid nfa_regprog_T pointer
#[allow(clippy::manual_c_str_literals)]
pub unsafe fn nfa_dump_impl(prog: NfaRegprogHandle) {
    if prog.is_null() {
        return;
    }

    let debugf = libc::fopen(NFA_REGEXP_DUMP_LOG.as_ptr().cast(), b"a\0".as_ptr().cast());
    if debugf.is_null() {
        return;
    }

    let start = nvim_nfa_regprog_get_start(prog);
    nfa_print_state_impl(debugf, start);

    let reganch = nvim_nfa_regprog_get_reganch(prog);
    if reganch != 0 {
        libc::fprintf(debugf, b"reganch: %d\n\0".as_ptr().cast(), reganch);
    }

    let regstart = nvim_nfa_regprog_get_regstart(prog);
    if regstart != 0 {
        libc::fprintf(
            debugf,
            b"regstart: %c (decimal: %d)\n\0".as_ptr().cast(),
            regstart,
            regstart,
        );
    }

    let match_text = nvim_nfa_regprog_get_match_text(prog);
    if !match_text.is_null() {
        libc::fprintf(
            debugf,
            b"match_text: \"%s\"\n\0".as_ptr().cast(),
            match_text,
        );
    }

    libc::fclose(debugf);
}

/// Print the postfix notation of a regexp.
///
/// This is the Rust implementation of `nfa_postfix_dump` from C.
/// Note: This function requires access to internal state (post_start, post_ptr)
/// that is managed by C, so it takes those as parameters.
///
/// # Safety
/// - `expr` must be a valid null-terminated string
/// - `post_start` and `post_end` must be valid pointers to postfix arrays
#[allow(clippy::manual_c_str_literals)]
#[allow(clippy::manual_range_contains)]
pub unsafe fn nfa_postfix_dump_impl(
    expr: *const u8,
    retval: c_int,
    post_start: *const c_int,
    post_end: *const c_int,
) {
    if expr.is_null() || post_start.is_null() {
        return;
    }

    let f = libc::fopen(NFA_REGEXP_DUMP_LOG.as_ptr().cast(), b"a\0".as_ptr().cast());
    if f.is_null() {
        return;
    }

    // Write header
    libc::fprintf(f, b"\n-------------------------\n\0".as_ptr().cast());

    // Print result status
    if retval == 0 {
        // FAIL
        libc::fprintf(f, b">>> NFA engine failed... \n\0".as_ptr().cast());
    } else {
        // OK
        libc::fprintf(f, b">>> NFA engine succeeded !\n\0".as_ptr().cast());
    }

    // Print the regexp
    libc::fprintf(
        f,
        b"Regexp: \"%s\"\nPostfix notation (char): \"\0"
            .as_ptr()
            .cast(),
        expr,
    );

    // Print postfix as opcode names
    let mut p = post_start;
    while p < post_end && *p != 0 {
        let c = *p;
        let name = nfa_opcode_name(c);

        // For NL variants, append suffix
        if is_nfa_nl_variant(c) {
            let base_name = nfa_opcode_name(c - NFA_ADD_NL);
            libc::fprintf(f, b"%s + NEWLINE , \0".as_ptr().cast(), base_name.as_ptr());
        } else if c >= 0 && name == "CHAR" {
            // Character literal
            if c >= 32 && c < 127 {
                libc::fprintf(f, b"CHAR(%c), \0".as_ptr().cast(), c);
            } else {
                libc::fprintf(f, b"CHAR(%d), \0".as_ptr().cast(), c);
            }
        } else {
            libc::fprintf(f, b"%s, \0".as_ptr().cast(), name.as_ptr());
        }

        p = p.add(1);
    }

    // Print postfix as integers
    libc::fprintf(f, b"\"\nPostfix notation (int): \0".as_ptr().cast());
    p = post_start;
    while p < post_end && *p != 0 {
        libc::fprintf(f, b"%d \0".as_ptr().cast(), *p);
        p = p.add(1);
    }
    libc::fprintf(f, b"\n\n\0".as_ptr().cast());

    libc::fclose(f);
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
// NFA Debug FFI Exports
// =============================================================================

/// Print the NFA state machine to the debug log file.
///
/// # Safety
/// - `prog` must be a valid nfa_regprog_T pointer or null
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_dump(prog: NfaRegprogHandle) {
    nfa_dump_impl(prog);
}

/// Print an NFA state tree to a file.
///
/// # Safety
/// - `debugf` must be a valid FILE pointer
/// - `state` must be a valid nfa_state_T pointer or null
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_print_state(debugf: *mut libc::FILE, state: NfaStateHandle) {
    nfa_print_state_impl(debugf, state);
}

/// Print the postfix notation of the current regexp.
///
/// # Safety
/// - `expr` must be a valid null-terminated string
/// - `post_start` and `post_end` must be valid pointers
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_postfix_dump(
    expr: *const u8,
    retval: c_int,
    post_start: *const c_int,
    post_end: *const c_int,
) {
    nfa_postfix_dump_impl(expr, retval, post_start, post_end);
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

    // =========================================================================
    // IndentBuffer Tests
    // =========================================================================

    #[test]
    fn test_indent_buffer_new() {
        let buf = IndentBuffer::new();
        assert_eq!(buf.len(), 1); // Just the NUL terminator
        assert_eq!(buf.data, vec![0]);
    }

    #[test]
    fn test_indent_buffer_push_str() {
        let mut buf = IndentBuffer::new();
        buf.push_str(b"| ");
        // Should be "| " followed by NUL
        assert_eq!(buf.data, vec![b'|', b' ', 0]);
        assert_eq!(buf.len(), 3);

        buf.push_str(b"  ");
        // Should be "| " + "  " followed by NUL
        assert_eq!(buf.data, vec![b'|', b' ', b' ', b' ', 0]);
    }

    #[test]
    fn test_indent_buffer_shrink() {
        let mut buf = IndentBuffer::new();
        buf.push_str(b"| ");
        buf.push_str(b"  ");
        // "| " + "  " + NUL = 5 bytes
        assert_eq!(buf.len(), 5);

        buf.shrink(3);
        // After shrinking by 3, should have "| " + NUL = 3 bytes
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn test_indent_buffer_get_for_branch() {
        let mut buf = IndentBuffer::new();
        buf.push_str(b"| ");
        buf.push_str(b"  ");
        // "| " + "  " + NUL

        let branch = buf.get_for_branch();
        // Last "  " should become "+-"
        assert!(branch.contains(&b'+'));
        assert!(branch.contains(&b'-'));
    }
}
