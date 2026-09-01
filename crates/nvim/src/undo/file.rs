//! The `.un~` file: where it lives, what it hashes, and how its
//! records are laid down and picked back up.
//!
//! Everything the file carries is a fixed-width big-endian integer written
//! by [`undo_write_bytes`] and range-checked on the way back in — everything
//! except the extmark payload, which is a native struct image. That one is
//! the reason [`unserialize_extmark`] validates: see [`format::decode_splice`].

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use core::ffi::CStr;

use super::format::*;
use super::tree::*;
use super::*;
use crate::message_fmt::c_str;
use crate::semsg;

/// The SHA-256 of every line of `buf`, each followed by a NUL separator.
///
/// This is what an undo file stakes its claim on: a buffer whose hash has
/// moved is holding different text, and the tree in the file describes text
/// that is no longer there.
///
/// # Safety
///
/// `hash` points at [`UNDO_HASH_SIZE`] writable bytes.
pub unsafe fn u_compute_hash(buf: Buf, hash: *mut uint8_t) {
    let mut ctx = Sha256::new();
    for lnum in 1..=buf.b_ml.ml_line_count {
        // SAFETY: a live buffer, so every line up to its own count is there.
        let line: *mut c_char = unsafe { ml_get_buf(buf.raw(), lnum) };
        // The terminating NUL goes in too, as a line separator.
        // SAFETY: that line, NUL-terminated, as `ml_get_buf` hands it back.
        let bytes =
            unsafe { ::core::slice::from_raw_parts(line.cast(), cstr::bytes_at(line).len() + 1) };
        ctx.update(bytes);
    }
    // SAFETY: `hash` points at `UNDO_HASH_SIZE` writable bytes, by the
    // contract above.
    let out = unsafe { ::core::slice::from_raw_parts_mut(hash, SHA256_SUM_SIZE) };
    out.copy_from_slice(&ctx.finish());
}

/// The undo file `'undodir'` picks for the file at `buf_ffname`, as a fresh
/// allocation the caller frees, or NULL when no directory in the list will
/// have it.
///
/// `reading` asks for a file that already exists; writing instead creates
/// the last directory in the list if it has to.
///
/// # Safety
///
/// `buf_ffname` is NULL or a NUL-terminated absolute path.
pub unsafe fn u_get_undo_file_name(buf_ffname: *const c_char, reading: bool) -> *mut c_char {
    if buf_ffname.is_null() {
        return ptr::null_mut();
    }
    let mut resolved: [c_char; MAXPATHL as usize] = [0; MAXPATHL as usize];
    // SAFETY: a NUL-terminated path, by the contract above, and `MAXPATHL`
    // bytes for the name it resolves to.
    let resolved_ok = unsafe { resolve_symlink(buf_ffname, resolved.as_mut_ptr()) }.is_ok();
    let ffname: *const c_char = if resolved_ok {
        resolved.as_ptr()
    } else {
        buf_ffname
    };

    // The name under a directory in 'undodir': the whole path with its
    // separators turned into '%', so that two files with the same tail
    // do not collide. Built once, whatever the list holds.
    let mut munged: *mut c_char = ptr::null_mut();
    let mut undo_file_name: *mut c_char = ptr::null_mut();
    let mut dirp: *mut c_char = p_udir.get();
    // SAFETY: 'undodir' is a NUL-terminated option string, and the walk stops
    // at that NUL.
    while unsafe { *dirp } != NUL as c_char {
        let mut dir_name: [c_char; MAXPATHL as usize + 1] = [0; MAXPATHL as usize + 1];
        let comma: *mut c_char = c",".as_ptr().cast_mut();
        let max = MAXPATHL as size_t;
        // SAFETY: `dirp` points into the option string, and `dir_name` holds
        // the `MAXPATHL` bytes the copy may write.
        let dir_len = unsafe { copy_option_part(&raw mut dirp, dir_name.as_mut_ptr(), max, comma) };
        if dir_len == 1 && dir_name[0] == '.' as c_char {
            // "." means beside the file itself, as ".name.un~".
            // SAFETY: a NUL-terminated path, by the contract above.
            undo_file_name = unsafe { hidden_sibling(ffname) };
        } else {
            dir_name[dir_len] = NUL as c_char;
            // SAFETY: `dir_name` is NUL-terminated at `dir_len`, just above.
            unsafe { trim_path_separators(dir_name.as_mut_ptr(), dir_len) };
            // SAFETY: `dirp` still points into the option string.
            let last_in_list = unsafe { *dirp } == NUL as c_char;
            // SAFETY: `dir_name` is that same NUL-terminated directory name.
            if unsafe { dir_is_usable(dir_name.as_mut_ptr(), last_in_list, reading) } {
                if munged.is_null() {
                    // SAFETY: a NUL-terminated path, by the contract above.
                    munged = unsafe { munge_separators(ffname) };
                }
                // SAFETY: two NUL-terminated names; the result is a fresh
                // allocation.
                undo_file_name = unsafe { concat_fnames(dir_name.as_mut_ptr(), munged, true) };
            }
        }
        // SAFETY: a NUL-terminated name, built just above.
        if !undo_file_name.is_null() && (!reading || unsafe { os_path_exists(undo_file_name) }) {
            break;
        }
        // SAFETY: NULL, or the allocation this pass made.
        unsafe { xfree(undo_file_name.cast()) };
        undo_file_name = ptr::null_mut();
    }
    // SAFETY: NULL, or the allocation `munge_separators` made.
    unsafe { xfree(munged.cast()) };
    undo_file_name
}

