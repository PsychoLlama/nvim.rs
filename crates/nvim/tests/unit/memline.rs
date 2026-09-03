//! The swap file's first page, and what survives a crash.
//!
//! `memline/` had no `#[test]` at all before this file: the only checked-in
//! `.swp` files are `test/old/testdir/samples/recover-crash{1,2}.swp`, both
//! deliberately corrupt and both read only for the *message* they provoke.
//! Nothing anywhere asserted the contents of a **valid** swap file.
//!
//! # Why no golden bytes
//!
//! A `.swp` cannot be a byte golden the way a `.spl` can. Block zero carries
//! the writing process's pid, the edited file's mtime and inode, the user
//! name and the host name — five fields that differ between two runs on the
//! same machine, let alone two machines — and the blocks after it are laid
//! out at a page size the platform chooses. So these cases build a swap file
//! in a sandbox and read it back with a decoder of their own, which is the
//! same oracle a golden would be minus the parts that could never match.
//!
//! # Offsets, not literals
//!
//! Every offset below comes from `offset_of!`, never from a number. Block
//! zero is written by `memcpy`-shaped stores into a `#[repr(C)]` struct whose
//! last four fields are a `c_long`, a `c_int`, an `int16_t` and a `c_char`,
//! so its size and its tail padding are the platform's business. A test that
//! wrote `1008` would be asserting this machine rather than the format.

#![cfg(not(miri))]

use std::ffi::{CString, c_char, c_int, c_long};
use std::mem::offset_of;

use neovim::buffer::{BLN_LISTED, DOBUF_WIPE, buflist_new, close_buffer};
use neovim::main::p_dir;
use neovim::memline::{
    B0_FNAME_SIZE_CRYPT, B0_FNAME_SIZE_NOCRYPT, B0_FNAME_SIZE_ORG, B0_HNAME_SIZE, B0_MAGIC_CHAR,
    B0_MAGIC_INT, B0_MAGIC_LONG, B0_MAGIC_SHORT, B0_UNAME_SIZE, BLOCK0_ID0, BLOCK0_ID1, ZeroBlock,
    ml_append_buf, ml_close, ml_get_buf, ml_open, ml_open_file, ml_preserve,
};
use neovim::types::{buf_T, colnr_T, linenr_T};
use neovim::winlayer::Buf;

use crate::support::{Sandbox, cstr};

/// The lines every case writes into its buffer, chosen so that the last one
/// is long enough to be worth finding in a data block and the first is
/// short enough that a length confusion shows up as a truncation.
const LINES: &[&str] = &["one", "two", "three", "a somewhat longer fourth line"];

/// Block zero as it comes off disk: the fields that are numbers, decoded
/// here rather than through `block0.rs`, so that a change to the encoding
/// has to be made twice before it stops being noticed.
///
/// `long_to_char`/`char_to_long` are little-endian, four bytes, signed —
/// which is why an inode above 2^31 is stored truncated and why the reader
/// only ever compares it for equality.
struct OnDisk {
    bytes: Vec<u8>,
}

impl OnDisk {
    /// Read the first `size_of::<ZeroBlock>()` bytes of a swap file.
    fn read(path: &std::path::Path) -> OnDisk {
        let bytes = std::fs::read(path).expect("a swap file");
        assert!(
            bytes.len() >= size_of::<ZeroBlock>(),
            "a swap file is at least one page: {} bytes",
            bytes.len()
        );
        OnDisk { bytes }
    }

    /// The bytes of the field at `offset`, `len` wide.
    fn field(&self, offset: usize, len: usize) -> &[u8] {
        &self.bytes[offset..offset + len]
    }

    /// A four-byte little-endian signed number, the format's only integer
    /// encoding.
    fn number(&self, offset: usize) -> i32 {
        i32::from_le_bytes(self.field(offset, 4).try_into().expect("four bytes"))
    }

    /// A NUL-terminated string field, up to its terminator.
    fn text(&self, offset: usize, len: usize) -> &[u8] {
        let field = self.field(offset, len);
        let end = field.iter().position(|&b| b == 0).unwrap_or(field.len());
        &field[..end]
    }
}

/// A buffer with a file name in the sandbox, its memline open, its swap
/// file created, and `LINES` in it — plus the promise that the buffer is
/// wiped and `'directory'` is put back when the case ends.
struct Swapped {
    sandbox: Sandbox,
    buf: *mut buf_T,
    /// `'directory'`'s value on the way in.
    saved_dir: *mut c_char,
    /// The sandbox-local value, owned here so it outlives every read of it.
    _dir: CString,
    swap: std::path::PathBuf,
}

