//! A `Dict` and the operations over a whole one.
//!
//! The items themselves, and the slots of the table that names them, are
//! [`dictitem`](super::dictitem)'s; the walk order they give is
//! user-visible, which is what [`Dict::items`] promises and what this
//! module's tests pin.
//!
//! [`tv_dict_alloc`] and [`tv_dict_unref`] are the reference-counted pair;
//! [`dict_clear`] empties one without freeing it.  The `Dict::add_*` family
//! is the C header's overload set, each taking the key as bytes and copying
//! exactly those.  [`dict_extend`] is `extend()` with its three `action`
//! modes, [`dict_copy`] is `copy()`/`deepcopy()` over a dictionary.
//!
//! The operations that reach the same dictionary again while they run take
//! a [`DictRef`] and borrow the dictionary one statement at a time:
//! [`dict_extend`] (the two arguments may be one dictionary, and watchers
//! are user code), [`dict_copy`] (a cycle is read back through the mark this
//! call writes), and [`dict_clear`] and [`tv_dict_free_contents`] (the values
//! they release may name the dictionary they were in). A removal answers a
//! [`RemovedItem`], dropped once the borrow of the table has ended.

#![forbid(unsafe_code)]

use crate::eval::collect::var_item_copy_with;
use crate::eval::userfunc::func_ref_name;
use crate::eval::vars::{valid_varname_named, var_check_fixed_named, var_check_ro_named};
use crate::mbyte::string_convert_bytes;
use crate::memory::ThinCString;
use crate::message_fmt::msg_cstr;
use ::core::ffi::CStr;

use super::*;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::types::{CONV_NONE, Failed};

impl Dict {
    /// Add `item`, which the dictionary takes over.  `Err` hands it back
    /// when the key is already there, or when it would shadow a builtin
    /// function in a scope dictionary.
    pub fn add_item(&mut self, item: Box<DictItem>) -> Result<(), Box<DictItem>> {
        if dict_wrong_func_name(self, &item) {
            return Err(item);
        }
        self.insert(item)
    }

    /// The tail every `add_*` below shares: hand `item` over, or free it
    /// again when the key is taken.
    ///
    /// **The value may not name this dictionary.** A taken key frees the
    /// item here, and releasing a container value *reads* the container --
    /// so a value naming `self` would be read while the exclusive borrow of
    /// `self` claims to be alone with it. Safe code cannot build such a
    /// value from a borrowed dictionary: a second handle needs the
    /// [`DictRef`] the borrow came from, and its holder adds through that
    /// handle one statement at a time ([`dict_extend`]).
    #[inline]
    fn add_or_free(&mut self, item: Box<DictItem>) -> Result<(), Failed> {
        self.add_item(item).map_err(|_refused| Failed)
    }

    /// Add a copy of `tv` under `key`.
    ///
    /// The copy takes its own reference, so `tv` stays the caller's either
    /// way — and the exclusive borrow is what rules out its being a value of
    /// this same dictionary.
    pub fn add_tv(&mut self, key: &[u8], tv: &TypVal) -> Result<(), Failed> {
        let mut item = DictItem::boxed(key);
        tv_copy(tv, &mut item.di_tv);
        self.add_or_free(item)
    }

    /// Add `tv` under `key`, taking the value over. A failure releases it
    /// with the item; the borrow rules out its naming this dictionary, as
    /// for [`add_tv`](Dict::add_tv).
    pub fn add_value(&mut self, key: &[u8], tv: TypVal) -> Result<(), Failed> {
        let mut item = DictItem::boxed(key);
        // The item holds `VAR_UNKNOWN`, so the overwrite releases nothing.
        item.di_tv.overwrite(tv);
        self.add_or_free(item)
    }

    /// Add `list` under `key`, taking the handle over.  A failure releases
    /// it with the item.
    pub fn add_list(&mut self, key: &[u8], list: Option<ListRef>) -> Result<(), Failed> {
        let mut item = DictItem::boxed(key);
        item.di_tv.write_list(list);
        self.add_or_free(item)
    }

