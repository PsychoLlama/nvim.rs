//! Menus end to end, through the commands and functions that keep them:
//! definition per mode, submenus, priorities and the order they give,
//! accelerators and mnemonics, `menu_get()`'s and `menu_info()`'s shapes,
//! the `:menu` listing, `:menu enable`/`disable`, deletion per mode and of
//! whole subtrees, and `:menutranslate`.
//!
//! Written against the behaviour the tree has before its nodes own their
//! strings and children; a row that moves is a behaviour change.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::test_fixture::Fixture;

/// Every menu in every mode index, and every translation, gone.
fn clean(fx: &Fixture) {
    for cmd in [
        "aunmenu *",
        "aunmenu! *",
        "tlunmenu *",
        "tunmenu *",
        "menutranslate clear",
    ] {
        fx.run(&format!("silent! {cmd}"));
    }
}

/// The names `menu_get()` answers at the top level, in order.
const TOPS: &str = "map(menu_get(''), 'v:val.name')";

#[test]
fn menus_are_defined_found_and_listed_per_mode() {
    let fx = Fixture::new();
    clean(&fx);
    for cmd in [
        "menu 10.10 &File.&Open<Tab>CTRL-O :echo 'open'<CR>",
        "menu 10.20 File.&Save :echo 'save'<CR>",
        "menu 10.30 File.-sep1- <Nop>",
        "imenu 20.10 Edit.OnlyInsert insert-rhs",
        "vmenu 20.20 Edit.OnlyVisual visual-rhs",
        "nmenu <silent> 30.10 Tools.Quiet :echo 'quiet'<CR>",
        "nnoremenu <script> 30.20 Tools.Scripted :echo 'scripted'<CR>",
        "menu 40.10.10 Deep.Sub.Leaf leaf",
        "menu ]Hidden.Item hidden",
        "tlmenu Term.Item term-rhs",
        "cmenu Cmd.Item cmd-rhs",
        "tmenu File.Open Open a file",
    ] {
        fx.run(cmd);
    }
    fx.expect_all(&[
        (TOPS, "['File', 'Edit', 'Tools', 'Deep', 'Term', 'Cmd', ']Hidden']"),
        ("map(menu_get('', 'i'), 'v:val.name')", "['Edit']"),
        ("map(menu_get('', 'v'), 'v:val.name')", "['File', 'Edit', 'Deep', ']Hidden']"),
        ("map(menu_get('', 'c'), 'v:val.name')", "['Cmd']"),
        ("map(menu_get('', 'tl'), 'v:val.name')", "['Term']"),
        ("map(menu_get('File'), {_, m -> map(m.submenus, 'v:val.name')})", "[['Open', 'Save', '-sep1-']]"),
        ("menu_get('File.Save')", "[{'hidden': 0, 'name': 'Save', 'priority': 20, 'shortcut': 'S', 'mappings': {'s': {'noremap': 0, 'enabled': 1, 'silent': 0, 'sid': 0, 'rhs': ':echo ''save''<CR>'}, 'v': {'noremap': 0, 'enabled': 1, 'silent': 0, 'sid': 0, 'rhs': ':echo ''save''<CR>'}, 'n': {'noremap': 0, 'enabled': 1, 'silent': 0, 'sid': 0, 'rhs': ':echo ''save''<CR>'}, 'o': {'noremap': 0, 'enabled': 1, 'silent': 0, 'sid': 0, 'rhs': ':echo ''save''<CR>'}}}]"),
        ("menu_get('Deep')", "[{'hidden': 0, 'submenus': [{'hidden': 0, 'submenus': [{'hidden': 0, 'name': 'Leaf', 'priority': 10, 'mappings': {'s': {'noremap': 0, 'enabled': 1, 'silent': 0, 'sid': 0, 'rhs': 'leaf'}, 'v': {'noremap': 0, 'enabled': 1, 'silent': 0, 'sid': 0, 'rhs': 'leaf'}, 'n': {'noremap': 0, 'enabled': 1, 'silent': 0, 'sid': 0, 'rhs': 'leaf'}, 'o': {'noremap': 0, 'enabled': 1, 'silent': 0, 'sid': 0, 'rhs': 'leaf'}}}], 'name': 'Sub', 'priority': 10}], 'name': 'Deep', 'priority': 40}]"),
        ("menu_get('NoSuch')", "[]"),
        ("menu_info('File.Open')", "{'modes': ' ', 'shortcut': 'O', 'name': '&Open\tCTRL-O', 'priority': 10, 'script': v:false, 'accel': 'CTRL-O', 'enabled': v:true, 'silent': v:false, 'rhs': ':echo ''open''<CR>', 'display': 'Open', 'noremenu': v:false}"),
        ("menu_info('File.Open', 'i')", "{}"),
        ("menu_info('Edit.OnlyInsert', 'i')", "{'modes': 'i', 'script': v:false, 'name': 'OnlyInsert', 'priority': 10, 'enabled': v:true, 'shortcut': '', 'silent': v:false, 'rhs': 'insert-rhs', 'display': 'OnlyInsert', 'noremenu': v:false}"),
        ("menu_info('Edit.OnlyInsert', 'n')", "{}"),
        ("menu_info('Tools.Quiet', 'n')", "{'modes': 'n', 'script': v:false, 'name': 'Quiet', 'priority': 10, 'enabled': v:true, 'shortcut': '', 'silent': v:true, 'rhs': ':echo ''quiet''<CR>', 'display': 'Quiet', 'noremenu': v:false}"),
        ("menu_info('Tools.Scripted', 'n')", "{'modes': 'n', 'script': v:true, 'name': 'Scripted', 'priority': 20, 'enabled': v:true, 'shortcut': '', 'silent': v:false, 'rhs': ':echo ''scripted''<CR>', 'display': 'Scripted', 'noremenu': v:false}"),
        ("menu_info('File')", "{'modes': ' ', 'submenus': ['Open', 'Save', '-sep1-'], 'name': '&File', 'priority': 10, 'shortcut': 'F', 'display': 'File'}"),
        ("menu_info('Deep.Sub', 'a')", "{'modes': ' ', 'submenus': ['Leaf'], 'name': 'Sub', 'priority': 10, 'shortcut': '', 'display': 'Sub'}"),
        ("menu_info('Term.Item', 'tl')", "{'modes': 'tl', 'script': v:false, 'name': 'Item', 'priority': 500, 'enabled': v:true, 'shortcut': '', 'silent': v:false, 'rhs': 'term-rhs', 'display': 'Item', 'noremenu': v:false}"),
        ("menu_info('Cmd.Item', 'c')", "{'modes': 'c', 'script': v:false, 'name': 'Item', 'priority': 500, 'enabled': v:true, 'shortcut': '', 'silent': v:false, 'rhs': 'cmd-rhs', 'display': 'Item', 'noremenu': v:false}"),
        ("menu_info('NoSuch')", "{}"),
    ]);
    fx.check_output("menu", "\n--- Menus ---\n10 &File\n  10 &Open^ICTRL-O\n      n    :echo 'open'<CR>\n      v    :echo 'open'<CR>\n      s    :echo 'open'<CR>\n      o    :echo 'open'<CR>\n  20 &Save\n      n    :echo 'save'<CR>\n      v    :echo 'save'<CR>\n      s    :echo 'save'<CR>\n      o    :echo 'save'<CR>\n  30 -sep1-\n      n    <Nop>\n      v    <Nop>\n      s    <Nop>\n      o    <Nop>\n20 Edit\n  20 OnlyVisual\n      v    visual-rhs\n      s    visual-rhs\n30 Tools\n  10 Quiet\n      n s  :echo 'quiet'<CR>\n  20 Scripted\n      n&   :echo 'scripted'<CR>\n40 Deep\n  10 Sub\n    10 Leaf\n        n    leaf\n        v    leaf\n        s    leaf\n        o    leaf");
    fx.check_output("menu File", "\n--- Menus ---\n10 &File\n  10 &Open^ICTRL-O\n      n    :echo 'open'<CR>\n      v    :echo 'open'<CR>\n      s    :echo 'open'<CR>\n      o    :echo 'open'<CR>\n  20 &Save\n      n    :echo 'save'<CR>\n      v    :echo 'save'<CR>\n      s    :echo 'save'<CR>\n      o    :echo 'save'<CR>\n  30 -sep1-\n      n    <Nop>\n      v    <Nop>\n      s    <Nop>\n      o    <Nop>");
    fx.check_output(
        "imenu Edit",
        "\n--- Menus ---\n20 Edit\n  10 OnlyInsert\n      i    insert-rhs",
    );
    fx.check_output("nmenu Tools", "\n--- Menus ---\n30 Tools\n  10 Quiet\n      n s  :echo 'quiet'<CR>\n  20 Scripted\n      n&   :echo 'scripted'<CR>");
    fx.check_output(
        "tlmenu",
        "\n--- Menus ---\n500 Term\n  500 Item\n      tl    term-rhs",
    );
    for (cmd, err) in [
        ("menu NoSuch", "E329: No menu \"NoSuch\""),
        (
            "menu File.Open.Deeper x",
            "E327: Part of menu-item path is not sub-menu",
        ),
        ("menu File", "none"),
        ("menu 10.x Bad.Prio x", "none"),
        ("menu .Item x", "E475: Invalid argument: .Item x"),
        ("emenu NoSuch.Item", "E334: Menu not found: NoSuch.Item"),
        ("emenu File", "E333: Menu path must lead to a menu item"),
        (
            "call menu_info()",
            "E119: Not enough arguments for function: menu_info",
        ),
    ] {
        fx.check_error(cmd, err);
    }
    clean(&fx);
    fx.expect_all(&[(TOPS, "[]")]);
}

