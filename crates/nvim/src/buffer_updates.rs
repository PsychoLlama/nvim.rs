//! The two lists of buffer-update subscribers, and the events sent to them.
//!
//! `nvim_buf_attach` records an RPC channel id in `buf->update_channels`
//! or, called from Lua, a table of callbacks in `buf->update_callbacks`.
//! Everything below walks one of those two `kvec_t`s: the channels get
//! `nvim_buf_{lines,changedtick,detach}_event` over RPC, the callbacks get
//! `on_lines` / `on_bytes` / `on_changedtick` / `on_reload` / `on_detach`.
//!
//! [`KVec`] is the lever. Both arrays are fields of `buf_T`, so borrowing
//! their three parts is a safe operation once the buffer pointer is wrapped
//! as a [`Buf`], and everything above it — the loops, the compaction, the
//! argument building — is ordinary checked code.
//!
//! Every view is **momentary**, and that is load-bearing rather than tidy.
//! `nlua_call_ref` re-enters the editor: a callback may attach (which
//! reallocates the array being walked), detach, or edit the buffer and come
//! back through [`buf_updates_send_changes`] recursively (which truncates
//! it). That is why upstream re-reads `kv_size`/`kv_A` on every iteration
//! instead of caching a pointer, why the loops here are `while i < len()`
//! rather than `for i in 0..len`, and why [`KVec::at`] indexes the
//! allocation rather than the live prefix — see its comment.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_int};
use core::{ptr, slice};

use crate::api::buffer::buf_collect_lines;
use crate::api::private::helpers::arena_array;
use crate::buffer::buf_get_changedtick;
use crate::guard::Lock;
use crate::log::{LOGLVL_ERR, logmsg};
use crate::lua::executor::{api_free_luaref, nlua_call_ref_quiet};
use crate::main::{cmdpreview, curbuf};
use crate::memline::ml_flush_deleted_bytes;
use crate::memory::{ARENA_EMPTY, arena_finish, arena_mem_free, xfree, xrealloc};
use crate::msgpack_rpc::channel::rpc_send_event;
use crate::types::builders::ArrayBuf;
use crate::types::{
    Arena, Array, BufUpdateCallbacks, Integer, LuaRef, LuaRetMode, Object, bcount_t, buf_T,
    colnr_T, int64_t, linenr_T, size_t, uint64_t,
};
use crate::winlayer::{Buf, Win};

pub const kRetObject: LuaRetMode = 0;
pub const kRetNilBool: LuaRetMode = 1;
pub const LUA_NOREF: c_int = -2;
const INTERNAL_CALL_MASK: uint64_t = 1 << (uint64_t::BITS - 1);
const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL + 1;

// ---------------------------------------------------------------------------
// The two `kvec_t`s

/// One of `buf_T`'s two subscriber arrays — `klib/kvec.h`'s growable vector
/// — borrowed field by field, so that only the element access below is
/// unchecked.
///
/// Short-lived by construction: the borrow of the buffer ends with the
/// expression that took it, which is what keeps a re-entrant callback from
/// running while a `&mut` into the array is outstanding.
struct KVec<'a, T> {
    size: &'a mut size_t,
    capacity: &'a mut size_t,
    items: &'a mut *mut T,
}

