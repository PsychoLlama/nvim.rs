//! The Vimscript and completion views of a tag.
//!
//! [`get_tags`] is `taglist()`: every match as a dictionary, one entry per
//! field of the tags line. [`tag_matches`] is the command-line completion
//! of tag names, which reshapes each match into the three NUL-separated
//! parts the completion display wants.

#![forbid(unsafe_code)]

use super::*;
use crate::cstr;
use crate::mbyte::c_cluster_len;
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::pos::MAXCOL;
use crate::smsg;
use crate::types::{Failed, MAXPATHL};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};

/// The byte `at` of a C string's bytes, NUL past the end.
fn byte(s: &[u8], at: usize) -> u8 {
    cstr::byte_at(s, at)
}

/// Command-line completion of tag names: the tags `pat` matches, owned.
///
/// With `tagnames`, the matches are bare names (`:tag <Tab>`). Otherwise
/// each match is `<name>NUL<kind>NUL<file name>`, which is the form the
/// completion menu displays and matches against, and the string keeps all
/// three.
pub(crate) fn tag_matches(tagnames: bool, pat: &CStr) -> Result<Vec<XString>, Failed> {
    let mut flags = (TAG_REGEXP | TAG_VERBOSE | TAG_NO_TAGFUNC) as c_int;
    // A leading '/' asks for the rest to be read as a regexp; without
    // one the pattern is a literal name, and case is then not folded.
    let pat = match pat.to_bytes() {
        [b'/', rest @ ..] => rest,
        pat => {
            flags |= TAG_NOIC as c_int;
            pat
        }
    };

    let mincount = TAG_MANY as c_int;
    // A copy: the search must not hold a borrow of the buffer.
    let buf_ffname = Buf::current().name.full().map(XString::from_cstr);
    let buf_ffname = buf_ffname.as_ref().map(XString::as_cstr);
    if tagnames {
        return find_tag_names(pat, flags, mincount, buf_ffname);
    }
    let found = find_tags(pat, flags, mincount, buf_ffname)?;
    Ok(found.iter().map(reshape_match).collect())
}

/// One match as `<name>NUL<kind>NUL<file name>`.
///
/// A match with no command of its own still has its name and file name,
/// and is reshaped all the same.
fn reshape_match(found: &TagMatch) -> XString {
    let (parts, _) = found.parse();
    let line = found.line_nul();
    let name = parts.tagname.of(line);
    let fname = parts.fname.of(line);
    // A match with no kind of its own is reported as a function.
    let kind = parts
        .tagkind
        .map(|kind| byte(line, kind.start))
        .filter(|&kind| kind != 0)
        .unwrap_or(b'f');

    let mut text = XString::with_capacity(name.len() + fname.len() + 3);
    text.push_bytes(name);
    text.push_bytes(&[0, kind, 0]);
    text.push_bytes(fname);
    text
}

/// `taglist()` — append a dictionary per match of `pat` to `list`.
///
/// `buf_fname` names the buffer whose matches sort first. Answers `Ok`, or
/// `Err` when a field could not be recorded. `'tagfunc'` may run, before
/// anything is appended.
pub fn get_tags(list: &mut List, pat: &CStr, buf_fname: Option<&CStr>) -> Result<(), Failed> {
    let flags = (TAG_REGEXP | TAG_NOIC) as c_int;
    let mincount = MAXCOL as c_int;
    let found = find_tags(pat.to_bytes(), flags, mincount, buf_fname)?;

    let mut ret = Ok(());
    for found in &found {
        if !describe_match(list, found) {
            ret = Err(Failed);
        }
    }
    ret
}

/// `text` less the CR and NL bytes it ends with.
fn trim_eol(mut text: &[u8]) -> &[u8] {
    while let [rest @ .., b'\r' | b'\n'] = text {
        text = rest;
    }
    text
}

