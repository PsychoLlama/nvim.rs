//! Cursor motions that are not searches: by character, word, line,
//! screen line, paragraph and sentence.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ptr;

use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::buffer::{bt_prompt, bt_quickfix};
use crate::src::nvim::charset::{vim_isprintc, vim_strsize};
use crate::src::nvim::cursor::{coladvance, gchar_cursor, get_cursor_pos_ptr};
use crate::src::nvim::decoration::{decor_conceal_line, win_lines_concealed};
use crate::src::nvim::edit::{
    beginline, cursor_down, cursor_down_inner, cursor_up, cursor_up_inner, oneleft, oneright,
};
use crate::src::nvim::eval::prompt_invoke_callback;
use crate::src::nvim::fold::hasFolding;
use crate::src::nvim::getchar::beep_flush;
use crate::src::nvim::main::{
    VIsual_active, VIsual_mode, VIsual_select_exclu_adj, cmdwin_result, cmdwin_type, curbuf,
    curwin, ins_at_eol, mod_mask, p_cpo, p_sel, p_ww, restart_edit,
};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::mbyte::{mb_adjust_cursor, utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memline::ml_get;
use crate::src::nvim::normal::{
    BL_FIX, BL_SOL, BL_WHITE, CA_NO_ADJ_OP_END, CAR, CPO_CHANGEW, FAIL, MOD_MASK_CTRL,
    MOD_MASK_SHIFT, NUL, TAB, adjust_for_sel, clearopbeep, false_0, kMTCharWise, kMTLineWise,
    may_fold_open, nv_page, true_0, unadjust_for_sel,
};
use crate::src::nvim::option::{get_showbreak_value, get_ve_flags};
use crate::src::nvim::options::{
    kOptFdoFlagBlock, kOptFdoFlagHor, kOptFdoFlagJump, kOptFdoFlagPercent, kOptVeFlagOnemore,
};
use crate::src::nvim::plines::{getvcol, linetabsize, plines_win, win_get_fill};
use crate::src::nvim::pos::{MAXCOL, lt};
use crate::src::nvim::quickfix::qf_view_result;
use crate::src::nvim::search::{BACKWARD, FORWARD, findmatch, searchc};
use crate::src::nvim::state::virtual_active;
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::textobject::{bck_word, end_word, findpar, findsent, fwd_word};
use crate::src::nvim::types::{
    Direction, OP_CHANGE, OP_DELETE, OP_NOP, cmdarg_T, colnr_T, linenr_T, oparg_T,
};
use core::ffi::{c_int, c_uint};

use crate::src::nvim::keycodes::{Ctrl_H, K_BS, K_LEFT, K_RIGHT};
use crate::src::nvim::r#move::{
    adjust_skipcol, cursor_correct, validate_botline_win, validate_virtcol, win_col_off,
    win_col_off2,
};