impl<T: Clone> KVec<'_, T> {
    /// `kv_size`.
    fn len(&self) -> size_t {
        *self.size
    }

    /// `kv_size(v) = n`, which upstream writes as a plain assignment when a
    /// compaction pass has finished.
    fn set_len(&mut self, len: size_t) {
        *self.size = len;
    }

    /// The live prefix. Only for whole-array questions asked between
    /// callbacks — an element read that a callback could race wants
    /// [`KVec::at`].
    fn as_slice(&self) -> &[T] {
        if *self.size == 0 {
            return &[];
        }
        // SAFETY: a kvec's first `size` elements are initialised, and
        // `items` is non-null once anything has been pushed.
        unsafe { slice::from_raw_parts(*self.items, *self.size) }
    }

    /// `kv_A`: element `i` of the *allocation*, not of the live prefix.
    ///
    /// Upstream bounds `kv_A` by nothing at all, and the difference is
    /// reachable: the compaction loops re-read `kv_size` each iteration, so
    /// a callback that detaches during `nlua_call_ref` shrinks it under
    /// them and leaves `i` and `j` past the new end but still inside the
    /// array — and still pointing at slots this buffer has written. Bound
    /// by `capacity` to keep that case behaving as upstream does.
    fn at(&self, i: size_t) -> T {
        assert!(i < *self.capacity, "kvec index past the allocation");
        // SAFETY: `i` is inside the allocation, and every index these loops
        // reach was written by a `push` before `size` ever passed it.
        unsafe { (*self.items.add(i)).clone() }
    }

    /// `kv_A(v, i) = x`. [`KVec::at`]'s bound, for the same reason.
    fn set_at(&mut self, i: size_t, value: T) {
        assert!(i < *self.capacity, "kvec index past the allocation");
        // SAFETY: as [`KVec::at`], and `push` also reaches the first
        // uninitialised slot past the live prefix. `write` is what covers
        // both: it never reads what is already there. The element types are
        // plain records whose owned handles are released by
        // `free_update_callbacks`, not by dropping the array, so overwriting
        // a live slot releases nothing either way.
        unsafe { self.items.add(i).write(value) };
    }

    /// `kv_push`.
    fn push(&mut self, value: T) {
        if *self.size == *self.capacity {
            *self.capacity = if *self.capacity != 0 {
                *self.capacity << 1
            } else {
                8
            };
            let bytes = size_of::<T>() * *self.capacity;
            let old = self.items.cast::<::core::ffi::c_void>();
            // SAFETY: `items` is null or this array's own allocation, and
            // the new size counts the same element type.
            *self.items = unsafe { xrealloc(old, bytes) }.cast::<T>();
        }
        let end = *self.size;
        *self.size = end + 1;
        self.set_at(end, value);
    }

    /// `kv_destroy`, which in this klib also re-inits the vector.
    fn destroy(&mut self) {
        // SAFETY: `items` is null or this array's own allocation, and the
        // three fields are reset before anything can read them again.
        unsafe { xfree(self.items.cast::<::core::ffi::c_void>()) };
        *self.size = 0;
        *self.capacity = 0;
        *self.items = ptr::null_mut();
    }
}

impl Buf {
    /// The RPC channels watching this buffer.
    fn channels(&mut self) -> KVec<'_, uint64_t> {
        let kv = &mut self.update_channels;
        KVec {
            size: &mut kv.size,
            capacity: &mut kv.capacity,
            items: &mut kv.items,
        }
    }

    /// The Lua callback tables watching this buffer.
    fn callbacks(&mut self) -> KVec<'_, BufUpdateCallbacks> {
        let kv = &mut self.update_callbacks;
        KVec {
            size: &mut kv.size,
            capacity: &mut kv.capacity,
            items: &mut kv.items,
        }
    }
}

// ---------------------------------------------------------------------------
// The calls out of the module

/// `b:changedtick`, as `buf_get_changedtick`.
fn changedtick(buf: Buf) -> Integer {
    // SAFETY: a live buffer, which is [`Buf`]'s promise.
    buf_get_changedtick(buf)
}

/// `rpc_send_event`, which only ever reads `args`.
fn send_event(channel_id: uint64_t, name: &'static CStr, args: Array) -> bool {
    // SAFETY: a NUL-terminated event name and an array borrowing the
    // caller's frame, which the callee serialises and returns.
    unsafe { rpc_send_event(channel_id, name.as_ptr(), args) }
}

/// C's `TEXTLOCK_WRAP`: run `f` with the cursor saved and restored and
/// `textlock` held.
///
/// `curwin` is read twice on purpose. Upstream's macro expands
/// `curwin->w_cursor = save_cursor` *after* `code`, so a callback that
/// switched windows has the saved position written into whatever window it
/// left current — not into the one the position came from.
fn textlock_wrap<R>(f: impl FnOnce() -> R) -> R {
    // SAFETY: `curwin` is set from startup to exit.
    let save_cursor = unsafe { Win::current() }.w_cursor;
    let result = {
        let _locked = Lock::text();
        f()
    };
    // SAFETY: `curwin` is set from startup to exit.
    let mut win = unsafe { Win::current() };
    win.w_cursor = save_cursor;
    result
}

