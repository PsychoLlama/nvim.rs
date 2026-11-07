//! Extmarks across text changes: the `extmark_splice()` arithmetic and its
//! undo.
//!
//! Every buffer change reaches `extmark/splice.rs` as a start position and
//! the extents removed and inserted, each a row, a byte column and a byte
//! count. `on_bytes` is handed exactly those nine numbers, so the first case
//! pins them for each kind of change — a line append and delete
//! (`extmark_adjust`), a charwise insert and delete with a multibyte
//! character (`extmark_splice`), a join, both directions of `:move`
//! (`extmark_move_region`, reported as a delete and an insert), Normal-mode
//! edits, and deleting every line. The others pin where marks land after
//! each kind of change and that undo and redo put them back
//! (`extmark/undo.rs`): a deleted mark's saved position, an invalidated
//! mark and its revalidation, a region moved back. The oracle for retyping
//! the rows and columns these carry.

#![cfg(not(miri))]

use crate::support::{Probe, editor_lock};

/// Each mark as `id:row,col`, `!` when invalid, in buffer order.
const MARKS: &str = "luaeval('vim.tbl_map(function(m) \
     return m[1] .. \":\" .. m[2] .. \",\" .. m[3] .. (m[4].invalid and \"!\" or \"\") \
     end, vim.api.nvim_buf_get_extmarks(0, _G.extmark_ns, 0, -1, {details = true}))')";

/// Four lines, a break in the undo sequence, and the namespace the marks go in.
fn setup(fx: &Probe) {
    fx.home();
    fx.run("call setline(1, ['aaaa', 'bbbb', 'cccc', 'dddd'])");
    fx.run("let &undolevels = &undolevels");
    fx.run("lua _G.extmark_ns = vim.api.nvim_create_namespace('unit_extmark')");
}

/// Run `cmd` and close the undo step it made.
fn step(fx: &Probe, cmd: &str) -> String {
    fx.run(&format!("silent {cmd}"));
    fx.run("let &undolevels = &undolevels");
    fx.eval(MARKS)
}

fn teardown(fx: &Probe) {
    fx.run("lua vim.api.nvim_buf_clear_namespace(0, _G.extmark_ns, 0, -1)");
    fx.home();
}

#[test]
fn on_bytes_is_told_the_start_and_both_extents_of_each_change() {
    let editor = editor_lock();
    let fx = Probe::new(&editor);
    fx.home();
    fx.run("call setline(1, ['aa', 'bbb', 'c'])");
    // The buffer must be loaded to attach; the callback detaches itself
    // once `_G.unit_bytes` is gone.
    fx.run(
        "lua _G.unit_bytes = {}
         vim.api.nvim_buf_attach(0, false, { on_bytes = function(_, _, _, ...)
           if not _G.unit_bytes then return true end
           table.insert(_G.unit_bytes, table.concat({ ... }, ','))
         end })",
    );
    // start row,col,byte; old end row,col,byte; new end row,col,byte.
    let rows: &[(&str, &str, &str)] = &[
        (
            "call append(1, ['x', 'yy'])",
            "['1,0,3,0,0,0,2,0,5']",
            "['aa', 'x', 'yy', 'bbb', 'c']",
        ),
        ("2,3delete", "['1,0,3,2,0,5,0,0,0']", "['aa', 'bbb', 'c']"),
        (
            "call nvim_buf_set_text(0, 0, 1, 0, 1, ['XY'])",
            "['0,1,1,0,0,0,0,2,2']",
            "['aXYa', 'bbb', 'c']",
        ),
        (
            "call nvim_buf_set_text(0, 0, 1, 0, 3, [])",
            "['0,1,1,0,2,2,0,0,0']",
            "['aa', 'bbb', 'c']",
        ),
        (
            "call nvim_buf_set_text(0, 1, 0, 1, 0, ['\u{e9}'])",
            "['1,0,3,0,0,0,0,2,2']",
            "['aa', '\u{e9}bbb', 'c']",
        ),
        ("1,2join", "['0,2,2,1,0,1,0,1,1']", "['aa \u{e9}bbb', 'c']"),
        (
            "2move 0",
            "['1,0,9,1,0,2,0,0,0', '0,0,0,0,0,0,1,0,2']",
            "['c', 'aa \u{e9}bbb']",
        ),
        (
            "1move 2",
            "['0,0,0,1,0,2,0,0,0', '1,0,9,0,0,0,1,0,2']",
            "['aa \u{e9}bbb', 'c']",
        ),
        (
            "normal! ggx",
            "['0,0,0,0,1,1,0,0,0']",
            "['a \u{e9}bbb', 'c']",
        ),
        (
            "normal! ggix",
            "['0,0,0,0,0,0,0,1,1']",
            "['xa \u{e9}bbb', 'c']",
        ),
        ("normal! 0D", "['0,0,0,0,8,8,0,0,0']", "['', 'c']"),
        // The empty line left behind is reported as inserted.
        ("%delete", "['0,0,0,2,0,3,1,0,1']", "['']"),
    ];
    for (cmd, bytes, lines) in rows {
        fx.run("lua _G.unit_bytes = {}");
        fx.run(cmd);
        assert_eq!(fx.eval("luaeval('_G.unit_bytes')"), *bytes, "{cmd}");
        assert_eq!(fx.eval("getline(1, '$')"), *lines, "{cmd}");
    }
    fx.run("lua _G.unit_bytes = nil");
    fx.home();
}