/// Move `dist` *screen* lines, which is what `gj`/`gk` and `g$` past the first
/// count mean, and what the arrow keys mean under 'wrap'.
///
/// The wanted column ('curswant') is carried in screen columns for the whole
/// walk and only turned back into a buffer column at the end, which is how a
/// long wrapped line and a short one keep the same apparent column. A closed
/// fold counts as one screen line however tall its text is, so the walk steps
/// over it rather than through it.
pub unsafe fn nv_screengo(
    oap: *mut oparg_T,
    dir: c_int,
    mut dist: c_int,
    skip_conceal: bool,
) -> bool {
    // SAFETY: `oap` is the caller's live operator.
    unsafe {
        let win = curwin.get();
        let mut linelen = linetabsize(win, (*win).w_cursor.lnum);
        let mut retval = true;
        // `$` asked for the end of the line, which has to be recomputed on
        // every row rather than carried as a column.
        let mut atend = false;
        (*oap).motion_type = kMTCharWise;
        (*oap).inclusive = (*win).w_curswant == MAXCOL as c_int;

        // The first screen row of a line can be narrower than the rest: only
        // it carries the number column and the signs.
        let col_off1 = win_col_off(win);
        let col_off2 = col_off1 - win_col_off2(win);
        let width1 = (*win).w_view_width - col_off1;
        let mut width2 = (*win).w_view_width - col_off2;
        if width2 == 0 {
            width2 = 1;
        }

        /// The last screen column of the line, given how many rows it takes.
        macro_rules! line_end {
            () => {
                if linelen > width1 {
                    ((linelen - width1 - 1) / width2 + 1) * width2 + width1
                } else {
                    width1
                }
            };
        }

        if (*win).w_view_width != 0 {
            if (*win).w_curswant == MAXCOL as c_int {
                atend = true;
                validate_virtcol(win);
                if width1 <= 0 {
                    (*win).w_curswant = 0;
                } else {
                    // Start from the end of the row the cursor is on.
                    (*win).w_curswant = width1 - 1;
                    if (*win).w_virtcol > (*win).w_curswant {
                        (*win).w_curswant +=
                            (((*win).w_virtcol - (*win).w_curswant - 1) / width2 + 1) * width2;
                    }
                }
            } else {
                let n = line_end!();
                (*win).w_curswant = (*win).w_curswant.min(n - 1);
            }
            while dist != 0 {
                dist -= 1;
                if dir == BACKWARD as c_int {
                    if (*win).w_curswant >= width1
                        && !hasFolding(win, (*win).w_cursor.lnum, ptr::null_mut(), ptr::null_mut())
                    {
                        // Still inside this line: back one row.
                        (*win).w_curswant -= width2;
                    } else if (*win).w_cursor.lnum <= 1 {
                        retval = false;
                        break;
                    } else {
                        cursor_up_inner(win, 1, skip_conceal);
                        linelen = linetabsize(win, (*win).w_cursor.lnum);
                        if linelen > width1 {
                            // Land on the *last* row of the line above.
                            let w = ((linelen - width1 - 1) / width2 + 1) * width2;
                            debug_assert!(w <= 0 || (*win).w_curswant <= c_int::MAX - w);
                            (*win).w_curswant += w;
                        }
                    }
                } else {
                    let n = line_end!();
                    if (*win).w_curswant + width2 < n
                        && !hasFolding(win, (*win).w_cursor.lnum, ptr::null_mut(), ptr::null_mut())
                    {
                        (*win).w_curswant += width2;
                    } else if (*win).w_cursor.lnum >= (*(*win).w_buffer).b_ml.ml_line_count {
                        retval = false;
                        break;
                    } else {
                        cursor_down_inner(win, 1, skip_conceal);
                        // Land on the *first* row of the line below.
                        (*win).w_curswant %= width2;
                        if (*win).w_curswant >= width1 {
                            (*win).w_curswant -= width2;
                        }
                        linelen = linetabsize(win, (*win).w_cursor.lnum);
                    }
                }
            }
        }

        if virtual_active(win) && atend {
            coladvance(win, MAXCOL as c_int);
        } else {
            coladvance(win, (*win).w_curswant);
        }

        if (*win).w_cursor.col > 0 && (*win).w_onebuf_opt.wo_wrap != 0 {
            validate_virtcol(win);
            let mut virtcol = (*win).w_virtcol;
            // 'showbreak' is drawn in front of every continuation row and is
            // not part of the text.
            if virtcol > width1 && *get_showbreak_value(win) as c_int != NUL {
                virtcol -= vim_strsize(get_showbreak_value(win));
            }
            let c = utf_ptr2char(get_cursor_pos_ptr());
            // A wide unprintable character is drawn as `<xxxx>`, which is
            // wider than the cell the column arithmetic assumed.
            if dir == FORWARD as c_int
                && virtcol < (*win).w_curswant
                && (*win).w_curswant <= width1
                && !vim_isprintc(c)
                && c > 255
            {
                oneright();
            }
            // Landed past the wanted column on a multi-cell character: keep
            // it only if more than half of it is before the wanted column.
            let mostly_past = if (*win).w_curswant < width1 {
                (*win).w_curswant > width1 / 2
            } else {
                ((*win).w_curswant - width1) % width2 > width2 / 2
            };
            if virtcol > (*win).w_curswant && mostly_past {
                (*win).w_cursor.col -= 1;
            }
        }
        if atend {
            (*win).w_curswant = MAXCOL as colnr_T;
        }
        adjust_skipcol();
        retval
    }
}

