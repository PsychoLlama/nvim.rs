//! The one-line accessors every other module reaches a `TypVal` through.
//!
//! Upstream declares these `static inline` in `typval.h`, so they are the part
//! of this file that is compiled into its callers rather than called; they keep
//! the `#[inline]` the transpile gave them for the same reason.
//!
//! Every accessor takes the raw pointer its callers already hold — 500-odd call
//! sites across the tree pass `*mut List`/`*mut Dict` around, and the
//! `TypVal` family's layout is frozen by the LuaJIT unit specs.  What they
//! buy the rest of the family is that *nothing else* has to spell a field walk:
//! the children below reach a list through `list_items`/`list_iter`/
//! `list_len`, never through `(*l).lv_items`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::winlayer::Live;

/// The `Copy` handles over the four objects this module manipulates through
/// raw pointers, plus the two it reaches through them.
///
/// See [`Live`](crate::winlayer::Live): construction is the one `unsafe` step
/// and records the caller's promise that the pointee stays live; every
/// `(*p).field` after it is ordinary checked code, which is why the
/// `tv_*_set`/`_alloc` families lose almost all of their regions to these.
///
/// A handle is never built from a pointer the code has not already committed
/// to dereferencing: the null-tolerant entry points (`list_len`,
/// `list_find`, …) keep their `as_ref()` guard and take no handle.
pub(crate) type Ls = Live<List>;
/// A live `Dict`; see [`Ls`].
pub(crate) type Dt = Live<Dict>;

/// The tag-checked readers, generated ten times over the one shape they all
/// have.
///
/// A `TypVal` is an enum, so the tag test *is* the read: the `as_*` form is a
/// `match` with one arm and answers `None` for every other variant. What this
/// buys over writing the match at the call site is that seven hundred of them
/// keep reading like the field accesses they replaced -- and that the family
/// is defined once, so a new variant is a new row here rather than a hunt.
///
/// The `*_or_null`/`*_or_zero` form answers the empty value this family
/// already reads as absent everywhere (`list_len(NULL) == 0`,
/// `partial_name(NULL)`, `tv_get_number` of a `VAR_SPECIAL`), which is what a
/// site whose type was established by an earlier `tv_check_for_*_arg` wants to
/// write.
macro_rules! union_readers {
    ($(
        $variant:ident, $ty:ty, $as_fn:ident $(, $or_fn:ident = $empty:expr)?;
    )*) => {
        impl TypVal {
            $(
                #[doc = concat!("The payload, or `None` unless this is a `", stringify!($variant), "`.")]
                #[inline(always)]
                pub(crate) fn $as_fn(&self) -> Option<$ty> {
                    match self {
                        TypVal::$variant(value) => Some(*value),
                        _ => None,
                    }
                }

                $(
                    #[doc = concat!("The payload, or the empty value unless this is a `", stringify!($variant), "`.")]
                    #[inline(always)]
                    pub(crate) fn $or_fn(&self) -> $ty {
                        self.$as_fn().unwrap_or($empty)
                    }
                )?
            )*
        }
    };
}

union_readers! {
    Number,  VarNumber,                as_number,    number_or_zero = 0;
    Bool,    BoolVarValue,             as_bool;
    Special, SpecialVarValue,          as_special;
    Float,   Float,                    as_float,     float_or_zero = 0.0;
}

impl TypVal {
    /// The dictionary this value holds, or NULL unless it is a dictionary
    /// holding one.  A **borrow**; see [`TypVal::list_or_null`].
    #[inline(always)]
    pub(crate) fn dict_or_null(&self) -> *mut Dict {
        match self {
            TypVal::Dict(dict) => dict
                .as_ref()
                .map_or(::core::ptr::null_mut(), DictRef::as_ptr),
            _ => ::core::ptr::null_mut(),
        }
    }

    /// The dictionary this value holds, borrowed -- `None` for every other
    /// kind and for `v:_null_dict`.
    ///
    /// The safe spelling of [`TypVal::dict_or_null`], and the one the
    /// `dict_*` family reads its argument in; see [`TypVal::list_ref`].
    #[inline(always)]
    pub(crate) fn dict_ref(&self) -> Option<&Dict> {
        match self {
            TypVal::Dict(dict) => dict.as_deref(),
            _ => None,
        }
    }

    /// The dictionary this value holds, borrowed for writing.
    ///
    /// See [`TypVal::list_mut`]: the exclusive borrow is the point.
    #[inline(always)]
    pub(crate) fn dict_mut(&mut self) -> Option<&mut Dict> {
        match self {
            TypVal::Dict(dict) => dict.as_deref_mut(),
            _ => None,
        }
    }

