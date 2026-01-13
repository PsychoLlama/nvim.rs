//! Compositor State Management
//!
//! This module provides state tracking and management utilities for the
//! compositor, including damage region tracking, flush coordination, and
//! the compositor state machine.
//! Phase 174 of Rust migration.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_lossless)]

use std::ffi::c_int;

use crate::floating::ClipRect;
use crate::ScreenGridHandle;

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    fn nvim_get_composed_uis() -> c_int;
    fn nvim_get_valid_screen() -> c_int;
    fn nvim_screengrid_get_comp_index(grid: ScreenGridHandle) -> usize;
    fn nvim_screengrid_get_pending_comp_index_update(grid: ScreenGridHandle) -> bool;
    fn nvim_screengrid_set_pending_comp_index_update(grid: ScreenGridHandle, val: bool);
}

// =============================================================================
// Compositor State
// =============================================================================

/// Compositor operation states
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositorState {
    /// Compositor is idle, no pending operations
    Idle = 0,
    /// Compositor is actively composing
    Composing = 1,
    /// Compositor needs flush
    NeedsFlush = 2,
    /// Compositor is flushing
    Flushing = 3,
}

impl CompositorState {
    /// Create from integer value
    pub fn from_int(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Idle),
            1 => Some(Self::Composing),
            2 => Some(Self::NeedsFlush),
            3 => Some(Self::Flushing),
            _ => None,
        }
    }

    /// Check if compositor is active (composing or flushing)
    pub fn is_active(self) -> bool {
        matches!(self, Self::Composing | Self::Flushing)
    }

    /// Check if compositor can accept new operations
    pub fn can_compose(self) -> bool {
        matches!(self, Self::Idle | Self::NeedsFlush)
    }
}

/// Get idle state value
#[no_mangle]
pub extern "C" fn rs_compositor_state_idle() -> c_int {
    CompositorState::Idle as c_int
}

/// Get composing state value
#[no_mangle]
pub extern "C" fn rs_compositor_state_composing() -> c_int {
    CompositorState::Composing as c_int
}

/// Get needs_flush state value
#[no_mangle]
pub extern "C" fn rs_compositor_state_needs_flush() -> c_int {
    CompositorState::NeedsFlush as c_int
}

/// Get flushing state value
#[no_mangle]
pub extern "C" fn rs_compositor_state_flushing() -> c_int {
    CompositorState::Flushing as c_int
}

/// Check if state is active
#[no_mangle]
pub extern "C" fn rs_compositor_state_is_active(state: c_int) -> c_int {
    c_int::from(CompositorState::from_int(state).is_some_and(CompositorState::is_active))
}

/// Check if state can compose
#[no_mangle]
pub extern "C" fn rs_compositor_state_can_compose(state: c_int) -> c_int {
    c_int::from(CompositorState::from_int(state).is_some_and(CompositorState::can_compose))
}

// =============================================================================
// Damage Region Tracking
// =============================================================================

/// Maximum number of damage regions to track before coalescing
const MAX_DAMAGE_REGIONS: usize = 8;

/// Damage region accumulator for efficient redraw tracking.
///
/// Tracks multiple damaged regions and coalesces them when there are too many.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct DamageTracker {
    /// Individual damaged regions
    regions: [ClipRect; MAX_DAMAGE_REGIONS],
    /// Number of valid regions
    count: usize,
    /// Bounding box of all damage (for fast total query)
    total_bounds: ClipRect,
    /// Whether damage has been added since last clear
    dirty: bool,
    /// Whether regions were coalesced (exceeded MAX_DAMAGE_REGIONS)
    coalesced: bool,
}

impl Default for DamageTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl DamageTracker {
    /// Create a new empty damage tracker.
    pub const fn new() -> Self {
        Self {
            regions: [ClipRect::empty(); MAX_DAMAGE_REGIONS],
            count: 0,
            total_bounds: ClipRect::empty(),
            dirty: false,
            coalesced: false,
        }
    }

    /// Clear all tracked damage.
    pub fn clear(&mut self) {
        self.count = 0;
        self.total_bounds = ClipRect::empty();
        self.dirty = false;
        self.coalesced = false;
    }

