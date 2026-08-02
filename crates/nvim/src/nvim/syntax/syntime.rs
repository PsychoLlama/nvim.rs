//! `:syntime` — per-pattern timing.
//!
//! With timing on, every `syn_regexec` accumulates into the pattern's own
//! `syn_time_T`; [`syntime_report`] sorts the patterns by total time and prints
//! the table. Used to find the pattern that makes a syntax file slow.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe fn ex_syntime(mut eap: *mut exarg_T) {
    unsafe {
        if strcmp((*eap).arg, b"on\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            syn_time_on.set(true_0 != 0);
        } else if strcmp((*eap).arg, b"off\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            syn_time_on.set(false_0 != 0);
        } else if strcmp(
            (*eap).arg,
            b"clear\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            syntime_clear();
        } else if strcmp(
            (*eap).arg,
            b"report\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            syntime_report();
        } else {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                (*eap).arg,
            );
        };
    }
}

pub(crate) unsafe extern "C" fn syn_clear_time(mut st: *mut syn_time_T) {
    unsafe {
        (*st).total = profile_zero();
        (*st).slowest = profile_zero();
        (*st).count = 0 as ::core::ffi::c_int;
        (*st).match_0 = 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn syntime_clear() {
    unsafe {
        let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
        if !syntax_present(curwin.get()) {
            msg(
                gettext(msg_no_items.ptr() as *mut ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
            );
            return;
        }
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while idx < (*(*curwin.get()).w_s).b_syn_patterns.ga_len {
            spp = ((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                .offset(idx as isize);
            syn_clear_time(&raw mut (*spp).sp_time);
            idx += 1;
        }
    }
}

pub unsafe extern "C" fn get_syntime_arg(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    match idx {
        0 => {
            return b"on\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        1 => {
            return b"off\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        2 => {
            return b"clear\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        3 => {
            return b"report\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        _ => {}
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}

pub(crate) unsafe extern "C" fn syn_compare_syntime(
    mut v1: *const ::core::ffi::c_void,
    mut v2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let mut s1: *const time_entry_T = v1 as *const time_entry_T;
        let mut s2: *const time_entry_T = v2 as *const time_entry_T;
        return profile_cmp((*s1).total, (*s2).total);
    }
}

pub(crate) unsafe extern "C" fn syntime_report() {
    unsafe {
        if !syntax_present(curwin.get()) {
            msg(
                gettext(msg_no_items.ptr() as *mut ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
            );
            return;
        }
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<time_entry_T>() as ::core::ffi::c_int,
            50 as ::core::ffi::c_int,
        );
        let mut total_total: proftime_T = profile_zero();
        let mut total_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut p: *mut time_entry_T = ::core::ptr::null_mut::<time_entry_T>();
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while idx < (*(*curwin.get()).w_s).b_syn_patterns.ga_len {
            let mut spp: *mut synpat_T = ((*(*curwin.get()).w_s).b_syn_patterns.ga_data
                as *mut synpat_T)
                .offset(idx as isize);
            if (*spp).sp_time.count > 0 as ::core::ffi::c_int {
                p = ga_append_via_ptr(&raw mut ga, ::core::mem::size_of::<time_entry_T>())
                    as *mut time_entry_T;
                (*p).total = (*spp).sp_time.total;
                total_total = profile_add(total_total, (*spp).sp_time.total);
                (*p).count = (*spp).sp_time.count;
                (*p).match_0 = (*spp).sp_time.match_0;
                total_count += (*spp).sp_time.count;
                (*p).slowest = (*spp).sp_time.slowest;
                let mut tm: proftime_T = profile_divide((*spp).sp_time.total, (*spp).sp_time.count);
                (*p).average = tm;
                (*p).id = (*spp).sp_syn.id as ::core::ffi::c_int;
                (*p).pattern = (*spp).sp_pattern;
            }
            idx += 1;
        }
        if ga.ga_len > 1 as ::core::ffi::c_int {
            qsort(
                ga.ga_data,
                ga.ga_len as size_t,
                ::core::mem::size_of::<time_entry_T>(),
                Some(
                    syn_compare_syntime
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                ),
            );
        }
        msg_puts_title(gettext(
            b"  TOTAL      COUNT  MATCH   SLOWEST     AVERAGE   NAME               PATTERN\0"
                .as_ptr() as *const ::core::ffi::c_char,
        ));
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        let mut idx_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while idx_0 < ga.ga_len && !got_int.get() {
            p = (ga.ga_data as *mut time_entry_T).offset(idx_0 as isize);
            msg_puts(profile_msg((*p).total));
            msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
            msg_advance(13 as ::core::ffi::c_int);
            msg_outnum((*p).count);
            msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
            msg_advance(20 as ::core::ffi::c_int);
            msg_outnum((*p).match_0);
            msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
            msg_advance(26 as ::core::ffi::c_int);
            msg_puts(profile_msg((*p).slowest));
            msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
            msg_advance(38 as ::core::ffi::c_int);
            msg_puts(profile_msg((*p).average));
            msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
            msg_advance(50 as ::core::ffi::c_int);
            msg_outtrans(
                highlight_group_name((*p).id - 1 as ::core::ffi::c_int),
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
            msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
            msg_advance(69 as ::core::ffi::c_int);
            let mut len: ::core::ffi::c_int = 0;
            if Columns.get() < 80 as ::core::ffi::c_int {
                len = 20 as ::core::ffi::c_int;
            } else {
                len = Columns.get() - 70 as ::core::ffi::c_int;
            }
            let mut patlen: ::core::ffi::c_int = strlen((*p).pattern) as ::core::ffi::c_int;
            len = if len < patlen { len } else { patlen };
            msg_outtrans_len((*p).pattern, len, 0 as ::core::ffi::c_int, false_0 != 0);
            msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
            idx_0 += 1;
        }
        ga_clear(&raw mut ga);
        if !got_int.get() {
            msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
            msg_puts(profile_msg(total_total));
            msg_advance(13 as ::core::ffi::c_int);
            msg_outnum(total_count);
            msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        }
    }
}
