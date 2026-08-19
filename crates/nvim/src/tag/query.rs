//! The Vimscript and completion views of a tag.
//!
//! [`get_tags`] is `taglist()`: every match as a dictionary, one entry per
//! field of the tags line. [`expand_tags`] is the command-line completion
//! of tag names, which reshapes each match in place into the three
//! NUL-separated parts the completion display wants.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::pos::MAXCOL;
use crate::smsg_c;
use crate::types::{FAIL, OK};
use core::ffi::{c_char, c_int};
use core::ptr;

/// A field's value is copied into a buffer this big; anything longer is
/// truncated, as upstream truncates it.
const MAXPATHL: usize = super::MAXPATHL as usize;

/// Command-line completion of tag names.
///
/// With `tagnames`, the matches are bare names (`:tag <Tab>`). Otherwise
/// each match is rewritten in place as `<name>NUL<kind>NUL<file name>NUL`,
/// which is the form the completion menu displays and matches against.
///
/// # Safety
/// `pat` must be NUL-terminated, and the two out-parameters must be
/// writable.
pub unsafe fn expand_tags(
    tagnames: bool,
    pat: *mut c_char,
    num_file: *mut c_int,
    file: *mut *mut *mut c_char,
) -> c_int {
    // SAFETY: the caller's promise; `find_tags` fills both out-parameters,
    // and the matches it answers are ours to rewrite and free.
    unsafe {
        let mut flags = (TAG_REGEXP | TAG_VERBOSE | TAG_NO_TAGFUNC) as c_int;
        if tagnames {
            flags |= TAG_NAMES as c_int;
        }
        // A leading '/' asks for the rest to be read as a regexp; without
        // one the pattern is a literal name, and case is then not folded.
        let pat = if *pat == b'/' as c_char {
            pat.add(1)
        } else {
            flags |= TAG_NOIC as c_int;
            pat
        };

        let ret = find_tags(
            pat,
            num_file,
            file,
            flags,
            TAG_MANY as c_int,
            (*curbuf.get()).b_ffname,
        );
        if ret == OK && !tagnames {
            // One scratch buffer for the whole set, as upstream keeps.
            let mut head = Vec::with_capacity(128);
            for i in 0..*num_file as usize {
                reshape_match(*(*file).add(i), &mut head);
            }
        }
        ret
    }
}

/// Rewrite one match in place as `<name>NUL<kind>NUL<file name>NUL`.
///
/// What replaces the match is built out of the match itself, and its tags
/// file name and command are dropped, so it always fits.
///
/// # Safety
/// `entry` must be a match [`find_tags`] answered, and ours to write.
unsafe fn reshape_match(entry: *mut c_char, head: &mut Vec<c_char>) {
    // SAFETY: the caller's promise. The two moves may overlap, which is
    // why they are `copy` and not `copy_nonoverlapping`.
    unsafe {
        let mut parts = TagParts::default();
        if !parse_match(entry, &mut parts) {
            // Not a tag line at all; upstream reads uninitialised pointers
            // here. Leaving it alone is the same answer without the crash.
            return;
        }

        // Built before anything is written, because it is read out of the
        // match itself.
        let name_len = parts.tagname_end.offset_from(parts.tagname) as usize;
        head.clear();
        head.extend_from_slice(core::slice::from_raw_parts(parts.tagname, name_len));
        head.push(0);
        // A match with no kind of its own is reported as a function.
        head.push(if !parts.tagkind.is_null() && *parts.tagkind != 0 {
            *parts.tagkind
        } else {
            b'f' as c_char
        });
        head.push(0);

        let fname_len = parts.fname_end.offset_from(parts.fname) as usize;
        ptr::copy(parts.fname, entry.add(head.len()), fname_len);
        *entry.add(head.len() + fname_len) = 0;
        ptr::copy(head.as_ptr(), entry, head.len());
    }
}

/// `taglist()` — append a dictionary per match of `pat` to `list`.
///
/// `buf_fname` names the buffer whose matches sort first, or is NULL.
/// Answers `OK`, or `FAIL` when a field could not be recorded.
///
/// # Safety
/// `list` must be live and `pat` NUL-terminated.
pub unsafe fn get_tags(list: *mut list_T, pat: *mut c_char, buf_fname: *mut c_char) -> c_int {
    // SAFETY: the caller's promise; `find_tags` fills both locals, and the
    // matches it answers become ours.
    unsafe {
        let mut num_matches = 0;
        let mut matches = ptr::null_mut::<*mut c_char>();
        let mut ret = find_tags(
            pat,
            &raw mut num_matches,
            &raw mut matches,
            (TAG_REGEXP | TAG_NOIC) as c_int,
            MAXCOL as c_int,
            buf_fname,
        );
        if ret != OK || num_matches <= 0 {
            return ret;
        }

        for i in 0..num_matches as usize {
            let entry = *matches.add(i);
            if !describe_match(list, entry) {
                ret = FAIL;
            }
            xfree(entry.cast());
        }
        xfree(matches.cast());
        ret
    }
}

