//! What a job's output looks like by the time `on_stdout` sees it.
//!
//! `channel/` had no in-process test: `channels_spec`, `job_spec` and
//! `terminal/channel_spec` drive whole child processes over the event loop,
//! which observes the *result* of the buffering but never the buffering.
//! `encode_list_write` — the split itself — is covered next door in
//! `eval_encode.rs`; what was not covered is `reader.rs`'s own contribution
//! to the shape of that list, which is where every "why did my line arrive in
//! two pieces" question about `jobstart()` actually lives:
//!
//! - every delivery seeds its list with an empty item, which is what the
//!   splitter's "continue the last line" branch lands in — so a delivery
//!   that begins mid-line reports that text as its *first* item, and the
//!   caller joins it onto the last item of the delivery before;
//! - the accumulator is cleared after each delivery, so a partial line is
//!   *not* held back in the buffer waiting for its newline;
//! - a `\r` is an ordinary byte, so a child writing CRLF delivers lines that
//!   still carry the carriage return;
//! - a NUL in the stream arrives as a newline *inside* an item, which is how
//!   a list of lines carries a byte it otherwise uses as its separator.
//!
//! The sequence below — `callback_reader_start`, `ga_concat_len` per chunk,
//! `reader_lines` then clearing the buffer per delivery — is exactly what
//! `on_channel_output` and `channel_callback_call` do around a live channel,
//! minus the channel: a `Channel` needs a stream, a multiqueue and a
//! `Callback` to reach the same two lines, and the event loop that drives
//! them is what the functional specs already cover.
//!
//! The last two cases are the other half of that: not what the list *says*
//! but what happens to it afterwards. `channel_callback_call` references the
//! list around the callback and drops that reference again, so a callback
//! that stored nothing leaves nothing behind — which is only observable as
//! the garbage collector's registry of live lists coming back to what it
//! was.

#![cfg(not(miri))]

use neovim::channel::reader::{callback_reader_free, callback_reader_start, reader_lines};
use neovim::eval::gc::rooted_lists;
use neovim::eval::typval::tv_list_alloc;
use neovim::types::{CallbackReader, kListLenUnknown};

use crate::support::tv::{self, Tv};
use crate::support::{Sandbox, cstr};

/// A reader with nothing in it, set up as `channel_alloc` sets one up.
fn reader() -> CallbackReader {
    CallbackReader::none()
}

/// Feed `chunks` to a reader and answer what each *delivery* hands the
/// callback: one `on_channel_output` per chunk, each followed by one
/// `channel_callback_call`.
fn delivered(chunks: &[&[u8]]) -> Vec<Tv> {
    let _sandbox = Sandbox::globals();
    let stdout = cstr("stdout");
    let mut reader = reader();
    let at = &raw mut reader;
    let mut out = Vec::new();
    // SAFETY: `reader` is this frame's, started and freed here; each chunk is
    // readable for its own length; each list is referenced for as long as it
    // is read and released straight after. The editor lock is held for the
    // list allocator.
    unsafe {
        callback_reader_start(at, stdout.as_ptr());
        for chunk in chunks {
            // `on_channel_output`: accept the bytes into the accumulator.
            (*at).buffer.extend_from_slice(chunk);
            // `channel_callback_call`: build the list, then drop what it was
            // built from.
            let list = reader_lines(at);
            out.push(tv::read_list(list.as_ptr()));
            drop(list);
            (*at).buffer.clear();
        }
        callback_reader_free(at);
    }
    out
}

/// A line with text in it.
fn line(text: &str) -> Tv {
    Tv::s(text)
}

/// The empty line a *newline* opens. `tv_list_append_allocated_string` takes
/// a null for it, so it reads back as a NULL string and not as `""` — the
/// two are the same line to Vimscript but not to `assert_eq!`.
fn opened() -> Tv {
    Tv::NullStr
}

/// The empty line `reader_lines` seeds every delivery with. This one is a
/// real empty string, and it survives only when the delivery's first byte is
/// a newline; otherwise the splitter extends it into the delivery's first
/// line.
fn seeded() -> Tv {
    Tv::s("")
}

#[test]
fn a_delivery_that_ends_on_a_newline_opens_an_empty_last_line() {
    // Two complete lines and the empty one the trailing newline opened.
    assert_eq!(
        delivered(&[b"one\ntwo\n"]),
        [Tv::List(vec![line("one"), line("two"), opened()])]
    );
}

#[test]
fn a_delivery_that_ends_mid_line_reports_only_what_arrived() {
    // No empty item at the end: the line is not finished, and the reader
    // says so by not opening the next one.
    assert_eq!(
        delivered(&[b"one\ntw"]),
        [Tv::List(vec![line("one"), line("tw")])]
    );
}

#[test]
fn a_line_split_across_two_deliveries_is_joined_by_the_leading_empty_item() {
    // The buffer is cleared after the first delivery, so "hello" is *not*
    // held back waiting for its newline: "hel" arrives as a partial last
    // line, and "lo" arrives as the next delivery's first line because the
    // splitter extended the seeded empty item into it.
    let got = delivered(&[b"hel", b"lo\nrest"]);
    assert_eq!(
        got,
        [
            Tv::List(vec![line("hel")]),
            Tv::List(vec![line("lo"), line("rest")]),
        ]
    );

    // Which is what makes `on_stdout`'s documented "concatenate the first
    // item onto the last one you had" reproduce the stream.
    assert_eq!(format!("{}{}", "hel", "lo"), "hello");
}

#[test]
fn a_carriage_return_stays_in_the_line_it_ended() {
    // A child writing CRLF is not special-cased anywhere on this path.
    assert_eq!(
        delivered(&[b"one\r\ntwo\r\n"]),
        [Tv::List(vec![line("one\r"), line("two\r"), opened()])]
    );
}

