//! [`WinLineVars`] — the state `win_line` passes to everything else.
//!
//! One screen line is drawn by `win_line`, but almost none of the work is done
//! there: the fold column, the sign column, the number column, `'statuscolumn'`,
//! `'breakindent'`, `'showbreak'` and the virtual texts are all separate
//! functions that advance the same cursor through the same line buffer. This
//! module owns the struct they share and the operations on it that belong to no
//! one column kind — starting a screen line ([`WinLineVars::start_line`]),
//! undoing the fake columns concealment inserts
//! ([`WinLineVars::fix_for_boguscols`]), walking the `'colorcolumn'` list
//! ([`WinLineVars::advance_color_col`]) and answering how far right anything
//! needs to be drawn ([`get_rightmost_vcol`]).
//!
//! ## The line buffer
//!
//! Drawing does not write to the grid. It fills three parallel arrays indexed
//! by screen column — `linebuf_char`, `linebuf_attr` and `linebuf_vcol` — and
//! `grid_put_linebuf` diffs the result against what the grid already holds.
//! [`WinLineVars::off`] is the write cursor into them; [`put_cell`] is the one
//! place all three are written together.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

/// The variables `win_line` passes to the functions that draw parts of a line.
///
/// `#[repr(C)]` is not required of it any more — nothing outside this family
/// names the type — but the field order is upstream's and is left alone so the
/// C can still be read beside it.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WinLineVars {
    /// Buffer line being drawn.
    pub lnum: linenr_T,
    /// Fold state of `lnum`, from `win_update`.
    pub foldinfo: foldinfo_T,

    /// First window row this buffer line occupies.
    pub startrow: ::core::ffi::c_int,
    /// Window row being drawn, excluding `w_winrow`.
    pub row: ::core::ffi::c_int,

    /// Virtual column in the buffer line, before wrapping.
    pub vcol: colnr_T,
    /// Screen column, after wrapping.
    pub col: ::core::ffi::c_int,
    /// Nonexistent columns added to `col` to force a wrap.
    pub boguscols: ::core::ffi::c_int,
    /// What `boguscols` held before [`WinLineVars::fix_for_boguscols`].
    pub old_boguscols: ::core::ffi::c_int,
    /// Virtual columns swallowed by concealed text.
    pub vcol_off_co: ::core::ffi::c_int,

    /// Write cursor into the three line buffers.
    pub off: ::core::ffi::c_int,

    /// `'cursorline'` attribute, 0 when it does not apply here.
    pub cursorline_attr: ::core::ffi::c_int,
    /// Attribute for the whole line.
    pub line_attr: ::core::ffi::c_int,
    /// Low-priority attribute for the whole line.
    pub line_attr_lowprio: ::core::ffi::c_int,
    /// Number-column attribute from a sign's `numhl`.
    pub sign_num_attr: ::core::ffi::c_int,
    /// The previous line's `sign_num_attr`, for virtual lines that belong to
    /// it. `-1` until [`WinLineVars::line_number_attr`] looks it up.
    pub prev_num_attr: ::core::ffi::c_int,
    /// Sign attribute from a sign's `culhl`.
    pub sign_cul_attr: ::core::ffi::c_int,

    /// Start of the inverted (Visual/`'incsearch'`) range.
    pub fromcol: ::core::ffi::c_int,
    /// End of the inverted range.
    pub tocol: ::core::ffi::c_int,

    /// Virtual column just after `'showbreak'`.
    pub showbreak_vcol: colnr_T,
    /// This row still owes a `'showbreak'`.
    pub need_showbreak: bool,

    /// Attribute for the next character.
    pub char_attr: ::core::ffi::c_int,

    /// How much of the run in [`WinLineVars::extra_text`] or
    /// [`WinLineVars::extra_fill`] is still to be drawn — bytes for the
    /// former, repeats for the latter.
    pub extra_todo: ::core::ffi::c_int,
    /// Characters left that take `extra_attr`.
    pub n_attr: ::core::ffi::c_int,
    /// Text to draw instead of buffer text; only used when `extra_fill` and
    /// `extra_last` are NUL.
    pub extra_text: *mut ::core::ffi::c_char,
    /// Attribute for `extra_text`.
    pub extra_attr: ::core::ffi::c_int,
    /// One character repeated `extra_todo` times.
    pub extra_fill: schar_T,
    /// Mandatory last character of a repeated run, when set.
    pub extra_last: schar_T,

    /// The run came from inline virtual text rather than from a form of the
    /// buffer's own text, which is what decides whether the diff and Visual
    /// highlighting apply to it.
    pub extra_is_virt_text: bool,

    /// Scratch for the `<xx>` form of one character; must be as large as
    /// `transchar_charbuf` in charset.c.
    pub escape_buf: [::core::ffi::c_char; 11],

    /// Kind of diff highlighting, `HLF_NONE` for none.
    pub diff_hlf: hlf_T,

    /// Virtual lines to draw for this buffer line.
    pub n_virt_lines: ::core::ffi::c_int,
    /// How many of them belong to the *previous* buffer line.
    pub n_virt_below: ::core::ffi::c_int,
    /// Filler lines (diff or virtual) to draw.
    pub filler_lines: ::core::ffi::c_int,
    /// Filler lines still to do, plus one.
    pub filler_todo: ::core::ffi::c_int,
    /// Signs to show in the sign column.
    pub sign_attrs: [SignTextAttrs; 9],
    /// A character outside `'breakat'` has been seen on this line, so
    /// `'linebreak'` may now break at the next blank. Leading indent must not
    /// arm it, or every long word would be broken at column zero.
    pub linebreak_armed: bool,

    /// Inline virtual text being fed to the character loop.
    pub virt_inline: VirtText,
    /// How far into `virt_inline` that feed has got.
    pub virt_inline_i: size_t,
    /// `hl_mode` of `virt_inline`.
    pub virt_inline_hl_mode: HlMode,

    /// `extra_attr` applies to one character only.
    pub reset_extra_attr: bool,

    /// Cells still to skip for `w_leftcol`, `w_skipcol` or concealing.
    pub skip_cells: ::core::ffi::c_int,
    /// Cells skipped for virtual text, to be added to `vcol` later.
    pub skipped_cells: ::core::ffi::c_int,

    /// Cursor into `w_p_cc_cols`, or null once it is past the last one. The
    /// array is terminated by a negative entry.
    pub color_cols: *mut ::core::ffi::c_int,
}

