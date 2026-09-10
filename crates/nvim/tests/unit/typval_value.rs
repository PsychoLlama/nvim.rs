//! `describe('tv')` from `test/unit/eval/typval_spec.lua`: the operations
//! over a `TypVal` itself rather than over a list or a dict.
//!
//! See `typval_list` for the shape. Every case needs a live editor, which
//! Miri cannot start.

#![cfg(not(miri))]

use std::ffi::{CStr, CString, c_char, c_int};
use std::mem::ManuallyDrop;
use std::ptr;

use neovim::eval::list::kTVCstring;
use neovim::eval::typval::{
    ListRef, tv_check_lock, tv_check_num, tv_check_str, tv_check_str_or_nr, tv_clear, tv_copy,
    tv_dict_alloc_ret, tv_equal, tv_get_float, tv_get_lnum, tv_get_number, tv_get_number_chk,
    tv_get_string_buf, tv_get_string_buf_chk, tv_islocked, tv_item_lock, tv_list_alloc_ret,
    tv_list_append_number, tv_list_first, value_check_lock,
};
use neovim::memory::{xfree, xmalloc};
use neovim::ops::NUMBUFLEN;
use neovim::types::{
    TypVal, VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_PARTIAL,
    VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN, VarLock, VarNumber, VarType, kBoolVarFalse, kBoolVarTrue,
    kSpecialVarNull,
};
use neovim::winlayer::Win;

use crate::support::alloc::{self, AllocLog};
use crate::support::tv::{self, Payload, Pt, Tv};
use crate::support::{check_emsg, cstr};

/// The spec's bare Lua numbers, which `lua2typvalt` made floats.
fn f(n: f64) -> Tv {
    Tv::Float(n)
}

/// A value of `v_type` whose payload is `bits`, the spec's
/// `typvalt(typ, vval)` — used where a case needs a value whose contents are
/// deliberately not a real one, so that a reader of the wrong arm would
/// answer something no honest value holds.
fn bogus(v_type: VarType, bits: usize) -> ManuallyDrop<TypVal> {
    ManuallyDrop::new(bogus_inner(v_type, bits))
}

fn bogus_inner(v_type: VarType, bits: usize) -> TypVal {
    let p = ptr::without_provenance_mut::<()>(bits);
    match v_type {
        VAR_NUMBER => TypVal::Number(bits as VarNumber),
        VAR_FLOAT => TypVal::Float(f64::from_bits(bits as u64)),
        VAR_STRING => TypVal::String(p.cast()),
        VAR_FUNC => TypVal::Func(p.cast()),
        // SAFETY: a made-up address the case never follows; the value is
        // a `ManuallyDrop`, so the handle is never released.
        VAR_LIST => TypVal::List(unsafe { ListRef::owning(p.cast()) }),
        VAR_DICT => TypVal::Dict(p.cast()),
        VAR_PARTIAL => TypVal::Partial(p.cast()),
        VAR_BLOB => TypVal::Blob(p.cast()),
        VAR_BOOL => TypVal::Bool(kBoolVarTrue),
        VAR_SPECIAL => TypVal::Special(kSpecialVarNull),
        VAR_UNKNOWN => TypVal::Unknown,
        other => panic!("no bogus value for v_type {other}"),
    }
}

// --------------------------------------------------------------- alloc

/// `describe('alloc') describe('list ret()') itp('works')`, spec line 2641,
/// and `dict ret()` beside it.
#[test]
fn allocating_into_a_return_value_leaves_an_empty_container() {
    let _log = AllocLog::start();
    // SAFETY: both values are this case's own and are cleared.
    unsafe {
        let mut rettv = TypVal::Unknown;
        let l = tv_list_alloc_ret(&mut rettv, 0);
        assert_eq!(tv::read(&raw const rettv), Tv::List(vec![]));
        assert_eq!(rettv.list(), l);
        tv_clear(&mut rettv);

        let mut rettv = TypVal::Unknown;
        tv_dict_alloc_ret(&mut rettv);
        assert_eq!(tv::read(&raw const rettv), Tv::Dict(vec![]));
        tv_clear(&mut rettv);
    }
}

// --------------------------------------------------------------- clear

/// `describe('clear()') itp('works')`, spec line 2660: what each type of
/// value costs to build and what clearing it gives back.
#[test]
fn clearing_a_value_releases_exactly_what_it_owns() {
    let log = AllocLog::start();
    // SAFETY: every value is this case's own.
    unsafe {
        log.check(&[]);

        // The scalars own nothing, and neither does a NULL string: an
        // already-empty value is recognised as such and does not reach the
        // allocator at all, where the C called `xfree(NULL)`.
        for (value, frees) in [
            (Tv::Nil, 0),
            (Tv::NullStr, 0),
            (f(0.0), 0),
            (Tv::Int(0), 0),
            (Tv::Bool(true), 0),
            (Tv::Bool(false), 0),
        ] {
            let mut tv = value.build();
            log.check(&[]);
            tv_clear(&mut tv);
            log.check(&vec![alloc::freed(ptr::null::<u8>()); frees]);
        }

        // A string, a dict and a list are one allocation each, released in
        // the reverse of the order they were made.
        let mut tv = Tv::s("true").build();
        log.check(&[alloc::string(tv.string(), "true".len())]);
        let s = tv.string();
        tv_clear(&mut tv);
        log.check(&[alloc::freed(s)]);

        let mut tv = Tv::Dict(vec![]).build();
        let d = tv.dict();
        log.check(&[alloc::dict(d)]);
        tv_clear(&mut tv);
        log.check(&[alloc::freed(d)]);

        let mut tv = Tv::List(vec![]).build();
        let l = tv.list();
        log.check(&[alloc::list(l)]);
        tv_clear(&mut tv);
        log.check(&[alloc::freed(l)]);

        // A self-referencing container holds itself, so clearing the only
        // *outside* reference frees nothing and leaves the count at one.
        let mut tv = Tv::List(vec![Tv::Cycle(0)]).build();
        let l = tv.list();
        log.check(&[alloc::list(l)]);
        tv_clear(&mut tv);
        log.check(&[]);
        assert_eq!((*l).lv_refcount.get(), 1);

        let mut tv = Tv::Dict(vec![(b"dd".to_vec(), Tv::Cycle(0))]).build();
        let d = tv.dict();
        log.check(&[alloc::dict(d)]);
        tv_clear(&mut tv);
        log.check(&[]);
        assert_eq!((*d).dv_refcount.get(), 1);
    }
}

