//! Syntax highlighting subsystem for Neovim
//!
//! This crate provides syntax pattern matching, state machine management,
//! and integration with the highlighting system. It manages:
//!
//! - Syntax patterns (match, region, keyword)
//! - State stack for parsing context
//! - Cluster and containedin logic
//! - Integration with regexp and highlight crates

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::manual_range_contains)]
#![allow(dead_code)] // Many extern fns declared for future use

use std::ffi::{c_char, c_int};

// =============================================================================
// Opaque handle types for C interop
// =============================================================================

/// Opaque handle to a synblock_T (syntax block attached to buffer/window)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SynBlockHandle(*mut std::ffi::c_void);

impl SynBlockHandle {
    /// Check if the handle is null
    #[must_use]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a synstate_T (syntax state at start of a line)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SynStateHandle(*mut std::ffi::c_void);

impl SynStateHandle {
    /// Check if the handle is null
    #[must_use]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle
    #[must_use]
    pub fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

/// Opaque handle to a synpat_T (syntax pattern)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SynPatHandle(*mut std::ffi::c_void);

impl SynPatHandle {
    /// Check if the handle is null
    #[must_use]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a syn_cluster_T (syntax cluster)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SynClusterHandle(*mut std::ffi::c_void);

impl SynClusterHandle {
    /// Check if the handle is null
    #[must_use]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a stateitem_T (current state stack item)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct StateItemHandle(*mut std::ffi::c_void);

impl StateItemHandle {
    /// Check if the handle is null
    #[must_use]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a keyentry_T (keyword entry in hashtable)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct KeyEntryHandle(*mut std::ffi::c_void);

impl KeyEntryHandle {
    /// Check if the handle is null
    #[must_use]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a regprog_T (compiled regex program)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct RegProgHandle(*mut std::ffi::c_void);

impl RegProgHandle {
    /// Check if the handle is null
    #[must_use]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle
    #[must_use]
    pub fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

/// Opaque handle to an ID list (int16_t array terminated by 0)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct IdListHandle(*mut i16);

impl IdListHandle {
    /// Check if the handle is null
    #[must_use]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle
    #[must_use]
    pub fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

// =============================================================================
// Constants - Highlight flags (HL_*)
// =============================================================================

/// Pattern is contained (not used on toplevel)
pub const HL_CONTAINED: c_int = 0x01;
/// Pattern has no highlighting (transparent)
pub const HL_TRANSP: c_int = 0x02;
/// Match within one line only
pub const HL_ONELINE: c_int = 0x04;
/// End pattern that matches with $
pub const HL_HAS_EOL: c_int = 0x08;
/// Sync point after this item (syncing only)
pub const HL_SYNC_HERE: c_int = 0x10;
/// Sync point at current line (syncing only)
pub const HL_SYNC_THERE: c_int = 0x20;
/// Use match ID instead of item ID
pub const HL_MATCH: c_int = 0x40;
/// Nextgroup can skip newlines
pub const HL_SKIPNL: c_int = 0x80;
/// Nextgroup can skip white space
pub const HL_SKIPWHITE: c_int = 0x100;
/// Nextgroup can skip empty lines
pub const HL_SKIPEMPTY: c_int = 0x200;
/// End match always kept
pub const HL_KEEPEND: c_int = 0x400;
/// Exclude NL from match
pub const HL_EXCLUDENL: c_int = 0x800;
/// Only used for displaying, not syncing
pub const HL_DISPLAY: c_int = 0x1000;
/// Define fold
pub const HL_FOLD: c_int = 0x2000;
/// Ignore a keepend
pub const HL_EXTEND: c_int = 0x4000;
/// Match continued from previous line
pub const HL_MATCHCONT: c_int = 0x8000;
/// Transparent item without contains arg
pub const HL_TRANS_CONT: c_int = 0x1_0000;
/// Can be concealed
pub const HL_CONCEAL: c_int = 0x2_0000;
/// Ends can be concealed
pub const HL_CONCEALENDS: c_int = 0x4_0000;
/// Toplevel item in included syntax, allowed by contains=TOP
pub const HL_INCLUDED_TOPLEVEL: c_int = 0x8_0000;

// =============================================================================
// Constants - Syntax pattern type (SPTYPE_*)
// =============================================================================

/// Match keyword with this group ID
pub const SPTYPE_MATCH: c_int = 1;
/// Match a regexp, start of item
pub const SPTYPE_START: c_int = 2;
/// Match a regexp, end of item
pub const SPTYPE_END: c_int = 3;
/// Match a regexp, skip within item
pub const SPTYPE_SKIP: c_int = 4;

// =============================================================================
// Constants - Syntax group IDs
// =============================================================================

/// Maximum highlight group ID for normal syntax groups
pub const MAX_HL_ID: c_int = 20000;
/// Syntax group ID for contains=ALLBUT
pub const SYNID_ALLBUT: c_int = MAX_HL_ID;
/// Syntax group ID for contains=TOP
pub const SYNID_TOP: c_int = 21000;
/// Syntax group ID for contains=CONTAINED
pub const SYNID_CONTAINED: c_int = 22000;
/// First syntax group ID for clusters
pub const SYNID_CLUSTER: c_int = 23000;

/// Maximum before the above overflow
pub const MAX_SYN_INC_TAG: c_int = 999;
/// Maximum cluster ID
pub const MAX_CLUSTER_ID: c_int = 32767 - SYNID_CLUSTER;

// =============================================================================
// Constants - Syntax sync flags (SF_*)
// =============================================================================

/// Sync on a C-style comment
pub const SF_CCOMMENT: c_int = 0x01;
/// Sync by matching a pattern
pub const SF_MATCH: c_int = 0x02;

// =============================================================================
// Constants - Syntax spell checking (SYNSPL_*)
// =============================================================================

/// Spelling not set
pub const SYNSPL_DEFAULT: c_int = 0;
/// Spell checking on (toplevel file)
pub const SYNSPL_TOP: c_int = 1;
/// Spell checking off (included file)
pub const SYNSPL_NOTOP: c_int = 2;

// =============================================================================
// Constants - Cluster operations
// =============================================================================

/// Replace first list with second
pub const CLUSTER_REPLACE: c_int = 1;
/// Add second list to first
pub const CLUSTER_ADD: c_int = 2;
/// Subtract second list from first
pub const CLUSTER_SUBTRACT: c_int = 3;

// =============================================================================
// Constants - State stack array sizes
// =============================================================================

/// Minimal size for state stack array
pub const SST_MIN_ENTRIES: c_int = 150;
/// Maximal size for state stack array
pub const SST_MAX_ENTRIES: c_int = 1000;
/// Size of fixed state stack (sst_stack)
pub const SST_FIX_STATES: c_int = 7;
/// Normal distance between entries
pub const SST_DIST: c_int = 16;

// =============================================================================
// Constants - Special indices
// =============================================================================

/// Value of si_idx for keywords
pub const KEYWORD_IDX: c_int = -1;
/// Value of sp_sync_idx for "NONE"
pub const NONE_IDX: c_int = -2;
/// Maximum length of a keyword
pub const MAXKEYWLEN: c_int = 80;

// =============================================================================
// Syntax ID helper functions
// =============================================================================

/// Represents the type of a syntax ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynIdType {
    /// Normal syntax group (0 - 19999)
    Normal,
    /// ALLBUT indicator (20000 - 20999, with inc_tag)
    AllBut,
    /// TOP indicator (21000 - 21999, with inc_tag)
    Top,
    /// CONTAINED indicator (22000 - 22999, with inc_tag)
    Contained,
    /// Cluster reference (23000 - 32767)
    Cluster,
}

/// Classify a syntax ID into its type
#[must_use]
pub const fn synid_type(id: i16) -> SynIdType {
    let id = id as c_int;
    if id >= SYNID_CLUSTER {
        SynIdType::Cluster
    } else if id >= SYNID_CONTAINED {
        SynIdType::Contained
    } else if id >= SYNID_TOP {
        SynIdType::Top
    } else if id >= SYNID_ALLBUT {
        SynIdType::AllBut
    } else {
        SynIdType::Normal
    }
}

/// Check if an ID is a cluster reference
#[must_use]
pub const fn is_cluster_id(id: i16) -> bool {
    (id as c_int) >= SYNID_CLUSTER
}

/// Check if an ID is a special group (ALLBUT, TOP, CONTAINED, or Cluster)
#[must_use]
pub const fn is_special_id(id: i16) -> bool {
    (id as c_int) >= SYNID_ALLBUT
}

/// Check if an ID is a normal syntax group
#[must_use]
pub const fn is_normal_id(id: i16) -> bool {
    (id as c_int) > 0 && (id as c_int) < SYNID_ALLBUT
}

/// Extract the cluster index from a cluster ID
/// Returns None if not a cluster ID
#[must_use]
pub const fn cluster_index(id: i16) -> Option<i16> {
    let id_int = id as c_int;
    if id_int >= SYNID_CLUSTER {
        Some((id_int - SYNID_CLUSTER) as i16)
    } else {
        None
    }
}

/// Create a cluster ID from a cluster index
#[must_use]
pub const fn make_cluster_id(index: i16) -> i16 {
    (SYNID_CLUSTER + index as c_int) as i16
}

/// Extract the include tag from an ALLBUT/TOP/CONTAINED ID
/// Returns None if not an ALLBUT/TOP/CONTAINED ID
#[must_use]
pub const fn extract_inc_tag(id: i16) -> Option<i16> {
    let id_int = id as c_int;
    if id_int >= SYNID_ALLBUT && id_int < SYNID_CLUSTER {
        // The inc_tag is encoded in the lower bits
        if id_int >= SYNID_CONTAINED {
            Some((id_int - SYNID_CONTAINED) as i16)
        } else if id_int >= SYNID_TOP {
            Some((id_int - SYNID_TOP) as i16)
        } else {
            Some((id_int - SYNID_ALLBUT) as i16)
        }
    } else {
        None
    }
}

// =============================================================================
// C accessor function declarations
// =============================================================================

extern "C" {
    // -------------------------------------------------------------------------
    // synblock_T accessors (syntax block)
    // -------------------------------------------------------------------------

    /// Get b_syn_patterns.ga_len (number of syntax patterns)
    fn nvim_synblock_get_pattern_count(block: SynBlockHandle) -> c_int;

    /// Get b_syn_clusters.ga_len (number of syntax clusters)
    fn nvim_synblock_get_cluster_count(block: SynBlockHandle) -> c_int;

    /// Get b_syn_ic (ignore case for :syn cmds)
    fn nvim_synblock_get_syn_ic(block: SynBlockHandle) -> c_int;

    /// Get b_syn_spell (SYNSPL_ values)
    fn nvim_synblock_get_syn_spell(block: SynBlockHandle) -> c_int;

    /// Get b_syn_foldlevel
    fn nvim_synblock_get_syn_foldlevel(block: SynBlockHandle) -> c_int;

    /// Get b_syn_containedin (true if any item has containedin)
    fn nvim_synblock_get_containedin(block: SynBlockHandle) -> c_int;

    /// Get b_syn_sync_flags
    fn nvim_synblock_get_sync_flags(block: SynBlockHandle) -> c_int;

    /// Get b_syn_sync_id
    fn nvim_synblock_get_sync_id(block: SynBlockHandle) -> i16;

    /// Get b_syn_sync_minlines
    fn nvim_synblock_get_sync_minlines(block: SynBlockHandle) -> c_int;

    /// Get b_syn_sync_maxlines
    fn nvim_synblock_get_sync_maxlines(block: SynBlockHandle) -> c_int;

    /// Get b_syn_sync_linebreaks
    fn nvim_synblock_get_sync_linebreaks(block: SynBlockHandle) -> c_int;

    /// Get b_syn_topgrp (for :syntax include)
    fn nvim_synblock_get_topgrp(block: SynBlockHandle) -> c_int;

    /// Get b_syn_conceal (auto-conceal for :syn cmds)
    fn nvim_synblock_get_conceal(block: SynBlockHandle) -> c_int;

    /// Get b_syn_folditems (number of patterns with HL_FOLD)
    fn nvim_synblock_get_folditems(block: SynBlockHandle) -> c_int;

    /// Get b_sst_len (number of entries in b_sst_array)
    fn nvim_synblock_get_sst_len(block: SynBlockHandle) -> c_int;

    /// Get b_sst_freecount (number of free entries)
    fn nvim_synblock_get_sst_freecount(block: SynBlockHandle) -> c_int;

    /// Get b_sst_check_lnum (entries after this need to be checked)
    fn nvim_synblock_get_sst_check_lnum(block: SynBlockHandle) -> c_int;

    /// Get b_syn_error (true when error occurred in HL)
    fn nvim_synblock_get_syn_error(block: SynBlockHandle) -> c_int;

    /// Get b_syn_slow (true when 'redrawtime' reached)
    fn nvim_synblock_get_syn_slow(block: SynBlockHandle) -> c_int;

    /// Get b_spell_cluster_id (@Spell cluster ID or 0)
    fn nvim_synblock_get_spell_cluster_id(block: SynBlockHandle) -> c_int;

    /// Get b_nospell_cluster_id (@NoSpell cluster ID or 0)
    fn nvim_synblock_get_nospell_cluster_id(block: SynBlockHandle) -> c_int;

    /// Get b_sst_first (first used entry in state array)
    fn nvim_synblock_get_sst_first(block: SynBlockHandle) -> SynStateHandle;

    /// Get b_sst_firstfree (first free entry in state array)
    fn nvim_synblock_get_sst_firstfree(block: SynBlockHandle) -> SynStateHandle;

    /// Check if b_sst_array is allocated
    fn nvim_synblock_has_sst_array(block: SynBlockHandle) -> c_int;

    /// Get synpat_T at index from b_syn_patterns
    fn nvim_synblock_get_pattern(block: SynBlockHandle, idx: c_int) -> SynPatHandle;

    /// Get syn_cluster_T at index from b_syn_clusters
    fn nvim_synblock_get_cluster(block: SynBlockHandle, idx: c_int) -> SynClusterHandle;

    // -------------------------------------------------------------------------
    // synstate_T accessors (syntax state)
    // -------------------------------------------------------------------------

    /// Get sst_next (next entry in used or free list)
    fn nvim_synstate_get_next(state: SynStateHandle) -> SynStateHandle;

    /// Get sst_lnum (line number for this state)
    fn nvim_synstate_get_lnum(state: SynStateHandle) -> c_int;

    /// Get sst_stacksize (number of states on the stack)
    fn nvim_synstate_get_stacksize(state: SynStateHandle) -> c_int;

    /// Get sst_next_flags (flags for sst_next_list)
    fn nvim_synstate_get_next_flags(state: SynStateHandle) -> c_int;

    /// Get sst_tick (tick when last displayed)
    fn nvim_synstate_get_tick(state: SynStateHandle) -> c_int;

    /// Get sst_change_lnum (line where change may have invalidated state)
    fn nvim_synstate_get_change_lnum(state: SynStateHandle) -> c_int;

    // -------------------------------------------------------------------------
    // synpat_T accessors (syntax pattern)
    // -------------------------------------------------------------------------

    /// Get sp_type (SPTYPE_* values)
    fn nvim_synpat_get_type(pat: SynPatHandle) -> c_int;

    /// Get sp_syncing (this item used for syncing)
    fn nvim_synpat_get_syncing(pat: SynPatHandle) -> c_int;

    /// Get sp_syn_match_id (highlight group ID of pattern)
    fn nvim_synpat_get_syn_match_id(pat: SynPatHandle) -> i16;

    /// Get sp_off_flags (offset flags)
    fn nvim_synpat_get_off_flags(pat: SynPatHandle) -> i16;

    /// Get sp_flags (HL_ flags)
    fn nvim_synpat_get_flags(pat: SynPatHandle) -> c_int;

    /// Get sp_cchar (conceal substitute character)
    fn nvim_synpat_get_cchar(pat: SynPatHandle) -> c_int;

    /// Get sp_ic (ignore-case flag for sp_prog)
    fn nvim_synpat_get_ic(pat: SynPatHandle) -> c_int;

    /// Get sp_sync_idx (sync item index, syncing only)
    fn nvim_synpat_get_sync_idx(pat: SynPatHandle) -> c_int;

    /// Get sp_pattern (pattern string)
    fn nvim_synpat_get_pattern(pat: SynPatHandle) -> *const c_char;

    /// Get sp_syn.id (highlight group ID)
    fn nvim_synpat_get_syn_id(pat: SynPatHandle) -> i16;

    /// Get sp_syn.inc_tag (include tag)
    fn nvim_synpat_get_syn_inc_tag(pat: SynPatHandle) -> c_int;

    // -------------------------------------------------------------------------
    // syn_cluster_T accessors (syntax cluster)
    // -------------------------------------------------------------------------

    /// Get scl_name (cluster name)
    fn nvim_syncluster_get_name(cluster: SynClusterHandle) -> *const c_char;

    /// Get scl_name_u (uppercase cluster name)
    fn nvim_syncluster_get_name_u(cluster: SynClusterHandle) -> *const c_char;

    // -------------------------------------------------------------------------
    // stateitem_T accessors (current state item)
    // -------------------------------------------------------------------------

    /// Get si_idx (index of syntax pattern or KEYWORD_IDX)
    fn nvim_stateitem_get_idx(item: StateItemHandle) -> c_int;

    /// Get si_id (highlight group ID for keywords)
    fn nvim_stateitem_get_id(item: StateItemHandle) -> c_int;

    /// Get si_trans_id (highlight group ID, transparency removed)
    fn nvim_stateitem_get_trans_id(item: StateItemHandle) -> c_int;

    /// Get si_m_lnum (lnum of the match)
    fn nvim_stateitem_get_m_lnum(item: StateItemHandle) -> c_int;

    /// Get si_m_startcol (starting column of the match)
    fn nvim_stateitem_get_m_startcol(item: StateItemHandle) -> c_int;

    /// Get si_attr (attributes in this state)
    fn nvim_stateitem_get_attr(item: StateItemHandle) -> c_int;

    /// Get si_flags (HL_ flags and skip flags)
    fn nvim_stateitem_get_flags(item: StateItemHandle) -> c_int;

    /// Get si_seqnr (sequence number)
    fn nvim_stateitem_get_seqnr(item: StateItemHandle) -> c_int;

    /// Get si_cchar (substitution character for conceal)
    fn nvim_stateitem_get_cchar(item: StateItemHandle) -> c_int;

    /// Get si_end_idx (group ID for end pattern or zero)
    fn nvim_stateitem_get_end_idx(item: StateItemHandle) -> c_int;

    /// Get si_ends (if match ends before si_m_endpos)
    fn nvim_stateitem_get_ends(item: StateItemHandle) -> c_int;

    // -------------------------------------------------------------------------
    // keyentry_T accessors (keyword entry)
    // -------------------------------------------------------------------------

    /// Get ke_next (next entry with identical keyword)
    fn nvim_keyentry_get_next(ke: KeyEntryHandle) -> KeyEntryHandle;

    /// Get k_syn.id (highlight group ID)
    fn nvim_keyentry_get_syn_id(ke: KeyEntryHandle) -> i16;

    /// Get k_syn.inc_tag (include tag)
    fn nvim_keyentry_get_syn_inc_tag(ke: KeyEntryHandle) -> c_int;

    /// Get flags
    fn nvim_keyentry_get_flags(ke: KeyEntryHandle) -> c_int;

    /// Get k_char (conceal substitute character)
    fn nvim_keyentry_get_char(ke: KeyEntryHandle) -> c_int;

    /// Get keyword string
    fn nvim_keyentry_get_keyword(ke: KeyEntryHandle) -> *const c_char;

    // -------------------------------------------------------------------------
    // Syntax state global accessors
    // -------------------------------------------------------------------------

    /// Get the current line number being processed
    fn nvim_syn_get_current_lnum() -> c_int;

    /// Get the current column being processed
    fn nvim_syn_get_current_col() -> c_int;

    /// Check if the current line has been finished
    fn nvim_syn_is_current_finished() -> c_int;

    /// Check if the current state has been stored
    fn nvim_syn_is_current_state_stored() -> c_int;

    /// Get the current state stack size
    fn nvim_syn_get_current_state_len() -> c_int;

    /// Check if the current state is valid
    fn nvim_syn_is_current_state_valid() -> c_int;

    /// Get the current highlight ID
    fn nvim_syn_get_current_id() -> c_int;

    /// Get the current transparent ID
    fn nvim_syn_get_current_trans_id() -> c_int;

    /// Get the current attribute
    fn nvim_syn_get_current_attr() -> c_int;

    /// Get the current flags
    fn nvim_syn_get_current_flags() -> c_int;

    /// Get the current sequence number
    fn nvim_syn_get_current_seqnr() -> c_int;

    /// Get the current substitution character
    fn nvim_syn_get_current_sub_char() -> c_int;

    /// Get the current next flags
    fn nvim_syn_get_current_next_flags() -> c_int;

    /// Get the keepend level
    fn nvim_syn_get_keepend_level() -> c_int;

    /// Get state item at index (NULL if out of bounds)
    fn nvim_syn_get_cur_state(idx: c_int) -> StateItemHandle;

    /// Get the current synblock
    fn nvim_syn_get_synblock() -> SynBlockHandle;

    /// Count items with HL_FOLD flag in current state
    fn nvim_syn_count_fold_items() -> c_int;

    // -------------------------------------------------------------------------
    // Phase 4: Pattern matching accessors
    // -------------------------------------------------------------------------

    /// Get sp_prog (compiled regex program)
    fn nvim_synpat_get_prog(pat: SynPatHandle) -> RegProgHandle;

    /// Check if pattern has a compiled program
    fn nvim_synpat_has_prog(pat: SynPatHandle) -> c_int;

    /// Get sp_cont_list (contains list)
    fn nvim_synpat_get_cont_list(pat: SynPatHandle) -> IdListHandle;

    /// Get sp_next_list (nextgroup list)
    fn nvim_synpat_get_next_list(pat: SynPatHandle) -> IdListHandle;

    /// Get sp_syn.cont_in_list (containedin list)
    fn nvim_synpat_get_cont_in_list(pat: SynPatHandle) -> IdListHandle;

    /// Check if pattern has a contains list
    fn nvim_synpat_has_cont_list(pat: SynPatHandle) -> c_int;

    /// Check if pattern has a nextgroup list
    fn nvim_synpat_has_next_list(pat: SynPatHandle) -> c_int;

    /// Check if pattern has a containedin list
    fn nvim_synpat_has_cont_in_list(pat: SynPatHandle) -> c_int;

    // -------------------------------------------------------------------------
    // Phase 4: Keyword hashtable accessors
    // -------------------------------------------------------------------------

    /// Check if synblock has matching-case keywords
    fn nvim_synblock_has_keywords(block: SynBlockHandle) -> c_int;

    /// Check if synblock has ignore-case keywords
    fn nvim_synblock_has_keywords_ic(block: SynBlockHandle) -> c_int;

    /// Get count of matching-case keywords
    fn nvim_synblock_keywtab_count(block: SynBlockHandle) -> usize;

    /// Get count of ignore-case keywords
    fn nvim_synblock_keywtab_ic_count(block: SynBlockHandle) -> usize;

    // -------------------------------------------------------------------------
    // Phase 4: Keyentry list accessors
    // -------------------------------------------------------------------------

    /// Get ke_next_list (nextgroup list for keyword)
    fn nvim_keyentry_get_next_list(ke: KeyEntryHandle) -> IdListHandle;

    /// Get k_syn.cont_in_list (containedin list for keyword)
    fn nvim_keyentry_get_cont_in_list(ke: KeyEntryHandle) -> IdListHandle;

    /// Check if keyword has a nextgroup list
    fn nvim_keyentry_has_next_list(ke: KeyEntryHandle) -> c_int;

    /// Check if keyword has a containedin list
    fn nvim_keyentry_has_cont_in_list(ke: KeyEntryHandle) -> c_int;

    // -------------------------------------------------------------------------
    // Phase 4: Cluster list accessors
    // -------------------------------------------------------------------------

    /// Get scl_list (cluster contains list)
    fn nvim_syncluster_get_list(cluster: SynClusterHandle) -> IdListHandle;

    /// Check if cluster has a list
    fn nvim_syncluster_has_list(cluster: SynClusterHandle) -> c_int;

    /// Get cluster ID from synblock at index
    fn nvim_synblock_get_cluster_id(block: SynBlockHandle, idx: c_int) -> c_int;

    // -------------------------------------------------------------------------
    // Phase 4: ID list iteration helpers
    // -------------------------------------------------------------------------

    /// Get first item in an ID list (returns 0 if NULL)
    fn nvim_id_list_first(list: IdListHandle) -> i16;

    /// Get item at index in an ID list
    fn nvim_id_list_get(list: IdListHandle, idx: c_int) -> i16;

    /// Check if list starts with ALLBUT/TOP/CONTAINED marker
    fn nvim_id_list_is_special(list: IdListHandle) -> c_int;

    /// Count items in an ID list
    fn nvim_id_list_count(list: IdListHandle) -> c_int;

    // -------------------------------------------------------------------------
    // Phase 4: Pattern matching state accessors
    // -------------------------------------------------------------------------

    /// Get next_match_idx
    fn nvim_syn_get_next_match_idx() -> c_int;

    /// Get next_match_col
    fn nvim_syn_get_next_match_col() -> c_int;

    /// Check if there is a pending next match
    fn nvim_syn_has_next_match() -> c_int;

    /// Get current_next_list
    fn nvim_syn_get_current_next_list() -> IdListHandle;

    /// Check if there is a current nextgroup list
    fn nvim_syn_has_current_next_list() -> c_int;
}

// =============================================================================
// Syntax state accessors (safe wrappers)
// =============================================================================

/// Get the current line number being processed
#[must_use]
pub fn current_lnum() -> i32 {
    unsafe { nvim_syn_get_current_lnum() }
}

/// Get the current column being processed
#[must_use]
pub fn current_col() -> i32 {
    unsafe { nvim_syn_get_current_col() }
}

/// Check if the current line has been finished
#[must_use]
pub fn is_current_finished() -> bool {
    unsafe { nvim_syn_is_current_finished() != 0 }
}

/// Check if the current state has been stored
#[must_use]
pub fn is_current_state_stored() -> bool {
    unsafe { nvim_syn_is_current_state_stored() != 0 }
}

/// Get the current state stack size
#[must_use]
pub fn current_state_len() -> i32 {
    unsafe { nvim_syn_get_current_state_len() }
}

/// Check if the current state is valid
#[must_use]
pub fn is_current_state_valid() -> bool {
    unsafe { nvim_syn_is_current_state_valid() != 0 }
}

/// Get the current highlight ID
#[must_use]
pub fn current_id() -> i32 {
    unsafe { nvim_syn_get_current_id() }
}

/// Get the current transparent ID
#[must_use]
pub fn current_trans_id() -> i32 {
    unsafe { nvim_syn_get_current_trans_id() }
}

/// Get the current attribute
#[must_use]
pub fn current_attr() -> i32 {
    unsafe { nvim_syn_get_current_attr() }
}

/// Get the current flags
#[must_use]
pub fn current_flags() -> i32 {
    unsafe { nvim_syn_get_current_flags() }
}

/// Get the current sequence number
#[must_use]
pub fn current_seqnr() -> i32 {
    unsafe { nvim_syn_get_current_seqnr() }
}

/// Get the current substitution character
#[must_use]
pub fn current_sub_char() -> i32 {
    unsafe { nvim_syn_get_current_sub_char() }
}

/// Get the current next flags
#[must_use]
pub fn current_next_flags() -> i32 {
    unsafe { nvim_syn_get_current_next_flags() }
}

/// Get the keepend level (-1 if no keepend on stack)
#[must_use]
pub fn keepend_level() -> i32 {
    unsafe { nvim_syn_get_keepend_level() }
}

/// Get state item at index (None if out of bounds or state invalid)
#[must_use]
pub fn get_cur_state(idx: i32) -> Option<StateItemHandle> {
    let handle = unsafe { nvim_syn_get_cur_state(idx) };
    if handle.is_null() {
        None
    } else {
        Some(handle)
    }
}

/// Get the current synblock (None if not set)
#[must_use]
pub fn get_synblock() -> Option<SynBlockHandle> {
    let handle = unsafe { nvim_syn_get_synblock() };
    if handle.is_null() {
        None
    } else {
        Some(handle)
    }
}

/// Count items with HL_FOLD flag in current state
#[must_use]
pub fn count_fold_items() -> i32 {
    unsafe { nvim_syn_count_fold_items() }
}

// =============================================================================
// Safe Rust wrappers for accessor functions
// =============================================================================

/// Get the number of syntax patterns in a block
#[must_use]
pub fn synblock_pattern_count(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_pattern_count(block) }
}

/// Get the number of syntax clusters in a block
#[must_use]
pub fn synblock_cluster_count(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_cluster_count(block) }
}

/// Check if the block uses ignore-case for :syn commands
#[must_use]
pub fn synblock_is_ignore_case(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_get_syn_ic(block) != 0 }
}

/// Get the spell checking mode for the block
#[must_use]
pub fn synblock_spell_mode(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return SYNSPL_DEFAULT;
    }
    unsafe { nvim_synblock_get_syn_spell(block) }
}

/// Check if any item has a containedin argument
#[must_use]
pub fn synblock_has_containedin(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_get_containedin(block) != 0 }
}

/// Get the first used state in the state array
#[must_use]
pub fn synblock_first_state(block: SynBlockHandle) -> SynStateHandle {
    if block.is_null() {
        return SynStateHandle::null();
    }
    unsafe { nvim_synblock_get_sst_first(block) }
}

/// Check if the block has an error
#[must_use]
pub fn synblock_has_error(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_get_syn_error(block) != 0 }
}

/// Check if the block is slow (redrawtime reached)
#[must_use]
pub fn synblock_is_slow(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_get_syn_slow(block) != 0 }
}