/// How many bytes of the next line the spell checker joins on, as a `usize`.
pub(crate) const SPELL_LOOKAHEAD: usize = SPWORDLEN as usize;

/// The tail of a line followed by the start of the next one, so that a word
/// wrapping across the line break can be spell-checked whole.
///
/// The next line arrives at offset [`SPWORDLEN`] and the two halves are then
/// joined at [`LineSetup::nextline_idx`]. Lives in `win_line`'s frame — too
/// large to hand over by value.
pub(crate) type SpellLookahead = [::core::ffi::c_char; SPELL_LOOKAHEAD * 2];

/// What the setup half tells the character loop.
///
/// [`WinLineVars`] carries the state the two halves *share* and that the
/// column drawers also touch; this carries the rest — facts about the window
/// and the line, and the highlighting sources found for it.
pub(crate) struct LineSetup {
    // -- the window and the line, decided once ------------------------------
    /// `w_view_width` of the window being drawn.
    pub(crate) view_width: ::core::ffi::c_int,
    /// `w_view_height` of the window being drawn.
    pub(crate) view_height: ::core::ffi::c_int,
    /// This is the cursor's own line in the current window.
    pub(crate) in_curline: bool,
    /// The line is inside a closed fold.
    pub(crate) has_fold: bool,
    /// The closed fold has a `'foldtext'` to draw instead of the line.
    pub(crate) has_foldtext: bool,
    /// The line wraps rather than being cut off at the right edge.
    pub(crate) is_wrapped: bool,
    /// There is buffer text to draw: the line is not concealed and is not the
    /// one-past-the-end line that exists only to carry filler.
    pub(crate) draw_text: bool,
    /// Virtual column the drawn part of the line starts at — `w_skipcol` on a
    /// wrapped first row, `w_leftcol` without `'wrap'`, otherwise 0.
    pub(crate) start_vcol: ::core::ffi::c_int,
    /// The window's background attribute, applied to everything.
    pub(crate) bg_attr: ::core::ffi::c_int,
    /// `Conceal` attribute, looked up before anything else so that its id is
    /// upstream's.
    pub(crate) conceal_attr: ::core::ffi::c_int,
    /// The buffer has inline virtual text somewhere, so the loop has to ask
    /// for it per character.
    pub(crate) may_have_inline_virt: bool,
    /// This is a `:terminal` buffer, so `win_line` has to fill its own
    /// `term_attrs`.
    ///
    /// The 4 KiB array is deliberately NOT part of the setup's answer: left
    /// to `win_line`, its zeroing sinks into this branch, and on every other
    /// line — which is nearly all of them — it costs nothing. Filling it here
    /// instead cost a 4 KiB `memset` per buffer line drawn, worth up to 8% of
    /// a redraw.
    pub(crate) has_terminal: bool,