#[test]
fn a_nul_in_the_stream_arrives_as_a_newline_inside_an_item() {
    // The `readfile()` convention: a list of lines carries an embedded NUL
    // as the newline it cannot otherwise hold.
    assert_eq!(
        delivered(&[b"a\0b\n"]),
        [Tv::List(vec![line("a\nb"), opened()])]
    );
    // And a NUL is not a separator, so this is one item and not two.
    assert_eq!(delivered(&[b"a\0b"]), [Tv::List(vec![line("a\nb")])]);
}

#[test]
fn a_delivery_of_nothing_is_the_one_empty_item() {
    // What EOF hands an unbuffered reader's callback: the list is never
    // empty, because the empty first item is unconditional.
    assert_eq!(delivered(&[b""]), [Tv::List(vec![seeded()])]);
}

#[test]
fn a_chunk_that_starts_with_a_newline_keeps_the_previous_line_closed() {
    // The empty first item is what the newline closes, so nothing is
    // appended to the caller's previous last line.
    assert_eq!(delivered(&[b"\nx"]), [Tv::List(vec![seeded(), line("x")])]);
}

#[test]
fn the_accumulator_is_a_byte_buffer_however_the_chunks_fall() {
    // One line arriving one byte at a time, delivered only at the end, is
    // the same line as one that arrived whole.
    let _sandbox = Sandbox::globals();
    let stdout = cstr("stdout");
    let mut reader = reader();
    let at = &raw mut reader;
    // SAFETY: as `delivered`, with the delivery moved out of the loop.
    let got = unsafe {
        callback_reader_start(at, stdout.as_ptr());
        for byte in b"one\ntwo" {
            (*at).buffer.push(*byte);
        }
        assert_eq!((*at).buffer.len(), "one\ntwo".len());
        let list = reader_lines(at);
        let got = tv::read_list(list.as_ptr());
        drop(list);
        (*at).buffer.clear();
        callback_reader_free(at);
        got
    };
    assert_eq!(got, Tv::List(vec![line("one"), line("two")]));
}

/// `test/functional/core/job_spec.lua`'s "lists passed to callbacks are freed
/// if not stored" (`#25891`), which used to `ffi.cdef` `gc_first_list`,
/// `tv_list_alloc` and `tv_list_free` and read the chain head across a live
/// `jobstart()`. Nothing about that assertion needed the child process: what
/// it watched was one list allocated *before* the deliveries still being the
/// most recently allocated one *after* them, which says every list built for
/// a callback in between was freed again.
///
/// The registry is the only place a leaked list is visible. It is not
/// reachable from any root once the callback returns, so `garbagecollect()`
/// would collect it and no Vimscript, Lua or API call can see it — which is
/// why the spec reached past the API in the first place, and why the
/// replacement is here rather than there.
#[test]
fn a_delivered_list_is_freed_again_when_the_callback_stores_nothing() {
    let _sandbox = Sandbox::globals();
    let stdout = cstr("stdout");
    let mut reader = reader();
    let at = &raw mut reader;
    // SAFETY: as `delivered`. The sentinel is this case's own: it is never
    // referenced, so nothing but the final `tv_list_free` can free it.
    unsafe {
        let before = rooted_lists();
        let held = tv_list_alloc(kListLenUnknown as isize);
        let sentinel = held.as_ptr();
        let with_sentinel = rooted_lists();
        assert_eq!(with_sentinel.len(), before.len() + 1);
        assert!(
            with_sentinel.contains(&sentinel),
            "a fresh list is registered"
        );

        callback_reader_start(at, stdout.as_ptr());
        for chunk in [&b"one\ntwo\n"[..], b"three\n", b""] {
            (*at).buffer.extend_from_slice(chunk);
            // `channel_callback_call` around a callback that reads the list
            // and keeps no reference to it.
            let list = reader_lines(at);
            let _ = tv::read_list(list.as_ptr());
            drop(list);
            (*at).buffer.clear();
            assert_eq!(
                rooted_lists(),
                with_sentinel,
                "the delivered list outlived its callback",
            );
        }
        callback_reader_free(at);

        drop(held);
        assert!(
            !rooted_lists().contains(&sentinel),
            "freeing a list leaves it registered",
        );
    }
}

/// The teeth of the case above: a callback that *does* store the list keeps
/// it alive, and the registry says so. Without this, a `reader_lines` that
/// forgot to register its list at all would pass the first case.
#[test]
fn a_delivered_list_the_callback_stored_stays_on_the_chain() {
    let _sandbox = Sandbox::globals();
    let stdout = cstr("stdout");
    let mut reader = reader();
    let at = &raw mut reader;
    // SAFETY: as above. The stored reference is released before the list is
    // freed, so this case owns everything it allocates.
    unsafe {
        let held = tv_list_alloc(kListLenUnknown as isize);
        let sentinel = held.as_ptr();
        let with_sentinel = rooted_lists();
        assert!(with_sentinel.contains(&sentinel));

        callback_reader_start(at, stdout.as_ptr());
        (*at).buffer.extend_from_slice(b"one\n");
        // The reference `channel_callback_call` holds for the length of the
        // callback.
        let list = reader_lines(at);
        let at_list = list.as_ptr();
        // The callback stores it -- `let g:saved = a:data`, one more
        // reference than `channel_callback_call` is about to drop.
        let stored = list.clone();
        drop(list);
        (*at).buffer.clear();
        callback_reader_free(at);

        assert!(
            rooted_lists().contains(&at_list),
            "a stored list was freed anyway"
        );
        assert_eq!(
            tv::read_list(at_list),
            Tv::List(vec![line("one"), opened()])
        );

        drop(stored);
        assert_eq!(rooted_lists(), with_sentinel);
        drop(held);
    }
}
