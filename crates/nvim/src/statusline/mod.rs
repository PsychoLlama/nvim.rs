//! The status line, the tab line, the winbar, the ruler and the status
//! column.
//!
//! Carved by what is being drawn:
//!
//! | child | what |
//! | --- | --- |
//! | [`status`] | the default status line and `stl_connected()` |
//! | [`custom`] | `'statusline'`/`'winbar'`/`'rulerformat'` rendering |
//! | [`tabline`] | `draw_tabline()` and its `ext_tabline` form |
//! | [`stl`] | `build_stl_str_hl()`, the `%` format language itself |
//!
//! What stays here is what the four share: the `StlItem` kinds, because
//! the expander and its callers both name them (the format language's own
//! alphabet is [`crate::types::StlOpt`]); [`build_statuscol_str`]
//! (`'statuscolumn'`, which is the same language with a different item set)
//! and the small entry points -- [`fillchar_status_of`] and
//! [`redraw_custom_statusline`] -- the drawing layer calls in; and the
//! pieces the three children share:
//!
//! * [`StlJob`] gathers everything one call into [`build_stl_str_hl`] is
//!   parameterised by, so the callers each spell one `run()`, which answers
//!   the text and the records the caller asked for, owned.
//! * The click-definition functions keep a window's (or the tab line's)
//!   `%@Func@` click definitions: one entry per screen cell, sized to the
//!   line's width.
//! * [`GridRef`] is the screen grid a line is painted on, and the `paint_*`
//!   family below is the one-line batch API with its "a batch is in
//!   progress" obligation discharged once here.
//!
//! Original: `src/nvim/statusline.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

pub(crate) mod state;
use crate::statusline::state::tab_page_click_defs;
use crate::types::CAR;
use crate::types::NL;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

use crate::charset::strnsize;
use crate::drawscreen::redrawing;
use crate::eval::vars::set_vim_var_nr;
use crate::global_cell::GlobalCell;
use crate::grid::{
    GridRef, grid_adjust, grid_line_fill, grid_line_flush, grid_line_put_schar, grid_line_puts,
    grid_line_start, screengrid_line_start,
};
pub(crate) use crate::highlight::namespace::hl_attr;
use crate::highlight::{hl_combine_attr, win_hl_attr};
use crate::highlight_group::{HLF_S, HLF_SNC};
use crate::options::kOptStatuscolumn;
use crate::optionstr::OptStringRef;
use crate::types::{
    AlignTextPos, ApiDict, Array, ClickLine, GridView, Hlf, LineNr, MAXPATHL, Object, OptIndex,
    OptionSetFlags, ScreenChar, SignTextAttrs, StatusCol, StlClickDefinition,
    StlClickDefinition_type_0, StlClickRecord, VarNumber, Vv, WinSplit, WinStyle,
};
use crate::window::global_stl_height;
use crate::winlayer::Win;

// The carve of the transpiled module; see each child's docs.
mod custom;
mod status;
mod stl;
mod tabline;

pub use self::custom::*;
pub use self::status::*;
pub use self::stl::*;
pub use self::tabline::*;

pub const kStlClickFuncRun: StlClickDefinition_type_0 = 3;
pub const kStlClickTabClose: StlClickDefinition_type_0 = 2;
pub const kStlClickTabSwitch: StlClickDefinition_type_0 = 1;
pub const kStlClickDisabled: StlClickDefinition_type_0 = 0;
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitLeft: WinSplit = 0;
/// Most sign columns `'signcolumn'` will ever ask for.
pub const SIGN_SHOW_MAX: ::core::ffi::c_int = 9;
pub type NumberBase = ::core::ffi::c_uint;
pub const kNumBaseHexadecimal: NumberBase = 16;
pub const kNumBaseDecimal: NumberBase = 10;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Array = Array::EMPTY;
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const FR_COL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const EOL_MAC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const MAX_NUMBERWIDTH: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const SID_ERROR: ::core::ffi::c_int = -5 as ::core::ffi::c_int;
pub const MAX_STL_EVAL_DEPTH: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
pub const RULER_BUF_LEN: ::core::ffi::c_int = 70 as ::core::ffi::c_int;
pub const TMPLEN: ::core::ffi::c_int = 70 as ::core::ffi::c_int;

