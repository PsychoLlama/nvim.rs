#![forbid(unsafe_code)]

//! How much room text takes up in a window.
//!
//! Two halves, in the order the original `plines.c` has them:
//!
//! * **Horizontal** — how many screen cells a character or a line occupies.
//!   `CharsizeArg::new` looks at the line once and answers whether the
//!   cheap measure ([`charsize_fast`]) is enough or the line needs the full
//!   one ([`charsize_regular`], which handles inline virtual text,
//!   'linebreak', 'breakindent' and 'showbreak'). Everything else here walks
//!   a line with one of those two.
//! * **Vertical** — how many window lines a buffer line occupies, built on
//!   the horizontal half plus folds, diff filler and virtual lines.
//!
//! This runs per character on every redraw. Prefer plain loops and keep the
//! small helpers inlined.

use crate::buffer::buf_meta_total;
use crate::charset::{byte2cells, vim_isbreak, vim_isprintc};
use crate::cstr::byte_at;
use crate::decoration::{decor_conceal_line, decor_virt_lines, mark_virt_chain, ns_in_win};
use crate::diff::{diff_check_fill, diffopt_filler};
use crate::fold::{has_folding, has_folding_win, line_folded};
use crate::indent::get_breakindent_win;
use crate::marktree::cursor::Cursor;
use crate::marktree::key::{kMTFilterSelect, mt_invalid, mt_right};
use crate::marktree::meta::MetaCount;
use crate::mbyte::{c_cells_in, c_char_info, c_cluster_len, c_next_cluster, char_at};
use crate::r#move::win_col_off2;
use crate::pos::{MAXCOL, lt, ltoreq};
use crate::state::mode::State;
use crate::state::{MODE_NORMAL, virtual_active};
use crate::types::{
    CharInfo, CharSize, CharsizeArg, CharsizeKind, ColNr, LineNr, MB_MAXCHAR, MetaIndex, NUL, Pos,
    int32_t, int64_t, uint32_t,
};
use crate::winlayer::{Buf, Win};

use core::ffi::{CStr, c_int, c_long};

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

/// The most bytes measuring one character reads: its own sequence and the
/// one after it, which a VS-16 can widen it by. A caller still holding a
/// pointer into a string hands the measure this much of it
/// (`cstr::prefix_at`), which answers what the whole string would -- and
/// does not pay a `strlen` per character for it.
pub(crate) const CHAR_CELLS_SPAN: usize = 2 * MB_MAXCHAR;

/// The marktree filter that selects inline virtual text and nothing else.
static INLINE_FILTER: MetaCount = [kMTFilterSelect, 0, 0, 0, 0];

impl Win {
    /// Whether a tab is expanded to a tabstop rather than shown as a
    /// 'listchars' character.
    fn expands_tab(self) -> bool {
        self.w_onebuf_opt.wo_list == 0 || self.w_p_lcs_chars.tab1 != 0
    }
}

// ---------------------------------------------------------------------------
// Horizontal size
// ---------------------------------------------------------------------------

/// Cells the character at byte `at` of `line` takes when it is not a tab,
/// `cur_char` being the codepoint there (negative for an invalid byte).
///
/// `ptr2cells` with the ASCII answer taken from the codepoint, so the byte
/// is not read again. An overlong encoding decodes to ASCII too, and is
/// drawn the way that character is -- except the overlong NUL, which is
/// `<c0><80>` on screen, so a codepoint of 0 reads the byte: a real NUL is
/// the two-cell `^@` there.
#[inline(always)]
fn char_cells(line: &[u8], at: usize, cur_char: int32_t) -> c_int {
    if cur_char < 0 {
        INVALID_BYTE_CELLS
    } else if (1..0x80).contains(&cur_char) {
        byte2cells(cur_char)
    } else {
        let byte = byte_at(line, at);
        if byte < 0x80 {
            byte2cells(c_int::from(byte))
        } else {
            c_cells_in(line, at)
        }
    }
}