    // -- the text ------------------------------------------------------------
    /// The buffer line, or the empty string when there is no text to draw.
    /// Anything that can run Lua invalidates it, so it is re-fetched after.
    pub(crate) line: *mut ::core::ffi::c_char,
    /// Read cursor into [`LineSetup::line`], already advanced past whatever is
    /// scrolled off to the left.
    pub(crate) ptr: *mut ::core::ffi::c_char,
    /// Byte index where the trailing whitespace `'listchars'` "trail" applies
    /// to starts, or `MAXCOL` when it does not apply.
    pub(crate) trailcol: colnr_T,
    /// Byte index one past the leading whitespace, or 0 when "lead" does not
    /// apply.
    pub(crate) leadcol: colnr_T,
    /// `'listchars'` "eol".
    pub(crate) lcs_eol: schar_T,
    /// `'listchars'` "prec", cleared by the loop once it has been drawn.
    pub(crate) lcs_prec_todo: schar_T,
    /// The skipped-over text ended inside a run of consecutive spaces.
    pub(crate) in_multispace: bool,
    /// How far into the `'listchars'` "multispace" pattern that run is.
    pub(crate) multispace_pos: ::core::ffi::c_int,

    // -- highlighting sources found for this line ---------------------------
    /// Some whole-line or ranged highlighting applies, so the loop has to
    /// recompute the attribute per cell.
    pub(crate) area_highlighting: bool,
    /// Something on this line needs the character loop's slow path.
    pub(crate) extra_check: bool,
    /// Syntax highlighting is running for this line.
    pub(crate) has_syntax: bool,
    /// Extmark decorations apply to this line.
    pub(crate) has_decor: bool,
    /// Attribute for the Visual or `'incsearch'` range.
    pub(crate) vi_attr: ::core::ffi::c_int,
    /// Attribute from `'hlsearch'`, `:match` or insert-mode completion.
    pub(crate) search_attr: ::core::ffi::c_int,
    /// [`LineSetup::search_attr`] came from `:match` rather than `'hlsearch'`.
    pub(crate) search_attr_from_match: bool,
    /// The cursor has to stay visible, so inverting skips over it.
    pub(crate) noinvcur: bool,
    /// Virtual column inverting resumes at after the skipped cursor, or `-2`.
    pub(crate) fromcol_prev: ::core::ffi::c_int,
    /// The line is between the two ends of the Visual selection.
    pub(crate) lnum_in_visual_area: bool,
    /// `'cursorlineopt'` is "screenline", so `'cursorline'` applies to the
    /// cursor's screen row rather than to the whole buffer line.
    pub(crate) cul_screenline: bool,
    /// First virtual column of the cursor's screen row, for that mode.
    pub(crate) left_curline_col: ::core::ffi::c_int,
    /// One past its last virtual column.
    pub(crate) right_curline_col: ::core::ffi::c_int,
    /// [`WinLineVars::line_attr`] as the setup left it; the loop restores it
    /// after a virtual line has borrowed the field.
    pub(crate) line_attr_save: ::core::ffi::c_int,
    /// [`WinLineVars::line_attr_lowprio`] likewise.
    pub(crate) line_attr_lowprio_save: ::core::ffi::c_int,

    // -- diff mode -----------------------------------------------------------
    /// The changed byte ranges of this line, in diff mode.
    pub(crate) line_changes: diffline_T,
    /// Which of them the loop is at, or `-1` when there are none.
    pub(crate) change_index: ::core::ffi::c_int,
    /// First byte of the change the loop is at.
    pub(crate) change_start: ::core::ffi::c_int,
    /// Last byte of it.
    pub(crate) change_end: ::core::ffi::c_int,

