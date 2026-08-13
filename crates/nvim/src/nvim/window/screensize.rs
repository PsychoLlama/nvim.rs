//! The screen changed size, and the autocommands that report it.
//!
//! [`win_new_screensize`] is the entry point when `'lines'` or `'columns'`
//! moved: it redistributes the new room over the frame tree and recomputes
//! every window's position.  The rest is the `WinScrolled`/`WinResized`
//! machinery -- [`snapshot_windows_scroll_size`] records every window's view
//! and size, [`check_window_scroll_resize`] compares the current state
//! against that snapshot, and [`may_trigger_win_scrolled_resized`] fires the
//! events with the `v:event` dict [`make_win_info_dict`] builds.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::autocmd::{
    EVENT_WINRESIZED, EVENT_WINSCROLLED, apply_autocmds, event_ignored, has_event,
};
use crate::src::nvim::buffer::{bufref_valid, set_bufref};
use crate::src::nvim::eval::typval::{
    tv_dict_add_dict, tv_dict_add_list, tv_dict_add_tv, tv_dict_alloc, tv_dict_extend,
    tv_dict_set_keys_readonly, tv_dict_unref, tv_list_alloc, tv_list_append_owned_tv,
};
use crate::src::nvim::eval::{get_v_event, restore_v_event};
use crate::src::nvim::ex_getln::compute_cmdrow;
use crate::src::nvim::garray::{ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    Columns, Rows, curbuf, curtab, firstwin, p_ch, p_window, skip_win_fix_scroll, topframe,
};
use crate::src::nvim::option::option_was_set;
use crate::src::nvim::options::kOptWindow;
use crate::src::nvim::os::libc::abs;
use crate::src::nvim::strings::vim_snprintf;
use crate::src::nvim::types::{
    OptInt, VAR_NUMBER, VAR_UNLOCKED, buf_T, bufref_T, dict_T, garray_T, hashitem_T, hashtab_T,
    linenr_T, list_T, ptrdiff_t, save_v_event_T, size_t, typval_T, typval_vval_union, varnumber_T,
    win_T,
};
use crate::src::nvim::winfloat::win_reconfig_floats;

pub unsafe extern "C" fn win_new_screensize() {
    unsafe {
        static old_Rows: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        static old_Columns: GlobalCell<::core::ffi::c_int> =
            GlobalCell::new(0 as ::core::ffi::c_int);
        if old_Rows.get() != Rows.get() {
            if p_window.get() == (old_Rows.get() - 1 as ::core::ffi::c_int) as OptInt
                || old_Rows.get() == 0 as ::core::ffi::c_int && !option_was_set(kOptWindow)
            {
                p_window.set((Rows.get() - 1 as ::core::ffi::c_int) as OptInt);
            }
            old_Rows.set(Rows.get());
            win_new_screen_rows();
        }
        if old_Columns.get() != Columns.get() {
            old_Columns.set(Columns.get());
            win_new_screen_cols();
        }
    }
}

pub unsafe extern "C" fn win_new_screen_rows() {
    unsafe {
        if (*firstwin.ptr()).is_null() {
            return;
        }
        let mut h: ::core::ffi::c_int = if (Rows.get() as OptInt
            - p_ch.get()
            - tabline_height() as OptInt
            - global_stl_height() as OptInt)
            as ::core::ffi::c_int
            > frame_minheight(topframe.get(), ::core::ptr::null_mut::<win_T>())
        {
            (Rows.get() as OptInt
                - p_ch.get()
                - tabline_height() as OptInt
                - global_stl_height() as OptInt) as ::core::ffi::c_int
        } else {
            frame_minheight(topframe.get(), ::core::ptr::null_mut::<win_T>())
        };
        frame_new_height(topframe.get(), h, false_0 != 0, true_0 != 0, false_0 != 0);
        if !frame_check_height(topframe.get(), h) {
            frame_new_height(topframe.get(), h, false_0 != 0, false_0 != 0, false_0 != 0);
        }
        win_comp_pos();
        win_reconfig_floats();
        compute_cmdrow();
        (*curtab.get()).tp_ch_used = p_ch.get();
        if !skip_win_fix_scroll.get() {
            win_fix_scroll(true_0 != 0);
        }
    }
}

