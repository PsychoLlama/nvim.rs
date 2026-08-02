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

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

/// Work out everything about `wlv.lnum` that does not depend on which cell is
/// being drawn, and leave `wlv` ready for the character loop.
///
/// `col_rows` is non-zero when only the columns left of the text are being
/// redrawn, which skips most of this; `concealed` says the line is hidden
/// behind a decoration.
///
/// # Safety
/// `wp` must be a live window, `spv` a live `spellvars_T`, and `wlv` must have
/// been initialised for this line (`lnum`, `foldinfo`, `startrow`).
pub(crate) unsafe fn prepare_line(
    wlv: &mut WinLineVars,
    wp: *mut win_T,
    endrow: ::core::ffi::c_int,
    col_rows: ::core::ffi::c_int,
    concealed: bool,
    spv: *mut spellvars_T,
    scratch: &mut LineScratch,
) -> LineSetup {
    // SAFETY: the caller's window, spell state and line.
    unsafe {
        let lnum = wlv.lnum;
        let mut s = LineSetup::new(wp, wlv, concealed);

        if col_rows == 0 && s.draw_text {
            // `extra_check` is the character loop's "nothing here needs the
            // slow path" test; every source of per-character work sets it.
            s.extra_check = (*wp).w_onebuf_opt.wo_lbr != 0;
            s.start_syntax(wp, lnum);
            s.check_decor_providers = true;

            // 'colorcolumn'; a terminal buffer never shows one.
            wlv.color_cols = if (*(*wp).w_buffer).terminal.is_null() {
                (*wp).w_p_cc_cols
            } else {
                ::core::ptr::null_mut()
            };
            wlv.advance_color_col(wlv.vcol - wlv.vcol_off_co);

            if VIsual_active.get() && (*wp).w_buffer == (*curwin.get()).w_buffer {
                s.visual_area(wlv, wp);
            } else if highlight_match.get()
                && wp == curwin.get()
                && !s.has_foldtext
                && lnum >= (*curwin.get()).w_cursor.lnum
                && lnum <= (*curwin.get()).w_cursor.lnum + search_match_lines.get()
            {
                s.incsearch_area(wlv, wp);
            }
        }

        s.bg_attr = win_bg_attr(wp);
        s.diff_state(wlv, wp);
        s.filler_lines(wlv, wp);
        s.cursorline(wlv, wp);
        s.signs_and_statuscolumn(wlv, wp);
        s.line_attr_save = wlv.line_attr;
        s.line_attr_lowprio_save = wlv.line_attr_lowprio;

        if (*spv).spv_has_spell && col_rows == 0 && s.draw_text {
            s.spell_line_start(wp, lnum, spv, scratch);
        }

        s.line = if s.draw_text {
            ml_get_buf((*wp).w_buffer, lnum)
        } else {
            c"".as_ptr().cast_mut()
        };
        s.ptr = s.line;
        s.lcs_eol = (*wp).w_p_lcs_chars.eol;
        s.lcs_prec_todo = (*wp).w_p_lcs_chars.prec;
        if (*wp).w_onebuf_opt.wo_list != 0 && !s.has_foldtext && s.draw_text {
            s.listchars_columns(wp, lnum);
        }

        // 'nowrap', or 'wrap' with a line scrolled sideways: advance to the
        // first character that is on screen.
        s.start_vcol = if (*wp).w_onebuf_opt.wo_wrap != 0 {
            if wlv.startrow == 0 {
                (*wp).w_skipcol
            } else {
                0
            }
        } else {
            (*wp).w_leftcol
        };
        if s.has_foldtext {
            wlv.vcol = s.start_vcol;
        } else if s.start_vcol > 0 && col_rows == 0 {
            s.skip_to_start_vcol(wlv, wp, spv);
        }

        if s.check_decor_providers {
            let at = s.ptr.offset_from(s.line) as ::core::ffi::c_int;
            s.decor_provider_end_col =
                decor_providers_setup(endrow - wlv.startrow, s.start_vcol == 0, lnum, at, wp);
            // A provider is Lua and may have changed the buffer under us.
            s.line = ml_get_buf((*wp).w_buffer, lnum);
            s.ptr = s.line.offset(at as isize);
        }

        decor_redraw_line(wp, lnum - 1, decor_state.ptr());
        if !s.has_decor && decor_has_more_decorations(decor_state.ptr(), lnum - 1) {
            s.has_decor = true;
            s.extra_check = true;
        }

        s.keep_cursor_visible(wlv, wp);

        if col_rows == 0 && s.draw_text && !s.has_foldtext {
            let at = s.ptr.offset_from(s.line) as ::core::ffi::c_int;
            // `|=`, not `||`: `prepare_search_hl_line` runs either way.
            s.area_highlighting |= prepare_search_hl_line(
                wp,
                lnum,
                at,
                &raw mut s.line,
                screen_search_hl.ptr(),
                &raw mut s.search_attr,
                &raw mut s.search_attr_from_match,
            );
            // "line" may have been updated.
            s.ptr = s.line.offset(at as isize);
        }

        // Insert-mode completion highlights the text it inserted.
        if State.get() & MODE_INSERT != 0
            && ins_compl_win_active(wp)
            && (s.in_curline || ins_compl_lnum_in_range(lnum))
        {
            s.area_highlighting = true;
        }

        wlv.start_line(wp);

        if !(*(*wp).w_buffer).terminal.is_null() {
            terminal_get_line_attributes(
                (*(*wp).w_buffer).terminal,
                wp,
                lnum,
                scratch.term_attrs.as_mut_ptr(),
            );
            s.extra_check = true;
        }
        s.may_have_inline_virt =
            !s.has_foldtext && buf_meta_total((*wp).w_buffer, kMTMetaInline) > 0;

        s
    }
}

