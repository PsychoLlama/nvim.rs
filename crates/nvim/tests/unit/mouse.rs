//! The mouse: what a button, a click count and a modifier do at a screen
//! position.
//!
//! `mouse` had no in-crate test. The battery's `mousesweep` drives it from an
//! `--embed` child; nothing in the crate pinned the decisions the module
//! makes about a *key* -- which button a mouse key names, whether it is a
//! press, a drag or a release, how many clicks it counts, which modifier
//! turns it into a search, a tag jump or a block selection, and what the
//! wheel's four directions scroll. Those are the decisions that read the
//! key's integer, and the integer is what the `Key` retype changes.
//!
//! One engine: [`Pointer::feed`] puts the pointer at a screen cell with
//! `nvim_input_mouse('move', ..)` -- which sets the position at once and
//! queues an event, drained with `getchar(0)` -- and then feeds the mouse
//! key itself with `feedkeys(.., 'xt')`, as if typed. 'mousetime' is 0, so
//! a click count is exactly the one the key names. The answers are read back
//! as expressions; every one is the tree's before the retype.
//!
//! Registers: a case that pastes uses `"x` and puts it back, and the unnamed
//! register and `@/` are restored, so nothing here leaks into a later case's
//! registers (a case elsewhere reads `@q`).

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// The editor with a sized screen and a pointer.
struct Pointer<'a> {
    fx: Probe<'a>,
}

impl<'a> Pointer<'a> {
    fn new(sandbox: &'a Sandbox) -> Self {
        let fx = Probe::new(sandbox.editor());
        fx.home();
        // The lone window may never have been given a height.
        fx.run("set lines=24 columns=80");
        fx.run("resize 10 | resize 22");
        fx.run("set mouse=a mousetime=0 mousemodel=extend mousescroll& nowrap& scrolloff=0");
        fx.run("let g:save_regs = map(['\"', '0', '1', '-'], {_, r -> [r, getreginfo(r)]})");
        fx.run("let g:save_search = @/");
        // The middle button pastes `*` when a clipboard provider is loaded,
        // which reads the host's clipboard: declined for these cases.
        fx.run("let g:save_clip = get(g:, 'loaded_clipboard_provider', 'none')");
        fx.run("let g:loaded_clipboard_provider = 1");
        Pointer { fx }
    }

    /// Start a case: `lines` in buffer 1, the cursor at (1, 1), Normal mode.
    fn lines(&self, lines: &str) {
        self.fx.run("silent! tabonly! | silent! only!");
        self.fx.run("silent! %delete _");
        self.fx.run(&format!("call setline(1, {lines})"));
        self.fx.run("call cursor(1, 1) | normal! zt0");
        self.fx.run("set nowrap& foldcolumn& showtabline&");
    }

    /// Put the pointer on screen cell (`row`, `col`), both from 0, then run
    /// `keys` (a double-quoted string body, `\<..>` notation) as typed.
    fn feed(&self, row: i32, col: i32, keys: &str) {
        // A drag can leave the key after it pushed back, with the pointer
        // where the drag was (`coalesce_drags`); reading it restores that
        // position, so it goes before the move.
        self.fx.run("while getchar(0) | endwhile");
        self.fx.run(&format!(
            "call nvim_input_mouse('move', '', '', 0, {row}, {col})"
        ));
        self.fx.run("while getchar(0) | endwhile");
        self.fx.run(&format!("call feedkeys(\"{keys}\", 'xt')"));
    }

    #[track_caller]
    fn expect(&self, want: &[(&str, &str)]) {
        self.fx.expect_all(want);
    }

    /// Leave whatever mode the case ended in.
    fn escape(&self) {
        self.fx.run("call feedkeys(\"\\<Esc>\\<Esc>\", 'xt')");
    }
}

