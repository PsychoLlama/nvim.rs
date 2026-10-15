//! Tag lookup: `taglist()`, `:tag`, `:tselect` and the tag stack, over tags
//! files of the case's own.
//!
//! `find_tags` answers the match list everything here reads, and
//! `parse_match`/`TagParts` split each line into the fields `taglist()`
//! reports: the name, the file (made relative to the tags file's directory
//! by `expand_tag_fname`), the command, the kind, `static` (`file:`) and
//! every other extension field. Sorted files are binary-searched, unsorted
//! ones scanned; 'tagcase' and 'ignorecase' decide the comparison. `:tag`
//! jumps by the command and pushes a `Taggy` onto the window's stack.
//!
//! Written against the behaviour before the match list stops being a
//! `char **`, `TagParts` becomes offsets and the stack entry owns its
//! strings; a row that moves is a behaviour change.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};

/// A sorted tags file, ctags' extended format: a search command, a line
/// number command, a static tag, an extension field, a kind given as a
/// `kind:` field, a name that only differs in case, and two tags of one name
/// in two files.
const SORTED: &str = "!_TAG_FILE_FORMAT\t2\t/extended format/\n\
!_TAG_FILE_SORTED\t1\t/0=unsorted, 1=sorted, 2=foldcase/\n\
Alpha\tsrc/a.c\t/^int Alpha(void)$/;\"\tf\n\
alpha\tsrc/a.c\t/^int alpha;$/;\"\tv\n\
beta\tsrc/b.c\t3;\"\tf\tsignature:(int x)\n\
dup\tsrc/a.c\t/^dup a$/;\"\tf\n\
dup\tsrc/b.c\t/^dup b$/;\"\tf\n\
gamma\tsrc/b.c\t/^static int gamma/;\"\tkind:function\tfile:\n";

const A_C: &str = "int Alpha(void)\nint alpha;\ndup a\n";
const B_C: &str = "line one\nline two\nint beta(int x)\ndup b\nstatic int gamma\n";

fn sources(name: &str) -> Sandbox {
    let sandbox = Sandbox::dir(name);
    sandbox.mkdir("src");
    sandbox.write("src/a.c", A_C.as_bytes());
    sandbox.write("src/b.c", B_C.as_bytes());
    sandbox.write("tags", SORTED.as_bytes());
    sandbox
}

/// `taglist(pat)` as `[name, filename, cmd, kind, static, extra fields]`
/// rows: the dictionary's own `string()` would also carry the absolute
/// path of the file a tag came from.
fn rows(pat: &str) -> String {
    format!(
        "map(taglist('{pat}'), {{_, t -> [t.name, t.filename, t.cmd, t.kind, t.static] + \
         map(sort(filter(keys(t), {{_, k -> index(['name', 'filename', 'cmd', 'kind', 'static'], k) < 0}})), \
         {{_, k -> k .. '=' .. t[k]}})}})"
    )
}

fn restore(fx: &Probe) {
    fx.home();
    fx.run("set tags& tagcase& ignorecase& smartcase& tagbsearch&");
    fx.run("call settagstack(winnr(), {'items': []}, 'r')");
}

#[test]
fn a_sorted_file_answers_every_field_of_every_match() {
    let sandbox = sources("tag_fields");
    let fx = Probe::new(sandbox.editor());
    fx.run("set tags=./tags");
    fx.expect_all(&[
        (
            &rows("^alpha$"),
            "[['alpha', 'src/a.c', '/^int alpha;$/', 'v', 0]]",
        ),
        // A line-number command, and an extension field.
        (
            &rows("^beta$"),
            "[['beta', 'src/b.c', '3', 'f', 0, 'signature=(int x)']]",
        ),
        (
            &rows("^dup$"),
            "[['dup', 'src/a.c', '/^dup a$/', 'f', 0], ['dup', 'src/b.c', '/^dup b$/', 'f', 0]]",
        ),
        // `kind:` spelled as a field, and `file:` making it static.
        (
            &rows("^gamma$"),
            "[['gamma', 'src/b.c', '/^static int gamma/', 'function', 1]]",
        ),
        (&rows("^nothing$"), "[]"),
        ("tagfiles()", "['tags']"),
    ]);
    restore(&fx);
}

