//! What `:syntax`, `:highlight` and the group-name table say when they
//! refuse: every message each refusal puts out, in order, so a message that
//! doubles, drops or moves shows as a moved row.
//!
//! Written against the behaviour before the parsers below the commands stop
//! printing at the point of refusal; a row that moves is a behaviour change.
//! Each case works in a buffer of its own and clears its syntax afterwards.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// A buffer of the case's own, with no syntax items.
fn buffer(fx: &Probe) {
    fx.home();
    fx.run("new | only!");
    fx.run("syntax clear");
}

fn restore(fx: &Probe) {
    fx.run("syntax clear");
    fx.run("set nomodified");
    fx.home();
    fx.settle();
}

#[test]
fn syntax_refusals_report_once() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    buffer(&fx);
    for (cmd, want) in [
        ("syntax case foo", "E390: Illegal argument: foo"),
        ("syntax spell foo", "E390: Illegal argument: foo"),
        ("syntax conceal foo", "E390: Illegal argument: foo"),
        ("syntax foldlevel foo", "E390: Illegal argument: foo"),
        (
            "syntax nosuchcmd",
            "E410: Invalid :syntax subcommand: nosuchcmd",
        ),
        ("syntax sync foo", "E404: Illegal arguments: foo"),
        ("syntax sync minlines", "E404: Illegal arguments: minlines"),
        (
            "syntax sync minlines=x",
            "E404: Illegal arguments: minlines=x",
        ),
        ("syntax sync linecont", "E404: Illegal arguments: linecont"),
        (
            "syntax sync linecont /a",
            "E404: Illegal arguments: linecont /a",
        ),
        (r"syntax sync linecont /\(/", "E54: Unmatched \\("),
        ("syntax sync linecont /a/", "none"),
        (
            "syntax sync linecont /b/",
            "E403: syntax sync: line continuations pattern specified twice",
        ),
        ("syntax include @", "E397: Filename required"),
        (
            "syntax include unit_no_such_syntax_file.vim",
            "E484: Can't open file unit_no_such_syntax_file.vim",
        ),
        (
            "syntax include @Incl /unit/no/such/file.vim",
            "E484: Can't open file /unit/no/such/file.vim",
        ),
        ("syntax match", "E475: Invalid argument: "),
        ("syntax match Foo", "E475: Invalid argument: Foo"),
        ("syntax match Foo /a", "E475: Invalid argument: Foo /a"),
        (
            "syntax match Foo /a/x",
            "E402: Garbage after pattern: /a/x\nE475: Invalid argument: Foo /a/x",
        ),
        (
            "syntax match Foo /a/ms=",
            "E402: Garbage after pattern: /a/ms=\nE475: Invalid argument: Foo /a/ms=",
        ),
        (
            r"syntax match Foo /\(/",
            "E54: Unmatched \\(\nE475: Invalid argument: Foo /\\(/",
        ),
        (
            "syntax match Foo /a/ junk",
            "E475: Invalid argument: Foo /a/ junk",
        ),
        (
            "syntax match Foo /a/ contains",
            "E475: Invalid argument: Foo /a/ contains",
        ),
        (
            "syntax match Foo /a/ contains=",
            "E406: Empty argument: contains=\nE475: Invalid argument: Foo /a/ contains=",
        ),
        (
            "syntax match Foo /a/ nextgroup=ALL",
            "E407: ALL not allowed here\nE475: Invalid argument: Foo /a/ nextgroup=ALL",
        ),
        (
            "syntax match Foo /a/ contains=Bar,ALL",
            "E408: ALL must be first in contains list\nE475: Invalid argument: Foo /a/ contains=Bar,ALL",
        ),
        (
            "syntax match Foo /a/ contains=Unit.*Nomatch",
            "E409: Unknown group name: Unit.*Nomatch\nE475: Invalid argument: Foo /a/ contains=Unit.*Nomatch",
        ),
        (
            r"syntax match Foo /a/ contains=\(",
            "E54: Unmatched \\(\nE475: Invalid argument: Foo /a/ contains=\\(",
        ),
        (
            "syntax match Foo /a/ contains=Bar!",
            "E5248: Invalid character in group name\nE409: Unknown group name: Bar!\nE475: Invalid argument: Foo /a/ contains=Bar!",
        ),
        ("syntax match Foo /a/ contains=@Bar!", "none"),
        (
            "syntax match Foo /a/ cchar=",
            "E844: Invalid cchar value\nE475: Invalid argument: Foo /a/ cchar=",
        ),
        (
            "syntax match Foo /a/ cchar=\t",
            "E844: Invalid cchar value\nE475: Invalid argument: Foo /a/ cchar=\t",
        ),
        (
            "syntax match Foo /a/ grouphere Bar",
            "E393: group[t]here not accepted here\nE475: Invalid argument: Foo /a/ grouphere Bar",
        ),
        (
            "syntax sync match Foo grouphere UnitNoRegion /a/",
            "E394: Didn't find region item for UnitNoRegion\nE475: Invalid argument: Foo grouphere UnitNoRegion /a/",
        ),
        (
            "syntax sync match Foo grouphere",
            "E475: Invalid argument: Foo grouphere",
        ),
        (
            "syntax match Foo! /a/",
            "E5248: Invalid character in group name",
        ),
        (
            "exe \"syntax match Foo\\x01 /a/\"",
            "E669: Unprintable character in group name",
        ),
        (
            "exe 'syntax match ' .. repeat('a', 300) .. ' /a/'",
            "E1249: Highlight group name too long",
        ),
        ("syntax region", "E475: Invalid argument: "),
        ("syntax region Foo", "E475: Invalid argument: Foo"),
        (
            "syntax region Foo start",
            "E398: Missing '=': Foo start\nE475: Invalid argument: Foo start",
        ),
        (
            "syntax region Foo start=/a/",
            "E399: Not enough arguments: syntax region Foo start=/a/",
        ),
        (
            "syntax region Foo start=/a/ end=/b",
            "E475: Invalid argument: Foo start=/a/ end=/b",
        ),
        (
            "syntax region Foo start=/a/ end=/b/ junk",
            "E475: Invalid argument: Foo start=/a/ end=/b/ junk",
        ),
        (
            "syntax region Foo matchgroup start=/a/ end=/b/",
            "E398: Missing '=': Foo matchgroup start=/a/ end=/b/\nE475: Invalid argument: Foo matchgroup start=/a/ end=/b/",
        ),
        (
            "syntax region Foo start=/a/ end=/b/ contains=Bar,ALL",
            "E408: ALL must be first in contains list\nE475: Invalid argument: Foo start=/a/ end=/b/ contains=Bar,ALL",
        ),
        (
            "syntax region Foo start=/a/ end=/b/ grouphere Bar",
            "E393: group[t]here not accepted here\nE475: Invalid argument: Foo start=/a/ end=/b/ grouphere Bar",
        ),
        ("syntax keyword", "E475: Invalid argument: "),
        ("syntax keyword Foo", "E475: Invalid argument: Foo"),
        ("syntax keyword Foo a[b", "E789: Missing ']': ab"),
        (
            "syntax keyword Foo a[b]c",
            "E890: Trailing char after ']': ab]c",
        ),
        (
            "syntax keyword Foo a contains=b",
            "E395: Contains argument not accepted here\nE475: Invalid argument: Foo a contains=b",
        ),
        (
            "syntax keyword Foo a nextgroup=",
            "E406: Empty argument: nextgroup=\nE475: Invalid argument: Foo a nextgroup=",
        ),
        (
            "syntax keyword Foo! a",
            "E5248: Invalid character in group name",
        ),
        (
            "syntax list @UnitNoCluster",
            "E392: No such syntax cluster: @UnitNoCluster",
        ),
        (
            "syntax list UnitNoGroup",
            "E28: No such highlight group name: UnitNoGroup",
        ),
        (
            "syntax clear @UnitNoCluster",
            "E391: No such syntax cluster: @UnitNoCluster",
        ),
        (
            "syntax clear UnitNoGroup",
            "E28: No such highlight group name: UnitNoGroup",
        ),
        (
            "syntax cluster",
            "E400: No cluster specified\nE475: Invalid argument: ",
        ),
        (
            "syntax cluster Foo",
            "E400: No cluster specified\nE475: Invalid argument: Foo",
        ),
        (
            "syntax cluster Foo junk=Bar",
            "E400: No cluster specified\nE475: Invalid argument: Foo junk=Bar",
        ),
        (
            "syntax cluster Foo add=",
            "E406: Empty argument: add=\nE475: Invalid argument: \nE400: No cluster specified",
        ),
        (
            "syntax cluster Foo add=Bar!",
            "E5248: Invalid character in group name\nE409: Unknown group name: Bar!\nE475: Invalid argument: Bar!\nE400: No cluster specified\nE475: Invalid argument: Foo add=Bar!",
        ),
        ("syntax cluster Foo! add=Bar", "none"),
        ("syntime foo", "E475: Invalid argument: foo"),
        ("syntax iskeyword @,x-", "none"),
        ("syntax match Foo /a/ contains=Bar", "none"),
    ] {
        fx.check_messages(cmd, want);
    }
    restore(&fx);
}