    /// Add `dict` under `key`, taking the handle over.  A failure releases
    /// it with the item.
    ///
    /// The handle may name *this* dictionary: it is a reference count and
    /// nothing reads through it here, which is what makes `let d.self = d`
    /// work.
    pub fn add_dict(&mut self, key: &[u8], dict: Option<DictRef>) -> Result<(), Failed> {
        let mut item = DictItem::boxed(key);
        item.di_tv.write_dict(dict);
        self.add_or_free(item)
    }

    /// Add the number `nr` under `key`.
    pub fn add_number(&mut self, key: &[u8], nr: VarNumber) -> Result<(), Failed> {
        let mut item = DictItem::boxed(key);
        item.di_tv.write_number(nr);
        self.add_or_free(item)
    }

    /// Add the float `nr` under `key`.
    pub fn add_float(&mut self, key: &[u8], nr: Float) -> Result<(), Failed> {
        let mut item = DictItem::boxed(key);
        item.di_tv.write_float(nr);
        self.add_or_free(item)
    }

    /// Add the boolean `val` under `key`.
    pub fn add_bool(&mut self, key: &[u8], val: BoolVarValue) -> Result<(), Failed> {
        let mut item = DictItem::boxed(key);
        item.di_tv.write_boolean(val);
        self.add_or_free(item)
    }

    /// Add a copy of the string `val` under `key`; `None` stores NULL.
    pub fn add_str(&mut self, key: &[u8], val: Option<&CStr>) -> Result<(), Failed> {
        self.add_value(key, TypVal::string(val.map(ThinCString::from_cstr)))
    }

    /// Add a copy of the bytes `val` under `key`; `None` stores NULL. A NUL
    /// among the bytes ends the string for every reader, as `xstrndup`'s
    /// copy did.
    pub fn add_str_len(&mut self, key: &[u8], val: Option<&[u8]>) -> Result<(), Failed> {
        self.add_value(key, TypVal::string(val.map(ThinCString::from_bytes)))
    }

    /// Add `val` under `key`, taking it over whether the key was free or
    /// not.
    pub fn add_allocated_str(
        &mut self,
        key: &[u8],
        val: Option<ThinCString>,
    ) -> Result<(), Failed> {
        self.add_value(key, TypVal::string(val))
    }

    /// Add a funcref to the function `name` under `key`.
    ///
    /// Only the name is copied; the funcref counts as a use of the function
    /// once it is stored. A refused item is released as any funcref is,
    /// which is upstream's.
    pub fn add_func(&mut self, key: &[u8], name: &CStr) -> Result<(), Failed> {
        let mut item = DictItem::boxed(key);
        item.di_tv
            .write_func_name(Some(ThinCString::from_cstr(name)));
        self.add_or_free(item)?;
        func_ref_name(name);
        Ok(())
    }
}

/// Free every item of `dict`, leaving the dictionary allocated and empty.
///
/// **A handle, not `&mut Dict`.** The values it frees may name this very
/// dictionary -- `deepcopy()` of a self-referencing one leaves exactly that,
/// and so does any cycle the collector has not yet reached -- and releasing
/// such a value *reads* the dictionary again. Each removal is one statement's
/// borrow, and the value is released after it.
///
/// Nothing else may be walking the dictionary: the hashtab is locked for this
/// walk, because the walk removes as it goes.
pub fn dict_clear(dict: &DictRef) {
    dict.edit().lock_table();
    let mut cursor = DictCursor::new(dict);
    while let Some(slot) = cursor.next(dict) {
        let removed = dict.edit().remove_at(slot);
        drop(removed);
    }
    dict.edit().unlock_table();
}

impl Dict {
    /// Mark every key read-only and fixed.
    pub fn set_keys_readonly(&mut self) {
        for di in self.items_mut() {
            di.di_flags |= (DI_FLAGS_RO | DI_FLAGS_FIX) as uint8_t;
        }
    }

