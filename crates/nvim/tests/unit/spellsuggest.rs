//! Spelling suggestions: `spellsuggest()`, `z=` and the `'spellsuggest'`
//! methods, over a small dictionary of the case's own.
//!
//! `spell_suggest_list` scores candidates from the word tree (the edit
//! walk), the `REP` table and, for `double`/`best`, the sound-a-like pass,
//! then sorts them; `'spellsuggest'` picks the method, the count, a
//! `file:` of `bad/good` pairs consulted first and an `expr:` whose list is
//! merged in. The cases pin which words come back and in what order.
//!
//! Written against the behaviour before the suggestion state stops holding
//! pointers into the line and into the loaded languages; a row that moves
//! is a behaviour change. Each case builds the dictionary into its own
//! directory under a language name of its own, so a language another case
//! loaded is never the one read.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// The word list. Hunspell's first line is the count.
const DIC: &str = "14\nhello\nhelp\nheld\nhell\nyellow\nworld\nword\nwould\nwork\nspell\nspelling\n\
                   check\nchecker\nVim\n";

/// UTF-8, a `TRY` order for the edit walk, a `REP` pair, and two `SAL`
/// rules so `double` has a sound-a-like pass to run.
const AFF: &str = "SET UTF-8\nTRY lehowrdpyskcgnivmLEHOWRDPYSKCGNIVM\nREP 1\nREP ph f\n\
                   SAL followup true\nSAL collapse_result true\nSAL ck K\nSAL h H\n";

/// The dictionary built into `sandbox` as language `lang`, with 'spell' on
/// and the cursor's buffer holding `text`.
fn speller(fx: &Probe, sandbox: &Sandbox, lang: &str, text: &str) {
    fx.home();
    sandbox.mkdir("spell");
    sandbox.write(&format!("{lang}.dic"), DIC.as_bytes());
    sandbox.write(&format!("{lang}.aff"), AFF.as_bytes());
    let root = sandbox.as_str();
    fx.run("set mkspellmem=460000,2000,500");
    fx.run(&format!(
        "silent mkspell! {root}/spell/{lang}.utf-8.spl {root}/{lang}"
    ));
    fx.run("let g:unit_sug_rtp = &rtp");
    fx.run(&format!("set rtp^={root} spelllang={lang} spell"));
    fx.run(&format!("call setline(1, '{text}')"));
}

fn restore(fx: &Probe) {
    fx.run("set spell& spelllang& spellsuggest& spellcapcheck& mkspellmem&");
    fx.run("let &rtp = g:unit_sug_rtp | unlet g:unit_sug_rtp");
    fx.run("set nomodified");
    fx.home();
}

#[test]
fn suggestions_come_best_first() {
    let sandbox = Sandbox::dir("spellsuggest_best");
    let fx = Probe::new(sandbox.editor());
    speller(&fx, &sandbox, "sugbest", "");
    fx.expect_all(&[
        ("spellbadword('helo wrld')", "['helo', 'bad']"),
        (
            "spellsuggest('helo')",
            "['hello', 'held', 'hell', 'help', 'yellow', 'check', 'spell', 'word', 'work', \
             'world', 'would', 'Vim', 'checker', 'spelling']",
        ),
        ("spellsuggest('helo', 2)", "['hello', 'held']"),
        (
            "spellsuggest('wrld')",
            "['world', 'word', 'would', 'held', 'work', 'hell', 'help', 'spell', 'hello', \
             'Vim', 'yellow', 'check', 'spelling', 'checker']",
        ),
        (
            "spellsuggest('spellling')",
            "['spelling', 'spell', 'yellow', 'hello', 'hell', 'held', 'help', 'world', \
             'would', 'Vim', 'checker', 'check', 'word', 'work']",
        ),
        // A word the dictionary has is still answered: its neighbours.
        ("spellsuggest('help', 3)", "['Help', 'held', 'hell']"),
        (
            "spellsuggest('xqzt')",
            "['word', 'work', 'Vim', 'held', 'hell', 'help', 'spell', 'world', 'would', \
             'hello', 'check', 'yellow', 'checker', 'spelling']",
        ),
    ]);
    restore(&fx);
}

