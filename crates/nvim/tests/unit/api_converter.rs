//! `test/unit/api/private_helpers_spec.lua`, `describe('vim_to_object')`.
//!
//! The last spec built on `test/unit/eval/testutil.lua`, and the reason that
//! harness outlived `typval_spec`. It is a conversion table between two
//! value types — [`Tv`] going in, [`Obj`] coming out — and the whole of what
//! it says is where the two disagree:
//!
//! - the API has **no NULL container**: a NULL list becomes an empty array
//!   and a NULL dict an empty dict;
//! - the API has **no funcref**: a `VAR_FUNC` or `VAR_PARTIAL` becomes nil;
//! - and a **cycle** becomes nil at the point it closes, one level down,
//!   rather than looping.
//!
//! Every case needs a live editor, which Miri cannot start.

#![cfg(not(miri))]

use std::ptr;

use c2rust_neovim::api::private::converter::vim_to_object;
use c2rust_neovim::api::private::helpers::api_free_object;
use c2rust_neovim::eval::typval::tv_clear;

use crate::support::alloc::AllocLog;
use crate::support::tv::{Obj, Pt, Tv};

/// Convert `value` and read the answer back, releasing both.
///
/// # Safety
/// The editor must be up.
unsafe fn convert(value: Tv) -> Obj {
    // SAFETY: the value and the object are this call's own.
    unsafe {
        let mut tv = value.build();
        let object = vim_to_object(&raw mut tv, ptr::null_mut(), false);
        let read = crate::support::tv::read_object(&raw const object);
        api_free_object(object);
        tv_clear(&raw mut tv);
        read
    }
}

/// The rows that convert to the same shape they went in as.
#[test]
fn every_scalar_and_container_converts_to_its_own_shape() {
    let _log = AllocLog::start();
    // SAFETY: `convert` owns everything it makes.
    unsafe {
        assert_eq!(convert(Tv::Bool(true)), Obj::Bool(true));
        assert_eq!(convert(Tv::Bool(false)), Obj::Bool(false));
        assert_eq!(convert(Tv::Nil), Obj::Nil);
        // A `VAR_FLOAT` stays a float; only `int(n)` is an integer.
        assert_eq!(convert(Tv::Float(1.0)), Obj::Float(1.0));
        assert_eq!(convert(Tv::Float(-1.5)), Obj::Float(-1.5));
        assert_eq!(convert(Tv::Int(10)), Obj::Int(10));
        assert_eq!(convert(Tv::s("")), Obj::s(""));
        assert_eq!(convert(Tv::s("foobar")), Obj::s("foobar"));

        assert_eq!(convert(Tv::Dict(vec![])), Obj::Dict(vec![]));
        assert_eq!(
            convert(Tv::dict([
                ("test", Tv::Float(10.0)),
                ("test2", Tv::Bool(true)),
                ("test3", Tv::s("test")),
            ])),
            Obj::dict([
                ("test", Obj::Float(10.0)),
                ("test2", Obj::Bool(true)),
                ("test3", Obj::s("test")),
            ])
        );
        assert_eq!(
            convert(Tv::dict([
                ("test", Tv::Dict(vec![])),
                ("test2", Tv::List(vec![Tv::Float(1.0), Tv::Float(2.0)])),
            ])),
            Obj::dict([
                ("test", Obj::Dict(vec![])),
                ("test2", Obj::Array(vec![Obj::Float(1.0), Obj::Float(2.0)])),
            ])
        );

        assert_eq!(convert(Tv::List(vec![])), Obj::Array(vec![]));
        assert_eq!(
            convert(Tv::List(vec![
                Tv::Float(1.0),
                Tv::Float(2.0),
                Tv::s("test"),
                Tv::s("foo"),
            ])),
            Obj::Array(vec![
                Obj::Float(1.0),
                Obj::Float(2.0),
                Obj::s("test"),
                Obj::s("foo"),
            ])
        );
        assert_eq!(
            convert(Tv::List(vec![
                Tv::Dict(vec![]),
                Tv::dict([
                    ("test", Tv::Dict(vec![])),
                    ("test3", Tv::dict([("test4", Tv::Bool(true))])),
                ]),
            ])),
            Obj::Array(vec![
                Obj::Dict(vec![]),
                Obj::dict([
                    ("test", Obj::Dict(vec![])),
                    ("test3", Obj::dict([("test4", Obj::Bool(true))])),
                ]),
            ])
        );
    }
}

