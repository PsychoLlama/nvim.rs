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

pub type BoolVarValue = ::core::ffi::c_uint;
/// The two `VAR_BOOL` values: `v:false` and `v:true`.
pub const kBoolVarFalse: BoolVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
/// A Vimscript or Lua callable, held by whatever registered it.
///
/// Not `Copy`. Whichever variant is live -- a funcref name, a `Partial`
/// refcount, a `LuaRef` -- is owned, and `callback_free` releases it.
/// Duplicating one without `callback_copy` is a second owner of the same
/// reference, so the copies that remain say `.clone()` and are visible.
///
/// `#[repr(C, u32)]` with `None` at zero, because several aggregates that
/// hold one are born from all-zero bytes and never write the field:
/// `Buffer` from `Box::new_zeroed`, `QfList` and a channel's readers from
/// `mem::zeroed`, `'complete'`'s per-source array from `xcalloc`. The
/// discriminant may not move into a niche, or "no callback" stops being
/// what those zeroes mean.
#[derive(Clone)]
#[repr(C, u32)]
pub enum Callback {
    None = 0,
    /// A function name, owned: `func_unref` and `xfree` release it.
    Funcref(*mut ::core::ffi::c_char) = 1,
    /// A partial, holding a reference of its own.
    Partial(*mut Partial) = 2,
    /// A Lua value in that state's registry.
    Lua(LuaRef) = 3,
}

impl Callback {
    /// Whether anything is registered.
    pub const fn is_set(&self) -> bool {
        !matches!(self, Callback::None)
    }
}
/// One `dictwatcheradd()` registration, linked into its dict's queue.
///
/// Not `Copy`: it owns `key_pattern`, its `callback`, and a queue node whose
/// neighbours point back at this address.
#[derive(Clone)]
pub struct DictWatcher {
    pub callback: Callback,
    pub key_pattern: *mut ::core::ffi::c_char,
    pub key_pattern_len: size_t,
    pub node: QUEUE,
    pub busy: bool,
    pub needs_free: bool,
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
    /// arguments, and the static lists `list_init_static` hands out.
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

pub struct Blob {
    pub bv_ga: GArray,
    pub bv_refcount: Refcount,
    pub bv_lock: VarLock,
}
/// A `Dict`.
///
/// Neither `Copy` nor `Clone`: it owns its hashtab -- and through it the
/// items every key points into -- its watcher queue and a Lua table
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
    pub watchers: QUEUE,
    pub lua_table_ref: LuaRef,
}
/// Not `Clone`: it holds `l:` and `a:` by value, and a dictionary owns the
/// items its hash table indexes.
pub struct FuncCall {
    pub fc_func: *mut UserFunc,
    pub fc_linenr: ::core::ffi::c_int,
    pub fc_returned: ::core::ffi::c_int,
    pub fc_fixvar: [DictItem; 12],
    pub fc_l_vars: Dict,
    pub fc_l_vars_var: ScopeDictItem,
    pub fc_l_avars: Dict,
    pub fc_l_avars_var: ScopeDictItem,
    pub fc_l_varlist: List,
    pub fc_rettv: *mut TypVal,
    pub fc_breakpoint: LineNr,
    pub fc_dbg_tick: ::core::ffi::c_int,
    pub fc_level: ::core::ffi::c_int,
    pub fc_defer: GArray,
    pub fc_prof_child: ProfTime,
    pub fc_caller: *mut FuncCall,
    pub fc_refcount: Refcount,
    pub fc_copy_id: ::core::ffi::c_int,
    pub fc_ufuncs: GArray,
}
/// A funccall's twelve fixed variables, `b:changedtick` and a `v:` row are
/// each an ordinary [`DictItem`](crate::types::DictItem) now: the four
/// look-alike structs existed only to spell the flexible key member out at
/// their own length, and an item owns its key.  The one still wrapped is the
/// scope dictionary's own entry, which must not drop the reference it names
/// ([`ScopeDictItem`](crate::types::ScopeDictItem)).
///
/// A funccall arrives `xcalloc`'d, so what a fixed variable's storage holds
/// before it is used has to be a *valid* item: an all-zero `DictItem` is
/// `VAR_UNKNOWN`, unlocked, unflagged, with the empty inline key.
#[cfg(not(randomized_layout))]
const _: () = {
    use crate::types::{DictItem, DictKey};
    assert!(
        DictKey::INLINE_MAX >= 20,
        "a funccall's short names fit inline"
    );
    assert!(::core::mem::size_of::<DictItem>() <= 48);
};
pub struct HtStack {
    pub ht: *mut DictTab,
    pub prev: *mut HtStack,
}
pub struct ListStack {
    pub list: *mut List,
    pub prev: *mut ListStack,
}
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

