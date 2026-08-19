//! Listing buffers, windows and tabpages, and switching between them.
//!
//! Ten accessors of one shape -- `nvim_list_*` walks the editor's own list
//! into an `Array` of handles, `nvim_get_current_*` reads the pointer and
//! `nvim_set_current_*` moves it -- plus `nvim_create_buf`, which is the
//! only one that builds something.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported, array_add};
use crate::types::OptionSetFlags;

pub unsafe fn nvim_list_bufs(arena: *mut Arena) -> Array {
    unsafe {
        let mut n: size_t = 0 as size_t;
        let mut b: *mut buf_T = firstbuf.get();
        while !b.is_null() {
            n = n.wrapping_add(1);
            b = (*b).b_next;
        }
        let mut rv: Array = arena_array(arena, n);
        let mut b_0: *mut buf_T = firstbuf.get();
        while !b_0.is_null() {
            array_add(&mut rv, Object::buffer((*b_0).handle));
            b_0 = (*b_0).b_next;
        }
        return rv;
    }
}

pub unsafe fn nvim_get_current_buf() -> Buffer {
    unsafe {
        return (*curbuf.get()).handle as Buffer;
    }
}

pub unsafe fn nvim_set_current_buf(buf: Buffer) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return ().reported(error);
        }
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        do_buffer(
            DOBUF_GOTO as ::core::ffi::c_int,
            DOBUF_FIRST as ::core::ffi::c_int,
            FORWARD as ::core::ffi::c_int,
            (*b).handle as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        try_leave(&raw mut tstate, err);
    }
    ().reported(error)
}

pub unsafe fn nvim_list_wins(arena: *mut Arena) -> Array {
    unsafe {
        let mut n: size_t = 0 as size_t;
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                n = n.wrapping_add(1);
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        let mut rv: Array = arena_array(arena, n);
        let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp_0.is_null() {
            let mut wp_0: *mut win_T = if tp_0 == curtab.get() {
                firstwin.get()
            } else {
                (*tp_0).tp_firstwin
            };
            while !wp_0.is_null() {
                array_add(&mut rv, Object::window((*wp_0).handle));
                wp_0 = (*wp_0).w_next;
            }
            tp_0 = (*tp_0).tp_next as *mut tabpage_T;
        }
        return rv;
    }
}

pub unsafe fn nvim_get_current_win() -> Window {
    unsafe {
        return (*curwin.get()).handle as Window;
    }
}

pub unsafe fn nvim_set_current_win(win: Window) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut w: *mut win_T = find_window_by_handle(win, err);
        if w.is_null() {
            return ().reported(error);
        }
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        if (*w).w_buffer != curbuf.get() {
            reset_VIsual_and_resel();
        }
        goto_tabpage_win(win_find_tabpage(w), w);
        try_leave(&raw mut tstate, err);
    }
    ().reported(error)
}

pub unsafe fn nvim_create_buf(listed: Boolean, scratch: Boolean) -> Result<Buffer, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut ret: Buffer = 0 as Buffer;
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        block_autocmds();
        let mut buf: *mut buf_T = buflist_new(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as linenr_T,
            BLN_NOOPT as ::core::ffi::c_int
                | BLN_NEW as ::core::ffi::c_int
                | (if listed as ::core::ffi::c_int != 0 {
                    BLN_LISTED as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }),
        );
        if buf.is_null() {
            unblock_autocmds();
        } else if ml_open(buf) == 0 as ::core::ffi::c_int {
            unblock_autocmds();
        } else {
            (*buf).b_last_changedtick = buf_get_changedtick(buf);
            (*buf).b_last_changedtick_i = buf_get_changedtick(buf);
            (*buf).b_last_changedtick_pum = buf_get_changedtick(buf);
            buf_copy_options(
                buf,
                BCO_ENTER as ::core::ffi::c_int | BCO_NOHELP as ::core::ffi::c_int,
            );
            if scratch {
                set_option_direct_for(
                    kOptBufhidden,
                    OptVal {
                        type_0: kOptValTypeString,
                        data: OptValData {
                            string: String_0 {
                                data: c"hide".as_ptr() as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                                    .wrapping_sub(1 as size_t),
                            },
                        },
                    },
                    OptionSetFlags::LOCAL,
                    0 as scid_T,
                    kOptScopeBuf,
                    buf as *mut ::core::ffi::c_void,
                );
                set_option_direct_for(
                    kOptBuftype,
                    OptVal {
                        type_0: kOptValTypeString,
                        data: OptValData {
                            string: String_0 {
                                data: c"nofile".as_ptr() as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                    .wrapping_sub(1 as size_t),
                            },
                        },
                    },
                    OptionSetFlags::LOCAL,
                    0 as scid_T,
                    kOptScopeBuf,
                    buf as *mut ::core::ffi::c_void,
                );
                debug_assert!(
                    (*(*buf).b_ml.ml_mfp).mf_fd < 0 as ::core::ffi::c_int,
                    "buf->b_ml.ml_mfp->mf_fd < 0"
                );
                (*buf).b_p_swf = 0 as ::core::ffi::c_int;
                (*buf).b_p_ml = 0 as ::core::ffi::c_int;
            }
            unblock_autocmds();
            let mut bufref: bufref_T = bufref_T::default();
            set_bufref(&raw mut bufref, buf);
            if !(apply_autocmds(
                EVENT_BUFNEW,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false,
                buf,
            ) as ::core::ffi::c_int
                != 0
                && !bufref_valid(&raw mut bufref))
            {
                if !(listed as ::core::ffi::c_int != 0
                    && apply_autocmds(
                        EVENT_BUFADD,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        false,
                        buf,
                    ) as ::core::ffi::c_int
                        != 0
                    && !bufref_valid(&raw mut bufref))
                {
                    ret = (*buf).handle as Buffer;
                }
            }
        }
        try_leave(&raw mut tstate, err);
        if ret == 0 as ::core::ffi::c_int
            && !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
        {
            api_set_error(
                err,
                kErrorTypeException,
                c"Failed to create buffer".as_ptr(),
            );
        }
        return ret.reported(error);
    }
}

pub unsafe fn nvim_list_tabpages(arena: *mut Arena) -> Array {
    unsafe {
        let mut n: size_t = 0 as size_t;
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            n = n.wrapping_add(1);
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        let mut rv: Array = arena_array(arena, n);
        let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp_0.is_null() {
            array_add(&mut rv, Object::tabpage((*tp_0).handle));
            tp_0 = (*tp_0).tp_next as *mut tabpage_T;
        }
        return rv;
    }
}

pub unsafe fn nvim_get_current_tabpage() -> Tabpage {
    unsafe {
        return (*curtab.get()).handle as Tabpage;
    }
}

pub unsafe fn nvim_set_current_tabpage(tabpage: Tabpage) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut tp: *mut tabpage_T = find_tab_by_handle(tabpage, err);
        if tp.is_null() {
            return ().reported(error);
        }
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        goto_tabpage_tp(tp, true, true);
        try_leave(&raw mut tstate, err);
    }
    ().reported(error)
}
