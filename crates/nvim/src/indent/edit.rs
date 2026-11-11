//! Reindenting text: the `=` operator, `:retab`, the `<C-t>`/`<C-d>`
//! shifts, Insert-mode smart indent, and copying an existing line's indent.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::extmark::EXTMARK_UNDO;
use crate::memory::XString;
use crate::message::IOSIZE;
use crate::narrow::as_index;
use crate::narrow::len_as_int;
use crate::narrow::number_as_col;
use crate::narrow::number_as_int;
use crate::normal::defs::OpArg;
use crate::option::defs::{OptVal, OptionSetFlags};
use crate::options::OPT_VARTABSTOP;
use crate::os::libc::int64_t;
use crate::plines::CharsizeArg;
use crate::types::{ColNr, ExArg, Failed, LineNr, OptInt, Pos};
use core::ffi::CStr;

use super::*;
use crate::ascii::{ascii_iswhite, ascii_iswhite_or_nul};
use crate::change::{changed_lines, ins_bytes, ins_str};
use crate::charset::skip;
use crate::cstr::byte_at;
use crate::cursor::coladvance;
use crate::drawscreen::{UPD_INVERTED, UPD_NOT_VALID, redraw_curbuf_later};
use crate::edit::{
    BeginlineOpts, backspace_until_column, beginline, replace_join, replace_push_nul,
};
use crate::ex_docmd::cmdmod_has;
use crate::ex_eval::state::TRYLEVEL;
use crate::extmark::extmark_splice_cols;
use crate::getchar::state::GOT_INT;
use crate::guard::Suppress;
use crate::indent_c::in_cinkeys;
use crate::mbyte::{c_char_info, c_cluster_len, c_next_cluster};
use crate::memline::Lines;
use crate::memline::ml_replace_owned;
use crate::message::{E_INTERR, E_MODIFIABLE, E_RESULTING_TEXT_TOO_LONG};
use crate::message::{emsg, msg_progress};
use crate::r#move::changed_cline_bef_curs;
use crate::ops::shift_line;
use crate::option::set_option_direct;
use crate::option::vars::{p_paste, p_report};
use crate::optionstr::OptString;
use crate::os::cshim::{gettext, ngettext};
use crate::os::input::line_breakcheck;
use crate::plines::{CHAR_CELLS_SPAN, getvcol_nolist, win_charsize, win_chartabsize};
use crate::pos::MAXCOL;
use crate::search::findmatch;
use crate::state::mode::{AI_COL, CAN_SI, CAN_SI_BACK, DID_SI, INSSTART, OLD_INDENT, STATE};
use crate::state::{MODE_INSERT, REPLACE_FLAG, VREPLACE_FLAG};
use crate::types::CmdModFlags;
use crate::types::Key;
use crate::undo::{u_clearline, u_save, u_savecommon};
use crate::winlayer::{Buf, Win};

/// Whether the cursor is before (or, with `extra` zero, on) the first
/// non-blank of the line.
pub fn inindent(extra: c_int) -> bool {
    let cursor = Win::current().w_cursor;
    let col = len_as_int(skip::white(Lines::current().line(cursor.lnum)));
    col >= cursor.col.get() + extra
}

/// Shows `fmt` formatted with `n` as `:indent` progress, cut where
/// upstream's shared `IObuff` would cut it.
fn indent_progress(fmt: &CStr, n: int64_t, status: &CStr) {
    let line = XString::from_bytes(&crate::formatted!(IOSIZE as usize, fmt.to_bytes(), n));
    msg_progress(line.as_cstr(), c"indent", status, 0, true, false);
}

