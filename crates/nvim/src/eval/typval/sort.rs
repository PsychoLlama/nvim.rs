//! `sort()` and `uniq()`: the comparators and the two driver loops.
//!
//! [`compare_builtin`] is the default ordering and [`compare_user`] the one
//! that calls a user function or dictionary method.  [`parse_sort_uniq_args`]
//! reads the optional `{how}` and `{dict}` arguments both builtins share.
//!
//! # The sort
//!
//! [`msort`] is a top-down merge sort written to make the *same comparator
//! calls in the same order* as the C library's `qsort` -- glibc's `msort`,
//! which is what `qsort` is from 2.39 on. Upstream sorts with `qsort`, and a
//! user comparison function is user code: it can count its calls, print, or
//! answer inconsistently, and which permutation of items it leaves is then
//! whatever that algorithm did with those answers. Rust's `sort_by` would
//! make different calls, and may panic on an order that is not total.
//! `sort()` breaks ties by the items' original indexes, which is what makes
//! it stable; `uniq()` asks only whether two neighbours compare equal.
//!
//! The ordering is the call's own [`SortInfo`]: a comparison function can
//! itself call `sort()`, which gets another. Upstream's `qsort` had nowhere
//! to put it and published it in a global, saved and restored around a
//! nested `sort()`.

#![forbid(unsafe_code)]

use super::*;
use crate::cjson::fpconv::strtod;
use crate::eval::encode::tv2string_bytes;
use crate::eval::userfunc::{CallWith, call_func_with};
use crate::memory::ThinCString;
use crate::os::cshim::strcoll;
use crate::semsg;
use crate::types::{Failed, ListWatch};
use ::core::cmp::Ordering;
use ::core::ffi::{CStr, c_int};
use ::std::borrow::Cow;
use ::std::ffi::CString;

/// What a comparison answers once the user's function has failed: anything
/// but zero, so that `uniq()` keeps the item, and big enough to stand out.
const ITEM_COMPARE_FAIL: c_int = 999;

/// The ordering `sort()` or `uniq()` was asked for, as
/// [`parse_sort_uniq_args`] read it from the arguments it borrows.
pub(crate) struct SortInfo<'a> {
    /// `'i'` or `1`: ignore case.
    ic: bool,
    /// `'l'`: the locale's collation.
    lc: bool,
    /// `'n'`: numbers by value, every string as `'`.
    numeric: bool,
    /// `'N'`: every item as a Number.
    numbers: bool,
    /// `'f'`: every item as a Float.
    float: bool,
    /// The user function to call, by name.
    func: Option<&'a CStr>,
    /// The partial the function came out of: bound arguments and `self`.
    partial: Option<&'a PartialRef>,
    /// The `{dict}` argument: `self` for a dictionary function.
    selfdict: Option<&'a DictRef>,
    /// The user function failed; every later comparison answers 0.
    func_err: bool,
}

impl SortInfo<'_> {
    /// The default ordering: `string()` forms, by their bytes.
    pub(crate) const fn new() -> Self {
        SortInfo {
            ic: false,
            lc: false,
            numeric: false,
            numbers: false,
            float: false,
            func: None,
            partial: None,
            selfdict: None,
            func_err: false,
        }
    }

    /// Whether the ordering is one of the built-in ones, rather than a call.
    fn is_builtin(&self) -> bool {
        self.func.is_none() && self.partial.is_none()
    }
}

