//! The file system, as `os/fs/` sees it.
//!
//! A port of `test/unit/os/fs_spec.lua`, the largest of the `os/*` specs and
//! the one that owned the most exports. Everything here talks to a real file
//! system: permissions, inodes, hard links, scatter reads and the recursive
//! `mkdir`.
//!
//! Every case holds the editor lock and works inside a directory of its own,
//! both through [`Sandbox`](crate::support::Sandbox). Both are load-bearing
//! here for reasons the LuaJIT harness never had to think about: several
//! cases change the working directory or the umask-like permission bits of a
//! shared fixture, and `cargo test` runs cases on threads of one process
//! rather than in a forked child each. The sandbox's drop is what puts those
//! permission bits back, without which the fixture cannot be removed.

#![cfg(not(miri))]

use std::ffi::{CString, c_int};
use std::io::{Read as _, Seek as _, Write as _};
use std::os::fd::AsRawFd as _;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use neovim::os::fs::FileInfo;
use neovim::os::fs::{
    NODE_NORMAL, NODE_WRITABLE, dir_exists, dirname_within, file_info, link_info, mkdir_recurse,
    open_file, os_can_exe, os_chdir, os_file_is_readable, os_file_is_writable, os_mkdir,
    os_nodetype, os_remove, os_rename, os_rmdir, os_setperm, path_exists,
};
use neovim::os::uv_error::{UV_EEXIST, UV_ENOENT};
use neovim::types::Failed;

use crate::support::Sandbox;

/// `rwx------`, which is what every `mkdir` case here asks for.
const RWX: c_int = 0o700;

/// A [`Sandbox`] with the spec's fixtures in it: the private directory holds
/// two regular files, a link to one of them and a link to nothing.
struct Fixture(Sandbox);

impl Fixture {
    /// The spec's `before_each`, in a directory named after the case.
    fn new(name: &str) -> Fixture {
        let fixture = Fixture(Sandbox::dir(&format!("fs-{name}")));
        fixture.0.mkdir("unit-test-directory");
        fixture.0.touch("unit-test-directory/test.file");
        fixture.0.touch("unit-test-directory/test_2.file");
        std::os::unix::fs::symlink("test.file", fixture.link()).expect("a link to test.file");
        std::os::unix::fs::symlink(
            "non_existing_file.file",
            fixture.path("unit-test-directory/test_broken_link.file"),
        )
        .expect("a link to nothing");
        fixture
    }

    /// The fixture root's absolute, resolved path.
    fn root(&self) -> &Path {
        self.0.root()
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.path(name)
    }

    /// `unit-test-directory/test.file`, the fixture most cases work on.
    fn file(&self) -> PathBuf {
        self.path("unit-test-directory/test.file")
    }

    /// A symbolic link to [`Self::file`].
    fn link(&self) -> PathBuf {
        self.path("unit-test-directory/test_link.file")
    }
}

fn cpath(path: &Path) -> CString {
    CString::new(path.as_os_str().as_encoded_bytes()).expect("a temp path holds no NUL")
}

fn cname(name: &str) -> CString {
    CString::new(name).expect("a fixture name holds no NUL")
}

/// `dir_exists` over a name.
fn isdir(name: &str) -> bool {
    dir_exists(&cname(name))
}

/// `getperm`, which answers the raw mode bits or a libuv error.
fn getperm(name: &str) -> c_int {
    neovim::os::fs::getperm(&cname(name))
}

fn setperm(name: &str, perm: c_int) -> Result<(), c_int> {
    let name = cname(name);
    os_setperm(&name, perm)
}

/// Whether an accessor actually wrote something: the spec's `has_fileinfo`.
fn filled_in(info: &FileInfo) -> bool {
    info.stat.st_ino > 0 && info.stat.st_dev > 0
}

/// `file_info` of a name.
fn info_of(path: &Path) -> Option<FileInfo> {
    file_info(&cpath(path))
}