/// Reindents `op`'s lines with `how`, one of the three indent engines.
pub fn op_reindent(op: &OpArg, how: Indenter) {
    let mut win = Win::current();
    let mut buf = Buf::current();
    let start_lnum = win.w_cursor.lnum;
    let line_count = op.line_count;
    if buf.b_p_ma == 0 {
        emsg(gettext(E_MODIFIABLE));
        return;
    }
    let mut first_changed: LineNr = LineNr::ZERO;
    let mut last_changed: LineNr = LineNr::ZERO;
    // Undo once for all the lines: much faster than per line, especially
    // when undoing.
    let mut i = 0;
    if u_savecommon(
        buf,
        start_lnum - 1,
        start_lnum + line_count,
        start_lnum + line_count,
        false,
    )
    .is_ok()
    {
        i = line_count - 1;
        while i >= 0 && !GOT_INT.get() {
            // A slow thing to do, so say so — otherwise it looks hung.
            if i > 1 && (i % 50 == 0 || i == line_count - 1) && i64::from(line_count) > p_report() {
                // Restore the cursor first, so the `msg_show` callback
                // does not redraw `curwin`.
                let save_lnum = win.w_cursor.lnum;
                win.w_cursor.lnum = start_lnum;
                indent_progress(
                    gettext(c"%ld lines to indent... "),
                    int64_t::from(i),
                    c"running",
                );
                win.w_cursor.lnum = save_lnum;
            }
            // Vi-compatible: with Lisp indenting the first line is not
            // indented, unless it is the only line.
            let lisp_first = i == line_count - 1 && line_count != 1 && how == Indenter::Lisp;
            if !lisp_first {
                // A blank line gets no indent rather than the engine's.
                let blank = {
                    let mut lines = Lines::current();
                    let line = lines.line(win.w_cursor.lnum);
                    skip::white(line) == line.len()
                };
                let amount = if blank { 0 } else { how.amount() };
                if amount >= 0 && set_indent(amount, 0) {
                    if first_changed == LineNr::ZERO {
                        first_changed = win.w_cursor.lnum;
                    }
                    last_changed = win.w_cursor.lnum;
                }
            }
            win.w_cursor.lnum += 1;
            win.w_cursor.col = ColNr::ZERO; // keep it valid
            i -= 1;
        }
    }
    // Put the cursor on the first non-blank of the indented line.
    win.w_cursor.lnum = start_lnum;
    beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    // Mark the changed lines for redraw. Under Visual highlighting that
    // has to reach the last line even when nothing changed, so that the
    // highlight goes away.
    if last_changed != LineNr::ZERO {
        let end = if op.is_visual {
            start_lnum + line_count
        } else {
            last_changed + 1
        };
        changed_lines(buf, first_changed, ColNr::ZERO, end, 0, true);
    } else if op.is_visual {
        redraw_curbuf_later(UPD_INVERTED);
    }
    if i64::from(line_count) > p_report() {
        let done = line_count - (i + 1);
        indent_progress(
            ngettext(
                c"%ld line indented ",
                c"%ld lines indented ",
                ::core::ffi::c_ulong::from(done.unsigned_abs()),
            ),
            int64_t::from(done),
            c"success",
        );
    }
    if !cmdmod_has(CmdModFlags::LOCKMARKS) {
        // Set the '[ and '] marks.
        buf.b_op_start = op.start;
        buf.b_op_end = op.end;
    }
}

/// Whether lines starting with `#` should be left aligned.
pub fn preprocs_left() -> bool {
    let buf = Buf::current();
    buf.b_p_si != 0 && buf.b_p_cin == 0
        || buf.b_p_cin != 0
            && in_cinkeys(Key::ascii(b'#'), b' ', true)
            && buf.b_ind_hash_comment == 0
}

/// Whether the conditions are right for smart indenting.
pub fn may_do_si() -> bool {
    let buf = Buf::current();
    buf.b_p_si != 0 && buf.b_p_cin == 0 && buf.b_p_inde.first_byte() == 0 && !p_paste()
}