/// A three-way comparison as the C comparator answers it.
fn sign(order: Ordering) -> c_int {
    match order {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

/// `cmp` on two scalars: upstream's `a == b ? 0 : a > b ? 1 : -1`, which
/// for a NaN float answers -1, as this does.
fn sign_of<T: PartialOrd>(a: T, b: T) -> c_int {
    if a == b {
        0
    } else if a > b {
        1
    } else {
        -1
    }
}

/// `strcasecmp()` in the C locale: the bytes compared with ASCII letters
/// folded to lower case.
fn ascii_casecmp(a: &[u8], b: &[u8]) -> c_int {
    let fold = |s: &[u8]| s.iter().map(u8::to_ascii_lowercase).collect::<Vec<u8>>();
    sign(fold(a).cmp(&fold(b)))
}

/// The text the default ordering compares for `tv`, against `other`.
///
/// `encode_tv2string()` puts quotes around a string and allocates memory.
/// Don't do that for string variables. Use a single quote when comparing
/// with a non-string to do what the docs promise.
fn sort_key<'v>(tv: &'v TypVal, other: &TypVal, numeric: bool) -> Cow<'v, [u8]> {
    if tv.v_type() == VAR_STRING {
        if other.v_type() != VAR_STRING || numeric {
            Cow::Borrowed(b"'")
        } else {
            Cow::Borrowed(tv.string_bytes())
        }
    } else {
        let mut text = tv2string_bytes(tv);
        // A NUL inside would have ended the C string there.
        if let Some(end) = text.iter().position(|&b| b == 0) {
            text.truncate(end);
        }
        Cow::Owned(text)
    }
}

/// Compare two values by the built-in ordering `info` selected: numeric,
/// float, or a string comparison of their `string()` forms.
fn compare_builtin(info: &SortInfo<'_>, tv1: &TypVal, tv2: &TypVal) -> c_int {
    if info.numbers {
        let v1 = tv_get_number(tv1);
        let v2 = tv_get_number(tv2);
        return sign_of(v1, v2);
    }
    if info.float {
        let v1 = tv_get_float(tv1);
        let v2 = tv_get_float(tv2);
        return sign_of(v1, v2);
    }
    // The first key is made before the second: `string()` can report an
    // error, and the order of two of them is upstream's.
    let key1 = sort_key(tv1, tv2, info.numeric);
    let key2 = sort_key(tv2, tv1, info.numeric);
    if info.numeric {
        // `strtod`'s prefix parse; a key with no number in it is 0.
        let n1 = strtod(&key1).0;
        let n2 = strtod(&key2).0;
        sign_of(n1, n2)
    } else if info.lc {
        // Only the locale's collation needs the terminated form.
        let terminated = |key: &[u8]| CString::new(key).expect("cut at the first NUL");
        strcoll(&terminated(&key1), &terminated(&key2))
    } else if info.ic {
        ascii_casecmp(&key1, &key2)
    } else {
        sign(key1.cmp(&key2))
    }
}

/// Compare two values by calling the user function `info` holds with
/// `argv`, which are copies: the callee may lock them, and may edit the
/// list they came from.
///
/// A failed call sets `func_err`, which makes the driver abandon the sort.
fn compare_user(info: &mut SortInfo<'_>, argv: [TypVal; 2]) -> c_int {
    let mut rettv = TV_INITIAL_VALUE;
    let with = CallWith {
        partial: info.partial,
        selfdict: info.selfdict,
        ..CallWith::new(true)
    };
    let name = info.func.unwrap_or(c"");
    let called = call_func_with(name, None, &mut rettv, &argv, with);
    drop(argv);

    let mut res;
    if called.is_err() {
        res = ITEM_COMPARE_FAIL;
        info.func_err = true;
    } else {
        let n = tv_get_number_chk(&rettv).unwrap_or_else(|_| {
            info.func_err = true;
            0
        });
        res = sign_of(n, 0);
    }
    if info.func_err {
        res = ITEM_COMPARE_FAIL; // return value has wrong type
    }
    tv_clear(&mut rettv);
    res
}

/// Sort `order` -- indexes, compared through `cmp` -- the way glibc's
/// `msort` sorts an array: split in two (the first half `n / 2` long), sort
/// each half, then merge, taking from the left run while `cmp(left, right)
/// <= 0`. The comparator is called exactly where that one calls it, in the
/// same order, and nothing here depends on its answers being consistent.
///
/// `tmp` is the merge buffer, shared by every level.
fn msort(order: &mut [usize], tmp: &mut Vec<usize>, cmp: &mut impl FnMut(usize, usize) -> c_int) {
    let n = order.len();
    if n <= 1 {
        return;
    }
    let n1 = n / 2;
    {
        let (left, right) = order.split_at_mut(n1);
        msort(left, tmp, cmp);
        msort(right, tmp, cmp);
    }
    tmp.clear();
    let (mut i, mut j) = (0, n1);
    while i < n1 && j < n {
        if cmp(order[i], order[j]) <= 0 {
            tmp.push(order[i]);
            i += 1;
        } else {
            tmp.push(order[j]);
            j += 1;
        }
    }
    // What is left of the left run goes after; what is left of the right
    // run is already where it belongs.
    tmp.extend_from_slice(&order[i..n1]);
    order[..tmp.len()].copy_from_slice(tmp);
}