pub unsafe extern "C" fn win_new_screen_cols() {
    unsafe {
        if (*firstwin.ptr()).is_null() {
            return;
        }
        frame_new_width(topframe.get(), Columns.get(), false_0 != 0, true_0 != 0);
        if !frame_check_width(topframe.get(), Columns.get()) {
            frame_new_width(topframe.get(), Columns.get(), false_0 != 0, false_0 != 0);
        }
        win_comp_pos();
        win_reconfig_floats();
    }
}

pub unsafe extern "C" fn snapshot_windows_scroll_size() {
    unsafe {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            (*wp).w_last_topline = (*wp).w_topline;
            (*wp).w_last_topfill = (*wp).w_topfill;
            (*wp).w_last_leftcol = (*wp).w_leftcol;
            (*wp).w_last_skipcol = (*wp).w_skipcol;
            (*wp).w_last_width = (*wp).w_width;
            (*wp).w_last_height = (*wp).w_height;
            wp = (*wp).w_next;
        }
    }
}

pub unsafe extern "C" fn may_make_initial_scroll_size_snapshot() {
    unsafe {
        if !did_initial_scroll_size_snapshot.get() {
            did_initial_scroll_size_snapshot.set(true_0 != 0);
            snapshot_windows_scroll_size();
        }
    }
}

unsafe extern "C" fn make_win_info_dict(
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
    mut topline: ::core::ffi::c_int,
    mut topfill: ::core::ffi::c_int,
    mut leftcol: ::core::ffi::c_int,
    mut skipcol: ::core::ffi::c_int,
) -> *mut dict_T {
    unsafe {
        let d: *mut dict_T = tv_dict_alloc();
        (*d).dv_refcount = 1 as ::core::ffi::c_int;
        let mut tv: typval_T = typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        tv.vval.v_number = width as varnumber_T;
        if tv_dict_add_tv(
            d,
            c"width".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            &raw mut tv,
        ) != FAIL
        {
            tv.vval.v_number = height as varnumber_T;
            if tv_dict_add_tv(
                d,
                c"height".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                &raw mut tv,
            ) != FAIL
            {
                tv.vval.v_number = topline as varnumber_T;
                if tv_dict_add_tv(
                    d,
                    c"topline".as_ptr(),
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                    &raw mut tv,
                ) != FAIL
                {
                    tv.vval.v_number = topfill as varnumber_T;
                    if tv_dict_add_tv(
                        d,
                        c"topfill".as_ptr(),
                        ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                            .wrapping_sub(1 as size_t),
                        &raw mut tv,
                    ) != FAIL
                    {
                        tv.vval.v_number = leftcol as varnumber_T;
                        if tv_dict_add_tv(
                            d,
                            c"leftcol".as_ptr(),
                            ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                                .wrapping_sub(1 as size_t),
                            &raw mut tv,
                        ) != FAIL
                        {
                            tv.vval.v_number = skipcol as varnumber_T;
                            if tv_dict_add_tv(
                                d,
                                c"skipcol".as_ptr(),
                                ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                                    .wrapping_sub(1 as size_t),
                                &raw mut tv,
                            ) != FAIL
                            {
                                return d;
                            }
                        }
                    }
                }
            }
        }
        tv_dict_unref(d);
        return ::core::ptr::null_mut::<dict_T>();
    }
}