impl LineSetup {
    /// The facts about the window and the line everything else is decided
    /// against, plus "nothing found yet" for the rest.
    ///
    /// # Safety
    /// `wp` must be a live window.
    unsafe fn new(wp: *mut win_T, wlv: &WinLineVars, concealed: bool) -> Self {
        // SAFETY: the caller's window.
        unsafe {
            let has_fold = wlv.foldinfo.fi_level != 0 && wlv.foldinfo.fi_lines > 0;
            let has_foldtext = has_fold && *(*wp).w_onebuf_opt.wo_fdt != 0;
            LineSetup {
                // First, because `win_hl_attr` hands out attribute ids in the
                // order it is asked for them.
                conceal_attr: win_hl_attr(wp, HLF_CONCEAL),
                view_width: (*wp).w_view_width,
                view_height: (*wp).w_view_height,
                in_curline: wp == curwin.get() && wlv.lnum == (*curwin.get()).w_cursor.lnum,
                has_fold,
                has_foldtext,
                is_wrapped: (*wp).w_onebuf_opt.wo_wrap != 0 && !has_fold,
                // The line one past the end of the buffer exists only to carry
                // the filler lines below the last one.
                draw_text: !concealed && wlv.lnum != (*(*wp).w_buffer).b_ml.ml_line_count + 1,
                start_vcol: 0,
                bg_attr: 0,
                may_have_inline_virt: false,

                line: ::core::ptr::null_mut(),
                ptr: ::core::ptr::null_mut(),
                trailcol: MAXCOL as colnr_T,
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

                line_changes: diffline_T::default(),
                change_index: -1,
                change_start: MAXCOL as ::core::ffi::c_int,
                change_end: -1,

                statuscol: statuscol_T::default(),
                virt_lines: VirtLines {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut(),
                },
                check_decor_providers: false,
                decor_provider_end_col: 0,

                nextlinecol: 0,
                nextline_idx: 0,
                spell_attr: 0,
                word_end: 0,
                cur_checked_col: 0,
            }
        }
    }

    /// Start syntax highlighting for this line, unless the buffer's syntax has
    /// already gone wrong or has been found too slow.
    ///
    /// An error raised while parsing disables syntax for the buffer rather
    /// than being reported once per redraw.
    ///
    /// # Safety
    /// `wp` must be a live window.
    unsafe fn start_syntax(&mut self, wp: *mut win_T, lnum: linenr_T) {
        // SAFETY: the caller's window.
        unsafe {
            if !syntax_present(wp)
                || (*(*wp).w_s).b_syn_error
                || (*(*wp).w_s).b_syn_slow
                || self.has_foldtext
            {
                return;
            }
            let save_did_emsg = did_emsg.get();
            did_emsg.set(0);
            syntax_start(wp, lnum);
            if did_emsg.get() != 0 {
                (*(*wp).w_s).b_syn_error = true;
            } else {
                did_emsg.set(save_did_emsg);
                if !(*(*wp).w_s).b_syn_slow {
                    self.has_syntax = true;
                    self.extra_check = true;
                }
            }
        }
    }

