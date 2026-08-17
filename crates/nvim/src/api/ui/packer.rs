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
    unsafe {
        if (*ui).cur_event.is_null() {
            return;
        }
        mpack_be16(&mut (*ui).ncalls_pos, 1 + (*ui).ncalls);
        (*ui).cur_event = core::ptr::null();
        (*ui).ncalls_pos = core::ptr::null_mut();
        (*ui).ncalls = 0;
    }
}

/// Gives `ui` a fresh empty block to pack into.
///
/// # Safety
///
/// `ui` must be live and hold no block.
pub(super) unsafe fn ui_alloc_buf(ui: *mut RemoteUI) {
    unsafe {
        (*ui).packer.startptr = alloc_block().cast::<c_char>();
        (*ui).packer.ptr = (*ui).packer.startptr;
        (*ui).packer.endptr = (*ui).packer.startptr.add(UI_BUF_SIZE);
    }
}

/// Makes room for one more call to `name`, opening a new event unless the
/// last one was for the same name.
///
/// # Safety
///
/// `ui` must be live and `name` a valid C string that outlives the batch —
/// it is stored on the UI and compared against on the next call.
pub(super) unsafe fn prepare_call(ui: *mut RemoteUI, name: &'static CStr) {
    unsafe {
        if !(*ui).packer.startptr.is_null() {
            let used = (*ui).packer.ptr.addr() - (*ui).packer.startptr.addr();
            if used > UI_BUF_SIZE - EVENT_BUF_SIZE || (*ui).ncells_pending >= MAX_CELLS_PENDING {
                ui_flush_buf(ui, false);
            }
        }
        if (*ui).packer.startptr.is_null() {
            ui_alloc_buf(ui);
        }

        if !(*ui).cur_event.is_null() && strequal((*ui).cur_event, name.as_ptr()) {
            (*ui).ncalls += 1;
            return;
        }

        if (*ui).nevents_pos.is_null() {
            // The notification header, once per batch.
            mpack_array(&mut (*ui).packer.ptr, 3);
            mpack_uint(&mut (*ui).packer.ptr, 2);
            mpack_str_small(&mut (*ui).packer.ptr, b"redraw");
            (*ui).nevents_pos = mpack_array_dyn16(&mut (*ui).packer.ptr);
            debug_assert!((*ui).cur_event.is_null());
        }
        flush_event(ui);
        (*ui).cur_event = name.as_ptr();
        (*ui).ncalls_pos = mpack_array_dyn16(&mut (*ui).packer.ptr);
        mpack_str_small(&mut (*ui).packer.ptr, name.to_bytes());
        (*ui).nevents += 1;
        (*ui).ncalls = 1;
    }
}

/// Queues `name(args...)` on `ui`'s buffer.
///
/// # Safety
///
/// `ui` must be live, `name` must outlive the batch (see [`prepare_call`]),
/// and every value reachable from `args` must stay valid until this
/// returns — the packer copies as it goes and keeps nothing.
pub(super) unsafe fn push_call(ui: *mut RemoteUI, name: &'static CStr, args: Array) {
    unsafe {
        prepare_call(ui, name);
        mpack_object_array(args, &mut (*ui).packer);
    }
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
pub(super) unsafe extern "C" fn ui_flush_callback(packer: *mut PackerBuffer) {
    unsafe {
        let ui = (*packer).anydata.cast::<RemoteUI>();
        ui_flush_buf(ui, true);
        ui_alloc_buf(ui);
    }
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
    unsafe {
        if (*ui).packer.startptr.is_null() || (*ui).packer.ptr == (*ui).packer.startptr {
            return;
        }
        (*ui).incomplete_event = incomplete_event;
        flush_event(ui);
        if !(*ui).nevents_pos.is_null() {
            mpack_be16(&mut (*ui).nevents_pos, (*ui).nevents);
            (*ui).nevents = 0;
            (*ui).nevents_pos = core::ptr::null_mut();
        }

        let size = (*ui).packer.ptr.addr() - (*ui).packer.startptr.addr();
        let buf = wstream_new_buffer((*ui).packer.startptr, size, 1, Some(free_block));
        rpc_write_raw((*ui).channel_id, buf);

        // The block belongs to the write stream now; the next event will
        // allocate another.
        (*ui).packer.startptr = core::ptr::null_mut();
        (*ui).packer.ptr = core::ptr::null_mut();
        (*ui).flushed_events = true;
        (*ui).ncells_pending = 0;
    }
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
    unsafe { ui_flush_buf(ui, false) };
}
