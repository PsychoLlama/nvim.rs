//! `:set` end to end, through the fixture's command line: the argument
//! parser (`[no|inv]name[?|!|&|<|[+^-]=value]`, the backslash rules, the
//! errors), the `+=`/`^=`/`-=` splicing over plain, comma, flag and
//! `key:value` lists, the scope rules for a global-local option, and the
//! re-entrancy a setter has to survive: an `OptionSet` autocommand, or a
//! value evaluated while it is being set, that sets the same option again.
//!
//! Written against the behaviour before an option value owns its string; a
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
use crate::winlayer::Win;
use crate::winlayer::graph::{firstwin, lastwin};

/// The fixture's window as the whole of the current tab page's list: a
/// set reaches `comp_col`, which asks the first window for its status line.
struct Listed;

impl Listed {
    fn new() -> Listed {
        let id = Win::current().id();
        firstwin.set(Some(id));
        lastwin.set(Some(id));
        Listed
    }
}

impl Drop for Listed {
    fn drop(&mut self) {
        firstwin.set(None);
        lastwin.set(None);
    }
}

/// Put the options a case touches back to what the fixture had.
fn reset(fx: &Fixture) {
    fx.run("silent! autocmd! OptionSet");
    for name in [
        "path",
        "tags",
        "suffixes",
        "wildignore",
        "cpoptions",
        "shortmess",
        "whichwrap",
        "statusline",
        "keywordprg",
        "matchpairs",
        "isfname",
    ] {
        fx.run(&format!("silent! setglobal {name}="));
        fx.run(&format!("silent! setlocal {name}<"));
    }
    fx.run("silent! set cpoptions=aABceFs_");
}

#[test]
fn a_string_value_drops_the_backslashes_that_escape_it() {
    let fx = Fixture::new();
    let _listed = Listed::new();
    reset(&fx);
    fx.run(r"set path=a\ b,c\\d,e\,f,g\h");
    fx.check("&path", "'a b,c\\d,e,f,gh'");
    fx.run(r"set statusline=x\ \ y\|z");
    fx.check("&statusline", "'x  y|z'");
    fx.run(r"set statusline=trailing\\");
    fx.check("&statusline", "'trailing\\'");
    fx.run("set statusline=one two");
    fx.check("&statusline", "'one'");
    fx.check_error("set statusline=a b", "E518: Unknown option: b");
    fx.run("set statusline=");
    fx.check("&statusline", "'%<%f %h%w%m%r %{% v:lua.require(''vim._core.util'').term_exitcode() %}%=%{% luaeval(''(package.loaded[''''vim.ui''''] and vim.api.nvim_get_current_win() == tonumber(vim.g.actual_curwin or -1) and vim.ui.progress_status()) or '''''''' '')%}%{% &showcmdloc == ''statusline'' ? ''%-10.S '' : '''' %}%{% exists(''b:keymap_name'') ? ''<''..b:keymap_name..''> '' : '''' %}%{% &busy > 0 ? ''◐ '' : '''' %}%{% luaeval(''(package.loaded[''''vim.diagnostic''''] and next(vim.diagnostic.count()) and vim.diagnostic.status() .. '''' '''') or '''''''' '') %}%{% &ruler ? ( &rulerformat == '''' ? ''%-14.(%l,%c%V%) %P'' : &rulerformat ) : '''' %}'");
    // `:set opt=a=b` is one value with an `=` in it.
    fx.run("set statusline=a=b");
    fx.check("&statusline", "'a=b'");
    fx.run("set statusline:colon");
    fx.check("&statusline", "'colon'");
    reset(&fx);
}

