//! The first cases lifted out of `test/unit/eval/typval_spec.lua`, plus the
//! three that close what no differential can see.
//!
//! The spec asserted an exact allocation sequence through the LuaJIT
//! allocator seam 285 times, which is why it was never ported; these were
//! written as the proof that the seam has a Rust twin, one per shape the
//! rest of the spec is built out of:
//!
//! - an allocation *sequence* whose order is the assertion
//!   (`tv_list_append_string`),
//! - a *size* derived from a struct's layout, which is the only evidence the
//!   over-allocation happened (`tv_dict_item_alloc`),
//! - and `tv_dict_add`, whose interesting step allocates nothing at all.
//!
//! The rest of the spec is in `typval_list`, `typval_dict` and
//! `typval_value`, and the value model both use is `crate::support::tv`.
//! See `crate::support::alloc` for the porting rules; the short version is
//! that a size is always `size_of`/`offset_of!`, never a literal.
//!
//! Every case here needs a live editor, which Miri cannot start.

#![cfg(not(miri))]

use std::ffi::{CStr, c_char};
use std::ptr;

use neovim::eval::typval::{
    tv_dict_add, tv_dict_alloc, tv_dict_free, tv_dict_is_watched, tv_dict_item_alloc,
    tv_dict_item_alloc_len, tv_dict_item_free, tv_dict_item_remove, tv_dict_watcher_add,
    tv_dict_watcher_remove, tv_list_alloc, tv_list_append_number, tv_list_append_string,
    tv_list_find, tv_list_first, tv_list_last, tv_list_len, tv_list_remove_range, tv_list_unref,
    tv_list_watch_add, tv_list_watch_remove,
};
use neovim::memory::xstrdup;
use neovim::types::{Callback, Failed, ListWatch, TypVal, VAR_UNKNOWN, kListLenUnknown, ptrdiff_t};

use crate::support::alloc::{self, AllocLog};
use crate::support::tv::Payload;
use crate::support::{check_emsg, cstr, editor_lock};

/// `describe('list') describe('append') describe('string()') itp('works')`,
/// spec line 663.
///
/// The assertion is that the *string* is copied and nothing else is: the
/// list owns its items in one array, so appending allocates only when the
/// array has to grow, and that growth is the Rust allocator's, which this
/// log does not see.
#[test]
fn tv_list_append_string_copies_the_string_and_allocates_no_item() {
    let log = AllocLog::start();
    // SAFETY: the list is this case's own, freed at the end; the strings
    // outlive every call that reads them.
    unsafe {
        let l = tv_list_alloc(kListLenUnknown as isize);
        log.check(&[alloc::list(l)]);

        let test = cstr("test");
        tv_list_append_string(l, test.as_ptr(), 3);
        log.check(&[alloc::string(last_string(l), 3)]);

        // A NULL string allocates nothing at all.
        tv_list_append_string(l, ptr::null(), 0);
        log.check(&[]);
        tv_list_append_string(l, ptr::null(), -1);
        log.check(&[]);

        // A negative length means "to the terminator".
        tv_list_append_string(l, test.as_ptr(), -1);
        log.check(&[alloc::string(last_string(l), 4)]);

        assert_eq!(strings(l), [Some("tes"), None, None, Some("test")]);

        // The spec left this to a LuaJIT finalizer, so it never said what
        // freeing costs. It is worth saying: the items release their strings
        // front to back, and the list goes last. An item holding a NULL
        // string reaches the allocator not at all: `tv_clear` recognises an
        // already-empty value and returns, where the C called `xfree(NULL)`.
        let held: Vec<*mut c_char> = (0..tv_list_len(l))
            .map(|at| (*tv_list_find(l, at)).li_tv.string())
            .collect();
        let mut expected: Vec<_> = held
            .iter()
            .filter(|s| !s.is_null())
            .map(|&s| alloc::freed(s))
            .collect();
        expected.push(alloc::freed(l));
        tv_list_unref(l);
        log.check(&expected);
    }
}

/// The string the last item of `l` holds.
///
/// # Safety
/// `l` is a live, non-empty list whose last item holds a `VAR_STRING`.
unsafe fn last_string(l: *mut neovim::types::List) -> *mut c_char {
    unsafe { (*tv_list_last(l)).li_tv.string() }
}

