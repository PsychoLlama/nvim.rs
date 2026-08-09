//! Everything drawn to the left of the text.
//!
//! In window order: the fold column ([`WinLineVars::draw_foldcolumn`],
//! [`fill_foldcolumn`]), the sign column ([`WinLineVars::draw_sign`]), the
//! number column ([`WinLineVars::draw_lnum_col`]) — or, when `'statuscolumn'`
//! is set, one expression replacing all three
//! ([`WinLineVars::draw_statuscol`]) — followed by the `'breakindent'` and
//! `'showbreak'` padding a wrapped line's continuation rows start with
//! ([`WinLineVars::handle_breakindent`],
//! [`WinLineVars::handle_showbreak_and_filler`]).
//!
//! [`WinLineVars::draw_col_buf`] and [`WinLineVars::draw_col_fill`] are the two
//! primitives all of them emit through: one copies a string into the line
//! buffer a character at a time, the other repeats one character. Both advance
//! [`WinLineVars::off`], so the order the callers run in *is* the column order
//! on screen.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::decoration::{SCL_NUM, SIGN_WIDTH};
use crate::src::nvim::statusline::{STL_FOLDCOL, STL_SIGNCOL};
use crate::src::nvim::types::VV_VIRTNUM;

/// The widest a `'statuscolumn'` may grow the number column to.
///
/// Upstream's `MAX_STCWIDTH`: a full-width line number, every sign the sign
/// column can show, and the widest fold column.
const MAX_STCWIDTH: ::core::ffi::c_int =
    MAX_NUMBERWIDTH + SIGN_SHOW_MAX * SIGN_WIDTH as ::core::ffi::c_int + 9;

/// The widest `'foldcolumn'` (and so the size of the arrays
/// [`fill_foldcolumn`] fills).
const MAX_FOLDCOLUMN: usize = 9;

// ---------------------------------------------------------------------------
// The two emit primitives
// ---------------------------------------------------------------------------

impl WinLineVars {
    /// Copy `len` bytes of `text` into the line buffer, one character per cell
    /// (a Tab expanding to its padding), stopping at the right edge.
    ///
    /// `inc_vcol` says the text counts as buffer virtual columns — true only
    /// for `'showbreak'`, which is the one thing in the left columns that
    /// participates in `'colorcolumn'`. Otherwise the cells take `fold_vcol`'s
    /// entries when it is non-null (the fold column's `-2`/`-3` markers, so a
    /// `'statuscolumn'` `%C` segment stays clickable) and `-1` when it is not.
    ///
    /// # Safety
    /// `text` must hold `len` readable bytes *and* be NUL-terminated at or
    /// after `len`: a multibyte character starting just before the limit is
    /// read whole. `fold_vcol` must be null or have an entry per cell drawn.
    pub(crate) unsafe fn draw_col_buf(
        &mut self,
        wp: *mut win_T,
        text: *const ::core::ffi::c_char,
        len: size_t,
        attr: ::core::ffi::c_int,
        mut fold_vcol: *const colnr_T,
        inc_vcol: bool,
    ) {
        // SAFETY: the caller's buffer, and `off` is kept under the view width
        // by the loop condition.
        unsafe {
            let end = text.add(len);
            let mut ptr = text;
            while ptr < end && self.off < (*wp).w_view_width {
                let cells = line_putchar(
                    (*wp).w_buffer,
                    &mut ptr,
                    linebuf_char.get().add(self.off as usize),
                    (*wp).w_view_width - self.off,
                    self.off,
                );
                let myattr = if inc_vcol {
                    self.color_col_attr(wp, attr)
                } else {
                    attr
                };
                for _ in 0..cells {
                    let vcol = if inc_vcol {
                        let at = self.vcol;
                        self.vcol += 1;
                        at
                    } else if !fold_vcol.is_null() {
                        let at = *fold_vcol;
                        fold_vcol = fold_vcol.add(1);
                        at
                    } else {
                        -1
                    };
                    *linebuf_attr.get().add(self.off as usize) = myattr as sattr_T;
                    *linebuf_vcol.get().add(self.off as usize) = vcol;
                    self.off += 1;
                }
            }
        }
    }

