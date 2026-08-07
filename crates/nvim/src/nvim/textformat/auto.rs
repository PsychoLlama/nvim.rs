//! 'formatoptions' `a`: reformatting the paragraph as it is edited.
//!
//! [`auto_format`] runs after nearly every change in Insert mode, decides
//! whether the paragraph wants reflowing at all, and hands the work to
//! `format_lines`.  The space it may add under the cursor so that a
//! part-typed word still ends a paragraph is `did_add_space`, and
//! [`check_auto_format`] is what takes it away again.

use super::*;
use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::change::{del_char, get_leader_len};
use crate::src::nvim::cursor::{
    check_cursor, check_cursor_col, coladvance, dec_cursor, gchar_cursor, get_cursor_line_len,
    get_cursor_line_ptr, get_cursor_pos_ptr, inc_cursor,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{State, curbuf, curwin, saved_cursor};
use crate::src::nvim::mbyte::{utf_iscomposing_first, utf_ptr2char};
use crate::src::nvim::memline::ml_replace;
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::state::MODE_INSERT;
use crate::src::nvim::strings::xstrnsave;
use crate::src::nvim::types::{colnr_T, linenr_T, pos_T, size_t};
use crate::src::nvim::undo::u_save_cursor;

static did_add_space: GlobalCell<bool> = GlobalCell::new(false);
pub unsafe extern "C" fn auto_format(mut trailblank: bool, mut prev_line: bool) {
    if !has_format_option(FO_AUTO) {
        return;
    }
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    let mut old: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    check_auto_format(false);
    let mut wasatend: bool = pos.col == get_cursor_line_len();
    if *old as ::core::ffi::c_int != NUL && !trailblank && wasatend as ::core::ffi::c_int != 0 {
        dec_cursor();
        let mut cc: ::core::ffi::c_int = gchar_cursor();
        if !(ascii_iswhite(cc) as ::core::ffi::c_int != 0
            && !utf_iscomposing_first(utf_ptr2char(
                get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
            )))
            && (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
            && has_format_option(FO_ONE_LETTER) as ::core::ffi::c_int != 0
        {
            dec_cursor();
        }
        cc = gchar_cursor();
        if ascii_iswhite(cc) as ::core::ffi::c_int != 0
            && !utf_iscomposing_first(utf_ptr2char(
                get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
            ))
        {
            (*curwin.get()).w_cursor = pos;
            return;
        }
        (*curwin.get()).w_cursor = pos;
    }
    if *old as ::core::ffi::c_int != NUL
        && !trailblank
        && !wasatend
        && pos.col > 0 as ::core::ffi::c_int
        && State.get() & MODE_INSERT != 0
    {
        let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        if ascii_iswhite(
            *line.offset((pos.col as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int,
        ) as ::core::ffi::c_int
            != 0
            && !utf_iscomposing_first(utf_ptr2char(
                get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
            ))
        {
            (*curwin.get()).w_cursor = pos;
            return;
        }
    }
    if has_format_option(FO_WRAP_COMS) as ::core::ffi::c_int != 0
        && !has_format_option(FO_WRAP)
        && get_leader_len(
            old,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            false,
            true,
        ) == 0 as ::core::ffi::c_int
    {
        return;
    }
    if prev_line as ::core::ffi::c_int != 0 && !paragraph_start((*curwin.get()).w_cursor.lnum) {
        (*curwin.get()).w_cursor.lnum -= 1;
        if u_save_cursor() == FAIL {
            return;
        }
    }
    saved_cursor.set(pos);
    format_lines(-1 as linenr_T, false);
    (*curwin.get()).w_cursor = saved_cursor.get();
    (*saved_cursor.ptr()).lnum = 0 as ::core::ffi::c_int as linenr_T;
    if (*curwin.get()).w_cursor.lnum > (*curbuf.get()).b_ml.ml_line_count {
        (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
        coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
    } else {
        check_cursor_col(curwin.get());
    }
    if !wasatend && has_format_option(FO_WHITE_PAR) as ::core::ffi::c_int != 0 {
        let mut linep: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        let mut len: colnr_T = get_cursor_line_len();
        if (*curwin.get()).w_cursor.col == len {
            let mut plinep: *mut ::core::ffi::c_char =
                xstrnsave(linep, (len as size_t).wrapping_add(2 as size_t));
            *plinep.offset(len as isize) = ' ' as ::core::ffi::c_char;
            *plinep.offset((len as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                NUL as ::core::ffi::c_char;
            ml_replace((*curwin.get()).w_cursor.lnum, plinep, false);
            did_add_space.set(true);
        } else {
            check_auto_format(false);
        }
    }
    check_cursor(curwin.get());
}
pub unsafe extern "C" fn check_auto_format(mut end_insert: bool) {
    if !did_add_space.get() {
        return;
    }
    let mut cc: ::core::ffi::c_int = gchar_cursor();
    if !(ascii_iswhite(cc) as ::core::ffi::c_int != 0
        && !utf_iscomposing_first(utf_ptr2char(
            get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
        )))
    {
        did_add_space.set(false);
    } else {
        let mut c: ::core::ffi::c_int = ' ' as ::core::ffi::c_int;
        if !end_insert {
            inc_cursor();
            c = gchar_cursor();
            dec_cursor();
        }
        if c != NUL {
            del_char(false);
            did_add_space.set(false);
        }
    };
}
