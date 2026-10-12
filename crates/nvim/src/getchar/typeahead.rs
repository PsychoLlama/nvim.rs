//! The typeahead buffer: [`TypeAhead`], the queue `vgetc` reads from.
//!
//! It holds bytes waiting to be interpreted, with a parallel `noremap` array
//! saying how much remapping each byte is still allowed. [`ins_typebuf`]
//! pushes (that is what `feedkeys()` and every mapping expansion do) and
//! [`del_typebuf`] pops; the pair must keep `off`, `len`, `maplen`, `silent`
//! and `no_abbr_cnt` consistent, which is why both are methods on the type
//! rather than field pokes spread over four files.
//!
//! The two arrays are owned `Vec<u8>`s of one length, the storage's
//! `buflen`, and the valid bytes are a window into them: there is room in
//! *front* of the window (`off`) so that a mapping's RHS can be pushed
//! without moving what follows, and room behind it for what `inchar` reads.
//! The geometry is upstream's, number for number -- where the window sits,
//! when the storage is reallocated and how long it then is -- because how
//! much `inchar` may read at once is the room left behind the window.
//!
//! Everything outside this module reaches the buffer through [`typeahead`],
//! a `Copy` handle that *names* the cell. Each of its methods takes its own
//! short borrow and answers a value or a copy, never a reference that could
//! outlive a call-out: user code run by a mapping, an autocommand or a timer
//! can reshape the typeahead under any caller, and `change_cnt` is how a
//! caller that waited tells.

#![forbid(unsafe_code)]

use super::*;
use crate::keycodes::ModMask;
use crate::types::Failed;
use core::ffi::c_int;

/// Size of the initial storage, upstream's `TYPELEN_INIT`.
///
/// The storage has three parts: room in front for the result of mappings, the
/// middle for typeahead, and room at the end for new characters.
const TYPELEN_INIT: usize = 5 * (MAXMAPLEN as usize + 3);

/// Where the valid bytes start in a freshly allocated buffer: enough room in
/// front that a mapping's RHS can be inserted without moving anything.
const HEAD_ROOM: c_int = MAXMAPLEN as c_int + 4;

/// The queue `vgetc` reads from.
///
/// Not `Copy`: `buf` and `noremap` are owned allocations. `GlobalCell::take`
/// moves one out; `save_typeahead` and `:source!` are the two callers that
/// do.
pub(crate) struct TypeAhead {
    /// The bytes; the valid ones are `off..off + len`.
    buf: Vec<u8>,
    /// How much remapping each byte of `buf` is still allowed.
    noremap: Vec<u8>,
    /// How long both are, as upstream's `int`.
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
    /// Bumped by every change, so that a caller that waited (or ran user
    /// code) can tell the typeahead moved under it.
    change_cnt: c_int,
}

