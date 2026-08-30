//! The typeahead buffer: [`TypeAhead`], the queue `vgetc` reads from.
//!
//! It holds bytes waiting to be interpreted, with a parallel `noremap` array
//! saying how much remapping each byte is still allowed. [`ins_typebuf`]
//! pushes (that is what `feedkeys()` and every mapping expansion do) and
//! [`del_typebuf`] pops; the pair must keep `off`, `len`, `maplen`, `silent`
//! and `no_abbr_cnt` consistent, which is why both are methods on the type
//! rather than field pokes spread over four files.
//!
//! The storage is deliberately not a `Vec`. It has room in *front* of the
//! valid bytes (`off`) so that a mapping's RHS can be pushed without moving
//! what follows, and both arrays are addressed by raw pointers that
//! `vgetorpeek` holds across calls that can reallocate them — which is what
//! `change_cnt` exists to detect.
//!
//! Everything outside this module reaches the buffer through [`typeahead`],
//! a `Copy` handle that *names* the cell. Each of its methods takes its own
//! short borrow and answers a value or a raw pointer, never a reference, so
//! that a call-out in between always sees — and can move — the current
//! storage. A snapshot would be actively wrong here: `inchar` can reallocate
//! the storage under its caller.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::types::{Failed, MB_MAXBYTES};
use core::ffi::{c_char, c_int};
use core::ptr;

/// Size of the two static initial buffers, upstream's `TYPELEN_INIT`.
///
/// The storage has three parts: room in front for the result of mappings, the
/// middle for typeahead, and room at the end for new characters.
const TYPELEN_INIT: c_int = 5 * (MAXMAPLEN as c_int + 3);

/// Where the valid bytes start in a freshly allocated buffer: enough room in
/// front that a mapping's RHS can be inserted without moving anything.
const HEAD_ROOM: c_int = MAXMAPLEN as c_int + 4;

/// The queue `vgetc` reads from.
///
/// Not `Copy`: `buf` and `noremap` are owned allocations (or, before the
/// first reallocation, the two static initial buffers), so a copy would be a
/// second owner of them. `GlobalCell::take` moves one out; `save_typeahead`
/// and `:source!` are the two callers that do.
pub(crate) struct TypeAhead {
    /// The bytes, NUL-terminated at `off + len`.
    buf: *mut u8,
    /// How much remapping each byte of `buf` is still allowed; no terminator.
    noremap: *mut u8,
    /// How long both allocations are.
    buflen: c_int,
    /// Where the valid bytes start.
    off: c_int,
    /// How many valid bytes there are.
    len: c_int,
    /// How many bytes at the *start* came from a mapping rather than the
    /// keyboard.
    maplen: c_int,
    /// How many bytes at the start were inserted silently.
    silent: c_int,
    /// How many bytes at the start may not trigger an abbreviation.
    no_abbr_cnt: c_int,
    /// Bumped by every change, so that a caller holding a pointer into the
    /// storage across a call that can reallocate it can tell.
    change_cnt: c_int,
}

impl TypeAhead {
    /// A typeahead with no storage at all: what [`init_typebuf`] looks for
    /// when it decides to hand out the static initial buffers, and what
    /// `GlobalCell::take` leaves behind.
    pub(crate) const EMPTY: Self = TypeAhead {
        buf: ptr::null_mut(),
        noremap: ptr::null_mut(),
        buflen: 0,
        off: 0,
        len: 0,
        maplen: 0,
        silent: 0,
        no_abbr_cnt: 0,
        change_cnt: 0,
    };

    /// The change counter this typeahead last published. Its one reader is
    /// [`alloc_typebuf`], which carries it on to the replacement.
    pub(crate) fn change_cnt(&self) -> c_int {
        self.change_cnt
    }

    /// Note a change, never landing on 0 — which [`typebuf_changed`] reads as
    /// "no snapshot taken".
    fn note_change(&mut self) {
        self.change_cnt = self.change_cnt.wrapping_add(1);
        if self.change_cnt == 0 {
            self.change_cnt = 1;
        }
    }
}

impl Default for TypeAhead {
    fn default() -> Self {
        TypeAhead::EMPTY
    }
}