#[test]
fn priorities_order_siblings() {
    let fx = Fixture::new();
    clean(&fx);
    for cmd in [
        "menu 30 Third.Item x",
        "menu 10 First.Item x",
        "menu 20 Second.Item x",
        "menu Default.Item x",
        "menu 10 FirstToo.Item x",
        "menu 10.30 First.C x",
        "menu 10.10 First.A x",
        "menu 10.20 First.B x",
        "menu First.NoPrio x",
        "menu 10.20.5 First.B2.Leaf x",
    ] {
        fx.run(cmd);
    }
    fx.expect_all(&[
        (TOPS, "['First', 'FirstToo', 'Second', 'Third', 'Default']"),
        (
            "map(menu_get(''), 'v:val.priority')",
            "[10, 10, 20, 30, 500]",
        ),
        (
            "map(menu_get('First')[0].submenus, 'v:val.name')",
            "['A', 'B', 'B2', 'C', 'Item', 'NoPrio']",
        ),
        (
            "map(menu_get('First')[0].submenus, 'v:val.priority')",
            "[10, 20, 20, 30, 500, 500]",
        ),
        ("menu_get('First.B2')[0].submenus[0].priority", "5"),
    ]);
    // Redefining an item keeps its place; a new priority is not a move.
    fx.run("menu 99 Third.Item y");
    fx.expect_all(&[
        (TOPS, "['First', 'FirstToo', 'Second', 'Third', 'Default']"),
        ("menu_info('Third.Item').rhs", "'y'"),
    ]);
    fx.check_output("menu First", "\n--- Menus ---\n10 First\n  10 A\n      n    x\n      v    x\n      s    x\n      o    x\n  20 B\n      n    x\n      v    x\n      s    x\n      o    x\n  20 B2\n    5 Leaf\n        n    x\n        v    x\n        s    x\n        o    x\n  30 C\n      n    x\n      v    x\n      s    x\n      o    x\n  500 Item\n      n    x\n      v    x\n      s    x\n      o    x\n  500 NoPrio\n      n    x\n      v    x\n      s    x\n      o    x");
    clean(&fx);
}