    /// `extend(d, d, action)`: the case where the two dictionaries are one.
    ///
    /// Every key the walk finds is already there, so the `"keep"`,
    /// `"force"` and `"move"` branches are all no-ops — `"force"` by
    /// upstream's own `di2 != di1` guard — and what is left is `"error"`
    /// reporting the first key, plus the scope check that runs ahead of it.
    pub fn extend_from_self(&mut self, action: u8) {
        let scoped = self.dv_scope != VAR_NO_SCOPE;
        for di in self.items() {
            if scoped && !valid_varname_named(di.key()) {
                break;
            }
            if action == b'e' {
                let key = msg_bytes(di.key());
                semsg!("E737: Key already exists: {key}");
                break;
            }
        }
    }
}

/// `extend(d1, d2, action)`: fold `d2`'s items into `d1`.
///
/// **`d1` and `d2` may be the same dictionary**, and every watcher this
/// fires is user code that can reach either of them, so both are handles and
/// the walk over `d2` is a slot cursor: an item is re-read from its slot
/// after anything that can run user code, and what a notification is handed
/// is copied out first. `"move"` empties `d2`, so it must not be the same
/// dictionary as `d1` and must not be locked against a walk.
pub fn dict_extend(d1: &DictRef, d2: &DictRef, action: u8) {
    let watched = dict_is_watched(Some(d1));
    let arg_errmsg = gettext(c"extend() argument").to_bytes();

    if action == b'm' {
        // don't rehash on hash_remove()
        d2.edit().lock_table();
    }

    let mut cursor = DictCursor::new(d2);
    while let Some(slot2) = cursor.next(d2) {
        let Some(item2) = d2.item_at(slot2) else {
            continue;
        };
        let slot1 = d1.slot_of(item2.key());
        // Check the key to be valid when adding to any scope.
        if d1.dv_scope != VAR_NO_SCOPE && !valid_varname_named(item2.key()) {
            break;
        }
        let Some(slot1) = slot1 else {
            if action == b'm' {
                // Cheap way to move a dict item from "d2" to "d1". If the
                // add would fail, "d2" keeps it.
                if dict_wrong_func_name(d1, item2) {
                    continue;
                }
                let key = ThinCString::from_bytes(item2.key());
                let moved = match d2.edit().remove_at(slot2) {
                    RemovedItem::Allocated(item) => item,
                    // An item embedded in its owner cannot change tables;
                    // what moves is its value, in an item of its own.
                    RemovedItem::Embedded(value) => {
                        let mut item = DictItem::boxed(key.as_bytes());
                        item.di_tv.overwrite(value);
                        item
                    }
                };
                // Note upstream does not gate this on `watched`, unlike the
                // copying branch below.
                let new = moved.di_tv.clone();
                let added = d1.edit().insert(moved);
                debug_assert!(added.is_ok(), "the key was not in d1");
                drop(added);
                dict_watcher_notify(d1, key.as_cstr(), Some(&new), None);
            } else {
                let new_item = tv_dict_item_copy(item2);
                let note = watched.then(|| {
                    (
                        ThinCString::from_bytes(new_item.key()),
                        new_item.di_tv.clone(),
                    )
                });
                let added = d1.edit().add_item(new_item);
                if added.is_err() {
                    drop(added);
                } else if let Some((key, new)) = note {
                    dict_watcher_notify(d1, key.as_cstr(), Some(&new), None);
                }
            }
            continue;
        };
        if action == b'e' {
            let key = msg_bytes(item2.key());
            semsg!("E737: Key already exists: {key}");
            break;
        }
        if action != b'f' || d1.ptr_eq(d2) {
            continue;
        }
        let Some(item1) = d1.item_at(slot1) else {
            continue;
        };
        if value_check_lock_named(item1.di_lock, arg_errmsg)
            || var_check_ro_named(::core::ffi::c_int::from(item1.di_flags), arg_errmsg)
        {
            break;
        }
        // Disallow replacing a builtin function.
        if dict_wrong_func_name(d1, item2) {
            break;
        }

        let oldtv = if watched {
            item1.di_tv.clone()
        } else {
            TypVal::Unknown
        };
        // Upstream's order: the old value goes, then the new one is copied
        // in. The old one is released once the borrow of `d1` has ended.
        let cleared = d1.edit().item_at_mut(slot1).map(|item1| item1.di_tv.take());
        drop(cleared);
        let Some(item2) = d2.item_at(slot2) else {
            continue;
        };
        let new = item2.di_tv.clone();
        let note = watched.then(|| (ThinCString::from_bytes(item2.key()), new.clone()));
        if let Some(item1) = d1.edit().item_at_mut(slot1) {
            item1.di_tv = new;
        }

        if let Some((key, new)) = note {
            dict_watcher_notify(d1, key.as_cstr(), Some(&new), Some(&oldtv));
        }
        drop(oldtv);
    }

    if action == b'm' {
        d2.edit().unlock_table();
    }
}

