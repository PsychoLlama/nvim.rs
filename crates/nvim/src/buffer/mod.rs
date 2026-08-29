//! Buffers: the list of them, and the state of one.
//!
//! Carved by the stage:
//!
//! | child | what |
//! | --- | --- |
//! | [`open`] | reading a file into a buffer, and the scratch forms |
//! | [`close`] | unloading, deleting and wiping one |
//! | [`switch`] | `:buffer`, `:bnext`, `:bdelete` |
//! | [`enter`] | making a buffer current |
//! | [`list`] | creating an entry in the buffer list, and finding one |
//! | [`expand`] | completing a buffer name |
//! | [`pos`] | the per-window remembered cursor position |
//! | [`info`] | `:ls`, CTRL-G, `'title'` |
//! | [`name`] | a buffer's file name and the alternate file |
//! | [`all`] | `:ball` |
//! | [`modeline`] | `chk_modeline()` and `'modelines'` |
//! | [`type`] | the `'buftype'` predicates |
//!
//! What stays here is the flag alphabet the twelve share (`DOBUF_*`,
//! `BLN_*`, `BFA_*`, `READ_*`), the [`BufRef`] layer every one of them uses to
//! survive an autocommand (upstream's `set_bufref`/`bufref_valid`, plus
//! [`buf_valid`]), the
//! `b:changedtick` and buffer-number counters, `buf_meta_total` -- the
//! marktree accessor `buffer.h` had as a `static inline` -- and the shims for
//! the neighbours more than one child reaches.
//!
//! # Surviving an autocommand
//!
//! Half of this family fires autocommands (`BufEnter`, `BufLeave`,
//! `BufUnload`, `BufDelete`, `BufWipeout`, ...) and **an autocommand may free
//! the buffer in hand**.  The C's answer is `bufref_T`: remember the pointer
//! together with the buffer number and a global free counter, and ask again
//! afterwards.  [`BufRef`] is that answer as a value type: it is *taken* from
//! a [`Buf`] ([`BufRef::of`], [`BufRef::of_opt`]) rather than written through
//! an out-pointer the way `set_bufref()` was, and [`BufRef::get`]
//! re-validates and only then hands a [`Buf`] back, so a stale pointer cannot
//! be dereferenced by accident.  The discipline the whole family follows is
//! **hold no [`Buf`], [`Win`] or borrow across a call that can fire an
//! autocommand**: take one from a `BufRef` or from the `curbuf`/`curwin`
//! cells on each side of it instead.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::autocmd::{apply_autocmds, apply_autocmds_retval, block_autocmds, unblock_autocmds};
use crate::change::unchanged;
use crate::ex_cmds::do_ecmd;
use crate::ex_docmd::do_cmdline_cmd;
use crate::ex_eval::aborting;
use crate::fold::{clear_folding, fold_update_all};
use crate::global_cell::GlobalCell;
use crate::main::{curbuf, curwin};
use crate::mark::setpcmark;
use crate::memline::ml_delete;
use crate::memory::xfree;
use crate::message::emsg_ptr;
use crate::normal::end_visual_mode;
use crate::option::shortmess;
use crate::os::cshim::gettext_ptr;
use crate::syntax::reset_synblock;
use crate::types::{
    AlignTextPos, CdCause, ExtmarkOp, MarkAdjustMode, MarkTree, MetaIndex, OptValType,
    UndoObjectType, WinSplit, WinStyle, bfa_values, bln_values, buf_T, bufref_T,
    dobuf_action_values, dobuf_start_values, etype_T, event_T, exarg_T, getf_values, linenr_T,
    uint32_t, varnumber_T,
};
use crate::undo::buf_is_changed;
use crate::window::{check_colorcolumn, close_windows, window_layout_lock, window_layout_unlock};
use crate::winlayer::{Buf, Win, buffers_back, first_buffer, last_buffer};

