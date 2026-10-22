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
//!
//! The later cases drive the block list the way the editor does: gets and
//! puts across three buffers, updates after edits that split or merge
//! blocks, line matching under the whitespace/case flags, `:move` and undo,
//! the external program against the internal engine, closing windows, the
//! per-line fold level, 'scrollbind' over filler, and user code (a
//! buffer-update callback) running while a get holds a block.

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

/// `Lines(w)`: window `w`'s buffer, joined by `|`.
const LINES: &str = r#"
func! Lines(w)
  return join(getbufline(winbufnr(a:w), 1, '$'), '|')
endfunc
"#;

/// Give the screen a size (the unit editor's window is never drawn, so
/// it has none), answering what to put back.
fn sized(fx: &Probe<'_>) -> String {
    let size = fx.eval("[&lines, &columns, &cmdheight]");
    fx.run("set lines=24 columns=80");
    size
}

/// Put back what [`sized`] answered. Closing the split windows hands
/// their rows to the command line, so 'cmdheight' is part of it.
fn unsize(fx: &Probe<'_>, size: &str) {
    fx.run("redraw!");
    fx.run(&format!("let [&lines, &columns, &cmdheight] = {size}"));
    fx.run("resize");
}

/// Three buffers in diff mode, made in `bufs` order by `vnew`: `bufs[0]`
/// ends in window 3, `bufs[2]` in window 1.
fn three(fx: &Probe<'_>, bufs: [&str; 3]) {
    fx.run("silent! diffoff! | silent! only! | enew!");
    fx.run(&format!("call setline(1, {}) | diffthis", bufs[0]));
    fx.run(&format!("vnew | call setline(1, {}) | diffthis", bufs[1]));
    fx.run(&format!("vnew | call setline(1, {}) | diffthis", bufs[2]));
}

#[test]
fn diffget_and_diffput_across_three_buffers() {
    let sandbox = Sandbox::dir("diff_getput");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    sandbox.write("lines.vim", LINES.as_bytes());
    fx.run("source lines.vim");
    fx.run("set diffopt=internal,filler");
    let base = "['1', '2', '3', '4', '5', '6', '7']";
    let mid = "['1', 'x', '3', '4', '5', 'z', '7']";
    let top = "['1', '2', '3', 'y', 'y2', '5', '6', '7']";
    three(&fx, [base, mid, top]);
    fx.check("Shape()", "'.T.TA.T. .T.T-1.T. .T.T-1.T.'");
    // With three buffers a bare `:diffget` is ambiguous: E101.
    fx.check_error(
        "3diffget",
        "E101: More than two buffers in diff mode, don''t know which one to use",
    );
    // From the top window take line 2's block from the middle buffer.
    fx.run("1wincmd w | exe '2diffget ' . winbufnr(2)");
    fx.check("Lines(1)", "'1|x|3|y|y2|5|6|7'");
    fx.check("Shape()", "'.T.TA.T. .T.T-1.T. .T.T-1.T.'");
    // Put the top buffer's added lines into the base buffer, by number.
    fx.run("1wincmd w | exe '4,5diffput ' . winbufnr(3)");
    fx.check("Lines(3)", "'1|2|3|y|y2|5|6|7'");
    fx.check("Shape()", "'.T.TC.T. .T.T-1.T. .T.TC.T.'");
    // A `:diffget` over the whole buffer from the base makes the middle
    // buffer equal to it in every block.
    fx.run("2wincmd w | exe '%diffget ' . winbufnr(3)");
    fx.check("Lines(2)", "'1|2|3|y|y2|5|6|7'");
    fx.check("Shape()", "'.T...... .T...... .T......'");
    // Undo in the buffer `:diffput` changed brings its block back once the
    // top buffer leaves diff mode ...
    fx.run("1wincmd w | diffoff");
    fx.run("3wincmd w | silent! undo");
    fx.check("Lines(3)", "'1|2|3|4|5|6|7'");
    fx.check("Shape()", "'........ ...TA... ...T-1...'");
    // ... and `dp` in what is now a two-buffer diff puts it again.
    fx.run("2wincmd w | call cursor(1, 1)");
    fx.run("silent! normal! ]cdp");
    fx.check("Lines(3)", "'1|2|3|y|y2|5|6|7'");
    fx.check("Shape()", "'........ ........ ........'");
    teardown(&fx);
    fx.run("silent! delfunction Lines");
}

