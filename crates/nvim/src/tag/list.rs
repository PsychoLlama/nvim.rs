//! Showing the matches to the user.
//!
//! [`print_tag_list`] is the numbered listing `:tselect` prompts with — one
//! match per entry, with its priority, kind, name, file and command laid
//! out in columns. [`add_llist_tags`] is the same information as a location
//! list for `:ltag`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::file_search::Name;
use crate::highlight_group::{HLF_CM, HLF_D, HLF_T};
use crate::pos::MAXCOL;
use crate::types::OK;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// The column the kind, name and command text line up at.
const INFO_COLUMN: c_int = 15;

/// The narrowest the tag-name column is ever made.
const MIN_TAG_WIDTH: c_int = 18;

/// A `:ltag` entry's tag name is truncated to this many bytes.
const MAX_LTAG_NAME: usize = 128;

/// A `:ltag` search pattern is built in a buffer this big.
const CMDBUFFSIZE: usize = super::CMDBUFFSIZE as usize;

/// How a match was found, in the order the match's bucket byte numbers
/// them: F(ull, not a partial match) / S(tatic, local to its file) /
/// C(ase matched).
const PRIORITY: [&CStr; 8] = [
    c"FSC", c"F C", c"F  ", c"FS ", c" SC", c"  C", c"   ", c" S ",
];

/// The `:tselect` listing: every match, one entry each.
///
/// `new_tag` says this is a fresh listing rather than a re-listing of the
/// matches already jumped into, in which case one entry is marked as the
/// current one — taken from the preview tag, or from the tag stack when
/// `use_tagstack`.
///
/// # Safety
/// `matches` must hold `num_matches` matches [`find_tags`] answered, and
/// there must be at least one.
pub(crate) unsafe fn print_tag_list(
    new_tag: bool,
    use_tagstack: bool,
    num_matches: c_int,
    matches: *mut *mut c_char,
) {
    // SAFETY: the caller's promise; `curwin` is live, and each match
    // outlives the `TagParts` taken from it.
    unsafe {
        let mut tagp = TagParts::default();

        // Take the first match for how wide the names are, and line the
        // file names up at that.
        parse_match(*matches, &mut tagp);
        let name_width = tagp.tagname_end.offset_from(tagp.tagname) as c_int + 2;
        let mut taglen = name_width.max(MIN_TAG_WIDTH);
        if taglen > Columns.get() - 25 {
            // Too wide to line up: every file name goes on its own line.
            taglen = MAXCOL as c_int;
        }

        if msg_col.get() == 0 {
            // Overwrite the previous message.
            msg_didout.set(false);
        }
        msg_ext_set_kind(c"confirm".as_ptr());
        msg_start();
        msg_puts_hl(gettext(c"  # pri kind tag".as_ptr()), HLF_T, false);
        msg_clr_eos();
        advance_to_files(taglen);
        msg_puts_hl(gettext(c"file\n".as_ptr()), HLF_T, false);

        'each: for i in 0..num_matches {
            if got_int.get() {
                break;
            }
            let entry = *matches.offset(i as isize);
            parse_match(entry, &mut tagp);

            let current = !new_tag && is_current(i, use_tagstack);
            print_entry_head(i, entry, &tagp, current, taglen);
            if msg_col.get() > 0 {
                msg_putchar('\n' as c_int);
            }
            if got_int.get() {
                break;
            }
            msg_advance(INFO_COLUMN);

            // Where the command stops and the extra fields begin.
            let command_end = if tagp.command_end.is_null() {
                command_text_end(tagp.command)
            } else {
                if !print_extra_fields(&tagp) {
                    break 'each;
                }
                tagp.command_end
            };
            print_command(&tagp, command_end);

            // The last entry needs no line ending of its own when the UI
            // draws the messages itself.
            if msg_col.get() != 0 && (!ui_has(kUIMessages) || i < num_matches - 1) {
                msg_putchar('\n' as c_int);
            }
            os_breakcheck();
        }
        if got_int.get() {
            // Only stop the listing, not whatever asked for it.
            got_int.set(false);
        }
    }
}

