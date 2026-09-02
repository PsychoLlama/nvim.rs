//! Painting the menu onto its own grid.
//!
//! [`pum_redraw`] draws one row per visible item: the three item columns in
//! `'completeitemalign'` order, the truncation marker when a column does not
//! fit, and the scrollbar. Everything else here serves it —
//! [`pum_compute_text_attrs`] works out the per-cell attributes that make the
//! typed leader stand out inside a match.
//!
//! The menu has a grid of its own (`pum_grid`), composited over the editor
//! grid, so every row is one `screengrid_line_start` .. `grid_line_flush`
//! batch and all the columns below are grid columns, not screen columns.
//! Under `'rightleft'` the row is drawn from the right edge leftwards, which
//! is why nearly every step here has two spellings.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::grid::linebuf;
use crate::message::emsg_ptr;
use crate::types::kFloatRelativeEditor;

/// `WIN_CONFIG_INIT`: the float config a fresh `parse_winborder` writes into.
///
/// Only the border half is used here, but `grid_draw_border` also reads the
/// title and footer fields, so the whole thing has to start out empty.
const WIN_CONFIG_INIT: WinConfig = WinConfig {
    window: 0,
    bufpos: lpos_T { lnum: -1, col: 0 },
    height: 0,
    width: 0,
    row: 0.0,
    col: 0.0,
    anchor: 0,
    relative: kFloatRelativeEditor,
    external: false,
    focusable: true,
    mouse: true,
    split: kWinSplitLeft,
    zindex: kZIndexFloatDefault as c_int,
    style: kWinStyleUnused,
    border: false,
    shadow: false,
    border_chars: [[0; 32]; 8],
    border_hl_ids: [0; 8],
    border_attr: [0; 8],
    title: false,
    title_pos: kAlignLeft,
    title_chunks: VirtText {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<VirtTextChunk>(),
    },
    title_width: 0,
    footer: false,
    footer_pos: kAlignLeft,
    footer_chunks: VirtText {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<VirtTextChunk>(),
    },
    footer_width: 0,
    noautocmd: false,
    fixed: false,
    hide: false,
    _cmdline_offset: c_int::MAX,
};

/// The order the three item columns are drawn in, per `'completeitemalign'`.
///
/// `cia_flags` holds the three column kinds as decimal digits, most
/// significant first; zero means the option is at its default.
#[inline]
fn pum_align_order() -> [c_int; 3] {
    let flags = cia_flags.get();
    if flags == 0 {
        return [CPT_ABBR as c_int, CPT_KIND as c_int, CPT_MENU as c_int];
    }
    [
        (flags / 100) as c_int,
        (flags / 10 % 10) as c_int,
        (flags % 10) as c_int,
    ]
}

/// One item's text for a column kind, or null when it has none.
///
/// # Safety
/// `idx` must be in range for the live item array.
#[inline]
unsafe fn pum_get_item(index: c_int, item_type: c_int) -> *mut c_char {
    // SAFETY: the caller guarantees `index` addresses a live item.
    let item = unsafe { &pum_items()[index as usize] };
    match item_type as c_uint {
        CPT_ABBR => item.pum_text,
        CPT_KIND => item.pum_kind,
        CPT_MENU => item.pum_extra,
        _ => ::core::ptr::null_mut(),
    }
}

/// Fold an item's own `abbr_hlattr`/`kind_hlattr` into the column attribute.
///
/// Only the first two column kinds can carry one; the caller never asks for
/// the third.
///
/// # Safety
/// `idx` must be in range for the live item array, `item_type` 0 or 1.
#[inline]
unsafe fn pum_user_attr_combine(idx: c_int, item_type: c_int, attr: c_int) -> c_int {
    // SAFETY: the caller guarantees both indices.
    let item = &unsafe { pum_items() }[idx as usize];
    let user_attr = [item.pum_user_abbr_hlattr, item.pum_user_kind_hlattr][item_type as usize];
    if user_attr > 0 {
        unsafe { hl_combine_attr(attr, user_attr) }
    } else {
        attr
    }
}