/// A Vimscript List: an array of values, reference counted, and a chain of
/// cursors (`lv_watch`) held by whatever `:for` loops are walking it.
///
/// The items are owned outright -- dropping the list drops them -- which is
/// why an item's identity outside the array is an *index* and not an
/// address.  [`ListWatch`] is the one identity that has to survive an edit,
/// and `tv_list_watch_*` is what keeps it pointing at the same item.
pub struct List {
    pub lv_items: Vec<ListItem>,
    pub lv_watch: *mut ListWatch,
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
            lv_watch: ::core::ptr::null_mut(),
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
/// Not `Copy`: a node of the intrusive watcher chain the loop links into its
/// list, so a duplicate would be a second node claiming the same place in
/// it.
#[derive(Clone)]
pub struct ListWatch {
    pub lw_index: ::core::ffi::c_int,
    pub lw_next: *mut ListWatch,
}

impl ListWatch {
    /// The cursor is past the last item.  Upstream spelled this a NULL
    /// `lw_item`, and it is *sticky*: a loop whose body appends to the list
    /// it is walking still ends, because the cursor ran off the end before
    /// the item existed.
    pub const ENDED: ::core::ffi::c_int = -1;

    /// A cursor on `l[0]`, which is where a `:for` loop starts.
    pub const fn at_start() -> ListWatch {
        ListWatch {
            lw_index: 0,
            lw_next: ::core::ptr::null_mut(),
        }
    }
}
/// A partial: a function plus bound arguments and an optional `self` dict.
///
/// Not `Copy`: `pt_name`, `pt_argv` and the two refcounts are owned, and
/// `partial_unref` is what releases them.
#[derive(Clone)]
pub struct Partial {
    pub pt_refcount: Refcount,
    pub pt_copy_id: ::core::ffi::c_int,
    pub pt_name: *mut ::core::ffi::c_char,
    pub pt_func: *mut UserFunc,
    pub pt_auto: bool,
    pub pt_argc: ::core::ffi::c_int,
    pub pt_argv: *mut TypVal,
    pub pt_dict: *mut Dict,
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
/// The pointer payloads are still raw, and the value still owns what they
/// point at: `Drop` is `tv_clear` and `Clone` is `tv_copy`.
///
/// **Every payload is released by [`TypVal`]'s own `Drop`, never by field
/// glue.** The container handles are held in a [`ManuallyDrop`] to say so:
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
    /// A string.  Owned, and null for `v:_null_string`.
    String(*mut ::core::ffi::c_char) = VAR_STRING,
    /// A funcref: an owned function name, plus a reference to the function.
    Func(*mut ::core::ffi::c_char) = VAR_FUNC,
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
#[repr(C)]
pub struct UserFunc {
    pub uf_varargs: ::core::ffi::c_int,
    pub uf_flags: crate::eval::userfunc::FuncFlags,
    pub uf_calls: ::core::ffi::c_int,
    pub uf_cleared: bool,
    pub uf_args: GArray,
    pub uf_def_args: GArray,
    pub uf_lines: GArray,
    pub uf_profiling: ::core::ffi::c_int,
    pub uf_prof_initialized: ::core::ffi::c_int,
    pub uf_luaref: LuaRef,
    pub uf_tm_count: ::core::ffi::c_int,
    pub uf_tm_total: ProfTime,
    pub uf_tm_self: ProfTime,
    pub uf_tm_children: ProfTime,
    pub uf_tml_count: *mut ::core::ffi::c_int,
    pub uf_tml_total: *mut ProfTime,
    pub uf_tml_self: *mut ProfTime,
    pub uf_tml_start: ProfTime,
    pub uf_tml_children: ProfTime,
    pub uf_tml_wait: ProfTime,
    pub uf_tml_idx: ::core::ffi::c_int,
    pub uf_tml_execed: ::core::ffi::c_int,
    pub uf_script_ctx: ScriptCtx,
    pub uf_refcount: Refcount,
    pub uf_scoped: *mut FuncCall,
    pub uf_name_exp: *mut ::core::ffi::c_char,
    pub uf_namelen: size_t,
    pub uf_name: [::core::ffi::c_char; 0],
}
pub type UVarNumber = uint64_t;
pub type VarNumber = int64_t;
