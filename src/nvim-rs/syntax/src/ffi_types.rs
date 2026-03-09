//! Repr(C) struct definitions for C syntax types.
//!
//! These structs mirror the C struct layouts exactly, verified by
//! _Static_assert checks in syntax_accessors.c.
//!
//! # Safety
//!
//! All structs in this module are `#[repr(C)]` and must match the corresponding
//! C struct layout exactly. Mismatches will cause undefined behavior (SIGSEGV).
//! Layout is verified at compile time by _Static_assert in syntax_accessors.c.

use std::ffi::{c_char, c_int, c_void};

// Sizes:
//   lpos_T:         8 bytes
//   syn_time_T:    24 bytes
//   SpSyn:         16 bytes
//   SynPat:       136 bytes
//   SynCluster:    24 bytes
//   StateItem:    104 bytes
//   BufState:      24 bytes
//   KeyEntry:      40 bytes (without flexible array member)

// =============================================================================
// lpos_T: line/column position (8 bytes)
// =============================================================================

/// Repr(C) mirror of lpos_T (line position with lnum and col).
///
/// Layout: { lnum: i32, col: i32 } = 8 bytes
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LPos {
    /// Line number (1-based; 0 means "not set")
    pub lnum: i32,
    /// Column number
    pub col: c_int,
}

// =============================================================================
// syn_time_T: syntax timing info (24 bytes)
// =============================================================================

/// Repr(C) mirror of syn_time_T.
///
/// Layout: { total: u64, slowest: u64, count: i32, match_: i32 } = 24 bytes
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SynTime {
    /// Total time used (proftime_T = u64)
    pub total: u64,
    /// Time of slowest call (proftime_T = u64)
    pub slowest: u64,
    /// Number of times used
    pub count: c_int,
    /// Number of times matched
    pub match_: c_int,
}

// =============================================================================
// struct sp_syn: passed to in_id_list() (16 bytes)
// =============================================================================

/// Repr(C) mirror of `struct sp_syn`.
///
/// Layout (16 bytes):
///   offset 0: inc_tag (i32)
///   offset 4: id (i16)
///   offset 6: _pad (2 bytes)
///   offset 8: cont_in_list (*mut i16)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SpSyn {
    /// ":syn include" unique tag
    pub inc_tag: c_int,
    /// Highlight group ID of item
    pub id: i16,
    /// Padding to align cont_in_list pointer
    pub _pad: i16,
    /// cont.in group IDs, if non-zero
    pub cont_in_list: *mut i16,
}

