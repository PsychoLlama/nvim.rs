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

use std::ffi::{CString, c_int, c_long};
use std::mem::offset_of;

use neovim::buffer::{BLN_LISTED, DOBUF_WIPE, buflist_new, close_buffer};
use neovim::memline::{
    B0_FNAME_SIZE_CRYPT, B0_FNAME_SIZE_NOCRYPT, B0_FNAME_SIZE_ORG, B0_HNAME_SIZE, B0_MAGIC_CHAR,
    B0_MAGIC_INT, B0_MAGIC_LONG, B0_MAGIC_SHORT, B0_UNAME_SIZE, BLOCK0_ID0, BLOCK0_ID1, Lines,
    ZeroBlock, ml_append_buf, ml_close, ml_get_buf, ml_open, ml_open_file, ml_preserve,
};
use neovim::memory::XString;
use neovim::option::vars::P_DIR;
use neovim::types::{Buffer, ColNr, LineNr};
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
    buf: *mut Buffer,
    /// `'directory'`'s value on the way in.
    saved_dir: Option<XString>,
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
        let saved_dir = P_DIR.swap(Some(XString::from_cstr(&dir)));

        let owned = cstr(name);
        let buf = buflist_new(Some(&owned), Some(&owned), 1, BLN_LISTED as c_int);
        let mut buf = buf.expect("a buffer for {name}");
        // SAFETY: a buffer just created, with no memline yet. `b_p_swf` is
        // what `ml_open_file` refuses on, and a buffer made outside the
        // usual `:edit` path has not been given the option's value.
        buf.b_p_swf = 1;
        ml_open(buf).expect("a memline");
        ml_open_file(buf);
        // SAFETY: the memline was just opened; appending after line `n`
        // puts each line at the end in turn.
        for (n, line) in LINES.iter().enumerate() {
            let text = cstr(*line);
            let appended = unsafe {
                ml_append_buf(
                    buf,
                    n as LineNr,
                    text.as_ptr().cast_mut(),
                    line.len() as ColNr + 1,
                    false,
                )
            };
            assert!(appended.is_ok(), "line {n} was appended");
        }
        // The buffer starts with one empty line, which the appends pushed
        // to the end; drop it so the line set is exactly `LINES`.
        neovim::memline::ml_delete_buf(buf, LINES.len() as LineNr + 1, false)
            .expect("the empty line goes");

        ml_preserve(buf, false, true);

        let swap = std::fs::read_dir(sandbox.root())
            .expect("the sandbox")
            .filter_map(Result::ok)
            .map(|e| e.path())
            .find(|p| p.extension().is_some_and(|e| e == "swp"))
            .expect("`ml_open_file` created a swap file");

        Swapped {
            sandbox,
            buf: buf.raw(),
            saved_dir,
            _dir: dir,
            swap,
        }
    }
}

