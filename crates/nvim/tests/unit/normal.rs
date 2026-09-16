//! Normal mode: what a keystroke becomes, and what running it does.
//!
//! `normal` had no in-crate test at all. The functional and oldtest suites
//! reach it through every key they send, but nothing in the crate pinned the
//! decisions `normal_execute` makes before the handler runs -- the count the
//! digits stack up, the register a `"` claims, the second character a table
//! row asks for, and the `NV_*` flags that say whether either happens -- nor
//! the bookkeeping `normal_finish_command` does after it.
//!
//! One engine: [`keys`] runs a key sequence through `:normal`, which is
//! `exec_normal` -> `normal_cmd` -> `normal_prepare` + `normal_execute` +
//! `normal_finish_command`, the whole per-key path. What a case asserts is
//! read back out of the editor with an expression, so the assertion is
//! about the editor's state and not about a field.
//!
//! `normal_check` is the other half of the `VimState` pair and no unit test
//! can reach it: it is the idle work `state_enter` runs between keys, and
//! the only way in is the main loop. The functional suite is its oracle.

#![cfg(not(miri))]

use crate::support::{Editor, cstr, editor_lock};
use neovim::eval::{eval_to_number, eval_to_string};
use neovim::ex_docmd::do_cmdline_cmd;
use neovim::types::Pos;
use neovim::winlayer::Win;

/// The buffer most cases are written against.
const LINES: &str =
    r"call setline(1, ['alpha beta', 'gamma delta', 'epsilon zeta', 'eta theta', 'iota kappa'])";

/// Puts the buffer, the cursor and the editor's per-command leftovers back.
///
/// There is one editor per test process, so a case that fills the buffer,
/// moves the cursor, leaves a register loaded or installs an autocommand is
/// visible to every case after it -- including the ones next door in
/// `ex_docmd.rs`, which read the cursor.
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
        for line in [
            "silent! autocmd! nvim_unit_normal",
            "silent! augroup! nvim_unit_normal",
            "silent! unlet g:seen",
            "silent! normal! \u{1b}",
            "silent! keepjumps keepmarks %delete _",
        ] {
            let text = cstr(line);
            // SAFETY: `text` is NUL-terminated and outlives the call.
            let _ = unsafe { do_cmdline_cmd(text.as_ptr()) };
        }
        Win::current().w_cursor = self.cursor;
    }
}

/// Run one Ex command line.
fn run(_editor: &Editor, line: &str) {
    let text = cstr(line);
    // SAFETY: `text` is NUL-terminated and outlives the call.
    let _ = unsafe { do_cmdline_cmd(text.as_ptr()) };
}

/// Send `seq` to normal mode with no mappings, the way `:normal!` does.
///
/// The sequence is written as a Vimscript double-quoted string, so `\<Esc>`
/// and friends name themselves.
fn keys(editor: &Editor, seq: &str) {
    run(editor, &format!("execute \"normal! {seq}\""));
}

/// As [`keys`], with mappings applied -- `:normal` without the bang.
fn mapped_keys(editor: &Editor, seq: &str) {
    run(editor, &format!("execute \"normal {seq}\""));
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

/// The whole buffer, one line per element, joined with `/`.
fn text(editor: &Editor) -> String {
    string(editor, "join(getline(1, '$'), '/')")
}

/// The cursor, as (line, column) with the column one-based the way
/// `col()` reports it.
fn cursor(editor: &Editor) -> (i64, i64) {
    (num(editor, "line('.')"), num(editor, "col('.')"))
}

/// A case's own buffer: [`LINES`], cursor at the top.
fn fresh(editor: &Editor) {
    run(editor, "silent! keepjumps keepmarks %delete _");
    run(editor, LINES);
    run(editor, "1");
    keys(editor, "0");
}

// ---------------------------------------------------------------------------
// Counts

#[test]
fn digits_before_a_command_stack_up_into_its_count() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    keys(&editor, "3j");
    assert_eq!(cursor(&editor).0, 4);
    // Two digits are one count, not two commands.
    fresh(&editor);
    keys(&editor, "3lx");
    assert_eq!(text(&editor).split('/').next(), Some("alpa beta"));
}

