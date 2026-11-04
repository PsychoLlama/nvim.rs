//! Ring/dedup logic of the command-line history, exercised on a plain
//! [`Ring`] with no editor state involved.

use neovim::cmdhist::Ring;

/// Entry texts oldest-first, walking raw slots back from the newest.
fn texts_newest_first(ring: &Ring) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let len = ring.len();
    let Some(start) = ring.newest_idx().filter(|_| len != 0) else {
        return out;
    };
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
    assert_eq!(ring.newest_idx(), Some(2));
    assert_eq!(ring.newest_number(), Some(3));
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
    assert_eq!(ring.newest_idx(), Some(0));
    assert_eq!(ring.newest_number(), Some(4));
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
    let front = ring.get(ring.newest_idx().unwrap()).unwrap();
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
    assert_eq!(ring.get(ring.newest_idx().unwrap()).unwrap().sep(), b'/');
}

#[test]
fn drop_newest_steps_back() {
    let mut ring = Ring::new(4);
    ring.add(b"one", 0, 1);
    ring.add(b"two", 0, 2);
    ring.drop_newest();
    assert_eq!(ring.newest_idx(), Some(0));
    assert_eq!(ring.newest_number(), Some(1));
    assert_eq!(ring.get(ring.newest_idx().unwrap()).unwrap().text(), b"one");
}

#[test]
fn calc_idx_maps_numbers_to_slots() {
    let mut ring = Ring::new(3);
    for (i, text) in [b"a1", b"a2", b"a3", b"a4"].iter().enumerate() {
        ring.add(&text[..], 0, i as u64);
    }
    // Numbers 2..=4 survive; 1 was overwritten.
    assert_eq!(ring.calc_idx(1), None);
    assert_eq!(ring.calc_idx(2), Some(1));
    assert_eq!(ring.calc_idx(4), Some(0));
    assert_eq!(ring.calc_idx(5), None);
    // Negative numbers count back from the newest.
    assert_eq!(ring.calc_idx(-1), Some(0));
    assert_eq!(ring.calc_idx(-3), Some(1));
    assert_eq!(ring.calc_idx(-4), None);
    assert_eq!(ring.calc_idx(0), None);
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
    assert_eq!(ring.get(ring.newest_idx().unwrap()).unwrap().number(), 3);
    assert!(!ring.delete_matching(|e| e.text() == b"gone"));
}

#[test]
fn delete_matching_everything_empties_the_ring() {
    let mut ring = Ring::new(4);
    ring.add(b"a", 0, 1);
    ring.add(b"b", 0, 2);
    assert!(ring.delete_matching(|_| true));
    assert_eq!(ring.newest_idx(), None);
    assert_eq!(ring.newest_number(), None);
}

#[test]
fn delete_at_newest_leaves_idx_on_vacant_slot() {
    let mut ring = Ring::new(4);
    ring.add(b"only", 0, 1);
    ring.delete_at(0);
    // C quirk: idx steps back onto a vacant slot, and the vacant slot
    // reads as number 0 (what `histnr()` reports in this state).
    assert_eq!(ring.newest_idx(), Some(3));
    assert_eq!(ring.newest_number(), Some(0));
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
    assert_eq!(ring.newest_idx(), Some(1));
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
    assert_eq!(ring.newest_number(), Some(2));
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
    assert_eq!(ring.newest_number(), Some(5));
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
    assert_eq!(ring.newest_number(), Some(1));
}

/// `:history`'s argument and the `hist*()` functions' refusals, with every
/// message each puts out: a message reported twice, or not at all, moves
/// the row.
#[cfg(not(miri))]
#[test]
fn refusals_report_once() {
    use crate::support::{Probe, Sandbox};
    let sandbox = Sandbox::globals();
    let fx = Probe::new(sandbox.editor());
    fx.run("call histdel('/') | call histdel('@') | call histdel('>')");
    fx.run("call histadd('/', 'one') | call histadd('/', 'two') | call histadd('/', 'three')");
    for (cmd, want) in [
        ("history bogus", "E488: Trailing characters: bogus"),
        ("history allx", "E488: Trailing characters: allx"),
        ("history / 1,x", "E488: Trailing characters: x"),
        ("history / 1 2", "E488: Trailing characters: 2"),
        (
            "history / 99999999999",
            "E1510: Value too large: / 99999999999",
        ),
        ("history / 2", "none"),
        ("history / -1", "none"),
        ("history / ,2", "none"),
        ("history / 5,9", "none"),
        ("history @", "none"),
    ] {
        fx.check_messages(cmd, want);
    }
    for (call, want, answer) in [
        ("histget('bogus')", "none", "''"),
        ("histget('/', 99)", "none", "''"),
        ("histget('/', -1)", "none", "'three'"),
        ("histget('/', 2)", "none", "'two'"),
        (
            "histget('/', [])",
            "E745: Using a List as a Number",
            "'three'",
        ),
        ("histnr('bogus')", "none", "-1"),
        ("histnr('/')", "none", "3"),
        ("histnr('@')", "none", "-1"),
        ("histadd('bogus', 'x')", "none", "0"),
        ("histadd('/', '')", "none", "0"),
        ("histdel('bogus')", "none", "0"),
        ("histdel('/', 99)", "none", "0"),
        ("histdel('/', 'zzz')", "none", "0"),
        ("histdel('/', 'two')", "none", "1"),
        ("histdel('/', [])", "E730: Using a List as a String", "0"),
        (
            "[histget('/', -1), histget('/', 1), histnr('/')]",
            "none",
            "['three', 'one', 3]",
        ),
    ] {
        fx.check_messages(&format!("let g:r = {call}"), want);
        fx.check("g:r", answer);
    }
    fx.run("call histdel('/')");
}