/// Append one match to `list` as a dictionary.
///
/// Answers `false` only when a field could not be recorded. A match that
/// is not a tag line, or is one of a tags file's own `!_TAG_` header
/// lines, is silently passed over.
///
/// # Safety
/// `list` must be live and `entry` must be a match [`find_tags`] answered.
unsafe fn describe_match(list: *mut list_T, entry: *mut c_char) -> bool {
    // SAFETY: the caller's promise. Everything written into the dict is
    // copied out of the match, which outlives the call.
    unsafe {
        let mut tp = TagParts::default();
        if !parse_match(entry, &mut tp) {
            return true;
        }
        let is_static = test_for_static(&tp);
        if strncmp(tp.tagname, c"!_TAG_".as_ptr(), 6) == 0 {
            // A pseudo-tag line: the file's own metadata, not a tag.
            return true;
        }

        let dict = tv_dict_alloc();
        tv_list_append_dict(list, dict);

        // Short-circuiting is upstream's: once one field fails, the rest
        // of these are not tried.
        let full_fname = tag_full_fname(&tp);
        let mut ok = add_tag_field(dict, c"name".as_ptr(), tp.tagname, tp.tagname_end) == OK
            && add_tag_field(dict, c"filename".as_ptr(), full_fname, ptr::null()) == OK
            && add_tag_field(dict, c"cmd".as_ptr(), tp.command, tp.command_end) == OK
            && add_tag_field(
                dict,
                c"kind".as_ptr(),
                tp.tagkind,
                if tp.tagkind.is_null() {
                    ptr::null()
                } else {
                    tp.tagkind_end
                },
            ) == OK
            && tv_dict_add_nr(
                dict,
                c"static".as_ptr(),
                c"static".count_bytes(),
                is_static as varnumber_T,
            ) == OK;
        xfree(full_fname.cast());

        ok &= add_extra_fields(dict, &tp);
        ok
    }
}

/// Record the `field:value` pairs after a tag's command as dict entries.
///
/// The kind and the `file:` marker are passed over: both are already
/// reported under their own keys.
///
/// # Safety
/// `dict` must be live and `tp` must describe a live match.
unsafe fn add_extra_fields(dict: *mut dict_T, tp: &TagParts) -> bool {
    if tp.command_end.is_null() {
        return true;
    }
    let mut ok = true;
    // SAFETY: the caller's promise; every scan stops at the terminator,
    // and the NUL written over a field name's colon is put back.
    unsafe {
        // Past the `;"` and the separator after it.
        let mut p = tp.command_end.wrapping_add(3);
        while !matches!(*p as u8, 0 | b'\n' | b'\r') {
            if p == tp.tagkind
                || (p.wrapping_add(5) == tp.tagkind && strncmp(p, c"kind:".as_ptr(), 5) == 0)
            {
                // "kind:<kind>" or a bare "<kind>": already reported.
                p = tp.tagkind_end.sub(1);
            } else if strncmp(p, c"file:".as_ptr(), 5) == 0 {
                // The static-tag marker, already reported.
                p = p.add(4);
            } else if !ascii_iswhite(*p as c_int) {
                let name = p;
                // The field name, read through a *signed* char: a byte
                // above 0x7f reads negative and ends the name. The value
                // below is read unsigned; the two really do differ.
                while *p != 0
                    && (*p as c_int) >= b' ' as c_int
                    && (*p as c_int) < 127
                    && *p != b':' as c_char
                {
                    p = p.add(1);
                }
                let len = p.offset_from(name) as usize;
                if *p == b':' as c_char && len > 0 {
                    p = p.add(1);
                    let value = p;
                    while *p != 0 && *p as u8 >= b' ' {
                        p = p.add(1);
                    }
                    // Terminate the name for the call, then put the colon
                    // back: the match is read on for the next field.
                    let colon = *name.add(len);
                    *name.add(len) = 0;
                    if add_tag_field(dict, name, value, p) != OK {
                        ok = false;
                    }
                    *name.add(len) = colon;
                } else {
                    // A field with no colon: pass over its text.
                    while *p != 0 && *p as u8 >= b' ' {
                        p = p.add(1);
                    }
                }
                if *p == 0 {
                    break;
                }
            }
            p = p.add(utfc_ptr2len(p) as usize);
        }
    }
    ok
}

/// Add one field of a tag to the dictionary describing it.
///
/// `start` and `end` bracket the value; a NULL `end` means "to the end of
/// the string", less any trailing CR/NL. A NULL `start` records an empty
/// value. Answers `OK` or `FAIL`.
///
/// # Safety
/// `dict` must be live, `field_name` NUL-terminated, and `start` either
/// NULL or a readable string reaching `end`.
unsafe fn add_tag_field(
    dict: *mut dict_T,
    field_name: *const c_char,
    start: *const c_char,
    end: *const c_char,
) -> c_int {
    // SAFETY: the caller's promise.
    unsafe {
        // A dictionary holds one value per key, so a field name the tags
        // line repeats is dropped rather than replacing the first.
        if !tv_dict_find(dict, field_name, -1).is_null() {
            if p_verbose.get() > 0 {
                verbose_enter();
                smsg_c!(0, gettext(c"Duplicate field name: %s".as_ptr()), field_name);
                verbose_leave();
            }
            return FAIL;
        }

        let mut value = Vec::with_capacity(MAXPATHL);
        if !start.is_null() {
            let end = if end.is_null() {
                // Only an unbracketed value has its line ending trimmed.
                let mut end = start.add(strlen(start));
                while end > start && matches!(*end.sub(1) as u8, b'\r' | b'\n') {
                    end = end.sub(1);
                }
                end
            } else {
                end
            };
            let len = (end.offset_from(start) as usize).min(MAXPATHL - 1);
            value.extend_from_slice(core::slice::from_raw_parts(start, len));
        }
        value.push(0);
        tv_dict_add_str(dict, field_name, strlen(field_name), value.as_ptr())
    }
}
