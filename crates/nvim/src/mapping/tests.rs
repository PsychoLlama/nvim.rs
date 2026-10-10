//! Mappings and abbreviations end to end, through the commands and functions
//! that keep them: definition per mode, lookup (`maparg()`, `mapcheck()`,
//! `hasmapto()`), the listing, deletion and `:mapclear`, `<buffer>` maps
//! shadowing global ones, abbreviations, `maplist()`'s and the listing's
//! order (the order of each hash bucket's list), and `mapset()` putting back
//! what `maparg()` took out.
//!
//! Written against the behaviour the table has before its intrusive lists
//! become owned ones; a row that moves is a behaviour change.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::test_fixture::Fixture;

/// Every mapping and abbreviation gone, global and buffer-local.
fn clean(fx: &Fixture) {
    for cmd in [
        "mapclear",
        "mapclear!",
        "tmapclear",
        "lmapclear",
        "abclear",
        "mapclear <buffer>",
        "mapclear! <buffer>",
        "tmapclear <buffer>",
        "abclear <buffer>",
    ] {
        fx.run(&format!("silent! {cmd}"));
    }
}

#[test]
fn maps_are_defined_looked_up_and_deleted_per_mode() {
    let fx = Fixture::new();
    clean(&fx);
    for cmd in [
        "nmap ,a :echo 1<CR>",
        "vnoremap ,v x",
        "xmap ,x y",
        "smap ,s z",
        "omap ,o w",
        "imap ,i <Esc>",
        "cmap ,c <C-R>",
        "tmap ,t <C-\\><C-N>",
        "lmap ,l q",
        "map! ,b B",
        "map ,m M",
    ] {
        fx.run(cmd);
    }
    fx.expect_all(&[
        ("maparg(',a', 'n')", "':echo 1<CR>'"),
        ("maparg(',a', 'v')", "''"),
        ("maparg(',v', 'v')", "'x'"),
        ("maparg(',v', 'x')", "'x'"),
        ("maparg(',v', 's')", "'x'"),
        ("maparg(',x', 'x')", "'y'"),
        ("maparg(',x', 's')", "''"),
        ("maparg(',s', 's')", "'z'"),
        ("maparg(',o', 'o')", "'w'"),
        ("maparg(',i', 'i')", "'<Esc>'"),
        ("maparg(',c', 'c')", "'<C-R>'"),
        ("maparg(',t', 't')", "'<C-\\><C-N>'"),
        ("maparg(',l', 'l')", "'q'"),
        ("[maparg(',b', 'i'), maparg(',b', 'c'), maparg(',b', 'n')]", "['B', 'B', '']"),
        ("[maparg(',m', 'n'), maparg(',m', 'x'), maparg(',m', 's'), maparg(',m', 'o'), maparg(',m', 'i')]", "['M', 'M', 'M', 'M', '']"),
        ("maparg(',m')", "'M'"),
        ("maparg(',nothing', 'n')", "''"),
        ("mapcheck(',', 'n')", "'M'"),
        ("mapcheck(',a', 'n')", "':echo 1<CR>'"),
        ("mapcheck(',ab', 'n')", "':echo 1<CR>'"),
        ("mapcheck('z', 'n')", "''"),
        ("hasmapto('x', 'v')", "1"),
        ("hasmapto('x', 'n')", "0"),
        ("hasmapto('B', '!')", "0"),
        ("hasmapto('<Esc>', 'i')", "1"),
    ]);
    fx.check_output(
        "nmap",
        "\n\n   ,m            M\nn  ,a            :echo 1<CR>",
    );
    fx.check_output("map", "\n\n   ,m            M\no  ,o            w\ns  ,s            z\nx  ,x            y\nv  ,v          * x\nn  ,a            :echo 1<CR>");
    fx.check_output(
        "map!",
        "\n\n!  ,b            B\nc  ,c            <C-R>\ni  ,i            <Esc>",
    );
    fx.check_output(
        "vmap ,",
        "\n\n   ,m            M\ns  ,s            z\nx  ,x            y\nv  ,v          * x",
    );
    fx.check_output("tmap", "\n\nt  ,t            <C-\\><C-N>");
    fx.check_output("lmap", "\n\nl  ,l            q");

    // Deleting from one mode of a multi-mode map leaves the others.
    fx.run("sunmap ,m");
    fx.run("nunmap ,a");
    fx.run("iunmap ,b");
    fx.expect_all(&[
        (
            "[maparg(',m', 'n'), maparg(',m', 'x'), maparg(',m', 's'), maparg(',m', 'o')]",
            "['M', 'M', '', 'M']",
        ),
        ("maparg(',a', 'n')", "''"),
        ("[maparg(',b', 'i'), maparg(',b', 'c')]", "['', 'B']"),
    ]);
    fx.check_output("map ,m", "\n\nnox,m            M");
    for (cmd, err) in [
        ("nunmap ,a", "E31: No such mapping"),
        ("unmap ,zz", "E31: No such mapping"),
        ("iunmap ,zz", "E31: No such mapping"),
        (
            "nmap <unique> ,m other",
            "E227: Mapping already exists for ,m",
        ),
        (
            "map <buffer> <unique> ,m other",
            "E225: Global mapping already exists for ,m",
        ),
        ("nnoremap", "none"),
        ("nmap ,q", "none"),
        ("map <nosuchmod> ,q x", "none"),
    ] {
        fx.check_error(cmd, err);
    }
    fx.run("mapclear");
    fx.run("mapclear!");
    fx.expect_all(&[
        (
            "[maparg(',m', 'n'), maparg(',i', 'i'), maparg(',c', 'c')]",
            "['', '', '']",
        ),
        (
            "[maparg(',t', 't'), maparg(',l', 'l')]",
            "['<C-\\><C-N>', 'q']",
        ),
    ]);
    clean(&fx);
    fx.expect_all(&[("[maparg(',t', 't'), maparg(',l', 'l')]", "['', '']")]);
}

