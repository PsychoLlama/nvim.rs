//! `test/unit/eval/encode_spec.lua` and `test/unit/eval/tv_clear_spec.lua`,
//! the two siblings of `typval_spec` that share its harness.
//!
//! Every case needs a live editor, which Miri cannot start.

#![cfg(not(miri))]

use std::ptr;

use neovim::eval::encode::encode_list_write;
use neovim::eval::typval::{tv_clear, tv_list_alloc, tv_list_append};
use neovim::types::{Refcount, VarLock, list_T, typval_T, typval_vval_union};

use crate::support::alloc::{self, AllocLog};
use crate::support::tv::{self, Tv};

// -------------------------------------------------------------- encode

/// `describe('encode_list_write()')`, spec `encode_spec.lua`.
///
/// The sink `writefile()` and `msgpackdump()` write through: bytes are
/// appended to the last item, and a newline starts a new one. The two
/// translations that make it interesting are that a NUL in the input is a
/// newline in the *value*, and that a list whose last item is a NULL string
/// is the same as one whose last item is empty.
#[test]
fn writing_to_a_list_splits_on_newlines_and_joins_on_nul() {
    let _log = AllocLog::start();
    // SAFETY: each list is this case's own and is freed.
    unsafe {
        let write = |l: *mut list_T, s: &[u8]| {
            encode_list_write(l.cast(), s.as_ptr().cast(), s.len());
        };
        let ns = Tv::NullStr;
        let s = Tv::s;

        // Each row is a sequence of writes, each paired with the list it
        // leaves behind — the spec asserted after every write, and so does
        // this.
        let rows: Vec<Vec<(&[u8], Vec<Tv>)>> = vec![
            // An empty write leaves an empty list, not a list of one empty
            // string.
            vec![(b"", vec![])],
            vec![(b"abc", vec![s("abc")])],
            // A leading newline opens with an empty line, which the list
            // spells as a NULL string.
            vec![(b"\nabc", vec![ns.clone(), s("abc")])],
            vec![
                (b"\nabc", vec![ns.clone(), s("abc")]),
                (b"\nabc", vec![ns.clone(), s("abc"), s("abc")]),
            ],
            // A trailing newline leaves the *next* line open.
            vec![(b"abc\n", vec![s("abc"), ns.clone()])],
            vec![
                (b"abc\n", vec![s("abc"), ns.clone()]),
                (b"abc\n", vec![s("abc"), s("abc"), ns.clone()]),
            ],
            vec![
                (b"\na\nb\n", vec![ns.clone(), s("a"), s("b"), ns.clone()]),
                (
                    b"\na\nb\n",
                    vec![
                        ns.clone(),
                        s("a"),
                        s("b"),
                        ns.clone(),
                        s("a"),
                        s("b"),
                        ns.clone(),
                    ],
                ),
            ],
            // A NUL byte in the input is a newline in the *value*, so this
            // write makes three one-newline items — and writing the same
            // again joins onto the open last one rather than starting over.
            vec![
                (b"\0\n\0\n\0", vec![s("\n"), s("\n"), s("\n")]),
                (
                    b"\0\n\0\n\0",
                    vec![s("\n"), s("\n"), s("\n\n"), s("\n"), s("\n")],
                ),
            ],
            vec![
                (
                    b"\n\0\n\0\n",
                    vec![ns.clone(), s("\n"), s("\n"), ns.clone()],
                ),
                (
                    b"\n\0\n\0\n",
                    vec![
                        ns.clone(),
                        s("\n"),
                        s("\n"),
                        ns.clone(),
                        s("\n"),
                        s("\n"),
                        ns.clone(),
                    ],
                ),
            ],
            vec![(b"\n", vec![ns.clone(); 2]), (b"\n", vec![ns.clone(); 3])],
            vec![
                (b"\n\n\n", vec![ns.clone(); 4]),
                (b"\n\n\n", vec![ns.clone(); 7]),
            ],
        ];

        for row in rows {
            let l = tv_list_alloc(0);
            (*l).lv_refcount = Refcount::ONE;
            for (chunk, expected) in row {
                write(l, chunk);
                assert_eq!(
                    tv::read_list(l),
                    Tv::List(expected),
                    "after writing {chunk:?}"
                );
            }
            let mut tv = typval_T {
                v_type: neovim::types::VAR_LIST,
                v_lock: VarLock::Unlocked,
                vval: typval_vval_union { v_list: l },
            };
            tv_clear(&raw mut tv);
        }
    }
}

// ------------------------------------------------------------ tv_clear

