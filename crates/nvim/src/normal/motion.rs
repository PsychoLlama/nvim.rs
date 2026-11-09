//! Cursor motions that are not searches: by character, word, line,
//! screen line, paragraph and sentence.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::cstr;
use crate::keycodes::ModMask;
use crate::keycodes::SpecialKey;
use crate::ops::Op;
use crate::types::Key;
use crate::winlayer::{Buf, Win};

use crate::ascii::NUL;
use crate::ascii::ascii_iswhite;
use crate::buffer::{buf_is_prompt, buf_is_quickfix, current_buf};
use crate::charset::vim_isprintc;
use crate::cursor::{coladvance, gchar_cursor, with_cursor_rest};
use crate::decoration::{decor_conceal_line, win_lines_concealed};
use crate::edit::{
    BeginlineOpts, beginline, cursor_down, cursor_down_inner, cursor_up, cursor_up_inner, oneleft,
    oneright,
};
use crate::eval::prompt_invoke_callback;
use crate::getchar::beep_flush;
use crate::getchar::state::mod_mask;
use crate::mark::setpcmark;
use crate::mbyte::{cluster_len, mb_adjust_cursor};
use crate::normal::defs::{CmdArg, Outcome};
use crate::normal::{
    MT_CHAR_WISE, MT_LINE_WISE, TAB, adjust_for_sel, clear_op_beep, may_fold_open, nv_page,
    unadjust_for_sel, visual_active, visual_mode,
};
use crate::ops::OpType;
use crate::option::defs::CpoFlag;
use crate::option::vars::{P_SEL, p_sel, p_ww};
use crate::option::{cpo_has, get_ve_flags};
use crate::options::{
    kOptFdoFlagBlock, kOptFdoFlagHor, kOptFdoFlagJump, kOptFdoFlagPercent, kOptVeFlagOnemore,
};
use crate::plines::{linetabsize, plines_win, win_get_fill};
use crate::pos::lt;
use crate::quickfix::qf_view_result;
use crate::search::{BACKWARD, FORWARD, findmatch, searchc};
use crate::state::mode::{VIsual_select_exclu_adj, ins_at_eol, restart_edit};
use crate::state::virtual_active;
use crate::textobject::{bck_word, end_word, findpar, findsent, fwd_word};
use crate::types::{ColNr, Direction, LineNr};
use crate::winlayer::graph::{cmdwin_result, cmdwin_type};
use core::ffi::{c_int, c_uint};

use crate::r#move::{
    adjust_skipcol, cursor_correct, validate_botline_win, validate_virtcol, win_col_off2,
};