    // -- columns and decorations ---------------------------------------------
    /// `'statuscolumn'` request; `draw` is false when the option is empty.
    ///
    /// Its `sattrs` is left null here and attached by `win_line` itself: the
    /// signs it points at are [`WinLineVars::sign_attrs`], which lives in
    /// `win_line`'s own frame, and a pointer derived from the `&mut` the
    /// setup half borrows would not outlive that borrow.
    pub(crate) statuscol: statuscol_T,
    /// Virtual lines to draw above or below this buffer line.
    pub(crate) virt_lines: VirtLines,
    /// Decoration providers are being driven for this line.
    pub(crate) check_decor_providers: bool,
    /// Byte column their last answer covered up to; past it the loop asks for
    /// the next chunk.
    pub(crate) decor_provider_end_col: ::core::ffi::c_int,

    // -- spell checking -------------------------------------------------------
    /// Byte column [`LineScratch::nextline`] starts at, or `MAXCOL` when there
    /// is no next line to join on.
    pub(crate) nextlinecol: ::core::ffi::c_int,
    /// Index in it where the next line begins.
    pub(crate) nextline_idx: ::core::ffi::c_int,
    /// Attribute for the badly spelled word being drawn.
    pub(crate) spell_attr: ::core::ffi::c_int,
    /// Byte after the last one [`LineSetup::spell_attr`] applies to.
    pub(crate) word_end: ::core::ffi::c_int,
    /// Byte column already checked, when a word wrapped from the line above.
    pub(crate) cur_checked_col: ::core::ffi::c_int,
}
// ---------------------------------------------------------------------------
// The line buffer
// ---------------------------------------------------------------------------

/// Write one cell of the line under construction.
///
/// `vcol` is the buffer virtual column the cell came from, or `-1` for a cell
/// that is not buffer text. Everything left of the text writes `-1` (or the
/// fold column's own `-2`/`-3` markers, which `'mouse'` handling reads back).
///
/// # Safety
/// `off` must be under `w_view_width` of the window being drawn, and the line
/// buffers must be sized for it — `grid_alloc` keeps them at least that wide.
#[inline(always)]
pub(crate) unsafe fn put_cell(
    off: ::core::ffi::c_int,
    ch: schar_T,
    attr: ::core::ffi::c_int,
    vcol: colnr_T,
) {
    // SAFETY: the caller's bound.
    unsafe {
        let at = off as usize;
        *linebuf_char.get().add(at) = ch;
        *linebuf_attr.get().add(at) = attr as sattr_T;
        *linebuf_vcol.get().add(at) = vcol;
    }
}

// ---------------------------------------------------------------------------
// Scratch and small window queries
// ---------------------------------------------------------------------------

/// The scratch buffer `win_line` renders a fold text or a `'listchars'`
/// replacement into. One allocation for the process, grown as needed.
static extra_buf: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());

/// How many bytes [`extra_buf`] holds.
static extra_buf_size: GlobalCell<size_t> = GlobalCell::new(0);

/// A scratch buffer of at least `size` bytes, valid until the next call.
///
/// # Safety
/// The answer may not be held across another call.
pub(crate) unsafe fn get_extra_buf(size: size_t) -> *mut ::core::ffi::c_char {
    let size = size.max(64);
    // SAFETY: the buffer is only ever reached through this function, so
    // nothing can hold the old pointer across the reallocation.
    unsafe {
        if extra_buf_size.get() < size {
            xfree(extra_buf.get().cast::<::core::ffi::c_void>());
            extra_buf.set(xmalloc(size).cast::<::core::ffi::c_char>());
            extra_buf_size.set(size);
        }
    }
    extra_buf.get()
}

/// The `'listchars'` "extends" character for `wp`, or NUL if it should not be
/// used.
///
/// # Safety
/// `wp` must be a live window.
pub(crate) unsafe fn get_lcs_ext(wp: *mut win_T) -> schar_T {
    // SAFETY: the caller's window.
    unsafe {
        if (*wp).w_onebuf_opt.wo_wrap != 0 {
            // With 'wrap' a line never continues past the right of the screen.
            return NUL as schar_T;
        }
        if (*wp).w_onebuf_opt.wo_wrap_flags & kOptFlagInsecure as uint32_t != 0 {
            // 'nowrap' set from a modeline: forcibly use '>'.
            return schar_from_ascii(b'>');
        }
        if (*wp).w_onebuf_opt.wo_list != 0 {
            (*wp).w_p_lcs_chars.ext
        } else {
            NUL as schar_T
        }
    }
}

