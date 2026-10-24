//! `win_line`'s setup half: everything decided once for a buffer line, before
//! the character loop starts.
//!
//! Drawing one screen line is two jobs. This module is the first: work out
//! *what* the line is — which text, starting at which virtual column, with
//! which highlighting sources active — and leave the character loop with
//! nothing to decide per cell that could have been decided per line. The
//! answers arrive in two places: the shared [`WinLineVars`], which the column
//! and virtual-text drawers also read, and [`LineSetup`], which is the
//! character loop's own half of the contract.
//!
//! The order things happen in is load-bearing beyond the obvious data
//! dependencies: `win_hl_attr` hands out attribute ids as it is asked for
//! them, so a highlight looked up out of order gets a different id and the
//! whole grid's attribute numbering shifts. Every lookup here is where
//! upstream put it.

#![forbid(unsafe_code)]

use super::*;
use crate::decoration::kMTMetaInline;
use crate::normal::{VisualSelection, visual_active, visual_selection};
use crate::option::vars::P_SEL;
use crate::optionstr::OptString;
use crate::pos::MAXCOL;
use crate::spell::SMT_ALL;
use crate::types::NUL;

/// Work out everything about `wlv.lnum` that does not depend on which cell is
/// being drawn, and leave `wlv` ready for the character loop.
///
/// `col_rows` is non-zero when only the columns left of the text are being
/// redrawn, which skips most of this; `concealed` says the line is hidden
/// behind a decoration.
///
/// `wlv` must have been initialised for this line (`lnum`, `foldinfo`,
/// `startrow`).
pub(crate) fn prepare_line(
    wlv: &mut WinLineVars,
    window: Win,
    endrow: ::core::ffi::c_int,
    col_rows: ::core::ffi::c_int,
    concealed: bool,
    spv: &mut SpellVars,
    nextline: &mut SpellLookahead,
) -> LineSetup {
    let lnum = wlv.lnum;
    let mut s = LineSetup::new(window, wlv, concealed);

    if col_rows == 0 && s.draw_text {
        // `extra_check` is the character loop's "nothing here needs the
        // slow path" test; every source of per-character work sets it.
        s.extra_check = window.w_onebuf_opt.wo_lbr != 0;
        s.start_syntax(window, lnum);
        s.check_decor_providers = true;

        // 'colorcolumn'; a terminal buffer never shows one.
        wlv.color_col = if window.w_buffer.terminal.is_null() && !window.w_p_cc_cols.is_empty() {
            0
        } else {
            NO_COLOR_COL
        };
        wlv.advance_color_col(window, wlv.vcol - wlv.vcol_off_co);

        if window.w_buffer == Win::current().w_buffer
            && let Some(sel) = visual_selection()
        {
            s.visual_area(wlv, window, sel);
        } else if highlight_match.get()
            && window.raw() == Win::current_raw()
            && !s.has_foldtext
            && lnum >= Win::current().w_cursor.lnum
            && lnum <= Win::current().w_cursor.lnum + search_match_lines.get()
        {
            s.incsearch_area(wlv, window);
        }
    }

    s.bg_attr = win_bg_attr(window);
    s.diff_state(wlv, window);
    s.filler_lines(wlv, window);
    s.cursorline(wlv, window);
    s.signs_and_statuscolumn(wlv, window);
    s.line_attr_save = wlv.line_attr;
    s.line_attr_lowprio_save = wlv.line_attr_lowprio;

    if s.draw_text {
        s.line.fill(window.buffer(), lnum);
    }
    if spv.spv_has_spell && col_rows == 0 && s.draw_text {
        s.spell_line_start(window, lnum, spv, nextline);
    }

    s.lcs_eol = window.w_p_lcs_chars.eol;
    s.lcs_prec_todo = window.w_p_lcs_chars.prec;
    if window.w_onebuf_opt.wo_list != 0 && !s.has_foldtext && s.draw_text {
        s.listchars_columns(window);
    }

    // 'nowrap', or 'wrap' with a line scrolled sideways: advance to the
    // first character that is on screen.
    s.start_vcol = if window.w_onebuf_opt.wo_wrap != 0 {
        if wlv.startrow == 0 {
            window.w_skipcol
        } else {
            0
        }
    } else {
        window.w_leftcol
    };
    if s.has_foldtext {
        wlv.vcol = s.start_vcol;
    } else if s.start_vcol > 0 && col_rows == 0 {
        s.skip_to_start_vcol(wlv, window, spv);
    }

    if s.check_decor_providers {
        let at = ColNr::try_from(s.pos).unwrap_or(ColNr::MAX);
        s.decor_provider_end_col =
            decor_providers_setup(endrow - wlv.startrow, s.start_vcol == 0, lnum, at, window);
        // A provider is Lua and may have changed the buffer under us.
        s.line.fill(window.buffer(), lnum);
    }

    decor_redraw_line(window, lnum - 1);
    if !s.has_decor && decor_has_more_decorations(lnum - 1) {
        s.has_decor = true;
        s.extra_check = true;
    }

    s.keep_cursor_visible(wlv, window);

    if col_rows == 0 && s.draw_text && !s.has_foldtext {
        let at = ColNr::try_from(s.pos).unwrap_or(ColNr::MAX);
        // `|=`, not `||`: `prepare_search_hl_line` runs either way. A
        // multi-line regexp may move the memline's line, but not the copy.
        s.area_highlighting |= prepare_search_hl_line(
            window,
            lnum,
            at,
            &mut s.search_attr,
            &mut s.search_attr_from_match,
        );
    }

    // Insert-mode completion highlights the text it inserted.
    if State.get() & MODE_INSERT != 0
        && ins_compl_win_active(window)
        && (s.in_curline || ins_compl_lnum_in_range(lnum))
    {
        s.area_highlighting = true;
    }

    wlv.start_line(window);

    // The `:terminal` attributes themselves are filled in by the caller:
    // see [`LineSetup::has_terminal`].
    if !window.w_buffer.terminal.is_null() {
        s.has_terminal = true;
        s.extra_check = true;
    }
    s.may_have_inline_virt = !s.has_foldtext && buf_meta_total(window.buffer(), kMTMetaInline) > 0;

    s
}

