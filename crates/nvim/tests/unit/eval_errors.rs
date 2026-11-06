//! What the expression evaluator, its builtins, the JSON and msgpack codecs
//! and the option setter say when they refuse: every message each refusal
//! puts out, in order, and the value a builtin answers with, so a message
//! that doubles, drops or moves shows as a moved row.
//!
//! Written against the behaviour before the helpers below the builtins and
//! the option callbacks stop printing at the point of refusal; a row that
//! moves is a behaviour change.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// One buffer of the case's own, holding `abc`.
fn buffer(fx: &Probe) {
    fx.home();
    fx.run("new | only!");
    fx.run("call setline(1, 'abc')");
}

fn restore(fx: &Probe) {
    fx.run("set nomodified");
    fx.home();
    fx.settle();
}

/// `let g:r = {expr}`: every message it puts out, then what `g:r` holds
/// (`'unset'` when the assignment never happened).
fn check_value(fx: &Probe, expr: &str, want_messages: &str, want_value: &str) {
    fx.run("let g:r = 'unset'");
    fx.check_messages(&format!("let g:r = {expr}"), want_messages);
    fx.check("g:r", want_value);
}

#[test]
fn expression_refusals_report_once() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    buffer(&fx);
    fx.run("let g:l = [1, 2] | let g:d = {}");
    for (expr, msgs, value) in [
        ("1.0 % 2.0", "E804: Cannot use '%' with Float", "'unset'"),
        (
            "[1] == {}",
            "E691: Can only compare List with List",
            "'unset'",
        ),
        ("[1] < [2]", "E692: Invalid operation for List", "'unset'"),
        (
            "{} < {}",
            "E736: Invalid operation for Dictionary",
            "'unset'",
        ),
        (
            "{} == []",
            "E691: Can only compare List with List",
            "'unset'",
        ),
        ("0z00 < 0z00", "E978: Invalid operation for Blob", "'unset'"),
        (
            "0z00 == []",
            "E977: Can only compare Blob with Blob",
            "'unset'",
        ),
        (
            "function('tr') < function('tr')",
            "E694: Invalid operation for Funcrefs",
            "'unset'",
        ),
        ("[] is 1", "none", "0"),
        (
            "sockconnect('pipe', '/unit/no/such/socket')",
            "connection failed: connection refused",
            "0",
        ),
        (
            "get(1, 0)",
            "E896: Argument of get() must be a List, Dictionary or Blob",
            "0",
        ),
        (
            "ctxget('x')",
            "E475: Invalid argument: expected nothing or a Number as an argument",
            "0",
        ),
        ("jobstop('x')", "E474: Invalid argument", "0"),
        ("jobwait(['x'])", "none", "[-3]"),
        (
            "matchbufline(bufnr(), 'a', 1, 1, {'submatches': 1})",
            "E475: Invalid value for argument submatches",
            "[]",
        ),
        (
            "getregion([0, 1, 1, 0], [0, 1, 2, 0], {'type': 'x'})",
            "E475: Invalid value for argument type: x",
            "[]",
        ),
        (
            "getregion([0, 99, 1, 0], [0, 1, 1, 0])",
            "E966: Invalid line number: 99",
            "[]",
        ),
        (
            "getregion([0, 1, 99, 0], [0, 1, 1, 0])",
            "E964: Invalid column number: 99",
            "[]",
        ),
        (
            "getregion([0, 1, 1, 0], [0, 1, 99, 0])",
            "E964: Invalid column number: 99",
            "[]",
        ),
        (
            "getregionpos([0, 99, 1, 0], [0, 1, 1, 0])",
            "E966: Invalid line number: 99",
            "[]",
        ),
        ("search('a', 'q')", "E475: Invalid argument: q", "0"),
        ("searchpos('a', 'q')", "E475: Invalid argument: q", "[0, 0]"),
        (
            "searchpair('a', '', 'b', 'e')",
            "E475: Invalid argument: e",
            "0",
        ),
        ("searchpair('a', '', 'b', 'nc')", "none", "0"),
        (
            "searchpair('a', '', 'b', '', '', -1)",
            "E475: Invalid argument: -1",
            "0",
        ),
        (
            "searchpair('a', '', 'b', '', '', 0, -1)",
            "E475: Invalid argument: -1",
            "0",
        ),
        (
            "searchpairpos('a', '', 'b', 'e')",
            "E475: Invalid argument: e",
            "[0, 0]",
        ),
        ("islocked('g:l[0:1]')", "E786: Range not allowed", "-1"),
        (
            "islocked('g:d.nokey')",
            "E716: Key not present in Dictionary: \"nokey\"",
            "-1",
        ),
        ("islocked('g:nosuch')", "none", "-1"),
        ("str2nr([])", "E730: Using a List as a String", "0"),
        ("str2nr({})", "E731: Using a Dictionary as a String", "0"),
        (
            "str2nr(function('tr'))",
            "E729: Using a Funcref as a String",
            "0",
        ),
        ("str2nr(0z00)", "E976: Using a Blob as a String", "0"),
        ("str2nr(1.5)", "none", "1"),
        ("toupper([])", "E730: Using a List as a String", "''"),
        ("[1, 2][[]]", "E730: Using a List as a String", "'unset'"),
        (
            "[1, 2][0 : []]",
            "E730: Using a List as a String",
            "'unset'",
        ),
        ("{'a': 1}[[]]", "E730: Using a List as a String", "'unset'"),
        ("-[]", "E745: Using a List as a Number", "'unset'"),
        ("-{}", "E728: Using a Dictionary as a Number", "'unset'"),
        (
            "+function('tr')",
            "E703: Using a Funcref as a Number",
            "'unset'",
        ),
        ("!0z00", "E974: Using a Blob as a Number", "'unset'"),
        ("[] .. 'a'", "E730: Using a List as a String", "'unset'"),
        (
            "'a' .. {}",
            "E731: Using a Dictionary as a String",
            "'unset'",
        ),
        ("1 + []", "E745: Using a List as a Number", "'unset'"),
        (
            "1.0 + {}",
            "E728: Using a Dictionary as a Number",
            "'unset'",
        ),
        (
            "bufnr([])",
            "E745: Expected a Number or a String, List found",
            "-1",
        ),
        (
            "bufnr({})",
            "E728: Expected a Number or a String, Dictionary found",
            "-1",
        ),
        ("printf('%s', [])", "none", "'[]'"),
        (
            "json_decode([])",
            "E474: Attempt to decode a blank string\nE474: Failed to parse ",
            "0",
        ),
        (
            "writefile([[]], '/unit/no/such/file')",
            "E745: Expected a Number or a String, List found",
            "-1",
        ),
        (
            "writefile(['a', {}], '/unit/no/such/file')",
            "E728: Expected a Number or a String, Dictionary found",
            "-1",
        ),
        (
            "writefile(['a'], '/unit/no/such/dir/file', 'D')",
            "E193: defer not inside a function",
            "-1",
        ),
        (
            "mkdir('/unit/no/such/dir', 'D')",
            "E193: defer not inside a function",
            "0",
        ),
        (
            "jobstart(['true'], {'on_stdout': 1})",
            "E6000: Argument is not a function or function name",
            "0",
        ),
        (
            "jobstart(['true'], {'on_exit': []})",
            "E6000: Argument is not a function or function name",
            "0",
        ),
        (
            "timer_start(10, 1)",
            "E921: Invalid callback argument",
            "-1",
        ),
        ("winnr('x')", "E15: Invalid expression: \"x\"", "0"),
        ("winnr('3x')", "E15: Invalid expression: \"3x\"", "0"),
        ("winnr('$x')", "E15: Invalid expression: \"$x\"", "0"),
        ("tabpagenr('x')", "E15: Invalid expression: \"x\"", "0"),
        (
            "tabpagewinnr(1, 'x')",
            "E15: Invalid expression: \"x\"",
            "0",
        ),
        (
            "function('s:unit_f')",
            "E81: Using <SID> not in a script context\nE475: Invalid argument: s:unit_f",
            "0",
        ),
        (
            "funcref('s:unit_f')",
            "E81: Using <SID> not in a script context\nE475: Invalid argument: s:unit_f",
            "0",
        ),
        ("{a, a -> 1}", "E853: Duplicate argument name: a", "'unset'"),
        (
            "call('s:unit_f', [])",
            "E81: Using <SID> not in a script context\nE117: Unknown function: s:unit_f",
            "0",
        ),
        (
            "unit_nosuch#f()",
            "E117: Unknown function: unit_nosuch#f",
            "'unset'",
        ),
        (
            "setbufvar('', 'a b', 1)",
            "E461: Illegal variable name: a b",
            "0",
        ),
        (
            "type(extend(g:, {'a b': 1}))",
            "E461: Illegal variable name: a b",
            "4",
        ),
        ("type(extend(g:, {'unit_ok': 1}))", "none", "4"),
        ("getbufvar('', '&nosuch', 'dflt')", "none", "'dflt'"),
        (
            "execute('defer Unit_none()')",
            "E193: defer not inside a function",
            "'\nE193: defer not inside a function'",
        ),
        (
            "execute('throw')",
            "E471: Argument required: throw",
            "'\nE471: Argument required: throw'",
        ),
    ] {
        check_value(&fx, expr, msgs, value);
    }
    fx.run("unlet! g:unit_ok g:l g:d g:r");
    restore(&fx);
}

