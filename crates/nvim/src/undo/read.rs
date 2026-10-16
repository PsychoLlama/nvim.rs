//! `u_read_undo`: reading an undo tree back in, and grafting it onto a
//! buffer whose contents still hash the same.
//!
//! Every step can fail on a corrupt file, and the transpiled shape of that
//! was one 180-column staircase of `else` branches around a `goto error`.
//! Here each step is its own function answering `Option`, and the caller
//! that owns an allocation is the one that frees it.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::memory::XString;
use crate::message_fmt::msg_cstr;
use crate::os::fs::file_info;
use crate::os::users::os_getuid;
use crate::semsg;
use crate::smsg;
use crate::winlayer::Buf;
use core::ffi::CStr;
use std::collections::HashSet;

use super::file::*;
use super::format::*;
use super::store::header_adopt;
use super::*;

/// Reads an undo file into the current buffer's tree.
///
/// `name` is the file the user named (`:rundo`), or `None` to let
/// `'undodir'` pick one for `orig_name`. `hash` is the current buffer's
/// content hash: a file whose hash disagrees describes text this buffer no
/// longer holds and is refused rather than applied.
pub fn u_read_undo(
    name: Option<&CStr>,
    hash: &[u8; UNDO_HASH_SIZE as usize],
    orig_name: Option<&CStr>,
) {
    let picked;
    let file_name: &CStr = match name {
        Some(name) => name,
        None => {
            let full = Buf::current().name.full().map(XString::from_cstr);
            let Some(found) = u_get_undo_file_name(full.as_ref().map(XString::as_cstr), true)
            else {
                return;
            };
            // Upstream returns here without freeing the name it just built.
            if !owner_matches(found.as_cstr(), orig_name) {
                return;
            }
            picked = found;
            picked.as_cstr()
        }
    };

    // Always under 'verbose', even when the user named the file.
    verbosely(true, || {
        let file_name = msg_cstr(file_name);
        smsg!(0, "Reading undo file: {file_name}");
    });
    let path = <std::ffi::OsStr as std::os::unix::ffi::OsStrExt>::from_bytes(file_name.to_bytes());
    match std::fs::File::open(path) {
        Err(_) => {
            if name.is_some() || p_verbose() > 0 {
                let file_name = msg_cstr(file_name);
                semsg!("E822: Cannot open undo file for reading: {file_name}");
            }
        }
        Ok(file) => {
            let mut bi = BufInfo::reader(Buf::current(), file);
            read_undo_file(&mut bi, file_name, name.is_none(), hash);
        }
    }
}

/// Whether the undo file at `file_name` may be trusted: either it belongs to
/// the same user as the edited file, or it belongs to us.
///
/// Says why not, under `'verbose'`.
fn owner_matches(file_name: &CStr, orig_name: Option<&CStr>) -> bool {
    let edited = orig_name.and_then(file_info);
    let undo = file_info(file_name);
    let foreign = match (edited, undo) {
        (Some(edited), Some(undo)) => {
            edited.stat.st_uid != undo.stat.st_uid
                && undo.stat.st_uid != uint64_t::from(os_getuid())
        }
        _ => false,
    };
    if !foreign {
        return true;
    }
    verbosely(true, || {
        let file_name = msg_cstr(file_name);
        smsg!(0, "Not reading undo file, owner differs: {file_name}");
    });
    false
}

/// Reads the whole of an open undo file and grafts it onto the current
/// buffer. Every failure has already said what was wrong with the file.
fn read_undo_file(
    bi: &mut BufInfo,
    file_name: &CStr,
    automatic: bool,
    hash: &[u8; UNDO_HASH_SIZE as usize],
) {
    let mut magic = [0u8; UF_START_MAGIC.len()];
    if !bi.read(&mut magic) || magic != UF_START_MAGIC {
        let file_name = msg_cstr(file_name);
        semsg!("E823: Not an undo file: {file_name}");
        return;
    }
    if bi.read_2c() != UF_VERSION {
        let file_name = msg_cstr(file_name);
        semsg!("E824: Incompatible undo file: {file_name}");
        return;
    }
    let mut read_hash = [0u8; UNDO_HASH_SIZE as usize];
    if !bi.read(&mut read_hash) {
        corruption_error(c"hash", file_name);
        return;
    }
    let line_count = bi.read_4c() as LineNr;
    // The tree describes text; a buffer holding different text cannot
    // have it applied.
    if *hash != read_hash || line_count != Buf::current().b_ml.ml_line_count {
        verbosely(automatic, || {
            let mesg = gettext(c"File contents changed, cannot use undo info");
            give_warning(mesg, true, true);
        });
        return;
    }

    let Some(header) = read_file_header(bi, file_name) else {
        return;
    };
    let Some(headers) = read_headers(bi, file_name, header.num_head) else {
        return;
    };
    if !graft(headers, header, file_name) {
        return;
    }
    if !automatic {
        let file_name = msg_cstr(file_name);
        smsg!(0, "Finished reading undo file {file_name}");
    }
}

/// Everything the undo file says before its first header record.
struct FileHeader {
    /// The `U` shadow line, and where it came from.
    line: Option<XString>,
    line_lnum: LineNr,
    line_colnr: ColNr,
    /// The three tree heads, still as the sequence numbers the file spells
    /// them as — a link *is* a sequence number, so no conversion happens.
    old_head: c_int,
    new_head: c_int,
    cur_head: c_int,
    /// How many header records follow.
    num_head: c_int,
    seq_last: c_int,
    seq_cur: c_int,
    seq_time: time_t,
    last_save_nr: c_int,
}