/// Per-cell attributes for one item's text, or `None` for "all the same".
///
/// The point is to show which part of the item the typed leader matched:
/// those cells get `PmenuMatch`/`PmenuMatchSel` blended over the column's own
/// highlight. With `'completeopt'` containing `fuzzy` the matched cells are
/// whatever the fuzzy matcher scored; otherwise it is the leading run that
/// case-insensitively equals the leader.
///
/// `None` is answered whenever no cell could differ from the rest — which is
/// the common case, and is why the caller can skip the per-cell path.
///
/// # Safety
/// `text` must be NUL-terminated.
unsafe fn pum_compute_text_attrs(
    text: *mut c_char,
    hlf: hlf_T,
    user_hlattr: c_int,
) -> Option<Vec<c_int>> {
    // SAFETY: `text` is the caller's NUL-terminated string; `ins_compl_leader`
    // and `cmdline_compl_pattern` answer editor-owned strings.
    let win = curwin.get();
    if unsafe { *text } == 0
        || (hlf != HLF_PSI as hlf_T && hlf != HLF_PNI as hlf_T)
        || (unsafe { win_hl_attr(win, HLF_PMSI) } == unsafe { win_hl_attr(win, HLF_PSI) }
            && unsafe { win_hl_attr(win, HLF_PMNI) } == unsafe { win_hl_attr(win, HLF_PNI) })
    {
        return None;
    }

    let leader = if State.get() & MODE_CMDLINE != 0 {
        unsafe { cmdline_compl_pattern() }
    } else {
        ins_compl_leader()
    };
    if leader.is_null() || unsafe { *leader } == 0 {
        return None;
    }

    let in_fuzzy = if State.get() & MODE_CMDLINE != 0 {
        unsafe { cmdline_compl_is_fuzzy() }
    } else {
        let flags = unsafe { get_cot_flags() };
        flags & kOptCotFlagFuzzy != 0
    };
    // The fuzzy matcher answers the character positions it matched, or
    // null when the item does not match at all.
    let ga = if in_fuzzy {
        let ga = unsafe { fuzzy_match_str_with_pos(text, leader) };
        if ga.is_null() {
            return None;
        }
        Some(ga)
    } else {
        None
    };

    // The attribute a matched cell gets. Upstream rebuilds it from three
    // lookups per matched *character*; it cannot change during the walk,
    // so it is built at most once, and not at all when nothing matches.
    let is_select = hlf == HLF_PSI as hlf_T;
    let mut matched: Option<c_int> = None;
    let mut matched_attr = |win| {
        *matched.get_or_insert_with(|| {
            let a = unsafe {
                hl_combine_attr(
                    win_hl_attr(win, HLF_PMNI),
                    win_hl_attr(win, if is_select { HLF_PMSI } else { HLF_PMNI }),
                )
            };
            unsafe { hl_combine_attr(win_hl_attr(win, hlf as c_int), a) }
        })
    };

    let leader_len = unsafe { cstr::bytes_at(leader) }.len();
    let mut attrs = vec![0; unsafe { vim_strsize(text) } as usize];
    let mut ptr: *const c_char = text;
    let mut cell_idx = 0;
    let mut char_pos: uint32_t = 0;
    // Outside fuzzy matching the leader matches one leading run, counted
    // down in bytes as the walk passes over it.
    let mut matched_len: c_int = -1;

    while unsafe { *ptr } != 0 {
        let mut new_attr = unsafe { win_hl_attr(win, hlf as c_int) };
        if let Some(ga) = ga {
            let positions = unsafe { (*ga).ga_data }.cast::<uint32_t>();
            for i in 0..unsafe { (*ga).ga_len } {
                if char_pos == unsafe { *positions.offset(i as isize) } {
                    new_attr = matched_attr(win);
                    break;
                }
            }
        } else {
            if matched_len < 0 && unsafe { mb_strnicmp(ptr, leader, leader_len) } == 0 {
                matched_len = leader_len as c_int;
            }
            if matched_len > 0 {
                new_attr = matched_attr(win);
                matched_len -= 1;
            }
        }

        new_attr = unsafe { hl_combine_attr(win_hl_attr(win, HLF_PNI), new_attr) };
        if user_hlattr > 0 {
            new_attr = unsafe { hl_combine_attr(new_attr, user_hlattr) };
        }

        let char_cells = unsafe { utf_ptr2cells(ptr) };
        for i in 0..char_cells {
            attrs[(cell_idx + i) as usize] = new_attr;
        }
        cell_idx += char_cells;

        ptr = unsafe { ptr.offset(utfc_ptr2len(ptr) as isize) };
        char_pos += 1;
    }

    if let Some(ga) = ga {
        unsafe { ga_clear(ga) };
        unsafe { xfree(ga.cast()) };
    }
    Some(attrs)
}

