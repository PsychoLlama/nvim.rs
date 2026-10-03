//! Printing a list to the message area.
//!
//! [`qf_list`] is `:clist`, one line per entry via [`qf_list_entry`];
//! [`qf_age`] is `:colder`/`:cnewer`, [`qf_history`] is `:chistory` and
//! [`qf_view_result`] is what `CTRL-W_<Enter>` does in the quickfix window.
//!
//! Printing can run user code — a Lua `ui_attach` handler sees every
//! message — so an entry is copied out of its list before any of it is
//! shown, and `:clist` re-finds the next one by position.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cstr;
use crate::highlight_group::{HLF_D, HLF_N, HLF_QFL};
use crate::memory::XString;
use crate::message::trunc_to;
use crate::message_fmt::msg_bytes;
use crate::path::tail_index;
use crate::semsg;
use crate::tr;
use crate::types::{CmdIdx, IOSIZE};
use core::ffi::c_int;
use std::ffi::CString;

/// The bytes of `text` after any leading spaces and tabs.
pub(crate) fn skip_white(text: &[u8]) -> &[u8] {
    let start = text.iter().position(|&b| b != b' ' && b != b'\t');
    &text[start.unwrap_or(text.len())..]
}

/// Append an error message with its newlines, and the whitespace following
/// them, squeezed into single spaces.
pub(crate) fn qf_fmt_text(out: &mut Vec<u8>, text: &[u8]) {
    let mut bytes = text.iter().copied().take_while(|&b| b != 0).peekable();
    while let Some(b) = bytes.next() {
        if b == b'\n' {
            out.push(b' ');
            while bytes
                .peek()
                .is_some_and(|&next| ascii_iswhite(c_int::from(next)) || next == b'\n')
            {
                bytes.next();
            }
        } else {
            out.push(b);
        }
    }
}

/// Append an entry's position: the line, the end line, and the columns when
/// the entry has them.
pub(crate) fn qf_range_text(out: &mut Vec<u8>, entry: &QfEntry) {
    let mut range = format!("{}", entry.lnum);
    if entry.end_lnum > 0 && entry.lnum != entry.end_lnum {
        range.push_str(&format!("-{}", entry.end_lnum));
    }
    if entry.col > 0 {
        range.push_str(&format!(" col {}", entry.col));
        if entry.end_col > 0 && entry.col != entry.end_col {
            range.push_str(&format!("-{}", entry.end_col));
        }
    }
    out.extend_from_slice(range.as_bytes());
}

/// What `:clist` shows of one entry, copied out of the list.
struct Listed {
    module: Option<XString>,
    /// The file name as shown: the entry's own, or its buffer's, or the
    /// tail of either for a help entry.
    fname: Option<Vec<u8>>,
    pattern: Option<XString>,
    text: XString,
    /// The position and type, as [`qf_range_text`] and [`qf_types`] spell
    /// them.
    position: Vec<u8>,
    has_lnum: bool,
}

impl Listed {
    fn of(entry: &QfEntry) -> Listed {
        let module = entry.module.clone().filter(|m| !m.is_empty());
        let mut fname = None;
        if module.is_none()
            && entry.fnum != 0
            && let Some(buf) = find_buf(entry.fnum)
        {
            let name = match &entry.fname {
                Some(own) => Some(own.to_vec()),
                None => buf.name.shown().map(|name| name.to_bytes().to_vec()),
            };
            // :helpgrep entries name the help file only.
            fname = name.map(|name| {
                if entry.kind == 1 {
                    name[tail_index(&name)..].to_vec()
                } else {
                    name
                }
            });
        }
        let mut position = Vec::new();
        if entry.lnum != 0 {
            qf_range_text(&mut position, entry);
        }
        position.extend_from_slice(qf_types(c_int::from(entry.kind), entry.nr).to_bytes());
        Listed {
            module,
            fname,
            pattern: entry.pattern.clone(),
            text: entry.text.clone(),
            position,
            has_lnum: entry.lnum != 0,
        }
    }
}

/// A NUL-terminated copy of `bytes`, up to any NUL in them.
fn owned_cstr(bytes: &[u8]) -> CString {
    cstr::owned(bytes)
}