#[test]
fn names_carry_accelerators_mnemonics_and_escapes() {
    let fx = Fixture::new();
    clean(&fx);
    for cmd in [
        "menu Top.&Mnemonic x",
        "menu Top.Accel<Tab>Ctrl-A x",
        "menu Top.Both&X<Tab>F5 x",
        "menu Top.With\\ Space x",
        "menu Top.With\\.Dot x",
        "menu Top.&&Amp x",
        "menu Top.Lit\\tTab x",
        "menu Top.\u{e9}t\u{e9} x",
        "menu Top.-Sep- :",
    ] {
        fx.run(cmd);
    }
    fx.expect_all(&[
        ("map(menu_get('Top')[0].submenus, 'v:val.name')", "['Mnemonic', 'Accel', 'BothX', 'With Space', 'With.Dot', '&Amp', 'LittTab', 'été', '-Sep-']"),
        ("map(menu_get('Top')[0].submenus, 'get(v:val, \"shortcut\", \"-\")')", "['M', '-', 'X', '-', '-', '-', '-', '-', '-']"),
        ("map(menu_get('Top')[0].submenus, 'get(v:val, \"accel\", \"-\")')", "['-', '-', '-', '-', '-', '-', '-', '-', '-']"),
        ("map(menu_get('Top')[0].submenus, 'v:val.hidden')", "[0, 0, 0, 0, 0, 0, 0, 0, 0]"),
        ("menu_info('Top.With\\ Space')", "{'modes': ' ', 'script': v:false, 'name': 'With Space', 'priority': 500, 'enabled': v:true, 'shortcut': '', 'silent': v:false, 'rhs': 'x', 'display': 'With Space', 'noremenu': v:false}"),
        ("menu_info('Top.With\\.Dot').name", "'With.Dot'"),
        ("menu_info('Top.Mnemonic').display", "'Mnemonic'"),
        ("menu_info('Top.-Sep-')", "{'modes': ' ', 'script': v:false, 'name': '-Sep-', 'priority': 500, 'enabled': v:true, 'shortcut': '', 'silent': v:false, 'rhs': ':', 'display': '-Sep-', 'noremenu': v:false}"),
    ]);
    fx.check_output("menu Top", "\n--- Menus ---\n500 Top\n  500 &Mnemonic\n      n    x\n      v    x\n      s    x\n      o    x\n  500 Accel^ICtrl-A\n      n    x\n      v    x\n      s    x\n      o    x\n  500 Both&X^IF5\n      n    x\n      v    x\n      s    x\n      o    x\n  500 With Space\n      n    x\n      v    x\n      s    x\n      o    x\n  500 With.Dot\n      n    x\n      v    x\n      s    x\n      o    x\n  500 &&Amp\n      n    x\n      v    x\n      s    x\n      o    x\n  500 LittTab\n      n    x\n      v    x\n      s    x\n      o    x\n  500 été\n      n    x\n      v    x\n      s    x\n      o    x\n  500 -Sep-\n      n    :\n      v    :\n      s    :\n      o    :");
    clean(&fx);
}