/// The one typeahead buffer. Private: [`typeahead`] is the way in.
static TYPEBUF: GlobalCell<TypeAhead> = GlobalCell::new(TypeAhead::EMPTY);

/// The typeahead each `:source!` displaced, put back by `closescript`.
static SAVED_TYPEBUF: GlobalCell<[TypeAhead; NSCRIPT as usize]> =
    GlobalCell::new([const { TypeAhead::EMPTY }; NSCRIPT as usize]);

/// The initial storage, used until the first reallocation.
///
/// `xmalloc` is not usable for it: out of memory it would be impossible to
/// type anything, which is the one situation where typing has to keep
/// working. Freeing it would be a bug, which is why [`free_typebuf`] and the
/// reallocation path compare against these two addresses.
static TYPEBUF_INIT: GlobalCell<[u8; TYPELEN_INIT as usize]> =
    GlobalCell::new([0; TYPELEN_INIT as usize]);
/// The initial `noremap` array; see [`TYPEBUF_INIT`].
static NOREMAPBUF_INIT: GlobalCell<[u8; TYPELEN_INIT as usize]> =
    GlobalCell::new([0; TYPELEN_INIT as usize]);

/// The address of the static initial storage.
fn static_buf() -> *mut u8 {
    TYPEBUF_INIT.ptr().cast()
}

/// The address of the static initial `noremap` array.
fn static_noremap() -> *mut u8 {
    NOREMAPBUF_INIT.ptr().cast()
}

