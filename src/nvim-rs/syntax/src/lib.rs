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

/// Opaque handle to a bufstate_T (stored state for state stack entry)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct BufStateHandle(*mut std::ffi::c_void);

impl BufStateHandle {
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

/// Opaque handle to a reg_extmatch_T (external match references)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ExtMatchHandle(*mut std::ffi::c_void);

impl ExtMatchHandle {
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

    // -------------------------------------------------------------------------
    // Phase 5: Cluster & containedin logic accessors
    // -------------------------------------------------------------------------

    /// Get cluster ID from a cluster
    fn nvim_syncluster_get_id(cluster: SynClusterHandle) -> c_int;

    // Note: nvim_synblock_get_cluster and nvim_synblock_get_pattern are already
    // declared above in the synblock accessors section

    /// Get the current synblock from curwin->w_s
    fn nvim_syn_get_curwin_synblock() -> SynBlockHandle;

    /// Get the spell cluster ID from a synblock
    fn nvim_synblock_get_spell_cluster(block: SynBlockHandle) -> c_int;

    /// Get the nospell cluster ID from a synblock
    fn nvim_synblock_get_nospell_cluster(block: SynBlockHandle) -> c_int;

    /// Check if a stateitem has the HL_TRANS_CONT flag
    fn nvim_stateitem_has_trans_cont(item: StateItemHandle) -> c_int;

    /// Check if a stateitem has the HL_MATCH flag
    fn nvim_stateitem_has_match(item: StateItemHandle) -> c_int;

    /// Get si_cont_list (containedin list for state item)
    fn nvim_stateitem_get_cont_list(item: StateItemHandle) -> IdListHandle;

    /// Check if stateitem has a containedin list
    fn nvim_stateitem_has_cont_list(item: StateItemHandle) -> c_int;

    // -------------------------------------------------------------------------
    // Phase 6: Command & user interface accessors
    // -------------------------------------------------------------------------

    /// Get the current syntax topgrp (for :syn include)
    fn nvim_syn_get_topgrp() -> c_int;

    /// Set the current syntax topgrp
    fn nvim_syn_set_topgrp(topgrp: c_int);

    /// Get the syntax block's conceal setting
    fn nvim_synblock_get_conceal_setting(block: SynBlockHandle) -> c_int;

    /// Get the syntax block's case ignore setting
    fn nvim_synblock_get_ic_setting(block: SynBlockHandle) -> c_int;

    /// Get the number of subcommands
    fn nvim_syn_get_subcommand_count() -> c_int;

    /// Get subcommand name by index
    fn nvim_syn_get_subcommand_name(idx: c_int) -> *const c_char;

    /// Check if a pattern at index is for syncing
    fn nvim_synblock_pattern_is_syncing(block: SynBlockHandle, idx: c_int) -> c_int;

    /// Get the hl group ID from a pattern (minus 1)
    fn nvim_synpat_get_hl_group(pat: SynPatHandle) -> c_int;

    /// Count patterns with a specific highlight group ID
    fn nvim_synblock_count_patterns_for_id(block: SynBlockHandle, id: c_int) -> c_int;

    /// Get expand_what variable
    fn nvim_syn_get_expand_what() -> c_int;

    /// Set expand_what variable
    fn nvim_syn_set_expand_what(what: c_int);

    // -------------------------------------------------------------------------
    // Phase 24.1: State Management Helpers
    // -------------------------------------------------------------------------

    /// Check if a state item at idx has a position spanning past current line
    fn nvim_syn_state_item_spans_line(idx: c_int, lnum: c_int) -> c_int;

    /// Find a state entry in the synblock at or before given line
    fn nvim_syn_stack_find_entry(lnum: c_int) -> SynStateHandle;

    /// Remove a state entry from the used list and move to free list
    fn nvim_syn_stack_remove_entry(sp: SynStateHandle);

    /// Allocate a new state entry for the given line
    fn nvim_syn_stack_alloc_entry(lnum: c_int, after: SynStateHandle) -> SynStateHandle;