    /// Check if any damage is tracked.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Check if damage has been added since last clear.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Get the total bounding box of all damage.
    pub fn bounds(&self) -> ClipRect {
        self.total_bounds
    }

    /// Get number of tracked regions.
    pub fn region_count(&self) -> usize {
        self.count
    }

    /// Add a damaged region.
    ///
    /// If too many regions are tracked, coalesces into the bounding box.
    pub fn add_region(&mut self, region: ClipRect) {
        if region.is_empty() {
            return;
        }

        self.dirty = true;
        self.total_bounds = self.total_bounds.union(&region);

        if self.count < MAX_DAMAGE_REGIONS {
            self.regions[self.count] = region;
            self.count += 1;
        } else {
            // Mark as coalesced - we've exceeded MAX_DAMAGE_REGIONS
            // so we just use total_bounds for all queries
            self.coalesced = true;
        }
    }

    /// Add damage for a rectangular area.
    pub fn add_rect(&mut self, top: c_int, bottom: c_int, left: c_int, right: c_int) {
        self.add_region(ClipRect::new(top, bottom, left, right));
    }

    /// Check if a point is in any damaged region.
    pub fn contains_point(&self, row: c_int, col: c_int) -> bool {
        if self.count == 0 && !self.coalesced {
            return false;
        }

        // Fast check against total bounds first
        if !self.total_bounds.contains_point(row, col) {
            return false;
        }

        // If coalesced, assume all points in bounds are damaged
        if self.coalesced {
            return true;
        }

        // Check individual regions
        for i in 0..self.count {
            if self.regions[i].contains_point(row, col) {
                return true;
            }
        }
        false
    }

    /// Check if a region overlaps any damaged area.
    pub fn overlaps(&self, region: &ClipRect) -> bool {
        if (self.count == 0 && !self.coalesced) || region.is_empty() {
            return false;
        }

        // Fast check against total bounds
        if !self.total_bounds.overlaps(region) {
            return false;
        }

        // If coalesced, assume all regions in bounds overlap
        if self.coalesced {
            return true;
        }

        // Check individual regions
        for i in 0..self.count {
            if self.regions[i].overlaps(region) {
                return true;
            }
        }
        false
    }
}

// =============================================================================
// FFI Exports for DamageTracker
// =============================================================================

/// Create a new damage tracker.
#[no_mangle]
pub extern "C" fn rs_damage_tracker_new() -> DamageTracker {
    DamageTracker::new()
}

/// Clear damage tracker.
#[no_mangle]
pub extern "C" fn rs_damage_tracker_clear(tracker: &mut DamageTracker) {
    tracker.clear();
}

/// Check if damage tracker is empty.
#[no_mangle]
pub extern "C" fn rs_damage_tracker_is_empty(tracker: &DamageTracker) -> c_int {
    c_int::from(tracker.is_empty())
}

/// Check if damage tracker is dirty.
#[no_mangle]
pub extern "C" fn rs_damage_tracker_is_dirty(tracker: &DamageTracker) -> c_int {
    c_int::from(tracker.is_dirty())
}

/// Get damage tracker total bounds.
#[no_mangle]
pub extern "C" fn rs_damage_tracker_bounds(tracker: &DamageTracker) -> ClipRect {
    tracker.bounds()
}

/// Get damage tracker region count.
#[no_mangle]
pub extern "C" fn rs_damage_tracker_region_count(tracker: &DamageTracker) -> usize {
    tracker.region_count()
}

/// Add damage region.
#[no_mangle]
pub extern "C" fn rs_damage_tracker_add_region(tracker: &mut DamageTracker, region: &ClipRect) {
    tracker.add_region(*region);
}

/// Add damage rectangle.
#[no_mangle]
pub extern "C" fn rs_damage_tracker_add_rect(
    tracker: &mut DamageTracker,
    top: c_int,
    bottom: c_int,
    left: c_int,
    right: c_int,
) {
    tracker.add_rect(top, bottom, left, right);
}