#[test]
fn deleted_lines_pull_marks_to_the_cut_and_undo_puts_them_back() {
    let editor = editor_lock();
    let fx = Probe::new(&editor);
    setup(&fx);
    fx.run(
        "lua for id, at in ipairs({ {0, 2}, {1, 1}, {1, 3}, {2, 0}, {3, 4} }) do
           vim.api.nvim_buf_set_extmark(0, _G.extmark_ns, at[1], at[2], { id = id })
         end
         vim.api.nvim_buf_set_extmark(0, _G.extmark_ns, 2, 2, { id = 9, invalidate = true })
         vim.api.nvim_buf_set_extmark(0, _G.extmark_ns, 1, 0,
           { id = 10, end_row = 2, end_col = 1, invalidate = true })",
    );
    let before = "['1:0,2', '10:1,0', '2:1,1', '3:1,3', '4:2,0', '9:2,2', '5:3,4']";
    assert_eq!(fx.eval(MARKS), before);
    // Marks in the deleted lines land at the start of the line after; an
    // `invalidate` mark there, and a pair whose whole range went, turn invalid.
    let after = "['1:0,2', '2:1,0', '3:1,0', '4:1,0', '10:1,0!', '9:1,0!', '5:1,4']";
    assert_eq!(step(&fx, "2,3delete"), after);
    assert_eq!(step(&fx, "undo"), before, "every saved position comes back");
    assert_eq!(step(&fx, "redo"), after);
    assert_eq!(step(&fx, "undo"), before);
    teardown(&fx);
}

#[test]
fn a_charwise_change_shifts_marks_by_bytes_and_undo_shifts_them_back() {
    let editor = editor_lock();
    let fx = Probe::new(&editor);
    setup(&fx);
    fx.run(
        "lua for id, at in ipairs({ {1, 0}, {1, 1}, {1, 2}, {1, 3}, {2, 1} }) do
           vim.api.nvim_buf_set_extmark(0, _G.extmark_ns, at[1], at[2], { id = id })
         end",
    );
    let before = "['1:1,0', '2:1,1', '3:1,2', '4:1,3', '5:2,1']";
    // Deleting columns 1..3: the marks inside move to the start of the cut.
    assert_eq!(
        step(&fx, "call nvim_buf_set_text(0, 1, 1, 1, 3, [])"),
        "['1:1,0', '2:1,1', '3:1,1', '4:1,1', '5:2,1']"
    );
    assert_eq!(step(&fx, "undo"), before);
    // Two bytes of a multibyte character in front of them all.
    assert_eq!(
        step(&fx, "call nvim_buf_set_text(0, 1, 0, 1, 0, ['\u{e9}'])"),
        "['1:1,2', '2:1,3', '3:1,4', '4:1,5', '5:2,1']"
    );
    assert_eq!(step(&fx, "undo"), before);
    // A line break inserted at column 2 carries the marks after it down.
    assert_eq!(
        step(&fx, "call nvim_buf_set_text(0, 1, 2, 1, 2, ['', ''])"),
        "['1:1,0', '2:1,1', '3:2,0', '4:2,1', '5:3,1']"
    );
    assert_eq!(step(&fx, "undo"), before);
    teardown(&fx);
}

#[test]
fn a_moved_line_takes_its_marks_along_and_undo_moves_them_back() {
    let editor = editor_lock();
    let fx = Probe::new(&editor);
    setup(&fx);
    fx.run(
        "lua for id, at in ipairs({ {0, 2}, {1, 1}, {2, 0}, {2, 2}, {3, 4} }) do
           vim.api.nvim_buf_set_extmark(0, _G.extmark_ns, at[1], at[2], { id = id })
         end",
    );
    let before = "['1:0,2', '2:1,1', '3:2,0', '4:2,2', '5:3,4']";
    assert_eq!(
        step(&fx, "3move 0"),
        "['3:0,0', '4:0,2', '1:1,2', '2:2,1', '5:3,4']"
    );
    assert_eq!(
        fx.eval("getline(1, '$')"),
        "['cccc', 'aaaa', 'bbbb', 'dddd']"
    );
    assert_eq!(step(&fx, "undo"), before);
    assert_eq!(
        step(&fx, "1move 3"),
        "['2:0,1', '3:1,0', '4:1,2', '1:2,2', '5:3,4']"
    );
    assert_eq!(step(&fx, "undo"), before);
    teardown(&fx);
}