/// `[&x, *x, …]`: a list of `n` items all naming the same container, built
/// in the order `lua2typvalt` built one — the outer list first, then the
/// shared container, then the items. Each item holds a reference, so the
/// container's count ends at `n`.
///
/// # Safety
/// The editor must be up. The caller owns the answer.
unsafe fn sharing(n: usize, inner: &Tv) -> typval_T {
    // SAFETY: the caller's.
    unsafe {
        let outer = tv_list_alloc(n as isize);
        (*outer).lv_refcount = Refcount::ONE;
        let inner_tv = inner.build();
        for i in 0..n {
            if i > 0 {
                match inner_tv.v_type {
                    neovim::types::VAR_LIST => (*inner_tv.vval.v_list).lv_refcount.retain(),
                    _ => (*inner_tv.vval.v_dict).dv_refcount.retain(),
                }
            }
            let li = tv::list_item_alloc();
            (*li).li_next = ptr::null_mut();
            (*li).li_prev = ptr::null_mut();
            (*li).li_tv = inner_tv;
            tv_list_append(outer, li);
        }
        typval_T {
            v_type: neovim::types::VAR_LIST,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union { v_list: outer },
        }
    }
}

/// `describe('tv_clear()')`, spec `tv_clear_spec.lua`: four shapes where
/// the same container is reached more than once, and clearing the outer
/// list has to release it exactly once.
#[test]
fn clearing_releases_a_shared_container_exactly_once() {
    let log = AllocLog::start();
    // SAFETY: each structure is this case's own and is cleared.
    unsafe {
        // `[&l [1], *l, *l]`
        let mut tv = sharing(3, &Tv::List(vec![Tv::Float(1.0)]));
        let outer = tv.vval.v_list;
        let lis = tv::list_items(outer);
        let inner = (*lis[0]).li_tv.vval.v_list;
        let inner_li = (*inner).lv_first;
        log.check(&[
            alloc::list(outer),
            alloc::list(inner),
            alloc::li(inner_li),
            alloc::li(lis[0]),
            alloc::li(lis[1]),
            alloc::li(lis[2]),
        ]);
        assert_eq!((*inner).lv_refcount.get(), 3);
        tv_clear(&raw mut tv);
        log.check(&[
            alloc::freed(inner_li),
            alloc::freed(inner),
            alloc::freed(lis[0]),
            alloc::freed(lis[1]),
            alloc::freed(lis[2]),
            alloc::freed(outer),
        ]);

        // `[&l [], *l, *l]`
        let mut tv = sharing(3, &Tv::List(vec![]));
        let outer = tv.vval.v_list;
        let lis = tv::list_items(outer);
        let inner = (*lis[0]).li_tv.vval.v_list;
        log.check(&[
            alloc::list(outer),
            alloc::list(inner),
            alloc::li(lis[0]),
            alloc::li(lis[1]),
            alloc::li(lis[2]),
        ]);
        assert_eq!((*inner).lv_refcount.get(), 3);
        tv_clear(&raw mut tv);
        log.check(&[
            alloc::freed(inner),
            alloc::freed(lis[0]),
            alloc::freed(lis[1]),
            alloc::freed(lis[2]),
            alloc::freed(outer),
        ]);

        // `[&d {}, *d]`
        let mut tv = sharing(2, &Tv::Dict(vec![]));
        let outer = tv.vval.v_list;
        let lis = tv::list_items(outer);
        let inner = (*lis[0]).li_tv.vval.v_dict;
        log.check(&[
            alloc::list(outer),
            alloc::dict(inner),
            alloc::li(lis[0]),
            alloc::li(lis[1]),
        ]);
        assert_eq!((*inner).dv_refcount.get(), 2);
        tv_clear(&raw mut tv);
        log.check(&[
            alloc::freed(inner),
            alloc::freed(lis[0]),
            alloc::freed(lis[1]),
            alloc::freed(outer),
        ]);

        // `[&d {a: 1}, *d]`
        let mut tv = sharing(2, &Tv::dict([("a", Tv::Float(1.0))]));
        let outer = tv.vval.v_list;
        let lis = tv::list_items(outer);
        let inner = (*lis[0]).li_tv.vval.v_dict;
        let di = tv::first_di(inner);
        log.check(&[
            alloc::list(outer),
            alloc::dict(inner),
            alloc::di(di, "a".len()),
            alloc::li(lis[0]),
            alloc::li(lis[1]),
        ]);
        assert_eq!((*inner).dv_refcount.get(), 2);
        tv_clear(&raw mut tv);
        log.check(&[
            alloc::freed(di),
            alloc::freed(inner),
            alloc::freed(lis[0]),
            alloc::freed(lis[1]),
            alloc::freed(outer),
        ]);
    }
}
