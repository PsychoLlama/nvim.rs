//! Cursor motions that are not searches: by character, word, line,
//! screen line, paragraph and sentence.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::ops::Op;
use crate::winlayer::{Buf, Win};
use core::ptr;

use crate::ascii::ascii_iswhite;
use crate::buffer::{bt_prompt, bt_quickfix};
use crate::charset::{vim_isprintc, vim_strsize};
use crate::cursor::{coladvance, gchar_cursor, get_cursor_pos_ptr};
use crate::decoration::{decor_conceal_line, win_lines_concealed};
use crate::edit::{
    BeginlineOpts, beginline, cursor_down, cursor_down_inner, cursor_up, cursor_up_inner, oneleft,
    oneright,
};
use crate::eval::prompt_invoke_callback;
use crate::fold::has_folding;
use crate::getchar::beep_flush;
use crate::main::{
    VIsual_select_exclu_adj, cmdwin_result, cmdwin_type, curbuf, curwin, ins_at_eol, mod_mask,
    p_sel, p_ww, restart_edit,
};
use crate::mark::setpcmark;
use crate::mbyte::{mb_adjust_cursor, utf_ptr2char, utfc_ptr2len};
use crate::memline::ml_get;
use crate::normal::{
    CA_NO_ADJ_OP_END, CAR, CmdArg, MOD_MASK_CTRL, MOD_MASK_SHIFT, TAB, adjust_for_sel,
    clear_op_beep, kMTCharWise, kMTLineWise, may_fold_open, nv_page, unadjust_for_sel,
    visual_active, visual_mode,
};
use crate::option::{cpo_has, get_showbreak_value, get_ve_flags};
use crate::options::{
    kOptFdoFlagBlock, kOptFdoFlagHor, kOptFdoFlagJump, kOptFdoFlagPercent, kOptVeFlagOnemore,
};
use crate::plines::{linetabsize, plines_win, win_get_fill};
use crate::pos::{MAXCOL, lt};
use crate::quickfix::qf_view_result;
use crate::search::{BACKWARD, FORWARD, findmatch, searchc};
use crate::state::virtual_active;
use crate::strings::vim_strchr;
use crate::textobject::{bck_word, end_word, findpar, findsent, fwd_word};
use crate::types::{
    CpoFlag, Direction, FAIL, NUL, OP_CHANGE, OP_DELETE, OP_NOP, cmdarg_T, colnr_T, linenr_T,
    oparg_T, win_T,
};
use core::ffi::{c_int, c_uint};

