//! Intersection set operations for B-tree node handling during deletion.
//!
//! These functions replace the C `intersect_merge`, `intersect_mov`,
//! `intersect_add`, `intersect_sub`, `intersect_common`, and `kvi_move`
//! functions, operating on `MTNodeHandle` instead of raw `Intersection*`.

use std::ffi::c_int;

use crate::intersection::intersect_merge_impl;
use crate::{MTNodeHandle, MarkTreeHandle};

// ============================================================================
// C accessors used here
// ============================================================================

extern "C" {
    fn nvim_mtnode_get_intersect_size(x: MTNodeHandle) -> usize;
    fn nvim_mtnode_get_intersect_elem(x: MTNodeHandle, idx: usize) -> u64;
    fn nvim_mtnode_intersect_clear(x: MTNodeHandle);
    fn nvim_mtnode_intersect_push(x: MTNodeHandle, id: u64);
    fn nvim_mtnode_get_ptr(x: MTNodeHandle, idx: c_int) -> MTNodeHandle;
    fn rs_intersect_node(b: MarkTreeHandle, x: MTNodeHandle, id: u64);
}

// ============================================================================
// Helpers: read/write node intersection lists as Vec<u64>
// ============================================================================

/// Read a node's intersection list into a Vec<u64>.
fn read_intersect(x: MTNodeHandle) -> Vec<u64> {
    let size = unsafe { nvim_mtnode_get_intersect_size(x) };
    let mut v = Vec::with_capacity(size);
    for i in 0..size {
        v.push(unsafe { nvim_mtnode_get_intersect_elem(x, i) });
    }
    v
}

/// Write a sorted Vec<u64> to a node's intersection list (clear then push each).
fn write_intersect(x: MTNodeHandle, data: &[u64]) {
    unsafe { nvim_mtnode_intersect_clear(x) };
    for &id in data {
        unsafe { nvim_mtnode_intersect_push(x, id) };
    }
}

/// Apply each id in `ids` to `node` via rs_intersect_node.
fn apply_intersect_to_node(b: MarkTreeHandle, node: MTNodeHandle, ids: &[u64]) {
    for &id in ids {
        unsafe { rs_intersect_node(b, node, id) };
    }
}

// ============================================================================
// Pure set operations on sorted Vec<u64>
// ============================================================================

/// In-place union: x |= y
pub fn intersect_add_vec(x: &mut Vec<u64>, y: &[u64]) {
    let mut xi = 0;
    let mut yi = 0;

    while xi < x.len() && yi < y.len() {
        match x[xi].cmp(&y[yi]) {
            std::cmp::Ordering::Equal => {
                xi += 1;
                yi += 1;
            }
            std::cmp::Ordering::Greater => {
                x.insert(xi, y[yi]);
                xi += 1; // skip newly inserted element
                yi += 1;
            }
            std::cmp::Ordering::Less => {
                xi += 1;
            }
        }
    }
    // Append remaining from y
    x.extend_from_slice(&y[yi..]);
}

/// intersect_mov: adjust intersections when child `w` moves from parent `x` to parent `y`.
///
/// - x is the old parent's intersect list (read-only conceptually)
/// - y is the new parent's intersect list (modified in-place, kept elements remain)
/// - w is the child's intersect list (modified in-place)
/// - d collects elements from y that must be propagated to y's other old children
pub fn intersect_mov_vec(x: &[u64], y: &mut Vec<u64>, w: &mut Vec<u64>, d: &mut Vec<u64>) {
    let mut wi = 0;
    let mut yi = 0;
    let mut wn = 0;
    let mut yn = 0;
    let mut xi = 0;

    while wi < w.len() || xi < x.len() {
        if wi < w.len() && (xi >= x.len() || x[xi] >= w[wi]) {
            if xi < x.len() && x[xi] == w[wi] {
                xi += 1;
            }
            // w[wi] < x[xi] strictly (or x exhausted)
            while yi < y.len() && y[yi] < w[wi] {
                d.push(y[yi]);
                yi += 1;
            }
            if yi < y.len() && y[yi] == w[wi] {
                // Keep y[yi] in y at position yn; w[wi] is consumed/matched, not kept in w
                y[yn] = y[yi];
                yn += 1;
                yi += 1;
            } else {
                // w[wi] not in y: keep w[wi] in w at position wn
                w[wn] = w[wi];
                wn += 1;
            }
            wi += 1;
        } else {
            // x[xi] < w[wi] strictly (or w exhausted)
            while yi < y.len() && y[yi] < x[xi] {
                d.push(y[yi]);
                yi += 1;
            }
            if yi < y.len() && y[yi] == x[xi] {
                // Keep y[yi] in y at position yn
                y[yn] = y[yi];
                yn += 1;
                yi += 1;
            } else {
                // x[xi] not in y and not in w: insert into w at position wn
                if wi == wn {
                    // wi and wn are at same position: insert x[xi] here, shifting rest right
                    w.insert(wn, x[xi]);
                    wn += 1;
                    wi += 1; // skip the inserted element
                } else {
                    // wn < wi: write at wn without disrupting wi
                    w[wn] = x[xi];
                    wn += 1;
                }
            }
            xi += 1;
        }
    }

    // Move remaining y elements to d
    if yi < y.len() {
        d.extend_from_slice(&y[yi..]);
    }

    w.truncate(wn);
    y.truncate(yn);
}

