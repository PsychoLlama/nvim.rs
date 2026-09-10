//! The one-line accessors every other module reaches a `TypVal` through.
//!
//! Upstream declares these `static inline` in `typval.h`, so they are the part
//! of this file that is compiled into its callers rather than called; they keep
//! the `#[inline]` the transpile gave them for the same reason.  The four
//! `QUEUE_*` helpers are the intrusive-list macros `dv_watchers` is threaded
//! on.
//!
//! Every accessor takes the raw pointer its callers already hold — 500-odd call
//! sites across the tree pass `*mut List`/`*mut Dict` around, and the
//! `TypVal` family's layout is frozen by the LuaJIT unit specs.  What they
//! buy the rest of the family is that *nothing else* has to spell a field walk:
//! the children below reach a list through `tv_list_items`/`tv_list_iter`/
//! `tv_list_len`, never through `(*l).lv_items`.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::types::{HashTab, SlotEntry};
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
/// to dereferencing: the null-tolerant entry points (`tv_list_len`,
/// `tv_list_unref`, …) keep their `as_ref()` guard and take no handle.
pub(crate) type Tv = Live<TypVal>;
/// A live `List`; see [`Tv`].
pub(crate) type Ls = Live<List>;
/// A live `ListItem`; see [`Tv`].
pub(crate) type Li = Live<ListItem>;
/// A live `Dict`; see [`Tv`].
pub(crate) type Dt = Live<Dict>;
/// A live `DictItem`; see [`Tv`].
pub(crate) type Di = Live<DictItem>;
/// A live `Blob`; see [`Tv`].
pub(crate) type Bl = Live<Blob>;
/// A live `GArray`; see [`Tv`].
pub(crate) type Ga = Live<GArray>;
/// A live `Partial`; see [`Tv`].
pub(crate) type Pt = Live<Partial>;
/// A live `DictWatcher`; see [`Tv`].
pub(crate) type Dw = Live<DictWatcher>;
/// A live `ListWatch`; see [`Tv`].
pub(crate) type Lw = Live<ListWatch>;
/// A live `SortInfo`; see [`Tv`].
pub(crate) type Si = Live<SortInfo>;

/// The address of a field of `*p`, **computed rather than read**.
///
/// `&raw mut (*p).field` still requires an `unsafe` block today even though
/// nothing is dereferenced; `wrapping_byte_add` is the same arithmetic
/// spelled with a safe method, and it is defined for *every* pointer, null
/// and dangling included. The obligation the field's address carries belongs
/// to whoever dereferences it, and it is paid there.
///
/// This is [`Live::field_ptr`] without the handle, for the many places that
/// hold a bare pointer and only want to name one of its fields. The named
/// wrappers below spell the offsets, so no call site writes `offset_of!`.
#[inline(always)]
pub(crate) fn field_of<T, F>(p: *mut T, offset: usize) -> *mut F {
    p.wrapping_byte_add(offset).cast()
}

/// The address of a dictionary item's value; see [`field_of`].
#[inline(always)]
pub(crate) fn di_tv(di: *mut DictItem) -> *mut TypVal {
    field_of(di, ::core::mem::offset_of!(DictItem, di_tv))
}

/// The address of a dictionary item's lock; see [`field_of`].
///
/// The lock belongs to the *slot*, not to the value in it: `:lockvar d.k`
/// locks the place `d.k` names, and the value that replaces it is locked
/// too.  Every `DictItem`-prefixed struct carries the field at this offset.
#[inline(always)]
pub(crate) fn di_lock(di: *mut DictItem) -> *mut VarLock {
    field_of(di, ::core::mem::offset_of!(DictItem, di_lock))
}

/// The address of a dictionary's hash table; see [`field_of`].
#[inline(always)]
pub(crate) fn dv_hashtab(d: *mut Dict) -> *mut DictTab {
    field_of(d, ::core::mem::offset_of!(Dict, dv_hashtab))
}

/// The address of a dictionary's copy mark; see [`field_of`].
#[inline(always)]
pub(crate) fn dv_copyid(d: *mut Dict) -> *mut ::core::ffi::c_int {
    field_of(d, ::core::mem::offset_of!(Dict, dv_copy_id))
}