/// Whether entry `i` is the match currently jumped to.
///
/// # Safety
/// `curwin` must be live.
unsafe fn is_current(i: c_int, use_tagstack: bool) -> bool {
    // SAFETY: the caller's promise.
    unsafe {
        if g_do_tagpreview.get() != 0 && i == (*ptag_entry.ptr()).cur_match {
            return true;
        }
        if !use_tagstack {
            return false;
        }
        // The index is one past the end when nothing has been popped;
        // upstream reads that slot anyway.
        let win = curwin.get();
        let at = (*win).w_tagstackidx as usize;
        (*win)
            .w_tagstack
            .get(at)
            .is_some_and(|entry| i == entry.cur_match)
    }
}

/// Print the number, priority, kind, name and file of one match.
///
/// # Safety
/// `entry` must be the match `tagp` was parsed from.
unsafe fn print_entry_head(
    i: c_int,
    entry: *const c_char,
    tagp: &TagParts,
    current: bool,
    taglen: c_int,
) {
    // SAFETY: the caller's promise. The number and priority are formatted
    // into `IObuff`, which truncates them where upstream truncates them.
    unsafe {
        let buf = IObuff.ptr().cast::<c_char>();
        *buf = if current { b'>' } else { b' ' } as c_char;
        vim_snprintf(
            buf.add(1),
            (IOSIZE - 1) as size_t,
            c"%2d %s ".as_ptr(),
            i + 1,
            PRIORITY[(*entry as c_int & MT_MASK as c_int) as usize].as_ptr(),
        );
        msg_puts(buf);

        if !tagp.tagkind.is_null() {
            let len = tagp.tagkind_end.offset_from(tagp.tagkind) as c_int;
            msg_outtrans_len(tagp.tagkind, len, 0, false);
        }
        msg_advance(13);
        let len = tagp.tagname_end.offset_from(tagp.tagname) as c_int;
        msg_outtrans_len(tagp.tagname, len, HLF_T, false);
        msg_putchar(' ' as c_int);
        advance_to_files(taglen);

        let fname = tag_full_fname(tagp);
        if !fname.is_null() {
            msg_outtrans(fname, HLF_D, false);
            xfree(fname.cast());
        }
    }
}

/// Print the `field:value` pairs after a tag's command.
///
/// Answers `false` when the user interrupted, which ends the listing.
///
/// # Safety
/// `tagp.command_end` must be non-NULL and point into the match.
unsafe fn print_extra_fields(tagp: &TagParts) -> bool {
    // SAFETY: the caller's promise; every scan stops at a line ending or
    // the terminator.
    unsafe {
        // Past the `;"` and the separator after it.
        let mut p: *const c_char = tagp.command_end.wrapping_add(3);
        while !matches!(*p as u8, 0 | b'\r' | b'\n') {
            while *p == TAB as c_char {
                p = p.add(1);
            }

            // Skip "file:" with no value: that is the static-tag marker,
            // and the priority column already says so.
            if strncmp(p, c"file:".as_ptr(), 5) == 0 && ascii_isspace(*p.add(5) as c_int) {
                p = p.add(5);
                continue;
            }
            // Skip "kind:<kind>" and a bare "<kind>": the kind has its own
            // column.
            if p == tagp.tagkind
                || (p.wrapping_add(5) == tagp.tagkind && strncmp(p, c"kind:".as_ptr(), 5) == 0)
            {
                p = tagp.tagkind_end;
                continue;
            }

            // Everything else is printed, the field name highlighted up to
            // its colon.
            let mut hl_id = HLF_CM;
            while !matches!(*p as u8, 0 | b'\r' | b'\n') {
                if msg_col.get() + ptr2cells(p) >= Columns.get() {
                    msg_putchar('\n' as c_int);
                    if got_int.get() {
                        break;
                    }
                    msg_advance(INFO_COLUMN);
                }
                p = msg_outtrans_one(p, hl_id, false);
                if *p == TAB as c_char {
                    msg_puts_hl(c" ".as_ptr(), hl_id, false);
                    break;
                }
                if *p == b':' as c_char {
                    hl_id = 0;
                }
            }
        }
        if msg_col.get() > INFO_COLUMN {
            msg_putchar('\n' as c_int);
            if got_int.get() {
                return false;
            }
            msg_advance(INFO_COLUMN);
        }
        true
    }
}

