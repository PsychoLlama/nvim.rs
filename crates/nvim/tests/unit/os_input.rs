//! Reading stdin: stopping and starting the read stream.
//!
//! Each case swaps a pipe in for the process's stdin, which is why it needs
//! nextest's process per case.

#![cfg(not(miri))]

use std::ffi::c_int;

use neovim::os::input::{input_available, input_start, input_stop, os_char_avail};

use crate::support::editor_lock;

/// Make a fresh pipe the process's stdin; answers the write end.
fn pipe_on_stdin() -> c_int {
    let mut ends: [c_int; 2] = [-1; 2];
    // SAFETY: `pipe` fills in the two descriptors of a pair this case owns;
    // `dup2` replaces descriptor 0, which nothing else in the process reads.
    unsafe {
        assert_eq!(libc::pipe(ends.as_mut_ptr()), 0, "a pipe");
        assert_eq!(libc::dup2(ends[0], 0), 0, "the pipe's read end on stdin");
        libc::close(ends[0]);
    }
    ends[1]
}

/// Write `bytes` to `fd` whole.
fn write_all(fd: c_int, bytes: &[u8]) {
    // SAFETY: a descriptor this case owns, and a live buffer of that length.
    let written = unsafe { libc::write(fd, bytes.as_ptr().cast(), bytes.len()) };
    assert_eq!(usize::try_from(written).ok(), Some(bytes.len()));
}

/// Poll the loop until `want` bytes are buffered, or give up.
fn wait_for_input(want: usize) -> usize {
    for _ in 0..1000 {
        if input_available() >= want {
            break;
        }
        os_char_avail();
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    input_available()
}

/// A stop closes the stream's handle on a later loop turn; a start before
/// then waits for that close instead of reinitialising a handle libuv is
/// still closing (whose close callback would then have released the new
/// stream's block).
#[test]
fn stdin_started_again_before_its_close_finishes_reads_again() {
    let _editor = editor_lock();
    let writer = pipe_on_stdin();
    input_start();
    input_stop();
    input_start();
    write_all(writer, b"abc");
    assert_eq!(wait_for_input(3), 3, "the reopened stream reads");
    input_stop();
    // SAFETY: the write end this case opened.
    unsafe { libc::close(writer) };
}