/// `/a/b/name` as `/a/b/.name.un~`, freshly allocated.
///
/// # Safety
///
/// `ffname` is a NUL-terminated path.
unsafe fn hidden_sibling(ffname: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated path, by the contract above.
    let (whole, tail_at) = unsafe {
        let whole = CStr::from_ptr(ffname).to_bytes();
        let tail = path_tail(ffname.cast_mut());
        (whole, tail.offset_from(ffname).unsigned_abs())
    };
    let mut name = Vec::with_capacity(whole.len() + 6);
    name.extend_from_slice(&whole[..tail_at]);
    name.push(b'.');
    name.extend_from_slice(&whole[tail_at..]);
    name.extend_from_slice(b".un~");
    name.push(0);
    // SAFETY: NUL-terminated by the push above, and `xstrdup` copies it into
    // an allocation the caller owns.
    unsafe { xstrdup(name.as_ptr().cast()) }
}

/// `/a/b/name` as `%a%b%name`, freshly allocated.
///
/// The scan steps a character at a time, so a separator byte inside a
/// multibyte character is left alone.
///
/// # Safety
///
/// `ffname` is a NUL-terminated path.
unsafe fn munge_separators(ffname: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated path, by the contract above.
    let munged = unsafe { xstrdup(ffname) };
    let mut c = munged;
    // SAFETY (each region below): the copy is ours, the walk stops at its
    // NUL, and a character step stays inside the string.
    while unsafe { *c } != NUL as c_char {
        if vim_ispathsep(c_int::from(unsafe { *c })) {
            unsafe { *c = '%' as c_char };
        }
        c = unsafe { c.offset(utfc_ptr2len(c) as isize) };
    }
    munged
}

/// Cuts the trailing path separators off a directory name, in place.
///
/// # Safety
///
/// `dir_name` is a NUL-terminated string of `dir_len` characters.
unsafe fn trim_path_separators(dir_name: *mut c_char, dir_len: size_t) {
    if dir_len <= 1 {
        return;
    }
    // SAFETY: `dir_len` characters, by the contract above; the walk back
    // stops at the first byte that is not a separator.
    let mut p = unsafe { dir_name.add(dir_len - 1) };
    while vim_ispathsep(c_int::from(unsafe { *p })) {
        unsafe { *p = NUL as c_char };
        p = unsafe { p.offset(-1) };
    }
}

/// Whether undo files may be put in `dir_name`, creating it if this is the
/// last entry in `'undodir'` and we are about to write.
///
/// # Safety
///
/// `dir_name` is a NUL-terminated path.
unsafe fn dir_is_usable(dir_name: *mut c_char, last_in_list: bool, reading: bool) -> bool {
    // SAFETY: a NUL-terminated path, by the contract above.
    if unsafe { os_isdir(dir_name) } {
        return true;
    }
    if !last_in_list || reading {
        return false;
    }
    let mut failed_dir: *mut c_char = ptr::null_mut();
    // SAFETY: a NUL-terminated path, and a writable slot for the name of the
    // directory that could not be made.
    let ret = unsafe { os_mkdir_recurse(dir_name, 0o755, &raw mut failed_dir, ptr::null_mut()) };
    if ret == 0 {
        return true;
    }
    // SAFETY: `failed_dir` is the NUL-terminated name `os_mkdir_recurse` left
    // there, and so is what `uv_strerror` hands back.
    let (shown, why) = unsafe { (c_str(failed_dir), c_str(uv_strerror(ret))) };
    semsg!("E5003: Unable to create directory \"{shown}\" for undo file: {why}");
    unsafe { xfree(failed_dir.cast()) };
    false
}

/// `E825`: the file said something a well-formed undo file cannot say.
///
/// # Safety
///
/// Both arguments are NUL-terminated.
pub(crate) unsafe fn corruption_error(mesg: *const c_char, file_name: *const c_char) {
    // SAFETY: NUL-terminated strings, by the contract above.
    let (mesg, file_name) = unsafe { (c_str(mesg), c_str(file_name)) };
    semsg!("E825: Corrupted undo file ({mesg}): {file_name}");
}

