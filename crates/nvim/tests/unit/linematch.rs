//! End-to-end checks of the diff linematch alignment. Pure logic: no
//! editor state, no FFI, so it runs under Miri too.

use std::ffi::c_int;

use c2rust_neovim::linematch::linematch_nbuffers;

/// Both buffers advance together, one line at a time.
const BOTH: c_int = 0b11;
/// Only the first buffer advances: its line has no counterpart.
const FIRST: c_int = 0b01;
/// Only the second buffer advances.
const SECOND: c_int = 0b10;

#[test]
fn identical_blocks_pair_up_line_by_line() {
    let block = &b"alpha\nbeta\n"[..];
    assert_eq!(
        linematch_nbuffers(&[block, block], &[2, 2], false),
        vec![BOTH, BOTH]
    );
}

#[test]
fn an_inserted_line_is_left_unpaired() {
    let a = &b"alpha\ngamma\n"[..];
    let b = &b"alpha\nbeta\ngamma\n"[..];
    assert_eq!(
        linematch_nbuffers(&[a, b], &[2, 3], false),
        vec![BOTH, SECOND, BOTH]
    );
}

#[test]
fn equal_scores_break_toward_the_fewest_decision_changes() {
    // Nothing in common, so pairing the two lines scores no better than
    // showing them separately — but it takes one decision instead of two.
    let a = &b"aaaa\n"[..];
    let b = &b"bbbb\n"[..];
    assert_eq!(linematch_nbuffers(&[a, b], &[1, 1], false), vec![BOTH]);
}

#[test]
fn the_matching_line_is_paired_across_a_reorder() {
    let a = &b"xxx\nkeep\n"[..];
    let b = &b"keep\nyyy\n"[..];
    assert_eq!(
        linematch_nbuffers(&[a, b], &[2, 2], false),
        vec![FIRST, BOTH, SECOND]
    );
}

#[test]
fn an_empty_side_consumes_the_other_alone() {
    let a = &b"alpha\nbeta\n"[..];
    assert_eq!(
        linematch_nbuffers(&[a, &[]], &[2, 0], false),
        vec![FIRST, FIRST]
    );
    assert!(linematch_nbuffers(&[&[], &[]], &[0, 0], false).is_empty());
}

#[test]
fn ignoring_whitespace_pairs_reindented_lines() {
    let a = &b"if x:\n    return\nend\n"[..];
    let b = &b"if x:\n\t\t\treturn\nend\n"[..];
    assert_eq!(
        linematch_nbuffers(&[a, b], &[3, 3], true),
        vec![BOTH, BOTH, BOTH]
    );
}

#[test]
fn three_buffers_align_on_the_shared_line() {
    let a = &b"shared\n"[..];
    let b = &b"shared\n"[..];
    let c = &b"shared\n"[..];
    assert_eq!(
        linematch_nbuffers(&[a, b, c], &[1, 1, 1], false),
        vec![0b111]
    );
}