/// `H`, `M` and `L`: to the top, middle or bottom line of the window.
pub(crate) unsafe fn nv_scroll(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let win = curwin.get();
        (*(*cap).oap).motion_type = kMTLineWise;
        setpcmark();
        if (*cap).cmdchar == 'L' as c_int {
            validate_botline_win(win);
            (*win).w_cursor.lnum = (*win).w_botline - 1;
            if (*cap).count1 as linenr_T - 1 >= (*win).w_cursor.lnum {
                (*win).w_cursor.lnum = 1;
            } else if win_lines_concealed(win) {
                // A concealed line takes no screen row, so the count has to be
                // walked rather than subtracted.
                let mut n = (*cap).count1 - 1;
                while n > 0 && (*win).w_cursor.lnum > (*win).w_topline {
                    hasFolding(
                        win,
                        (*win).w_cursor.lnum,
                        &raw mut (*win).w_cursor.lnum,
                        ptr::null_mut(),
                    );
                    n += decor_conceal_line(win, (*win).w_cursor.lnum as c_int, true) as c_int;
                    if (*win).w_cursor.lnum > (*win).w_topline {
                        (*win).w_cursor.lnum -= 1;
                    }
                    n -= 1;
                }
            } else {
                (*win).w_cursor.lnum -= (*cap).count1 as linenr_T - 1;
            }
        } else {
            let mut n;
            if (*cap).cmdchar == 'M' as c_int {
                // Walk down counting screen rows until half the window's are
                // used up. Filler lines above the top line count against it.
                let mut used = -(win_get_fill(win, (*win).w_topline) - (*win).w_topfill);
                validate_botline_win(win);
                let half = ((*win).w_view_height - (*win).w_empty_rows + 1) / 2;
                n = 0;
                while ((*win).w_topline + n as linenr_T) < (*curbuf.get()).b_ml.ml_line_count {
                    if n > 0
                        && used + win_get_fill(win, (*win).w_topline + n as linenr_T) / 2 >= half
                    {
                        n -= 1;
                        break;
                    }
                    used += plines_win(win, (*win).w_topline + n as linenr_T, true);
                    if used >= half {
                        break;
                    }
                    let mut last: linenr_T = 0;
                    if hasFolding(
                        win,
                        (*win).w_topline + n as linenr_T,
                        ptr::null_mut(),
                        &raw mut last,
                    ) {
                        // The whole fold is one screen row.
                        n = (last - (*win).w_topline) as c_int;
                    }
                    n += 1;
                }
                if n > 0 && used > (*win).w_view_height {
                    n -= 1;
                }
            } else {
                n = (*cap).count1 - 1;
                if win_lines_concealed(win) {
                    let mut lnum = (*win).w_topline;
                    // The decrement is inside the condition, so a concealed
                    // line is stepped over without spending any of the count.
                    while (decor_conceal_line(win, lnum as c_int - 1, true) || {
                        let before = n;
                        n -= 1;
                        before > 0
                    }) && lnum < (*win).w_botline - 1
                    {
                        hasFolding(win, lnum, ptr::null_mut(), &raw mut lnum);
                        lnum += 1;
                    }
                    n = (lnum - (*win).w_topline) as c_int;
                }
            }
            (*win).w_cursor.lnum =
                ((*win).w_topline + n as linenr_T).min((*curbuf.get()).b_ml.ml_line_count);
        }
        if (*(*cap).oap).op_type == OP_NOP {
            cursor_correct(win);
        }
        beginline(BL_SOL as c_int | BL_FIX as c_int);
    }
}

