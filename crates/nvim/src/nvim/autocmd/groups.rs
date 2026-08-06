//! Autocommand groups: the two maps, and `:augroup`.
//!
//! A group is an id and a name, held in `map_augroup_name_to_id` and
//! `map_augroup_id_to_name`; `augroup_add` allocates an id (reviving a
//! deleted group's), `augroup_del` releases the name but *not* the
//! autocommands defined under it, and `augroup_name` renders an id --
//! including `AUGROUP_DELETED`, whose name is the `--Deleted--`
//! placeholder.  `do_augroup` is `:augroup` and `:augroup!`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn augroup_map_del(
    mut id: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
) {
    unsafe {
        if !name.is_null() {
            let mut key: String_0 = String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            };
            map_del_String_int(
                map_augroup_name_to_id.ptr(),
                cstr_as_string(name),
                &raw mut key,
            );
            api_free_string(key);
        }
        if id > 0 as ::core::ffi::c_int {
            let mut mapped: String_0 = map_del_int_String(
                map_augroup_id_to_name.ptr(),
                id,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            );
            api_free_string(mapped);
        }
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn get_deleted_augroup() -> *const ::core::ffi::c_char {
    unsafe {
        if (*deleted_augroup.ptr()).is_null() {
            deleted_augroup.set(gettext(
                b"--Deleted--\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        return deleted_augroup.get();
    }
}

pub unsafe extern "C" fn augroup_add(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        '_c2rust_label: {
            if strcasecmp(
                name as *mut ::core::ffi::c_char,
                b"end\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) != 0 as ::core::ffi::c_int
            {
            } else {
                __assert_fail(
                    b"STRICMP(name, \"end\") != 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    400 as ::core::ffi::c_uint,
                    b"int augroup_add(const char *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut existing_id: ::core::ffi::c_int = augroup_find(name);
        if existing_id > 0 as ::core::ffi::c_int {
            '_c2rust_label_0: {
                if existing_id != AUGROUP_DELETED as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"existing_id != AUGROUP_DELETED\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        404 as ::core::ffi::c_uint,
                        b"int augroup_add(const char *)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            return existing_id;
        }
        if existing_id == AUGROUP_DELETED as ::core::ffi::c_int {
            augroup_map_del(existing_id, name);
        }
        let c2rust_fresh0 = next_augroup_id.get();
        next_augroup_id.set(next_augroup_id.get() + 1);
        let mut next_id: ::core::ffi::c_int = c2rust_fresh0;
        let mut name_key: String_0 = cstr_to_string(name);
        let mut name_val: String_0 = cstr_to_string(name);
        map_put_String_int(map_augroup_name_to_id.ptr(), name_key, next_id);
        map_put_int_String(map_augroup_id_to_name.ptr(), next_id, name_val);
        return next_id;
    }
}

pub unsafe extern "C" fn augroup_del(
    mut name: *mut ::core::ffi::c_char,
    mut stupid_legacy_mode: bool,
) {
    unsafe {
        let mut group: ::core::ffi::c_int = augroup_find(name);
        if group == AUGROUP_ERROR as ::core::ffi::c_int {
            semsg(
                gettext(b"E367: No such group: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
                name,
            );
            return;
        } else if group == current_augroup.get() {
            emsg(gettext(
                b"E936: Cannot delete the current group\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return;
        }
        if stupid_legacy_mode {
            let mut event: event_T = EVENT_BUFADD;
            while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
                let acs: *mut AutoCmdVec = (autocmds.ptr() as *mut AutoCmdVec)
                    .offset(event as ::core::ffi::c_int as isize);
                let mut i: size_t = 0 as size_t;
                while i < (*acs).size {
                    let ap: *mut AutoPat = (*(*acs).items.offset(i as isize)).pat;
                    if !ap.is_null() && (*ap).group == group {
                        give_warning(
                            gettext(b"W19: Deleting augroup that is still in use\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            true_0 != 0,
                            true_0 != 0,
                        );
                        map_put_String_int(
                            map_augroup_name_to_id.ptr(),
                            cstr_as_string(name),
                            AUGROUP_DELETED as ::core::ffi::c_int,
                        );
                        augroup_map_del((*ap).group, ::core::ptr::null::<::core::ffi::c_char>());
                        return;
                    }
                    i = i.wrapping_add(1);
                }
                event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
            }
        } else {
            let mut event_0: event_T = EVENT_BUFADD;
            while (event_0 as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
                let acs_0: *mut AutoCmdVec = (autocmds.ptr() as *mut AutoCmdVec)
                    .offset(event_0 as ::core::ffi::c_int as isize);
                let mut i_0: size_t = 0 as size_t;
                while i_0 < (*acs_0).size {
                    let ac: *mut AutoCmd = (*acs_0).items.offset(i_0 as isize);
                    if !(*ac).pat.is_null() && (*(*ac).pat).group == group {
                        aucmd_del(ac);
                    }
                    i_0 = i_0.wrapping_add(1);
                }
                event_0 = (event_0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
            }
        }
        augroup_map_del(group, name);
        au_cleanup();
    }
}

pub unsafe extern "C" fn augroup_find(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let mut existing_id: ::core::ffi::c_int =
            map_get_String_int(map_augroup_name_to_id.ptr(), cstr_as_string(name));
        if existing_id == AUGROUP_DELETED as ::core::ffi::c_int {
            return existing_id;
        }
        if existing_id > 0 as ::core::ffi::c_int {
            return existing_id;
        }
        return AUGROUP_ERROR as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn augroup_name(mut group: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    unsafe {
        '_c2rust_label: {
            if group != 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"group != 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    496 as ::core::ffi::c_uint,
                    b"char *augroup_name(int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if group == AUGROUP_DELETED as ::core::ffi::c_int {
            return get_deleted_augroup() as *mut ::core::ffi::c_char;
        }
        if group == AUGROUP_ALL as ::core::ffi::c_int {
            group = current_augroup.get();
        }
        if group == next_augroup_id.get() {
            return b"END\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        if group > next_augroup_id.get() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut key: String_0 = map_get_int_String(map_augroup_id_to_name.ptr(), group);
        if !key.data.is_null() {
            return key.data;
        }
        return get_deleted_augroup() as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn augroup_exists(mut name: *const ::core::ffi::c_char) -> bool {
    unsafe {
        return augroup_find(name) > 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn do_augroup(mut arg: *mut ::core::ffi::c_char, mut del_group: bool) {
    unsafe {
        if del_group {
            if *arg as ::core::ffi::c_int == NUL {
                emsg(gettext(&raw const e_argreq as *const ::core::ffi::c_char));
            } else {
                augroup_del(arg, true_0 != 0);
            }
        } else if strcasecmp(
            arg,
            b"end\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            current_augroup.set(AUGROUP_DEFAULT as ::core::ffi::c_int);
        } else if *arg != 0 {
            current_augroup.set(augroup_add(arg));
        } else {
            msg_start();
            msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
            let mut name: String_0 = String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            };
            let mut value: ::core::ffi::c_int = 0;
            let mut __i: uint32_t = 0;
            __i = 0 as uint32_t;
            while __i < (*map_augroup_name_to_id.ptr()).set.h.n_keys {
                name = *(*map_augroup_name_to_id.ptr())
                    .set
                    .keys
                    .offset(__i as isize);
                value = *(*map_augroup_name_to_id.ptr()).values.offset(__i as isize);
                if value > 0 as ::core::ffi::c_int {
                    msg_puts(name.data);
                } else {
                    msg_puts(augroup_name(value));
                }
                msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
                __i = __i.wrapping_add(1);
            }
            msg_clr_eos();
            msg_end();
        };
    }
}

pub unsafe extern "C" fn expand_get_augroup_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        return augroup_name(idx + 1 as ::core::ffi::c_int);
    }
}

pub(crate) unsafe extern "C" fn arg_augroup_get(
    mut argp: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut arg: *mut ::core::ffi::c_char = *argp;
        p = arg;
        while *p as ::core::ffi::c_int != 0
            && !ascii_iswhite(*p as ::core::ffi::c_int)
            && *p as ::core::ffi::c_int != '|' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        if p <= arg {
            return AUGROUP_ALL as ::core::ffi::c_int;
        }
        let mut group_name: *mut ::core::ffi::c_char = xmemdupz(
            arg as *const ::core::ffi::c_void,
            p.offset_from(arg) as size_t,
        ) as *mut ::core::ffi::c_char;
        let mut group: ::core::ffi::c_int = augroup_find(group_name);
        if group == AUGROUP_ERROR as ::core::ffi::c_int {
            group = AUGROUP_ALL as ::core::ffi::c_int;
        } else {
            *argp = skipwhite(p);
        }
        xfree(group_name as *mut ::core::ffi::c_void);
        return group;
    }
}