    /// Repeat one character for `width` cells.
    ///
    /// The virtual column of each cell is deliberately left alone:
    /// [`WinLineVars::start_line`] already set the whole line buffer to `-1`,
    /// and a filled column is never buffer text.
    ///
    /// # Safety
    /// `off + width` must be within the line buffer.
    #[inline]
    pub(crate) unsafe fn draw_col_fill(
        &mut self,
        fillchar: schar_T,
        width: ::core::ffi::c_int,
        attr: ::core::ffi::c_int,
    ) {
        // SAFETY: the caller's bound.
        unsafe {
            for _ in 0..width {
                *linebuf_char.get().add(self.off as usize) = fillchar;
                *linebuf_attr.get().add(self.off as usize) = attr as sattr_T;
                self.off += 1;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The fold column
// ---------------------------------------------------------------------------

/// Whether the `CursorLineSign`/`CursorLineFold` highlights apply to `lnum`.
///
/// # Safety
/// `wp` must be a live window.
pub unsafe fn use_cursor_line_highlight(wp: *mut win_T, lnum: linenr_T) -> bool {
    // SAFETY: the caller's window.
    unsafe {
        (*wp).w_onebuf_opt.wo_cul != 0
            && lnum == (*wp).w_cursorline
            && (*wp).w_p_culopt_flags as ::core::ffi::c_int
                & kOptCuloptFlagNumber as ::core::ffi::c_int
                != 0
    }
}

/// The glyph for a fold-column cell that is inside a fold but is neither the
/// fold's opening nor its closing marker.
///
/// `'fillchars'` `foldsep` when the column really is the outermost level
/// shown, `foldinner` when that is set, and otherwise a *digit*: with a
/// `'foldcolumn'` narrower than the nesting, the number says how deep the
/// column is, and `>` once that runs past nine.
///
/// # Safety
/// `wp` must be a live window.
#[inline]
unsafe fn foldcolumn_sep_char(
    first_level: ::core::ffi::c_int,
    i: ::core::ffi::c_int,
    wp: *mut win_T,
) -> schar_T {
    // SAFETY: the caller's window.
    unsafe {
        if first_level == 1 {
            (*wp).w_p_fcs_chars.foldsep
        } else if (*wp).w_p_fcs_chars.foldinner != NUL as schar_T {
            (*wp).w_p_fcs_chars.foldinner
        } else if first_level + i <= 9 {
            schar_from_ascii(b'0' + (first_level + i) as u8)
        } else {
            schar_from_ascii(b'>')
        }
    }
}

/// The `fdc` fold-column cells for `lnum`: each is a glyph and the pseudo
/// virtual column mouse handling reads back for it (`-1` outside every fold,
/// `-2` on the closed-fold marker, `-3` inside a fold).
///
/// `is_virt` marks a filler line — a diff filler or a virtual line. Those are
/// drawn *above* their buffer line, so a line that opens a fold must show the
/// fold column of the line before it, not its own opening marker.
///
/// # Safety
/// `wp` must be live, and `fdc` may not exceed [`MAX_FOLDCOLUMN`].
unsafe fn fold_column_cells(
    wp: *mut win_T,
    foldinfo: foldinfo_T,
    lnum: linenr_T,
    fdc: ::core::ffi::c_int,
    is_virt: bool,
) -> [(schar_T, colnr_T); MAX_FOLDCOLUMN] {
    // SAFETY: the caller's window.
    unsafe {
        let closed = foldinfo.fi_level != 0 && foldinfo.fi_lines > 0;
        let level = foldinfo.fi_level;
        // Too narrow for the nesting: start at the lowest level that fits and
        // use digits for the depth.
        let first_level = (level - fdc - closed as ::core::ffi::c_int + 1).max(1);
        let closedcol = fdc.min(level);
        // A filler line shows the fold column of the line above, so the
        // `foldopen`/`foldclose` markers are not drawn twice. That is the
        // same computation one level out.
        let outer = if is_virt && foldinfo.fi_level != 0 && foldinfo.fi_lnum == lnum {
            let outer_level = (foldinfo.fi_low_level - 1).max(0);
            Some((outer_level, (outer_level - fdc + 1).max(1)))
        } else {
            None
        };

        let mut cells = [(0 as schar_T, 0 as colnr_T); MAX_FOLDCOLUMN];
        for (i, cell) in cells.iter_mut().enumerate().take(fdc as usize) {
            let i = i as ::core::ffi::c_int;
            let mut symbol = if i >= level {
                schar_from_ascii(b' ')
            } else if i == closedcol - 1 && closed {
                (*wp).w_p_fcs_chars.foldclosed
            } else if foldinfo.fi_lnum == lnum && first_level + i >= foldinfo.fi_low_level {
                (*wp).w_p_fcs_chars.foldopen
            } else {
                foldcolumn_sep_char(first_level, i, wp)
            };
            if let Some((outer_level, outer_first_level)) = outer {
                symbol = if i >= outer_level {
                    schar_from_ascii(b' ')
                } else {
                    foldcolumn_sep_char(outer_first_level, i, wp)
                };
            }
            let vcol = if i >= level {
                -1
            } else if i == closedcol - 1 && closed {
                -2
            } else {
                -3
            };
            *cell = (symbol, vcol);
        }
        cells
    }
}

/// Fill a caller's arrays with the fold column, for `'statuscolumn'`'s `%C`.
///
/// # Safety
/// `wp` must be live, `fdc` may not exceed [`MAX_FOLDCOLUMN`], and both arrays
/// must have `fdc` entries.
pub unsafe fn fill_foldcolumn(
    wp: *mut win_T,
    foldinfo: foldinfo_T,
    lnum: linenr_T,
    fdc: ::core::ffi::c_int,
    is_virt: bool,
    out_vcol: *mut colnr_T,
    out_buffer: *mut schar_T,
) {
    // SAFETY: the caller's window and arrays.
    unsafe {
        let cells = fold_column_cells(wp, foldinfo, lnum, fdc, is_virt);
        for (i, &(symbol, vcol)) in cells.iter().enumerate().take(fdc as usize) {
            *out_vcol.add(i) = vcol;
            *out_buffer.add(i) = symbol;
        }
    }
}

impl WinLineVars {
    /// Draw the `'foldcolumn'`, if there is one.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(crate) unsafe fn draw_foldcolumn(&mut self, wp: *mut win_T) {
        // SAFETY: the caller's window.
        unsafe {
            let fdc = compute_foldcolumn(wp, 0);
            if fdc <= 0 {
                return;
            }
            let attr = win_hl_attr(
                wp,
                if use_cursor_line_highlight(wp, self.lnum) {
                    HLF_CLF
                } else {
                    HLF_FC
                },
            );
            let is_virt = self.filler_todo > 0;
            let cells = fold_column_cells(wp, self.foldinfo, self.lnum, fdc, is_virt);
            for &(symbol, vcol) in cells.iter().take(fdc as usize) {
                put_cell(self.off, symbol, attr, vcol);
                self.off += 1;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The sign column
// ---------------------------------------------------------------------------

impl WinLineVars {
    /// Draw sign `sign_idx`, or blank cells if there is no sign here.
    ///
    /// `nrcol` means the sign goes in the *number* column (`'signcolumn'` is
    /// `number`), which is wider and is only ever asked for when there really
    /// is a sign — [`WinLineVars::draw_lnum_col`] tests that first.
    ///
    /// # Safety
    /// `wp` must be live and `sign_idx` a valid index into
    /// [`WinLineVars::sattrs`].
    pub(crate) unsafe fn draw_sign(
        &mut self,
        nrcol: bool,
        wp: *mut win_T,
        sign_idx: ::core::ffi::c_int,
    ) {
        // SAFETY: the caller's window and index.
        unsafe {
            let sattr = self.sign_attrs[sign_idx as usize];
            let scl_attr = win_hl_attr(
                wp,
                if use_cursor_line_highlight(wp, self.lnum) {
                    HLF_CLS
                } else {
                    HLF_SC
                },
            );

            if sattr.text[0] == 0
                || self.row != self.startrow + self.filler_lines
                || self.filler_todo > 0
            {
                // No sign on this row. The number column never gets here:
                // `draw_lnum_col` only asks for `nrcol` when there is a sign.
                debug_assert!(!nrcol);
                self.draw_col_fill(
                    schar_from_ascii(b' '),
                    SIGN_WIDTH as ::core::ffi::c_int,
                    scl_attr,
                );
                return;
            }

            let fill = if nrcol {
                number_width(wp) + 1
            } else {
                SIGN_WIDTH as ::core::ffi::c_int
            };
            let attr = if self.sign_cul_attr != 0 {
                self.sign_cul_attr
            } else if sattr.hl_id != 0 {
                syn_id2attr(sattr.hl_id)
            } else {
                0
            };
            // Blank the whole column first, then overwrite the two cells the
            // sign text occupies: in the number column the sign is drawn at
            // the right, with the extra width padding it on the left.
            self.draw_col_fill(
                schar_from_ascii(b' '),
                fill,
                hl_combine_attr(scl_attr, attr),
            );
            let sign_pos =
                self.off - SIGN_WIDTH as ::core::ffi::c_int - nrcol as ::core::ffi::c_int;
            debug_assert!(sign_pos >= 0);
            *linebuf_char.get().add(sign_pos as usize) = sattr.text[0];
            *linebuf_char.get().add(sign_pos as usize + 1) = sattr.text[1];
        }
    }
}

// ---------------------------------------------------------------------------
// The number column
// ---------------------------------------------------------------------------

/// Render the line number into `buf`, right-aligned in `numberwidth` columns
/// with a trailing space.
///
/// With `'number'` and `'relativenumber'` both set, the cursor line shows its
/// absolute number *left*-aligned instead — that is what makes it stand out
/// from the relative numbers around it.
///
/// # Safety
/// `wp` must be a live window.
#[inline]
unsafe fn line_number_str(wp: *mut win_T, lnum: linenr_T, buf: &mut [::core::ffi::c_char; 32]) {
    // SAFETY: the caller's window; `snprintf` is bounded by the array size.
    unsafe {
        let (num, fmt) = if (*wp).w_onebuf_opt.wo_nu != 0 && (*wp).w_onebuf_opt.wo_rnu == 0 {
            (lnum, c"%*d ")
        } else {
            let rel = abs(get_cursor_rel_lnum(wp, lnum)) as linenr_T;
            if rel == 0 && (*wp).w_onebuf_opt.wo_nu != 0 && (*wp).w_onebuf_opt.wo_rnu != 0 {
                (lnum, c"%-*d ")
            } else {
                (rel, c"%*d ")
            }
        };
        snprintf(
            buf.as_mut_ptr(),
            buf.len() as size_t,
            fmt.as_ptr(),
            number_width(wp),
            num,
        );
    }
}

impl WinLineVars {
    /// Whether `CursorLineNr` applies to this row's number column.
    ///
    /// It does on the first screen row of the cursor line, and on the
    /// continuation rows only when `'cursorlineopt'` also contains "line" —
    /// otherwise the highlight follows the number itself, not the column.
    ///
    /// # Safety
    /// `wp` must be a live window.
    unsafe fn use_cursor_line_nr(&self, wp: *mut win_T) -> bool {
        // SAFETY: the caller's window.
        unsafe {
            let culopt = (*wp).w_p_culopt_flags as ::core::ffi::c_int;
            (*wp).w_onebuf_opt.wo_cul != 0
                && self.lnum == (*wp).w_cursorline
                && culopt & kOptCuloptFlagNumber as ::core::ffi::c_int != 0
                && (self.row == self.startrow + self.filler_lines
                    || (self.row > self.startrow + self.filler_lines
                        && culopt & kOptCuloptFlagLine as ::core::ffi::c_int != 0))
        }
    }

    /// The number-column attribute: the right `LineNr*` highlight with the
    /// highest-priority sign `numhl` combined in.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(crate) unsafe fn line_number_attr(&mut self, wp: *mut win_T) -> ::core::ffi::c_int {
        // SAFETY: the caller's window.
        unsafe {
            let mut numhl_attr = self.sign_num_attr;
            if self.n_virt_lines - self.filler_todo < self.n_virt_below {
                // A virtual line belonging to the line above takes *its* sign
                // numhl. Looked up once and cached: this runs per row.
                if self.prev_num_attr == -1 {
                    decor_redraw_signs(
                        wp,
                        (*wp).w_buffer,
                        self.lnum - 2,
                        ::core::ptr::null_mut(),
                        ::core::ptr::null_mut(),
                        ::core::ptr::null_mut(),
                        &raw mut self.prev_num_attr,
                    );
                    if self.prev_num_attr > 0 {
                        self.prev_num_attr = syn_id2attr(self.prev_num_attr);
                    }
                }
                numhl_attr = self.prev_num_attr;
            }

            let hlf = if self.use_cursor_line_nr(wp) {
                // TODO(vim): can CursorLine stand in when CursorLineNr is unset?
                HLF_CLN
            } else if (*wp).w_onebuf_opt.wo_rnu != 0 && self.lnum < (*wp).w_cursor.lnum {
                HLF_LNA
            } else if (*wp).w_onebuf_opt.wo_rnu != 0 && self.lnum > (*wp).w_cursor.lnum {
                HLF_LNB
            } else {
                HLF_N
            };
            hl_combine_attr(win_hl_attr(wp, hlf), numhl_attr)
        }
    }

    /// Draw the number column: the absolute or relative line number on the
    /// first row of a buffer line, blanks on its continuation rows unless
    /// `'cpoptions'` contains "n".
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(crate) unsafe fn draw_lnum_col(&mut self, wp: *mut win_T) {
        // SAFETY: the caller's window.
        unsafe {
            let has_cpo_n = !vim_strchr(p_cpo.get(), CPO_NUMCOL).is_null();
            if (*wp).w_onebuf_opt.wo_nu == 0 && (*wp).w_onebuf_opt.wo_rnu == 0 {
                return;
            }
            if self.row != self.startrow + self.filler_lines && has_cpo_n {
                return;
            }
            // With "n" in 'cpoptions' there is no number column on a wrapped
            // line — but 'breakindent' assumes there is one anyway.
            if has_cpo_n
                && (*wp).w_onebuf_opt.wo_bri == 0
                && (*wp).w_skipcol > 0
                && self.lnum == (*wp).w_topline
            {
                return;
            }

            let first_row = self.row == self.startrow + self.filler_lines;
            // 'signcolumn'=number: a sign on this line replaces the number.
            if (*wp).w_minscwidth == SCL_NUM
                && self.sign_attrs[0].text[0] != 0
                && first_row
                && self.filler_todo <= 0
            {
                self.draw_sign(true, wp, 0);
                return;
            }

            let width = number_width(wp) + 1;
            let attr = self.line_number_attr(wp);
            let both = (*wp).w_onebuf_opt.wo_nu != 0 && (*wp).w_onebuf_opt.wo_rnu != 0;
            if !(first_row && ((*wp).w_skipcol == 0 || self.row > 0 || both)) {
                // A continuation row, or the first row of a line whose top is
                // scrolled off with 'smoothscroll': blank.
                self.draw_col_fill(schar_from_ascii(b' '), width, attr);
                return;
            }

            let mut buf: [::core::ffi::c_char; 32] = [0; 32];
            line_number_str(wp, self.lnum, &mut buf);
            if (*wp).w_skipcol > 0 && self.startrow == 0 {
                // Part of this line is scrolled off above: say so by filling
                // the number's padding with dashes.
                for c in buf.iter_mut() {
                    if *c != b' ' as ::core::ffi::c_char {
                        break;
                    }
                    *c = b'-' as ::core::ffi::c_char;
                }
            }
            if (*wp).w_onebuf_opt.wo_rl != 0 {
                let num = skipwhite(buf.as_mut_ptr());
                rl_mirror_ascii(num, skiptowhite(num));
            }
            self.draw_col_buf(
                wp,
                buf.as_ptr(),
                width as size_t,
                attr,
                ::core::ptr::null(),
                false,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 'statuscolumn'
// ---------------------------------------------------------------------------

impl WinLineVars {
    /// Build and draw the `'statuscolumn'` string for this row.
    ///
    /// `virtnum` is `v:virtnum`: 0 on a buffer line's first row, positive on
    /// its wrapped continuation rows and negative on a filler line above it.
    ///
    /// The expression can come out wider than the column it was sized for, in
    /// which case this widens `w_nrwidth` and asks for another redraw rather
    /// than truncating; an expression that *errors* resets `'statuscolumn'`
    /// altogether, and the number column goes back to `number_width`.
    ///
    /// # Safety
    /// `wp` must be live and `stcp` must point at the caller's `statuscol_T`,
    /// which `build_statuscol_str` fills in.
    pub(crate) unsafe fn draw_statuscol(
        &mut self,
        wp: *mut win_T,
        virtnum: ::core::ffi::c_int,
        col_rows: ::core::ffi::c_int,
        stcp: *mut statuscol_T,
    ) {
        // SAFETY: the caller's window and status-column state.
        unsafe {
            // Filler lines belonging to the line above report that line's
            // number; `v:relnum` is only set on the first row of each of the
            // three groups (filler, buffer line, virtual lines below).
            let lnum =
                self.lnum - (self.n_virt_lines - self.filler_todo < self.n_virt_below) as linenr_T;
            let relnum = if virtnum == -self.filler_lines
                || virtnum == 0
                || virtnum == self.n_virt_below - self.filler_lines
            {
                abs(get_cursor_rel_lnum(wp, lnum)) as linenr_T
            } else {
                -1
            };

            let mut buf: [::core::ffi::c_char; MAXPATHL as usize] = [0; MAXPATHL as usize];
            if (*wp).w_statuscol_line_count != (*wp).w_nrwidth_line_count {
                // The line count changed. Estimate the full width by building
                // with the largest line number there can be, and widen before
                // anything is drawn.
                (*wp).w_statuscol_line_count = (*wp).w_nrwidth_line_count;
                set_vim_var_nr(VV_VIRTNUM, 0);
                let width = build_statuscol_str(
                    wp,
                    (*wp).w_nrwidth_line_count,
                    (*wp).w_nrwidth_line_count,
                    buf.as_mut_ptr(),
                    stcp,
                );
                if width > (*stcp).width {
                    let addwidth = (width - (*stcp).width).min(MAX_STCWIDTH - (*stcp).width);
                    (*wp).w_nrwidth += addwidth;
                    (*wp).w_nrwidth_width = (*wp).w_nrwidth;
                    if col_rows > 0 {
                        // Only the columns were being redrawn; the text has to
                        // move too now.
                        (*wp).w_redr_statuscol = true;
                        return;
                    }
                    (*stcp).width += addwidth;
                    (*wp).w_valid &= !VALID_WCOL;
                }
            }

            set_vim_var_nr(VV_VIRTNUM, virtnum as varnumber_T);
            let width = build_statuscol_str(wp, lnum, relnum, buf.as_mut_ptr(), stcp);
            let was_reset = *(*wp).w_onebuf_opt.wo_stc == NUL as ::core::ffi::c_char;
            if was_reset || (width > (*stcp).width && (*stcp).width < MAX_STCWIDTH) {
                if was_reset {
                    // 'statuscolumn' was reset because the expression failed.
                    (*wp).w_nrwidth_line_count = 0;
                    (*wp).w_nrwidth = ((*wp).w_onebuf_opt.wo_nu != 0
                        || (*wp).w_onebuf_opt.wo_rnu != 0)
                        as ::core::ffi::c_int
                        * number_width(wp);
                } else {
                    (*wp).w_nrwidth += (width - (*stcp).width).min(MAX_STCWIDTH - (*stcp).width);
                    (*wp).w_nrwidth_width = (*wp).w_nrwidth;
                }
                (*wp).w_redr_statuscol = true;
                return;
            }

            // Draw each segment with the highlight `build_statuscol_str`
            // recorded for it. A `hlrec` entry marks where a new highlight
            // *starts*, so each pass draws the text up to it and then works
            // out the attribute the next stretch takes.
            let scl_attr = win_hl_attr(
                wp,
                if use_cursor_line_highlight(wp, self.lnum) {
                    HLF_CLS
                } else {
                    HLF_SC
                },
            );
            let num_attr = self.line_number_attr(wp);
            let mut cur_attr = num_attr;
            let mut fold_vcol: *const colnr_T = ::core::ptr::null();
            let mut transbuf: [::core::ffi::c_char; MAXPATHL as usize] = [0; MAXPATHL as usize];
            let mut p = buf.as_ptr();
            let mut sp = (*stcp).hlrec;
            while !(*sp).start.is_null() {
                let textlen = (*sp).start.offset_from(p);
                let translen = transstr_buf(
                    p,
                    textlen as ssize_t,
                    transbuf.as_mut_ptr(),
                    MAXPATHL as size_t,
                    true,
                );
                self.draw_col_buf(wp, transbuf.as_ptr(), translen, cur_attr, fold_vcol, false);
                // The sign segment takes the sign column's highlight and the
                // number segments the number column's; the fold segment takes
                // none, because `fill_foldcolumn` gave it its own through the
                // user highlight below.
                let base = if (*sp).item == STL_SIGNCOL {
                    scl_attr
                } else if (*sp).item == STL_FOLDCOL {
                    0
                } else {
                    num_attr
                };
                cur_attr = hl_combine_attr(
                    base,
                    if (*sp).userhl < 0 {
                        syn_id2attr(-(*sp).userhl)
                    } else {
                        0
                    },
                );
                fold_vcol = if (*sp).item == STL_FOLDCOL {
                    (&raw const (*stcp).fold_vcol).cast::<colnr_T>()
                } else {
                    ::core::ptr::null()
                };
                p = (*sp).start;
                sp = sp.add(1);
            }
            let len = strlen(buf.as_ptr());
            let translen = transstr_buf(
                p,
                buf.as_ptr().add(len as usize).offset_from(p) as ssize_t,
                transbuf.as_mut_ptr(),
                MAXPATHL as size_t,
                true,
            );
            self.draw_col_buf(wp, transbuf.as_ptr(), translen, cur_attr, fold_vcol, false);
            self.draw_col_fill(schar_from_ascii(b' '), (*stcp).width - width, cur_attr);
        }
    }
}

// ---------------------------------------------------------------------------
// 'breakindent' and 'showbreak'
// ---------------------------------------------------------------------------

impl WinLineVars {
    /// Indent a wrapped line's continuation row to match its first row.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(crate) unsafe fn handle_breakindent(&mut self, wp: *mut win_T) {
        // SAFETY: the caller's window.
        unsafe {
            if (*wp).w_onebuf_opt.wo_bri != 0
                && (self.row > self.startrow + self.filler_lines || self.need_showbreak)
            {
                let attr = if self.diff_hlf != HLF_NONE {
                    win_hl_attr(wp, self.diff_hlf)
                } else {
                    0
                };
                let mut num = get_breakindent_win(wp, ml_get_buf((*wp).w_buffer, self.lnum));
                if self.row == self.startrow {
                    // The first row of a line whose top is scrolled off: the
                    // indent is measured from the second row's left edge.
                    num -= win_col_off2(wp);
                    if self.extra_todo < 0 {
                        num = 0;
                    }
                }

                let vcol_before = self.vcol;
                for _ in 0..num {
                    // These really are vcols: the indent counts as part of the
                    // line for 'colorcolumn' and for the highlighted area.
                    let myattr = self.color_col_attr(wp, attr);
                    put_cell(self.off, schar_from_ascii(b' '), myattr, self.vcol);
                    self.vcol += 1;
                    self.off += 1;
                }

                // Move the start and end of the highlighted area past the
                // indent. The end needs it when 'linebreak' is also set.
                if self.fromcol >= vcol_before && self.fromcol < self.vcol {
                    self.fromcol = self.vcol;
                }
                if self.tocol == vcol_before {
                    self.tocol = self.vcol;
                }
            }

            if (*wp).w_skipcol > 0
                && self.startrow == 0
                && (*wp).w_onebuf_opt.wo_wrap != 0
                && (*wp).w_briopt_sbr
            {
                self.need_showbreak = false;
            }
        }
    }

    /// Fill a filler line, and draw `'showbreak'` at the start of a wrapped
    /// line's continuation row.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(crate) unsafe fn handle_showbreak_and_filler(&mut self, wp: *mut win_T) {
        // SAFETY: the caller's window.
        unsafe {
            let remaining = (*wp).w_view_width - self.off;
            if self.filler_todo > self.filler_lines - self.n_virt_lines {
                // A virtual line: its text is drawn by the decoration code, so
                // all this owes is the background.
                // TODO(bfredl): check this doesn't inhibit TUI-style
                //               clear-to-end-of-line.
                self.draw_col_fill(schar_from_ascii(b' '), remaining, 0);
            } else if self.filler_todo > 0 {
                // A "deleted" diff line.
                self.draw_col_fill(
                    (*wp).w_p_fcs_chars.diff,
                    remaining,
                    win_hl_attr(wp, HLF_DED),
                );
            }

            let sbr = get_showbreak_value(wp);
            if *sbr != NUL as ::core::ffi::c_char && self.need_showbreak {
                // 'showbreak' combined with 'cursorline', 'showbreak' winning.
                let attr = hl_combine_attr(self.cursorline_attr, win_hl_attr(wp, HLF_AT));
                let vcol_before = self.vcol;
                self.draw_col_buf(wp, sbr, strlen(sbr), attr, ::core::ptr::null(), true);
                self.showbreak_vcol = self.vcol;

                // As in `handle_breakindent`: move the highlighted area past
                // what was just drawn.
                if self.fromcol >= vcol_before && self.fromcol < self.vcol {
                    self.fromcol = self.vcol;
                }
                if self.tocol == vcol_before {
                    self.tocol = self.vcol;
                }
            }

            if (*wp).w_skipcol == 0
                || self.startrow > 0
                || (*wp).w_onebuf_opt.wo_wrap == 0
                || !(*wp).w_briopt_sbr
            {
                self.need_showbreak = false;
            }
        }
    }
}