/// Get the line number for a syntax state
#[must_use]
pub fn synstate_lnum(state: SynStateHandle) -> i32 {
    if state.is_null() {
        return 0;
    }
    unsafe { nvim_synstate_get_lnum(state) }
}

/// Get the stack size for a syntax state
#[must_use]
pub fn synstate_stacksize(state: SynStateHandle) -> i32 {
    if state.is_null() {
        return 0;
    }
    unsafe { nvim_synstate_get_stacksize(state) }
}

/// Get the next state in the list
#[must_use]
pub fn synstate_next(state: SynStateHandle) -> SynStateHandle {
    if state.is_null() {
        return SynStateHandle::null();
    }
    unsafe { nvim_synstate_get_next(state) }
}

/// Check if a state change may have invalidated the state
#[must_use]
pub fn synstate_is_valid(state: SynStateHandle) -> bool {
    if state.is_null() {
        return false;
    }
    unsafe { nvim_synstate_get_change_lnum(state) == 0 }
}

/// Get the pattern type (SPTYPE_* value)
#[must_use]
pub fn synpat_type(pat: SynPatHandle) -> i32 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_type(pat) }
}

/// Get the flags for a pattern
#[must_use]
pub fn synpat_flags(pat: SynPatHandle) -> i32 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_flags(pat) }
}