#[test]
fn case_is_kept_and_capital_is_asked_for() {
    let sandbox = Sandbox::dir("spellsuggest_case");
    let fx = Probe::new(sandbox.editor());
    speller(&fx, &sandbox, "sugcase", "");
    fx.expect_all(&[
        ("spellsuggest('Helo', 3)", "['Hello', 'Held', 'Hell']"),
        ("spellsuggest('HELO', 3)", "['HELLO', 'HELD', 'HELL']"),
        ("spellsuggest('helo', 3, 1)", "['Hello', 'Held', 'Hell']"),
        // A keep-case word keeps its own case.
        ("spellsuggest('vim', 3)", "['Vim', 'word', 'work']"),
    ]);
    restore(&fx);
}

#[test]
fn the_method_decides_the_list() {
    let sandbox = Sandbox::dir("spellsuggest_methods");
    let fx = Probe::new(sandbox.editor());
    speller(&fx, &sandbox, "sugmeth", "");
    // The count in the option is `z=`'s; `spellsuggest()` without a
    // maximum lists every candidate whatever it says.
    let long = "['check', 'hell', 'held', 'help', 'work', 'checker', 'hello', 'word', 'spell', \
                'Vim', 'world', 'would', 'yellow', 'spelling']";
    for (setting, want) in [
        (
            "fast",
            "['check', 'hell', 'work', 'held', 'help', 'checker']",
        ),
        (
            "double",
            "['check', 'held', 'hell', 'help', 'hello', 'work', 'checker', 'word', 'Vim', \
             'spell', 'world', 'would', 'yellow', 'spelling']",
        ),
        ("best", long),
        ("best,2", long),
        ("3", long),
    ] {
        fx.run(&format!("set spellsuggest={setting}"));
        fx.check("spellsuggest('chek')", want);
    }
    // `REP ph f`: the replacement table, not the edit walk.
    fx.run("set spellsuggest=best");
    fx.check("spellsuggest('phelp', 3)", "['help', 'held', 'hell']");
    restore(&fx);
}

#[test]
fn a_file_of_pairs_comes_first() {
    let sandbox = Sandbox::dir("spellsuggest_file");
    let fx = Probe::new(sandbox.editor());
    speller(&fx, &sandbox, "sugfile", "");
    sandbox.write("pairs.txt", b"helo/yellow\nwrld/work\nnot a pair\n");
    fx.run(&format!(
        "set spellsuggest=file:{}/pairs.txt,best",
        sandbox.as_str()
    ));
    fx.expect_all(&[
        ("spellsuggest('helo', 3)", "['yellow', 'hello', 'held']"),
        ("spellsuggest('wrld', 3)", "['work', 'world', 'word']"),
        ("spellsuggest('chek', 2)", "['check', 'hell']"),
    ]);
    // A missing file is an error, and the other methods still answer.
    fx.run("set spellsuggest=file:/nonexistent/pairs.txt,best");
    fx.check_error(
        "let g:unit_sug = spellsuggest('helo', 2)",
        "E484: Can''t open file /nonexistent/pairs.txt",
    );
    fx.check("g:unit_sug", "['hello', 'held']");
    fx.run("unlet g:unit_sug");
    restore(&fx);
}

#[test]
fn an_expression_is_merged_in() {
    let sandbox = Sandbox::dir("spellsuggest_expr");
    let fx = Probe::new(sandbox.editor());
    speller(&fx, &sandbox, "sugexpr", "");
    fx.run(
        "call execute(['function! UnitSugExpr() abort', \
         \"return [['spell', 1], ['wordy', 2], ['hello', 500]]\", 'endfunction'])",
    );
    fx.check("exists('*UnitSugExpr')", "1");
    fx.run("set spellsuggest=expr:UnitSugExpr(),fast");
    // `spell` is the expression's best and comes first; `wordy` is not in
    // the dictionary and is checked out again; `hello` is already there.
    fx.check(
        "spellsuggest('helo', 4)",
        "['spell', 'hello', 'held', 'hell']",
    );
    fx.run("delfunction UnitSugExpr");
    restore(&fx);
}

#[test]
fn z_equals_with_a_count_and_spellrepall() {
    let sandbox = Sandbox::dir("spellsuggest_zeq");
    let fx = Probe::new(sandbox.editor());
    speller(&fx, &sandbox, "sugzeq", "a helo there");
    fx.run("set spellsuggest=best,4");
    fx.run("call cursor(1, 3)");
    // A count picks that suggestion without the prompt (which would wait
    // for a key in the shared editor).
    fx.run("normal! 2z=");
    fx.check("getline(1)", "'a held there'");
    // `:spellrepall` repeats the replacement over the buffer.
    fx.run("call setline(2, 'helo again helo')");
    fx.check("execute('spellrepall')", "''");
    fx.check("getline(1, '$')", "['a held there', 'held again held']");
    restore(&fx);
}