/// Put `text` at `col` one character at a time, each with its own attribute.
///
/// `attrs` is indexed by cell offset into the *unreversed* text, so under
/// `'rightleft'` — where `text` has already been reversed — the index is
/// mirrored across the `cells` the run occupies.
///
/// # Safety
/// A line batch must be in progress and `text` must be NUL-terminated.
unsafe fn pum_grid_puts_with_attrs(col: c_int, cells: c_int, text: *const c_char, attrs: &[c_int]) {
    // SAFETY: the caller holds the batch and owns `text`.
    let col_start = col;
    let mut col = col;
    let mut ptr = text;
    while unsafe { *ptr } != 0 {
        let char_len = unsafe { utfc_ptr2len(ptr) };
        let at = if pum_rl.get() {
            col_start + cells - col - 1
        } else {
            col - col_start
        };
        unsafe { grid_line_puts(col, ptr, char_len, attrs[at as usize]) };
        col += unsafe { utf_ptr2cells(ptr) };
        ptr = unsafe { ptr.offset(char_len as isize) };
    }
}

/// `'pumborder'`, resolved to what a redraw needs.
///
/// The config it is read into stays in the caller's frame: it is ~460 bytes
/// and `grid_draw_border` wants a pointer to it, so moving it around costs
/// more than it reads.
struct PumBorder {
    /// Cells the border costs: 0 (none), 1 (shadow, right and bottom only)
    /// or 2 (a full box).
    width: c_int,
    /// Glyph and attribute the scrollbar trough borrows from the border's
    /// right edge. Only set when there is a scrollbar to draw.
    scrollbar: Option<(schar_T, c_int)>,
}

/// Read `'pumborder'` into `config`.
///
/// Answers `None` only when the option fails to parse, in which case the
/// message has already been given and the caller must draw nothing.
///
/// # Safety
/// The highlight tables must be initialised.
unsafe fn resolve_border(config: &mut WinConfig) -> Option<PumBorder> {
    // SAFETY: `p_pumborder` is an editor-owned string; `parse_winborder`
    // writes through the config pointer.
    let width = unsafe { pum_border_width() };
    if width == 0 {
        return Some(PumBorder {
            width,
            scrollbar: None,
        });
    }

    let mut err = Error::none();
    if !unsafe { parse_winborder(&raw mut *config, p_pumborder.get(), &mut err) } {
        if err.is_set() {
            unsafe { emsg_ptr(err.message_or_empty().as_ptr()) };
        }
        err.clear();
        return None;
    }
    err.clear();

    // The shadow style is not a box: it darkens the cells to the right
    // and below instead, in two dedicated highlight groups.
    if unsafe { strequal(p_pumborder.get(), BORDER_SHADOW.as_ptr()) } {
        config.shadow = true;
        let blend = unsafe { syn_check_group(c"PmenuShadow".as_ptr(), 11) };
        let through = unsafe { syn_check_group(c"PmenuShadowThrough".as_ptr(), 18) };
        config.border_hl_ids[2] = through;
        config.border_hl_ids[3] = blend;
        config.border_hl_ids[4] = blend;
        config.border_hl_ids[5] = blend;
        config.border_hl_ids[6] = through;
    }

    // Resolve the eight edges' highlight ids, PmenuBorder by default.
    for i in 0..8 {
        config.border_attr[i] = if config.border_hl_ids[i] != 0 {
            unsafe { hl_get_ui_attr(-1, HLF_PBR, config.border_hl_ids[i], false) }
        } else {
            unsafe { *hl_attr_active.get().offset(HLF_PBR as isize) }
        };
    }

    let scrollbar = (pum_scrollbar.get() != 0).then(|| {
        let right = (&raw const config.border_chars[3]).cast::<c_char>();
        (unsafe { schar_from_str(right) }, config.border_attr[3])
    });
    Some(PumBorder { width, scrollbar })
}

