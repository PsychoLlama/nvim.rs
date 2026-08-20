//! Walking the tree: `:undo`, `:redo`, `:earlier`/`:later`, and the
//! entry-swapping that actually changes the buffer.

use super::tree::*;
use super::*;
use crate::drawscreen::UPD_NOT_VALID;
use crate::edit::BeginlineOpts;
use crate::option::cpo_has;
use crate::pos::MAXLNUM;
use crate::{semsg_c, smsg_keep_c};

pub unsafe fn u_undo(mut count: c_int) {
    if !(*curbuf.get()).b_u_synced {
        u_sync(true);
        count = 1;
    }
    if !cpo_has(CpoFlag::UNDO) {
        undo_undoes.set(true);
    } else {
        undo_undoes.set(!undo_undoes.get());
    }
    u_doit(count, false, true);
}
pub unsafe fn u_redo(mut count: c_int) {
    if !cpo_has(CpoFlag::UNDO) {
        undo_undoes.set(false);
    }
    u_doit(count, false, true);
}
pub unsafe fn u_undo_and_forget(mut count: c_int, mut do_buf_event: bool) -> bool {
    if !(*curbuf.get()).b_u_synced {
        u_sync(true);
        count = 1;
    }
    undo_undoes.set(true);
    u_doit(count, true, do_buf_event);
    if (*curbuf.get()).b_u_curhead.is_null() {
        return false;
    }
    let mut to_forget: *mut u_header_T = (*curbuf.get()).b_u_curhead;
    (*curbuf.get()).b_u_newhead = (*to_forget).uh_next.ptr;
    (*curbuf.get()).b_u_curhead = (*to_forget).uh_alt_next.ptr;
    if !(*curbuf.get()).b_u_curhead.is_null() {
        (*to_forget).uh_alt_next.ptr = ptr::null_mut();
        (*(*curbuf.get()).b_u_curhead).uh_alt_prev.ptr = (*to_forget).uh_alt_prev.ptr;
        (*curbuf.get()).b_u_seq_cur = if !(*(*curbuf.get()).b_u_curhead).uh_next.ptr.is_null() {
            (*(*(*curbuf.get()).b_u_curhead).uh_next.ptr).uh_seq
        } else {
            0
        };
    } else if !(*curbuf.get()).b_u_newhead.is_null() {
        (*curbuf.get()).b_u_seq_cur = (*(*curbuf.get()).b_u_newhead).uh_seq;
    }
    if !(*to_forget).uh_alt_prev.ptr.is_null() {
        (*(*to_forget).uh_alt_prev.ptr).uh_alt_next.ptr = (*curbuf.get()).b_u_curhead;
    }
    if !(*curbuf.get()).b_u_newhead.is_null() {
        (*(*curbuf.get()).b_u_newhead).uh_prev.ptr = (*curbuf.get()).b_u_curhead;
    }
    if (*curbuf.get()).b_u_seq_last == (*to_forget).uh_seq {
        (*curbuf.get()).b_u_seq_last -= 1;
    }
    u_freebranch(curbuf.get(), to_forget, ptr::null_mut());
    true
}
pub(crate) unsafe fn u_doit(mut startcount: c_int, mut quiet: bool, mut do_buf_event: bool) {
    if !undo_allowed(curbuf.get()) {
        return;
    }
    u_newcount.set(0);
    u_oldcount.set(0);
    if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
        u_oldcount.set(-1);
    }
    msg_ext_set_kind(c"undo".as_ptr());
    let mut count: c_int = startcount;
    loop {
        let c2rust_fresh4 = count;
        count -= 1;
        if c2rust_fresh4 == 0 {
            break;
        }
        change_warning(curbuf.get(), 0);
        if undo_undoes.get() {
            if (*curbuf.get()).b_u_curhead.is_null() {
                (*curbuf.get()).b_u_curhead = (*curbuf.get()).b_u_newhead;
            } else if get_undolevel(curbuf.get()) > 0 {
                (*curbuf.get()).b_u_curhead = (*(*curbuf.get()).b_u_curhead).uh_next.ptr;
            }
            if (*curbuf.get()).b_u_numhead == 0 || (*curbuf.get()).b_u_curhead.is_null() {
                (*curbuf.get()).b_u_curhead = (*curbuf.get()).b_u_oldhead;
                beep_flush();
                if count == startcount - 1 {
                    msg(gettext(c"Already at oldest change".as_ptr()), 0);
                    return;
                }
                break;
            } else {
                u_undoredo(true, do_buf_event);
            }
        } else if (*curbuf.get()).b_u_curhead.is_null() || get_undolevel(curbuf.get()) <= 0 {
            beep_flush();
            if count == startcount - 1 {
                msg(gettext(c"Already at newest change".as_ptr()), 0);
                return;
            }
            break;
        } else {
            u_undoredo(false, do_buf_event);
            if (*(*curbuf.get()).b_u_curhead).uh_prev.ptr.is_null() {
                (*curbuf.get()).b_u_newhead = (*curbuf.get()).b_u_curhead;
            }
            (*curbuf.get()).b_u_curhead = (*(*curbuf.get()).b_u_curhead).uh_prev.ptr;
        }
    }
    u_undo_end(undo_undoes.get(), false, quiet);
}
pub unsafe fn undo_time(mut step: c_int, mut sec: bool, mut file: bool, mut absolute: bool) {
    if text_locked() {
        text_locked_msg();
        return;
    }
    if !(*curbuf.get()).b_u_synced {
        u_sync(true);
    }
    u_newcount.set(0);
    u_oldcount.set(0);
    if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
        u_oldcount.set(-1);
    }
    let mut target: c_int = 0;
    let mut closest: c_int = 0;
    let mut uhp: *mut u_header_T = ptr::null_mut();
    let mut dosec: bool = sec;
    let mut dofile: bool = file;
    let mut above: bool = false;
    let mut did_undo: bool = true;
    if absolute {
        target = step;
        closest = -1;
    } else {
        if dosec {
            target = ((*curbuf.get()).b_u_time_cur as c_int).wrapping_add(step);
        } else if dofile {
            if step < 0 {
                uhp = (*curbuf.get()).b_u_curhead;
                if !uhp.is_null() {
                    uhp = (*uhp).uh_next.ptr;
                } else {
                    uhp = (*curbuf.get()).b_u_newhead;
                }
                if !uhp.is_null() && (*uhp).uh_save_nr != 0 {
                    target = (*curbuf.get()).b_u_save_nr_cur.wrapping_add(step);
                } else {
                    target = (*curbuf.get())
                        .b_u_save_nr_cur
                        .wrapping_add(step)
                        .wrapping_add(1);
                }
                if target <= 0 {
                    dofile = false;
                }
            } else {
                target = (*curbuf.get()).b_u_save_nr_cur.wrapping_add(step);
                if target > (*curbuf.get()).b_u_save_nr_last {
                    target = (*curbuf.get()).b_u_seq_last + 1;
                    dofile = false;
                }
            }
        } else {
            // `step` is a user count, so this is `int` arithmetic that
            // wraps in the C: `:later 2147483647` runs past `INT_MAX` and
            // the clamp below is what catches it.
            target = (*curbuf.get()).b_u_seq_cur.wrapping_add(step);
        }
        if step < 0 {
            target = if target > 0 { target } else { 0 };
            closest = -1;
        } else {
            if dosec {
                closest = os_time().wrapping_add(1 as Timestamp) as c_int;
            } else if dofile {
                closest = (*curbuf.get()).b_u_save_nr_last + 2;
            } else {
                closest = (*curbuf.get()).b_u_seq_last + 2;
            }
            if target >= closest {
                target = closest - 1;
            }
        }
    }
    let mut closest_start: c_int = closest;
    let mut closest_seq: c_int = (*curbuf.get()).b_u_seq_cur;
    let mut mark: c_int = 0;
    let mut nomark: c_int = 0;
    if target == 0 {
        mark = lastmark.get();
    } else {
        let mut round: c_int = 1;
        while round <= 2 {
            (*lastmark.ptr()) += 1;
            mark = lastmark.get();
            (*lastmark.ptr()) += 1;
            nomark = lastmark.get();
            if (*curbuf.get()).b_u_curhead.is_null() {
                uhp = (*curbuf.get()).b_u_newhead;
            } else {
                uhp = (*curbuf.get()).b_u_curhead;
            }
            while !uhp.is_null() {
                (*uhp).uh_walk = mark;
                let mut val: c_int = if dosec {
                    (*uhp).uh_time as c_int
                } else if dofile {
                    (*uhp).uh_save_nr
                } else {
                    (*uhp).uh_seq
                };
                if round == 1
                    && !(dofile && val == 0)
                    && (if step < 0 {
                        ((*uhp).uh_seq <= (*curbuf.get()).b_u_seq_cur) as c_int
                    } else {
                        ((*uhp).uh_seq > (*curbuf.get()).b_u_seq_cur) as c_int
                    }) != 0
                    && (if dosec && val == closest {
                        if step < 0 {
                            ((*uhp).uh_seq < closest_seq) as c_int
                        } else {
                            ((*uhp).uh_seq > closest_seq) as c_int
                        }
                    } else {
                        (closest == closest_start
                            || (if val > target {
                                if closest > target {
                                    (val - target <= closest - target) as c_int
                                } else {
                                    (val - target <= target - closest) as c_int
                                }
                            } else {
                                if closest > target {
                                    (target - val <= closest - target) as c_int
                                } else {
                                    (target - val <= target - closest) as c_int
                                }
                            }) != 0) as c_int
                    }) != 0
                {
                    closest = val;
                    closest_seq = (*uhp).uh_seq;
                }
                if target == val && !dosec {
                    target = (*uhp).uh_seq;
                    break;
                } else if !(*uhp).uh_prev.ptr.is_null()
                    && (*(*uhp).uh_prev.ptr).uh_walk != nomark
                    && (*(*uhp).uh_prev.ptr).uh_walk != mark
                {
                    uhp = (*uhp).uh_prev.ptr;
                } else if !(*uhp).uh_alt_next.ptr.is_null()
                    && (*(*uhp).uh_alt_next.ptr).uh_walk != nomark
                    && (*(*uhp).uh_alt_next.ptr).uh_walk != mark
                {
                    uhp = (*uhp).uh_alt_next.ptr;
                } else if !(*uhp).uh_next.ptr.is_null()
                    && (*uhp).uh_alt_prev.ptr.is_null()
                    && (*(*uhp).uh_next.ptr).uh_walk != nomark
                    && (*(*uhp).uh_next.ptr).uh_walk != mark
                {
                    if uhp == (*curbuf.get()).b_u_curhead {
                        (*uhp).uh_walk = nomark;
                    }
                    uhp = (*uhp).uh_next.ptr;
                } else {
                    (*uhp).uh_walk = nomark;
                    if !(*uhp).uh_alt_prev.ptr.is_null() {
                        uhp = (*uhp).uh_alt_prev.ptr;
                    } else {
                        uhp = (*uhp).uh_next.ptr;
                    }
                }
            }
            if !uhp.is_null() {
                break;
            }
            if absolute {
                semsg_c!(
                    gettext(c"E830: Undo number %ld not found".as_ptr()),
                    step as int64_t,
                );
                return;
            }
            if closest == closest_start {
                if step < 0 {
                    msg(gettext(c"Already at oldest change".as_ptr()), 0);
                } else {
                    msg(gettext(c"Already at newest change".as_ptr()), 0);
                }
                return;
            }
            target = closest_seq;
            dosec = false;
            dofile = false;
            if step < 0 {
                above = true;
            }
            round += 1;
        }
    }
    if !uhp.is_null() || target == 0 {
        while !got_int.get() {
            change_warning(curbuf.get(), 0);
            uhp = (*curbuf.get()).b_u_curhead;
            if uhp.is_null() {
                uhp = (*curbuf.get()).b_u_newhead;
            } else {
                uhp = (*uhp).uh_next.ptr;
            }
            if uhp.is_null()
                || target > 0 && (*uhp).uh_walk != mark
                || (*uhp).uh_seq == target && !above
            {
                break;
            }
            (*curbuf.get()).b_u_curhead = uhp;
            u_undoredo(true, true);
            if target > 0 {
                (*uhp).uh_walk = nomark;
            }
        }
        if target > 0 {
            while !got_int.get() {
                change_warning(curbuf.get(), 0);
                uhp = (*curbuf.get()).b_u_curhead;
                if uhp.is_null() {
                    break;
                }
                while !(*uhp).uh_alt_prev.ptr.is_null() && (*(*uhp).uh_alt_prev.ptr).uh_walk == mark
                {
                    uhp = (*uhp).uh_alt_prev.ptr;
                }
                let mut last: *mut u_header_T = uhp;
                while !(*last).uh_alt_next.ptr.is_null()
                    && (*(*last).uh_alt_next.ptr).uh_walk == mark
                {
                    last = (*last).uh_alt_next.ptr;
                }
                if last != uhp {
                    while !(*uhp).uh_alt_prev.ptr.is_null() {
                        uhp = (*uhp).uh_alt_prev.ptr;
                    }
                    if !(*last).uh_alt_next.ptr.is_null() {
                        (*(*last).uh_alt_next.ptr).uh_alt_prev.ptr = (*last).uh_alt_prev.ptr;
                    }
                    (*(*last).uh_alt_prev.ptr).uh_alt_next.ptr = (*last).uh_alt_next.ptr;
                    (*last).uh_alt_prev.ptr = ptr::null_mut();
                    (*last).uh_alt_next.ptr = uhp;
                    (*uhp).uh_alt_prev.ptr = last;
                    if (*curbuf.get()).b_u_oldhead == uhp {
                        (*curbuf.get()).b_u_oldhead = last;
                    }
                    uhp = last;
                    if !(*uhp).uh_next.ptr.is_null() {
                        (*(*uhp).uh_next.ptr).uh_prev.ptr = uhp;
                    }
                }
                (*curbuf.get()).b_u_curhead = uhp;
                if (*uhp).uh_walk != mark {
                    break;
                }
                if (*uhp).uh_seq == target && above {
                    (*curbuf.get()).b_u_seq_cur = target - 1;
                    break;
                } else {
                    u_undoredo(false, true);
                    if (*uhp).uh_prev.ptr.is_null() {
                        (*curbuf.get()).b_u_newhead = uhp;
                    }
                    (*curbuf.get()).b_u_curhead = (*uhp).uh_prev.ptr;
                    did_undo = false;
                    if (*uhp).uh_seq == target {
                        break;
                    }
                    uhp = (*uhp).uh_prev.ptr;
                    if !(uhp.is_null() || (*uhp).uh_walk != mark) {
                        continue;
                    }
                    internal_error(c"undo_time()".as_ptr());
                    break;
                }
            }
        }
    }
    u_undo_end(did_undo, absolute, false);
}
pub(crate) unsafe fn u_undoredo(mut undo: bool, mut do_buf_event: bool) {
    let mut newarray: *mut *mut c_char = ptr::null_mut();
    let mut newlnum: linenr_T = MAXLNUM as c_int as linenr_T;
    let mut new_curpos: pos_T = (*curwin.get()).w_cursor;
    let mut nuep: *mut u_entry_T = ptr::null_mut();
    let mut newlist: *mut u_entry_T = ptr::null_mut();
    let mut namedm: [fmark_T; 26] = [fmark_T {
        mark: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        fnum: 0,
        timestamp: 0,
        view: fmarkv_T {
            topline_offset: 0,
            skipcol: 0,
        },
        additional_data: ptr::null_mut(),
    }; 26];
    let mut curhead: *mut u_header_T = (*curbuf.get()).b_u_curhead;
    block_autocmds();
    let mut old_flags: c_int = (*curhead).uh_flags;
    let mut new_flags: c_int = (if (*curbuf.get()).b_changed != 0 {
        UH_CHANGED as c_int
    } else {
        0
    }) | (if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
        UH_EMPTYBUF as c_int
    } else {
        0
    }) | old_flags & UH_RELOAD as c_int;
    setpcmark();
    zero_fmark_additional_data(&raw mut (*curbuf.get()).b_namedm as *mut fmark_T);
    memmove(
        &raw mut namedm as *mut fmark_T as *mut c_void,
        &raw mut (*curbuf.get()).b_namedm as *mut fmark_T as *const c_void,
        size_of::<fmark_T>().wrapping_mul(NMARKS as size_t),
    );
    let mut visualinfo: visualinfo_T = (*curbuf.get()).b_visual;
    (*curbuf.get()).b_op_start.lnum = (*curbuf.get()).b_ml.ml_line_count;
    (*curbuf.get()).b_op_start.col = 0;
    (*curbuf.get()).b_op_end.lnum = 0;
    (*curbuf.get()).b_op_end.col = 0;
    let mut uep: *mut u_entry_T = (*curhead).uh_entry;
    while !uep.is_null() {
        let mut top: linenr_T = (*uep).ue_top;
        let mut bot: linenr_T = (*uep).ue_bot;
        if bot == 0 {
            bot = (*curbuf.get()).b_ml.ml_line_count + 1;
        }
        if top > (*curbuf.get()).b_ml.ml_line_count
            || top >= bot
            || bot > (*curbuf.get()).b_ml.ml_line_count + 1
        {
            unblock_autocmds();
            iemsg(gettext(c"E438: u_undo: line numbers wrong".as_ptr()));
            changed(curbuf.get());
            return;
        }
        let mut oldsize: linenr_T = bot - top - 1;
        let mut newsize: linenr_T = (*uep).ue_size;
        let mut lnum: linenr_T = (*curhead).uh_cursor.lnum;
        if lnum >= top && lnum <= top + newsize + 1 {
            new_curpos = (*curhead).uh_cursor;
            newlnum = -1;
        } else if top < newlnum {
            let mut i: c_int = 0;
            i = 0;
            while (i as linenr_T) < newsize && (i as linenr_T) < oldsize {
                if strcmp(
                    *(*uep).ue_array.offset(i as isize),
                    ml_get(top + 1 + i as linenr_T),
                ) != 0
                {
                    break;
                }
                i += 1;
            }
            if i as linenr_T == newsize
                && newlnum == MAXLNUM as c_int as linenr_T
                && (*uep).ue_next.is_null()
            {
                newlnum = top;
                new_curpos.lnum = newlnum + 1;
            } else if (i as linenr_T) < newsize {
                newlnum = top + i as linenr_T;
                new_curpos.lnum = newlnum + 1;
            }
        }
        let mut empty_buffer: bool = false;
        if oldsize > 0 {
            newarray = xmalloc(size_of::<*mut c_char>().wrapping_mul(oldsize as size_t))
                as *mut *mut c_char;
            let mut i_0: c_int = 0;
            let mut lnum_0: linenr_T = 0;
            lnum_0 = bot - 1;
            i_0 = oldsize as c_int;
            loop {
                i_0 -= 1;
                if i_0 < 0 {
                    break;
                }
                *newarray.offset(i_0 as isize) = u_save_line(lnum_0);
                if (*curbuf.get()).b_ml.ml_line_count == 1 {
                    empty_buffer = true;
                }
                ml_delete(lnum_0);
                lnum_0 -= 1;
            }
        } else {
            newarray = ptr::null_mut();
        }
        check_cursor_lnum(curwin.get());
        if newsize != 0 {
            let mut i_1: c_int = 0;
            let mut lnum_1: linenr_T = 0;
            lnum_1 = top;
            i_1 = 0;
            while (i_1 as linenr_T) < newsize {
                if empty_buffer && lnum_1 == 0 {
                    ml_replace(1, *(*uep).ue_array.offset(i_1 as isize), true);
                } else {
                    ml_append_flags(lnum_1, *(*uep).ue_array.offset(i_1 as isize), 0, 0);
                }
                xfree(*(*uep).ue_array.offset(i_1 as isize) as *mut c_void);
                i_1 += 1;
                lnum_1 += 1;
            }
            xfree((*uep).ue_array as *mut c_void);
        }
        if oldsize != newsize {
            mark_adjust(
                top + 1,
                top + oldsize,
                MAXLNUM as c_int as linenr_T,
                newsize - oldsize,
                kExtmarkNOOP,
            );
            if (*curbuf.get()).b_op_start.lnum > top + oldsize {
                (*curbuf.get()).b_op_start.lnum += newsize - oldsize;
            }
            if (*curbuf.get()).b_op_end.lnum > top + oldsize {
                (*curbuf.get()).b_op_end.lnum += newsize - oldsize;
            }
        }
        if oldsize > 0 || newsize > 0 {
            changed_lines(
                curbuf.get(),
                top + 1,
                0,
                bot,
                newsize - oldsize,
                do_buf_event,
            );
            if spell_check_window(curwin.get()) && bot <= (*curbuf.get()).b_ml.ml_line_count {
                redraw_win_line(curwin.get(), bot);
            }
        }
        (*curbuf.get()).b_op_start.lnum = if (*curbuf.get()).b_op_start.lnum < top + 1 {
            (*curbuf.get()).b_op_start.lnum
        } else {
            top + 1
        };
        if newsize == 0 && top + 1 > (*curbuf.get()).b_op_end.lnum {
            (*curbuf.get()).b_op_end.lnum = top + 1;
        } else if top + newsize > (*curbuf.get()).b_op_end.lnum {
            (*curbuf.get()).b_op_end.lnum = top + newsize;
        }
        (*u_newcount.ptr()) += newsize as c_int;
        (*u_oldcount.ptr()) += oldsize as c_int;
        (*uep).ue_size = oldsize;
        (*uep).ue_array = newarray;
        (*uep).ue_bot = top + newsize + 1;
        nuep = (*uep).ue_next;
        (*uep).ue_next = newlist;
        newlist = uep;
        uep = nuep;
    }
    (*curbuf.get()).b_op_start.lnum =
        if (*curbuf.get()).b_op_start.lnum < (*curbuf.get()).b_ml.ml_line_count {
            (*curbuf.get()).b_op_start.lnum
        } else {
            (*curbuf.get()).b_ml.ml_line_count
        };
    (*curbuf.get()).b_op_end.lnum =
        if (*curbuf.get()).b_op_end.lnum < (*curbuf.get()).b_ml.ml_line_count {
            (*curbuf.get()).b_op_end.lnum
        } else {
            (*curbuf.get()).b_ml.ml_line_count
        };
    if undo {
        let mut i_2: c_int = (*curhead).uh_extmark.size as c_int - 1;
        while i_2 > -1 {
            extmark_apply_undo(*(*curhead).uh_extmark.items.offset(i_2 as isize), undo);
            i_2 -= 1;
        }
    } else {
        let mut i_3: c_int = 0;
        while i_3 < (*curhead).uh_extmark.size as c_int {
            extmark_apply_undo(*(*curhead).uh_extmark.items.offset(i_3 as isize), undo);
            i_3 += 1;
        }
    }
    if (*curhead).uh_flags & UH_RELOAD != 0 {
        buf_updates_unload(curbuf.get(), true);
    }
    (*curwin.get()).w_cursor = new_curpos;
    check_cursor_lnum(curwin.get());
    (*curhead).uh_entry = newlist;
    (*curhead).uh_flags = new_flags;
    if old_flags & UH_EMPTYBUF != 0 && buf_is_empty(curbuf.get()) {
        (*curbuf.get()).b_ml.ml_flags |= ML_EMPTY;
    }
    if old_flags & UH_CHANGED != 0 {
        changed(curbuf.get());
    } else {
        unchanged(curbuf.get(), false, true);
    }
    if do_buf_event {
        buf_updates_changedtick(curbuf.get());
    }
    let mut i_4: c_int = 0;
    while i_4 < NMARKS {
        if (*curhead).uh_namedm[i_4 as usize].mark.lnum != 0 {
            free_fmark((*curbuf.get()).b_namedm[i_4 as usize]);
            (*curbuf.get()).b_namedm[i_4 as usize] = (*curhead).uh_namedm[i_4 as usize];
        }
        if namedm[i_4 as usize].mark.lnum != 0 {
            (*curhead).uh_namedm[i_4 as usize] = namedm[i_4 as usize];
        } else {
            (*curhead).uh_namedm[i_4 as usize].mark.lnum = 0;
        }
        i_4 += 1;
    }
    if (*curhead).uh_visual.vi_start.lnum != 0 {
        (*curbuf.get()).b_visual = (*curhead).uh_visual;
        (*curhead).uh_visual = visualinfo;
    }
    if (*curhead).uh_cursor.lnum + 1 == (*curwin.get()).w_cursor.lnum
        && (*curwin.get()).w_cursor.lnum > 1
    {
        (*curwin.get()).w_cursor.lnum -= 1;
    }
    if (*curwin.get()).w_cursor.lnum <= (*curbuf.get()).b_ml.ml_line_count {
        if (*curhead).uh_cursor.lnum == (*curwin.get()).w_cursor.lnum {
            (*curwin.get()).w_cursor.col = (*curhead).uh_cursor.col;
            if virtual_active(curwin.get()) && (*curhead).uh_cursor_vcol >= 0 {
                coladvance(curwin.get(), (*curhead).uh_cursor_vcol);
            } else {
                (*curwin.get()).w_cursor.coladd = 0;
            }
        } else {
            beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
        }
    } else {
        (*curwin.get()).w_cursor.col = 0;
        (*curwin.get()).w_cursor.coladd = 0;
    }
    check_cursor(curwin.get());
    (*curbuf.get()).b_u_seq_cur = (*curhead).uh_seq;
    if undo {
        (*curbuf.get()).b_u_seq_cur = if !(*curhead).uh_next.ptr.is_null() {
            (*(*curhead).uh_next.ptr).uh_seq
        } else {
            0
        };
    }
    if (*curhead).uh_save_nr != 0 {
        if undo {
            (*curbuf.get()).b_u_save_nr_cur = (*curhead).uh_save_nr - 1;
        } else {
            (*curbuf.get()).b_u_save_nr_cur = (*curhead).uh_save_nr;
        }
    }
    (*curbuf.get()).b_u_time_cur = (*curhead).uh_time;
    unblock_autocmds();
}
pub(crate) unsafe fn u_undo_end(mut did_undo: bool, mut absolute: bool, mut quiet: bool) {
    if fdo_flags.get() & kOptFdoFlagUndo as c_int as c_uint != 0 && KeyTyped.get() {
        foldOpenCursor();
    }
    if quiet || global_busy.get() != 0 || !messaging() {
        return;
    }
    if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
        (*u_newcount.ptr()) -= 1;
    }
    (*u_oldcount.ptr()) -= u_newcount.get();
    let mut msgstr: *mut c_char = ptr::null_mut();
    if u_oldcount.get() == -1 {
        msgstr = c"more line".as_ptr() as *mut c_char;
    } else if u_oldcount.get() < 0 {
        msgstr = c"more lines".as_ptr() as *mut c_char;
    } else if u_oldcount.get() == 1 {
        msgstr = c"line less".as_ptr() as *mut c_char;
    } else if u_oldcount.get() > 1 {
        msgstr = c"fewer lines".as_ptr() as *mut c_char;
    } else {
        u_oldcount.set(u_newcount.get());
        if u_newcount.get() == 1 {
            msgstr = c"change".as_ptr() as *mut c_char;
        } else {
            msgstr = c"changes".as_ptr() as *mut c_char;
        }
    }
    let mut uhp: *mut u_header_T = ptr::null_mut();
    if !(*curbuf.get()).b_u_curhead.is_null() {
        if absolute && !(*(*curbuf.get()).b_u_curhead).uh_next.ptr.is_null() {
            uhp = (*(*curbuf.get()).b_u_curhead).uh_next.ptr;
            did_undo = false;
        } else if did_undo {
            uhp = (*curbuf.get()).b_u_curhead;
        } else {
            uhp = (*(*curbuf.get()).b_u_curhead).uh_next.ptr;
        }
    } else {
        uhp = (*curbuf.get()).b_u_newhead;
    }
    let mut msgbuf: [c_char; 80] = [0; 80];
    if uhp.is_null() {
        *(&raw mut msgbuf as *mut c_char) = NUL as c_char;
    } else {
        undo_fmt_time(
            &raw mut msgbuf as *mut c_char,
            size_of::<[c_char; 80]>(),
            (*uhp).uh_time,
        );
    }
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer == curbuf.get() && (*wp).w_onebuf_opt.wo_cole > 0 {
            redraw_later(wp, UPD_NOT_VALID);
        }
        wp = (*wp).w_next;
    }
    if VIsual_active.get() {
        check_pos(curbuf.get(), VIsual.ptr());
    }
    smsg_keep_c!(
        0,
        gettext(c"%ld %s; %s #%ld  %s".as_ptr()),
        if u_oldcount.get() < 0 {
            -u_oldcount.get() as int64_t
        } else {
            u_oldcount.get() as int64_t
        },
        gettext(msgstr),
        if did_undo {
            gettext(c"before".as_ptr())
        } else {
            gettext(c"after".as_ptr())
        },
        if uhp.is_null() {
            0 as int64_t
        } else {
            (*uhp).uh_seq as int64_t
        },
        &raw mut msgbuf as *mut c_char,
    );
}
