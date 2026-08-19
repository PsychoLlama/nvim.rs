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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::private::helpers::cstr_as_string;
use crate::ascii::ascii_isdigit;
use crate::buffer::{buf_ensure_loaded, buflist_findnr};
use crate::decoration::bufhl_add_hl_pos_offset;
use crate::ex_cmds::{PreviewLines, SID_NONE, SubResult, do_sub, kOptValTypeString};
use crate::main::{
    KeyTyped, curbuf, curwin, e_interr, got_int, msg_buf, p_icm, p_rdt, p_report, p_shm,
    sub_nlines, sub_nsubs,
};
use crate::memline::{ml_append_buf, ml_get_buf, ml_get_buf_len, ml_replace_buf};
use crate::memory::{xfree, xrealloc, xstrdup};
use crate::message::{MSG_BUF_LEN, emsg, messaging, msg, set_keep_msg};
use crate::r#move::update_topline;
use crate::option::set_option_direct;
use crate::options::kOptShortmess;
use crate::os::cshim::{gettext, ngettext, snprintf};
use crate::profile::{profile_setlimit, profile_zero};
use crate::strings::vim_snprintf_add;
use crate::types::{
    NUL, OptInt, OptVal, OptValData, OptionSetFlags, String_0, buf_T, colnr_T, exarg_T, handle_T,
    int64_t, linenr_T, lpos_T, pos_T, size_t,
};
use ::libc::strcpy;
use core::ffi::{CStr, c_char, c_int, c_ulong, c_void};
use core::ptr;

