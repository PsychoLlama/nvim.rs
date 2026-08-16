#![deny(unsafe_op_in_unsafe_fn)]

//! How much room text takes up in a window.
//!
//! Two halves, in the order the original `plines.c` has them:
//!
//! * **Horizontal** — how many screen cells a character or a line occupies.
//!   `init_charsize_arg` looks at the line once and answers whether the
//!   cheap measure ([`charsize_fast`]) is enough or the line needs the full
//!   one ([`charsize_regular`], which handles inline virtual text,
//!   'linebreak', 'breakindent' and 'showbreak'). Everything else here walks
//!   a line with one of those two.
//! * **Vertical** — how many window lines a buffer line occupies, built on
//!   the horizontal half plus folds, diff filler and virtual lines.
//!
//! This runs per character on every redraw. Prefer plain loops and keep the
//! small helpers inlined.

use crate::src::nvim::buffer::buf_meta_total;
use crate::src::nvim::charset::vim_isbreak;
use crate::src::nvim::charset::{ptr2cells, vim_isprintc, vim_strsize};
use crate::src::nvim::decoration::{decor_conceal_line, decor_virt_lines, ns_in_win};
use crate::src::nvim::diff::{diff_check_fill, diffopt_filler};
use crate::src::nvim::fold::{hasFolding, hasFoldingWin, lineFolded};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::indent::{get_breakindent_win, tabstop_padding};
use crate::src::nvim::main::{State, VIsual, VIsual_active, curwin, p_sel};
use crate::src::nvim::marktree::key::{kMTFilterSelect, mt_decor, mt_invalid, mt_right};
use crate::src::nvim::marktree::{
    marktree_itr_current, marktree_itr_get_filter, marktree_itr_next_filter,
};
use crate::src::nvim::mbyte::{utf_ptr2StrCharInfo, utf_ptr2char, utfc_next, utfc_ptr2len};
use crate::src::nvim::memline::{ml_get_buf, ml_get_buf_len};
use crate::src::nvim::r#move::{win_col_off, win_col_off2};
use crate::src::nvim::option::get_showbreak_value;
use crate::src::nvim::pos::{MAXCOL, lt, ltoreq};
use crate::src::nvim::state::{MODE_NORMAL, virtual_active};
use crate::src::nvim::types::{
    CharSize, CharsizeArg, CharsizeKind, MarkTreeIter, MetaFilter, MetaIndex, StrCharInfo,
    VirtLines, buf_T, colnr_T, foldinfo_T, int32_t, int64_t, linenr_T, pos_T, uint32_t, win_T,
};
use crate::src::nvim::winlayer::Win;

use ::core::ffi::{c_char, c_int, c_long};

/// Cells a byte that is not part of a valid UTF-8 sequence occupies: it is
/// shown as `<xx>`.
const INVALID_BYTE_CELLS: c_int = 4;
/// `kVTIsLines` — the virtual text is a block of whole lines, not inline.
const VT_IS_LINES: c_int = 1;
/// `kVPosInline` — the virtual text sits between two characters of the line.
const VPOS_INLINE: uint32_t = 2;
/// `kMTMetaLines` — the marktree's per-node count of virtual-line marks.
const MT_META_LINES: MetaIndex = 1;
const TAB: int32_t = b'\t' as int32_t;
const NUL: c_char = 0;

/// The marktree filter that selects inline virtual text and nothing else.
static INLINE_FILTER: GlobalCell<[uint32_t; 5]> = GlobalCell::new([kMTFilterSelect, 0, 0, 0, 0]);

fn inline_filter() -> MetaFilter {
    INLINE_FILTER.ptr().cast::<uint32_t>()
}

// ---------------------------------------------------------------------------
// Horizontal size
// ---------------------------------------------------------------------------

