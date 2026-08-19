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
use core::ffi::{c_char, c_int, c_uint, c_void};

use super::*;
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
    unsafe {
        let non_linewise_vis = VIsual_active.get() && VIsual_mode.get() != 'V' as c_int;

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
            do_put(
                '_' as c_int,
                ::core::ptr::null_mut(),
                dir,
                1,
                PUT_LINE as c_int,
            );

            stuffcharReadbuff(command_start_char);
            while count > 0 {
                stuff_inserted(NUL, 1, (count != 1) as c_int);
                if count != 1 {
                    // `<CR>` then CTRL-U, to take off the indent 'autoindent'
                    // would add. CTRL-U on its own would go back to the
                    // previous line under 'nobackspace'-`eol`, so it is given
                    // a space to consume.
                    stuffReadbuff(c"\n ".as_ptr());
                    stuffcharReadbuff(Ctrl_U);
                }
                count -= 1;
            }
        } else {
            stuff_inserted(command_start_char, count, false as c_int);
        }

        // The text goes in later, so the cursor cannot be moved past it here;
        // motion commands stuffed after the insert do it instead.
        if flags & PUT_CURSEND as c_int != 0 {
            if flags & PUT_LINE as c_int != 0 {
                stuffReadbuff(c"j0".as_ptr());
            } else {
                // Stuffing `l` would ring the bell at the end of a line, so
                // only do it when the cursor can actually move right:
                // 'virtualedit' allows it, or the cursor is neither at the
                // end of the line nor one past the end of the last line. The
                // last case is a Visual put over a selection reaching past
                // the end of the line, which joins the line below.
                let cursor_pos = get_cursor_pos_ptr();
                let one_past_line = c_int::from(*cursor_pos) == NUL;
                let eol = !one_past_line
                    && c_int::from(*cursor_pos.offset(utfc_ptr2len(cursor_pos) as isize)) == NUL;
                let ve_allows =
                    ve_flags == kOptVeFlagAll as c_uint || ve_flags == kOptVeFlagOnemore as c_uint;
                let eof = (*curbuf.get()).b_ml.ml_line_count == (*curwin.get()).w_cursor.lnum
                    && one_past_line;
                if ve_allows || !(eol || eof) {
                    stuffcharReadbuff('l' as c_int);
                }
            }
        } else if flags & PUT_LINE as c_int != 0 {
            stuffReadbuff(c"g'[".as_ptr());
        }

        // Save the cursor position now (though no text), so that `u` after
        // `".p` restores it.
        if command_start_char == 'a' as c_int {
            u_save(
                (*curwin.get()).w_cursor.lnum,
                (*curwin.get()).w_cursor.lnum + 1,
            );
        }
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
    unsafe {
        // Two passes over the same walk: the first counts the lines, the
        // second fills the array the count sized.
        let mut y_array: *mut String_0 = ::core::ptr::null_mut();
        let mut y_type = kMTCharWise;
        loop {
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
            if !y_array.is_null() {
                return (y_array, y_size, y_type);
            }
            y_array =
                xmalloc(y_size.wrapping_mul(::core::mem::size_of::<String_0>())) as *mut String_0;
        }
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
        unsafe {
            if u_save_cursor() == FAIL {
                return false;
            }
            let curline = get_cursor_line_ptr();
            let p_orig = get_cursor_pos_ptr();
            let plen = get_cursor_pos_len() as size_t;
            let mut p = p_orig;
            if self.dir == FORWARD && c_int::from(*p) != NUL {
                p = p.offset(utfc_ptr2len(p) as isize);
            }
            // Kept for the extmark_splice() the multiline put emits.
            self.split_pos = p.offset_from(curline) as colnr_T;

            let ptr = xmemdupz(
                p as *const c_void,
                plen.wrapping_sub(p.offset_from(p_orig) as size_t),
            ) as *mut c_char;
            ml_append((*curwin.get()).w_cursor.lnum, ptr, 0, false);
            xfree(ptr as *mut c_void);

            let ptr = xmemdupz(
                get_cursor_line_ptr() as *const c_void,
                self.split_pos as size_t,
            ) as *mut c_char;
            ml_replace((*curwin.get()).w_cursor.lnum, ptr, false);
            self.nr_lines += 1;
            self.dir = FORWARD;

            buf_updates_send_changes(curbuf.get(), (*curwin.get()).w_cursor.lnum, 1, 1);
            true
        }
    }

    /// Save for undo and put the cursor where the text goes.
    ///
    /// Answers false when undo could not be saved.
    ///
    /// # Safety
    /// The cursor must be on a valid line.
    unsafe fn save_for_undo(&self) -> bool {
        unsafe {
            if self.y_type == kMTBlockWise {
                let mut lnum = (*curwin.get()).w_cursor.lnum + self.y_size as linenr_T + 1;
                lnum = lnum.min((*curbuf.get()).b_ml.ml_line_count + 1);
                return u_save((*curwin.get()).w_cursor.lnum - 1, lnum) != FAIL;
            }

            if self.y_type != kMTLineWise {
                return u_save_cursor() != FAIL;
            }

            let mut lnum = (*curwin.get()).w_cursor.lnum;
            // Correct for a closed fold. The cursor must not move yet:
            // u_save() reads it.
            if self.dir == BACKWARD {
                hasFolding(curwin.get(), lnum, &raw mut lnum, ::core::ptr::null_mut());
            } else {
                hasFolding(curwin.get(), lnum, ::core::ptr::null_mut(), &raw mut lnum);
            }
            if self.dir == FORWARD {
                lnum += 1;
            }
            // An empty buffer's one empty line is going to be replaced, so it
            // has to be part of what is saved.
            let saved = if buf_is_empty(curbuf.get()) {
                u_save(0, 2)
            } else {
                u_save(lnum - 1, lnum)
            };
            if saved == FAIL {
                return false;
            }
            (*curwin.get()).w_cursor.lnum = if self.dir == FORWARD { lnum - 1 } else { lnum };
            (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor; // for mark_adjust()
            true
        }
    }

    /// With 'virtualedit' "all", make the cursor a real position before the
    /// text goes in: break a tab into spaces, or pad out past the end of the
    /// line.
    ///
    /// # Safety
    /// The cursor must be on a valid line.
    unsafe fn make_room_for_virtualedit(&self) {
        unsafe {
            if self.ve_flags != kOptVeFlagAll as c_uint || self.y_type != kMTCharWise {
                return;
            }
            if gchar_cursor() == TAB {
                let viscol = getviscol();
                let ts = (*curbuf.get()).b_p_ts;
                // No spaces needed for `p` on the last position of a tab, or
                // `P` on the first.
                let splits_tab = if self.dir == FORWARD {
                    tabstop_padding(viscol, ts, (*curbuf.get()).b_p_vts_array) != 1
                } else {
                    (*curwin.get()).w_cursor.coladd > 0
                };
                if splits_tab {
                    coladvance_force(viscol);
                } else {
                    (*curwin.get()).w_cursor.coladd = 0;
                }
            } else if (*curwin.get()).w_cursor.coladd > 0 || gchar_cursor() == NUL {
                coladvance_force(getviscol() + c_int::from(self.dir == FORWARD));
            }
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
    unsafe {
        let orig_start = (*curbuf.get()).b_op_start;
        let orig_end = (*curbuf.get()).b_op_end;
        let ve_flags = get_ve_flags(curwin.get());

        // Remove any preinserted completion text (vim/vim#19329).
        if ins_compl_preinsert_effect() {
            ins_compl_delete(false);
        }

        // Defaults for the `'[` and `']` marks.
        (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
        (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;

        if regname == '.' as c_int && reg.is_null() {
            put_last_insert(dir, count, flags, ve_flags);
            return;
        }

        // A computed register becomes a fake one-line yankreg.
        let mut insert_string = String_0::NULL;
        let mut allocated = false;
        if reg.is_null()
            && get_spec_reg(regname, insert_string.data_mut(), &raw mut allocated, true)
            && insert_string.data().is_null()
        {
            return;
        }

        if (*curbuf.get()).terminal.is_null() {
            // Saving for undo can run autocommands, which would invalidate
            // `y_array`, so it happens before the register is read.
            if u_save(
                (*curwin.get()).w_cursor.lnum,
                (*curwin.get()).w_cursor.lnum + 1,
            ) == FAIL
            {
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
            insert_string.set_len(strlen(insert_string.data()));
            if regname == '=' as c_int {
                // Only `"=` can produce more than one line.
                (put.y_array, put.y_size, put.y_type) = split_expr_result(insert_string);
            } else {
                put.y_size = 1;
                put.y_array = &raw mut insert_string;
            }
        } else {
            // Visual-mode replace may have handed us the register already, so
            // that the deleted text is not what comes back.
            let reg = if reg.is_null() {
                get_yank_register(regname, YREG_PASTE)
            } else {
                reg
            };
            put.y_type = (*reg).y_type;
            put.y_width = (*reg).y_width;
            put.y_size = (*reg).y_size;
            put.y_array = (*reg).y_array;
        }

        'end: {
            if !(*curbuf.get()).terminal.is_null() {
                terminal_paste(count, put.y_array, put.y_size);
                break 'end;
            }

            if put.y_type == kMTLineWise {
                if put.flags & PUT_LINE_SPLIT as c_int != 0 && !put.split_current_line() {
                    break 'end;
                }
                if put.flags & PUT_LINE_FORWARD as c_int != 0 {
                    // `p` over a Visual block puts the lines below the block.
                    (*curwin.get()).w_cursor = (*curbuf.get()).b_visual.vi_end;
                    put.dir = FORWARD;
                }
                (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
                (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
            }

            if put.flags & PUT_LINE as c_int != 0 {
                // `:put`, or `p` in Visual line mode.
                put.y_type = kMTLineWise;
            }

            if put.y_size == 0 || put.y_array.is_null() {
                semsg_c!(
                    gettext(c"E353: Nothing in register %s".as_ptr()),
                    if regname == 0 {
                        c"\"".as_ptr()
                    } else {
                        transchar(regname) as *const c_char
                    },
                );
                break 'end;
            }

            if !put.save_for_undo() {
                break 'end;
            }
            put.make_room_for_virtualedit();

            let mut lnum = (*curwin.get()).w_cursor.lnum;
            let mut col = (*curwin.get()).w_cursor.col;

            if put.y_type == kMTBlockWise {
                put.blockwise(lnum);
            } else {
                if put.y_type == kMTCharWise {
                    // For charwise text, FORWARD is BACKWARD on the next
                    // character.
                    if put.dir == FORWARD && gchar_cursor() != NUL {
                        let bytelen = utfc_ptr2len(get_cursor_pos_ptr());
                        col += bytelen;
                        if (*put.y_array).len() != 0 {
                            (*curwin.get()).w_cursor.col += bytelen;
                            (*curbuf.get()).b_op_end.col += bytelen;
                        }
                    }
                    (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
                } else if put.dir == BACKWARD {
                    // Linewise: BACKWARD is FORWARD on the previous line.
                    lnum -= 1;
                }
                let new_cursor = (*curwin.get()).w_cursor;

                if put.y_type == kMTCharWise && put.y_size == 1 {
                    put.charwise_one_line(lnum, col);
                } else {
                    put.multiline(lnum, col, new_cursor);
                }
            }

            msgmore(put.nr_lines);
            (*curwin.get()).w_set_curswant = true as c_int;

            // Don't leave the cursor after the NUL.
            let len = get_cursor_line_len();
            if (*curwin.get()).w_cursor.col > len {
                if ve_flags == kOptVeFlagAll as c_uint {
                    (*curwin.get()).w_cursor.coladd = (*curwin.get()).w_cursor.col - len;
                }
                (*curwin.get()).w_cursor.col = len;
            }
        }

        if cmdmod_has(CmdModFlags::LOCKMARKS) {
            (*curbuf.get()).b_op_start = orig_start;
            (*curbuf.get()).b_op_end = orig_end;
        }
        if allocated {
            xfree(insert_string.data() as *mut c_void);
        }
        if regname == '=' as c_int {
            xfree(put.y_array as *mut c_void);
        }

        if (*curbuf.get()).terminal.is_null() {
            VIsual_active.set(false);
        }

        adjust_cursor_eol();
    }
}
