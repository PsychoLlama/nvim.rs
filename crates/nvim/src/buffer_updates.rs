//! The two lists of buffer-update subscribers, and the events sent to them.
//!
//! `nvim_buf_attach` records an RPC channel id in `buf->update_channels`
//! or, called from Lua, a table of callbacks in `buf->update_callbacks`.
//! Everything below walks one of those two `kvec_t`s: the channels get
//! `nvim_buf_{lines,changedtick,detach}_event` over RPC, the callbacks get
//! `on_lines` / `on_bytes` / `on_changedtick` / `on_reload` / `on_detach`.
//!
//! Every access is **momentary**, and that is load-bearing rather than tidy.
//! `nlua_call_ref` re-enters the editor: a callback may attach (which grows
//! the list being walked), detach, or edit the buffer and come back through
//! [`buf_updates_send_changes`] recursively (which truncates it). That is why
//! upstream re-reads `kv_size`/`kv_A` on every iteration instead of caching a
//! pointer, why the loops here are `while i < len()` rather than
//! `for i in 0..len`, and why [`SubscriberList::at`] indexes every slot ever
//! written rather than the live prefix — see its comment.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::narrow::len_as_integer;
use core::ffi::{CStr, c_int};

use crate::api::buffer::LineSink;
use crate::buffer::buf_get_changedtick;
use crate::ex_getln::state::CMDPREVIEW;
use crate::extmark::BCount;
use crate::guard::Lock;
use crate::log::{LOGLVL_ERR, logmsg};
use crate::lua::executor::LuaRetMode;
use crate::lua::executor::{nlua_call_ref_quiet, release_luaref};
use crate::memline::ml_flush_deleted_bytes;
use crate::msgpack_rpc::channel::rpc_send_event;
use crate::os::libc::{int64_t, size_t, uint64_t};
use crate::types::builders::ArrayBuf;
use crate::types::{Array, BufUpdateCallbacks, ColNr, Integer, LineNr, LuaRef, Object};
use crate::winlayer::{Buf, Win};

pub const LUA_NOREF: c_int = -2;
const INTERNAL_CALL_MASK: uint64_t = 1 << (uint64_t::BITS - 1);
const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL + 1;

// ---------------------------------------------------------------------------
// The two subscriber lists

/// One of a buffer's two subscriber lists — upstream's `kvec_t` — with the
/// one property of that layout the loops below lean on: shrinking the live
/// length forgets nothing. `slots` is every element ever written (the
/// allocation, as far as anything reads it); `len` is the live prefix.
#[derive(Debug)]
pub(crate) struct SubscriberList<T> {
    slots: Vec<T>,
    len: usize,
}

impl<T> Default for SubscriberList<T> {
    fn default() -> Self {
        SubscriberList {
            slots: Vec::new(),
            len: 0,
        }
    }
}

impl<T: Clone> SubscriberList<T> {
    /// `kv_size`.
    fn len(&self) -> usize {
        self.len
    }

    /// `kv_size(v) = n`, which upstream writes as a plain assignment when a
    /// compaction pass has finished.
    fn set_len(&mut self, len: usize) {
        assert!(
            len <= self.slots.len(),
            "subscriber list grown past its slots"
        );
        self.len = len;
    }

    /// The live prefix. Only for whole-list questions asked between
    /// callbacks — an element read that a callback could race wants
    /// [`SubscriberList::at`].
    fn as_slice(&self) -> &[T] {
        &self.slots[..self.len]
    }

    /// `kv_A`: element `i` of every slot written, not of the live prefix.
    ///
    /// Upstream bounds `kv_A` by nothing at all, and the difference is
    /// reachable: the compaction loops re-read `kv_size` each iteration, so
    /// a callback that detaches during `nlua_call_ref` shrinks it under
    /// them and leaves `i` and `j` past the new end but still inside the
    /// array — and still pointing at slots this buffer has written. Indexing
    /// the written slots keeps that case behaving as upstream does.
    fn at(&self, i: usize) -> T {
        self.slots[i].clone()
    }

