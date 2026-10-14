//! Reading a file name out of the buffer's text.
//!
//! [`file_name_in_line`] is what `gf` and its neighbours use: it finds the
//! run of `'isfname'` characters around a column, allows the extra
//! characters a URL needs, drops trailing punctuation, and picks up a
//! trailing `" line 99"`. [`find_file_name_in_path`] then looks the name up
//! along `'path'`, applying `'includeexpr'` when asked to or when the plain
//! lookup failed.

#![forbid(unsafe_code)]

use super::*;
use crate::charset::skip;
use crate::charset::{getdigits_at, getdigits_int_at};
use crate::guard::Script;
use crate::mbyte::{c_cluster_len, head_off};
use crate::memline::Lines;
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::normal::visual_active;
use crate::optionstr::OptString;
use crate::semsg;
use crate::strings::has_char;
use crate::types::{OptionSetFlags, Vv};
use crate::winlayer::{Buf, Win};
use core::ffi::{c_int, c_long};

/// The current buffer's full name, copied: the name is looked up after
/// `'includeexpr'` has run, and that may rename the buffer.
fn own_buffer_name() -> Option<XString> {
    Buf::current().name.full().map(XString::from_cstr)
}

/// The file name at the cursor, or the Visual selection when there is one.
///
/// The line number written after the name goes to `file_lnum`, when given.
pub(crate) fn grab_file_name(count: c_int, file_lnum: Option<&mut LineNr>) -> Option<XString> {
    let options = FileNameOpts::MESS | FileNameOpts::EXP | FileNameOpts::REL | FileNameOpts::UNESC;
    if !visual_active() {
        return file_name_at_cursor(options | FileNameOpts::HYP, count, file_lnum);
    }

    let selection = get_visual_text(None)?;
    let (text, len) = (selection.to_end_of_line().to_bytes(), selection.len());
    // Only recognize ":123" here. The bytes looked at are past the
    // selection but still on its line, which is why the selection carries
    // the rest of the line with it.
    if let Some(file_lnum) = file_lnum
        && cstr::byte_at(text, len) == b':'
        && cstr::byte_at(text, len + 1).is_ascii_digit()
    {
        *file_lnum = getdigits_int_at(text, len + 1, false, 0).0;
    }
    let rel_fname = own_buffer_name();
    find_file_name_in_path(
        &text[..len],
        options,
        c_long::from(count),
        rel_fname.as_ref().map(XString::as_cstr),
    )
}

/// The file name under or after the cursor.
///
/// `'path'` is searched when the name is not absolute. `None` when the file
/// name or the file is not found.
///
/// options:
/// - `FileNameOpts::MESS`  give error messages
/// - `FileNameOpts::EXP`   expand to path
/// - `FileNameOpts::HYP`   check for hypertext link
/// - `FileNameOpts::INCL`  apply `'includeexpr'`
///
/// The line number written after the name goes to `file_lnum`, when given.
pub(crate) fn file_name_at_cursor(
    options: FileNameOpts,
    count: c_int,
    file_lnum: Option<&mut LineNr>,
) -> Option<XString> {
    let cursor = Win::current().w_cursor;
    // The name is copied out of the line before anything can run that
    // might change the buffer.
    let name = {
        let mut lines = Lines::current();
        let line = lines.line(cursor.lnum);
        name_in_line(
            line,
            usize::try_from(cursor.col).unwrap_or(0),
            options,
            file_lnum,
        )?
    };
    let rel_fname = own_buffer_name();
    find_file_name_in_path(
        &name,
        options,
        c_long::from(count),
        rel_fname.as_ref().map(XString::as_cstr),
    )
}

/// Is the `":/"` (or `":\"`) that separates a URL's scheme from its path at
/// `line[at]`? [`path_is_url`] over a line.
fn url_at(line: &[u8], at: usize) -> bool {
    let rest = line.get(at..).unwrap_or_default();
    rest.starts_with(b":/") || rest.starts_with(b":\\")
}

