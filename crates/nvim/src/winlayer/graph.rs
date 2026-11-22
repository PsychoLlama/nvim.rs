//! The window, buffer and tabpage graph: which of each exists, in what
//! order, and which one is current.
//!
//! The transpiler parked these beside `main()` because upstream declares them
//! in `globals.h`, but every one of them is a node of the tree [`winlayer`]
//! wraps: the doubly-linked window list (`FIRSTWIN`/`lastwin`), the buffer
//! list (`FIRSTBUF`/`LASTBUF`), the tabpage list (`FIRST_TABPAGE`), the frame
//! tree's root (`topframe`), and the `cur*` cursors into all three. The
//! command-line window's saved graph (`cmdwin_*`) belongs with them: it is a
//! second, temporary editor tree that the first one is swapped out for.
//!
//! Which one is current is a [`WinId`]/[`BufId`]/[`TabId`], like the list
//! heads: [`CURRENT_WIN`] and its two siblings are the truth, and the raw
//! `curwin`/`CURBUF`/`CURTAB` beside them are mirrors the same setters
//! `prevwin`, `LASTUSED_TABPAGE` and the `cmdwin_*` block are ids too, and
//! for the same reason the list heads are: each one is read *after* a call
//! that may have freed what it names, and a stale id answers `None` where a
//! stale address answers with whatever the allocator has put there since.
//! `topframe` is the last raw pointer here, and is waiting on a frame
//! registry.
//!
//! [`winlayer`]: super
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `winlayer/` row in docs/perimeter.md.
// The exports here are metrics/abi-ledger.jsonl rows (`curwin`, `win_col_off`), and
// `#[unsafe(no_mangle)]` is itself an unsafe attribute.
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use super::{Buf, BufId, FrameId, Stamp, TabId, TabPage, Win, WinId};
use crate::global_cell::GlobalCell;
use crate::marktree::MarkTree;
use crate::memline::state::MemLine;
use crate::memory::XString;
use crate::optionstr::BUF_STRING_OPTIONS;
use crate::registry::id_map;
use crate::slot::{Parked, Pooled, SlotBox};
use crate::types::{BufName, Buffer, Handle, Key, Tabpage, Window};
use core::ffi::c_int;
use core::mem::MaybeUninit;

pub(crate) static FIRSTWIN: GlobalCell<Option<WinId>> = GlobalCell::new(None);
pub(crate) static LASTWIN: GlobalCell<Option<WinId>> = GlobalCell::new(None);
pub(crate) static PREVWIN: GlobalCell<Option<WinId>> = GlobalCell::new(None);
/// The current window's address, mirroring [`CURRENT_WIN`].
///
/// `pub` and unmangled because it is read as a **data symbol** from outside
/// the crate: plugins do it, and so does
/// `test/functional/lua/ffi_spec.lua` (`extern win_T *curwin`, handed to
/// `win_col_off`). Inside the crate it answers [`Win::is_current`] without a
/// registry lookup. Nothing but the funnel below writes it.
#[unsafe(no_mangle)]
pub static curwin: GlobalCell<*mut Window> = GlobalCell::new(::core::ptr::null_mut::<Window>());
/// The root of the current tab page's layout tree, mirroring
/// `tp_topframe`. An identity, as the tree's own links are.
pub(crate) static TOPFRAME: GlobalCell<Option<FrameId>> = GlobalCell::new(None);
pub(crate) static FIRST_TABPAGE: GlobalCell<Option<TabId>> = GlobalCell::new(None);
/// The current tab page's address, mirroring [`CURRENT_TAB`]. [`curwin`],
/// minus the symbol: only [`TabPage::is_current`] reads it.
pub(super) static CURTAB: GlobalCell<*mut Tabpage> =
    GlobalCell::new(::core::ptr::null_mut::<Tabpage>());
pub(crate) static LASTUSED_TABPAGE: GlobalCell<Option<TabId>> = GlobalCell::new(None);
pub(crate) static FIRSTBUF: GlobalCell<Option<BufId>> = GlobalCell::new(None);
pub(crate) static LASTBUF: GlobalCell<Option<BufId>> = GlobalCell::new(None);
/// The current buffer's address, mirroring [`CURRENT_BUF`]. [`CURTAB`]'s
/// shape: only `Buf::is_current` and [`switch_buffer`] read it.
pub(super) static CURBUF: GlobalCell<*mut Buffer> =
    GlobalCell::new(::core::ptr::null_mut::<Buffer>());
