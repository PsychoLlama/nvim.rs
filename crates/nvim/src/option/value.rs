//! [`OptVal`]: freeing, copying, comparing, and converting to and from the
//! option variable, an API [`Object`] and a C string.
//!
//! It is an enum, so the payload cannot be read under the wrong kind at all
//! and the `unreachable!()` arms the transpile needed for the tag values the
//! union had no payload for are gone. What is left is the *pairing* between
//! a value and the option row it belongs to, which the type system cannot
//! see: [`set_option_varp`] asserts it, and [`optval_from_varp`] relies on
//! the table having pinned it for every row.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::winlayer::Buf;
use core::ffi::{CStr, c_char};

use crate::cstr;
use crate::memory::{xmalloc, xstrdup};
use crate::optionstr::is_empty_option;
use crate::os::cshim::snprintf;
use crate::types::{
    Object, OptIndex, OptStr, OptVal, OptValType, String_0, kObjectTypeBoolean, kObjectTypeInteger,
    kObjectTypeNil, kObjectTypeString, size_t,
};
use crate::undo::curbuf_is_changed;

use super::{
    NUMBUFLEN, OptSlot, get_option, is_option_hidden, kOptValTypeBoolean, kOptValTypeNil,
    kOptValTypeNumber, kOptValTypeString, option_default, option_has_type,
};

/// The `OptVal` for a boolean option's value.
///
/// `None` is upstream's `kNone`: a global-local option with no value of its
/// own in this scope.  It is not "false", and `optval_as_object` reports it
/// as nil rather than as `false`.
pub(crate) const fn boolean_optval(value: Option<bool>) -> OptVal {
    OptVal::Boolean(match value {
        Some(true) => 1,
        Some(false) => 0,
        None => -1,
    })
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
    /// A string value that **takes over** `string`'s allocation:
    /// [`optval_free`] is what releases it now.
    pub(crate) fn string(string: String_0) -> Self {
        let size = string.len();
        OptVal::String(OptStr::from_raw_parts(string.into_raw(), size))
    }
}

impl OptStr {
    /// The bytes, not counting the terminator.
    ///
    /// # Safety
    /// The pair must still name live bytes -- it is `Copy`, so a value
    /// released through [`optval_free`] leaves copies of itself behind.
    pub(crate) unsafe fn as_bytes<'a>(&self) -> &'a [u8] {
        // An option variable that has never been set is a null pointer,
        // which no slice may be built over even at zero length.
        if self.data().is_null() {
            return &[];
        }
        // SAFETY: the caller's promise.
        unsafe { core::slice::from_raw_parts(self.data().cast::<u8>(), self.len()) }
    }

    /// Take over a NUL-terminated allocation the option module will free.
    ///
    /// # Safety
    /// `value` is NUL-terminated and has one owner, which the value this
    /// ends up in becomes.
    pub(crate) unsafe fn owning(value: *mut c_char) -> Self {
        // SAFETY: the caller's NUL-terminated string.
        let len = unsafe { cstr::bytes_at(value) }.len();
        OptStr::from_raw_parts(value, len)
    }

    /// A view of a NUL-terminated string the caller goes on owning.
    ///
    /// # Safety
    /// `value` is NUL-terminated and outlives every use of the answer, and
    /// nothing passes the value it ends up in to [`optval_free`] --
    /// `set_option_direct` and `set_option_value` both copy.
    pub(crate) unsafe fn borrowing(value: *const c_char) -> Self {
        // SAFETY: the caller's NUL-terminated string.
        let len = unsafe { cstr::bytes_at(value) }.len();
        OptStr::from_raw_parts(value.cast_mut(), len)
    }

    /// An owned copy of the bytes, for the API layer.
    ///
    /// # Safety
    /// As [`OptStr::as_bytes`].
    pub(crate) unsafe fn to_owned_string(&self) -> String_0 {
        // SAFETY: the caller's promise.
        String_0::from_bytes(unsafe { self.as_bytes() })
    }
}

