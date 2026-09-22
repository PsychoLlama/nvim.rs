//! The Ex command dispatcher: what a command line parses to, and what
//! running one does.
//!
//! `ex_docmd` had no in-crate test at all. The functional and oldtest
//! suites reach it through every `:` command they run, but nothing pinned
//! the parse's *answers* — the fields `do_one_cmd` fills in before it
//! dispatches — or the handful of dispatch decisions that have no visible
//! output: which handler a line reaches, where the next command after a
//! `|` starts, and what a nested `do_cmdline` sees.
//!
//! Two engines here. [`parse`] runs `parse_cmdline`, which is every stage
//! of `do_one_cmd`'s parse and none of its effects, and reads the resulting
//! `ExArg` back as an owned record. [`run`] runs a real command line
//! through `do_cmdline_cmd` and reads the result out of a `g:` variable.

#![cfg(not(miri))]

use std::ffi::{CString, c_int};

use neovim::eval::{eval_to_number, eval_to_string};
use neovim::ex_docmd::{FORCE_BIN, FORCE_NOBIN};
use neovim::ex_docmd::{do_cmdline_cmd, getargopt, parse_cmdline, separate_nextcmd};
use neovim::types::{CmdIdx, CmdLine, CmdModFlags, CmdParseInfo, ExArg, ExArgt, LineNr, Pos};
use neovim::winlayer::Win;

use crate::support::{Editor, Sandbox, check_emsg, cstr, editor_lock};

/// The buffer every case that needs lines is written against.
const LINES: &str = "call setline(1, ['one', 'two', 'three', 'four', 'five'])";

/// Puts the scratch buffer and the cursor back on the way out.
///
/// There is one editor per test process and one current buffer in it, so a
/// case that fills the buffer with lines or moves the cursor is visible to
/// every other case — the statusline cases next door read both. Running any
/// command at all moves the cursor, because `do_one_cmd` lifts a zero line
/// number to one on its way out, so this is not only for the cases that
/// call [`LINES`].
struct Restore {
    cursor: Pos,
}

impl Restore {
    /// The caller holds the editor lock for the whole of the value's life,
    /// which is what makes the restoration exclusive too.
    fn new(_editor: &Editor) -> Restore {
        Restore {
            cursor: Win::current().w_cursor,
        }
    }
}

impl Drop for Restore {
    fn drop(&mut self) {
        let empty = cstr("silent! keepjumps keepmarks %delete _");
        // SAFETY: `empty` is NUL-terminated and outlives the call.
        let _ = do_cmdline_cmd(&empty);
        Win::current().w_cursor = self.cursor;
    }
}

/// Everything a case asserts about a parsed command line, owned so that the
/// line buffer it was parsed out of can go away.
#[derive(Debug, PartialEq, Eq)]
struct Parsed {
    cmdidx: &'static str,
    line1: LineNr,
    line2: LineNr,
    addr_count: c_int,
    forceit: bool,
    regname: Option<char>,
    arg: String,
    nextcmd: Option<String>,
}

/// `cmdidx` as a name, since `CmdIdx` is an enum of 557 commands with no
/// equality and the assertion reads better as the command's own spelling.
fn cmd_name(idx: CmdIdx) -> &'static str {
    Box::leak(format!("{idx:?}").into_boxed_str())
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Parse one line, answering the record on success and the message on
/// failure.
fn parse(_editor: &Editor, line: &str) -> Result<Parsed, String> {
    let mut args = ExArg::default();
    // SAFETY: `CmdParseInfo` is a `repr(Rust)` aggregate of scalars and
    // pointers, and `parse_cmdline` zeroes it before its first read anyway.
    let mut info: CmdParseInfo = unsafe { std::mem::zeroed() };
    let mut errormsg: Option<CString> = None;
    // SAFETY: both out-parameters are locals of this frame, unaliased for
    // the call.
    let ok = unsafe {
        parse_cmdline(
            CmdLine::from_bytes(line.as_bytes()),
            &mut args,
            &raw mut info,
            &mut errormsg,
        )
    };
    if !ok {
        return Err(errormsg.map_or_else(String::new, |m| m.to_string_lossy().into_owned()));
    }
    Ok(Parsed {
        cmdidx: cmd_name(args.cmdidx),
        line1: args.line1,
        line2: args.line2,
        addr_count: args.addr_count,
        forceit: args.forceit,
        regname: u8::try_from(args.regname)
            .ok()
            .filter(|&r| r != 0)
            .map(char::from),
        arg: text(args.line.arg()),
        nextcmd: args.line.next_cmd().map(text),
    })
}

