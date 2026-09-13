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
    BlobRef, DictRef, ListRef, PartialRef, list_find, list_len, tv_blob_alloc, tv_clear, tv_copy,
    tv_dict_add, tv_dict_alloc, tv_dict_item_alloc, tv_list_alloc,
};
use neovim::garray::ga_append;
use neovim::memory::{xcalloc, xmalloc, xmemdupz};
use neovim::types::{
    Blob, Callback, Dict, DictItem, DictWatcher, List, ListItem, Object, Partial, Refcount, TypVal,
    VarNumber, kBoolVarFalse, kBoolVarTrue, kSpecialVarNull,
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
            Tv::NullList => list_tv(None),
            Tv::NullDict => dict_tv(None),
            Tv::NullBlob => blob_tv(None),
            Tv::Blob(bytes) => {
                let held = tv_blob_alloc();
                let b = held.as_ptr();
                for byte in bytes {
                    unsafe { ga_append(&raw mut (*b).bv_ga, *byte) };
                }
                blob_tv(Some(held))
            }
            Tv::List(items) => {
                let list = tv_list_alloc(items.len() as isize);
                let l = list.as_ptr();
                path.push(Container::List(l));
                for item in items {
                    let item_tv = unsafe { item.build_at(path) };
                    unsafe { (*l).push(item_tv) };
                }
                path.pop();
                list_tv(Some(list))
            }
            Tv::Dict(entries) => {
                let dict = tv_dict_alloc();
                let d = dict.as_ptr();
                path.push(Container::Dict(d));
                for (key, value) in entries {
                    let di = unsafe { tv_dict_item_alloc(cstr(key.clone()).as_ptr()) };
                    let mut value_tv = unsafe { value.build_at(path) };
                    unsafe { tv_copy(&value_tv, &mut (*di).di_tv) };
                    tv_clear(&mut value_tv);
                    let _ = unsafe { tv_dict_add(d, di) };
                }
                path.pop();
                dict_tv(Some(dict))
            }
            Tv::Func(name) => {
                TypVal::Func(unsafe { xmemdupz(name.as_ptr().cast(), name.len()) }.cast())
            }
            // SAFETY: the partial just built, at a count of one.
            Tv::Partial(pt) => partial_tv(unsafe { PartialRef::owning(pt.build_at(path)) }),
            Tv::Cycle(up) => {
                // The container is already live and gains a reference.
                match path[*up] {
                    // SAFETY: the container is already live, and this is
                    // a second reference to it.
                    Container::List(l) => list_tv(unsafe { ListRef::retained(l) }),
                    Container::Dict(d) => dict_tv(unsafe { DictRef::retained(d) }),
                }
            }
            Tv::Copied(from) => {
                let mut to = TypVal::Unknown;
                unsafe { tv_copy(&**from, &mut to) };
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
                TypVal::Dict(d) => d.as_ref().map_or(ptr::null_mut(), DictRef::as_ptr),
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
    /// The number this value holds.
    fn number(&self) -> VarNumber;
}

impl Payload for TypVal {
    fn list(&self) -> *mut List {
        match self {
            TypVal::List(l) => l.as_ref().map_or(ptr::null_mut(), ListRef::as_ptr),
            other => panic!("not a list: v_type {}", other.v_type()),
        }
    }

    fn dict(&self) -> *mut Dict {
        match self {
            TypVal::Dict(d) => d.as_ref().map_or(ptr::null_mut(), DictRef::as_ptr),
            other => panic!("not a dictionary: v_type {}", other.v_type()),
        }
    }

    fn blob(&self) -> *mut Blob {
        match self {
            TypVal::Blob(b) => b.as_ref().map_or(ptr::null_mut(), BlobRef::as_ptr),
            other => panic!("not a blob: v_type {}", other.v_type()),
        }
    }

    fn partial(&self) -> *mut Partial {
        match self {
            TypVal::Partial(pt) => pt.as_ref().map_or(ptr::null_mut(), PartialRef::as_ptr),
            other => panic!("not a partial: v_type {}", other.v_type()),
        }
    }

    fn string(&self) -> *mut c_char {
        match self {
            TypVal::String(s) | TypVal::Func(s) => *s,
            other => panic!("not a string: v_type {}", other.v_type()),
        }
    }

    fn number(&self) -> VarNumber {
        match self {
            TypVal::Number(n) => *n,
            other => panic!("not a number: v_type {}", other.v_type()),
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
        TypVal::Blob(b) if b.is_none() => Tv::NullBlob,
        TypVal::Blob(b) => Tv::Blob(unsafe { blob_bytes(b.as_ref().expect("a blob").as_ptr()) }),
        TypVal::List(l) => {
            let at: *const List = l.as_ref().map_or(ptr::null(), |l| l.as_ptr().cast_const());
            unsafe { read_list_at(at, path) }
        }
        TypVal::Dict(d) => {
            let at: *const Dict = d.as_ref().map_or(ptr::null(), |d| d.as_ptr().cast_const());
            unsafe { read_dict_at(at, path) }
        }
        TypVal::Partial(pt) => {
            let at: *const Partial = pt.as_ref().map_or(ptr::null(), |p| p.as_ptr().cast_const());
            Tv::Partial(Box::new(unsafe { read_partial(at, path) }))
        }
    }
}

/// A blob's bytes.
///
/// # Safety
/// `b` points at a live blob.
unsafe fn blob_bytes(b: *const Blob) -> Vec<u8> {
    // SAFETY: the caller's promise: a live blob.
    unsafe { &*b }.bytes().to_vec()
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
    for at in 0..list_len(unsafe { l.as_ref() }) {
        let li = list_find(unsafe { l.cast_mut().as_mut() }, at);
        items.push(unsafe { read_at(&raw const (*li).li_tv, path) });
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
/// The items are the list's own array, so these addresses are a *borrow*
/// and any edit to the list invalidates them -- unlike the spec's, which
/// were one allocation each.  A case that outlives an edit compares values
/// ([`read_list`]) rather than addresses.
///
/// # Safety
/// `l` is NULL or points at a live list.
pub(crate) unsafe fn list_items(l: *const List) -> Vec<*mut ListItem> {
    (0..list_len(unsafe { l.as_ref() }))
        .map(|at| list_find(unsafe { l.cast_mut().as_mut() }, at))
        .collect()
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
        let di = hi.hi_key.item();
        out.push((unsafe { (*di).key_bytes() }.to_vec(), di));
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
/// A list value over `handle`, which the value takes over.
///
/// The crate's own `TypVal::list`, which is `pub(crate)`. The variant holds
/// its handle in a `ManuallyDrop` so that nothing but `TypVal`'s own `Drop`
/// ever releases a payload (see the type's documentation); every place the
/// harness builds one goes through here rather than spelling the wrapper.
pub(crate) fn list_tv(handle: Option<ListRef>) -> TypVal {
    TypVal::List(ManuallyDrop::new(handle))
}

/// The dictionary half of [`list_tv`].
pub(crate) fn dict_tv(handle: Option<DictRef>) -> TypVal {
    TypVal::Dict(ManuallyDrop::new(handle))
}

/// The blob half of [`list_tv`].
pub(crate) fn blob_tv(handle: Option<BlobRef>) -> TypVal {
    TypVal::Blob(ManuallyDrop::new(handle))
}

/// The partial half of [`list_tv`].
pub(crate) fn partial_tv(handle: Option<PartialRef>) -> TypVal {
    TypVal::Partial(ManuallyDrop::new(handle))
}

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
    match unsafe { &*o } {
        Object::Nil => Obj::Nil,
        Object::Boolean(on) => Obj::Bool(*on),
        Object::Integer(n) => Obj::Int(*n),
        Object::Float(f) => Obj::Float(*f),
        Object::String(s) => Obj::Str(s.as_bytes().to_vec()),
        Object::Array(a) => Obj::Array(a.iter().map(|item| unsafe { read_object(item) }).collect()),
        Object::Dict(d) => {
            let mut entries: Vec<(Vec<u8>, Obj)> = d
                .iter()
                .map(|kv| {
                    (kv.key.bytes().to_vec(), unsafe {
                        read_object(&raw const kv.value)
                    })
                })
                .collect();
            entries.sort_by(|(a, _), (b, _)| a.cmp(b));
            Obj::Dict(entries)
        }
        other => panic!("reading Object kind {} is not implemented", other.kind()),
    }
}