// ---------------------------------------------------------------------------
// The format expander, wrapped
// ---------------------------------------------------------------------------

/// One expansion of a `%`-format: everything `build_stl_str_hl()` takes.
pub(crate) struct StlJob<'a> {
    /// The window the items describe. For `'tabline'` this is `curwin`.
    pub win: Win,
    /// The format itself, the caller's own copy: expanding a format can run
    /// a function that `:set`s the very option the format came from.
    pub fmt: Vec<u8>,
    /// Which option the format came from, and in which scope. `kOptInvalid`
    /// means "no option", which is also what switches the sandbox off --
    /// `nvim_eval_statusline()` can therefore never reach it.
    pub opt: (OptIndex, OptionSetFlags),
    /// What to pad with, and how many cells there are to fill.
    pub fillchar: ScreenChar,
    pub maxwidth: c_int,
    /// The size of the buffer the expansion works in, which is where the
    /// text is cut: `MAXPATHL` for everything drawn.
    pub room: usize,
    /// Whether to record the highlight runs and the click records.
    pub want_hl: bool,
    pub want_clicks: bool,
    /// The `'statuscolumn'` line, when that is what is being built.
    pub stc: Option<StcLine<'a>>,
}

impl StlJob<'_> {
    /// Expand the format. This re-enters the editor -- a `%{}` item
    /// evaluates arbitrary Vim script -- so nothing may be held across it.
    pub(crate) fn run(self) -> StlOutput {
        let (opt_idx, opt_scope) = self.opt;
        let wants = StlWants {
            hl: self.want_hl,
            clicks: self.want_clicks,
            stc: self.stc,
        };
        let from = FmtSource { opt_idx, opt_scope };
        build_stl_str_hl(
            self.win,
            self.room,
            self.fmt,
            from,
            self.fillchar,
            self.maxwidth,
            wants,
        )
    }
}

// ---------------------------------------------------------------------------
// The click definitions
// ---------------------------------------------------------------------------

/// A cell nobody claimed.
pub(crate) const NO_CLICK: StlClickDefinition = StlClickDefinition {
    type_0: kStlClickDisabled,
    tabnr: 0,
    func: 0,
};

/// Blank every definition, and forget the handler names they referred to.
pub(crate) fn clear_click_defs(line: &mut ClickLine) {
    line.cells.fill(NO_CLICK);
    line.funcs.clear();
}

/// Make room for `width` cells, replacing the definitions when there are
/// fewer: a cleared line's are blank, and so are the new ones.
pub(crate) fn reserve_click_defs(line: &mut ClickLine, width: c_int) {
    let width = usize::try_from(width).unwrap_or(0);
    if line.cells.len() < width {
        line.cells = vec![NO_CLICK; width];
    }
}

/// Claim `cols` for tab page `tabnr`, the tab line's own way of filling its
/// definitions.
///
/// An empty or backwards range claims nothing, which is what upstream's
/// `while (scol < col)` does when the line is already full.
pub(crate) fn set_click_defs(
    line: &mut ClickLine,
    cols: core::ops::Range<c_int>,
    kind: StlClickDefinition_type_0,
    tabnr: c_int,
) {
    if line.cells.is_empty() || cols.start >= cols.end {
        return;
    }
    line.cells[cols.start as usize..cols.end as usize].fill(StlClickDefinition {
        type_0: kind,
        tabnr,
        func: 0,
    });
}

