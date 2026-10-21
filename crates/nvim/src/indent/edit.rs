//! Reindenting text: the `=` operator, `:retab`, the `<C-t>`/`<C-d>`
//! shifts, Insert-mode smart indent, and copying an existing line's indent.

#![forbid(unsafe_code)]

use crate::memory::XString;
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
use crate::ex_eval::state::trylevel;
use crate::extmark::extmark_splice_cols;
use crate::getchar::state::got_int;
use crate::guard::Suppress;
use crate::indent_c::in_cinkeys;
use crate::mbyte::{c_char_info, c_cluster_len, c_next_cluster};
use crate::memline::Lines;
use crate::memline::ml_replace_owned;
use crate::message::{e_interr, e_modifiable, e_resulting_text_too_long};
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
use crate::state::mode::{Insstart, State, ai_col, can_si, can_si_back, did_si, old_indent};
use crate::state::{MODE_INSERT, REPLACE_FLAG, VREPLACE_FLAG};
use crate::types::CmdModFlags;
use crate::undo::{u_clearline, u_save, u_savecommon};
use crate::winlayer::{Buf, Win};

/// Whether the cursor is before (or, with `extra` zero, on) the first
/// non-blank of the line.
pub fn inindent(extra: c_int) -> bool {
    let cursor = Win::current().w_cursor;
    let col = skip::white(Lines::current().line(cursor.lnum)) as c_int;
    col >= cursor.col as c_int + extra
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
        emsg(gettext(e_modifiable));
        return;
    }
    let mut first_changed: LineNr = 0;
    let mut last_changed: LineNr = 0;
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
        i = (line_count - 1) as c_int;
        while i >= 0 && !got_int.get() {
            // A slow thing to do, so say so — otherwise it looks hung.
            if i > 1
                && (i % 50 == 0 || i as LineNr == line_count - 1)
                && line_count as OptInt > p_report()
            {
                // Restore the cursor first, so the `msg_show` callback
                // does not redraw `curwin`.
                let save_lnum = win.w_cursor.lnum;
                win.w_cursor.lnum = start_lnum;
                indent_progress(
                    gettext(c"%ld lines to indent... "),
                    i as int64_t,
                    c"running",
                );
                win.w_cursor.lnum = save_lnum;
            }
            // Vi-compatible: with Lisp indenting the first line is not
            // indented, unless it is the only line.
            let lisp_first =
                i as LineNr == line_count - 1 && line_count != 1 && how == Indenter::Lisp;
            if !lisp_first {
                // A blank line gets no indent rather than the engine's.
                let blank = {
                    let mut lines = Lines::current();
                    let line = lines.line(win.w_cursor.lnum);
                    skip::white(line) == line.len()
                };
                let amount = if blank { 0 } else { how.amount() };
                if amount >= 0 && set_indent(amount, 0) {
                    if first_changed == 0 {
                        first_changed = win.w_cursor.lnum;
                    }
                    last_changed = win.w_cursor.lnum;
                }
            }
            win.w_cursor.lnum += 1;
            win.w_cursor.col = 0; // keep it valid
            i -= 1;
        }
    }
    // Put the cursor on the first non-blank of the indented line.
    win.w_cursor.lnum = start_lnum;
    beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    // Mark the changed lines for redraw. Under Visual highlighting that
    // has to reach the last line even when nothing changed, so that the
    // highlight goes away.
    if last_changed != 0 {
        let end = if op.is_visual {
            start_lnum + line_count
        } else {
            last_changed + 1
        };
        changed_lines(buf, first_changed, 0, end, 0, true);
    } else if op.is_visual {
        redraw_curbuf_later(UPD_INVERTED);
    }
    if line_count as OptInt > p_report() {
        let done = (line_count - (i as LineNr + 1)) as c_int;
        indent_progress(
            ngettext(
                c"%ld line indented ",
                c"%ld lines indented ",
                done as ::core::ffi::c_ulong,
            ),
            done as int64_t,
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
            && in_cinkeys('#' as c_int, ' ' as c_int, true)
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
    // SAFETY: the caller's position, and the cursor is put back before the
    // indent is applied.
    let mut win = Win::current();
    let old_pos = win.w_cursor;
    let mut i = pos.col as c_int;
    // The line is read here and nowhere else: `findmatch` below walks the
    // buffer itself, so the borrow has to be over before it is called.
    let mut lines = Lines::current();
    let line = lines.line(pos.lnum);
    if i > 0 {
        // Skip the blanks before the '{'.
        while {
            i -= 1;
            i > 0 && ascii_iswhite(c_int::from(byte_at(line, i as usize)))
        } {}
    }
    let after_paren = byte_at(line, i as usize) == b')';
    win.w_cursor.lnum = pos.lnum;
    win.w_cursor.col = i as ColNr;
    if after_paren && let Some(open) = findmatch(None, '(' as c_int) {
        win.w_cursor = open;
    }
    let indent = get_indent();
    win.w_cursor = old_pos;
    if State.get() & VREPLACE_FLAG != 0 {
        change_indent(INDENT_SET as c_int, indent, 0, true);
    } else {
        set_indent(indent, SIN_CHANGED as c_int);
    }
}

/// Whether a `{` typed after an `O` should reduce this line's indent: it may
/// not go below the indent of the previous real line.
fn si_should_shift_back() -> bool {
    // SAFETY: the caller's contract; the walk stops at line 1 and the cursor
    // is put back before answering.
    let mut win = Win::current();
    let old_pos = win.w_cursor;
    let here = get_indent();
    let mut lines = Lines::current();
    while win.w_cursor.lnum > 1 {
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
    // SAFETY: the caller's contract; every helper below reads and restores
    // the cursor itself.
    let win = Win::current();
    if (did_si.get() || can_si_back.get()) && c == '{' as c_int
        || can_si.get() && c == '}' as c_int && inindent(0)
    {
        let matching = if c == '}' as c_int {
            findmatch(None, '{' as c_int)
        } else {
            None
        };
        if let Some(matching) = matching {
            si_indent_like_open_brace(matching);
        } else if win.w_cursor.col > 0 {
            let shift = !(c == '{' as c_int
                && can_si_back.get()
                && win.w_cursor.lnum > 1
                && !si_should_shift_back());
            if shift {
                shift_line(true, false, 1, true);
            }
        }
    }
    // The indent of a '#' is always zero.
    if win.w_cursor.col > 0 && can_si.get() && c == '#' as c_int && inindent(0) {
        // Remember the current indent for the next line.
        old_indent.set(get_indent());
        set_indent(0, SIN_CHANGED as c_int);
    }
    // Adjust `ai_col`: the character at this position can be deleted.
    ai_col.set(ai_col.get().min(win.w_cursor.col));
}

/// Applies the indent [`change_indent`] was asked for, leaving the cursor on
/// the first non-blank.
fn apply_indent(type_0: c_int, amount: c_int, round: c_int, call_changed_bytes: bool) {
    if type_0 == INDENT_SET as c_int {
        set_indent(
            amount,
            if call_changed_bytes {
                SIN_CHANGED as c_int
            } else {
                0
            },
        );
        return;
    }
    let save_state = State.get();
    // Avoid being called recursively.
    if State.get() & VREPLACE_FLAG != 0 {
        State.set(MODE_INSERT);
    }
    shift_line(
        type_0 == INDENT_DEC as c_int,
        round != 0,
        1,
        call_changed_bytes,
    );
    State.set(save_state);
}

/// Puts the cursor `end_vcol` screen columns into the new indent, padding
/// the line with spaces when no character starts exactly there. Answers the
/// byte column the cursor should take.
fn place_cursor_in_indent(end_vcol: c_int) -> c_int {
    let mut win = Win::current();
    win.w_virtcol = end_vcol as ColNr;
    let mut new_cursor_col = 0;
    let mut vcol = 0;
    {
        let mut lines = Lines::current();
        let line = lines.line(win.w_cursor.lnum);
        if !line.is_empty() {
            let (mut csarg, cstype) = CharsizeArg::new(win, 0, line);
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
            new_cursor_col = at as c_int;
        }
    }
    if vcol == win.w_virtcol as c_int {
        return new_cursor_col;
    }
    // No character starts at that column, so make one: insert the spaces
    // the cursor needs to sit where it was asked to.
    win.w_cursor.col = new_cursor_col as ColNr;
    let ptrlen = (win.w_virtcol as c_int - vcol) as size_t;
    ins_str(&vec![b' '; ptrlen]);
    new_cursor_col + ptrlen as c_int
}

/// Moves `Insstart` and `ai_col` back by what the indent lost, so that the
/// insert still starts where the user began typing.
fn adjust_insert_start(insstart_less: c_int) {
    let mut insstart = Insstart.get();
    let lnum = Win::current().w_cursor.lnum;
    if lnum == insstart.lnum && insstart.col != 0 {
        insstart.col = if (insstart.col as c_int) <= insstart_less {
            0
        } else {
            insstart.col - insstart_less as ColNr
        };
        Insstart.set(insstart);
    }
    ai_col.set(if (ai_col.get() as c_int) <= insstart_less {
        0
    } else {
        ai_col.get() - insstart_less as ColNr
    });
}

/// Fixes the Replace-mode stack after the indent moved the cursor: pop what
/// the line lost, push NULs for what it gained.
fn fix_replace_stack(mut start_col: c_int) {
    let win = Win::current();
    while start_col > win.w_cursor.col as c_int {
        replace_join(0); // remove a NUL from the replace stack
        start_col -= 1;
    }
    while start_col < win.w_cursor.col as c_int {
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
        line[..(new_col as usize).min(line.len())].to_vec()
    };
    let _ = ml_replace_owned(win.w_cursor.lnum, orig_line);
    win.w_cursor.col = orig_col;
    let splice = Suppress::splice();
    backspace_until_column(0);
    ins_bytes(&new_line);
    drop(splice);
    let delta = orig_col as c_int - new_col as c_int;
    extmark_splice_cols(
        Buf::current(),
        win.w_cursor.lnum as c_int - 1,
        new_col,
        if delta < 0 { -delta as ColNr } else { 0 },
        if delta > 0 { delta as ColNr } else { 0 },
        kExtmarkUndo,
    );
}

/// Inserts an indent (`<Tab>`/`<C-t>`), deletes one (`<C-d>`) or sets one,
/// keeping the cursor on the same character. `round` rounds to 'shiftwidth'
/// and applies only to the two shifts; `call_changed_bytes` asks for
/// `changed_bytes()`.
pub fn change_indent(type_0: c_int, amount: c_int, round: c_int, call_changed_bytes: bool) {
    let mut win = Win::current();
    // Virtual Replace needs to know what the line looked like before.
    let orig = (State.get() & VREPLACE_FLAG != 0).then(|| {
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
    let mut vcol = getvcol_nolist(&mut cursor) as c_int;
    win.w_cursor = cursor;
    // Replace mode fixes its stack below, which is only possible when the
    // cursor is in the indent; this is how many characters precede it.
    let mut start_col = win.w_cursor.col as c_int;
    // Offset of the cursor from the first non-blank.
    let mut new_cursor_col = win.w_cursor.col as c_int;
    beginline(BeginlineOpts::WHITE);
    new_cursor_col -= win.w_cursor.col as c_int;
    let mut insstart_less = win.w_cursor.col as c_int;
    if new_cursor_col < 0 {
        // The cursor is inside the indent: how many screen columns it
        // sits left of the first non-blank.
        vcol = get_indent() - vcol;
    }
    if new_cursor_col > 0 {
        start_col = -1; // the replace stack cannot be fixed
    }
    apply_indent(type_0, amount, round, call_changed_bytes);
    insstart_less -= win.w_cursor.col as c_int;

    // Try to keep the cursor on the same character: at or after the first
    // non-blank that is a byte offset from it, outside Insert mode it just
    // stays on the first non-blank, and inside the indent it is a screen
    // column, which may need spaces inserted to reach.
    if new_cursor_col >= 0 {
        if new_cursor_col == 0 {
            // The cursor is touching the indent, so `Insstart.col` resets.
            insstart_less = MAXCOL as c_int;
        }
        new_cursor_col += win.w_cursor.col as c_int;
    } else if State.get() & MODE_INSERT == 0 {
        new_cursor_col = win.w_cursor.col as c_int;
    } else {
        new_cursor_col = place_cursor_in_indent((get_indent() - vcol).max(0));
        insstart_less = MAXCOL as c_int;
    }

    win.w_onebuf_opt.wo_list = save_p_list;
    win.w_cursor.col = new_cursor_col.max(0) as ColNr;
    win.w_set_curswant = true;
    changed_cline_bef_curs(win);
    if State.get() & MODE_INSERT != 0 {
        adjust_insert_start(insstart_less);
    }
    if State.get() & REPLACE_FLAG != 0 && State.get() & VREPLACE_FLAG == 0 && start_col >= 0 {
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
    let pad = |col: c_int| buf.tab_padding(col as ColNr);
    let expandtab = buf.b_p_et != 0;
    let src_indent = src;

    let mut indent: Vec<u8> = Vec::new();
    let mut todo = size;
    let mut ind_done = 0;
    let mut ind_col = 0;

    // Copy the usable part of the source's own indent.
    for &b in src_indent {
        if todo <= 0 || !(b == b' ' || b == TAB as u8) {
            break;
        }
        if b == TAB as u8 {
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
        indent.push(TAB as u8);
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
            indent.push(TAB as u8);
        }
    }
    // Spaces for what is left.
    indent.resize(indent.len() + todo.max(0) as usize, b' ');
    let ind_len = indent.len() as c_int;

    // The indent, then the whole of the cursor line.
    let lnum = Win::current().w_cursor.lnum;
    indent.extend_from_slice(Lines::current().line(lnum));
    let _ = ml_replace_owned(lnum, XString::from_vec(indent));
    Win::current().w_cursor.col = ind_len as ColNr;
    true
}

/// Reports "resulting text is too long", and breaks out of any loop when
/// there is no `:try` to catch it.
fn emsg_text_too_long() {
    emsg(gettext(e_resulting_text_too_long));
    if trylevel.get() == 0 {
        got_int.set(true);
    }
}

/// The tabstops `:retab` was asked to retabulate to.
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

/// Parses `:retab`'s argument. `None` is a malformed tabstop list, which
/// `tabstop_set` has already reported.
fn parse_retab_arg(arg: &[u8]) -> Option<RetabTabs> {
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
    Some(match vts {
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
        let width = (scan.vcol - self.start_vcol) as c_int;
        self.num_spaces = width;
        let mut num_tabs = 0;
        if buf.b_p_et == 0 {
            let (start, end) = (self.start_vcol as ColNr, scan.vcol as ColNr);
            (num_tabs, self.num_spaces) = match tabstops(tabs.vts.as_deref()) {
                Some(stops) => stops.from_to(start, end),
                None => {
                    let ts = buf.b_p_ts as c_int;
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
        let old_len = scan.text.len() as c_int;
        let new_len = old_len - scan.col + self.start_col + len + 1;
        if new_len <= 0 || new_len >= MAXCOL as c_int {
            emsg_text_too_long();
            return Retabulated::TooLong;
        }
        let mut new_line = Vec::with_capacity(new_len as usize);
        new_line.extend_from_slice(&scan.text[..self.start_col as usize]);
        new_line.extend((0..len).map(|i| if i < num_tabs { b'\t' } else { b' ' }));
        new_line.extend_from_slice(&scan.text[scan.col as usize..]);
        if ml_replace_owned(scan.lnum, XString::from_bytes(&new_line)).is_ok() {
            let lnum = scan.lnum as c_int - 1;
            let (old, new) = (old_len, new_len - 1);
            extmark_splice_cols(buf, lnum, 0, old, new, kExtmarkUndo);
        }
        if self.first_line == 0 {
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
        let byte = |s: &LineScan| byte_at(&s.text, s.col as usize);
        let win = Win::current();
        let width = |s: &LineScan| {
            let at = s.col as usize;
            let text = &s.text[at..(at + CHAR_CELLS_SPAN).min(s.text.len())];
            win_chartabsize(win, text, s.vcol as ColNr)
        };
        let charlen = |s: &LineScan| c_cluster_len(&s.text[s.col as usize..]) as c_int;
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
            scan.vcol += width(&scan) as int64_t;
            if scan.vcol >= MAXCOL as int64_t {
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
            kOptVartabstop,
            OptVal::from_cstr(ts_str.as_cstr()),
            OptionSetFlags::LOCAL,
            0,
        );
        buf.b_p_vts_array = vts;
    } else {
        // A single stop with 'vartabstop' unused is 'tabstop'.
        buf.b_p_ts = first_stop(vts.as_deref()) as OptInt;
    }
}

/// `:retab`.
pub fn ex_retab(excmd: &mut ExArg) {
    let mut win = Win::current();
    let buf = Buf::current();
    let save_list = win.w_onebuf_opt.wo_list;
    win.w_onebuf_opt.wo_list = 0; // 'list' mode is not wanted here
    let Some(tabs) = parse_retab_arg(excmd.line.arg()) else {
        // Upstream returns here without restoring 'list', which it has
        // already cleared. Kept: a `:retab` with a malformed tabstop
        // list leaves 'list' off, and that is observable.
        return;
    };
    let mut retab = Retab {
        got_tab: false,
        num_spaces: 0,
        start_col: 0,
        start_vcol: 0,
        first_line: 0,
        last_line: 0,
    };
    let mut lnum = excmd.line1;
    while !got_int.get() && lnum <= excmd.line2 {
        if !retab.line(lnum, &tabs, excmd.forceit) {
            break;
        }
        line_breakcheck();
        lnum += 1;
    }
    if got_int.get() {
        emsg(gettext(e_interr));
    }
    // A single value given is equal to either 'tabstop' or 'vartabstop',
    // and then nothing on screen changed.
    let (own, given) = (buf.b_p_vts_array.as_deref(), tabs.vts.as_deref());
    let unchanged =
        stop_count(own) == 0 && stop_count(given) == 1 && buf.b_p_ts == first_stop(given) as OptInt
            || stop_count(own) > 0 && tabstop::eq(own, given);
    if !unchanged {
        redraw_curbuf_later(UPD_NOT_VALID);
    }
    if retab.first_line != 0 {
        changed_lines(buf, retab.first_line, 0, retab.last_line + 1, 0, true);
    }
    win.w_onebuf_opt.wo_list = save_list; // restore 'list'
    if let Some(ts_str) = &tabs.ts_str {
        set_retab_tabstop(tabs.vts, ts_str);
    }
    coladvance(win, win.w_curswant);
    u_clearline(buf);
}
