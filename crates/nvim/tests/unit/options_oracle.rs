//! What the option layer writes and lists, folded into digests: the `:set`
//! listings (`:set`, `:set all`, `:setlocal all`, `:setglobal all`, `:set!`),
//! the `:set` lines `:mkvimrc`, `:mkview` and `:mksession` write (the
//! escaping, `~` for `$HOME`, a long list split into `+=` lines, the guarded
//! `'syntax'`/`'filetype'`, the key names of `'wildchar'`), and every
//! `:set` operator over `support::corpus` for one option of each list shape,
//! through the command line, `:let &opt` and the API. Recorded before an
//! option value owned its string; the owned one has to reproduce every
//! digest (see `mbyte_oracle.rs` for the rule).

#![cfg(not(miri))]

use crate::support::corpus::{Digest, corpus};
use crate::support::{Probe, Sandbox};

/// The options whose defaults the machine decides (paths, the shell, the
/// screen), pinned so that a listing does not depend on where it ran.
const PINNED: &[(&str, &str)] = &[
    ("runtimepath", "/rt/a,/rt/b"),
    ("packpath", "/pp"),
    ("backupdir", ".,/state/backup//"),
    ("directory", "/state/swap//"),
    ("undodir", "/state/undo//"),
    ("viewdir", "/state/view//"),
    ("shell", "sh"),
    ("shellcmdflag", "-c"),
    ("shellpipe", "2>&1| tee"),
    ("shellredir", ">%s 2>&1"),
    ("shellquote", ""),
    ("shellxquote", ""),
    ("backupskip", "/tmp/*"),
    ("cdpath", ",,"),
    ("helpfile", "/rt/doc/help.txt"),
    ("helplang", "en"),
    ("grepprg", "grep -n $* /dev/null"),
    ("grepformat", "%f:%l:%m"),
    ("shada", "!,'100,<50,s10,h"),
    ("spellfile", ""),
    ("titleold", ""),
    ("fileencodings", "ucs-bom,utf-8,default,latin1"),
];

/// Every option back at its default, then [`PINNED`]; what each held at
/// both scopes kept in `g:oracle_saved`. One editor serves the whole unit
/// suite, and a case that leaves an option moved would otherwise move the
/// `:set` listing with it.
fn pin(fx: &Probe) {
    fx.run("let g:oracle_saved = []");
    fx.run(
        "for s:name in sort(keys(nvim_get_all_options_info())) | \
           call add(g:oracle_saved, [s:name, \
             nvim_get_option_value(s:name, {'scope': 'global'}), \
             nvim_get_option_value(s:name, {'scope': 'local'})]) | \
         endfor",
    );
    fx.run("silent! set all&");
    for (name, value) in PINNED {
        fx.run(&format!("let &{name} = '{value}'"));
    }
}

/// Put every option back as [`pin`] found it.
fn unpin(fx: &Probe) {
    fx.run(
        "for [s:name, s:global, s:local] in g:oracle_saved | \
           silent! call nvim_set_option_value(s:name, s:global, {'scope': 'global'}) | \
           silent! call nvim_set_option_value(s:name, s:local, {'scope': 'local'}) | \
         endfor",
    );
    fx.run("unlet g:oracle_saved");
}

/// The values the listings and writers are run over: every kind of value,
/// a local copy that differs from its global one, the bytes the writers
/// escape, a path under `$HOME`, and a list long enough to be split.
const SETUP: &[&str] = &[
    "set nowrapscan ignorecase shiftwidth=3 tabstop=5 textwidth=72",
    "set wildcharm=<C-Z> wildchar=<Esc>",
    "let &titlestring = \"tab\\there quote\\\" back\\\\slash bar| ctl\\x01 hi\\xff utf\\u00e9 nl\\nx\"",
    "set statusline=%f\\ %m%=%l",
    "let &tags = $HOME . '/tags,./tags,' . $HOME",
    "let &path = '.,' . repeat('/long/directory/name,', 220) . $HOME . '/end'",
    "set cpoptions+=n shortmess+=s matchpairs+=<:>",
    "set listchars=tab:>-,trail:~ fillchars=vert:\\|,fold:-",
    "setlocal path=local,,list undolevels=40 autoread number norelativenumber",
    "setlocal foldmethod=marker foldmarker={{{,}}} foldlevel=2 foldcolumn=1",
    "setlocal syntax=c filetype=c",
    "setglobal undolevels=77 noautoread",
    // Both follow the window and the buffer the rest of the suite left.
    "setlocal scroll=5 nomodified",
    "setglobal nomodified",
];

