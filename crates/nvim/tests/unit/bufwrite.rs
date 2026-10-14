//! `:write`: the backup it makes, and the bytes it puts on disk.
//!
//! `buf_write` decides whether to back the file up and how — a copy that
//! keeps the original's inode ('backupcopy' yes), a rename that gives the
//! new file a fresh one (no), 'backupdir' with its `//` name mangling,
//! 'backupext', 'patchmode''s first-write-only original, a 'writebackup'
//! that is removed after a good write, 'backupskip' — then converts the
//! buffer for 'fileformat', 'fileencoding', 'bomb' and 'fixendofline'.
//!
//! Written against the behaviour before `backup.rs`'s copy loop moves onto
//! `File`s and the conversion onto `Iconv`; a row that moves is a behaviour
//! change. Inodes are compared in Rust: nothing in Vimscript reports one.

#![cfg(not(miri))]

use std::os::unix::fs::MetadataExt;

use crate::support::{Probe, Sandbox};

fn ino(sandbox: &Sandbox, name: &str) -> u64 {
    std::fs::metadata(sandbox.path(name)).expect(name).ino()
}

fn bytes(sandbox: &Sandbox, name: &str) -> Vec<u8> {
    std::fs::read(sandbox.path(name)).unwrap_or_else(|_| b"<missing>".to_vec())
}

/// Edit `f.txt` (which the case wrote as "old\n") and replace its line with
/// "new". 'backupskip' is emptied: its default covers the temp directory
/// every sandbox lives in.
fn open(fx: &Probe) {
    fx.run("set backupskip= backupdir=. writebackup nobackup backupcopy=auto");
    fx.run("edit! f.txt");
    fx.run("call setline(1, 'new')");
}

fn restore(fx: &Probe) {
    fx.home();
    fx.run(
        "set backup& backupcopy& backupdir& backupext& backupskip& writebackup& patchmode& \
         fileformat& fileencoding& bomb& fixendofline& endofline&",
    );
}

#[test]
fn backupcopy_yes_copies_and_keeps_the_inode() {
    let sandbox = Sandbox::dir("bufwrite_copy");
    sandbox.write("f.txt", b"old\n");
    let fx = Probe::new(sandbox.editor());
    open(&fx);
    let before = ino(&sandbox, "f.txt");
    fx.run("set backup backupcopy=yes");
    fx.check_error("write", "none");
    assert_eq!(
        ino(&sandbox, "f.txt"),
        before,
        "the original is overwritten in place"
    );
    assert_eq!(bytes(&sandbox, "f.txt"), b"new\n");
    assert_eq!(bytes(&sandbox, "f.txt~"), b"old\n");
    restore(&fx);
}

#[test]
fn backupcopy_no_renames_and_the_file_is_new() {
    let sandbox = Sandbox::dir("bufwrite_rename");
    sandbox.write("f.txt", b"old\n");
    let fx = Probe::new(sandbox.editor());
    open(&fx);
    let before = ino(&sandbox, "f.txt");
    fx.run("set backup backupcopy=no");
    fx.check_error("write", "none");
    assert_eq!(
        ino(&sandbox, "f.txt~"),
        before,
        "the original became the backup"
    );
    assert_ne!(ino(&sandbox, "f.txt"), before);
    assert_eq!(bytes(&sandbox, "f.txt"), b"new\n");
    assert_eq!(bytes(&sandbox, "f.txt~"), b"old\n");
    restore(&fx);
}

#[test]
fn backupdir_and_backupext_name_the_backup() {
    let sandbox = Sandbox::dir("bufwrite_backupdir");
    sandbox.write("f.txt", b"old\n");
    sandbox.mkdir("bak");
    sandbox.mkdir("full");
    let fx = Probe::new(sandbox.editor());
    open(&fx);
    fx.run("set backup backupdir=bak backupext=.orig");
    fx.check_error("write", "none");
    assert_eq!(bytes(&sandbox, "bak/f.txt.orig"), b"old\n");
    // A trailing `//` keeps the full path in the name, `%` for each `/`.
    fx.run("set backupdir=full// backupext=~");
    fx.run("call setline(1, 'newer')");
    fx.check_error("write", "none");
    let mangled = format!("{}/f.txt~", sandbox.as_str()).replace('/', "%");
    assert_eq!(bytes(&sandbox, &format!("full/{mangled}")), b"new\n");
    // A directory that does not exist yet is made.
    fx.run("set backupdir=missing");
    fx.run("call setline(1, 'newest')");
    fx.check_error("write", "none");
    assert_eq!(bytes(&sandbox, "f.txt"), b"newest\n");
    assert_eq!(bytes(&sandbox, "missing/f.txt~"), b"newer\n");
    // One that cannot be made (a file is in the way): E510, and the write
    // is refused.
    sandbox.write("blocked", b"");
    fx.run("set backupdir=blocked/sub");
    fx.run("call setline(1, 'refused')");
    fx.check_error("write", "E510: Can''t make backup file (add ! to override)");
    assert_eq!(bytes(&sandbox, "f.txt"), b"newest\n");
    restore(&fx);
}

