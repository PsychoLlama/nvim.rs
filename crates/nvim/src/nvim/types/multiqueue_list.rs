//! A doubly-linked list over a slab, so that any element can be unlinked in
//! constant time given the handle its insertion returned.
//!
//! The multi-level queue needs exactly that. An event pushed onto a child
//! queue also takes a slot in the parent's list, and taking that event from
//! either side has to unlink the other's slot without walking to it.
//! Upstream got there with libuv's intrusive `QUEUE`, where each node's
//! address is its own handle; a slab expresses the same structure with
//! indices, and so without raw pointers.
//!
//! Handles carry the generation of the slot they name, so using one after
//! its element has been removed is a no-op rather than a hit on whatever
//! took the slot over.
#![forbid(unsafe_code)]

/// Where an element sits in a [`List`]. Only meaningful for the list that
/// produced it, and only until that element is removed.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Handle {
    index: u32,
    generation: u32,
}

struct Slot<T> {
    prev: Option<u32>,
    next: Option<u32>,
    value: Option<T>,
    generation: u32,
}

pub struct List<T> {
    slots: Vec<Slot<T>>,
    /// Slots holding no element, available for the next insertion.
    vacant: Vec<u32>,
    head: Option<u32>,
    tail: Option<u32>,
    len: usize,
}

impl<T> List<T> {
    pub const fn new() -> Self {
        List {
            slots: Vec::new(),
            vacant: Vec::new(),
            head: None,
            tail: None,
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Append `value`, returning the handle that names it.
    pub fn push_back(&mut self, value: T) -> Handle {
        let prev = self.tail;
        let index = match self.vacant.pop() {
            Some(index) => {
                let slot = &mut self.slots[index as usize];
                slot.prev = prev;
                slot.next = None;
                slot.value = Some(value);
                index
            }
            None => {
                let index = u32::try_from(self.slots.len()).expect("event queue slot overflow");
                self.slots.push(Slot {
                    prev,
                    next: None,
                    value: Some(value),
                    generation: 0,
                });
                index
            }
        };
        match prev {
            Some(prev) => self.slots[prev as usize].next = Some(index),
            None => self.head = Some(index),
        }
        self.tail = Some(index);
        self.len += 1;
        Handle {
            index,
            generation: self.slots[index as usize].generation,
        }
    }

    /// Take the first element.
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.map(|index| self.unlink(index))
    }

    /// Take the element `handle` names, or `None` if it is already gone.
    pub fn remove(&mut self, handle: Handle) -> Option<T> {
        let slot = self.slots.get(handle.index as usize)?;
        if slot.generation != handle.generation || slot.value.is_none() {
            return None;
        }
        Some(self.unlink(handle.index))
    }

    fn unlink(&mut self, index: u32) -> T {
        let slot = &self.slots[index as usize];
        let (prev, next) = (slot.prev, slot.next);
        match prev {
            Some(prev) => self.slots[prev as usize].next = next,
            None => self.head = next,
        }
        match next {
            Some(next) => self.slots[next as usize].prev = prev,
            None => self.tail = prev,
        }
        let slot = &mut self.slots[index as usize];
        slot.prev = None;
        slot.next = None;
        slot.generation = slot.generation.wrapping_add(1);
        let value = slot.value.take().expect("a linked slot holds an element");
        self.len -= 1;
        self.vacant.push(index);
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drain<T>(list: &mut List<T>) -> Vec<T> {
        let mut out = Vec::new();
        while let Some(value) = list.pop_front() {
            out.push(value);
        }
        out
    }

    #[test]
    fn pops_in_insertion_order() {
        let mut list = List::new();
        for n in 0..5 {
            list.push_back(n);
        }
        assert_eq!(list.len(), 5);
        assert_eq!(drain(&mut list), [0, 1, 2, 3, 4]);
        assert!(list.is_empty());
    }

    #[test]
    fn removes_from_the_middle_without_disturbing_the_order() {
        let mut list = List::new();
        let handles: Vec<_> = (0..5).map(|n| list.push_back(n)).collect();
        assert_eq!(list.remove(handles[2]), Some(2));
        assert_eq!(list.len(), 4);
        assert_eq!(drain(&mut list), [0, 1, 3, 4]);
    }

    #[test]
    fn removes_the_ends() {
        let mut list = List::new();
        let handles: Vec<_> = (0..3).map(|n| list.push_back(n)).collect();
        assert_eq!(list.remove(handles[0]), Some(0));
        assert_eq!(list.remove(handles[2]), Some(2));
        assert_eq!(drain(&mut list), [1]);
    }

    #[test]
    fn a_handle_stops_naming_anything_once_its_element_is_gone() {
        let mut list = List::new();
        let first = list.push_back('a');
        assert_eq!(list.pop_front(), Some('a'));
        assert_eq!(list.remove(first), None);
        // The slot is recycled; the stale handle must not reach its new
        // occupant.
        let second = list.push_back('b');
        assert_ne!(first, second);
        assert_eq!(list.remove(first), None);
        assert_eq!(list.remove(second), Some('b'));
    }

    #[test]
    fn reuses_vacant_slots_rather_than_growing() {
        let mut list = List::new();
        for _ in 0..100 {
            let handle = list.push_back(0);
            assert_eq!(list.remove(handle), Some(0));
        }
        assert_eq!(list.slots.len(), 1);
    }

    #[test]
    fn interleaves_pushes_and_removals() {
        let mut list = List::new();
        let a = list.push_back(1);
        list.push_back(2);
        let c = list.push_back(3);
        assert_eq!(list.remove(a), Some(1));
        list.push_back(4);
        assert_eq!(list.remove(c), Some(3));
        list.push_back(5);
        assert_eq!(drain(&mut list), [2, 4, 5]);
    }
}