/// The address of a dictionary's watcher queue; see [`field_of`].
#[inline(always)]
pub(crate) fn dv_watchers(d: *mut Dict) -> *mut QUEUE {
    field_of(d, ::core::mem::offset_of!(Dict, watchers))
}

/// The address of a list's copy mark; see [`field_of`].
#[inline(always)]
pub(crate) fn lv_copyid(l: *mut List) -> *mut ::core::ffi::c_int {
    field_of(l, ::core::mem::offset_of!(List, lv_copy_id))
}

/// The address of a list's watcher chain head; see [`field_of`].
#[inline(always)]
pub(crate) fn lv_watch(l: *mut List) -> *mut *mut ListWatch {
    field_of(l, ::core::mem::offset_of!(List, lv_watch))
}

/// The address of a blob's byte array; see [`field_of`].
#[inline(always)]
pub(crate) fn bv_ga(b: *mut Blob) -> *mut GArray {
    field_of(b, ::core::mem::offset_of!(Blob, bv_ga))
}

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
/// already reads as absent everywhere (`tv_list_len(NULL) == 0`,
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
    String,  *mut ::core::ffi::c_char, as_string,    string_or_null = ::core::ptr::null_mut();
    Func,    *mut ::core::ffi::c_char, as_func_name, func_name_or_null = ::core::ptr::null_mut();
    Partial, *mut Partial,             as_partial,   partial_or_null = ::core::ptr::null_mut();
    Blob,    *mut Blob,                as_blob,      blob_or_null = ::core::ptr::null_mut();
}

impl TypVal {
    /// The handle, or `None` unless this is a list holding one.
    ///
    /// Hand-written where the other nine readers are generated, because the
    /// payload is a [`ListRef`] rather than a `Copy` pointer: it can be
    /// borrowed but never handed out, since a copy of it would be a
    /// reference nobody took.
    #[inline(always)]
    pub(crate) fn list_ref(&self) -> Option<&ListRef> {
        match self {
            TypVal::List(list) => list.as_ref(),
            _ => None,
        }
    }

    /// The dictionary, or `None` unless this is a `Dict` -- including the
    /// `v:_null_dict` case, which answers `Some(NULL)`.
    #[inline(always)]
    pub(crate) fn as_dict(&self) -> Option<*mut Dict> {
        match self {
            TypVal::Dict(dict) => Some(
                dict.as_ref()
                    .map_or(::core::ptr::null_mut(), DictRef::as_ptr),
            ),
            _ => None,
        }
    }

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
            VAR_STRING => TypVal::String(::core::ptr::null_mut()),
            VAR_FUNC => TypVal::Func(::core::ptr::null_mut()),
            VAR_LIST => TypVal::list(None),
            VAR_DICT => TypVal::dict(None),
            VAR_FLOAT => TypVal::Float(0.0),
            VAR_BOOL => TypVal::Bool(crate::types::kBoolVarFalse),
            VAR_SPECIAL => TypVal::Special(kSpecialVarNull),
            VAR_PARTIAL => TypVal::Partial(::core::ptr::null_mut()),
            VAR_BLOB => TypVal::Blob(::core::ptr::null_mut()),
            _ => panic!("a VarType outside the eleven the enum names"),
        }
    }

    /// The string under either variant that holds one — `String`'s text or
    /// `Func`'s function name — and NULL under any other.
    ///
    /// The arms that treat the two alike (`tv2bool`, `tv_copy`, the encoders)
    /// are the reason this exists; a site that means only one of them wants
    /// [`TypVal::string_or_null`] or [`TypVal::func_name_or_null`].
    #[inline(always)]
    pub(crate) fn string_or_func_name(&self) -> *mut ::core::ffi::c_char {
        match self {
            TypVal::String(text) | TypVal::Func(text) => *text,
            _ => ::core::ptr::null_mut(),
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
            TypVal::String(p) | TypVal::Func(p) => p.is_null(),
            TypVal::List(ref list) => list.is_none(),
            TypVal::Dict(ref dict) => dict.is_none(),
            TypVal::Partial(p) => p.is_null(),
            TypVal::Blob(p) => p.is_null(),
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
            TypVal::String(p) | TypVal::Func(p) => p.cast_const().cast(),
            TypVal::List(list) => list
                .as_ref()
                .map_or(::core::ptr::null(), |l| l.as_ptr().cast_const().cast()),
            TypVal::Dict(dict) => dict
                .as_ref()
                .map_or(::core::ptr::null(), |d| d.as_ptr().cast_const().cast()),
            TypVal::Partial(p) => p.cast_const().cast(),
            TypVal::Blob(p) => p.cast_const().cast(),
            TypVal::Number(n) => bits(n.cast_unsigned()),
            TypVal::Float(f) => bits(f.to_bits()),
            TypVal::Bool(b) => bits(u64::from(*b)),
            TypVal::Special(s) => bits(u64::from(*s)),
        }
    }
}

