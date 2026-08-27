#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

pub type BoolVarValue = ::core::ffi::c_uint;
/// The two `VAR_BOOL` values: `v:false` and `v:true`.
pub const kBoolVarFalse: BoolVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
/// A Vimscript or Lua callable, held by whatever registered it.
///
/// Not `Copy`. Whichever arm is live -- a funcref name, a `partial_T`
/// refcount, a `LuaRef` -- is owned, and `callback_free` releases it.
/// Duplicating one without `callback_copy` is a second owner of the same
/// reference, so the copies that remain say `.clone()` and are visible.
#[derive(Clone)]
pub struct Callback {
    pub data: Callback_data,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub union Callback_data {
    pub funcref: *mut ::core::ffi::c_char,
    pub partial: *mut partial_T,
    pub luaref: LuaRef,
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
    /// arguments, and the static lists `tv_list_init_static` hands out.
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
/// `typval_T::v_type` — which arm of `typval_T::vval` is live.
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

pub type blob_T = blobvar_S;
#[derive(Copy, Clone)]
pub struct blobvar_S {
    pub bv_ga: garray_T,
    pub bv_refcount: Refcount,
    pub bv_lock: VarLock,
}
pub type dict_T = dictvar_S;
/// A `dict_T`.
///
/// Not `Copy`: it owns its hashtab (itself self-referential), its watcher
/// queue and a Lua table reference.
#[derive(Clone)]
pub struct dictvar_S {
    pub dv_lock: VarLock,
    pub dv_scope: ScopeType,
    pub dv_refcount: Refcount,
    pub dv_copyID: ::core::ffi::c_int,
    pub dv_hashtab: hashtab_T,
    pub dv_copydict: *mut dict_T,
    pub dv_used_next: *mut dict_T,
    pub dv_used_prev: *mut dict_T,
    pub watchers: QUEUE,
    pub lua_table_ref: LuaRef,
}
#[derive(Clone)]
pub struct funccall_S {
    pub fc_func: *mut ufunc_T,
    pub fc_linenr: ::core::ffi::c_int,
    pub fc_returned: ::core::ffi::c_int,
    pub fc_fixvar: [funccall_S_fc_fixvar; 12],
    pub fc_l_vars: dict_T,
    pub fc_l_vars_var: ScopeDictDictItem,
    pub fc_l_avars: dict_T,
    pub fc_l_avars_var: ScopeDictDictItem,
    pub fc_l_varlist: list_T,
    pub fc_l_listitems: [listitem_T; 20],
    pub fc_rettv: *mut typval_T,
    pub fc_breakpoint: linenr_T,
    pub fc_dbg_tick: ::core::ffi::c_int,
    pub fc_level: ::core::ffi::c_int,
    pub fc_defer: garray_T,
    pub fc_prof_child: proftime_T,
    pub fc_caller: *mut funccall_T,
    pub fc_refcount: Refcount,
    pub fc_copyID: ::core::ffi::c_int,
    pub fc_ufuncs: garray_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct funccall_S_fc_fixvar {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 21],
}
pub type funccall_T = funccall_S;
#[derive(Copy, Clone)]
pub struct ht_stack_S {
    pub ht: *mut hashtab_T,
    pub prev: *mut ht_stack_S,
}
pub type ht_stack_T = ht_stack_S;
pub type list_T = listvar_S;
#[derive(Copy, Clone)]
pub struct list_stack_S {
    pub list: *mut list_T,
    pub prev: *mut list_stack_S,
}
pub type list_stack_T = list_stack_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listitem_S {
    pub li_next: *mut listitem_T,
    pub li_prev: *mut listitem_T,
    pub li_tv: typval_T,
}
pub type listitem_T = listitem_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listvar_S {
    pub lv_first: *mut listitem_T,
    pub lv_last: *mut listitem_T,
    pub lv_watch: *mut listwatch_T,
    pub lv_idx_item: *mut listitem_T,
    pub lv_copylist: *mut list_T,
    pub lv_used_next: *mut list_T,
    pub lv_used_prev: *mut list_T,
    pub lv_refcount: Refcount,
    pub lv_len: ::core::ffi::c_int,
    pub lv_idx: ::core::ffi::c_int,
    pub lv_copyID: ::core::ffi::c_int,
    pub lv_lock: VarLock,
    pub lua_table_ref: LuaRef,
}
/// Not `Copy`: a node of the intrusive watcher list a `:for` loop links
/// into its list, so a duplicate would be a second node claiming the same
/// place in it.
#[derive(Clone)]
pub struct listwatch_S {
    pub lw_item: *mut listitem_T,
    pub lw_next: *mut listwatch_T,
}
pub type listwatch_T = listwatch_S;
/// A partial: a function plus bound arguments and an optional `self` dict.
///
/// Not `Copy`: `pt_name`, `pt_argv` and the two refcounts are owned, and
/// `partial_unref` is what releases them.
#[derive(Clone)]
pub struct partial_S {
    pub pt_refcount: Refcount,
    pub pt_copyID: ::core::ffi::c_int,
    pub pt_name: *mut ::core::ffi::c_char,
    pub pt_func: *mut ufunc_T,
    pub pt_auto: bool,
    pub pt_argc: ::core::ffi::c_int,
    pub pt_argv: *mut typval_T,
    pub pt_dict: *mut dict_T,
}
pub type partial_T = partial_S;
pub type scid_T = ::core::ffi::c_int;
#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct sctx_T {
    pub sc_sid: scid_T,
    pub sc_seq: ::core::ffi::c_int,
    pub sc_lnum: linenr_T,
    pub sc_chan: uint64_t,
}

impl sctx_T {
    /// No script is running: the all-zero context every table of script
    /// contexts starts out holding. A `const` because most of its uses are
    /// `static` initialisers, where `Default` cannot reach.
    pub const NONE: sctx_T = sctx_T {
        sc_sid: 0,
        sc_seq: 0,
        sc_lnum: 0,
        sc_chan: 0,
    };