/// Spread `recs` over the first `width` cells.
///
/// `text` is the expanded line the records name bytes of; each record claims
/// everything from where the previous one started up to its own start,
/// measured in screen cells rather than bytes. A handler name is kept only
/// when its region claims at least one cell.
///
/// `tabline` keeps the tab-page kinds; everywhere else only a click
/// *function* is honoured, because nothing else has a tab to name.
pub(crate) fn fill_click_defs(
    line: &mut ClickLine,
    recs: Vec<StlClickRecord>,
    text: &[u8],
    width: c_int,
    tabline: bool,
) {
    if line.cells.is_empty() {
        return;
    }
    let mut col = 0;
    let mut len = 0;
    let mut from = 0;
    let mut cur = NO_CLICK;
    let mut cur_name = None;
    // The slice bounds below are C's `assert(len <= width)` made real:
    // upstream writes past the array when a record claims more cells than
    // the line has, and the reserve before the expansion guarantees `width`
    // of them.
    for rec in recs {
        let bytes = rec.start as isize - from as isize;
        len += strnsize(text.get(from..).unwrap_or_default(), bytes as c_int);
        debug_assert!(len <= width, "len <= width");
        if col < len {
            claim(line, col..len, cur, cur_name.take());
            col = len;
        }
        // Otherwise this run is zero cells wide, and the name it carries is
        // dropped with it.
        from = rec.start;
        (cur, cur_name) = (rec.def, rec.func);
        if !tabline && !(cur.type_0 == kStlClickDisabled || cur.type_0 == kStlClickFuncRun) {
            cur.type_0 = kStlClickDisabled;
        }
    }
    if col < width {
        claim(line, col..width, cur, cur_name);
    }
}

/// Fill `cols` with `def`, naming `name` as its handler when there is one.
fn claim(
    line: &mut ClickLine,
    cols: core::ops::Range<c_int>,
    mut def: StlClickDefinition,
    name: Option<CString>,
) {
    if let Some(name) = name {
        line.funcs.push(name);
        def.func = u32::try_from(line.funcs.len()).unwrap_or(0);
    }
    line.cells[cols.start as usize..cols.end as usize].fill(def);
}

/// Run `f` over the tab page line's definitions, which the screen resize
/// sizes to `Columns`.
pub(crate) fn with_tab_click_defs<R>(f: impl FnOnce(&mut ClickLine) -> R) -> R {
    tab_page_click_defs.with_mut(f)
}

// ---------------------------------------------------------------------------
// Painting
// ---------------------------------------------------------------------------

/// Begin a batch on `row` of a window-relative view.
///
/// The batch API underneath is a global: this (or [`screengrid_line_start`]
/// for the callers that already hold the grid) opens the one batch there is
/// and [`paint_flush`] closes it. Everything between the two addresses cells
/// of that line by column, which is why the `paint_*` family is free
/// functions rather than methods on the grid.
pub(crate) fn view_line_start(view: GridView, row: c_int) {
    grid_line_start(view, row);
}

/// Put one glyph in the open batch.
pub(crate) fn paint_schar(col: c_int, sc: ScreenChar, attr: c_int) {
    grid_line_put_schar(col, sc, attr);
}

/// Put `text` in the open batch, answering how many cells it took.
pub(crate) fn paint_text(col: c_int, text: &[u8], attr: c_int) -> c_int {
    grid_line_puts(col, text, attr)
}

/// Fill `col..end_col` with one glyph, answering where it stopped.
pub(crate) fn paint_fill(col: c_int, end_col: c_int, sc: ScreenChar, attr: c_int) -> c_int {
    grid_line_fill(col, end_col, sc, attr)
}

/// Close the batch, sending the line to the UI.
pub(crate) fn paint_flush() {
    grid_line_flush();
}

/// Whether the screen may be redrawn right now -- C's `redrawing()`, which
/// only reads globals.
pub(crate) fn is_redrawing() -> bool {
    redrawing()
}

/// The attribute `group` has in `win`, i.e. C's `win_hl_attr` -- [`hl_attr`]
/// unless the window carries a `'winhighlight'` override.
pub(crate) fn win_hl(win: Win, group: c_int) -> c_int {
    win_hl_attr(win, group)
}