/// Whether `d1` and `d2` hold the same keys with equal values.
///
/// The two borrows may name one dictionary — comparing a value to itself is
/// a read on both sides — and the identity test is the fast path for that.
/// Comparing values can recurse, so a cycle must already have been ruled out
/// by the caller's `copy_id` bookkeeping.
pub fn dict_equal(d1: Option<&Dict>, d2: Option<&Dict>, ic: bool) -> bool {
    let at = |d: Option<&Dict>| d.map_or(::core::ptr::null(), ::core::ptr::from_ref);
    if at(d1) == at(d2) {
        return true;
    }
    let len1 = dict_len(d1);
    if len1 != dict_len(d2) {
        return false;
    }
    if len1 == 0 {
        return true;
    }
    let (Some(d1), Some(d2)) = (d1, d2) else {
        return false;
    };

    for di1 in d1.items() {
        let Some(di2) = d2.find(di1.key()) else {
            return false;
        };
        if !tv_equal(&di1.di_tv, &di2.di_tv, ic) {
            return false;
        }
    }
    true
}

/// Copy `orig`, deeply when `deep`, converting keys through `conv`.
///
/// `copy_id` is the garbage collector's mark: non-zero records the copy on
/// the original so a self-referencing dictionary resolves to the same copy.
/// A non-zero one must be one the caller reserved from `get_copyID`: a stale
/// one makes an unrelated walk think this dictionary is already visited.
///
/// A deep copy re-enters through `var_item_copy`, and a dictionary that
/// holds itself is read again from in there -- through the mark this call
/// has just written onto it -- so the walk is a slot cursor over the handle.
pub fn dict_copy(
    conv: Option<&VimConv>,
    orig: &DictRef,
    deep: bool,
    copy_id: ::core::ffi::c_int,
) -> Option<DictRef> {
    let mut copy = tv_dict_alloc();
    if copy_id != 0 {
        orig.remember_copy(copy_id, &copy);
    }
    let conv = conv.filter(|conv| conv.vc_type != CONV_NONE);
    let mut cursor = DictCursor::new(orig);
    while let Some(slot) = cursor.next(orig) {
        if got_int.get() {
            break;
        }
        let Some(item) = orig.item_at(slot) else {
            continue;
        };
        let key = item.key();
        let mut new_item = match conv {
            None => DictItem::boxed(key),
            Some(conv) => match string_convert_bytes(conv, key) {
                (Some(converted), _) => DictItem::boxed(&converted),
                // The conversion failed: keep the original key, but at the
                // length the conversion left behind.
                (None, len) => DictItem::boxed(&key[..len.min(key.len())]),
            },
        };
        if deep {
            if var_item_copy_with(conv, &item.di_tv, &mut new_item.di_tv, deep, copy_id).is_err() {
                drop(new_item);
                break;
            }
        } else {
            tv_copy(&item.di_tv, &mut new_item.di_tv);
        }
        if let Err(refused) = copy.add_item(new_item) {
            drop(refused);
            break;
        }
    }

    if got_int.get() {
        // The partial copy goes with the handle, which is its only
        // reference.
        return None;
    }
    Some(copy)
}