    /// Store the current state into a synstate entry
    fn nvim_syn_store_state_to_entry(sp: SynStateHandle);

    /// Mark current state as stored
    fn nvim_syn_set_state_stored(stored: c_int);

    /// Call clear_current_state()
    fn nvim_syn_clear_current_state();

    /// Call validate_current_state()
    fn nvim_syn_validate_current_state();

    /// Call invalidate_current_state()
    fn nvim_syn_invalidate_current_state();

    /// Set keepend_level
    fn nvim_syn_set_keepend_level(level: c_int);

    /// Grow current_state array
    fn nvim_syn_grow_current_state(size: c_int);

    /// Set current_state.ga_len
    fn nvim_syn_set_current_state_len(len: c_int);

    /// Set current_next_list
    fn nvim_syn_set_current_next_list(list: IdListHandle);

    /// Set current_next_flags
    fn nvim_syn_set_current_next_flags(flags: c_int);

    /// Set current_lnum
    fn nvim_syn_set_current_lnum(lnum: c_int);

    /// Get sst_next_list from a synstate
    fn nvim_synstate_get_next_list(state: SynStateHandle) -> IdListHandle;

    /// Get bufstate item from synstate at index
    fn nvim_synstate_get_bufstate(state: SynStateHandle, idx: c_int) -> BufStateHandle;

    /// Get bs_idx from bufstate
    fn nvim_bufstate_get_idx(bs: BufStateHandle) -> c_int;

    /// Get bs_flags from bufstate
    fn nvim_bufstate_get_flags(bs: BufStateHandle) -> c_int;

    /// Get bs_seqnr from bufstate
    fn nvim_bufstate_get_seqnr(bs: BufStateHandle) -> c_int;

    /// Get bs_cchar from bufstate
    fn nvim_bufstate_get_cchar(bs: BufStateHandle) -> c_int;

    /// Get bs_extmatch from bufstate (opaque pointer)
    fn nvim_bufstate_get_extmatch(bs: BufStateHandle) -> ExtMatchHandle;

    /// Set stateitem fields at index (used by load_current_state)
    fn nvim_syn_set_cur_state_item(
        idx: c_int,
        si_idx: c_int,
        si_flags: c_int,
        si_seqnr: c_int,
        si_cchar: c_int,
        extmatch: ExtMatchHandle,
    );

    /// Call update_si_attr for item at index
    fn nvim_syn_update_si_attr(idx: c_int);

    /// Compare two extmatch pointers (for syn_stack_equal)
    fn nvim_syn_extmatch_equal(a: ExtMatchHandle, b: ExtMatchHandle) -> c_int;

    /// Compare extmatch strings at given sub-index
    fn nvim_syn_extmatch_strings_equal(
        a: ExtMatchHandle,
        b: ExtMatchHandle,
        subidx: c_int,
        pat_idx: c_int,
    ) -> c_int;

    /// Get NSUBEXP constant
    fn nvim_syn_get_nsubexp() -> c_int;

    /// Get the sp_ic (ignore case) flag for a pattern at index
    fn nvim_synblock_pattern_ic(pat_idx: c_int) -> c_int;

    /// Get si_extmatch from a stateitem
    fn nvim_stateitem_get_extmatch(item: StateItemHandle) -> ExtMatchHandle;
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
// Phase 5: Cluster & containedin safe wrappers
// =============================================================================

/// Get the cluster ID from a cluster
#[must_use]
pub fn syncluster_id(cluster: SynClusterHandle) -> i32 {
    if cluster.is_null() {
        return 0;
    }
    unsafe { nvim_syncluster_get_id(cluster) }
}

/// Get a cluster at an index from a synblock
#[must_use]
pub fn synblock_get_cluster(block: SynBlockHandle, idx: i32) -> Option<SynClusterHandle> {
    if block.is_null() {
        return None;
    }
    let cluster = unsafe { nvim_synblock_get_cluster(block, idx) };
    if cluster.is_null() {
        None
    } else {
        Some(cluster)
    }
}

/// Get a pattern at an index from a synblock
#[must_use]
pub fn synblock_get_pattern(block: SynBlockHandle, idx: i32) -> Option<SynPatHandle> {
    if block.is_null() {
        return None;
    }
    let pat = unsafe { nvim_synblock_get_pattern(block, idx) };
    if pat.is_null() {
        None
    } else {
        Some(pat)
    }
}

/// Get the current window's synblock
#[must_use]
pub fn curwin_synblock() -> SynBlockHandle {
    unsafe { nvim_syn_get_curwin_synblock() }
}

/// Get the spell cluster ID from a synblock
#[must_use]
pub fn synblock_spell_cluster(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_spell_cluster(block) }
}

/// Get the nospell cluster ID from a synblock
#[must_use]
pub fn synblock_nospell_cluster(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_nospell_cluster(block) }
}

