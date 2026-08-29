//! `u_read_undo`: reading an undo tree back in, and grafting it onto a
//! buffer whose contents still hash the same.
//!
//! Every step can fail on a corrupt file, and the transpiled shape of that
//! was one 180-column staircase of `else` branches around a `goto error`.
//! Here each step is its own function answering `Option`, and the caller
//! that owns an allocation is the one that frees it.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::Buf;
use std::collections::HashSet;

use super::file::*;
use super::format::*;
use super::store::header_adopt;
use super::tree::*;
use super::*;
use crate::{semsg_c, smsg_c};

/// Reads an undo file into the current buffer's tree.
///
/// `name` is the file the user named (`:rundo`), or NULL to let `'undodir'`
/// pick one for `orig_name`. `hash` is the current buffer's content hash: a
/// file whose hash disagrees describes text this buffer no longer holds and
/// is refused rather than applied.
///
/// # Safety
///
/// A live current buffer; `name` and `orig_name` are NULL or NUL-terminated,
/// and `hash` points at [`UNDO_HASH_SIZE`] readable bytes.
pub unsafe fn u_read_undo(name: *mut c_char, hash: *const uint8_t, orig_name: *const c_char) {
    // SAFETY: a live current buffer and NUL-terminated names, by the above.
    let file_name: *mut c_char = if name.is_null() {
        let picked = unsafe { u_get_undo_file_name(cur_buf().b_ffname, true) };
        if picked.is_null() {
            return;
        }
        if !unsafe { owner_matches(picked, orig_name) } {
            // Upstream returns here without freeing the name it just
            // built; there is nothing to be gained by keeping the leak.
            unsafe { xfree(picked.cast()) };
            return;
        }
        picked
    } else {
        name
    };

    // Always under 'verbose', even when the user named the file.
    verbosely(true, || {
        // SAFETY: the message macros expand to a `vim_snprintf` over
        // the format literal above and the editor's message buffers.
        unsafe { smsg_c!(0, gettext(c"Reading undo file: %s").as_ptr(), file_name) };
    });
    let fp: *mut FILE = unsafe { os_fopen(file_name, c"r".as_ptr()) };
    if fp.is_null() {
        if !name.is_null() || p_verbose.get() > 0 {
            let fmt = gettext(c"E822: Cannot open undo file for reading: %s");
            unsafe { semsg_c!(fmt, file_name) };
        }
    } else {
        unsafe { read_undo_file(fp, file_name, name.is_null(), hash) };
        unsafe { fclose(fp) };
    }
    if !ptr::eq(file_name, name) {
        unsafe { xfree(file_name.cast()) };
    }
}

/// Whether the undo file at `file_name` may be trusted: either it belongs to
/// the same user as the edited file, or it belongs to us.
///
/// Says why not, under `'verbose'`.
///
/// # Safety
///
/// Both names are NUL-terminated.
unsafe fn owner_matches(file_name: *const c_char, orig_name: *const c_char) -> bool {
    // SAFETY: NUL-terminated names, by the contract above.
    let mut edited = FileInfo::default();
    let mut undo = FileInfo::default();
    if !(unsafe { os_fileinfo(orig_name, &raw mut edited) }
        && unsafe { os_fileinfo(file_name, &raw mut undo) }
        && edited.stat.st_uid != undo.stat.st_uid
        && undo.stat.st_uid != uint64_t::from(unsafe { getuid() }))
    {
        return true;
    }
    verbosely(true, || {
        let fmt = gettext(c"Not reading undo file, owner differs: %s");
        unsafe { smsg_c!(0, fmt.as_ptr(), file_name) };
    });
    false
}