/// The same statement for the kinds the spec's case did not build, said as
/// a leak check rather than as a sequence: whatever the build allocated, the
/// clear gives back. A partial owns its name, its bound arguments and its
/// dict; a blob owns its byte array; a container owns everything under it.
///
/// Written this way on purpose — `alloc_log`'s exact sequences are what the
/// value model's rewrite will invalidate, and "nothing was left held" is the
/// half of the assertion that survives it.
#[test]
fn clearing_releases_everything_a_value_allocated() {
    let log = AllocLog::start();
    // SAFETY: every value is this iteration's own and is cleared before the
    // next one is built.
    unsafe {
        for value in [
            Tv::Partial(Box::new(Pt {
                value: b"tr".to_vec(),
                auto: false,
                args: vec![Tv::s("x"), Tv::List(vec![Tv::Int(1)])],
                dict: Some(Tv::dict([("a", Tv::Int(1))])),
            })),
            Tv::Blob(vec![0, 1, 2]),
            Tv::Func(b"tr".to_vec()),
            Tv::List(vec![
                Tv::s("a"),
                Tv::dict([("k", Tv::List(vec![Tv::Int(1)]))]),
            ]),
            Tv::dict([("k", Tv::Blob(vec![9]))]),
        ] {
            log.clear();
            let mut tv = value.clone().build();
            tv_clear(&mut tv);
            log.check_net(true, &[]);
            // And clearing what is already cleared costs nothing: the value
            // keeps its type and is left holding an empty one, which is
            // what makes `tv_clear` safe to call on the way out of a frame
            // that may or may not have got that far.
            tv_clear(&mut tv);
            log.check_net(true, &[]);
        }
    }
}

// ---------------------------------------------------------------- copy

/// `describe('copy()') itp('works')`, spec line 2736: a container is shared
/// with the copy, a string is duplicated, and a scalar costs nothing.
#[test]
fn copying_a_value_shares_containers_and_duplicates_strings() {
    let log = AllocLog::start();
    // SAFETY: every value is this case's own.
    unsafe {
        for value in [
            Tv::Nil,
            Tv::NullStr,
            f(0.0),
            Tv::Int(0),
            Tv::Bool(true),
            Tv::Bool(false),
        ] {
            let mut from = value.clone().build();
            log.check(&[]);
            let mut to = TypVal::Unknown;
            tv_copy(&from, &mut to);
            assert_eq!(tv::read(&raw const to), value);
            log.check(&[]);
            tv_clear(&mut from);
            tv_clear(&mut to);
            log.clear();
        }

        let mut from = Tv::Dict(vec![]).build();
        log.check(&[alloc::dict(from.dict())]);
        let mut to = TypVal::Unknown;
        tv_copy(&from, &mut to);
        assert_eq!(tv::read(&raw const to), Tv::Dict(vec![]));
        log.check(&[]);
        assert_eq!((*to.dict()).dv_refcount.get(), 2);
        assert_eq!(to.dict(), from.dict());
        tv_clear(&mut from);
        tv_clear(&mut to);
        log.clear();

        let mut from = Tv::List(vec![]).build();
        log.check(&[alloc::list(from.list())]);
        let mut to = TypVal::Unknown;
        tv_copy(&from, &mut to);
        assert_eq!(tv::read(&raw const to), Tv::List(vec![]));
        log.check(&[]);
        assert_eq!((*to.list()).lv_refcount.get(), 2);
        assert_eq!(to.list(), from.list());
        tv_clear(&mut from);
        tv_clear(&mut to);
        log.clear();

        let mut from = Tv::s("test").build();
        log.check(&[alloc::string(from.string(), "test".len())]);
        let mut to = TypVal::Unknown;
        tv_copy(&from, &mut to);
        assert_eq!(tv::read(&raw const to), Tv::s("test"));
        log.check(&[alloc::string(to.string(), "test".len())]);
        assert_ne!(to.string(), from.string());
        tv_clear(&mut from);
        tv_clear(&mut to);
    }
}

/// The rest of `copy()`: the four kinds the spec's case never built, and
/// the property the whole phase turns on — **`tv_copy` is shallow**. It
/// takes one reference to whatever the value names and stops there, so a
/// nested container is the *same* container in both values and its own
/// count does not move. `deepcopy()` is the other one, and it goes through
/// `tv_list_copy`/`tv_dict_copy` (see `typval_list`, `typval_dict`).
#[test]
fn copying_a_container_is_shallow() {
    let _log = AllocLog::start();
    // SAFETY: both values are this case's own and are cleared.
    unsafe {
        let mut from = Tv::List(vec![Tv::List(vec![Tv::Int(1)])]).build();
        let outer = from.list();
        let inner = (*tv_list_first(outer)).li_tv.list();
        assert_eq!(
            ((*outer).lv_refcount.get(), (*inner).lv_refcount.get()),
            (1, 1)
        );

        let mut to = TypVal::Unknown;
        tv_copy(&from, &mut to);

        assert_eq!(to.list(), outer, "the copy names the same list");
        assert_eq!(
            (*tv_list_first(to.list())).li_tv.list(),
            inner,
            "and the same list inside it",
        );
        assert_eq!(
            ((*outer).lv_refcount.get(), (*inner).lv_refcount.get()),
            (2, 1),
            "only the container named by the value gained a reference",
        );

        // Which is what makes the copy an alias: appending through one is
        // visible through the other.
        tv_list_append_number(inner, 2);
        assert_eq!(
            tv::read(&raw const to),
            Tv::List(vec![Tv::List(vec![Tv::Int(1), Tv::Int(2)])]),
        );

        tv_clear(&mut from);
        assert_eq!(
            (*outer).lv_refcount.get(),
            1,
            "the other copy still holds it"
        );
        tv_clear(&mut to);
    }
}

/// A funcref is a string that also holds a reference to the function: the
/// name is duplicated exactly as `VAR_STRING`'s is.
#[test]
fn copying_a_funcref_duplicates_its_name() {
    let log = AllocLog::start();
    // SAFETY: both values are this case's own and are cleared.
    unsafe {
        let mut from = Tv::Func(b"tr".to_vec()).build();
        log.clear();
        let mut to = TypVal::Unknown;
        tv_copy(&from, &mut to);

        assert_eq!(to.v_type(), VAR_FUNC);
        assert_ne!(to.string(), from.string(), "the name was shared");
        assert_eq!(tv::read(&raw const to), Tv::Func(b"tr".to_vec()));
        log.check(&[alloc::string(to.string(), "tr".len())]);

        tv_clear(&mut from);
        tv_clear(&mut to);
    }
}