use crate::keycodes::{Ctrl_H, K_BS, K_LEFT, K_RIGHT};
use crate::r#move::{
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
pub(crate) unsafe fn nv_screengo(
    oap: *mut oparg_T,
    dir: c_int,
    mut dist: c_int,
    skip_conceal: bool,
) -> bool {
    // SAFETY (throughout): `oap` is the caller's live operator.
    let mut op = unsafe { Op::new(oap) };
    let mut win = cur_win();
    let wp = win.raw();
    // SAFETY: the cursor line is a line of the window's own buffer.
    let mut linelen = unsafe { linetabsize(wp, win.w_cursor.lnum) };
    let mut retval = true;
    // `$` asked for the end of the line, which has to be recomputed on
    // every row rather than carried as a column.
    let mut atend = false;
    op.motion_type = kMTCharWise;
    op.inclusive = win.w_curswant == MAXCOL as c_int;

    // The first screen row of a line can be narrower than the rest: only
    // it carries the number column and the signs.
    let col_off1 = unsafe { win_col_off(wp) };
    let col_off2 = col_off1 - unsafe { win_col_off2(wp) };
    let width1 = win.w_view_width - col_off1;
    let mut width2 = win.w_view_width - col_off2;
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

    if win.w_view_width != 0 {
        if win.w_curswant == MAXCOL as c_int {
            atend = true;
            unsafe { validate_virtcol(wp) };
            if width1 <= 0 {
                win.w_curswant = 0;
            } else {
                // Start from the end of the row the cursor is on.
                win.w_curswant = width1 - 1;
                if win.w_virtcol > win.w_curswant {
                    win.w_curswant += ((win.w_virtcol - win.w_curswant - 1) / width2 + 1) * width2;
                }
            }
        } else {
            let n = line_end!();
            win.w_curswant = win.w_curswant.min(n - 1);
        }
        while dist != 0 {
            dist -= 1;
            if dir == BACKWARD as c_int {
                if win.w_curswant >= width1 && !folded(wp, win.w_cursor.lnum) {
                    // Still inside this line: back one row.
                    win.w_curswant -= width2;
                } else if win.w_cursor.lnum <= 1 {
                    retval = false;
                    break;
                } else {
                    unsafe { cursor_up_inner(wp, 1, skip_conceal) };
                    linelen = unsafe { linetabsize(wp, win.w_cursor.lnum) };
                    if linelen > width1 {
                        // Land on the *last* row of the line above.
                        let w = ((linelen - width1 - 1) / width2 + 1) * width2;
                        debug_assert!(w <= 0 || win.w_curswant <= c_int::MAX - w);
                        win.w_curswant += w;
                    }
                }
            } else {
                let n = line_end!();
                if win.w_curswant + width2 < n && !folded(wp, win.w_cursor.lnum) {
                    win.w_curswant += width2;
                } else if win.w_cursor.lnum >= win.buffer().b_ml.ml_line_count {
                    retval = false;
                    break;
                } else {
                    unsafe { cursor_down_inner(wp, 1, skip_conceal) };
                    // Land on the *first* row of the line below.
                    win.w_curswant %= width2;
                    if win.w_curswant >= width1 {
                        win.w_curswant -= width2;
                    }
                    linelen = unsafe { linetabsize(wp, win.w_cursor.lnum) };
                }
            }
        }
    }

    if unsafe { virtual_active(wp) } && atend {
        unsafe { coladvance(wp, MAXCOL as c_int) };
    } else {
        unsafe { coladvance(wp, win.w_curswant) };
    }

    if win.w_cursor.col > 0 && win.w_onebuf_opt.wo_wrap != 0 {
        unsafe { validate_virtcol(wp) };
        let mut virtcol = win.w_virtcol;
        // 'showbreak' is drawn in front of every continuation row and is
        // not part of the text.
        if virtcol > width1 && unsafe { *get_showbreak_value(wp) } as c_int != NUL {
            virtcol -= unsafe { vim_strsize(get_showbreak_value(wp)) };
        }
        let c = unsafe { utf_ptr2char(get_cursor_pos_ptr()) };
        // A wide unprintable character is drawn as `<xxxx>`, which is
        // wider than the cell the column arithmetic assumed.
        if dir == FORWARD as c_int
            && virtcol < win.w_curswant
            && win.w_curswant <= width1
            && !unsafe { vim_isprintc(c) }
            && c > 255
        {
            unsafe { oneright() };
        }
        // Landed past the wanted column on a multi-cell character: keep
        // it only if more than half of it is before the wanted column.
        let mostly_past = if win.w_curswant < width1 {
            win.w_curswant > width1 / 2
        } else {
            (win.w_curswant - width1) % width2 > width2 / 2
        };
        if virtcol > win.w_curswant && mostly_past {
            win.w_cursor.col -= 1;
        }
    }
    if atend {
        win.w_curswant = MAXCOL as colnr_T;
    }
    unsafe { adjust_skipcol() };
    retval
}