/// Check if a stateitem has the HL_TRANS_CONT flag
#[must_use]
pub fn stateitem_has_trans_cont(item: StateItemHandle) -> bool {
    if item.is_null() {
        return false;
    }
    unsafe { nvim_stateitem_has_trans_cont(item) != 0 }
}

/// Check if a stateitem has the HL_MATCH flag
#[must_use]
pub fn stateitem_has_match(item: StateItemHandle) -> bool {
    if item.is_null() {
        return false;
    }
    unsafe { nvim_stateitem_has_match(item) != 0 }
}

/// Get the containedin list for a state item
#[must_use]
pub fn stateitem_cont_list(item: StateItemHandle) -> Option<IdListHandle> {
    if item.is_null() {
        return None;
    }
    let list = unsafe { nvim_stateitem_get_cont_list(item) };
    if list.is_null() {
        None
    } else {
        Some(list)
    }
}

/// Check if a stateitem has a containedin list
#[must_use]
pub fn stateitem_has_cont_list(item: StateItemHandle) -> bool {
    if item.is_null() {
        return false;
    }
    unsafe { nvim_stateitem_has_cont_list(item) != 0 }
}

/// Cluster operation type for combining cluster lists
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClusterOp {
    /// Replace first list with second
    Replace,
    /// Add second list to first
    Add,
    /// Subtract second list from first
    Subtract,
}

impl ClusterOp {
    /// Convert from integer constant
    #[must_use]
    pub fn from_int(op: i32) -> Option<Self> {
        match op {
            CLUSTER_REPLACE => Some(Self::Replace),
            CLUSTER_ADD => Some(Self::Add),
            CLUSTER_SUBTRACT => Some(Self::Subtract),
            _ => None,
        }
    }

    /// Convert to integer constant
    #[must_use]
    pub fn to_int(self) -> i32 {
        match self {
            Self::Replace => CLUSTER_REPLACE,
            Self::Add => CLUSTER_ADD,
            Self::Subtract => CLUSTER_SUBTRACT,
        }
    }
}

// =============================================================================
// Phase 6: Command & user interface safe wrappers
// =============================================================================

/// Get the current syntax topgrp (for :syn include)
#[must_use]
pub fn topgrp() -> i32 {
    unsafe { nvim_syn_get_topgrp() }
}

/// Set the current syntax topgrp
pub fn set_topgrp(topgrp: i32) {
    unsafe { nvim_syn_set_topgrp(topgrp) }
}

/// Get the syntax block's conceal setting
#[must_use]
pub fn synblock_conceal_setting(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_conceal_setting(block) }
}

/// Get the syntax block's case ignore setting
#[must_use]
pub fn synblock_ic_setting(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_ic_setting(block) }
}

/// Get the number of syntax subcommands
#[must_use]
pub fn subcommand_count() -> i32 {
    unsafe { nvim_syn_get_subcommand_count() }
}

/// Get subcommand name by index
#[must_use]
pub fn subcommand_name(idx: i32) -> Option<&'static str> {
    let ptr = unsafe { nvim_syn_get_subcommand_name(idx) };
    if ptr.is_null() {
        return None;
    }
    // SAFETY: The subcommand names are static strings in C
    unsafe { std::ffi::CStr::from_ptr(ptr).to_str().ok() }
}