/// The rightmost virtual column anything wants drawn at, so `win_line` knows
/// how far past the end of a short line it still has work to do.
///
/// `color_cols` is `w_p_cc_cols`: an array of `'colorcolumn'` virtual columns
/// terminated by a negative entry, or null when the option is empty.
///
/// # Safety
/// `wp` must be live and `color_cols` must be null or so terminated.
pub(crate) unsafe fn get_rightmost_vcol(
    wp: *mut win_T,
    color_cols: *const ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    // SAFETY: the caller's window and terminated array.
    unsafe {
        let mut ret = if (*wp).w_onebuf_opt.wo_cuc != 0 {
            (*wp).w_virtcol
        } else {
            0
        };
        if !color_cols.is_null() {
            let mut i = 0;
            while *color_cols.add(i) >= 0 {
                ret = ret.max(*color_cols.add(i));
                i += 1;
            }
        }
        ret
    }
}

/// The screen columns `'cursorlineopt'` "screenline" highlights between.
///
/// Answers `(left, right)` as virtual columns of the cursor's own screen row:
/// the cursor line is highlighted only over the row the cursor is on, so the
/// margins are the first and last virtual column of that row.
///
/// Memoised on `w_virtcol` and the two widths, because `win_line` asks once
/// per cell of the cursor line.
///
/// # Safety
/// `wp` must be a live window.
pub(crate) unsafe fn margin_columns_win(
    wp: *mut win_T,
) -> (::core::ffi::c_int, ::core::ffi::c_int) {
    static SAVED_W_VIRTCOL: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    static PREV_WP: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
    static PREV_WIDTH1: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    static PREV_WIDTH2: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    static PREV_LEFT_COL: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    static PREV_RIGHT_COL: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);

    // SAFETY: the caller's window.
    unsafe {
        let width1 = (*wp).w_view_width - win_col_off(wp);
        let width2 = width1 + win_col_off2(wp);
        if SAVED_W_VIRTCOL.get() == (*wp).w_virtcol
            && PREV_WP.get() == wp
            && PREV_WIDTH1.get() == width1
            && PREV_WIDTH2.get() == width2
        {
            return (PREV_LEFT_COL.get(), PREV_RIGHT_COL.get());
        }

        let (left_col, right_col) = if (*wp).w_virtcol >= width1 && width2 > 0 {
            let past = (*wp).w_virtcol - width1;
            (
                past / width2 * width2 + width1,
                width1 + (past / width2 + 1) * width2,
            )
        } else {
            (0, width1)
        };

        PREV_LEFT_COL.set(left_col);
        PREV_RIGHT_COL.set(right_col);
        PREV_WP.set(wp);
        PREV_WIDTH1.set(width1);
        PREV_WIDTH2.set(width2);
        SAVED_W_VIRTCOL.set((*wp).w_virtcol);
        (left_col, right_col)
    }
}

// ---------------------------------------------------------------------------
// The state itself
// ---------------------------------------------------------------------------

impl WinLineVars {
    /// Start a screen line at column zero: reset the write cursor and blank
    /// the whole line buffer, so anything not drawn below reads as a space
    /// with no attribute and no virtual column.
    ///
    /// # Safety
    /// `wp` must be live and the line buffers sized for its width.
    pub(crate) unsafe fn start_line(&mut self, wp: *mut win_T) {
        self.col = 0;
        self.off = 0;
        self.linebreak_armed = false;
        // SAFETY: `grid_alloc` keeps the line buffers at least `w_view_width`
        // wide, which is the invariant every writer here relies on.
        unsafe {
            for i in 0..(*wp).w_view_width {
                put_cell(i, schar_from_ascii(b' '), 0, -1);
            }
        }
    }

    /// The virtual column the *highlighting* is at, which is the buffer's own
    /// virtual column less whatever concealment swallowed.
    ///
    /// `'colorcolumn'` and `'cursorcolumn'` are buffer columns, so they have
    /// to be compared against this rather than against [`WinLineVars::vcol`],
    /// which counts the cells the line would have taken unconcealed.
    #[inline(always)]
    pub(crate) fn hl_vcol(&self) -> ::core::ffi::c_int {
        self.vcol - self.vcol_off_co
    }

