//! What the string options' `did_set_*` callbacks accept, reject and leave
//! behind: every callback with a good value and a bad one, the exact message
//! each rejection reports, the value the option holds afterwards, and the
//! 'listchars'/'fillchars' glyph tables and the 'statusline' validator over
//! `support::corpus`. Recorded on the callbacks that read their value
//! through a raw pointer; the ones that read it as a slice have to
//! reproduce every row and every digest.

#![cfg(not(miri))]

use crate::support::corpus::{Digest, corpus};
use crate::support::{Probe, Sandbox};
use neovim::winlayer::Win;

/// One case: the option it moves, and the command that moves it.
///
/// The answer is `error|global|local`: the message the command left in
/// `v:errmsg` (`none` for none) and what `&opt` and `&l:opt` say after it.
/// Every case starts from the option's default at both scopes.
const CASES: &[(&str, &str)] = &[
    // 'backspace': a word list, or the number 2.
    ("backspace", "set bs=2"),
    ("backspace", "set bs=3"),
    ("backspace", "set bs=0"),
    ("backspace", "set bs=indent,eol,nostop"),
    ("backspace", "set bs=indent,foo"),
    // 'backupcopy': exactly one of yes/no/auto.
    ("backupcopy", "set bkc=yes"),
    ("backupcopy", "set bkc=yes,no"),
    ("backupcopy", "set bkc=breaksymlink"),
    ("backupcopy", "set bkc=auto,breakhardlink"),
    ("backupcopy", "setlocal bkc=no"),
    ("backupcopy", "setlocal bkc="),
    ("backupcopy", "setlocal bkc=bogus"),
    ("backupcopy", "setglobal bkc=no,yes"),
    // 'backupext' and 'patchmode' must differ, a leading dot aside.
    ("patchmode", "set bex=orig | set pm=.orig"),
    ("patchmode", "set pm=.orig"),
    ("backupext", "set pm=~ | set bex=~"),
    // 'bufhidden', 'buftype'.
    ("bufhidden", "setlocal bh=wipe"),
    ("bufhidden", "setlocal bh=wipe,hide"),
    ("bufhidden", "setlocal bh=x"),
    ("buftype", "setlocal bt=nofile"),
    ("buftype", "setlocal bt=terminal"),
    ("buftype", "setlocal bt=bogus"),
    ("buftype", "setlocal bt=help"),
    ("buftype", "setlocal bt=prompt | setlocal bt="),
    // 'comments': `{flags}:{leader}` parts.
    ("comments", "setlocal com=b:#,://"),
    ("comments", "setlocal com=b"),
    ("comments", "setlocal com=b:"),
    ("comments", "setlocal com=b:,:x"),
    ("comments", "setlocal com=q:x"),
    ("comments", "setlocal com=q"),
    ("comments", "setlocal com=s1:/*,mb:*,ex:*/"),
    ("comments", "setlocal com=-5:x"),
    ("comments", "setlocal com=:a\\\\,b,:c"),
    ("comments", "setlocal com=:a\\\\"),
    ("comments", "setlocal com=,:x"),
    ("comments", "setlocal com=:x,,:y"),
    // 'commentstring'.
    ("commentstring", "setlocal cms=/*%s*/"),
    ("commentstring", "setlocal cms=foo"),
    ("commentstring", "setlocal cms="),
    ("commentstring", "setlocal cms=%"),
    // Flag-letter options.
    ("cpoptions", "set cpo=aA"),
    ("cpoptions", "set cpo=a1"),
    ("cpoptions", "set cpo+=,"),
    ("formatoptions", "setlocal fo=tcq"),
    ("formatoptions", "setlocal fo=tc%"),
    ("formatoptions", "setlocal fo=\\ t"),
    ("concealcursor", "setlocal cocu=nv"),
    ("concealcursor", "setlocal cocu=x"),
    ("mouse", "set mouse=a"),
    ("mouse", "set mouse=nvz"),
    ("whichwrap", "set ww=b,s,h,l"),
    ("whichwrap", "set ww=<,>,z"),
    ("shortmess", "set shm=aI"),
    ("shortmess", "set shm=aZ"),
    // 'encoding' and 'fileencoding'.
    ("encoding", "set enc=latin1"),
    ("encoding", "set enc=utf8"),
    ("encoding", "set enc=UTF-8"),
    ("fileencoding", "setlocal fenc=LATIN1"),
    ("fileencoding", "setlocal fenc=utf8"),
    ("fileencoding", "setlocal fenc=a,b"),
    ("fileencoding", "setlocal fenc=ucs-2le"),
    (
        "fileencoding",
        "setlocal nomodifiable | setlocal fenc=latin1",
    ),
    ("fileencoding", "setlocal modifiable"),
    ("makeencoding", "set menc=char"),
    ("makeencoding", "set menc=CP1252"),
    // 'eventignore' and 'eventignorewin'.
    ("eventignore", "set ei=all"),
    ("eventignore", "set ei=BufRead,WinEnter"),
    ("eventignore", "set ei=BufRead,bogus"),
    ("eventignore", "set ei=-BufRead,all"),
    ("eventignore", "set ei=BufRead,"),
    ("eventignorewin", "setlocal eiw=WinEnter"),
    ("eventignorewin", "setlocal eiw=all"),
    ("eventignorewin", "setlocal eiw=BufRead"),
    ("eventignorewin", "setlocal eiw=bogus"),
    // 'fileformat'.
    ("fileformat", "setlocal ff=dos"),
    ("fileformat", "setlocal ff=foo"),
    ("fileformat", "setlocal ff="),
    ("fileformat", "setlocal ff=mac | setlocal ff=unix"),
    ("fileformat", "setlocal nomodifiable | setlocal ff=dos"),
    ("fileformat", "setlocal modifiable"),
    ("fileformats", "set ffs=unix,dos,mac"),
    ("fileformats", "set ffs=unix,unix"),
    ("fileformats", "set ffs=unix,"),
    // 'filetype', 'syntax', 'keymap'.
    ("filetype", "setlocal ft=c"),
    ("filetype", "setlocal ft=a/b"),
    ("filetype", "setlocal ft=c.doxygen"),
    ("filetype", "setlocal ft=a\\ b"),
    ("syntax", "setlocal syn=foo-bar_1"),
    ("syntax", "setlocal syn=a*"),
    ("keymap", "setlocal kmp=a/b"),
    ("keymap", "setlocal kmp=no_such_keymap"),
    // Folding.
    ("foldmarker", "setlocal fmr=<<,>>"),
    ("foldmarker", "setlocal fmr=ab"),
    ("foldmarker", "setlocal fmr=,b"),
    ("foldmarker", "setlocal fmr=a,"),
    ("foldmarker", "setlocal fmr=a,b,c"),
    ("foldmethod", "setlocal fdm=indent"),
    ("foldmethod", "setlocal fdm=x"),
    ("foldmethod", "setlocal fdm=diff"),
    ("foldignore", "setlocal fdi=#;"),
    ("foldexpr", "setlocal fde=1"),
    ("foldexpr", "setlocal fde=s:Nope()"),
    ("foldopen", "set fdo=all"),
    ("foldopen", "set fdo=bogus"),
    // Character classes.
    ("iskeyword", "setlocal isk=@,48-57,_"),
    ("iskeyword", "setlocal isk=300"),
    ("iskeyword", "setlocal isk=a-"),
    ("iskeyword", "setglobal isk=@,^a"),
    ("iskeyword", "setglobal isk=1-"),
    ("isfname", "set isf=@,48-57,/"),
    ("isfname", "set isf=x-"),
    ("isprint", "set isp=@,161-255"),
    ("isident", "set isi=@,48-57,_,192-255"),
    // 'lispoptions', 'matchpairs'.
    ("lispoptions", "setlocal lop=expr:1"),
    ("lispoptions", "setlocal lop=expr:0"),
    ("lispoptions", "setlocal lop=expr:2"),
    ("matchpairs", "setlocal mps=(:),<:>"),
    ("matchpairs", "setlocal mps=("),
    ("matchpairs", "setlocal mps=(;)"),
    ("matchpairs", "setlocal mps=(:"),
    ("matchpairs", "setlocal mps=\u{e9}:\u{e8},\u{ab}:\u{bb}"),
    ("matchpairs", "setlocal mps=(:),"),
    ("matchpairs", "setlocal mps=(:)x"),
    ("matchpairs", "setlocal mps=a\u{301}:b"),
    // 'vartabstop', 'varsofttabstop'.
    ("vartabstop", "setlocal vts=4,8"),
    ("vartabstop", "setlocal vts=4,,8"),
    ("vartabstop", "setlocal vts=0"),
    ("vartabstop", "setlocal vts="),
    ("vartabstop", "setlocal vts=,4"),
    ("vartabstop", "setlocal vts=4,0"),
    ("vartabstop", "setlocal vts=99999999999"),
    ("vartabstop", "setlocal vts=8 | setlocal vts=x"),
    ("varsofttabstop", "setlocal vsts=2,4"),
    ("varsofttabstop", "setlocal vsts=-1"),
    ("varsofttabstop", "setlocal vsts=a"),
    // Completion.
    ("complete", "setlocal cpt=.,w,b,u,t,i"),
    ("complete", "setlocal cpt=x"),
    ("complete", "setlocal cpt=.^5,w^10"),
    ("complete", "setlocal cpt=.^"),
    ("complete", "setlocal cpt=.^x"),
    ("complete", "setlocal cpt=.x"),
    ("complete", "setlocal cpt=kfoo,sbar"),
    ("complete", "setlocal cpt=k\\\\,foo"),
    ("complete", "setlocal cpt=.,\\ ,w"),
    ("complete", "setlocal cpt="),
    ("complete", "setlocal cpt=o"),
    ("complete", "setlocal cpt=FNoSuchFunction"),
    ("completeitemalign", "set cia=kind,abbr,menu"),
    ("completeitemalign", "set cia=abbr,abbr,menu"),
    ("completeitemalign", "set cia=abbr,kind"),
    ("completeitemalign", "set cia=abbr,kind,menu,abbr"),
    ("completeitemalign", "set cia=abbrkindmenu,kind,menu"),
    ("completeitemalign", "set cia=menu,kind,abbr"),
    ("completeopt", "set cot=menu,menuone"),
    ("completeopt", "set cot=menu,menu"),
    ("completeopt", "set cot=foo"),
    ("completeopt", "set cot=menu,"),
    ("completeopt", "setlocal cot=popup"),
    ("completeopt", "setlocal cot="),
    ("completeopt", "setlocal cot=popup,bogus"),
    ("wildoptions", "set wop=pum,tagfile"),
    ("wildoptions", "set wop=pum,pum"),
    ("wildoptions", "set wop=bogus"),
    ("wildmode", "set wim=list:longest,full"),
    ("wildmode", "set wim=bogus"),
    // Help, spelling, tags.
    ("helplang", "set hlg=en,de"),
    ("helplang", "set hlg=e"),
    ("helplang", "set hlg=en,"),
    ("helplang", "set hlg=english"),
    ("helplang", "set hlg="),
    ("helplang", "set hlg=en,d"),
    ("spellfile", "setlocal spf=/tmp/en.utf-8.add"),
    ("spellfile", "setlocal spf=foo"),
    ("spellfile", "setlocal spf=a.add,,b.add"),
    ("spellfile", "setlocal spf=a*b.add"),
    ("spelllang", "setlocal spl=en_us,cjk"),
    ("spelllang", "setlocal spl=en/x"),
    ("spelllang", "setlocal spl=en-rare,de_de"),
    ("spelllang", "setlocal spl=cjk"),
    ("spelllang", "setlocal spl=en,,de"),
    ("spelllang", "setlocal spl=a\\ b"),
    ("spelllang", "setlocal spl=/abs/path.utf-8.spl"),
    ("spelloptions", "setlocal spo=camel"),
    ("spelloptions", "setlocal spo=bogus"),
    ("spelloptions", "setglobal spo=noplainbuffer"),
    (
        "spellcapcheck",
        "setlocal spc=[.?!]\\\\_[\\\\])'\\\"\\ \\\\t]\\\\+",
    ),
    ("spellcapcheck", "setlocal spc=\\\\("),
    ("spellsuggest", "set sps=best,10"),
    ("spellsuggest", "set sps=bogus"),
    ("mkspellmem", "set msm=460000,2000,500"),
    ("mkspellmem", "set msm=1"),
    ("tagcase", "set tc=match"),
    ("tagcase", "set tc=x"),
    ("tagcase", "setlocal tc=smart"),
    ("tagcase", "setlocal tc="),
    ("tagcase", "setlocal tc=bad"),
    // Display.
    ("background", "set bg=light"),
    ("background", "set bg=x"),
    ("breakat", "set brk=\\ -"),
    ("breakindentopt", "setlocal briopt=shift:2,min:20,sbr"),
    ("breakindentopt", "setlocal briopt=foo"),
    ("breakindentopt", "setlocal briopt=shift:"),
    ("breakindentopt", "setlocal briopt=min:99999999999999999999"),
    ("breakindentopt", "setglobal briopt=list:-1"),
    ("colorcolumn", "setlocal cc=+1,80"),
    ("colorcolumn", "setlocal cc=x"),
    ("colorcolumn", "setlocal cc=1,,2"),
    ("colorcolumn", "setlocal cc=-2"),
    ("colorcolumn", "setlocal cc=1,"),
    ("colorcolumn", "setglobal cc=+x"),
    ("colorcolumn", "setlocal cc=99999999999999"),
    ("cursorlineopt", "setlocal culopt=number,line"),
    ("cursorlineopt", "setlocal culopt="),
    ("cursorlineopt", "setlocal culopt=both,foo"),
    ("cursorlineopt", "setlocal culopt=screenline,line"),
    ("display", "set dy=lastline,uhex"),
    ("display", "set dy=bogus"),
    ("highlight", "set hl=x"),
    ("inccommand", "set icm=split"),
    ("inccommand", "set icm=bogus"),
    ("keymodel", "set km=startsel,stopsel"),
    ("keymodel", "set km=bogus"),
    ("mousescroll", "set mousescroll=ver:3,hor:2"),
    ("mousescroll", "set mousescroll=ver:x"),
    ("mousescroll", "set mousescroll=ver:1,ver:2"),
    ("mousescroll", "set mousescroll=ver:99999999999"),
    ("mousescroll", "set mousescroll=hor:"),
    ("mousescroll", "set mousescroll=up:1"),
    ("mousescroll", "set mousescroll=hor:4"),
    ("selection", "set sel=exclusive"),
    ("selection", "set sel=x"),
    ("showbreak", "set sbr=>>\\ "),
    ("showbreak", "set sbr=\\\t"),
    ("showbreak", "set sbr=\u{3042}"),
    ("showbreak", "set sbr=\u{e9}\u{301}"),
    ("showbreak", "setlocal sbr=NONE"),
    ("showcmdloc", "set sloc=statusline"),
    ("showcmdloc", "set sloc=x"),
    ("signcolumn", "setlocal scl=auto:1-3"),
    ("signcolumn", "setlocal scl=auto:3-1"),
    ("signcolumn", "setlocal scl=number"),
    ("signcolumn", "setlocal scl=yes:9"),
    ("signcolumn", "setlocal scl=x"),
    ("signcolumn", "setlocal scl="),
    ("signcolumn", "setlocal scl=auto:0-2"),
    ("signcolumn", "setglobal scl=auto:2-9"),
    ("virtualedit", "set ve=all"),
    ("virtualedit", "set ve=x"),
    ("virtualedit", "setlocal ve=block,onemore"),
    ("virtualedit", "setlocal ve="),
    ("virtualedit", "setlocal ve=none,bogus"),
    ("winborder", "set winborder=rounded"),
    ("winborder", "set winborder=x"),
    ("winborder", "set winborder=a,b,c,d,e,f,g,h"),
    ("winborder", "set winborder=a,b"),
    ("pumborder", "set pumborder=single"),
    ("pumborder", "set pumborder=bogus"),
    ("winhighlight", "setlocal winhl=Normal:Foo"),
    ("winhighlight", "setlocal winhl=Normal"),
    ("winhighlight", "setlocal winhl=Normal:Foo,"),
    ("winhighlight", "setlocal winhl=:Foo"),
    // Format strings, with the `%` that is wrong.
    ("statusline", "set stl=%f\\ %m%=%l"),
    ("statusline", "set stl=%{"),
    ("statusline", "set stl=%{%x}"),
    ("statusline", "set stl=%{%}"),
    ("statusline", "set stl=%("),
    ("statusline", "set stl=%)"),
    ("statusline", "set stl=%Q"),
    ("statusline", "set stl=abc%"),
    ("statusline", "set stl=%-10.5f"),
    ("statusline", "set stl=%5*x%*"),
    ("statusline", "set stl=%!bogus("),
    ("statusline", "set stl="),
    ("statusline", "setlocal stl=%{"),
    ("statusline", "setglobal stl="),
    ("statusline", "set stl=%{%x%}%(%)%%%<%="),
    ("tabline", "set tal=%T%X"),
    ("tabline", "set tal=%1T%999X"),
    ("tabline", "set tal=%z"),
    ("winbar", "setlocal wbr=%f"),
    ("winbar", "setlocal wbr=%("),
    ("statuscolumn", "setlocal stc=%l%s"),
    ("statuscolumn", "setlocal stc=%{"),
    ("rulerformat", "set ruf=%15(%l,%c%)"),
    ("rulerformat", "set ruf=%-15(%l%)"),
    ("rulerformat", "set ruf=%15(%l"),
    ("rulerformat", "set ruf=%!x"),
    ("rulerformat", "set ruf=%99999999999(x%)"),
    ("rulerformat", "set ruf=%0(x%)"),
    ("rulerformat", "set ruf=%15x"),
    ("rulerformat", "set ruf=%Q"),
    ("rulerformat", "set ruf=plain"),
    ("titlestring", "set titlestring=%t%Q"),
    ("titlestring", "set titlestring=%{"),
    ("iconstring", "set iconstring=%t"),
    // Session, history and shell specs.
    ("sessionoptions", "set ssop=buffers,curdir"),
    ("sessionoptions", "set ssop=curdir,sesdir"),
    ("sessionoptions", "set ssop=bogus"),
    ("viewoptions", "set vop=folds,cursor"),
    ("viewoptions", "set vop=bogus"),
    ("shada", "set sd=!,'100,<50,s10,h"),
    ("shada", "set sd=x"),
    ("shada", "set sd='"),
    ("shada", "set sd='100x"),
    ("shada", "set sd=<50"),
    ("shada", "set sd="),
    ("shada", "set sd='10,n/tmp/x,y"),
    ("shada", "set sd='10,r/tmp,%5"),
    ("shada", "set sd='10,:"),
    ("shellpipe", "set sp=2>&1\\|\\ tee\\ %s"),
    ("shellpipe", "set sp=%s%s"),
    ("shellpipe", "set sp=x%"),
    ("shellpipe", "set sp=%%%s"),
    ("shellredir", "set srr=>%s\\ 2>&1"),
    ("shellredir", "set srr=%d"),
    ("verbosefile", "set vfile="),
    // Word lists the generic check handles.
    ("switchbuf", "set swb=useopen,usetab"),
    ("switchbuf", "set swb=bogus"),
    ("casemap", "set cmp=internal,keepascii"),
    ("clipboard", "set cb=unnamed"),
    ("clipboard", "set cb=bogus"),
    ("belloff", "set bo=all"),
    ("jumpoptions", "set jop=stack,view"),
    ("jumpoptions", "set jop=bogus"),
    ("redrawdebug", "set rdb=compositor"),
    ("tabclose", "set tcl=left,uselast"),
    ("termpastefilter", "set tpf=BS,HT"),
    ("diffopt", "set dip=filler,context:3"),
    ("diffopt", "set dip=bogus"),
    ("guicursor", "set gcr=n:block"),
    ("guicursor", "set gcr=x"),
    ("messagesopt", "set mopt=hit-enter,history:500"),
    ("messagesopt", "set mopt=bogus"),
    ("ambiwidth", "set ambw=single"),
    ("ambiwidth", "set ambw=x"),
];

