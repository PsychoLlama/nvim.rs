//! The `.un~` file: where it lives, what it hashes, and how its
//! records are laid down and picked back up.
//!
//! Everything the file carries is a fixed-width big-endian integer written
//! by [`BufInfo::write_bytes`] and range-checked on the way back in —
//! everything except the extmark payload, which is a native struct image.
//! That one is the reason [`unserialize_extmark`] validates: see
//! [`format::decode_splice`].
//!
//! The reader builds owned records — a header owns its entries and they own
//! their lines — so a record abandoned half-read is dropped, not freed by
//! hand.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::CStr;

use super::format::*;
use super::*;
use crate::memory::XString;
use crate::message_fmt::{msg_cstr, msg_cstr_opt};
use crate::option::split_option_part;
use crate::option::vars::P_UDIR;
use crate::os::fs::{dir_exists, mkdir_recurse, path_exists};
use crate::semsg;

/// The SHA-256 of every line of `buffer`, each followed by a NUL separator.
///
/// This is what an undo file stakes its claim on: a buffer whose hash has
/// moved is holding different text, and the tree in the file describes text
/// that is no longer there.
pub fn u_compute_hash(buffer: Buf, hash: &mut [u8; UNDO_HASH_SIZE as usize]) {
    let mut ctx = Sha256::new();
    for lnum in 1..=buffer.b_ml.ml_line_count {
        let mut lines = buffer.lines();
        ctx.update(lines.line(lnum));
        // The terminating NUL goes in too, as a line separator.
        ctx.update(&[0]);
    }
    hash.copy_from_slice(&ctx.finish());
}

/// The undo file `'undodir'` picks for the file at `buf_ffname`, or `None`
/// when there is no name or no directory in the list will have it.
///
/// `reading` asks for a file that already exists; writing instead creates
/// the last directory in the list if it has to.
pub fn u_get_undo_file_name(buf_ffname: Option<&CStr>, reading: bool) -> Option<XString> {
    let buf_ffname = buf_ffname?;
    let resolved = resolve_symlink(buf_ffname);
    let ffname: &[u8] = resolved.as_deref().unwrap_or(buf_ffname.to_bytes());

    // The name under a directory in 'undodir': the whole path with its
    // separators turned into '%', so that two files with the same tail
    // do not collide. Built once, whatever the list holds.
    let mut munged: Option<Vec<u8>> = None;
    let udir = P_UDIR.get();
    let mut rest: &[u8] = &udir;
    while !rest.is_empty() {
        let (mut dir_name, after) = split_option_part(rest, MAXPATHL as usize, b",");
        rest = after;
        let candidate = if dir_name == b"." {
            // "." means beside the file itself, as ".name.un~".
            Some(hidden_sibling(ffname))
        } else {
            trim_path_separators(&mut dir_name);
            let dir_name = XString::from_vec(dir_name);
            if dir_is_usable(dir_name.as_cstr(), rest.is_empty(), reading) {
                let munged = munged.get_or_insert_with(|| munge_separators(ffname));
                Some(concat_fnames(&dir_name, munged, true))
            } else {
                None
            }
        };
        if let Some(name) = candidate
            && (!reading || path_exists(name.as_cstr()))
        {
            return Some(name);
        }
    }
    None
}

/// `/a/b/name` as `/a/b/.name.un~`.
fn hidden_sibling(ffname: &[u8]) -> XString {
    let tail_at = tail_index(ffname);
    let mut name = XString::with_capacity(ffname.len() + 5);
    name.push_bytes(&ffname[..tail_at]);
    name.push_byte(b'.');
    name.push_bytes(&ffname[tail_at..]);
    name.push_bytes(b".un~");
    name
}

/// `/a/b/name` as `%a%b%name`.
///
/// The scan steps a character at a time, so a separator byte inside a
/// multibyte character is left alone.
fn munge_separators(ffname: &[u8]) -> Vec<u8> {
    let mut munged = ffname.to_vec();
    let mut at = 0;
    while at < munged.len() {
        if vim_ispathsep(c_int::from(munged[at])) {
            munged[at] = b'%';
        }
        at += c_cluster_len(&munged[at..]).max(1);
    }
    munged
}

