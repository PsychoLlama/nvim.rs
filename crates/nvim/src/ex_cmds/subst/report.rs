//! What the user sees after a `:s` -- the summary line and the live preview.
//!
//! [`do_sub_msg`] is the "N substitutions on N lines" report, with the
//! 'report' option and `:s#` numbering deciding whether it prints at all;
//! [`show_sub`] is `'inccommand'`, which runs the substitution into a preview
//! buffer and highlights the matches as the command line is typed.
//! [`ex_substitute`] and [`ex_substitute_preview`] are the two Ex entry
//! points.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use crate::ascii::ascii_isdigit;
use crate::buffer::{buf_ensure_loaded, find_buf};
use crate::decoration::bufhl_add_hl_pos_offset;
use crate::ex_cmds::{PreviewLines, SID_NONE, SubResult, do_sub};
use crate::ex_cmds::{SUB_NLINES, SUB_NSUBS};
use crate::getchar::state::{KeyTyped, got_int};
use crate::memline::{Lines, ml_append_bytes, ml_replace_buf_text};
use crate::memory::XString;
use crate::message::e_interr;
use crate::message::{MSG_BUF_LEN, emsg, messaging, msg, set_keep_msg};
use crate::r#move::update_topline;
use crate::option::set_option_direct;
use crate::option::vars::{P_ICM, p_rdt, p_report, p_shm};
use crate::options::kOptShortmess;
use crate::os::cshim::{gettext, ngettext};
use crate::profile::{profile_setlimit, profile_zero};
use crate::types::OptStr;
use crate::types::{
    ColNr, ExArg, Handle, LPos, LineNr, OptInt, OptVal, OptionSetFlags, Pos, int64_t,
};
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int, c_ulong};
use std::ffi::CString;

/// An option value that borrows a static C string, for the two options this
/// module sets and puts back.
pub(crate) const fn static_cstr_optval(s: &'static CStr) -> OptVal {
    OptVal::static_string(s)
}

