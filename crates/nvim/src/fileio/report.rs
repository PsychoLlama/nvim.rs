//! What the read ended up doing, in one message.
//!
//! Every note the message can carry — `[readonly]`, `[noeol]`,
//! `[converted]`, the line the first bad byte was on — is one field of
//! [`Outcome`], and `report_read` turns them into the line Nvim prints
//! after a file is read.

#![forbid(unsafe_code)]

use crate::memory::XString;
use crate::os::env::home_replace_in;
use crate::strings::{FmtArg, format_into};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};

use crate::bufwrite::translate;

use super::*;
use crate::types::{IOSIZE, ShmFlag};

/// A message assembled the way upstream assembles one in `IObuff`: never
/// more than `IOSIZE - 1` bytes, each piece cut to what still fits.
pub(crate) struct Report(XString);

impl Report {
    /// The file name in quotes, with the home directory at its start
    /// replaced by `~`, and a space after it -- in at most `room` bytes.
    /// Reading stdin has no name: it is shown as `-stdin-`.
    pub(crate) fn quoted_fname(buffer: Buf, fname: Option<&CStr>, room: usize) -> Report {
        let name = fname.unwrap_or(c"-stdin-");
        let mut text = XString::from_bytes(b"\"");
        text.push_bytes(&home_replace_in(Some(buffer), name, room - 4, true));
        let mut report = Report(text);
        report.push_within(b"\" ", room);
        report
    }

    /// Append `bytes`, cut so the whole stays under `size` bytes with its
    /// terminator: `xstrlcat`.
    pub(crate) fn push_within(&mut self, bytes: &[u8], size: usize) {
        let room = size.saturating_sub(1).saturating_sub(self.0.len());
        self.0.push_bytes(&bytes[..bytes.len().min(room)]);
    }

    /// Append `bytes`, cut to what the report still has room for.
    pub(crate) fn push(&mut self, bytes: &[u8]) {
        self.push_within(bytes, IOSIZE as usize);
    }

    /// Append `fmt` formatted with `args`, cut to what the report still has
    /// room for: `snprintf` at the report's end.
    pub(crate) fn push_fmt(&mut self, fmt: &CStr, args: &[FmtArg<'_>]) {
        let room = (IOSIZE as usize).saturating_sub(self.0.len());
        if room == 0 {
            return;
        }
        let mut out = vec![0; room];
        let len = format_into(&mut out, fmt.to_bytes(), args).min(room - 1);
        self.0.push_bytes(&out[..len]);
    }

    pub(crate) fn as_cstr(&self) -> &CStr {
        self.0.as_cstr()
    }
}

/// Append the file format to `report`, unless it is the platform default.
///
/// @return  true if something was appended.
pub(crate) fn msg_add_fileformat(report: &mut Report, eol_type: c_int) -> bool {
    let note = match eol_type {
        EOL_DOS => c"[dos]",
        EOL_MAC => c"[mac]",
        // On a platform where CRLF is the default, EOL_UNIX would be
        // worth noting instead; upstream compiles that in with USE_CRNL.
        _ => return false,
    };
    report.push(gettext(note).to_bytes());
    true
}

/// Append the line and character count to `report`.
pub(crate) fn msg_add_lines(
    report: &mut Report,
    insert_space: bool,
    lnum: LineNr,
    nchars: FileOffset,
) {
    let space: &[u8] = if insert_space { b" " } else { b"" };
    let (lines, bytes) = (int64_t::from(lnum), nchars);

    if shortmess(ShmFlag::LINES) {
        // l10n: L as in line, B as in byte.
        let fmt = gettext(c"%s%ldL, %ldB");
        report.push_fmt(fmt, &[space.into(), lines.into(), bytes.into()]);
        return;
    }

    let fmt = ngettext(
        c"%s%ld line, ",
        c"%s%ld lines, ",
        lnum as core::ffi::c_ulong,
    );
    report.push_fmt(fmt, &[space.into(), lines.into()]);
    let fmt = ngettext(c"%ld byte", c"%ld bytes", nchars as core::ffi::c_ulong);
    report.push_fmt(fmt, &[bytes.into()]);
}

/// Everything the closing message needs to know about how the read went.
pub(crate) struct Outcome {
    pub perm: c_int,
    pub read_no_eol_lnum: LineNr,
    pub ff_error: c_int,
    pub split: c_int,
    pub notconverted: bool,
    pub converted: bool,
    pub conv_error: LineNr,
    pub illegal_byte: LineNr,
    pub error: bool,
    pub fileformat: c_int,
    pub linecnt: LineNr,
    pub filesize: FileOffset,
}

/// Report what was read.
pub(crate) fn report_read(sfname: Option<&CStr>, how: How, out: &Outcome) {
    // The report. Upstream assembles it in `IObuff`, which `msg_trunc` and
    // `set_keep_msg` reach the message machinery through.
    let mut report = Report::quoted_fname(Buf::current(), sfname, IOSIZE as usize);
    let mut noted = false;
    let mut note = |report: &mut Report, text: &CStr| {
        report.push(text.to_bytes());
        noted = true;
    };

    if out.perm & S_IFMT == 0o10000 {
        note(&mut report, translate(c"[fifo]"));
    }
    if out.perm & S_IFMT == 0o140000 {
        note(&mut report, translate(c"[socket]"));
    }
    if Buf::current().b_p_ro != 0 {
        let ro = if shortmess(ShmFlag::RO) {
            translate(c"[RO]")
        } else {
            translate(c"[readonly]")
        };
        note(&mut report, ro);
    }
    if out.read_no_eol_lnum != 0 {
        note(&mut report, translate(c"[noeol]"));
    }
    if out.ff_error == EOL_DOS {
        note(&mut report, translate(c"[CR missing]"));
    }
    if out.split != 0 {
        note(&mut report, translate(c"[long lines split]"));
    }
    if out.notconverted {
        note(&mut report, translate(c"[NOT converted]"));
    } else if out.converted {
        note(&mut report, translate(c"[converted]"));
    }
    if out.conv_error != 0 {
        let fmt = translate(c"[CONVERSION ERROR in line %ld]");
        report.push_fmt(fmt, &[int64_t::from(out.conv_error).into()]);
        noted = true;
    } else if out.illegal_byte > 0 {
        let fmt = translate(c"[ILLEGAL BYTE in line %ld]");
        report.push_fmt(fmt, &[int64_t::from(out.illegal_byte).into()]);
        noted = true;
    } else if out.error {
        note(&mut report, translate(c"[READ ERRORS]"));
    }
    if msg_add_fileformat(&mut report, out.fileformat) {
        noted = true;
    }
    msg_add_lines(&mut report, noted, out.linecnt, out.filesize);

    keep_msg.set(None);
    let mut shown = None;
    msg_scrolled_ign.set(true);

    if !how.stdin && !how.buffer {
        if msg_col.get() > 0 {
            msg_putchar(c_int::from(b'\r')); // overwrite previous message
        }
        shown = msg_fit_head(report.as_cstr(), false, 0);
    }

    // The message has to be repeated after redrawing when reading from
    // stdin (the screen is cleared next), when `restart_edit` is set
    // (otherwise there is a delay before redrawing), and when the screen
    // scrolled but there is no wait-return prompt.
    if how.stdin
        || how.buffer
        || restart_edit.get() != 0
        || (msg_scrolled.get() != 0 && !need_wait_return.get())
    {
        set_keep_msg(shown.as_ref().map(XString::as_cstr), 0);
    }
    msg_scrolled_ign.set(false);
}