/// Get the highlight group ID for a pattern
#[must_use]
pub fn synpat_syn_id(pat: SynPatHandle) -> i16 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_syn_id(pat) }
}

/// Check if a pattern is for syncing
#[must_use]
pub fn synpat_is_syncing(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    unsafe { nvim_synpat_get_syncing(pat) != 0 }
}

/// Check if a pattern is transparent
#[must_use]
pub fn synpat_is_transparent(pat: SynPatHandle) -> bool {
    synpat_flags(pat) & HL_TRANSP != 0
}

/// Check if a pattern is contained
#[must_use]
pub fn synpat_is_contained(pat: SynPatHandle) -> bool {
    synpat_flags(pat) & HL_CONTAINED != 0
}

/// Check if a pattern has keepend
#[must_use]
pub fn synpat_has_keepend(pat: SynPatHandle) -> bool {
    synpat_flags(pat) & HL_KEEPEND != 0
}

/// Check if a pattern defines a fold
#[must_use]
pub fn synpat_defines_fold(pat: SynPatHandle) -> bool {
    synpat_flags(pat) & HL_FOLD != 0
}

/// Get the index of a state item
#[must_use]
pub fn stateitem_idx(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_idx(item) }
}

/// Check if a state item is for a keyword
#[must_use]
pub fn stateitem_is_keyword(item: StateItemHandle) -> bool {
    stateitem_idx(item) == KEYWORD_IDX
}

