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
//! And a computed register (`"%`, `":`, `"=`, ...) is turned into one line
//! of its own -- `"=` being the exception, since its result may hold
//! newlines and has to be split.
//!
//! A put holds its lines as a share of the register's (see `RegLines`): the
//! splices below send `on_bytes`/`on_lines` callbacks, which may write the
//! very register being put, and the put goes on with the text it started
//! with.

#![forbid(unsafe_code)]

use crate::cstr;
use crate::ex_docmd::cmdmod_has;
use crate::mbyte::c_cluster_len;
use crate::message_fmt::msg_cstr;
use crate::semsg;
use crate::types::RegLines;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_int, c_uint};

use super::*;
use crate::normal::{set_visual_active, visual_active, visual_mode};
use crate::types::NUL;

mod block;
mod lines;

/// The state a put carries between its phases.
///
/// `kind`, `width` and `lines` are the text being put, which may come from a
/// real register or from a computed one.
pub(crate) struct Put {
    /// `FORWARD` for `p`, `BACKWARD` for `P`. A `PUT_LINE_SPLIT` or
    /// `PUT_LINE_FORWARD` put rewrites it to `FORWARD`.
    dir: c_int,
    count: c_int,
    /// The `PUT_*` set.
    flags: c_int,
    /// 'virtualedit', read once because it does not change under the put.
    ve_flags: c_uint,

    kind: MotionType,
    width: c_int,
    lines: RegLines,

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
        do_put('_' as c_int, None, dir, 1, PUT_LINE as c_int);

        stuff_readbuf_char(command_start_char);
        while count > 0 {
            let _ = stuff_inserted(NUL, 1, (count != 1) as c_int);
            if count != 1 {
                // `<CR>` then CTRL-U, to take off the indent 'autoindent'
                // would add. CTRL-U on its own would go back to the
                // previous line under 'nobackspace'-`eol`, so it is given
                // a space to consume.
                stuff_readbuf(b"\n ");
                stuff_readbuf_char(Ctrl_U);
            }
            count -= 1;
        }
    } else {
        let _ = stuff_inserted(command_start_char, count, false as c_int);
    }

    // The text goes in later, so the cursor cannot be moved past it here;
    // motion commands stuffed after the insert do it instead.
    if flags & PUT_CURSEND as c_int != 0 {
        if flags & PUT_LINE as c_int != 0 {
            stuff_readbuf(b"j0");
        } else {
            // Stuffing `l` would ring the bell at the end of a line, so
            // only do it when the cursor can actually move right:
            // 'virtualedit' allows it, or the cursor is neither at the
            // end of the line nor one past the end of the last line. The
            // last case is a Visual put over a selection reaching past
            // the end of the line, which joins the line below.
            let (one_past_line, eol) = with_cursor_text(|rest| {
                let one_past_line = cstr::byte_at(rest, 0) == 0;
                let eol = !one_past_line && cstr::byte_at(rest, c_cluster_len(rest)) == 0;
                (one_past_line, eol)
            });
            let ve_allows =
                ve_flags == kOptVeFlagAll as c_uint || ve_flags == kOptVeFlagOnemore as c_uint;
            let eof =
                Buf::current().b_ml.ml_line_count == Win::current().w_cursor.lnum && one_past_line;
            if ve_allows || !(eol || eof) {
                stuff_readbuf_char('l' as c_int);
            }
        }
    } else if flags & PUT_LINE as c_int != 0 {
        stuff_readbuf(b"g'[");
    }

    // Save the cursor position now (though no text), so that `u` after
    // `".p` restores it.
    if command_start_char == 'a' as c_int {
        let lnum = Win::current().w_cursor.lnum;
        let _ = u_save(lnum, lnum + 1);
    }
}