/// `sort()` over `list`, in place.
///
/// The items are **taken out of the list** for the duration: the sort
/// permutes indexes into them, and a user comparator that re-enters the
/// evaluator must not be able to move them.  Upstream sorted the links in
/// place and left a comparator that edited the list holding freed items;
/// what a comparator sees here is an empty list instead, and anything it
/// appends is dropped when the sorted items go back (upstream leaked them).
pub(crate) fn do_sort(list: &ListRef, info: &mut SortInfo<'_>) {
    let taken = ::core::mem::take(&mut list.edit().lv_items);
    let len = taken.len();

    info.func_err = false;
    let builtin = info.is_builtin();
    let mut order: Vec<usize> = (0..len).collect();
    let mut tmp = Vec::with_capacity(len);
    msort(&mut order, &mut tmp, &mut |a, b| {
        let res = if builtin {
            compare_builtin(info, &taken[a].li_tv, &taken[b].li_tv)
        } else if info.func_err {
            // Shortcut after failure in previous call; compare all items
            // equal.
            return 0;
        } else {
            compare_user(info, [taken[a].li_tv.clone(), taken[b].li_tv.clone()])
        };
        if res != 0 {
            res
        } else if a > b {
            1
        } else {
            -1
        }
    });

    if info.func_err {
        emsg(gettext(c"E702: Sort compare function failed"));
        // The list is left as it was.
        let appended = ::core::mem::replace(&mut list.edit().lv_items, taken);
        drop(appended);
        return;
    }

    // Put the items back in the sorted order; each moves out exactly once.
    let mut taken = taken;
    let sorted: Vec<ListItem> = order
        .iter()
        .map(|&from| {
            let item = &mut taken[from];
            ListItem {
                li_tv: item.li_tv.take(),
                li_lock: item.li_lock,
            }
        })
        .collect();
    // Every value has moved out; what is left releases nothing.
    drop(taken);
    // A cursor stands on an item, not on a place, so it goes where the item
    // went.  Only paid for when something is actually walking the list.
    if list.is_watched() {
        let mut moved = vec![ListWatch::ENDED; len];
        for (dest, &from) in order.iter().enumerate() {
            moved[from] = index_of(dest);
        }
        watch_permute(list.edit(), &moved);
    }
    // Anything the comparator appended goes once the borrow has ended.
    let appended = ::core::mem::replace(&mut list.edit().lv_items, sorted);
    drop(appended);
}

/// `uniq()` over `list`, in place: drop each item equal to the one before
/// it.
///
/// The list is re-read at every step: a user comparator may edit it. Its
/// two values are copied out before the call, so no borrow of the list is
/// live while the function runs.
pub(crate) fn do_uniq(list: &ListRef, info: &mut SortInfo<'_>) {
    info.func_err = false;
    let builtin = info.is_builtin();

    let mut at = 1;
    while at < list.len() {
        let res = if builtin {
            let items = list.items();
            compare_builtin(info, &items[at - 1].li_tv, &items[at].li_tv)
        } else if info.func_err {
            0
        } else {
            let argv = [
                list.items()[at - 1].li_tv.clone(),
                list.items()[at].li_tv.clone(),
            ];
            compare_user(info, argv)
        };
        if res == 0 {
            list.remove_at(at);
        } else {
            at += 1;
        }
        if info.func_err {
            emsg(gettext(c"E882: Uniq compare function failed"));
            break;
        }
    }
}

