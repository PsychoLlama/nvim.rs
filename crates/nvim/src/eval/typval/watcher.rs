//! `Dict` watchers: `dictwatcheradd()`'s registrations and the walk that
//! fires them.
//!
//! [`Dict::watcher_add`] appends a [`DictWatcher`] to the dictionary's
//! `watchers` and [`dict_watcher_notify`] fires every watcher whose pattern
//! matches a key that just changed, building the `{old, new}` dictionary each
//! callback is handed. The `Callback` values they hold are
//! [`crate::eval::callback`]'s.
//!
//! A watcher is an `Rc` so that the walk can hold the one it is firing while
//! the callback -- user code -- adds watchers to the same dictionary or
//! removes them; no borrow of the dictionary is live across a callback.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use ::core::cell::Cell;
use ::core::ffi::CStr;
use ::std::rc::Rc;

use crate::memory::ThinCString;

use super::*;

impl DictWatcher {
    /// Whether this watcher's pattern matches `key`. A trailing `*` makes
    /// it a prefix match.
    pub(crate) fn matches(&self, key: &[u8]) -> bool {
        match self.key_pattern.strip_suffix(b"*") {
            Some(prefix) => key.starts_with(prefix),
            None => key == self.key_pattern.as_slice(),
        }
    }
}

impl Dict {
    /// Register `callback` to fire when a key matching `key_pattern`
    /// changes.  A trailing `*` in the pattern matches a prefix.
    ///
    /// The callback is taken over; the pattern is copied.
    pub fn watcher_add(&mut self, key_pattern: &[u8], callback: Callback) {
        self.watchers.push(Rc::new(DictWatcher {
            callback,
            key_pattern: key_pattern.to_vec(),
            busy: Cell::new(false),
            needs_free: Cell::new(false),
        }));
    }

    /// Unregister the first watcher with this exact pattern and callback.
    ///
    /// While any watcher of the dictionary is mid-callback the match is only
    /// marked `needs_free`; [`dict_watcher_notify`] removes it when the walk
    /// that fired the marked watcher finishes.
    ///
    /// `callback` is only compared against the registered ones — it stays
    /// the caller's to free, which is why it arrives borrowed. Contrast
    /// [`Dict::watcher_add`], which takes its callback over.
    pub fn watcher_remove(&mut self, key_pattern: &[u8], callback: &Callback) -> bool {
        let mut queue_is_busy = false;
        let mut found = None;
        for (at, watcher) in self.watchers.iter().enumerate() {
            if watcher.busy.get() {
                queue_is_busy = true;
            }
            if watcher.callback.same_as(callback) && watcher.key_pattern == key_pattern {
                found = Some(at);
                break;
            }
        }
        let Some(at) = found else {
            return false;
        };

        if queue_is_busy {
            self.watchers[at].needs_free.set(true);
        } else {
            let removed = self.watchers.remove(at);
            drop(removed);
        }
        true
    }
}