// ============================================================================
// Node-handle-based functions for merge_node
// ============================================================================

/// Perform the intersection update for merge_node (internal nodes only).
///
/// This function replaces the following C pattern in merge_node:
/// ```c
/// Intersection mi;
/// kvi_init(mi);
/// intersect_merge(&mi, &x->intersect, &y->intersect);
/// for k in 0..x_old_n+1: for id in x->intersect: rs_intersect_node(b, x->ptr[k], id)
/// for ky in 0..y_n+1:
///     k = x_old_n + ky + 1
///     x->ptr[k]->parent = x; x->ptr[k]->p_idx = k
///     for id in y->intersect: rs_intersect_node(b, x->ptr[k], id)
/// kvi_destroy(x->intersect); kvi_move(&x->intersect, &mi)
/// ```
///
/// Precondition: y's child ptrs have already been memmoved into x at positions
/// x_old_n+1 .. x_old_n+1+y_n+1 (so x->ptr[x_old_n+1+ky] == y->ptr[ky]).
///
/// `x_old_n` = x->n before the merge (number of old x children = x_old_n + 1)
/// `y_n` = y->n (number of y children = y_n + 1)
#[no_mangle]
pub extern "C" fn rs_merge_node_intersect(
    b: MarkTreeHandle,
    x_node: MTNodeHandle,
    x_old_n: c_int,
    y_node: MTNodeHandle,
    y_n: c_int,
) {
    let mut x_list = read_intersect(x_node);
    let mut y_list = read_intersect(y_node);

    // Compute merge: x_list becomes x_unique, y_list becomes y_unique, common is returned
    let common = intersect_merge_impl(&mut x_list, &mut y_list);

    // Apply x_unique to x's old children (indices 0 .. x_old_n inclusive)
    for k in 0..=(x_old_n) {
        let child = unsafe { nvim_mtnode_get_ptr(x_node, k) };
        apply_intersect_to_node(b, child, &x_list);
    }

    // Apply y_unique to y's children (now at x's indices x_old_n+1+ky)
    for ky in 0..=(y_n) {
        let k = x_old_n + ky + 1;
        let child = unsafe { nvim_mtnode_get_ptr(x_node, k) };
        apply_intersect_to_node(b, child, &y_list);
    }

    // Set x_node->intersect = common
    write_intersect(x_node, &common);
    // y_node->intersect will be freed by C's marktree_free_node; write y_unique back for consistency
    write_intersect(y_node, &y_list);
}

// ============================================================================
// Node-handle-based functions for pivot_right / pivot_left
// ============================================================================

/// Perform the intersect_mov update for pivot_right (internal nodes only).
///
/// This replaces the following C pattern in pivot_right:
/// ```c
/// Intersection d;
/// kvi_init(d);
/// intersect_mov(&x->intersect, &y->intersect, &y->ptr[0]->intersect, &d);
/// if (kv_size(d)) {
///     for yi in 1..y->n+1: intersect_add(&y->ptr[yi]->intersect, &d)
/// }
/// kvi_destroy(d);
/// ```
///
/// `y_n` = y->n after the pivot (after the push, before calling this).
/// The moved child is at y->ptr[0].
#[no_mangle]
pub extern "C" fn rs_pivot_right_intersect(
    _b: MarkTreeHandle,
    x_node: MTNodeHandle,
    y_node: MTNodeHandle,
    y_n: c_int,
) {
    let x_list = read_intersect(x_node);
    let mut y_list = read_intersect(y_node);
    let w_node = unsafe { nvim_mtnode_get_ptr(y_node, 0) };
    let mut w_list = read_intersect(w_node);
    let mut d_list: Vec<u64> = Vec::new();

    intersect_mov_vec(&x_list, &mut y_list, &mut w_list, &mut d_list);

    write_intersect(w_node, &w_list);
    write_intersect(y_node, &y_list);

    if !d_list.is_empty() {
        for yi in 1..=(y_n) {
            let child = unsafe { nvim_mtnode_get_ptr(y_node, yi) };
            let mut child_list = read_intersect(child);
            intersect_add_vec(&mut child_list, &d_list);
            write_intersect(child, &child_list);
        }
    }
}

