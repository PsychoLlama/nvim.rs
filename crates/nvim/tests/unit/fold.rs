//! The fold tree's line-number bookkeeping, and the tree the editor builds.
//!
//! `adjust_fold_list` is what every buffer change that inserts, deletes or
//! moves lines eventually reaches, and it is the only part of the fold
//! machinery that runs without a window: it walks a `Vec<Fold>` and rewrites
//! `top`/`len` in place, recursing into `nested`. The first cases are the
//! shapes its `if` chain distinguishes, with the expected results derived
//! from `v0.12.4`'s `src/nvim/fold.c` rather than from this port; Miri runs
//! them.

use neovim::fold::{Fold, adjust_fold_list};
use neovim::pos::MAXLNUM;
use neovim::types::LineNr;

/// The sentinel `mark_adjust` passes as `amount` to mean "these lines are
/// gone".
const DELETED: LineNr = MAXLNUM;

/// Every fold in `folds` as `(top, len)`.
fn spans(folds: &[Fold]) -> Vec<(LineNr, LineNr)> {
    folds.iter().map(|fold| (fold.top, fold.len)).collect()
}

#[test]
fn lines_inserted_above_a_fold_push_it_down() {
    let mut folds = vec![Fold::new(10, 5)];
    // `:normal 3O` at line 5 -> mark_adjust(5, MAXLNUM, 3, 0).
    adjust_fold_list(&mut folds, 5, DELETED, 3, 0);
    assert_eq!(spans(&folds), [(13, 5)]);
}

#[test]
fn a_fold_swallowed_whole_is_deleted() {
    let mut folds = vec![Fold::new(10, 5), Fold::new(20, 2)];
    // `:10,14d` -> the first fold's every line is gone, and the second
    // one slides up by the five removed lines.
    adjust_fold_list(&mut folds, 10, 14, DELETED, -5);
    assert_eq!(spans(&folds), [(15, 2)]);
}

#[test]
fn a_deletion_inside_a_fold_only_shortens_it() {
    let mut folds = vec![Fold::new(10, 5)];
    // `:12,13d` -> the fold still starts at 10 and is two lines shorter.
    adjust_fold_list(&mut folds, 12, 13, DELETED, -2);
    assert_eq!(spans(&folds), [(10, 3)]);
}

#[test]
fn a_deletion_reaches_nested_folds() {
    let mut parent = Fold::new(10, 10);
    // Absolute 12..14, i.e. 2..4 relative to the parent.
    parent.nested.push(Fold::new(2, 3));
    let mut folds = vec![parent];
    adjust_fold_list(&mut folds, 12, 13, DELETED, -2);
    assert_eq!(spans(&folds), [(10, 8)]);
    assert_eq!(spans(&folds[0].nested), [(2, 1)]);
}

#[test]
fn a_fold_entirely_below_the_change_only_shifts() {
    let mut folds = vec![Fold::new(10, 3), Fold::new(30, 4)];
    // `:15,19d` touches neither fold's lines; only the later one moves.
    adjust_fold_list(&mut folds, 15, 19, DELETED, -5);
    assert_eq!(spans(&folds), [(10, 3), (25, 4)]);
}

#[test]
fn an_empty_list_is_left_alone() {
    let mut folds: Vec<Fold> = Vec::new();
    adjust_fold_list(&mut folds, 1, 10, DELETED, -10);
    assert!(folds.is_empty());
}

/// The fold tree as the editor builds it: manual folds under `:move`, the
/// computed methods under edits, markers with multibyte text, and the
/// user code ('foldexpr', 'foldtext') the update and the drawing run.
///
/// Every case reads the tree back through the Vimscript builtins
/// (`foldlevel()`, `foldclosed()`, `foldclosedend()`, `foldtextresult()`),
/// which walk it without the display cache. The expected shapes were
/// recorded on the `GArray` tree before it became nested vectors.
#[cfg(not(miri))]
mod editor {
    use crate::support::{Editor, cstr, editor_lock};
    use neovim::eval::eval_to_string;
    use neovim::ex_docmd::do_cmdline_cmd;

    /// Puts the buffer and the fold options back for the next case.
    struct Restore;