/// What every row of the menu draws the same way.
struct RowStyle {
    /// Grid column the item text starts at: the left edge, or the right one
    /// under `'rightleft'`.
    col_off: c_int,
    /// There is room for a padding space before the text (after it, under
    /// `'rightleft'`).
    extra_space: bool,
    /// The `'fillchars'` `trunc`/`truncrl` glyph, 0 when unset.
    fcs_trunc: schar_T,
    /// Scrollbar trough and thumb attributes.
    attr_scroll: c_int,
    attr_thumb: c_int,
    /// Trough glyph and attribute borrowed from a box border, if there is one.
    border_scroll: Option<(schar_T, c_int)>,
    /// The thumb's rows, as a start and a length.
    thumb_pos: c_int,
    thumb_height: c_int,
}

/// One row of the menu, mid-draw.
struct PumRow {
    /// The item this row shows.
    idx: c_int,
    /// Highlight groups for this row's three columns, selected or not.
    hlfs: [hlf_T; 3],
    /// Grid column the next glyph goes at.
    grid_col: c_int,
    /// Cells of `pum_width` used so far — what the width limit is against.
    totwidth: c_int,
    /// A column did not fit: the row gets a truncation marker.
    need_trunc: bool,
    /// The last column's own attribute, which the trailing fill uses.
    orig_attr: c_int,
}

impl PumRow {
    /// Draw one run of an item's text, ending at the truncation point.
    ///
    /// `text` is an owned `transstr` result (already reversed under
    /// `'rightleft'`) and `width` the cells the run was measured at; the
    /// truncating branch shortens both. Advances `grid_col` past the run.
    ///
    /// # Safety
    /// A line batch must be in progress and `text` must be a live
    /// NUL-terminated buffer of the run.
    unsafe fn emit(
        &mut self,
        style: &RowStyle,
        text: *mut c_char,
        mut width: c_int,
        attrs: Option<&[c_int]>,
        attr: c_int,
        next_isempty: bool,
    ) {
        // SAFETY: the caller holds the batch and owns `text`.
        let width_limit = pum_width.get();
        // Two cells are kept for the separator unless nothing follows.
        let pad = if next_isempty { 0 } else { 2 };
        let mut cells = unsafe { mb_string2cells(text) } as c_int;
        if width_limit - self.totwidth < cells + pad {
            self.need_trunc = true;
        }

        if pum_rl.get() {
            let mut rt = text;
            // Drop leading characters until what is left fits.
            if self.grid_col - cells < style.col_off - width_limit {
                loop {
                    cells -= unsafe { utf_ptr2cells(rt) };
                    rt = unsafe { rt.offset(utfc_ptr2len(rt) as isize) };
                    if self.grid_col - cells >= style.col_off - width_limit {
                        break;
                    }
                }
                if self.grid_col - cells > style.col_off - width_limit {
                    // The leftmost character wants two cells and only one
                    // is left: mark it with a '<' instead.
                    rt = unsafe { rt.offset(-1) };
                    unsafe { *rt = b'<' as c_char };
                    cells += 1;
                }
            }

            match attrs {
                None => {
                    unsafe { grid_line_puts(self.grid_col - cells + 1, rt, -1, attr) };
                }
                Some(attrs) => {
                    unsafe {
                        pum_grid_puts_with_attrs(self.grid_col - cells + 1, cells, rt, attrs)
                    };
                }
            }
            self.grid_col -= width;
        } else {
            if self.need_trunc {
                // Cut the run at the last character that still fits.
                let available_cells = width_limit - self.totwidth;
                let mut p_end = text;
                let mut displayed = 0;
                while unsafe { *p_end } != 0 {
                    let char_cells = unsafe { utf_ptr2cells(p_end) };
                    if displayed + char_cells > available_cells {
                        break;
                    }
                    displayed += char_cells;
                    p_end = unsafe { p_end.offset(utfc_ptr2len(p_end) as isize) };
                }
                unsafe { *p_end = 0 };
                cells = displayed;
                width = displayed;
            }

            match attrs {
                None => {
                    unsafe { grid_line_puts(self.grid_col, text, -1, attr) };
                }
                Some(attrs) => {
                    unsafe { pum_grid_puts_with_attrs(self.grid_col, cells, text, attrs) };
                }
            }
            self.grid_col += width;
        }
    }

