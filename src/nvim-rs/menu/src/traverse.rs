//! Menu tree traversal helpers.
//!
//! This module provides utilities for traversing the menu tree structure.
//! Menus form a tree where each menu can have children (submenus) and
//! siblings (next menu at the same level).

use crate::handle::VimMenuHandle;

/// Iterator over sibling menus.
///
/// This iterates through the linked list of menus at the same level,
/// following the `next` pointer.
pub struct MenuSiblingIter {
    current: VimMenuHandle,
}

impl MenuSiblingIter {
    /// Create a new iterator starting at the given menu.
    #[must_use]
    pub const fn new(start: VimMenuHandle) -> Self {
        Self { current: start }
    }
}

impl Iterator for MenuSiblingIter {
    type Item = VimMenuHandle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }
        let menu = self.current;
        self.current = menu.next();
        Some(menu)
    }
}

/// Iterator for depth-first (pre-order) traversal of the menu tree.
///
/// This visits each menu in depth-first order, visiting a node before its
/// children, and children before siblings.
pub struct MenuDepthFirstIter {
    /// Stack of menus to visit.
    stack: Vec<VimMenuHandle>,
}

impl MenuDepthFirstIter {
    /// Create a new depth-first iterator starting at the given menu.
    ///
    /// This will iterate over the menu, all its descendants, and all its
    /// siblings and their descendants.
    #[must_use]
    pub fn new(start: VimMenuHandle) -> Self {
        let mut stack = Vec::new();
        if !start.is_null() {
            stack.push(start);
        }
        Self { stack }
    }

    /// Create a new depth-first iterator that iterates over all siblings
    /// starting at the given menu, collecting them first.
    ///
    /// This differs from `new()` in that siblings are collected upfront
    /// rather than discovered during iteration.
    #[must_use]
    pub fn from_siblings(start: VimMenuHandle) -> Self {
        let mut stack = Vec::new();
        let mut current = start;
        // Push all siblings in reverse order so they come out in correct order
        let mut siblings = Vec::new();
        while !current.is_null() {
            siblings.push(current);
            current = current.next();
        }
        for menu in siblings.into_iter().rev() {
            stack.push(menu);
        }
        Self { stack }
    }
}

impl Iterator for MenuDepthFirstIter {
    type Item = VimMenuHandle;

    fn next(&mut self) -> Option<Self::Item> {
        let menu = self.stack.pop()?;

        // Push siblings first (processed last due to stack LIFO)
        let next_sibling = menu.next();
        if !next_sibling.is_null() {
            self.stack.push(next_sibling);
        }

        // Push children second (processed before siblings)
        let children = menu.children();
        if !children.is_null() {
            self.stack.push(children);
        }

        Some(menu)
    }
}

/// Count the depth of a menu in the tree.
///
/// Returns 0 for root menus, 1 for first-level submenus, etc.
pub fn menu_depth(menu: VimMenuHandle) -> usize {
    let mut depth = 0;
    let mut current = menu.parent();
    while !current.is_null() {
        depth += 1;
        current = current.parent();
    }
    depth
}

/// Check if a menu has any children.
#[inline]
pub fn menu_has_children(menu: VimMenuHandle) -> bool {
    !menu.is_null() && !menu.children().is_null()
}

/// Check if a menu is a leaf (has no children).
#[inline]
pub fn menu_is_leaf(menu: VimMenuHandle) -> bool {
    !menu.is_null() && menu.children().is_null()
}

// Note: Tests that exercise the iterator and depth functions require linking
// with the C library, so they are tested through the full build integration
// tests rather than Rust-only unit tests.
