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
use crate::message_fmt::c_str;
use crate::semsg;

/// The `name -> id` half of the augroup registry, by address: every `map_*`
/// operation the tree has takes one. It stays a khash — `:augroup` lists the
/// groups in the map's own order (F-P21-9).
fn augroup_by_name() -> *mut Map_String_int {
    map_augroup_name_to_id.ptr()
}

/// The `id -> name` half. See [`augroup_by_name`].
fn augroup_by_id() -> *mut Map_int_String {
    map_augroup_id_to_name.ptr()
}

/// Drop a group's entries from the two maps, freeing the keys they own.
///
/// A null `name` leaves the name map alone, which is how `:augroup! X`
/// releases the *id* of a group whose name it has just re-pointed at
/// `AUGROUP_DELETED`.
unsafe fn augroup_map_del(id: ::core::ffi::c_int, name: *const ::core::ffi::c_char) {
    if !name.is_null() {
        let mut key = String_0::NULL;
        // SAFETY: `name` is the caller's NUL-terminated string, non-null
        // here, and `cstr_as_string` only borrows its bytes for the call;
        // `key` is a live local the map writes the key it owned into.
        unsafe { map_del_string_int(augroup_by_name(), cstr_as_string(name), &raw mut key) };
        // SAFETY: `key` is the map's own copy, which it has just given up.
        unsafe { api_free_string(key) };
    }
    if id > 0 {
        // SAFETY: the two maps are the live augroup registry; a null
        // out-parameter is how `map_del` is told the key is not wanted.
        let mapped = unsafe { map_del_int_string(augroup_by_id(), id, ::core::ptr::null_mut()) };
        // SAFETY: `mapped` is the name the id map has just given up.
        unsafe { api_free_string(mapped) };
    }
}

/// The name a deleted-but-still-referenced group lists under, translated
/// once and cached.
///
/// Safe: it takes no pointer and only reads the cache and the message
/// catalogue, both of which are live for as long as the editor is.
#[inline(always)]
pub(crate) fn get_deleted_augroup() -> *const ::core::ffi::c_char {
    if deleted_augroup.get().is_null() {
        deleted_augroup.set(gettext(c"--Deleted--").as_ptr());
    }
    deleted_augroup.get()
}

/// The id of the group called `name`, creating one if there is not
/// already an id for that name.
pub unsafe fn augroup_add(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    // SAFETY: `name` is the caller's NUL-terminated string.
    debug_assert!(unsafe { strcasecmp(name, c"end".as_ptr()) } != 0);

    // SAFETY: as above -- `augroup_find` only reads through `name`.
    let existing_id = unsafe { augroup_find(name) };
    if existing_id > 0 {
        debug_assert!(existing_id != AUGROUP_DELETED);
        return existing_id;
    }

    // The name still maps to `AUGROUP_DELETED` from an earlier
    // `:augroup! name`; that mapping has to go before a fresh id can
    // take the name.
    if existing_id == AUGROUP_DELETED {
        // SAFETY: `name` is the caller's string, and `existing_id` is the
        // id the name map just answered for it.
        unsafe { augroup_map_del(existing_id, name) };
    }

    let next_id = next_augroup_id.get();
    next_augroup_id.set(next_id + 1);
    // The two maps each own their copy of the name.
    // SAFETY: `cstr_to_string` copies out of the caller's string, so each
    // map is handed an allocation of its own.
    unsafe { map_put_string_int(augroup_by_name(), cstr_to_string(name), next_id) };
    // SAFETY: as above.
    unsafe { map_put_int_string(augroup_by_id(), next_id, cstr_to_string(name)) };
    next_id
}