/// The list's items as UTF-8, with a NULL string spelled `None`.
///
/// # Safety
/// `l` is a live list of `VAR_STRING` items.
unsafe fn strings(l: *mut neovim::types::List) -> Vec<Option<&'static str>> {
    (0..unsafe { tv_list_len(l) })
        .map(|at| {
            let s = unsafe { (*tv_list_find(l, at)).li_tv.string() };
            (!s.is_null()).then(|| unsafe { CStr::from_ptr(s) }.to_str().unwrap())
        })
        .collect()
}

/// `describe('dict') describe('item') describe('alloc()/free()')
/// itp('works')`, spec line 1682.
///
/// The spec's subject was the *arithmetic*: a `DictItem` was over-allocated
/// so the NUL-terminated key fitted in its flexible `di_key` member, and the
/// case asserted `offsetof(DictItem, di_key) + len + 1` as the malloc size.
/// There is no such allocation any more -- an item owns its key, short ones
/// in the item and long ones in their own box, and neither goes through the
/// `xmalloc` family the log records. What is left to state is what the
/// arithmetic was there to protect: the item carries back exactly the bytes
/// it was given, terminated, at every length either arm can hold.
#[test]
fn a_dict_item_owns_exactly_the_key_it_was_given() {
    let log = AllocLog::start();
    let long = "a_key_long_enough_to_need_its_own_allocation_and_then_some";
    for (key, len) in [
        ("", None),
        ("t", None),
        ("TEST", None),
        ("", Some(0)),
        ("TEST", Some(2)),
        // Either side of the boundary between the two arms.
        ("a_key_of_21_chars_xxx", None),
        ("a_key_of_22_characters", None),
        (long, None),
        (long, Some(9)),
    ] {
        // SAFETY: the item is this iteration's own and is freed below; the
        // key outlives the copy.
        unsafe {
            let c_key = cstr(key);
            let di = match len {
                None => tv_dict_item_alloc(c_key.as_ptr()),
                Some(len) => tv_dict_item_alloc_len(c_key.as_ptr(), len),
            };
            let len = len.unwrap_or(key.len());
            assert_eq!((*di).key_bytes(), &key.as_bytes()[..len], "{key:?}/{len}");
            assert_eq!((*di).key().to_bytes(), &key.as_bytes()[..len]);
            assert_eq!(
                (*di).di_tv.v_type(),
                VAR_UNKNOWN,
                "a fresh item holds nothing"
            );
            tv_dict_item_free(di);
            // Neither the item nor its key is an `xmalloc`: the log is
            // silent, and a leak would be Miri's or ASan's to report.
            log.check(&[]);
        }
    }
}

/// The same case's tail: an item holding a string frees the string too, and
/// in that order.
#[test]
fn freeing_a_dict_item_frees_its_value_first() {
    let log = AllocLog::start();
    // SAFETY: the string is handed to the item, which takes ownership of it;
    // `tv_dict_item_free` releases both.
    unsafe {
        let value = xstrdup(cstr("test").as_ptr());
        log.check(&[alloc::string(value, 4)]);

        let di = tv_dict_item_alloc(cstr("").as_ptr());
        log.check(&[]);
        (*di).di_tv = TypVal::String(value);

        tv_dict_item_free(di);
        log.check(&[alloc::freed(value)]);
    }
}