/// A partial and a blob are reference-counted like a list and a dict: one
/// object, two names for it.
#[test]
fn copying_a_partial_or_a_blob_takes_a_reference() {
    let _log = AllocLog::start();
    // SAFETY: every value is this case's own and is cleared.
    unsafe {
        let mut from = Tv::Partial(Box::new(Pt {
            value: b"tr".to_vec(),
            auto: false,
            args: vec![Tv::Int(1)],
            dict: None,
        }))
        .build();
        let pt = from.partial();
        let mut to = TypVal::Unknown;
        tv_copy(&from, &mut to);
        assert_eq!(to.partial(), pt);
        assert_eq!((*pt).pt_refcount.get(), 2);
        tv_clear(&mut from);
        assert_eq!((*pt).pt_refcount.get(), 1, "one clear freed the partial");
        assert_eq!(tv::read(&raw const to), tv::read(&raw const to));
        tv_clear(&mut to);

        let mut from = Tv::Blob(vec![0x00, 0xff]).build();
        let blob = from.blob();
        let mut to = TypVal::Unknown;
        tv_copy(&from, &mut to);
        assert_eq!(to.blob(), blob);
        assert_eq!((*blob).bv_refcount.get(), 2);
        tv_clear(&mut from);
        assert_eq!((*blob).bv_refcount.get(), 1);
        assert_eq!(tv::read(&raw const to), Tv::Blob(vec![0x00, 0xff]));
        tv_clear(&mut to);
    }
}

/// A NULL container copies as a NULL container and costs nothing: there is
/// no reference to take.
#[test]
fn copying_a_null_container_answers_a_null_container() {
    let log = AllocLog::start();
    // SAFETY: nothing here owns anything.
    unsafe {
        for value in [Tv::NullList, Tv::NullDict, Tv::NullBlob] {
            let from = value.clone().build();
            let mut to = TypVal::Unknown;
            tv_copy(&from, &mut to);
            assert_eq!(tv::read(&raw const to), value);
            assert_eq!(to.v_type(), from.v_type());
            log.check(&[]);
        }
    }
}

/// A copy carries **no lock at all**: the lock belongs to the slot a value
/// sits in, not to the value, so `tv_copy` neither reads the source slot's
/// nor writes the destination's. The container's own lock is a different
/// lock and travels with the container, so a copy of a locked list still
/// names a locked list.
#[test]
fn copying_leaves_every_lock_where_it_is() {
    let _log = AllocLog::start();
    // SAFETY: both values are this case's own and are cleared.
    unsafe {
        for lock in [VarLock::Locked, VarLock::Fixed] {
            let mut from = Slot::new(Tv::List(vec![Tv::Int(1)]).build());
            from.lock = lock;
            (*from.tv.list()).lv_lock = lock;

            let mut to = Slot::new(TypVal::Unknown);
            to.lock = lock;
            tv_copy(&from.tv, &mut to.tv);
            assert_eq!(to.lock, lock, "the copy moved the destination's lock");
            assert_eq!(
                (*to.tv.list()).lv_lock,
                lock,
                "the container's own lock moved",
            );

            (*from.tv.list()).lv_lock = VarLock::Unlocked;
            tv_clear(&mut from.tv);
            tv_clear(&mut to.tv);
        }
    }
}

/// A value beside the lock of the slot it sits in — a `ListItem` or a
/// `DictItem` minus the links.
///
/// A `TypVal` carries no lock of its own: `:lockvar l[0]` locks the *place*,
/// which is why the lock lives on the item and `tv_item_lock`/`tv_islocked`/
/// `tv_check_lock` are handed it. A case that locks a bare value keeps one
/// here.
struct Slot {
    lock: VarLock,
    tv: TypVal,
}

impl Slot {
    fn new(tv: TypVal) -> Slot {
        Slot {
            lock: VarLock::Unlocked,
            tv,
        }
    }

    /// `tv_item_lock` over this slot.
    ///
    /// # Safety
    /// As `tv_item_lock`.
    unsafe fn item_lock(&mut self, deep: c_int, lock: bool, check_refcount: bool) {
        unsafe {
            tv_item_lock(&raw mut self.lock, &mut self.tv, deep, lock, check_refcount);
        }
    }

    /// `tv_islocked` over this slot.
    ///
    /// # Safety
    /// As `tv_islocked`.
    unsafe fn islocked(&self) -> bool {
        unsafe { tv_islocked(self.lock, &self.tv) }
    }
}

/// Copying a `VAR_UNKNOWN` is an internal error, reported and not fatal —
/// the one arm of the match that says so.
#[test]
fn copying_an_unknown_value_is_an_internal_error() {
    let log = AllocLog::start();
    // SAFETY: neither value owns anything.
    unsafe {
        let from = TypVal::Unknown;
        let mut to = TypVal::Number(7);
        check_emsg(
            log.editor(),
            || tv_copy(&from, &mut to),
            Some("E685: Internal error: tv_copy(UNKNOWN)"),
        );
        assert_eq!(to.v_type(), VAR_UNKNOWN, "the type was still copied over");
    }
}

// ----------------------------------------------------------- item_lock

/// `describe('item_lock()') itp('does not alter VAR_PARTIAL')`, spec line
/// 2792: a partial's bound dict is not part of the value being locked.
#[test]
fn locking_a_partial_leaves_its_dict_alone() {
    let _log = AllocLog::start();
    // SAFETY: the partial is this case's own.
    unsafe {
        let p_tv = Tv::Partial(Box::new(Pt {
            value: b"tr".to_vec(),
            auto: false,
            args: vec![],
            dict: Some(Tv::Dict(vec![])),
        }))
        .build();
        let mut p = Slot::new(p_tv);
        p.item_lock(-1, true, false);
        assert_eq!((*(*p.tv.partial()).pt_dict).dv_lock, VarLock::Unlocked);
        tv_clear(&mut p.tv);
    }
}

/// The same `describe`'s `itp('does not change VarLock::Fixed values')`, spec
/// line 2801: `VarLock::Fixed` outranks both locking and unlocking.
#[test]
fn locking_never_moves_a_fixed_value() {
    let log = AllocLog::start();
    // SAFETY: both values are this case's own.
    unsafe {
        let mut d_tv = Slot::new(Tv::Dict(vec![]).build());
        let mut l_tv = Slot::new(Tv::List(vec![]).build());
        log.clear();
        d_tv.lock = VarLock::Fixed;
        (*d_tv.tv.dict()).dv_lock = VarLock::Fixed;
        l_tv.lock = VarLock::Fixed;
        (*l_tv.tv.list()).lv_lock = VarLock::Fixed;

        for lock in [true, false] {
            d_tv.item_lock(1, lock, false);
            l_tv.item_lock(1, lock, false);
            assert_eq!(d_tv.lock, VarLock::Fixed);
            assert_eq!(l_tv.lock, VarLock::Fixed);
            assert_eq!((*d_tv.tv.dict()).dv_lock, VarLock::Fixed);
            assert_eq!((*l_tv.tv.list()).lv_lock, VarLock::Fixed);
        }
        log.check(&[]);

        tv_clear(&mut d_tv.tv);
        tv_clear(&mut l_tv.tv);
    }
}

