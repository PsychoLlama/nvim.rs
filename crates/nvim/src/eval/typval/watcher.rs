//! `Dict` watchers and the `Callback` values they hold.
//!
//! [`Dict::watcher_add`] threads a `DictWatcher` onto `dv_watchers` and
//! [`dict_watcher_notify`] fires every watcher whose pattern matches a
//! key that just changed, building the `{old, new}` dictionary each callback
//! is handed.  The `callback_*` half is `Callback`'s own lifetime — funcref,
//! partial and LuaRef each reference-counted differently.
//!
//! The three walks over `dv_watchers` are written out rather than folded into
//! a shared iterator: each is upstream's `QUEUE_FOREACH`, which caches the
//! next node *before* running the body precisely so the body may unlink the
//! current one, and a callback fired from the middle of one can re-enter and
//! edit the queue.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::memory::ThinCString;
use crate::snprintf;
use ::core::ffi::CStr;

use super::*;
use crate::cstr;

/// Free `watcher` and the callback and pattern it owns.
///
/// # Safety
///
/// `watcher` must point at a live dictionary watcher, unaliased for the call.
pub(crate) unsafe fn tv_dict_watcher_free(watcher: *mut DictWatcher) {
    unsafe { callback_free(&raw mut (*watcher).callback) };
    unsafe { xfree((*watcher).key_pattern.cast()) };
    unsafe { xfree(watcher.cast()) };
}

impl Dict {
    /// Register `callback` to fire when a key matching `key_pattern`
    /// changes.  A trailing `*` in the pattern matches a prefix.
    ///
    /// The callback is taken over; the pattern is copied.
    pub fn watcher_add(&mut self, key_pattern: &[u8], callback: Callback) {
        // SAFETY: room for one watcher, filled in field by field below.
        let watcher =
            unsafe { xmalloc(::core::mem::size_of::<DictWatcher>()) }.cast::<DictWatcher>();
        // SAFETY: the allocation just made, and `key_pattern` is a slice.
        unsafe {
            (*watcher).key_pattern =
                xmemdupz(key_pattern.as_ptr().cast(), key_pattern.len()).cast();
        };
        // SAFETY: freshly allocated just above.
        let mut w = unsafe { Dw::new(watcher) };
        w.key_pattern_len = key_pattern.len();
        w.callback = callback;
        w.busy = false;
        w.needs_free = false;
        // SAFETY: this dictionary's own queue head, and a node on no queue.
        unsafe { queue_insert_tail(&raw mut self.watchers, &raw mut (*watcher).node) };
    }

    /// Unregister the watcher with this exact pattern and callback.
    ///
    /// A watcher removed while any watcher on the queue is mid-callback is
    /// only marked `needs_free`; [`dict_watcher_notify`] unlinks it when the
    /// walk that is running finishes.
    ///
    /// `callback` is only compared against the registered ones — it stays
    /// the caller's to free, which is why it arrives borrowed. Contrast
    /// [`Dict::watcher_add`], which takes its callback over.
    pub fn watcher_remove(&mut self, key_pattern: &[u8], callback: &Callback) -> bool {
        let head = &raw mut self.watchers;
        let mut watcher = ::core::ptr::null_mut::<DictWatcher>();
        let mut matched = false;
        let mut queue_is_busy = false;
        // QUEUE_FOREACH; `w` stays on the matching node when the walk breaks.
        let mut w = self.watchers.next;
        while w != head {
            // SAFETY: an entry of this dictionary's watcher queue.
            let next = unsafe { (*w).next };
            // SAFETY: as above.
            watcher = unsafe { tv_dict_watcher_node_data(w) };
            // SAFETY: as above.
            let wd = unsafe { Dw::new(watcher) };
            if wd.busy {
                queue_is_busy = true;
            }
            // SAFETY: the watcher's own callback and pattern.
            if unsafe { tv_callback_equal(&raw const (*watcher).callback, callback) }
                && wd.key_pattern_len == key_pattern.len()
                // SAFETY: the watcher's own pattern, that many bytes of it.
                && unsafe { cstr::slice_at(wd.key_pattern, key_pattern.len()) } == key_pattern
            {
                matched = true;
                break;
            }
            w = next;
        }

        if !matched {
            return false;
        }

        if queue_is_busy {
            // SAFETY: the watcher the walk stopped on.
            unsafe { (*watcher).needs_free = true };
        } else {
            // SAFETY: as above, and the node it is on.
            unsafe { queue_remove(w) };
            // SAFETY: as above, now off the queue.
            unsafe { tv_dict_watcher_free(watcher) };
        }
        true
    }
}