/// Allocate an empty dictionary with the given lock status.
///
/// The handle owns the one reference the dictionary arrives with; see
/// [`tv_dict_alloc`].
pub fn tv_dict_alloc_lock(lock: VarLock) -> DictRef {
    let mut d = tv_dict_alloc();
    d.dv_lock = lock;
    d
}

/// Allocate an empty dictionary and store it in `ret_tv` as the return value.
pub fn tv_dict_alloc_ret(ret_tv: &mut TypVal) {
    ret_tv.write_dict(Some(tv_dict_alloc_lock(VarLock::Unlocked)));
}

/// `remove()` over a dictionary: move `args[0][args[1]]` into `result`.
///
/// `result` must hold no value yet. `arg_errmsg` names the argument in a
/// lock error, translated.
pub fn tv_dict_remove(args: &[TypVal], result: &mut TypVal, arg_errmsg: &'static CStr) {
    let mut numbuf = NumBuf::new();
    if args.len() > 2 {
        let arg0 = "remove()";
        semsg!("E118: Too many arguments for function: {arg0}");
        return;
    }

    let TypVal::Dict(dict) = &args[0] else {
        return;
    };
    let Some(dict) = &**dict else {
        return;
    };
    let name = gettext(arg_errmsg).to_bytes();
    if value_check_lock_named(dict.dv_lock, name) {
        return;
    }
    let Some(key) = numbuf.string_chk(&args[1]) else {
        return;
    };
    let Some(slot) = dict.slot_of(key.to_bytes()) else {
        let key = msg_cstr(key);
        semsg!("E716: Key not present in Dictionary: \"{key}\"");
        return;
    };
    let flags = dict
        .item_at(slot)
        .map_or(0, |item| ::core::ffi::c_int::from(item.di_flags));
    if var_check_fixed_named(flags, name) || var_check_ro_named(flags, name) {
        return;
    }

    // Move the value out rather than copying it: `result` takes the
    // reference the item held.
    let mut removed = dict.edit().remove_at(slot);
    *result = removed.value_mut().take();
    drop(removed);
    if dict_is_watched(Some(dict)) {
        dict_watcher_notify(dict, key, None, Some(result));
    }
}

/// Free every item and watcher of `dict`, leaving the `Dict` itself
/// allocated and empty: the free path.
///
/// Nothing else may be walking the dictionary: the hashtab is locked for the
/// walk, so a re-entrant call would see a half-emptied dictionary.
pub fn tv_dict_free_contents(dict: &DictRef) {
    // Lock the hashtab so the removals below cannot rehash it under the
    // walk.
    dict.edit().lock_table();
    let mut cursor = DictCursor::new(dict);
    while let Some(slot) = cursor.next(dict) {
        // Out of the table before it is freed, so that a release that
        // reaches this dictionary does not see a freed value.
        let removed = dict.edit().remove_at(slot);
        drop(removed);
    }

    let watchers = ::core::mem::take(&mut dict.edit().watchers);
    drop(watchers);

    let dict = dict.edit();
    dict.dv_hashtab.ht_locked -= 1;
    hash_reset(&mut dict.dv_hashtab);
}

impl Dict {
    /// How many entries the dictionary holds.
    pub fn len(&self) -> usize {
        self.dv_hashtab.ht_used
    }

    /// Whether the dictionary holds no entries.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_cell::editor_state_lock;

    /// A dictionary holding `keys`, each under its own position as a number.
    fn dict_of(keys: &[&str]) -> DictRef {
        let mut d = tv_dict_alloc();
        for (n, key) in keys.iter().enumerate() {
            let nr = VarNumber::try_from(n).expect("a short dict");
            d.add_number(key.as_bytes(), nr).expect("a key used once");
        }
        d
    }

