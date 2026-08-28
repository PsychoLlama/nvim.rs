//! `win_line`'s character loop: the cell-by-cell half of drawing one buffer
//! line.
//!
//! The setup half ([`prepare_line`](super::prepare_line)) decides everything
//! that is true of the line as a whole and hands it over as a [`LineSetup`];
//! this module walks the line from left to right, one screen cell at a time,
//! for as many screen rows as the line occupies.
//!
//! One pass round the loop produces at most one cell. That is not the same as
//! one buffer character: a Tab, a `<xx>` escape, a `'listchars'` replacement
//! and an inline virtual text all feed [`WinLineVars::extra_todo`] and are then
//! spent one cell per pass, and a concealed character produces no cell at all.
//! The order of a pass is fixed and each step is a method here or in
//! [`chars`](super::chars):
//!
//! 1. ask the decoration providers for more of the line, if the read cursor
//!    has walked past what their last answer covered
//!    ([`Cells::provider_chunk`]);
//! 2. at the start of a screen row, draw the info columns
//!    ([`Cells::draw_columns`]) — fold, sign, number or `'statuscolumn'`,
//!    then `'breakindent'` and `'showbreak'`;
//! 3. work out the attribute this cell would take from everything that is not
//!    the character itself ([`Cells::cell_attributes`]): Visual, `'hlsearch'`,
//!    diff, decorations, `'cursorline'`;
//! 4. decide *which* character goes in the cell ([`Cells::next_char`], in
//!    `chars.rs`);
//! 5. store it ([`Cells::store_cell`]) and advance;
//! 6. when the row is full, hand the line buffer to the grid and start the
//!    next row ([`Cells::finish_screen_line`]); when the text runs out, do the
//!    same and stop ([`Cells::finish_line`]).
//!
//! Nothing here writes to the grid. Everything goes into the three line
//! buffers through [`put_cell`], and `wlv_put_linebuf` diffs the result
//! against what the grid already holds.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::r#move::WinValid;
use crate::pos::MAXCOL;
use crate::types::NUL;

/// The filler that stands in for the half of a double-width character that
/// did not fit, at either edge of the text.
pub(super) const MB_FILLER_CHAR: u8 = b'<';

/// What a phase of the loop wants the driver to do next.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    /// Carry on with the rest of this pass.
    Go,
    /// This buffer line is finished.
    Done,
    /// Start the next pass from the top of the loop.
    NextRow,
    /// Nothing more fits on this screen row; go straight to the end check.
    RowFull,
}

/// The parts of `win_line`'s own frame the character loop borrows.
///
/// All raw pointers on purpose: `fold_buf` in particular has its address
/// stored in [`WinLineVars::extra_text`] and read again on later passes, so it
/// may not be derived from a `&mut` that ends when the method does.
pub(crate) struct LineFrame {
    /// Row after the last one this buffer line may use.
    pub(crate) endrow: ::core::ffi::c_int,
    /// Non-zero when only the info columns are being redrawn, over this many
    /// screen rows.
    pub(crate) col_rows: ::core::ffi::c_int,
    /// The redraw's spell state.
    pub(crate) spv: *mut spellvars_T,
    /// `'statuscolumn'` request. Its `sattrs` points into
    /// [`WinLineVars::sign_attrs`].
    pub(crate) statuscol: *mut statuscol_T,
    /// Per-column attributes of a `:terminal` buffer's line,
    /// [`TERM_ATTRS_MAX`] of them, all zero for any other buffer.
    pub(crate) term_attrs: *const ::core::ffi::c_int,
    /// This line joined with the start of the next one, for spell checking.
    pub(crate) nextline: *mut ::core::ffi::c_char,
    /// [`FOLD_TEXT_LEN`] bytes of scratch for `'foldtext'`.
    pub(crate) fold_buf: *mut ::core::ffi::c_char,
}