/// Release what a value owns. Only a string owns anything, and the shared
/// empty string every unset string option points at is not ours to free.
///
/// This is the *only* release path: an [`OptVal`]'s string sits in a
/// `ManuallyDrop`, so a value that merely goes out of scope frees nothing.
pub(crate) fn optval_free(value: OptVal) {
    let Some(string) = value.as_string() else {
        return;
    };
    if is_empty_option(string.data()) {
        // The shared empty string is nobody's to free.
        return;
    }
    // SAFETY: the caller's promise -- this value's bytes are one allocation
    // with one owner, and this is it.
    drop(unsafe { String_0::from_owned_parts(string.data(), string.len()) });
}

/// A value that owns its own copy of whatever the original owned.
pub(crate) fn optval_copy(value: &OptVal) -> OptVal {
    match value.as_string() {
        // SAFETY: a live value's string names live bytes.
        Some(string) => OptVal::string(unsafe { string.to_owned_string() }),
        None => *value,
    }
}

/// Whether two values are the same. Two strings of the same length are
/// compared byte-wise, so an option holding a NUL still compares correctly.
pub(crate) fn optval_equal(o1: &OptVal, o2: &OptVal) -> bool {
    match (o1, o2) {
        (OptVal::Nil, OptVal::Nil) => true,
        (OptVal::Boolean(a), OptVal::Boolean(b)) => a == b,
        (OptVal::Number(a), OptVal::Number(b)) => a == b,
        // SAFETY: both values are live, so both pairs name live bytes.
        (OptVal::String(s1), OptVal::String(s2)) => unsafe { s1.as_bytes() == s2.as_bytes() },
        _ => false,
    }
}

/// The type the table declares for an option.
pub(crate) fn option_get_type(opt_idx: OptIndex) -> OptValType {
    get_option(opt_idx).type_0
}

/// Read the option variable `slot` names as a value of `opt_idx`'s type.
///
/// # Safety
///
/// `slot` must be the variable the table names for `opt_idx`, in some scope
/// — what `get_varp`/`get_varp_scope` hand out.
pub(crate) unsafe fn optval_from_varp(opt_idx: OptIndex, slot: OptSlot) -> OptVal {
    // 'modified' has no variable of its own worth reading: `b_changed` alone
    // misses a buffer whose undo state says it is unchanged after all.
    // SAFETY: `curbuf` is a live buffer for as long as the editor is running.
    if slot == OptSlot::Boolean(unsafe { &raw mut (*Buf::current_raw()).b_changed }) {
        // SAFETY: reading the current buffer's change state.
        return boolean_optval(Some(curbuf_is_changed()));
    }
    // SAFETY (all three): the slot names this option's variable and its
    // variant is the type that variable holds. A boolean's word is the
    // option's own tri-state, so anything above 1 reads as true.
    let value = match slot {
        OptSlot::None => OptVal::Nil,
        OptSlot::Boolean(var) => OptVal::Boolean(unsafe { *var }.clamp(-1, 1)),
        OptSlot::Number(var) => OptVal::Number(unsafe { *var }),
        // A *borrow* of the option variable's own buffer, which is what
        // makes `optval_free` of this value free the variable's string --
        // the protocol `set_option_varp`'s `free_oldval` relies on.
        OptSlot::String(var) => {
            let data = unsafe { *var };
            // An option variable that has never been set is a null pointer,
            // which is the empty value rather than a string of no bytes.
            let len = if data.is_null() {
                0
            } else {
                unsafe { cstr::bytes_at(data) }.len()
            };
            OptVal::String(OptStr::from_raw_parts(data, len))
        }
    };
    // The C read the row's declared type and stamped it on the payload; the
    // slot answers the same thing, so this is where the two are tied.
    debug_assert!(value.is_nil() || option_has_type(opt_idx, value.kind()));
    value
}