    /// `kv_A(v, i) = x`. [`SubscriberList::at`]'s bound, for the same reason.
    /// The element types are plain records whose owned handles are released
    /// by `free_update_callbacks`, not by dropping, so overwriting a live
    /// slot releases nothing.
    fn set_at(&mut self, i: usize, value: T) {
        self.slots[i] = value;
    }

    /// `kv_push`: into the first slot past the live prefix.
    fn push(&mut self, value: T) {
        if self.len == self.slots.len() {
            self.slots.push(value);
        } else {
            self.slots[self.len] = value;
        }
        self.len += 1;
    }

    /// `kv_destroy`, which in this klib also re-inits the vector.
    fn destroy(&mut self) {
        *self = SubscriberList::default();
    }
}

/// What is watching a buffer: the RPC channels and the Lua callback tables.
/// A buffer holds one only once something attached (`Buffer` is allocated
/// zeroed, and a `None` box is what zeroed bytes are).
#[derive(Default, Debug)]
pub(crate) struct Subscribers {
    channels: SubscriberList<uint64_t>,
    callbacks: SubscriberList<BufUpdateCallbacks>,
}

impl Buf {
    /// This buffer's subscribers, created empty on first use.
    fn subscribers(&mut self) -> &mut Subscribers {
        self.update_subscribers.get_or_insert_with(Box::default)
    }

    /// The RPC channels watching this buffer.
    fn channels(&mut self) -> &mut SubscriberList<uint64_t> {
        &mut self.subscribers().channels
    }

    /// The Lua callback tables watching this buffer.
    fn callbacks(&mut self) -> &mut SubscriberList<BufUpdateCallbacks> {
        &mut self.subscribers().callbacks
    }
}

/// Drop every subscriber `buffer` has, as a buffer reused for another file
/// must: `kv_destroy` + `kv_init` of both lists.
pub(crate) fn reset_subscribers(mut buffer: Buf) {
    buffer.update_subscribers = None;
}

/// Whether a Lua callback is watching `buffer` — the only subscribers that
/// are told how many bytes an edit deleted.
pub(crate) fn has_callbacks(buffer: Buf) -> bool {
    buffer
        .update_subscribers
        .as_ref()
        .is_some_and(|subscribers| subscribers.callbacks.len() != 0)
}

// ---------------------------------------------------------------------------
// The calls out of the module

/// `b:changedtick`, as `buf_get_changedtick`.
fn changedtick(buffer: Buf) -> Integer {
    // SAFETY: a live buffer, which is [`Buf`]'s promise.
    buf_get_changedtick(buffer)
}

/// `rpc_send_event`, which only ever reads `args`.
fn send_event(channel_id: uint64_t, name: &'static CStr, args: Array) -> bool {
    rpc_send_event(channel_id, name, args)
}

/// C's `TEXTLOCK_WRAP`: run `f` with the cursor saved and restored and
/// `TEXTLOCK` held.
///
/// `curwin` is read twice on purpose. Upstream's macro expands
/// `curwin->w_cursor = save_cursor` *after* `code`, so a callback that
/// switched windows has the saved position written into whatever window it
/// left current — not into the one the position came from.
fn textlock_wrap<R>(f: impl FnOnce() -> R) -> R {
    let save_cursor = Win::current().w_cursor;
    let result = {
        let _locked = Lock::text();
        f()
    };
    let mut win = Win::current();
    win.w_cursor = save_cursor;
    result
}

/// One callback invocation, inside [`textlock_wrap`] as upstream has it.
fn call_ref(cb: LuaRef, name: &'static CStr, args: Array, mode: LuaRetMode) -> Object {
    // A null arena and error are what upstream passes -- the callee treats
    // both as "not interested".
    textlock_wrap(|| nlua_call_ref_quiet(cb, Some(name), args, mode))
}

/// C's `LUARET_TRUTHY`: a callback asking to be detached.
fn truthy(res: Object) -> bool {
    res.as_boolean() == Some(true)
}

/// Release the five Lua references one attachment holds.
fn callbacks_free(cb: BufUpdateCallbacks) {
    let refs = [
        cb.on_lines,
        cb.on_bytes,
        cb.on_changedtick,
        cb.on_reload,
        cb.on_detach,
    ];
    for ref_0 in refs {
        // A reference this value owns; `LUA_NOREF` is ignored.
        release_luaref(ref_0);
    }
}