#[test]
fn the_working_directory_is_reported_and_cannot_be_set_to_a_tilde() {
    let fixture = Fixture::new("cwd");
    let here = fixture.root().to_str().expect("text").to_string();

    let dirname = |len: usize| {
        let (result, text) = match dirname_within(len) {
            Ok(text) => (Ok(()), text),
            Err(text) => (Err(Failed), text),
        };
        (result, String::from_utf8(text).expect("text"))
    };
    assert_eq!(dirname(here.len() + 1), (Ok(()), here.clone()));
    assert_eq!(
        dirname(here.len()).0,
        Err(Failed),
        "the buffer must hold the NUL"
    );

    // `os_chdir` answers 0 for success, not OK. A literal `~` is not a
    // directory and is not expanded here, so both of these fail and neither
    // moves the process.
    assert!(!isdir("~"), "sanity check: no literal ~ directory");
    for name in ["~", "~/"] {
        let name = cname(name);
        assert_ne!(os_chdir(&name), 0, "chdir to a tilde");
    }
    assert_eq!(dirname(here.len() + 1), (Ok(()), here));
}

#[test]
fn only_a_directory_is_a_directory() {
    let fixture = Fixture::new("isdir");
    assert!(!isdir(""), "the empty name");
    assert!(!isdir("non-existing-directory"));
    assert!(!isdir("/non-existing-directory"));
    assert!(!isdir("unit-test-directory/test.file"), "a regular file");
    assert!(isdir("."));
    assert!(isdir(".."));
    assert!(isdir("unit-test-directory"));
    assert!(isdir(fixture.root().to_str().expect("text")), "absolute");
}

/// `os_can_exe` answers whether a name names something runnable, and writes
/// the resolved path only when it does. Both halves matter: a false answer
/// that still wrote a path would leak it.
#[test]
fn an_executable_is_found_by_path_or_relative_to_here() {
    let fixture = Fixture::new("can-exe");

    let can_exe = |name: &str| {
        let name = cname(name);
        let mut resolved = None;
        if os_can_exe(&name, Some(&mut resolved), true) {
            let resolved = resolved.expect("a true answer must set the path");
            Some(resolved.as_cstr().to_string_lossy().into_owned())
        } else {
            assert!(resolved.is_none(), "a false answer must not set one");
            None
        }
    };

    assert_eq!(can_exe("./unit-test-directory"), None, "a directory");
    assert_eq!(can_exe("unit-test-directory/test.file"), None, "no x bit");
    assert_eq!(can_exe("does-not-exist.file"), None);

    let found = can_exe("ls").expect("`ls` is on $PATH");
    assert!(found.starts_with('/'), "{found:?} is absolute");

    // A relative name resolves against the working directory, and gives the
    // same answer as the absolute name it stands for. The test binary is the
    // executable to hand; `current_exe` is already canonical, and the
    // directory is `chdir`ed into so that the relative form is the same file.
    let exe = std::env::current_exe().expect("this binary");
    let dir = exe.parent().expect("a directory").to_path_buf();
    let name = exe
        .file_name()
        .expect("a name")
        .to_string_lossy()
        .into_owned();
    std::env::set_current_dir(&dir).expect("standing beside the binary");
    let absolute = can_exe(
        std::env::current_dir()
            .expect("here")
            .join(&name)
            .to_str()
            .expect("text"),
    );
    let relative = can_exe(&format!("./{name}"));
    std::env::set_current_dir(fixture.root()).expect("back to the fixture");
    assert_eq!(relative, absolute);
    assert!(relative.is_some(), "the test binary is executable");
}

