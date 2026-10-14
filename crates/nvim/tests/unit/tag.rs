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
