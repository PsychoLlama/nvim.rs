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
//! What stays here is what the four share. The `STL_*` item alphabet the
//! format language is written in and the `stl_item_t` kinds, because the
//! expander and its callers both name them; [`build_statuscol_str`]
//! (`'statuscolumn'`, which is the same language with a different item set)
//! and the two small entry points -- [`fillchar_status`] and
//! [`redraw_custom_statusline`] -- the drawing layer calls in; and four
//! wrappers that are what makes the three children safe code:
//!
//! * [`StlJob`] gathers everything one call into [`build_stl_str_hl`] is
//!   parameterised by, so the four callers each spell one `run()` instead
//!   of a twelve-argument call with four pointer-shaped out-parameters.
//!   [`HlRuns`] and [`ClickRecs`] wrap the two NUL-terminated arrays it
//!   answers through, and iterating one is then safe.
//! * [`ClickArena`] is a window's (or the tab line's) `%@Func@` click
//!   definitions: one entry per screen cell, sized to the line's width.
//!   Allocating, clearing and filling it are the three C entry points, of
//!   which the first two are still called from `window.rs` by pointer.
//! * [`Canvas`] is the screen grid a line is painted on, and the `paint_*`
//!   family below is the one-line batch API with its "a batch is in
//!   progress" obligation discharged once here.
//! * [`with_name_buff`] hands out the shared `NameBuff` scratch buffer,
//!   which `get_trans_bufname()` fills and both the tab line and the
//!   `ext_tabline` event read back.
//!
//! Original: `src/nvim/statusline.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::{ptr, slice};