/// The options [`CASES`] reach that are global-local, whose local value has
/// to be dropped as well as the global one reset.
const GLOBAL_LOCAL: &[&str] = &[
    "backupcopy",
    "completeopt",
    "eventignore",
    "makeencoding",
    "showbreak",
    "statusline",
    "tagcase",
    "virtualedit",
    "winbar",
    "listchars",
    "fillchars",
];

/// What `&opt` and `&l:opt` say, and the error the last command left.
fn answer(fx: &Probe, name: &str, error: String) -> String {
    format!(
        "{error}|{}|{}",
        fx.eval(&format!("&{name}")),
        fx.eval(&format!("&l:{name}"))
    )
}

/// Back to the default at both scopes.
fn reset(fx: &Probe, name: &str) {
    fx.run(&format!("silent! set {name}&"));
    if GLOBAL_LOCAL.contains(&name) {
        fx.run(&format!("silent! set {name}<"));
    }
}

#[test]
fn every_callback_with_a_good_value_and_a_bad_one() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.home();
    // The spell and keymap callbacks read the runtime; none of these cases
    // needs one to exist.
    fx.run("set nospell");
    let mut got = Vec::new();
    for (name, cmd) in CASES {
        reset(&fx, name);
        let error = fx.error_from(cmd);
        got.push(answer(&fx, name, error));
        reset(&fx, name);
    }
    fx.run("setlocal buftype= modifiable");
    fx.home();
    if let Some(dump) = std::env::var_os("OPTIONSTR_ORACLE_DUMP") {
        let mut text = String::new();
        for answer in &got {
            text.push_str(&format!("    {answer:?},\n"));
        }
        let _ = std::fs::write(std::path::PathBuf::from(dump).join("cases"), text);
    }
    let moved: Vec<String> = CASES
        .iter()
        .zip(&got)
        .zip(ANSWERS)
        .filter(|((_, got), want)| got != *want)
        .map(|(((_, cmd), got), want)| format!("{cmd}: {got:?}, was {want:?}"))
        .collect();
    assert!(
        moved.is_empty(),
        "{} answers moved:\n{}",
        moved.len(),
        moved.join("\n")
    );
    assert_eq!(got.len(), ANSWERS.len());
}