/// Fire every watcher of `dict` that matches `key`, handing each
/// `(dict, key, {old, new})`.
///
/// A callback may add or remove watchers, and may re-enter this function; the
/// `busy` flag is what stops a watcher firing inside its own callback, and the
/// second pass is the deferred removal the first one could not do.
///
/// Every callback fired here runs arbitrary Vimscript or Lua, which reaches
/// this same dictionary through whatever named it in the first place, so the
/// walk holds a reference (the argument frame's) and the next watcher, and
/// borrows the watcher list one statement at a time.
pub fn dict_watcher_notify(
    dict: &DictRef,
    key: &CStr,
    newtv: Option<&TypVal>,
    oldtv: Option<&TypVal>,
) {
    let mut event = tv_dict_alloc();
    // A `&[u8]` is upstream's `S_LEN(…)`: the key is copied at exactly the
    // length given, and the item appends the NUL itself.
    if let Some(newtv) = newtv {
        let _ = event.add_tv(b"new", newtv);
    }
    if let Some(oldtv) = oldtv
        && oldtv.v_type() != VAR_UNKNOWN
    {
        let _ = event.add_tv(b"old", oldtv);
    }

    // Slot 0 is the reference the callbacks run under: one of them may drop
    // the last other reference to the dictionary.
    let mut argv = CallFrame::<3>::new();
    argv.push_owned(TypVal::dict(Some(dict.clone())));
    argv.push_owned(TypVal::string(Some(ThinCString::from_cstr(key))));
    argv.push_owned(TypVal::dict(Some(event)));

    let mut any_needs_free = false;
    // Upstream's QUEUE_FOREACH, which reads the next node *before* the body:
    // a watcher a callback appends behind the last one is not visited by
    // this walk, and one appended while others are still ahead of it is.
    // The next watcher is held, not indexed, and found again afterwards.
    let mut next = dict.watchers.first().cloned();
    while let Some(watcher) = next.take() {
        let at = dict.watchers.iter().position(|w| Rc::ptr_eq(w, &watcher));
        next = at.and_then(|at| dict.watchers.get(at + 1)).cloned();
        if watcher.busy.get() || !watcher.matches(key.to_bytes()) {
            continue;
        }
        let mut rettv = TV_INITIAL_VALUE;
        watcher.busy.set(true);
        callback_call(&watcher.callback, argv.args(), &mut rettv);
        watcher.busy.set(false);
        tv_clear(&mut rettv);
        if watcher.needs_free.get() {
            any_needs_free = true;
        }
    }

    if any_needs_free {
        let all = ::core::mem::take(&mut dict.edit().watchers);
        let (gone, kept): (Vec<_>, Vec<_>) = all.into_iter().partition(|w| w.needs_free.get());
        dict.edit().watchers = kept;
        drop(gone);
    }
    drop(argv);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_cell::editor_state_lock;

    /// A callback naming a function nothing defines. No case here fires
    /// one: only the bookkeeping is under test.
    fn never_called(name: &CStr) -> Callback {
        Callback::Funcref(::core::mem::ManuallyDrop::new(ThinCString::from_cstr(name)))
    }

    /// The watchers of `d`, in registration order, as their patterns.
    fn patterns(d: &DictRef) -> Vec<Vec<u8>> {
        d.watchers.iter().map(|w| w.key_pattern.clone()).collect()
    }

    /// The first watcher of `d` whose pattern is `pattern`.
    fn watcher_of(d: &DictRef, pattern: &[u8]) -> Rc<DictWatcher> {
        d.watchers
            .iter()
            .find(|w| w.key_pattern == pattern)
            .cloned()
            .unwrap_or_else(|| panic!("no watcher on {pattern:?}"))
    }

    /// Whether `pattern` (as a registered watcher) matches `key`.
    fn matches(d: &DictRef, pattern: &[u8], key: &CStr) -> bool {
        watcher_of(d, pattern).matches(key.to_bytes())
    }

    #[test]
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
    fn freeing_a_watched_dict_frees_its_watchers() {
        let _serial = editor_state_lock();
        let mut d = tv_dict_alloc();
        d.watcher_add(b"*", never_called(c"NeverCalledA"));
        d.watcher_add(b"k", never_called(c"NeverCalledB"));
        d.add_number(b"k", 1).expect("a key used once");
        drop(d);
    }

    /// While any watcher is mid-callback, removing one only marks it: the
    /// walk that is running removes it when it finishes.  Nothing fires a
    /// callback here, so the walk is simulated by setting `busy`.
    #[test]
    fn removing_a_watcher_while_one_is_busy_defers_the_free() {
        let _serial = editor_state_lock();
        let mut d = tv_dict_alloc();
        d.watcher_add(b"x", never_called(c"NeverCalledA"));
        d.watcher_add(b"y", never_called(c"NeverCalledB"));
        let busy = watcher_of(&d, b"x");
        busy.busy.set(true);

        let mut probe = never_called(c"NeverCalledB");
        assert!(d.watcher_remove(b"y", &probe));
        // Still registered, marked.
        assert_eq!(patterns(&d), [b"x".to_vec(), b"y".to_vec()]);
        assert!(watcher_of(&d, b"y").needs_free.get());
        // A second removal finds it again and marks it again.
        assert!(d.watcher_remove(b"y", &probe));

        // With nothing busy, a removal takes it out at once.
        busy.busy.set(false);
        drop(busy);
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
    /// knows to remove it.
    #[test]
    fn a_notification_nothing_matches_leaves_the_queue_alone() {
        let _serial = editor_state_lock();
        let mut d = tv_dict_alloc();
        d.watcher_add(b"x", never_called(c"NeverCalledA"));
        watcher_of(&d, b"x").needs_free.set(true);
        let refs = d.dv_refcount.get();
        dict_watcher_notify(&d, c"other", Some(&TypVal::Number(1)), None);
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
        assert!(a.same_as(&b));
        assert!(!a.same_as(&Callback::None));
        let (Callback::Funcref(pa), Callback::Funcref(pb)) = (&a, &b) else {
            panic!("a funcref copies to a funcref");
        };
        assert_ne!(pa.as_ptr(), pb.as_ptr(), "the name is copied, not shared");
        a.clear();
        b.clear();
        assert!(!a.is_set());
    }
}
