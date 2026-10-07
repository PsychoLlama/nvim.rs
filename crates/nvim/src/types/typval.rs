#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;
use crate::eval::gc::RootId;
pub use crate::eval::typval::{BlobRef, DictRef, DictTab, ItemSlot, ListRef, PartialRef};
use crate::memory::ThinCString;

pub type BoolVarValue = ::core::ffi::c_uint;
/// The two `VAR_BOOL` values: `v:false` and `v:true`.
pub const kBoolVarFalse: BoolVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
/// A Vimscript or Lua callable, held by whatever registered it.
///
/// Neither `Copy` nor `Clone`. Whichever variant is live -- a funcref name,
/// a `Partial` reference, a `LuaRef` -- is owned: [`Callback::duplicate`]
/// takes a second reference, and [`Callback::clear`] gives this one back.
/// The payloads sit in a [`ManuallyDrop`](::core::mem::ManuallyDrop) so
/// that no structure's drop glue releases them: the release stays the
/// explicit `clear` it has always been, which also gives back the funcref's
/// count on its function.
///
/// `#[repr(C, u32)]` with `None` at zero, because several aggregates that
/// hold one are born from all-zero bytes and never write the field:
/// `Buffer` from `Box::new_zeroed`, `QfList` and a channel's readers from
/// `mem::zeroed`, `'complete'`'s per-source array from `xcalloc`. The
/// discriminant may not move into a niche, or "no callback" stops being
/// what those zeroes mean.
#[repr(C, u32)]
pub enum Callback {
    None = 0,
    /// A function name, owned, plus a counted use of the function.
    Funcref(::core::mem::ManuallyDrop<ThinCString>) = 1,
    /// A partial, holding a reference of its own.
    Partial(::core::mem::ManuallyDrop<PartialRef>) = 2,
    /// A Lua value in that state's registry.
    Lua(LuaRef) = 3,
}

impl Callback {
    /// Whether anything is registered.
    pub const fn is_set(&self) -> bool {
        !matches!(self, Callback::None)
    }
}
/// One `dictwatcheradd()` registration, held by its dictionary's
/// `watchers`.
///
/// Shared (`Rc`) so that the walk firing it can keep it alive across its
/// callback while that callback edits the list; `busy` and `needs_free` are
/// cells for the same reason.
pub struct DictWatcher {
    pub callback: Callback,
    pub key_pattern: Vec<u8>,
    pub busy: ::core::cell::Cell<bool>,
    pub needs_free: ::core::cell::Cell<bool>,
}

impl Drop for DictWatcher {
    fn drop(&mut self) {
        self.callback.clear();
    }
}
pub type ListLenSpecials = ::core::ffi::c_int;
/// The negative lengths `tv_list_alloc` accepts in place of a real count.
pub const kListLenUnknown: ListLenSpecials = -1;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub type ScopeType = ::core::ffi::c_uint;
/// `dv_scope`: whether a dict is a scope dict, and whether it is the
/// function-local one `l:` refers to by default.
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub type SpecialVarValue = ::core::ffi::c_uint;
/// The only `VAR_SPECIAL` value: `v:null`.
pub const kSpecialVarNull: SpecialVarValue = 0;
/// `v_lock`, `dv_lock`, `lv_lock`, `bv_lock`: whether a value may be
/// changed.
///
/// Three states, so an enumeration rather than an `int` -- p22's `flags.rs`
/// ruling, one level down. A `VarLockStatus` that is neither 0, 1 nor 2 was
/// always unreachable; saying so in the type means the two `_` arms of
/// `value_check_lock` really are [`Fixed`](Self::Fixed) and the compiler
/// knows it.
///
/// `#[repr(u32)]` because it *is* the `unsigned int` C declared -- these
/// fields sit in `#[repr(C)]` structs the FFI edge reads, and the zero
/// pattern a `calloc`'d one starts life with is [`Unlocked`](Self::Unlocked).
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Hash)]
#[repr(u32)]
pub enum VarLock {
    /// Changeable.
    #[default]
    Unlocked = 0,
    /// `:lockvar` set this, and `:unlockvar` can clear it.
    Locked = 1,
    /// A slot that cannot be unlocked at all: `v:` variables, `a:`
    /// arguments, `a:000` and a `\=` expression's submatch list.
    Fixed = 2,
}