// The carve of the transpiled module; see each child's docs.
mod all;
mod close;
mod enter;
mod expand;
mod info;
mod list;
mod modeline;
mod name;
mod open;
mod pos;
mod switch;
mod r#type;

pub use self::all::*;
pub use self::close::*;
pub use self::enter::*;
pub use self::expand::*;
pub use self::info::*;
pub use self::list::*;
pub use self::modeline::*;
pub use self::name::*;
pub use self::open::*;
pub use self::pos::*;
pub use self::switch::*;
pub use self::r#type::*;
pub const _ISdigit: ::core::ffi::c_uint = 2048;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleMinimal: WinStyle = 1;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitLeft: WinSplit = 0;
pub const DO_NOT_FREE_CNT: ::core::ffi::c_uint = 1073741823;
pub const DI_FLAGS_FIX: ::core::ffi::c_uint = 4;
pub const DI_FLAGS_RO_SBX: ::core::ffi::c_uint = 2;
pub const DI_FLAGS_RO: ::core::ffi::c_uint = 1;
pub const kCdCauseAuto: CdCause = 2;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const kMarkAdjustTerm: MarkAdjustMode = 2;
pub const kMarkAdjustApi: MarkAdjustMode = 1;
pub const kMarkAdjustNormal: MarkAdjustMode = 0;
pub const GETF_SWITCH: getf_values = 4;
pub const GETF_ALT: getf_values = 2;
pub const GETF_SETMARK: getf_values = 1;
pub const BLN_NOCURWIN: bln_values = 128;
pub const BLN_NOOPT: bln_values = 16;
pub const BLN_NEW: bln_values = 8;
pub const BLN_DUMMY: bln_values = 4;
pub const BLN_LISTED: bln_values = 2;
pub const BLN_CURBUF: bln_values = 1;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_SPLIT: dobuf_action_values = 1;
pub const DOBUF_GOTO: dobuf_action_values = 0;
pub const DOBUF_MOD: dobuf_start_values = 3;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const DOBUF_CURRENT: dobuf_start_values = 0;
pub type dobuf_flags_value = ::core::ffi::c_uint;
pub const DOBUF_SKIPHELP: dobuf_flags_value = 4;
pub const DOBUF_FORCEIT: dobuf_flags_value = 1;
pub const BFA_IGNORE_ABORT: bfa_values = 8;
pub const BFA_KEEP_UNDO: bfa_values = 4;
pub const BFA_WIPE: bfa_values = 2;
pub const BFA_DEL: bfa_values = 1;
pub const READ_NOWINENTER: ::core::ffi::c_uint = 128;
pub const ETYPE_MODELINE: etype_T = 4;
pub const READ_BUFFER: ::core::ffi::c_uint = 8;
pub const READ_STDIN: ::core::ffi::c_uint = 4;
pub const READ_NEW: ::core::ffi::c_uint = 1;
pub const READ_FIFO: ::core::ffi::c_uint = 64;
pub const READ_NOFILE: ::core::ffi::c_uint = 256;
pub const BCO_NOHELP: ::core::ffi::c_uint = 4;
pub const BCO_ENTER: ::core::ffi::c_uint = 1;
pub const kBffInitChangedtick: ::core::ffi::c_uint = 2;
pub const kBffClearWinInfo: ::core::ffi::c_uint = 1;
pub const BCO_ALWAYS: ::core::ffi::c_uint = 2;
pub const ECMD_FORCEIT: ::core::ffi::c_uint = 8;
pub const ECMD_ONE: ::core::ffi::c_int = 1;
#[derive(Copy, Clone)]
pub struct bufmatch_T {
    pub buf: *mut buf_T,
    pub match_0: *mut ::core::ffi::c_char,
}
pub const FUZZY_SCORE_NONE: ::core::ffi::c_int = -2147483648;

