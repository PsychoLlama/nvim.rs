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

fn mode(sandbox: &Sandbox, name: &str) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(sandbox.path(name))
        .expect(name)
        .permissions()
        .mode()
        & 0o7777
}

fn is_symlink(sandbox: &Sandbox, name: &str) -> bool {
    std::fs::symlink_metadata(sandbox.path(name)).is_ok_and(|m| m.file_type().is_symlink())
}

/// 'backupcopy' "auto" copies when renaming would break a link: a hard link
/// keeps sharing its inode, a symbolic link stays a link. A plain file is
/// renamed. "breakhardlink"/"breaksymlink" ask for the rename anyway.
#[test]
fn backupcopy_auto_keeps_links_and_break_options_break_them() {
    let sandbox = Sandbox::dir("bufwrite_links");
    sandbox.write("f.txt", b"old\n");
    std::fs::hard_link(sandbox.path("f.txt"), sandbox.path("h.txt")).expect("hard link");
    sandbox.write("t.txt", b"target\n");
    std::os::unix::fs::symlink("t.txt", sandbox.path("s.txt")).expect("symlink");
    sandbox.write("p.txt", b"plain\n");
    let fx = Probe::new(sandbox.editor());
    fx.run("set backupskip= backupdir=. writebackup backup backupcopy=auto");

    fx.run("edit! h.txt");
    fx.run("call setline(1, 'via hard link')");
    let shared = ino(&sandbox, "f.txt");
    fx.check_error("write", "none");
    assert_eq!(ino(&sandbox, "h.txt"), shared, "auto copies a hard link");
    assert_eq!(bytes(&sandbox, "f.txt"), b"via hard link\n");
    assert_eq!(bytes(&sandbox, "h.txt~"), b"old\n");

    fx.run("edit! s.txt");
    fx.run("call setline(1, 'via symlink')");
    fx.check_error("write", "none");
    assert!(is_symlink(&sandbox, "s.txt"), "auto keeps the symlink");
    assert_eq!(bytes(&sandbox, "t.txt"), b"via symlink\n");
    assert_eq!(bytes(&sandbox, "s.txt~"), b"target\n");

    fx.run("edit! p.txt");
    fx.run("call setline(1, 'plain new')");
    let before = ino(&sandbox, "p.txt");
    fx.check_error("write", "none");
    assert_eq!(ino(&sandbox, "p.txt~"), before, "auto renames a plain file");
    assert_eq!(bytes(&sandbox, "p.txt"), b"plain new\n");

    fx.run("set backupcopy=auto,breakhardlink");
    fx.run("edit! h.txt");
    fx.run("call setline(1, 'broken hard link')");
    fx.check_error("write", "none");
    assert_ne!(ino(&sandbox, "h.txt"), ino(&sandbox, "f.txt"));
    assert_eq!(bytes(&sandbox, "f.txt"), b"via hard link\n");
    assert_eq!(bytes(&sandbox, "h.txt"), b"broken hard link\n");

    fx.run("set backupcopy=auto,breaksymlink");
    fx.run("edit! s.txt");
    fx.run("call setline(1, 'broken symlink')");
    fx.check_error("write", "none");
    assert!(!is_symlink(&sandbox, "s.txt"), "breaksymlink replaces it");
    assert_eq!(bytes(&sandbox, "t.txt"), b"via symlink\n");
    assert_eq!(bytes(&sandbox, "s.txt"), b"broken symlink\n");
    restore(&fx);
}

/// 'patchmode' on a file that did not exist: an empty original is made.
#[test]
fn patchmode_on_a_new_file_touches_an_empty_original() {
    let sandbox = Sandbox::dir("bufwrite_patchmode_new");
    let fx = Probe::new(sandbox.editor());
    fx.run("set backupskip= backupdir=. writebackup nobackup patchmode=.orig");
    fx.run("new");
    fx.run("only!");
    fx.run("call setline(1, 'fresh')");
    fx.check_error("write n.txt", "none");
    assert_eq!(bytes(&sandbox, "n.txt"), b"fresh\n");
    assert_eq!(bytes(&sandbox, "n.txt.orig"), b"");
    fx.run("bwipe!");
    restore(&fx);
}

/// `++enc=` on the write itself: a character the encoding cannot hold is a
/// conversion error, reported in the message, for an internal conversion
/// (latin1: written as `?`-like 0xbf) and for iconv (the write fails).
#[test]
fn a_forced_encoding_with_a_bad_character() {
    let sandbox = Sandbox::dir("bufwrite_badenc");
    sandbox.write("f.txt", b"old\n");
    let fx = Probe::new(sandbox.editor());
    open(&fx);
    fx.run("call setline(1, ['caf\u{e9}', 'x\u{4e00}y'])");
    fx.check_output(
        "write ++enc=latin1 l.txt",
        "\n\"l.txt\"  CONVERSION ERROR in line 2;[New] 2L, 12B written",
    );
    fx.check("readblob(\'l.txt\')", "0z636166E9.0A78BF79.0A");
    fx.check_error(
        "write ++enc=cp1252 w.txt",
        "E513: Write error, conversion failed (make ''fenc'' empty to override)",
    );
    fx.check("readblob(\'w.txt\')", "0z");
    fx.run("set nowritebackup");
    fx.check_error(
        "write ++enc=cp1252 v.txt",
        "E513: Write error, conversion failed (make ''fenc'' empty to override)",
    );
    fx.check("readblob(\'v.txt\')", "0z");
    fx.check_output(
        "write! ++enc=ucs-2 u.txt",
        "\n\"u.txt\" [converted][New] 2L, 12B written",
    );
    fx.check(
        "readblob(\'u.txt\')",
        "0z00630061.006600E9.000A0078.4E000079.000A",
    );
    restore(&fx);
}