unsafe extern "C" fn check_window_scroll_resize(
    mut size_count: *mut ::core::ffi::c_int,
    mut first_scroll_win: *mut *mut win_T,
    mut first_size_win: *mut *mut win_T,
    mut winlist: *mut list_T,
    mut v_event: *mut dict_T,
) {
    unsafe {
        let mut tot_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut tot_height: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut tot_topline: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut tot_topfill: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut tot_leftcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut tot_skipcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_floating as ::core::ffi::c_int != 0 && (*wp).w_last_topline == 0 as linenr_T
            {
                (*wp).w_last_topline = (*wp).w_topline;
                (*wp).w_last_topfill = (*wp).w_topfill;
                (*wp).w_last_leftcol = (*wp).w_leftcol;
                (*wp).w_last_skipcol = (*wp).w_skipcol;
                (*wp).w_last_width = (*wp).w_width;
                (*wp).w_last_height = (*wp).w_height;
            } else {
                let ignore_scroll: bool =
                    event_ignored(EVENT_WINSCROLLED, (*wp).w_onebuf_opt.wo_eiw);
                let size_changed: bool =
                    !event_ignored(EVENT_WINRESIZED, (*wp).w_onebuf_opt.wo_eiw)
                        && ((*wp).w_last_width != (*wp).w_width
                            || (*wp).w_last_height != (*wp).w_height);
                if size_changed {
                    if !winlist.is_null() {
                        let mut tv: typval_T = typval_T {
                            v_type: VAR_NUMBER,
                            v_lock: VAR_UNLOCKED,
                            vval: typval_vval_union {
                                v_number: (*wp).handle as varnumber_T,
                            },
                        };
                        tv_list_append_owned_tv(winlist, tv);
                    } else if !size_count.is_null() {
                        debug_assert!(
                            !first_size_win.is_null() && !first_scroll_win.is_null(),
                            "first_size_win != NULL && first_scroll_win != NULL"
                        );
                        *size_count += 1;
                        if (*first_size_win).is_null() {
                            *first_size_win = wp;
                        }
                        if (*first_scroll_win).is_null() && !ignore_scroll {
                            *first_scroll_win = wp;
                        }
                    }
                }
                let scroll_changed: bool = !ignore_scroll
                    && ((*wp).w_last_topline != (*wp).w_topline
                        || (*wp).w_last_topfill != (*wp).w_topfill
                        || (*wp).w_last_leftcol != (*wp).w_leftcol
                        || (*wp).w_last_skipcol != (*wp).w_skipcol);
                if scroll_changed as ::core::ffi::c_int != 0
                    && !first_scroll_win.is_null()
                    && (*first_scroll_win).is_null()
                {
                    *first_scroll_win = wp;
                }
                if (size_changed as ::core::ffi::c_int != 0
                    || scroll_changed as ::core::ffi::c_int != 0)
                    && !v_event.is_null()
                {
                    let mut width: ::core::ffi::c_int = (*wp).w_width - (*wp).w_last_width;
                    let mut height: ::core::ffi::c_int = (*wp).w_height - (*wp).w_last_height;
                    let mut topline: ::core::ffi::c_int = (*wp).w_topline as ::core::ffi::c_int
                        - (*wp).w_last_topline as ::core::ffi::c_int;
                    let mut topfill: ::core::ffi::c_int = (*wp).w_topfill - (*wp).w_last_topfill;
                    let mut leftcol: ::core::ffi::c_int = (*wp).w_leftcol as ::core::ffi::c_int
                        - (*wp).w_last_leftcol as ::core::ffi::c_int;
                    let mut skipcol: ::core::ffi::c_int = (*wp).w_skipcol as ::core::ffi::c_int
                        - (*wp).w_last_skipcol as ::core::ffi::c_int;
                    let mut d: *mut dict_T =
                        make_win_info_dict(width, height, topline, topfill, leftcol, skipcol);
                    if d.is_null() {
                        break;
                    }
                    let mut winid: [::core::ffi::c_char; 65] = [0; 65];
                    let mut key_len: ::core::ffi::c_int = vim_snprintf(
                        &raw mut winid as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
                        c"%d".as_ptr(),
                        (*wp).handle,
                    );
                    if tv_dict_add_dict(
                        v_event,
                        &raw mut winid as *mut ::core::ffi::c_char,
                        key_len as size_t,
                        d,
                    ) == FAIL
                    {
                        tv_dict_unref(d);
                        break;
                    } else {
                        (*d).dv_refcount -= 1;
                        tot_width += abs(width);
                        tot_height += abs(height);
                        tot_topline += abs(topline);
                        tot_topfill += abs(topfill);
                        tot_leftcol += abs(leftcol);
                        tot_skipcol += abs(skipcol);
                    }
                }
            }
            wp = (*wp).w_next;
        }
        if !v_event.is_null() {
            let mut alldict: *mut dict_T = make_win_info_dict(
                tot_width,
                tot_height,
                tot_topline,
                tot_topfill,
                tot_leftcol,
                tot_skipcol,
            );
            if !alldict.is_null() {
                if tv_dict_add_dict(
                    v_event,
                    c"all".as_ptr(),
                    ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                    alldict,
                ) == FAIL
                {
                    tv_dict_unref(alldict);
                } else {
                    (*alldict).dv_refcount -= 1;
                }
            }
        }
    }
}

