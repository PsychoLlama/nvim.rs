//! `nvim_exec_autocmds()`: firing an event by hand.
//!
//! It resolves the same (events, pattern, group, buffer) tuple the other
//! entry points do, then calls `apply_autocmds_group` once per event with
//! the caller's `data` published as `v:event` for the duration.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nvim_exec_autocmds(
    mut event: Object,
    mut opts: *mut KeyDict_exec_autocmds,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    unsafe {
        let mut au_group: ::core::ffi::c_int = AUGROUP_ALL as ::core::ffi::c_int;
        let mut modeline: bool = true_0 != 0;
        let mut b: *mut buf_T = curbuf.get();
        let mut data: *mut Object = ::core::ptr::null_mut::<Object>();
        let mut event_array: Array = unpack_string_or_array(
            event,
            c"event".as_ptr() as *mut ::core::ffi::c_char,
            true_0 != 0,
            arena,
            err,
        );
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return;
        }
        let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        match (*opts).group.type_0 as ::core::ffi::c_uint {
            0 => {}
            4 => {
                au_group = augroup_find((*opts).group.data.string.data);
                if !(au_group != AUGROUP_ERROR as ::core::ffi::c_int) {
                    api_err_invalid(
                        err,
                        c"group".as_ptr(),
                        (*opts).group.data.string.data,
                        0 as int64_t,
                        true_0 != 0,
                    );
                    return;
                }
            }
            2 => {
                au_group = (*opts).group.data.integer as ::core::ffi::c_int;
                name = if au_group == 0 as ::core::ffi::c_int {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                } else {
                    augroup_name(au_group)
                };
                if !augroup_exists(name) {
                    api_err_invalid(
                        err,
                        c"group".as_ptr(),
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        au_group as int64_t,
                        false_0 != 0,
                    );
                    return;
                }
            }
            _ => {
                if true {
                    api_err_exp(
                        err,
                        c"group".as_ptr(),
                        c"String or Integer".as_ptr(),
                        api_typename((*opts).group.type_0),
                    );
                    return;
                }
            }
        }
        let mut has_buf: bool = (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_exec_autocmds__buf
            != 0 as ::core::ffi::c_ulonglong
            || (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_exec_autocmds__buffer
                != 0 as ::core::ffi::c_ulonglong;
        let mut buf: Buffer = if (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_exec_autocmds__buf
            != 0 as ::core::ffi::c_ulonglong
        {
            (*opts).buf
        } else {
            (*opts).buffer
        };
        if !(!((*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong)
            || !((*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong))
        {
            api_err_conflict(err, c"buf".as_ptr(), c"buffer".as_ptr());
            return;
        }
        if has_buf {
            if (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong
            {
                api_err_conflict(err, c"pattern".as_ptr(), c"buf".as_ptr());
                return;
            }
            b = find_buffer_by_handle(buf, err);
            if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                return;
            }
        }
        let mut patterns: Array = get_patterns_from_pattern_or_buf(
            (*opts).pattern,
            has_buf,
            buf,
            c"".as_ptr() as *mut ::core::ffi::c_char,
            arena,
            err,
        );
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return;
        }
        if (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_exec_autocmds__data
            != 0 as ::core::ffi::c_ulonglong
        {
            data = &raw mut (*opts).data;
        }
        modeline = if (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_exec_autocmds__modeline
            != 0 as ::core::ffi::c_ulonglong
        {
            (*opts).modeline as ::core::ffi::c_int
        } else {
            true_0
        } != 0;
        let mut did_aucmd: bool = false_0 != 0;
        let mut event_str_index: size_t = 0 as size_t;
        while event_str_index < event_array.size {
            let mut event_str: Object = *event_array.items.add(event_str_index);
            let mut event_nr: event_T = event_name2nr_str(event_str.data.string);
            if !((event_nr as ::core::ffi::c_uint)
                < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                api_err_invalid(
                    err,
                    c"event".as_ptr(),
                    event_str.data.string.data,
                    0 as int64_t,
                    true,
                );
                return;
            }
            let mut pat_index: size_t = 0 as size_t;
            while pat_index < patterns.size {
                let mut pat: Object = *patterns.items.add(pat_index);
                let mut fname: *mut ::core::ffi::c_char = if !has_buf {
                    pat.data.string.data
                } else {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                };
                did_aucmd = did_aucmd as ::core::ffi::c_int
                    | apply_autocmds_group(
                        event_nr,
                        fname,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        true,
                        au_group,
                        b,
                        ::core::ptr::null_mut::<exarg_T>(),
                        data,
                    ) as ::core::ffi::c_int
                    != 0;
                pat_index = pat_index.wrapping_add(1);
            }
            event_str_index = event_str_index.wrapping_add(1);
        }
        if did_aucmd as ::core::ffi::c_int != 0 && modeline as ::core::ffi::c_int != 0 {
            do_modelines(0 as ::core::ffi::c_int);
        }
    }
}
