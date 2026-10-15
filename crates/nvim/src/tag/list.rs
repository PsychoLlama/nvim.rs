//! Showing the matches to the user.
//!
//! [`print_tag_list`] is the numbered listing `:tselect` prompts with — one
//! match per entry, with its priority, kind, name, file and command laid
//! out in columns. [`add_llist_tags`] is the same information as a location
//! list for `:ltag`.

#![forbid(unsafe_code)]

use super::*;
use crate::charset::ptr2cells_at;
use crate::cstr;
use crate::debugger::atoi;
use crate::formatted;
use crate::highlight_group::{HLF_CM, HLF_D, HLF_T};
use crate::pos::MAXCOL;
use crate::types::{IOSIZE, MAXPATHL};
use crate::winlayer::Win;
use core::ffi::c_int;

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
const PRIORITY: [&[u8]; 8] = [
    b"FSC", b"F C", b"F  ", b"FS ", b" SC", b"  C", b"   ", b" S ",
];

/// The byte `at` of a C string's bytes, NUL past the end.
fn byte(s: &[u8], at: usize) -> u8 {
    cstr::byte_at(s, at)
}

/// The `:tselect` listing: every match, one entry each.
///
/// `new_tag` says this is a fresh listing rather than a re-listing of the
/// matches already jumped into, in which case one entry is marked as the
/// current one — taken from the preview tag, or from the tag stack when
/// `use_tagstack`. There must be at least one match.
pub(crate) fn print_tag_list(new_tag: bool, use_tagstack: bool, matches: &[TagMatch]) {
    // Take the first match for how wide the names are, and line the
    // file names up at that.
    let (first, _) = matches[0].parse();
    let name_width = c_int::try_from(first.tagname.len()).unwrap_or(c_int::MAX - 2) + 2;
    let mut taglen = name_width.max(MIN_TAG_WIDTH);
    if taglen > Columns.get() - 25 {
        // Too wide to line up: every file name goes on its own line.
        taglen = MAXCOL as c_int;
    }

    if msg_col.get() == 0 {
        // Overwrite the previous message.
        msg_didout.set(false);
    }
    msg_ext_set_kind(c"confirm");
    msg_start();
    msg_str_hl(gettext(c"  # pri kind tag"), HLF_T, false);
    msg_clr_eos();
    advance_to_files(taglen);
    msg_str_hl(gettext(c"file\n"), HLF_T, false);

    let count = matches.len();
    for (i, found) in matches.iter().enumerate() {
        if got_int.get() {
            break;
        }
        let (tagp, _) = found.parse();
        let line = found.line_nul();
        let number = c_int::try_from(i).unwrap_or(c_int::MAX);

        let current = !new_tag && is_current(number, use_tagstack);
        print_entry_head(number, found, &tagp, current, taglen);
        if msg_col.get() > 0 {
            msg_putchar('\n' as c_int);
        }
        if got_int.get() {
            break;
        }
        msg_advance(INFO_COLUMN);

        // Where the command stops and the extra fields begin.
        let command_end = match tagp.command_end {
            None => command_text_end(line, tagp.command),
            Some(end) => {
                if !print_extra_fields(line, end, &tagp) {
                    break;
                }
                end
            }
        };
        print_command(line, tagp.command, command_end);

        // The last entry needs no line ending of its own when the UI
        // draws the messages itself.
        if msg_col.get() != 0 && (!ui_has(kUIMessages) || i + 1 < count) {
            msg_putchar('\n' as c_int);
        }
        os_breakcheck();
    }
    if got_int.get() {
        // Only stop the listing, not whatever asked for it.
        got_int.set(false);
    }
}

/// Whether entry `i` is the match currently jumped to.
fn is_current(i: c_int, use_tagstack: bool) -> bool {
    if g_do_tagpreview.get() != 0 && i == ptag_entry.with(|entry| entry.cur_match) {
        return true;
    }
    if !use_tagstack {
        return false;
    }
    // The index is one past the end when nothing has been popped;
    // upstream reads that slot anyway.
    let win = Win::current();
    let at = usize::try_from(win.w_tagstackidx).unwrap_or(usize::MAX);
    win.w_tagstack
        .get(at)
        .is_some_and(|entry| i == entry.cur_match)
}