pub unsafe extern "C" fn may_trigger_win_scrolled_resized() {
    unsafe {
        static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        let do_resize: bool = has_event(EVENT_WINRESIZED);
        let do_scroll: bool = has_event(EVENT_WINSCROLLED);
        if recursive.get() as ::core::ffi::c_int != 0
            || !(do_scroll as ::core::ffi::c_int != 0 || do_resize as ::core::ffi::c_int != 0)
            || !did_initial_scroll_size_snapshot.get()
        {
            return;
        }
        let mut size_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut first_scroll_win: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut first_size_win: *mut win_T = ::core::ptr::null_mut::<win_T>();
        check_window_scroll_resize(
            &raw mut size_count,
            &raw mut first_scroll_win,
            &raw mut first_size_win,
            ::core::ptr::null_mut::<list_T>(),
            ::core::ptr::null_mut::<dict_T>(),
        );
        let mut trigger_resize: bool =
            do_resize as ::core::ffi::c_int != 0 && size_count > 0 as ::core::ffi::c_int;
        let mut trigger_scroll: bool =
            do_scroll as ::core::ffi::c_int != 0 && !first_scroll_win.is_null();
        if !trigger_resize && !trigger_scroll {
            return;
        }
        let mut windows_list: *mut list_T = ::core::ptr::null_mut::<list_T>();
        if trigger_resize {
            windows_list = tv_list_alloc(size_count as ptrdiff_t);
            check_window_scroll_resize(
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ::core::ptr::null_mut::<*mut win_T>(),
                ::core::ptr::null_mut::<*mut win_T>(),
                windows_list,
                ::core::ptr::null_mut::<dict_T>(),
            );
        }
        let mut scroll_dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        if trigger_scroll {
            scroll_dict = tv_dict_alloc();
            (*scroll_dict).dv_refcount = 1 as ::core::ffi::c_int;
            check_window_scroll_resize(
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ::core::ptr::null_mut::<*mut win_T>(),
                ::core::ptr::null_mut::<*mut win_T>(),
                ::core::ptr::null_mut::<list_T>(),
                scroll_dict,
            );
        }
        snapshot_windows_scroll_size();
        recursive.set(true_0 != 0);
        let mut resize_winid: [::core::ffi::c_char; 65] = [0; 65];
        let mut resize_bufref: bufref_T = bufref_T::default();
        if trigger_resize {
            vim_snprintf(
                &raw mut resize_winid as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
                c"%d".as_ptr(),
                (*first_size_win).handle,
            );
            set_bufref(&raw mut resize_bufref, (*first_size_win).w_buffer);
        }
        let mut scroll_winid: [::core::ffi::c_char; 65] = [0; 65];
        let mut scroll_bufref: bufref_T = bufref_T::default();
        if trigger_scroll {
            vim_snprintf(
                &raw mut scroll_winid as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
                c"%d".as_ptr(),
                (*first_scroll_win).handle,
            );
            set_bufref(&raw mut scroll_bufref, (*first_scroll_win).w_buffer);
        }
        if trigger_resize {
            let mut save_v_event: save_v_event_T = save_v_event_T {
                sve_did_save: false,
                sve_hashtab: hashtab_T {
                    ht_mask: 0,
                    ht_used: 0,
                    ht_filled: 0,
                    ht_changed: 0,
                    ht_locked: 0,
                    ht_array: ::core::ptr::null_mut::<hashitem_T>(),
                    ht_smallarray: [hashitem_T {
                        hi_hash: 0,
                        hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    }; 16],
                },
            };
            let mut v_event: *mut dict_T = get_v_event(&raw mut save_v_event);
            if tv_dict_add_list(
                v_event,
                c"windows".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                windows_list,
            ) == OK
            {
                tv_dict_set_keys_readonly(v_event);
                let mut buf: *mut buf_T =
                    if bufref_valid(&raw mut resize_bufref) as ::core::ffi::c_int != 0 {
                        resize_bufref.br_buf
                    } else {
                        curbuf.get()
                    };
                apply_autocmds(
                    EVENT_WINRESIZED,
                    &raw mut resize_winid as *mut ::core::ffi::c_char,
                    &raw mut resize_winid as *mut ::core::ffi::c_char,
                    false_0 != 0,
                    buf,
                );
            }
            restore_v_event(v_event, &raw mut save_v_event);
        }
        if trigger_scroll {
            let mut save_v_event_0: save_v_event_T = save_v_event_T {
                sve_did_save: false,
                sve_hashtab: hashtab_T {
                    ht_mask: 0,
                    ht_used: 0,
                    ht_filled: 0,
                    ht_changed: 0,
                    ht_locked: 0,
                    ht_array: ::core::ptr::null_mut::<hashitem_T>(),
                    ht_smallarray: [hashitem_T {
                        hi_hash: 0,
                        hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    }; 16],
                },
            };
            let mut v_event_0: *mut dict_T = get_v_event(&raw mut save_v_event_0);
            tv_dict_extend(v_event_0, scroll_dict, c"move".as_ptr());
            tv_dict_set_keys_readonly(v_event_0);
            tv_dict_unref(scroll_dict);
            let mut buf_0: *mut buf_T =
                if bufref_valid(&raw mut scroll_bufref) as ::core::ffi::c_int != 0 {
                    scroll_bufref.br_buf
                } else {
                    curbuf.get()
                };
            apply_autocmds(
                EVENT_WINSCROLLED,
                &raw mut scroll_winid as *mut ::core::ffi::c_char,
                &raw mut scroll_winid as *mut ::core::ffi::c_char,
                false_0 != 0,
                buf_0,
            );
            restore_v_event(v_event_0, &raw mut save_v_event_0);
        }
        recursive.set(false_0 != 0);
    }
}

