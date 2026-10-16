//! The runtime: reading a sourced script, the script registry, the
//! 'runtimepath' search and its cache, packages, and the completion the
//! runtime commands offer.
//!
//! `getsourceline` is the reader `do_cmdline` pulls a script's lines from: it
//! joins `\` continuations (and skips the `"\ ` comments between them),
//! keeps a newline escaped by an odd run of CTRL-V's, reads a line longer
//! than one chunk whole, strips a UTF-8 BOM and applies 'scriptencoding'.
//! `:source` of a buffer range and `nvim_exec2()` of a string share that
//! reader. The registry answers `:scriptnames`, `getscriptinfo()` and
//! "Last set from". `do_in_runtimepath` walks the cached search path, with
//! `after/` last and the `start` packages spliced in; `:packadd` splices an
//! `opt` package into both 'runtimepath' and the cache.
//!
//! Written against the behaviour before the cookie owns its reader, the
//! registry hands out its items by id, the search callback becomes a closure
//! and the cached path a `Vec`; a row that moves is a behaviour change.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// Check every `(expr, want)` row with the sandbox's path written `<DIR>`,
/// failing once with every row that moved, spelled as the row that would
/// pass.
#[track_caller]
fn table(fx: &Probe, sandbox: &Sandbox, rows: &[(&str, &str)]) {
    let moved: Vec<String> = rows
        .iter()
        .filter_map(|(expr, want)| {
            let got = fx.eval(expr).replace(sandbox.as_str(), "<DIR>");
            (got != *want).then(|| format!("({expr:?}, {got:?}),"))
        })
        .collect();
    assert!(
        moved.is_empty(),
        "{} rows moved:\n{}",
        moved.len(),
        moved.join("\n")
    );
}

/// What `cmd` prints, with the sandbox's path written `<DIR>`.
fn output(fx: &Probe, sandbox: &Sandbox, cmd: &str) -> String {
    fx.output(cmd).replace(sandbox.as_str(), "<DIR>")
}

fn restore(fx: &Probe) {
    fx.home();
    fx.run(
        "if exists('g:rt_save_rtp') | let &rtp = g:rt_save_rtp | let &pp = g:rt_save_pp | endif",
    );
    fx.run("unlet! g:rt_save_rtp g:rt_save_pp");
    fx.run("for s:v in filter(keys(g:), {_, k -> k =~# '^rt_'}) | exe 'unlet! g:' .. s:v | endfor");
    fx.run("set cpoptions& verbose=0");
}

/// Point 'runtimepath' and 'packpath' at the sandbox until [`restore`].
fn own_paths(fx: &Probe, rtp: &str, pp: &str) {
    fx.run("let g:rt_save_rtp = &rtp | let g:rt_save_pp = &pp");
    fx.run(&format!("let &rtp = '{rtp}' | let &pp = '{pp}'"));
}

