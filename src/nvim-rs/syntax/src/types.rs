//! Core types and constants for the syntax highlighting subsystem.
//!
//! This module defines:
//! - Opaque handle types for C interop
//! - Constants (HL_* flags, SYNID_* types, SPTYPE_*, etc.)
//! - Syntax ID classification helpers

use std::ffi::c_int;

// =============================================================================
// Opaque handle types for C interop
// =============================================================================

/// Opaque handle to a synblock_T (syntax block attached to buffer/window)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SynBlockHandle(pub(crate) *mut std::ffi::c_void);

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
pub struct SynStateHandle(pub(crate) *mut std::ffi::c_void);

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
pub struct SynPatHandle(pub(crate) *mut std::ffi::c_void);

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
pub struct SynClusterHandle(pub(crate) *mut std::ffi::c_void);

impl SynClusterHandle {
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

/// Opaque handle to a stateitem_T (current state stack item)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct StateItemHandle(pub(crate) *mut std::ffi::c_void);

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
pub struct KeyEntryHandle(pub(crate) *mut std::ffi::c_void);

impl KeyEntryHandle {
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

/// Opaque handle to a regprog_T (compiled regex program)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct RegProgHandle(pub(crate) *mut std::ffi::c_void);

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
#[derive(Clone, Copy, Debug)]
pub struct IdListHandle(pub(crate) *mut i16);

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
pub struct BufStateHandle(pub(crate) *mut std::ffi::c_void);

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
pub struct ExtMatchHandle(pub(crate) *mut std::ffi::c_void);

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

/// Opaque handle to a win_T (window)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct WinHandle(pub(crate) *mut std::ffi::c_void);

impl WinHandle {
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

/// Opaque handle to a buf_T (buffer)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct BufHandle(pub(crate) *mut std::ffi::c_void);

impl BufHandle {
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
/// Number of regexp sub-expressions (matches NSUBEXP in regexp_defs.h)
pub const NSUBEXP: c_int = 10;

// =============================================================================
// Constants - Syntax pattern offset types (SPO_*)
// =============================================================================

/// Match start offset
pub const SPO_MS_OFF: c_int = 0;
/// Match end offset
pub const SPO_ME_OFF: c_int = 1;
/// Highlight start offset
pub const SPO_HS_OFF: c_int = 2;
/// Highlight end offset
pub const SPO_HE_OFF: c_int = 3;
/// Region start offset
pub const SPO_RS_OFF: c_int = 4;
/// Region end offset
pub const SPO_RE_OFF: c_int = 5;
/// Leading context offset
pub const SPO_LC_OFF: c_int = 6;
/// Number of offset types
pub const SPO_COUNT: c_int = 7;

// =============================================================================
// Constants - Item argument types (ITEM_*)
// =============================================================================

/// Start argument for syn_cmd_match/region
pub const ITEM_START: c_int = 0;
/// Skip argument for syn_cmd_region
pub const ITEM_SKIP: c_int = 1;
/// End argument for syn_cmd_region
pub const ITEM_END: c_int = 2;
/// Matchgroup argument
pub const ITEM_MATCHGROUP: c_int = 3;

// =============================================================================
// Constants - Special ID list sentinel
// =============================================================================

/// ID list that means "all but contained groups"
/// In C this is `((int16_t *)-1)`, we use a sentinel value
pub const ID_LIST_ALL_SENTINEL: isize = -1;

// =============================================================================
// Syntax ID helper types and functions
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synid_type_classification() {
        // Normal syntax group IDs
        assert_eq!(synid_type(1), SynIdType::Normal);
        assert_eq!(synid_type(100), SynIdType::Normal);
        assert_eq!(synid_type(19999), SynIdType::Normal);

        // ALLBUT indicator
        assert_eq!(synid_type(20000), SynIdType::AllBut);
        assert_eq!(synid_type(20500), SynIdType::AllBut);
        assert_eq!(synid_type(20999), SynIdType::AllBut);

        // TOP indicator
        assert_eq!(synid_type(21000), SynIdType::Top);
        assert_eq!(synid_type(21500), SynIdType::Top);
        assert_eq!(synid_type(21999), SynIdType::Top);

        // CONTAINED indicator
        assert_eq!(synid_type(22000), SynIdType::Contained);
        assert_eq!(synid_type(22500), SynIdType::Contained);
        assert_eq!(synid_type(22999), SynIdType::Contained);

        // Cluster reference
        assert_eq!(synid_type(23000), SynIdType::Cluster);
        assert_eq!(synid_type(25000), SynIdType::Cluster);
        assert_eq!(synid_type(32767), SynIdType::Cluster);
    }