/// Where the file name around `line[col]` starts, or `None` when the rest of
/// the line holds no `'isfname'` character at all.
///
/// Goes one character back to the `":"` before `"//"`, or to the drive letter
/// before `":\"`, even when `":"` is not in `'isfname'`.
fn name_start(line: &[u8], col: usize, options: FileNameOpts) -> Option<usize> {
    // Search forward for what could be the start of a file name.
    let mut at = col;
    while cstr::byte_at(line, at) != 0 && !vim_isfilec(c_int::from(line[at])) {
        at += c_cluster_len(&line[at..]);
    }
    if cstr::byte_at(line, at) == 0 {
        return None;
    }

    // Search backward for the first character of the file name.
    while at > 0 {
        let back = head_off(line, at - 1);
        if back > 0 {
            at -= back + 1;
        } else if vim_isfilec(c_int::from(line[at - 1]))
            || (options.has(FileNameOpts::HYP) && url_at(line, at - 1))
        {
            at -= 1;
        } else {
            break;
        }
    }
    Some(at)
}

/// How many bytes of `name` belong to the file name that starts there.
///
/// `":"`, `"?"`, `"&"` and `"="` join the name once a `type://` prefix has
/// been seen, so that `http://google.com:8080?q=this&that=ok` comes out
/// whole. `"\ "` is an escaped space and counts as two.
fn name_length(name: &[u8], options: FileNameOpts) -> usize {
    let hyp = options.has(FileNameOpts::HYP);
    let at = |i: usize| cstr::byte_at(name, i);
    // TODO(justinmk): Check for driveletter "x:/" at start, regardless of
    // 'isfname'.
    let mut len = if path_has_drive_letter(name) { 2 } else { 0 };
    let mut in_type = true;
    let mut is_url = false;
    loop {
        let escaped_space = at(len) == b'\\' && at(len + 1) == b' ';
        if !(vim_isfilec(c_int::from(at(len)))
            || escaped_space
            || (hyp && url_at(name, len))
            || (is_url && has_char(c":?&=", c_int::from(at(len)))))
        {
            break;
        }
        if at(len).is_ascii_alphabetic() {
            if in_type && url_at(name, len + 1) {
                is_url = true;
            }
        } else {
            in_type = false;
        }
        if escaped_space {
            len += 1; // skip over the "\" in "\ "
        }
        len += c_cluster_len(&name[len..]);
    }

    // If there is trailing punctuation, remove it. But don't remove "..",
    // which could be a directory name.
    if len > 2 && has_char(c".,:;!", c_int::from(name[len - 1])) && name[len - 2] != b'.' {
        len -= 1;
    }
    len
}

/// The line number written after a file name, as `" line 99"` or after any
/// single separator character. Both the English spelling and the translated
/// one are accepted, as `last_set_msg()` writes the latter.
fn trailing_line_number(after_name: &[u8]) -> Option<c_long> {
    let english: &[u8] = b" line ";
    let localized = gettext(line_msg).to_bytes();

    let mut p = if after_name.starts_with(english) {
        english.len()
    } else if after_name.starts_with(localized) {
        localized.len()
    } else {
        skip::white(after_name)
    };

    if cstr::byte_at(after_name, p) == 0 {
        return None;
    }
    if !after_name[p].is_ascii_digit() {
        p += 1; // skip the separator
    }
    p += skip::white(&after_name[p..]);
    cstr::byte_at(after_name, p)
        .is_ascii_digit()
        .then(|| getdigits_at(after_name, p, false, 0).0)
}

/// The name of the file under or after `line[col]`, copied out of the line,
/// with the line number written after it going to `file_lnum` when given.
/// `None`, said when `options` asks for messages, when there is none.
fn name_in_line(
    line: &[u8],
    col: usize,
    options: FileNameOpts,
    file_lnum: Option<&mut LineNr>,
) -> Option<Vec<u8>> {
    let Some(start) = name_start(line, col, options) else {
        if options.has(FileNameOpts::MESS) {
            emsg(gettext(c"E446: No file name under cursor"));
        }
        return None;
    };

    let name = &line[start..];
    let len = name_length(name, options);
    if let Some(file_lnum) = file_lnum
        && let Some(lnum) = trailing_line_number(&name[len..])
    {
        *file_lnum = lnum as LineNr;
    }
    Some(name[..len].to_vec())
}

