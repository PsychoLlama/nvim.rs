//! The typeahead buffer: [`typebuf`], the queue `vgetc` reads from.
//!
//! `typebuf.tb_buf` holds bytes waiting to be interpreted, with a parallel
//! `tb_noremap` array saying how much remapping each byte is still allowed.
//! [`ins_typebuf`] pushes (that is what `feedkeys()` and every mapping
//! expansion do) and [`del_typebuf`] pops; the pair must keep `tb_off`,
//! `tb_len`, `tb_maplen`, `tb_silent` and `tb_no_abbr_cnt` consistent.
//!
//! The buffer is deliberately not a `Vec`. It has room in *front* of the
//! valid bytes (`tb_off`) so that a mapping's RHS can be pushed without
//! moving what follows, and both arrays are addressed by raw pointers that
//! `vgetorpeek` holds across calls that can reallocate them — which is what
//! `tb_change_cnt` exists to detect.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use core::ffi::{c_char, c_int};
use core::ptr;

/// Longest UTF-8 sequence a character can occupy.
const MB_MAXBYTES: usize = 21;

/// Size of the two static initial buffers, upstream's `TYPELEN_INIT`.
///
/// `tb_buf` has three parts: room in front for the result of mappings, the
/// middle for typeahead, and room at the end for new characters.
const TYPELEN_INIT: c_int = 5 * (MAXMAPLEN as c_int + 3);

/// Where the valid bytes start in a freshly allocated buffer: enough room in
/// front that a mapping's RHS can be inserted without moving anything.
const HEAD_ROOM: c_int = MAXMAPLEN as c_int + 4;

/// Point `typebuf` at the static initial buffers, if it has none.
///
/// `xmalloc` is not usable here: out of memory it would be impossible to type
/// anything, which is the one situation where typing has to keep working.
///
/// # Safety
/// Callable at any time.
pub(crate) unsafe fn init_typebuf() {
    unsafe {
        let tb = typebuf.ptr();
        if !(*tb).tb_buf.is_null() {
            return;
        }
        (*tb).tb_buf = typebuf_init.ptr().cast();
        (*tb).tb_noremap = noremapbuf_init.ptr().cast();
        (*tb).tb_buflen = TYPELEN_INIT;
        (*tb).tb_len = 0;
        (*tb).tb_off = HEAD_ROOM;
        (*tb).tb_change_cnt = 1;
    }
}

/// Whether the keys being read now may not be remapped.
pub fn noremap_keys() -> bool {
    KeyNoremap.get() & (RM_NONE as c_int | RM_SCRIPT as c_int) != 0
}

