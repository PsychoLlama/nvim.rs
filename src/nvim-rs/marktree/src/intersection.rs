//! Intersection management for marktree paired marks
//!
//! Intersections track which paired marks span across a node's subtree.
//! This allows efficient overlap queries without traversing the entire tree.
//!
//! The intersection list is a sorted vector of mark IDs (u64), stored
//! inline (up to 4 elements) or on the heap.

use std::cmp::Ordering;

use crate::MTNodeHandle;

// ============================================================================
// C Accessor Functions for Intersection Operations
// ============================================================================

extern "C" {
    // Intersection array operations
    fn nvim_mtnode_get_intersect_size(x: MTNodeHandle) -> usize;
    fn nvim_mtnode_get_intersect_elem(x: MTNodeHandle, idx: usize) -> u64;
}

// ============================================================================
// Pure Rust Implementation - Intersection Type
// ============================================================================

/// A sorted vector of u64 mark IDs, optimized for small sizes.
///
/// Uses inline storage for up to 4 elements (matching C's kvec_withinit_t).
#[derive(Debug, Clone, Default)]
pub struct Intersection {
    data: Vec<u64>,
}

impl Intersection {
    /// Create a new empty intersection.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Create from a slice (assumes sorted).
    #[must_use]
    pub fn from_sorted(data: &[u64]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    /// Get the number of elements.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get element at index.
    #[inline]
    #[must_use]
    pub fn get(&self, idx: usize) -> Option<u64> {
        self.data.get(idx).copied()
    }

    /// Get the underlying slice.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[u64] {
        &self.data
    }

    /// Binary search for membership.
    #[must_use]
    pub fn has(&self, id: u64) -> bool {
        self.data.binary_search(&id).is_ok()
    }

    /// Insert id in sorted order (no-op if already present).
    pub fn insert(&mut self, id: u64) {
        match self.data.binary_search(&id) {
            Ok(_) => {} // Already present
            Err(pos) => self.data.insert(pos, id),
        }
    }

    /// Remove id (no-op if not present).
    pub fn remove(&mut self, id: u64) -> bool {
        match self.data.binary_search(&id) {
            Ok(pos) => {
                self.data.remove(pos);
                true
            }
            Err(_) => false,
        }
    }

    /// Clear all elements.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Compute intersection: self = self & other
    pub fn intersect_with(&mut self, other: &Self) {
        let result = intersect_common_impl(&self.data, &other.data);
        self.data = result;
    }

    /// Compute union: self = self | other
    pub fn union_with(&mut self, other: &Self) {
        intersect_add_impl(&mut self.data, &other.data);
    }

    /// Compute difference: self = self - other
    pub fn subtract(&mut self, other: &Self) {
        intersect_sub_impl(&mut self.data, &other.data);
    }
}

// ============================================================================
// Pure Rust Implementation - Set Operations
// ============================================================================

/// Compute intersection of two sorted slices.
#[must_use]
fn intersect_common_impl(x: &[u64], y: &[u64]) -> Vec<u64> {
    let mut result = Vec::new();
    let mut xi = 0;
    let mut yi = 0;

    while xi < x.len() && yi < y.len() {
        match x[xi].cmp(&y[yi]) {
            Ordering::Equal => {
                result.push(x[xi]);
                xi += 1;
                yi += 1;
            }
            Ordering::Less => xi += 1,
            Ordering::Greater => yi += 1,
        }
    }

    result
}

/// In-place union: x |= y
fn intersect_add_impl(x: &mut Vec<u64>, y: &[u64]) {
    let mut xi = 0;
    let mut yi = 0;

    while xi < x.len() && yi < y.len() {
        match y[yi].cmp(&x[xi]) {
            Ordering::Equal => {
                xi += 1;
                yi += 1;
            }
            Ordering::Less => {
                x.insert(xi, y[yi]);
                xi += 1; // Skip newly inserted
                yi += 1;
            }
            Ordering::Greater => xi += 1,
        }
    }

    // Append remaining from y
    x.extend_from_slice(&y[yi..]);
}

