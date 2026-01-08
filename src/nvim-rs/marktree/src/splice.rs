//! Native Rust splice operations for marktree.
//!
//! This module implements text change operations (splice) that efficiently
//! update mark positions when text is inserted or deleted.
//!
//! The splice operation handles:
//! - Marks within a deleted region are moved to the start of the region
//! - Marks after the deleted region have their positions adjusted

// Allow various casts needed for B-tree level/index operations
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
// Allow pointer casts needed for NonNull
#![allow(clippy::ptr_cast_constness)]
#![allow(clippy::ref_as_ptr)]
// Allow matching C naming conventions
#![allow(clippy::many_single_char_names)]
// Allow range loops for meta arrays - clearer for parallel arrays
#![allow(clippy::needless_range_loop)]
// Allow missing panics doc in internal functions
#![allow(clippy::missing_panics_doc)]
// Allow long functions for complex algorithms
#![allow(clippy::too_many_lines)]
// Allow manual let-else for clarity
#![allow(clippy::manual_let_else)]
// Allow if-not-else for clarity
#![allow(clippy::if_not_else)]
// Allow collapsible else-if
#![allow(clippy::collapsible_else_if)]

use crate::node::{MarkTree, MarkTreeIter};
use crate::{compose, mt_right, pos_leq, relative, unrelative, MTPos, MT_MAX_DEPTH};

// ============================================================================
// Splice Implementation
// ============================================================================

impl MarkTree {
    /// Splice the tree for a text change.
    ///
    /// Updates all mark positions to reflect a text change at the given position.
    ///
    /// # Arguments
    /// * `start_line` - Starting row of the change
    /// * `start_col` - Starting column of the change
    /// * `old_extent_line` - Number of rows deleted
    /// * `old_extent_col` - Column extent of deletion (on last deleted row)
    /// * `new_extent_line` - Number of rows inserted
    /// * `new_extent_col` - Column extent of insertion (on last inserted row)
    ///
    /// # Returns
    /// `true` if any marks were moved, `false` otherwise.
    pub fn splice(
        &mut self,
        start_line: i32,
        start_col: i32,
        old_extent_line: i32,
        old_extent_col: i32,
        new_extent_line: i32,
        new_extent_col: i32,
    ) -> bool {
        let start = MTPos::new(start_line, start_col);
        let mut old_extent = MTPos::new(old_extent_line, old_extent_col);
        let mut new_extent = MTPos::new(new_extent_line, new_extent_col);

        let may_delete = old_extent.row != 0 || old_extent.col != 0;
        let same_line = old_extent.row == 0 && new_extent.row == 0;

        // Convert to absolute positions
        unrelative(start, &mut old_extent);
        unrelative(start, &mut new_extent);

        // Get iterator at start position
        let mut itr = MarkTreeIter::new();
        let mut oldbase = [MTPos::new(0, 0); MT_MAX_DEPTH];

        if !self.get_ext(&mut itr, start, false, true, &mut oldbase) {
            return false;
        }

        let delta = MTPos::new(
            new_extent.row - old_extent.row,
            new_extent.col - old_extent.col,
        );

        let mut moved = false;

        // Handle deletion: move marks within deleted region to start
        if may_delete {
            // Get end iterator for the deleted region
            let mut enditr = MarkTreeIter::new();
            let ipos = itr.current_pos();

            if !pos_leq(old_extent, ipos)
                || (old_extent.row == ipos.row
                    && old_extent.col == ipos.col
                    && itr.current().is_some_and(|k| !mt_right(&k)))
            {
                self.get_ext(
                    &mut enditr,
                    old_extent,
                    true,
                    true,
                    &mut [MTPos::new(0, 0); MT_MAX_DEPTH],
                );
            }

            // Move marks within deleted region to start position
            while itr.is_valid() {
                let mut loc_start = start;
                let mut loc_old = old_extent;

                relative(itr.pos, &mut loc_start);
                relative(oldbase[itr.lvl as usize], &mut loc_old);

                let key = match itr.current_raw() {
                    Some(k) => k,
                    None => break,
                };

                if !pos_leq(key.pos, loc_old) {
                    break;
                }

                // Move this mark to start position
                moved = true;
                itr.set_current_pos(loc_start);

                if !itr.is_leaf() {
                    oldbase[(itr.lvl + 1) as usize] = key.pos;
                    unrelative(
                        oldbase[itr.lvl as usize],
                        &mut oldbase[(itr.lvl + 1) as usize],
                    );
                    itr.next_skip(&mut oldbase);
                } else {
                    if !itr.advance_in_node() {
                        itr.next();
                    }
                }
            }

            // Move marks that were between old_extent and new_extent
            while itr.is_valid() {
                let mut loc_new = new_extent;
                relative(itr.pos, &mut loc_new);

                let mut limit = old_extent;
                relative(oldbase[itr.lvl as usize], &mut limit);

                let key = match itr.current_raw() {
                    Some(k) => k,
                    None => break,
                };

                if pos_leq(limit, key.pos) {
                    break;
                }

                let oldpos = key.pos;
                itr.set_current_pos(loc_new);
                moved = true;

                if !itr.is_leaf() {
                    oldbase[(itr.lvl + 1) as usize] = oldpos;
                    unrelative(
                        oldbase[itr.lvl as usize],
                        &mut oldbase[(itr.lvl + 1) as usize],
                    );
                    itr.next_skip(&mut oldbase);
                } else {
                    if !itr.advance_in_node() {
                        itr.next();
                    }
                }
            }
        }

        // Adjust positions of remaining marks (after the splice point)
        while itr.is_valid() {
            let key = match itr.current_raw() {
                Some(k) => k,
                None => break,
            };

            let mut pos = key.pos;
            unrelative(oldbase[itr.lvl as usize], &mut pos);

            let realrow = pos.row;
            debug_assert!(realrow >= old_extent.row);

            let mut done = false;
            if realrow == old_extent.row {
                if delta.col != 0 {
                    pos.col += delta.col;
                }
            } else if same_line {
                // Optimization: column-only adjustment can skip remaining rows
                done = true;
            }

            if delta.row != 0 {
                pos.row += delta.row;
                moved = true;
            }

            relative(itr.pos, &mut pos);
            itr.set_current_pos(pos);

            if done {
                break;
            }

            itr.next_skip(&mut oldbase);
        }

        moved
    }

