//! Decoration caching
//!
//! This module provides caching infrastructure for decoration
//! providers to optimize performance.

use std::ffi::c_int;

// =============================================================================
// Cache Entry State
// =============================================================================

/// State of a cache entry.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CacheEntryState {
    /// Entry is not valid
    #[default]
    Invalid = 0,
    /// Entry is valid
    Valid = 1,
    /// Entry is stale (needs refresh)
    Stale = 2,
    /// Entry is being refreshed
    Refreshing = 3,
    /// Entry is expired
    Expired = 4,
}

impl CacheEntryState {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Valid,
            2 => Self::Stale,
            3 => Self::Refreshing,
            // 0, 4, and unrecognized values
            _ => Self::Invalid,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if entry is usable.
    #[must_use]
    pub const fn is_usable(self) -> bool {
        matches!(self, Self::Valid | Self::Stale)
    }

    /// Check if entry needs refresh.
    #[must_use]
    pub const fn needs_refresh(self) -> bool {
        matches!(self, Self::Stale | Self::Invalid | Self::Expired)
    }
}

/// FFI: Check if cache entry is usable.
#[no_mangle]
pub extern "C" fn rs_cache_entry_state_is_usable(state: c_int) -> c_int {
    c_int::from(CacheEntryState::from_c_int(state).is_usable())
}

/// FFI: Check if cache entry needs refresh.
#[no_mangle]
pub extern "C" fn rs_cache_entry_state_needs_refresh(state: c_int) -> c_int {
    c_int::from(CacheEntryState::from_c_int(state).needs_refresh())
}

// =============================================================================
// Cache Policy
// =============================================================================

/// Caching policy for decorations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CachePolicy {
    /// No caching
    #[default]
    None = 0,
    /// Cache per line
    PerLine = 1,
    /// Cache per window
    PerWindow = 2,
    /// Cache per buffer
    PerBuffer = 3,
    /// Global cache
    Global = 4,
}

impl CachePolicy {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::PerLine,
            2 => Self::PerWindow,
            3 => Self::PerBuffer,
            4 => Self::Global,
            _ => Self::None,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if policy is hierarchical (can invalidate children).
    #[must_use]
    pub const fn is_hierarchical(self) -> bool {
        matches!(self, Self::PerBuffer | Self::Global)
    }
}

// =============================================================================
// Cache Entry
// =============================================================================

/// A cache entry for decorations.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CacheEntry {
    /// Entry state
    pub state: c_int,
    /// Provider ID that created this entry
    pub provider_id: c_int,
    /// Buffer handle
    pub buf_id: c_int,
    /// Window handle (for per-window cache)
    pub win_id: c_int,
    /// Start line (for per-line cache)
    pub start_line: c_int,
    /// End line (for per-line cache)
    pub end_line: c_int,
    /// Creation timestamp (microseconds)
    pub created_us: i64,
    /// Last access timestamp (microseconds)
    pub accessed_us: i64,
    /// Number of decorations in this entry
    pub decor_count: c_int,
    /// Entry version (for invalidation)
    pub version: u64,
}

impl CacheEntry {
    /// Create new cache entry.
    #[must_use]
    pub const fn new(provider_id: c_int, buf_id: c_int) -> Self {
        Self {
            state: CacheEntryState::Invalid as c_int,
            provider_id,
            buf_id,
            win_id: 0,
            start_line: 0,
            end_line: 0,
            created_us: 0,
            accessed_us: 0,
            decor_count: 0,
            version: 0,
        }
    }

    /// Create entry for line range.
    #[must_use]
    pub const fn for_lines(provider_id: c_int, buf_id: c_int, start: c_int, end: c_int) -> Self {
        Self {
            state: CacheEntryState::Invalid as c_int,
            provider_id,
            buf_id,
            win_id: 0,
            start_line: start,
            end_line: end,
            created_us: 0,
            accessed_us: 0,
            decor_count: 0,
            version: 0,
        }
    }