    impl Drop for Restore {
        fn drop(&mut self) {
            for line in [
                "silent! normal! \u{1b}",
                "silent! set foldmethod& foldexpr& foldtext& foldmarker& foldminlines& foldlevel& foldenable&",
                "silent! set commentstring& shiftwidth& foldignore&",
                "silent! normal! zE",
                "silent! delfunction! g:FoldUnitExpr",
                "silent! delfunction! g:FoldUnitText",
                "silent! unlet! g:fold_unit_log",
                "silent! keepjumps keepmarks %delete _",
            ] {
                let text = cstr(line);
                let _ = do_cmdline_cmd(&text);
            }
        }
    }

    fn run(_editor: &Editor, line: &str) {
        let text = cstr(line);
        let _ = do_cmdline_cmd(&text);
    }

    fn string(_editor: &Editor, expr: &str) -> String {
        eval_to_string(expr.as_bytes(), false, false)
            .map(|got| String::from_utf8(got.to_vec()).unwrap())
            .unwrap_or_default()
    }

    /// Every line's fold level, and with every fold closed the span each
    /// closed fold covers: `level` or `level:first-last`, space separated.
    fn shape(editor: &Editor) -> String {
        run(editor, "silent! normal! zM");
        let got = string(
            editor,
            "join(map(range(1, line('$')), {_, l -> foldclosed(l) < 0 ? \
             string(foldlevel(l)) : foldlevel(l) .. ':' .. foldclosed(l) .. '-' .. \
             foldclosedend(l)}), ' ')",
        );
        run(editor, "silent! normal! zR");
        got
    }

    /// The buffer's text, lines joined with `/`.
    fn text(editor: &Editor) -> String {
        string(editor, "join(getline(1, '$'), '/')")
    }

    /// `n` lines `l1`..`ln`.
    fn numbered(editor: &Editor, n: usize) {
        run(editor, "silent! keepjumps keepmarks %delete _");
        run(
            editor,
            &format!("call setline(1, map(range(1, {n}), '\"l\" .. v:val'))"),
        );
    }

    #[test]
    fn nested_manual_folds_follow_moved_lines() {
        let editor = editor_lock();
        let _restore = Restore;
        numbered(&editor, 30);
        run(&editor, "set foldmethod=manual foldminlines=0");
        for range in ["3,8", "4,5", "10,20", "12,14", "16,18", "13,13", "22,25"] {
            // Opened again at once: a range landing in a closed fold would
            // be stretched over all of it.
            run(&editor, &format!("{range}fold | normal! zR"));
        }
        run(&editor, "normal! zR");
        let before = shape(&editor);
        // A block holding nested folds jumps folds below it, inside one
        // parent: the rotation `fold_move_range` does by three reversals.
        run(&editor, "12,14move 18");
        let inner = shape(&editor);
        // Whole toplevel folds swap places.
        run(&editor, "22,25move 2");
        let toplevel = shape(&editor);
        // A move that cuts folds at both ends.
        run(&editor, "5,11move 27");
        let cut = shape(&editor);
        // And back up to the top, past everything.
        run(&editor, "20,30move 0");
        let top = shape(&editor);
        assert_eq!(
            [before, inner, toplevel, cut, top],
            [
                "0 0 1:3-8 2:3-8 2:3-8 1:3-8 1:3-8 1:3-8 0 1:10-20 1:10-20 2:10-20 3:10-20 2:10-20 1:10-20 2:10-20 2:10-20 2:10-20 1:10-20 1:10-20 0 1:22-25 1:22-25 1:22-25 1:22-25 0 0 0 0 0",
                "0 0 1:3-8 2:3-8 2:3-8 1:3-8 1:3-8 1:3-8 0 1:10-20 1:10-20 1:10-20 2:10-20 2:10-20 2:10-20 2:10-20 3:10-20 2:10-20 1:10-20 1:10-20 0 1:22-25 1:22-25 1:22-25 1:22-25 0 0 0 0 0",
                "0 0 1:3-6 1:3-6 1:3-6 1:3-6 1:7-12 2:7-12 2:7-12 1:7-12 1:7-12 1:7-12 0 1:14-24 1:14-24 1:14-24 2:14-24 2:14-24 2:14-24 2:14-24 3:14-24 2:14-24 1:14-24 1:14-24 0 0 0 0 0 0",
                "0 0 1:3-4 1:3-4 0 0 1:7-17 1:7-17 1:7-17 2:7-17 2:7-17 2:7-17 2:7-17 3:7-17 2:7-17 1:7-17 1:7-17 0 0 0 0 0 1:23-27 2:23-27 2:23-27 1:23-27 1:23-27 0 0 0",
                "0 0 0 1:4-8 2:4-8 2:4-8 1:4-8 1:4-8 0 0 0 0 0 1:14-15 1:14-15 0 0 1:18-28 1:18-28 1:18-28 2:18-28 2:18-28 2:18-28 2:18-28 3:18-28 2:18-28 1:18-28 1:18-28 0 0",
            ].map(String::from)
        );
    }