/// Cells the first character of `text` takes on the screen, given that it
/// starts at virtual column `col` (which only matters for a tab).
pub(crate) fn win_chartabsize(window: Win, text: &[u8], col: ColNr) -> c_int {
    let byte = byte_at(text, 0);
    if c_int::from(byte) == TAB && window.expands_tab() {
        return window.buffer().tab_padding(col);
    }
    if byte < 0x80 {
        byte2cells(c_int::from(byte))
    } else {
        c_cells_in(text, 0)
    }
}

/// Cells the string `s` takes, as if it began at virtual column `startvcol`
/// of the current window.
pub(crate) fn linetabsize_col(startvcol: c_int, s: &[u8]) -> c_int {
    win_linetabsize_col(Win::current(), 0, s, startvcol, MAXCOL)
}

/// [`linetabsize_col`] of a string the caller holds.
pub(crate) fn linetabsize_text(startvcol: c_int, s: &CStr) -> c_int {
    linetabsize_col(startvcol, s.to_bytes())
}

/// The screen width of a whole line, starting from virtual column zero.
pub(crate) fn linetabsize_str(s: &[u8]) -> c_int {
    linetabsize_col(0, s)
}

/// Cells the first `len` bytes of `line` take in `window`, counting inline
/// virtual text. Pass `MAXCOL` for the whole line.
///
/// `line` is line `lnum` of `window`'s buffer, or any text when `lnum` is 0
/// (which skips virtual text). The walk reads no further than the end of
/// `line`, a NUL in it, or `len`, whichever comes first, so a slice that is
/// not that line misplaces the virtual text and reads nothing it does not
/// hold: the regexp matcher's `\%v` hands its *string* here with line 1,
/// which is upstream's behaviour.
#[inline(always)]
pub(crate) fn win_linetabsize(window: Win, lnum: LineNr, line: &[u8], len: ColNr) -> c_int {
    win_linetabsize_col(window, lnum, line, 0, len)
}

/// [`win_linetabsize`] starting from virtual column `startvcol` — the one
/// body both it and [`linetabsize_col`] are.
#[inline(always)]
fn win_linetabsize_col(
    window: Win,
    lnum: LineNr,
    line: &[u8],
    startvcol: c_int,
    len: ColNr,
) -> c_int {
    let (mut csarg, kind) = CharsizeArg::new(window, lnum, line);
    match kind {
        CharsizeKind::Fast => linesize_fast(&csarg, startvcol, len),
        CharsizeKind::Regular => linesize_regular(&mut csarg, startvcol, len),
    }
}

/// Cells line `lnum` takes in `window`, counting inline virtual text but not the
/// 'listchars' "eol".
pub(crate) fn linetabsize(window: Win, lnum: LineNr) -> c_int {
    let mut lines = window.buffer().lines();
    win_linetabsize(window, lnum, lines.line(lnum), MAXCOL)
}

/// Cells the first `len` bytes of line `lnum` take in `window`, counting
/// inline virtual text: the virtual column `len` starts at.
pub(crate) fn win_line_prefix_cells(window: Win, lnum: LineNr, len: ColNr) -> ColNr {
    let mut lines = window.buffer().lines();
    win_linetabsize(window, lnum, lines.line(lnum), len)
}

/// Like [`linetabsize`], but counts the 'listchars' "eol".
pub(crate) fn linetabsize_eol(window: Win, lnum: LineNr) -> c_int {
    let eol = window.w_onebuf_opt.wo_list != 0 && window.w_p_lcs_chars.eol != 0;
    linetabsize(window, lnum) + c_int::from(eol)
}