    /// Get entry state.
    #[must_use]
    pub const fn get_state(&self) -> CacheEntryState {
        CacheEntryState::from_c_int(self.state)
    }

    /// Set entry state.
    pub fn set_state(&mut self, state: CacheEntryState) {
        self.state = state as c_int;
    }

    /// Mark entry as valid.
    pub fn validate(&mut self, version: u64, created_us: i64) {
        self.state = CacheEntryState::Valid as c_int;
        self.version = version;
        self.created_us = created_us;
        self.accessed_us = created_us;
    }

    /// Mark entry as stale.
    pub fn mark_stale(&mut self) {
        if self.get_state() == CacheEntryState::Valid {
            self.state = CacheEntryState::Stale as c_int;
        }
    }

    /// Invalidate entry.
    pub fn invalidate(&mut self) {
        self.state = CacheEntryState::Invalid as c_int;
    }

    /// Record access.
    pub fn record_access(&mut self, timestamp_us: i64) {
        self.accessed_us = timestamp_us;
    }

    /// Check if entry covers line.
    #[must_use]
    pub const fn covers_line(&self, line: c_int) -> bool {
        line >= self.start_line && line <= self.end_line
    }

    /// Check if entry is for buffer.
    #[must_use]
    pub const fn is_for_buffer(&self, buf_id: c_int) -> bool {
        self.buf_id == buf_id
    }
}

/// FFI: Create cache entry.
#[no_mangle]
pub extern "C" fn rs_cache_entry_new(provider_id: c_int, buf_id: c_int) -> CacheEntry {
    CacheEntry::new(provider_id, buf_id)
}

/// FFI: Create cache entry for lines.
#[no_mangle]
pub extern "C" fn rs_cache_entry_for_lines(
    provider_id: c_int,
    buf_id: c_int,
    start: c_int,
    end: c_int,
) -> CacheEntry {
    CacheEntry::for_lines(provider_id, buf_id, start, end)
}

/// FFI: Validate cache entry.
///
/// # Safety
/// `entry` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_cache_entry_validate(
    entry: *mut CacheEntry,
    version: u64,
    created_us: i64,
) {
    if !entry.is_null() {
        (*entry).validate(version, created_us);
    }
}

/// FFI: Mark cache entry stale.
///
/// # Safety
/// `entry` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_cache_entry_mark_stale(entry: *mut CacheEntry) {
    if !entry.is_null() {
        (*entry).mark_stale();
    }
}

/// FFI: Invalidate cache entry.
///
/// # Safety
/// `entry` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_cache_entry_invalidate(entry: *mut CacheEntry) {
    if !entry.is_null() {
        (*entry).invalidate();
    }
}

// =============================================================================
// Cache Configuration
// =============================================================================

/// Configuration for decoration cache.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CacheConfig {
    /// Caching policy
    pub policy: c_int,
    /// Maximum entries in cache
    pub max_entries: c_int,
    /// TTL in milliseconds (0 = infinite)
    pub ttl_ms: c_int,
    /// Auto-invalidate on buffer change
    pub invalidate_on_change: bool,
    /// Auto-invalidate on window resize
    pub invalidate_on_resize: bool,
    /// Prefetch adjacent lines
    pub prefetch: bool,
    /// Number of lines to prefetch
    pub prefetch_lines: c_int,
}