    /// The keys of `d` in slot order -- the order `keys()` shows.
    fn slot_order(d: &Dict) -> Vec<String> {
        d.items()
            .map(|di| String::from_utf8(di.key().to_vec()).expect("an ASCII key"))
            .collect()
    }

    /// The number `d[key]` holds, or `None` when there is no such key.
    fn number_at(d: &Dict, key: &[u8]) -> Option<VarNumber> {
        d.find(key).map(|di| di.di_tv.number_or_zero())
    }

    /// **The slot order is user-visible**, so it is pinned here rather than
    /// described: `keys()` shows this, and a change to the hash, the probe
    /// sequence or the resize thresholds would change it.
    ///
    /// The keys straddle the first two rehashes, and are of several shapes
    /// and lengths, which is what makes this more than a spelling of
    /// insertion order -- the answer is neither that nor sorted.
    #[test]
    fn the_slot_order_survives_growth() {
        let _held = editor_state_lock();
        const KEYS: [&str; 14] = [
            "a",
            "bb",
            "ccc",
            "dddd",
            "k0",
            "k1",
            "k2",
            "k3",
            "k4",
            "k5",
            "zz",
            "Z",
            "_",
            "a_long_key_past_the_inline_cap",
        ];
        let d = dict_of(&KEYS);
        assert_eq!(
            slot_order(&d),
            [
                "dddd",
                "a",
                "zz",
                "a_long_key_past_the_inline_cap",
                "Z",
                "k5",
                "k0",
                "k1",
                "k2",
                "k3",
                "k4",
                "bb",
                "ccc",
                "_",
            ]
        );
        assert_eq!(d.len(), KEYS.len());
    }

    /// A rehash moves *slots*, not items: an item is its own allocation, so
    /// the address a lookup answered before a growth is the address it
    /// answers after one.
    ///
    /// This is what lets [`Dict::find`] hand back a borrow of the item at
    /// all. What the borrow does **not** survive is the item being removed
    /// -- that frees it -- which is why it is a borrow of the dictionary.
    #[test]
    fn an_item_outlives_the_rehash_that_moves_its_slot() {
        let _held = editor_state_lock();
        let mut d = dict_of(&["first"]);
        let at = |d: &Dict| d.find(b"first").map(::core::ptr::from_ref);
        let before = at(&d);
        let slots_before = d.dv_hashtab.size();
        for n in 0..40 {
            let key = format!("k{n}");
            d.add_number(key.as_bytes(), VarNumber::from(n))
                .expect("a key used once");
        }
        assert!(d.dv_hashtab.size() > slots_before, "the table never grew");
        assert_eq!(before, at(&d));
        assert_eq!(number_at(&d, b"first"), Some(0));
        assert_eq!(number_at(&d, b"k39"), Some(39));
        assert_eq!(number_at(&d, b"absent"), None);
    }

    /// The empty string is a key like any other: `{'': 1}` holds one entry.
    ///
    /// It is also the one input on which the two hashes the C had disagree
    /// -- and they agree here, because a key that reaches the table is
    /// NUL-terminated and so carries no NUL of its own.
    #[test]
    fn the_empty_key_is_a_key() {
        let _held = editor_state_lock();
        let d = dict_of(&["", "a"]);
        assert_eq!(d.len(), 2);
        assert_eq!(number_at(&d, b""), Some(0));
        assert_eq!(number_at(&d, b"a"), Some(1));
        assert!(d.has_key(b""));
        assert!(!d.has_key(b"b"));
    }

