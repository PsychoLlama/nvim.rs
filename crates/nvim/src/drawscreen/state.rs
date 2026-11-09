//! What the screen owes the buffer.
//!
//! `MUST_REDRAW` is the headline: the coarsest redraw type anything has
//! asked for since the last update, which [`update_screen`] consumes. Around
//! it sit the finer "this part is stale" flags (`REDRAW_CMDLINE`,
//! `REDRAW_MODE`, `REDRAW_TABLINE`, `CLEAR_CMDLINE`, `NEED_MAKETITLE`), the
//! re-entrancy interlocks that say a redraw must not start now
//! (`UPDATING_SCREEN`, `REDRAW_NOT_ALLOWED`, `REDRAWING_DISABLED`), the
//! screen geometry the message and ruler code lays out against
//! (`CMDLINE_ROW`, `RU_COL`, `RU_WID`, `SC_COL`), and `display_tick`, the
//! counter a decoration provider compares against to know it has already
//! been asked about this pass.
//!
//! [`update_screen`]: super::update_screen
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::drawline::WinExtmark;
use crate::global_cell::GlobalCell;
/// Exported by name for the FFI spec, so the cell lives in the harness.
pub use crate::harness::display_tick;
use crate::regexp::RegMMatch;
use crate::types::{ColNr, LPos, LineNr, MatchState};
use core::ffi::c_int;

/// The `ui_watched` extmarks the redraw in progress has passed positions for.
///
/// Filled by `draw_virt_text` one window line at a time and drained by
/// `win_update` once the window is done, so it never outlives one window's
/// redraw. It was upstream's hand-rolled growable array; nothing outside the
/// two of them ever named its layout.
pub(crate) static WIN_EXTMARK_ARR: GlobalCell<Vec<WinExtmark>> = GlobalCell::new(Vec::new());
pub(crate) static UPDATING_SCREEN: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static REDRAW_NOT_ALLOWED: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static SCREEN_SEARCH_HL: GlobalCell<MatchState> = GlobalCell::new(MatchState {
    rm: RegMMatch {
        regprog: None,
        startpos: [LPos {
            lnum: LineNr::ZERO,
            col: ColNr::ZERO,
        }; 10],
        endpos: [LPos {
            lnum: LineNr::ZERO,
            col: ColNr::ZERO,
        }; 10],
        rmm_matchcol: ColNr::ZERO,
        rmm_ic: 0,
        rmm_maxcol: ColNr::ZERO,
    },
    lnum: LineNr::ZERO,
    attr: 0,
    attr_cur: 0,
    first_lnum: LineNr::ZERO,
    startcol: ColNr::ZERO,
    endcol: ColNr::ZERO,
    is_addpos: false,
    has_cursor: false,
    conceals: false,
    tm: 0,
});
pub(crate) static SEARCH_HL_HAS_CURSOR_LNUM: GlobalCell<LineNr> = GlobalCell::new(LineNr::ZERO);
pub(crate) static CMDLINE_ROW: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static REDRAW_CMDLINE: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static REDRAW_MODE: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static CLEAR_CMDLINE: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static MODE_DISPLAYED: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static DOLLAR_VCOL: GlobalCell<ColNr> = GlobalCell::new(ColNr::NONE);
pub(crate) static NEED_MAKETITLE: GlobalCell<bool> = GlobalCell::new(true);
pub(crate) static REDRAW_TABLINE: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static RU_COL: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static RU_WID: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static SC_COL: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static REDRAWING_DISABLED: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static MUST_REDRAW: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static SKIP_REDRAW: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static DO_REDRAW: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static RESIZING_SCREEN: GlobalCell<bool> = GlobalCell::new(false);
