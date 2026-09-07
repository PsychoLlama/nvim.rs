//! The window, buffer and tabpage graph: which of each exists, in what
//! order, and which one is current.
//!
//! The transpiler parked these beside `main()` because upstream declares them
//! in `globals.h`, but every one of them is a node of the tree [`winlayer`]
//! wraps: the doubly-linked window list (`firstwin`/`lastwin`), the buffer
//! list (`firstbuf`/`lastbuf`), the tabpage list (`first_tabpage`), the frame
//! tree's root (`topframe`), and the `cur*` cursors into all three. The
//! command-line window's saved graph (`cmdwin_*`) belongs with them: it is a
//! second, temporary editor tree that the first one is swapped out for.
//!
//! Which one is current is a [`WinId`]/[`BufId`]/[`TabId`], like the list
//! heads: [`CURRENT_WIN`] and its two siblings are the truth, and the raw
//! `curwin`/`curbuf`/`curtab` beside them are mirrors the same setters
//! `prevwin`, `lastused_tabpage` and the `cmdwin_*` block are ids too, and
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
// Not `forbid(unsafe_code)`: that lint rejects the name-mangling override on
// `curwin`, whose symbol plugins and the functional suite read directly.
#![deny(unsafe_op_in_unsafe_fn)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::{Buf, BufId, FrameId, TabId, TabPage, Win, WinId};
use crate::global_cell::GlobalCell;
use crate::types::{Buffer, Tabpage, Window};
use core::ffi::c_int;

pub(crate) static firstwin: GlobalCell<Option<WinId>> = GlobalCell::new(None);
pub(crate) static lastwin: GlobalCell<Option<WinId>> = GlobalCell::new(None);
pub(crate) static prevwin: GlobalCell<Option<WinId>> = GlobalCell::new(None);
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
pub(crate) static topframe: GlobalCell<Option<FrameId>> = GlobalCell::new(None);
pub(crate) static first_tabpage: GlobalCell<Option<TabId>> = GlobalCell::new(None);
/// The current tab page's address, mirroring [`CURRENT_TAB`]. [`curwin`],
/// minus the symbol: only [`TabPage::is_current`] reads it.
pub(super) static curtab: GlobalCell<*mut Tabpage> =
    GlobalCell::new(::core::ptr::null_mut::<Tabpage>());
pub(crate) static lastused_tabpage: GlobalCell<Option<TabId>> = GlobalCell::new(None);
pub(crate) static firstbuf: GlobalCell<Option<BufId>> = GlobalCell::new(None);
pub(crate) static lastbuf: GlobalCell<Option<BufId>> = GlobalCell::new(None);
/// The current buffer's address, mirroring [`CURRENT_BUF`]. Private: a
/// buffer has no `is_current`, so nothing outside this module reads it, and
/// [`switch_buffer`] is the one place that does.
static curbuf: GlobalCell<*mut Buffer> = GlobalCell::new(::core::ptr::null_mut::<Buffer>());
pub(crate) static cmdwin_type: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static cmdwin_result: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static cmdwin_level: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static cmdwin_buf: GlobalCell<Option<BufId>> = GlobalCell::new(None);
pub(crate) static cmdwin_win: GlobalCell<Option<WinId>> = GlobalCell::new(None);
pub(crate) static cmdwin_old_curwin: GlobalCell<Option<WinId>> = GlobalCell::new(None);
pub(crate) static cmdline_win: GlobalCell<Option<WinId>> = GlobalCell::new(None);

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
pub(super) static CURRENT_WIN: GlobalCell<Option<WinId>> = GlobalCell::new(None);

/// The buffer the editor is working in. `None` where [`leave_curbuf`] left
/// it -- the editor really has none for those few statements.
pub(super) static CURRENT_BUF: GlobalCell<Option<BufId>> = GlobalCell::new(None);

/// The tab page the editor is working in. [`CURRENT_WIN`].
pub(super) static CURRENT_TAB: GlobalCell<Option<TabId>> = GlobalCell::new(None);

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
    /// `curbuf` is left alone: a window and its buffer move together often
    /// enough to have their own spelling ([`switch_to`]), and the callers
    /// that move only one half mean it.
    #[inline]
    pub fn make_current(self) {
        CURRENT_WIN.set(Some(self.id()));
        curwin.set(self.raw());
    }
}