/// The command-line window's type: the first character of the line it
/// edits (`:`, `/`, `?`, ...), 0 when there is no such window.
pub(crate) static CMDWIN_TYPE: GlobalCell<u8> = GlobalCell::new(0);
pub(crate) static CMDWIN_RESULT: GlobalCell<Key> = GlobalCell::new(Key::NUL);
pub(crate) static CMDWIN_LEVEL: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static CMDWIN_BUF: GlobalCell<Option<BufId>> = GlobalCell::new(None);
pub(crate) static CMDWIN_WIN: GlobalCell<Option<WinId>> = GlobalCell::new(None);
pub(crate) static CMDWIN_OLD_CURWIN: GlobalCell<Option<WinId>> = GlobalCell::new(None);
pub(crate) static CMDLINE_WIN: GlobalCell<Option<WinId>> = GlobalCell::new(None);

/// Columns of window `window` that are not text -- the 'number' /
/// 'statuscolumn' column, the command-line window's marker, the fold column
/// and the sign column.
///
/// Here rather than in `move`, where its body is, for the same reason
/// [`curwin`] is here: `test/functional/lua/ffi_spec.lua` reads `curwin` as a
/// data symbol and hands it straight to this one through an `ffi.cdef` that
/// spells `win_T *`, so the pair is frozen together and the raw parameter is
/// the ABI, not a gap in the migration. Rust callers want
/// `crate::r#move::win_col_off`, which takes a [`Win`].
///
/// # Safety
/// `window` must be a live window.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn win_col_off(window: *mut Window) -> c_int {
    // SAFETY: the caller's promise.
    unsafe { Win::new(window) }.col_off()
}

// ---------------------------------------------------------------------------
// Which one is current
//
// The truth is the **identity**: an `Option<WinId>`/`BufId`/`TabId` the
// registry answers for, so that "the current window" is a question with a
// checked answer rather than an address nobody promised. The three raw
// statics above are *mirrors* the same setters write, kept for two reasons
// and no others: `curwin` is a data symbol plugins and
// `test/functional/lua/ffi_spec.lua` read (`extern win_T *curwin`), and all
// three answer `is_current()` without touching a registry.
//
// They are written together, always, by the funnel below. Nothing else in
// the tree writes either half.

/// The window the editor is working in, as the registry names it.
///
/// `None` only before `win_alloc_first` and after the last window goes.
pub(super) static CURRENT_WIN: GlobalCell<Stamp> = GlobalCell::new(Stamp::NONE);

/// The buffer the editor is working in. `None` where [`leave_curbuf`] left
/// it -- the editor really has none for those few statements.
pub(super) static CURRENT_BUF: GlobalCell<Stamp> = GlobalCell::new(Stamp::NONE);

/// The tab page the editor is working in. [`CURRENT_WIN`].
pub(super) static CURRENT_TAB: GlobalCell<Stamp> = GlobalCell::new(Stamp::NONE);

// ---------------------------------------------------------------------------
// Becoming current
//
// **Everything below is the only code in the tree that writes which window,
// buffer or tab page is current** — either half of it, the id and the
// mirror. That is what lets the two be relied on to agree, and it is why
// retyping the truth from an address to an id was a change to the six
// functions here rather than to the 125 assignments that used to be spread
// over 33 files.
//
// The funnel takes handles, never raw pointers, so "make this current" cannot
// be spelled with an address whose liveness nobody promised. Building a
// `Win`/`Buf` out of the mirrors needs no `unsafe` *here* — this module is
// inside `winlayer`, which is where the promise those types carry is made.
//
// Taking the id costs one field read of an object the caller has already
// promised is live, and it is the read that makes `Win::current()` safe: an
// id the registry cannot answer for is a window that is gone, which the
// accessor says out loud instead of handing back a dangling address.

impl Win {
    /// Make this the window the editor is working in.
    ///
    /// `CURBUF` is left alone: a window and its buffer move together often
    /// enough to have their own spelling ([`switch_to`]), and the callers
    /// that move only one half mean it.
    #[inline]
    pub fn make_current(self) {
        // The identity is taken for its panic: a handle with none (a null
        // one, or one built before the allocator numbered it) is never
        // current.
        let _ = self.id();
        CURRENT_WIN.set(self.stamp());
        curwin.set(self.raw());
    }
}

impl Buf {
    /// Make this the buffer the editor is working in. [`Win::make_current`].
    #[inline]
    pub fn make_current(self) {
        // The identity is taken for its panic: a handle with none (a null
        // one, or one built before the allocator numbered it) is never
        // current.
        let _ = self.id();
        CURRENT_BUF.set(self.stamp());
        CURBUF.set(self.raw());
    }
}

