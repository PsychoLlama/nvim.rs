//! Growable array of items (`garray_T`): safe core + C-ABI shims.
//!
//! The struct layout is frozen: call sites all over the crate (and the unit
//! suite, via FFI) poke the fields directly and `xfree` the data pointer.
//! Every heap byte stays on the `xmalloc` family so the unit suite's
//! allocator seam observes the same allocation sequence as before. The
//! `extern "C"` shims keep the raw-pointer plumbing; the growth policy and
//! joining logic live in safe code below them.

use core::ffi::{c_char, c_int, c_void, CStr};
use core::ptr;
use core::slice;

use crate::src::nvim::log::logmsg;
use crate::src::nvim::memory::{xfree, xmallocz, xrealloc, xstrdup};
use crate::src::nvim::path::path_fnamecmp;
use crate::src::nvim::strings::sort_strings;

pub use crate::src::nvim::types::garray::garray_T;

const LOGLVL_WRN: c_int = 3;

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
    let itemsize = ga.ga_itemsize as usize;
    Some(GrowPlan {
        new_maxlen,
        old_size: itemsize.wrapping_mul(ga.ga_maxlen as usize),
        new_size: itemsize.wrapping_mul(new_maxlen as usize),
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

#[no_mangle]
pub unsafe extern "C" fn ga_clear(gap: *mut garray_T) {
    xfree((*gap).ga_data);
    (*gap).ga_data = ptr::null_mut();
    (*gap).ga_maxlen = 0;
    (*gap).ga_len = 0;
}

pub unsafe extern "C" fn ga_clear_strings(gap: *mut garray_T) {
    if !(*gap).ga_data.is_null() {
        let items =
            slice::from_raw_parts((*gap).ga_data as *const *mut c_void, (*gap).ga_len as usize);
        for &item in items {
            xfree(item);
        }
    }
    ga_clear(gap);
}

#[no_mangle]
pub unsafe extern "C" fn ga_init(gap: *mut garray_T, itemsize: c_int, growsize: c_int) {
    (*gap).ga_data = ptr::null_mut();
    (*gap).ga_maxlen = 0;
    (*gap).ga_len = 0;
    (*gap).ga_itemsize = itemsize;
    ga_set_growsize(gap, growsize);
}

pub unsafe extern "C" fn ga_set_growsize(gap: *mut garray_T, growsize: c_int) {
    if growsize < 1 {
        logmsg(
            LOGLVL_WRN,
            ptr::null(),
            b"ga_set_growsize\0".as_ptr() as *const c_char,
            57,
            true,
            b"trying to set an invalid ga_growsize: %d\0".as_ptr() as *const c_char,
            growsize,
        );
        (*gap).ga_growsize = 1;
    } else {
        (*gap).ga_growsize = growsize;
    }
}

pub unsafe extern "C" fn ga_grow(gap: *mut garray_T, n: c_int) {
    let Some(plan) = grow_plan(&*gap, n) else {
        return;
    };
    if (*gap).ga_growsize < 1 {
        logmsg(
            LOGLVL_WRN,
            ptr::null(),
            b"ga_grow\0".as_ptr() as *const c_char,
            76,
            true,
            b"ga_growsize(%d) is less than 1\0".as_ptr() as *const c_char,
            (*gap).ga_growsize,
        );
    }
    let data = xrealloc((*gap).ga_data, plan.new_size) as *mut u8;
    slice::from_raw_parts_mut(
        data.add(plan.old_size),
        plan.new_size.wrapping_sub(plan.old_size),
    )
    .fill(0);
    (*gap).ga_maxlen = plan.new_maxlen;
    (*gap).ga_data = data as *mut c_void;
}

pub unsafe extern "C" fn ga_remove_duplicate_strings(gap: *mut garray_T) {
    let fnames = (*gap).ga_data as *mut *mut c_char;
    sort_strings(fnames, (*gap).ga_len);
    let mut i = (*gap).ga_len - 1;
    while i > 0 {
        let names = slice::from_raw_parts_mut(fnames, (*gap).ga_len as usize);
        let (prev, cur) = (i as usize - 1, i as usize);
        if path_fnamecmp(names[prev], names[cur]) == 0 {
            xfree(names[cur] as *mut c_void);
            names.copy_within(cur + 1.., cur);
            (*gap).ga_len -= 1;
        }
        i -= 1;
    }
}

pub unsafe extern "C" fn ga_concat_strings(
    gap: *const garray_T,
    sep: *const c_char,
) -> *mut c_char {
    if (*gap).ga_len == 0 {
        return xstrdup(b"\0".as_ptr() as *const c_char);
    }
    let strings = slice::from_raw_parts(
        (*gap).ga_data as *const *const c_char,
        (*gap).ga_len as usize,
    );
    let parts: Vec<&[u8]> = strings
        .iter()
        .map(|&s| CStr::from_ptr(s).to_bytes())
        .collect();
    let sep = CStr::from_ptr(sep).to_bytes();
    let len = joined_len(&parts, sep.len());
    let ret = xmallocz(len) as *mut u8;
    join_into(slice::from_raw_parts_mut(ret, len), &parts, sep);
    ret as *mut c_char
}

pub unsafe extern "C" fn ga_concat(gap: *mut garray_T, s: *const c_char) {
    if s.is_null() {
        return;
    }
    ga_concat_len(gap, s, CStr::from_ptr(s).to_bytes().len());
}

pub unsafe extern "C" fn ga_concat_len(gap: *mut garray_T, s: *const c_char, len: usize) {
    if len == 0 {
        return;
    }
    ga_grow(gap, len as c_int);
    let src = slice::from_raw_parts(s as *const u8, len);
    let dst =
        slice::from_raw_parts_mut(((*gap).ga_data as *mut u8).add((*gap).ga_len as usize), len);
    dst.copy_from_slice(src);
    (*gap).ga_len += len as c_int;
}

pub unsafe extern "C" fn ga_append(gap: *mut garray_T, c: u8) {
    ga_grow(gap, 1);
    *((*gap).ga_data as *mut u8).add((*gap).ga_len as usize) = c;
    (*gap).ga_len += 1;
}

pub unsafe extern "C" fn ga_append_via_ptr(gap: *mut garray_T, item_size: usize) -> *mut c_void {
    if item_size as c_int != (*gap).ga_itemsize {
        logmsg(
            LOGLVL_WRN,
            ptr::null(),
            b"ga_append_via_ptr\0".as_ptr() as *const c_char,
            209,
            true,
            b"wrong item size (%zu), should be %d\0".as_ptr() as *const c_char,
            item_size,
            (*gap).ga_itemsize,
        );
    }
    ga_grow(gap, 1);
    let idx = (*gap).ga_len;
    (*gap).ga_len += 1;
    ((*gap).ga_data as *mut u8).add(item_size.wrapping_mul(idx as usize)) as *mut c_void
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