#[test]
fn menus_are_enabled_disabled_and_removed() {
    let fx = Fixture::new();
    clean(&fx);
    for cmd in [
        "amenu Top.A a",
        "amenu Top.B b",
        "amenu Top.Sub.C c",
        "nmenu Other.D d",
    ] {
        fx.run(cmd);
    }
    fx.run("menu disable Top.A");
    fx.run("nmenu disable Top.Sub");
    fx.expect_all(&[
        ("menu_info('Top.A', 'n').enabled", "v:false"),
        ("menu_info('Top.A', 'i').enabled", "v:true"),
        ("menu_info('Top.Sub.C', 'n').enabled", "v:true"),
        ("menu_info('Top.Sub.C', 'v').enabled", "v:true"),
        ("map(menu_get('Top', 'n')[0].submenus, '[v:val.name, get(v:val, \"mappings\", {})->get(\"n\", {})->get(\"enabled\", -1)]')", "[['A', 0], ['B', 1], ['Sub', -1]]"),
    ]);
    fx.run("menu enable *");
    fx.expect_all(&[(
        "[menu_info('Top.A', 'n').enabled, menu_info('Top.Sub.C', 'n').enabled]",
        "[v:false, v:true]",
    )]);
    // Removing from one mode leaves the others; removing a submenu takes its
    // children; `*` takes everything in the mode.
    fx.run("nunmenu Top.A");
    fx.expect_all(&[
        (
            "[menu_info('Top.A', 'n'), menu_info('Top.A', 'i').rhs]",
            "[{}, '<C-\\><C-O>a']",
        ),
        ("map(menu_get('', 'n'), 'v:val.name')", "['Top', 'Other']"),
    ]);
    fx.run("aunmenu Top.Sub");
    fx.expect_all(&[
        (
            "map(menu_get('Top')[0].submenus, 'v:val.name')",
            "['A', 'B']",
        ),
        ("menu_info('Top.Sub.C')", "{}"),
    ]);
    for (cmd, err) in [
        ("aunmenu Top.Nope", "E329: No menu \"Nope\""),
        ("aunmenu Nope", "E329: No menu \"Nope\""),
        ("menu disable Nope", "E329: No menu \"Nope\""),
    ] {
        fx.check_error(cmd, err);
    }
    fx.run("nunmenu *");
    fx.expect_all(&[
        ("map(menu_get('', 'n'), 'v:val.name')", "[]"),
        ("map(menu_get('', 'i'), 'v:val.name')", "['Top']"),
    ]);
    clean(&fx);
    fx.expect_all(&[(TOPS, "[]")]);
}

#[test]
fn translations_rename_what_is_defined_after_them() {
    let fx = Fixture::new();
    clean(&fx);
    fx.run("menu Before.Item x");
    fx.run("menutranslate &File &Fichier");
    fx.run("menutranslate Item Element");
    fx.run("menu &File.Item x");
    fx.run("menu Before.Item2 x");
    fx.expect_all(&[
        (TOPS, "['Before', 'Fichier']"),
        ("map(menu_get('Fichier')[0].submenus, 'v:val.name')", "['Element']"),
        ("map(menu_get('Before')[0].submenus, 'v:val.name')", "['Item', 'Item2']"),
        ("menu_info('Fichier.Element')", "{'modes': ' ', 'script': v:false, 'name': 'Element', 'priority': 500, 'enabled': v:true, 'shortcut': '', 'silent': v:false, 'rhs': 'x', 'display': 'Element', 'noremenu': v:false}"),
    ]);
    fx.run("menutranslate clear");
    fx.run("menu &File.Item y");
    fx.expect_all(&[(TOPS, "['Before', 'Fichier']")]);
    clean(&fx);
}

