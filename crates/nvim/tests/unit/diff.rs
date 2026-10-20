//! Diff mode's block list: what an external diff's text becomes, and how the
//! hunks of several buffers merge.
//!
//! The oracle the `diff/` retype is checked against. The external diff's
//! output is read back line by line (`diff_read` -> `extract_hunk` ->
//! `parse_diff_ed`/`parse_diff_unified`) and every hunk is merged into the
//! tab page's block chain by `process_hunk`; none of that is reachable
//! except through a diff. So each case sets `'diffexpr'` to a function that
//! answers a fixed text -- the hunks under test, in `ed` or unified form --
//! and reads the blocks back as each line's highlight and filler count. The
//! internal engine's hunks go through the same merge, which the three-buffer
//! cases use.
//!
//! The answers are today's, and upstream 0.12.6 gives the same for every row;
//! the parse quirks are pinned too: a line that is not a header is skipped, a
//! reversed range or an unknown command drops the hunk, the `--- `/`+++ `
//! file header is optional, and a block whose lines are equal on both sides
//! is dropped as it is read.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// `Shape()`: the current tab page's blocks as each window sees them, one
/// word per window: per line `.` (no difference), `A` (added), `C`
/// (changed), `T` (the changed text inside a changed line), and a `-N` after
/// the line when `N` filler lines follow it; `^N` first for filler above
/// line 1.
///
/// `Answer(lines)`: make `'diffexpr'` answer `lines` for the real diff, and
/// the probe `check_external_diff` sends (`line1` vs `line2`) with the `1c1`
/// it wants.
const PRELUDE: &str = r#"
func! Shape()
  let out = []
  for w in range(1, winnr('$'))
    let s = ''
    let f = win_execute(win_getid(w), 'let g:t = diff_filler(1)')
    let s .= g:t ? '^' . g:t : ''
    let n = line('$', win_getid(w))
    for l in range(1, n)
      call win_execute(win_getid(w), 'let g:h = synIDattr(diff_hlID(' . l . ', 1), "name")')
      call win_execute(win_getid(w), 'let g:f = diff_filler(' . (l + 1) . ')')
      let s .= get({'DiffAdd': 'A', 'DiffChange': 'C', 'DiffText': 'T'}, g:h, '.')
      let s .= g:f ? '-' . g:f : ''
    endfor
    call add(out, s)
  endfor
  return join(out, ' ')
endfunc
func! Answer(lines)
  let g:answer = a:lines
  set diffexpr=Expr()
endfunc
func! Expr()
  if readfile(v:fname_in) == ['line1']
    call writefile(['1c1'], v:fname_out)
  else
    call writefile(g:answer, v:fname_out)
  endif
endfunc
func! Two(a, b)
  silent! diffoff!
  silent! only!
  enew!
  call setline(1, a:a)
  diffthis
  vnew
  call setline(1, a:b)
  diffthis
endfunc
"#;

fn setup(sandbox: &Sandbox, fx: &Probe<'_>) {
    fx.run("silent! diffoff!");
    fx.run("set diffexpr& diffopt&");
    fx.home();
    sandbox.write("prelude.vim", PRELUDE.as_bytes());
    fx.run("source prelude.vim");
    fx.run("set diffopt=filler");
    fx.run("set nofoldenable");
}

fn teardown(fx: &Probe<'_>) {
    fx.run("silent! diffoff!");
    fx.run("set diffexpr& diffopt& foldenable&");
    fx.run("silent! delfunction Shape | delfunction Answer | delfunction Expr");
    fx.run("silent! delfunction Two");
    fx.run("silent! unlet g:answer g:t g:h g:f");
    fx.home();
}

/// The two buffers every two-way case diffs: the new one (window 1, made
/// last by `vnew`) and the original (window 2). Their contents only matter
/// for `T` against `C`; the blocks are the answer's.
const OLD: &str = "['a1', 'a2', 'a3', 'a4', 'a5', 'a6']";
const NEW: &str = "['a1', 'b2', 'a3', 'a4', 'a5', 'a6']";

/// Each row: the external diff's text, and `Shape()` after it. `diff` is run
/// with the original as the first file and the new buffer as the second.
/// Answers the rows that moved, so the caller can tear down first.
fn two_way(fx: &Probe<'_>, rows: &[(&str, &str)]) -> Vec<String> {
    let mut moved = Vec::new();
    for (answer, want) in rows {
        fx.run(&format!("call Answer({answer})"));
        fx.run(&format!("call Two({OLD}, {NEW})"));
        let got = fx.eval("Shape()");
        if got != format!("'{want}'") {
            moved.push(format!("({answer:?}, {got})"));
        }
    }
    moved
}

#[track_caller]
fn none_moved(moved: &[String]) {
    assert!(moved.is_empty(), "rows moved:\n{}", moved.join("\n"));
}