/// Move `dist` *screen* lines, which is what `gj`/`gk` and `g$` past the first
/// count mean, and what the arrow keys mean under 'wrap'.
///
/// The wanted column ('curswant') is carried in screen columns for the whole
/// walk and only turned back into a buffer column at the end, which is how a
/// long wrapped line and a short one keep the same apparent column. A closed
/// fold counts as one screen line however tall its text is, so the walk steps
/// over it rather than through it.
pub(crate) fn nv_screengo(mut op: Op, dir: c_int, mut dist: c_int, skip_conceal: bool) -> bool {
    let mut win = Win::current();
    let wp = win;
    let mut linelen = linetabsize(wp, win.w_cursor.lnum);
    let mut retval = true;
    // `$` asked for the end of the line, which has to be recomputed on
    // every row rather than carried as a column.
    let mut atend = false;
    op.motion_type = MT_CHAR_WISE;
    op.inclusive = win.w_curswant == ColNr::MAX;

    // The first screen row of a line can be narrower than the rest: only
    // it carries the number column and the signs.
    let col_off1 = wp.col_off();
    let col_off2 = col_off1 - win_col_off2(wp);
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
        if win.w_curswant == ColNr::MAX {
            atend = true;
            validate_virtcol(wp);
            if width1 <= 0 {
                win.w_curswant = ColNr::ZERO;
            } else {
                // Start from the end of the row the cursor is on.
                win.w_curswant = ColNr::new(width1 - 1);
                if win.w_virtcol > win.w_curswant {
                    let rows = (win.w_virtcol - win.w_curswant - 1) / width2 + 1;
                    win.w_curswant += rows * width2;
                }
            }
        } else {
            let n = line_end!();
            win.w_curswant = win.w_curswant.min(ColNr::new(n - 1));
        }
        while dist != 0 {
            dist -= 1;
            if dir == BACKWARD as c_int {
                if win.w_curswant >= ColNr::new(width1) && !folded(win, win.w_cursor.lnum) {
                    // Still inside this line: back one row.
                    win.w_curswant -= width2;
                } else if win.w_cursor.lnum <= LineNr::ONE {
                    retval = false;
                    break;
                } else {
                    cursor_up_inner(wp, 1, skip_conceal);
                    linelen = linetabsize(wp, win.w_cursor.lnum);
                    if linelen > width1 {
                        // Land on the *last* row of the line above.
                        let w = ((linelen - width1 - 1) / width2 + 1) * width2;
                        debug_assert!(w <= 0 || win.w_curswant <= ColNr::new(c_int::MAX - w));
                        win.w_curswant += w;
                    }
                }
            } else {
                let n = line_end!();
                if win.w_curswant + width2 < ColNr::new(n) && !folded(win, win.w_cursor.lnum) {
                    win.w_curswant += width2;
                } else if win.w_cursor.lnum >= win.buffer().b_ml.ml_line_count {
                    retval = false;
                    break;
                } else {
                    cursor_down_inner(wp, 1, skip_conceal);
                    // Land on the *first* row of the line below.
                    win.w_curswant = ColNr::new(win.w_curswant.get() % width2);
                    if win.w_curswant >= ColNr::new(width1) {
                        win.w_curswant -= width2;
                    }
                    linelen = linetabsize(wp, win.w_cursor.lnum);
                }
            }
        }
    }

    if virtual_active(win) && atend {
        coladvance(win, ColNr::MAX);
    } else {
        coladvance(win, win.w_curswant);
    }

    if win.w_cursor.col > ColNr::ZERO && win.w_onebuf_opt.wo_wrap != 0 {
        validate_virtcol(wp);
        let mut virtcol = win.w_virtcol;
        // 'showbreak' is drawn in front of every continuation row and is
        // not part of the text.
        if virtcol > ColNr::new(width1) && wp.has_showbreak() {
            virtcol -= wp.showbreak_cells();
        }
        let c = gchar_cursor();
        // A wide unprintable character is drawn as `<xxxx>`, which is
        // wider than the cell the column arithmetic assumed.
        if dir == FORWARD as c_int
            && virtcol < win.w_curswant
            && win.w_curswant <= ColNr::new(width1)
            && !vim_isprintc(c)
            && c > 255
        {
            let _ = oneright();
        }
        // Landed past the wanted column on a multi-cell character: keep
        // it only if more than half of it is before the wanted column.
        let mostly_past = if win.w_curswant < ColNr::new(width1) {
            win.w_curswant > ColNr::new(width1 / 2)
        } else {
            (win.w_curswant - width1).get() % width2 > width2 / 2
        };
        if virtcol > win.w_curswant && mostly_past {
            win.w_cursor.col -= 1;
        }
    }
    if atend {
        win.w_curswant = ColNr::MAX;
    }
    adjust_skipcol();
    retval
}