#[test]
fn a_script_joins_continuations_and_reads_long_lines_whole() {
    let sandbox = Sandbox::dir("runtime_reader");
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    let long = "x".repeat(300);
    let mut script = Vec::new();
    script.extend_from_slice(b"let g:rt_list = ['a',\n");
    script.extend_from_slice(b"      \\ 'b',\n");
    script.extend_from_slice(b"      \"\\ a comment between continuations\n");
    script.extend_from_slice(b"      \\ 'c']\n");
    script.extend_from_slice(b"let g:rt_after_join = expand('<slnum>')\n");
    script.extend_from_slice(format!("let g:rt_long = '{long}'\n").as_bytes());
    // An odd run of CTRL-V's keeps the newline in the line; an even one
    // does not.
    script.extend_from_slice(b"let g:rt_cv1 = 'a\x16\nb'\n");
    script.extend_from_slice(b"let g:rt_cv1_lnum = expand('<slnum>')\n");
    script.extend_from_slice(b"let g:rt_cv2 = 'a\x16\x16'\n");
    script.extend_from_slice(b"let g:rt_cr = 'x'\r\n");
    script.extend_from_slice(
        b"try | call RtNoSuchFunction() | catch | let g:rt_throw = v:throwpoint | endtry\n",
    );
    // A continuation line is not joined under 'cpoptions' C; the line
    // after it then is an error.
    script.extend_from_slice(b"let g:rt_last = 1");
    sandbox.write("reader.vim", &script);
    fx.check_error("source reader.vim", "E488: Trailing characters: \r");
    table(
        &fx,
        &sandbox,
        &[
            ("g:rt_list", "['a', 'b', 'c']"),
            ("g:rt_after_join", "'5'"),
            ("strlen(g:rt_long)", "300"),
            ("g:rt_cv1", "'a\u{16}\nb'"),
            ("g:rt_cv1_lnum", "'9'"),
            ("g:rt_cv2", "'a\u{16}\u{16}'"),
            ("exists('g:rt_cr')", "0"),
            ("g:rt_throw", "'<DIR>/reader.vim, line 12'"),
            ("g:rt_last", "1"),
        ],
    );

    // 'cpoptions' C: no joining.
    sandbox.write(
        "nojoin.vim",
        b"let g:rt_nojoin = ['a',\n\\ 'b']\nlet g:rt_nojoin_done = 1\n",
    );
    fx.run("set cpoptions+=C");
    let err = fx.error_from("source nojoin.vim");
    fx.run("set cpoptions&");
    assert_eq!(err, "E10: \\ should be followed by /, ? or &: \\ ''b'']");
    table(
        &fx,
        &sandbox,
        &[("exists('g:rt_nojoin')", "0"), ("g:rt_nojoin_done", "1")],
    );
    restore(&fx);
}

#[test]
fn a_bom_and_scriptencoding_convert_the_lines() {
    let sandbox = Sandbox::dir("runtime_encoding");
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    sandbox.write(
        "bom.vim",
        b"\xef\xbb\xbflet g:rt_bom = '\xc3\xa9'\nlet g:rt_bom2 = 'e'\n",
    );
    sandbox.write(
        "latin.vim",
        b"scriptencoding latin1\nlet g:rt_latin = '\xe9'\nlet g:rt_cont = ['\xe9',\n  \\ '\xe8']\n",
    );
    fx.check_error("source bom.vim", "none");
    fx.check_error("source latin.vim", "none");
    table(
        &fx,
        &sandbox,
        &[
            ("g:rt_bom", "'é'"),
            ("g:rt_bom2", "'e'"),
            ("g:rt_latin", "'é'"),
            ("strlen(g:rt_latin)", "2"),
            ("g:rt_cont", "['é', 'è']"),
        ],
    );
    fx.check_error(
        "scriptencoding latin1",
        "E167: :scriptencoding used outside of a sourced file",
    );
    restore(&fx);
}