/// One callback invocation, inside [`textlock_wrap`] as upstream has it.
fn call_ref(cb: LuaRef, name: &'static CStr, args: Array, mode: LuaRetMode) -> Object {
    textlock_wrap(|| {
        let no_arena = ptr::null_mut();
        // SAFETY: `cb` is a reference this buffer owns, `args` borrows the
        // caller's frame, and a null arena and error are what upstream
        // passes — the callee treats both as "not interested".
        unsafe { nlua_call_ref_quiet(cb, name.as_ptr(), args, mode, no_arena) }
    })
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
        // SAFETY: a reference this value owns; the callee ignores
        // `LUA_NOREF`.
        unsafe { api_free_luaref(ref_0) };
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

/// What `ml_flush_deleted_bytes` reports through three out-parameters.
struct Deleted {
    bytes: size_t,
    codepoints: size_t,
    codeunits: size_t,
}

fn flush_deleted_bytes(buf: Buf) -> Deleted {
    let (mut codepoints, mut codeunits) = (0, 0);
    let (cp, cu) = (&raw mut codepoints, &raw mut codeunits);
    // SAFETY: a live buffer and two live out-parameters.
    let bytes = unsafe { ml_flush_deleted_bytes(buf.raw(), cp, cu) };
    Deleted {
        bytes,
        codepoints,
        codeunits,
    }
}

/// `linedata` for `nvim_buf_lines_event`: `n` lines from `first`, allocated
/// in `arena`.
fn collect_lines(buf: Buf, n: size_t, first: linenr_T, arena: &mut Arena) -> Array {
    let ar = &raw mut *arena;
    let mut linedata = arena_array(ar, n);
    let (b, out, none) = (buf.raw(), &raw mut linedata, ptr::null_mut());
    // SAFETY: a live buffer holding lines `first ..= first + n - 1`, and an
    // array of `n` slots in the same arena the callee fills from.
    unsafe { buf_collect_lines(b, n, first, 0, true, out, none, ar) };
    linedata
}

// ---------------------------------------------------------------------------
// Registering and unregistering

/// Attach `channel_id` (or, for `LUA_INTERNAL_CALL`, `cb`) to `buf`.
///
/// True when the subscriber is watching afterwards, whether it was added
/// now or already there; false only when the buffer is not loaded.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn buf_updates_register(
    buf: *mut buf_T,
    channel_id: uint64_t,
    cb: BufUpdateCallbacks,
    send_buffer: bool,
) -> bool {
    // SAFETY: the caller's promise.
    register(unsafe { Buf::new(buf) }, channel_id, cb, send_buffer)
}

fn register(mut buf: Buf, channel_id: uint64_t, cb: BufUpdateCallbacks, send_buffer: bool) -> bool {
    // Must fail if the buffer isn't loaded.
    if buf.b_ml.ml_mfp.is_null() {
        return false;
    }

    if channel_id == LUA_INTERNAL_CALL {
        // No duplicate check and no `send_buffer` — the Lua path returns
        // before both. Attaching the same table twice really does mean two
        // subscriptions (each carries its own refs), and `nvim_buf_attach`
        // documents `send_buffer` as "Not for Lua callbacks".
        let utf_sizes = cb.utf_sizes;
        buf.callbacks().push(cb);
        if utf_sizes {
            // Sticky: nothing clears it when the callback detaches, so the
            // buffer keeps counting codepoints for the rest of its life.
            buf.update_need_codepoints = true;
        }
        return true;
    }

    // Already watching: nothing to do.
    if buf.channels().as_slice().contains(&channel_id) {
        return true;
    }

    buf.channels().push(channel_id);

    if send_buffer {
        send_whole_buffer(buf, channel_id);
    } else {
        changedtick_single(buf, channel_id);
    }

    true
}