/// Append one match to `list` as a dictionary.
///
/// Answers `false` only when a field could not be recorded. A match that
/// is not a tag line, or is one of a tags file's own `!_TAG_` header
/// lines, is silently passed over.
fn describe_match(list: &mut List, found: &TagMatch) -> bool {
    let (tp, complete) = found.parse();
    if !complete {
        return true;
    }
    let line = found.line_nul();
    let text = found.line();
    let is_static = test_for_static(text, &tp);
    if text.starts_with(b"!_TAG_") {
        // A pseudo-tag line: the file's own metadata, not a tag.
        return true;
    }

    let held = tv_dict_alloc();
    let dict = held.edit();

    // Short-circuiting is upstream's: once one field fails, the rest
    // of these are not tried.
    let full_fname = tag_full_fname(found, &tp);
    // A command with no extra fields runs to the end of the line.
    let command = match tp.command_end {
        Some(end) => &line[tp.command..end],
        None => trim_eol(&text[tp.command..]),
    };
    let mut ok = add_tag_field(dict, b"name", Some(tp.tagname.of(line))).is_ok()
        && add_tag_field(dict, b"filename", Some(trim_eol(&full_fname))).is_ok()
        && add_tag_field(dict, b"cmd", Some(command)).is_ok()
        && add_tag_field(dict, b"kind", tp.tagkind.as_ref().map(|kind| kind.of(line))).is_ok()
        && dict
            .add_number(b"static", VarNumber::from(is_static))
            .is_ok();

    ok &= add_extra_fields(dict, line, &tp);
    list.push_dict(Some(held));
    ok
}

/// Record the `field:value` pairs after a tag's command as dict entries.
/// `line` is NUL-terminated.
///
/// The kind and the `file:` marker are passed over: both are already
/// reported under their own keys.
fn add_extra_fields(dict: &mut Dict, line: &[u8], parts: &TagParts) -> bool {
    let Some(command_end) = parts.command_end else {
        return true;
    };
    let kind_start = parts.tagkind.as_ref().map(|kind| kind.start);
    let mut ok = true;
    // Past the `;"` and the separator after it.
    let mut p = command_end + 3;
    while !matches!(byte(line, p), 0 | b'\n' | b'\r') {
        if let Some(kind) = &parts.tagkind
            && (Some(p) == kind_start
                || (Some(p + 5) == kind_start && line[p..].starts_with(b"kind:")))
        {
            // "kind:<kind>" or a bare "<kind>": already reported.
            p = kind.end - 1;
        } else if line[p..].starts_with(b"file:") {
            // The static-tag marker, already reported.
            p += 4;
        } else if !ascii_iswhite(c_int::from(byte(line, p))) {
            let name = p;
            // The field name, read through a *signed* char: a byte
            // above 0x7f reads negative and ends the name. The value
            // below is read unsigned; the two really do differ.
            while (b' '..127).contains(&byte(line, p)) && byte(line, p) != b':' {
                p += 1;
            }
            let len = p - name;
            if byte(line, p) == b':' && len > 0 {
                p += 1;
                let value = p;
                while byte(line, p) >= b' ' {
                    p += 1;
                }
                if add_tag_field(dict, &line[name..name + len], Some(&line[value..p])).is_err() {
                    ok = false;
                }
            } else {
                // A field with no colon: pass over its text.
                while byte(line, p) >= b' ' {
                    p += 1;
                }
            }
            if byte(line, p) == 0 {
                break;
            }
        }
        p += c_cluster_len(&line[p..]);
    }
    ok
}

/// Add one field of a tag to the dictionary describing it.
///
/// `None` records an empty value. The value is cut where upstream's
/// `MAXPATHL` buffer cut it. Answers `Err` on failure.
fn add_tag_field(dict: &mut Dict, field_name: &[u8], value: Option<&[u8]>) -> Result<(), Failed> {
    // A dictionary holds one value per key, so a field name the tags
    // line repeats is dropped rather than replacing the first.
    if dict_find(Some(&*dict), field_name).is_some() {
        if p_verbose() > 0 {
            verbose_enter();
            let field_name = msg_bytes(field_name);
            smsg!(0, "Duplicate field name: {field_name}");
            verbose_leave();
        }
        return Err(Failed);
    }

    let value = value.unwrap_or_default();
    let value = &value[..value.len().min(MAXPATHL as usize - 1)];
    dict.add_str_len(field_name, Some(value))
}