#[test]
fn maparg_and_maplist_answer_dicts_in_bucket_order() {
    let fx = Fixture::new();
    clean(&fx);
    fx.run("nnoremap <silent> <nowait> <expr> ,d 'rhs'");
    fx.run("nmap <script> ,e <SID>x");
    fx.run("noremap! <special> ,f <Tab>f");
    // Shapes: every key, then the values that are not this run's script id
    // or line number.
    fx.expect_all(&[
        ("sort(keys(maparg(',d', 'n', 0, 1)))", "['abbr', 'buffer', 'expr', 'lhs', 'lhsraw', 'lnum', 'mode', 'mode_bits', 'noremap', 'nowait', 'replace_keycodes', 'rhs', 'script', 'scriptversion', 'sid', 'silent']"),
        ("filter(maparg(',d', 'n', 0, 1), {k -> k !~ '^\\(sid\\|lnum\\|scriptversion\\)$'})", "{'lhs': ',d', 'mode': 'n', 'expr': 1, 'noremap': 1, 'nowait': 1, 'rhs': '''rhs''', 'lhsraw': ',d', 'abbr': 0, 'script': 0, 'replace_keycodes': 0, 'mode_bits': 1, 'silent': 1, 'buffer': 0}"),
        ("filter(maparg(',e', 'n', 0, 1), {k -> k !~ '^\\(sid\\|lnum\\|scriptversion\\)$'})", "{'lhs': ',e', 'mode': 'n', 'expr': 0, 'noremap': 1, 'nowait': 0, 'rhs': '<SID>x', 'lhsraw': ',e', 'abbr': 0, 'script': 1, 'replace_keycodes': 0, 'mode_bits': 1, 'silent': 0, 'buffer': 0}"),
        ("filter(maparg(',f', 'i', 0, 1), {k -> k !~ '^\\(sid\\|lnum\\|scriptversion\\)$'})", "{'lhs': ',f', 'mode': '!', 'expr': 0, 'noremap': 1, 'nowait': 0, 'rhs': '<Tab>f', 'lhsraw': ',f', 'abbr': 0, 'script': 0, 'replace_keycodes': 0, 'mode_bits': 24, 'silent': 0, 'buffer': 0}"),
        ("maparg(',zz', 'n', 0, 1)", "{}"),
        ("maparg(',d', 'n', 0, 0)", "'''rhs'''"),
    ]);
    fx.check_output(
        "nmap ,",
        "\n\nn  ,e          & <SID>x\nn  ,d          * 'rhs'",
    );
    fx.check_output("map! ,f", "\n\n!  ,f          * <Tab>f");

    // One bucket: every lhs starting `,` hashes to the same list, newest
    // first; a different first byte is another bucket.
    clean(&fx);
    for lhs in [",1", ",2", ",3", "a1", ",4", "b1"] {
        fx.run(&format!("nmap {lhs} {lhs}-rhs"));
    }
    let order = "map(maplist(), 'v:val.lhs')";
    fx.expect_all(&[(order, "[',4', ',3', ',2', ',1', 'a1', 'b1']")]);
    fx.check_output("nmap", "\n\nn  ,4            ,4-rhs\nn  ,3            ,3-rhs\nn  ,2            ,2-rhs\nn  ,1            ,1-rhs\nn  a1            a1-rhs\nn  b1            b1-rhs");
    fx.run("nunmap ,2");
    fx.run("nmap ,2 again");
    fx.run("nmap ,1 redefined");
    fx.expect_all(&[
        (order, "[',2', ',4', ',3', ',1', 'a1', 'b1']"),
        (
            "map(maplist(), 'v:val.rhs')",
            "['again', ',4-rhs', ',3-rhs', 'redefined', 'a1-rhs', 'b1-rhs']",
        ),
    ]);
    // Abbreviations are a list of their own and maplist() takes a flag.
    fx.run("iabbrev ab1 one");
    fx.run("iabbrev ab2 two");
    fx.expect_all(&[
        ("map(maplist(1), 'v:val.lhs')", "['ab2', 'ab1']"),
        (
            "map(maplist(0), 'v:val.lhs')",
            "[',2', ',4', ',3', ',1', 'a1', 'b1']",
        ),
    ]);
    clean(&fx);
}