impl VarLock {
    /// Whether a change is forbidden -- either lock state answers yes.
    pub const fn is_locked(self) -> bool {
        !matches!(self, VarLock::Unlocked)
    }

    /// Upstream's `CHANGE_LOCK`: what this status becomes under
    /// `:lockvar` (`lock`) or `:unlockvar`.
    ///
    /// [`Fixed`](Self::Fixed) never changes -- that is a slot that cannot
    /// be unlocked at all, not merely one that is locked.
    pub const fn changed(self, lock: bool) -> VarLock {
        match self {
            VarLock::Fixed => VarLock::Fixed,
            _ if lock => VarLock::Locked,
            _ => VarLock::Unlocked,
        }
    }
}
pub type VarType = ::core::ffi::c_uint;
/// `TypVal::v_type` — which arm of `TypVal::vval` is live.
pub const VAR_UNKNOWN: VarType = 0;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_STRING: VarType = 2;
pub const VAR_FUNC: VarType = 3;
pub const VAR_LIST: VarType = 4;
pub const VAR_DICT: VarType = 5;
pub const VAR_FLOAT: VarType = 6;
pub const VAR_BOOL: VarType = 7;
pub const VAR_SPECIAL: VarType = 8;
pub const VAR_PARTIAL: VarType = 9;
pub const VAR_BLOB: VarType = 10;
/// The numbers `type()` answers with.  A separate enum from [`VarType`]:
/// funcref and partial share one code, and the codes are the documented
/// `v:t_*` values, so they are not free to follow `v_type`.
pub type VarTypeCode = ::core::ffi::c_uint;
pub const VAR_TYPE_NUMBER: VarTypeCode = 0;
pub const VAR_TYPE_STRING: VarTypeCode = 1;
pub const VAR_TYPE_FUNC: VarTypeCode = 2;
pub const VAR_TYPE_LIST: VarTypeCode = 3;
pub const VAR_TYPE_DICT: VarTypeCode = 4;
pub const VAR_TYPE_FLOAT: VarTypeCode = 5;
pub const VAR_TYPE_BOOL: VarTypeCode = 6;
pub const VAR_TYPE_SPECIAL: VarTypeCode = 7;
pub const VAR_TYPE_BLOB: VarTypeCode = 10;
/// Longest variable name `eval_variable` will look up without allocating.
pub const VAR_SHORT_LEN: ::core::ffi::c_uint = 20;
/// A reference count.
///
/// Every refcounted object in the tree -- lists, dictionaries, blobs,
/// partials, user functions, funccalls, argument lists, location-list
/// stacks -- counts its owners in one of these. On the wire it is still
/// the `int` C declared: `#[repr(transparent)]`, so no struct's layout
/// moves and the FFI edge sees an integer.
///
/// What it does **not** have is arithmetic operators. `+= 1` scattered
/// across eighty-two call sites is how a port leaks and double-frees;
/// naming the two directions [`retain`](Self::retain) and
/// [`release`](Self::release) puts every one of them through four lines
/// of code, and makes a stray increment a compile error rather than a
/// bug that shows up as a use-after-free three commands later.
///
/// [`release`](Self::release) answers what is left, because *every*
/// caller of it asks: the point of decrementing is to find out whether
/// this was the last owner.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash)]
#[repr(transparent)]
pub struct Refcount(::core::ffi::c_int);

impl Refcount {
    /// Nobody owns this yet. What `xcalloc` leaves behind, and what a
    /// dictionary allocated for a scope that is about to adopt it holds
    /// until the adoption.
    pub const ZERO: Refcount = Refcount(0);
    /// Exactly one owner: what an allocator hands back.
    pub const ONE: Refcount = Refcount(1);

