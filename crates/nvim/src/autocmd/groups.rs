//! Autocommand groups: the two maps, and `:augroup`.
//!
//! A group is an id and a name, held in `map_augroup_name_to_id` and
//! `map_augroup_id_to_name`; [`augroup_add`] allocates an id (reviving a
//! deleted group's), [`augroup_del`] releases the name but *not* the
//! autocommands defined under it, and [`augroup_name`] renders an id --
//! including `AUGROUP_DELETED`, whose name is the `--Deleted--`
//! placeholder.  [`do_augroup`] is `:augroup` and `:augroup!`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
#[allow(unused_imports)]
use crate::semsg_c;

/// Drop a group's entries from the two maps, freeing the keys they own.
///
/// A null `name` leaves the name map alone, which is how `:augroup! X`
/// releases the *id* of a group whose name it has just re-pointed at
/// `AUGROUP_DELETED`.
unsafe fn augroup_map_del(id: ::core::ffi::c_int, name: *const ::core::ffi::c_char) {
    unsafe {
        if !name.is_null() {
            let mut key = STRING_INIT;
            map_del_String_int(
                map_augroup_name_to_id.ptr(),
                cstr_as_string(name),
                &raw mut key,
            );
            api_free_string(key);
        }
        if id > 0 {
            let mapped =
                map_del_int_String(map_augroup_id_to_name.ptr(), id, ::core::ptr::null_mut());
            api_free_string(mapped);
        }
    }
}

/// The name a deleted-but-still-referenced group lists under, translated
/// once and cached.
#[inline(always)]
pub(crate) unsafe extern "C" fn get_deleted_augroup() -> *const ::core::ffi::c_char {
    unsafe {
        if deleted_augroup.get().is_null() {
            deleted_augroup.set(gettext(c"--Deleted--".as_ptr()));
        }
        deleted_augroup.get()
    }
}

/// The id of the group called `name`, creating one if there is not
/// already an id for that name.
pub unsafe extern "C" fn augroup_add(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        debug_assert!(strcasecmp(name, c"end".as_ptr()) != 0);

        let existing_id = augroup_find(name);
        if existing_id > 0 {
            debug_assert!(existing_id != AUGROUP_DELETED);
            return existing_id;
        }

        // The name still maps to `AUGROUP_DELETED` from an earlier
        // `:augroup! name`; that mapping has to go before a fresh id can
        // take the name.
        if existing_id == AUGROUP_DELETED {
            augroup_map_del(existing_id, name);
        }

        let next_id = next_augroup_id.get();
        next_augroup_id.set(next_id + 1);
        // The two maps each own their copy of the name.
        map_put_String_int(map_augroup_name_to_id.ptr(), cstr_to_string(name), next_id);
        map_put_int_String(map_augroup_id_to_name.ptr(), next_id, cstr_to_string(name));
        next_id
    }
}

/// Delete the group called `name`.
///
/// `stupid_legacy_mode` is `:augroup! {name}`: a group that still holds
/// autocommands keeps them and is merely renamed `--Deleted--`, leaving
/// them defined and unreachable (O-B14-2).  Everywhere else the
/// autocommands go with the group.
pub unsafe extern "C" fn augroup_del(name: *mut ::core::ffi::c_char, stupid_legacy_mode: bool) {
    unsafe {
        let group = augroup_find(name);
        if group == AUGROUP_ERROR {
            semsg_c!(gettext(c"E367: No such group: \"%s\"".as_ptr()), name);
            return;
        } else if group == current_augroup.get() {
            emsg(gettext(c"E936: Cannot delete the current group".as_ptr()));
            return;
        }

        for event in 0..NUM_EVENTS {
            let acs = au_event_vec(event);
            let mut i: usize = 0;
            // `(*acs).size` is re-read every step: `aucmd_del` only marks a
            // row deleted, but nothing here may assume the list is frozen.
            while i < (*acs).size {
                let ac = (*acs).items.add(i);
                let ap = (*ac).pat;
                if !ap.is_null() && (*ap).group == group {
                    if stupid_legacy_mode {
                        give_warning(
                            gettext(c"W19: Deleting augroup that is still in use".as_ptr()),
                            true,
                            true,
                        );
                        // Re-point the *name* at the deleted-group id and
                        // give up the old id, leaving the autocommands on it.
                        map_put_String_int(
                            map_augroup_name_to_id.ptr(),
                            cstr_as_string(name),
                            AUGROUP_DELETED,
                        );
                        augroup_map_del((*ap).group, ::core::ptr::null());
                        return;
                    }
                    aucmd_del(ac);
                }
                i = i.wrapping_add(1);
            }
        }

        // Nothing is using the group, so it can go for real.
        augroup_map_del(group, name);
        au_cleanup();
    }
}

