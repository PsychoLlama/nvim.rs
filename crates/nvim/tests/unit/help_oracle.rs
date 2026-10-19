//! `:helptags`' output, byte for byte, recorded on the `FILE *` writer that
//! upstream ported: the owned reader and the `Write` sink that replaced it
//! have to reproduce every digest (see `mbyte_oracle.rs` for the rule).
//!
//! Two trees. The repository's own `runtime/doc`, copied into a sandbox and
//! tagged with `++t` (not compared with `runtime/doc/tags`: that file is
//! ignored, and whichever `nvim` the dev shell found wrote it, without
//! `++t`). And a tree of hostile help files:
//! multibyte tags, tags holding `*`/`|`/`\`/`/`, `**` and `***`, CR line
//! ends, examples opened by `>` and `>lang` and closed by a first column,
//! lines longer than the 1024-byte read with tags on both sides of the cut,
//! a NUL inside a line, duplicates within a file and across files (E154),
//! subdirectories, and suffixes in every case (`.txt`, `.TXT`, `.frx`,
//! `.FRX`, `.f1x`, `.abc`). Each tags file is read back as a blob, so a
//! byte that moves moves the digest, and so does every message the run
//! leaves in the history.

#![cfg(not(miri))]

use crate::support::corpus::Digest;
use crate::support::{Probe, Sandbox};
use std::path::Path;

/// Every file of `from`, copied into `to`.
fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a copy directory");
    for entry in std::fs::read_dir(from).expect("the runtime docs") {
        let entry = entry.expect("an entry");
        if entry.file_type().expect("a type").is_file() {
            std::fs::copy(entry.path(), to.join(entry.file_name())).expect("a copy");
        }
    }
}

#[test]
fn helptags_over_runtime_doc() {
    let sandbox = Sandbox::dir("help_oracle_runtime");
    let fx = Probe::new(sandbox.editor());
    let doc = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../runtime/doc");
    let copy = sandbox.path("doc");
    copy_dir(&doc, &copy);
    let _ = std::fs::remove_file(copy.join("tags"));
    fx.check_error(&format!("helptags ++t {}", copy.display()), "none");
    let written = std::fs::read(copy.join("tags")).expect("the written tags file");
    let mut d = Digest::new();
    d.bytes(&written);
    assert_eq!(d.value(), RUNTIME, "runtime/doc tags digest moved");
}

/// The hostile tree: `(path, contents)` under `doc/`.
fn hostile_files() -> Vec<(&'static str, Vec<u8>)> {
    let mut long = vec![b'x'; 1100];
    long[0..7].copy_from_slice(b"*early*");
    long[7] = b' ';
    long[1015..1030].copy_from_slice(b" *straddle-it* ");
    long[1050..1058].copy_from_slice(b" *late* ");
    long.push(b'\n');
    let mut exact = vec![b'y'; 1023];
    exact[0..8].copy_from_slice(b"*exact* ");
    exact.push(b'\n');
    let mut exact_gt = vec![b'z'; 1022];
    exact_gt.extend_from_slice(b" >\n");
    let mut a = Vec::new();
    a.extend_from_slice(
        "*multi-é* *日本語* *émoji-😀* *a*b* *x|y* *tab\ttag* *|* *** ** *back\\slash* *fw/slash* *both\\/x*\n"
            .as_bytes(),
    );
    a.extend_from_slice(b"word*notatag* *ok*trail *ok2*\ttab-after *end*\n");
    a.extend_from_slice(b"*crlf*\r\n*cr-mid*\rmore\n");
    a.extend_from_slice(b"an example >\n   *in-example*\n\t*in-example-tab*\n*after-example*\n");
    a.extend_from_slice(b"lang example >vim\n  *in-lang-example*\nend\n");
    a.extend_from_slice(b"not one x>\n  *not-example*\n");
    a.extend_from_slice(b"upper >VIM\n  *after-upper-lang*\n");
    a.extend_from_slice(b"digits >vim9\n  *in-digit-lang*\n");
    a.extend_from_slice(b">\n  *after-bare-gt*\n");
    a.extend_from_slice(&long);
    a.extend_from_slice(b"*after-long*\n");
    a.extend_from_slice(&exact);
    a.extend_from_slice(b"*after-exact*\n");
    a.extend_from_slice(&exact_gt);
    a.extend_from_slice(b"  *after-exact-gt*\n");
    a.extend_from_slice(b"*pre-nul* \0 *post-nul*\n*dupe* *dupe*\n*%s-pct* *%d*\n");
    a.extend_from_slice(b"*no-newline-at-eof*");
    vec![
        ("doc/a.txt", a),
        ("doc/b.txt", b"*ok* again\n*end*\n".to_vec()),
        ("doc/sub/deep.txt", b"*deep* *ok2*\n".to_vec()),
        ("doc/UPPER.TXT", b"*upper-txt*\n".to_vec()),
        (
            "doc/c.frx",
            "*ok* *français*\n*c-only*\n".as_bytes().to_vec(),
        ),
        ("doc/d.FRX", b"*d-upper-frx*\n".to_vec()),
        ("doc/e.f1x", b"*e-digit*\n".to_vec()),
        ("doc/f.abc", b"*f-abc*\n".to_vec()),
        ("doc/g.jax", "*ja-tag* *日本語*\n".as_bytes().to_vec()),
    ]
}

#[test]
fn helptags_over_hostile_files() {
    let sandbox = Sandbox::dir("help_oracle_hostile");
    let fx = Probe::new(sandbox.editor());
    sandbox.mkdir("doc/sub");
    for (path, contents) in hostile_files() {
        sandbox.write(path, &contents);
    }
    let root = sandbox.as_str().to_string();
    let mut d = Digest::new();
    for cmd in [
        "helptags doc",
        "helptags ++t doc",
        &format!("helptags {root}/doc/"),
    ] {
        fx.run("messages clear");
        fx.run(&format!("silent {cmd}"));
        let messages = fx.output("messages").replace(&root, "<root>");
        d.bytes(messages.as_bytes());
        let listing = fx.eval("sort(readdir('doc'))");
        d.bytes(listing.as_bytes());
        for tags in ["tags", "tags-fr", "tags-ja", "tags-ab", "tags-f1"] {
            d.bytes(
                fx.eval(&format!(
                    "filereadable('doc/{tags}') ? readblob('doc/{tags}') : 0z"
                ))
                .as_bytes(),
            );
        }
    }
    fx.run("messages clear");
    assert_eq!(d.value(), HOSTILE, "hostile helptags digest moved");
}

/// `HOSTILE` was recorded on the `FILE *` writer. `RUNTIME` follows the
/// docs: it was re-recorded when the upstream v0.12.5 docs landed (new tags
/// such as `v:startreason` and `:restart!`), with the tag writer unchanged.
const RUNTIME: u64 = 5_486_496_003_043_726_054;
const HOSTILE: u64 = 14_626_439_989_836_119_607;
