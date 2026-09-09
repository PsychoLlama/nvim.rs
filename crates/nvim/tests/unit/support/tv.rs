//! The Rust twin of `test/unit/eval/testutil.lua`: a value model that
//! `TypVal` is built from and read back into.
//!
//! The Lua harness spelled a value as a Lua table and converted with
//! `lua2typvalt`/`typvalt2lua`, which is what let a case say
//! `eq({ 'tes', null_string }, typvalt2lua(l_tv))` — one assertion over a
//! whole structure instead of a walk. [`Tv`] is that table, with the two
//! places Lua was ambiguous made explicit:
//!
//! - a Lua number was a `VAR_FLOAT` and `int(n)` was a `VAR_NUMBER`; here
//!   they are [`Tv::Float`] and [`Tv::Int`];
//! - a Lua `nil` inside a table ended the array, so the spec spelled the
//!   three NULL containers with sentinel tables; here they are
//!   [`Tv::NullStr`], [`Tv::NullList`] and [`Tv::NullDict`].
//!
//! Cycles are the one thing a Rust value cannot hold directly. `lst2tbl`
//! made a self-referencing list a self-referencing *Lua table*, which
//! `eq` compared without looping; [`Tv::Cycle`] names the container `n`
//! levels up the path instead, so `[[...]]` reads back as
//! `List([Cycle(0)])`. The Lua harness deduplicated *every* repeated
//! container, not only ancestors; this deduplicates only ancestors, which
//! is the same answer for every structure in the specs and a shorter one
//! to write down.

use std::ffi::{CStr, c_char, c_int, c_void};
use std::mem::{ManuallyDrop, offset_of};
use std::ops::Deref;
use std::ptr;

use neovim::eval::typval::{
    tv_blob_alloc, tv_blob_get, tv_blob_len, tv_clear, tv_copy, tv_dict_add, tv_dict_alloc,
    tv_dict_item_alloc, tv_list_alloc, tv_list_append,
};
use neovim::garray::ga_append;
use neovim::memory::{xcalloc, xmalloc, xmemdupz};
use neovim::types::{
    Blob, Callback, Dict, DictItem, DictWatcher, List, ListItem, Object, Partial, Refcount, TypVal,
    VarLock, kBoolVarFalse, kBoolVarTrue, kSpecialVarNull,
};

use super::cstr;

/// A Vimscript value, as the spec's Lua tables spelled one.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Tv {
    /// `VAR_UNKNOWN` — the type a fresh `TypVal` starts in.
    Unknown,
    /// `VAR_SPECIAL`, `v:null`; the spec's `nil_value`.
    Nil,
    /// `VAR_BOOL`.
    Bool(bool),
    /// `VAR_NUMBER`; the spec's `int(n)`.
    Int(i64),
    /// `VAR_FLOAT`; a bare Lua number in the spec.
    Float(f64),
    /// `VAR_STRING` with a value.
    Str(Vec<u8>),
    /// `VAR_STRING` whose `v_string` is NULL; the spec's `null_string`.
    NullStr,
    /// `VAR_LIST` with a value. The empty list is `List(vec![])`, the
    /// spec's `empty_list`.
    List(Vec<Tv>),
    /// `VAR_LIST` whose `v_list` is NULL; the spec's `null_list`.
    NullList,
    /// `VAR_DICT`, read back sorted by key so a case does not depend on
    /// the hashtab's order. Built in the order given.
    Dict(Vec<(Vec<u8>, Tv)>),
    /// `VAR_DICT` whose `v_dict` is NULL; the spec's `null_dict`.
    NullDict,
    /// `VAR_BLOB` with a value. The empty blob is `Blob(vec![])`.
    Blob(Vec<u8>),
    /// `VAR_BLOB` whose `v_blob` is NULL; `v:_null_blob`.
    NullBlob,
    /// `VAR_FUNC`: a function name and nothing else.
    Func(Vec<u8>),
    /// `VAR_PARTIAL`: a name with bound arguments and/or a dict.
    Partial(Box<Pt>),
    /// The container this many levels up the path from the root — how a
    /// cycle is spelled. `Cycle(0)` is the outermost container.
    Cycle(usize),
    /// Build by `tv_copy`ing an existing value in, the Lua harness's
    /// `type(l) == 'cdata'` arm. Never produced by a read.
    Copied(*const TypVal),
}