/// Reads the whole of an open undo file and grafts it onto the current
/// buffer. Every failure has already said what was wrong with the file.
///
/// # Safety
///
/// `fp` is open for reading on `file_name`, there is a live current buffer,
/// and `hash` points at [`UNDO_HASH_SIZE`] readable bytes.
unsafe fn read_undo_file(
    fp: *mut FILE,
    file_name: *const c_char,
    automatic: bool,
    hash: *const uint8_t,
) {
    // SAFETY: an open file and a live current buffer, by the contract above.
    let mut bi = bufinfo_T {
        bi_buf: unsafe { Buf::current() },
        bi_fp: fp,
    };
    let bi = &raw mut bi;

    let mut magic = [0u8; UF_START_MAGIC.len()];
    if unsafe { fread(magic.as_mut_ptr().cast(), magic.len(), 1, fp) } != 1
        || magic != UF_START_MAGIC
    {
        // SAFETY: the message macros expand to a `vim_snprintf` over
        // the format literal above and the editor's message buffers.
        unsafe { semsg_c!(gettext(c"E823: Not an undo file: %s"), file_name) };
        return;
    }
    if unsafe { get2c(fp) } != UF_VERSION {
        let fmt = gettext(c"E824: Incompatible undo file: %s");
        unsafe { semsg_c!(fmt, file_name) };
        return;
    }
    let mut read_hash = [0u8; UNDO_HASH_SIZE as usize];
    if !unsafe { undo_read(bi, read_hash.as_mut_ptr(), read_hash.len()) } {
        unsafe { corruption_error(c"hash".as_ptr(), file_name) };
        return;
    }
    let line_count = unsafe { undo_read_4c(bi) } as linenr_T;
    // The tree describes text; a buffer holding different text cannot
    // have it applied.
    if unsafe { core::slice::from_raw_parts(hash, read_hash.len()) } != read_hash
        || line_count != cur_buf().b_ml.ml_line_count
    {
        verbosely(automatic, || {
            let mesg = gettext(c"File contents changed, cannot use undo info");
            unsafe { give_warning(mesg.as_ptr(), true, true) };
        });
        return;
    }

    let Some(header) = (unsafe { read_file_header(bi, file_name) }) else {
        return;
    };
    let Some(headers) = (unsafe { read_headers(bi, file_name, header.num_head) }) else {
        unsafe { xfree(header.line_ptr.cast()) };
        return;
    };
    if !unsafe { graft(&headers, &header, file_name) } {
        unsafe { free_headers(&headers) };
        unsafe { xfree(header.line_ptr.cast()) };
        return;
    }
    if !automatic {
        let fmt = gettext(c"Finished reading undo file %s");
        unsafe { smsg_c!(0, fmt.as_ptr(), file_name) };
    }
}

