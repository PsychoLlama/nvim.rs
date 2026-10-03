//! The list stack, driven through what Vimscript reaches it by:
//! `setqflist()`'s list and `what` forms, `getqflist({what})` and the entry
//! walk `:cnext` and friends start from.
//!
//! No buffer and no window: every entry names no file, so nothing is listed
//! and no autocommand can fire, and the quickfix window the updates look for
//! does not exist. What is left is the stack itself — how many lists it
//! keeps, which one is current, the entries and the current one among them.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::buffer::{bare_buffer, free_bare_buffer};
use crate::eval::list::{string_bytes, string_tv};
use crate::eval::typval::{dict_find, list_items};
use crate::global_cell::{editor_state::Held, editor_state_lock};
use crate::memory::XString;
use crate::option::vars::{P_CPO, P_EFM};
use crate::types::{DictRef, ListRef, kListLenMayKnow};
use crate::window::{bare_window, free_bare_window};
use crate::winlayer::graph::{leave_curbuf, leave_curwin};
use core::ffi::c_int;
use std::ffi::CString;

/// `kListLenMayKnow`, as the length hint `tv_list_alloc` takes.
const MAY_KNOW: ptrdiff_t = -3;
const _: () = assert!(kListLenMayKnow == -3);

/// The quickfix stack emptied, with room for `slots` lists; emptied and
/// given back its default room on drop.
///
/// A bare window is made current for the length of the test, because adding
/// an entry asks for the current window's alternate buffer, and
/// `'errorformat'` and `'cpoptions'` are given values, since nothing ran the
/// option defaults.
struct Fixture {
    window: Win,
    buffer: Buf,
    efm: Option<XString>,
    cpo: Option<XString>,
    _held: Held,
}