impl TypeAhead {
    /// A typeahead with no storage at all: what [`init_typebuf`] looks for
    /// when it decides to give it some, and what `GlobalCell::take` leaves
    /// behind.
    pub(crate) const EMPTY: Self = TypeAhead {
        buf: Vec::new(),
        noremap: Vec::new(),
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

    /// The index of valid byte `offset` in the storage.
    fn index(&self, offset: c_int) -> usize {
        usize::try_from(self.off + offset).expect("an offset inside the typeahead")
    }

    /// Fresh storage of the initial length, empty, with room in front.
    fn fresh_storage(&mut self) {
        self.buf = vec![0; TYPELEN_INIT];
        self.noremap = vec![0; TYPELEN_INIT];
        self.buflen = TYPELEN_INIT as c_int;
        self.off = HEAD_ROOM;
        self.len = 0;
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

/// The most bytes [`TypeAheadRef::copy_out`] answers without allocating:
/// more than any key, and than any mapping's LHS but a pathological one.
const COPY_INLINE: usize = 64;

/// Bytes copied out of the typeahead: on the stack when they fit, which is
/// every key and nearly every mapping.
pub(crate) enum KeyBytes {
    Inline([u8; COPY_INLINE], usize),
    Heap(Vec<u8>),
}

impl KeyBytes {
    pub(crate) fn as_slice(&self) -> &[u8] {
        match self {
            KeyBytes::Inline(bytes, len) => &bytes[..*len],
            KeyBytes::Heap(bytes) => bytes,
        }
    }
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

    /// How long the storage is.
    pub(crate) fn buflen(self) -> usize {
        self.0.with(|tb| tb.buf.len())
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

    /// Note that the typeahead changed under whoever is waiting on it.
    pub(crate) fn note_change(self) {
        self.0.with_mut(TypeAhead::note_change);
    }

    // -- the bytes ---------------------------------------------------------

    /// The valid byte at `offset`.
    pub(crate) fn byte(self, offset: c_int) -> c_int {
        self.0.with(|tb| c_int::from(tb.buf[tb.index(offset)]))
    }

    /// Say what the valid byte at `offset` is.
    pub(crate) fn set_byte(self, offset: c_int, value: u8) {
        self.0.with_mut(|tb| {
            let at = tb.index(offset);
            tb.buf[at] = value;
        });
    }

    /// Overwrite the valid bytes from `offset` on with `bytes`.
    pub(crate) fn set_bytes(self, offset: c_int, bytes: &[u8]) {
        self.0.with_mut(|tb| {
            let at = tb.index(offset);
            tb.buf[at..at + bytes.len()].copy_from_slice(bytes);
        });
    }

    /// A copy of `len` valid bytes from `offset` on, for a caller about to
    /// run code that may change the typeahead.
    pub(crate) fn copy_out(self, offset: c_int, len: usize) -> KeyBytes {
        self.0.with(|tb| {
            let at = tb.index(offset);
            let bytes = &tb.buf[at..at + len];
            if len <= COPY_INLINE {
                let mut inline = [0; COPY_INLINE];
                inline[..len].copy_from_slice(bytes);
                KeyBytes::Inline(inline, len)
            } else {
                KeyBytes::Heap(bytes.to_vec())
            }
        })
    }

    /// Lend the valid bytes and their `noremap` flags, front first.
    ///
    /// The closure must not reach the typeahead; it is for the walks that
    /// read it whole without calling out, the mapping match among them.
    pub(crate) fn with_valid<R>(self, f: impl FnOnce(&[u8], &[u8]) -> R) -> R {
        self.0.with(|tb| {
            let (from, to) = (tb.index(0), tb.index(tb.len));
            f(&tb.buf[from..to], &tb.noremap[from..to])
        })
    }

    /// How many bytes `inchar` may read past the end, leaving room for the
    /// terminating NUL upstream writes there.
    pub(crate) fn room(self) -> c_int {
        self.0.with(|tb| tb.buflen - tb.off - tb.len - 1)
    }

    /// Append `bytes`, just read from the user, as remappable. The caller
    /// has made sure they fit in [`room`](Self::room) -- `inchar` never
    /// answers more -- and the storage only grows when they do not.
    pub(crate) fn append_typed(self, bytes: &[u8]) {
        self.0.with_mut(|tb| {
            let at = tb.index(tb.len);
            if at + bytes.len() + 1 > tb.buf.len() {
                // Only reachable when a caller read more than the room it
                // was given; keep every byte rather than lose one.
                tb.buf.resize(at + bytes.len() + 1, 0);
                tb.noremap.resize(at + bytes.len() + 1, 0);
                tb.buflen = c_int::try_from(tb.buf.len()).expect("a typeahead shorter than an int");
            }
            tb.buf[at..at + bytes.len()].copy_from_slice(bytes);
            tb.noremap[at..at + bytes.len()].fill(RM_YES as u8);
            tb.len += c_int::try_from(bytes.len()).expect("a read fits an int");
        });
    }

    /// How much remapping the valid byte at `offset` is still allowed.
    pub(crate) fn noremap(self, offset: c_int) -> c_int {
        self.0.with(|tb| c_int::from(tb.noremap[tb.index(offset)]))
    }

    // -- the whole buffer --------------------------------------------------

    /// Move the typeahead out, leaving one with no storage at all.
    pub(crate) fn take(self) -> TypeAhead {
        self.0.take()
    }

    /// Install `tb`, dropping what the cell held.
    pub(crate) fn set(self, tb: TypeAhead) {
        self.0.with_mut(|slot| *slot = tb);
    }
}

/// Give the typeahead its initial storage, if it has none.
pub(crate) fn init_typebuf() {
    TYPEBUF.with_mut(|tb| {
        if !tb.buf.is_empty() {
            return;
        }
        tb.fresh_storage();
        tb.change_cnt = 1;
    });
}

/// Whether the keys being read now may not be remapped.
pub fn noremap_keys() -> bool {
    KeyNoremap.get() & (RM_NONE as c_int | RM_SCRIPT as c_int) != 0
}

impl TypeAhead {
    /// Insert `s` at `offset`, marking its bytes with `noremap`. Answers
    /// false when the buffer would overflow an `int`, which is upstream's
    /// `e_toocompl`; the message is the caller's to emit, because emitting it
    /// flushes the buffers and this runs inside the cell's borrow.
    ///
    /// `offset` must be within the current typeahead.
    fn insert(
        &mut self,
        s: &[u8],
        offset: c_int,
        noremap: c_int,
        nottyped: bool,
        silent: bool,
    ) -> bool {
        let Ok(addlen) = c_int::try_from(s.len()) else {
            return false;
        };
        let head = usize::try_from(offset).expect("an offset inside the typeahead");
        if offset == 0 && addlen <= self.off {
            // Easy case: there is room in front of the valid bytes.
            self.off -= addlen;
            let at = self.index(0);
            self.buf[at..at + s.len()].copy_from_slice(s);
        } else if self.len == 0 && self.buflen >= addlen.saturating_add(3 * HEAD_ROOM) {
            // Buffer is empty and the string fits: centre it, leaving
            // room before and after.
            self.off = (self.buflen - addlen - 3 * HEAD_ROOM) / 2;
            let at = self.index(0);
            self.buf[at..at + s.len()].copy_from_slice(s);
        } else {
            // Reallocate. There must always be room for 3 * HEAD_ROOM
            // bytes, and some extra so this does not happen every time.
            let Some(newlen) = addlen
                .checked_add(5 * HEAD_ROOM)
                .and_then(|extra| self.len.checked_add(extra))
            else {
                // The string is getting too long for a 32-bit int.
                return false;
            };
            let size = usize::try_from(newlen).expect("a positive length");
            let (from, to) = (self.index(0), self.index(self.len));
            let room = usize::try_from(HEAD_ROOM).expect("a positive offset");

            // Old bytes before the insertion point, then the new ones,
            // then the old bytes after it. The new `noremap` flags are
            // written below.
            let mut buf = vec![0; size];
            buf[room..room + head].copy_from_slice(&self.buf[from..from + head]);
            buf[room + head..room + head + s.len()].copy_from_slice(s);
            let rest = room + head + s.len();
            buf[rest..rest + (to - from - head)].copy_from_slice(&self.buf[from + head..to]);
            let mut noremaps = vec![0; size];
            noremaps[room..room + head].copy_from_slice(&self.noremap[from..from + head]);
            noremaps[rest..rest + (to - from - head)]
                .copy_from_slice(&self.noremap[from + head..to]);
            self.buf = buf;
            self.noremap = noremaps;
            self.buflen = newlen;
            self.off = HEAD_ROOM;
        }
        self.len += addlen;

        // What the characters that may not be remapped are marked with,
        // and how many of them there are.
        let val = if noremap == REMAP_SCRIPT {
            RM_SCRIPT
        } else if noremap == REMAP_SKIP {
            RM_ABBR
        } else {
            RM_NONE
        } as u8;
        let noremapped = if noremap == REMAP_SKIP {
            1
        } else if noremap < 0 {
            s.len()
        } else {
            usize::try_from(noremap).expect("a positive count")
        };
        let at = self.index(offset);
        for (i, flags) in self.noremap[at..at + s.len()].iter_mut().enumerate() {
            *flags = if i < noremapped { val } else { RM_YES as u8 };
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

    /// Remove `len` bytes at `offset`, which must be within the current
    /// typeahead.
    fn delete(&mut self, len: c_int, offset: c_int) {
        self.len -= len;

        if offset == 0 && self.buflen - (self.off + len) >= 3 * MAXMAPLEN as c_int + 3 {
            // Easy case: just leave the bytes in front and step over them.
            self.off += len;
        } else {
            self.move_down(len, offset);
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

    /// The hard case of [`delete`](Self::delete): both arrays moved down
    /// over the deleted run. Out of line, so that the easy case -- every
    /// key read off the front -- stays a few instructions.
    #[cold]
    #[inline(never)]
    fn move_down(&mut self, len: c_int, offset: c_int) {
        let from = self.index(offset);
        let head = usize::try_from(offset).expect("an offset inside the typeahead");
        if self.off > MAXMAPLEN as c_int {
            // Leave some extra room at the end to avoid a reallocation.
            let (old, keep) = (self.index(0), MAXMAPLEN as usize);
            self.buf.copy_within(old..old + head, keep);
            self.noremap.copy_within(old..old + head, keep);
            self.off = MAXMAPLEN as c_int;
        }
        // `from` is where the deleted run started *before* the move above,
        // which is upstream's arithmetic too.
        let gone = usize::try_from(len).expect("a positive length");
        let kept = usize::try_from(self.len - offset).expect("the run after the deletion");
        let dst = self.index(offset);
        self.buf.copy_within(from + gone..from + gone + kept, dst);
        self.noremap
            .copy_within(from + gone..from + gone + kept, dst);
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

/// Insert `text` into the typeahead buffer at `offset`.
///
/// `noremap` says how much of it may be mapped again: `REMAP_YES` all of it,
/// `REMAP_NONE` none, `REMAP_SCRIPT` only script-local mappings,
/// `REMAP_SKIP` only the first character (but abbreviations still apply), and
/// a positive count that many characters.
///
/// With `nottyped` the string does not set `KeyTyped` — do not use it with a
/// non-zero `offset`. With `silent`, `cmd_silent` is set when the characters
/// are read back. `text` is read up to its first NUL, as upstream's `char *`
/// is, and `offset` must be within the current typeahead. Answers `Err` when
/// the buffer would overflow an `int`.
pub fn ins_typebuf(
    text: &[u8],
    noremap: c_int,
    offset: c_int,
    nottyped: bool,
    silent: bool,
) -> Result<(), Failed> {
    init_typebuf();
    typeahead().note_change();
    state_no_longer_safe(Some(c"ins_typebuf()"));

    let text = up_to_nul(text);
    let inserted = TYPEBUF.with_mut(|tb| tb.insert(text, offset, noremap, nottyped, silent));
    if !inserted {
        // Outside the borrow: `emsg` also flushes the buffers, i.e.
        // reaches the typeahead itself.
        emsg(gettext(e_toocompl));
        setcursor();
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
pub fn ins_char_typebuf(c: c_int, modifiers: ModMask, on_key_ignore: bool) -> c_int {
    // The modifier prefix plus a K_SPECIAL-escaped character.
    let key = special_key_bytes(c, modifiers, true);
    let len = key.as_bytes().len();
    let _ = ins_typebuf(
        key.as_bytes(),
        KeyNoremap.get(),
        0,
        !KeyTyped.get(),
        cmd_silent.get(),
    );
    if KeyTyped.get() && on_key_ignore {
        on_key_ignore_len.set(on_key_ignore_len.get() + len);
    }
    c_int::try_from(len).expect("a key is a few bytes")
}

/// Whether the typeahead buffer changed while waiting for a character —
/// which happens when a message arrives from a client or from `feedkeys()`.
///
/// The test is deliberately generic: when the storage changed it may have
/// been reallocated, and `off` may have moved so that what was read for the
/// old window would land on bytes that were just added.
pub fn typebuf_changed(change_cnt: c_int) -> bool {
    change_cnt != 0 && (typeahead().change_cnt() != change_cnt || typebuf_was_filled.get())
}

/// Remove `len` characters at `offset` from the typeahead buffer. `offset +
/// len` must be within the current typeahead.
pub fn del_typebuf(len: c_int, offset: c_int) {
    if len == 0 {
        return; // nothing to do
    }
    TYPEBUF.with_mut(|tb| {
        tb.delete(len, offset);
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
    last_recorded_len.set(
        last_recorded_len
            .get()
            .wrapping_sub(len.unsigned_abs() as usize),
    );
}

/// Sync undo, as reading typed characters out of the typeahead should.
///
/// Not in Insert or Cmdline mode unless a cursor key was used, and not while
/// reading a script file — in both cases the keys are one edit, not several.
pub(crate) fn may_sync_undo() {
    if (State.get() & (MODE_INSERT | MODE_CMDLINE) == 0 || arrow_used.get()) && curscript.get() < 0
    {
        u_sync(false);
    }
}

/// Empty the typeahead and give it fresh storage.
///
/// `was` is the change counter of the typeahead being replaced, which the
/// fresh one carries on from: both callers have just *moved* the old
/// typeahead out, so the counter is no longer in the cell to be read, and a
/// snapshot taken before the swap still has to compare unequal afterwards.
pub(crate) fn alloc_typebuf(was: c_int) {
    TYPEBUF.with_mut(|tb| {
        tb.fresh_storage(); // can insert without reallocating
        tb.maplen = 0;
        tb.silent = 0;
        tb.no_abbr_cnt = 0;
        tb.change_cnt = was;
        tb.note_change();
    });
    typebuf_was_filled.set(false);
}

/// Put the current typeahead aside for the script `:source!` is about to
/// read, and start a fresh one.
pub(crate) fn save_typebuf() {
    debug_assert!(curscript.get() >= 0);
    init_typebuf();
    let saved = typeahead().take();
    let was = saved.change_cnt();
    SAVED_TYPEBUF.with_mut(|slots| slots[script_slot(curscript.get())] = saved);
    alloc_typebuf(was);
}

/// Put back the typeahead [`save_typebuf`] displaced for script `script`,
/// dropping the one read since.
pub(crate) fn restore_saved_typebuf(script: c_int) {
    let saved = SAVED_TYPEBUF.with_mut(|slots| core::mem::take(&mut slots[script_slot(script)]));
    typeahead().set(saved);
}

/// The stack slot of script `script`, which must be open.
fn script_slot(script: c_int) -> usize {
    usize::try_from(script).expect("an open script")
}

/// Whether the character `vungetc` put back can be handed out now.
///
/// It cannot when it was not stuffed and something has since been added to
/// the stuff buffer: those characters have to come first.
pub(crate) fn ungot_key_ready() -> bool {
    ungot
        .with(|key| key.as_ref().map(|key| key.stuffed))
        .is_some_and(|stuffed| stuffed || stuff_empty())
}

/// Save all three kinds of typeahead into `save`, so that a prompt really
/// has to be answered by the user.
pub fn save_typeahead(save: &mut TypeaheadSave) {
    save.save_typebuf = typeahead().take();
    // A fresh buffer for the typeahead just moved out.
    alloc_typebuf(save.save_typebuf.change_cnt());
    save.typebuf_valid = true;
    save.ungot = ungot.take();

    save.save_readbuf1 = readbuf1().take();
    save.save_readbuf2 = readbuf2().take();
}

/// Put back what [`save_typeahead`] saved in `save`, dropping what was read
/// in the meantime. Can only be called once per save.
pub fn restore_typeahead(save: &mut TypeaheadSave) {
    if save.typebuf_valid {
        typeahead().set(core::mem::take(&mut save.save_typebuf));
    }
    ungot.set(save.ungot.take());

    readbuf1().set(core::mem::take(&mut save.save_readbuf1));
    readbuf2().set(core::mem::take(&mut save.save_readbuf2));
}

#[cfg(test)]
mod tests {
    //! The typeahead buffer and the one key `vungetc` puts back, through
    //! the entry points the editor uses.

    use super::*;
    use crate::global_cell::{editor_state::Held, editor_state_lock};
    use crate::keycodes::{K_SPECIAL, KS_MODIFIER, Key, key_escape};
    use crate::types::NUL;

    /// The typeahead is process-wide; hold the lock and start from an
    /// empty one. Dropping the answer empties it again.
    struct Fresh(#[allow(dead_code, reason = "held for its drop")] Held);

    impl Fresh {
        fn new() -> Self {
            let held = editor_state_lock();
            clear();
            Fresh(held)
        }
    }

    impl Drop for Fresh {
        fn drop(&mut self) {
            clear();
        }
    }

    fn clear() {
        init_typebuf();
        flush_typebuf(false);
        ungot.set(None);
        while read_readbuffers(true) != NUL {}
    }

    /// The valid bytes, front first.
    fn bytes() -> Vec<u8> {
        (0..typeahead().len())
            .map(|i| u8::try_from(typeahead().byte(i)).expect("the typeahead holds bytes"))
            .collect()
    }

    #[test]
    fn a_put_back_key_goes_in_front() {
        let _fresh = Fresh::new();
        assert_eq!(ins_char_typebuf(c_int::from(b'a'), ModMask::NONE, false), 1);
        assert_eq!(ins_char_typebuf(c_int::from(b'b'), ModMask::NONE, false), 1);
        assert_eq!(bytes(), b"ba");
        // Remappable: `KeyNoremap` was clear.
        assert_eq!(typeahead().noremap(0), 0);
        assert_eq!(typeahead().noremap(1), 0);
    }

    #[test]
    fn a_special_key_is_escaped_with_its_modifiers() {
        let _fresh = Fresh::new();
        let left = Key::Left.code();
        assert_eq!(ins_char_typebuf(left, ModMask::NONE, false), 3);
        assert_eq!(bytes(), key_escape(left));
        flush_typebuf(false);
        assert_eq!(ins_char_typebuf(left, ModMask::SHIFT, false), 6);
        let byte = |c: c_int| u8::try_from(c).expect("a byte");
        let mut expected = vec![
            byte(K_SPECIAL),
            byte(KS_MODIFIER),
            byte(ModMask::SHIFT.bits()),
        ];
        expected.extend_from_slice(&key_escape(left));
        assert_eq!(bytes(), expected);
    }

    #[test]
    fn growing_past_the_initial_storage_keeps_every_byte() {
        let _fresh = Fresh::new();
        let before = typeahead().change_cnt();
        assert!(!typebuf_changed(before));
        let mut expected = Vec::new();
        for i in 0..1000u16 {
            let c = b'a' + u8::try_from(i % 26).expect("a letter");
            ins_char_typebuf(c_int::from(c), ModMask::NONE, false);
            expected.insert(0, c);
        }
        assert_eq!(bytes(), expected);
        assert!(typeahead().buflen() > TYPELEN_INIT);
        assert!(typebuf_changed(before));
        flush_typebuf(false);
        assert!(typeahead().is_empty());
    }

    /// The valid bytes with their `noremap` flags, front first.
    fn flagged() -> Vec<(u8, u8)> {
        typeahead()
            .with_valid(|bytes, flags| bytes.iter().copied().zip(flags.iter().copied()).collect())
    }

    /// What `noremap` marks the first `count` inserted bytes with, and how
    /// many it marks.
    fn marks(noremap: c_int, len: usize) -> (u8, usize) {
        match noremap {
            REMAP_SCRIPT => (RM_SCRIPT as u8, len),
            REMAP_SKIP => (RM_ABBR as u8, 1),
            REMAP_NONE => (RM_NONE as u8, len),
            n => (RM_NONE as u8, usize::try_from(n).expect("a count")),
        }
    }

    #[test]
    fn insert_and_delete_at_every_offset_keep_bytes_and_flags() {
        let _fresh = Fresh::new();
        let base = b"abcdefgh";
        // Short and long inserts: the long one cannot fit in front or in an
        // empty buffer, so it reallocates.
        let long: Vec<u8> = (0..300u16)
            .map(|i| b'A' + u8::try_from(i % 26).expect("small"))
            .collect();
        for insert in [&b"XYZ"[..], &long] {
            for noremap in [REMAP_YES, REMAP_NONE, REMAP_SCRIPT, REMAP_SKIP, 2] {
                for offset in 0..=base.len() {
                    flush_typebuf(false);
                    ins_typebuf(base, REMAP_YES, 0, false, false).unwrap();
                    let at = c_int::try_from(offset).expect("small");
                    ins_typebuf(insert, noremap, at, false, false).unwrap();
                    let mut model: Vec<(u8, u8)> =
                        base.iter().map(|&b| (b, RM_YES as u8)).collect();
                    let (flag, count) = if noremap == REMAP_YES {
                        (0, 0)
                    } else {
                        marks(noremap, insert.len())
                    };
                    for (i, &b) in insert.iter().enumerate() {
                        model.insert(offset + i, (b, if i < count { flag } else { RM_YES as u8 }));
                    }
                    assert_eq!(flagged(), model, "insert at {offset}, noremap {noremap}");

                    // Delete two bytes at every offset in turn, front to back.
                    let mut from = 0;
                    while from + 2 <= model.len() {
                        del_typebuf(2, c_int::try_from(from).expect("small"));
                        model.drain(from..from + 2);
                        assert_eq!(flagged(), model, "delete at {from}");
                        from += 3;
                    }
                }
            }
        }
    }

    #[test]
    fn the_mapped_run_tracks_inserts_and_deletes() {
        let _fresh = Fresh::new();
        // Typed keys, then a mapping's RHS in front of them.
        ins_typebuf(b"typed", REMAP_YES, 0, false, false).unwrap();
        ins_typebuf(b"rhs", REMAP_YES, 0, true, true).unwrap();
        assert_eq!((typeahead().maplen(), typeahead().silent()), (3, 3));
        // Inside the run: it grows.
        ins_typebuf(b"in", REMAP_YES, 1, false, false).unwrap();
        assert_eq!(typeahead().maplen(), 5);
        // Past it: it does not.
        ins_typebuf(b"out", REMAP_YES, 7, false, false).unwrap();
        assert_eq!(typeahead().maplen(), 5);
        // A delete straddling its end cuts it at the delete's start.
        del_typebuf(3, 4);
        assert_eq!(typeahead().maplen(), 4);
        del_typebuf(2, 0);
        assert_eq!(typeahead().maplen(), 2);
        flush_typebuf(true);
        assert_eq!(typeahead().maplen(), 0);
        assert_eq!(bytes(), b"outped");
    }

    #[test]
    fn an_ungotten_key_is_peeked_before_the_typeahead() {
        let _fresh = Fresh::new();
        ins_char_typebuf(c_int::from(b't'), ModMask::NONE, false);
        vungetc(c_int::from(b'u'));
        assert!(ungot_key_ready());
        assert_eq!(vpeekc(), c_int::from(b'u'));
        // A typed key waits for the stuff buffer to empty; a stuffed one
        // does not.
        stuff_readbuf(b"s");
        assert!(!ungot_key_ready());
        KeyStuffed.set(1);
        vungetc(c_int::from(b'u'));
        KeyStuffed.set(0);
        assert!(ungot_key_ready());
    }
}