/// The modifier flags a line parses to, and the `:verbose` level it asks
/// for.
fn parse_mods(_editor: &Editor, line: &str) -> (CmdModFlags, c_int) {
    let mut args = ExArg::default();
    // SAFETY: as in `parse`.
    let mut info: CmdParseInfo = unsafe { std::mem::zeroed() };
    let mut errormsg: Option<CString> = None;
    // SAFETY: as in `parse`.
    let ok = unsafe {
        parse_cmdline(
            CmdLine::from_bytes(line.as_bytes()),
            &mut args,
            &raw mut info,
            &mut errormsg,
        )
    };
    assert!(ok, "{line:?} did not parse");
    (info.cmdmod.cmod_flags, info.cmdmod.cmod_verbose)
}

/// Run a command line the way a mapping would.
fn run(_editor: &Editor, line: &str) {
    let text = cstr(line);
    // SAFETY: `text` is NUL-terminated and outlives the call.
    let _ = do_cmdline_cmd(&text);
}

/// Evaluate an expression for its number.
fn num(_editor: &Editor, expr: &str) -> i64 {
    let text = cstr(expr);
    // SAFETY: `text` is NUL-terminated and outlives the call.
    unsafe { eval_to_number(text.as_ptr().cast_mut(), false) }
}

/// Evaluate an expression for its string.
fn string(_editor: &Editor, expr: &str) -> String {
    let text = cstr(expr);
    // SAFETY: `text` is NUL-terminated and outlives the call; the answer is
    // an allocation this takes ownership of.
    let got = unsafe { eval_to_string(text.as_ptr().cast_mut(), false, false) };
    if got.is_null() {
        return String::new();
    }
    // SAFETY: `eval_to_string` answers an `xmalloc`ed NUL-terminated string.
    unsafe { crate::support::internalize(got) }
}

// ---------------------------------------------------------------------------
// The parse

#[test]
fn a_range_fills_both_endpoints_and_counts_them() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    run(&editor, LINES);
    assert_eq!(
        parse(&editor, "2,4delete"),
        Ok(Parsed {
            cmdidx: "delete",
            line1: 2,
            line2: 4,
            addr_count: 2,
            forceit: false,
            regname: None,
            arg: String::new(),
            nextcmd: None,
        })
    );
    // One address sets both ends.
    let one = parse(&editor, "3print").unwrap();
    assert_eq!((one.line1, one.line2, one.addr_count), (3, 3, 1));
    // `%` is the whole buffer, `$` the last line, `.` the cursor's.
    let whole = parse(&editor, "%print").unwrap();
    assert_eq!((whole.line1, whole.line2, whole.addr_count), (1, 5, 2));
    let last = parse(&editor, "$print").unwrap();
    assert_eq!((last.line1, last.line2, last.addr_count), (5, 5, 1));
    // No range at all leaves the count at zero and both ends at the cursor.
    let none = parse(&editor, "print").unwrap();
    assert_eq!(none.addr_count, 0);
}

#[test]
fn an_offset_and_a_search_are_addresses_too() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    run(&editor, LINES);
    run(&editor, "1");
    let offset = parse(&editor, ".+2print").unwrap();
    assert_eq!((offset.line1, offset.line2), (3, 3));
    let search = parse(&editor, "/four/print").unwrap();
    assert_eq!((search.line1, search.line2), (4, 4));
    // The parse puts the cursor and the last search pattern back.
    assert_eq!(num(&editor, "line('.')"), 1);
}

