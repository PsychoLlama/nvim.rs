//! One step of a firing walk.
//!
//! `getnextac` is the `do_cmdline` getline callback an autocommand body is
//! executed through: each call hands back the next matching command,
//! advancing `aucmd_next` over the pattern list until one matches the file
//! name being fired for.  `au_callback` is the same step for an autocommand
//! whose body is a Lua callback rather than a Vimscript command.  Because
//! the walk runs *while* the tables may be edited, every position here is
//! re-validated rather than cached.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn aucmd_next(mut apc: *mut AutoPatCmd) {
    unsafe {
        let entry: *mut estack_T = ((*exestack.ptr()).ga_data as *mut estack_T)
            .offset((*exestack.ptr()).ga_len as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
        let acs: *mut AutoCmdVec =
            (autocmds.ptr() as *mut AutoCmdVec).offset((*apc).event as ::core::ffi::c_int as isize);
        '_c2rust_label: {
            if (*apc).ausize <= (*acs).size {
            } else {
                __assert_fail(
                    b"apc->ausize <= kv_size(*acs)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2077 as ::core::ffi::c_uint,
                    b"void aucmd_next(AutoPatCmd *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut i: size_t = (*apc).auidx;
        while i < (*apc).ausize && !got_int.get() {
            let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
            let ap: *mut AutoPat = (*ac).pat;
            's_11: {
                if !ap.is_null() {
                    if ap != (*apc).lastpat {
                        if (*apc).group != AUGROUP_ALL as ::core::ffi::c_int
                            && (*apc).group != (*ap).group
                        {
                            break 's_11;
                        } else if if (*ap).buflocal_nr == 0 as ::core::ffi::c_int {
                            !match_file_pat(
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                &raw mut (*ap).reg_prog,
                                (*apc).fname,
                                (*apc).sfname,
                                (*apc).tail,
                                (*ap).allow_dirs as ::core::ffi::c_int,
                            ) as ::core::ffi::c_int
                        } else {
                            ((*ap).buflocal_nr != (*apc).arg_bufnr) as ::core::ffi::c_int
                        } != 0
                        {
                            break 's_11;
                        } else {
                            let name: *const ::core::ffi::c_char = event_nr2name((*apc).event);
                            let s: *const ::core::ffi::c_char =
                                gettext(b"%s Autocommands for \"%s\"\0".as_ptr()
                                    as *const ::core::ffi::c_char);
                            let sourcing_name_len: size_t = strlen(s)
                                .wrapping_add(strlen(name))
                                .wrapping_add((*ap).patlen as size_t)
                                .wrapping_add(1 as size_t);
                            let namep: *mut ::core::ffi::c_char =
                                xmalloc(sourcing_name_len) as *mut ::core::ffi::c_char;
                            snprintf(namep, sourcing_name_len, s, name, (*ap).pat);
                            if p_verbose.get() >= 8 as OptInt {
                                verbose_enter();
                                smsg(
                                    0 as ::core::ffi::c_int,
                                    gettext(
                                        b"Executing %s\0".as_ptr() as *const ::core::ffi::c_char
                                    ),
                                    namep,
                                );
                                verbose_leave();
                            }
                            let mut ptr_: *mut *mut ::core::ffi::c_void =
                                &raw mut (*entry).es_name as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr_);
                            *ptr_ = NULL_0;
                            let _ = *ptr_;
                            (*entry).es_name = namep;
                            (*entry).es_info.aucmd = apc;
                        }
                    }
                    (*apc).lastpat = ap;
                    (*apc).auidx = i;
                    line_breakcheck();
                    return;
                }
            }
            i = i.wrapping_add(1);
        }
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*entry).es_name as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
        (*entry).es_info.aucmd = ::core::ptr::null_mut::<AutoPatCmd>();
        (*apc).lastpat = ::core::ptr::null_mut::<AutoPat>();
        (*apc).auidx = SIZE_MAX as size_t;
    }
}