/// Sets the cursor line's indent to that of the line holding the `{` that
/// `pos` matched — or, when that `{` has a `)` just before it, to the line
/// holding the matching `(`, which is what makes an `if (..\n..) {`
/// spanning several lines come out right (Webb).
fn si_indent_like_open_brace(pos: Pos) {
    let mut win = Win::current();
    let old_pos = win.w_cursor;
    let mut i = pos.col.get();
    // The line is read here and nowhere else: `findmatch` below walks the
    // buffer itself, so the borrow has to be over before it is called.
    let mut lines = Lines::current();
    let line = lines.line(pos.lnum);
    if i > 0 {
        // Skip the blanks before the '{'.
        while {
            i -= 1;
            i > 0 && ascii_iswhite(c_int::from(byte_at(line, as_index(i))))
        } {}
    }
    let after_paren = byte_at(line, as_index(i)) == b')';
    win.w_cursor.lnum = pos.lnum;
    win.w_cursor.col = ColNr::new(i);
    if after_paren && let Some(open) = findmatch(None, Codepoint::byte(b'(')) {
        win.w_cursor = open;
    }
    let indent = get_indent();
    win.w_cursor = old_pos;
    if STATE.get() & VREPLACE_FLAG != 0 {
        change_indent(INDENT_SET, indent, 0, true);
    } else {
        set_indent(indent, SIN_CHANGED);
    }
}

/// Whether a `{` typed after an `O` should reduce this line's indent: it may
/// not go below the indent of the previous real line.
fn si_should_shift_back() -> bool {
    let mut win = Win::current();
    let old_pos = win.w_cursor;
    let here = get_indent();
    let mut lines = Lines::current();
    while win.w_cursor.lnum > LineNr::ONE {
        win.w_cursor.lnum -= 1;
        let line = lines.line(win.w_cursor.lnum);
        let first = byte_at(line, skip::white(line));
        // Ignore empty lines and lines starting with '#'.
        if first != b'#' && first != 0 {
            break;
        }
    }
    let above = get_indent();
    win.w_cursor = old_pos;
    above < here
}

/// Very smart auto-indenting for a "normal" character typed in Insert mode:
/// `{`, `}` and `#`.
pub fn ins_try_si(c: c_int) {
    let win = Win::current();
    if (DID_SI.get() || CAN_SI_BACK.get()) && c == c_int::from(b'{')
        || CAN_SI.get() && c == c_int::from(b'}') && inindent(0)
    {
        let matching = if c == c_int::from(b'}') {
            findmatch(None, Codepoint::byte(b'{'))
        } else {
            None
        };
        if let Some(matching) = matching {
            si_indent_like_open_brace(matching);
        } else if win.w_cursor.col > ColNr::ZERO {
            let shift = !(c == c_int::from(b'{')
                && CAN_SI_BACK.get()
                && win.w_cursor.lnum > LineNr::ONE
                && !si_should_shift_back());
            if shift {
                shift_line(true, false, 1, true);
            }
        }
    }
    // The indent of a '#' is always zero.
    if win.w_cursor.col > ColNr::ZERO && CAN_SI.get() && c == c_int::from(b'#') && inindent(0) {
        // Remember the current indent for the next line.
        OLD_INDENT.set(get_indent());
        set_indent(0, SIN_CHANGED);
    }
    // Adjust `AI_COL`: the character at this position can be deleted.
    AI_COL.set(AI_COL.get().min(win.w_cursor.col));
}

/// Applies the indent [`change_indent`] was asked for, leaving the cursor on
/// the first non-blank.
fn apply_indent(type_0: c_int, amount: c_int, round: c_int, call_changed_bytes: bool) {
    if type_0 == INDENT_SET {
        set_indent(amount, if call_changed_bytes { SIN_CHANGED } else { 0 });
        return;
    }
    let save_state = STATE.get();
    // Avoid being called recursively.
    if STATE.get() & VREPLACE_FLAG != 0 {
        STATE.set(MODE_INSERT);
    }
    shift_line(type_0 == INDENT_DEC, round != 0, 1, call_changed_bytes);
    STATE.set(save_state);
}