#[test]
fn a_bang_is_separated_from_the_command_name() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    assert!(parse(&editor, "write! /dev/null").unwrap().forceit);
    assert!(!parse(&editor, "write /dev/null").unwrap().forceit);
    // A command that takes no bang refuses one.
    assert_eq!(parse(&editor, "print!"), Err("E477: No ! allowed".into()));
}

#[test]
fn a_register_and_a_count_follow_the_argument() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    run(&editor, LINES);
    let got = parse(&editor, "2delete x 3").unwrap();
    assert_eq!(got.regname, Some('x'));
    // A count after the register is a *number of lines* from `line1`.
    assert_eq!((got.line1, got.line2), (2, 4));
    assert_eq!(parse(&editor, "2delete").unwrap().regname, None);
}

/// Run `getargopt` over every `++opt` at the head of `arg`, the way
/// `do_one_cmd` does, and answer what it recorded: the forced file format
/// (kept as its first letter), the forced encoding (an offset into the
/// line), `++bin`'s three-valued flag, `++edit`, `++p`, `++bad=` and what
/// is left of the argument.
type ArgOpts = (char, String, c_int, bool, bool, c_int, String);

fn argopts(_editor: &Editor, line: &str) -> Result<ArgOpts, ()> {
    let mut args = ExArg {
        line: CmdLine::from_bytes(line.as_bytes()),
        ..Default::default()
    };
    while args.line.arg().starts_with(b"++") {
        if getargopt(&mut args).is_err() {
            return Err(());
        }
    }
    // An offset `getargopt` recorded is from the command word, and the
    // value it names was NUL-terminated in place.
    let at = |off: c_int| -> String {
        if off == 0 {
            String::new()
        } else {
            text(args.line.rest_of(args.line.cmd + off as usize))
        }
    };
    Ok((
        char::from(u8::try_from(args.force_ff).unwrap_or(0)),
        at(args.force_enc),
        args.force_bin,
        args.read_edit,
        args.mkdir_p,
        args.bad_char,
        text(args.line.arg()),
    ))
}

#[test]
fn a_plus_plus_option_is_taken_off_the_argument() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    // `++ff=` keeps only the first letter of the format it forced, and
    // `++enc=` is lower-cased in place.
    assert_eq!(
        argopts(&editor, "++ff=unix ++enc=LATIN1 ++bin ++edit ++p somefile"),
        Ok((
            'u',
            "latin1".into(),
            FORCE_BIN,
            true,
            true,
            0,
            "somefile".into()
        ))
    );
    // `++bin` is three-valued, not a flag: `++nobin` forces the other way.
    assert_eq!(argopts(&editor, "++nobin x").map(|o| o.2), Ok(FORCE_NOBIN));
    assert_eq!(argopts(&editor, "x").map(|o| o.2), Ok(0));
    // `++bad=` takes a character, `keep` or `drop`.
    assert_eq!(argopts(&editor, "++bad=? x").map(|o| o.5), Ok('?' as c_int));
    // An unknown option is refused, and so is a bad `++ff=`.
    assert_eq!(argopts(&editor, "++nosuch x"), Err(()));
    assert_eq!(argopts(&editor, "++ff=nosuch x"), Err(()));
}

#[test]
fn a_bar_ends_the_command_and_names_the_next_one() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    // `:set` carries `TRLBAR`, so the parse splits the line at the bar.
    let got = parse(&editor, "set ai | set noai").unwrap();
    assert_eq!(got.arg, "ai");
    assert_eq!(got.nextcmd.as_deref(), Some("set noai"));
    // `:let` does not: `NOTRLCOM` keeps the bar in the argument, and the
    // handler is what finds the next command.
    let keeps = parse(&editor, "let x = 1 | let y = 2").unwrap();
    assert_eq!(keeps.arg, "x = 1 | let y = 2");
    assert_eq!(keeps.nextcmd, None);
    // `:normal` takes the bar as part of its argument.
    let normal = parse(&editor, "normal ix|y").unwrap();
    assert_eq!(normal.arg, "ix|y");
    assert_eq!(normal.nextcmd, None);
    // So does `:execute`, up to the bar that is not inside an expression.
    let execute = parse(&editor, "execute 'a' | let y = 2").unwrap();
    assert_eq!(execute.arg.trim_end(), "'a'");
    assert_eq!(execute.nextcmd.as_deref(), Some("let y = 2"));
}