/// The typeahead buffer, *named* rather than pointed at.
///
/// See the module docs: every method takes its own short borrow, so nothing
/// here can hold a reference across a call-out.
#[derive(Clone, Copy)]
pub struct TypeAheadRef(&'static GlobalCell<TypeAhead>);

/// The typeahead buffer.
pub fn typeahead() -> TypeAheadRef {
    TypeAheadRef(&TYPEBUF)
}

impl TypeAheadRef {
    // -- the counters ------------------------------------------------------

    /// How many valid bytes the typeahead holds.
    pub fn len(self) -> c_int {
        self.0.with(|tb| tb.len)
    }

    /// Whether the typeahead holds nothing.
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Note `by` more valid bytes, which `vgetorpeek` just read into the room
    /// past the end.
    pub(crate) fn grow(self, by: c_int) {
        self.0.with_mut(|tb| tb.len += by);
    }

    /// How long the storage is.
    pub(crate) fn buflen(self) -> c_int {
        self.0.with(|tb| tb.buflen)
    }

    /// How many bytes at the start came from a mapping rather than the
    /// keyboard.
    pub fn maplen(self) -> c_int {
        self.0.with(|tb| tb.maplen)
    }

    /// How many bytes at the start were inserted silently.
    pub(crate) fn silent(self) -> c_int {
        self.0.with(|tb| tb.silent)
    }

    /// How many bytes at the start may not trigger an abbreviation.
    pub fn no_abbr_cnt(self) -> c_int {
        self.0.with(|tb| tb.no_abbr_cnt)
    }

    /// Block abbreviations for the first `n` bytes.
    pub fn set_no_abbr_cnt(self, n: c_int) {
        self.0.with_mut(|tb| tb.no_abbr_cnt = n);
    }

    /// Block abbreviations for `n` more bytes.
    pub fn add_no_abbr_cnt(self, n: c_int) {
        self.0.with_mut(|tb| tb.no_abbr_cnt += n);
    }

    /// The change counter, to be handed back to [`typebuf_changed`].
    pub fn change_cnt(self) -> c_int {
        self.0.with(|tb| tb.change_cnt)
    }

    /// Note that the storage changed under whoever holds a pointer into it.
    pub(crate) fn note_change(self) {
        self.0.with_mut(TypeAhead::note_change);
    }

    // -- the bytes ---------------------------------------------------------

    /// The whole storage, from byte zero rather than from `off`. Wanted only
    /// by the flush paths, which fill it from the front.
    pub(crate) fn storage(self) -> *mut u8 {
        self.0.with(|tb| tb.buf)
    }

    /// The valid byte at `offset`, as a pointer.
    pub(crate) fn at(self, offset: c_int) -> *mut u8 {
        // SAFETY: `offset` is within the typeahead, the caller's obligation
        // exactly as it was through the raw pointer this replaces.
        self.0
            .with(|tb| unsafe { tb.buf.offset((tb.off + offset) as isize) })
    }

    /// The valid byte at `offset`.
    pub(crate) fn byte(self, offset: c_int) -> c_int {
        // SAFETY: as `at`.
        c_int::from(unsafe { *self.at(offset) })
    }

    /// One past the last valid byte: where `inchar` reads new bytes into.
    pub(crate) fn tail(self) -> *mut u8 {
        // SAFETY: as `at`; `len` is in range by construction.
        self.0
            .with(|tb| unsafe { tb.buf.offset((tb.off + tb.len) as isize) })
    }

    /// How many bytes `inchar` may read into [`tail`](Self::tail), leaving
    /// room for the terminating NUL.
    pub(crate) fn room(self) -> c_int {
        self.0.with(|tb| tb.buflen - tb.off - tb.len - 1)
    }

    /// Say what the valid byte at `offset` is.
    pub(crate) fn set_byte(self, offset: c_int, value: u8) {
        // SAFETY: as `at`.
        self.0
            .with_mut(|tb| unsafe { *tb.buf.offset((tb.off + offset) as isize) = value });
    }

    /// How much remapping the valid byte at `offset` is still allowed.
    pub(crate) fn noremap(self, offset: c_int) -> c_int {
        // SAFETY: as `at`.
        self.0
            .with(|tb| c_int::from(unsafe { *tb.noremap.offset((tb.off + offset) as isize) }))
    }

    /// Say how much remapping the valid byte at `offset` is allowed.
    pub(crate) fn set_noremap(self, offset: c_int, flags: u8) {
        // SAFETY: as `at`.
        self.0
            .with_mut(|tb| unsafe { *tb.noremap.offset((tb.off + offset) as isize) = flags });
    }

    // -- the whole buffer --------------------------------------------------

    /// Move the typeahead out, leaving one with no storage at all.
    pub(crate) fn take(self) -> TypeAhead {
        self.0.take()
    }

    /// Install `tb`. Whatever the cell held is dropped on the floor, so it
    /// must have been freed or moved out first.
    pub(crate) fn set(self, tb: TypeAhead) {
        self.0.with_mut(|slot| *slot = tb);
    }
}

/// Point the typeahead at the static initial buffers, if it has none.
pub(crate) fn init_typebuf() {
    TYPEBUF.with_mut(|tb| {
        if !tb.buf.is_null() {
            return;
        }
        tb.buf = static_buf();
        tb.noremap = static_noremap();
        tb.buflen = TYPELEN_INIT;
        tb.len = 0;
        tb.off = HEAD_ROOM;
        tb.change_cnt = 1;
    });
}

/// Whether the keys being read now may not be remapped.
pub fn noremap_keys() -> bool {
    KeyNoremap.get() & (RM_NONE as c_int | RM_SCRIPT as c_int) != 0
}

impl TypeAhead {
    /// Insert `str`'s `addlen` bytes at `offset`, marking them with
    /// `noremap`. Answers false when the buffer would overflow an `int`,
    /// which is upstream's `e_toocompl`; the message is the caller's to emit,
    /// because emitting it flushes the buffers and this runs inside the
    /// cell's borrow.
    ///
    /// # Safety
    /// `str` must point at `addlen` readable bytes, and `offset` must be
    /// within the current typeahead.
    unsafe fn insert(
        &mut self,
        str: *mut c_char,
        addlen: c_int,
        offset: c_int,
        noremap: c_int,
        nottyped: bool,
        silent: bool,
    ) -> bool {
        let src = str.cast::<u8>();
        let head = offset as usize;
        let added = addlen as usize;
        if offset == 0 && addlen <= self.off {
            // Easy case: there is room in front of the valid bytes.
            self.off -= addlen;
            // SAFETY: `addlen <= off`, so the bytes fit in front of the valid
            // ones, and `str` is `addlen` readable bytes by the caller's
            // promise.
            unsafe {
                let dst = self.buf.offset(self.off as isize);
                ptr::copy_nonoverlapping(src, dst, added);
            }
        } else if self.len == 0 && self.buflen >= addlen + 3 * HEAD_ROOM {
            // Buffer is empty and the string fits: centre it, leaving
            // room before and after.
            self.off = (self.buflen - addlen - 3 * HEAD_ROOM) / 2;
            // SAFETY: `off` was just placed so that `addlen` bytes fit after
            // it, and `str` is that many readable bytes.
            unsafe {
                let dst = self.buf.offset(self.off as isize);
                ptr::copy_nonoverlapping(src, dst, added);
            }
        } else {
            // Reallocate. There must always be room for 3 * HEAD_ROOM
            // bytes, and some extra so this does not happen every time.
            let extra = addlen + HEAD_ROOM + 4 * HEAD_ROOM;
            if self.len > c_int::MAX - extra {
                // The string is getting too long for a 32-bit int.
                return false;
            }
            let newlen = self.len + extra;
            let size = newlen as usize;
            // SAFETY: `xmalloc` never returns null, and `newlen` covers
            // `HEAD_ROOM` in front, the old bytes with the new ones spliced in
            // at `offset`, and the room the next insert wants at the end.
            let buf = unsafe { xmalloc(size) }.cast::<u8>();
            // SAFETY: as above.
            let noremaps = unsafe { xmalloc(size) }.cast::<u8>();
            self.buflen = newlen;

            // Old bytes before the insertion point, then the new ones,
            // then the old bytes after it -- including the NUL at the end.
            let tail = self.len - offset + 1;
            debug_assert!(tail > 0);
            // SAFETY: `offset` is within the old typeahead by the caller's
            // promise, so the three runs together are `len + addlen + 1`
            // bytes, which `newlen` has room for after `HEAD_ROOM`; source and
            // destination are separate allocations, so none of them overlap.
            unsafe {
                let old = self.buf.offset(self.off as isize);
                let at = buf.offset(HEAD_ROOM as isize);
                ptr::copy_nonoverlapping(old, at, head);
                ptr::copy_nonoverlapping(src, at.add(head), added);
                let rest = at.add(head).add(added);
                ptr::copy_nonoverlapping(old.add(head), rest, tail as usize);
            }
            if self.buf != static_buf() {
                // SAFETY: the previous storage, which the test just showed is
                // not the static initial buffer.
                unsafe { xfree(self.buf.cast()) };
            }
            self.buf = buf;

            // The same for `noremap`, which has no terminator to carry.
            let kept = (self.len - offset) as usize;
            // SAFETY: as the copy above, one byte shorter for the missing
            // terminator; the `addlen` flags in the gap are written below.
            unsafe {
                let old = self.noremap.offset(self.off as isize);
                let at = noremaps.offset(HEAD_ROOM as isize);
                ptr::copy_nonoverlapping(old, at, head);
                let rest = at.add(head).add(added);
                ptr::copy_nonoverlapping(old.add(head), rest, kept);
            }
            if self.noremap != static_noremap() {
                // SAFETY: as the `xfree` above.
                unsafe { xfree(self.noremap.cast()) };
            }
            self.noremap = noremaps;

            self.off = HEAD_ROOM;
        }
        self.len += addlen;

        // What the characters that may not be remapped are marked with,
        // and how many of them there are.
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
            // SAFETY (this body): the two arrays were just sized for `off +
            // len + addlen`, so every flag written below is in range.
            unsafe { *self.noremap.offset((self.off + i + offset) as isize) = flags as u8 };
        }

        // `maplen` and `silent` only remember the length of the mapped
        // and/or silent run at the *start* of the buffer, on the
        // assumption that a mapped sequence does not produce typed
        // characters.
        if nottyped || self.maplen > offset {
            self.maplen += addlen;
        }
        if silent || self.silent > offset {
            self.silent += addlen;
            cmd_silent.set(true);
        }
        if self.no_abbr_cnt != 0 && offset == 0 {
            // ... and is not to be used for abbreviations.
            self.no_abbr_cnt += addlen;
        }
        true
    }

    /// Remove `len` bytes at `offset`.
    ///
    /// # Safety
    /// `offset + len` must be within the current typeahead.
    unsafe fn delete(&mut self, len: c_int, offset: c_int) {
        self.len -= len;

        if offset == 0 && self.buflen - (self.off + len) >= 3 * MAXMAPLEN as c_int + 3 {
            // Easy case: just leave the bytes in front and step over them.
            self.off += len;
        } else {
            // Otherwise both arrays have to be moved down.
            let from = self.off + offset;
            let head = offset as usize;
            if self.off > MAXMAPLEN as c_int {
                // Leave some extra room at the end to avoid a
                // reallocation.
                // SAFETY: `off > MAXMAPLEN`, so the destination is below the
                // source and both runs are inside the two arrays.
                unsafe {
                    let (buf, noremap) = (self.buf, self.noremap);
                    let keep = MAXMAPLEN as usize;
                    ptr::copy(buf.offset(self.off as isize), buf.add(keep), head);
                    ptr::copy(noremap.offset(self.off as isize), noremap.add(keep), head);
                }
                self.off = MAXMAPLEN as c_int;
            }
            // Include the NUL at the end for `buf`; `noremap` has none.
            let tail = self.len - offset + 1;
            debug_assert!(tail > 0);
            let kept = (self.len - offset) as usize;
            let (src, dst) = ((from + len) as isize, (self.off + offset) as isize);
            // SAFETY: the caller's promise — `offset + len` is within the
            // typeahead, so both runs are inside the two arrays; the copies
            // move bytes down, which `ptr::copy` allows to overlap.
            unsafe {
                ptr::copy(self.buf.offset(src), self.buf.offset(dst), tail as usize);
                let noremap = self.noremap;
                ptr::copy(noremap.offset(src), noremap.offset(dst), kept);
            }
        }

        // Each of the three run lengths shrinks only by the part of the
        // deletion that fell inside it.
        for run in [&mut self.maplen, &mut self.silent, &mut self.no_abbr_cnt] {
            if *run > offset {
                *run = if *run < offset + len {
                    offset
                } else {
                    *run - len
                };
            }
        }
    }

    /// Throw away the mapped characters at the start, or everything.
    ///
    /// The `FLUSH_MINIMAL` half of [`flush_buffers`]; see it for the rest.
    fn flush(&mut self, minimal: bool) {
        if minimal {
            // Remove the mapped characters at the start only, and only when
            // that leaves enough room in the buffer.
            if self.off + self.maplen >= self.buflen {
                self.off = MAXMAPLEN as c_int;
                self.len = 0;
            } else {
                self.off += self.maplen;
                self.len -= self.maplen;
            }
        } else {
            self.off = MAXMAPLEN as c_int;
            self.len = 0;
        }
        self.maplen = 0;
        self.silent = 0;
        self.no_abbr_cnt = 0;
        self.note_change();
    }
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
/// are read back. Answers `Err` when the buffer would overflow an `int`.
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
) -> Result<(), Failed> {
    init_typebuf();
    typeahead().note_change();
    // SAFETY (this body): the caller's promise -- `str` is NUL-terminated and
    // `offset` within the current typeahead.
    unsafe { state_no_longer_safe(c"ins_typebuf()".as_ptr()) };

    let addlen = unsafe { cstr::bytes_at(str) }.len() as c_int;
    let inserted =
        TYPEBUF.with_mut(|tb| unsafe { tb.insert(str, addlen, offset, noremap, nottyped, silent) });
    if !inserted {
        // Outside the borrow: `emsg` also flushes the buffers, i.e.
        // reaches the typeahead itself.
        emsg(gettext(e_toocompl));
        unsafe { setcursor() };
        return Err(Failed);
    }
    Ok(())
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
    // Room for the modifier prefix plus a K_SPECIAL-escaped character.
    let mut buf = [0 as c_char; MB_MAXBYTES * 3 + 4];
    // SAFETY (this body): `buf` is this frame's own array, sized for the
    // longest key escape, which `special_to_buf` fills.
    let len = unsafe { special_to_buf(c, modifiers, true, buf.as_mut_ptr()) } as usize;
    debug_assert!(len < buf.len());
    buf[len] = 0;
    let _ = unsafe {
        ins_typebuf(
            buf.as_mut_ptr(),
            KeyNoremap.get(),
            0,
            !KeyTyped.get(),
            cmd_silent.get(),
        )
    };
    if KeyTyped.get() && on_key_ignore {
        on_key_ignore_len.set(on_key_ignore_len.get() + len);
    }
    len as c_int
}