/// Two attributes layered, i.e. C's `hl_combine_attr`.
pub(crate) fn combine_attr(under: c_int, over: c_int) -> c_int {
    hl_combine_attr(under, over)
}

/// Whether `'laststatus'` puts one status line at the bottom of the screen
/// instead of one per window.
pub(crate) fn stl_is_global() -> bool {
    global_stl_height() > 0
}

/// C's `PUT_C`: put `key` in a dictionary that was sized up front.
///
/// Safe because every caller in this module sizes the dictionary from the
/// same expression that decides how many keys it puts; the debug assertion
/// inside catches a mismatch.
pub(crate) fn put(dict: &mut ApiDict, key: &'static CStr, value: Object) {
    dict.insert(key, value);
}

/// C's `ADD_C`. See [`put`].
pub(crate) fn push(array: &mut Array, value: Object) {
    array.push(value);
}

// ---------------------------------------------------------------------------
// The entry points the drawing layer calls in
// ---------------------------------------------------------------------------

/// The fill character and highlight group of `win`'s status line.
pub(crate) fn fillchar_status_of(win: Win) -> (Hlf, ScreenChar) {
    if win.is_current() {
        (HLF_S, win.w_p_fcs_chars.stl)
    } else {
        (HLF_SNC, win.w_p_fcs_chars.stlnc)
    }
}

/// Redraw `window`'s status line from `'statusline'`.
pub fn redraw_custom_statusline(window: Win) {
    static ENTERED: GlobalCell<bool> = GlobalCell::new(false);
    // A `'statusline'` expression that triggers a redraw gets here again.
    if ENTERED.get() {
        return;
    }
    ENTERED.set(true);
    win_redr_custom(Some(window), false, false, false);
    ENTERED.set(false);
}

/// Build the `'statuscolumn'` string for line `lnum`.
///
/// `relnum` of -1 means the caller has already set `v:lnum`/`v:relnum` and
/// only wants the text -- which is also what keeps the click definitions
/// from being rebuilt for every line of the window. `stcp` is this line's
/// status-column state, whose `fold_vcol` the expansion writes, and `signs`
/// the line's signs. The answer's `hl` is the highlight runs over its text.
pub(crate) fn build_statuscol_str(
    window: Win,
    lnum: LineNr,
    relnum: LineNr,
    stcp: &mut StatusCol,
    signs: &[SignTextAttrs],
) -> StlOutput {
    let mut win = window;
    // Only update the click definitions once per window per redraw, and not
    // at all while the column is empty -- it is redrawn again once it is not.
    let fillclick = relnum >= 0 && stcp.width > 0 && lnum == win.w_topline;

    if relnum >= 0 {
        set_vim_var_nr(Vv::Lnum, lnum as VarNumber);
        set_vim_var_nr(Vv::Relnum, relnum as VarNumber);
    }

    let job = StlJob {
        win,
        fmt: win.w_onebuf_opt.wo_stc.bytes().to_vec(),
        opt: (kOptStatuscolumn, OptionSetFlags::LOCAL),
        fillchar: 0,
        maxwidth: stcp.width,
        room: MAXPATHL as usize,
        want_hl: true,
        want_clicks: fillclick,
        stc: Some(StcLine {
            foldinfo: stcp.foldinfo,
            lnum: stcp.lnum,
            sign_cul_id: stcp.sign_cul_id,
            signs,
            fold_vcol: core::cell::Cell::new(stcp.fold_vcol),
        }),
    };
    // The expander re-enters the editor; nothing is held across it.
    let mut built = job.run();
    if let Some(fold_vcol) = built.fold_vcol {
        stcp.fold_vcol = fold_vcol;
    }

    if fillclick {
        let defs = &mut win.w_click_defs.statuscol;
        clear_click_defs(defs);
        reserve_click_defs(defs, built.width);
        let recs = core::mem::take(&mut built.clicks);
        fill_click_defs(defs, recs, &built.text, built.width, false);
    }
    built
}