/// Cuts the trailing path separators off a directory name. A name of one
/// byte is left alone, so `/` stays the root.
fn trim_path_separators(dir_name: &mut Vec<u8>) {
    if dir_name.len() <= 1 {
        return;
    }
    while dir_name
        .last()
        .is_some_and(|&c| vim_ispathsep(c_int::from(c)))
    {
        dir_name.pop();
    }
}

/// Whether undo files may be put in `dir_name`, creating it if this is the
/// last entry in `'undodir'` and we are about to write.
fn dir_is_usable(dir_name: &CStr, last_in_list: bool, reading: bool) -> bool {
    if dir_exists(dir_name) {
        return true;
    }
    if !last_in_list || reading {
        return false;
    }
    match mkdir_recurse(dir_name, 0o755) {
        Ok(_) => true,
        Err(failure) => {
            let shown = msg_cstr_opt(failure.dir.as_deref());
            let why = msg_cstr(failure.why);
            semsg!("E5003: Unable to create directory \"{shown}\" for undo file: {why}");
            false
        }
    }
}

/// `E825`: the file said something a well-formed undo file cannot say.
pub(crate) fn corruption_error(mesg: &CStr, file_name: &CStr) {
    let (mesg, file_name) = (msg_cstr(mesg), msg_cstr(file_name));
    semsg!("E825: Corrupted undo file ({mesg}): {file_name}");
}

/// Writes the file header: the magic, the version, the buffer hash, the `U`
/// shadow line and the three tree heads.
pub(crate) fn serialize_header(bi: &mut BufInfo, hash: &[u8; UNDO_HASH_SIZE as usize]) -> bool {
    let buf: Buf = bi.bi_buf;
    if !bi.write(&UF_START_MAGIC) {
        return false;
    }
    bi.write_bytes(UF_VERSION as uintmax_t, 2);
    if !bi.write(hash) {
        return false;
    }
    bi.write_int(buf.b_ml.ml_line_count, 4);

    let line: &[u8] = buf.b_u_line.as_deref().unwrap_or_default();
    bi.write_len(line.len());
    if !line.is_empty() && !bi.write(line) {
        return false;
    }
    bi.write_int(buf.b_u_line_lnum, 4);
    bi.write_int(buf.b_u_line_colnr, 4);

    bi.put_header_link(buf.b_u_oldhead);
    bi.put_header_link(buf.b_u_newhead);
    bi.put_header_link(buf.b_u_curhead);
    bi.write_int(buf.b_u_numhead, 4);
    bi.write_int(buf.b_u_seq_last, 4);
    bi.write_int(buf.b_u_seq_cur, 4);
    bi.put_time(buf.b_u_time_cur);
    bi.put_optional_field(UF_LAST_SAVE_NR, buf.b_u_save_nr_last);
    true
}

/// Writes one undo header and everything hanging off it: its four links, its
/// cursor and marks, its entries (newest first, as upstream's list ran), and
/// its extmark records.
pub(crate) fn serialize_uhp(bi: &mut BufInfo, uh: &UndoHeader) -> bool {
    if !bi.write_bytes(UF_HEADER_MAGIC as uintmax_t, 2) {
        return false;
    }
    bi.put_header_link(uh.next);
    bi.put_header_link(uh.prev);
    bi.put_header_link(uh.alt_next);
    bi.put_header_link(uh.alt_prev);
    bi.write_int(uh.seq, 4);
    serialize_pos(bi, uh.cursor);
    bi.write_int(uh.cursor_vcol, 4);
    bi.write_int(uh.flags, 2);
    for mark in &uh.named_marks {
        serialize_pos(bi, mark.mark);
    }
    serialize_visualinfo(bi, &uh.visual);
    bi.put_time(uh.time);
    bi.put_optional_field(UHP_SAVE_NR, uh.save_nr);

    for entry in uh.entries_newest_first() {
        bi.write_bytes(UF_ENTRY_MAGIC as uintmax_t, 2);
        if !serialize_uep(bi, entry) {
            return false;
        }
    }
    bi.write_bytes(UF_ENTRY_END_MAGIC as uintmax_t, 2);

    for &extup in &uh.extmarks {
        if !serialize_extmark(bi, extup) {
            return false;
        }
    }
    bi.write_bytes(UF_ENTRY_END_MAGIC as uintmax_t, 2);
    true
}

