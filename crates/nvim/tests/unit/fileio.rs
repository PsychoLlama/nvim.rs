//! Buffered file I/O over libuv: `os/fileio.rs`.
//!
//! A port of `test/unit/os/fileio_spec.lua`. Every case works on real files
//! in a real directory, which is the point — the buffering is only
//! interesting against a file system that can say `EEXIST`, `EISDIR` and
//! `ELOOP` for itself.
//!
//! Two isolation obligations the LuaJIT harness did not have. A
//! `FileDescriptor` buffers through `alloc_block`, whose reuse list is a
//! process-wide global, so every case takes the editor lock; and the spec's
//! fixture directory had a fixed name, which two cases running at once would
//! have fought over, so each case here gets its own.

#![cfg(not(miri))]

use std::ffi::{CString, c_char, c_int};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::io::IntoRawFd;
use std::path::{Path, PathBuf};

use c2rust_neovim::os::fileio::{
    FileOpenFlags, file_close, file_flush, file_fsync, file_open, file_open_fd, file_read,
    file_skip, file_write, kFileCreate, kFileCreateOnly, kFileNoSymlink, kFileReadOnly,
    kFileTruncate, kFileWriteOnly,
};
use c2rust_neovim::os::uv_error::{UV_EEXIST, UV_EISDIR, UV_ELOOP, UV_EMLINK, UV_ENOENT};
use c2rust_neovim::types::FileDescriptor;

use crate::support::{Editor, editor_lock};

/// The spec's fixture contents: every byte value once, sixteen times over.
fn contents() -> Vec<u8> {
    (0..=255_u8).cycle().take(256 * 16).collect()
}

/// `rwx------` and `rw-------`, as the spec spelled them in decimal.
const RWX: c_int = 0o700;
const RW: c_int = 0o600;

/// A private directory holding the spec's four fixtures, removed when the
/// case ends, plus the editor lock the block allocator needs.
struct Fixture {
    dir: PathBuf,
    _editor: Editor,
}