impl LineSetup {
    /// The facts about the window and the line everything else is decided
    /// against, plus "nothing found yet" for the rest.
    fn new(window: Win, wlv: &WinLineVars, concealed: bool) -> Self {
        let has_fold = wlv.foldinfo.fi_level != 0 && wlv.foldinfo.fi_lines > 0;
        let has_foldtext = has_fold && window.w_onebuf_opt.wo_fdt.first_byte() != 0;
        LineSetup {
            // First, because `win_hl_attr` hands out attribute ids in the
            // order it is asked for them.
            conceal_attr: win_hl_attr(window, HLF_CONCEAL),
            view_width: window.w_view_width,
            view_height: window.w_view_height,
            in_curline: window.raw() == Win::current_raw()
                && wlv.lnum == Win::current().w_cursor.lnum,
            has_fold,
            has_foldtext,
            is_wrapped: window.w_onebuf_opt.wo_wrap != 0 && !has_fold,
            // The line one past the end of the buffer exists only to carry
            // the filler lines below the last one.
            draw_text: !concealed && wlv.lnum != window.w_buffer.b_ml.ml_line_count + 1,
            start_vcol: 0,
            bg_attr: 0,
            may_have_inline_virt: false,
            has_terminal: false,

            line: HeldLine::take(),
            pos: 0,
            trailcol: MAXCOL as ColNr,
            leadcol: 0,
            lcs_eol: 0,
            lcs_prec_todo: 0,
            in_multispace: false,
            multispace_pos: 0,

            area_highlighting: false,
            extra_check: false,
            has_syntax: false,
            has_decor: false,
            vi_attr: 0,
            search_attr: 0,
            search_attr_from_match: false,
            noinvcur: false,
            fromcol_prev: -2,
            lnum_in_visual_area: false,
            cul_screenline: false,
            left_curline_col: 0,
            right_curline_col: 0,
            line_attr_save: 0,
            line_attr_lowprio_save: 0,

            line_changes: DiffLine::default(),
            change_index: -1,
            change_start: MAXCOL as ::core::ffi::c_int,
            change_end: -1,

            statuscol: StatusCol::default(),
            virt_lines: ManuallyDrop::new(Vec::new()),
            check_decor_providers: false,
            decor_provider_end_col: 0,

            nextlinecol: 0,
            nextline_idx: 0,
            spell_attr: 0,
            word_end: 0,
            cur_checked_col: 0,
        }
    }

