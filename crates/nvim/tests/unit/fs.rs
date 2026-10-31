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

use std::ffi::{CString, c_char, c_int, c_void};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::ptr;

use neovim::os::fs::{
    NODE_NORMAL, NODE_WRITABLE, dir_exists, dirname_within, file_info, link_info, mkdir_recurse,
    open_fd, os_can_exe, os_chdir, os_close, os_dup, os_fchown, os_file_is_readable,
    os_file_is_writable, os_fileinfo_fd, os_mkdir, os_nodetype, os_read, os_readv, os_remove,
    os_rename, os_rmdir, os_setperm, os_write, path_exists,
};
use neovim::os::uv_error::{UV_EBADF, UV_EEXIST, UV_ENOENT};
use neovim::types::{FAIL, Failed, FileInfo, OK, iovec};

use crate::support::Sandbox;

/// The spec's fixture contents: every byte value once, sixteen times over.
fn contents() -> Vec<u8> {
    (0..=255_u8).cycle().take(256 * 16).collect()
}

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

    /// A file holding [`contents`].
    fn filled(&self, name: &str) -> PathBuf {
        self.0.write(name, &contents())
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

fn setperm(name: &str, perm: c_int) -> c_int {
    let name = cname(name);
    os_setperm(&name, perm)
}

/// A zeroed `FileInfo`, which every accessor fills in completely.
fn blank_info() -> FileInfo {
    // SAFETY: `FileInfo` wraps a `uv_stat_t`, which is plain data.
    unsafe { std::mem::zeroed() }
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
    assert_eq!(setperm("non-existing-file", RWX), FAIL);

    // The executable bit goes off and back on.
    let perm = getperm(file);
    assert_eq!(setperm(file, perm & !(libc::S_IXUSR as c_int)), OK);
    assert_eq!(getperm(file) & libc::S_IXUSR as c_int, 0);
    assert_eq!(setperm(file, perm | libc::S_IXUSR as c_int), OK);
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
    assert_eq!(setperm(file, getperm(file) & !no_read), OK);
    assert!(!readable(file));

    // Writable answers 1 for a file, 2 for a directory and 0 for neither.
    let writable = |name: &str| {
        let name = cname(name);
        os_file_is_writable(&name)
    };
    assert_eq!(writable(file), 1);
    assert_eq!(writable("unit-test-directory"), 2);
    let no_write = (libc::S_IWUSR | libc::S_IWGRP | libc::S_IWOTH) as c_int;
    assert_eq!(setperm(file, getperm(file) & !no_write), OK);
    assert_eq!(writable(file), 0);
}

/// `os_fchown` with -1 for both ids changes nothing, which is how callers
/// ask to set only one of them.
#[test]
fn changing_the_owner_of_a_file_to_nobody_in_particular_changes_nothing() {
    let fixture = Fixture::new("fchown");
    let file = fixture.file();
    let before = std::fs::metadata(&file).expect("the fixture");

    let name = cpath(&file);
    // SAFETY: `name` is this frame's; the fd is closed below.
    let fd = unsafe { libc::open(name.as_ptr(), 0) };
    assert!(fd >= 0);
    // SAFETY: `fd` is open.
    let result = unsafe { os_fchown(fd, u32::MAX, u32::MAX) };
    // SAFETY: `fd` is open and closed exactly once.
    unsafe { libc::close(fd) };
    assert_eq!(result, 0);

    let after = std::fs::metadata(&file).expect("the fixture");
    assert_eq!((after.uid(), after.gid()), (before.uid(), before.gid()));
}

/// Giving a file away to root is refused for anyone who is not root. Skipped
/// when the tests run as root, which the spec skipped too.
#[test]
fn giving_a_file_to_root_is_refused() {
    // SAFETY: `geteuid` takes no arguments.
    if unsafe { libc::geteuid() } == 0 {
        return;
    }
    let fixture = Fixture::new("fchown-root");
    let name = cpath(&fixture.file());
    // SAFETY: `name` is this frame's; the fd is closed below.
    let fd = unsafe { libc::open(name.as_ptr(), 0) };
    assert!(fd >= 0);
    // SAFETY: `fd` is open.
    let result = unsafe { os_fchown(fd, 0, 0) };
    // SAFETY: `fd` is open and closed exactly once.
    unsafe { libc::close(fd) };
    assert_ne!(result, 0);
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

    assert_eq!(rename(test, absent), OK);
    assert!(!fixture.file().exists());
    assert!(fixture.path(absent).exists());
    assert_eq!(rename(absent, test), OK, "and back again");

    assert_eq!(rename(absent, test), FAIL, "there is nothing to rename");

    let other = "unit-test-directory/other.file";
    std::fs::write(fixture.path(other), b"other").expect("another file");
    assert_eq!(rename(other, test), OK);
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

#[test]
fn duplicating_a_descriptor_answers_one_nothing_else_holds() {
    let _fixture = Fixture::new("dup");
    // SAFETY: 0, 1 and 2 are open in any process that runs a test.
    let dups: Vec<c_int> = (0..3).map(|fd| unsafe { os_dup(fd) }).collect();
    let mut all: Vec<c_int> = vec![0, 1, 2];
    all.extend(&dups);
    all.sort_unstable();
    all.dedup();
    assert_eq!(all.len(), 6, "all six descriptors are distinct");
    for fd in dups {
        // SAFETY: each is a descriptor `os_dup` just handed out.
        assert_eq!(unsafe { os_close(fd) }, 0);
    }
}

/// Opening, with the flags and the mode that the caller asked for, and
/// closing — including the two spellings of a descriptor that was never one.
#[test]
fn opening_honours_the_flags_and_the_mode() {
    let fixture = Fixture::new("open");
    let open = |name: &str, flags: c_int, mode: c_int| open_fd(&cname(name), flags, mode);
    // SAFETY: every descriptor closed here came from `open_fd`.
    let close = |fd: c_int| unsafe { os_close(fd) };

    let existing = "unit-test-directory/test_existing.file";
    std::fs::write(fixture.path(existing), b"").expect("an existing file");
    let fresh = "test_new_file";

    assert_eq!(open("non-existing-file", libc::O_RDWR, 0), UV_ENOENT);
    assert_eq!(
        open(existing, libc::O_CREAT | libc::O_EXCL, 0),
        UV_EEXIST,
        "O_EXCL insists the file is new"
    );

    for (name, expected_mode) in [(fresh, 0o700), ("test_new_file_2", 0o600)] {
        assert!(!fixture.path(name).exists());
        let fd = open(name, libc::O_CREAT, expected_mode);
        assert!(fd >= 0, "{name}");
        assert_eq!(close(fd), 0);
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
    // O_RDWR.
    for flags in [libc::O_CREAT, libc::O_RDWR] {
        let fd = open(existing, flags, 0);
        assert!(fd >= 0, "{flags:#o}");
        assert_eq!(close(fd), 0);
    }

    assert_eq!(close(-1), UV_EBADF);
    assert_eq!(close(-1000), UV_EBADF);
}

/// Reading, which loops over short reads and reports end of file separately
/// from the count — a zero-byte read at end of file is not the same answer
/// as a zero-byte read before it.
#[test]
fn reading_reports_the_count_and_the_end_of_file_apart() {
    let fixture = Fixture::new("read");
    let file = fixture.filled("read.dat");
    let want = contents();
    let name = cpath(&file);
    let fd = open_fd(&name, libc::O_RDONLY, 0);
    assert!(fd >= 0);

    let read = |size: Option<usize>| {
        let mut buf = size.map(|size| vec![0_u8; size]);
        let mut eof = true;
        let ptr = buf
            .as_mut()
            .map_or(ptr::null_mut(), |b| b.as_mut_ptr().cast::<c_char>());
        // SAFETY: `eof` is this frame's, and `ptr` addresses `size` writable
        // bytes or is null with a size of zero.
        let count = unsafe { os_read(fd, &raw mut eof, ptr, size.unwrap_or(0), false) };
        (eof, count, buf.unwrap_or_default())
    };

    assert_eq!(read(None), (false, 0, Vec::new()), "nowhere to read into");
    assert_eq!(read(Some(0)), (false, 0, Vec::new()), "nothing to read");
    assert_eq!(read(Some(2)), (false, 2, vec![0, 1]));
    assert_eq!(read(Some(2)), (false, 2, vec![2, 3]));

    // Three quarters, then the rest — the second call hits the end and says
    // so, and leaves the tail of the buffer alone.
    let three_quarters = want.len() * 3 / 4;
    // Start over.
    // SAFETY: `fd` is open.
    assert_eq!(unsafe { os_close(fd) }, 0);
    let fd = open_fd(&name, libc::O_RDONLY, 0);
    assert!(fd >= 0);
    let read = |size: usize| {
        let mut buf = vec![0_u8; size];
        let mut eof = true;
        // SAFETY: `eof` and `buf` are this frame's.
        let count = unsafe {
            os_read(
                fd,
                &raw mut eof,
                buf.as_mut_ptr().cast::<c_char>(),
                size,
                false,
            )
        };
        (eof, count, buf)
    };
    let (eof, count, got) = read(three_quarters);
    assert_eq!((eof, count), (false, three_quarters as isize));
    assert_eq!(got, want[..three_quarters]);
    let (eof, count, got) = read(three_quarters);
    let rest = want.len() - three_quarters;
    assert_eq!((eof, count), (true, rest as isize));
    assert_eq!(&got[..rest], &want[three_quarters..]);
    assert!(got[rest..].iter().all(|&b| b == 0), "the tail is untouched");
    // SAFETY: `fd` is open and closed exactly once.
    assert_eq!(unsafe { os_close(fd) }, 0);
}

/// `os_readv` fills a list of buffers in order, and reports the total. Its
/// interesting case is a list whose entries are different sizes, because it
/// has to advance through them as short reads land.
#[test]
fn a_scatter_read_fills_its_buffers_in_order() {
    let fixture = Fixture::new("readv");
    let file = fixture.filled("readv.dat");
    let want = contents();
    let name = cpath(&file);
    let fd = open_fd(&name, libc::O_RDONLY, 0);
    assert!(fd >= 0);

    let readv = |sizes: &[usize]| {
        let mut buffers: Vec<Vec<u8>> = sizes.iter().map(|&n| vec![0_u8; n]).collect();
        let mut iov: Vec<iovec> = buffers
            .iter_mut()
            .map(|b| iovec {
                iov_base: b.as_mut_ptr().cast::<c_void>(),
                iov_len: b.len(),
            })
            .collect();
        let mut eof = true;
        // SAFETY: `eof` is this frame's and `iov` addresses `sizes.len()`
        // live entries whose buffers are writable for their stated lengths.
        let count = unsafe { os_readv(fd, &raw mut eof, iov.as_mut_ptr(), iov.len(), false) };
        (eof, count, buffers)
    };

    assert_eq!(readv(&[]), (false, 0, Vec::new()));
    assert_eq!(
        readv(&[0, 0, 0]),
        (false, 0, vec![Vec::new(), Vec::new(), Vec::new()])
    );
    assert_eq!(readv(&[2]), (false, 2, vec![vec![0, 1]]));
    assert_eq!(
        readv(&[2, 3]),
        (false, 5, vec![vec![2, 3], vec![4, 5, 6]]),
        "the second buffer is a different size from the first"
    );

    // Start over and take the whole file in four unequal pieces.
    // SAFETY: `fd` is open.
    assert_eq!(unsafe { os_close(fd) }, 0);
    let fd = open_fd(&name, libc::O_RDONLY, 0);
    assert!(fd >= 0);
    let readv = |sizes: &[usize]| {
        let mut buffers: Vec<Vec<u8>> = sizes.iter().map(|&n| vec![0_u8; n]).collect();
        let mut iov: Vec<iovec> = buffers
            .iter_mut()
            .map(|b| iovec {
                iov_base: b.as_mut_ptr().cast::<c_void>(),
                iov_len: b.len(),
            })
            .collect();
        let mut eof = true;
        // SAFETY: as above.
        let count = unsafe { os_readv(fd, &raw mut eof, iov.as_mut_ptr(), iov.len(), false) };
        (eof, count, buffers)
    };
    let n = want.len();
    let sizes = [n / 4, n / 2, n * 3 / 16, n / 16];
    let (eof, count, got) = readv(&sizes);
    assert_eq!((eof, count), (false, n as isize));
    let mut at = 0;
    for (piece, size) in got.iter().zip(sizes) {
        assert_eq!(piece, &want[at..at + size]);
        at += size;
    }
    assert_eq!(readv(&[1]), (true, 0, vec![vec![0]]), "and now the end");
    // SAFETY: `fd` is open and closed exactly once.
    assert_eq!(unsafe { os_close(fd) }, 0);
}

#[test]
fn writing_puts_bytes_where_the_descriptor_is_pointing() {
    let fixture = Fixture::new("write");
    let file = fixture.filled("write.dat");
    let want = contents();
    let name = cpath(&file);
    let fd = open_fd(&name, libc::O_WRONLY, 0);
    assert!(fd >= 0);

    let write = |bytes: Option<&[u8]>| {
        let ptr = bytes.map_or(ptr::null(), |b| b.as_ptr().cast::<c_char>());
        // SAFETY: `ptr` addresses `len` readable bytes, or is null with a
        // length of zero.
        unsafe { os_write(fd, ptr, bytes.map_or(0, <[u8]>::len), false) }
    };

    assert_eq!(write(Some(b"")), 0);
    assert_eq!(write(None), 0, "nothing to write from");
    assert_eq!(std::fs::read(&file).unwrap(), want, "and nothing written");

    assert_eq!(write(Some(b"abc")), 3);
    assert_eq!(write(Some(b" def")), 4);
    // SAFETY: `fd` is open and closed exactly once.
    assert_eq!(unsafe { os_close(fd) }, 0);

    let mut expected = b"abc def".to_vec();
    expected.extend_from_slice(&want[7..]);
    assert_eq!(std::fs::read(&file).unwrap(), expected);
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

/// `file_info` follows links, `link_info` does not, and
/// `os_fileinfo_fd` asks about an already-open descriptor. All three fail
/// rather than half-fill their answer.
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

    // And by descriptor, which has no name to fail on but does have a
    // descriptor that was never one.
    let mut by_fd = blank_info();
    // SAFETY: -1 is never an open descriptor.
    assert!(!unsafe { os_fileinfo_fd(-1, &raw mut by_fd) });
    let name = cpath(&fixture.file());
    // SAFETY: `name` is this frame's; the fd is closed below.
    let fd = unsafe { libc::open(name.as_ptr(), 0) };
    assert!(fd >= 0);
    // SAFETY: `fd` is open and `by_fd` is writable.
    assert!(unsafe { os_fileinfo_fd(fd, &raw mut by_fd) });
    // SAFETY: `fd` is open and closed exactly once.
    unsafe { libc::close(fd) };
    assert!(filled_in(&by_fd));
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