/// A `FileType` autocommand that sets 'filetype' again from inside the set
/// that fired it, and a `Syntax` one that does the same to 'syntax'. (The
/// unit lane's editor is still `starting`, so `OptionSet` does not fire
/// here; `test_autocmd` holds that half.)
#[test]
fn an_autocommand_that_sets_the_option_again() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.home();
    fx.run("let g:fired = []");
    fx.run("augroup optionstr_oracle | autocmd! | augroup END");
    fx.run(
        "autocmd optionstr_oracle FileType * call add(g:fired, expand('<amatch>')) \
         | if expand('<amatch>') ==# 'c' | setlocal filetype=cpp | endif",
    );
    fx.run(
        "autocmd optionstr_oracle Syntax * call add(g:fired, 'syn:' . expand('<amatch>')) \
         | if expand('<amatch>') ==# 'one' | setlocal syntax=two | endif",
    );
    fx.check_error("setlocal filetype=c", "none");
    fx.check("&l:filetype", "'cpp'");
    fx.check_error("setlocal filetype=c", "none");
    fx.check("&l:filetype", "'cpp'");
    fx.check_error("setlocal syntax=one", "none");
    fx.check("&l:syntax", "'two'");
    fx.check("g:fired", "['c', 'cpp', 'c', 'cpp', 'syn:one', 'syn:two']");
    fx.run("autocmd! optionstr_oracle");
    fx.run("augroup! optionstr_oracle");
    fx.run("unlet g:fired");
    fx.run("setlocal filetype= syntax=");
    fx.home();
}

/// A Vim script double-quoted literal of `text`, byte for byte.
fn literal(text: &[u8]) -> String {
    let hex: String = text.iter().map(|b| format!("\\x{b:02x}")).collect();
    format!("\"{hex}\"")
}

/// The line `:list` is shown, which every 'listchars' field draws on: tabs
/// leading and inside, a leading run of spaces, single and multiple inner
/// spaces, a no-break space and trailing blanks.
const LIST_LINE: &str = "\t  \tlead  two   three x\u{a0}y \t ";

/// Everything a window's glyph tables say, and how `:list` draws
/// [`LIST_LINE`] with them.
fn glyphs(fx: &Probe) -> Vec<u8> {
    let win = Win::current();
    let lcs = &win.w_p_lcs_chars;
    let fcs = &win.w_p_fcs_chars;
    let mut out = format!(
        "{:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x}|",
        lcs.eol,
        lcs.ext,
        lcs.prec,
        lcs.nbsp,
        lcs.space,
        lcs.tab1,
        lcs.tab2,
        lcs.tab3,
        lcs.leadtab1,
        lcs.leadtab2,
        lcs.leadtab3,
        lcs.lead,
        lcs.trail,
        lcs.conceal,
    );
    for c in [
        fcs.stl,
        fcs.stlnc,
        fcs.wbr,
        fcs.horiz,
        fcs.horizup,
        fcs.horizdown,
        fcs.vert,
        fcs.vertleft,
        fcs.vertright,
        fcs.verthoriz,
        fcs.fold,
        fcs.foldopen,
        fcs.foldclosed,
        fcs.foldsep,
        fcs.foldinner,
        fcs.diff,
        fcs.msgsep,
        fcs.eob,
        fcs.lastline,
        fcs.trunc,
        fcs.truncrl,
    ] {
        out.push_str(&format!("{c:x} "));
    }
    out.push('|');
    out.push_str(&fx.output("1list"));
    out.into_bytes()
}