/// Get the highlight group ID for a state item
#[must_use]
pub fn stateitem_id(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_id(item) }
}

/// Get the transparent ID for a state item
#[must_use]
pub fn stateitem_trans_id(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_trans_id(item) }
}

/// Get the attributes for a state item
#[must_use]
pub fn stateitem_attr(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_attr(item) }
}

/// Get the conceal character for a state item
#[must_use]
pub fn stateitem_cchar(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_cchar(item) }
}

// =============================================================================
// Phase 4: Pattern matching safe wrappers
// =============================================================================

/// Check if a pattern has a compiled regex program
#[must_use]
pub fn synpat_has_prog(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    unsafe { nvim_synpat_has_prog(pat) != 0 }
}

/// Get the compiled regex program for a pattern
#[must_use]
pub fn synpat_prog(pat: SynPatHandle) -> Option<RegProgHandle> {
    if pat.is_null() {
        return None;
    }
    let prog = unsafe { nvim_synpat_get_prog(pat) };
    if prog.is_null() {
        None
    } else {
        Some(prog)
    }
}

/// Check if a pattern has a contains list
#[must_use]
pub fn synpat_has_contains(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    unsafe { nvim_synpat_has_cont_list(pat) != 0 }
}

/// Get the contains list for a pattern
#[must_use]
pub fn synpat_contains_list(pat: SynPatHandle) -> Option<IdListHandle> {
    if pat.is_null() {
        return None;
    }
    let list = unsafe { nvim_synpat_get_cont_list(pat) };
    if list.is_null() {
        None
    } else {
        Some(list)
    }
}

