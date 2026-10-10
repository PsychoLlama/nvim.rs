//! User commands end to end, through the commands that keep them:
//! definition and redefinition (with and without `!`), every attribute, the
//! `<...>` codes a run expands (`<args>`, `<q-args>`, `<f-args>`,
//! `<bang>`, `<reg>`, `<mods>`, `<lt>`), buffer-local commands shadowing
//! global ones, the listing, `:delcommand` and `:comclear`, and the name
//! lookup (`exists()`, `fullcommand()`, completion). Ranges and counts need
//! lines in a buffer, which this fixture has no memline for; they are in
//! the unit suite's `usercmd.rs`.
//!
//! Written against the behaviour before a `UserCmd` owns its strings; a
//! row that moves is a behaviour change.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::test_fixture::Fixture;

fn clean(fx: &Fixture) {
    fx.run("silent! comclear");
    fx.run("silent! delcommand -buffer UcLocal");
    fx.run("silent! delcommand -buffer UcBoth");
    fx.run("let g:uc = []");
}

/// What the commands below recorded, one entry per run.
const GOT: &str = "g:uc";

#[test]
fn commands_are_defined_redefined_and_deleted() {
    let fx = Fixture::new();
    clean(&fx);
    fx.run("command UcOne call add(g:uc, 'one')");
    fx.check_error(
        "command UcOne call add(g:uc, 'again')",
        "E174: Command already exists: add ! to replace it: UcOne call add(g:uc, ''again'')",
    );
    fx.run("UcOne");
    fx.run("command! UcOne call add(g:uc, 'redefined')");
    fx.run("UcOne");
    fx.check(GOT, "['one', 'redefined']");
    for (cmd, err) in [
        (
            "command lower echo 1",
            "E183: User defined commands must start with an uppercase letter",
        ),
        ("command Uc1x echo 1", "none"),
        ("command Uc-x echo 1", "E182: Invalid command name"),
        ("command UcNo", "none"),
        (
            "command -nargs=x UcBad echo 1",
            "E176: Invalid number of arguments",
        ),
        (
            "command -nosuch UcBad echo 1",
            "E181: Invalid attribute: nosuch",
        ),
        (
            "command -complete=nosuch UcBad echo 1",
            "E180: Invalid complete value: nosuch UcBad echo 1",
        ),
        (
            "command -range=x UcBad echo 1",
            "E178: Invalid default value for count",
        ),
        (
            "command -count=x UcBad echo 1",
            "E178: Invalid default value for count",
        ),
        (
            "command -addr=nosuch UcBad echo 1",
            "E180: Invalid address type value: nosuch",
        ),
        ("command -range -count UcBad echo 1", "none"),
        (
            "command -complete=custom UcBad echo 1",
            "E467: Custom completion requires a function argument",
        ),
        (
            "command -nargs=0 -complete=file UcBad echo 1",
            "E1208: -complete used without allowing arguments",
        ),
        (
            "delcommand UcNoSuch",
            "E184: No such user-defined command: UcNoSuch",
        ),
        (
            "delcommand -buffer UcOne",
            "E1237: No such user-defined command in current buffer: UcOne",
        ),
        ("UcNoSuch", "E492: Not an editor command: silent! UcNoSuch"),
        (
            "UcOne extra",
            "E488: Trailing characters: extra: silent! UcOne extra",
        ),
    ] {
        fx.check_error(cmd, err);
    }
    fx.expect_all(&[
        ("exists(':UcOne')", "2"),
        ("exists(':UcBad')", "2"),
        ("fullcommand('UcO')", "'UcOne'"),
        ("fullcommand('UcOne')", "'UcOne'"),
        (
            "getcompletion('Uc', 'command')",
            "['Uc1x', 'UcBad', 'UcOne']",
        ),
    ]);
    fx.run("delcommand UcOne");
    fx.check("exists(':UcOne')", "0");
    fx.run("command UcA echo 1");
    fx.run("command UcB echo 2");
    fx.run("comclear");
    fx.check("[exists(':UcA'), exists(':UcB')]", "[0, 0]");
    clean(&fx);
}

