//! The mark stores as the editor drives them: the change marks a put
//! leaves, the jump list at its hundred-entry limit (with entries that name
//! a file rather than a buffer), a global mark whose file is not loaded, and
//! marks read out of a ShaDa file — which carry extra data the stores own —
//! under `:move`, undo and `:delmarks`.
//!
//! Written against the stores as fixed arrays of records holding raw
//! allocations; the expectations are what that code answered.

#![cfg(not(miri))]

use crate::shada::{bin, entry_bytes, map};
use crate::support::{Probe, Sandbox};

/// A hand-packed ShaDa file of `jumps` jump entries, the `i`th naming file
/// `/nonexistent/f<i>` at line `i + 1`, stamped oldest first.
fn jump_file(jumps: u32) -> Vec<u8> {
    let mut file = Vec::new();
    for i in 0..jumps {
        let name = format!("/nonexistent/f{i}");
        let line = u8::try_from(i % 100 + 1).expect("a fixint line");
        file.extend(entry_bytes(
            8,
            1_000_000 + i,
            &map(&[(b"f", bin(name.as_bytes())), (b"l", vec![line])]),
        ));
    }
    file
}

fn restore(fx: &Probe) {
    fx.run("silent! clearjumps");
    fx.home();
    fx.run("set shada&");
}

#[test]
fn a_put_leaves_the_change_marks_around_what_it_put() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.home();
    fx.run("call setline(1, ['one', 'two', 'three', 'four'])");
    fx.run("normal! 2Gyyp");
    fx.expect_all(&[
        ("getpos(\"'[\")", "[0, 3, 1, 0]"),
        ("getpos(\"']\")", "[0, 3, 3, 0]"),
    ]);
    fx.run("normal! 1Gy2jGp");
    fx.expect_all(&[
        ("getpos(\"'[\")", "[0, 6, 1, 0]"),
        ("getpos(\"']\")", "[0, 8, 3, 0]"),
        ("line('$')", "8"),
    ]);
    // Blockwise: two columns of two lines, put after the cursor.
    fx.run("execute \"normal! 1G\\<C-v>jly3G$p\"");
    fx.expect_all(&[
        ("getpos(\"'[\")", "[0, 3, 4, 0]"),
        ("getpos(\"']\")", "[0, 4, 5, 0]"),
        ("getline(3, 4)", "['twoon', 'thrtwee']"),
    ]);
    // Charwise `P` before the cursor.
    fx.run("normal! 1G0ywGP");
    fx.expect_all(&[
        ("getpos(\"'[\")", "[0, 8, 1, 0]"),
        ("getpos(\"']\")", "[0, 8, 3, 0]"),
    ]);
    restore(&fx);
}

#[test]
fn the_jump_list_keeps_the_newest_hundred() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.home();
    fx.run("call setline(1, range(1, 150))");
    fx.run("clearjumps");
    fx.run("for i in range(1, 130) | execute 'normal! ' .. i .. 'G' | endfor");
    fx.expect_all(&[
        ("len(getjumplist()[0])", "100"),
        ("getjumplist()[1]", "100"),
        ("getjumplist()[0][0].lnum", "30"),
        ("getjumplist()[0][99].lnum", "129"),
    ]);
    // Back three, then a new jump: the list is full, so the oldest goes.
    fx.run("execute \"normal! 3\\<C-o>\"");
    fx.expect_all(&[("line('.')", "127"), ("getjumplist()[1]", "96")]);
    fx.run("normal! 140G");
    fx.expect_all(&[
        ("len(getjumplist()[0])", "99"),
        ("getjumplist()[0][0].lnum", "32"),
        ("getjumplist()[0][98].lnum", "127"),
    ]);
    // A split copies the list; closing the copy frees it, the original
    // keeps its own.
    fx.run("split");
    fx.expect_all(&[
        ("len(getjumplist()[0])", "99"),
        ("getjumplist()[0][50].lnum", "82"),
    ]);
    fx.run("close");
    fx.expect_all(&[("len(getjumplist()[0])", "99")]);
    // `:5,40d` leaves duplicates for the cleanup to drop.
    fx.run("silent 5,40delete _");
    fx.expect_all(&[
        ("len(getjumplist()[0])", "91"),
        ("getjumplist()[0][0].lnum", "5"),
        ("getjumplist()[0][90].lnum", "104"),
    ]);
    fx.run("clearjumps");
    fx.check("getjumplist()", "[[], 0]");
    restore(&fx);
}

#[test]
fn jumps_read_from_shada_keep_their_file_names() {
    let sandbox = Sandbox::dir("mark_shada_jumps");
    let fx = Probe::new(sandbox.editor());
    fx.home();
    fx.run("clearjumps");
    let file = sandbox.write("jumps.shada", &jump_file(105));
    fx.run("set shada='100,f1");
    fx.check_error(&format!("rshada! {}", file.display()), "none");
    // Five of the hundred and five do not fit; the oldest are dropped.
    let jumps = fx.output("jumps");
    let rows: Vec<&str> = jumps.lines().collect();
    fx.check("len(getjumplist()[0])", "100");
    assert_eq!(rows.len(), 103, "{jumps}");
    assert_eq!(rows[2], "  100     6    0 /nonexistent/f5");
    assert_eq!(rows[101], "   1     5    0 /nonexistent/f104");
    // Three jumps of our own push three more of the file's out, freeing
    // the names they carried.
    fx.run("call setline(1, range(1, 20))");
    fx.run("normal! 5G10G15G");
    let jumps = fx.output("jumps");
    let rows: Vec<&str> = jumps.lines().collect();
    assert_eq!(rows[2], "  100     9    0 /nonexistent/f8");
    // A split copies the names, closing it frees the copies.
    fx.run("split");
    fx.check("len(getjumplist()[0])", "100");
    fx.run("close");
    fx.run("clearjumps");
    fx.check("getjumplist()", "[[], 0]");
    restore(&fx);
}