    /// Start syntax highlighting for this line, unless the buffer's syntax has
    /// already gone wrong or has been found too slow.
    ///
    /// An error raised while parsing disables syntax for the buffer rather
    /// than being reported once per redraw.
    fn start_syntax(&mut self, mut window: Win, lnum: LineNr) {
        if !syntax_present(window)
            || window.syn_block().b_syn_error
            || window.syn_block().b_syn_slow
            || self.has_foldtext
        {
            return;
        }
        let save_did_emsg = did_emsg.get();
        did_emsg.set(0);
        window.syntax_start(lnum);
        if did_emsg.get() != 0 {
            window.syn_block_mut().b_syn_error = true;
        } else {
            did_emsg.set(save_did_emsg);
            if !window.syn_block().b_syn_slow {
                self.has_syntax = true;
                self.extra_check = true;
            }
        }
    }

    /// The inverted range for an active Visual selection.
    fn visual_area(&mut self, wlv: &mut WinLineVars, window: Win, sel: VisualSelection) {
        let lnum = wlv.lnum;
        // Both ends by value: nothing here writes through either.
        let cursor = Win::current().w_cursor;
        let (mut top, bot) = if ltoreq(cursor, sel.anchor) {
            (cursor, sel.anchor)
        } else {
            (sel.anchor, cursor)
        };
        self.lnum_in_visual_area = lnum >= top.lnum && lnum <= bot.lnum;
        if sel.mode.is_block() {
            // Blockwise: the columns were worked out for the whole
            // selection when it last moved.
            if self.lnum_in_visual_area {
                wlv.fromcol = window.w_old_cursor_fcol;
                wlv.tocol = window.w_old_cursor_lcol;
            }
        } else {
            if lnum > top.lnum && lnum <= bot.lnum {
                wlv.fromcol = 0;
            } else if lnum == top.lnum {
                if sel.mode.is_line() {
                    wlv.fromcol = 0;
                } else {
                    wlv.fromcol = getvvcol(window, &mut top).0;
                    if gchar_pos(&top) == NUL {
                        // Empty line: invert the one cell past its end.
                        wlv.tocol = wlv.fromcol + 1;
                    }
                }
            }
            if !sel.mode.is_line() && lnum == bot.lnum {
                if P_SEL.first_byte() == b'e' && bot.col == 0 && bot.coladd == 0 {
                    // 'selection' "exclusive" and the selection stops at
                    // the start of this line: none of it is here.
                    wlv.fromcol = -10;
                    wlv.tocol = MAXCOL as ::core::ffi::c_int;
                } else if bot.col == MAXCOL as ColNr {
                    wlv.tocol = MAXCOL as ::core::ffi::c_int;
                } else {
                    let mut pos = bot;
                    if P_SEL.first_byte() == b'e' {
                        wlv.tocol = getvvcol(window, &mut pos).0;
                    } else {
                        wlv.tocol = getvvcol(window, &mut pos).2 + 1;
                    }
                }
            }
        }

        // The character under the cursor is inverted only if the cursor is
        // drawn as a block inside the selection anyway.
        if !highlight_match.get()
            && self.in_curline
            && cursor_is_block_during_visual(P_SEL.first_byte() == b'e')
        {
            self.noinvcur = true;
        }

        if wlv.fromcol >= 0 {
            self.area_highlighting = true;
            self.vi_attr = win_hl_attr(window, HLF_V);
        }
    }

    /// The inverted range for `'incsearch'` and `:s///c`.
    fn incsearch_area(&mut self, wlv: &mut WinLineVars, window: Win) {
        let lnum = wlv.lnum;
        if lnum == Win::current().w_cursor.lnum {
            let win = Win::current();
            wlv.fromcol = win.vcol_triple(win.cursor()).0;
        } else {
            wlv.fromcol = 0;
        }
        if lnum == Win::current().w_cursor.lnum + search_match_lines.get() {
            let mut pos = Pos {
                lnum,
                col: search_match_endcol.get(),
                coladd: 0,
            };
            wlv.tocol = getvcol(Win::current(), &mut pos).0;
        }
        // Do at least one character; the match can be past the end of the
        // line.
        if wlv.fromcol == wlv.tocol && search_match_endcol.get() != 0 {
            wlv.tocol = wlv.fromcol + 1;
        }
        self.area_highlighting = true;
        self.vi_attr = win_hl_attr(window, HLF_I);
    }