#[test]
fn a_buffer_range_and_a_string_source_through_the_same_reader() {
    let sandbox = Sandbox::dir("runtime_buffer");
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    fx.run("call setline(1, ['let g:rt_one = 1', 'let g:rt_two = [2,', '  \\ 3]', 'let g:rt_name = expand(\"<sfile>\")', 'let g:rt_lnum = expand(\"<slnum>\")', 'finish', 'let g:rt_never = 1'])");
    fx.check_error("2,5source", "none");
    table(
        &fx,
        &sandbox,
        &[
            ("exists('g:rt_one')", "0"),
            ("g:rt_two", "[2, 3]"),
            ("g:rt_name =~# '^:source buffer=\\d\\+$'", "1"),
            ("g:rt_lnum", "'5'"),
        ],
    );
    fx.check_error("source", "none");
    table(
        &fx,
        &sandbox,
        &[("g:rt_one", "1"), ("exists('g:rt_never')", "0")],
    );
    fx.check_error("source!", "E471: Argument required");
    fx.check_error("2source buf.vim", "E481: No range allowed");

    // A named buffer goes by its file name.
    fx.run("silent! file named.vim");
    fx.run("1,4source");
    table(&fx, &sandbox, &[("g:rt_name", "'<DIR>/named.vim'")]);
    fx.run("silent! 0file");

    // A string, nested inside a sourced script and a function.
    sandbox.write(
        "outer.vim",
        b"function! RtOuter() abort\n  call nvim_exec2(\"let g:rt_inner = expand('<sfile>')\\nlet g:rt_stack = expand('<stack>')\", {})\nendfunction\ncall RtOuter()\nlet g:rt_str = nvim_exec2('echo expand(\"<sfile>\")', {'output': v:true}).output\n",
    );
    fx.check_error("source outer.vim", "none");
    table(
        &fx,
        &sandbox,
        &[
            ("g:rt_inner", "'nvim_exec2() called at RtOuter:1'"),
            (
                "g:rt_stack",
                "'<DIR>/outer.vim[4]..function RtOuter[1]..script nvim_exec2() called at RtOuter:1[2]'",
            ),
            ("g:rt_str", "'nvim_exec2() called at <DIR>/outer.vim:5'"),
        ],
    );
    fx.run("delfunction RtOuter");
    restore(&fx);
}

#[test]
fn the_registry_keeps_one_id_per_script() {
    let sandbox = Sandbox::dir("runtime_registry");
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    sandbox.write(
        "reg.vim",
        b"let g:rt_count = get(g:, 'rt_count', 0) + 1\nlet s:local = 42\nfunction! s:Hidden() abort\nendfunction\nset tabstop=7\nlet g:rt_sid = expand('<SID>')\n",
    );
    fx.check_error("source reg.vim", "none");
    fx.run("let g:rt_first_sid = g:rt_sid");
    fx.check_error("source reg.vim", "none");
    table(
        &fx,
        &sandbox,
        &[
            ("g:rt_count", "2"),
            ("g:rt_sid ==# g:rt_first_sid", "1"),
            (
                "map(getscriptinfo({'name': 'runtime_registry'}), {_, s -> s.name})",
                "['<DIR>/reg.vim']",
            ),
            (
                "map(getscriptinfo({'name': 'runtime_registry'}), {_, s -> s.sid == matchstr(g:rt_sid, '\\d\\+')})",
                "[1]",
            ),
            (
                "getscriptinfo({'sid': matchstr(g:rt_sid, '\\d\\+')})[0].variables",
                "{'local': 42}",
            ),
            (
                "len(getscriptinfo({'sid': matchstr(g:rt_sid, '\\d\\+')})[0].functions)",
                "1",
            ),
            ("getscriptinfo({'sid': 999999})", "[]"),
            ("getscriptinfo({'name': 'no-such-script-anywhere'})", "[]"),
        ],
    );
    let names = output(&fx, &sandbox, "scriptnames");
    let ours: Vec<&str> = names
        .lines()
        .filter(|line| line.contains("<DIR>"))
        .map(|line| line.split_once(": ").map_or(line, |(_, name)| name))
        .collect();
    assert_eq!(ours, ["<DIR>/reg.vim"]);
    let verbose = output(&fx, &sandbox, "verbose set tabstop?");
    assert_eq!(
        verbose,
        "\n  tabstop=7\n\tLast set from <DIR>/reg.vim line 5"
    );
    fx.check_error(
        "script 0",
        "E939: Positive count required: silent! script 0",
    );
    fx.run("set tabstop&");
    restore(&fx);
}

