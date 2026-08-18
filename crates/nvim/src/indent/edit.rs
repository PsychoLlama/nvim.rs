//! Reindenting text: the `=` operator, `:retab`, the `<C-t>`/`<C-d>`
//! shifts, Insert-mode smart indent, and copying an existing line's indent.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::CStr;
use core::ptr;

use super::*;
use crate::api::private::helpers::cstr_as_string;
use crate::ascii::{ascii_isdigit, ascii_iswhite, ascii_iswhite_or_nul};
use crate::change::{changed_lines, ins_bytes, ins_str};
use crate::charset::skipwhite;
use crate::cursor::{coladvance, get_cursor_line_len, get_cursor_line_ptr};
use crate::drawscreen::{UPD_INVERTED, UPD_NOT_VALID, redraw_curbuf_later};
use crate::edit::{backspace_until_column, beginline, replace_join, replace_push_nul};
use crate::extmark::extmark_splice_cols;
use crate::indent_c::in_cinkeys;
use crate::main::{
    IObuff, Insstart, State, ai_col, can_si, can_si_back, cmdmod, curbuf_splice_pending, did_si,
    e_interr, e_modifiable, e_resulting_text_too_long, got_int, old_indent, p_paste, p_report,
    trylevel,
};
use crate::mbyte::{utf_ptr2StrCharInfo, utfc_next, utfc_ptr2len};
use crate::memline::{ml_get, ml_get_len, ml_replace};
use crate::memory::{xfree, xmalloc, xmallocz, xmemdupz};
use crate::message::{emsg, msg_progress};
use crate::r#move::changed_cline_bef_curs;
use crate::ops::shift_line;
use crate::option::set_option_direct;
use crate::os::cshim::{gettext, memmove, ngettext, snprintf, strncmp};
use crate::os::input::line_breakcheck;
use crate::plines::{getvcol_nolist, init_charsize_arg, win_charsize, win_chartabsize};
use crate::pos::MAXCOL;
use crate::search::findmatch;
use crate::state::{MODE_INSERT, REPLACE_FLAG, VREPLACE_FLAG};
use crate::strings::xstrnsave;
use crate::types::CMOD_LOCKMARKS;
use crate::undo::{u_clearline, u_save, u_savecommon};
use ::libc::memset;

/// Whether the cursor is before (or, with `extra` zero, on) the first
/// non-blank of the line.
///
/// # Safety
/// There must be a current window and line.
pub unsafe fn inindent(extra: c_int) -> bool {
    // SAFETY: the caller's contract; the walk is stopped by the NUL, which
    // is not white space.
    unsafe {
        let mut ptr = get_cursor_line_ptr();
        let mut col = 0;
        while ascii_iswhite(*ptr as c_int) {
            ptr = ptr.add(1);
            col += 1;
        }
        col >= (*curwin.get()).w_cursor.col as c_int + extra
    }
}

/// Writes `fmt` with `n` into `IObuff` and shows it as `:indent` progress.
///
/// # Safety
/// `fmt` must be a NUL-terminated format string taking one `int64_t`.
unsafe fn indent_progress(fmt: *const c_char, n: int64_t, status: &CStr) {
    // SAFETY: `IObuff` is `IOSIZE` bytes, which is what bounds the format,
    // and the two labels are NUL-terminated constants.
    unsafe {
        snprintf(IObuff.ptr().cast(), IOSIZE as size_t, fmt, n);
        msg_progress(
            IObuff.ptr().cast(),
            c"indent".as_ptr().cast_mut(),
            status.as_ptr().cast_mut(),
            0,
            true,
            false,
        );
    }
}