/// Frees a header that never reached a buffer's store, and its entries.
///
/// # Safety
///
/// `uhp` points at a live header nothing else owns.
pub(crate) unsafe fn u_free_uhp(uhp: *mut u_header_T) {
    // SAFETY: a live header, by the contract above.
    let mut uep: *mut u_entry_T = unsafe { (*uhp).uh_entry };
    while !uep.is_null() {
        // SAFETY: the entry list is walked one node ahead of the free.
        let next: *mut u_entry_T = unsafe { (*uep).ue_next };
        // SAFETY: a live entry holding `ue_size` lines.
        unsafe { u_freeentry(uep, (*uep).ue_size as c_int) };
        uep = next;
    }
    // SAFETY: the header itself, which nothing else owns.
    unsafe { xfree(uhp.cast()) };
}

/// Writes the file header: the magic, the version, the buffer hash, the `U`
/// shadow line and the three tree heads.
///
/// # Safety
///
/// `bi` is open for writing on a live buffer and `hash` points at
/// [`UNDO_HASH_SIZE`] readable bytes.
pub(crate) unsafe fn serialize_header(bi: *mut bufinfo_T, hash: *mut uint8_t) -> bool {
    // SAFETY: an open file on a live buffer, by the contract above.
    let buf: Buf = unsafe { (*bi).bi_buf };
    // SAFETY: as above.
    let fp: *mut FILE = unsafe { (*bi).bi_fp };
    // SAFETY: the magic is a readable byte array, and `fp` is open for
    // writing.
    if unsafe { fwrite(UF_START_MAGIC.as_ptr().cast(), UF_START_MAGIC.len(), 1, fp) } != 1 {
        return false;
    }
    // SAFETY: an open file, and `hash` readable for `UNDO_HASH_SIZE` bytes,
    // both by the contract above.
    unsafe { undo_write_bytes(bi, UF_VERSION as uintmax_t, 2) };
    if !unsafe { undo_write(bi, hash, UNDO_HASH_SIZE as size_t) } {
        return false;
    }
    unsafe { undo_write_bytes(bi, buf.b_ml.ml_line_count as uintmax_t, 4) };

    let line_ptr = buf.b_u_line_ptr;
    // SAFETY: the `U` shadow line is NULL or NUL-terminated.
    let len: size_t = if line_ptr.is_null() {
        0
    } else {
        unsafe { cstr::bytes_at(line_ptr) }.len()
    };
    // SAFETY: an open file, and `len` readable bytes of the shadow line; the
    // buffer stays live for the fields read off it.
    unsafe { undo_write_bytes(bi, len as uintmax_t, 4) };
    if len > 0 && !unsafe { undo_write(bi, line_ptr.cast(), len) } {
        return false;
    }
    unsafe { undo_write_bytes(bi, buf.b_u_line_lnum as uintmax_t, 4) };
    unsafe { undo_write_bytes(bi, buf.b_u_line_colnr as uintmax_t, 4) };

    unsafe { put_header_link(bi, buf.b_u_oldhead) };
    unsafe { put_header_link(bi, buf.b_u_newhead) };
    unsafe { put_header_link(bi, buf.b_u_curhead) };
    unsafe { undo_write_bytes(bi, buf.b_u_numhead as uintmax_t, 4) };
    unsafe { undo_write_bytes(bi, buf.b_u_seq_last as uintmax_t, 4) };
    unsafe { undo_write_bytes(bi, buf.b_u_seq_cur as uintmax_t, 4) };
    unsafe { put_time(bi, buf.b_u_time_cur) };
    unsafe { put_optional_field(bi, UF_LAST_SAVE_NR, buf.b_u_save_nr_last) };
    true
}

/// Writes one undo header and everything hanging off it: its four links, its
/// cursor and marks, its entries, and its extmark records.
///
/// # Safety
///
/// `bi` is open for writing and `uhp` points at a live header.
pub(crate) unsafe fn serialize_uhp(bi: *mut bufinfo_T, uhp: *mut u_header_T) -> bool {
    // SAFETY: an open file, by the contract above.
    if !unsafe { undo_write_bytes(bi, UF_HEADER_MAGIC as uintmax_t, 2) } {
        return false;
    }
    // SAFETY: an open file and a live header, by the contract above.
    unsafe { put_header_link(bi, (*uhp).uh_next) };
    unsafe { put_header_link(bi, (*uhp).uh_prev) };
    unsafe { put_header_link(bi, (*uhp).uh_alt_next) };
    unsafe { put_header_link(bi, (*uhp).uh_alt_prev) };
    unsafe { undo_write_bytes(bi, (*uhp).uh_seq as uintmax_t, 4) };
    unsafe { serialize_pos(bi, (*uhp).uh_cursor) };
    unsafe { undo_write_bytes(bi, (*uhp).uh_cursor_vcol as uintmax_t, 4) };
    unsafe { undo_write_bytes(bi, (*uhp).uh_flags as uintmax_t, 2) };
    for mark in unsafe { &(*uhp).uh_namedm } {
        unsafe { serialize_pos(bi, mark.mark) };
    }
    unsafe { serialize_visualinfo(bi, &raw const (*uhp).uh_visual) };
    unsafe { put_time(bi, (*uhp).uh_time) };
    unsafe { put_optional_field(bi, UHP_SAVE_NR, (*uhp).uh_save_nr) };

    // SAFETY: a live header, whose entry list ends in NULL.
    let mut uep: *mut u_entry_T = unsafe { (*uhp).uh_entry };
    while !uep.is_null() {
        // SAFETY (each region below): an open file, and a live entry off that
        // list.
        unsafe { undo_write_bytes(bi, UF_ENTRY_MAGIC as uintmax_t, 2) };
        if !unsafe { serialize_uep(bi, uep) } {
            return false;
        }
        uep = unsafe { (*uep).ue_next };
    }
    // SAFETY: an open file, by the contract above.
    unsafe { undo_write_bytes(bi, UF_ENTRY_END_MAGIC as uintmax_t, 2) };

    // SAFETY: a live header, whose extmark list holds `size` records.
    let count = unsafe { (*uhp).uh_extmark.size };
    for i in 0..count {
        // SAFETY: `i` is within that list, and each record's tag says which
        // union arm is live.
        if !unsafe { serialize_extmark(bi, *(*uhp).uh_extmark.items.add(i)) } {
            return false;
        }
    }
    // SAFETY: an open file, by the contract above.
    unsafe { undo_write_bytes(bi, UF_ENTRY_END_MAGIC as uintmax_t, 2) };
    true
}

