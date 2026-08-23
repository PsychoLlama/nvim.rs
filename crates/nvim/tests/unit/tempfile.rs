//! Nvim's private temporary directory: `fileio/tempfile.rs`.
//!
//! One directory per process, made on first use, holding a shared `flock` so
//! a `/tmp` cleaner cannot pull it out from under a running editor. Swap
//! files, `:diff`, `:!` filters and `writefile()`'s fsync dance all put
//! things in it, so "is it there, is it ours, and is it private" is a real
//! contract.
//!
//! Ported from `test/unit/tempfile_spec.lua`. Everything here is
//! process-global — the directory, its open handle and the file counter — so
//! every case holds the editor lock and puts the directory back the way it
//! found it.

#![cfg(not(miri))]

use std::path::{Path, PathBuf};

use neovim::fileio::{vim_deltempdir, vim_gettempdir, vim_tempname};
use neovim::os::fs::os_file_is_writable;

use crate::support::{Sandbox, cstr, internalize};

/// The temp directory, without its trailing separator.
fn tempdir() -> PathBuf {
    let raw = unsafe { std::ffi::CStr::from_ptr(vim_gettempdir()) }
        .to_str()
        .expect("a temp path is text");
    assert!(raw.ends_with('/'), "{raw:?} should end with a separator");
    PathBuf::from(raw.trim_end_matches('/'))
}

/// A fresh name inside it. The file is not created.
fn tempname() -> String {
    unsafe { internalize(vim_tempname()) }
}

/// The editor lock plus a promise to leave no directory behind: whatever the
/// case did to the tempdir, the next one starts from nothing.
fn fresh() -> Sandbox {
    let sandbox = Sandbox::globals();
    unsafe { vim_deltempdir() };
    sandbox
}

fn is_writable_dir(path: &Path) -> bool {
    let owned = cstr(path.to_str().unwrap());
    // 2 is "a directory we may write into"; 1 is a writable file.
    let kind = unsafe { os_file_is_writable(owned.as_ptr()) };
    kind == 2
}

#[test]
fn the_temp_directory_is_ours_private_and_empty() {
    let _sandbox = fresh();
    let dir = tempdir();

    assert!(is_writable_dir(&dir), "{dir:?} is not a writable directory");
    assert_eq!(
        std::fs::read_dir(&dir).unwrap().count(),
        0,
        "a fresh temp directory holds nothing"
    );

    // 0700, because anyone who can read it can read every swap file in it.
    let mode =
        std::os::unix::fs::PermissionsExt::mode(&std::fs::metadata(&dir).unwrap().permissions());
    assert_eq!(mode & 0o777, 0o700, "{dir:?} is not private");

    // The name is `<root>/nvim.<user>/XXXXXX`, so the parent is shared and
    // the leaf is not.
    let parent = dir.parent().expect("a parent");
    assert!(
        parent
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("nvim."),
        "{dir:?} is not under an `nvim.<user>` root"
    );

    unsafe { vim_deltempdir() };
}

#[test]
fn the_same_directory_comes_back_every_time() {
    let _sandbox = fresh();
    assert_eq!(tempdir(), tempdir());
    unsafe { vim_deltempdir() };
}

#[test]
fn a_temp_name_is_fresh_unused_and_inside_the_directory() {
    let _sandbox = fresh();
    let dir = tempdir();

    let first = tempname();
    let second = tempname();
    assert_ne!(first, second, "each name is a new one");
    for name in [&first, &second] {
        assert!(!Path::new(name).exists(), "{name} should not exist yet");
        assert!(
            Path::new(name).starts_with(&dir),
            "{name} is not inside {dir:?}"
        );
    }

    unsafe { vim_deltempdir() };
}

/// The directory is deleted with everything in it, not just when it is empty
/// — which is the whole point of `delete_recursive` being what removes it.
#[test]
fn deleting_the_directory_takes_its_contents_with_it() {
    let _sandbox = fresh();
    let dir = tempdir();
    std::fs::write(dir.join("a-swapfile"), b"x").unwrap();
    std::fs::create_dir(dir.join("nested")).unwrap();
    std::fs::write(dir.join("nested/deeper"), b"y").unwrap();

    unsafe { vim_deltempdir() };
    assert!(!dir.exists(), "{dir:?} survived deletion");

    // And the next request makes a new one, at a new path.
    let next = tempdir();
    assert_ne!(next, dir);
    assert!(next.exists());
    unsafe { vim_deltempdir() };
}

/// `$TMPDIR` is the first candidate root, so pointing it somewhere puts the
/// whole tree there. The Lua spec never checked where the directory landed.
#[test]
fn tmpdir_decides_where_the_directory_goes() {
    let mut sandbox = Sandbox::dir("tempfile-tmpdir");
    unsafe { vim_deltempdir() };
    let root = sandbox.mkdir("elsewhere");
    sandbox.set_env("TMPDIR", root.to_str().unwrap());

    let dir = tempdir();
    assert!(
        dir.starts_with(&root),
        "{dir:?} is not under the requested {root:?}"
    );

    unsafe { vim_deltempdir() };
}

/// A hostile `umask` does not get to make the directory unusable. The
/// comment upstream names the case — "repl has been reported to use 0177" —
/// and 0177 would take the execute bit off, which is what makes a directory
/// enterable at all.
#[test]
fn a_umask_that_would_strip_the_execute_bit_is_overridden() {
    let _sandbox = fresh();
    // SAFETY: process-wide, and the editor lock is held for the whole case.
    let saved = unsafe { libc::umask(0o177) };
    let dir = tempdir();
    let mode =
        std::os::unix::fs::PermissionsExt::mode(&std::fs::metadata(&dir).unwrap().permissions());
    unsafe { libc::umask(saved) };

    assert_eq!(mode & 0o777, 0o700, "{dir:?} came out of a 0177 umask");
    assert!(is_writable_dir(&dir));
    unsafe { vim_deltempdir() };
}

/// A `/tmp` cleaner that removes the directory anyway is survivable: the
/// next request notices it is gone and makes another.
///
/// The replacement is **not** locked, though. `vim_opentempdir` returns
/// early whenever a handle is already open, and the stale one from the
/// deleted directory is never closed — so the recovery path leaks that
/// handle and leaves the new directory with no `flock` to protect it.
/// Upstream behaviour (`v0.12.4` `fileio.c`), asserted here so that fixing
/// it is a deliberate change rather than a surprise.
#[test]
fn a_directory_that_disappears_is_replaced() {
    let _sandbox = fresh();
    let dir = tempdir();
    std::fs::remove_dir_all(&dir).unwrap();

    let replacement = tempdir();
    assert_ne!(replacement, dir);
    assert!(replacement.exists());
    assert!(is_writable_dir(&replacement));

    // Names are still handed out inside the new directory.
    assert!(Path::new(&tempname()).starts_with(&replacement));

    // The stale handle, observed rather than asserted about from the source:
    // a descriptor still resolves to the deleted directory.
    let stale = std::fs::read_dir("/proc/self/fd")
        .unwrap()
        .filter_map(Result::ok)
        .any(|fd| {
            std::fs::read_link(fd.path()).is_ok_and(|target| {
                target
                    .to_string_lossy()
                    .starts_with(&*dir.to_string_lossy())
            })
        });
    assert!(stale, "expected the handle on {dir:?} to still be open");

    unsafe { vim_deltempdir() };
}