/// Reindents `oap`'s lines with `how`, one of the three indent engines.
///
/// # Safety
/// `oap` must be a live operator argument.
pub unsafe fn op_reindent(oap: *mut oparg_T, how: Indenter) {
    // SAFETY: the caller's operator argument, and the buffer it names is
    // the current one.
    unsafe {
        let win = curwin.get();
        let buf = curbuf.get();
        let start_lnum = (*win).w_cursor.lnum;
        let line_count = (*oap).line_count;
        if (*buf).b_p_ma == 0 {
            emsg(gettext((&raw const e_modifiable).cast()));
            return;
        }
        let mut first_changed: linenr_T = 0;
        let mut last_changed: linenr_T = 0;
        // Undo once for all the lines: much faster than per line, especially
        // when undoing.
        let mut i = 0;
        if u_savecommon(
            buf,
            start_lnum - 1,
            start_lnum + line_count,
            start_lnum + line_count,
            false,
        ) == OK
        {
            i = (line_count - 1) as c_int;
            while i >= 0 && !got_int.get() {
                // A slow thing to do, so say so — otherwise it looks hung.
                if i > 1
                    && (i % 50 == 0 || i as linenr_T == line_count - 1)
                    && line_count as OptInt > p_report.get()
                {
                    // Restore the cursor first, so the `msg_show` callback
                    // does not redraw `curwin`.
                    let save_lnum = (*win).w_cursor.lnum;
                    (*win).w_cursor.lnum = start_lnum;
                    indent_progress(
                        gettext(c"%ld lines to indent... ".as_ptr()),
                        i as int64_t,
                        c"running",
                    );
                    (*win).w_cursor.lnum = save_lnum;
                }
                // Vi-compatible: with Lisp indenting the first line is not
                // indented, unless it is the only line.
                let lisp_first = i as linenr_T == line_count - 1
                    && line_count != 1
                    && how.is_some_and(|f| {
                        ptr::fn_addr_eq(f, get_lisp_indent as unsafe fn() -> c_int)
                    });
                if !lisp_first {
                    // A blank line gets no indent rather than the engine's.
                    let blank = *skipwhite(get_cursor_line_ptr()) == 0;
                    let amount = if blank {
                        0
                    } else {
                        how.expect("non-null function pointer")()
                    };
                    if amount >= 0 && set_indent(amount, 0) {
                        if first_changed == 0 {
                            first_changed = (*win).w_cursor.lnum;
                        }
                        last_changed = (*win).w_cursor.lnum;
                    }
                }
                (*win).w_cursor.lnum += 1;
                (*win).w_cursor.col = 0; // keep it valid
                i -= 1;
            }
        }
        // Put the cursor on the first non-blank of the indented line.
        (*win).w_cursor.lnum = start_lnum;
        beginline(BL_SOL as c_int | BL_FIX as c_int);
        // Mark the changed lines for redraw. Under Visual highlighting that
        // has to reach the last line even when nothing changed, so that the
        // highlight goes away.
        if last_changed != 0 {
            let end = if (*oap).is_VIsual {
                start_lnum + line_count
            } else {
                last_changed + 1
            };
            changed_lines(buf, first_changed, 0, end, 0, true);
        } else if (*oap).is_VIsual {
            redraw_curbuf_later(UPD_INVERTED);
        }
        if line_count as OptInt > p_report.get() {
            let done = (line_count - (i as linenr_T + 1)) as c_int;
            indent_progress(
                ngettext(
                    c"%ld line indented ".as_ptr(),
                    c"%ld lines indented ".as_ptr(),
                    done as ::core::ffi::c_ulong,
                ),
                done as int64_t,
                c"success",
            );
        }
        if cmdmod.with(|m| m.cmod_flags) & CMOD_LOCKMARKS as c_int == 0 {
            // Set the '[ and '] marks.
            (*buf).b_op_start = (*oap).start;
            (*buf).b_op_end = (*oap).end;
        }
    }
}

/// Whether lines starting with `#` should be left aligned.
///
/// # Safety
/// There must be a current buffer.
pub unsafe fn preprocs_left() -> bool {
    // SAFETY: the caller's contract.
    unsafe {
        let buf = curbuf.get();
        (*buf).b_p_si != 0 && (*buf).b_p_cin == 0
            || (*buf).b_p_cin != 0
                && in_cinkeys('#' as c_int, ' ' as c_int, true)
                && (*buf).b_ind_hash_comment == 0
    }
}