    /// A count of `n` owners, for the handful of places that seed one
    /// with something other than 0 or 1 (`DO_NOT_FREE_CNT`).
    pub const fn new(n: ::core::ffi::c_int) -> Self {
        Refcount(n)
    }

    /// The count as an integer, for messages and assertions. Not for
    /// arithmetic -- there is no way to write the result back.
    pub const fn get(self) -> ::core::ffi::c_int {
        self.0
    }

    /// One more owner.
    pub fn retain(&mut self) {
        self.0 += 1;
    }

    /// One fewer owner; answers how many are left. Zero means the caller
    /// released the last reference and now owes the object its teardown.
    pub fn release(&mut self) -> ::core::ffi::c_int {
        self.0 -= 1;
        self.0
    }

    /// Release `n` owners at once. Only `unref_var_dict` needs it: a
    /// scope dictionary is seeded with `DO_NOT_FREE_CNT` so nothing can
    /// free it mid-scope, and giving it up is one bulk release.
    pub fn release_many(&mut self, n: ::core::ffi::c_int) -> ::core::ffi::c_int {
        self.0 -= n;
        self.0
    }

    /// Whether somebody other than the caller holds a reference, so
    /// dropping the caller's cannot free the object.
    pub const fn is_shared(self) -> bool {
        self.0 > 1
    }
}

/// [`Refcount`] for the objects the event loop counts in a `size_t`:
/// processes, channels, terminals, write buffers and autocommand
/// patterns. A separate type only because the width is: an over-release
/// wraps to `SIZE_MAX` here and to `-1` there, and one of those two
/// behaviours is what each of these objects already had.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash)]
#[repr(transparent)]
pub struct RefcountSize(size_t);

impl RefcountSize {
    /// Nobody owns this yet.
    pub const ZERO: RefcountSize = RefcountSize(0);
    /// Exactly one owner.
    pub const ONE: RefcountSize = RefcountSize(1);

    /// A count of `n` owners. `wstream_new_buffer` takes the number of
    /// writers that will share the payload as an argument, so this one
    /// is not always 0 or 1.
    pub const fn new(n: size_t) -> Self {
        RefcountSize(n)
    }

    /// The count as an integer, for messages and assertions.
    pub const fn get(self) -> size_t {
        self.0
    }

    /// One more owner.
    pub fn retain(&mut self) {
        self.0 += 1;
    }

    /// One fewer owner; answers how many are left. The subtraction is
    /// checked, not wrapping: releasing a reference nobody holds is a
    /// bug, and a debug build should say so where it happens rather
    /// than hand back `SIZE_MAX` for the caller to compare against 0.
    pub fn release(&mut self) -> size_t {
        self.0 -= 1;
        self.0
    }

    /// Whether somebody other than the caller holds a reference.
    pub const fn is_shared(self) -> bool {
        self.0 > 1
    }
}

#[cfg(test)]
mod refcount_tests {
    use super::*;

    /// The FFI edge reads these fields through `unit-cdefs.h`, where
    /// `Refcount` is a `typedef` for `int` and `VarLock` one for `unsigned
    /// int`. A newtype that stopped being the integer it wraps would put
    /// every `#[repr(C)]` struct that has one out of step with the header
    /// silently, so say it here.
    #[test]
    fn the_wrappers_are_still_the_integers_they_wrap() {
        assert_eq!(
            size_of::<Refcount>(),
            size_of::<::core::ffi::c_int>(),
            "Refcount must stay ABI-identical to int"
        );
        assert_eq!(align_of::<Refcount>(), align_of::<::core::ffi::c_int>());
        assert_eq!(size_of::<RefcountSize>(), size_of::<size_t>());
        assert_eq!(align_of::<RefcountSize>(), align_of::<size_t>());
        assert_eq!(size_of::<VarLock>(), size_of::<::core::ffi::c_uint>());
        assert_eq!(align_of::<VarLock>(), align_of::<::core::ffi::c_uint>());
    }