#[test]
fn an_operator_count_and_a_motion_count_multiply() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    // `2d3w` is six words, which is this line and the next two.
    keys(&editor, "2d3w");
    assert_eq!(text(&editor), "eta theta/iota kappa");
    // The count in front of the operator alone.
    fresh(&editor);
    keys(&editor, "3dd");
    assert_eq!(text(&editor), "eta theta/iota kappa");
    // The count after the operator alone.
    fresh(&editor);
    keys(&editor, "d3d");
    assert_eq!(text(&editor), "eta theta/iota kappa");
}

#[test]
fn a_count_is_published_as_v_count_and_v_count1() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    // Through a mapping, because a count in front of a bare `:` becomes a
    // *range* rather than a count -- which is itself the parse deciding
    // what the digits were for.
    run(
        &editor,
        "nnoremap <buffer> gC :<C-u>let g:seen = [v:count, v:count1]<CR>",
    );
    mapped_keys(&editor, "3gC");
    assert_eq!(string(&editor, "string(g:seen)"), "[3, 3]");
    // No count at all is 0 and 1.
    mapped_keys(&editor, "gC");
    assert_eq!(string(&editor, "string(g:seen)"), "[0, 1]");
    // And an operator count and a motion count reach it multiplied.
    run(
        &editor,
        "nnoremap <buffer> gD :<C-u>let g:seen = v:count<CR>",
    );
    mapped_keys(&editor, "2\\\"a3gD");
    assert_eq!(num(&editor, "g:seen"), 6);
    run(&editor, "silent! nunmap <buffer> gC");
    run(&editor, "silent! nunmap <buffer> gD");
}

// ---------------------------------------------------------------------------
// Registers

#[test]
fn a_double_quote_names_the_register_the_next_command_uses() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    keys(&editor, "\\\"ayy");
    assert_eq!(string(&editor, "@a"), "alpha beta\n");
    // An upper-case name appends to the same register.
    keys(&editor, "j\\\"Ayy");
    assert_eq!(string(&editor, "@a"), "alpha beta\ngamma delta\n");
}

#[test]
fn a_command_that_does_not_keep_the_register_releases_it() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    run(&editor, "let @a = 'kept'");
    // `j` has no NV_KEEPREG, so the register named in front of it is
    // dropped rather than carried to the yank after it.
    keys(&editor, "\\\"ajyy");
    assert_eq!(string(&editor, "@a"), "kept");
    assert_eq!(string(&editor, "@\""), "gamma delta\n");
    // `x` *does* claim NV_KEEPREG, so the register in front of it is the
    // one it writes to.
    keys(&editor, "\\\"bx");
    assert_eq!(string(&editor, "@b"), "g");
}

// ---------------------------------------------------------------------------
// Operator-pending

#[test]
fn an_operator_applies_to_the_motion_that_follows_it() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    keys(&editor, "wc$END\\<Esc>");
    assert_eq!(text(&editor).split('/').next(), Some("alpha END"));
    fresh(&editor);
    run(&editor, "setlocal shiftwidth=4 expandtab");
    keys(&editor, ">>");
    assert_eq!(text(&editor).split('/').next(), Some("    alpha beta"));
}

#[test]
fn a_text_object_is_read_as_the_operators_argument() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    keys(&editor, "wdiw");
    assert_eq!(text(&editor).split('/').next(), Some("alpha "));
    // `a`/`i` are only text objects while an operator or a selection is
    // waiting; on their own they start insert mode.
    fresh(&editor);
    keys(&editor, "iX\\<Esc>");
    assert_eq!(text(&editor).split('/').next(), Some("Xalpha beta"));
}

// ---------------------------------------------------------------------------
// The second character

#[test]
fn a_command_flagged_nv_nch_reads_a_second_character() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    // `f<char>` searches the line for it.
    keys(&editor, "fb");
    assert_eq!(cursor(&editor), (1, 7));
    // `r<char>` replaces under the cursor.
    keys(&editor, "rB");
    assert_eq!(text(&editor).split('/').next(), Some("alpha Beta"));
}