impl<'a> CharsizeArg<'a> {
    /// The walk state for `line`, and which charsize function the walk has
    /// to use.
    ///
    /// `lnum` of 0 means "do not consider inline virtual text", which is how
    /// the callers that measure a bare string rather than a buffer line ask
    /// for it; otherwise `line` is line `lnum` of `window`'s buffer.
    ///
    /// Always inlined: the walk state holds a marktree iterator, and built
    /// out of line it is copied into the caller's frame on the way back.
    #[inline(always)]
    pub(crate) fn new(window: Win, lnum: LineNr, line: &'a [u8]) -> (Self, CharsizeKind) {
        let mut csarg = CharsizeArg {
            win: window,
            line,
            use_tabstop: window.expands_tab(),
            indent_width: c_int::MIN,
            virt_row: -1,
            cur_text_width_left: 0,
            cur_text_width_right: 0,
            max_head_vcol: 0,
            iter: Default::default(),
        };

        let mut walk = Cursor::in_buffer(window.buffer(), &mut csarg.iter[0]);
        if lnum > 0 && walk.seek_filter(lnum - 1, 0, lnum, 0, &INLINE_FILTER) {
            csarg.virt_row = lnum - 1;
        }

        let needs_regular = csarg.virt_row >= 0
            || (window.w_onebuf_opt.wo_wrap != 0
                && (window.w_onebuf_opt.wo_lbr != 0
                    || window.w_onebuf_opt.wo_bri != 0
                    || window.has_showbreak()));
        let kind = if needs_regular {
            CharsizeKind::Regular
        } else {
            CharsizeKind::Fast
        };
        (csarg, kind)
    }
}