/// The 'listchars' and 'fillchars' values the glyph digest runs over, beyond
/// the corpus: every field, each field twice, the escapes, multibyte and
/// double-width characters, and the malformed shapes.
const CHARS_VALUES: &[(&str, &str)] = &[
    (
        "listchars",
        "eol:$,tab:>-,trail:~,extends:>,precedes:<,nbsp:+",
    ),
    ("listchars", "tab:>-<,space:.,lead:_,trail:-"),
    ("listchars", "tab:\\x3e\\u2015,eol:\\U0001f600"),
    ("listchars", "tab:\\x3g-"),
    ("listchars", "multispace:---+,trail:x"),
    ("listchars", "multispace:ab,multispace:xyz,tab:>-"),
    ("listchars", "leadmultispace:\u{b7}\u{2015},multispace:.:"),
    ("listchars", "leadmultispace:,tab:>-"),
    ("listchars", "multispace:"),
    ("listchars", "multispace:\u{3042}"),
    ("listchars", "lead:\u{e9}\u{301},trail:\u{2026}"),
    ("listchars", "leadtab:>-,tab:>-"),
    ("listchars", "leadtab:>-"),
    ("listchars", "leadtab:>-x,tab:|-x"),
    ("listchars", "tab:>"),
    ("listchars", "tab:"),
    ("listchars", "tab:>->-"),
    ("listchars", "eol:ab"),
    ("listchars", "eol:"),
    ("listchars", "eol"),
    ("listchars", "bogus:x"),
    ("listchars", "eol:$,"),
    ("listchars", ",eol:$"),
    ("listchars", "conceal:*,nbsp:\u{a0}"),
    ("listchars", "eol:\u{3042}"),
    ("listchars", "eol:\\x01"),
    ("listchars", ""),
    ("fillchars", "vert:|,fold:-,eob:\\ "),
    ("fillchars", "stl:^,stlnc:=,wbr:-,diff:/,msgsep:_"),
    (
        "fillchars",
        "horiz:\u{2501},horizup:\u{253b},horizdown:\u{2533},vert:\u{2503}",
    ),
    ("fillchars", "vertleft:<,vertright:>,verthoriz:+,lastline:!"),
    (
        "fillchars",
        "foldopen:v,foldclose:>,foldsep:|,foldinner:\\ ,trunc:%,truncrl:#",
    ),
    ("fillchars", "vert:\u{3042}"),
    ("fillchars", "vert:ab"),
    ("fillchars", "fold:"),
    ("fillchars", "eob:\\u00e9"),
    ("fillchars", "eob:x,eob:y"),
    ("fillchars", "bogus:x"),
    ("fillchars", "foldclosed:x"),
    ("fillchars", ""),
];

#[test]
fn listchars_and_fillchars_glyph_tables() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.home();
    fx.run(&format!(
        "call setline(1, {})",
        literal(LIST_LINE.as_bytes())
    ));
    fx.run("setlocal list");
    let mut fixed = Digest::new();
    let mut d = Digest::new();
    let mut rows = String::new();
    // The fixed values at every scope a set can reach, then over the corpus.
    for (name, value) in CHARS_VALUES {
        for how in ["set", "setlocal", "setglobal"] {
            fx.run(&format!(
                "let g:oracle_value = {}",
                literal(value.as_bytes())
            ));
            let error = fx.error_from(&format!(
                "execute '{how} {name}=' . escape(g:oracle_value, ' \\|\"')"
            ));
            let row = answer(&fx, name, error);
            fixed.bytes(row.as_bytes());
            fixed.bytes(&glyphs(&fx));
            rows.push_str(&format!("{how} {name}={value}: {row}\n"));
            reset(&fx, name);
        }
    }
    let texts: Vec<Vec<u8>> = corpus()
        .into_iter()
        .filter(|text| !text.contains(&0) && !text.contains(&b'\n'))
        .collect();
    for text in &texts {
        fx.run(&format!("let g:oracle_text = {}", literal(text)));
        for (name, field) in [
            ("listchars", "multispace:"),
            ("listchars", "leadmultispace:"),
            ("listchars", "tab:"),
            ("listchars", "eol:$,trail:"),
            ("listchars", "lead:"),
            ("fillchars", "vert:"),
            ("fillchars", "eob:"),
        ] {
            fx.run("let v:errmsg = ''");
            fx.run(&format!("silent! let &{name} = '{field}' . g:oracle_text"));
            d.bytes(answer(&fx, name, fx.eval("v:errmsg")).as_bytes());
            d.bytes(&glyphs(&fx));
            reset(&fx, name);
        }
    }
    fx.run("unlet! g:oracle_value g:oracle_text");
    fx.run("setlocal list&");
    fx.home();
    if let Some(dump) = std::env::var_os("OPTIONSTR_ORACLE_DUMP") {
        let _ = std::fs::write(std::path::PathBuf::from(dump).join("chars"), rows);
    }
    assert_eq!(
        [fixed.value(), d.value()],
        GLYPHS,
        "glyph table digests moved"
    );
}

#[test]
fn the_statusline_validator_over_the_corpus() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.home();
    let texts: Vec<Vec<u8>> = corpus()
        .into_iter()
        .filter(|text| !text.contains(&0) && !text.contains(&b'\n'))
        .collect();
    let mut digests = Vec::new();
    for name in ["statusline", "rulerformat", "tabline", "titlestring"] {
        let mut d = Digest::new();
        for text in &texts {
            fx.run(&format!("let g:oracle_text = {}", literal(text)));
            for frame in [
                "g:oracle_text",
                "'%' . g:oracle_text",
                "'%-5.3' . g:oracle_text",
                "'%{' . g:oracle_text",
                "'%{%' . g:oracle_text . '%}'",
                "'%(' . g:oracle_text . '%)'",
                "'%15(' . g:oracle_text",
                "g:oracle_text . '%'",
            ] {
                fx.run("let v:errmsg = ''");
                fx.run(&format!("silent! let &{name} = {frame}"));
                d.bytes(answer(&fx, name, fx.eval("v:errmsg")).as_bytes());
                reset(&fx, name);
            }
        }
        digests.push(d.value());
    }
    fx.run("unlet! g:oracle_text");
    fx.home();
    assert_eq!(digests, STATUSLINE, "statusline validator digests moved");
}