/// Check if a pattern at index is for syncing
#[must_use]
pub fn synblock_pattern_is_syncing(block: SynBlockHandle, idx: i32) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_pattern_is_syncing(block, idx) != 0 }
}

/// Get the highlight group ID from a pattern (minus 1)
#[must_use]
pub fn synpat_hl_group(pat: SynPatHandle) -> i32 {
    if pat.is_null() {
        return -1;
    }
    unsafe { nvim_synpat_get_hl_group(pat) }
}

/// Count patterns with a specific highlight group ID
#[must_use]
pub fn synblock_count_patterns_for_id(block: SynBlockHandle, id: i32) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_count_patterns_for_id(block, id) }
}

/// Get the expand_what variable
#[must_use]
pub fn expand_what() -> i32 {
    unsafe { nvim_syn_get_expand_what() }
}

/// Set the expand_what variable
pub fn set_expand_what(what: i32) {
    unsafe { nvim_syn_set_expand_what(what) }
}

/// Expansion types for command completion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpandWhat {
    /// Expand subcommand names
    Subcmd,
    /// Expand case arguments
    Case,
    /// Expand spell arguments
    Spell,
    /// Expand sync arguments
    Sync,
    /// Expand cluster names
    Cluster,
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

// =============================================================================
// FFI exports - Phase 5: Cluster & containedin
// =============================================================================

/// Get a cluster at an index from a synblock
/// Returns NULL if index is out of bounds
///
/// # Safety
/// `block` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synblock_get_cluster(block: SynBlockHandle, idx: c_int) -> SynClusterHandle {
    synblock_get_cluster(block, idx).unwrap_or(SynClusterHandle(std::ptr::null_mut()))
}

/// Get a pattern at an index from a synblock
/// Returns NULL if index is out of bounds
///
/// # Safety
/// `block` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synblock_get_pattern(block: SynBlockHandle, idx: c_int) -> SynPatHandle {
    synblock_get_pattern(block, idx).unwrap_or(SynPatHandle(std::ptr::null_mut()))
}

/// Get the current window's synblock
#[no_mangle]
pub extern "C" fn rs_curwin_synblock() -> SynBlockHandle {
    curwin_synblock()
}

/// Get the spell cluster ID from a synblock
///
/// # Safety
/// `block` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synblock_spell_cluster(block: SynBlockHandle) -> c_int {
    synblock_spell_cluster(block)
}

/// Get the nospell cluster ID from a synblock
///
/// # Safety
/// `block` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synblock_nospell_cluster(block: SynBlockHandle) -> c_int {
    synblock_nospell_cluster(block)
}

/// Check if a stateitem has the HL_TRANS_CONT flag
///
/// # Safety
/// `item` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_stateitem_has_trans_cont(item: StateItemHandle) -> c_int {
    if stateitem_has_trans_cont(item) {
        1
    } else {
        0
    }
}

/// Check if a stateitem has the HL_MATCH flag
///
/// # Safety
/// `item` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_stateitem_has_match(item: StateItemHandle) -> c_int {
    if stateitem_has_match(item) {
        1
    } else {
        0
    }
}

/// Check if a stateitem has a containedin list
///
/// # Safety
/// `item` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_stateitem_has_cont_list(item: StateItemHandle) -> c_int {
    if stateitem_has_cont_list(item) {
        1
    } else {
        0
    }
}

/// Get the cluster ID from a cluster handle
///
/// # Safety
/// `cluster` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_syncluster_id(cluster: SynClusterHandle) -> c_int {
    syncluster_id(cluster)
}

// =============================================================================
// FFI exports - Phase 6: Commands & user interface
// =============================================================================

/// Get the current syntax topgrp
#[no_mangle]
pub extern "C" fn rs_syn_topgrp() -> c_int {
    topgrp()
}

/// Set the current syntax topgrp
#[no_mangle]
pub extern "C" fn rs_syn_set_topgrp(val: c_int) {
    set_topgrp(val);
}