/// The id of the group called `name`, or `AUGROUP_ERROR` when there is
/// none.  `AUGROUP_DELETED` is an answer of its own: the name is known and
/// belongs to a group `:augroup!` renamed.
pub unsafe extern "C" fn augroup_find(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let existing_id = map_get_String_int(map_augroup_name_to_id.ptr(), cstr_as_string(name));
        if existing_id == AUGROUP_DELETED || existing_id > 0 {
            existing_id
        } else {
            AUGROUP_ERROR
        }
    }
}

/// The name of group `group`, or null when no group ever had that id.
///
/// `next_augroup_id` is the source of truth about which ids have existed:
/// the map shrinks when a group is deleted, so its size is not.  The id
/// one past the last is spelled `END`, which is what makes `:augroup`
/// completion terminate.
pub unsafe extern "C" fn augroup_name(mut group: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    unsafe {
        debug_assert!(group != 0);

        if group == AUGROUP_DELETED {
            return get_deleted_augroup().cast_mut();
        }
        if group == AUGROUP_ALL {
            group = current_augroup.get();
        }
        if group == next_augroup_id.get() {
            return c"END".as_ptr().cast_mut();
        }
        if group > next_augroup_id.get() {
            return ::core::ptr::null_mut();
        }

        let key = map_get_int_String(map_augroup_id_to_name.ptr(), group);
        if !key.data.is_null() {
            return key.data;
        }
        // The id existed but is no longer in the map, so it was deleted.
        get_deleted_augroup().cast_mut()
    }
}

/// Whether a group called `name` exists.
pub unsafe extern "C" fn augroup_exists(name: *const ::core::ffi::c_char) -> bool {
    unsafe { augroup_find(name) > 0 }
}

/// `:augroup`: switch to a group, leave one, delete one, or list them.
pub unsafe extern "C" fn do_augroup(arg: *mut ::core::ffi::c_char, del_group: bool) {
    unsafe {
        if del_group {
            if *arg == 0 {
                emsg(gettext((&raw const e_argreq).cast::<::core::ffi::c_char>()));
            } else {
                augroup_del(arg, true);
            }
        } else if strcasecmp(arg, c"end".as_ptr()) == 0 {
            current_augroup.set(AUGROUP_DEFAULT);
        } else if *arg != 0 {
            current_augroup.set(augroup_add(arg));
        } else {
            msg_start();
            msg_ext_set_kind(c"list_cmd".as_ptr());
            let map = map_augroup_name_to_id.ptr();
            for i in 0..(*map).set.h.n_keys {
                let name = *(*map).set.keys.add(i as usize);
                let value = *(*map).values.add(i as usize);
                // A group `:augroup!` renamed lists as `--Deleted--`; its
                // key is still the old name.
                if value > 0 {
                    msg_puts(name.data);
                } else {
                    msg_puts(augroup_name(value));
                }
                msg_puts(c"  ".as_ptr());
            }
            msg_clr_eos();
            msg_end();
        }
    }
}

/// Completion source for a group name: [`augroup_name`] answers null once
/// `idx` runs past the last id.
pub unsafe extern "C" fn expand_get_augroup_name(
    _xp: *mut expand_T,
    idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe { augroup_name(idx + 1) }
}

/// Take a leading group name off `*argp`, answering its id.
///
/// A name that is not a group is *not* consumed and answers
/// `AUGROUP_ALL`, which is how `:autocmd BufEnter …` is told from
/// `:autocmd MyGroup BufEnter …` without a lookahead.
pub(crate) unsafe extern "C" fn arg_augroup_get(
    argp: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let arg = *argp;
        let bytes = CStr::from_ptr(arg).to_bytes();
        let len = bytes
            .iter()
            .position(|&c| ascii_iswhite(c as ::core::ffi::c_int) || c == b'|')
            .unwrap_or(bytes.len());
        if len == 0 {
            return AUGROUP_ALL;
        }

        let group_name =
            xmemdupz(arg.cast::<::core::ffi::c_void>(), len).cast::<::core::ffi::c_char>();
        let mut group = augroup_find(group_name);
        if group == AUGROUP_ERROR {
            group = AUGROUP_ALL;
        } else {
            *argp = skipwhite(arg.add(len));
        }
        xfree(group_name.cast::<::core::ffi::c_void>());
        group
    }
}