use crate::api::private::helpers::{array_add, dict_put};
use crate::charset::vim_strnsize;
use crate::drawscreen::redrawing;
use crate::eval::vars::set_vim_var_nr;
use crate::global_cell::GlobalCell;
use crate::grid::{
    grid_adjust, grid_line_fill, grid_line_flush, grid_line_put_schar, grid_line_puts,
    grid_line_start, screengrid_line_start,
};
use crate::highlight::{hl_combine_attr, win_hl_attr};
use crate::highlight_group::{HLF_S, HLF_SNC};
use crate::main::{NameBuff, hl_attr_active};
use crate::memory::{xcalloc, xfree, xstrdup};
use crate::options::kOptStatuscolumn;
use crate::types::{
    AlignTextPos, Array, Dict, GridView, MAXPATHL, Object, OptIndex, OptValType, OptionSetFlags,
    ScreenGrid, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_13, StlClickRecord,
    StlFlag, VV_LNUM, VV_RELNUM, WinSplit, WinStyle, hlf_T, linenr_T, schar_T, size_t, statuscol_T,
    stl_hlrec_t, varnumber_T, win_T,
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

pub const kStlClickFuncRun: C2Rust_Unnamed_13 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_13 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_13 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_13 = 0;
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitLeft: WinSplit = 0;
pub const kOptValTypeString: OptValType = 2;
/// Most sign columns `'signcolumn'` will ever ask for.
pub const SIGN_SHOW_MAX: ::core::ffi::c_int = 9;
pub const STL_CLICK_FUNC: StlFlag = 64;
pub const STL_TABCLOSENR: StlFlag = 88;
pub const STL_TABPAGENR: StlFlag = 84;
pub const STL_HIGHLIGHT_COMB: StlFlag = 36;
pub const STL_HIGHLIGHT: StlFlag = 35;
pub const STL_USER_HL: StlFlag = 42;
pub const STL_TRUNCMARK: StlFlag = 60;
pub const STL_SEPARATE: StlFlag = 61;
pub const STL_VIM_EXPR: StlFlag = 123;
pub const STL_SIGNCOL: StlFlag = 115;
pub const STL_FOLDCOL: StlFlag = 67;
pub const STL_SHOWCMD: StlFlag = 83;
pub const STL_PAGENUM: StlFlag = 78;
pub const STL_ARGLISTSTAT: StlFlag = 97;
pub const STL_ALTPERCENT: StlFlag = 80;
pub const STL_PERCENTAGE: StlFlag = 112;
pub const STL_QUICKFIX: StlFlag = 113;
pub const STL_MODIFIED_ALT: StlFlag = 77;
pub const STL_MODIFIED: StlFlag = 109;
pub const STL_PREVIEWFLAG_ALT: StlFlag = 87;
pub const STL_PREVIEWFLAG: StlFlag = 119;
pub const STL_FILETYPE_ALT: StlFlag = 89;
pub const STL_FILETYPE: StlFlag = 121;
pub const STL_HELPFLAG_ALT: StlFlag = 72;
pub const STL_HELPFLAG: StlFlag = 104;
pub const STL_ROFLAG_ALT: StlFlag = 82;
pub const STL_ROFLAG: StlFlag = 114;
pub const STL_BYTEVAL_X: StlFlag = 66;
pub const STL_BYTEVAL: StlFlag = 98;
pub const STL_OFFSET_X: StlFlag = 79;
pub const STL_OFFSET: StlFlag = 111;
pub const STL_KEYMAP: StlFlag = 107;
pub const STL_BUFNO: StlFlag = 110;
pub const STL_NUMLINES: StlFlag = 76;
pub const STL_LINE: StlFlag = 108;
pub const STL_VIRTCOL_ALT: StlFlag = 86;
pub const STL_VIRTCOL: StlFlag = 118;
pub const STL_COLUMN: StlFlag = 99;
pub const STL_FILENAME: StlFlag = 116;
pub const STL_FULLPATH: StlFlag = 70;
pub const STL_FILEPATH: StlFlag = 102;
#[derive(Copy, Clone)]
pub struct stl_item {
    pub start: *mut ::core::ffi::c_char,
    pub cmd: *mut ::core::ffi::c_char,
    pub minwid: ::core::ffi::c_int,
    pub maxwid: ::core::ffi::c_int,
    pub type_0: C2Rust_Unnamed_15,
}
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const Trunc: C2Rust_Unnamed_15 = 10;
pub const ClickFunc: C2Rust_Unnamed_15 = 9;
pub const TabPage: C2Rust_Unnamed_15 = 8;
pub const HighlightFold: C2Rust_Unnamed_15 = 7;
pub const HighlightSign: C2Rust_Unnamed_15 = 6;
pub const HighlightCombining: C2Rust_Unnamed_15 = 5;
pub const Highlight: C2Rust_Unnamed_15 = 4;
pub const Separate: C2Rust_Unnamed_15 = 3;
pub const Group: C2Rust_Unnamed_15 = 2;
pub const Empty: C2Rust_Unnamed_15 = 1;
pub const Normal: C2Rust_Unnamed_15 = 0;
pub type stl_item_t = stl_item;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub type NumberBase = ::core::ffi::c_uint;
pub const kNumBaseHexadecimal: NumberBase = 16;
pub const kNumBaseDecimal: NumberBase = 10;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
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

/// The format string one expansion runs over.
///
/// Every drawing caller hands over a private *copy*: expanding a format can
/// run a function that `:set`s the very option the format came from, which
/// frees the string out from under the expander. `nvim_eval_statusline()`
/// is the exception -- its format is an API argument nobody can reach.
pub(crate) struct Fmt {
    text: *mut c_char,
    owned: bool,
}

impl Fmt {
    /// A heap copy of `src`, freed when the value is dropped.
    ///
    /// # Safety
    /// `src` must be a NUL-terminated string.
    pub(crate) unsafe fn copy_of(src: *const c_char) -> Self {
        Fmt {
            // SAFETY: the caller's promise.
            text: unsafe { xstrdup(src) },
            owned: true,
        }
    }

    /// A format the caller already owns a stable buffer for.
    ///
    /// # Safety
    /// `text` must be a NUL-terminated string that outlives the expansion.
    pub(crate) const unsafe fn borrowed(text: *mut c_char) -> Self {
        Fmt { text, owned: false }
    }
}

impl Drop for Fmt {
    fn drop(&mut self) {
        if self.owned {
            // SAFETY: the copy this value made, freed exactly once.
            unsafe { xfree(self.text.cast()) };
        }
    }
}

/// The highlight runs `build_stl_str_hl()` recorded: one per group change,
/// terminated by a run with a null `start`.
#[derive(Clone, Copy)]
pub(crate) struct HlRuns(*mut stl_hlrec_t);

impl HlRuns {
    /// The runs, in order. The terminator is not one of them.
    pub(crate) fn iter(self) -> impl Iterator<Item = stl_hlrec_t> {
        (0..).map_while(move |i| {
            // SAFETY: the expander wrote a terminating run, so the walk
            // stops inside the array it allocated.
            let run = unsafe { *self.0.add(i) };
            (!run.start.is_null()).then_some(run)
        })
    }

    /// Where the first run starts, when it starts anywhere.
    pub(crate) fn first_start(self) -> Option<*mut c_char> {
        // SAFETY: the array holds at least the terminator.
        let start = unsafe { (*self.0).start };
        (!start.is_null()).then_some(start)
    }
}

/// The `%@Func@` click records `build_stl_str_hl()` recorded, terminated by
/// a record with a null `start`.
#[derive(Clone, Copy)]
pub(crate) struct ClickRecs(*mut StlClickRecord);

impl ClickRecs {
    /// The records, in order. The terminator is not one of them.
    pub(crate) fn iter(self) -> impl Iterator<Item = StlClickRecord> {
        (0..).map_while(move |i| {
            // SAFETY: as [`HlRuns::iter`].
            let rec = unsafe { *self.0.add(i) };
            (!rec.start.is_null()).then_some(rec)
        })
    }
}

/// Where the highlight runs go.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum HlDest {
    /// Nowhere: the caller only wants the text.
    Discard,
    /// Back to the caller, in [`StlBuilt::hl`].
    Runs,
    /// Into the `'statuscolumn'` state's own `hlrec` field, which is where
    /// the column drawer reads them from. C passes `&stcp->hlrec` as the
    /// same out-parameter, so this is one destination and not two.
    StatusCol,
}