/// Whether the typeahead buffer changed while waiting for a character —
/// which happens when a message arrives from a client or from `feedkeys()`.
///
/// The test is deliberately generic: when the storage changed it was
/// reallocated and the old pointer is dead, and `off` may have moved so that
/// a write through the old one would land on bytes that were just added.
pub fn typebuf_changed(change_cnt: c_int) -> bool {
    change_cnt != 0 && (typeahead().change_cnt() != change_cnt || typebuf_was_filled.get())
}

/// Remove `len` characters at `offset` from the typeahead buffer.
///
/// # Safety
/// `offset + len` must be within the current typeahead.
pub unsafe fn del_typebuf(len: c_int, offset: c_int) {
    if len == 0 {
        return; // nothing to do
    }
    TYPEBUF.with_mut(|tb| {
        // SAFETY: the caller's obligation, forwarded.
        unsafe { tb.delete(len, offset) };
        tb.note_change();
    });
    // Text received from a client or from feedkeys() is no longer what is in
    // the buffer.
    typebuf_was_filled.set(false);
}

/// Throw away the mapped characters in the typeahead, or all of it.
///
/// Split out of [`flush_buffers`] so that the whole update is one borrow.
pub(crate) fn flush_typebuf(minimal: bool) {
    TYPEBUF.with_mut(|tb| tb.flush(minimal));
    if !minimal || typeahead().is_empty() {
        // Text received from a client or from feedkeys() is gone with it.
        typebuf_was_filled.set(false);
    }
    cmd_silent.set(false);
}