/// A `Partial`, as `partial2lua` spelled one.
#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct Pt {
    /// `pt_name`.
    pub value: Vec<u8>,
    /// `pt_auto`.
    pub auto: bool,
    /// The bound arguments, `pt_argv[0..pt_argc]`.
    pub args: Vec<Tv>,
    /// `pt_dict`, when it has one.
    pub dict: Option<Tv>,
}

impl Tv {
    /// `Tv::Str` from anything string-shaped.
    pub(crate) fn s(bytes: impl AsRef<[u8]>) -> Tv {
        Tv::Str(bytes.as_ref().to_vec())
    }

    /// `Tv::Dict` from `&str` keys.
    pub(crate) fn dict<const N: usize>(entries: [(&str, Tv); N]) -> Tv {
        Tv::Dict(
            entries
                .into_iter()
                .map(|(k, v)| (k.as_bytes().to_vec(), v))
                .collect(),
        )
    }

    /// The `TypVal` this value describes, owned by the caller.
    ///
    /// The Lua harness attached a `tv_clear` finaliser here; a Rust case
    /// clears explicitly, or hands the value to something that takes it.
    ///
    /// # Safety
    /// The editor must be up (the caller holds the editor lock): building a
    /// list or a dict calls into the allocator and the hashtab.
    pub(crate) unsafe fn build(&self) -> TypVal {
        let mut path = Vec::new();
        unsafe { self.build_at(&mut path) }
    }

    /// # Safety
    /// As [`Tv::build`]. `path` holds the containers currently being built,
    /// outermost first, for [`Tv::Cycle`] to name.
    unsafe fn build_at(&self, path: &mut Vec<Container>) -> TypVal {
        match self {
            Tv::Unknown => TypVal::Unknown,
            Tv::Nil => TypVal::Special(kSpecialVarNull),
            Tv::Bool(b) => TypVal::Bool(if *b { kBoolVarTrue } else { kBoolVarFalse }),
            Tv::Int(n) => TypVal::Number(*n),
            Tv::Float(f) => TypVal::Float(*f),
            Tv::Str(s) => TypVal::String(unsafe { xmemdupz(s.as_ptr().cast(), s.len()) }.cast()),
            Tv::NullStr => TypVal::String(ptr::null_mut()),
            Tv::NullList => TypVal::List(ptr::null_mut()),
            Tv::NullDict => TypVal::Dict(ptr::null_mut()),
            Tv::NullBlob => TypVal::Blob(ptr::null_mut()),
            Tv::Blob(bytes) => {
                let b = tv_blob_alloc();
                unsafe { (*b).bv_refcount = Refcount::ONE };
                for byte in bytes {
                    unsafe { ga_append(&raw mut (*b).bv_ga, *byte) };
                }
                TypVal::Blob(b)
            }
            Tv::List(items) => {
                let l = tv_list_alloc(items.len() as isize);
                unsafe { (*l).lv_refcount = Refcount::ONE };
                path.push(Container::List(l));
                for item in items {
                    let item_tv = unsafe { item.build_at(path) };
                    let li = unsafe { list_item_alloc() };
                    // The item's value has never held one, so the value goes
                    // in rather than over: an assignment would drop what the
                    // allocator left behind.
                    unsafe { (&raw mut (*li).li_tv).write(item_tv) };
                    unsafe { tv_list_append(l, li) };
                }
                path.pop();
                TypVal::List(l)
            }
            Tv::Dict(entries) => {
                let d = unsafe { tv_dict_alloc() };
                unsafe { (*d).dv_refcount = Refcount::ONE };
                path.push(Container::Dict(d));
                for (key, value) in entries {
                    let di = unsafe { tv_dict_item_alloc(cstr(key.clone()).as_ptr()) };
                    let mut value_tv = unsafe { value.build_at(path) };
                    unsafe { tv_copy(&raw const value_tv, &raw mut (*di).di_tv) };
                    unsafe { tv_clear(&raw mut value_tv) };
                    let _ = unsafe { tv_dict_add(d, di) };
                }
                path.pop();
                TypVal::Dict(d)
            }
            Tv::Func(name) => {
                TypVal::Func(unsafe { xmemdupz(name.as_ptr().cast(), name.len()) }.cast())
            }
            Tv::Partial(pt) => TypVal::Partial(unsafe { pt.build_at(path) }),
            Tv::Cycle(up) => {
                // The container is already live and gains a reference.
                match path[*up] {
                    Container::List(l) => {
                        unsafe { (*l).lv_refcount.retain() };
                        TypVal::List(l)
                    }
                    Container::Dict(d) => {
                        unsafe { (*d).dv_refcount.retain() };
                        TypVal::Dict(d)
                    }
                }
            }
            Tv::Copied(from) => {
                let mut to = TypVal::Unknown;
                unsafe { tv_copy(*from, &raw mut to) };
                to
            }
        }
    }
}

