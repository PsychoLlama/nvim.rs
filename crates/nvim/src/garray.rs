//! Growable array of items (`garray_T`): safe core + raw-pointer shims.
//!
//! The struct layout is frozen: call sites all over the crate (and the unit
//! suite, via FFI) poke the fields directly and `xfree` the data pointer.
//! Every heap byte stays on the `xmalloc` family so the unit suite's
//! allocator seam observes the same allocation sequence as before. The
//! `unsafe fn` shims keep the raw-pointer plumbing; the growth policy and
//! joining logic live in safe code below them. Only `ga_clear`/`ga_init`
//! still carry the C ABI, and only because the unit suite calls them.
//!
//! # Boundary
//!
//! Every entry point takes `*mut garray_T` because its callers hold one --
//! a field of `buf_T`, a `static`, a local the transpile never borrowed.
//! Each shim turns that pointer into a `&mut garray_T` once, at the top, and
//! the rest of the body is ordinary Rust. The *contract* the callers rely on
//! is unchanged and load-bearing: many of them (`FoldList` and friends)
//! re-read `ga_len` on every step of a loop that also grows the array, so
//! growth must stay in place through `xrealloc` and must keep zeroing the
//! tail it adds.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::{ptr, slice};

use crate::log::{LOGLVL_WRN, logmsg};
use crate::memory::{xfree, xmallocz, xrealloc, xstrdup};
use crate::path::path_fnamecmp;
use crate::strings::sort_strings;

use crate::types::garray::garray_T;

/// A count of items as a byte count: `garray_T` keeps both its lengths and
/// its item size in a `c_int`, and every one of the three is a size the
/// array itself set.
fn as_size(n: c_int) -> usize {
    usize::try_from(n).expect("a garray length is never negative")
}

/// A reallocation the growth policy decided on: realloc `ga_data` to
/// `new_size` bytes and zero the tail starting at `old_size`.
struct GrowPlan {
    new_maxlen: c_int,
    old_size: usize,
    new_size: usize,
}

/// The C growth policy, verbatim: nothing to do while `n` more items fit;
/// otherwise grow by at least `ga_growsize` items and at least half the
/// current length.
fn grow_plan(ga: &garray_T, n: c_int) -> Option<GrowPlan> {
    if ga.ga_maxlen - ga.ga_len >= n {
        return None;
    }
    let n = n.max(ga.ga_growsize).max(ga.ga_len / 2);
    let new_maxlen = ga.ga_len + n;
    let itemsize = as_size(ga.ga_itemsize);
    Some(GrowPlan {
        new_maxlen,
        old_size: itemsize.wrapping_mul(as_size(ga.ga_maxlen)),
        new_size: itemsize.wrapping_mul(as_size(new_maxlen)),
    })
}

/// Length of `parts` joined by a `sep_len`-byte separator. `parts` must be
/// non-empty (the empty case never reaches the join).
fn joined_len(parts: &[&[u8]], sep_len: usize) -> usize {
    let payload: usize = parts.iter().map(|p| p.len()).sum();
    payload.wrapping_add((parts.len() - 1).wrapping_mul(sep_len))
}

/// Write `parts` joined by `sep` into `dst`, which is exactly
/// `joined_len(parts, sep.len())` bytes.
fn join_into(dst: &mut [u8], parts: &[&[u8]], sep: &[u8]) {
    let mut off = 0;
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            dst[off..off + sep.len()].copy_from_slice(sep);
            off += sep.len();
        }
        dst[off..off + part.len()].copy_from_slice(part);
        off += part.len();
    }
}

/// The array's items, as a slice.
///
/// # Safety
///
/// The array's items really are `T`s, and `ga_data` really points at
/// `ga_len` of them. An array that never grew has a null `ga_data` and must
/// not reach here with a nonzero `ga_len`.
unsafe fn items<T>(ga: &garray_T) -> &[T] {
    // SAFETY: the caller's promise. `ga_data` is null only for an untouched
    // array, whose `ga_len` is 0 -- and `from_raw_parts` rejects a null base
    // even then, which is why the callers test it first.
    unsafe { slice::from_raw_parts(ga.ga_data.cast::<T>(), as_size(ga.ga_len)) }
}

/// Where item number `ga_len` starts, given `item_size` bytes per item: the
/// slot an append writes before bumping the length.
///
/// # Safety
///
/// The array has room for that item -- i.e. [`ga_grow`] has just run.
unsafe fn tail(ga: &garray_T, item_size: usize) -> *mut u8 {
    // SAFETY: the caller's promise puts the offset inside the allocation.
    unsafe {
        ga.ga_data
            .cast::<u8>()
            .add(item_size.wrapping_mul(as_size(ga.ga_len)))
    }
}