/// `H`, `M` and `L`: to the top, middle or bottom line of the window.
pub(crate) unsafe fn nv_scroll(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let (cmdchar, count1) = (ca.cmdchar, ca.count1);
    let mut op = ca.op();
    let mut win = cur_win();
    let wp = win.raw();
    op.motion_type = kMTLineWise;
    setpcmark();
    if cmdchar == 'L' as c_int {
        unsafe { validate_botline_win(wp) };
        win.w_cursor.lnum = win.w_botline - 1;
        if count1 as linenr_T > win.w_cursor.lnum {
            win.w_cursor.lnum = 1;
        } else if unsafe { win_lines_concealed(wp) } {
            // A concealed line takes no screen row, so the count has to be
            // walked rather than subtracted.
            let mut n = count1 - 1;
            while n > 0 && win.w_cursor.lnum > win.w_topline {
                let lnum = &raw mut win.w_cursor.lnum;
                unsafe { has_folding(wp, win.w_cursor.lnum, lnum, ptr::null_mut()) };
                n += unsafe { decor_conceal_line(wp, win.w_cursor.lnum as c_int, true) } as c_int;
                if win.w_cursor.lnum > win.w_topline {
                    win.w_cursor.lnum -= 1;
                }
                n -= 1;
            }
        } else {
            win.w_cursor.lnum -= count1 as linenr_T - 1;
        }
    } else {
        let mut n;
        if cmdchar == 'M' as c_int {
            // Walk down counting screen rows until half the window's are
            // used up. Filler lines above the top line count against it.
            let mut used = -(unsafe { win_get_fill(wp, win.w_topline) } - win.w_topfill);
            unsafe { validate_botline_win(wp) };
            let half = (win.w_view_height - win.w_empty_rows + 1) / 2;
            n = 0;
            while (win.w_topline + n as linenr_T) < cur_buf().b_ml.ml_line_count {
                if n > 0
                    && used + unsafe { win_get_fill(wp, win.w_topline + n as linenr_T) } / 2 >= half
                {
                    n -= 1;
                    break;
                }
                used += unsafe { plines_win(wp, win.w_topline + n as linenr_T, true) };
                if used >= half {
                    break;
                }
                let mut last: linenr_T = 0;
                let at = win.w_topline + n as linenr_T;
                if unsafe { has_folding(wp, at, ptr::null_mut(), &raw mut last) } {
                    // The whole fold is one screen row.
                    n = (last - win.w_topline) as c_int;
                }
                n += 1;
            }
            if n > 0 && used > win.w_view_height {
                n -= 1;
            }
        } else {
            n = count1 - 1;
            if unsafe { win_lines_concealed(wp) } {
                let mut lnum = win.w_topline;
                // The decrement is inside the condition, so a concealed
                // line is stepped over without spending any of the count.
                while (unsafe { decor_conceal_line(wp, lnum as c_int - 1, true) } || {
                    let before = n;
                    n -= 1;
                    before > 0
                }) && lnum < win.w_botline - 1
                {
                    unsafe { has_folding(wp, lnum, ptr::null_mut(), &raw mut lnum) };
                    lnum += 1;
                }
                n = (lnum - win.w_topline) as c_int;
            }
        }
        win.w_cursor.lnum = (win.w_topline + n as linenr_T).min(cur_buf().b_ml.ml_line_count);
    }
    if op.op_type == OP_NOP {
        // SAFETY: `wp` is the live window.
        unsafe { cursor_correct(wp) };
    }
    beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
}

/// `l`, `<Space>` and `<Right>`.
pub(crate) unsafe fn nv_right(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    // A modifier turns this into a word move.
    if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0 {
        if mod_mask.get() & MOD_MASK_CTRL != 0 {
            ca.arg = 1;
        }
        // SAFETY: `cap` is the caller's live command argument.
        unsafe { nv_wordcmd(cap) };
        return;
    }
    // SAFETY: `cap` is the caller's live command argument.
    let (cmdchar, count1) = (ca.cmdchar, ca.count1);
    let mut op = ca.op();
    let mut win = cur_win();
    let wp = win.raw();
    op.motion_type = kMTCharWise;
    op.inclusive = false;
    // With an inclusive selection the cursor may sit one past the last
    // character; 'virtualedit' handles that itself.
    // SAFETY: 'selection' is a NUL-terminated option string.
    let sel = unsafe { *p_sel.get() } as c_int;
    let past_line = visual_active() && sel != 'o' as c_int && !unsafe { virtual_active(wp) };

    // Which 'whichwrap' flag lets this key wrap to the next line.
    let wrap_flag = if cmdchar == ' ' as c_int {
        's' as c_int
    } else if cmdchar == 'l' as c_int {
        'l' as c_int
    } else if cmdchar == K_RIGHT {
        '>' as c_int
    } else {
        NUL
    };

    let mut n = count1;
    while n > 0 {
        // SAFETY: the cursor position is inside its own NUL-terminated line.
        let at_end = if past_line {
            unsafe { *get_cursor_pos_ptr() as c_int == NUL }
        } else {
            unsafe { oneright() == 0 }
        };
        if at_end {
            if wrap_flag != NUL
                && !unsafe { vim_strchr(p_ww.get(), wrap_flag) }.is_null()
                && win.w_cursor.lnum < cur_buf().b_ml.ml_line_count
            {
                // A pending exclusive operator eats the line break by
                // becoming inclusive instead of moving.
                // SAFETY: `oap` is live and the cursor line is terminated.
                let eat = unsafe {
                    op.op_type != OP_NOP
                        && !op.inclusive
                        && *ml_get(win.w_cursor.lnum) as c_int != NUL
                };
                if eat {
                    op.inclusive = true;
                } else {
                    win.w_cursor.lnum += 1;
                    win.w_cursor.col = 0;
                    win.w_cursor.coladd = 0;
                    win.w_set_curswant = true;
                    op.inclusive = false;
                }
            } else {
                // Only the *first* step failing is worth a beep; running
                // out part-way through a count is not.
                if op.op_type == OP_NOP {
                    if n == count1 {
                        beep_flush();
                    }
                    // SAFETY: the cursor line is NUL-terminated.
                } else if unsafe { *ml_get(win.w_cursor.lnum) } as c_int != NUL {
                    op.inclusive = true;
                }
                break;
            }
        } else if past_line {
            win.w_set_curswant = true;
            if unsafe { virtual_active(wp) } {
                // SAFETY: the cursor is in its own line.
                unsafe { oneright() };
            } else {
                // SAFETY: as above.
                win.w_cursor.col += unsafe { utfc_ptr2len(get_cursor_pos_ptr()) };
            }
        }
        n -= 1;
    }
    if n != count1 {
        // SAFETY: `cap` is the caller's live command argument.
        unsafe { may_fold_open(cap, kOptFdoFlagHor as c_uint) };
    }
}