#[test]
fn a_comma_list_adds_prepends_and_removes_whole_items() {
    let fx = Fixture::new();
    let _listed = Listed::new();
    reset(&fx);
    for (cmd, want) in [
        ("set path=a,b", "'a,b||'"),
        ("set path+=c", "'a,b,c||'"),
        ("set path^=z", "'z,a,b,c||'"),
        ("set path-=b", "'z,a,c||'"),
        ("set path-=z", "'a,c||'"),
        ("set path-=c", "'a||'"),
        ("set path-=nosuch", "'a||'"),
        ("set path+=a", "'a||'"),
        ("set path=a,", "'a,||'"),
        ("set path+=b", "'a,,b||'"),
        (r"set path=x\\,y", "'x\\,y||'"),
        ("set path+=q", "'x\\,y,q||'"),
        ("set path-=y", "'x\\,y,q||'"),
        (r"set path-=x\\,y", "'q||'"),
        ("set path=", "'||'"),
        ("set path+=one", "'one||'"),
        ("set path^=zero", "'zero,one||'"),
        ("set path-=one", "'zero||'"),
        ("set path-=zero", "'||'"),
        // A no-duplicates list keeps one copy of an item.
        ("set wildignore=*.o", "'|*.o|'"),
        ("set wildignore+=*.o", "'|*.o|'"),
        ("set wildignore^=*.o", "'|*.o|'"),
        ("set wildignore+=*.a,*.o", "'|*.o,*.a,*.o|'"),
        ("set wildignore-=*.o", "'|*.a,*.o|'"),
        ("set suffixes=.bak,~", "'|*.a,*.o|.bak,~'"),
        ("set suffixes+=.bak", "'|*.a,*.o|.bak,~'"),
        ("set suffixes-=~", "'|*.a,*.o|.bak'"),
    ] {
        fx.run(cmd);
        fx.check("&path . '|' . &wildignore . '|' . &suffixes", want);
    }
    reset(&fx);
}

#[test]
fn a_flag_list_drops_the_letters_it_already_has() {
    let fx = Fixture::new();
    let _listed = Listed::new();
    reset(&fx);
    for (cmd, want) in [
        ("set cpoptions=aA", "'aA||'"),
        ("set cpoptions+=a", "'Aa||'"),
        ("set cpoptions+=bb", "'Aab||'"),
        ("set cpoptions^=Ac", "'cAab||'"),
        ("set cpoptions-=A", "'cab||'"),
        ("set cpoptions-=ca", "'b||'"),
        ("set cpoptions-=c", "'b||'"),
        ("set cpoptions=aaa", "'a||'"),
        ("set shortmess=s", "'a|s|'"),
        ("set shortmess+=Is", "'a|Is|'"),
        ("set shortmess^=W", "'a|WIs|'"),
        ("set shortmess-=I", "'a|Ws|'"),
        ("set whichwrap=b,s", "'a|Ws|b,s'"),
        ("set whichwrap+=b", "'a|Ws|s,b'"),
        ("set whichwrap+=h,b", "'a|Ws|s,h,b'"),
        ("set whichwrap-=s", "'a|Ws|h,b'"),
        ("set whichwrap^=<", "'a|Ws|<,h,b'"),
    ] {
        fx.run(cmd);
        fx.check("&cpoptions . '|' . &shortmess . '|' . &whichwrap", want);
    }
    fx.run("set cpoptions=aABceFs_");
    reset(&fx);
}

#[test]
fn a_key_value_list_replaces_the_item_with_the_same_key() {
    let fx = Fixture::new();
    let _listed = Listed::new();
    reset(&fx);
    for (cmd, want) in [
        ("set matchpairs=(:),{:}", "'(:),{:}|'"),
        ("set matchpairs+=<:>", "'(:),{:},<:>|'"),
        ("set matchpairs+=(:)", "'(:),{:},<:>|'"),
        ("set matchpairs^=[:]", "'[:],(:),{:},<:>|'"),
        ("set matchpairs-={:}", "'[:],(:),<:>|'"),
        ("set matchpairs-=x:y", "'[:],(:),<:>|'"),
        ("set listchars=eol:$,tab:>-", "'[:],(:),<:>|eol:$,tab:>-'"),
        ("set listchars+=tab:xy", "'[:],(:),<:>|eol:$,tab:xy'"),
        (
            "set listchars^=tab:ab,trail:-",
            "'[:],(:),<:>|trail:-,tab:ab,eol:$'",
        ),
        ("set listchars-=eol:$", "'[:],(:),<:>|trail:-,tab:ab'"),
        ("set listchars-=trail:", "'[:],(:),<:>|tab:ab'"),
        ("set listchars+=eol:$,eol:%", "'[:],(:),<:>|tab:ab,eol:%'"),
    ] {
        fx.run(cmd);
        fx.check("&matchpairs . '|' . &listchars", want);
    }
    fx.run("set listchars=eol:$");
    reset(&fx);
}

