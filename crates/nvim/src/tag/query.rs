//! The Vimscript and completion views of a tag.
//!
//! [`get_tags`] is `taglist()`: every match as a dictionary, one entry per
//! field of the tags line. [`expand_tags`] is the command-line completion
//! of tag names, which reshapes each match in place into the three
//! NUL-separated parts the completion display wants.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::message_fmt::c_str;
use crate::pos::MAXCOL;
use crate::smsg;
use crate::types::{Failed, MAXPATHL};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

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
) -> Result<(), Failed> {
    // SAFETY: the caller's promise; `find_tags` fills both out-parameters,
    // and the matches it answers are ours to rewrite and free.
    let mut flags = (TAG_REGEXP | TAG_VERBOSE | TAG_NO_TAGFUNC) as c_int;
    if tagnames {
        flags |= TAG_NAMES as c_int;
    }
    // A leading '/' asks for the rest to be read as a regexp; without
    // one the pattern is a literal name, and case is then not folded.
    let pat = if unsafe { *pat } == b'/' as c_char {
        unsafe { pat.add(1) }
    } else {
        flags |= TAG_NOIC as c_int;
        pat
    };

    let mincount = TAG_MANY as c_int;
    let buf_ffname = cur_buf().b_ffname;
    let ret = unsafe { find_tags(pat, num_file, file, flags, mincount, buf_ffname) };
    if ret.is_ok() && !tagnames {
        // One scratch buffer for the whole set, as upstream keeps.
        let mut head = Vec::with_capacity(128);
        for i in 0..unsafe { *num_file } as usize {
            unsafe { reshape_match(*(*file).add(i), &mut head) };
        }
    }
    ret
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
    let mut parts = TagParts::default();
    if !unsafe { parse_match(entry, &mut parts) } {
        // Not a tag line at all; upstream reads uninitialised pointers
        // here. Leaving it alone is the same answer without the crash.
        return;
    }

    // Built before anything is written, because it is read out of the
    // match itself.
    let name_len = unsafe { parts.tagname_end.offset_from(parts.tagname) } as usize;
    head.clear();
    head.extend_from_slice(unsafe { core::slice::from_raw_parts(parts.tagname, name_len) });
    head.push(0);
    // A match with no kind of its own is reported as a function.
    head.push(
        if !parts.tagkind.is_null() && unsafe { *parts.tagkind } != 0 {
            unsafe { *parts.tagkind }
        } else {
            b'f' as c_char
        },
    );
    head.push(0);

    let fname_len = unsafe { parts.fname_end.offset_from(parts.fname) } as usize;
    unsafe { ptr::copy(parts.fname, entry.add(head.len()), fname_len) };
    unsafe { *entry.add(head.len() + fname_len) = 0 };
    unsafe { ptr::copy(head.as_ptr(), entry, head.len()) };
}