/// Virtual columns the inline virtual text around the cursor shifts it by.
///
/// `csarg` must hold the widths [`charsize_regular`] left in it.
fn virt_text_cursor_off(csarg: &CharsizeArg<'_>, on_nul: bool) -> c_int {
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
fn wrapped_indent_width(csarg: &mut CharsizeArg<'_>) -> c_int {
    if csarg.indent_width != c_int::MIN {
        return csarg.indent_width;
    }
    let wp = csarg.win;
    let mut width = wp.showbreak_cells();
    if wp.w_onebuf_opt.wo_bri != 0 {
        width += get_breakindent_win(wp, csarg.line);
    }
    csarg.indent_width = width;
    width
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
fn showbreak_head(
    csarg: &mut CharsizeArg<'_>,
    at: usize,
    vcol: ColNr,
    size: c_int,
    mb_added: c_int,
) -> BreakHead {
    let wp = csarg.win;
    let view_width = wp.w_view_width;
    let mut col_off_prev = wp.col_off();
    let width2 = view_width - col_off_prev + wp.col_off2();
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
        head_prev = wrapped_indent_width(csarg);
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
        let head_mid = wrapped_indent_width(csarg);
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
                let on_nul = byte_at(csarg.line, at) == NUL as u8;
                let off = mb_added + virt_text_cursor_off(csarg, on_nul);
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

/// Widen the character at byte `at` by the inline virtual text attached
/// there, advancing `csarg`'s marktree iterator past that position.
/// `csarg.virt_row` must be set.
///
/// A tab is re-measured after each chunk: inserting text moves the tab's end
/// to a different tabstop.
fn add_inline_virt_text(
    csarg: &mut CharsizeArg<'_>,
    at: usize,
    vcol: ColNr,
    mut size: c_int,
    expand_tab: bool,
) -> c_int {
    let wp = csarg.win;
    let buf = wp.buffer();
    let mut tab_size = size;
    let col = int32_t::try_from(at).unwrap_or(int32_t::MAX);

    // The walk steps the iterator while the two width fields are written, so
    // the borrows have to be split field by field.
    let virt_row = csarg.virt_row;
    let CharsizeArg {
        iter,
        cur_text_width_left,
        cur_text_width_right,
        ..
    } = csarg;
    let mut walk = Cursor::in_buffer(buf, &mut iter[0]);

    loop {
        let mark = walk.current();
        if mark.pos.row != virt_row || mark.pos.col > col {
            break;
        }
        if mark.pos.col == col && !mt_invalid(mark) && ns_in_win(mark.ns, wp) {
            let inline = mark_virt_chain(mark)
                .filter(|vt| vt.flags as c_int & VT_IS_LINES == 0)
                .filter(|vt| vt.pos as uint32_t == VPOS_INLINE);
            for vt in inline {
                if mt_right(mark) {
                    *cur_text_width_right += vt.width;
                } else {
                    *cur_text_width_left += vt.width;
                }
                size += vt.width;
                if expand_tab {
                    // The tab's width changes with the inserted text.
                    size -= tab_size;
                    tab_size = buf.tab_padding(vcol + size);
                    size += tab_size;
                }
            }
        }
        walk.step_filter(virt_row + 1, 0, &INLINE_FILTER);
    }
    size
}

/// Whether the character at byte `at` is where 'linebreak' would break the
/// line: a blank followed by a non-blank, outside the leading whitespace.
fn breaks_here(csarg: &CharsizeArg<'_>, at: usize) -> bool {
    let wp = csarg.win;
    if wp.w_onebuf_opt.wo_lbr == 0 || wp.w_onebuf_opt.wo_wrap == 0 || wp.w_view_width == 0 {
        return false;
    }
    let line = csarg.line;
    if !vim_isbreak(c_int::from(byte_at(line, at))) {
        return false;
    }
    // The byte after a break character is still inside the line: NUL is not
    // a break character. Upstream's `||` chain tests this one second for
    // that reason.
    if vim_isbreak(c_int::from(byte_at(line, at + 1))) {
        return false;
    }
    // 'linebreak' is only needed when not in leading whitespace. NUL is not
    // a break character, so the walk stops inside the line.
    let mut t = 0;
    while vim_isbreak(c_int::from(byte_at(line, t))) {
        t += 1;
    }
    at >= t
}

/// The 'linebreak' half of [`charsize_regular`]: the blank at byte `at` of
/// `line` is stretched so that the following word starts on the next screen
/// line.
fn linebreak_size(win: Win, line: &[u8], at: usize, vcol: ColNr, size: c_int) -> c_int {
    // Count all characters from the first non-blank after a blank up to the
    // next non-blank after a blank.
    let numberextra = win.col_off();
    let col_adj = size - 1;
    let mut colmax = win.w_view_width - numberextra - col_adj;
    if vcol >= colmax {
        colmax += col_adj;
        let n = colmax + win.col_off2();
        if n > 0 {
            colmax += ((vcol + col_adj - colmax) / n + 1) * n - col_adj;
        }
    }

    let mut s = at;
    let mut vcol2 = vcol;
    loop {
        let ps = s;
        s += c_cluster_len(line.get(s..).unwrap_or(&[]));
        let (c, prev) = (
            c_int::from(byte_at(line, s)),
            c_int::from(byte_at(line, ps)),
        );
        if c == NUL || !(vim_isbreak(c) || vcol2 == vcol || !vim_isbreak(prev)) {
            return size;
        }
        vcol2 += win_chartabsize(win, &line[s..], vcol2);
        if vcol2 >= colmax {
            // Doesn't fit.
            return colmax - vcol + col_adj;
        }
    }
}

/// Cells the character at byte `at` of `csarg`'s line takes, with everything
/// accounted for; `cur_char` is the codepoint there (negative for an invalid
/// byte).
///
/// Sets `csarg.cur_text_width_left`/`_right` to the inline virtual text
/// widths on either side of it. See [`showbreak_head`] for what
/// `csarg.max_head_vcol` selects.
pub(crate) fn charsize_regular(
    csarg: &mut CharsizeArg<'_>,
    at: usize,
    vcol: ColNr,
    cur_char: int32_t,
) -> CharSize {
    csarg.cur_text_width_left = 0;
    csarg.cur_text_width_right = 0;

    let wp = csarg.win;
    let buf = wp.buffer();
    let expand_tab = cur_char == TAB && csarg.use_tabstop;
    let has_lcs_eol = wp.w_onebuf_opt.wo_list != 0 && wp.w_p_lcs_chars.eol != 0;
    let at_nul = byte_at(csarg.line, at) == NUL as u8;

    // First the plain size, without 'linebreak' or inline virtual text.
    let mut size;
    let mut is_doublewidth = false;
    if expand_tab {
        size = buf.tab_padding(vcol);
    } else if at_nul {
        // One cell for the "eol" list char if there is one, as opposed to the
        // two-cell ^@ a NUL *in the text* would get.
        size = c_int::from(has_lcs_eol);
    } else {
        size = char_cells(csarg.line, at, cur_char);
        is_doublewidth = size == 2 && cur_char >= 0x80;
    }

    if csarg.virt_row >= 0 {
        size = add_inline_virt_text(csarg, at, vcol, size, expand_tab);
    }

    let mut mb_added = 0;
    if is_doublewidth && wp.w_onebuf_opt.wo_wrap != 0 && in_win_border(wp, vcol + size - 2) {
        // Count the ">" in the last column.
        size += 1;
        mb_added = 1;
    }

    let mut head = mb_added;
    // When "size" is 0 no new screen line is started, so nothing to add.
    if size > 0
        && wp.w_onebuf_opt.wo_wrap != 0
        && (wp.has_showbreak() || wp.w_onebuf_opt.wo_bri != 0)
    {
        let extra = showbreak_head(csarg, at, vcol, size, mb_added);
        head += extra.head;
        size += extra.added;
    }

    if breaks_here(csarg, at) {
        size = linebreak_size(wp, csarg.line, at, vcol, size);
    }

    CharSize { width: size, head }
}

/// Like [`charsize_regular`] but with no inline virtual text, 'linebreak',
/// 'breakindent' or 'showbreak' to worry about: normal characters, tabs and
/// wrapping only. Always inlined — it is the per-character hot path.
#[inline(always)]
fn charsize_fast_impl(
    window: Win,
    line: &[u8],
    at: usize,
    use_tabstop: bool,
    vcol: ColNr,
    cur_char: int32_t,
) -> CharSize {
    // A tab is expanded according to the column it starts at.
    if cur_char == TAB && use_tabstop {
        let width = window.buffer().tab_padding(vcol);
        return CharSize { width, head: 0 };
    }

    let width = char_cells(line, at, cur_char);

    // A double-width char that does not fit at the end of a screen line
    // wraps to the next one, and the last column shows a '>'.
    if width == 2
        && cur_char >= 0x80
        && window.w_onebuf_opt.wo_wrap != 0
        && in_win_border(window, vcol)
    {
        CharSize { width: 3, head: 1 }
    } else {
        CharSize { width, head: 0 }
    }
}

/// [`charsize_fast_impl`] for callers holding a [`CharsizeArg`]. Only valid
/// when [`CharsizeArg::new`] answered [`CharsizeKind::Fast`].
#[inline]
pub(crate) fn charsize_fast(
    csarg: &CharsizeArg<'_>,
    at: usize,
    vcol: ColNr,
    cur_char: int32_t,
) -> CharSize {
    charsize_fast_impl(csarg.win, csarg.line, at, csarg.use_tabstop, vcol, cur_char)
}

/// Dispatch to whichever charsize function [`CharsizeArg::new`] chose, for
/// the character at byte `at` of the line.
#[inline(always)]
pub(crate) fn win_charsize(
    cstype: CharsizeKind,
    vcol: c_int,
    at: usize,
    chr: int32_t,
    csarg: &mut CharsizeArg<'_>,
) -> CharSize {
    if cstype == CharsizeKind::Fast {
        charsize_fast(csarg, at, vcol, chr)
    } else {
        charsize_regular(csarg, at, vcol, chr)
    }
}

/// Cells the first character of `text` takes when there is no wrapping to
/// consider; `cur_char` is its codepoint.
pub(crate) fn charsize_nowrap(
    buffer: Buf,
    text: &[u8],
    use_tabstop: bool,
    vcol: ColNr,
    cur_char: int32_t,
) -> c_int {
    if cur_char == TAB && use_tabstop {
        buffer.tab_padding(vcol)
    } else {
        char_cells(text, 0, cur_char)
    }
}

/// Whether `vcol` lands in the rightmost column of `window`.
#[inline]
fn in_win_border(window: Win, vcol: ColNr) -> bool {
    let view_width = window.w_view_width;
    if view_width == 0 {
        // There is no border.
        return false;
    }
    // Width of the first screen line, after the line number.
    let width1 = view_width - window.col_off();
    if vcol < width1 - 1 {
        return false;
    }
    if vcol == width1 - 1 {
        return true;
    }
    // Width of the wrapped screen lines after it.
    let width2 = width1 + win_col_off2(window);
    if width2 <= 0 {
        return false;
    }
    (vcol - width1) % width2 == width2 - 1
}

/// Whether the character `chr` at byte `at` of `line` is the end of the
/// text: a NUL byte, or past the end. The codepoint is enough for every
/// character but the one that decodes to 0 -- a NUL, or the overlong `<c0><80>`
/// -- so the byte is only read for that one.
#[inline(always)]
fn ends_at(line: &[u8], at: usize, chr: CharInfo) -> bool {
    chr.value == 0 && byte_at(line, at) == NUL as u8
}

/// How far a walk over `line` may go: `len` bytes, or the whole line for a
/// larger `len` -- and none of it for a negative one, which is where the
/// pointer walk's `p - line < len` test stopped it.
#[inline(always)]
fn walk_end(line: &[u8], len: ColNr) -> usize {
    usize::try_from(len).map_or(0, |len| len.min(line.len()))
}

/// Virtual column reached after walking `csarg`'s line up to byte `len`,
/// starting from `vcol_arg`. Pass `MAXCOL` for the whole line, which also
/// counts inline virtual text sitting past its end.
pub(crate) fn linesize_regular(
    csarg: &mut CharsizeArg<'_>,
    mut vcol_arg: c_int,
    len: ColNr,
) -> c_int {
    let line = csarg.line;
    let end = walk_end(line, len);
    let mut vcol = vcol_arg as int64_t;

    let mut at = 0;
    let mut chr: CharInfo = c_char_info(line, 0);
    while at < end && !ends_at(line, at, chr) {
        vcol += charsize_regular(csarg, at, vcol_arg, chr.value).width as int64_t;
        (at, chr) = c_next_cluster(line, at, chr);
        if vcol > MAXCOL as int64_t {
            vcol_arg = MAXCOL;
            break;
        }
        vcol_arg = vcol as c_int;
    }

    // Inline virtual text after the end of the line.
    if len == MAXCOL && csarg.virt_row >= 0 && byte_at(line, at) == NUL as u8 {
        let head = charsize_regular(csarg, at, vcol_arg, chr.value).head;
        vcol += (csarg.cur_text_width_left + csarg.cur_text_width_right + head) as int64_t;
        vcol_arg = if vcol > MAXCOL as int64_t {
            MAXCOL
        } else {
            vcol as c_int
        };
    }

    vcol_arg
}

/// [`linesize_regular`] for a line [`CharsizeArg::new`] called
/// [`CharsizeKind::Fast`].
pub(crate) fn linesize_fast(csarg: &CharsizeArg<'_>, mut vcol_arg: c_int, len: ColNr) -> c_int {
    let use_tabstop = csarg.use_tabstop;
    let line = csarg.line;
    let end = walk_end(line, len);
    let wp = csarg.win;
    let mut vcol = vcol_arg as int64_t;

    let mut at = 0;
    let mut chr: CharInfo = c_char_info(line, 0);
    while at < end && !ends_at(line, at, chr) {
        vcol += charsize_fast_impl(wp, line, at, use_tabstop, vcol_arg, chr.value).width as int64_t;
        (at, chr) = c_next_cluster(line, at, chr);
        if vcol > MAXCOL as int64_t {
            vcol_arg = MAXCOL;
            break;
        }
        vcol_arg = vcol as c_int;
    }

    vcol_arg
}

// Split out for size; the rest of the tree calls all of it as `plines::*`.
pub(crate) mod lines;
pub(crate) mod vcol;

pub(crate) use lines::{
    plines_m_win, plines_m_win_fill, plines_win, plines_win_col, plines_win_full,
    plines_win_nofill, plines_win_nofold, win_get_fill, win_may_fill, win_text_height,
};
pub(crate) use vcol::{getvcol, getvcol_nolist, getvcols, getvvcol};
