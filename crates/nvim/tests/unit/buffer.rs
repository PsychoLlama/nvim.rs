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
use neovim::types::Buffer;
use neovim::winlayer::{Buf, BufId};

use crate::support::{Probe, Sandbox, cstr};

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
    opened: std::cell::RefCell<Vec<*mut Buffer>>,
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
    fn open(&self, name: &str) -> *mut Buffer {
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
        let buf = buf.expect("buflist_new answers a buffer").raw();
        self.opened.borrow_mut().push(buf);
        buf
    }

    /// `close_buffer(NULL, buf, action, 0, 0)`.
    fn close(&self, buffer: *mut Buffer, action: c_int) {
        // SAFETY: a buffer this case opened and has not yet wiped, and a
        // null window -- the spec's own call.
        unsafe { close_buffer(None, Buf::new(buffer), action, false, false) };
        if action == DOBUF_WIPE as c_int {
            self.opened.borrow_mut().retain(|&b| b != buffer);
        }
    }

    /// The handle of a live buffer.
    fn handle(&self, buffer: *mut Buffer) -> c_int {
        // SAFETY: a buffer this case opened and has not wiped.
        unsafe { (*buffer).handle }
    }

    /// `buf_valid`, which is the whole of the first `describe` block. It
    /// takes the *identity*, so a case that wipes the buffer first has to
    /// have taken one while it was live -- which [`Buffers::id`] does.
    fn valid(&self, buffer: Option<BufId>) -> bool {
        buffer.is_some_and(buf_valid)
    }

    /// The identity of a buffer that is live now.
    fn id(&self, buffer: *mut Buffer) -> Option<BufId> {
        // SAFETY: a buffer this case opened and has not wiped.
        Some(unsafe { Buf::new(buffer) }.id())
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
fn no_buffer_is_not_valid() {
    let bufs = Buffers::new("valid-null");
    assert!(!bufs.valid(None));
}

#[test]
fn an_open_buffer_is_valid() {
    let bufs = Buffers::new("valid-open");
    let buf = bufs.id(bufs.open(PATH1));
    assert!(bufs.valid(buf));
}

/// Hiding a buffer (action 0) leaves it in the list.
#[test]
fn a_hidden_buffer_is_still_valid() {
    let bufs = Buffers::new("valid-hidden");
    let buf = bufs.open(PATH1);
    let id = bufs.id(buf);
    bufs.close(buf, 0);
    assert!(bufs.valid(id));
}

/// Unloading frees the buffer's contents but not the buffer.
#[test]
fn an_unloaded_buffer_is_still_valid() {
    let bufs = Buffers::new("valid-unloaded");
    let buf = bufs.open(PATH1);
    let id = bufs.id(buf);
    bufs.close(buf, DOBUF_UNLOAD as c_int);
    assert!(bufs.valid(id));
}

/// Wiping is the one action that takes the buffer out of the list, which is
/// the only thing `buf_valid` looks at.
#[test]
fn a_wiped_buffer_is_not_valid() {
    let bufs = Buffers::new("valid-wiped");
    let buf = bufs.open(PATH1);
    let id = bufs.id(buf);
    bufs.close(buf, DOBUF_WIPE as c_int);
    assert!(!bufs.valid(id));
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

// ---------------------------------------------------------------------------
// :ls, CTRL-G and the ruler's fragments, through the editor

/// What `cmd` prints with what depends on the case's order masked: a buffer
/// number (`:ls`'s `%3d` column keeps its width as `NNN`, CTRL-G's as `N`),
/// the sandbox's path, and `:ls t`'s times.
fn masked(fx: &Probe, sandbox: &Sandbox, cmd: &str) -> String {
    let quoted = cmd.replace('\'', "''");
    let expr = format!(
        "substitute(substitute(substitute(execute('{quoted}'), \
         '\\n\\zs[ 0-9]\\{{3}}', 'NNN', 'g'), 'buf \\d\\+:', 'buf N:', 'g'), \
         '\\d\\+ seconds\\? ago', 'N ago', 'g')"
    );
    let text = fx.eval(&expr);
    format!("{cmd} => {}", text.replace(sandbox.as_str(), "<root>"))
}

#[track_caller]
fn verify(rows: &[String], want: &str) {
    let got = rows.join("\n");
    if let Some(dir) = std::env::var_os("NVIM_UNIT_RECORD") {
        let name = std::thread::current()
            .name()
            .unwrap_or("case")
            .replace("::", "-");
        std::fs::write(std::path::Path::new(&dir).join(name), &got).expect("recording");
        return;
    }
    assert_eq!(got, want.trim_matches('\n'), "\n{got}\n");
}

/// `:ls` with each flag column set, each filter, a name long enough to push
/// the line number out of column 40, and the most-recently-used order.
#[test]
fn ls_shows_every_flag_column_and_filter() {
    let sandbox = Sandbox::dir("buffer-ls");
    let fx = Probe::new(sandbox.editor());
    fx.home();
    let long = "a_name_long_enough_to_reach_past_column_forty.txt";
    for cmd in [
        "set hidden",
        "badd a.txt",
        "badd b.txt",
        "badd c.txt",
        "badd u.txt",
        &format!("badd {long}"),
        "buffer a.txt",
        "call setline(1, ['x', 'y', 'z'])",
        "3",
        "buffer b.txt",
        "setlocal nomodifiable",
        "split",
        "buffer c.txt",
        "setlocal readonly",
        "bdelete u.txt",
    ] {
        fx.run(cmd);
    }
    let rows: Vec<String> = [
        "ls",
        "ls!",
        "ls +",
        "ls h",
        "ls a",
        "ls -",
        "ls =",
        "ls u",
        "ls %",
        "ls #",
        "ls x",
        "ls R",
        "ls F",
        "ls a+",
        "filter /b\\.txt/ ls",
        "filter! /\\.txt/ ls!",
        "ls t%",
    ]
    .iter()
    .map(|cmd| masked(&fx, &sandbox, cmd))
    .collect();
    verify(
        &rows,
        r#"
ls => '
NNN  h   "[No Name]"                    line 1
NNN  h + "a.txt"                        line 3
NNN #a-  "b.txt"                        line 1
NNN %a=  "c.txt"                        line 1
NNN      "a_name_long_enough_to_reach_past_column_forty.txt" line 0'
ls! => '
NNN  h   "[No Name]"                    line 1
NNN  h + "a.txt"                        line 3
NNN #a-  "b.txt"                        line 1
NNN %a=  "c.txt"                        line 1
NNNu     "u.txt"                        line 1
NNN      "a_name_long_enough_to_reach_past_column_forty.txt" line 0'
ls + => '
NNN  h + "a.txt"                        line 3'
ls h => '
NNN  h   "[No Name]"                    line 1
NNN  h + "a.txt"                        line 3'
ls a => '
NNN #a-  "b.txt"                        line 1
NNN %a=  "c.txt"                        line 1'
ls - => '
NNN #a-  "b.txt"                        line 1'
ls = => '
NNN %a=  "c.txt"                        line 1'
ls u => '
NNNu     "u.txt"                        line 1'
ls % => '
NNN %a=  "c.txt"                        line 1'
ls # => '
NNN #a-  "b.txt"                        line 1'
ls x => ''
ls R => ''
ls F => ''
ls a+ => ''
filter /b\.txt/ ls => '
NNN #a-  "b.txt"                        line 1'
filter! /\.txt/ ls! => '
NNN  h   "[No Name]"                    line 1'
ls t% => '
NNN %a=  "c.txt"                        N ago'
"#,
    );
    fx.run("set hidden&");
    fx.home();
}

/// CTRL-G / `:file`: the name, the flag notes, the position with and without
/// 'ruler', the column pair, the argument number and the buffer number.
#[test]
fn file_info_message() {
    let sandbox = Sandbox::dir("buffer-fileinfo");
    sandbox.write("f.txt", b"one\n\ttwo\nthree\nfour\n");
    let fx = Probe::new(sandbox.editor());
    fx.home();
    let mut rows = Vec::new();
    for setup in [
        "badd f.txt | buffer f.txt",
        "set noruler",
        "2 | normal! $",
        "call setline(1, 'changed')",
        "set shortmess+=m",
        "setlocal readonly",
        "set shortmess+=r",
        "set shortmess-=m shortmess-=r ruler | setlocal noreadonly",
        "file renamed.txt",
        "enew | file missing.txt",
        "arglocal f.txt g.txt | argument 1",
    ] {
        fx.run(setup);
        rows.push(format!("{setup}: {}", masked(&fx, &sandbox, "file")));
    }
    rows.push(masked(&fx, &sandbox, "normal! 2\x07"));
    rows.push(masked(&fx, &sandbox, "normal! \x07"));
    verify(
        &rows,
        r#"
badd f.txt | buffer f.txt: file => '
"f.txt" 4 lines --25%--'
set noruler: file => '
"f.txt" line 1 of 4 --25%-- col 1'
2 | normal! $: file => '
"f.txt" line 2 of 4 --50%-- col 4-11'
call setline(1, 'changed'): file => '
"f.txt" [Modified] line 2 of 4 --50%-- col 4-11'
set shortmess+=m: file => '
"f.txt" [+] line 2 of 4 --50%-- col 4-11'
setlocal readonly: file => '
"f.txt" [+][readonly] line 2 of 4 --50%-- col 4-11'
set shortmess+=r: file => '
"f.txt" [+][RO] line 2 of 4 --50%-- col 4-11'
set shortmess-=m shortmess-=r ruler | setlocal noreadonly: file => '
"f.txt" [Modified] 4 lines --50%--'
file renamed.txt: file => '
"renamed.txt" [Modified][Not edited] 4 lines --50%--'
enew | file missing.txt: file => '
"missing.txt" [Not edited] --No lines in buffer--'
arglocal f.txt g.txt | argument 1: file => '
"f.txt" 4 lines --50%-- (1 of 2)'
normal! 2 => '

buf N: "<root>/f.txt" 4 lines --50%-- (1 of 2)'
normal!  => '

"f.txt" 4 lines --50%-- (1 of 2)'
"#,
    );
    fx.run("argglobal | set ruler& shortmess&");
    fx.home();
}

/// The ruler's `All`/`Top`/`Bot`/percentage and its column pair.
#[test]
fn ruler_position_fragments() {
    let sandbox = Sandbox::dir("buffer-relpos");
    let fx = Probe::new(sandbox.editor());
    fx.home();
    fx.run("resize 10 | resize 22");
    let at = |cmd: &str, want: &str| {
        fx.run(cmd);
        fx.check("nvim_eval_statusline('%P %c%V %l', {})['str']", want);
    };
    at("call setline(1, ['a'])", "'All 1 1'");
    at("call setline(1, map(range(1, 100), 'v:val'))", "'Top 1 1'");
    at("normal! 50Gzt", "'38% 1 50'");
    at("normal! G", "'72% 1 100'");
    at("call setline(100, \"\\tx\") | normal! G$", "'Bot 2-1 100'");
    fx.home();
}
