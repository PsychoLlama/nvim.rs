//! Sign placements: `:sign place`/`unplace`/`jump` and the listing, with
//! groups, ids and priorities; `sign_place()`/`sign_placelist()`/
//! `sign_unplace()`/`sign_unplacelist()` and the shapes they answer;
//! `sign_getplaced()` filtered by `group` (`'*'` included), `lnum` and `id`;
//! `getbufinfo()`'s `signs`; a redefinition reaching placed copies, and an
//! undefined sign reported as `[Deleted]`; `nvim_buf_set_extmark`'s
//! `sign_text`, which takes the text without unescaping it.
//!
//! In the unit lane because a placement is an extmark on a buffer line,
//! which the lib tests' fixture has no memline for; definitions alone are
//! covered there (`sign/tests.rs`). Written against the behaviour before a
//! definition owns its strings; a row that moves is a behaviour change.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

fn setup(fx: &Probe) {
    fx.home();
    fx.run("call sign_unplace('*') | call sign_undefine()");
    fx.run("call setline(1, ['one', 'two', 'three', 'four', 'five'])");
    fx.run("sign define Sa text=A texthl=Search");
    fx.run("sign define Sb text=中 linehl=Error priority=20");
    fx.run("sign define Sc text=\\ c numhl=Number");
}