/// Insert `str` into the typeahead buffer at `offset`.
///
/// `noremap` says how much of it may be mapped again: `REMAP_YES` all of it,
/// `REMAP_NONE` none, `REMAP_SCRIPT` only script-local mappings,
/// `REMAP_SKIP` only the first character (but abbreviations still apply), and
/// a positive count that many characters.
///
/// With `nottyped` the string does not set `KeyTyped` — do not use it with a
/// non-zero `offset`. With `silent`, `cmd_silent` is set when the characters
/// are read back. Answers `FAIL` when the buffer would overflow an `int`.
///
/// # Safety
/// `str` must point at a NUL-terminated string, and `offset` must be within
/// the current typeahead.
pub unsafe fn ins_typebuf(
    str: *mut c_char,
    noremap: c_int,
    offset: c_int,
    nottyped: bool,
    silent: bool,
) -> c_int {
    unsafe {
        init_typebuf();
        let tb = typebuf.ptr();
        (*tb).tb_change_cnt += 1;
        if (*tb).tb_change_cnt == 0 {
            (*tb).tb_change_cnt = 1;
        }
        state_no_longer_safe(c"ins_typebuf()".as_ptr());

        let addlen = strlen(str) as c_int;

        if offset == 0 && addlen <= (*tb).tb_off {
            // Easy case: there is room in front of the valid bytes.
            (*tb).tb_off -= addlen;
            ptr::copy_nonoverlapping(
                str.cast::<u8>(),
                (*tb).tb_buf.offset((*tb).tb_off as isize),
                addlen as usize,
            );
        } else if (*tb).tb_len == 0 && (*tb).tb_buflen >= addlen + 3 * HEAD_ROOM {
            // Buffer is empty and the string fits: centre it, leaving room
            // before and after.
            (*tb).tb_off = ((*tb).tb_buflen - addlen - 3 * HEAD_ROOM) / 2;
            ptr::copy_nonoverlapping(
                str.cast::<u8>(),
                (*tb).tb_buf.offset((*tb).tb_off as isize),
                addlen as usize,
            );
        } else {
            // Reallocate. There must always be room for 3 * HEAD_ROOM bytes,
            // and some extra so this does not happen every time.
            let extra = addlen + HEAD_ROOM + 4 * HEAD_ROOM;
            if (*tb).tb_len > c_int::MAX - extra {
                // The string is getting too long for a 32-bit int.
                emsg(gettext(&raw const e_toocompl as *const c_char)); // also flushes the buffers
                setcursor();
                return FAIL;
            }
            let newlen = (*tb).tb_len + extra;
            let buf = xmalloc(newlen as usize).cast::<u8>();
            let noremaps = xmalloc(newlen as usize).cast::<u8>();
            (*tb).tb_buflen = newlen;

            // Old bytes before the insertion point, then the new ones, then
            // the old bytes after it -- including the NUL at the end.
            let old = (*tb).tb_buf.offset((*tb).tb_off as isize);
            let at = buf.offset(HEAD_ROOM as isize);
            ptr::copy_nonoverlapping(old, at, offset as usize);
            ptr::copy_nonoverlapping(
                str.cast::<u8>(),
                at.offset(offset as isize),
                addlen as usize,
            );
            let tail = (*tb).tb_len - offset + 1;
            debug_assert!(tail > 0);
            ptr::copy_nonoverlapping(
                old.offset(offset as isize),
                at.offset(offset as isize).offset(addlen as isize),
                tail as usize,
            );
            if (*tb).tb_buf != typebuf_init.ptr().cast() {
                xfree((*tb).tb_buf.cast());
            }
            (*tb).tb_buf = buf;

            // The same for tb_noremap, which has no terminator to carry.
            let old = (*tb).tb_noremap.offset((*tb).tb_off as isize);
            let at = noremaps.offset(HEAD_ROOM as isize);
            ptr::copy_nonoverlapping(old, at, offset as usize);
            ptr::copy_nonoverlapping(
                old.offset(offset as isize),
                at.offset(offset as isize).offset(addlen as isize),
                ((*tb).tb_len - offset) as usize,
            );
            if (*tb).tb_noremap != noremapbuf_init.ptr().cast() {
                xfree((*tb).tb_noremap.cast());
            }
            (*tb).tb_noremap = noremaps;

            (*tb).tb_off = HEAD_ROOM;
        }
        (*tb).tb_len += addlen;

        // What the characters that may not be remapped are marked with, and
        // how many of them there are.
        let val = if noremap == REMAP_SCRIPT {
            RM_SCRIPT as c_int
        } else if noremap == REMAP_SKIP {
            RM_ABBR as c_int
        } else {
            RM_NONE as c_int
        };
        let noremapped = if noremap == REMAP_SKIP {
            1
        } else if noremap < 0 {
            addlen
        } else {
            noremap
        };
        for i in 0..addlen {
            let flags = if i < noremapped { val } else { RM_YES as c_int };
            *(*tb)
                .tb_noremap
                .offset(((*tb).tb_off + i + offset) as isize) = flags as u8;
        }

        // tb_maplen and tb_silent only remember the length of the mapped
        // and/or silent run at the *start* of the buffer, on the assumption
        // that a mapped sequence does not produce typed characters.
        if nottyped || (*tb).tb_maplen > offset {
            (*tb).tb_maplen += addlen;
        }
        if silent || (*tb).tb_silent > offset {
            (*tb).tb_silent += addlen;
            cmd_silent.set(true);
        }
        if (*tb).tb_no_abbr_cnt != 0 && offset == 0 {
            // ... and is not to be used for abbreviations.
            (*tb).tb_no_abbr_cnt += addlen;
        }

        OK
    }
}

/// Put character `c` back into the typeahead buffer, restoring the flags that
/// belong to it from `cmd_silent`, `KeyTyped` and `KeyNoremap`.
///
/// Used for a character `vgetc` handed out and the caller then decided not to
/// consume. With `on_key_ignore` the bytes are not reported to `vim.on_key()`.
/// Answers how many bytes went in.
///
/// # Safety
/// Callable at any time.
pub unsafe fn ins_char_typebuf(c: c_int, modifiers: c_int, on_key_ignore: bool) -> c_int {
    unsafe {
        // Room for the modifier prefix plus a K_SPECIAL-escaped character.
        let mut buf = [0 as c_char; MB_MAXBYTES * 3 + 4];
        let len = special_to_buf(c, modifiers, true, buf.as_mut_ptr()) as usize;
        debug_assert!(len < buf.len());
        buf[len] = 0;
        ins_typebuf(
            buf.as_mut_ptr(),
            KeyNoremap.get(),
            0,
            !KeyTyped.get(),
            cmd_silent.get(),
        );
        if KeyTyped.get() && on_key_ignore {
            on_key_ignore_len.set(on_key_ignore_len.get() + len);
        }
        len as c_int
    }
}