/// Check if a pattern has a nextgroup list
#[must_use]
pub fn synpat_has_nextgroup(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    unsafe { nvim_synpat_has_next_list(pat) != 0 }
}

/// Get the nextgroup list for a pattern
#[must_use]
pub fn synpat_nextgroup_list(pat: SynPatHandle) -> Option<IdListHandle> {
    if pat.is_null() {
        return None;
    }
    let list = unsafe { nvim_synpat_get_next_list(pat) };
    if list.is_null() {
        None
    } else {
        Some(list)
    }
}

/// Check if a pattern has a containedin list
#[must_use]
pub fn synpat_has_containedin(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    unsafe { nvim_synpat_has_cont_in_list(pat) != 0 }
}

/// Get the containedin list for a pattern
#[must_use]
pub fn synpat_containedin_list(pat: SynPatHandle) -> Option<IdListHandle> {
    if pat.is_null() {
        return None;
    }
    let list = unsafe { nvim_synpat_get_cont_in_list(pat) };
    if list.is_null() {
        None
    } else {
        Some(list)
    }
}

// =============================================================================
// Phase 4: Keyword safe wrappers
// =============================================================================

/// Check if a synblock has matching-case keywords
#[must_use]
pub fn synblock_has_keywords(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_has_keywords(block) != 0 }
}

/// Check if a synblock has ignore-case keywords
#[must_use]
pub fn synblock_has_keywords_ic(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_has_keywords_ic(block) != 0 }
}

/// Get the count of matching-case keywords
#[must_use]
pub fn synblock_keyword_count(block: SynBlockHandle) -> usize {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_keywtab_count(block) }
}

/// Get the count of ignore-case keywords
#[must_use]
pub fn synblock_keyword_count_ic(block: SynBlockHandle) -> usize {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_keywtab_ic_count(block) }
}

/// Check if a keyword entry has a nextgroup list
#[must_use]
pub fn keyentry_has_nextgroup(ke: KeyEntryHandle) -> bool {
    if ke.is_null() {
        return false;
    }
    unsafe { nvim_keyentry_has_next_list(ke) != 0 }
}

/// Get the nextgroup list for a keyword
#[must_use]
pub fn keyentry_nextgroup_list(ke: KeyEntryHandle) -> Option<IdListHandle> {
    if ke.is_null() {
        return None;
    }
    let list = unsafe { nvim_keyentry_get_next_list(ke) };
    if list.is_null() {
        None
    } else {
        Some(list)
    }
}