pub unsafe extern "C" fn win_size_save(mut gap: *mut garray_T) {
    unsafe {
        ga_init(
            gap,
            ::core::mem::size_of::<::core::ffi::c_int>() as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
        ga_grow(
            gap,
            win_count() * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
        );
        let c2rust_fresh3 = (*gap).ga_len;
        (*gap).ga_len = (*gap).ga_len + 1;
        *((*gap).ga_data as *mut ::core::ffi::c_int).offset(c2rust_fresh3 as isize) =
            (Rows.get() as OptInt
                - p_ch.get()
                - tabline_height() as OptInt
                - global_stl_height() as OptInt) as ::core::ffi::c_int
                + global_stl_height()
                - last_stl_height(false_0 != 0);
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            let c2rust_fresh4 = (*gap).ga_len;
            (*gap).ga_len = (*gap).ga_len + 1;
            *((*gap).ga_data as *mut ::core::ffi::c_int).offset(c2rust_fresh4 as isize) =
                (*wp).w_width + (*wp).w_vsep_width;
            let c2rust_fresh5 = (*gap).ga_len;
            (*gap).ga_len = (*gap).ga_len + 1;
            *((*gap).ga_data as *mut ::core::ffi::c_int).offset(c2rust_fresh5 as isize) =
                (*wp).w_height;
            wp = (*wp).w_next;
        }
    }
}

pub unsafe extern "C" fn win_size_restore(mut gap: *mut garray_T) {
    unsafe {
        if win_count() * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int == (*gap).ga_len
            && *((*gap).ga_data as *mut ::core::ffi::c_int).offset(0 as ::core::ffi::c_int as isize)
                as OptInt
                == Rows.get() as OptInt
                    - p_ch.get()
                    - tabline_height() as OptInt
                    - global_stl_height() as OptInt
                    + global_stl_height() as OptInt
                    - last_stl_height(false_0 != 0) as OptInt
        {
            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while j < 2 as ::core::ffi::c_int {
                let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                    firstwin.get()
                } else {
                    (*curtab.get()).tp_firstwin
                };
                while !wp.is_null() {
                    let c2rust_fresh6 = i;
                    i = i + 1;
                    let mut width: ::core::ffi::c_int =
                        *((*gap).ga_data as *mut ::core::ffi::c_int).offset(c2rust_fresh6 as isize);
                    let c2rust_fresh7 = i;
                    i = i + 1;
                    let mut height: ::core::ffi::c_int =
                        *((*gap).ga_data as *mut ::core::ffi::c_int).offset(c2rust_fresh7 as isize);
                    if !(*wp).w_floating {
                        frame_setwidth((*wp).w_frame, width);
                        win_setheight_win(height, wp);
                    }
                    wp = (*wp).w_next;
                }
                j += 1;
            }
            win_comp_pos();
        }
    }
}