/// Recorded on the pointer-walking callbacks.
const ANSWERS: &[&str] = &[
    "none|'2'|'2'",
    "E474: Invalid argument: bs=3|'indent,eol,start'|'indent,eol,start'",
    "E474: Invalid argument: bs=0|'indent,eol,start'|'indent,eol,start'",
    "none|'indent,eol,nostop'|'indent,eol,nostop'",
    "E474: Invalid argument: bs=indent,foo|'indent,eol,start'|'indent,eol,start'",
    "none|'yes'|''",
    "E474: Invalid argument: bkc=yes,no|'auto'|''",
    "E474: Invalid argument: bkc=breaksymlink|'auto'|''",
    "none|'auto,breakhardlink'|''",
    "none|'no'|'no'",
    "none|'auto'|''",
    "E474: Invalid argument: bkc=bogus|'auto'|''",
    "E474: Invalid argument: bkc=no,yes|'auto'|''",
    "E589: ''backupext'' and ''patchmode'' are equal: pm=.orig|''|''",
    "E589: ''backupext'' and ''patchmode'' are equal: pm=.orig|''|''",
    "E589: ''backupext'' and ''patchmode'' are equal: pm=~|'~'|'~'",
    "none|'wipe'|'wipe'",
    "E474: Invalid argument: bh=wipe,hide|''|''",
    "E474: Invalid argument: bh=x|''|''",
    "none|'nofile'|'nofile'",
    "E474: Invalid argument: bt=terminal|''|''",
    "E474: Invalid argument: bt=bogus|''|''",
    "none|'help'|'help'",
    "none|''|''",
    "none|'b:#,://'|'b:#,://'",
    "E524: Missing colon: com=b|'s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-,fb:•'|'s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-,fb:•'",
    "E525: Zero length string: com=b:|'s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-,fb:•'|'s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-,fb:•'",
    "E525: Zero length string: com=b:,:x|'s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-,fb:•'|'s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-,fb:•'",
    "E539: Illegal character <q>: com=q:x|'s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-,fb:•'|'s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-,fb:•'",
    "E525: Zero length string: com=q|'s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-,fb:•'|'s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-,fb:•'",
    "none|'s1:/*,mb:*,ex:*/'|'s1:/*,mb:*,ex:*/'",
    "none|'-5:x'|'-5:x'",
    "none|':a\\,b,:c'|':a\\,b,:c'",
    "none|':a\\'|':a\\'",
    "E539: Illegal character <,>: com=,:x|'s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-,fb:•'|'s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-,fb:•'",
    "E539: Illegal character <,>: com=:x,,:y|'s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-,fb:•'|'s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-,fb:•'",
    "none|'/*%s*/'|'/*%s*/'",
    "E537: ''commentstring'' must be empty or contain %s: cms=foo|''|''",
    "none|''|''",
    "E537: ''commentstring'' must be empty or contain %s: cms=%|''|''",
    "none|'aA'|'aA'",
    "E539: Illegal character <1>: cpo=a1|'aABceFs_'|'aABceFs_'",
    "E539: Illegal character <,>: cpo+=,|'aABceFs_'|'aABceFs_'",
    "none|'tcq'|'tcq'",
    "E539: Illegal character <%>: fo=tc%|'tcqj'|'tcqj'",
    "E539: Illegal character < >: fo=\\ t|'tcqj'|'tcqj'",
    "none|'nv'|'nv'",
    "E539: Illegal character <x>: cocu=x|''|''",
    "none|'a'|'a'",
    "E539: Illegal character <z>: mouse=nvz|'nvi'|'nvi'",
    "none|'b,s,h,l'|'b,s,h,l'",
    "E539: Illegal character <z>: ww=<,>,z|'b,s'|'b,s'",
    "none|'aI'|'aI'",
    "E539: Illegal character <Z>: shm=aZ|'ltToOCF'|'ltToOCF'",
    "E519: Option not supported: enc=latin1|'utf-8'|'utf-8'",
    "none|'utf-8'|'utf-8'",
    "none|'utf-8'|'utf-8'",
    "none|'latin1'|'latin1'",
    "none|'utf-8'|'utf-8'",
    "E474: Invalid argument: fenc=a,b|''|''",
    "none|'ucs-2le'|'ucs-2le'",
    "E21: Cannot make changes, ''modifiable'' is off: fenc=latin1|''|''",
    "none|''|''",
    "none|'char'|''",
    "none|'cp1252'|''",
    "none|'all'|'all'",
    "none|'BufRead,WinEnter'|'BufRead,WinEnter'",
    "E474: Invalid argument: ei=BufRead,bogus|''|''",
    "none|'-BufRead,all'|'-BufRead,all'",
    "none|'BufRead,'|'BufRead,'",
    "none|'WinEnter'|'WinEnter'",
    "none|'all'|'all'",
    "none|'BufRead'|'BufRead'",
    "E474: Invalid argument: eiw=bogus|''|''",
    "none|'dos'|'dos'",
    "E474: Invalid argument: ff=foo|'unix'|'unix'",
    "E474: Invalid argument: ff=|'unix'|'unix'",
    "none|'unix'|'unix'",
    "E21: Cannot make changes, ''modifiable'' is off: ff=dos|'unix'|'unix'",
    "none|'unix'|'unix'",
    "none|'unix,dos,mac'|'unix,dos,mac'",
    "none|'unix,unix'|'unix,unix'",
    "none|'unix,'|'unix,'",
    "none|'c'|'c'",
    "E474: Invalid argument: ft=a/b|''|''",
    "none|'c.doxygen'|'c.doxygen'",
    "E474: Invalid argument: ft=a\\ b|''|''",
    "none|'foo-bar_1'|'foo-bar_1'",
    "E474: Invalid argument: syn=a*|''|''",
    "E474: Invalid argument: kmp=a/b|''|''",
    "E544: Keymap file not found: kmp=no_such_keymap|''|''",
    "none|'<<,>>'|'<<,>>'",
    "E536: Comma required: fmr=ab|'{{{,}}}'|'{{{,}}}'",
    "E474: Invalid argument: fmr=,b|'{{{,}}}'|'{{{,}}}'",
    "E474: Invalid argument: fmr=a,|'{{{,}}}'|'{{{,}}}'",
    "none|'a,b,c'|'a,b,c'",
    "none|'indent'|'indent'",
    "E474: Invalid argument: fdm=x|'manual'|'manual'",
    "none|'diff'|'diff'",
    "none|'#;'|'#;'",
    "none|'1'|'1'",
    "E81: Using <SID> not in a script context|'s:Nope()'|'s:Nope()'",
    "none|'all'|'all'",
    "E474: Invalid argument: fdo=bogus|'block,hor,mark,percent,quickfix,search,tag,undo'|'block,hor,mark,percent,quickfix,search,tag,undo'",
    "none|'@,48-57,_'|'@,48-57,_'",
    "E474: Invalid argument: isk=300|'@,48-57,_,192-255'|'@,48-57,_,192-255'",
    "E474: Invalid argument: isk=a-|'@,48-57,_,192-255'|'@,48-57,_,192-255'",
    "none|'@,48-57,_,192-255'|'@,48-57,_,192-255'",
    "E474: Invalid argument: isk=1-|'@,48-57,_,192-255'|'@,48-57,_,192-255'",
    "none|'@,48-57,/'|'@,48-57,/'",
    "E474: Invalid argument: isf=x-|'@,48-57,/,.,-,_,+,,,#,$,%,~,='|'@,48-57,/,.,-,_,+,,,#,$,%,~,='",
    "none|'@,161-255'|'@,161-255'",
    "none|'@,48-57,_,192-255'|'@,48-57,_,192-255'",
    "none|'expr:1'|'expr:1'",
    "none|'expr:0'|'expr:0'",
    "E474: Invalid argument: lop=expr:2|''|''",
    "none|'(:),<:>'|'(:),<:>'",
    "E474: Invalid argument: mps=(|'(:),{:},[:]'|'(:),{:},[:]'",
    "E474: Invalid argument: mps=(;)|'(:),{:},[:]'|'(:),{:},[:]'",
    "E474: Invalid argument: mps=(:|'(:),{:},[:]'|'(:),{:},[:]'",
    "none|'é:è,«:»'|'é:è,«:»'",
    "none|'(:),'|'(:),'",
    "E474: Invalid argument: mps=(:)x|'(:),{:},[:]'|'(:),{:},[:]'",
    "none|'a\u{301}:b'|'a\u{301}:b'",
    "none|'4,8'|'4,8'",
    "E474: Invalid argument: vts=4,,8|''|''",
    "none|'0'|'0'",
    "none|''|''",
    "E474: Invalid argument: vts=,4|''|''",
    "E474: Invalid argument: vts=4,0|''|''",
    "E474: Invalid argument: vts=99999999999|''|''",
    "E474: Invalid argument: vts=x|'8'|'8'",
    "none|'2,4'|'2,4'",
    "E474: Invalid argument: vsts=-1|''|''",
    "E474: Invalid argument: vsts=a|''|''",
    "none|'.,w,b,u,t,i'|'.,w,b,u,t,i'",
    "E539: Illegal character <x>: cpt=x|'.,w,b,u,t'|'.,w,b,u,t'",
    "none|'.^5,w^10'|'.^5,w^10'",
    "E535: Illegal character after <^>: cpt=.^|'.,w,b,u,t'|'.,w,b,u,t'",
    "E535: Illegal character after <^>: cpt=.^x|'.,w,b,u,t'|'.,w,b,u,t'",
    "E535: Illegal character after <.>: cpt=.x|'.,w,b,u,t'|'.,w,b,u,t'",
    "none|'kfoo,sbar'|'kfoo,sbar'",
    "none|'k\\,foo'|'k\\,foo'",
    "none|'., ,w'|'., ,w'",
    "none|''|''",
    "none|'o'|'o'",
    "none|'FNoSuchFunction'|'FNoSuchFunction'",
    "none|'kind,abbr,menu'|'kind,abbr,menu'",
    "E474: Invalid argument: cia=abbr,abbr,menu|'abbr,kind,menu'|'abbr,kind,menu'",
    "E474: Invalid argument: cia=abbr,kind|'abbr,kind,menu'|'abbr,kind,menu'",
    "E474: Invalid argument: cia=abbr,kind,menu,abbr|'abbr,kind,menu'|'abbr,kind,menu'",
    "E474: Invalid argument: cia=abbrkindmenu,kind,menu|'abbr,kind,menu'|'abbr,kind,menu'",
    "none|'menu,kind,abbr'|'menu,kind,abbr'",
    "none|'menu,menuone'|''",
    "none|'menu,menu'|''",
    "E474: Invalid argument: cot=foo|'menu,popup'|''",
    "none|'menu,'|''",
    "none|'popup'|'popup'",
    "none|'menu,popup'|''",
    "E474: Invalid argument: cot=popup,bogus|'menu,popup'|''",
    "none|'pum,tagfile'|'pum,tagfile'",
    "none|'pum,pum'|'pum,pum'",
    "E474: Invalid argument: wop=bogus|'pum,tagfile'|'pum,tagfile'",
    "none|'list:longest,full'|'list:longest,full'",
    "E474: Invalid argument: wim=bogus|'full'|'full'",
    "none|'en,de'|'en,de'",
    "E474: Invalid argument: hlg=e|''|''",
    "E474: Invalid argument: hlg=en,|''|''",
    "E474: Invalid argument: hlg=english|''|''",
    "none|''|''",
    "E474: Invalid argument: hlg=en,d|''|''",
    "none|'/tmp/en.utf-8.add'|'/tmp/en.utf-8.add'",
    "E474: Invalid argument: spf=foo|''|''",
    "E474: Invalid argument: spf=a.add,,b.add|''|''",
    "E474: Invalid argument: spf=a*b.add|''|''",
    "none|'en_us,cjk'|'en_us,cjk'",
    "E474: Invalid argument: spl=en/x|'en'|'en'",
    "none|'en-rare,de_de'|'en-rare,de_de'",
    "none|'cjk'|'cjk'",
    "none|'en,,de'|'en,,de'",
    "E474: Invalid argument: spl=a\\ b|'en'|'en'",
    "E474: Invalid argument: spl=/abs/path.utf-8.spl|'en'|'en'",
    "none|'camel'|'camel'",
    "E474: Invalid argument: spo=bogus|''|''",
    "none|''|''",
    "none|'[.?!]\\_[\\])''\" \\t]\\+'|'[.?!]\\_[\\])''\" \\t]\\+'",
    "E474: Invalid argument: spc=\\\\(|'[.?!]\\_[\\])''\"\\t ]\\+'|'[.?!]\\_[\\])''\"\\t ]\\+'",
    "none|'best,10'|'best,10'",
    "E474: Invalid argument: sps=bogus|'best'|'best'",
    "none|'460000,2000,500'|'460000,2000,500'",
    "E474: Invalid argument: msm=1|'460000,2000,500'|'460000,2000,500'",
    "none|'match'|''",
    "E474: Invalid argument: tc=x|'followic'|''",
    "none|'smart'|'smart'",
    "none|'followic'|''",
    "E474: Invalid argument: tc=bad|'followic'|''",
    "none|'light'|'light'",
    "E474: Invalid argument: bg=x|'dark'|'dark'",
    "none|' -'|' -'",
    "none|'shift:2,min:20,sbr'|'shift:2,min:20,sbr'",
    "E474: Invalid argument: briopt=foo|''|''",
    "E474: Invalid argument: briopt=shift:|''|''",
    "none|'min:99999999999999999999'|'min:99999999999999999999'",
    "none|''|''",
    "none|'+1,80'|'+1,80'",
    "E474: Invalid argument: cc=x|''|''",
    "E474: Invalid argument: cc=1,,2|''|''",
    "none|'-2'|'-2'",
    "E474: Invalid argument: cc=1,|''|''",
    "E474: Invalid argument: cc=+x|''|''",
    "none|'99999999999999'|'99999999999999'",
    "none|'number,line'|'number,line'",
    "E474: Invalid argument: culopt=|'both'|'both'",
    "E474: Invalid argument: culopt=both,foo|'both'|'both'",
    "E474: Invalid argument: culopt=screenline,line|'both'|'both'",
    "none|'lastline,uhex'|'lastline,uhex'",
    "E474: Invalid argument: dy=bogus|'lastline'|'lastline'",
    "E519: Option not supported: hl=x|'8:SpecialKey,~:EndOfBuffer,z:TermCursor,@:NonText,d:Directory,e:ErrorMsg,i:IncSearch,l:Search,y:CurSearch,m:MoreMsg,M:ModeMsg,n:LineNr,a:LineNrAbove,b:LineNrBelow,N:CursorLineNr,G:CursorLineSign,O:CursorLineFold,r:Question,s:StatusLine,S:StatusLineNC,c:VertSplit,t:Title,v:Visual,V:VisualNOS,w:WarningMsg,W:WildMenu,f:Folded,F:FoldColumn,A:DiffAdd,C:DiffChange,D:DiffDelete,T:DiffText,E:DiffTextAdd,>:SignColumn,-:Conceal,B:SpellBad,P:SpellCap,R:SpellRare,L:SpellLocal,+:Pmenu,=:PmenuSel,k:PmenuMatch,<:PmenuMatchSel,[:PmenuKind,]:PmenuKindSel,{:PmenuExtra,}:PmenuExtraSel,x:PmenuSbar,X:PmenuThumb,*:TabLine,#:TabLineSel,_:TabLineFill,!:CursorColumn,.:CursorLine,o:ColorColumn,q:QuickFixLine,z:StatusLineTerm,Z:StatusLineTermNC,g:MsgArea,h:ComplMatchIns,0:Whitespace,I:PreInsert'|'8:SpecialKey,~:EndOfBuffer,z:TermCursor,@:NonText,d:Directory,e:ErrorMsg,i:IncSearch,l:Search,y:CurSearch,m:MoreMsg,M:ModeMsg,n:LineNr,a:LineNrAbove,b:LineNrBelow,N:CursorLineNr,G:CursorLineSign,O:CursorLineFold,r:Question,s:StatusLine,S:StatusLineNC,c:VertSplit,t:Title,v:Visual,V:VisualNOS,w:WarningMsg,W:WildMenu,f:Folded,F:FoldColumn,A:DiffAdd,C:DiffChange,D:DiffDelete,T:DiffText,E:DiffTextAdd,>:SignColumn,-:Conceal,B:SpellBad,P:SpellCap,R:SpellRare,L:SpellLocal,+:Pmenu,=:PmenuSel,k:PmenuMatch,<:PmenuMatchSel,[:PmenuKind,]:PmenuKindSel,{:PmenuExtra,}:PmenuExtraSel,x:PmenuSbar,X:PmenuThumb,*:TabLine,#:TabLineSel,_:TabLineFill,!:CursorColumn,.:CursorLine,o:ColorColumn,q:QuickFixLine,z:StatusLineTerm,Z:StatusLineTermNC,g:MsgArea,h:ComplMatchIns,0:Whitespace,I:PreInsert'",
    "none|'split'|'split'",
    "E474: Invalid argument: icm=bogus|'nosplit'|'nosplit'",
    "none|'startsel,stopsel'|'startsel,stopsel'",
    "E474: Invalid argument: km=bogus|''|''",
    "none|'ver:3,hor:2'|'ver:3,hor:2'",
    "E5080: Digit expected: mousescroll=ver:x|'ver:3,hor:6'|'ver:3,hor:6'",
    "E474: Invalid argument: mousescroll=ver:1,ver:2|'ver:3,hor:6'|'ver:3,hor:6'",
    "E474: Invalid argument: mousescroll=ver:99999999999|'ver:3,hor:6'|'ver:3,hor:6'",
    "E474: Invalid argument: mousescroll=hor:|'ver:3,hor:6'|'ver:3,hor:6'",
    "E474: Invalid argument: mousescroll=up:1|'ver:3,hor:6'|'ver:3,hor:6'",
    "none|'hor:4'|'hor:4'",
    "none|'exclusive'|'exclusive'",
    "E474: Invalid argument: sel=x|'inclusive'|'inclusive'",
    "none|'>> '|''",
    "E595: ''showbreak'' contains unprintable or wide character: sbr=\\^I|''|''",
    "E595: ''showbreak'' contains unprintable or wide character: sbr=あ|''|''",
    "none|'é\u{301}'|''",
    "none|'NONE'|'NONE'",
    "none|'statusline'|'statusline'",
    "E474: Invalid argument: sloc=x|'last'|'last'",
    "none|'auto:1-3'|'auto:1-3'",
    "E474: Invalid argument: scl=auto:3-1|'auto'|'auto'",
    "none|'number'|'number'",
    "none|'yes:9'|'yes:9'",
    "E474: Invalid argument: scl=x|'auto'|'auto'",
    "E474: Invalid argument: scl=|'auto'|'auto'",
    "E474: Invalid argument: scl=auto:0-2|'auto'|'auto'",
    "none|'auto'|'auto'",
    "none|'all'|''",
    "E474: Invalid argument: ve=x|''|''",
    "none|'block,onemore'|'block,onemore'",
    "none|''|''",
    "E474: Invalid argument: ve=none,bogus|''|''",
    "none|'rounded'|'rounded'",
    "E474: Invalid argument: winborder=x|''|''",
    "none|'a,b,c,d,e,f,g,h'|'a,b,c,d,e,f,g,h'",
    "E474: Invalid argument: winborder=a,b|''|''",
    "none|'single'|'single'",
    "E474: Invalid argument: pumborder=bogus|''|''",
    "none|'Normal:Foo'|'Normal:Foo'",
    "E474: Invalid argument: winhl=Normal|''|''",
    "none|'Normal:Foo,'|'Normal:Foo,'",
    "E474: Invalid argument: winhl=:Foo|''|''",
    "none|'%f %m%=%l'|''",
    "E540: Unclosed expression sequence: stl=%{|'%<%f %h%w%m%r %{% v:lua.require(''vim._core.util'').term_exitcode() %}%=%{% luaeval(''(package.loaded[''''vim.ui''''] and vim.api.nvim_get_current_win() == tonumber(vim.g.actual_curwin or -1) and vim.ui.progress_status()) or '''''''' '')%}%{% &showcmdloc == ''statusline'' ? ''%-10.S '' : '''' %}%{% exists(''b:keymap_name'') ? ''<''..b:keymap_name..''> '' : '''' %}%{% &busy > 0 ? ''◐ '' : '''' %}%{% luaeval(''(package.loaded[''''vim.diagnostic''''] and next(vim.diagnostic.count()) and vim.diagnostic.status() .. '''' '''') or '''''''' '') %}%{% &ruler ? ( &rulerformat == '''' ? ''%-14.(%l,%c%V%) %P'' : &rulerformat ) : '''' %}'|''",
    "E540: Unclosed expression sequence: stl=%{%x}|'%<%f %h%w%m%r %{% v:lua.require(''vim._core.util'').term_exitcode() %}%=%{% luaeval(''(package.loaded[''''vim.ui''''] and vim.api.nvim_get_current_win() == tonumber(vim.g.actual_curwin or -1) and vim.ui.progress_status()) or '''''''' '')%}%{% &showcmdloc == ''statusline'' ? ''%-10.S '' : '''' %}%{% exists(''b:keymap_name'') ? ''<''..b:keymap_name..''> '' : '''' %}%{% &busy > 0 ? ''◐ '' : '''' %}%{% luaeval(''(package.loaded[''''vim.diagnostic''''] and next(vim.diagnostic.count()) and vim.diagnostic.status() .. '''' '''') or '''''''' '') %}%{% &ruler ? ( &rulerformat == '''' ? ''%-14.(%l,%c%V%) %P'' : &rulerformat ) : '''' %}'|''",
    "E539: Illegal character <}>: stl=%{%}|'%<%f %h%w%m%r %{% v:lua.require(''vim._core.util'').term_exitcode() %}%=%{% luaeval(''(package.loaded[''''vim.ui''''] and vim.api.nvim_get_current_win() == tonumber(vim.g.actual_curwin or -1) and vim.ui.progress_status()) or '''''''' '')%}%{% &showcmdloc == ''statusline'' ? ''%-10.S '' : '''' %}%{% exists(''b:keymap_name'') ? ''<''..b:keymap_name..''> '' : '''' %}%{% &busy > 0 ? ''◐ '' : '''' %}%{% luaeval(''(package.loaded[''''vim.diagnostic''''] and next(vim.diagnostic.count()) and vim.diagnostic.status() .. '''' '''') or '''''''' '') %}%{% &ruler ? ( &rulerformat == '''' ? ''%-14.(%l,%c%V%) %P'' : &rulerformat ) : '''' %}'|''",
    "E542: Unbalanced groups: stl=%(|'%<%f %h%w%m%r %{% v:lua.require(''vim._core.util'').term_exitcode() %}%=%{% luaeval(''(package.loaded[''''vim.ui''''] and vim.api.nvim_get_current_win() == tonumber(vim.g.actual_curwin or -1) and vim.ui.progress_status()) or '''''''' '')%}%{% &showcmdloc == ''statusline'' ? ''%-10.S '' : '''' %}%{% exists(''b:keymap_name'') ? ''<''..b:keymap_name..''> '' : '''' %}%{% &busy > 0 ? ''◐ '' : '''' %}%{% luaeval(''(package.loaded[''''vim.diagnostic''''] and next(vim.diagnostic.count()) and vim.diagnostic.status() .. '''' '''') or '''''''' '') %}%{% &ruler ? ( &rulerformat == '''' ? ''%-14.(%l,%c%V%) %P'' : &rulerformat ) : '''' %}'|''",
    "E542: Unbalanced groups: stl=%)|'%<%f %h%w%m%r %{% v:lua.require(''vim._core.util'').term_exitcode() %}%=%{% luaeval(''(package.loaded[''''vim.ui''''] and vim.api.nvim_get_current_win() == tonumber(vim.g.actual_curwin or -1) and vim.ui.progress_status()) or '''''''' '')%}%{% &showcmdloc == ''statusline'' ? ''%-10.S '' : '''' %}%{% exists(''b:keymap_name'') ? ''<''..b:keymap_name..''> '' : '''' %}%{% &busy > 0 ? ''◐ '' : '''' %}%{% luaeval(''(package.loaded[''''vim.diagnostic''''] and next(vim.diagnostic.count()) and vim.diagnostic.status() .. '''' '''') or '''''''' '') %}%{% &ruler ? ( &rulerformat == '''' ? ''%-14.(%l,%c%V%) %P'' : &rulerformat ) : '''' %}'|''",
    "E539: Illegal character <Q>: stl=%Q|'%<%f %h%w%m%r %{% v:lua.require(''vim._core.util'').term_exitcode() %}%=%{% luaeval(''(package.loaded[''''vim.ui''''] and vim.api.nvim_get_current_win() == tonumber(vim.g.actual_curwin or -1) and vim.ui.progress_status()) or '''''''' '')%}%{% &showcmdloc == ''statusline'' ? ''%-10.S '' : '''' %}%{% exists(''b:keymap_name'') ? ''<''..b:keymap_name..''> '' : '''' %}%{% &busy > 0 ? ''◐ '' : '''' %}%{% luaeval(''(package.loaded[''''vim.diagnostic''''] and next(vim.diagnostic.count()) and vim.diagnostic.status() .. '''' '''') or '''''''' '') %}%{% &ruler ? ( &rulerformat == '''' ? ''%-14.(%l,%c%V%) %P'' : &rulerformat ) : '''' %}'|''",
    "E539: Illegal character <^@>: stl=abc%|'%<%f %h%w%m%r %{% v:lua.require(''vim._core.util'').term_exitcode() %}%=%{% luaeval(''(package.loaded[''''vim.ui''''] and vim.api.nvim_get_current_win() == tonumber(vim.g.actual_curwin or -1) and vim.ui.progress_status()) or '''''''' '')%}%{% &showcmdloc == ''statusline'' ? ''%-10.S '' : '''' %}%{% exists(''b:keymap_name'') ? ''<''..b:keymap_name..''> '' : '''' %}%{% &busy > 0 ? ''◐ '' : '''' %}%{% luaeval(''(package.loaded[''''vim.diagnostic''''] and next(vim.diagnostic.count()) and vim.diagnostic.status() .. '''' '''') or '''''''' '') %}%{% &ruler ? ( &rulerformat == '''' ? ''%-14.(%l,%c%V%) %P'' : &rulerformat ) : '''' %}'|''",
    "none|'%-10.5f'|''",
    "none|'%5*x%*'|''",
    "none|'%!bogus('|''",
    "none|'%<%f %h%w%m%r %{% v:lua.require(''vim._core.util'').term_exitcode() %}%=%{% luaeval(''(package.loaded[''''vim.ui''''] and vim.api.nvim_get_current_win() == tonumber(vim.g.actual_curwin or -1) and vim.ui.progress_status()) or '''''''' '')%}%{% &showcmdloc == ''statusline'' ? ''%-10.S '' : '''' %}%{% exists(''b:keymap_name'') ? ''<''..b:keymap_name..''> '' : '''' %}%{% &busy > 0 ? ''◐ '' : '''' %}%{% luaeval(''(package.loaded[''''vim.diagnostic''''] and next(vim.diagnostic.count()) and vim.diagnostic.status() .. '''' '''') or '''''''' '') %}%{% &ruler ? ( &rulerformat == '''' ? ''%-14.(%l,%c%V%) %P'' : &rulerformat ) : '''' %}'|''",
    "E540: Unclosed expression sequence: stl=%{|'%<%f %h%w%m%r %{% v:lua.require(''vim._core.util'').term_exitcode() %}%=%{% luaeval(''(package.loaded[''''vim.ui''''] and vim.api.nvim_get_current_win() == tonumber(vim.g.actual_curwin or -1) and vim.ui.progress_status()) or '''''''' '')%}%{% &showcmdloc == ''statusline'' ? ''%-10.S '' : '''' %}%{% exists(''b:keymap_name'') ? ''<''..b:keymap_name..''> '' : '''' %}%{% &busy > 0 ? ''◐ '' : '''' %}%{% luaeval(''(package.loaded[''''vim.diagnostic''''] and next(vim.diagnostic.count()) and vim.diagnostic.status() .. '''' '''') or '''''''' '') %}%{% &ruler ? ( &rulerformat == '''' ? ''%-14.(%l,%c%V%) %P'' : &rulerformat ) : '''' %}'|''",
    "none|'%<%f %h%w%m%r %{% v:lua.require(''vim._core.util'').term_exitcode() %}%=%{% luaeval(''(package.loaded[''''vim.ui''''] and vim.api.nvim_get_current_win() == tonumber(vim.g.actual_curwin or -1) and vim.ui.progress_status()) or '''''''' '')%}%{% &showcmdloc == ''statusline'' ? ''%-10.S '' : '''' %}%{% exists(''b:keymap_name'') ? ''<''..b:keymap_name..''> '' : '''' %}%{% &busy > 0 ? ''◐ '' : '''' %}%{% luaeval(''(package.loaded[''''vim.diagnostic''''] and next(vim.diagnostic.count()) and vim.diagnostic.status() .. '''' '''') or '''''''' '') %}%{% &ruler ? ( &rulerformat == '''' ? ''%-14.(%l,%c%V%) %P'' : &rulerformat ) : '''' %}'|''",
    "none|'%{%x%}%(%)%%%<%='|''",
    "none|'%T%X'|'%T%X'",
    "none|'%1T%999X'|'%1T%999X'",
    "E539: Illegal character <z>: tal=%z|''|''",
    "E36: Not enough room|'%f'|'%f'",
    "E542: Unbalanced groups: wbr=%(|''|''",
    "none|'%l%s'|'%l%s'",
    "E540: Unclosed expression sequence: stc=%{|''|''",
    "none|'%15(%l,%c%)'|'%15(%l,%c%)'",
    "none|'%-15(%l%)'|'%-15(%l%)'",
    "E542: Unbalanced groups: ruf=%15(%l|''|''",
    "none|'%!x'|'%!x'",
    "none|'%99999999999(x%)'|'%99999999999(x%)'",
    "none|'%0(x%)'|'%0(x%)'",
    "E539: Illegal character <x>: ruf=%15x|''|''",
    "E539: Illegal character <Q>: ruf=%Q|''|''",
    "none|'plain'|'plain'",
    "none|'%t%Q'|'%t%Q'",
    "none|'%{'|'%{'",
    "none|'%t'|'%t'",
    "none|'buffers,curdir'|'buffers,curdir'",
    "E474: Invalid argument: ssop=curdir,sesdir|'blank,buffers,curdir,folds,help,tabpages,winsize,terminal'|'blank,buffers,curdir,folds,help,tabpages,winsize,terminal'",
    "E474: Invalid argument: ssop=bogus|'blank,buffers,curdir,folds,help,tabpages,winsize,terminal'|'blank,buffers,curdir,folds,help,tabpages,winsize,terminal'",
    "none|'folds,cursor'|'folds,cursor'",
    "E474: Invalid argument: vop=bogus|'folds,cursor,curdir'|'folds,cursor,curdir'",
    "none|'!,''100,<50,s10,h'|'!,''100,<50,s10,h'",
    "E539: Illegal character <x>: sd=x|'!,''100,<50,s10,h,r/tmp/,r/private/'|'!,''100,<50,s10,h,r/tmp/,r/private/'",
    "E526: Missing number after <''>: sd=|'!,''100,<50,s10,h,r/tmp/,r/private/'|'!,''100,<50,s10,h,r/tmp/,r/private/'",
    "E527: Missing comma: sd=''100x|'!,''100,<50,s10,h,r/tmp/,r/private/'|'!,''100,<50,s10,h,r/tmp/,r/private/'",
    "E528: Must specify a '' value: sd=<50|'!,''100,<50,s10,h,r/tmp/,r/private/'|'!,''100,<50,s10,h,r/tmp/,r/private/'",
    "none|''|''",
    "none|'''10,n/tmp/x,y'|'''10,n/tmp/x,y'",
    "none|'''10,r/tmp,%5'|'''10,r/tmp,%5'",
    "E526: Missing number after <:>: sd=''10,:|'!,''100,<50,s10,h,r/tmp/,r/private/'|'!,''100,<50,s10,h,r/tmp/,r/private/'",
    "none|'2>&1| tee %s'|'2>&1| tee %s'",
    "E1577: Invalid format string, only one \"%s\" is allowed: sp=%s%s|'| tee'|'| tee'",
    "E1577: Invalid format string, only one \"%s\" is allowed: sp=x%|'| tee'|'| tee'",
    "none|'%%%s'|'%%%s'",
    "none|'>%s 2>&1'|'>%s 2>&1'",
    "E1577: Invalid format string, only one \"%s\" is allowed: srr=%d|'>'|'>'",
    "none|''|''",
    "none|'useopen,usetab'|'useopen,usetab'",
    "E474: Invalid argument: swb=bogus|'uselast'|'uselast'",
    "none|'internal,keepascii'|'internal,keepascii'",
    "none|'unnamed'|'unnamed'",
    "E474: Invalid argument: cb=bogus|''|''",
    "none|'all'|'all'",
    "none|'stack,view'|'stack,view'",
    "E474: Invalid argument: jop=bogus|'clean'|'clean'",
    "none|'compositor'|'compositor'",
    "none|'left,uselast'|'left,uselast'",
    "none|'BS,HT'|'BS,HT'",
    "none|'filler,context:3'|'filler,context:3'",
    "E474: Invalid argument: dip=bogus|'internal,filler,closeoff,indent-heuristic,inline:char,linematch:40'|'internal,filler,closeoff,indent-heuristic,inline:char,linematch:40'",
    "none|'n:block'|'n:block'",
    "E545: Missing colon: gcr=x|'n-v-c-sm:block,i-ci-ve:ver25,r-cr-o:hor20,t:block-blinkon500-blinkoff500-TermCursor'|'n-v-c-sm:block,i-ci-ve:ver25,r-cr-o:hor20,t:block-blinkon500-blinkoff500-TermCursor'",
    "none|'hit-enter,history:500'|'hit-enter,history:500'",
    "E474: Invalid argument: mopt=bogus|'hit-enter,history:500,progress:c'|'hit-enter,history:500,progress:c'",
    "none|'single'|'single'",
    "E474: Invalid argument: ambw=x|'single'|'single'",
];

/// Recorded on the pointer-walking callbacks: the fixed values, the corpus.
const GLYPHS: [u64; 2] = [14_580_284_359_558_147_153, 9_683_694_814_199_680_794];

/// Recorded on the pointer-walking callbacks.
const STATUSLINE: [u64; 4] = [
    5_836_775_544_737_168_705,
    17_838_727_023_538_172_234,
    17_838_727_023_538_172_234,
    17_729_494_920_692_445_697,
];