/// `l`, `<Space>` and `<Right>`.
pub(crate) unsafe fn nv_right(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let win = curwin.get();
        // A modifier turns this into a word move.
        if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0 {
            if mod_mask.get() & MOD_MASK_CTRL != 0 {
                (*cap).arg = true_0;
            }
            nv_wordcmd(cap);
            return;
        }
        (*(*cap).oap).motion_type = kMTCharWise;
        (*(*cap).oap).inclusive = false;
        // With an inclusive selection the cursor may sit one past the last
        // character; 'virtualedit' handles that itself.
        let past_line =
            VIsual_active.get() && *p_sel.get() as c_int != 'o' as c_int && !virtual_active(win);

        // Which 'whichwrap' flag lets this key wrap to the next line.
        let wrap_flag = if (*cap).cmdchar == ' ' as c_int {
            's' as c_int
        } else if (*cap).cmdchar == 'l' as c_int {
            'l' as c_int
        } else if (*cap).cmdchar == K_RIGHT {
            '>' as c_int
        } else {
            NUL
        };

        let mut n = (*cap).count1;
        while n > 0 {
            let at_end = if past_line {
                *get_cursor_pos_ptr() as c_int == NUL
            } else {
                oneright() == false_0
            };
            if at_end {
                if wrap_flag != NUL
                    && !vim_strchr(p_ww.get(), wrap_flag).is_null()
                    && (*win).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
                {
                    // A pending exclusive operator eats the line break by
                    // becoming inclusive instead of moving.
                    if (*(*cap).oap).op_type != OP_NOP
                        && !(*(*cap).oap).inclusive
                        && *ml_get((*win).w_cursor.lnum) as c_int != NUL
                    {
                        (*(*cap).oap).inclusive = true;
                    } else {
                        (*win).w_cursor.lnum += 1;
                        (*win).w_cursor.col = 0;
                        (*win).w_cursor.coladd = 0;
                        (*win).w_set_curswant = true_0;
                        (*(*cap).oap).inclusive = false;
                    }
                } else {
                    // Only the *first* step failing is worth a beep; running
                    // out part-way through a count is not.
                    if (*(*cap).oap).op_type == OP_NOP {
                        if n == (*cap).count1 {
                            beep_flush();
                        }
                    } else if *ml_get((*win).w_cursor.lnum) as c_int != NUL {
                        (*(*cap).oap).inclusive = true;
                    }
                    break;
                }
            } else if past_line {
                (*win).w_set_curswant = true_0;
                if virtual_active(win) {
                    oneright();
                } else {
                    (*win).w_cursor.col += utfc_ptr2len(get_cursor_pos_ptr());
                }
            }
            n -= 1;
        }
        if n != (*cap).count1 {
            may_fold_open(cap, kOptFdoFlagHor as c_uint);
        }
    }
}

/// `h`, `<BS>`, CTRL-H and `<Left>`.
pub(crate) unsafe fn nv_left(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let win = curwin.get();
        // A modifier turns this into a word move.
        if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0 {
            if mod_mask.get() & MOD_MASK_CTRL != 0 {
                (*cap).arg = 1;
            }
            nv_bck_word(cap);
            return;
        }
        (*(*cap).oap).motion_type = kMTCharWise;
        (*(*cap).oap).inclusive = false;

        // Which 'whichwrap' flag lets this key wrap to the previous line.
        let wrap_flag = if (*cap).cmdchar == K_BS || (*cap).cmdchar == Ctrl_H {
            'b' as c_int
        } else if (*cap).cmdchar == 'h' as c_int {
            'h' as c_int
        } else if (*cap).cmdchar == K_LEFT {
            '<' as c_int
        } else {
            NUL
        };

        let mut n = (*cap).count1;
        while n > 0 {
            if oneleft() == false_0 {
                if wrap_flag != NUL
                    && !vim_strchr(p_ww.get(), wrap_flag).is_null()
                    && (*win).w_cursor.lnum > 1
                {
                    (*win).w_cursor.lnum -= 1;
                    coladvance(win, MAXCOL as c_int);
                    (*win).w_set_curswant = true_0;
                    // A delete or a change that wrapped back over the line
                    // break must take the break with it, so put the cursor
                    // one past the last character and tell the caller not to
                    // pull it back.
                    if ((*(*cap).oap).op_type == OP_DELETE || (*(*cap).oap).op_type == OP_CHANGE)
                        && *ml_get((*win).w_cursor.lnum) as c_int != NUL
                    {
                        let cp = get_cursor_pos_ptr();
                        if *cp as c_int != NUL {
                            (*win).w_cursor.col += utfc_ptr2len(cp);
                        }
                        (*cap).retval |= CA_NO_ADJ_OP_END as c_int;
                    }
                } else {
                    if (*(*cap).oap).op_type == OP_NOP && n == (*cap).count1 {
                        beep_flush();
                    }
                    break;
                }
            }
            n -= 1;
        }
        if n != (*cap).count1 {
            may_fold_open(cap, kOptFdoFlagHor as c_uint);
        }
    }
}