impl Pt {
    /// # Safety
    /// As [`Tv::build`].
    unsafe fn build_at(&self, path: &mut Vec<Container>) -> *mut Partial {
        let pt: *mut Partial = unsafe { xcalloc(1, size_of::<Partial>()) }.cast();
        let argv: *mut TypVal = if self.args.is_empty() {
            ptr::null_mut()
        } else {
            unsafe { xmalloc(size_of::<TypVal>() * self.args.len()) }.cast()
        };
        for (i, arg) in self.args.iter().enumerate() {
            // Raw storage: the value is written in, not assigned over.
            unsafe { argv.add(i).write(arg.build_at(path)) };
        }
        let dict = match &self.dict {
            None => ptr::null_mut(),
            // The partial takes the dictionary over, so the value that
            // built it must not release it on the way out of the match.
            Some(dict) => match ManuallyDrop::new(unsafe { dict.build_at(path) }).deref() {
                TypVal::Dict(d) => *d,
                other => panic!("a partial's dict is a dict, not {}", other.v_type()),
            },
        };
        unsafe {
            (*pt).pt_refcount = Refcount::ONE;
            (*pt).pt_name = xmemdupz(self.value.as_ptr().cast(), self.value.len()).cast();
            (*pt).pt_auto = self.auto;
            (*pt).pt_argc = c_int::try_from(self.args.len()).expect("a small argument count");
            (*pt).pt_argv = argv;
            (*pt).pt_dict = dict;
        }
        pt
    }
}

/// The payload readers the crate keeps to itself.
///
/// The enum's variants are the seam a spec gets; the crate's `as_*`/
/// `*_or_null` family is `pub(crate)`. These are what the specs' old
/// `tv.list()` became, and they **panic** on the wrong kind rather than
/// answering NULL: a spec that reads the wrong arm has a bug, where the
/// crate's readers answer the empty value on purpose.
pub(crate) trait Payload {
    /// The list this value holds.
    fn list(&self) -> *mut List;
    /// The dictionary this value holds.
    fn dict(&self) -> *mut Dict;
    /// The blob this value holds.
    fn blob(&self) -> *mut Blob;
    /// The partial this value holds.
    fn partial(&self) -> *mut Partial;
    /// The string this value holds, under either kind that holds one.
    fn string(&self) -> *mut c_char;
}

impl Payload for TypVal {
    fn list(&self) -> *mut List {
        match self {
            TypVal::List(l) => *l,
            other => panic!("not a list: v_type {}", other.v_type()),
        }
    }

    fn dict(&self) -> *mut Dict {
        match self {
            TypVal::Dict(d) => *d,
            other => panic!("not a dictionary: v_type {}", other.v_type()),
        }
    }

    fn blob(&self) -> *mut Blob {
        match self {
            TypVal::Blob(b) => *b,
            other => panic!("not a blob: v_type {}", other.v_type()),
        }
    }

    fn partial(&self) -> *mut Partial {
        match self {
            TypVal::Partial(pt) => *pt,
            other => panic!("not a partial: v_type {}", other.v_type()),
        }
    }

    fn string(&self) -> *mut c_char {
        match self {
            TypVal::String(s) | TypVal::Func(s) => *s,
            other => panic!("not a string: v_type {}", other.v_type()),
        }
    }
}

/// A container on the path from the root, for [`Tv::Cycle`].
#[derive(Clone, Copy)]
enum Container {
    List(*mut List),
    Dict(*mut Dict),
}

impl Container {
    fn addr(self) -> *const c_void {
        match self {
            Container::List(l) => l.cast(),
            Container::Dict(d) => d.cast(),
        }
    }
}