/// Print one entry of `:clist`, unless `:filter` rejects it.
fn qf_list_entry(entry: &Listed, qf_idx: c_int, cursel: bool) {
    // "%2d %s": bytes, because a file name need not be UTF-8.
    let heading = match (&entry.module, &entry.fname) {
        (Some(module), _) => [format!("{qf_idx:2} ").as_bytes(), module].concat(),
        (None, Some(fname)) => [format!("{qf_idx:2} ").as_bytes(), fname.as_slice()].concat(),
        (None, None) => format!("{qf_idx:2}").into_bytes(),
    };

    // `:filter /pat/ clist` matches the module name, the file name, the
    // search pattern and the text; the entry is dropped only when every
    // one of them is filtered out.
    let mut filtered = true;
    if let Some(module) = &entry.module {
        filtered = message_filtered(module.as_cstr());
    }
    if filtered && let Some(fname) = &entry.fname {
        filtered = message_filtered(&owned_cstr(fname));
    }
    if filtered && let Some(pattern) = &entry.pattern {
        filtered = message_filtered(pattern.as_cstr());
    }
    if filtered {
        filtered = message_filtered(entry.text.as_cstr());
    }
    if filtered {
        return;
    }

    if msg_col.get() > 0 {
        msg_putchar(c_int::from(b'\n'));
    }
    let cursel = if cursel { HLF_QFL } else { qfFile_hl_id.get() };
    msg_display(&owned_cstr(&heading), cursel, false);

    // The position: "<lnum>[-<end>][ col <col>[-<end>]][ <type> <nr>]".
    if entry.has_lnum {
        msg_str_hl(c":", qfSep_hl_id.get(), false);
    }
    if !entry.position.is_empty() {
        msg_str_hl(&owned_cstr(&entry.position), qfLine_hl_id.get(), false);
    }
    msg_str_hl(c":", qfSep_hl_id.get(), false);

    if let Some(pattern) = &entry.pattern {
        let mut out = Vec::new();
        qf_fmt_text(&mut out, pattern);
        msg_str(&owned_cstr(&out));
        msg_str_hl(c":", qfSep_hl_id.get(), false);
    }
    msg_str(c" ");

    // The message itself. An unrecognized line keeps its indent, since
    // the compiler may be marking a word with "^^^^".
    let text: &[u8] = if entry.fname.is_some() || entry.has_lnum {
        skip_white(&entry.text)
    } else {
        &entry.text
    };
    let mut line = Vec::new();
    qf_fmt_text(&mut line, text);
    msg_prt_line(&owned_cstr(&line), false);
}

/// `:clist`/`:llist`: print the entries of the current list.
pub fn qf_list(excmd: &mut ExArg) {
    let Some(qi) = stack_for_cmd(excmd.cmdidx, true) else {
        return;
    };
    if qi.is_empty() || qi.current_list().is_empty() {
        qf_emsg(e_no_errors);
        return;
    }

    // "+N" lists N entries from the current one; otherwise the argument
    // is a range, counted from the end when negative.
    let arg = excmd.line.arg();
    let plus = arg.first() == Some(&b'+');
    let range = if plus { &arg[1..] } else { arg };
    let Some((mut idx1, mut idx2)) = parse_list_range(range) else {
        let arg = msg_bytes(range);
        semsg!("E488: Trailing characters: {arg}");
        return;
    };
    let qfl = qi.current_slot();
    if plus {
        idx2 = qfl.index + idx1;
        idx1 = qfl.index;
    } else {
        let count = qfl.count();
        if idx1 < 0 {
            idx1 = if -idx1 > count { 0 } else { idx1 + count + 1 };
        }
        if idx2 < 0 {
            idx2 = if -idx2 > count { 0 } else { idx2 + count + 1 };
        }
    }

    // Shorten all the file names, so that it is easy to read.
    shorten_fnames(c_int::from(false));

    // The highlighting comes from the qf.vim syntax file.
    qfFile_hl_id.set(syn_name2id(c"qfFileName"));
    if qfFile_hl_id.get() == 0 {
        qfFile_hl_id.set(HLF_D);
    }
    qfSep_hl_id.set(syn_name2id(c"qfSeparator"));
    if qfSep_hl_id.get() == 0 {
        qfSep_hl_id.set(HLF_D);
    }
    qfLine_hl_id.set(syn_name2id(c"qfLineNr"));
    if qfLine_hl_id.get() == 0 {
        qfLine_hl_id.set(HLF_N);
    }

    // Without "!" only recognised entries are listed — unless none of
    // them is recognised, when they all are.
    let all = excmd.forceit || qfl.no_valid;
    msg_ext_set_kind(c"list_cmd");
    let mut i: c_int = 1;
    while !got_int.get() {
        // Re-found every time round: printing and the break check can
        // both run user code, which may have changed the list.
        let wanted = {
            let Some(entry) = qfl.nth(i) else {
                break;
            };
            ((entry.valid || all) && idx1 <= i && i <= idx2)
                .then(|| (Listed::of(entry), i == qfl.index))
        };
        if let Some((listed, current)) = wanted {
            qf_list_entry(&listed, i, current);
        }
        os_breakcheck();
        i += 1;
    }
}

