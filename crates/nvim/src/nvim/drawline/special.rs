//! Cells that stand for something other than the character under them.
//!
//! A Tab becomes a run of blanks or `'listchars'` "tab" characters; an
//! unprintable character becomes `^X` or `<xx>`; the line break itself can
//! become a `'listchars'` "eol" or a highlighted blank; a row that starts or
//! stops part-way through the line gets the "precedes" and "extends" markers;
//! a closed fold replaces the whole line with its `'foldtext'`; and a
//! concealed run becomes one stand-in character followed by nothing at all.
//!
//! What these have in common is that they all set up a run in
//! [`WinLineVars::extra_todo`] and hand the first cell of it back to the loop.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

impl Cells {
    /// Show the `'listchars'` "precedes" character in column zero of a row
    /// that starts part-way into the line.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(super) unsafe fn draw_precedes(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window.
        unsafe {
            let scrolled = if (*wp).w_onebuf_opt.wo_wrap != 0 {
                (*wp).w_skipcol > 0 && wlv.row == 0
            } else {
                (*wp).w_leftcol > 0
            };
            if self.lcs_prec_todo == NUL as schar_T
                || (*wp).w_onebuf_opt.wo_list == 0
                || !scrolled
                || wlv.filler_todo > 0
                || wlv.skip_cells > 0
                || self.cell_char == NUL as schar_T
            {
                return;
            }
            self.lcs_prec_todo = NUL as schar_T;
            if schar_cells(self.cell_char) > 1 {
                // The "precedes" character overwrites a double-width one;
                // fill up its other half.
                wlv.extra_fill = schar_from_ascii(MB_FILLER_CHAR);
                wlv.extra_last = NUL as schar_T;
                if wlv.extra_todo > 0 {
                    assert!(!wlv.extra_text.is_null());
                    self.extra_todo_next = wlv.extra_todo;
                    self.extra_attr_next = wlv.extra_attr;
                    wlv.n_attr = (wlv.n_attr + 1).max(2);
                } else {
                    wlv.n_attr = 2;
                }
                wlv.extra_todo = 1;
                wlv.extra_attr = win_hl_attr(wp, HLF_AT);
            }
            self.cell_char = (*wp).w_p_lcs_chars.prec;
            self.char_code = schar_get_first_codepoint(self.cell_char);
            self.attr_before_prec = wlv.char_attr;
            wlv.char_attr = win_hl_attr(wp, HLF_AT);
            self.prec_attr_todo = 1;
        }
    }

    /// Show the `'listchars'` "extends" character in the last column when the
    /// line goes on past the right edge.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(super) unsafe fn draw_extends(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window and the redraw's decoration state.
        unsafe {
            let lcs_ext = get_lcs_ext(wp);
            if lcs_ext == NUL as schar_T
                || wlv.filler_todo > 0
                || wlv.col != self.view_width - 1
                || self.has_foldtext
            {
                return;
            }
            if self.has_decor
                && *self.ptr as ::core::ffi::c_int == NUL
                && self.lcs_eol == 0
                && self.lcs_eol_todo
            {
                // Tricky: there might be a virtual text just *after* the last
                // character.
                decor_redraw_col(
                    wp,
                    self.byte_col(),
                    -1,
                    false,
                    decor_state.ptr(),
                    self.decor_provider_end_col - 1,
                );
            }
            if *self.ptr as ::core::ffi::c_int != NUL
                || (self.lcs_eol > 0 && self.lcs_eol_todo)
                || (wlv.extra_todo > 0
                    && (wlv.extra_fill != NUL as schar_T
                        || *wlv.extra_text as ::core::ffi::c_int != NUL))
                || (self.may_have_inline_virt
                    && wlv.has_more_inline_virt(self.ptr.offset_from(self.line)))
            {
                self.cell_char = lcs_ext;
                wlv.char_attr = win_hl_attr(wp, HLF_AT);
                self.char_code = schar_get_first_codepoint(self.cell_char);
            }
        }
    }

    /// Render `'foldtext'` into the scratch buffer and set up the `'fold'`
    /// fill that follows it.
    ///
    /// # Safety
    /// `wp` must be live and `f` must hold the caller's frame.
    pub(super) unsafe fn fold_text(
        &mut self,
        wlv: &mut WinLineVars,
        wp: *mut win_T,
        f: &LineFrame,
    ) {
        // SAFETY: the caller's window, frame and fold scratch.
        unsafe {
            if self.draw_folded
                && self.has_foldtext
                && wlv.extra_todo == 0
                && wlv.col == self.text_start_col
            {
                let at = self.byte_col();
                let lnume = wlv.lnum + wlv.foldinfo.fi_lines - 1;
                memset(
                    f.fold_buf.cast::<::core::ffi::c_void>(),
                    ' ' as ::core::ffi::c_int,
                    FOLD_TEXT_LEN as size_t,
                );
                wlv.extra_text = get_foldtext(
                    wp,
                    wlv.lnum,
                    lnume,
                    wlv.foldinfo,
                    f.fold_buf,
                    &raw mut self.fold_vt,
                );
                wlv.extra_todo = strlen(wlv.extra_text) as ::core::ffi::c_int;
                if wlv.extra_text != f.fold_buf {
                    assert!(self.foldtext_free.is_null());
                    self.foldtext_free = wlv.extra_text;
                }
                wlv.extra_fill = NUL as schar_T;
                wlv.extra_last = NUL as schar_T;
                *wlv.extra_text.offset(wlv.extra_todo as isize) = NUL as ::core::ffi::c_char;
                // Evaluating 'foldtext' may have freed the line.
                self.refetch_line(wp, wlv.lnum, at);
            }

            // Fill the rest of the row with the 'fold' fillchar — after the
            // fold text, or after the "eol" listchar for a transparent one.
            if self.draw_folded
                && wlv.extra_todo == 0
                && wlv.col < self.view_width
                && (self.has_foldtext
                    || (*self.ptr as ::core::ffi::c_int == NUL
                        && ((*wp).w_onebuf_opt.wo_list == 0
                            || !self.lcs_eol_todo
                            || self.lcs_eol == NUL as schar_T)))
            {
                wlv.extra_fill = (*wp).w_p_fcs_chars.fold;
                wlv.extra_last = NUL as schar_T;
                wlv.extra_todo = self.view_width - wlv.col;
                // Search highlighting stops at the first filler character.
                self.search_attr = 0;
            }

            if self.draw_folded && wlv.extra_todo != 0 && wlv.col >= self.view_width {
                // Truncate the folding.
                wlv.extra_todo = 0;
            }
        }
    }

    /// Invert one cell past the end of the text, for a Visual selection that
    /// includes the line break or an `'hlsearch'` match that ends there.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(super) unsafe fn highlight_at_eol(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window and the redraw's match state.
        unsafe {
            if self.cell_char != NUL as schar_T || self.eol_extra_cell != 0 {
                return;
            }
            // Does the previous column start a search match?
            let prevcol_hl_flag =
                get_prevcol_hl_flag(wp, screen_search_hl.ptr(), self.byte_col() - 1);
            let want = self.lcs_eol_todo
                && ((self.area_attr != 0
                    && wlv.vcol == wlv.fromcol
                    && (VIsual_mode.get() != Ctrl_V
                        || wlv.lnum == VIsual.get().lnum
                        || wlv.lnum == (*curwin.get()).w_cursor.lnum))
                    || prevcol_hl_flag);
            if !want {
                return;
            }
            if wlv.col >= self.view_width {
                // At the window boundary, highlight the last character
                // instead — better than nothing.
                wlv.off -= 1;
                wlv.col -= 1;
            } else {
                // Add a blank character to highlight.
                *linebuf_char.get().add(wlv.off as usize) = schar_from_ascii(b' ');
            }
            if self.area_attr == 0 && !self.has_fold {
                // Use the attributes of the highest-priority match.
                get_search_match_hl(
                    wp,
                    screen_search_hl.ptr(),
                    self.byte_col(),
                    &raw mut wlv.char_attr,
                );
            }
            let eol_attr = if wlv.cursorline_attr != 0 {
                hl_combine_attr(wlv.cursorline_attr, wlv.char_attr)
            } else {
                wlv.char_attr
            };
            *linebuf_attr.get().add(wlv.off as usize) = eol_attr as sattr_T;
            *linebuf_vcol.get().add(wlv.off as usize) = wlv.vcol;
            wlv.col += 1;
            wlv.off += 1;
            wlv.vcol += 1;
            self.eol_extra_cell = 1;
        }
    }

    /// A character that cannot be put on the screen as itself: a Tab, the NUL
    /// that ends the line, or something that shows as `^X` or `<xx>`.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(super) unsafe fn unprintable(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window and the loop's line pointers.
        unsafe {
            if self.char_code == TAB
                && ((*wp).w_onebuf_opt.wo_list == 0 || (*wp).w_p_lcs_chars.tab1 != 0)
            {
                self.tab(wlv, wp);
            } else if self.cell_char == NUL as schar_T && self.wants_eol_cell(wlv, wp) {
                self.eol_cell(wlv, wp);
            } else if self.cell_char != NUL as schar_T {
                self.escaped(wlv, wp);
            } else if VIsual_active.get()
                && (VIsual_mode.get() == Ctrl_V || VIsual_mode.get() == 'v' as ::core::ffi::c_int)
                && virtual_active(wp)
                && wlv.tocol != MAXCOL as ::core::ffi::c_int
                && wlv.vcol < wlv.tocol
                && wlv.col < self.view_width
            {
                // With 'virtualedit' the selection can run past the end of the
                // line; draw a blank for it.
                self.char_code = ' ' as ::core::ffi::c_int;
                self.cell_char = schar_from_char(self.char_code);
                // Put the pointer back at the NUL.
                self.ptr = self.ptr.offset(-1);
            }
        }
    }

    /// Turn a Tab into the cells it occupies.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(super) unsafe fn tab(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window and the scratch buffer, which is not
        // held across another `get_extra_buf`.
        unsafe {
            let lcs = (*wp).w_p_lcs_chars;
            let (mut lcs_tab1, mut lcs_tab2, mut lcs_tab3) = (lcs.tab1, lcs.tab2, lcs.tab3);
            if (*wp).w_onebuf_opt.wo_list != 0
                && lcs.leadtab1 != NUL as schar_T
                && self.ptr < self.line.offset(self.leadcol as isize)
            {
                lcs_tab1 = lcs.leadtab1;
                lcs_tab2 = lcs.leadtab2;
                lcs_tab3 = lcs.leadtab3;
            }

            // The Tab's width depends on the column, with `'showbreak'`
            // removed: it is not part of the buffer line.
            let sbr = get_showbreak_value(wp);
            let vcol_adjusted = if *sbr != NUL as ::core::ffi::c_char
                && wlv.vcol == wlv.showbreak_vcol
                && (*wp).w_onebuf_opt.wo_wrap != 0
            {
                wlv.vcol - mb_charlen(sbr)
            } else {
                wlv.vcol
            };
            let mut tab_len = tabstop_padding(
                vcol_adjusted,
                (*(*wp).w_buffer).b_p_ts,
                (*(*wp).w_buffer).b_p_vts_array,
            ) - 1;

            if (*wp).w_onebuf_opt.wo_lbr == 0 || (*wp).w_onebuf_opt.wo_list == 0 {
                wlv.extra_todo = tab_len;
            } else {
                let saved_nextra = wlv.extra_todo;
                if wlv.vcol_off_co > 0 {
                    // There are characters to conceal.
                    tab_len += wlv.vcol_off_co;
                }
                // The bogus columns from before `fix_for_boguscols` above.
                if lcs_tab1 != 0 && wlv.old_boguscols > 0 && wlv.extra_todo > tab_len {
                    tab_len += wlv.extra_todo - tab_len;
                }
                if tab_len > 0 {
                    // With 'linebreak' the Tab is spelled out rather than
                    // repeated, so it needs a buffer of its own.
                    //
                    // Both corrections below are genuinely negative sometimes
                    // — a one-byte "tab3" after a two-byte "tab2", and an
                    // `extra_todo` shorter than the Tab — so the running total is
                    // signed. Upstream lets `size_t` wrap through the same
                    // arithmetic and relies on the total coming back positive.
                    let tab2_len = schar_len(lcs_tab2) as isize;
                    let mut len = tab_len as isize * tab2_len;
                    if lcs_tab3 != 0 {
                        len += schar_len(lcs_tab3) as isize - tab2_len;
                    }
                    if wlv.extra_todo > 0 {
                        len += (wlv.extra_todo - tab_len) as isize;
                    }
                    let len = len as size_t;
                    self.cell_char = lcs_tab1;
                    self.char_code = schar_get_first_codepoint(self.cell_char);
                    let mut p = get_extra_buf(len + 1);
                    memset(
                        p.cast::<::core::ffi::c_void>(),
                        ' ' as ::core::ffi::c_int,
                        len,
                    );
                    *p.add(len) = NUL as ::core::ffi::c_char;
                    wlv.extra_text = p;
                    for i in 0..tab_len {
                        if *p == NUL as ::core::ffi::c_char {
                            tab_len = i;
                            break;
                        }
                        // The last cell takes "tab3" when there is one.
                        let lcs_here = if lcs_tab3 != 0 && i == tab_len - 1 {
                            lcs_tab3
                        } else {
                            lcs_tab2
                        };
                        let slen = schar_get_adv(&raw mut p, lcs_here);
                        wlv.extra_todo +=
                            slen as ::core::ffi::c_int - ::core::ffi::c_int::from(saved_nextra > 0);
                    }
                    if wlv.vcol_off_co > 0 {
                        // `fix_for_boguscols` below adds it back.
                        wlv.extra_todo -= wlv.vcol_off_co;
                    }
                }
            }

            {
                let vc_saved = wlv.vcol_off_co;
                // Tab alignment has to be the same whatever 'conceallevel'
                // says, so a Tab compensates for every concealed character
                // before it and resets the counters. That is also why a Tab
                // can be wider than 'tabstop'.
                wlv.fix_for_boguscols();
                // Put back what the line below needs to get the Tab's own
                // highlight right.
                if wlv.extra_todo == tab_len + vc_saved
                    && (*wp).w_onebuf_opt.wo_list != 0
                    && (*wp).w_p_lcs_chars.tab1 != 0
                {
                    tab_len += vc_saved;
                }
            }

            if (*wp).w_onebuf_opt.wo_list != 0 {
                self.cell_char = if wlv.extra_todo == 0 && lcs_tab3 != 0 {
                    lcs_tab3
                } else {
                    lcs_tab1
                };
                if (*wp).w_onebuf_opt.wo_lbr != 0
                    && !wlv.extra_text.is_null()
                    && *wlv.extra_text != NUL as ::core::ffi::c_char
                {
                    // Using `extra_text` from above.
                    wlv.extra_fill = NUL as schar_T;
                } else {
                    wlv.extra_fill = lcs_tab2;
                }
                wlv.extra_last = lcs_tab3;
                wlv.n_attr = tab_len + 1;
                wlv.extra_attr = win_hl_attr(wp, HLF_0);
                self.attr_before_run = wlv.char_attr;
            } else {
                wlv.extra_last = NUL as schar_T;
                wlv.extra_fill = schar_from_ascii(b' ');
                self.cell_char = schar_from_ascii(b' ');
            }
            self.char_code = schar_get_first_codepoint(self.cell_char);
        }
    }

    /// Draw an unprintable character in its `^X` or `<xx>` form.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(super) unsafe fn escaped(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window; `transchar_buf` answers a static
        // NUL-terminated buffer.
        unsafe {
            wlv.extra_text = transchar_buf((*wp).w_buffer, self.char_code);
            if wlv.extra_todo == 0 {
                wlv.extra_todo = byte2cells(self.char_code) - 1;
            }
            if dy_flags.get() & kOptDyFlagUhex as uint32_t != 0 && (*wp).w_onebuf_opt.wo_rl != 0 {
                // Reverse "<12>".
                rl_mirror_ascii(wlv.extra_text, ::core::ptr::null_mut());
            }
            wlv.extra_fill = NUL as schar_T;
            wlv.extra_last = NUL as schar_T;
            if (*wp).w_onebuf_opt.wo_lbr != 0 {
                // With 'linebreak' the escape has to be padded out to the
                // width the character would have had.
                self.char_code = *wlv.extra_text as uint8_t as ::core::ffi::c_int;
                let p = get_extra_buf(wlv.extra_todo as size_t + 1);
                memset(
                    p.cast::<::core::ffi::c_void>(),
                    ' ' as ::core::ffi::c_int,
                    wlv.extra_todo as size_t,
                );
                memcpy(
                    p.cast::<::core::ffi::c_void>(),
                    wlv.extra_text.offset(1).cast::<::core::ffi::c_void>(),
                    strlen(wlv.extra_text) - 1,
                );
                *p.offset(wlv.extra_todo as isize) = NUL as ::core::ffi::c_char;
                wlv.extra_text = p;
            } else {
                wlv.extra_todo = byte2cells(self.char_code) - 1;
                self.char_code = *wlv.extra_text as uint8_t as ::core::ffi::c_int;
                wlv.extra_text = wlv.extra_text.offset(1);
            }
            wlv.n_attr = wlv.extra_todo + 1;
            wlv.extra_attr = win_hl_attr(wp, HLF_8);
            self.attr_before_run = wlv.char_attr;
            self.cell_char = schar_from_ascii(self.char_code as u8);
        }
    }

    /// Should a cell be drawn for the line break itself?
    ///
    /// Either because `'list'` asks for one, or because a Visual or
    /// `'incsearch'` range includes the break and needs somewhere to show it.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(super) unsafe fn wants_eol_cell(&self, wlv: &WinLineVars, wp: *mut win_T) -> bool {
        // SAFETY: the caller's window.
        unsafe {
            ((*wp).w_onebuf_opt.wo_list != 0
                || ((wlv.fromcol >= 0 || self.fromcol_prev >= 0)
                    && wlv.tocol > wlv.vcol
                    && VIsual_mode.get() != Ctrl_V
                    && wlv.col < self.view_width
                    && !(self.noinvcur
                        && wlv.lnum == (*wp).w_cursor.lnum
                        && wlv.vcol == (*wp).w_virtcol)))
                && self.lcs_eol_todo
                && self.lcs_eol != NUL as schar_T
        }
    }

    /// Draw the `'listchars'` "eol" character, or a highlighted blank standing
    /// for the line break.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(super) unsafe fn eol_cell(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window.
        unsafe {
            // For a diff line the highlighting continues after the "$".
            if wlv.diff_hlf == HLF_NONE && wlv.line_attr == 0 && wlv.line_attr_lowprio == 0 {
                if !(self.area_highlighting
                    && virtual_active(wp)
                    && wlv.tocol != MAXCOL as ::core::ffi::c_int
                    && wlv.vcol < wlv.tocol)
                {
                    // Under 'virtualedit' a Visual selection may extend past
                    // the end of the line, and then the run has to survive.
                    wlv.extra_text = c"".as_ptr() as *mut ::core::ffi::c_char;
                }
                wlv.extra_todo = 0;
            }
            self.cell_char = if (*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.eol > 0 {
                (*wp).w_p_lcs_chars.eol
            } else {
                schar_from_ascii(b' ')
            };
            self.lcs_eol_todo = false;
            // Put the pointer back at the NUL.
            self.ptr = self.ptr.offset(-1);
            wlv.extra_attr = win_hl_attr(wp, HLF_AT);
            wlv.n_attr = 1;
            self.char_code = schar_get_first_codepoint(self.cell_char);
        }
    }

    /// Concealment: either replace the first character of a concealed run with
    /// one stand-in character, or drop the cell entirely.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(super) unsafe fn conceal(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window and the redraw's decoration state.
        unsafe {
            let wants_conceal = (*wp).w_onebuf_opt.wo_cole > 0
                && (wp != curwin.get()
                    || wlv.lnum != (*wp).w_cursor.lnum
                    || conceal_cursor_line(wp))
                && (self.syntax_flags & HL_CONCEAL as ::core::ffi::c_int != 0
                    || self.has_match_conc > 0
                    || self.decor_conceal > 0)
                // 'concealcursor' does not name "v", so the Visual area shows
                // its text.
                && !(self.lnum_in_visual_area
                    && vim_strchr((*wp).w_onebuf_opt.wo_cocu, 'v' as ::core::ffi::c_int).is_null());
            if !wants_conceal {
                self.prev_syntax_id = 0;
                self.is_concealing = false;
                return;
            }

            let syntax_conceal = self.syntax_flags & HL_CONCEAL as ::core::ffi::c_int != 0;
            wlv.char_attr = self.conceal_attr;

            // The first character of a concealed run shows the stand-in; the
            // rest of the run shows nothing. 'conceallevel' 1 always has a
            // stand-in (a space when nothing else names one), 3 never does.
            let first_of_run = (self.prev_syntax_id != self.syntax_seqnr && syntax_conceal)
                || self.has_match_conc > 1
                || self.decor_conceal > 1;
            let have_char = (syntax_conceal && syn_get_sub_char() != NUL)
                || (self.has_match_conc != 0 && self.match_conc != 0)
                || (self.decor_conceal != 0 && (*decor_state.ptr()).conceal_char != 0)
                || (*wp).w_onebuf_opt.wo_cole == 1;
            if first_of_run && have_char && (*wp).w_onebuf_opt.wo_cole != 3 {
                if schar_cells(self.cell_char) > 1 {
                    // The first concealed character is double-width, so one
                    // more virtual column goes with it.
                    wlv.extra_todo += 1;
                }
                self.cell_char = if self.has_match_conc != 0 && self.match_conc != 0 {
                    schar_from_char(self.match_conc)
                } else if self.decor_conceal != 0 && (*decor_state.ptr()).conceal_char != 0 {
                    if (*decor_state.ptr()).conceal_attr != 0 {
                        wlv.char_attr = (*decor_state.ptr()).conceal_attr;
                    }
                    (*decor_state.ptr()).conceal_char
                } else if syntax_conceal && syn_get_sub_char() != NUL {
                    schar_from_char(syn_get_sub_char())
                } else if (*wp).w_p_lcs_chars.conceal != NUL as schar_T {
                    (*wp).w_p_lcs_chars.conceal
                } else {
                    schar_from_ascii(b' ')
                };
                self.char_code = schar_get_first_codepoint(self.cell_char);
                self.prev_syntax_id = self.syntax_seqnr;

                if wlv.extra_todo > 0 {
                    wlv.vcol_off_co += wlv.extra_todo;
                }
                wlv.vcol += wlv.extra_todo;
                if self.is_wrapped && wlv.extra_todo > 0 {
                    wlv.boguscols += wlv.extra_todo;
                    wlv.col += wlv.extra_todo;
                }
                wlv.extra_todo = 0;
                wlv.n_attr = 0;
            } else if wlv.skip_cells == 0 {
                self.is_concealing = true;
                wlv.skip_cells = 1;
            }
        }
    }
}