/// Check if point is damaged.
#[no_mangle]
pub extern "C" fn rs_damage_tracker_contains(
    tracker: &DamageTracker,
    row: c_int,
    col: c_int,
) -> c_int {
    c_int::from(tracker.contains_point(row, col))
}

/// Check if region overlaps damage.
#[no_mangle]
pub extern "C" fn rs_damage_tracker_overlaps(tracker: &DamageTracker, region: &ClipRect) -> c_int {
    c_int::from(tracker.overlaps(region))
}

// =============================================================================
// Flush Coordination
// =============================================================================

/// Check if compositor should perform operations.
///
/// Returns true if there are composed UIs and the screen is valid.
#[no_mangle]
pub extern "C" fn rs_compositor_should_draw() -> c_int {
    unsafe { c_int::from(nvim_get_composed_uis() != 0 && nvim_get_valid_screen() != 0) }
}

/// Check if compositor has any attached UIs.
#[no_mangle]
pub extern "C" fn rs_compositor_has_uis() -> c_int {
    unsafe { c_int::from(nvim_get_composed_uis() > 0) }
}

/// Get number of composed UIs.
#[no_mangle]
pub extern "C" fn rs_compositor_ui_count() -> c_int {
    unsafe { nvim_get_composed_uis() }
}

/// Check if screen state is valid for compositing.
#[no_mangle]
pub extern "C" fn rs_compositor_screen_valid() -> c_int {
    unsafe { nvim_get_valid_screen() }
}

// =============================================================================
// Grid Index Update Tracking
// =============================================================================

/// Check if a grid has pending compositor index update.
///
/// # Safety
/// `grid` must be a valid ScreenGrid handle.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_has_pending_update(grid: ScreenGridHandle) -> c_int {
    if grid.is_null() {
        return 0;
    }
    c_int::from(nvim_screengrid_get_pending_comp_index_update(grid))
}

/// Mark grid as having pending compositor update.
///
/// # Safety
/// `grid` must be a valid ScreenGrid handle.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_mark_pending_update(grid: ScreenGridHandle, pending: c_int) {
    if !grid.is_null() {
        nvim_screengrid_set_pending_comp_index_update(grid, pending != 0);
    }
}

/// Get grid's compositor index.
///
/// # Safety
/// `grid` must be a valid ScreenGrid handle.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_comp_index(grid: ScreenGridHandle) -> usize {
    if grid.is_null() {
        return 0;
    }
    nvim_screengrid_get_comp_index(grid)
}

/// Check if grid is in the compositor layer stack.
///
/// A grid is in the stack if its comp_index is non-zero.
///
/// # Safety
/// `grid` must be a valid ScreenGrid handle.
#[no_mangle]
pub unsafe extern "C" fn rs_grid_in_layer_stack(grid: ScreenGridHandle) -> c_int {
    if grid.is_null() {
        return 0;
    }
    c_int::from(nvim_screengrid_get_comp_index(grid) != 0)
}

// =============================================================================
// Debug Helpers
// =============================================================================

/// Debug highlight types for compositor visualization.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugHighlightType {
    /// Normal content
    Normal = 0,
    /// Cleared area
    Clear = 1,
    /// Composed/blended area
    Composed = 2,
    /// Area being recomposed
    Recompose = 3,
}

/// Get debug highlight type value.
#[no_mangle]
pub extern "C" fn rs_debug_hl_normal() -> c_int {
    DebugHighlightType::Normal as c_int
}

/// Get debug highlight type value.
#[no_mangle]
pub extern "C" fn rs_debug_hl_clear() -> c_int {
    DebugHighlightType::Clear as c_int
}

/// Get debug highlight type value.
#[no_mangle]
pub extern "C" fn rs_debug_hl_composed() -> c_int {
    DebugHighlightType::Composed as c_int
}