#[test]
fn numbers_booleans_and_keys_take_their_own_spellings() {
    let fx = Fixture::new();
    let _listed = Listed::new();
    reset(&fx);
    for (cmd, want) in [
        ("set shiftwidth=5", "5"),
        ("set shiftwidth+=3", "8"),
        ("set shiftwidth-=2", "6"),
        ("set shiftwidth^=4", "24"),
        ("set shiftwidth=0x10", "16"),
        ("set shiftwidth=010", "8"),
        ("set shiftwidth=0b11", "3"),
        ("set shiftwidth=-0", "0"),
        ("set wildchar=<Tab>", "9"),
        ("set wildchar=^X", "24"),
        ("set wildchar=^@", "-22783"),
        ("set wildchar=x", "120"),
        ("set wildchar=<C-A>", "1"),
        ("set wildchar=65", "65"),
    ] {
        fx.run(cmd);
        let expr = if cmd.contains("wildchar") {
            "&wildchar"
        } else {
            "&shiftwidth"
        };
        fx.check(expr, want);
    }
    fx.check_output("set wildchar?", "\n  wildchar=A");
    fx.run("set wildchar=<Tab>");
    fx.check_output("set wildchar?", "\n  wildchar=<Tab>");
    fx.run("set wildchar=<Esc>");
    fx.check_output("set wildchar?", "\n  wildchar=<Esc>");
    fx.run("set wildchar=^A");
    fx.check_output("set wildchar?", "\n  wildchar=^A");
    fx.run("set wildchar=<Tab>");
    for (cmd, want) in [
        ("set wrapscan", "1"),
        ("set nowrapscan", "0"),
        ("set invwrapscan", "1"),
        ("set wrapscan!", "0"),
        ("set wrapscan&", "1"),
        ("set nowrapscan", "0"),
        ("set wrapscan&vim", "1"),
        ("set nowrapscan", "0"),
        ("set wrapscan&vi", "1"),
    ] {
        fx.run(cmd);
        fx.check("&wrapscan", want);
    }
    fx.check_output("set wrapscan?", "\n  wrapscan");
    fx.run("set nowrapscan");
    fx.check_output("set wrapscan?", "\nnowrapscan");
    fx.check_output("set wrapscan", "");
    fx.run("set wrapscan");
    fx.run("set shiftwidth=8");
    reset(&fx);
}