/// Undo the last [`gotchars`] for `len` bytes, so that putting a typed
/// character back into the typeahead does not record it twice.
///
/// Only the recording is affected. `len` is expected to be at most what the
/// last `gotchars` recorded.
pub fn ungetchars(len: c_int) {
    if reg_recording.get() == 0 {
        return;
    }
    recordbuff().delete_tail(len);
    // Wrapping, as the C's `size_t -=` is: `delete_tail` gives up when the
    // bytes are not all in the last block, and the counter then goes below
    // zero rather than tracking what was really removed.
    last_recorded_len.set(last_recorded_len.get().wrapping_sub(len as usize));
}

/// Sync undo, as reading typed characters out of the typeahead should.
///
/// Not in Insert or Cmdline mode unless a cursor key was used, and not while
/// reading a script file — in both cases the keys are one edit, not several.
///
/// Safe: callable at any time; it only reads the editor's own mode and
/// script state, and `u_sync` is itself safe.
pub fn may_sync_undo() {
    if (State.get() & (MODE_INSERT | MODE_CMDLINE) == 0 || arrow_used.get()) && curscript.get() < 0
    {
        u_sync(false);
    }
}

/// Empty the typeahead and give it freshly allocated buffers.
///
/// `was` is the change counter of the typeahead being replaced, which the
/// fresh one carries on from: both callers have just *moved* the old
/// typeahead out, so the counter is no longer in the cell to be read, and a
/// snapshot taken before the swap still has to compare unequal afterwards.
///
/// # Safety
/// The current buffers must already have been saved or freed.
pub(crate) unsafe fn alloc_typebuf(was: c_int) {
    TYPEBUF.with_mut(|tb| {
        // SAFETY: `xmalloc` either answers an allocation or aborts.
        (tb.buf, tb.noremap) = unsafe {
            (
                xmalloc(TYPELEN_INIT as usize).cast(),
                xmalloc(TYPELEN_INIT as usize).cast(),
            )
        };
        tb.buflen = TYPELEN_INIT;
        tb.off = HEAD_ROOM; // can insert without reallocating
        tb.len = 0;
        tb.maplen = 0;
        tb.silent = 0;
        tb.no_abbr_cnt = 0;
        tb.change_cnt = was;
        tb.note_change();
    });
    typebuf_was_filled.set(false);
}