/// C's `ELOG` for the one complaint this file makes.
///
/// The line number is upstream's `__LINE__` at the call, kept so the log
/// still names the C source everyone reads.
fn elog_dead_channel(channelid: uint64_t) {
    logmsg!(
        LOGLVL_ERR,
        c"buf_updates_send_changes",
        258,
        "Disabling buffer updates for dead channel {}",
        channelid
    );
}

/// What `ml_flush_deleted_bytes` reports.
#[derive(Debug)]
struct Deleted {
    bytes: size_t,
    codepoints: size_t,
    codeunits: size_t,
}

fn flush_deleted_bytes(buffer: Buf) -> Deleted {
    let (bytes, codepoints, codeunits) = ml_flush_deleted_bytes(buffer);
    Deleted {
        bytes,
        codepoints,
        codeunits,
    }
}

/// `linedata` for `nvim_buf_lines_event`: `n` lines from `first`.
fn collect_lines(buffer: Buf, n: size_t, first: LineNr) -> Array {
    let mut linedata = Vec::new();
    let mut lines = LineSink::new(None, &mut linedata, n);
    buffer.collect_lines(n, first, 0, true, &mut lines);
    linedata.into()
}

// ---------------------------------------------------------------------------
// Registering and unregistering

/// Attach `channel_id` (or, for `LUA_INTERNAL_CALL`, `cb`) to `buffer`.
///
/// True when the subscriber is watching afterwards, whether it was added
/// now or already there; false only when the buffer is not loaded.
pub fn buf_updates_register(
    buffer: Buf,
    channel_id: uint64_t,
    cb: BufUpdateCallbacks,
    send_buffer: bool,
) -> bool {
    register(buffer, channel_id, cb, send_buffer)
}

fn register(
    mut buffer: Buf,
    channel_id: uint64_t,
    cb: BufUpdateCallbacks,
    send_buffer: bool,
) -> bool {
    // Must fail if the buffer isn't loaded.
    if buffer.b_ml.ml_mfp.is_none() {
        return false;
    }

    if channel_id == LUA_INTERNAL_CALL {
        // No duplicate check and no `send_buffer` — the Lua path returns
        // before both. Attaching the same table twice really does mean two
        // subscriptions (each carries its own refs), and `nvim_buf_attach`
        // documents `send_buffer` as "Not for Lua callbacks".
        let utf_sizes = cb.utf_sizes;
        buffer.callbacks().push(cb);
        if utf_sizes {
            // Sticky: nothing clears it when the callback detaches, so the
            // buffer keeps counting codepoints for the rest of its life.
            buffer.update_need_codepoints = true;
        }
        return true;
    }

    // Already watching: nothing to do.
    if buffer.channels().as_slice().contains(&channel_id) {
        return true;
    }

    buffer.channels().push(channel_id);

    if send_buffer {
        send_whole_buffer(buffer, channel_id);
    } else {
        changedtick_single(buffer, channel_id);
    }

    true
}

/// The `nvim_buf_lines_event` a channel attaching with `send_buffer` gets:
/// the whole buffer as one replacement of the range `0 .. -1`.
fn send_whole_buffer(buffer: Buf, channel_id: uint64_t) {
    let line_count = usize::try_from(buffer.line_count()).unwrap_or(0);
    let mut linedata = Array::new();
    if line_count > 0 {
        linedata = collect_lines(buffer, line_count, LineNr::ONE);
    }

    let mut args = ArrayBuf::<6>::new();
    args.push(Object::buffer(buffer.handle));
    args.push(Object::integer(changedtick(buffer)));
    // The first line that changed (zero-indexed), then the last.
    args.push(Object::integer(0));
    args.push(Object::integer(-1));
    args.push(Object::array(linedata));
    args.push(Object::boolean(false));
    send_event(channel_id, c"nvim_buf_lines_event", args.array());
}

/// Whether anything is watching `buffer`.
pub fn buf_updates_active(buffer: Buf) -> bool {
    active(buffer)
}