#[test]
fn diffupdate_after_edits_that_split_and_merge_blocks() {
    let sandbox = Sandbox::dir("diff_split_merge");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    fx.run("set diffopt=internal,filler");
    fx.run(&format!(
        "call Two({}, {})",
        "['1', '2', '3', '4', '5', '6', '7', '8']", "['1', 'a', 'b', 'c', '5', 'd', '7', '8']"
    ));
    fx.check("Shape()", "'.TTT.T.. .TTT.T..'");
    // Make the middle of the first block equal again: one block becomes
    // two once updated, and stays one until then.
    fx.run("1wincmd w | call setline(3, '3')");
    fx.check("Shape()", "'.TCT.T.. .TCT.T..'");
    fx.run("diffupdate");
    fx.check("Shape()", "'.T.T.T.. .T.T.T..'");
    // Make the line between two blocks differ: they merge.
    fx.run("call setline(5, 'e')");
    fx.run("diffupdate");
    fx.check("Shape()", "'.T.TTT.. .T.TTT..'");
    // Lines inserted inside a block grow it; deleted ones shrink it.
    fx.run("call append(2, ['n1', 'n2'])");
    fx.check("Shape()", "'.TAA.TTT.. .T-2.TTT..'");
    fx.run("diffupdate");
    fx.check("Shape()", "'.TAA.TTT.. .T-2.TTT..'");
    fx.run("3,6delete _");
    fx.check("Shape()", "'.TTT-2.. .TTTAA..'");
    fx.run("diffupdate");
    fx.check("Shape()", "'.TTT-2.. .TTTAA..'");
    // Make the buffers equal: no block is left.
    fx.run("%delete _ | call setline(1, getbufline(winbufnr(2), 1, '$'))");
    fx.run("diffupdate");
    fx.check("Shape()", "'........ ........'");
    teardown(&fx);
}

#[test]
fn linematch_with_iwhite_and_icase() {
    let sandbox = Sandbox::dir("diff_linematch_opts");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    // The second hunk is 2 lines against 4: linematch pairs `B one` and
    // `c` with their look-alikes (or, under the flags, their equals).
    let old = "['a', 'common1', 'B one two', 'c three four', 'common2', 'tail']";
    let new = "['A', 'common1', 'x', 'b  one two', 'y', 'C three four', 'common2', 'tail']";
    let size = sized(&fx);
    let mut moved = Vec::new();
    for opts in [
        "internal,filler",
        "internal,filler,linematch:30",
        "internal,filler,linematch:30,iwhite",
        "internal,filler,linematch:30,icase",
        "internal,filler,linematch:30,iwhite,icase",
        "internal,filler,iwhite,icase",
        "internal,filler,linematch:30,iwhiteall,icase",
        // Past the limit, linematch gives the hunk up.
        "internal,filler,linematch:3",
    ] {
        fx.run(&format!("set diffopt={opts}"));
        fx.run(&format!("call Two({old}, {new})"));
        // Line matching waits until a block is drawn: before a redraw the
        // window has no `w_botline` to say what is on screen.
        let before = fx.eval("Shape()");
        fx.run("resize 20 | redraw!");
        moved.push(format!("{opts}: {before} {}", fx.eval("Shape()")));
    }
    teardown(&fx);
    unsize(&fx, &size);
    // Each row: the blocks before the first redraw, then after it.
    let want: &[&str] = &[
        "internal,filler: 'T.TTAA.. T.TT-2..' 'T.TTAA.. T.TT-2..'",
        "internal,filler,linematch:30: 'T.TTAA.. T.-1T-1T..' 'T.ATAT.. T.-1T-1T..'",
        "internal,filler,linematch:30,iwhite: 'T.TTAA.. T.-1T-1T..' 'T.ATAT.. T.-1T-1T..'",
        "internal,filler,linematch:30,icase: '..TAA... ..-1C-1...' '..ACA... ..-1C-1...'",
        "internal,filler,linematch:30,iwhite,icase: '..A.A... ..-1.-1...' '..A.A... ..-1.-1...'",
        "internal,filler,iwhite,icase: '..A.A... ..-1.-1...' '..A.A... ..-1.-1...'",
        "internal,filler,linematch:30,iwhiteall,icase: '..A.A... ..-1.-1...' '..A.A... ..-1.-1...'",
        "internal,filler,linematch:3: 'T.TTAA.. T.TT-2..' 'T.TTAA.. T.TT-2..'",
    ];
    assert_eq!(moved, want);
}

