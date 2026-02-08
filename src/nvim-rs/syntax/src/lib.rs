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
// Modules
// =============================================================================

pub mod api;
pub mod attr;
pub mod buffer;
pub mod cache;
pub mod cluster;
pub mod commands;
pub mod containment;
pub mod engine;
pub mod fold;
pub mod group;
pub mod highlight;
pub mod item;
pub mod keyword;
pub mod match_engine;
pub mod parse;
pub mod pattern;
pub mod region;
pub mod state;
pub mod sync;
pub mod types;

// =============================================================================
// Re-exports from types module
// =============================================================================

pub use types::{
    cluster_index,
    extract_inc_tag,
    is_cluster_id,
    is_normal_id,
    is_special_id,
    make_cluster_id,
    synid_type,
    // Opaque handles
    BufHandle,
    BufStateHandle,
    // Types and functions
    ExpandWhat,
    ExtMatchHandle,
    IdListHandle,
    KeyEntryHandle,
    RegProgHandle,
    StateItemHandle,
    SynBlockHandle,
    SynClusterHandle,
    SynIdType,
    SynPatHandle,
    SynStateHandle,
    WinHandle,
    // Constants - Cluster operations
    CLUSTER_ADD,
    CLUSTER_REPLACE,
    CLUSTER_SUBTRACT,
    // Constants - Highlight flags
    HL_CONCEAL,
    HL_CONCEALENDS,
    HL_CONTAINED,
    HL_DISPLAY,
    HL_EXCLUDENL,
    HL_EXTEND,
    HL_FOLD,
    HL_HAS_EOL,
    HL_INCLUDED_TOPLEVEL,
    HL_KEEPEND,
    HL_MATCH,
    HL_MATCHCONT,
    HL_ONELINE,
    HL_SKIPEMPTY,
    HL_SKIPNL,
    HL_SKIPWHITE,
    HL_SYNC_HERE,
    HL_SYNC_THERE,
    HL_TRANSP,
    HL_TRANS_CONT,
    // Constants - Item argument types
    ID_LIST_ALL_SENTINEL,
    ITEM_END,
    ITEM_MATCHGROUP,
    ITEM_SKIP,
    ITEM_START,
    // Constants - Special indices
    KEYWORD_IDX,
    MAXKEYWLEN,
    // Constants - Syntax group IDs
    MAX_CLUSTER_ID,
    MAX_HL_ID,
    MAX_SYN_INC_TAG,
    NONE_IDX,
    // Constants - Sync flags
    SF_CCOMMENT,
    SF_MATCH,
    // Constants - Pattern offset types
    SPO_COUNT,
    SPO_HE_OFF,
    SPO_HS_OFF,
    SPO_LC_OFF,
    SPO_ME_OFF,
    SPO_MS_OFF,
    SPO_RE_OFF,
    SPO_RS_OFF,
    // Constants - Pattern types
    SPTYPE_END,
    SPTYPE_MATCH,
    SPTYPE_SKIP,
    SPTYPE_START,
    // Constants - State stack sizes
    SST_DIST,
    SST_FIX_STATES,
    SST_MAX_ENTRIES,
    SST_MIN_ENTRIES,
    SYNID_ALLBUT,
    SYNID_CLUSTER,
    SYNID_CONTAINED,
    SYNID_TOP,
    // Constants - Spell checking
    SYNSPL_DEFAULT,
    SYNSPL_NOTOP,
    SYNSPL_TOP,
};

// Re-export pattern offset and item type enums
pub use pattern::PatternType;