    #[test]
    fn indent_folds_split_and_merge_under_edits() {
        let editor = editor_lock();
        let _restore = Restore;
        run(&editor, "silent! keepjumps keepmarks %delete _");
        run(
            &editor,
            "call setline(1, ['a', '  b', '    c', '    d', '  e', 'f', '  g', '  h', \
             'i', '  j', '    k', '  l', 'm'])",
        );
        run(&editor, "set shiftwidth=2 foldminlines=0 foldmethod=indent");
        let before = shape(&editor);
        // Indenting the line between two folds merges them.
        run(&editor, "6normal! >>");
        let merged = shape(&editor);
        run(&editor, "9normal! >>");
        let merged2 = shape(&editor);
        // Taking it back out splits them again.
        run(&editor, "6normal! <<");
        let split = shape(&editor);
        // Deleting lines inside nested folds, and a line that ends one.
        run(&editor, "3,4delete _");
        let deleted = shape(&editor);
        // Lines opened inside a fold, in Insert mode.
        run(&editor, "execute \"2normal! o    x\\<CR>y\\<Esc>\"");
        let inserted = shape(&editor);
        assert_eq!(
            [
                before,
                merged,
                merged2,
                split,
                deleted,
                inserted,
                text(&editor)
            ],
            [
                "0 1:2-5 2:2-5 2:2-5 1:2-5 0 1:7-8 1:7-8 0 1:10-12 2:10-12 1:10-12 0",
                "0 1:2-8 2:2-8 2:2-8 1:2-8 1:2-8 1:2-8 1:2-8 0 1:10-12 2:10-12 1:10-12 0",
                "0 1:2-12 2:2-12 2:2-12 1:2-12 1:2-12 1:2-12 1:2-12 1:2-12 1:2-12 2:2-12 1:2-12 0",
                "0 1:2-5 2:2-5 2:2-5 1:2-5 0 1:7-12 1:7-12 1:7-12 1:7-12 2:7-12 1:7-12 0",
                "0 1:2-3 1:2-3 0 1:5-10 1:5-10 1:5-10 1:5-10 2:5-10 1:5-10 0",
                "0 1:2-5 3:2-5 3:2-5 1:2-5 0 1:7-12 1:7-12 1:7-12 1:7-12 2:7-12 1:7-12 0",
                "a/  b/      x/      y/  e/f/  g/  h/  i/  j/    k/  l/m",
            ]
            .map(String::from)
        );
    }

    #[test]
    fn multibyte_markers_open_close_and_clean_up() {
        let editor = editor_lock();
        let _restore = Restore;
        run(&editor, "silent! keepjumps keepmarks %delete _");
        run(
            &editor,
            "call setline(1, ['été «', 'naïve «2', '日本 x', '»2', 'café', '» end', \
             'ü', 'ö', 'ä'])",
        );
        run(
            &editor,
            "set foldmarker=«,» commentstring=/*%s*/ foldminlines=0 foldmethod=marker",
        );
        let levels = shape(&editor);
        run(&editor, "normal! zM");
        let title = string(&editor, "foldtextresult(1)");
        let nested_title = string(&editor, "foldtextresult(2)");
        run(&editor, "normal! zR");
        // `zf` writes the markers, wrapped in 'commentstring'.
        run(&editor, "7,9fold");
        let created = text(&editor);
        let with_new = shape(&editor);
        // `zd` takes them out again, comment and all.
        run(&editor, "8normal! zd");
        let deleted = text(&editor);
        // `zD` on the outer fold takes the nested markers too.
        run(&editor, "1normal! zD");
        let all_gone = text(&editor);
        assert_eq!(
            [
                levels,
                title,
                nested_title,
                created,
                with_new,
                deleted,
                all_gone,
                shape(&editor)
            ],
            [
                "1:1-6 2:1-6 2:1-6 2:1-6 1:1-6 1:1-6 0 0 0",
                "+--  6 lines: été ",
                "+--  5 lines: naïve ",
                "été «/naïve «2/日本 x/»2/café/» end/ü/*«*//ö/ä/*»*/",
                "1:1-6 2:1-6 2:1-6 2:1-6 1:1-6 1:1-6 1:7-9 1:7-9 1:7-9",
                "été «/naïve «2/日本 x/»2/café/» end/ü/ö/ä",
                "été /naïve /日本 x//café/ end/ü/ö/ä",
                "0 0 0 0 0 0 0 0 0",
            ]
            .map(String::from)
        );
    }