impl Drop for Pointer<'_> {
    fn drop(&mut self) {
        self.escape();
        self.fx
            .run("for [r, info] in g:save_regs | call setreg(r, info) | endfor");
        self.fx.run("let @/ = g:save_search");
        self.fx.run(
            "if g:save_clip is 'none' | unlet g:loaded_clipboard_provider \
             | else | let g:loaded_clipboard_provider = g:save_clip | endif",
        );
        self.fx.run("unlet g:save_regs g:save_search g:save_clip");
        self.fx.run("call setreg('x', [])");
        self.fx.run("call histdel('/', -1)");
        self.fx.run(
            "set mouse& mousetime& mousemodel& mousescroll& wrap& scrolloff& foldcolumn& \
             showtabline& selection& selectmode&",
        );
        self.fx.run("silent! tabonly! | silent! only!");
        self.fx.home();
    }
}

const TEXT: &str =
    "['abc def ghi', 'line two', 'three (x y) z', '', '\tt\u{e9}st \u{65e5}\u{672c}']";

#[test]
fn a_left_click_moves_the_cursor_to_the_cell() {
    let sandbox = Sandbox::globals();
    let p = Pointer::new(&sandbox);
    for (row, col, want) in [
        (0, 4, "[1, 5]"),
        (1, 2, "[2, 3]"),
        // Past the end of the line: the last character.
        (1, 40, "[2, 8]"),
        // An empty line.
        (3, 7, "[4, 1]"),
        // Past the last line: the last line.
        (15, 3, "[5, 1]"),
        // A tab spans eight cells, a wide character two.
        (4, 3, "[5, 1]"),
        (4, 8, "[5, 2]"),
        (4, 9, "[5, 3]"),
        (4, 13, "[5, 8]"),
        (4, 14, "[5, 8]"),
        (4, 15, "[5, 11]"),
    ] {
        p.lines(TEXT);
        p.feed(row, col, r"\<LeftMouse>");
        p.expect(&[("[line('.'), col('.')]", want), ("mode()", "'n'")]);
    }
    // The release after it changes nothing.
    p.lines(TEXT);
    p.feed(1, 2, r"\<LeftMouse>");
    p.feed(1, 2, r"\<LeftRelease>");
    p.expect(&[("[line('.'), col('.')]", "[2, 3]"), ("mode()", "'n'")]);
    // A release or a drag without a press is ignored.
    p.lines(TEXT);
    p.feed(1, 2, r"\<RightRelease>");
    p.feed(2, 2, r"\<RightDrag>");
    p.expect(&[("[line('.'), col('.')]", "[1, 1]"), ("mode()", "'n'")]);
    // 'mouse' is the UI's filter, not this one's: a fed click lands anyway.
    p.lines(TEXT);
    p.fx.run("set mouse=i");
    p.feed(1, 2, r"\<LeftMouse>");
    p.expect(&[("[line('.'), col('.')]", "[2, 3]")]);
    p.fx.run("set mouse=a");
    // A count before the click.
    p.lines(TEXT);
    p.feed(2, 2, r"3\<LeftMouse>");
    p.expect(&[("[line('.'), col('.')]", "[3, 3]")]);
}

#[test]
fn a_drag_selects_and_the_buttons_extend() {
    let sandbox = Sandbox::globals();
    let p = Pointer::new(&sandbox);
    let sel = "[mode(), getpos('v')[1:2], line('.'), col('.')]";
    // Press, drag, release: a characterwise selection from the press.
    p.lines(TEXT);
    p.feed(0, 2, r"\<LeftMouse>");
    p.feed(1, 4, r"\<LeftDrag>");
    p.feed(1, 4, r"\<LeftRelease>");
    p.expect(&[(sel, "['v', [1, 3], 2, 5]")]);
    // A drag back over the press point.
    p.feed(0, 1, r"\<LeftMouse>");
    p.feed(0, 6, r"\<LeftDrag>");
    p.feed(0, 0, r"\<LeftDrag>");
    p.expect(&[(sel, "['v', [1, 2], 1, 1]")]);
    p.escape();
    // ALT drags blockwise.
    p.lines(TEXT);
    p.feed(0, 1, r"\<A-LeftMouse>");
    p.feed(2, 3, r"\<A-LeftDrag>");
    p.expect(&[(sel, "['v', [1, 2], 3, 4]")]);
    p.escape();
    // The right button extends from the cursor ('mousemodel' extend).
    p.lines(TEXT);
    p.fx.run("call cursor(1, 3)");
    p.feed(2, 5, r"\<RightMouse>");
    p.expect(&[(sel, "['v', [1, 3], 3, 6]")]);
    // ... and again, from the nearer end.
    p.feed(0, 0, r"\<RightMouse>");
    p.expect(&[(sel, "['v', [3, 6], 1, 1]")]);
    // A left click ends the selection.
    p.feed(1, 1, r"\<LeftMouse>");
    p.expect(&[(sel, "['n', [2, 2], 2, 2]")]);
    // 'selection' exclusive.
    p.lines(TEXT);
    p.fx.run("set selection=exclusive");
    p.feed(0, 2, r"\<LeftMouse>");
    p.feed(0, 6, r"\<LeftDrag>");
    p.expect(&[(sel, "['v', [1, 3], 1, 7]")]);
    p.escape();
    p.fx.run("set selection&");
    // 'selectmode' mouse starts Select mode.
    p.lines(TEXT);
    p.fx.run("set selectmode=mouse");
    p.feed(0, 2, r"\<LeftMouse>");
    p.feed(0, 6, r"\<LeftDrag>");
    p.expect(&[(sel, "['s', [1, 3], 1, 7]")]);
    p.escape();
    p.fx.run("set selectmode&");
}