#[test]
fn a_global_mark_in_a_file_that_is_not_loaded() {
    let sandbox = Sandbox::dir("mark_global_unloaded");
    let fx = Probe::new(sandbox.editor());
    fx.home();
    fx.run("delmarks A-Z0-9");
    let file = entry_bytes(
        7,
        1_000_000,
        &map(&[
            (b"n", vec![b'G']),
            (b"f", bin(b"/nonexistent/other")),
            (b"l", vec![0x02]),
            (b"c", vec![0x03]),
            (b"zz", vec![0x07]),
        ]),
    );
    let path = sandbox.write("global.shada", &file);
    fx.run("set shada='100,f1");
    fx.check_error(&format!("rshada! {}", path.display()), "none");
    fx.expect_all(&[
        ("nvim_get_mark('G', {})", "[2, 3, 0, '/nonexistent/other']"),
        // `getpos()` loads the file; buffer numbers are never reused, so
        // only the match is checked.
        ("getpos(\"'G\")[1:]", "[2, 4, 0]"),
        ("getpos(\"'G\")[0] == bufnr('/nonexistent/other')", "1"),
        (
            "getmarklist()->filter({_, m -> m.mark ==# \"'G\"})\
             ->map({_, m -> [m.mark, m.file, m.pos[1:]]})",
            "[['''G', '/nonexistent/other', [2, 4, 0]]]",
        ),
        ("bufnr('/nonexistent/other') > 0", "1"),
    ]);
    fx.check_output(
        "marks G",
        "\nmark line  col file/text\n G      2    3 /nonexistent/other",
    );
    // Setting it over frees the name the file gave it.
    fx.run("call setline(1, ['a', 'b'])");
    fx.run("normal! 2GmG");
    fx.expect_all(&[
        ("nvim_get_mark('G', {})[0:2]", "[2, 0, 1]"),
        ("getpos(\"'G\")", "[1, 2, 1, 0]"),
    ]);
    fx.check_error(&format!("rshada! {}", path.display()), "none");
    fx.run("delmarks G");
    fx.check("nvim_get_mark('G', {})", "[0, 0, 0, '']");
    restore(&fx);
}

#[test]
fn local_marks_and_the_change_list_under_edits() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.home();
    // Nothing runs the main loop between these commands, so nothing would
    // close an undo block: `sync` does, as a typed key would.
    let sync = || fx.run("let &undolevels = &undolevels");
    fx.run("call setline(1, map(range(1, 10), '\"l\" .. v:val'))");
    sync();
    fx.run("normal! 3Gma6Gmb9GmA");
    fx.run("normal! 2Gix\u{1b}");
    sync();
    fx.run("normal! 7Giy\u{1b}");
    sync();
    fx.run("4,6move 9");
    sync();
    fx.expect_all(&[
        ("line(\"'a\")", "3"),
        ("line(\"'b\")", "9"),
        ("line(\"'A\")", "6"),
        ("map(getchangelist()[0], 'v:val.lnum')", "[2, 4, 4]"),
        ("nvim_buf_get_mark(0, 'b')", "[9, 0]"),
    ]);
    fx.run("undo");
    fx.expect_all(&[
        ("line(\"'b\")", "6"),
        ("line(\"'A\")", "9"),
        ("nvim_buf_get_mark(0, 'a')", "[3, 0]"),
    ]);
    fx.run("silent 2,3delete _");
    sync();
    fx.expect_all(&[
        ("line(\"'a\")", "0"),
        ("nvim_buf_get_mark(0, 'a')", "[0, 0]"),
        ("line(\"'b\")", "4"),
    ]);
    fx.run("undo");
    fx.expect_all(&[("line(\"'a\")", "3"), ("line(\"'b\")", "6")]);
    fx.run("redo");
    fx.expect_all(&[("line(\"'a\")", "0"), ("line(\"'b\")", "4")]);
    fx.run("undo");
    fx.check("nvim_buf_del_mark(0, 'b')", "v:true");
    fx.check("nvim_buf_del_mark(0, 'b')", "v:false");
    fx.check_error("call nvim_buf_set_mark(0, 'c', 5, 1, {})", "none");
    fx.check("getpos(\"'c\")", "[0, 5, 2, 0]");
    fx.check_output(
        "changes",
        "\nchange line  col text\n    4     4    0 l4\n    3     7    0 yl7\n    2    10    0 l10\n    1     2    0 xl2\n>",
    );
    fx.check_output(
        "marks aAc",
        "\nmark line  col file/text\n a      3    0 l3\n c      5    1 l5\n A      9    0 l9",
    );
    fx.run("delmarks A");
    restore(&fx);
}