/// Get the conceal setting from a synblock
///
/// # Safety
/// `block` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synblock_conceal_setting(block: SynBlockHandle) -> c_int {
    synblock_conceal_setting(block)
}

/// Get the case ignore setting from a synblock
///
/// # Safety
/// `block` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synblock_ic_setting(block: SynBlockHandle) -> c_int {
    synblock_ic_setting(block)
}

/// Get the number of syntax subcommands
#[no_mangle]
pub extern "C" fn rs_syn_subcommand_count() -> c_int {
    subcommand_count()
}

/// Check if a pattern at index is for syncing
///
/// # Safety
/// `block` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synblock_pattern_is_syncing(block: SynBlockHandle, idx: c_int) -> c_int {
    if synblock_pattern_is_syncing(block, idx) {
        1
    } else {
        0
    }
}

/// Get the highlight group ID from a pattern
///
/// # Safety
/// `pat` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synpat_hl_group(pat: SynPatHandle) -> c_int {
    synpat_hl_group(pat)
}

/// Count patterns with a specific highlight group ID
///
/// # Safety
/// `block` must be a valid pointer or null.
#[no_mangle]
pub extern "C" fn rs_synblock_count_patterns_for_id(block: SynBlockHandle, id: c_int) -> c_int {
    synblock_count_patterns_for_id(block, id)
}

/// Get the expand_what variable
#[no_mangle]
pub extern "C" fn rs_syn_expand_what() -> c_int {
    expand_what()
}

/// Set the expand_what variable
#[no_mangle]
pub extern "C" fn rs_syn_set_expand_what(what: c_int) {
    set_expand_what(what);
}

// =============================================================================
// Pattern Flag Analysis Helpers
// =============================================================================

/// Describes the flags present in a syntax pattern.
#[repr(C)]
pub struct SynPatFlagsInfo {
    /// Pattern is contained (used inside other patterns)
    pub contained: c_int,
    /// Pattern is transparent (inherits highlighting)
    pub transparent: c_int,
    /// Match within one line only
    pub oneline: c_int,
    /// Uses keepend flag
    pub keepend: c_int,
    /// Uses extend flag
    pub extend: c_int,
    /// Pattern can be concealed
    pub conceal: c_int,
    /// Ends can be concealed
    pub conceal_ends: c_int,
    /// Defines a fold
    pub fold: c_int,
    /// Display-only (not for syncing)
    pub display: c_int,
}

/// Analyze a pattern's flags and return a structured description.
/// This is useful for debugging and introspection of syntax patterns.
#[no_mangle]
pub extern "C" fn rs_analyze_syn_pat_flags(flags: c_int) -> SynPatFlagsInfo {
    SynPatFlagsInfo {
        contained: c_int::from((flags & HL_CONTAINED) != 0),
        transparent: c_int::from((flags & HL_TRANSP) != 0),
        oneline: c_int::from((flags & HL_ONELINE) != 0),
        keepend: c_int::from((flags & HL_KEEPEND) != 0),
        extend: c_int::from((flags & HL_EXTEND) != 0),
        conceal: c_int::from((flags & HL_CONCEAL) != 0),
        conceal_ends: c_int::from((flags & HL_CONCEALENDS) != 0),
        fold: c_int::from((flags & HL_FOLD) != 0),
        display: c_int::from((flags & HL_DISPLAY) != 0),
    }
}

/// Check if pattern flags contain any skip-related flags.
/// Returns a bitmask of (SKIPNL, SKIPWHITE, SKIPEMPTY) flags that are set.
#[no_mangle]
pub extern "C" fn rs_syn_pat_skip_flags(flags: c_int) -> c_int {
    flags & (HL_SKIPNL | HL_SKIPWHITE | HL_SKIPEMPTY)
}

/// Check if pattern flags indicate a sync-related pattern.
#[no_mangle]
pub extern "C" fn rs_syn_pat_is_sync_related(flags: c_int) -> c_int {
    c_int::from((flags & (HL_SYNC_HERE | HL_SYNC_THERE)) != 0)
}