/// An option value that borrows a static C string, for the two options this
/// module sets and puts back.
pub(crate) fn static_cstr_optval(s: &'static CStr) -> OptVal {
    OptVal {
        type_0: kOptValTypeString,
        data: OptValData {
            string: String_0 {
                data: s.as_ptr() as *mut c_char,
                size: s.to_bytes().len() as size_t,
            },
        },
    }
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
///
/// # Safety
/// Main thread, message state.
pub unsafe fn do_sub_msg(count_only: bool) -> bool {
    // Only report substitutions when there were more than 'report' of them,
    // the command was typed by the user or more than one line changed, and
    // messages are not disabled.
    let worth_reporting = sub_nsubs.get() as OptInt > p_report.get()
        && (KeyTyped.get() || sub_nlines.get() > 1 as linenr_T || p_report.get() < 1 as OptInt);
    // SAFETY: message state.
    if (worth_reporting || count_only) && unsafe { messaging() } {
        let buf = msg_buf.ptr() as *mut c_char;
        let forms = report_forms(count_only);
        let nsubs = sub_nsubs.get();
        let nlines = sub_nlines.get();
        // SAFETY: `msg_buf` is `MSG_BUF_LEN` bytes and no reference into it
        // is outstanding; the format strings come from the catalogue and take
        // exactly the two `int64_t` given.
        unsafe {
            if got_int.get() {
                strcpy(buf, gettext(c"(Interrupted) ".as_ptr()));
            } else {
                *buf = NUL as c_char;
            }
            let single = ngettext(forms[0][0].as_ptr(), forms[0][1].as_ptr(), nsubs as c_ulong);
            let plural = ngettext(forms[1][0].as_ptr(), forms[1][1].as_ptr(), nsubs as c_ulong);
            vim_snprintf_add(
                buf,
                MSG_BUF_LEN as size_t,
                ngettext(single, plural, nlines as c_ulong),
                nsubs as int64_t,
                nlines as int64_t,
            );
            if msg(buf, 0 as c_int) {
                // Save the message to display it after a redraw.
                set_keep_msg(buf, 0 as c_int);
            }
        }
        return true;
    }
    if got_int.get() {
        // SAFETY: a live message string.
        unsafe { emsg(gettext(&raw const e_interr as *const c_char)) };
        return true;
    }
    false
}

/// The `[Preview]` buffer being filled, and how far the fill has reached.
struct PreviewBuf {
    buf: *mut buf_T,
    /// Width of the "|lnum| " column that carries the line numbers.
    col_width: c_int,
    /// Last line added to the preview buffer.
    linenr_preview: linenr_T,
    /// Last line of the original buffer already shown.
    linenr_origbuf: linenr_T,
    /// Scratch holding one formatted line, grown as the lines get longer.
    str: *mut c_char,
    old_line_size: colnr_T,
    /// The size the last real line needed.  Deliberately *not* reset for the
    /// past-the-end line, which formats into whatever the previous one left.
    line_size: colnr_T,
}

impl PreviewBuf {
    /// Copy the lines `match` spans out of `orig_buf` into the preview
    /// buffer, and answer where the match sits in what was written.
    ///
    /// # Safety
    /// Main thread; `orig_buf` must be live and `self.buf` a real buffer.
    unsafe fn add_match(&mut self, orig_buf: *mut buf_T, m: SubResult) -> (lpos_T, lpos_T) {
        let mut p_start = lpos_T {
            lnum: 0 as linenr_T,
            col: m.start.col,
        };
        let mut p_end = lpos_T {
            lnum: 0 as linenr_T,
            col: m.end.col,
        };
        // You Might Gonna Need It.
        // SAFETY: caller's contract.
        unsafe { buf_ensure_loaded(self.buf) };

        let mut next_linenr = if m.pre_match == 0 as linenr_T {
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
                p_start.lnum = self.linenr_preview + 1 as linenr_T;
            }
            if next_linenr == m.end.lnum {
                p_end.lnum = self.linenr_preview + 1 as linenr_T;
            }
            // SAFETY: `next_linenr` is a line of `orig_buf`, or one past its
            // last, which is the empty-line case.
            unsafe { self.add_line(orig_buf, next_linenr) };
            next_linenr += 1;
        }
        self.linenr_origbuf = m.end.lnum;
        (p_start, p_end)
    }

    /// Put `"|lnum| line"` into the scratch and append it to the preview.
    ///
    /// # Safety
    /// Main thread; `orig_buf` must be live and `lnum` one of its lines or
    /// one past the last.
    unsafe fn add_line(&mut self, orig_buf: *mut buf_T, lnum: linenr_T) {
        // SAFETY: caller's contract.
        let line = unsafe {
            if lnum == (*orig_buf).b_ml.ml_line_count + 1 as linenr_T {
                c"".as_ptr() as *mut c_char
            } else {
                let line = ml_get_buf(orig_buf, lnum);
                self.line_size = ml_get_buf_len(orig_buf, lnum) + self.col_width + 1 as c_int;
                // Reallocate if the line is not long enough.
                if self.line_size > self.old_line_size {
                    self.str =
                        xrealloc(self.str as *mut c_void, self.line_size as size_t) as *mut c_char;
                    self.old_line_size = self.line_size;
                }
                line
            }
        };
        // SAFETY: the scratch holds `line_size` bytes, which is what bounds
        // the write; the format takes an `int` width, a line number and a
        // string.
        unsafe {
            snprintf(
                self.str,
                self.line_size as size_t,
                c"|%*d| %s".as_ptr(),
                self.col_width - 3 as c_int,
                lnum,
                line,
            );
            if self.linenr_preview == 0 as linenr_T {
                ml_replace_buf(self.buf, 1 as linenr_T, self.str, true, false);
            } else {
                ml_append_buf(
                    self.buf,
                    self.linenr_preview,
                    self.str,
                    self.line_size,
                    false,
                );
            }
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
///
/// # Safety
/// Main thread; `eap` must be live, and `cmdpreview_bufnr` name the preview
/// buffer when 'inccommand' is `split`.
pub(crate) unsafe fn show_sub(
    eap: *mut exarg_T,
    old_cusr: pos_T,
    preview_lines: &PreviewLines,
    hl_id: c_int,
    cmdpreview_ns: c_int,
    cmdpreview_bufnr: handle_T,
) -> c_int {
    // SAFETY: 'shortmess' is a live string option value.
    let save_shm_p = unsafe { xstrdup(p_shm.get()) };
    let orig_buf = curbuf.get();

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
        // SAFETY: the current window is live.
        unsafe {
            (*curwin.get()).w_cursor.lnum = curres.start.lnum;
            (*curwin.get()).w_cursor.col = curres.start.col;
        }
    }

    // Update the topline so that the main window is on the correct line.
    // SAFETY: the current window is live.
    unsafe { update_topline(curwin.get()) };

    // Use the preview window only when inccommand=split and the range is more
    // than the current line.
    // SAFETY: 'inccommand' is a live string option, `eap` is the caller's.
    let preview = unsafe {
        *p_icm.get() as u8 == b's'
            && ((*eap).line1 != old_cusr.lnum || (*eap).line2 != old_cusr.lnum)
    };

    let mut pv = if preview {
        let buf = buflist_findnr(cmdpreview_bufnr as c_int);
        debug_assert!(!buf.is_null(), "cmdpreview_buf != NULL");
        // Width of the "|lnum|..." column, from the highest line number in
        // the last match -- whose `end.lnum` may be 0 under the `n` flag.
        let mut col_width = 0 as c_int;
        if let Some(last) = preview_lines.subresults.last() {
            let highest_lnum = last.start.lnum.max(last.end.lnum);
            debug_assert!(highest_lnum > 0 as linenr_T, "highest_lnum > 0");
            col_width = f64::from(highest_lnum).log10() as c_int + 1 as c_int + 3 as c_int;
        }
        Some(PreviewBuf {
            buf,
            col_width,
            linenr_preview: 0 as linenr_T,
            linenr_origbuf: 0 as linenr_T,
            str: ptr::null_mut(),
            old_line_size: 0 as colnr_T,
            line_size: 0 as colnr_T,
        })
    } else {
        None
    };

    for &m in &preview_lines.subresults {
        if let Some(pv) = pv.as_mut() {
            // SAFETY: `orig_buf` is the buffer the matches were found in.
            let (p_start, p_end) = unsafe { pv.add_match(orig_buf, m) };
            // SAFETY: the preview buffer and namespace are live.
            unsafe {
                bufhl_add_hl_pos_offset(
                    pv.buf,
                    cmdpreview_ns,
                    hl_id,
                    p_start,
                    p_end,
                    pv.col_width as colnr_T,
                )
            };
        }
        // SAFETY: as above, over the buffer the match came from.
        unsafe {
            bufhl_add_hl_pos_offset(orig_buf, cmdpreview_ns, hl_id, m.start, m.end, 0 as colnr_T)
        };
    }

    // SAFETY: the scratch and the saved option string are both ours.
    unsafe {
        if let Some(pv) = pv {
            xfree(pv.str as *mut c_void);
        }
        set_option_direct(
            kOptShortmess,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(save_shm_p),
                },
            },
            OptionSetFlags::NONE,
            SID_NONE,
        );
        xfree(save_shm_p as *mut c_void);
    }

    if preview { 2 as c_int } else { 1 as c_int }
}