    /// `extend(d, d)` walks the dictionary it is adding to. Every key it
    /// finds is already there, so nothing is added and nothing overwritten.
    #[test]
    fn extending_a_dictionary_with_itself_is_the_identity() {
        let _held = editor_state_lock();
        let mut d = dict_of(&["a", "b", "c"]);
        let order = slot_order(&d);
        d.extend_from_self(b'f');
        assert_eq!(slot_order(&d), order);
        assert_eq!(number_at(&d, b"a"), Some(0));
        assert_eq!(number_at(&d, b"c"), Some(2));
    }

    /// And the handle form reaches both cases: the two arguments may be one
    /// dictionary.
    #[test]
    fn the_branching_extend_reaches_both_cases() {
        let _held = editor_state_lock();
        let into = dict_of(&["a"]);
        let from = dict_of(&["b", "c"]);
        dict_extend(&into, &from, b'f');
        assert_eq!(into.len(), 3);
        assert_eq!(number_at(&into, b"b"), Some(0));
        dict_extend(&into, &into, b'f');
        assert_eq!(into.len(), 3);
        assert_eq!(number_at(&into, b"a"), Some(0));
    }

    /// A dictionary equals itself without walking into the comparison, and
    /// two dictionaries with the same keys and values are equal whatever
    /// order they were built in.
    #[test]
    fn equality_is_by_key_not_by_slot() {
        let _held = editor_state_lock();
        let d1 = dict_of(&["a", "b"]);
        assert!(dict_equal(Some(&d1), Some(&d1), false));
        let mut d2 = tv_dict_alloc();
        for (n, key) in [(1, b"b"), (0, b"a")] {
            d2.add_number(key, VarNumber::from(n))
                .expect("a key used once");
        }
        assert!(dict_equal(Some(&d1), Some(&d2), false));
        // `v:_null_dict` is not a two-entry dictionary, but it is an empty
        // one.
        assert!(!dict_equal(Some(&d1), None, false));
        assert!(dict_equal(None, None, false));
    }

    /// A deep copy of a dictionary that holds itself resolves to the *copy*,
    /// not to the original: `copy_id` is what records the answer on the way
    /// down, and the cycle is what would otherwise recurse forever.
    #[test]
    fn a_deep_copy_of_a_cycle_points_at_the_copy() {
        let _held = editor_state_lock();
        let mut d = dict_of(&["n"]);
        // The value is a second reference to the dictionary itself.
        let held = d.clone();
        d.add_dict(b"self", Some(held)).expect("a key used once");

        let copy_id = crate::eval::get_copy_id();
        // No conversion, and a fresh copy id.
        let copy = dict_copy(None, &d, true, copy_id).expect("the copy was not interrupted");
        assert_eq!(copy.len(), 2);
        assert_eq!(number_at(&copy, b"n"), Some(0));
        let inner = copy.find(b"self").expect("the copy kept the key");
        assert_eq!(
            inner.di_tv.dict_or_null(),
            copy.as_ptr(),
            "the cycle followed the original"
        );

        // Break the cycles so both dictionaries actually go away. A cycle
        // is exactly why this takes a handle: releasing the value reads the
        // dictionary it names, which is this one.
        dict_clear(&d);
        dict_clear(&copy);
    }

    /// A walk may remove the entry it is standing on, but only with the
    /// table locked: an unlocked removal may rehash and renumber the slots
    /// the cursor is counting through.
    #[test]
    fn a_locked_walk_may_remove_as_it_goes() {
        let _held = editor_state_lock();
        let d = dict_of(&["a", "b", "c", "d"]);
        d.edit().lock_table();
        let mut cursor = DictCursor::new(&d);
        while let Some(slot) = cursor.next(&d) {
            let even = d
                .item_at(slot)
                .is_some_and(|di| di.di_tv.number_or_zero() % 2 == 0);
            if even {
                let removed = d.edit().remove_at(slot);
                drop(removed);
            }
        }
        d.edit().unlock_table();
        assert_eq!(slot_order(&d), ["b", "d"]);
        assert_eq!(number_at(&d, b"a"), None);
        assert_eq!(number_at(&d, b"d"), Some(3));
    }