#[test]
fn the_click_count_selects_a_word_a_line_or_a_block() {
    let sandbox = Sandbox::globals();
    let p = Pointer::new(&sandbox);
    let sel = "[mode(), getpos('v')[1:2], line('.'), col('.')]";
    for (row, col, keys, want) in [
        (0, 5, r"\<2-LeftMouse>", "['v', [1, 5], 1, 7]"),
        (0, 5, r"\<3-LeftMouse>", "['V', [1, 6], 1, 6]"),
        (0, 5, r"\<4-LeftMouse>", "['\u{16}', [1, 6], 1, 6]"),
        (0, 5, r"\<A-2-LeftMouse>", "['n', [1, 6], 1, 6]"),
        // On a bracket: the block it opens.
        (2, 6, r"\<2-LeftMouse>", "['v', [3, 7], 3, 11]"),
        (2, 10, r"\<2-LeftMouse>", "['v', [3, 11], 3, 7]"),
        // On white space and on punctuation.
        (0, 3, r"\<2-LeftMouse>", "['v', [1, 4], 1, 4]"),
        (4, 9, r"\<2-LeftMouse>", "['v', [5, 2], 5, 6]"),
    ] {
        p.lines(TEXT);
        // The single click a double one follows, as a terminal sends them.
        p.feed(row, col, r"\<LeftMouse>");
        p.feed(row, col, keys);
        p.expect(&[(sel, want)]);
        p.escape();
    }
    // A double click then a drag extends by words.
    p.lines(TEXT);
    p.feed(0, 5, r"\<LeftMouse>");
    p.feed(0, 5, r"\<2-LeftMouse>");
    p.feed(0, 9, r"\<2-LeftDrag>");
    p.expect(&[(sel, "['v', [1, 5], 1, 10]")]);
    p.escape();
    p.lines(TEXT);
    p.feed(0, 9, r"\<LeftMouse>");
    p.feed(0, 9, r"\<2-LeftMouse>");
    p.feed(0, 1, r"\<2-LeftDrag>");
    p.expect(&[(sel, "['v', [1, 9], 1, 2]")]);
    p.escape();
}