/// Whether `cb1` and `cb2` name the same function.
///
/// # Safety
///
/// `cb1` must point at an initialized callback. `cb2` must point at an
/// initialized callback.
pub unsafe fn tv_callback_equal(cb1: *const Callback, cb2: *const Callback) -> bool {
    // SAFETY: the caller's callbacks, live for the comparison.
    match unsafe { (&*cb1, &*cb2) } {
        (Callback::None, Callback::None) => true,
        // SAFETY: a funcref names its own NUL-terminated bytes.
        (Callback::Funcref(a), Callback::Funcref(b)) => unsafe { cstr::eq(*a, *b) },
        (Callback::Partial(a), Callback::Partial(b)) => a == b,
        (Callback::Lua(a), Callback::Lua(b)) => a == b,
        _ => false,
    }
}

/// Drop whatever `callback` holds and leave it `kCallbackNone`.
///
/// # Safety
///
/// `callback` must point at an initialized callback, unaliased for the call.
pub unsafe fn callback_free(callback: *mut Callback) {
    // SAFETY: the caller's promise: a live callback, whose payload it owns.
    match unsafe { &*callback } {
        Callback::Funcref(name) => {
            // SAFETY: a funcref owns its NUL-terminated name.
            unsafe { func_unref(*name) };
            unsafe { xfree(name.cast()) };
        }
        Callback::Partial(partial) => unsafe { partial_unref(*partial) },
        // NLUA_CLEAR_REF
        Callback::Lua(reference) => {
            if *reference != LUA_NOREF {
                // SAFETY: a registry index, not a pointer.
                unsafe { api_free_luaref(*reference) };
            }
        }
        Callback::None => {}
    }
    // SAFETY: as above.
    unsafe { *callback = Callback::None };
}

/// Store `cb` in `tv` as a Vimscript value, taking a reference to it.
///
/// A Lua callback has no Vimscript form and comes out as `v:null`.
///
/// # Safety
///
/// `cb` must point at an initialized callback, unaliased for the call. `tv`
/// must point at an initialized typval, unaliased for the call.
pub unsafe fn callback_put(cb: *mut Callback, tv: &mut TypVal) {
    // SAFETY: the caller's promise: a live typval.
    let mut value = unsafe { Tv::new(tv) };
    // SAFETY: as above, and a live callback whose payload it owns.
    match unsafe { &*cb } {
        Callback::Partial(partial) => {
            // SAFETY: the partial the callback holds; the typval takes a
            // reference of its own.
            value.write_partial(unsafe { PartialRef::retained(*partial) });
        }
        Callback::Funcref(name) => {
            // SAFETY: a funcref names its own NUL-terminated bytes.
            let copy = ThinCString::from_cstr(unsafe { CStr::from_ptr(*name) });
            value.write_func_name(Some(copy));
            // SAFETY: as above.
            unsafe { func_ref(*name) };
        }
        // A Lua callback and no callback at all have no Vimscript form.
        Callback::Lua(_) | Callback::None => {
            value.write_special(kSpecialVarNull);
        }
    }
}

/// Copy `src` into `dest`, taking a reference to whatever it holds.
///
/// # Safety
///
/// `dest` must point at an initialized callback, unaliased for the call.
/// `src` must point at an initialized callback, unaliased for the call.
pub unsafe fn callback_copy(dest: *mut Callback, src: *mut Callback) {
    // SAFETY: the caller's callbacks; `dest` need not hold a value yet, and
    // a `Callback` has no destructor to run over what was there.
    let copy = match unsafe { &*src } {
        Callback::Partial(partial) => {
            // SAFETY: the partial the source holds; the destination becomes
            // a second owner of it.
            unsafe { (**partial).pt_refcount.retain() };
            Callback::Partial(*partial)
        }
        Callback::Funcref(name) => {
            // SAFETY: a funcref names its own NUL-terminated bytes.
            unsafe {
                func_ref(*name);
                Callback::Funcref(xstrdup(*name))
            }
        }
        Callback::Lua(reference) => Callback::Lua(api_new_luaref(*reference)),
        Callback::None => Callback::None,
    };
    // SAFETY: as above.
    unsafe { *dest = copy };
}