/// Whether the conditions are right for smart indenting.
///
/// # Safety
/// There must be a current buffer.
pub unsafe fn may_do_si() -> bool {
    // SAFETY: the caller's contract.
    unsafe {
        let buf = curbuf.get();
        (*buf).b_p_si != 0 && (*buf).b_p_cin == 0 && *(*buf).b_p_inde == 0 && p_paste.get() == 0
    }
}

/// Sets the cursor line's indent to that of the line holding the `{` that
/// `pos` matched — or, when that `{` has a `)` just before it, to the line
/// holding the matching `(`, which is what makes an `if (..\n..) {`
/// spanning several lines come out right (Webb).
///
/// # Safety
/// `pos` must be a position in the current buffer.
unsafe fn si_indent_like_open_brace(pos: *mut pos_T) {
    // SAFETY: the caller's position, and the cursor is put back before the
    // indent is applied.
    unsafe {
        let win = curwin.get();
        let old_pos = (*win).w_cursor;
        let ptr = ml_get((*pos).lnum);
        let mut i = (*pos).col as c_int;
        if i > 0 {
            // Skip the blanks before the '{'.
            while {
                i -= 1;
                i > 0 && ascii_iswhite(*ptr.offset(i as isize) as c_int)
            } {}
        }
        (*win).w_cursor.lnum = (*pos).lnum;
        (*win).w_cursor.col = i as colnr_T;
        if *ptr.offset(i as isize) == b')' as c_char {
            let open = findmatch(ptr::null_mut(), '(' as c_int);
            if !open.is_null() {
                (*win).w_cursor = *open;
            }
        }
        let indent = get_indent();
        (*win).w_cursor = old_pos;
        if State.get() & VREPLACE_FLAG != 0 {
            change_indent(INDENT_SET as c_int, indent, 0, true);
        } else {
            set_indent(indent, SIN_CHANGED as c_int);
        }
    }
}

/// Whether a `{` typed after an `O` should reduce this line's indent: it may
/// not go below the indent of the previous real line.
///
/// # Safety
/// There must be a current window with the cursor past line 1.
unsafe fn si_should_shift_back() -> bool {
    // SAFETY: the caller's contract; the walk stops at line 1 and the cursor
    // is put back before answering.
    unsafe {
        let win = curwin.get();
        let old_pos = (*win).w_cursor;
        let here = get_indent();
        while (*win).w_cursor.lnum > 1 {
            (*win).w_cursor.lnum -= 1;
            let ptr = skipwhite(ml_get((*win).w_cursor.lnum));
            // Ignore empty lines and lines starting with '#'.
            if *ptr != b'#' as c_char && *ptr != 0 {
                break;
            }
        }
        let above = get_indent();
        (*win).w_cursor = old_pos;
        above < here
    }
}

/// Very smart auto-indenting for a "normal" character typed in Insert mode:
/// `{`, `}` and `#`.
///
/// # Safety
/// There must be a current window and buffer.
pub unsafe fn ins_try_si(c: c_int) {
    // SAFETY: the caller's contract; every helper below reads and restores
    // the cursor itself.
    unsafe {
        let win = curwin.get();
        if (did_si.get() || can_si_back.get()) && c == '{' as c_int
            || can_si.get() && c == '}' as c_int && inindent(0)
        {
            let matching = if c == '}' as c_int {
                findmatch(ptr::null_mut(), '{' as c_int)
            } else {
                ptr::null_mut()
            };
            if !matching.is_null() {
                si_indent_like_open_brace(matching);
            } else if (*win).w_cursor.col > 0 {
                let shift = !(c == '{' as c_int
                    && can_si_back.get()
                    && (*win).w_cursor.lnum > 1
                    && !si_should_shift_back());
                if shift {
                    shift_line(true, false, 1, true);
                }
            }
        }
        // The indent of a '#' is always zero.
        if (*win).w_cursor.col > 0 && can_si.get() && c == '#' as c_int && inindent(0) {
            // Remember the current indent for the next line.
            old_indent.set(get_indent());
            set_indent(0, SIN_CHANGED as c_int);
        }
        // Adjust `ai_col`: the character at this position can be deleted.
        ai_col.set(ai_col.get().min((*win).w_cursor.col));
    }
}