fn active(buffer: Buf) -> bool {
    buffer
        .update_subscribers
        .as_ref()
        .is_some_and(|subscribers| {
            subscribers.channels.len() != 0 || subscribers.callbacks.len() != 0
        })
}

/// Tell one channel it is no longer attached.
pub fn buf_updates_send_end(buffer: Buf, channelid: uint64_t) {
    send_end(buffer, channelid);
}

fn send_end(buffer: Buf, channelid: uint64_t) {
    let mut args = ArrayBuf::<1>::new();
    args.push(Object::buffer(buffer.handle));
    send_event(channelid, c"nvim_buf_detach_event", args.array());
}

/// Detach `channelid` from `buffer`, if it is attached.
pub fn buf_updates_unregister(buffer: Buf, channelid: uint64_t) {
    unregister(buffer, channelid);
}

fn unregister(mut buffer: Buf, channelid: uint64_t) {
    if buffer.update_subscribers.is_none() {
        return;
    }
    let size = buffer.channels().len();
    if size == 0 {
        return;
    }

    // Compact the id out of the list — it should never appear more than
    // once, but upstream counts rather than assuming.
    let (mut j, mut found) = (0, 0);
    let channels = buffer.channels();
    for i in 0..size {
        if channels.at(i) == channelid {
            found += 1;
        } else {
            if i != j {
                channels.set_at(j, channels.at(i));
            }
            j += 1;
        }
    }

    if found != 0 {
        // Remove `found` items from the end of the array.
        buffer.channels().set_len(size - found);
        // Upstream tells the channel *before* releasing the array, and the
        // order is kept: `rpc_send_event` reads only the buffer handle.
        send_end(buffer, channelid);
        if found == size {
            buffer.channels().destroy();
        }
    }
}

/// Drop everything watching `buffer`, silently: the buffer itself is going
/// away, so nobody is told.
pub fn buf_free_callbacks(buffer: Buf) {
    free_callbacks(buffer);
}

fn free_callbacks(mut buffer: Buf) {
    let Some(subscribers) = buffer.update_subscribers.take() else {
        return;
    };
    for cb in subscribers.callbacks.as_slice() {
        callbacks_free(cb.clone());
    }
}

/// The buffer's contents are gone: detach every channel, and give every
/// callback its `on_reload` (when the contents are coming back) or its
/// `on_detach` (when they are not).
pub fn buf_updates_unload(buffer: Buf, can_reload: bool) {
    unload(buffer, can_reload);
}

fn unload(mut buffer: Buf, can_reload: bool) {
    if buffer.update_subscribers.is_none() {
        return;
    }
    let size = buffer.channels().len();
    if size != 0 {
        for i in 0..size {
            let channelid = buffer.channels().at(i);
            send_end(buffer, channelid);
        }
        buffer.channels().destroy();
    }

    let mut j = 0;
    let mut i = 0;
    while i < buffer.callbacks().len() {
        let cb = buffer.callbacks().at(i);
        let mut thecb = LUA_NOREF;

        let mut keep = false;
        if can_reload && cb.on_reload != LUA_NOREF {
            keep = true;
            thecb = cb.on_reload;
        } else if cb.on_detach != LUA_NOREF {
            thecb = cb.on_detach;
        }

        if thecb != LUA_NOREF {
            let mut args = ArrayBuf::<1>::new();
            args.push(Object::buffer(buffer.handle));
            let name = if keep { c"reload" } else { c"detach" };
            // Upstream discards the result here: a reload callback cannot
            // detach itself the way `on_lines` can.
            call_ref(thecb, name, args.array(), LuaRetMode::Object);
        }

        if keep {
            let moved = buffer.callbacks().at(i);
            buffer.callbacks().set_at(j, moved);
            j += 1;
        } else {
            callbacks_free(cb);
        }
        i += 1;
    }
    buffer.callbacks().set_len(j);
    if buffer.callbacks().len() == 0 {
        buffer.callbacks().destroy();
    }
}

// ---------------------------------------------------------------------------
// The events

/// `num_added` lines replaced `num_removed` lines starting at `firstline`.
pub fn buf_updates_send_changes(
    buffer: Buf,
    firstline: LineNr,
    num_added: int64_t,
    num_removed: int64_t,
) {
    send_changes(buffer, firstline, num_added, num_removed);
}

