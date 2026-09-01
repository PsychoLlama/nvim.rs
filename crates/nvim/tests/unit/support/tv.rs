//! The Rust twin of `test/unit/eval/testutil.lua`: a value model that
//! `typval_T` is built from and read back into.
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
use std::mem::offset_of;
use std::ptr;

use neovim::eval::typval::{
    tv_clear, tv_copy, tv_dict_add, tv_dict_alloc, tv_dict_item_alloc, tv_list_alloc,
    tv_list_append,
};
use neovim::memory::{xcalloc, xmalloc, xmemdupz};
use neovim::types::{
    Callback, DictWatcher, Object, Refcount, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST,
    VAR_NUMBER, VAR_PARTIAL, VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN, VarLock, dict_T, dictitem_T,
    kBoolVarFalse, kBoolVarTrue, kSpecialVarNull, list_T, listitem_T, partial_T, typval_T,
    typval_vval_union,
};

use super::cstr;

/// A Vimscript value, as the spec's Lua tables spelled one.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Tv {
    /// `VAR_UNKNOWN` — the type a fresh `typval_T` starts in.
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
    /// `VAR_FUNC`: a function name and nothing else.
    Func(Vec<u8>),
    /// `VAR_PARTIAL`: a name with bound arguments and/or a dict.
    Partial(Box<Pt>),
    /// The container this many levels up the path from the root — how a
    /// cycle is spelled. `Cycle(0)` is the outermost container.
    Cycle(usize),
    /// Build by `tv_copy`ing an existing value in, the Lua harness's
    /// `type(l) == 'cdata'` arm. Never produced by a read.
    Copied(*const typval_T),
}

/// A `partial_T`, as `partial2lua` spelled one.
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

    /// The `typval_T` this value describes, owned by the caller.
    ///
    /// The Lua harness attached a `tv_clear` finaliser here; a Rust case
    /// clears explicitly, or hands the value to something that takes it.
    ///
    /// # Safety
    /// The editor must be up (the caller holds the editor lock): building a
    /// list or a dict calls into the allocator and the hashtab.
    pub(crate) unsafe fn build(&self) -> typval_T {
        let mut path = Vec::new();
        unsafe { self.build_at(&mut path) }
    }

    /// # Safety
    /// As [`Tv::build`]. `path` holds the containers currently being built,
    /// outermost first, for [`Tv::Cycle`] to name.
    unsafe fn build_at(&self, path: &mut Vec<Container>) -> typval_T {
        let (v_type, vval) = match self {
            Tv::Unknown => (VAR_UNKNOWN, typval_vval_union { v_number: 0 }),
            Tv::Nil => (
                VAR_SPECIAL,
                typval_vval_union {
                    v_special: kSpecialVarNull,
                },
            ),
            Tv::Bool(b) => (
                VAR_BOOL,
                typval_vval_union {
                    v_bool: if *b { kBoolVarTrue } else { kBoolVarFalse },
                },
            ),
            Tv::Int(n) => (VAR_NUMBER, typval_vval_union { v_number: *n }),
            Tv::Float(f) => (VAR_FLOAT, typval_vval_union { v_float: *f }),
            Tv::Str(s) => (
                VAR_STRING,
                typval_vval_union {
                    v_string: unsafe { xmemdupz(s.as_ptr().cast(), s.len()) }.cast(),
                },
            ),
            Tv::NullStr => (
                VAR_STRING,
                typval_vval_union {
                    v_string: ptr::null_mut(),
                },
            ),
            Tv::NullList => (
                VAR_LIST,
                typval_vval_union {
                    v_list: ptr::null_mut(),
                },
            ),
            Tv::NullDict => (
                VAR_DICT,
                typval_vval_union {
                    v_dict: ptr::null_mut(),
                },
            ),
            Tv::List(items) => {
                let l = unsafe { tv_list_alloc(items.len() as isize) };
                unsafe { (*l).lv_refcount = Refcount::ONE };
                path.push(Container::List(l));
                for item in items {
                    let item_tv = unsafe { item.build_at(path) };
                    let li = unsafe { list_item_alloc() };
                    unsafe { (*li).li_tv = item_tv };
                    unsafe { tv_list_append(l, li) };
                }
                path.pop();
                (VAR_LIST, typval_vval_union { v_list: l })
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
                (VAR_DICT, typval_vval_union { v_dict: d })
            }
            Tv::Func(name) => (
                VAR_FUNC,
                typval_vval_union {
                    v_string: unsafe { xmemdupz(name.as_ptr().cast(), name.len()) }.cast(),
                },
            ),
            Tv::Partial(pt) => (
                VAR_PARTIAL,
                typval_vval_union {
                    v_partial: unsafe { pt.build_at(path) },
                },
            ),
            Tv::Cycle(up) => {
                // The container is already live and gains a reference.
                match path[*up] {
                    Container::List(l) => {
                        unsafe { (*l).lv_refcount.retain() };
                        (VAR_LIST, typval_vval_union { v_list: l })
                    }
                    Container::Dict(d) => {
                        unsafe { (*d).dv_refcount.retain() };
                        (VAR_DICT, typval_vval_union { v_dict: d })
                    }
                }
            }
            Tv::Copied(from) => {
                let mut to = typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VarLock::Unlocked,
                    vval: typval_vval_union { v_number: 0 },
                };
                unsafe { tv_copy(*from, &raw mut to) };
                return to;
            }
        };
        typval_T {
            v_type,
            v_lock: VarLock::Unlocked,
            vval,
        }
    }
}