/// Reads the file header. `None` is a file that has already been complained
/// about, or one whose shadow-line length is not a length at all — which
/// upstream rejects silently, and so does this.
///
/// `bi` is positioned at the shadow-line length.
fn read_file_header(bi: &mut BufInfo, file_name: &CStr) -> Option<FileHeader> {
    let str_len = usize::try_from(bi.read_4c()).ok()?;
    // A line the file ran out in the middle of is no line, as upstream's
    // NULL from `read_string_decrypt` was.
    let line = if str_len > 0 {
        undo_read_string(bi, str_len)
    } else {
        None
    };
    let line_lnum = bi.read_4c() as LineNr;
    let line_colnr = bi.read_4c();
    if line_lnum < 0 || line_colnr < 0 {
        corruption_error(c"line lnum/col", file_name);
        return None;
    }
    let mut header = FileHeader {
        line,
        line_lnum,
        line_colnr,
        old_head: bi.read_4c(),
        new_head: bi.read_4c(),
        cur_head: bi.read_4c(),
        num_head: bi.read_4c(),
        seq_last: bi.read_4c(),
        seq_cur: bi.read_4c(),
        seq_time: bi.read_time(),
        last_save_nr: 0,
    };
    // A truncated trailer is not an error here: upstream stops at EOF and
    // keeps whatever it had.
    for (what, value) in optional_fields(bi).unwrap_or_default() {
        if what == UF_LAST_SAVE_NR {
            header.last_save_nr = value;
        }
    }
    Some(header)
}

/// Reads the `num_head` header records and the end marker after them.
///
/// The headers come back unlinked: their four link fields are the sequence
/// numbers the file carried and still have to be checked against the headers
/// that actually turned up. On failure nothing survives.
///
/// `bi` is positioned at the first header record.
fn read_headers(bi: &mut BufInfo, file_name: &CStr, num_head: c_int) -> Option<Vec<UndoHeader>> {
    let mut headers: Vec<UndoHeader> = Vec::new();
    loop {
        let c = bi.read_2c();
        if c != UF_HEADER_MAGIC {
            // The file claims a count of its own; a run that stops short
            // of it, or an end marker that is not one, is corruption.
            if headers.len() != usize::try_from(num_head).unwrap_or(usize::MAX) {
                corruption_error(c"num_head", file_name);
            } else if c != UF_HEADER_END_MAGIC {
                corruption_error(c"end marker", file_name);
            } else {
                return Some(headers);
            }
            return None;
        }
        if headers.len() >= usize::try_from(num_head).unwrap_or(0) {
            corruption_error(c"num_head too small", file_name);
            return None;
        }
        headers.push(unserialize_uhp(bi, file_name)?);
    }
}

/// Replaces the current buffer's undo tree with the one the file describes.
///
/// Answers whether it took: a file naming one header twice describes a tree
/// with two headers of the same name, which is not a tree.
fn graft(mut headers: Vec<UndoHeader>, header: FileHeader, file_name: &CStr) -> bool {
    let mut seqs: HashSet<c_int> = HashSet::with_capacity(headers.len());
    for uh in &headers {
        if !seqs.insert(uh.seq) {
            corruption_error(c"duplicate uh_seq", file_name);
            return false;
        }
    }

    // A link to the header's own sequence number, or to one no header in
    // this file carries, links to nothing — which is what the scans
    // worked out to, since they skipped `i == j` and left NULL when
    // nothing matched.
    let resolve = |from: c_int, link: UndoLink| {
        if link.seq() == from || !seqs.contains(&link.seq()) {
            UndoLink::NONE
        } else {
            link
        }
    };
    for uh in &mut headers {
        let seq = uh.seq;
        uh.next = resolve(seq, uh.next);
        uh.prev = resolve(seq, uh.prev);
        uh.alt_next = resolve(seq, uh.alt_next);
        uh.alt_prev = resolve(seq, uh.alt_prev);
    }

    let mut buf = Buf::current();
    u_blockfree(buf);
    // The links already name these headers; the store is what turns a
    // name back into one.
    for uh in headers {
        header_adopt(buf, Box::new(uh));
    }
    // `unserialize_uhp` refuses a header whose sequence number is not
    // positive, so a number the set holds is always a usable link.
    let head = |seq: c_int| {
        if seqs.contains(&seq) {
            UndoLink::to_seq(seq)
        } else {
            UndoLink::NONE
        }
    };
    buf.b_u_oldhead = head(header.old_head);
    buf.b_u_newhead = head(header.new_head);
    buf.b_u_curhead = head(header.cur_head);
    buf.b_u_line = header.line;
    buf.b_u_line_lnum = header.line_lnum;
    buf.b_u_line_colnr = header.line_colnr;
    buf.b_u_numhead = header.num_head;
    // Every header the file carried has already been handed out, so the
    // next sequence number must come after them all. A well-formed file
    // says so itself; a corrupt one would otherwise hand out a number
    // some header already has.
    buf.b_u_seq_last = header.seq_last.max(seqs.iter().copied().max().unwrap_or(0));
    buf.b_u_seq_cur = header.seq_cur;
    buf.b_u_time_cur = header.seq_time;
    buf.b_u_save_nr_last = header.last_save_nr;
    buf.b_u_save_nr_cur = header.last_save_nr;
    buf.b_u_synced = true;
    true
}