fn send_changes(mut buffer: Buf, firstline: LineNr, num_added: int64_t, num_removed: int64_t) {
    let deleted = flush_deleted_bytes(buffer);

    if !active(buffer) {
        return;
    }

    // Don't send b:changedtick during 'inccommand' preview if "buf" is the
    // current buffer.
    let send_tick = !(CMDPREVIEW.get() && buffer.raw() == Buf::current_raw());

    // If one of the channels doesn't work, put its ID here so we can remove
    // it later.
    let mut badchannelid = 0;

    let mut linedata = Array::new();
    if num_added > 0 && buffer.channels().len() != 0 {
        let n = usize::try_from(num_added).expect("tested positive");
        linedata = collect_lines(buffer, n, firstline);
    }

    // Notify each of the active channels.
    let mut i = 0;
    while i < buffer.channels().len() {
        let channelid = buffer.channels().at(i);
        let mut args = ArrayBuf::<6>::new();
        args.push(Object::buffer(buffer.handle));
        args.push(tick_obj(buffer, send_tick));
        // The first line that changed (zero-indexed), then the last.
        args.push(Object::integer(i64::from(firstline - 1)));
        args.push(Object::integer(i64::from(firstline - 1) + num_removed));
        // Linedata of the lines being swapped in.
        // One copy per channel: the event takes over what it is sent.
        args.push(Object::array(linedata.clone()));
        args.push(Object::boolean(false));
        if !send_event(channelid, c"nvim_buf_lines_event", args.array()) {
            // The channel can't be unregistered while this loop is walking
            // the array, so remember it and do it at the end.
            badchannelid = channelid;
        }
        i += 1;
    }

    // Only one dead channel goes per call. That is fine: the notifications
    // are frequent enough that a pile of them clears quickly.
    if badchannelid != 0 {
        elog_dead_channel(badchannelid);
        unregister(buffer, badchannelid);
    }

    // Notify each of the active callbacks.
    let mut j = 0;
    let mut i = 0;
    while i < buffer.callbacks().len() {
        let cb = buffer.callbacks().at(i);
        let mut keep = true;
        if cb.on_lines != LUA_NOREF && (cb.preview || !CMDPREVIEW.get()) {
            // Six arguments, or eight with the UTF sizes.
            let mut args = ArrayBuf::<8>::new();
            args.push(Object::buffer(buffer.handle));
            args.push(tick_obj(buffer, send_tick));
            // First changed line, last changed line, last line of the new
            // range, then the byte count of the previous contents.
            args.push(Object::integer(i64::from(firstline - 1)));
            args.push(Object::integer(i64::from(firstline - 1) + num_removed));
            args.push(Object::integer(i64::from(firstline - 1) + num_added));
            args.push(Object::integer(len_as_integer(deleted.bytes)));
            if cb.utf_sizes {
                args.push(Object::integer(len_as_integer(deleted.codepoints)));
                args.push(Object::integer(len_as_integer(deleted.codeunits)));
            }
            let res = call_ref(cb.on_lines, c"lines", args.array(), LuaRetMode::NilBool);
            if truthy(res) {
                callbacks_free(cb);
                keep = false;
            }
        }
        if keep {
            let moved = buffer.callbacks().at(i);
            buffer.callbacks().set_at(j, moved);
            j += 1;
        }
        i += 1;
    }
    buffer.callbacks().set_len(j);
}

/// `b:changedtick` when it is being sent, nil when 'inccommand' preview is
/// suppressing it.
fn tick_obj(buffer: Buf, send_tick: bool) -> Object {
    if send_tick {
        Object::integer(changedtick(buffer))
    } else {
        Object::Nil
    }
}

/// A byte-level edit: `old_*` bytes at `start_*` became `new_*` bytes.
/// Callbacks only — no RPC event carries this.
pub fn buf_updates_send_splice(
    buffer: Buf,
    start_row: c_int,
    start_col: ColNr,
    start_byte: BCount,
    old_row: c_int,
    old_col: ColNr,
    old_byte: BCount,
    new_row: c_int,
    new_col: ColNr,
    new_byte: BCount,
) {
    let start = Corner::new(start_row, start_col, start_byte);
    let old = Corner::new(old_row, old_col, old_byte);
    let new = Corner::new(new_row, new_col, new_byte);
    send_splice(buffer, start, old, new);
}