/// The character loop's state.
///
/// The first half is [`LineSetup`] moved in whole — the setup half's answers,
/// which the loop reads and in a few cases updates (`line`/`ptr` are
/// re-fetched after anything that can run Lua, `has_decor` and `extra_check`
/// can be turned on by a decoration provider, the spell fields are the
/// checker's running position). The second half is the loop's own working
/// state, which upstream declares at the top of `win_line` because C has no
/// better place for it.
pub(crate) struct Cells {
    // -- the window and the line, decided by the setup half -----------------
    /// `w_view_width` of the window being drawn.
    pub(super) view_width: ::core::ffi::c_int,
    /// `w_view_height` of the window being drawn.
    pub(super) view_height: ::core::ffi::c_int,
    /// This is the cursor's own line in the current window.
    pub(super) in_curline: bool,
    /// The line is inside a closed fold.
    pub(super) has_fold: bool,
    /// The closed fold has a `'foldtext'` to draw instead of the line.
    pub(super) has_foldtext: bool,
    /// The line wraps rather than being cut off at the right edge.
    pub(super) is_wrapped: bool,
    /// There is buffer text to draw.
    pub(super) draw_text: bool,
    /// Virtual column the drawn part of the line starts at.
    pub(super) start_vcol: ::core::ffi::c_int,
    /// The window's background attribute.
    pub(super) bg_attr: ::core::ffi::c_int,
    /// `Conceal` attribute.
    pub(super) conceal_attr: ::core::ffi::c_int,
    /// The buffer has inline virtual text somewhere.
    pub(super) may_have_inline_virt: bool,

    // -- the text ------------------------------------------------------------
    /// The buffer line. Re-fetched after anything that can run Lua.
    pub(super) line: *mut ::core::ffi::c_char,
    /// Read cursor into [`Cells::line`].
    pub(super) ptr: *mut ::core::ffi::c_char,
    /// Byte index the `'listchars'` "trail" applies from, or `MAXCOL`.
    pub(super) trailcol: colnr_T,
    /// Byte index one past the leading whitespace, or 0.
    pub(super) leadcol: colnr_T,
    /// `'listchars'` "eol".
    pub(super) lcs_eol: schar_T,
    /// `'listchars'` "precedes", cleared once it has been drawn on a row.
    pub(super) lcs_prec_todo: schar_T,
    /// The skipped-over text ended inside a run of consecutive spaces.
    pub(super) in_multispace: bool,
    /// How far into the `'listchars'` "multispace" pattern that run is.
    pub(super) multispace_pos: ::core::ffi::c_int,

    // -- highlighting sources ------------------------------------------------
    /// Some whole-line or ranged highlighting applies.
    pub(super) area_highlighting: bool,
    /// Something on this line needs the slow path.
    pub(super) extra_check: bool,
    /// Syntax highlighting is running for this line.
    pub(super) has_syntax: bool,
    /// Extmark decorations apply to this line.
    pub(super) has_decor: bool,
    /// Attribute for the Visual or `'incsearch'` range.
    pub(super) vi_attr: ::core::ffi::c_int,
    /// Attribute from `'hlsearch'`, `:match` or insert-mode completion.
    pub(super) search_attr: ::core::ffi::c_int,
    /// [`Cells::search_attr`] came from `:match` rather than `'hlsearch'`.
    pub(super) search_attr_from_match: bool,
    /// The cursor has to stay visible, so inverting skips over it.
    pub(super) noinvcur: bool,
    /// Virtual column inverting resumes at after the skipped cursor, or `-2`.
    pub(super) fromcol_prev: ::core::ffi::c_int,
    /// The line is between the two ends of the Visual selection.
    pub(super) lnum_in_visual_area: bool,
    /// `'cursorlineopt'` is "screenline".
    pub(super) cul_screenline: bool,
    /// First virtual column of the cursor's screen row.
    pub(super) left_curline_col: ::core::ffi::c_int,
    /// One past its last virtual column.
    pub(super) right_curline_col: ::core::ffi::c_int,
    /// [`WinLineVars::line_attr`] as the setup half left it.
    pub(super) line_attr_save: ::core::ffi::c_int,
    /// [`WinLineVars::line_attr_lowprio`] likewise.
    pub(super) line_attr_lowprio_save: ::core::ffi::c_int,