/// Get the effective visibility flags from pattern flags.
/// Returns 1 if the pattern should be visible (not transparent, not display-only for sync).
#[no_mangle]
pub extern "C" fn rs_syn_pat_is_visible(flags: c_int) -> c_int {
    c_int::from((flags & HL_TRANSP) == 0)
}

/// Convert a pattern type integer to its name.
/// Returns a static string pointer for the pattern type name.
#[no_mangle]
pub extern "C" fn rs_sptype_name(sptype: c_int) -> *const c_char {
    static MATCH_STR: &[u8] = b"MATCH\0";
    static START_STR: &[u8] = b"START\0";
    static END_STR: &[u8] = b"END\0";
    static SKIP_STR: &[u8] = b"SKIP\0";
    static UNKNOWN_STR: &[u8] = b"UNKNOWN\0";

    let s = match sptype {
        SPTYPE_MATCH => MATCH_STR,
        SPTYPE_START => START_STR,
        SPTYPE_END => END_STR,
        SPTYPE_SKIP => SKIP_STR,
        _ => UNKNOWN_STR,
    };
    s.as_ptr() as *const c_char
}

// =============================================================================
// Phase 24.1: State Management Functions (FFI exports)
// =============================================================================

/// Try saving the current state in b_sst_array[].
/// The current state must be valid for the start of the current_lnum line!
/// Returns the synstate entry (or NULL if not stored).
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_store_current_state() -> SynStateHandle {
    let lnum = nvim_syn_get_current_lnum();
    let state_len = nvim_syn_get_current_state_len();

    // Find existing entry at or before current line
    let sp = nvim_syn_stack_find_entry(lnum);

    // Check if current state contains items that span across lines
    // If so, we can't use this state - it's not valid for line boundaries
    let mut has_spanning_item = false;
    for i in (0..state_len).rev() {
        if nvim_syn_state_item_spans_line(i, lnum) != 0 {
            has_spanning_item = true;
            break;
        }
    }

    if has_spanning_item {
        // Current state spans lines, can't store it
        // If there was an existing entry at this line, remove it
        if !sp.is_null() {
            nvim_syn_stack_remove_entry(sp);
        }
        nvim_syn_set_state_stored(1);
        return SynStateHandle::null();
    }

    // Determine if we need to allocate a new entry
    let entry = if sp.is_null() || nvim_synstate_get_lnum(sp) != lnum {
        // Need to allocate a new entry
        nvim_syn_stack_alloc_entry(lnum, sp)
    } else {
        // Reuse existing entry
        sp
    };

    if !entry.is_null() {
        // Store current state to the entry
        nvim_syn_store_state_to_entry(entry);
    }

    nvim_syn_set_state_stored(1);
    entry
}

/// Copy a state stack from a synstate entry to current_state.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_load_current_state(from: SynStateHandle) {
    if from.is_null() {
        return;
    }

    // Clear and validate current state
    nvim_syn_clear_current_state();
    nvim_syn_validate_current_state();
    nvim_syn_set_keepend_level(-1);

    let stacksize = nvim_synstate_get_stacksize(from);
    if stacksize > 0 {
        // Grow current state array
        nvim_syn_grow_current_state(stacksize);
        nvim_syn_set_current_state_len(stacksize);

        // Copy each state item
        let mut keepend_level = -1;
        for i in 0..stacksize {
            let bs = nvim_synstate_get_bufstate(from, i);
            if bs.is_null() {
                continue;
            }

            let bs_idx = nvim_bufstate_get_idx(bs);
            let bs_flags = nvim_bufstate_get_flags(bs);
            let bs_seqnr = nvim_bufstate_get_seqnr(bs);
            let bs_cchar = nvim_bufstate_get_cchar(bs);
            let extmatch = nvim_bufstate_get_extmatch(bs);

            // Set the state item (this also sets si_next_list based on pattern)
            nvim_syn_set_cur_state_item(i, bs_idx, bs_flags, bs_seqnr, bs_cchar, extmatch);

            // Track keepend level
            if keepend_level < 0 && (bs_flags & HL_KEEPEND) != 0 {
                keepend_level = i;
            }

            // Update attributes for this item
            nvim_syn_update_si_attr(i);
        }

        nvim_syn_set_keepend_level(keepend_level);
    }

    // Copy next_list and next_flags from saved state
    let next_list = nvim_synstate_get_next_list(from);
    nvim_syn_set_current_next_list(next_list);
    nvim_syn_set_current_next_flags(nvim_synstate_get_next_flags(from));
    nvim_syn_set_current_lnum(nvim_synstate_get_lnum(from));
}