    /// Draw one of the three item columns.
    ///
    /// The text is walked forwards until it runs out, hits a Tab or would
    /// exceed `pum_width`; each such run is made printable with `transstr`
    /// and drawn, and a Tab is drawn as two spaces before the walk resumes.
    ///
    /// # Safety
    /// A line batch must be in progress and `self.idx` must address a live
    /// item.
    unsafe fn put_column(&mut self, style: &RowStyle, item_type: c_int, next_isempty: bool) {
        // SAFETY: the caller holds the batch; `p` walks an item string, which
        // is NUL-terminated and stays live for the whole redraw.
        let win = curwin.get();
        let hlf = self.hlfs[item_type as usize];
        self.orig_attr =
            unsafe { hl_combine_attr(win_hl_attr(win, HLF_PNI), win_hl_attr(win, hlf as c_int)) };
        let attr = if item_type < 2 {
            unsafe { pum_user_attr_combine(self.idx, item_type, self.orig_attr) }
        } else {
            self.orig_attr
        };

        let mut p = unsafe { pum_get_item(self.idx, item_type) };
        if p.is_null() {
            return;
        }
        let mut run_start: *mut c_char = ::core::ptr::null_mut();
        let mut width = 0;

        loop {
            if run_start.is_null() {
                run_start = p;
            }
            let w = unsafe { ptr2cells(p) };
            if unsafe { *p } != 0
                && unsafe { *p } != b'\t' as c_char
                && self.totwidth + w <= pum_width.get()
            {
                width += w;
                p = unsafe { p.offset(utfc_ptr2len(p) as isize) };
                continue;
            }

            // Draw [run_start, p). The item string is NUL-terminated in
            // place for `transstr`, then put back.
            let saved = unsafe { *p };
            if saved != 0 {
                unsafe { *p = 0 };
            }
            let st = unsafe { transstr(run_start, true) };
            if saved != 0 {
                unsafe { *p = saved };
            }

            let attrs = if item_type == CPT_ABBR as c_int {
                let user = unsafe { pum_items() }[self.idx as usize].pum_user_abbr_hlattr;
                unsafe { pum_compute_text_attrs(st, hlf, user) }
            } else {
                None
            };

            if pum_rl.get() {
                let rt = unsafe { reverse_text(st) };
                unsafe { self.emit(style, rt, width, attrs.as_deref(), attr, next_isempty) };
                unsafe { xfree(rt.cast()) };
            } else {
                unsafe { self.emit(style, st, width, attrs.as_deref(), attr, next_isempty) };
            }
            unsafe { xfree(st.cast()) };

            if unsafe { *p } != b'\t' as c_char {
                break;
            }

            // A Tab shows as two spaces, and the walk starts a new run.
            if pum_rl.get() {
                unsafe { grid_line_puts(self.grid_col - 1, c"  ".as_ptr(), 2, attr) };
                self.grid_col -= 2;
            } else {
                unsafe { grid_line_puts(self.grid_col, c"  ".as_ptr(), 2, attr) };
                self.grid_col += 2;
            }
            self.totwidth += 2;
            run_start = ::core::ptr::null_mut();
            width = 0;
            p = unsafe { p.offset(utfc_ptr2len(p) as isize) };
        }
    }
}