    /// The inverted range for an active Visual selection.
    ///
    /// # Safety
    /// `wp` must be a live window showing the current buffer.
    unsafe fn visual_area(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        let lnum = wlv.lnum;
        // SAFETY: the caller's window.
        unsafe {
            let cursor = &raw mut (*curwin.get()).w_cursor;
            let (top, bot) = if ltoreq(*cursor, VIsual.get()) {
                (cursor, VIsual.ptr())
            } else {
                (VIsual.ptr(), cursor)
            };
            self.lnum_in_visual_area = lnum >= (*top).lnum && lnum <= (*bot).lnum;

            if VIsual_mode.get() == Ctrl_V {
                // Blockwise: the columns were worked out for the whole
                // selection when it last moved.
                if self.lnum_in_visual_area {
                    wlv.fromcol = (*wp).w_old_cursor_fcol;
                    wlv.tocol = (*wp).w_old_cursor_lcol;
                }
            } else {
                if lnum > (*top).lnum && lnum <= (*bot).lnum {
                    wlv.fromcol = 0;
                } else if lnum == (*top).lnum {
                    if VIsual_mode.get() == 'V' as ::core::ffi::c_int {
                        wlv.fromcol = 0;
                    } else {
                        getvvcol(
                            wp,
                            top,
                            &raw mut wlv.fromcol,
                            ::core::ptr::null_mut(),
                            ::core::ptr::null_mut(),
                        );
                        if gchar_pos(top) == NUL {
                            // Empty line: invert the one cell past its end.
                            wlv.tocol = wlv.fromcol + 1;
                        }
                    }
                }
                if VIsual_mode.get() != 'V' as ::core::ffi::c_int && lnum == (*bot).lnum {
                    if *p_sel.get() == b'e' as ::core::ffi::c_char
                        && (*bot).col == 0
                        && (*bot).coladd == 0
                    {
                        // 'selection' "exclusive" and the selection stops at
                        // the start of this line: none of it is here.
                        wlv.fromcol = -10;
                        wlv.tocol = MAXCOL as ::core::ffi::c_int;
                    } else if (*bot).col == MAXCOL as colnr_T {
                        wlv.tocol = MAXCOL as ::core::ffi::c_int;
                    } else {
                        let mut pos = *bot;
                        if *p_sel.get() == b'e' as ::core::ffi::c_char {
                            getvvcol(
                                wp,
                                &raw mut pos,
                                &raw mut wlv.tocol,
                                ::core::ptr::null_mut(),
                                ::core::ptr::null_mut(),
                            );
                        } else {
                            getvvcol(
                                wp,
                                &raw mut pos,
                                ::core::ptr::null_mut(),
                                ::core::ptr::null_mut(),
                                &raw mut wlv.tocol,
                            );
                            wlv.tocol += 1;
                        }
                    }
                }
            }

            // The character under the cursor is inverted only if the cursor is
            // drawn as a block inside the selection anyway.
            if !highlight_match.get()
                && self.in_curline
                && cursor_is_block_during_visual(*p_sel.get() == b'e' as ::core::ffi::c_char)
            {
                self.noinvcur = true;
            }

            if wlv.fromcol >= 0 {
                self.area_highlighting = true;
                self.vi_attr = win_hl_attr(wp, HLF_V);
            }
        }
    }

    /// The inverted range for `'incsearch'` and `:s///c`.
    ///
    /// # Safety
    /// `wp` must be the current window.
    unsafe fn incsearch_area(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        let lnum = wlv.lnum;
        // SAFETY: the caller's window.
        unsafe {
            if lnum == (*curwin.get()).w_cursor.lnum {
                getvcol(
                    curwin.get(),
                    &raw mut (*curwin.get()).w_cursor,
                    &raw mut wlv.fromcol,
                    ::core::ptr::null_mut(),
                    ::core::ptr::null_mut(),
                );
            } else {
                wlv.fromcol = 0;
            }
            if lnum == (*curwin.get()).w_cursor.lnum + search_match_lines.get() {
                let mut pos = pos_T {
                    lnum,
                    col: search_match_endcol.get(),
                    coladd: 0,
                };
                getvcol(
                    curwin.get(),
                    &raw mut pos,
                    &raw mut wlv.tocol,
                    ::core::ptr::null_mut(),
                    ::core::ptr::null_mut(),
                );
            }
            // Do at least one character; the match can be past the end of the
            // line.
            if wlv.fromcol == wlv.tocol && search_match_endcol.get() != 0 {
                wlv.tocol = wlv.fromcol + 1;
            }
            self.area_highlighting = true;
            self.vi_attr = win_hl_attr(wp, HLF_I);
        }
    }