/// Applies the indent [`change_indent`] was asked for, leaving the cursor on
/// the first non-blank.
///
/// # Safety
/// There must be a modifiable current line.
unsafe fn apply_indent(type_0: c_int, amount: c_int, round: c_int, call_changed_bytes: bool) {
    // SAFETY: the caller's contract.
    unsafe {
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
}

/// Puts the cursor `end_vcol` screen columns into the new indent, padding
/// the line with spaces when no character starts exactly there. Answers the
/// byte column the cursor should take.
///
/// # Safety
/// There must be a current window and line.
unsafe fn place_cursor_in_indent(end_vcol: c_int) -> c_int {
    // SAFETY: the caller's contract; the walk is over the cursor line and
    // stopped by its NUL.
    unsafe {
        let win = curwin.get();
        (*win).w_virtcol = end_vcol as colnr_T;
        let line = get_cursor_line_ptr();
        let mut new_cursor_col = 0;
        let mut vcol = 0;
        if *line != 0 {
            let mut csarg = CharsizeArg::default();
            let cstype = init_charsize_arg(&mut csarg, win, 0, line);
            let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
            loop {
                let next_vcol =
                    vcol + win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &mut csarg).width;
                if next_vcol > end_vcol {
                    break;
                }
                vcol = next_vcol;
                ci = utfc_next(ci);
                if *ci.ptr == 0 {
                    break;
                }
            }
            new_cursor_col = ci.ptr.offset_from(line) as c_int;
        }
        if vcol == (*win).w_virtcol as c_int {
            return new_cursor_col;
        }
        // No character starts at that column, so make one: insert the spaces
        // the cursor needs to sit where it was asked to.
        (*win).w_cursor.col = new_cursor_col as colnr_T;
        let ptrlen = ((*win).w_virtcol as c_int - vcol) as size_t;
        let spaces: *mut c_char = xmallocz(ptrlen).cast();
        memset(spaces.cast(), ' ' as c_int, ptrlen);
        ins_str(spaces, ptrlen);
        xfree(spaces.cast());
        new_cursor_col + ptrlen as c_int
    }
}

/// Moves `Insstart` and `ai_col` back by what the indent lost, so that the
/// insert still starts where the user began typing.
///
/// # Safety
/// There must be a current window.
unsafe fn adjust_insert_start(insstart_less: c_int) {
    Insstart.with_mut(|insstart| {
        // SAFETY: the caller's contract.
        let lnum = unsafe { (*curwin.get()).w_cursor.lnum };
        if lnum == insstart.lnum && insstart.col != 0 {
            insstart.col = if (insstart.col as c_int) <= insstart_less {
                0
            } else {
                insstart.col - insstart_less as colnr_T
            };
        }
    });
    ai_col.set(if (ai_col.get() as c_int) <= insstart_less {
        0
    } else {
        ai_col.get() - insstart_less as colnr_T
    });
}

/// Fixes the Replace-mode stack after the indent moved the cursor: pop what
/// the line lost, push NULs for what it gained.
///
/// # Safety
/// There must be an open replace stack.
unsafe fn fix_replace_stack(mut start_col: c_int) {
    // SAFETY: the caller's contract.
    unsafe {
        let win = curwin.get();
        while start_col > (*win).w_cursor.col as c_int {
            replace_join(0); // remove a NUL from the replace stack
            start_col -= 1;
        }
        while start_col < (*win).w_cursor.col as c_int {
            replace_push_nul();
            start_col += 1;
        }
    }
}