impl Li {
    /// The item's value, as a handle: it lives exactly as long as the item.
    #[inline(always)]
    pub(crate) fn tv(self) -> Tv {
        // SAFETY: `li_tv` is a field of the live item this handle names, and
        // `field_ptr` computes its address without borrowing the item.
        unsafe { Tv::new(self.field_ptr(::core::mem::offset_of!(ListItem, li_tv))) }
    }

    /// `li_tv`'s kind; see [`TypVal::v_type`].
    #[inline(always)]
    pub(crate) fn v_type(self) -> crate::types::VarType {
        self.li_tv.v_type()
    }

    /// `li_tv.vval.v_number`; see [`TypVal::as_number`].
    #[inline(always)]
    pub(crate) fn number(self) -> VarNumber {
        self.tv().number_or_zero()
    }

    /// `li_tv.vval.v_list`; see [`TypVal::list_or_null`].
    #[inline(always)]
    pub(crate) fn list(self) -> *mut List {
        self.tv().list_or_null()
    }
}

/// `_()`: the translation of a message, which is always a literal here.
///
/// Safe by construction rather than by promise: the argument is a `&CStr`,
/// so the NUL `gettext` looks for is part of the type.
#[inline(always)]
pub(crate) fn tr(msg: &'static ::core::ffi::CStr) -> *const ::core::ffi::c_char {
    gettext(msg).as_ptr()
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
    String,  string,    *mut ::core::ffi::c_char, write_string,    "an owned string";
    Func,    name,      *mut ::core::ffi::c_char, write_func_name, "an owned function name";
    Partial, partial,   *mut Partial,             write_partial,   "a partial";
    Blob,    blob,      *mut Blob,                write_blob,      "a blob";
}

impl TypVal {
    /// Put `value` in this slot and **abandon** what was there.
    ///
    /// The engine under [`union_writers`]; see its note for why the release
    /// is the caller's and not this call's.
    #[inline(always)]
    pub(crate) fn overwrite(&mut self, value: TypVal) {
        // SAFETY: `self` is a `&mut`, so the place is writable and aligned;
        // `write` does not read what was there, which is the point -- these
        // callers own the old value's release and some of them fill storage
        // that has never held one.
        unsafe { ::core::ptr::write(self, value) };
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

    /// Duplicate the value's bits, **sharing** whatever it points at.
    ///
    /// This is not a copy of the *value*: no string is duplicated and no
    /// reference count moves, so the payload now has two holders — and, with
    /// `Drop` live, two would-be releasers.  Every use of this is a place
    /// where upstream relies on two typvals naming one object for a bounded
    /// window: an argument vector that borrows the caller's values for the
    /// length of a call, a slot packed for output while the original is still
    /// the owner.
    ///
    /// A real copy — one that duplicates the string and takes the
    /// reference — is [`Clone`].
    ///
    /// # Safety
    /// The duplicate must not be released: exactly one of the two holders
    /// may, and it is the original. In practice the duplicate goes into a
    /// [`ManuallyDrop`](core::mem::ManuallyDrop) frame that outlives nothing.
    #[inline(always)]
    pub(crate) unsafe fn bit_copy(&self) -> TypVal {
        // SAFETY: the caller's promise above -- the duplicate is not released,
        // so the payload keeps its one owner.
        unsafe { ::core::ptr::read(self) }
    }
}

/// Where a dictionary pointer *lives*, for the walk that has to clear it.
///
/// [`TypvalSink`](crate::eval::typval_encode::TypvalSink)'s dictionary hooks
/// are handed the place rather than the value, because the `nothing` sink
/// releases the reference and blanks the slot it came out of. Two different
/// places are that slot — a `TypVal::Dict`'s payload, and a partial's
/// `pt_dict`, which is a bare `*mut Dict` and no typval at all — so one
/// pointer type cannot serve both.
#[derive(Clone, Copy)]
pub(crate) enum DictSlot {
    /// A typval holding the dictionary. Cleared, it is a `TypVal::Dict` over
    /// NULL: still a dictionary, holding none.
    Value(*mut TypVal),
    /// A partial's `pt_dict` field.
    Field(*mut *mut Dict),
}

impl DictSlot {
    /// The dictionary in the slot, or NULL.
    ///
    /// # Safety
    /// The slot must be live for the call, and a [`Value`](Self::Value) must
    /// hold a dictionary.
    #[inline(always)]
    pub(crate) unsafe fn get(self) -> *mut Dict {
        match self {
            // SAFETY: the caller's promise: a live slot.
            DictSlot::Value(tv) => unsafe { (*tv).dict_or_null() },
            DictSlot::Field(dictp) => unsafe { *dictp },
        }
    }

