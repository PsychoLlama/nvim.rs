//! The buffer list: what counts as a live buffer, and which one a pattern
//! names.
//!
//! A port of `test/unit/buffer_spec.lua`.
//!
//! Two things this file has to do that the LuaJIT harness did not. The
//! buffer list is one process-wide linked list, so every case here takes the
//! editor lock and **wipes every buffer it created before it returns** —
//! `itp` forked a child per case and could leak them. And `buflist_new`
//! resolves the name it is given against the working directory and stats it,
//! so the fixture files live in a private directory the case stands in.

#![cfg(not(miri))]

use std::ffi::{c_char, c_int};

use neovim::buffer::{
    BLN_LISTED, DOBUF_DEL, DOBUF_UNLOAD, DOBUF_WIPE, buf_valid, buflist_findpat, buflist_new,
    close_buffer,
};
use neovim::types::buf_T;
use neovim::winlayer::Buf;

use crate::support::{Sandbox, cstr};

/// The three names every `buflist_findpat` case below is written against.
/// Each contains all three of `test`, `file` and `path`, in a different
/// order, so a case can say which part of a name a match came from.
const PATH1: &str = "test_file_path";
const PATH2: &str = "file_path_test";
const PATH3: &str = "path_test_file";

/// `buflist_findpat`'s `unlisted` argument, named.
const ONLY_LISTED: bool = false;
const ALLOW_UNLISTED: bool = true;

/// The editor lock, a private directory holding the three fixture files, and
/// a record of every buffer the case opened so the drop can wipe them.
///
/// Wiping is not tidiness: a buffer left in the list is visible to the next
/// case's `buflist_findpat`, and all three fixture names match all three
/// patterns.
struct Buffers {
    /// Held for its drop: the lock, the directory and the fixtures.
    _sandbox: Sandbox,
    opened: std::cell::RefCell<Vec<*mut buf_T>>,
}

impl Buffers {
    fn new(name: &str) -> Buffers {
        let sandbox = Sandbox::dir(&format!("buffer-{name}"));
        for path in [PATH1, PATH2, PATH3] {
            sandbox.touch(path);
        }
        Buffers {
            _sandbox: sandbox,
            opened: std::cell::RefCell::new(Vec::new()),
        }
    }

    /// `buflist_new(name, name, 1, BLN_LISTED)`, remembered for the wipe.
    ///
    /// The spec passed the same pointer as both the full and the short name,
    /// which is what a caller with a bare relative name does.
    fn open(&self, name: &str) -> *mut buf_T {
        let mut owned: Vec<c_char> = cstr(name)
            .as_bytes_with_nul()
            .iter()
            .map(|&b| b as c_char)
            .collect();
        // SAFETY: `owned` is this frame's, NUL-terminated and writable --
        // `fname_expand` reads it and may replace the pointer's target with
        // an allocation of its own, which it owns.
        let buf = unsafe {
            buflist_new(
                owned.as_mut_ptr(),
                owned.as_mut_ptr(),
                1,
                BLN_LISTED as c_int,
            )
        };
        assert!(!buf.is_null(), "buflist_new({name:?})");
        self.opened.borrow_mut().push(buf);
        buf
    }

    /// `close_buffer(NULL, buf, action, 0, 0)`.
    fn close(&self, buf: *mut buf_T, action: c_int) {
        // SAFETY: a buffer this case opened and has not yet wiped, and a
        // null window -- the spec's own call.
        unsafe { close_buffer(None, Buf::new(buf), action, false, false) };
        if action == DOBUF_WIPE as c_int {
            self.opened.borrow_mut().retain(|&b| b != buf);
        }
    }

    /// The handle of a live buffer.
    fn handle(&self, buf: *mut buf_T) -> c_int {
        // SAFETY: a buffer this case opened and has not wiped.
        unsafe { (*buf).handle }
    }

    /// `buf_valid`, which is the whole of the first `describe` block.
    fn valid(&self, buf: *mut buf_T) -> bool {
        // SAFETY: `buf_valid` walks the list and compares addresses; it
        // never dereferences the pointer it is given.
        unsafe { buf_valid(buf) }
    }

    /// `buflist_findpat(pat, NULL, unlisted, 0, 0)` — the buffer's handle,
    /// `-1` for no match or `-2` for several.
    fn findpat(&self, pattern: &str, unlisted: bool) -> c_int {
        let pattern = cstr(pattern);
        // SAFETY: `pattern` is this frame's and NUL-terminated; a null end
        // pointer means "to the terminator", which is the spec's call.
        unsafe { buflist_findpat(pattern.as_ptr(), std::ptr::null(), unlisted, false, false) }
    }
}

impl Drop for Buffers {
    fn drop(&mut self) {
        for buf in self.opened.borrow_mut().drain(..) {
            // SAFETY: a buffer this case opened; wiping it is what the spec
            // did at the end of every case that could.
            unsafe { close_buffer(None, Buf::new(buf), DOBUF_WIPE as c_int, false, false) };
        }
    }
}

// ---------------------------------------------------------------------------
// buf_valid

#[test]
fn a_null_buffer_is_not_valid() {
    let bufs = Buffers::new("valid-null");
    assert!(!bufs.valid(std::ptr::null_mut()));
}

#[test]
fn an_open_buffer_is_valid() {
    let bufs = Buffers::new("valid-open");
    let buf = bufs.open(PATH1);
    assert!(bufs.valid(buf));
}

