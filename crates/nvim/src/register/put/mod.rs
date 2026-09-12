//! `do_put` -- `p`, `P`, `gp`, `gP`, `]p`, `[p` and `zp`.
//!
//! The driver here does four things in order: work out what text is being
//! put, get it into a shape the three inserters can use, save for undo, and
//! then hand off by motion type:
//!
//! | register | where |
//! | --- | --- |
//! | blockwise | [`block`] |
//! | charwise, one line | [`lines`]'s `charwise_one_line` |
//! | charwise over several lines, or linewise | [`lines`]'s `multiline` |
//!
//! Two of the sources are not registers at all. `".` is handled by
//! [`put_last_insert`], which does not put anything: it stuffs an Insert-mode
//! command into the read buffer, because the last insert is *keys*, not text.
//! And a computed register (`"%`, `":`, `"=`, ...) is turned into a
//! one-element fake `YankReg` on the stack -- `"=` being the exception,
//! since its result may hold newlines and has to be split.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::ex_docmd::cmdmod_has;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_uint, c_void};

use super::*;
use crate::normal::{set_visual_active, visual_active, visual_mode};
use crate::types::NUL;

mod block;
mod lines;

/// The state a put carries between its phases.
///
/// The four `y_*` fields are the text being put, which may come from a real
/// register, from a computed one, or from a stack-allocated fake.
pub(crate) struct Put {
    /// `FORWARD` for `p`, `BACKWARD` for `P`. A `PUT_LINE_SPLIT` or
    /// `PUT_LINE_FORWARD` put rewrites it to `FORWARD`.
    dir: c_int,
    count: c_int,
    /// The `PUT_*` set.
    flags: c_int,
    /// 'virtualedit', read once because it does not change under the put.
    ve_flags: c_uint,

    y_type: MotionType,
    y_size: size_t,
    y_width: c_int,
    y_array: *mut String_0,

    /// Lines the put added, for `msgmore` and `mark_adjust`.
    nr_lines: LineNr,
    /// Where a `PUT_LINE_SPLIT` broke the cursor line, for the extmark
    /// splice.
    split_pos: ColNr,
}

