//! `describe('dict')` from `test/unit/eval/typval_spec.lua`.
//!
//! See `typval_list` for the shape; `describe('item')`'s two cases live in
//! `typval` beside `tv_get_lnum`, where they landed first.
//!
//! Every case needs a live editor, which Miri cannot start.

#![cfg(not(miri))]

use std::ffi::{CStr, c_char};
use std::ptr;

use neovim::buffer::{DI_FLAGS_FIX, DI_FLAGS_RO, DI_FLAGS_RO_SBX};
use neovim::eval::typval::{
    callback_free, tv_clear, tv_dict_add, tv_dict_add_allocated_str, tv_dict_add_dict,
    tv_dict_add_float, tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc,
    tv_dict_clear, tv_dict_copy, tv_dict_equal, tv_dict_extend, tv_dict_find, tv_dict_free,
    tv_dict_get_callback, tv_dict_get_number, tv_dict_get_string_alloc, tv_dict_get_string_buf,
    tv_dict_get_string_buf_chk, tv_dict_item_alloc_len, tv_dict_set_keys_readonly, tv_dict_unref,
    tv_dict_watcher_add, tv_dict_watcher_remove, tv_list_unref,
};
use neovim::main::{emsg_skip, sandbox};
use neovim::mbyte::convert_setup;
use neovim::memory::{xfree, xmalloc, xstrdup};
use neovim::ops::NUMBUFLEN;
use neovim::types::{Callback, Failed, OK, VarLock, dict_T, vimconv_T};

use crate::support::alloc::{self, AllocLog};
use crate::support::tv::{self, Cb, Pt, Tv};
use crate::support::{check_emsg, cstr};

/// The spec's bare Lua numbers, which `lua2typvalt` made floats.
fn f(n: f64) -> Tv {
    Tv::Float(n)
}

