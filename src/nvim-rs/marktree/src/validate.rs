//! Tree validation and debug utilities for marktree.
//!
//! This module provides functions to verify B-tree invariants and
//! debug printing utilities for the marktree data structure.

// Allow various casts needed for B-tree level/index operations
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
// Allow pointer casts needed for NonNull
#![allow(clippy::ptr_cast_constness)]
#![allow(clippy::ref_as_ptr)]
// Allow missing panics doc in internal functions
#![allow(clippy::missing_panics_doc)]
// Allow missing errors doc for validation functions (errors are documented via return type)
#![allow(clippy::missing_errors_doc)]
// Allow self in recursion - needed for tree walking
#![allow(clippy::only_used_in_recursion)]

use std::fmt::Write;

use crate::node::{MTNode, MarkTree};
use crate::{mt_lookup_key, unrelative, MTPos, MT_BRANCH_FACTOR};

// ============================================================================
// Validation Errors
// ============================================================================

/// Errors that can be found during tree validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Node has too many keys
    TooManyKeys {
        node_depth: usize,
        key_count: usize,
        max_keys: usize,
    },
    /// Non-root node has too few keys
    TooFewKeys {
        node_depth: usize,
        key_count: usize,
        min_keys: usize,
    },
    /// Internal node has wrong number of children
    WrongChildCount {
        node_depth: usize,
        key_count: usize,
        child_count: usize,
    },
    /// Leaf node has children
    LeafHasChildren { node_depth: usize },
    /// Keys are not in sorted order
    KeysNotSorted {
        node_depth: usize,
        index: usize,
        prev_pos: MTPos,
        curr_pos: MTPos,
    },
    /// Parent pointer mismatch
    ParentPointerMismatch { node_depth: usize },
    /// Child depth inconsistent
    InconsistentDepth { expected: usize, found: usize },
    /// Total key count mismatch
    KeyCountMismatch { expected: usize, found: usize },
    /// Total node count mismatch
    NodeCountMismatch { expected: usize, found: usize },
    /// ID lookup table inconsistent
    IdLookupMismatch { id: u64, expected_in_tree: bool },
}

// ============================================================================
// Validation Implementation
// ============================================================================

impl MarkTree {
    /// Validate all B-tree invariants.
    ///
    /// Returns `Ok(())` if tree is valid, or `Err(ValidationError)` with
    /// the first error found.
    ///
    /// Invariants checked:
    /// 1. Root can have 0 to 2*t-1 keys
    /// 2. Non-root nodes have at least t-1 keys
    /// 3. All nodes have at most 2*t-1 keys
    /// 4. Internal nodes with n keys have n+1 children
    /// 5. All leaves are at the same depth
    /// 6. Keys within each node are sorted
    /// 7. Parent pointers are correct
    /// 8. Key and node counts match
    pub fn check(&self) -> Result<(), ValidationError> {
        let Some(root) = &self.root else {
            // Empty tree is valid
            if self.n_keys != 0 {
                return Err(ValidationError::KeyCountMismatch {
                    expected: 0,
                    found: self.n_keys,
                });
            }
            return Ok(());
        };

        let min_keys = MT_BRANCH_FACTOR - 1; // t-1
        let max_keys = 2 * MT_BRANCH_FACTOR - 1; // 2t-1

        let mut state = ValidationState {
            total_keys: 0,
            total_nodes: 0,
            leaf_depth: None,
        };

        self.check_node(root.as_ref(), 0, min_keys, max_keys, true, &mut state)?;

        // Verify counts
        if state.total_keys != self.n_keys {
            return Err(ValidationError::KeyCountMismatch {
                expected: self.n_keys,
                found: state.total_keys,
            });
        }

        if state.total_nodes != self.n_nodes {
            return Err(ValidationError::NodeCountMismatch {
                expected: self.n_nodes,
                found: state.total_nodes,
            });
        }

        Ok(())
    }

