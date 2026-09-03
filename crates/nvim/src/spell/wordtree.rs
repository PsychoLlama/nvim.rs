//! The flattened word tries a `.spl` file stores, and the walk over them.
//!
//! A language keeps up to four of these: case-folded words, keep-case
//! words, postponed prefixes, and — once a `.sug` file has been read —
//! sound-folded forms.
//!
//! # Layout
//!
//! Both arrays are indexed together. A node starts at some index `n`:
//! `byts[n]` is how many children follow, and `byts[n + 1 ..= n + count]`
//! are their bytes in ascending order. `idxs[n + i]` is where the child at
//! `byts[n + i]` continues.
//!
//! A child byte of zero is not a character: it means "a word ends here",
//! and the `idxs` entry beside it holds that word's `WF_*` flags and region
//! mask instead of a node index. A node can carry several of those in a
//! row, one per flag/region combination the same spelling has, and they
//! sort before every real byte, so they are always the node's first
//! children.
//!
//! # Bounds
//!
//! Every index below is checked by Rust. That is affordable because the
//! reader (`spellfile::read::read_tree_node`) has already refused a tree
//! whose node lengths run past the array: it rejects a node with no
//! children and one whose last child would land at or past the end, so
//! after reading a count at `n` the byte at `n + 1` is always there. A
//! panic here would therefore mean the reader let a malformed tree
//! through, not that a caller mis-stepped.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::types::idx_T;

/// One language tree: the child bytes and the indices beside them.
///
/// An empty tree is the language not having that tree at all, which is
/// what a null `byts` used to say; [`WordTree::is_empty`] is the test.
#[derive(Default)]
pub struct WordTree {
    byts: Box<[u8]>,
    idxs: Box<[idx_T]>,
}

impl WordTree {
    /// Adopt the two arrays a reader filled in. They must be the same
    /// length, which is what `spell_read_tree` allocates them as.
    pub(crate) fn from_parts(byts: Box<[u8]>, idxs: Box<[idx_T]>) -> WordTree {
        debug_assert_eq!(byts.len(), idxs.len());
        WordTree { byts, idxs }
    }

    /// Whether the language has this tree at all.
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.byts.is_empty()
    }

    /// How many entries the tree has, counting nodes and children alike.
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.byts.len()
    }

    /// The byte at `i`: a node's child count when `i` is a node start, one
    /// of its children's bytes otherwise.
    #[inline]
    pub(crate) fn byte(&self, i: usize) -> u8 {
        self.byts[i]
    }

    /// The index beside the byte at `i`: where that child continues, or —
    /// when the byte is zero — the word's flags.
    #[inline]
    pub(crate) fn idx(&self, i: usize) -> idx_T {
        self.idxs[i]
    }

    /// How many children the node starting at `n` has.
    #[inline]
    pub(crate) fn node_len(&self, n: usize) -> usize {
        usize::from(self.byts[n])
    }

    /// Whether the child at `i` is a "word ends here" entry.
    #[inline]
    pub(crate) fn ends_word(&self, i: usize) -> bool {
        self.byts[i] == 0
    }

    /// How many of the `len` children from `first` are word ends. They are
    /// the node's first children, so this is where the real bytes start.
    #[inline]
    pub(crate) fn word_ends(&self, first: usize, len: usize) -> usize {
        self.byts[first..first + len]
            .iter()
            .take_while(|&&b| b == 0)
            .count()
    }

    /// Find the child of this node whose byte is `c`, among the `len`
    /// children starting at `first`, and answer where it sits.
    ///
    /// The children are sorted, so this is a binary search — the same one
    /// the C did by hand, and the reason a tree lookup is logarithmic in
    /// the alphabet rather than linear.
    #[inline]
    pub(crate) fn child(&self, first: usize, len: usize, c: u8) -> Option<usize> {
        self.byts[first..first + len]
            .binary_search(&c)
            .ok()
            .map(|at| first + at)
    }

    /// The two arrays as slices, for a decoder that walks them itself.
    /// The unit suite reads a loaded tree back this way, deliberately
    /// without going through the lookup.
    #[inline]
    pub fn as_slices(&self) -> (&[u8], &[idx_T]) {
        (&self.byts, &self.idxs)
    }

    /// Where the child at `i` continues. Only meaningful when the byte
    /// there is not zero: a word end's entry holds flags, not an index.
    #[inline]
    pub(crate) fn child_node(&self, i: usize) -> usize {
        // A tree the reader accepted never stores a negative index.
        usize::try_from(self.idxs[i]).unwrap_or(0)
    }

    /// The index array, mutable: `tree_count_words` rewrites every word
    /// end's entry with the number of words below it.
    #[inline]
    pub(crate) fn idxs_mut(&mut self) -> &mut [idx_T] {
        &mut self.idxs
    }
}