#[test]
fn amenu_fans_out_per_mode_with_its_prefixes() {
    let fx = Fixture::new();
    clean(&fx);
    fx.run("amenu 5.5 Fan.Item :echo<CR>");
    fx.run("amenu Fan.Nop <Nop>");
    fx.run("tmenu Fan.Item the tip");
    fx.run("amenu PopUp.Cut d");
    fx.run("menu Fan.Sub.Deep x");
    fx.run("amenu Fan.-Sep- :");
    fx.expect_all(&[
        ("map(['n', 'v', 's', 'o', 'i', 'c', 'tl'], 'menu_info(\"Fan.Item\", v:val)->get(\"rhs\", \"-\")')", "[':echo<CR>', '<C-C>:echo<CR><C-\\><C-G>', '<C-C>:echo<CR><C-\\><C-G>', '<C-C>:echo<CR><C-\\><C-G>', '<C-\\><C-O>:echo<CR>', '<C-C>:echo<CR><C-\\><C-G>', '-']"),
        ("menu_info('Fan.Item').modes", "'a'"),
        ("menu_get('Fan.Item')[0].tooltip", "'the tip'"),
        ("sort(keys(menu_get('Fan.Item')[0].mappings))", "['c', 'i', 'n', 'o', 's', 'v']"),
        ("menu_get('Fan.Nop')[0].mappings.i.rhs", "''"),
        ("menu_info('Fan.Nop', 'i').rhs", "'<Nop>'"),
        (TOPS, "['Fan', 'PopUpc', 'PopUpi', 'PopUpo', 'PopUps', 'PopUpv', 'PopUpn', 'PopUp']"),
        ("map(menu_get(''), 'v:val.hidden')", "[0, 1, 1, 1, 1, 1, 1, 0]"),
        ("menu_info('PopUpi.Cut', 'i').rhs", "'<C-\\><C-O>d'"),
        ("getcompletion('Fan.', 'menu')", "['Item', 'Nop', 'Sub.', '-Sep-']"),
        ("getcompletion('', 'menu')", "['Fan.', 'PopUp.']"),
    ]);
    fx.check_output(
        "tmenu",
        "\n--- Menus ---\n5 Fan\n  5 Item\n      t    the tip",
    );
    // A `*` component is a literal name, not "everything": only a whole
    // path of `*` is.
    fx.run("silent! aunmenu Fan.Sub.*");
    fx.expect_all(&[(
        "map(menu_get('Fan.Sub')[0].submenus, 'v:val.name')",
        "['Deep']",
    )]);
    fx.run("tunmenu Fan.Item");
    fx.expect_all(&[("has_key(menu_get('Fan.Item')[0], 'tooltip')", "0")]);
    fx.run("iunmenu PopUp");
    fx.expect_all(&[(
        TOPS,
        "['Fan', 'PopUpc', 'PopUpo', 'PopUps', 'PopUpv', 'PopUpn', 'PopUp']",
    )]);
    fx.run("silent! aunmenu Fan.*");
    fx.expect_all(&[(
        TOPS,
        "['Fan', 'PopUpc', 'PopUpo', 'PopUps', 'PopUpv', 'PopUpn', 'PopUp']",
    )]);
    fx.run("aunmenu PopUp");
    fx.expect_all(&[(TOPS, "['Fan']")]);
    for (cmd, err) in [
        ("amenu Fan x", "E330: Menu path must not lead to a sub-menu"),
        (
            "amenu Bar x",
            "E331: Must not add menu items directly to menu bar",
        ),
        (
            "amenu Top.-Sep-.X x",
            "E332: Separator cannot be part of a menu path",
        ),
        ("amenu Top.& x", "none"),
        ("amenu Top.\\& x", "none"),
        ("popup NoSuch", "E337: Menu not found - check menu names"),
        ("aunmenu! Nope", "E477: No ! allowed: silent! aunmenu! Nope"),
        ("unmenu! Nope", "E329: No menu \"Nope\""),
    ] {
        fx.check_error(cmd, err);
    }
    clean(&fx);
    fx.expect_all(&[(TOPS, "[]")]);
}