#[test]
fn a_mark_is_set_and_jumped_to_by_its_second_character() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    keys(&editor, "3jma");
    keys(&editor, "gg");
    assert_eq!(cursor(&editor).0, 1);
    keys(&editor, "`a");
    assert_eq!(cursor(&editor).0, 4);
    keys(&editor, "gg'a");
    assert_eq!(cursor(&editor).0, 4);
}

#[test]
fn nv_nch_nop_reads_no_second_character_while_an_operator_is_pending() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    // `m` is NV_NCH_NOP: on its own it takes a mark name, but with `d`
    // pending it is not a motion, so the operator is abandoned and the
    // `a` that follows is read as a command of its own -- insert.
    keys(&editor, "dma");
    assert_eq!(text(&editor).split('/').next(), Some("alpha beta"));
    assert_eq!(num(&editor, "line('$')"), 5);
    // With no operator pending the same `m` does take a mark name.
    keys(&editor, "jjmb");
    assert_eq!(num(&editor, "line(\"'b\")"), 3);
}

// ---------------------------------------------------------------------------
// The prefixed tables

#[test]
fn the_g_prefix_reaches_its_own_commands() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    keys(&editor, "guu");
    assert_eq!(text(&editor).split('/').next(), Some("alpha beta"));
    keys(&editor, "gUU");
    assert_eq!(text(&editor).split('/').next(), Some("ALPHA BETA"));
    keys(&editor, "g~~");
    assert_eq!(text(&editor).split('/').next(), Some("alpha beta"));
    // `gJ` joins without inserting a space, which `J` would.
    keys(&editor, "gJ");
    assert_eq!(
        text(&editor).split('/').next(),
        Some("alpha betagamma delta")
    );
    // A third character: `gr` replaces without affecting the layout.
    fresh(&editor);
    keys(&editor, "grX");
    assert_eq!(text(&editor).split('/').next(), Some("Xlpha beta"));
}

#[test]
fn the_z_prefix_reads_a_second_character_and_sometimes_a_count() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    run(&editor, "setlocal foldmethod=manual");
    keys(&editor, "zfj");
    assert_eq!(num(&editor, "foldclosed(1)"), 1);
    assert_eq!(num(&editor, "foldclosedend(1)"), 2);
    keys(&editor, "zo");
    assert_eq!(num(&editor, "foldclosed(1)"), -1);
    run(&editor, "setlocal foldmethod=manual foldlevel=0");
    run(&editor, "normal! zE");
}

#[test]
fn the_ctrl_w_prefix_takes_its_own_count_after_the_key() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    run(&editor, "silent! only");
    keys(&editor, "\\<C-w>s");
    assert_eq!(num(&editor, "winnr('$')"), 2);
    // A count *after* CTRL-W picks the window by number.
    keys(&editor, "\\<C-w>2\\<C-w>\\<C-w>");
    assert_eq!(num(&editor, "winnr()"), 2);
    run(&editor, "silent! only");
    assert_eq!(num(&editor, "winnr('$')"), 1);
}

#[test]
fn the_z_capital_prefix_is_a_command_of_its_own() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    // `ZZ` would write and quit; `Z` followed by something it does not
    // know beeps and leaves the buffer alone.
    keys(&editor, "Zq");
    assert_eq!(text(&editor).split('/').next(), Some("alpha beta"));
}

// ---------------------------------------------------------------------------
// Repeat

#[test]
fn a_dot_repeats_the_last_change_with_its_own_count() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    keys(&editor, "x");
    keys(&editor, ".");
    assert_eq!(text(&editor).split('/').next(), Some("pha beta"));
    // A count in front of `.` replaces the original one.
    keys(&editor, "3.");
    assert_eq!(text(&editor).split('/').next(), Some(" beta"));
    // `.` itself is NV_KEEPREG, so the register a repeat uses is the one
    // the original command was given.
    fresh(&editor);
    keys(&editor, "\\\"adw");
    keys(&editor, ".");
    assert_eq!(string(&editor, "@a"), "beta");
}