/// Reads one undo header back, or `None` for a file that has already been
/// complained about (or that ran out mid-record).
///
/// The four link fields come back as the sequence numbers the file spells
/// them as, which is what a link already is; nothing is converted. What is
/// checked is that the header has a name at all: a sequence number of zero
/// or less is not one a buffer ever hands out.
///
/// `bi` is positioned just after the header magic.
pub(crate) fn unserialize_uhp(bi: &mut BufInfo, file_name: &CStr) -> Option<UndoHeader> {
    let mut uh = UndoHeader {
        next: UndoLink::to_seq(bi.read_4c()),
        prev: UndoLink::to_seq(bi.read_4c()),
        alt_next: UndoLink::to_seq(bi.read_4c()),
        alt_prev: UndoLink::to_seq(bi.read_4c()),
        seq: bi.read_4c(),
        ..Default::default()
    };
    if uh.seq <= 0 {
        corruption_error(c"uh_seq", file_name);
        return None;
    }
    uh.cursor = unserialize_pos(bi);
    uh.cursor_vcol = bi.read_4c();
    uh.flags = bi.read_2c();
    let now: Timestamp = os_time();
    for mark in &mut uh.named_marks {
        mark.mark = unserialize_pos(bi);
        mark.timestamp = now;
        mark.fnum = 0;
    }
    uh.visual = unserialize_visualinfo(bi);
    uh.time = bi.read_time();
    // Unlike the file header's, a truncated trailer here is corruption:
    // the entry records that must follow it are gone.
    let Some(fields) = optional_fields(bi) else {
        corruption_error(c"truncated", file_name);
        return None;
    };
    for (what, value) in fields {
        if what == UHP_SAVE_NR {
            uh.save_nr = value;
        }
    }
    (unserialize_entries(bi, &mut uh, file_name) && unserialize_extmarks(bi, &mut uh, file_name))
        .then_some(uh)
}

/// The optional-field trailer both the file header and every undo header end
/// with: `{len what payload[len]}* 0`.
///
/// Only `UF_LAST_SAVE_NR`/`UHP_SAVE_NR` are ever written — they are the same
/// tag number — and both carry a four-byte value; a tag this build does not
/// know is skipped by its own length, which is what makes the trailer an
/// extension point rather than a desync.
///
/// `None` means the file ended in the middle of the trailer. The file header
/// tolerates that and an undo header does not, so the answer is the caller's
/// to interpret.
pub(crate) fn optional_fields(bi: &mut BufInfo) -> Option<Vec<(c_int, c_int)>> {
    let mut fields = Vec::new();
    loop {
        let len = bi.read_byte();
        if len == EOF {
            return None;
        }
        if len == 0 {
            return Some(fields);
        }
        let what = bi.read_byte();
        if what == UF_LAST_SAVE_NR {
            fields.push((what, bi.read_4c()));
        } else {
            for _ in 0..len {
                bi.read_byte();
            }
        }
    }
}

/// Reads the run of entry records into `uh`, up to the end marker. Says so
/// on a corrupt one; the caller drops the header either way it fails.
fn unserialize_entries(bi: &mut BufInfo, uh: &mut UndoHeader, file_name: &CStr) -> bool {
    loop {
        let c = bi.read_2c();
        if c != UF_ENTRY_MAGIC {
            if c == UF_ENTRY_END_MAGIC {
                // The file lists them newest first; the header keeps the
                // newest last.
                uh.entries.reverse();
                return true;
            }
            corruption_error(c"entry end", file_name);
            return false;
        }
        let Some(entry) = unserialize_uep(bi, file_name) else {
            return false;
        };
        uh.entries.push(entry);
    }
}

/// Reads the run of extmark records into `uh`'s list, up to the end marker.
fn unserialize_extmarks(bi: &mut BufInfo, uh: &mut UndoHeader, file_name: &CStr) -> bool {
    loop {
        let c = bi.read_2c();
        if c != UF_ENTRY_MAGIC {
            if c == UF_ENTRY_END_MAGIC {
                return true;
            }
            corruption_error(c"entry end", file_name);
            return false;
        }
        // Upstream leaks the header here; every other failure in this
        // function frees it, and so does dropping it.
        let Some(extup) = unserialize_extmark(bi, file_name) else {
            return false;
        };
        uh.extmarks.push(extup);
    }
}

