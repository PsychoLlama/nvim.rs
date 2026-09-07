//! What the screen owes the buffer.
//!
//! `must_redraw` is the headline: the coarsest redraw type anything has
//! asked for since the last update, which [`update_screen`] consumes. Around
//! it sit the finer "this part is stale" flags (`redraw_cmdline`,
//! `redraw_mode`, `redraw_tabline`, `clear_cmdline`, `need_maketitle`), the
//! re-entrancy interlocks that say a redraw must not start now
//! (`updating_screen`, `redraw_not_allowed`, `RedrawingDisabled`), the
//! screen geometry the message and ruler code lays out against
//! (`cmdline_row`, `ru_col`, `ru_wid`, `sc_col`), and `display_tick`, the
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
#![deny(unsafe_op_in_unsafe_fn)]
// The exports here are metrics/abi-ledger.jsonl rows (`display_tick`), and
// `#[unsafe(no_mangle)]` is itself an unsafe attribute.
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::global_cell::GlobalCell;
use crate::types::{
    Buffer, ColNr, DispTick, LPos, LineNr, MatchState, RegMMatch, RegProg, WinExtmark,
};
use core::ffi::c_int;

/// The `ui_watched` extmarks the redraw in progress has passed positions for.
///
/// Filled by `draw_virt_text` one window line at a time and drained by
/// `win_update` once the window is done, so it never outlives one window's
/// redraw. It was upstream's hand-rolled growable array; nothing outside the
/// two of them ever named its layout.
pub(crate) static win_extmark_arr: GlobalCell<Vec<WinExtmark>> = GlobalCell::new(Vec::new());
pub(crate) static updating_screen: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static redraw_not_allowed: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static screen_search_hl: GlobalCell<MatchState> = GlobalCell::new(MatchState {
    rm: RegMMatch {
        regprog: ::core::ptr::null_mut::<RegProg>(),
        startpos: [LPos { lnum: 0, col: 0 }; 10],
        endpos: [LPos { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    },
    buf: ::core::ptr::null_mut::<Buffer>(),
    lnum: 0,
    attr: 0,
    attr_cur: 0,
    first_lnum: 0,
    startcol: 0,
    endcol: 0,
    is_addpos: false,
    has_cursor: false,
    tm: 0,
});
pub(crate) static search_hl_has_cursor_lnum: GlobalCell<LineNr> = GlobalCell::new(0 as LineNr);
pub(crate) static cmdline_row: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static redraw_cmdline: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static redraw_mode: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static clear_cmdline: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static mode_displayed: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static dollar_vcol: GlobalCell<ColNr> = GlobalCell::new(-1 as ColNr);
pub(crate) static need_maketitle: GlobalCell<bool> = GlobalCell::new(true);
pub(crate) static redraw_tabline: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static ru_col: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static ru_wid: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static sc_col: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static RedrawingDisabled: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static must_redraw: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static skip_redraw: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static do_redraw: GlobalCell<bool> = GlobalCell::new(false);
#[unsafe(no_mangle)]
pub static display_tick: GlobalCell<DispTick> = GlobalCell::new(0 as DispTick);
pub(crate) static resizing_screen: GlobalCell<bool> = GlobalCell::new(false);