pub use attr::{
    rs_combine_attrs, rs_invalidation_needs_full_sync, rs_should_spell_check,
    rs_syn_attr_stack_clear, rs_syn_attr_stack_depth, rs_syn_attr_stack_effective_attr,
    rs_syn_attr_stack_effective_spell, rs_syn_attr_stack_free, rs_syn_attr_stack_new,
    rs_syn_attr_stack_pop, rs_syn_attr_stack_push, rs_syn_attr_state_conceal_char,
    rs_syn_attr_state_has_attr, rs_syn_attr_state_is_concealed, rs_syn_attr_state_new,
    rs_syn_change_merge, rs_syn_change_no_change, rs_syn_change_range, rs_syn_change_single_line,
};

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

    // -------------------------------------------------------------------------
    // Phase 18a: Synblock setters for :syntax commands
    // -------------------------------------------------------------------------

    /// Set b_syn_ic (ignore case for :syn cmds)
    fn nvim_synblock_set_syn_ic(block: SynBlockHandle, ic: c_int);

    /// Set b_syn_spell (SYNSPL_ values)
    fn nvim_synblock_set_syn_spell(block: SynBlockHandle, spell: c_int);

    /// Set b_syn_foldlevel
    fn nvim_synblock_set_syn_foldlevel(block: SynBlockHandle, foldlevel: c_int);

    /// Set b_syn_conceal (auto-conceal for :syn cmds)
    fn nvim_synblock_set_conceal(block: SynBlockHandle, conceal: c_int);

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

    // -------------------------------------------------------------------------
    // Phase 24.2: Core Pattern Matching Helpers
    // -------------------------------------------------------------------------

    /// Get si_m_endpos.lnum from stateitem
    fn nvim_stateitem_get_m_endpos_lnum(item: StateItemHandle) -> c_int;

    /// Get si_m_endpos.col from stateitem
    fn nvim_stateitem_get_m_endpos_col(item: StateItemHandle) -> c_int;

    /// Get si_h_startpos.lnum from stateitem
    fn nvim_stateitem_get_h_startpos_lnum(item: StateItemHandle) -> c_int;

    /// Get si_h_startpos.col from stateitem
    fn nvim_stateitem_get_h_startpos_col(item: StateItemHandle) -> c_int;

    /// Get si_h_endpos.lnum from stateitem
    fn nvim_stateitem_get_h_endpos_lnum(item: StateItemHandle) -> c_int;

    /// Get si_h_endpos.col from stateitem
    fn nvim_stateitem_get_h_endpos_col(item: StateItemHandle) -> c_int;

    /// Get si_eoe_pos.lnum from stateitem
    fn nvim_stateitem_get_eoe_pos_lnum(item: StateItemHandle) -> c_int;

    /// Get si_eoe_pos.col from stateitem
    fn nvim_stateitem_get_eoe_pos_col(item: StateItemHandle) -> c_int;

    /// Set si_m_endpos
    fn nvim_stateitem_set_m_endpos(item: StateItemHandle, lnum: c_int, col: c_int);

    /// Set si_h_endpos
    fn nvim_stateitem_set_h_endpos(item: StateItemHandle, lnum: c_int, col: c_int);

    /// Set si_eoe_pos
    fn nvim_stateitem_set_eoe_pos(item: StateItemHandle, lnum: c_int, col: c_int);

    /// Set si_idx
    fn nvim_stateitem_set_idx(item: StateItemHandle, idx: c_int);

    /// Set si_end_idx
    fn nvim_stateitem_set_end_idx(item: StateItemHandle, end_idx: c_int);

    /// Set si_flags
    fn nvim_stateitem_set_flags(item: StateItemHandle, flags: c_int);

    /// Add flags to si_flags
    fn nvim_stateitem_add_flags(item: StateItemHandle, flags: c_int);

    /// Set si_seqnr
    fn nvim_stateitem_set_seqnr(item: StateItemHandle, seqnr: c_int);

    /// Set si_ends
    fn nvim_stateitem_set_ends(item: StateItemHandle, ends: c_int);

    /// Set si_id
    fn nvim_stateitem_set_id(item: StateItemHandle, id: c_int);

    /// Set si_trans_id
    fn nvim_stateitem_set_trans_id(item: StateItemHandle, trans_id: c_int);

    /// Set si_attr
    fn nvim_stateitem_set_attr(item: StateItemHandle, attr: c_int);

    /// Set si_cont_list
    fn nvim_stateitem_set_cont_list(item: StateItemHandle, list: IdListHandle);

    /// Get next_seqnr and increment it
    fn nvim_syn_next_seqnr() -> c_int;

    /// Get next_match_idx
    fn nvim_syn_get_next_match_idx_value() -> c_int;

    /// Set next_match_idx
    fn nvim_syn_set_next_match_idx(idx: c_int);

    /// Set next_match_col
    fn nvim_syn_set_next_match_col(col: c_int);

    /// Set current_next_list
    fn nvim_syn_set_current_next_list_ptr(list: IdListHandle);

    /// Get current_next_list
    fn nvim_syn_get_current_next_list_ptr() -> IdListHandle;

    /// Call check_state_ends
    fn nvim_syn_check_state_ends();

    /// Call update_si_attr
    fn nvim_syn_call_update_si_attr(idx: c_int);

    /// Call check_keepend
    fn nvim_syn_check_keepend();

    /// Call pop_current_state
    fn nvim_syn_pop_current_state();

    /// Call push_current_state
    fn nvim_syn_push_current_state(idx: c_int);

    /// Get the current line at the current column
    fn nvim_syn_getcurline_at_col() -> c_char;

    /// Check if current_state is empty
    fn nvim_syn_current_state_is_empty() -> c_int;

    /// Set current_finished
    fn nvim_syn_set_current_finished(finished: c_int);

    /// Get synpat_T sp_type for pattern at index
    fn nvim_synblock_pattern_type(idx: c_int) -> c_int;

    /// Get synpat_T sp_flags for pattern at index
    fn nvim_synblock_pattern_flags(idx: c_int) -> c_int;

    /// Get synpat_T sp_syn.id for pattern at index
    fn nvim_synblock_pattern_syn_id(idx: c_int) -> c_int;

    /// Get synpat_T sp_syn_match_id for pattern at index
    fn nvim_synblock_pattern_match_id(idx: c_int) -> c_int;

    /// Get synpat_T sp_cont_list for pattern at index
    fn nvim_synblock_pattern_cont_list(idx: c_int) -> IdListHandle;

    /// Get synpat_T sp_next_list for pattern at index
    fn nvim_synblock_pattern_next_list(idx: c_int) -> IdListHandle;

    /// Call syn_id2attr
    fn nvim_syn_id2attr_wrapper(syn_id: c_int) -> c_int;

    /// Call syn_update_ends
    fn nvim_syn_call_syn_update_ends(syncing: c_int);

    /// Get si_next_list from stateitem
    fn nvim_stateitem_get_next_list(item: StateItemHandle) -> IdListHandle;

    /// Set si_next_list for stateitem
    fn nvim_stateitem_set_next_list(item: StateItemHandle, list: IdListHandle);

    /// Check if the ID_LIST_ALL constant matches a pointer
    fn nvim_syn_is_id_list_all(list: IdListHandle) -> c_int;

    /// Get the ID_LIST_ALL pointer
    fn nvim_syn_get_id_list_all() -> IdListHandle;

    // -------------------------------------------------------------------------
    // Phase 24.3: Keyword Matching Helpers
    // -------------------------------------------------------------------------

    /// Call check_keyword_id from Rust
    fn nvim_syn_check_keyword_id(
        line: *mut c_char,
        startcol: c_int,
        endcolp: *mut c_int,
        flagsp: *mut c_int,
        next_listp: *mut IdListHandle,
        cur_si: StateItemHandle,
        ccharp: *mut c_int,
    ) -> c_int;

    /// Call in_id_list from Rust
    fn nvim_syn_in_id_list(
        cur_si: StateItemHandle,
        list: IdListHandle,
        id: c_int,
        inc_tag: c_int,
        cont_in_list: IdListHandle,
        flags: c_int,
    ) -> c_int;

    /// Check if there are keywords (case sensitive) in synblock
    fn nvim_syn_has_keywords() -> c_int;

    /// Check if there are keywords (case insensitive) in synblock
    fn nvim_syn_has_keywords_ic() -> c_int;

    /// Check if a char is a keyword character at position in syn_buf
    fn nvim_syn_is_keyword_char(line: *mut c_char, pos: c_int) -> c_int;

    /// Get the current line from syn_getcurline
    fn nvim_syn_getcurline() -> *mut c_char;

    /// Call save_chartab for syntax iskeyword
    fn nvim_syn_save_chartab(buf: *mut c_char);

    /// Call restore_chartab
    fn nvim_syn_restore_chartab(buf: *mut c_char);

    // NOTE: Keyword-related FFI functions moved to keyword.rs module

    /// Get syn_buf pointer (for keyword char checks)
    fn nvim_syn_get_buf() -> *mut std::ffi::c_void;

    // -------------------------------------------------------------------------
    // Phase 24.4: Pattern Stack Operations Helpers (new declarations only)
    // -------------------------------------------------------------------------

    /// Get the current_state garray length
    fn nvim_syn_current_state_len() -> c_int;

    /// Get a stateitem from current_state by index
    fn nvim_syn_get_stateitem(index: c_int) -> StateItemHandle;

    /// Get top stateitem from current_state
    fn nvim_syn_get_top_stateitem() -> StateItemHandle;

    /// Get next_seqnr global
    fn nvim_syn_get_next_seqnr() -> c_int;

    /// Set next_seqnr global
    fn nvim_syn_set_next_seqnr(seqnr: c_int);

    /// Increment and get next_seqnr global
    fn nvim_syn_incr_next_seqnr() -> c_int;

    /// Get next_match_h_startpos
    fn nvim_syn_get_next_match_h_startpos(lnum: *mut c_int, col: *mut c_int);

    /// Get next_match_m_endpos
    fn nvim_syn_get_next_match_m_endpos(lnum: *mut c_int, col: *mut c_int);

    /// Get next_match_h_endpos
    fn nvim_syn_get_next_match_h_endpos(lnum: *mut c_int, col: *mut c_int);

    /// Get next_match_eos_pos
    fn nvim_syn_get_next_match_eos_pos(lnum: *mut c_int, col: *mut c_int);

    /// Get next_match_eoe_pos
    fn nvim_syn_get_next_match_eoe_pos(lnum: *mut c_int, col: *mut c_int);

    /// Get next_match_flags
    fn nvim_syn_get_next_match_flags() -> c_int;

    /// Get next_match_end_idx
    fn nvim_syn_get_next_match_end_idx() -> c_int;

    /// Get next_match_extmatch
    fn nvim_syn_get_next_match_extmatch() -> ExtMatchHandle;

    /// Call ref_extmatch
    fn nvim_syn_ref_extmatch(em: ExtMatchHandle) -> ExtMatchHandle;

    /// Call unref_extmatch
    fn nvim_syn_unref_extmatch(em: ExtMatchHandle);

    /// Call update_si_end from Rust
    fn nvim_syn_update_si_end(sip: StateItemHandle, startcol: c_int, force: c_int);

    /// Call push_next_match from Rust
    fn nvim_syn_push_next_match() -> StateItemHandle;

    /// Call find_endpos from Rust
    fn nvim_syn_find_endpos(
        idx: c_int,
        start_lnum: c_int,
        start_col: c_int,
        m_end_lnum: *mut c_int,
        m_end_col: *mut c_int,
        hl_end_lnum: *mut c_int,
        hl_end_col: *mut c_int,
        flagsp: *mut c_int,
        end_end_lnum: *mut c_int,
        end_end_col: *mut c_int,
        end_idx: *mut c_int,
        start_ext: ExtMatchHandle,
    );

    /// Get synpat sp_flags by index
    fn nvim_syn_get_pattern_flags(idx: c_int) -> c_int;

    /// Get synpat sp_cchar by index
    fn nvim_syn_get_pattern_cchar(idx: c_int) -> c_int;

    /// Get synpat sp_next_list by index
    fn nvim_syn_get_pattern_next_list(idx: c_int) -> IdListHandle;

    /// Get synpat sp_type by index
    fn nvim_syn_get_pattern_type(idx: c_int) -> c_int;

    /// Get synpat sp_syn_match_id by index
    fn nvim_syn_get_pattern_syn_match_id(idx: c_int) -> c_int;

    /// Get GA_EMPTY(&current_state) check
    fn nvim_syn_is_current_state_empty() -> c_int;

    /// Set si_h_startpos
    fn nvim_stateitem_set_h_startpos(item: StateItemHandle, lnum: c_int, col: c_int);

    /// Set si_m_startcol
    fn nvim_stateitem_set_m_startcol(item: StateItemHandle, col: c_int);

    /// Set si_m_lnum
    fn nvim_stateitem_set_m_lnum(item: StateItemHandle, lnum: c_int);

    /// Or si_flags with a value
    fn nvim_stateitem_or_flags(item: StateItemHandle, flags: c_int);

    /// Set si_cchar
    fn nvim_stateitem_set_cchar(item: StateItemHandle, cchar: c_int);

    /// Set si_extmatch
    fn nvim_stateitem_set_extmatch(item: StateItemHandle, em: ExtMatchHandle);

    /// Get SPTYPE_START constant
    fn nvim_syn_get_sptype_start() -> c_int;

    /// Get HL_ONELINE constant
    fn nvim_syn_get_hl_oneline() -> c_int;

    /// Get HL_KEEPEND constant
    fn nvim_syn_get_hl_keepend() -> c_int;

    /// Get HL_MATCH constant
    fn nvim_syn_get_hl_match() -> c_int;

    /// Get HL_CONCEAL constant
    fn nvim_syn_get_hl_conceal() -> c_int;

    /// Get HL_CONCEALENDS constant
    fn nvim_syn_get_hl_concealends() -> c_int;

    // -------------------------------------------------------------------------
    // Phase 24.5: Sync and Line Operations Helpers
    // -------------------------------------------------------------------------

    /// Call syn_start_line from Rust
    fn nvim_syn_start_line();

    /// Call syn_finish_line from Rust
    fn nvim_syn_finish_line(syncing: c_int) -> c_int;

    /// Call syn_update_ends from Rust
    fn nvim_syn_update_ends(startofline: c_int);

    /// Call syn_sync from Rust
    fn nvim_syn_sync(wp: WinHandle, start_lnum: c_int, last_valid: SynStateHandle);

    /// Call syntax_start from Rust
    fn nvim_syntax_start(wp: WinHandle, lnum: c_int);

    /// Call clear_syn_state from Rust
    fn nvim_syn_clear_syn_state(p: SynStateHandle);

    /// Get current_line_id global
    fn nvim_syn_get_current_line_id() -> c_int;

    /// Increment current_line_id global
    fn nvim_syn_incr_current_line_id();

    /// Get syn_block pointer
    fn nvim_syn_get_syn_block() -> SynBlockHandle;

    /// Set syn_block pointer
    fn nvim_syn_set_syn_block(block: SynBlockHandle);

    /// Get syn_win pointer
    fn nvim_syn_get_syn_win() -> WinHandle;

    /// Set syn_win pointer
    fn nvim_syn_set_syn_win(win: WinHandle);

    /// Get b_syn_sync_minlines from syn_block
    fn nvim_syn_get_sync_minlines() -> c_int;

    /// Get b_syn_sync_maxlines from syn_block
    fn nvim_syn_get_sync_maxlines() -> c_int;

    /// Get b_syn_sync_flags from syn_block
    fn nvim_syn_get_sync_flags() -> c_int;

    /// Get b_syn_sync_id from syn_block
    fn nvim_syn_get_sync_id() -> c_int;

    /// Get b_sst_first from syn_block (first in valid state list)
    fn nvim_syn_get_sst_first() -> SynStateHandle;

    /// Get b_sst_array from syn_block
    fn nvim_syn_get_sst_array() -> *mut std::ffi::c_void;

    /// Get b_sst_len from syn_block
    fn nvim_syn_get_sst_len() -> c_int;

    /// Set synstate sst_change_lnum
    fn nvim_synstate_set_change_lnum(p: SynStateHandle, lnum: c_int);

    /// Get synstate sst_next (void* return for FFI)
    fn nvim_syn_synstate_get_next_ptr(p: SynStateHandle) -> SynStateHandle;

    /// Call syn_stack_find_entry from Rust (void* return for FFI)
    fn nvim_syn_stack_find_entry_ptr(lnum: c_int) -> SynStateHandle;

    /// Set current_id global
    fn nvim_syn_set_current_id(id: c_int);

    /// Set current_trans_id global
    fn nvim_syn_set_current_trans_id(id: c_int);

    /// Set current_flags global
    fn nvim_syn_set_current_flags(flags: c_int);

    /// Set current_seqnr global
    fn nvim_syn_set_current_seqnr(seqnr: c_int);

    /// Get HL_MATCHCONT constant
    fn nvim_syn_get_hl_matchcont() -> c_int;

    /// Get HL_EXTEND constant
    fn nvim_syn_get_hl_extend() -> c_int;

    /// Get SF_CCOMMENT constant
    fn nvim_syn_get_sf_ccomment() -> c_int;

    /// Get SF_MATCH constant
    fn nvim_syn_get_sf_match() -> c_int;

    /// Get HL_SYNC_HERE constant
    fn nvim_syn_get_hl_sync_here() -> c_int;

    /// Get HL_SYNC_THERE constant
    fn nvim_syn_get_hl_sync_there() -> c_int;

    /// Get SPTYPE_MATCH constant
    fn nvim_syn_get_sptype_match() -> c_int;

    /// Call syn_stack_alloc from Rust
    fn nvim_syn_stack_alloc();

    /// Get w_s from window (synblock)
    fn nvim_win_get_synblock(wp: WinHandle) -> SynBlockHandle;

    /// Get w_buffer from window (void* return for FFI)
    fn nvim_syn_win_get_buffer_ptr(wp: WinHandle) -> BufHandle;

    /// Get ml_line_count from buffer (void* input for FFI)
    fn nvim_syn_buf_get_line_count(buf: BufHandle) -> c_int;

    /// Call buf_get_changedtick from Rust (void* input for FFI)
    fn nvim_syn_buf_get_changed_tick(buf: BufHandle) -> c_int;

    /// Set b_sst_lasttick in syn_block
    fn nvim_syn_set_sst_lasttick(tick: c_int);

    /// Get display_tick global
    fn nvim_syn_get_display_tick() -> c_int;

    /// Call line_breakcheck from Rust
    fn nvim_syn_line_breakcheck();

    /// Get got_int global
    fn nvim_syn_get_got_int() -> c_int;

    /// Get Rows global
    fn nvim_syn_get_rows() -> c_int;
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

// =============================================================================
// Phase 24.2: Core Pattern Matching Functions (FFI exports)
// =============================================================================

/// Check for end of current state (and the states before it).
/// This is a Rust wrapper that delegates to the C implementation.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_check_state_ends() {
    nvim_syn_check_state_ends();
}

/// Update an entry in the current_state stack for a match or region.
/// This fills in si_attr, si_next_list and si_cont_list.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_update_si_attr(idx: c_int) {
    nvim_syn_call_update_si_attr(idx);
}

/// Check the current stack for patterns with "keepend" flag.
/// Propagate the match-end to contained items, until a "skipend" item is found.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_check_keepend() {
    nvim_syn_check_keepend();
}

/// Pop the current state from the stack.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_pop_current_state() {
    nvim_syn_pop_current_state();
}

/// Push a new state onto the stack with the given pattern index.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_push_current_state(idx: c_int) {
    nvim_syn_push_current_state(idx);
}

/// Get the next sequence number and increment it.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_next_seqnr() -> c_int {
    nvim_syn_next_seqnr()
}

/// Get the attribute for a syntax ID (syntax crate version).
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syntax_id2attr(syn_id: c_int) -> c_int {
    nvim_syn_id2attr_wrapper(syn_id)
}

/// Check if the current state stack is empty.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_is_empty() -> c_int {
    nvim_syn_current_state_is_empty()
}

/// Get the character at the current column.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_getcurline_at_col() -> c_char {
    nvim_syn_getcurline_at_col()
}

/// Set the current_finished flag.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_set_finished(finished: c_int) {
    nvim_syn_set_current_finished(finished);
}

/// Call syn_update_ends.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_update_ends(syncing: c_int) {
    nvim_syn_call_syn_update_ends(syncing);
}

/// Set the next_match_idx value.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_set_next_match_idx(idx: c_int) {
    nvim_syn_set_next_match_idx(idx);
}

/// Set the next_match_col value.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_set_next_match_col(col: c_int) {
    nvim_syn_set_next_match_col(col);
}