pub const READ_DUMMY: ::core::ffi::c_uint = 16;
pub const ECMD_HIDE: ::core::ffi::c_uint = 1;
pub const ECMD_OLDBUF: ::core::ffi::c_uint = 4;
pub const ECMD_LAST: ::core::ffi::c_int = -1;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
crate::flag_set! {
    /// What has and has not happened to a buffer -- upstream's `BF_*`, the
    /// bits `buf_T::b_flags` carries.
    pub struct BufFlags;

    /// The buffer was recovered from a swap file.
    const RECOVERED = 0x1;
    /// `'readonly'` has not been checked for this buffer yet.
    const CHECK_RO = 0x2;
    /// The buffer has never been loaded, so its options still hold their
    /// defaults rather than anything a file or a modeline set.
    const NEVERLOADED = 0x4;
    /// The buffer's contents are not what its file holds -- it was never
    /// read, or `:file` renamed it.
    const NOTEDITED = 0x8;
    /// The file did not exist when the buffer was created.
    const NEW = 0x10;
    /// [`Self::NEW`] as it stood when the buffer was last *written*, which
    /// is what decides whether `'cpoptions'`'s `+` applies.
    const NEW_W = 0x20;
    /// Reading the file failed part-way, so the buffer is incomplete.
    const READERR = 0x40;
    /// A scratch buffer that exists only to be looked at once and thrown
    /// away -- `:vimgrep`'s and `:helpgrep`'s.
    const DUMMY = 0x80;
    /// `'syntax'` was set for this buffer, so `:syntax` state exists.
    const SYN_SET = 0x200;

    /// The three a successful write clears, and the ones `:write` copies
    /// from the buffer it wrote into the one it wrote *for*.
    const WRITE_MASK = 0x8 | 0x10 | 0x40;
}
pub const KEYMAP_INIT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NMARKS: ::core::ffi::c_int =
    'z' as ::core::ffi::c_int - 'a' as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
#[inline(always)]
pub fn buf_get_changedtick(buf: Buf) -> varnumber_T {
    // SAFETY: `b:changedtick`'s dict item is always a `VAR_NUMBER`, which is
    // the only variant this union is ever given here.
    unsafe { buf.changedtick_di.di_tv.vval.v_number }
}
static e_attempt_to_delete_buffer_that_is_in_use_str: &::core::ffi::CStr =
    c"E937: Attempt to delete a buffer that is in use: %s";
static buf_free_count: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static top_file_num: GlobalCell<::core::ffi::c_int> = GlobalCell::new(1 as ::core::ffi::c_int);

/// Run `b:undo_ftplugin` with the buffer and the window pinned, so that what
/// it does cannot close either out from under the caller.
pub(crate) fn trigger_undo_ftplugin(mut buf: Buf, mut win: Win) {
    let win_was_locked: bool = win.w_locked;
    layout_lock();
    buf.b_locked += 1;
    win.w_locked = true;
    // b:undo_ftplugin may be set, undo it
    run_cmdline(c"if exists('b:undo_ftplugin') | exe b:undo_ftplugin | endif");
    buf.b_locked -= 1;
    win.w_locked = win_was_locked;
    layout_unlock();
}

// ---------------------------------------------------------------------------
// A buffer that survives an autocommand

/// A remembered buffer, as the C's `bufref_T`: the pointer, the buffer number
/// it had, and the value of the global free counter when it was taken.
///
/// Autocommands fired anywhere in this family may free the buffer in hand, so
/// nothing may be dereferenced across one. Take a `BufRef` before the call and
/// [`get`](BufRef::get) it afterwards: it re-validates and only then answers a
/// [`Buf`]. The buffer number is part of the check because a `:bwipe` followed
/// by a `:new` can hand the same allocation back as a *different* buffer.
#[derive(Clone, Copy)]
pub(crate) struct BufRef(bufref_T);

impl BufRef {
    /// The reference that names no buffer, for a cell's initial value.
    pub(crate) const NONE: Self = BufRef(bufref_T::new());

