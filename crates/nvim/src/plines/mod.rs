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

use crate::buffer::buf_meta_total;
use crate::charset::{ptr2cells, vim_isbreak, vim_isprintc, vim_strsize};
use crate::decoration::{decor_conceal_line, decor_virt_lines, mark_virt_chain, ns_in_win};
use crate::diff::{diff_check_fill, diffopt_filler};
use crate::fold::{has_folding, has_folding_win, line_folded};
use crate::indent::{get_breakindent_win, tabstop_padding};
use crate::main::{State, curwin, p_sel};
use crate::marktree::cursor::Cursor;
use crate::marktree::key::{kMTFilterSelect, mt_invalid, mt_right};
use crate::marktree::meta::MetaCount;
use crate::mbyte::{utf_ptr2char, utf_ptr2str_char_info, utfc_next, utfc_ptr2len};
use crate::memline::{ml_get_buf, ml_get_buf_len};
use crate::r#move::{win_col_off, win_col_off2};
use crate::option::get_showbreak_value;
use crate::pos::{MAXCOL, lt, ltoreq};
use crate::state::{MODE_NORMAL, virtual_active};
use crate::types::{
    CharSize, CharsizeArg, CharsizeKind, MetaIndex, NUL, OptInt, StrCharInfo, VirtLines, buf_T,
    colnr_T, foldinfo_T, int32_t, int64_t, linenr_T, pos_T, uint32_t, win_T,
};
use crate::winlayer::{Buf, Win};

use core::ffi::{c_char, c_int, c_long};

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

/// The marktree filter that selects inline virtual text and nothing else.
static INLINE_FILTER: MetaCount = [kMTFilterSelect, 0, 0, 0, 0];

// ---------------------------------------------------------------------------
// The promises this file rests on
//
// Every entry point below takes a live window (or a `CharsizeArg` that
// `init_charsize_arg` built from one) and a NUL-terminated line. These are the
// places that spend those promises; nothing else in the file needs one.
// ---------------------------------------------------------------------------

impl Win {
    /// The 'showbreak' in effect here — window-local or global — and whether
    /// it is non-empty, which is the only thing most callers ask.
    fn showbreak(self) -> (*mut c_char, bool) {
        // SAFETY: a live window.
        let sbr = unsafe { get_showbreak_value(self.raw()) };
        // SAFETY: 'showbreak' is a NUL-terminated option string.
        (sbr, unsafe { byte_at(sbr) } != NUL as c_int)
    }

    /// Cells 'breakindent' adds to a wrapped screen line of `line`.
    ///
    /// # Safety
    /// `line` must be NUL-terminated.
    unsafe fn breakindent(self, line: *mut c_char) -> c_int {
        // SAFETY: a live window and the caller's line.
        unsafe { get_breakindent_win(self.raw(), line) }
    }

    /// Whether a tab is expanded to a tabstop rather than shown as a
    /// 'listchars' character.
    fn expands_tab(self) -> bool {
        self.w_onebuf_opt.wo_list == 0 || self.w_p_lcs_chars.tab1 != 0
    }
}

impl Buf {
    /// Cells a tab starting at virtual column `col` takes here.
    fn tab_width(mut self, col: colnr_T) -> c_int {
        let (ts, vts): (OptInt, *const colnr_T) = (self.b_p_ts, self.b_p_vts_array);
        // SAFETY: a live buffer, whose 'vartabstop' array is its own.
        unsafe { tabstop_padding(col, ts, vts) }
    }
}

impl CharsizeArg {
    /// The window this walk measures in.
    ///
    /// Safe here: a `CharsizeArg` only reaches these functions through
    /// [`init_charsize_arg`], whose `# Safety` section is where the promise
    /// that `win` is live was taken.
    fn window(&self) -> Win {
        // SAFETY: as above.
        unsafe { Win::new(self.win) }
    }
}

/// The byte at `p`, as [`vim_isbreak`] and friends want it.
///
/// # Safety
/// `p` must point into a NUL-terminated line.
#[inline(always)]
unsafe fn byte_at(p: *const c_char) -> c_int {
    // SAFETY: the caller's pointer.
    unsafe { *p as u8 as c_int }
}