/// Permissions, read back through the three different questions `os/fs`
/// asks about them.
#[test]
fn permission_bits_are_read_back_set_and_answered_about() {
    let _fixture = Fixture::new("perm");
    let file = "unit-test-directory/test.file";

    assert_eq!(getperm("non-existing-file"), UV_ENOENT);
    assert!(getperm("unit-test-directory") > 0);
    assert_ne!(
        getperm("unit-test-directory") & libc::S_IRUSR as c_int,
        0,
        "the fixture directory is readable"
    );
    assert!(setperm("non-existing-file", RWX).is_err());

    // The executable bit goes off and back on.
    let perm = getperm(file);
    assert!(setperm(file, perm & !(libc::S_IXUSR as c_int)).is_ok());
    assert_eq!(getperm(file) & libc::S_IXUSR as c_int, 0);
    assert!(setperm(file, perm | libc::S_IXUSR as c_int).is_ok());
    assert_ne!(getperm(file) & libc::S_IXUSR as c_int, 0);

    // Readable: yes, then no once every read bit is off, and no for a name
    // that is not there at all.
    let readable = |name: &str| {
        let name = cname(name);
        os_file_is_readable(&name)
    };
    assert!(readable(file));
    assert!(!readable("unit-test-directory/what_are_you_smoking.gif"));
    let no_read = (libc::S_IRUSR | libc::S_IRGRP | libc::S_IROTH) as c_int;
    assert!(setperm(file, getperm(file) & !no_read).is_ok());
    assert!(!readable(file));

    // Writable answers 1 for a file, 2 for a directory and 0 for neither.
    let writable = |name: &str| {
        let name = cname(name);
        os_file_is_writable(&name)
    };
    assert_eq!(writable(file), 1);
    assert_eq!(writable("unit-test-directory"), 2);
    let no_write = (libc::S_IWUSR | libc::S_IWGRP | libc::S_IWOTH) as c_int;
    assert!(setperm(file, getperm(file) & !no_write).is_ok());
    assert_eq!(writable(file), 0);
}

/// Existence, which follows links — so a link to nothing does not exist.
#[test]
fn a_broken_link_does_not_exist() {
    let _fixture = Fixture::new("exists");
    let exists = |name: &str| path_exists(&cname(name));
    assert!(!exists("non-existing-file"));
    assert!(exists("unit-test-directory/test.file"));
    assert!(exists("unit-test-directory"));
    assert!(!exists("unit-test-directory/test_broken_link.file"));
}

#[test]
fn renaming_moves_a_file_and_overwrites_what_is_there() {
    let fixture = Fixture::new("rename");
    let rename = |from: &str, to: &str| {
        let (from, to) = (cname(from), cname(to));
        os_rename(&from, &to)
    };
    let test = "unit-test-directory/test.file";
    let absent = "unit-test-directory/not_exist.file";

    assert!(rename(test, absent).is_ok());
    assert!(!fixture.file().exists());
    assert!(fixture.path(absent).exists());
    assert!(rename(absent, test).is_ok(), "and back again");

    assert!(rename(absent, test).is_err(), "there is nothing to rename");

    let other = "unit-test-directory/other.file";
    std::fs::write(fixture.path(other), b"other").expect("another file");
    assert!(rename(other, test).is_ok());
    assert!(!fixture.path(other).exists());
    assert_eq!(std::fs::read(fixture.file()).unwrap(), b"other");
}

#[test]
fn removing_a_file_needs_it_to_be_there() {
    let fixture = Fixture::new("remove");
    let remove = |name: &str| {
        let name = cname(name);
        os_remove(&name)
    };
    assert_ne!(remove("non-existing-file"), 0);
    let doomed = "unit-test-directory/test_remove.file";
    std::fs::write(fixture.path(doomed), b"").expect("a file to remove");
    assert_eq!(remove(doomed), 0);
    assert!(!fixture.path(doomed).exists());
}

/// Opening, with the flags and the mode that the caller asked for.
#[test]
fn opening_honours_the_flags_and_the_mode() {
    let fixture = Fixture::new("open");
    let open = |name: &str, flags: c_int, mode: c_int| open_file(&cname(name), flags, mode);

    let existing = "unit-test-directory/test_existing.file";
    std::fs::write(fixture.path(existing), b"").expect("an existing file");
    let fresh = "test_new_file";

    assert_eq!(
        open("non-existing-file", libc::O_RDWR, 0).err(),
        Some(UV_ENOENT)
    );
    assert_eq!(
        open(existing, libc::O_CREAT | libc::O_EXCL, 0).err(),
        Some(UV_EEXIST),
        "O_EXCL insists the file is new"
    );

    for (name, expected_mode) in [(fresh, 0o700), ("test_new_file_2", 0o600)] {
        assert!(!fixture.path(name).exists());
        let file = open(name, libc::O_CREAT, expected_mode);
        assert!(file.is_ok(), "{name}");
        assert_eq!(
            std::fs::metadata(fixture.path(name))
                .expect(name)
                .permissions()
                .mode(),
            0o100_000 | expected_mode as u32,
            "{name}"
        );
    }

    // O_CREAT on something that is already there just opens it, and so does
    // O_RDWR -- which opens it for both reading and writing, and, as every
    // open here, close-on-exec.
    assert!(open(existing, libc::O_CREAT, 0).is_ok());
    let mut file = open(existing, libc::O_RDWR, 0).expect("an existing file");
    file.write_all(b"both ways").expect("open for writing");
    file.rewind().expect("a file seeks");
    let mut back = String::new();
    file.read_to_string(&mut back).expect("open for reading");
    assert_eq!(back, "both ways");
    // SAFETY: `F_GETFD` on a descriptor `file` keeps open.
    let flags = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFD) };
    assert_eq!(flags & libc::FD_CLOEXEC, libc::FD_CLOEXEC);
}