#[test]
fn modifiers_are_read_off_the_front_of_the_line() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    let (flags, verbose) = parse_mods(&editor, "silent! echo 1");
    assert!(flags.has(CmdModFlags::SILENT));
    assert!(flags.has(CmdModFlags::ERRSILENT));
    assert_eq!(verbose, 0);
    let (flags, verbose) = parse_mods(&editor, "3verbose echo 1");
    assert!(!flags.has(CmdModFlags::SILENT));
    // Stored as the count plus one, so that zero can mean "not given".
    assert_eq!(verbose, 4);
    let (flags, _) = parse_mods(&editor, "noautocmd keepjumps echo 1");
    assert!(flags.has(CmdModFlags::NOAUTOCMD));
    assert!(flags.has(CmdModFlags::KEEPJUMPS));
    // The command after the modifiers is still found.
    assert_eq!(parse(&editor, "silent echo 1").unwrap().cmdidx, "echo");
}

#[test]
fn a_visual_range_survives_the_modifiers_it_was_typed_in_front_of() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    run(&editor, LINES);
    run(&editor, r#"call setpos("'<", [0, 2, 1, 0])"#);
    run(&editor, r#"call setpos("'>", [0, 4, 1, 0])"#);
    // The modifier scan steps over the `'<,'>` so that a modifier behind it
    // is still seen, and then has to shuffle the modifier out of the way so
    // the range reaches the command. Both have to survive it.
    let got = parse(&editor, "'<,'>keepjumps print").unwrap();
    assert_eq!((got.cmdidx, got.line1, got.line2), ("print", 2, 4));
    let (flags, _) = parse_mods(&editor, "'<,'>keepjumps print");
    assert!(flags.has(CmdModFlags::KEEPJUMPS));
    // Two modifiers: the shuffle moves both, and the range still lands
    // immediately in front of the command word.
    let got = parse(&editor, "'<,'>keepjumps keepmarks print").unwrap();
    assert_eq!((got.cmdidx, got.line1, got.line2), ("print", 2, 4));
    // No modifier at all: the scan puts the cursor back where it found it
    // rather than moving anything.
    let got = parse(&editor, "'<,'>print").unwrap();
    assert_eq!((got.cmdidx, got.line1, got.line2), ("print", 2, 4));
    // A `'<,'>` with nothing behind it is not stepped over at all: it is
    // a bare range, which `parse_cmdline` reads without finding a command.
    let got = parse(&editor, "'<,'>").unwrap();
    assert_eq!((got.cmdidx, got.line1, got.line2), ("SIZE", 2, 4));
}

#[test]
fn an_unknown_command_is_named_in_the_message() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    assert_eq!(
        parse(&editor, "Nosuchcommand"),
        Err("E492: Not an editor command: Nosuchcommand".into())
    );
}

#[test]
fn a_user_command_parses_to_the_user_row() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    run(
        &editor,
        "command! -nargs=* -range -bang -register Rec let g:seen = 1",
    );
    let got = parse(&editor, "2,4Rec! y one two").unwrap();
    assert_eq!(got.cmdidx, "USER");
    assert_eq!((got.line1, got.line2, got.addr_count), (2, 4, 2));
    assert!(got.forceit);
    assert_eq!(got.regname, Some('y'));
    assert_eq!(got.arg, "one two");
    run(&editor, "delcommand Rec");
}