/// Cells the first character of `p` takes on the screen, given that it starts
/// at virtual column `col` (which only matters for a tab).
///
/// # Safety
/// `wp` must be live and `p` must point into a NUL-terminated line.
pub unsafe fn win_chartabsize(wp: *mut win_T, p: *mut c_char, col: colnr_T) -> c_int {
    unsafe {
        let buf: *mut buf_T = (*wp).w_buffer;
        if *p as int32_t == TAB
            && ((*wp).w_onebuf_opt.wo_list == 0 || (*wp).w_p_lcs_chars.tab1 != 0)
        {
            return tabstop_padding(col, (*buf).b_p_ts, (*buf).b_p_vts_array);
        }
        ptr2cells(p)
    }
}

/// Cells the string `s` takes, as if it began at virtual column `startvcol`
/// of the current window.
///
/// # Safety
/// `s` must be a NUL-terminated string.
pub unsafe fn linetabsize_col(startvcol: c_int, s: *mut c_char) -> c_int {
    unsafe {
        let mut csarg = CharsizeArg::default();
        match init_charsize_arg(&mut csarg, curwin.get(), 0, s) {
            CharsizeKind::Fast => linesize_fast(&csarg, startvcol, MAXCOL),
            CharsizeKind::Regular => linesize_regular(&mut csarg, startvcol, MAXCOL),
        }
    }
}

/// The screen width of a whole line, starting from virtual column zero.
///
/// # Safety
/// `s` must be a NUL-terminated string.
pub unsafe fn linetabsize_str(s: *mut c_char) -> c_int {
    unsafe { linetabsize_col(0, s) }
}

/// Cells the first `len` bytes of `line` take in `wp`, counting inline
/// virtual text. Pass `MAXCOL` for the whole line.
///
/// # Safety
/// `wp` must be live; `line` must be line `lnum` of its buffer, or any
/// NUL-terminated string when `lnum` is 0 (which skips virtual text).
#[inline(always)]
pub unsafe fn win_linetabsize(
    wp: *mut win_T,
    lnum: linenr_T,
    line: *mut c_char,
    len: colnr_T,
) -> c_int {
    unsafe {
        let mut csarg = CharsizeArg::default();
        match init_charsize_arg(&mut csarg, wp, lnum, line) {
            CharsizeKind::Fast => linesize_fast(&csarg, 0, len),
            CharsizeKind::Regular => linesize_regular(&mut csarg, 0, len),
        }
    }
}

/// Cells line `lnum` takes in `wp`, counting inline virtual text but not the
/// 'listchars' "eol".
///
/// # Safety
/// `wp` must be live and `lnum` must be a line of its buffer.
pub unsafe fn linetabsize(wp: *mut win_T, lnum: linenr_T) -> c_int {
    unsafe { win_linetabsize(wp, lnum, ml_get_buf((*wp).w_buffer, lnum), MAXCOL) }
}

