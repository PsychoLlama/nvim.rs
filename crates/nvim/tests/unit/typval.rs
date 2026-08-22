//! Cases lifted out of `test/unit/eval/typval_spec.lua`.
//!
//! The spec is 3,418 lines and 111 cases, 285 of which assert an exact
//! allocation sequence through the LuaJIT allocator seam. These three are the
//! proof that the seam has a Rust twin — one case per shape the rest of the
//! spec is built out of:
//!
//! - an allocation *sequence* whose order is the assertion
//!   (`tv_list_append_string`),
//! - a *size* derived from a struct's layout, which is the only evidence the
//!   over-allocation happened (`tv_dict_item_alloc`),
//! - and a case that allocates nothing but reads editor globals
//!   (`tv_get_lnum`, the one place in all 3,418 lines that mentions
//!   `curwin`).
//!
//! See `crate::support::alloc` for the porting rules; the short version is
//! that a size is always `size_of`/`offset_of!`, never a literal.
//!
//! Every case here needs a live editor, which Miri cannot start.

#![cfg(not(miri))]

use std::ffi::{CStr, c_char};
use std::ptr;

use c2rust_neovim::eval::typval::{
    kCallbackNone, tv_dict_add, tv_dict_alloc, tv_dict_free, tv_dict_is_watched,
    tv_dict_item_alloc, tv_dict_item_alloc_len, tv_dict_item_free, tv_dict_item_remove,
    tv_dict_watcher_add, tv_dict_watcher_remove, tv_get_lnum, tv_list_alloc, tv_list_append_number,
    tv_list_append_string, tv_list_drop_items, tv_list_first, tv_list_last, tv_list_len,
    tv_list_unref, tv_list_watch_add, tv_list_watch_remove,
};
use c2rust_neovim::main::curwin;
use c2rust_neovim::memory::{xfree, xstrdup};
use c2rust_neovim::types::{
    Callback, Callback_data, FAIL, OK, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST,
    VAR_NUMBER, VAR_PARTIAL, VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, kBoolVarFalse,
    kBoolVarTrue, kListLenUnknown, kSpecialVarNull, listitem_T, listwatch_T, ptrdiff_t, typval_T,
    typval_vval_union, win_T,
};

use crate::support::alloc::{self, AllocLog};
use crate::support::{check_emsg, cstr, editor_lock};

/// `describe('list') describe('append') describe('string()') itp('works')`,
/// spec line 663.
///
/// The assertion is the *order*: `tv_list_append_string` copies the string
/// before it allocates the item that will hold it.
#[test]
fn tv_list_append_string_copies_then_appends() {
    let log = AllocLog::start();
    // SAFETY: the list is this case's own, freed at the end; the strings
    // outlive every call that reads them.
    unsafe {
        let l = tv_list_alloc(kListLenUnknown as isize);
        log.check(&[alloc::list(l)]);

        let test = cstr("test");
        tv_list_append_string(l, test.as_ptr(), 3);
        log.check(&[
            alloc::string((*(*l).lv_last).li_tv.vval.v_string, 3),
            alloc::li((*l).lv_last),
        ]);

        // A NULL string allocates nothing but the item.
        tv_list_append_string(l, ptr::null(), 0);
        log.check(&[alloc::li((*l).lv_last)]);
        tv_list_append_string(l, ptr::null(), -1);
        log.check(&[alloc::li((*l).lv_last)]);

        // A negative length means "to the terminator".
        tv_list_append_string(l, test.as_ptr(), -1);
        log.check(&[
            alloc::string((*(*l).lv_last).li_tv.vval.v_string, 4),
            alloc::li((*l).lv_last),
        ]);

        assert_eq!(strings(l), [Some("tes"), None, None, Some("test")]);

        // The spec left this to a LuaJIT finalizer, so it never said what
        // freeing costs. It is worth saying: each item releases its string
        // — a NULL one included, since `xfree(NULL)` still reaches the
        // allocator — before itself, and the list goes last.
        let items: Vec<(*mut c_char, *mut listitem_T)> = {
            let mut items = Vec::new();
            let mut item = (*l).lv_first;
            while !item.is_null() {
                items.push(((*item).li_tv.vval.v_string, item));
                item = (*item).li_next;
            }
            items
        };
        let mut expected: Vec<_> = items
            .iter()
            .flat_map(|&(string, item)| [alloc::freed(string), alloc::freed(item)])
            .collect();
        expected.push(alloc::freed(l));
        tv_list_unref(l);
        log.check(&expected);
    }
}

/// The list's items as UTF-8, with a NULL string spelled `None`.
///
/// # Safety
/// `l` is a live list of `VAR_STRING` items.
unsafe fn strings(l: *const c2rust_neovim::types::list_T) -> Vec<Option<&'static str>> {
    let mut out = Vec::new();
    let mut item = unsafe { (*l).lv_first };
    while !item.is_null() {
        let s = unsafe { (*item).li_tv.vval.v_string };
        out.push((!s.is_null()).then(|| unsafe { CStr::from_ptr(s) }.to_str().unwrap()));
        item = unsafe { (*item).li_next };
    }
    out
}

