//! The blocks a memfile holds in memory, found by number or by handle.
//!
//! Split out of [`super`] for the file-size cap. The order the table keeps
//! is the one the module docs describe — insertion order, with a removal
//! swapping the last block into the hole — and it is observable in the swap
//! file, so every operation here reproduces exactly what the `PMap` it
//! replaced did to that order. What it adds is a handle: a block is filed
//! in a slot that does not move while the block is in memory, so a
//! [`BlockId`] stays good across everything that reorders the table and is
//! refused (by a panic) once the block has gone.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::BlockMap;
use crate::types::BlockNr;
use core::ffi::c_uint;

/// A handle on a block in memory.
///
/// Good from [`MemFile::new_block`](super::MemFile::new_block) or
/// [`MemFile::get`](super::MemFile::get) until the block is freed or
/// released; after that the slot's generation has moved on and the handle
/// names nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct BlockId {
    slot: u32,
    generation: u32,
}

impl BlockId {
    /// A handle that names no block in any memfile, for tests of what holds
    /// one.
    #[cfg(test)]
    pub(crate) fn unbacked() -> BlockId {
        BlockId {
            slot: u32::MAX,
            generation: 0,
        }
    }
}

/// One block: a run of `page_count` pages, and the bytes holding them.
pub(crate) struct BlockHdr {
    /// The block number, which is also the key it is filed under.
    pub(super) bnum: BlockNr,
    /// `page_count` pages, zeroed when the block is made.
    pub(super) data: Box<[u8]>,
    pub(super) page_count: c_uint,
    /// `BH_DIRTY` and/or `BH_LOCKED`.
    pub(super) flags: c_uint,
}

impl BlockHdr {
    /// A block of `page_count` pages, zeroed.
    ///
    /// The zeroing is not just hygiene: a page written to the swap file
    /// straight from `malloc` would carry whatever the last owner of that
    /// memory held — up to and including the contents of a file the user
    /// read earlier in the session.
    pub(super) fn new(page_size: c_uint, page_count: c_uint) -> BlockHdr {
        let bytes = page_size as usize * page_count as usize;
        BlockHdr {
            bnum: 0,
            data: vec![0u8; bytes].into_boxed_slice(),
            page_count,
            flags: 0,
        }
    }
}

/// A slot or order index as the table stores it.
///
/// # Panics
/// Past four billion blocks in memory, which a page each would take sixteen
/// terabytes to reach.
fn position(at: usize) -> u32 {
    u32::try_from(at).expect("fewer than 2^32 blocks in memory")
}

struct Slot {
    block: Option<BlockHdr>,
    /// Bumped every time the slot is vacated, so a stale handle misses.
    generation: u32,
    /// Where the block is in `order`.
    order_at: u32,
}

#[derive(Default)]
pub(super) struct BlockTable {
    slots: Vec<Slot>,
    /// Slots with no block in them, reused before `slots` grows.
    vacant: Vec<u32>,
    /// The slots holding a block, in the iteration order.
    order: Vec<u32>,
    index: BlockMap<u32>,
}

impl BlockTable {
    pub(super) fn len(&self) -> usize {
        self.order.len()
    }

    /// The `i`-th block in iteration order. Callers walk by index because
    /// the table can change under them.
    pub(super) fn id_at(&self, i: usize) -> BlockId {
        let slot = self.order[i];
        BlockId {
            slot,
            generation: self.slots[slot as usize].generation,
        }
    }

    pub(super) fn find(&self, nr: BlockNr) -> Option<BlockId> {
        let slot = *self.index.get(&nr)?;
        Some(BlockId {
            slot,
            generation: self.slots[slot as usize].generation,
        })
    }

    fn slot(&self, id: BlockId) -> &Slot {
        let slot = &self.slots[id.slot as usize];
        assert!(
            slot.generation == id.generation,
            "memfile: a block was used after it was released"
        );
        slot
    }

    fn slot_mut(&mut self, id: BlockId) -> &mut Slot {
        let slot = &mut self.slots[id.slot as usize];
        assert!(
            slot.generation == id.generation,
            "memfile: a block was used after it was released"
        );
        slot
    }

    #[inline]
    pub(super) fn block(&self, id: BlockId) -> &BlockHdr {
        self.slot(id)
            .block
            .as_ref()
            .expect("memfile: an empty slot")
    }

    #[inline]
    pub(super) fn block_mut(&mut self, id: BlockId) -> &mut BlockHdr {
        self.slot_mut(id)
            .block
            .as_mut()
            .expect("memfile: an empty slot")
    }

    /// Two different blocks, both writable.
    pub(super) fn block_pair_mut(
        &mut self,
        a: BlockId,
        b: BlockId,
    ) -> (&mut BlockHdr, &mut BlockHdr) {
        let _ = (self.slot(a), self.slot(b));
        let [a, b] = self
            .slots
            .get_disjoint_mut([a.slot as usize, b.slot as usize])
            .expect("memfile: two different blocks");
        (
            a.block.as_mut().expect("memfile: an empty slot"),
            b.block.as_mut().expect("memfile: an empty slot"),
        )
    }