    /// Leave the slot holding no dictionary, releasing nothing.
    ///
    /// # Safety
    /// As [`get`](Self::get).
    #[inline(always)]
    pub(crate) unsafe fn clear(self) {
        match self {
            // SAFETY: the caller's promise: a live slot.
            DictSlot::Value(tv) => unsafe { (*tv).write_dict(None) },
            DictSlot::Field(dictp) => unsafe { *dictp = ::core::ptr::null_mut() },
        }
    }
}

/// True when an intrusive queue head has no entries.
///
/// # Safety
/// `q` must point at an initialised queue node — one that has been through
/// [`queue_init`] or spliced onto a queue that has.
#[inline(always)]
pub unsafe fn queue_empty(q: *const QUEUE) -> bool {
    unsafe { q == (*q).next }
}

/// Make `q` an empty queue head, pointing at itself both ways.
///
/// # Safety
/// `q` must point at writable `QUEUE`-sized storage. Anything it was
/// already linked into is left pointing at it, so only initialise a node
/// that is on no queue.
#[inline(always)]
pub unsafe fn queue_init(q: *mut QUEUE) {
    unsafe { (*q).next = q };
    unsafe { (*q).prev = q };
}

/// Splice `q` in as the last entry of the queue headed by `h`.
///
/// # Safety
/// `h` must be an initialised queue head and `q` a node that is on no
/// queue. Both must outlive the link.
#[inline(always)]
pub(crate) unsafe fn queue_insert_tail(h: *mut QUEUE, q: *mut QUEUE) {
    unsafe { (*q).next = h };
    unsafe { (*q).prev = (*h).prev };
    unsafe { (*(*q).prev).next = q };
    unsafe { (*h).prev = q };
}

/// Unlink `q` from whatever queue it is on.
///
/// # Safety
/// `q` must be a node currently on a queue, and its neighbours must still
/// be live. The node's own links are left stale, so it has to be
/// re-initialised before it is used as a head again.
#[inline(always)]
pub(crate) unsafe fn queue_remove(q: *mut QUEUE) {
    unsafe { (*(*q).prev).next = (*q).next };
    unsafe { (*(*q).next).prev = (*q).prev };
}

/// Lock status of `l`; a NULL list reads as `VarLock::Fixed`.
///
/// # Safety
/// `l` is null or points at a live list.
#[inline]
pub unsafe fn tv_list_locked(l: *const List) -> VarLock {
    unsafe { l.as_ref() }.map_or(VarLock::Fixed, |l| l.lv_lock)
}

/// Set the lock status of `l`.  A NULL list may only be "set" to `VarLock::Fixed`.
///
/// # Safety
/// `l` is null or points at a live list. A null list can only be "set" to
/// `VarLock::Fixed`, which is what a `debug_assert` here checks.
#[inline]
pub unsafe fn tv_list_set_lock(l: *mut List, lock: VarLock) {
    match unsafe { l.as_mut() } {
        Some(l) => l.lv_lock = lock,
        None => debug_assert!(lock == VarLock::Fixed),
    }
}

/// Set the copyID of `l`.  Does not expect a NULL list, be careful.
///
/// # Safety
/// `l` must point at a live list — **not** null, unlike its neighbours. The
/// `copyid` must be one the caller reserved from `get_copyID`.
#[inline]
pub unsafe fn tv_list_set_copyid(l: *mut List, copyid: ::core::ffi::c_int) {
    unsafe { (*l).lv_copy_id = copyid };
}

/// Number of items in `l`; a NULL list is empty.
///
/// # Safety
/// `l` is null or points at a live list.
#[inline]
pub unsafe fn tv_list_len(l: *const List) -> ::core::ffi::c_int {
    // SAFETY: the caller's promise: null or a live list.
    index_of(unsafe { tv_list_items(l) }.len())
}

/// The copyID of `l`.  Does not expect a NULL list, be careful.
///
/// # Safety
/// `l` must point at a live list — **not** null, unlike its neighbours.
#[inline]
pub unsafe fn tv_list_copyid(l: *const List) -> ::core::ffi::c_int {
    unsafe { (*l).lv_copy_id }
}

/// Normalise a possibly negative list index against `l`'s length.
///
/// Returns an index in `0..tv_list_len(l)`, or -1 when it is out of range.
///
/// # Safety
/// `l` is null or points at a live list.
#[inline]
pub unsafe fn tv_list_uidx(l: *const List, n: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let len = unsafe { tv_list_len(l) };
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
///
/// Takes an `Option` rather than a pointer because the macro's NULL handling
/// is half of what it does, and because that makes this one safe: `l.as_ref()`
/// at the call site costs the caller nothing, its `unsafe` block being
/// already open.
#[inline]
pub(crate) fn tv_list_iter(l: Option<&List>) -> ::core::slice::Iter<'_, ListItem> {
    l.map_or(&[][..], |l| &l.lv_items).iter()
}

/// [`tv_list_iter`] with the items writable.
#[inline]
pub(crate) fn tv_list_iter_mut(l: Option<&mut List>) -> ::core::slice::IterMut<'_, ListItem> {
    l.map_or(&mut [][..], |l| &mut l.lv_items).iter_mut()
}