/// Writes one extmark record: its type, then its payload.
///
/// Only splices and moves are written; a `kExtmarkSavePos` record describes
/// marks in a buffer this file will be read into fresh, and is dropped.
pub(crate) fn serialize_extmark(bi: &mut BufInfo, extup: ExtmarkUndoObject) -> bool {
    let image = match &extup {
        ExtmarkUndoObject::Splice(splice) => encode_splice(splice),
        ExtmarkUndoObject::Move(move_0) => encode_move(move_0),
        ExtmarkUndoObject::SavePos(_) => return true,
    };
    bi.write_bytes(UF_ENTRY_MAGIC as uintmax_t, 2);
    bi.write_bytes(u64::from(extup.wire_type()), 4);
    bi.write(&image)
}

/// Reads one extmark record back, checking that its payload names
/// coordinates a change to a buffer could have produced.
///
/// **This check is a deliberate divergence from upstream.** The payload is a
/// raw native struct image: neither the owner check nor the `E823` text hash
/// covers it, so a file nvim itself wrote stays acceptable after the 48
/// bytes are edited, and `extmark_splice_impl` then overflows on
/// `start_row + old_row`. See `undo-extmark-payload-unvalidated` and
/// [`format::decode_splice`].
///
/// `bi` is positioned just after the entry magic.
pub(crate) fn unserialize_extmark(bi: &mut BufInfo, file_name: &CStr) -> Option<ExtmarkUndoObject> {
    let type_0: UndoObjectType = bi.read_4c().cast_unsigned();
    let mut image = [0u8; EXTMARK_PAYLOAD_LEN];
    let decoded = if type_0 == kExtmarkSplice {
        if !bi.read(&mut image) {
            corruption_error(c"extmark truncated", file_name);
            return None;
        }
        decode_splice(&image)
            .map(ExtmarkUndoObject::Splice)
            .ok_or(c"extmark splice")
    } else if type_0 == kExtmarkMove {
        if !bi.read(&mut image) {
            corruption_error(c"extmark truncated", file_name);
            return None;
        }
        decode_move(&image)
            .map(ExtmarkUndoObject::Move)
            .ok_or(c"extmark move")
    } else {
        Err(c"extmark type")
    };
    decoded
        .inspect_err(|mesg| corruption_error(mesg, file_name))
        .ok()
}

/// Writes one undo entry: where the lines it saved came from, and the lines.
pub(crate) fn serialize_uep(bi: &mut BufInfo, entry: &UndoEntry) -> bool {
    bi.write_int(entry.top, 4);
    bi.write_int(entry.bot, 4);
    bi.write_int(entry.lcount, 4);
    bi.write_len(entry.lines.len());
    for line in &entry.lines {
        if !bi.write_len(line.len()) {
            return false;
        }
        if !line.is_empty() && !bi.write(line) {
            return false;
        }
    }
    true
}

/// How many lines an entry is given room for before any are read: the count
/// is the file's word, and a corrupt one must not reserve gigabytes.
const ENTRY_LINES_RESERVED: usize = 1024;

/// Reads one undo entry back, or `None` when it is corrupt (said) or the
/// file ran out in the middle of it (not said, as upstream).
pub(crate) fn unserialize_uep(bi: &mut BufInfo, file_name: &CStr) -> Option<UndoEntry> {
    let top = bi.read_4c();
    let bot = bi.read_4c();
    let lcount = bi.read_4c();
    // A count, not an offset. Upstream widens it to `size_t` for the loop
    // below, so a negative one becomes about 2^64 iterations over an array
    // it never allocated.
    let Ok(size) = usize::try_from(bi.read_4c()) else {
        corruption_error(c"entry size", file_name);
        return None;
    };
    let mut lines = Vec::with_capacity(size.min(ENTRY_LINES_RESERVED));
    for _ in 0..size {
        let Ok(line_len) = usize::try_from(bi.read_4c()) else {
            corruption_error(c"line length", file_name);
            return None;
        };
        lines.push(undo_read_string(bi, line_len)?);
    }
    Some(UndoEntry {
        top,
        bot,
        lcount,
        lines,
    })
}

/// Writes a position as three four-byte fields.
pub(crate) fn serialize_pos(bi: &mut BufInfo, pos: Pos) {
    bi.write_int(pos.lnum, 4);
    bi.write_int(pos.col, 4);
    bi.write_int(pos.coladd, 4);
}