    #[test]
    fn test_cluster_id_helpers() {
        // is_cluster_id
        assert!(!is_cluster_id(1));
        assert!(!is_cluster_id(22999));
        assert!(is_cluster_id(23000));
        assert!(is_cluster_id(25000));

        // cluster_index
        assert_eq!(cluster_index(1), None);
        assert_eq!(cluster_index(22999), None);
        assert_eq!(cluster_index(23000), Some(0));
        assert_eq!(cluster_index(23100), Some(100));

        // make_cluster_id
        assert_eq!(make_cluster_id(0), 23000);
        assert_eq!(make_cluster_id(100), 23100);
    }

    #[test]
    fn test_special_id_helpers() {
        // is_special_id
        assert!(!is_special_id(1));
        assert!(!is_special_id(19999));
        assert!(is_special_id(20000)); // ALLBUT
        assert!(is_special_id(21000)); // TOP
        assert!(is_special_id(22000)); // CONTAINED
        assert!(is_special_id(23000)); // Cluster

        // is_normal_id
        assert!(is_normal_id(1));
        assert!(is_normal_id(19999));
        assert!(!is_normal_id(0));
        assert!(!is_normal_id(20000));
    }

    #[test]
    fn test_extract_inc_tag() {
        // Normal IDs return None
        assert_eq!(extract_inc_tag(1), None);
        assert_eq!(extract_inc_tag(19999), None);

        // ALLBUT range
        assert_eq!(extract_inc_tag(20000), Some(0));
        assert_eq!(extract_inc_tag(20500), Some(500));

        // TOP range
        assert_eq!(extract_inc_tag(21000), Some(0));
        assert_eq!(extract_inc_tag(21500), Some(500));

        // CONTAINED range
        assert_eq!(extract_inc_tag(22000), Some(0));
        assert_eq!(extract_inc_tag(22500), Some(500));

        // Cluster IDs return None
        assert_eq!(extract_inc_tag(23000), None);
    }

    #[test]
    fn test_handle_null_checks() {
        assert!(SynBlockHandle(std::ptr::null_mut()).is_null());
        assert!(SynStateHandle::null().is_null());
        assert!(RegProgHandle::null().is_null());
        assert!(IdListHandle::null().is_null());
        assert!(BufStateHandle::null().is_null());
        assert!(ExtMatchHandle::null().is_null());
        assert!(WinHandle::null().is_null());
        assert!(BufHandle::null().is_null());
    }

    #[test]
    fn test_spo_offset_constants() {
        // Verify offset constants are sequential
        assert_eq!(SPO_MS_OFF, 0);
        assert_eq!(SPO_ME_OFF, 1);
        assert_eq!(SPO_HS_OFF, 2);
        assert_eq!(SPO_HE_OFF, 3);
        assert_eq!(SPO_RS_OFF, 4);
        assert_eq!(SPO_RE_OFF, 5);
        assert_eq!(SPO_LC_OFF, 6);
        assert_eq!(SPO_COUNT, 7);

        // Verify all offsets are less than count (use runtime to avoid clippy warning)
        let count = SPO_COUNT;
        assert!(SPO_MS_OFF < count);
        assert!(SPO_ME_OFF < count);
        assert!(SPO_HS_OFF < count);
        assert!(SPO_HE_OFF < count);
        assert!(SPO_RS_OFF < count);
        assert!(SPO_RE_OFF < count);
        assert!(SPO_LC_OFF < count);
    }

    #[test]
    fn test_item_argument_constants() {
        // Verify item argument constants
        assert_eq!(ITEM_START, 0);
        assert_eq!(ITEM_SKIP, 1);
        assert_eq!(ITEM_END, 2);
        assert_eq!(ITEM_MATCHGROUP, 3);
    }