#[test]
fn the_parse_reports_what_it_refuses() {
    let fx = Fixture::new();
    let _listed = Listed::new();
    reset(&fx);
    for (cmd, want) in [
        ("set nosuchoption", "E518: Unknown option: nosuchoption"),
        ("set nosuchoption=1", "E518: Unknown option: nosuchoption=1"),
        ("set nonosuch", "E518: Unknown option: nonosuch"),
        ("set invnosuch", "E518: Unknown option: invnosuch"),
        (
            "set shiftwidth=abc",
            "E521: Number required after =: shiftwidth=abc",
        ),
        (
            "set shiftwidth=",
            "E521: Number required after =: shiftwidth=",
        ),
        (
            "set shiftwidth=3x",
            "E521: Number required after =: shiftwidth=3x",
        ),
        (
            "set shiftwidth=-1",
            "E487: Argument must be positive: shiftwidth=-1",
        ),
        ("set wrapscan=1", "E474: Invalid argument: wrapscan=1"),
        ("set wrapscan+=1", "E474: Invalid argument: wrapscan+=1"),
        ("set nopath", "E474: Invalid argument: nopath"),
        ("set invshiftwidth", "E474: Invalid argument: invshiftwidth"),
        ("set path!", "E488: Trailing characters: path!"),
        ("set wrapscan?x", "E488: Trailing characters: wrapscan?x"),
        ("set wrapscan&x", "E488: Trailing characters: wrapscan&x"),
        ("set wrapscanx", "E518: Unknown option: wrapscanx"),
        (
            "set wildchar=<Nosuch>",
            "E474: Invalid argument: wildchar=<Nosuch>",
        ),
        (
            "set cpoptions=\x01",
            "E539: Illegal character <^A>: cpoptions=^A",
        ),
        (
            "set shortmess=*",
            "E539: Illegal character <*>: shortmess=*",
        ),
        (
            "set whichwrap=z",
            "E539: Illegal character <z>: whichwrap=z",
        ),
        ("set matchpairs=ab", "E474: Invalid argument: matchpairs=ab"),
        (
            "set listchars=nosuch:x",
            "E474: Invalid argument: listchars=nosuch:x",
        ),
        ("set =", "E518: Unknown option: ="),
        ("set compatible", "E519: Option not supported: compatible"),
        ("set t_xx=abc", "none"),
        ("set <t_xx>=abc", "none"),
        ("set term=dumb", "E518: Unknown option: term=dumb"),
        ("set t_Co?", "none"),
        // The first refusal ends the line; what came before it stands.
        (
            "set shiftwidth=4 nosuch shiftwidth=6",
            "E518: Unknown option: nosuch ",
        ),
    ] {
        // `:set path!` shows the value, folding `$HOME` through libuv: not
        // under Miri.
        if cfg!(miri) && cmd == "set path!" {
            continue;
        }
        fx.check_error(cmd, want);
    }
    fx.check("&shiftwidth", "4");
    fx.run("set shiftwidth=8");
    fx.check("&cpoptions . '|' . &shortmess", "'aABceFs_|'");
    reset(&fx);
}

#[test]
fn a_global_local_option_reads_through_until_it_is_set_locally() {
    let fx = Fixture::new();
    let _listed = Listed::new();
    reset(&fx);
    for (cmd, want) in [
        ("setglobal path=g", "'g|g|'"),
        ("setlocal path=l", "'l|g|l'"),
        ("setlocal path+=m", "'l,m|g|l,m'"),
        ("setglobal path+=h", "'l,m|g,h|l,m'"),
        ("set path<", "'g,h|g,h|'"),
        ("setlocal path^=k", "'k,g,h|g,h|k,g,h'"),
        ("setlocal path<", "'g,h|g,h|g,h'"),
        ("set path=both", "'both|both|'"),
        ("setlocal path=", "'both|both|'"),
        ("setlocal path&", "'.,,|both|.,,'"),
        ("set path&", "'.,,|.,,|'"),
    ] {
        fx.run(cmd);
        fx.check("&path . '|' . &g:path . '|' . &l:path", want);
    }
    for (cmd, want) in [
        ("setglobal undolevels=44", "[0, 44, 0]"),
        ("setlocal undolevels=7", "[7, 44, 7]"),
        ("set undolevels<", "[44, 44, -123456]"),
        ("setlocal undolevels<", "[44, 44, 44]"),
        ("set undolevels&", "[1000, 1000, -123456]"),
        ("setglobal noautoread", "[0, 0, 0]"),
        ("setlocal autoread", "[1, 0, 1]"),
        ("set autoread<", "[0, 0, -1]"),
        ("setlocal autoread!", "[1, 0, 1]"),
        ("set autoread&", "[1, 1, -1]"),
    ] {
        fx.run(cmd);
        let expr = if cmd.contains("undolevels") {
            "[&undolevels, &g:undolevels, &l:undolevels]"
        } else {
            "[&autoread, &g:autoread, &l:autoread]"
        };
        fx.check(expr, want);
    }
    // Showing a path-like value folds `$HOME`, which asks libuv for the
    // environment: not under Miri.
    if !cfg!(miri) {
        fx.run("setglobal path=x");
        fx.check_output("setlocal path?", "\n  path=");
        fx.check_output("setglobal path?", "\n  path=x");
        fx.run("setlocal path=y");
        fx.check_output("setlocal path?", "\n  path=y");
        fx.check_output("set path?", "\n  path=y");
        fx.check_output("setglobal path?", "\n  path=x");
    }
    reset(&fx);
}