/// The `nvim_buf_lines_event` a channel attaching with `send_buffer` gets:
/// the whole buffer as one replacement of the range `0 .. -1`.
fn send_whole_buffer(buf: Buf, channel_id: uint64_t) {
    let line_count = buf.line_count() as size_t;
    let mut arena = ARENA_EMPTY;
    let mut linedata = Array::EMPTY;
    if line_count > 0 {
        linedata = collect_lines(buf, line_count, 1, &mut arena);
    }

    let mut args = ArrayBuf::<6>::new();
    args.push(Object::buffer(buf.handle));
    args.push(Object::integer(changedtick(buf)));
    // The first line that changed (zero-indexed), then the last.
    args.push(Object::integer(0));
    args.push(Object::integer(-1));
    args.push(Object::array(linedata));
    args.push(Object::boolean(false));
    send_event(channel_id, c"nvim_buf_lines_event", args.array());

    // SAFETY: the arena is this frame's, and `linedata` is not read again.
    unsafe { arena_mem_free(arena_finish(&raw mut arena)) };
}

/// Whether anything is watching `buf`.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn buf_updates_active(buf: *mut buf_T) -> bool {
    // SAFETY: the caller's promise.
    active(unsafe { Buf::new(buf) })
}

fn active(mut buf: Buf) -> bool {
    buf.channels().len() != 0 || buf.callbacks().len() != 0
}

/// Tell one channel it is no longer attached.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn buf_updates_send_end(buf: *mut buf_T, channelid: uint64_t) {
    // SAFETY: the caller's promise.
    send_end(unsafe { Buf::new(buf) }, channelid);
}

fn send_end(buf: Buf, channelid: uint64_t) {
    let mut args = ArrayBuf::<1>::new();
    args.push(Object::buffer(buf.handle));
    send_event(channelid, c"nvim_buf_detach_event", args.array());
}

/// Detach `channelid` from `buf`, if it is attached.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn buf_updates_unregister(buf: *mut buf_T, channelid: uint64_t) {
    // SAFETY: the caller's promise.
    unregister(unsafe { Buf::new(buf) }, channelid);
}

fn unregister(mut buf: Buf, channelid: uint64_t) {
    let size = buf.channels().len();
    if size == 0 {
        return;
    }

    // Compact the id out of the list — it should never appear more than
    // once, but upstream counts rather than assuming.
    let (mut j, mut found) = (0, 0);
    let mut channels = buf.channels();
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
        buf.channels().set_len(size - found);
        // Upstream tells the channel *before* releasing the array, and the
        // order is kept: `rpc_send_event` reads only the buffer handle.
        send_end(buf, channelid);
        if found == size {
            buf.channels().destroy();
        }
    }
}

/// Drop everything watching `buf`, silently: the buffer itself is going
/// away, so nobody is told.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn buf_free_callbacks(buf: *mut buf_T) {
    // SAFETY: the caller's promise.
    free_callbacks(unsafe { Buf::new(buf) });
}

fn free_callbacks(mut buf: Buf) {
    buf.channels().destroy();
    let mut i = 0;
    while i < buf.callbacks().len() {
        callbacks_free(buf.callbacks().at(i));
        i += 1;
    }
    buf.callbacks().destroy();
}

/// The buffer's contents are gone: detach every channel, and give every
/// callback its `on_reload` (when the contents are coming back) or its
/// `on_detach` (when they are not).
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn buf_updates_unload(buf: *mut buf_T, can_reload: bool) {
    // SAFETY: the caller's promise.
    unload(unsafe { Buf::new(buf) }, can_reload);
}

fn unload(mut buf: Buf, can_reload: bool) {
    let size = buf.channels().len();
    if size != 0 {
        for i in 0..size {
            let channelid = buf.channels().at(i);
            send_end(buf, channelid);
        }
        buf.channels().destroy();
    }

    let mut j = 0;
    let mut i = 0;
    while i < buf.callbacks().len() {
        let cb = buf.callbacks().at(i);
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
            args.push(Object::buffer(buf.handle));
            let name = if keep { c"reload" } else { c"detach" };
            // Upstream discards the result here: a reload callback cannot
            // detach itself the way `on_lines` can.
            call_ref(thecb, name, args.array(), kRetObject);
        }

        if keep {
            let moved = buf.callbacks().at(i);
            buf.callbacks().set_at(j, moved);
            j += 1;
        } else {
            callbacks_free(cb);
        }
        i += 1;
    }
    buf.callbacks().set_len(j);
    if buf.callbacks().len() == 0 {
        buf.callbacks().destroy();
    }
}

// ---------------------------------------------------------------------------
// The events