/// A read-only file: refused without `!`, written with it and left
/// read-only; 'cpoptions' "W" refuses even with `!`.
#[test]
fn writing_a_read_only_file() {
    use std::os::unix::fs::PermissionsExt;
    let sandbox = Sandbox::dir("bufwrite_readonly");
    sandbox.write("f.txt", b"old\n");
    std::fs::set_permissions(
        sandbox.path("f.txt"),
        std::fs::Permissions::from_mode(0o444),
    )
    .expect("chmod");
    let fx = Probe::new(sandbox.editor());
    open(&fx);
    fx.check_error(
        "write",
        "E45: ''readonly'' option is set (add ! to override)",
    );
    fx.run("set noreadonly");
    fx.check_error("write", "E505: \"f.txt\" is read-only (add ! to override)");
    assert_eq!(bytes(&sandbox, "f.txt"), b"old\n");
    fx.run("set cpoptions+=W");
    fx.check_error(
        "write!",
        "E504: \"f.txt\" is read-only (cannot override: \"W\" in ''cpoptions'')",
    );
    assert_eq!(bytes(&sandbox, "f.txt"), b"old\n");
    fx.run("set cpoptions-=W");
    fx.check_error("write!", "none");
    assert_eq!(bytes(&sandbox, "f.txt"), b"new\n");
    assert_eq!(mode(&sandbox, "f.txt"), 0o444, "the 'w' bit comes back off");
    fx.check("&readonly", "0");
    fx.run("set cpoptions&");
    restore(&fx);
}

/// A `BufWritePre` that renames the buffer, and one that wipes it: the
/// write follows the rename (the names are the buffer's own), and a wiped
/// buffer is E203.
#[test]
fn a_write_autocommand_renames_or_wipes_the_buffer() {
    let sandbox = Sandbox::dir("bufwrite_au_rename");
    sandbox.write("f.txt", b"old\n");
    let fx = Probe::new(sandbox.editor());
    open(&fx);
    fx.run("autocmd BufWritePre f.txt ++once file renamed.txt");
    fx.check_error("write", "none");
    fx.check("readblob(\'f.txt\')", "0z6F6C640A");
    fx.check("readblob(\'renamed.txt\')", "0z6E65770A");
    fx.check("bufname('%')", "'renamed.txt'");
    fx.check("&modified", "0");
    fx.run("call setline(1, 'again')");
    fx.run("autocmd FileWritePre * ++once file other.txt");
    fx.check_error("1write part.txt", "none");
    fx.check("readblob(\'part.txt\')", "0z61676169.6E0A");
    fx.check("bufname('%')", "'renamed.txt'");
    fx.run("new");
    fx.run("call setline(1, 'doomed')");
    fx.run("file doomed.txt");
    fx.run("autocmd BufWritePre doomed.txt ++once bwipe! doomed.txt");
    fx.check_error(
        "write",
        "E203: Autocommands deleted or unloaded buffer to be written",
    );
    fx.check("readblob(\'doomed.txt\')", "0z");
    restore(&fx);
}

/// The message a write ends with: the notes between the name and the
/// counts, and "written"/"appended" or their 'shortmess' forms.
#[test]
fn the_write_report() {
    let sandbox = Sandbox::dir("bufwrite_report");
    sandbox.write("f.txt", b"old\n");
    let fx = Probe::new(sandbox.editor());
    open(&fx);
    fx.run("call setline(1, ['one', 'two'])");
    let cases = [
        ("write", "\n\"f.txt\" 2L, 8B written"),
        ("write new.txt", "\n\"new.txt\" [New] 2L, 8B written"),
        ("write >> new.txt", "\n\"new.txt\" 2L, 8B appended"),
        (
            "set shortmess+=W | write | set shortmess-=W",
            "\n\"f.txt\" 2L, 8B",
        ),
        (
            "set shortmess+=w | write! new.txt | write >> new.txt | set shortmess-=w",
            "\n\"new.txt\" 2L, 8B [w]\n\"new.txt\" 2L, 8B [a]",
        ),
        (
            "set fileformat=dos | write | set fileformat=unix",
            "\n\"f.txt\" [dos] 2L, 10B written",
        ),
        (
            "set fileformat=mac | write | set fileformat=unix",
            "\n\"f.txt\" [mac] 2L, 8B written",
        ),
        (
            "set nofixeol noeol | write | set fixeol eol",
            "\n\"f.txt\" [noeol] 2L, 7B written",
        ),
        (
            "write ++enc=latin1",
            "\n\"f.txt\" [converted] 2L, 8B written",
        ),
        (
            "write ++enc=utf-16le",
            "\n\"f.txt\" [converted] 2L, 8B written",
        ),
        ("write ++bin", "\n\"f.txt\" 2L, 8B written"),
    ];
    for (cmd, want) in cases {
        fx.check_output(cmd, want);
    }
    restore(&fx);
}