#[test]
fn a_modifier_turns_a_click_into_a_command() {
    let sandbox = Sandbox::globals();
    let p = Pointer::new(&sandbox);
    let at = "[line('.'), col('.'), mode()]";
    // Shift-click searches for the word under the pointer, forward with the
    // left button and backward with the right.
    p.lines("['foo bar', 'baz foo', 'foo']");
    p.feed(0, 1, r"\<S-LeftMouse>");
    p.expect(&[(at, "[2, 5, 'n']"), ("@/", "'\\<foo\\>'")]);
    p.lines("['foo bar', 'baz foo', 'foo']");
    p.feed(1, 5, r"\<S-RightMouse>");
    p.expect(&[(at, "[1, 1, 'n']"), ("@/", "'\\<foo\\>'")]);
    // CTRL-click jumps to the tag: there is none.
    p.lines("['foo bar']");
    p.fx.run("let v:errmsg = ''");
    p.feed(0, 5, r"\<C-LeftMouse>");
    p.expect(&[
        (at, "[1, 6, 'n']"),
        ("v:errmsg", "'E426: Tag not found: bar'"),
    ]);
    // CTRL with another button, and a modified drag, are ignored.
    p.lines(TEXT);
    p.feed(1, 4, r"\<C-MiddleMouse>");
    p.feed(1, 4, r"\<S-LeftDrag>");
    p.expect(&[(at, "[2, 5, 'v']")]);
    // CTRL-right is CTRL-T: the tag stack is empty.
    p.lines(TEXT);
    p.fx.run("let v:errmsg = ''");
    p.feed(1, 4, r"\<C-RightMouse>");
    p.expect(&[
        (at, "[1, 1, 'n']"),
        ("v:errmsg", "'E555: At bottom of tag stack'"),
    ]);
    // 'mousemodel' popup: shift-left is the right button, which extends.
    p.lines(TEXT);
    p.fx.run("set mousemodel=popup");
    p.fx.run("call cursor(1, 2)");
    p.feed(1, 4, r"\<S-LeftMouse>");
    p.expect(&[(at, "[2, 5, 'v']")]);
    p.escape();
    p.fx.run("set mousemodel=extend");
    // The side buttons do nothing unmapped.
    p.lines(TEXT);
    p.feed(1, 4, r"\<X1Mouse>");
    p.feed(1, 4, r"\<X2Mouse>");
    p.expect(&[(at, "[2, 5, 'n']")]);
}