/// Puts the cursor `end_vcol` screen columns into the new indent, padding
/// the line with spaces when no character starts exactly there. Answers the
/// byte column the cursor should take.
fn place_cursor_in_indent(end_vcol: c_int) -> c_int {
    let mut win = Win::current();
    win.w_virtcol = ColNr::new(end_vcol);
    let mut new_cursor_col = 0;
    let mut vcol = 0;
    {
        let mut lines = Lines::current();
        let line = lines.line(win.w_cursor.lnum);
        if !line.is_empty() {
            let (mut csarg, cstype) = CharsizeArg::new(win, LineNr::ZERO, line);
            let mut at = 0;
            let mut chr = c_char_info(line, 0);
            loop {
                let next_vcol = vcol + win_charsize(cstype, vcol, at, chr.value, &mut csarg).width;
                if next_vcol > end_vcol {
                    break;
                }
                vcol = next_vcol;
                (at, chr) = c_next_cluster(line, at, chr);
                if byte_at(line, at) == 0 {
                    break;
                }
            }
            new_cursor_col = len_as_int(at);
        }
    }
    if vcol == win.w_virtcol.get() {
        return new_cursor_col;
    }
    // No character starts at that column, so make one: insert the spaces
    // the cursor needs to sit where it was asked to.
    win.w_cursor.col = ColNr::new(new_cursor_col);
    let ptrlen = usize::try_from(win.w_virtcol.get() - vcol).expect("non-negative");
    ins_str(&vec![b' '; ptrlen]);
    new_cursor_col + len_as_int(ptrlen)
}

/// Moves `INSSTART` and `AI_COL` back by what the indent lost, so that the
/// insert still starts where the user began typing.
fn adjust_insert_start(insstart_less: c_int) {
    let mut insstart = INSSTART.get();
    let lnum = Win::current().w_cursor.lnum;
    if lnum == insstart.lnum && insstart.col != ColNr::ZERO {
        insstart.col = if insstart.col.get() <= insstart_less {
            ColNr::ZERO
        } else {
            insstart.col - insstart_less
        };
        INSSTART.set(insstart);
    }
    AI_COL.set(if AI_COL.get().get() <= insstart_less {
        ColNr::ZERO
    } else {
        AI_COL.get() - insstart_less
    });
}

/// Fixes the Replace-mode stack after the indent moved the cursor: pop what
/// the line lost, push NULs for what it gained.
fn fix_replace_stack(mut start_col: c_int) {
    let win = Win::current();
    while start_col > win.w_cursor.col.get() {
        replace_join(0); // remove a NUL from the replace stack
        start_col -= 1;
    }
    while start_col < win.w_cursor.col.get() {
        replace_push_nul();
        start_col += 1;
    }
}

/// Fixes the Virtual Replace stack: put the original line back, then
/// backspace over it and type the new one — always possible, because the
/// whole line is replayed. `orig_line`/`orig_col` are what [`change_indent`]
/// saved.
fn vreplace_restore(orig_line: XString, orig_col: ColNr) {
    let mut win = Win::current();
    // The new line, but only up to the cursor.
    let new_col = win.w_cursor.col;
    let new_line = {
        let mut lines = Lines::current();
        let line = lines.line(win.w_cursor.lnum);
        line[..new_col.as_usize().min(line.len())].to_vec()
    };
    let _ = ml_replace_owned(win.w_cursor.lnum, orig_line);
    win.w_cursor.col = orig_col;
    let splice = Suppress::splice();
    backspace_until_column(0);
    ins_bytes(&new_line);
    drop(splice);
    let delta = orig_col.get() - new_col.get();
    extmark_splice_cols(
        Buf::current(),
        win.w_cursor.lnum.get() - 1,
        new_col,
        if delta < 0 {
            ColNr::new(-delta)
        } else {
            ColNr::ZERO
        },
        if delta > 0 {
            ColNr::new(delta)
        } else {
            ColNr::ZERO
        },
        EXTMARK_UNDO,
    );
}