/// Put back what [`SETUP`] changed.
const TEARDOWN: &[&str] = &[
    "set wrapscan& ignorecase& shiftwidth& tabstop& textwidth&",
    "set wildcharm& wildchar&",
    "set titlestring& statusline& tags& path& cpoptions& shortmess& matchpairs&",
    "set listchars& fillchars&",
    "set undolevels& autoread& number& relativenumber&",
    "set foldmethod& foldmarker& foldlevel& foldcolumn&",
    "setlocal syntax= filetype=",
];

/// The `:set` lines of a written file: the rest names where it ran.
fn set_lines(text: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for line in text.split(|&b| b == b'\n') {
        if line.starts_with(b"set") || line.starts_with(b"if &") || line.starts_with(b"endif") {
            out.extend_from_slice(line);
            out.push(b'\n');
        }
    }
    out
}

#[test]
fn listings_and_written_set_lines() {
    let sandbox = Sandbox::dir("options_oracle_listings");
    let fx = Probe::new(sandbox.editor());
    fx.home();
    pin(&fx);
    for line in SETUP {
        fx.run(line);
    }
    let mut digests = Vec::new();

    for cmd in [
        "set",
        "set all",
        "setlocal all",
        "setglobal all",
        "set!",
        "setlocal",
        "setglobal",
        "set termcap",
        "set path? tags? titlestring? wildchar? wildcharm? undolevels? autoread?",
        "setlocal path? undolevels? autoread? foldmarker?",
        "setglobal path? undolevels? autoread?",
    ] {
        let mut d = Digest::new();
        if let Some(dump) = std::env::var_os("OPTIONS_ORACLE_DUMP") {
            let dump = std::path::PathBuf::from(dump);
            let _ = std::fs::write(dump.join(cmd.replace(['?', ' ', '!'], "_")), fx.output(cmd));
        }
        d.bytes(fx.output(cmd).as_bytes());
        digests.push(d.value());
    }

    let vimrc = sandbox.path("oracle.vimrc");
    fx.run(&format!("mkvimrc! {}", vimrc.display()));
    let text = std::fs::read(&vimrc).expect("the vimrc was written");
    if let Some(dump) = std::env::var_os("OPTIONS_ORACLE_DUMP") {
        let dump = std::path::PathBuf::from(dump);
        let _ = std::fs::write(dump.join("vimrc"), &text);
    }
    let mut d = Digest::new();
    d.bytes(&text);
    digests.push(d.value());

    fx.run("set viewoptions=options,folds");
    let view = sandbox.path("oracle.view");
    fx.run(&format!("mkview! {}", view.display()));
    let text = std::fs::read(&view).expect("the view was written");
    let mut d = Digest::new();
    d.bytes(&set_lines(&text));
    digests.push(d.value());
    fx.run("set viewoptions&");

    fx.run("set sessionoptions+=options,localoptions");
    let session = sandbox.path("oracle.session");
    fx.run(&format!("mksession! {}", session.display()));
    let text = std::fs::read(&session).expect("the session was written");
    let mut d = Digest::new();
    d.bytes(&set_lines(&text));
    digests.push(d.value());
    fx.run("set sessionoptions&");

    for line in TEARDOWN {
        fx.run(line);
    }
    unpin(&fx);
    fx.home();
    assert_eq!(digests, LISTINGS, "option listing digests moved");
}

/// A Vim script double-quoted literal of `text`, byte for byte.
fn literal(text: &[u8]) -> String {
    let hex: String = text.iter().map(|b| format!("\\x{b:02x}")).collect();
    format!("\"{hex}\"")
}

