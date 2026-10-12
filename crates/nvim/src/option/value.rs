//! [`OptVal`]: reading it out of an option variable and writing it back,
//! comparing it, and converting it to and from an API [`Object`], a typval
//! and the text a message shows.
//!
//! It is an enum, so the payload cannot be read under the wrong kind at all.
//! What is left is the *pairing* between a value and the option row it
//! belongs to, which the type system cannot see: [`write_slot`] asserts it,
//! and [`slot_value`] relies on the table having pinned it for every row.
//!
//! A value owns its string (see [`OptVal`]): a read answers a copy, a write
//! moves the value into the variable, and nothing here frees anything by
//! hand. The two reads that only *compare* ([`slot_holds`],
//! [`optval_is_default`]) borrow instead, so listing every option copies
//! none of them.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};

use crate::memory::{ThinCString, XString};
use crate::types::{BoolVarValue, TypVal, VarNumber, kSpecialVarNull};
use crate::types::{
    Object, OptIndex, OptStr, OptVal, OptValType, String_0, kObjectTypeBoolean, kObjectTypeInteger,
    kObjectTypeNil, kObjectTypeString,
};
use crate::undo::curbuf_is_changed;

use super::{
    OptSlot, get_option, is_option_hidden, kOptValTypeBoolean, kOptValTypeNil, kOptValTypeNumber,
    kOptValTypeString, option_has_type, with_option_default,
};

/// The `Option<bool>` an option variable's tri-state word stands for: a
/// negative word is "not set in this scope", and anything above 1 is true.
pub(crate) const fn tristate(word: c_int) -> Option<bool> {
    match word {
        ..0 => None,
        0 => Some(false),
        1.. => Some(true),
    }
}

/// The `OptVal` for a boolean option's value.
///
/// `None` is upstream's `kNone`: a global-local option with no value of its
/// own in this scope.  It is not "false", and `optval_as_object` reports it
/// as nil rather than as `false`.
pub(crate) const fn boolean_optval(value: Option<bool>) -> OptVal {
    OptVal::Boolean(value)
}

/// What `:set` and `nvim_get_option_info` call each value type.
pub(crate) fn optval_type_name(type_0: OptValType) -> &'static CStr {
    match type_0 {
        kOptValTypeNil => c"nil",
        kOptValTypeBoolean => c"boolean",
        kOptValTypeNumber => c"number",
        kOptValTypeString => c"string",
        _ => unreachable!("option value type {type_0}"),
    }
}

impl OptVal {
    /// A string value copying an API string, up to its first NUL.
    pub(crate) fn string(string: String_0) -> Self {
        OptVal::from_bytes(string.as_bytes())
    }
}

/// `cur` and `new` joined, when both are strings: the `.=` of a string
/// option. `None` leaves `new` as it is.
pub(crate) fn optval_concat(cur: &OptVal, new: &OptVal) -> Option<OptVal> {
    let (OptVal::String(cur), OptVal::String(new)) = (cur, new) else {
        return None;
    };
    let mut joined = Vec::with_capacity(cur.len() + new.len());
    joined.extend_from_slice(cur.as_bytes());
    joined.extend_from_slice(new.as_bytes());
    Some(OptVal::from_xstring(XString::from_vec(joined)))
}

/// The type the table declares for an option.
pub(crate) fn option_get_type(opt_idx: OptIndex) -> OptValType {
    get_option(opt_idx).type_0
}

/// A copy of what the option variable `slot` names holds, as a value of
/// `opt_idx`'s type.
pub(crate) fn slot_value(opt_idx: OptIndex, slot: OptSlot) -> OptVal {
    // 'modified' has no variable of its own worth reading: `b_changed` alone
    // misses a buffer whose undo state says it is unchanged after all.
    if slot.is_current_modified() {
        return boolean_optval(Some(curbuf_is_changed()));
    }
    let value = match slot {
        OptSlot::None => OptVal::Nil,
        // The variable's word is the option's own tri-state, so anything
        // above 1 reads as true and a negative one is "not set here".
        OptSlot::Boolean(var) => boolean_optval(tristate(var.get())),
        OptSlot::Number(var) => OptVal::Number(var.get()),
        // A variable's bytes hold no NUL, so they are copied whole: no
        // scan for one.
        OptSlot::String(var) => {
            var.with_bytes(|bytes| OptVal::from_xstring(XString::from_bytes(bytes)))
        }
    };
    // The C read the row's declared type and stamped it on the payload; the
    // slot answers the same thing, so this is where the two are tied.
    debug_assert!(value.is_nil() || option_has_type(opt_idx, value.kind()));
    value
}

/// Whether the variable `slot` names holds `value` -- [`slot_value`]'s
/// answer compared, without the copy.
pub(crate) fn slot_holds(opt_idx: OptIndex, slot: OptSlot, value: &OptVal) -> bool {
    match (slot, value) {
        (OptSlot::String(var), OptVal::String(text)) => {
            var.with_bytes(|bytes| bytes == text.as_bytes())
        }
        (OptSlot::String(_), _) => false,
        _ => slot_value(opt_idx, slot) == *value,
    }
}

/// What a variable held before [`replace_slot`] wrote it, which
/// [`restore_slot`] puts back exactly: a string variable that owned no
/// string of its own goes back to owning none.
pub(crate) struct Replaced {
    /// The value as the setter's callbacks see it.
    pub(crate) value: OptVal,
    /// For a string variable, whether it owned no string of its own.
    pub(crate) unset: bool,
}

