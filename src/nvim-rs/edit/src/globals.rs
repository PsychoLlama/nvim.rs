//! Static variables for insert mode, moved from edit.c.
//!
//! Provides `#[no_mangle]` get/set exports matching the original C accessor
//! signatures so that all existing callers (C and Rust) continue to work.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(static_mut_refs)]

use std::ffi::c_int;

/// Line number type (matches `linenr_T`).
type LinenrT = i32;

/// Column number type (matches `colnr_T`).
type ColnrT = i32;

// ============================================================================
// compl_busy — completion re-entry guard
// ============================================================================

static mut COMPL_BUSY: bool = false;

#[no_mangle]
pub unsafe extern "C" fn nvim_get_compl_busy() -> c_int {
    c_int::from(COMPL_BUSY)
}

#[no_mangle]
pub unsafe extern "C" fn nvim_set_compl_busy(val: bool) {
    COMPL_BUSY = val;
}

// ============================================================================
// Insstart_textlen / Insstart_blank_vcol
// ============================================================================

static mut INSSTART_TEXTLEN: ColnrT = 0;
static mut INSSTART_BLANK_VCOL: ColnrT = 0;

#[no_mangle]
pub unsafe extern "C" fn nvim_get_Insstart_textlen() -> ColnrT {
    INSSTART_TEXTLEN
}

#[no_mangle]
pub unsafe extern "C" fn nvim_set_Insstart_textlen(val: ColnrT) {
    INSSTART_TEXTLEN = val;
}

#[no_mangle]
pub unsafe extern "C" fn nvim_get_Insstart_blank_vcol() -> ColnrT {
    INSSTART_BLANK_VCOL
}

#[no_mangle]
pub unsafe extern "C" fn nvim_set_Insstart_blank_vcol(val: ColnrT) {
    INSSTART_BLANK_VCOL = val;
}

// ============================================================================
// update_Insstart_orig
// ============================================================================

static mut UPDATE_INSSTART_ORIG: bool = true;

#[no_mangle]
pub unsafe extern "C" fn nvim_get_update_Insstart_orig() -> c_int {
    c_int::from(UPDATE_INSSTART_ORIG)
}

#[no_mangle]
pub unsafe extern "C" fn nvim_set_update_Insstart_orig(val: c_int) {
    UPDATE_INSSTART_ORIG = val != 0;
}

// ============================================================================
// last_insert_skip / new_insert_skip
// ============================================================================

static mut LAST_INSERT_SKIP: c_int = 0;
static mut NEW_INSERT_SKIP: c_int = 0;

#[no_mangle]
pub unsafe extern "C" fn nvim_get_last_insert_skip() -> c_int {
    LAST_INSERT_SKIP
}

#[no_mangle]
pub unsafe extern "C" fn nvim_set_last_insert_skip(val: c_int) {
    LAST_INSERT_SKIP = val;
}

#[no_mangle]
pub unsafe extern "C" fn nvim_get_new_insert_skip() -> c_int {
    NEW_INSERT_SKIP
}

#[no_mangle]
pub unsafe extern "C" fn nvim_set_new_insert_skip(val: c_int) {
    NEW_INSERT_SKIP = val;
}

// ============================================================================
// did_restart_edit
// ============================================================================

static mut DID_RESTART_EDIT: c_int = 0;

#[no_mangle]
pub unsafe extern "C" fn nvim_get_did_restart_edit() -> c_int {
    DID_RESTART_EDIT
}

#[no_mangle]
pub unsafe extern "C" fn nvim_set_did_restart_edit(val: c_int) {
    DID_RESTART_EDIT = val;
}

// ============================================================================
// can_cindent
// ============================================================================

static mut CAN_CINDENT: bool = false;

#[no_mangle]
pub unsafe extern "C" fn nvim_get_can_cindent() -> c_int {
    c_int::from(CAN_CINDENT)
}

#[no_mangle]
pub unsafe extern "C" fn nvim_set_can_cindent(val: c_int) {
    CAN_CINDENT = val != 0;
}

// ============================================================================
// revins_on / revins_chars / revins_legal / revins_scol
// ============================================================================

static mut REVINS_ON: bool = false;
static mut REVINS_CHARS: c_int = 0;
static mut REVINS_LEGAL: c_int = 0;
static mut REVINS_SCOL: c_int = 0;

#[no_mangle]
pub unsafe extern "C" fn nvim_get_revins_on() -> c_int {
    c_int::from(REVINS_ON)
}

/// Alias used by enter.rs which calls `nvim_edit_set_revins_on`.
#[no_mangle]
pub unsafe extern "C" fn nvim_edit_set_revins_on(val: c_int) {
    REVINS_ON = val != 0;
}

#[no_mangle]
pub unsafe extern "C" fn nvim_get_revins_chars() -> c_int {
    REVINS_CHARS
}

#[no_mangle]
pub unsafe extern "C" fn nvim_set_revins_chars(val: c_int) {
    REVINS_CHARS = val;
}

#[no_mangle]
pub unsafe extern "C" fn nvim_get_revins_legal() -> c_int {
    REVINS_LEGAL
}

#[no_mangle]
pub unsafe extern "C" fn nvim_set_revins_legal(val: c_int) {
    REVINS_LEGAL = val;
}