/// `taglist()` — append a dictionary per match of `pat` to `list`.
///
/// `buf_fname` names the buffer whose matches sort first, or is NULL.
/// Answers `Ok`, or `Err` when a field could not be recorded.
///
/// # Safety
/// `list` must be live and `pat` NUL-terminated.
pub unsafe fn get_tags(
    list: *mut list_T,
    pat: *mut c_char,
    buf_fname: *mut c_char,
) -> Result<(), Failed> {
    // SAFETY: the caller's promise; `find_tags` fills both locals, and the
    // matches it answers become ours.
    let mut num_matches = 0;
    let mut matches = ptr::null_mut::<*mut c_char>();
    let num_matches2 = &raw mut num_matches;
    let matchesp = &raw mut matches;
    let flags2 = (TAG_REGEXP | TAG_NOIC) as c_int;
    let mincount = MAXCOL as c_int;
    let mut ret = unsafe { find_tags(pat, num_matches2, matchesp, flags2, mincount, buf_fname) };
    if ret.is_err() || num_matches <= 0 {
        return ret;
    }

    for i in 0..num_matches as usize {
        let entry = unsafe { *matches.add(i) };
        if !unsafe { describe_match(list, entry) } {
            ret = Err(Failed);
        }
        unsafe { xfree(entry.cast()) };
    }
    unsafe { xfree(matches.cast()) };
    ret
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
    let mut tp = TagParts::default();
    if !unsafe { parse_match(entry, &mut tp) } {
        return true;
    }
    let is_static = unsafe { test_for_static(&tp) };
    if unsafe { strncmp(tp.tagname, c"!_TAG_".as_ptr(), 6) } == 0 {
        // A pseudo-tag line: the file's own metadata, not a tag.
        return true;
    }

    let dict = unsafe { tv_dict_alloc() };
    unsafe { tv_list_append_dict(list, dict) };

    // Short-circuiting is upstream's: once one field fails, the rest
    // of these are not tried.
    let full_fname = unsafe { tag_full_fname(&tp) };
    // A kind with no text of its own runs to the end of the string.
    let kind_end = if tp.tagkind.is_null() {
        ptr::null()
    } else {
        tp.tagkind_end
    };
    let static_key = c"static";
    let mut ok = unsafe { add_tag_field(dict, c"name".as_ptr(), tp.tagname, tp.tagname_end) }
        .is_ok()
        && unsafe { add_tag_field(dict, c"filename".as_ptr(), full_fname, ptr::null()) }.is_ok()
        && unsafe { add_tag_field(dict, c"cmd".as_ptr(), tp.command, tp.command_end) }.is_ok()
        && unsafe { add_tag_field(dict, c"kind".as_ptr(), tp.tagkind, kind_end) }.is_ok()
        && unsafe {
            tv_dict_add_nr(
                dict,
                static_key.as_ptr(),
                static_key.count_bytes(),
                is_static as varnumber_T,
            )
        }
        .is_ok();
    unsafe { xfree(full_fname.cast()) };

    ok &= unsafe { add_extra_fields(dict, &tp) };
    ok
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
    // SAFETY: the caller's promise -- `command_end` points into the match,
    // which is NUL-terminated, and every step below stops at that NUL.
    // Past the `;"` and the separator after it.
    let mut p = unsafe { Scan::new(tp.command_end.wrapping_add(3)) };
    while !matches!(p.byte() as u8, 0 | b'\n' | b'\r') {
        if p.here() == tp.tagkind
            || (p.here().wrapping_add(5) == tp.tagkind && p.starts_with(c"kind:"))
        {
            // "kind:<kind>" or a bare "<kind>": already reported.
            p = p.at(tp.tagkind_end.wrapping_sub(1));
        } else if p.starts_with(c"file:") {
            // The static-tag marker, already reported.
            p.step(4);
        } else if !ascii_iswhite(p.byte() as c_int) {
            let name = p.here();
            // The field name, read through a *signed* char: a byte
            // above 0x7f reads negative and ends the name. The value
            // below is read unsigned; the two really do differ.
            while p.byte() != 0
                && (p.byte() as c_int) >= b' ' as c_int
                && (p.byte() as c_int) < 127
                && p.byte() != b':' as c_char
            {
                p.step(1);
            }
            let len = p.here().addr() - name.addr();
            if p.byte() == b':' as c_char && len > 0 {
                p.step(1);
                let value = p.here();
                while p.byte() != 0 && p.byte() as u8 >= b' ' {
                    p.step(1);
                }
                // Terminate the name for the call, then put the colon
                // back: the match is read on for the next field.
                // SAFETY: `name` is inside the match, `len` bytes before
                // the cursor, so the byte written is the field's colon.
                let colon = unsafe { *name.add(len) };
                unsafe { *name.add(len) = 0 };
                // SAFETY: as above; `name`, `value` and the cursor all
                // point into the same NUL-terminated match.
                let added = unsafe { add_tag_field(dict, name, value, p.here()) };
                if added.is_err() {
                    ok = false;
                }
                // SAFETY: as above.
                unsafe { *name.add(len) = colon };
            } else {
                // A field with no colon: pass over its text.
                while p.byte() != 0 && p.byte() as u8 >= b' ' {
                    p.step(1);
                }
            }
            if p.byte() == 0 {
                break;
            }
        }
        p.step(p.char_len());
    }
    ok
}