impl Callback {
    /// Release what this holds and leave it [`Callback::None`]: the safe
    /// spelling of [`callback_free`] for a callback the caller owns.
    pub fn clear(&mut self) {
        // SAFETY: a `&mut` is a live, unaliased callback.
        unsafe { callback_free(self) };
    }

    /// A second owner of what this holds: [`callback_copy`] into a fresh
    /// value.
    pub fn duplicate(&self) -> Callback {
        let mut copy = Callback::None;
        // SAFETY: both are live; the source is only read.
        unsafe { callback_copy(&mut copy, ::core::ptr::from_ref(self).cast_mut()) };
        copy
    }

    /// Store this callback in `tv` as a Vimscript value, taking a reference
    /// of the value's own: [`callback_put`].
    pub fn put(&self, tv: &mut TypVal) {
        // SAFETY: as [`Callback::duplicate`]; `tv` is a `&mut`.
        unsafe { callback_put(::core::ptr::from_ref(self).cast_mut(), tv) };
    }
}

/// A freshly allocated description of `cb`, as `string()` prints it.
///
/// # Safety
///
/// A `Partial` callback's pointer must name a live partial.
pub unsafe fn callback_to_string(callback: &Callback) -> *mut ::core::ffi::c_char {
    if let Callback::Lua(reference) = callback {
        // SAFETY: a registry index.
        return unsafe { nlua_funcref_str(*reference) };
    }

    let msglen: size_t = 100;
    let msg = unsafe { xmallocz(msglen) }.cast::<::core::ffi::c_char>();
    // SAFETY: `msg` is `msglen` writable bytes, and each name below is a
    // NUL-terminated string the callback owns.
    match callback {
        Callback::Funcref(name) => unsafe {
            snprintf!(msg, msglen, c"<vim function: %s>".as_ptr(), *name);
        },
        Callback::Partial(partial) => unsafe {
            let name = (**partial).pt_name;
            snprintf!(msg, msglen, c"<vim partial: %s>".as_ptr(), name);
        },
        // Anything else leaves the message an empty string.
        _ => unsafe { *msg = 0 },
    }
    msg
}

/// Whether `watcher`'s pattern matches `key`.  A trailing `*` makes it a
/// prefix match.
///
/// # Safety
///
/// `watcher` must point at a live entry of some dictionary's watcher chain
/// and `key` at a NUL-terminated key, both live for the call.
pub(crate) unsafe fn tv_dict_watcher_matches(
    watcher: *mut DictWatcher,
    key: *const ::core::ffi::c_char,
) -> bool {
    let len = unsafe { (*watcher).key_pattern_len };
    if len != 0
        && ::core::ffi::c_int::from(unsafe { *(*watcher).key_pattern.add(len - 1) }) == '*' as i32
    {
        return unsafe { cstr::prefix_eq(key, (*watcher).key_pattern, len - 1) };
    }
    unsafe { cstr::eq(key, (*watcher).key_pattern) }
}

/// [`dict_watcher_notify`] for the dictionary `dict` holds.
pub(crate) fn notify_watchers(
    dict: &DictRef,
    key: &CStr,
    newtv: Option<&TypVal>,
    oldtv: Option<&TypVal>,
) {
    // SAFETY: a dictionary the handle keeps alive across the call.
    unsafe { dict_watcher_notify(dict.as_ptr(), key, newtv, oldtv) }
}