#[test]
fn command_refusals_report_once() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    buffer(&fx);
    fx.run("let g:l = [1, 2] | let g:d = {}");
    fx.run("let g:deep = [] | for i in range(110) | let g:deep = [g:deep] | endfor");
    fx.run("function! Unit_listed()\nendfunction");
    for (cmd, want) in [
        (
            "lockvar 200 g:deep",
            "E743: Variable nested too deep for (un)lock",
        ),
        (
            "unlockvar 200 g:deep",
            "E743: Variable nested too deep for (un)lock",
        ),
        (
            "exe \"function! Unit_dup(a, a)\\nendfunction\"",
            "E853: Duplicate argument name: a",
        ),
        (
            "exe \"function! s:unit_s()\\nendfunction\"",
            "E81: Using <SID> not in a script context",
        ),
        (
            "exe \"function! unit_lower()\\nendfunction\"",
            "E128: Function name must start with a capital or \"s:\": unit_lower()\nendfunction",
        ),
        (
            "exe \"function! Unit:colon()\\nendfunction\"",
            "E124: Missing '(': Unit:colon()\nendfunction",
        ),
        ("exe \"function! g:Unit_g() dict\\nendfunction\"", "none"),
        (
            "exe \"function! g:.Unit_g()\\nendfunction\"",
            "E862: Cannot use g: here",
        ),
        (
            "function Unit_listed junk",
            "E488: Trailing characters:  junk",
        ),
        ("defer Unit_listed()", "E193: defer not inside a function"),
        (
            "let 9a = 1",
            "E121: Undefined variable: 9a\nE488: Trailing characters: = 1",
        ),
        (
            "let v:['lua'] = 1",
            "E46: Cannot change read-only variable \"v:['lua']\"",
        ),
        ("unlet v:['lua']", "E461: Illegal variable name: v:['lua']"),
        ("edit #<99", "E684: List index out of range: 98"),
        (
            "let v:lua = 1",
            "E46: Cannot change read-only variable \"v:lua\"",
        ),
        ("unlet v:lua", "E795: Cannot delete variable v:lua"),
        ("let v:nosuch = 1", "E461: Illegal variable name: v:nosuch"),
        ("let g:['a b'] = 1", "E461: Illegal variable name: a b"),
        ("let g:d['a b'] = 1", "none"),
        (
            "let g:unit_v = function('s:unit_f')",
            "E81: Using <SID> not in a script context\nE475: Invalid argument: s:unit_f",
        ),
        (
            "call execute(['let g:h =<< eval END', 'a}b', 'END'])",
            "E1278: Stray '}' without a matching '{': a}b",
        ),
        (
            "call execute(['let g:h =<< eval END', 'a{1}b{', 'END'])",
            "E1279: Missing '}': {",
        ),
        (
            "call execute(['let g:h =<< END junk', 'a', 'END'])",
            "E488: Trailing characters:  junk",
        ),
        (
            "call execute(['let g:h =<< trim', 'a', 'END'])",
            "E172: Missing marker",
        ),
        (
            "call execute(['let g:h =<< end', 'a', 'end'])",
            "E221: Marker cannot start with lower case letter",
        ),
        ("throw", "E471: Argument required: silent throw"),
        ("throw ''", "Error in :\nE605: Exception not caught: "),
        (
            "call execute(['try', 'throw \"x\"', 'catch /\\(/', 'endtry'])",
            "E475: Invalid argument: \\(",
        ),
        (
            "call execute(['try', 'throw \"x\"', 'catch /x/', 'endtry'])",
            "none",
        ),
        ("tabnew | let g:w = win_getid() | tabprev", "none"),
        (
            "call win_move_separator(g:w, 1)",
            "E1308: Cannot resize a window in another tab page",
        ),
        (
            "call win_move_statusline(g:w, 1)",
            "E1308: Cannot resize a window in another tab page",
        ),
        ("call win_move_separator(9999, 1)", "none"),
        ("tabonly!", "none"),
    ] {
        fx.check_messages(cmd, want);
    }
    fx.run("unlet! g:l g:d g:deep g:h g:w g:unit_v g:i");
    fx.run("silent! delfunction Unit_listed");
    restore(&fx);
}