/// The same `describe`'s `itp('works with NULL values')`, spec line 2823:
/// the *slot* locks even when there is no container behind it.
#[test]
fn locking_a_null_container_locks_the_slot_itself() {
    let log = AllocLog::start();
    // SAFETY: none of the three values owns anything.
    unsafe {
        let mut slots = [
            Slot::new(Tv::NullList.build()),
            Slot::new(Tv::NullDict.build()),
            Slot::new(Tv::NullStr.build()),
        ];
        log.clear();
        for slot in &mut slots {
            slot.item_lock(1, true, false);
        }
        assert_eq!(tv::read(&raw const slots[0].tv), Tv::NullList);
        assert_eq!(tv::read(&raw const slots[1].tv), Tv::NullDict);
        assert_eq!(tv::read(&raw const slots[2].tv), Tv::NullStr);
        for slot in &slots {
            assert_eq!(slot.lock, VarLock::Locked);
        }
        log.check(&[]);
    }
}

// ------------------------------------------------------------ islocked

/// `describe('islocked()') itp('works with NULL values')`, spec line 2841.
#[test]
fn a_null_container_is_not_locked() {
    let _log = AllocLog::start();
    // SAFETY: neither value owns anything.
    unsafe {
        let l_tv = Slot::new(Tv::NullList.build());
        let d_tv = Slot::new(Tv::NullDict.build());
        assert!(!l_tv.islocked());
        assert!(!d_tv.islocked());
    }
}

/// The same `describe`'s `itp('works')`, spec line 2847: `tv_islocked` is
/// `VarLock::Locked` on the value *or* on the container, and `VarLock::Fixed` counts
/// as neither.
#[test]
fn a_value_is_locked_by_its_own_lock_or_its_containers() {
    let log = AllocLog::start();
    // SAFETY: every value is this case's own.
    unsafe {
        let mut tv = Slot::new(Tv::Nil.build());
        let mut d_tv = Slot::new(Tv::Dict(vec![]).build());
        let mut l_tv = Slot::new(Tv::List(vec![]).build());
        log.clear();
        let d = d_tv.tv.dict();
        let l = l_tv.tv.list();

        assert_eq!(
            (tv.islocked(), l_tv.islocked(), d_tv.islocked()),
            (false, false, false)
        );

        // The container's lock alone.
        (*d).dv_lock = VarLock::Locked;
        (*l).lv_lock = VarLock::Locked;
        assert_eq!((l_tv.islocked(), d_tv.islocked()), (true, true));

        // And the slot's own, which holds whatever the container says.
        tv.lock = VarLock::Locked;
        d_tv.lock = VarLock::Locked;
        l_tv.lock = VarLock::Locked;
        assert_eq!(
            (tv.islocked(), l_tv.islocked(), d_tv.islocked()),
            (true, true, true)
        );
        (*d).dv_lock = VarLock::Unlocked;
        (*l).lv_lock = VarLock::Unlocked;
        assert_eq!(
            (tv.islocked(), l_tv.islocked(), d_tv.islocked()),
            (true, true, true)
        );

        // `VarLock::Fixed` is not "locked" to this question.
        tv.lock = VarLock::Fixed;
        d_tv.lock = VarLock::Fixed;
        l_tv.lock = VarLock::Fixed;
        assert_eq!(
            (tv.islocked(), l_tv.islocked(), d_tv.islocked()),
            (false, false, false)
        );
        (*d).dv_lock = VarLock::Locked;
        (*l).lv_lock = VarLock::Locked;
        assert_eq!((l_tv.islocked(), d_tv.islocked()), (true, true));
        (*d).dv_lock = VarLock::Fixed;
        (*l).lv_lock = VarLock::Fixed;
        assert_eq!((l_tv.islocked(), d_tv.islocked()), (false, false));
        log.check(&[]);

        (*d).dv_lock = VarLock::Unlocked;
        (*l).lv_lock = VarLock::Unlocked;
        tv_clear(&mut d_tv.tv);
        tv_clear(&mut l_tv.tv);
    }
}

/// `describe('check_lock()') itp('works')`, spec line 2893.
///
/// `name_len` is a length *or* the sentinel `kTVCstring`, which means "the
/// whole NUL-terminated name" — the difference between `tes` and `test` in
/// the last two rows.
#[test]
fn checking_a_lock_names_what_is_locked() {
    let log = AllocLog::start();
    // SAFETY: every name is this frame's and NUL-terminated.
    unsafe {
        let test = cstr("test");
        let cstring = kTVCstring.get();
        let check = |lock, name: *const c_char, len, msg| {
            check_emsg(log.editor(), || value_check_lock(lock, name, len), msg)
        };

        assert!(!check(VarLock::Unlocked, test.as_ptr(), 3, None));
        assert!(check(
            VarLock::Locked,
            test.as_ptr(),
            3,
            Some("E741: Value is locked: tes")
        ));
        assert!(check(
            VarLock::Fixed,
            test.as_ptr(),
            3,
            Some("E742: Cannot change value of tes")
        ));
        assert!(check(
            VarLock::Locked,
            ptr::null(),
            0,
            Some("E741: Value is locked")
        ));
        assert!(check(
            VarLock::Fixed,
            ptr::null(),
            0,
            Some("E742: Cannot change value")
        ));
        assert!(check(
            VarLock::Locked,
            ptr::null(),
            cstring,
            Some("E741: Value is locked")
        ));
        assert!(check(
            VarLock::Fixed,
            test.as_ptr(),
            cstring,
            Some("E742: Cannot change value of test")
        ));
        log.clear();
    }
}