impl TabPage {
    /// Make this the tab page the editor is working in.
    #[inline]
    pub(crate) fn make_current(self) {
        // The identity is taken for its panic: a handle with none (a null
        // one, or one built before the allocator numbered it) is never
        // current.
        let _ = self.id();
        CURRENT_TAB.set(self.stamp());
        CURTAB.set(self.raw());
    }
}

/// Leave the editor with no current buffer at all.
///
/// Three callers, all of them a moment the editor really has none:
/// `close_windows` after the last buffer went, `no_memfile` between
/// abandoning a buffer and finding another, and `free_buffer` freeing the
/// one that was current. Everything in between reads `CURBUF` as null and is
/// expected to — which is why this is a name of its own rather than an
/// `Option` argument that would read as ordinary.
#[inline]
pub(crate) fn leave_curbuf() {
    CURRENT_BUF.set(Stamp::NONE);
    CURBUF.set(::core::ptr::null_mut::<Buffer>());
}

/// Leave the editor with no current window: what a lib test that made a
/// bare window current puts back before freeing it, and what
/// [`Saved::restore`] puts back when a switch found no window.
pub(crate) fn leave_curwin() {
    CURRENT_WIN.set(Stamp::NONE);
    curwin.set(::core::ptr::null_mut::<Window>());
}

// ---------------------------------------------------------------------------
// Reading which one is current
//
// A handle is `(address, identity)` and the funnel above writes **both**
// halves of it, together, under one call. So the current window is already
// stored: `curwin` is the address and `CURRENT_WIN` is the identity, and the
// reader below is two static loads and a move, with no registry probe and no
// dereference of the window itself.
//
// It did probe, until p28-16 measured what that cost. `Win::current()` is
// inlined at 2,551 sites and `Buf::current()` at 1,539, and a probe is a
// `HandleMap` bounds check plus two dependent loads at every one of them:
// 1.7-2.5 % of the instructions the spell and screen benches retire
// (p28-15 §6). The probe answered a question no caller here asks -- "is the
// current window still registered" -- because the funnel is the only writer
// and it writes a handle the caller had already promised was live.
//
// What that gives up, deliberately: between a window leaving the registry
// and something else being made current, this hands back the window that
// left rather than `None`. `aucmd_restbuf` is the one place that happens,
// and it holds the address in a local across exactly that gap rather than
// reading it back (p28-3 §2). A caller who wants "is it still there" asks
// `WinId::get`, which is what that is for.

/// The current window: the mirror's address with the identity beside it.
///
/// `None` before `win_alloc_first` and after the last window goes.
#[inline(always)]
pub(super) fn current_win() -> Option<Win> {
    let stamp = CURRENT_WIN.get();
    // A current object has an identity, so its stamp is never the empty
    // word.
    (stamp != Stamp::NONE).then(|| Win::from_stamp(curwin.get(), stamp))
}

/// [`current_win`] for the buffer. `None` where [`leave_curbuf`] left it.
#[inline(always)]
pub(super) fn current_buf() -> Option<Buf> {
    let stamp = CURRENT_BUF.get();
    // A current object has an identity, so its stamp is never the empty
    // word.
    (stamp != Stamp::NONE).then(|| Buf::from_stamp(CURBUF.get(), stamp))
}

/// [`current_win`] for the tab page.
#[inline(always)]
pub(super) fn current_tab() -> Option<TabPage> {
    let stamp = CURRENT_TAB.get();
    // A current object has an identity, so its stamp is never the empty
    // word.
    (stamp != Stamp::NONE).then(|| TabPage::from_stamp(CURTAB.get(), stamp))
}

/// The window the mirror names, whatever its identity.
///
/// The switch pair below saves what it displaces as a value rather than as an
/// identity, exactly as the C saved the pointer: it is put back a few
/// statements later with nothing in between that can free it. A null one is
/// the editor between windows, which `Saved::restore` puts back as it found.
#[inline(always)]
fn current_window() -> Win {
    current_win().unwrap_or(Win::NULL)
}

/// [`current_window`] for the buffer mirror.
#[inline(always)]
fn current_buffer() -> Buf {
    current_buf().unwrap_or(Buf::NULL)
}

