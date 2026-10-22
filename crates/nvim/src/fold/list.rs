//! Finding a fold in the tree.
//!
//! Every fold list in a window — `w_folds`, and the `nested` list of every
//! fold in it — is a `Vec<Fold>` sorted by `top`, whose entries never
//! overlap. [`find`] is the binary search every walk in `fold/` starts with,
//! and [`list_at`]/[`fold_on_line`] re-derive a list or a fold from the root
//! when a walk had to let go of the tree.
//!
//! A walk lets go whenever it is about to run something that can reach the
//! tree itself — 'foldexpr', the syntax and diff level getters, a screen-line
//! count — and so the walks hold *positions* (indices, depths, line numbers)
//! rather than references, and find their place again afterwards.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::Fold;
use crate::types::LineNr;

/// Search `folds` for line `lnum`.
///
/// `Ok(i)` is the fold that contains `lnum`; `Err(i)` is the first fold
/// *below* it, which may be `folds.len()` — there is a gap at `lnum`, or
/// `lnum` is past the last fold.
///
/// The search is written out rather than handed to `slice::binary_search_by`
/// so that a malformed list — a zero-length entry, say — lands on exactly the
/// entry `fold.c`'s loop would have landed on. `rest` is upstream's
/// `low..=high` and `base` its `low`; the probe `(low + high) / 2` is `base`
/// plus `(rest.len() - 1) / 2`, which the slice keeps in bounds without a
/// check.
#[inline]
pub(super) fn find(folds: &[Fold], lnum: LineNr) -> Result<usize, usize> {
    let mut base = 0;
    let mut rest = folds;
    while !rest.is_empty() {
        let mid = (rest.len() - 1) / 2;
        let fold = &rest[mid];
        if fold.top > lnum {
            rest = &rest[..mid];
        } else if fold.top + fold.len <= lnum {
            base += mid + 1;
            rest = &rest[mid + 1..];
        } else {
            return Ok(base + mid);
        }
    }
    Err(base)
}

/// [`find`]'s answer whichever way it went: the fold containing `lnum`, or
/// the first one below it.
pub(super) fn find_at_or_below(folds: &[Fold], lnum: LineNr) -> usize {
    match find(folds, lnum) {
        Ok(i) | Err(i) => i,
    }
}

/// The fold list `path` names under `root`: each step is the index of a fold
/// in the list above, whose nested list is the next one. `None` once the tree
/// no longer has that shape.
pub(super) fn list_at<'a>(root: &'a mut Vec<Fold>, path: &[usize]) -> Option<&'a mut Vec<Fold>> {
    let mut list = root;
    for &i in path {
        list = &mut list.get_mut(i)?.nested;
    }
    Some(list)
}

/// The fold list at nesting depth `depth` (0 is the window's toplevel) on
/// the way down to line `lnum`, and `lnum` relative to it.
///
/// The walks that let go of the tree come back this way: they only ever
/// descend through folds that contain their line, so the line and the depth
/// name the list (and [`find`] the fold in it), as long as nothing changed
/// the tree's shape in between.
pub(super) fn list_on_line(
    root: &mut Vec<Fold>,
    lnum: LineNr,
    depth: usize,
) -> Option<(&mut Vec<Fold>, LineNr)> {
    let mut folds = root;
    let mut lnum_rel = lnum;
    for _ in 0..depth {
        let i = find(folds, lnum_rel).ok()?;
        let fold = &mut folds[i];
        lnum_rel -= fold.top;
        folds = &mut fold.nested;
    }
    Some((folds, lnum_rel))
}

/// The fold containing `lnum` at nesting depth `depth`: [`list_on_line`]'s
/// list, searched.
pub(super) fn fold_on_line(root: &mut Vec<Fold>, lnum: LineNr, depth: usize) -> Option<&mut Fold> {
    let (folds, lnum_rel) = list_on_line(root, lnum, depth)?;
    let i = find(folds, lnum_rel).ok()?;
    folds.get_mut(i)
}

#[cfg(test)]
mod tests {
    use super::super::FoldState;
    use super::*;

    /// A detached fold list from `(top, len)` pairs.
    fn list(spans: &[(LineNr, LineNr)]) -> Vec<Fold> {
        spans
            .iter()
            .map(|&(top, len)| Fold {
                top,
                len,
                nested: Vec::new(),
                state: FoldState::Open,
                small: None,
            })
            .collect()
    }

    #[test]
    fn find_lands_on_the_fold_that_contains_the_line() {
        let folds = list(&[(2, 3), (10, 1), (20, 5)]);
        assert_eq!(find(&folds, 2), Ok(0));
        assert_eq!(find(&folds, 4), Ok(0));
        assert_eq!(find(&folds, 10), Ok(1));
        assert_eq!(find(&folds, 24), Ok(2));
    }

    #[test]
    fn a_miss_names_the_first_fold_below_the_line() {
        let folds = list(&[(2, 3), (10, 1), (20, 5)]);
        // Above everything, in the gaps, and past the end.
        assert_eq!(find(&folds, 1), Err(0));
        assert_eq!(find(&folds, 5), Err(1));
        assert_eq!(find(&folds, 11), Err(2));
        assert_eq!(find(&folds, 25), Err(3));
        assert_eq!(find(&folds, 9_999), Err(3));
        assert_eq!(find_at_or_below(&folds, 11), 2);
    }

    #[test]
    fn an_empty_list_misses_at_zero() {
        assert_eq!(find(&[], 1), Err(0));
    }

    #[test]
    fn a_zero_length_entry_is_stepped_over_as_upstream_steps() {
        // `fold.c`'s loop on `[(5, 0), (5, 3)]` probes index 0 first, finds
        // line 5 past its (empty) end, and moves right.
        let folds = list(&[(5, 0), (5, 3)]);
        assert_eq!(find(&folds, 5), Ok(1));
        assert_eq!(find(&folds, 4), Err(0));
    }

    #[test]
    fn a_path_names_a_nested_list_until_the_tree_changes() {
        let mut root = list(&[(2, 10), (20, 5)]);
        root[0].nested = list(&[(1, 3), (6, 2)]);
        root[0].nested[1].nested = list(&[(0, 1)]);
        assert_eq!(list_at(&mut root, &[]).map(|l| l.len()), Some(2));
        assert_eq!(list_at(&mut root, &[0]).map(|l| l.len()), Some(2));
        assert_eq!(list_at(&mut root, &[0, 1]).map(|l| l[0].len), Some(1));
        assert!(list_at(&mut root, &[1]).is_some_and(|l| l.is_empty()));
        assert!(list_at(&mut root, &[1, 0]).is_none());
        root.clear();
        assert!(list_at(&mut root, &[0]).is_none());
    }

    #[test]
    fn a_line_and_a_depth_name_a_fold() {
        let mut root = list(&[(2, 10), (20, 5)]);
        root[0].nested = list(&[(1, 3), (6, 2)]);
        // Line 9 is 7 into the first fold: inside its second child.
        assert_eq!(fold_on_line(&mut root, 9, 0).map(|f| f.top), Some(2));
        assert_eq!(fold_on_line(&mut root, 9, 1).map(|f| f.top), Some(6));
        assert!(fold_on_line(&mut root, 9, 2).is_none());
        assert!(fold_on_line(&mut root, 15, 0).is_none());
    }
}