    /// Diff-mode state for this line: how many filler lines it needs above it
    /// and which diff highlight its text takes.
    fn diff_state(&mut self, wlv: &mut WinLineVars, window: Win) {
        let (filler_lines, linestatus) = diff_check_with_linestatus(window, wlv.lnum);
        wlv.filler_lines = filler_lines;
        if linestatus >= 0 {
            return;
        }
        // An added line, either because the status says so or because
        // the change scan found nothing to narrow it to.
        if linestatus != -1 || diff_find_change(window, wlv.lnum, &mut self.line_changes) {
            wlv.diff_hlf = HLF_ADD;
        } else if self.line_changes.num_changes() > 0 {
            let (start, end, added) = self.line_changes.change(0);
            self.change_start = start;
            self.change_end = end;
            wlv.diff_hlf = if self.change_start != 0 {
                HLF_CHD // unchanged text on a changed line
            } else if added {
                HLF_TXA // added text on a changed line
            } else {
                HLF_TXD // changed text on a changed line
            };
            self.change_index = 0;
        } else {
            wlv.diff_hlf = HLF_CHD; // changed line
            self.change_index = 0;
        }
        self.area_highlighting = true;
    }

    /// Count the filler lines above this one — diff filler plus virtual lines.
    fn filler_lines(&mut self, wlv: &mut WinLineVars, window: Win) {
        wlv.n_virt_lines = decor_virt_lines(
            window,
            wlv.lnum - 1,
            wlv.lnum,
            Some(&mut wlv.n_virt_below),
            Some(&mut self.virt_lines),
            true,
        );
        wlv.filler_lines += wlv.n_virt_lines;
        if wlv.lnum == window.w_topline {
            // The top line shows only as much filler as it is scrolled to.
            wlv.filler_lines = window.w_topfill;
            wlv.n_virt_lines = wlv.n_virt_lines.min(wlv.filler_lines);
        }
        wlv.filler_todo = wlv.filler_lines;
    }

    /// Apply `'cursorline'` to this line, if it is the cursor's.
    fn cursorline(&mut self, wlv: &mut WinLineVars, window: Win) {
        if window.w_onebuf_opt.wo_cul == 0
            || window.w_p_culopt_flags as ::core::ffi::c_int
                == kOptCuloptFlagNumber as ::core::ffi::c_int
            || wlv.lnum != window.w_cursorline
            // Not while Visual mode is active: it would stop being clear
            // what is selected.
            || (window.raw() == Win::current_raw() && visual_active())
        {
            return;
        }
        self.cul_screenline = self.is_wrapped
            && window.w_p_culopt_flags as ::core::ffi::c_int
                & kOptCuloptFlagScreenline as ::core::ffi::c_int
                != 0;
        if self.cul_screenline {
            // Only the cursor's own screen row is highlighted, so the loop
            // needs that row's margins.
            (self.left_curline_col, self.right_curline_col) = window.cursorline_margins();
        } else {
            wlv.apply_cursorline_highlight(window);
        }
        self.area_highlighting = true;
    }