unsafe extern "C" fn au_callback(mut ac: *const AutoCmd, mut apc: *const AutoPatCmd) -> bool {
    unsafe {
        let mut callback: Callback = (*ac).handler_fn;
        if callback.type_0 as ::core::ffi::c_uint
            == kCallbackLua as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut data: Dict = Dict {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            };
            let mut data__items: [KeyValuePair; 7] = [KeyValuePair {
                key: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                value: Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                },
            }; 7];
            data.capacity = 7 as size_t;
            data.items = &raw mut data__items as *mut KeyValuePair;
            let c2rust_fresh3 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh3 as isize) = key_value_pair {
                key: cstr_as_string(b"id\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed { integer: (*ac).id },
                },
            };
            let c2rust_fresh4 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh4 as isize) = key_value_pair {
                key: cstr_as_string(b"event\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(event_nr2name((*apc).event)),
                    },
                },
            };
            let c2rust_fresh5 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh5 as isize) = key_value_pair {
                key: cstr_as_string(b"file\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string((*apc).afile_orig),
                    },
                },
            };
            let c2rust_fresh6 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh6 as isize) = key_value_pair {
                key: cstr_as_string(b"match\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(autocmd_match.get()),
                    },
                },
            };
            let c2rust_fresh7 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh7 as isize) = key_value_pair {
                key: cstr_as_string(b"buf\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: autocmd_bufnr.get() as Integer,
                    },
                },
            };
            if !(*apc).data.is_null() {
                let c2rust_fresh8 = data.size;
                data.size = data.size.wrapping_add(1);
                *data.items.offset(c2rust_fresh8 as isize) = key_value_pair {
                    key: cstr_as_string(b"data\0".as_ptr() as *const ::core::ffi::c_char),
                    value: *(*apc).data,
                };
            }
            let mut group: ::core::ffi::c_int = (*(*ac).pat).group;
            match group {
                -2 => {
                    abort();
                }
                -1 | -3 | -4 => {}
                _ => {
                    let c2rust_fresh9 = data.size;
                    data.size = data.size.wrapping_add(1);
                    *data.items.offset(c2rust_fresh9 as isize) = key_value_pair {
                        key: cstr_as_string(b"group\0".as_ptr() as *const ::core::ffi::c_char),
                        value: object {
                            type_0: kObjectTypeInteger,
                            data: C2Rust_Unnamed {
                                integer: group as Integer,
                            },
                        },
                    };
                }
            }
            let mut args: Array = Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
            let mut args__items: [Object; 1] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            }; 1];
            args.capacity = 1 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh10 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh10 as isize) = object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: data },
            };
            let mut result: Object = nlua_call_ref(
                callback.data.luaref,
                ::core::ptr::null::<::core::ffi::c_char>(),
                args,
                kRetNilBool,
                ::core::ptr::null_mut::<Arena>(),
                ::core::ptr::null_mut::<Error>(),
            );
            return result.type_0 as ::core::ffi::c_uint
                == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                && result.data.boolean as ::core::ffi::c_int == true_0;
        } else {
            let mut argsin: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            let mut rettv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            callback_call(
                &raw mut callback,
                0 as ::core::ffi::c_int,
                &raw mut argsin,
                &raw mut rettv,
            );
            return false_0 != 0;
        };
    }
}

pub unsafe extern "C" fn getnextac(
    mut _c: ::core::ffi::c_int,
    mut cookie: *mut ::core::ffi::c_void,
    mut _indent: ::core::ffi::c_int,
    mut _do_concat: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let apc: *mut AutoPatCmd = cookie as *mut AutoPatCmd;
        let acs: *mut AutoCmdVec =
            (autocmds.ptr() as *mut AutoCmdVec).offset((*apc).event as ::core::ffi::c_int as isize);
        aucmd_next(apc);
        if (*apc).lastpat.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        '_c2rust_label: {
            if (*apc).auidx < (*acs).size {
            } else {
                __assert_fail(
                    b"apc->auidx < kv_size(*acs)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2193 as ::core::ffi::c_uint,
                    b"char *getnextac(int, void *, int, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let ac: *mut AutoCmd = (*acs).items.offset((*apc).auidx as isize);
        '_c2rust_label_0: {
            if !(*ac).pat.is_null() {
            } else {
                __assert_fail(
                    b"ac->pat != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2195 as ::core::ffi::c_uint,
                    b"char *getnextac(int, void *, int, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut oneshot: bool = (*ac).once;
        if p_verbose.get() >= 9 as OptInt {
            verbose_enter_scroll();
            let mut handler_str: *mut ::core::ffi::c_char = aucmd_handler_to_string(ac);
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"autocommand %s\0".as_ptr() as *const ::core::ffi::c_char),
                handler_str,
            );
            msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut handler_str as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
            verbose_leave_scroll();
        }
        autocmd_nested.set((*ac).nested);
        current_sctx.set((*ac).script_ctx);
        (*apc).script_ctx = current_sctx.get();
        let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if !(*ac).handler_cmd.is_null() {
            retval = xstrdup((*ac).handler_cmd);
        } else {
            let mut ac_copy: AutoCmd = *ac;
            (*ac).pat = if oneshot as ::core::ffi::c_int != 0 {
                ::core::ptr::null_mut::<AutoPat>()
            } else {
                (*ac).pat
            };
            let mut rv: bool = au_callback(&raw mut ac_copy, apc);
            if oneshot {
                (*(*acs).items.offset((*apc).auidx as isize)).pat = ac_copy.pat;
            }
            oneshot = oneshot as ::core::ffi::c_int != 0 || rv as ::core::ffi::c_int != 0;
            retval = xcalloc(1 as size_t, 1 as size_t) as *mut ::core::ffi::c_char;
        }
        if oneshot {
            aucmd_del((*acs).items.offset((*apc).auidx as isize));
        }
        if (*apc).auidx < (*apc).ausize {
            (*apc).auidx = (*apc).auidx.wrapping_add(1);
        } else {
            (*apc).auidx = SIZE_MAX as size_t;
        }
        return retval;
    }
}
