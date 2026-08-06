//! The event table's readers: name to number, number to name,
//! 'eventignore'.
//!
//! `event_name2nr` is the one parse -- it measures a name up to the first
//! whitespace, comma or bar and binary searches [`event_names`] for it, and
//! every other spelling of a lookup goes through it or its `String_0`
//! twin.  `event_ignored` and `check_ei` are the 'eventignore' half:
//! whether an event is currently suppressed, and whether an option value
//! is a list of real event names.  `au_event_disable`/`au_event_restore`
//! are the save-and-restore pair the rest of the editor brackets a
//! side-effecting operation with.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn event_name2nr(
    mut start: *const ::core::ffi::c_char,
    mut end: *mut *mut ::core::ffi::c_char,
) -> event_T {
    unsafe {
        let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        p = start;
        while *p as ::core::ffi::c_int != 0
            && !ascii_iswhite(*p as ::core::ffi::c_int)
            && *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int
            && *p as ::core::ffi::c_int != '|' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        let mut name_idx: ::core::ffi::c_int =
            event_name_index(start, p.offset_from(start) as size_t);
        if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            p = p.offset(1);
        }
        *end = p as *mut ::core::ffi::c_char;
        if name_idx < 0 as ::core::ffi::c_int {
            return NUM_EVENTS;
        }
        return abs((*event_names.ptr())[name_idx as usize].event) as event_T;
    }
}

pub unsafe extern "C" fn event_name2nr_str(mut str: String_0) -> event_T {
    unsafe {
        let mut name_idx: ::core::ffi::c_int = event_name_index(str.data, str.size);
        if name_idx < 0 as ::core::ffi::c_int {
            return NUM_EVENTS;
        }
        return abs((*event_names.ptr())[name_idx as usize].event) as event_T;
    }
}

pub unsafe extern "C" fn event_nr2name(mut event: event_T) -> *const ::core::ffi::c_char {
    unsafe {
        return if event as ::core::ffi::c_uint >= 0 as ::core::ffi::c_uint
            && (event as ::core::ffi::c_uint)
                < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*event_names.ptr())[event as usize].name as *const ::core::ffi::c_char
        } else {
            b"Unknown\0".as_ptr() as *const ::core::ffi::c_char
        };
    }
}

pub unsafe extern "C" fn event_ignored(
    mut event: event_T,
    mut ei: *mut ::core::ffi::c_char,
) -> bool {
    unsafe {
        let mut ignored: bool = false_0 != 0;
        while *ei as ::core::ffi::c_int != NUL {
            let mut unignore: bool = *ei as ::core::ffi::c_int == '-' as ::core::ffi::c_int;
            ei = ei.offset(unignore as ::core::ffi::c_int as isize);
            if strncasecmp(
                ei,
                b"all\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                3 as ::core::ffi::c_int as size_t,
            ) == 0 as ::core::ffi::c_int
                && (*ei.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || *ei.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ',' as ::core::ffi::c_int)
            {
                ignored = ei == p_ei.get()
                    || (*event_names.ptr())[event as usize].event <= 0 as ::core::ffi::c_int;
                ei = ei.offset(
                    (3 as ::core::ffi::c_int
                        + (*ei.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == ',' as ::core::ffi::c_int)
                            as ::core::ffi::c_int) as isize,
                );
            } else if event_name2nr(ei, &raw mut ei) as ::core::ffi::c_uint
                == event as ::core::ffi::c_uint
            {
                if unignore {
                    return false_0 != 0;
                }
                ignored = true_0 != 0;
            }
        }
        return ignored;
    }
}

pub unsafe extern "C" fn check_ei(mut ei: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let mut win: bool = ei != p_ei.get();
        while *ei != 0 {
            if strncasecmp(
                ei,
                b"all\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                3 as ::core::ffi::c_int as size_t,
            ) == 0 as ::core::ffi::c_int
                && (*ei.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || *ei.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ',' as ::core::ffi::c_int)
            {
                ei = ei.offset(
                    (3 as ::core::ffi::c_int
                        + (*ei.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == ',' as ::core::ffi::c_int)
                            as ::core::ffi::c_int) as isize,
                );
            } else {
                ei = ei.offset(
                    (*ei as ::core::ffi::c_int == '-' as ::core::ffi::c_int) as ::core::ffi::c_int
                        as isize,
                );
                let mut event: event_T = event_name2nr(ei, &raw mut ei);
                if event as ::core::ffi::c_uint
                    == NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint
                    || win as ::core::ffi::c_int != 0
                        && (*event_names.ptr())[event as usize].event > 0 as ::core::ffi::c_int
                {
                    return FAIL;
                }
            }
        }
        return OK;
    }
}