    /// Zeroed memory is what `xcalloc` hands the allocators, and every
    /// refcounted object is filled in from there. Both wrappers and the
    /// lock have to read as their at-rest value out of it, or a freshly
    /// allocated dictionary starts life locked or already referenced.
    #[test]
    fn zeroed_memory_reads_as_the_at_rest_value() {
        assert_eq!(Refcount::default(), Refcount::ZERO);
        assert_eq!(RefcountSize::default(), RefcountSize::ZERO);
        assert_eq!(VarLock::default(), VarLock::Unlocked);
        assert_eq!(Refcount::ZERO.get(), 0);
        assert_eq!(VarLock::Unlocked as u32, 0);
        assert_eq!(VarLock::Locked as u32, 1);
        assert_eq!(VarLock::Fixed as u32, 2);
    }

    /// `release` answers what is left, which is the whole reason callers
    /// stopped writing the decrement themselves: the question every one of
    /// them asks is "was that the last owner".
    #[test]
    fn release_answers_what_is_left() {
        let mut count = Refcount::ONE;
        count.retain();
        assert_eq!(count.get(), 2);
        assert!(count.is_shared());
        assert_eq!(count.release(), 1);
        assert!(!count.is_shared());
        assert_eq!(count.release(), 0);
        assert_eq!(count, Refcount::ZERO);

        let mut sized = RefcountSize::ONE;
        sized.retain();
        assert!(sized.is_shared());
        assert_eq!(sized.release(), 1);
        assert_eq!(sized.release(), 0);
        assert_eq!(sized, RefcountSize::ZERO);
    }

    /// `unref_var_dict`'s bulk release: a scope dictionary is seeded with
    /// `DO_NOT_FREE_CNT` and gives all but one of them up at once.
    #[test]
    fn release_many_gives_up_a_run_of_references() {
        let mut count = Refcount::new(1_073_741_823);
        assert_eq!(count.release_many(1_073_741_822), 1);
        assert_eq!(count, Refcount::ONE);
    }

    /// `CHANGE_LOCK`: `:unlockvar` cannot reach a `VAR_FIXED` slot, which
    /// is what keeps `v:` variables and `a:` arguments read-only.
    #[test]
    fn fixed_survives_both_lockvar_and_unlockvar() {
        assert_eq!(VarLock::Unlocked.changed(true), VarLock::Locked);
        assert_eq!(VarLock::Locked.changed(false), VarLock::Unlocked);
        assert_eq!(VarLock::Locked.changed(true), VarLock::Locked);
        assert_eq!(VarLock::Fixed.changed(true), VarLock::Fixed);
        assert_eq!(VarLock::Fixed.changed(false), VarLock::Fixed);

        assert!(!VarLock::Unlocked.is_locked());
        assert!(VarLock::Locked.is_locked());
        assert!(VarLock::Fixed.is_locked());
    }
}

