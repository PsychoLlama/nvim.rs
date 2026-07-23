//! Ring/dedup logic of the command-line history, exercised on a plain
//! [`Ring`] with no editor state involved.

use c2rust_neovim::src::nvim::cmdhist::Ring;

/// Entry texts oldest-first, walking raw slots back from the newest.
fn texts_newest_first(ring: &Ring) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let len = ring.len() as i32;
    if len == 0 || ring.newest_idx() < 0 {
        return out;
    }
    let start = ring.newest_idx();
    let mut i = start;
    loop {
        match ring.get(i) {
            None => break,
            Some(e) => out.push(e.text().to_vec()),
        }
        i = if i == 0 { len - 1 } else { i - 1 };
        if i == start {
            break;
        }
    }
    out
}

#[test]
fn add_assigns_increasing_numbers() {
    let mut ring = Ring::new(10);
    assert!(ring.is_empty());
    ring.add(b"one", 0, 100);
    ring.add(b"two", 0, 101);
    ring.add(b"three", 0, 102);
    assert_eq!(ring.newest_idx(), 2);
    assert_eq!(ring.newest_number(), 3);
    let newest = ring.get(2).unwrap();
    assert_eq!(newest.text(), b"three");
    assert_eq!(newest.timestamp(), 102);
    assert_eq!(ring.get(0).unwrap().number(), 1);
}

#[test]
fn add_wraps_and_overwrites_oldest() {
    let mut ring = Ring::new(3);
    for (i, text) in [b"a1", b"a2", b"a3", b"a4"].iter().enumerate() {
        ring.add(&text[..], 0, i as u64);
    }
    // Slot 0 was overwritten by the fourth entry.
    assert_eq!(ring.newest_idx(), 0);
    assert_eq!(ring.newest_number(), 4);
    assert_eq!(ring.get(0).unwrap().text(), b"a4");
    assert_eq!(ring.get(1).unwrap().text(), b"a2");
}

#[test]
fn move_to_front_renumbers_and_restamps() {
    let mut ring = Ring::new(5);
    ring.add(b"one", 0, 1);
    ring.add(b"two", 0, 2);
    ring.add(b"three", 0, 3);
    assert!(ring.move_to_front(b"one", None, 99));
    assert_eq!(
        texts_newest_first(&ring),
        [b"one".to_vec(), b"three".to_vec(), b"two".to_vec()]
    );
    let front = ring.get(ring.newest_idx()).unwrap();
    assert_eq!(front.number(), 4);
    assert_eq!(front.timestamp(), 99);
    // Unknown text is not found.
    assert!(!ring.move_to_front(b"missing", None, 100));
}

#[test]
fn move_to_front_distinguishes_search_separator() {
    let mut ring = Ring::new(5);
    ring.add(b"pat", b'/', 1);
    // Same text under the other search separator is a different entry.
    assert!(!ring.move_to_front(b"pat", Some(b'?'), 2));
    assert!(ring.move_to_front(b"pat", Some(b'/'), 3));
    assert_eq!(ring.get(ring.newest_idx()).unwrap().sep(), b'/');
}

#[test]
fn drop_newest_steps_back() {
    let mut ring = Ring::new(4);
    ring.add(b"one", 0, 1);
    ring.add(b"two", 0, 2);
    ring.drop_newest();
    assert_eq!(ring.newest_idx(), 0);
    assert_eq!(ring.newest_number(), 1);
    assert_eq!(ring.get(ring.newest_idx()).unwrap().text(), b"one");
}

#[test]
fn calc_idx_maps_numbers_to_slots() {
    let mut ring = Ring::new(3);
    for (i, text) in [b"a1", b"a2", b"a3", b"a4"].iter().enumerate() {
        ring.add(&text[..], 0, i as u64);
    }
    // Numbers 2..=4 survive; 1 was overwritten.
    assert_eq!(ring.calc_idx(1), -1);
    assert_eq!(ring.calc_idx(2), 1);
    assert_eq!(ring.calc_idx(4), 0);
    assert_eq!(ring.calc_idx(5), -1);
    // Negative numbers count back from the newest.
    assert_eq!(ring.calc_idx(-1), 0);
    assert_eq!(ring.calc_idx(-3), 1);
    assert_eq!(ring.calc_idx(-4), -1);
    assert_eq!(ring.calc_idx(0), -1);
}