impl Pt {
    /// # Safety
    /// As [`Tv::build`].
    unsafe fn build_at(&self, path: &mut Vec<Container>) -> *mut partial_T {
        let pt: *mut partial_T = unsafe { xcalloc(1, size_of::<partial_T>()) }.cast();
        let argv: *mut typval_T = if self.args.is_empty() {
            ptr::null_mut()
        } else {
            unsafe { xmalloc(size_of::<typval_T>() * self.args.len()) }.cast()
        };
        for (i, arg) in self.args.iter().enumerate() {
            unsafe { *argv.add(i) = arg.build_at(path) };
        }
        let dict = match &self.dict {
            None => ptr::null_mut(),
            Some(dict) => {
                let tv = unsafe { dict.build_at(path) };
                assert_eq!(tv.v_type, VAR_DICT, "a partial's dict is a dict");
                unsafe { tv.vval.v_dict }
            }
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

/// A container on the path from the root, for [`Tv::Cycle`].
#[derive(Clone, Copy)]
enum Container {
    List(*mut list_T),
    Dict(*mut dict_T),
}

impl Container {
    fn addr(self) -> *const c_void {
        match self {
            Container::List(l) => l.cast(),
            Container::Dict(d) => d.cast(),
        }
    }
}

/// `typvalt2lua`: read a value back out.
///
/// # Safety
/// `tv` points at a live `typval_T` whose contents are live.
pub(crate) unsafe fn read(tv: *const typval_T) -> Tv {
    let mut path = Vec::new();
    unsafe { read_at(tv, &mut path) }
}

/// `lst2tbl`: read a list back out, NULL included.
///
/// # Safety
/// `l` is NULL or points at a live list.
pub(crate) unsafe fn read_list(l: *const list_T) -> Tv {
    let mut path = Vec::new();
    unsafe { read_list_at(l, &mut path) }
}

/// `dct2tbl`: read a dict back out, NULL included.
///
/// # Safety
/// `d` is NULL or points at a live dict.
pub(crate) unsafe fn read_dict(d: *const dict_T) -> Tv {
    let mut path = Vec::new();
    unsafe { read_dict_at(d, &mut path) }
}

/// # Safety
/// As [`read`].
unsafe fn read_at(tv: *const typval_T, path: &mut Vec<Container>) -> Tv {
    let vval = unsafe { (*tv).vval };
    match unsafe { (*tv).v_type } {
        VAR_UNKNOWN => Tv::Unknown,
        VAR_SPECIAL => {
            assert_eq!(unsafe { vval.v_special }, kSpecialVarNull);
            Tv::Nil
        }
        VAR_BOOL => Tv::Bool(match unsafe { vval.v_bool } {
            b if b == kBoolVarTrue => true,
            b if b == kBoolVarFalse => false,
            other => panic!("not a boolean: {other}"),
        }),
        VAR_NUMBER => Tv::Int(unsafe { vval.v_number }),
        VAR_FLOAT => Tv::Float(unsafe { vval.v_float }),
        VAR_STRING => match unsafe { vval.v_string } {
            s if s.is_null() => Tv::NullStr,
            s => Tv::Str(unsafe { CStr::from_ptr(s) }.to_bytes().to_vec()),
        },
        VAR_FUNC => match unsafe { vval.v_string } {
            s if s.is_null() => Tv::NullStr,
            s => Tv::Func(unsafe { CStr::from_ptr(s) }.to_bytes().to_vec()),
        },
        VAR_LIST => unsafe { read_list_at(vval.v_list, path) },
        VAR_DICT => unsafe { read_dict_at(vval.v_dict, path) },
        VAR_PARTIAL => Tv::Partial(Box::new(unsafe { read_partial(vval.v_partial, path) })),
        other => panic!("reading v_type {other} is not implemented"),
    }
}

/// # Safety
/// As [`read_list`].
unsafe fn read_list_at(l: *const list_T, path: &mut Vec<Container>) -> Tv {
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
unsafe fn read_dict_at(d: *const dict_T, path: &mut Vec<Container>) -> Tv {
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
unsafe fn read_partial(pt: *const partial_T, path: &mut Vec<Container>) -> Pt {
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

/// `tv_list_item_alloc`, which the crate keeps private: an uninitialised
/// item the caller fills in and hands to `tv_list_append`.
///
/// # Safety
/// The editor must be up.
pub(crate) unsafe fn list_item_alloc() -> *mut listitem_T {
    unsafe { xmalloc(size_of::<listitem_T>()) }.cast()
}

/// The spec's `li_alloc`: an item holding `VAR_UNKNOWN`, unlinked.
///
/// # Safety
/// As [`list_item_alloc`].
pub(crate) unsafe fn li_alloc() -> *mut listitem_T {
    let li = unsafe { list_item_alloc() };
    unsafe {
        (*li).li_next = ptr::null_mut();
        (*li).li_prev = ptr::null_mut();
        (*li).li_tv = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union { v_number: 0 },
        };
    }
    li
}

/// The spec's `list(...)`: a fresh list with `lv_refcount` 1 holding these
/// values.
///
/// # Safety
/// The editor must be up.
pub(crate) unsafe fn new_list(items: &[Tv]) -> *mut list_T {
    let tv = unsafe { Tv::List(items.to_vec()).build() };
    unsafe { tv.vval.v_list }
}

/// The spec's `dict{...}`: a fresh dict with `dv_refcount` 1.
///
/// # Safety
/// The editor must be up.
pub(crate) unsafe fn new_dict(entries: &[(&str, Tv)]) -> *mut dict_T {
    let entries: Vec<(Vec<u8>, Tv)> = entries
        .iter()
        .map(|(k, v)| (k.as_bytes().to_vec(), v.clone()))
        .collect();
    let tv = unsafe { Tv::Dict(entries).build() };
    unsafe { tv.vval.v_dict }
}

/// The spec's `list_items`: every item of `l`, front to back.
///
/// # Safety
/// `l` is NULL or points at a live list.
pub(crate) unsafe fn list_items(l: *const list_T) -> Vec<*mut listitem_T> {
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
pub(crate) unsafe fn dict_items(d: *const dict_T) -> Vec<(Vec<u8>, *mut dictitem_T)> {
    let ht = unsafe { &(*d).dv_hashtab };
    let mut out = Vec::new();
    for hi in ht.items() {
        let key = hi.hi_key;
        let di: *mut dictitem_T = unsafe { key.byte_sub(offset_of!(dictitem_T, di_key)) }.cast();
        out.push((unsafe { CStr::from_ptr(key) }.to_bytes().to_vec(), di));
    }
    out
}

/// The item `d` holds under `key`. Panics if there is none — a case that
/// wants the absence asserts it through `tv_dict_find`.
///
/// # Safety
/// As [`dict_items`].
pub(crate) unsafe fn di_of(d: *const dict_T, key: &str) -> *mut dictitem_T {
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
pub(crate) unsafe fn first_di(d: *const dict_T) -> *mut dictitem_T {
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
pub(crate) unsafe fn dict_watchers(d: *const dict_T) -> Vec<Watcher> {
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

/// The spec's `ga_alloc`: a `garray_T` on the caller's stack, initialised.
pub(crate) fn ga_alloc(itemsize: c_int, growsize: c_int) -> neovim::types::garray_T {
    let mut ga = neovim::types::garray_T {
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
pub(crate) unsafe fn eval0(expr: &str) -> Option<typval_T> {
    use neovim::eval::EVAL_EVALUATE;
    use neovim::types::evalarg_T;

    let mut tv = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VarLock::Unlocked,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut evalarg = evalarg_T {
        eval_flags: EVAL_EVALUATE as c_int,
        eval_getline: None,
        eval_cookie: ptr::null_mut(),
        eval_tofree: ptr::null_mut(),
    };
    // `eval0` takes a mutable buffer: it writes the terminator back over
    // what it consumed.
    let mut arg: Vec<c_char> = expr.bytes().map(|b| b as c_char).chain([0]).collect();
    let ok = unsafe {
        neovim::eval::eval0(
            arg.as_mut_ptr(),
            &raw mut tv,
            ptr::null_mut(),
            &raw mut evalarg,
        )
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