/// A runtime tree: two 'runtimepath' entries, an `after/` one, and a
/// 'packpath' with a `start` and an `opt` package, each with an `after/`.
/// Every script appends its own name to `g:rt_order`.
fn runtime_tree(sandbox: &Sandbox) {
    for (dir, name) in [
        ("a/plugin", "a1"),
        ("b/plugin", "b1"),
        ("a/after/plugin", "aa"),
        ("pp/pack/p/start/s/plugin", "s1"),
        ("pp/pack/p/start/s/after/plugin", "sa"),
        ("pp/pack/p/opt/o/plugin", "o1"),
        ("pp/pack/p/opt/o/after/plugin", "oa"),
    ] {
        sandbox.mkdir(dir);
        let body = format!("call add(g:rt_order, '{name}')\n");
        sandbox.write(&format!("{dir}/{name}.vim"), body.as_bytes());
    }
    sandbox.mkdir("b/plugin/sub");
    sandbox.write(
        "b/plugin/sub/deep.lua",
        b"vim.g.rt_order_lua = vim.list_extend(vim.g.rt_order_lua, {'deep'})\n",
    );
    sandbox.write(
        "b/plugin/b2.lua",
        b"vim.g.rt_lua = (vim.g.rt_lua or 0) + 1\n",
    );
    sandbox.mkdir("a/colors");
    sandbox.write("a/colors/rtdark.vim", b"");
    sandbox.write("a/colors/rtlight.lua", b"");
    sandbox.mkdir("b/colors");
    sandbox.write("b/colors/rtdark.vim", b"");
    sandbox.mkdir("a/autoload");
    sandbox.write(
        "a/autoload/rtauto.vim",
        b"let g:rt_auto_loads = get(g:, 'rt_auto_loads', 0) + 1\nfunction! rtauto#f() abort\n  return 'f'\nendfunction\n",
    );
}