/// One corner of a splice, as `on_bytes` reports it: a row, a column, and a
/// byte offset. Upstream spells the same three numbers as nine separate
/// parameters.
#[derive(Clone, Copy, Debug)]
struct Corner {
    row: c_int,
    col: ColNr,
    byte: BCount,
}

impl Corner {
    fn new(row: c_int, col: ColNr, byte: BCount) -> Self {
        Self { row, col, byte }
    }
}

fn send_splice(mut buffer: Buf, start: Corner, old: Corner, new: Corner) {
    if !active(buffer) || (old.byte == 0 && new.byte == 0) {
        return;
    }

    // Notify each of the active callbacks.
    let mut j = 0;
    let mut i = 0;
    while i < buffer.callbacks().len() {
        let cb = buffer.callbacks().at(i);
        let mut keep = true;
        if cb.on_bytes != LUA_NOREF && (cb.preview || !CMDPREVIEW.get()) {
            let mut args = ArrayBuf::<11>::new();
            args.push(Object::buffer(buffer.handle));
            args.push(Object::integer(changedtick(buffer)));
            for corner in [start, old, new] {
                args.push(Object::integer(Integer::from(corner.row)));
                args.push(Object::integer(i64::from(corner.col)));
                args.push(Object::integer(corner.byte as Integer));
            }
            let res = call_ref(cb.on_bytes, c"bytes", args.array(), LuaRetMode::NilBool);
            if truthy(res) {
                callbacks_free(cb);
                keep = false;
            }
        }
        if keep {
            let moved = buffer.callbacks().at(i);
            buffer.callbacks().set_at(j, moved);
            j += 1;
        }
        i += 1;
    }
    buffer.callbacks().set_len(j);
}

/// `b:changedtick` moved without the text moving.
///
/// The only event with no `cb.preview || !cmdpreview` guard on its
/// callback, and it does not need one: its single caller is `u_undoredo`,
/// and the 'inccommand' undo reaches that through `u_undo_and_forget(count,
/// false)`, which suppresses the event outright.
pub fn buf_updates_changedtick(buffer: Buf) {
    changedtick_event(buffer);
}

fn changedtick_event(mut buffer: Buf) {
    // Notify each of the active channels.
    let mut i = 0;
    while i < buffer.channels().len() {
        let channel_id = buffer.channels().at(i);
        changedtick_single(buffer, channel_id);
        i += 1;
    }

    let mut j = 0;
    let mut i = 0;
    while i < buffer.callbacks().len() {
        let cb = buffer.callbacks().at(i);
        let mut keep = true;
        if cb.on_changedtick != LUA_NOREF {
            let mut args = ArrayBuf::<2>::new();
            args.push(Object::buffer(buffer.handle));
            args.push(Object::integer(changedtick(buffer)));
            let res = call_ref(
                cb.on_changedtick,
                c"changedtick",
                args.array(),
                LuaRetMode::NilBool,
            );
            if truthy(res) {
                callbacks_free(cb);
                keep = false;
            }
        }
        if keep {
            let moved = buffer.callbacks().at(i);
            buffer.callbacks().set_at(j, moved);
            j += 1;
        }
        i += 1;
    }
    buffer.callbacks().set_len(j);
}

/// `nvim_buf_changedtick_event` for one channel.
pub fn buf_updates_changedtick_single(buffer: Buf, channel_id: uint64_t) {
    changedtick_single(buffer, channel_id);
}

fn changedtick_single(buffer: Buf, channel_id: uint64_t) {
    let mut args = ArrayBuf::<2>::new();
    args.push(Object::buffer(buffer.handle));
    args.push(Object::integer(changedtick(buffer)));
    // Don't try and clean up dead channels here.
    send_event(channel_id, c"nvim_buf_changedtick_event", args.array());
}

/// Release one attachment's Lua references.
pub fn buffer_update_callbacks_free(cb: BufUpdateCallbacks) {
    callbacks_free(cb);
}