/// `".p` -- putting the last inserted text.
///
/// Nothing is put here. The register holds the *keys* of the last insert,
/// newlines and all, so the only way to reproduce it is to re-enter Insert
/// mode and replay them: this stuffs a command into the read buffer and
/// returns, and the main loop does the work.
fn put_last_insert(dir: c_int, mut count: c_int, flags: c_int, ve_flags: c_uint) {
    let non_linewise_vis = visual_active() && !visual_mode().is_line();

    // A Visual selection is replaced (`c`); `PUT_LINE` opens its own line
    // below, so it inserts at the start of it.
    let command_start_char = if non_linewise_vis {
        'c' as c_int
    } else if flags & PUT_LINE as c_int != 0 {
        'i' as c_int
    } else if dir == FORWARD {
        'a' as c_int
    } else {
        'i' as c_int
    };

    if flags & PUT_LINE as c_int != 0 {
        // Open the line with a black-hole `:put _`, so that 'autoindent'
        // does not reach the text.
        let nothing = ::core::ptr::null_mut();
        // SAFETY: the cursor is on a valid line, and the black hole has
        // nothing in it to put.
        unsafe { do_put('_' as c_int, nothing, dir, 1, PUT_LINE as c_int) };

        // SAFETY: stuffing keys into the read buffer, from NUL-terminated
        // literals; `stuff_inserted` replays the last insert.
        unsafe {
            stuff_readbuf_char(command_start_char);
            while count > 0 {
                let _ = stuff_inserted(NUL, 1, (count != 1) as c_int);
                if count != 1 {
                    // `<CR>` then CTRL-U, to take off the indent 'autoindent'
                    // would add. CTRL-U on its own would go back to the
                    // previous line under 'nobackspace'-`eol`, so it is given
                    // a space to consume.
                    stuff_readbuf(c"\n ".as_ptr());
                    stuff_readbuf_char(Ctrl_U);
                }
                count -= 1;
            }
        }
    } else {
        // SAFETY: replays the last insert into the read buffer.
        let _ = unsafe { stuff_inserted(command_start_char, count, false as c_int) };
    }

    // The text goes in later, so the cursor cannot be moved past it here;
    // motion commands stuffed after the insert do it instead.
    if flags & PUT_CURSEND as c_int != 0 {
        if flags & PUT_LINE as c_int != 0 {
            // SAFETY: a NUL-terminated literal.
            unsafe { stuff_readbuf(c"j0".as_ptr()) };
        } else {
            // Stuffing `l` would ring the bell at the end of a line, so
            // only do it when the cursor can actually move right:
            // 'virtualedit' allows it, or the cursor is neither at the
            // end of the line nor one past the end of the last line. The
            // last case is a Visual put over a selection reaching past
            // the end of the line, which joins the line below.
            //
            // SAFETY: the cursor is on a valid line, so its position is a
            // byte of that line's NUL-terminated text.  `!one_past_line` is
            // what proves the character step below stays inside it, so the
            // chain is left whole.
            let cursor_pos = get_cursor_pos_ptr();
            let one_past_line = unsafe { c_int::from(*cursor_pos) } == NUL;
            let eol = !one_past_line
                && unsafe { c_int::from(*cursor_pos.offset(utfc_ptr2len(cursor_pos) as isize)) }
                    == NUL;
            let ve_allows =
                ve_flags == kOptVeFlagAll as c_uint || ve_flags == kOptVeFlagOnemore as c_uint;
            let eof =
                Buf::current().b_ml.ml_line_count == Win::current().w_cursor.lnum && one_past_line;
            if ve_allows || !(eol || eof) {
                stuff_readbuf_char('l' as c_int);
            }
        }
    } else if flags & PUT_LINE as c_int != 0 {
        // SAFETY: a NUL-terminated literal.
        unsafe { stuff_readbuf(c"g'[".as_ptr()) };
    }

    // Save the cursor position now (though no text), so that `u` after
    // `".p` restores it.
    if command_start_char == 'a' as c_int {
        let lnum = Win::current().w_cursor.lnum;
        let _ = u_save(lnum, lnum + 1);
    }
}

/// Split a `"=` result into lines in place, overwriting each `\n` with a NUL.
///
/// Answers the allocated line array and whether a trailing newline made the
/// register linewise. `insert_string` is edited, not copied.
///
fn split_expr_result(insert_string: &String_0) -> (Vec<String_0>, MotionType) {
    // Each line is its own string now, so the walk is one pass over the
    // bytes rather than upstream's count-then-fill over one allocation it
    // punched NULs into.
    let mut y_type = kMTCharWise;
    let mut lines: Vec<String_0> = Vec::new();
    let mut rest = insert_string.as_bytes();
    loop {
        match rest.iter().position(|&b| b == b'\n') {
            None => {
                lines.push(String_0::from_bytes(rest));
                break;
            }
            Some(at) => {
                lines.push(String_0::from_bytes(&rest[..at]));
                rest = &rest[at + 1..];
                // A trailing newline makes the register linewise.
                if rest.is_empty() {
                    y_type = kMTLineWise;
                    break;
                }
            }
        }
    }
    (lines, y_type)
}