#[test]
fn place_list_unplace_and_jump_by_command() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    setup(&fx);
    for (cmd, err) in [
        ("sign place 1 line=1 name=Sa", "none"),
        ("sign place 2 line=1 name=Sb", "none"),
        ("sign place 3 line=1 name=Sc priority=30", "none"),
        ("sign place 4 line=3 name=Sa group=G1", "none"),
        ("sign place 5 line=3 name=Sb group=G2 priority=5", "none"),
        ("sign place 6 line=99 name=Sc group=G1", "none"),
        ("sign place 07 line=2 name=0Sa", "none"),
        ("sign place 8 line=2 name=Nope", "E155: Unknown sign: Nope"),
        ("sign place 9 line=2 name=Sa group=*", "none"),
        ("sign place 10 name=Sa", "E474: Invalid argument"),
        ("sign place 1 name=Sb", "E474: Invalid argument"),
        (
            "sign place 77 name=Sb buffer=1",
            "E885: Not possible to change sign Sb",
        ),
        ("sign place line=2 name=Sa", "E474: Invalid argument"),
        (
            "sign place 11 line=2 name=Sa group=",
            "E474: Invalid argument",
        ),
        (
            "sign place 12 line=2 name=Sa buffer=99",
            "E158: Invalid buffer name: 99",
        ),
        (
            "sign place 13 line=2 name=Sa file=nosuch",
            "E158: Invalid buffer name: nosuch",
        ),
        (
            "sign place 14 line=2 name=Sa buffer=1 x",
            "E488: Trailing characters:  x",
        ),
        ("sign place 15 line=2 frob=1", "E474: Invalid argument"),
        ("sign place 3name=Sa", "E474: Invalid argument"),
    ] {
        fx.check_error(cmd, err);
    }
    fx.check_output("sign place", "\n--- Signs ---\nSigns for [NULL]:\n    line=1  id=3  name=Sc  priority=30\n    line=1  id=2  name=Sb  priority=20\n    line=1  id=1  name=Sa  priority=10\n    line=2  id=14  name=Sa  priority=10\n    line=2  id=7  name=Sa  priority=10");
    fx.check_output("sign place group=G1", "\n--- Signs ---\nSigns for [NULL]:\n    line=3  id=4  group=G1  name=Sa  priority=10\n    line=5  id=6  group=G1  name=Sc  priority=10");
    fx.check_output("sign place group=*", "\n--- Signs ---\nSigns for [NULL]:\n    line=1  id=3  name=Sc  priority=30\n    line=1  id=2  name=Sb  priority=20\n    line=1  id=1  name=Sa  priority=10\n    line=2  id=14  name=Sa  priority=10\n    line=2  id=7  name=Sa  priority=10\n    line=3  id=4  group=G1  name=Sa  priority=10\n    line=3  id=5  group=G2  name=Sb  priority=5\n    line=5  id=6  group=G1  name=Sc  priority=10");
    fx.check_output(
        "sign place group=Nope",
        "\n--- Signs ---\nSigns for [NULL]:",
    );
    fx.check_output("sign place buffer=1", "\n--- Signs ---\nSigns for [NULL]:\n    line=1  id=3  name=Sc  priority=30\n    line=1  id=2  name=Sb  priority=20\n    line=1  id=1  name=Sa  priority=10\n    line=2  id=14  name=Sa  priority=10\n    line=2  id=7  name=Sa  priority=10");
    fx.check("sign_getplaced(1, {'group': '*'})", "[{'signs': [{'lnum': 1, 'id': 3, 'name': 'Sc', 'priority': 30, 'group': ''}, {'lnum': 1, 'id': 2, 'name': 'Sb', 'priority': 20, 'group': ''}, {'lnum': 1, 'id': 1, 'name': 'Sa', 'priority': 10, 'group': ''}, {'lnum': 2, 'id': 14, 'name': 'Sa', 'priority': 10, 'group': ''}, {'lnum': 2, 'id': 7, 'name': 'Sa', 'priority': 10, 'group': ''}, {'lnum': 3, 'id': 4, 'name': 'Sa', 'priority': 10, 'group': 'G1'}, {'lnum': 3, 'id': 5, 'name': 'Sb', 'priority': 5, 'group': 'G2'}, {'lnum': 5, 'id': 6, 'name': 'Sc', 'priority': 10, 'group': 'G1'}], 'bufnr': 1}]");
    for (cmd, err) in [
        ("sign jump 4 group=G1", "none"),
        ("sign jump 99", "E157: Invalid sign ID: 99"),
        ("sign jump 4", "E157: Invalid sign ID: 4"),
        ("sign jump", "E471: Argument required"),
        ("sign jump 1 line=2", "E474: Invalid argument"),
        ("sign unplace 4 group=G1", "none"),
        ("sign unplace 4 group=G1", "none"),
        ("sign unplace 5 group=*", "none"),
        ("sign unplace 1 line=1", "E474: Invalid argument"),
        ("sign unplace 1 name=Sa", "E474: Invalid argument"),
        ("sign unplace 1 group=", "E474: Invalid argument"),
        ("sign unplace * 3", "E474: Invalid argument"),
        ("sign unplace 3 *", "E474: Invalid argument"),
    ] {
        fx.check_error(cmd, err);
    }
    fx.check("line('.')", "3");
    fx.check("sign_getplaced(1, {'group': '*'})", "[{'signs': [{'lnum': 1, 'id': 3, 'name': 'Sc', 'priority': 30, 'group': ''}, {'lnum': 1, 'id': 2, 'name': 'Sb', 'priority': 20, 'group': ''}, {'lnum': 1, 'id': 1, 'name': 'Sa', 'priority': 10, 'group': ''}, {'lnum': 2, 'id': 14, 'name': 'Sa', 'priority': 10, 'group': ''}, {'lnum': 2, 'id': 7, 'name': 'Sa', 'priority': 10, 'group': ''}, {'lnum': 5, 'id': 6, 'name': 'Sc', 'priority': 10, 'group': 'G1'}], 'bufnr': 1}]");
    // No id: the highest-priority sign on the cursor line goes.
    fx.run("1");
    fx.check_error("sign unplace", "none");
    fx.check("sign_getplaced(1, {'group': '*'})", "[{'signs': [{'lnum': 1, 'id': 2, 'name': 'Sb', 'priority': 20, 'group': ''}, {'lnum': 1, 'id': 1, 'name': 'Sa', 'priority': 10, 'group': ''}, {'lnum': 2, 'id': 14, 'name': 'Sa', 'priority': 10, 'group': ''}, {'lnum': 2, 'id': 7, 'name': 'Sa', 'priority': 10, 'group': ''}, {'lnum': 5, 'id': 6, 'name': 'Sc', 'priority': 10, 'group': 'G1'}], 'bufnr': 1}]");
    fx.run("2");
    fx.check_error("sign unplace", "none");
    fx.check_error("sign unplace * group=*", "none");
    fx.check(
        "sign_getplaced(1, {'group': '*'})",
        "[{'signs': [], 'bufnr': 1}]",
    );
    fx.check_error("sign unplace *", "none");
    fx.check(
        "sign_getplaced(1, {'group': '*'})",
        "[{'signs': [], 'bufnr': 1}]",
    );
    fx.run("call sign_unplace('*') | call sign_undefine()");
    fx.home();
}