    /// `set_bufref()`, where the C passes a pointer that may be NULL.
    ///
    /// Safe because the absence is in the type: this used to take a
    /// `*mut buf_T` and read `(*buf).handle` whenever it was non-null, which
    /// made it a *safe* function with an unstated precondition about a
    /// pointer -- the shape p23-5 rules out.
    pub(crate) fn of_opt(buf: Option<Buf>) -> Self {
        BufRef(bufref_T {
            br_buf: buf.map_or(ptr::null_mut(), Buf::raw),
            br_fnum: buf.map_or(0, |b| b.handle as c_int),
            br_buf_free_count: buf_free_count.get(),
        })
    }

    /// `set_bufref()` over a buffer the caller already holds.
    pub(crate) fn of(buf: Buf) -> Self {
        Self::of_opt(Some(buf))
    }

    /// `bufref_valid()`: whether the remembered buffer is still the buffer it
    /// was. Only walks the list when the free counter has moved.
    pub(crate) fn valid(self) -> bool {
        self.0.br_buf_free_count == buf_free_count.get()
            || buffers_back().any(|b| b.raw() == self.0.br_buf && b.handle == self.0.br_fnum)
    }

    /// The buffer, if it is still the one that was remembered.
    ///
    /// Null answers `None`, which `bufref_valid()` does not: the C's callers
    /// test the pointer separately wherever it can be null.
    pub(crate) fn get(self) -> Option<Buf> {
        let buf = self.0.br_buf;
        // SAFETY: `valid` found this pointer in the buffer list, or the free
        // counter has not moved since it was taken from a live one.
        (!buf.is_null() && self.valid()).then(|| unsafe { Buf::new(buf) })
    }

    /// The remembered pointer, valid or not -- for the two comparisons the C
    /// makes without dereferencing it.
    pub(crate) fn raw(self) -> *mut buf_T {
        self.0.br_buf
    }

    /// The record itself, for the two places it has to live in a C struct:
    /// `aco_save_T`'s `new_curbuf`.
    pub(crate) const fn record(self) -> bufref_T {
        self.0
    }

    /// [`record`](BufRef::record) the other way.
    pub(crate) const fn of_record(record: bufref_T) -> Self {
        BufRef(record)
    }
}

/// Whether `buf` is still in the buffer list.
///
/// Can be slow when there are many buffers; prefer [`BufRef`].
///
/// # Safety
/// `buf` may be any pointer, live or dangling: it is only ever compared.
///
/// The null test is a short circuit and nothing more: no buffer in the list
/// has a null address, so removing it changes no answer. That is why the
/// "NULL is not a valid buffer" case cannot fail — it states the contract
/// callers rely on rather than covering a branch.
pub unsafe fn buf_valid(buf: *mut buf_T) -> bool {
    // Assume that we more often have a recent buffer, start with the last one.
    !buf.is_null() && buffers_back().any(|b| b.raw() == buf)
}

static lasttitle: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static lasticon: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub const NO_LOCAL_UNDOLEVEL: ::core::ffi::c_int = -123456 as ::core::ffi::c_int;
pub const SID_MODELINE: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const SEA_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEA_DIALOG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SEA_QUIT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SEA_RECOVER: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const STL_IN_ICON: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STL_IN_TITLE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;

/// The buffer's running total of one kind of extmark metadata, kept at the
/// root of its marktree. `buffer.h` had this as a `static inline`.
pub fn buf_meta_total(b: Buf, m: MetaIndex) -> uint32_t {
    // SAFETY: the buffer's own marktree, read through the layout the
    // extmark code gives it.
    unsafe { (*(&raw const b.b_marktree as *const MarkTree)).meta_root[m as usize] }
}