    // -- diff mode -----------------------------------------------------------
    /// The changed byte ranges of this line.
    pub(super) line_changes: diffline_T,
    /// Which of them the loop is at, or `-1` when there are none.
    pub(super) change_index: ::core::ffi::c_int,
    /// First byte of the change the loop is at.
    pub(super) change_start: ::core::ffi::c_int,
    /// Last byte of it.
    pub(super) change_end: ::core::ffi::c_int,

    // -- decorations ----------------------------------------------------------
    /// Virtual lines to draw above or below this buffer line.
    pub(super) virt_lines: VirtLines,
    /// Decoration providers are being driven for this line.
    pub(super) check_decor_providers: bool,
    /// Byte column their last answer covered up to.
    pub(super) decor_provider_end_col: ::core::ffi::c_int,

    // -- spell checking --------------------------------------------------------
    /// Byte column [`LineFrame::nextline`] starts at, or `MAXCOL`.
    pub(super) nextlinecol: ::core::ffi::c_int,
    /// Index in it where the next line begins.
    pub(super) nextline_idx: ::core::ffi::c_int,
    /// Attribute for the badly spelled word being drawn.
    pub(super) spell_attr: ::core::ffi::c_int,
    /// Byte after the last one [`Cells::spell_attr`] applies to.
    pub(super) word_end: ::core::ffi::c_int,
    /// Byte column already checked, when a word wrapped from the line above.
    pub(super) cur_checked_col: ::core::ffi::c_int,

    // -- the loop's own state ---------------------------------------------------
    /// The info columns have not been drawn on this screen row yet.
    pub(super) columns_todo: bool,
    /// Screen column the buffer text starts at, past the info columns.
    pub(super) text_start_col: ::core::ffi::c_int,
    /// Width of the info columns on the *first* row of the line, which is how
    /// the loop notices a window too narrow to draw anything in.
    pub(super) left_columns_width: ::core::ffi::c_int,
    /// The `'listchars'` "eol" for this line has not been drawn yet.
    pub(super) lcs_eol_todo: bool,
    /// Index into [`Cells::virt_lines`] of the virtual line this row is, or
    /// `-1` when the row is not one.
    pub(super) virt_line_index: ::core::ffi::c_int,
    /// `kVL*` flags of that virtual line.
    pub(super) virt_line_flags: ::core::ffi::c_int,
    /// This row is the one that shows a closed fold.
    pub(super) draw_folded: bool,
    /// `'foldtext'` virtual text, drawn after the fold text itself.
    pub(super) fold_vt: VirtText,
    /// `'foldtext'` result that has to be freed, when it did not fit
    /// [`LineFrame::fold_buf`].
    pub(super) foldtext_free: *mut ::core::ffi::c_char,
    /// `Folded` attribute while a fold is being drawn.
    pub(super) fold_attr: ::core::ffi::c_int,

    /// The character being placed, as a screen cell.
    pub(super) cell_char: schar_T,
    /// Its first codepoint, or the character the cell came from.
    pub(super) char_code: ::core::ffi::c_int,
    /// Bytes it took in the buffer.
    pub(super) char_len: ::core::ffi::c_int,

    /// Virtual column of the previous pass, for the "not at a margin" tests
    /// that resume inverting after a skipped cursor.
    pub(super) prev_vcol: colnr_T,
    /// Attribute to restore when [`WinLineVars::n_attr`] runs out.
    pub(super) attr_before_run: ::core::ffi::c_int,
    /// Cells left of the `'listchars'` "precedes" attribute.
    pub(super) prec_attr_todo: ::core::ffi::c_int,
    /// Attribute to restore when that runs out.
    pub(super) attr_before_prec: ::core::ffi::c_int,
    /// High-priority half of the cell attribute: Visual, search, line.
    pub(super) attr_pri: ::core::ffi::c_int,
    /// Low-priority half: fold, syntax, decorations, spell.
    pub(super) attr_base: ::core::ffi::c_int,
    /// Visual or `'incsearch'` attribute while inside that range.
    pub(super) area_attr: ::core::ffi::c_int,
    /// Attribute to restore after the `'cursorcolumn'`/`'colorcolumn'`
    /// overlay, or `-1` when there was none.
    pub(super) attr_before_vcol_hl: ::core::ffi::c_int,
    /// Syntax and decoration attribute for this cell.
    pub(super) decor_attr: ::core::ffi::c_int,
    /// Attribute of the `>` that stands for a double-width character with
    /// nowhere to go, applied to that one cell only.
    pub(super) overflow_attr: ::core::ffi::c_int,
    /// Extmark attribute the decoration walk answered for this cell.
    pub(super) extmark_attr: ::core::ffi::c_int,
    /// One cell was added past the end of the line to carry a highlight.
    pub(super) eol_extra_cell: ::core::ffi::c_int,