#[test]
fn patterns_case_and_the_unsorted_scan() {
    let sandbox = sources("tag_patterns");
    let fx = Probe::new(sandbox.editor());
    fx.run("set tags=./tags");
    let names = |pat: &str| format!("map(taglist('{pat}'), 'v:val.name')");
    fx.expect_all(&[
        (&names("a"), "['Alpha', 'alpha', 'beta', 'gamma']"),
        (&names("^a"), "['alpha']"),
        (&names("^A"), "['Alpha']"),
    ]);
    // An exact match sorts ahead of the case-folded one.
    fx.run("set tagcase=ignore");
    fx.check(&names("^alpha$"), "['alpha', 'Alpha']");
    // 'tagcase' "match" wins over 'ignorecase'.
    fx.run("set tagcase=match ignorecase");
    fx.check(&names("^alpha$"), "['alpha']");
    fx.run("set tagcase=smart smartcase");
    fx.check(&names("^Alpha$"), "['Alpha']");
    fx.run("set tagcase& noignorecase nosmartcase");
    // The tag lines reversed under a header that says so: the linear scan
    // finds what the binary search did, in file order.
    let (header, lines): (Vec<&str>, Vec<&str>) = SORTED.lines().partition(|l| l.starts_with('!'));
    let unsorted: String = header
        .iter()
        .map(|line| line.replace("_SORTED\t1", "_SORTED\t0"))
        .chain(lines.iter().rev().map(|line| (*line).to_owned()))
        .map(|line| line + "\n")
        .collect();
    sandbox.write("tags", unsorted.as_bytes());
    fx.check(&names("^dup$"), "['dup', 'dup']");
    fx.check(&names("^alpha$"), "['alpha']");
    // Reversed with no header, the file claims nothing and is binary
    // searched as if sorted: E432, and whether a name is found depends on
    // where the bisection happens to land.
    let headless: String = lines.iter().rev().map(|line| format!("{line}\n")).collect();
    sandbox.write("tags", headless.as_bytes());
    fx.check(&names("^dup$"), "[]");
    fx.check(&names("^alpha$"), "['alpha']");
    restore(&fx);
}

#[test]
fn file_names_are_relative_to_the_tags_file() {
    let sandbox = sources("tag_relative");
    let fx = Probe::new(sandbox.editor());
    sandbox.mkdir("sub");
    sandbox.write("sub/tags", b"delta\t../src/b.c\t/^line one$/;\"\tf\n");
    fx.run("set tags=./tags,sub/tags");
    fx.check("map(taglist('^delta$'), 'v:val.filename')", "['src/b.c']");
    fx.check("tagfiles()", "['tags', 'sub/tags']");
    restore(&fx);
}

#[test]
fn jumping_follows_the_command_and_pushes_the_stack() {
    let sandbox = sources("tag_jump");
    let fx = Probe::new(sandbox.editor());
    fx.run("set tags=tags");
    fx.check_error("tag beta", "none");
    fx.check("[bufname(), line('.'), col('.')]", "['src/b.c', 3, 1]");
    fx.check_error("tag gamma", "none");
    fx.check("[bufname(), line('.')]", "['src/b.c', 5]");
    fx.check(
        "map(gettagstack().items, {_, i -> [i.tagname, i.matchnr, i.from[1]]})",
        "[['beta', 1, 1], ['gamma', 1, 3]]",
    );
    fx.check("gettagstack().curidx", "3");
    fx.check_error("pop", "none");
    fx.check("[bufname(), line('.')]", "['src/b.c', 3]");
    fx.check_error("tag nothing", "E426: Tag not found: nothing");
    // `./tags` is the current *file's* directory: from `src/b.c` it names
    // `src/tags`, which does not exist.
    fx.run("set tags=./tags");
    fx.check_error("tag beta", "E426: Tag not found: beta");
    restore(&fx);
}