#[test]
fn the_middle_button_puts_a_register() {
    let sandbox = Sandbox::globals();
    let p = Pointer::new(&sandbox);
    let text = "[getline(1, '$'), line('.'), col('.'), mode()]";
    // At the pointer, before it; past the end of the line, after it.
    p.lines("['abc', 'def']");
    p.fx.run("call setreg('x', 'XY', 'c')");
    p.feed(1, 1, r#"\"x\<MiddleMouse>"#);
    p.expect(&[(text, "[['abc', 'dXYef'], 2, 4, 'n']")]);
    p.lines("['abc', 'def']");
    p.fx.run("call setreg('x', 'XY', 'c')");
    p.feed(1, 10, r#"\"x\<MiddleMouse>"#);
    p.expect(&[(text, "[['abc', 'defXY'], 2, 5, 'n']")]);
    // A linewise register goes above the line, or below past the last.
    p.lines("['abc', 'def']");
    p.fx.run("call setreg('x', ['L1'], 'l')");
    p.feed(1, 1, r#"\"x\<MiddleMouse>"#);
    p.expect(&[(text, "[['abc', 'L1', 'def'], 3, 1, 'n']")]);
    p.lines("['abc', 'def']");
    p.fx.run("call setreg('x', ['L1'], 'l')");
    p.feed(9, 1, r#"\"x\<MiddleMouse>"#);
    p.expect(&[(text, "[['abc', 'def', 'L1'], 3, 1, 'n']")]);
    // CTRL-fixindent is not reachable from Normal mode; count repeats.
    p.lines("['abc', 'def']");
    p.fx.run("call setreg('x', 'Z', 'c')");
    p.feed(0, 1, r#"\"x3\<MiddleMouse>"#);
    p.expect(&[(text, "[['aZZZbc', 'def'], 1, 5, 'n']")]);
    // In Insert mode: at the cursor, not the pointer.
    p.lines("['abc', 'def']");
    p.fx.run("call setreg('\"', 'Q', 'c')");
    p.feed(1, 2, r"la\<MiddleMouse>\<Esc>");
    p.expect(&[(text, "[['abQc', 'def'], 1, 3, 'n']")]);
    // In Visual mode: yank the selection, put it at the pointer.
    p.lines("['abc', 'def']");
    p.feed(1, 0, r"vl\<MiddleMouse>");
    p.expect(&[(text, "[['abc', 'abdef'], 2, 3, 'n']")]);
    // With an operator pending: beep, no put.
    p.lines("['abc', 'def']");
    p.feed(1, 0, r"d\<MiddleMouse>");
    p.expect(&[(text, "[['abc', 'def'], 1, 1, 'n']")]);
}

#[test]
fn a_click_is_an_operators_motion() {
    let sandbox = Sandbox::globals();
    let p = Pointer::new(&sandbox);
    let text = "[getline(1, '$'), line('.'), col('.'), mode()]";
    p.lines("['abc def', 'ghi']");
    p.feed(0, 5, r#"\"_d\<LeftMouse>"#);
    p.expect(&[(text, "[['ef', 'ghi'], 1, 1, 'n']")]);
    p.lines("['abc def', 'ghi']");
    p.fx.run("call cursor(1, 6)");
    p.feed(0, 1, r#"\"_d\<LeftMouse>"#);
    p.expect(&[(text, "[['aef', 'ghi'], 1, 2, 'n']")]);
    p.lines("['abc def', 'ghi']");
    p.feed(1, 1, r#"\"_d\<LeftMouse>"#);
    p.expect(&[(text, "[['hi'], 1, 1, 'n']")]);
    p.lines("['abc def', 'ghi']");
    p.feed(0, 5, r#"\"_c\<LeftMouse>X\<Esc>"#);
    p.expect(&[(text, "[['Xef', 'ghi'], 1, 1, 'n']")]);
}

#[test]
fn a_click_in_insert_mode_moves_the_insert() {
    let sandbox = Sandbox::globals();
    let p = Pointer::new(&sandbox);
    let text = "[getline(1, '$'), line('.'), col('.'), mode()]";
    p.lines("['abc def', 'ghi']");
    p.feed(1, 1, r"iX\<LeftMouse>Y\<Esc>");
    p.expect(&[(text, "[['Xabc def', 'gYhi'], 2, 2, 'n']")]);
    p.lines("['abc def', 'ghi']");
    p.feed(0, 30, r"iX\<LeftMouse>Y\<Esc>");
    p.expect(&[(text, "[['Xabc defY', 'ghi'], 1, 9, 'n']")]);
    // The insert is two undo steps.
    p.fx.run("undo");
    p.expect(&[(text, "[['Xabc def', 'ghi'], 1, 8, 'n']")]);
    // A drag in Insert mode selects (CTRL-O v).
    p.lines("['abc def', 'ghi']");
    p.feed(0, 1, r"i\<LeftMouse>");
    p.feed(0, 5, r"\<LeftDrag>");
    p.expect(&[
        (text, "[['abc def', 'ghi'], 1, 6, 'v']"),
        ("getpos('v')[1:2]", "[1, 1]"),
    ]);
    p.escape();
    // Replace mode.
    p.lines("['abc def', 'ghi']");
    p.feed(1, 1, r"RX\<LeftMouse>Y\<Esc>");
    p.expect(&[(text, "[['Xbc def', 'gYi'], 2, 2, 'n']")]);
    // A double click in Insert mode selects the word.
    p.lines("['abc def', 'ghi']");
    p.feed(0, 5, r"i\<LeftMouse>\<2-LeftMouse>");
    p.expect(&[
        (text, "[['abc def', 'ghi'], 1, 6, 'v']"),
        ("getpos('v')[1:2]", "[1, 5]"),
    ]);
    p.escape();
}

#[test]
fn the_wheel_scrolls_by_mousescroll() {
    let sandbox = Sandbox::globals();
    let p = Pointer::new(&sandbox);
    let view = "[line('w0'), line('.'), winsaveview().leftcol, col('.')]";
    let long = "map(range(1, 100), {_, v -> repeat(v, 30)})";
    for (opt, keys, want) in [
        ("ver:3,hor:6", r"\<ScrollWheelDown>", "[4, 4, 0, 1]"),
        (
            "ver:3,hor:6",
            r"\<ScrollWheelDown>\<ScrollWheelDown>\<ScrollWheelUp>",
            "[4, 7, 0, 1]",
        ),
        ("ver:1", r"\<ScrollWheelDown>", "[2, 2, 0, 1]"),
        ("ver:0", r"\<ScrollWheelDown>", "[1, 1, 0, 1]"),
        ("ver:3,hor:6", r"\<S-ScrollWheelDown>", "[21, 21, 0, 1]"),
        (
            "ver:3,hor:6",
            r"\<C-ScrollWheelDown>\<S-ScrollWheelUp>",
            "[1, 22, 0, 1]",
        ),
        ("ver:3,hor:6", r"\<ScrollWheelRight>", "[1, 1, 6, 7]"),
        (
            "hor:2",
            r"\<ScrollWheelRight>\<ScrollWheelRight>\<ScrollWheelLeft>",
            "[1, 1, 2, 5]",
        ),
        ("ver:3,hor:6", r"\<S-ScrollWheelRight>", "[1, 10, 59, 60]"),
        ("ver:3,hor:6", r"\<ScrollWheelLeft>", "[1, 1, 0, 1]"),
        ("ver:3,hor:6", r"i\<ScrollWheelDown>\<Esc>", "[4, 4, 0, 1]"),
        ("ver:3,hor:6", r"i\<ScrollWheelRight>\<Esc>", "[1, 1, 6, 6]"),
        ("ver:3,hor:6", r"v\<ScrollWheelDown>\<Esc>", "[4, 4, 0, 1]"),
    ] {
        p.lines(long);
        p.fx.run("set nowrap");
        p.fx.run(&format!("set mousescroll={opt}"));
        p.feed(5, 5, keys);
        p.expect(&[(view, want)]);
        p.fx.run("set mousescroll&");
    }
    // Over the other window: that one scrolls.
    p.lines(long);
    p.fx.run("split | wincmd j");
    p.feed(2, 5, r"\<ScrollWheelDown>");
    p.expect(&[(
        "[winnr(), line('w0', win_getid(1)), line('w0', win_getid(2))]",
        "[2, 4, 1]",
    )]);
}

#[test]
fn a_status_line_or_separator_drag_resizes() {
    let sandbox = Sandbox::globals();
    let p = Pointer::new(&sandbox);
    let layout = "[winnr(), winheight(1), winheight(2), winwidth(1)]";
    p.lines(TEXT);
    p.fx.run("split");
    p.fx.run("redraw!");
    p.expect(&[(layout, "[1, 11, 10, 80]")]);
    // The top window's status line is at row winheight(1).
    p.fx.run("let g:stl = winheight(1)");
    let row: i32 = p.fx.eval("g:stl").parse().unwrap();
    p.feed(row, 10, r"\<LeftMouse>");
    p.feed(row - 4, 10, r"\<LeftDrag>");
    p.feed(row - 4, 10, r"\<LeftRelease>");
    p.expect(&[(layout, "[1, 7, 14, 80]")]);
    p.fx.run("only!");
    p.lines(TEXT);
    p.fx.run("vsplit");
    p.fx.run("redraw!");
    let col: i32 = p.fx.eval("winwidth(1)").parse().unwrap();
    p.feed(3, col, r"\<LeftMouse>");
    p.feed(3, col + 6, r"\<LeftDrag>");
    p.expect(&[(layout, "[1, 22, 22, 46]")]);
    // A click in the other window moves the focus there.
    p.feed(3, col + 10, r"\<LeftRelease>");
    p.feed(3, col + 10, r"\<LeftMouse>");
    p.expect(&[(layout, "[2, 22, 22, 46]")]);
}

#[test]
fn the_tab_line_and_the_fold_column_take_clicks() {
    let sandbox = Sandbox::globals();
    let p = Pointer::new(&sandbox);
    let tabs = "[tabpagenr(), tabpagenr('$')]";
    p.lines(TEXT);
    p.fx.run("tabnew | tabnew");
    // The labels' click regions are recorded when the tab line is drawn.
    p.fx.run("redraw!");
    p.expect(&[(tabs, "[3, 3]")]);
    // The first label.
    p.feed(0, 1, r"\<LeftMouse>");
    p.feed(0, 1, r"\<LeftRelease>");
    p.expect(&[(tabs, "[1, 3]")]);
    // Middle click closes the label's tab page.
    p.feed(0, 1, r"\<MiddleMouse>");
    p.expect(&[(tabs, "[1, 2]")]);
    // A double click past the labels opens a new one.
    p.feed(0, 60, r"\<2-LeftMouse>");
    p.expect(&[(tabs, "[3, 3]")]);
    p.fx.run("silent! tabonly!");
    // A click in the fold column opens and closes the fold.
    p.lines("['a', 'b', 'c', 'd']");
    p.fx.run("set foldcolumn=2 foldmethod=manual | 2,3fold");
    p.fx.run("redraw!");
    p.expect(&[("foldclosed(2)", "2")]);
    p.feed(1, 0, r"\<LeftMouse>");
    p.expect(&[("foldclosed(2)", "-1"), ("line('.')", "1")]);
    p.fx.run("redraw!");
    p.feed(1, 0, r"\<LeftMouse>");
    p.expect(&[("foldclosed(2)", "2"), ("line('.')", "1")]);
    p.fx.run("normal! zE");
    p.fx.run("set foldmethod&");
}

#[test]
fn a_click_on_the_command_line_moves_its_cursor() {
    let sandbox = Sandbox::globals();
    let p = Pointer::new(&sandbox);
    p.lines(TEXT);
    p.fx.run("silent! unlet g:x");
    // `:let g:x='abcdef'` -- screen column 10 is the `a`.
    for (col, keys, want) in [
        (10, r":let g:x='abcdef'\<LeftMouse>Z\<CR>", "'Zabcdef'"),
        (
            12,
            r":let g:x='abcdef'\<Home>\<LeftMouse>Z\<CR>",
            "'abZcdef'",
        ),
        (
            70,
            r":let g:x='abcdef'\<Home>\<LeftMouse>\<BS>Z'\<CR>",
            "'abcdefZ'",
        ),
        (
            10,
            r":let g:x='abcdef'\<Left>\<ScrollWheelDown>Z\<CR>",
            "'abcdefZ'",
        ),
        (
            10,
            r":let g:x='abcdef'\<Left>\<RightMouse>Z\<CR>",
            "'Zabcdef'",
        ),
        (
            10,
            r":let g:x='abcdef'\<Left>\<MiddleMouse>Z\<CR>",
            "'abcdefZ'",
        ),
    ] {
        p.fx.run("silent! unlet g:x");
        p.fx.run("call setreg('\"', 'R', 'c')");
        p.feed(23, col, keys);
        p.expect(&[("get(g:, 'x', 'none')", want)]);
    }
    p.fx.run("silent! unlet g:x");
}

/// `getmousepos()` with the window as its number: window ids are never reused,
/// so the id depends on every case before this one.
const POS: &str = "map(getmousepos(), {k, v -> k == 'winid' ? win_id2win(v) : v})";

#[test]
fn getmousepos_names_the_cell() {
    let sandbox = Sandbox::globals();
    let p = Pointer::new(&sandbox);
    p.lines(TEXT);
    for (row, col, want) in [
        (
            0,
            0,
            "{'screencol': 1, 'coladd': 0, 'screenrow': 1, 'column': 1, 'winrow': 1, 'line': 1, 'wincol': 1, 'winid': 1}",
        ),
        (
            4,
            9,
            "{'screencol': 10, 'coladd': 0, 'screenrow': 5, 'column': 3, 'winrow': 5, 'line': 5, 'wincol': 10, 'winid': 1}",
        ),
        (
            4,
            15,
            "{'screencol': 16, 'coladd': 0, 'screenrow': 5, 'column': 11, 'winrow': 5, 'line': 5, 'wincol': 16, 'winid': 1}",
        ),
        (
            15,
            3,
            "{'screencol': 4, 'coladd': 3, 'screenrow': 16, 'column': 1, 'winrow': 16, 'line': 5, 'wincol': 4, 'winid': 1}",
        ),
    ] {
        p.feed(row, col, "");
        p.expect(&[(POS, want)]);
    }
    p.fx.run("vsplit");
    p.feed(2, 50, "");
    p.expect(&[(POS, "{'screencol': 51, 'coladd': 0, 'screenrow': 3, 'column': 10, 'winrow': 3, 'line': 3, 'wincol': 10, 'winid': 2}")]);
}