/// `deep` is a *level count*, not a flag: 1 locks the value and the
/// container it names, 2 reaches that container's items, and a negative
/// number reaches all the way down. Nothing in `typval_spec.lua` said this,
/// and it is the whole difference between `:lockvar` and `:lockvar!`.
#[test]
fn locking_descends_exactly_as_deep_as_it_is_told() {
    let _log = AllocLog::start();
    // SAFETY: the structure is this case's own and is cleared at the end.
    unsafe {
        let tv = Tv::List(vec![
            Tv::List(vec![Tv::Int(1)]),
            Tv::dict([("a", Tv::List(vec![Tv::Int(2)]))]),
        ])
        .build();
        let mut tv = Slot::new(tv);
        let outer = tv.tv.list();
        let items = tv::list_items(outer);
        let inner_list = (*items[0]).li_tv.list();
        let inner_dict = (*items[1]).li_tv.dict();
        let deepest = (*tv::di_of(inner_dict, "a")).di_tv.list();

        // The three container locks, outermost first.
        let locks = || {
            (
                (*outer).lv_lock,
                ((*inner_list).lv_lock, (*inner_dict).dv_lock),
                (*deepest).lv_lock,
            )
        };
        let unlocked = (
            VarLock::Unlocked,
            (VarLock::Unlocked, VarLock::Unlocked),
            VarLock::Unlocked,
        );
        assert_eq!(locks(), unlocked);

        // `deep == 0` is "do nothing at all", the slot included.
        tv.item_lock(0, true, false);
        assert_eq!((tv.lock, locks()), (VarLock::Unlocked, unlocked));

        // One level: the slot and the list it names, nothing inside it.
        tv.item_lock(1, true, false);
        assert_eq!(tv.lock, VarLock::Locked);
        assert_eq!(
            locks(),
            (
                VarLock::Locked,
                (VarLock::Unlocked, VarLock::Unlocked),
                VarLock::Unlocked
            ),
        );

        // Two: its items as well, but not what *they* name.
        tv.item_lock(2, true, false);
        assert_eq!(
            locks(),
            (
                VarLock::Locked,
                (VarLock::Locked, VarLock::Locked),
                VarLock::Unlocked
            ),
        );

        // All the way down, and back up again: unlocking takes the same
        // depth argument and undoes exactly what locking did.
        tv.item_lock(-1, true, false);
        assert_eq!(
            locks(),
            (
                VarLock::Locked,
                (VarLock::Locked, VarLock::Locked),
                VarLock::Locked
            ),
        );
        tv.item_lock(-1, false, false);
        assert_eq!((tv.lock, locks()), (VarLock::Unlocked, unlocked));

        tv_clear(&mut tv.tv);
    }
}

/// `check_refcount` is what keeps `:lockvar` on a function argument from
/// locking the caller's value: a container more than one name refers to is
/// left alone, and so is everything inside it.
#[test]
fn locking_leaves_a_shared_container_alone_when_asked() {
    let _log = AllocLog::start();
    // SAFETY: both values are this case's own and are cleared.
    unsafe {
        let mut tv = Slot::new(Tv::List(vec![Tv::List(vec![Tv::Int(1)])]).build());
        let outer = tv.tv.list();
        let inner = (*tv_list_first(outer)).li_tv.list();

        // A second name for the outer list, as an argument binding is.
        let mut other = TypVal::Unknown;
        tv_copy(&tv.tv, &mut other);
        assert_eq!((*outer).lv_refcount.get(), 2);

        tv.item_lock(-1, true, true);
        assert_eq!((*outer).lv_lock, VarLock::Unlocked, "a shared list locked");
        assert_eq!(
            (*inner).lv_lock,
            VarLock::Unlocked,
            "the walk went inside a list it refused to lock",
        );
        // The *slot* still locks: it is this place's, not the container's.
        assert_eq!(tv.lock, VarLock::Locked);

        // Unshared, the same call reaches both.
        tv_clear(&mut other);
        assert_eq!((*outer).lv_refcount.get(), 1);
        tv.item_lock(-1, true, true);
        assert_eq!(
            ((*outer).lv_lock, (*inner).lv_lock),
            (VarLock::Locked, VarLock::Locked)
        );

        tv.item_lock(-1, false, true);
        tv_clear(&mut tv.tv);
    }
}

/// `tv_check_lock` is the one every assignment path calls, and it is
/// [`value_check_lock`] asked **twice**: once about the slot's own lock and,
/// only if that said nothing, once about the container's. Nothing pinned it
/// before, and the container half is the half that makes `:lockvar l` stop
/// `let l[0] = 1`.
#[test]
fn checking_a_lock_reads_the_value_and_then_its_container() {
    let log = AllocLog::start();
    // SAFETY: every value is this case's own; the name outlives the calls.
    unsafe {
        let name = cstr("v");
        let cstring = kTVCstring.get();
        let check = |slot: &Slot, msg| {
            check_emsg(
                log.editor(),
                || tv_check_lock(slot.lock, &slot.tv, name.as_ptr(), cstring),
                msg,
            )
        };
        let locked = "E741: Value is locked: v";
        let fixed = "E742: Cannot change value of v";

        for (tv, set_container) in [
            (
                Tv::List(vec![]).build(),
                Box::new(|tv: &TypVal, lock| (*tv.list()).lv_lock = lock)
                    as Box<dyn Fn(&TypVal, VarLock)>,
            ),
            (
                Tv::Dict(vec![]).build(),
                Box::new(|tv: &TypVal, lock| (*tv.dict()).dv_lock = lock),
            ),
            (
                Tv::Blob(vec![]).build(),
                Box::new(|tv: &TypVal, lock| (*tv.blob()).bv_lock = lock),
            ),
        ] {
            let mut slot = Slot::new(tv);
            assert!(!check(&slot, None), "an unlocked value refused a change");

            // The slot's own lock, whatever the container says.
            slot.lock = VarLock::Locked;
            assert!(check(&slot, Some(locked)));
            slot.lock = VarLock::Fixed;
            assert!(check(&slot, Some(fixed)));
            slot.lock = VarLock::Unlocked;

            // The container's, which only an unlocked slot reaches.
            set_container(&slot.tv, VarLock::Locked);
            assert!(check(&slot, Some(locked)));
            set_container(&slot.tv, VarLock::Fixed);
            // A `VarLock::Fixed` container reports too, because the second
            // question is asked of `is_locked()` -- which is true for both
            // locked states. `tv_islocked` next door asks `== Locked`, so
            // it answers *false* for exactly this value: the two questions
            // are not the same question.
            assert!(check(&slot, Some(fixed)));
            assert!(!slot.islocked());
            set_container(&slot.tv, VarLock::Unlocked);

            tv_clear(&mut slot.tv);
        }

        // A NULL container has no lock to read, so only the slot's counts.
        for value in [Tv::NullList, Tv::NullDict, Tv::NullBlob] {
            let mut slot = Slot::new(value.build());
            assert!(!check(&slot, None));
            slot.lock = VarLock::Locked;
            assert!(check(&slot, Some(locked)));
        }
    }
}

// --------------------------------------------------------------- equal

/// `describe('equal()') itp('compares empty and NULL lists correctly')`,
/// spec line 2907.
#[test]
fn a_null_list_value_equals_an_empty_one() {
    let _log = AllocLog::start();
    // SAFETY: every value is this case's own.
    unsafe {
        let mut l = Tv::List(vec![]).build();
        let mut l2 = Tv::List(vec![]).build();
        let nl = Tv::NullList.build();

        for ic in [true, false] {
            assert!(tv_equal(&l, &nl, ic));
            assert!(tv_equal(&nl, &l, ic));
            assert!(tv_equal(&nl, &nl, ic));
            assert!(tv_equal(&l, &l, ic));
            assert!(tv_equal(&l, &l2, ic));
            assert!(tv_equal(&l2, &l, ic));
        }

        tv_clear(&mut l);
        tv_clear(&mut l2);
    }
}

