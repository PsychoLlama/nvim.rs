//! `test/unit/eval/encode_spec.lua` and `test/unit/eval/tv_clear_spec.lua`,
//! the two siblings of `typval_spec` that share its harness.
//!
//! Every case needs a live editor, which Miri cannot start.

#![cfg(not(miri))]

use std::mem::ManuallyDrop;

use neovim::eval::encode::encode_list_write;
use neovim::eval::typval::{DictRef, list_first, tv_clear, tv_list_alloc};
use neovim::types::{List, TypVal};

use crate::support::alloc::AllocLog;
use crate::support::tv::{self, Payload, Tv, ValuePtr};

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
        let write = |l: *mut List, s: &[u8]| {
            encode_list_write(&mut *l, s);
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
            let list = tv_list_alloc(0);
            let l = list.as_ptr();
            for (chunk, expected) in row {
                write(tv::lval(l), chunk);
                assert_eq!(
                    tv::read_list(tv::lval(l)),
                    Tv::List(expected),
                    "after writing {chunk:?}"
                );
            }
            let mut tv = tv::list_tv(Some(list));
            tv_clear(&mut tv);
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
unsafe fn sharing(n: usize, inner: &Tv) -> TypVal {
    // SAFETY: the caller's.
    unsafe {
        let list = tv_list_alloc(isize::try_from(n).unwrap());
        let outer = list.value();
        // The items share this reference -- item 0 *is* it -- so the value
        // that built it must not release it on the way out.
        let inner_tv = ManuallyDrop::new(inner.build());
        for i in 0..n {
            if i > 0 {
                match inner_tv.v_type() {
                    neovim::types::VAR_LIST => std::mem::forget(tv::handle(inner_tv.list())),
                    _ => std::mem::forget(DictRef::retained(tv::dbox(inner_tv.dict()))),
                }
            }
            // Every item names the same container; the retain above is
            // what pays for the extra holder.
            (*outer).push(tv::bit_copy(&inner_tv));
        }
        tv::list_tv(Some(list))
    }
}

/// `describe('tv_clear()')`, spec `tv_clear_spec.lua`: four shapes where
/// the same container is reached more than once, and clearing the outer
/// list has to release it exactly once per holder.
///
/// The spec watched the shared container's free in the allocation log; a
/// container is a `Box` now, which the log does not record, so the case
/// holds a reference of its own across the clear instead: the clear gives
/// back every holder's and leaves exactly that one.
#[test]
fn clearing_releases_a_shared_container_exactly_once() {
    let log = AllocLog::start();
    // SAFETY: each structure is this case's own and is cleared.
    unsafe {
        // `[&l [1], *l, *l]` and `[&l [], *l, *l]`
        for inner in [Tv::List(vec![Tv::Float(1.0)]), Tv::List(vec![])] {
            let mut tv = sharing(3, &inner);
            let inner = list_first(tv.list().as_mut()).unwrap().li_tv.list();
            let held = tv::handle(inner);
            assert_eq!(held.refcount(), 4);
            tv_clear(&mut tv);
            assert_eq!(held.refcount(), 1);
            // Nothing per item: the items are the lists' own arrays, whose
            // growth this log does not see.
            log.check(&[]);
        }

        // `[&d {}, *d]` and `[&d {a: 1}, *d]`
        for inner in [Tv::Dict(vec![]), Tv::dict([("a", Tv::Float(1.0))])] {
            let mut tv = sharing(2, &inner);
            let inner = list_first(tv.list().as_mut()).unwrap().li_tv.dict();
            let held = DictRef::retained(tv::dbox(inner)).expect("a live dict");
            assert_eq!(held.refcount(), 3);
            tv_clear(&mut tv);
            assert_eq!(held.refcount(), 1);
            log.check(&[]);
        }
    }
}