/// Perform the intersect_mov update for pivot_left (internal nodes only).
///
/// This replaces the following C pattern in pivot_left:
/// ```c
/// Intersection d;
/// kvi_init(d);
/// intersect_mov(&y->intersect, &x->intersect, &x->ptr[x->n]->intersect, &d);
/// if (kv_size(d)) {
///     for xi in 0..x->n: intersect_add(&x->ptr[xi]->intersect, &d)
/// }
/// kvi_destroy(d);
/// ```
///
/// `x_n` = x->n after the pivot (the new last child of x is at x->ptr[x_n]).
/// The moved child is at x->ptr[x_n].
#[no_mangle]
pub extern "C" fn rs_pivot_left_intersect(
    _b: MarkTreeHandle,
    x_node: MTNodeHandle,
    x_n: c_int,
    y_node: MTNodeHandle,
) {
    let y_list = read_intersect(y_node);
    let mut x_list = read_intersect(x_node);
    let w_node = unsafe { nvim_mtnode_get_ptr(x_node, x_n) };
    let mut w_list = read_intersect(w_node);
    let mut d_list: Vec<u64> = Vec::new();

    intersect_mov_vec(&y_list, &mut x_list, &mut w_list, &mut d_list);

    write_intersect(w_node, &w_list);
    write_intersect(x_node, &x_list);

    if !d_list.is_empty() {
        // Apply delta to x's old children (0 .. x_n-1, deliberately skipping x->ptr[x_n])
        for xi in 0..(x_n) {
            let child = unsafe { nvim_mtnode_get_ptr(x_node, xi) };
            let mut child_list = read_intersect(child);
            intersect_add_vec(&mut child_list, &d_list);
            write_intersect(child, &child_list);
        }
    }
}

// ============================================================================
// intersect_mov_test: test harness (pure, no node handles)
// ============================================================================

/// Test harness for intersect_mov. Takes raw arrays, returns results.
#[no_mangle]
pub extern "C" fn rs_intersect_mov_test(
    x: *const u64,
    nx: usize,
    y: *const u64,
    ny: usize,
    win: *const u64,
    nwin: usize,
    wout: *mut u64,
    w_result_size: *mut usize,
    dout: *mut u64,
    d_result_size: *mut usize,
) -> bool {
    // SAFETY: Caller must provide valid pointers and sizes
    let x_vec = unsafe { std::slice::from_raw_parts(x, nx) }.to_vec();
    let mut y_vec = unsafe { std::slice::from_raw_parts(y, ny) }.to_vec();
    let mut w_vec = unsafe { std::slice::from_raw_parts(win, nwin) }.to_vec();
    let mut d_vec: Vec<u64> = Vec::new();

    intersect_mov_vec(&x_vec, &mut y_vec, &mut w_vec, &mut d_vec);

    let w_cap = unsafe { *w_result_size };
    let d_cap = unsafe { *d_result_size };

    if w_vec.len() > w_cap || d_vec.len() > d_cap {
        return false;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(w_vec.as_ptr(), wout, w_vec.len());
        *w_result_size = w_vec.len();
        std::ptr::copy_nonoverlapping(d_vec.as_ptr(), dout, d_vec.len());
        *d_result_size = d_vec.len();
    }

    true
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersect_add_vec() {
        let mut x = vec![1u64, 3, 5];
        intersect_add_vec(&mut x, &[2, 3, 6]);
        assert_eq!(x, [1, 2, 3, 5, 6]);
    }

    #[test]
    fn test_intersect_add_empty_y() {
        let mut x = vec![1u64, 2, 3];
        intersect_add_vec(&mut x, &[]);
        assert_eq!(x, [1, 2, 3]);
    }

    #[test]
    fn test_intersect_add_empty_x() {
        let mut x: Vec<u64> = vec![];
        intersect_add_vec(&mut x, &[1, 2, 3]);
        assert_eq!(x, [1, 2, 3]);
    }

    #[test]
    fn test_intersect_mov_basic() {
        // x=[1,3], y=[2,3,4], w=[1,3] -> w moved from x to y
        // w is in x so 1,3 were inherited. Now in y, adjust:
        // 1 is in x but not in y -> insert into w
        // 3 is in x AND y -> keep in y, not in w
        // 2,4 in y not in x or w -> go to d
        let x = [1u64, 3];
        let mut y = vec![2u64, 3, 4];
        let mut w = vec![1u64, 3];
        let mut d: Vec<u64> = vec![];

        intersect_mov_vec(&x, &mut y, &mut w, &mut d);

        // After: y should keep 3 (it's in x), discard 2,4 (to d)
        // w should keep 1 (in x not in y), discard 3 (in both x and y -> already in y)
        // d = [2, 4]
        assert_eq!(y, [3]);
        assert_eq!(w, [1]);
        assert_eq!(d, [2, 4]);
    }

    #[test]
    fn test_intersect_mov_via_test_fn() {
        let x = [1u64, 3, 5];
        let y = [2u64, 3, 4];
        let win = [1u64, 3, 5];
        let mut wout = [0u64; 16];
        let mut w_size = 16usize;
        let mut dout = [0u64; 16];
        let mut d_size = 16usize;

        let result = rs_intersect_mov_test(
            x.as_ptr(),
            x.len(),
            y.as_ptr(),
            y.len(),
            win.as_ptr(),
            win.len(),
            wout.as_mut_ptr(),
            &raw mut w_size,
            dout.as_mut_ptr(),
            &raw mut d_size,
        );

        assert!(result);
    }
}