/// `h`, `<BS>`, CTRL-H and `<Left>`.
pub(crate) unsafe fn nv_left(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let mut win = cur_win();
    // A modifier turns this into a word move.
    if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0 {
        if mod_mask.get() & MOD_MASK_CTRL != 0 {
            ca.arg = 1;
        }
        unsafe { nv_bck_word(cap) };
        return;
    }
    ca.op().motion_type = kMTCharWise;
    ca.op().inclusive = false;

    // Which 'whichwrap' flag lets this key wrap to the previous line.
    let wrap_flag = if ca.cmdchar == K_BS || ca.cmdchar == Ctrl_H {
        'b' as c_int
    } else if ca.cmdchar == 'h' as c_int {
        'h' as c_int
    } else if ca.cmdchar == K_LEFT {
        '<' as c_int
    } else {
        NUL
    };

    let mut n = ca.count1;
    while n > 0 {
        if unsafe { oneleft() } == 0 {
            if wrap_flag != NUL
                && !unsafe { vim_strchr(p_ww.get(), wrap_flag) }.is_null()
                && win.w_cursor.lnum > 1
            {
                win.w_cursor.lnum -= 1;
                unsafe { coladvance(win.raw(), MAXCOL as c_int) };
                win.w_set_curswant = true;
                // A delete or a change that wrapped back over the line
                // break must take the break with it, so put the cursor
                // one past the last character and tell the caller not to
                // pull it back.
                if (ca.op().op_type == OP_DELETE || ca.op().op_type == OP_CHANGE)
                    && unsafe { *ml_get(win.w_cursor.lnum) } as c_int != NUL
                {
                    let cp = get_cursor_pos_ptr();
                    if unsafe { *cp } as c_int != NUL {
                        unsafe { win.w_cursor.col += utfc_ptr2len(cp) };
                    }
                    ca.retval |= CA_NO_ADJ_OP_END as c_int;
                }
            } else {
                if ca.op().op_type == OP_NOP && n == ca.count1 {
                    beep_flush();
                }
                break;
            }
        }
        n -= 1;
    }
    if n != ca.count1 {
        unsafe { may_fold_open(cap, kOptFdoFlagHor as c_uint) };
    }
}

/// `k`, `CTRL-P`, `-` and `<Up>`. Shifted, it is a page up.
pub(crate) unsafe fn nv_up(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if mod_mask.get() & MOD_MASK_SHIFT != 0 {
        ca.arg = BACKWARD as c_int;
        unsafe { nv_page(cap) };
        return;
    }
    ca.op().motion_type = kMTLineWise;
    if unsafe { cursor_up(ca.count1 as linenr_T, ca.op().op_type == OP_NOP) } == 0 {
        clear_op_beep(ca.op());
    } else if ca.arg != 0 {
        // `-` and `CTRL-P` land on the first non-blank; `k` does not.
        beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    }
}