/// Set the current_next_list pointer.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_set_current_next_list(list: IdListHandle) {
    nvim_syn_set_current_next_list_ptr(list);
}

/// Get the current_next_list pointer.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_current_next_list() -> IdListHandle {
    nvim_syn_get_current_next_list_ptr()
}

/// Check if a list is ID_LIST_ALL.
///
/// # Safety
/// This function is safe to call.
#[no_mangle]
pub unsafe extern "C" fn rs_is_id_list_all(list: IdListHandle) -> c_int {
    nvim_syn_is_id_list_all(list)
}

/// Get the ID_LIST_ALL constant.
///
/// # Safety
/// This function is safe to call.
#[no_mangle]
pub unsafe extern "C" fn rs_get_id_list_all() -> IdListHandle {
    nvim_syn_get_id_list_all()
}

// =============================================================================
// Stateitem position accessors (safe wrappers)
// =============================================================================

/// Get match end position lnum from a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_m_endpos_lnum(item: StateItemHandle) -> c_int {
    nvim_stateitem_get_m_endpos_lnum(item)
}

/// Get match end position col from a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_m_endpos_col(item: StateItemHandle) -> c_int {
    nvim_stateitem_get_m_endpos_col(item)
}

/// Get highlight start position lnum from a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_h_startpos_lnum(item: StateItemHandle) -> c_int {
    nvim_stateitem_get_h_startpos_lnum(item)
}

/// Get highlight start position col from a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_h_startpos_col(item: StateItemHandle) -> c_int {
    nvim_stateitem_get_h_startpos_col(item)
}

/// Get highlight end position lnum from a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_h_endpos_lnum(item: StateItemHandle) -> c_int {
    nvim_stateitem_get_h_endpos_lnum(item)
}

/// Get highlight end position col from a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_h_endpos_col(item: StateItemHandle) -> c_int {
    nvim_stateitem_get_h_endpos_col(item)
}

/// Get end-of-end position lnum from a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_eoe_pos_lnum(item: StateItemHandle) -> c_int {
    nvim_stateitem_get_eoe_pos_lnum(item)
}

/// Get end-of-end position col from a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_eoe_pos_col(item: StateItemHandle) -> c_int {
    nvim_stateitem_get_eoe_pos_col(item)
}

/// Set match end position for a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_m_endpos(item: StateItemHandle, lnum: c_int, col: c_int) {
    nvim_stateitem_set_m_endpos(item, lnum, col);
}

/// Set highlight end position for a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_h_endpos(item: StateItemHandle, lnum: c_int, col: c_int) {
    nvim_stateitem_set_h_endpos(item, lnum, col);
}

/// Set end-of-end position for a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_eoe_pos(item: StateItemHandle, lnum: c_int, col: c_int) {
    nvim_stateitem_set_eoe_pos(item, lnum, col);
}

// =============================================================================
// Stateitem field setters (safe wrappers)
// =============================================================================

/// Set the pattern index for a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_idx(item: StateItemHandle, idx: c_int) {
    nvim_stateitem_set_idx(item, idx);
}

/// Set the end pattern index for a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_end_idx(item: StateItemHandle, end_idx: c_int) {
    nvim_stateitem_set_end_idx(item, end_idx);
}

/// Set flags for a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_flags(item: StateItemHandle, flags: c_int) {
    nvim_stateitem_set_flags(item, flags);
}

/// Add flags to a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_add_flags(item: StateItemHandle, flags: c_int) {
    nvim_stateitem_add_flags(item, flags);
}

/// Set sequence number for a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_seqnr(item: StateItemHandle, seqnr: c_int) {
    nvim_stateitem_set_seqnr(item, seqnr);
}

/// Set ends flag for a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_ends(item: StateItemHandle, ends: c_int) {
    nvim_stateitem_set_ends(item, ends);
}

/// Set ID for a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_id(item: StateItemHandle, id: c_int) {
    nvim_stateitem_set_id(item, id);
}

/// Set trans_id for a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_trans_id(item: StateItemHandle, trans_id: c_int) {
    nvim_stateitem_set_trans_id(item, trans_id);
}

/// Set attr for a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_attr(item: StateItemHandle, attr: c_int) {
    nvim_stateitem_set_attr(item, attr);
}

/// Set cont_list for a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_cont_list(item: StateItemHandle, list: IdListHandle) {
    nvim_stateitem_set_cont_list(item, list);
}

/// Set next_list for a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_next_list(item: StateItemHandle, list: IdListHandle) {
    nvim_stateitem_set_next_list(item, list);
}

/// Get next_list from a state item.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_get_next_list(item: StateItemHandle) -> IdListHandle {
    nvim_stateitem_get_next_list(item)
}

// =============================================================================
// Pattern accessors (safe wrappers)
// =============================================================================

/// Get the pattern type for a pattern at index.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_pattern_type(idx: c_int) -> c_int {
    nvim_synblock_pattern_type(idx)
}

/// Get the pattern flags for a pattern at index.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_pattern_flags(idx: c_int) -> c_int {
    nvim_synblock_pattern_flags(idx)
}

/// Get the syntax ID for a pattern at index.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_pattern_syn_id(idx: c_int) -> c_int {
    nvim_synblock_pattern_syn_id(idx)
}

/// Get the match ID for a pattern at index.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_pattern_match_id(idx: c_int) -> c_int {
    nvim_synblock_pattern_match_id(idx)
}

/// Get the cont_list for a pattern at index.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_pattern_cont_list(idx: c_int) -> IdListHandle {
    nvim_synblock_pattern_cont_list(idx)
}

/// Get the next_list for a pattern at index.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_pattern_next_list(idx: c_int) -> IdListHandle {
    nvim_synblock_pattern_next_list(idx)
}

// =============================================================================
// Phase 24.3: Keyword Matching Functions (FFI exports)
// =============================================================================

/// Check for a keyword match at the given position.
/// Returns the syntax ID if a keyword is found, 0 otherwise.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_check_keyword_id(
    line: *mut c_char,
    startcol: c_int,
    endcolp: *mut c_int,
    flagsp: *mut c_int,
    next_listp: *mut IdListHandle,
    cur_si: StateItemHandle,
    ccharp: *mut c_int,
) -> c_int {
    nvim_syn_check_keyword_id(line, startcol, endcolp, flagsp, next_listp, cur_si, ccharp)
}

/// Check if an ID is in a contains or nextgroup list.
/// Returns 1 if in list, 0 otherwise.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_in_id_list(
    cur_si: StateItemHandle,
    list: IdListHandle,
    id: c_int,
    inc_tag: c_int,
    cont_in_list: IdListHandle,
    flags: c_int,
) -> c_int {
    nvim_syn_in_id_list(cur_si, list, id, inc_tag, cont_in_list, flags)
}

/// Check if there are case-sensitive keywords in the synblock.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_has_keywords() -> c_int {
    nvim_syn_has_keywords()
}

/// Check if there are case-insensitive keywords in the synblock.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_has_keywords_ic() -> c_int {
    nvim_syn_has_keywords_ic()
}

/// Get the cont_list from a stateitem.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_get_cont_list(item: StateItemHandle) -> IdListHandle {
    nvim_stateitem_get_cont_list(item)
}

/// Check if a character at position is a keyword character.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_is_keyword_char(line: *mut c_char, pos: c_int) -> c_int {
    nvim_syn_is_keyword_char(line, pos)
}

/// Get the current line being processed.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_getcurline() -> *mut c_char {
    nvim_syn_getcurline()
}

/// Save the chartab for syntax iskeyword.
///
/// # Safety
/// Buffer must be at least 32 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_save_chartab(buf: *mut c_char) {
    nvim_syn_save_chartab(buf);
}

/// Restore the chartab.
///
/// # Safety
/// Buffer must be the same as passed to rs_syn_save_chartab.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_restore_chartab(buf: *mut c_char) {
    nvim_syn_restore_chartab(buf);
}

/// Get the maximum keyword length constant.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_maxkeywlen() -> c_int {
    keyword::max_keyword_len()
}

/// Find a keyword in the hashtab.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_keyword_find(
    keyword: *mut c_char,
    use_ic: c_int,
) -> KeyEntryHandle {
    keyword::keyword_find(keyword, use_ic != 0)
}

/// Match a keyword against the hashtab.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_match_keyword(
    keyword: *mut c_char,
    use_ic: c_int,
    cur_si: StateItemHandle,
) -> KeyEntryHandle {
    keyword::match_keyword(keyword, use_ic != 0, cur_si)
}

/// Fold case for keyword comparison.
///
/// # Safety
/// src and dst must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_keyword_foldcase(
    src: *mut c_char,
    srclen: c_int,
    dst: *mut c_char,
    dstlen: c_int,
) {
    keyword::keyword_foldcase(src, srclen, dst, dstlen);
}

/// Get UTF character length.
///
/// # Safety
/// p must be a valid pointer to a UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_utfc_ptr2len(p: *mut c_char) -> c_int {
    keyword::utfc_ptr2len(p)
}

/// Get the syn_buf pointer.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_buf() -> *mut std::ffi::c_void {
    nvim_syn_get_buf()
}

// =============================================================================
// Phase 24.4: Pattern Stack Operations Exports (new unique functions only)
// =============================================================================

/// Get the current_state stack length.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_current_state_len() -> c_int {
    nvim_syn_current_state_len()
}

/// Get a stateitem from current_state by index.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_stateitem(index: c_int) -> StateItemHandle {
    nvim_syn_get_stateitem(index)
}

/// Get the top stateitem from current_state.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_top_stateitem() -> StateItemHandle {
    nvim_syn_get_top_stateitem()
}

/// Get the next_seqnr global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_next_seqnr() -> c_int {
    nvim_syn_get_next_seqnr()
}

/// Set the next_seqnr global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_set_next_seqnr(seqnr: c_int) {
    nvim_syn_set_next_seqnr(seqnr);
}

/// Increment and return the next_seqnr global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_incr_next_seqnr() -> c_int {
    nvim_syn_incr_next_seqnr()
}

/// Get next_match_h_startpos.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_next_match_h_startpos(lnum: *mut c_int, col: *mut c_int) {
    nvim_syn_get_next_match_h_startpos(lnum, col);
}

/// Get next_match_m_endpos.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_next_match_m_endpos(lnum: *mut c_int, col: *mut c_int) {
    nvim_syn_get_next_match_m_endpos(lnum, col);
}

/// Get next_match_h_endpos.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_next_match_h_endpos(lnum: *mut c_int, col: *mut c_int) {
    nvim_syn_get_next_match_h_endpos(lnum, col);
}

/// Get next_match_eos_pos.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_next_match_eos_pos(lnum: *mut c_int, col: *mut c_int) {
    nvim_syn_get_next_match_eos_pos(lnum, col);
}

/// Get next_match_eoe_pos.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_next_match_eoe_pos(lnum: *mut c_int, col: *mut c_int) {
    nvim_syn_get_next_match_eoe_pos(lnum, col);
}

/// Get the next_match_flags global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_next_match_flags() -> c_int {
    nvim_syn_get_next_match_flags()
}

/// Get the next_match_end_idx global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_next_match_end_idx() -> c_int {
    nvim_syn_get_next_match_end_idx()
}

/// Get the next_match_extmatch global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_next_match_extmatch() -> ExtMatchHandle {
    nvim_syn_get_next_match_extmatch()
}

/// Update state item end position.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_update_si_end(sip: StateItemHandle, startcol: c_int, force: c_int) {
    nvim_syn_update_si_end(sip, startcol, force);
}

/// Push next match onto state stack.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_push_next_match() -> StateItemHandle {
    nvim_syn_push_next_match()
}

/// Find end position of a syntax region.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_find_endpos(
    idx: c_int,
    start_lnum: c_int,
    start_col: c_int,
    m_end_lnum: *mut c_int,
    m_end_col: *mut c_int,
    hl_end_lnum: *mut c_int,
    hl_end_col: *mut c_int,
    flagsp: *mut c_int,
    end_end_lnum: *mut c_int,
    end_end_col: *mut c_int,
    end_idx: *mut c_int,
    start_ext: ExtMatchHandle,
) {
    nvim_syn_find_endpos(
        idx,
        start_lnum,
        start_col,
        m_end_lnum,
        m_end_col,
        hl_end_lnum,
        hl_end_col,
        flagsp,
        end_end_lnum,
        end_end_col,
        end_idx,
        start_ext,
    );
}