/// Read `sort()`/`uniq()`'s optional `{how}` and `{dict}` arguments into
/// `info`.
///
/// A `{how}` given as a Number has no string of its own, so the caller lends
/// `how` for it: `info` may borrow it, as it borrows the arguments.
pub(crate) fn parse_sort_uniq_args<'a>(
    args: &'a [TypVal],
    info: &mut SortInfo<'a>,
    how: &'a mut NumBuf,
) -> Result<(), Failed> {
    *info = SortInfo::new();

    let Some(arg1) = args.get(1) else {
        return Ok(());
    };

    // optional second argument: {func}
    if arg1.v_type() == VAR_FUNC {
        info.func = arg1.func_name().map(ThinCString::as_cstr);
    } else if arg1.v_type() == VAR_PARTIAL {
        info.partial = match arg1 {
            TypVal::Partial(partial) => (**partial).as_ref(),
            _ => None,
        };
        if info.partial.is_some() {
            // The name the call is made under; the empty name for a
            // partial that has none.
            info.func = Some(arg1.callable_name().unwrap_or(c""));
        }
    } else {
        let Ok(nr) = tv_get_number_chk(arg1) else {
            return Err(Failed); // type error; errmsg already given
        };
        let nr = nr as c_int;
        if nr == 1 {
            info.ic = true;
        } else if arg1.v_type() != VAR_NUMBER {
            info.func = Some(how.string(arg1));
        } else if nr != 0 {
            emsg(gettext(e_invarg));
            return Err(Failed);
        }

        if let Some(name) = info.func {
            match *name.to_bytes() {
                // empty string means default sort
                [] => info.func = None,
                // The five built-in orderings are one-character names;
                // upstream spells each as a `strcmp` against a literal.
                [first] => {
                    let mut builtin = true;
                    match first {
                        b'n' => info.numeric = true,
                        b'N' => info.numbers = true,
                        b'f' => info.float = true,
                        b'i' => info.ic = true,
                        b'l' => info.lc = true,
                        _ => builtin = false,
                    }
                    if builtin {
                        info.func = None;
                    }
                }
                _ => {}
            }
        }
    }

    if args.len() > 2 {
        // optional third argument: {dict}
        tv_check_for_dict_arg(args, 2)?;
        info.selfdict = match &args[2] {
            TypVal::Dict(dict) => (**dict).as_ref(),
            _ => None,
        };
    }

    Ok(())
}

/// The body `sort()` and `uniq()` share: check the argument, read the
/// ordering, and run the driver.
///
/// The ordering is this call's local: a user comparison function can itself
/// call `sort()`, which gets its own.
pub(crate) fn do_sort_uniq(args: &[TypVal], result: &mut TypVal, sort: bool) {
    let mut how = NumBuf::new();
    let first = &args[0];
    if first.v_type() != VAR_LIST {
        let name = if sort { "sort()" } else { "uniq()" };
        semsg!("E686: Argument of {name} must be a List");
        return;
    }

    let mut info = SortInfo::new();

    let arg_errmsg = if sort {
        c"sort() argument"
    } else {
        c"uniq() argument"
    };
    if !value_check_lock(
        list_locked(first.list_ref()),
        LockName::Translate(arg_errmsg),
    ) {
        // The answer is the argument's own list, with a reference of its
        // own.
        result.write_list(first.list_handle());
        if let Some(list) = first.list_shared()
            && list.len() > 1
            && parse_sort_uniq_args(args, &mut info, &mut how).is_ok()
        {
            if sort {
                do_sort(list, &mut info);
            } else {
                do_uniq(list, &mut info);
            }
        }
    }
}

/// `sort()`.
pub fn f_sort(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    do_sort_uniq(args, result, true);
}

