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
//! one-element fake `yankreg_T` on the stack -- `"=` being the exception,
//! since its result may hold newlines and has to be split.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::ex_docmd::cmdmod_has;
use crate::semsg_c;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_uint, c_void};

use super::*;
use crate::normal::{set_visual_active, visual_active, visual_mode};
use crate::types::{FAIL, NUL};

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
    nr_lines: linenr_T,
    /// Where a `PUT_LINE_SPLIT` broke the cursor line, for the extmark
    /// splice.
    split_pos: colnr_T,
}

/// `".p` -- putting the last inserted text.
///
/// Nothing is put here. The register holds the *keys* of the last insert,
/// newlines and all, so the only way to reproduce it is to re-enter Insert
/// mode and replay them: this stuffs a command into the read buffer and
/// returns, and the main loop does the work.
///
/// # Safety
/// The cursor must be on a valid line.
unsafe fn put_last_insert(dir: c_int, mut count: c_int, flags: c_int, ve_flags: c_uint) {
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
                stuff_inserted(NUL, 1, (count != 1) as c_int);
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
        unsafe { stuff_inserted(command_start_char, count, false as c_int) };
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
            let eof = cur_buf().b_ml.ml_line_count == cur_win().w_cursor.lnum && one_past_line;
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
        let lnum = cur_win().w_cursor.lnum;
        // SAFETY: the cursor is on a valid line.
        u_save(lnum, lnum + 1);
    }
}

/// Split a `"=` result into lines in place, overwriting each `\n` with a NUL.
///
/// Answers the allocated line array and whether a trailing newline made the
/// register linewise. `insert_string` is edited, not copied.
///
/// # Safety
/// `insert_string.data` must be an allocated, NUL-terminated string of
/// `insert_string.size` bytes.
unsafe fn split_expr_result(insert_string: String_0) -> (*mut String_0, size_t, MotionType) {
    // Two passes over the same walk: the first counts the lines, the
    // second fills the array the count sized.
    let mut y_array: *mut String_0 = ::core::ptr::null_mut();
    let mut y_type = kMTCharWise;
    loop {
        // A pure pointer walk, so it keeps one region around the whole of it.
        //
        // SAFETY: the caller promises `insert_string` is an allocated,
        // NUL-terminated string of `len()` bytes.  `ptr` only ever moves
        // forward inside it: `vim_strchr` answers a newline of that string or
        // null, and the byte after a newline is still one of its own -- worst
        // case the NUL, which is what ends the walk.  On the filling pass
        // `y_array` has exactly the `y_size` slots the counting pass measured,
        // and `y_size` is only ever the index of the line just started.
        let (y_size, done) = unsafe {
            let mut y_size: size_t = 0;
            let mut ptr = insert_string.data();
            let mut ptrlen = insert_string.len();
            while !ptr.is_null() {
                if !y_array.is_null() {
                    (*y_array.add(y_size)).set_data(ptr);
                }
                y_size = y_size.wrapping_add(1);
                let mut tmp = vim_strchr(ptr, '\n' as c_int);
                if tmp.is_null() {
                    if !y_array.is_null() {
                        (*y_array.add(y_size.wrapping_sub(1))).set_len(ptrlen);
                    }
                } else {
                    if !y_array.is_null() {
                        *tmp = NUL as c_char;
                        let len = tmp.offset_from(ptr) as size_t;
                        (*y_array.add(y_size.wrapping_sub(1))).set_len(len);
                        ptrlen = ptrlen.wrapping_sub(len.wrapping_add(1));
                    }
                    tmp = tmp.add(1);
                    // A trailing newline makes the register linewise.
                    if c_int::from(*tmp) == NUL {
                        y_type = kMTLineWise;
                        break;
                    }
                }
                ptr = tmp;
            }
            (y_size, !y_array.is_null())
        };
        if done {
            return (y_array, y_size, y_type);
        }
        // SAFETY: `y_size` slots is what the counting pass just measured.
        let room = y_size.wrapping_mul(::core::mem::size_of::<String_0>());
        y_array = unsafe { xmalloc(room) } as *mut String_0;
    }
}