#[test]
fn a_name_that_is_not_there_is_a_normal_node() {
    let _fixture = Fixture::new("nodetype");
    let nodetype = |name: &str| {
        let name = cname(name);
        os_nodetype(&name)
    };
    assert_eq!(nodetype("non-existing-file"), NODE_NORMAL);

    // `/dev/stderr` is a character device only when the test's own stderr is
    // one. These suites are usually run with output redirected to a log, and
    // then it is a regular file and the question is not the same.
    let stderr_is_a_file = std::fs::metadata("/dev/stderr").is_ok_and(|m| m.is_file());
    if !stderr_is_a_file {
        assert_eq!(nodetype("/dev/stderr"), NODE_WRITABLE);
    }
}

#[test]
fn making_and_removing_a_directory() {
    let _fixture = Fixture::new("mkdir");
    let mkdir = |name: &str| {
        let name = cname(name);
        os_mkdir(&name, RWX)
    };
    let rmdir = |name: &str| {
        let name = cname(name);
        os_rmdir(&name)
    };

    assert_ne!(mkdir("unit-test-directory"), 0, "it is already there");
    assert!(!isdir("unit-test-directory/new-dir"));
    assert_eq!(mkdir("unit-test-directory/new-dir"), 0);
    assert!(isdir("unit-test-directory/new-dir"));

    assert_ne!(rmdir("non-existing-directory"), 0);
    assert_eq!(rmdir("unit-test-directory/new-dir"), 0);
    assert!(!isdir("unit-test-directory/new-dir"));
}

/// `mkdir_recurse` reports two things besides success: which directory it
/// could not make, and the *first* one it did — the one a caller has to
/// remove to undo the whole tree.
#[test]
fn making_a_directory_tree_names_the_first_one_it_made() {
    let fixture = Fixture::new("mkdir-recurse");
    let text = |name: CString| name.into_string().expect("text");
    let recurse = |name: &str| match mkdir_recurse(&cname(name), RWX) {
        Ok(created) => (0, None, created.map(text)),
        Err(failure) => (failure.code, failure.dir.map(text), None),
    };

    assert_eq!(
        recurse("unit-test-directory"),
        (0, None, None),
        "already there, nothing made"
    );

    // A file in the way, either as the target or as a component.
    let in_the_way = "unit-test-directory/test.file";
    let (result, failed, created) = recurse(in_the_way);
    assert_ne!(result, 0);
    assert_eq!(failed.as_deref(), Some(in_the_way));
    assert_eq!(created, None);
    let (result, failed, created) = recurse("unit-test-directory/test.file/test");
    assert_ne!(result, 0);
    assert_eq!(
        failed.as_deref(),
        Some(in_the_way),
        "the component, not the whole name"
    );
    assert_eq!(created, None);

    // A single directory, however it is spelled, and then a whole tree.
    for name in [
        "unit-test-directory/new-dir-recurse",
        "unit-test-directory/new-dir-recurse/",
        "unit-test-directory/new-dir-recurse///",
    ] {
        let (result, failed, created) = recurse(name);
        assert_eq!((result, failed), (0, None), "{name}");
        assert!(
            created
                .as_deref()
                .is_some_and(|c| c.ends_with("unit-test-directory/new-dir-recurse")),
            "{name}: {created:?}"
        );
        assert!(isdir("unit-test-directory/new-dir-recurse"));
        std::fs::remove_dir(fixture.path("unit-test-directory/new-dir-recurse")).expect("undo");
    }

    let (result, failed, created) = recurse("unit-test-directory/new-dir-recurse/1/2/3");
    assert_eq!((result, failed), (0, None));
    assert!(
        created
            .as_deref()
            .is_some_and(|c| c.ends_with("unit-test-directory/new-dir-recurse")),
        "the first one made, not the last: {created:?}"
    );
    for made in [
        "unit-test-directory/new-dir-recurse",
        "unit-test-directory/new-dir-recurse/1",
        "unit-test-directory/new-dir-recurse/1/2",
        "unit-test-directory/new-dir-recurse/1/2/3",
    ] {
        assert!(isdir(made), "{made}");
    }
}