/// The cursor line from the cursor on, as a C string reads it: empty past the
/// end.
fn with_cursor_text<R>(f: impl FnOnce(&[u8]) -> R) -> R {
    let win = Win::current();
    let mut lines = win.buffer().lines();
    let line = lines.line(win.w_cursor.lnum);
    let col = usize::try_from(win.w_cursor.col).unwrap_or(0);
    f(line.get(col..).unwrap_or_default())
}

/// The length of the character under the cursor, its composing characters
/// included; 0 at the end of the line.
fn cursor_char_len() -> ColNr {
    with_cursor_text(|rest| ColNr::try_from(c_cluster_len(rest)).unwrap_or(0))
}

/// Split a `"=` result into lines at each `\n`.
///
/// Answers the lines and whether a trailing newline made the register
/// linewise.
fn split_expr_result(insert_string: &XString) -> (Vec<XString>, MotionType) {
    // Each line is its own string now, so the walk is one pass over the
    // bytes rather than upstream's count-then-fill over one allocation it
    // punched NULs into.
    let mut y_type = kMTCharWise;
    let mut lines: Vec<XString> = Vec::new();
    let mut rest: &[u8] = insert_string;
    loop {
        match rest.iter().position(|&b| b == b'\n') {
            None => {
                lines.push(XString::from_bytes(rest));
                break;
            }
            Some(at) => {
                lines.push(XString::from_bytes(&rest[..at]));
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
        if u_save_cursor().is_err() {
            return false;
        }
        let lnum = Win::current().w_cursor.lnum;
        let col = usize::try_from(Win::current().w_cursor.col).unwrap_or(0);
        let (split, tail) = {
            let mut lines = Buf::current().lines();
            let line = lines.line(lnum);
            // The second half starts after the cursor's character for `p`,
            // at it for `P`.
            let mut split = col;
            if self.dir == FORWARD && cstr::byte_at(line, col) != 0 {
                split += c_cluster_len(&line[col..]);
            }
            (split, XString::from_bytes(&line[split..]))
        };
        // Kept for the extmark_splice() the multiline put emits.
        self.split_pos = ColNr::try_from(split).unwrap_or(ColNr::MAX);
        let _ = ml_append_cstr(lnum, tail.as_cstr());

        // The head of the line stays where it is, cut short at the split.
        // The line is re-read because `ml_append` may have moved it.
        let head = XString::from_bytes(&Buf::current().lines().line(lnum)[..split]);
        let _ = ml_replace_owned(lnum, head);
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
        if self.kind == kMTBlockWise {
            let mut lnum = Win::current().w_cursor.lnum + self.lines.len() as LineNr + 1;
            lnum = lnum.min(Buf::current().b_ml.ml_line_count + 1);
            // `lnum` is capped at one past the last line.
            return u_save(Win::current().w_cursor.lnum - 1, lnum).is_ok();
        }

        if self.kind != kMTLineWise {
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
        if self.ve_flags != kOptVeFlagAll as c_uint || self.kind != kMTCharWise {
            return;
        }
        if gchar_cursor() == TAB {
            let viscol = getviscol();
            // No spaces needed for `p` on the last position of a tab, or
            // `P` on the first.
            let splits_tab = if self.dir == FORWARD {
                Buf::current().tab_padding(viscol) != 1
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

/// [`do_put`] of register `regname` at the cursor, which is first put back
/// on a line of the current buffer if it is not on one.
pub(crate) fn put_at_cursor(regname: c_int, dir: c_int, count: c_int, flags: c_int) {
    crate::cursor::check_cursor_lnum(Win::current());
    do_put(regname, None, dir, count, flags);
}

/// Put the contents of register `regname` into the text.
///
/// The caller must check that `regname` is valid, and the cursor must be on a
/// valid line. `reg` may be a register the caller already fetched --
/// Visual-mode replace does that, so that the text it just deleted is not
/// what gets put back.
///
/// `dir` is `BACKWARD` for `P` and `FORWARD` for `p`; `flags` is the `PUT_*`
/// set: `PUT_FIXINDENT` reindents (`]p`), `PUT_CURSEND` leaves the cursor
/// after the new text, `PUT_LINE` forces a linewise put (`:put`), and
/// `PUT_BLOCK_INNER` leaves a block's trailing spaces off.
///
/// May run the clipboard provider and, by way of `"=`, arbitrary Vimscript.
pub fn do_put(regname: c_int, reg: Option<&YankReg>, dir: c_int, count: c_int, flags: c_int) {
    let orig_start = Buf::current().b_op_start;
    let orig_end = Buf::current().b_op_end;
    let ve_flags = get_ve_flags(Win::current());

    // Remove any preinserted completion text (vim/vim#19329).
    if ins_compl_preinsert_effect() {
        ins_compl_delete(false);
    }

    // Defaults for the `'[` and `']` marks.
    Buf::current().b_op_start = Win::current().w_cursor;
    Buf::current().b_op_end = Win::current().w_cursor;

    if regname == '.' as c_int && reg.is_none() {
        put_last_insert(dir, count, flags, ve_flags);
        return;
    }

    // A computed register becomes one line of its own. The register is only
    // read when the caller did not hand one over, and `"=` running Vimscript
    // is this function's own promise.
    let insert_string = match reg {
        Some(_) => None,
        None => match get_spec_reg(regname, true) {
            Some(None) => return, // nothing to put
            Some(Some(mut text)) => {
                // Read as the C string it was, to its first NUL.
                if let Some(nul) = text.iter().position(|&b| b == 0) {
                    text.truncate(nul);
                }
                Some(text)
            }
            None => None,
        },
    };

    if Buf::current().terminal.is_null() {
        // Saving for undo can run autocommands, which may write the register,
        // so it happens before the register is read.
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
        kind: kMTCharWise,
        width: 0,
        lines: RegLines::NONE,
        nr_lines: 0,
        split_pos: 0,
    };

    if let Some(text) = insert_string {
        if regname == '=' as c_int {
            // Only `"=` can produce more than one line.
            let (lines, kind) = split_expr_result(&text);
            put.kind = kind;
            put.lines = RegLines::new(lines);
        } else {
            put.lines = RegLines::new(vec![text]);
        }
    } else {
        // Visual-mode replace may have handed us the register already, so
        // that the deleted text is not what comes back.
        let share = |reg: &YankReg| (reg.kind, reg.width, reg.lines.clone());
        (put.kind, put.width, put.lines) = match reg {
            Some(reg) => share(reg),
            None => get_yank_register(regname, YREG_PASTE).read(share),
        };
    }

    'end: {
        if !Buf::current().terminal.is_null() {
            terminal_paste(&put.lines, count);
            break 'end;
        }

        if put.kind == kMTLineWise {
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
            put.kind = kMTLineWise;
        }

        if put.lines.is_empty() {
            let display = transchar(regname);
            let name = if regname != 0 {
                cstr::in_chars(&display)
            } else {
                c"\""
            };
            let name = msg_cstr(name);
            semsg!("E353: Nothing in register {name}");
            break 'end;
        }

        if !put.save_for_undo() {
            break 'end;
        }
        put.make_room_for_virtualedit();

        let mut lnum = Win::current().w_cursor.lnum;
        let mut col = Win::current().w_cursor.col;

        if put.kind == kMTBlockWise {
            put.blockwise(lnum);
        } else {
            if put.kind == kMTCharWise {
                // For charwise text, FORWARD is BACKWARD on the next
                // character.
                if put.dir == FORWARD && gchar_cursor() != NUL {
                    let bytelen = cursor_char_len();
                    col += bytelen;
                    if !put.lines[0].is_empty() {
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

            if put.kind == kMTCharWise && put.lines.len() == 1 {
                put.charwise_one_line(lnum, col);
            } else {
                put.multiline(lnum, col, new_cursor);
            }
        }

        msgmore(put.nr_lines);
        Win::current().w_set_curswant = true;

        // Don't leave the cursor after the NUL.
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