/// Inserts an indent (`<Tab>`/`<C-t>`), deletes one (`<C-d>`) or sets one,
/// keeping the cursor on the same character. `round` rounds to 'shiftwidth'
/// and applies only to the two shifts; `call_changed_bytes` asks for
/// `changed_bytes()`.
pub fn change_indent(type_0: c_int, amount: c_int, round: c_int, call_changed_bytes: bool) {
    let mut win = Win::current();
    // Virtual Replace needs to know what the line looked like before.
    let orig = (STATE.get() & VREPLACE_FLAG != 0).then(|| {
        (
            XString::from_bytes(Lines::current().line(win.w_cursor.lnum)),
            win.w_cursor.col,
        )
    });
    // The tricks below do not want 'list' mode.
    let save_p_list = win.w_onebuf_opt.wo_list;
    win.w_onebuf_opt.wo_list = 0;
    // A copy of the cursor, written back: the walk reads the window.
    let mut cursor = win.w_cursor;
    let mut vcol = getvcol_nolist(&mut cursor).get();
    win.w_cursor = cursor;
    // Replace mode fixes its stack below, which is only possible when the
    // cursor is in the indent; this is how many characters precede it.
    let mut start_col = win.w_cursor.col.get();
    // Offset of the cursor from the first non-blank.
    let mut new_cursor_col = win.w_cursor.col.get();
    beginline(BeginlineOpts::WHITE);
    new_cursor_col -= win.w_cursor.col.get();
    let mut insstart_less = win.w_cursor.col.get();
    if new_cursor_col < 0 {
        // The cursor is inside the indent: how many screen columns it
        // sits left of the first non-blank.
        vcol = get_indent() - vcol;
    }
    if new_cursor_col > 0 {
        start_col = -1; // the replace stack cannot be fixed
    }
    apply_indent(type_0, amount, round, call_changed_bytes);
    insstart_less -= win.w_cursor.col.get();

    // Try to keep the cursor on the same character: at or after the first
    // non-blank that is a byte offset from it, outside Insert mode it just
    // stays on the first non-blank, and inside the indent it is a screen
    // column, which may need spaces inserted to reach.
    if new_cursor_col >= 0 {
        if new_cursor_col == 0 {
            // The cursor is touching the indent, so `INSSTART.col` resets.
            insstart_less = c_int::MAX;
        }
        new_cursor_col += win.w_cursor.col.get();
    } else if STATE.get() & MODE_INSERT == 0 {
        new_cursor_col = win.w_cursor.col.get();
    } else {
        new_cursor_col = place_cursor_in_indent((get_indent() - vcol).max(0));
        insstart_less = c_int::MAX;
    }

    win.w_onebuf_opt.wo_list = save_p_list;
    win.w_cursor.col = ColNr::new(new_cursor_col.max(0));
    win.w_set_curswant = true;
    changed_cline_bef_curs(win);
    if STATE.get() & MODE_INSERT != 0 {
        adjust_insert_start(insstart_less);
    }
    if STATE.get() & REPLACE_FLAG != 0 && STATE.get() & VREPLACE_FLAG == 0 && start_col >= 0 {
        fix_replace_stack(start_col);
    }
    if let Some((orig_line, orig_col)) = orig {
        vreplace_restore(orig_line, orig_col);
    }
}