/// A C string's bytes, terminator excluded.
///
/// # Safety
///
/// `s` is NUL-terminated and stays valid for as long as the result is used.
unsafe fn cbytes<'a>(s: *const c_char) -> &'a [u8] {
    // SAFETY: the caller's promise.
    unsafe { CStr::from_ptr(s) }.to_bytes()
}

/// Release the array's storage and reset it to empty. The items themselves
/// are the caller's (see [`ga_clear_strings`] for the `char *` case).
///
/// # Safety
///
/// `gap` points to a live `garray_T` whose `ga_data` is null or an
/// `xmalloc`-family allocation this call takes over.
pub unsafe fn ga_clear(gap: *mut garray_T) {
    // SAFETY: the caller's array, and its own data allocation.
    let ga = unsafe { &mut *gap };
    unsafe { xfree(ga.ga_data) };
    ga.ga_data = ptr::null_mut();
    ga.ga_maxlen = 0;
    ga.ga_len = 0;
}

/// [`ga_clear`] for an array of owned `char *`: free every string first.
///
/// # Safety
///
/// `gap` points to a live `garray_T` of `ga_len` owned C strings.
pub unsafe fn ga_clear_strings(gap: *mut garray_T) {
    // SAFETY: the caller's array; `ga_data` holds `ga_len` owned pointers,
    // and is null only when the array never grew.
    let ga = unsafe { &*gap };
    if !ga.ga_data.is_null() {
        let strings: &[*mut c_void] = unsafe { items(ga) };
        strings.iter().for_each(|&s| unsafe { xfree(s) });
    }
    unsafe { ga_clear(gap) };
}

/// Set up an empty array of `itemsize`-byte items.
///
/// # Safety
///
/// `gap` points to writable, possibly uninitialized `garray_T` storage. Any
/// allocation it already held is leaked, as upstream's is.
pub unsafe fn ga_init(gap: *mut garray_T, itemsize: c_int, growsize: c_int) {
    // SAFETY: the caller's storage, written before anything reads it.
    let ga = unsafe { &mut *gap };
    ga.ga_data = ptr::null_mut();
    ga.ga_maxlen = 0;
    ga.ga_len = 0;
    ga.ga_itemsize = itemsize;
    set_growsize(ga, growsize);
}

/// How many items each growth step adds, at minimum. A non-positive value is
/// a caller bug; it is logged and clamped, as upstream does.
fn set_growsize(ga: &mut garray_T, growsize: c_int) {
    if growsize < 1 {
        logmsg!(
            LOGLVL_WRN,
            c"ga_set_growsize",
            57,
            "trying to set an invalid ga_growsize: {growsize}"
        );
        ga.ga_growsize = 1;
    } else {
        ga.ga_growsize = growsize;
    }
}

/// # Safety
///
/// `gap` points to a live `garray_T`.
pub unsafe fn ga_set_growsize(gap: *mut garray_T, growsize: c_int) {
    // SAFETY: the caller's array.
    set_growsize(unsafe { &mut *gap }, growsize);
}

/// Make room for `n` more items, reallocating in place when the current
/// block is too small. The new tail is zeroed; callers rely on that as much
/// as `xcalloc`'s callers rely on theirs.
///
/// # Safety
///
/// `gap` points to a live `garray_T` whose `ga_data` is null or an
/// `xmalloc`-family allocation of `ga_maxlen * ga_itemsize` bytes.
pub unsafe fn ga_grow(gap: *mut garray_T, n: c_int) {
    // SAFETY: the caller's array.
    let ga = unsafe { &mut *gap };
    let Some(plan) = grow_plan(ga, n) else {
        return;
    };
    if ga.ga_growsize < 1 {
        let growsize = ga.ga_growsize;
        logmsg!(
            LOGLVL_WRN,
            c"ga_grow",
            76,
            "ga_growsize({growsize}) is less than 1"
        );
    }
    let added = plan.new_size.wrapping_sub(plan.old_size);
    // SAFETY: `ga_data` is the array's own allocation of `old_size` bytes,
    // and `xrealloc` answers `new_size` writable bytes or does not return.
    // Only the `added` tail bytes are written, and they held no live item.
    let data = unsafe { xrealloc(ga.ga_data, plan.new_size) }.cast::<u8>();
    unsafe { ptr::write_bytes(data.add(plan.old_size), 0, added) };
    ga.ga_data = data.cast::<c_void>();
    ga.ga_maxlen = plan.new_maxlen;
}

