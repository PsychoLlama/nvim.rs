//! Everything that fires an event on purpose.
//!
//! `do_doautocmd` is `:doautocmd` and `ex_doautoall` is `:doautoall`, which
//! runs the event in every loaded buffer.  Below them are the editor's own
//! triggers: the deferred queue (`aucmd_defer`/`deferred_event`, for events
//! that must not fire inside the code that noticed them),
//! `do_termresponse_autocmd`, the UIEnter/UILeave pair, FocusGained/Lost,
//! VimSuspend/VimResume and FileType.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn do_doautocmd(
    mut arg_start: *mut ::core::ffi::c_char,
    mut do_msg: bool,
    mut did_something: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = arg_start;
        let mut nothing_done: ::core::ffi::c_int = true_0;
        if !did_something.is_null() {
            *did_something = false_0 != 0;
        }
        let mut group: ::core::ffi::c_int = arg_augroup_get(&raw mut arg);
        if *arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
            emsg(gettext(
                b"E217: Can't execute autocommands for ALL events\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        let mut fname: *mut ::core::ffi::c_char =
            arg_event_skip(arg, group != AUGROUP_ALL as ::core::ffi::c_int);
        if fname.is_null() {
            return FAIL;
        }
        fname = skipwhite(fname);
        while *arg as ::core::ffi::c_int != 0
            && ends_excmd(*arg as ::core::ffi::c_int) == 0
            && !ascii_iswhite(*arg as ::core::ffi::c_int)
        {
            if apply_autocmds_group(
                event_name2nr(arg, &raw mut arg),
                fname,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                true_0 != 0,
                group,
                curbuf.get(),
                ::core::ptr::null_mut::<exarg_T>(),
                ::core::ptr::null_mut::<Object>(),
            ) {
                nothing_done = false_0;
            }
        }
        if nothing_done != 0 && do_msg as ::core::ffi::c_int != 0 && !aborting() {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"No matching autocommands: %s\0".as_ptr() as *const ::core::ffi::c_char),
                arg_start,
            );
        }
        if !did_something.is_null() {
            *did_something = nothing_done == 0;
        }
        return if aborting() as ::core::ffi::c_int != 0 {
            FAIL
        } else {
            OK
        };
    }
}

pub unsafe fn ex_doautoall(mut eap: *mut exarg_T) {
    unsafe {
        let mut retval: ::core::ffi::c_int = OK;
        let mut aco: aco_save_T = aco_save_T::default();
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut call_do_modelines: ::core::ffi::c_int =
            check_nomodeline(&raw mut arg) as ::core::ffi::c_int;
        let mut bufref: bufref_T = bufref_T::default();
        let mut did_aucmd: bool = false;
        let mut buf: *mut buf_T = firstbuf.get();
        while !buf.is_null() {
            if !((*buf).b_ml.ml_mfp.is_null() || buf == curbuf.get()) {
                aucmd_prepbuf(&raw mut aco, buf);
                set_bufref(&raw mut bufref, buf);
                retval = do_doautocmd(arg, false_0 != 0, &raw mut did_aucmd);
                if call_do_modelines != 0 && did_aucmd as ::core::ffi::c_int != 0 {
                    do_modelines(if is_aucmd_win(curwin.get()) as ::core::ffi::c_int != 0 {
                        OPT_NOWIN as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    });
                }
                aucmd_restbuf(&raw mut aco);
                if retval == FAIL || !bufref_valid(&raw mut bufref) {
                    retval = FAIL;
                    break;
                }
            }
            buf = (*buf).b_next;
        }
        if retval == OK {
            do_doautocmd(arg, false_0 != 0, &raw mut did_aucmd);
            if call_do_modelines != 0 && did_aucmd as ::core::ffi::c_int != 0 {
                do_modelines(0 as ::core::ffi::c_int);
            }
        }
    }
}