#[test]
fn exists_says_whether_a_command_name_was_spelled_out_in_full() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    // 0 is "no such command", 1 an abbreviation, 2 the full name, 3 a user
    // command the abbreviation cannot choose between. The 1-versus-2 answer
    // is the `full` flag `find_ex_command` writes, which nothing else in the
    // crate reads -- and which a user command gets from `find_ucmd` rather
    // than from the command table.
    assert_eq!(num(&editor, r#"exists(":print")"#), 2);
    assert_eq!(num(&editor, r#"exists(":pri")"#), 1);
    assert_eq!(num(&editor, r#"exists(":Nosuchcommand")"#), 0);
    run(&editor, "command! MyCmd let g:seen = 1");
    assert_eq!(num(&editor, r#"exists(":MyCmd")"#), 2);
    assert_eq!(num(&editor, r#"exists(":My")"#), 1);
    run(&editor, "command! MyOther let g:seen = 2");
    assert_eq!(num(&editor, r#"exists(":My")"#), 3);
    run(&editor, "delcommand MyCmd");
    run(&editor, "delcommand MyOther");
    // A modifier counts as a command, and answers the same 1/2.
    assert_eq!(num(&editor, r#"exists(":silent")"#), 2);
    assert_eq!(num(&editor, r#"exists(":sil")"#), 1);
}

// ---------------------------------------------------------------------------
// The dispatch

/// Define the recording user command, run `line`, and answer what the
/// command saw.
fn record(editor: &Editor, line: &str) -> String {
    run(
        editor,
        "command! -nargs=* -range -bang -register Rec \
         let g:rec = [<line1>, <line2>, <range>, '<bang>', '<reg>', <q-args>, '<mods>']",
    );
    run(editor, "unlet! g:rec");
    // `line1` defaults to the cursor's line, and the editor is process-wide.
    run(editor, "1");
    run(editor, line);
    let got = string(editor, "string(get(g:, 'rec', 'unrun'))");
    run(editor, "delcommand Rec");
    got
}

#[test]
fn a_user_command_is_dispatched_with_its_range_bang_register_and_arguments() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    run(&editor, LINES);
    assert_eq!(
        record(&editor, "2,4Rec! y one two"),
        "[2, 4, 2, '!', 'y', 'one two', '']"
    );
    assert_eq!(record(&editor, "Rec"), "[1, 1, 0, '', '', '', '']");
    assert_eq!(
        record(&editor, "silent keepjumps Rec"),
        "[1, 1, 0, '', '', '', 'keepjumps silent']"
    );
}

#[test]
fn an_unknown_command_reports_e492_and_swallows_the_rest_of_the_line() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    run(&editor, "unlet! g:reached");
    // An unknown command has no `argt`, so `separate_nextcmd` never ran and
    // there is no next command: the bar and everything after it are part of
    // the name being refused. Upstream behaviour, pinned because the
    // dispatch is what decides it.
    check_emsg(
        &editor,
        || run(&editor, "Nosuchcommand | let g:reached = 1"),
        Some("E492: Not an editor command: Nosuchcommand | let g:reached = 1"),
    );
    assert_eq!(num(&editor, "exists('g:reached')"), 0);
}

#[test]
fn a_sourced_script_runs_line_by_line_and_stops_at_finish() {
    let sandbox = Sandbox::dir("ex-docmd-source");
    let editor = sandbox.editor();
    let _restore = Restore::new(editor);
    let script = sandbox.write(
        "run.vim",
        [
            "let g:one = 1",
            "try",
            "Nosuchcommand",
            "catch",
            "let g:caught = v:exception",
            "endtry",
            "let g:two = 2",
            "finish",
            "let g:three = 3",
        ]
        .join("\n")
        .as_bytes(),
    );
    run(
        editor,
        "unlet! g:one | unlet! g:two | unlet! g:three | unlet! g:caught",
    );
    run(editor, &format!("source {}", script.display()));
    assert_eq!(num(editor, "g:one"), 1);
    // Lines come from the getter one at a time, so the `:catch` is still
    // there to see the refusal the bar form above swallowed.
    assert_eq!(
        string(editor, "get(g:, 'caught', '')"),
        "Vim:E492: Not an editor command: Nosuchcommand"
    );
    assert_eq!(num(editor, "g:two"), 2);
    // `:finish` ends the script; the line after it never runs.
    assert_eq!(num(editor, "exists('g:three')"), 0);
}

#[test]
fn a_bar_runs_both_commands_in_order() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    run(&editor, "unlet! g:a | unlet! g:b");
    run(&editor, "let g:a = 1 | let g:b = g:a + 1");
    assert_eq!(num(&editor, "g:b"), 2);
    // A trailing bar with nothing after it is not a next command.
    run(&editor, "let g:c = 3 |");
    assert_eq!(num(&editor, "g:c"), 3);
}

#[test]
fn execute_runs_a_multi_line_string_line_by_line() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    run(&editor, "unlet! g:lines");
    run(&editor, "let g:lines = []");
    run(
        &editor,
        "execute \"call add(g:lines, 'first')\\ncall add(g:lines, 'second')\"",
    );
    assert_eq!(string(&editor, "join(g:lines, ',')"), "first,second");
    // And a nested `do_cmdline` sees its own `|` separation.
    run(&editor, "execute 'let g:n1 = 10 | let g:n2 = g:n1 * 2'");
    assert_eq!(num(&editor, "g:n2"), 20);
}

#[test]
fn an_error_inside_a_try_is_caught_and_the_rest_of_the_block_is_skipped() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    run(
        &editor,
        "unlet! g:caught | unlet! g:after | unlet! g:finally",
    );
    run(
        &editor,
        "try | throw 'boom' | let g:after = 1 | catch /boom/ \
         | let g:caught = v:exception | finally | let g:finally = 1 | endtry",
    );
    assert_eq!(string(&editor, "get(g:, 'caught', '')"), "boom");
    assert_eq!(num(&editor, "exists('g:after')"), 0);
    assert_eq!(num(&editor, "g:finally"), 1);
    // An editor error, not a `:throw`, reaches the same place.
    run(&editor, "unlet! g:caught");
    run(
        &editor,
        "try | call nosuchfunction() | catch | let g:caught = v:exception | endtry",
    );
    assert!(
        string(&editor, "get(g:, 'caught', '')").contains("E117"),
        "an unknown function should be catchable"
    );
}

#[test]
fn silent_is_scoped_to_the_one_command_it_modifies() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    // `:silent` raises `msg_silent` for the command and puts it back after,
    // even when the command is a whole nested `do_cmdline`.
    run(&editor, "unlet! g:inner | unlet! g:outer");
    run(&editor, "silent execute 'let g:inner = &verbose'");
    run(&editor, "let g:outer = &verbose");
    assert_eq!(num(&editor, "g:inner"), num(&editor, "g:outer"));
    // `:verbose` is undone the same way.
    run(&editor, "3verbose let g:v = &verbose");
    assert_eq!(num(&editor, "g:v"), 3);
    assert_eq!(num(&editor, "&verbose"), 0);
}

#[test]
fn a_range_with_no_command_moves_the_cursor() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    run(&editor, LINES);
    run(&editor, "1");
    run(&editor, "4");
    assert_eq!(num(&editor, "line('.')"), 4);
    run(&editor, "$");
    assert_eq!(num(&editor, "line('.')"), 5);
}

#[test]
fn the_command_table_agrees_with_what_the_parse_reports() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    // An abbreviation reaches the same row as the full name, and the row's
    // `argt` is what the parse enforces.
    for (line, name) in [
        ("s/a/b/", "substitute"),
        ("substitute/a/b/", "substitute"),
        ("e file", "edit"),
        ("ed file", "edit"),
        ("norm ix", "normal"),
        ("g/x/d", "global"),
        ("v/x/d", "vglobal"),
        ("k a", "k"),
        ("&&", "and"),
        ("<", "lshift"),
        (">>", "rshift"),
    ] {
        assert_eq!(parse(&editor, line).unwrap().cmdidx, name, "{line:?}");
    }
    // `:!` keeps the space after it; every other command drops it.
    assert_eq!(parse(&editor, "! -l").unwrap().arg, " -l");
    assert_eq!(parse(&editor, "edit  x").unwrap().arg, "x");
}

// ---------------------------------------------------------------------------
// The line the parse walks, and what moves it

#[test]
fn an_expansion_that_grows_the_line_repoints_every_cursor_into_it() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    // `expand_filename` replaces `<cword>` with a word far longer than the
    // line it sits in, which means `repl_cmdline` allocates a new line and
    // frees the old one *while the parse is still holding cursors into it*
    // -- `eap->arg`, `eap->cmd`, `eap->args` and `eap->nextcmd` are all
    // offsets into the buffer that just went away. This is the case the
    // `CmdLine` rewrite has to keep true, and nothing pinned it before.
    let long = "w".repeat(400);
    run(&editor, &format!("call setline(1, ['{long}'])"));
    run(&editor, "1");
    run(&editor, "normal! 0");
    run(
        &editor,
        "command! -bar -nargs=* -complete=file Grown let g:grown = <q-args>",
    );
    run(&editor, "unlet! g:grown | unlet! g:after");

    run(&editor, "Grown <cword> | let g:after = 1");

    // The handler read its argument out of the *new* allocation...
    assert_eq!(string(&editor, "g:grown"), long);
    // ... and the command after the `|` was found in it too, which is the
    // half `repl_cmdline` has to copy by hand.
    assert_eq!(num(&editor, "g:after"), 1);

    // The same line with two arguments: `eap->args` is repointed entry by
    // entry, and an argument *before* the replacement must not move while
    // one after it does.
    run(&editor, "unlet! g:grown | unlet! g:after");
    run(&editor, "Grown head <cword> tail | let g:after = 2");
    assert_eq!(string(&editor, "g:grown"), format!("head {long} tail"));
    assert_eq!(num(&editor, "g:after"), 2);

    run(&editor, "delcommand Grown");
}

#[test]
fn separate_nextcmd_cuts_the_argument_at_the_bar_that_ends_it() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);

    // The argument, and what the scan left as the next command. `nextcmd`
    // points at the byte *after* the separator, which is why the expected
    // text keeps its leading space -- while the argument loses its trailing
    // one, which `del_trailing_spaces` takes off on the way out.
    #[track_caller]
    fn split(argt: ExArgt, cmdidx: CmdIdx, line: &str) -> (String, Option<String>) {
        let mut excmd = ExArg {
            cmdidx,
            argt,
            line: CmdLine::from_bytes(line.as_bytes()),
            ..ExArg::default()
        };
        separate_nextcmd(&mut excmd);
        let arg = text(excmd.line.arg());
        let next = excmd.line.next_cmd().map(text);
        (arg, next)
    }

    let plain = ExArgt::EXTRA | ExArgt::TRLBAR;
    // A bare `|` ends the argument and starts the next command.
    assert_eq!(
        split(plain, CmdIdx::print, "one | two"),
        ("one".to_string(), Some(" two".to_string()))
    );
    // A backslash before it escapes it: the backslash is removed and the
    // bar stays in the argument.
    assert_eq!(
        split(plain, CmdIdx::print, r"one \| two"),
        ("one | two".to_string(), None)
    );
    // Only the *first* unescaped bar separates; the rest belong to the
    // next command's own line.
    assert_eq!(
        split(plain, CmdIdx::print, "a | b | c"),
        ("a".to_string(), Some(" b | c".to_string()))
    );
    // A `"` starts a trailing comment for a command that does not take one
    // literally, and ends the argument just as a bar does.
    assert_eq!(
        split(plain, CmdIdx::print, "one \" a comment"),
        ("one".to_string(), None)
    );
    // ... and does not, for a command that reads the rest of the line.
    assert_eq!(
        split(
            ExArgt::EXTRA | ExArgt::NOTRLCOM,
            CmdIdx::print,
            "one \" not a comment"
        ),
        ("one \" not a comment".to_string(), None)
    );
    // `:append`, `:change` and `:insert` read the following lines, so a
    // bar is part of their argument.
    assert_eq!(
        split(plain, CmdIdx::append, "text | more"),
        ("text | more".to_string(), None)
    );
}

/// `CmdLine::find_next` is upstream's `find_nextcmd`, which every `:syntax`
/// subcommand calls before it looks at its argument at all. It is the
/// *search* for a separator, where `check_next` asks whether one is already
/// under the cursor -- and unlike `separate_nextcmd` it recognises neither an
/// escape nor a comment, because the commands that use it take an argument
/// no bar can appear in.
#[test]
fn find_next_looks_for_the_separator_that_check_next_only_tests_for() {
    #[track_caller]
    fn find(line: &str, at: usize) -> Option<String> {
        let cmdline = CmdLine::from_bytes(line.as_bytes());
        cmdline
            .find_next(at)
            .map(|next| text(cmdline.rest_of(next)))
    }
    #[track_caller]
    fn check(line: &str, at: usize) -> Option<String> {
        let cmdline = CmdLine::from_bytes(line.as_bytes());
        cmdline
            .check_next(at)
            .map(|next| text(cmdline.rest_of(next)))
    }

    // The answer is what follows the separator, terminator included.
    assert_eq!(find("on | echo 1", 0), Some(" echo 1".to_string()));
    assert_eq!(find("on \n echo 1", 0), Some(" echo 1".to_string()));
    // No separator at all, and a separator before the offset, both answer
    // nothing: the search starts at `at`.
    assert_eq!(find("on", 0), None);
    assert_eq!(find("on | echo 1", 5), None);
    // A backslash does not escape it here, where `separate_nextcmd` would
    // have taken the bar into the argument.
    assert_eq!(find(r"on \| echo 1", 0), Some(" echo 1".to_string()));
    // The search stops at the line's own NUL, so a bar in a *later* string
    // in the same buffer is not found.
    assert_eq!(find("on", 3), None);

    // `check_next` is the other question: it skips white space and then
    // demands the separator be right there.
    assert_eq!(check("  | echo 1", 0), Some(" echo 1".to_string()));
    assert_eq!(check("on | echo 1", 0), None);
    assert_eq!(check("on | echo 1", 2), Some(" echo 1".to_string()));
}

#[test]
fn parse_cmd_reports_a_user_commands_name_and_every_argument() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    run(
        &editor,
        "command! -bar -nargs=* -range -bang Parsed let g:parsed = <q-args>",
    );

    // `nvim_parse_cmd` is the external observer of everything `do_one_cmd`
    // works out before it dispatches, and a `-nargs=*` user command is the
    // case where the argument list is split rather than handed over whole.
    assert_eq!(
        string(&editor, "nvim_parse_cmd('Parsed one two three', {}).cmd"),
        "Parsed"
    );
    assert_eq!(
        string(
            &editor,
            "join(nvim_parse_cmd('Parsed one two three', {}).args, ',')"
        ),
        "one,two,three"
    );
    // An escaped space keeps two words in one argument, and the escape is
    // gone by the time the argument is reported.
    assert_eq!(
        string(
            &editor,
            r"join(nvim_parse_cmd('Parsed one\ two three', {}).args, ',')"
        ),
        "one two,three"
    );
    // The bang and the range come back beside them.
    assert_eq!(num(&editor, "nvim_parse_cmd('2,4Parsed! x', {}).bang"), 1);
    assert_eq!(
        string(
            &editor,
            "join(nvim_parse_cmd('2,4Parsed! x', {}).range, ',')"
        ),
        "2,4"
    );
    // And a `|` is reported as the next command rather than swallowed --
    // which it only is because the command was declared `-bar`; without it
    // a bar is one of the command's own arguments.
    assert_eq!(
        string(&editor, "nvim_parse_cmd('Parsed x | echo 1', {}).nextcmd"),
        "echo 1"
    );

    run(&editor, "delcommand Parsed");
}