/// `k`, `CTRL-P`, `-` and `<Up>`. Shifted, it is a page up.
pub(crate) unsafe fn nv_up(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if mod_mask.get() & MOD_MASK_SHIFT != 0 {
            (*cap).arg = BACKWARD as c_int;
            nv_page(cap);
            return;
        }
        (*(*cap).oap).motion_type = kMTLineWise;
        if cursor_up((*cap).count1 as linenr_T, (*(*cap).oap).op_type == OP_NOP) == false_0 {
            clearopbeep((*cap).oap);
        } else if (*cap).arg != 0 {
            // `-` and `CTRL-P` land on the first non-blank; `k` does not.
            beginline(BL_WHITE as c_int | BL_FIX as c_int);
        }
    }
}

/// `j`, `CTRL-N`, `+`, `<CR>` and `<Down>`. Shifted, it is a page down.
pub(crate) unsafe fn nv_down(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if mod_mask.get() & MOD_MASK_SHIFT != 0 {
            (*cap).arg = FORWARD as c_int;
            nv_page(cap);
            return;
        }
        // In three kinds of window `<CR>` means "act on this line" rather
        // than "move down".
        if (*cap).cmdchar == CAR {
            if bt_quickfix(curbuf.get()) {
                qf_view_result(false);
                return;
            }
            if cmdwin_type.get() != 0 {
                cmdwin_result.set(CAR);
                return;
            }
            if bt_prompt(curbuf.get())
                && (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_ml.ml_line_count
            {
                prompt_invoke_callback();
                if restart_edit.get() == 0 {
                    restart_edit.set('a' as c_int);
                }
                return;
            }
        }
        (*(*cap).oap).motion_type = kMTLineWise;
        if cursor_down((*cap).count1, (*(*cap).oap).op_type == OP_NOP) == false_0 {
            clearopbeep((*cap).oap);
        } else if (*cap).arg != 0 {
            // `+`, `<CR>` and `CTRL-N` land on the first non-blank; `j` does
            // not.
            beginline(BL_WHITE as c_int | BL_FIX as c_int);
        }
    }
}

/// `<End>`: the end of the line -- of the last line with CTRL, which is what
/// the argument says.
pub(crate) unsafe fn nv_end(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if (*cap).arg != 0 || mod_mask.get() & MOD_MASK_CTRL != 0 {
            (*cap).arg = true_0;
            nv_goto(cap);
            // The count named the line, so `$` must not use it again.
            (*cap).count1 = 1;
        }
        nv_dollar(cap);
    }
}

/// `$`: the end of the line, `count1 - 1` lines down.
pub(crate) unsafe fn nv_dollar(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        (*(*cap).oap).motion_type = kMTCharWise;
        (*(*cap).oap).inclusive = true;
        // Under 'virtualedit' an operator that starts past the end of the
        // line keeps the column it has rather than asking for the end again.
        if !virtual_active(curwin.get()) || gchar_cursor() != NUL || (*(*cap).oap).op_type == OP_NOP
        {
            (*curwin.get()).w_curswant = MAXCOL as colnr_T;
        }
        if cursor_down((*cap).count1 - 1, (*(*cap).oap).op_type == OP_NOP) == false_0 {
            clearopbeep((*cap).oap);
        } else {
            may_fold_open(cap, kOptFdoFlagHor as c_uint);
        }
    }
}

/// `f`, `F`, `t`, `T`, `;` and `,`: search this line for a character.
pub(crate) unsafe fn nv_csearch(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        // An exclusive Select-mode selection was widened by one when it was
        // made; the search has to run against the real cursor position.
        let mut cursor_dec = false;
        if *p_sel.get() as c_int == 'e' as c_int
            && VIsual_active.get()
            && VIsual_mode.get() == 'v' as c_int
            && VIsual_select_exclu_adj.get()
        {
            unadjust_for_sel();
            cursor_dec = true;
        }
        // `t` and `T` stop *before* the character.
        let t_cmd = (*cap).cmdchar == 't' as c_int || (*cap).cmdchar == 'T' as c_int;
        (*(*cap).oap).motion_type = kMTCharWise;
        if (*cap).nchar < 0 || searchc(cap, t_cmd) == false_0 {
            clearopbeep((*cap).oap);
            if cursor_dec {
                adjust_for_sel(cap);
            }
            return;
        }
        (*curwin.get()).w_set_curswant = true_0;
        // Landing on a TAB with 'virtualedit' means the *last* cell of it,
        // so that `dt<Tab>` takes the whole tab.
        if gchar_cursor() == TAB
            && virtual_active(curwin.get())
            && (*cap).arg == FORWARD as c_int
            && (t_cmd || (*(*cap).oap).op_type != OP_NOP)
        {
            let mut scol: colnr_T = 0;
            let mut ecol: colnr_T = 0;
            getvcol(
                curwin.get(),
                &raw mut (*curwin.get()).w_cursor,
                &raw mut scol,
                ptr::null_mut(),
                &raw mut ecol,
            );
            (*curwin.get()).w_cursor.coladd = ecol - scol;
        } else {
            (*curwin.get()).w_cursor.coladd = 0;
        }
        adjust_for_sel(cap);
        may_fold_open(cap, kOptFdoFlagHor as c_uint);
    }
}