#[test]
fn runtime_walks_the_cached_path_in_order() {
    let sandbox = Sandbox::dir("runtime_search");
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    runtime_tree(&sandbox);
    let dir = sandbox.as_str();
    own_paths(
        &fx,
        &format!("{dir}/a,{dir}/b,{dir}/a/after"),
        &format!("{dir}/pp"),
    );
    let order = |cmd: &str| {
        fx.run("let g:rt_order = []");
        fx.run(cmd);
        fx.eval("g:rt_order")
    };
    let rows = [
        ("runtime plugin/*.vim", order("runtime plugin/*.vim")),
        ("runtime! plugin/*.vim", order("runtime! plugin/*.vim")),
        (
            "runtime! START plugin/*.vim",
            order("runtime! START plugin/*.vim"),
        ),
        (
            "runtime! OPT plugin/*.vim",
            order("runtime! OPT plugin/*.vim"),
        ),
        (
            "runtime! PACK plugin/*.vim",
            order("runtime! PACK plugin/*.vim"),
        ),
        (
            "runtime! ALL plugin/*.vim",
            order("runtime! ALL plugin/*.vim"),
        ),
        (
            "runtime! STA plugin/*.vim",
            order("runtime! STA plugin/*.vim"),
        ),
        (
            "runtime! plugin/a1.vim plugin/b1.vim",
            order("runtime! plugin/a1.vim plugin/b1.vim"),
        ),
        (
            "runtime plugin/nosuch.vim plugin/b1.vim",
            order("runtime plugin/nosuch.vim plugin/b1.vim"),
        ),
    ];
    let got: Vec<String> = rows
        .iter()
        .map(|(cmd, got)| format!("{cmd} => {got}"))
        .collect();
    assert_eq!(
        got,
        [
            "runtime plugin/*.vim => ['a1']",
            "runtime! plugin/*.vim => ['a1', 'b1', 's1', 'sa', 'aa']",
            "runtime! START plugin/*.vim => ['s1']",
            "runtime! OPT plugin/*.vim => ['o1']",
            "runtime! PACK plugin/*.vim => ['s1', 'o1']",
            "runtime! ALL plugin/*.vim => ['a1', 'b1', 's1', 'sa', 'aa', 'o1']",
            "runtime! STA plugin/*.vim => ['s1']",
            "runtime! plugin/a1.vim plugin/b1.vim => ['a1', 'b1']",
            "runtime plugin/nosuch.vim plugin/b1.vim => ['b1']",
        ]
    );

    // `pos_in_rtp` is a byte offset into 'runtimepath', so it moves with
    // the sandbox's path: `<DIR>/a,` is `dir.len() + 3` bytes. Entries
    // 'runtimepath' does not spell sit on the comma before its `after/` tail.
    let b = dir.len() + 3;
    let after = 2 * b;
    let pack = after - 1;
    let positions = format!("[0, {b}, {pack}, {pack}, {after}]");
    table(
        &fx,
        &sandbox,
        &[
            (
                "nvim_get_runtime_file('plugin/*.vim', v:true)",
                "['<DIR>/a/plugin/a1.vim', '<DIR>/b/plugin/b1.vim', '<DIR>/pp/pack/p/start/s/plugin/s1.vim', '<DIR>/pp/pack/p/start/s/after/plugin/sa.vim', '<DIR>/a/after/plugin/aa.vim']",
            ),
            (
                "nvim_get_runtime_file('plugin/*.vim', v:false)",
                "['<DIR>/a/plugin/a1.vim']",
            ),
            (
                "nvim_get_runtime_file('plugin', v:true)",
                "['<DIR>/a/plugin', '<DIR>/b/plugin', '<DIR>/pp/pack/p/start/s/plugin', '<DIR>/pp/pack/p/start/s/after/plugin', '<DIR>/a/after/plugin']",
            ),
            (
                "nvim_get_runtime_file('colors/*', v:true)",
                "['<DIR>/a/colors/rtdark.vim', '<DIR>/a/colors/rtlight.lua', '<DIR>/b/colors/rtdark.vim']",
            ),
            (
                "nvim_get_runtime_file('plugin/**/*.lua', v:true)",
                "['<DIR>/b/plugin/b2.lua', '<DIR>/b/plugin/sub/deep.lua']",
            ),
            ("nvim_get_runtime_file('nosuch/*', v:true)", "[]"),
            (
                "nvim_list_runtime_paths()",
                "['<DIR>/a', '<DIR>/b', '<DIR>/pp/pack/p/start/s', '<DIR>/pp/pack/p/start/s/after', '<DIR>/a/after']",
            ),
            (
                "map(nvim__runtime_inspect(), {_, e -> [e.path, get(e, 'after', v:false), get(e, 'pack_inserted', v:false)]})",
                "[['<DIR>/a', v:false, v:false], ['<DIR>/b', v:false, v:false], ['<DIR>/pp/pack/p/start/s', v:false, v:false], ['<DIR>/pp/pack/p/start/s/after', v:true, v:false], ['<DIR>/a/after', v:true, v:false]]",
            ),
            (
                "map(nvim__runtime_inspect(), {_, e -> e.pos_in_rtp})",
                &positions,
            ),
            (
                "nvim__get_runtime(['plugin/a1.vim', 'plugin/b2.lua'], v:true, {})",
                "['<DIR>/a/plugin/a1.vim', '<DIR>/b/plugin/b2.lua']",
            ),
            (
                "nvim__get_runtime(['plugin/a1.vim', 'plugin/b2.lua'], v:false, {})",
                "['<DIR>/a/plugin/a1.vim']",
            ),
            (
                "nvim__get_runtime(['rtauto.lua'], v:true, {'is_lua': v:true})",
                "[]",
            ),
        ],
    );
    fx.run("let g:rt_order_lua = []");
    fx.run("runtime! plugin/**/*.lua");
    table(
        &fx,
        &sandbox,
        &[("g:rt_order_lua", "['deep']"), ("g:rt_lua", "1")],
    );
    fx.check_error("runtime nosuch/x.vim", "none");
    restore(&fx);
}