/// Store `d` in `tv` as the return value, taking a reference to it.
///
/// # Safety
/// `tv` must point at a writable `TypVal` holding no value yet — the old
/// contents are overwritten, not cleared — and `d` is null or a live
/// dictionary the caller holds a reference to.
#[inline(always)]
pub unsafe fn tv_dict_set_ret(tv: &mut TypVal, d: *mut Dict) {
    // SAFETY: the caller's promise: a writable typval and a live dictionary.
    unsafe { Tv::new(tv) }.write_dict(unsafe { DictRef::retained(d) });
}

/// Number of items in `d`; a NULL dictionary is empty.
///
/// # Safety
/// `d` is null or points at a live dictionary.
#[inline]
pub unsafe fn tv_dict_len(d: *const Dict) -> ::core::ffi::c_long {
    unsafe { d.as_ref() }.map_or(0, |d| {
        ::core::ffi::c_long::try_from(d.dv_hashtab.ht_used)
            .expect("a dictionary never holds more items than a long counts")
    })
}

/// Whether at least one watcher is registered on `d`.
///
/// # Safety
/// `d` is null or points at a live dictionary whose watcher queue has been
/// initialised (every dictionary from `tv_dict_alloc` has).
#[inline]
pub unsafe fn tv_dict_is_watched(d: *const Dict) -> bool {
    unsafe { d.as_ref() }.is_some_and(|d| !unsafe { queue_empty(&raw const d.watchers) })
}