#[test]
fn several_matches_are_listed_and_selected() {
    let sandbox = sources("tag_select");
    let fx = Probe::new(sandbox.editor());
    fx.run("set tags=tags");
    fx.check_error("tag dup", "none");
    fx.check("[bufname(), line('.')]", "['src/a.c', 3]");
    fx.check_error("tnext", "none");
    fx.check("[bufname(), line('.')]", "['src/b.c', 4]");
    fx.check_error("tnext", "E428: Cannot go beyond last matching tag");
    fx.check_output(
        "tags",
        "\n  # TO tag         FROM line  in file/text\n  1  2 dup                 1  src/a.c\n>",
    );
    fx.check_error("tfirst", "none");
    fx.check("[bufname(), line('.')]", "['src/a.c', 3]");
    restore(&fx);
}

/// A sorted file of 3,000 tags with lines of every length, a duplicate
/// every 500 and one line longer than the reader's first buffer: what the
/// bisection finds is what the linear scan finds.
fn large_sorted() -> String {
    let mut text = String::from("!_TAG_FILE_SORTED\t1\t//\n");
    for i in 0..3000 {
        let pad = "x".repeat(i % 37);
        text += &format!("t{i:04}\tf{}.c\t/^{pad}$/;\"\tf\n", i % 7);
        if i % 500 == 250 {
            text += &format!("t{i:04}\tg.c\t/^dup {i}$/;\"\tv\n");
        }
        if i == 1700 {
            text += &format!("t1700x\tlong.c\t/^{}$/;\"\tf\n", "y".repeat(1500));
        }
    }
    text
}

fn bisect_rows(fx: &Probe) {
    let found =
        |pat: &str| format!("map(taglist('{pat}'), {{_, t -> t.name .. ':' .. t.filename}})");
    fx.expect_all(&[
        (&found("^t0000$"), "['t0000:f0.c']"),
        (&found("^t2999$"), "['t2999:f3.c']"),
        (&found("^t1250$"), "['t1250:f4.c', 't1250:g.c']"),
        (&found("^t1700x$"), "['t1700x:long.c']"),
        (&found("^t1701$"), "['t1701:f0.c']"),
        (&found("^t1699$"), "['t1699:f5.c']"),
        (&found("^t9999$"), "[]"),
        (&found("^t0$"), "[]"),
        ("len(taglist('^t17'))", "102"),
        ("len(taglist('t0001'))", "1"),
        ("taglist('^t1700x$')[0].cmd[0:4]", "'/^yyy'"),
        ("len(taglist('^t1700x$')[0].cmd)", "1504"),
    ]);
}

#[test]
fn bisection_finds_what_the_scan_finds() {
    let sandbox = Sandbox::dir("tag_bisect");
    sandbox.write("tags", large_sorted().as_bytes());
    let fx = Probe::new(sandbox.editor());
    fx.run("set tags=./tags");
    bisect_rows(&fx);
    fx.run("set notagbsearch");
    bisect_rows(&fx);
    fx.run("set tagbsearch ignorecase");
    // Case folded: the scan, and an upper-case pattern finds them all.
    fx.check("len(taglist('^T17'))", "102");
    fx.check(
        "map(taglist('^T1250$'), 'v:val.filename')",
        "['f4.c', 'g.c']",
    );
    fx.run("set noignorecase");
    // The same file with CR-LF line ends and no newline at the very end.
    let crlf = large_sorted().replace('\n', "\r\n");
    sandbox.write("tags", crlf.trim_end().as_bytes());
    fx.expect_all(&[
        ("map(taglist('^t2999$'), 'v:val.cmd')", "['/^xx$/']"),
        ("map(taglist('^t0001$'), 'v:val.kind')", "['f']"),
        ("len(taglist('^t17'))", "102"),
    ]);
    restore(&fx);
}