/// Draw one row of the menu: its item, the truncation marker, the scrollbar.
///
/// # Safety
/// No line batch may be in progress; `pum_grid` must be allocated for
/// `pum_height` rows.
unsafe fn pum_draw_row(style: &RowStyle, i: c_int, grid_row: c_int) {
    // SAFETY: the item indices come from `pum_first`/`pum_height`, which
    // `pum_redraw` has already clamped to the array.
    let win = curwin.get();
    let idx = i + pum_first.get();
    let selected = idx == pum_selected.get();
    let mut row = PumRow {
        idx,
        hlfs: if selected {
            [HLF_PSI as hlf_T, HLF_PSK as hlf_T, HLF_PSX as hlf_T]
        } else {
            [HLF_PNI as hlf_T, HLF_PNK as hlf_T, HLF_PNX as hlf_T]
        },
        grid_col: style.col_off,
        totwidth: 0,
        need_trunc: false,
        orig_attr: -1,
    };
    let trunc_attr = unsafe { win_hl_attr(win, if selected { HLF_PSI } else { HLF_PNI }) };

    unsafe { screengrid_line_start(pum_grid_ref(), grid_row, 0) };

    if style.extra_space {
        let attr = unsafe {
            hl_combine_attr(
                win_hl_attr(win, HLF_PNI),
                win_hl_attr(win, row.hlfs[0] as c_int),
            )
        };
        let col = if pum_rl.get() {
            style.col_off + 1
        } else {
            style.col_off - 1
        };
        unsafe { grid_line_puts(col, c" ".as_ptr(), 1, attr) };
    }

    let order = pum_align_order();
    let widths = [
        pum_base_width.get(),
        pum_kind_width.get(),
        pum_extra_width.get(),
    ];
    // Where the second and third columns start: the first column's width,
    // plus the separator the layout reserved.
    let basic_width = widths[order[0] as usize];
    let last_isabbr = order[2] == CPT_ABBR as c_int;

    for j in 0..3 {
        let item_type = order[j];
        let next_isempty = j + 1 >= 3 || unsafe { pum_get_item(idx, order[j + 1]) }.is_null();
        unsafe { row.put_column(style, item_type, next_isempty) };

        let n = if j > 0 {
            widths[order[1] as usize] + c_int::from(!last_isabbr)
        } else {
            c_int::from(order[j] == CPT_ABBR as c_int)
        };

        // Stop when there is nothing more to display.
        if j == 2
            || (next_isempty
                && (j == 1 || (j == 0 && unsafe { pum_get_item(idx, order[j + 2]) }.is_null())))
            || basic_width + n >= pum_width.get()
        {
            break;
        }

        // Pad out to where the next column starts.
        if pum_rl.get() {
            grid_line_fill(
                style.col_off - basic_width - n + 1,
                row.grid_col + 1,
                schar_from_ascii(b' '),
                row.orig_attr,
            );
            row.grid_col = style.col_off - basic_width - n;
        } else {
            grid_line_fill(
                row.grid_col,
                style.col_off + basic_width + n,
                schar_from_ascii(b' '),
                row.orig_attr,
            );
            row.grid_col = style.col_off + basic_width + n;
        }
        row.totwidth = basic_width + n;
    }

    // Blank the rest of the row, then overwrite its far cell with the
    // truncation marker if anything was cut. The marker is written
    // straight into the line buffer because it replaces a cell
    // `grid_line_fill` has already put there -- so the handle is taken
    // here but each slice of it is borrowed for one write only, with the
    // `grid_line_fill` calls between them.
    let mut line = linebuf();
    if pum_rl.get() {
        let lcol = style.col_off - pum_width.get() + 1;
        grid_line_fill(
            lcol,
            row.grid_col + 1,
            schar_from_ascii(b' '),
            row.orig_attr,
        );
        if row.need_trunc {
            line.chars_mut()[lcol as usize] = if style.fcs_trunc != 0 {
                style.fcs_trunc
            } else {
                schar_from_ascii(b'<')
            };
            line.attrs_mut()[lcol as usize] = trunc_attr as sattr_T;
            // The marker may have replaced the left half of a wide
            // character; give the orphaned right half a space.
            if pum_width.get() > 1 && line.chars()[lcol as usize + 1] == 0 {
                line.chars_mut()[lcol as usize + 1] = schar_from_ascii(b' ');
            }
        }
    } else {
        let rcol = style.col_off + pum_width.get();
        grid_line_fill(row.grid_col, rcol, schar_from_ascii(b' '), row.orig_attr);
        if row.need_trunc {
            if pum_width.get() > 1 && line.chars()[(rcol - 1) as usize] == 0 {
                line.chars_mut()[(rcol - 2) as usize] = schar_from_ascii(b' ');
            }
            line.chars_mut()[(rcol - 1) as usize] = if style.fcs_trunc != 0 {
                style.fcs_trunc
            } else {
                schar_from_ascii(b'>')
            };
            line.attrs_mut()[(rcol - 1) as usize] = trunc_attr as sattr_T;
        }
    }

    if pum_scrollbar.get() > 0 {
        let thumb = i >= style.thumb_pos && i < style.thumb_pos + style.thumb_height;
        let scrollbar_col = style.col_off
            + if pum_rl.get() {
                -pum_width.get()
            } else {
                pum_width.get()
            };
        let (sc, attr) = match (thumb, style.border_scroll) {
            (true, _) => (schar_from_ascii(b' '), style.attr_thumb),
            (false, Some(border)) => border,
            (false, None) => (schar_from_ascii(b' '), style.attr_scroll),
        };
        grid_line_put_schar(scrollbar_col, sc, attr);
    }
    unsafe { grid_line_flush() };
}