/// Stand in `win` and the buffer it shows, until [`Saved::restore`].
///
/// This is the editor's most common shape by far: some piece of code has to
/// run against another window because what it calls reads `curwin`/`CURBUF`
/// rather than taking them as arguments. It is *not* `win_enter` — no
/// autocommand fires, no option is copied, nothing is redrawn — so the pair
/// must bracket a stretch that does none of those things either.
#[inline]
#[must_use = "the switch is undone by Saved::restore; dropping this leaves \
              the editor standing in the wrong window"]
pub(crate) fn switch_to(win: Win) -> Saved {
    let saved = Saved(Displaced::WindowAndBuffer(
        current_window(),
        current_buffer(),
    ));
    win.make_current();
    win.buffer().make_current();
    saved
}

/// [`switch_to`] for `curwin` alone, leaving `CURBUF` where it is.
#[inline]
#[must_use = "the switch is undone by Saved::restore"]
pub(crate) fn switch_window(win: Win) -> Saved {
    let saved = Saved(Displaced::Window(current_window()));
    win.make_current();
    saved
}

/// [`switch_to`] for `CURBUF` alone, leaving `curwin` where it is.
#[inline]
#[must_use = "the switch is undone by Saved::restore"]
pub(crate) fn switch_buffer(buffer: Buf) -> Saved {
    let saved = Saved(Displaced::Buffer(current_buffer()));
    buffer.make_current();
    saved
}

/// What a switch displaced, and the only way to put it back.
///
/// # Why there is no `Drop`
///
/// A guard that restored itself would be wrong here, not merely
/// unidiomatic. The editor re-enters itself constantly — an autocommand, a
/// Lua callback, `:normal` — and a body between a switch and its restore
/// may deliberately end somewhere else: `do_mousescroll` leaves `curwin`
/// wherever the wheel took it and the caller reads that before putting the
/// old one back, `aucmd_restbuf` decides *by handle* which window to return
/// to because the saved one may have been closed, and several callers give
/// the globals back early and then keep working. A `Drop` would fire at the
/// end of the scope in every one of those, silently, after the code that
/// cared had already moved on. Restoring is a statement the caller writes.
#[derive(Debug)]
pub(crate) struct Saved(Displaced);

/// Which halves a switch took, and what they held.
#[derive(Debug)]
enum Displaced {
    /// [`switch_to`]: the window, with `CURBUF` following it back -- or,
    /// when there was no window, going back to the buffer beside it.
    WindowAndBuffer(Win, Buf),
    /// [`switch_window`]: the window alone.
    Window(Win),
    /// [`switch_buffer`]: the buffer alone.
    Buffer(Buf),
}

impl Saved {
    /// Put back exactly the halves the switch took.
    ///
    /// A window brings its buffer with it: `CURBUF` is re-read from the
    /// window being returned to rather than restored from a second saved
    /// value, which is what the hand-written pairs this replaces did and is
    /// the answer that stays right when the buffer behind that window
    /// changed while the caller was away.
    ///
    /// A null window or buffer is the editor between them, and is put back
    /// as none; with no window to follow, `CURBUF` goes back to what the
    /// switch found.
    #[inline]
    pub(crate) fn restore(self) {
        fn put_back_buffer(displaced: Buf) {
            if displaced.is_null() {
                leave_curbuf();
            } else {
                displaced.make_current();
            }
        }
        match self.0 {
            Displaced::WindowAndBuffer(win, _) if !win.raw().is_null() => {
                win.make_current();
                win.buffer().make_current();
            }
            Displaced::WindowAndBuffer(_, buf) => {
                leave_curwin();
                put_back_buffer(buf);
            }
            Displaced::Window(win) if win.raw().is_null() => leave_curwin(),
            Displaced::Window(win) => win.make_current(),
            Displaced::Buffer(buf) => put_back_buffer(buf),
        }
    }
}

/// The slot of a window the editor has given up, held until it is emptied.
///
/// By the time a window is freed its fields have been released one by one
/// (`win_free`), its handle has left the registry and it is off every list,
/// so what is left is the slot [`Win::alloc`] filled, with a `Window` whose
/// contents no longer count. That is what this holds: every handle to the
/// window went stale when it was made, and dropping it frees the slot and
/// runs no destructor, as the `xfree` of the block did not. The free path
/// either drops it at once or parks it in the deferred-free set while an
/// autocommand runs ([`super::defer_free_window`]); the slot takes no new
/// window until then.
///
/// The one way to get one is [`forget_window`], so the only windows emptied
/// this way are the ones the window registry held.
#[derive(Debug)]
pub(crate) struct ParkedWindow {
    /// Always `Some` until `drop` takes it.
    parked: Option<Parked<Window>>,
}