#[test]
fn packadd_splices_the_package_into_both_paths() {
    let sandbox = Sandbox::dir("runtime_packadd");
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    runtime_tree(&sandbox);
    let dir = sandbox.as_str();
    own_paths(
        &fx,
        &format!("{dir}/a,{dir}/b,{dir}/a/after"),
        &format!("{dir}/pp"),
    );
    // Validate the cache before the splice, so the splice is what moves it.
    fx.run("call nvim_get_runtime_file('plugin/a1.vim', v:false)");
    fx.run("let g:rt_order = []");
    fx.check_error("packadd o", "none");
    // The splice shifts the byte offsets of everything after it by the two
    // entries it wrote into 'runtimepath', commas included; see the sibling
    // test for why they are computed.
    let b = dir.len() + 3;
    let opt = 2 * b;
    let pack = opt - 1;
    let opt_after = opt + dir.len() + "/pp/pack/p/opt/o,".len();
    let after = opt_after + dir.len() + "/pp/pack/p/opt/o/after,".len();
    let inspected = format!(
        "[['<DIR>/a', v:false, v:false, 0], ['<DIR>/b', v:false, v:false, {b}], \
         ['<DIR>/pp/pack/p/start/s', v:false, v:false, {pack}], \
         ['<DIR>/pp/pack/p/start/s/after', v:true, v:false, {pack}], \
         ['<DIR>/pp/pack/p/opt/o', v:false, v:true, {opt}], \
         ['<DIR>/pp/pack/p/opt/o/after', v:true, v:true, {opt_after}], \
         ['<DIR>/a/after', v:true, v:false, {after}]]"
    );
    table(
        &fx,
        &sandbox,
        &[
            (
                "&rtp",
                "'<DIR>/a,<DIR>/b,<DIR>/pp/pack/p/opt/o,<DIR>/pp/pack/p/opt/o/after,<DIR>/a/after'",
            ),
            ("g:rt_order", "['o1']"),
            (
                "map(nvim__runtime_inspect(), {_, e -> [e.path, get(e, 'after', v:false), get(e, 'pack_inserted', v:false), e.pos_in_rtp]})",
                &inspected,
            ),
            (
                "nvim_get_runtime_file('plugin/*.vim', v:true)",
                "['<DIR>/a/plugin/a1.vim', '<DIR>/b/plugin/b1.vim', '<DIR>/pp/pack/p/start/s/plugin/s1.vim', '<DIR>/pp/pack/p/start/s/after/plugin/sa.vim', '<DIR>/pp/pack/p/opt/o/plugin/o1.vim', '<DIR>/pp/pack/p/opt/o/after/plugin/oa.vim', '<DIR>/a/after/plugin/aa.vim']",
            ),
        ],
    );
    fx.run("let g:rt_order = []");
    fx.check_error("packadd! o", "none");
    table(&fx, &sandbox, &[("g:rt_order", "[]")]);
    fx.check_error(
        "packadd nosuch",
        "E919: Directory not found in ''packpath'': \"pack/*/opt/nosuch\"",
    );
    fx.run("let g:rt_order = []");
    fx.check_error("packloadall!", "none");
    table(
        &fx,
        &sandbox,
        &[
            ("g:rt_order", "['s1']"),
            (
                "&rtp",
                "'<DIR>/a,<DIR>/b,<DIR>/pp/pack/p/opt/o,<DIR>/pp/pack/*/start/*,<DIR>/pp/pack/*/start/*/after,<DIR>/pp/pack/p/opt/o/after,<DIR>/a/after'",
            ),
        ],
    );
    restore(&fx);
}