#[test]
fn place_and_query_by_function() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    setup(&fx);
    fx.expect_all(&[
        ("sign_place(0, '', 'Sa', 1, {'lnum': 2})", "1"),
        ("sign_place(0, '', 'Sb', '%', {'lnum': 2})", "2"),
        ("sign_place(5, 'G1', 'Sc', 1, {'lnum': 4, 'priority': 40})", "5"),
        ("sign_place(5, 'G1', 'Sa', 1)", "5"),
        ("sign_place(6, 'G1', 'Sa', 1, {'lnum': 0})", "-1"),
        ("sign_place(-1, '', 'Sa', 1, {'lnum': 1})", "-1"),
        ("sign_place(7, '', 'Nope', 1, {'lnum': 1})", "-1"),
        ("sign_place(7, '', 'Sa', 99, {'lnum': 1})", "-1"),
        ("sign_place(7, '*', 'Sa', 1, {'lnum': 1})", "-1"),
        ("sign_place(7, [], 'Sa', 1, {'lnum': 1})", "-1"),
        ("sign_place(7, '', 'Sa', 1, [])", "-1"),
        ("sign_placelist([{'id': 8, 'name': 'Sa', 'buffer': 1, 'lnum': 3}, {'name': 'Sb', 'buffer': 1, 'lnum': 3, 'group': 'G2', 'priority': 2}, {'name': 'Nope', 'buffer': 1, 'lnum': 1}, 4, {'buffer': 1}, {'name': 'Sa'}, {'id': -2, 'name': 'Sa', 'buffer': 1, 'lnum': 1}])", "[8, 1, -1, -1, -1, -1, -1]"),
        ("sign_placelist('x')", "[]"),
        ("sign_getplaced()", "[{'signs': [{'lnum': 2, 'id': 2, 'name': 'Sb', 'priority': 20, 'group': ''}, {'lnum': 2, 'id': 1, 'name': 'Sa', 'priority': 10, 'group': ''}, {'lnum': 3, 'id': 8, 'name': 'Sa', 'priority': 10, 'group': ''}], 'bufnr': 1}]"),
        ("sign_getplaced(1)", "[{'signs': [{'lnum': 2, 'id': 2, 'name': 'Sb', 'priority': 20, 'group': ''}, {'lnum': 2, 'id': 1, 'name': 'Sa', 'priority': 10, 'group': ''}, {'lnum': 3, 'id': 8, 'name': 'Sa', 'priority': 10, 'group': ''}], 'bufnr': 1}]"),
        ("sign_getplaced(1, {'group': '*'})", "[{'signs': [{'lnum': 2, 'id': 2, 'name': 'Sb', 'priority': 20, 'group': ''}, {'lnum': 2, 'id': 1, 'name': 'Sa', 'priority': 10, 'group': ''}, {'lnum': 3, 'id': 8, 'name': 'Sa', 'priority': 10, 'group': ''}, {'lnum': 3, 'id': 1, 'name': 'Sb', 'priority': 2, 'group': 'G2'}, {'lnum': 4, 'id': 5, 'name': 'Sa', 'priority': 10, 'group': 'G1'}], 'bufnr': 1}]"),
        ("sign_getplaced(1, {'group': 'G1'})", "[{'signs': [{'lnum': 4, 'id': 5, 'name': 'Sa', 'priority': 10, 'group': 'G1'}], 'bufnr': 1}]"),
        ("sign_getplaced(1, {'group': 'Nope'})", "[{'signs': [], 'bufnr': 1}]"),
        ("sign_getplaced(1, {'lnum': 3, 'group': '*'})", "[{'signs': [{'lnum': 3, 'id': 8, 'name': 'Sa', 'priority': 10, 'group': ''}, {'lnum': 3, 'id': 1, 'name': 'Sb', 'priority': 2, 'group': 'G2'}], 'bufnr': 1}]"),
        ("sign_getplaced(1, {'id': 8})", "[{'signs': [{'lnum': 3, 'id': 8, 'name': 'Sa', 'priority': 10, 'group': ''}], 'bufnr': 1}]"),
        ("sign_getplaced(1, {'id': 5, 'group': 'G1', 'lnum': 4})", "[{'signs': [{'lnum': 4, 'id': 5, 'name': 'Sa', 'priority': 10, 'group': 'G1'}], 'bufnr': 1}]"),
        ("sign_getplaced(1, {'lnum': 0})", "[]"),
        ("sign_getplaced(99)", "[]"),
        ("sign_getplaced(1, 3)", "[]"),
        ("getbufinfo(1)[0].signs", "[{'lnum': 2, 'id': 1, 'name': 'Sa', 'priority': 10, 'group': ''}, {'lnum': 2, 'id': 2, 'name': 'Sb', 'priority': 20, 'group': ''}, {'lnum': 3, 'id': 8, 'name': 'Sa', 'priority': 10, 'group': ''}, {'lnum': 3, 'id': 1, 'name': 'Sb', 'priority': 2, 'group': 'G2'}, {'lnum': 4, 'id': 5, 'name': 'Sa', 'priority': 10, 'group': 'G1'}]"),
        ("sign_jump(5, 'G1', '')", "4"),
        ("line('.')", "4"),
        ("sign_jump(0, '', '')", "-1"),
        ("sign_jump(99, '', '')", "-1"),
        ("sign_unplace('G1', {'id': 5})", "0"),
        ("sign_unplace('G1', {'id': 5})", "-1"),
        ("sign_unplace('', {'id': 0})", "-1"),
        ("sign_unplace('', {'buffer': 99})", "-1"),
        ("sign_unplacelist([{'id': 8}, {'id': 1, 'buffer': 1}, {'group': 'G2'}, 3, {'id': 99}])", "[0, 0, 0, -1, -1]"),
        ("sign_getplaced(1, {'group': '*'})", "[{'signs': [{'lnum': 2, 'id': 2, 'name': 'Sb', 'priority': 20, 'group': ''}], 'bufnr': 1}]"),
        ("sign_unplace('*')", "0"),
        ("sign_getplaced(1, {'group': '*'})", "[{'signs': [], 'bufnr': 1}]"),
    ]);
    fx.run("call sign_unplace('*') | call sign_undefine()");
    fx.home();
}