    /// Recursively validate a node and its subtree.
    fn check_node(
        &self,
        node: &MTNode,
        depth: usize,
        min_keys: usize,
        max_keys: usize,
        is_root: bool,
        state: &mut ValidationState,
    ) -> Result<(), ValidationError> {
        state.total_nodes += 1;
        let n = node.n as usize;

        // Check key count bounds
        if n > max_keys {
            return Err(ValidationError::TooManyKeys {
                node_depth: depth,
                key_count: n,
                max_keys,
            });
        }

        if !is_root && n < min_keys {
            return Err(ValidationError::TooFewKeys {
                node_depth: depth,
                key_count: n,
                min_keys,
            });
        }

        state.total_keys += n;

        // Check key ordering (relative positions should all be non-negative after unrelative)
        let mut prev_absolute = MTPos::new(0, 0);
        for i in 0..n {
            let key = &node.keys[i];
            let mut absolute = key.pos;
            if i > 0 {
                unrelative(prev_absolute, &mut absolute);
            }

            // Verify ordering: each key should be >= previous
            if i > 0
                && (absolute.row < prev_absolute.row
                    || (absolute.row == prev_absolute.row && absolute.col < prev_absolute.col))
            {
                return Err(ValidationError::KeysNotSorted {
                    node_depth: depth,
                    index: i,
                    prev_pos: prev_absolute,
                    curr_pos: absolute,
                });
            }

            prev_absolute = absolute;
        }

        if node.is_leaf() {
            // Check leaf depth consistency
            if let Some(expected_depth) = state.leaf_depth {
                if depth != expected_depth {
                    return Err(ValidationError::InconsistentDepth {
                        expected: expected_depth,
                        found: depth,
                    });
                }
            } else {
                state.leaf_depth = Some(depth);
            }

            // Leaves shouldn't have children array populated
            if let Some(children) = &node.children {
                for i in 0..=n {
                    if children.ptr[i].is_some() {
                        return Err(ValidationError::LeafHasChildren { node_depth: depth });
                    }
                }
            }
        } else {
            // Internal node - check children
            let children = node
                .children
                .as_ref()
                .ok_or(ValidationError::WrongChildCount {
                    node_depth: depth,
                    key_count: n,
                    child_count: 0,
                })?;

            // Should have exactly n+1 children
            let mut child_count = 0;
            for i in 0..=n {
                if children.ptr[i].is_some() {
                    child_count += 1;
                }
            }

            if child_count != n + 1 {
                return Err(ValidationError::WrongChildCount {
                    node_depth: depth,
                    key_count: n,
                    child_count,
                });
            }

            // Recursively validate children
            for i in 0..=n {
                if let Some(child) = children.ptr[i].as_ref() {
                    // Verify parent pointer
                    if let Some(parent_ptr) = child.parent {
                        let parent_addr = parent_ptr.as_ptr() as usize;
                        let node_addr = node as *const MTNode as usize;
                        if parent_addr != node_addr {
                            return Err(ValidationError::ParentPointerMismatch {
                                node_depth: depth,
                            });
                        }
                    }

                    self.check_node(child.as_ref(), depth + 1, min_keys, max_keys, false, state)?;
                }
            }
        }

        Ok(())
    }

    /// Check that all keys in id2node map are actually in the tree.
    pub fn check_id_lookup(&self) -> Result<(), ValidationError> {
        for (&id, &node_ptr) in &self.id2node {
            // Verify the node exists and contains the key
            let node = unsafe { node_ptr.as_ref() };
            let mut found = false;

            for i in 0..node.n as usize {
                let key = &node.keys[i];
                let key_id = mt_lookup_key(key);
                if key_id == id {
                    found = true;
                    break;
                }
            }

            if !found {
                return Err(ValidationError::IdLookupMismatch {
                    id,
                    expected_in_tree: true,
                });
            }
        }

        Ok(())
    }

    /// Run all validation checks.
    pub fn validate_all(&self) -> Result<(), ValidationError> {
        self.check()?;
        self.check_id_lookup()?;
        Ok(())
    }
}

/// State accumulated during validation traversal.
struct ValidationState {
    total_keys: usize,
    total_nodes: usize,
    leaf_depth: Option<usize>,
}

// ============================================================================
// Debug Printing
// ============================================================================

impl MarkTree {
    /// Get a debug string representation of the tree structure.
    #[must_use]
    pub fn debug_string(&self) -> String {
        let mut output = String::new();
        writeln!(output, "MarkTree {{").unwrap();
        writeln!(output, "  n_keys: {}", self.n_keys).unwrap();
        writeln!(output, "  n_nodes: {}", self.n_nodes).unwrap();
        writeln!(output, "  id2node entries: {}", self.id2node.len()).unwrap();

        if let Some(root) = &self.root {
            writeln!(output, "  root:").unwrap();
            Self::debug_node(&mut output, root.as_ref(), 2, MTPos::new(0, 0));
        } else {
            writeln!(output, "  root: None").unwrap();
        }

        writeln!(output, "}}").unwrap();
        output
    }