/// Copies `src`'s indent onto the cursor line, filling to `size` columns and
/// leaving the cursor on the first non-blank.
///
/// Upstream runs the body twice — once to size the allocation, once to fill
/// it. The indent is built into a `Vec` here instead, so the shape is stated
/// once.
///
/// `src` is the line whose indent is copied; only its leading white space is
/// read.
pub fn copy_indent(size: c_int, src: &[u8]) -> bool {
    let buf = Buf::current();
    let pad = |col: c_int| buf.tab_padding(ColNr::new(col));
    let expandtab = buf.b_p_et != 0;
    let src_indent = src;

    let mut indent: Vec<u8> = Vec::new();
    let mut todo = size;
    let mut ind_done = 0;
    let mut ind_col = 0;

    // Copy the usable part of the source's own indent.
    for &b in src_indent {
        if todo <= 0 || !(b == b' ' || b == b'\t') {
            break;
        }
        if b == b'\t' {
            let tab_pad = pad(ind_done);
            if todo < tab_pad {
                break; // this tab would overshoot the target
            }
            todo -= tab_pad;
            ind_done += tab_pad;
            ind_col += tab_pad;
        } else {
            todo -= 1;
            ind_done += 1;
            ind_col += 1;
        }
        indent.push(b);
    }
    // Fill to the next tabstop with a tab, if one fits.
    let tab_pad = pad(ind_done);
    if todo >= tab_pad && !expandtab {
        todo -= tab_pad;
        ind_col += tab_pad;
        indent.push(b'\t');
    }
    // Whole tabs for as much of the rest as they cover.
    if !expandtab {
        loop {
            let tab_pad = pad(ind_col);
            if todo < tab_pad {
                break;
            }
            todo -= tab_pad;
            ind_col += tab_pad;
            indent.push(b'\t');
        }
    }
    // Spaces for what is left.
    indent.resize(indent.len() + as_index(todo.max(0)), b' ');
    let ind_len = len_as_int(indent.len());

    // The indent, then the whole of the cursor line.
    let lnum = Win::current().w_cursor.lnum;
    indent.extend_from_slice(Lines::current().line(lnum));
    let _ = ml_replace_owned(lnum, XString::from_vec(indent));
    Win::current().w_cursor.col = ColNr::new(ind_len);
    true
}

/// Reports "resulting text is too long", and breaks out of any loop when
/// there is no `:try` to catch it.
fn emsg_text_too_long() {
    emsg(gettext(E_RESULTING_TEXT_TOO_LONG));
    if TRYLEVEL.get() == 0 {
        GOT_INT.set(true);
    }
}

/// The tabstops `:retab` was asked to retabulate to.
#[derive(Debug)]
struct RetabTabs {
    /// The list to measure against: the one given, or a copy of the
    /// buffer's own.
    vts: TabStopArray,
    /// The argument text, when it named a *new* list that must also be set
    /// on the buffer afterwards; `None` when the buffer's own list is
    /// reused, in which case no option is touched.
    ts_str: Option<XString>,
    /// `:retab -indentonly`: stop at the end of each line's indent.
    indent_only: bool,
}

/// Parses `:retab`'s argument. `Err` is a malformed tabstop list, which
/// `tabstop_set` has already reported.
fn parse_retab_arg(arg: &[u8]) -> Result<RetabTabs, Failed> {
    let mut at = 0;
    let indent_only =
        arg.starts_with(b"-indentonly") && ascii_iswhite_or_nul(c_int::from(byte_at(arg, 11)));
    if indent_only {
        at = 11 + skip::white(&arg[11..]);
    }
    let ts_start = at;
    let vts = tabstop_set(&arg[at..])?;
    while byte_at(arg, at).is_ascii_digit() || byte_at(arg, at) == b',' {
        at += 1;
    }
    Ok(match vts {
        // No list given: measure against the buffer's own.
        None => RetabTabs {
            vts: Buf::current().b_p_vts_array.clone(),
            ts_str: None,
            indent_only,
        },
        Some(vts) => RetabTabs {
            vts: Some(vts),
            ts_str: Some(XString::from_bytes(&arg[ts_start..at])),
            indent_only,
        },
    })
}

/// How one white-space run's retabulation ended.
#[derive(Debug)]
enum Retabulated {
    /// Either the line was rewritten or it did not need to be.
    Done,
    /// The rewritten line would not fit; reported, stop this line.
    TooLong,
    /// `u_save` failed; stop the command.
    OutOfMemory,
}