/// Reads a position back, clamping each field at zero: a negative line or
/// column is not a position, and every caller would have to check anyway.
pub(crate) fn unserialize_pos(bi: &mut BufInfo) -> Pos {
    Pos {
        lnum: bi.read_4c().max(0),
        col: bi.read_4c().max(0),
        coladd: bi.read_4c().max(0),
    }
}

/// Writes what a Visual selection covered.
pub(crate) fn serialize_visualinfo(bi: &mut BufInfo, info: &VisualInfo) {
    serialize_pos(bi, info.vi_start);
    serialize_pos(bi, info.vi_end);
    bi.write_int(info.vi_mode, 4);
    bi.write_int(info.vi_curswant, 4);
}

/// Reads a Visual selection back.
pub(crate) fn unserialize_visualinfo(bi: &mut BufInfo) -> VisualInfo {
    let vi_start = unserialize_pos(bi);
    let vi_end = unserialize_pos(bi);
    let vi_mode = bi.read_4c();
    let vi_curswant = bi.read_4c();
    VisualInfo {
        vi_start,
        vi_end,
        vi_mode,
        vi_curswant,
    }
}

/// Reads a `len`-byte string, or `None` if the file ran out.
///
/// A NUL inside the bytes ends the string there, as every reader of
/// upstream's `xmemdupz` copy measured it with `strlen`.
pub(crate) fn undo_read_string(bi: &mut BufInfo, len: size_t) -> Option<XString> {
    let mut bytes = vec![0u8; len];
    if !bi.read(&mut bytes) {
        return None;
    }
    if let Some(nul) = bytes.iter().position(|&b| b == 0) {
        bytes.truncate(nul);
    }
    Some(XString::from_vec(bytes))
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use super::*;
    use crate::buffer::{bare_buffer, free_bare_buffer};
    use crate::global_cell::editor_state_lock;
    use crate::types::{ExtmarkMove, ExtmarkSavePos, ExtmarkSplice};

    /// A scratch file of this case's own, removed on drop.
    struct Scratch(std::path::PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            Self(std::env::temp_dir().join(format!("nvim-lib-undo-{}-{name}", std::process::id())))
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    fn splice(n: c_int) -> ExtmarkSplice {
        ExtmarkSplice {
            start_row: n,
            start_col: 1,
            old_row: 0,
            old_col: 2,
            new_row: 1,
            new_col: 0,
            start_byte: 10,
            old_byte: 2,
            new_byte: 1,
        }
    }

    /// What `serialize_uhp` writes, `unserialize_uhp` reads back: the links,
    /// the cursor, the marks, the Visual area, the entries in their order
    /// (newest last in memory, newest first in the file) with multibyte and
    /// empty lines, and the extmark records — minus the saved positions,
    /// which are never written.
    #[test]
    fn a_header_round_trips_through_the_file() {
        let _held = editor_state_lock();
        let buffer = bare_buffer();
        let scratch = Scratch::new("header");
        let mut uh = UndoHeader {
            next: UndoLink::to_seq(3),
            prev: UndoLink::to_seq(7),
            alt_next: UndoLink::to_seq(8),
            alt_prev: UndoLink::NONE,
            seq: 5,
            cursor: Pos {
                lnum: 4,
                col: 2,
                coladd: 0,
            },
            cursor_vcol: -1,
            flags: UH_CHANGED | UH_RELOAD,
            save_nr: 2,
            time: 1_700_000_000,
            ..Default::default()
        };
        uh.named_marks[0].mark.lnum = 9;
        uh.named_marks[25].mark.col = 3;
        uh.visual.vi_start.lnum = 1;
        uh.visual.vi_mode = c_int::from(b'V');
        uh.entries.push(UndoEntry {
            top: 0,
            bot: 0,
            lcount: 0,
            lines: vec![XString::from_bytes("h\u{e9}llo".as_bytes()), XString::new()],
        });
        uh.entries.push(UndoEntry {
            top: 3,
            bot: 5,
            lcount: 12,
            lines: vec![XString::from_bytes(b"newest")],
        });
        uh.extmarks.push(ExtmarkUndoObject::Splice(splice(1)));
        uh.extmarks.push(ExtmarkUndoObject::SavePos(ExtmarkSavePos {
            mark: 0,
            old_row: 1,
            old_col: 1,
            invalidated: false,
        }));
        uh.extmarks.push(ExtmarkUndoObject::Move(ExtmarkMove {
            start_row: 1,
            start_col: 0,
            extent_row: 2,
            extent_col: 0,
            new_row: 4,
            new_col: 0,
            start_byte: 5,
            extent_byte: 8,
            new_byte: 20,
        }));
        uh.extmarks.push(ExtmarkUndoObject::Splice(splice(2)));

        let mut bi = BufInfo::writer(buffer, File::create(&scratch.0).expect("a scratch file"));
        assert!(serialize_uhp(&mut bi, &uh));
        drop(bi);

        let mut bi = BufInfo::reader(buffer, File::open(&scratch.0).expect("written"));
        assert_eq!(bi.read_2c(), UF_HEADER_MAGIC);
        let back = unserialize_uhp(&mut bi, c"lib test").expect("reads back");
        assert_eq!(back.link(), UndoLink::to_seq(5));
        assert_eq!(
            (back.next, back.prev, back.alt_next, back.alt_prev),
            (uh.next, uh.prev, uh.alt_next, uh.alt_prev)
        );
        assert!(back.cursor == uh.cursor);
        assert_eq!((back.cursor_vcol, back.flags), (-1, uh.flags));
        assert_eq!((back.save_nr, back.time), (2, 1_700_000_000));
        assert_eq!(back.named_marks[0].mark.lnum, 9);
        assert_eq!(back.named_marks[25].mark.col, 3);
        assert_eq!(
            (back.visual.vi_start.lnum, back.visual.vi_mode),
            (1, c_int::from(b'V'))
        );
        let entries: Vec<(LineNr, LineNr, LineNr, Vec<Vec<u8>>)> = back
            .entries
            .iter()
            .map(|e| {
                (
                    e.top,
                    e.bot,
                    e.lcount,
                    e.lines.iter().map(|l| l.to_vec()).collect(),
                )
            })
            .collect();
        assert_eq!(
            entries,
            vec![
                (0, 0, 0, vec!["h\u{e9}llo".as_bytes().to_vec(), Vec::new()]),
                (3, 5, 12, vec![b"newest".to_vec()]),
            ]
        );
        assert!(back.getbot.is_none());
        let kinds: Vec<UndoObjectType> = back
            .extmarks
            .iter()
            .map(ExtmarkUndoObject::wire_type)
            .collect();
        assert_eq!(kinds, vec![kExtmarkSplice, kExtmarkMove, kExtmarkSplice]);
        match back.extmarks[2] {
            ExtmarkUndoObject::Splice(s) => assert_eq!(s.start_row, 2),
            _ => panic!("the last record is the second splice"),
        }
        drop(bi);
        free_bare_buffer(buffer);
    }

    /// A line stored with a NUL inside it reads back cut at the NUL, which
    /// is where upstream's readers of the copy stopped.
    #[test]
    fn a_line_holding_a_nul_reads_back_cut_there() {
        let _held = editor_state_lock();
        let buffer = bare_buffer();
        let scratch = Scratch::new("nul");
        File::create(&scratch.0)
            .and_then(|mut file| file.write_all(b"ab\0cd"))
            .expect("a scratch file");
        let mut bi = BufInfo::reader(buffer, File::open(&scratch.0).expect("written"));
        let line = undo_read_string(&mut bi, 5).expect("five bytes are there");
        assert_eq!(&*line, b"ab");
        // And a read past the end is no line at all.
        assert!(undo_read_string(&mut bi, 1).is_none());
        drop(bi);
        free_bare_buffer(buffer);
    }

    #[test]
    fn the_munged_name_keeps_multibyte_characters_whole() {
        assert_eq!(munge_separators(b"/a/b/c"), b"%a%b%c");
        assert_eq!(
            munge_separators("/d\u{e9}j\u{e0}/x".as_bytes()),
            "%d\u{e9}j\u{e0}%x".as_bytes()
        );
        assert_eq!(&*hidden_sibling(b"/a/b/name"), b"/a/b/.name.un~");
        assert_eq!(&*hidden_sibling(b"name"), b".name.un~");
        let mut dir = b"/tmp/undo//".to_vec();
        trim_path_separators(&mut dir);
        assert_eq!(dir, b"/tmp/undo");
        let mut root = b"/".to_vec();
        trim_path_separators(&mut root);
        assert_eq!(root, b"/");
    }
}