/// Free the typeahead's buffers.
///
/// Freeing the two *static* initial buffers would be a bug, so that is
/// reported rather than done.
///
/// # Safety
/// Nothing may hold a pointer into either buffer.
pub(crate) unsafe fn free_typebuf() {
    // Which of the two was static, so that `internal_error` -- which reaches
    // the message machinery -- is called outside the borrow.
    let (buf_static, noremap_static) = TYPEBUF.with_mut(|tb| {
        let buf_static = tb.buf == static_buf();
        if !buf_static {
            // SAFETY: the caller's obligation, and the buffer was
            // `xmalloc`ed by `alloc_typebuf` or `insert`.
            unsafe { xfree(tb.buf.cast()) };
            tb.buf = ptr::null_mut();
        }
        let noremap_static = tb.noremap == static_noremap();
        if !noremap_static {
            // SAFETY: as above.
            unsafe { xfree(tb.noremap.cast()) };
            tb.noremap = ptr::null_mut();
        }
        (buf_static, noremap_static)
    });
    if buf_static {
        // SAFETY: a static string.
        unsafe { internal_error(c"Free typebuf 1".as_ptr()) };
    }
    if noremap_static {
        // SAFETY: a static string.
        unsafe { internal_error(c"Free typebuf 2".as_ptr()) };
    }
}