/// A `Blob`: a byte vector with a reference count and a lock.
pub struct Blob {
    pub bv_data: Vec<u8>,
    pub bv_refcount: Refcount,
    pub bv_lock: VarLock,
}
/// A `Dict`.
///
/// Neither `Copy` nor `Clone`: it owns its hashtab -- and through it the
/// items every key points into -- its watchers and a Lua table
/// reference, none of which a second holder may free.
pub struct Dict {
    pub dv_lock: VarLock,
    pub dv_scope: ScopeType,
    pub dv_refcount: Refcount,
    pub dv_copy_id: ::core::ffi::c_int,
    pub dv_hashtab: DictTab,
    pub dv_copydict: *mut Dict,
    /// Where the collector's registry holds this dictionary, or
    /// `RootId::NONE` for one the allocator never handed out -- every scope
    /// dictionary initialised in place.
    pub dv_root: RootId,
    pub watchers: Vec<::std::rc::Rc<DictWatcher>>,
    pub lua_table_ref: LuaRef,
}
/// One call of a user function: its `a:`/`l:` scopes, its caller (kept by the
/// funccall table, see `eval/userfunc/frames.rs`), its place in the body and
/// its return value.
///
/// Shared (`Rc`) by the table and whoever is using it across user code; the
/// fields that change during the call are cells, each borrowed only for the
/// access that asks.
pub struct FuncCall {
    /// The function being run.
    pub(crate) func: ::std::rc::Rc<UserFunc>,
    /// The scope dictionaries and `a:000`.
    pub(crate) scopes: crate::eval::userfunc::FrameScopes,
    /// Whether `l:`/`a:` are set up: a variable lookup made before that --
    /// a breakpoint expression evaluated as the call starts -- sees no
    /// function scope, as upstream's did while `l:`'s count was still 0.
    pub(crate) scope_ready: ::core::cell::Cell<bool>,
    /// The body line the next read hands out.
    pub(crate) linenr: ::core::cell::Cell<::core::ffi::c_int>,
    /// `:return` ran (and is not pending behind a `:finally`).
    pub(crate) returned: ::core::cell::Cell<bool>,
    /// What `:return` set.
    pub(crate) rettv: ::core::cell::RefCell<TypVal>,
    pub(crate) breakpoint: ::core::cell::Cell<LineNr>,
    pub(crate) dbg_tick: ::core::cell::Cell<::core::ffi::c_int>,
    /// The `:if`/`:while` nesting of the line that made the call.
    pub(crate) level: ::core::ffi::c_int,
    /// The calls `:defer` recorded, made newest first on the way out.
    pub(crate) defer: ::core::cell::RefCell<Vec<crate::eval::userfunc::Defer>>,
    pub(crate) prof_child: ::core::cell::Cell<ProfTime>,
    /// Its own place in the funccall table, which also knows its caller.
    pub(crate) id: FcId,
    /// How many closures captured this call's scope.
    pub(crate) refcount: ::core::cell::Cell<::core::ffi::c_int>,
    pub(crate) copy_id: ::core::cell::Cell<::core::ffi::c_int>,
    /// The functions defined as closures over this call. A slot is emptied
    /// when its function lets go of the scope.
    pub(crate) ufuncs: ::core::cell::RefCell<Vec<::std::rc::Weak<UserFunc>>>,
}
/// An item's short keys -- every fixed `a:` name -- fit inline.
#[cfg(not(randomized_layout))]
const _: () = {
    use crate::types::{DictItem, DictKey};
    assert!(
        DictKey::INLINE_MAX >= 20,
        "a funccall's short names fit inline"
    );
    assert!(::core::mem::size_of::<DictItem>() <= 48);
};
/// One item of a [`List`].
///
/// `li_lock` is the *slot's* lock -- `:lockvar l[0]` locks the place, not the
/// value that happens to sit in it -- so it lives here rather than in the
/// value.  See [`TypVal`].
///
/// Twenty-four bytes and no links: the list owns its items in an array, so
/// an item's identity is its index and there is nothing to thread.
pub struct ListItem {
    pub li_tv: TypVal,
    pub li_lock: VarLock,
}

impl ListItem {
    /// An unlocked slot holding `tv`.  Every push starts here; the two
    /// places that want a locked one (`a:000`, the submatch list) set
    /// `li_lock` afterwards.
    pub const fn new(li_tv: TypVal) -> ListItem {
        ListItem {
            li_tv,
            li_lock: VarLock::Unlocked,
        }
    }
}

/// A Vimscript List: an array of values, reference counted, and the cursors
/// (`lv_watch`) of whatever `:for` loops are walking it.
///
/// The items are owned outright -- dropping the list drops them -- which is
/// why an item's identity outside the array is an *index* and not an
/// address.  [`ListWatch`] is the one identity that has to survive an edit,
/// and `tv_list_watch_*` is what keeps it pointing at the same item.
pub struct List {
    pub lv_items: Vec<ListItem>,
    /// The `:for` cursors on this list, each named by its `id`.
    pub lv_watch: Vec<ListWatch>,
    pub lv_copylist: *mut List,
    /// Where the collector's registry holds this list, or `RootId::NONE`
    /// for one the allocator never handed out.
    pub lv_root: RootId,
    pub lv_refcount: Refcount,
    pub lv_copy_id: ::core::ffi::c_int,
    pub lv_lock: VarLock,
    pub lua_table_ref: LuaRef,
}