impl Swapped {
    /// `name` is both the case's private directory and the edited file's
    /// name; the swap file lands beside it because `'directory'` is set to
    /// the sandbox for the duration.
    fn new(name: &str) -> Swapped {
        let sandbox = Sandbox::dir(name);
        // The edited file has to exist for block zero's mtime and inode to
        // be anything but zero, which is half of what these cases assert.
        sandbox.write(name, b"one\ntwo\nthree\n");

        // `'directory'` decides where `findswapname` puts the file. Its
        // default is the user's real state directory, which a test must not
        // write into, so it is pointed at the sandbox and restored on drop.
        let dir = cstr(sandbox.as_str());
        let saved_dir = p_dir.get();
        p_dir.set(dir.as_ptr().cast_mut());

        let mut owned: Vec<c_char> = cstr(name)
            .as_bytes_with_nul()
            .iter()
            .map(|&b| b as c_char)
            .collect();
        // SAFETY: `owned` is this frame's, NUL-terminated and writable, as
        // `buflist_new`'s other callers pass it.
        let buf = unsafe {
            buflist_new(
                owned.as_mut_ptr(),
                owned.as_mut_ptr(),
                1,
                BLN_LISTED as c_int,
            )
        };
        assert!(!buf.is_null(), "a buffer for {name}");
        // SAFETY: a buffer just created, with no memline yet. `b_p_swf` is
        // what `ml_open_file` refuses on, and a buffer made outside the
        // usual `:edit` path has not been given the option's value.
        unsafe {
            (*buf).b_p_swf = 1;
            ml_open(buf).expect("a memline");
            ml_open_file(buf);
        }
        // SAFETY: the memline was just opened; appending after line `n`
        // puts each line at the end in turn.
        for (n, line) in LINES.iter().enumerate() {
            let text = cstr(*line);
            let appended = unsafe {
                ml_append_buf(
                    buf,
                    n as linenr_T,
                    text.as_ptr().cast_mut(),
                    line.len() as colnr_T + 1,
                    false,
                )
            };
            assert!(appended.is_ok(), "line {n} was appended");
        }
        // The buffer starts with one empty line, which the appends pushed
        // to the end; drop it so the line set is exactly `LINES`.
        // SAFETY: the memline holds `LINES.len() + 1` lines.
        unsafe { neovim::memline::ml_delete_buf(buf, LINES.len() as linenr_T + 1, false) }
            .expect("the empty line goes");

        // SAFETY: as above; a swap file was opened, so this writes it.
        unsafe { ml_preserve(buf, false, true) };

        let swap = std::fs::read_dir(sandbox.root())
            .expect("the sandbox")
            .filter_map(Result::ok)
            .map(|e| e.path())
            .find(|p| p.extension().is_some_and(|e| e == "swp"))
            .expect("`ml_open_file` created a swap file");

        Swapped {
            sandbox,
            buf,
            saved_dir,
            _dir: dir,
            swap,
        }
    }
}

impl Drop for Swapped {
    fn drop(&mut self) {
        p_dir.set(self.saved_dir);
        // SAFETY: the buffer this case opened, wiped as `buffer.rs` does —
        // a buffer left on the list is visible to every later case.
        unsafe {
            ml_close(self.buf, 1);
            close_buffer(None, Buf::new(self.buf), DOBUF_WIPE as c_int, false, false);
        }
        let _ = &self.sandbox;
    }
}