/// What one expansion answered.
pub(crate) struct StlBuilt {
    /// The width of the result in screen cells.
    pub width: c_int,
    /// The highlight runs, for [`HlDest::Runs`].
    pub hl: Option<HlRuns>,
    /// How many of them there are, which only `nvim_eval_statusline()` uses.
    pub hl_len: size_t,
    /// The click records, when [`StlJob::want_clicks`] asked for them.
    pub clicks: Option<ClickRecs>,
}

/// One expansion of a `%`-format: everything `build_stl_str_hl()` takes
/// apart from the buffer it writes into.
pub(crate) struct StlJob<'a> {
    /// The window the items describe. For `'tabline'` this is `curwin`.
    pub win: Win,
    /// The format itself.
    pub fmt: Fmt,
    /// Which option the format came from, and in which scope. `kOptInvalid`
    /// means "no option", which is also what switches the sandbox off --
    /// `nvim_eval_statusline()` can therefore never reach it.
    pub opt: (OptIndex, OptionSetFlags),
    /// What to pad with, and how many cells there are to fill.
    pub fillchar: schar_T,
    pub maxwidth: c_int,
    /// Where the highlight runs go, and whether to record click records.
    pub hl: HlDest,
    pub want_clicks: bool,
    /// The `'statuscolumn'` state, when that is what is being built.
    pub stcp: Option<&'a mut statuscol_T>,
}