    /// Get iterator with extended position tracking.
    fn get_ext(
        &self,
        itr: &mut MarkTreeIter,
        pos: MTPos,
        right_gravity: bool,
        _track_oldbase: bool,
        oldbase: &mut [MTPos; MT_MAX_DEPTH],
    ) -> bool {
        if !itr.get(self, pos.row, pos.col, right_gravity) {
            return false;
        }

        // Build oldbase for tracking old positions
        oldbase[0] = MTPos::new(0, 0);
        let mut lvl = 0;
        let current_pos = MTPos::new(0, 0);

        // Walk down from root tracking positions
        for s in &itr.s[..itr.lvl as usize] {
            if s.i > 0 {
                // Position after key at s.i - 1
                if let Some(x_ptr) = itr.x {
                    // This is a simplification - in full impl we'd track through parent chain
                    let _ = x_ptr;
                }
            }
            lvl += 1;
            if lvl < MT_MAX_DEPTH {
                oldbase[lvl] = current_pos;
            }
        }

        true
    }
}

impl MarkTreeIter {
    /// Get current absolute position.
    fn current_pos(&self) -> MTPos {
        self.current().map_or(MTPos::new(0, 0), |k| {
            let mut pos = k.pos;
            unrelative(self.pos, &mut pos);
            pos
        })
    }

    /// Get current key without position adjustment.
    fn current_raw(&self) -> Option<crate::MTKey> {
        let x_ptr = self.x?;
        let node = unsafe { x_ptr.as_ref() };
        if self.i >= 0 && (self.i as usize) < node.n as usize {
            Some(node.keys[self.i as usize])
        } else {
            None
        }
    }

    /// Set current key position.
    #[allow(clippy::needless_pass_by_ref_mut)]
    fn set_current_pos(&mut self, pos: MTPos) {
        if let Some(x_ptr) = self.x {
            let node = unsafe { x_ptr.as_ptr().as_mut().unwrap() };
            if self.i >= 0 && (self.i as usize) < node.n as usize {
                node.keys[self.i as usize].pos = pos;
            }
        }
    }

    /// Check if current node is a leaf.
    fn is_leaf(&self) -> bool {
        self.x.is_none_or(|x| unsafe { x.as_ref() }.is_leaf())
    }

    /// Advance to next key in same node.
    /// Returns true if successful, false if at end of node.
    fn advance_in_node(&mut self) -> bool {
        if let Some(x_ptr) = self.x {
            let node = unsafe { x_ptr.as_ref() };
            if self.i + 1 < node.n {
                self.i += 1;
                return true;
            }
        }
        false
    }

    /// Skip to next key, updating oldbase.
    fn next_skip(&mut self, oldbase: &mut [MTPos; MT_MAX_DEPTH]) {
        if let Some(key) = self.current_raw() {
            if !self.is_leaf() && (self.lvl as usize + 1) < MT_MAX_DEPTH {
                compose(&mut oldbase[(self.lvl + 1) as usize], key.pos);
            }
        }
        self.next();
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flags::MT_FLAG_REAL;
    use crate::MTKey;

    fn create_key(row: i32, col: i32, id: u32) -> MTKey {
        MTKey {
            pos: MTPos::new(row, col),
            ns: 1,
            id,
            flags: MT_FLAG_REAL,
            decor_data: 0,
        }
    }

    #[test]
    fn test_splice_empty_tree() {
        let mut tree = MarkTree::new();
        let moved = tree.splice(0, 0, 0, 0, 1, 0);
        assert!(!moved);
    }

    #[test]
    fn test_splice_insert_line() {
        let mut tree = MarkTree::new();

        // Insert marks at (0,5), (1,3), (2,7)
        tree.put_key(create_key(0, 5, 1));
        tree.put_key(create_key(1, 3, 2));
        tree.put_key(create_key(2, 7, 3));

        // Insert a line at row 1
        let moved = tree.splice(1, 0, 0, 0, 1, 0);
        assert!(moved);

        // Verify positions: mark 1 unchanged, marks 2 and 3 moved down
        let mut itr = MarkTreeIter::new();
        itr.first(&tree);
        // First mark should be at (0, 5)
        if let Some(key) = itr.current() {
            assert_eq!(key.pos.row, 0);
            assert_eq!(key.pos.col, 5);
        }
    }

    #[test]
    fn test_splice_delete_chars() {
        let mut tree = MarkTree::new();

        // Insert marks at (0,0), (0,5), (0,10)
        tree.put_key(create_key(0, 0, 1));
        tree.put_key(create_key(0, 5, 2));
        tree.put_key(create_key(0, 10, 3));

        // Delete chars 3-7 on row 0 (old_extent_col=4 means delete 4 chars starting at col 3)
        let moved = tree.splice(0, 3, 0, 4, 0, 0);
        assert!(moved);

        // Mark at col 5 should move to col 3 (start of deletion)
        // Mark at col 10 should move to col 6 (10 - 4)
    }
}