/// Fixes the Virtual Replace stack: put the original line back, then
/// backspace over it and type the new one — always possible, because the
/// whole line is replayed. Takes ownership of `orig_line`.
///
/// # Safety
/// `orig_line`/`orig_col` must be what [`change_indent`] saved.
unsafe fn vreplace_restore(orig_line: *mut c_char, orig_col: colnr_T) {
    // SAFETY: the caller's saved line, which `ml_replace` takes over.
    unsafe {
        let win = curwin.get();
        // The new line, but only up to the cursor.
        let new_line = xstrnsave(get_cursor_line_ptr(), get_cursor_line_len() as size_t);
        let new_col = (*win).w_cursor.col;
        *new_line.offset(new_col as isize) = 0;
        ml_replace((*win).w_cursor.lnum, orig_line, false);
        (*win).w_cursor.col = orig_col;
        curbuf_splice_pending.set(curbuf_splice_pending.get() + 1);
        backspace_until_column(0);
        ins_bytes(new_line);
        xfree(new_line.cast());
        curbuf_splice_pending.set(curbuf_splice_pending.get() - 1);
        let delta = orig_col as c_int - new_col as c_int;
        extmark_splice_cols(
            curbuf.get(),
            (*win).w_cursor.lnum as c_int - 1,
            new_col,
            if delta < 0 { -delta as colnr_T } else { 0 },
            if delta > 0 { delta as colnr_T } else { 0 },
            kExtmarkUndo,
        );
    }
}