#[test]
#[cfg_attr(miri, ignore = "`<mods>` is built with libc's `strcat`")]
fn arguments_expand_in_every_form() {
    let fx = Fixture::new();
    clean(&fx);
    for cmd in [
        "command -nargs=* UcArgs call add(g:uc, [<q-args>, [<f-args>]])",
        "command -nargs=* UcRaw call add(g:uc, '<args>')",
        "command -nargs=1 UcOneArg call add(g:uc, <q-args>)",
        "command -nargs=? UcOpt call add(g:uc, [<q-args>, <q-bang>])",
        "command -nargs=+ UcPlus call add(g:uc, [<f-args>])",
        "command -bang UcBang call add(g:uc, ['<bang>', <q-bang>])",
        "command -register UcReg call add(g:uc, ['<reg>', <q-reg>, '<register>'])",
        "command -nargs=* UcLt call add(g:uc, '<lt>args> <lt>lt> <q-args>')",
        "command -nargs=* UcMods call add(g:uc, [<q-mods>, '<mods>'])",
        "command -bar UcBar call add(g:uc, 'bar')",
        "command UcNoBar call add(g:uc, 'nobar')",
    ] {
        fx.run(cmd);
    }
    fx.run(r"UcArgs a b\ c  d\\e 'q' ");
    fx.run("UcArgs");
    fx.run("UcRaw raw  words\\ here");
    fx.run("UcOneArg  spaced  words ");
    fx.run("UcOpt");
    fx.run("UcOpt x y");
    fx.run(r"UcPlus a\ b c");
    fx.run("UcBang");
    fx.run("UcBang!");
    fx.run("UcReg");
    fx.run("UcReg a");
    fx.run("UcReg _");
    fx.run("UcLt x y");
    fx.run("UcMods");
    fx.run("silent keepjumps UcMods");
    fx.run("vertical botright UcMods");
    // `tab` counts tab pages, and the fixture has none.
    fx.run("aboveleft noautocmd hide UcMods");
    fx.run("verbose confirm lockmarks keepalt keeppatterns UcMods");
    fx.run("sandbox UcMods");
    fx.run("UcBar | call add(g:uc, 'after bar')");
    fx.run("UcNoBar | call add(g:uc, 'not reached')");
    fx.check(GOT, "[['a b\\ c  d\\\\e ''q'' ', ['a', 'b c', 'd\\e', '''q''']], ['', []], 'raw  words\\ here', 'spaced  words ', ['', ''], ['x y', ''], ['a b', 'c'], ['', ''], ['!', '!'], ['', '', ''], ['a', 'a', 'a'], ['_', '_', '_'], '<args> <lt> \"x y\"', ['', ''], ['keepjumps silent', 'keepjumps silent'], ['botright vertical', 'botright vertical'], ['hide noautocmd aboveleft', 'hide noautocmd aboveleft'], ['confirm keepalt keeppatterns lockmarks verbose', 'confirm keepalt keeppatterns lockmarks verbose'], 'bar', 'after bar']");
    for (cmd, err) in [
        ("UcOneArg", "E471: Argument required: silent! UcOneArg"),
        ("UcPlus", "E471: Argument required: silent! UcPlus"),
        (
            "UcBar extra",
            "E488: Trailing characters: extra: silent! UcBar extra",
        ),
        ("UcArgs!", "E477: No ! allowed: silent! UcArgs!"),
    ] {
        fx.check_error(cmd, err);
    }
    clean(&fx);
}

#[test]
fn buffer_commands_shadow_global_ones_and_are_listed() {
    let fx = Fixture::new();
    clean(&fx);
    fx.run("command UcBoth call add(g:uc, 'global')");
    fx.run("command -buffer UcBoth call add(g:uc, 'local')");
    fx.run("command -buffer -nargs=1 -complete=file UcLocal call add(g:uc, 'only local')");
    fx.run(
        "command -nargs=+ -complete=customlist,UcComplete -range=% -bar -keepscript UcFancy echo",
    );
    fx.run("command -nargs=? -complete=custom,UcComplete -count=3 -register -bang -addr=windows UcFlags echo");
    fx.run("UcBoth");
    fx.run("UcLocal x");
    fx.check(GOT, "['local', 'only local']");
    fx.expect_all(&[
        ("exists(':UcBoth')", "2"),
        ("exists(':UcLocal')", "2"),
        ("getcompletion('UcB', 'command')", "['UcBoth']"),
    ]);
    fx.check_output("command", "\n    Name              Args Address Complete    Definition\nb   UcBoth            0                        call add(g:uc, 'local')\nb   UcLocal           1            file        call add(g:uc, 'only local')\n    UcBoth            0                        call add(g:uc, 'global')\n|   UcFancy           +    %       customlist  echo\n!\"  UcFlags           ?    3c win  custom      echo");
    fx.check_output("command UcF", "\n    Name              Args Address Complete    Definition\n|   UcFancy           +    %       customlist  echo\n!\"  UcFlags           ?    3c win  custom      echo");
    fx.check_output("command UcBoth", "\n    Name              Args Address Complete    Definition\nb   UcBoth            0                        call add(g:uc, 'local')\n    UcBoth            0                        call add(g:uc, 'global')");
    fx.check_output(
        "command -buffer",
        "\nE183: User defined commands must start with an uppercase letter",
    );
    fx.check_output("verbose command UcFlags", "\n    Name              Args Address Complete    Definition\n!\"  UcFlags           ?    3c win  custom      echo");
    fx.check_error(
        "command -buffer UcBoth again",
        "E174: Command already exists: add ! to replace it: UcBoth again",
    );
    fx.run("delcommand -buffer UcBoth");
    fx.run("let g:uc = []");
    fx.run("UcBoth");
    fx.check(GOT, "['global']");
    fx.check_error(
        "delcommand -buffer UcBoth",
        "E1237: No such user-defined command in current buffer: UcBoth",
    );
    fx.run("comclear");
    // `:comclear` empties both tables.
    fx.check(
        "[exists(':UcBoth'), exists(':UcLocal'), exists(':UcFancy')]",
        "[0, 0, 0]",
    );
    clean(&fx);
}