#[test]
fn a_bare_keywordprg_means_help_only_for_the_global_value() {
    let fx = Fixture::new();
    let _listed = Listed::new();
    reset(&fx);
    fx.run("set keywordprg=");
    fx.check("[&g:keywordprg, &l:keywordprg]", "['', '']");
    fx.run("setlocal keywordprg=");
    fx.check("[&g:keywordprg, &l:keywordprg]", "['', '']");
    fx.run("setlocal keywordprg=man");
    fx.check("[&g:keywordprg, &l:keywordprg]", "['', 'man']");
    reset(&fx);
}

/// `OptionSet` runs once the new value is in, and may set the same option
/// again: the setter's old value has to be its own, not a view of the
/// variable the autocommand just rewrote.
#[test]
fn an_optionset_autocommand_may_set_the_same_option_again() {
    let fx = Fixture::new();
    let _listed = Listed::new();
    reset(&fx);
    let starting = crate::startup::starting.replace(0);
    fx.run("let g:seen = []");
    fx.block(&[
        "autocmd OptionSet path call add(g:seen, [v:option_old, v:option_new, v:option_oldlocal, v:option_oldglobal, v:option_type, v:option_command, &path]) | set path+=z",
    ]);
    fx.run("set path=a,b");
    fx.check("&path", "'a,b,z'");
    fx.check("g:seen", "[['', 'a,b', '', '', 'global', 'set', 'a,b']]");
    fx.run("let g:seen = []");
    fx.run("setlocal path=q");
    fx.check("[&path, &g:path]", "['q,z', 'q,z']");
    fx.check(
        "g:seen",
        "[['a,b,z', 'q', 'a,b,z', '', 'local', 'setlocal', 'q']]",
    );
    fx.run("autocmd! OptionSet");
    // The autocommand replaces the value outright, several times over.
    fx.run("let g:n = 0");
    fx.block(&[
        "autocmd OptionSet statusline let g:n += 1 | if g:n < 4 | let &statusline = 'round' . g:n | endif",
    ]);
    fx.run("set statusline=start");
    fx.check("[&statusline, g:n]", "['round1', 1]");
    fx.run("autocmd! OptionSet");
    // A flag list and a number, the same way.
    fx.block(&["autocmd OptionSet shortmess,shiftwidth if v:option_new !~ 'q' && v:option_new != 2 | execute 'set ' . (expand('<amatch>') == 'shortmess' ? 'shortmess+=q' : 'shiftwidth=2') | endif"]);
    fx.run("set shortmess=s");
    fx.run("set shiftwidth=7");
    fx.check("[&shortmess, &shiftwidth]", "['sq', 2]");
    fx.run("autocmd! OptionSet");
    fx.run("set shiftwidth=8 shortmess=");
    crate::startup::starting.set(starting);
    fx.run("unlet! g:seen g:n");
    reset(&fx);
}

/// A function option's value is evaluated while the option is being set;
/// evaluating it can set the same option again.
#[test]
fn a_function_option_evaluated_while_it_is_set_may_set_itself() {
    let fx = Fixture::new();
    let _listed = Listed::new();
    reset(&fx);
    fx.block(&["function! OptionTestF(...)", "  return []", "endfunction"]);
    fx.run("let &tagfunc = \"function('OptionTestF', [execute('set tagfunc=OptionTestF')])\"");
    fx.check("&tagfunc", "'OptionTestF'");
    fx.run("let &tagfunc = \"function('OptionTestF', [execute('set tagfunc=')])\"");
    fx.check("&tagfunc", "''");
    fx.run("set tagfunc=");
    fx.check("&tagfunc", "''");
    fx.delete(&["OptionTestF"]);
    reset(&fx);
}