/// The options the operators run over: a plain string, a comma list, a
/// comma list of file names, a no-duplicates comma list, and a flag list.
const SHAPES: &[&str] = &[
    "titlestring",
    "cinwords",
    "dictionary",
    "wildignore",
    "breakat",
];

/// What the option and the last command say now, with the home directory
/// written out: `'dictionary'` expands a leading `~`.
fn state(fx: &Probe, name: &str) -> String {
    let text = format!(
        "{}|{}|{}|{}",
        fx.eval(&format!("&{name}")),
        fx.eval(&format!("&l:{name}")),
        fx.eval("v:errmsg"),
        fx.output(&format!("silent! set {name}?"))
    );
    match std::env::var("HOME") {
        Ok(home) if !home.is_empty() => text.replace(&home, "<H>"),
        _ => text,
    }
}

#[test]
fn every_operator_over_the_corpus() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.home();
    let texts: Vec<Vec<u8>> = corpus()
        .into_iter()
        .filter(|text| !text.contains(&0) && !text.contains(&b'\n'))
        .collect();
    fx.run("let g:texts = []");
    for text in &texts {
        fx.run(&format!("call add(g:texts, {})", literal(text)));
    }
    let n = texts.len();
    let mut digests = Vec::new();
    for name in SHAPES {
        fx.run(&format!("let g:oracle_kept = &{name}"));
        let mut d = Digest::new();
        for i in 0..n {
            let text = format!("g:texts[{i}]");
            for (start, how) in [
                ("", format!("execute 'set {name}=' . {text}")),
                (
                    "",
                    format!("execute 'set {name}=' . escape({text}, ' \\|\"')"),
                ),
                (
                    "a,b,a",
                    format!("execute 'set {name}+=' . escape({text}, ' \\')"),
                ),
                (
                    "a,b,a",
                    format!("execute 'set {name}^=' . escape({text}, ' \\')"),
                ),
                (
                    "a,b,a",
                    format!("execute 'set {name}-=' . escape({text}, ' \\')"),
                ),
                (
                    ",,x,",
                    format!("execute 'set {name}-=' . escape({text}, ' \\')"),
                ),
                ("", format!("let &{name} = {text}")),
                ("ab", format!("let &{name} .= {text}")),
                (
                    "ab",
                    format!("execute 'setlocal {name}+=' . escape({text}, ' \\')"),
                ),
                (
                    "",
                    format!("call nvim_set_option_value('{name}', {text}, {{}})"),
                ),
                (
                    "q",
                    format!("call nvim_set_option_value('{name}', {text}, {{'scope': 'global'}})"),
                ),
            ] {
                fx.run(&format!("let &{name} = '{start}'"));
                fx.run("let v:errmsg = ''");
                fx.run(&format!("silent! {how}"));
                d.bytes(state(&fx, name).as_bytes());
            }
        }
        digests.push(d.value());
        fx.run(&format!("let &{name} = g:oracle_kept"));
        fx.run(&format!("silent! set {name}&"));
    }
    fx.run("unlet! g:texts g:oracle_kept");
    fx.home();
    assert_eq!(digests, OPERATORS, "option operator digests moved");
}

/// Recorded before an option value owned its string.
const LISTINGS: [u64; 14] = [
    12_612_268_189_461_343_449,
    8_842_074_278_758_821_893,
    7_531_884_126_042_602_013,
    15_979_880_788_673_987_954,
    2_426_064_214_825_396_049,
    5_946_684_547_250_516_943,
    16_870_943_830_077_724_186,
    7_813_610_647_824_926_591,
    5_140_786_922_008_102_299,
    7_748_541_572_500_887_708,
    732_243_126_655_155_893,
    1_735_588_277_759_414_069,
    14_069_596_821_870_957_886,
    17_717_685_146_253_658_485,
];

/// Recorded before an option value owned its string. `'dictionary'`'s was
/// re-cut when the home directory was written out of the answers, from the
/// same answers: the old digest held on the home it was recorded under.
const OPERATORS: [u64; 5] = [
    17_641_244_318_484_148_716,
    7_872_315_180_379_759_583,
    13_586_010_553_937_603_543,
    6_950_669_116_562_947_449,
    9_401_097_154_406_692_227,
];