impl Put {
    /// `p`/`P` in Visual mode over a linewise register: break the cursor line
    /// in two so that the text goes *between* the halves.
    ///
    /// Answers false when undo could not be saved.
    fn split_current_line(&mut self) -> bool {
        // SAFETY: the cursor is on a valid line, which is what undo saves.
        if u_save_cursor().is_err() {
            return false;
        }
        // SAFETY (these four): the cursor is on a valid line, so all three
        // answers are that line's NUL-terminated text and a position in it.
        let curline = get_cursor_line_ptr();
        let p_orig = get_cursor_pos_ptr();
        let plen = get_cursor_pos_len() as size_t;
        let mut p = p_orig;
        // The second half starts after the cursor's character for `p`, at it
        // for `P`.
        //
        // SAFETY: the `!= NUL` test in front is what proves that stepping
        // over the cursor's character stays inside the line.
        if self.dir == FORWARD && unsafe { c_int::from(*p) } != NUL {
            p = unsafe { p.offset(utfc_ptr2len(p) as isize) };
        }
        // Kept for the extmark_splice() the multiline put emits.
        //
        // SAFETY: `p` is a position in `curline`.
        self.split_pos = unsafe { p.offset_from(curline) } as ColNr;

        // SAFETY: `p` is inside the cursor line, and what is left of the line
        // from there is `plen` less the bytes in front of it.
        let taillen = plen.wrapping_sub(unsafe { p.offset_from(p_orig) } as size_t);
        // SAFETY: `p` points at those `taillen` bytes.
        let tail = unsafe { xmemdupz(p as *const c_void, taillen) } as *mut c_char;
        // SAFETY: `tail` is NUL-terminated and `ml_append` copies it.
        let _ = unsafe { ml_append(Win::current().w_cursor.lnum, tail, 0, false) };
        // SAFETY: the copy is ours.
        unsafe { xfree(tail as *mut c_void) };

        // The head of the line stays where it is, cut short at the split.
        //
        // SAFETY: the line is re-read because `ml_append` may have moved it;
        // `split_pos` is a column of it, and `ml_replace` takes the copy over.
        let head = get_cursor_line_ptr() as *const c_void;
        let head = unsafe { xmemdupz(head, self.split_pos as size_t) } as *mut c_char;
        let _ = unsafe { ml_replace(Win::current().w_cursor.lnum, head, false) };
        self.nr_lines += 1;
        self.dir = FORWARD;

        let lnum = Win::current().w_cursor.lnum;
        buf_updates_send_changes(Buf::current(), lnum, 1, 1);
        true
    }

    /// Save for undo and put the cursor where the text goes.
    ///
    /// Answers false when undo could not be saved.
    fn save_for_undo(&self) -> bool {
        if self.y_type == kMTBlockWise {
            let mut lnum = Win::current().w_cursor.lnum + self.y_size as LineNr + 1;
            lnum = lnum.min(Buf::current().b_ml.ml_line_count + 1);
            // SAFETY: the cursor is on a valid line and `lnum` is capped at
            // one past the last, so the range is the buffer's.
            return u_save(Win::current().w_cursor.lnum - 1, lnum).is_ok();
        }

        if self.y_type != kMTLineWise {
            // SAFETY: the cursor is on a valid line.
            return u_save_cursor().is_ok();
        }

        // Correct for a closed fold. The cursor must not move yet:
        // u_save() reads it.
        let cursor_lnum = Win::current().w_cursor.lnum;
        let mut lnum = if self.dir == BACKWARD {
            Win::current()
                .fold_first(cursor_lnum)
                .unwrap_or(cursor_lnum)
        } else {
            Win::current().fold_last(cursor_lnum)
        };
        if self.dir == FORWARD {
            lnum += 1;
        }
        // An empty buffer's one empty line is going to be replaced, so it
        // has to be part of what is saved.
        //
        let saved = if buf_is_empty(Buf::current()) {
            u_save(0, 2)
        } else {
            u_save(lnum - 1, lnum)
        };
        if saved.is_err() {
            return false;
        }
        Win::current().w_cursor.lnum = if self.dir == FORWARD { lnum - 1 } else { lnum };
        Buf::current().b_op_start = Win::current().w_cursor; // for mark_adjust()
        true
    }

    /// With 'virtualedit' "all", make the cursor a real position before the
    /// text goes in: break a tab into spaces, or pad out past the end of the
    /// line.
    fn make_room_for_virtualedit(&self) {
        if self.ve_flags != kOptVeFlagAll as c_uint || self.y_type != kMTCharWise {
            return;
        }
        // SAFETY (all through): the cursor is on a valid line, which is the
        // line every one of these reads, measures or moves within.
        if gchar_cursor() == TAB {
            let viscol = getviscol();
            let ts = Buf::current().b_p_ts;
            // No spaces needed for `p` on the last position of a tab, or
            // `P` on the first.
            let splits_tab = if self.dir == FORWARD {
                let pad = unsafe { tabstop_padding(viscol, ts, Buf::current().b_p_vts_array) };
                pad != 1
            } else {
                Win::current().w_cursor.coladd > 0
            };
            if splits_tab {
                coladvance_force(viscol);
            } else {
                Win::current().w_cursor.coladd = 0;
            }
        } else if Win::current().w_cursor.coladd > 0 || gchar_cursor() == NUL {
            let to = getviscol() + c_int::from(self.dir == FORWARD);
            coladvance_force(to);
        }
    }
}

