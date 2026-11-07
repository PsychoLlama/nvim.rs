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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

pub mod builtin;
pub mod caps;
pub mod entry;
pub mod param;

use crate::charset::transstr_owned;
use crate::tui::terminfo::caps::{
    BACK_COLOR_ERASE, COLUMNS, EXT_CAPS, FUNCTION_KEYS, KEYS, LINES, MAX_COLORS, STRING_CAPS,
    TerminfoDef, kTerm_reset_cursor_style,
};
use crate::tui::terminfo::entry::TerminfoEntry;
use crate::tui::unibi;
use crate::types::ApiString;
use core::ffi::CStr;
use std::borrow::Cow;

/// Does `term` name a terminal of the `family` family?
///
/// Per terminfo's own commentary a minus is the only valid suffix separator,
/// but `screen` describes terminals like `screen.xterm`, so a dot counts too
/// -- which is what makes `screen.xterm` a screen rather than an xterm.
pub fn is_term_family(term: &[u8], family: &[u8]) -> bool {
    term.strip_prefix(family)
        .is_some_and(|rest| matches!(rest.first(), None | Some(b'-') | Some(b'.')))
}

/// Is this one of the BSD system consoles, which claim to be a terminal they
/// are not?
///
/// Always false here: the check only ever had a body on the BSDs, where it
/// recognises `vt220`/`vt100` and a FreeBSD console pretending to be an xterm
/// (#8644). The callers' workarounds are kept so a BSD build can switch it
/// back on.
pub fn terminfo_is_bsd_console(_term: Option<&CStr>) -> bool {
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
/// The sequences are copied out of the parsed description, so the entry owns
/// them and outlives the database read.
pub fn terminfo_from_database(termname: &CStr) -> Option<TerminfoEntry> {
    let term = unibi::from_term(termname)?;
    let owned = |val: Option<&CStr>| val.map(|s| Cow::Owned(s.to_bytes().to_vec()));

    let mut entry = TerminfoEntry {
        bce: term.get_bool(BACK_COLOR_ERASE),
        max_colors: term.get_num(MAX_COLORS),
        lines: term.get_num(LINES),
        columns: term.get_num(COLUMNS),
        ..TerminfoEntry::default()
    };
    for name in term.ext_bool_names() {
        match name.to_bytes() {
            b"Tc" | b"RGB" => entry.has_tc_or_rgb = true,
            b"Su" => entry.su = true,
            _ => {}
        }
    }

    for (slot, cap) in STRING_CAPS.iter().enumerate() {
        entry.defs[slot] = owned(term.get_str(cap.cap));
    }
    // The extensions fill the slots from `kTerm_reset_cursor_style` on, and
    // are looked up by the name the description gives them.
    for (i, cap) in EXT_CAPS.iter().enumerate() {
        if let Some((_, val)) = term
            .ext_strs()
            .find(|(name, _)| name.to_bytes() == cap.terminfo_name)
        {
            entry.defs[kTerm_reset_cursor_style as usize + i] = owned(val);
        }
    }
    for (slot, key) in KEYS.iter().enumerate() {
        if let Some(val) = term.get_str(key.cap) {
            entry.keys[slot][0] = owned(Some(val));
            // The shifted variant is only consulted when the unshifted one is
            // there, so a description with only `key_sfoo` gets neither.
            if let Some(shifted) = key.shifted_cap {
                entry.keys[slot][1] = owned(term.get_str(shifted));
            }
        }
    }
    for (slot, &cap) in FUNCTION_KEYS.iter().enumerate() {
        entry.f_keys[slot] = owned(term.get_str(cap));
    }
    Some(entry)
}

impl TerminfoEntry {
    /// The sequence in string capability `slot`, if the terminal has one.
    pub fn def(&self, slot: TerminfoDef) -> Option<&[u8]> {
        self.defs[slot as usize].as_deref()
    }

    /// Does the terminal have string capability `slot` at all?
    pub fn has(&self, slot: TerminfoDef) -> bool {
        self.defs[slot as usize].is_some()
    }

    /// Put `seq`, a sequence in static data, in slot `slot`.
    pub fn set_def(&mut self, slot: TerminfoDef, seq: &'static CStr) {
        self.defs[slot as usize] = Some(Cow::Borrowed(seq.to_bytes()));
    }
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
pub fn terminfo_info_msg(entry: &TerminfoEntry, termname: &CStr, from_db: bool) -> ApiString {
    let mut msg = Vec::new();
    msg.extend_from_slice(b"&term: ");
    msg.extend_from_slice(termname.to_bytes());
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
            yes_no(entry.has_tc_or_rgb),
            yes_no(entry.su),
            entry.lines,
            entry.columns,
            entry.columns,
        )
        .as_bytes(),
    );

    // Most of these are escape sequences, so they are shown the way an
    // unprintable option value is shown.
    let escaped = |msg: &mut Vec<u8>, seq: &[u8]| {
        msg.extend_from_slice(transstr_owned(seq, false).as_cstr().to_bytes());
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
        if let Some(seq) = entry.defs[slot].as_deref() {
            msg.extend_from_slice(format!("  {name:<31} = ").as_bytes());
            escaped(&mut msg, seq);
            msg.push(b'\n');
        }
    }
    for (slot, key) in KEYS.iter().enumerate().skip(1) {
        if let Some(seq) = entry.keys[slot][0].as_deref() {
            msg.extend_from_slice(format!("  key_{:<27} = ", key.stem).as_bytes());
            escaped(&mut msg, seq);
            if let Some(shifted) = entry.keys[slot][1].as_deref() {
                msg.extend_from_slice(format!(", key_s{} = ", key.stem).as_bytes());
                escaped(&mut msg, shifted);
            }
            msg.push(b'\n');
        }
    }
    for (slot, seq) in entry.f_keys.iter().enumerate().skip(1) {
        if let Some(seq) = seq.as_deref() {
            let name = format!("f{}", slot + 1);
            msg.extend_from_slice(format!("  key_{name:<27} = ").as_bytes());
            escaped(&mut msg, seq);
            msg.push(b'\n');
        }
    }

    ApiString::from_bytes(&msg)
}

/// Expand a parameterised capability into `buf`.
///
/// Returns the number of bytes written, or 0 for any failure -- an output
/// that would not fit, or a capability that overflows the operand stack.
/// `params` is written back through: `%i` increments the first two, which is
/// why a caller that means to retry hands over a copy.
pub(crate) fn terminfo_fmt(
    buf: &mut [u8],
    capability: &[u8],
    params: &mut [param::Param; 9],
) -> usize {
    let mut out = param::Out::new(buf);
    if param::expand(capability, params, &mut out) {
        out.len()
    } else {
        0
    }
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