#[no_mangle]
pub unsafe extern "C" fn nvim_get_revins_scol() -> c_int {
    REVINS_SCOL
}

#[no_mangle]
pub unsafe extern "C" fn nvim_set_revins_scol(val: c_int) {
    REVINS_SCOL = val;
}

// ============================================================================
// ins_need_undo
// ============================================================================

static mut INS_NEED_UNDO: bool = false;

#[no_mangle]
pub unsafe extern "C" fn nvim_get_ins_need_undo() -> c_int {
    c_int::from(INS_NEED_UNDO)
}

#[no_mangle]
pub unsafe extern "C" fn nvim_set_ins_need_undo(val: c_int) {
    INS_NEED_UNDO = val != 0;
}

// ============================================================================
// dont_sync_undo (TriState: kFalse=0, kTrue=1, kNone=-1)
// ============================================================================

static mut DONT_SYNC_UNDO: c_int = 0; // kFalse

#[no_mangle]
pub unsafe extern "C" fn nvim_get_dont_sync_undo() -> c_int {
    DONT_SYNC_UNDO
}

#[no_mangle]
pub unsafe extern "C" fn nvim_set_dont_sync_undo(val: c_int) {
    DONT_SYNC_UNDO = val;
}

// ============================================================================
// o_lnum
// ============================================================================

static mut O_LNUM: LinenrT = 0;

#[no_mangle]
pub unsafe extern "C" fn nvim_get_o_lnum() -> LinenrT {
    O_LNUM
}

#[no_mangle]
pub unsafe extern "C" fn nvim_set_o_lnum(val: LinenrT) {
    O_LNUM = val;
}

// ============================================================================
// edit_saved_cursor[2] — two cursor save slots for arrow key operations
// ============================================================================

/// A cursor position matching `pos_T` layout: lnum, col, coladd.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct SavedCursor {
    lnum: LinenrT,
    col: ColnrT,
    coladd: ColnrT,
}

static mut EDIT_SAVED_CURSOR: [SavedCursor; 2] = [SavedCursor {
    lnum: 0,
    col: 0,
    coladd: 0,
}; 2];

/// `MAXCOL` value from `pos_defs.h`
const MAXCOL: ColnrT = 0x7fff_ffff;

extern "C" {
    fn nvim_linetabsize_cursor_line() -> ColnrT;
}

/// Initialize `Insstart_textlen` from the current cursor line length.
/// Replaces the C function `nvim_edit_init_Insstart_textlen`.
#[no_mangle]
pub unsafe extern "C" fn nvim_edit_init_Insstart_textlen() {
    INSSTART_TEXTLEN = nvim_linetabsize_cursor_line();
    INSSTART_BLANK_VCOL = MAXCOL;
}

// C accessor functions used by save/restore
extern "C" {
    fn nvim_save_cursor_pos(lnum_out: *mut LinenrT, col_out: *mut ColnrT, coladd_out: *mut ColnrT);
    // start_arrow / start_arrow_with_change take *mut pos_T cast to *mut c_void
    fn start_arrow(end_insert_pos: *mut SavedCursor);
    fn start_arrow_with_change(end_insert_pos: *mut SavedCursor, end_change: c_int);
}

/// Save `curwin->w_cursor` into slot `slot` (0 or 1).
#[no_mangle]
pub unsafe extern "C" fn nvim_edit_save_cursor(slot: c_int) {
    let s = &raw mut EDIT_SAVED_CURSOR[slot as usize];
    nvim_save_cursor_pos(&raw mut (*s).lnum, &raw mut (*s).col, &raw mut (*s).coladd);
}

/// Call `start_arrow()` with the cursor saved in slot `slot`.
#[no_mangle]
pub unsafe extern "C" fn nvim_edit_start_arrow_from_slot(slot: c_int) {
    start_arrow(&raw mut EDIT_SAVED_CURSOR[slot as usize]);
}

/// Call `start_arrow_with_change()` with the cursor saved in slot `slot`.
#[no_mangle]
pub unsafe extern "C" fn nvim_edit_start_arrow_with_change_from_slot(
    slot: c_int,
    end_change: c_int,
) {
    start_arrow_with_change(&raw mut EDIT_SAVED_CURSOR[slot as usize], end_change);
}

// ============================================================================
// saved_topline / saved_topfill — for topline change detection
// ============================================================================

static mut SAVED_TOPLINE: LinenrT = 0;
static mut SAVED_TOPFILL: c_int = 0;

extern "C" {
    fn nvim_get_curwin() -> nvim_window::WinHandle;
}

/// Save `curwin->w_topline` and `w_topfill`.
#[no_mangle]
pub unsafe extern "C" fn nvim_edit_save_topline() {
    let curwin = nvim_get_curwin();
    let cw = nvim_window::win_struct::win_ref(curwin);
    SAVED_TOPLINE = cw.w_topline;
    SAVED_TOPFILL = cw.w_topfill;
}

/// Return 1 if topline/topfill changed since last save, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn nvim_edit_topline_changed() -> c_int {
    let curwin = nvim_get_curwin();
    let cw = nvim_window::win_struct::win_ref(curwin);
    let changed = SAVED_TOPLINE != cw.w_topline || SAVED_TOPFILL != cw.w_topfill;
    c_int::from(changed)
}