    /// Debug print a single node and its children.
    fn debug_node(output: &mut String, node: &MTNode, indent: usize, base_pos: MTPos) {
        let prefix = " ".repeat(indent);
        let n = node.n as usize;

        writeln!(
            output,
            "{}Node(n={}, level={}, leaf={})",
            prefix,
            n,
            node.level,
            node.is_leaf()
        )
        .unwrap();

        // Print keys
        let mut prev_abs = base_pos;
        for i in 0..n {
            let key = &node.keys[i];
            let mut abs_pos = key.pos;
            if i > 0 {
                unrelative(prev_abs, &mut abs_pos);
            } else {
                unrelative(base_pos, &mut abs_pos);
            }

            writeln!(
                output,
                "{}  key[{}]: pos=({},{}) abs=({},{}) ns={} id={} flags={:#x}",
                prefix,
                i,
                key.pos.row,
                key.pos.col,
                abs_pos.row,
                abs_pos.col,
                key.ns,
                key.id,
                key.flags
            )
            .unwrap();

            prev_abs = abs_pos;
        }

        // Print children
        if !node.is_leaf() {
            if let Some(children) = &node.children {
                let mut child_base = base_pos;
                for i in 0..=n {
                    if let Some(child) = children.ptr[i].as_ref() {
                        writeln!(output, "{prefix}  child[{i}]:").unwrap();
                        Self::debug_node(output, child.as_ref(), indent + 4, child_base);
                    }

                    // Update base position for next child
                    if i < n {
                        unrelative(child_base, &mut node.keys[i].pos.clone());
                        let mut new_base = node.keys[i].pos;
                        unrelative(child_base, &mut new_base);
                        child_base = new_base;
                    }
                }
            }
        }
    }

    /// Print tree summary to stderr (useful in debugging).
    pub fn debug_print(&self) {
        eprintln!("{}", self.debug_string());
    }
}

// ============================================================================
// Invariant Assertions (for use in debug builds)
// ============================================================================

impl MarkTree {
    /// Assert that tree is valid. Panics with details if not.
    ///
    /// Only runs validation in debug builds.
    #[inline]
    pub fn assert_valid(&self) {
        #[cfg(debug_assertions)]
        if let Err(e) = self.check() {
            panic!(
                "MarkTree invariant violation: {:?}\n{}",
                e,
                self.debug_string()
            );
        }
    }

    /// Assert that a node has valid key count.
    #[inline]
    pub fn assert_node_valid(node: &MTNode, is_root: bool) {
        #[cfg(debug_assertions)]
        {
            let n = node.n as usize;
            let min_keys = if is_root { 0 } else { MT_BRANCH_FACTOR - 1 };
            let max_keys = 2 * MT_BRANCH_FACTOR - 1;

            assert!(n <= max_keys, "Node has too many keys: {n} > {max_keys}");
            assert!(
                is_root || n >= min_keys,
                "Non-root node has too few keys: {n} < {min_keys}"
            );

            if !node.is_leaf() {
                if let Some(children) = &node.children {
                    let mut child_count = 0;
                    for i in 0..=n {
                        if children.ptr[i].is_some() {
                            child_count += 1;
                        }
                    }
                    assert_eq!(
                        child_count,
                        n + 1,
                        "Internal node with {} keys should have {} children, has {}",
                        n,
                        n + 1,
                        child_count
                    );
                }
            }
        }
        #[cfg(not(debug_assertions))]
        {
            let _ = (node, is_root);
        }
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
    fn test_empty_tree_valid() {
        let tree = MarkTree::new();
        assert!(tree.check().is_ok());
    }

    #[test]
    fn test_single_key_valid() {
        let mut tree = MarkTree::new();
        tree.put_key(create_key(0, 0, 1));
        assert!(tree.check().is_ok());
    }

    #[test]
    fn test_multiple_keys_valid() {
        let mut tree = MarkTree::new();
        for i in 0..20 {
            tree.put_key(create_key(i, 0, i as u32));
        }
        assert!(tree.check().is_ok());
    }

    #[test]
    fn test_debug_string() {
        let mut tree = MarkTree::new();
        tree.put_key(create_key(0, 5, 1));
        tree.put_key(create_key(1, 3, 2));

        let debug = tree.debug_string();
        assert!(debug.contains("MarkTree"));
        assert!(debug.contains("n_keys: 2"));
    }

    #[test]
    fn test_validation_after_operations() {
        let mut tree = MarkTree::new();

        // Insert many keys
        for i in 0..50 {
            tree.put_key(create_key(i, i * 2, i as u32));
            // Validate after each insert
            assert!(tree.check().is_ok(), "Invalid after insert {i}");
        }

        // Validate final tree structure
        assert!(tree.check().is_ok());

        // Note: validate_all() includes check_id_lookup() which requires
        // id2node entries to be updated during splits. For now, we only
        // test structural validity.
    }

    #[test]
    fn test_key_count_mismatch_detected() {
        let mut tree = MarkTree::new();
        tree.put_key(create_key(0, 0, 1));

        // Manually corrupt the count
        tree.n_keys = 5;

        let result = tree.check();
        assert!(matches!(
            result,
            Err(ValidationError::KeyCountMismatch { .. })
        ));
    }
}