impl StlJob<'_> {
    /// Expand the format into `out`.
    ///
    /// # Safety
    /// This re-enters the editor -- a `%{}` item evaluates arbitrary Vim
    /// script -- so nothing may be held across it, and `out` must not alias
    /// `NameBuff` (the expander uses it as scratch).
    pub(crate) unsafe fn run(self, out: &mut [c_char]) -> StlBuilt {
        let mut runs = ptr::null_mut();
        let mut hl_len: size_t = 0;
        let mut clicks = ptr::null_mut();
        let stcp = self.stcp.map_or(ptr::null_mut(), ptr::from_mut);
        let hltab = match self.hl {
            HlDest::Discard => ptr::null_mut(),
            HlDest::Runs => &raw mut runs,
            // SAFETY: `stcp` is the caller's live status-column state.
            HlDest::StatusCol => unsafe { &raw mut (*stcp).hlrec },
        };
        let clk = if self.want_clicks {
            &raw mut clicks
        } else {
            ptr::null_mut()
        };
        // The twelve arguments, in the order the C entry point takes them:
        // the window, the output buffer and its size, the format, the
        // option and its scope, the fill character, the width, and the four
        // out-parameters.
        let (w, o, n, f) = (self.win.raw(), out.as_mut_ptr(), out.len(), self.fmt.text);
        let ((oi, os), fc, mw, hll) = (self.opt, self.fillchar, self.maxwidth, &raw mut hl_len);
        // SAFETY: `out` is the caller's buffer with its own length, `fmt`
        // is NUL-terminated by [`Fmt`]'s constructors, and the remaining
        // out-parameters are locals of this frame.
        let width = unsafe { build_stl_str_hl(w, o, n, f, oi, os, fc, mw, hltab, hll, clk, stcp) };
        StlBuilt {
            width,
            hl: (self.hl == HlDest::Runs).then_some(HlRuns(runs)),
            hl_len,
            clicks: self.want_clicks.then_some(ClickRecs(clicks)),
        }
    }
}

// ---------------------------------------------------------------------------
// The click-definition arenas
// ---------------------------------------------------------------------------

/// A blank click definition: what a cell nobody claimed holds.
const NO_CLICK: StlClickDefinition = StlClickDefinition {
    type_0: kStlClickDisabled,
    tabnr: 0,
    func: ptr::null_mut(),
};

/// A `%@Func@` click-definition array: one entry per screen cell of the line
/// it describes.
///
/// The array outlives the redraw that filled it -- `jump_to_mouse()` reads it
/// when the click arrives -- so it is a plain C allocation the window (or the
/// `tab_page_click_defs` global) owns, not a `Vec`. Each `func` string is
/// owned by the *run* of cells holding it, which is why clearing compares
/// each entry with its predecessor before freeing.
pub(crate) struct ClickArena {
    defs: *mut StlClickDefinition,
    size: size_t,
}

impl ClickArena {
    /// # Safety
    /// `defs` must be null or an allocation of `size` entries.
    pub(crate) unsafe fn new(defs: *mut StlClickDefinition, size: size_t) -> Self {
        ClickArena { defs, size }
    }

    /// The allocation and its size, to store back into whatever owns them.
    pub(crate) fn parts(&self) -> (*mut StlClickDefinition, size_t) {
        (self.defs, self.size)
    }

    /// The whole allocation, when there is one.
    fn entries(&mut self) -> Option<&mut [StlClickDefinition]> {
        (!self.defs.is_null()).then(|| {
            // SAFETY: the constructor's promise.
            unsafe { slice::from_raw_parts_mut(self.defs, self.size) }
        })
    }

    /// Free the strings the array holds and blank it.
    pub(crate) fn clear(&mut self) {
        let Some(defs) = self.entries() else { return };
        for i in 0..defs.len() {
            if i == 0 || defs[i].func != defs[i - 1].func {
                // SAFETY: one allocation per run of equal `func`s, made by
                // the expander and released exactly here.
                unsafe { xfree(defs[i].func.cast()) };
            }
        }
        // Blanking is a second pass on purpose: the comparison above reads
        // the entry before this one.
        defs.fill(NO_CLICK);
    }