    /// The same context under a different script id.
    ///
    /// A `Copy` cell's field write is a read-modify-write, and spelling it
    /// as one expression keeps the "which field" out of the caller's
    /// bookkeeping -- `pos_T::with_col`'s shape.
    pub fn with_sid(self, sc_sid: scid_T) -> Self {
        sctx_T { sc_sid, ..self }
    }

    /// The same context at a different line inside the script.
    pub fn with_lnum(self, sc_lnum: linenr_T) -> Self {
        sctx_T { sc_lnum, ..self }
    }

    /// The same context under a different sourcing sequence number.
    pub fn with_seq(self, sc_seq: ::core::ffi::c_int) -> Self {
        sctx_T { sc_seq, ..self }
    }
}

impl Default for sctx_T {
    fn default() -> Self {
        Self::NONE
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct staticList10_T {
    pub sl_list: list_T,
    pub sl_items: [listitem_T; 10],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct typval_T {
    pub v_type: VarType,
    pub v_lock: VarLock,
    pub vval: typval_vval_union,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union typval_vval_union {
    pub v_number: varnumber_T,
    pub v_bool: BoolVarValue,
    pub v_special: SpecialVarValue,
    pub v_float: float_T,
    pub v_string: *mut ::core::ffi::c_char,
    pub v_list: *mut list_T,
    pub v_dict: *mut dict_T,
    pub v_partial: *mut partial_T,
    pub v_blob: *mut blob_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ufunc_S {
    pub uf_varargs: ::core::ffi::c_int,
    pub uf_flags: ::core::ffi::c_int,
    pub uf_calls: ::core::ffi::c_int,
    pub uf_cleared: bool,
    pub uf_args: garray_T,
    pub uf_def_args: garray_T,
    pub uf_lines: garray_T,
    pub uf_profiling: ::core::ffi::c_int,
    pub uf_prof_initialized: ::core::ffi::c_int,
    pub uf_luaref: LuaRef,
    pub uf_tm_count: ::core::ffi::c_int,
    pub uf_tm_total: proftime_T,
    pub uf_tm_self: proftime_T,
    pub uf_tm_children: proftime_T,
    pub uf_tml_count: *mut ::core::ffi::c_int,
    pub uf_tml_total: *mut proftime_T,
    pub uf_tml_self: *mut proftime_T,
    pub uf_tml_start: proftime_T,
    pub uf_tml_children: proftime_T,
    pub uf_tml_wait: proftime_T,
    pub uf_tml_idx: ::core::ffi::c_int,
    pub uf_tml_execed: ::core::ffi::c_int,
    pub uf_script_ctx: sctx_T,
    pub uf_refcount: Refcount,
    pub uf_scoped: *mut funccall_T,
    pub uf_name_exp: *mut ::core::ffi::c_char,
    pub uf_namelen: size_t,
    pub uf_name: [::core::ffi::c_char; 0],
}
pub type ufunc_T = ufunc_S;
pub type uvarnumber_T = uint64_t;
pub type varnumber_T = int64_t;