/// The `:substitute` command.
///
/// # Safety
/// Main thread; `eap` must be the live Ex-command argument.
pub unsafe fn ex_substitute(eap: *mut exarg_T) {
    // SAFETY: caller's contract.
    unsafe { do_sub(eap, profile_zero(), 0 as c_int, 0 as handle_T) };
}

/// The `:substitute` command's `'inccommand'` preview callback.
///
/// # Safety
/// Main thread; `eap` must be the live Ex-command argument.
pub unsafe fn ex_substitute_preview(
    eap: *mut exarg_T,
    cmdpreview_ns: c_int,
    cmdpreview_bufnr: handle_T,
) -> c_int {
    // Only preview once the pattern delimiter has been typed.
    // SAFETY: caller's contract -- the argument is NUL-terminated.
    let first = unsafe { *(*eap).arg } as u8;
    if first == 0 || first.is_ascii_alphabetic() || ascii_isdigit(first as c_int) {
        return 0 as c_int;
    }
    // SAFETY: caller's contract; `do_sub` may move `eap->arg`, which the
    // caller still needs where it was.
    unsafe {
        let save_eap = (*eap).arg;
        let retv = do_sub(
            eap,
            profile_setlimit(p_rdt.get() as int64_t),
            cmdpreview_ns,
            cmdpreview_bufnr,
        );
        (*eap).arg = save_eap;
        retv
    }
}