    /// Collect the signs on this line, and either build the `'statuscolumn'`
    /// request or resolve the sign highlights the number column will use.
    fn signs_and_statuscolumn(&mut self, wlv: &mut WinLineVars, window: Win) {
        let mut sign_line_attr = 0;
        // TODO(bfredl, vigoux): line_attr should not take priority over
        // decoration.
        decor_redraw_signs(
            window,
            window.buffer(),
            wlv.lnum - 1,
            Some(&mut wlv.sign_attrs),
            Some(&mut sign_line_attr),
            Some(&mut wlv.sign_cul_attr),
            Some(&mut wlv.sign_num_attr),
        );

        if window.w_onebuf_opt.wo_stc.first_byte() != 0 {
            // 'statuscolumn' replaces the fold, sign and number columns;
            // the expression is evaluated per row by `draw_statuscol`.
            self.statuscol.draw = true;
            self.statuscol.lnum = wlv.lnum;
            self.statuscol.foldinfo = wlv.foldinfo;
            self.statuscol.width =
                window.col_off() - (cmdwin_win.get() == Some(window.id())) as ::core::ffi::c_int;
            self.statuscol.sign_cul_id = if use_cursor_line_highlight(window, wlv.lnum) {
                wlv.sign_cul_attr
            } else {
                0
            };
        } else if wlv.sign_cul_attr > 0 {
            wlv.sign_cul_attr = if use_cursor_line_highlight(window, wlv.lnum) {
                syn_id2attr(wlv.sign_cul_attr)
            } else {
                0
            };
        }
        if wlv.sign_num_attr > 0 {
            wlv.sign_num_attr = syn_id2attr(wlv.sign_num_attr);
        }
        if sign_line_attr > 0 {
            wlv.line_attr = syn_id2attr(sign_line_attr);
        }

        // The quickfix window highlights the entry the cursor is on.
        if window.shows_quickfix_buffer() && window.quickfix_current_line() == wlv.lnum {
            wlv.line_attr = win_hl_attr(window, HLF_QFL);
        }
        if wlv.line_attr_lowprio != 0 || wlv.line_attr != 0 {
            self.area_highlighting = true;
        }
    }

    /// Prepare the spell checker for this line: decide where a capital is
    /// required, and join the tail of this line to the start of the next one
    /// so that a word wrapping across the break ("et<line-break>al.") is still
    /// seen whole.
    ///
    fn spell_line_start(
        &mut self,
        window: Win,
        lnum: LineNr,
        spv: &mut SpellVars,
        nextline: &mut SpellLookahead,
    ) {
        self.extra_check = true;

        // A word wrapped from the previous line leaves the start of this
        // one already checked.
        if lnum == spv.spv_checked_lnum {
            self.cur_checked_col = spv.spv_checked_col;
        }
        // The previous line was not spell checked — the first line of an
        // updated region, or the line after a closed fold — so this one
        // has to decide for itself whether a capital is required.
        if spv.spv_capcol_lnum == 0 && check_need_cap(window, lnum, 0) {
            spv.spv_cap_col = 0;
        } else if lnum != spv.spv_capcol_lnum {
            spv.spv_cap_col = -1;
        }
        spv.spv_checked_lnum = 0;

        // Trick: `spell_cat_line` skips a few characters for C/shell/Vim
        // comment leaders.
        nextline[SPELL_LOOKAHEAD] = 0;
        if lnum < window.w_buffer.b_ml.ml_line_count {
            let mut lines = window.buffer().lines();
            let next = lines.line(lnum + 1);
            spell_cat_line(&mut nextline[SPELL_LOOKAHEAD..], next, SPWORDLEN);
        }
        let line = self.line.text();

        // An empty line: check the first word of the next one for a
        // capital instead.
        let first = line
            .iter()
            .take_while(|&&c| c == b' ' || c == b'\t')
            .count();
        if first == line.len() {
            spv.spv_cap_col = 0;
            spv.spv_capcol_lnum = lnum + 1;
        } else if spv.spv_cap_col == 0 {
            spv.spv_cap_col =
                ::core::ffi::c_int::try_from(first).unwrap_or(::core::ffi::c_int::MAX);
        }

        if nextline[SPELL_LOOKAHEAD] == 0 {
            // No next line, or it is empty.
            self.nextlinecol = MAXCOL as ::core::ffi::c_int;
            self.nextline_idx = 0;
            return;
        }
        let line_len = line.len();
        if line_len < SPELL_LOOKAHEAD {
            // Short line: use all of it, then move the next line's start
            // up against it.
            let tail = nextline[SPELL_LOOKAHEAD..]
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(SPELL_LOOKAHEAD - 1);
            nextline[..line_len].copy_from_slice(line);
            nextline.copy_within(SPELL_LOOKAHEAD..SPELL_LOOKAHEAD + tail + 1, line_len);
            self.nextlinecol = 0;
            self.nextline_idx = line_len as ::core::ffi::c_int + 1;
        } else {
            // Long line: only its last `SPWORDLEN` bytes can matter.
            self.nextlinecol = (line_len - SPELL_LOOKAHEAD) as ::core::ffi::c_int;
            nextline[..SPELL_LOOKAHEAD].copy_from_slice(&line[line_len - SPELL_LOOKAHEAD..]);
            self.nextline_idx = SPWORDLEN + 1;
        }
    }

