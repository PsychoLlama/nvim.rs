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
//!
//! An event whose arguments outgrow the block grows it instead of being cut:
//! the packer's hook here only makes room, and [`push_call`] sends the
//! whole batch once the event is complete. So no UI ever has half an event
//! on its wire, and nothing that writes to the same channel has to flush
//! the UI first. (Upstream cut the event and sent the first half, then had
//! every RPC message on the channel flush the second half before itself.)

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::event::wstream::SharedBuffer;
use crate::memory::{ARENA_BLOCK_SIZE, block_vec};
use crate::msgpack_rpc::channel::rpc_write_raw;
use crate::msgpack_rpc::packer::{
    mpack_array, mpack_array_dyn16, mpack_object_array, mpack_str_small, mpack_uint,
};
use crate::types::{Array, PackerBuffer, RemoteUI};
use core::ffi::CStr;

/// The whole buffer for one batch. One arena block, so that the allocator
/// hands out and takes back a size it already keeps a free list for.
pub(super) const UI_BUF_SIZE: usize = ARENA_BLOCK_SIZE;

/// Room kept free for the next event, so that [`push_call`] never has to
/// check whether what it is packing fits.
const EVENT_BUF_SIZE: usize = 256;

/// Cells to let accumulate before flushing, so that a UI on a slow link
/// starts drawing the top of the screen while the rest is still coming.
pub(super) const MAX_CELLS_PENDING: usize = 500;

/// A packer for a new UI: no block until the first event.
pub(super) const fn ui_packer() -> PackerBuffer {
    PackerBuffer::detached(Some(grow_block))
}

/// Closes the current event, filling in how many argument lists it got.
///
/// The count is one more than the calls made, because the name occupies the
/// first slot of the event's array.
fn flush_event(ui: &mut RemoteUI) {
    if ui.cur_event.take().is_none() {
        return;
    }
    let ncalls = 1 + ui.ncalls;
    // `ncalls_pos` is the placeholder this event's header left in the block,
    // which is still the one being packed into.
    if let Some(at) = ui.ncalls_pos.take() {
        ui.packer.patch_be16(at, ncalls);
    }
    ui.ncalls = 0;
}

/// Makes room for one more call to `name`, opening a new event unless the
/// last one was for the same name.
pub(super) fn prepare_call(ui: &mut RemoteUI, name: &'static CStr) {
    if !ui.packer.is_detached() {
        let used = ui.packer.used();
        if used > UI_BUF_SIZE - EVENT_BUF_SIZE || ui.ncells_pending >= MAX_CELLS_PENDING {
            ui_flush_buf(ui);
        }
    }
    if ui.packer.is_detached() {
        // The UI owns the block until `ui_flush_buf` gives it away.
        ui.packer.adopt(block_vec());
    }

    if ui.cur_event == Some(name) {
        ui.ncalls += 1;
        return;
    }

    if ui.nevents_pos.is_none() {
        // The notification header, once per batch. The block has room --
        // `EVENT_BUF_SIZE` bytes are kept free for exactly this.
        mpack_array(&mut ui.packer, 3);
        mpack_uint(&mut ui.packer, 2);
        mpack_str_small(&mut ui.packer, b"redraw");
        let pos = mpack_array_dyn16(&mut ui.packer);
        ui.nevents_pos = Some(pos);
        debug_assert!(ui.cur_event.is_none());
    }
    flush_event(ui);
    ui.cur_event = Some(name);
    let pos = mpack_array_dyn16(&mut ui.packer);
    ui.ncalls_pos = Some(pos);
    mpack_str_small(&mut ui.packer, name.to_bytes());
    ui.nevents += 1;
    ui.ncalls = 1;
}

/// Queues `name(args...)` on `ui`'s buffer.
pub(super) fn push_call(ui: &mut RemoteUI, name: &'static CStr, mut args: Array) {
    prepare_call(ui, name);
    mpack_object_array(&mut args, &mut ui.packer);
    if ui.packer.capacity() > UI_BUF_SIZE {
        // The arguments outgrew the block (see the module docs): the event
        // is complete now, so the batch goes out whole.
        ui_flush_buf(ui);
    }
}

/// The packer's out-of-room hook: double the block, keeping what was packed.
///
/// Only [`push_call`]'s argument walk reaches it -- every other write is
/// covered by the reserve or checks for room itself -- and that caller sends
/// the grown block as soon as the event is complete.
fn grow_block(packer: &mut PackerBuffer) {
    packer.grow();
}

/// Sends whatever is packed, handing the block to the write stream.
pub(super) fn ui_flush_buf(ui: &mut RemoteUI) {
    if ui.packer.is_detached() || ui.packer.used() == 0 {
        return;
    }
    flush_event(ui);
    if let Some(at) = ui.nevents_pos.take() {
        // `at` is the placeholder the batch header left.
        ui.packer.patch_be16(at, ui.nevents);
        ui.nevents = 0;
    }

    // The block belongs to the write stream now; the next event will
    // allocate another.
    let block = ui.packer.take();
    rpc_write_raw(ui.channel_id, SharedBuffer::from_vec(block));
    ui.flushed_events = true;
    ui.ncells_pending = 0;
}