#[test]
fn highlight_refusals_report_once() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    buffer(&fx);
    for (cmd, want) in [
        (
            "highlight UnitNoSuchGroup",
            "E411: Highlight group not found: UnitNoSuchGroup",
        ),
        (
            "highlight link UnitHlA",
            "E412: Not enough arguments: \":highlight link UnitHlA\"",
        ),
        (
            "highlight link UnitHlA UnitHlB UnitHlC",
            "E413: Too many arguments: \":highlight link UnitHlA UnitHlB UnitHlC\"",
        ),
        ("highlight UnitHlA =x", "E415: Unexpected equal sign: =x"),
        (
            "highlight UnitHlA ctermfg",
            "E416: Missing equal sign: ctermfg",
        ),
        (
            "highlight UnitHlA ctermfg=",
            "E417: Missing argument: ctermfg=",
        ),
        (
            "highlight UnitHlA guifg='x",
            "E475: Invalid argument: guifg='x",
        ),
        (
            "highlight UnitHlA term=nosuchattr",
            "E418: Illegal value: nosuchattr",
        ),
        (
            "highlight UnitHlA cterm=bold,nosuchattr",
            "E418: Illegal value: bold,nosuchattr",
        ),
        (
            "highlight UnitHlA nosuchkey=1",
            "E423: Illegal argument: nosuchkey=1",
        ),
        (
            "highlight UnitHlA ctermfg=nosuchcolor",
            "E421: Color name or number not recognized: ctermfg=nosuchcolor",
        ),
        (
            "highlight UnitHlA ctermbg=nosuchcolor",
            "E421: Color name or number not recognized: ctermbg=nosuchcolor",
        ),
        ("highlight UnitHlA guifg=nosuchcolor", "none"),
        ("highlight UnitHlA guisp=nosuchcolor", "none"),
        ("highlight UnitHlA blend=x", "none"),
        (
            "exe 'highlight UnitHlA guifg=' .. repeat('a', 600)",
            "E423: Illegal argument",
        ),
        (
            "exe 'highlight ' .. repeat('a', 300) .. ' ctermfg=1'",
            "E1249: Highlight group name too long",
        ),
        (
            "highlight UnitHl! ctermfg=1",
            "E5248: Invalid character in group name",
        ),
        (
            "exe \"highlight UnitHl\\x01 ctermfg=1\"",
            "E669: Unprintable character in group name",
        ),
        ("highlight UnitHlSet ctermfg=1", "none"),
        (
            "highlight link UnitHlSet Search",
            "E414: Group has settings, highlight link ignored",
        ),
        ("highlight! link UnitHlSet Search", "none"),
        ("highlight default link UnitHlSet2 Search", "none"),
        (
            "highlight link UnitHl! Search",
            "E5248: Invalid character in group name",
        ),
        (
            "highlight clear UnitHl!",
            "E5248: Invalid character in group name",
        ),
        (
            "highlight UnitHlA ctermfg=1 ctermfg",
            "E416: Missing equal sign: ctermfg",
        ),
        ("echo hlID(repeat('a', 300))", "none"),
        ("echo hlexists('UnitHl!')", "none"),
        ("echo synIDattr(hlID('UnitHl!'), 'fg')", "none"),
        (
            "call matchadd('UnitHl!', 'a')",
            "E5248: Invalid character in group name",
        ),
        (
            "call matchadd(repeat('a', 300), 'a')",
            "E1249: Highlight group name too long",
        ),
        (
            "set winhighlight=Normal:UnitHl!",
            "E5248: Invalid character in group name\nE474: Invalid argument: winhighlight=Normal:UnitHl!",
        ),
        ("set winhighlight&", "none"),
        (
            "set highlight=8:UnitHl!",
            "E519: Option not supported: highlight=8:UnitHl!",
        ),
        ("set highlight&", "none"),
    ] {
        fx.check_messages(cmd, want);
    }
    for (expr, want) in [
        ("hlID('UnitHl!')", "0"),
        ("hlID(repeat('a', 300))", "0"),
        ("hlexists('UnitHl!')", "0"),
        ("hlexists('UnitHlSet')", "1"),
    ] {
        fx.run(&format!("silent! let g:answer = {expr}"));
        fx.check("g:answer", want);
    }
    fx.run("highlight clear UnitHlSet | highlight clear UnitHlA");
    restore(&fx);
}

#[test]
fn spell_refusals_report_once() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    buffer(&fx);
    fx.run("call setline(1, 'one two')");
    for (cmd, want) in [
        (
            "set nospell | normal! ]s",
            "E756: Spell checking is not possible",
        ),
        (
            "set nospell | normal! [s",
            "E756: Spell checking is not possible",
        ),
        ("set nospell | normal! z=", "none"),
        ("set spell spelllang=unitnosuch", "none"),
        ("echo spellbadword('one')", "none"),
        ("echo spellsuggest('one')", "none"),
        ("set spell& spelllang&", "none"),
    ] {
        fx.check_messages(cmd, want);
    }
    restore(&fx);
}