/// Sort an array of owned `char *` and drop the duplicates, freeing them.
///
/// # Safety
///
/// `gap` points to a live `garray_T` of `ga_len` owned C strings.
pub unsafe fn ga_remove_duplicate_strings(gap: *mut garray_T) {
    // SAFETY: the caller's array of owned strings. The walk shrinks `ga_len`
    // as it frees, so the slice is rebuilt on every step -- and it walks
    // downwards, so the shrinking tail is always behind it.
    let ga = unsafe { &mut *gap };
    let fnames = ga.ga_data.cast::<*mut c_char>();
    unsafe { sort_strings(fnames, ga.ga_len) };
    let mut i = as_size(ga.ga_len);
    while i > 1 {
        i -= 1;
        let names = unsafe { slice::from_raw_parts_mut(fnames, as_size(ga.ga_len)) };
        if unsafe { path_fnamecmp(names[i - 1], names[i]) } == 0 {
            unsafe { xfree(names[i].cast::<c_void>()) };
            names.copy_within(i + 1.., i);
            ga.ga_len -= 1;
        }
    }
}

/// The array's `ga_len` C strings joined by `sep`. The caller owns the
/// result, which is empty (but allocated) for an empty array.
///
/// # Safety
///
/// `gap` points to a live `garray_T` of `ga_len` C strings, and `sep` is a
/// NUL-terminated string.
pub unsafe fn ga_concat_strings(gap: *const garray_T, sep: *const c_char) -> *mut c_char {
    // SAFETY: the caller's array and separator, both live for the call, and
    // every item is a NUL-terminated string the array does not own.
    let ga = unsafe { &*gap };
    if ga.ga_len == 0 {
        return unsafe { xstrdup(c"".as_ptr()) };
    }
    let strings: &[*const c_char] = unsafe { items(ga) };
    let parts: Vec<&[u8]> = strings.iter().map(|&s| unsafe { cbytes(s) }).collect();
    let sep = unsafe { cbytes(sep) };
    let len = joined_len(&parts, sep.len());
    // SAFETY: `xmallocz` answers `len + 1` writable bytes or does not
    // return, and the borrowed parts do not point into them.
    let ret = unsafe { xmallocz(len) }.cast::<u8>();
    join_into(unsafe { slice::from_raw_parts_mut(ret, len) }, &parts, sep);
    ret.cast::<c_char>()
}

/// Append a C string's bytes (without its terminator) to a byte array. A
/// null `s` appends nothing.
///
/// # Safety
///
/// `gap` points to a live byte `garray_T`; `s` is null or NUL-terminated.
pub unsafe fn ga_concat(gap: *mut garray_T, s: *const c_char) {
    if s.is_null() {
        return;
    }
    // SAFETY: `s` is NUL-terminated, and `gap` is the caller's array.
    unsafe { ga_concat_len(gap, s, cbytes(s).len()) };
}

/// Append `len` bytes of `s` to a byte array, NUL bytes and all.
///
/// # Safety
///
/// `gap` points to a live byte `garray_T`, and `s` is readable for `len`
/// bytes and does not point into the array's own storage.
pub unsafe fn ga_concat_len(gap: *mut garray_T, s: *const c_char, len: usize) {
    if len == 0 {
        return;
    }
    let n = c_int::try_from(len).expect("a concatenated run fits a garray length");
    // SAFETY: the caller's array, grown to hold `len` more bytes just above,
    // and `s`'s `len` readable bytes, which do not alias it.
    unsafe { ga_grow(gap, n) };
    let ga = unsafe { &mut *gap };
    let dst = unsafe { tail(ga, 1) };
    unsafe { ptr::copy_nonoverlapping(s.cast::<u8>(), dst, len) };
    ga.ga_len += n;
}

/// Append one byte to a byte array.
///
/// # Safety
///
/// `gap` points to a live byte `garray_T`.
pub unsafe fn ga_append(gap: *mut garray_T, c: u8) {
    // SAFETY: the caller's array, grown to hold one more byte just above.
    unsafe { ga_grow(gap, 1) };
    let ga = unsafe { &mut *gap };
    unsafe { *tail(ga, 1) = c };
    ga.ga_len += 1;
}