// ---------------------------------------------------------------------------
// Horizontal size
// ---------------------------------------------------------------------------

/// Cells the first character of `p` takes on the screen, given that it starts
/// at virtual column `col` (which only matters for a tab).
///
/// # Safety
/// `wp` must be live and `p` must point into a NUL-terminated line.
pub(crate) unsafe fn win_chartabsize(wp: *mut win_T, p: *mut c_char, col: colnr_T) -> c_int {
    // SAFETY: the caller's window.
    let wp = unsafe { Win::new(wp) };
    // SAFETY: the caller's pointer into a NUL-terminated line.
    if unsafe { byte_at(p) } == TAB && wp.expands_tab() {
        return wp.buffer().tab_width(col);
    }
    // SAFETY: as above.
    unsafe { ptr2cells(p) }
}

/// Cells the string `s` takes, as if it began at virtual column `startvcol`
/// of the current window.
///
/// # Safety
/// `s` must be a NUL-terminated string.
pub(crate) unsafe fn linetabsize_col(startvcol: c_int, s: *mut c_char) -> c_int {
    // SAFETY: `curwin` is live from startup to exit, and `s` is the caller's
    // NUL-terminated string.
    unsafe { win_linetabsize_col(curwin.get(), 0, s, startvcol, MAXCOL) }
}

/// The screen width of a whole line, starting from virtual column zero.
///
/// # Safety
/// `s` must be a NUL-terminated string.
pub(crate) unsafe fn linetabsize_str(s: *mut c_char) -> c_int {
    // SAFETY: the caller's NUL-terminated string.
    unsafe { linetabsize_col(0, s) }
}

/// Cells the first `len` bytes of `line` take in `wp`, counting inline
/// virtual text. Pass `MAXCOL` for the whole line.
///
/// # Safety
/// `wp` must be live; `line` must be line `lnum` of its buffer, or any
/// NUL-terminated string when `lnum` is 0 (which skips virtual text).
#[inline(always)]
pub(crate) unsafe fn win_linetabsize(
    wp: *mut win_T,
    lnum: linenr_T,
    line: *mut c_char,
    len: colnr_T,
) -> c_int {
    // SAFETY: the caller's window and line.
    unsafe { win_linetabsize_col(wp, lnum, line, 0, len) }
}

/// [`win_linetabsize`] starting from virtual column `startvcol` — the one
/// body both it and [`linetabsize_col`] are.
///
/// # Safety
/// As [`win_linetabsize`].
#[inline(always)]
unsafe fn win_linetabsize_col(
    wp: *mut win_T,
    lnum: linenr_T,
    line: *mut c_char,
    startvcol: c_int,
    len: colnr_T,
) -> c_int {
    let mut csarg = CharsizeArg::default();
    // SAFETY: the caller's window and line.
    let kind = unsafe { init_charsize_arg(&mut csarg, wp, lnum, line) };
    match kind {
        // SAFETY: `csarg` is now initialised for `line`.
        CharsizeKind::Fast => unsafe { linesize_fast(&csarg, startvcol, len) },
        // SAFETY: as above.
        CharsizeKind::Regular => unsafe { linesize_regular(&mut csarg, startvcol, len) },
    }
}

/// Cells line `lnum` takes in `wp`, counting inline virtual text but not the
/// 'listchars' "eol".
///
/// # Safety
/// `wp` must be live and `lnum` must be a line of its buffer.
pub(crate) unsafe fn linetabsize(wp: *mut win_T, lnum: linenr_T) -> c_int {
    // SAFETY: the caller's window, and `lnum` is a line of its buffer.
    let line = unsafe { Win::new(wp).buffer().line(lnum) };
    // SAFETY: as above.
    unsafe { win_linetabsize(wp, lnum, line.raw(), MAXCOL) }
}