/// A cursor over one NUL-terminated match line.
///
/// The byte is read in one place, once per step, and everything else — the
/// loop, its `break`s, the arithmetic — is ordinary checked code. Building
/// one is the promise; `step` cannot leave the string because every caller
/// stops at the NUL it reports.
#[derive(Clone, Copy)]
struct Scan(*mut c_char);

impl Scan {
    /// # Safety
    /// `at` must point into a NUL-terminated string that outlives the value.
    unsafe fn new(at: *mut c_char) -> Self {
        Scan(at)
    }

    /// The same string, read from `at` instead.
    fn at(self, at: *mut c_char) -> Self {
        Scan(at)
    }

    /// Where the cursor stands.
    fn here(self) -> *mut c_char {
        self.0
    }

    /// The byte here, `0` at the end.
    fn byte(self) -> c_char {
        // SAFETY: the constructor's promise -- inside the string, and no
        // caller steps past the NUL this reports.
        unsafe { *self.0 }
    }

    /// Whether the text here starts with `what`.
    fn starts_with(self, what: &CStr) -> bool {
        let n = what.count_bytes();
        // SAFETY: as [`Scan::byte`]; `strncmp` stops at either NUL.
        unsafe { strncmp(self.0, what.as_ptr(), n) == 0 }
    }

    /// How many bytes the character here takes.
    fn char_len(self) -> usize {
        // SAFETY: as [`Scan::byte`].
        unsafe { utfc_ptr2len(self.0) as usize }
    }

    /// Move on `n` bytes, which must stay inside the string.
    fn step(&mut self, n: usize) {
        self.0 = self.0.wrapping_add(n);
    }
}

/// Add one field of a tag to the dictionary describing it.
///
/// `start` and `end` bracket the value; a NULL `end` means "to the end of
/// the string", less any trailing CR/NL. A NULL `start` records an empty
/// value. Answers `Err` on failure.
///
/// # Safety
/// `dict` must be live, `field_name` NUL-terminated, and `start` either
/// NULL or a readable string reaching `end`.
unsafe fn add_tag_field(
    dict: *mut dict_T,
    field_name: *const c_char,
    start: *const c_char,
    end: *const c_char,
) -> Result<(), Failed> {
    // SAFETY: the caller's promise.
    // A dictionary holds one value per key, so a field name the tags
    // line repeats is dropped rather than replacing the first.
    if !unsafe { tv_dict_find(dict, field_name, -1) }.is_null() {
        if p_verbose.get() > 0 {
            unsafe { verbose_enter() };
            // SAFETY: the message macros expand to a `vim_snprintf` over // the format literal above and the editor's message buffers.
            let field_name = unsafe { c_str(field_name) };
            smsg!(0, "Duplicate field name: {field_name}");
            unsafe { verbose_leave() };
        }
        return Err(Failed);
    }

    let mut value = Vec::with_capacity(MAXPATHL as usize);
    if !start.is_null() {
        let end = if end.is_null() {
            // Only an unbracketed value has its line ending trimmed.
            let mut end = unsafe { start.add(strlen(start)) };
            while end > start && matches!(unsafe { *end.sub(1) } as u8, b'\r' | b'\n') {
                end = unsafe { end.sub(1) };
            }
            end
        } else {
            end
        };
        let len = (unsafe { end.offset_from(start) } as usize).min(MAXPATHL as usize - 1);
        value.extend_from_slice(unsafe { core::slice::from_raw_parts(start, len) });
    }
    value.push(0);
    unsafe { tv_dict_add_str(dict, field_name, strlen(field_name), value.as_ptr()) }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