#[test]
fn buffer_maps_shadow_global_ones() {
    let fx = Fixture::new();
    clean(&fx);
    fx.run("nmap ,b global");
    fx.run("nmap <buffer> ,b local");
    fx.run("nmap <buffer> ,c only-local");
    fx.expect_all(&[
        ("maparg(',b', 'n')", "'local'"),
        ("maparg(',b', 'n', 0, 1).buffer", "1"),
        ("maparg(',c', 'n', 0, 1).buffer", "1"),
        (
            "map(maplist(), '[v:val.lhs, v:val.rhs, v:val.buffer]')",
            "[[',b', 'global', 0], [',c', 'only-local', 1], [',b', 'local', 1]]",
        ),
        ("mapcheck(',b', 'n')", "'local'"),
        ("hasmapto('local', 'n')", "1"),
    ]);
    fx.check_output(
        "nmap ,",
        "\n\nn  ,c           @only-local\nn  ,b           @local\nn  ,b            global",
    );
    fx.check_output(
        "nmap <buffer>",
        "\n\nn  ,c           @only-local\nn  ,b           @local",
    );
    fx.check_error(
        "nmap <unique> ,b again",
        "E227: Mapping already exists for ,b",
    );
    fx.check_error(
        "nmap <buffer> <unique> ,b again",
        "E225: Global mapping already exists for ,b",
    );
    fx.check_error("nmap <unique> ,c again", "none");
    fx.run("nunmap <buffer> ,b");
    fx.expect_all(&[("maparg(',b', 'n')", "'global'")]);
    fx.run("nmap <buffer> ,b local2");
    fx.run("mapclear <buffer>");
    fx.expect_all(&[
        (
            "[maparg(',b', 'n'), maparg(',c', 'n')]",
            "['global', 'again']",
        ),
        ("map(maplist(), 'v:val.lhs')", "[',c', ',b']"),
    ]);
    clean(&fx);
}