/// Print the number, priority, kind, name and file of one match.
fn print_entry_head(i: c_int, found: &TagMatch, tagp: &TagParts, current: bool, taglen: c_int) {
    let line = found.line();
    let mut head = vec![if current { b'>' } else { b' ' }];
    // Cut where upstream's `IOSIZE` scratch cut it.
    let priority = PRIORITY[usize::from(found.priority) & MT_MASK as usize];
    head.extend(formatted!(IOSIZE as usize - 1, b"%2d %s ", i + 1, priority));
    head.push(0);
    msg_str(cstr::in_bytes(&head));

    if let Some(kind) = &tagp.tagkind {
        msg_display_bytes(kind.of(line), 0, false);
    }
    msg_advance(13);
    msg_display_bytes(tagp.tagname.of(line), HLF_T, false);
    msg_putchar(' ' as c_int);
    advance_to_files(taglen);

    let fname = tag_full_fname(found, tagp);
    msg_display(fname.as_cstr(), HLF_D, false);
}

/// Print the `field:value` pairs after a tag's command, which ends at
/// `command_end` in the NUL-terminated `line`.
///
/// Answers `false` when the user interrupted, which ends the listing.
fn print_extra_fields(line: &[u8], command_end: usize, tagp: &TagParts) -> bool {
    let kind_start = tagp.tagkind.as_ref().map(|kind| kind.start);
    // Past the `;"` and the separator after it.
    let mut p = command_end + 3;
    while !matches!(byte(line, p), 0 | b'\r' | b'\n') {
        while byte(line, p) == TAB as u8 {
            p += 1;
        }

        // Skip "file:" with no value: that is the static-tag marker,
        // and the priority column already says so.
        if line[p..].starts_with(b"file:") && ascii_isspace(c_int::from(byte(line, p + 5))) {
            p += 5;
            continue;
        }
        // Skip "kind:<kind>" and a bare "<kind>": the kind has its own
        // column.
        if let Some(kind) = &tagp.tagkind
            && (Some(p) == kind_start
                || (Some(p + 5) == kind_start && line[p..].starts_with(b"kind:")))
        {
            p = kind.end;
            continue;
        }

        // Everything else is printed, the field name highlighted up to
        // its colon.
        let mut hl_id = HLF_CM;
        while !matches!(byte(line, p), 0 | b'\r' | b'\n') {
            if msg_col.get() + ptr2cells_at(&line[p..]) >= Columns.get() {
                msg_putchar('\n' as c_int);
                if got_int.get() {
                    break;
                }
                msg_advance(INFO_COLUMN);
            }
            p += msg_display_char(cstr::until_nul(&line[p..]), hl_id, false);
            if byte(line, p) == TAB as u8 {
                msg_str_hl(c" ", hl_id, false);
                break;
            }
            if byte(line, p) == b':' {
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

/// Print the command that locates the tag, at [`INFO_COLUMN`]: the bytes
/// of the NUL-terminated `line` from `command` to `command_end`.
///
/// The `/^` or `?^` a search pattern opens with, and the `$/;"` it closes
/// with, are punctuation rather than part of the line, and are not shown.
fn print_command(line: &[u8], command: usize, command_end: usize) {
    let delim = byte(line, command);
    let mut p = command;
    if matches!(delim, b'/' | b'?') {
        p += 1;
        if byte(line, p) == b'^' {
            p += 1;
        }
    }
    // Leading whitespace in the pattern is not worth a column.
    while p != command_end && ascii_isspace(c_int::from(byte(line, p))) {
        p += 1;
    }

    while p != command_end {
        let width = if byte(line, p) == TAB as u8 {
            1
        } else {
            ptr2cells_at(&line[p..])
        };
        if msg_col.get() + width > Columns.get() {
            msg_putchar('\n' as c_int);
        }
        if got_int.get() {
            break;
        }
        msg_advance(INFO_COLUMN);

        // A backslash escaping the delimiter or another backslash is
        // punctuation too.
        if byte(line, p) == b'\\' && (byte(line, p + 1) == delim || byte(line, p + 1) == b'\\') {
            p += 1;
        }
        if byte(line, p) == TAB as u8 {
            msg_putchar(' ' as c_int);
            p += 1;
        } else {
            p += msg_display_char(cstr::until_nul(&line[p..]), 0, false);
        }

        // Stop before the `$/` or `$?` that closes an anchored pattern.
        if Some(p) == command_end.checked_sub(2)
            && byte(line, p) == b'$'
            && byte(line, p + 1) == delim
        {
            break;
        }
        // ... or before the closing delimiter on its own.
        if Some(p) == command_end.checked_sub(1)
            && byte(line, p) == delim
            && matches!(delim, b'/' | b'?')
        {
            break;
        }
    }
}

/// Where a command with no extra fields after it ends: at a line ending or
/// the terminator.
fn command_text_end(line: &[u8], command: usize) -> usize {
    let mut p = command;
    while !matches!(byte(line, p), 0 | b'\r' | b'\n') {
        p += 1;
    }
    p
}

/// Move to the column the file names start at.
///
/// A `taglen` of `MAXCOL` means the names were too wide to line up, so the
/// file goes on a line of its own.
fn advance_to_files(taglen: c_int) {
    if taglen == MAXCOL as c_int {
        msg_putchar('\n' as c_int);
        msg_advance(24);
    } else {
        msg_advance(13 + taglen);
    }
}

/// `:ltag` — put the matches in the current window's location list.
///
/// Each entry carries the tag's name, its file, and either the line number
/// or a very-nomagic form of the search pattern that finds it.
pub(crate) fn add_llist_tags(tag: &[u8], matches: &[TagMatch]) {
    let held = tv_list_alloc(0);

    for found in matches {
        let (tagp, _) = found.parse();
        let line = found.line_nul();

        let name = tagp.tagname.of(line);
        let name = cstr::owned(&name[..name.len().min(MAX_LTAG_NAME)]);

        // Upstream copies it into a `MAXPATHL` buffer; the truncation
        // is kept.
        let full_fname = tag_full_fname(found, &tagp);
        let full_fname: &[u8] = &full_fname;
        let fname = cstr::owned(&full_fname[..full_fname.len().min(MAXPATHL as usize - 1)]);

        // A command that starts with a digit is a line number;
        // anything else is a search pattern.
        let lnum = if byte(line, tagp.command).is_ascii_digit() {
            atoi(&line[tagp.command..])
        } else {
            0
        };

        let dict = tv_dict_alloc();
        let entry = dict.edit();
        let _ = entry.add_str(b"text", Some(&name));
        let _ = entry.add_str(b"filename", Some(&fname));
        let _ = entry.add_number(b"lnum", VarNumber::from(lnum));
        if lnum == 0 {
            let _ = entry.add_str_len(b"pattern", Some(&search_pattern(line, &tagp)));
        }
        held.edit().push_dict(Some(dict));
    }

    // The list's title, cut where upstream's `IOSIZE` scratch cut it.
    let mut title = formatted!(IOSIZE as usize, b"ltag %s", tag);
    title.push(0);
    // Answers `Ok` for a plain entry list; upstream discarded it too.
    let _ = set_errorlist(
        Win::current_or_none(),
        Some(held),
        b' ',
        cstr::in_bytes(&title),
        None,
    );
}

/// The location-list pattern that finds one tag: its search command, made
/// very nomagic so that nothing in it is read as a regexp. `line` is
/// NUL-terminated.
fn search_pattern(line: &[u8], tagp: &TagParts) -> Vec<u8> {
    let mut start = tagp.command;
    // Upstream steps back one from the end of the command, so this is
    // its *last* character rather than the one after; for an empty
    // command that is the TAB before it.
    let mut end = tagp
        .command_end
        .unwrap_or_else(|| command_text_end(line, tagp.command))
        - 1;

    // Drop the delimiters a search pattern is wrapped in.
    if matches!(byte(line, start), b'/' | b'?') {
        start += 1;
    }
    if matches!(byte(line, end), b'/' | b'?') {
        end -= 1;
    }

    let mut cmd = Vec::with_capacity(CMDBUFFSIZE);
    // A leading anchor has to stay in front of the \V.
    if byte(line, start) == b'^' {
        cmd.push(b'^');
        start += 1;
    }
    cmd.extend_from_slice(b"\\V");

    // The text between the delimiters, at most `CMDBUFFSIZE - 5` bytes. A
    // command that is a lone delimiter has its last byte *before* its
    // first: upstream adds that negative length to the pattern's, which
    // leaves just the backslash of the `\V`.
    let text_len = (end + 1).cast_signed() - start.cast_signed();
    match usize::try_from(text_len) {
        Ok(len) => {
            let len = len.min(CMDBUFFSIZE - 5);
            cmd.extend_from_slice(&line[start..start + len]);
        }
        Err(_) => {
            let keep = cmd.len().checked_add_signed(text_len).unwrap_or(0);
            cmd.truncate(keep);
        }
    }

    if cmd.last() == Some(&b'$') {
        // A trailing '$' would anchor the pattern at the end of the
        // line, which the tags file did not ask for.
        cmd.pop();
        cmd.extend_from_slice(b"\\$");
    }
    cmd
}