/// Get pattern flags by index.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_pattern_flags(idx: c_int) -> c_int {
    nvim_syn_get_pattern_flags(idx)
}

/// Get pattern cchar by index.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_pattern_cchar(idx: c_int) -> c_int {
    nvim_syn_get_pattern_cchar(idx)
}

/// Get pattern next_list by index.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_pattern_next_list(idx: c_int) -> IdListHandle {
    nvim_syn_get_pattern_next_list(idx)
}

/// Get pattern type by index.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_pattern_type(idx: c_int) -> c_int {
    nvim_syn_get_pattern_type(idx)
}

/// Get pattern syn_match_id by index.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_pattern_syn_match_id(idx: c_int) -> c_int {
    nvim_syn_get_pattern_syn_match_id(idx)
}

/// Check if current_state is empty.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_is_current_state_empty() -> c_int {
    nvim_syn_is_current_state_empty()
}

/// Set stateitem h_startpos.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_h_startpos(
    item: StateItemHandle,
    lnum: c_int,
    col: c_int,
) {
    nvim_stateitem_set_h_startpos(item, lnum, col);
}

/// Set stateitem m_startcol.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_m_startcol(item: StateItemHandle, col: c_int) {
    nvim_stateitem_set_m_startcol(item, col);
}

/// Set stateitem m_lnum.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_m_lnum(item: StateItemHandle, lnum: c_int) {
    nvim_stateitem_set_m_lnum(item, lnum);
}

/// Or stateitem flags with a value.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_or_flags(item: StateItemHandle, flags: c_int) {
    nvim_stateitem_or_flags(item, flags);
}

/// Set stateitem cchar.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_cchar(item: StateItemHandle, cchar: c_int) {
    nvim_stateitem_set_cchar(item, cchar);
}

/// Set stateitem extmatch.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_extmatch(item: StateItemHandle, em: ExtMatchHandle) {
    nvim_stateitem_set_extmatch(item, em);
}

/// Get SPTYPE_START constant.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_sptype_start() -> c_int {
    nvim_syn_get_sptype_start()
}

/// Get HL_ONELINE constant.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_hl_oneline() -> c_int {
    nvim_syn_get_hl_oneline()
}

/// Get HL_KEEPEND constant.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_hl_keepend() -> c_int {
    nvim_syn_get_hl_keepend()
}

/// Get HL_MATCH constant.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_hl_match() -> c_int {
    nvim_syn_get_hl_match()
}

/// Get HL_CONCEAL constant.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_hl_conceal() -> c_int {
    nvim_syn_get_hl_conceal()
}

/// Get HL_CONCEALENDS constant.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_hl_concealends() -> c_int {
    nvim_syn_get_hl_concealends()
}

// =============================================================================
// Phase 24.5: Sync and Line Operations Exports
// =============================================================================

/// Call syn_start_line to prepare current state for start of a line.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_start_line() {
    nvim_syn_start_line();
}

/// Call syn_finish_line to process to end of current line.
/// Returns 1 if a sync point was found (when syncing), 0 otherwise.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_finish_line(syncing: c_int) -> c_int {
    nvim_syn_finish_line(syncing)
}

/// Call syn_sync to synchronize syntax state for a line.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_sync(wp: WinHandle, start_lnum: c_int, last_valid: SynStateHandle) {
    nvim_syn_sync(wp, start_lnum, last_valid);
}

/// Call syntax_start - main entry point for syntax highlighting.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syntax_start(wp: WinHandle, lnum: c_int) {
    nvim_syntax_start(wp, lnum);
}

/// Call clear_current_state to clean up the current state stack.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_clear_current_state() {
    nvim_syn_clear_current_state();
}

/// Call clear_syn_state to clean up a synstate entry.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_clear_syn_state(p: SynStateHandle) {
    nvim_syn_clear_syn_state(p);
}

/// Get current_finished global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_current_finished() -> c_int {
    nvim_syn_is_current_finished()
}

/// Set current_finished global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_set_current_finished(finished: c_int) {
    nvim_syn_set_current_finished(finished);
}

/// Get current_line_id global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_current_line_id() -> c_int {
    nvim_syn_get_current_line_id()
}

/// Increment current_line_id global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_incr_current_line_id() {
    nvim_syn_incr_current_line_id();
}

/// Get syn_block global pointer.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_syn_block() -> SynBlockHandle {
    nvim_syn_get_syn_block()
}

/// Set syn_block global pointer.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_set_syn_block(block: SynBlockHandle) {
    nvim_syn_set_syn_block(block);
}

/// Get syn_win global pointer.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_syn_win() -> WinHandle {
    nvim_syn_get_syn_win()
}

/// Set syn_win global pointer.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_set_syn_win(win: WinHandle) {
    nvim_syn_set_syn_win(win);
}

/// Get b_syn_sync_minlines from syn_block.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_sync_minlines() -> c_int {
    nvim_syn_get_sync_minlines()
}

/// Get b_syn_sync_maxlines from syn_block.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_sync_maxlines() -> c_int {
    nvim_syn_get_sync_maxlines()
}

/// Get b_syn_sync_flags from syn_block.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_sync_flags() -> c_int {
    nvim_syn_get_sync_flags()
}

/// Get b_syn_sync_id from syn_block.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_sync_id() -> c_int {
    nvim_syn_get_sync_id()
}

/// Get b_sst_first from syn_block (first in valid state list).
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_sst_first() -> SynStateHandle {
    nvim_syn_get_sst_first()
}

/// Get b_sst_array from syn_block.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_sst_array() -> *mut std::ffi::c_void {
    nvim_syn_get_sst_array()
}

/// Get b_sst_len from syn_block.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_sst_len() -> c_int {
    nvim_syn_get_sst_len()
}

/// Set synstate sst_change_lnum.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_synstate_set_change_lnum(p: SynStateHandle, lnum: c_int) {
    nvim_synstate_set_change_lnum(p, lnum);
}

/// Call validate_current_state to ensure current_state is valid.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_current_state() {
    nvim_syn_validate_current_state();
}

/// Get current_id global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_current_id() -> c_int {
    nvim_syn_get_current_id()
}

/// Set current_id global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_set_current_id(id: c_int) {
    nvim_syn_set_current_id(id);
}

/// Get current_trans_id global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_current_trans_id() -> c_int {
    nvim_syn_get_current_trans_id()
}

/// Set current_trans_id global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_set_current_trans_id(id: c_int) {
    nvim_syn_set_current_trans_id(id);
}

/// Get current_flags global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_current_flags() -> c_int {
    nvim_syn_get_current_flags()
}

/// Set current_flags global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_set_current_flags(flags: c_int) {
    nvim_syn_set_current_flags(flags);
}

/// Get current_seqnr global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_current_seqnr() -> c_int {
    nvim_syn_get_current_seqnr()
}

/// Set current_seqnr global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_set_current_seqnr(seqnr: c_int) {
    nvim_syn_set_current_seqnr(seqnr);
}

/// Get HL_MATCHCONT constant.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_hl_matchcont() -> c_int {
    nvim_syn_get_hl_matchcont()
}

/// Get HL_EXTEND constant.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_hl_extend() -> c_int {
    nvim_syn_get_hl_extend()
}

/// Get SF_CCOMMENT constant.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_sf_ccomment() -> c_int {
    nvim_syn_get_sf_ccomment()
}

/// Get SF_MATCH constant.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_sf_match() -> c_int {
    nvim_syn_get_sf_match()
}

/// Get HL_SYNC_HERE constant.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_hl_sync_here() -> c_int {
    nvim_syn_get_hl_sync_here()
}

/// Get HL_SYNC_THERE constant.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_hl_sync_there() -> c_int {
    nvim_syn_get_hl_sync_there()
}

/// Get SPTYPE_MATCH constant.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_sptype_match() -> c_int {
    nvim_syn_get_sptype_match()
}

/// Call syn_stack_alloc to allocate syntax stack.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_stack_alloc() {
    nvim_syn_stack_alloc();
}

/// Call syn_stack_find_entry to find a synstate entry for a line.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_stack_find_entry_24_5(lnum: c_int) -> SynStateHandle {
    nvim_syn_stack_find_entry_ptr(lnum)
}

/// Get w_s (synblock) from window.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_win_get_synblock_24_5(wp: WinHandle) -> SynBlockHandle {
    nvim_win_get_synblock(wp)
}

/// Get w_buffer from window.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_win_get_buffer_24_5(wp: WinHandle) -> BufHandle {
    nvim_syn_win_get_buffer_ptr(wp)
}

/// Get ml_line_count from buffer.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_get_ml_line_count_24_5(buf: BufHandle) -> c_int {
    nvim_syn_buf_get_line_count(buf)
}

/// Get changedtick from buffer.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_get_changed_tick_24_5(buf: BufHandle) -> c_int {
    nvim_syn_buf_get_changed_tick(buf)
}

/// Set b_sst_lasttick in syn_block.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_set_sst_lasttick(tick: c_int) {
    nvim_syn_set_sst_lasttick(tick);
}

/// Get display_tick global.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_display_tick() -> c_int {
    nvim_syn_get_display_tick()
}

/// Call line_breakcheck to check for user interrupt.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_line_breakcheck() {
    nvim_syn_line_breakcheck();
}

/// Get got_int global (whether CTRL-C was pressed).
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_got_int() -> c_int {
    nvim_syn_get_got_int()
}

/// Get Rows global (for syntax).
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_rows() -> c_int {
    nvim_syn_get_rows()
}

// =============================================================================
// Phase 32.1: Stack management exports
// =============================================================================

/// Free all syntax state entries for a synblock.
///
/// # Safety
/// The caller must ensure the synblock handle is valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_stack_free_all(block: SynBlockHandle) {
    state::stack_free_all(block);
}

/// Apply buffer changes to syntax states.
///
/// # Safety
/// The caller must ensure the buffer handle is valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_stack_apply_changes(buf: BufHandle) {
    state::stack_apply_changes(buf);
}

/// Get the line where a buffer change starts.
///
/// # Safety
/// The caller must ensure the buffer handle is valid.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_mod_top(buf: BufHandle) -> c_int {
    state::buf_mod_top(buf)
}

/// Get the line after a buffer change.
///
/// # Safety
/// The caller must ensure the buffer handle is valid.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_mod_bot(buf: BufHandle) -> c_int {
    state::buf_mod_bot(buf)
}

/// Get the number of extra lines from a buffer change.
///
/// # Safety
/// The caller must ensure the buffer handle is valid.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_mod_xlines(buf: BufHandle) -> c_int {
    state::buf_mod_xlines(buf)
}

/// Get the sync linebreaks setting from a synblock.
///
/// # Safety
/// The caller must ensure the synblock handle is valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_linebreaks(block: SynBlockHandle) -> c_int {
    state::synblock_linebreaks(block)
}

/// Set the line number for a syntax state.
///
/// # Safety
/// The caller must ensure the synstate handle is valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synstate_set_lnum(state: SynStateHandle, lnum: c_int) {
    state::synstate_set_lnum(state, lnum);
}

/// Check if two synstates have equal next_list pointers.
///
/// # Safety
/// The caller must ensure the synstate handles are valid (or null).
#[no_mangle]
pub unsafe extern "C" fn rs_synstate_next_list_eq(a: SynStateHandle, b: SynStateHandle) -> c_int {
    if state::synstate_next_list_eq(a, b) {
        1
    } else {
        0
    }
}

// =============================================================================
// Phase 32.2: Group management exports
// =============================================================================

/// Get the syntax group ID from a pattern.
///
/// # Safety
/// The caller must ensure the pattern handle is valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synpat_syn_id(pat: SynPatHandle) -> i16 {
    group::synpat_syn_id(pat)
}

/// Get the match group ID from a pattern.
///
/// # Safety
/// The caller must ensure the pattern handle is valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synpat_match_id(pat: SynPatHandle) -> i16 {
    group::synpat_match_id(pat)
}

/// Get the topgrp setting for a synblock.
///
/// # Safety
/// The caller must ensure the synblock handle is valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_topgrp(block: SynBlockHandle) -> c_int {
    group::synblock_topgrp(block)
}