/// The range `:clist` takes — one number, two numbers separated by a comma,
/// or none, defaulting to `1,-1` — as upstream's `get_list_range()` parses
/// it. `None` when the range is bad or something follows it.
fn parse_list_range(arg: &[u8]) -> Option<(c_int, c_int)> {
    let (mut first, mut last): (c_int, c_int) = (1, -1);
    let mut have_first = false;
    let mut at = skip_white(arg);
    if at.first().is_some_and(|&b| b == b'-' || b.is_ascii_digit()) {
        let (n, len) = decimal(at);
        first = n?;
        have_first = true;
        at = &at[len..];
    }
    at = skip_white(at);
    if let Some(rest) = at.strip_prefix(b",") {
        at = skip_white(rest);
        let (n, len) = decimal(at);
        if len > 0 {
            at = skip_white(&at[len..]);
            last = n?;
        } else if !have_first {
            return None;
        }
    } else if have_first {
        last = first;
    }
    at.is_empty().then_some((first, last))
}

/// The decimal number, possibly negative, at the start of `text` — `None`
/// past `INT_MAX` — and how many bytes it took, as `vim_str2nr()` reads one
/// with no other bases: a lone `-` is a zero one byte long.
fn decimal(text: &[u8]) -> (Option<c_int>, usize) {
    let negative = text.first() == Some(&b'-');
    let digits = &text[usize::from(negative)..];
    let count = digits.iter().take_while(|b| b.is_ascii_digit()).count();
    let len = count + usize::from(negative);
    let mut n: i64 = 0;
    for &d in &digits[..count] {
        n = n.saturating_mul(10).saturating_add(i64::from(d - b'0'));
    }
    if negative {
        n = -n;
    }
    let n = if n > i64::from(c_int::MAX) {
        None
    } else {
        Some(c_int::try_from(n).unwrap_or(c_int::MIN))
    };
    (n, len)
}

/// Print the number, size and title of one list in the stack.
fn qf_msg(qi: Qi, which: c_int, lead: &str) {
    let (listcount, count, title) = {
        let qfl = qi.slot(which);
        (qi.list_count, qfl.count(), qfl.title.clone())
    };
    let mut line = tr!(
        "{}error list {} of {}; {} errors ",
        lead,
        which + 1,
        listcount,
        count
    )
    .into_bytes();
    if let Some(title) = title {
        // The title starts at a fixed column, when there is room.
        if line.len() < 34 {
            line.resize(34, b' ');
        }
        line.extend_from_slice(&title);
    }
    // Upstream builds this in an `IOSIZE` buffer.
    line.truncate(IOSIZE as usize - 1);
    let text = trunc_to(&owned_cstr(&line), Columns.get() - 1, IOSIZE as usize);
    msg(&owned_cstr(&text), 0);
}

