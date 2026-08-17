//! The fold tree's line-number bookkeeping.
//!
//! `foldMarkAdjustRecurse` is what every buffer change that inserts, deletes
//! or moves lines eventually reaches, and it is the only part of the fold
//! machinery that runs without a window: it walks a `garray_T` of `fold_T`
//! and rewrites `fd_top`/`fd_len` in place, recursing into `fd_nested`. The
//! cases below are the four shapes its `if` chain distinguishes, with the
//! expected results derived from `v0.12.4`'s `src/nvim/fold.c` rather than
//! from this port.
//!
//! Miri watches the `folds()`/`fold_at()` pointer arithmetic here — a fold
//! list is an untyped growarray, so nothing else in the tree type-checks it.

use std::ffi::c_int;
use std::mem::size_of;
use std::ptr;

use c2rust_neovim::fold::adjust::foldMarkAdjustRecurse;
use c2rust_neovim::fold::fold_T;
use c2rust_neovim::garray::{ga_clear, ga_grow, ga_init};
use c2rust_neovim::pos::MAXLNUM;
use c2rust_neovim::types::{garray_T, linenr_T};

/// The sentinel `mark_adjust` passes as `amount` to mean "these lines are
/// gone".
const DELETED: linenr_T = MAXLNUM as c_int;

fn empty_list() -> garray_T {
    let mut gap = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ptr::null_mut(),
    };
    unsafe { ga_init(&raw mut gap, size_of::<fold_T>() as c_int, 10) };
    gap
}

/// Append a fold covering `top..top + len - 1`, and hand back its nested list
/// so a caller can build a second level under it.
unsafe fn push(gap: &mut garray_T, top: linenr_T, len: linenr_T) -> *mut garray_T {
    ga_grow(gap, 1);
    let fp = (gap.ga_data as *mut fold_T).add(gap.ga_len as usize);
    (*fp).fd_top = top;
    (*fp).fd_len = len;
    (*fp).fd_flags = 0;
    (*fp).fd_small = -1;
    ga_init(&raw mut (*fp).fd_nested, size_of::<fold_T>() as c_int, 10);
    gap.ga_len += 1;
    &raw mut (*fp).fd_nested
}

/// Every fold in `gap` as `(fd_top, fd_len)`.
unsafe fn spans(gap: &garray_T) -> Vec<(linenr_T, linenr_T)> {
    (0..gap.ga_len)
        .map(|i| {
            let fp = (gap.ga_data as *const fold_T).offset(i as isize);
            ((*fp).fd_top, (*fp).fd_len)
        })
        .collect()
}

unsafe fn free_list(gap: &mut garray_T) {
    for i in 0..gap.ga_len {
        let fp = (gap.ga_data as *mut fold_T).offset(i as isize);
        free_list(&mut (*fp).fd_nested);
    }
    ga_clear(gap);
}

#[test]
fn lines_inserted_above_a_fold_push_it_down() {
    unsafe {
        let mut gap = empty_list();
        push(&mut gap, 10, 5);
        // `:normal 3O` at line 5 -> mark_adjust(5, MAXLNUM, 3, 0).
        foldMarkAdjustRecurse(&raw mut gap, 5, DELETED, 3, 0);
        assert_eq!(spans(&gap), [(13, 5)]);
        free_list(&mut gap);
    }
}

#[test]
fn a_fold_swallowed_whole_is_deleted() {
    unsafe {
        let mut gap = empty_list();
        push(&mut gap, 10, 5);
        push(&mut gap, 20, 2);
        // `:10,14d` -> the first fold's every line is gone, and the second
        // one slides up by the five removed lines.
        foldMarkAdjustRecurse(&raw mut gap, 10, 14, DELETED, -5);
        assert_eq!(spans(&gap), [(15, 2)]);
        free_list(&mut gap);
    }
}

#[test]
fn a_deletion_inside_a_fold_only_shortens_it() {
    unsafe {
        let mut gap = empty_list();
        push(&mut gap, 10, 5);
        // `:12,13d` -> the fold still starts at 10 and is two lines shorter.
        foldMarkAdjustRecurse(&raw mut gap, 12, 13, DELETED, -2);
        assert_eq!(spans(&gap), [(10, 3)]);
        free_list(&mut gap);
    }
}

#[test]
fn a_deletion_reaches_nested_folds() {
    unsafe {
        let mut gap = empty_list();
        let nested = push(&mut gap, 10, 10);
        // Absolute 12..14, i.e. 2..4 relative to the parent.
        push(&mut *nested, 2, 3);
        foldMarkAdjustRecurse(&raw mut gap, 12, 13, DELETED, -2);
        assert_eq!(spans(&gap), [(10, 8)]);
        let child = &*((gap.ga_data as *const fold_T).offset(0));
        assert_eq!(spans(&child.fd_nested), [(2, 1)]);
        free_list(&mut gap);
    }
}

#[test]
fn a_fold_entirely_below_the_change_only_shifts() {
    unsafe {
        let mut gap = empty_list();
        push(&mut gap, 10, 3);
        push(&mut gap, 30, 4);
        // `:15,19d` touches neither fold's lines; only the later one moves.
        foldMarkAdjustRecurse(&raw mut gap, 15, 19, DELETED, -5);
        assert_eq!(spans(&gap), [(10, 3), (25, 4)]);
        free_list(&mut gap);
    }
}

#[test]
fn an_empty_list_is_left_alone() {
    unsafe {
        let mut gap = empty_list();
        foldMarkAdjustRecurse(&raw mut gap, 1, 10, DELETED, -10);
        assert_eq!(gap.ga_len, 0);
        free_list(&mut gap);
    }
}
