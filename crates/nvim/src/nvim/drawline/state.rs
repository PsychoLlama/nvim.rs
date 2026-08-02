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
    pub cul_attr: ::core::ffi::c_int,
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
    pub vcol_sbr: colnr_T,
    /// This row still owes a `'showbreak'`.
    pub need_showbreak: bool,

    /// Attribute for the next character.
    pub char_attr: ::core::ffi::c_int,

    /// Bytes left in `p_extra`/`sc_extra`.
    pub n_extra: ::core::ffi::c_int,
    /// Characters left that take `extra_attr`.
    pub n_attr: ::core::ffi::c_int,
    /// Text to draw instead of buffer text; only used when `sc_extra` and
    /// `sc_final` are NUL.
    pub p_extra: *mut ::core::ffi::c_char,
    /// Attribute for `p_extra`.
    pub extra_attr: ::core::ffi::c_int,
    /// One character repeated `n_extra` times.
    pub sc_extra: schar_T,
    /// Mandatory last character of a repeated run, when set.
    pub sc_final: schar_T,

    /// `n_extra` came from inline virtual text.
    pub extra_for_extmark: bool,

    /// Scratch for one character's display form; must be as large as
    /// `transchar_charbuf` in charset.c.
    pub extra: [::core::ffi::c_char; 11],

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
    pub sattrs: [SignTextAttrs; 9],
    /// In `'linebreak'` mode, only consider wrapping after a non-blank.
    pub need_lbr: bool,

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
        self.need_lbr = false;
        // SAFETY: `grid_alloc` keeps the line buffers at least `w_view_width`
        // wide, which is the invariant every writer here relies on.
        unsafe {
            for i in 0..(*wp).w_view_width {
                put_cell(i, schar_from_ascii(b' '), 0, -1);
            }
        }
    }

    /// Undo the fake columns concealment added to force a wrap.
    ///
    /// Concealed text is drawn as nothing, but the character loop still has to
    /// reach the right edge for the line to wrap where the buffer says it
    /// does; it does that by counting columns that do not exist. This puts the
    /// counters back, and remembers how many there were: `old_boguscols` is
    /// read after the fact when the cursor position is worked out.
    pub(crate) fn fix_for_boguscols(&mut self) {
        self.n_extra += self.vcol_off_co;
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
            self.cul_attr = win_hl_attr(wp, HLF_CUL);
            let ae = syn_attr2entry(self.cul_attr);
            if ae.rgb_fg_color == -1 as RgbValue && ae.cterm_fg_color == 0 {
                self.line_attr_lowprio = self.cul_attr;
            } else if State.get() & MODE_INSERT == 0
                && bt_quickfix((*wp).w_buffer)
                && qf_current_entry(wp) == self.lnum
            {
                // A quickfix window's current-entry highlight keeps its own
                // colours; CursorLine goes underneath it.
                self.line_attr = hl_combine_attr(self.cul_attr, self.line_attr);
            } else {
                self.line_attr = self.cul_attr;
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
            if self.cul_attr != 0 {
                self.line_attr = if self.line_attr_lowprio != 0 {
                    hl_combine_attr(
                        hl_combine_attr(self.cul_attr, self.line_attr),
                        hl_get_underline(),
                    )
                } else {
                    hl_combine_attr(self.line_attr, self.cul_attr)
                };
            }
        }
    }
}