/// In-place difference: x -= y
fn intersect_sub_impl(x: &mut Vec<u64>, y: &[u64]) {
    let mut xi = 0;
    let mut yi = 0;
    let mut write_idx = 0;

    while xi < x.len() && yi < y.len() {
        match x[xi].cmp(&y[yi]) {
            Ordering::Equal => {
                xi += 1;
                yi += 1;
            }
            Ordering::Less => {
                x[write_idx] = x[xi];
                write_idx += 1;
                xi += 1;
            }
            Ordering::Greater => yi += 1,
        }
    }

    // Copy remaining from x
    while xi < x.len() {
        x[write_idx] = x[xi];
        write_idx += 1;
        xi += 1;
    }

    x.truncate(write_idx);
}

/// Compute merge operation: find common elements and remove them from both.
/// Returns the common elements.
#[must_use]
pub fn intersect_merge_impl(x: &mut Vec<u64>, y: &mut Vec<u64>) -> Vec<u64> {
    let mut common = Vec::new();
    let mut xi = 0;
    let mut yi = 0;
    let mut xn = 0;
    let mut yn = 0;

    while xi < x.len() && yi < y.len() {
        match x[xi].cmp(&y[yi]) {
            Ordering::Equal => {
                common.push(x[xi]);
                xi += 1;
                yi += 1;
            }
            Ordering::Less => {
                x[xn] = x[xi];
                xn += 1;
                xi += 1;
            }
            Ordering::Greater => {
                y[yn] = y[yi];
                yn += 1;
                yi += 1;
            }
        }
    }

    // Copy remaining
    while xi < x.len() {
        x[xn] = x[xi];
        xn += 1;
        xi += 1;
    }
    while yi < y.len() {
        y[yn] = y[yi];
        yn += 1;
        yi += 1;
    }

    x.truncate(xn);
    y.truncate(yn);

    common
}

// ============================================================================
// FFI Exports
// ============================================================================

/// Check if intersection contains the given ID.
#[no_mangle]
pub extern "C" fn rs_intersection_has(x: MTNodeHandle, id: u64) -> bool {
    let size = unsafe { nvim_mtnode_get_intersect_size(x) };
    for i in 0..size {
        let elem = unsafe { nvim_mtnode_get_intersect_elem(x, i) };
        if elem == id {
            return true;
        } else if elem > id {
            return false; // Sorted, so no need to continue
        }
    }
    false
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersection_insert_remove() {
        let mut i = Intersection::new();
        assert!(i.is_empty());

        i.insert(5);
        i.insert(3);
        i.insert(7);
        i.insert(3); // Duplicate

        assert_eq!(i.len(), 3);
        assert_eq!(i.as_slice(), &[3, 5, 7]);
        assert!(i.has(5));
        assert!(!i.has(4));

        assert!(i.remove(5));
        assert!(!i.remove(5)); // Already removed
        assert_eq!(i.as_slice(), &[3, 7]);
    }

    #[test]
    fn test_intersect_common() {
        let x = vec![1, 3, 5, 7, 9];
        let y = vec![2, 3, 5, 8];
        let result = intersect_common_impl(&x, &y);
        assert_eq!(result, vec![3, 5]);
    }

    #[test]
    fn test_intersect_add() {
        let mut x = vec![1, 3, 5];
        let y = vec![2, 3, 6];
        intersect_add_impl(&mut x, &y);
        assert_eq!(x, vec![1, 2, 3, 5, 6]);
    }

    #[test]
    fn test_intersect_sub() {
        let mut x = vec![1, 2, 3, 5, 7];
        let y = vec![2, 3, 6];
        intersect_sub_impl(&mut x, &y);
        assert_eq!(x, vec![1, 5, 7]);
    }

    #[test]
    fn test_intersect_merge() {
        let mut x = vec![1, 3, 5, 7];
        let mut y = vec![2, 3, 5, 8];
        let common = intersect_merge_impl(&mut x, &mut y);
        assert_eq!(common, vec![3, 5]);
        assert_eq!(x, vec![1, 7]);
        assert_eq!(y, vec![2, 8]);
    }
}