/// `file_info` follows links and `link_info` does not. Both fail rather
/// than half-fill their answer.
#[test]
fn the_three_ways_of_asking_about_a_file() {
    let fixture = Fixture::new("fileinfo");
    let missing = cname("/non-existent");
    assert!(file_info(&missing).is_none());
    assert!(link_info(&missing).is_none());

    let file = info_of(&fixture.file()).expect("the fixture exists");
    assert!(filled_in(&file));

    // Through the link, `file_info` describes a regular file...
    let through = info_of(&fixture.link()).expect("the link resolves");
    assert!(filled_in(&through));
    assert_eq!(
        through.stat.st_mode as libc::mode_t & libc::S_IFMT,
        libc::S_IFREG
    );

    // ...and `link_info` describes the link itself.
    let link = link_info(&cpath(&fixture.link())).expect("the link is there");
    assert!(filled_in(&link));
    assert_eq!(
        link.stat.st_mode as libc::mode_t & libc::S_IFMT,
        libc::S_IFLNK
    );
}

/// The fields of a `FileInfo`, each against what the standard library says
/// about the same file.
#[test]
fn a_file_info_answers_the_same_numbers_the_system_does() {
    let fixture = Fixture::new("fileinfo-fields");
    let path = fixture.file();
    std::fs::write(&path, b"some bytes to get filesize != 0").expect("something to measure");
    let meta = std::fs::metadata(&path).expect("the fixture");
    let info = info_of(&path).expect("the fixture exists");

    assert_eq!(info.stat.st_ino, meta.ino());
    assert_eq!(info.stat.st_size, meta.len());
    assert_eq!(info.stat.st_blksize, meta.blksize());
    assert_eq!(info.stat.st_nlink, 1);

    // A hard link is a second name for the same inode, and the count
    // follows it — but only once the info is read again.
    let hard = fixture.path("unit-test-directory/test_hlink.file");
    std::fs::hard_link(&path, &hard).expect("a hard link");
    let relinked = info_of(&path).expect("still there");
    assert_eq!(relinked.stat.st_nlink, 2);

    // `id` copies the identifying pair out.
    let id = info.id();
    assert_eq!((id.inode, id.device_id), (meta.ino(), meta.dev()));
}

/// Identity, asked three ways: info against info, id against id, and id
/// against info. A link and its target are one file; two files are not.
#[test]
fn two_names_are_the_same_file_when_their_ids_agree() {
    let fixture = Fixture::new("fileid");
    let one = fixture.file();
    let two = fixture.path("unit-test-directory/test_2.file");
    let link = fixture.link();

    let info_one = info_of(&one).expect("one");
    let info_two = info_of(&two).expect("two");
    let info_link = info_of(&link).expect("the link resolves");

    assert!(info_one.same_file_as(&info_one));
    assert!(info_one.same_file_as(&info_link));
    assert!(!info_one.same_file_as(&info_two));

    // The id of a name: the stat's (inode, device) pair, as the buffer
    // list compares names by.
    let id_of = |name: &std::ffi::CStr| file_info(name).map(|info| info.id());
    assert!(id_of(&cname("/non-existent")).is_none());
    let id_one = id_of(&cpath(&one)).expect("one exists");
    assert!(id_one.inode > 0 && id_one.device_id > 0);
    let id_two = id_of(&cpath(&two)).expect("two exists");

    assert_eq!(id_one, id_one);
    assert_ne!(id_one, id_two);
    assert_eq!(id_one, info_one.id());
    assert_ne!(id_one, info_two.id());
}