/// `j`, `CTRL-N`, `+`, `<CR>` and `<Down>`. Shifted, it is a page down.
pub(crate) unsafe fn nv_down(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if mod_mask.get() & MOD_MASK_SHIFT != 0 {
        ca.arg = FORWARD as c_int;
        unsafe { nv_page(cap) };
        return;
    }
    // In three kinds of window `<CR>` means "act on this line" rather
    // than "move down".
    if ca.cmdchar == CAR {
        if unsafe { bt_quickfix(curbuf.get()) } {
            unsafe { qf_view_result(false) };
            return;
        }
        if cmdwin_type.get() != 0 {
            cmdwin_result.set(CAR);
            return;
        }
        if unsafe { bt_prompt(curbuf.get()) }
            && cur_win().w_cursor.lnum == cur_buf().b_ml.ml_line_count
        {
            unsafe { prompt_invoke_callback() };
            if restart_edit.get() == 0 {
                restart_edit.set('a' as c_int);
            }
            return;
        }
    }
    ca.op().motion_type = kMTLineWise;
    if unsafe { cursor_down(ca.count1, ca.op().op_type == OP_NOP) } == 0 {
        clear_op_beep(ca.op());
    } else if ca.arg != 0 {
        // `+`, `<CR>` and `CTRL-N` land on the first non-blank; `j` does
        // not.
        beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    }
}

/// `<End>`: the end of the line -- of the last line with CTRL, which is what
/// the argument says.
pub(crate) unsafe fn nv_end(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if ca.arg != 0 || mod_mask.get() & MOD_MASK_CTRL != 0 {
        ca.arg = 1;
        unsafe { nv_goto(cap) };
        // The count named the line, so `$` must not use it again.
        ca.count1 = 1;
    }
    unsafe { nv_dollar(cap) };
}

/// `$`: the end of the line, `count1 - 1` lines down.
pub(crate) unsafe fn nv_dollar(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    ca.op().motion_type = kMTCharWise;
    ca.op().inclusive = true;
    // Under 'virtualedit' an operator that starts past the end of the
    // line keeps the column it has rather than asking for the end again.
    if !unsafe { virtual_active(curwin.get()) }
        || gchar_cursor() != NUL
        || ca.op().op_type == OP_NOP
    {
        cur_win().w_curswant = MAXCOL as colnr_T;
    }
    if unsafe { cursor_down(ca.count1 - 1, ca.op().op_type == OP_NOP) } == 0 {
        clear_op_beep(ca.op());
    } else {
        unsafe { may_fold_open(cap, kOptFdoFlagHor as c_uint) };
    }
}

/// `f`, `F`, `t`, `T`, `;` and `,`: search this line for a character.
pub(crate) unsafe fn nv_csearch(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    // An exclusive Select-mode selection was widened by one when it was
    // made; the search has to run against the real cursor position.
    let mut cursor_dec = false;
    if unsafe { *p_sel.get() } as c_int == 'e' as c_int
        && visual_active()
        && visual_mode().is_char()
        && VIsual_select_exclu_adj.get()
    {
        unsafe { unadjust_for_sel() };
        cursor_dec = true;
    }
    // `t` and `T` stop *before* the character.
    let t_cmd = ca.cmdchar == 't' as c_int || ca.cmdchar == 'T' as c_int;
    ca.op().motion_type = kMTCharWise;
    if ca.nchar < 0 || unsafe { searchc(cap, t_cmd) } == 0 {
        clear_op_beep(ca.op());
        if cursor_dec {
            unsafe { adjust_for_sel(cap) };
        }
        return;
    }
    cur_win().w_set_curswant = true;
    // Landing on a TAB with 'virtualedit' means the *last* cell of it,
    // so that `dt<Tab>` takes the whole tab.
    if gchar_cursor() == TAB
        && unsafe { virtual_active(curwin.get()) }
        && ca.arg == FORWARD as c_int
        && (t_cmd || ca.op().op_type != OP_NOP)
    {
        let win = cur_win();
        let (scol, ecol) = win.vcol_span(win.cursor());
        cur_win().w_cursor.coladd = ecol - scol;
    } else {
        cur_win().w_cursor.coladd = 0;
    }
    unsafe { adjust_for_sel(cap) };
    unsafe { may_fold_open(cap, kOptFdoFlagHor as c_uint) };
}