/// `H`, `M` and `L`: to the top, middle or bottom line of the window.
pub(crate) fn nv_scroll(cmd_arg: &mut CmdArg) {
    let (cmdchar, count1) = (cmd_arg.cmdchar, cmd_arg.count1);
    let mut op = cmd_arg.op();
    let mut win = Win::current();
    let wp = win;
    op.motion_type = MT_LINE_WISE;
    setpcmark();
    if cmdchar.is('L') {
        validate_botline_win(wp);
        win.w_cursor.lnum = win.w_botline - 1;
        if LineNr::new(count1) > win.w_cursor.lnum {
            win.w_cursor.lnum = LineNr::ONE;
        } else if win_lines_concealed(wp) {
            // A concealed line takes no screen row, so the count has to be
            // walked rather than subtracted.
            let mut n = count1 - 1;
            while n > 0 && win.w_cursor.lnum > win.w_topline {
                let lnum = win.w_cursor.lnum;
                win.has_folding(lnum, Some(&mut win.w_cursor.lnum), None);
                n += c_int::from(decor_conceal_line(wp, win.w_cursor.lnum.get(), true));
                if win.w_cursor.lnum > win.w_topline {
                    win.w_cursor.lnum -= 1;
                }
                n -= 1;
            }
        } else {
            win.w_cursor.lnum -= count1 - 1;
        }
    } else {
        let mut n;
        if cmdchar.is('M') {
            // Walk down counting screen rows until half the window's are
            // used up. Filler lines above the top line count against it.
            let mut used = -(win_get_fill(wp, win.w_topline) - win.w_topfill);
            validate_botline_win(wp);
            let half = (win.w_view_height - win.w_empty_rows + 1) / 2;
            n = 0;
            while (win.w_topline + n) < Buf::current().b_ml.ml_line_count {
                if n > 0 && used + win_get_fill(wp, win.w_topline + n) / 2 >= half {
                    n -= 1;
                    break;
                }
                used += plines_win(wp, win.w_topline + n, true);
                if used >= half {
                    break;
                }
                let mut last: LineNr = LineNr::ZERO;
                let at = win.w_topline + n;
                if wp.has_folding(at, None, Some(&mut last)) {
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
            if win_lines_concealed(wp) {
                let mut lnum = win.w_topline;
                // The decrement is inside the condition, so a concealed
                // line is stepped over without spending any of the count.
                while (decor_conceal_line(wp, lnum.get() - 1, true) || {
                    let before = n;
                    n -= 1;
                    before > 0
                }) && lnum < win.w_botline - 1
                {
                    wp.has_folding(lnum, None, Some(&mut lnum));
                    lnum += 1;
                }
                n = (lnum - win.w_topline) as c_int;
            }
        }
        win.w_cursor.lnum = (win.w_topline + n).min(Buf::current().b_ml.ml_line_count);
    }
    if op.op_type == OpType::Nop {
        cursor_correct(wp);
    }
    beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
}

/// `l`, `<Space>` and `<Right>`.
pub(crate) fn nv_right(cmd_arg: &mut CmdArg) {
    // A modifier turns this into a word move.
    if mod_mask.get().has(ModMask::SHIFT | ModMask::CTRL) {
        if mod_mask.get().has(ModMask::CTRL) {
            cmd_arg.arg = 1;
        }
        nv_wordcmd(cmd_arg);
        return;
    }
    let (cmdchar, count1) = (cmd_arg.cmdchar, cmd_arg.count1);
    let mut op = cmd_arg.op();
    let mut win = Win::current();
    op.motion_type = MT_CHAR_WISE;
    op.inclusive = false;
    // With an inclusive selection the cursor may sit one past the last
    // character; 'virtualedit' handles that itself.
    let sel = p_sel(cstr::first);
    let past_line = visual_active() && sel != b'o' && !virtual_active(win);

    // Which 'whichwrap' flag lets this key wrap to the next line.
    let wrap_flag = if cmdchar.is(' ') {
        b's'
    } else if cmdchar.is('l') {
        b'l'
    } else if cmdchar == SpecialKey::Right {
        b'>'
    } else {
        0
    };

    let mut n = count1;
    while n > 0 {
        let at_end = if past_line {
            gchar_cursor() == NUL
        } else {
            oneright().is_err()
        };
        if at_end {
            if wrap_flag != 0
                && p_ww(|value| value.to_bytes().contains(&wrap_flag))
                && win.w_cursor.lnum < Buf::current().b_ml.ml_line_count
            {
                // A pending exclusive operator eats the line break by
                // becoming inclusive instead of moving.
                let eat = op.op_type != OpType::Nop
                    && !op.inclusive
                    && !Buf::current().lines().line(win.w_cursor.lnum).is_empty();
                if eat {
                    op.inclusive = true;
                } else {
                    win.w_cursor.lnum += 1;
                    win.w_cursor.col = ColNr::ZERO;
                    win.w_cursor.coladd = 0;
                    win.w_set_curswant = true;
                    op.inclusive = false;
                }
            } else {
                // Only the *first* step failing is worth a beep; running
                // out part-way through a count is not.
                if op.op_type == OpType::Nop {
                    if n == count1 {
                        beep_flush();
                    }
                } else if !Buf::current().lines().line(win.w_cursor.lnum).is_empty() {
                    op.inclusive = true;
                }
                break;
            }
        } else if past_line {
            win.w_set_curswant = true;
            if virtual_active(win) {
                let _ = oneright();
            } else {
                win.w_cursor.col += ColNr::try_from(with_cursor_rest(cluster_len))
                    .unwrap_or(ColNr::MAX)
                    .get();
            }
        }
        n -= 1;
    }
    if n != count1 {
        may_fold_open(cmd_arg, kOptFdoFlagHor as c_uint);
    }
}

/// `h`, `<BS>`, CTRL-H and `<Left>`.
pub(crate) fn nv_left(cmd_arg: &mut CmdArg) {
    let mut win = Win::current();
    // A modifier turns this into a word move.
    if mod_mask.get().has(ModMask::SHIFT | ModMask::CTRL) {
        if mod_mask.get().has(ModMask::CTRL) {
            cmd_arg.arg = 1;
        }
        nv_bck_word(cmd_arg);
        return;
    }
    cmd_arg.op().motion_type = MT_CHAR_WISE;
    cmd_arg.op().inclusive = false;

    // Which 'whichwrap' flag lets this key wrap to the previous line.
    let wrap_flag = if cmd_arg.cmdchar == SpecialKey::Bs || cmd_arg.cmdchar == Key::CTRL_H {
        b'b'
    } else if cmd_arg.cmdchar.is('h') {
        b'h'
    } else if cmd_arg.cmdchar == SpecialKey::Left {
        b'<'
    } else {
        0
    };

    let mut n = cmd_arg.count1;
    while n > 0 {
        if oneleft().is_err() {
            if wrap_flag != 0
                && p_ww(|value| value.to_bytes().contains(&wrap_flag))
                && win.w_cursor.lnum > LineNr::ONE
            {
                win.w_cursor.lnum -= 1;
                coladvance(win, ColNr::MAX);
                win.w_set_curswant = true;
                // A delete or a change that wrapped back over the line
                // break must take the break with it, so put the cursor
                // one past the last character and tell the caller not to
                // pull it back.
                if (cmd_arg.op().op_type == OpType::Delete
                    || cmd_arg.op().op_type == OpType::Change)
                    && !Buf::current().lines().line(win.w_cursor.lnum).is_empty()
                {
                    win.w_cursor.col += ColNr::try_from(with_cursor_rest(cluster_len))
                        .unwrap_or(ColNr::MAX)
                        .get();
                    cmd_arg.outcome |= Outcome::NO_ADJ_OP_END;
                }
            } else {
                if cmd_arg.op().op_type == OpType::Nop && n == cmd_arg.count1 {
                    beep_flush();
                }
                break;
            }
        }
        n -= 1;
    }
    if n != cmd_arg.count1 {
        may_fold_open(cmd_arg, kOptFdoFlagHor as c_uint);
    }
}

/// `k`, `CTRL-P`, `-` and `<Up>`. Shifted, it is a page up.
pub(crate) fn nv_up(cmd_arg: &mut CmdArg) {
    if mod_mask.get().has(ModMask::SHIFT) {
        cmd_arg.arg = BACKWARD as c_int;
        nv_page(cmd_arg);
        return;
    }
    cmd_arg.op().motion_type = MT_LINE_WISE;
    if cursor_up(cmd_arg.count1, cmd_arg.op().op_type == OpType::Nop).is_err() {
        clear_op_beep(cmd_arg.op());
    } else if cmd_arg.arg != 0 {
        // `-` and `CTRL-P` land on the first non-blank; `k` does not.
        beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    }
}

/// `j`, `CTRL-N`, `+`, `<CR>` and `<Down>`. Shifted, it is a page down.
pub(crate) fn nv_down(cmd_arg: &mut CmdArg) {
    if mod_mask.get().has(ModMask::SHIFT) {
        cmd_arg.arg = FORWARD as c_int;
        nv_page(cmd_arg);
        return;
    }
    // In three kinds of window `<CR>` means "act on this line" rather
    // than "move down".
    if cmd_arg.cmdchar == Key::CAR {
        if buf_is_quickfix(current_buf()) {
            qf_view_result(false);
            return;
        }
        if cmdwin_type.get() != 0 {
            cmdwin_result.set(Key::CAR);
            return;
        }
        if buf_is_prompt(current_buf())
            && Win::current().w_cursor.lnum == Buf::current().b_ml.ml_line_count
        {
            prompt_invoke_callback();
            if restart_edit.get().is_nul() {
                restart_edit.set(Key::ascii(b'a'));
            }
            return;
        }
    }
    cmd_arg.op().motion_type = MT_LINE_WISE;
    if cursor_down(cmd_arg.count1, cmd_arg.op().op_type == OpType::Nop).is_err() {
        clear_op_beep(cmd_arg.op());
    } else if cmd_arg.arg != 0 {
        // `+`, `<CR>` and `CTRL-N` land on the first non-blank; `j` does
        // not.
        beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    }
}

/// `<End>`: the end of the line -- of the last line with CTRL, which is what
/// the argument says.
pub(crate) fn nv_end(cmd_arg: &mut CmdArg) {
    if cmd_arg.arg != 0 || mod_mask.get().has(ModMask::CTRL) {
        cmd_arg.arg = 1;
        nv_goto(cmd_arg);
        // The count named the line, so `$` must not use it again.
        cmd_arg.count1 = 1;
    }
    nv_dollar(cmd_arg);
}

/// `$`: the end of the line, `count1 - 1` lines down.
pub(crate) fn nv_dollar(cmd_arg: &mut CmdArg) {
    cmd_arg.op().motion_type = MT_CHAR_WISE;
    cmd_arg.op().inclusive = true;
    // Under 'virtualedit' an operator that starts past the end of the
    // line keeps the column it has rather than asking for the end again.
    if !virtual_active(Win::current())
        || gchar_cursor() != NUL
        || cmd_arg.op().op_type == OpType::Nop
    {
        Win::current().w_curswant = ColNr::MAX;
    }
    if cursor_down(cmd_arg.count1 - 1, cmd_arg.op().op_type == OpType::Nop).is_err() {
        clear_op_beep(cmd_arg.op());
    } else {
        may_fold_open(cmd_arg, kOptFdoFlagHor as c_uint);
    }
}

/// `f`, `F`, `t`, `T`, `;` and `,`: search this line for a character.
pub(crate) fn nv_csearch(cmd_arg: &mut CmdArg) {
    // An exclusive Select-mode selection was widened by one when it was
    // made; the search has to run against the real cursor position.
    let mut cursor_dec = false;
    if P_SEL.first_byte() == b'e'
        && visual_active()
        && visual_mode().is_char()
        && VIsual_select_exclu_adj.get()
    {
        unadjust_for_sel();
        cursor_dec = true;
    }
    // `t` and `T` stop *before* the character.
    let t_cmd = cmd_arg.cmdchar.is('t') || cmd_arg.cmdchar.is('T');
    cmd_arg.op().motion_type = MT_CHAR_WISE;
    if cmd_arg.nchar.is_special() || searchc(cmd_arg, t_cmd).is_err() {
        clear_op_beep(cmd_arg.op());
        if cursor_dec {
            adjust_for_sel(cmd_arg);
        }
        return;
    }
    Win::current().w_set_curswant = true;
    // Landing on a TAB with 'virtualedit' means the *last* cell of it,
    // so that `dt<Tab>` takes the whole tab.
    if gchar_cursor() == TAB
        && virtual_active(Win::current())
        && cmd_arg.arg == FORWARD as c_int
        && (t_cmd || cmd_arg.op().op_type != OpType::Nop)
    {
        let win = Win::current();
        let (scol, ecol) = win.vcol_span(win.cursor());
        Win::current().w_cursor.coladd = ecol - scol;
    } else {
        Win::current().w_cursor.coladd = 0;
    }
    adjust_for_sel(cmd_arg);
    may_fold_open(cmd_arg, kOptFdoFlagHor as c_uint);
}

/// `%`: to the matching bracket, or with a count to that percentage of the
/// file.
pub(crate) fn nv_percent(cmd_arg: &mut CmdArg) {
    let count0 = cmd_arg.count0;
    let mut op = cmd_arg.op();
    let mut win = Win::current();
    let lnum = win.w_cursor.lnum;
    op.inclusive = true;
    if count0 != 0 {
        if count0 > 100 {
            clear_op_beep(op);
        } else {
            op.motion_type = MT_LINE_WISE;
            setpcmark();
            let line_count = Buf::current().b_ml.ml_line_count;
            let count = line_count.get();
            // Divide first for a file long enough that `count * 100`
            // would not fit.
            win.w_cursor.lnum = LineNr::new(if count >= 21474836 {
                (count + 99) / 100 * count0
            } else {
                (count * count0 + 99) / 100
            });
            win.w_cursor.lnum = win.w_cursor.lnum.max(LineNr::ONE).min(line_count);
            beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
        }
    } else {
        op.motion_type = MT_CHAR_WISE;
        op.use_reg_one = true;
        let pos = findmatch(Some(&mut op), NUL);
        if let Some(pos) = pos {
            setpcmark();
            win.w_cursor = pos;
            win.w_set_curswant = true;
            win.w_cursor.coladd = 0;
            adjust_for_sel(cmd_arg);
        } else {
            clear_op_beep(op);
        }
    }
    if lnum != win.w_cursor.lnum {
        may_fold_open(cmd_arg, kOptFdoFlagPercent as c_uint);
    }
}

/// `(` and `)`: back and forward a sentence.
pub(crate) fn nv_brace(cmd_arg: &mut CmdArg) {
    cmd_arg.op().motion_type = MT_CHAR_WISE;
    cmd_arg.op().use_reg_one = true;
    cmd_arg.op().inclusive = false;
    Win::current().w_set_curswant = true;
    if findsent(cmd_arg.arg as Direction, cmd_arg.count1).is_err() {
        clear_op_beep(cmd_arg.op());
        return;
    }
    adjust_cursor(cmd_arg.op());
    Win::current().w_cursor.coladd = 0;
    may_fold_open(cmd_arg, kOptFdoFlagBlock as c_uint);
}

/// `{` and `}`: back and forward a paragraph.
pub(crate) fn nv_findpar(cmd_arg: &mut CmdArg) {
    cmd_arg.op().motion_type = MT_CHAR_WISE;
    cmd_arg.op().inclusive = false;
    cmd_arg.op().use_reg_one = true;
    Win::current().w_set_curswant = true;
    let Some(inclusive) = findpar(cmd_arg.arg, cmd_arg.count1, NUL, false) else {
        clear_op_beep(cmd_arg.op());
        return;
    };
    if inclusive {
        cmd_arg.op().inclusive = true;
    }
    Win::current().w_cursor.coladd = 0;
    may_fold_open(cmd_arg, kOptFdoFlagBlock as c_uint);
}

/// `<Home>`: the first column -- the first line with CTRL.
pub(crate) fn nv_home(cmd_arg: &mut CmdArg) {
    if mod_mask.get().has(ModMask::CTRL) {
        nv_goto(cmd_arg);
    } else {
        // `<Home>` is `1|`.
        cmd_arg.count0 = 1;
        nv_pipe(cmd_arg);
    }
    ins_at_eol.set(false);
}

/// `|`: to a screen column.
pub(crate) fn nv_pipe(cmd_arg: &mut CmdArg) {
    cmd_arg.op().motion_type = MT_CHAR_WISE;
    cmd_arg.op().inclusive = false;
    beginline(BeginlineOpts::NONE);
    if cmd_arg.count0 > 0 {
        coladvance(Win::current(), ColNr::new(cmd_arg.count0 - 1));
        Win::current().w_curswant = ColNr::new(cmd_arg.count0 - 1);
    } else {
        Win::current().w_curswant = ColNr::ZERO;
    }
    // The column was named outright, so it is not a remembered want.
    Win::current().w_set_curswant = false;
}

/// `b` and `B`: back a word.
pub(crate) fn nv_bck_word(cmd_arg: &mut CmdArg) {
    cmd_arg.op().motion_type = MT_CHAR_WISE;
    cmd_arg.op().inclusive = false;
    Win::current().w_set_curswant = true;
    if bck_word(cmd_arg.count1, cmd_arg.arg != 0, false).is_err() {
        clear_op_beep(cmd_arg.op());
    } else {
        may_fold_open(cmd_arg, kOptFdoFlagHor as c_uint);
    }
}

/// `w`, `W`, `e` and `E`: forward a word, or to a word's end.
pub(crate) fn nv_wordcmd(cmd_arg: &mut CmdArg) {
    let startpos = Win::current().w_cursor;
    let mut word_end = cmd_arg.cmdchar.is('e') || cmd_arg.cmdchar.is('E');
    cmd_arg.op().inclusive = word_end;

    // `cw` on a non-blank is `ce`: it changes the word, not up to the next
    // one. 'cpoptions' with `_` extends that to trailing white space.
    let mut cw_on_word = false;
    if !word_end && cmd_arg.op().op_type == OpType::Change {
        let c = gchar_cursor();
        if c != NUL && !ascii_iswhite(c) {
            if cpo_has(CpoFlag::CHANGEW) {
                cmd_arg.op().inclusive = true;
                word_end = true;
            }
            cw_on_word = true;
        }
    }

    cmd_arg.op().motion_type = MT_CHAR_WISE;
    Win::current().w_set_curswant = true;
    let moved = if word_end {
        end_word(cmd_arg.count1, cmd_arg.arg != 0, cw_on_word, false)
    } else {
        fwd_word(
            cmd_arg.count1,
            cmd_arg.arg != 0,
            cmd_arg.op().op_type != OpType::Nop,
        )
    };
    if lt(startpos, Win::current().w_cursor) {
        adjust_cursor(cmd_arg.op());
    }
    if moved.is_err() && cmd_arg.op().op_type == OpType::Nop {
        clear_op_beep(cmd_arg.op());
    } else {
        adjust_for_sel(cmd_arg);
        may_fold_open(cmd_arg, kOptFdoFlagHor as c_uint);
    }
}

/// Pull the cursor back off the line's terminator, which is not a position an
/// operator may include -- and say the operator now covers the last character.
pub(crate) fn adjust_cursor(mut op: Op) {
    if Win::current().w_cursor.col > ColNr::ZERO
        && gchar_cursor() == NUL
        && (!visual_active() || P_SEL.first_byte() == b'o')
        && !virtual_active(Win::current())
        && get_ve_flags(Win::current()) & kOptVeFlagOnemore as c_uint == 0
    {
        Win::current().w_cursor.col -= 1;
        mb_adjust_cursor();
        op.inclusive = true;
    }
}

/// `0` and `^`: the first column, or the first non-blank, which is what the
/// argument says.
pub(crate) fn nv_beginline(cmd_arg: &mut CmdArg) {
    cmd_arg.op().motion_type = MT_CHAR_WISE;
    cmd_arg.op().inclusive = false;
    beginline(BeginlineOpts::from_bits(cmd_arg.arg));
    may_fold_open(cmd_arg, kOptFdoFlagHor as c_uint);
    ins_at_eol.set(false);
}

/// `gg` and `G`: to the first or last line, or to the count'th.
pub(crate) fn nv_goto(cmd_arg: &mut CmdArg) {
    let last = Buf::current().b_ml.ml_line_count;
    let mut lnum = if cmd_arg.arg != 0 { last } else { LineNr::ONE };
    cmd_arg.op().motion_type = MT_LINE_WISE;
    setpcmark();
    if cmd_arg.count0 != 0 {
        lnum = LineNr::new(cmd_arg.count0);
    }
    Win::current().w_cursor.lnum = lnum.max(LineNr::ONE).min(last);
    beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    may_fold_open(cmd_arg, kOptFdoFlagJump as c_uint);
}

/// Whether `lnum` is inside a closed fold of `window`.
fn folded(window: Win, lnum: LineNr) -> bool {
    // Both fold ends are unwanted.
    window.has_folding(lnum, None, None)
}