    /// File `block` under its own number, at the end of the order.
    pub(super) fn insert(&mut self, block: BlockHdr) -> BlockId {
        let nr = block.bnum;
        let order_at = position(self.order.len());
        let slot = match self.vacant.pop() {
            Some(slot) => {
                let entry = &mut self.slots[slot as usize];
                entry.block = Some(block);
                entry.order_at = order_at;
                slot
            }
            None => {
                self.slots.push(Slot {
                    block: Some(block),
                    generation: 0,
                    order_at,
                });
                position(self.slots.len() - 1)
            }
        };
        self.order.push(slot);
        let previous = self.index.insert(nr, slot);
        debug_assert!(previous.is_none(), "memfile: block {nr} filed twice");
        BlockId {
            slot,
            generation: self.slots[slot as usize].generation,
        }
    }

    /// Take `id`'s slot out of the order, moving the last one into its place.
    fn unorder(&mut self, slot: u32) {
        let at = self.slots[slot as usize].order_at as usize;
        self.order.swap_remove(at);
        if let Some(&moved) = self.order.get(at) {
            self.slots[moved as usize].order_at = position(at);
        }
    }

    /// Put block `id` last in the order, as taking it out and filing it
    /// again did.
    pub(super) fn move_to_end(&mut self, id: BlockId) {
        let _ = self.slot(id);
        self.unorder(id.slot);
        self.slots[id.slot as usize].order_at = position(self.order.len());
        self.order.push(id.slot);
    }

    /// File block `id` under `nr` instead, last in the order.
    pub(super) fn renumber(&mut self, id: BlockId, nr: BlockNr) {
        let old = self.block(id).bnum;
        self.index.remove(&old);
        self.block_mut(id).bnum = nr;
        let previous = self.index.insert(nr, id.slot);
        debug_assert!(previous.is_none(), "memfile: block {nr} filed twice");
        self.move_to_end(id);
    }

    /// Take block `id` out, moving the last one into its place in the order.
    pub(super) fn remove(&mut self, id: BlockId) -> BlockHdr {
        let _ = self.slot(id);
        self.unorder(id.slot);
        let entry = &mut self.slots[id.slot as usize];
        let block = entry.block.take().expect("memfile: an empty slot");
        entry.generation = entry.generation.wrapping_add(1);
        self.vacant.push(id.slot);
        self.index.remove(&block.bnum);
        block
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(nr: BlockNr) -> BlockHdr {
        let mut block = BlockHdr::new(16, 1);
        block.bnum = nr;
        block
    }

    fn order(table: &BlockTable) -> Vec<BlockNr> {
        (0..table.len())
            .map(|i| table.block(table.id_at(i)).bnum)
            .collect()
    }

    #[test]
    fn removal_swaps_the_last_block_into_the_hole() {
        let mut table = BlockTable::default();
        let ids: Vec<_> = (0..5).map(|nr| table.insert(block(nr))).collect();
        drop(table.remove(ids[1]));
        assert_eq!(order(&table), [0, 4, 2, 3]);
        assert!(table.find(1).is_none());
        assert_eq!(table.find(4), Some(ids[4]));
    }

    #[test]
    fn a_handle_survives_reordering_and_renumbering() {
        let mut table = BlockTable::default();
        let a = table.insert(block(-1));
        let b = table.insert(block(7));
        table.page_write(a, 3);
        table.move_to_end(a);
        assert_eq!(order(&table), [7, -1]);
        table.renumber(a, 9);
        assert_eq!(table.block(a).bnum, 9);
        assert_eq!(table.find(9), Some(a));
        assert!(table.find(-1).is_none());
        assert_eq!(table.block(a).data[0], 3);
        assert_eq!(table.block(b).bnum, 7);
    }

    #[test]
    #[should_panic(expected = "used after it was released")]
    fn a_released_block_s_handle_names_nothing() {
        let mut table = BlockTable::default();
        let a = table.insert(block(1));
        drop(table.remove(a));
        // The slot is reused, but under a new generation.
        let b = table.insert(block(2));
        assert_ne!(a, b);
        let _ = table.block(a);
    }

    #[test]
    fn two_blocks_at_once() {
        let mut table = BlockTable::default();
        let a = table.insert(block(1));
        let b = table.insert(block(2));
        let (x, y) = table.block_pair_mut(a, b);
        x.data[0] = 1;
        y.data[0] = 2;
        assert_eq!((table.block(a).data[0], table.block(b).data[0]), (1, 2));
    }

    impl BlockTable {
        fn page_write(&mut self, id: BlockId, byte: u8) {
            self.block_mut(id).data[0] = byte;
        }
    }
}