/// The same `describe`'s two list `itp`s, spec lines 2926 and 2947 — the
/// `tv_equal` twin of `typval_list`'s `tv_list_equal` corpus.
#[test]
fn comparing_values_folds_case_only_when_asked() {
    let _log = AllocLog::start();
    // SAFETY: every value is this case's own.
    unsafe {
        let inner = |items: Vec<Tv>| Tv::List(items);
        let corpus = [
            vec![
                Tv::s("abc"),
                inner(vec![f(1.0), f(2.0), Tv::s("Abc")]),
                Tv::s("def"),
            ],
            vec![Tv::s("abc"), inner(vec![f(1.0), f(2.0), Tv::s("Abc")])],
            vec![
                Tv::s("abc"),
                inner(vec![f(1.0), f(2.0), Tv::s("Abc")]),
                Tv::s("Def"),
            ],
            vec![
                Tv::s("abc"),
                inner(vec![f(1.0), f(2.0), Tv::s("Abc"), f(4.0)]),
                Tv::s("def"),
            ],
            vec![
                Tv::s("Abc"),
                inner(vec![f(1.0), f(2.0), Tv::s("Abc")]),
                Tv::s("def"),
            ],
            vec![
                Tv::s("abc"),
                inner(vec![f(1.0), f(2.0), Tv::s("Abc")]),
                Tv::s("def"),
            ],
            vec![
                Tv::s("abc"),
                inner(vec![f(1.0), f(2.0), Tv::s("abc")]),
                Tv::s("def"),
            ],
            vec![Tv::s("abc"), Tv::Nil, Tv::s("def")],
            vec![Tv::s("abc"), inner(vec![f(1.0), f(2.0)]), Tv::s("def")],
        ];
        let mut tvs: Vec<TypVal> = corpus
            .into_iter()
            .map(|items| Tv::List(items).build())
            .collect();

        let expected = [
            (true, true),
            (false, false),
            (false, true),
            (false, false),
            (false, true),
            (true, true),
            (false, true),
            (false, false),
            (false, false),
        ];
        for (i, (exact, folded)) in expected.into_iter().enumerate() {
            let first = &raw mut tvs[0];
            let other = &raw mut tvs[i];
            assert_eq!(tv_equal(&*first, &*other, false), exact, "exact, value {i}");
            assert_eq!(
                tv_equal(&*first, &*other, true),
                folded,
                "folded, value {i}"
            );
        }

        for tv in &mut tvs {
            tv_clear(&mut *tv);
        }
    }
}

/// The same `describe`'s `itp('works with dictionaries')`, spec line 2971.
#[test]
fn comparing_dict_values_folds_values_but_never_keys() {
    let log = AllocLog::start();
    // SAFETY: every value is this case's own.
    unsafe {
        let nd = Tv::NullDict.build();
        assert!(tv_equal(&nd, &nd, false));
        log.check(&[]);

        let mut d1 = Tv::Dict(vec![]).build();
        log.check(&[alloc::dict(d1.dict())]);
        assert_eq!((*d1.dict()).dv_refcount.get(), 1);
        assert!(tv_equal(&nd, &d1, false));
        assert!(tv_equal(&d1, &nd, false));
        assert!(tv_equal(&d1, &d1, false));
        assert_eq!((*d1.dict()).dv_refcount.get(), 1);
        log.check(&[]);

        let build = |key: &str, value: &str| {
            let tv = Tv::dict([(key, Tv::s(value))]).build();
            let d = tv.dict();
            let di = tv::first_di(d);
            log.check_net(
                false,
                &[
                    alloc::dict(d),
                    alloc::string((*di).di_tv.string(), value.len()),
                ],
            );
            tv
        };
        let mut upper = build("a", "TEST");
        let mut lower = build("a", "test");
        let mut kupper_upper = build("A", "TEST");
        let mut kupper_lower = build("A", "test");

        assert!(tv_equal(&upper, &upper, false));
        assert!(tv_equal(&upper, &upper, true));
        assert!(!tv_equal(&upper, &lower, false));
        assert!(tv_equal(&upper, &lower, true));
        assert!(tv_equal(&kupper_upper, &kupper_lower, true));
        assert!(!tv_equal(&kupper_upper, &lower, true));
        assert!(!tv_equal(&kupper_upper, &upper, true));
        log.check(&[]);

        tv_clear(&mut d1);
        for tv in [&mut upper, &mut lower, &mut kupper_upper, &mut kupper_lower] {
            tv_clear(&mut *tv);
        }
    }
}

// --------------------------------------------------------------- check

