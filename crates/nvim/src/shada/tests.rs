//! The reader and the writer against each other, and the keyset reader
//! against foreign bytes, with no editor behind them: what `parse` makes of
//! a payload and what `pack` makes of an entry are pure, which is what lets
//! these run under Miri.
//!
//! The hand-built entries are spelled the way the writer spells them (its
//! key order, its widths), so reading one and packing it again has to give
//! the same bytes back -- extra keys and elements included.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::pack::{Packer, pack_entry};
use super::unpack::{Cursor, Source};
use super::*;
use crate::types::{KeyDict__shada_mark, KeyDict__shada_register};
use core::ffi::CStr;
use std::ffi::CString;

/// One entry: kind, timestamp 7, and a fixint length.
fn entry(kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = vec![
        kind,
        7,
        u8::try_from(payload.len()).expect("a short payload"),
    ];
    out.extend_from_slice(payload);
    out
}

/// Every entry `bytes` holds, every kind wanted.
fn read_all(bytes: &[u8]) -> Vec<ShadaEntry> {
    let mut source = Source::Bytes {
        data: bytes,
        read: 0,
    };
    let mut found = Vec::new();
    loop {
        match source.next_entry(ReadKinds::MAX, 0) {
            Ok(entry) => found.push(entry),
            Err(status) => {
                assert_eq!(status, ReadStatus::Finished);
                return found;
            }
        }
    }
}

/// The entries packed again, as the writer would.
fn pack_all(entries: &[ShadaEntry]) -> Vec<u8> {
    let mut out = Packer::default();
    for entry in entries {
        assert_eq!(pack_entry(&mut out, entry, 0), WriteStatus::Successful);
    }
    out.take()
}

/// Each kind the reader keeps, with a key or element it has no field for
/// (the `compatibility_spec` cases): a search pattern's `sX`, a
/// replacement's trailing value, the marks' `mX` and a `<C-a>` name, a
/// register of type 10 with `rX`, a buffer list item's `bX`, a history line
/// of type 10 and a search one with a trailing value, and a kind-100 item.
fn foreign_file() -> Vec<u8> {
    let mut file = Vec::new();
    file.extend(entry(2, b"\x82\xa2sp\xc4\x03pat\xa2sX\xc3"));
    file.extend(entry(3, b"\x92\xc4\x01r\xc3"));
    file.extend(entry(7, b"\x83\xa1f\xc4\x02/x\xa1n\x01\xa2mX\xc3"));
    file.extend(entry(8, b"\x83\xa1f\xc4\x02/x\xa1l\x05\xa2mX\xc3"));
    file.extend(entry(10, b"\x83\xa1f\xc4\x02/x\xa1n\x61\xa2mX\xc3"));
    file.extend(entry(
        11,
        b"\x84\xa1f\xc4\x02/x\xa1l\x02\xa1c\x03\xa2mX\xc3",
    ));
    file.extend(entry(
        5,
        b"\x84\xa2rc\x91\xc4\x01x\xa1n\x61\xa2rt\x0a\xa2rX\xc3",
    ));
    file.extend(entry(9, b"\x91\x83\xa1f\xc4\x02/b\xa1l\x03\xa2bX\xc3"));
    file.extend(entry(4, b"\x93\x0a\xc4\x01h\xc3"));
    file.extend(entry(4, b"\x94\x01\xc4\x01s\x2f\xc3"));
    file.extend(entry(100, b"\x91\x01"));
    file
}

#[test]
fn foreign_entries_round_trip_byte_for_byte() {
    let file = foreign_file();
    let entries = read_all(&file);
    let kinds: Vec<u64> = entries.iter().map(ShadaEntry::kind).collect();
    assert_eq!(kinds, [2, 3, 7, 8, 10, 11, 5, 9, 4, 4, 100]);
    // Every one but the buffer list (whose item holds it) and the unknown
    // item (which is all payload) kept one extra item.
    let extras: Vec<u32> = entries
        .iter()
        .map(|e| e.extra.as_ref().map_or(0, ShadaExtra::items))
        .collect();
    assert_eq!(extras, [1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0]);
    assert_eq!(pack_all(&entries), file);
}

#[test]
fn what_the_parser_reads_out_of_each_kind() {
    let entries = read_all(&foreign_file());
    let ShadaEntryData::GlobalMark(mark) = &entries[2].data else {
        panic!("a global mark");
    };
    assert_eq!(
        (mark.name, mark.mark.lnum, mark.fname.as_cstr()),
        (1, 1, c"/x")
    );
    let ShadaEntryData::Register(reg) = &entries[6].data else {
        panic!("a register");
    };
    assert_eq!(
        (reg.name, reg.motion, reg.contents.len()),
        (b'a'.cast_signed(), 10, 1)
    );
    let ShadaEntryData::BufferList(list) = &entries[7].data else {
        panic!("a buffer list");
    };
    assert_eq!(
        list[0].extra.as_ref().map(ShadaExtra::bytes),
        Some(&b"\xa2bX\xc3"[..])
    );
    let item = entries[9].data.history();
    assert_eq!((item.histtype, &*item.text, item.sep), (1, c"s", b'/'));
    assert_eq!(entries.iter().map(|e| e.timestamp).max(), Some(7));
}