/// The layout the on-disk block is: every field's offset derived from the
/// struct, and each one where the previous field ends.
///
/// This is not a restatement of the declaration. Block zero is the one
/// structure in the tree whose *padding* is part of a file format: the three
/// magic values at the end exist to catch a swap file written by a build with
/// different type sizes, so a field that moved would be read as garbage by
/// the very check meant to detect that. Naming the adjacency here is what
/// makes a reordering a test failure rather than an unreadable swap file.
#[test]
fn block_zero_is_a_contiguous_run_of_fields() {
    let mut at = 0;
    let mut ends_at = |offset: usize, width: usize, name: &str| {
        assert_eq!(offset, at, "{name} starts where the last field ended");
        at = offset + width;
    };
    ends_at(offset_of!(ZeroBlock, b0_id), 2, "b0_id");
    ends_at(offset_of!(ZeroBlock, b0_version), 10, "b0_version");
    ends_at(offset_of!(ZeroBlock, b0_page_size), 4, "b0_page_size");
    ends_at(offset_of!(ZeroBlock, b0_mtime), 4, "b0_mtime");
    ends_at(offset_of!(ZeroBlock, b0_ino), 4, "b0_ino");
    ends_at(offset_of!(ZeroBlock, b0_pid), 4, "b0_pid");
    ends_at(
        offset_of!(ZeroBlock, b0_uname),
        B0_UNAME_SIZE as usize,
        "b0_uname",
    );
    ends_at(
        offset_of!(ZeroBlock, b0_hname),
        B0_HNAME_SIZE as usize,
        "b0_hname",
    );
    ends_at(
        offset_of!(ZeroBlock, b0_fname),
        B0_FNAME_SIZE_ORG as usize,
        "b0_fname",
    );

    // The magic tail. Each is one C type, and its alignment is the point:
    // `b0_magic_long` must sit at a `c_long`'s alignment for a build with a
    // different `long` to disagree about the *value* rather than to read a
    // different field.
    assert_eq!(
        offset_of!(ZeroBlock, b0_magic_long) % align_of::<c_long>(),
        0
    );
    assert!(offset_of!(ZeroBlock, b0_magic_long) >= at);
    assert!(offset_of!(ZeroBlock, b0_magic_int) > offset_of!(ZeroBlock, b0_magic_long));
    assert!(offset_of!(ZeroBlock, b0_magic_short) > offset_of!(ZeroBlock, b0_magic_int));
    assert!(offset_of!(ZeroBlock, b0_magic_char) > offset_of!(ZeroBlock, b0_magic_short));
    assert!(size_of::<ZeroBlock>() > offset_of!(ZeroBlock, b0_magic_char));

    // The name field is never filled to the brim: its last two bytes are
    // `b0_flags` and `b0_dirty`, and the crypt-era limit leaves eight more
    // spare on top of that.
    assert_eq!(B0_FNAME_SIZE_NOCRYPT, B0_FNAME_SIZE_ORG - 2);
    assert_eq!(B0_FNAME_SIZE_CRYPT, B0_FNAME_SIZE_NOCRYPT - 8);
}

/// A swap file written by `ml_open_file` reads back as one: the id, the
/// version string, the page size, and the four magic values that say the
/// build's type sizes match.
#[test]
fn a_fresh_swap_file_identifies_itself() {
    let swapped = Swapped::new("memline-identity");
    let b0 = OnDisk::read(&swapped.swap);

    let id = b0.field(offset_of!(ZeroBlock, b0_id), 2);
    assert_eq!(id, [BLOCK0_ID0 as u8, BLOCK0_ID1 as u8], "b0");
    // "VIM " and the oldest version that can read this file.
    let version = b0.text(offset_of!(ZeroBlock, b0_version), 10);
    assert_eq!(version, b"VIM 8.1");

    // The page size is what the memfile chose, and it has to be at least
    // big enough to hold the block that names it.
    let page_size = b0.number(offset_of!(ZeroBlock, b0_page_size));
    assert!(
        page_size as usize >= size_of::<ZeroBlock>(),
        "a page holds block zero: {page_size}"
    );
    assert_eq!(page_size.count_ones(), 1, "a power of two: {page_size}");

    // The magic tail, read at the struct's own offsets. A build whose
    // `long` were a different width would write these in different places,
    // which is the whole reason they exist.
    let long = c_long::from_ne_bytes(
        b0.field(offset_of!(ZeroBlock, b0_magic_long), size_of::<c_long>())
            .try_into()
            .expect("a c_long"),
    );
    assert_eq!(long, B0_MAGIC_LONG as c_long);
    let int = c_int::from_ne_bytes(
        b0.field(offset_of!(ZeroBlock, b0_magic_int), size_of::<c_int>())
            .try_into()
            .expect("a c_int"),
    );
    assert_eq!(int, B0_MAGIC_INT as c_int);
    let short = i16::from_ne_bytes(
        b0.field(offset_of!(ZeroBlock, b0_magic_short), 2)
            .try_into()
            .expect("an int16_t"),
    );
    assert_eq!(short, B0_MAGIC_SHORT as i16);
    assert_eq!(
        b0.field(offset_of!(ZeroBlock, b0_magic_char), 1)[0],
        B0_MAGIC_CHAR as u8
    );
}