/// `%`: to the matching bracket, or with a count to that percentage of the
/// file.
pub(crate) unsafe fn nv_percent(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let win = curwin.get();
        let lnum = (*win).w_cursor.lnum;
        (*(*cap).oap).inclusive = true;
        if (*cap).count0 != 0 {
            if (*cap).count0 > 100 {
                clearopbeep((*cap).oap);
            } else {
                (*(*cap).oap).motion_type = kMTLineWise;
                setpcmark();
                let count = (*curbuf.get()).b_ml.ml_line_count;
                // Divide first for a file long enough that `count * 100`
                // would not fit.
                (*win).w_cursor.lnum = if count >= 21474836 {
                    (count + 99) / 100 * (*cap).count0 as linenr_T
                } else {
                    (count * (*cap).count0 as linenr_T + 99) / 100
                };
                (*win).w_cursor.lnum = (*win).w_cursor.lnum.max(1).min(count);
                beginline(BL_SOL as c_int | BL_FIX as c_int);
            }
        } else {
            (*(*cap).oap).motion_type = kMTCharWise;
            (*(*cap).oap).use_reg_one = true;
            let pos = findmatch((*cap).oap, NUL);
            if pos.is_null() {
                clearopbeep((*cap).oap);
            } else {
                setpcmark();
                (*win).w_cursor = *pos;
                (*win).w_set_curswant = true_0;
                (*win).w_cursor.coladd = 0;
                adjust_for_sel(cap);
            }
        }
        if lnum != (*win).w_cursor.lnum {
            may_fold_open(cap, kOptFdoFlagPercent as c_uint);
        }
    }
}

/// `(` and `)`: back and forward a sentence.
pub(crate) unsafe fn nv_brace(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        (*(*cap).oap).motion_type = kMTCharWise;
        (*(*cap).oap).use_reg_one = true;
        (*(*cap).oap).inclusive = false;
        (*curwin.get()).w_set_curswant = true_0;
        if findsent((*cap).arg as Direction, (*cap).count1) == FAIL {
            clearopbeep((*cap).oap);
            return;
        }
        adjust_cursor((*cap).oap);
        (*curwin.get()).w_cursor.coladd = 0;
        may_fold_open(cap, kOptFdoFlagBlock as c_uint);
    }
}

/// `{` and `}`: back and forward a paragraph.
pub(crate) unsafe fn nv_findpar(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        (*(*cap).oap).motion_type = kMTCharWise;
        (*(*cap).oap).inclusive = false;
        (*(*cap).oap).use_reg_one = true;
        (*curwin.get()).w_set_curswant = true_0;
        if !findpar(
            &raw mut (*(*cap).oap).inclusive,
            (*cap).arg,
            (*cap).count1,
            NUL,
            false,
        ) {
            clearopbeep((*cap).oap);
            return;
        }
        (*curwin.get()).w_cursor.coladd = 0;
        may_fold_open(cap, kOptFdoFlagBlock as c_uint);
    }
}

/// `<Home>`: the first column -- the first line with CTRL.
pub(crate) unsafe fn nv_home(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if mod_mask.get() & MOD_MASK_CTRL != 0 {
            nv_goto(cap);
        } else {
            // `<Home>` is `1|`.
            (*cap).count0 = 1;
            nv_pipe(cap);
        }
    }
    ins_at_eol.set(false);
}