/// `num_added` lines replaced `num_removed` lines starting at `firstline`.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn buf_updates_send_changes(
    buf: *mut buf_T,
    firstline: linenr_T,
    num_added: int64_t,
    num_removed: int64_t,
) {
    // SAFETY: the caller's promise.
    let buf = unsafe { Buf::new(buf) };
    send_changes(buf, firstline, num_added, num_removed);
}

fn send_changes(mut buf: Buf, firstline: linenr_T, num_added: int64_t, num_removed: int64_t) {
    let deleted = flush_deleted_bytes(buf);

    if !active(buf) {
        return;
    }

    // Don't send b:changedtick during 'inccommand' preview if "buf" is the
    // current buffer.
    let send_tick = !(cmdpreview.get() && buf.raw() == curbuf.get());

    // If one of the channels doesn't work, put its ID here so we can remove
    // it later.
    let mut badchannelid = 0;

    let mut arena = ARENA_EMPTY;
    let mut linedata = Array::EMPTY;
    if num_added > 0 && buf.channels().len() != 0 {
        let n = num_added as size_t;
        linedata = collect_lines(buf, n, firstline, &mut arena);
    }

    // Notify each of the active channels.
    let mut i = 0;
    while i < buf.channels().len() {
        let channelid = buf.channels().at(i);
        let mut args = ArrayBuf::<6>::new();
        args.push(Object::buffer(buf.handle));
        args.push(tick_obj(buf, send_tick));
        // The first line that changed (zero-indexed), then the last.
        args.push(Object::integer((firstline - 1) as Integer));
        args.push(Object::integer((firstline - 1) as int64_t + num_removed));
        // Linedata of the lines being swapped in.
        args.push(Object::array(linedata));
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
        unregister(buf, badchannelid);
    }

    // The callbacks don't use linedata.
    // SAFETY: the arena is this frame's, and `linedata` is not read again.
    unsafe { arena_mem_free(arena_finish(&raw mut arena)) };

    // Notify each of the active callbacks.
    let mut j = 0;
    let mut i = 0;
    while i < buf.callbacks().len() {
        let cb = buf.callbacks().at(i);
        let mut keep = true;
        if cb.on_lines != LUA_NOREF && (cb.preview || !cmdpreview.get()) {
            // Six arguments, or eight with the UTF sizes.
            let mut args = ArrayBuf::<8>::new();
            args.push(Object::buffer(buf.handle));
            args.push(tick_obj(buf, send_tick));
            // First changed line, last changed line, last line of the new
            // range, then the byte count of the previous contents.
            args.push(Object::integer((firstline - 1) as Integer));
            args.push(Object::integer((firstline - 1) as int64_t + num_removed));
            args.push(Object::integer((firstline - 1) as int64_t + num_added));
            args.push(Object::integer(deleted.bytes as Integer));
            if cb.utf_sizes {
                args.push(Object::integer(deleted.codepoints as Integer));
                args.push(Object::integer(deleted.codeunits as Integer));
            }
            let res = call_ref(cb.on_lines, c"lines", args.array(), kRetNilBool);
            if truthy(res) {
                callbacks_free(cb);
                keep = false;
            }
        }
        if keep {
            let moved = buf.callbacks().at(i);
            buf.callbacks().set_at(j, moved);
            j += 1;
        }
        i += 1;
    }
    buf.callbacks().set_len(j);
}

/// `b:changedtick` when it is being sent, nil when 'inccommand' preview is
/// suppressing it.
fn tick_obj(buf: Buf, send_tick: bool) -> Object {
    if send_tick {
        Object::integer(changedtick(buf))
    } else {
        Object::Nil
    }
}

/// A byte-level edit: `old_*` bytes at `start_*` became `new_*` bytes.
/// Callbacks only — no RPC event carries this.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn buf_updates_send_splice(
    buf: *mut buf_T,
    start_row: c_int,
    start_col: colnr_T,
    start_byte: bcount_t,
    old_row: c_int,
    old_col: colnr_T,
    old_byte: bcount_t,
    new_row: c_int,
    new_col: colnr_T,
    new_byte: bcount_t,
) {
    // SAFETY: the caller's promise.
    let buf = unsafe { Buf::new(buf) };
    let start = Corner::new(start_row, start_col, start_byte);
    let old = Corner::new(old_row, old_col, old_byte);
    let new = Corner::new(new_row, new_col, new_byte);
    send_splice(buf, start, old, new);
}

