//! Terminal descriptions.
//!
//! Everything the TUI knows about the terminal it is talking to arrives
//! through here: the description itself (from the system's terminfo database
//! if there is one, from [`builtin`] otherwise), the `$TERM` family tests the
//! TUI's workaround code asks, the human-readable dump `:verbose` prints, and
//! [`terminfo_fmt`], which turns a parameterised capability into bytes.
//!
//! The capability slots every one of those indexes by are defined once, in
//! [`caps`].

pub mod builtin;
pub mod caps;
pub mod param;

use crate::charset::transstr;
use crate::memory::{arena_strdup, xfree, xmemdupz};
use crate::tui::terminfo::caps::{
    BACK_COLOR_ERASE, COLUMNS, EXT_CAPS, FUNCTION_KEYS, KEYS, LINES, MAX_COLORS, STRING_CAPS,
    kTerm_reset_cursor_style, kTermCount,
};
use crate::tui::unibi;
use crate::types::{Arena, String_0, TPVAR, TerminfoEntry, size_t};
use core::ffi::{CStr, c_char, c_long, c_void};

/// Does `term` name a terminal of the `family` family?
///
/// Per terminfo's own commentary a minus is the only valid suffix separator,
/// but `screen` describes terminals like `screen.xterm`, so a dot counts too
/// -- which is what makes `screen.xterm` a screen rather than an xterm.
pub fn is_term_family(term: &[u8], family: &[u8]) -> bool {
    term.strip_prefix(family)
        .is_some_and(|rest| matches!(rest.first(), None | Some(b'-') | Some(b'.')))
}

/// `is_term_family` for the transpiled callers, which hold `$TERM` as a
/// possibly-null C string.
pub unsafe fn terminfo_is_term_family(term: *const c_char, family: &CStr) -> bool {
    !term.is_null() && is_term_family(CStr::from_ptr(term).to_bytes(), family.to_bytes())
}

/// Is this one of the BSD system consoles, which claim to be a terminal they
/// are not?
///
/// Always false here: the check only ever had a body on the BSDs, where it
/// recognises `vt220`/`vt100` and a FreeBSD console pretending to be an xterm
/// (#8644). The callers' workarounds are kept so a BSD build can switch it
/// back on.
pub fn terminfo_is_bsd_console(_term: *const c_char) -> bool {
    false
}

/// The built-in description for `$TERM` (`None` when nvim has no `$TERM`),
/// and the name nvim will report for the terminal.
///
/// This is the fallback for termcap systems, an unrecognised `$TERM`, and
/// anything else that leaves the database empty-handed. It does not try to
/// detect xterm pretenders.
pub fn terminfo_from_builtin(term: Option<&CStr>) -> (&'static CStr, TerminfoEntry) {
    let (name, description) = builtin::from_term(term);
    (name, description.entry())
}

/// The system terminfo database's description of `termname`, or `None` when
/// it has none.
///
/// The sequences are copied into `arena`, which the TUI keeps for as long as
/// it keeps the entry.
pub unsafe fn terminfo_from_database(termname: &CStr, arena: *mut Arena) -> Option<TerminfoEntry> {
    let term = unibi::from_term(termname)?;
    let dup = |val: Option<&CStr>| match val {
        Some(s) => arena_strdup(arena, s.as_ptr()) as *const c_char,
        None => core::ptr::null(),
    };

    let mut entry = TerminfoEntry {
        bce: term.get_bool(BACK_COLOR_ERASE),
        has_Tc_or_RGB: false,
        Su: false,
        max_colors: term.get_num(MAX_COLORS),
        lines: term.get_num(LINES),
        columns: term.get_num(COLUMNS),
        defs: [core::ptr::null(); kTermCount as usize],
        keys: [[core::ptr::null(); 2]; KEYS.len()],
        f_keys: [core::ptr::null(); FUNCTION_KEYS.len()],
    };
    for name in term.ext_bool_names() {
        match name.to_bytes() {
            b"Tc" | b"RGB" => entry.has_Tc_or_RGB = true,
            b"Su" => entry.Su = true,
            _ => {}
        }
    }

    for (slot, cap) in STRING_CAPS.iter().enumerate() {
        entry.defs[slot] = dup(term.get_str(cap.cap));
    }
    // The extensions fill the slots from `kTerm_reset_cursor_style` on, and
    // are looked up by the name the description gives them.
    for (i, cap) in EXT_CAPS.iter().enumerate() {
        if let Some((_, val)) = term
            .ext_strs()
            .find(|(name, _)| name.to_bytes() == cap.terminfo_name)
        {
            entry.defs[kTerm_reset_cursor_style as usize + i] = dup(val);
        }
    }
    for (slot, key) in KEYS.iter().enumerate() {
        if let Some(val) = term.get_str(key.cap) {
            entry.keys[slot][0] = dup(Some(val));
            // The shifted variant is only consulted when the unshifted one is
            // there, so a description with only `key_sfoo` gets neither.
            if let Some(shifted) = key.shifted_cap {
                entry.keys[slot][1] = dup(term.get_str(shifted));
            }
        }
    }
    for (slot, &cap) in FUNCTION_KEYS.iter().enumerate() {
        entry.f_keys[slot] = dup(term.get_str(cap));
    }
    Some(entry)
}