    /// A dictionary value over `dict`, which the value takes over.
    ///
    /// The dictionary half of [`TypVal::list`]: the one place the
    /// [`ManuallyDrop`](::core::mem::ManuallyDrop) the variant carries is
    /// spelled.
    #[inline(always)]
    pub(crate) const fn dict(dict: Option<DictRef>) -> TypVal {
        TypVal::Dict(::core::mem::ManuallyDrop::new(dict))
    }

    /// Overwrite this slot with `dict`, **releasing nothing**: see
    /// [`union_writers`].  The slot takes over whatever the handle owns.
    #[inline(always)]
    pub(crate) fn write_dict(&mut self, dict: Option<DictRef>) {
        self.overwrite(TypVal::dict(dict));
    }

    /// The handle this value holds, borrowed; see [`TypVal::list_shared`].
    #[inline(always)]
    pub(crate) fn dict_shared(&self) -> Option<&DictRef> {
        match self {
            TypVal::Dict(dict) => dict.as_ref(),
            _ => None,
        }
    }

    /// Another reference to the dictionary this value holds; see
    /// [`TypVal::list_handle`].
    #[inline(always)]
    pub(crate) fn dict_handle(&self) -> Option<DictRef> {
        match self {
            TypVal::Dict(dict) => (**dict).clone(),
            _ => None,
        }
    }

    /// Move the dictionary out of this slot, leaving `v:_null_dict` behind.
    /// See [`TypVal::take_list`].
    #[inline(always)]
    pub(crate) fn take_dict(&mut self) -> Option<DictRef> {
        match self {
            TypVal::Dict(dict) => dict.take(),
            _ => None,
        }
    }

    /// Which kind of value this is, as the `VAR_*` code the tree tests
    /// against.
    ///
    /// The discriminants *are* those codes (see [`TypVal`]), so this is a
    /// load, not a jump table -- which is why the tables indexed by type
    /// (`num_errors`, `str_errors`, `type()`'s codes) and the hundreds of
    /// `== VAR_X` tests keep their spelling instead of becoming matches.
    #[inline(always)]
    pub const fn v_type(&self) -> crate::types::VarType {
        match self {
            TypVal::Unknown => VAR_UNKNOWN,
            TypVal::Number(_) => VAR_NUMBER,
            TypVal::String(_) => VAR_STRING,
            TypVal::Func(_) => VAR_FUNC,
            TypVal::List(_) => VAR_LIST,
            TypVal::Dict(_) => VAR_DICT,
            TypVal::Float(_) => VAR_FLOAT,
            TypVal::Bool(_) => VAR_BOOL,
            TypVal::Special(_) => VAR_SPECIAL,
            TypVal::Partial(_) => VAR_PARTIAL,
            TypVal::Blob(_) => VAR_BLOB,
        }
    }

    /// The empty value of `v_type`: a null pointer, a zero, a `v:false`.
    ///
    /// What c2rust's `{ .v_type = X }` designated initialiser was, and what a
    /// slot holds once its payload has been released. Panics on a `v_type`
    /// that is not one of the eleven, which is unreachable: they are the
    /// discriminants.
    pub(crate) const fn empty(v_type: crate::types::VarType) -> TypVal {
        match v_type {
            VAR_UNKNOWN => TypVal::Unknown,
            VAR_NUMBER => TypVal::Number(0),
            VAR_STRING => TypVal::string(None),
            VAR_FUNC => TypVal::func(None),
            VAR_LIST => TypVal::list(None),
            VAR_DICT => TypVal::dict(None),
            VAR_FLOAT => TypVal::Float(0.0),
            VAR_BOOL => TypVal::Bool(crate::types::kBoolVarFalse),
            VAR_SPECIAL => TypVal::Special(kSpecialVarNull),
            VAR_PARTIAL => TypVal::partial(None),
            VAR_BLOB => TypVal::blob(None),
            _ => panic!("a VarType outside the eleven the enum names"),
        }
    }

    /// Whether this value holds nothing to release, and clearing it would
    /// write back exactly what is already there.
    ///
    /// [`tv_clear`](crate::eval::typval::tv_clear)'s fast path, and the
    /// reason `Drop` after an explicit clear costs a compare rather than a
    /// second walk of the value.
    #[inline(always)]
    pub(crate) fn is_empty(&self) -> bool {
        match *self {
            TypVal::Unknown => true,
            TypVal::Number(n) => n == 0,
            // The exact `+0.0` a clear writes; `-0.0` compares equal to it
            // and is not the same value.
            TypVal::Float(f) => f.to_bits() == 0,
            TypVal::Bool(b) => b == crate::types::kBoolVarFalse,
            TypVal::Special(s) => s == kSpecialVarNull,
            TypVal::String(ref text) | TypVal::Func(ref text) => text.is_none(),
            TypVal::List(ref list) => list.is_none(),
            TypVal::Dict(ref dict) => dict.is_none(),
            TypVal::Partial(ref pt) => pt.is_none(),
            TypVal::Blob(ref blob) => blob.is_none(),
        }
    }