/// Print the command that locates the tag, at [`INFO_COLUMN`].
///
/// The `/^` or `?^` a search pattern opens with, and the `$/;"` it closes
/// with, are punctuation rather than part of the line, and are not shown.
///
/// # Safety
/// `tagp.command` must be NUL-terminated, and `command_end` must point
/// into it.
unsafe fn print_command(tagp: &TagParts, command_end: *const c_char) {
    // SAFETY: the caller's promise; the walk stops at `command_end`.
    unsafe {
        let delim = *tagp.command;
        let mut p: *const c_char = tagp.command;
        if matches!(delim as u8, b'/' | b'?') {
            p = p.add(1);
            if *p == b'^' as c_char {
                p = p.add(1);
            }
        }
        // Leading whitespace in the pattern is not worth a column.
        while p != command_end && ascii_isspace(*p as c_int) {
            p = p.add(1);
        }

        while p != command_end {
            let width = if *p == TAB as c_char { 1 } else { ptr2cells(p) };
            if msg_col.get() + width > Columns.get() {
                msg_putchar('\n' as c_int);
            }
            if got_int.get() {
                break;
            }
            msg_advance(INFO_COLUMN);

            // A backslash escaping the delimiter or another backslash is
            // punctuation too.
            if *p == b'\\' as c_char && (*p.add(1) == delim || *p.add(1) == b'\\' as c_char) {
                p = p.add(1);
            }
            if *p == TAB as c_char {
                msg_putchar(' ' as c_int);
                p = p.add(1);
            } else {
                p = msg_outtrans_one(p, 0, false);
            }

            // Stop before the `$/` or `$?` that closes an anchored pattern.
            if p == command_end.wrapping_sub(2) && *p == b'$' as c_char && *p.add(1) == delim {
                break;
            }
            // ... or before the closing delimiter on its own.
            if p == command_end.wrapping_sub(1) && *p == delim && matches!(delim as u8, b'/' | b'?')
            {
                break;
            }
        }
    }
}

/// Where a command with no extra fields after it ends.
///
/// # Safety
/// `command` must be NUL-terminated.
unsafe fn command_text_end(command: *const c_char) -> *const c_char {
    // SAFETY: the caller's promise.
    unsafe {
        let mut p = command;
        while !matches!(*p as u8, 0 | b'\r' | b'\n') {
            p = p.add(1);
        }
        p
    }
}

/// Move to the column the file names start at.
///
/// A `taglen` of `MAXCOL` means the names were too wide to line up, so the
/// file goes on a line of its own.
///
/// # Safety
/// Message output must be in progress.
unsafe fn advance_to_files(taglen: c_int) {
    // SAFETY: the caller's promise.
    unsafe {
        if taglen == MAXCOL as c_int {
            msg_putchar('\n' as c_int);
            msg_advance(24);
        } else {
            msg_advance(13 + taglen);
        }
    }
}