#[test]
fn blocks_follow_move_and_undo() {
    let sandbox = Sandbox::dir("diff_move_undo");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    fx.run("set diffopt=internal,filler");
    fx.run(&format!(
        "call Two({}, {})",
        "['1', '2', '3', '4', '5', '6', '7', '8']", "['1', 'x', '3', '4', '5', 'y', '7', '8']"
    ));
    fx.check("Shape()", "'.T...T.. .T...T..'");
    fx.run("let &ul = &ul");
    // `:move` is a delete and an append: the blocks move with the lines
    // before any update, and the update then finds the real ones.
    fx.run("1wincmd w | 2move 7");
    fx.check("Shape()", "'.-1...T.A. .A...T.-1.'");
    fx.run("diffupdate");
    fx.check("Shape()", "'.-1...T.A. .A...T.-1.'");
    fx.run("let &ul = &ul");
    fx.run("1,3move $");
    fx.check("Shape()", "'^4.T.A.AAA AAAA.T.-1.-3'");
    fx.run("diffupdate");
    fx.check("Shape()", "'^4.T.A.AAA AAAA.T.-1.-3'");
    fx.run("let &ul = &ul");
    // Undo puts the lines back, and the blocks with them.
    fx.run("silent! undo");
    fx.check("Shape()", "'.-1...T.A. .A...T.-1.'");
    fx.run("diffupdate");
    fx.check("Shape()", "'.-1...T.A. .A...T.-1.'");
    fx.run("let &ul = &ul");
    fx.run("silent! undo");
    fx.check("Shape()", "'.T...T.. .T...T..'");
    fx.run("diffupdate");
    fx.check("Shape()", "'.T...T.. .T...T..'");
    fx.run("silent! redo");
    fx.check("Shape()", "'.-1...T.A. .A...T.-1.'");
    // The same `:move` in the other buffer.
    fx.run("2wincmd w | 8move 0");
    fx.check("Shape()", "'^1.-1...T.AA A.A...T.-2'");
    fx.run("diffupdate");
    fx.check("Shape()", "'^1.-1...T.AA A.A...T.-2'");
    teardown(&fx);
}

/// The same pairs through the external `diff` program (with and without
/// a 'diffexpr' that runs it) and the internal engine: the blocks agree.
#[test]
fn the_external_and_internal_paths_agree() {
    let sandbox = Sandbox::dir("diff_external");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    fx.run(concat!(
        "func! ExtExpr()\n",
        "  call system('diff -a ' . v:fname_in . ' ' . v:fname_new . ' > ' . v:fname_out)\n",
        "endfunc"
    ));
    let pairs: &[(&str, &str)] = &[
        ("['1', '2', '3']", "['1', '2', '3']"),
        ("['1', '2', '3', '4', '5']", "['1', 'x', '3', '4', 'y']"),
        ("['1', '2', '3']", "['0', '1', '2', '3', '4']"),
        (
            "['1', '2', '3', '4', '5', '6']",
            "['1', '4', '5', 'a', 'b']",
        ),
        ("['a', 'b']", "['c']"),
        ("['', '', 'x']", "['x', '', '']"),
    ];
    let mut rows = Vec::new();
    for (old, new) in pairs {
        let mut got = Vec::new();
        for how in [
            "set diffopt=internal,filler diffexpr=",
            "set diffopt=filler diffexpr=",
            "set diffopt=filler diffexpr=ExtExpr()",
        ] {
            fx.run(how);
            fx.run(&format!("call Two({old}, {new})"));
            got.push(fx.eval("Shape()"));
        }
        if got.iter().any(|g| *g != got[0]) {
            rows.push(format!("{old} {new}: {got:?}"));
        }
        rows.push(format!("{old} {new}: {}", got[0]));
    }
    fx.run("silent! delfunction ExtExpr");
    teardown(&fx);
    let want: &[&str] = &[
        "['1', '2', '3'] ['1', '2', '3']: '... ...'",
        "['1', '2', '3', '4', '5'] ['1', 'x', '3', '4', 'y']: '.T..T .T..T'",
        "['1', '2', '3'] ['0', '1', '2', '3', '4']: 'A...A ^1...-1'",
        "['1', '2', '3', '4', '5', '6'] ['1', '4', '5', 'a', 'b']: '.-2..TA .AA..T-1'",
        "['a', 'b'] ['c']: 'T-1 TA'",
        "['', '', 'x'] ['x', '', '']: 'A..-1 ^1..A'",
    ];
    assert_eq!(rows, want);
}