/// Fire every watcher of `dict` that matches `key`, handing each
/// `(dict, key, {old, new})`.
///
/// A callback may add or remove watchers, and may re-enter this function; the
/// `busy` flag is what stops a watcher firing inside its own callback, and the
/// second walk is the deferred deletion the first one could not do.
///
/// **The dictionary stays a pointer.** Every callback fired here runs
/// arbitrary Vimscript or Lua, which reaches this same dictionary through
/// whatever named it in the first place -- a scope, a variable, the
/// argument the callback is handed -- so no borrow of it may be live across
/// the walk. The reference the callbacks run under is the retain below.
///
/// # Safety
///
/// `dict` must point at a live dictionary.
pub unsafe fn dict_watcher_notify(
    dict: *mut Dict,
    key: &CStr,
    newtv: Option<&TypVal>,
    oldtv: Option<&TypVal>,
) {
    // Slot 0 names the caller's dictionary, which the retain below holds
    // across the callbacks; the other two are this frame's own.
    let mut argv = CallFrame::<3>::new();
    // SAFETY: the caller's dictionary, live for the call. The slot *names*
    // it: the reference the callbacks run under is the retain below, and
    // `tv_dict_unref` at the bottom is what gives it back.
    argv.push_naming(TypVal::dict(unsafe { DictRef::owning(dict) }));
    argv.push_owned(TypVal::string(Some(ThinCString::from_cstr(key))));
    argv.push_owned(TypVal::dict(Some(tv_dict_alloc())));
    let event = argv.args()[2].dict_or_null();

    // A `&[u8]` is upstream's `S_LEN(…)`: the key is copied at exactly the
    // length given, and the item appends the NUL itself.
    let add = |name: &[u8], from: &TypVal| {
        // SAFETY: the dictionary this call just allocated into the frame.
        let _ = unsafe { (*event).add_tv(name, from) };
    };
    if let Some(newtv) = newtv {
        add(b"new", newtv);
    }
    if let Some(oldtv) = oldtv
        && oldtv.v_type() != VAR_UNKNOWN
    {
        add(b"old", oldtv);
    }

    let mut any_needs_free = false;
    // Hold the dictionary across the callbacks: one of them may drop the
    // last other reference to it.
    // SAFETY: the caller's promise: a live dictionary.
    let mut d = unsafe { Dt::new(dict) };
    d.dv_refcount.retain();
    // The queue head is a *field of* the dictionary, so its address is taken
    // from the raw pointer rather than through the handle: a `DerefMut`
    // reborrow ends with the expression that asked for it, and a raw pointer
    // outliving one is exactly what Stacked and Tree Borrows reject.
    let head = dv_watchers(dict);
    // QUEUE_FOREACH: the next node is read before the body, so a callback
    // that unlinks the current watcher does not strand the walk.
    let mut w = d.watchers.next;
    while w != head {
        let next = unsafe { (*w).next };
        let watcher = unsafe { tv_dict_watcher_node_data(w) };
        if !unsafe { (*watcher).busy } && unsafe { tv_dict_watcher_matches(watcher, key.as_ptr()) }
        {
            let mut rettv = TV_INITIAL_VALUE;
            // SAFETY: an entry of the dictionary's watcher queue.
            let mut wd = unsafe { Dw::new(watcher) };
            wd.busy = true;
            let cb = wd.field_ptr(::core::mem::offset_of!(DictWatcher, callback));
            unsafe { callback_call(cb, argv.args(), &mut rettv) };
            wd.busy = false;
            tv_clear(&mut rettv);
            if wd.needs_free {
                any_needs_free = true;
            }
        }
        w = next;
    }

    if any_needs_free {
        let mut w = d.watchers.next;
        while w != head {
            let next = unsafe { (*w).next };
            let watcher = unsafe { tv_dict_watcher_node_data(w) };
            // SAFETY: an entry of the dictionary's watcher queue.
            if unsafe { Dw::new(watcher) }.needs_free {
                unsafe { queue_remove(w) };
                unsafe { tv_dict_watcher_free(watcher) };
            }
            w = next;
        }
    }
    unsafe { tv_dict_unref(dict) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_cell::editor_state_lock;

    // Every case that registers a watcher is ignored under Miri: the first
    // `Dict::watcher_add` on a dictionary links the new node through the
    // queue head's `prev`, which points back at the head with the
    // allocation's own provenance, and that write lands while the
    // `&mut self` the method runs under is protected.

    /// A callback naming a function nothing defines. No case here fires
    /// one: only the bookkeeping is under test.
    fn never_called(name: &CStr) -> Callback {
        // SAFETY: a NUL-terminated name, copied into a block the callback
        // owns and `callback_free` releases.
        Callback::Funcref(unsafe { xstrdup(name.as_ptr()) })
    }

    /// The watchers of `d`, in registration order, as their patterns.
    fn patterns(d: &DictRef) -> Vec<Vec<u8>> {
        let head = dv_watchers(d.as_ptr());
        let mut out = Vec::new();
        // SAFETY: the dictionary's own queue head.
        let mut w = unsafe { (*head).next };
        while w != head {
            // SAFETY: an entry of the dictionary's watcher queue.
            let watcher = unsafe { &*tv_dict_watcher_node_data(w) };
            // SAFETY: the watcher's own pattern, that many bytes of it.
            out.push(
                unsafe { cstr::slice_at(watcher.key_pattern, watcher.key_pattern_len) }.to_vec(),
            );
            // SAFETY: as above.
            w = unsafe { (*w).next };
        }
        out
    }

    /// The first watcher of `d` whose pattern is `pattern`.
    fn watcher_of(d: &DictRef, pattern: &[u8]) -> *mut DictWatcher {
        let head = dv_watchers(d.as_ptr());
        // SAFETY: the dictionary's own queue head.
        let mut w = unsafe { (*head).next };
        while w != head {
            // SAFETY: an entry of the dictionary's watcher queue.
            let watcher = unsafe { tv_dict_watcher_node_data(w) };
            // SAFETY: as above.
            let wd = unsafe { &*watcher };
            // SAFETY: the watcher's own pattern, that many bytes of it.
            if unsafe { cstr::slice_at(wd.key_pattern, wd.key_pattern_len) } == pattern {
                return watcher;
            }
            // SAFETY: as above.
            w = unsafe { (*w).next };
        }
        panic!("no watcher on {pattern:?}");
    }

    /// Whether `pattern` (as a registered watcher) matches `key`.
    fn matches(d: &DictRef, pattern: &[u8], key: &CStr) -> bool {
        // SAFETY: a registered watcher and a NUL-terminated key.
        unsafe { tv_dict_watcher_matches(watcher_of(d, pattern), key.as_ptr()) }
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "UB in current code: Dict::watcher_add writes the queue head's self-link \
                  under a protected &mut Dict (Stacked Borrows: strongly protected Unique)"
    )]
    fn removing_a_watcher_takes_only_the_exact_registration() {
        let _serial = editor_state_lock();
        let mut d = tv_dict_alloc();
        d.watcher_add(b"a*", never_called(c"NeverCalledA"));
        d.watcher_add(b"abc", never_called(c"NeverCalledB"));
        d.watcher_add(b"*", never_called(c"NeverCalledA"));
        assert_eq!(
            patterns(&d),
            [b"a*".to_vec(), b"abc".to_vec(), b"*".to_vec()]
        );

        let mut a = never_called(c"NeverCalledA");
        let mut b = never_called(c"NeverCalledB");
        // The pattern matches but the callback does not, and vice versa.
        assert!(!d.watcher_remove(b"abc", &a));
        assert!(!d.watcher_remove(b"zz", &a));
        assert!(!d.watcher_remove(b"a", &a), "a prefix of the pattern");
        assert_eq!(patterns(&d).len(), 3);

        assert!(d.watcher_remove(b"abc", &b));
        assert_eq!(patterns(&d), [b"a*".to_vec(), b"*".to_vec()]);
        assert!(!d.watcher_remove(b"abc", &b), "already gone");
        assert!(d.watcher_remove(b"*", &a));
        assert_eq!(patterns(&d), [b"a*".to_vec()]);

        // The probes stay the caller's.
        a.clear();
        b.clear();
        // The last one goes with the dictionary.
        drop(d);
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "UB in current code: Dict::watcher_add writes the queue head's self-link \
                  under a protected &mut Dict (Stacked Borrows: strongly protected Unique)"
    )]
    fn a_pattern_matches_exactly_or_by_its_prefix() {
        let _serial = editor_state_lock();
        let mut d = tv_dict_alloc();
        for pattern in [&b"*"[..], b"a*", b"abc", b""] {
            d.watcher_add(pattern, Callback::None);
        }
        assert!(matches(&d, b"*", c"anything"));
        assert!(matches(&d, b"*", c""));
        assert!(matches(&d, b"a*", c"a"));
        assert!(matches(&d, b"a*", c"abd"));
        assert!(!matches(&d, b"a*", c"ba"));
        assert!(matches(&d, b"abc", c"abc"));
        assert!(!matches(&d, b"abc", c"abcd"));
        assert!(!matches(&d, b"abc", c"ab"));
        assert!(matches(&d, b"", c""));
        assert!(!matches(&d, b"", c"a"));
        // `Callback::None` equals itself, so it can be removed by pattern.
        assert!(d.watcher_remove(b"", &Callback::None));
        assert_eq!(patterns(&d).len(), 3);
    }

    /// A dictionary freed with its watchers still registered frees them:
    /// under Miri, a watcher left behind is a leak.
    #[test]
    #[cfg_attr(
        miri,
        ignore = "UB in current code: Dict::watcher_add writes the queue head's self-link \
                  under a protected &mut Dict (Stacked Borrows: strongly protected Unique)"
    )]
    fn freeing_a_watched_dict_frees_its_watchers() {
        let _serial = editor_state_lock();
        let mut d = tv_dict_alloc();
        d.watcher_add(b"*", never_called(c"NeverCalledA"));
        d.watcher_add(b"k", never_called(c"NeverCalledB"));
        d.add_number(b"k", 1).expect("a key used once");
        drop(d);
    }

    /// While any watcher is mid-callback, removing one only marks it: the
    /// walk that is running unlinks it when it finishes.  Nothing fires a
    /// callback here, so the walk is simulated by setting `busy`.
    #[test]
    #[cfg_attr(
        miri,
        ignore = "UB in current code: Dict::watcher_add writes the queue head's self-link \
                  under a protected &mut Dict (Stacked Borrows: strongly protected Unique)"
    )]
    fn removing_a_watcher_while_one_is_busy_defers_the_free() {
        let _serial = editor_state_lock();
        let mut d = tv_dict_alloc();
        d.watcher_add(b"x", never_called(c"NeverCalledA"));
        d.watcher_add(b"y", never_called(c"NeverCalledB"));
        let busy = watcher_of(&d, b"x");
        // SAFETY: a registered watcher.
        unsafe { (*busy).busy = true };

        let mut probe = never_called(c"NeverCalledB");
        assert!(d.watcher_remove(b"y", &probe));
        // Still on the queue, marked.
        assert_eq!(patterns(&d), [b"x".to_vec(), b"y".to_vec()]);
        // SAFETY: the watcher just marked, still registered.
        assert!(unsafe { (*watcher_of(&d, b"y")).needs_free });
        // A second removal finds it again and marks it again.
        assert!(d.watcher_remove(b"y", &probe));

        // With nothing busy, a removal unlinks at once.
        // SAFETY: as above.
        unsafe { (*busy).busy = false };
        let mut a = never_called(c"NeverCalledA");
        assert!(d.watcher_remove(b"x", &a));
        assert_eq!(patterns(&d), [b"y".to_vec()]);

        probe.clear();
        a.clear();
        // The marked watcher is the dictionary's to free.
        drop(d);
    }

    /// A notification no pattern matches runs nothing, and leaves a
    /// deferred removal deferred: only a walk that fired the marked watcher
    /// knows to unlink it.
    #[test]
    #[cfg_attr(
        miri,
        ignore = "UB in current code: Dict::watcher_add writes the queue head's self-link \
                  under a protected &mut Dict (Stacked Borrows: strongly protected Unique)"
    )]
    fn a_notification_nothing_matches_leaves_the_queue_alone() {
        let _serial = editor_state_lock();
        let mut d = tv_dict_alloc();
        d.watcher_add(b"x", never_called(c"NeverCalledA"));
        let w = watcher_of(&d, b"x");
        // SAFETY: a registered watcher.
        unsafe { (*w).needs_free = true };
        let refs = d.dv_refcount.get();
        notify_watchers(&d, c"other", Some(&TypVal::Number(1)), None);
        assert_eq!(d.dv_refcount.get(), refs);
        assert_eq!(patterns(&d), [b"x".to_vec()]);
        drop(d);
    }

    /// `duplicate` takes a reference of its own, and two copies of one
    /// callback are equal.
    #[test]
    fn a_duplicated_callback_equals_its_source() {
        let _serial = editor_state_lock();
        let mut a = never_called(c"NeverCalledA");
        let mut b = a.duplicate();
        // SAFETY: two live callbacks.
        assert!(unsafe { tv_callback_equal(&a, &b) });
        // SAFETY: as above.
        assert!(!unsafe { tv_callback_equal(&a, &Callback::None) });
        let (Callback::Funcref(pa), Callback::Funcref(pb)) = (&a, &b) else {
            panic!("a funcref copies to a funcref");
        };
        assert_ne!(pa, pb, "the name is copied, not shared");
        a.clear();
        b.clear();
        assert!(!a.is_set());
    }
}