/// Like [`linetabsize`], but counts the 'listchars' "eol".
///
/// # Safety
/// `wp` must be live and `lnum` must be a line of its buffer.
pub(crate) unsafe fn linetabsize_eol(wp: *mut win_T, lnum: linenr_T) -> c_int {
    // SAFETY: the caller's window.
    let win = unsafe { Win::new(wp) };
    let eol = win.w_onebuf_opt.wo_list != 0 && win.w_p_lcs_chars.eol != 0;
    // SAFETY: the caller's window, and `lnum` is a line of its buffer.
    unsafe { linetabsize(wp, lnum) + c_int::from(eol) }
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
pub(crate) unsafe fn init_charsize_arg(
    csarg: &mut CharsizeArg,
    wp: *mut win_T,
    lnum: linenr_T,
    line: *mut c_char,
) -> CharsizeKind {
    // SAFETY: the caller's window.
    let wp = unsafe { Win::new(wp) };
    csarg.win = wp.raw();
    csarg.line = line;
    csarg.max_head_vcol = 0;
    csarg.cur_text_width_left = 0;
    csarg.cur_text_width_right = 0;
    csarg.virt_row = -1;
    csarg.indent_width = c_int::MIN;
    csarg.use_tabstop = wp.expands_tab();

    let mut walk = Cursor::in_buffer(wp.buffer(), &mut csarg.iter[0]);
    if lnum > 0 && walk.seek_filter(lnum - 1, 0, lnum, 0, &INLINE_FILTER) {
        csarg.virt_row = lnum - 1;
    }

    let has_sbr = wp.showbreak().1;
    let needs_regular = csarg.virt_row >= 0
        || (wp.w_onebuf_opt.wo_wrap != 0
            && (wp.w_onebuf_opt.wo_lbr != 0 || wp.w_onebuf_opt.wo_bri != 0 || has_sbr));
    if needs_regular {
        CharsizeKind::Regular
    } else {
        CharsizeKind::Fast
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
    if csarg.indent_width != c_int::MIN {
        return csarg.indent_width;
    }
    let wp = csarg.window();
    let mut width = 0;
    // SAFETY: the caller's 'showbreak', a NUL-terminated string.
    if unsafe { byte_at(sbr) } != NUL as c_int {
        // SAFETY: as above.
        width += unsafe { vim_strsize(sbr) };
    }
    if wp.w_onebuf_opt.wo_bri != 0 {
        // SAFETY: `csarg` is initialised, so its line is NUL-terminated.
        width += unsafe { wp.breakindent(csarg.line) };
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
    {
        let wp = csarg.window();
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
            // SAFETY: the caller's 'showbreak' and initialised `csarg`.
            head_prev = unsafe { wrapped_indent_width(csarg, sbr) };
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
            // SAFETY: as above.
            let head_mid = unsafe { wrapped_indent_width(csarg, sbr) };
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
                    // SAFETY: `cur` points into `csarg`'s line.
                    let on_nul = unsafe { byte_at(cur) } == NUL as c_int;
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
    let wp = csarg.window();
    let buf = wp.buffer();
    let mut tab_size = size;
    // SAFETY: `cur` points into `csarg.line`, per the caller.
    let col = unsafe { cur.offset_from(csarg.line) } as int32_t;

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
                    tab_size = buf.tab_width(vcol + size);
                    size += tab_size;
                }
            }
        }
        walk.step_filter(virt_row + 1, 0, &INLINE_FILTER);
    }
    size
}

/// Whether the character at `cur` is where 'linebreak' would break the line:
/// a blank followed by a non-blank, outside the leading whitespace.
///
/// # Safety
/// `cur` must point into `csarg`'s NUL-terminated line.
unsafe fn breaks_here(csarg: &CharsizeArg, cur: *mut c_char) -> bool {
    let wp = csarg.window();
    if wp.w_onebuf_opt.wo_lbr == 0 || wp.w_onebuf_opt.wo_wrap == 0 || wp.w_view_width == 0 {
        return false;
    }
    // SAFETY: `cur` points into a NUL-terminated line.
    if !vim_isbreak(unsafe { byte_at(cur) }) {
        return false;
    }
    // SAFETY: `cur` is a break character, so it is NOT the line's terminating
    // NUL and the byte after it is still inside the line. Reading it
    // unconditionally walks off the end of the allocation, which is why
    // upstream's `||` chain tests this one second.
    if vim_isbreak(unsafe { byte_at(cur.offset(1)) }) {
        return false;
    }
    // 'linebreak' is only needed when not in leading whitespace.
    let mut t = csarg.line;
    // SAFETY: the line is NUL-terminated and NUL is not a break character,
    // so the walk stops inside it.
    while vim_isbreak(unsafe { byte_at(t) }) {
        t = t.wrapping_offset(1);
    }
    cur >= t
}