pub unsafe extern "C" fn aucmd_defer(
    mut event: event_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fname_io: *mut ::core::ffi::c_char,
    mut group: ::core::ffi::c_int,
    mut buf: *mut buf_T,
    mut eap: *mut exarg_T,
    mut data: *mut Object,
) {
    unsafe {
        let mut evdata: *mut AutoCmdEvent =
            xmalloc(::core::mem::size_of::<AutoCmdEvent>()) as *mut AutoCmdEvent;
        (*evdata).event = event;
        (*evdata).fname = if !fname.is_null() {
            xstrdup(fname)
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        };
        (*evdata).fname_io = if !fname_io.is_null() {
            xstrdup(fname_io)
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        };
        (*evdata).group = group;
        (*evdata).buf = (*buf).handle as Buffer;
        (*evdata).eap = eap;
        if !data.is_null() {
            (*evdata).data = xmalloc(::core::mem::size_of::<Object>()) as *mut Object;
            *(*evdata).data = copy_object(*data, ::core::ptr::null_mut::<Arena>());
        } else {
            (*evdata).data = ::core::ptr::null_mut::<Object>();
        }
        multiqueue_put_event(
            deferred_events.get(),
            Event {
                handler: Some(
                    deferred_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    evdata as *mut ::core::ffi::c_void,
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ],
            },
        );
    }
}

unsafe extern "C" fn deferred_event(mut argv: *mut *mut ::core::ffi::c_void) {
    unsafe {
        let mut e: *mut AutoCmdEvent =
            *argv.offset(0 as ::core::ffi::c_int as isize) as *mut AutoCmdEvent;
        let mut event: event_T = (*e).event;
        let mut fname: *mut ::core::ffi::c_char = (*e).fname;
        let mut fname_io: *mut ::core::ffi::c_char = (*e).fname_io;
        let mut group: ::core::ffi::c_int = (*e).group;
        let mut eap: *mut exarg_T = (*e).eap;
        let mut data: *mut Object = (*e).data;
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut buf: *mut buf_T = find_buffer_by_handle((*e).buf, &raw mut err);
        if !buf.is_null() {
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
            if !data.is_null()
                && (*data).type_0 as ::core::ffi::c_uint
                    == kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut i: size_t = 0 as size_t;
                while i < (*data).data.dict.size {
                    let mut item: KeyValuePair = *(*data).data.dict.items.offset(i as isize);
                    let mut tv: typval_T = typval_T {
                        v_type: VAR_UNKNOWN,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_number: 0 },
                    };
                    object_to_vim(item.value, &raw mut tv, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        api_clear_error(&raw mut err);
                    } else {
                        tv_dict_add_tv(v_event, item.key.data, item.key.size, &raw mut tv);
                        tv_clear(&raw mut tv);
                    }
                    i = i.wrapping_add(1);
                }
            }
            tv_dict_set_keys_readonly(v_event);
            let mut aco: aco_save_T = aco_save_T::default();
            aucmd_prepbuf(&raw mut aco, buf);
            apply_autocmds_group(event, fname, fname_io, false_0 != 0, group, buf, eap, data);
            aucmd_restbuf(&raw mut aco);
            restore_v_event(v_event, &raw mut save_v_event);
        }
        xfree(fname as *mut ::core::ffi::c_void);
        xfree(fname_io as *mut ::core::ffi::c_void);
        if !data.is_null() {
            api_free_object(*data);
            xfree(data as *mut ::core::ffi::c_void);
        }
        xfree(e as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn do_termresponse_autocmd(sequence: String_0) {
    unsafe {
        let mut data: Dict = Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
        let mut data__items: [KeyValuePair; 1] = [KeyValuePair {
            key: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            value: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
        }; 1];
        data.capacity = 1 as size_t;
        data.items = &raw mut data__items as *mut KeyValuePair;
        let c2rust_fresh11 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh11 as isize) = key_value_pair {
            key: cstr_as_string(b"sequence\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed { string: sequence },
            },
        };
        let mut c2rust_lvalue: Object = object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: data },
        };
        apply_autocmds_group(
            EVENT_TERMRESPONSE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            true_0 != 0,
            AUGROUP_ALL as ::core::ffi::c_int,
            ::core::ptr::null_mut::<buf_T>(),
            ::core::ptr::null_mut::<exarg_T>(),
            &raw mut c2rust_lvalue,
        );
        termresponse_changed.set(true_0 != 0);
    }
}