// ---------------------------------------------------------------------------
// The neighbours more than one child reaches
//
// Every one of these is still an `unsafe fn` over raw pointers, and
// all any of them needs is a live buffer or window -- which `Buf`/`Win` carry.
// One wrapper per *exit* therefore makes each call site ordinary code, and the
// cost is the number of distinct neighbours rather than the number of calls.
// They live here rather than in a child because each is reached from two or
// more of them, and a child sees its parent's private items.

/// `_()`.
pub(crate) fn tr(msg: &CStr) -> *mut c_char {
    tr_raw(msg.as_ptr())
}

/// `_()` over a pointer, for the message statics `main.rs` holds as byte
/// arrays.
pub(crate) fn tr_raw(msg: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated literal or message static.
    unsafe { gettext_ptr(msg).as_ptr().cast_mut() }
}

/// `emsg(_(msg))`.
pub(crate) fn err(msg: &CStr) {
    err_raw(tr(msg));
}

/// `emsg()` over an already translated message.
pub(crate) fn err_raw(msg: *mut c_char) {
    // SAFETY: a NUL-terminated message.
    unsafe { emsg_ptr(msg) };
}

/// The current buffer. Null only between `open_buffer()` freeing the last one
/// and finding a replacement, which is why [`current_buf`] exists beside it.
pub(crate) fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit, bar that one window.
    unsafe { Buf::current() }
}

/// The current buffer, or `None` where the C tests `curbuf != NULL`.
pub(crate) fn current_buf() -> Option<Buf> {
    let buf = curbuf.get();
    // SAFETY: non-null, hence live.
    (!buf.is_null()).then(|| unsafe { Buf::new(buf) })
}

/// The current window. Null only while exiting, which is why
/// [`current_win`] exists beside it.
pub(crate) fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// The current window, or `None` where the C tests `curwin != NULL`.
pub(crate) fn current_win() -> Option<Win> {
    let win = curwin.get();
    // SAFETY: non-null, hence live.
    (!win.is_null()).then(|| unsafe { Win::new(win) })
}

/// The first buffer in the list, `None` before any exists.
pub(crate) fn first_buf() -> Option<Buf> {
    first_buffer()
}

/// The last buffer in the list, `None` before any exists.
pub(crate) fn last_buf() -> Option<Buf> {
    last_buffer()
}

/// `apply_autocmds(event, NULL, NULL, false, buf)`.
///
/// **Everything the caller holds may be stale afterwards** -- take a
/// [`BufRef`] first.
pub(crate) fn fire(event: event_T, mut buf: Buf) -> bool {
    // SAFETY: a live buffer; both name arguments are optional.
    unsafe { apply_autocmds(event, ptr::null_mut(), ptr::null_mut(), false, buf.raw()) }
}

/// `apply_autocmds(event, buf->b_fname, buf->b_fname, false, buf)`, the form
/// the unload/delete/wipe events take.
pub(crate) fn fire_named(event: event_T, mut buf: Buf) -> bool {
    let (name, raw) = (buf.b_fname, buf.raw());
    // SAFETY: a live buffer and its own file name.
    unsafe { apply_autocmds(event, name, name, false, raw) }
}

/// `apply_autocmds_retval()`: as [`fire`], but the event may turn `retval`
/// into `FAIL`.
pub(crate) fn fire_retval(event: event_T, mut buf: Buf, retval: &mut c_int) {
    let (none, raw) = (ptr::null_mut(), buf.raw());
    // SAFETY: a live buffer and a local to report through.
    unsafe { apply_autocmds_retval(event, none, none, false, raw, retval) };
}

pub(crate) fn block_autocmds_now() {
    // SAFETY: paired with `unblock_autocmds_now` by every caller.
    unsafe { block_autocmds() };
}

pub(crate) fn unblock_autocmds_now() {
    // SAFETY: paired with `block_autocmds_now` by every caller.
    unsafe { unblock_autocmds() };
}

/// Whether an error, interrupt or exception is unwinding the script.
pub(crate) fn aborting_now() -> bool {
    aborting()
}