/// Whether the typeahead buffer changed while waiting for a character —
/// which happens when a message arrives from a client or from `feedkeys()`.
///
/// The test is deliberately generic: when `tb_buf` changed it was reallocated
/// and the old pointer is dead, and `tb_off` may have moved so that a write
/// through the old one would land on bytes that were just added.
///
/// # Safety
/// Callable at any time.
pub unsafe fn typebuf_changed(tb_change_cnt: c_int) -> bool {
    unsafe {
        tb_change_cnt != 0
            && ((*typebuf.ptr()).tb_change_cnt != tb_change_cnt || typebuf_was_filled.get())
    }
}

/// Whether every character in the typeahead was actually typed, rather than
/// produced by a mapping or by `:normal`.
///
/// # Safety
/// Callable at any time.
pub unsafe fn typebuf_typed() -> c_int {
    c_int::from(unsafe { (*typebuf.ptr()).tb_maplen } == 0)
}

/// How many characters of the typeahead were mapped rather than typed.
///
/// # Safety
/// Callable at any time.
pub unsafe fn typebuf_maplen() -> c_int {
    unsafe { (*typebuf.ptr()).tb_maplen }
}

/// Remove `len` characters at `offset` from the typeahead buffer.
///
/// # Safety
/// `offset + len` must be within the current typeahead.
pub unsafe fn del_typebuf(len: c_int, offset: c_int) {
    unsafe {
        if len == 0 {
            return; // nothing to do
        }
        let tb = typebuf.ptr();
        (*tb).tb_len -= len;

        if offset == 0 && (*tb).tb_buflen - ((*tb).tb_off + len) >= 3 * MAXMAPLEN as c_int + 3 {
            // Easy case: just leave the bytes in front and step over them.
            (*tb).tb_off += len;
        } else {
            // Otherwise both arrays have to be moved down.
            let from = (*tb).tb_off + offset;
            if (*tb).tb_off > MAXMAPLEN as c_int {
                // Leave some extra room at the end to avoid a reallocation.
                ptr::copy(
                    (*tb).tb_buf.offset((*tb).tb_off as isize),
                    (*tb).tb_buf.offset(MAXMAPLEN as isize),
                    offset as usize,
                );
                ptr::copy(
                    (*tb).tb_noremap.offset((*tb).tb_off as isize),
                    (*tb).tb_noremap.offset(MAXMAPLEN as isize),
                    offset as usize,
                );
                (*tb).tb_off = MAXMAPLEN as c_int;
            }
            // Include the NUL at the end for tb_buf; tb_noremap has none.
            let tail = (*tb).tb_len - offset + 1;
            debug_assert!(tail > 0);
            ptr::copy(
                (*tb).tb_buf.offset((from + len) as isize),
                (*tb).tb_buf.offset(((*tb).tb_off + offset) as isize),
                tail as usize,
            );
            ptr::copy(
                (*tb).tb_noremap.offset((from + len) as isize),
                (*tb).tb_noremap.offset(((*tb).tb_off + offset) as isize),
                ((*tb).tb_len - offset) as usize,
            );
        }

        // Each of the three run lengths shrinks only by the part of the
        // deletion that fell inside it.
        for run in [
            &raw mut (*tb).tb_maplen,
            &raw mut (*tb).tb_silent,
            &raw mut (*tb).tb_no_abbr_cnt,
        ] {
            if *run > offset {
                *run = if *run < offset + len {
                    offset
                } else {
                    *run - len
                };
            }
        }

        // Text received from a client or from feedkeys() is no longer what is
        // in the buffer.
        typebuf_was_filled.set(false);
        (*tb).tb_change_cnt += 1;
        if (*tb).tb_change_cnt == 0 {
            (*tb).tb_change_cnt = 1;
        }
    }
}

/// Undo the last [`gotchars`] for `len` bytes, so that putting a typed
/// character back into the typeahead does not record it twice.
///
/// Only the recording is affected.
///
/// # Safety
/// `len` must be at most what the last `gotchars` recorded.
pub unsafe fn ungetchars(len: c_int) {
    unsafe {
        if reg_recording.get() == 0 {
            return;
        }
        delete_buff_tail(recordbuff.ptr(), len);
        last_recorded_len.set(last_recorded_len.get() - len as usize);
    }
}