impl Default for SpSyn {
    fn default() -> Self {
        Self {
            inc_tag: 0,
            id: 0,
            _pad: 0,
            cont_in_list: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// synpat_S / synpat_T: syntax pattern (136 bytes)
// =============================================================================

/// Repr(C) mirror of `struct synpat_S` (aka `synpat_T`).
///
/// Layout (136 bytes):
///   offset   0: sp_type (i8 / char)
///   offset   1: sp_syncing (bool)
///   offset   2: sp_syn_match_id (i16)
///   offset   4: sp_off_flags (i16)
///   offset   6: _pad2 (2 bytes, implicit)
///   offset   8: sp_offsets ([i32; 7]) = 28 bytes
///   offset  36: sp_flags (i32)
///   offset  40: sp_cchar (i32)
///   offset  44: sp_ic (i32)
///   offset  48: sp_sync_idx (i32)
///   offset  52: sp_line_id (i32)
///   offset  56: sp_startcol (i32)
///   offset  60: _pad3 (4 bytes, implicit)
///   offset  64: sp_cont_list (*mut i16)
///   offset  72: sp_next_list (*mut i16)
///   offset  80: sp_syn (SpSyn, 16 bytes)
///   offset  96: sp_pattern (*mut c_char)
///   offset 104: sp_prog (*mut c_void / regprog_T*)
///   offset 112: sp_time (SynTime, 24 bytes)
#[repr(C)]
pub struct SynPat {
    /// Pattern type (SPTYPE_MATCH, SPTYPE_START, SPTYPE_END, SPTYPE_SKIP)
    pub sp_type: i8,
    /// True if this item is used for syncing
    pub sp_syncing: bool,
    /// Highlight group ID of pattern
    pub sp_syn_match_id: i16,
    /// Offset flags (see SPO_* constants)
    pub sp_off_flags: i16,
    /// Padding between sp_off_flags and sp_offsets
    pub _pad: [u8; 2],
    /// Offsets array (SPO_COUNT = 7 entries)
    pub sp_offsets: [c_int; 7],
    /// Highlight flags (HL_* constants)
    pub sp_flags: c_int,
    /// Conceal substitute character
    pub sp_cchar: c_int,
    /// Ignore-case flag for sp_prog
    pub sp_ic: c_int,
    /// Sync item index (syncing only)
    pub sp_sync_idx: c_int,
    /// ID of last line where tried
    pub sp_line_id: c_int,
    /// Next match in sp_line_id line
    pub sp_startcol: c_int,
    /// Padding before sp_cont_list pointer
    pub _pad2: [u8; 4],
    /// Contained group IDs (null-terminated i16 array, or null)
    pub sp_cont_list: *mut i16,
    /// Nextgroup IDs (null-terminated i16 array, or null)
    pub sp_next_list: *mut i16,
    /// sp_syn sub-struct for in_id_list()
    pub sp_syn: SpSyn,
    /// Regexp pattern string
    pub sp_pattern: *mut c_char,
    /// Compiled regexp program
    pub sp_prog: *mut c_void,
    /// Timing info
    pub sp_time: SynTime,
}

impl Default for SynPat {
    fn default() -> Self {
        Self {
            sp_type: 0,
            sp_syncing: false,
            sp_syn_match_id: 0,
            sp_off_flags: 0,
            _pad: [0; 2],
            sp_offsets: [0; 7],
            sp_flags: 0,
            sp_cchar: 0,
            sp_ic: 0,
            sp_sync_idx: 0,
            sp_line_id: 0,
            sp_startcol: 0,
            _pad2: [0; 4],
            sp_cont_list: std::ptr::null_mut(),
            sp_next_list: std::ptr::null_mut(),
            sp_syn: SpSyn::default(),
            sp_pattern: std::ptr::null_mut(),
            sp_prog: std::ptr::null_mut(),
            sp_time: SynTime::default(),
        }
    }
}

// =============================================================================
// syn_cluster_S / syn_cluster_T: syntax cluster (24 bytes)
// =============================================================================

/// Repr(C) mirror of `struct syn_cluster_S` (aka `syn_cluster_T`).
///
/// Layout (24 bytes):
///   offset  0: scl_name (*mut c_char)
///   offset  8: scl_name_u (*mut c_char)
///   offset 16: scl_list (*mut i16)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SynCluster {
    /// Syntax cluster name
    pub scl_name: *mut c_char,
    /// Uppercase of scl_name
    pub scl_name_u: *mut c_char,
    /// IDs in this syntax cluster
    pub scl_list: *mut i16,
}

impl Default for SynCluster {
    fn default() -> Self {
        Self {
            scl_name: std::ptr::null_mut(),
            scl_name_u: std::ptr::null_mut(),
            scl_list: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// stateitem_S / stateitem_T: current state stack item (104 bytes)
// =============================================================================

/// Repr(C) mirror of `struct stateitem_S` (aka `stateitem_T`).
///
/// Layout (104 bytes):
///   offset  0: si_idx (i32)
///   offset  4: si_id (i32)
///   offset  8: si_trans_id (i32)
///   offset 12: si_m_lnum (i32)
///   offset 16: si_m_startcol (i32)
///   offset 20: si_m_endpos (LPos, 8 bytes)
///   offset 28: si_h_startpos (LPos, 8 bytes)
///   offset 36: si_h_endpos (LPos, 8 bytes)
///   offset 44: si_eoe_pos (LPos, 8 bytes)
///   offset 52: si_end_idx (i32)
///   offset 56: si_ends (i32)
///   offset 60: si_attr (i32)
///   offset 64: si_flags (i32)
///   offset 68: si_seqnr (i32)
///   offset 72: si_cchar (i32)
///   offset 76: _pad (4 bytes, implicit)
///   offset 80: si_cont_list (*mut i16)
///   offset 88: si_next_list (*mut i16)
///   offset 96: si_extmatch (*mut c_void / reg_extmatch_T*)
#[repr(C)]
pub struct StateItem {
    /// Index of syntax pattern or KEYWORD_IDX
    pub si_idx: c_int,
    /// Highlight group ID for keywords
    pub si_id: c_int,
    /// Transparency-removed ID
    pub si_trans_id: c_int,
    /// Line number of the match
    pub si_m_lnum: c_int,
    /// Starting column of the match
    pub si_m_startcol: c_int,
    /// Just after end position of the match
    pub si_m_endpos: LPos,
    /// Start position of the highlighting
    pub si_h_startpos: LPos,
    /// End position of the highlighting
    pub si_h_endpos: LPos,
    /// End position of end pattern
    pub si_eoe_pos: LPos,
    /// Group ID for end pattern or zero
    pub si_end_idx: c_int,
    /// True if match ends before si_m_endpos
    pub si_ends: c_int,
    /// Attributes in this state
    pub si_attr: c_int,
    /// HL_HAS_EOL flag and HL_SKIP* for si_next_list
    pub si_flags: c_int,
    /// Sequence number
    pub si_seqnr: c_int,
    /// Substitution character for conceal
    pub si_cchar: c_int,
    /// Padding before si_cont_list pointer
    pub _pad: [u8; 4],
    /// List of contained groups
    pub si_cont_list: *mut i16,
    /// Nextgroup IDs after this item ends
    pub si_next_list: *mut i16,
    /// External match references from start pattern
    pub si_extmatch: *mut c_void,
}

impl Default for StateItem {
    fn default() -> Self {
        Self {
            si_idx: 0,
            si_id: 0,
            si_trans_id: 0,
            si_m_lnum: 0,
            si_m_startcol: 0,
            si_m_endpos: LPos::default(),
            si_h_startpos: LPos::default(),
            si_h_endpos: LPos::default(),
            si_eoe_pos: LPos::default(),
            si_end_idx: 0,
            si_ends: 0,
            si_attr: 0,
            si_flags: 0,
            si_seqnr: 0,
            si_cchar: 0,
            _pad: [0; 4],
            si_cont_list: std::ptr::null_mut(),
            si_next_list: std::ptr::null_mut(),
            si_extmatch: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// bufstate_T: stored state for state stack entry (24 bytes)
// =============================================================================

/// Repr(C) mirror of `bufstate_T`.
///
/// Layout (24 bytes):
///   offset  0: bs_idx (i32)
///   offset  4: bs_flags (i32)
///   offset  8: bs_seqnr (i32)
///   offset 12: bs_cchar (i32)
///   offset 16: bs_extmatch (*mut c_void)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BufState {
    /// Index of pattern
    pub bs_idx: c_int,
    /// Flags for pattern
    pub bs_flags: c_int,
    /// Stores si_seqnr
    pub bs_seqnr: c_int,
    /// Stores si_cchar
    pub bs_cchar: c_int,
    /// External matches from start pattern
    pub bs_extmatch: *mut c_void,
}

impl Default for BufState {
    fn default() -> Self {
        Self {
            bs_idx: 0,
            bs_flags: 0,
            bs_seqnr: 0,
            bs_cchar: 0,
            bs_extmatch: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// keyentry_T: keyword hash table entry (40 bytes + flexible array member)
// =============================================================================

/// Repr(C) mirror of `struct keyentry` (aka `keyentry_T`), without the
/// flexible array member `keyword[]`.
///
/// The `keyword` field is a flexible array member in C, so it cannot be
/// represented in a normal Rust struct. Access to the keyword string must
/// be done via raw pointer arithmetic (ptr + 40).
///
/// Layout (40 bytes, no FAM):
///   offset  0: ke_next (*mut KeyEntry)
///   offset  8: k_syn (SpSyn, 16 bytes)
///   offset 24: next_list (*mut i16)
///   offset 32: flags (i32)
///   offset 36: k_char (i32)
///   [offset 40: keyword[] -- flexible array, not in struct]
#[repr(C)]
pub struct KeyEntry {
    /// Next entry with identical keyword[]
    pub ke_next: *mut KeyEntry,
    /// sp_syn sub-struct for in_id_list()
    pub k_syn: SpSyn,
    /// ID list for next match (if non-zero)
    pub next_list: *mut i16,
    /// Keyword flags
    pub flags: c_int,
    /// Conceal substitute character
    pub k_char: c_int,
    // keyword[] is a flexible array member -- NOT in this struct.
    // Access via: (ke as *const u8).add(40) as *const c_char
}

impl KeyEntry {
    /// Get a pointer to the keyword string (flexible array member).
    ///
    /// # Safety
    ///
    /// `ke` must point to a valid `KeyEntry` allocated with space for the
    /// keyword string appended after the fixed fields.
    pub unsafe fn keyword_ptr(ke: *const KeyEntry) -> *const c_char {
        unsafe { (ke as *const u8).add(40) as *const c_char }
    }
}

// =============================================================================
// Compile-time size assertions
// =============================================================================

const _: () = {
    assert!(std::mem::size_of::<LPos>() == 8, "LPos size mismatch");
    assert!(
        std::mem::size_of::<SynTime>() == 24,
        "SynTime size mismatch"
    );
    assert!(std::mem::size_of::<SpSyn>() == 16, "SpSyn size mismatch");
    assert!(std::mem::size_of::<SynPat>() == 136, "SynPat size mismatch");
    assert!(
        std::mem::size_of::<SynCluster>() == 24,
        "SynCluster size mismatch"
    );
    assert!(
        std::mem::size_of::<StateItem>() == 104,
        "StateItem size mismatch"
    );
    assert!(
        std::mem::size_of::<BufState>() == 24,
        "BufState size mismatch"
    );
    assert!(
        std::mem::size_of::<KeyEntry>() == 40,
        "KeyEntry size mismatch"
    );
};
