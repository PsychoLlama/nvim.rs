//! Command-line history: `<Up>`, `<Down>` and the history commands.
//!
//! [`command_line_browse_history`] is the recall itself and
//! [`command_line_next_histidx`] the index walk it drives, matching the
//! typed prefix where `'wildoptions'` asks for it.  [`get_list_range`] parses
//! the `:history` style range argument.

#![forbid(unsafe_code)]

use super::*;
use crate::charset::{Str2NrBases, skip, str2nr_scan};
use crate::cstr;
use crate::keycodes::{Ctrl_N, Key};
use crate::types::{ExpandContext, Failed, NUL};

/// Step `s->hiscnt` one entry back (or forward, with `next_match`) through
/// the history, skipping entries that do not start with what was typed.
pub(crate) fn command_line_next_histidx(s: &mut CommandLineState, next_match: bool) {
    loop {
        if !next_match {
            // One step backwards.
            if s.hiscnt == get_hislen() {
                // first time
                s.hiscnt = get_hisidx(s.histype);
            } else if s.hiscnt == 0 && get_hisidx(s.histype) != get_hislen() - 1 {
                s.hiscnt = get_hislen() - 1;
            } else if s.hiscnt != get_hisidx(s.histype) + 1 {
                s.hiscnt -= 1;
            } else {
                // at the top of the list
                s.hiscnt = s.save_hiscnt;
                break;
            }
        } else if s.hiscnt == get_hisidx(s.histype) {
            // On the last entry: clear the line.
            s.hiscnt = get_hislen();
            break;
        } else if s.hiscnt == get_hislen() {
            // Not on a history line, nothing to do.
            break;
        } else if s.hiscnt == get_hislen() - 1 {
            s.hiscnt = 0; // wrap around
        } else {
            s.hiscnt += 1;
        }

        let Some(entry) = hist_entry_text(s.histype, s.hiscnt) else {
            s.hiscnt = s.save_hiscnt;
            break;
        };
        if (s.c != Key::Up.code() && s.c != Key::Down.code())
            || s.hiscnt == s.save_hiscnt
            || entry
                .text
                .to_bytes()
                .starts_with(s.lookfor.as_deref().unwrap_or_default())
        {
            break;
        }
    }
}

/// Handle Up, Down, PageUp, PageDown, CTRL-N and CTRL-P on the command line.
pub(crate) fn command_line_browse_history(s: &mut CommandLineState) -> KeyOutcome {
    let mut cc = Cc::current();
    if s.histype == HIST_INVALID || get_hislen() == 0 || s.firstc == NUL {
        return KeyOutcome::NotChanged; // no history
    }

    s.save_hiscnt = s.hiscnt;

    // Save the current command string, so that it can be restored later.
    if s.lookfor.is_none() {
        // C's `xstrnsave(cmdbuff, cmdlen)` cut at the cursor.
        let line = cc.bytes();
        let upto = usize::try_from(cc.cmdpos).unwrap_or(0).min(line.len());
        s.lookfor = Some(cstr::until_nul(&line[..upto]).to_vec());
    }

    let next_match = s.c == Key::Down.code()
        || s.c == Key::SDown.code()
        || s.c == Ctrl_N
        || s.c == Key::Pagedown.code()
        || s.c == Key::Kpagedown.code();
    command_line_next_histidx(s, next_match);

    if s.hiscnt == s.save_hiscnt {
        beep_flush();
        return KeyOutcome::NotChanged;
    }

    // Jumped to another entry: the text to put on the line, and the
    // separator it was added under when it is a history entry.
    dealloc_cmdbuff();
    completion_mut(|xpc| xpc.context = ExpandContext::Nothing);
    let (text, hist_sep): (Vec<u8>, Option<::core::ffi::c_int>) = if s.hiscnt == get_hislen() {
        // Back to the old one.
        (s.lookfor.clone().unwrap_or_default(), None)
    } else {
        let entry =
            hist_entry_text(s.histype, s.hiscnt).expect("the walk stopped on an occupied slot");
        (
            entry.text.to_bytes().to_vec(),
            Some(::core::ffi::c_int::from(entry.sep)),
        )
    };

    let old_firstc = hist_sep.unwrap_or(NUL);
    let line = if s.histype == HIST_SEARCH && hist_sep.is_some() && old_firstc != s.firstc {
        // Correct for the separator character used when the history entry
        // was added versus the one used now.
        let firstc = s.firstc as u8;
        let unescaped = |j: usize| j == 0 || text[j - 1] != b'\\';
        let mut line = Vec::with_capacity(text.len() + 1);
        for (j, &b) in text.iter().enumerate() {
            if ::core::ffi::c_int::from(b) == old_firstc && unescaped(j) {
                // Replace the old separator with the new one, unless it is
                // escaped.
                line.push(firstc);
            } else {
                // Escape the new separator, unless it is already escaped.
                if b == firstc && unescaped(j) {
                    line.push(b'\\');
                }
                line.push(b);
            }
        }
        line
    } else {
        text
    };
    let len = ::core::ffi::c_int::try_from(line.len()).expect("a history entry fits a c_int");
    cc.open(len);
    cc.cmdbuff.write_text(&line);
    cc.cmdpos = len;

    redrawcmd();
    KeyOutcome::Changed
}

/// Parse a `[N][,[M]]` range argument, as `:history` takes, from
/// `text[*at..]` (`text` read as a C string).
///
/// `at` is advanced past what was parsed; `num1` and `num2` are only written
/// when the corresponding number was present.  Answers `Err` on a malformed
/// range or one whose numbers do not fit an `int`.
pub(crate) fn get_list_range(
    text: &[u8],
    at: &mut usize,
    num1: &mut ::core::ffi::c_int,
    num2: &mut ::core::ffi::c_int,
) -> Result<(), Failed> {
    let skipwhite = |i: usize| i + skip::white(text.get(i..).unwrap_or_default());
    // `vim_str2nr` with no other bases: how long the number at `i` is, and
    // its value.
    let number = |i: usize| {
        let parsed = str2nr_scan(
            text.get(i..).unwrap_or_default(),
            0,
            Str2NrBases::NONE,
            false,
            true,
        );
        (parsed.len, parsed.value)
    };
    let mut first = false;

    *at = skipwhite(*at);
    let c = cstr::byte_at(text, *at);
    if c == b'-' || c.is_ascii_digit() {
        let (len, num) = number(*at);
        *at += len;
        if num > INT_MAX as VarNumber {
            return Err(Failed);
        }
        *num1 = num as ::core::ffi::c_int;
        first = true;
    }

    *at = skipwhite(*at);
    if cstr::byte_at(text, *at) == b',' {
        // parse "to" part of range
        *at = skipwhite(*at + 1);
        let (len, num) = number(*at);
        if len > 0 {
            *at = skipwhite(*at + len);
            if num > INT_MAX as VarNumber {
                return Err(Failed);
            }
            *num2 = num as ::core::ffi::c_int;
        } else if !first {
            return Err(Failed);
        }
    } else if first {
        // only one number given
        *num2 = *num1;
    }
    Ok(())
}