/// `!_TAG_FILE_SORTED 2`: sorted with case folded, bisected on the folded
/// name; 'tagcase' still decides whether the case has to match.
#[test]
fn a_foldcase_file_is_bisected_folded() {
    let sandbox = Sandbox::dir("tag_foldcase");
    let mut text = String::from("!_TAG_FILE_SORTED\t2\t/0=unsorted, 1=sorted, 2=foldcase/\n");
    for line in ["ab", "Abc", "ABD", "abe", "Beta", "beta2"] {
        text += &format!("{line}\tf.c\t1;\"\tf\n");
    }
    for i in 0..500 {
        let name = if i % 2 == 0 {
            format!("K{i:04}")
        } else {
            format!("k{i:04}")
        };
        text += &format!("{name}\tk.c\t{i};\"\tv\n");
    }
    sandbox.write("tags", text.as_bytes());
    let fx = Probe::new(sandbox.editor());
    fx.run("set tags=./tags tagcase=match");
    let names = |pat: &str| format!("map(taglist('{pat}'), 'v:val.name')");
    fx.expect_all(&[
        (&names("^ABD$"), "['ABD']"),
        (&names("^abd$"), "[]"),
        (&names("^beta2$"), "['beta2']"),
        (&names("^K0250$"), "['K0250']"),
        (&names("^k0251$"), "['k0251']"),
        (&names("^k0250$"), "[]"),
        ("len(taglist('^K02'))", "50"),
    ]);
    fx.run("set tagcase=ignore");
    fx.expect_all(&[
        (&names("^abd$"), "['ABD']"),
        (&names("^k0250$"), "['K0250']"),
        (&names("^AB"), "['ABD', 'ab', 'Abc', 'abe']"),
        ("len(taglist('^k02'))", "100"),
    ]);
    fx.run("set tagcase=smart");
    fx.check(&names("^ab$"), "['ab']");
    fx.check(&names("^Ab"), "['Abc']");
    restore(&fx);
}

/// An Emacs `TAGS` file is not a format this reads: the first line is a
/// format error, reported with where the reader had got to.
#[test]
fn an_emacs_tags_file_is_a_format_error() {
    let sandbox = sources("tag_emacs");
    sandbox.write(
        "TAGS",
        b"\x0c\nsrc/a.c,24\nint Alpha(void)\x7fAlpha\x011,0\n",
    );
    let fx = Probe::new(sandbox.editor());
    fx.run("set tags=./TAGS");
    fx.check_output(
        "silent! echo taglist('Alpha')",
        "\nE431: Format error in tags file \"TAGS\"\nBefore byte 2\n[]",
    );
    fx.check_error("tag Alpha", "E426: Tag not found: Alpha");
    restore(&fx);
}

/// The order matches come out in: a static tag in the current file first,
/// then global ones, then static ones elsewhere — what `:ltag` lists and
/// `:tselect` numbers.
#[test]
fn matches_are_ordered_by_priority() {
    let sandbox = sources("tag_priority");
    sandbox.write(
        "tags",
        b"!_TAG_FILE_SORTED\t1\t//\n\
same\tsrc/a.c\t/^int Alpha(void)$/;\"\tf\tfile:\n\
same\tsrc/b.c\t/^line one$/;\"\tf\n\
same\tsrc/b.c\t/^static int gamma/;\"\tf\tfile:\n\
same\tsrc/b.c\t3\n\
same\tsrc/c.c\t/^a $ and \\/ a\\\\b/\n",
    );
    let fx = Probe::new(sandbox.editor());
    fx.run("set tags=tags");
    fx.run("badd src/b.c");
    fx.run("buffer src/b.c");
    fx.run("ltag same");
    fx.check(
        "map(getloclist(0), {_, e -> [bufname(e.bufnr), e.lnum, e.pattern, e.text]})",
        "[['src/b.c', 0, '^\\Vstatic int gamma', 'same'], ['src/b.c', 0, '^\\Vline one\\$', 'same'], \
         ['src/b.c', 3, '', 'same'], ['src/c.c', 0, '^\\Va $ and \\/ a\\\\b', 'same'], \
         ['src/a.c', 0, '^\\Vint Alpha(void)\\$', 'same']]",
    );
    fx.check("getloclist(0, {'title': 0}).title", "'ltag same'");
    fx.run("lclose");
    fx.run("call setloclist(0, [], 'f')");
    fx.run("buffer src/b.c");
    fx.run("call feedkeys(\"\\<CR>\", 't')");
    fx.check_output(
        "tselect same",
        "\n  # pri kind tag               file\n  1 FSC f    same              src/b.c\n               static int gamma\n  \
         2 F C f    same              src/b.c\n               line one\n  3 F C      same              src/b.c\n               3\n  \
         4 F        same              src/c.c\n               a $ and / a\\b\n  5 FS  f    same              src/a.c\n               int Alpha(void)\n",
    );
    restore(&fx);
}