impl Drop for ParkedWindow {
    fn drop(&mut self) {
        if let Some(parked) = self.parked.take() {
            parked.abandon();
        }
    }
}

/// Take `win` out of the registry for good, answering its slot: every
/// handle to it is stale from here on, and dropping the answer frees the
/// slot. `None` when nothing is registered under its handle.
///
/// The caller must have released the window's fields and unlinked it from
/// every list first -- the re-entry rule's "it did not survive", made by
/// the free path itself. A window that stays alive out of the registry (the
/// idle autocommand window) goes through [`super::hide_window`] instead.
#[must_use = "dropping the answer is the free; ignoring it frees the window at once"]
pub(crate) fn forget_window(win: Win) -> Option<ParkedWindow> {
    super::handles::unregister_window(win).map(|slot| ParkedWindow {
        parked: Some(Parked::of(slot)),
    })
}

impl Win {
    /// A window built **in its slot** by `init` (a window is several
    /// kilobytes; building one and moving it in was a `memcpy` of all of
    /// it), before the allocator has given it a handle -- so nothing is read
    /// and the handle is zero until [`Win::set_handle`]. The slot is the
    /// window list's from here on: [`forget_window`] empties it, without
    /// running the window's destructor.
    pub(crate) fn alloc(init: impl FnOnce(&mut MaybeUninit<Window>) -> &mut Window) -> Self {
        Self::at(
            Window::pool().with_mut(|pool| pool.insert_with(init)),
            Handle::NONE,
        )
    }
}