/// Put the contents of register `regname` into the text.
///
/// The caller must check that `regname` is valid. `reg` may be a register the
/// caller already fetched -- Visual-mode replace does that, so that the text
/// it just deleted is not what gets put back.
///
/// `dir` is `BACKWARD` for `P` and `FORWARD` for `p`; `flags` is the `PUT_*`
/// set: `PUT_FIXINDENT` reindents (`]p`), `PUT_CURSEND` leaves the cursor
/// after the new text, `PUT_LINE` forces a linewise put (`:put`), and
/// `PUT_BLOCK_INNER` leaves a block's trailing spaces off.
///
/// # Safety
/// The cursor must be on a valid line. May run the clipboard provider and, by
/// way of `"=`, arbitrary Vimscript.
pub unsafe fn do_put(regname: c_int, reg: *mut YankReg, dir: c_int, count: c_int, flags: c_int) {
    let orig_start = Buf::current().b_op_start;
    let orig_end = Buf::current().b_op_end;
    // SAFETY: a live window.
    let ve_flags = get_ve_flags(Win::current());

    // Remove any preinserted completion text (vim/vim#19329).
    if ins_compl_preinsert_effect() {
        ins_compl_delete(false);
    }

    // Defaults for the `'[` and `']` marks.
    Buf::current().b_op_start = Win::current().w_cursor;
    Buf::current().b_op_end = Win::current().w_cursor;

    if regname == '.' as c_int && reg.is_null() {
        // SAFETY: the cursor is on a valid line.
        put_last_insert(dir, count, flags, ve_flags);
        return;
    }

    // A computed register becomes a fake one-line yankreg.
    let mut spec_data: *mut c_char = ::core::ptr::null_mut();
    let mut allocated = false;
    // SAFETY: both out-parameters are writable locals.  The chain is left
    // whole: the register is only read when the caller did not hand one over,
    // and `"=` running Vimscript is this function's own promise.
    let nothing_to_put = reg.is_null()
        && unsafe { get_spec_reg(regname, &raw mut spec_data, &raw mut allocated, true) }
        && spec_data.is_null();
    if nothing_to_put {
        return;
    }
    // The answer owns its bytes either way: `allocated` is `get_spec_reg`
    // saying the block is ours, and otherwise it lent one that is copied.
    // SAFETY: a non-null answer is NUL-terminated.
    let mut insert_string = if spec_data.is_null() {
        String_0::NULL
    } else if allocated {
        unsafe { String_0::from_owned_parts(spec_data, cstr::bytes_at(spec_data).len()) }
    } else {
        unsafe { String_0::from_bytes(cstr::bytes_at(spec_data)) }
    };
    // The lines `"=` splits into, kept alive for as long as `put` names them.
    let mut expr_lines: Vec<String_0>;

    if Buf::current().terminal.is_null() {
        // Saving for undo can run autocommands, which would invalidate
        // `y_array`, so it happens before the register is read.
        let lnum = Win::current().w_cursor.lnum;
        if u_save(lnum, lnum + 1).is_err() {
            return;
        }
    }

    let mut put = Put {
        dir,
        count,
        flags,
        ve_flags,
        y_type: kMTCharWise,
        y_size: 0,
        y_width: 0,
        y_array: ::core::ptr::null_mut(),
        nr_lines: 0,
        split_pos: 0,
    };

    if !insert_string.is_null() {
        if regname == '=' as c_int {
            // Only `"=` can produce more than one line.
            let (lines, y_type) = split_expr_result(&insert_string);
            expr_lines = lines;
            put.y_type = y_type;
            put.y_size = expr_lines.len();
            put.y_array = expr_lines.as_mut_ptr();
        } else {
            put.y_size = 1;
            put.y_array = &raw mut insert_string;
        }
    } else {
        // Visual-mode replace may have handed us the register already, so
        // that the deleted text is not what comes back.
        //
        // SAFETY: `regname` is a register name the caller checked.
        let reg = if reg.is_null() {
            unsafe { get_yank_register(regname, YREG_PASTE) }
        } else {
            reg
        };
        // SAFETY: a live register, whose four fields describe its text.
        unsafe {
            put.y_type = (*reg).y_type;
            put.y_width = (*reg).y_width;
            put.y_size = (*reg).y_size;
            put.y_array = (*reg).y_array;
        }
    }

    'end: {
        if !Buf::current().terminal.is_null() {
            // SAFETY: `y_array` holds `y_size` NUL-terminated strings.
            unsafe { terminal_paste(count, put.y_array, put.y_size) };
            break 'end;
        }

        if put.y_type == kMTLineWise {
            // SAFETY: the cursor is on a valid line.
            let split_failed =
                put.flags & PUT_LINE_SPLIT as c_int != 0 && !put.split_current_line();
            if split_failed {
                break 'end;
            }
            if put.flags & PUT_LINE_FORWARD as c_int != 0 {
                // `p` over a Visual block puts the lines below the block.
                Win::current().w_cursor = Buf::current().b_visual.vi_end;
                put.dir = FORWARD;
            }
            Buf::current().b_op_start = Win::current().w_cursor;
            Buf::current().b_op_end = Win::current().w_cursor;
        }

        if put.flags & PUT_LINE as c_int != 0 {
            // `:put`, or `p` in Visual line mode.
            put.y_type = kMTLineWise;
        }

        if put.y_size == 0 || put.y_array.is_null() {
            let display = transchar(regname);
            let mut name = c"\"".as_ptr();
            if regname != 0 {
                name = display.as_ptr();
            }
            // SAFETY: the format takes the single `%s` given, and `name` is // NUL-terminated.
            let name = unsafe { c_str(name) };
            semsg!("E353: Nothing in register {name}");
            break 'end;
        }

        // SAFETY: the cursor is on a valid line.
        if !put.save_for_undo() {
            break 'end;
        }
        put.make_room_for_virtualedit();

        let mut lnum = Win::current().w_cursor.lnum;
        let mut col = Win::current().w_cursor.col;

        if put.y_type == kMTBlockWise {
            put.blockwise(lnum);
        } else {
            if put.y_type == kMTCharWise {
                // For charwise text, FORWARD is BACKWARD on the next
                // character.
                //
                // SAFETY: the cursor is on a valid line, and the `!= NUL`
                // test in front is what proves it has a character to step
                // over.
                if put.dir == FORWARD && gchar_cursor() != NUL {
                    let bytelen = unsafe { utfc_ptr2len(get_cursor_pos_ptr()) };
                    col += bytelen;
                    // SAFETY: a charwise register holds at least one line.
                    if !unsafe { (*put.y_array).is_empty() } {
                        Win::current().w_cursor.col += bytelen;
                        Buf::current().b_op_end.col += bytelen;
                    }
                }
                Buf::current().b_op_start = Win::current().w_cursor;
            } else if put.dir == BACKWARD {
                // Linewise: BACKWARD is FORWARD on the previous line.
                lnum -= 1;
            }
            let new_cursor = Win::current().w_cursor;

            if put.y_type == kMTCharWise && put.y_size == 1 {
                put.charwise_one_line(lnum, col);
            } else {
                put.multiline(lnum, col, new_cursor);
            }
        }

        msgmore(put.nr_lines);
        Win::current().w_set_curswant = true;

        // Don't leave the cursor after the NUL.
        // SAFETY: the cursor is on a line of the current buffer.
        let len = get_cursor_line_len();
        if Win::current().w_cursor.col > len {
            if ve_flags == kOptVeFlagAll as c_uint {
                Win::current().w_cursor.coladd = Win::current().w_cursor.col - len;
            }
            Win::current().w_cursor.col = len;
        }
    }

    if cmdmod_has(CmdModFlags::LOCKMARKS) {
        Buf::current().b_op_start = orig_start;
        Buf::current().b_op_end = orig_end;
    }

    if Buf::current().terminal.is_null() {
        set_visual_active(false);
    }

    adjust_cursor_eol();
}