    /// Whether this value is callable: a funcref or a partial.
    ///
    /// Upstream's `tv_is_func`, which took the whole typval by value for
    /// two tag comparisons.
    #[inline(always)]
    pub(crate) fn is_func(&self) -> bool {
        matches!(self, TypVal::Func(_) | TypVal::Partial(_))
    }

    /// The payload as an address: what `printf("%p")` and `id()` answer.
    ///
    /// Upstream reads `vval.v_string` here with **no** tag test at all, and
    /// this keeps that answer for every kind: a pointer-shaped value gives
    /// its address, and a scalar gives its own bits, so `printf('%p', 42)`
    /// still says `0x2a`.  The two four-byte kinds are the one place the two
    /// can differ -- upstream reads four bytes of payload and four of
    /// padding, this reads the four that were written -- and `v:true`'s
    /// address was never a number anybody could use.
    ///
    /// Nothing is dereferenced: the answer is printed, and `id()` uses it as
    /// an identity, which is why `id()` of two distinct empty lists must
    /// differ and `id(v:_null_list)` must not.
    #[inline(always)]
    pub(crate) fn payload_address(&self) -> *const ::core::ffi::c_void {
        // The editor is 64-bit by construction, so a payload always fits an
        // address; upstream's read is the same eight bytes either way.
        let bits =
            |n: u64| ::core::ptr::without_provenance(usize::try_from(n).expect("a 64-bit host"));
        match self {
            TypVal::Unknown => ::core::ptr::null(),
            TypVal::String(text) | TypVal::Func(text) => text
                .as_ref()
                .map_or(::core::ptr::null(), |s| s.as_ptr().cast()),
            TypVal::List(list) => list
                .as_ref()
                .map_or(::core::ptr::null(), |l| l.as_ptr().cast_const().cast()),
            TypVal::Dict(dict) => dict
                .as_ref()
                .map_or(::core::ptr::null(), |d| d.as_ptr().cast_const().cast()),
            TypVal::Partial(pt) => pt
                .as_ref()
                .map_or(::core::ptr::null(), |p| p.as_ptr().cast_const().cast()),
            TypVal::Blob(blob) => blob
                .as_ref()
                .map_or(::core::ptr::null(), |b| b.as_ptr().cast_const().cast()),
            TypVal::Number(n) => bits(n.cast_unsigned()),
            TypVal::Float(f) => bits(f.to_bits()),
            TypVal::Bool(b) => bits(u64::from(*b)),
            TypVal::Special(s) => bits(u64::from(*s)),
        }
    }
}

/// The slot writers, generated over the enum's ten value-carrying variants.
///
/// `tv.write_x(v)` **overwrites a slot without releasing what it held**.
/// That is not what `*tv = TypVal::X(v)` does — an assignment drops the old
/// value — and the difference is the whole reason these exist: six hundred
/// call sites inherited C's rule that a slot is *filled*, never replaced, and
/// the ones that mean "replace" clear first, by hand, at a point of their own
/// choosing.  Some cannot do otherwise: the `nothing` sink frees a string and
/// *then* blanks the slot it came out of, so an assignment there would free
/// it twice.
///
/// Making a value rather than filling a slot is the variant itself now —
/// `TypVal::Number(n)`, not a `TypVal::number(n)` beside it — so these ten
/// rows are the whole family.
macro_rules! union_writers {
    ($(
        $variant:ident, $payload:ident, $ty:ty, $write_fn:ident, $what:expr;
    )*) => {
        impl TypVal {
            $(
                #[doc = concat!("Overwrite this slot with ", $what, ".")]
                #[doc = ""]
                #[doc = "Releases nothing: see [`union_writers`]."]
                #[inline(always)]
                pub(crate) fn $write_fn(&mut self, $payload: $ty) {
                    self.overwrite(TypVal::$variant($payload));
                }
            )*
        }
    };
}

union_writers! {
    Number,  number,    VarNumber,                write_number,    "an integer";
    Bool,    boolean,   BoolVarValue,             write_boolean,   "`v:true`/`v:false`";
    Special, special,   SpecialVarValue,          write_special,   "`v:null`";
    Float,   float,     Float,                    write_float,     "a float";
}