/// Check if a keyword entry has a containedin list
#[must_use]
pub fn keyentry_has_containedin(ke: KeyEntryHandle) -> bool {
    if ke.is_null() {
        return false;
    }
    unsafe { nvim_keyentry_has_cont_in_list(ke) != 0 }
}

/// Get the containedin list for a keyword
#[must_use]
pub fn keyentry_containedin_list(ke: KeyEntryHandle) -> Option<IdListHandle> {
    if ke.is_null() {
        return None;
    }
    let list = unsafe { nvim_keyentry_get_cont_in_list(ke) };
    if list.is_null() {
        None
    } else {
        Some(list)
    }
}

// =============================================================================
// Phase 4: Cluster safe wrappers
// =============================================================================

/// Check if a cluster has a contains list
#[must_use]
pub fn syncluster_has_list(cluster: SynClusterHandle) -> bool {
    if cluster.is_null() {
        return false;
    }
    unsafe { nvim_syncluster_has_list(cluster) != 0 }
}

/// Get the contains list for a cluster
#[must_use]
pub fn syncluster_list(cluster: SynClusterHandle) -> Option<IdListHandle> {
    if cluster.is_null() {
        return None;
    }
    let list = unsafe { nvim_syncluster_get_list(cluster) };
    if list.is_null() {
        None
    } else {
        Some(list)
    }
}

/// Get the cluster ID at an index in a synblock
#[must_use]
pub fn synblock_cluster_id(block: SynBlockHandle, idx: i32) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_cluster_id(block, idx) }
}

// =============================================================================
// Phase 4: ID list safe wrappers
// =============================================================================

/// Get the first item in an ID list
#[must_use]
pub fn id_list_first(list: IdListHandle) -> i16 {
    if list.is_null() {
        return 0;
    }
    unsafe { nvim_id_list_first(list) }
}

/// Get an item at index in an ID list
#[must_use]
pub fn id_list_get(list: IdListHandle, idx: i32) -> i16 {
    if list.is_null() {
        return 0;
    }
    unsafe { nvim_id_list_get(list, idx) }
}

/// Check if an ID list starts with a special marker (ALLBUT/TOP/CONTAINED)
#[must_use]
pub fn id_list_is_special(list: IdListHandle) -> bool {
    if list.is_null() {
        return false;
    }
    unsafe { nvim_id_list_is_special(list) != 0 }
}

/// Count the number of items in an ID list
#[must_use]
pub fn id_list_count(list: IdListHandle) -> i32 {
    if list.is_null() {
        return 0;
    }
    unsafe { nvim_id_list_count(list) }
}

// =============================================================================
// Phase 4: Pattern matching state safe wrappers
// =============================================================================

/// Get the index of the next pattern to match
#[must_use]
pub fn next_match_idx() -> i32 {
    unsafe { nvim_syn_get_next_match_idx() }
}

/// Get the column where the next match starts
#[must_use]
pub fn next_match_col() -> i32 {
    unsafe { nvim_syn_get_next_match_col() }
}

/// Check if there is a pending next match
#[must_use]
pub fn has_next_match() -> bool {
    unsafe { nvim_syn_has_next_match() != 0 }
}

/// Get the current nextgroup list
#[must_use]
pub fn current_next_list() -> Option<IdListHandle> {
    let list = unsafe { nvim_syn_get_current_next_list() };
    if list.is_null() {
        None
    } else {
        Some(list)
    }
}

/// Check if there is a current nextgroup list
#[must_use]
pub fn has_current_next_list() -> bool {
    unsafe { nvim_syn_has_current_next_list() != 0 }
}

// =============================================================================
// FFI exports - Syntax state checking
// =============================================================================

/// Check if syntax highlighting is enabled (block has patterns)
///
/// # Safety
/// `block` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_syntax_block_has_patterns(block: SynBlockHandle) -> c_int {
    if synblock_pattern_count(block) > 0 {
        1
    } else {
        0
    }
}

/// Check if the syntax block has any clusters defined
///
/// # Safety
/// `block` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_syntax_block_has_clusters(block: SynBlockHandle) -> c_int {
    if synblock_cluster_count(block) > 0 {
        1
    } else {
        0
    }
}

/// Check if the syntax block has any items that define folds
///
/// # Safety
/// `block` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_syntax_block_has_folds(block: SynBlockHandle) -> c_int {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_folditems(block) }
}

/// Check if a synstate is valid (not invalidated by changes)
///
/// # Safety
/// `state` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synstate_is_valid(state: SynStateHandle) -> c_int {
    if synstate_is_valid(state) {
        1
    } else {
        0
    }
}

/// Get the line number for a synstate
///
/// # Safety
/// `state` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synstate_get_lnum(state: SynStateHandle) -> c_int {
    synstate_lnum(state)
}

/// Check if a pattern defines a fold
///
/// # Safety
/// `pat` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synpat_defines_fold(pat: SynPatHandle) -> c_int {
    if synpat_defines_fold(pat) {
        1
    } else {
        0
    }
}

/// Check if a pattern is transparent
///
/// # Safety
/// `pat` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synpat_is_transparent(pat: SynPatHandle) -> c_int {
    if synpat_is_transparent(pat) {
        1
    } else {
        0
    }
}

/// Check if a pattern is contained
///
/// # Safety
/// `pat` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synpat_is_contained(pat: SynPatHandle) -> c_int {
    if synpat_is_contained(pat) {
        1
    } else {
        0
    }
}

/// Check if a pattern has keepend
///
/// # Safety
/// `pat` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synpat_has_keepend(pat: SynPatHandle) -> c_int {
    if synpat_has_keepend(pat) {
        1
    } else {
        0
    }
}

/// Get the pattern type (SPTYPE_*)
///
/// # Safety
/// `pat` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synpat_get_type(pat: SynPatHandle) -> c_int {
    synpat_type(pat)
}

/// Get the highlight group ID for a pattern
///
/// # Safety
/// `pat` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synpat_get_syn_id(pat: SynPatHandle) -> c_int {
    c_int::from(synpat_syn_id(pat))
}

/// Check if a state item is for a keyword
///
/// # Safety
/// `item` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_stateitem_is_keyword(item: StateItemHandle) -> c_int {
    if stateitem_is_keyword(item) {
        1
    } else {
        0
    }
}

/// Get the highlight group ID for a state item
///
/// # Safety
/// `item` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_stateitem_get_id(item: StateItemHandle) -> c_int {
    stateitem_id(item)
}

/// Get the transparent highlight group ID for a state item
///
/// # Safety
/// `item` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_stateitem_get_trans_id(item: StateItemHandle) -> c_int {
    stateitem_trans_id(item)
}

/// Get the conceal character for a state item
///
/// # Safety
/// `item` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_stateitem_get_cchar(item: StateItemHandle) -> c_int {
    stateitem_cchar(item)
}

// =============================================================================
// FFI exports - Syntax ID helpers
// =============================================================================

/// Check if a syntax ID is a cluster reference
#[no_mangle]
pub extern "C" fn rs_is_cluster_id(id: i16) -> c_int {
    if is_cluster_id(id) {
        1
    } else {
        0
    }
}

/// Check if a syntax ID is a special group (ALLBUT, TOP, CONTAINED, or Cluster)
#[no_mangle]
pub extern "C" fn rs_is_special_id(id: i16) -> c_int {
    if is_special_id(id) {
        1
    } else {
        0
    }
}

/// Check if a syntax ID is a normal syntax group
#[no_mangle]
pub extern "C" fn rs_is_normal_id(id: i16) -> c_int {
    if is_normal_id(id) {
        1
    } else {
        0
    }
}