    /// Inline virtual text lends [`WinLineVars::extra_todo`] to whatever was
    /// being drawn; these four hold what it displaced until it is spent.
    pub(super) saved_search_attr: ::core::ffi::c_int,
    /// See [`Cells::saved_search_attr`].
    pub(super) saved_area_attr: ::core::ffi::c_int,
    /// See [`Cells::saved_search_attr`].
    pub(super) saved_decor_attr: ::core::ffi::c_int,
    /// See [`Cells::saved_search_attr`].
    pub(super) saved_search_attr_from_match: bool,
    /// The rest of a run interrupted to draw a `<` filler, and the attribute
    /// to give it.
    pub(super) extra_todo_next: ::core::ffi::c_int,
    /// See [`Cells::extra_todo_next`]; `-1` for none.
    pub(super) extra_attr_next: ::core::ffi::c_int,

    /// The cell is inside the Visual or `'incsearch'` range.
    pub(super) area_active: bool,
    /// A wrapped row ended mid-decoration, so where its virtual texts go has
    /// to be decided again on the next row.
    pub(super) decor_need_recheck: bool,

    /// A `:match` wants this cell concealed (`> 1` at the start of the run).
    pub(super) has_match_conc: ::core::ffi::c_int,
    /// The character it wants shown instead.
    pub(super) match_conc: ::core::ffi::c_int,
    /// A decoration wants this cell concealed (`> 1` at the start).
    pub(super) decor_conceal: ::core::ffi::c_int,
    /// `'hlsearch'` says the match ends on the last column of the row.
    pub(super) on_last_col: bool,
    /// `SynFlags::CONCEAL` and friends for the syntax item under the cell.
    pub(super) syntax_flags: SynFlags,
    /// Sequence number of that item, so a run of one item conceals once.
    pub(super) syntax_seqnr: ::core::ffi::c_int,
    /// The one the last concealed cell belonged to.
    pub(super) prev_syntax_id: ::core::ffi::c_int,
    /// This cell is being concealed, rather than merely skipped.
    pub(super) is_concealing: bool,
    /// The read cursor was pushed back so that a double-width character is
    /// drawn at the start of the next row.
    pub(super) did_decrement_ptr: bool,
    /// The cursor's screen position has been worked out for this line.
    pub(super) did_cursor_col: bool,
}