impl List {
    /// An empty, unreferenced list on no chain: what a fresh allocation is
    /// written with, and what a `List` embedded in another structure
    /// (`FuncCall`'s `a:000`, a `\=` expression's submatch list) starts as.
    ///
    /// A `const` and not `Default` because the embedders reach it from a
    /// `const` context, and because a zeroed allocation is *not* a valid
    /// `List` any more -- `lv_items` is a `Vec`, whose pointer is never
    /// null.
    pub const fn empty() -> List {
        List {
            lv_items: Vec::new(),
            lv_watch: Vec::new(),
            lv_copylist: ::core::ptr::null_mut(),
            lv_root: RootId::NONE,
            lv_refcount: Refcount::ZERO,
            lv_copy_id: 0,
            lv_lock: VarLock::Unlocked,
            // `LUA_NOREF`: no Lua table mirrors this list.
            lua_table_ref: -2,
        }
    }
}

/// A `:for` loop's cursor into the list it is walking: the index of the item
/// it will hand out next, or [`ListWatch::ENDED`] once it has run off the
/// end.
///
/// The list owns its cursors; a loop holds the `id` its cursor was
/// registered under and asks the list for the index.
#[derive(Clone)]
pub struct ListWatch {
    /// Unique among the cursors of one list.
    pub id: u32,
    pub index: ::core::ffi::c_int,
}

impl ListWatch {
    /// The cursor is past the last item.  Upstream spelled this a NULL
    /// `lw_item`, and it is *sticky*: a loop whose body appends to the list
    /// it is walking still ends, because the cursor ran off the end before
    /// the item existed.
    pub const ENDED: ::core::ffi::c_int = -1;
}
/// A partial: a function plus bound arguments and an optional `self` dict.
///
/// It owns its name, its arguments and its dictionary reference; the last
/// [`PartialRef`] going is what releases them (`partial_free`, in upstream's
/// order). Built in place with [`PartialRef::new`].
pub struct Partial {
    pub pt_refcount: Refcount,
    pub pt_copy_id: ::core::ffi::c_int,
    /// The function's name, holding a reference to it by name. `None` when
    /// `pt_func` is the function instead.
    pub pt_name: Option<ThinCString>,
    pub pt_func: Option<::std::rc::Rc<UserFunc>>,
    pub pt_auto: bool,
    pub pt_argv: Vec<TypVal>,
    pub pt_dict: Option<DictRef>,
}
pub type ScriptId = ::core::ffi::c_int;
#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct ScriptCtx {
    pub sc_sid: ScriptId,
    pub sc_seq: ::core::ffi::c_int,
    pub sc_lnum: LineNr,
    pub sc_chan: uint64_t,
}

impl ScriptCtx {
    /// No script is running: the all-zero context every table of script
    /// contexts starts out holding. A `const` because most of its uses are
    /// `static` initialisers, where `Default` cannot reach.
    pub const NONE: ScriptCtx = ScriptCtx {
        sc_sid: 0,
        sc_seq: 0,
        sc_lnum: 0,
        sc_chan: 0,
    };

    /// The same context under a different script id.
    ///
    /// A `Copy` cell's field write is a read-modify-write, and spelling it
    /// as one expression keeps the "which field" out of the caller's
    /// bookkeeping -- `Pos::with_col`'s shape.
    pub fn with_sid(self, sc_sid: ScriptId) -> Self {
        ScriptCtx { sc_sid, ..self }
    }

    /// The same context at a different line inside the script.
    pub fn with_lnum(self, sc_lnum: LineNr) -> Self {
        ScriptCtx { sc_lnum, ..self }
    }

    /// The same context under a different sourcing sequence number.
    pub fn with_seq(self, sc_seq: ::core::ffi::c_int) -> Self {
        ScriptCtx { sc_seq, ..self }
    }
}