/// The terminal description as `:verbose` dumps it, for the "what does nvim
/// think this terminal can do" question. Serves the purpose Vim's `:set
/// termcap` did.
///
/// Two oddities in the output are upstream's and deliberately kept: the
/// `max_colors` line reports the column count, and the key listings start at
/// their second entry, so `key_backspace` and `key_f1` never appear.
///
/// The returned string is allocated for the caller to free.
pub unsafe fn terminfo_info_msg(
    entry: &TerminfoEntry,
    termname: *const c_char,
    from_db: bool,
) -> String_0 {
    let mut msg = Vec::new();
    msg.extend_from_slice(b"&term: ");
    msg.extend_from_slice(CStr::from_ptr(termname).to_bytes());
    msg.extend_from_slice(if from_db {
        b"\nusing terminfo database\n\n".as_slice()
    } else {
        b"\nusing builtin terminfo\n\n".as_slice()
    });

    let yes_no = |val: bool| if val { "true" } else { "false" };
    msg.extend_from_slice(
        format!(
            "Boolean capabilities:\n  \
             back_color_erase: {}\n  \
             truecolor ('Tc' or 'RGB'): {}\n  \
             extended underline ('Su'): {}\n\n\
             Numeric capabilities: (-1 for unknown)\n  \
             lines: {}\n  columns: {}\n  max_colors: {}\n\n\
             String capabilities:\n",
            yes_no(entry.bce),
            yes_no(entry.has_Tc_or_RGB),
            yes_no(entry.Su),
            entry.lines,
            entry.columns,
            entry.columns,
        )
        .as_bytes(),
    );

    // Most of these are escape sequences, so they are shown the way an
    // unprintable option value is shown.
    let escaped = |msg: &mut Vec<u8>, seq: *const c_char| {
        let printable = transstr(seq, false);
        msg.extend_from_slice(CStr::from_ptr(printable).to_bytes());
        xfree(printable as *mut c_void);
    };

    let def_names = STRING_CAPS
        .iter()
        .map(|cap| cap.name.to_string())
        .chain(EXT_CAPS.iter().map(|cap| {
            format!(
                "{} ({})",
                cap.name,
                String::from_utf8_lossy(cap.terminfo_name)
            )
        }));
    for (slot, name) in def_names.enumerate() {
        let seq = entry.defs[slot];
        if !seq.is_null() {
            msg.extend_from_slice(format!("  {name:<31} = ").as_bytes());
            escaped(&mut msg, seq);
            msg.push(b'\n');
        }
    }
    for (slot, key) in KEYS.iter().enumerate().skip(1) {
        let seq = entry.keys[slot][0];
        if !seq.is_null() {
            msg.extend_from_slice(format!("  key_{:<27} = ", key.stem).as_bytes());
            escaped(&mut msg, seq);
            let shifted = entry.keys[slot][1];
            if !shifted.is_null() {
                msg.extend_from_slice(format!(", key_s{} = ", key.stem).as_bytes());
                escaped(&mut msg, shifted);
            }
            msg.push(b'\n');
        }
    }
    for (slot, &seq) in entry.f_keys.iter().enumerate().skip(1) {
        if !seq.is_null() {
            let name = format!("f{}", slot + 1);
            msg.extend_from_slice(format!("  key_{name:<27} = ").as_bytes());
            escaped(&mut msg, seq);
            msg.push(b'\n');
        }
    }

    String_0::from_raw_parts(
        xmemdupz(msg.as_ptr() as *const c_void, msg.len()) as *mut c_char,
        msg.len(),
    )
}

/// Expand a parameterised capability into `[buf_start, buf_end)`.
///
/// Returns the number of bytes written, or 0 for any failure -- an output
/// that would not fit, or a capability that overflows the operand stack.
/// `params` is written back through: `%i` increments the first two, which is
/// why a caller that means to retry hands over a copy.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn terminfo_fmt(
    buf_start: *mut c_char,
    buf_end: *mut c_char,
    capability: *const c_char,
    params: *mut TPVAR,
) -> size_t {
    let buf = core::slice::from_raw_parts_mut(
        buf_start as *mut u8,
        buf_end.offset_from(buf_start) as usize,
    );
    let mut values = [param::Param::default(); 9];
    for (i, value) in values.iter_mut().enumerate() {
        let given = *params.add(i);
        value.num = given.num;
        value.string = (!given.string.is_null()).then(|| CStr::from_ptr(given.string).to_bytes());
    }

    let mut out = param::Out::new(buf);
    let expanded = param::expand(CStr::from_ptr(capability).to_bytes(), &mut values, &mut out);

    for (i, value) in values.iter().enumerate() {
        (*params.add(i)).num = value.num as c_long;
    }
    if expanded { out.len() } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn family_matches_only_on_a_separator() {
        assert!(is_term_family(b"xterm", b"xterm"));
        assert!(is_term_family(b"xterm-256color", b"xterm"));
        assert!(is_term_family(b"screen.xterm-256color", b"screen"));
        assert!(!is_term_family(b"xterms", b"xterm"));
        assert!(!is_term_family(b"xter", b"xterm"));
        assert!(!is_term_family(b"", b"xterm"));
        assert!(is_term_family(b"", b""));
    }
}