/// Sync undo, as reading typed characters out of the typeahead should.
///
/// Not in Insert or Cmdline mode unless a cursor key was used, and not while
/// reading a script file — in both cases the keys are one edit, not several.
///
/// # Safety
/// Callable at any time.
pub unsafe fn may_sync_undo() {
    unsafe {
        if (State.get() & (MODE_INSERT | MODE_CMDLINE) == 0 || arrow_used.get())
            && curscript.get() < 0
        {
            u_sync(false);
        }
    }
}

/// Empty `typebuf` and give it freshly allocated buffers.
///
/// # Safety
/// The current buffers must already have been saved or freed.
pub(crate) unsafe fn alloc_typebuf() {
    unsafe {
        let tb = typebuf.ptr();
        (*tb).tb_buf = xmalloc(TYPELEN_INIT as usize).cast();
        (*tb).tb_noremap = xmalloc(TYPELEN_INIT as usize).cast();
        (*tb).tb_buflen = TYPELEN_INIT;
        (*tb).tb_off = HEAD_ROOM; // can insert without reallocating
        (*tb).tb_len = 0;
        (*tb).tb_maplen = 0;
        (*tb).tb_silent = 0;
        (*tb).tb_no_abbr_cnt = 0;
        (*tb).tb_change_cnt += 1;
        if (*tb).tb_change_cnt == 0 {
            (*tb).tb_change_cnt = 1;
        }
        typebuf_was_filled.set(false);
    }
}

/// Free `typebuf`'s buffers.
///
/// Freeing the two *static* initial buffers would be a bug, so that is
/// reported rather than done.
///
/// # Safety
/// Nothing may hold a pointer into either buffer.
pub(crate) unsafe fn free_typebuf() {
    unsafe {
        let tb = typebuf.ptr();
        if (*tb).tb_buf == typebuf_init.ptr().cast() {
            internal_error(c"Free typebuf 1".as_ptr());
        } else {
            xfree((*tb).tb_buf.cast());
            (*tb).tb_buf = ptr::null_mut();
        }
        if (*tb).tb_noremap == noremapbuf_init.ptr().cast() {
            internal_error(c"Free typebuf 2".as_ptr());
        } else {
            xfree((*tb).tb_noremap.cast());
            (*tb).tb_noremap = ptr::null_mut();
        }
    }
}

/// Put the current typeahead aside for the script `:source!` is about to
/// read, and start a fresh one.
///
/// # Safety
/// `curscript` must name an open script.
pub(crate) unsafe fn save_typebuf() {
    unsafe {
        debug_assert!(curscript.get() >= 0);
        init_typebuf();
        (*saved_typebuf.ptr())[curscript.get() as usize] = typebuf.get();
        alloc_typebuf();
    }
}

/// Whether the character `vungetc` put back can be handed out now.
///
/// It cannot when it was not stuffed and something has since been added to
/// the stuff buffer: those characters have to come first.
///
/// # Safety
/// Callable at any time.
pub(crate) unsafe fn can_get_old_char() -> bool {
    unsafe { old_char.get() != -1 && (old_KeyStuffed.get() != 0 || stuff_empty()) }
}

/// Save all three kinds of typeahead, so that a prompt really has to be
/// answered by the user.
///
/// # Safety
/// `tp` must point at writable storage that outlives the matching
/// [`restore_typeahead`].
pub unsafe fn save_typeahead(tp: *mut tasave_T) {
    unsafe {
        (*tp).save_typebuf = typebuf.get();
        alloc_typebuf();
        (*tp).typebuf_valid = true;
        (*tp).old_char = old_char.get();
        (*tp).old_mod_mask = old_mod_mask.get();
        old_char.set(-1);

        (*tp).save_readbuf1 = readbuf1.get();
        (*readbuf1.ptr()).bh_first.b_next = ptr::null_mut();
        (*tp).save_readbuf2 = readbuf2.get();
        (*readbuf2.ptr()).bh_first.b_next = ptr::null_mut();
    }
}

/// Put back what [`save_typeahead`] saved, freeing what was read in the
/// meantime. Can only be called once per save.
///
/// # Safety
/// `tp` must be the one a matching [`save_typeahead`] filled.
pub unsafe fn restore_typeahead(tp: *mut tasave_T) {
    unsafe {
        if (*tp).typebuf_valid {
            free_typebuf();
            typebuf.set((*tp).save_typebuf);
        }
        old_char.set((*tp).old_char);
        old_mod_mask.set((*tp).old_mod_mask);

        free_buff(readbuf1.ptr());
        readbuf1.set((*tp).save_readbuf1);
        free_buff(readbuf2.ptr());
        readbuf2.set((*tp).save_readbuf2);
    }
}