impl Default for ScriptCtx {
    fn default() -> Self {
        Self::NONE
    }
}
/// A Vimscript value.
///
/// Eleven kinds, one payload each, sixteen bytes: the tag and, beside it, the
/// eight the payload needs.  `#[repr(C, u32)]` because that layout is the one
/// the C had -- a `v_type` word followed by a union -- and because it lets the
/// discriminants *be* the [`VarType`] codes, so `type()`, the error tables and
/// the `tv_check_for_*_arg` family keep reading a number rather than matching
/// eleven arms.  [`v_type`](Self::v_type) is that number.
///
/// **The lock is not here.** `:lockvar l[0]` locks the place, not the value in
/// it, so the lock lives on the [`ListItem`]/[`DictItem`](crate::types::DictItem)
/// -- which is also what keeps this type at sixteen bytes, since an enum has
/// nowhere to put a second field.
///
/// [`VAR_STRING`] and [`VAR_FUNC`] shared `v_string` in the union and are two
/// variants here: the string a funcref holds is a *name*, and copying one
/// takes a reference to the function as well.
///
/// The value owns every payload: `Drop` is `tv_clear` and `Clone` is
/// `tv_copy`. A string is a [`ThinCString`] -- one word, so the value stays
/// sixteen bytes; `Option`'s `None` is the null string.
///
/// **Every payload is released by [`TypVal`]'s own `Drop`, never by field
/// glue.** The strings and the container handles are held in a
/// [`ManuallyDrop`] to say so:
/// `tv_clear` walks a value *iteratively* and takes each handle out of its
/// slot itself, so a field destructor after it would be dead code -- and the
/// compiler cannot know that, so it emits the switch anyway and every
/// implicit drop of a value pays for it (0.5 % of a non-eval bench, measured).
/// The invariant a new payload has to keep is the whole of it: whatever owns
/// something is released by the clear, and the wrapper keeps the compiler
/// from doing it a second time.
///
/// [`ManuallyDrop`]: ::core::mem::ManuallyDrop
#[repr(C, u32)]
pub enum TypVal {
    /// No value: what a fresh slot holds, and what one is left as after being
    /// moved out of.
    Unknown = VAR_UNKNOWN,
    /// An integer.
    Number(VarNumber) = VAR_NUMBER,
    /// A string, owned; `None` is `v:_null_string`.
    String(::core::mem::ManuallyDrop<Option<ThinCString>>) = VAR_STRING,
    /// A funcref: an owned function name, plus a reference to the function.
    /// `None` is a funcref that names nothing (`v:_null_function`'s shape
    /// before it is resolved, and what a clear leaves).
    Func(::core::mem::ManuallyDrop<Option<ThinCString>>) = VAR_FUNC,
    /// A list, owned as one reference; `None` is `v:_null_list`.
    List(::core::mem::ManuallyDrop<Option<ListRef>>) = VAR_LIST,
    /// A dictionary, owned as one reference; `None` is `v:_null_dict`.
    Dict(::core::mem::ManuallyDrop<Option<DictRef>>) = VAR_DICT,
    /// A float.
    Float(Float) = VAR_FLOAT,
    /// `v:true` or `v:false`.
    Bool(BoolVarValue) = VAR_BOOL,
    /// `v:null`.
    Special(SpecialVarValue) = VAR_SPECIAL,
    /// A partial, owned as one reference; `None` is a funcref that could
    /// not be built.
    Partial(::core::mem::ManuallyDrop<Option<PartialRef>>) = VAR_PARTIAL,
    /// A blob, owned as one reference; `None` is `v:_null_blob`.
    Blob(::core::mem::ManuallyDrop<Option<BlobRef>>) = VAR_BLOB,
}
/// A user function: `:function`, a numbered dictionary function, a lambda,
/// or a Lua reference given a Vimscript name.
///
/// Shared (`Rc`) by whatever can reach it: the function table, a partial
/// that names it by pointer (`funcref()`, a lambda), the call running it, and
/// the closures a call registered. That sharing is only *memory*; the
/// editor-visible life of a function is still `refcount`/`calls`, upstream's
/// counts, which decide when it is cleared and taken out of the table
/// ([`crate::eval::userfunc::func_clear_free`]). The fields that change after
/// the definition are cells, borrowed only for the access that asks, never
/// across a call that can run user code.
pub struct UserFunc {
    /// The name the table holds it under: a global name, `<SNR>`-mangled
    /// (`K_SPECIAL KS_EXTRA KE_SNR` + `123_name`), a number, `<lambda>N`.
    pub(crate) name: ThinCString,
    /// `<SNR>123_name`, the printable form of a mangled name.
    pub(crate) name_exp: Option<ThinCString>,
    pub(crate) flags: ::core::cell::Cell<crate::eval::userfunc::FuncFlags>,
    /// How many calls of it are running.
    pub(crate) calls: ::core::cell::Cell<::core::ffi::c_int>,
    pub(crate) cleared: ::core::cell::Cell<bool>,
    pub(crate) refcount: ::core::cell::Cell<Refcount>,
    /// The arguments, their defaults and the body, replaced as a whole when
    /// the function is redefined in place.
    pub(crate) body: ::core::cell::RefCell<::std::rc::Rc<FuncBody>>,
    pub(crate) luaref: ::core::cell::Cell<LuaRef>,
    pub(crate) script_ctx: ::core::cell::Cell<ScriptCtx>,
    /// The call whose scope a closure captured.
    pub(crate) scoped: ::core::cell::Cell<Option<FcId>>,
    pub(crate) prof: ::core::cell::RefCell<FuncProfile>,
}