/// Reserve one more item and hand back a pointer to it, uninitialized (but
/// zeroed, as [`ga_grow`] leaves it). `item_size` is checked against the
/// array's own, which is what makes this a shim rather than a cast.
///
/// # Safety
///
/// `gap` points to a live `garray_T`.
pub unsafe fn ga_append_via_ptr(gap: *mut garray_T, item_size: usize) -> *mut c_void {
    // SAFETY: the caller's array, grown to hold one more item just above.
    let ga = unsafe { &mut *gap };
    if item_size != as_size(ga.ga_itemsize) {
        let want = ga.ga_itemsize;
        logmsg!(
            LOGLVL_WRN,
            c"ga_append_via_ptr",
            209,
            "wrong item size ({item_size}), should be {want}"
        );
    }
    unsafe { ga_grow(gap, 1) };
    let ga = unsafe { &mut *gap };
    let mem = unsafe { tail(ga, item_size) };
    ga.ga_len += 1;
    mem.cast::<c_void>()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ga(len: c_int, maxlen: c_int, itemsize: c_int, growsize: c_int) -> garray_T {
        garray_T {
            ga_len: len,
            ga_maxlen: maxlen,
            ga_itemsize: itemsize,
            ga_growsize: growsize,
            ga_data: ptr::null_mut(),
        }
    }

    #[test]
    fn grow_plan_noop_while_capacity_lasts() {
        assert!(grow_plan(&ga(2, 6, 16, 4), 4).is_none());
        assert!(grow_plan(&ga(0, 0, 16, 4), 0).is_none());
    }

    #[test]
    fn grow_plan_grows_by_growsize_when_request_is_smaller() {
        let plan = grow_plan(&ga(0, 0, 16, 4), 3).unwrap();
        assert_eq!(plan.new_maxlen, 4);
        assert_eq!(plan.old_size, 0);
        assert_eq!(plan.new_size, 64);
    }

    #[test]
    fn grow_plan_grows_by_request_when_larger_than_growsize() {
        let plan = grow_plan(&ga(0, 0, 16, 4), 5).unwrap();
        assert_eq!(plan.new_maxlen, 5);
        assert_eq!(plan.new_size, 80);
    }

    #[test]
    fn grow_plan_grows_by_at_least_half_the_length() {
        let plan = grow_plan(&ga(100, 100, 1, 1), 1).unwrap();
        assert_eq!(plan.new_maxlen, 150);
        assert_eq!(plan.old_size, 100);
        assert_eq!(plan.new_size, 150);
    }

    #[test]
    fn grow_plan_turns_over_at_the_exact_fit() {
        // Room for exactly `n` more items is room enough...
        assert!(grow_plan(&ga(4, 10, 8, 4), 6).is_none());
        // ... one item past it is not, and the growth is the growsize.
        let plan = grow_plan(&ga(4, 10, 8, 4), 7).unwrap();
        assert_eq!(plan.new_maxlen, 11);
        assert_eq!(plan.old_size, 80);
        assert_eq!(plan.new_size, 88);
    }

    #[test]
    fn grow_plan_never_shrinks_the_allocation() {
        // Every plan reallocates upwards: `new_maxlen` is `ga_len + n` with
        // `n` at least the shortfall, so the new block always covers the old
        // items even when `ga_maxlen` was already generous.
        for len in [0, 1, 7, 64] {
            for maxlen in [len, len + 1, len + 33] {
                for n in [1, 2, 100] {
                    let ga = ga(len, maxlen, 4, 3);
                    let Some(plan) = grow_plan(&ga, n) else {
                        continue;
                    };
                    assert!(plan.new_maxlen >= len + n, "{len} {maxlen} {n}");
                    assert!(plan.new_size >= plan.old_size, "{len} {maxlen} {n}");
                }
            }
        }
    }

    #[test]
    fn join_produces_separated_concatenation() {
        let parts: &[&[u8]] = &[b"oh", b"my", b"neovim"];
        let len = joined_len(parts, 1);
        assert_eq!(len, 12);
        let mut dst = vec![0; len];
        join_into(&mut dst, parts, b",");
        assert_eq!(dst, b"oh,my,neovim");
    }

    #[test]
    fn join_of_single_part_has_no_separator() {
        let parts: &[&[u8]] = &[b"solo"];
        let len = joined_len(parts, 3);
        assert_eq!(len, 4);
        let mut dst = vec![0; len];
        join_into(&mut dst, parts, b"---");
        assert_eq!(dst, b"solo");
    }
}
