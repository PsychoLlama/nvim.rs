//! The deep free [`tv_clear`] runs: releasing a value and everything only it
//! holds, without recursing.
//!
//! Upstream instantiates `typval_encode.c.h` a seventh time for this, with
//! every hook releasing what it is handed and writing nothing anywhere: the
//! `nothing` sink. What that buys is an *iterative* free -- a container
//! nested a thousand deep is released without a thousand frames of machine
//! stack -- and that is all that is kept. A free needs none of the rest of
//! the encode walk: it can take each value out of its slot as it passes, so
//! it never walks a container a second time and has no self-reference to
//! look for.
//!
//! The rule is upstream's: a container with another holder gives up one
//! reference and is not descended into, because its items are not this
//! free's to release; one whose last reference this is has every item
//! released, deepest first, and is then freed by its handle. A cycle is
//! therefore never followed -- every container on one has a holder inside
//! it -- and is left to the collector. While the collector is freeing,
//! counts can reach zero without anything being freed
//! (`tv_in_free_unref_items`); taking each value out as it is reached is
//! what keeps a container the collector also reaches from being released
//! twice.
//!
//! [`tv_clear`]: super::value::tv_clear

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::{DictRef, ListRef, PartialRef};
use crate::eval::typval_encode::InlineStack;
use crate::eval::userfunc::func_unref_name;
use crate::types::{
    TypVal, VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_PARTIAL,
    VAR_SPECIAL, VAR_STRING, kBoolVarFalse, kSpecialVarNull,
};

/// A container whose last reference the free holds, and how far through its
/// items it has got.
enum Owned {
    List { list: ListRef, at: usize },
    Dict { dict: DictRef, slot: usize },
    Partial { partial: PartialRef, arg: usize },
}

/// Containers being released without allocating, for the nests most frees
/// meet; the free runs on every value the interpreter drops.
type Pending = InlineStack<Owned, 8>;

/// Release what `tv` holds, leaving the empty value of its own kind; a
/// container that was this value's alone is pushed onto `pending` to have
/// its items released in turn.
fn release_slot(tv: &mut TypVal, pending: &mut Pending) {
    match tv.v_type() {
        VAR_NUMBER => tv.write_number(0),
        VAR_FLOAT => tv.write_float(0.0),
        VAR_BOOL => tv.write_boolean(kBoolVarFalse),
        VAR_SPECIAL => tv.write_special(kSpecialVarNull),
        // The text goes with the value.
        VAR_STRING => drop(tv.take_string()),
        // A funcref holds a reference to the function by name, and the name
        // goes with it.
        VAR_FUNC => {
            if let Some(name) = tv.take_func_name() {
                func_unref_name(name.as_cstr());
            }
        }
        // A blob holds nothing that holds anything.
        VAR_BLOB => drop(tv.take_blob()),
        VAR_LIST => {
            if let Some(list) = tv.take_list() {
                if list.lv_refcount.is_shared() {
                    drop(list);
                } else {
                    pending.push(Owned::List { list, at: 0 });
                }
            }
        }
        VAR_DICT => {
            if let Some(dict) = tv.take_dict() {
                if dict.dv_refcount.is_shared() {
                    drop(dict);
                } else {
                    pending.push(Owned::Dict { dict, slot: 0 });
                }
            }
        }
        VAR_PARTIAL => {
            if let Some(partial) = tv.take_partial() {
                if partial.pt_refcount.is_shared() {
                    drop(partial);
                } else {
                    pending.push(Owned::Partial { partial, arg: 0 });
                }
            }
        }
        // `VAR_UNKNOWN` holds nothing.
        _ => {}
    }
}

/// The next value `owned` still holds, taken out of its slot, or `None` once
/// it holds nothing more.
///
/// The value is *moved* out rather than released in place, so that the
/// borrow of the container ends before the release can reach another one.
fn next_value(owned: &mut Owned) -> Option<TypVal> {
    match owned {
        Owned::List { list, at } => {
            let item = list.edit().lv_items.get_mut(*at)?;
            *at += 1;
            Some(item.li_tv.take())
        }
        Owned::Dict { dict, slot } => {
            let dict = dict.edit();
            // The table is not resized: only values leave it.
            while *slot < dict.dv_hashtab.slots().len() {
                let at = *slot;
                *slot += 1;
                if let Some(item) = dict.item_at_mut(at) {
                    return Some(item.di_tv.take());
                }
            }
            None
        }
        // The arguments, then the self dictionary.
        Owned::Partial { partial, arg } => {
            let partial = partial.edit();
            if let Some(tv) = partial.pt_argv.get_mut(*arg) {
                *arg += 1;
                return Some(tv.take());
            }
            partial.pt_dict.take().map(|dict| TypVal::dict(Some(dict)))
        }
    }
}

/// Release whatever `tv` holds and everything only it holds, leaving `tv`
/// the empty value of its own kind.
pub(super) fn release_deep(tv: &mut TypVal) {
    let mut pending = Pending::new();
    release_slot(tv, &mut pending);
    while let Some(owned) = pending.last_mut() {
        match next_value(owned) {
            Some(mut value) => release_slot(&mut value, &mut pending),
            // Emptied: the handle's own drop frees the container, which
            // holds nothing any more.
            None => drop(pending.pop()),
        }
    }
}