impl TypVal {
    /// Put `value` in this slot and **abandon** what was there.
    ///
    /// The engine under [`union_writers`]; see its note for why the release
    /// is the caller's and not this call's.
    #[inline(always)]
    pub(crate) fn overwrite(&mut self, value: TypVal) {
        // Forgotten, not dropped: these callers own the old value's release.
        ::core::mem::forget(::core::mem::replace(self, value));
    }

    /// Give up what this slot holds **without releasing it**: the payload
    /// has been handed to a new owner by pointer, and this slot must not
    /// free it.
    ///
    /// The C shape it replaces is a `*mut` copied out of a typval into a
    /// struct field with the typval simply left alone, which was free while
    /// a `TypVal` released nothing of its own accord.
    #[inline(always)]
    pub(crate) fn disown(&mut self) {
        self.overwrite(TypVal::Unknown);
    }

    /// Overwrite this slot with the empty value of `v_type`: the kind, and a
    /// null pointer or a zero.
    ///
    /// c2rust's lone `x.v_type = VAR_X;`, which is what every one of these
    /// sites was: a return slot is declared to be of a kind before the value
    /// that goes in it is known, and the paths that answer nothing leave the
    /// empty one behind.  Releases nothing, as [`union_writers`].
    #[inline(always)]
    pub(crate) fn write_empty(&mut self, v_type: crate::types::VarType) {
        self.overwrite(TypVal::empty(v_type));
    }

    /// Move this slot's value out, leaving the empty value of the same kind.
    ///
    /// The slot keeps saying what type it is — which is the whole point:
    /// `prepare_vimvar` blanks a `v:` variable and the tag it leaves behind
    /// is what tells `restore_vimvar` there was one, and what keeps a
    /// still-untyped `v:val` out of the `v:` dictionary.  What the slot no
    /// longer holds is anything to free: the caller owns that now, so
    /// dropping the slot afterwards is a no-op.
    ///
    /// The slot's lock stays where it is: it belongs to the place, not to
    /// the value that was sitting in it.
    #[inline(always)]
    pub(crate) fn take_value(&mut self) -> TypVal {
        let empty = TypVal::empty(self.v_type());
        ::core::mem::replace(self, empty)
    }

    /// Move the value out, leaving an unset slot.
    ///
    /// What stays behind is [`TypVal::Unknown`], the value a typval is born
    /// as.  This is the shape of every "hand the value on and reset the
    /// source" site the tree had spelled `*to = *from; tv_init(from)`.
    ///
    /// Where the slot has to keep saying what type it held, the take is
    /// [`TypVal::take_value`] instead.
    #[inline(always)]
    pub(crate) fn take(&mut self) -> TypVal {
        ::core::mem::replace(self, TypVal::Unknown)
    }
}

/// Lock status of `l`; a NULL list reads as `VarLock::Fixed`.
#[inline]
pub fn list_locked(l: Option<&List>) -> VarLock {
    l.map_or(VarLock::Fixed, List::lock)
}

/// Set the lock status of `l`.  A NULL list may only be "set" to
/// `VarLock::Fixed`, which is what a `debug_assert` here checks.
#[inline]
pub fn list_set_lock(l: Option<&mut List>, lock: VarLock) {
    match l {
        Some(l) => l.set_lock(lock),
        None => debug_assert!(lock == VarLock::Fixed),
    }
}

/// Number of items in `l`, as the `int` the family counts in; a NULL list is
/// empty.
#[inline]
pub fn list_len(l: Option<&List>) -> ::core::ffi::c_int {
    index_of(l.map_or(0, List::len))
}

/// Normalise a possibly negative list index against `l`'s length.
///
/// Returns an index in `0..list_len(l)`, or -1 when it is out of range.
#[inline]
pub fn list_uidx(l: Option<&List>, n: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let len = list_len(l);
    // A negative index counts back from the end.
    let n = if n < 0 { n + len } else { n };
    if n < 0 || n >= len { -1 } else { n }
}

/// Walk `l`'s items: upstream's `TV_LIST_ITER_CONST`.
///
/// It is **not** `TV_LIST_ITER`.  That macro re-reads the link *after* the
/// body has run, so a body that removes the item it is standing on still
/// advances correctly; this one is a borrow of the item array, so a body
/// that edits the list is a borrow error rather than a wrong answer.  Where
/// the body does edit the list, walk it by index instead.
#[inline]
pub(crate) fn list_iter(l: Option<&List>) -> ::core::slice::Iter<'_, ListItem> {
    list_items(l).iter()
}

