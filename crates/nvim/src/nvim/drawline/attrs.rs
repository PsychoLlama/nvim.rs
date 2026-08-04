//! What colour a cell takes.
//!
//! Every source of highlighting is asked once per cell and the answers are
//! combined in a fixed order. They fall into two priorities:
//!
//! - `attr_base` — the fold text, syntax, extmark decorations and spelling,
//!   which describe the *character*;
//! - `attr_pri` — the Visual or `'incsearch'` range, `'hlsearch'` and `:match`,
//!   and whole-line highlights such as diff mode, which describe the *place*.
//!
//! `'cursorcolumn'` and `'colorcolumn'` are applied on top of both and undone
//! again after the cell, and `'cursorline'`'s low-priority form goes
//! underneath everything — except where the character has only Normal's
//! background, which is the one case where the order is reversed.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::decoration::kHlModeReplace;
use crate::src::nvim::pos::MAXCOL;

impl Cells {
    /// Work out what this cell takes from everything that is not the character
    /// itself: decorations, the Visual range, `'hlsearch'` and diff mode.
    ///
    /// # Safety
    /// `wp` must be live and `f` must hold the caller's frame.
    pub(super) unsafe fn cell_attributes(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window and frame.
        unsafe {
            if wlv.extra_todo == 0 || !wlv.extra_is_virt_text {
                wlv.reset_extra_attr = false;
            }

            if self.has_decor && wlv.extra_todo == 0 {
                self.decorations_at(wlv, wp);
            }

            // While inline virtual text is being drawn the real area
            // attribute is parked in `saved_area_attr`; the two are the same
            // slot as far as this test is concerned.
            let use_saved = wlv.extra_is_virt_text && wlv.virt_inline_hl_mode <= kHlModeReplace;
            let mut area = if use_saved {
                self.saved_area_attr
            } else {
                self.area_attr
            };
            if self.area_starts_here(wlv) {
                area = self.vi_attr;
                self.area_active = true;
            } else if area != 0
                && (wlv.vcol == wlv.tocol || (self.noinvcur && wlv.vcol == (*wp).w_virtcol))
            {
                area = 0;
                self.area_active = false;
            }
            if use_saved {
                self.saved_area_attr = area;
            } else {
                self.area_attr = area;
            }

            if !self.has_foldtext && wlv.extra_todo == 0 {
                self.search_highlight(wlv, wp);
            }

            if wlv.diff_hlf != HLF_NONE {
                self.diff_highlight(wlv, wp);
            }

            // Decide which of the highlight attributes to use.
            self.attr_pri = if self.area_attr != 0 {
                let pri = hl_combine_attr(wlv.line_attr, self.area_attr);
                if highlight_match.get() {
                    pri
                } else {
                    // Let search highlighting show through the Visual area.
                    hl_combine_attr(self.search_attr, pri)
                }
            } else if self.search_attr != 0 {
                hl_combine_attr(wlv.line_attr, self.search_attr)
            } else if wlv.line_attr != 0
                && ((wlv.fromcol == -10 && wlv.tocol == MAXCOL as ::core::ffi::c_int)
                    || wlv.vcol < wlv.fromcol
                    || self.prev_vcol < self.fromcol_prev
                    || wlv.vcol >= wlv.tocol)
            {
                // `area_attr` may be 0 here even inside the range, when
                // "noinvcur" made it skip the cursor.
                wlv.line_attr
            } else {
                0
            };
            self.attr_base = hl_combine_attr(self.fold_attr, self.decor_attr);
            wlv.char_attr = hl_combine_attr(self.attr_base, self.attr_pri);
        }
    }

    /// Is this the cell the Visual or `'incsearch'` range starts at?
    ///
    /// Three ways in: exactly at the start; one column before it when the
    /// character is double-width, so that inverting starts on its first half;
    /// or resuming after the cursor that "noinvcur" skipped, which is what
    /// `prev_vcol == fromcol_prev` says.
    ///
    /// # Safety
    /// `ptr` and `extra_text` must be readable.
    pub(super) unsafe fn area_starts_here(&self, wlv: &WinLineVars) -> bool {
        // SAFETY: the loop's own pointers.
        unsafe {
            wlv.vcol == wlv.fromcol
                || (wlv.vcol + 1 == wlv.fromcol
                    && ((wlv.extra_todo == 0 && utf_ptr2cells(self.ptr) > 1)
                        || (wlv.extra_todo > 0
                            && !wlv.extra_text.is_null()
                            && utf_ptr2cells(wlv.extra_text) > 1)))
                || (self.prev_vcol == self.fromcol_prev
                    && self.prev_vcol < wlv.vcol
                    && wlv.vcol < wlv.tocol)
        }
    }