/// One corner of a splice, as `on_bytes` reports it: a row, a column, and a
/// byte offset. Upstream spells the same three numbers as nine separate
/// parameters.
#[derive(Clone, Copy)]
struct Corner {
    row: c_int,
    col: colnr_T,
    byte: bcount_t,
}

impl Corner {
    fn new(row: c_int, col: colnr_T, byte: bcount_t) -> Self {
        Self { row, col, byte }
    }
}

fn send_splice(mut buf: Buf, start: Corner, old: Corner, new: Corner) {
    if !active(buf) || (old.byte == 0 && new.byte == 0) {
        return;
    }

    // Notify each of the active callbacks.
    let mut j = 0;
    let mut i = 0;
    while i < buf.callbacks().len() {
        let cb = buf.callbacks().at(i);
        let mut keep = true;
        if cb.on_bytes != LUA_NOREF && (cb.preview || !cmdpreview.get()) {
            let mut args = ArrayBuf::<11>::new();
            args.push(Object::buffer(buf.handle));
            args.push(Object::integer(changedtick(buf)));
            for corner in [start, old, new] {
                args.push(Object::integer(corner.row as Integer));
                args.push(Object::integer(corner.col as Integer));
                args.push(Object::integer(corner.byte as Integer));
            }
            let res = call_ref(cb.on_bytes, c"bytes", args.array(), kRetNilBool);
            if truthy(res) {
                callbacks_free(cb);
                keep = false;
            }
        }
        if keep {
            let moved = buf.callbacks().at(i);
            buf.callbacks().set_at(j, moved);
            j += 1;
        }
        i += 1;
    }
    buf.callbacks().set_len(j);
}

/// `b:changedtick` moved without the text moving.
///
/// The only event with no `cb.preview || !cmdpreview` guard on its
/// callback, and it does not need one: its single caller is `u_undoredo`,
/// and the 'inccommand' undo reaches that through `u_undo_and_forget(count,
/// false)`, which suppresses the event outright.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn buf_updates_changedtick(buf: *mut buf_T) {
    // SAFETY: the caller's promise.
    changedtick_event(unsafe { Buf::new(buf) });
}

fn changedtick_event(mut buf: Buf) {
    // Notify each of the active channels.
    let mut i = 0;
    while i < buf.channels().len() {
        let channel_id = buf.channels().at(i);
        changedtick_single(buf, channel_id);
        i += 1;
    }

    let mut j = 0;
    let mut i = 0;
    while i < buf.callbacks().len() {
        let cb = buf.callbacks().at(i);
        let mut keep = true;
        if cb.on_changedtick != LUA_NOREF {
            let mut args = ArrayBuf::<2>::new();
            args.push(Object::buffer(buf.handle));
            args.push(Object::integer(changedtick(buf)));
            let res = call_ref(cb.on_changedtick, c"changedtick", args.array(), kRetNilBool);
            if truthy(res) {
                callbacks_free(cb);
                keep = false;
            }
        }
        if keep {
            let moved = buf.callbacks().at(i);
            buf.callbacks().set_at(j, moved);
            j += 1;
        }
        i += 1;
    }
    buf.callbacks().set_len(j);
}

/// `nvim_buf_changedtick_event` for one channel.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn buf_updates_changedtick_single(buf: *mut buf_T, channel_id: uint64_t) {
    // SAFETY: the caller's promise.
    changedtick_single(unsafe { Buf::new(buf) }, channel_id);
}

fn changedtick_single(buf: Buf, channel_id: uint64_t) {
    let mut args = ArrayBuf::<2>::new();
    args.push(Object::buffer(buf.handle));
    args.push(Object::integer(changedtick(buf)));
    // Don't try and clean up dead channels here.
    send_event(channel_id, c"nvim_buf_changedtick_event", args.array());
}

/// Release one attachment's Lua references.
///
/// # Safety
/// The references must be ones the caller owns.
pub unsafe fn buffer_update_callbacks_free(cb: BufUpdateCallbacks) {
    callbacks_free(cb);
}