/// Get the current highlight group ID.
///
/// # Safety
/// Must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_group_current_id() -> c_int {
    group::current_id()
}

/// Get the current transparent group ID.
///
/// # Safety
/// Must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_group_current_trans_id() -> c_int {
    group::current_trans_id()
}

/// Get the syntax ID at a position in the buffer.
///
/// # Safety
/// The window handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_get_syntax_id(
    wp: WinHandle,
    lnum: c_int,
    col: c_int,
    trans: c_int,
    keep_state: c_int,
) -> c_int {
    group::get_syntax_id(wp, lnum, col, trans != 0, keep_state != 0)
}

// =============================================================================
// Phase 32.3: Cluster operations exports
// =============================================================================

/// Lookup a cluster by name and return its ID.
///
/// # Safety
/// The name pointer must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cluster_name_to_id(name: *const c_char) -> c_int {
    cluster::cluster_name_to_id(name)
}

/// Check if the synblock has any containedin items.
///
/// # Safety
/// The synblock handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_has_containedin(block: SynBlockHandle) -> c_int {
    if cluster::synblock_has_containedin(block) {
        1
    } else {
        0
    }
}

/// Get the pattern count for a synblock.
///
/// # Safety
/// The synblock handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_pattern_count(block: SynBlockHandle) -> c_int {
    cluster::synblock_pattern_count(block)
}

/// Get the inc_tag from a pattern.
///
/// # Safety
/// The pattern handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synpat_inc_tag(pat: SynPatHandle) -> c_int {
    cluster::synpat_inc_tag(pat)
}

/// Check if a cluster ID is the @Spell cluster.
///
/// # Safety
/// The synblock handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_is_spell_cluster(block: SynBlockHandle, id: c_int) -> c_int {
    if cluster::is_spell_cluster(block, id) {
        1
    } else {
        0
    }
}

/// Check if a cluster ID is the @NoSpell cluster.
///
/// # Safety
/// The synblock handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_is_nospell_cluster(block: SynBlockHandle, id: c_int) -> c_int {
    if cluster::is_nospell_cluster(block, id) {
        1
    } else {
        0
    }
}

/// Check if an ID is a special spell-related cluster.
///
/// # Safety
/// The synblock handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_is_spell_related_cluster(block: SynBlockHandle, id: c_int) -> c_int {
    if cluster::is_spell_related_cluster(block, id) {
        1
    } else {
        0
    }
}

// =============================================================================
// Phase 32.4: Line highlighting exports
// =============================================================================

/// Get syntax attributes at a column.
///
/// # Safety
/// Must be called after syntax_start for the current window/line.
#[no_mangle]
pub unsafe extern "C" fn rs_get_syntax_attr(
    col: c_int,
    can_spell: *mut c_int,
    keep_state: c_int,
) -> c_int {
    let result = highlight::get_syntax_attr(col, keep_state != 0);
    if !can_spell.is_null() {
        *can_spell = if result.can_spell { 1 } else { 0 };
    }
    result.attr
}

/// Set the current column for processing.
///
/// # Safety
/// Must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_set_current_col(col: c_int) {
    highlight::set_current_col(col);
}

/// Check if current state has been stored.
///
/// # Safety
/// Must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_current_state_stored() -> c_int {
    if highlight::current_state_stored() {
        1
    } else {
        0
    }
}

/// Get the spell setting for a synblock.
///
/// # Safety
/// The synblock handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_syn_spell(block: SynBlockHandle) -> c_int {
    highlight::synblock_syn_spell(block)
}

/// Get the synmaxcol setting from a buffer.
///
/// # Safety
/// The buffer handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_synmaxcol(buf: BufHandle) -> c_int {
    highlight::buf_synmaxcol(buf)
}

/// Ensure current state is valid, validating if needed.
///
/// # Safety
/// Must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_ensure_current_state_valid() {
    highlight::ensure_current_state_valid();
}

// Note: rs_syn_current_col, rs_syn_is_finished, rs_syn_is_state_valid,
// rs_syn_next_match_idx, rs_syn_next_match_col, rs_syn_has_next_match
// are already defined earlier in this file.

// =============================================================================
// Phase 32.5: Buffer attachment exports
// =============================================================================

// Note: rs_syntax_start already defined at line ~3893
// Note: rs_buf_mod_top, rs_buf_mod_bot, rs_buf_mod_xlines already defined at lines ~4329-4347

/// Check if synblock has an error.
///
/// # Safety
/// The synblock handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_has_error(block: SynBlockHandle) -> c_int {
    if buffer::synblock_has_error(block) {
        1
    } else {
        0
    }
}

/// Check if synblock is running slowly.
///
/// # Safety
/// The synblock handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_is_slow(block: SynBlockHandle) -> c_int {
    if buffer::synblock_is_slow(block) {
        1
    } else {
        0
    }
}

/// Check if syntax highlighting is active.
///
/// # Safety
/// The synblock handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_is_active(block: SynBlockHandle) -> c_int {
    if buffer::synblock_is_active(block) {
        1
    } else {
        0
    }
}

/// Get number of fold items.
///
/// # Safety
/// The synblock handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_folditems(block: SynBlockHandle) -> c_int {
    buffer::synblock_folditems(block)
}

/// Get fold level mode.
///
/// # Safety
/// The synblock handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_foldlevel_mode(block: SynBlockHandle) -> c_int {
    buffer::synblock_foldlevel_mode(block).to_raw()
}

/// Check if concealing is enabled.
///
/// # Safety
/// The synblock handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_conceal_enabled(block: SynBlockHandle) -> c_int {
    if buffer::synblock_conceal_enabled(block) {
        1
    } else {
        0
    }
}

// rs_buf_mod_top, rs_buf_mod_bot, rs_buf_mod_xlines already defined above

/// Apply buffer changes to syntax state.
///
/// # Safety
/// The buffer handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_apply_buffer_changes(buf: BufHandle) {
    buffer::apply_buffer_changes(buf);
}

/// Get the current syntax buffer.
///
/// # Safety
/// Must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_current_syn_buf() -> BufHandle {
    buffer::current_syn_buf()
}

/// Get the current synblock.
///
/// # Safety
/// Must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_current_syn_block() -> SynBlockHandle {
    buffer::current_syn_block()
}

/// Get the current syntax window.
///
/// # Safety
/// Must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_current_syn_win() -> WinHandle {
    buffer::current_syn_win()
}

/// Get current fold level from syntax state.
///
/// # Safety
/// Must be called during syntax highlighting.
#[no_mangle]
pub unsafe extern "C" fn rs_current_foldlevel() -> c_int {
    buffer::current_foldlevel()
}

/// Check if syntax-based folding is available.
///
/// # Safety
/// The synblock handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_can_compute_folds(block: SynBlockHandle) -> c_int {
    if buffer::can_compute_folds(block) {
        1
    } else {
        0
    }
}

// =============================================================================
// Phase 32.6: Ex commands exports
// =============================================================================

/// Get the command line pointer.
///
/// # Safety
/// Must be called during command execution.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmdlinep() -> *mut *mut c_char {
    commands::cmdlinep()
}

// Note: rs_curwin_synblock already defined at line ~2543

/// Get current window handle.
///
/// # Safety
/// Must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_curwin() -> WinHandle {
    commands::curwin()
}

/// Get include_link flag for completion.
#[no_mangle]
pub extern "C" fn rs_syn_include_link() -> c_int {
    if commands::include_link() {
        1
    } else {
        0
    }
}

/// Get include_default flag for completion.
#[no_mangle]
pub extern "C" fn rs_syn_include_default() -> c_int {
    if commands::include_default() {
        1
    } else {
        0
    }
}

/// Get include_none flag for completion.
#[no_mangle]
pub extern "C" fn rs_syn_include_none() -> c_int {
    if commands::include_none() {
        1
    } else {
        0
    }
}

/// Get running include tag.
#[no_mangle]
pub extern "C" fn rs_syn_running_inc_tag() -> c_int {
    commands::running_inc_tag()
}

/// Set running include tag.
///
/// # Safety
/// Must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_set_running_inc_tag(tag: c_int) {
    commands::set_running_inc_tag(tag);
}

/// Get conceal setting for synblock.
///
/// # Safety
/// The synblock handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_conceal_setting(block: SynBlockHandle) -> c_int {
    if commands::synblock_conceal_setting(block) {
        1
    } else {
        0
    }
}

/// Get case-insensitive setting for synblock.
///
/// # Safety
/// The synblock handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_ic_setting(block: SynBlockHandle) -> c_int {
    if commands::synblock_ic_setting(block) {
        1
    } else {
        0
    }
}

// =============================================================================
// Phase 143: Syntax State Machine Migration
// =============================================================================

extern "C" {
    // Additional C accessors for Phase 143
    fn nvim_syn_get_id(
        wp: WinHandle,
        lnum: c_int,
        col: c_int,
        trans: c_int,
        spellp: *mut c_int,
        keep_state: c_int,
    ) -> c_int;

    fn nvim_syn_get_concealed_id(wp: WinHandle, lnum: c_int, col: c_int) -> c_int;

    fn nvim_get_syntax_info(seqnrp: *mut c_int) -> c_int;

    fn nvim_syn_get_foldlevel(wp: WinHandle, lnum: c_int) -> c_int;

    fn nvim_syntax_end_parsing(wp: WinHandle, lnum: c_int);

    fn nvim_syn_stack_cleanup() -> c_int;

    fn nvim_syn_buf_get_synmaxcol() -> c_int;

    fn nvim_syn_get_sync_linebreaks() -> c_int;

    fn nvim_synstate_set_tick(state: SynStateHandle, tick: c_int);

    fn nvim_synstate_get_tick_val(state: SynStateHandle) -> c_int;

    // Note: nvim_stateitem_get_m_lnum, nvim_stateitem_get_m_startcol,
    // nvim_stateitem_set_m_lnum, nvim_stateitem_set_m_startcol,
    // nvim_stateitem_get_end_idx, nvim_stateitem_get_ends are already
    // declared in the main extern block near the top of the file.
}

/// Get syntax ID at a file position.
/// This is the main entry point for getting syntax highlighting info.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_id(
    wp: WinHandle,
    lnum: c_int,
    col: c_int,
    trans: c_int,
    spellp: *mut c_int,
    keep_state: c_int,
) -> c_int {
    nvim_syn_get_id(wp, lnum, col, trans, spellp, keep_state)
}

/// Return the syntax ID at position "i" in the current stack.
/// The caller must have called syn_get_id() before to fill the stack.
/// Returns -1 when "i" is out of range.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_stack_item(i: c_int) -> c_int {
    let state_len = nvim_syn_get_current_state_len();
    if i >= state_len {
        nvim_syn_invalidate_current_state();
        return -1;
    }

    let item = nvim_syn_get_cur_state(i);
    if item.is_null() {
        return -1;
    }
    nvim_stateitem_get_id(item)
}

/// Get extra information about the syntax item.
/// Must be called right after get_syntax_attr().
/// Returns the current flags.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_syntax_info(seqnrp: *mut c_int) -> c_int {
    nvim_get_syntax_info(seqnrp)
}

/// Get the sequence number of the concealed file position.
/// Returns seqnr if the file position is concealed, 0 otherwise.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_concealed_id(wp: WinHandle, lnum: c_int, col: c_int) -> c_int {
    nvim_syn_get_concealed_id(wp, lnum, col)
}

/// Get folding level for line "lnum" in window "wp".
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_foldlevel(wp: WinHandle, lnum: c_int) -> c_int {
    nvim_syn_get_foldlevel(wp, lnum)
}

/// End parsing syntax above line "lnum".
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syntax_end_parsing(wp: WinHandle, lnum: c_int) {
    nvim_syntax_end_parsing(wp, lnum);
}

/// Clean up the syntax state stack.
/// Returns 1 if an entry was freed, 0 otherwise.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_stack_cleanup() -> c_int {
    nvim_syn_stack_cleanup()
}

/// Get the synmaxcol setting for the current syntax buffer.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_buf_get_synmaxcol() -> c_int {
    nvim_syn_buf_get_synmaxcol()
}