/// Get the cluster index from a cluster ID
/// Returns -1 if not a cluster ID
#[no_mangle]
pub extern "C" fn rs_get_cluster_index(id: i16) -> c_int {
    cluster_index(id).map_or(-1, c_int::from)
}

/// Create a cluster ID from a cluster index
#[no_mangle]
pub extern "C" fn rs_make_cluster_id(index: i16) -> i16 {
    make_cluster_id(index)
}

/// Get the syntax ID type as an integer
/// 0 = Normal, 1 = AllBut, 2 = Top, 3 = Contained, 4 = Cluster
#[no_mangle]
pub extern "C" fn rs_synid_type(id: i16) -> c_int {
    match synid_type(id) {
        SynIdType::Normal => 0,
        SynIdType::AllBut => 1,
        SynIdType::Top => 2,
        SynIdType::Contained => 3,
        SynIdType::Cluster => 4,
    }
}

// =============================================================================
// FFI exports - Syntax state machine accessors
// =============================================================================

/// Get the current line number being processed
#[no_mangle]
pub extern "C" fn rs_syn_current_lnum() -> c_int {
    current_lnum()
}

/// Get the current column being processed
#[no_mangle]
pub extern "C" fn rs_syn_current_col() -> c_int {
    current_col()
}

/// Check if the current line has been finished
#[no_mangle]
pub extern "C" fn rs_syn_is_finished() -> c_int {
    if is_current_finished() {
        1
    } else {
        0
    }
}

/// Check if the current state is valid
#[no_mangle]
pub extern "C" fn rs_syn_is_state_valid() -> c_int {
    if is_current_state_valid() {
        1
    } else {
        0
    }
}

/// Get the current state stack size
#[no_mangle]
pub extern "C" fn rs_syn_state_len() -> c_int {
    current_state_len()
}

/// Get the current highlight ID
#[no_mangle]
pub extern "C" fn rs_syn_current_id() -> c_int {
    current_id()
}

/// Get the current transparent ID
#[no_mangle]
pub extern "C" fn rs_syn_current_trans_id() -> c_int {
    current_trans_id()
}

/// Get the current attribute
#[no_mangle]
pub extern "C" fn rs_syn_current_attr() -> c_int {
    current_attr()
}

/// Get the current flags
#[no_mangle]
pub extern "C" fn rs_syn_current_flags() -> c_int {
    current_flags()
}

/// Get the keepend level (-1 if none)
#[no_mangle]
pub extern "C" fn rs_syn_keepend_level() -> c_int {
    keepend_level()
}

/// Count items with HL_FOLD flag in current state
/// This implements the logic of syn_cur_foldlevel() in Rust
#[no_mangle]
pub extern "C" fn rs_syn_cur_foldlevel() -> c_int {
    count_fold_items()
}

/// Get a state item from the current state stack
/// Returns NULL if index is out of bounds
#[no_mangle]
pub extern "C" fn rs_syn_get_state_item(idx: c_int) -> StateItemHandle {
    get_cur_state(idx).unwrap_or(StateItemHandle(std::ptr::null_mut()))
}

// =============================================================================
// FFI exports - Phase 4: Pattern matching
// =============================================================================

/// Check if a pattern has a compiled regex program
///
/// # Safety
/// `pat` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synpat_has_prog(pat: SynPatHandle) -> c_int {
    if synpat_has_prog(pat) {
        1
    } else {
        0
    }
}

/// Check if a pattern has a contains list
///
/// # Safety
/// `pat` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synpat_has_contains(pat: SynPatHandle) -> c_int {
    if synpat_has_contains(pat) {
        1
    } else {
        0
    }
}

/// Check if a pattern has a nextgroup list
///
/// # Safety
/// `pat` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synpat_has_nextgroup(pat: SynPatHandle) -> c_int {
    if synpat_has_nextgroup(pat) {
        1
    } else {
        0
    }
}

/// Check if a pattern has a containedin list
///
/// # Safety
/// `pat` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synpat_has_containedin(pat: SynPatHandle) -> c_int {
    if synpat_has_containedin(pat) {
        1
    } else {
        0
    }
}

/// Check if a synblock has matching-case keywords
///
/// # Safety
/// `block` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synblock_has_keywords(block: SynBlockHandle) -> c_int {
    if synblock_has_keywords(block) {
        1
    } else {
        0
    }
}

/// Check if a synblock has ignore-case keywords
///
/// # Safety
/// `block` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synblock_has_keywords_ic(block: SynBlockHandle) -> c_int {
    if synblock_has_keywords_ic(block) {
        1
    } else {
        0
    }
}

/// Get the count of matching-case keywords
///
/// # Safety
/// `block` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synblock_keyword_count(block: SynBlockHandle) -> usize {
    synblock_keyword_count(block)
}

/// Get the count of ignore-case keywords
///
/// # Safety
/// `block` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synblock_keyword_count_ic(block: SynBlockHandle) -> usize {
    synblock_keyword_count_ic(block)
}

/// Check if a keyword entry has a nextgroup list
///
/// # Safety
/// `ke` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_keyentry_has_nextgroup(ke: KeyEntryHandle) -> c_int {
    if keyentry_has_nextgroup(ke) {
        1
    } else {
        0
    }
}

/// Check if a keyword entry has a containedin list
///
/// # Safety
/// `ke` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_keyentry_has_containedin(ke: KeyEntryHandle) -> c_int {
    if keyentry_has_containedin(ke) {
        1
    } else {
        0
    }
}

/// Check if a cluster has a contains list
///
/// # Safety
/// `cluster` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_syncluster_has_list(cluster: SynClusterHandle) -> c_int {
    if syncluster_has_list(cluster) {
        1
    } else {
        0
    }
}

/// Get the cluster ID at an index in a synblock
///
/// # Safety
/// `block` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synblock_cluster_id(block: SynBlockHandle, idx: c_int) -> c_int {
    synblock_cluster_id(block, idx)
}

/// Get the first item in an ID list
#[no_mangle]
pub extern "C" fn rs_id_list_first(list: IdListHandle) -> i16 {
    id_list_first(list)
}

/// Get an item at index in an ID list
#[no_mangle]
pub extern "C" fn rs_id_list_get(list: IdListHandle, idx: c_int) -> i16 {
    id_list_get(list, idx)
}

/// Check if an ID list starts with a special marker (ALLBUT/TOP/CONTAINED)
#[no_mangle]
pub extern "C" fn rs_id_list_is_special(list: IdListHandle) -> c_int {
    if id_list_is_special(list) {
        1
    } else {
        0
    }
}

/// Count the number of items in an ID list
#[no_mangle]
pub extern "C" fn rs_id_list_count(list: IdListHandle) -> c_int {
    id_list_count(list)
}

/// Get the index of the next pattern to match
#[no_mangle]
pub extern "C" fn rs_syn_next_match_idx() -> c_int {
    next_match_idx()
}

/// Get the column where the next match starts
#[no_mangle]
pub extern "C" fn rs_syn_next_match_col() -> c_int {
    next_match_col()
}

/// Check if there is a pending next match
#[no_mangle]
pub extern "C" fn rs_syn_has_next_match() -> c_int {
    if has_next_match() {
        1
    } else {
        0
    }
}