#[test]
fn ed_style_hunks() {
    let sandbox = Sandbox::dir("diff_ed_style");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    let moved = two_way(
        &fx,
        &[
            // No hunks: no differences.
            ("[]", "...... ......"),
            ("['2c2', '< a2', '---', '> b2']", ".T.... .T...."),
            // A range on both sides.
            ("['2,3c2,3']", ".TC... .TC..."),
            // `a`: the lines after line 3 of the original are new.
            ("['3a4,5']", "...AA. ...-2..."),
            // `d`: lines gone from the new side, a filler in their place.
            ("['4,5d3']", "...-2... ...AA."),
            // Two hunks whose lines are equal on both sides: a block that
            // changes nothing is dropped as it is read.
            ("['1c1', '5,6c5,6']", "...... ......"),
            // Lines that do not start with a digit are skipped, whatever
            // they say.
            ("['junk', '> b2', '2c2']", ".T.... .T...."),
            // An unknown command letter drops the hunk ...
            ("['2x2']", "...... ......"),
            // ... and so does a range that runs backwards.
            ("['5,3c5']", "...... ......"),
        ],
    );
    teardown(&fx);
    none_moved(&moved);
}

#[test]
fn unified_hunks() {
    let sandbox = Sandbox::dir("diff_unified");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    let moved = two_way(
        &fx,
        &[
            ("['@@ -2 +2 @@', '-a2', '+b2']", ".T.... .T...."),
            // The file header in front is allowed.
            ("['--- a', '+++ b', '@@ -2,2 +2,2 @@']", ".TC... .TC..."),
            // A zero count adds or deletes *after* the line named.
            ("['@@ -3,0 +4,2 @@']", "...AA. ...-2..."),
            ("['@@ -4,2 +3,0 @@']", "...-2... ...AA."),
            // At the very top: `+0,0` and `-0,0`.
            ("['@@ -0,0 +1,2 @@']", "AA.... ^2......"),
            ("['@@ -1,2 +0,0 @@']", "^2...... AA...."),
            // Two hunks, a non-header line between them, both over equal
            // lines: both dropped.
            (
                "['@@ -1 +1 @@', ' context', '@@ -6 +6 @@']",
                "...... ......",
            ),
            // A header without its `+` half is not a hunk.
            ("['@@ -2 @@']", "...... ......"),
        ],
    );
    teardown(&fx);
    none_moved(&moved);
}

#[test]
fn hunks_of_three_buffers_merge() {
    let sandbox = Sandbox::dir("diff_merge");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    fx.run("set diffopt=internal,filler");
    // The original is the first buffer in diff mode; the second and third
    // are each diffed against it and their hunks merged into one list. A
    // line that differs in any buffer is marked in all three, so the blocks
    // read the same in each window but the first (outside diff mode).
    let cases: &[(&str, &str, &str, &str)] = &[
        // Disjoint hunks stay separate blocks.
        (
            "['1', '2', '3', '4', '5', '6']",
            "['1', 'x', '3', '4', '5', '6']",
            "['1', '2', '3', '4', 'y', '6']",
            ". .T..T. .T..T. .T..T.",
        ),
        // Overlapping hunks: the merged block covers both.
        (
            "['1', '2', '3', '4', '5', '6']",
            "['1', 'x', 'x', '4', '5', '6']",
            "['1', '2', 'y', 'y', '5', '6']",
            ". .TTT.. .TTT.. .TTT..",
        ),
        // Adjacent one-line hunks, one in each buffer.
        (
            "['1', '2', '3', '4', '5', '6']",
            "['1', 'x', '3', '4', '5', '6']",
            "['1', '2', 'y', '4', '5', '6']",
            ". .TT... .TT... .TT...",
        ),
        // An added line in one and a change in another, at the same place:
        // one block, with filler in the two that lack the line.
        (
            "['1', '2', '3']",
            "['1', 'new', '2', '3']",
            "['1', 'z', '3']",
            ". .T-1. .TA. .T-1.",
        ),
    ];
    let mut moved = Vec::new();
    for (orig, second, third, want) in cases {
        fx.run("silent! diffoff! | silent! only! | enew!");
        fx.run(&format!("call setline(1, {orig}) | diffthis"));
        fx.run(&format!("vnew | call setline(1, {second}) | diffthis"));
        fx.run(&format!("vnew | call setline(1, {third}) | diffthis"));
        // A fourth window outside diff mode shows nothing.
        fx.run("vnew | call setline(1, 'x')");
        let got = fx.eval("Shape()");
        if got != format!("'{want}'") {
            moved.push(format!("({orig:?}, {second:?}, {third:?}, {got})"));
        }
    }
    teardown(&fx);
    none_moved(&moved);
}

#[test]
fn the_blocks_follow_edits() {
    let sandbox = Sandbox::dir("diff_edits");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    fx.run("set diffopt=internal,filler");
    fx.run("call Two(['1', '2', '3', '4', '5'], ['1', 'x', '3', '4', '5'])");
    fx.check("Shape()", "'.T... .T...'");
    // Lines added above a block move it down without a new diff.
    fx.run("call append(0, ['p', 'q'])");
    fx.check("Shape()", "'AA.T... ^2.T...'");
    fx.run("diffupdate");
    fx.check("Shape()", "'AA.T... ^2.T...'");
    // Deleting the changed line removes the block once updated.
    fx.run("4delete | diffupdate");
    fx.check("Shape()", "'AA.-1... ^2.A...'");
    // `]c` / `[c` walk the blocks.
    fx.run("call cursor(1, 1) | silent! normal! ]c");
    fx.check("line('.')", "4");
    fx.run("silent! normal! [c");
    fx.check("line('.')", "1");
    teardown(&fx);
}