/// Get the sync linebreaks from the current synblock.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_sync_linebreaks() -> c_int {
    nvim_syn_get_sync_linebreaks()
}

/// Set the tick value for a synstate.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_synstate_set_tick(state: SynStateHandle, tick: c_int) {
    if !state.is_null() {
        nvim_synstate_set_tick(state, tick);
    }
}

/// Get the tick value for a synstate.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_synstate_get_tick(state: SynStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    nvim_synstate_get_tick_val(state)
}

/// Get stateitem m_lnum field.
///
/// # Safety
/// The item handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_get_m_lnum(item: StateItemHandle) -> c_int {
    if item.is_null() {
        return 0;
    }
    nvim_stateitem_get_m_lnum(item)
}

/// Get stateitem m_startcol field.
///
/// # Safety
/// The item handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_get_m_startcol(item: StateItemHandle) -> c_int {
    if item.is_null() {
        return 0;
    }
    nvim_stateitem_get_m_startcol(item)
}

/// Set stateitem m_lnum field.
///
/// # Safety
/// The item handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_m_lnum_p143(item: StateItemHandle, lnum: c_int) {
    if !item.is_null() {
        nvim_stateitem_set_m_lnum(item, lnum);
    }
}

/// Set stateitem m_startcol field.
///
/// # Safety
/// The item handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_set_m_startcol_p143(item: StateItemHandle, col: c_int) {
    if !item.is_null() {
        nvim_stateitem_set_m_startcol(item, col);
    }
}

/// Get stateitem end_idx field.
///
/// # Safety
/// The item handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_get_end_idx_p143(item: StateItemHandle) -> c_int {
    if item.is_null() {
        return 0;
    }
    nvim_stateitem_get_end_idx(item)
}

/// Get stateitem ends field.
///
/// # Safety
/// The item handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_get_ends_p143(item: StateItemHandle) -> c_int {
    if item.is_null() {
        return 0;
    }
    nvim_stateitem_get_ends(item)
}

// Note: Many constants and accessors defined earlier in this file are re-exported
// through these Rust wrappers for Phase 143 compatibility.

/// Get KEYWORD_IDX constant.
#[no_mangle]
pub extern "C" fn rs_get_keyword_idx() -> c_int {
    KEYWORD_IDX
}

/// Get NONE_IDX constant.
#[no_mangle]
pub extern "C" fn rs_get_none_idx() -> c_int {
    NONE_IDX
}

/// Get SST_FIX_STATES constant.
#[no_mangle]
pub extern "C" fn rs_get_sst_fix_states() -> c_int {
    SST_FIX_STATES
}

/// Get SST_DIST constant.
#[no_mangle]
pub extern "C" fn rs_get_sst_dist() -> c_int {
    SST_DIST
}

/// Get SST_MIN_ENTRIES constant.
#[no_mangle]
pub extern "C" fn rs_get_sst_min_entries() -> c_int {
    SST_MIN_ENTRIES
}

/// Get SST_MAX_ENTRIES constant.
#[no_mangle]
pub extern "C" fn rs_get_sst_max_entries() -> c_int {
    SST_MAX_ENTRIES
}

// Note: rs_syn_set_current_col, rs_syn_get_current_col,
// rs_syn_get_current_lnum, rs_syn_set_current_lnum are already defined
// earlier in the file.

/// Get current state stored flag.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_get_current_state_stored() -> c_int {
    nvim_syn_is_current_state_stored()
}

/// Set current state stored flag.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_set_current_state_stored(stored: c_int) {
    nvim_syn_set_state_stored(stored);
}

/// Get stateitem flags field.
///
/// # Safety
/// The item handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_get_flags(item: StateItemHandle) -> c_int {
    if item.is_null() {
        return 0;
    }
    nvim_stateitem_get_flags(item)
}

/// Get stateitem idx field.
///
/// # Safety
/// The item handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_get_idx(item: StateItemHandle) -> c_int {
    if item.is_null() {
        return 0;
    }
    nvim_stateitem_get_idx(item)
}

/// Get stateitem attr field.
///
/// # Safety
/// The item handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_get_attr(item: StateItemHandle) -> c_int {
    if item.is_null() {
        return 0;
    }
    nvim_stateitem_get_attr(item)
}

/// Get stateitem seqnr field.
///
/// # Safety
/// The item handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_stateitem_get_seqnr(item: StateItemHandle) -> c_int {
    if item.is_null() {
        return 0;
    }
    nvim_stateitem_get_seqnr(item)
}

// Note: rs_stateitem_get_cchar, rs_stateitem_get_end_idx, rs_stateitem_get_ends,
// rs_stateitem_get_m_lnum, rs_stateitem_get_m_startcol,
// rs_stateitem_set_m_lnum, rs_stateitem_set_m_startcol,
// rs_stateitem_set_cchar, rs_stateitem_set_h_startpos are already
// defined earlier in the file.

/// Get synstate stacksize field.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synstate_get_stacksize(state: SynStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    nvim_synstate_get_stacksize(state)
}

/// Get synstate next_flags field.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synstate_get_next_flags(state: SynStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    nvim_synstate_get_next_flags(state)
}

/// Get synstate change_lnum field.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synstate_get_change_lnum_val(state: SynStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    nvim_synstate_get_change_lnum(state)
}

// Note: rs_synstate_set_change_lnum is already defined earlier in the file.

/// Get synstate next pointer.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synstate_get_next(state: SynStateHandle) -> SynStateHandle {
    if state.is_null() {
        return SynStateHandle::null();
    }
    nvim_synstate_get_next(state)
}

/// Get synstate next_list pointer.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synstate_get_next_list(state: SynStateHandle) -> IdListHandle {
    if state.is_null() {
        return IdListHandle::null();
    }
    nvim_synstate_get_next_list(state)
}

/// Get synblock sst_first pointer.
///
/// # Safety
/// The block handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_get_sst_first(block: SynBlockHandle) -> SynStateHandle {
    if block.is_null() {
        return SynStateHandle::null();
    }
    nvim_synblock_get_sst_first(block)
}

/// Get synblock sst_firstfree pointer.
///
/// # Safety
/// The block handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_get_sst_firstfree(block: SynBlockHandle) -> SynStateHandle {
    if block.is_null() {
        return SynStateHandle::null();
    }
    nvim_synblock_get_sst_firstfree(block)
}

/// Check if synblock has an sst_array.
///
/// # Safety
/// The block handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_has_sst_array(block: SynBlockHandle) -> c_int {
    if block.is_null() {
        return 0;
    }
    nvim_synblock_has_sst_array(block)
}

/// Get synblock sst_len.
///
/// # Safety
/// The block handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_get_sst_len(block: SynBlockHandle) -> c_int {
    if block.is_null() {
        return 0;
    }
    nvim_synblock_get_sst_len(block)
}

/// Get synblock sst_freecount.
///
/// # Safety
/// The block handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_get_sst_freecount(block: SynBlockHandle) -> c_int {
    if block.is_null() {
        return 0;
    }
    nvim_synblock_get_sst_freecount(block)
}

/// Get synblock sync_minlines.
///
/// # Safety
/// The block handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_get_sync_minlines(block: SynBlockHandle) -> c_int {
    if block.is_null() {
        return 0;
    }
    nvim_synblock_get_sync_minlines(block)
}

/// Get synblock sync_maxlines.
///
/// # Safety
/// The block handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_get_sync_maxlines(block: SynBlockHandle) -> c_int {
    if block.is_null() {
        return 0;
    }
    nvim_synblock_get_sync_maxlines(block)
}

/// Get synblock sync_flags.
///
/// # Safety
/// The block handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_get_sync_flags(block: SynBlockHandle) -> c_int {
    if block.is_null() {
        return 0;
    }
    nvim_synblock_get_sync_flags(block)
}

/// Get synblock sync_id.
///
/// # Safety
/// The block handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_get_sync_id(block: SynBlockHandle) -> i16 {
    if block.is_null() {
        return 0;
    }
    nvim_synblock_get_sync_id(block)
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

// =============================================================================
// Phase 82: Syntax Highlighting Engine Helpers
// =============================================================================

/// Syntax state stack management flags
pub mod state_flags {
    use std::os::raw::c_int;

    /// State is valid
    pub const VALID: c_int = 0x01;
    /// State has been modified
    pub const MODIFIED: c_int = 0x02;
    /// State needs re-sync
    pub const NEED_SYNC: c_int = 0x04;
    /// State is cached
    pub const CACHED: c_int = 0x08;
    /// State includes continuation
    pub const CONTINUED: c_int = 0x10;
    /// State at end of line
    pub const EOL: c_int = 0x20;
}

/// Check if state has a specific flag.
#[no_mangle]
pub const extern "C" fn rs_syn_state_has_flag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

/// Set a state flag.
#[no_mangle]
pub const extern "C" fn rs_syn_state_set_flag(flags: c_int, flag: c_int) -> c_int {
    flags | flag
}

/// Clear a state flag.
#[no_mangle]
pub const extern "C" fn rs_syn_state_clear_flag(flags: c_int, flag: c_int) -> c_int {
    flags & !flag
}

/// Sync method types
pub mod sync_method {
    use std::os::raw::c_int;

    /// No sync method specified
    pub const NONE: c_int = 0;
    /// Sync from cursor position
    pub const CCOMMENT: c_int = 1;
    /// Sync using linebreaks
    pub const LINEBREAKS: c_int = 2;
    /// Sync from start of buffer
    pub const FROMSTART: c_int = 3;
    /// Sync to match patterns
    pub const MATCH: c_int = 4;
    /// Minimum number of lines
    pub const MINLINES: c_int = 5;
    /// Maximum number of lines
    pub const MAXLINES: c_int = 6;
}

/// Check if sync method is line-based.
#[no_mangle]
pub const extern "C" fn rs_syn_sync_is_line_based(method: c_int) -> bool {
    matches!(
        method,
        x if x == sync_method::LINEBREAKS
            || x == sync_method::MINLINES
            || x == sync_method::MAXLINES
    )
}

/// Check if sync method scans from start.
#[no_mangle]
pub const extern "C" fn rs_syn_sync_from_start(method: c_int) -> bool {
    method == sync_method::FROMSTART
}

// =============================================================================
// Line State Helpers
// =============================================================================

/// Syntax line processing states
pub mod line_state {
    use std::os::raw::c_int;

    /// Not started
    pub const INIT: c_int = 0;
    /// Processing in progress
    pub const ACTIVE: c_int = 1;
    /// Finished processing
    pub const DONE: c_int = 2;
    /// Error occurred
    pub const ERROR: c_int = 3;
}

/// Check if line state indicates processing is needed.
#[no_mangle]
pub const extern "C" fn rs_syn_line_needs_processing(state: c_int) -> bool {
    state == line_state::INIT || state == line_state::ACTIVE
}

/// Check if line processing is complete.
#[no_mangle]
pub const extern "C" fn rs_syn_line_is_done(state: c_int) -> bool {
    state == line_state::DONE || state == line_state::ERROR
}

// =============================================================================
// Match Position Helpers
// =============================================================================

/// Match position result
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SynMatchPos {
    /// Column where match starts (0-based)
    pub start_col: c_int,
    /// Column where match ends (0-based, exclusive)
    pub end_col: c_int,
    /// Match flags
    pub flags: c_int,
}

/// Create an empty match position.
#[no_mangle]
pub const extern "C" fn rs_syn_match_pos_empty() -> SynMatchPos {
    SynMatchPos {
        start_col: 0,
        end_col: 0,
        flags: 0,
    }
}

/// Create a match position with given bounds.
#[no_mangle]
pub const extern "C" fn rs_syn_match_pos_new(
    start_col: c_int,
    end_col: c_int,
    flags: c_int,
) -> SynMatchPos {
    SynMatchPos {
        start_col,
        end_col,
        flags,
    }
}

/// Check if match position is valid.
#[no_mangle]
pub const extern "C" fn rs_syn_match_pos_is_valid(pos: SynMatchPos) -> bool {
    pos.start_col >= 0 && pos.end_col >= pos.start_col
}