    /// Make room for `width` cells, growing the allocation if it is smaller.
    pub(crate) fn reserve(&mut self, width: c_int) {
        if self.size >= width as size_t {
            return;
        }
        // SAFETY: the constructor's promise -- this allocation is ours.
        unsafe { xfree(self.defs.cast()) };
        self.size = width as size_t;
        // SAFETY: a fresh zeroed allocation of the size just recorded.
        self.defs = unsafe { xcalloc(self.size, size_of::<StlClickDefinition>()) }.cast();
    }

    /// Claim `cols` for tab page `tabnr`, the tab line's own way of filling
    /// the arena: no strings are involved, so nothing has to be freed.
    ///
    /// An empty or backwards range claims nothing, which is what upstream's
    /// `while (scol < col)` does when the line is already full.
    pub(crate) fn set(
        &mut self,
        cols: core::ops::Range<c_int>,
        kind: C2Rust_Unnamed_13,
        tabnr: c_int,
    ) {
        let Some(defs) = self.entries() else { return };
        if cols.start >= cols.end {
            return;
        }
        defs[cols.start as usize..cols.end as usize].fill(StlClickDefinition {
            type_0: kind,
            tabnr,
            func: ptr::null_mut(),
        });
    }

    /// Spread `recs` over the first `width` cells.
    ///
    /// `buf` is the expanded line the records point into; each record claims
    /// everything from where the previous one started up to its own start,
    /// measured in screen cells rather than bytes.
    ///
    /// `tabline` keeps the tab-page kinds; everywhere else only a click
    /// *function* is honoured, because nothing else has a tab to name.
    pub(crate) fn fill(
        &mut self,
        recs: ClickRecs,
        buf: *const c_char,
        width: c_int,
        tabline: bool,
    ) {
        let Some(defs) = self.entries() else { return };
        let mut col = 0;
        let mut len = 0;
        let mut from = buf;
        let mut cur = NO_CLICK;
        // The slice bounds below are C's `assert(len <= width)` made real:
        // upstream writes past the array when a record claims more cells
        // than the line has, and the reserve above guarantees `width` of
        // them.
        for rec in recs.iter() {
            // SAFETY: `from` and `rec.start` are positions in the expanded
            // line, `from` at or before `rec.start`.
            len += unsafe { vim_strnsize(from, rec.start.offset_from(from) as c_int) };
            debug_assert!(len <= width, "len <= width");
            if col < len {
                defs[col as usize..len as usize].fill(cur);
                col = len;
            } else {
                // Nothing to claim: this run is zero cells wide, so the
                // string it carries has no owner.
                // SAFETY: as [`ClickArena::clear`].
                unsafe { xfree(cur.func.cast()) };
            }
            from = rec.start;
            cur = rec.def;
            if !tabline && !(cur.type_0 == kStlClickDisabled || cur.type_0 == kStlClickFuncRun) {
                cur.type_0 = kStlClickDisabled;
            }
        }
        if col < width {
            defs[col as usize..width as usize].fill(cur);
        } else {
            // SAFETY: as above.
            unsafe { xfree(cur.func.cast()) };
        }
    }
}

/// C's `stl_clear_click_defs()`, for the callers in `window.rs` that hold
/// the array and its size as two separate fields.
///
/// # Safety
/// `click_defs` must be null or an allocation of `click_defs_size` entries.
pub unsafe fn stl_clear_click_defs(click_defs: *mut StlClickDefinition, click_defs_size: size_t) {
    // SAFETY: the caller's promise.
    unsafe { ClickArena::new(click_defs, click_defs_size) }.clear();
}

/// C's `stl_alloc_click_defs()`.
///
/// # Safety
/// As [`stl_clear_click_defs`]; `size` must be the owner's own record.
pub unsafe fn stl_alloc_click_defs(
    cdp: *mut StlClickDefinition,
    width: ::core::ffi::c_int,
    size: *mut size_t,
) -> *mut StlClickDefinition {
    // SAFETY: the caller's promise -- `size` is a live `size_t`.
    let mut arena = unsafe { ClickArena::new(cdp, *size) };
    arena.reserve(width);
    let (defs, new_size) = arena.parts();
    // SAFETY: the caller's out-parameter.
    unsafe { *size = new_size };
    defs
}