/// Redraw the popup menu, using `pum_first` and `pum_selected`.
///
/// # Safety
/// The menu must be displayed (`pum_display` or `pum_show_popupmenu` has run
/// the placement) and no line batch may be in progress.
pub unsafe fn pum_redraw() {
    let mut grid = pum_grid_ref();
    // SAFETY: the placement functions have filled the state cells and the
    // item array is live.
    let win = curwin.get();

    // Room for one padding cell beside the text, when there is any.
    let mut grid_width = pum_width.get();
    let mut col_off = 0;
    let mut extra_space = false;
    if pum_rl.get() {
        col_off = pum_width.get() - 1;
        debug_assert!(State.get() & MODE_CMDLINE == 0, "!(State & MODE_CMDLINE)");
        let win_end_col = unsafe { (*win).w_wincol } + unsafe { (*win).w_width };
        if pum_col.get() < win_end_col - 1 {
            grid_width += 1;
            extra_space = true;
        }
    } else if pum_col.get() > 0 {
        grid_width += 1;
        col_off = 1;
        extra_space = true;
    }

    let mut config = WIN_CONFIG_INIT;
    // SAFETY: `config` is this frame's own.
    let resolved = unsafe { resolve_border(&mut config) };
    let Some(border) = resolved else {
        return; // 'pumborder' did not parse; the message is already out
    };

    // A scrollbar drawn by the menu itself needs a column of its own; one
    // drawn into a box border reuses the border's.
    if pum_scrollbar.get() > 0 && (!config.border || config.shadow) {
        grid_width += 1;
        if pum_rl.get() {
            col_off += 1;
        }
    }

    grid.blending = p_pb.get() > 0 || config.shadow;
    grid_assign_handle(&mut grid);

    pum_left_col.set(pum_col.get() - col_off);
    pum_right_col.set(pum_left_col.get() + grid_width);
    let moved = unsafe {
        ui_comp_put_grid(
            grid.raw(),
            pum_row.get(),
            pum_left_col.get(),
            pum_height.get() + border.width,
            grid_width + border.width,
            false,
            true,
        )
    };
    let invalid_grid = moved || pum_invalid.get();
    pum_invalid.set(false);
    must_redraw_pum.set(false);

    let (rows, cols) = (pum_height.get() + border.width, grid_width + border.width);
    if !grid.is_allocated() || grid.rows != rows || grid.cols != cols {
        grid_alloc(&mut grid, rows, cols, !invalid_grid, false);
        ui_call_grid_resize(
            grid.handle as Integer,
            grid.cols as Integer,
            grid.rows as Integer,
        );
    } else if invalid_grid {
        grid.invalidate();
    }
    if ui_has(kUIMultigrid) {
        unsafe { pum_send_float_pos() };
    }

    let mut grid_row = 0;
    if config.border {
        unsafe {
            grid_draw_border(
                grid.raw(),
                &raw mut config,
                ::core::ptr::null_mut(),
                0,
                ::core::ptr::null_mut(),
            )
        };
        if !config.shadow {
            grid_row += 1;
            col_off += 1;
        }
    }

    // Never display more than there is.
    let scroll_range = pum_size.get() - pum_height.get();
    pum_first.set(pum_first.get().min(scroll_range));

    let (mut thumb_pos, mut thumb_height) = (0, 1);
    if pum_scrollbar.get() != 0 {
        thumb_height = (pum_height.get() * pum_height.get() / pum_size.get()).max(1);
        thumb_pos =
            (pum_first.get() * (pum_height.get() - thumb_height) + scroll_range / 2) / scroll_range;
    }

    let style = RowStyle {
        col_off,
        extra_space,
        fcs_trunc: if pum_rl.get() {
            unsafe { (*win).w_p_fcs_chars.truncrl }
        } else {
            unsafe { (*win).w_p_fcs_chars.trunc }
        },
        attr_scroll: unsafe { win_hl_attr(win, HLF_PSB) },
        attr_thumb: unsafe { win_hl_attr(win, HLF_PST) },
        border_scroll: if border.width > 0 && !config.shadow {
            border.scrollbar
        } else {
            None
        },
        thumb_pos,
        thumb_height,
    };

    for i in 0..pum_height.get() {
        unsafe { pum_draw_row(&style, i, grid_row) };
        grid_row += 1;
    }
}