/// `'tagfunc'` answers instead of the files: its extra fields reach
/// `taglist()`, and its `user_data` rides on the stack entry.
#[test]
fn tagfunc_answers_and_its_user_data_is_kept() {
    let sandbox = sources("tag_tagfunc");
    let fx = Probe::new(sandbox.editor());
    fx.run("set tags=tags");
    fx.run(
        "let &tagfunc = \"{p, f, i -> [{'name': p, 'filename': 'src/a.c', 'cmd': '2', \
         'kind': 'v', 'user_data': 'ud-' .. p, 'extra': 'e'}]}\"",
    );
    fx.check(
        &rows("thing"),
        "[['thing', 'src/a.c', '2', 'v', 0, 'extra=e', 'user_data=ud-thing']]",
    );
    fx.check_error("tag thing", "none");
    fx.check("[bufname(), line('.')]", "['src/a.c', 2]");
    fx.check(
        "map(gettagstack().items, {_, i -> [i.tagname, i.user_data]})",
        "[['thing', 'ud-thing']]",
    );
    // A null dict in the answer is malformed, like any non-dict: E987,
    // where reading it used to dereference null.
    fx.run("let &tagfunc = \"{p, f, i -> [v:_null_dict]}\"");
    fx.check_error(
        "call taglist('thing')",
        "E987: Invalid return value from tagfunc",
    );
    fx.check("taglist('thing')", "[]");
    fx.run("set tagfunc=");
    fx.run("setlocal tagfunc=");
    restore(&fx);
}

/// The stack keeps twenty entries, dropping the oldest; a new window
/// starts with a copy, and `settagstack()` replaces, appends and truncates.
#[test]
fn the_stack_holds_twenty_and_is_copied_on_split() {
    let sandbox = Sandbox::dir("tag_stack");
    sandbox.write("tags", large_sorted().as_bytes());
    for i in 0..7 {
        sandbox.write(&format!("f{i}.c"), b"one\ntwo\n");
    }
    let fx = Probe::new(sandbox.editor());
    fx.run("set tags=tags");
    for i in 1..=22 {
        fx.run(&format!("silent! tag t{i:04}"));
    }
    let names = "map(gettagstack().items, 'v:val.tagname')";
    fx.check("gettagstack().length", "20");
    fx.check(&format!("{names}[0:1]"), "['t0003', 't0004']");
    fx.check(&format!("{names}[-1:]"), "['t0022']");
    fx.run("split");
    fx.check("gettagstack().length", "20");
    fx.check(&format!("{names}[0]"), "'t0003'");
    fx.run("silent! pop");
    fx.run("silent! pop");
    fx.check("gettagstack().curidx", "19");
    fx.run("silent! tag t0100");
    fx.check("gettagstack().length", "19");
    fx.check(&format!("{names}[-2:]"), "['t0020', 't0100']");
    fx.run("wincmd p");
    fx.check("gettagstack().length", "20");
    fx.run("call settagstack(winnr(), {'items': [{'tagname': 'x', 'from': [0, 1, 1, 0], 'user_data': 'u'}]}, 'a')");
    fx.check("gettagstack().length", "20");
    fx.check("gettagstack().items[-1].user_data", "'u'");
    fx.check(&format!("{names}[0]"), "'t0004'");
    fx.run("call settagstack(winnr(), {'curidx': 3}, 't')");
    fx.check("gettagstack().length", "2");
    fx.run("only");
    restore(&fx);
}