/// `uniq()`.
pub fn f_uniq(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    do_sort_uniq(args, result, false);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::list::string_tv;
    use crate::global_cell::editor_state_lock;
    use crate::types::EvalFuncData;

    use V::{F, N, Str};

    /// One item of a test list, as the case wrote it.
    #[derive(Clone, Debug, PartialEq)]
    enum V {
        N(VarNumber),
        Str(String),
        F(f64),
    }

    /// A string item.
    #[allow(non_snake_case)]
    fn S(text: &str) -> V {
        Str(text.to_owned())
    }

    impl V {
        fn tv(&self) -> TypVal {
            match self {
                N(n) => TypVal::Number(*n),
                Str(s) => string_tv(s.as_bytes()),
                F(f) => TypVal::Float(*f),
            }
        }

        fn of(tv: &TypVal) -> V {
            match tv {
                TypVal::Number(n) => N(*n),
                TypVal::Float(f) => F(*f),
                _ => {
                    let text = tv.string_ref().map(|s| s.to_bytes().to_vec());
                    Str(String::from_utf8(text.unwrap_or_default()).expect("ASCII"))
                }
            }
        }
    }

    fn list_of(items: &[V]) -> ListRef {
        let mut l = tv_list_alloc(ptrdiff_t::try_from(items.len()).expect("short"));
        for v in items {
            let mut tv = v.tv();
            l.push_copy(&tv);
            tv_clear(&mut tv);
        }
        l
    }

    fn contents(l: &ListRef) -> Vec<V> {
        l.items().iter().map(|li| V::of(&li.li_tv)).collect()
    }

    /// `sort(l, how)` or `uniq(l, how)` in place, through the builtin.
    fn run(l: &ListRef, how: Option<V>, sort: bool) {
        let mut argv = vec![TypVal::list(Some(l.clone()))];
        if let Some(how) = how {
            argv.push(how.tv());
        }
        let mut result = TypVal::Number(0);
        if sort {
            f_sort(&argv, &mut result, EvalFuncData::None);
        } else {
            f_uniq(&argv, &mut result, EvalFuncData::None);
        }
        assert_eq!(
            result.list_or_null(),
            l.as_ptr(),
            "the answer is the list itself"
        );
        tv_clear(&mut result);
        for tv in &mut argv {
            tv_clear(tv);
        }
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "UB in typval_encode/walk.rs:573 (the default ordering's string() forms): \
                  encode_typval_read retags a &TypVal as &mut"
    )]
    fn sorting_numbers_by_the_default_and_the_numeric_orders() {
        let _serial = editor_state_lock();
        let l = list_of(&[N(10), N(9), N(-2), N(100)]);
        // The default compares string forms.
        run(&l, None, true);
        assert_eq!(contents(&l), [N(-2), N(10), N(100), N(9)]);
        run(&l, Some(S("N")), true);
        assert_eq!(contents(&l), [N(-2), N(9), N(10), N(100)]);
        let l = list_of(&[S("10"), S("9"), N(3), S("x")]);
        run(&l, Some(S("N")), true);
        assert_eq!(contents(&l), [S("x"), N(3), S("9"), S("10")]);
        // An `n` sort orders numbers and floats; every string is `'`.
        run(&l, Some(S("n")), true);
        assert_eq!(contents(&l), [S("x"), S("9"), S("10"), N(3)]);
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "UB in typval_encode/walk.rs:573 (the default ordering's string() forms): \
                  encode_typval_read retags a &TypVal as &mut"
    )]
    fn sorting_floats_and_ignoring_case() {
        let _serial = editor_state_lock();
        let l = list_of(&[F(2.5), N(1), F(-0.5), F(1e10)]);
        run(&l, Some(S("f")), true);
        assert_eq!(contents(&l), [F(-0.5), N(1), F(2.5), F(1e10)]);
        let l = list_of(&[S("b"), S("A"), S("a"), S("B")]);
        run(&l, Some(S("i")), true);
        assert_eq!(contents(&l), [S("A"), S("a"), S("b"), S("B")]);
        run(&l, Some(N(1)), true);
        assert_eq!(contents(&l), [S("A"), S("a"), S("b"), S("B")]);
        run(&l, None, true);
        assert_eq!(contents(&l), [S("A"), S("B"), S("a"), S("b")]);
    }

    /// A `:for` cursor on a list being sorted stands on an *item*: it goes
    /// where the item went.
    #[test]
    fn a_watcher_follows_its_item_through_a_sort() {
        let _serial = editor_state_lock();
        let mut l = list_of(&[N(3), N(0), N(2), N(1)]);
        let two = l.watch_add();
        l.set_watch_index(two, 2);
        let three = l.watch_add();
        l.set_watch_index(three, 0);
        run(&l, Some(S("N")), true);
        assert_eq!(contents(&l), [N(0), N(1), N(2), N(3)]);
        assert_eq!((l.watch_index(two), l.watch_index(three)), (2, 3));
        // An ended cursor stays ended.
        l.set_watch_index(three, ListWatch::ENDED);
        run(&l, Some(S("N")), true);
        assert_eq!(l.watch_index(three), ListWatch::ENDED);
        l.watch_remove(two);
        l.watch_remove(three);
    }

    #[test]
    fn uniq_drops_each_item_equal_to_the_one_before() {
        let _serial = editor_state_lock();
        let l = list_of(&[N(1), N(1), N(2), N(1), N(1), N(3), N(3)]);
        run(&l, Some(S("N")), false);
        assert_eq!(contents(&l), [N(1), N(2), N(1), N(3)]);
        // Strings by their bytes.
        let l = list_of(&[S("a"), S("a"), S("A"), S("b"), S("b")]);
        run(&l, None, false);
        assert_eq!(contents(&l), [S("a"), S("A"), S("b")]);
        // By number, 1 and '1' and '01' are one.  (A Float here would be
        // E805, which needs an execution stack to report against.)
        let l = list_of(&[N(1), S("1"), S("1"), S("01")]);
        run(&l, Some(S("N")), false);
        assert_eq!(contents(&l), [N(1)]);
        let l = list_of(&[F(1.0), N(1), F(1.5)]);
        run(&l, Some(S("f")), false);
        assert_eq!(contents(&l), [F(1.0), F(1.5)]);
    }

    /// The default ordering compares `string()` forms, so a number and the
    /// string of its digits differ: one is quoted.
    ///
    /// That comparison is `encode_tv2string(&TypVal)`, whose walk
    /// (`encode_typval_read`) casts the shared borrow to `*mut` and hands
    /// each hook `tv.as_mut()` -- a `&mut` retag of a `SharedReadOnly`
    /// pointer, which Stacked Borrows rejects before anything is written.
    /// Every `string()` of a non-string value through that entry point
    /// does the same.
    #[test]
    #[cfg_attr(
        miri,
        ignore = "UB in typval_encode/walk.rs:573: encode_typval_read retags a &TypVal \
                  as &mut (Stacked Borrows: Unique retag of a SharedReadOnly tag)"
    )]
    fn uniq_by_string_form_tells_a_number_from_its_digits() {
        let _serial = editor_state_lock();
        let l = list_of(&[N(1), N(1), S("1"), S("1"), S("01")]);
        run(&l, None, false);
        assert_eq!(contents(&l), [N(1), S("1"), S("01")]);
    }

    #[test]
    fn uniq_ignoring_case() {
        let _serial = editor_state_lock();
        let l = list_of(&[S("a"), S("A"), S("b"), S("B"), S("a")]);
        run(&l, Some(S("i")), false);
        assert_eq!(contents(&l), [S("a"), S("b"), S("a")]);
    }

    /// A watcher on an item `uniq` removes lands on the next one, as for any
    /// removal.
    #[test]
    fn a_watcher_on_a_duplicate_lands_on_what_followed_it() {
        let _serial = editor_state_lock();
        let mut l = list_of(&[N(1), N(1), N(2), N(2), N(3)]);
        let w = l.watch_add();
        l.set_watch_index(w, 3);
        run(&l, Some(S("N")), false);
        assert_eq!(contents(&l), [N(1), N(2), N(3)]);
        assert_eq!(l.watch_index(w), 2);
        l.watch_remove(w);
    }

    /// A list of one, or none, is answered without parsing `{how}`: even a
    /// nonsense ordering is no error then.
    #[test]
    fn a_short_list_is_answered_untouched() {
        let _serial = editor_state_lock();
        for items in [vec![], vec![N(5)]] {
            let l = list_of(&items);
            run(&l, Some(S("no-such-function")), true);
            run(&l, Some(S("no-such-function")), false);
            assert_eq!(contents(&l), items);
        }
    }
}