/// Put the current typeahead aside for the script `:source!` is about to
/// read, and start a fresh one.
///
/// # Safety
/// `curscript` must name an open script.
pub(crate) unsafe fn save_typebuf() {
    debug_assert!(curscript.get() >= 0);
    init_typebuf();
    let saved = typeahead().take();
    let was = saved.change_cnt();
    SAVED_TYPEBUF.with_mut(|slots| slots[curscript.get() as usize] = saved);
    // SAFETY: the typeahead was just moved into `SAVED_TYPEBUF`.
    unsafe { alloc_typebuf(was) };
}

/// Put back the typeahead [`save_typebuf`] displaced for script `script`.
///
/// # Safety
/// The typeahead the script was reading must already have been freed.
pub(crate) fn restore_saved_typebuf(script: c_int) {
    let saved = SAVED_TYPEBUF.with_mut(|slots| core::mem::take(&mut slots[script as usize]));
    typeahead().set(saved);
}

/// Whether the character `vungetc` put back can be handed out now.
///
/// It cannot when it was not stuffed and something has since been added to
/// the stuff buffer: those characters have to come first.
pub(crate) fn can_get_old_char() -> bool {
    old_char.get() != -1 && (old_KeyStuffed.get() != 0 || stuff_empty())
}

/// Save all three kinds of typeahead, so that a prompt really has to be
/// answered by the user.
///
/// # Safety
/// `tp` must point at writable storage that outlives the matching
/// [`restore_typeahead`].
pub unsafe fn save_typeahead(tp: *mut tasave_T) {
    // SAFETY (this body): the caller's promise -- `tp` is writable storage
    // that outlives the matching restore.
    unsafe { (*tp).save_typebuf = typeahead().take() };
    unsafe { alloc_typebuf((*tp).save_typebuf.change_cnt()) };
    unsafe { (*tp).typebuf_valid = true };
    unsafe { (*tp).old_char = old_char.get() };
    unsafe { (*tp).old_mod_mask = old_mod_mask.get() };
    old_char.set(-1);

    unsafe { (*tp).save_readbuf1 = readbuf1().take() };
    unsafe { (*tp).save_readbuf2 = readbuf2().take() };
}

/// Put back what [`save_typeahead`] saved, freeing what was read in the
/// meantime. Can only be called once per save.
///
/// # Safety
/// `tp` must be the one a matching [`save_typeahead`] filled.
pub unsafe fn restore_typeahead(tp: *mut tasave_T) {
    // SAFETY (this body): as [`save_typeahead`] -- `tp` is the one a matching
    // save filled.
    if unsafe { (*tp).typebuf_valid } {
        unsafe { free_typebuf() };
        typeahead().set(core::mem::take(unsafe { &mut (*tp).save_typebuf }));
    }
    old_char.set(unsafe { (*tp).old_char });
    old_mod_mask.set(unsafe { (*tp).old_mod_mask });

    unsafe { readbuf1().free() };
    readbuf1().set(core::mem::take(unsafe { &mut (*tp).save_readbuf1 }));
    unsafe { readbuf2().free() };
    readbuf2().set(core::mem::take(unsafe { &mut (*tp).save_readbuf2 }));
}