impl CacheConfig {
    /// Create config with no caching.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            policy: CachePolicy::None as c_int,
            max_entries: 0,
            ttl_ms: 0,
            invalidate_on_change: true,
            invalidate_on_resize: false,
            prefetch: false,
            prefetch_lines: 0,
        }
    }

    /// Create config for per-line caching.
    #[must_use]
    pub const fn per_line(max_entries: c_int) -> Self {
        Self {
            policy: CachePolicy::PerLine as c_int,
            max_entries,
            ttl_ms: 0,
            invalidate_on_change: true,
            invalidate_on_resize: false,
            prefetch: true,
            prefetch_lines: 50,
        }
    }

    /// Create config for per-window caching.
    #[must_use]
    pub const fn per_window(max_entries: c_int) -> Self {
        Self {
            policy: CachePolicy::PerWindow as c_int,
            max_entries,
            ttl_ms: 0,
            invalidate_on_change: true,
            invalidate_on_resize: true,
            prefetch: false,
            prefetch_lines: 0,
        }
    }

    /// Create config for per-buffer caching.
    #[must_use]
    pub const fn per_buffer(max_entries: c_int) -> Self {
        Self {
            policy: CachePolicy::PerBuffer as c_int,
            max_entries,
            ttl_ms: 0,
            invalidate_on_change: true,
            invalidate_on_resize: false,
            prefetch: false,
            prefetch_lines: 0,
        }
    }

    /// Get caching policy.
    #[must_use]
    pub const fn get_policy(&self) -> CachePolicy {
        CachePolicy::from_c_int(self.policy)
    }
}

/// FFI: Create no-cache config.
#[no_mangle]
pub extern "C" fn rs_cache_config_none() -> CacheConfig {
    CacheConfig::none()
}

/// FFI: Create per-line cache config.
#[no_mangle]
pub extern "C" fn rs_cache_config_per_line(max_entries: c_int) -> CacheConfig {
    CacheConfig::per_line(max_entries)
}

/// FFI: Create per-window cache config.
#[no_mangle]
pub extern "C" fn rs_cache_config_per_window(max_entries: c_int) -> CacheConfig {
    CacheConfig::per_window(max_entries)
}

/// FFI: Create per-buffer cache config.
#[no_mangle]
pub extern "C" fn rs_cache_config_per_buffer(max_entries: c_int) -> CacheConfig {
    CacheConfig::per_buffer(max_entries)
}

// =============================================================================
// Cache Statistics
// =============================================================================

/// Statistics for decoration cache.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CacheStats {
    /// Total cache lookups
    pub lookups: u64,
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Stale hits (usable but stale)
    pub stale_hits: u64,
    /// Entries added
    pub entries_added: u64,
    /// Entries evicted
    pub entries_evicted: u64,
    /// Entries invalidated
    pub entries_invalidated: u64,
    /// Current entry count
    pub current_entries: c_int,
    /// Memory used (bytes)
    pub memory_bytes: u64,
}

impl CacheStats {
    /// Create new stats.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            lookups: 0,
            hits: 0,
            misses: 0,
            stale_hits: 0,
            entries_added: 0,
            entries_evicted: 0,
            entries_invalidated: 0,
            current_entries: 0,
            memory_bytes: 0,
        }
    }

    /// Record a cache hit.
    pub fn record_hit(&mut self, was_stale: bool) {
        self.lookups += 1;
        self.hits += 1;
        if was_stale {
            self.stale_hits += 1;
        }
    }

    /// Record a cache miss.
    pub fn record_miss(&mut self) {
        self.lookups += 1;
        self.misses += 1;
    }

    /// Record entry added.
    pub fn record_add(&mut self, memory_bytes: u64) {
        self.entries_added += 1;
        self.current_entries += 1;
        self.memory_bytes += memory_bytes;
    }

    /// Record entry evicted.
    pub fn record_evict(&mut self, memory_bytes: u64) {
        self.entries_evicted += 1;
        if self.current_entries > 0 {
            self.current_entries -= 1;
        }
        if self.memory_bytes >= memory_bytes {
            self.memory_bytes -= memory_bytes;
        }
    }

    /// Record entry invalidated.
    pub fn record_invalidate(&mut self) {
        self.entries_invalidated += 1;
    }

    /// Get hit rate (0.0 to 1.0).
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        if self.lookups == 0 {
            0.0
        } else {
            self.hits as f64 / self.lookups as f64
        }
    }

    /// Reset statistics.
    pub fn reset(&mut self) {
        self.lookups = 0;
        self.hits = 0;
        self.misses = 0;
        self.stale_hits = 0;
        self.entries_added = 0;
        self.entries_evicted = 0;
        self.entries_invalidated = 0;
        // Don't reset current_entries or memory_bytes
    }
}