#[test]
fn codec_refusals_report_once() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    buffer(&fx);
    fx.run("let g:sr = {} | let g:sr.s = g:sr");
    for (expr, msgs, value) in [
        (
            "json_decode('')",
            "E474: Attempt to decode a blank string\nE474: Failed to parse ",
            "0",
        ),
        (
            "json_decode(' ')",
            "E474: Attempt to decode a blank string\nE474: Failed to parse  ",
            "0",
        ),
        (
            "json_decode(']')",
            "E474: No container to close: ]\nE474: Failed to parse ]",
            "0",
        ),
        (
            "json_decode('[}')",
            "E474: Closing list with curly bracket: }\nE474: Failed to parse [}",
            "0",
        ),
        (
            "json_decode('{]')",
            "E474: Closing dictionary with square bracket: ]\nE474: Failed to parse {]",
            "0",
        ),
        (
            "json_decode('[1,]')",
            "E474: Trailing comma: ]\nE474: Failed to parse [1,]",
            "0",
        ),
        (
            "json_decode('{\"a\":}')",
            "E474: Expected value after colon: }\nE474: Failed to parse {\"a\":}",
            "0",
        ),
        (
            "json_decode('{\"a\"}')",
            "E474: Expected value: }\nE474: Failed to parse {\"a\"}",
            "0",
        ),
        (
            "json_decode(',')",
            "E474: Comma not inside container: ,\nE474: Failed to parse ,",
            "0",
        ),
        (
            "json_decode('[1,,2]')",
            "E474: Duplicate comma: ,2]\nE474: Failed to parse [1,,2]",
            "0",
        ),
        (
            "json_decode('{\"a\":,1}')",
            "E474: Comma after colon: ,1}\nE474: Failed to parse {\"a\":,1}",
            "0",
        ),
        (
            "json_decode('{\"a\",1}')",
            "E474: Using comma in place of colon: ,1}\nE474: Failed to parse {\"a\",1}",
            "0",
        ),
        (
            "json_decode('[,1]')",
            "E474: Leading comma: ,1]\nE474: Failed to parse [,1]",
            "0",
        ),
        (
            "json_decode('{\"a\",:1}')",
            "E474: Using comma in place of colon: ,:1}\nE474: Failed to parse {\"a\",:1}",
            "0",
        ),
        (
            "json_decode(':')",
            "E474: Colon not inside container: :\nE474: Failed to parse :",
            "0",
        ),
        (
            "json_decode('[1:2]')",
            "E474: Using colon not in dictionary: :2]\nE474: Failed to parse [1:2]",
            "0",
        ),
        (
            "json_decode('{:1}')",
            "E474: Unexpected colon: :1}\nE474: Failed to parse {:1}",
            "0",
        ),
        (
            "json_decode('{\"a\"::1}')",
            "E474: Duplicate colon: :1}\nE474: Failed to parse {\"a\"::1}",
            "0",
        ),
        (
            "json_decode('nul')",
            "E474: Expected null: nul\nE474: Failed to parse nul",
            "0",
        ),
        (
            "json_decode('tru')",
            "E474: Expected true: tru\nE474: Failed to parse tru",
            "0",
        ),
        (
            "json_decode('fals')",
            "E474: Expected false: fals\nE474: Failed to parse fals",
            "0",
        ),
        (
            "json_decode('x')",
            "E474: Unidentified byte: x\nE474: Failed to parse x",
            "0",
        ),
        (
            "json_decode('1 2')",
            "E474: Trailing characters: 2\nE474: Failed to parse 1 2",
            "0",
        ),
        (
            "json_decode('[1')",
            "E474: Unexpected end of input: [1\nE474: Failed to parse [1",
            "0",
        ),
        (
            "json_decode('[1 2]')",
            "E474: Expected comma before list item: ]\nE474: Failed to parse [1 2]",
            "0",
        ),
        (
            "json_decode('{\"a\":1 \"b\":2}')",
            "E474: Expected comma before dictionary key: \":2}\nE474: Failed to parse {\"a\":1 \"b\":2}",
            "0",
        ),
        (
            "json_decode('{1:2}')",
            "E474: Expected string key: :2}\nE474: Failed to parse {1:2}",
            "0",
        ),
        (
            "json_decode('\"\\')",
            "E474: Unfinished escape sequence: \"\\\nE474: Failed to parse \"\\",
            "0",
        ),
        (
            "json_decode('\"\\u')",
            "E474: Unfinished unicode escape sequence: \"\\u\nE474: Failed to parse \"\\u",
            "0",
        ),
        (
            "json_decode('\"\\u12x4\"')",
            "E474: Expected four hex digits after \\u: \\u12x4\"\nE474: Failed to parse \"\\u12x4\"",
            "0",
        ),
        (
            "json_decode('\"\\x\"')",
            "E474: Unknown escape sequence: \\x\"\nE474: Failed to parse \"\\x\"",
            "0",
        ),
        (
            "json_decode(\"\\\"\\x01\\\"\")",
            "E474: ASCII control characters cannot be present inside string: ^A\"\nE474: Failed to parse \"^A\"",
            "0",
        ),
        (
            "json_decode(\"\\\"\\xff\\\"\")",
            "E474: Only UTF-8 strings allowed: <ff>\"\nE474: Failed to parse \"<ff>\"",
            "0",
        ),
        (
            "json_decode(\"\\\"\\xf4\\x90\\x80\\x80\\\"\")",
            "E474: Only UTF-8 code points up to U+10FFFF are allowed to appear unescaped: ����\"\nE474: Failed to parse \"����\"",
            "0",
        ),
        (
            "json_decode('\"abc')",
            "E474: Expected string end: \"abc\nE474: Failed to parse \"abc",
            "0",
        ),
        (
            "json_decode('01')",
            "E474: Leading zeroes are not allowed: 01\nE474: Failed to parse 01",
            "0",
        ),
        (
            "json_decode('-')",
            "E474: Missing number after minus sign: -\nE474: Failed to parse -",
            "0",
        ),
        (
            "json_decode('1.')",
            "E474: Missing number after decimal dot: 1.\nE474: Failed to parse 1.",
            "0",
        ),
        (
            "json_decode('1e')",
            "E474: Missing exponent: 1e\nE474: Failed to parse 1e",
            "0",
        ),
        (
            "json_decode('1.5e+')",
            "E474: Missing exponent: 1.5e+\nE474: Failed to parse 1.5e+",
            "0",
        ),
        ("json_decode(['[1,', '2]'])", "none", "[1, 2]"),
        (
            "json_encode(function('tr'))",
            "E474: Error while dumping encode_tv2json() argument, itself: attempt to dump function reference",
            "''",
        ),
        (
            "json_encode(\"\\xff\")",
            "E474: String \"<ff>\" contains byte that does not start any UTF-8 character",
            "''",
        ),
        (
            "json_encode(\"a\\xc3\")",
            "E474: String \"<c3>\" contains byte that does not start any UTF-8 character",
            "''",
        ),
        (
            "json_encode({'_TYPE': v:msgpack_types.map, '_VAL': [[1, 2]]})",
            "E474: Invalid key in special dictionary",
            "''",
        ),
        (
            "json_encode({'_TYPE': v:msgpack_types.binary, '_VAL': ['a']})",
            "E716: Key not present in Dictionary: \"binary\"\nE116: Invalid arguments for function json_encode",
            "'unset'",
        ),
        (
            "json_encode({'_TYPE': v:msgpack_types.ext, '_VAL': [1, ['a']]})",
            "E474: Unable to convert EXT string to JSON",
            "''",
        ),
        (
            "json_encode({'_TYPE': v:msgpack_types.integer, '_VAL': [1, 1, 0, 0]})",
            "none",
            "'4611686018427387904'",
        ),
        (
            "json_encode(g:sr)",
            "E724: unable to correctly dump variable with self-referencing container",
            "'{\"s\": }'",
        ),
        (
            "msgpackdump([function('tr')])",
            "E5004: Error while dumping msgpackdump() argument, index 0, itself: attempt to dump function reference",
            "[]",
        ),
        (
            "msgpackdump(1)",
            "E686: Argument of msgpackdump() must be a List",
            "0",
        ),
        ("msgpackdump([1], 'x')", "none", "['\u{1}']"),
        (
            "len(msgpackdump([{'_TYPE': v:msgpack_types.map, '_VAL': 1}]))",
            "none",
            "1",
        ),
        (
            "msgpackparse(1)",
            "E899: Argument of msgpackparse() must be a List or Blob",
            "0",
        ),
        (
            "msgpackparse([1])",
            "E475: Invalid argument: List item is not a string",
            "[]",
        ),
        ("msgpackparse(0z)", "none", "[]"),
        (
            "msgpackparse(0zc1)",
            "E475: Invalid argument: Failed to parse msgpack string",
            "[]",
        ),
        (
            "msgpackparse(0z91)",
            "E475: Invalid argument: Incomplete msgpack string",
            "[]",
        ),
        (
            "string(function('tr', [g:sr]))",
            "E724: unable to correctly dump variable with self-referencing container",
            "'function(''tr'', [{''s'': {E724@2}}])'",
        ),
        ("execute('echo [g:sr]')", "none", "'\n[{''s'': {...@1}}]'"),
    ] {
        check_value(&fx, expr, msgs, value);
    }
    fx.run("unlet! g:r g:sr");
    restore(&fx);
}

