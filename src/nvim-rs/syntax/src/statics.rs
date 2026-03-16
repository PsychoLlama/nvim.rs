//! Migrated C static variables for the syntax engine.
//!
//! These `static mut` variables correspond to file-scope statics that were
//! previously in `syntax_accessors.c`. Moving them to Rust eliminates the
//! need for C accessor functions to bridge the FFI boundary.
//!
//! # Safety
//!
//! All access to these statics must be done in `unsafe` blocks. The syntax
//! engine is single-threaded (runs on the main Neovim thread), so no
//! synchronization is needed.

use std::ffi::c_int;

use crate::ffi_types::LPos;

// =============================================================================
// Phase 1: Scalar statics migrated from syntax_accessors.c
// =============================================================================

/// Attribute of current syntax word (mapped from C `current_attr`).
#[no_mangle]
pub static mut CURRENT_ATTR: c_int = 0;

/// ID of current char for syn_get_id() (mapped from C `current_id`).
#[no_mangle]
pub static mut CURRENT_ID: c_int = 0;

/// Transparency-removed ID (mapped from C `current_trans_id`).
#[no_mangle]
pub static mut CURRENT_TRANS_ID: c_int = 0;

/// Current highlight flags (mapped from C `current_flags`).
#[no_mangle]
pub static mut CURRENT_FLAGS: c_int = 0;

/// Current sequence number (mapped from C `current_seqnr`).
#[no_mangle]
pub static mut CURRENT_SEQNR: c_int = 0;

/// Current substitution character (mapped from C `current_sub_char`).
#[no_mangle]
pub static mut CURRENT_SUB_CHAR: c_int = 0;

/// Line number of current state (mapped from C `current_lnum`).
#[no_mangle]
pub static mut CURRENT_LNUM: c_int = 0;

/// Column of current state (mapped from C `current_col`).
#[no_mangle]
pub static mut CURRENT_COL: c_int = 0;

/// True if current line has been finished (mapped from C `current_finished`).
/// Stored as int (0/1) to allow atomic access from both Rust and C during migration.
#[no_mangle]
pub static mut CURRENT_FINISHED: c_int = 0;

/// True if stored current state after setting current_finished
/// (mapped from C `current_state_stored`).
#[no_mangle]
pub static mut CURRENT_STATE_STORED: c_int = 0;

/// Flags for current_next_list (mapped from C `current_next_flags`).
#[no_mangle]
pub static mut CURRENT_NEXT_FLAGS: c_int = 0;

/// Unique number for current line (mapped from C `current_line_id`).
#[no_mangle]
pub static mut CURRENT_LINE_ID: c_int = 0;

/// Unique tag for `:syn include`'d rules (mapped from C `current_syn_inc_tag`).
#[no_mangle]
pub static mut CURRENT_SYN_INC_TAG: c_int = 0;

/// Running tag counter for `:syn include` (mapped from C `running_syn_inc_tag`).
#[no_mangle]
pub static mut RUNNING_SYN_INC_TAG: c_int = 0;

/// Level of first keepend item on state stack, -1 if none
/// (mapped from C `keepend_level`).
#[no_mangle]
pub static mut KEEPEND_LEVEL: c_int = -1;

/// Value to use for si_seqnr (mapped from C `next_seqnr`).
#[no_mangle]
pub static mut NEXT_SEQNR: c_int = 1;

/// True if syntax timing is enabled (mapped from C `syn_time_on`).
#[no_mangle]
pub static mut SYN_TIME_ON: c_int = 0;

/// True if `:syntax on/off` was called (mapped from C `did_syntax_onoff`).
#[no_mangle]
pub static mut DID_SYNTAX_ONOFF: c_int = 0;

/// What to expand for `:syn` completion (mapped from C `expand_what`).
#[no_mangle]
pub static mut EXPAND_WHAT: c_int = 0;

// =============================================================================
// Phase 2: Pointer and struct statics migrated from syntax_accessors.c
// =============================================================================

/// Column for start of next match (mapped from C `next_match_col`).
/// Value MAXCOL (0x7fffffff) means no match found.
#[no_mangle]
pub static mut NEXT_MATCH_COL: c_int = 0;

/// Index of matched item (mapped from C `next_match_idx`).
/// Value -1 means not tried yet.
#[no_mangle]
pub static mut NEXT_MATCH_IDX: c_int = -1;

/// Flags for next match (mapped from C `next_match_flags`).
#[no_mangle]
pub static mut NEXT_MATCH_FLAGS: c_int = 0;

/// ID of group for end pattern or zero (mapped from C `next_match_end_idx`).
#[no_mangle]
pub static mut NEXT_MATCH_END_IDX: c_int = 0;

/// Position for end of next match (mapped from C `next_match_m_endpos`).
#[no_mangle]
pub static mut NEXT_MATCH_M_ENDPOS: LPos = LPos { lnum: 0, col: 0 };

/// Position for highlight start of next match (mapped from C `next_match_h_startpos`).
#[no_mangle]
pub static mut NEXT_MATCH_H_STARTPOS: LPos = LPos { lnum: 0, col: 0 };

/// Position for highlight end of next match (mapped from C `next_match_h_endpos`).
#[no_mangle]
pub static mut NEXT_MATCH_H_ENDPOS: LPos = LPos { lnum: 0, col: 0 };

/// End of start pattern position (mapped from C `next_match_eos_pos`).
#[no_mangle]
pub static mut NEXT_MATCH_EOS_POS: LPos = LPos { lnum: 0, col: 0 };

/// Position for end of end pattern (mapped from C `next_match_eoe_pos`).
#[no_mangle]
pub static mut NEXT_MATCH_EOE_POS: LPos = LPos { lnum: 0, col: 0 };

/// Current nextgroup list pointer (mapped from C `current_next_list`).
/// NULL when not active.
#[no_mangle]
pub static mut CURRENT_NEXT_LIST: *mut i16 = std::ptr::null_mut();

/// Extmatch for next match (mapped from C `next_match_extmatch`).
/// Opaque pointer -- NULL when no match.
#[no_mangle]
pub static mut NEXT_MATCH_EXTMATCH: *mut std::ffi::c_void = std::ptr::null_mut();