#[test]
fn closing_a_diffed_window_keeps_the_others_blocks() {
    let sandbox = Sandbox::dir("diff_close");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    fx.run("set diffopt=internal,filler hidden");
    let base = "['1', '2', '3', '4', '5']";
    let mid = "['1', 'x', '3', '4', '5']";
    let top = "['1', '2', '3', 'y', '5']";
    three(&fx, [base, mid, top]);
    fx.check("Shape()", "'.T.T. .T.T. .T.T.'");
    // Closing a window hides its buffer, which stays in diff mode.
    fx.run("let g:mid = winbufnr(2) | 2wincmd w | close");
    fx.check("Shape()", "'.T.T. .T.T.'");
    fx.run("diffupdate");
    fx.check("Shape()", "'.T.T. .T.T.'");
    // Wiping it out takes it out of the blocks.
    fx.run("exe 'bwipeout! ' . g:mid | unlet g:mid");
    fx.check("Shape()", "'...T. ...T.'");
    fx.run("diffupdate");
    fx.check("Shape()", "'...T. ...T.'");
    // A tab page with its own diff; closing it leaves the first alone.
    fx.run("tabnew | call setline(1, ['a', 'b']) | diffthis");
    fx.run("vnew | call setline(1, ['a', 'c']) | diffthis");
    fx.check("Shape()", "'.T .T'");
    fx.run("tabclose!");
    fx.check("Shape()", "'...T. ...T.'");
    // `:diffoff!` drops every block.
    fx.run("diffoff!");
    fx.check("Shape()", "'..... .....'");
    fx.run("set hidden&");
    teardown(&fx);
}

/// What the drawing asks per line: the fold level 'foldmethod' `diff`
/// gives each line, which lines are folded, and the filler/highlight.
#[test]
fn fold_levels_and_line_status_per_line() {
    let sandbox = Sandbox::dir("diff_foldlevel");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    fx.run("set diffopt=internal,filler,context:1");
    let old: Vec<String> = (1..=20).map(|n| format!("'{n}'")).collect();
    let mut new = old.clone();
    new[4] = "'x'".into();
    new.remove(12);
    new.insert(17, "'add'".into());
    fx.run(&format!(
        "call Two([{}], [{}])",
        old.join(", "),
        new.join(", ")
    ));
    fx.run("set foldenable");
    fx.check("Shape()", "'....T.......-1.....A.. ....T.......A.....-1..'");
    fx.check(
        "map(range(1, 20), 'foldlevel(v:val)')",
        "[1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 1]",
    );
    fx.check(
        "map(range(1, 20), 'foldclosed(v:val)')",
        "[1, 1, 1, -1, -1, -1, 7, 7, 7, 7, 7, -1, -1, 14, 14, 14, -1, -1, -1, -1]",
    );
    fx.check("&l:foldmethod", "'diff'");
    fx.check(
        "[&l:foldlevel, &l:foldcolumn, &l:scrollbind, &l:cursorbind, &l:wrap]",
        "[0, '2', 1, 1, 0]",
    );
    // The other window, through `win_execute`.
    fx.run("call win_execute(win_getid(2), 'let g:lv = map(range(1, 20), \"foldlevel(v:val)\")')");
    fx.check(
        "g:lv",
        "[1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 1]",
    );
    // Context 0 and no filler.
    fx.run("set diffopt=internal,context:0");
    fx.check(
        "map(range(1, 20), 'foldlevel(v:val)')",
        "[1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 1]",
    );
    fx.check("Shape()", "'....T............A.. ....T.......A.......'");
    fx.run("diffoff!");
    fx.check(
        "[&l:foldmethod, &l:foldcolumn, &l:scrollbind]",
        "['manual', '0', 0]",
    );
    fx.run("set foldenable& foldmethod& foldcolumn& nofoldenable | unlet! g:lv");
    teardown(&fx);
}