/// A bit copy of a typval: the payload is *shared*, so the copy and the
/// original name one object and only one of them may release it.
///
/// The crate's own `TypVal::bit_copy` is `pub(crate)`; this is the harness's
/// copy of it, for the specs that deliberately put one value in two places
/// and hand out the reference counts themselves.
///
/// # Safety
/// The caller owns the refcount reasoning; see above.
pub(crate) unsafe fn bit_copy(tv: &TypVal) -> TypVal {
    // SAFETY: the caller's promise above -- the duplicate is not released.
    unsafe { ptr::read(tv) }
}

/// `typvalt2lua`: read a value back out.
///
/// # Safety
/// `tv` points at a live `TypVal` whose contents are live.
pub(crate) unsafe fn read(tv: *const TypVal) -> Tv {
    let mut path = Vec::new();
    unsafe { read_at(tv, &mut path) }
}

/// `lst2tbl`: read a list back out, NULL included.
///
/// # Safety
/// `l` is NULL or points at a live list.
pub(crate) unsafe fn read_list(l: *const List) -> Tv {
    let mut path = Vec::new();
    unsafe { read_list_at(l, &mut path) }
}

/// `dct2tbl`: read a dict back out, NULL included.
///
/// # Safety
/// `d` is NULL or points at a live dict.
pub(crate) unsafe fn read_dict(d: *const Dict) -> Tv {
    let mut path = Vec::new();
    unsafe { read_dict_at(d, &mut path) }
}

/// # Safety
/// As [`read`].
unsafe fn read_at(tv: *const TypVal, path: &mut Vec<Container>) -> Tv {
    match unsafe { &*tv } {
        TypVal::Unknown => Tv::Unknown,
        TypVal::Special(special) => {
            assert_eq!(*special, kSpecialVarNull);
            Tv::Nil
        }
        TypVal::Bool(b) => Tv::Bool(match *b {
            b if b == kBoolVarTrue => true,
            b if b == kBoolVarFalse => false,
            other => panic!("not a boolean: {other}"),
        }),
        TypVal::Number(n) => Tv::Int(*n),
        TypVal::Float(f) => Tv::Float(*f),
        TypVal::String(s) if s.is_null() => Tv::NullStr,
        TypVal::String(s) => Tv::Str(unsafe { CStr::from_ptr(*s) }.to_bytes().to_vec()),
        TypVal::Func(s) if s.is_null() => Tv::NullStr,
        TypVal::Func(s) => Tv::Func(unsafe { CStr::from_ptr(*s) }.to_bytes().to_vec()),
        TypVal::Blob(b) if b.is_null() => Tv::NullBlob,
        TypVal::Blob(b) => Tv::Blob(unsafe { blob_bytes(*b) }),
        TypVal::List(l) => unsafe { read_list_at(*l, path) },
        TypVal::Dict(d) => unsafe { read_dict_at(*d, path) },
        TypVal::Partial(pt) => Tv::Partial(Box::new(unsafe { read_partial(*pt, path) })),
    }
}

/// A blob's bytes.
///
/// # Safety
/// `b` points at a live blob.
unsafe fn blob_bytes(b: *const Blob) -> Vec<u8> {
    (0..unsafe { tv_blob_len(b) })
        .map(|i| unsafe { tv_blob_get(b, i) })
        .collect()
}

/// # Safety
/// As [`read_list`].
unsafe fn read_list_at(l: *const List, path: &mut Vec<Container>) -> Tv {
    if l.is_null() {
        return Tv::NullList;
    }
    if let Some(up) = seen(path, l.cast()) {
        return Tv::Cycle(up);
    }
    path.push(Container::List(l.cast_mut()));
    let mut items = Vec::new();
    let mut li = unsafe { (*l).lv_first };
    while !li.is_null() {
        items.push(unsafe { read_at(&raw const (*li).li_tv, path) });
        li = unsafe { (*li).li_next };
    }
    path.pop();
    Tv::List(items)
}