#[test]
fn option_refusals_report_once() {
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    buffer(&fx);
    for (cmd, want) in [
        (
            "set tabstop=-1",
            "E487: Argument must be positive: tabstop=-1",
        ),
        ("set tabstop=x", "E521: Number required after =: tabstop=x"),
        ("set tabstop=99999", "E474: Invalid argument: tabstop=99999"),
        (
            "set shiftwidth=-1",
            "E487: Argument must be positive: shiftwidth=-1",
        ),
        ("set fileformat=x", "E474: Invalid argument: fileformat=x"),
        ("set fileformats=x", "E474: Invalid argument: fileformats=x"),
        ("set nosuchoption", "E518: Unknown option: nosuchoption"),
        ("set nosuchoption=1", "E518: Unknown option: nosuchoption=1"),
        ("set ts", "none"),
        ("set ts!", "E488: Trailing characters: ts!"),
        ("set invts", "E474: Invalid argument: invts"),
        ("set fileformat+=x", "E474: Invalid argument: fileformat+=x"),
        (
            "set history=-1",
            "E487: Argument must be positive: history=-1",
        ),
        (
            "set winheight=0",
            "E487: Argument must be positive: winheight=0",
        ),
        (
            "set winminheight=99",
            "E591: 'winheight' cannot be smaller than 'winminheight': winminheight=99",
        ),
        (
            "set winwidth=0",
            "E487: Argument must be positive: winwidth=0",
        ),
        (
            "set winminwidth=99",
            "E592: 'winwidth' cannot be smaller than 'winminwidth': winminwidth=99",
        ),
        ("set scroll=-1", "none"),
        (
            "set textwidth=-1",
            "E487: Argument must be positive: textwidth=-1",
        ),
        (
            "set report=-1",
            "E487: Argument must be positive: report=-1",
        ),
        (
            "set cmdheight=-1",
            "E487: Argument must be positive: cmdheight=-1",
        ),
        (
            "set titlelen=-1",
            "E487: Argument must be positive: titlelen=-1",
        ),
        (
            "set undolevels=x",
            "E521: Number required after =: undolevels=x",
        ),
        (
            "set numberwidth=0",
            "E487: Argument must be positive: numberwidth=0",
        ),
        (
            "set numberwidth=99",
            "E474: Invalid argument: numberwidth=99",
        ),
        (
            "set conceallevel=9",
            "E474: Invalid argument: conceallevel=9",
        ),
        ("set wildchar=x | echomsg &wildchar", "120"),
        ("set wildchar=<Esc> | echomsg &wildchar", "27"),
        ("set wildchar=99999999 | echomsg &wildchar", "99999999"),
        ("set wildcharm=<Tab> | echomsg &wildcharm", "9"),
        ("set cedit=<C-F> | echomsg &cedit", "<C-F>"),
        (
            "set cedit=x | echomsg &cedit",
            "E474: Invalid argument: cedit=x",
        ),
        ("set cedit= | echomsg &cedit", "none"),
        (
            "set backupext=.orig patchmode=.orig",
            "E589: 'backupext' and 'patchmode' are equal: patchmode=.orig",
        ),
        ("set patchmode=", "none"),
        (
            "exe \"set showbreak=\\x01\"",
            "E595: 'showbreak' contains unprintable or wide character: showbreak=^A",
        ),
        (
            "set statusline=%{",
            "E540: Unclosed expression sequence: statusline=%{",
        ),
        (
            "set statusline=%(",
            "E542: Unbalanced groups: statusline=%(",
        ),
        ("set statusline=%!", "none"),
        ("set statusline=", "none"),
        (
            "set listchars=tab:x",
            "E1511: Wrong number of characters for field \"tab\": listchars=tab:x",
        ),
        (
            "set listchars=leadtab:ab",
            "E1572: 'listchars' field \"leadtab\" requires \"tab\" to be specified: listchars=leadtab:ab",
        ),
        (
            "set fillchars=vert:xy",
            "E1511: Wrong number of characters for field \"vert\": fillchars=vert:xy",
        ),
        ("set fillchars=", "none"),
        ("set listchars=eol:$", "none"),
        ("set completeopt=x", "E474: Invalid argument: completeopt=x"),
        (
            "set encoding=latin1",
            "E519: Option not supported: encoding=latin1",
        ),
        ("set modifiable | set nomodifiable", "none"),
        (
            "set fileformat=dos",
            "E21: Cannot make changes, 'modifiable' is off: fileformat=dos",
        ),
        ("set modifiable", "none"),
        ("set fileformat=unix", "none"),
        ("set spelllang=a/b", "E474: Invalid argument: spelllang=a/b"),
        (
            "set rulerformat=%{",
            "E540: Unclosed expression sequence: rulerformat=%{",
        ),
        ("set mouse=z", "E539: Illegal character <z>: mouse=z"),
        (
            "set whichwrap=z",
            "E539: Illegal character <z>: whichwrap=z",
        ),
        ("set guicursor=x", "E545: Missing colon: guicursor=x"),
        ("set matchpairs=x", "E474: Invalid argument: matchpairs=x"),
        ("set comments=x", "E524: Missing colon: comments=x"),
        ("set foldmethod=x", "E474: Invalid argument: foldmethod=x"),
        ("let &tabstop = -1", "E487: Argument must be positive"),
        ("let &fileformat = 'x'", "E474: Invalid argument"),
        ("let &l:fileformat = 'x'", "E474: Invalid argument"),
        (
            "let &nosuchoption = 1",
            "E355: Unknown option: nosuchoption",
        ),
        (
            "call setbufvar('', '&fileformat', 'x')",
            "E474: Invalid argument",
        ),
        (
            "call setbufvar('', '&tabstop', -1)",
            "E487: Argument must be positive",
        ),
        ("call setwinvar(0, '&scroll', -1)", "none"),
        (
            "call nvim_set_option_value('tabstop', -1, {})",
            "E5555: API call: E487: Argument must be positive",
        ),
        ("set tabstop=8 shiftwidth=8 history=10000", "none"),
        (
            "set winheight=1 winminheight=1 winwidth=20 winminwidth=1",
            "none",
        ),
        (
            "set textwidth=0 report=2 cmdheight=1 titlelen=85 undolevels=1000",
            "none",
        ),
        (
            "set numberwidth=4 conceallevel=0 wildchar=<Tab> wildcharm=0",
            "none",
        ),
        (
            "set backupext=~ statusline= rulerformat= listchars& fillchars&",
            "none",
        ),
        (
            "set completeopt& spelllang& mouse& whichwrap& guicursor&",
            "none",
        ),
        (
            "set matchpairs& comments& foldmethod& showbreak& cedit&",
            "none",
        ),
    ] {
        fx.check_messages(cmd, want);
    }
    restore(&fx);
}