/// `describe('dict') describe('item') describe('add()/remove()')
/// itp('works')`, spec line 1697.
///
/// Adding an item transfers ownership without allocating; adding it twice is
/// an internal error and still allocates nothing; removing it releases the
/// value before the item.
#[test]
fn a_dict_item_is_added_by_move_and_removed_with_its_value() {
    let log = AllocLog::start();
    // SAFETY: the dict and the item are this case's own; the item is handed
    // to the dict, which then owns it, and the dict is freed at the end.
    unsafe {
        let d = tv_dict_alloc();
        log.check(&[alloc::dict(d)]);

        let di = tv_dict_item_alloc(cstr("").as_ptr());
        let value = xstrdup(cstr("test").as_ptr());
        (*di).di_tv = TypVal::String(value);
        log.check(&[alloc::string(value, 4)]);

        assert_eq!(tv_dict_add(d, di), Ok(()));
        log.check(&[]);

        // The same key again. The hashtab reports it and nothing is
        // allocated for the failure.
        let again = check_emsg(
            log.editor(),
            || tv_dict_add(d, di),
            Some(r#"E685: Internal error: hash_add(): duplicate key """#),
        );
        assert_eq!(again, Err(Failed));
        log.clear();

        tv_dict_item_remove(d, di);
        log.check(&[alloc::freed(value)]);

        // Freeing the now-empty dict releases the dict and nothing else —
        // an empty hashtab still lives in its own static array. Said through
        // `check_net`, the twin of the spec's `clear_tmp_allocs`: with every
        // matched allocate/release pair dropped, what remains is the release
        // of something allocated before this stretch of the log.
        tv_dict_free(d);
        log.check_net(true, &[alloc::freed(d)]);
    }
}

/// `tv_list_drop_items` unlinks a run of items and shortens the list.
///
/// **No differential can see this.** `string()` and all six encoder sinks
/// walk the *links*, and every `len()` in every sweep corpus is over a
/// literal, so a list whose cached `lv_len` an unlink got wrong renders
/// byte-identically — measured NOT CAUGHT by `evalsweep`
/// (`1787432513-typvalmutate.py --blind list-drop-len`). The length is
/// still what `len()` answers for a list any *runtime* code shortened.
#[test]
fn removing_a_run_shortens_the_list() {
    let _editor = editor_lock();
    // SAFETY: the list is this case's own, freed below.
    unsafe {
        let l = tv_list_alloc(kListLenUnknown as ptrdiff_t);
        for n in 1..=4 {
            tv_list_append_number(l, n);
        }
        assert_eq!(tv_list_len(l), 4);

        tv_list_remove_range(l, 1, 2);

        assert_eq!(tv_list_len(l), 2, "two of the four items were removed");
        assert_eq!((*tv_list_first(l)).li_tv.number(), 1);
        assert_eq!((*tv_list_last(l)).li_tv.number(), 4, "the gap closed");
        tv_list_unref(l);
    }
}

/// A `ListWatch` standing on an item that is being unlinked is advanced to
/// the item *after* it — what keeps `:for` and `filter()` walking a list
/// whose current item they just removed.
///
/// **No differential can see this either.** The only corpus row that removes
/// a watched item is `filter([1, 2, 3], 'v:val > 1')`, which removes the
/// *first* one; a watcher pushed backwards off the front is NULL, which ends
/// the walk with the same answer. Measured NOT CAUGHT by `evalsweep`.
#[test]
fn a_watcher_on_a_removed_item_advances_past_it() {
    let _editor = editor_lock();
    // SAFETY: as above; `lw` outlives its registration.
    unsafe {
        let l = tv_list_alloc(kListLenUnknown as ptrdiff_t);
        for n in 1..=3 {
            tv_list_append_number(l, n);
        }

        let mut lw = ListWatch {
            lw_index: 1,
            lw_next: ptr::null_mut(),
        };
        tv_list_watch_add(l, &raw mut lw);
        tv_list_remove_range(l, 1, 1);
        // Index 1 again -- but the item that *followed* the removed one,
        // which has shifted down into its place.
        assert_eq!(lw.lw_index, 1, "the watcher moved on, not back");
        assert_eq!((*tv_list_find(l, lw.lw_index)).li_tv.number(), 3);

        tv_list_watch_remove(l, &raw mut lw);
        tv_list_unref(l);
    }
}

/// `tv_dict_watcher_remove` matches a watcher on all three of its callback,
/// its pattern *length* and its pattern bytes.
///
/// **Nothing else in the tree reaches `watcher.rs` at all.** It is reachable
/// from Vimscript only through `dictwatcheradd()`/`dictwatcherdel()`, and no
/// sweep corpus calls either; `extend()`'s notify is a no-op when no watcher
/// is registered. Measured NOT CAUGHT by `evalsweep`.
#[test]
fn a_watcher_is_removed_only_by_its_own_pattern() {
    let _editor = editor_lock();
    // SAFETY: the dict is this case's own and is freed below; a
    // `kCallbackNone` callback owns nothing.
    unsafe {
        let d = tv_dict_alloc();
        let callback = Callback::None;
        let pattern = cstr("key*");
        tv_dict_watcher_add(d, pattern.as_ptr(), 4, callback.clone());
        assert!(tv_dict_is_watched(d));

        // A prefix of the pattern is not the pattern ...
        let shorter = cstr("key");
        assert!(!tv_dict_watcher_remove(d, shorter.as_ptr(), 3, &callback));
        assert!(tv_dict_is_watched(d), "a shorter pattern matched");

        // ... and neither are different bytes of the same length.
        let same_len = cstr("kex*");
        assert!(!tv_dict_watcher_remove(d, same_len.as_ptr(), 4, &callback));
        assert!(tv_dict_is_watched(d), "a different pattern matched");

        assert!(tv_dict_watcher_remove(d, pattern.as_ptr(), 4, &callback));
        assert!(!tv_dict_is_watched(d), "its own pattern did not match");
        tv_dict_free(d);
    }
}