/// Get debug highlight type value.
#[no_mangle]
pub extern "C" fn rs_debug_hl_recompose() -> c_int {
    DebugHighlightType::Recompose as c_int
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compositor_state() {
        assert_eq!(rs_compositor_state_idle(), 0);
        assert_eq!(rs_compositor_state_composing(), 1);
        assert_eq!(rs_compositor_state_needs_flush(), 2);
        assert_eq!(rs_compositor_state_flushing(), 3);

        // Active states
        assert_eq!(rs_compositor_state_is_active(0), 0); // Idle not active
        assert_eq!(rs_compositor_state_is_active(1), 1); // Composing is active
        assert_eq!(rs_compositor_state_is_active(3), 1); // Flushing is active

        // Can compose
        assert_eq!(rs_compositor_state_can_compose(0), 1); // Idle can compose
        assert_eq!(rs_compositor_state_can_compose(2), 1); // NeedsFlush can compose
        assert_eq!(rs_compositor_state_can_compose(1), 0); // Composing cannot
    }

    #[test]
    fn test_damage_tracker_basic() {
        let mut tracker = DamageTracker::new();
        assert!(tracker.is_empty());
        assert!(!tracker.is_dirty());

        tracker.add_rect(10, 20, 5, 15);
        assert!(!tracker.is_empty());
        assert!(tracker.is_dirty());
        assert_eq!(tracker.region_count(), 1);

        let bounds = tracker.bounds();
        assert_eq!(bounds.top, 10);
        assert_eq!(bounds.bottom, 20);
        assert_eq!(bounds.left, 5);
        assert_eq!(bounds.right, 15);
    }

    #[test]
    fn test_damage_tracker_multiple_regions() {
        let mut tracker = DamageTracker::new();

        tracker.add_rect(10, 20, 5, 15);
        tracker.add_rect(30, 40, 25, 35);

        assert_eq!(tracker.region_count(), 2);

        // Total bounds should encompass both
        let bounds = tracker.bounds();
        assert_eq!(bounds.top, 10);
        assert_eq!(bounds.bottom, 40);
        assert_eq!(bounds.left, 5);
        assert_eq!(bounds.right, 35);
    }

    #[test]
    fn test_damage_tracker_contains() {
        let mut tracker = DamageTracker::new();

        tracker.add_rect(10, 20, 5, 15);

        // Inside region
        assert!(tracker.contains_point(15, 10));

        // Outside region but in bounds
        assert!(!tracker.contains_point(25, 10));

        // Completely outside
        assert!(!tracker.contains_point(5, 3));
    }

    #[test]
    fn test_damage_tracker_overlaps() {
        let mut tracker = DamageTracker::new();

        tracker.add_rect(10, 20, 5, 15);

        // Overlapping region
        let overlapping = ClipRect::new(15, 25, 10, 20);
        assert!(tracker.overlaps(&overlapping));

        // Non-overlapping region
        let not_overlapping = ClipRect::new(30, 40, 5, 15);
        assert!(!tracker.overlaps(&not_overlapping));
    }

    #[test]
    fn test_damage_tracker_clear() {
        let mut tracker = DamageTracker::new();

        tracker.add_rect(10, 20, 5, 15);
        assert!(!tracker.is_empty());

        tracker.clear();
        assert!(tracker.is_empty());
        assert!(!tracker.is_dirty());
    }

    #[test]
    fn test_damage_tracker_coalesce() {
        let mut tracker = DamageTracker::new();

        // Add more regions than MAX_DAMAGE_REGIONS
        for i in 0..MAX_DAMAGE_REGIONS + 2 {
            tracker.add_rect(i as c_int * 10, (i as c_int + 1) * 10, 0, 10);
        }

        // Region count stops at MAX
        assert_eq!(tracker.region_count(), MAX_DAMAGE_REGIONS);

        // But total bounds still tracked
        assert!(!tracker.bounds().is_empty());

        // Contains should use bounds check for efficiency
        let last_row = (MAX_DAMAGE_REGIONS + 1) as c_int * 10 + 5;
        assert!(tracker.contains_point(last_row, 5));
    }

    #[test]
    fn test_debug_highlight_types() {
        assert_eq!(rs_debug_hl_normal(), 0);
        assert_eq!(rs_debug_hl_clear(), 1);
        assert_eq!(rs_debug_hl_composed(), 2);
        assert_eq!(rs_debug_hl_recompose(), 3);
    }
}