/// The `describe('check')` group, spec lines 3021–3134.
///
/// All three read `v_type` and nothing else, which is what the case says by
/// pointing `vval` at a one-byte allocation that would crash if it were
/// dereferenced.
#[test]
fn the_type_checks_read_only_the_type() {
    let log = AllocLog::start();
    // SAFETY: `vval` is never dereferenced by anything under test — that
    // is the assertion. The allocation is freed at the end.
    unsafe {
        let bogus_alloc = xmalloc(1);
        let addr = bogus_alloc.addr();
        log.clear();

        type Check = (&'static str, unsafe fn(&TypVal) -> bool);
        /// One check, and the nine rows it is stated over.
        type Table = (Check, [(VarType, Option<&'static str>); 9]);
        let checks: [Table; 3] = [
            (
                ("str_or_nr", tv_check_str_or_nr),
                [
                    (VAR_NUMBER, None),
                    (
                        VAR_FLOAT,
                        Some("E805: Expected a Number or a String, Float found"),
                    ),
                    (
                        VAR_PARTIAL,
                        Some("E703: Expected a Number or a String, Funcref found"),
                    ),
                    (
                        VAR_FUNC,
                        Some("E703: Expected a Number or a String, Funcref found"),
                    ),
                    (
                        VAR_LIST,
                        Some("E745: Expected a Number or a String, List found"),
                    ),
                    (
                        VAR_DICT,
                        Some("E728: Expected a Number or a String, Dictionary found"),
                    ),
                    (VAR_SPECIAL, Some("E5300: Expected a Number or a String")),
                    (
                        VAR_UNKNOWN,
                        Some("E685: Internal error: tv_check_str_or_nr(UNKNOWN)"),
                    ),
                    // `str_or_nr` has no `VAR_BOOL` row in the spec; repeat
                    // the last so the three tables have one shape.
                    (
                        VAR_UNKNOWN,
                        Some("E685: Internal error: tv_check_str_or_nr(UNKNOWN)"),
                    ),
                ],
            ),
            (
                ("num", tv_check_num),
                [
                    (VAR_NUMBER, None),
                    (VAR_FLOAT, Some("E805: Using a Float as a Number")),
                    (VAR_PARTIAL, Some("E703: Using a Funcref as a Number")),
                    (VAR_FUNC, Some("E703: Using a Funcref as a Number")),
                    (VAR_LIST, Some("E745: Using a List as a Number")),
                    (VAR_DICT, Some("E728: Using a Dictionary as a Number")),
                    (VAR_SPECIAL, None),
                    (
                        VAR_UNKNOWN,
                        Some("E685: using an invalid value as a Number"),
                    ),
                    (
                        VAR_UNKNOWN,
                        Some("E685: using an invalid value as a Number"),
                    ),
                ],
            ),
            (
                ("str", tv_check_str),
                [
                    (VAR_NUMBER, None),
                    (VAR_FLOAT, None),
                    (VAR_PARTIAL, Some("E729: Using a Funcref as a String")),
                    (VAR_FUNC, Some("E729: Using a Funcref as a String")),
                    (VAR_LIST, Some("E730: Using a List as a String")),
                    (VAR_DICT, Some("E731: Using a Dictionary as a String")),
                    (VAR_BOOL, None),
                    (VAR_SPECIAL, None),
                    (
                        VAR_UNKNOWN,
                        Some("E908: Using an invalid value as a String"),
                    ),
                ],
            ),
        ];

        for ((name, check), rows) in checks {
            for (v_type, msg) in rows {
                let tv = bogus(v_type, addr);
                let ok = check_emsg(log.editor(), || check(&tv), msg);
                assert_eq!(ok, msg.is_none(), "{name} of {v_type}");
                if msg.is_some() {
                    log.clear();
                } else {
                    log.check(&[]);
                }
            }
        }

        xfree(bogus_alloc);
    }
}

// ----------------------------------------------------------------- get

/// One row of the `describe('get')` tables: a value and the message
/// reading it raises, if any.
struct Row {
    /// [`ManuallyDrop`], because a row's string is the case's own `CString`
    /// and its containers are NULL: no row owns anything to release.
    tv: ManuallyDrop<TypVal>,
    emsg: Option<&'static str>,
}

/// The rows the number-shaped getters share, in the spec's order. The
/// answers differ, so each case supplies its own.
fn number_rows(number: &CString) -> Vec<Row> {
    let row = |tv, emsg| Row {
        tv: ManuallyDrop::new(tv),
        emsg,
    };
    vec![
        row(TypVal::Number(42), None),
        row(TypVal::String(number.as_ptr().cast_mut()), None),
        row(
            TypVal::Float(42.53),
            Some("E805: Using a Float as a Number"),
        ),
        row(
            TypVal::Partial(ptr::null_mut()),
            Some("E703: Using a Funcref as a Number"),
        ),
        row(
            TypVal::Func(ptr::null_mut()),
            Some("E703: Using a Funcref as a Number"),
        ),
        row(TypVal::List(None), Some("E745: Using a List as a Number")),
        row(
            TypVal::Dict(ptr::null_mut()),
            Some("E728: Using a Dictionary as a Number"),
        ),
        row(TypVal::Special(kSpecialVarNull), None),
        row(TypVal::Bool(kBoolVarTrue), None),
        row(TypVal::Bool(kBoolVarFalse), None),
        row(
            TypVal::Unknown,
            Some("E685: Internal error: tv_get_number(UNKNOWN)"),
        ),
    ]
}

/// `describe('get') describe('number()') itp('works')`, spec line 3135, and
/// `number_chk()` beside it — the same table, plus the error flag.
#[test]
fn getting_a_number_reads_a_string_and_reports_the_rest() {
    let log = AllocLog::start();
    // SAFETY: every value is this case's own and owns nothing.
    unsafe {
        let number = cstr("100500");
        let answers = [42, 100500, 0, 0, 0, 0, 0, 0, 1, 0, 0];
        for (row, want) in number_rows(&number).into_iter().zip(answers) {
            let tv = row.tv;
            log.check(&[]);
            let got = check_emsg(log.editor(), || tv_get_number(&tv), row.emsg);
            assert_eq!(got, want, "{}", tv.v_type());
            if row.emsg.is_some() {
                log.clear();
            } else {
                log.check(&[]);
            }
        }

        for (row, want) in number_rows(&number).into_iter().zip(answers) {
            let tv = row.tv;
            let mut err = false;
            let got = check_emsg(
                log.editor(),
                || tv_get_number_chk(&tv, &raw mut err),
                row.emsg,
            );
            assert_eq!((got, err), (want, row.emsg.is_some()), "{}", tv.v_type());
            if row.emsg.is_some() {
                log.clear();
            } else {
                log.check(&[]);
            }
        }
    }
}

/// `describe('lnum()') itp('works')`, spec line 3205.
///
/// The only case in the whole spec that touches `curwin`: a `"."` resolves
/// through `var2fpos` to the cursor's line. Everything else here allocates
/// nothing, and says so.
#[test]
fn getting_a_line_number_resolves_the_cursor() {
    let log = AllocLog::start();
    // The editor's own window, not a boxed one of the case's: "current" is
    // the *identity* of a registered window ([`Win::current`]), and a boxed
    // `Window` is in no registry, so pointing `curwin` at one is no longer
    // something the editor can stand in. The cursor line is the only field
    // the case writes, and it is put back at the end -- which is what the
    // LuaJIT spec's forked child got for free.
    let mut win = Win::current();
    let saved_cursor = win.w_cursor;

    // SAFETY: every value is this case's own and owns nothing.
    unsafe {
        let number = cstr("100500");
        let dot = cstr(".");
        let mut rows = number_rows(&number);
        // `lnum` answers -1 where `number` answers 0, and reads `"."` as
        // the cursor's line — which is the row `number` has no twin for.
        rows.insert(
            2,
            Row {
                tv: ManuallyDrop::new(TypVal::String(dot.as_ptr().cast_mut())),
                emsg: None,
            },
        );
        let answers = [42, 100500, 46, -1, -1, -1, -1, -1, 0, 1, 0, -1];

        for (row, want) in rows.into_iter().zip(answers) {
            win.w_cursor.lnum = 46;
            let tv = row.tv;
            log.check(&[]);
            let got = check_emsg(log.editor(), || tv_get_lnum(&tv), row.emsg);
            assert_eq!(i64::from(got), want, "{}", tv.v_type());
            if row.emsg.is_some() {
                log.clear();
            } else {
                log.check(&[]);
            }
        }
    }

    win.w_cursor = saved_cursor;
}

/// `describe('float()') itp('works')`, spec line 3241: only a number and a
/// float have one; everything else, a string included, reports.
#[test]
fn getting_a_float_accepts_only_numbers() {
    let log = AllocLog::start();
    // SAFETY: every value is this case's own and owns nothing.
    unsafe {
        let number = cstr("100500");
        let rows: [(ManuallyDrop<TypVal>, Option<&str>, f64); 11] = [
            (ManuallyDrop::new(TypVal::Number(42)), None, 42.0),
            (
                ManuallyDrop::new(TypVal::String(number.as_ptr().cast_mut())),
                Some("E892: Using a String as a Float"),
                0.0,
            ),
            (ManuallyDrop::new(TypVal::Float(42.53)), None, 42.53),
            (
                ManuallyDrop::new(TypVal::Partial(ptr::null_mut())),
                Some("E891: Using a Funcref as a Float"),
                0.0,
            ),
            (
                ManuallyDrop::new(TypVal::Func(ptr::null_mut())),
                Some("E891: Using a Funcref as a Float"),
                0.0,
            ),
            (
                ManuallyDrop::new(TypVal::List(None)),
                Some("E893: Using a List as a Float"),
                0.0,
            ),
            (
                ManuallyDrop::new(TypVal::Dict(ptr::null_mut())),
                Some("E894: Using a Dictionary as a Float"),
                0.0,
            ),
            (
                ManuallyDrop::new(TypVal::Special(kSpecialVarNull)),
                Some("E907: Using a special value as a Float"),
                0.0,
            ),
            (
                ManuallyDrop::new(TypVal::Bool(kBoolVarTrue)),
                Some("E362: Using a boolean value as a Float"),
                0.0,
            ),
            (
                ManuallyDrop::new(TypVal::Bool(kBoolVarFalse)),
                Some("E362: Using a boolean value as a Float"),
                0.0,
            ),
            (
                ManuallyDrop::new(TypVal::Unknown),
                Some("E685: Internal error: tv_get_float(UNKNOWN)"),
                0.0,
            ),
        ];

        for (tv, emsg, want) in rows {
            log.check(&[]);
            let got = check_emsg(log.editor(), || tv_get_float(&tv), emsg);
            assert_eq!(got, want, "{}", tv.v_type());
            if emsg.is_some() {
                log.clear();
            } else {
                log.check(&[]);
            }
        }
    }
}

/// The four string getters, spec lines 3334–3416.
///
/// Each row also says *where* the answer lives: a scalar is formatted into
/// the buffer the caller supplied (or the getter's own static one), while a
/// string is answered in place. The `_chk` pair answers NULL where the
/// other two answer the empty string.
#[test]
fn getting_a_string_formats_scalars_into_the_buffer() {
    let log = AllocLog::start();
    // SAFETY: every value is this case's own and owns nothing.
    unsafe {
        let number = cstr("100500");
        let rows: [(ManuallyDrop<TypVal>, Option<&str>, Option<&str>); 11] = [
            (ManuallyDrop::new(TypVal::Number(42)), None, Some("42")),
            (
                ManuallyDrop::new(TypVal::String(number.as_ptr().cast_mut())),
                None,
                Some("100500"),
            ),
            (ManuallyDrop::new(TypVal::Float(42.53)), None, Some("42.53")),
            (
                ManuallyDrop::new(TypVal::Partial(ptr::null_mut())),
                Some("E729: Using a Funcref as a String"),
                None,
            ),
            (
                ManuallyDrop::new(TypVal::Func(ptr::null_mut())),
                Some("E729: Using a Funcref as a String"),
                None,
            ),
            (
                ManuallyDrop::new(TypVal::List(None)),
                Some("E730: Using a List as a String"),
                None,
            ),
            (
                ManuallyDrop::new(TypVal::Dict(ptr::null_mut())),
                Some("E731: Using a Dictionary as a String"),
                None,
            ),
            (
                ManuallyDrop::new(TypVal::Special(kSpecialVarNull)),
                None,
                Some("v:null"),
            ),
            (
                ManuallyDrop::new(TypVal::Bool(kBoolVarTrue)),
                None,
                Some("v:true"),
            ),
            (
                ManuallyDrop::new(TypVal::Bool(kBoolVarFalse)),
                None,
                Some("v:false"),
            ),
            (
                ManuallyDrop::new(TypVal::Unknown),
                Some("E908: Using an invalid value as a String"),
                None,
            ),
        ];

        // Every answer comes from the buffer its caller lends, so two
        // coerced values can be held at once: the second no longer overwrites
        // the first the way one process-wide buffer made it.
        let one = Tv::Int(1).build();
        let two = Tv::Int(2).build();
        let mut first = [0 as c_char; NUMBUFLEN as usize];
        let mut second = [0 as c_char; NUMBUFLEN as usize];
        let a = tv_get_string_buf(&one, first.as_mut_ptr());
        let b = tv_get_string_buf(&two, second.as_mut_ptr());
        assert_ne!(a, b);
        assert_eq!(CStr::from_ptr(a).to_bytes(), b"1");
        assert_eq!(CStr::from_ptr(b).to_bytes(), b"2");

        // The caller's buffer is this frame's, so allocating it does not
        // land in the log the rows below assert over.
        let mut buffer = [0 as c_char; NUMBUFLEN as usize];
        let scratch: *mut c_char = buffer.as_mut_ptr();

        for (name, checked, in_buffer) in [
            ("string_buf", false, scratch.cast_const()),
            ("string_buf_chk", true, scratch.cast_const()),
        ] {
            for (tv, emsg, answer) in &rows {
                let v_type = tv.v_type();
                log.check(&[]);
                let got = check_emsg(
                    log.editor(),
                    || match name {
                        "string_buf" => tv_get_string_buf(tv, scratch),
                        _ => tv_get_string_buf_chk(tv, scratch),
                    },
                    *emsg,
                );

                // A scalar is formatted into the buffer; a string is not.
                let scalar = matches!(v_type, VAR_NUMBER | VAR_FLOAT | VAR_SPECIAL | VAR_BOOL);
                if scalar {
                    assert_eq!(got, in_buffer, "{name} of {v_type} should use the buffer");
                } else if !got.is_null() {
                    assert_ne!(got, in_buffer, "{name} of {v_type} should not");
                }

                let want = match (answer, checked) {
                    (Some(s), _) => Some(*s),
                    // Unchecked, a failure is the empty string, not NULL.
                    (None, false) => Some(""),
                    (None, true) => None,
                };
                let text =
                    (!got.is_null()).then(|| CStr::from_ptr(got).to_string_lossy().into_owned());
                assert_eq!(text.as_deref(), want, "{name} of {v_type}");

                if emsg.is_some() {
                    log.clear();
                } else if v_type == VAR_FLOAT {
                    // Rendering a float goes through `vim_snprintf`.
                    log.check(&[
                        alloc::freed(ptr::null::<u8>()),
                        alloc::freed(ptr::null::<u8>()),
                    ]);
                } else {
                    log.check(&[]);
                }
            }
        }
    }
}
