//! The `linematch:` post-pass.
//!
//! The diff engine answers in whole blocks; with `'diffopt'`'s `linematch:N`
//! each block short enough is handed to `linematch_nbuffers`, which re-aligns
//! the individual lines inside it, and `apply_linematch_results` replaces the
//! one block with the several the alignment implies.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn diff_linematch(mut dp: *mut diff_T) -> bool {
    unsafe {
        if diff_flags.get() & DIFF_LINEMATCH == 0 {
            return false_0 != 0;
        }
        let mut tsize: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < DB_COUNT {
            if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
                if (*dp).df_count[i as usize] < 0 as linenr_T {
                    return false_0 != 0;
                }
                tsize += (*dp).df_count[i as usize] as ::core::ffi::c_int;
            }
            i += 1;
        }
        return tsize <= linematch_lines.get();
    }
}

unsafe extern "C" fn apply_linematch_results(
    mut dp: *mut diff_T,
    mut decisions_length: size_t,
    mut decisions: *const ::core::ffi::c_int,
) {
    unsafe {
        let mut line_numbers: [::core::ffi::c_int; 8] = [0; 8];
        let mut outputmap: [::core::ffi::c_int; 8] = [0; 8];
        let mut ndiffs: size_t = 0 as size_t;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < DB_COUNT {
            if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
                line_numbers[i as usize] = (*dp).df_lnum[i as usize] as ::core::ffi::c_int;
                (*dp).df_count[i as usize] = 0 as ::core::ffi::c_int as linenr_T;
                outputmap[ndiffs as usize] = i;
                ndiffs = ndiffs.wrapping_add(1);
            }
            i += 1;
        }
        let mut dp_s: *mut diff_T = dp;
        let mut i_0: size_t = 0 as size_t;
        while i_0 < decisions_length {
            if i_0 != 0 as size_t
                && *decisions.offset(i_0.wrapping_sub(1 as size_t) as isize)
                    != *decisions.offset(i_0 as isize)
            {
                dp_s = diff_alloc_new(curtab.get(), dp_s, (*dp_s).df_next);
                (*dp_s).is_linematched = true_0 != 0;
                let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while j < DB_COUNT {
                    if !(*curtab.get()).tp_diffbuf[j as usize].is_null() {
                        (*dp_s).df_lnum[j as usize] = line_numbers[j as usize] as linenr_T;
                        (*dp_s).df_count[j as usize] = 0 as ::core::ffi::c_int as linenr_T;
                    }
                    j += 1;
                }
            }
            let mut j_0: size_t = 0 as size_t;
            while j_0 < ndiffs {
                if *decisions.offset(i_0 as isize) & (1 as ::core::ffi::c_int) << j_0 != 0 {
                    (*dp_s).df_count[outputmap[j_0 as usize] as usize] += 1;
                    line_numbers[outputmap[j_0 as usize] as usize] += 1;
                }
                j_0 = j_0.wrapping_add(1);
            }
            i_0 = i_0.wrapping_add(1);
        }
        (*dp).is_linematched = true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn run_linematch_algorithm(mut dp: *mut diff_T) {
    unsafe {
        let mut diffbufs_mm: [mmfile_t; 8] = [mmfile_t {
            ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        }; 8];
        let mut diff_length: [::core::ffi::c_int; 8] = [0; 8];
        let mut ndiffs: size_t = 0 as size_t;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < DB_COUNT {
            if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
                if (*dp).df_count[i as usize] > 0 as linenr_T {
                    diff_write_buffer(
                        (*curtab.get()).tp_diffbuf[i as usize] as *mut buf_T,
                        (&raw mut diffbufs_mm as *mut mmfile_t).offset(ndiffs as isize),
                        (*dp).df_lnum[i as usize],
                        (*dp).df_lnum[i as usize] + (*dp).df_count[i as usize] - 1 as linenr_T,
                    );
                } else {
                    diffbufs_mm[ndiffs as usize].size = 0 as ::core::ffi::c_int;
                    diffbufs_mm[ndiffs as usize].ptr =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                diff_length[ndiffs as usize] = (*dp).df_count[i as usize] as ::core::ffi::c_int;
                ndiffs = ndiffs.wrapping_add(1);
            }
            i += 1;
        }
        let iwhite: bool =
            diff_flags.get() & (DIFF_IWHITEALL | DIFF_IWHITE) > 0 as ::core::ffi::c_int;
        // `diff_write_buffer` leaves an empty block as a NULL pointer, which
        // `from_raw_parts` may not see; those axes have zero length anyway.
        let mut blocks: [&[u8]; 8] = [&[]; 8];
        for (block, mm) in blocks.iter_mut().zip(&diffbufs_mm[..ndiffs]) {
            if !mm.ptr.is_null() {
                *block = ::core::slice::from_raw_parts(mm.ptr as *const u8, mm.size as usize);
            }
        }
        let decisions = linematch_nbuffers(&blocks[..ndiffs], &diff_length[..ndiffs], iwhite);
        let mut i_0: size_t = 0 as size_t;
        while i_0 < ndiffs {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*(&raw mut diffbufs_mm as *mut mmfile_t).offset(i_0 as isize)).ptr
                    as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
            i_0 = i_0.wrapping_add(1);
        }
        apply_linematch_results(dp, decisions.len(), decisions.as_ptr());
    }
}