// ---------------------------------------------------------------------------
// Visual mode

#[test]
fn an_operator_over_a_visual_selection_takes_the_selection_as_its_range() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    keys(&editor, "Vjd");
    assert_eq!(text(&editor), "epsilon zeta/eta theta/iota kappa");
    // Characterwise.
    fresh(&editor);
    keys(&editor, "vlld");
    assert_eq!(text(&editor).split('/').next(), Some("ha beta"));
    // Blockwise, over three lines.
    fresh(&editor);
    keys(&editor, "\\<C-v>jjd");
    assert_eq!(
        text(&editor),
        "lpha beta/amma delta/psilon zeta/eta theta/iota kappa"
    );
}

#[test]
fn gv_reselects_what_the_last_visual_command_covered() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    keys(&editor, "vjy");
    // The yank put the cursor back at the start, so `gv` is the same
    // selection again: from (1, 1) to (2, 1), charwise.
    keys(&editor, "gvd");
    assert_eq!(
        text(&editor),
        "amma delta/epsilon zeta/eta theta/iota kappa"
    );
}

// ---------------------------------------------------------------------------
// Re-entrancy

#[test]
fn a_mode_changed_autocommand_runs_inside_the_command_that_changed_it() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    run(&editor, "let g:seen = []");
    run(
        &editor,
        "augroup nvim_unit_normal | autocmd! | autocmd ModeChanged * call add(g:seen, v:event.old_mode .. '>' .. v:event.new_mode) | augroup END",
    );
    // `d` enters operator-pending and `w` leaves it again, both inside one
    // `normal_execute` pass.
    keys(&editor, "dw");
    assert_eq!(string(&editor, "string(g:seen)"), "['n>no', 'no>n']");
    assert_eq!(text(&editor).split('/').next(), Some("beta"));
}

#[test]
fn an_autocommand_that_runs_a_command_of_its_own_nests() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    run(&editor, "let g:seen = 0");
    run(
        &editor,
        "augroup nvim_unit_normal | autocmd! | autocmd TextChanged * let g:seen += 1 | augroup END",
    );
    // `:normal` inside the command's own autocommand re-enters `normal_cmd`
    // with a second `CmdArg` one frame down.
    keys(&editor, "x");
    run(&editor, "doautocmd TextChanged");
    assert_eq!(num(&editor, "g:seen"), 1);
}

// ---------------------------------------------------------------------------
// Refusals

#[test]
fn a_character_no_row_answers_to_clears_the_pending_operator() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    // CTRL-K is `nv_error`: with `d` pending the operator is abandoned,
    // and the `x` after it deletes one character rather than a line.
    keys(&editor, "d\\<C-k>x");
    // The beep that reports the refusal also flushes the typeahead, so the
    // `x` after it never runs -- and the line is untouched either way.
    assert_eq!(text(&editor).split('/').next(), Some("alpha beta"));
    assert_eq!(num(&editor, "line('$')"), 5);
    // With the operator gone, the same `x` on its own does delete.
    keys(&editor, "x");
    assert_eq!(text(&editor).split('/').next(), Some("lpha beta"));
}

#[test]
fn an_escape_in_place_of_the_second_character_abandons_the_command() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    keys(&editor, "r\\<Esc>");
    assert_eq!(text(&editor).split('/').next(), Some("alpha beta"));
    // And so does an Escape where a mark name was expected.
    keys(&editor, "m\\<Esc>x");
    assert_eq!(text(&editor).split('/').next(), Some("lpha beta"));
}

// ---------------------------------------------------------------------------
// Mappings

#[test]
fn a_mapping_is_expanded_before_the_table_is_consulted() {
    let editor = editor_lock();
    let _restore = Restore::new(&editor);
    fresh(&editor);
    run(&editor, "nnoremap <buffer> gQ 3dd");
    mapped_keys(&editor, "gQ");
    assert_eq!(text(&editor), "eta theta/iota kappa");
    run(&editor, "silent! nunmap <buffer> gQ");
}