/// Number of items in `d`, as the `long` the family counts in; a NULL
/// dictionary is empty.
#[inline]
pub fn dict_len(d: Option<&Dict>) -> ::core::ffi::c_long {
    d.map_or(0, |d| {
        ::core::ffi::c_long::try_from(d.len())
            .expect("a dictionary never holds more items than a long counts")
    })
}

/// Whether at least one watcher is registered on `d`.
#[inline]
pub fn dict_is_watched(d: Option<&Dict>) -> bool {
    d.is_some_and(|d| !d.watchers.is_empty())
}

/// The `DictItem` a dictionary hashtab slot names: upstream's
/// `TV_DICT_HI2DI`.
///
/// Safe, where it used to subtract an offset from the slot's key pointer
/// and hand back a wild pointer for an empty slot: the slot names the item.
/// An empty or removed slot answers null or the table's own tombstone
/// sentinel, neither of which may be dereferenced -- ask `hi.is_kept()`
/// first, exactly as before.
#[inline(always)]
pub(crate) fn tv_dict_hi2di(hi: Slot<DictEntry>) -> *mut DictItem {
    hi.hi_key.item()
}

/// Store `b` in `tv` as the return value: the slot takes the handle over.
///
/// The old contents are overwritten, not cleared, as every
/// [`union_writers`] row is.
#[inline(always)]
pub fn tv_blob_set_ret(tv: &mut TypVal, b: Option<BlobRef>) {
    tv.write_blob(b);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_cell::editor_state_lock;

    /// The payload `0xdead_beef` is what a hoisted read would follow as a
    /// pointer: the point of these cases is that the *kind* gates the read.
    #[test]
    fn a_reader_answers_only_for_its_own_tag() {
        let num = TypVal::Number(0xdead_beef);
        assert_eq!(num.as_number(), Some(0xdead_beef));
        assert!(num.list_ref().is_none());
        assert!(num.dict_ref().is_none());
        assert!(num.blob_ref().is_none());
        assert!(num.partial_ref().is_none());
        assert!(!num.is_string());
        assert_eq!(num.as_float(), None);
        assert_eq!(num.as_bool(), None);
        assert_eq!(num.as_special(), None);
    }

    #[test]
    fn the_null_form_is_the_option_form_with_the_familys_empty_value() {
        let num = TypVal::Number(0xdead_beef);
        assert!(num.list_or_null().is_null());
        assert!(num.dict_or_null().is_null());
        assert!(num.partial_shared().is_none());
        assert!(num.string_ref().is_none());
        assert!(num.func_name().is_none());
        assert!(num.text_or_name().is_none());
    }

    #[test]
    fn a_list_reads_back_as_the_pointer_it_was_given() {
        let _serial = editor_state_lock();
        let l = tv_list_alloc(0);
        let tv = TypVal::list(Some(l.clone()));
        assert!(tv.list_shared().is_some_and(|held| held.ptr_eq(&l)));
        assert_eq!(tv.list_or_null(), l.as_ptr());
        // Any other kind is not a list.
        assert!(TypVal::dict(Some(tv_dict_alloc())).list_ref().is_none());
    }

    /// Sixteen bytes. The size is the reason the lock lives on the slot: a
    /// `TypVal` is copied into every argument frame and every return slot
    /// in the interpreter, and twenty-four would be paid for on all of them.
    /// Where the tag sits is the core's test.
    #[test]
    fn a_value_is_sixteen_bytes() {
        assert_eq!(::core::mem::size_of::<TypVal>(), 16);
        assert_eq!(::core::mem::align_of::<TypVal>(), 8);
    }

    /// `%p` answers the payload under every kind, as upstream's untagged
    /// union read did.
    #[test]
    fn the_printed_address_is_the_payload_whatever_the_kind_is() {
        let _serial = editor_state_lock();
        let l = tv_list_alloc(0);
        let tv = TypVal::list(Some(l.clone()));
        assert_eq!(tv.payload_address().addr(), l.as_ptr().addr());
        assert_eq!(TypVal::Number(42).payload_address().addr(), 42);
        assert_eq!(TypVal::Unknown.payload_address().addr(), 0);
    }

    #[test]
    fn an_unknown_typval_reads_as_nothing_at_all() {
        let unknown = TV_INITIAL_VALUE;
        assert_eq!(unknown.as_number(), None);
        assert!(!unknown.is_string());
        assert!(unknown.list_or_null().is_null());
        assert!(unknown.text_or_name().is_none());
    }
}