/// `|`: to a screen column.
pub(crate) unsafe fn nv_pipe(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        (*(*cap).oap).motion_type = kMTCharWise;
        (*(*cap).oap).inclusive = false;
        beginline(0);
        if (*cap).count0 > 0 {
            coladvance(curwin.get(), (*cap).count0 - 1);
            (*curwin.get()).w_curswant = (*cap).count0 - 1;
        } else {
            (*curwin.get()).w_curswant = 0;
        }
        // The column was named outright, so it is not a remembered want.
        (*curwin.get()).w_set_curswant = false_0;
    }
}

/// `b` and `B`: back a word.
pub(crate) unsafe fn nv_bck_word(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        (*(*cap).oap).motion_type = kMTCharWise;
        (*(*cap).oap).inclusive = false;
        (*curwin.get()).w_set_curswant = true_0;
        if bck_word((*cap).count1, (*cap).arg != 0, false) == false_0 {
            clearopbeep((*cap).oap);
        } else {
            may_fold_open(cap, kOptFdoFlagHor as c_uint);
        }
    }
}

/// `w`, `W`, `e` and `E`: forward a word, or to a word's end.
pub(crate) unsafe fn nv_wordcmd(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let startpos = (*curwin.get()).w_cursor;
        let mut word_end = (*cap).cmdchar == 'e' as c_int || (*cap).cmdchar == 'E' as c_int;
        (*(*cap).oap).inclusive = word_end;

        // `cw` on a non-blank is `ce`: it changes the word, not up to the next
        // one. 'cpoptions' with `_` extends that to trailing white space.
        let mut cw_on_word = false;
        if !word_end && (*(*cap).oap).op_type == OP_CHANGE {
            let c = gchar_cursor();
            if c != NUL && !ascii_iswhite(c) {
                if !vim_strchr(p_cpo.get(), CPO_CHANGEW).is_null() {
                    (*(*cap).oap).inclusive = true;
                    word_end = true;
                }
                cw_on_word = true;
            }
        }

        (*(*cap).oap).motion_type = kMTCharWise;
        (*curwin.get()).w_set_curswant = true_0;
        let moved = if word_end {
            end_word((*cap).count1, (*cap).arg != 0, cw_on_word, false)
        } else {
            fwd_word(
                (*cap).count1,
                (*cap).arg != 0,
                (*(*cap).oap).op_type != OP_NOP,
            )
        };
        if lt(startpos, (*curwin.get()).w_cursor) {
            adjust_cursor((*cap).oap);
        }
        if moved == false_0 && (*(*cap).oap).op_type == OP_NOP {
            clearopbeep((*cap).oap);
        } else {
            adjust_for_sel(cap);
            may_fold_open(cap, kOptFdoFlagHor as c_uint);
        }
    }
}

/// Pull the cursor back off the line's terminator, which is not a position an
/// operator may include -- and say the operator now covers the last character.
pub(crate) unsafe fn adjust_cursor(oap: *mut oparg_T) {
    // SAFETY: `oap` is the caller's live operator.
    unsafe {
        if (*curwin.get()).w_cursor.col > 0
            && gchar_cursor() == NUL
            && (!VIsual_active.get() || *p_sel.get() as c_int == 'o' as c_int)
            && !virtual_active(curwin.get())
            && get_ve_flags(curwin.get()) & kOptVeFlagOnemore as c_uint == 0
        {
            (*curwin.get()).w_cursor.col -= 1;
            mb_adjust_cursor();
            (*oap).inclusive = true;
        }
    }
}

/// `0` and `^`: the first column, or the first non-blank, which is what the
/// argument says.
pub(crate) unsafe fn nv_beginline(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        (*(*cap).oap).motion_type = kMTCharWise;
        (*(*cap).oap).inclusive = false;
        beginline((*cap).arg);
        may_fold_open(cap, kOptFdoFlagHor as c_uint);
    }
    ins_at_eol.set(false);
}

/// `gg` and `G`: to the first or last line, or to the count'th.
pub(crate) unsafe fn nv_goto(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let last = (*curbuf.get()).b_ml.ml_line_count;
        let mut lnum = if (*cap).arg != 0 { last } else { 1 };
        (*(*cap).oap).motion_type = kMTLineWise;
        setpcmark();
        if (*cap).count0 != 0 {
            lnum = (*cap).count0 as linenr_T;
        }
        (*curwin.get()).w_cursor.lnum = lnum.max(1).min(last);
        beginline(BL_SOL as c_int | BL_FIX as c_int);
        may_fold_open(cap, kOptFdoFlagJump as c_uint);
    }
}