#[test]
fn widths_the_writer_keeps() {
    let mut out = Packer::default();
    let reg = ShadaRegister {
        name: b'q'.cast_signed(),
        motion: MOTION_BLOCKWISE,
        contents: vec![String_0::from_bytes(&[b'y'; 255])],
        is_unnamed: true,
        width: 300_000_000,
    };
    let entry = ShadaEntry::new(300_000_000, ShadaEntryData::Register(reg));
    pack_entry(&mut out, &entry, 0);
    let bytes = out.take();
    // The kind, then the timestamp past 2^28 as a uint64.
    assert_eq!(bytes[..10], [5, 0xcf, 0, 0, 0, 0, 0x11, 0xe1, 0xa3, 0]);
    // A 255-byte line steps up to bin16.
    let at = bytes.windows(3).position(|w| w == [0xc5, 0, 255]);
    assert!(at.is_some(), "{bytes:x?}");
    // The width, past 2^28, is a uint64 too.
    let at = bytes.windows(4).position(|w| w == *b"\xa2rw\xcf");
    assert!(at.is_some(), "{bytes:x?}");
    // And the whole entry reads back as itself.
    assert_eq!(pack_all(&read_all(&bytes)), bytes);
}

#[test]
fn the_keyset_reader_names_what_is_wrong() {
    fn mark(bytes: &[u8]) -> Result<(), Option<CString>> {
        let mut it = KeyDict__shada_mark::default();
        Cursor::new(bytes).keydict(&mut it, &mut None)
    }
    fn register(bytes: &[u8]) -> Result<(), Option<CString>> {
        let mut it = KeyDict__shada_register::default();
        Cursor::new(bytes).keydict(&mut it, &mut None)
    }
    let says = |text: &CStr| Err(Some(text.to_owned()));
    assert_eq!(mark(b"\x91\x01"), says(c"is not a dict"));
    assert_eq!(
        mark(b"\x81\x01\x01"),
        says(c"has key value which is not a string")
    );
    assert_eq!(mark(b"\x81\xa0\x01"), says(c"has empty key"));
    assert_eq!(
        mark(b"\x82\xa1f\xc4\x00\xa1f\xc4\x00"),
        says(c"duplicate key")
    );
    assert_eq!(
        mark(b"\x81\xa1l\xc4\x01x"),
        says(c"has l key value which is not an integer")
    );
    assert_eq!(
        mark(b"\x81\xa1f\x01"),
        says(c"has f key value which is not a binary")
    );
    assert_eq!(
        register(b"\x81\xa2ru\x01"),
        says(c"has ru key value which is not a boolean")
    );
    assert_eq!(
        register(b"\x81\xa2rc\x01"),
        says(c"has rc key with non-array value")
    );
    assert_eq!(
        register(b"\x81\xa2rc\x91\x01"),
        says(c"has rc array with non-binary value")
    );
    // A value under a key nobody knows that is not msgpack at all: no reason.
    assert_eq!(mark(b"\x81\xa2zz\xc1"), Err(None));
    // A known key's string whose bytes are not all there is not a string.
    assert_eq!(
        mark(b"\x81\xa1f\xd9\x05ab"),
        says(c"has f key value which is not a binary")
    );
}

#[test]
fn unknown_keys_are_kept_in_order() {
    let mut it = KeyDict__shada_mark::default();
    let mut extra = None;
    let bytes = b"\x84\xa2zz\x01\xa1f\xc4\x01x\xa2yy\x92\x01\x02\xa1l\x03";
    assert_eq!(Cursor::new(bytes).keydict(&mut it, &mut extra), Ok(()));
    assert_eq!(it.l, Some(3));
    let extra = extra.expect("two unknown keys");
    assert_eq!(extra.items(), 2);
    assert_eq!(extra.bytes(), b"\xa2zz\x01\xa2yy\x92\x01\x02");
}

#[test]
fn the_history_ring_keeps_the_newest_and_one_of_each_text() {
    fn line(text: &CStr, timestamp: Timestamp) -> ShadaEntry {
        let item = ShadaHistoryItem {
            histtype: 0,
            text: Rc::from(text),
            sep: 0,
        };
        ShadaEntry::new(timestamp, ShadaEntryData::HistoryEntry(item))
    }
    let mut hms = HistoryMergerState {
        hmll: merge::HistoryRing::new(3),
        pending: [line(c"b", 2), line(c"d", 4)].into(),
        history_type: 0,
    };
    // From the file: older Nvim entries are folded in first; a newer copy
    // of a text replaces the older one; a tie from the file loses.
    hms.insert(line(c"a", 1), true);
    hms.insert(line(c"c", 3), true);
    hms.insert(line(c"b", 5), true);
    hms.insert(line(c"c", 3), true);
    hms.insert_whole_neovim_history();
    let order: Vec<(CString, Timestamp)> = hms
        .hmll
        .drain()
        .into_iter()
        .map(|e| (e.data.history().text.as_ref().to_owned(), e.timestamp))
        .collect();
    assert_eq!(
        order,
        [
            (c"c".to_owned(), 3),
            (c"d".to_owned(), 4),
            (c"b".to_owned(), 5)
        ]
    );
}