/// FFI: Create cache stats.
#[no_mangle]
pub extern "C" fn rs_cache_stats_new() -> CacheStats {
    CacheStats::new()
}

/// FFI: Record cache hit.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_cache_stats_record_hit(stats: *mut CacheStats, was_stale: c_int) {
    if !stats.is_null() {
        (*stats).record_hit(was_stale != 0);
    }
}

/// FFI: Record cache miss.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_cache_stats_record_miss(stats: *mut CacheStats) {
    if !stats.is_null() {
        (*stats).record_miss();
    }
}

/// FFI: Record entry added.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_cache_stats_record_add(stats: *mut CacheStats, memory_bytes: u64) {
    if !stats.is_null() {
        (*stats).record_add(memory_bytes);
    }
}

/// FFI: Record entry evicted.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_cache_stats_record_evict(stats: *mut CacheStats, memory_bytes: u64) {
    if !stats.is_null() {
        (*stats).record_evict(memory_bytes);
    }
}

/// FFI: Get hit rate.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_cache_stats_hit_rate(stats: *const CacheStats) -> f64 {
    if stats.is_null() {
        return 0.0;
    }
    (*stats).hit_rate()
}

/// FFI: Reset cache stats.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_cache_stats_reset(stats: *mut CacheStats) {
    if !stats.is_null() {
        (*stats).reset();
    }
}

// =============================================================================
// Invalidation Reason
// =============================================================================

/// Reason for cache invalidation.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InvalidationReason {
    /// Unknown reason
    #[default]
    Unknown = 0,
    /// Buffer content changed
    BufferChange = 1,
    /// Buffer was unloaded
    BufferUnload = 2,
    /// Window was closed
    WindowClose = 3,
    /// Window was resized
    WindowResize = 4,
    /// Provider was removed
    ProviderRemoved = 5,
    /// Manual invalidation
    Manual = 6,
    /// TTL expired
    Expired = 7,
    /// Evicted due to cache full
    Evicted = 8,
}

impl InvalidationReason {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::BufferChange,
            2 => Self::BufferUnload,
            3 => Self::WindowClose,
            4 => Self::WindowResize,
            5 => Self::ProviderRemoved,
            6 => Self::Manual,
            7 => Self::Expired,
            8 => Self::Evicted,
            _ => Self::Unknown,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if invalidation is due to buffer change.
    #[must_use]
    pub const fn is_buffer_related(self) -> bool {
        matches!(self, Self::BufferChange | Self::BufferUnload)
    }

    /// Check if invalidation is due to window change.
    #[must_use]
    pub const fn is_window_related(self) -> bool {
        matches!(self, Self::WindowClose | Self::WindowResize)
    }
}

// =============================================================================
// Invalidation Request
// =============================================================================

/// Request to invalidate cache entries.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct InvalidationRequest {
    /// Reason for invalidation
    pub reason: c_int,
    /// Provider ID (0 = all providers)
    pub provider_id: c_int,
    /// Buffer ID (0 = all buffers)
    pub buf_id: c_int,
    /// Window ID (0 = all windows)
    pub win_id: c_int,
    /// Start line (-1 = from beginning)
    pub start_line: c_int,
    /// End line (-1 = to end)
    pub end_line: c_int,
}

impl InvalidationRequest {
    /// Create request to invalidate all entries.
    #[must_use]
    pub const fn all(reason: InvalidationReason) -> Self {
        Self {
            reason: reason as c_int,
            provider_id: 0,
            buf_id: 0,
            win_id: 0,
            start_line: -1,
            end_line: -1,
        }
    }