/// Write `value` into the option variable `slot` names, taking ownership
/// of whatever it holds. With `free_oldval` the value already there is freed
/// first; without it the caller has kept a copy and will free it later.
///
/// # Safety
///
/// `slot` must be the variable the table names for `opt_idx`, in some scope.
pub(crate) unsafe fn set_option_varp(
    opt_idx: OptIndex,
    slot: OptSlot,
    value: OptVal,
    free_oldval: bool,
) {
    debug_assert!(option_has_type(opt_idx, value.kind()));
    if free_oldval {
        // SAFETY: the caller's slot is this option's variable.
        optval_free(unsafe { optval_from_varp(opt_idx, slot) });
    }
    // SAFETY: the slot and the value are the same type — the table asserts
    // it for every row at compile time, and the assertion above ties this
    // value to the same row.
    match (slot, value) {
        (OptSlot::Boolean(var), OptVal::Boolean(word)) => unsafe { *var = word },
        (OptSlot::Number(var), OptVal::Number(n)) => unsafe { *var = n },
        // The variable takes the allocation over.
        (OptSlot::String(var), OptVal::String(s)) => unsafe { *var = s.data() },
        _ => unreachable!("an option's slot is not the type its value is"),
    }
}

/// The value as a freshly allocated C string, the way `:set` reports it in a
/// message: a string is quoted, a boolean spelled out. The caller owns it.
pub(crate) fn optval_to_cstr(value: &OptVal) -> *mut c_char {
    match value {
        // SAFETY (every arm): the literal is NUL-terminated, and `buf` is
        // the allocation the arm just made.
        OptVal::Nil => unsafe { xstrdup(c"".as_ptr()) },
        OptVal::Boolean(word) => {
            let word = if *word != 0 { c"true" } else { c"false" };
            unsafe { xstrdup(word.as_ptr()) }
        }
        OptVal::Number(n) => {
            let len = NUMBUFLEN as size_t;
            let buf = unsafe { xmalloc(len) }.cast::<c_char>();
            unsafe { snprintf(buf, len, c"%ld".as_ptr(), *n) };
            buf
        }
        OptVal::String(s) => {
            // Two quotes and the terminator.
            let len = s.len().wrapping_add(3);
            let buf = unsafe { xmalloc(len) }.cast::<c_char>();
            unsafe { snprintf(buf, len, c"\"%s\"".as_ptr(), s.data()) };
            buf
        }
    }
}

/// The value as the API reports it. A tri-state boolean that is neither true
/// nor false comes back as nil.
///
/// **The string is copied**, and the value goes on owning its own: an option
/// value may be borrowing the option variable's buffer (`optval_from_varp`)
/// or a `.rodata` default, neither of which an owning `Object` may hold.
/// A caller whose value *did* own its bytes still has to `optval_free` it.
pub(crate) fn optval_as_object(value: OptVal) -> Object {
    match value {
        OptVal::Nil => Object::Nil,
        // A global-local option with no local value has no API spelling but
        // nil, which is what `as_boolean` answers `None` for.
        OptVal::Boolean(_) => match value.as_boolean() {
            Some(boolean) => Object::Boolean(boolean),
            None => Object::Nil,
        },
        OptVal::Number(n) => Object::Integer(n),
        // SAFETY: the value is live, so its pair names live bytes; the
        // object gets a copy, because the option layer goes on owning its.
        OptVal::String(s) => Object::string(unsafe { s.to_owned_string() }),
    }
}

/// The API value as an option value, or `None` for an `Object` no option can
/// hold. The object's string moves into the answer.
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
///
/// # Safety
///
/// `slot` must be the variable the table names for `opt_idx`, in some scope.
pub(crate) unsafe fn optval_is_default(opt_idx: OptIndex, slot: OptSlot) -> bool {
    if is_option_hidden(opt_idx) {
        return true;
    }
    // SAFETY: the caller's slot is this option's variable.
    let current = unsafe { optval_from_varp(opt_idx, slot) };
    optval_equal(&current, &option_default(opt_idx))
}