/// A placement carries a copy of its definition: redefining reaches every
/// copy, undefining leaves them in place under `[Deleted]`.
#[test]
fn placements_follow_redefinition() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    setup(&fx);
    fx.run("sign place 1 line=1 name=Sa");
    fx.run("sign place 2 line=2 name=Sa group=G1");
    fx.run("call nvim_buf_set_extmark(0, nvim_create_namespace('ext'), 2, 0, {'sign_text': '\\x', 'sign_hl_group': 'Error'})");
    fx.run("sign define Sa text=é linehl=Visual culhl=Search");
    let details = "map(nvim_buf_get_extmarks(0, -1, 0, -1, {'details': 1, 'type': 'sign'}), 'filter(v:val[3], \"v:key !=# ''ns_id''\")')";
    fx.check(details, "[{'sign_name': 'Sa', 'invalidate': v:true, 'cursorline_hl_group': 'Search', 'priority': 10, 'line_hl_group': 'Visual', 'sign_text': 'é ', 'right_gravity': v:true, 'sign_hl_group': 'Search', 'undo_restore': v:false}, {'sign_name': 'Sa', 'invalidate': v:true, 'cursorline_hl_group': 'Search', 'priority': 10, 'line_hl_group': 'Visual', 'sign_text': 'é ', 'right_gravity': v:true, 'sign_hl_group': 'Search', 'undo_restore': v:false}, {'priority': 4096, 'sign_text': '\\x', 'right_gravity': v:true, 'sign_hl_group': 'Error'}]");
    fx.check("sign_getplaced(1, {'group': '*'})", "[{'signs': [{'lnum': 1, 'id': 1, 'name': 'Sa', 'priority': 10, 'group': ''}, {'lnum': 2, 'id': 2, 'name': 'Sa', 'priority': 10, 'group': 'G1'}, {'lnum': 3, 'id': 1, 'name': '', 'priority': 4096, 'group': 'ext'}], 'bufnr': 1}]");
    fx.run("sign undefine Sa");
    fx.check_output("sign place group=*", "\n--- Signs ---\nSigns for [NULL]:\n    line=1  id=1  name=[Deleted]  priority=10\n    line=2  id=2  group=G1  name=[Deleted]  priority=10\n    line=3  id=1  group=ext  priority=4096");
    fx.check("sign_getplaced(1, {'group': '*'})", "[{'signs': [{'lnum': 1, 'id': 1, 'name': '[Deleted]', 'priority': 10, 'group': ''}, {'lnum': 2, 'id': 2, 'name': '[Deleted]', 'priority': 10, 'group': 'G1'}, {'lnum': 3, 'id': 1, 'name': '', 'priority': 4096, 'group': 'ext'}], 'bufnr': 1}]");
    fx.check_error(
        "call nvim_buf_set_extmark(0, nvim_create_namespace('ext'), 0, 0, {'sign_text': 'abc'})",
        "E5555: API call: Invalid ''sign_text",
    );
    fx.check_error(
        "call nvim_buf_set_extmark(0, nvim_create_namespace('ext'), 0, 0, {'sign_text': \"\\t\"})",
        "E5555: API call: Invalid ''sign_text",
    );
    fx.check_error(
        "call nvim_buf_set_extmark(0, nvim_create_namespace('ext'), 0, 0, {'sign_text': '中'})",
        "none",
    );
    fx.check(details, "[{'sign_name': 'Sa', 'invalidate': v:true, 'cursorline_hl_group': 'Search', 'priority': 10, 'line_hl_group': 'Visual', 'sign_text': 'é ', 'right_gravity': v:true, 'sign_hl_group': 'Search', 'undo_restore': v:false}, {'priority': 4096, 'sign_text': '中', 'right_gravity': v:true}, {'sign_name': 'Sa', 'invalidate': v:true, 'cursorline_hl_group': 'Search', 'priority': 10, 'line_hl_group': 'Visual', 'sign_text': 'é ', 'right_gravity': v:true, 'sign_hl_group': 'Search', 'undo_restore': v:false}, {'priority': 4096, 'sign_text': '\\x', 'right_gravity': v:true, 'sign_hl_group': 'Error'}]");
    fx.run("call nvim_buf_clear_namespace(0, -1, 0, -1)");
    fx.run("call sign_unplace('*') | call sign_undefine()");
    fx.home();
}