impl Buf {
    /// Make this the buffer the editor is working in. [`Win::make_current`].
    #[inline]
    pub fn make_current(self) {
        CURRENT_BUF.set(Some(self.id()));
        curbuf.set(self.raw());
    }
}

impl TabPage {
    /// Make this the tab page the editor is working in.
    #[inline]
    pub(crate) fn make_current(self) {
        CURRENT_TAB.set(Some(self.id()));
        curtab.set(self.raw());
    }
}

/// Leave the editor with no current buffer at all.
///
/// Three callers, all of them a moment the editor really has none:
/// `close_windows` after the last buffer went, `no_memfile` between
/// abandoning a buffer and finding another, and `free_buffer` freeing the
/// one that was current. Everything in between reads `curbuf` as null and is
/// expected to — which is why this is a name of its own rather than an
/// `Option` argument that would read as ordinary.
#[inline]
pub(crate) fn leave_curbuf() {
    CURRENT_BUF.set(None);
    curbuf.set(::core::ptr::null_mut::<Buffer>());
}

/// The window the mirror names, whatever its identity.
///
/// The switch pair below saves what it displaces as a value rather than as an
/// identity, exactly as the C saved the pointer: it is put back a few
/// statements later with nothing in between that can free it. Reading the
/// handle out of it is sound for the same reason.
#[inline(always)]
fn current_window() -> Win {
    // SAFETY: the mirror is written only by the funnel above, beside the
    // identity it mirrors, so it names the live current window or is null --
    // and `Win::new` tolerates null.
    unsafe { Win::new(curwin.get()) }
}

/// [`current_window`] for the buffer mirror.
#[inline(always)]
fn current_buffer() -> Buf {
    // SAFETY: as [`current_window`]; `leave_curbuf` spells "none" as null.
    unsafe { Buf::new(curbuf.get()) }
}

/// Stand in `win` and the buffer it shows, until [`Saved::restore`].
///
/// This is the editor's most common shape by far: some piece of code has to
/// run against another window because what it calls reads `curwin`/`curbuf`
/// rather than taking them as arguments. It is *not* `win_enter` — no
/// autocommand fires, no option is copied, nothing is redrawn — so the pair
/// must bracket a stretch that does none of those things either.
#[inline]
#[must_use = "the switch is undone by Saved::restore; dropping this leaves \
              the editor standing in the wrong window"]
pub(crate) fn switch_to(win: Win) -> Saved {
    let saved = Saved(Displaced::WindowAndBuffer(current_window()));
    win.make_current();
    win.buffer().make_current();
    saved
}

/// [`switch_to`] for `curwin` alone, leaving `curbuf` where it is.
#[inline]
#[must_use = "the switch is undone by Saved::restore"]
pub(crate) fn switch_window(win: Win) -> Saved {
    let saved = Saved(Displaced::Window(current_window()));
    win.make_current();
    saved
}

/// [`switch_to`] for `curbuf` alone, leaving `curwin` where it is.
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
pub(crate) struct Saved(Displaced);

/// Which halves a switch took, and what they held.
enum Displaced {
    /// [`switch_to`]: the window, with `curbuf` following it back.
    WindowAndBuffer(Win),
    /// [`switch_window`]: the window alone.
    Window(Win),
    /// [`switch_buffer`]: the buffer alone.
    Buffer(Buf),
}

impl Saved {
    /// Put back exactly the halves the switch took.
    ///
    /// A window brings its buffer with it: `curbuf` is re-read from the
    /// window being returned to rather than restored from a second saved
    /// value, which is what the hand-written pairs this replaces did and is
    /// the answer that stays right when the buffer behind that window
    /// changed while the caller was away.
    #[inline]
    pub(crate) fn restore(self) {
        match self.0 {
            Displaced::WindowAndBuffer(win) => {
                win.make_current();
                win.buffer().make_current();
            }
            Displaced::Window(win) => win.make_current(),
            Displaced::Buffer(buf) => buf.make_current(),
        }
    }
}