    /// Create request for buffer.
    #[must_use]
    pub const fn for_buffer(buf_id: c_int, reason: InvalidationReason) -> Self {
        Self {
            reason: reason as c_int,
            provider_id: 0,
            buf_id,
            win_id: 0,
            start_line: -1,
            end_line: -1,
        }
    }

    /// Create request for line range.
    #[must_use]
    pub const fn for_lines(
        buf_id: c_int,
        start: c_int,
        end: c_int,
        reason: InvalidationReason,
    ) -> Self {
        Self {
            reason: reason as c_int,
            provider_id: 0,
            buf_id,
            win_id: 0,
            start_line: start,
            end_line: end,
        }
    }

    /// Create request for provider.
    #[must_use]
    pub const fn for_provider(provider_id: c_int, reason: InvalidationReason) -> Self {
        Self {
            reason: reason as c_int,
            provider_id,
            buf_id: 0,
            win_id: 0,
            start_line: -1,
            end_line: -1,
        }
    }

    /// Get reason.
    #[must_use]
    pub const fn get_reason(&self) -> InvalidationReason {
        InvalidationReason::from_c_int(self.reason)
    }

    /// Check if request matches entry.
    #[must_use]
    pub const fn matches_entry(&self, entry: &CacheEntry) -> bool {
        // Check provider filter
        if self.provider_id != 0 && self.provider_id != entry.provider_id {
            return false;
        }

        // Check buffer filter
        if self.buf_id != 0 && self.buf_id != entry.buf_id {
            return false;
        }

        // Check window filter
        if self.win_id != 0 && self.win_id != entry.win_id {
            return false;
        }

        // Check line range filter
        if self.start_line >= 0 && self.end_line >= 0 {
            // Request has line range - check for overlap
            if entry.end_line < self.start_line || entry.start_line > self.end_line {
                return false;
            }
        }

        true
    }
}

/// FFI: Create invalidation request for all.
#[no_mangle]
pub extern "C" fn rs_invalidation_request_all(reason: c_int) -> InvalidationRequest {
    InvalidationRequest::all(InvalidationReason::from_c_int(reason))
}

/// FFI: Create invalidation request for buffer.
#[no_mangle]
pub extern "C" fn rs_invalidation_request_for_buffer(
    buf_id: c_int,
    reason: c_int,
) -> InvalidationRequest {
    InvalidationRequest::for_buffer(buf_id, InvalidationReason::from_c_int(reason))
}

/// FFI: Create invalidation request for lines.
#[no_mangle]
pub extern "C" fn rs_invalidation_request_for_lines(
    buf_id: c_int,
    start: c_int,
    end: c_int,
    reason: c_int,
) -> InvalidationRequest {
    InvalidationRequest::for_lines(buf_id, start, end, InvalidationReason::from_c_int(reason))
}