/// `E685` for a key that is already there, as `hash_add` words it.
fn duplicate(key: &str) -> String {
    format!(r#"E685: Internal error: hash_add(): duplicate key "{key}""#)
}

// ------------------------------------------------------------- watcher

/// `describe('watcher') describe('add()/remove()')
/// itp('works with an empty key')`, spec line 1521.
///
/// A zero-length pattern matches every key, which is why removing with a
/// *different* pattern of the same length succeeds.
#[test]
fn a_zero_length_watch_pattern_matches_anything() {
    let log = AllocLog::start();
    // SAFETY: the dict and the callback are this case's own.
    unsafe {
        let d = tv::new_dict(&[]);
        assert_eq!(tv::dict_watchers(d), []);
        let cb = tv::build_callback(&Cb::None);
        log.clear();

        tv_dict_watcher_add(d, cstr("*").as_ptr(), 0, cb.clone());
        let ws = tv::dict_watchers(d);
        log.check(&[alloc::dwatcher(ws[0].at), alloc::string(ws[0].pattern, 0)]);
        assert_eq!(ws[0].pat, b"");
        assert_eq!(ws[0].cb, Cb::None);
        assert!(!ws[0].busy);

        assert!(tv_dict_watcher_remove(d, cstr("x").as_ptr(), 0, &cb));
        log.check(&[alloc::freed(ws[0].pattern), alloc::freed(ws[0].at)]);
        assert_eq!(tv::dict_watchers(d), []);

        tv_dict_free(d);
    }
}

/// The same `describe`'s `itp('works with multiple callbacks')`, spec line
/// 1541 — the whole of what a `DictWatcher` costs, and the whole of what
/// releasing one gives back.
#[test]
fn watchers_are_removed_one_at_a_time_with_what_they_hold() {
    let log = AllocLog::start();
    // SAFETY: every callback is handed to the dict, which frees it.
    unsafe {
        let d = tv::new_dict(&[]);
        assert_eq!(tv::dict_watchers(d), []);
        log.check(&[alloc::dict(d)]);

        // A `kCallbackNone` owns nothing.
        let none = tv::build_callback(&Cb::None);
        log.check(&[]);

        // A funcref owns its name.
        let fref = tv::build_callback(&Cb::Fref(b"tr".to_vec()));
        log.check(&[alloc::string(fref.data.funcref, "tr".len())]);

        // A partial owns its argument vector, each argument, its dict and
        // its name — allocated in that order.
        let partial = tv::build_callback(&Cb::Pt(Box::new(Pt {
            value: b"tr".to_vec(),
            auto: false,
            args: vec![Tv::s("test")],
            dict: Some(Tv::Dict(vec![])),
        })));
        let pt = partial.data.partial;
        let pt_argv = (*pt).pt_argv;
        let pt_dict = (*pt).pt_dict;
        let pt_name = (*pt).pt_name;
        let pt_arg = (*pt_argv).vval.v_string;
        log.check(&[
            alloc::partial(pt),
            alloc::argv(pt_argv, 1),
            alloc::string(pt_arg, "test".len()),
            alloc::dict(pt_dict),
            alloc::string(pt_name, "tr".len()),
        ]);

        let registered = [("te", none), ("foo", fref), ("te", partial)];
        for (pattern, cb) in registered.clone() {
            tv_dict_watcher_add(d, cstr(pattern).as_ptr(), pattern.len(), cb);
        }
        let ws = tv::dict_watchers(d);
        assert_eq!(
            ws.iter()
                .map(|w| (w.pat.clone(), w.cb.clone(), w.busy))
                .collect::<Vec<_>>(),
            vec![
                (b"te".to_vec(), Cb::None, false),
                (b"foo".to_vec(), Cb::Fref(b"tr".to_vec()), false),
                (
                    b"te".to_vec(),
                    Cb::Pt(Box::new(Pt {
                        value: b"tr".to_vec(),
                        auto: false,
                        args: vec![Tv::s("test")],
                        dict: Some(Tv::Dict(vec![])),
                    })),
                    false,
                ),
            ]
        );
        log.check(&[
            alloc::dwatcher(ws[0].at),
            alloc::string(ws[0].pattern, "te".len()),
            alloc::dwatcher(ws[1].at),
            alloc::string(ws[1].pattern, "foo".len()),
            alloc::dwatcher(ws[2].at),
            alloc::string(ws[2].pattern, "te".len()),
        ]);

        // The funcref: its name, its pattern, itself.
        assert!(tv_dict_watcher_remove(
            d,
            cstr("foo").as_ptr(),
            3,
            &registered[1].1
        ));
        log.check(&[
            alloc::freed(registered[1].1.data.funcref),
            alloc::freed(ws[1].pattern),
            alloc::freed(ws[1].at),
        ]);
        assert!(!tv_dict_watcher_remove(
            d,
            cstr("foo").as_ptr(),
            3,
            &registered[1].1
        ));
        assert_eq!(tv::dict_watchers(d).len(), 2);

        // The partial: everything it owns, innermost first.
        assert!(tv_dict_watcher_remove(
            d,
            cstr("te").as_ptr(),
            2,
            &registered[2].1
        ));
        log.check(&[
            alloc::freed(pt_arg),
            alloc::freed(pt_argv),
            alloc::freed(pt_dict),
            alloc::freed(pt_name),
            alloc::freed(pt),
            alloc::freed(ws[2].pattern),
            alloc::freed(ws[2].at),
        ]);
        assert!(!tv_dict_watcher_remove(
            d,
            cstr("te").as_ptr(),
            2,
            &registered[2].1
        ));
        assert_eq!(tv::dict_watchers(d).len(), 1);

        // And the one that owns nothing.
        assert!(tv_dict_watcher_remove(
            d,
            cstr("te").as_ptr(),
            2,
            &registered[0].1
        ));
        log.check(&[alloc::freed(ws[0].pattern), alloc::freed(ws[0].at)]);
        assert!(!tv_dict_watcher_remove(
            d,
            cstr("te").as_ptr(),
            2,
            &registered[0].1
        ));
        assert_eq!(tv::dict_watchers(d), []);

        tv_dict_free(d);
    }
}

// ------------------------------------------------------------ indexing

/// The dict the `find()` and `get_string_buf*` cases index into, and its
/// value as a `Tv`.
fn prefix_dict() -> Vec<(&'static str, Tv)> {
    vec![
        ("", f(0.0)),
        ("t", f(1.0)),
        ("te", f(2.0)),
        ("tes", f(3.0)),
        ("test", f(4.0)),
        ("testt", f(5.0)),
    ]
}

/// `describe('indexing') describe('find()') itp('works with NULL dict')`,
/// spec line 1731.
#[test]
fn finding_in_a_null_dict_answers_nothing() {
    let _log = AllocLog::start();
    // SAFETY: no dict is dereferenced.
    unsafe {
        assert!(tv_dict_find(ptr::null(), cstr("").as_ptr(), 0).is_null());
        assert!(tv_dict_find(ptr::null(), cstr("test").as_ptr(), -1).is_null());
        assert!(tv_dict_find(ptr::null(), ptr::null(), 0).is_null());
    }
}

/// The same `describe`'s two `itp`s at spec lines 1736 and 1752: the empty
/// key is a key, and `len` decides how much of the key is read.
#[test]
fn finding_reads_exactly_the_key_length_asked_for() {
    let log = AllocLog::start();
    // SAFETY: the dict is this case's own.
    unsafe {
        let entries = prefix_dict();
        let d = tv::new_dict(&entries);
        log.clear();
        assert_eq!(
            tv::read_dict(d),
            Tv::Dict(
                entries
                    .iter()
                    .map(|(k, v)| (k.as_bytes().to_vec(), v.clone()))
                    .collect()
            )
        );
        log.check(&[]);

        let find = |key: &str, len: isize| -> Option<(Tv, Vec<u8>)> {
            let di = tv_dict_find(d, cstr(key).as_ptr(), len);
            (!di.is_null()).then(|| {
                (
                    tv::read(&raw const (*di).di_tv),
                    CStr::from_ptr((&raw const (*di).di_key).cast())
                        .to_bytes()
                        .to_vec(),
                )
            })
        };

        assert_eq!(find("", 0), Some((f(0.0), b"".to_vec())));
        for i in 0..=5 {
            assert_eq!(
                find("testt", i),
                Some((
                    f(f64::from(i32::try_from(i).unwrap())),
                    b"testt"[..i as usize].to_vec()
                )),
                "length {i}"
            );
        }
        // Six bytes of `testt` is five bytes and the terminator.
        assert_eq!(find("testt", 6), None);
        // A negative length reads to the terminator.
        assert_eq!(find("testt", -1), Some((f(5.0), b"testt".to_vec())));
        log.check(&[]);

        tv_dict_free(d);
    }
}

/// `describe('get_number()')`'s two `itp`s, spec lines 1775 and 1783: a
/// NULL dict and a missing key are both zero, a string is read as a number,
/// and anything else reports.
#[test]
fn getting_a_number_reads_through_strings_and_reports_otherwise() {
    let log = AllocLog::start();
    // SAFETY: every dict is this case's own.
    unsafe {
        let get = |d: *const dict_T, key: &str, msg: Option<&str>| {
            check_emsg(
                log.editor(),
                || tv_dict_get_number(d, cstr(key).as_ptr()),
                msg,
            )
        };

        assert_eq!(get(ptr::null(), "test", None), 0);

        let d = tv::new_dict(&[("test", Tv::Dict(vec![]))]);
        assert_eq!(
            get(d, "test", Some("E728: Using a Dictionary as a Number")),
            0
        );
        tv_dict_free(d);
        log.clear();

        let d = tv::new_dict(&[("tes", Tv::Int(42)), ("t", f(44.0)), ("te", Tv::s("43"))]);
        log.clear();
        assert_eq!(get(d, "test", None), 0, "a missing key is zero");
        assert_eq!(get(d, "tes", None), 42);
        assert_eq!(get(d, "te", None), 43, "a string is read as a number");
        log.check(&[]);
        assert_eq!(get(d, "t", Some("E805: Using a Float as a Number")), 0);
        log.clear();
        tv_dict_free(d);
    }
}

/// `describe('get_string()') itp('works')`, spec line 1829.
///
/// The answer for a scalar is rendered into the buffer the caller lends, so
/// two scalar lookups given two buffers stay independent -- the C's one
/// process-wide buffer let the second overwrite the first.
#[test]
fn getting_a_string_renders_a_scalar_into_the_lent_buffer() {
    let log = AllocLog::start();
    // SAFETY: every dict is this case's own; the answers are borrowed and
    // every buffer lent outlives the answer taken from it.
    unsafe {
        let get = |d: *const dict_T, key: &str, msg: Option<&str>, buf: &mut [c_char; 65]| {
            check_emsg(
                log.editor(),
                || tv_dict_get_string_buf(d, cstr(key).as_ptr(), buf.as_mut_ptr()),
                msg,
            )
        };
        let mut buf1 = [0 as c_char; 65];
        let mut buf2 = [0 as c_char; 65];
        let text = |p: *const c_char| CStr::from_ptr(p).to_string_lossy().into_owned();

        assert!(get(ptr::null(), "test", None, &mut buf1).is_null());

        let d = tv::new_dict(&[("test", Tv::Dict(vec![]))]);
        assert_eq!(
            text(get(
                d,
                "test",
                Some("E731: Using a Dictionary as a String"),
                &mut buf1
            )),
            ""
        );
        tv_dict_free(d);
        log.clear();

        let d = tv::new_dict(&[
            ("tes", Tv::Int(42)),
            ("t", f(44.0)),
            ("te", Tv::s("43")),
            ("xx", Tv::Int(45)),
        ]);
        log.clear();

        assert!(
            get(d, "test", None, &mut buf1).is_null(),
            "a missing key is NULL"
        );
        let s42 = get(d, "tes", None, &mut buf1);
        assert_eq!(text(s42), "42");
        let s45 = get(d, "xx", None, &mut buf2);
        assert_ne!(s45, s42, "a buffer each");
        assert_eq!(text(s45), "45");
        assert_eq!(text(s42), "42", "and the first answer is still there");

        // A string item is answered in place, not through the buffer.
        let s43 = get(d, "te", None, &mut buf1);
        assert_eq!(text(s43), "43");
        assert_ne!(s43, s42);
        assert_eq!(s43, (*tv::di_of(d, "te")).di_tv.vval.v_string);
        log.check(&[]);

        // Rendering a float goes through `vim_snprintf`, which costs two
        // `free(NULL)`s and nothing else.
        assert_eq!(text(get(d, "t", None, &mut buf1)), "44.0");
        log.check(&[
            alloc::freed(ptr::null::<u8>()),
            alloc::freed(ptr::null::<u8>()),
        ]);

        tv_dict_free(d);
    }
}

/// The same `describe`'s `itp('allocates a string copy when requested')`,
/// spec line 1869: with `save` the answer is the caller's.
#[test]
fn getting_a_string_with_save_allocates_the_answer() {
    let log = AllocLog::start();
    // SAFETY: every answer with `save` is freed here.
    unsafe {
        let get = |d: *const dict_T, key: &str, msg: Option<&str>, is_float: bool| {
            log.clear();
            let ret = check_emsg(
                log.editor(),
                || tv_dict_get_string_alloc(d, cstr(key).as_ptr()),
                msg,
            );
            let answer =
                (!ret.is_null()).then(|| CStr::from_ptr(ret).to_string_lossy().into_owned());
            if msg.is_none() {
                match &answer {
                    None => log.check(&[]),
                    Some(s) if is_float => log.check(&[
                        alloc::freed(ptr::null::<u8>()),
                        alloc::freed(ptr::null::<u8>()),
                        alloc::string(ret, s.len()),
                    ]),
                    Some(s) => log.check(&[alloc::string(ret, s.len())]),
                }
            }
            xfree(ret.cast());
            answer
        };

        let d = tv::new_dict(&[("test", Tv::Dict(vec![]))]);
        assert_eq!(
            get(
                d,
                "test",
                Some("E731: Using a Dictionary as a String"),
                false
            )
            .as_deref(),
            Some("")
        );
        tv_dict_free(d);
        log.clear();

        let d = tv::new_dict(&[
            ("tes", Tv::Int(42)),
            ("t", f(44.0)),
            ("te", Tv::s("43")),
            ("xx", Tv::Int(45)),
        ]);
        assert_eq!(get(d, "test", None, false), None);
        assert_eq!(get(d, "tes", None, false).as_deref(), Some("42"));
        assert_eq!(get(d, "xx", None, false).as_deref(), Some("45"));
        assert_eq!(get(d, "te", None, false).as_deref(), Some("43"));
        assert_eq!(get(d, "t", None, true).as_deref(), Some("44.0"));
        tv_dict_free(d);
    }
}

/// `describe('get_string_buf()')`'s two `itp`s, spec lines 1922 and 1925:
/// the caller's buffer is used for a scalar and bypassed for a string.
#[test]
fn getting_a_string_into_a_buffer_uses_it_only_for_scalars() {
    let log = AllocLog::start();
    // SAFETY: the buffer and the dict are this case's own.
    unsafe {
        let buf: *mut c_char = xmalloc(NUMBUFLEN as usize).cast();
        let get = |d: *const dict_T, key: &str, is_float: bool| -> Option<(String, bool)> {
            log.clear();
            let ret = check_emsg(
                log.editor(),
                || tv_dict_get_string_buf(d, cstr(key).as_ptr(), buf),
                None,
            );
            if is_float {
                log.check(&[
                    alloc::freed(ptr::null::<u8>()),
                    alloc::freed(ptr::null::<u8>()),
                ]);
            } else {
                log.check(&[]);
            }
            (!ret.is_null()).then(|| {
                (
                    CStr::from_ptr(ret).to_string_lossy().into_owned(),
                    ret == buf.cast_const(),
                )
            })
        };

        assert_eq!(get(ptr::null(), "test", false), None);

        let entries: Vec<(&str, Tv)> = vec![
            ("", Tv::Dict(vec![])),
            ("t", f(1.0)),
            ("te", Tv::Int(2)),
            ("tes", Tv::List(vec![])),
            ("test", Tv::s("tset")),
            ("testt", f(5.0)),
        ];
        let d = tv::new_dict(&entries);
        log.clear();
        assert_eq!(
            tv::read_dict(d),
            Tv::Dict(
                entries
                    .iter()
                    .map(|(k, v)| (k.as_bytes().to_vec(), v.clone()))
                    .collect()
            )
        );
        log.check(&[]);

        assert_eq!(get(d, "test", false), Some(("tset".into(), false)));
        assert_eq!(get(d, "t", true), Some(("1.0".into(), true)));
        assert_eq!(get(d, "te", false), Some(("2".into(), true)));

        tv_dict_free(d);
        xfree(buf.cast());
    }
}

/// `describe('get_string_buf_chk()')`'s two `itp`s, spec lines 1969 and
/// 1972: the same, plus a default for a key that is not there.
#[test]
fn getting_a_checked_string_falls_back_to_the_default() {
    let log = AllocLog::start();
    // SAFETY: the buffer, the default and the dict are this case's own.
    unsafe {
        let buf: *mut c_char = xmalloc(NUMBUFLEN as usize).cast();
        let def = xstrdup(cstr("DEFAULT").as_ptr());
        let get = |d: *const dict_T,
                   key: &str,
                   len: Option<isize>,
                   is_float: bool|
         -> Option<(String, bool, bool)> {
            let len = len.unwrap_or_else(|| isize::try_from(key.len()).unwrap());
            log.clear();
            let ret = check_emsg(
                log.editor(),
                || tv_dict_get_string_buf_chk(d, cstr(key).as_ptr(), len, buf, def),
                None,
            );
            if is_float {
                log.check(&[
                    alloc::freed(ptr::null::<u8>()),
                    alloc::freed(ptr::null::<u8>()),
                ]);
            } else {
                log.check(&[]);
            }
            (!ret.is_null()).then(|| {
                (
                    CStr::from_ptr(ret).to_string_lossy().into_owned(),
                    ret == buf.cast_const(),
                    ret == def.cast_const(),
                )
            })
        };

        assert_eq!(
            get(ptr::null(), "test", None, false),
            Some(("DEFAULT".into(), false, true))
        );

        let d = tv::new_dict(&[
            ("", Tv::Dict(vec![])),
            ("t", f(1.0)),
            ("te", Tv::Int(2)),
            ("tes", Tv::List(vec![])),
            ("test", Tv::s("tset")),
            ("testt", f(5.0)),
        ]);
        log.clear();

        assert_eq!(
            get(d, "test", None, false),
            Some(("tset".into(), false, false))
        );
        // One byte of `test` is the key `t`, whose float goes to the buffer.
        assert_eq!(
            get(d, "test", Some(1), true),
            Some(("1.0".into(), true, false))
        );
        assert_eq!(get(d, "te", None, false), Some(("2".into(), true, false)));
        assert_eq!(
            get(d, "TEST", None, false),
            Some(("DEFAULT".into(), false, true)),
            "keys are case-sensitive"
        );

        tv_dict_free(d);
        xfree(buf.cast());
        xfree(def.cast());
    }
}

/// `describe('get_callback()')`'s two `itp`s, spec lines 2016 and 2019.
///
/// A missing key answers true with `kCallbackNone`; a value that is neither
/// a function nor a string answers false and reports.
#[test]
fn getting_a_callback_accepts_a_name_a_funcref_or_a_partial() {
    let log = AllocLog::start();
    // SAFETY: each callback is released before the next lookup reuses the
    // slot; the dict is this case's own.
    unsafe {
        let get = |d: *mut dict_T, key: &str, len: isize, msg: Option<&str>| -> (Cb, bool) {
            let slot: *mut Callback = xmalloc(size_of::<Callback>()).cast();
            log.clear();
            let ok = check_emsg(
                log.editor(),
                || tv_dict_get_callback(d, cstr(key).as_ptr(), len, slot),
                msg,
            );
            let cb = tv::read_callback(slot);
            callback_free(slot);
            xfree(slot.cast());
            (cb, ok)
        };

        assert_eq!(get(ptr::null_mut(), "", 0, None), (Cb::None, true));

        let with_dict = |args: Vec<Tv>| {
            Tv::Partial(Box::new(Pt {
                value: b"Test".to_vec(),
                auto: false,
                args,
                dict: Some(Tv::dict([("test", f(1.0))])),
            }))
        };
        let entries: Vec<(&str, Tv)> = vec![
            ("", Tv::s("tr")),
            ("t", Tv::Int(1)),
            ("te", Tv::Func(b"tr".to_vec())),
            (
                "tes",
                Tv::Partial(Box::new(Pt {
                    value: b"tr".to_vec(),
                    auto: false,
                    args: vec![Tv::s("a"), Tv::s("b")],
                    dict: None,
                })),
            ),
            ("test", with_dict(vec![])),
            ("testt", with_dict(vec![f(1.0)])),
        ];
        let d = tv::new_dict(&entries);
        assert_eq!(
            tv::read_dict(d),
            Tv::Dict(
                entries
                    .iter()
                    .map(|(k, v)| (k.as_bytes().to_vec(), v.clone()))
                    .collect()
            )
        );

        // The empty key holds the *string* `tr`, which is a function name.
        assert_eq!(get(d, "", -1, None), (Cb::Fref(b"tr".to_vec()), true));
        // A missing key leaves the slot alone and still succeeds.
        assert_eq!(get(d, "x", -1, None), (Cb::None, true));
        // `key_len` picks which of the six prefixes is looked up.
        assert_eq!(get(d, "testt", 0, None), (Cb::Fref(b"tr".to_vec()), true));
        assert_eq!(
            get(
                d,
                "test",
                1,
                Some("E6000: Argument is not a function or function name")
            ),
            (Cb::None, false),
            "the key `t` holds a number"
        );
        assert_eq!(get(d, "testt", 2, None), (Cb::Fref(b"tr".to_vec()), true));
        assert_eq!(
            get(d, "testt", 3, None),
            (
                Cb::Pt(Box::new(Pt {
                    value: b"tr".to_vec(),
                    auto: false,
                    args: vec![Tv::s("a"), Tv::s("b")],
                    dict: None,
                })),
                true
            )
        );
        for (len, args) in [(4, vec![]), (5, vec![f(1.0)])] {
            assert_eq!(
                get(d, "testt", len, None),
                (
                    Cb::Pt(Box::new(Pt {
                        value: b"Test".to_vec(),
                        auto: false,
                        args,
                        dict: Some(Tv::dict([("test", f(1.0))])),
                    })),
                    true
                ),
                "key length {len}"
            );
        }

        tv_dict_free(d);
    }
}

// ----------------------------------------------------------------- add

/// `describe('add') describe('()') itp('works')`, spec line 2075.
#[test]
fn adding_an_item_transfers_it_and_refuses_a_duplicate() {
    let log = AllocLog::start();
    // SAFETY: the item is handed to the dict, which frees it.
    unsafe {
        let di = tv_dict_item_alloc_len(cstr("t-est").as_ptr(), 5);
        log.check(&[alloc::di(di, "t-est".len())]);
        (*di).di_tv = Tv::Int(42).build();

        let d = tv::new_dict(&[("test", f(10.0))]);
        log.check(&[
            alloc::dict(d),
            alloc::di(tv::di_of(d, "test"), "test".len()),
        ]);
        assert_eq!(tv::read_dict(d), Tv::dict([("test", f(10.0))]));
        log.clear();

        assert_eq!(tv_dict_add(d, di), Ok(()));
        log.check(&[]);
        assert_eq!(
            tv::read_dict(d),
            Tv::dict([("t-est", Tv::Int(42)), ("test", f(10.0))])
        );

        assert_eq!(
            check_emsg(
                log.editor(),
                || tv_dict_add(d, di),
                Some(&duplicate("t-est"))
            ),
            Err(Failed)
        );

        log.clear();
        tv_dict_free(d);
    }
}

/// The `add > list/dict/nr/float/str/allocated_str` group, spec lines
/// 2100–2295.
///
/// All six share one shape: the key is taken by *length*, so `'testt'`
/// with 3 is the key `tes`; a duplicate reports `E685`; and with
/// `emsg_skip` raised the failure is silent but the item is still not
/// leaked — the value it was given is released with it.
#[test]
fn adding_a_typed_value_takes_the_key_by_length() {
    let log = AllocLog::start();
    // SAFETY: every dict and list here is this case's own.
    unsafe {
        // A list and a dict are added by reference; the others by value.
        let l = tv::new_list(&[f(1.0), f(2.0), f(3.0)]);
        let d2 = tv::new_dict(&[("foo", f(42.0))]);
        let s = [
            xstrdup(cstr("TEST").as_ptr()),
            xstrdup(cstr("TEST").as_ptr()),
            xstrdup(cstr("TEST").as_ptr()),
        ];
        log.clear();

        type Add = Box<dyn Fn(*mut dict_T, usize) -> Result<(), Failed>>;
        let adds: Vec<(&str, Add, Tv, bool)> = vec![
            (
                "list",
                Box::new(move |d, _| tv_dict_add_list(d, cstr("testt").as_ptr(), 3, l)),
                Tv::List(vec![f(1.0), f(2.0), f(3.0)]),
                false,
            ),
            (
                "dict",
                Box::new(move |d, _| tv_dict_add_dict(d, cstr("testt").as_ptr(), 3, d2)),
                Tv::dict([("foo", f(42.0))]),
                false,
            ),
            (
                "nr",
                Box::new(|d, _| tv_dict_add_nr(d, cstr("testt").as_ptr(), 3, 2)),
                Tv::Int(2),
                false,
            ),
            (
                "float",
                Box::new(|d, _| tv_dict_add_float(d, cstr("testt").as_ptr(), 3, 1.5)),
                f(1.5),
                false,
            ),
            (
                "str",
                Box::new(|d, _| {
                    tv_dict_add_str(d, cstr("testt").as_ptr(), 3, cstr("TEST").as_ptr())
                }),
                Tv::s("TEST"),
                true,
            ),
            (
                "allocated_str",
                Box::new(move |d, n| tv_dict_add_allocated_str(d, cstr("testt").as_ptr(), 3, s[n])),
                Tv::s("TEST"),
                false,
            ),
        ];

        for (n, (name, add, expected, copies)) in adds.into_iter().enumerate() {
            let d = tv::new_dict(&[("test", f(10.0))]);
            log.clear();
            assert_eq!(tv::read_dict(d), Tv::dict([("test", f(10.0))]));

            assert_eq!(add(d, 0), Ok(()), "{name}");
            let di = tv::di_of(d, "tes");
            if copies {
                // `add_str` duplicates the value before the item.
                log.check(&[
                    alloc::string((*di).di_tv.vval.v_string, "TEST".len()),
                    alloc::di(di, "tes".len()),
                ]);
            } else {
                log.check(&[alloc::di(di, "tes".len())]);
            }
            assert_eq!(
                tv::read_dict(d),
                Tv::Dict(vec![
                    (b"tes".to_vec(), expected.clone()),
                    (b"test".to_vec(), f(10.0)),
                ]),
                "{name}"
            );

            // The same key again reports and frees the item it made.
            assert_eq!(
                check_emsg(log.editor(), || add(d, 1), Some(&duplicate("tes"))),
                Err(Failed),
                "{name}"
            );
            log.clear();

            // And with messages skipped, silently.
            emsg_skip.set(emsg_skip.get() + 1);
            assert_eq!(
                check_emsg(log.editor(), || add(d, 2), None),
                Err(Failed),
                "{name}"
            );
            emsg_skip.set(emsg_skip.get() - 1);
            // Everything the failed add allocated it also released — except
            // the string `allocated_str` was handed, which it owns.
            if name == "allocated_str" {
                log.check_net(false, &[alloc::freed(s[2])]);
            } else {
                log.check_net(false, &[]);
            }

            tv_dict_free(d);
            let _ = n;
        }

        // Each container is still held by nothing but this case.
        assert_eq!((*l).lv_refcount.get(), 1);
        assert_eq!((*d2).dv_refcount.get(), 1);
        tv_list_unref(l);
        tv_dict_unref(d2);
    }
}

// --------------------------------------------------------------- clear

/// `describe('clear()') itp('works')`, spec line 2296.
#[test]
fn clearing_a_dict_frees_its_items() {
    let log = AllocLog::start();
    // SAFETY: the dict is this case's own.
    unsafe {
        let d = tv_dict_alloc();
        log.check(&[alloc::dict(d)]);
        assert_eq!(tv::read_dict(d), Tv::Dict(vec![]));

        // Clearing an empty dict is a no-op.
        tv_dict_clear(d);
        assert_eq!(tv::read_dict(d), Tv::Dict(vec![]));

        let _ = tv_dict_add_str(d, cstr("TEST").as_ptr(), 3, cstr("tEsT").as_ptr());
        let di = tv::di_of(d, "TES");
        let value = (*di).di_tv.vval.v_string;
        log.check(&[
            alloc::string(value, "tEsT".len()),
            alloc::di(di, "TES".len()),
        ]);
        assert_eq!(tv::read_dict(d), Tv::dict([("TES", Tv::s("tEsT"))]));

        tv_dict_clear(d);
        log.check(&[alloc::freed(value), alloc::freed(di)]);
        assert_eq!(tv::read_dict(d), Tv::Dict(vec![]));

        tv_dict_free(d);
    }
}

// -------------------------------------------------------------- extend

/// `describe('extend()') itp('works')`, spec line 2320: the three actions.
#[test]
fn extending_a_dict_keeps_forces_or_reports() {
    let log = AllocLog::start();
    // SAFETY: both dicts are this case's own.
    unsafe {
        let extend = |d1: *mut dict_T, d2: *mut dict_T, action: &str, msg: Option<&str>| {
            check_emsg(
                log.editor(),
                || tv_dict_extend(d1, d2, cstr(action).as_ptr()),
                msg,
            );
        };

        let d1 = tv_dict_alloc();
        log.check(&[alloc::dict(d1)]);
        let d2 = tv_dict_alloc();
        log.check(&[alloc::dict(d2)]);
        for action in ["error", "keep", "force"] {
            extend(d1, d2, action, None);
        }
        log.check(&[]);
        tv_dict_free(d1);
        tv_dict_free(d2);
        log.clear();

        let d1 = tv::new_dict(&[("a", Tv::s("TEST"))]);
        let a1 = tv::di_of(d1, "a");
        let a1_s = (*a1).di_tv.vval.v_string;
        log.check_net(
            false,
            &[
                alloc::dict(d1),
                alloc::di(a1, "a".len()),
                alloc::string(a1_s, "TEST".len()),
            ],
        );
        let d2 = tv::new_dict(&[("a", Tv::s("TSET"))]);
        let a2 = tv::di_of(d2, "a");
        let a2_s = (*a2).di_tv.vval.v_string;
        log.check_net(
            false,
            &[
                alloc::dict(d2),
                alloc::di(a2, "a".len()),
                alloc::string(a2_s, "TSET".len()),
            ],
        );

        extend(d1, d2, "error", Some("E737: Key already exists: a"));
        assert_eq!(tv::read_dict(d1), Tv::dict([("a", Tv::s("TEST"))]));
        assert_eq!(tv::read_dict(d2), Tv::dict([("a", Tv::s("TSET"))]));
        log.clear();

        extend(d1, d2, "keep", None);
        log.check(&[]);
        assert_eq!(tv::read_dict(d1), Tv::dict([("a", Tv::s("TEST"))]));
        assert_eq!(tv::read_dict(d2), Tv::dict([("a", Tv::s("TSET"))]));

        extend(d1, d2, "force", None);
        log.check(&[
            alloc::freed(a1_s),
            alloc::string((*a1).di_tv.vval.v_string, "TSET".len()),
        ]);
        assert_eq!(tv::read_dict(d1), Tv::dict([("a", Tv::s("TSET"))]));
        assert_eq!(tv::read_dict(d2), Tv::dict([("a", Tv::s("TSET"))]));

        tv_dict_free(d1);
        tv_dict_free(d2);
    }
}

/// The same `describe`'s `itp('cares about locks and read-only items')`,
/// spec line 2412: four ways a target item refuses to be written.
#[test]
fn extending_a_dict_refuses_locked_and_read_only_items() {
    let log = AllocLog::start();
    // SAFETY: every dict is this case's own; `sandbox` is put back below.
    unsafe {
        let extend = |d1: *mut dict_T, d2: *mut dict_T, msg: Option<&str>| {
            check_emsg(
                log.editor(),
                || tv_dict_extend(d1, d2, cstr("force").as_ptr()),
                msg,
            );
        };

        let mut target = vec![
            ("tv_locked", f(1.0)),
            ("tv_fixed", f(2.0)),
            ("di_ro", f(3.0)),
            ("di_ro_sbx", f(4.0)),
        ];
        let d = tv::new_dict(&target);
        (*tv::di_of(d, "tv_locked")).di_tv.v_lock = VarLock::Locked;
        (*tv::di_of(d, "tv_fixed")).di_tv.v_lock = VarLock::Fixed;
        let di_ro = tv::di_of(d, "di_ro");
        (*di_ro).di_flags |= u8::try_from(DI_FLAGS_RO).unwrap();
        let di_ro_sbx = tv::di_of(d, "di_ro_sbx");
        (*di_ro_sbx).di_flags |= u8::try_from(DI_FLAGS_RO_SBX).unwrap();

        let saved_sandbox = sandbox.get();
        sandbox.set(1);
        let sources: Vec<(*mut dict_T, &str)> = vec![
            (
                tv::new_dict(&[("tv_locked", f(41.0))]),
                "E741: Value is locked: extend() argument",
            ),
            (
                tv::new_dict(&[("tv_fixed", f(42.0))]),
                "E742: Cannot change value of extend() argument",
            ),
            (
                tv::new_dict(&[("di_ro", f(43.0))]),
                r#"E46: Cannot change read-only variable "extend() argument""#,
            ),
            (
                tv::new_dict(&[("di_ro_sbx", f(44.0))]),
                r#"E794: Cannot set variable in the sandbox: "extend() argument""#,
            ),
        ];
        for (source, msg) in &sources {
            extend(d, *source, Some(msg));
            log.clear();
        }
        let as_tv = |entries: &[(&str, Tv)]| {
            let mut entries: Vec<(Vec<u8>, Tv)> = entries
                .iter()
                .map(|(k, v)| (k.as_bytes().to_vec(), v.clone()))
                .collect();
            entries.sort_by(|(a, _), (b, _)| a.cmp(b));
            Tv::Dict(entries)
        };
        assert_eq!(tv::read_dict(d), as_tv(&target));

        // Out of the sandbox the `RO_SBX` item is writable again.
        sandbox.set(0);
        extend(d, sources[3].0, None);
        target[3].1 = f(44.0);
        assert_eq!(tv::read_dict(d), as_tv(&target));
        sandbox.set(saved_sandbox);

        log.clear();
        tv_dict_free(d);
        for (source, _) in sources {
            tv_dict_free(source);
        }
    }
}

// --------------------------------------------------------------- equal

/// `describe('equal()') itp('works')`, spec line 2445: a NULL dict is the
/// empty dict, values fold case when asked, and keys never do.
#[test]
fn comparing_dicts_folds_the_values_case_but_never_the_keys() {
    let log = AllocLog::start();
    // SAFETY: every dict is this case's own.
    unsafe {
        assert!(tv_dict_equal(ptr::null_mut(), ptr::null_mut(), false));
        let d1 = tv_dict_alloc();
        log.check(&[alloc::dict(d1)]);
        assert_eq!((*d1).dv_refcount.get(), 0);
        assert!(tv_dict_equal(ptr::null_mut(), d1, false));
        assert!(tv_dict_equal(d1, ptr::null_mut(), false));
        assert!(tv_dict_equal(d1, d1, false));
        log.check(&[]);

        let build = |key: &str, value: &str| {
            let d = tv::new_dict(&[(key, Tv::s(value))]);
            let di = tv::di_of(d, key);
            log.check_net(
                false,
                &[
                    alloc::dict(d),
                    alloc::di(di, key.len()),
                    alloc::string((*di).di_tv.vval.v_string, value.len()),
                ],
            );
            d
        };
        let upper = build("a", "TEST");
        let lower = build("a", "test");
        let kupper_upper = build("A", "TEST");
        let kupper_lower = build("A", "test");

        assert!(tv_dict_equal(upper, upper, false));
        assert!(tv_dict_equal(upper, upper, true));
        assert!(!tv_dict_equal(upper, lower, false));
        assert!(tv_dict_equal(upper, lower, true));
        assert!(tv_dict_equal(kupper_upper, kupper_lower, true));
        assert!(!tv_dict_equal(kupper_upper, lower, true), "the key differs");
        assert!(
            !tv_dict_equal(kupper_upper, upper, true),
            "so does this one"
        );
        log.check(&[]);

        tv_dict_free(d1);
        for d in [upper, lower, kupper_upper, kupper_lower] {
            tv_dict_free(d);
        }
    }
}

// ---------------------------------------------------------------- copy

/// `describe('copy()') itp('copies NULL correctly')`, spec line 2495.
#[test]
fn copying_a_null_dict_answers_null() {
    let _log = AllocLog::start();
    // SAFETY: no dict is dereferenced.
    unsafe {
        for deep in [true, false] {
            for copy_id in [0, 1] {
                assert!(
                    tv_dict_copy(ptr::null(), ptr::null_mut(), deep, copy_id).is_null(),
                    "deep {deep} copyID {copy_id}"
                );
            }
        }
    }
}

/// The corpus the two `copy()` cases walk.
fn copy_corpus() -> Vec<(&'static str, Tv)> {
    vec![
        ("1", f(1.0)),
        ("a", Tv::dict([("«", Tv::s("»"))])),
        ("b", Tv::List(vec![Tv::s("„")])),
        ("nd", Tv::NullDict),
        ("nl", Tv::NullList),
        ("ns", Tv::NullStr),
        ("«»", Tv::s("“")),
    ]
}

/// `itp('copies dict correctly without converting items')`, spec line 2501.
#[test]
fn copying_a_dict_shares_or_rebuilds_its_containers() {
    let _log = AllocLog::start();
    // SAFETY: every dict is this case's own.
    unsafe {
        let entries = copy_corpus();
        let expected = Tv::Dict(
            entries
                .iter()
                .map(|(k, v)| (k.as_bytes().to_vec(), v.clone()))
                .collect(),
        );
        let d = tv::new_dict(&entries);
        let inner_dict = (*tv::di_of(d, "a")).di_tv.vval.v_dict;
        let inner_list = (*tv::di_of(d, "b")).di_tv.vval.v_list;

        assert_eq!((*inner_dict).dv_refcount.get(), 1);
        assert_eq!((*inner_list).lv_refcount.get(), 1);
        let shallow = tv_dict_copy(ptr::null(), d, false, 0);
        assert_eq!((*inner_dict).dv_refcount.get(), 2);
        assert_eq!((*inner_list).lv_refcount.get(), 2);
        assert_eq!((*tv::di_of(shallow, "a")).di_tv.vval.v_dict, inner_dict);
        assert_eq!((*tv::di_of(shallow, "b")).di_tv.vval.v_list, inner_list);
        assert_eq!(tv::read_dict(shallow), expected);
        tv_dict_free(shallow);

        assert_eq!((*inner_dict).dv_refcount.get(), 1);
        assert_eq!((*inner_list).lv_refcount.get(), 1);
        let deep = tv_dict_copy(ptr::null(), d, true, 0);
        assert!(!deep.is_null());
        assert_eq!(
            (*inner_dict).dv_refcount.get(),
            1,
            "a deep copy shares nothing"
        );
        assert_eq!((*inner_list).lv_refcount.get(), 1);
        assert_ne!((*tv::di_of(deep, "a")).di_tv.vval.v_dict, inner_dict);
        assert_ne!((*tv::di_of(deep, "b")).di_tv.vval.v_list, inner_list);
        assert_eq!(tv::read_dict(deep), expected);
        tv_dict_free(deep);

        tv_dict_free(d);
    }
}

/// `itp('copies dict correctly and converts items')`, spec line 2544 — the
/// same walk through a `vimconv_T`, which rewrites the *keys* too.
#[test]
fn a_converting_dict_copy_rewrites_the_keys_as_well() {
    let _log = AllocLog::start();
    // SAFETY: the converter and both dicts are this case's own.
    unsafe {
        let mut vc: vimconv_T = std::mem::zeroed();
        assert_eq!(
            convert_setup(
                &raw mut vc,
                cstr("utf-8").as_ptr().cast_mut(),
                cstr("latin1").as_ptr().cast_mut(),
            ),
            OK
        );

        let d = tv::new_dict(&copy_corpus());
        let inner_dict = (*tv::di_of(d, "a")).di_tv.vval.v_dict;
        let inner_list = (*tv::di_of(d, "b")).di_tv.vval.v_list;

        let deep = tv_dict_copy(&raw const vc, d, true, 0);
        assert!(!deep.is_null());
        assert_eq!((*inner_dict).dv_refcount.get(), 1);
        assert_eq!((*inner_list).lv_refcount.get(), 1);
        assert_ne!((*tv::di_of(deep, "a")).di_tv.vval.v_dict, inner_dict);
        assert_ne!((*tv::di_of(deep, "b")).di_tv.vval.v_list, inner_list);
        assert_eq!(
            tv::read_dict(deep),
            Tv::Dict(vec![
                (b"1".to_vec(), f(1.0)),
                (
                    b"a".to_vec(),
                    Tv::Dict(vec![(vec![0xAB], Tv::Str(vec![0xBB]))])
                ),
                (b"b".to_vec(), Tv::List(vec![Tv::Str(vec![0xBF])])),
                (b"nd".to_vec(), Tv::NullDict),
                (b"nl".to_vec(), Tv::NullList),
                (b"ns".to_vec(), Tv::NullStr),
                (vec![0xAB, 0xBB], Tv::Str(vec![0xBF])),
            ])
        );

        tv_dict_free(deep);
        tv_dict_free(d);
        convert_setup(&raw mut vc, ptr::null_mut(), ptr::null_mut());
    }
}

/// `itp('returns different/same containers with(out) copyID')`, spec line
/// 2584.
#[test]
fn a_dict_copy_id_preserves_sharing() {
    let _log = AllocLog::start();
    // SAFETY: every dict is this case's own.
    unsafe {
        let mut inner_tv = Tv::Dict(vec![]).build();
        let mut d_tv = Tv::Dict(vec![
            (b"a".to_vec(), Tv::Copied(&raw const inner_tv)),
            (b"b".to_vec(), Tv::Copied(&raw const inner_tv)),
        ])
        .build();
        let inner = inner_tv.vval.v_dict;
        assert_eq!((*inner).dv_refcount.get(), 3);
        let d = d_tv.vval.v_dict;
        assert_eq!(
            (*tv::di_of(d, "a")).di_tv.vval.v_dict,
            (*tv::di_of(d, "b")).di_tv.vval.v_dict
        );

        let without = tv_dict_copy(ptr::null(), d, true, 0);
        assert_ne!(
            (*tv::di_of(without, "a")).di_tv.vval.v_dict,
            (*tv::di_of(without, "b")).di_tv.vval.v_dict
        );
        assert_eq!(
            tv::read_dict(without),
            Tv::dict([("a", Tv::Dict(vec![])), ("b", Tv::Dict(vec![]))])
        );

        let with = tv_dict_copy(ptr::null(), d, true, 2);
        assert_eq!(
            (*tv::di_of(with, "a")).di_tv.vval.v_dict,
            (*tv::di_of(with, "b")).di_tv.vval.v_dict
        );
        assert_eq!(
            tv::read_dict(with),
            Tv::dict([("a", Tv::Dict(vec![])), ("b", Tv::Dict(vec![]))])
        );

        assert_eq!((*inner).dv_refcount.get(), 3);
        tv_dict_unref(without);
        tv_dict_unref(with);
        tv_clear(&raw mut d_tv);
        tv_clear(&raw mut inner_tv);
    }
}

/// `itp('works with self-referencing dict with copyID')`, spec line 2604.
#[test]
fn a_self_referencing_dict_copies_into_a_self_referencing_copy() {
    let _log = AllocLog::start();
    // SAFETY: both cycles are broken before either dict is released.
    unsafe {
        let mut d_tv = Tv::Dict(vec![]).build();
        let d = d_tv.vval.v_dict;
        assert_eq!((*d).dv_refcount.get(), 1);
        let _ = tv_dict_add_dict(d, cstr("test").as_ptr(), 4, d);
        assert_eq!((*d).dv_refcount.get(), 2);

        let copy = tv_dict_copy(ptr::null(), d, true, 2);
        assert_eq!((*copy).dv_refcount.get(), 2, "the copy holds itself");
        assert_eq!(tv::read_dict(copy), Tv::dict([("test", Tv::Cycle(0))]));

        tv_dict_clear(d);
        assert_eq!((*d).dv_refcount.get(), 1);
        tv_dict_clear(copy);
        assert_eq!((*copy).dv_refcount.get(), 1);

        tv_dict_unref(copy);
        tv_clear(&raw mut d_tv);
    }
}

/// `describe('set_keys_readonly()') itp('works')`, spec line 2625.
#[test]
fn making_keys_read_only_sets_both_flags_on_every_item() {
    let log = AllocLog::start();
    // SAFETY: the dict is this case's own.
    unsafe {
        let d = tv::new_dict(&[("a", Tv::Bool(true))]);
        let di = tv::di_of(d, "a");
        log.check(&[alloc::dict(d), alloc::di(di, "a".len())]);
        let ro = u8::try_from(DI_FLAGS_RO).unwrap();
        let fix = u8::try_from(DI_FLAGS_FIX).unwrap();
        assert_eq!((*di).di_flags & ro, 0);
        assert_eq!((*di).di_flags & fix, 0);

        tv_dict_set_keys_readonly(d);
        log.check(&[]);
        assert_eq!((*di).di_flags & ro, ro);
        assert_eq!((*di).di_flags & fix, fix);

        tv_dict_free(d);
    }
}