    /// Find the leading and trailing whitespace the `'listchars'` "lead" and
    /// "trail" marks apply to.
    fn listchars_columns(&mut self, window: Win) {
        if window.w_p_lcs_chars.space != 0
            || window.w_p_lcs_chars.multispace.is_some()
            || window.w_p_lcs_chars.leadmultispace.is_some()
            || window.w_p_lcs_chars.trail != 0
            || window.w_p_lcs_chars.lead != 0
            || window.w_p_lcs_chars.nbsp != 0
        {
            self.extra_check = true;
        }
        if window.w_p_lcs_chars.trail != 0 {
            let rest = self.line.from(self.pos);
            let text = &rest[..rest.len().saturating_sub(1)];
            let trailcol = text.len()
                - text
                    .iter()
                    .rev()
                    .take_while(|&&c| ascii_iswhite(::core::ffi::c_int::from(c)))
                    .count();
            self.trailcol = ColNr::try_from(trailcol + self.pos).unwrap_or(ColNr::MAX);
        }
        if window.w_p_lcs_chars.lead != 0
            || window.w_p_lcs_chars.leadmultispace.is_some()
            || window.w_p_lcs_chars.leadtab1 != 0
        {
            let rest = self.line.from(self.pos);
            let leadcol = rest
                .iter()
                .take_while(|&&c| ascii_iswhite(::core::ffi::c_int::from(c)))
                .count();
            self.leadcol = if rest.get(leadcol).is_none_or(|&c| c == 0) {
                // In a line of nothing but spaces they all count as
                // trailing.
                0
            } else {
                // The first column not filled with spaces.
                ColNr::try_from(leadcol + self.pos + 1).unwrap_or(ColNr::MAX)
            };
        }
    }

    /// Advance [`LineSetup::pos`] and `wlv.vcol` to the first character that is
    /// on screen, when the line is scrolled sideways or `w_skipcol` is set.
    fn skip_to_start_vcol(&mut self, wlv: &mut WinLineVars, window: Win, spv: &mut SpellVars) {
        let start_vcol = self.start_vcol;
        let mut cs = CharSize { width: 0, head: 0 };
        let line = self.line.with_nul();
        let (mut csarg, cstype) = CharsizeArg::new(window, wlv.lnum, self.line.text());
        csarg.max_head_vcol = start_vcol;
        let mut vcol = wlv.vcol;
        let mut at = self.pos;
        let mut prev_at = at;
        let mut ci = c_char_info(line, at);
        let mut multispace = (self.in_multispace, self.multispace_pos);
        while vcol < start_vcol {
            cs = win_charsize(cstype, vcol, at, ci.value, &mut csarg);
            vcol += cs.width;
            prev_at = at;
            if line.get(prev_at).is_none_or(|&c| c == 0) {
                break;
            }
            (at, ci) = c_next_cluster(line, at, ci);
            if window.w_onebuf_opt.wo_list != 0 {
                multispace = track_multispace(window, line, self.leadcol, multispace, prev_at, at);
            }
        }
        drop(csarg);
        (self.in_multispace, self.multispace_pos) = multispace;
        wlv.vcol = vcol;
        self.pos = at;

        // The end of the line can be left of the first displayed column
        // when 'cursorcolumn' or 'colorcolumn' is set, when 'virtualedit'
        // or Visual mode is active, or when a fold is being drawn — all of
        // which still have something to draw out there.
        if wlv.vcol < start_vcol
            && (window.w_onebuf_opt.wo_cuc != 0
                || wlv.color_col != NO_COLOR_COL
                || virtual_active(window)
                || (visual_active() && window.w_buffer == Win::current().w_buffer)
                || self.has_fold)
        {
            wlv.vcol = start_vcol;
        }

        // A character that is only partly on screen: stand on it, and skip
        // the cells of it that are not.
        if wlv.vcol > start_vcol {
            wlv.vcol -= cs.width;
            self.pos = prev_at;
        }
        if start_vcol > wlv.vcol {
            wlv.skip_cells = start_vcol - wlv.vcol - cs.head;
        }

        // Adjust for inverted text that is, or starts, left of the screen.
        if wlv.tocol <= wlv.vcol {
            wlv.fromcol = 0;
        } else if wlv.fromcol >= 0 && wlv.fromcol < wlv.vcol {
            wlv.fromcol = wlv.vcol;
        }

        // With a non-zero `w_skipcol` the first row still owes a
        // 'showbreak'.
        if window.w_onebuf_opt.wo_wrap != 0 {
            wlv.need_showbreak = true;
        }

        if spv.spv_has_spell {
            self.spell_at_start_vcol(window, wlv.lnum);
        }
    }