/// Reads one undo header back, or NULL for a file that has already been
/// complained about.
///
/// The four link fields come back as the sequence numbers the file spells
/// them as, which is what a link already is; nothing is converted. What is
/// checked is that the header has a name at all: a sequence number of zero
/// or less is not one a buffer ever hands out.
///
/// # Safety
///
/// `bi` is open for reading and positioned just after the header magic;
/// `file_name` is NUL-terminated.
pub(crate) unsafe fn unserialize_uhp(
    bi: *mut bufinfo_T,
    file_name: *const c_char,
) -> *mut u_header_T {
    // SAFETY: a fresh allocation the size of a header, written before it is
    // read.
    let uhp: *mut u_header_T = unsafe { xmalloc(size_of::<u_header_T>()).cast() };
    // SAFETY: as above.
    unsafe { uhp.write(u_header_T::default()) };
    // SAFETY: an open file, by the contract above, and the header just made.
    unsafe { (*uhp).uh_next = UndoLink::to_seq(undo_read_4c(bi)) };
    unsafe { (*uhp).uh_prev = UndoLink::to_seq(undo_read_4c(bi)) };
    unsafe { (*uhp).uh_alt_next = UndoLink::to_seq(undo_read_4c(bi)) };
    unsafe { (*uhp).uh_alt_prev = UndoLink::to_seq(undo_read_4c(bi)) };
    unsafe { (*uhp).uh_seq = undo_read_4c(bi) };
    // SAFETY: the header being built.
    if unsafe { (*uhp).uh_seq } <= 0 {
        // SAFETY: a NUL-terminated name, and the allocation above, which
        // nothing else has seen.
        unsafe { corruption_error(c"uh_seq".as_ptr(), file_name) };
        unsafe { xfree(uhp.cast()) };
        return ptr::null_mut();
    }
    // SAFETY: an open file, and the header being built.
    unsafe { unserialize_pos(bi, &raw mut (*uhp).uh_cursor) };
    unsafe { (*uhp).uh_cursor_vcol = undo_read_4c(bi) };
    unsafe { (*uhp).uh_flags = undo_read_2c(bi) };
    let now: Timestamp = os_time();
    // SAFETY: the header's own mark array.
    for mark in unsafe { &mut (*uhp).uh_namedm } {
        // SAFETY: an open file, and a writable position.
        unsafe { unserialize_pos(bi, &raw mut mark.mark) };
        mark.timestamp = now;
        mark.fnum = 0;
    }
    // SAFETY: an open file, and the header being built.
    unsafe { unserialize_visualinfo(bi, &raw mut (*uhp).uh_visual) };
    unsafe { (*uhp).uh_time = undo_read_time(bi) };
    // Unlike the file header's, a truncated trailer here is corruption:
    // the entry records that must follow it are gone.
    // SAFETY: an open file, by the contract above.
    let Some(fields) = (unsafe { optional_fields(bi) }) else {
        // SAFETY: a NUL-terminated name, and the header this call owns.
        unsafe { corruption_error(c"truncated".as_ptr(), file_name) };
        unsafe { u_free_uhp(uhp) };
        return ptr::null_mut();
    };
    for (what, value) in fields {
        if what == UHP_SAVE_NR {
            // SAFETY: the header being built.
            unsafe { (*uhp).uh_save_nr = value };
        }
    }

    // SAFETY: an open file and the header being built; each of these frees
    // the header on the failure it reports.
    let read = unsafe {
        unserialize_entries(bi, uhp, file_name) && unserialize_extmarks(bi, uhp, file_name)
    };
    if !read {
        return ptr::null_mut();
    }
    uhp
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
///
/// # Safety
///
/// `bi` is open for reading and positioned at the first length byte.
pub(crate) unsafe fn optional_fields(bi: *mut bufinfo_T) -> Option<Vec<(c_int, c_int)>> {
    let mut fields = Vec::new();
    loop {
        // SAFETY (each region below): an open undo file, by the contract
        // above.
        let len = unsafe { undo_read_byte(bi) };
        if len == EOF {
            return None;
        }
        if len == 0 {
            return Some(fields);
        }
        let what = unsafe { undo_read_byte(bi) };
        if what == UF_LAST_SAVE_NR {
            fields.push((what, unsafe { undo_read_4c(bi) }));
        } else {
            for _ in 0..len {
                unsafe { undo_read_byte(bi) };
            }
        }
    }
}

/// Reads the run of entry records into `uhp`, up to the end marker. Frees
/// the header and says so on a corrupt one.
///
/// # Safety
///
/// As [`unserialize_uhp`], with `uhp` the header being read.
unsafe fn unserialize_entries(
    bi: *mut bufinfo_T,
    uhp: *mut u_header_T,
    file_name: *const c_char,
) -> bool {
    let mut last: *mut u_entry_T = ptr::null_mut();
    loop {
        // SAFETY: an open file, by the contract above.
        let c = unsafe { undo_read_2c(bi) };
        if c != UF_ENTRY_MAGIC {
            if c == UF_ENTRY_END_MAGIC {
                return true;
            }
            // SAFETY: a NUL-terminated name, and the header this call owns.
            unsafe { corruption_error(c"entry end".as_ptr(), file_name) };
            unsafe { u_free_uhp(uhp) };
            return false;
        }
        let mut error = false;
        // SAFETY: an open file, a writable flag, and a NUL-terminated name.
        let uep: *mut u_entry_T = unsafe { unserialize_uep(bi, &raw mut error, file_name) };
        // Linked in even when it is broken, so that freeing the header
        // frees the lines it did manage to read.
        if last.is_null() {
            // SAFETY: the header being built.
            unsafe { (*uhp).uh_entry = uep };
        } else {
            // SAFETY: the entry this loop linked in on its previous pass.
            unsafe { (*last).ue_next = uep };
        }
        last = uep;
        if uep.is_null() || error {
            // SAFETY: the header this call owns.
            unsafe { u_free_uhp(uhp) };
            return false;
        }
    }
}

/// Reads the run of extmark records into `uhp`'s list, up to the end marker.
///
/// # Safety
///
/// As [`unserialize_entries`].
unsafe fn unserialize_extmarks(
    bi: *mut bufinfo_T,
    uhp: *mut u_header_T,
    file_name: *const c_char,
) -> bool {
    loop {
        // SAFETY: an open file, by the contract above.
        let c = unsafe { undo_read_2c(bi) };
        if c != UF_ENTRY_MAGIC {
            if c == UF_ENTRY_END_MAGIC {
                return true;
            }
            // SAFETY: a NUL-terminated name, and the header this call owns.
            unsafe { corruption_error(c"entry end".as_ptr(), file_name) };
            unsafe { u_free_uhp(uhp) };
            return false;
        }
        // SAFETY: an open file and a NUL-terminated name, by the above.
        let Some(extup) = (unsafe { unserialize_extmark(bi, file_name) }) else {
            // The header's own list goes with it; upstream leaks the
            // header itself here, which cannot be right when every other
            // failure in this function frees it.
            // SAFETY: the header this call owns.
            unsafe { u_free_uhp(uhp) };
            return false;
        };
        // SAFETY: the header's own extmark list.
        unsafe { push_extmark(&raw mut (*uhp).uh_extmark, extup) };
    }
}

/// Appends one record to an undo header's extmark list, growing it the way
/// the rest of the tree grows a `kvec`.
///
/// # Safety
///
/// `list` points at an undo header's own extmark list.
unsafe fn push_extmark(list: *mut extmark_undo_vec_t, extup: ExtmarkUndoObject) {
    // SAFETY: a header's own list, by the contract above.
    let list = unsafe { &mut *list };
    if list.size == list.capacity {
        list.capacity = if list.capacity != 0 {
            list.capacity << 1
        } else {
            8
        };
        let bytes = size_of::<ExtmarkUndoObject>() * list.capacity;
        // SAFETY: `items` is NULL or this list's own allocation.
        list.items = unsafe { xrealloc(list.items.cast(), bytes) }.cast();
    }
    // SAFETY: the slot at `size` is inside the capacity checked just above.
    unsafe { list.items.add(list.size).write(extup) };
    list.size += 1;
}

/// Writes one extmark record: its type, then its payload.
///
/// Only splices and moves are written; a `kExtmarkSavePos` record describes
/// marks in a buffer this file will be read into fresh, and is dropped.
///
/// # Safety
///
/// `bi` is open for writing.
pub(crate) unsafe fn serialize_extmark(bi: *mut bufinfo_T, extup: ExtmarkUndoObject) -> bool {
    let mut image = match &extup {
        ExtmarkUndoObject::Splice(splice) => encode_splice(splice),
        ExtmarkUndoObject::Move(move_0) => encode_move(move_0),
        ExtmarkUndoObject::SavePos(_) => return true,
    };
    // SAFETY: an open file, by the contract above, and `image.len()` readable
    // bytes of our own array.
    unsafe { undo_write_bytes(bi, UF_ENTRY_MAGIC as uintmax_t, 2) };
    unsafe { undo_write_bytes(bi, extup.wire_type() as uintmax_t, 4) };
    unsafe { undo_write(bi, image.as_mut_ptr(), image.len()) }
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
/// # Safety
///
/// `bi` is open for reading and positioned just after the entry magic;
/// `file_name` is NUL-terminated.
pub(crate) unsafe fn unserialize_extmark(
    bi: *mut bufinfo_T,
    file_name: *const c_char,
) -> Option<ExtmarkUndoObject> {
    // SAFETY: an open file, by the contract above.
    let type_0: UndoObjectType = unsafe { undo_read_4c(bi) } as UndoObjectType;
    let mut image = [0u8; EXTMARK_PAYLOAD_LEN];
    // SAFETY (each region below): an open file and a NUL-terminated name, by
    // the contract above; `image` is that many writable bytes of our own.
    if type_0 == kExtmarkSplice {
        if !unsafe { undo_read(bi, image.as_mut_ptr(), image.len()) } {
            return unsafe { refuse_extmark(c"extmark truncated", file_name) };
        }
        match decode_splice(&image) {
            Some(splice) => Some(ExtmarkUndoObject::Splice(splice)),
            None => unsafe { refuse_extmark(c"extmark splice", file_name) },
        }
    } else if type_0 == kExtmarkMove {
        if !unsafe { undo_read(bi, image.as_mut_ptr(), image.len()) } {
            return unsafe { refuse_extmark(c"extmark truncated", file_name) };
        }
        match decode_move(&image) {
            Some(move_0) => Some(ExtmarkUndoObject::Move(move_0)),
            None => unsafe { refuse_extmark(c"extmark move", file_name) },
        }
    } else {
        unsafe { refuse_extmark(c"extmark type", file_name) }
    }
}

/// Reports a bad extmark record and answers `None`, so that the caller reads
/// as one `return`.
///
/// # Safety
///
/// `file_name` is NUL-terminated.
unsafe fn refuse_extmark(mesg: &CStr, file_name: *const c_char) -> Option<ExtmarkUndoObject> {
    // SAFETY: NUL-terminated strings, by the contract above.
    unsafe { corruption_error(mesg.as_ptr(), file_name) };
    None
}

/// Writes one undo entry: where the lines it saved came from, and the lines.
///
/// # Safety
///
/// `bi` is open for writing and `uep` points at a live entry holding
/// `ue_size` lines.
pub(crate) unsafe fn serialize_uep(bi: *mut bufinfo_T, uep: *mut u_entry_T) -> bool {
    // SAFETY: an open file and a live entry, by the contract above.
    unsafe { undo_write_bytes(bi, (*uep).ue_top as uintmax_t, 4) };
    unsafe { undo_write_bytes(bi, (*uep).ue_bot as uintmax_t, 4) };
    unsafe { undo_write_bytes(bi, (*uep).ue_lcount as uintmax_t, 4) };
    unsafe { undo_write_bytes(bi, (*uep).ue_size as uintmax_t, 4) };
    // SAFETY: a live entry, by the contract above.
    let size = unsafe { (*uep).ue_size } as size_t;
    for i in 0..size {
        // SAFETY: the entry holds `ue_size` NUL-terminated lines.
        let line: *mut c_char = unsafe { *(*uep).ue_array.add(i) };
        // SAFETY: as above.
        let len = unsafe { cstr::bytes_at(line) }.len();
        // SAFETY: an open file, by the contract above.
        if !unsafe { undo_write_bytes(bi, len as uintmax_t, 4) } {
            return false;
        }
        // SAFETY: an open file, and `len` readable bytes of that line.
        if len > 0 && !unsafe { undo_write(bi, line.cast(), len) } {
            return false;
        }
    }
    true
}

/// Reads one undo entry back. A `true` in `*error` means the entry is there
/// but incomplete: the caller links it in anyway so that freeing the header
/// frees the lines that did arrive.
///
/// # Safety
///
/// `bi` is open for reading, `error` points at a writable `bool`, and
/// `file_name` is NUL-terminated.
pub(crate) unsafe fn unserialize_uep(
    bi: *mut bufinfo_T,
    error: *mut bool,
    file_name: *const c_char,
) -> *mut u_entry_T {
    // SAFETY: a fresh allocation the size of an entry, written before it is
    // read.
    let uep: *mut u_entry_T = unsafe { xmalloc(size_of::<u_entry_T>()).cast() };
    // SAFETY: as above.
    unsafe { uep.write(u_entry_T::default()) };
    // SAFETY: an open file, by the contract above, and the entry just made.
    unsafe { (*uep).ue_top = undo_read_4c(bi) };
    unsafe { (*uep).ue_bot = undo_read_4c(bi) };
    unsafe { (*uep).ue_lcount = undo_read_4c(bi) };
    unsafe { (*uep).ue_size = undo_read_4c(bi) };
    // SAFETY: the entry being built.
    let ue_size = unsafe { (*uep).ue_size };
    if ue_size < 0 {
        // A count, not an offset. Upstream widens it to `size_t` for the
        // loop below, so a negative one becomes about 2^64 iterations
        // over an array it never allocated.
        // SAFETY: a NUL-terminated name, the entry above, and a writable
        // flag, all by the contract above.
        unsafe { corruption_error(c"entry size".as_ptr(), file_name) };
        unsafe { (*uep).ue_size = 0 };
        unsafe { *error = true };
        return uep;
    }
    let size = ue_size as size_t;
    if ue_size > 0 && size < SIZE_MAX as usize / size_of::<*mut c_char>() {
        let bytes = size_of::<*mut c_char>() * size;
        // SAFETY: a fresh allocation of `bytes`, zeroed before it is read.
        let array: *mut *mut c_char = unsafe { xmalloc(bytes).cast() };
        // SAFETY: `size` pointer-sized slots, from that same allocation.
        unsafe { array.write_bytes(0, size) };
        // SAFETY: the entry being built.
        unsafe { (*uep).ue_array = array };
    }
    for i in 0..size {
        // SAFETY: an open file, by the contract above.
        let line_len = unsafe { undo_read_4c(bi) };
        let line: *mut c_char = if line_len >= 0 {
            // SAFETY: an open file, by the contract above.
            unsafe { undo_read_string(bi, line_len as size_t) }
        } else {
            // SAFETY: a NUL-terminated name, by the contract above.
            unsafe { corruption_error(c"line length".as_ptr(), file_name) };
            ptr::null_mut()
        };
        if line.is_null() {
            // SAFETY: a writable flag, by the contract above.
            unsafe { *error = true };
            return uep;
        }
        // SAFETY: `i` is below `ue_size`, the length the array was made with.
        unsafe { *(*uep).ue_array.add(i) = line };
    }
    uep
}

/// Writes a position as three four-byte fields.
///
/// # Safety
///
/// `bi` is open for writing.
pub(crate) unsafe fn serialize_pos(bi: *mut bufinfo_T, pos: pos_T) {
    // SAFETY: an open file, by the contract above.
    unsafe { undo_write_bytes(bi, pos.lnum as uintmax_t, 4) };
    unsafe { undo_write_bytes(bi, pos.col as uintmax_t, 4) };
    unsafe { undo_write_bytes(bi, pos.coladd as uintmax_t, 4) };
}

/// Reads a position back, clamping each field at zero: a negative line or
/// column is not a position, and every caller would have to check anyway.
///
/// # Safety
///
/// `bi` is open for reading and `pos` points at a writable position.
pub(crate) unsafe fn unserialize_pos(bi: *mut bufinfo_T, pos: *mut pos_T) {
    // SAFETY: an open file and a writable position, by the contract above.
    unsafe { (*pos).lnum = undo_read_4c(bi).max(0) };
    unsafe { (*pos).col = undo_read_4c(bi).max(0) };
    unsafe { (*pos).coladd = undo_read_4c(bi).max(0) };
}

/// Writes what a Visual selection covered.
///
/// # Safety
///
/// `bi` is open for writing and `info` points at a readable record.
pub(crate) unsafe fn serialize_visualinfo(bi: *mut bufinfo_T, info: *const visualinfo_T) {
    // SAFETY: an open file and a readable record, by the contract above.
    unsafe { serialize_pos(bi, (*info).vi_start) };
    unsafe { serialize_pos(bi, (*info).vi_end) };
    unsafe { undo_write_bytes(bi, (*info).vi_mode as uintmax_t, 4) };
    unsafe { undo_write_bytes(bi, (*info).vi_curswant as uintmax_t, 4) };
}

/// Reads a Visual selection back.
///
/// # Safety
///
/// `bi` is open for reading and `info` points at a writable record.
pub(crate) unsafe fn unserialize_visualinfo(bi: *mut bufinfo_T, info: *mut visualinfo_T) {
    // SAFETY: an open file and a writable record, by the contract above.
    unsafe { unserialize_pos(bi, &raw mut (*info).vi_start) };
    unsafe { unserialize_pos(bi, &raw mut (*info).vi_end) };
    unsafe { (*info).vi_mode = undo_read_4c(bi) };
    unsafe { (*info).vi_curswant = undo_read_4c(bi) };
}

/// Writes a timestamp as eight bytes. The one clock-dependent field in the
/// whole file.
///
/// # Safety
///
/// `bi` is open for writing.
unsafe fn put_time(bi: *mut bufinfo_T, when: time_t) {
    let mut buf: [uint8_t; 8] = [0; 8];
    // SAFETY: an eight-byte buffer, and an open file.
    unsafe { time_to_bytes(when, buf.as_mut_ptr()) };
    unsafe { undo_write(bi, buf.as_mut_ptr(), buf.len()) };
}

/// Writes the optional-field trailer that both the file header and every
/// undo header end with: one tagged four-byte field, then the terminator.
///
/// The reader skips a tag it does not know by the length written here, which
/// is what makes this an extension point.
///
/// # Safety
///
/// `bi` is open for writing.
unsafe fn put_optional_field(bi: *mut bufinfo_T, what: c_int, value: c_int) {
    // SAFETY: an open file, by the contract above.
    unsafe { undo_write_bytes(bi, 4, 1) };
    unsafe { undo_write_bytes(bi, what as uintmax_t, 1) };
    unsafe { undo_write_bytes(bi, value as uintmax_t, 4) };
    unsafe { undo_write_bytes(bi, 0, 1) };
}

/// Writes `len` bytes.
///
/// # Safety
///
/// `bi` is open for writing and `ptr` points at `len` readable bytes.
pub(crate) unsafe fn undo_write(bi: *mut bufinfo_T, ptr: *mut uint8_t, len: size_t) -> bool {
    // SAFETY: an open file and `len` readable bytes, by the contract above.
    unsafe { fwrite(ptr.cast(), len, 1, (*bi).bi_fp) == 1 }
}

/// Writes `nr` as a `len`-byte big-endian field. See [`encode_be`].
///
/// # Safety
///
/// `bi` is open for writing.
pub(crate) unsafe fn undo_write_bytes(bi: *mut bufinfo_T, nr: uintmax_t, len: size_t) -> bool {
    let mut buf = encode_be(nr, len);
    // SAFETY: an open file, and `len` bytes of `buf`.
    unsafe { undo_write(bi, buf.as_mut_ptr(), len) }
}

/// Writes one link as the four big-endian bytes of the sequence number it
/// already holds -- 0 for a link to nothing, which is what the reader takes
/// it back as.
///
/// # Safety
///
/// `bi` is open for writing.
pub(crate) unsafe fn put_header_link(bi: *mut bufinfo_T, link: UndoLink) {
    debug_assert!(link.seq() >= 0, "an undo link is 0 or a sequence number");
    // SAFETY: an open file, by the contract above.
    unsafe { undo_write_bytes(bi, link.seq() as uintmax_t, 4) };
}

/// Reads a four-byte big-endian field.
///
/// # Safety
///
/// `bi` is open for reading.
pub(crate) unsafe fn undo_read_4c(bi: *mut bufinfo_T) -> c_int {
    // SAFETY: an open file, by the contract above.
    unsafe { get4c((*bi).bi_fp) }
}

/// Reads a two-byte big-endian field.
///
/// # Safety
///
/// `bi` is open for reading.
pub(crate) unsafe fn undo_read_2c(bi: *mut bufinfo_T) -> c_int {
    // SAFETY: an open file, by the contract above.
    unsafe { get2c((*bi).bi_fp) }
}

/// Reads one byte, or [`EOF`].
///
/// # Safety
///
/// `bi` is open for reading.
pub(crate) unsafe fn undo_read_byte(bi: *mut bufinfo_T) -> c_int {
    // SAFETY: an open file, by the contract above.
    unsafe { getc((*bi).bi_fp) }
}

/// Reads an eight-byte timestamp.
///
/// # Safety
///
/// `bi` is open for reading.
pub(crate) unsafe fn undo_read_time(bi: *mut bufinfo_T) -> time_t {
    // SAFETY: an open file, by the contract above.
    unsafe { get8ctime((*bi).bi_fp) }
}

/// Reads `size` bytes, zeroing the buffer if the file ran out.
///
/// # Safety
///
/// `bi` is open for reading and `buffer` points at `size` writable bytes.
pub(crate) unsafe fn undo_read(bi: *mut bufinfo_T, buffer: *mut uint8_t, size: size_t) -> bool {
    // SAFETY: an open file and `size` writable bytes, by the contract above.
    if unsafe { fread(buffer.cast(), size, 1, (*bi).bi_fp) } == 1 {
        return true;
    }
    // SAFETY: as above.
    unsafe { buffer.write_bytes(0, size) };
    false
}

/// Reads a `len`-byte string, NUL-terminated, or NULL if the file ran out.
///
/// # Safety
///
/// `bi` is open for reading.
pub(crate) unsafe fn undo_read_string(bi: *mut bufinfo_T, len: size_t) -> *mut c_char {
    // SAFETY: an allocation of `len + 1` bytes, zeroed.
    let ptr: *mut c_char = unsafe { xmallocz(len).cast() };
    // SAFETY: an open file, and `len` writable bytes of that allocation.
    if len > 0 && !unsafe { undo_read(bi, ptr.cast(), len) } {
        // SAFETY: our own allocation.
        unsafe { xfree(ptr.cast()) };
        return ptr::null_mut();
    }
    ptr
}