// ---------------------------------------------------------------------------
// Painting
// ---------------------------------------------------------------------------

/// The screen grid one line of the status line, tab line, winbar or ruler is
/// painted on.
///
/// The batch API underneath is a global: [`Canvas::line_start`] opens the one
/// batch there is and [`paint_flush`] closes it. Everything between the two
/// addresses cells of that line by column, which is why the `paint_*` family
/// is free functions rather than methods.
#[derive(Clone, Copy)]
pub(crate) struct Canvas(*mut ScreenGrid);

impl Canvas {
    /// # Safety
    /// `grid` must stay live until the batch is flushed.
    pub(crate) const unsafe fn new(grid: *mut ScreenGrid) -> Self {
        Canvas(grid)
    }

    /// Resolve `view` to the grid it really draws on, folding the view's
    /// offsets into `row`/`col`.
    ///
    /// # Safety
    /// `win_grid_alloc()` must already have run for this view.
    pub(crate) unsafe fn adjust(view: *mut GridView, row: &mut c_int, col: &mut c_int) -> Self {
        // SAFETY: the caller's promise; the two are locals of the caller.
        Canvas(unsafe { grid_adjust(view, row, col) })
    }

    /// Begin a batch on `row`, whose column zero is `col` of the grid.
    ///
    /// # Safety
    /// No other batch may be in progress; one [`paint_flush`] must follow.
    pub(crate) unsafe fn line_start(self, row: c_int, col: c_int) {
        // SAFETY: the caller's promise, and the constructor's.
        unsafe { screengrid_line_start(self.0, row, col) };
    }
}

/// [`Canvas::line_start`] against a window-relative view.
///
/// # Safety
/// As [`Canvas::line_start`].
pub(crate) unsafe fn view_line_start(view: *mut GridView, row: c_int) {
    // SAFETY: the caller's promise.
    unsafe { grid_line_start(view, row) };
}

/// Put one glyph in the open batch.
pub(crate) fn paint_schar(col: c_int, sc: schar_T, attr: c_int) {
    // SAFETY: the batch is open for as long as a `Canvas` is being painted.
    unsafe { grid_line_put_schar(col, sc, attr) };
}

/// Put `text` in the open batch, answering how many cells it took.
pub(crate) fn paint_text(col: c_int, text: &[c_char], attr: c_int) -> c_int {
    // SAFETY: `text` holds its own length in readable bytes.
    unsafe { grid_line_puts(col, text.as_ptr(), text.len() as c_int, attr) }
}

/// [`paint_text`] up to the string's NUL rather than to a byte count.
///
/// Not the same call: with a length the expander measures each character
/// against what is left of the slice, without one it measures against the
/// terminator.
pub(crate) fn paint_cstr(col: c_int, text: &CStr, attr: c_int) -> c_int {
    // SAFETY: `text` is NUL-terminated.
    unsafe { grid_line_puts(col, text.as_ptr(), -1, attr) }
}

/// Fill `col..end_col` with one glyph, answering where it stopped.
pub(crate) fn paint_fill(col: c_int, end_col: c_int, sc: schar_T, attr: c_int) -> c_int {
    // SAFETY: as [`paint_schar`].
    unsafe { grid_line_fill(col, end_col, sc, attr) }
}

/// Close the batch, sending the line to the UI.
pub(crate) fn paint_flush() {
    // SAFETY: as [`paint_schar`].
    unsafe { grid_line_flush() };
}

/// Whether the screen may be redrawn right now -- C's `redrawing()`, which
/// only reads globals.
pub(crate) fn is_redrawing() -> bool {
    // SAFETY: reads `RedrawingDisabled`, `updating_screen` and `p_lz`.
    unsafe { redrawing() }
}