/// `describe('dict') describe('item') describe('alloc()/free()')
/// itp('works')`, spec line 1682.
///
/// A `dictitem_T` is over-allocated so the NUL-terminated key fits in its
/// flexible `di_key` member — but never below the struct's own size. The
/// expectation is the *arithmetic*, so it is written as
/// `offset_of!(dictitem_T, di_key) + len + 1` exactly as the Lua spelled it
/// `ffi.offsetof(...)`, and it would not survive being written as a number.
#[test]
fn tv_dict_item_is_allocated_around_its_key() {
    let log = AllocLog::start();
    for (key, len) in [
        ("", None),
        ("t", None),
        ("TEST", None),
        ("", Some(0)),
        ("TEST", Some(2)),
    ] {
        // SAFETY: the item is this iteration's own and is freed below; the
        // key outlives the allocation that copies it.
        unsafe {
            let c_key = cstr(key);
            let di = match len {
                None => tv_dict_item_alloc(c_key.as_ptr()),
                Some(len) => tv_dict_item_alloc_len(c_key.as_ptr(), len),
            };
            let len = len.unwrap_or(key.len());
            assert_eq!(
                CStr::from_ptr((&raw const (*di).di_key).cast()).to_bytes(),
                &key.as_bytes()[..len],
            );
            log.check(&[alloc::di(di, len)]);

            (*di).di_tv.v_type = VAR_UNKNOWN;
            tv_dict_item_free(di);
            log.check(&[alloc::freed(di)]);
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
        log.check(&[alloc::di(di, 0)]);
        (*di).di_tv.v_type = VAR_STRING;
        (*di).di_tv.v_lock = VAR_UNLOCKED;
        (*di).di_tv.vval.v_string = value;

        tv_dict_item_free(di);
        log.check(&[alloc::freed(value), alloc::freed(di)]);
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
        (*di).di_tv.v_type = VAR_STRING;
        (*di).di_tv.v_lock = VAR_UNLOCKED;
        (*di).di_tv.vval.v_string = value;
        log.check(&[alloc::di(di, 0), alloc::string(value, 4)]);

        assert_eq!(tv_dict_add(d, di), OK);
        log.check(&[]);

        // The same key again. The hashtab reports it and nothing is
        // allocated for the failure.
        let again = check_emsg(
            log.editor(),
            || tv_dict_add(d, di),
            Some(r#"E685: Internal error: hash_add(): duplicate key """#),
        );
        assert_eq!(again, FAIL);
        log.clear();

        tv_dict_item_remove(d, di);
        log.check(&[alloc::freed(value), alloc::freed(di)]);

        // Freeing the now-empty dict releases the dict and nothing else —
        // an empty hashtab still lives in its own static array. Said through
        // `check_net`, the twin of the spec's `clear_tmp_allocs`: with every
        // matched allocate/release pair dropped, what remains is the release
        // of something allocated before this stretch of the log.
        tv_dict_free(d);
        log.check_net(true, &[alloc::freed(d)]);
    }
}

/// `describe('get') describe('lnum()') itp('works')`, spec line 3204.
///
/// The only case in the spec that touches `curwin`: a `"."` resolves through
/// `var2fpos` to the cursor's line, which is what pins `win_T` for the whole
/// file. Everything else here allocates nothing, and the case says so.
#[test]
fn tv_get_lnum_resolves_the_cursor_and_reports_the_rest() {
    let log = AllocLog::start();
    // A window is all `var2fpos` needs for `"."`; it never reaches the
    // buffer on that path.
    let mut win: Box<win_T> = Box::new(unsafe { std::mem::zeroed() });
    let saved_curwin = curwin.get();
    curwin.set(&raw mut *win);

    let dot = cstr(".");
    let number = cstr("100500");
    let cases: [(u32, typval_vval_union, Option<&str>, i64); 12] = [
        (VAR_NUMBER, typval_vval_union { v_number: 42 }, None, 42),
        (
            VAR_STRING,
            typval_vval_union {
                v_string: number.as_ptr().cast_mut(),
            },
            None,
            100500,
        ),
        (
            VAR_STRING,
            typval_vval_union {
                v_string: dot.as_ptr().cast_mut(),
            },
            None,
            46,
        ),
        (
            VAR_FLOAT,
            typval_vval_union { v_float: 42.53 },
            Some("E805: Using a Float as a Number"),
            -1,
        ),
        (
            VAR_PARTIAL,
            typval_vval_union {
                v_partial: ptr::null_mut(),
            },
            Some("E703: Using a Funcref as a Number"),
            -1,
        ),
        (
            VAR_FUNC,
            typval_vval_union {
                v_string: ptr::null_mut(),
            },
            Some("E703: Using a Funcref as a Number"),
            -1,
        ),
        (
            VAR_LIST,
            typval_vval_union {
                v_list: ptr::null_mut(),
            },
            Some("E745: Using a List as a Number"),
            -1,
        ),
        (
            VAR_DICT,
            typval_vval_union {
                v_dict: ptr::null_mut(),
            },
            Some("E728: Using a Dictionary as a Number"),
            -1,
        ),
        (
            VAR_SPECIAL,
            typval_vval_union {
                v_special: kSpecialVarNull,
            },
            None,
            0,
        ),
        (
            VAR_BOOL,
            typval_vval_union {
                v_bool: kBoolVarTrue,
            },
            None,
            1,
        ),
        (
            VAR_BOOL,
            typval_vval_union {
                v_bool: kBoolVarFalse,
            },
            None,
            0,
        ),
        (
            VAR_UNKNOWN,
            typval_vval_union { v_number: 0 },
            Some("E685: Internal error: tv_get_number(UNKNOWN)"),
            -1,
        ),
    ];

    for (v_type, vval, emsg, expected) in cases {
        win.w_cursor.lnum = 46;
        let tv = typval_T {
            v_type,
            v_lock: VAR_UNLOCKED,
            vval,
        };
        log.check(&[]);
        // SAFETY: `tv` is a value this case owns and does not free; the
        // editor lock is held by `log`.
        let got = check_emsg(log.editor(), || unsafe { tv_get_lnum(&raw const tv) }, emsg);
        assert_eq!(i64::from(got), expected, "{v_type:?} {emsg:?}");
        if emsg.is_some() {
            // Reporting the error allocated; the spec does not describe what.
            log.clear();
        } else {
            log.check(&[]);
        }
    }

    curwin.set(saved_curwin);
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
fn dropping_items_shortens_the_list() {
    let _editor = editor_lock();
    // SAFETY: the list and its items are this case's own, freed below.
    unsafe {
        let l = tv_list_alloc(kListLenUnknown as ptrdiff_t);
        for n in 1..=4 {
            tv_list_append_number(l, n);
        }
        assert_eq!(tv_list_len(l), 4);

        let second = (*tv_list_first(l)).li_next;
        let third = (*second).li_next;
        tv_list_drop_items(l, second, third);

        assert_eq!(tv_list_len(l), 2, "two of the four items were unlinked");
        let first = tv_list_first(l);
        let last = tv_list_last(l);
        assert_eq!((*first).li_next, last, "the gap closed forwards");
        assert_eq!((*last).li_prev, first, "and backwards");

        // `drop` does not free; these two are ours now. They hold numbers,
        // so there is nothing to clear.
        xfree(second.cast());
        xfree(third.cast());
        tv_list_unref(l);
    }
}

/// A `listwatch_T` standing on an item that is being unlinked is advanced to
/// the item *after* it — what keeps `:for` and `filter()` walking a list
/// whose current item they just removed.
///
/// **No differential can see this either.** The only corpus row that removes
/// a watched item is `filter([1, 2, 3], 'v:val > 1')`, which removes the
/// *first* one; a watcher pushed backwards off the front is NULL, which ends
/// the walk with the same answer. Measured NOT CAUGHT by `evalsweep`.
#[test]
fn a_watcher_on_a_dropped_item_advances_past_it() {
    let _editor = editor_lock();
    // SAFETY: as above; `lw` outlives its registration.
    unsafe {
        let l = tv_list_alloc(kListLenUnknown as ptrdiff_t);
        for n in 1..=3 {
            tv_list_append_number(l, n);
        }
        let second = (*tv_list_first(l)).li_next;
        let third = (*second).li_next;

        let mut lw = listwatch_T {
            lw_item: second,
            lw_next: ptr::null_mut(),
        };
        tv_list_watch_add(l, &raw mut lw);
        tv_list_drop_items(l, second, second);
        assert_eq!(lw.lw_item, third, "the watcher moved on, not back");

        tv_list_watch_remove(l, &raw mut lw);
        xfree(second.cast());
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
        let callback = Callback {
            data: Callback_data { luaref: 0 },
            type_0: kCallbackNone,
        };
        let pattern = cstr("key*");
        tv_dict_watcher_add(d, pattern.as_ptr(), 4, callback);
        assert!(tv_dict_is_watched(d));

        // A prefix of the pattern is not the pattern ...
        let shorter = cstr("key");
        assert!(!tv_dict_watcher_remove(d, shorter.as_ptr(), 3, callback));
        assert!(tv_dict_is_watched(d), "a shorter pattern matched");

        // ... and neither are different bytes of the same length.
        let same_len = cstr("kex*");
        assert!(!tv_dict_watcher_remove(d, same_len.as_ptr(), 4, callback));
        assert!(tv_dict_is_watched(d), "a different pattern matched");

        assert!(tv_dict_watcher_remove(d, pattern.as_ptr(), 4, callback));
        assert!(!tv_dict_is_watched(d), "its own pattern did not match");
        tv_dict_free(d);
    }
}