impl Cells {
    /// Take over from the setup half.
    pub(crate) fn new(s: LineSetup) -> Self {
        Cells {
            view_width: s.view_width,
            view_height: s.view_height,
            in_curline: s.in_curline,
            has_fold: s.has_fold,
            has_foldtext: s.has_foldtext,
            is_wrapped: s.is_wrapped,
            draw_text: s.draw_text,
            start_vcol: s.start_vcol,
            bg_attr: s.bg_attr,
            conceal_attr: s.conceal_attr,
            may_have_inline_virt: s.may_have_inline_virt,
            line: s.line,
            ptr: s.ptr,
            trailcol: s.trailcol,
            leadcol: s.leadcol,
            lcs_eol: s.lcs_eol,
            lcs_prec_todo: s.lcs_prec_todo,
            in_multispace: s.in_multispace,
            multispace_pos: s.multispace_pos,
            area_highlighting: s.area_highlighting,
            extra_check: s.extra_check,
            has_syntax: s.has_syntax,
            has_decor: s.has_decor,
            vi_attr: s.vi_attr,
            search_attr: s.search_attr,
            search_attr_from_match: s.search_attr_from_match,
            noinvcur: s.noinvcur,
            fromcol_prev: s.fromcol_prev,
            lnum_in_visual_area: s.lnum_in_visual_area,
            cul_screenline: s.cul_screenline,
            left_curline_col: s.left_curline_col,
            right_curline_col: s.right_curline_col,
            line_attr_save: s.line_attr_save,
            line_attr_lowprio_save: s.line_attr_lowprio_save,
            line_changes: s.line_changes,
            change_index: s.change_index,
            change_start: s.change_start,
            change_end: s.change_end,
            virt_lines: s.virt_lines,
            check_decor_providers: s.check_decor_providers,
            decor_provider_end_col: s.decor_provider_end_col,
            nextlinecol: s.nextlinecol,
            nextline_idx: s.nextline_idx,
            spell_attr: s.spell_attr,
            word_end: s.word_end,
            cur_checked_col: s.cur_checked_col,

            columns_todo: true,
            text_start_col: 0,
            left_columns_width: 0,
            lcs_eol_todo: true,
            virt_line_index: -1,
            virt_line_flags: 0,
            draw_folded: false,
            fold_vt: VIRTTEXT_EMPTY,
            foldtext_free: ::core::ptr::null_mut(),
            fold_attr: 0,

            cell_char: 0,
            char_code: 0,
            char_len: 1,

            prev_vcol: -1,
            attr_before_run: 0,
            prec_attr_todo: 0,
            attr_before_prec: 0,
            attr_pri: 0,
            attr_base: 0,
            area_attr: 0,
            attr_before_vcol_hl: 0,
            decor_attr: 0,
            overflow_attr: 0,
            extmark_attr: 0,
            eol_extra_cell: 0,

            saved_search_attr: 0,
            saved_area_attr: 0,
            saved_decor_attr: 0,
            saved_search_attr_from_match: false,
            extra_todo_next: 0,
            extra_attr_next: -1,

            area_active: false,
            decor_need_recheck: false,

            has_match_conc: 0,
            match_conc: 0,
            decor_conceal: 0,
            on_last_col: false,
            syntax_flags: SynFlags::NONE,
            syntax_seqnr: 0,
            prev_syntax_id: 0,
            is_concealing: false,
            did_decrement_ptr: false,
            did_cursor_col: false,
        }
    }

    /// Byte index of the read cursor in the line.
    #[inline(always)]
    pub(super) fn byte_col(&self) -> ::core::ffi::c_int {
        // SAFETY: `ptr` always points into `line`.
        unsafe { self.ptr.offset_from(self.line) as ::core::ffi::c_int }
    }

    /// Re-fetch the line after something that could have freed it, keeping the
    /// read cursor where it was.
    ///
    /// # Safety
    /// `wp` must be a live window and `lnum` one of its buffer's lines.
    #[inline]
    pub(super) unsafe fn refetch_line(&mut self, wp: Win, lnum: linenr_T, at: ::core::ffi::c_int) {
        // SAFETY: the caller's window and line.
        self.line = unsafe { ml_get_buf(wp.w_buffer, lnum) };
        self.ptr = unsafe { self.line.offset(at as isize) };
    }

    // -----------------------------------------------------------------------
    // The driver
    // -----------------------------------------------------------------------