/// The first byte of an option string, which is what the "is it set?" tests
/// read.
///
/// Safe because every option of string type holds a NUL-terminated string
/// from the moment the option table is initialised; there is no window in
/// which one is null.
pub(crate) fn opt_first(s: *const c_char) -> c_char {
    // SAFETY: the invariant above.
    unsafe { *s }
}

/// Whether an option string is empty, i.e. C's `*wp->w_p_stl == NUL`.
pub(crate) fn opt_is_empty(s: *const c_char) -> bool {
    opt_first(s) == 0
}

/// The active attribute of highlight group `hlf`, i.e. C's `HL_ATTR`.
pub(crate) fn hl_attr(hlf: c_int) -> c_int {
    // SAFETY: the attribute table is built before the first redraw and is
    // indexed by every `HLF_*`.
    unsafe { *hl_attr_active.get().add(hlf as usize) }
}

/// The attribute `group` has in `win`, i.e. C's `win_hl_attr` -- [`hl_attr`]
/// unless the window carries a `'winhighlight'` override.
pub(crate) fn win_hl(win: Win, group: c_int) -> c_int {
    // SAFETY: a live window and an `HLF_*` index.
    unsafe { win_hl_attr(win.raw(), group) }
}

/// Two attributes layered, i.e. C's `hl_combine_attr`.
pub(crate) fn combine_attr(under: c_int, over: c_int) -> c_int {
    // SAFETY: two resolved attribute ids.
    unsafe { hl_combine_attr(under, over) }
}

/// Whether `'laststatus'` puts one status line at the bottom of the screen
/// instead of one per window.
pub(crate) fn stl_is_global() -> bool {
    global_stl_height() > 0
}

/// The window a drawing entry point was handed, or `None` for the tab line.
///
/// # Safety
/// `wp` must be null or a live window.
pub(crate) unsafe fn win_opt(wp: *mut win_T) -> Option<Win> {
    // SAFETY: the caller's promise, minus the null case.
    (!wp.is_null()).then(|| unsafe { Win::new(wp) })
}

/// C's `PUT_C`: put `key` in a dictionary that was sized up front.
///
/// Safe because every caller in this module sizes the dictionary from the
/// same expression that decides how many keys it puts; the debug assertion
/// inside catches a mismatch.
pub(crate) fn put(dict: &mut Dict, key: &'static CStr, value: Object) {
    // SAFETY: the invariant above.
    unsafe { dict_put(dict, key, value) };
}

/// C's `ADD_C`. See [`put`].
pub(crate) fn push(array: &mut Array, value: Object) {
    // SAFETY: the invariant above.
    unsafe { array_add(array, value) };
}

// ---------------------------------------------------------------------------
// The shared name scratch
// ---------------------------------------------------------------------------

/// Run `f` over `NameBuff`, the editor's shared path scratch buffer.
///
/// `get_trans_bufname()` fills it and the tab line reads it back, so the two
/// have to be separate borrows: the fill re-enters here.
pub(crate) fn with_name_buff<R>(f: impl FnOnce(&mut [c_char; MAXPATHL as usize]) -> R) -> R {
    NameBuff.with_mut(f)
}

/// The NUL-terminated contents of `buf` as a C string.
pub(crate) fn as_cstr(buf: &[c_char]) -> &CStr {
    // SAFETY: every writer of these buffers NUL-terminates; `from_bytes_until_nul`
    // still checks, and the fallback is the empty string rather than a panic.
    let bytes = unsafe { slice::from_raw_parts(buf.as_ptr().cast::<u8>(), buf.len()) };
    CStr::from_bytes_until_nul(bytes).unwrap_or(c"")
}

// ---------------------------------------------------------------------------
// The entry points the drawing layer calls in
// ---------------------------------------------------------------------------

/// The fill character and highlight group of `win`'s status line.
pub(crate) fn fillchar_status_of(win: Win) -> (hlf_T, schar_T) {
    if win.is_current() {
        (HLF_S, win.w_p_fcs_chars.stl)
    } else {
        (HLF_SNC, win.w_p_fcs_chars.stlnc)
    }
}