/// # Safety
/// As [`read_dict`].
unsafe fn read_dict_at(d: *const Dict, path: &mut Vec<Container>) -> Tv {
    if d.is_null() {
        return Tv::NullDict;
    }
    if let Some(up) = seen(path, d.cast()) {
        return Tv::Cycle(up);
    }
    path.push(Container::Dict(d.cast_mut()));
    let mut entries: Vec<(Vec<u8>, Tv)> = unsafe { dict_items(d) }
        .into_iter()
        .map(|(key, di)| (key, unsafe { read_at(&raw const (*di).di_tv, path) }))
        .collect();
    path.pop();
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));
    Tv::Dict(entries)
}

/// `partial2lua`.
///
/// # Safety
/// `pt` is NULL or points at a live partial.
unsafe fn read_partial(pt: *const Partial, path: &mut Vec<Container>) -> Pt {
    if pt.is_null() {
        return Pt::default();
    }
    let args = (0..unsafe { (*pt).pt_argc })
        .map(|i| unsafe { read_at((*pt).pt_argv.offset(i as isize), path) })
        .collect();
    Pt {
        value: unsafe { CStr::from_ptr((*pt).pt_name) }.to_bytes().to_vec(),
        auto: unsafe { (*pt).pt_auto },
        args,
        dict: match unsafe { (*pt).pt_dict } {
            d if d.is_null() => None,
            d => Some(unsafe { read_dict_at(d, path) }),
        },
    }
}

fn seen(path: &[Container], at: *const c_void) -> Option<usize> {
    path.iter().position(|c| c.addr() == at)
}

/// `tv_list_item_alloc`, which the crate keeps private: an item whose links
/// and value the caller fills in and hands to `tv_list_append`.
///
/// The lock is written here for the same reason the crate's copy writes it:
/// it is the *slot's*, so no caller sets it and an `xmalloc`'d one would be
/// whatever the heap last held.
///
/// # Safety
/// The editor must be up.
pub(crate) unsafe fn list_item_alloc() -> *mut ListItem {
    let li: *mut ListItem = unsafe { xmalloc(size_of::<ListItem>()) }.cast();
    unsafe { (&raw mut (*li).li_lock).write(VarLock::Unlocked) };
    li
}

/// The spec's `li_alloc`: an item holding `VAR_UNKNOWN`, unlinked.
///
/// # Safety
/// As [`list_item_alloc`].
pub(crate) unsafe fn li_alloc() -> *mut ListItem {
    let li = unsafe { list_item_alloc() };
    unsafe {
        (*li).li_next = ptr::null_mut();
        (*li).li_prev = ptr::null_mut();
        // Written, not assigned: the slot has never held a value.
        (&raw mut (*li).li_tv).write(TypVal::Unknown);
    }
    li
}

/// The spec's `list(...)`: a fresh list with `lv_refcount` 1 holding these
/// values.
///
/// # Safety
/// The editor must be up.
pub(crate) unsafe fn new_list(items: &[Tv]) -> *mut List {
    // The list outlives the value that built it: the caller owns the
    // reference now, so the builder must not release it.
    let tv = ManuallyDrop::new(unsafe { Tv::List(items.to_vec()).build() });
    tv.list()
}

/// The spec's `dict{...}`: a fresh dict with `dv_refcount` 1.
///
/// # Safety
/// The editor must be up.
pub(crate) unsafe fn new_dict(entries: &[(&str, Tv)]) -> *mut Dict {
    let entries: Vec<(Vec<u8>, Tv)> = entries
        .iter()
        .map(|(k, v)| (k.as_bytes().to_vec(), v.clone()))
        .collect();
    // As `new_list`: the caller owns the reference.
    let tv = ManuallyDrop::new(unsafe { Tv::Dict(entries).build() });
    tv.dict()
}

/// The spec's `list_items`: every item of `l`, front to back.
///
/// # Safety
/// `l` is NULL or points at a live list.
pub(crate) unsafe fn list_items(l: *const List) -> Vec<*mut ListItem> {
    let mut items = Vec::new();
    if l.is_null() {
        return items;
    }
    let mut li = unsafe { (*l).lv_first };
    while !li.is_null() {
        items.push(li);
        li = unsafe { (*li).li_next };
    }
    items
}