/// The 'linebreak' half of [`charsize_regular`]: the blank at `cur` is
/// stretched so that the following word starts on the next screen line.
///
/// # Safety
/// `wp` must be live and `cur` must point into a NUL-terminated line.
unsafe fn linebreak_size(win: Win, cur: *mut c_char, vcol: colnr_T, size: c_int) -> c_int {
    // Count all characters from the first non-blank after a blank up to the
    // next non-blank after a blank.
    let numberextra = win.col_off();
    let col_adj = size - 1;
    let mut colmax = win.w_view_width - numberextra - col_adj;
    if vcol >= colmax {
        colmax += col_adj;
        let n = colmax + win.col_off2();
        if n > 0 {
            colmax += ((vcol - colmax) / n + 1) * n - col_adj;
        }
    }

    let mut s = cur;
    let mut vcol2 = vcol;
    loop {
        let ps = s;
        // SAFETY: `s` walks a NUL-terminated line one character at a time.
        s = unsafe { s.offset(utfc_ptr2len(s) as isize) };
        // SAFETY: as above.
        let (c, prev) = unsafe { (byte_at(s), byte_at(ps)) };
        if c == NUL as c_int || !(vim_isbreak(c) || vcol2 == vcol || !vim_isbreak(prev)) {
            return size;
        }
        // SAFETY: a live window and a pointer into its line.
        vcol2 += unsafe { win_chartabsize(win.raw(), s, vcol2) };
        if vcol2 >= colmax {
            // Doesn't fit.
            return colmax - vcol + col_adj;
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
pub(crate) unsafe fn charsize_regular(
    csarg: &mut CharsizeArg,
    cur: *mut c_char,
    vcol: colnr_T,
    cur_char: int32_t,
) -> CharSize {
    csarg.cur_text_width_left = 0;
    csarg.cur_text_width_right = 0;

    let wp = csarg.window();
    let buf = wp.buffer();
    let expand_tab = cur_char == TAB && csarg.use_tabstop;
    let has_lcs_eol = wp.w_onebuf_opt.wo_list != 0 && wp.w_p_lcs_chars.eol != 0;
    // SAFETY: `cur` points into `csarg`'s NUL-terminated line.
    let at_nul = unsafe { byte_at(cur) } == NUL as c_int;

    // First the plain size, without 'linebreak' or inline virtual text.
    let mut size;
    let mut is_doublewidth = false;
    if expand_tab {
        size = buf.tab_width(vcol);
    } else if at_nul {
        // One cell for the "eol" list char if there is one, as opposed to the
        // two-cell ^@ a NUL *in the text* would get.
        size = c_int::from(has_lcs_eol);
    } else if cur_char < 0 {
        size = INVALID_BYTE_CELLS;
    } else {
        // SAFETY: as above.
        size = unsafe { ptr2cells(cur) };
        is_doublewidth = size == 2 && cur_char >= 0x80;
    }

    if csarg.virt_row >= 0 {
        // SAFETY: `virt_row >= 0` is what the walk needs, and `cur` points
        // into the line `csarg` was initialised for.
        size = unsafe { add_inline_virt_text(csarg, cur, vcol, size, expand_tab) };
    }

    let mut mb_added = 0;
    // SAFETY: `csarg`'s window is live.
    if is_doublewidth
        && wp.w_onebuf_opt.wo_wrap != 0
        && unsafe { in_win_border(wp.raw(), vcol + size - 2) }
    {
        // Count the ">" in the last column.
        size += 1;
        mb_added = 1;
    }

    let (sbr, has_sbr) = wp.showbreak();
    let mut head = mb_added;
    // When "size" is 0 no new screen line is started, so nothing to add.
    if size > 0 && wp.w_onebuf_opt.wo_wrap != 0 && (has_sbr || wp.w_onebuf_opt.wo_bri != 0) {
        // SAFETY: as above, plus the caller's initialised `csarg`.
        let extra = unsafe { showbreak_head(csarg, cur, vcol, size, mb_added, sbr) };
        head += extra.head;
        size += extra.added;
    }

    // SAFETY: `cur` points into `csarg`'s NUL-terminated line.
    if unsafe { breaks_here(csarg, cur) } {
        // SAFETY: as above.
        size = unsafe { linebreak_size(wp, cur, vcol, size) };
    }

    CharSize { width: size, head }
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
    // A tab is expanded according to the column it starts at.
    if cur_char == TAB && use_tabstop {
        // SAFETY: a live window's buffer is live, and its 'vartabstop' array
        // is its own.
        let width = unsafe {
            let buf = (*wp).w_buffer;
            tabstop_padding(vcol, (*buf).b_p_ts, (*buf).b_p_vts_array)
        };
        return CharSize { width, head: 0 };
    }

    let width = if cur_char < 0 {
        INVALID_BYTE_CELLS
    } else {
        // SAFETY: the caller's pointer into a NUL-terminated line.
        unsafe { ptr2cells(cur) }
    };

    // A double-width char that does not fit at the end of a screen line
    // wraps to the next one, and the last column shows a '>'.
    // SAFETY: the caller's window, on both sides of the `&&`.
    if width == 2
        && cur_char >= 0x80
        && unsafe { (*wp).w_onebuf_opt.wo_wrap } != 0
        && unsafe { in_win_border(wp, vcol) }
    {
        CharSize { width: 3, head: 1 }
    } else {
        CharSize { width, head: 0 }
    }
}

/// [`charsize_fast_impl`] for callers holding a [`CharsizeArg`]. Only valid
/// when `init_charsize_arg` answered [`CharsizeKind::Fast`].
///
/// # Safety
/// As `charsize_fast_impl`.
#[inline]
pub(crate) unsafe fn charsize_fast(
    csarg: &CharsizeArg,
    cur: *const c_char,
    vcol: colnr_T,
    cur_char: int32_t,
) -> CharSize {
    // SAFETY: `csarg` is initialised and `cur` points into its line.
    unsafe { charsize_fast_impl(csarg.win, cur, csarg.use_tabstop, vcol, cur_char) }
}

/// Dispatch to whichever charsize function `init_charsize_arg` chose.
///
/// # Safety
/// As [`charsize_regular`].
#[inline(always)]
pub(crate) unsafe fn win_charsize(
    cstype: CharsizeKind,
    vcol: c_int,
    ptr: *mut c_char,
    chr: int32_t,
    csarg: &mut CharsizeArg,
) -> CharSize {
    if cstype == CharsizeKind::Fast {
        // SAFETY: `csarg` is initialised and `ptr` points into its line.
        unsafe { charsize_fast(csarg, ptr, vcol, chr) }
    } else {
        // SAFETY: as above.
        unsafe { charsize_regular(csarg, ptr, vcol, chr) }
    }
}

/// Cells the character at `cur` takes when there is no wrapping to consider.
///
/// # Safety
/// `buf` must be live and `cur` must point into a NUL-terminated line.
pub(crate) unsafe fn charsize_nowrap(
    buf: *mut buf_T,
    cur: *const c_char,
    use_tabstop: bool,
    vcol: colnr_T,
    cur_char: int32_t,
) -> c_int {
    if cur_char == TAB && use_tabstop {
        // SAFETY: the caller's buffer.
        unsafe { Buf::new(buf) }.tab_width(vcol)
    } else if cur_char < 0 {
        INVALID_BYTE_CELLS
    } else {
        // SAFETY: the caller's pointer into a NUL-terminated line.
        unsafe { ptr2cells(cur) }
    }
}

/// Whether `vcol` lands in the rightmost column of `wp`.
///
/// Takes the raw pointer rather than a [`Win`]: this is inlined into the
/// per-character fast loop, and going through the wrapper there costs
/// measurable throughput (F-P17-10).
///
/// # Safety
/// `wp` must be live.
#[inline]
unsafe fn in_win_border(wp: *mut win_T, vcol: colnr_T) -> bool {
    // SAFETY: the caller's window.
    let view_width = unsafe { (*wp).w_view_width };
    if view_width == 0 {
        // There is no border.
        return false;
    }
    // Width of the first screen line, after the line number.
    // SAFETY: as above.
    let width1 = view_width - unsafe { win_col_off(wp) };
    if vcol < width1 - 1 {
        return false;
    }
    if vcol == width1 - 1 {
        return true;
    }
    // Width of the wrapped screen lines after it.
    // SAFETY: as above.
    let width2 = width1 + unsafe { win_col_off2(wp) };
    if width2 <= 0 {
        return false;
    }
    (vcol - width1) % width2 == width2 - 1
}

/// Virtual column reached after walking `csarg`'s line up to byte `len`,
/// starting from `vcol_arg`. Pass `MAXCOL` for the whole line, which also
/// counts inline virtual text sitting past its end.
///
/// # Safety
/// `csarg` must be initialised.
pub(crate) unsafe fn linesize_regular(
    csarg: &mut CharsizeArg,
    mut vcol_arg: c_int,
    len: colnr_T,
) -> c_int {
    let line = csarg.line;
    let mut vcol = vcol_arg as int64_t;

    // SAFETY: `csarg` is initialised, so its line is NUL-terminated.
    let mut ci: StrCharInfo = unsafe { utf_ptr2str_char_info(line) };
    // SAFETY: `ci` walks that line, so both the length test and the step are
    // inside it.
    while unsafe { ci.ptr.offset_from(line) } < len as isize && unsafe { byte_at(ci.ptr) } != 0 {
        // SAFETY: as above.
        vcol += unsafe { charsize_regular(csarg, ci.ptr, vcol_arg, ci.chr.value) }.width as int64_t;
        // SAFETY: as above.
        ci = unsafe { utfc_next(ci) };
        if vcol > MAXCOL as int64_t {
            vcol_arg = MAXCOL;
            break;
        }
        vcol_arg = vcol as c_int;
    }

    // Inline virtual text after the end of the line.
    // SAFETY: as above.
    if len == MAXCOL && csarg.virt_row >= 0 && unsafe { byte_at(ci.ptr) } == 0 {
        // SAFETY: as above.
        let head = unsafe { charsize_regular(csarg, ci.ptr, vcol_arg, ci.chr.value) }.head;
        vcol += (csarg.cur_text_width_left + csarg.cur_text_width_right + head) as int64_t;
        vcol_arg = if vcol > MAXCOL as int64_t {
            MAXCOL
        } else {
            vcol as c_int
        };
    }

    vcol_arg
}

/// [`linesize_regular`] for a line `init_charsize_arg` called
/// [`CharsizeKind::Fast`].
///
/// # Safety
/// `csarg` must be initialised.
pub(crate) unsafe fn linesize_fast(
    csarg: &CharsizeArg,
    mut vcol_arg: c_int,
    len: colnr_T,
) -> c_int {
    let wp = csarg.win;
    let use_tabstop = csarg.use_tabstop;
    let line = csarg.line;
    let mut vcol = vcol_arg as int64_t;

    // SAFETY: `csarg` is initialised, so its line is NUL-terminated.
    let mut ci: StrCharInfo = unsafe { utf_ptr2str_char_info(line) };
    // SAFETY: `ci` walks that line, so both the length test and the step are
    // inside it.
    while unsafe { ci.ptr.offset_from(line) } < len as isize && unsafe { *ci.ptr } != NUL as c_char
    {
        // SAFETY: as above, plus the live window `csarg` was built from.
        vcol += unsafe { charsize_fast_impl(wp, ci.ptr, use_tabstop, vcol_arg, ci.chr.value) }.width
            as int64_t;
        // SAFETY: as above.
        ci = unsafe { utfc_next(ci) };
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