/// C's `fillchar_status()`, for the three callers outside this module.
///
/// # Safety
/// `wp` must be a live window and `group` a writable `hlf_T`.
pub unsafe fn fillchar_status(group: *mut hlf_T, wp: *mut win_T) -> schar_T {
    // SAFETY: the caller's promise.
    let (g, fillchar) = fillchar_status_of(unsafe { Win::new(wp) });
    // SAFETY: the caller's out-parameter.
    unsafe { *group = g };
    fillchar
}

/// Redraw `wp`'s status line from `'statusline'`.
///
/// # Safety
/// `wp` must be a live window. This evaluates the option, so it re-enters
/// the editor.
pub unsafe fn redraw_custom_statusline(wp: *mut win_T) {
    static ENTERED: GlobalCell<bool> = GlobalCell::new(false);
    // A `'statusline'` expression that triggers a redraw gets here again.
    if ENTERED.get() {
        return;
    }
    ENTERED.set(true);
    // SAFETY: the caller's promise.
    unsafe { win_redr_custom(wp, false, false, false) };
    ENTERED.set(false);
}

/// Build the `'statuscolumn'` string for line `lnum` into `buf`, answering
/// its width in screen cells.
///
/// `relnum` of -1 means the caller has already set `v:lnum`/`v:relnum` and
/// only wants the text -- which is also what keeps the click definitions
/// from being rebuilt for every line of the window.
///
/// # Safety
/// `wp` must be a live window, `lnum` one of its buffer's lines, `buf` a
/// buffer of `MAXPATHL` bytes and `stcp` this line's status-column state.
pub unsafe fn build_statuscol_str(
    wp: *mut win_T,
    lnum: linenr_T,
    relnum: linenr_T,
    buf: *mut ::core::ffi::c_char,
    stcp: *mut statuscol_T,
) -> ::core::ffi::c_int {
    // SAFETY: the caller's promise.
    let (mut win, stcp) = unsafe { (Win::new(wp), &mut *stcp) };
    // Only update the click definitions once per window per redraw, and not
    // at all while the column is empty -- it is redrawn again once it is not.
    let fillclick = relnum >= 0 && stcp.width > 0 && lnum == win.w_topline;

    if relnum >= 0 {
        // SAFETY: both are plain number variables of the editor's own.
        unsafe { set_vim_var_nr(VV_LNUM, lnum as varnumber_T) };
        // SAFETY: as above.
        unsafe { set_vim_var_nr(VV_RELNUM, relnum as varnumber_T) };
    }

    let job = StlJob {
        win,
        // SAFETY: the window's own option string.
        fmt: unsafe { Fmt::copy_of(win.w_onebuf_opt.wo_stc) },
        opt: (kOptStatuscolumn, OptionSetFlags::LOCAL),
        fillchar: 0,
        maxwidth: stcp.width,
        hl: HlDest::StatusCol,
        want_clicks: fillclick,
        stcp: Some(stcp),
    };
    // SAFETY: `buf` is the caller's `MAXPATHL` buffer, and is never
    // `NameBuff` (the drawing layer keeps its own).
    let out = unsafe { slice::from_raw_parts_mut(buf, MAXPATHL as usize) };
    // SAFETY: the expander re-enters the editor; nothing is held across it.
    let built = unsafe { job.run(out) };

    if let Some(clicks) = built.clicks {
        // SAFETY: the window's own arena and its recorded size.
        let mut arena =
            unsafe { ClickArena::new(win.w_statuscol_click_defs, win.w_statuscol_click_defs_size) };
        arena.clear();
        arena.reserve(built.width);
        arena.fill(clicks, buf, built.width, false);
        let (defs, size) = arena.parts();
        win.w_statuscol_click_defs = defs;
        win.w_statuscol_click_defs_size = size;
    }
    built.width
}