/// Calculate match length.
#[no_mangle]
pub const extern "C" fn rs_syn_match_pos_len(pos: SynMatchPos) -> c_int {
    if pos.end_col >= pos.start_col {
        pos.end_col - pos.start_col
    } else {
        0
    }
}

// =============================================================================
// State Stack Depth Helpers
// =============================================================================

/// Maximum syntax state stack depth
pub const MAX_SYN_STACK_DEPTH: c_int = 100;

/// Minimum state stack depth before warning
pub const MIN_SYN_STACK_THRESHOLD: c_int = 10;

/// Check if syntax stack depth is safe.
#[no_mangle]
pub const extern "C" fn rs_syn_stack_depth_ok(depth: c_int) -> bool {
    depth >= 0 && depth < MAX_SYN_STACK_DEPTH
}

/// Check if syntax stack is near limit.
#[no_mangle]
pub const extern "C" fn rs_syn_stack_near_limit(depth: c_int) -> bool {
    depth >= MAX_SYN_STACK_DEPTH - MIN_SYN_STACK_THRESHOLD
}

/// Calculate remaining stack capacity.
#[no_mangle]
pub const extern "C" fn rs_syn_stack_remaining(depth: c_int) -> c_int {
    if depth < 0 {
        MAX_SYN_STACK_DEPTH
    } else if depth >= MAX_SYN_STACK_DEPTH {
        0
    } else {
        MAX_SYN_STACK_DEPTH - depth
    }
}

// =============================================================================
// Highlight ID Helpers
// =============================================================================

/// Check if a highlight ID is valid for syntax use.
#[no_mangle]
pub const extern "C" fn rs_syn_hl_id_is_valid(id: c_int) -> bool {
    id >= 0 && id < MAX_HL_ID
}

/// Check if a highlight ID is in the cluster range.
///
/// Uses the existing is_cluster_id function which correctly checks SYNID_CLUSTER range.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub const extern "C" fn rs_syn_id_is_cluster_range(id: c_int) -> bool {
    is_cluster_id(id as i16)
}

/// Convert syntax ID to highlight group index.
#[no_mangle]
pub const extern "C" fn rs_syn_id_to_hl_idx(id: c_int) -> c_int {
    if id >= 0 && id < MAX_HL_ID {
        id
    } else {
        0
    }
}

// =============================================================================
// Concealment Helpers
// =============================================================================

/// Concealment levels
pub mod conceal_level {
    use std::os::raw::c_int;

    /// No concealment
    pub const NONE: c_int = 0;
    /// Conceal with replacement character
    pub const REPLACE: c_int = 1;
    /// Conceal with space
    pub const SPACE: c_int = 2;
    /// Full concealment (nothing shown)
    pub const FULL: c_int = 3;
}

/// Check if concealment is active.
#[no_mangle]
pub const extern "C" fn rs_syn_conceal_is_active(level: c_int) -> bool {
    level > conceal_level::NONE
}

/// Check if concealment completely hides text.
#[no_mangle]
pub const extern "C" fn rs_syn_conceal_is_hidden(level: c_int) -> bool {
    level >= conceal_level::FULL
}

/// Get effective concealment character.
#[no_mangle]
pub const extern "C" fn rs_syn_conceal_char(level: c_int, cchar: c_int) -> c_int {
    match level {
        x if x <= conceal_level::NONE => 0,
        x if x == conceal_level::REPLACE && cchar != 0 => cchar,
        x if x == conceal_level::SPACE => b' ' as c_int,
        _ => 0,
    }
}

// =============================================================================
// Pattern Timeout Helpers
// =============================================================================

/// Default syntax timeout in milliseconds
pub const SYN_TIMEOUT_DEFAULT: c_int = 3000;

/// Minimum syntax timeout in milliseconds
pub const SYN_TIMEOUT_MIN: c_int = 100;

/// Maximum syntax timeout in milliseconds
pub const SYN_TIMEOUT_MAX: c_int = 60000;

/// Clamp timeout value to valid range.
#[no_mangle]
pub const extern "C" fn rs_syn_timeout_clamp(timeout: c_int) -> c_int {
    if timeout < SYN_TIMEOUT_MIN {
        SYN_TIMEOUT_MIN
    } else if timeout > SYN_TIMEOUT_MAX {
        SYN_TIMEOUT_MAX
    } else {
        timeout
    }
}

/// Check if timeout value is valid.
#[no_mangle]
pub const extern "C" fn rs_syn_timeout_is_valid(timeout: c_int) -> bool {
    timeout >= SYN_TIMEOUT_MIN && timeout <= SYN_TIMEOUT_MAX
}

// =============================================================================
// Phase 144: Pattern Matching Integration
// =============================================================================

// Note: The extern declarations for these functions are already present
// earlier in the file. We just add new Rust exports below.

/// Check state ends - clean up finished states from the stack.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_check_state_ends() {
    nvim_syn_check_state_ends();
}

/// Push next match onto the state stack.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_push_next_match() -> StateItemHandle {
    nvim_syn_push_next_match()
}

/// Find end position for a syntax pattern.
///
/// # Safety
/// This function accesses C global state. All output pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_find_endpos(
    idx: c_int,
    start_lnum: c_int,
    start_col: c_int,
    m_end_lnum: *mut c_int,
    m_end_col: *mut c_int,
    hl_end_lnum: *mut c_int,
    hl_end_col: *mut c_int,
    flagsp: *mut c_int,
    end_end_lnum: *mut c_int,
    end_end_col: *mut c_int,
    end_idx: *mut c_int,
    start_ext: ExtMatchHandle,
) {
    nvim_syn_find_endpos(
        idx,
        start_lnum,
        start_col,
        m_end_lnum,
        m_end_col,
        hl_end_lnum,
        hl_end_col,
        flagsp,
        end_end_lnum,
        end_end_col,
        end_idx,
        start_ext,
    );
}

/// Update state item end position.
///
/// # Safety
/// The state item handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_update_si_end(sip: StateItemHandle, startcol: c_int, force: c_int) {
    if !sip.is_null() {
        nvim_syn_update_si_end(sip, startcol, force);
    }
}

/// Get pattern flags for a syntax pattern.
///
/// # Safety
/// The index must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_pattern_flags(idx: c_int) -> c_int {
    nvim_syn_get_pattern_flags(idx)
}

/// Get pattern conceal character.
///
/// # Safety
/// The index must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_pattern_cchar(idx: c_int) -> c_int {
    nvim_syn_get_pattern_cchar(idx)
}

/// Get pattern type (SPTYPE_MATCH, SPTYPE_START, SPTYPE_END, SPTYPE_SKIP).
///
/// # Safety
/// The index must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_pattern_type(idx: c_int) -> c_int {
    nvim_syn_get_pattern_type(idx)
}

/// Get syntax match ID from pattern.
///
/// # Safety
/// The index must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_pattern_syn_match_id(idx: c_int) -> c_int {
    nvim_syn_get_pattern_syn_match_id(idx)
}

/// Check if ID list represents "ALL".
///
/// # Safety
/// The list handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_is_id_list_all(list: IdListHandle) -> c_int {
    nvim_syn_is_id_list_all(list)
}

// Note: rs_syn_get_next_match_attr is defined in highlight.rs.

/// Get highlight start position for next match.
///
/// # Safety
/// Output pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_next_match_h_startpos(lnum: *mut c_int, col: *mut c_int) {
    nvim_syn_get_next_match_h_startpos(lnum, col);
}

/// Get match end position for next match.
///
/// # Safety
/// Output pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_next_match_m_endpos(lnum: *mut c_int, col: *mut c_int) {
    nvim_syn_get_next_match_m_endpos(lnum, col);
}

/// Get highlight end position for next match.
///
/// # Safety
/// Output pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_next_match_h_endpos(lnum: *mut c_int, col: *mut c_int) {
    nvim_syn_get_next_match_h_endpos(lnum, col);
}

/// Get end of start position for next match.
///
/// # Safety
/// Output pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_next_match_eos_pos(lnum: *mut c_int, col: *mut c_int) {
    nvim_syn_get_next_match_eos_pos(lnum, col);
}

/// Get end of end position for next match.
///
/// # Safety
/// Output pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_next_match_eoe_pos(lnum: *mut c_int, col: *mut c_int) {
    nvim_syn_get_next_match_eoe_pos(lnum, col);
}

/// Get flags for next match.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_next_match_flags() -> c_int {
    nvim_syn_get_next_match_flags()
}

/// Get end index for next match.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_next_match_end_idx() -> c_int {
    nvim_syn_get_next_match_end_idx()
}

/// Get next sequence number.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_next_seqnr() -> c_int {
    nvim_syn_get_next_seqnr()
}

/// Set next sequence number.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_set_next_seqnr(seqnr: c_int) {
    nvim_syn_set_next_seqnr(seqnr);
}

/// Increment and return next sequence number.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_incr_next_seqnr() -> c_int {
    nvim_syn_incr_next_seqnr()
}

/// Unreference an extmatch structure.
///
/// # Safety
/// The handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_unref_extmatch_p144(em: ExtMatchHandle) {
    if !em.is_null() {
        nvim_syn_unref_extmatch(em);
    }
}

/// Update syntax item attribute.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_call_update_si_attr(idx: c_int) {
    nvim_syn_call_update_si_attr(idx);
}

/// Check keepend flag propagation.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_check_keepend() {
    nvim_syn_check_keepend();
}

/// Pop current state from stack.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_pop_current_state() {
    nvim_syn_pop_current_state();
}

/// Push state onto current state stack.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_push_current_state(idx: c_int) {
    nvim_syn_push_current_state(idx);
}

/// Get HL_ONELINE flag.
///
/// # Safety
/// This function accesses C global state.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_hl_oneline() -> c_int {
    nvim_syn_get_hl_oneline()
}

/// Get HL_KEEPEND flag.
///
/// # Safety
/// This function accesses C global state.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_hl_keepend() -> c_int {
    nvim_syn_get_hl_keepend()
}

/// Get HL_MATCH flag.
///
/// # Safety
/// This function accesses C global state.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_hl_match() -> c_int {
    nvim_syn_get_hl_match()
}

/// Get HL_CONCEAL flag.
///
/// # Safety
/// This function accesses C global state.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_hl_conceal() -> c_int {
    nvim_syn_get_hl_conceal()
}

/// Get HL_CONCEALENDS flag.
///
/// # Safety
/// This function accesses C global state.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_hl_concealends() -> c_int {
    nvim_syn_get_hl_concealends()
}

/// Get HL_MATCHCONT flag.
///
/// # Safety
/// This function accesses C global state.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_hl_matchcont() -> c_int {
    nvim_syn_get_hl_matchcont()
}

/// Get HL_EXTEND flag.
///
/// # Safety
/// This function accesses C global state.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_hl_extend() -> c_int {
    nvim_syn_get_hl_extend()
}

/// Get SF_CCOMMENT sync flag.
///
/// # Safety
/// This function accesses C global state.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_sf_ccomment() -> c_int {
    nvim_syn_get_sf_ccomment()
}

/// Get SF_MATCH sync flag.
///
/// # Safety
/// This function accesses C global state.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_sf_match() -> c_int {
    nvim_syn_get_sf_match()
}

/// Get HL_SYNC_HERE flag.
///
/// # Safety
/// This function accesses C global state.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_hl_sync_here() -> c_int {
    nvim_syn_get_hl_sync_here()
}

/// Get HL_SYNC_THERE flag.
///
/// # Safety
/// This function accesses C global state.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_hl_sync_there() -> c_int {
    nvim_syn_get_hl_sync_there()
}

/// Get SPTYPE_MATCH constant.
///
/// # Safety
/// This function accesses C global state.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_sptype_match() -> c_int {
    nvim_syn_get_sptype_match()
}

/// Get SPTYPE_START constant.
///
/// # Safety
/// This function accesses C global state.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_sptype_start() -> c_int {
    nvim_syn_get_sptype_start()
}

// Note: rs_syn_extmatch_equal, rs_syn_extmatch_strings_equal are already
// defined earlier in the file using ExtMatchHandle.

// =============================================================================
// Phase 145: Syntax Groups & Clusters
// =============================================================================