/// The singular and plural forms of the report, for one line and for several.
///
/// Both are themselves `ngettext` choices over the number of substitutions,
/// so the message is right in languages with more than two plural forms.
fn report_forms(count_only: bool) -> [[&'static CStr; 2]; 2] {
    if count_only {
        [
            [c"%ld match on %ld line", c"%ld matches on %ld line"],
            [c"%ld match on %ld lines", c"%ld matches on %ld lines"],
        ]
    } else {
        [
            [
                c"%ld substitution on %ld line",
                c"%ld substitutions on %ld line",
            ],
            [
                c"%ld substitution on %ld lines",
                c"%ld substitutions on %ld lines",
            ],
        ]
    }
}

/// Give the message for the number of substitutions.  Also used after a
/// `:global`.
///
/// `count_only` is the `n` flag, which counts matches instead of replacing
/// them and always reports.
///
/// Returns true if a message was given.
pub fn do_sub_msg(count_only: bool) -> bool {
    // Only report substitutions when there were more than 'report' of them,
    // the command was typed by the user or more than one line changed, and
    // messages are not disabled.
    let worth_reporting = SUB_NSUBS.get() as OptInt > p_report()
        && (KeyTyped.get() || SUB_NLINES.get() > 1 as LineNr || p_report() < 1 as OptInt);
    if (worth_reporting || count_only) && messaging() {
        let forms = report_forms(count_only);
        let nsubs = SUB_NSUBS.get();
        let nlines = SUB_NLINES.get();
        // Upstream builds the report in a `MSG_BUF_LEN` buffer: the count
        // is cut to what the interruption note leaves of it.
        let room = MSG_BUF_LEN as usize;
        let mut report = if got_int.get() {
            gettext(c"(Interrupted) ").to_bytes().to_vec()
        } else {
            Vec::new()
        };
        report.truncate(room - 1);
        let single = ngettext(forms[0][0], forms[0][1], nsubs as c_ulong);
        let plural = ngettext(forms[1][0], forms[1][1], nsubs as c_ulong);
        let fmt = ngettext(single, plural, nlines as c_ulong);
        let left = room - report.len();
        let counts = crate::formatted!(left, fmt.to_bytes(), nsubs as int64_t, nlines as int64_t);
        report.extend_from_slice(&counts);
        let report = XString::from_bytes(&report);
        if msg(report.as_cstr(), 0) {
            // Save the message to display it after a redraw.
            set_keep_msg(Some(report.as_cstr()), 0);
        }
        return true;
    }
    if got_int.get() {
        emsg(gettext(e_interr));
        return true;
    }
    false
}

/// The `[Preview]` buffer being filled, and how far the fill has reached.
struct PreviewBuf {
    buf: Buf,
    /// Width of the "|lnum| " column that carries the line numbers.
    col_width: c_int,
    /// Last line added to the preview buffer.
    linenr_preview: LineNr,
    /// Last line of the original buffer already shown.
    linenr_origbuf: LineNr,
    /// Scratch holding one formatted line, grown as the lines get longer
    /// and never shrunk: upstream's `xrealloc`ed `str`, whose length is
    /// `old_line_size`.
    scratch: Vec<u8>,
    /// The size the last real line needed, terminator included.
    /// Deliberately *not* reset for the past-the-end line, which formats
    /// into whatever the previous one left -- and is appended at this size,
    /// so the bytes after its own terminator are the previous line's tail.
    line_size: usize,
}

impl PreviewBuf {
    /// Copy the lines `match` spans out of `orig_buf` into the preview
    /// buffer, and answer where the match sits in what was written.
    fn add_match(&mut self, orig_buf: Buf, m: SubResult) -> (LPos, LPos) {
        let mut p_start = LPos {
            lnum: 0 as LineNr,
            col: m.start.col,
        };
        let mut p_end = LPos {
            lnum: 0 as LineNr,
            col: m.end.col,
        };
        // You Might Gonna Need It.
        buf_ensure_loaded(self.buf);

        let mut next_linenr = if m.pre_match == 0 as LineNr {
            m.start.lnum
        } else {
            m.pre_match
        };
        // Don't add a line twice.
        if next_linenr == self.linenr_origbuf {
            next_linenr += 1;
            // Both may be redefined below.
            p_start.lnum = self.linenr_preview;
            p_end.lnum = self.linenr_preview;
        }

        while next_linenr <= m.end.lnum {
            if next_linenr == m.start.lnum {
                p_start.lnum = self.linenr_preview + 1 as LineNr;
            }
            if next_linenr == m.end.lnum {
                p_end.lnum = self.linenr_preview + 1 as LineNr;
            }
            self.add_line(orig_buf, next_linenr);
            next_linenr += 1;
        }
        self.linenr_origbuf = m.end.lnum;
        (p_start, p_end)
    }

    /// Put `"|lnum| line"` into the scratch and append it to the preview.
    fn add_line(&mut self, orig_buf: Buf, lnum: LineNr) {
        let line = if lnum == orig_buf.b_ml.ml_line_count + 1 as LineNr {
            Vec::new()
        } else {
            let line = Lines::in_buffer(orig_buf).line(lnum).to_vec();
            self.line_size = line.len() + self.col_width as usize + 1;
            // Grow the scratch if the line is longer than any before it.
            if self.line_size > self.scratch.len() {
                self.scratch.resize(self.line_size, 0);
            }
            line
        };
        if self.line_size == 0 {
            // Nothing was ever formatted: upstream's scratch is still null.
            return;
        }
        // `snprintf(str, line_size, "|%*d| %s", col_width - 3, lnum, line)`.
        let width = (self.col_width - 3 as c_int).unsigned_abs() as usize;
        let mut text = if self.col_width - 3 as c_int >= 0 {
            format!("|{lnum:>width$}| ").into_bytes()
        } else {
            format!("|{lnum:<width$}| ").into_bytes()
        };
        text.extend_from_slice(&line);
        text.truncate(self.line_size - 1);
        self.scratch[..text.len()].copy_from_slice(&text);
        self.scratch[text.len()] = 0;
        if self.linenr_preview == 0 as LineNr {
            let _ = ml_replace_buf_text(self.buf, 1 as LineNr, &text);
        } else {
            // The whole `line_size`, as upstream appends it: past the
            // terminator just written may lie the previous line's tail.
            let appended = &self.scratch[..self.line_size - 1];
            let _ = ml_append_bytes(self.buf, self.linenr_preview, appended);
        }
        self.linenr_preview += 1;
    }
}

/// Show the `'inccommand'` preview: highlight every match in the buffer, and
/// with `inccommand=split` also list the substituted lines in the preview
/// buffer.
///
/// Returns 1 when only highlights were added and 2 when the preview window
/// should be shown, which is what `cmdpreview_may_show` switches on.
pub(crate) fn show_sub(
    range: (LineNr, LineNr),
    old_cusr: Pos,
    preview_lines: &PreviewLines,
    hl_id: c_int,
    cmdpreview_ns: c_int,
    cmdpreview_bufnr: Handle,
) -> c_int {
    let save_shm: CString = p_shm(CStr::to_owned);
    let orig_buf = Buf::current();

    // Disable the file info message.
    set_option_direct(
        kOptShortmess,
        static_cstr_optval(c"F"),
        OptionSetFlags::NONE,
        SID_NONE,
    );

    // Place the cursor on the nearest matching line, to undo do_sub()'s
    // placement.  If all the matches are above, do_sub() already put it right.
    if let Some(curres) = preview_lines
        .subresults
        .iter()
        .find(|r| r.start.lnum >= old_cusr.lnum)
    {
        Win::current().w_cursor.lnum = curres.start.lnum;
        Win::current().w_cursor.col = curres.start.col;
    }

    // Update the topline so that the main window is on the correct line.
    update_topline(Win::current());

    // Use the preview window only when inccommand=split and the range is more
    // than the current line.
    let preview =
        P_ICM.first_byte() == b's' && (range.0 != old_cusr.lnum || range.1 != old_cusr.lnum);

    let mut pv = if preview {
        // With 'inccommand' at `split` the caller's contract makes
        // `cmdpreview_bufnr` the live preview buffer, and upstream `assert`s
        // exactly that -- but the assert is compiled out under `NDEBUG` and
        // every use below it sits behind `if (cmdpreview_buf)`, so a release
        // build that somehow lost the buffer *skips the preview* rather than
        // faulting. Keep both halves: the debug assertion, and the skip.
        let buf = find_buf(cmdpreview_bufnr as c_int);
        debug_assert!(buf.is_some(), "cmdpreview_buf != NULL");
        buf.map(|buf| {
            // Width of the "|lnum|..." column, from the highest line number
            // in the last match -- whose `end.lnum` may be 0 under `n`.
            let mut col_width = 0 as c_int;
            if let Some(last) = preview_lines.subresults.last() {
                let highest_lnum = last.start.lnum.max(last.end.lnum);
                debug_assert!(highest_lnum > 0 as LineNr, "highest_lnum > 0");
                col_width = f64::from(highest_lnum).log10() as c_int + 1 as c_int + 3 as c_int;
            }
            PreviewBuf {
                buf,
                col_width,
                linenr_preview: 0 as LineNr,
                linenr_origbuf: 0 as LineNr,
                scratch: Vec::new(),
                line_size: 0,
            }
        })
    } else {
        None
    };

    for &m in &preview_lines.subresults {
        if let Some(pv) = pv.as_mut() {
            let (p_start, p_end) = pv.add_match(orig_buf, m);
            bufhl_add_hl_pos_offset(
                pv.buf,
                cmdpreview_ns,
                hl_id,
                p_start,
                p_end,
                pv.col_width as ColNr,
            );
        }
        let (start, end) = (m.start, m.end);
        bufhl_add_hl_pos_offset(orig_buf, cmdpreview_ns, hl_id, start, end, 0);
    }

    drop(pv);
    // The value borrows rather than copies, so the saved option must
    // outlive the call that puts it back -- which is why it is dropped here
    // and not before. `set_option_direct` copies what it is given.
    set_option_direct(
        kOptShortmess,
        OptVal::String(OptStr::borrowing_bytes(save_shm.as_bytes())),
        OptionSetFlags::NONE,
        SID_NONE,
    );
    drop(save_shm);

    if preview { 2 as c_int } else { 1 as c_int }
}

/// The `:substitute` command.
pub fn ex_substitute(excmd: &mut ExArg) {
    do_sub(excmd, profile_zero(), 0 as c_int, 0 as Handle);
}

/// The `:substitute` command's `'inccommand'` preview callback.
pub fn ex_substitute_preview(
    excmd: &mut ExArg,
    cmdpreview_ns: c_int,
    cmdpreview_bufnr: Handle,
) -> c_int {
    // Only preview once the pattern delimiter has been typed.
    let first = excmd.line.byte_at(excmd.line.arg);
    if first == 0 || first.is_ascii_alphabetic() || ascii_isdigit(first as c_int) {
        return 0 as c_int;
    }
    // `do_sub` may move `args.arg`, which the caller still needs where it was.
    let save_arg = excmd.line.arg;
    let retv = do_sub(
        excmd,
        profile_setlimit(p_rdt() as int64_t),
        cmdpreview_ns,
        cmdpreview_bufnr,
    );
    excmd.line.arg = save_arg;
    retv
}