    /// Diff-mode state for this line: how many filler lines it needs above it
    /// and which diff highlight its text takes.
    ///
    /// # Safety
    /// `wp` must be a live window.
    unsafe fn diff_state(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window.
        unsafe {
            let mut linestatus = 0;
            wlv.filler_lines = diff_check_with_linestatus(wp, wlv.lnum, &raw mut linestatus);
            if linestatus >= 0 {
                return;
            }
            if linestatus != -1 {
                wlv.diff_hlf = HLF_ADD; // added line
            } else if diff_find_change(wp, wlv.lnum, &raw mut self.line_changes) {
                wlv.diff_hlf = HLF_ADD; // added line
            } else if self.line_changes.num_changes > 0 {
                let added = diff_change_parse(
                    &raw mut self.line_changes,
                    self.line_changes.changes,
                    &raw mut self.change_start,
                    &raw mut self.change_end,
                );
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
    }

    /// Count the filler lines above this one — diff filler plus virtual lines.
    ///
    /// # Safety
    /// `wp` must be a live window.
    unsafe fn filler_lines(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window.
        unsafe {
            wlv.n_virt_lines = decor_virt_lines(
                wp,
                wlv.lnum - 1,
                wlv.lnum,
                &raw mut wlv.n_virt_below,
                &raw mut self.virt_lines,
                true,
            );
            wlv.filler_lines += wlv.n_virt_lines;
            if wlv.lnum == (*wp).w_topline {
                // The top line shows only as much filler as it is scrolled to.
                wlv.filler_lines = (*wp).w_topfill;
                wlv.n_virt_lines = wlv.n_virt_lines.min(wlv.filler_lines);
            }
            wlv.filler_todo = wlv.filler_lines;
        }
    }

    /// Apply `'cursorline'` to this line, if it is the cursor's.
    ///
    /// # Safety
    /// `wp` must be a live window.
    unsafe fn cursorline(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window.
        unsafe {
            if (*wp).w_onebuf_opt.wo_cul == 0
                || (*wp).w_p_culopt_flags as ::core::ffi::c_int
                    == kOptCuloptFlagNumber as ::core::ffi::c_int
                || wlv.lnum != (*wp).w_cursorline
                // Not while Visual mode is active: it would stop being clear
                // what is selected.
                || (wp == curwin.get() && VIsual_active.get())
            {
                return;
            }
            self.cul_screenline = self.is_wrapped
                && (*wp).w_p_culopt_flags as ::core::ffi::c_int
                    & kOptCuloptFlagScreenline as ::core::ffi::c_int
                    != 0;
            if self.cul_screenline {
                // Only the cursor's own screen row is highlighted, so the loop
                // needs that row's margins.
                (self.left_curline_col, self.right_curline_col) = margin_columns_win(wp);
            } else {
                wlv.apply_cursorline_highlight(wp);
            }
            self.area_highlighting = true;
        }
    }

    /// Collect the signs on this line, and either build the `'statuscolumn'`
    /// request or resolve the sign highlights the number column will use.
    ///
    /// # Safety
    /// `wp` must be a live window.
    unsafe fn signs_and_statuscolumn(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window.
        unsafe {
            let mut sign_line_attr = 0;
            // TODO(bfredl, vigoux): line_attr should not take priority over
            // decoration.
            decor_redraw_signs(
                wp,
                (*wp).w_buffer,
                wlv.lnum - 1,
                &raw mut wlv.sattrs as *mut SignTextAttrs,
                &raw mut sign_line_attr,
                &raw mut wlv.sign_cul_attr,
                &raw mut wlv.sign_num_attr,
            );

            if *(*wp).w_onebuf_opt.wo_stc != 0 {
                // 'statuscolumn' replaces the fold, sign and number columns;
                // the expression is evaluated per row by `draw_statuscol`.
                self.statuscol.draw = true;
                self.statuscol.sattrs = &raw mut wlv.sattrs as *mut SignTextAttrs;
                self.statuscol.lnum = wlv.lnum;
                self.statuscol.foldinfo = wlv.foldinfo;
                self.statuscol.width =
                    win_col_off(wp) - (wp == cmdwin_win.get()) as ::core::ffi::c_int;
                self.statuscol.sign_cul_id = if use_cursor_line_highlight(wp, wlv.lnum) {
                    wlv.sign_cul_attr
                } else {
                    0
                };
            } else if wlv.sign_cul_attr > 0 {
                wlv.sign_cul_attr = if use_cursor_line_highlight(wp, wlv.lnum) {
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
            if bt_quickfix((*wp).w_buffer) && qf_current_entry(wp) == wlv.lnum {
                wlv.line_attr = win_hl_attr(wp, HLF_QFL);
            }
            if wlv.line_attr_lowprio != 0 || wlv.line_attr != 0 {
                self.area_highlighting = true;
            }
        }
    }

    /// Prepare the spell checker for this line: decide where a capital is
    /// required, and join the tail of this line to the start of the next one
    /// so that a word wrapping across the break ("et<line-break>al.") is still
    /// seen whole.
    ///
    /// # Safety
    /// `wp` must be a live window and `spv` its spell state.
    unsafe fn spell_line_start(
        &mut self,
        wp: *mut win_T,
        lnum: linenr_T,
        spv: *mut spellvars_T,
        scratch: &mut LineScratch,
    ) {
        // SAFETY: the caller's window and spell state.
        unsafe {
            self.extra_check = true;

            // A word wrapped from the previous line leaves the start of this
            // one already checked.
            if lnum == (*spv).spv_checked_lnum {
                self.cur_checked_col = (*spv).spv_checked_col;
            }
            // The previous line was not spell checked — the first line of an
            // updated region, or the line after a closed fold — so this one
            // has to decide for itself whether a capital is required.
            if (*spv).spv_capcol_lnum == 0 && check_need_cap(wp, lnum, 0) {
                (*spv).spv_cap_col = 0;
            } else if lnum != (*spv).spv_capcol_lnum {
                (*spv).spv_cap_col = -1;
            }
            (*spv).spv_checked_lnum = 0;

            // Trick: `spell_cat_line` skips a few characters for C/shell/Vim
            // comment leaders.
            scratch.nextline[SPELL_LOOKAHEAD] = 0;
            if lnum < (*(*wp).w_buffer).b_ml.ml_line_count {
                let next = ml_get_buf((*wp).w_buffer, lnum + 1);
                spell_cat_line(
                    scratch.nextline.as_mut_ptr().add(SPELL_LOOKAHEAD),
                    next,
                    SPWORDLEN,
                );
            }
            let line = ml_get_buf((*wp).w_buffer, lnum);

            // An empty line: check the first word of the next one for a
            // capital instead.
            let first = skipwhite(line);
            if *first == 0 {
                (*spv).spv_cap_col = 0;
                (*spv).spv_capcol_lnum = lnum + 1;
            } else if (*spv).spv_cap_col == 0 {
                (*spv).spv_cap_col = first.offset_from(line) as ::core::ffi::c_int;
            }

            if scratch.nextline[SPELL_LOOKAHEAD] == 0 {
                // No next line, or it is empty.
                self.nextlinecol = MAXCOL as ::core::ffi::c_int;
                self.nextline_idx = 0;
                return;
            }
            let line_len = ml_get_buf_len((*wp).w_buffer, lnum) as usize;
            if line_len < SPELL_LOOKAHEAD {
                // Short line: use all of it, then move the next line's start
                // up against it.
                let tail = scratch.nextline[SPELL_LOOKAHEAD..]
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(SPELL_LOOKAHEAD - 1);
                ::core::ptr::copy_nonoverlapping(line, scratch.nextline.as_mut_ptr(), line_len);
                scratch
                    .nextline
                    .copy_within(SPELL_LOOKAHEAD..SPELL_LOOKAHEAD + tail + 1, line_len);
                self.nextlinecol = 0;
                self.nextline_idx = line_len as ::core::ffi::c_int + 1;
            } else {
                // Long line: only its last `SPWORDLEN` bytes can matter.
                self.nextlinecol = (line_len - SPELL_LOOKAHEAD) as ::core::ffi::c_int;
                ::core::ptr::copy_nonoverlapping(
                    line.add(self.nextlinecol as usize),
                    scratch.nextline.as_mut_ptr(),
                    SPELL_LOOKAHEAD,
                );
                self.nextline_idx = SPWORDLEN + 1;
            }
        }
    }

    /// Find the leading and trailing whitespace the `'listchars'` "lead" and
    /// "trail" marks apply to.
    ///
    /// # Safety
    /// `wp` must be a live window and [`LineSetup::line`] its line `lnum`.
    unsafe fn listchars_columns(&mut self, wp: *mut win_T, lnum: linenr_T) {
        // SAFETY: the caller's window and line.
        unsafe {
            if (*wp).w_p_lcs_chars.space != 0
                || !(*wp).w_p_lcs_chars.multispace.is_null()
                || !(*wp).w_p_lcs_chars.leadmultispace.is_null()
                || (*wp).w_p_lcs_chars.trail != 0
                || (*wp).w_p_lcs_chars.lead != 0
                || (*wp).w_p_lcs_chars.nbsp != 0
            {
                self.extra_check = true;
            }
            if (*wp).w_p_lcs_chars.trail != 0 {
                let mut trailcol = ml_get_buf_len((*wp).w_buffer, lnum);
                while trailcol > 0
                    && ascii_iswhite(*self.ptr.offset(trailcol as isize - 1) as ::core::ffi::c_int)
                {
                    trailcol -= 1;
                }
                self.trailcol = trailcol + self.ptr.offset_from(self.line) as colnr_T;
            }
            if (*wp).w_p_lcs_chars.lead != 0
                || !(*wp).w_p_lcs_chars.leadmultispace.is_null()
                || (*wp).w_p_lcs_chars.leadtab1 != 0
            {
                let mut leadcol: colnr_T = 0;
                while ascii_iswhite(*self.ptr.offset(leadcol as isize) as ::core::ffi::c_int) {
                    leadcol += 1;
                }
                self.leadcol = if *self.ptr.offset(leadcol as isize) == 0 {
                    // In a line of nothing but spaces they all count as
                    // trailing.
                    0
                } else {
                    // The first column not filled with spaces.
                    leadcol + (self.ptr.offset_from(self.line) + 1) as colnr_T
                };
            }
        }
    }

    /// Advance [`LineSetup::ptr`] and `wlv.vcol` to the first character that is
    /// on screen, when the line is scrolled sideways or `w_skipcol` is set.
    ///
    /// # Safety
    /// `wp` must be a live window, `spv` its spell state, and
    /// [`LineSetup::line`] its line.
    unsafe fn skip_to_start_vcol(
        &mut self,
        wlv: &mut WinLineVars,
        wp: *mut win_T,
        spv: *mut spellvars_T,
    ) {
        let start_vcol = self.start_vcol;
        // SAFETY: the caller's window and line.
        unsafe {
            let mut prev_ptr = self.ptr;
            let mut cs = CharSize { width: 0, head: 0 };
            let mut csarg = CharsizeArg::default();
            let cstype = init_charsize_arg(&mut csarg, wp, wlv.lnum, self.line);
            csarg.max_head_vcol = start_vcol;
            let mut vcol = wlv.vcol;
            let mut ci = utf_ptr2StrCharInfo(self.ptr);
            while vcol < start_vcol {
                cs = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &mut csarg);
                vcol += cs.width;
                prev_ptr = ci.ptr;
                if *prev_ptr == 0 {
                    break;
                }
                ci = utfc_next(ci);
                if (*wp).w_onebuf_opt.wo_list != 0 {
                    self.track_multispace(wp, prev_ptr, ci.ptr);
                }
            }
            wlv.vcol = vcol;
            self.ptr = ci.ptr;

            // The end of the line can be left of the first displayed column
            // when 'cursorcolumn' or 'colorcolumn' is set, when 'virtualedit'
            // or Visual mode is active, or when a fold is being drawn — all of
            // which still have something to draw out there.
            if wlv.vcol < start_vcol
                && ((*wp).w_onebuf_opt.wo_cuc != 0
                    || !wlv.color_cols.is_null()
                    || virtual_active(wp)
                    || (VIsual_active.get() && (*wp).w_buffer == (*curwin.get()).w_buffer)
                    || self.has_fold)
            {
                wlv.vcol = start_vcol;
            }

            // A character that is only partly on screen: stand on it, and skip
            // the cells of it that are not.
            if wlv.vcol > start_vcol {
                wlv.vcol -= cs.width;
                self.ptr = prev_ptr;
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
            if (*wp).w_onebuf_opt.wo_wrap != 0 {
                wlv.need_showbreak = true;
            }

            if (*spv).spv_has_spell {
                self.spell_at_start_vcol(wp, wlv.lnum);
            }
        }
    }

    /// Track where in a run of consecutive spaces the skipped-over text has
    /// got to, so that `'listchars'` "multispace" resumes at the right glyph.
    ///
    /// # Safety
    /// `wp` must be a live window and both pointers must point into
    /// [`LineSetup::line`].
    unsafe fn track_multispace(
        &mut self,
        wp: *mut win_T,
        prev_ptr: *const ::core::ffi::c_char,
        next_ptr: *const ::core::ffi::c_char,
    ) {
        // SAFETY: the caller's window and line.
        unsafe {
            self.in_multispace = *prev_ptr == b' ' as ::core::ffi::c_char
                && (*next_ptr == b' ' as ::core::ffi::c_char
                    || (prev_ptr > self.line
                        && *prev_ptr.offset(-1) == b' ' as ::core::ffi::c_char));
            if !self.in_multispace {
                self.multispace_pos = 0;
                return;
            }
            let lead = self.line.offset(self.leadcol as isize);
            let pattern = if next_ptr >= lead {
                (*wp).w_p_lcs_chars.multispace
            } else {
                (*wp).w_p_lcs_chars.leadmultispace
            };
            if pattern.is_null() {
                return;
            }
            self.multispace_pos += 1;
            if *pattern.offset(self.multispace_pos as isize) == 0 {
                self.multispace_pos = 0;
            }
        }
    }

    /// Work out whether the first character on screen is inside a badly
    /// spelled word, since the loop only ever starts a spell check at a word
    /// boundary.
    ///
    /// # Safety
    /// `wp` must be a live window and [`LineSetup::line`] its line `lnum`.
    unsafe fn spell_at_start_vcol(&mut self, wp: *mut win_T, lnum: linenr_T) {
        // SAFETY: the caller's window and line.
        unsafe {
            let linecol = self.ptr.offset_from(self.line) as colnr_T;
            let mut spell_hlf: hlf_T = HLF_COUNT;

            let saved_cursor = (*wp).w_cursor;
            (*wp).w_cursor.lnum = lnum;
            (*wp).w_cursor.col = linecol;
            let len = spell_move_to(wp, FORWARD, SMT_ALL, true, &raw mut spell_hlf);

            // `spell_move_to` may call `ml_get` and invalidate "line".
            self.line = ml_get_buf((*wp).w_buffer, lnum);
            self.ptr = self.line.offset(linecol as isize);

            if len == 0 || (*wp).w_cursor.col > linecol {
                // No bad word at the line start: do not check again until the
                // end of a word.
                self.word_end = (spell_to_word_end(self.ptr, wp).offset_from(self.line) + 1)
                    as ::core::ffi::c_int;
            } else {
                // Bad word found: its attribute applies to the end of it.
                assert!(len <= ::core::ffi::c_int::MAX as size_t);
                self.word_end = (*wp).w_cursor.col + len as ::core::ffi::c_int + 1;
                if spell_hlf != HLF_COUNT {
                    self.spell_attr = (*highlight_attr.ptr())[spell_hlf as usize];
                }
            }
            (*wp).w_cursor = saved_cursor;

            // Syntax highlighting has to be restarted for this line.
            if self.has_syntax {
                syntax_start(wp, lnum);
            }
        }
    }

    /// Correct the inverted range so that it never swallows a cursor that has
    /// to stay visible.
    ///
    /// Doing it once here saves testing for it on every character.
    ///
    /// # Safety
    /// `wp` must be a live window.
    unsafe fn keep_cursor_visible(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window.
        unsafe {
            if wlv.fromcol < 0 {
                return;
            }
            if self.noinvcur {
                if wlv.fromcol == (*wp).w_virtcol {
                    // Inverting starts at the cursor; start just after it.
                    self.fromcol_prev = wlv.fromcol;
                    wlv.fromcol = -1;
                } else if wlv.fromcol < (*wp).w_virtcol {
                    // Resume inverting after the cursor.
                    self.fromcol_prev = (*wp).w_virtcol;
                }
            }
            if wlv.fromcol >= wlv.tocol {
                wlv.fromcol = -1;
            }
        }
    }
}