    /// [`dict_clear`] empties a dictionary without freeing it, and the
    /// table it leaves behind takes new keys in the order a fresh one does.
    #[test]
    fn clearing_leaves_a_usable_table() {
        let _held = editor_state_lock();
        let mut d = dict_of(&["a", "b", "c"]);
        dict_clear(&d);
        assert_eq!(d.len(), 0);
        assert!(d.is_empty());
        assert_eq!(number_at(&d, b"a"), None);
        let refilled = dict_of(&["x", "y"]);
        for (n, key) in [b"x", b"y"].iter().enumerate() {
            let nr = VarNumber::try_from(n).expect("a short dict");
            d.add_number(*key, nr).expect("a key used once");
        }
        assert_eq!(slot_order(&d), slot_order(&refilled));
    }

    /// The reference count of `d`.
    fn refs(d: &Dict) -> i32 {
        d.dv_refcount.get()
    }

    /// Hold `tv_in_free_unref_items` up for a scope, and put it down even
    /// when an assertion unwinds through it.
    struct Collecting;
    impl Collecting {
        fn start() -> Collecting {
            tv_in_free_unref_items.set(true);
            Collecting
        }
    }
    impl Drop for Collecting {
        fn drop(&mut self) {
            tv_in_free_unref_items.set(false);
        }
    }

    /// A deep copy of a list and a dictionary that hold each other: the
    /// copy's cycle runs through the two *copies*, and the originals'
    /// counts are untouched.
    #[test]
    fn a_deep_copy_of_a_list_dict_cycle_stays_inside_the_copy() {
        let _held = editor_state_lock();
        let mut d = dict_of(&["n"]);
        let mut l = tv_list_alloc(1);
        l.push_dict(Some(d.clone()));
        d.add_list(b"l", Some(l.clone())).expect("a key used once");
        assert_eq!((l.lv_refcount.get(), refs(&d)), (2, 2));

        let from = TypVal::list(Some(l.clone()));
        let mut to = TypVal::Unknown;
        let copy_id = crate::eval::get_copy_id();
        // No conversion, a fresh copy id.
        let copied = var_item_copy_with(None, &from, &mut to, true, copy_id);
        assert_eq!(copied, Ok(()));
        let cl = to.list_handle().expect("the copy is a list");
        let cd = list_items(Some(&cl))[0]
            .li_tv
            .dict_handle()
            .expect("the copied list holds a dictionary");
        assert!(!cl.ptr_eq(&l));
        assert!(!cd.ptr_eq(&d));
        let back = cd
            .find(b"l")
            .expect("the copy kept the key")
            .di_tv
            .list_or_null();
        assert_eq!(back, cl.as_ptr(), "the cycle followed the original");
        assert_eq!(number_at(&cd, b"n"), Some(0));
        // The extra reference of the value `from` holds is the list's.
        assert_eq!((l.lv_refcount.get(), refs(&d)), (3, 2));

        // Break both cycles at the dictionary, then let everything go.
        dict_clear(&d);
        dict_clear(&cd);
        drop((cl, cd));
        let mut from = from;
        tv_clear(&mut from);
        tv_clear(&mut to);
        drop(l);
        drop(d);
    }

    /// While the collector is freeing, a dictionary's items give back the
    /// references they hold on it without freeing it: the collector's
    /// second pass does that.
    #[test]
    fn a_dict_cycle_is_emptied_without_being_freed_by_the_collectors_first_pass() {
        let _held = editor_state_lock();
        let mut d = dict_of(&["n"]);
        let held = d.clone();
        d.add_dict(b"self", Some(held)).expect("a key used once");
        assert_eq!(refs(&d), 2, "the handle and the cycle");

        let collecting = Collecting::start();
        tv_dict_free_contents(&d);
        // Released by its own item, and still allocated.
        assert_eq!(refs(&d), 1);
        assert_eq!(d.len(), 0);
        drop(collecting);
        drop(d);
    }
}