/// Check if there is a current nextgroup list
#[no_mangle]
pub extern "C" fn rs_syn_has_current_next_list() -> c_int {
    if has_current_next_list() {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_constants() {
        // Verify highlight flags are powers of 2 and non-overlapping
        assert_eq!(HL_CONTAINED, 0x01);
        assert_eq!(HL_TRANSP, 0x02);
        assert_eq!(HL_ONELINE, 0x04);
        assert_eq!(HL_HAS_EOL, 0x08);
        assert_eq!(HL_FOLD, 0x2000);
        assert_eq!(HL_CONCEAL, 0x2_0000);
        assert_eq!(HL_INCLUDED_TOPLEVEL, 0x8_0000);

        // Verify SPTYPE values
        assert_eq!(SPTYPE_MATCH, 1);
        assert_eq!(SPTYPE_START, 2);
        assert_eq!(SPTYPE_END, 3);
        assert_eq!(SPTYPE_SKIP, 4);

        // Verify SYNID ranges
        assert!(SYNID_ALLBUT < SYNID_TOP);
        assert!(SYNID_TOP < SYNID_CONTAINED);
        assert!(SYNID_CONTAINED < SYNID_CLUSTER);

        // Verify state stack constants
        assert!(SST_MIN_ENTRIES < SST_MAX_ENTRIES);
        assert!(SST_FIX_STATES > 0);
        assert!(SST_DIST > 0);

        // Verify special indices
        assert_eq!(KEYWORD_IDX, -1);
        assert_eq!(NONE_IDX, -2);
    }

    #[test]
    fn test_null_handles() {
        // Test that null handles are properly detected
        let null_block = SynBlockHandle(std::ptr::null_mut());
        let null_state = SynStateHandle::null();
        let null_pat = SynPatHandle(std::ptr::null_mut());
        let null_item = StateItemHandle(std::ptr::null_mut());
        let null_cluster = SynClusterHandle(std::ptr::null_mut());
        let null_key = KeyEntryHandle(std::ptr::null_mut());
        let null_prog = RegProgHandle::null();
        let null_idlist = IdListHandle::null();

        assert!(null_block.is_null());
        assert!(null_state.is_null());
        assert!(null_pat.is_null());
        assert!(null_item.is_null());
        assert!(null_cluster.is_null());
        assert!(null_key.is_null());
        assert!(null_prog.is_null());
        assert!(null_idlist.is_null());
    }

    // Note: test_phase4_null_safe_wrappers cannot be run in isolation because
    // the safe wrappers call C FFI functions. These are tested via the full
    // Neovim functional test suite instead.

    #[test]
    fn test_hl_flags_are_distinct() {
        // Verify no flags overlap
        let all_flags = [
            HL_CONTAINED,
            HL_TRANSP,
            HL_ONELINE,
            HL_HAS_EOL,
            HL_SYNC_HERE,
            HL_SYNC_THERE,
            HL_MATCH,
            HL_SKIPNL,
            HL_SKIPWHITE,
            HL_SKIPEMPTY,
            HL_KEEPEND,
            HL_EXCLUDENL,
            HL_DISPLAY,
            HL_FOLD,
            HL_EXTEND,
            HL_MATCHCONT,
            HL_TRANS_CONT,
            HL_CONCEAL,
            HL_CONCEALENDS,
            HL_INCLUDED_TOPLEVEL,
        ];

        for (i, &flag_a) in all_flags.iter().enumerate() {
            for (j, &flag_b) in all_flags.iter().enumerate() {
                if i != j {
                    assert_eq!(
                        flag_a & flag_b,
                        0,
                        "Flags at indices {} and {} overlap",
                        i,
                        j
                    );
                }
            }
        }
    }

    #[test]
    fn test_synid_type_classification() {
        // Normal IDs (0 - 19999)
        assert_eq!(synid_type(0), SynIdType::Normal);
        assert_eq!(synid_type(1), SynIdType::Normal);
        assert_eq!(synid_type(100), SynIdType::Normal);
        assert_eq!(synid_type(19999), SynIdType::Normal);

        // ALLBUT IDs (20000 - 20999)
        assert_eq!(synid_type(20000), SynIdType::AllBut);
        assert_eq!(synid_type(20500), SynIdType::AllBut);
        assert_eq!(synid_type(20999), SynIdType::AllBut);

        // TOP IDs (21000 - 21999)
        assert_eq!(synid_type(21000), SynIdType::Top);
        assert_eq!(synid_type(21500), SynIdType::Top);
        assert_eq!(synid_type(21999), SynIdType::Top);

        // CONTAINED IDs (22000 - 22999)
        assert_eq!(synid_type(22000), SynIdType::Contained);
        assert_eq!(synid_type(22500), SynIdType::Contained);
        assert_eq!(synid_type(22999), SynIdType::Contained);

        // Cluster IDs (23000+)
        assert_eq!(synid_type(23000), SynIdType::Cluster);
        assert_eq!(synid_type(25000), SynIdType::Cluster);
        assert_eq!(synid_type(32767), SynIdType::Cluster);
    }

    #[test]
    fn test_id_classification_helpers() {
        // Test is_cluster_id
        assert!(!is_cluster_id(100));
        assert!(!is_cluster_id(20000));
        assert!(!is_cluster_id(22000));
        assert!(is_cluster_id(23000));
        assert!(is_cluster_id(25000));

        // Test is_special_id
        assert!(!is_special_id(100));
        assert!(!is_special_id(19999));
        assert!(is_special_id(20000)); // ALLBUT
        assert!(is_special_id(21000)); // TOP
        assert!(is_special_id(22000)); // CONTAINED
        assert!(is_special_id(23000)); // Cluster

        // Test is_normal_id
        assert!(!is_normal_id(0)); // 0 is not a valid group
        assert!(is_normal_id(1));
        assert!(is_normal_id(100));
        assert!(is_normal_id(19999));
        assert!(!is_normal_id(20000));
    }

    #[test]
    fn test_cluster_index_extraction() {
        // Non-cluster IDs return None
        assert_eq!(cluster_index(100), None);
        assert_eq!(cluster_index(22000), None);

        // Cluster IDs return the index
        assert_eq!(cluster_index(23000), Some(0));
        assert_eq!(cluster_index(23001), Some(1));
        assert_eq!(cluster_index(23100), Some(100));
        assert_eq!(cluster_index(32767), Some(32767 - 23000));
    }

    #[test]
    fn test_make_cluster_id() {
        assert_eq!(make_cluster_id(0), 23000);
        assert_eq!(make_cluster_id(1), 23001);
        assert_eq!(make_cluster_id(100), 23100);

        // Round-trip test
        for i in 0i16..100 {
            let cluster_id = make_cluster_id(i);
            assert_eq!(cluster_index(cluster_id), Some(i));
        }
    }

    #[test]
    fn test_extract_inc_tag() {
        // Normal IDs return None
        assert_eq!(extract_inc_tag(100), None);

        // Cluster IDs return None
        assert_eq!(extract_inc_tag(23000), None);

        // ALLBUT IDs
        assert_eq!(extract_inc_tag(20000), Some(0));
        assert_eq!(extract_inc_tag(20001), Some(1));
        assert_eq!(extract_inc_tag(20100), Some(100));

        // TOP IDs
        assert_eq!(extract_inc_tag(21000), Some(0));
        assert_eq!(extract_inc_tag(21001), Some(1));
        assert_eq!(extract_inc_tag(21100), Some(100));

        // CONTAINED IDs
        assert_eq!(extract_inc_tag(22000), Some(0));
        assert_eq!(extract_inc_tag(22001), Some(1));
        assert_eq!(extract_inc_tag(22100), Some(100));
    }
}