/// Everything the undo file says before its first header record.
struct FileHeader {
    /// The `U` shadow line, and where it came from. Owned: whoever holds a
    /// `FileHeader` and gives up on it frees this.
    line_ptr: *mut c_char,
    line_lnum: linenr_T,
    line_colnr: colnr_T,
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
/// # Safety
///
/// `bi` is open for reading and positioned at the shadow-line length.
unsafe fn read_file_header(bi: *mut bufinfo_T, file_name: *const c_char) -> Option<FileHeader> {
    // SAFETY: an open undo file, by the contract above.
    let str_len = unsafe { undo_read_4c(bi) };
    if str_len < 0 {
        return None;
    }
    let line_ptr = if str_len > 0 {
        unsafe { undo_read_string(bi, str_len as size_t) }
    } else {
        ptr::null_mut()
    };
    let line_lnum = unsafe { undo_read_4c(bi) } as linenr_T;
    let line_colnr = unsafe { undo_read_4c(bi) };
    if line_lnum < 0 || line_colnr < 0 {
        unsafe { corruption_error(c"line lnum/col".as_ptr(), file_name) };
        unsafe { xfree(line_ptr.cast()) };
        return None;
    }
    let mut header = FileHeader {
        line_ptr,
        line_lnum,
        line_colnr,
        old_head: unsafe { undo_read_4c(bi) },
        new_head: unsafe { undo_read_4c(bi) },
        cur_head: unsafe { undo_read_4c(bi) },
        num_head: unsafe { undo_read_4c(bi) },
        seq_last: unsafe { undo_read_4c(bi) },
        seq_cur: unsafe { undo_read_4c(bi) },
        seq_time: unsafe { undo_read_time(bi) },
        last_save_nr: 0,
    };
    // A truncated trailer is not an error here: upstream stops at EOF and
    // keeps whatever it had.
    for (what, value) in unsafe { optional_fields(bi) }.unwrap_or_default() {
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
/// # Safety
///
/// `bi` is open for reading and positioned at the first header record.
unsafe fn read_headers(
    bi: *mut bufinfo_T,
    file_name: *const c_char,
    num_head: c_int,
) -> Option<Vec<*mut u_header_T>> {
    let mut headers: Vec<*mut u_header_T> = Vec::new();
    // SAFETY: an open undo file, by the contract above.
    loop {
        let c = unsafe { undo_read_2c(bi) };
        if c != UF_HEADER_MAGIC {
            // The file claims a count of its own; a run that stops short
            // of it, or an end marker that is not one, is corruption.
            if headers.len() != usize::try_from(num_head).unwrap_or(usize::MAX) {
                unsafe { corruption_error(c"num_head".as_ptr(), file_name) };
            } else if c != UF_HEADER_END_MAGIC {
                unsafe { corruption_error(c"end marker".as_ptr(), file_name) };
            } else {
                return Some(headers);
            }
            unsafe { free_headers(&headers) };
            return None;
        }
        if headers.len() >= usize::try_from(num_head).unwrap_or(0) {
            unsafe { corruption_error(c"num_head too small".as_ptr(), file_name) };
            unsafe { free_headers(&headers) };
            return None;
        }
        let uhp = unsafe { unserialize_uhp(bi, file_name) };
        if uhp.is_null() {
            unsafe { free_headers(&headers) };
            return None;
        }
        headers.push(uhp);
    }
}

/// Frees headers that never made it into a buffer's store.
///
/// # Safety
///
/// Every pointer is a live header nothing else owns.
unsafe fn free_headers(headers: &[*mut u_header_T]) {
    for &uhp in headers {
        // SAFETY: a live header nobody else owns, by the contract above.
        unsafe { u_free_uhp(uhp) };
    }
}

/// Replaces the current buffer's undo tree with the one the file describes.
///
/// Answers whether it took: a file naming one header twice describes a tree
/// with two headers of the same name, which is not a tree.
///
/// # Safety
///
/// A live current buffer, and every header is live and owned by nobody else.
unsafe fn graft(
    headers: &[*mut u_header_T],
    header: &FileHeader,
    file_name: *const c_char,
) -> bool {
    // SAFETY: live headers and a live current buffer, by the contract above.
    // A sequence number is a header's name, so two headers carrying the
    // same one are indistinguishable: corrupt. Collecting them also
    // answers "is there a header for this link?" in one lookup, which is
    // what the transpiled code's five O(n^2) scans worked out to.
    let mut seqs: HashSet<c_int> = HashSet::with_capacity(headers.len());
    for &uhp in headers {
        if !seqs.insert(unsafe { (*uhp).uh_seq }) {
            unsafe { corruption_error(c"duplicate uh_seq".as_ptr(), file_name) };
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
    for &uhp in headers {
        let seq = unsafe { (*uhp).uh_seq };
        unsafe { (*uhp).uh_next = resolve(seq, (*uhp).uh_next) };
        unsafe { (*uhp).uh_prev = resolve(seq, (*uhp).uh_prev) };
        unsafe { (*uhp).uh_alt_next = resolve(seq, (*uhp).uh_alt_next) };
        unsafe { (*uhp).uh_alt_prev = resolve(seq, (*uhp).uh_alt_prev) };
    }

    // SAFETY: a live current buffer, by the contract above.
    let mut buf = unsafe { Buf::current() };
    u_blockfree(buf);
    // The links already name these headers; the store is what turns a
    // name back into one.
    for &uhp in headers {
        unsafe { header_adopt(buf, uhp) };
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
    buf.b_u_line_ptr = header.line_ptr;
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

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