/// The fields that say *which* file this swap file belongs to and *who*
/// left it there — the ones the ATTENTION message and `swapinfo()` read.
#[test]
fn a_fresh_swap_file_names_its_file_and_its_owner() {
    let swapped = Swapped::new("memline-owner");
    let b0 = OnDisk::read(&swapped.swap);
    let edited = swapped.sandbox.path("memline-owner");

    // The name is stored absolute, with `$HOME` folded to `~user/` when the
    // file is under it. The sandbox is under the temp directory, so it is
    // stored as it stands; the folded form is asserted from the functional
    // side, in `swapfile_preserve_recover_spec.lua`, where the file really
    // is under the home directory.
    let name = b0.text(
        offset_of!(ZeroBlock, b0_fname),
        B0_FNAME_SIZE_CRYPT as usize,
    );
    assert_eq!(
        name,
        edited.to_str().expect("a temp path is text").as_bytes()
    );

    // The two fields that make the swap file worth reading: this process
    // wrote it, and the file it names has not changed since.
    let pid = b0.number(offset_of!(ZeroBlock, b0_pid));
    assert_eq!(pid, std::process::id() as i32);

    let stat = std::fs::metadata(&edited).expect("the edited file");
    let mtime = b0.number(offset_of!(ZeroBlock, b0_mtime));
    let expected = std::os::unix::fs::MetadataExt::mtime(&stat) as i32;
    assert_eq!(mtime, expected);
    // The inode is stored in four signed bytes, so a large one is
    // truncated; the reader only ever compares it, so that is fine, but it
    // is why this is not `assert_eq!(ino, stat.ino())`.
    let ino = b0.number(offset_of!(ZeroBlock, b0_ino));
    assert_eq!(ino, std::os::unix::fs::MetadataExt::ino(&stat) as i32);

    // Both names are NUL-terminated inside their own field, which is what
    // `ml_check_b0_strings` refuses a file over.
    let uname = b0.text(offset_of!(ZeroBlock, b0_uname), B0_UNAME_SIZE as usize);
    assert!(uname.len() < B0_UNAME_SIZE as usize, "b0_uname terminates");
    assert!(!uname.is_empty(), "the process owner has a name");
    let hname = b0.text(offset_of!(ZeroBlock, b0_hname), B0_HNAME_SIZE as usize);
    assert!(hname.len() < B0_HNAME_SIZE as usize, "b0_hname terminates");
    assert!(!hname.is_empty(), "the host has a name");

    // The last two bytes of the name field are not the name: `b0_flags`
    // carries the `'fileformat'` and the "beside the file" bit, `b0_dirty`
    // whether the buffer had unsaved changes. Nothing here changed the
    // buffer, so it is clean.
    let flags = b0.field(offset_of!(ZeroBlock, b0_fname), B0_FNAME_SIZE_ORG as usize);
    assert_ne!(
        flags[B0_FNAME_SIZE_ORG as usize - 2],
        0,
        "b0_flags carries a 'fileformat'"
    );
    assert_eq!(
        flags[B0_FNAME_SIZE_ORG as usize - 1],
        0,
        "b0_dirty is clear"
    );
}

/// The flag the ATTENTION message and `:recover` key on: block zero says
/// whether the buffer had unsaved changes when the swap file was last
/// written. It is the difference between "there is a stale swap file" and
/// "there is work in here you have not saved".
#[test]
fn a_modified_buffer_marks_its_swap_file_dirty() {
    let swapped = Swapped::new("memline-dirty");
    let dirty_at = offset_of!(ZeroBlock, b0_fname) + B0_FNAME_SIZE_ORG as usize - 1;
    assert_eq!(
        OnDisk::read(&swapped.swap).bytes[dirty_at],
        0,
        "clean before the change"
    );

    // SAFETY: the buffer this case owns, with its memline open. Marking it
    // changed is what `changed_internal` does for a real edit; `ml_setflags`
    // is what carries that into block zero, and `ml_preserve` writes it.
    unsafe {
        (*swapped.buf).b_changed = 1;
        neovim::memline::ml_setflags(swapped.buf);
        ml_preserve(swapped.buf, false, true);
    }
    assert_ne!(
        OnDisk::read(&swapped.swap).bytes[dirty_at],
        0,
        "dirty after it"
    );

    // The line set is still what was written, so the flag is the only thing
    // that moved.
    // SAFETY: the buffer's memline holds `LINES`.
    let lines: Vec<String> = (1..=LINES.len() as linenr_T)
        .map(|lnum| unsafe {
            std::ffi::CStr::from_ptr(ml_get_buf(swapped.buf, lnum))
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(lines, LINES);
}