    /// Run the decoration walk for this cell and feed the loop whatever inline
    /// virtual text starts here.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(super) unsafe fn decorations_at(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window and the redraw's decoration state.
        unsafe {
            // The Visual-area test below is repeated here rather than shared,
            // because this one may not look inside `extra_text`.
            if wlv.vcol == wlv.fromcol
                || (wlv.vcol + 1 == wlv.fromcol
                    && wlv.extra_todo == 0
                    && utf_ptr2cells(self.ptr) > 1)
                || (self.prev_vcol == self.fromcol_prev
                    && self.prev_vcol < wlv.vcol
                    && wlv.vcol < wlv.tocol)
            {
                self.area_active = true;
            } else if self.area_active
                && (wlv.vcol == wlv.tocol || (self.noinvcur && wlv.vcol == (*wp).w_virtcol))
            {
                self.area_active = false;
            }

            let selected = self.area_active
                || (self.area_highlighting && self.noinvcur && wlv.vcol == (*wp).w_virtcol);

            // Where non-inline virtual text goes can only be decided once the
            // inline text with a lower priority has been drawn.
            if self.decor_need_recheck {
                if !self.may_have_inline_virt {
                    decor_recheck_draw_col(wlv.off, selected, decor_state.ptr());
                }
                self.decor_need_recheck = false;
            }
            self.extmark_attr = decor_redraw_col(
                wp,
                self.byte_col(),
                if self.may_have_inline_virt {
                    -3
                } else {
                    wlv.off
                },
                selected,
                decor_state.ptr(),
                self.decor_provider_end_col - 1,
            );
            if !self.may_have_inline_virt {
                return;
            }
            wlv.handle_inline_virtual_text(self.ptr.offset_from(self.line), selected);
            if wlv.extra_todo > 0 && wlv.virt_inline_hl_mode <= kHlModeReplace {
                // Park the attributes the virtual text replaces; they come
                // back when `extra_todo` is spent.
                self.saved_search_attr = self.search_attr;
                self.saved_area_attr = self.area_attr;
                self.saved_decor_attr = self.decor_attr;
                self.saved_search_attr_from_match = self.search_attr_from_match;
                self.search_attr = 0;
                self.area_attr = 0;
                self.decor_attr = 0;
                self.search_attr_from_match = false;
            }
        }
    }

    /// Check for the start or end of an `'hlsearch'` or `:match` run, and for
    /// the insert-mode completion highlight.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(super) unsafe fn search_highlight(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window and the redraw's match state.
        unsafe {
            let at = self.byte_col();
            self.search_attr = update_search_hl(
                wp,
                wlv.lnum,
                at,
                &raw mut self.line,
                screen_search_hl.ptr(),
                &raw mut self.has_match_conc,
                &raw mut self.match_conc,
                self.lcs_eol_todo,
                &raw mut self.on_last_col,
                &raw mut self.search_attr_from_match,
            );
            // `line` may have moved.
            self.ptr = self.line.offset(at as isize);

            // A conceal over the end of the line would hide the eol itself.
            if *self.ptr as ::core::ffi::c_int == NUL {
                self.has_match_conc = 0;
            }

            if State.get() & MODE_INSERT != 0
                && ins_compl_win_active(wp)
                && (self.in_curline || ins_compl_lnum_in_range(wlv.lnum))
            {
                let ins_match_attr = ins_compl_col_range_attr(wlv.lnum, self.byte_col());
                if ins_match_attr > 0 {
                    self.search_attr = hl_combine_attr(self.search_attr, ins_match_attr);
                }
            }
        }
    }

    /// Move the diff highlight between "this line changed" and "this text
    /// changed" as the read cursor enters and leaves each changed range.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(super) unsafe fn diff_highlight(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window and the diff answer for this line.
        unsafe {
            let at = self.ptr.offset_from(self.line);
            if self.line_changes.num_changes > 0
                && self.change_index >= 0
                && self.change_index < self.line_changes.num_changes - 1
                && at
                    >= (*self
                        .line_changes
                        .changes
                        .offset(self.change_index as isize + 1))
                    .dc_start[self.line_changes.bufidx as usize] as isize
            {
                self.change_index += 1;
            }
            let mut added = false;
            if self.line_changes.num_changes > 0
                && self.change_index >= 0
                && self.change_index < self.line_changes.num_changes
            {
                added = diff_change_parse(
                    &raw mut self.line_changes,
                    self.line_changes.changes.offset(self.change_index as isize),
                    &raw mut self.change_start,
                    &raw mut self.change_end,
                );
            }
            // Extra text (virtual text, say) takes the *line's* diff
            // highlight, never the changed-text one.
            if wlv.diff_hlf == HLF_CHD && at >= self.change_start as isize && wlv.extra_todo == 0 {
                wlv.diff_hlf = if added { HLF_TXA } else { HLF_TXD };
            }
            if (wlv.diff_hlf == HLF_TXD || wlv.diff_hlf == HLF_TXA)
                && ((at >= self.change_end as isize && wlv.extra_todo == 0)
                    || (wlv.extra_todo > 0 && wlv.extra_is_virt_text))
            {
                wlv.diff_hlf = HLF_CHD;
            }
            wlv.set_line_attr_for_diff(wp);
        }
    }

    /// Combine [`WinLineVars::extra_attr`] in, without overriding a Visual
    /// selection.
    pub(super) unsafe fn apply_extra_attr(&mut self, wlv: &mut WinLineVars) {
        if wlv.n_attr <= 0 || self.search_attr_from_match {
            return;
        }
        // SAFETY: `hl_combine_attr` only reads the attribute table.
        wlv.char_attr = unsafe { hl_combine_attr(wlv.char_attr, wlv.extra_attr) };
        if !wlv.reset_extra_attr {
            return;
        }
        wlv.reset_extra_attr = false;
        if self.extra_attr_next >= 0 {
            wlv.extra_attr = self.extra_attr_next;
            self.extra_attr_next = -1;
        } else {
            wlv.extra_attr = 0;
            // The extra attribute has been applied, so a `:match` may take
            // priority again.
            self.search_attr_from_match = self.saved_search_attr_from_match;
        }
    }

    /// Overlay `'cursorcolumn'` or `'colorcolumn'` on this cell, remembering
    /// what to put back afterwards.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(super) unsafe fn column_highlight(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window and 'colorcolumn' list.
        unsafe {
            self.attr_before_vcol_hl = -1;
            if self.lnum_in_visual_area
                || self.search_attr != 0
                || self.area_attr != 0
                || wlv.filler_todo > 0
            {
                return;
            }
            if (*wp).w_onebuf_opt.wo_cuc != 0
                && wlv.hl_vcol() == (*wp).w_virtcol
                && wlv.lnum != (*wp).w_cursor.lnum
            {
                self.attr_before_vcol_hl = wlv.char_attr;
                wlv.char_attr = hl_combine_attr(win_hl_attr(wp, HLF_CUC), wlv.char_attr);
            } else if !wlv.color_cols.is_null() && wlv.hl_vcol() == *wlv.color_cols {
                self.attr_before_vcol_hl = wlv.char_attr;
                wlv.char_attr = hl_combine_attr(win_hl_attr(wp, HLF_MC), wlv.char_attr);
            }
        }
    }

    /// Put the lowest-priority whole-line attribute underneath everything
    /// else — usually, but see the `CursorLine` exception below.
    pub(super) unsafe fn apply_line_attr_lowprio(&mut self, wlv: &mut WinLineVars) {
        // SAFETY: `hl_combine_attr` only reads the attribute table.
        unsafe {
            if wlv.filler_todo > 0 {
                return;
            }
            let mut low = wlv.line_attr_lowprio;
            let mut high = wlv.char_attr;
            if wlv.line_attr_lowprio != 0 {
                let line_ae = syn_attr2entry(wlv.line_attr_lowprio);
                let char_ae = syn_attr2entry(wlv.char_attr);
                // The window-local Normal background, which 'winhighlight' can
                // change.
                let (mut normal_rgb_bg, mut normal_cterm_bg) = (
                    normal_bg.get() as ::core::ffi::c_int,
                    cterm_normal_bg_color.get(),
                );
                if self.bg_attr != 0 {
                    let norm_ae = syn_attr2entry(self.bg_attr);
                    normal_rgb_bg = norm_ae.rgb_bg_color as ::core::ffi::c_int;
                    normal_cterm_bg = norm_ae.cterm_bg_color as ::core::ffi::c_int;
                }
                let char_is_normal_bg = if ui_rgb_attached() {
                    char_ae.rgb_bg_color == normal_rgb_bg as RgbValue
                } else {
                    char_ae.cterm_bg_color as ::core::ffi::c_int == normal_cterm_bg
                };
                // When the line has a background of its own (CursorLine) and the
                // character's is only Normal's, reverse the order so CursorLine
                // wins.
                if (line_ae.rgb_bg_color >= 0 as RgbValue
                    || line_ae.cterm_bg_color as ::core::ffi::c_int > 0)
                    && char_is_normal_bg
                {
                    low = wlv.char_attr;
                    high = wlv.line_attr_lowprio;
                }
            }
            wlv.char_attr = hl_combine_attr(low, high);
        }
    }
}