/// `:retab`'s state.
///
/// `got_tab`, `num_spaces` and the start of the run are at *function* scope
/// upstream, so a white-space run that reaches the end of a line is still
/// open when the next line starts. That is observable, so they live here
/// rather than being per-line locals.
#[derive(Debug, Default)]
struct Retab {
    got_tab: bool,
    num_spaces: c_int,
    /// Byte and screen column where the current white-space run began.
    start_col: c_int,
    start_vcol: int64_t,
    first_line: LineNr,
    last_line: LineNr,
}

/// One line's walk: where the scan has got to, and the line as it stands.
#[derive(Debug)]
struct LineScan {
    lnum: LineNr,
    /// The line's text, a copy: the walk rewrites the line under it.
    text: Vec<u8>,
    /// Byte and screen column the scan has reached.
    col: c_int,
    vcol: int64_t,
    /// Whether `u_save` has already run for this line.
    did_undo: bool,
}

impl Retab {
    /// Rewrites the white-space run that just ended, when doing so is
    /// shorter than what is there (or 'expandtab' or a tab demands it).
    /// `scan` is moved onto the rewritten line.
    fn retabulate(&mut self, scan: &mut LineScan, tabs: &RetabTabs) -> Retabulated {
        let buf = Buf::current();
        // The run's width on screen.
        let width = number_as_int(scan.vcol - self.start_vcol);
        self.num_spaces = width;
        let mut num_tabs = 0;
        if buf.b_p_et == 0 {
            let (start, end) = (number_as_col(self.start_vcol), number_as_col(scan.vcol));
            (num_tabs, self.num_spaces) = match tabstops(tabs.vts.as_deref()) {
                Some(stops) => stops.from_to(start, end),
                None => {
                    let ts = number_as_int(buf.b_p_ts);
                    debug_assert!(ts != 0);
                    tabstop::uniform_from_to(start, end, ts)
                }
            };
        }
        if !(buf.b_p_et != 0 || self.got_tab || self.num_spaces + num_tabs < width) {
            return Retabulated::Done;
        }
        if !scan.did_undo {
            scan.did_undo = true;
            if u_save(scan.lnum - 1, scan.lnum + 1).is_err() {
                return Retabulated::OutOfMemory;
            }
        }
        // How many characters the run will actually take.
        let len = self.num_spaces + num_tabs;
        let old_len = len_as_int(scan.text.len());
        let new_len = old_len - scan.col + self.start_col + len + 1;
        if new_len <= 0 || new_len == c_int::MAX {
            emsg_text_too_long();
            return Retabulated::TooLong;
        }
        let mut new_line = Vec::with_capacity(as_index(new_len));
        new_line.extend_from_slice(&scan.text[..as_index(self.start_col)]);
        new_line.extend((0..len).map(|i| if i < num_tabs { b'\t' } else { b' ' }));
        new_line.extend_from_slice(&scan.text[as_index(scan.col)..]);
        if ml_replace_owned(scan.lnum, XString::from_bytes(&new_line)).is_ok() {
            let lnum = scan.lnum.get() - 1;
            let (old, new) = (old_len, new_len - 1);
            extmark_splice_cols(
                buf,
                lnum,
                ColNr::ZERO,
                ColNr::new(old),
                ColNr::new(new),
                EXTMARK_UNDO,
            );
        }
        if self.first_line == LineNr::ZERO {
            self.first_line = scan.lnum;
        }
        self.last_line = scan.lnum;
        scan.text = new_line;
        scan.col = self.start_col + len;
        Retabulated::Done
    }

