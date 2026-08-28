//! One UI's outgoing msgpack buffer.
//!
//! Everything a UI is told arrives here as a name and an argument list and
//! leaves as bytes on that UI's channel. The wire shape is one `redraw`
//! notification carrying a batch:
//!
//! ```text
//! [2, "redraw", [ [name, args, args, ...], [name, args, ...], ... ]]
//! ```
//!
//! Two of those array lengths are not known when their header has to be
//! written — how many events the batch will hold, and how many argument
//! lists this event will get. Both are emitted as a fixed-width 16-bit
//! header with a placeholder ([`mpack_array_dyn16`]) whose position is
//! remembered on the [`RemoteUI`] (`nevents_pos`, `ncalls_pos`) and
//! overwritten once the count is known. That back-patch is what lets the
//! batch be streamed into a fixed 4 KiB block instead of being assembled in
//! a growable buffer first, and it is why this module is written by hand
//! rather than through the value builders.
//!
//! Consecutive calls with the same name are merged into one event, which is
//! what makes a screen redraw one `grid_line` event with hundreds of
//! argument lists rather than hundreds of events.
//!
//! The buffer is flushed when it is nearly full, when too many cells are
//! pending, or at the end of a redraw. Nearly-full is
//! [`UI_BUF_SIZE`] − [`EVENT_BUF_SIZE`]: every event small enough to be
//! packed by [`push_call`] fits in the reserve, so packing never has to
//! check for space mid-event. `grid_line` is the exception and checks for
//! itself — see [`line`](super::line).

#![deny(unsafe_op_in_unsafe_fn)]

use super::Ui;
use crate::event::wstream::wstream_new_buffer;
use crate::memory::{ARENA_BLOCK_SIZE, alloc_block, free_block, strequal};
use crate::msgpack_rpc::channel::rpc_write_raw;
use crate::msgpack_rpc::packer::{
    mpack_array, mpack_array_dyn16, mpack_be16, mpack_object_array, mpack_str_small, mpack_uint,
};
use crate::types::{Array, PackerBuffer, RemoteUI};
use core::ffi::{CStr, c_char};

/// The whole buffer for one batch. One arena block, so that the allocator
/// hands out and takes back a size it already keeps a free list for.
pub(super) const UI_BUF_SIZE: usize = ARENA_BLOCK_SIZE;

/// Room kept free for the next event, so that [`push_call`] never has to
/// check whether what it is packing fits.
const EVENT_BUF_SIZE: usize = 256;

/// Cells to let accumulate before flushing, so that a UI on a slow link
/// starts drawing the top of the screen while the rest is still coming.
pub(super) const MAX_CELLS_PENDING: usize = 500;

/// Closes the current event, filling in how many argument lists it got.
///
/// The count is one more than the calls made, because the name occupies the
/// first slot of the event's array.
///
/// # Safety
///
/// `ui` must be live.
unsafe fn flush_event(ui: *mut RemoteUI) {
    // SAFETY: the caller's promise -- `ui` is live.
    let mut live = unsafe { Ui::new(ui) };
    if live.cur_event.is_null() {
        return;
    }
    let ncalls = 1 + live.ncalls;
    // `ncalls_pos` is the placeholder this event's header left in the block,
    // which is still the one being packed into.
    mpack_be16(&mut live.ncalls_pos, ncalls);
    live.cur_event = core::ptr::null();
    live.ncalls_pos = core::ptr::null_mut();
    live.ncalls = 0;
}

/// Gives `ui` a fresh empty block to pack into.
///
/// # Safety
///
/// `ui` must be live and hold no block.
pub(super) unsafe fn ui_alloc_buf(ui: *mut RemoteUI) {
    // SAFETY: the caller's promise -- `ui` is live.
    let mut live = unsafe { Ui::new(ui) };
    // SAFETY: the allocator hands out a block nothing else names, and it is
    // `UI_BUF_SIZE` bytes, so its end is one past it.
    let (start, end) = unsafe {
        let start = alloc_block().cast::<c_char>();
        (start, start.add(UI_BUF_SIZE))
    };
    live.packer.startptr = start;
    live.packer.ptr = start;
    live.packer.endptr = end;
}