    /// Undo the fake columns concealment added to force a wrap.
    ///
    /// Concealed text is drawn as nothing, but the character loop still has to
    /// reach the right edge for the line to wrap where the buffer says it
    /// does; it does that by counting columns that do not exist. This puts the
    /// counters back, and remembers how many there were: `old_boguscols` is
    /// read after the fact when the cursor position is worked out.
    pub(crate) fn fix_for_boguscols(&mut self) {
        self.extra_todo += self.vcol_off_co;
        self.vcol -= self.vcol_off_co;
        self.vcol_off_co = 0;
        self.col -= self.boguscols;
        self.old_boguscols = self.boguscols;
        self.boguscols = 0;
    }

    /// Advance [`WinLineVars::color_cols`] past every `'colorcolumn'` left of
    /// `vcol`, and drop it entirely once the list is exhausted.
    ///
    /// # Safety
    /// `color_cols` must be null or point into a negative-terminated array.
    #[inline]
    pub(crate) unsafe fn advance_color_col(&mut self, vcol: ::core::ffi::c_int) {
        // SAFETY: the caller's array, walked only while its entries are
        // non-negative — the terminator stops it.
        unsafe {
            if self.color_cols.is_null() {
                return;
            }
            while *self.color_cols >= 0 && vcol > *self.color_cols {
                self.color_cols = self.color_cols.add(1);
            }
            if *self.color_cols < 0 {
                self.color_cols = ::core::ptr::null_mut();
            }
        }
    }

    /// `attr` with `ColorColumn` combined in if the current
    /// [`WinLineVars::vcol`] is one, having first advanced the list to it.
    ///
    /// The padding drawn before a wrapped line's text (`'breakindent'`,
    /// `'showbreak'`) participates in `'colorcolumn'` exactly like buffer
    /// text does, so both go through here.
    ///
    /// # Safety
    /// `wp` must be live and `color_cols` null or negative-terminated.
    #[inline]
    pub(crate) unsafe fn color_col_attr(
        &mut self,
        wp: *mut win_T,
        attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int {
        // SAFETY: the caller's window and array.
        unsafe {
            self.advance_color_col(self.vcol);
            if !self.color_cols.is_null() && self.vcol == *self.color_cols {
                hl_combine_attr(win_hl_attr(wp, HLF_MC), attr)
            } else {
                attr
            }
        }
    }

    /// Apply `'cursorline'` to the whole line.
    ///
    /// A compromise upstream made in vim/vim#7383: `CursorLine` is
    /// low-priority when it sets no foreground (so syntax colours survive it)
    /// and high-priority when it does.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(crate) unsafe fn apply_cursorline_highlight(&mut self, wp: *mut win_T) {
        // SAFETY: the caller's window.
        unsafe {
            self.cursorline_attr = win_hl_attr(wp, HLF_CUL);
            let ae = syn_attr2entry(self.cursorline_attr);
            if ae.rgb_fg_color == -1 as RgbValue && ae.cterm_fg_color == 0 {
                self.line_attr_lowprio = self.cursorline_attr;
            } else if State.get() & MODE_INSERT == 0
                && bt_quickfix((*wp).w_buffer)
                && qf_current_entry(wp) == self.lnum
            {
                // A quickfix window's current-entry highlight keeps its own
                // colours; CursorLine goes underneath it.
                self.line_attr = hl_combine_attr(self.cursorline_attr, self.line_attr);
            } else {
                self.line_attr = self.cursorline_attr;
            }
        }
    }

    /// Overlay `'cursorline'` onto the diff-mode line highlight.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(crate) unsafe fn set_line_attr_for_diff(&mut self, wp: *mut win_T) {
        // SAFETY: the caller's window.
        unsafe {
            self.line_attr = win_hl_attr(wp, self.diff_hlf);
            if self.cursorline_attr != 0 {
                self.line_attr = if self.line_attr_lowprio != 0 {
                    hl_combine_attr(
                        hl_combine_attr(self.cursorline_attr, self.line_attr),
                        hl_get_underline(),
                    )
                } else {
                    hl_combine_attr(self.line_attr, self.cursorline_attr)
                };
            }
        }
    }
}