impl Drop for Swapped {
    fn drop(&mut self) {
        P_DIR.restore(self.saved_dir.take());
        // SAFETY: the buffer this case opened, wiped as `buffer.rs` does —
        // a buffer left on the list is visible to every later case.
        unsafe {
            ml_close(Buf::new(self.buf), 1);
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
        neovim::memline::ml_setflags(Buf::new(swapped.buf));
        ml_preserve(Buf::new(swapped.buf), false, true);
    }
    assert_ne!(
        OnDisk::read(&swapped.swap).bytes[dirty_at],
        0,
        "dirty after it"
    );

    // The line set is still what was written, so the flag is the only thing
    // that moved.
    // SAFETY: the buffer's memline holds `LINES`.
    let lines: Vec<String> = (1..=LINES.len() as LineNr)
        .map(|lnum| unsafe {
            std::ffi::CStr::from_ptr(ml_get_buf(Buf::new(swapped.buf), lnum))
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(lines, LINES);
}

/// `Lines` reads the same bytes the pointer form does, and writes through
/// the line the memline already holds.
///
/// The slice's length is the one thing a `*mut c_char` cannot carry, so this
/// is where it is checked against `ml_get_buf_len` and against the text the
/// buffer was built from. The in-place write then goes back out through
/// `ml_get_buf`, which is the path every other reader in the tree takes: a
/// `line_mut` that wrote somewhere else would show up here as an unchanged
/// line.
#[test]
fn the_line_handle_reads_and_writes_the_line_the_memline_holds() {
    let swapped = Swapped::new("lines-handle");
    // SAFETY: the fixture's buffer is live and has a memline; nothing below
    // reads a line except through the handle.
    let mut lines = unsafe { Lines::in_buffer(Buf::new(swapped.buf)) };

    for (n, want) in LINES.iter().enumerate() {
        let lnum = n as LineNr + 1;
        assert_eq!(lines.line(lnum), want.as_bytes(), "line {lnum}");
        // SAFETY: a line of this buffer.
        let len = unsafe { neovim::memline::ml_get_buf_len(Buf::new(swapped.buf), lnum) };
        assert_eq!(
            lines.line(lnum).len(),
            len as usize,
            "length of line {lnum}"
        );
    }

    // Rewrite the bytes already there -- the only change `line_mut` allows.
    let last = LINES.len() as LineNr;
    lines.line_mut(last).make_ascii_uppercase();
    assert_eq!(
        lines.line(last),
        LINES[LINES.len() - 1].to_uppercase().as_bytes()
    );

    // ... and the pointer form sees it, so the write landed in the memline
    // and not in a copy.
    // SAFETY: a line of this buffer, NUL-terminated as `ml_get_buf` promises.
    let through_pointer = unsafe {
        std::ffi::CStr::from_ptr(ml_get_buf(Buf::new(swapped.buf), last))
            .to_string_lossy()
            .into_owned()
    };
    assert_eq!(through_pointer, LINES[LINES.len() - 1].to_uppercase());
}

/// What the line handle answers for a line that is not there, and for the
/// two shapes that carry the terminator with them.
///
/// `ml_get_buf` cannot fail — every failure path hands back the `???`
/// placeholder instead, and the cache is set to describe it — which is why
/// `Lines::line` has no error case either. That placeholder used to be a
/// `static mut` buffer rewritten on every call; this pins that it still
/// reads as three bytes and that asking for it does not disturb the line
/// that was cached before.
#[test]
fn an_out_of_range_line_reads_as_the_placeholder() {
    let swapped = Swapped::new("lines-placeholder");
    // SAFETY: the fixture's buffer is live and has a memline.
    let mut lines = unsafe { Lines::in_buffer(Buf::new(swapped.buf)) };

    let past_the_end = LINES.len() as LineNr + 1;
    assert_eq!(lines.line(past_the_end), b"???");
    assert_eq!(lines.line_len(past_the_end), 3);
    // Reading it twice answers the same thing -- it is not a buffer that
    // one read empties.
    assert_eq!(lines.line(past_the_end), b"???");
    // And a real line still reads as itself afterwards.
    assert_eq!(lines.line(1), LINES[0].as_bytes());
}

/// The terminated views of a line: `line_cstr` borrows the cache and
/// `line_copy` owns a copy, and both carry the NUL the memline keeps after
/// every line.
///
/// The NUL is the whole point (`LineCopy`'s own documentation says so): the
/// walks that step one past the last character read it, and a bare `Vec` of
/// the line's bytes has uninitialised capacity there. A suffix of a
/// terminated string is terminated, so `line_cstr` at a non-zero column is
/// the one shape a "pointer into a line" can take without a promise.
#[test]
fn the_terminated_views_carry_the_line_s_own_nul() {
    let swapped = Swapped::new("lines-terminated");
    // SAFETY: the fixture's buffer is live and has a memline.
    let mut lines = unsafe { Lines::in_buffer(Buf::new(swapped.buf)) };

    let last = LINES.len() as LineNr;
    let text = LINES[LINES.len() - 1];

    assert_eq!(lines.line_cstr(last, 0).to_bytes(), text.as_bytes());
    // From a column on: the tail, still terminated.
    let at = text.len() - 4;
    assert_eq!(lines.line_cstr(last, at).to_bytes(), &text.as_bytes()[at..]);
    // The end of the line is the empty string, not a panic.
    assert_eq!(lines.line_cstr(last, text.len()).to_bytes(), b"");

    // The copy derefs to the bytes without the terminator, and its `as_cstr`
    // puts it back.
    let copy = lines.line_copy(last);
    assert_eq!(&*copy, text.as_bytes());
    assert_eq!(copy.as_cstr().to_bytes(), text.as_bytes());

    // A copy survives a read of another line, which is what it is for.
    let other = lines.line_copy(1);
    assert_eq!(lines.line(last), text.as_bytes());
    assert_eq!(&*other, LINES[0].as_bytes());
}

// ---------------------------------------------------------------------------
// The block tree under edits, and a swap file read back.
//
// The cases below drive the editor through commands, so they read the tree
// only through what a user can see: the lines, their byte offsets, and what
// `:recover` makes of a swap file. Each sets 'directory' to its own sandbox
// so a swap file never lands in the user's state directory, and puts it back
// on the way out.

use crate::support::Probe;

/// `count` lines `width` bytes long, each starting with its own number so a
/// line that moved shows up as the wrong number in the wrong place.
fn numbered(count: usize, width: usize, tag: char) -> Vec<String> {
    (0..count)
        .map(|i| {
            let mut line = format!("{tag}{i:05}");
            while line.len() < width {
                line.push(char::from(b'a' + (line.len() % 26) as u8));
            }
            line
        })
        .collect()
}

/// `lines` as `string(getline(1, '$'))` renders them.
fn rendered(lines: &[String]) -> String {
    let quoted: Vec<String> = lines.iter().map(|l| format!("'{l}'")).collect();
    format!("[{}]", quoted.join(", "))
}

/// The byte offset of every line and of the line past the end, as
/// `line2byte()` answers them for `lines`.
fn offsets(lines: &[String]) -> String {
    let mut at = 1;
    let mut all = Vec::with_capacity(lines.len() + 1);
    for line in lines {
        all.push(at.to_string());
        at += line.len() + 1;
    }
    all.push(at.to_string());
    format!("[{}]", all.join(", "))
}

/// Replace the current buffer's text with `lines`, one append at a time --
/// what fills a data block to the brim before the next one starts.
fn fill(fx: &Probe, lines: &[String]) {
    fx.run("silent! %delete _");
    for (n, line) in lines.iter().enumerate() {
        fx.run(&format!("call append({n}, '{line}')"));
    }
    fx.run(&format!("silent! {}delete _", lines.len() + 1));
}

/// `expr` evaluates to `want`; on a mismatch, say where the two part rather
/// than printing a buffer's worth of text twice.
#[track_caller]
fn same(fx: &Probe, expr: &str, want: &str) {
    let got = fx.eval(expr);
    if got != want {
        let at = got
            .bytes()
            .zip(want.bytes())
            .take_while(|(a, b)| a == b)
            .count();
        let from = at.saturating_sub(60);
        panic!(
            "{expr}: differs at byte {at} (lengths {} and {})\n got: {:?}\nwant: {:?}",
            got.len(),
            want.len(),
            &got[from..(at + 60).min(got.len())],
            &want[from..(at + 60).min(want.len())],
        );
    }
}

/// The buffer holds `lines`, and every offset is where the lines say.
#[track_caller]
fn holds(fx: &Probe, lines: &[String]) {
    same(fx, "getline(1, '$')", &rendered(lines));
    same(
        fx,
        "map(range(1, line('$') + 1), 'line2byte(v:val)')",
        &offsets(lines),
    );
}

/// A scratch buffer the case owns, with 'directory' pointed at the sandbox.
fn scratch(fx: &Probe) {
    fx.run("set directory=. swapfile");
    fx.run("new");
    fx.run("only!");
}

fn unscratch(fx: &Probe) {
    fx.run("silent! bwipeout!");
    fx.run("set directory& swapfile&");
    fx.home();
}

/// Insert one line at every position of a data block that is already full,
/// in front of the first line through after the last: each insert splits it,
/// and the lines that follow the insertion point move into the new block (or
/// the new line goes there alone, when it is the last).
#[test]
fn inserting_at_every_line_of_a_full_data_block() {
    let sandbox = Sandbox::dir("memline-split-every-line");
    let fx = Probe::new(sandbox.editor());
    scratch(&fx);
    // A 4 KiB page holds 38 of these, so the first block is full and the
    // rest spill into a second and third.
    let lines = numbered(80, 100, 'L');
    for at in 0..=40 {
        fill(&fx, &lines);
        let new = format!("N{at:05}{}", "z".repeat(150));
        fx.run(&format!("call append({at}, '{new}')"));
        let mut want = lines.clone();
        want.insert(at, new);
        holds(&fx, &want);
    }
    unscratch(&fx);
}

/// Enough lines that one pointer block cannot point at all their data
/// blocks: the root fills, the tree grows a level, and then deletes empty
/// whole data blocks and shrink the pointer blocks again.
#[test]
fn a_pointer_block_splits_and_shrinks() {
    let sandbox = Sandbox::dir("memline-pointer-split");
    let fx = Probe::new(sandbox.editor());
    scratch(&fx);
    // One line per data block: 400 blocks, more than the 170 entries a
    // 4 KiB pointer block holds.
    let mut lines = numbered(400, 3000, 'P');
    fill(&fx, &lines);
    holds(&fx, &lines);

    // Inserts in the middle of the tree, each into a block of its own.
    for at in [1, 170, 171, 200, 341] {
        let new = format!("I{at:05}{}", "y".repeat(2500));
        fx.run(&format!("call append({at}, '{new}')"));
        lines.insert(at, new);
    }
    holds(&fx, &lines);

    // Delete every other line from the back, freeing every other data
    // block, then a run from the front that empties whole pointer blocks.
    let mut n = lines.len();
    while n > 1 {
        fx.run(&format!("silent! {n}delete _"));
        lines.remove(n - 1);
        n -= 2;
    }
    fx.run("silent! 1,150delete _");
    lines.drain(..150);
    holds(&fx, &lines);
    unscratch(&fx);
}

/// Many short lines: past `MLCS_MAXL` so the chunk index that `line2byte`
/// and `byte2line` skip by splits, and edits at both ends of a chunk that
/// merge it again.
#[test]
fn byte_offsets_across_the_chunk_index() {
    let sandbox = Sandbox::dir("memline-chunks");
    let fx = Probe::new(sandbox.editor());
    scratch(&fx);
    let mut lines = numbered(2000, 12, 'C');
    fx.run(&format!("call setline(1, {})", rendered(&lines)));
    holds(&fx, &lines);
    for at in [0, 399, 400, 401, 799, 800, 1999] {
        let new = format!("X{at:05}");
        fx.run(&format!("call append({at}, '{new}')"));
        lines.insert(at, new);
    }
    fx.run("silent! 300,900delete _");
    lines.drain(299..900);
    holds(&fx, &lines);
    // `byte2line` of the first byte of every 97th line.
    let probe: Vec<usize> = (1..=lines.len()).step_by(97).collect();
    let mut want = Vec::new();
    let mut at = 1;
    for (i, line) in lines.iter().enumerate() {
        if probe.contains(&(i + 1)) {
            want.push(format!("{}", i + 1));
            let _ = at;
        }
        at += line.len() + 1;
    }
    let exprs: Vec<String> = probe
        .iter()
        .map(|l| format!("byte2line(line2byte({l}))"))
        .collect();
    fx.check(
        &format!("[{}]", exprs.join(", ")),
        &format!("[{}]", want.join(", ")),
    );
    unscratch(&fx);
}

/// Edit a file of many blocks, change it, `:preserve`, and read the swap
/// file back with `:recover`: the recovered buffer holds the changed lines,
/// in order, across every data block.
#[test]
fn preserve_then_recover_a_multi_block_file() {
    let sandbox = Sandbox::dir("memline-preserve-recover");
    let fx = Probe::new(sandbox.editor());
    let swap = preserved(&fx, &sandbox);
    let want = changed_lines();

    std::fs::write(sandbox.path(".f.txt.swp"), &swap).expect("the swap file back");
    fx.run("new");
    fx.run("only!");
    fx.run("silent! recover f.txt");
    same(&fx, "getline(1, '$')", &rendered(&want));
    fx.check("&modified", "1");
    fx.run("silent! bwipeout!");
    let _ = std::fs::remove_file(sandbox.path(".f.txt.swp"));
    fx.run("set directory& swapfile&");
    fx.home();
}

/// The lines `preserved` leaves in the buffer.
fn changed_lines() -> Vec<String> {
    let mut lines = numbered(300, 60, 'F');
    lines.insert(10, "inserted".into());
    lines.remove(201);
    lines[151] = "replaced".into();
    lines
}

/// Edit `f.txt` (300 lines), change it, `:preserve`, and answer the swap
/// file's bytes. The buffer is wiped, which deletes the swap file.
fn preserved(fx: &Probe, sandbox: &Sandbox) -> Vec<u8> {
    let original = numbered(300, 60, 'F');
    sandbox.write("f.txt", (original.join("\n") + "\n").as_bytes());
    fx.run("set directory=. swapfile");
    fx.run("badd f.txt");
    fx.run("buffer f.txt");
    fx.run("call append(10, 'inserted')");
    fx.run("silent! 202delete _");
    fx.run("call setline(152, 'replaced')");
    fx.run("preserve");
    let swap = std::fs::read(sandbox.path(".f.txt.swp")).expect("`:preserve` wrote a swap file");
    fx.run("silent! bwipeout! f.txt");
    swap
}

/// A swap file that block zero says was written by another build, or that
/// claims a page smaller than any swap file can have, is refused with the
/// message that says why, and the buffer is left as it was.
#[test]
fn a_swap_file_with_a_wrong_magic_or_page_size_is_refused() {
    let sandbox = Sandbox::dir("memline-bad-block-zero");
    let fx = Probe::new(sandbox.editor());
    let swap = preserved(&fx, &sandbox);

    let magic = offset_of!(ZeroBlock, b0_magic_long);
    let page = offset_of!(ZeroBlock, b0_page_size);
    let id = offset_of!(ZeroBlock, b0_id);
    let cases: [(&str, Box<dyn Fn(&mut Vec<u8>)>, &str); 3] = [
        (
            "magic",
            Box::new(move |b: &mut Vec<u8>| b[magic] ^= 0xff),
            "cannot be used on this computer",
        ),
        (
            "page size",
            Box::new(move |b: &mut Vec<u8>| {
                b[page..page + 4].copy_from_slice(&512u32.to_le_bytes())
            }),
            "has been damaged (page size is smaller than minimum value)",
        ),
        (
            "id",
            Box::new(move |b: &mut Vec<u8>| b[id] = b'x'),
            "does not look like a Nvim swap file",
        ),
    ];
    for (what, corrupt, said) in cases {
        let mut bytes = swap.clone();
        corrupt(&mut bytes);
        std::fs::write(sandbox.path(".f.txt.swp"), &bytes).expect("the swap file back");
        fx.run("new");
        fx.run("only!");
        let out = fx.output("silent! recover f.txt") + &fx.eval("v:errmsg");
        assert!(out.contains(said), "{what}: {out:?}");
        fx.check("line('$')", "1");
        fx.run("silent! bwipeout!");
        let _ = std::fs::remove_file(sandbox.path(".f.txt.swp"));
    }
    fx.run("set directory& swapfile&");
    fx.home();
}