pub unsafe extern "C" fn au_event_disable(
    mut what: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut p_ei_len: size_t = strlen(p_ei.get());
        let mut save_ei: *mut ::core::ffi::c_char =
            xmemdupz(p_ei.get() as *const ::core::ffi::c_void, p_ei_len)
                as *mut ::core::ffi::c_char;
        let mut new_ei: *mut ::core::ffi::c_char =
            xstrnsave(p_ei.get(), p_ei_len.wrapping_add(strlen(what)));
        if *what as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            && *p_ei.get() as ::core::ffi::c_int == NUL
        {
            strcpy(new_ei, what.offset(1 as ::core::ffi::c_int as isize));
        } else {
            strcpy(new_ei.offset(p_ei_len as isize), what);
        }
        set_option_direct(
            kOptEventignore,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(new_ei),
                },
            },
            0 as ::core::ffi::c_int,
            SID_NONE,
        );
        xfree(new_ei as *mut ::core::ffi::c_void);
        return save_ei;
    }
}

pub unsafe extern "C" fn au_event_restore(mut old_ei: *mut ::core::ffi::c_char) {
    unsafe {
        if !old_ei.is_null() {
            set_option_direct(
                kOptEventignore,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string(old_ei),
                    },
                },
                0 as ::core::ffi::c_int,
                SID_NONE,
            );
            xfree(old_ei as *mut ::core::ffi::c_void);
        }
    }
}

pub unsafe extern "C" fn has_event(mut event: event_T) -> bool {
    unsafe {
        return (*autocmds.ptr())[event as ::core::ffi::c_int as usize].size != 0 as size_t;
    }
}

unsafe extern "C" fn has_cursorhold() -> bool {
    unsafe {
        return has_event(
            (if get_real_state() == MODE_NORMAL_BUSY {
                EVENT_CURSORHOLD as ::core::ffi::c_int
            } else {
                EVENT_CURSORHOLDI as ::core::ffi::c_int
            }) as event_T,
        );
    }
}

pub unsafe extern "C" fn trigger_cursorhold() -> bool {
    unsafe {
        if !did_cursorhold.get()
            && has_cursorhold() as ::core::ffi::c_int != 0
            && reg_recording.get() == 0 as ::core::ffi::c_int
            && (*typebuf.ptr()).tb_len == 0 as ::core::ffi::c_int
            && !ins_compl_active()
        {
            let mut state: ::core::ffi::c_int = get_real_state();
            if state == MODE_NORMAL_BUSY || state & MODE_INSERT != 0 as ::core::ffi::c_int {
                return true_0 != 0;
            }
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn expand_get_event_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut name: *mut ::core::ffi::c_char = augroup_name(idx + 1 as ::core::ffi::c_int);
        if !name.is_null() {
            if !autocmd_include_groups.get()
                || name == get_deleted_augroup() as *mut ::core::ffi::c_char
            {
                return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            return name;
        }
        let mut i: ::core::ffi::c_int = idx - next_augroup_id.get();
        if i < 0 as ::core::ffi::c_int || i >= NUM_EVENTS as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        return (*event_names.ptr())[i as usize].name;
    }
}

pub unsafe extern "C" fn get_event_name_no_group(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
    mut win: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if idx < 0 as ::core::ffi::c_int || idx >= NUM_EVENTS as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if !win {
            return (*event_names.ptr())[idx as usize].name;
        }
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < NUM_EVENTS as ::core::ffi::c_int {
            j += ((*event_names.ptr())[i as usize].event <= 0 as ::core::ffi::c_int)
                as ::core::ffi::c_int;
            if j == idx + 1 as ::core::ffi::c_int {
                return (*event_names.ptr())[i as usize].name;
            }
            i += 1;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn autocmd_supported(event: *const ::core::ffi::c_char) -> bool {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        return event_name2nr(event, &raw mut p) as ::core::ffi::c_uint
            != NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
}

/// ASCII-case-folded comparison, the order [`event_names`] is in.
///
/// Upstream reaches the same rows through a generated perfect hash
/// (`v0.12.4`'s `src/gen/gen_events.lua` calls `gen.hashy`, which emits a
/// ~400-line `switch` on the name's length and one discriminating byte,
/// plus a permutation table `event_hash` to undo the bucket order).  None
/// of that is needed: `gen_events.lua` sorts the names by `name:lower()`
/// before it numbers the enum, so [`event_names`] is *already* the sorted
/// table and a binary search over it answers the same lookup.
fn cmp_ignore_ascii_case(a: &[u8], b: &[u8]) -> Ordering {
    a.iter()
        .map(u8::to_ascii_lowercase)
        .cmp(b.iter().map(u8::to_ascii_lowercase))
}

/// The index into [`event_names`] of the event spelled by the first `len`
/// bytes of `name`, or -1 when no event is spelled that way.  ASCII case is
/// folded and the whole name has to match -- `len` is the length the caller
/// measured, so a name that merely starts with an event's name is not one.
unsafe fn event_name_index(name: *const ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int {
    unsafe {
        // `name` is only a valid pointer once there is a byte to read: the
        // `String_0` overload can hand this an empty, null-data string.
        let wanted: &[u8] = if len == 0 {
            &[]
        } else {
            slice::from_raw_parts(name.cast::<u8>(), len)
        };
        let rows = &*event_names.ptr();
        match rows.binary_search_by(|row| {
            cmp_ignore_ascii_case(CStr::from_ptr(row.name).to_bytes(), wanted)
        }) {
            Ok(at) => at as ::core::ffi::c_int,
            Err(_) => -(1 as ::core::ffi::c_int),
        }
    }
}