/// The spec's `dict_items`: every live item of `d`, in hashtab order —
/// which is the order the allocation log sees them in.
///
/// # Safety
/// `d` points at a live dict.
pub(crate) unsafe fn dict_items(d: *const Dict) -> Vec<(Vec<u8>, *mut DictItem)> {
    let ht = unsafe { &(*d).dv_hashtab };
    let mut out = Vec::new();
    for hi in ht.items() {
        let key = hi.hi_key;
        let di: *mut DictItem = unsafe { key.byte_sub(offset_of!(DictItem, di_key)) }.cast();
        out.push((unsafe { CStr::from_ptr(key) }.to_bytes().to_vec(), di));
    }
    out
}

/// The item `d` holds under `key`. Panics if there is none — a case that
/// wants the absence asserts it through `tv_dict_find`.
///
/// # Safety
/// As [`dict_items`].
pub(crate) unsafe fn di_of(d: *const Dict, key: &str) -> *mut DictItem {
    unsafe { dict_items(d) }
        .into_iter()
        .find(|(k, _)| k == key.as_bytes())
        .unwrap_or_else(|| panic!("no key {key:?}"))
        .1
}

/// The spec's `first_di`: the item in the first occupied slot.
///
/// # Safety
/// As [`dict_items`].
pub(crate) unsafe fn first_di(d: *const Dict) -> *mut DictItem {
    let items = unsafe { dict_items(d) };
    items[0].1
}

/// A `Callback`, as `callback2tbl` spelled one.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Cb {
    /// `Callback::None`.
    None,
    /// `Callback::Funcref`, and the name it holds.
    Fref(Vec<u8>),
    /// `Callback::Partial`, and the partial it holds.
    Pt(Box<Pt>),
}

/// `callback2tbl`.
///
/// # Safety
/// `cb` points at a live callback.
pub(crate) unsafe fn read_callback(cb: *const Callback) -> Cb {
    let mut path = Vec::new();
    match unsafe { &*cb } {
        Callback::None => Cb::None,
        Callback::Funcref(name) => Cb::Fref(unsafe { CStr::from_ptr(*name) }.to_bytes().to_vec()),
        Callback::Partial(partial) => {
            Cb::Pt(Box::new(unsafe { read_partial(*partial, &mut path) }))
        }
        Callback::Lua(_) => panic!("a Lua callback is not implemented"),
    }
}

/// `tbl2callback`: the callback a spec case hands to
/// `tv_dict_watcher_add`. The caller owns it and releases it with
/// `callback_free`.
///
/// # Safety
/// The editor must be up.
pub(crate) unsafe fn build_callback(cb: &Cb) -> Callback {
    match cb {
        Cb::None => Callback::None,
        Cb::Fref(name) => {
            Callback::Funcref(unsafe { xmemdupz(name.as_ptr().cast(), name.len()) }.cast())
        }
        Cb::Pt(pt) => {
            let mut path = Vec::new();
            Callback::Partial(unsafe { pt.build_at(&mut path) })
        }
    }
}

/// One registered dict watcher, as `dict_watchers` spelled it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Watcher {
    /// The watcher itself, for the allocation log.
    pub at: *mut DictWatcher,
    /// `key_pattern`, for the allocation log.
    pub pattern: *mut c_char,
    /// `key_pattern[0..key_pattern_len]`.
    pub pat: Vec<u8>,
    /// `callback`.
    pub cb: Cb,
    /// `busy`.
    pub busy: bool,
}

/// The spec's `dict_watchers`, in registration order.
///
/// # Safety
/// `d` points at a live dict.
pub(crate) unsafe fn dict_watchers(d: *const Dict) -> Vec<Watcher> {
    let head = unsafe { &raw const (*d).watchers };
    let mut out = Vec::new();
    let mut q = unsafe { (*head).next };
    while q.cast_const() != head {
        let w: *mut DictWatcher = unsafe { q.byte_sub(offset_of!(DictWatcher, node)) }.cast();
        let pattern = unsafe { (*w).key_pattern };
        let len = unsafe { (*w).key_pattern_len };
        out.push(Watcher {
            at: w,
            pattern,
            pat: unsafe { std::slice::from_raw_parts(pattern.cast::<u8>(), len) }.to_vec(),
            cb: unsafe { read_callback(&raw const (*w).callback) },
            busy: unsafe { (*w).busy },
        });
        q = unsafe { (*q).next };
    }
    out
}