/// Delete the group called `name`.
///
/// `stupid_legacy_mode` is `:augroup! {name}`: a group that still holds
/// autocommands keeps them and is merely renamed `--Deleted--`, leaving
/// them defined and unreachable (O-B14-2).  Everywhere else the
/// autocommands go with the group.
pub unsafe fn augroup_del(name: *mut ::core::ffi::c_char, stupid_legacy_mode: bool) {
    // SAFETY: `name` is the caller's NUL-terminated string.
    let group = unsafe { augroup_find(name) };
    if group == AUGROUP_ERROR {
        // SAFETY: the message macros expand to a `vim_snprintf` over the
        // format literal above and the editor's message buffers.
        unsafe { semsg!("E367: No such group: \"{}\"", c_str(name)) };
        return;
    } else if group == current_augroup.get() {
        emsg(gettext(c"E936: Cannot delete the current group"));
        return;
    }

    for event in 0..NUM_EVENTS {
        let acs = au_event_vec(event);
        let mut i: usize = 0;
        // `(*acs).size` is re-read every step: `aucmd_del` only marks a
        // row deleted, but nothing here may assume the list is frozen.
        // SAFETY: `acs` is the event's own vector, which lives as long as
        // the editor does; `i` is below its size at every read below.
        while i < unsafe { (*acs).size } {
            let ac = unsafe { (*acs).items.add(i) };
            let ap = unsafe { (*ac).pat };
            // The deref stays on the right of the `&&`: a deleted row's
            // pattern is null.
            if !ap.is_null() && unsafe { (*ap).group } == group {
                if stupid_legacy_mode {
                    let warning = gettext(c"W19: Deleting augroup that is still in use");
                    // SAFETY: `warning` is the catalogue's own string.
                    unsafe { give_warning(warning.as_ptr(), true, true) };
                    // Re-point the *name* at the deleted-group id and
                    // give up the old id, leaving the autocommands on it.
                    // SAFETY: `name` is the caller's string, borrowed for
                    // the length of the put.
                    let key = unsafe { cstr_as_string(name) };
                    // SAFETY: the name map is the live registry.
                    unsafe { map_put_string_int(augroup_by_name(), key, AUGROUP_DELETED) };
                    // SAFETY: `ap` is the live pattern of row `i`, checked
                    // non-null above.
                    unsafe { augroup_map_del((*ap).group, ::core::ptr::null()) };
                    return;
                }
                // SAFETY: `ac` is row `i` of the vector, still in bounds.
                unsafe { aucmd_del(ac) };
            }
            i = i.wrapping_add(1);
        }
    }

    // Nothing is using the group, so it can go for real.
    // SAFETY: `name` is the caller's string and `group` the id it named.
    unsafe { augroup_map_del(group, name) };
    au_cleanup();
}

/// The id of the group called `name`, or `AUGROUP_ERROR` when there is
/// none.  `AUGROUP_DELETED` is an answer of its own: the name is known and
/// belongs to a group `:augroup!` renamed.
pub unsafe fn augroup_find(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    // SAFETY: `name` is the caller's NUL-terminated string, which
    // `cstr_as_string` only borrows for the length of the lookup; the name
    // map is the live registry.
    let existing_id = unsafe { map_get_string_int(augroup_by_name(), cstr_as_string(name)) };
    if existing_id == AUGROUP_DELETED || existing_id > 0 {
        existing_id
    } else {
        AUGROUP_ERROR
    }
}

/// The name of group `group`, or null when no group ever had that id.
///
/// `next_augroup_id` is the source of truth about which ids have existed:
/// the map shrinks when a group is deleted, so its size is not.  The id
/// one past the last is spelled `END`, which is what makes `:augroup`
/// completion terminate.
///
/// Safe: it takes an id rather than a pointer, and the string it answers
/// is owned by the id map or by the message catalogue, both of which
/// outlive the call.
pub fn augroup_name(mut group: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
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

    // SAFETY: the id map is the live registry; a miss answers the empty
    // `String_0`, whose `data` is null.
    let key = unsafe { map_get_int_string(augroup_by_id(), group) };
    if !key.data().is_null() {
        return key.data();
    }
    // The id existed but is no longer in the map, so it was deleted.
    get_deleted_augroup().cast_mut()
}

/// Whether a group called `name` exists.
pub unsafe fn augroup_exists(name: *const ::core::ffi::c_char) -> bool {
    unsafe { augroup_find(name) > 0 }
}