/// `:ltag` — put the matches in the current window's location list.
///
/// Each entry carries the tag's name, its file, and either the line number
/// or a very-nomagic form of the search pattern that finds it.
///
/// # Safety
/// `tag` must be NUL-terminated and `matches` must hold `num_matches`
/// matches [`find_tags`] answered.
pub(crate) unsafe fn add_llist_tags(
    tag: *mut c_char,
    num_matches: c_int,
    matches: *mut *mut c_char,
) -> c_int {
    // SAFETY: the caller's promise; each match outlives the `TagParts`
    // taken from it, and the list is handed to `set_errorlist` before it
    // is freed.
    unsafe {
        let list = tv_list_alloc(0);
        let mut tagp = TagParts::default();

        for i in 0..num_matches {
            parse_match(*matches.offset(i as isize), &mut tagp);

            let name_len = (tagp.tagname_end.offset_from(tagp.tagname) as usize).min(MAX_LTAG_NAME);
            let name = Name::from_bytes(core::slice::from_raw_parts(
                tagp.tagname.cast::<u8>(),
                name_len,
            ));

            let full_fname = tag_full_fname(&tagp);
            if full_fname.is_null() {
                continue;
            }
            // Upstream copies it into a `MAXPATHL` buffer; the truncation
            // is kept.
            let bytes = CStr::from_ptr(full_fname).to_bytes();
            let fname = Name::from_bytes(&bytes[..bytes.len().min(MAXPATHL as usize - 1)]);
            xfree(full_fname.cast());

            // A command that starts with a digit is a line number;
            // anything else is a search pattern.
            let lnum = if ascii_isdigit(*tagp.command as u8 as c_int) {
                atoi(tagp.command) as linenr_T
            } else {
                0
            };
            let pattern = (lnum == 0).then(|| search_pattern(&tagp));

            let dict = tv_dict_alloc();
            tv_list_append_dict(list, dict);
            add_str(dict, c"text", name.as_ptr());
            add_str(dict, c"filename", fname.as_ptr());
            tv_dict_add_nr(
                dict,
                c"lnum".as_ptr(),
                c"lnum".count_bytes(),
                lnum as varnumber_T,
            );
            if let Some(pattern) = &pattern {
                add_str(dict, c"pattern", pattern.as_ptr());
            }
        }

        vim_snprintf(
            IObuff.ptr().cast::<c_char>(),
            IOSIZE as size_t,
            c"ltag %s".as_ptr(),
            tag,
        );
        set_errorlist(
            curwin.get(),
            list,
            ' ' as c_int,
            IObuff.ptr().cast::<c_char>(),
            ptr::null_mut(),
        );
        tv_list_free(list);
        OK
    }
}

/// The location-list pattern that finds one tag: its search command, made
/// very nomagic so that nothing in it is read as a regexp.
///
/// # Safety
/// `tagp.command` must be NUL-terminated.
unsafe fn search_pattern(tagp: &TagParts) -> Name {
    // SAFETY: the caller's promise; the buffer is `CMDBUFFSIZE + 1` bytes
    // and every write into it is bounded.
    unsafe {
        let mut start = tagp.command;
        // Upstream steps back one from the end of the command, so this is
        // its *last* character rather than the one after.
        let mut end = if tagp.command_end.is_null() {
            command_text_end(tagp.command)
        } else {
            tagp.command_end
        }
        .sub(1);

        // Drop the delimiters a search pattern is wrapped in.
        if matches!(*start as u8, b'/' | b'?') {
            start = start.add(1);
        }
        if matches!(*end as u8, b'/' | b'?') {
            end = end.sub(1);
        }

        let mut cmd = vec![0 as c_char; CMDBUFFSIZE + 1];
        let mut len = 0;
        // A leading anchor has to stay in front of the \V.
        if *start == b'^' as c_char {
            cmd[len] = b'^' as c_char;
            len += 1;
            start = start.add(1);
        }
        cmd[len] = b'\\' as c_char;
        cmd[len + 1] = b'V' as c_char;
        len += 2;

        let text_len = (end.offset_from(start) as c_int + 1).min(CMDBUFFSIZE as c_int - 5);
        snprintf(
            cmd.as_mut_ptr().add(len),
            (CMDBUFFSIZE + 1 - len) as size_t,
            c"%.*s".as_ptr(),
            text_len,
            start,
        );
        len += text_len as usize;

        if cmd[len - 1] == b'$' as c_char {
            // A trailing '$' would anchor the pattern at the end of the
            // line, which the tags file did not ask for.
            cmd[len - 1] = b'\\' as c_char;
            cmd[len] = b'$' as c_char;
            len += 1;
        }
        Name::from_bytes(core::slice::from_raw_parts(cmd.as_ptr().cast::<u8>(), len))
    }
}

/// [`tv_dict_add_str`] with the key's length taken from the literal.
///
/// # Safety
/// `d` must be live and `val` NUL-terminated.
unsafe fn add_str(d: *mut dict_T, key: &CStr, val: *const c_char) {
    // SAFETY: the caller's promise.
    unsafe { tv_dict_add_str(d, key.as_ptr(), key.count_bytes(), val) };
}