/// Like [`linetabsize`], but counts the 'listchars' "eol".
///
/// # Safety
/// `wp` must be live and `lnum` must be a line of its buffer.
pub unsafe fn linetabsize_eol(wp: *mut win_T, lnum: linenr_T) -> c_int {
    unsafe {
        let eol = (*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.eol != 0;
        linetabsize(wp, lnum) + c_int::from(eol)
    }
}

/// Prepare `csarg` for a walk over `line`, and answer which charsize function
/// that walk has to use.
///
/// `lnum` of 0 means "do not consider inline virtual text", which is how the
/// callers that measure a bare string rather than a buffer line ask for it.
///
/// # Safety
/// `wp` must be live; `line` must be NUL-terminated, and must be line `lnum`
/// of `wp`'s buffer when `lnum` is not 0.
pub unsafe fn init_charsize_arg(
    csarg: &mut CharsizeArg,
    wp: *mut win_T,
    lnum: linenr_T,
    line: *mut c_char,
) -> CharsizeKind {
    unsafe {
        csarg.win = wp;
        csarg.line = line;
        csarg.max_head_vcol = 0;
        csarg.cur_text_width_left = 0;
        csarg.cur_text_width_right = 0;
        csarg.virt_row = -1;
        csarg.indent_width = c_int::MIN;
        csarg.use_tabstop = (*wp).w_onebuf_opt.wo_list == 0 || (*wp).w_p_lcs_chars.tab1 != 0;

        if lnum > 0
            && marktree_itr_get_filter(
                &mut (*(*wp).w_buffer).b_marktree[0],
                lnum - 1,
                0,
                lnum,
                0,
                inline_filter(),
                &mut csarg.iter[0],
            )
        {
            csarg.virt_row = lnum - 1;
        }

        let needs_regular = csarg.virt_row >= 0
            || ((*wp).w_onebuf_opt.wo_wrap != 0
                && ((*wp).w_onebuf_opt.wo_lbr != 0
                    || (*wp).w_onebuf_opt.wo_bri != 0
                    || *get_showbreak_value(wp) != NUL));
        if needs_regular {
            CharsizeKind::Regular
        } else {
            CharsizeKind::Fast
        }
    }
}

/// Virtual columns the inline virtual text around the cursor shifts it by.
///
/// `csarg` must hold the widths [`charsize_regular`] left in it.
fn virt_text_cursor_off(csarg: &CharsizeArg, on_nul: bool) -> c_int {
    let mut off = 0;
    if !on_nul || State.get() & MODE_NORMAL == 0 {
        off += csarg.cur_text_width_left;
    }
    if !on_nul && State.get() & MODE_NORMAL != 0 {
        off += csarg.cur_text_width_right;
    }
    off
}

/// Extra cells 'showbreak' and 'breakindent' contribute around one character.
struct BreakHead {
    /// Added to the character's width.
    added: c_int,
    /// Of `added`, the part that precedes the character.
    head: c_int,
}

/// Width of 'showbreak' plus 'breakindent', computed once per line and
/// memoised in `csarg`.
///
/// # Safety
/// `sbr` must be NUL-terminated and `csarg` must be initialised.
unsafe fn wrapped_indent_width(csarg: &mut CharsizeArg, sbr: *mut c_char) -> c_int {
    unsafe {
        if csarg.indent_width == c_int::MIN {
            let mut width = 0;
            if *sbr != NUL {
                width += vim_strsize(sbr);
            }
            if (*csarg.win).w_onebuf_opt.wo_bri != 0 {
                width += get_breakindent_win(csarg.win, csarg.line);
            }
            csarg.indent_width = width;
        }
        csarg.indent_width
    }
}

/// The 'showbreak'/'breakindent' half of [`charsize_regular`].
///
/// A wrapped screen line starts with 'showbreak' and/or a 'breakindent',
/// which cost cells that belong to no character. They are charged to the
/// character that crosses onto the new screen line: `added` widens it, and
/// `head` says how much of that sits before it.
///
/// `csarg.max_head_vcol` selects who is asking. Zero means "count all of it";
/// a positive value means "only what falls before that virtual column";
/// negative means "only what falls before where the cursor goes", which is
/// the one case that has to know about inline virtual text.
///
/// # Safety
/// All pointers must be live and `csarg` initialised for the line `cur`
/// points into.
unsafe fn showbreak_head(
    csarg: &mut CharsizeArg,
    cur: *mut c_char,
    vcol: colnr_T,
    size: c_int,
    mb_added: c_int,
    sbr: *mut c_char,
) -> BreakHead {
    unsafe {
        let wp = csarg.win;
        let view_width = (*wp).w_view_width;
        let mut col_off_prev = win_col_off(wp);
        let width2 = view_width - col_off_prev + win_col_off2(wp);
        let mut wcol = vcol + col_off_prev;
        let max_head_vcol = csarg.max_head_vcol;
        let mut added = 0;
        let mut head = 0;

        // Cells taken by 'showbreak'/'breakindent' before the current char.
        let mut head_prev = 0;
        if wcol >= view_width {
            wcol -= view_width;
            col_off_prev = view_width - width2;
            if wcol >= width2 && width2 > 0 {
                wcol %= width2;
            }
            head_prev = wrapped_indent_width(csarg, sbr);
            if wcol < head_prev {
                head_prev -= wcol;
                wcol += head_prev;
                added += head_prev;
                if max_head_vcol <= 0 || vcol < max_head_vcol {
                    head += head_prev;
                }
            } else {
                head_prev = 0;
            }
            wcol += col_off_prev;
        }

        if wcol + size > view_width {
            // Cells taken by 'showbreak'/'breakindent' partway through it.
            let head_mid = wrapped_indent_width(csarg, sbr);
            if head_mid > 0 {
                // Effective width of the screen lines it spans.
                let prev_rem = view_width - wcol;
                let mut width = width2 - head_mid;
                if width <= 0 {
                    width = 1;
                }
                // Divide "size - prev_rem" by "width", rounding up.
                let cnt = (size - prev_rem + width - 1) / width;
                added += cnt * head_mid;

                if max_head_vcol == 0 || vcol + size + added < max_head_vcol {
                    head += cnt * head_mid;
                } else if width2 > 0 && max_head_vcol > vcol + head_prev + prev_rem {
                    head += (max_head_vcol - (vcol + head_prev + prev_rem) + width2 - 1) / width2
                        * head_mid;
                } else if max_head_vcol < 0 {
                    let off = mb_added + virt_text_cursor_off(csarg, *cur == NUL);
                    if off >= prev_rem {
                        head += if size > off {
                            (1 + (off - prev_rem) / width) * head_mid
                        } else {
                            (off - prev_rem + width - 1) / width * head_mid
                        };
                    }
                }
            }
        }

        BreakHead { added, head }
    }
}

/// Widen the character at `cur` by the inline virtual text attached at its
/// byte position, advancing `csarg`'s marktree iterator past that position.
///
/// A tab is re-measured after each chunk: inserting text moves the tab's end
/// to a different tabstop.
///
/// # Safety
/// `csarg` must be initialised with `virt_row >= 0` and `cur` must point into
/// its line.
unsafe fn add_inline_virt_text(
    csarg: &mut CharsizeArg,
    cur: *mut c_char,
    vcol: colnr_T,
    mut size: c_int,
    expand_tab: bool,
) -> c_int {
    unsafe {
        let wp = csarg.win;
        let buf: *mut buf_T = (*wp).w_buffer;
        let mut tab_size = size;
        let col = cur.offset_from(csarg.line) as int32_t;
        let iter = (&raw mut csarg.iter).cast::<MarkTreeIter>();

        loop {
            let mark = marktree_itr_current(&mut *iter);
            if mark.pos.row != csarg.virt_row || mark.pos.col > col {
                break;
            }
            if mark.pos.col == col && !mt_invalid(mark) && ns_in_win(mark.ns, Win::new(wp)) {
                let decor = mt_decor(mark);
                let mut vt = if decor.ext {
                    decor.data.ext.vt
                } else {
                    ::core::ptr::null_mut()
                };
                while !vt.is_null() {
                    if (*vt).flags as c_int & VT_IS_LINES == 0
                        && (*vt).pos as uint32_t == VPOS_INLINE
                    {
                        if mt_right(mark) {
                            csarg.cur_text_width_right += (*vt).width;
                        } else {
                            csarg.cur_text_width_left += (*vt).width;
                        }
                        size += (*vt).width;
                        if expand_tab {
                            // The tab's width changes with the inserted text.
                            size -= tab_size;
                            tab_size =
                                tabstop_padding(vcol + size, (*buf).b_p_ts, (*buf).b_p_vts_array);
                            size += tab_size;
                        }
                    }
                    vt = (*vt).next;
                }
            }
            marktree_itr_next_filter(
                &mut (*(*wp).w_buffer).b_marktree[0],
                &mut *iter,
                csarg.virt_row + 1,
                0,
                inline_filter(),
            );
        }
        size
    }
}

/// Whether the character at `cur` is where 'linebreak' would break the line:
/// a blank followed by a non-blank, outside the leading whitespace.
///
/// # Safety
/// `cur` must point into `csarg`'s NUL-terminated line.
unsafe fn breaks_here(csarg: &CharsizeArg, cur: *mut c_char) -> bool {
    unsafe {
        let wp = csarg.win;
        if (*wp).w_onebuf_opt.wo_lbr == 0
            || (*wp).w_onebuf_opt.wo_wrap == 0
            || (*wp).w_view_width == 0
            || !vim_isbreak(*cur as u8 as c_int)
            || vim_isbreak(*cur.offset(1) as u8 as c_int)
        {
            return false;
        }
        // 'linebreak' is only needed when not in leading whitespace.
        let mut t = csarg.line;
        while vim_isbreak(*t as u8 as c_int) {
            t = t.offset(1);
        }
        cur >= t
    }
}

/// The 'linebreak' half of [`charsize_regular`]: the blank at `cur` is
/// stretched so that the following word starts on the next screen line.
///
/// # Safety
/// `wp` must be live and `cur` must point into a NUL-terminated line.
unsafe fn linebreak_size(wp: *mut win_T, cur: *mut c_char, vcol: colnr_T, size: c_int) -> c_int {
    unsafe {
        // Count all characters from the first non-blank after a blank up to
        // the next non-blank after a blank.
        let numberextra = win_col_off(wp);
        let col_adj = size - 1;
        let mut colmax = (*wp).w_view_width - numberextra - col_adj;
        if vcol >= colmax {
            colmax += col_adj;
            let n = colmax + win_col_off2(wp);
            if n > 0 {
                colmax += ((vcol - colmax) / n + 1) * n - col_adj;
            }
        }

        let mut s = cur;
        let mut vcol2 = vcol;
        loop {
            let ps = s;
            s = s.offset(utfc_ptr2len(s) as isize);
            let c = *s as u8 as c_int;
            if c == NUL as c_int
                || !(vim_isbreak(c) || vcol2 == vcol || !vim_isbreak(*ps as u8 as c_int))
            {
                return size;
            }
            vcol2 += win_chartabsize(wp, s, vcol2);
            if vcol2 >= colmax {
                // Doesn't fit.
                return colmax - vcol + col_adj;
            }
        }
    }
}

/// Cells the character at `cur` takes, with everything accounted for.
///
/// Sets `csarg.cur_text_width_left`/`_right` to the inline virtual text
/// widths on either side of it. See [`showbreak_head`] for what
/// `csarg.max_head_vcol` selects.
///
/// # Safety
/// `csarg` must be initialised for the line `cur` points into, and `cur_char`
/// must be the codepoint `cur` decodes to (negative for an invalid byte).
pub unsafe fn charsize_regular(
    csarg: &mut CharsizeArg,
    cur: *mut c_char,
    vcol: colnr_T,
    cur_char: int32_t,
) -> CharSize {
    unsafe {
        csarg.cur_text_width_left = 0;
        csarg.cur_text_width_right = 0;

        let wp = csarg.win;
        let buf: *mut buf_T = (*wp).w_buffer;
        let expand_tab = cur_char == TAB && csarg.use_tabstop;
        let has_lcs_eol = (*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.eol != 0;

        // First the plain size, without 'linebreak' or inline virtual text.
        let mut size;
        let mut is_doublewidth = false;
        if expand_tab {
            size = tabstop_padding(vcol, (*buf).b_p_ts, (*buf).b_p_vts_array);
        } else if *cur == NUL {
            // One cell for the "eol" list char if there is one, as opposed to
            // the two-cell ^@ a NUL *in the text* would get.
            size = c_int::from(has_lcs_eol);
        } else if cur_char < 0 {
            size = INVALID_BYTE_CELLS;
        } else {
            size = ptr2cells(cur);
            is_doublewidth = size == 2 && cur_char >= 0x80;
        }

        if csarg.virt_row >= 0 {
            size = add_inline_virt_text(csarg, cur, vcol, size, expand_tab);
        }

        let mut mb_added = 0;
        if is_doublewidth && (*wp).w_onebuf_opt.wo_wrap != 0 && in_win_border(wp, vcol + size - 2) {
            // Count the ">" in the last column.
            size += 1;
            mb_added = 1;
        }

        let sbr = get_showbreak_value(wp);
        let mut head = mb_added;
        // When "size" is 0 no new screen line is started, so nothing to add.
        if size > 0
            && (*wp).w_onebuf_opt.wo_wrap != 0
            && (*sbr != NUL || (*wp).w_onebuf_opt.wo_bri != 0)
        {
            let extra = showbreak_head(csarg, cur, vcol, size, mb_added, sbr);
            head += extra.head;
            size += extra.added;
        }

        if breaks_here(csarg, cur) {
            size = linebreak_size(wp, cur, vcol, size);
        }

        CharSize { width: size, head }
    }
}

/// Like [`charsize_regular`] but with no inline virtual text, 'linebreak',
/// 'breakindent' or 'showbreak' to worry about: normal characters, tabs and
/// wrapping only. Always inlined — it is the per-character hot path.
///
/// # Safety
/// `wp` must be live and `cur` must point into a NUL-terminated line.
#[inline(always)]
unsafe fn charsize_fast_impl(
    wp: *mut win_T,
    cur: *const c_char,
    use_tabstop: bool,
    vcol: colnr_T,
    cur_char: int32_t,
) -> CharSize {
    unsafe {
        // A tab is expanded according to the column it starts at.
        if cur_char == TAB && use_tabstop {
            return CharSize {
                width: tabstop_padding(
                    vcol,
                    (*(*wp).w_buffer).b_p_ts,
                    (*(*wp).w_buffer).b_p_vts_array,
                ),
                head: 0,
            };
        }

        let width = if cur_char < 0 {
            INVALID_BYTE_CELLS
        } else {
            ptr2cells(cur)
        };

        // A double-width char that does not fit at the end of a screen line
        // wraps to the next one, and the last column shows a '>'.
        if width == 2
            && cur_char >= 0x80
            && (*wp).w_onebuf_opt.wo_wrap != 0
            && in_win_border(wp, vcol)
        {
            CharSize { width: 3, head: 1 }
        } else {
            CharSize { width, head: 0 }
        }
    }
}

/// [`charsize_fast_impl`] for callers holding a [`CharsizeArg`]. Only valid
/// when `init_charsize_arg` answered [`CharsizeKind::Fast`].
///
/// # Safety
/// As `charsize_fast_impl`.
#[inline]
pub unsafe fn charsize_fast(
    csarg: &CharsizeArg,
    cur: *const c_char,
    vcol: colnr_T,
    cur_char: int32_t,
) -> CharSize {
    unsafe { charsize_fast_impl(csarg.win, cur, csarg.use_tabstop, vcol, cur_char) }
}

/// Dispatch to whichever charsize function `init_charsize_arg` chose.
///
/// # Safety
/// As [`charsize_regular`].
#[inline(always)]
pub unsafe fn win_charsize(
    cstype: CharsizeKind,
    vcol: c_int,
    ptr: *mut c_char,
    chr: int32_t,
    csarg: &mut CharsizeArg,
) -> CharSize {
    unsafe {
        if cstype == CharsizeKind::Fast {
            charsize_fast(csarg, ptr, vcol, chr)
        } else {
            charsize_regular(csarg, ptr, vcol, chr)
        }
    }
}

/// Cells the character at `cur` takes when there is no wrapping to consider.
///
/// # Safety
/// `buf` must be live and `cur` must point into a NUL-terminated line.
pub unsafe fn charsize_nowrap(
    buf: *mut buf_T,
    cur: *const c_char,
    use_tabstop: bool,
    vcol: colnr_T,
    cur_char: int32_t,
) -> c_int {
    unsafe {
        if cur_char == TAB && use_tabstop {
            tabstop_padding(vcol, (*buf).b_p_ts, (*buf).b_p_vts_array)
        } else if cur_char < 0 {
            INVALID_BYTE_CELLS
        } else {
            ptr2cells(cur)
        }
    }
}

/// Whether `vcol` lands in the rightmost column of `wp`.
///
/// # Safety
/// `wp` must be live.
#[inline]
unsafe fn in_win_border(wp: *mut win_T, vcol: colnr_T) -> bool {
    unsafe {
        if (*wp).w_view_width == 0 {
            // There is no border.
            return false;
        }
        // Width of the first screen line, after the line number.
        let width1 = (*wp).w_view_width - win_col_off(wp);
        if vcol < width1 - 1 {
            return false;
        }
        if vcol == width1 - 1 {
            return true;
        }
        // Width of the wrapped screen lines after it.
        let width2 = width1 + win_col_off2(wp);
        if width2 <= 0 {
            return false;
        }
        (vcol - width1) % width2 == width2 - 1
    }
}

/// Virtual column reached after walking `csarg`'s line up to byte `len`,
/// starting from `vcol_arg`. Pass `MAXCOL` for the whole line, which also
/// counts inline virtual text sitting past its end.
///
/// # Safety
/// `csarg` must be initialised.
pub unsafe fn linesize_regular(
    csarg: &mut CharsizeArg,
    mut vcol_arg: c_int,
    len: colnr_T,
) -> c_int {
    unsafe {
        let line = csarg.line;
        let mut vcol = vcol_arg as int64_t;

        let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
        while ci.ptr.offset_from(line) < len as isize && *ci.ptr != NUL {
            vcol += charsize_regular(csarg, ci.ptr, vcol_arg, ci.chr.value).width as int64_t;
            ci = utfc_next(ci);
            if vcol > MAXCOL as int64_t {
                vcol_arg = MAXCOL;
                break;
            }
            vcol_arg = vcol as c_int;
        }

        // Inline virtual text after the end of the line.
        if len == MAXCOL && csarg.virt_row >= 0 && *ci.ptr == NUL {
            let head = charsize_regular(csarg, ci.ptr, vcol_arg, ci.chr.value).head;
            vcol += (csarg.cur_text_width_left + csarg.cur_text_width_right + head) as int64_t;
            vcol_arg = if vcol > MAXCOL as int64_t {
                MAXCOL
            } else {
                vcol as c_int
            };
        }

        vcol_arg
    }
}

/// [`linesize_regular`] for a line `init_charsize_arg` called
/// [`CharsizeKind::Fast`].
///
/// # Safety
/// `csarg` must be initialised.
pub unsafe fn linesize_fast(csarg: &CharsizeArg, mut vcol_arg: c_int, len: colnr_T) -> c_int {
    unsafe {
        let wp = csarg.win;
        let use_tabstop = csarg.use_tabstop;
        let line = csarg.line;
        let mut vcol = vcol_arg as int64_t;

        let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
        while ci.ptr.offset_from(line) < len as isize && *ci.ptr != NUL {
            vcol += charsize_fast_impl(wp, ci.ptr, use_tabstop, vcol_arg, ci.chr.value).width
                as int64_t;
            ci = utfc_next(ci);
            if vcol > MAXCOL as int64_t {
                vcol_arg = MAXCOL;
                break;
            }
            vcol_arg = vcol as c_int;
        }

        vcol_arg
    }
}

// Split out for size; the rest of the tree calls all of it as `plines::*`.
pub mod lines;
pub mod vcol;

pub use lines::{
    plines_m_win, plines_m_win_fill, plines_win, plines_win_col, plines_win_full,
    plines_win_nofill, plines_win_nofold, win_get_fill, win_may_fill, win_text_height,
};
pub use vcol::{getvcol, getvcol_nolist, getvcols, getvvcol};