/// A container that reaches itself converts to nil at the point the cycle
/// closes — the API's value type has no way to hold one, and the walk must
/// not follow it.
#[test]
fn a_cycle_converts_to_nil_where_it_closes() {
    let _log = AllocLog::start();
    // SAFETY: `convert` owns everything it makes, and `Tv::Cycle` builds a
    // real reference that `tv_clear` releases.
    unsafe {
        // One level: `d.dct = d`.
        assert_eq!(
            convert(Tv::Dict(vec![(b"dct".to_vec(), Tv::Cycle(0))])),
            Obj::dict([("dct", Obj::Nil)])
        );
        // And `l[1] = l`.
        assert_eq!(
            convert(Tv::List(vec![Tv::Cycle(0)])),
            Obj::Array(vec![Obj::Nil])
        );

        // Two levels, through a list: `d.dct = [d]`.
        assert_eq!(
            convert(Tv::Dict(vec![
                (b"test".to_vec(), Tv::Bool(true)),
                (b"dict".to_vec(), Tv::Nil),
                (b"dct".to_vec(), Tv::List(vec![Tv::Cycle(0)])),
            ])),
            Obj::dict([
                ("dct", Obj::Array(vec![Obj::Nil])),
                ("dict", Obj::Nil),
                ("test", Obj::Bool(true)),
            ])
        );
        // Two levels, through a dict: `d.dct = {dctin = d}`.
        assert_eq!(
            convert(Tv::Dict(vec![
                (b"test".to_vec(), Tv::Bool(true)),
                (b"dict".to_vec(), Tv::Nil),
                (
                    b"dct".to_vec(),
                    Tv::Dict(vec![(b"dctin".to_vec(), Tv::Cycle(0))])
                ),
            ])),
            Obj::dict([
                ("dct", Obj::dict([("dctin", Obj::Nil)])),
                ("dict", Obj::Nil),
                ("test", Obj::Bool(true)),
            ])
        );
        // Two levels, list in list: `l[1] = [l]`.
        assert_eq!(
            convert(Tv::List(vec![Tv::List(vec![Tv::Cycle(0)])])),
            Obj::Array(vec![Obj::Array(vec![Obj::Nil])])
        );
        // Two levels, dict in list: `l[1] = {lst = l}`.
        assert_eq!(
            convert(Tv::List(vec![
                Tv::Dict(vec![(b"lst".to_vec(), Tv::Cycle(0))]),
                Tv::Bool(true),
                Tv::Bool(false),
                Tv::s("ttest"),
            ])),
            Obj::Array(vec![
                Obj::dict([("lst", Obj::Nil)]),
                Obj::Bool(true),
                Obj::Bool(false),
                Obj::s("ttest"),
            ])
        );
    }
}

/// A NULL container is not a distinct value on the API side: it converts to
/// the empty one.
#[test]
fn a_null_container_converts_to_an_empty_one() {
    let _log = AllocLog::start();
    // SAFETY: neither value owns anything.
    unsafe {
        assert_eq!(convert(Tv::NullList), Obj::Array(vec![]));
        assert_eq!(convert(Tv::NullDict), Obj::Dict(vec![]));
    }
}

/// `itp('regression: partials in a list')`, spec line 105.
///
/// A partial has no API representation, so it converts to nil — and the
/// regression was that the walk stopped there instead of carrying on to the
/// item beside it.
#[test]
fn a_partial_in_a_list_converts_to_nil_without_ending_the_walk() {
    let _log = AllocLog::start();
    // SAFETY: `convert` owns everything it makes.
    unsafe {
        let partial = Tv::Partial(Box::new(Pt {
            value: b"printf".to_vec(),
            auto: false,
            args: vec![Tv::s("%s")],
            dict: Some(Tv::dict([("v", Tv::Float(1.0))])),
        }));
        assert_eq!(
            convert(Tv::List(vec![partial, Tv::Dict(vec![])])),
            Obj::Array(vec![Obj::Nil, Obj::Dict(vec![])])
        );
    }
}