#[test]
fn writebackup_alone_leaves_no_backup_and_backupskip_skips() {
    let sandbox = Sandbox::dir("bufwrite_writebackup");
    sandbox.write("f.txt", b"old\n");
    sandbox.write("g.log", b"old log\n");
    let fx = Probe::new(sandbox.editor());
    open(&fx);
    fx.check_error("write", "none");
    assert_eq!(bytes(&sandbox, "f.txt"), b"new\n");
    assert_eq!(bytes(&sandbox, "f.txt~"), b"<missing>");
    fx.run("set backup backupskip=*.log");
    fx.run("edit! g.log");
    fx.run("call setline(1, 'new log')");
    fx.check_error("write", "none");
    assert_eq!(bytes(&sandbox, "g.log~"), b"<missing>");
    restore(&fx);
}

#[test]
fn patchmode_keeps_the_first_original_only() {
    let sandbox = Sandbox::dir("bufwrite_patchmode");
    sandbox.write("f.txt", b"old\n");
    let fx = Probe::new(sandbox.editor());
    open(&fx);
    fx.run("set patchmode=.first");
    fx.check_error("write", "none");
    fx.run("call setline(1, 'second')");
    fx.check_error("write", "none");
    assert_eq!(bytes(&sandbox, "f.txt.first"), b"old\n");
    assert_eq!(bytes(&sandbox, "f.txt"), b"second\n");
    // 'patchmode' equal to 'backupext' is refused.
    fx.check_error(
        "set patchmode=~",
        "E589: ''backupext'' and ''patchmode'' are equal: patchmode=~",
    );
    restore(&fx);
}

#[test]
fn the_written_bytes_follow_format_encoding_bom_and_eol() {
    let sandbox = Sandbox::dir("bufwrite_bytes");
    sandbox.write("f.txt", b"old\n");
    let fx = Probe::new(sandbox.editor());
    open(&fx);
    fx.run("call setline(1, ['caf\u{e9}', 'two'])");
    let cases: [(&str, &[u8]); 6] = [
        ("fileformat=unix", b"caf\xc3\xa9\ntwo\n"),
        ("fileformat=dos", b"caf\xc3\xa9\r\ntwo\r\n"),
        ("fileformat=mac", b"caf\xc3\xa9\rtwo\r"),
        ("fileformat=unix fileencoding=latin1", b"caf\xe9\ntwo\n"),
        ("fileencoding=utf-8 bomb", b"\xef\xbb\xbfcaf\xc3\xa9\ntwo\n"),
        ("nobomb nofixendofline noendofline", b"caf\xc3\xa9\ntwo"),
    ];
    for (options, want) in cases {
        fx.run(&format!("set {options}"));
        fx.check_error("write", "none");
        assert_eq!(bytes(&sandbox, "f.txt"), want, "{options}");
    }
    restore(&fx);
}

#[test]
fn appending_and_partial_writes() {
    let sandbox = Sandbox::dir("bufwrite_append");
    sandbox.write("f.txt", b"old\n");
    let fx = Probe::new(sandbox.editor());
    open(&fx);
    fx.run("call setline(1, ['one', 'two', 'three'])");
    fx.check_error("2,3write >> f.txt", "none");
    assert_eq!(bytes(&sandbox, "f.txt"), b"old\ntwo\nthree\n");
    fx.check_error("1write other.txt", "none");
    assert_eq!(bytes(&sandbox, "other.txt"), b"one\n");
    // An existing file is not overwritten without `!`.
    fx.check_error("write other.txt", "E13: File exists (add ! to override)");
    fx.check_error("write! other.txt", "none");
    assert_eq!(bytes(&sandbox, "other.txt"), b"one\ntwo\nthree\n");
    restore(&fx);
}