/// FFI: Check if request matches entry.
#[no_mangle]
pub extern "C" fn rs_invalidation_matches_entry(
    request: InvalidationRequest,
    entry: CacheEntry,
) -> c_int {
    c_int::from(request.matches_entry(&entry))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_state() {
        assert!(!CacheEntryState::Invalid.is_usable());
        assert!(CacheEntryState::Valid.is_usable());
        assert!(CacheEntryState::Stale.is_usable());

        assert!(CacheEntryState::Invalid.needs_refresh());
        assert!(CacheEntryState::Stale.needs_refresh());
        assert!(!CacheEntryState::Valid.needs_refresh());
    }

    #[test]
    fn test_cache_policy() {
        assert!(!CachePolicy::PerLine.is_hierarchical());
        assert!(CachePolicy::PerBuffer.is_hierarchical());
        assert!(CachePolicy::Global.is_hierarchical());
    }

    #[test]
    fn test_cache_entry() {
        let mut entry = CacheEntry::new(1, 100);
        assert_eq!(entry.provider_id, 1);
        assert_eq!(entry.buf_id, 100);
        assert_eq!(entry.get_state(), CacheEntryState::Invalid);

        entry.validate(42, 1000);
        assert_eq!(entry.get_state(), CacheEntryState::Valid);
        assert_eq!(entry.version, 42);

        entry.mark_stale();
        assert_eq!(entry.get_state(), CacheEntryState::Stale);

        entry.invalidate();
        assert_eq!(entry.get_state(), CacheEntryState::Invalid);
    }

    #[test]
    fn test_cache_entry_lines() {
        let entry = CacheEntry::for_lines(1, 100, 10, 20);
        assert!(entry.covers_line(10));
        assert!(entry.covers_line(15));
        assert!(entry.covers_line(20));
        assert!(!entry.covers_line(5));
        assert!(!entry.covers_line(25));
    }

    #[test]
    fn test_cache_config() {
        let none = CacheConfig::none();
        assert_eq!(none.get_policy(), CachePolicy::None);

        let per_line = CacheConfig::per_line(100);
        assert_eq!(per_line.get_policy(), CachePolicy::PerLine);
        assert_eq!(per_line.max_entries, 100);
        assert!(per_line.prefetch);

        let per_window = CacheConfig::per_window(50);
        assert_eq!(per_window.get_policy(), CachePolicy::PerWindow);
        assert!(per_window.invalidate_on_resize);
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::new();
        assert_eq!(stats.lookups, 0);
        assert!((stats.hit_rate() - 0.0).abs() < f64::EPSILON);

        stats.record_hit(false);
        stats.record_hit(true);
        stats.record_miss();
        assert_eq!(stats.lookups, 3);
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.stale_hits, 1);
        assert!((stats.hit_rate() - (2.0 / 3.0)).abs() < 0.001);

        stats.record_add(1000);
        assert_eq!(stats.entries_added, 1);
        assert_eq!(stats.current_entries, 1);
        assert_eq!(stats.memory_bytes, 1000);

        stats.record_evict(500);
        assert_eq!(stats.entries_evicted, 1);
        assert_eq!(stats.current_entries, 0);
        assert_eq!(stats.memory_bytes, 500);

        stats.reset();
        assert_eq!(stats.lookups, 0);
        assert_eq!(stats.memory_bytes, 500); // Not reset
    }

    #[test]
    fn test_invalidation_reason() {
        assert!(InvalidationReason::BufferChange.is_buffer_related());
        assert!(InvalidationReason::BufferUnload.is_buffer_related());
        assert!(!InvalidationReason::WindowClose.is_buffer_related());

        assert!(InvalidationReason::WindowClose.is_window_related());
        assert!(InvalidationReason::WindowResize.is_window_related());
        assert!(!InvalidationReason::BufferChange.is_window_related());
    }

    #[test]
    fn test_invalidation_request() {
        let entry = CacheEntry::for_lines(1, 100, 10, 20);

        // All request matches everything
        let all = InvalidationRequest::all(InvalidationReason::Manual);
        assert!(all.matches_entry(&entry));

        // Buffer request
        let buf_req = InvalidationRequest::for_buffer(100, InvalidationReason::BufferChange);
        assert!(buf_req.matches_entry(&entry));

        let other_buf = InvalidationRequest::for_buffer(200, InvalidationReason::BufferChange);
        assert!(!other_buf.matches_entry(&entry));

        // Line range request
        let overlapping =
            InvalidationRequest::for_lines(100, 15, 25, InvalidationReason::BufferChange);
        assert!(overlapping.matches_entry(&entry));

        let non_overlapping =
            InvalidationRequest::for_lines(100, 25, 30, InvalidationReason::BufferChange);
        assert!(!non_overlapping.matches_entry(&entry));

        // Provider request
        let prov_req = InvalidationRequest::for_provider(1, InvalidationReason::ProviderRemoved);
        assert!(prov_req.matches_entry(&entry));

        let other_prov = InvalidationRequest::for_provider(2, InvalidationReason::ProviderRemoved);
        assert!(!other_prov.matches_entry(&entry));
    }
}