/// `%`: to the matching bracket, or with a count to that percentage of the
/// file.
pub(crate) unsafe fn nv_percent(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let count0 = ca.count0;
    let mut op = ca.op();
    let mut win = cur_win();
    let lnum = win.w_cursor.lnum;
    op.inclusive = true;
    if count0 != 0 {
        if count0 > 100 {
            // SAFETY: `oap` is the command's live operator.
            clear_op_beep(op);
        } else {
            op.motion_type = kMTLineWise;
            setpcmark();
            let count = cur_buf().b_ml.ml_line_count;
            // Divide first for a file long enough that `count * 100`
            // would not fit.
            win.w_cursor.lnum = if count >= 21474836 {
                (count + 99) / 100 * count0 as linenr_T
            } else {
                (count * count0 as linenr_T + 99) / 100
            };
            win.w_cursor.lnum = win.w_cursor.lnum.max(1).min(count);
            // SAFETY: the editor's text state is live.
            beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
        }
    } else {
        op.motion_type = kMTCharWise;
        op.use_reg_one = true;
        // SAFETY: `op` is the command's live operator.
        let pos = unsafe { findmatch(op.raw(), NUL) };
        if let Some(pos) = pos {
            // SAFETY: the jump list is live and `cap` is the caller's.
            setpcmark();
            win.w_cursor = pos;
            win.w_set_curswant = true;
            win.w_cursor.coladd = 0;
            unsafe { adjust_for_sel(cap) };
        } else {
            // SAFETY: `oap` is the command's live operator.
            clear_op_beep(op);
        }
    }
    if lnum != win.w_cursor.lnum {
        // SAFETY: `cap` is the caller's live command argument.
        unsafe { may_fold_open(cap, kOptFdoFlagPercent as c_uint) };
    }
}

/// `(` and `)`: back and forward a sentence.
pub(crate) unsafe fn nv_brace(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    ca.op().motion_type = kMTCharWise;
    ca.op().use_reg_one = true;
    ca.op().inclusive = false;
    cur_win().w_set_curswant = true;
    if unsafe { findsent(ca.arg as Direction, ca.count1) } == FAIL {
        clear_op_beep(ca.op());
        return;
    }
    unsafe { adjust_cursor(ca.oap) };
    cur_win().w_cursor.coladd = 0;
    unsafe { may_fold_open(cap, kOptFdoFlagBlock as c_uint) };
}

/// `{` and `}`: back and forward a paragraph.
pub(crate) unsafe fn nv_findpar(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    ca.op().motion_type = kMTCharWise;
    ca.op().inclusive = false;
    ca.op().use_reg_one = true;
    cur_win().w_set_curswant = true;
    if !unsafe { findpar(&raw mut ca.op().inclusive, ca.arg, ca.count1, NUL, false) } {
        clear_op_beep(ca.op());
        return;
    }
    cur_win().w_cursor.coladd = 0;
    unsafe { may_fold_open(cap, kOptFdoFlagBlock as c_uint) };
}

/// `<Home>`: the first column -- the first line with CTRL.
pub(crate) unsafe fn nv_home(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if mod_mask.get() & MOD_MASK_CTRL != 0 {
        unsafe { nv_goto(cap) };
    } else {
        // `<Home>` is `1|`.
        ca.count0 = 1;
        unsafe { nv_pipe(cap) };
    }
    ins_at_eol.set(false);
}

/// `|`: to a screen column.
pub(crate) unsafe fn nv_pipe(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    ca.op().motion_type = kMTCharWise;
    ca.op().inclusive = false;
    beginline(BeginlineOpts::NONE);
    if ca.count0 > 0 {
        unsafe { coladvance(curwin.get(), ca.count0 - 1) };
        cur_win().w_curswant = ca.count0 - 1;
    } else {
        cur_win().w_curswant = 0;
    }
    // The column was named outright, so it is not a remembered want.
    cur_win().w_set_curswant = false;
}