impl Put {
    /// `p`/`P` in Visual mode over a linewise register: break the cursor line
    /// in two so that the text goes *between* the halves.
    ///
    /// Answers false when undo could not be saved.
    ///
    /// # Safety
    /// The cursor must be on a valid line.
    unsafe fn split_current_line(&mut self) -> bool {
        // SAFETY: the cursor is on a valid line, which is what undo saves.
        if u_save_cursor() == FAIL {
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
        self.split_pos = unsafe { p.offset_from(curline) } as colnr_T;

        // SAFETY: `p` is inside the cursor line, and what is left of the line
        // from there is `plen` less the bytes in front of it.
        let taillen = plen.wrapping_sub(unsafe { p.offset_from(p_orig) } as size_t);
        // SAFETY: `p` points at those `taillen` bytes.
        let tail = unsafe { xmemdupz(p as *const c_void, taillen) } as *mut c_char;
        // SAFETY: `tail` is NUL-terminated and `ml_append` copies it.
        unsafe { ml_append(cur_win().w_cursor.lnum, tail, 0, false) };
        // SAFETY: the copy is ours.
        unsafe { xfree(tail as *mut c_void) };

        // The head of the line stays where it is, cut short at the split.
        //
        // SAFETY: the line is re-read because `ml_append` may have moved it;
        // `split_pos` is a column of it, and `ml_replace` takes the copy over.
        let head = get_cursor_line_ptr() as *const c_void;
        let head = unsafe { xmemdupz(head, self.split_pos as size_t) } as *mut c_char;
        unsafe { ml_replace(cur_win().w_cursor.lnum, head, false) };
        self.nr_lines += 1;
        self.dir = FORWARD;

        let lnum = cur_win().w_cursor.lnum;
        // SAFETY: a live buffer, in which one line just became two.
        unsafe { buf_updates_send_changes(curbuf.get(), lnum, 1, 1) };
        true
    }

    /// Save for undo and put the cursor where the text goes.
    ///
    /// Answers false when undo could not be saved.
    ///
    /// # Safety
    /// The cursor must be on a valid line.
    unsafe fn save_for_undo(&self) -> bool {
        if self.y_type == kMTBlockWise {
            let mut lnum = cur_win().w_cursor.lnum + self.y_size as linenr_T + 1;
            lnum = lnum.min(cur_buf().b_ml.ml_line_count + 1);
            // SAFETY: the cursor is on a valid line and `lnum` is capped at
            // one past the last, so the range is the buffer's.
            return u_save(cur_win().w_cursor.lnum - 1, lnum) != FAIL;
        }

        if self.y_type != kMTLineWise {
            // SAFETY: the cursor is on a valid line.
            return u_save_cursor() != FAIL;
        }

        // Correct for a closed fold. The cursor must not move yet:
        // u_save() reads it.
        let cursor_lnum = cur_win().w_cursor.lnum;
        let mut lnum = if self.dir == BACKWARD {
            cur_win().fold_first(cursor_lnum).unwrap_or(cursor_lnum)
        } else {
            cur_win().fold_last(cursor_lnum)
        };
        if self.dir == FORWARD {
            lnum += 1;
        }
        // An empty buffer's one empty line is going to be replaced, so it
        // has to be part of what is saved.
        //
        // SAFETY (these three): a live buffer, and `lnum` is a line of it.
        let saved = if unsafe { buf_is_empty(curbuf.get()) } {
            u_save(0, 2)
        } else {
            u_save(lnum - 1, lnum)
        };
        if saved == FAIL {
            return false;
        }
        cur_win().w_cursor.lnum = if self.dir == FORWARD { lnum - 1 } else { lnum };
        cur_buf().b_op_start = cur_win().w_cursor; // for mark_adjust()
        true
    }

    /// With 'virtualedit' "all", make the cursor a real position before the
    /// text goes in: break a tab into spaces, or pad out past the end of the
    /// line.
    ///
    /// # Safety
    /// The cursor must be on a valid line.
    unsafe fn make_room_for_virtualedit(&self) {
        if self.ve_flags != kOptVeFlagAll as c_uint || self.y_type != kMTCharWise {
            return;
        }
        // SAFETY (all through): the cursor is on a valid line, which is the
        // line every one of these reads, measures or moves within.
        if gchar_cursor() == TAB {
            let viscol = unsafe { getviscol() };
            let ts = cur_buf().b_p_ts;
            // No spaces needed for `p` on the last position of a tab, or
            // `P` on the first.
            let splits_tab = if self.dir == FORWARD {
                let pad = unsafe { tabstop_padding(viscol, ts, cur_buf().b_p_vts_array) };
                pad != 1
            } else {
                cur_win().w_cursor.coladd > 0
            };
            if splits_tab {
                unsafe { coladvance_force(viscol) };
            } else {
                cur_win().w_cursor.coladd = 0;
            }
        } else if cur_win().w_cursor.coladd > 0 || gchar_cursor() == NUL {
            let to = unsafe { getviscol() } + c_int::from(self.dir == FORWARD);
            unsafe { coladvance_force(to) };
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
pub unsafe fn do_put(regname: c_int, reg: *mut yankreg_T, dir: c_int, count: c_int, flags: c_int) {
    let orig_start = cur_buf().b_op_start;
    let orig_end = cur_buf().b_op_end;
    // SAFETY: a live window.
    let ve_flags = get_ve_flags(cur_win());

    // Remove any preinserted completion text (vim/vim#19329).
    // SAFETY: main thread; the completion state is its own.
    if unsafe { ins_compl_preinsert_effect() } {
        // SAFETY: as above.
        unsafe { ins_compl_delete(false) };
    }

    // Defaults for the `'[` and `']` marks.
    cur_buf().b_op_start = cur_win().w_cursor;
    cur_buf().b_op_end = cur_win().w_cursor;

    if regname == '.' as c_int && reg.is_null() {
        // SAFETY: the cursor is on a valid line.
        unsafe { put_last_insert(dir, count, flags, ve_flags) };
        return;
    }

    // A computed register becomes a fake one-line yankreg.
    let mut insert_string = String_0::NULL;
    let mut allocated = false;
    // SAFETY: both out-parameters are writable locals.  The chain is left
    // whole: the register is only read when the caller did not hand one over,
    // and `"=` running Vimscript is this function's own promise.
    let nothing_to_put = reg.is_null()
        && unsafe { get_spec_reg(regname, insert_string.data_mut(), &raw mut allocated, true) }
        && insert_string.data().is_null();
    if nothing_to_put {
        return;
    }

    if cur_buf().terminal.is_null() {
        // Saving for undo can run autocommands, which would invalidate
        // `y_array`, so it happens before the register is read.
        let lnum = cur_win().w_cursor.lnum;
        // SAFETY: the cursor is on a valid line.
        if u_save(lnum, lnum + 1) == FAIL {
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

    if !insert_string.data().is_null() {
        // SAFETY: the computed register answered a NUL-terminated string.
        insert_string.set_len(unsafe { strlen(insert_string.data()) });
        if regname == '=' as c_int {
            // Only `"=` can produce more than one line.
            //
            // SAFETY: as above, and it is an allocated one this call owns.
            (put.y_array, put.y_size, put.y_type) = unsafe { split_expr_result(insert_string) };
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
        if !cur_buf().terminal.is_null() {
            // SAFETY: `y_array` holds `y_size` NUL-terminated strings.
            unsafe { terminal_paste(count, put.y_array, put.y_size) };
            break 'end;
        }

        if put.y_type == kMTLineWise {
            // SAFETY: the cursor is on a valid line.
            let split_failed =
                put.flags & PUT_LINE_SPLIT as c_int != 0 && !unsafe { put.split_current_line() };
            if split_failed {
                break 'end;
            }
            if put.flags & PUT_LINE_FORWARD as c_int != 0 {
                // `p` over a Visual block puts the lines below the block.
                cur_win().w_cursor = cur_buf().b_visual.vi_end;
                put.dir = FORWARD;
            }
            cur_buf().b_op_start = cur_win().w_cursor;
            cur_buf().b_op_end = cur_win().w_cursor;
        }

        if put.flags & PUT_LINE as c_int != 0 {
            // `:put`, or `p` in Visual line mode.
            put.y_type = kMTLineWise;
        }

        if put.y_size == 0 || put.y_array.is_null() {
            // SAFETY: `transchar` answers its own NUL-terminated buffer.
            let display = unsafe { transchar(regname) };
            let mut name = c"\"".as_ptr();
            if regname != 0 {
                name = display.as_ptr();
            }
            // SAFETY: a NUL-terminated message literal, translated.
            let fmt = unsafe { gettext(c"E353: Nothing in register %s".as_ptr()) };
            // SAFETY: the format takes the single `%s` given, and `name` is
            // NUL-terminated.
            unsafe { semsg_c!(fmt, name) };
            break 'end;
        }

        // SAFETY: the cursor is on a valid line.
        if !unsafe { put.save_for_undo() } {
            break 'end;
        }
        // SAFETY: as above.
        unsafe { put.make_room_for_virtualedit() };

        let mut lnum = cur_win().w_cursor.lnum;
        let mut col = cur_win().w_cursor.col;

        if put.y_type == kMTBlockWise {
            // SAFETY: the cursor is on a valid line and `y_array` holds
            // `y_size` NUL-terminated strings.
            unsafe { put.blockwise(lnum) };
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
                        cur_win().w_cursor.col += bytelen;
                        cur_buf().b_op_end.col += bytelen;
                    }
                }
                cur_buf().b_op_start = cur_win().w_cursor;
            } else if put.dir == BACKWARD {
                // Linewise: BACKWARD is FORWARD on the previous line.
                lnum -= 1;
            }
            let new_cursor = cur_win().w_cursor;

            // SAFETY (both): `lnum`/`col` is a position of the buffer and
            // undo has just been saved.
            if put.y_type == kMTCharWise && put.y_size == 1 {
                unsafe { put.charwise_one_line(lnum, col) };
            } else {
                unsafe { put.multiline(lnum, col, new_cursor) };
            }
        }

        // SAFETY: main thread, reporting how many lines went in.
        unsafe { msgmore(put.nr_lines) };
        cur_win().w_set_curswant = true;

        // Don't leave the cursor after the NUL.
        // SAFETY: the cursor is on a line of the current buffer.
        let len = get_cursor_line_len();
        if cur_win().w_cursor.col > len {
            if ve_flags == kOptVeFlagAll as c_uint {
                cur_win().w_cursor.coladd = cur_win().w_cursor.col - len;
            }
            cur_win().w_cursor.col = len;
        }
    }

    if cmdmod_has(CmdModFlags::LOCKMARKS) {
        cur_buf().b_op_start = orig_start;
        cur_buf().b_op_end = orig_end;
    }
    if allocated {
        // SAFETY: `allocated` is `get_spec_reg` saying the string is ours.
        unsafe { xfree(insert_string.data() as *mut c_void) };
    }
    if regname == '=' as c_int {
        // SAFETY: `split_expr_result` allocated the array above.
        unsafe { xfree(put.y_array as *mut c_void) };
    }

    if cur_buf().terminal.is_null() {
        set_visual_active(false);
    }

    // SAFETY: the cursor is on a line of the current buffer.
    unsafe { adjust_cursor_eol() };
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