/// `:colder`/`:cnewer`/`:lolder`/`:lnewer`: move up or down the stack.
pub fn qf_age(excmd: &mut ExArg) {
    let Some(mut qi) = stack_for_cmd(excmd.cmdidx, true) else {
        return;
    };
    let count = if excmd.addr_count != 0 {
        excmd.line2 as c_int
    } else {
        1
    };
    let older = excmd.cmdidx == CmdIdx::colder || excmd.cmdidx == CmdIdx::lolder;
    for _ in 0..count {
        if older {
            if qi.current == 0 {
                qf_emsg(c"E380: At bottom of quickfix stack");
                break;
            }
            qi.current -= 1;
        } else {
            if qi.current >= qi.list_count - 1 {
                qf_emsg(c"E381: At top of quickfix stack");
                break;
            }
            qi.current += 1;
        }
    }
    qf_msg(qi, qi.current, "");
    qf_update_buffer(qi, None);
}

/// `:chistory`/`:lhistory`: print every list in the stack, or with a count,
/// go to one of them.
pub fn qf_history(excmd: &mut ExArg) {
    let stack = stack_for_cmd(excmd.cmdidx, false);
    if excmd.addr_count > 0 {
        match stack {
            None => qf_emsg(e_loclist),
            Some(mut qi) if excmd.line2 > 0 && excmd.line2 <= LineNr::from(qi.list_count) => {
                qi.current = (excmd.line2 - 1) as c_int;
                qf_msg(qi, qi.current, "");
                qf_update_buffer(qi, None);
            }
            Some(_) => qf_emsg(e_invrange),
        }
        return;
    }
    // No location list at all counts as an empty stack.
    match stack.filter(|qi| !qi.is_empty()) {
        None => {
            msg(gettext(c"No entries"), 0);
        }
        Some(qi) => {
            for i in 0..qi.list_count {
                let lead = if i == qi.current { "> " } else { "  " };
                qf_msg(qi, i, lead);
            }
        }
    }
}

/// The type of an entry as it is printed: `" error"`, `" warning"`, … plus
/// the error number when there is one.
///
/// Upstream answers one of two static buffers the next call overwrites; this
/// answers a string the caller owns.
pub(crate) fn qf_types(c: c_int, nr: c_int) -> CString {
    const W: c_int = b'W' as c_int;
    const LOWER_W: c_int = b'w' as c_int;
    const I: c_int = b'I' as c_int;
    const LOWER_I: c_int = b'i' as c_int;
    const N: c_int = b'N' as c_int;
    const LOWER_N: c_int = b'n' as c_int;
    const E: c_int = b'E' as c_int;
    const LOWER_E: c_int = b'e' as c_int;
    let name: &[u8] = match c {
        W | LOWER_W => b" warning",
        I | LOWER_I => b" info",
        N | LOWER_N => b" note",
        E | LOWER_E => b" error",
        0 if nr > 0 => b" error",
        0 | 1 => b"",
        other => return numbered(&[b' ', other.to_le_bytes()[0]], nr),
    };
    numbered(name, nr)
}

/// `qf_types`' tail: `name`, with ` %3d` of `nr` after it when there is one.
fn numbered(name: &[u8], nr: c_int) -> CString {
    if nr <= 0 {
        return cstr::owned(name);
    }
    let mut text = name.to_vec();
    text.extend_from_slice(format!(" {nr:3}").as_bytes());
    cstr::owned(&text)
}

/// Open the entry under the cursor in the quickfix window, in a new window
/// when `split`.
pub fn qf_view_result(split: bool) {
    let window = Win::current();
    let in_ll_window = window.is_location_list_window();
    let qi = if in_ll_window {
        window
            .w_llist_ref
            .expect("a location list window shows a stack")
            .stack()
    } else {
        Qi::global()
    };
    if qi.current_list().is_empty() {
        qf_emsg(e_no_errors);
        return;
    }
    if split {
        let lnum = Win::current().w_cursor.lnum as c_int;
        qf_jump_newwin(qi, 0, lnum, false, true);
        let _ = do_cmdline_cmd(c"clearjumps");
        return;
    }
    let _ = do_cmdline_cmd(if in_ll_window { c".ll" } else { c".cc" });
}