/// `b` and `B`: back a word.
pub(crate) unsafe fn nv_bck_word(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    ca.op().motion_type = kMTCharWise;
    ca.op().inclusive = false;
    cur_win().w_set_curswant = true;
    if unsafe { bck_word(ca.count1, ca.arg != 0, false) } == 0 {
        clear_op_beep(ca.op());
    } else {
        unsafe { may_fold_open(cap, kOptFdoFlagHor as c_uint) };
    }
}

/// `w`, `W`, `e` and `E`: forward a word, or to a word's end.
pub(crate) unsafe fn nv_wordcmd(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let startpos = cur_win().w_cursor;
    let mut word_end = ca.cmdchar == 'e' as c_int || ca.cmdchar == 'E' as c_int;
    ca.op().inclusive = word_end;

    // `cw` on a non-blank is `ce`: it changes the word, not up to the next
    // one. 'cpoptions' with `_` extends that to trailing white space.
    let mut cw_on_word = false;
    if !word_end && ca.op().op_type == OP_CHANGE {
        let c = gchar_cursor();
        if c != NUL && !ascii_iswhite(c) {
            if cpo_has(CpoFlag::CHANGEW) {
                ca.op().inclusive = true;
                word_end = true;
            }
            cw_on_word = true;
        }
    }

    ca.op().motion_type = kMTCharWise;
    cur_win().w_set_curswant = true;
    let moved = if word_end {
        unsafe { end_word(ca.count1, ca.arg != 0, cw_on_word, false) }
    } else {
        unsafe { fwd_word(ca.count1, ca.arg != 0, ca.op().op_type != OP_NOP) }
    };
    if lt(startpos, cur_win().w_cursor) {
        unsafe { adjust_cursor(ca.oap) };
    }
    if moved == 0 && ca.op().op_type == OP_NOP {
        clear_op_beep(ca.op());
    } else {
        unsafe { adjust_for_sel(cap) };
        unsafe { may_fold_open(cap, kOptFdoFlagHor as c_uint) };
    }
}

/// Pull the cursor back off the line's terminator, which is not a position an
/// operator may include -- and say the operator now covers the last character.
pub(crate) unsafe fn adjust_cursor(oap: *mut oparg_T) {
    // SAFETY (throughout): `oap` is the caller's live operator.
    let mut op = unsafe { Op::new(oap) };
    if cur_win().w_cursor.col > 0
        && gchar_cursor() == NUL
        && (!visual_active() || unsafe { *p_sel.get() } as c_int == 'o' as c_int)
        && !unsafe { virtual_active(curwin.get()) }
        && unsafe { get_ve_flags(curwin.get()) } & kOptVeFlagOnemore as c_uint == 0
    {
        cur_win().w_cursor.col -= 1;
        unsafe { mb_adjust_cursor() };
        op.inclusive = true;
    }
}

/// `0` and `^`: the first column, or the first non-blank, which is what the
/// argument says.
pub(crate) unsafe fn nv_beginline(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    ca.op().motion_type = kMTCharWise;
    ca.op().inclusive = false;
    beginline(BeginlineOpts::from_bits(ca.arg));
    unsafe { may_fold_open(cap, kOptFdoFlagHor as c_uint) };
    ins_at_eol.set(false);
}

/// `gg` and `G`: to the first or last line, or to the count'th.
pub(crate) unsafe fn nv_goto(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let last = cur_buf().b_ml.ml_line_count;
    let mut lnum = if ca.arg != 0 { last } else { 1 };
    ca.op().motion_type = kMTLineWise;
    setpcmark();
    if ca.count0 != 0 {
        lnum = ca.count0 as linenr_T;
    }
    cur_win().w_cursor.lnum = lnum.max(1).min(last);
    beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    unsafe { may_fold_open(cap, kOptFdoFlagJump as c_uint) };
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

/// Whether `lnum` is inside a closed fold of `wp`.
fn folded(wp: *mut win_T, lnum: linenr_T) -> bool {
    // SAFETY: the caller's window is live; both fold ends are unwanted.
    unsafe { has_folding(wp, lnum, ptr::null_mut(), ptr::null_mut()) }
}