impl Fixture {
    fn new(slots: c_int) -> Fixture {
        let held = editor_state_lock();
        got_int.set(false);
        let window = bare_window();
        window.make_current();
        // Compiling a pattern reads the current buffer's 'iskeyword'.
        let buffer = bare_buffer();
        buffer.make_current();
        let efm = P_EFM.swap(Some(XString::from("%f:%l:%m")));
        // Compiling a pattern reads 'cpoptions' too.
        let cpo = P_CPO.swap(Some(XString::from("aABceFs_")));
        qf_resize_stack(slots);
        free_stack();
        Fixture {
            window,
            buffer,
            efm,
            cpo,
            _held: held,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        free_stack();
        qf_resize_stack(10);
        P_EFM.restore(self.efm.take());
        P_CPO.restore(self.cpo.take());
        leave_curwin();
        free_bare_window(self.window);
        leave_curbuf();
        free_bare_buffer(self.buffer);
    }
}

/// One entry: `text` at `lnum`, valid or not as `valid` says. An entry
/// naming no file would be invalid by default, so every one says.
struct Item {
    text: &'static str,
    lnum: VarNumber,
    valid: Option<bool>,
}

const fn item(text: &'static str, lnum: VarNumber) -> Item {
    Item {
        text,
        lnum,
        valid: Some(true),
    }
}

const fn invalid(text: &'static str, lnum: VarNumber) -> Item {
    Item {
        text,
        lnum,
        valid: Some(false),
    }
}

fn items(entries: &[Item]) -> ListRef {
    let mut list = tv_list_alloc(MAY_KNOW);
    for entry in entries {
        let mut d = tv_dict_alloc();
        d.add_tv(b"text", &string_tv(entry.text.as_bytes()))
            .unwrap();
        d.add_number(b"lnum", entry.lnum).unwrap();
        if let Some(valid) = entry.valid {
            d.add_number(b"valid", VarNumber::from(valid)).unwrap();
        }
        list.push_dict(Some(d));
    }
    list
}

/// A `what` dictionary from number and string pairs.
fn what(numbers: &[(&str, VarNumber)], strings: &[(&str, &str)]) -> DictRef {
    let mut d = tv_dict_alloc();
    for (key, value) in numbers {
        d.add_number(key.as_bytes(), *value).unwrap();
    }
    for (key, value) in strings {
        d.add_tv(key.as_bytes(), &string_tv(value.as_bytes()))
            .unwrap();
    }
    d
}

fn title_of(title: &str) -> CString {
    CString::new(title).expect("a title without a NUL")
}

/// `setqflist(entries, action, title)`.
fn set_list(entries: &[Item], action: u8, title: &str) {
    let done = set_errorlist(None, Some(items(entries)), action, &title_of(title), None);
    assert!(done.is_ok());
}

/// `setqflist(entries, action, what)`, answering whether it was done.
fn set_what(entries: &[Item], action: u8, what: &DictRef) -> bool {
    let title = title_of(":setqflist()");
    set_errorlist(None, Some(items(entries)), action, &title, Some(what)).is_ok()
}

/// `setqflist([], 'f')`.
fn free_stack() {
    let _ = set_errorlist(None, Some(tv_list_alloc(0)), b'f', c"", None);
}

/// `getqflist(what)`.
fn get(what: &DictRef) -> DictRef {
    let mut answer = tv_dict_alloc();
    let _ = qf_get_properties(None, what, &mut answer);
    answer
}

fn number(d: &DictRef, key: &str) -> VarNumber {
    dict_find(Some(d), key.as_bytes())
        .unwrap_or_else(|| panic!("no {key} in the answer"))
        .di_tv
        .number_or_zero()
}

fn string(d: &DictRef, key: &str) -> Vec<u8> {
    string_bytes(
        &dict_find(Some(d), key.as_bytes())
            .unwrap_or_else(|| panic!("no {key} in the answer"))
            .di_tv,
    )
    .to_vec()
}

/// The `text` of every item in the answer's `items`.
fn texts(d: &DictRef) -> Vec<Vec<u8>> {
    let items = dict_find(Some(d), b"items").expect("no items in the answer");
    list_items(items.di_tv.list_ref())
        .iter()
        .map(|li| {
            let entry = li.li_tv.dict_ref().expect("an item is a dictionary");
            string_bytes(&dict_find(Some(entry), b"text").unwrap().di_tv).to_vec()
        })
        .collect()
}

/// The titles of lists 1 to `count`.
fn titles(count: VarNumber) -> Vec<Vec<u8>> {
    (1..=count)
        .map(|nr| string(&get(&what(&[("nr", nr), ("title", 1)], &[])), "title"))
        .collect()
}

fn current_nr() -> VarNumber {
    number(&get(&what(&[("nr", 0)], &[])), "nr")
}

fn current_idx() -> VarNumber {
    number(&get(&what(&[("idx", 0)], &[])), "idx")
}

fn bytes(texts: &[&str]) -> Vec<Vec<u8>> {
    texts.iter().map(|t| t.as_bytes().to_vec()).collect()
}

/// Where the entry walk lands from the current entry: `errornr` steps in
/// `dir`, or entry `errornr` itself when `dir` is 0.
fn walk(errornr: c_int, dir: c_int) -> Option<c_int> {
    let mut idx = 0;
    let found = qf_get_entry(Qi::global().current_list(), errornr, dir, &mut idx);
    found.ok().map(|_| idx)
}

#[test]
fn a_full_stack_drops_its_oldest_list() {
    let _f = Fixture::new(3);
    for title in ["a", "b", "c", "d", "e"] {
        set_list(&[item(title, 1)], b' ', title);
    }
    assert_eq!(number(&get(&what(&[], &[("nr", "$")])), "nr"), 3);
    assert_eq!(current_nr(), 3);
    assert_eq!(titles(3), bytes(&["c", "d", "e"]));
    // Ids are never reused: each newer list's is larger.
    let ids: Vec<VarNumber> = (1..=3)
        .map(|nr| number(&get(&what(&[("nr", nr), ("id", 0)], &[])), "id"))
        .collect();
    assert!(ids[0] < ids[1] && ids[1] < ids[2], "{ids:?}");
}

#[test]
fn shrinking_the_stack_keeps_the_newest_lists() {
    let _f = Fixture::new(5);
    for title in ["a", "b", "c", "d"] {
        set_list(&[item(title, 1)], b' ', title);
    }
    qf_resize_stack(2);
    assert_eq!(number(&get(&what(&[], &[("nr", "$")])), "nr"), 2);
    assert_eq!(titles(2), bytes(&["c", "d"]));
    assert_eq!(current_nr(), 2);
}

#[test]
fn a_new_list_below_the_top_drops_the_newer_ones() {
    let _f = Fixture::new(10);
    for title in ["a", "b", "c"] {
        set_list(&[item(title, 1)], b' ', title);
    }
    // Starting a list with list 1 current is `:colder 2` then `:grep`.
    assert!(set_what(&[], b' ', &what(&[("nr", 1)], &[("title", "x")])));
    assert_eq!(number(&get(&what(&[], &[("nr", "$")])), "nr"), 2);
    assert_eq!(titles(2), bytes(&["a", "x"]));
}

#[test]
fn setqflist_starts_at_the_first_entry_valid_or_not() {
    let _f = Fixture::new(10);
    set_list(
        &[invalid("banner", 0), item("one", 3), item("two", 4)],
        b' ',
        "t",
    );
    assert_eq!(current_idx(), 1);
    assert_eq!(number(&get(&what(&[("size", 0)], &[])), "size"), 3);
}

#[test]
fn a_parsed_list_starts_at_its_first_valid_entry() {
    let _f = Fixture::new(10);
    let mut lines = tv_list_alloc(MAY_KNOW);
    for line in ["banner", "3:one", "4:two"] {
        lines.push(string_tv(line.as_bytes()));
    }
    let mut w = what(&[], &[("efm", "%l:%m")]);
    w.add_list(b"lines", Some(lines)).unwrap();
    assert!(set_what(&[], b' ', &w));
    assert_eq!(current_idx(), 2);
}

#[test]
fn idx_moves_the_current_entry() {
    let _f = Fixture::new(10);
    let five = [
        item("1", 1),
        item("2", 2),
        item("3", 3),
        item("4", 4),
        item("5", 5),
    ];
    set_list(&five, b' ', "t");
    assert_eq!(current_idx(), 1);
    assert!(set_what(&[], b'a', &what(&[("idx", 4)], &[])));
    assert_eq!(current_idx(), 4);
    assert!(set_what(&[], b'a', &what(&[], &[("idx", "$")])));
    assert_eq!(current_idx(), 5);
    // Past the end is the last entry; zero is refused.
    assert!(set_what(&[], b'a', &what(&[("idx", 99)], &[])));
    assert_eq!(current_idx(), 5);
    assert!(!set_what(&[], b'a', &what(&[("idx", 0)], &[])));
    assert_eq!(current_idx(), 5);
}

#[test]
fn items_and_idx_answer_one_entry() {
    let _f = Fixture::new(10);
    set_list(&[item("a", 1), item("b", 2), item("c", 3)], b' ', "t");
    let all = get(&what(&[("items", 0)], &[]));
    assert_eq!(texts(&all), bytes(&["a", "b", "c"]));
    let one = get(&what(&[("items", 0), ("idx", 2)], &[]));
    assert_eq!(texts(&one), bytes(&["b"]));
    assert_eq!(number(&one, "idx"), 2);
}

#[test]
fn append_replace_and_items_keep_the_list() {
    let _f = Fixture::new(10);
    set_list(&[item("a", 1)], b' ', "first");
    let id = number(&get(&what(&[("id", 0)], &[])), "id");
    set_list(&[item("b", 2)], b'a', "ignored");
    assert_eq!(texts(&get(&what(&[("items", 0)], &[]))), bytes(&["a", "b"]));
    set_list(&[item("c", 3)], b'r', "replaced");
    let after = get(&what(&[("items", 0), ("id", 0), ("title", 0)], &[]));
    assert_eq!(texts(&after), bytes(&["c"]));
    assert_eq!(number(&after, "id"), id);
    assert_eq!(string(&after, "title"), b"replaced");

    // The `items` key replaces the entries and keeps the title.
    let mut w = what(&[], &[]);
    w.add_list(b"items", Some(items(&[item("d", 4), item("e", 5)])))
        .unwrap();
    assert!(set_what(&[], b'r', &w));
    let after = get(&what(&[("items", 0), ("title", 0), ("nr", 0)], &[]));
    assert_eq!(texts(&after), bytes(&["d", "e"]));
    assert_eq!(string(&after, "title"), b"replaced");
    assert_eq!(number(&after, "nr"), 1);
}

#[test]
fn the_change_tick_moves_with_each_change() {
    let _f = Fixture::new(10);
    set_list(&[item("a", 1)], b' ', "t");
    let tick = |()| number(&get(&what(&[("changedtick", 0)], &[])), "changedtick");
    let first = tick(());
    set_list(&[item("b", 2)], b'a', "t");
    assert_eq!(tick(()), first + 1);
    assert!(set_what(&[], b'a', &what(&[], &[("title", "u")])));
    assert_eq!(tick(()), first + 2);
}

#[test]
fn context_and_user_data_come_back() {
    let _f = Fixture::new(10);
    let mut list = tv_list_alloc(MAY_KNOW);
    let mut d = tv_dict_alloc();
    d.add_tv(b"text", &string_tv(b"e")).unwrap();
    d.add_number(b"user_data", 42).unwrap();
    list.push_dict(Some(d));
    let done = set_errorlist(None, Some(list), b' ', &title_of("t"), None);
    assert!(done.is_ok());
    assert!(set_what(&[], b'a', &what(&[("context", 7)], &[])));

    let answer = get(&what(&[("context", 0), ("items", 0)], &[]));
    assert_eq!(number(&answer, "context"), 7);
    let items = dict_find(Some(&answer), b"items").unwrap();
    let entry = list_items(items.di_tv.list_ref())[0]
        .li_tv
        .dict_ref()
        .unwrap();
    assert_eq!(
        dict_find(Some(entry), b"user_data")
            .unwrap()
            .di_tv
            .number_or_zero(),
        42
    );
}

#[test]
fn freeing_the_stack_leaves_nothing() {
    let _f = Fixture::new(10);
    set_list(&[item("a", 1)], b' ', "t");
    set_list(&[item("b", 1)], b' ', "u");
    free_stack();
    assert_eq!(current_nr(), 0);
    assert_eq!(number(&get(&what(&[], &[("nr", "$")])), "nr"), 0);
}

#[test]
fn lines_parse_into_a_throwaway_list() {
    let _f = Fixture::new(10);
    set_list(&[item("kept", 1)], b' ', "t");
    let mut lines = tv_list_alloc(MAY_KNOW);
    for line in ["10:2:first", "  more", "20:4:second"] {
        lines.push(string_tv(line.as_bytes()));
    }
    let mut w = what(&[], &[("efm", "%l:%c:%m")]);
    w.add_list(b"lines", Some(lines)).unwrap();
    let answer = get(&w);
    let items = dict_find(Some(&answer), b"items").unwrap();
    let got: Vec<(VarNumber, VarNumber, Vec<u8>)> = list_items(items.di_tv.list_ref())
        .iter()
        .map(|li| {
            let d = li.li_tv.dict_ref().unwrap();
            let n = |k: &[u8]| dict_find(Some(d), k).unwrap().di_tv.number_or_zero();
            let text = string_bytes(&dict_find(Some(d), b"text").unwrap().di_tv).to_vec();
            (n(b"lnum"), n(b"col"), text)
        })
        .collect();
    assert_eq!(
        got,
        vec![
            (10, 2, b"first".to_vec()),
            (0, 0, b"  more".to_vec()),
            (20, 4, b"second".to_vec()),
        ]
    );
    // The real list is untouched.
    assert_eq!(texts(&get(&what(&[("items", 0)], &[]))), bytes(&["kept"]));
}

#[test]
fn a_multiline_message_folds_into_one_entry() {
    let _f = Fixture::new(10);
    let mut lines = tv_list_alloc(MAY_KNOW);
    for line in ["E 10:first", "C second", "Z", "E 20:other", "Z"] {
        lines.push(string_tv(line.as_bytes()));
    }
    let mut w = what(&[], &[("efm", "%EE %l:%m,%CC %m,%ZZ")]);
    w.add_list(b"lines", Some(lines)).unwrap();
    let answer = get(&w);
    assert_eq!(texts(&answer), bytes(&["first\nsecond", "other"]));
}

#[test]
fn the_entry_walk_skips_invalid_entries() {
    let _f = Fixture::new(10);
    set_list(
        &[
            item("1", 1),
            invalid("2", 2),
            item("3", 3),
            invalid("4", 4),
            item("5", 5),
        ],
        b' ',
        "t",
    );
    assert_eq!(current_idx(), 1);
    assert_eq!(walk(1, FORWARD as c_int), Some(3));
    assert_eq!(walk(2, FORWARD as c_int), Some(5));
    // Running out part way stops on the last one found.
    assert_eq!(walk(9, FORWARD as c_int), Some(5));
    // Without a direction the number is the entry, invalid or not.
    assert_eq!(walk(4, 0), Some(4));
    assert_eq!(walk(99, 0), Some(5));
    assert_eq!(walk(0, 0), Some(1));

    assert!(set_what(&[], b'a', &what(&[("idx", 5)], &[])));
    assert_eq!(walk(1, BACKWARD as c_int), Some(3));
    assert_eq!(walk(2, BACKWARD as c_int), Some(1));
}

#[test]
fn a_list_of_nothing_valid_starts_at_its_first_entry() {
    let _f = Fixture::new(10);
    let mut lines = tv_list_alloc(MAY_KNOW);
    for line in ["no", "match", "here"] {
        lines.push(string_tv(line.as_bytes()));
    }
    let mut w = what(&[], &[("efm", "%l:%m")]);
    w.add_list(b"lines", Some(lines)).unwrap();
    assert!(set_what(&[], b' ', &w));
    assert_eq!(current_idx(), 1);
    // Every entry is fair game when none is valid.
    assert_eq!(walk(2, FORWARD as c_int), Some(3));
    assert_eq!(
        texts(&get(&what(&[("items", 0)], &[]))),
        bytes(&["no", "match", "here"])
    );
}