/// The key of `di`, which upstream reads as the plain `di->di_key`.
///
/// An item owns its key now, so this is a read rather than the pointer
/// arithmetic it used to be; the answer is still the NUL-terminated bytes
/// the hash table probes on.
///
/// # Safety
/// `di` points at a live item. The key borrows it.
#[inline(always)]
pub(crate) unsafe fn tv_dict_item_key(di: *const DictItem) -> *const ::core::ffi::c_char {
    // SAFETY: the caller's live item.
    unsafe { (*di).di_key.as_ptr() }
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

/// A walk over the occupied slots of a dictionary's hashtab.
///
/// See [`tv_dict_iter`].
pub(crate) struct TableIter<E: SlotEntry> {
    ht: *const HashTab<E>,
    idx: usize,
    todo: size_t,
}

/// A walk over the occupied slots of a dictionary's hashtab.
pub(crate) type DictIter = TableIter<DictEntry>;

impl<E: SlotEntry> Iterator for TableIter<E> {
    type Item = Slot<E>;

    #[inline]
    fn next(&mut self) -> Option<Slot<E>> {
        while self.todo != 0 {
            // The cursor is an index, and the slot is read out of the table
            // afresh each step: a body may take `&mut` to the table (every
            // `hash_remove` does), and the small run lives *in* the table,
            // so a pointer cursor would not survive the first removal.
            // SAFETY: the walk's table is live for the walk, and `todo`
            // live entries remain, so `idx` is one of its slots.
            let hi = unsafe { (*self.ht).slot(self.idx) };
            self.idx += 1;
            if hi.is_kept() {
                self.todo -= 1;
                return Some(hi);
            }
        }
        None
    }
}

/// Walk the occupied slots of `d`'s hashtab: upstream's `TV_DICT_ITER`, which
/// is `HASHTAB_ITER` plus a `TV_DICT_HI2DI`.
///
/// The item is yielded as the [`Slot`], not the `DictItem`, because the
/// bodies that remove entries need it for `hash_remove`; [`tv_dict_hi2di`] is
/// the other half.
///
/// The live-item count is snapshotted before the first step, exactly as the
/// macro does.  That is what lets a body remove entries as it goes — but only
/// with the hashtab locked, since an unlocked `hash_remove` may rehash and
/// renumber the slots underneath the walk.
///
/// # Safety
/// `d` points at a live dictionary that outlives the walk. A raw pointer and
/// not a reference: a body writes through the table (upstream's does), so the
/// walk must not be holding a borrow of it.
#[inline]
pub(crate) unsafe fn tv_dict_iter(d: *const Dict) -> DictIter {
    // SAFETY: the caller's live dictionary.
    unsafe { tv_ht_iter(&raw const (*d).dv_hashtab) }
}

/// [`tv_dict_iter`] over a bare hashtab: upstream's `HASHTAB_ITER`.
///
/// The variable scopes are reached both ways -- as a `Dict` and as the
/// `HashTab` inside it -- so both spellings exist. The contract is the
/// same one.
///
/// # Safety
/// As [`tv_dict_iter`], for the table rather than the dictionary.
#[inline]
pub(crate) unsafe fn tv_ht_iter<E: SlotEntry>(ht: *const HashTab<E>) -> TableIter<E> {
    TableIter {
        ht,
        idx: 0,
        // SAFETY: the caller's live table.
        todo: unsafe { (*ht).ht_used },
    }
}

/// Store `b` in `tv` as the return value, taking a reference to it.
///
/// # Safety
/// `tv` must point at a writable `TypVal` holding no value yet — the old
/// contents are overwritten, not cleared — and `b` is null or a live blob.
#[inline(always)]
pub unsafe fn tv_blob_set_ret(tv: &mut TypVal, b: *mut Blob) {
    // SAFETY: the caller's promise: a writable typval.
    let mut val = unsafe { Tv::new(tv) };
    val.write_blob(b);
    if let Some(b) = unsafe { b.as_mut() } {
        b.bv_refcount.retain();
    }
}

/// Length of `b`'s data in bytes; a NULL blob is empty.
///
/// # Safety
/// `b` is null or points at a live blob.
#[inline]
pub unsafe fn tv_blob_len(b: *const Blob) -> ::core::ffi::c_int {
    unsafe { b.as_ref() }.map_or(0, |b| b.bv_ga.ga_len)
}

/// The byte at `idx` in `b`.  `b` must be non-NULL and `idx` in range.
///
/// # Safety
/// `b` must point at a live blob and `idx` must be in `0..tv_blob_len(b)`.
/// Neither is checked.
#[inline(always)]
pub unsafe fn tv_blob_get(b: *const Blob, idx: ::core::ffi::c_int) -> uint8_t {
    unsafe { *(*b).bv_ga.ga_data.cast::<uint8_t>().offset(idx as isize) }
}

/// Store `c` at `idx` in `blob`.  `blob` must be non-NULL and `idx` in range.
///
/// # Safety
/// `blob` must point at a live blob and `idx` must be in
/// `0..tv_blob_len(blob)`. Neither is checked.
#[inline(always)]
pub unsafe fn tv_blob_set(blob: *mut Blob, idx: ::core::ffi::c_int, c: uint8_t) {
    unsafe { *(*blob).bv_ga.ga_data.cast::<uint8_t>().offset(idx as isize) = c };
}

/// The `DictWatcher` a queue node is embedded in (upstream's `QUEUE_DATA`).
///
/// Upstream spells this out as a function rather than the macro purely so it
/// can carry `FUNC_ATTR_NO_SANITIZE_ADDRESS`: ASan does not follow the pointer
/// arithmetic back out of the struct.
///
/// # Safety
/// `q` must be the `node` field of a live `DictWatcher` — a node from any
/// other queue yields a wild pointer, since the watcher is found by
/// subtracting an offset.
#[inline(always)]
pub unsafe fn tv_dict_watcher_node_data(q: *mut QUEUE) -> *mut DictWatcher {
    unsafe {
        q.cast::<::core::ffi::c_char>()
            .sub(::core::mem::offset_of!(DictWatcher, node))
    }
    .cast::<DictWatcher>()
}

/// The one step of a [`CallFrame`] that is not the frame's own business:
/// duplicating a value the caller keeps, so that the frame can *name* it.
///
/// It lives here rather than beside the type because `bit_copy` is this
/// module's, and a frame that never takes one is safe code end to end.
impl<const N: usize> CallFrame<N> {
    /// Append a bit copy of a value the caller keeps.
    pub(crate) fn push_borrowed(&mut self, tv: &TypVal) {
        // SAFETY: the duplicate is never released -- the slot's bit is
        // clear, so `truncate` disowns it rather than clearing it.
        self.push_naming(unsafe { tv.bit_copy() });
    }

    /// Append a bit copy of each of the caller's values.
    pub(crate) fn extend_borrowed(&mut self, tvs: &[TypVal]) {
        for tv in tvs {
            self.push_borrowed(tv);
        }
    }

    /// Put a bit copy of `tv` in front of everything already in the frame,
    /// which is what makes `base->Method(a)` a call of `Method(base, a)`.
    pub(crate) fn insert_borrowed_front(&mut self, tv: &TypVal) {
        self.push_borrowed(tv);
        self.rotate_last_to_front();
    }
}

#[cfg(test)]
mod tests {
    use ::core::mem::ManuallyDrop;

    use super::*;
    use crate::types::VarType;

    /// The value of `v_type` whose payload is `bits`, so that reading the
    /// wrong arm would answer something a real value never holds: the point
    /// of these cases is that the *kind* gates the read.
    ///
    /// [`ManuallyDrop`], because the payload is a made-up address and
    /// releasing it would follow it.
    fn tagged(v_type: VarType, bits: usize) -> ManuallyDrop<TypVal> {
        ManuallyDrop::new(tagged_inner(v_type, bits))
    }

    fn tagged_inner(v_type: VarType, bits: usize) -> TypVal {
        let p = ::core::ptr::without_provenance_mut::<()>(bits);
        match v_type {
            VAR_NUMBER => TypVal::Number(VarNumber::try_from(bits).expect("a small address")),
            // SAFETY: a made-up address, wrapped in a `ManuallyDrop` by
            // `tagged` so that nothing ever releases it.
            VAR_LIST => TypVal::list(unsafe { ListRef::owning(p.cast()) }),
            // SAFETY: as the list arm above.
            VAR_DICT => TypVal::dict(unsafe { DictRef::owning(p.cast()) }),
            VAR_BLOB => TypVal::Blob(p.cast()),
            VAR_PARTIAL => TypVal::Partial(p.cast()),
            VAR_STRING => TypVal::String(p.cast()),
            VAR_FUNC => TypVal::Func(p.cast()),
            other => panic!("no bogus payload for {other}"),
        }
    }

    #[test]
    fn a_reader_answers_only_for_its_own_tag() {
        // 0xdead_beef is what a hoisted read would follow as a pointer.
        let num = tagged(VAR_NUMBER, 0xdead_beef);
        assert_eq!(num.as_number(), Some(0xdead_beef));
        assert_eq!(num.as_list(), None);
        assert_eq!(num.as_dict(), None);
        assert_eq!(num.as_blob(), None);
        assert_eq!(num.as_partial(), None);
        assert_eq!(num.as_string(), None);
        assert_eq!(num.as_float(), None);
        assert_eq!(num.as_bool(), None);
        assert_eq!(num.as_special(), None);
    }

    #[test]
    fn the_null_form_is_the_option_form_with_the_familys_empty_value() {
        let num = tagged(VAR_NUMBER, 0xdead_beef);
        assert!(num.list_or_null().is_null());
        assert!(num.dict_or_null().is_null());
        assert!(num.blob_or_null().is_null());
        assert!(num.partial_or_null().is_null());
        assert!(num.string_or_null().is_null());
        assert!(num.func_name_or_null().is_null());
        assert!(num.string_or_func_name().is_null());
    }

    #[test]
    fn a_list_reads_back_as_the_pointer_it_was_given() {
        // Any address will do: nothing here dereferences it.
        let l = ::core::ptr::without_provenance_mut::<List>(0x1000);
        let tv = tagged(VAR_LIST, l.addr());
        assert_eq!(tv.as_list(), Some(l));
        assert_eq!(tv.list_or_null(), l);
        // The same bits under any other tag are not a list.
        assert_eq!(tagged(VAR_DICT, l.addr()).as_list(), None);
    }

    #[test]
    fn the_two_kinds_that_both_hold_a_string_stay_apart() {
        let text = c"x".as_ptr().cast_mut();
        let string = ManuallyDrop::new(TypVal::String(text));
        assert_eq!(string.as_string(), Some(text));
        assert_eq!(string.as_func_name(), None);
        assert_eq!(string.string_or_func_name(), text);

        let func = ManuallyDrop::new(TypVal::Func(text));
        assert_eq!(func.as_string(), None);
        assert_eq!(func.as_func_name(), Some(text));
        assert_eq!(func.string_or_func_name(), text);
    }

    /// Sixteen bytes, and the discriminant is the `VarType` code at offset
    /// zero.
    ///
    /// The size is the reason the lock lives on the slot: a `TypVal` is
    /// copied into every argument frame and every return slot in the
    /// interpreter, and twenty-four would be paid for on all of them. The
    /// offset is what `#[repr(C, u32)]` promises and what the generated
    /// `ffi.cdef` chunk describes to the unit fixtures.
    #[test]
    fn a_value_is_sixteen_bytes_tagged_by_its_var_type() {
        assert_eq!(::core::mem::size_of::<TypVal>(), 16);
        assert_eq!(::core::mem::align_of::<TypVal>(), 8);
        for tv in [
            TypVal::Unknown,
            TypVal::Number(1),
            TypVal::String(::core::ptr::null_mut()),
            TypVal::Func(::core::ptr::null_mut()),
            TypVal::list(None),
            TypVal::dict(None),
            TypVal::Float(1.0),
            TypVal::Bool(kBoolVarTrue),
            TypVal::Special(kSpecialVarNull),
            TypVal::Partial(::core::ptr::null_mut()),
            TypVal::Blob(::core::ptr::null_mut()),
        ] {
            // Every one of these is an empty value, so dropping it is free.
            // SAFETY: `repr(C, u32)` puts the discriminant first, and it is
            // a `u32`.
            let tag = unsafe { *(&raw const tv).cast::<VarType>() };
            assert_eq!(tag, tv.v_type());
        }
    }

    /// `%p` answers the payload under every kind, as upstream's untagged
    /// union read did.
    #[test]
    fn the_printed_address_is_the_payload_whatever_the_kind_is() {
        let l = ::core::ptr::without_provenance_mut::<List>(0x1000);
        assert_eq!(tagged(VAR_LIST, 0x1000).payload_address().addr(), l.addr());
        assert_eq!(TypVal::Number(42).payload_address().addr(), 42);
        assert_eq!(TypVal::Unknown.payload_address().addr(), 0);
    }

    #[test]
    fn an_unknown_typval_reads_as_nothing_at_all() {
        let unknown = TV_INITIAL_VALUE;
        assert_eq!(unknown.as_number(), None);
        assert_eq!(unknown.as_string(), None);
        assert!(unknown.list_or_null().is_null());
        assert!(unknown.string_or_func_name().is_null());
    }
}