impl Fixture {
    fn new(name: &str) -> Fixture {
        let editor = editor_lock();
        let dir = std::env::temp_dir().join(format!("nvim-unit-fileio-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a private fixture directory");
        let fixture = Fixture {
            dir,
            _editor: editor,
        };
        std::fs::write(fixture.file1(), contents()).expect("file1");
        std::fs::write(fixture.path("file2.dat"), contents()).expect("file2");
        std::os::unix::fs::symlink("file1.dat", fixture.link()).expect("a link to file1");
        std::os::unix::fs::symlink("broken.dat", fixture.broken()).expect("a link to nothing");
        fixture
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    /// A 4,096-byte file that exists.
    fn file1(&self) -> PathBuf {
        self.path("file1.dat")
    }

    /// A symbolic link to [`Self::file1`].
    fn link(&self) -> PathBuf {
        self.path("file.lnk")
    }

    /// A symbolic link to a name that does not exist.
    fn broken(&self) -> PathBuf {
        self.path("broken.dat.lnk")
    }

    /// A name nothing has created yet.
    fn fresh(&self) -> PathBuf {
        self.path("created-file.dat")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// A `FileDescriptor` and the calls that drive it, so the cases read as the
/// protocol rather than as pointer work.
struct Handle(FileDescriptor);

impl Handle {
    /// `file_open`: (the error, the descriptor).
    fn open(path: &Path, flags: FileOpenFlags, mode: c_int) -> (c_int, Handle) {
        let mut handle = Handle(zeroed_descriptor());
        let name = cpath(path);
        // SAFETY: the descriptor is this frame's and `name` outlives the call.
        let err = unsafe { file_open(&raw mut handle.0, name.as_ptr(), flags as c_int, mode) };
        (err, handle)
    }

    /// `file_open_fd` over a descriptor the standard library opened, which is
    /// what the spec did through `os_open`.
    fn adopt(fd: c_int, flags: FileOpenFlags) -> (c_int, Handle) {
        let mut handle = Handle(zeroed_descriptor());
        // SAFETY: `fd` is open and the descriptor is this frame's.
        let err = unsafe { file_open_fd(&raw mut handle.0, fd, flags as c_int) };
        (err, handle)
    }

    /// Whether this side is the write side — the spec's `fp.wr`.
    fn writing(&self) -> bool {
        self.0.wr
    }

    /// `file_read` of `size` bytes: (the count, the `size`-byte buffer). The
    /// buffer is returned whole, zero-padded, because the spec asserted on
    /// what a short read leaves behind as well as on the count.
    fn read(&mut self, size: usize) -> (isize, Vec<u8>) {
        let mut buf = vec![0_u8; size];
        // SAFETY: the descriptor is live and `buf` is writable for `size`.
        let count = unsafe { file_read(&raw mut self.0, buf.as_mut_ptr().cast::<c_char>(), size) };
        (count, buf)
    }

    fn write(&mut self, bytes: &[u8]) -> isize {
        // SAFETY: the descriptor is live and `bytes` is this frame's.
        unsafe {
            file_write(
                &raw mut self.0,
                bytes.as_ptr().cast::<c_char>(),
                bytes.len(),
            )
        }
    }

    fn skip(&mut self, size: usize) -> isize {
        // SAFETY: the descriptor is live.
        unsafe { file_skip(&raw mut self.0, size) }
    }

    fn flush(&mut self) -> c_int {
        // SAFETY: the descriptor is live.
        unsafe { file_flush(&raw mut self.0) }
    }

    fn fsync(&mut self) -> c_int {
        // SAFETY: the descriptor is live.
        unsafe { file_fsync(&raw mut self.0) }
    }

    /// `file_close` spends the descriptor: it releases the buffer block and
    /// leaves the fd behind unchanged, so nothing may close it twice.
    fn close(&mut self, do_fsync: bool) -> c_int {
        // SAFETY: the descriptor is live and this is its only close.
        unsafe { file_close(&raw mut self.0, do_fsync) }
    }
}

/// The `ffi.new('FileDescriptor')` the spec handed to `file_open`: zeroed,
/// and filled in entirely by the call.
fn zeroed_descriptor() -> FileDescriptor {
    // SAFETY: `FileDescriptor` is plain data, and every field is written by
    // `file_open`/`file_open_fd` before anything reads one.
    unsafe { std::mem::zeroed() }
}

fn cpath(path: &Path) -> CString {
    CString::new(path.as_os_str().as_encoded_bytes()).expect("a temp path holds no NUL")
}

/// The mode bits of `path`, or `None` when it does not exist.
fn mode_of(path: &Path) -> Option<u32> {
    std::fs::metadata(path).ok().map(|m| m.permissions().mode())
}

fn size_of(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().map(|m| m.len())
}

#[test]
fn a_descriptor_can_be_adopted_for_reading_and_for_writing() {
    let fixture = Fixture::new("adopt");

    let fd = std::fs::File::open(fixture.file1())
        .expect("file1")
        .into_raw_fd();
    let (err, mut fp) = Handle::adopt(fd, kFileReadOnly);
    assert_eq!(err, 0);
    let want = contents();
    assert_eq!(fp.read(want.len()), (want.len() as isize, want.clone()));
    assert_eq!(fp.close(false), 0);

    assert_eq!(size_of(&fixture.fresh()), None);
    let fd = std::fs::File::create(fixture.fresh())
        .expect("a new file")
        .into_raw_fd();
    let (err, mut fp) = Handle::adopt(fd, kFileWriteOnly);
    assert_eq!(err, 0);
    assert_eq!(fp.write(b"test"), 4);
    assert_eq!(fp.close(false), 0);
    assert_eq!(std::fs::read(fixture.fresh()).unwrap(), b"test");
}

/// The two creating flags both honour the mode they are given, and they
/// differ only on a file that already exists.
#[test]
fn creating_a_file_honours_the_mode_and_create_only_refuses_an_existing_one() {
    let fixture = Fixture::new("create");
    for flags in [kFileCreate, kFileCreateOnly] {
        for mode in [RWX, RW] {
            let _ = std::fs::remove_file(fixture.fresh());
            let (err, mut fp) = Handle::open(&fixture.fresh(), flags, mode);
            assert_eq!(err, 0);
            assert_eq!(
                mode_of(&fixture.fresh()),
                Some(0o100_000 | mode as u32),
                "{flags:#x} with mode {mode:#o}"
            );
            assert_eq!(fp.close(false), 0);
        }
    }

    let (err, _) = Handle::open(&fixture.file1(), kFileCreateOnly, RW);
    assert_eq!(err, UV_EEXIST);
    // ...whereas `kFileCreate` is happy to open it, for writing.
    let (err, mut fp) = Handle::open(&fixture.file1(), kFileCreate, RW);
    assert_eq!(err, 0);
    assert!(fp.writing());
    assert_eq!(fp.close(false), 0);
}

/// Which flags open which side. Read is the default, and every creating or
/// truncating flag implies write — this interface never opens both at once.
#[test]
fn the_flags_decide_which_side_the_descriptor_is() {
    let fixture = Fixture::new("sides");
    for flags in [0, kFileReadOnly, kFileNoSymlink] {
        let (err, mut fp) = Handle::open(&fixture.file1(), flags, RW);
        assert_eq!(err, 0, "{flags:#x}");
        assert!(!fp.writing(), "{flags:#x} reads");
        assert_eq!(fp.close(false), 0);
    }

    // Write-only leaves the contents alone...
    let (err, mut fp) = Handle::open(&fixture.file1(), kFileWriteOnly, RW);
    assert_eq!(err, 0);
    assert!(fp.writing());
    assert_eq!(fp.close(false), 0);
    assert_eq!(size_of(&fixture.file1()), Some(4096));

    // ...and truncating does not. The write proves the side is real: with
    // only `O_TRUNC` and no `O_WRONLY` the file would still come out empty,
    // and `fp.wr` would still be true.
    let (err, mut fp) = Handle::open(&fixture.file1(), kFileTruncate, RW);
    assert_eq!(err, 0);
    assert!(fp.writing());
    assert_eq!(fp.write(b"kept"), 4);
    assert_eq!(fp.close(false), 0);
    assert_eq!(std::fs::read(fixture.file1()).unwrap(), b"kept");
}

/// What cannot be opened, and with which error. `kFileWriteOnly` alone never
/// creates anything, which is the distinction between it and `kFileCreate`.
#[test]
fn the_four_ways_an_open_fails() {
    let fixture = Fixture::new("failures");

    let (err, _) = Handle::open(&fixture.fresh(), kFileWriteOnly, RW);
    assert_eq!(err, UV_ENOENT, "write-only creates nothing");
    assert_eq!(size_of(&fixture.fresh()), None);

    let (err, _) = Handle::open(&fixture.dir, kFileWriteOnly, RW);
    assert_eq!(err, UV_EISDIR, "a directory has no write side");

    for flags in [kFileWriteOnly, kFileReadOnly] {
        let (err, _) = Handle::open(&fixture.broken(), flags, RW);
        assert_eq!(err, UV_ENOENT, "a broken link points at nothing");
    }

    let (err, _) = Handle::open(&fixture.link(), kFileNoSymlink, RW);
    // `O_NOFOLLOW` is `ELOOP` on Linux and `EMLINK` on FreeBSD. The spec
    // spelled the alternative this way round so a third answer would show up
    // in the failure rather than be swallowed by an `or`.
    if err != UV_ELOOP {
        assert_eq!(err, UV_EMLINK);
    }
}

/// A link is followed for everything except `kFileNoSymlink`, so truncating
/// through one truncates its target.
#[test]
fn truncating_through_a_symlink_truncates_the_target() {
    let fixture = Fixture::new("symlink-truncate");
    let (err, mut fp) = Handle::open(&fixture.link(), kFileTruncate, RW);
    assert_eq!(err, 0);
    assert!(fp.writing());
    assert_eq!(fp.close(false), 0);
    assert_eq!(size_of(&fixture.file1()), Some(0));
}

/// Reading is buffered, and the buffer is 4,096 bytes — the same size as the
/// fixture, so each chunk size below crosses it differently. A read past the
/// end answers what was left and leaves the rest of the caller's buffer
/// alone.
#[test]
fn reading_in_chunks_of_every_awkward_size() {
    let fixture = Fixture::new("read");
    let want = contents();

    for size in [3, 768, want.len()] {
        let (err, mut fp) = Handle::open(&fixture.file1(), 0, RW);
        assert_eq!(err, 0);
        assert!(!fp.writing());
        let mut shift = 0;
        while shift < want.len() {
            let left = want.len() - shift;
            let (count, got) = fp.read(size);
            let taken = size.min(left);
            assert_eq!(count, taken as isize, "at {shift} by {size}");
            assert_eq!(&got[..taken], &want[shift..shift + taken]);
            assert!(
                got[taken..].iter().all(|&b| b == 0),
                "a short read leaves the rest of the buffer alone"
            );
            shift += size;
        }
        // At end of file: nothing read, nothing written.
        assert_eq!(fp.read(size), (0, vec![0; size]));
        assert_eq!(fp.close(false), 0);
    }

    // A small read followed by one larger than the buffer: the second has to
    // pick up what the first left buffered before going back to the file.
    let (err, mut fp) = Handle::open(&fixture.file1(), 0, RW);
    assert_eq!(err, 0);
    assert_eq!(fp.read(5), (5, want[..5].to_vec()));
    let (count, got) = fp.read(want.len());
    assert_eq!(count, (want.len() - 5) as isize);
    assert_eq!(&got[..want.len() - 5], &want[5..]);
    assert_eq!(fp.close(false), 0);
}

#[test]
fn writing_in_chunks_of_every_awkward_size() {
    let fixture = Fixture::new("write");
    let want = contents();

    for size in [3, 768, want.len()] {
        let _ = std::fs::remove_file(fixture.fresh());
        let (err, mut fp) = Handle::open(&fixture.fresh(), kFileCreateOnly, RW);
        assert_eq!(err, 0);
        assert!(fp.writing());
        for chunk in want.chunks(size) {
            assert_eq!(fp.write(chunk), chunk.len() as isize);
        }
        assert_eq!(fp.close(false), 0);
        assert_eq!(std::fs::read(fixture.fresh()).unwrap(), want, "by {size}");
    }
}

/// Three ways to get pending output onto the disk, all equivalent as far as
/// the file's size is concerned. What each case really asserts is the other
/// half: that nothing reaches the disk *before* it is asked for.
#[test]
fn output_reaches_the_disk_only_when_it_is_asked_to() {
    let fixture = Fixture::new("flush");
    // `file_close` spends the descriptor, so unlike the other two it is not
    // followed by a close of its own.
    type Flusher = (&'static str, fn(&mut Handle) -> c_int, bool);
    let flushers: [Flusher; 3] = [
        ("file_flush", Handle::flush, true),
        ("file_fsync", Handle::fsync, true),
        ("file_close(true)", |fp| fp.close(true), false),
    ];
    for (name, flush, close_after) in flushers {
        let _ = std::fs::remove_file(fixture.fresh());
        let (err, mut fp) = Handle::open(&fixture.fresh(), kFileCreateOnly, RW);
        assert_eq!(err, 0);
        assert_eq!(size_of(&fixture.fresh()), Some(0));

        assert_eq!(fp.write(b"test"), 4);
        assert_eq!(size_of(&fixture.fresh()), Some(0), "{name} not called yet");
        assert_eq!(flush(&mut fp), 0, "{name}");
        assert_eq!(size_of(&fixture.fresh()), Some(4), "{name}");
        if close_after {
            assert_eq!(fp.close(false), 0, "{name}");
        }
    }
}

#[test]
fn skipping_advances_the_read_position() {
    let fixture = Fixture::new("skip");
    let want = contents();
    let (err, mut fp) = Handle::open(&fixture.file1(), 0, RW);
    assert_eq!(err, 0);
    assert!(!fp.writing());
    assert_eq!(fp.skip(3), 3);
    assert_eq!(fp.read(3), (3, want[3..6].to_vec()));
    // Skipping past the end answers what it could skip, and reads nothing.
    assert_eq!(fp.skip(want.len()), (want.len() - 6) as isize);
    assert_eq!(fp.read(3), (0, vec![0; 3]));
    assert_eq!(fp.close(false), 0);
}