    /// Retabulates one line. Answers false when the command has to stop.
    fn line(&mut self, lnum: LineNr, tabs: &RetabTabs, forceit: bool) -> bool {
        let mut scan = LineScan {
            lnum,
            text: Lines::current().line(lnum).to_vec(),
            col: 0,
            vcol: 0,
            did_undo: false,
        };
        let byte = |s: &LineScan| byte_at(&s.text, as_index(s.col));
        let win = Win::current();
        let width = |s: &LineScan| {
            let at = as_index(s.col);
            let text = &s.text[at..(at + CHAR_CELLS_SPAN).min(s.text.len())];
            win_chartabsize(win, text, number_as_col(s.vcol))
        };
        let charlen = |s: &LineScan| len_as_int(c_cluster_len(&s.text[as_index(s.col)..]));
        loop {
            let at = byte(&scan);
            if ascii_iswhite(c_int::from(at)) {
                if !self.got_tab && self.num_spaces == 0 {
                    // The first character of a run of white space.
                    self.start_vcol = scan.vcol;
                    self.start_col = scan.col;
                }
                if at == b' ' {
                    self.num_spaces += 1;
                } else {
                    self.got_tab = true;
                }
            } else {
                if self.got_tab || (forceit && self.num_spaces > 1) {
                    match self.retabulate(&mut scan, tabs) {
                        Retabulated::Done => {}
                        Retabulated::TooLong => break,
                        Retabulated::OutOfMemory => return false,
                    }
                }
                self.got_tab = false;
                self.num_spaces = 0;
                if tabs.indent_only {
                    break;
                }
            }
            if byte(&scan) == 0 {
                break;
            }
            scan.vcol += int64_t::from(width(&scan));
            if scan.vcol >= int64_t::from(MAXCOL) {
                emsg_text_too_long();
                break;
            }
            scan.col += charlen(&scan);
        }
        true
    }
}

/// Applies the tabstop list `:retab` was given (`vts`, spelled `ts_str`) to
/// the buffer.
fn set_retab_tabstop(vts: TabStopArray, ts_str: &XString) {
    let mut buf = Buf::current();
    if stop_count(buf.b_p_vts_array.as_deref()) > 0 || stop_count(vts.as_deref()) > 1 {
        // 'vartabstop' is in use, or more than one stop was given.
        set_option_direct(
            OPT_VARTABSTOP,
            OptVal::from_cstr(ts_str.as_cstr()),
            OptionSetFlags::LOCAL,
            0,
        );
        buf.b_p_vts_array = vts;
    } else {
        // A single stop with 'vartabstop' unused is 'tabstop'.
        buf.b_p_ts = OptInt::from(first_stop(vts.as_deref()));
    }
}

/// `:retab`.
pub fn ex_retab(excmd: &mut ExArg) {
    let mut win = Win::current();
    let buf = Buf::current();
    let save_list = win.w_onebuf_opt.wo_list;
    win.w_onebuf_opt.wo_list = 0; // 'list' mode is not wanted here
    let Ok(tabs) = parse_retab_arg(excmd.line.arg()) else {
        // Upstream returns here without restoring 'list', which it has
        // already cleared. Kept: a `:retab` with a malformed tabstop
        // list leaves 'list' off, and that is observable.
        return;
    };
    let mut retab = Retab::default();
    let mut lnum = excmd.line1;
    while !GOT_INT.get() && lnum <= excmd.line2 {
        if !retab.line(lnum, &tabs, excmd.forceit) {
            break;
        }
        line_breakcheck();
        lnum += 1;
    }
    if GOT_INT.get() {
        emsg(gettext(E_INTERR));
    }
    // A single value given is equal to either 'tabstop' or 'vartabstop',
    // and then nothing on screen changed.
    let (own, given) = (buf.b_p_vts_array.as_deref(), tabs.vts.as_deref());
    let unchanged = stop_count(own) == 0
        && stop_count(given) == 1
        && buf.b_p_ts == OptInt::from(first_stop(given))
        || stop_count(own) > 0 && tabstop::eq(own, given);
    if !unchanged {
        redraw_curbuf_later(UPD_NOT_VALID);
    }
    if retab.first_line != LineNr::ZERO {
        changed_lines(
            buf,
            retab.first_line,
            ColNr::ZERO,
            retab.last_line + 1,
            0,
            true,
        );
    }
    win.w_onebuf_opt.wo_list = save_list; // restore 'list'
    if let Some(ts_str) = &tabs.ts_str {
        set_retab_tabstop(tabs.vts, ts_str);
    }
    coladvance(win, win.w_curswant);
    u_clearline(buf);
}