// Note: The extern declarations for these functions are already present
// earlier in the file. We just add new Rust exports below.
extern "C" {
    fn nvim_syn_cluster_name2id(name: *const c_char) -> c_int;
    fn nvim_synblock_is_spell_cluster(block: SynBlockHandle, id: c_int) -> c_int;
    fn nvim_synblock_is_nospell_cluster(block: SynBlockHandle, id: c_int) -> c_int;
    // nvim_synblock_get_spell_cluster_id and nvim_synblock_get_nospell_cluster_id
    // are already declared earlier in the file.
}

/// Get cluster ID by name.
///
/// # Safety
/// The name must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cluster_name2id(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }
    nvim_syn_cluster_name2id(name)
}

/// Check if cluster ID is the spell cluster.
///
/// # Safety
/// The block handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_is_spell_cluster(block: SynBlockHandle, id: c_int) -> c_int {
    if block.is_null() {
        return 0;
    }
    nvim_synblock_is_spell_cluster(block, id)
}

/// Check if cluster ID is the nospell cluster.
///
/// # Safety
/// The block handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_is_nospell_cluster(block: SynBlockHandle, id: c_int) -> c_int {
    if block.is_null() {
        return 0;
    }
    nvim_synblock_is_nospell_cluster(block, id)
}

/// Get spell cluster ID from block.
///
/// # Safety
/// The block handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_get_spell_cluster_id(block: SynBlockHandle) -> c_int {
    if block.is_null() {
        return 0;
    }
    nvim_synblock_get_spell_cluster_id(block)
}

/// Get nospell cluster ID from block.
///
/// # Safety
/// The block handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_get_nospell_cluster_id(block: SynBlockHandle) -> c_int {
    if block.is_null() {
        return 0;
    }
    nvim_synblock_get_nospell_cluster_id(block)
}

/// Get cluster name.
///
/// # Safety
/// The cluster handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syncluster_get_name(cluster: SynClusterHandle) -> *const c_char {
    if cluster.is_null() {
        return std::ptr::null();
    }
    nvim_syncluster_get_name(cluster)
}

/// Get cluster name (uppercase).
///
/// # Safety
/// The cluster handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syncluster_get_name_u(cluster: SynClusterHandle) -> *const c_char {
    if cluster.is_null() {
        return std::ptr::null();
    }
    nvim_syncluster_get_name_u(cluster)
}

/// Get highlight group from pattern.
///
/// # Safety
/// The pattern handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synpat_get_hl_group(pat: SynPatHandle) -> c_int {
    if pat.is_null() {
        return 0;
    }
    nvim_synpat_get_hl_group(pat)
}

/// Get cluster ID at index from synblock.
///
/// # Safety
/// The block handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_get_cluster_id(block: SynBlockHandle, idx: c_int) -> c_int {
    if block.is_null() {
        return 0;
    }
    nvim_synblock_get_cluster_id(block, idx)
}

/// Get cluster ID from cluster structure.
///
/// # Safety
/// The cluster handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syncluster_get_id(cluster: SynClusterHandle) -> c_int {
    if cluster.is_null() {
        return 0;
    }
    nvim_syncluster_get_id(cluster)
}

/// Get cluster list pointer.
///
/// # Safety
/// The cluster handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syncluster_get_list(cluster: SynClusterHandle) -> IdListHandle {
    if cluster.is_null() {
        return IdListHandle::null();
    }
    nvim_syncluster_get_list(cluster)
}

// =============================================================================
// Phase 146: State Caching
// =============================================================================

// Note: Extern declarations for state caching functions are already declared
// earlier in the file (around lines 586-659 and 1058-1061).

/// Remove an entry from the state stack.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_stack_remove_entry(sp: SynStateHandle) {
    if !sp.is_null() {
        nvim_syn_stack_remove_entry(sp);
    }
}

/// Allocate a new entry in the state stack.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_stack_alloc_entry(
    lnum: c_int,
    after: SynStateHandle,
) -> SynStateHandle {
    nvim_syn_stack_alloc_entry(lnum, after)
}

/// Store current state to a stack entry.
///
/// # Safety
/// The state handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_store_state_to_entry(sp: SynStateHandle) {
    if !sp.is_null() {
        nvim_syn_store_state_to_entry(sp);
    }
}

/// Set current state item at index.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_set_cur_state_item(
    idx: c_int,
    si_idx: c_int,
    si_flags: c_int,
    si_seqnr: c_int,
    si_cchar: c_int,
    extmatch: ExtMatchHandle,
) {
    nvim_syn_set_cur_state_item(idx, si_idx, si_flags, si_seqnr, si_cchar, extmatch);
}

/// Check if a state item spans past a line.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_state_item_spans_line(idx: c_int, lnum: c_int) -> c_int {
    nvim_syn_state_item_spans_line(idx, lnum)
}

/// Get sst_check_lnum from synblock.
///
/// # Safety
/// The block handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_get_sst_check_lnum(block: SynBlockHandle) -> c_int {
    if block.is_null() {
        return 0;
    }
    nvim_synblock_get_sst_check_lnum(block)
}

/// Get first state in sst array (using current syn_block).
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_sst_first() -> SynStateHandle {
    nvim_syn_get_sst_first()
}

/// Get sst array pointer.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_sst_array() -> *mut std::ffi::c_void {
    nvim_syn_get_sst_array()
}

/// Find entry in stack by line number.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_stack_find_entry_ptr(lnum: c_int) -> SynStateHandle {
    nvim_syn_stack_find_entry_ptr(lnum)
}

// =============================================================================
// Phase 82 Tests
// =============================================================================

#[cfg(test)]
mod phase82_tests {
    use super::*;

    #[test]
    fn test_state_flags() {
        let flags = 0;
        let flags = rs_syn_state_set_flag(flags, state_flags::VALID);
        assert!(rs_syn_state_has_flag(flags, state_flags::VALID));
        assert!(!rs_syn_state_has_flag(flags, state_flags::MODIFIED));

        let flags = rs_syn_state_set_flag(flags, state_flags::CACHED);
        assert!(rs_syn_state_has_flag(flags, state_flags::VALID));
        assert!(rs_syn_state_has_flag(flags, state_flags::CACHED));

        let flags = rs_syn_state_clear_flag(flags, state_flags::VALID);
        assert!(!rs_syn_state_has_flag(flags, state_flags::VALID));
        assert!(rs_syn_state_has_flag(flags, state_flags::CACHED));
    }

    #[test]
    fn test_sync_method() {
        assert!(rs_syn_sync_is_line_based(sync_method::LINEBREAKS));
        assert!(rs_syn_sync_is_line_based(sync_method::MINLINES));
        assert!(rs_syn_sync_is_line_based(sync_method::MAXLINES));
        assert!(!rs_syn_sync_is_line_based(sync_method::CCOMMENT));
        assert!(!rs_syn_sync_is_line_based(sync_method::MATCH));

        assert!(rs_syn_sync_from_start(sync_method::FROMSTART));
        assert!(!rs_syn_sync_from_start(sync_method::LINEBREAKS));
    }

    #[test]
    fn test_line_state() {
        assert!(rs_syn_line_needs_processing(line_state::INIT));
        assert!(rs_syn_line_needs_processing(line_state::ACTIVE));
        assert!(!rs_syn_line_needs_processing(line_state::DONE));
        assert!(!rs_syn_line_needs_processing(line_state::ERROR));

        assert!(rs_syn_line_is_done(line_state::DONE));
        assert!(rs_syn_line_is_done(line_state::ERROR));
        assert!(!rs_syn_line_is_done(line_state::INIT));
        assert!(!rs_syn_line_is_done(line_state::ACTIVE));
    }

    #[test]
    fn test_match_pos() {
        let empty = rs_syn_match_pos_empty();
        assert_eq!(empty.start_col, 0);
        assert_eq!(empty.end_col, 0);
        assert!(rs_syn_match_pos_is_valid(empty));

        let pos = rs_syn_match_pos_new(5, 10, 0);
        assert!(rs_syn_match_pos_is_valid(pos));
        assert_eq!(rs_syn_match_pos_len(pos), 5);

        let invalid = rs_syn_match_pos_new(10, 5, 0);
        assert!(!rs_syn_match_pos_is_valid(invalid));
        assert_eq!(rs_syn_match_pos_len(invalid), 0);
    }

    #[test]
    fn test_stack_depth() {
        assert!(rs_syn_stack_depth_ok(0));
        assert!(rs_syn_stack_depth_ok(99));
        assert!(!rs_syn_stack_depth_ok(100));
        assert!(!rs_syn_stack_depth_ok(-1));

        assert!(!rs_syn_stack_near_limit(0));
        assert!(!rs_syn_stack_near_limit(89));
        assert!(rs_syn_stack_near_limit(90));
        assert!(rs_syn_stack_near_limit(99));

        assert_eq!(rs_syn_stack_remaining(0), 100);
        assert_eq!(rs_syn_stack_remaining(50), 50);
        assert_eq!(rs_syn_stack_remaining(100), 0);
        assert_eq!(rs_syn_stack_remaining(-1), 100);
    }

    #[test]
    fn test_hl_id_helpers() {
        use crate::types::SYNID_CLUSTER;

        assert!(rs_syn_hl_id_is_valid(0));
        assert!(rs_syn_hl_id_is_valid(MAX_HL_ID - 1));
        assert!(!rs_syn_hl_id_is_valid(MAX_HL_ID));
        assert!(!rs_syn_hl_id_is_valid(-1));

        // Cluster IDs start at SYNID_CLUSTER (23000)
        assert!(!rs_syn_id_is_cluster_range(0));
        assert!(!rs_syn_id_is_cluster_range(MAX_HL_ID - 1));
        assert!(!rs_syn_id_is_cluster_range(MAX_HL_ID)); // Not in cluster range
        assert!(rs_syn_id_is_cluster_range(SYNID_CLUSTER)); // First cluster ID
        assert!(rs_syn_id_is_cluster_range(SYNID_CLUSTER + 100));

        assert_eq!(rs_syn_id_to_hl_idx(5), 5);
        assert_eq!(rs_syn_id_to_hl_idx(MAX_HL_ID), 0); // Out of range
        assert_eq!(rs_syn_id_to_hl_idx(-1), 0); // Negative
    }

    #[test]
    fn test_conceal() {
        assert!(!rs_syn_conceal_is_active(conceal_level::NONE));
        assert!(rs_syn_conceal_is_active(conceal_level::REPLACE));
        assert!(rs_syn_conceal_is_active(conceal_level::SPACE));
        assert!(rs_syn_conceal_is_active(conceal_level::FULL));

        assert!(!rs_syn_conceal_is_hidden(conceal_level::NONE));
        assert!(!rs_syn_conceal_is_hidden(conceal_level::REPLACE));
        assert!(!rs_syn_conceal_is_hidden(conceal_level::SPACE));
        assert!(rs_syn_conceal_is_hidden(conceal_level::FULL));

        assert_eq!(rs_syn_conceal_char(conceal_level::NONE, b'x' as c_int), 0);
        assert_eq!(
            rs_syn_conceal_char(conceal_level::REPLACE, b'x' as c_int),
            b'x' as c_int
        );
        assert_eq!(rs_syn_conceal_char(conceal_level::REPLACE, 0), 0);
        assert_eq!(
            rs_syn_conceal_char(conceal_level::SPACE, b'x' as c_int),
            b' ' as c_int
        );
        assert_eq!(rs_syn_conceal_char(conceal_level::FULL, b'x' as c_int), 0);
    }

    #[test]
    fn test_timeout() {
        assert_eq!(rs_syn_timeout_clamp(50), SYN_TIMEOUT_MIN);
        assert_eq!(rs_syn_timeout_clamp(3000), 3000);
        assert_eq!(rs_syn_timeout_clamp(100000), SYN_TIMEOUT_MAX);

        assert!(!rs_syn_timeout_is_valid(50));
        assert!(rs_syn_timeout_is_valid(SYN_TIMEOUT_MIN));
        assert!(rs_syn_timeout_is_valid(SYN_TIMEOUT_DEFAULT));
        assert!(rs_syn_timeout_is_valid(SYN_TIMEOUT_MAX));
        assert!(!rs_syn_timeout_is_valid(100000));
    }
}