unsafe extern "C" fn vimresume_event(mut _argv: *mut *mut ::core::ffi::c_void) {
    unsafe {
        apply_autocmds(
            EVENT_VIMRESUME,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            ::core::ptr::null_mut::<buf_T>(),
        );
        pending_vimresume.set(kFalse);
    }
}

pub unsafe extern "C" fn may_trigger_vim_suspend_resume(mut suspend: bool) {
    unsafe {
        if suspend as ::core::ffi::c_int != 0
            && pending_vimresume.get() as ::core::ffi::c_int == kFalse as ::core::ffi::c_int
        {
            pending_vimresume.set(kNone);
            apply_autocmds(
                EVENT_VIMSUSPEND,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                ::core::ptr::null_mut::<buf_T>(),
            );
            pending_vimresume.set(kTrue);
        } else if !suspend
            && pending_vimresume.get() as ::core::ffi::c_int == kTrue as ::core::ffi::c_int
        {
            pending_vimresume.set(kNone);
            multiqueue_put_event(
                (*main_loop.ptr()).events,
                Event {
                    handler: Some(
                        vimresume_event
                            as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                    ),
                    argv: [
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ],
                },
            );
        }
    }
}

pub unsafe extern "C" fn do_autocmd_uienter(mut chanid: uint64_t, mut attached: bool) {
    unsafe {
        static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        if starting.get() == NO_SCREEN {
            return;
        }
        if recursive.get() {
            return;
        }
        recursive.set(true_0 != 0);
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
        let mut dict: *mut dict_T = get_v_event(&raw mut save_v_event);
        '_c2rust_label: {
            if chanid < 9223372036854775807 as uint64_t {
            } else {
                __assert_fail(
                    b"chanid < VARNUMBER_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2697 as ::core::ffi::c_uint,
                    b"void do_autocmd_uienter(uint64_t, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        tv_dict_add_nr(
            dict,
            b"chan\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            chanid as varnumber_T,
        );
        tv_dict_set_keys_readonly(dict);
        apply_autocmds(
            (if attached as ::core::ffi::c_int != 0 {
                EVENT_UIENTER as ::core::ffi::c_int
            } else {
                EVENT_UILEAVE as ::core::ffi::c_int
            }) as event_T,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        restore_v_event(dict, &raw mut save_v_event);
        recursive.set(false_0 != 0);
    }
}

pub unsafe extern "C" fn do_autocmd_focusgained(mut gained: bool) {
    unsafe {
        static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        static last_time: GlobalCell<Timestamp> = GlobalCell::new(0 as Timestamp);
        if recursive.get() {
            return;
        }
        recursive.set(true_0 != 0);
        apply_autocmds(
            (if gained as ::core::ffi::c_int != 0 {
                EVENT_FOCUSGAINED as ::core::ffi::c_int
            } else {
                EVENT_FOCUSLOST as ::core::ffi::c_int
            }) as event_T,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        if gained as ::core::ffi::c_int != 0
            && (*last_time.ptr()).wrapping_add(2000 as ::core::ffi::c_int as Timestamp) < os_now()
        {
            check_timestamps(true_0);
            last_time.set(os_now() as Timestamp);
        }
        recursive.set(false_0 != 0);
    }
}

pub unsafe extern "C" fn do_filetype_autocmd(mut buf: *mut buf_T, mut force: bool) -> bool {
    unsafe {
        static ft_recursive: GlobalCell<::core::ffi::c_int> =
            GlobalCell::new(0 as ::core::ffi::c_int);
        if ft_recursive.get() > 0 as ::core::ffi::c_int && !force {
            return false_0 != 0;
        }
        let mut secure_save: ::core::ffi::c_int = secure.get();
        secure.set(0 as ::core::ffi::c_int);
        (*ft_recursive.ptr()) += 1;
        (*buf).b_did_filetype = true_0 != 0;
        let mut ret: bool = apply_autocmds(
            EVENT_FILETYPE,
            (*buf).b_p_ft,
            (*buf).b_fname,
            force as ::core::ffi::c_int != 0 || ft_recursive.get() == 1 as ::core::ffi::c_int,
            buf,
        );
        (*ft_recursive.ptr()) -= 1;
        secure.set(secure_save);
        return ret;
    }
}