/// Write `value` into the option variable `slot` names, and answer what was
/// there -- the setter's own, so that whatever runs before it is put back
/// or dropped cannot free it.
pub(crate) fn replace_slot(opt_idx: OptIndex, slot: OptSlot, value: OptVal) -> Replaced {
    debug_assert!(option_has_type(opt_idx, value.kind()));
    if let OptSlot::String(var) = slot {
        let OptVal::String(text) = value else {
            unreachable!("an option's slot is not the type its value is")
        };
        return match var.put(text) {
            Some(old) => Replaced {
                value: OptVal::String(old),
                unset: false,
            },
            None => Replaced {
                value: OptVal::static_string(c""),
                unset: true,
            },
        };
    }
    let old = slot_value(opt_idx, slot);
    write_slot(opt_idx, slot, value);
    Replaced {
        value: old,
        unset: false,
    }
}

/// Put back what [`replace_slot`] answered.
pub(crate) fn restore_slot(opt_idx: OptIndex, slot: OptSlot, old: Replaced) {
    match (slot, old.value) {
        (OptSlot::String(var), OptVal::String(text)) => {
            var.restore((!old.unset).then_some(text));
        }
        (_, value) => write_slot(opt_idx, slot, value),
    }
}

/// Write `value` into the option variable `slot` names, dropping what it
/// held.
pub(crate) fn write_slot(opt_idx: OptIndex, slot: OptSlot, value: OptVal) {
    debug_assert!(option_has_type(opt_idx, value.kind()));
    match (slot, value) {
        (OptSlot::Boolean(var), OptVal::Boolean(boolean)) => {
            var.set(boolean.map_or(-1, c_int::from));
        }
        (OptSlot::Number(var), OptVal::Number(n)) => var.set(n),
        (OptSlot::String(var), OptVal::String(text)) => drop(var.put(text)),
        _ => unreachable!("an option's slot is not the type its value is"),
    }
}

/// The value as the text `:set` reports it with in a message: a string is
/// quoted, a boolean spelled out.
pub(crate) fn optval_to_text(value: &OptVal) -> XString {
    match value {
        OptVal::Nil => XString::new(),
        OptVal::Boolean(boolean) => XString::from_cstr(if *boolean == Some(true) {
            c"true"
        } else {
            c"false"
        }),
        OptVal::Number(n) => XString::from_vec(n.to_string().into_bytes()),
        OptVal::String(text) => {
            let mut quoted = Vec::with_capacity(text.len() + 2);
            quoted.push(b'"');
            quoted.extend_from_slice(text.as_bytes());
            quoted.push(b'"');
            XString::from_vec(quoted)
        }
    }
}

/// An option's value as a typval.  `numbool` renders a Boolean option as a
/// Number, which is what the old spelling of the accessors answered. A
/// string moves into the answer.
pub(crate) fn optval_as_tv(value: OptVal, numbool: bool) -> TypVal {
    let mut rettv = TypVal::Special(kSpecialVarNull);
    match value {
        OptVal::Boolean(boolean) => {
            if numbool {
                let word = value.tristate().expect("the arm is Boolean");
                rettv.write_number(VarNumber::from(word));
            } else if let Some(boolean) = boolean {
                // An unset global-local boolean has no Vimscript
                // spelling and stays the `v:null` this started as.
                rettv.write_boolean(BoolVarValue::from(boolean));
            }
        }
        OptVal::Number(number) => {
            rettv.write_number(number as VarNumber);
        }
        OptVal::String(text) => {
            rettv.write_string(Some(ThinCString::from(text.into_xstring())));
        }
        OptVal::Nil => {}
    }
    rettv
}

/// The value as the API reports it. A tri-state boolean that is neither true
/// nor false comes back as nil; a string moves into the answer.
pub(crate) fn optval_as_object(value: OptVal) -> Object {
    match value {
        OptVal::Nil => Object::Nil,
        // A global-local option with no local value has no API spelling but
        // nil, which is what `as_boolean` answers `None` for.
        OptVal::Boolean(boolean) => match boolean {
            Some(boolean) => Object::Boolean(boolean),
            None => Object::Nil,
        },
        OptVal::Number(n) => Object::Integer(n),
        OptVal::String(text) => Object::string(String_0::from_xstring(text.into_xstring())),
    }
}

/// The API value as an option value, or `None` for an `Object` no option can
/// hold.
pub(crate) fn object_as_optval(o: Object) -> Option<OptVal> {
    Some(match o.kind() {
        kObjectTypeNil => OptVal::Nil,
        kObjectTypeBoolean => boolean_optval(o.as_boolean()),
        kObjectTypeInteger => OptVal::Number(o.as_integer().expect("the tag says Integer")),
        kObjectTypeString => OptVal::string(o.into_string().expect("the tag says String")),
        _ => return None,
    })
}

/// Whether the option still holds the default the table gives it. A hidden
/// option counts as default whatever its variable says.
pub(crate) fn optval_is_default(opt_idx: OptIndex, slot: OptSlot) -> bool {
    if is_option_hidden(opt_idx) {
        return true;
    }
    match slot {
        // Two borrows of two different cells: the variable's holder and the
        // option state.
        OptSlot::String(var) => var.with_bytes(|bytes| {
            with_option_default(opt_idx, |default| {
                default.as_string().map(OptStr::as_bytes) == Some(bytes)
            })
        }),
        _ => {
            let value = slot_value(opt_idx, slot);
            with_option_default(opt_idx, |default| value == *default)
        }
    }
}