/// Inserts an indent (`<Tab>`/`<C-t>`), deletes one (`<C-d>`) or sets one,
/// keeping the cursor on the same character. `round` rounds to 'shiftwidth'
/// and applies only to the two shifts; `call_changed_bytes` asks for
/// `changed_bytes()`.
///
/// # Safety
/// There must be a current window and a modifiable line.
pub unsafe fn change_indent(type_0: c_int, amount: c_int, round: c_int, call_changed_bytes: bool) {
    // SAFETY: the caller's contract; every deref is the current window.
    unsafe {
        let win = curwin.get();
        // Virtual Replace needs to know what the line looked like before.
        let orig = (State.get() & VREPLACE_FLAG != 0).then(|| {
            (
                xstrnsave(get_cursor_line_ptr(), get_cursor_line_len() as size_t),
                (*win).w_cursor.col,
            )
        });
        // The tricks below do not want 'list' mode.
        let save_p_list = (*win).w_onebuf_opt.wo_list;
        (*win).w_onebuf_opt.wo_list = 0;
        let mut vcol = getvcol_nolist(&raw mut (*win).w_cursor) as c_int;
        // Replace mode fixes its stack below, which is only possible when the
        // cursor is in the indent; this is how many characters precede it.
        let mut start_col = (*win).w_cursor.col as c_int;
        // Offset of the cursor from the first non-blank.
        let mut new_cursor_col = (*win).w_cursor.col as c_int;
        beginline(BL_WHITE as c_int);
        new_cursor_col -= (*win).w_cursor.col as c_int;
        let mut insstart_less = (*win).w_cursor.col as c_int;
        if new_cursor_col < 0 {
            // The cursor is inside the indent: how many screen columns it
            // sits left of the first non-blank.
            vcol = get_indent() - vcol;
        }
        if new_cursor_col > 0 {
            start_col = -1; // the replace stack cannot be fixed
        }
        apply_indent(type_0, amount, round, call_changed_bytes);
        insstart_less -= (*win).w_cursor.col as c_int;

        // Try to keep the cursor on the same character: at or after the first
        // non-blank that is a byte offset from it, outside Insert mode it just
        // stays on the first non-blank, and inside the indent it is a screen
        // column, which may need spaces inserted to reach.
        if new_cursor_col >= 0 {
            if new_cursor_col == 0 {
                // The cursor is touching the indent, so `Insstart.col` resets.
                insstart_less = MAXCOL as c_int;
            }
            new_cursor_col += (*win).w_cursor.col as c_int;
        } else if State.get() & MODE_INSERT == 0 {
            new_cursor_col = (*win).w_cursor.col as c_int;
        } else {
            new_cursor_col = place_cursor_in_indent((get_indent() - vcol).max(0));
            insstart_less = MAXCOL as c_int;
        }

        (*win).w_onebuf_opt.wo_list = save_p_list;
        (*win).w_cursor.col = new_cursor_col.max(0) as colnr_T;
        (*win).w_set_curswant = 1;
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
}

/// Copies `src`'s indent onto the cursor line, filling to `size` columns and
/// leaving the cursor on the first non-blank.
///
/// Upstream runs the body twice — once to size the allocation, once to fill
/// it. The indent is built into a `Vec` here instead, so the shape is stated
/// once.
///
/// # Safety
/// `src` must be NUL-terminated, and there must be a current window.
pub unsafe fn copy_indent(size: c_int, src: *mut c_char) -> bool {
    let buf = curbuf.get();
    // SAFETY: `b_p_ts`/`b_p_vts_array` are the buffer's own tabstop
    // settings. Written as a closure inside the `unsafe`, so that the walk
    // below — which is the whole of this function — stays checked code.
    let pad = |col: c_int| unsafe {
        tabstop_padding(col as colnr_T, (*buf).b_p_ts, (*buf).b_p_vts_array)
    };
    // SAFETY: the caller's NUL-terminated source line.
    let expandtab = unsafe { (*buf).b_p_et != 0 };
    // SAFETY: the caller's NUL-terminated source line.
    let src_indent = unsafe { CStr::from_ptr(src) }.to_bytes();

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

    // SAFETY: the allocation is sized from what was just built plus the
    // line it is prefixed to, and `ml_replace` takes it over.
    unsafe {
        // The rest of the line, including its NUL.
        let line_len = get_cursor_line_len() + 1;
        // Both operands are non-negative `int`s, so the only way the
        // narrowing to `size_t` the C guarded could fail is a negative sum.
        debug_assert!(ind_len + line_len >= 0, "STRICT_ADD overflow");
        let line: *mut c_char = xmalloc((ind_len + line_len) as size_t).cast();
        ptr::copy_nonoverlapping(indent.as_ptr(), line.cast::<u8>(), indent.len());
        memmove(
            line.add(indent.len()).cast(),
            get_cursor_line_ptr().cast(),
            line_len as size_t,
        );
        ml_replace((*curwin.get()).w_cursor.lnum, line, false);
        (*curwin.get()).w_cursor.col = ind_len as colnr_T;
    }
    true
}

/// Reports "resulting text is too long", and breaks out of any loop when
/// there is no `:try` to catch it.
///
/// # Safety
/// There must be a message layer, i.e. anywhere in the editor.
unsafe fn emsg_text_too_long() {
    // SAFETY: the message is a NUL-terminated constant.
    unsafe { emsg(gettext((&raw const e_resulting_text_too_long).cast())) };
    if trylevel.get() == 0 {
        got_int.set(true);
    }
}

/// The tabstops `:retab` was asked to retabulate to.
struct RetabTabs {
    /// The list to measure against.
    vts: *mut colnr_T,
    /// Owned argument text, when it named a *new* list that must also be set
    /// on the buffer afterwards; null when the buffer's own list is reused,
    /// in which case no option is touched.
    ts_str: *mut c_char,
    /// `:retab -indentonly`: stop at the end of each line's indent.
    indent_only: bool,
}

/// Parses `:retab`'s argument. `None` is a malformed tabstop list, which
/// `tabstop_set` has already reported.
///
/// # Safety
/// `arg` must be a NUL-terminated string.
unsafe fn parse_retab_arg(arg: *mut c_char) -> Option<RetabTabs> {
    // SAFETY: the caller's argument, walked to its NUL.
    unsafe {
        let mut ptr = arg;
        let indent_only = strncmp(ptr, c"-indentonly".as_ptr(), 11) == 0
            && ascii_iswhite_or_nul(*ptr.offset(11) as c_int);
        if indent_only {
            ptr = skipwhite(ptr.offset(11));
        }
        let ts_start = ptr;
        let mut vts: *mut colnr_T = ptr::null_mut();
        if !tabstop_set(ptr, &raw mut vts) {
            return None;
        }
        while ascii_isdigit(*ptr as c_int) || *ptr == b',' as c_char {
            ptr = ptr.add(1);
        }
        // Either both are freshly allocated, or `vts` is the buffer's own
        // array and `ts_str` is null.
        if vts.is_null() {
            Some(RetabTabs {
                vts: (*curbuf.get()).b_p_vts_array,
                ts_str: ptr::null_mut(),
                indent_only,
            })
        } else {
            Some(RetabTabs {
                vts,
                ts_str: xmemdupz(ts_start.cast(), ptr.offset_from(ts_start) as size_t).cast(),
                indent_only,
            })
        }
    }
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
    first_line: linenr_T,
    last_line: linenr_T,
}

/// One line's walk: where the scan has got to, and the line as it stands.
struct LineScan {
    lnum: linenr_T,
    ptr: *mut c_char,
    old_len: c_int,
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
    ///
    /// # Safety
    /// `scan` must address a line of the current buffer.
    unsafe fn retabulate(&mut self, scan: &mut LineScan, tabs: &RetabTabs) -> Retabulated {
        // SAFETY: the caller's line; the replacement is sized from its
        // length and handed to `ml_replace`, which takes it over.
        unsafe {
            let buf = curbuf.get();
            // The run's width on screen.
            let width = (scan.vcol - self.start_vcol) as c_int;
            self.num_spaces = width;
            let mut num_tabs = 0;
            if (*buf).b_p_et == 0 {
                let (mut t, mut s) = (0, 0);
                tabstop_fromto(
                    self.start_vcol as colnr_T,
                    scan.vcol as colnr_T,
                    (*buf).b_p_ts as c_int,
                    tabs.vts,
                    &raw mut t,
                    &raw mut s,
                );
                num_tabs = t;
                self.num_spaces = s;
            }
            if !((*buf).b_p_et != 0 || self.got_tab || self.num_spaces + num_tabs < width) {
                return Retabulated::Done;
            }
            if !scan.did_undo {
                scan.did_undo = true;
                if u_save(scan.lnum - 1, scan.lnum + 1) == FAIL {
                    return Retabulated::OutOfMemory;
                }
            }
            // How many characters the run will actually take.
            let len = self.num_spaces + num_tabs;
            let new_len = scan.old_len - scan.col + self.start_col + len + 1;
            if new_len <= 0 || new_len >= MAXCOL as c_int {
                emsg_text_too_long();
                return Retabulated::TooLong;
            }
            let new_line: *mut c_char = xmalloc(new_len as size_t).cast();
            if self.start_col > 0 {
                memmove(new_line.cast(), scan.ptr.cast(), self.start_col as size_t);
            }
            memmove(
                new_line.offset((self.start_col + len) as isize).cast(),
                scan.ptr.offset(scan.col as isize).cast(),
                (scan.old_len - scan.col + 1) as size_t,
            );
            let run = new_line.offset(self.start_col as isize);
            for i in 0..len {
                *run.offset(i as isize) = if i < num_tabs { b'\t' } else { b' ' } as c_char;
            }
            let mut line = new_line;
            if ml_replace(scan.lnum, new_line, false) == OK {
                // `new_line` may have been copied.
                line = (*buf).b_ml.ml_line_ptr;
                let lnum = scan.lnum as c_int - 1;
                extmark_splice_cols(buf, lnum, 0, scan.old_len, new_len - 1, kExtmarkUndo);
            }
            if self.first_line == 0 {
                self.first_line = scan.lnum;
            }
            self.last_line = scan.lnum;
            scan.ptr = line;
            scan.old_len = new_len - 1;
            scan.col = self.start_col + len;
            Retabulated::Done
        }
    }

    /// Retabulates one line. Answers false when the command has to stop.
    ///
    /// # Safety
    /// `lnum` must be a line of the current buffer.
    unsafe fn line(&mut self, lnum: linenr_T, tabs: &RetabTabs, forceit: bool) -> bool {
        // SAFETY: the caller's line. The three closures below are the only
        // places it is read, `col` never passes its NUL, and `retabulate`
        // moves the scan onto whatever line it leaves behind.
        let mut scan = unsafe {
            LineScan {
                lnum,
                ptr: ml_get(lnum),
                old_len: ml_get_len(lnum),
                col: 0,
                vcol: 0,
                did_undo: false,
            }
        };
        // Written inside `unsafe` blocks so that the walk stays checked code.
        let byte = |s: &LineScan| unsafe { *s.ptr.offset(s.col as isize) };
        let width = |s: &LineScan| unsafe {
            win_chartabsize(
                curwin.get(),
                s.ptr.offset(s.col as isize),
                s.vcol as colnr_T,
            )
        };
        let charlen = |s: &LineScan| unsafe { utfc_ptr2len(s.ptr.offset(s.col as isize)) };
        loop {
            let at = byte(&scan);
            if ascii_iswhite(at as c_int) {
                if !self.got_tab && self.num_spaces == 0 {
                    // The first character of a run of white space.
                    self.start_vcol = scan.vcol;
                    self.start_col = scan.col;
                }
                if at == b' ' as c_char {
                    self.num_spaces += 1;
                } else {
                    self.got_tab = true;
                }
            } else {
                if self.got_tab || (forceit && self.num_spaces > 1) {
                    // SAFETY: `scan` addresses a line of the current buffer.
                    match unsafe { self.retabulate(&mut scan, tabs) } {
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
                // SAFETY: reporting an error needs no more than a live editor.
                unsafe { emsg_text_too_long() };
                break;
            }
            scan.col += charlen(&scan);
        }
        true
    }
}

/// Applies the tabstop list `:retab` was given to the buffer, releasing
/// whichever of the two arrays is no longer wanted.
///
/// # Safety
/// `tabs.ts_str` must be non-null and owned.
unsafe fn set_retab_tabstop(tabs: &RetabTabs) {
    // SAFETY: the caller's contract; the buffer takes over `tabs.vts` on the
    // 'vartabstop' path and the old array is freed instead.
    unsafe {
        let buf = curbuf.get();
        let old_vts_ary = (*buf).b_p_vts_array;
        if tabstop_count(old_vts_ary) > 0 || tabstop_count(tabs.vts) > 1 {
            // 'vartabstop' is in use, or more than one stop was given.
            set_option_direct(
                kOptVartabstop,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string(tabs.ts_str),
                    },
                },
                OPT_LOCAL as c_int,
                0,
            );
            (*buf).b_p_vts_array = tabs.vts;
            xfree(old_vts_ary.cast());
        } else {
            // A single stop with 'vartabstop' unused is 'tabstop'.
            (*buf).b_p_ts = tabstop_first(tabs.vts) as OptInt;
            xfree(tabs.vts.cast());
        }
        xfree(tabs.ts_str.cast());
    }
}

/// `:retab`.
///
/// # Safety
/// `eap` must be a live Ex-command argument.
pub unsafe fn ex_retab(eap: *mut exarg_T) {
    // SAFETY: the caller's Ex-command argument; the line range it names is
    // the current buffer's.
    unsafe {
        let win = curwin.get();
        let buf = curbuf.get();
        let save_list = (*win).w_onebuf_opt.wo_list;
        (*win).w_onebuf_opt.wo_list = 0; // 'list' mode is not wanted here
        let Some(tabs) = parse_retab_arg((*eap).arg) else {
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
        let mut lnum = (*eap).line1;
        while !got_int.get() && lnum <= (*eap).line2 {
            if !retab.line(lnum, &tabs, (*eap).forceit != 0) {
                break;
            }
            line_breakcheck();
            lnum += 1;
        }
        if got_int.get() {
            emsg(gettext((&raw const e_interr).cast()));
        }
        // A single value given is equal to either 'tabstop' or 'vartabstop',
        // and then nothing on screen changed.
        let unchanged = tabstop_count((*buf).b_p_vts_array) == 0
            && tabstop_count(tabs.vts) == 1
            && (*buf).b_p_ts == tabstop_first(tabs.vts) as OptInt
            || tabstop_count((*buf).b_p_vts_array) > 0
                && tabstop_eq((*buf).b_p_vts_array, tabs.vts);
        if !unchanged {
            redraw_curbuf_later(UPD_NOT_VALID);
        }
        if retab.first_line != 0 {
            changed_lines(buf, retab.first_line, 0, retab.last_line + 1, 0, true);
        }
        (*win).w_onebuf_opt.wo_list = save_list; // restore 'list'
        if !tabs.ts_str.is_null() {
            set_retab_tabstop(&tabs);
        }
        coladvance(win, (*win).w_curswant);
        u_clearline(buf);
    }
}