    #[test]
    fn test_opaque_handle_sizes() {
        // Verify all opaque handles are pointer-sized for FFI compatibility
        use std::mem::size_of;

        assert_eq!(size_of::<SynBlockHandle>(), size_of::<*mut ()>());
        assert_eq!(size_of::<SynStateHandle>(), size_of::<*mut ()>());
        assert_eq!(size_of::<SynPatHandle>(), size_of::<*mut ()>());
        assert_eq!(size_of::<SynClusterHandle>(), size_of::<*mut ()>());
        assert_eq!(size_of::<StateItemHandle>(), size_of::<*mut ()>());
        assert_eq!(size_of::<KeyEntryHandle>(), size_of::<*mut ()>());
        assert_eq!(size_of::<RegProgHandle>(), size_of::<*mut ()>());
        assert_eq!(size_of::<IdListHandle>(), size_of::<*mut i16>());
        assert_eq!(size_of::<BufStateHandle>(), size_of::<*mut ()>());
        assert_eq!(size_of::<ExtMatchHandle>(), size_of::<*mut ()>());
        assert_eq!(size_of::<WinHandle>(), size_of::<*mut ()>());
        assert_eq!(size_of::<BufHandle>(), size_of::<*mut ()>());
    }

    #[test]
    fn test_opaque_handle_alignment() {
        // Verify all opaque handles have pointer alignment for FFI compatibility
        use std::mem::align_of;

        assert_eq!(align_of::<SynBlockHandle>(), align_of::<*mut ()>());
        assert_eq!(align_of::<SynStateHandle>(), align_of::<*mut ()>());
        assert_eq!(align_of::<SynPatHandle>(), align_of::<*mut ()>());
        assert_eq!(align_of::<SynClusterHandle>(), align_of::<*mut ()>());
        assert_eq!(align_of::<StateItemHandle>(), align_of::<*mut ()>());
        assert_eq!(align_of::<KeyEntryHandle>(), align_of::<*mut ()>());
        assert_eq!(align_of::<RegProgHandle>(), align_of::<*mut ()>());
        assert_eq!(align_of::<IdListHandle>(), align_of::<*mut i16>());
        assert_eq!(align_of::<BufStateHandle>(), align_of::<*mut ()>());
        assert_eq!(align_of::<ExtMatchHandle>(), align_of::<*mut ()>());
        assert_eq!(align_of::<WinHandle>(), align_of::<*mut ()>());
        assert_eq!(align_of::<BufHandle>(), align_of::<*mut ()>());
    }

    #[test]
    fn test_hl_flags_are_distinct_bits() {
        // Verify all HL_* flags use distinct bit positions (no overlap)
        let flags = [
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

        // Check that each flag is a power of 2 (single bit set)
        for flag in flags.iter() {
            assert_eq!(flag.count_ones(), 1, "Flag {:#x} is not a single bit", flag);
        }

        // Check no two flags share the same bit
        let mut combined = 0;
        for flag in flags.iter() {
            assert_eq!(
                combined & flag,
                0,
                "Flag {:#x} overlaps with previous flags",
                flag
            );
            combined |= flag;
        }
    }

    #[test]
    fn test_synid_ranges_dont_overlap() {
        // Verify the SYNID_* ranges don't overlap
        // Use runtime variables to avoid clippy's assertions_on_constants
        let max_hl_id = MAX_HL_ID;
        let synid_allbut = SYNID_ALLBUT;
        let synid_top = SYNID_TOP;
        let synid_contained = SYNID_CONTAINED;
        let synid_cluster = SYNID_CLUSTER;

        assert!(max_hl_id > 0);
        assert_eq!(synid_allbut, max_hl_id);
        assert!(synid_top > synid_allbut);
        assert!(synid_contained > synid_top);
        assert!(synid_cluster > synid_contained);

        // Verify each range is at least 1000 wide
        assert!(synid_top - synid_allbut >= 1000);
        assert!(synid_contained - synid_top >= 1000);
        assert!(synid_cluster - synid_contained >= 1000);
    }

    #[test]
    fn test_sptype_constants_match_c() {
        // Verify SPTYPE_* constants match C definitions
        assert_eq!(SPTYPE_MATCH, 1);
        assert_eq!(SPTYPE_START, 2);
        assert_eq!(SPTYPE_END, 3);
        assert_eq!(SPTYPE_SKIP, 4);
    }

    #[test]
    fn test_id_list_all_sentinel() {
        // Verify the sentinel value for ID_LIST_ALL
        assert_eq!(ID_LIST_ALL_SENTINEL, -1);
    }
}