    #[test]
    fn foldexpr_that_edits_the_buffer_or_switches_method() {
        let editor = editor_lock();
        let _restore = Restore;
        numbered(&editor, 8);
        run(
            &editor,
            "function! g:FoldUnitExpr()\n\
               silent! call setline(v:lnum, 'edited')\n\
               if v:lnum == 4 | set foldmethod=indent | endif\n\
               return v:lnum % 4 == 1 ? '>1' : v:lnum % 4 == 3 ? '>2' : '='\n\
             endfunction",
        );
        run(
            &editor,
            "set foldminlines=0 foldexpr=g:FoldUnitExpr() foldmethod=expr",
        );
        let first = shape(&editor);
        let method = string(&editor, "&foldmethod");
        assert_eq!(
            [first, method, text(&editor)],
            [
                "1:1-4 1:1-4 2:1-4 2:1-4 1:5-8 1:5-8 2:5-8 2:5-8",
                "indent",
                "l1/l2/l3/l4/l5/l6/l7/l8",
            ]
            .map(String::from)
        );
    }

    /// A 'foldexpr' that switches to "manual" and deletes every fold while
    /// the update is inside one. Upstream (and the `GArray` tree before the
    /// vectors) went on writing through the `fold_T *` it held into the
    /// freed array: heap-use-after-free under ASan. The update now notices
    /// its place is gone and stops, leaving what `zE` left.
    #[test]
    fn foldexpr_that_deletes_every_fold_mid_update() {
        let editor = editor_lock();
        let _restore = Restore;
        run(&editor, "silent! keepjumps keepmarks %delete _");
        run(
            &editor,
            "call setline(1, ['a', '  b', '  c', 'd', '  e', '  f', 'g', '  h'])",
        );
        run(
            &editor,
            "function! g:FoldUnitExpr()\n\
               if v:lnum == 5 | set foldmethod=manual | silent! normal! zE\n\
               endif\n\
               return getline(v:lnum) =~ '^\\s' ? '1' : '>1'\n\
             endfunction",
        );
        run(
            &editor,
            "set foldminlines=0 foldexpr=g:FoldUnitExpr() foldmethod=expr",
        );
        let got = shape(&editor);
        assert_eq!(
            [got, string(&editor, "&foldmethod")],
            ["0 0 0 0 0 0 0 0", "manual"].map(String::from)
        );
    }

    #[test]
    fn foldtext_answers_are_sanitised() {
        let editor = editor_lock();
        let _restore = Restore;
        numbered(&editor, 6);
        run(&editor, "set foldmethod=manual foldminlines=0");
        run(&editor, "1,2fold | 3,4fold | 5,6fold");
        run(
            &editor,
            "function! g:FoldUnitText()\n\
               return v:foldstart == 1 ? \"a\\tb\" : v:foldstart == 3 ? \"日本\\x01z\" : \
             [['chunk', 'Error'], [' two', 'Normal']]\n\
             endfunction",
        );
        run(&editor, "normal! zM");
        let default = string(&editor, "foldtextresult(1)");
        run(&editor, "set foldtext=g:FoldUnitText()");
        let answers = string(
            &editor,
            "join(map([1, 3, 5], {_, l -> foldtextresult(l)}), '|')",
        );
        assert_eq!(
            [default, answers],
            ["+--  2 lines: l1", "a b|日本^Az|chunk two",].map(String::from)
        );
    }
}