    /// Work out whether the first character on screen is inside a badly
    /// spelled word, since the loop only ever starts a spell check at a word
    /// boundary.
    fn spell_at_start_vcol(&mut self, mut window: Win, lnum: LineNr) {
        let linecol = ColNr::try_from(self.pos).unwrap_or(ColNr::MAX);
        let mut spell_hlf: Hlf = HLF_COUNT;

        let saved_cursor = window.w_cursor;
        window.w_cursor.lnum = lnum;
        window.w_cursor.col = linecol;
        // Reads lines of its own, which moves the memline's but not the copy.
        let len = spell_move_to(window, FORWARD, SMT_ALL, true, Some(&mut spell_hlf));

        if len == 0 || window.w_cursor.col > linecol {
            // No bad word at the line start: do not check again until the
            // end of a word.
            let end =
                spell_to_word_end(self.line.text().get(self.pos..).unwrap_or_default(), window);
            self.word_end = (linecol as usize + end + 1) as ::core::ffi::c_int;
        } else {
            // Bad word found: its attribute applies to the end of it.
            debug_assert!(len <= ::core::ffi::c_int::MAX as size_t);
            self.word_end = window.w_cursor.col + len as ::core::ffi::c_int + 1;
            if spell_hlf != HLF_COUNT {
                self.spell_attr = default_hl_attr(spell_hlf as usize);
            }
        }
        window.w_cursor = saved_cursor;

        // Syntax highlighting has to be restarted for this line.
        if self.has_syntax {
            window.syntax_start(lnum);
        }
    }

    /// Correct the inverted range so that it never swallows a cursor that has
    /// to stay visible.
    ///
    /// Doing it once here saves testing for it on every character.
    fn keep_cursor_visible(&mut self, wlv: &mut WinLineVars, window: Win) {
        if wlv.fromcol < 0 {
            return;
        }
        if self.noinvcur {
            if wlv.fromcol == window.w_virtcol {
                // Inverting starts at the cursor; start just after it.
                self.fromcol_prev = wlv.fromcol;
                wlv.fromcol = -1;
            } else if wlv.fromcol < window.w_virtcol {
                // Resume inverting after the cursor.
                self.fromcol_prev = window.w_virtcol;
            }
        }
        if wlv.fromcol >= wlv.tocol {
            wlv.fromcol = -1;
        }
    }
}

/// Track where in a run of consecutive spaces the skipped-over text has got
/// to, so that `'listchars'` "multispace" resumes at the right glyph.
///
/// `state` is (inside a run, how far into the pattern), `prev` the byte the
/// character just skipped starts at and `next` the next one's, both in
/// `line`, whose NUL is included.
fn track_multispace(
    window: Win,
    line: &[u8],
    leadcol: ColNr,
    state: (bool, ::core::ffi::c_int),
    prev: usize,
    next: usize,
) -> (bool, ::core::ffi::c_int) {
    let byte = |at: usize| line.get(at).copied().unwrap_or(0);
    let in_multispace =
        byte(prev) == b' ' && (byte(next) == b' ' || (prev > 0 && byte(prev - 1) == b' '));
    if !in_multispace {
        return (false, 0);
    }
    let lcs = &window.w_p_lcs_chars;
    let pattern = if next >= usize::try_from(leadcol).unwrap_or(0) {
        lcs.multispace.as_deref()
    } else {
        lcs.leadmultispace.as_deref()
    };
    let Some(pattern) = pattern else {
        return (true, state.1);
    };
    let mut pos = state.1 + 1;
    // Past the run's end is where upstream's terminator stops it.
    if usize::try_from(pos).is_ok_and(|pos| pos >= pattern.len()) {
        pos = 0;
    }
    (true, pos)
}