    /// Draw the whole buffer line and answer the window row after it.
    ///
    /// `#[inline(always)]` is load-bearing and measured, not a hint. There is
    /// exactly one caller ([`win_line`](super::win_line)), and this is one
    /// half of the function upstream writes as a single body; when the
    /// inliner declines — which it did from `39579d0db5`, a commit that
    /// touches no drawing file at all — `Cells` and `WinLineVars` stop being
    /// one frame's worth of locals and the per-cell loop reloads them through
    /// pointers. Measured on a 600-round `redraw!` of 400 wrapped lines
    /// (`perf stat -e instructions:u`, which repeats to 0.03 %): **2,834.1 M
    /// instructions without the attribute, 2,782.6 M with it, −1.8 %**. Adding
    /// it to [`Cells::new`] as well costs 5.8 M back, so it stays here only.
    ///
    /// # Safety
    /// `wp`, `buf` and everything in `f` must be live, and `wlv` must be the
    /// state the setup half filled in for this line.
    #[inline(always)]
    pub(crate) unsafe fn run(
        &mut self,
        wlv: &mut WinLineVars,
        wp: Win,
        buf: *mut buf_T,
        f: &LineFrame,
    ) -> ::core::ffi::c_int {
        // SAFETY: the caller's window, buffer, line state and frame.
        let grid: GridView = wp.w_grid;
        'row: loop {
            self.has_match_conc = 0;
            self.decor_conceal = 0;
            self.did_decrement_ptr = false;
            unsafe { self.provider_chunk(wp, wlv.lnum, wlv.decor) };

            'row_full: {
                if self.columns_todo {
                    match unsafe { self.draw_columns(wlv, wp, f) } {
                        Step::Done => break 'row,
                        Step::NextRow => continue 'row,
                        Step::RowFull => break 'row_full,
                        Step::Go => {}
                    }
                }

                if self.cul_screenline
                    && wlv.filler_todo <= 0
                    && wlv.vcol >= self.left_curline_col
                    && wlv.vcol < self.right_curline_col
                {
                    unsafe { wlv.apply_cursorline_highlight(wp) };
                }

                // Still showing the '$' of a change command: stop at the
                // cursor.
                if dollar_vcol.get() >= 0 && self.in_curline && wlv.vcol >= wp.w_virtcol {
                    wlv.col = unsafe { draw_virt_text(wp, buf, self.text_start_col, wlv.col, wlv) };
                    // Nothing after `col` is ours to clear.
                    unsafe { wlv_put_linebuf(wp, wlv, wlv.col, false, self.bg_attr, 0) };
                    // Pretend the window is finished, except that
                    // 'cursorcolumn' still wants the rest of it.
                    wlv.row = if wp.w_onebuf_opt.wo_cuc != 0 {
                        wp.w_cline_row + wp.w_cline_height
                    } else {
                        self.view_height
                    };
                    break 'row;
                }

                self.draw_folded = self.has_fold && wlv.row == wlv.startrow + wlv.filler_lines;
                if self.draw_folded && wlv.extra_todo == 0 {
                    self.fold_attr = unsafe { win_hl_attr(wp.raw(), HLF_FL) };
                    wlv.char_attr = self.fold_attr;
                    self.decor_attr = 0;
                }

                self.extmark_attr = 0;
                if wlv.filler_todo <= 0
                    && (self.area_highlighting
                        || unsafe { (*f.spv).spv_has_spell }
                        || self.extra_check)
                {
                    unsafe { self.cell_attributes(wlv, wp) };
                }

                unsafe { self.fold_text(wlv, wp, f) };
                unsafe { self.next_char(wlv, wp, f) };
                unsafe { self.correct_cursor_col(wlv, wp) };
                unsafe { self.apply_extra_attr(wlv) };
                unsafe { self.draw_precedes(wlv, wp) };
                unsafe { self.highlight_at_eol(wlv, wp) };

                if self.cell_char == NUL as schar_T {
                    unsafe { self.finish_line(wlv, wp, buf, f) };
                    break 'row;
                }

                unsafe { self.draw_extends(wlv, wp) };
                unsafe { wlv.advance_color_col(wlv.hl_vcol()) };
                unsafe { self.column_highlight(wlv, wp) };
                unsafe { self.apply_line_attr_lowprio(wlv) };
                if wlv.filler_todo <= 0 {
                    self.prev_vcol = wlv.vcol;
                }
                unsafe { self.store_cell(wlv, wp) };
                self.advance_vcol(wlv);
                unsafe { self.peek_decor_past_edge(wlv, wp) };
            }