/// Hiding a buffer (action 0) leaves it in the list.
#[test]
fn a_hidden_buffer_is_still_valid() {
    let bufs = Buffers::new("valid-hidden");
    let buf = bufs.open(PATH1);
    bufs.close(buf, 0);
    assert!(bufs.valid(buf));
}

/// Unloading frees the buffer's contents but not the buffer.
#[test]
fn an_unloaded_buffer_is_still_valid() {
    let bufs = Buffers::new("valid-unloaded");
    let buf = bufs.open(PATH1);
    bufs.close(buf, DOBUF_UNLOAD as c_int);
    assert!(bufs.valid(buf));
}

/// Wiping is the one action that takes the buffer out of the list, which is
/// the only thing `buf_valid` looks at.
#[test]
fn a_wiped_buffer_is_not_valid() {
    let bufs = Buffers::new("valid-wiped");
    let buf = bufs.open(PATH1);
    bufs.close(buf, DOBUF_WIPE as c_int);
    assert!(!bufs.valid(buf));
}

// ---------------------------------------------------------------------------
// buflist_findpat

#[test]
fn a_whole_name_matches_itself() {
    let bufs = Buffers::new("findpat-exact");
    let buf = bufs.open(PATH1);
    assert_eq!(bufs.findpat(PATH1, ONLY_LISTED), bufs.handle(buf));
}

/// Of the four attempts `buflist_findpat` makes, the anchored ones come
/// first, so a fragment at the start of a name beats the same fragment in
/// the middle or at the end.
#[test]
fn a_match_at_the_start_of_a_name_wins() {
    let bufs = Buffers::new("findpat-start");
    let buf1 = bufs.open(PATH1);
    let buf2 = bufs.open(PATH2);
    let buf3 = bufs.open(PATH3);
    assert_eq!(bufs.findpat("test", ONLY_LISTED), bufs.handle(buf1));
    assert_eq!(bufs.findpat("file", ONLY_LISTED), bufs.handle(buf2));
    assert_eq!(bufs.findpat("path", ONLY_LISTED), bufs.handle(buf3));
}

/// With no name starting with the fragment, the one *ending* in it wins over
/// the one carrying it in the middle.
#[test]
fn a_match_at_the_end_beats_one_in_the_middle() {
    let bufs = Buffers::new("findpat-end");
    // `test` ends `file_path_test` and sits in the middle of `path_test_file`.
    let buf2 = bufs.open(PATH2);
    let buf3 = bufs.open(PATH3);
    assert_eq!(bufs.findpat("test", ONLY_LISTED), bufs.handle(buf2));

    // With that one gone, and `file` now ending `path_test_file` while it
    // sits in the middle of `test_file_path`, the same rule picks buf3.
    bufs.close(buf2, DOBUF_WIPE as c_int);
    bufs.open(PATH1);
    assert_eq!(bufs.findpat("file", ONLY_LISTED), bufs.handle(buf3));
}

/// A fragment only one name carries matches it wherever it sits.
#[test]
fn a_fragment_unique_to_one_name_matches_it() {
    let bufs = Buffers::new("findpat-unique");
    bufs.open(PATH1);
    bufs.open(PATH2);
    let buf3 = bufs.open(PATH3);
    assert_eq!(bufs.findpat("_test_", ONLY_LISTED), bufs.handle(buf3));
}

/// `unlisted` is the second pass: the unlisted buffers are searched only
/// when it is set *and* no listed buffer matched.
#[test]
fn unlisted_buffers_are_searched_only_when_asked_for() {
    let bufs = Buffers::new("findpat-unlisted");
    let buf3 = bufs.open(PATH3);
    let handle3 = bufs.handle(buf3);
    assert_eq!(bufs.findpat("_test_", ONLY_LISTED), handle3);

    // `:bdelete` unlists the buffer without wiping it.
    bufs.close(buf3, DOBUF_DEL as c_int);
    assert_eq!(bufs.findpat("_test_", ONLY_LISTED), -1);
    assert_eq!(bufs.findpat("_test_", ALLOW_UNLISTED), handle3);

    // Wiping takes it out of the list altogether.
    bufs.close(buf3, DOBUF_WIPE as c_int);
    assert_eq!(bufs.findpat("_test_", ONLY_LISTED), -1);
    assert_eq!(bufs.findpat("_test_", ALLOW_UNLISTED), -1);
}

/// A listed buffer beats an unlisted one even when the unlisted one is the
/// better match; between two of the same listing status, the better match
/// wins.
#[test]
fn a_listed_buffer_beats_a_better_unlisted_match() {
    let bufs = Buffers::new("findpat-listed-first");
    let buf1 = bufs.open(PATH1);
    let buf2 = bufs.open(PATH2);
    let (handle1, handle2) = (bufs.handle(buf1), bufs.handle(buf2));

    // `test` starts buf1 and ends buf2, so buf1 is the better match.
    assert_eq!(bufs.findpat("test", ONLY_LISTED), handle1);

    // Unlist buf1 and only buf2 is left among the listed.
    bufs.close(buf1, DOBUF_DEL as c_int);
    assert_eq!(bufs.findpat("test", ONLY_LISTED), handle2);
    // ... and allowing unlisted does not resurrect buf1, because the
    // unlisted pass only runs when the listed one found nothing.
    assert_eq!(bufs.findpat("test", ALLOW_UNLISTED), handle2);

    // With both unlisted, the better match wins again.
    bufs.close(buf2, DOBUF_DEL as c_int);
    assert_eq!(bufs.findpat("test", ALLOW_UNLISTED), handle1);
    assert_eq!(bufs.findpat("test", ONLY_LISTED), -1);
}