/// Makes room for one more call to `name`, opening a new event unless the
/// last one was for the same name.
///
/// # Safety
///
/// `ui` must be live and `name` a valid C string that outlives the batch —
/// it is stored on the UI and compared against on the next call.
pub(super) unsafe fn prepare_call(ui: *mut RemoteUI, name: &'static CStr) {
    // SAFETY: the caller's promise -- `ui` is live.
    let mut live = unsafe { Ui::new(ui) };
    if !live.packer.startptr.is_null() {
        let used = live.packer.ptr.addr() - live.packer.startptr.addr();
        if used > UI_BUF_SIZE - EVENT_BUF_SIZE || live.ncells_pending >= MAX_CELLS_PENDING {
            // SAFETY: as above.
            unsafe { ui_flush_buf(ui, false) };
        }
    }
    if live.packer.startptr.is_null() {
        // SAFETY: as above -- there is no block, whether or not one was
        // just handed away.
        unsafe { ui_alloc_buf(ui) };
    }

    let cur_event = live.cur_event;
    // SAFETY: `cur_event` is null or the static name the last call carried.
    let same = !cur_event.is_null() && unsafe { strequal(cur_event, name.as_ptr()) };
    if same {
        live.ncalls += 1;
        return;
    }

    if live.nevents_pos.is_null() {
        // The notification header, once per batch. The block has room --
        // `EVENT_BUF_SIZE` bytes are kept free for exactly this.
        mpack_array(&mut live.packer.ptr, 3);
        mpack_uint(&mut live.packer.ptr, 2);
        mpack_str_small(&mut live.packer.ptr, b"redraw");
        let pos = mpack_array_dyn16(&mut live.packer.ptr);
        live.nevents_pos = pos;
        debug_assert!(live.cur_event.is_null());
    }
    // SAFETY: as above.
    unsafe { flush_event(ui) };
    live.cur_event = name.as_ptr();
    let pos = mpack_array_dyn16(&mut live.packer.ptr);
    live.ncalls_pos = pos;
    mpack_str_small(&mut live.packer.ptr, name.to_bytes());
    live.nevents += 1;
    live.ncalls = 1;
}

/// Queues `name(args...)` on `ui`'s buffer.
///
/// # Safety
///
/// `ui` must be live, `name` must outlive the batch (see [`prepare_call`]),
/// and every value reachable from `args` must stay valid until this
/// returns — the packer copies as it goes and keeps nothing.
pub(super) unsafe fn push_call(ui: *mut RemoteUI, name: &'static CStr, args: Array) {
    // SAFETY: the caller's promise.
    unsafe { prepare_call(ui, name) };
    // SAFETY: as above. The packer is the UI's own, and reaches back to it
    // through `anydata` if it runs out of room mid-array.
    unsafe { mpack_object_array(args, &mut (*ui).packer) };
}

/// The packer's out-of-room callback: send what is packed and continue in a
/// fresh block.
///
/// It is reached from inside `mpack_object`, mid-event, which is why the
/// flush is told the event is incomplete: the reader must not act on the
/// batch until the rest arrives.
///
/// # Safety
///
/// Called by the packer with `packer` belonging to a live [`RemoteUI`].
pub(super) unsafe fn ui_flush_callback(packer: *mut PackerBuffer) {
    // SAFETY: the caller's promise -- the packer's `anydata` is the UI that
    // owns it, set when the UI was created.
    let ui = unsafe { (*packer).anydata }.cast::<RemoteUI>();
    // SAFETY: as above.
    unsafe { ui_flush_buf(ui, true) };
    // SAFETY: the flush left no block.
    unsafe { ui_alloc_buf(ui) };
}

/// Sends whatever is packed, handing the block to the write stream.
///
/// `incomplete_event` records that the batch was cut mid-event, which the
/// channel checks before deciding a UI is caught up.
///
/// # Safety
///
/// `ui` must be live.
pub(super) unsafe fn ui_flush_buf(ui: *mut RemoteUI, incomplete_event: bool) {
    // SAFETY: the caller's promise -- `ui` is live.
    let mut live = unsafe { Ui::new(ui) };
    if live.packer.startptr.is_null() || live.packer.ptr == live.packer.startptr {
        return;
    }
    live.incomplete_event = incomplete_event;
    // SAFETY: as above.
    unsafe { flush_event(ui) };
    if !live.nevents_pos.is_null() {
        let nevents = live.nevents;
        // `nevents_pos` is the placeholder the batch header left.
        mpack_be16(&mut live.nevents_pos, nevents);
        live.nevents = 0;
        live.nevents_pos = core::ptr::null_mut();
    }

    let start = live.packer.startptr;
    let size = live.packer.ptr.addr() - start.addr();
    let channel_id = live.channel_id;
    // `start` is the block, whose first `size` bytes are packed; the write
    // stream owns it from here and frees it with `free_block`.
    let buf = wstream_new_buffer(start, size, 1, Some(free_block));
    // SAFETY: `buf` is that buffer, and `channel_id` this UI's channel.
    unsafe { rpc_write_raw(channel_id, buf) };

    // The block belongs to the write stream now; the next event will
    // allocate another.
    live.packer.startptr = core::ptr::null_mut();
    live.packer.ptr = core::ptr::null_mut();
    live.flushed_events = true;
    live.ncells_pending = 0;
}

/// Sends anything packed but not yet flushed.
///
/// The channel calls this when it has drained its input, so that a UI does
/// not sit on a partial batch waiting for an event that is not coming.
///
/// # Safety
///
/// `ui` must be live.
pub unsafe fn remote_ui_flush_pending_data(ui: *mut RemoteUI) {
    // SAFETY: the caller's promise.
    unsafe { ui_flush_buf(ui, false) };
}