/// 'scrollbind' between diffed windows lines up the blocks: filler lines
/// count as screen lines, and `:syncbind` and `]c` honour them.
#[test]
fn scrollbind_over_filler_lines() {
    let sandbox = Sandbox::dir("diff_scrollbind");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    let size = sized(&fx);
    fx.run("set diffopt=internal,filler");
    let old: Vec<String> = (1..=60).map(|n| format!("'{n}'")).collect();
    let mut new = old.clone();
    // Five lines gone from the new side at 10, three added at 30.
    new.drain(9..14);
    for (i, a) in ["'a1'", "'a2'", "'a3'"].iter().enumerate() {
        new.insert(25 + i, (*a).into());
    }
    fx.run(&format!(
        "call Two([{}], [{}])",
        old.join(", "),
        new.join(", ")
    ));
    // Diff mode folds the equal lines; this case reads the lines as they
    // are.
    fx.run("windo setlocal nofoldenable");
    fx.run("resize 10 | redraw!");
    fx.check("[winheight(1), winheight(2), &l:scrollbind]", "[10, 10, 1]");
    fx.run(concat!(
        "func! Views()\n",
        "  return map([1, 2], {_, w -> [getwininfo(win_getid(w))[0].topline, ",
        "win_execute(win_getid(w), 'echon winsaveview().topfill'), ",
        "line('.', win_getid(w))]})\n",
        "endfunc"
    ));
    fx.check("Views()", "[[1, '0', 1], [1, '0', 1]]");
    // Scroll the new side to its line 9 at the top: the old side shows its
    // line 9 too, with the deleted block's filler below.
    fx.run("1wincmd w | call winrestview({'lnum': 9, 'topline': 9}) | syncbind");
    fx.check("Views()", "[[9, '0', 9], [9, '0', 9]]");
    fx.run("call winrestview({'lnum': 10, 'topline': 10}) | syncbind");
    fx.check("Views()", "[[10, '0', 10], [15, '0', 15]]");
    fx.run("2wincmd w | call winrestview({'lnum': 12, 'topline': 12}) | syncbind");
    fx.check("Views()", "[[10, '3', 10], [12, '0', 12]]");
    // `]c` from the top lands on each block in both windows.
    fx.run("1wincmd w | call cursor(1, 1) | silent! normal! ]c");
    fx.run("redraw!");
    fx.check("Views()", "[[6, '0', 10], [12, '0', 12]]");
    fx.run("silent! normal! ]c");
    fx.run("redraw!");
    fx.check("Views()", "[[22, '0', 26], [27, '0', 31]]");
    // Scrolling by `CTRL-E` in one window moves the other.
    fx.run("1wincmd w | call cursor(1, 1) | normal! gg");
    fx.run("exe \"normal! 12\\<C-E>\" | redraw!");
    fx.check("Views()", "[[10, '2', 10], [13, '0', 13]]");
    fx.run("2wincmd w | exe \"normal! 3\\<C-Y>\" | redraw!");
    fx.check("Views()", "[[10, '5', 10], [10, '0', 13]]");
    fx.run("silent! delfunction Views");
    teardown(&fx);
    unsize(&fx, &size);
}

/// `:diffget` holds a block across `changed_lines`, whose buffer-update
/// callbacks are user code: an `:diffupdate` there waits until the get is
/// done, and the blocks come out as if it ran after.
#[test]
fn diffget_under_an_on_lines_callback() {
    let sandbox = Sandbox::dir("diff_on_lines");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    fx.run("set diffopt=internal,filler");
    fx.run(&format!(
        "call Two({}, {})",
        "['1', '2', '3', '4', '5', '6', '7', '8']", "['1', 'x', '3', '4', 'y', 'z', '7', '8', '9']"
    ));
    fx.check("Shape()", "'.T..TT..A .T..TT..-1'");
    fx.run(concat!(
        "lua vim.g.calls = 0; vim.api.nvim_buf_attach(0, false, {on_lines = function() ",
        "vim.g.calls = vim.g.calls + 1; vim.cmd('silent! diffupdate') end})"
    ));
    fx.run("1wincmd w | %diffget");
    fx.check("g:calls", "3");
    fx.check(
        "getline(1, '$')",
        "['1', '2', '3', '4', '5', '6', '7', '8']",
    );
    fx.check("Shape()", "'........ ........'");
    fx.run("lua vim.g.calls = nil");
    teardown(&fx);
}

/// A `:diffoff!` from the same callback only marks the list invalid: the
/// get still walks every block it held, and the next update drops them.
#[test]
fn diffoff_under_an_on_lines_callback() {
    let sandbox = Sandbox::dir("diff_on_lines2");
    let fx = Probe::new(sandbox.editor());
    setup(&sandbox, &fx);
    fx.run("set diffopt=internal,filler");
    fx.run("call Two(['1', '2', '3', '4', '5', '6'], ['1', 'x', '3', '4', 'y', '6'])");
    fx.run(concat!(
        "lua vim.api.nvim_buf_attach(0, false, {on_lines = function() ",
        "vim.cmd('silent! diffoff!') end})"
    ));
    fx.run("1wincmd w | %diffget");
    fx.check("getline(1, '$')", "['1', '2', '3', '4', '5', '6']");
    fx.check("Shape()", "'...... ......'");
    teardown(&fx);
}