/// `xfree()`.
pub(crate) fn free<T>(p: *mut T) {
    // SAFETY: an owned allocation or null.
    unsafe { xfree(p.cast::<c_void>()) };
}

/// `XFREE_CLEAR()` over a slot holding an owned allocation.
pub(crate) fn xfree_clear<T>(slot: &mut *mut T) {
    free(*slot);
    *slot = ptr::null_mut();
}

/// `ml_delete()` on the current buffer.
pub(crate) fn delete_line(lnum: linenr_T) {
    // SAFETY: the caller has checked the line is in the current buffer.
    unsafe { ml_delete(lnum) };
}

/// `unchanged()`: clear `'modified'`, and with `ff` the file-format flags.
pub(crate) fn unchanged_now(buf: Buf, ff: bool, always_inc_changedtick: bool) {
    unchanged(buf, ff, always_inc_changedtick);
}

pub(crate) fn end_visual() {
    end_visual_mode();
}

/// `close_windows()`: close every window showing `buf`.
///
/// Fires `WinClosed`/`BufWinLeave`; everything held may be stale afterwards.
pub(crate) fn close_all_windows(mut buf: Buf, keep_curwin: bool) {
    // SAFETY: a live buffer.
    unsafe { close_windows(buf.raw(), keep_curwin) };
}

/// Re-check `'colorcolumn'` after `'textwidth'` changed under the window.
pub(crate) fn recheck_colorcolumn(mut win: Win) {
    // SAFETY: a live window; a null pattern means "the option's own value".
    unsafe { check_colorcolumn(ptr::null_mut(), win.raw()) };
}

pub(crate) fn clear_window_folds(mut win: Win) {
    // SAFETY: a live window.
    clear_folding(win);
}

pub(crate) fn invalidate_window_folds(mut win: Win) {
    // SAFETY: a live window.
    fold_update_all(win);
}

/// Drop the window's own syntax state (`:ownsyntax`).
pub(crate) fn reset_syntax(mut win: Win) {
    // SAFETY: a live window.
    unsafe { reset_synblock(win.raw()) };
}

/// Remember the cursor position in the jump list.
pub(crate) fn set_pcmark() {
    // SAFETY: reads the current window and buffer, both set.
    setpcmark();
}

/// Whether `buf` has unsaved changes.
pub(crate) fn is_changed(mut buf: Buf) -> bool {
    // SAFETY: a live buffer.
    buf_is_changed(buf)
}

/// `semsg(fmt, n)`, for the three errors that name a buffer number.
pub(crate) fn err_num<T: crate::message_fmt::CArg>(fmt: *mut c_char, n: T) {
    // SAFETY: a translated format taking one number, and the number.
    let _: bool = unsafe { semsg_c!(fmt, n) };
}

/// `do_ecmd()`: edit `fname` (or buffer `fnum`) in `win`.
///
/// Re-enters the whole edit path; nothing held survives it.
pub(crate) fn edit_file(
    fnum: c_int,
    ffname: *mut c_char,
    sfname: *mut c_char,
    eap: *mut exarg_T,
    newlnum: linenr_T,
    flags: c_int,
    mut win: Win,
) -> c_int {
    let raw = win.raw();
    // SAFETY: a live window, and the caller's own arguments passed on.
    unsafe { do_ecmd(fnum, ffname, sfname, eap, newlnum, flags, raw) }
}

fn layout_lock() {
    window_layout_lock();
}

fn layout_unlock() {
    window_layout_unlock();
}

fn run_cmdline(cmd: &CStr) {
    // SAFETY: a NUL-terminated command line.
    unsafe { do_cmdline_cmd(cmd.as_ptr()) };
}

/// `buf_free_count++`: one more buffer has been freed, so every [`BufRef`]
/// taken before now has to walk the list to answer.
pub(crate) fn note_buffer_freed() {
    buf_free_count.set(buf_free_count.get() + 1);
}