/// Compare saved state stack with the current state.
/// Returns 1 if they are equal, 0 otherwise.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_stack_equal(sp: SynStateHandle) -> c_int {
    if sp.is_null() {
        return 0;
    }

    let sp_stacksize = nvim_synstate_get_stacksize(sp);
    let current_len = nvim_syn_get_current_state_len();

    // Quick check: stack sizes must match
    if sp_stacksize != current_len {
        return 0;
    }

    // Quick check: next_list pointers must match
    // (We compare raw pointers since they point to the same data)
    let sp_next_list = nvim_synstate_get_next_list(sp);
    let cur_next_list = nvim_syn_get_current_next_list();
    if sp_next_list.0 != cur_next_list.0 {
        return 0;
    }

    // Compare each state item
    let nsubexp = nvim_syn_get_nsubexp();
    for i in (0..current_len).rev() {
        let bs = nvim_synstate_get_bufstate(sp, i);
        if bs.is_null() {
            return 0;
        }

        let cur_si = nvim_syn_get_cur_state(i);
        if cur_si.is_null() {
            return 0;
        }

        // Compare indices
        let bs_idx = nvim_bufstate_get_idx(bs);
        let si_idx = nvim_stateitem_get_idx(cur_si);
        if bs_idx != si_idx {
            return 0;
        }

        // Compare extmatch
        let bs_extmatch = nvim_bufstate_get_extmatch(bs);
        let si_extmatch = nvim_stateitem_get_extmatch(cur_si);
        let cmp = nvim_syn_extmatch_equal(bs_extmatch, si_extmatch);

        if cmp == 1 {
            // Same pointer or both NULL, continue
            continue;
        } else if cmp == 0 {
            // One is NULL, the other isn't
            return 0;
        }

        // cmp == -1: need to compare strings
        for j in 0..nsubexp {
            if nvim_syn_extmatch_strings_equal(bs_extmatch, si_extmatch, j, si_idx) == 0 {
                return 0;
            }
        }
    }

    1
}

/// Invalidate the current state - clear it and mark as invalid.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_invalidate_current_state() {
    nvim_syn_invalidate_current_state();
}