/// `:augroup`: switch to a group, leave one, delete one, or list them.
pub unsafe fn do_augroup(arg: *mut ::core::ffi::c_char, del_group: bool) {
    // SAFETY, for every region in this function: `arg` is the caller's
    // NUL-terminated string, so reading its first byte and comparing it
    // against a literal are both in bounds, and it stays live throughout.
    if del_group {
        if unsafe { *arg } == 0 {
            emsg(gettext(e_argreq));
        } else {
            // SAFETY: `arg` is the caller's string.
            unsafe { augroup_del(arg, true) };
        }
    } else if unsafe { strcasecmp(arg, c"end".as_ptr()) } == 0 {
        current_augroup.set(AUGROUP_DEFAULT);
    } else if unsafe { *arg } != 0 {
        // SAFETY: `arg` is the caller's string.
        current_augroup.set(unsafe { augroup_add(arg) });
    } else {
        // SAFETY: the message routines write to the editor's own message
        // buffers, which are live for as long as it is.
        unsafe { msg_start() };
        // SAFETY: a static literal names the message kind.
        unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
        let map = augroup_by_name();
        // SAFETY: `map` is the live name registry; `n_keys` is the length of
        // the `keys`/`values` arrays it is read alongside below.
        let n_keys = unsafe { (*map).set.h.n_keys };
        for i in 0..n_keys {
            // SAFETY: `i` is below `n_keys`, so both reads are in bounds.
            let name = unsafe { *(*map).set.keys.add(i as usize) };
            let value = unsafe { *(*map).values.add(i as usize) };
            // A group `:augroup!` renamed lists as `--Deleted--`; its
            // key is still the old name.
            if value > 0 {
                // SAFETY: `name` is the map's own key, still in the map.
                unsafe { msg_puts(name.data()) };
            } else {
                // SAFETY: `augroup_name` answers a string the id map or the
                // catalogue owns.
                unsafe { msg_puts(augroup_name(value)) };
            }
            // SAFETY: a static literal.
            unsafe { msg_puts(c"  ".as_ptr()) };
        }
        // SAFETY: the message buffers again.
        unsafe { msg_clr_eos() };
        unsafe { msg_end() };
    }
}

/// Completion source for a group name: [`augroup_name`] answers null once
/// `idx` runs past the last id.
pub unsafe fn expand_get_augroup_name(
    _xp: *mut expand_T,
    idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    augroup_name(idx + 1)
}

/// Take a leading group name off `*argp`, answering its id.
///
/// A name that is not a group is *not* consumed and answers
/// `AUGROUP_ALL`, which is how `:autocmd BufEnter …` is told from
/// `:autocmd MyGroup BufEnter …` without a lookahead.
pub(crate) unsafe fn arg_augroup_get(argp: *mut *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    // SAFETY: `argp` points at the caller's live `char *` slot, and what it
    // holds is a NUL-terminated string.
    let arg = unsafe { *argp };
    let bytes = unsafe { CStr::from_ptr(arg) }.to_bytes();
    let len = bytes
        .iter()
        .position(|&c| ascii_iswhite(c as ::core::ffi::c_int) || c == b'|')
        .unwrap_or(bytes.len());
    if len == 0 {
        return AUGROUP_ALL;
    }

    // SAFETY: `len` is an index into `arg`'s own bytes, so the copy is in
    // bounds; the result is a fresh NUL-terminated allocation.
    let group_name =
        unsafe { xmemdupz(arg.cast::<::core::ffi::c_void>(), len) }.cast::<::core::ffi::c_char>();
    // SAFETY: `group_name` is the allocation just made.
    let mut group = unsafe { augroup_find(group_name) };
    if group == AUGROUP_ERROR {
        group = AUGROUP_ALL;
    } else {
        // SAFETY: `len` is within `arg`, so `arg.add(len)` is at worst its
        // one-past-the-end NUL; `argp` is the caller's live slot.
        unsafe { *argp = skipwhite(arg.add(len)) };
    }
    // SAFETY: `group_name` is the allocation above, dead from here.
    unsafe { xfree(group_name.cast::<::core::ffi::c_void>()) };
    group
}