#[test]
fn delete_matching_compacts_survivors() {
    let mut ring = Ring::new(5);
    ring.add(b"keep1", 0, 1);
    ring.add(b"drop1", 0, 2);
    ring.add(b"keep2", 0, 3);
    ring.add(b"drop2", 0, 4);
    assert!(ring.delete_matching(|e| e.text().starts_with(b"drop")));
    assert_eq!(
        texts_newest_first(&ring),
        [b"keep2".to_vec(), b"keep1".to_vec()]
    );
    // Survivors keep their numbers.
    assert_eq!(ring.get(ring.newest_idx()).unwrap().number(), 3);
    assert!(!ring.delete_matching(|e| e.text() == b"gone"));
}

#[test]
fn delete_matching_everything_empties_the_ring() {
    let mut ring = Ring::new(4);
    ring.add(b"a", 0, 1);
    ring.add(b"b", 0, 2);
    assert!(ring.delete_matching(|_| true));
    assert_eq!(ring.newest_idx(), -1);
    assert_eq!(ring.newest_number(), -1);
}

#[test]
fn delete_at_newest_leaves_idx_on_vacant_slot() {
    let mut ring = Ring::new(4);
    ring.add(b"only", 0, 1);
    ring.delete_at(0);
    // C quirk: idx steps back onto a vacant slot, and the vacant slot
    // reads as number 0 (what `histnr()` reports in this state).
    assert_eq!(ring.newest_idx(), 3);
    assert_eq!(ring.newest_number(), 0);
    assert!(ring.get(3).is_none());
}

#[test]
fn delete_at_middle_shifts_newer_entries_down() {
    let mut ring = Ring::new(4);
    ring.add(b"one", 0, 1);
    ring.add(b"two", 0, 2);
    ring.add(b"three", 0, 3);
    ring.delete_at(1); // "two"
    assert_eq!(
        texts_newest_first(&ring),
        [b"three".to_vec(), b"one".to_vec()]
    );
    assert_eq!(ring.newest_idx(), 1);
}

#[test]
fn resize_grow_keeps_all_entries() {
    let mut ring = Ring::new(3);
    ring.add(b"one", 0, 1);
    ring.add(b"two", 0, 2);
    ring.resize(6);
    assert_eq!(ring.len(), 6);
    assert_eq!(
        texts_newest_first(&ring),
        [b"two".to_vec(), b"one".to_vec()]
    );
    assert_eq!(ring.newest_number(), 2);
}

#[test]
fn resize_shrink_keeps_newest_entries() {
    let mut ring = Ring::new(5);
    for (i, text) in [b"e1", b"e2", b"e3", b"e4", b"e5"].iter().enumerate() {
        ring.add(&text[..], 0, i as u64);
    }
    ring.resize(2);
    assert_eq!(ring.len(), 2);
    assert_eq!(texts_newest_first(&ring), [b"e5".to_vec(), b"e4".to_vec()]);
    assert_eq!(ring.newest_number(), 5);
}

#[test]
fn resize_to_zero_and_back() {
    let mut ring = Ring::new(3);
    ring.add(b"gone", 0, 1);
    ring.resize(0);
    assert_eq!(ring.len(), 0);
    assert!(ring.is_empty());
    ring.resize(4);
    assert_eq!(ring.len(), 4);
    assert!(ring.is_empty());
}

#[test]
fn clear_resets_numbering() {
    let mut ring = Ring::new(3);
    ring.add(b"x", 0, 1);
    ring.clear();
    assert!(ring.is_empty());
    ring.add(b"y", 0, 2);
    assert_eq!(ring.newest_number(), 1);
}