/// Clear the current state stack.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_current_state() {
    nvim_syn_clear_current_state();
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

        // Verify cluster operation constants
        assert_eq!(CLUSTER_REPLACE, 1);
        assert_eq!(CLUSTER_ADD, 2);
        assert_eq!(CLUSTER_SUBTRACT, 3);
        assert!(MAX_CLUSTER_ID > 0);
    }

    #[test]
    fn test_cluster_op_conversion() {
        // Test ClusterOp::from_int
        assert_eq!(
            ClusterOp::from_int(CLUSTER_REPLACE),
            Some(ClusterOp::Replace)
        );
        assert_eq!(ClusterOp::from_int(CLUSTER_ADD), Some(ClusterOp::Add));
        assert_eq!(
            ClusterOp::from_int(CLUSTER_SUBTRACT),
            Some(ClusterOp::Subtract)
        );
        assert_eq!(ClusterOp::from_int(0), None);
        assert_eq!(ClusterOp::from_int(4), None);

        // Test ClusterOp::to_int
        assert_eq!(ClusterOp::Replace.to_int(), CLUSTER_REPLACE);
        assert_eq!(ClusterOp::Add.to_int(), CLUSTER_ADD);
        assert_eq!(ClusterOp::Subtract.to_int(), CLUSTER_SUBTRACT);
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

    #[test]
    fn test_analyze_syn_pat_flags() {
        // Test with no flags
        let info = rs_analyze_syn_pat_flags(0);
        assert_eq!(info.contained, 0);
        assert_eq!(info.transparent, 0);
        assert_eq!(info.conceal, 0);
        assert_eq!(info.fold, 0);

        // Test with contained flag
        let info = rs_analyze_syn_pat_flags(HL_CONTAINED);
        assert_eq!(info.contained, 1);
        assert_eq!(info.transparent, 0);

        // Test with multiple flags
        let flags = HL_CONTAINED | HL_TRANSP | HL_KEEPEND | HL_FOLD;
        let info = rs_analyze_syn_pat_flags(flags);
        assert_eq!(info.contained, 1);
        assert_eq!(info.transparent, 1);
        assert_eq!(info.keepend, 1);
        assert_eq!(info.fold, 1);
        assert_eq!(info.oneline, 0);
        assert_eq!(info.conceal, 0);

        // Test conceal flags
        let info = rs_analyze_syn_pat_flags(HL_CONCEAL | HL_CONCEALENDS);
        assert_eq!(info.conceal, 1);
        assert_eq!(info.conceal_ends, 1);
    }

    #[test]
    fn test_syn_pat_skip_flags() {
        assert_eq!(rs_syn_pat_skip_flags(0), 0);
        assert_eq!(rs_syn_pat_skip_flags(HL_SKIPNL), HL_SKIPNL);
        assert_eq!(rs_syn_pat_skip_flags(HL_SKIPWHITE), HL_SKIPWHITE);
        assert_eq!(rs_syn_pat_skip_flags(HL_SKIPEMPTY), HL_SKIPEMPTY);

        // Combined
        let flags = HL_SKIPNL | HL_SKIPWHITE | HL_SKIPEMPTY;
        assert_eq!(rs_syn_pat_skip_flags(flags), flags);

        // With other flags mixed in
        let flags = HL_CONTAINED | HL_SKIPNL | HL_FOLD;
        assert_eq!(rs_syn_pat_skip_flags(flags), HL_SKIPNL);
    }

    #[test]
    fn test_syn_pat_is_sync_related() {
        assert_eq!(rs_syn_pat_is_sync_related(0), 0);
        assert_eq!(rs_syn_pat_is_sync_related(HL_CONTAINED), 0);
        assert_eq!(rs_syn_pat_is_sync_related(HL_SYNC_HERE), 1);
        assert_eq!(rs_syn_pat_is_sync_related(HL_SYNC_THERE), 1);
        assert_eq!(rs_syn_pat_is_sync_related(HL_SYNC_HERE | HL_SYNC_THERE), 1);
    }

    #[test]
    fn test_syn_pat_is_visible() {
        assert_eq!(rs_syn_pat_is_visible(0), 1); // No flags = visible
        assert_eq!(rs_syn_pat_is_visible(HL_CONTAINED), 1);
        assert_eq!(rs_syn_pat_is_visible(HL_TRANSP), 0); // Transparent = not visible
        assert_eq!(rs_syn_pat_is_visible(HL_TRANSP | HL_FOLD), 0);
    }

    #[test]
    fn test_sptype_name() {
        use std::ffi::CStr;

        unsafe {
            let match_name = CStr::from_ptr(rs_sptype_name(SPTYPE_MATCH));
            assert_eq!(match_name.to_str().unwrap(), "MATCH");

            let start_name = CStr::from_ptr(rs_sptype_name(SPTYPE_START));
            assert_eq!(start_name.to_str().unwrap(), "START");

            let end_name = CStr::from_ptr(rs_sptype_name(SPTYPE_END));
            assert_eq!(end_name.to_str().unwrap(), "END");

            let skip_name = CStr::from_ptr(rs_sptype_name(SPTYPE_SKIP));
            assert_eq!(skip_name.to_str().unwrap(), "SKIP");

            let unknown = CStr::from_ptr(rs_sptype_name(99));
            assert_eq!(unknown.to_str().unwrap(), "UNKNOWN");
        }
    }
}