/// What `:function` collected between the parentheses and `:endfunction`.
#[derive(Default)]
pub(crate) struct FuncBody {
    /// The argument names.
    pub args: Vec<Box<[u8]>>,
    /// The source of each default, right-aligned with `args`.
    pub def_args: Vec<Box<[u8]>>,
    /// The body, one entry per source line; `None` for a continuation
    /// line, so that an index is a line number.
    pub lines: Vec<Option<Box<[u8]>>>,
    /// Whether a `...` was declared.
    pub varargs: bool,
}

/// A function's `:profile` counters, the per-line ones sized to its body.
#[derive(Default)]
pub(crate) struct FuncProfile {
    pub profiling: bool,
    pub initialized: bool,
    pub tm_count: ::core::ffi::c_int,
    pub tm_total: ProfTime,
    pub tm_self: ProfTime,
    pub tm_children: ProfTime,
    pub tml_count: Vec<::core::ffi::c_int>,
    pub tml_total: Vec<ProfTime>,
    pub tml_self: Vec<ProfTime>,
    pub tml_start: ProfTime,
    pub tml_children: ProfTime,
    pub tml_wait: ProfTime,
    /// The body line being timed, or -1.
    pub tml_idx: ::core::ffi::c_int,
    pub tml_execed: bool,
}

impl UserFunc {
    /// The name the table holds it under.
    pub fn name(&self) -> &ThinCString {
        &self.name
    }

    /// The name to show a user: `<SNR>123_name` for a script-local one.
    pub fn printable_name(&self) -> &ThinCString {
        self.name_exp.as_ref().unwrap_or(&self.name)
    }

    /// The body as it stands; a redefinition replaces it, and a holder of
    /// this one keeps reading the text it started on.
    pub(crate) fn body(&self) -> ::std::rc::Rc<FuncBody> {
        self.body.borrow().clone()
    }

    /// Whether any of `flags` is set.
    pub fn has_flag(&self, flags: crate::eval::userfunc::FuncFlags) -> bool {
        self.flags.get().has(flags)
    }

    /// One more counted holder.
    pub fn retain(&self) {
        let mut count = self.refcount.get();
        count.retain();
        self.refcount.set(count);
    }

    /// One counted holder fewer; answers how many are left.
    pub fn release(&self) -> ::core::ffi::c_int {
        let mut count = self.refcount.get();
        let left = count.release();
        self.refcount.set(count);
        left
    }
}

pub type UVarNumber = uint64_t;
pub type VarNumber = int64_t;