/// The name of the file under or after `line[col]`, looked up in `'path'`.
///
/// `rel_fname` is the file we are searching relative to, and the line
/// number written after the name goes to `file_lnum`, when given. `line`
/// must be the caller's own: the lookup can run user code.
///
/// Otherwise like [`file_name_at_cursor`].
pub(crate) fn file_name_in_line(
    line: &[u8],
    col: usize,
    options: FileNameOpts,
    count: c_int,
    rel_fname: Option<&CStr>,
    file_lnum: Option<&mut LineNr>,
) -> Option<XString> {
    let name = name_in_line(line, col, options, file_lnum)?;
    find_file_name_in_path(&name, options, c_long::from(count), rel_fname)
}

/// Run `'includeexpr'` over `name`, with the name in `v:fname`.
pub(crate) fn eval_includeexpr(name: &[u8]) -> Option<XString> {
    set_vim_var_string(Vv::Fname, Some(name));
    // Errors go against the script that set `'includeexpr'`.
    let script_ctx = Script::context(Buf::current().b_p_script_ctx[kBufOptIncludeexpr as usize]);

    // A copy: the expression may set the option and free its text.
    let expr = Buf::current().b_p_inex.get();
    let sandbox = was_set_insecurely(Win::current(), kOptIncludeexpr, OptionSetFlags::LOCAL);
    let res = eval_to_string_safe(&expr, sandbox, true);

    set_vim_var_string(Vv::Fname, None);
    drop(script_ctx);
    res
}

/// The name of the file `name` in `'path'`.
///
/// Otherwise like [`file_name_at_cursor`]. `rel_fname` is the file we are
/// searching relative to; it and `name` must be the caller's own, as
/// `'includeexpr'` and the search can run user code.
pub(crate) fn find_file_name_in_path(
    name: &[u8],
    options: FileNameOpts,
    count: c_long,
    rel_fname: Option<&CStr>,
) -> Option<XString> {
    if name.is_empty() {
        return None;
    }

    // "file:/name" and "file://name" both name "/name"; a drive letter
    // after "file:/" keeps the slash.
    let mut name = name;
    if options.has(FileNameOpts::HYP)
        && name.len() > 6
        && name.starts_with(b"file:/")
        && !vim_ispathsep(c_int::from(name[6]))
    {
        let off = if path_has_drive_letter(&name[6..]) {
            6
        } else {
            5
        };
        name = &name[off..];
    }

    // What 'includeexpr' made of the name, when it ran.
    let mut included: Option<XString> = None;
    if options.has(FileNameOpts::INCL) && Buf::current().b_p_inex.first_byte() != 0 {
        included = eval_includeexpr(name);
    }

    if !options.has(FileNameOpts::EXP) {
        return Some(included.unwrap_or_else(|| XString::from_bytes(name)));
    }

    let mut walk = PathWalk::new();
    let quiet = options.without(FileNameOpts::MESS);
    let current = included.as_deref().unwrap_or(name);
    let mut file_name = find_file_in_path(&mut walk, current, quiet, rel_fname);

    // If the file could not be found in a normal way, try applying
    // 'includeexpr' (unless done already).
    if file_name.is_none()
        && !options.has(FileNameOpts::INCL)
        && Buf::current().b_p_inex.first_byte() != 0
        && let Some(expr) = eval_includeexpr(name)
    {
        let current = included.insert(expr);
        file_name = find_file_in_path(&mut walk, current, quiet, rel_fname);
    }
    let name = included.as_deref().unwrap_or(name);
    if file_name.is_none() && options.has(FileNameOpts::MESS) {
        let shown = msg_bytes(name);
        semsg!("E447: Can't find file \"{shown}\" in path");
    }

    // Repeat finding the file "count" times. This matters when it appears
    // several times in the path.
    //
    // Note the repeats pass `options` unmasked, so FileNameOpts::MESS
    // reaches the walk and its "No more file" message. Upstream.
    let mut count = count;
    while file_name.is_some() && {
        count -= 1;
        count > 0
    } {
        file_name = walk.next(options);
    }
    file_name
}