/// A `Buffer` that lives **outside** the handle registry and off the buffer
/// list: scratch storage whose only real content is a memline.
///
/// Two exist, and both are the editor's own business rather than the user's:
/// `ml_recover`'s, which holds the memline of the swap file being read, and
/// `open_spellbuf`'s, which backs a `.sug` word list with a swap file so it
/// need not stay in memory. Neither is given a handle, neither is registered
/// (so `find_buf`, `nvim_list_bufs` and every `FOR_ALL_BUFFERS` walk
/// are blind to them, which is the point), and neither goes through
/// `close_buffer`/`free_buffer` — each is freed by the code that made it.
///
/// The constructor is named so that "the registry owns every buffer" has a
/// stated exception rather than a silent one; `buffer::list::new_buffer` is
/// the ordinary path, which starts from one of these too.
///
/// Most fields stay zero and are *not* valid buffer state: string options
/// are null, there are no `b:` variables and no undo information. Only what
/// the caller fills in afterwards may be read.
pub(crate) fn alloc_unregistered_buffer() -> SlotBox<Buffer> {
    SlotBox::new_with(|storage: &mut MaybeUninit<Buffer>| {
        // A reused slot holds whatever its last tenant left: zero it first,
        // as `xcalloc` would have.
        // SAFETY: the place is the slot's, handed over vacant; all-zero bytes
        // are written over it, and nothing reads them as a `Buffer` until the
        // fields below are made valid.
        unsafe { storage.as_mut_ptr().write_bytes(0, 1) };
        let at = storage.as_mut_ptr();
        // The fields a zeroed `Buffer` is *not* a valid value for -- an empty
        // `Vec` holds a non-null dangling pointer, not a zero one, a `HashMap`
        // holds a seeded hasher, and an `Option<XString>`'s niche is the
        // vector's capacity, so all-zero bytes are `Some` of a null vector --
        // are the user-command list, the keymap, the remembered window
        // positions, the `'complete'` function callbacks, the buffer's syntax
        // block (`SynBlock::new`), what the memline owns, the two extmark
        // tables, the ShaDa extra data (a vector inside an `Option`), the `U`
        // line, the two 'vartabstop' arrays, the string options and the file
        // name. The unit case `buffer::a_fresh_buffer_holds_valid_owners`
        // reads each of them back.
        // SAFETY: all are inside the place just zeroed, nothing has read or
        // dropped them, and `write` does not drop what was there.
        unsafe { (&raw mut (*at).b_ucmds).write(Vec::new()) };
        unsafe { (&raw mut (*at).b_kmap_ga).write(Vec::new()) };
        unsafe { (&raw mut (*at).b_wininfo).write(Vec::new()) };
        unsafe { (&raw mut (*at).b_p_cpt_cb).write(Vec::new()) };
        unsafe { (&raw mut (*at).b_s).write(crate::types::SynBlock::new()) };
        unsafe { (&raw mut (*at).b_ml).write(core::cell::UnsafeCell::new(MemLine::closed())) };
        unsafe { (&raw mut (*at).b_marktree).write(MarkTree::EMPTY) };
        unsafe { (&raw mut (*at).b_extmark_ns).write(id_map()) };
        unsafe { (&raw mut (*at).additional_data).write(None) };
        unsafe { (&raw mut (*at).b_u_line).write(None) };
        // The 'vartabstop' arrays: a boxed slice's `None` is a null data
        // pointer, but only a `Sized` box's niche is promised.
        unsafe { (&raw mut (*at).b_p_vts_array).write(None) };
        unsafe { (&raw mut (*at).b_p_vsts_array).write(None) };
        // ... and the string options, which are `Option<XString>`s.
        for field in BUF_STRING_OPTIONS {
            let slot = at.wrapping_byte_add(field.field_offset());
            // SAFETY: as above; the selector names one such field of the place.
            unsafe { slot.cast::<Option<XString>>().write(None) };
        }
        // ... and the file name, for the same reason: `BufName` holds two of
        // those `Option<XString>`s and an enum whose niche is one of them.
        // SAFETY: as above.
        unsafe { (&raw mut (*at).name).write(BufName::default()) };
        // SAFETY: all-zero bytes are otherwise what upstream's
        // `xcalloc(1, sizeof(Buffer))` hands a fresh buffer.
        unsafe { storage.assume_init_mut() }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{bare_buffer, free_bare_buffer};
    use crate::global_cell::editor_state_lock;
    use crate::types::{ColNr, LineNr};
    use crate::window::{bare_window, free_bare_window};

    /// The current window and buffer are a registered block's address with
    /// its identity beside it: what `Win::current()` hands back writes the
    /// very block the caller's own handle reads, and a switch puts back
    /// exactly what it displaced -- here, no window at all.
    #[test]
    fn a_switch_stands_in_a_window_and_puts_the_old_one_back() {
        let _held = editor_state_lock();
        let mut win = bare_window();
        let buffer = bare_buffer();
        win.w_buffer = buffer;
        leave_curwin();
        leave_curbuf();

        let saved = switch_to(win);
        assert!(win.is_current() && buffer.is_current());
        assert_eq!(Win::current(), win);
        assert_eq!(Buf::current(), buffer);
        Win::current().w_cursor.lnum = LineNr::new(5);
        assert_eq!(win.w_cursor.lnum, LineNr::new(5));
        Buf::current().set_b_changed(1);
        assert_eq!(win.buffer().b_changed(), 1);
        saved.restore();
        assert_eq!(Win::current_or_none(), None);
        assert_eq!(Buf::current_or_none(), None);

        let saved = switch_buffer(buffer);
        assert_eq!(Buf::current(), buffer);
        saved.restore();
        assert_eq!(Buf::current_or_none(), None);

        free_bare_window(win);
        free_bare_buffer(buffer);
    }

    /// A forgotten window's block goes back and its id answers `None`; its
    /// neighbours' handles stay good.
    #[test]
    fn a_forgotten_window_is_found_no_more() {
        let _held = editor_state_lock();
        let gone = bare_window();
        let mut stays = bare_window();
        let id = gone.id();
        assert_eq!(id.get(), Some(gone));
        free_bare_window(gone);
        assert_eq!(id.get(), None);
        assert!(forget_window(gone).is_none(), "freed once");
        stays.w_cursor.col = ColNr::new(3);
        assert_eq!(
            stays.id().get().map(|win| win.w_cursor.col),
            Some(ColNr::new(3))
        );
        free_bare_window(stays);
    }

    /// The fields a zeroed `Buffer` is not a valid value for are written
    /// before anything reads them: the owners read back empty, and dropping
    /// the block drops each of them.
    #[test]
    fn an_unregistered_buffer_holds_valid_owners() {
        let owned = alloc_unregistered_buffer();
        let buffer = Buf::owned_by(&owned);
        assert!(!buffer.is_null() && buffer.try_id().is_none());
        assert!(buffer.read().b_ucmds.is_empty() && buffer.read().b_wininfo.is_empty());
        assert!(buffer.read().b_kmap_ga.is_empty() && buffer.read().b_p_cpt_cb.is_empty());
        assert!(buffer.read().b_u_line.is_none() && buffer.read().additional_data.is_none());
        assert!(buffer.read().b_p_vts_array.is_none() && buffer.read().b_p_vsts_array.is_none());
        drop(owned);
    }
}