/// The spec's `ga_alloc`: a `GArray` on the caller's stack, initialised.
pub(crate) fn ga_alloc(itemsize: c_int, growsize: c_int) -> neovim::types::GArray {
    let mut ga = neovim::types::GArray {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ptr::null_mut(),
    };
    // SAFETY: `ga` is this frame's and `ga_init` only writes the header.
    unsafe { neovim::garray::ga_init(&raw mut ga, itemsize, growsize) };
    ga
}

/// The spec's `eval0`: evaluate an expression, answering the value or
/// `None` when evaluation failed.
///
/// # Safety
/// The editor must be up. The answer owns its contents; clear it.
pub(crate) unsafe fn eval0(expr: &str) -> Option<TypVal> {
    use neovim::eval::EVAL_EVALUATE;
    use neovim::types::EvalArg;

    let mut tv = TypVal::Unknown;
    let mut evalarg = EvalArg {
        eval_flags: EVAL_EVALUATE as c_int,
        eval_getline: None,
        eval_cookie: ptr::null_mut(),
        eval_tofree: ptr::null_mut(),
    };
    // `eval0` takes a mutable buffer: it writes the terminator back over
    // what it consumed.
    let mut arg: Vec<c_char> = expr.bytes().map(|b| b as c_char).chain([0]).collect();
    let ok = unsafe {
        neovim::eval::eval0(arg.as_mut_ptr(), &mut tv, ptr::null_mut(), &raw mut evalarg)
    };
    ok.is_ok().then_some(tv)
}

/// An API `Object`, as `test/unit/api/testutil.lua`'s `obj2lua` spelled one.
///
/// The API's value type is [`Tv`]'s twin one layer out: it has no NULL
/// container (a NULL list converts to an *empty* array) and no funcref at
/// all (a partial converts to nil), which is most of what the conversion
/// cases are about.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Obj {
    /// `kObjectTypeNil`.
    Nil,
    /// `kObjectTypeBoolean`.
    Bool(bool),
    /// `kObjectTypeInteger`.
    Int(i64),
    /// `kObjectTypeFloat`.
    Float(f64),
    /// `kObjectTypeString`, by its `size` — interior NULs included.
    Str(Vec<u8>),
    /// `kObjectTypeArray`.
    Array(Vec<Obj>),
    /// `kObjectTypeDict`, read back sorted by key.
    Dict(Vec<(Vec<u8>, Obj)>),
}

impl Obj {
    /// `Obj::Str` from anything string-shaped.
    pub(crate) fn s(bytes: impl AsRef<[u8]>) -> Obj {
        Obj::Str(bytes.as_ref().to_vec())
    }

    /// `Obj::Dict` from `&str` keys.
    pub(crate) fn dict<const N: usize>(entries: [(&str, Obj); N]) -> Obj {
        Obj::Dict(
            entries
                .into_iter()
                .map(|(k, v)| (k.as_bytes().to_vec(), v))
                .collect(),
        )
    }
}

/// `obj2lua`: read an `Object` back out.
///
/// # Safety
/// `o` points at a live `Object` whose contents are live.
pub(crate) unsafe fn read_object(o: *const Object) -> Obj {
    match unsafe { *o } {
        Object::Nil => Obj::Nil,
        Object::Boolean(on) => Obj::Bool(on),
        Object::Integer(n) => Obj::Int(n),
        Object::Float(f) => Obj::Float(f),
        Object::String(s) => Obj::Str(if s.is_null() {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(s.data().cast::<u8>(), s.len()) }.to_vec()
        }),
        Object::Array(a) => Obj::Array(
            (0..a.size)
                .map(|i| unsafe { read_object(a.items.add(i)) })
                .collect(),
        ),
        Object::Dict(d) => {
            let mut entries: Vec<(Vec<u8>, Obj)> = (0..d.size)
                .map(|i| {
                    let kv = unsafe { *d.items.add(i) };
                    (
                        unsafe {
                            std::slice::from_raw_parts(kv.key.data().cast::<u8>(), kv.key.len())
                        }
                        .to_vec(),
                        unsafe { read_object(&raw const kv.value) },
                    )
                })
                .collect();
            entries.sort_by(|(a, _), (b, _)| a.cmp(b));
            Obj::Dict(entries)
        }
        other => panic!("reading Object kind {} is not implemented", other.kind()),
    }
}