#[test]
fn abbreviations_are_defined_listed_and_deleted() {
    let fx = Fixture::new();
    clean(&fx);
    fx.run("iabbrev teh the");
    fx.run("cabbrev W w");
    fx.run("abbreviate foo bar");
    fx.run("inoreabbrev <buffer> loc local");
    fx.run("iabbrev <expr> now 'later'");
    fx.expect_all(&[
        ("maparg('teh', 'i', 1)", "'the'"),
        ("maparg('teh', 'c', 1)", "''"),
        ("maparg('W', 'c', 1)", "'w'"),
        (
            "[maparg('foo', 'i', 1), maparg('foo', 'c', 1)]",
            "['bar', 'bar']",
        ),
        ("maparg('loc', 'i', 1, 1).buffer", "1"),
        ("maparg('now', 'i', 1, 1).expr", "1"),
        ("maparg('teh', 'i')", "''"),
        (
            "map(maplist(1), '[v:val.lhs, v:val.mode]')",
            "[['now', 'i'], ['foo', '!'], ['W', 'c'], ['teh', 'i'], ['loc', 'i']]",
        ),
    ]);
    fx.check_output("abbreviate", "\n\ni  loc         *@local\ni  now           'later'\n!  foo           bar\nc  W             w\ni  teh           the");
    fx.check_output("iabbrev", "\n\ni  loc         *@local\ni  now           'later'\n!  foo           bar\ni  teh           the");
    fx.check_output("cabbrev", "\n\n!  foo           bar\nc  W             w");
    for (cmd, err) in [
        ("iunabbrev nosuch", "E24: No such abbreviation"),
        (
            "iabbrev <unique> teh again",
            "E226: Abbreviation already exists for teh",
        ),
        ("iabbrev a!b x", "none"),
        ("iabbrev", "none"),
    ] {
        fx.check_error(cmd, err);
    }
    fx.run("iunabbrev teh");
    fx.run("unabbreviate foo");
    fx.expect_all(&[(
        "[maparg('teh', 'i', 1), maparg('foo', 'i', 1), maparg('foo', 'c', 1), maparg('W', 'c', 1)]",
        "['', '', '', 'w']",
    )]);
    fx.run("iabclear");
    fx.expect_all(&[(
        "[maparg('now', 'i', 1), maparg('W', 'c', 1), maparg('loc', 'i', 1)]",
        "['', 'w', 'local']",
    )]);
    fx.run("abclear <buffer>");
    fx.expect_all(&[("maparg('loc', 'i', 1)", "''")]);
    clean(&fx);
}

#[test]
fn mapset_puts_back_what_maparg_took_out() {
    let fx = Fixture::new();
    clean(&fx);
    fx.run("nnoremap <silent> ,r :echo 'r'<CR>");
    fx.run("let g:mp_saved = maparg(',r', 'n', 0, 1)");
    fx.run("nunmap ,r");
    fx.expect_all(&[("maparg(',r', 'n')", "''")]);
    fx.run("call mapset(g:mp_saved)");
    fx.expect_all(&[
        ("maparg(',r', 'n', 0, 1) == g:mp_saved", "1"),
        ("maparg(',r', 'n')", "':echo ''r''<CR>'"),
    ]);
    fx.run("nunmap ,r");
    fx.run("call mapset('n', 0, g:mp_saved)");
    fx.expect_all(&[("maparg(',r', 'n', 0, 1).silent", "1")]);
    for (cmd, err) in [
        (
            "call mapset({})",
            "E460: Entries missing in mapset() dict argument",
        ),
        (
            "call mapset('n', 0, {})",
            "E460: Entries missing in mapset() dict argument",
        ),
        (
            "call mapset('n', 0, {'lhs': ',x'})",
            "E460: Entries missing in mapset() dict argument",
        ),
        (
            "call mapset('Q', 0, g:mp_saved)",
            "E1276: Illegal map mode string: ''Q",
        ),
    ] {
        fx.check_error(cmd, err);
    }
    fx.run("unlet g:mp_saved");
    clean(&fx);
}

#[test]
fn special_keys_are_kept_and_listed_by_name() {
    let fx = Fixture::new();
    clean(&fx);
    fx.run("nmap <F2> <C-A><Tab>x<Esc>");
    fx.run("nmap <lt>x <Bar><Bslash>");
    fx.run("nmap <M-x> <Cmd>echo 1<CR>");
    fx.run("imap <C-K>e \u{e9}");
    fx.expect_all(&[
        ("maparg('<F2>', 'n')", "'<C-A><Tab>x<Esc>'"),
        ("maparg('<lt>x', 'n')", "'|\\'"),
        ("maparg('<M-x>', 'n')", "'<Cmd>echo 1<CR>'"),
        ("maparg('<C-K>e', 'i')", "'é'"),
        ("maparg('<F2>', 'n', 0, 1).lhsraw", "'�k2'"),
        ("mapcheck('<F2>', 'n')", "'<C-A><Tab>x<Esc>'"),
    ]);
    fx.check_output("nmap", "\n\nn  <x            |\\\nn  <M-x>         <Cmd>echo 1<CR>\nn  <F2>          <C-A><Tab>x<Esc>");
    fx.check_output("imap", "\n\ni  <C-K>e        é");
    clean(&fx);
}