            if !unsafe { self.row_is_full(wlv, wp) } {
                continue 'row;
            }
            if unsafe { self.finish_screen_line(wlv, wp, buf, f, grid) } == Step::Done {
                break 'row;
            }
        }

        unsafe { clear_virttext(&raw mut self.fold_vt) };
        unsafe { xfree(self.virt_lines.items.cast::<::core::ffi::c_void>()) };
        self.virt_lines = VirtLines {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<virt_line>(),
        };
        unsafe { xfree(self.foldtext_free.cast::<::core::ffi::c_void>()) };
        wlv.row
    }

    // -----------------------------------------------------------------------
    // Phases
    // -----------------------------------------------------------------------

    /// Ask the decoration providers for the next chunk of the line once the
    /// read cursor has walked past what their last answer covered.
    ///
    /// # Safety
    /// `wp` must be a live window and `lnum` one of its buffer's lines.
    pub(super) unsafe fn provider_chunk(&mut self, wp: Win, lnum: linenr_T, decor: DecorStateRef) {
        // SAFETY: the caller's window and line.
        if !self.check_decor_providers || self.byte_col() < self.decor_provider_end_col {
            return;
        }
        let at = self.byte_col();
        self.decor_provider_end_col = unsafe { invoke_range_next(wp, lnum, at, 100) };
        unsafe { self.refetch_line(wp, lnum, at) };
        if !self.has_decor && decor_has_more_decorations(decor, lnum - 1) {
            self.has_decor = true;
            self.extra_check = true;
        }
    }

    /// Work out the cursor's screen position while drawing its own line.
    ///
    /// Concealing means the cursor's screen column cannot be computed from the
    /// virtual column alone, so it is read off here as the loop reaches it —
    /// or, under `'virtualedit'`, at the end of the line, which the cursor may
    /// be past.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(super) unsafe fn correct_cursor_col(&mut self, wlv: &WinLineVars, mut wp: Win) {
        // SAFETY: the caller's window.
        if self.did_cursor_col
            || wlv.filler_todo > 0
            || !self.in_curline
            || !unsafe { conceal_cursor_line(wp.raw()) }
            || !(wlv.vcol + wlv.skip_cells >= wp.w_virtcol || self.cell_char == NUL as schar_T)
        {
            return;
        }
        wp.w_wcol = wlv.col - wlv.boguscols;
        if wlv.vcol + wlv.skip_cells < wp.w_virtcol {
            // Cursor beyond the end of the line with 'virtualedit'.
            wp.w_wcol += wp.w_virtcol - wlv.vcol - wlv.skip_cells;
        }
        wp.w_wrow = wlv.row;
        self.did_cursor_col = true;
        wp.w_valid |= WinValid::WCOL | WinValid::WROW | WinValid::VIRTCOL;
    }

    /// Write the cell — or, when it is being concealed or skipped over, count
    /// it without writing anything.
    ///
    /// # Safety
    /// `wp` must be a live window and `off` inside the line buffers.
    pub(super) unsafe fn store_cell(&mut self, wlv: &mut WinLineVars, wp: Win) {
        // SAFETY: the caller's window; `off` is bounded by `view_width`.
        if wlv.filler_todo > 0 {
            // TODO(bfredl): the main render loop should get called with
            // the virtual line chunks too, so they get line wrapping and
            // other Nice Things.
            return;
        }
        if wlv.skip_cells <= 0 {
            let attr = if self.overflow_attr != 0 {
                let a = self.overflow_attr;
                self.overflow_attr = 0;
                a
            } else {
                wlv.char_attr
            };
            put_cell(wlv.off, self.cell_char, attr, wlv.vcol);
            if unsafe { schar_cells(self.cell_char) } > 1 {
                // A double-width character needs two screen columns; the
                // second carries a 0 and the same attribute.
                wlv.off += 1;
                wlv.col += 1;
                wlv.vcol += 1;
                put_cell(wlv.off, 0, attr, wlv.vcol);
                // When "tocol" is halfway through a character, put it at
                // the end of it, or highlighting would not stop.
                if wlv.tocol == wlv.vcol {
                    wlv.tocol += 1;
                }
            }
            wlv.off += 1;
            wlv.col += 1;
        } else if wp.w_onebuf_opt.wo_cole > 0 && self.is_concealing {
            self.skip_concealed(wlv, unsafe { schar_cells(self.cell_char) } > 1);
        } else {
            wlv.skip_cells -= 1;
        }
    }

    /// Account for a concealed cell that is drawn as nothing.
    ///
    /// With `'wrap'` the column indicator is advanced anyway, so that the line
    /// takes the same screen space as it would unconcealed and the cursor
    /// arithmetic elsewhere still works. `boguscols` counts how far it was
    /// advanced for nothing, so that the trailing junk is never written out.
    pub(super) fn skip_concealed(&mut self, wlv: &mut WinLineVars, concealed_wide: bool) {
        wlv.skip_cells -= 1;
        wlv.vcol_off_co += 1;
        if concealed_wide {
            // A concealed double-width character swallows one more virtual
            // column.
            wlv.vcol += 1;
            wlv.vcol_off_co += 1;
        }
        if wlv.extra_todo > 0 {
            wlv.vcol_off_co += wlv.extra_todo;
        }

        if self.is_wrapped {
            if wlv.extra_todo > 0 {
                wlv.vcol += wlv.extra_todo;
                wlv.col += wlv.extra_todo;
                wlv.boguscols += wlv.extra_todo;
                wlv.extra_todo = 0;
                wlv.n_attr = 0;
            }
            if concealed_wide {
                wlv.boguscols += 1;
                wlv.col += 1;
            }
            wlv.boguscols += 1;
            wlv.col += 1;
        } else if wlv.extra_todo > 0 {
            wlv.vcol += wlv.extra_todo;
            wlv.extra_todo = 0;
            wlv.n_attr = 0;
        }
    }

    /// Advance the virtual column and put back the attributes that applied to
    /// a fixed number of cells.
    pub(super) fn advance_vcol(&mut self, wlv: &mut WinLineVars) {
        // Cells skipped for virtual text are counted here rather than as they
        // were skipped.
        if wlv.skipped_cells > 0 {
            wlv.vcol += wlv.skipped_cells;
            wlv.skipped_cells = 0;
        }
        // Only past the 'number'/'relativenumber' column does vcol move.
        if wlv.filler_todo <= 0 {
            wlv.vcol += 1;
        }
        if self.attr_before_vcol_hl >= 0 {
            wlv.char_attr = self.attr_before_vcol_hl;
        }
        if self.prec_attr_todo > 0 {
            self.prec_attr_todo -= 1;
            if self.prec_attr_todo == 0 {
                wlv.char_attr = self.attr_before_prec;
            }
        }
        if wlv.n_attr > 0 {
            wlv.n_attr -= 1;
            if wlv.n_attr == 0 {
                wlv.char_attr = self.attr_before_run;
            }
        }
    }

    /// At the right edge of a screen row, look for decorations that sit just
    /// past it.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(super) unsafe fn peek_decor_past_edge(&mut self, wlv: &WinLineVars, wp: Win) {
        // SAFETY: the caller's window and the redraw's decoration state.
        if !self.has_decor || wlv.filler_todo > 0 || wlv.col < self.view_width {
            return;
        }
        if self.is_wrapped && wlv.extra_todo == 0 {
            unsafe {
                decor_redraw_col(
                    wp.raw(),
                    self.byte_col(),
                    -3,
                    false,
                    wlv.decor,
                    self.decor_provider_end_col - 1,
                )
            };
            // Where they go has to be decided again on the next row.
            self.decor_need_recheck = true;
        } else if !self.is_wrapped {
            // Without wrapping, "right_align" and "win_col" virtual texts
            // for the whole line still have to be placed.
            decor_recheck_draw_col(-1, true, wlv.decor);
            unsafe {
                decor_redraw_col(
                    wp.raw(),
                    MAXCOL as ::core::ffi::c_int,
                    -1,
                    true,
                    wlv.decor,
                    self.decor_provider_end_col - 1,
                )
            };
        }
    }
}