#[test]
fn completion_offers_bare_names() {
    let sandbox = Sandbox::dir("runtime_complete");
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    runtime_tree(&sandbox);
    let dir = sandbox.as_str();
    own_paths(
        &fx,
        &format!("{dir}/a,{dir}/b,{dir}/a/after"),
        &format!("{dir}/pp"),
    );
    table(
        &fx,
        &sandbox,
        &[
            ("getcompletion('rt', 'color')", "['rtdark', 'rtlight']"),
            ("getcompletion('', 'packadd')", "['o']"),
            ("getcompletion('pl', 'runtime')", "['plugin/']"),
            (
                "getcompletion('plugin/', 'runtime')",
                "['plugin/a1.vim', 'plugin/aa.vim', 'plugin/b1.vim', 'plugin/b2.lua', 'plugin/sub/']",
            ),
            (
                "getcompletion('', 'runtime')",
                "['after/', 'autoload/', 'colors/', 'plugin/', 'START', 'OPT', 'PACK', 'ALL']",
            ),
            ("getcompletion('S', 'runtime')", "['START']"),
            ("getcompletion('START pl', 'cmdline')", "[]"),
            (
                "getcompletion('runtime START plugin/', 'cmdline')",
                "['plugin/s1.vim']",
            ),
            (
                "getcompletion('runtime OPT plugin/', 'cmdline')",
                "['plugin/o1.vim']",
            ),
            (
                "getcompletion('runtime plugin/a1.vim pl', 'cmdline')",
                "['plugin/']",
            ),
            ("getcompletion('runtime A', 'cmdline')", "['ALL']"),
        ],
    );
    restore(&fx);
}

#[test]
fn an_autoload_script_is_sourced_once() {
    let sandbox = Sandbox::dir("runtime_autoload");
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    runtime_tree(&sandbox);
    let dir = sandbox.as_str();
    own_paths(
        &fx,
        &format!("{dir}/a,{dir}/b,{dir}/a/after"),
        &format!("{dir}/pp"),
    );
    table(
        &fx,
        &sandbox,
        &[
            ("rtauto#f()", "'f'"),
            ("rtauto#f()", "'f'"),
            ("g:rt_auto_loads", "1"),
            ("exists('*rtauto#g')", "0"),
            ("g:rt_auto_loads", "1"),
            ("exists('*rtnone#g')", "0"),
        ],
    );
    restore(&fx);
}

#[test]
fn the_stack_names_scripts_and_functions() {
    let sandbox = Sandbox::dir("runtime_estack");
    let fx = Probe::new(sandbox.editor());
    restore(&fx);
    sandbox.write(
        "inner.vim",
        b"function! RtInner() abort\n  let g:rt_trace = getstacktrace()\n  let g:rt_sfile = expand('<sfile>')\n  let g:rt_stack = expand('<stack>')\n  let g:rt_script = expand('<script>')\nendfunction\n",
    );
    sandbox.write(
        "outer.vim",
        b"source <sfile>:h/inner.vim\nlet g:rt_outer_sfile = expand('<sfile>')\naugroup rt_estack\n  autocmd User RtEvent call RtInner()\naugroup END\ndoautocmd User RtEvent\nlet g:rt_slnum = expand('<slnum>')\n",
    );
    fx.check_error("source outer.vim", "none");
    table(
        &fx,
        &sandbox,
        &[
            ("g:rt_outer_sfile", "'<DIR>/outer.vim'"),
            (
                "g:rt_sfile",
                "'<DIR>/outer.vim[6]..User Autocommands for \"RtEvent\"..function RtInner'",
            ),
            (
                "substitute(g:rt_stack, '\\d\\+_', 'N_', 'g')",
                "'<DIR>/outer.vim[6]..User Autocommands for \"RtEvent\"..function RtInner[3]'",
            ),
            ("g:rt_script", "'<DIR>/inner.vim'"),
            ("g:rt_slnum", "'7'"),
            (
                "map(copy(g:rt_trace), {_, f -> [get(f, 'event', ''), f.lnum, f.filepath, has_key(f, 'funcref')]})",
                "[['', 6, '<DIR>/outer.vim', 0], ['User Autocommands for \"RtEvent\"', 4, '<DIR>/outer.vim', 0], ['', 2, '<DIR>/inner.vim', 1]]",
            ),
            ("expand('<sfile>')", "''"),
            ("expand('<stack>')", "''"),
            ("getstacktrace()", "[]"),
        ],
    );
    fx.run("autocmd! rt_estack | augroup! rt_estack");
    fx.run("delfunction RtInner");
    restore(&fx);
}
