//! Highlight and color manipulation functions for Neovim
//!
//! This crate provides color blending and conversion functions used by the
//! highlight system. It also manages the highlight attribute entry table
//! that maps attribute IDs to their properties.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::manual_range_contains)]

use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_void, CStr};
use std::sync::{LazyLock, Mutex};

// Re-export API types for hlattrs2dict, hl_inspect, object_to_color
use nvim_api::{Arena, Array, Dict, Error, NvimString, Object, ObjectType};

/// Properly-sized Arena for stack allocation from Rust.
/// Layout: { char *cur_blk; size_t pos; size_t size; } = 24 bytes on 64-bit.
/// Initialized and freed via C accessors.
#[repr(C)]
struct SizedArena {
    cur_blk: *mut c_char,
    pos: usize,
    size: usize,
}

impl SizedArena {
    /// Create a new empty arena (ARENA_EMPTY equivalent).
    fn new() -> Self {
        let mut arena = SizedArena {
            cur_blk: std::ptr::null_mut(),
            pos: 0,
            size: 0,
        };
        unsafe { nvim_arena_init(arena.as_arena_mut()) };
        arena
    }

    /// Get a mutable Arena pointer suitable for passing to C/Rust FFI functions.
    fn as_arena_mut(&mut self) -> *mut Arena {
        self as *mut SizedArena as *mut Arena
    }

    /// Finish and free the arena memory.
    fn finish_and_free(&mut self) {
        unsafe { nvim_arena_finish_and_free(self.as_arena_mut()) };
    }
}

impl Drop for SizedArena {
    fn drop(&mut self) {
        self.finish_and_free();
    }
}

extern "C" {
    /// Get the terminal color count from C globals
    fn nvim_get_t_colors() -> c_int;
    /// Get the normal foreground color
    fn nvim_get_normal_fg() -> c_int;
    /// Get the normal background color
    fn nvim_get_normal_bg() -> c_int;
    /// Get the normal special color
    fn nvim_get_normal_sp() -> c_int;
    /// Get p_bg (background option: 'd' for dark, 'l' for light)
    fn nvim_get_p_bg() -> c_char;

    // Namespace highlight global accessors
    /// Get global highlight namespace (0 = use default)
    fn nvim_get_ns_hl_global() -> c_int;
    /// Set global highlight namespace
    fn nvim_set_ns_hl_global(ns: c_int);
    /// Get window-specific highlight namespace (-1 = not set)
    fn nvim_get_ns_hl_win() -> c_int;
    /// Set window-specific highlight namespace
    fn nvim_set_ns_hl_win(ns: c_int);
    /// Get fast callback highlight namespace (-1 = not set)
    fn nvim_get_ns_hl_fast() -> c_int;
    /// Set fast callback highlight namespace
    fn nvim_set_ns_hl_fast(ns: c_int);
    /// Get currently active/cached highlight namespace
    fn nvim_get_ns_hl_active() -> c_int;
    /// Set currently active/cached highlight namespace
    fn nvim_set_ns_hl_active(ns: c_int);
    /// Get pointer to currently active highlight attributes
    fn nvim_get_hl_attr_active() -> *const c_int;
    /// Set pointer to active highlight attributes
    fn nvim_set_hl_attr_active(attrs: *const c_int);
    /// Get pointer to default highlight_attr array
    fn nvim_get_highlight_attr() -> *const c_int;

    // DecorProvider accessors
    /// Get hl_valid and set hl_cached=false atomically. Creates provider if needed.
    fn nvim_decor_provider_hl_def_prepare(ns_id: c_int) -> c_int;
    /// Get hl_valid for a namespace. Returns -1 if no provider exists.
    fn nvim_decor_provider_get_hl_valid(ns_id: c_int) -> c_int;
    /// Check if namespace has a hl_def callback defined.
    fn nvim_decor_provider_has_hl_def(ns_id: c_int) -> bool;
    /// Get hl_cached for a namespace. Returns false if provider doesn't exist.
    fn nvim_decor_provider_get_hl_cached(ns_id: c_int) -> bool;
    /// Set hl_cached for a namespace. Creates provider if force=true.
    fn nvim_decor_provider_set_hl_cached(ns_id: c_int, cached: bool, force: bool);

    /// Set need_highlight_changed global
    fn nvim_set_need_highlight_changed(value: bool);

    // hl_table (HlGroup array) accessors from highlight_group.c
    /// Get the number of highlight groups
    fn highlight_num_groups() -> c_int;
    /// Get the name of a highlight group (0-based index)
    fn highlight_group_name(id: c_int) -> *const c_char;
    /// Get the link target ID of a highlight group (0-based index)
    fn highlight_link_id(id: c_int) -> c_int;
    /// Get the attribute ID (sg_attr) of a highlight group (0-based index)
    fn highlight_group_attr(id: c_int) -> c_int;
    /// Check if a highlight group has been cleared (0-based index)
    fn highlight_group_cleared(id: c_int) -> bool;
    /// Get the sg_set flags of a highlight group (0-based index)
    fn highlight_group_set(id: c_int) -> c_int;
    /// Get the parent ID of a highlight group (0-based index, for @nested.groups)
    fn highlight_group_parent(id: c_int) -> c_int;
    /// Lookup a highlight group by uppercase name, returns ID (1-based) or 0 if not found
    fn nvim_highlight_name_lookup(name_u: *const c_char) -> c_int;

    // Accessors for hl_get_ui_attr (Phase 15)
    /// Get 'pumblend' option value
    fn nvim_get_p_pb() -> c_int;
    /// Check if popup menu is drawn
    fn nvim_get_pum_drawn() -> bool;
    /// Set must_redraw_pum flag
    fn nvim_set_must_redraw_pum(value: bool);
    /// Get HLF_PNI enum value
    fn nvim_get_hlf_pni() -> c_int;
    /// Get HLF_PST enum value
    fn nvim_get_hlf_pst() -> c_int;

    // Accessors for win_bg_attr (Phase 16)
    /// Get current window pointer
    fn nvim_get_curwin() -> *mut c_void;
    /// Get w_hl_attr_normal field from window
    fn nvim_win_get_hl_attr_normal(wp: *mut c_void) -> c_int;
    /// Get w_hl_attr_normalnc field from window
    fn nvim_win_get_hl_attr_normalnc(wp: *mut c_void) -> c_int;
    /// Get HLF_NONE enum value
    fn nvim_get_hlf_none() -> c_int;
    /// Get HLF_INACTIVE enum value
    fn nvim_get_hlf_inactive() -> c_int;

    // Accessors for update_window_hl (Phase 17)
    /// Get HLF_NFLOAT enum value
    fn nvim_get_hlf_nfloat() -> c_int;
    /// Get HLF_BORDER enum value
    fn nvim_get_hlf_border() -> c_int;
    /// Get HLF_COUNT enum value
    fn nvim_get_hlf_count() -> c_int;
    /// Get hlf_names[idx] string
    fn nvim_get_hlf_name(idx: c_int) -> *const c_char;
    // nvim_get_highlight_attr is already defined above (line 45)
    // nvim_win_get_ns_hl is defined below in window accessors section
    /// Get w_ns_hl_active field from window
    fn nvim_win_get_ns_hl_active(wp: *mut c_void) -> c_int;
    /// Set w_ns_hl_active field of window
    fn nvim_win_set_ns_hl_active(wp: *mut c_void, val: c_int);
    /// Get w_ns_hl_attr pointer from window
    fn nvim_win_get_ns_hl_attr(wp: *mut c_void) -> *mut c_int;
    /// Set w_ns_hl_attr pointer of window
    fn nvim_win_set_ns_hl_attr(wp: *mut c_void, val: *mut c_int);
    /// Get w_hl_needs_update field from window
    fn nvim_win_get_hl_needs_update(wp: *mut c_void) -> bool;
    /// Set w_hl_needs_update field of window
    fn nvim_win_set_hl_needs_update(wp: *mut c_void, val: bool);
    /// Set w_hl_attr_normal field of window
    fn nvim_win_set_hl_attr_normal(wp: *mut c_void, val: c_int);
    /// Set w_hl_attr_normalnc field of window
    fn nvim_win_set_hl_attr_normalnc(wp: *mut c_void, val: c_int);
    /// Get w_floating field from window
    fn nvim_win_get_floating(wp: *mut c_void) -> c_int;
    /// Get w_config.external field from window
    fn nvim_win_get_config_external(wp: *mut c_void) -> bool;
    /// Get w_config.border field from window
    fn nvim_win_get_config_border(wp: *mut c_void) -> bool;
    /// Get w_config.border_hl_ids[idx] from window
    fn nvim_win_get_config_border_hl_id(wp: *mut c_void, idx: c_int) -> c_int;
    /// Set w_config.border_attr[idx] of window
    fn nvim_win_set_config_border_attr(wp: *mut c_void, idx: c_int, val: c_int);
    /// Set w_config.shadow field of window
    fn nvim_win_set_config_shadow(wp: *mut c_void, val: bool);
    /// Get w_config.shadow field from window
    fn nvim_win_get_config_shadow(wp: *mut c_void) -> bool;
    /// Get w_p_winbl field from window
    fn nvim_win_get_p_winbl(wp: *mut c_void) -> c_int;
    /// Set w_grid_alloc.blending field of window
    fn nvim_win_set_grid_blending(wp: *mut c_void, val: bool);

    // Syntax accessors
    /// Get current_sub_char static variable (conceal substitution character)
    fn nvim_syn_get_current_sub_char() -> c_int;
}

// ============================================================================
// Highlight Attribute Flags (from highlight_defs.h)
// ============================================================================

/// Highlight attribute flags
pub mod hl_attr_flags {
    pub const HL_INVERSE: i16 = 0x01;
    pub const HL_BOLD: i16 = 0x02;
    pub const HL_ITALIC: i16 = 0x04;
    pub const HL_UNDERLINE_MASK: i16 = 0x38;
    pub const HL_UNDERLINE: i16 = 0x08;
    pub const HL_UNDERCURL: i16 = 0x10;
    pub const HL_UNDERDOUBLE: i16 = 0x18;
    pub const HL_UNDERDOTTED: i16 = 0x20;
    pub const HL_UNDERDASHED: i16 = 0x28;
    pub const HL_STANDOUT: i16 = 0x0040;
    pub const HL_STRIKETHROUGH: i16 = 0x0080;
    pub const HL_ALTFONT: i16 = 0x0100;
    pub const HL_NOCOMBINE: i16 = 0x0400;
    pub const HL_BG_INDEXED: i16 = 0x0800;
    pub const HL_FG_INDEXED: i16 = 0x1000;
    pub const HL_DEFAULT: i16 = 0x2000;
    pub const HL_GLOBAL: i16 = 0x4000;
}

use hl_attr_flags::*;

// ============================================================================
// HlAttrs Struct (matches C struct in highlight_defs.h)
// ============================================================================

/// Stores a complete highlighting entry, including colors and attributes
/// for both TUI and GUI. This must match the C struct exactly.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct HlAttrs {
    pub rgb_ae_attr: i16,
    pub cterm_ae_attr: i16,
    pub rgb_fg_color: i32,
    pub rgb_bg_color: i32,
    pub rgb_sp_color: i32,
    pub cterm_fg_color: i16,
    pub cterm_bg_color: i16,
    pub hl_blend: i32,
    pub url: i32,
}

impl HlAttrs {
    /// Create a new HlAttrs with default values (matching HLATTRS_INIT)
    pub const fn new() -> Self {
        HlAttrs {
            rgb_ae_attr: 0,
            cterm_ae_attr: 0,
            rgb_fg_color: -1,
            rgb_bg_color: -1,
            rgb_sp_color: -1,
            cterm_fg_color: 0,
            cterm_bg_color: 0,
            hl_blend: -1,
            url: -1,
        }
    }
}

// ============================================================================
// HlKind Enum (matches C enum in highlight_defs.h)
// ============================================================================

/// The kind/source of a highlight entry
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HlKind {
    #[default]
    Unknown = 0,
    UI = 1,
    Syntax = 2,
    Terminal = 3,
    Combine = 4,
    Blend = 5,
    BlendThrough = 6,
    Invalid = 7,
}

// ============================================================================
// HlEntry Struct (matches C struct in highlight_defs.h)
// ============================================================================

/// A complete highlight entry with attributes and semantic information.
/// This must match the C struct exactly for FFI compatibility.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct HlEntry {
    /// The highlight attributes (colors, styles)
    pub attr: HlAttrs,
    /// The kind/source of this highlight
    pub kind: HlKind,
    /// First ID (meaning depends on kind: syntax group ID, UI index, etc.)
    pub id1: c_int,
    /// Second ID (meaning depends on kind: namespace ID, linked group ID, etc.)
    pub id2: c_int,
    /// Window ID (used for window-specific highlights)
    pub winid: c_int,
}

impl HlEntry {
    /// Create a new HlEntry with default values
    pub const fn new() -> Self {
        HlEntry {
            attr: HlAttrs::new(),
            kind: HlKind::Invalid,
            id1: 0,
            id2: 0,
            winid: 0,
        }
    }
}

// ============================================================================
// ColorKey and ColorItem (for namespace highlight storage)
// ============================================================================

/// Key for namespace highlight lookup (matches C ColorKey)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorKey {
    pub ns_id: c_int,
    pub syn_id: c_int,
}

/// Cached highlight item for namespace highlights (matches C ColorItem)
/// Default values match COLOR_ITEM_INITIALIZER
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ColorItem {
    pub attr_id: c_int,
    pub link_id: c_int,
    pub version: c_int,
    pub is_default: bool,
    pub link_global: bool,
}

impl Default for ColorItem {
    fn default() -> Self {
        // Matches COLOR_ITEM_INITIALIZER from C
        ColorItem {
            attr_id: -1,
            link_id: -1,
            version: -1,
            is_default: false,
            link_global: false,
        }
    }
}

// ============================================================================
// NSHlAttr - Per-namespace UI highlight attributes
// ============================================================================

/// Number of UI highlight groups (HLF_COUNT from highlight_defs.h)
/// This must match the C definition.
pub const HLF_COUNT: usize = 76;

/// Per-namespace UI highlight attribute array
pub type NSHlAttr = [c_int; HLF_COUNT];

// For HashMap key usage - entries are considered equal if all fields match
impl PartialEq for HlEntry {
    fn eq(&self, other: &Self) -> bool {
        // Compare attr fields
        self.attr.rgb_ae_attr == other.attr.rgb_ae_attr
            && self.attr.cterm_ae_attr == other.attr.cterm_ae_attr
            && self.attr.rgb_fg_color == other.attr.rgb_fg_color
            && self.attr.rgb_bg_color == other.attr.rgb_bg_color
            && self.attr.rgb_sp_color == other.attr.rgb_sp_color
            && self.attr.cterm_fg_color == other.attr.cterm_fg_color
            && self.attr.cterm_bg_color == other.attr.cterm_bg_color
            && self.attr.hl_blend == other.attr.hl_blend
            && self.attr.url == other.attr.url
            // Compare other fields
            && self.kind == other.kind
            && self.id1 == other.id1
            && self.id2 == other.id2
            && self.winid == other.winid
    }
}

impl Eq for HlEntry {}

impl std::hash::Hash for HlEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.attr.rgb_ae_attr.hash(state);
        self.attr.cterm_ae_attr.hash(state);
        self.attr.rgb_fg_color.hash(state);
        self.attr.rgb_bg_color.hash(state);
        self.attr.rgb_sp_color.hash(state);
        self.attr.cterm_fg_color.hash(state);
        self.attr.cterm_bg_color.hash(state);
        self.attr.hl_blend.hash(state);
        self.attr.url.hash(state);
        self.kind.hash(state);
        self.id1.hash(state);
        self.id2.hash(state);
        self.winid.hash(state);
    }
}

// ============================================================================
// Attribute Entry Store - Global State
// ============================================================================

/// Maximum number of attribute entries (from C: MAX_TYPENR)
const MAX_TYPENR: usize = 65535;

/// The attribute entry store manages the global table of highlight entries.
/// Each entry maps an attribute ID to its HlEntry.
struct AttrEntryStore {
    /// Vector of entries indexed by attribute ID
    entries: Vec<HlEntry>,
    /// Reverse lookup: entry -> attribute ID (for deduplication)
    entry_to_id: HashMap<HlEntry, c_int>,
    /// Cache for combined attributes: (char_attr << 16 | prim_attr) -> result_id
    combine_cache: HashMap<c_int, c_int>,
    /// Cache for blended attributes: (back_attr << 16 | front_attr) -> result_id
    blend_cache: HashMap<c_int, c_int>,
    /// Cache for blend-through attributes
    blendthrough_cache: HashMap<c_int, c_int>,
    /// URL storage: index -> URL string (as CString for FFI)
    urls: Vec<std::ffi::CString>,
    /// URL reverse lookup: URL string -> index
    url_to_index: HashMap<String, u32>,
    /// Whether hlstate mode is active
    hlstate_active: bool,
    /// Namespace highlight storage: (ns_id, syn_id) -> ColorItem
    ns_hls: HashMap<ColorKey, ColorItem>,
    /// Per-namespace UI highlight attributes: ns_id -> Box<NSHlAttr>
    /// Using Box to ensure stable pointers when HashMap reallocates
    ns_hl_attr: HashMap<c_int, Box<NSHlAttr>>,
}

impl AttrEntryStore {
    /// Create a new empty store
    fn new() -> Self {
        AttrEntryStore {
            entries: Vec::new(),
            entry_to_id: HashMap::new(),
            combine_cache: HashMap::new(),
            blend_cache: HashMap::new(),
            blendthrough_cache: HashMap::new(),
            urls: Vec::new(),
            url_to_index: HashMap::new(),
            hlstate_active: false,
            ns_hls: HashMap::new(),
            ns_hl_attr: HashMap::new(),
        }
    }

    /// Initialize the store with a dummy entry at index 0
    fn init(&mut self) {
        if self.entries.is_empty() {
            // Index 0 is reserved for "no attribute"
            let dummy = HlEntry {
                attr: HlAttrs::new(),
                kind: HlKind::Invalid,
                id1: 0,
                id2: 0,
                winid: 0,
            };
            self.entries.push(dummy);
            // Don't add to entry_to_id - we want ID 0 to be special
        }
    }

    /// Get the number of entries
    fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if an entry exists and return its ID, or insert and return new ID
    fn get_or_insert(&mut self, mut entry: HlEntry) -> Option<c_int> {
        // If hlstate is not active, clear semantic info to reduce table size
        if !self.hlstate_active {
            entry.kind = HlKind::Unknown;
            entry.id1 = 0;
            entry.id2 = 0;
        }

        // Check if entry already exists
        if let Some(&id) = self.entry_to_id.get(&entry) {
            return Some(id);
        }

        // Check if we're at capacity
        if self.entries.len() >= MAX_TYPENR {
            return None; // Signal that we need to clear and retry
        }

        // Insert new entry
        let id = self.entries.len() as c_int;
        self.entries.push(entry);
        self.entry_to_id.insert(entry, id);
        Some(id)
    }

    /// Get an entry by ID
    fn get(&self, id: c_int) -> Option<&HlEntry> {
        if id >= 0 && (id as usize) < self.entries.len() {
            Some(&self.entries[id as usize])
        } else {
            None
        }
    }

    /// Clear all tables and reinitialize
    fn clear(&mut self) {
        self.entries.clear();
        self.entry_to_id.clear();
        self.combine_cache.clear();
        self.blend_cache.clear();
        self.blendthrough_cache.clear();
        self.urls.clear();
        self.url_to_index.clear();
        // Note: ns_hls is NOT cleared here - only on full destruction
        self.init();
    }

    /// Destroy the namespace highlight storage (only on full cleanup)
    fn destroy_ns_hls(&mut self) {
        self.ns_hls.clear();
        self.ns_hl_attr.clear();
    }

    // ========================================================================
    // Namespace highlight storage operations
    // ========================================================================

    /// Check if a namespace highlight exists
    fn ns_hls_has(&self, ns_id: c_int, syn_id: c_int) -> bool {
        self.ns_hls.contains_key(&ColorKey { ns_id, syn_id })
    }

    /// Get a namespace highlight item (returns default if not found)
    fn ns_hls_get(&self, ns_id: c_int, syn_id: c_int) -> ColorItem {
        self.ns_hls
            .get(&ColorKey { ns_id, syn_id })
            .copied()
            .unwrap_or_default()
    }

    /// Put a namespace highlight item
    fn ns_hls_put(&mut self, ns_id: c_int, syn_id: c_int, item: ColorItem) {
        self.ns_hls.insert(ColorKey { ns_id, syn_id }, item);
    }

    // ========================================================================
    // Per-namespace UI highlight attribute operations (ns_hl_attr)
    // ========================================================================

    /// Get pointer to namespace UI highlight attributes.
    /// Returns null if no entry exists for this namespace.
    fn ns_hl_attr_get(&self, ns_id: c_int) -> *const c_int {
        self.ns_hl_attr
            .get(&ns_id)
            .map(|boxed| boxed.as_ptr())
            .unwrap_or(std::ptr::null())
    }

    /// Get or create pointer to namespace UI highlight attributes.
    /// Creates a zeroed array if no entry exists.
    /// Returns mutable pointer for C to fill in the values.
    fn ns_hl_attr_get_or_create(&mut self, ns_id: c_int) -> *mut c_int {
        self.ns_hl_attr
            .entry(ns_id)
            .or_insert_with(|| Box::new([0; HLF_COUNT]))
            .as_mut_ptr()
    }

    /// Add or lookup a URL. Returns the index.
    fn add_url(&mut self, url: &str) -> u32 {
        if let Some(&index) = self.url_to_index.get(url) {
            return index;
        }
        let index = self.urls.len() as u32;
        // Store as CString for FFI compatibility
        let cstring = std::ffi::CString::new(url).unwrap_or_default();
        self.urls.push(cstring);
        self.url_to_index.insert(url.to_string(), index);
        index
    }

    /// Get a URL C string pointer by index
    fn get_url_ptr(&self, index: u32) -> *const c_char {
        self.urls
            .get(index as usize)
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null())
    }

    /// Clear just the blend caches
    fn clear_blend_caches(&mut self) {
        self.blend_cache.clear();
        self.blendthrough_cache.clear();
    }

    /// Enable hlstate mode
    fn enable_hlstate(&mut self) -> bool {
        if self.hlstate_active {
            return false;
        }
        self.hlstate_active = true;
        true
    }
}

// Global store instance - using LazyLock + Mutex for thread safety
// Note: Neovim is single-threaded, but Mutex is required for safe global state in Rust
static ATTR_STORE: LazyLock<Mutex<AttrEntryStore>> =
    LazyLock::new(|| Mutex::new(AttrEntryStore::new()));

// ============================================================================
// FFI Functions for Attribute Entry Management
// ============================================================================

/// Initialize the highlight attribute table.
/// This should be called once during Neovim startup.
#[export_name = "highlight_init"]
pub extern "C" fn rs_highlight_init() {
    let mut store = ATTR_STORE.lock().unwrap();
    store.init();
}

/// Get the number of attribute entries in the table.
#[no_mangle]
pub extern "C" fn rs_attr_entry_count() -> c_int {
    let store = ATTR_STORE.lock().unwrap();
    store.len() as c_int
}

/// Get highlight attributes for an attribute ID.
/// Returns HLATTRS_INIT if the ID is invalid or the tables were cleared.
#[must_use]
#[export_name = "syn_attr2entry"]
pub extern "C" fn rs_syn_attr2entry(attr: c_int) -> HlAttrs {
    let store = ATTR_STORE.lock().unwrap();
    if let Some(entry) = store.get(attr) {
        entry.attr
    } else {
        HlAttrs::new()
    }
}

/// Get a full HlEntry by attribute ID.
/// Returns a default Invalid entry if the ID is invalid.
#[no_mangle]
pub extern "C" fn rs_get_attr_entry_by_id(attr: c_int) -> HlEntry {
    let store = ATTR_STORE.lock().unwrap();
    if let Some(entry) = store.get(attr) {
        *entry
    } else {
        HlEntry::new()
    }
}

/// Result type for get_attr_entry operations
#[repr(C)]
pub struct GetAttrEntryResult {
    /// The attribute ID (0 = error, >0 = valid ID)
    pub id: c_int,
    /// Whether this is a new entry that needs UI notification
    pub is_new: bool,
}

/// Add or lookup an attribute entry.
/// Returns the attribute ID and whether it's a new entry.
/// If the table is full and the entry is a Combine type, returns id=0.
#[no_mangle]
pub extern "C" fn rs_get_attr_entry(entry: HlEntry) -> GetAttrEntryResult {
    let mut store = ATTR_STORE.lock().unwrap();

    // Check if entry already exists (quick path)
    let mut test_entry = entry;
    if !store.hlstate_active {
        test_entry.kind = HlKind::Unknown;
        test_entry.id1 = 0;
        test_entry.id2 = 0;
    }
    if let Some(&id) = store.entry_to_id.get(&test_entry) {
        return GetAttrEntryResult { id, is_new: false };
    }

    // Try to insert
    match store.get_or_insert(entry) {
        Some(id) => GetAttrEntryResult { id, is_new: true },
        None => {
            // Table is full - signal that C code should clear and retry
            // For Combine entries, we return 0 to indicate failure
            GetAttrEntryResult {
                id: -1, // Signal overflow
                is_new: false,
            }
        }
    }
}

// ============================================================================
// Full get_attr_entry with UI dispatch (Phase 21)
// ============================================================================

extern "C" {
    /// C wrapper for UI dispatch - sends hl_attr_define event to all UIs
    fn nvim_ui_call_hl_attr_define(id: c_int, attrs: HlAttrs, inspect: Array);
    /// C wrapper for emsg - reports table overflow error
    fn nvim_highlight_emsg_overflow();
    /// Arena management
    fn nvim_arena_init(arena: *mut Arena);
    fn nvim_arena_finish_and_free(arena: *mut Arena);
    /// Reinit callbacks - called from rs_clear_hl_tables_full when reinit=true
    fn nvim_memset_highlight_attr_last();
    fn nvim_call_highlight_attr_set_all();
    fn nvim_call_highlight_changed();
    fn nvim_call_screen_invalidate_highlights();
}

/// Thread-local flag to detect recursive get_attr_entry calls
static GET_ATTR_ENTRY_RECURSIVE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Full get_attr_entry implementation with retry logic and UI dispatch.
///
/// Core get_attr_entry implementation with caller-provided arena.
///
/// Returns 0 for error, positive ID for success.
///
/// # Safety
/// - `arena` must be a valid Arena pointer for hl_inspect allocation
unsafe fn get_attr_entry_impl(entry: HlEntry, arena: *mut Arena) -> c_int {
    use std::sync::atomic::Ordering;

    let mut retried = false;

    loop {
        // Try to get or insert the entry
        let result = rs_get_attr_entry(entry);

        if result.id == -1 {
            // Table overflow
            let recursive = GET_ATTR_ENTRY_RECURSIVE.load(Ordering::SeqCst);
            if recursive || retried {
                nvim_highlight_emsg_overflow();
                return 0;
            }

            GET_ATTR_ENTRY_RECURSIVE.store(true, Ordering::SeqCst);
            rs_clear_hl_tables_full(true);
            GET_ATTR_ENTRY_RECURSIVE.store(false, Ordering::SeqCst);

            if entry.kind == HlKind::Combine {
                // Combine entry is now invalid, don't retry
                return 0;
            }
            retried = true;
            continue;
        }

        if !result.is_new {
            // Existing entry - just return the ID
            return result.id;
        }

        // New entry - send UI event
        let id = result.id;
        let inspect = rs_hl_inspect(id, arena);
        nvim_ui_call_hl_attr_define(id, entry.attr, inspect);

        return id;
    }
}

/// Self-contained get_attr_entry: manages its own arena.
/// This replaces both C's get_attr_entry() and rs_get_attr_entry_full().
#[no_mangle]
pub unsafe extern "C" fn rs_get_attr_entry_full(entry: HlEntry) -> c_int {
    let mut arena = SizedArena::new();
    get_attr_entry_impl(entry, arena.as_arena_mut())
    // arena dropped here, freeing memory
}

// ============================================================================
// ui_send_all_hls - Send all highlights to a newly connected UI (Phase 22)
// ============================================================================

extern "C" {
    /// C wrapper for remote_ui_hl_attr_define - send highlight to one UI
    fn nvim_remote_ui_hl_attr_define(ui: *mut c_void, id: c_int, attrs: HlAttrs, inspect: Array);
    /// C wrapper for remote_ui_hl_group_set - send group to one UI
    fn nvim_remote_ui_hl_group_set(ui: *mut c_void, name: *const c_char, id: c_int);
}

/// Send one highlight attribute entry to a UI.
///
/// # Safety
/// - `ui` must be a valid RemoteUI pointer
/// - `arena` must be a valid Arena pointer for hl_inspect allocation
#[no_mangle]
pub unsafe extern "C" fn rs_ui_send_hl_attr(ui: *mut c_void, id: c_int, arena: *mut Arena) {
    let inspect = rs_hl_inspect(id, arena);
    let attrs = rs_syn_attr2entry(id);
    nvim_remote_ui_hl_attr_define(ui, id, attrs, inspect);
}

/// Send one highlight group to a UI.
///
/// # Safety
/// - `ui` must be a valid RemoteUI pointer
#[no_mangle]
pub unsafe extern "C" fn rs_ui_send_hl_group(ui: *mut c_void, hlf: c_int) {
    let name = nvim_get_hlf_name(hlf);
    let highlight_attr = nvim_get_highlight_attr();
    let attr = *highlight_attr.add(hlf as usize);
    nvim_remote_ui_hl_group_set(ui, name, attr);
}

/// Send all highlights to a newly connected UI. Manages arena per iteration.
///
/// # Safety
/// - `ui` must be a valid RemoteUI pointer
#[export_name = "ui_send_all_hls"]
pub unsafe extern "C" fn rs_ui_send_all_hls(ui: *mut c_void) {
    let count = rs_attr_entry_count();
    for i in 1..count {
        let mut arena = SizedArena::new();
        rs_ui_send_hl_attr(ui, i, arena.as_arena_mut());
        // arena dropped and freed here
    }
    let hlf_count = nvim_get_hlf_count();
    for hlf in 0..hlf_count {
        rs_ui_send_hl_group(ui, hlf);
    }
}

/// Clear all highlight tables. If reinit is true, reinitialize after clearing.
/// If reinit is false, also destroys namespace storage.
#[no_mangle]
pub extern "C" fn rs_clear_hl_tables(reinit: bool) {
    let mut store = ATTR_STORE.lock().unwrap();
    if reinit {
        store.clear();
    } else {
        // Full destruction - clear everything including namespace storage
        store.entries.clear();
        store.entry_to_id.clear();
        store.combine_cache.clear();
        store.blend_cache.clear();
        store.blendthrough_cache.clear();
        store.destroy_ns_hls();
    }
}

/// Full clear_hl_tables: clears Rust storage and runs reinit callbacks if needed.
/// Replaces the combined C clear_hl_tables + reinit logic.
///
/// Note: we clear the store first (dropping the lock), then call callbacks.
/// The callbacks (highlight_attr_set_all, etc.) re-enter Rust (hl_get_syn_attr),
/// so the lock must not be held during callback execution.
#[export_name = "clear_hl_tables"]
pub unsafe extern "C" fn rs_clear_hl_tables_full(reinit: bool) {
    // Phase 1: clear Rust storage (acquires and releases ATTR_STORE lock)
    rs_clear_hl_tables(reinit);
    // Phase 2: reinit callbacks (may re-enter Rust, so lock must be free)
    if reinit {
        nvim_memset_highlight_attr_last();
        nvim_call_highlight_attr_set_all();
        nvim_call_highlight_changed();
        nvim_call_screen_invalidate_highlights();
    }
}

/// Full hl_invalidate_blends: clears blend caches and refreshes highlights.
#[export_name = "hl_invalidate_blends"]
pub unsafe extern "C" fn rs_hl_invalidate_blends_full() {
    rs_hl_invalidate_blends();
    // highlight_changed and update_window_hl are called from C
    // to avoid re-entrancy issues with curwin access
    nvim_hl_invalidate_blends_callbacks();
}

extern "C" {
    fn nvim_hl_invalidate_blends_callbacks();
}

/// Full highlight_use_hlstate: enables hlstate and clears tables if first time.
/// Returns true if hl tables were reset.
#[export_name = "highlight_use_hlstate"]
pub unsafe extern "C" fn rs_highlight_use_hlstate_full() -> bool {
    if !rs_highlight_use_hlstate() {
        return false;
    }
    rs_clear_hl_tables_full(true);
    true
}

/// Enable hlstate mode. Returns true if the table was reset (first time enabling).
#[no_mangle]
pub extern "C" fn rs_highlight_use_hlstate() -> bool {
    let mut store = ATTR_STORE.lock().unwrap();
    if store.enable_hlstate() {
        // Tables need to be rebuilt - but we don't clear here,
        // the caller (C code) will handle that
        true
    } else {
        false
    }
}

/// Get hlstate_active flag. Rust is the source of truth.
#[no_mangle]
pub extern "C" fn rs_get_hlstate_active() -> bool {
    let store = ATTR_STORE.lock().unwrap();
    store.hlstate_active
}

// ============================================================================
// FFI Functions for Namespace Highlight Storage (ns_hls)
// ============================================================================

/// Check if a namespace highlight entry exists.
#[no_mangle]
pub extern "C" fn rs_ns_hls_has(ns_id: c_int, syn_id: c_int) -> bool {
    let store = ATTR_STORE.lock().unwrap();
    store.ns_hls_has(ns_id, syn_id)
}

/// Get a namespace highlight item. Returns COLOR_ITEM_INITIALIZER if not found.
#[no_mangle]
pub extern "C" fn rs_ns_hls_get(ns_id: c_int, syn_id: c_int) -> ColorItem {
    let store = ATTR_STORE.lock().unwrap();
    store.ns_hls_get(ns_id, syn_id)
}

/// Put a namespace highlight item.
#[no_mangle]
pub extern "C" fn rs_ns_hls_put(ns_id: c_int, syn_id: c_int, item: ColorItem) {
    let mut store = ATTR_STORE.lock().unwrap();
    store.ns_hls_put(ns_id, syn_id, item);
}

/// Define a highlight in a namespace.
/// This is the core logic of ns_hl_def() moved to Rust.
///
/// # Arguments
/// * `ns_id` - Namespace ID (must be > 0, ns_id=0 is handled by C)
/// * `hl_id` - Highlight group ID (syn_id)
/// * `attrs` - Highlight attributes
/// * `link_id` - Link target ID (0 if not a link)
///
/// The function:
/// 1. Checks if HL_DEFAULT flag is set and entry already exists -> returns false
/// 2. Gets/creates DecorProvider and retrieves hl_valid, sets hl_cached=false
/// 3. Computes attr_id via hl_get_syn_attr (if not a link)
/// 4. Creates ColorItem and stores it
///
/// Returns true if the highlight was defined, false if skipped due to HL_DEFAULT.
#[no_mangle]
pub extern "C" fn rs_ns_hl_def(ns_id: c_int, hl_id: c_int, attrs: HlAttrs, link_id: c_int) -> bool {
    let is_default = (attrs.rgb_ae_attr & HL_DEFAULT) != 0;
    let is_global = (attrs.rgb_ae_attr & HL_GLOBAL) != 0;

    // If HL_DEFAULT is set and entry already exists, skip
    if is_default {
        let store = ATTR_STORE.lock().unwrap();
        if store.ns_hls_has(ns_id, hl_id) {
            return false;
        }
    }

    // Get hl_valid from DecorProvider and set hl_cached = false
    let version = unsafe { nvim_decor_provider_hl_def_prepare(ns_id) };

    // Compute attr_id: -1 if link, otherwise call hl_get_syn_attr
    let attr_id = if link_id > 0 {
        -1
    } else {
        unsafe { rs_hl_get_syn_attr(ns_id, hl_id, attrs) }
    };

    // Create and store the ColorItem
    let item = ColorItem {
        attr_id,
        link_id,
        version,
        is_default,
        link_global: is_global,
    };

    let mut store = ATTR_STORE.lock().unwrap();
    store.ns_hls_put(ns_id, hl_id, item);

    true
}

// ============================================================================
// FFI Functions for ns_get_hl() Pre/Post Split
// ============================================================================
//
// ns_get_hl() is split into three parts:
// 1. Pre: Check cache, resolve namespace, determine if Lua callback is needed
// 2. Middle: Lua callback (stays in C)
// 3. Post: Store result and compute final return value

/// Result from rs_ns_get_hl_pre().
/// Tells C whether to call Lua callback or return immediately.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NsGetHlPreResult {
    /// Resolved namespace ID (updated from input if was < 0)
    pub ns_id: c_int,
    /// If true, caller should invoke Lua callback
    pub need_callback: bool,
    /// If need_callback is false, this is the final result
    pub result: c_int,
    /// If need_callback is false and link was requested, indicates if ns should be set to 0
    pub set_ns_to_zero: bool,
    /// The current ColorItem for this hl_id (for C to use if callback needed)
    pub item: ColorItem,
}

/// Result from C bridge `c_ns_get_hl_lua_call()`.
#[repr(C)]
pub struct NsGetHlLuaResult {
    pub ret: Object,
    pub is_recursive: bool,
}

extern "C" {
    /// C bridge for Lua callback in ns_get_hl.
    /// Handles recursion guard, DecorProvider lookup, args building, and nlua_call_ref.
    fn c_ns_get_hl_lua_call(ns_id: c_int, hl_id: c_int, link: bool) -> NsGetHlLuaResult;
}

/// Pre-callback phase of ns_get_hl().
/// Checks if we have a valid cached entry or need to call Lua.
///
/// # Arguments
/// * `ns_hl` - Namespace ID (can be negative, meaning use ns_hl_active)
/// * `hl_id` - Highlight group ID
/// * `link` - If true, return link target instead of attr
/// * `nodefault` - If true, return -1 for default highlights
///
/// # Returns
/// A result indicating whether Lua callback is needed, or the final result.
#[no_mangle]
pub extern "C" fn rs_ns_get_hl_pre(
    ns_hl: c_int,
    hl_id: c_int,
    link: bool,
    nodefault: bool,
) -> NsGetHlPreResult {
    // ns=0 (the default namespace) does not have a provider
    if ns_hl == 0 {
        return NsGetHlPreResult {
            ns_id: 0,
            need_callback: false,
            result: -1,
            set_ns_to_zero: false,
            item: ColorItem::default(),
        };
    }

    // Resolve negative ns_hl to ns_hl_active
    let ns_id = if ns_hl < 0 {
        let active = unsafe { nvim_get_ns_hl_active() };
        if active <= 0 {
            return NsGetHlPreResult {
                ns_id: active,
                need_callback: false,
                result: -1,
                set_ns_to_zero: false,
                item: ColorItem::default(),
            };
        }
        active
    } else {
        ns_hl
    };

    // Get the cached ColorItem
    let store = ATTR_STORE.lock().unwrap();
    let item = store.ns_hls_get(ns_id, hl_id);
    drop(store);

    // Check if item is valid (version >= hl_valid)
    let hl_valid = unsafe { nvim_decor_provider_get_hl_valid(ns_id) };
    let valid_item = item.version >= hl_valid;

    // Check if we need to call the Lua callback
    let has_callback = unsafe { nvim_decor_provider_has_hl_def(ns_id) };

    if !valid_item && has_callback {
        // Need to call Lua callback - C will handle this
        return NsGetHlPreResult {
            ns_id,
            need_callback: true,
            result: 0,
            set_ns_to_zero: false,
            item,
        };
    }

    // No callback needed - compute final result
    compute_ns_get_hl_result(ns_id, &item, valid_item, link, nodefault)
}

/// Post-callback phase of ns_get_hl().
/// Called after Lua callback to store the new ColorItem and compute result.
///
/// # Arguments
/// * `ns_id` - Namespace ID
/// * `hl_id` - Highlight group ID
/// * `attrs` - Parsed highlight attributes from Lua
/// * `link_id` - Link target from Lua (-1 if no link)
/// * `fallback` - Whether highlight should fall back
/// * `version_offset` - Offset to subtract from hl_valid (for "tmp" flag)
/// * `link` - If true, return link target
/// * `nodefault` - If true, return -1 for default highlights
///
/// # Returns
/// A result with the final value and whether to set ns to 0.
#[no_mangle]
pub extern "C" fn rs_ns_get_hl_post(
    ns_id: c_int,
    hl_id: c_int,
    attrs: HlAttrs,
    link_id: c_int,
    fallback: bool,
    version_offset: c_int,
    link: bool,
    nodefault: bool,
) -> NsGetHlPreResult {
    // Get hl_valid from provider
    let hl_valid = unsafe { nvim_decor_provider_get_hl_valid(ns_id) };

    // Compute attr_id
    let attr_id = if fallback {
        -1
    } else {
        unsafe { rs_hl_get_syn_attr(ns_id, hl_id, attrs) }
    };

    // Create and store ColorItem
    let item = ColorItem {
        attr_id,
        link_id,
        version: hl_valid - version_offset,
        is_default: (attrs.rgb_ae_attr & HL_DEFAULT) != 0,
        link_global: (attrs.rgb_ae_attr & HL_GLOBAL) != 0,
    };

    {
        let mut store = ATTR_STORE.lock().unwrap();
        store.ns_hls_put(ns_id, hl_id, item);
    }

    // Compute and return final result (valid_item is true after storing)
    compute_ns_get_hl_result(ns_id, &item, true, link, nodefault)
}

/// Helper to compute final ns_get_hl result from ColorItem.
fn compute_ns_get_hl_result(
    ns_id: c_int,
    item: &ColorItem,
    valid_item: bool,
    link: bool,
    nodefault: bool,
) -> NsGetHlPreResult {
    // Check is_default && nodefault, or !valid_item
    if (item.is_default && nodefault) || !valid_item {
        return NsGetHlPreResult {
            ns_id,
            need_callback: false,
            result: -1,
            set_ns_to_zero: false,
            item: *item,
        };
    }

    if link {
        if item.attr_id >= 0 {
            // Has attr, not a link
            NsGetHlPreResult {
                ns_id,
                need_callback: false,
                result: 0,
                set_ns_to_zero: false,
                item: *item,
            }
        } else {
            // Is a link
            NsGetHlPreResult {
                ns_id,
                need_callback: false,
                result: item.link_id,
                set_ns_to_zero: item.link_global,
                item: *item,
            }
        }
    } else {
        NsGetHlPreResult {
            ns_id,
            need_callback: false,
            result: item.attr_id,
            set_ns_to_zero: false,
            item: *item,
        }
    }
}

/// Full implementation of ns_get_hl: pre + Lua callback + parse + post.
///
/// This replaces the old C ns_get_hl function body. The Lua callback is
/// dispatched through a C bridge (`c_ns_get_hl_lua_call`), but all other
/// logic (cache check, dict parsing, result storage) is in Rust.
#[export_name = "ns_get_hl"]
pub unsafe extern "C" fn rs_ns_get_hl_full(
    ns_hl: *mut c_int,
    hl_id: c_int,
    link: bool,
    nodefault: bool,
) -> c_int {
    // Pre-callback phase: check cache, resolve namespace
    let pre = rs_ns_get_hl_pre(*ns_hl, hl_id, link, nodefault);
    *ns_hl = pre.ns_id;

    // If no callback needed, return the result directly
    if !pre.need_callback {
        if pre.set_ns_to_zero {
            *ns_hl = 0;
        }
        return pre.result;
    }

    // Lua callback phase via C bridge
    let lua_result = c_ns_get_hl_lua_call(pre.ns_id, hl_id, link);

    if lua_result.is_recursive {
        return -1;
    }

    let ret = lua_result.ret;

    // Parse Lua callback result
    let mut fallback = true;
    let mut version_offset: c_int = 0;
    let mut attrs = HlAttrs::new();
    let mut link_id = pre.item.link_id;

    if ret.obj_type == ObjectType::Dict as c_int {
        fallback = false;
        let dict = ret.data.dict;
        let mut err = Error {
            err_type: -1,
            msg: std::ptr::null_mut(),
        };
        let parsed = dict2hlattrs_impl(dict, true, true, &mut err);
        if !error_is_set(&err) {
            attrs = parsed.attrs;
            link_id = parsed.link_id;
            fallback = parsed.fallback;
            version_offset = parsed.version_offset;
            if parsed.link_id >= 0 {
                fallback = true;
            }
        }
    }

    // Post-callback phase: store result and compute final return value
    let post = rs_ns_get_hl_post(
        pre.ns_id,
        hl_id,
        attrs,
        link_id,
        fallback,
        version_offset,
        link,
        nodefault,
    );

    if post.set_ns_to_zero {
        *ns_hl = 0;
    }

    post.result
}

// ============================================================================
// FFI Functions for hl_check_ns()
// ============================================================================

/// Check and switch to the active highlight namespace.
/// Returns true if the namespace changed.
///
/// This implements the core logic of hl_check_ns():
/// 1. Resolve namespace priority: ns_hl_fast > ns_hl_win > ns_hl_global
/// 2. If namespace unchanged, return false
/// 3. Update ns_hl_active and hl_attr_active
/// 4. If namespace > 0, call update_ns_hl and get namespace attrs
/// 5. Set need_highlight_changed = true
#[must_use]
#[export_name = "hl_check_ns"]
pub extern "C" fn rs_hl_check_ns() -> bool {
    // Resolve namespace priority
    let ns_hl_fast = unsafe { nvim_get_ns_hl_fast() };
    let ns_hl_win = unsafe { nvim_get_ns_hl_win() };
    let ns_hl_global = unsafe { nvim_get_ns_hl_global() };

    let ns = if ns_hl_fast > 0 {
        ns_hl_fast
    } else if ns_hl_win >= 0 {
        ns_hl_win
    } else {
        ns_hl_global
    };

    // Check if namespace changed
    let ns_hl_active = unsafe { nvim_get_ns_hl_active() };
    if ns_hl_active == ns {
        return false;
    }

    // Update active namespace
    unsafe { nvim_set_ns_hl_active(ns) };

    // Reset to default highlight_attr
    let highlight_attr = unsafe { nvim_get_highlight_attr() };
    unsafe { nvim_set_hl_attr_active(highlight_attr) };

    // If namespace > 0, update namespace highlights and get the attrs
    if ns > 0 {
        // Update namespace highlights
        unsafe { rs_update_ns_hl(ns) };

        // Get the namespace-specific attribute array
        let store = ATTR_STORE.lock().unwrap();
        let hl_def = store.ns_hl_attr_get(ns);
        drop(store);

        if !hl_def.is_null() {
            unsafe { nvim_set_hl_attr_active(hl_def) };
        }
    }

    // Signal that highlights changed
    unsafe { nvim_set_need_highlight_changed(true) };

    true
}

/// Update namespace highlight attributes.
///
/// This function populates the namespace's UI highlight attribute array
/// by iterating through all HLF_* types and computing their attributes.
///
/// # Safety
/// Must be called from the main thread (accesses global state).
#[no_mangle]
pub unsafe extern "C" fn rs_update_ns_hl(ns_id: c_int) {
    if ns_id <= 0 {
        return;
    }

    // Check if already cached
    if nvim_decor_provider_get_hl_cached(ns_id) {
        return;
    }

    // Get or create the attribute array
    let hl_attrs = rs_ns_hl_attr_get_or_create(ns_id);

    let hlf_count = nvim_get_hlf_count();
    let hlf_inactive = nvim_get_hlf_inactive();
    let hlf_nfloat = nvim_get_hlf_nfloat();
    let hlf_none = nvim_get_hlf_none();

    // Iterate through all HLF_* types (starting from 1, skipping HLF_NONE)
    for hlf in 1..hlf_count {
        let name = nvim_get_hlf_name(hlf);
        if name.is_null() {
            continue;
        }
        let name_cstr = CStr::from_ptr(name);
        let name_len = name_cstr.to_bytes().len();
        let id = rs_syn_check_group(name, name_len);
        let optional = hlf == hlf_inactive || hlf == hlf_nfloat;
        *hl_attrs.add(hlf as usize) = rs_hl_get_ui_attr(ns_id, hlf, id, optional);
    }

    // Handle "Normal" specially - stored at HLF_NONE (index 0)
    static NORMAL: &[u8] = b"Normal\0";
    let normality = rs_syn_check_group(NORMAL.as_ptr() as *const c_char, 6);
    *hl_attrs.add(hlf_none as usize) = rs_hl_get_ui_attr(ns_id, -1, normality, true);

    // Mark as cached (hl_get_ui_attr might have invalidated, so re-get provider)
    nvim_decor_provider_set_hl_cached(ns_id, true, true);
}

// ============================================================================
// FFI Functions for Per-namespace UI Highlight Attributes (ns_hl_attr)
// ============================================================================

/// Get pointer to namespace UI highlight attributes.
/// Returns NULL if no entry exists for this namespace.
#[no_mangle]
pub extern "C" fn rs_ns_hl_attr_get(ns_id: c_int) -> *const c_int {
    let store = ATTR_STORE.lock().unwrap();
    store.ns_hl_attr_get(ns_id)
}

/// Get or create pointer to namespace UI highlight attributes.
/// Creates a zeroed array if no entry exists.
/// Returns mutable pointer for C to fill in the values.
#[no_mangle]
pub extern "C" fn rs_ns_hl_attr_get_or_create(ns_id: c_int) -> *mut c_int {
    let mut store = ATTR_STORE.lock().unwrap();
    store.ns_hl_attr_get_or_create(ns_id)
}

// ============================================================================
// Namespace Global Accessor FFI Functions
// ============================================================================
//
// These functions wrap C accessors for namespace globals, allowing Rust code
// to read/write these values. The globals remain in C for now because they
// are accessed from many files, but Rust can call these to manipulate them.

/// Get the global highlight namespace (0 = use default).
#[no_mangle]
pub extern "C" fn rs_get_ns_hl_global() -> c_int {
    unsafe { nvim_get_ns_hl_global() }
}

/// Set the global highlight namespace.
#[no_mangle]
pub extern "C" fn rs_set_ns_hl_global(ns: c_int) {
    unsafe { nvim_set_ns_hl_global(ns) }
}

/// Get the window-specific highlight namespace (-1 = not set).
#[no_mangle]
pub extern "C" fn rs_get_ns_hl_win() -> c_int {
    unsafe { nvim_get_ns_hl_win() }
}

/// Set the window-specific highlight namespace.
#[no_mangle]
pub extern "C" fn rs_set_ns_hl_win(ns: c_int) {
    unsafe { nvim_set_ns_hl_win(ns) }
}

/// Get the fast callback highlight namespace (-1 = not set).
#[no_mangle]
pub extern "C" fn rs_get_ns_hl_fast() -> c_int {
    unsafe { nvim_get_ns_hl_fast() }
}

/// Set the fast callback highlight namespace.
#[no_mangle]
pub extern "C" fn rs_set_ns_hl_fast(ns: c_int) {
    unsafe { nvim_set_ns_hl_fast(ns) }
}

/// Get the currently active/cached highlight namespace.
#[no_mangle]
pub extern "C" fn rs_get_ns_hl_active() -> c_int {
    unsafe { nvim_get_ns_hl_active() }
}

/// Set the currently active/cached highlight namespace.
#[no_mangle]
pub extern "C" fn rs_set_ns_hl_active(ns: c_int) {
    unsafe { nvim_set_ns_hl_active(ns) }
}

/// Get pointer to currently active highlight attributes.
#[no_mangle]
pub extern "C" fn rs_get_hl_attr_active() -> *const c_int {
    unsafe { nvim_get_hl_attr_active() }
}

/// Set pointer to active highlight attributes.
///
/// # Safety
/// The caller must ensure that `attrs` is a valid pointer to an array of
/// at least HLF_COUNT integers.
#[no_mangle]
pub unsafe extern "C" fn rs_set_hl_attr_active(attrs: *const c_int) {
    nvim_set_hl_attr_active(attrs)
}

/// Get pointer to default highlight_attr array.
#[no_mangle]
pub extern "C" fn rs_get_highlight_attr() -> *const c_int {
    unsafe { nvim_get_highlight_attr() }
}

/// Invalidate blend caches. Called when colors change.
#[no_mangle]
pub extern "C" fn rs_hl_invalidate_blends() {
    let mut store = ATTR_STORE.lock().unwrap();
    store.clear_blend_caches();
}

/// Check if an entry with the given key exists in the combine cache.
/// Returns the cached ID or -1 if not found.
#[no_mangle]
pub extern "C" fn rs_combine_cache_get(combine_tag: c_int) -> c_int {
    let store = ATTR_STORE.lock().unwrap();
    store.combine_cache.get(&combine_tag).copied().unwrap_or(-1)
}

/// Insert a value into the combine cache.
#[no_mangle]
pub extern "C" fn rs_combine_cache_put(combine_tag: c_int, id: c_int) {
    let mut store = ATTR_STORE.lock().unwrap();
    store.combine_cache.insert(combine_tag, id);
}

/// Check if an entry with the given key exists in the blend cache.
/// Returns the cached ID or -1 if not found.
#[no_mangle]
pub extern "C" fn rs_blend_cache_get(combine_tag: c_int, through: bool) -> c_int {
    let store = ATTR_STORE.lock().unwrap();
    let cache = if through {
        &store.blendthrough_cache
    } else {
        &store.blend_cache
    };
    cache.get(&combine_tag).copied().unwrap_or(-1)
}

/// Insert a value into the blend cache.
#[no_mangle]
pub extern "C" fn rs_blend_cache_put(combine_tag: c_int, id: c_int, through: bool) {
    let mut store = ATTR_STORE.lock().unwrap();
    let cache = if through {
        &mut store.blendthrough_cache
    } else {
        &mut store.blend_cache
    };
    cache.insert(combine_tag, id);
}

// ============================================================================
// Highlight Group Name Functions (syn_* family from highlight_group.c)
// ============================================================================

/// Maximum length for a highlight group name.
/// Must match MAX_SYN_NAME in highlight_group.c.
const MAX_SYN_NAME: usize = 200;

/// Empty string constant for returning when group ID is invalid.
/// This is a static CStr that lives for the lifetime of the program.
static EMPTY_STRING: &CStr = c"";

/// Return the name of highlight group "id".
/// When not a valid ID (1-based), returns an empty string.
///
/// This mirrors the C function `syn_id2name(int id)` from highlight_group.c.
/// Note: The returned pointer points to memory owned by C (highlight_arena).
#[export_name = "syn_id2name"]
pub extern "C" fn rs_syn_id2name(id: c_int) -> *const c_char {
    let num_groups = unsafe { highlight_num_groups() };
    if id <= 0 || id > num_groups {
        return EMPTY_STRING.as_ptr();
    }
    // id is 1-based, index is 0-based
    unsafe { highlight_group_name(id - 1) }
}

/// Lookup a highlight group name and return its ID.
///
/// This mirrors the C function `syn_name2id_len(const char *name, size_t len)`.
/// Returns the highlight group ID (1-based), or 0 if not found.
///
/// # Safety
/// The name pointer must be valid for at least `len` bytes.
#[export_name = "syn_name2id_len"]
pub unsafe extern "C" fn rs_syn_name2id_len(name: *const c_char, len: usize) -> c_int {
    if name.is_null() || len == 0 || len > MAX_SYN_NAME {
        return 0;
    }

    // Convert name to uppercase in a stack buffer
    let mut name_u = [0u8; MAX_SYN_NAME + 1];
    let name_bytes = std::slice::from_raw_parts(name as *const u8, len);
    for (i, &b) in name_bytes.iter().enumerate() {
        name_u[i] = b.to_ascii_uppercase();
    }
    name_u[len] = 0; // null-terminate

    // Lookup in the highlight_unames map via C accessor
    nvim_highlight_name_lookup(name_u.as_ptr() as *const c_char)
}

// C functions for highlight group operations
extern "C" {
    /// Add a new highlight group (stays in C due to Arena allocation)
    fn c_syn_add_group(name: *const c_char, len: usize) -> c_int;
    /// Emit an error message
    fn emsg(s: *const c_char) -> c_int;
    /// extern error string for highlight group name too long
    static e_highlight_group_name_too_long: c_char;
    /// gettext translation function (no-op in many builds, but required for correctness)
    fn gettext(s: *const c_char) -> *const c_char;
}

/// Find highlight group name in the table and return its ID.
/// If it doesn't exist yet, a new entry is created (via C).
///
/// This directly provides the C symbol `syn_check_group`, replacing the thin wrapper.
/// Returns the highlight group ID (1-based), or 0 for failure.
///
/// # Safety
/// The name pointer must be valid for at least `len` bytes.
#[export_name = "syn_check_group"]
pub unsafe extern "C" fn rs_syn_check_group(name: *const c_char, len: usize) -> c_int {
    if name.is_null() || len == 0 {
        return 0;
    }
    if len > MAX_SYN_NAME {
        emsg(gettext(&e_highlight_group_name_too_long as *const c_char));
        return 0;
    }

    // Try to find existing group first
    let id = rs_syn_name2id_len(name, len);
    if id != 0 {
        return id;
    }

    // Group doesn't exist - call C to create it (handles Arena allocation)
    c_syn_add_group(name, len)
}

/// Translate a group ID to the final group ID (following links).
/// Also checks namespace overrides.
///
/// This mirrors the C function `syn_ns_get_final_id(int *ns_id, int *hl_idp)`.
///
/// # Arguments
/// * `ns_id` - Pointer to namespace ID (may be modified)
/// * `hl_idp` - Pointer to highlight group ID (will be set to final ID)
///
/// # Returns
/// true if a namespace explicitly defined a value (making the highlight non-optional).
///
/// # Safety
/// Both pointers must be valid.
#[export_name = "syn_ns_get_final_id"]
pub unsafe extern "C" fn rs_syn_ns_get_final_id(ns_id: *mut c_int, hl_idp: *mut c_int) -> bool {
    let mut hl_id = *hl_idp;
    let mut used = false;

    let num_groups = highlight_num_groups();
    if hl_id > num_groups || hl_id < 1 {
        *hl_idp = 0;
        return false; // Can be called from eval!!
    }

    // Follow links until there is no more.
    // Look out for loops! Break after 100 links.
    for _count in 0..100 {
        let idx = hl_id - 1; // index is ID minus one

        // Get sg_set for this group (needed for ns_get_hl)
        let sg_set = highlight_group_set(idx);

        // Check namespace override (link=true to get link target)
        let check = rs_ns_get_hl_full(ns_id, hl_id, true, sg_set != 0);

        if check == 0 {
            // Namespace explicitly defined this group to be empty (broke the link)
            *hl_idp = hl_id;
            return true;
        } else if check > 0 {
            // Namespace provides a link target
            used = true;
            hl_id = check;
            continue;
        }

        // check < 0 means no namespace override, use hl_table values

        // Check sg_link
        let link_id = highlight_link_id(idx);
        if link_id > 0 && link_id <= num_groups {
            hl_id = link_id;
            continue;
        }

        // Check sg_parent for cleared @nested.groups
        let cleared = highlight_group_cleared(idx);
        let parent = highlight_group_parent(idx);
        if cleared && parent > 0 {
            hl_id = parent;
            continue;
        }

        // No more links to follow
        break;
    }

    *hl_idp = hl_id;
    used
}

/// Translate a group ID to highlight attributes.
/// Also checks namespace overrides.
///
/// This mirrors the C function `syn_ns_id2attr(int ns_id, int hl_id, bool *optional)`.
///
/// # Arguments
/// * `ns_id` - Namespace ID (-1 for current active namespace)
/// * `hl_id` - Highlight group ID (1-based)
/// * `optional` - Pointer to flag indicating if highlight is optional
///
/// # Returns
/// The attribute ID for this highlight group.
///
/// # Safety
/// The optional pointer must be valid.
#[export_name = "syn_ns_id2attr"]
pub unsafe extern "C" fn rs_syn_ns_id2attr(
    mut ns_id: c_int,
    mut hl_id: c_int,
    optional: *mut bool,
) -> c_int {
    // Follow links to final ID
    if rs_syn_ns_get_final_id(&mut ns_id, &mut hl_id) {
        // If the namespace explicitly defines a group to be empty, it is not optional
        *optional = false;
    }

    // Handle case where hl_id became 0 (invalid)
    if hl_id == 0 {
        return 0;
    }

    let idx = hl_id - 1; // index is ID minus one

    // Get sg_set for ns_get_hl
    let sg_set = highlight_group_set(idx);

    // Check namespace for attribute (link=false to get attr, not link target)
    let attr = rs_ns_get_hl_full(&mut ns_id, hl_id, false, sg_set != 0);

    // if a highlight group is optional, don't use the global value
    if attr >= 0 || (*optional && ns_id > 0) {
        return attr;
    }

    // Fall back to sg_attr from hl_table
    highlight_group_attr(idx)
}

/// Translate a group ID to highlight attributes.
/// This is a simple wrapper around rs_syn_ns_id2attr(-1, hl_id, &optional).
///
/// This mirrors the C function `syn_id2attr(int hl_id)`.
#[export_name = "syn_id2attr"]
pub unsafe extern "C" fn rs_syn_id2attr(hl_id: c_int) -> c_int {
    let mut optional = false;
    rs_syn_ns_id2attr(-1, hl_id, &mut optional)
}

/// Translate a group ID to the final group ID (following links).
/// Uses the current window's active namespace.
///
/// This mirrors the C function `syn_get_final_id(int hl_id)`.
///
/// Note: This function needs access to curwin->w_ns_hl_active which
/// requires a window accessor. For now we pass -1 to use the current
/// active namespace.
#[export_name = "syn_get_final_id"]
pub unsafe extern "C" fn rs_syn_get_final_id(mut hl_id: c_int) -> c_int {
    // Get current window's active namespace via C accessor
    let mut ns_id = c_curwin_ns_hl_active();
    rs_syn_ns_get_final_id(&mut ns_id, &mut hl_id);
    hl_id
}

/// Lookup a highlight group name and return its attributes.
/// Returns 0 if not found.
///
/// This mirrors the C function `syn_name2attr(const char *name)`.
///
/// # Safety
/// The name pointer must be a valid null-terminated C string.
#[export_name = "syn_name2attr"]
pub unsafe extern "C" fn rs_syn_name2attr(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }
    let name_cstr = CStr::from_ptr(name);
    let len = name_cstr.to_bytes().len();
    let id = rs_syn_name2id_len(name, len);
    if id != 0 {
        rs_syn_id2attr(id)
    } else {
        0
    }
}

/// Return true (1) if highlight group "name" exists.
///
/// This mirrors the C function `highlight_exists(const char *name)`.
///
/// # Safety
/// The name pointer must be a valid null-terminated C string.
#[export_name = "highlight_exists"]
pub unsafe extern "C" fn rs_highlight_exists(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }
    let name_cstr = CStr::from_ptr(name);
    let len = name_cstr.to_bytes().len();
    let id = rs_syn_name2id_len(name, len);
    if id > 0 {
        1
    } else {
        0
    }
}

// ============================================================================
// Color Reset Functions
// ============================================================================

extern "C" {
    /// Set normal_fg global
    fn nvim_set_normal_fg(val: c_int);
    /// Set normal_bg global
    fn nvim_set_normal_bg(val: c_int);
    /// Set normal_sp global
    fn nvim_set_normal_sp(val: c_int);
    /// Set cterm_normal_fg_color global
    fn nvim_set_cterm_normal_fg_color(val: c_int);
    /// Set cterm_normal_bg_color global
    fn nvim_set_cterm_normal_bg_color(val: c_int);
    /// Get curwin->w_ns_hl_active (current window's active namespace)
    fn c_curwin_ns_hl_active() -> c_int;
    /// Get w_ns_hl field from a window
    fn nvim_win_get_ns_hl(wp: *mut c_void) -> c_int;
}

/// Reset the cterm colors to what they were before Vim was started.
/// Resets normal_fg, normal_bg, normal_sp to -1 and cterm colors to 0.
///
/// This mirrors the C function `restore_cterm_colors(void)`.
#[export_name = "restore_cterm_colors"]
pub extern "C" fn rs_restore_cterm_colors() {
    unsafe {
        nvim_set_normal_fg(-1);
        nvim_set_normal_bg(-1);
        nvim_set_normal_sp(-1);
        nvim_set_cterm_normal_fg_color(0);
        nvim_set_cterm_normal_bg_color(0);
    }
}

// ============================================================================
// Window Highlight Functions
// ============================================================================

/// Prepare window for drawing with namespace highlights.
/// Sets ns_hl_win to the window's namespace and calls hl_check_ns().
///
/// This mirrors the C function `win_check_ns_hl(win_T *wp)`.
///
/// # Arguments
/// * `wp` - Pointer to win_T struct (can be null)
///
/// # Returns
/// true if the namespace changed.
///
/// # Safety
/// The wp pointer must be either null or a valid pointer to a win_T struct.
#[export_name = "win_check_ns_hl"]
pub unsafe extern "C" fn rs_win_check_ns_hl(wp: *mut c_void) -> bool {
    // Set ns_hl_win based on whether wp is provided
    let ns_hl = if wp.is_null() {
        -1
    } else {
        nvim_win_get_ns_hl(wp)
    };
    rs_set_ns_hl_win(ns_hl);
    rs_hl_check_ns()
}

// ============================================================================
// Attribute Entry Creation Functions
// ============================================================================

/// Gets HL_UNDERLINE highlight.
/// Creates an attribute entry with just underline style set.
///
/// This mirrors the C function `hl_get_underline(void)`.
#[must_use]
#[export_name = "hl_get_underline"]
pub extern "C" fn rs_hl_get_underline() -> c_int {
    let entry = HlEntry {
        attr: HlAttrs {
            cterm_ae_attr: HL_UNDERLINE,
            cterm_fg_color: 0,
            cterm_bg_color: 0,
            rgb_ae_attr: HL_UNDERLINE,
            rgb_fg_color: -1,
            rgb_bg_color: -1,
            rgb_sp_color: -1,
            hl_blend: -1,
            url: -1,
        },
        kind: HlKind::UI,
        id1: 0,
        id2: 0,
        winid: 0,
    };
    unsafe { rs_get_attr_entry_full(entry) }
}

/// Get attribute code for forwarded :terminal highlights.
///
/// This mirrors the C function `hl_get_term_attr(HlAttrs *aep)`.
///
/// # Safety
/// The aep pointer must be valid.
#[export_name = "hl_get_term_attr"]
pub unsafe extern "C" fn rs_hl_get_term_attr(aep: *const HlAttrs) -> c_int {
    if aep.is_null() {
        return 0;
    }
    let entry = HlEntry {
        attr: *aep,
        kind: HlKind::Terminal,
        id1: 0,
        id2: 0,
        winid: 0,
    };
    rs_get_attr_entry_full(entry)
}

/// Apply 'winblend' to highlight attributes.
/// If the blend attribute is not set, the winblend value overrides it.
///
/// This mirrors the C function `hl_apply_winblend(int winbl, int attr)`.
///
/// # Arguments
/// * `winbl` - The 'winblend' value
/// * `attr` - The original attribute code
///
/// # Returns
/// The attribute code with 'winblend' applied.
#[must_use]
#[export_name = "hl_apply_winblend"]
pub extern "C" fn rs_hl_apply_winblend(winbl: c_int, attr: c_int) -> c_int {
    let mut entry = rs_get_attr_entry_by_id(attr);
    // if blend= attribute is not set, 'winblend' value overrides it.
    if entry.attr.hl_blend == -1 && winbl > 0 {
        entry.attr.hl_blend = winbl;
        unsafe { rs_get_attr_entry_full(entry) }
    } else {
        attr
    }
}

// ============================================================================
// URL Management Functions
// ============================================================================

/// Add or lookup a URL. Returns the index.
///
/// # Safety
/// The url pointer must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_hl_add_url_index(url: *const c_char) -> u32 {
    let url_str = std::ffi::CStr::from_ptr(url).to_str().unwrap_or("");
    let mut store = ATTR_STORE.lock().unwrap();
    store.add_url(url_str)
}

/// Get a URL by its index. Returns null if index is invalid.
///
/// Note: The returned pointer points to memory owned by the URL store.
/// It remains valid as long as the URL store isn't cleared.
/// This is safe because:
/// 1. URLs are never removed individually (only during clear_hl_tables)
/// 2. CString memory is stable (doesn't move during reallocations of Vec)
#[must_use]
#[export_name = "hl_get_url"]
pub extern "C" fn rs_hl_get_url(index: u32) -> *const c_char {
    let store = ATTR_STORE.lock().unwrap();
    store.get_url_ptr(index)
}

/// Get the number of URLs stored.
#[no_mangle]
pub extern "C" fn rs_hl_url_count() -> u32 {
    let store = ATTR_STORE.lock().unwrap();
    store.urls.len() as u32
}

// ============================================================================
// Highlight Attribute Combination Functions
// ============================================================================

/// Combine two attribute flags, handling underline styles specially.
/// The primary (prim_ae) attribute overrides the char attribute, except
/// for underline styles where we prefer the primary if set.
#[no_mangle]
pub extern "C" fn rs_hl_combine_ae(char_ae: i16, prim_ae: i16) -> i16 {
    let char_ul = char_ae & HL_UNDERLINE_MASK;
    let prim_ul = prim_ae & HL_UNDERLINE_MASK;
    let new_ul = if prim_ul != 0 { prim_ul } else { char_ul };
    (char_ae & !HL_UNDERLINE_MASK) | (prim_ae & !HL_UNDERLINE_MASK) | new_ul
}

/// Get the used RGB colors for an attr group, filling in defaults.
/// If colors are unset (-1), use normal_fg/bg/sp or builtin defaults.
/// Also handles HL_INVERSE by swapping fg and bg.
///
/// # Safety
/// This function calls C accessor functions for global variables.
/// It must only be called when the Neovim runtime is properly initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_get_colors_force(mut attrs: HlAttrs) -> HlAttrs {
    // Fill in defaults from normal highlight group
    if attrs.rgb_bg_color == -1 {
        attrs.rgb_bg_color = nvim_get_normal_bg();
    }
    if attrs.rgb_fg_color == -1 {
        attrs.rgb_fg_color = nvim_get_normal_fg();
    }
    if attrs.rgb_sp_color == -1 {
        attrs.rgb_sp_color = nvim_get_normal_sp();
    }

    // Apply builtin defaults based on 'background' option
    let dark = nvim_get_p_bg() == b'd' as c_char;
    if attrs.rgb_fg_color == -1 {
        attrs.rgb_fg_color = if dark { 0xFFFFFF } else { 0x000000 };
    }
    if attrs.rgb_bg_color == -1 {
        attrs.rgb_bg_color = if dark { 0x000000 } else { 0xFFFFFF };
    }
    if attrs.rgb_sp_color == -1 {
        attrs.rgb_sp_color = 0xFF0000; // default special color is red
    }

    // Handle inverse attribute by swapping fg and bg
    if (attrs.rgb_ae_attr & HL_INVERSE) != 0 {
        std::mem::swap(&mut attrs.rgb_fg_color, &mut attrs.rgb_bg_color);
        attrs.rgb_ae_attr &= !HL_INVERSE;
    }

    attrs
}

/// Input for hl_combine_attr computation - contains the two attribute sets to combine
#[repr(C)]
pub struct HlCombineInput {
    /// Character (low-priority) attributes
    pub char_aep: HlAttrs,
    /// Primary (high-priority) attributes
    pub prim_aep: HlAttrs,
}

/// Compute combined highlight attributes.
///
/// This is the pure computation core of hl_combine_attr. It takes pre-fetched
/// attributes and returns the combined result without any side effects.
///
/// The primary (prim_aep) attributes override the character (char_aep) attributes,
/// with special handling for various properties like colors, blend, and URL.
#[no_mangle]
pub extern "C" fn rs_hl_combine_attrs_compute(input: HlCombineInput) -> HlAttrs {
    let HlCombineInput { char_aep, prim_aep } = input;

    // Start with low-priority attribute, and override colors if present below
    let mut new_en = char_aep;

    // Handle cterm attributes with HL_NOCOMBINE check
    if (prim_aep.cterm_ae_attr & HL_NOCOMBINE) != 0 {
        new_en.cterm_ae_attr = prim_aep.cterm_ae_attr;
    } else {
        new_en.cterm_ae_attr = rs_hl_combine_ae(new_en.cterm_ae_attr, prim_aep.cterm_ae_attr);
    }

    // Handle rgb attributes with HL_NOCOMBINE check
    if (prim_aep.rgb_ae_attr & HL_NOCOMBINE) != 0 {
        new_en.rgb_ae_attr = prim_aep.rgb_ae_attr;
    } else {
        new_en.rgb_ae_attr = rs_hl_combine_ae(new_en.rgb_ae_attr, prim_aep.rgb_ae_attr);
    }

    // Override cterm foreground color if primary has one
    if prim_aep.cterm_fg_color > 0 {
        new_en.cterm_fg_color = prim_aep.cterm_fg_color;
        new_en.rgb_ae_attr &= (!HL_FG_INDEXED) | (prim_aep.rgb_ae_attr & HL_FG_INDEXED);
    }

    // Override cterm background color if primary has one
    if prim_aep.cterm_bg_color > 0 {
        new_en.cterm_bg_color = prim_aep.cterm_bg_color;
        new_en.rgb_ae_attr &= (!HL_BG_INDEXED) | (prim_aep.rgb_ae_attr & HL_BG_INDEXED);
    }

    // Override rgb foreground color if primary has one
    if prim_aep.rgb_fg_color >= 0 {
        new_en.rgb_fg_color = prim_aep.rgb_fg_color;
        new_en.rgb_ae_attr &= (!HL_FG_INDEXED) | (prim_aep.rgb_ae_attr & HL_FG_INDEXED);
    }

    // Override rgb background color if primary has one
    if prim_aep.rgb_bg_color >= 0 {
        new_en.rgb_bg_color = prim_aep.rgb_bg_color;
        new_en.rgb_ae_attr &= (!HL_BG_INDEXED) | (prim_aep.rgb_ae_attr & HL_BG_INDEXED);
    }

    // Override special color if primary has one
    if prim_aep.rgb_sp_color >= 0 {
        new_en.rgb_sp_color = prim_aep.rgb_sp_color;
    }

    // Override blend if primary has one
    if prim_aep.hl_blend >= 0 {
        new_en.hl_blend = prim_aep.hl_blend;
    }

    // Inherit URL if char doesn't have one and prim does
    if new_en.url == -1 && prim_aep.url >= 0 {
        new_en.url = prim_aep.url;
    }

    new_en
}

/// Input for blend computation - contains raw and forced colors for both layers
#[repr(C)]
pub struct HlBlendInput {
    /// Back layer raw attributes (before color forcing)
    pub battrs_raw: HlAttrs,
    /// Back layer attributes (with forced colors)
    pub battrs: HlAttrs,
    /// Front layer raw attributes (before color forcing)
    pub fattrs_raw: HlAttrs,
    /// Front layer attributes (with forced colors)
    pub fattrs: HlAttrs,
    /// Blend ratio (0-100)
    pub ratio: c_int,
    /// Whether this is a "through" blend (for floating windows)
    pub through: bool,
}

/// Compute blended highlight attributes.
///
/// This is the pure computation core of hl_blend_attrs. It takes pre-fetched
/// attributes and returns the blended result without any side effects.
///
/// # Arguments
/// * `input` - Contains raw and forced attributes for both layers, ratio, and blend mode
///
/// # Returns
/// The blended HlAttrs
#[no_mangle]
pub extern "C" fn rs_hl_blend_attrs_compute(input: HlBlendInput) -> HlAttrs {
    let HlBlendInput {
        battrs_raw,
        battrs,
        fattrs_raw,
        fattrs,
        ratio,
        through,
    } = input;

    let mut cattrs: HlAttrs;

    if through {
        // "Through" blend: back layer shows through front layer's background
        cattrs = battrs;
        cattrs.rgb_fg_color = rgb_blend_internal(ratio, battrs.rgb_fg_color, fattrs.rgb_bg_color);

        // Only apply special colors when the foreground attribute has an underline or undercurl
        if (fattrs_raw.rgb_ae_attr & (HL_UNDERLINE | HL_UNDERCURL)) != 0 {
            cattrs.rgb_sp_color =
                rgb_blend_internal(ratio, battrs.rgb_sp_color, fattrs.rgb_bg_color);
        } else {
            cattrs.rgb_sp_color = -1;
        }

        cattrs.cterm_bg_color = fattrs.cterm_bg_color;
        cattrs.cterm_fg_color =
            cterm_blend_internal(ratio, battrs.cterm_fg_color, fattrs.cterm_bg_color);
        cattrs.rgb_ae_attr &= !(HL_FG_INDEXED | HL_BG_INDEXED);
    } else {
        // Normal blend: front layer blends with back layer
        cattrs = fattrs;
        cattrs.rgb_fg_color =
            rgb_blend_internal(ratio / 2, battrs.rgb_fg_color, fattrs.rgb_fg_color);

        if (cattrs.rgb_ae_attr & HL_UNDERLINE_MASK) != 0 {
            cattrs.rgb_sp_color =
                rgb_blend_internal(ratio / 2, battrs.rgb_bg_color, fattrs.rgb_sp_color);
        } else {
            cattrs.rgb_sp_color = -1;
        }

        cattrs.rgb_ae_attr &= !HL_BG_INDEXED;
    }

    // Handle background transparency
    // Special case for blend=100: preserve back layer background exactly (including bg=NONE)
    if ratio == 100 && battrs_raw.rgb_bg_color == -1 {
        // For 100% blend with transparent background, preserve the transparency
        cattrs.rgb_bg_color = -1;
    } else {
        // Use the raw attributes (before forcing colors) to check original transparency
        cattrs.rgb_bg_color = if battrs_raw.rgb_bg_color == -1 && fattrs_raw.rgb_bg_color == -1 {
            -1
        } else {
            rgb_blend_internal(ratio, battrs.rgb_bg_color, fattrs.rgb_bg_color)
        };
    }

    // Blend property was consumed
    cattrs.hl_blend = -1;

    cattrs
}

/// Internal RGB blend function (same as rs_rgb_blend but for internal use)
fn rgb_blend_internal(ratio: c_int, rgb1: c_int, rgb2: c_int) -> c_int {
    let a = ratio;
    let b = 100 - ratio;

    let r1 = (rgb1 >> 16) & 0xFF;
    let g1 = (rgb1 >> 8) & 0xFF;
    let b1 = rgb1 & 0xFF;

    let r2 = (rgb2 >> 16) & 0xFF;
    let g2 = (rgb2 >> 8) & 0xFF;
    let b2 = rgb2 & 0xFF;

    let mr = (a * r1 + b * r2) / 100;
    let mg = (a * g1 + b * g2) / 100;
    let mb = (a * b1 + b * b2) / 100;

    (mr << 16) + (mg << 8) + mb
}

/// Internal cterm blend function
fn cterm_blend_internal(ratio: c_int, c1: i16, c2: i16) -> i16 {
    // Convert cterm colors to RGB, blend, then convert back
    let rgb1 = cterm2rgb_internal(c1 as c_int);
    let rgb2 = cterm2rgb_internal(c2 as c_int);
    let blended = rgb_blend_internal(ratio, rgb1, rgb2);
    rgb2cterm_internal(blended)
}

// ============================================================================
// Full Attribute Combination Functions
// ============================================================================

/// Combine two attribute codes to create a third.
///
/// This combines "char_attr" (e.g., from syntax highlighting) with "prim_attr"
/// (e.g., from search highlighting). "prim_attr" overrules "char_attr".
/// This creates a new group when required.
///
/// Since we expect there to be a lot of spelling mistakes we cache the result.
///
/// # Returns
/// The combined attribute ID.
#[export_name = "hl_combine_attr"]
pub unsafe extern "C" fn rs_hl_combine_attr(char_attr: c_int, prim_attr: c_int) -> c_int {
    // Early returns
    if char_attr == 0 {
        return prim_attr;
    }
    if prim_attr == 0 {
        return char_attr;
    }

    // Check cache first
    let combine_tag = (char_attr << 16) + prim_attr;
    let cached = rs_combine_cache_get(combine_tag);
    if cached > 0 {
        return cached;
    }

    // Compute combined attributes
    let char_aep = rs_syn_attr2entry(char_attr);
    let prim_aep = rs_syn_attr2entry(prim_attr);
    let input = HlCombineInput { char_aep, prim_aep };
    let new_en = rs_hl_combine_attrs_compute(input);

    // Get or create entry
    let id = rs_get_attr_entry_full(HlEntry {
        attr: new_en,
        kind: HlKind::Combine,
        id1: char_attr,
        id2: prim_attr,
        winid: 0,
    });

    if id > 0 {
        rs_combine_cache_put(combine_tag, id);
    }

    id
}

/// Blend overlay attributes (for popup menu) with other attributes.
///
/// This creates a new group when required.
/// This is called per-cell, so cache the result.
///
/// # Arguments
/// * `back_attr` - Background attribute
/// * `front_attr` - Foreground (overlay) attribute
/// * `through` - Input/output: whether this is a "through" blend (caller decides, may be cleared)
///
/// # Returns
/// The blended attribute ID.
#[export_name = "hl_blend_attrs"]
pub unsafe extern "C" fn rs_hl_blend_attrs(
    back_attr: c_int,
    front_attr: c_int,
    through: *mut bool,
) -> c_int {
    // Cannot blend uninitialized cells
    if front_attr < 0 || back_attr < 0 {
        return front_attr;
    }

    let fattrs_raw = rs_syn_attr2entry(front_attr);
    let fattrs = rs_get_colors_force(fattrs_raw);
    let ratio = fattrs.hl_blend;

    if ratio <= 0 {
        *through = false;
        return front_attr;
    }

    // Check cache using the through value passed by caller
    let combine_tag = (back_attr << 16) + front_attr;
    let cached = rs_blend_cache_get(combine_tag, *through);
    if cached > 0 {
        return cached;
    }

    // Compute blended attributes
    let battrs_raw = rs_syn_attr2entry(back_attr);
    let battrs = rs_get_colors_force(battrs_raw);
    let input = HlBlendInput {
        battrs_raw,
        battrs,
        fattrs_raw,
        fattrs,
        ratio,
        through: *through,
    };
    let cattrs = rs_hl_blend_attrs_compute(input);

    let kind = if *through {
        HlKind::BlendThrough
    } else {
        HlKind::Blend
    };
    let id = rs_get_attr_entry_full(HlEntry {
        attr: cattrs,
        kind,
        id1: back_attr,
        id2: front_attr,
        winid: 0,
    });

    if id > 0 {
        rs_blend_cache_put(combine_tag, id, *through);
    }

    id
}

/// Get highlight attribute for syntax highlighting.
///
/// Creates a new highlight entry for syntax highlighting with the given attributes.
///
/// # Arguments
/// * `ns_id` - Namespace ID (0 for global)
/// * `idx` - Syntax group index
/// * `at_en` - Highlight attributes
///
/// # Returns
/// The attribute ID, or 0 if all attributes are cleared.
#[export_name = "hl_get_syn_attr"]
pub unsafe extern "C" fn rs_hl_get_syn_attr(ns_id: c_int, idx: c_int, at_en: HlAttrs) -> c_int {
    // Check if any meaningful attribute is set
    if at_en.cterm_fg_color != 0
        || at_en.cterm_bg_color != 0
        || at_en.rgb_fg_color != -1
        || at_en.rgb_bg_color != -1
        || at_en.rgb_sp_color != -1
        || at_en.cterm_ae_attr != 0
        || at_en.rgb_ae_attr != 0
        || ns_id != 0
    {
        return rs_get_attr_entry_full(HlEntry {
            attr: at_en,
            kind: HlKind::Syntax,
            id1: idx,
            id2: ns_id,
            winid: 0,
        });
    }

    // If all fields are cleared, return default
    0
}

/// Add a URL to an existing attribute.
///
/// # Arguments
/// * `attr` - Existing attribute to combine with
/// * `url` - URL string
///
/// # Returns
/// Combined attribute with URL.
#[export_name = "hl_add_url"]
pub unsafe extern "C" fn rs_hl_add_url(attr: c_int, url: *const c_char) -> c_int {
    let mut attrs = HlAttrs::new();

    // Add URL to storage and get index
    let k = rs_hl_add_url_index(url);
    attrs.url = k as i32;

    // Create new entry for the URL attribute
    let new = rs_get_attr_entry_full(HlEntry {
        attr: attrs,
        kind: HlKind::UI,
        id1: 0,
        id2: 0,
        winid: 0,
    });

    // Combine with existing attribute
    rs_hl_combine_attr(attr, new)
}

/// Get attribute code for a builtin highlight group.
///
/// The final syntax group could be modified by hi-link or 'winhighlight'.
///
/// # Arguments
/// * `ns_id` - Namespace ID
/// * `idx` - Highlight field index (HLF_* value)
/// * `final_id` - Final syntax group ID after link resolution
/// * `optional` - If true, return 0 when no attributes are set
///
/// # Returns
/// The attribute ID for the UI highlight.
#[export_name = "hl_get_ui_attr"]
pub unsafe extern "C" fn rs_hl_get_ui_attr(
    ns_id: c_int,
    idx: c_int,
    final_id: c_int,
    optional: bool,
) -> c_int {
    let mut attrs = HlAttrs::new();
    let mut available = false;
    let mut opt = optional;

    if final_id > 0 {
        let syn_attr = rs_syn_ns_id2attr(ns_id, final_id, &mut opt);
        if syn_attr > 0 {
            attrs = rs_syn_attr2entry(syn_attr);
            available = true;
        }
    }

    // Handle popup menu highlights - apply 'pumblend'
    let hlf_pni = nvim_get_hlf_pni();
    let hlf_pst = nvim_get_hlf_pst();
    if hlf_pni <= idx && idx <= hlf_pst {
        let p_pb = nvim_get_p_pb();
        if attrs.hl_blend == -1 && p_pb > 0 {
            attrs.hl_blend = p_pb;
        }
        if nvim_get_pum_drawn() {
            nvim_set_must_redraw_pum(true);
        }
    }

    // Use 'opt' (which may have been modified by rs_syn_ns_id2attr) instead of 'optional'
    if opt && !available {
        return 0;
    }

    rs_get_attr_entry_full(HlEntry {
        attr: attrs,
        kind: HlKind::UI,
        id1: idx,
        id2: final_id,
        winid: 0,
    })
}

/// Get background attribute for a window.
///
/// Returns the appropriate background highlight attribute based on whether
/// the window is current or not, and whether namespace fast mode is active.
///
/// # Arguments
/// * `wp` - Window pointer (opaque c_void)
///
/// # Returns
/// The background attribute ID for the window.
#[export_name = "win_bg_attr"]
pub unsafe extern "C" fn rs_win_bg_attr(wp: *mut c_void) -> c_int {
    let ns_hl_fast = nvim_get_ns_hl_fast();
    let curwin = nvim_get_curwin();
    let hl_attr_active = nvim_get_hl_attr_active();
    let hlf_none = nvim_get_hlf_none();
    let hlf_inactive = nvim_get_hlf_inactive();

    if ns_hl_fast < 0 {
        let local = if wp == curwin {
            nvim_win_get_hl_attr_normal(wp)
        } else {
            nvim_win_get_hl_attr_normalnc(wp)
        };
        if local != 0 {
            return local;
        }
    }

    if wp == curwin || *hl_attr_active.offset(hlf_inactive as isize) == 0 {
        *hl_attr_active.offset(hlf_none as isize)
    } else {
        *hl_attr_active.offset(hlf_inactive as isize)
    }
}

/// Helper: check_blending - updates w_grid_alloc.blending based on winbl and shadow
unsafe fn check_blending(wp: *mut c_void) {
    let winbl = nvim_win_get_p_winbl(wp);
    let floating = nvim_win_get_floating(wp) != 0;
    let shadow = nvim_win_get_config_shadow(wp);
    let blending = winbl > 0 || (floating && shadow);
    nvim_win_set_grid_blending(wp, blending);
}

/// Update window highlights.
///
/// This function updates the highlight attributes for a window based on its
/// namespace highlights and floating window configuration.
#[export_name = "update_window_hl"]
pub unsafe extern "C" fn rs_update_window_hl(wp: *mut c_void, invalid: bool) {
    let ns_id = nvim_win_get_ns_hl(wp);

    // Update namespace highlights
    rs_update_ns_hl(ns_id);

    // Get or update the highlight attribute array for this namespace
    if ns_id != nvim_win_get_ns_hl_active(wp) || nvim_win_get_ns_hl_attr(wp).is_null() {
        nvim_win_set_ns_hl_active(wp, ns_id);

        let hl_attr = rs_ns_hl_attr_get(ns_id);
        if hl_attr.is_null() {
            // No specific highlights, use the defaults
            // Cast const to mut - C code does the same: (int *)rs_ns_hl_attr_get()
            nvim_win_set_ns_hl_attr(wp, nvim_get_highlight_attr() as *mut c_int);
        } else {
            nvim_win_set_ns_hl_attr(wp, hl_attr as *mut c_int);
        }
    }

    let hl_def = nvim_win_get_ns_hl_attr(wp);

    // Early return if no update needed
    if !nvim_win_get_hl_needs_update(wp) && !invalid {
        return;
    }
    nvim_win_set_hl_needs_update(wp, false);

    // Get HLF constants
    let hlf_nfloat = nvim_get_hlf_nfloat();
    let hlf_none = nvim_get_hlf_none();
    let hlf_inactive = nvim_get_hlf_inactive();
    let hlf_border = nvim_get_hlf_border();

    // If a floating window is blending it always has a named
    // wp->w_hl_attr_normal group. HL_ATTR(HLF_NFLOAT) is always named.
    let floating = nvim_win_get_floating(wp) != 0;
    let external = nvim_win_get_config_external(wp);
    let float_win = floating && !external;

    let hl_attr_active = nvim_get_hl_attr_active();
    let highlight_attr = nvim_get_highlight_attr();

    // Determine window specific background set in 'winhighlight'
    let hl_attr_normal;
    if float_win && *hl_def.offset(hlf_nfloat as isize) != 0 && ns_id > 0 {
        hl_attr_normal = *hl_def.offset(hlf_nfloat as isize);
    } else if *hl_def.offset(hlf_none as isize) > 0 {
        hl_attr_normal = *hl_def.offset(hlf_none as isize);
    } else if float_win {
        let hl_nfloat = *hl_attr_active.offset(hlf_nfloat as isize);
        hl_attr_normal = if hl_nfloat > 0 {
            hl_nfloat
        } else {
            *highlight_attr.offset(hlf_nfloat as isize)
        };
    } else {
        hl_attr_normal = 0;
    }

    let winbl = nvim_win_get_p_winbl(wp);
    let hl_attr_normal = if floating {
        rs_hl_apply_winblend(winbl, hl_attr_normal)
    } else {
        hl_attr_normal
    };
    nvim_win_set_hl_attr_normal(wp, hl_attr_normal);

    // Handle border highlights for floating windows
    nvim_win_set_config_shadow(wp, false);
    if floating && nvim_win_get_config_border(wp) {
        for i in 0..8 {
            let mut attr = *hl_def.offset(hlf_border as isize);
            let border_hl_id = nvim_win_get_config_border_hl_id(wp, i);
            if border_hl_id != 0 {
                attr = rs_hl_get_ui_attr(ns_id, hlf_border, border_hl_id, false);
            }
            attr = rs_hl_apply_winblend(winbl, attr);
            // Check if this attr has blend > 0
            let entry = rs_get_attr_entry_by_id(attr);
            if entry.attr.hl_blend > 0 {
                nvim_win_set_config_shadow(wp, true);
            }
            nvim_win_set_config_border_attr(wp, i, attr);
        }
    }

    // Shadow might cause blending
    check_blending(wp);

    // Compute normalnc (non-current window normal)
    let hl_attr_normalnc = if *hl_def.offset(hlf_inactive as isize) == 0 {
        rs_hl_combine_attr(
            *hl_attr_active.offset(hlf_inactive as isize),
            hl_attr_normal,
        )
    } else {
        *hl_def.offset(hlf_inactive as isize)
    };

    let hl_attr_normalnc = if floating {
        rs_hl_apply_winblend(winbl, hl_attr_normalnc)
    } else {
        hl_attr_normalnc
    };
    nvim_win_set_hl_attr_normalnc(wp, hl_attr_normalnc);
}

/// Internal cterm to RGB conversion
fn cterm2rgb_internal(nr: c_int) -> c_int {
    if nr < 16 {
        let entry = &ANSI_TABLE[nr as usize];
        return (c_int::from(entry[0]) << 16)
            | (c_int::from(entry[1]) << 8)
            | c_int::from(entry[2]);
    }

    if nr < 232 {
        let idx = (nr - 16) as usize;
        let r = CUBE_VALUE[idx / 36];
        let g = CUBE_VALUE[(idx / 6) % 6];
        let b = CUBE_VALUE[idx % 6];
        return (r << 16) | (g << 8) | b;
    }

    let grey = GREY_RAMP[(nr - 232) as usize];
    (grey << 16) | (grey << 8) | grey
}

/// Internal RGB to cterm conversion
///
/// Matches C's hl_rgb2cterm_color: simple 6x6x6 cube mapping without grey detection.
/// Returns 0-215 (cube indices without the +16 offset).
fn rgb2cterm_internal(rgb: c_int) -> i16 {
    let r = (rgb >> 16) & 0xFF;
    let g = (rgb >> 8) & 0xFF;
    let b = rgb & 0xFF;

    // Simple cube mapping: (r * 6 / 256) * 36 + (g * 6 / 256) * 6 + (b * 6 / 256)
    // This matches C's hl_rgb2cterm_color exactly
    ((r * 6 / 256) * 36 + (g * 6 / 256) * 6 + (b * 6 / 256)) as i16
}

// ============================================================================
// Color Blending
// ============================================================================

/// Blend two RGB colors together based on a ratio.
///
/// # Arguments
/// * `ratio` - Blend ratio (0-100). 100 means full rgb1, 0 means full rgb2.
/// * `rgb1` - First RGB color (0xRRGGBB format)
/// * `rgb2` - Second RGB color (0xRRGGBB format)
///
/// # Returns
/// Blended RGB color in 0xRRGGBB format
#[no_mangle]
pub extern "C" fn rs_rgb_blend(ratio: c_int, rgb1: c_int, rgb2: c_int) -> c_int {
    let a = ratio;
    let b = 100 - ratio;

    let r1 = (rgb1 >> 16) & 0xFF;
    let g1 = (rgb1 >> 8) & 0xFF;
    let b1 = rgb1 & 0xFF;

    let r2 = (rgb2 >> 16) & 0xFF;
    let g2 = (rgb2 >> 8) & 0xFF;
    let b2 = rgb2 & 0xFF;

    let mr = (a * r1 + b * r2) / 100;
    let mg = (a * g1 + b * g2) / 100;
    let mb = (a * b1 + b * b2) / 100;

    (mr << 16) + (mg << 8) + mb
}

// ============================================================================
// Color Conversion Tables
// ============================================================================

/// xterm 6x6x6 color cube values
const CUBE_VALUE: [c_int; 6] = [0x00, 0x5F, 0x87, 0xAF, 0xD7, 0xFF];

/// xterm grey ramp values (colors 232-255)
const GREY_RAMP: [c_int; 24] = [
    0x08, 0x12, 0x1C, 0x26, 0x30, 0x3A, 0x44, 0x4E, 0x58, 0x62, 0x6C, 0x76, 0x80, 0x8A, 0x94, 0x9E,
    0xA8, 0xB2, 0xBC, 0xC6, 0xD0, 0xDA, 0xE4, 0xEE,
];

/// ANSI 16-color table: [R, G, B, idx]
const ANSI_TABLE: [[u8; 4]; 16] = [
    [0, 0, 0, 1],        // black
    [224, 0, 0, 2],      // dark red
    [0, 224, 0, 3],      // dark green
    [224, 224, 0, 4],    // dark yellow / brown
    [0, 0, 224, 5],      // dark blue
    [224, 0, 224, 6],    // dark magenta
    [0, 224, 224, 7],    // dark cyan
    [224, 224, 224, 8],  // light grey
    [128, 128, 128, 9],  // dark grey
    [255, 64, 64, 10],   // light red
    [64, 255, 64, 11],   // light green
    [255, 255, 64, 12],  // light yellow
    [64, 64, 255, 13],   // light blue
    [255, 64, 255, 14],  // light magenta
    [64, 255, 255, 15],  // light cyan
    [255, 255, 255, 16], // white
];

/// Convert 8-bit color (0-255) to RGB color.
/// This is compatible with xterm.
///
/// # Arguments
/// * `nr` - 8-bit color number (0-255)
///
/// # Returns
/// RGB color in 0xRRGGBB format
#[no_mangle]
pub extern "C" fn rs_hl_cterm2rgb_color(nr: c_int) -> c_int {
    if nr < 16 {
        // ANSI colors
        let entry = &ANSI_TABLE[nr as usize];
        return (c_int::from(entry[0]) << 16)
            | (c_int::from(entry[1]) << 8)
            | c_int::from(entry[2]);
    }

    if nr < 232 {
        // 6x6x6 color cube (colors 16-231)
        let idx = nr - 16;
        let r_idx = idx / 36;
        let g_idx = (idx % 36) / 6;
        let b_idx = idx % 6;
        let r = CUBE_VALUE[r_idx as usize];
        let g = CUBE_VALUE[g_idx as usize];
        let b = CUBE_VALUE[b_idx as usize];
        return (r << 16) | (g << 8) | b;
    }

    // Grey ramp (colors 232-255)
    let grey = GREY_RAMP[(nr - 232) as usize];
    (grey << 16) | (grey << 8) | grey
}

/// Convert RGB color to 8-bit color (0-255).
/// Uses the 6x6x6 color cube portion of the xterm 256-color palette.
///
/// # Arguments
/// * `rgb` - RGB color in 0xRRGGBB format
///
/// # Returns
/// 8-bit color number (16-231, the color cube range)
#[no_mangle]
pub extern "C" fn rs_hl_rgb2cterm_color(rgb: c_int) -> c_int {
    let r = (rgb >> 16) & 0xFF;
    let g = (rgb >> 8) & 0xFF;
    let b = rgb & 0xFF;

    // Map to 6x6x6 cube indices and add offset 16
    (r * 6 / 256) * 36 + (g * 6 / 256) * 6 + (b * 6 / 256) + 16
}

/// Blend two cterm colors together based on a ratio.
///
/// 1. Converts cterm color numbers to RGB.
/// 2. Blends the RGB colors.
/// 3. Converts the RGB result back to a cterm color.
///
/// # Arguments
/// * `ratio` - Blend ratio (0-100). 100 means full c1, 0 means full c2.
/// * `c1` - First cterm color (0-255)
/// * `c2` - Second cterm color (0-255)
///
/// # Returns
/// Blended cterm color number
#[no_mangle]
pub extern "C" fn rs_cterm_blend(ratio: c_int, c1: i16, c2: i16) -> c_int {
    let rgb1 = rs_hl_cterm2rgb_color(c_int::from(c1));
    let rgb2 = rs_hl_cterm2rgb_color(c_int::from(c2));
    let rgb_blended = rs_rgb_blend(ratio, rgb1, rgb2);
    rs_hl_rgb2cterm_color(rgb_blended)
}

// ============================================================================
// Color Name Lookup
// ============================================================================

/// Color names for terminal colors (28 entries)
const COLOR_NAMES: [&str; 28] = [
    "Black",
    "DarkBlue",
    "DarkGreen",
    "DarkCyan",
    "DarkRed",
    "DarkMagenta",
    "Brown",
    "DarkYellow",
    "Gray",
    "Grey",
    "LightGray",
    "LightGrey",
    "DarkGray",
    "DarkGrey",
    "Blue",
    "LightBlue",
    "Green",
    "LightGreen",
    "Cyan",
    "LightCyan",
    "Red",
    "LightRed",
    "Magenta",
    "LightMagenta",
    "Yellow",
    "LightYellow",
    "White",
    "NONE",
];

/// Color numbers for 16-color terminals
const COLOR_NUMBERS_16: [c_int; 28] = [
    0, 1, 2, 3, 4, 5, 6, 6, 7, 7, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13, 14, 14, 15, -1,
];

/// Color numbers for 8-color terminals
const COLOR_NUMBERS_8: [c_int; 28] = [
    0, 4, 2, 6, 1, 5, 3, 3, 7, 7, 7, 7, 8, 8, 12, 12, 10, 10, 14, 14, 9, 9, 13, 13, 11, 11, 15, -1,
];

/// Color numbers for xterm with 88 colors
const COLOR_NUMBERS_88: [c_int; 28] = [
    0, 4, 2, 6, 1, 5, 32, 72, 84, 84, 7, 7, 82, 82, 12, 43, 10, 61, 14, 63, 9, 74, 13, 75, 11, 78,
    15, -1,
];

/// Color numbers for xterm with 256 colors
const COLOR_NUMBERS_256: [c_int; 28] = [
    0, 4, 2, 6, 1, 5, 130, 3, 248, 248, 7, 7, 242, 242, 12, 81, 10, 121, 14, 159, 9, 224, 13, 225,
    11, 229, 15, -1,
];

/// Lookup the "cterm" value for a color index based on terminal color count.
fn lookup_color(idx: usize, t_colors: c_int) -> c_int {
    // Use the _16 table to check if it's a valid color name.
    let color = COLOR_NUMBERS_16[idx];
    if color < 0 {
        return -1;
    }

    // Select appropriate color table based on terminal color count
    if t_colors == 8 {
        COLOR_NUMBERS_8[idx] & 7 // truncate to 8 colors
    } else if t_colors == 16 {
        COLOR_NUMBERS_8[idx]
    } else if t_colors == 88 {
        COLOR_NUMBERS_88[idx]
    } else if t_colors >= 256 {
        COLOR_NUMBERS_256[idx]
    } else {
        color
    }
}

/// Result of lookup_color with bold state
#[repr(C)]
pub struct LookupColorResult {
    /// The cterm color number (-1 if invalid)
    pub color: c_int,
    /// Bold state: -1 = unchanged, 0 = false, 1 = true
    pub bold: c_int,
}

/// Lookup the "cterm" value for a color index.
///
/// This is the FFI wrapper that handles the bold attribute for foreground
/// colors on 8-color terminals.
///
/// # Arguments
/// * `idx` - Index into color_names array (0-27)
/// * `foreground` - Whether this is a foreground color lookup
///
/// # Returns
/// LookupColorResult with color number and bold state.
/// - bold == -1: unchanged (don't modify boldp)
/// - bold == 0: set to kFalse
/// - bold == 1: set to kTrue
///
/// # Safety
/// Calls C accessor functions for global variables.
#[no_mangle]
pub unsafe extern "C" fn rs_lookup_color(idx: c_int, foreground: bool) -> LookupColorResult {
    // Bounds check
    if !(0..28).contains(&idx) {
        return LookupColorResult {
            color: -1,
            bold: -1,
        };
    }
    let idx = idx as usize;

    let t_colors = nvim_get_t_colors();

    // Use the _16 table to check if it's a valid color name.
    let color_16 = COLOR_NUMBERS_16[idx];
    if color_16 < 0 {
        return LookupColorResult {
            color: -1,
            bold: -1,
        };
    }

    if t_colors == 8 {
        // t_Co is 8: use the 8 colors table
        let color = COLOR_NUMBERS_8[idx];
        let bold = if foreground {
            // set/reset bold attribute to get light foreground
            // colors (on some terminals, e.g. "linux")
            if color & 8 != 0 {
                1
            } else {
                0
            }
        } else {
            -1 // unchanged
        };
        LookupColorResult {
            color: color & 7, // truncate to 8 colors
            bold,
        }
    } else if t_colors == 16 {
        LookupColorResult {
            color: COLOR_NUMBERS_8[idx],
            bold: -1,
        }
    } else if t_colors == 88 {
        LookupColorResult {
            color: COLOR_NUMBERS_88[idx],
            bold: -1,
        }
    } else if t_colors >= 256 {
        LookupColorResult {
            color: COLOR_NUMBERS_256[idx],
            bold: -1,
        }
    } else {
        LookupColorResult {
            color: color_16,
            bold: -1,
        }
    }
}

/// Case-insensitive comparison for ASCII strings
fn str_icmp(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .all(|(c1, c2)| c1.eq_ignore_ascii_case(&c2))
}

/// Convert a color name to its cterm color number.
///
/// # Arguments
/// * `name` - Color name (null-terminated C string)
///
/// # Returns
/// cterm color number, or -1 if not found
///
/// # Safety
/// `name` must be a valid null-terminated C string.
#[export_name = "name_to_ctermcolor"]
pub unsafe extern "C" fn rs_name_to_ctermcolor(name: *const c_char) -> c_int {
    if name.is_null() {
        return -1;
    }

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    if name_str.is_empty() {
        return -1;
    }

    // Find matching color name (case-insensitive)
    for (idx, color_name) in COLOR_NAMES.iter().enumerate().rev() {
        if str_icmp(name_str, color_name) {
            let t_colors = unsafe { nvim_get_t_colors() };
            return lookup_color(idx, t_colors);
        }
    }

    -1
}

// ============================================================================
// RGB Color Name Table (from rgb.txt)
// ============================================================================

/// Special color index values
const COLOR_IDX_NONE: c_int = -1;
const COLOR_IDX_HEX: c_int = -2;
const COLOR_IDX_FG: c_int = -3;
const COLOR_IDX_BG: c_int = -4;

// Color name table (moved to COLOR_NAME_TABLE below for C FFI export)

/// Result type for name_to_color function
#[repr(C)]
pub struct NameToColorResult {
    /// RGB color value, or -1 if not found
    pub color: c_int,
    /// Index into color table, or special constant (COLOR_IDX_*)
    pub idx: c_int,
}

/// Check if a character is a hex digit
fn is_hex_digit(c: u8) -> bool {
    c.is_ascii_hexdigit()
}

/// Binary search for a color name in the table (case-insensitive)
fn find_color_name(name: &str) -> Option<(usize, c_int)> {
    // Search the exported COLOR_NAME_TABLE (excludes the null sentinel)
    let table = &COLOR_NAME_TABLE[..COLOR_NAME_TABLE.len() - 1];
    let mut lo = 0usize;
    let mut hi = table.len();

    while lo < hi {
        let mid = (lo + hi) / 2;
        // SAFETY: COLOR_NAME_TABLE entries have static lifetime name pointers
        let entry_name = unsafe { CStr::from_ptr(table[mid].name).to_str().unwrap_or("") };
        let color = table[mid].color;
        match name
            .to_ascii_lowercase()
            .cmp(&entry_name.to_ascii_lowercase())
        {
            std::cmp::Ordering::Less => hi = mid,
            std::cmp::Ordering::Greater => lo = mid + 1,
            std::cmp::Ordering::Equal => return Some((mid, color)),
        }
    }
    None
}

/// Convert a color name to its RGB value.
///
/// Handles:
/// - Hex strings like "#RRGGBB"
/// - Special names "fg", "foreground", "bg", "background"
/// - Named colors from rgb.txt (707 entries)
///
/// # Arguments
/// * `name` - Color name (null-terminated C string)
///
/// # Returns
/// NameToColorResult with color value and index
///
/// # Safety
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_name_to_color(name: *const c_char) -> NameToColorResult {
    if name.is_null() {
        return NameToColorResult {
            color: -1,
            idx: COLOR_IDX_NONE,
        };
    }

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_bytes = name_cstr.to_bytes();

    // Check for hex color string "#RRGGBB"
    if name_bytes.len() == 7
        && name_bytes[0] == b'#'
        && is_hex_digit(name_bytes[1])
        && is_hex_digit(name_bytes[2])
        && is_hex_digit(name_bytes[3])
        && is_hex_digit(name_bytes[4])
        && is_hex_digit(name_bytes[5])
        && is_hex_digit(name_bytes[6])
    {
        // Parse hex value
        let hex_str = unsafe { std::str::from_utf8_unchecked(&name_bytes[1..7]) };
        if let Ok(color) = i32::from_str_radix(hex_str, 16) {
            return NameToColorResult {
                color,
                idx: COLOR_IDX_HEX,
            };
        }
    }

    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => {
            return NameToColorResult {
                color: -1,
                idx: COLOR_IDX_NONE,
            }
        }
    };

    // Check for "bg" or "background"
    if name_str.eq_ignore_ascii_case("bg") || name_str.eq_ignore_ascii_case("background") {
        let color = unsafe { nvim_get_normal_bg() };
        return NameToColorResult {
            color,
            idx: COLOR_IDX_BG,
        };
    }

    // Check for "fg" or "foreground"
    if name_str.eq_ignore_ascii_case("fg") || name_str.eq_ignore_ascii_case("foreground") {
        let color = unsafe { nvim_get_normal_fg() };
        return NameToColorResult {
            color,
            idx: COLOR_IDX_FG,
        };
    }

    // Binary search in color table
    if let Some((idx, color)) = find_color_name(name_str) {
        return NameToColorResult {
            color,
            idx: idx as c_int,
        };
    }

    NameToColorResult {
        color: -1,
        idx: COLOR_IDX_NONE,
    }
}

/// Translate to RgbValue if name is a hex value or named color.
/// Directly provides the C symbol `name_to_color`, replacing the thin wrapper.
///
/// # Safety
/// `name` must be a valid null-terminated C string.
/// `idx` must be a valid pointer to an int.
#[export_name = "name_to_color"]
pub unsafe extern "C" fn rs_name_to_color_adapted(name: *const c_char, idx: *mut c_int) -> c_int {
    let result = rs_name_to_color(name);
    *idx = result.idx;
    result.color
}

// ============================================================================
// Unified Color Name Table (exported as C symbol `color_name_table`)
// ============================================================================

/// `#[repr(C)]` entry matching `color_name_table_T` in C.
#[repr(C)]
pub struct ColorNameEntry {
    pub name: *const c_char,
    pub color: c_int,
}

unsafe impl Sync for ColorNameEntry {}
unsafe impl Send for ColorNameEntry {}

/// Color name table exported as C symbol `color_name_table`.
/// Terminated by a null-name sentinel entry.
#[export_name = "color_name_table"]
pub static COLOR_NAME_TABLE: [ColorNameEntry; 708] = [
    ColorNameEntry {
        name: c"AliceBlue".as_ptr(),
        color: 0xf0f8ff,
    },
    ColorNameEntry {
        name: c"AntiqueWhite".as_ptr(),
        color: 0xfaebd7,
    },
    ColorNameEntry {
        name: c"AntiqueWhite1".as_ptr(),
        color: 0xffefdb,
    },
    ColorNameEntry {
        name: c"AntiqueWhite2".as_ptr(),
        color: 0xeedfcc,
    },
    ColorNameEntry {
        name: c"AntiqueWhite3".as_ptr(),
        color: 0xcdc0b0,
    },
    ColorNameEntry {
        name: c"AntiqueWhite4".as_ptr(),
        color: 0x8b8378,
    },
    ColorNameEntry {
        name: c"Aqua".as_ptr(),
        color: 0x00ffff,
    },
    ColorNameEntry {
        name: c"Aquamarine".as_ptr(),
        color: 0x7fffd4,
    },
    ColorNameEntry {
        name: c"Aquamarine1".as_ptr(),
        color: 0x7fffd4,
    },
    ColorNameEntry {
        name: c"Aquamarine2".as_ptr(),
        color: 0x76eec6,
    },
    ColorNameEntry {
        name: c"Aquamarine3".as_ptr(),
        color: 0x66cdaa,
    },
    ColorNameEntry {
        name: c"Aquamarine4".as_ptr(),
        color: 0x458b74,
    },
    ColorNameEntry {
        name: c"Azure".as_ptr(),
        color: 0xf0ffff,
    },
    ColorNameEntry {
        name: c"Azure1".as_ptr(),
        color: 0xf0ffff,
    },
    ColorNameEntry {
        name: c"Azure2".as_ptr(),
        color: 0xe0eeee,
    },
    ColorNameEntry {
        name: c"Azure3".as_ptr(),
        color: 0xc1cdcd,
    },
    ColorNameEntry {
        name: c"Azure4".as_ptr(),
        color: 0x838b8b,
    },
    ColorNameEntry {
        name: c"Beige".as_ptr(),
        color: 0xf5f5dc,
    },
    ColorNameEntry {
        name: c"Bisque".as_ptr(),
        color: 0xffe4c4,
    },
    ColorNameEntry {
        name: c"Bisque1".as_ptr(),
        color: 0xffe4c4,
    },
    ColorNameEntry {
        name: c"Bisque2".as_ptr(),
        color: 0xeed5b7,
    },
    ColorNameEntry {
        name: c"Bisque3".as_ptr(),
        color: 0xcdb79e,
    },
    ColorNameEntry {
        name: c"Bisque4".as_ptr(),
        color: 0x8b7d6b,
    },
    ColorNameEntry {
        name: c"Black".as_ptr(),
        color: 0x000000,
    },
    ColorNameEntry {
        name: c"BlanchedAlmond".as_ptr(),
        color: 0xffebcd,
    },
    ColorNameEntry {
        name: c"Blue".as_ptr(),
        color: 0x0000ff,
    },
    ColorNameEntry {
        name: c"Blue1".as_ptr(),
        color: 0x0000ff,
    },
    ColorNameEntry {
        name: c"Blue2".as_ptr(),
        color: 0x0000ee,
    },
    ColorNameEntry {
        name: c"Blue3".as_ptr(),
        color: 0x0000cd,
    },
    ColorNameEntry {
        name: c"Blue4".as_ptr(),
        color: 0x00008b,
    },
    ColorNameEntry {
        name: c"BlueViolet".as_ptr(),
        color: 0x8a2be2,
    },
    ColorNameEntry {
        name: c"Brown".as_ptr(),
        color: 0xa52a2a,
    },
    ColorNameEntry {
        name: c"Brown1".as_ptr(),
        color: 0xff4040,
    },
    ColorNameEntry {
        name: c"Brown2".as_ptr(),
        color: 0xee3b3b,
    },
    ColorNameEntry {
        name: c"Brown3".as_ptr(),
        color: 0xcd3333,
    },
    ColorNameEntry {
        name: c"Brown4".as_ptr(),
        color: 0x8b2323,
    },
    ColorNameEntry {
        name: c"BurlyWood".as_ptr(),
        color: 0xdeb887,
    },
    ColorNameEntry {
        name: c"Burlywood1".as_ptr(),
        color: 0xffd39b,
    },
    ColorNameEntry {
        name: c"Burlywood2".as_ptr(),
        color: 0xeec591,
    },
    ColorNameEntry {
        name: c"Burlywood3".as_ptr(),
        color: 0xcdaa7d,
    },
    ColorNameEntry {
        name: c"Burlywood4".as_ptr(),
        color: 0x8b7355,
    },
    ColorNameEntry {
        name: c"CadetBlue".as_ptr(),
        color: 0x5f9ea0,
    },
    ColorNameEntry {
        name: c"CadetBlue1".as_ptr(),
        color: 0x98f5ff,
    },
    ColorNameEntry {
        name: c"CadetBlue2".as_ptr(),
        color: 0x8ee5ee,
    },
    ColorNameEntry {
        name: c"CadetBlue3".as_ptr(),
        color: 0x7ac5cd,
    },
    ColorNameEntry {
        name: c"CadetBlue4".as_ptr(),
        color: 0x53868b,
    },
    ColorNameEntry {
        name: c"ChartReuse".as_ptr(),
        color: 0x7fff00,
    },
    ColorNameEntry {
        name: c"Chartreuse1".as_ptr(),
        color: 0x7fff00,
    },
    ColorNameEntry {
        name: c"Chartreuse2".as_ptr(),
        color: 0x76ee00,
    },
    ColorNameEntry {
        name: c"Chartreuse3".as_ptr(),
        color: 0x66cd00,
    },
    ColorNameEntry {
        name: c"Chartreuse4".as_ptr(),
        color: 0x458b00,
    },
    ColorNameEntry {
        name: c"Chocolate".as_ptr(),
        color: 0xd2691e,
    },
    ColorNameEntry {
        name: c"Chocolate1".as_ptr(),
        color: 0xff7f24,
    },
    ColorNameEntry {
        name: c"Chocolate2".as_ptr(),
        color: 0xee7621,
    },
    ColorNameEntry {
        name: c"Chocolate3".as_ptr(),
        color: 0xcd661d,
    },
    ColorNameEntry {
        name: c"Chocolate4".as_ptr(),
        color: 0x8b4513,
    },
    ColorNameEntry {
        name: c"Coral".as_ptr(),
        color: 0xff7f50,
    },
    ColorNameEntry {
        name: c"Coral1".as_ptr(),
        color: 0xff7256,
    },
    ColorNameEntry {
        name: c"Coral2".as_ptr(),
        color: 0xee6a50,
    },
    ColorNameEntry {
        name: c"Coral3".as_ptr(),
        color: 0xcd5b45,
    },
    ColorNameEntry {
        name: c"Coral4".as_ptr(),
        color: 0x8b3e2f,
    },
    ColorNameEntry {
        name: c"CornFlowerBlue".as_ptr(),
        color: 0x6495ed,
    },
    ColorNameEntry {
        name: c"Cornsilk".as_ptr(),
        color: 0xfff8dc,
    },
    ColorNameEntry {
        name: c"Cornsilk1".as_ptr(),
        color: 0xfff8dc,
    },
    ColorNameEntry {
        name: c"Cornsilk2".as_ptr(),
        color: 0xeee8cd,
    },
    ColorNameEntry {
        name: c"Cornsilk3".as_ptr(),
        color: 0xcdc8b1,
    },
    ColorNameEntry {
        name: c"Cornsilk4".as_ptr(),
        color: 0x8b8878,
    },
    ColorNameEntry {
        name: c"Crimson".as_ptr(),
        color: 0xdc143c,
    },
    ColorNameEntry {
        name: c"Cyan".as_ptr(),
        color: 0x00ffff,
    },
    ColorNameEntry {
        name: c"Cyan1".as_ptr(),
        color: 0x00ffff,
    },
    ColorNameEntry {
        name: c"Cyan2".as_ptr(),
        color: 0x00eeee,
    },
    ColorNameEntry {
        name: c"Cyan3".as_ptr(),
        color: 0x00cdcd,
    },
    ColorNameEntry {
        name: c"Cyan4".as_ptr(),
        color: 0x008b8b,
    },
    ColorNameEntry {
        name: c"DarkBlue".as_ptr(),
        color: 0x00008b,
    },
    ColorNameEntry {
        name: c"DarkCyan".as_ptr(),
        color: 0x008b8b,
    },
    ColorNameEntry {
        name: c"DarkGoldenrod".as_ptr(),
        color: 0xb8860b,
    },
    ColorNameEntry {
        name: c"DarkGoldenrod1".as_ptr(),
        color: 0xffb90f,
    },
    ColorNameEntry {
        name: c"DarkGoldenrod2".as_ptr(),
        color: 0xeead0e,
    },
    ColorNameEntry {
        name: c"DarkGoldenrod3".as_ptr(),
        color: 0xcd950c,
    },
    ColorNameEntry {
        name: c"DarkGoldenrod4".as_ptr(),
        color: 0x8b6508,
    },
    ColorNameEntry {
        name: c"DarkGray".as_ptr(),
        color: 0xa9a9a9,
    },
    ColorNameEntry {
        name: c"DarkGreen".as_ptr(),
        color: 0x006400,
    },
    ColorNameEntry {
        name: c"DarkGrey".as_ptr(),
        color: 0xa9a9a9,
    },
    ColorNameEntry {
        name: c"DarkKhaki".as_ptr(),
        color: 0xbdb76b,
    },
    ColorNameEntry {
        name: c"DarkMagenta".as_ptr(),
        color: 0x8b008b,
    },
    ColorNameEntry {
        name: c"DarkOliveGreen".as_ptr(),
        color: 0x556b2f,
    },
    ColorNameEntry {
        name: c"DarkOliveGreen1".as_ptr(),
        color: 0xcaff70,
    },
    ColorNameEntry {
        name: c"DarkOliveGreen2".as_ptr(),
        color: 0xbcee68,
    },
    ColorNameEntry {
        name: c"DarkOliveGreen3".as_ptr(),
        color: 0xa2cd5a,
    },
    ColorNameEntry {
        name: c"DarkOliveGreen4".as_ptr(),
        color: 0x6e8b3d,
    },
    ColorNameEntry {
        name: c"DarkOrange".as_ptr(),
        color: 0xff8c00,
    },
    ColorNameEntry {
        name: c"DarkOrange1".as_ptr(),
        color: 0xff7f00,
    },
    ColorNameEntry {
        name: c"DarkOrange2".as_ptr(),
        color: 0xee7600,
    },
    ColorNameEntry {
        name: c"DarkOrange3".as_ptr(),
        color: 0xcd6600,
    },
    ColorNameEntry {
        name: c"DarkOrange4".as_ptr(),
        color: 0x8b4500,
    },
    ColorNameEntry {
        name: c"DarkOrchid".as_ptr(),
        color: 0x9932cc,
    },
    ColorNameEntry {
        name: c"DarkOrchid1".as_ptr(),
        color: 0xbf3eff,
    },
    ColorNameEntry {
        name: c"DarkOrchid2".as_ptr(),
        color: 0xb23aee,
    },
    ColorNameEntry {
        name: c"DarkOrchid3".as_ptr(),
        color: 0x9a32cd,
    },
    ColorNameEntry {
        name: c"DarkOrchid4".as_ptr(),
        color: 0x68228b,
    },
    ColorNameEntry {
        name: c"DarkRed".as_ptr(),
        color: 0x8b0000,
    },
    ColorNameEntry {
        name: c"DarkSalmon".as_ptr(),
        color: 0xe9967a,
    },
    ColorNameEntry {
        name: c"DarkSeaGreen".as_ptr(),
        color: 0x8fbc8f,
    },
    ColorNameEntry {
        name: c"DarkSeaGreen1".as_ptr(),
        color: 0xc1ffc1,
    },
    ColorNameEntry {
        name: c"DarkSeaGreen2".as_ptr(),
        color: 0xb4eeb4,
    },
    ColorNameEntry {
        name: c"DarkSeaGreen3".as_ptr(),
        color: 0x9bcd9b,
    },
    ColorNameEntry {
        name: c"DarkSeaGreen4".as_ptr(),
        color: 0x698b69,
    },
    ColorNameEntry {
        name: c"DarkSlateBlue".as_ptr(),
        color: 0x483d8b,
    },
    ColorNameEntry {
        name: c"DarkSlateGray".as_ptr(),
        color: 0x2f4f4f,
    },
    ColorNameEntry {
        name: c"DarkSlateGray1".as_ptr(),
        color: 0x97ffff,
    },
    ColorNameEntry {
        name: c"DarkSlateGray2".as_ptr(),
        color: 0x8deeee,
    },
    ColorNameEntry {
        name: c"DarkSlateGray3".as_ptr(),
        color: 0x79cdcd,
    },
    ColorNameEntry {
        name: c"DarkSlateGray4".as_ptr(),
        color: 0x528b8b,
    },
    ColorNameEntry {
        name: c"DarkSlateGrey".as_ptr(),
        color: 0x2f4f4f,
    },
    ColorNameEntry {
        name: c"DarkTurquoise".as_ptr(),
        color: 0x00ced1,
    },
    ColorNameEntry {
        name: c"DarkViolet".as_ptr(),
        color: 0x9400d3,
    },
    ColorNameEntry {
        name: c"DarkYellow".as_ptr(),
        color: 0xbbbb00,
    },
    ColorNameEntry {
        name: c"DeepPink".as_ptr(),
        color: 0xff1493,
    },
    ColorNameEntry {
        name: c"DeepPink1".as_ptr(),
        color: 0xff1493,
    },
    ColorNameEntry {
        name: c"DeepPink2".as_ptr(),
        color: 0xee1289,
    },
    ColorNameEntry {
        name: c"DeepPink3".as_ptr(),
        color: 0xcd1076,
    },
    ColorNameEntry {
        name: c"DeepPink4".as_ptr(),
        color: 0x8b0a50,
    },
    ColorNameEntry {
        name: c"DeepSkyBlue".as_ptr(),
        color: 0x00bfff,
    },
    ColorNameEntry {
        name: c"DeepSkyBlue1".as_ptr(),
        color: 0x00bfff,
    },
    ColorNameEntry {
        name: c"DeepSkyBlue2".as_ptr(),
        color: 0x00b2ee,
    },
    ColorNameEntry {
        name: c"DeepSkyBlue3".as_ptr(),
        color: 0x009acd,
    },
    ColorNameEntry {
        name: c"DeepSkyBlue4".as_ptr(),
        color: 0x00688b,
    },
    ColorNameEntry {
        name: c"DimGray".as_ptr(),
        color: 0x696969,
    },
    ColorNameEntry {
        name: c"DimGrey".as_ptr(),
        color: 0x696969,
    },
    ColorNameEntry {
        name: c"DodgerBlue".as_ptr(),
        color: 0x1e90ff,
    },
    ColorNameEntry {
        name: c"DodgerBlue1".as_ptr(),
        color: 0x1e90ff,
    },
    ColorNameEntry {
        name: c"DodgerBlue2".as_ptr(),
        color: 0x1c86ee,
    },
    ColorNameEntry {
        name: c"DodgerBlue3".as_ptr(),
        color: 0x1874cd,
    },
    ColorNameEntry {
        name: c"DodgerBlue4".as_ptr(),
        color: 0x104e8b,
    },
    ColorNameEntry {
        name: c"Firebrick".as_ptr(),
        color: 0xb22222,
    },
    ColorNameEntry {
        name: c"Firebrick1".as_ptr(),
        color: 0xff3030,
    },
    ColorNameEntry {
        name: c"Firebrick2".as_ptr(),
        color: 0xee2c2c,
    },
    ColorNameEntry {
        name: c"Firebrick3".as_ptr(),
        color: 0xcd2626,
    },
    ColorNameEntry {
        name: c"Firebrick4".as_ptr(),
        color: 0x8b1a1a,
    },
    ColorNameEntry {
        name: c"FloralWhite".as_ptr(),
        color: 0xfffaf0,
    },
    ColorNameEntry {
        name: c"ForestGreen".as_ptr(),
        color: 0x228b22,
    },
    ColorNameEntry {
        name: c"Fuchsia".as_ptr(),
        color: 0xff00ff,
    },
    ColorNameEntry {
        name: c"Gainsboro".as_ptr(),
        color: 0xdcdcdc,
    },
    ColorNameEntry {
        name: c"GhostWhite".as_ptr(),
        color: 0xf8f8ff,
    },
    ColorNameEntry {
        name: c"Gold".as_ptr(),
        color: 0xffd700,
    },
    ColorNameEntry {
        name: c"Gold1".as_ptr(),
        color: 0xffd700,
    },
    ColorNameEntry {
        name: c"Gold2".as_ptr(),
        color: 0xeec900,
    },
    ColorNameEntry {
        name: c"Gold3".as_ptr(),
        color: 0xcdad00,
    },
    ColorNameEntry {
        name: c"Gold4".as_ptr(),
        color: 0x8b7500,
    },
    ColorNameEntry {
        name: c"Goldenrod".as_ptr(),
        color: 0xdaa520,
    },
    ColorNameEntry {
        name: c"Goldenrod1".as_ptr(),
        color: 0xffc125,
    },
    ColorNameEntry {
        name: c"Goldenrod2".as_ptr(),
        color: 0xeeb422,
    },
    ColorNameEntry {
        name: c"Goldenrod3".as_ptr(),
        color: 0xcd9b1d,
    },
    ColorNameEntry {
        name: c"Goldenrod4".as_ptr(),
        color: 0x8b6914,
    },
    ColorNameEntry {
        name: c"Gray".as_ptr(),
        color: 0x808080,
    },
    ColorNameEntry {
        name: c"Gray0".as_ptr(),
        color: 0x000000,
    },
    ColorNameEntry {
        name: c"Gray1".as_ptr(),
        color: 0x030303,
    },
    ColorNameEntry {
        name: c"Gray10".as_ptr(),
        color: 0x1a1a1a,
    },
    ColorNameEntry {
        name: c"Gray100".as_ptr(),
        color: 0xffffff,
    },
    ColorNameEntry {
        name: c"Gray11".as_ptr(),
        color: 0x1c1c1c,
    },
    ColorNameEntry {
        name: c"Gray12".as_ptr(),
        color: 0x1f1f1f,
    },
    ColorNameEntry {
        name: c"Gray13".as_ptr(),
        color: 0x212121,
    },
    ColorNameEntry {
        name: c"Gray14".as_ptr(),
        color: 0x242424,
    },
    ColorNameEntry {
        name: c"Gray15".as_ptr(),
        color: 0x262626,
    },
    ColorNameEntry {
        name: c"Gray16".as_ptr(),
        color: 0x292929,
    },
    ColorNameEntry {
        name: c"Gray17".as_ptr(),
        color: 0x2b2b2b,
    },
    ColorNameEntry {
        name: c"Gray18".as_ptr(),
        color: 0x2e2e2e,
    },
    ColorNameEntry {
        name: c"Gray19".as_ptr(),
        color: 0x303030,
    },
    ColorNameEntry {
        name: c"Gray2".as_ptr(),
        color: 0x050505,
    },
    ColorNameEntry {
        name: c"Gray20".as_ptr(),
        color: 0x333333,
    },
    ColorNameEntry {
        name: c"Gray21".as_ptr(),
        color: 0x363636,
    },
    ColorNameEntry {
        name: c"Gray22".as_ptr(),
        color: 0x383838,
    },
    ColorNameEntry {
        name: c"Gray23".as_ptr(),
        color: 0x3b3b3b,
    },
    ColorNameEntry {
        name: c"Gray24".as_ptr(),
        color: 0x3d3d3d,
    },
    ColorNameEntry {
        name: c"Gray25".as_ptr(),
        color: 0x404040,
    },
    ColorNameEntry {
        name: c"Gray26".as_ptr(),
        color: 0x424242,
    },
    ColorNameEntry {
        name: c"Gray27".as_ptr(),
        color: 0x454545,
    },
    ColorNameEntry {
        name: c"Gray28".as_ptr(),
        color: 0x474747,
    },
    ColorNameEntry {
        name: c"Gray29".as_ptr(),
        color: 0x4a4a4a,
    },
    ColorNameEntry {
        name: c"Gray3".as_ptr(),
        color: 0x080808,
    },
    ColorNameEntry {
        name: c"Gray30".as_ptr(),
        color: 0x4d4d4d,
    },
    ColorNameEntry {
        name: c"Gray31".as_ptr(),
        color: 0x4f4f4f,
    },
    ColorNameEntry {
        name: c"Gray32".as_ptr(),
        color: 0x525252,
    },
    ColorNameEntry {
        name: c"Gray33".as_ptr(),
        color: 0x545454,
    },
    ColorNameEntry {
        name: c"Gray34".as_ptr(),
        color: 0x575757,
    },
    ColorNameEntry {
        name: c"Gray35".as_ptr(),
        color: 0x595959,
    },
    ColorNameEntry {
        name: c"Gray36".as_ptr(),
        color: 0x5c5c5c,
    },
    ColorNameEntry {
        name: c"Gray37".as_ptr(),
        color: 0x5e5e5e,
    },
    ColorNameEntry {
        name: c"Gray38".as_ptr(),
        color: 0x616161,
    },
    ColorNameEntry {
        name: c"Gray39".as_ptr(),
        color: 0x636363,
    },
    ColorNameEntry {
        name: c"Gray4".as_ptr(),
        color: 0x0a0a0a,
    },
    ColorNameEntry {
        name: c"Gray40".as_ptr(),
        color: 0x666666,
    },
    ColorNameEntry {
        name: c"Gray41".as_ptr(),
        color: 0x696969,
    },
    ColorNameEntry {
        name: c"Gray42".as_ptr(),
        color: 0x6b6b6b,
    },
    ColorNameEntry {
        name: c"Gray43".as_ptr(),
        color: 0x6e6e6e,
    },
    ColorNameEntry {
        name: c"Gray44".as_ptr(),
        color: 0x707070,
    },
    ColorNameEntry {
        name: c"Gray45".as_ptr(),
        color: 0x737373,
    },
    ColorNameEntry {
        name: c"Gray46".as_ptr(),
        color: 0x757575,
    },
    ColorNameEntry {
        name: c"Gray47".as_ptr(),
        color: 0x787878,
    },
    ColorNameEntry {
        name: c"Gray48".as_ptr(),
        color: 0x7a7a7a,
    },
    ColorNameEntry {
        name: c"Gray49".as_ptr(),
        color: 0x7d7d7d,
    },
    ColorNameEntry {
        name: c"Gray5".as_ptr(),
        color: 0x0d0d0d,
    },
    ColorNameEntry {
        name: c"Gray50".as_ptr(),
        color: 0x7f7f7f,
    },
    ColorNameEntry {
        name: c"Gray51".as_ptr(),
        color: 0x828282,
    },
    ColorNameEntry {
        name: c"Gray52".as_ptr(),
        color: 0x858585,
    },
    ColorNameEntry {
        name: c"Gray53".as_ptr(),
        color: 0x878787,
    },
    ColorNameEntry {
        name: c"Gray54".as_ptr(),
        color: 0x8a8a8a,
    },
    ColorNameEntry {
        name: c"Gray55".as_ptr(),
        color: 0x8c8c8c,
    },
    ColorNameEntry {
        name: c"Gray56".as_ptr(),
        color: 0x8f8f8f,
    },
    ColorNameEntry {
        name: c"Gray57".as_ptr(),
        color: 0x919191,
    },
    ColorNameEntry {
        name: c"Gray58".as_ptr(),
        color: 0x949494,
    },
    ColorNameEntry {
        name: c"Gray59".as_ptr(),
        color: 0x969696,
    },
    ColorNameEntry {
        name: c"Gray6".as_ptr(),
        color: 0x0f0f0f,
    },
    ColorNameEntry {
        name: c"Gray60".as_ptr(),
        color: 0x999999,
    },
    ColorNameEntry {
        name: c"Gray61".as_ptr(),
        color: 0x9c9c9c,
    },
    ColorNameEntry {
        name: c"Gray62".as_ptr(),
        color: 0x9e9e9e,
    },
    ColorNameEntry {
        name: c"Gray63".as_ptr(),
        color: 0xa1a1a1,
    },
    ColorNameEntry {
        name: c"Gray64".as_ptr(),
        color: 0xa3a3a3,
    },
    ColorNameEntry {
        name: c"Gray65".as_ptr(),
        color: 0xa6a6a6,
    },
    ColorNameEntry {
        name: c"Gray66".as_ptr(),
        color: 0xa8a8a8,
    },
    ColorNameEntry {
        name: c"Gray67".as_ptr(),
        color: 0xababab,
    },
    ColorNameEntry {
        name: c"Gray68".as_ptr(),
        color: 0xadadad,
    },
    ColorNameEntry {
        name: c"Gray69".as_ptr(),
        color: 0xb0b0b0,
    },
    ColorNameEntry {
        name: c"Gray7".as_ptr(),
        color: 0x121212,
    },
    ColorNameEntry {
        name: c"Gray70".as_ptr(),
        color: 0xb3b3b3,
    },
    ColorNameEntry {
        name: c"Gray71".as_ptr(),
        color: 0xb5b5b5,
    },
    ColorNameEntry {
        name: c"Gray72".as_ptr(),
        color: 0xb8b8b8,
    },
    ColorNameEntry {
        name: c"Gray73".as_ptr(),
        color: 0xbababa,
    },
    ColorNameEntry {
        name: c"Gray74".as_ptr(),
        color: 0xbdbdbd,
    },
    ColorNameEntry {
        name: c"Gray75".as_ptr(),
        color: 0xbfbfbf,
    },
    ColorNameEntry {
        name: c"Gray76".as_ptr(),
        color: 0xc2c2c2,
    },
    ColorNameEntry {
        name: c"Gray77".as_ptr(),
        color: 0xc4c4c4,
    },
    ColorNameEntry {
        name: c"Gray78".as_ptr(),
        color: 0xc7c7c7,
    },
    ColorNameEntry {
        name: c"Gray79".as_ptr(),
        color: 0xc9c9c9,
    },
    ColorNameEntry {
        name: c"Gray8".as_ptr(),
        color: 0x141414,
    },
    ColorNameEntry {
        name: c"Gray80".as_ptr(),
        color: 0xcccccc,
    },
    ColorNameEntry {
        name: c"Gray81".as_ptr(),
        color: 0xcfcfcf,
    },
    ColorNameEntry {
        name: c"Gray82".as_ptr(),
        color: 0xd1d1d1,
    },
    ColorNameEntry {
        name: c"Gray83".as_ptr(),
        color: 0xd4d4d4,
    },
    ColorNameEntry {
        name: c"Gray84".as_ptr(),
        color: 0xd6d6d6,
    },
    ColorNameEntry {
        name: c"Gray85".as_ptr(),
        color: 0xd9d9d9,
    },
    ColorNameEntry {
        name: c"Gray86".as_ptr(),
        color: 0xdbdbdb,
    },
    ColorNameEntry {
        name: c"Gray87".as_ptr(),
        color: 0xdedede,
    },
    ColorNameEntry {
        name: c"Gray88".as_ptr(),
        color: 0xe0e0e0,
    },
    ColorNameEntry {
        name: c"Gray89".as_ptr(),
        color: 0xe3e3e3,
    },
    ColorNameEntry {
        name: c"Gray9".as_ptr(),
        color: 0x171717,
    },
    ColorNameEntry {
        name: c"Gray90".as_ptr(),
        color: 0xe5e5e5,
    },
    ColorNameEntry {
        name: c"Gray91".as_ptr(),
        color: 0xe8e8e8,
    },
    ColorNameEntry {
        name: c"Gray92".as_ptr(),
        color: 0xebebeb,
    },
    ColorNameEntry {
        name: c"Gray93".as_ptr(),
        color: 0xededed,
    },
    ColorNameEntry {
        name: c"Gray94".as_ptr(),
        color: 0xf0f0f0,
    },
    ColorNameEntry {
        name: c"Gray95".as_ptr(),
        color: 0xf2f2f2,
    },
    ColorNameEntry {
        name: c"Gray96".as_ptr(),
        color: 0xf5f5f5,
    },
    ColorNameEntry {
        name: c"Gray97".as_ptr(),
        color: 0xf7f7f7,
    },
    ColorNameEntry {
        name: c"Gray98".as_ptr(),
        color: 0xfafafa,
    },
    ColorNameEntry {
        name: c"Gray99".as_ptr(),
        color: 0xfcfcfc,
    },
    ColorNameEntry {
        name: c"Green".as_ptr(),
        color: 0x008000,
    },
    ColorNameEntry {
        name: c"Green1".as_ptr(),
        color: 0x00ff00,
    },
    ColorNameEntry {
        name: c"Green2".as_ptr(),
        color: 0x00ee00,
    },
    ColorNameEntry {
        name: c"Green3".as_ptr(),
        color: 0x00cd00,
    },
    ColorNameEntry {
        name: c"Green4".as_ptr(),
        color: 0x008b00,
    },
    ColorNameEntry {
        name: c"GreenYellow".as_ptr(),
        color: 0xadff2f,
    },
    ColorNameEntry {
        name: c"Grey".as_ptr(),
        color: 0x808080,
    },
    ColorNameEntry {
        name: c"Grey0".as_ptr(),
        color: 0x000000,
    },
    ColorNameEntry {
        name: c"Grey1".as_ptr(),
        color: 0x030303,
    },
    ColorNameEntry {
        name: c"Grey10".as_ptr(),
        color: 0x1a1a1a,
    },
    ColorNameEntry {
        name: c"Grey100".as_ptr(),
        color: 0xffffff,
    },
    ColorNameEntry {
        name: c"Grey11".as_ptr(),
        color: 0x1c1c1c,
    },
    ColorNameEntry {
        name: c"Grey12".as_ptr(),
        color: 0x1f1f1f,
    },
    ColorNameEntry {
        name: c"Grey13".as_ptr(),
        color: 0x212121,
    },
    ColorNameEntry {
        name: c"Grey14".as_ptr(),
        color: 0x242424,
    },
    ColorNameEntry {
        name: c"Grey15".as_ptr(),
        color: 0x262626,
    },
    ColorNameEntry {
        name: c"Grey16".as_ptr(),
        color: 0x292929,
    },
    ColorNameEntry {
        name: c"Grey17".as_ptr(),
        color: 0x2b2b2b,
    },
    ColorNameEntry {
        name: c"Grey18".as_ptr(),
        color: 0x2e2e2e,
    },
    ColorNameEntry {
        name: c"Grey19".as_ptr(),
        color: 0x303030,
    },
    ColorNameEntry {
        name: c"Grey2".as_ptr(),
        color: 0x050505,
    },
    ColorNameEntry {
        name: c"Grey20".as_ptr(),
        color: 0x333333,
    },
    ColorNameEntry {
        name: c"Grey21".as_ptr(),
        color: 0x363636,
    },
    ColorNameEntry {
        name: c"Grey22".as_ptr(),
        color: 0x383838,
    },
    ColorNameEntry {
        name: c"Grey23".as_ptr(),
        color: 0x3b3b3b,
    },
    ColorNameEntry {
        name: c"Grey24".as_ptr(),
        color: 0x3d3d3d,
    },
    ColorNameEntry {
        name: c"Grey25".as_ptr(),
        color: 0x404040,
    },
    ColorNameEntry {
        name: c"Grey26".as_ptr(),
        color: 0x424242,
    },
    ColorNameEntry {
        name: c"Grey27".as_ptr(),
        color: 0x454545,
    },
    ColorNameEntry {
        name: c"Grey28".as_ptr(),
        color: 0x474747,
    },
    ColorNameEntry {
        name: c"Grey29".as_ptr(),
        color: 0x4a4a4a,
    },
    ColorNameEntry {
        name: c"Grey3".as_ptr(),
        color: 0x080808,
    },
    ColorNameEntry {
        name: c"Grey30".as_ptr(),
        color: 0x4d4d4d,
    },
    ColorNameEntry {
        name: c"Grey31".as_ptr(),
        color: 0x4f4f4f,
    },
    ColorNameEntry {
        name: c"Grey32".as_ptr(),
        color: 0x525252,
    },
    ColorNameEntry {
        name: c"Grey33".as_ptr(),
        color: 0x545454,
    },
    ColorNameEntry {
        name: c"Grey34".as_ptr(),
        color: 0x575757,
    },
    ColorNameEntry {
        name: c"Grey35".as_ptr(),
        color: 0x595959,
    },
    ColorNameEntry {
        name: c"Grey36".as_ptr(),
        color: 0x5c5c5c,
    },
    ColorNameEntry {
        name: c"Grey37".as_ptr(),
        color: 0x5e5e5e,
    },
    ColorNameEntry {
        name: c"Grey38".as_ptr(),
        color: 0x616161,
    },
    ColorNameEntry {
        name: c"Grey39".as_ptr(),
        color: 0x636363,
    },
    ColorNameEntry {
        name: c"Grey4".as_ptr(),
        color: 0x0a0a0a,
    },
    ColorNameEntry {
        name: c"Grey40".as_ptr(),
        color: 0x666666,
    },
    ColorNameEntry {
        name: c"Grey41".as_ptr(),
        color: 0x696969,
    },
    ColorNameEntry {
        name: c"Grey42".as_ptr(),
        color: 0x6b6b6b,
    },
    ColorNameEntry {
        name: c"Grey43".as_ptr(),
        color: 0x6e6e6e,
    },
    ColorNameEntry {
        name: c"Grey44".as_ptr(),
        color: 0x707070,
    },
    ColorNameEntry {
        name: c"Grey45".as_ptr(),
        color: 0x737373,
    },
    ColorNameEntry {
        name: c"Grey46".as_ptr(),
        color: 0x757575,
    },
    ColorNameEntry {
        name: c"Grey47".as_ptr(),
        color: 0x787878,
    },
    ColorNameEntry {
        name: c"Grey48".as_ptr(),
        color: 0x7a7a7a,
    },
    ColorNameEntry {
        name: c"Grey49".as_ptr(),
        color: 0x7d7d7d,
    },
    ColorNameEntry {
        name: c"Grey5".as_ptr(),
        color: 0x0d0d0d,
    },
    ColorNameEntry {
        name: c"Grey50".as_ptr(),
        color: 0x7f7f7f,
    },
    ColorNameEntry {
        name: c"Grey51".as_ptr(),
        color: 0x828282,
    },
    ColorNameEntry {
        name: c"Grey52".as_ptr(),
        color: 0x858585,
    },
    ColorNameEntry {
        name: c"Grey53".as_ptr(),
        color: 0x878787,
    },
    ColorNameEntry {
        name: c"Grey54".as_ptr(),
        color: 0x8a8a8a,
    },
    ColorNameEntry {
        name: c"Grey55".as_ptr(),
        color: 0x8c8c8c,
    },
    ColorNameEntry {
        name: c"Grey56".as_ptr(),
        color: 0x8f8f8f,
    },
    ColorNameEntry {
        name: c"Grey57".as_ptr(),
        color: 0x919191,
    },
    ColorNameEntry {
        name: c"Grey58".as_ptr(),
        color: 0x949494,
    },
    ColorNameEntry {
        name: c"Grey59".as_ptr(),
        color: 0x969696,
    },
    ColorNameEntry {
        name: c"Grey6".as_ptr(),
        color: 0x0f0f0f,
    },
    ColorNameEntry {
        name: c"Grey60".as_ptr(),
        color: 0x999999,
    },
    ColorNameEntry {
        name: c"Grey61".as_ptr(),
        color: 0x9c9c9c,
    },
    ColorNameEntry {
        name: c"Grey62".as_ptr(),
        color: 0x9e9e9e,
    },
    ColorNameEntry {
        name: c"Grey63".as_ptr(),
        color: 0xa1a1a1,
    },
    ColorNameEntry {
        name: c"Grey64".as_ptr(),
        color: 0xa3a3a3,
    },
    ColorNameEntry {
        name: c"Grey65".as_ptr(),
        color: 0xa6a6a6,
    },
    ColorNameEntry {
        name: c"Grey66".as_ptr(),
        color: 0xa8a8a8,
    },
    ColorNameEntry {
        name: c"Grey67".as_ptr(),
        color: 0xababab,
    },
    ColorNameEntry {
        name: c"Grey68".as_ptr(),
        color: 0xadadad,
    },
    ColorNameEntry {
        name: c"Grey69".as_ptr(),
        color: 0xb0b0b0,
    },
    ColorNameEntry {
        name: c"Grey7".as_ptr(),
        color: 0x121212,
    },
    ColorNameEntry {
        name: c"Grey70".as_ptr(),
        color: 0xb3b3b3,
    },
    ColorNameEntry {
        name: c"Grey71".as_ptr(),
        color: 0xb5b5b5,
    },
    ColorNameEntry {
        name: c"Grey72".as_ptr(),
        color: 0xb8b8b8,
    },
    ColorNameEntry {
        name: c"Grey73".as_ptr(),
        color: 0xbababa,
    },
    ColorNameEntry {
        name: c"Grey74".as_ptr(),
        color: 0xbdbdbd,
    },
    ColorNameEntry {
        name: c"Grey75".as_ptr(),
        color: 0xbfbfbf,
    },
    ColorNameEntry {
        name: c"Grey76".as_ptr(),
        color: 0xc2c2c2,
    },
    ColorNameEntry {
        name: c"Grey77".as_ptr(),
        color: 0xc4c4c4,
    },
    ColorNameEntry {
        name: c"Grey78".as_ptr(),
        color: 0xc7c7c7,
    },
    ColorNameEntry {
        name: c"Grey79".as_ptr(),
        color: 0xc9c9c9,
    },
    ColorNameEntry {
        name: c"Grey8".as_ptr(),
        color: 0x141414,
    },
    ColorNameEntry {
        name: c"Grey80".as_ptr(),
        color: 0xcccccc,
    },
    ColorNameEntry {
        name: c"Grey81".as_ptr(),
        color: 0xcfcfcf,
    },
    ColorNameEntry {
        name: c"Grey82".as_ptr(),
        color: 0xd1d1d1,
    },
    ColorNameEntry {
        name: c"Grey83".as_ptr(),
        color: 0xd4d4d4,
    },
    ColorNameEntry {
        name: c"Grey84".as_ptr(),
        color: 0xd6d6d6,
    },
    ColorNameEntry {
        name: c"Grey85".as_ptr(),
        color: 0xd9d9d9,
    },
    ColorNameEntry {
        name: c"Grey86".as_ptr(),
        color: 0xdbdbdb,
    },
    ColorNameEntry {
        name: c"Grey87".as_ptr(),
        color: 0xdedede,
    },
    ColorNameEntry {
        name: c"Grey88".as_ptr(),
        color: 0xe0e0e0,
    },
    ColorNameEntry {
        name: c"Grey89".as_ptr(),
        color: 0xe3e3e3,
    },
    ColorNameEntry {
        name: c"Grey9".as_ptr(),
        color: 0x171717,
    },
    ColorNameEntry {
        name: c"Grey90".as_ptr(),
        color: 0xe5e5e5,
    },
    ColorNameEntry {
        name: c"Grey91".as_ptr(),
        color: 0xe8e8e8,
    },
    ColorNameEntry {
        name: c"Grey92".as_ptr(),
        color: 0xebebeb,
    },
    ColorNameEntry {
        name: c"Grey93".as_ptr(),
        color: 0xededed,
    },
    ColorNameEntry {
        name: c"Grey94".as_ptr(),
        color: 0xf0f0f0,
    },
    ColorNameEntry {
        name: c"Grey95".as_ptr(),
        color: 0xf2f2f2,
    },
    ColorNameEntry {
        name: c"Grey96".as_ptr(),
        color: 0xf5f5f5,
    },
    ColorNameEntry {
        name: c"Grey97".as_ptr(),
        color: 0xf7f7f7,
    },
    ColorNameEntry {
        name: c"Grey98".as_ptr(),
        color: 0xfafafa,
    },
    ColorNameEntry {
        name: c"Grey99".as_ptr(),
        color: 0xfcfcfc,
    },
    ColorNameEntry {
        name: c"Honeydew".as_ptr(),
        color: 0xf0fff0,
    },
    ColorNameEntry {
        name: c"Honeydew1".as_ptr(),
        color: 0xf0fff0,
    },
    ColorNameEntry {
        name: c"Honeydew2".as_ptr(),
        color: 0xe0eee0,
    },
    ColorNameEntry {
        name: c"Honeydew3".as_ptr(),
        color: 0xc1cdc1,
    },
    ColorNameEntry {
        name: c"Honeydew4".as_ptr(),
        color: 0x838b83,
    },
    ColorNameEntry {
        name: c"HotPink".as_ptr(),
        color: 0xff69b4,
    },
    ColorNameEntry {
        name: c"HotPink1".as_ptr(),
        color: 0xff6eb4,
    },
    ColorNameEntry {
        name: c"HotPink2".as_ptr(),
        color: 0xee6aa7,
    },
    ColorNameEntry {
        name: c"HotPink3".as_ptr(),
        color: 0xcd6090,
    },
    ColorNameEntry {
        name: c"HotPink4".as_ptr(),
        color: 0x8b3a62,
    },
    ColorNameEntry {
        name: c"IndianRed".as_ptr(),
        color: 0xcd5c5c,
    },
    ColorNameEntry {
        name: c"IndianRed1".as_ptr(),
        color: 0xff6a6a,
    },
    ColorNameEntry {
        name: c"IndianRed2".as_ptr(),
        color: 0xee6363,
    },
    ColorNameEntry {
        name: c"IndianRed3".as_ptr(),
        color: 0xcd5555,
    },
    ColorNameEntry {
        name: c"IndianRed4".as_ptr(),
        color: 0x8b3a3a,
    },
    ColorNameEntry {
        name: c"Indigo".as_ptr(),
        color: 0x4b0082,
    },
    ColorNameEntry {
        name: c"Ivory".as_ptr(),
        color: 0xfffff0,
    },
    ColorNameEntry {
        name: c"Ivory1".as_ptr(),
        color: 0xfffff0,
    },
    ColorNameEntry {
        name: c"Ivory2".as_ptr(),
        color: 0xeeeee0,
    },
    ColorNameEntry {
        name: c"Ivory3".as_ptr(),
        color: 0xcdcdc1,
    },
    ColorNameEntry {
        name: c"Ivory4".as_ptr(),
        color: 0x8b8b83,
    },
    ColorNameEntry {
        name: c"Khaki".as_ptr(),
        color: 0xf0e68c,
    },
    ColorNameEntry {
        name: c"Khaki1".as_ptr(),
        color: 0xfff68f,
    },
    ColorNameEntry {
        name: c"Khaki2".as_ptr(),
        color: 0xeee685,
    },
    ColorNameEntry {
        name: c"Khaki3".as_ptr(),
        color: 0xcdc673,
    },
    ColorNameEntry {
        name: c"Khaki4".as_ptr(),
        color: 0x8b864e,
    },
    ColorNameEntry {
        name: c"Lavender".as_ptr(),
        color: 0xe6e6fa,
    },
    ColorNameEntry {
        name: c"LavenderBlush".as_ptr(),
        color: 0xfff0f5,
    },
    ColorNameEntry {
        name: c"LavenderBlush1".as_ptr(),
        color: 0xfff0f5,
    },
    ColorNameEntry {
        name: c"LavenderBlush2".as_ptr(),
        color: 0xeee0e5,
    },
    ColorNameEntry {
        name: c"LavenderBlush3".as_ptr(),
        color: 0xcdc1c5,
    },
    ColorNameEntry {
        name: c"LavenderBlush4".as_ptr(),
        color: 0x8b8386,
    },
    ColorNameEntry {
        name: c"LawnGreen".as_ptr(),
        color: 0x7cfc00,
    },
    ColorNameEntry {
        name: c"LemonChiffon".as_ptr(),
        color: 0xfffacd,
    },
    ColorNameEntry {
        name: c"LemonChiffon1".as_ptr(),
        color: 0xfffacd,
    },
    ColorNameEntry {
        name: c"LemonChiffon2".as_ptr(),
        color: 0xeee9bf,
    },
    ColorNameEntry {
        name: c"LemonChiffon3".as_ptr(),
        color: 0xcdc9a5,
    },
    ColorNameEntry {
        name: c"LemonChiffon4".as_ptr(),
        color: 0x8b8970,
    },
    ColorNameEntry {
        name: c"LightBlue".as_ptr(),
        color: 0xadd8e6,
    },
    ColorNameEntry {
        name: c"LightBlue1".as_ptr(),
        color: 0xbfefff,
    },
    ColorNameEntry {
        name: c"LightBlue2".as_ptr(),
        color: 0xb2dfee,
    },
    ColorNameEntry {
        name: c"LightBlue3".as_ptr(),
        color: 0x9ac0cd,
    },
    ColorNameEntry {
        name: c"LightBlue4".as_ptr(),
        color: 0x68838b,
    },
    ColorNameEntry {
        name: c"LightCoral".as_ptr(),
        color: 0xf08080,
    },
    ColorNameEntry {
        name: c"LightCyan".as_ptr(),
        color: 0xe0ffff,
    },
    ColorNameEntry {
        name: c"LightCyan1".as_ptr(),
        color: 0xe0ffff,
    },
    ColorNameEntry {
        name: c"LightCyan2".as_ptr(),
        color: 0xd1eeee,
    },
    ColorNameEntry {
        name: c"LightCyan3".as_ptr(),
        color: 0xb4cdcd,
    },
    ColorNameEntry {
        name: c"LightCyan4".as_ptr(),
        color: 0x7a8b8b,
    },
    ColorNameEntry {
        name: c"LightGoldenrod".as_ptr(),
        color: 0xeedd82,
    },
    ColorNameEntry {
        name: c"LightGoldenrod1".as_ptr(),
        color: 0xffec8b,
    },
    ColorNameEntry {
        name: c"LightGoldenrod2".as_ptr(),
        color: 0xeedc82,
    },
    ColorNameEntry {
        name: c"LightGoldenrod3".as_ptr(),
        color: 0xcdbe70,
    },
    ColorNameEntry {
        name: c"LightGoldenrod4".as_ptr(),
        color: 0x8b814c,
    },
    ColorNameEntry {
        name: c"LightGoldenrodYellow".as_ptr(),
        color: 0xfafad2,
    },
    ColorNameEntry {
        name: c"LightGray".as_ptr(),
        color: 0xd3d3d3,
    },
    ColorNameEntry {
        name: c"LightGreen".as_ptr(),
        color: 0x90ee90,
    },
    ColorNameEntry {
        name: c"LightGrey".as_ptr(),
        color: 0xd3d3d3,
    },
    ColorNameEntry {
        name: c"LightMagenta".as_ptr(),
        color: 0xffbbff,
    },
    ColorNameEntry {
        name: c"LightPink".as_ptr(),
        color: 0xffb6c1,
    },
    ColorNameEntry {
        name: c"LightPink1".as_ptr(),
        color: 0xffaeb9,
    },
    ColorNameEntry {
        name: c"LightPink2".as_ptr(),
        color: 0xeea2ad,
    },
    ColorNameEntry {
        name: c"LightPink3".as_ptr(),
        color: 0xcd8c95,
    },
    ColorNameEntry {
        name: c"LightPink4".as_ptr(),
        color: 0x8b5f65,
    },
    ColorNameEntry {
        name: c"LightRed".as_ptr(),
        color: 0xffbbbb,
    },
    ColorNameEntry {
        name: c"LightSalmon".as_ptr(),
        color: 0xffa07a,
    },
    ColorNameEntry {
        name: c"LightSalmon1".as_ptr(),
        color: 0xffa07a,
    },
    ColorNameEntry {
        name: c"LightSalmon2".as_ptr(),
        color: 0xee9572,
    },
    ColorNameEntry {
        name: c"LightSalmon3".as_ptr(),
        color: 0xcd8162,
    },
    ColorNameEntry {
        name: c"LightSalmon4".as_ptr(),
        color: 0x8b5742,
    },
    ColorNameEntry {
        name: c"LightSeaGreen".as_ptr(),
        color: 0x20b2aa,
    },
    ColorNameEntry {
        name: c"LightSkyBlue".as_ptr(),
        color: 0x87cefa,
    },
    ColorNameEntry {
        name: c"LightSkyBlue1".as_ptr(),
        color: 0xb0e2ff,
    },
    ColorNameEntry {
        name: c"LightSkyBlue2".as_ptr(),
        color: 0xa4d3ee,
    },
    ColorNameEntry {
        name: c"LightSkyBlue3".as_ptr(),
        color: 0x8db6cd,
    },
    ColorNameEntry {
        name: c"LightSkyBlue4".as_ptr(),
        color: 0x607b8b,
    },
    ColorNameEntry {
        name: c"LightSlateBlue".as_ptr(),
        color: 0x8470ff,
    },
    ColorNameEntry {
        name: c"LightSlateGray".as_ptr(),
        color: 0x778899,
    },
    ColorNameEntry {
        name: c"LightSlateGrey".as_ptr(),
        color: 0x778899,
    },
    ColorNameEntry {
        name: c"LightSteelBlue".as_ptr(),
        color: 0xb0c4de,
    },
    ColorNameEntry {
        name: c"LightSteelBlue1".as_ptr(),
        color: 0xcae1ff,
    },
    ColorNameEntry {
        name: c"LightSteelBlue2".as_ptr(),
        color: 0xbcd2ee,
    },
    ColorNameEntry {
        name: c"LightSteelBlue3".as_ptr(),
        color: 0xa2b5cd,
    },
    ColorNameEntry {
        name: c"LightSteelBlue4".as_ptr(),
        color: 0x6e7b8b,
    },
    ColorNameEntry {
        name: c"LightYellow".as_ptr(),
        color: 0xffffe0,
    },
    ColorNameEntry {
        name: c"LightYellow1".as_ptr(),
        color: 0xffffe0,
    },
    ColorNameEntry {
        name: c"LightYellow2".as_ptr(),
        color: 0xeeeed1,
    },
    ColorNameEntry {
        name: c"LightYellow3".as_ptr(),
        color: 0xcdcdb4,
    },
    ColorNameEntry {
        name: c"LightYellow4".as_ptr(),
        color: 0x8b8b7a,
    },
    ColorNameEntry {
        name: c"Lime".as_ptr(),
        color: 0x00ff00,
    },
    ColorNameEntry {
        name: c"LimeGreen".as_ptr(),
        color: 0x32cd32,
    },
    ColorNameEntry {
        name: c"Linen".as_ptr(),
        color: 0xfaf0e6,
    },
    ColorNameEntry {
        name: c"Magenta".as_ptr(),
        color: 0xff00ff,
    },
    ColorNameEntry {
        name: c"Magenta1".as_ptr(),
        color: 0xff00ff,
    },
    ColorNameEntry {
        name: c"Magenta2".as_ptr(),
        color: 0xee00ee,
    },
    ColorNameEntry {
        name: c"Magenta3".as_ptr(),
        color: 0xcd00cd,
    },
    ColorNameEntry {
        name: c"Magenta4".as_ptr(),
        color: 0x8b008b,
    },
    ColorNameEntry {
        name: c"Maroon".as_ptr(),
        color: 0x800000,
    },
    ColorNameEntry {
        name: c"Maroon1".as_ptr(),
        color: 0xff34b3,
    },
    ColorNameEntry {
        name: c"Maroon2".as_ptr(),
        color: 0xee30a7,
    },
    ColorNameEntry {
        name: c"Maroon3".as_ptr(),
        color: 0xcd2990,
    },
    ColorNameEntry {
        name: c"Maroon4".as_ptr(),
        color: 0x8b1c62,
    },
    ColorNameEntry {
        name: c"MediumAquamarine".as_ptr(),
        color: 0x66cdaa,
    },
    ColorNameEntry {
        name: c"MediumBlue".as_ptr(),
        color: 0x0000cd,
    },
    ColorNameEntry {
        name: c"MediumOrchid".as_ptr(),
        color: 0xba55d3,
    },
    ColorNameEntry {
        name: c"MediumOrchid1".as_ptr(),
        color: 0xe066ff,
    },
    ColorNameEntry {
        name: c"MediumOrchid2".as_ptr(),
        color: 0xd15fee,
    },
    ColorNameEntry {
        name: c"MediumOrchid3".as_ptr(),
        color: 0xb452cd,
    },
    ColorNameEntry {
        name: c"MediumOrchid4".as_ptr(),
        color: 0x7a378b,
    },
    ColorNameEntry {
        name: c"MediumPurple".as_ptr(),
        color: 0x9370db,
    },
    ColorNameEntry {
        name: c"MediumPurple1".as_ptr(),
        color: 0xab82ff,
    },
    ColorNameEntry {
        name: c"MediumPurple2".as_ptr(),
        color: 0x9f79ee,
    },
    ColorNameEntry {
        name: c"MediumPurple3".as_ptr(),
        color: 0x8968cd,
    },
    ColorNameEntry {
        name: c"MediumPurple4".as_ptr(),
        color: 0x5d478b,
    },
    ColorNameEntry {
        name: c"MediumSeaGreen".as_ptr(),
        color: 0x3cb371,
    },
    ColorNameEntry {
        name: c"MediumSlateBlue".as_ptr(),
        color: 0x7b68ee,
    },
    ColorNameEntry {
        name: c"MediumSpringGreen".as_ptr(),
        color: 0x00fa9a,
    },
    ColorNameEntry {
        name: c"MediumTurquoise".as_ptr(),
        color: 0x48d1cc,
    },
    ColorNameEntry {
        name: c"MediumVioletRed".as_ptr(),
        color: 0xc71585,
    },
    ColorNameEntry {
        name: c"MidnightBlue".as_ptr(),
        color: 0x191970,
    },
    ColorNameEntry {
        name: c"MintCream".as_ptr(),
        color: 0xf5fffa,
    },
    ColorNameEntry {
        name: c"MistyRose".as_ptr(),
        color: 0xffe4e1,
    },
    ColorNameEntry {
        name: c"MistyRose1".as_ptr(),
        color: 0xffe4e1,
    },
    ColorNameEntry {
        name: c"MistyRose2".as_ptr(),
        color: 0xeed5d2,
    },
    ColorNameEntry {
        name: c"MistyRose3".as_ptr(),
        color: 0xcdb7b5,
    },
    ColorNameEntry {
        name: c"MistyRose4".as_ptr(),
        color: 0x8b7d7b,
    },
    ColorNameEntry {
        name: c"Moccasin".as_ptr(),
        color: 0xffe4b5,
    },
    ColorNameEntry {
        name: c"NavajoWhite".as_ptr(),
        color: 0xffdead,
    },
    ColorNameEntry {
        name: c"NavajoWhite1".as_ptr(),
        color: 0xffdead,
    },
    ColorNameEntry {
        name: c"NavajoWhite2".as_ptr(),
        color: 0xeecfa1,
    },
    ColorNameEntry {
        name: c"NavajoWhite3".as_ptr(),
        color: 0xcdb38b,
    },
    ColorNameEntry {
        name: c"NavajoWhite4".as_ptr(),
        color: 0x8b795e,
    },
    ColorNameEntry {
        name: c"Navy".as_ptr(),
        color: 0x000080,
    },
    ColorNameEntry {
        name: c"NavyBlue".as_ptr(),
        color: 0x000080,
    },
    ColorNameEntry {
        name: c"NvimDarkBlue".as_ptr(),
        color: 0x004c73,
    },
    ColorNameEntry {
        name: c"NvimDarkCyan".as_ptr(),
        color: 0x007373,
    },
    ColorNameEntry {
        name: c"NvimDarkGray1".as_ptr(),
        color: 0x07080d,
    },
    ColorNameEntry {
        name: c"NvimDarkGray2".as_ptr(),
        color: 0x14161b,
    },
    ColorNameEntry {
        name: c"NvimDarkGray3".as_ptr(),
        color: 0x2c2e33,
    },
    ColorNameEntry {
        name: c"NvimDarkGray4".as_ptr(),
        color: 0x4f5258,
    },
    ColorNameEntry {
        name: c"NvimDarkGreen".as_ptr(),
        color: 0x005523,
    },
    ColorNameEntry {
        name: c"NvimDarkGrey1".as_ptr(),
        color: 0x07080d,
    },
    ColorNameEntry {
        name: c"NvimDarkGrey2".as_ptr(),
        color: 0x14161b,
    },
    ColorNameEntry {
        name: c"NvimDarkGrey3".as_ptr(),
        color: 0x2c2e33,
    },
    ColorNameEntry {
        name: c"NvimDarkGrey4".as_ptr(),
        color: 0x4f5258,
    },
    ColorNameEntry {
        name: c"NvimDarkMagenta".as_ptr(),
        color: 0x470045,
    },
    ColorNameEntry {
        name: c"NvimDarkRed".as_ptr(),
        color: 0x590008,
    },
    ColorNameEntry {
        name: c"NvimDarkYellow".as_ptr(),
        color: 0x6b5300,
    },
    ColorNameEntry {
        name: c"NvimLightBlue".as_ptr(),
        color: 0xa6dbff,
    },
    ColorNameEntry {
        name: c"NvimLightCyan".as_ptr(),
        color: 0x8cf8f7,
    },
    ColorNameEntry {
        name: c"NvimLightGray1".as_ptr(),
        color: 0xeef1f8,
    },
    ColorNameEntry {
        name: c"NvimLightGray2".as_ptr(),
        color: 0xe0e2ea,
    },
    ColorNameEntry {
        name: c"NvimLightGray3".as_ptr(),
        color: 0xc4c6cd,
    },
    ColorNameEntry {
        name: c"NvimLightGray4".as_ptr(),
        color: 0x9b9ea4,
    },
    ColorNameEntry {
        name: c"NvimLightGreen".as_ptr(),
        color: 0xb3f6c0,
    },
    ColorNameEntry {
        name: c"NvimLightGrey1".as_ptr(),
        color: 0xeef1f8,
    },
    ColorNameEntry {
        name: c"NvimLightGrey2".as_ptr(),
        color: 0xe0e2ea,
    },
    ColorNameEntry {
        name: c"NvimLightGrey3".as_ptr(),
        color: 0xc4c6cd,
    },
    ColorNameEntry {
        name: c"NvimLightGrey4".as_ptr(),
        color: 0x9b9ea4,
    },
    ColorNameEntry {
        name: c"NvimLightMagenta".as_ptr(),
        color: 0xffcaff,
    },
    ColorNameEntry {
        name: c"NvimLightRed".as_ptr(),
        color: 0xffc0b9,
    },
    ColorNameEntry {
        name: c"NvimLightYellow".as_ptr(),
        color: 0xfce094,
    },
    ColorNameEntry {
        name: c"OldLace".as_ptr(),
        color: 0xfdf5e6,
    },
    ColorNameEntry {
        name: c"Olive".as_ptr(),
        color: 0x808000,
    },
    ColorNameEntry {
        name: c"OliveDrab".as_ptr(),
        color: 0x6b8e23,
    },
    ColorNameEntry {
        name: c"OliveDrab1".as_ptr(),
        color: 0xc0ff3e,
    },
    ColorNameEntry {
        name: c"OliveDrab2".as_ptr(),
        color: 0xb3ee3a,
    },
    ColorNameEntry {
        name: c"OliveDrab3".as_ptr(),
        color: 0x9acd32,
    },
    ColorNameEntry {
        name: c"OliveDrab4".as_ptr(),
        color: 0x698b22,
    },
    ColorNameEntry {
        name: c"Orange".as_ptr(),
        color: 0xffa500,
    },
    ColorNameEntry {
        name: c"Orange1".as_ptr(),
        color: 0xffa500,
    },
    ColorNameEntry {
        name: c"Orange2".as_ptr(),
        color: 0xee9a00,
    },
    ColorNameEntry {
        name: c"Orange3".as_ptr(),
        color: 0xcd8500,
    },
    ColorNameEntry {
        name: c"Orange4".as_ptr(),
        color: 0x8b5a00,
    },
    ColorNameEntry {
        name: c"OrangeRed".as_ptr(),
        color: 0xff4500,
    },
    ColorNameEntry {
        name: c"OrangeRed1".as_ptr(),
        color: 0xff4500,
    },
    ColorNameEntry {
        name: c"OrangeRed2".as_ptr(),
        color: 0xee4000,
    },
    ColorNameEntry {
        name: c"OrangeRed3".as_ptr(),
        color: 0xcd3700,
    },
    ColorNameEntry {
        name: c"OrangeRed4".as_ptr(),
        color: 0x8b2500,
    },
    ColorNameEntry {
        name: c"Orchid".as_ptr(),
        color: 0xda70d6,
    },
    ColorNameEntry {
        name: c"Orchid1".as_ptr(),
        color: 0xff83fa,
    },
    ColorNameEntry {
        name: c"Orchid2".as_ptr(),
        color: 0xee7ae9,
    },
    ColorNameEntry {
        name: c"Orchid3".as_ptr(),
        color: 0xcd69c9,
    },
    ColorNameEntry {
        name: c"Orchid4".as_ptr(),
        color: 0x8b4789,
    },
    ColorNameEntry {
        name: c"PaleGoldenrod".as_ptr(),
        color: 0xeee8aa,
    },
    ColorNameEntry {
        name: c"PaleGreen".as_ptr(),
        color: 0x98fb98,
    },
    ColorNameEntry {
        name: c"PaleGreen1".as_ptr(),
        color: 0x9aff9a,
    },
    ColorNameEntry {
        name: c"PaleGreen2".as_ptr(),
        color: 0x90ee90,
    },
    ColorNameEntry {
        name: c"PaleGreen3".as_ptr(),
        color: 0x7ccd7c,
    },
    ColorNameEntry {
        name: c"PaleGreen4".as_ptr(),
        color: 0x548b54,
    },
    ColorNameEntry {
        name: c"PaleTurquoise".as_ptr(),
        color: 0xafeeee,
    },
    ColorNameEntry {
        name: c"PaleTurquoise1".as_ptr(),
        color: 0xbbffff,
    },
    ColorNameEntry {
        name: c"PaleTurquoise2".as_ptr(),
        color: 0xaeeeee,
    },
    ColorNameEntry {
        name: c"PaleTurquoise3".as_ptr(),
        color: 0x96cdcd,
    },
    ColorNameEntry {
        name: c"PaleTurquoise4".as_ptr(),
        color: 0x668b8b,
    },
    ColorNameEntry {
        name: c"PaleVioletRed".as_ptr(),
        color: 0xdb7093,
    },
    ColorNameEntry {
        name: c"PaleVioletRed1".as_ptr(),
        color: 0xff82ab,
    },
    ColorNameEntry {
        name: c"PaleVioletRed2".as_ptr(),
        color: 0xee799f,
    },
    ColorNameEntry {
        name: c"PaleVioletRed3".as_ptr(),
        color: 0xcd6889,
    },
    ColorNameEntry {
        name: c"PaleVioletRed4".as_ptr(),
        color: 0x8b475d,
    },
    ColorNameEntry {
        name: c"PapayaWhip".as_ptr(),
        color: 0xffefd5,
    },
    ColorNameEntry {
        name: c"PeachPuff".as_ptr(),
        color: 0xffdab9,
    },
    ColorNameEntry {
        name: c"PeachPuff1".as_ptr(),
        color: 0xffdab9,
    },
    ColorNameEntry {
        name: c"PeachPuff2".as_ptr(),
        color: 0xeecbad,
    },
    ColorNameEntry {
        name: c"PeachPuff3".as_ptr(),
        color: 0xcdaf95,
    },
    ColorNameEntry {
        name: c"PeachPuff4".as_ptr(),
        color: 0x8b7765,
    },
    ColorNameEntry {
        name: c"Peru".as_ptr(),
        color: 0xcd853f,
    },
    ColorNameEntry {
        name: c"Pink".as_ptr(),
        color: 0xffc0cb,
    },
    ColorNameEntry {
        name: c"Pink1".as_ptr(),
        color: 0xffb5c5,
    },
    ColorNameEntry {
        name: c"Pink2".as_ptr(),
        color: 0xeea9b8,
    },
    ColorNameEntry {
        name: c"Pink3".as_ptr(),
        color: 0xcd919e,
    },
    ColorNameEntry {
        name: c"Pink4".as_ptr(),
        color: 0x8b636c,
    },
    ColorNameEntry {
        name: c"Plum".as_ptr(),
        color: 0xdda0dd,
    },
    ColorNameEntry {
        name: c"Plum1".as_ptr(),
        color: 0xffbbff,
    },
    ColorNameEntry {
        name: c"Plum2".as_ptr(),
        color: 0xeeaeee,
    },
    ColorNameEntry {
        name: c"Plum3".as_ptr(),
        color: 0xcd96cd,
    },
    ColorNameEntry {
        name: c"Plum4".as_ptr(),
        color: 0x8b668b,
    },
    ColorNameEntry {
        name: c"PowderBlue".as_ptr(),
        color: 0xb0e0e6,
    },
    ColorNameEntry {
        name: c"Purple".as_ptr(),
        color: 0x800080,
    },
    ColorNameEntry {
        name: c"Purple1".as_ptr(),
        color: 0x9b30ff,
    },
    ColorNameEntry {
        name: c"Purple2".as_ptr(),
        color: 0x912cee,
    },
    ColorNameEntry {
        name: c"Purple3".as_ptr(),
        color: 0x7d26cd,
    },
    ColorNameEntry {
        name: c"Purple4".as_ptr(),
        color: 0x551a8b,
    },
    ColorNameEntry {
        name: c"RebeccaPurple".as_ptr(),
        color: 0x663399,
    },
    ColorNameEntry {
        name: c"Red".as_ptr(),
        color: 0xff0000,
    },
    ColorNameEntry {
        name: c"Red1".as_ptr(),
        color: 0xff0000,
    },
    ColorNameEntry {
        name: c"Red2".as_ptr(),
        color: 0xee0000,
    },
    ColorNameEntry {
        name: c"Red3".as_ptr(),
        color: 0xcd0000,
    },
    ColorNameEntry {
        name: c"Red4".as_ptr(),
        color: 0x8b0000,
    },
    ColorNameEntry {
        name: c"RosyBrown".as_ptr(),
        color: 0xbc8f8f,
    },
    ColorNameEntry {
        name: c"RosyBrown1".as_ptr(),
        color: 0xffc1c1,
    },
    ColorNameEntry {
        name: c"RosyBrown2".as_ptr(),
        color: 0xeeb4b4,
    },
    ColorNameEntry {
        name: c"RosyBrown3".as_ptr(),
        color: 0xcd9b9b,
    },
    ColorNameEntry {
        name: c"RosyBrown4".as_ptr(),
        color: 0x8b6969,
    },
    ColorNameEntry {
        name: c"RoyalBlue".as_ptr(),
        color: 0x4169e1,
    },
    ColorNameEntry {
        name: c"RoyalBlue1".as_ptr(),
        color: 0x4876ff,
    },
    ColorNameEntry {
        name: c"RoyalBlue2".as_ptr(),
        color: 0x436eee,
    },
    ColorNameEntry {
        name: c"RoyalBlue3".as_ptr(),
        color: 0x3a5fcd,
    },
    ColorNameEntry {
        name: c"RoyalBlue4".as_ptr(),
        color: 0x27408b,
    },
    ColorNameEntry {
        name: c"SaddleBrown".as_ptr(),
        color: 0x8b4513,
    },
    ColorNameEntry {
        name: c"Salmon".as_ptr(),
        color: 0xfa8072,
    },
    ColorNameEntry {
        name: c"Salmon1".as_ptr(),
        color: 0xff8c69,
    },
    ColorNameEntry {
        name: c"Salmon2".as_ptr(),
        color: 0xee8262,
    },
    ColorNameEntry {
        name: c"Salmon3".as_ptr(),
        color: 0xcd7054,
    },
    ColorNameEntry {
        name: c"Salmon4".as_ptr(),
        color: 0x8b4c39,
    },
    ColorNameEntry {
        name: c"SandyBrown".as_ptr(),
        color: 0xf4a460,
    },
    ColorNameEntry {
        name: c"SeaGreen".as_ptr(),
        color: 0x2e8b57,
    },
    ColorNameEntry {
        name: c"SeaGreen1".as_ptr(),
        color: 0x54ff9f,
    },
    ColorNameEntry {
        name: c"SeaGreen2".as_ptr(),
        color: 0x4eee94,
    },
    ColorNameEntry {
        name: c"SeaGreen3".as_ptr(),
        color: 0x43cd80,
    },
    ColorNameEntry {
        name: c"SeaGreen4".as_ptr(),
        color: 0x2e8b57,
    },
    ColorNameEntry {
        name: c"SeaShell".as_ptr(),
        color: 0xfff5ee,
    },
    ColorNameEntry {
        name: c"Seashell1".as_ptr(),
        color: 0xfff5ee,
    },
    ColorNameEntry {
        name: c"Seashell2".as_ptr(),
        color: 0xeee5de,
    },
    ColorNameEntry {
        name: c"Seashell3".as_ptr(),
        color: 0xcdc5bf,
    },
    ColorNameEntry {
        name: c"Seashell4".as_ptr(),
        color: 0x8b8682,
    },
    ColorNameEntry {
        name: c"Sienna".as_ptr(),
        color: 0xa0522d,
    },
    ColorNameEntry {
        name: c"Sienna1".as_ptr(),
        color: 0xff8247,
    },
    ColorNameEntry {
        name: c"Sienna2".as_ptr(),
        color: 0xee7942,
    },
    ColorNameEntry {
        name: c"Sienna3".as_ptr(),
        color: 0xcd6839,
    },
    ColorNameEntry {
        name: c"Sienna4".as_ptr(),
        color: 0x8b4726,
    },
    ColorNameEntry {
        name: c"Silver".as_ptr(),
        color: 0xc0c0c0,
    },
    ColorNameEntry {
        name: c"SkyBlue".as_ptr(),
        color: 0x87ceeb,
    },
    ColorNameEntry {
        name: c"SkyBlue1".as_ptr(),
        color: 0x87ceff,
    },
    ColorNameEntry {
        name: c"SkyBlue2".as_ptr(),
        color: 0x7ec0ee,
    },
    ColorNameEntry {
        name: c"SkyBlue3".as_ptr(),
        color: 0x6ca6cd,
    },
    ColorNameEntry {
        name: c"SkyBlue4".as_ptr(),
        color: 0x4a708b,
    },
    ColorNameEntry {
        name: c"SlateBlue".as_ptr(),
        color: 0x6a5acd,
    },
    ColorNameEntry {
        name: c"SlateBlue1".as_ptr(),
        color: 0x836fff,
    },
    ColorNameEntry {
        name: c"SlateBlue2".as_ptr(),
        color: 0x7a67ee,
    },
    ColorNameEntry {
        name: c"SlateBlue3".as_ptr(),
        color: 0x6959cd,
    },
    ColorNameEntry {
        name: c"SlateBlue4".as_ptr(),
        color: 0x473c8b,
    },
    ColorNameEntry {
        name: c"SlateGray".as_ptr(),
        color: 0x708090,
    },
    ColorNameEntry {
        name: c"SlateGray1".as_ptr(),
        color: 0xc6e2ff,
    },
    ColorNameEntry {
        name: c"SlateGray2".as_ptr(),
        color: 0xb9d3ee,
    },
    ColorNameEntry {
        name: c"SlateGray3".as_ptr(),
        color: 0x9fb6cd,
    },
    ColorNameEntry {
        name: c"SlateGray4".as_ptr(),
        color: 0x6c7b8b,
    },
    ColorNameEntry {
        name: c"SlateGrey".as_ptr(),
        color: 0x708090,
    },
    ColorNameEntry {
        name: c"Snow".as_ptr(),
        color: 0xfffafa,
    },
    ColorNameEntry {
        name: c"Snow1".as_ptr(),
        color: 0xfffafa,
    },
    ColorNameEntry {
        name: c"Snow2".as_ptr(),
        color: 0xeee9e9,
    },
    ColorNameEntry {
        name: c"Snow3".as_ptr(),
        color: 0xcdc9c9,
    },
    ColorNameEntry {
        name: c"Snow4".as_ptr(),
        color: 0x8b8989,
    },
    ColorNameEntry {
        name: c"SpringGreen".as_ptr(),
        color: 0x00ff7f,
    },
    ColorNameEntry {
        name: c"SpringGreen1".as_ptr(),
        color: 0x00ff7f,
    },
    ColorNameEntry {
        name: c"SpringGreen2".as_ptr(),
        color: 0x00ee76,
    },
    ColorNameEntry {
        name: c"SpringGreen3".as_ptr(),
        color: 0x00cd66,
    },
    ColorNameEntry {
        name: c"SpringGreen4".as_ptr(),
        color: 0x008b45,
    },
    ColorNameEntry {
        name: c"SteelBlue".as_ptr(),
        color: 0x4682b4,
    },
    ColorNameEntry {
        name: c"SteelBlue1".as_ptr(),
        color: 0x63b8ff,
    },
    ColorNameEntry {
        name: c"SteelBlue2".as_ptr(),
        color: 0x5cacee,
    },
    ColorNameEntry {
        name: c"SteelBlue3".as_ptr(),
        color: 0x4f94cd,
    },
    ColorNameEntry {
        name: c"SteelBlue4".as_ptr(),
        color: 0x36648b,
    },
    ColorNameEntry {
        name: c"Tan".as_ptr(),
        color: 0xd2b48c,
    },
    ColorNameEntry {
        name: c"Tan1".as_ptr(),
        color: 0xffa54f,
    },
    ColorNameEntry {
        name: c"Tan2".as_ptr(),
        color: 0xee9a49,
    },
    ColorNameEntry {
        name: c"Tan3".as_ptr(),
        color: 0xcd853f,
    },
    ColorNameEntry {
        name: c"Tan4".as_ptr(),
        color: 0x8b5a2b,
    },
    ColorNameEntry {
        name: c"Teal".as_ptr(),
        color: 0x008080,
    },
    ColorNameEntry {
        name: c"Thistle".as_ptr(),
        color: 0xd8bfd8,
    },
    ColorNameEntry {
        name: c"Thistle1".as_ptr(),
        color: 0xffe1ff,
    },
    ColorNameEntry {
        name: c"Thistle2".as_ptr(),
        color: 0xeed2ee,
    },
    ColorNameEntry {
        name: c"Thistle3".as_ptr(),
        color: 0xcdb5cd,
    },
    ColorNameEntry {
        name: c"Thistle4".as_ptr(),
        color: 0x8b7b8b,
    },
    ColorNameEntry {
        name: c"Tomato".as_ptr(),
        color: 0xff6347,
    },
    ColorNameEntry {
        name: c"Tomato1".as_ptr(),
        color: 0xff6347,
    },
    ColorNameEntry {
        name: c"Tomato2".as_ptr(),
        color: 0xee5c42,
    },
    ColorNameEntry {
        name: c"Tomato3".as_ptr(),
        color: 0xcd4f39,
    },
    ColorNameEntry {
        name: c"Tomato4".as_ptr(),
        color: 0x8b3626,
    },
    ColorNameEntry {
        name: c"Turquoise".as_ptr(),
        color: 0x40e0d0,
    },
    ColorNameEntry {
        name: c"Turquoise1".as_ptr(),
        color: 0x00f5ff,
    },
    ColorNameEntry {
        name: c"Turquoise2".as_ptr(),
        color: 0x00e5ee,
    },
    ColorNameEntry {
        name: c"Turquoise3".as_ptr(),
        color: 0x00c5cd,
    },
    ColorNameEntry {
        name: c"Turquoise4".as_ptr(),
        color: 0x00868b,
    },
    ColorNameEntry {
        name: c"Violet".as_ptr(),
        color: 0xee82ee,
    },
    ColorNameEntry {
        name: c"VioletRed".as_ptr(),
        color: 0xd02090,
    },
    ColorNameEntry {
        name: c"VioletRed1".as_ptr(),
        color: 0xff3e96,
    },
    ColorNameEntry {
        name: c"VioletRed2".as_ptr(),
        color: 0xee3a8c,
    },
    ColorNameEntry {
        name: c"VioletRed3".as_ptr(),
        color: 0xcd3278,
    },
    ColorNameEntry {
        name: c"VioletRed4".as_ptr(),
        color: 0x8b2252,
    },
    ColorNameEntry {
        name: c"WebGray".as_ptr(),
        color: 0x808080,
    },
    ColorNameEntry {
        name: c"WebGreen".as_ptr(),
        color: 0x008000,
    },
    ColorNameEntry {
        name: c"WebGrey".as_ptr(),
        color: 0x808080,
    },
    ColorNameEntry {
        name: c"WebMaroon".as_ptr(),
        color: 0x800000,
    },
    ColorNameEntry {
        name: c"WebPurple".as_ptr(),
        color: 0x800080,
    },
    ColorNameEntry {
        name: c"Wheat".as_ptr(),
        color: 0xf5deb3,
    },
    ColorNameEntry {
        name: c"Wheat1".as_ptr(),
        color: 0xffe7ba,
    },
    ColorNameEntry {
        name: c"Wheat2".as_ptr(),
        color: 0xeed8ae,
    },
    ColorNameEntry {
        name: c"Wheat3".as_ptr(),
        color: 0xcdba96,
    },
    ColorNameEntry {
        name: c"Wheat4".as_ptr(),
        color: 0x8b7e66,
    },
    ColorNameEntry {
        name: c"White".as_ptr(),
        color: 0xffffff,
    },
    ColorNameEntry {
        name: c"WhiteSmoke".as_ptr(),
        color: 0xf5f5f5,
    },
    ColorNameEntry {
        name: c"X11Gray".as_ptr(),
        color: 0xbebebe,
    },
    ColorNameEntry {
        name: c"X11Green".as_ptr(),
        color: 0x00ff00,
    },
    ColorNameEntry {
        name: c"X11Grey".as_ptr(),
        color: 0xbebebe,
    },
    ColorNameEntry {
        name: c"X11Maroon".as_ptr(),
        color: 0xb03060,
    },
    ColorNameEntry {
        name: c"X11Purple".as_ptr(),
        color: 0xa020f0,
    },
    ColorNameEntry {
        name: c"Yellow".as_ptr(),
        color: 0xffff00,
    },
    ColorNameEntry {
        name: c"Yellow1".as_ptr(),
        color: 0xffff00,
    },
    ColorNameEntry {
        name: c"Yellow2".as_ptr(),
        color: 0xeeee00,
    },
    ColorNameEntry {
        name: c"Yellow3".as_ptr(),
        color: 0xcdcd00,
    },
    ColorNameEntry {
        name: c"Yellow4".as_ptr(),
        color: 0x8b8b00,
    },
    ColorNameEntry {
        name: c"YellowGreen".as_ptr(),
        color: 0x9acd32,
    },
    ColorNameEntry {
        name: std::ptr::null(),
        color: 0,
    },
];

/// Static strings for special color indices
static FG_STR: &CStr = c"fg";
static BG_STR: &CStr = c"bg";

/// Convert a color index to a color name string.
///
/// # Arguments
/// * `idx` - Color index from name_to_color, or special constant
/// * `val` - RGB color value (used only for kColorIdxHex)
/// * `hexbuf` - Buffer to write hex string to (must be at least 8 bytes)
///
/// # Returns
/// Pointer to color name string (static or in hexbuf), or NULL if idx is kColorIdxNone
///
/// # Safety
/// - hexbuf must be a valid pointer to at least 8 bytes of writable memory
/// - The returned pointer is valid for the lifetime of the static string or hexbuf
#[export_name = "coloridx_to_name"]
pub unsafe extern "C" fn rs_coloridx_to_name(
    idx: c_int,
    val: c_int,
    hexbuf: *mut c_char,
) -> *const c_char {
    if idx >= 0 {
        // Valid index into color table (excluding null sentinel)
        let idx_usize = idx as usize;
        let table_len = COLOR_NAME_TABLE.len() - 1; // subtract null sentinel
        if idx_usize < table_len {
            return COLOR_NAME_TABLE[idx_usize].name;
        }
        // Index out of bounds - should not happen, but handle gracefully
        return std::ptr::null();
    }

    match idx {
        COLOR_IDX_NONE => std::ptr::null(),
        COLOR_IDX_FG => FG_STR.as_ptr(),
        COLOR_IDX_BG => BG_STR.as_ptr(),
        COLOR_IDX_HEX => {
            // Format as "#RRGGBB"
            // hexbuf must be at least 8 bytes
            let hex_digits = b"0123456789abcdef";
            let buf = hexbuf as *mut u8;
            *buf = b'#';
            *buf.add(1) = hex_digits[((val >> 20) & 0xF) as usize];
            *buf.add(2) = hex_digits[((val >> 16) & 0xF) as usize];
            *buf.add(3) = hex_digits[((val >> 12) & 0xF) as usize];
            *buf.add(4) = hex_digits[((val >> 8) & 0xF) as usize];
            *buf.add(5) = hex_digits[((val >> 4) & 0xF) as usize];
            *buf.add(6) = hex_digits[(val & 0xF) as usize];
            *buf.add(7) = 0; // null terminator
            hexbuf
        }
        _ => {
            // Unknown index - C code calls abort() here, but we'll return NULL
            // to avoid undefined behavior
            std::ptr::null()
        }
    }
}

// ============================================================================
// hlattrs2dict - Convert HlAttrs to Dict
// ============================================================================

/// Static key strings for hlattrs2dict (zero-copy)
mod hlattrs_keys {
    pub static REVERSE: &[u8] = b"reverse\0";
    pub static BOLD: &[u8] = b"bold\0";
    pub static ITALIC: &[u8] = b"italic\0";
    pub static UNDERLINE: &[u8] = b"underline\0";
    pub static UNDERCURL: &[u8] = b"undercurl\0";
    pub static UNDERDOUBLE: &[u8] = b"underdouble\0";
    pub static UNDERDOTTED: &[u8] = b"underdotted\0";
    pub static UNDERDASHED: &[u8] = b"underdashed\0";
    pub static STANDOUT: &[u8] = b"standout\0";
    pub static STRIKETHROUGH: &[u8] = b"strikethrough\0";
    pub static ALTFONT: &[u8] = b"altfont\0";
    pub static NOCOMBINE: &[u8] = b"nocombine\0";
    pub static FG: &[u8] = b"fg\0";
    pub static BG: &[u8] = b"bg\0";
    pub static SP: &[u8] = b"sp\0";
    pub static FOREGROUND: &[u8] = b"foreground\0";
    pub static BACKGROUND: &[u8] = b"background\0";
    pub static SPECIAL: &[u8] = b"special\0";
    pub static FG_INDEXED: &[u8] = b"fg_indexed\0";
    pub static BG_INDEXED: &[u8] = b"bg_indexed\0";
    pub static CTERMFG: &[u8] = b"ctermfg\0";
    pub static CTERMBG: &[u8] = b"ctermbg\0";
    pub static BLEND: &[u8] = b"blend\0";
}

/// Maximum items in hlattrs dict
const HLATTRS_DICT_SIZE: usize = 16;

/// Convert HlAttrs to Dict representation.
///
/// Equivalent to C's hlattrs2dict function. Outputs highlight attributes as
/// key-value pairs suitable for API responses.
///
/// # Arguments
/// * `hl` - Pre-allocated Dict for colors (must have capacity >= HLATTRS_DICT_SIZE)
/// * `hl_attrs` - Pre-allocated Dict for attributes (can be NULL to use `hl`)
/// * `ae` - The HlAttrs to convert
/// * `use_rgb` - If true, output RGB colors; if false, output cterm colors
/// * `short_keys` - If true, use short keys (fg/bg/sp/ctermfg/ctermbg); if false, use long keys
///
/// # Safety
/// - `hl` must be a valid pointer to a Dict with capacity >= HLATTRS_DICT_SIZE
/// - `hl_attrs` must be NULL or a valid pointer to a Dict with capacity >= HLATTRS_DICT_SIZE
#[export_name = "hlattrs2dict"]
pub unsafe extern "C" fn rs_hlattrs2dict(
    hl: *mut Dict,
    hl_attrs: *mut Dict,
    ae: HlAttrs,
    use_rgb: bool,
    short_keys: bool,
) {
    use hlattrs_keys::*;

    // Use raw pointers to avoid borrow checker issues when hl_attrs == hl
    let hl_ptr = hl;
    let attrs_ptr = if hl_attrs.is_null() { hl } else { hl_attrs };

    debug_assert!((*hl_ptr).capacity >= HLATTRS_DICT_SIZE);
    debug_assert!((*attrs_ptr).capacity >= HLATTRS_DICT_SIZE);

    let mask = if use_rgb {
        ae.rgb_ae_attr
    } else {
        ae.cterm_ae_attr
    };

    // Attribute flags
    if (mask & HL_INVERSE) != 0 {
        (*attrs_ptr).put_static(REVERSE.as_ptr() as *const c_char, Object::boolean(true));
    }

    if (mask & HL_BOLD) != 0 {
        (*attrs_ptr).put_static(BOLD.as_ptr() as *const c_char, Object::boolean(true));
    }

    if (mask & HL_ITALIC) != 0 {
        (*attrs_ptr).put_static(ITALIC.as_ptr() as *const c_char, Object::boolean(true));
    }

    // Underline styles (mutually exclusive)
    match mask & HL_UNDERLINE_MASK {
        x if x == HL_UNDERLINE => {
            (*attrs_ptr).put_static(UNDERLINE.as_ptr() as *const c_char, Object::boolean(true));
        }
        x if x == HL_UNDERCURL => {
            (*attrs_ptr).put_static(UNDERCURL.as_ptr() as *const c_char, Object::boolean(true));
        }
        x if x == HL_UNDERDOUBLE => {
            (*attrs_ptr).put_static(UNDERDOUBLE.as_ptr() as *const c_char, Object::boolean(true));
        }
        x if x == HL_UNDERDOTTED => {
            (*attrs_ptr).put_static(UNDERDOTTED.as_ptr() as *const c_char, Object::boolean(true));
        }
        x if x == HL_UNDERDASHED => {
            (*attrs_ptr).put_static(UNDERDASHED.as_ptr() as *const c_char, Object::boolean(true));
        }
        _ => {}
    }

    if (mask & HL_STANDOUT) != 0 {
        (*attrs_ptr).put_static(STANDOUT.as_ptr() as *const c_char, Object::boolean(true));
    }

    if (mask & HL_STRIKETHROUGH) != 0 {
        (*attrs_ptr).put_static(
            STRIKETHROUGH.as_ptr() as *const c_char,
            Object::boolean(true),
        );
    }

    if (mask & HL_ALTFONT) != 0 {
        (*attrs_ptr).put_static(ALTFONT.as_ptr() as *const c_char, Object::boolean(true));
    }

    if (mask & HL_NOCOMBINE) != 0 {
        (*attrs_ptr).put_static(NOCOMBINE.as_ptr() as *const c_char, Object::boolean(true));
    }

    // Colors
    if use_rgb {
        if ae.rgb_fg_color != -1 {
            let key = if short_keys { FG } else { FOREGROUND };
            (*hl_ptr).put_static(
                key.as_ptr() as *const c_char,
                Object::integer(ae.rgb_fg_color as i64),
            );
        }

        if ae.rgb_bg_color != -1 {
            let key = if short_keys { BG } else { BACKGROUND };
            (*hl_ptr).put_static(
                key.as_ptr() as *const c_char,
                Object::integer(ae.rgb_bg_color as i64),
            );
        }

        if ae.rgb_sp_color != -1 {
            let key = if short_keys { SP } else { SPECIAL };
            (*hl_ptr).put_static(
                key.as_ptr() as *const c_char,
                Object::integer(ae.rgb_sp_color as i64),
            );
        }

        if !short_keys {
            if (mask & HL_FG_INDEXED) != 0 {
                (*hl_ptr).put_static(FG_INDEXED.as_ptr() as *const c_char, Object::boolean(true));
            }

            if (mask & HL_BG_INDEXED) != 0 {
                (*hl_ptr).put_static(BG_INDEXED.as_ptr() as *const c_char, Object::boolean(true));
            }
        }
    } else {
        // cterm colors
        if ae.cterm_fg_color != 0 {
            let key = if short_keys { CTERMFG } else { FOREGROUND };
            (*hl_ptr).put_static(
                key.as_ptr() as *const c_char,
                Object::integer((ae.cterm_fg_color - 1) as i64),
            );
        }

        if ae.cterm_bg_color != 0 {
            let key = if short_keys { CTERMBG } else { BACKGROUND };
            (*hl_ptr).put_static(
                key.as_ptr() as *const c_char,
                Object::integer((ae.cterm_bg_color - 1) as i64),
            );
        }
    }

    // Blend (only for RGB, or for long keys)
    if ae.hl_blend > -1 && (use_rgb || !short_keys) {
        (*hl_ptr).put_static(
            BLEND.as_ptr() as *const c_char,
            Object::integer(ae.hl_blend as i64),
        );
    }
}

// ============================================================================
// hl_inspect - Convert highlight attribute to Array of Dicts
// ============================================================================

extern "C" {
    // nvim_get_hlf_name already declared in main extern block
    fn arena_alloc(arena: *mut Arena, size: usize) -> *mut c_char;
}

/// Static key strings for hl_inspect (zero-copy)
mod hl_inspect_keys {
    pub static KIND: &[u8] = b"kind\0";
    pub static HI_NAME: &[u8] = b"hi_name\0";
    pub static UI_NAME: &[u8] = b"ui_name\0";
    pub static ID: &[u8] = b"id\0";
    pub static SYNTAX: &[u8] = b"syntax\0";
    pub static UI: &[u8] = b"ui\0";
    pub static TERM: &[u8] = b"term\0";
    pub static NORMAL: &[u8] = b"Normal\0";
}

/// Get the size needed for hl_inspect array (recursive for combined attributes)
fn hl_inspect_size(attr: c_int) -> usize {
    let count = rs_attr_entry_count();
    if attr <= 0 || attr >= count {
        return 0;
    }

    let e = rs_get_attr_entry_by_id(attr);
    if e.kind == HlKind::Combine || e.kind == HlKind::Blend || e.kind == HlKind::BlendThrough {
        return hl_inspect_size(e.id1) + hl_inspect_size(e.id2);
    }
    1
}

/// Recursive implementation of hl_inspect
///
/// # Safety
/// - `arr` must be a valid pointer to an Array with sufficient capacity
/// - `arena` must be a valid Arena pointer
unsafe fn hl_inspect_impl(arr: *mut Array, attr: c_int, arena: *mut Arena) {
    use hl_inspect_keys::*;

    let count = rs_attr_entry_count();
    if attr <= 0 || attr >= count {
        return;
    }

    let e = rs_get_attr_entry_by_id(attr);

    match e.kind {
        HlKind::Syntax => {
            // Create dict with 3 items: kind, hi_name, id
            let dict_items = arena_alloc(arena, 3 * std::mem::size_of::<nvim_api::KeyValuePair>())
                as *mut nvim_api::KeyValuePair;
            let mut item = Dict {
                size: 0,
                capacity: 3,
                items: dict_items,
            };

            item.put_static(
                KIND.as_ptr() as *const c_char,
                Object::string(NvimString {
                    data: SYNTAX.as_ptr() as *mut c_char,
                    size: SYNTAX.len() - 1, // exclude null terminator
                }),
            );

            let hi_name = rs_syn_id2name(e.id1);
            item.put_static(
                HI_NAME.as_ptr() as *const c_char,
                Object::string(NvimString {
                    data: hi_name as *mut c_char,
                    size: if hi_name.is_null() {
                        0
                    } else {
                        strlen(hi_name)
                    },
                }),
            );

            item.put_static(ID.as_ptr() as *const c_char, Object::integer(attr as i64));

            (*arr).push(Object::dict(item));
        }

        HlKind::UI => {
            // Create dict with 4 items: kind, ui_name, hi_name, id
            let dict_items = arena_alloc(arena, 4 * std::mem::size_of::<nvim_api::KeyValuePair>())
                as *mut nvim_api::KeyValuePair;
            let mut item = Dict {
                size: 0,
                capacity: 4,
                items: dict_items,
            };

            item.put_static(
                KIND.as_ptr() as *const c_char,
                Object::string(NvimString {
                    data: UI.as_ptr() as *mut c_char,
                    size: UI.len() - 1,
                }),
            );

            // ui_name: "Normal" if id1 == -1, else hlf_names[id1]
            let ui_name = if e.id1 == -1 {
                NORMAL.as_ptr() as *const c_char
            } else {
                nvim_get_hlf_name(e.id1)
            };
            item.put_static(
                UI_NAME.as_ptr() as *const c_char,
                Object::string(NvimString {
                    data: ui_name as *mut c_char,
                    size: if ui_name.is_null() {
                        0
                    } else {
                        strlen(ui_name)
                    },
                }),
            );

            let hi_name = rs_syn_id2name(e.id2);
            item.put_static(
                HI_NAME.as_ptr() as *const c_char,
                Object::string(NvimString {
                    data: hi_name as *mut c_char,
                    size: if hi_name.is_null() {
                        0
                    } else {
                        strlen(hi_name)
                    },
                }),
            );

            item.put_static(ID.as_ptr() as *const c_char, Object::integer(attr as i64));

            (*arr).push(Object::dict(item));
        }

        HlKind::Terminal => {
            // Create dict with 2 items: kind, id
            let dict_items = arena_alloc(arena, 2 * std::mem::size_of::<nvim_api::KeyValuePair>())
                as *mut nvim_api::KeyValuePair;
            let mut item = Dict {
                size: 0,
                capacity: 2,
                items: dict_items,
            };

            item.put_static(
                KIND.as_ptr() as *const c_char,
                Object::string(NvimString {
                    data: TERM.as_ptr() as *mut c_char,
                    size: TERM.len() - 1,
                }),
            );

            item.put_static(ID.as_ptr() as *const c_char, Object::integer(attr as i64));

            (*arr).push(Object::dict(item));
        }

        HlKind::Combine | HlKind::Blend | HlKind::BlendThrough => {
            // Flatten combined attributes recursively
            hl_inspect_impl(arr, e.id1, arena);
            hl_inspect_impl(arr, e.id2, arena);
        }

        HlKind::Unknown | HlKind::Invalid => {
            // Do nothing
        }
    }
}

extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn nvim_get_hlstate_active() -> bool;
}

/// Inspect a highlight attribute and return an Array of Dicts describing its composition.
///
/// This is equivalent to C's `hl_inspect()` function. For combined/blended attributes,
/// it recursively flattens the tree structure into an array.
///
/// # Safety
/// - `arena` must be a valid Arena pointer
#[export_name = "hl_inspect"]
pub unsafe extern "C" fn rs_hl_inspect(attr: c_int, arena: *mut Arena) -> Array {
    if !nvim_get_hlstate_active() {
        return Array::empty();
    }

    let size = hl_inspect_size(attr);
    if size == 0 {
        return Array::empty();
    }

    // Allocate array using arena
    let items = arena_alloc(arena, size * std::mem::size_of::<Object>()) as *mut Object;
    let mut ret = Array {
        size: 0,
        capacity: size,
        items,
    };

    hl_inspect_impl(&mut ret, attr, arena);
    ret
}

// ============================================================================
// object_to_color - Convert API Object to color value
// ============================================================================

extern "C" {
    fn api_set_error(err: *mut Error, err_type: c_int, format: *const c_char, ...);
}

/// Error type constants
const K_ERROR_TYPE_VALIDATION: c_int = 1;

/// Case-insensitive string comparison for C strings
unsafe fn stricmp_none(data: *const c_char, size: usize) -> bool {
    // Check if string is "NONE" (case-insensitive)
    if size != 4 {
        return false;
    }
    let bytes = std::slice::from_raw_parts(data as *const u8, size);
    bytes.eq_ignore_ascii_case(b"NONE")
}

/// Convert an API Object (Integer or String) to a color value.
///
/// For Integer objects, returns the value directly.
/// For String objects, parses the color name or returns -1 for "NONE".
///
/// # Safety
/// - `key` must be a valid null-terminated C string
/// - `err` must be a valid Error pointer
#[export_name = "object_to_color"]
pub unsafe extern "C" fn rs_object_to_color(
    val: Object,
    key: *const c_char,
    rgb: bool,
    err: *mut Error,
) -> c_int {
    match val.obj_type {
        t if t == ObjectType::Integer as c_int => val.data.integer as c_int,

        t if t == ObjectType::String as c_int => {
            let str_val: NvimString = val.data.string;

            // Empty string or "NONE" returns -1
            if str_val.size == 0 {
                return -1;
            }

            // Check for "NONE" (case-insensitive)
            if stricmp_none(str_val.data, str_val.size) {
                return -1;
            }

            // Parse color name
            let color = if rgb {
                // Use name_to_color (returns NameToColorResult)
                let result = rs_name_to_color(str_val.data);
                result.color
            } else {
                // Use name_to_ctermcolor (returns int directly)
                rs_name_to_ctermcolor(str_val.data)
            };

            // Validate color was found
            if color < 0 {
                // "Invalid highlight color: '%s'" - matches VALIDATE_S which quotes the value
                static FMT: &[u8] = b"Invalid highlight color: '%s'\0";
                api_set_error(
                    err,
                    K_ERROR_TYPE_VALIDATION,
                    FMT.as_ptr() as *const c_char,
                    str_val.data,
                );
            }

            color
        }

        _ => {
            // "Invalid '%s': expected String or Integer"
            static FMT: &[u8] = b"Invalid '%s': expected String or Integer\0";
            api_set_error(
                err,
                K_ERROR_TYPE_VALIDATION,
                FMT.as_ptr() as *const c_char,
                key,
            );
            0
        }
    }
}

// ============================================================================
// hl_get_attr_by_id - Get highlight attributes as Dict
// ============================================================================

/// Error type for exceptions
const K_ERROR_TYPE_EXCEPTION: c_int = 0;

/// Gets highlight description for id `attr_id` as a map.
///
/// # Safety
/// - `arena` must be a valid Arena pointer
/// - `err` must be a valid Error pointer
#[export_name = "hl_get_attr_by_id"]
pub unsafe extern "C" fn rs_hl_get_attr_by_id(
    attr_id: i64,
    rgb: bool,
    arena: *mut Arena,
    err: *mut Error,
) -> Dict {
    // Empty dict for attr_id == 0
    if attr_id == 0 {
        return Dict::empty();
    }

    // Validate attr_id range
    let count = rs_attr_entry_count();
    if attr_id <= 0 || attr_id >= count as i64 {
        // "Invalid attribute id: %lld"
        static FMT: &[u8] = b"Invalid attribute id: %lld\0";
        api_set_error(
            err,
            K_ERROR_TYPE_EXCEPTION,
            FMT.as_ptr() as *const c_char,
            attr_id,
        );
        return Dict::empty();
    }

    // Allocate dict with arena
    let items = arena_alloc(
        arena,
        HLATTRS_DICT_SIZE * std::mem::size_of::<nvim_api::KeyValuePair>(),
    ) as *mut nvim_api::KeyValuePair;
    let mut retval = Dict {
        size: 0,
        capacity: HLATTRS_DICT_SIZE,
        items,
    };

    // Get the attributes and convert to dict
    let attrs = rs_syn_attr2entry(attr_id as c_int);
    rs_hlattrs2dict(&mut retval, std::ptr::null_mut(), attrs, rgb, false);

    retval
}

// ============================================================================
// Syntax Functions
// ============================================================================

/// Get the conceal substitution character.
///
/// Returns `current_sub_char` from syntax.c.
///
/// # Safety
/// Calls external C function to access static variable.
#[export_name = "syn_get_sub_char"]
pub unsafe extern "C" fn rs_syn_get_sub_char() -> c_int {
    nvim_syn_get_current_sub_char()
}

extern "C" {
    fn nvim_get_highlight_ga_len() -> c_int;
    fn nvim_hl_table_get_sg_gui(idx: c_int) -> c_int;
    fn nvim_hl_table_get_sg_cterm(idx: c_int) -> c_int;
}

/// Static string "1" for attribute return value
static ATTR_TRUE: &[u8; 2] = b"1\0";

/// Returns the number of highlight groups.
#[export_name = "highlight_num_groups"]
pub unsafe extern "C" fn rs_highlight_num_groups() -> c_int {
    nvim_get_highlight_ga_len()
}

/// Check whether highlight group has attribute.
///
/// # Arguments
/// * `id` - Highlight group ID (1-based)
/// * `flag` - Attribute flag to check
/// * `modec` - 'g' for GUI, 'c' for cterm
///
/// # Returns
/// Pointer to "1" if attribute is set, NULL otherwise
#[export_name = "highlight_has_attr"]
pub unsafe extern "C" fn rs_highlight_has_attr(
    id: c_int,
    flag: c_int,
    modec: c_int,
) -> *const c_char {
    use hl_attr_flags::HL_UNDERLINE_MASK;
    let ul_mask = c_int::from(HL_UNDERLINE_MASK);

    if id <= 0 || id > nvim_get_highlight_ga_len() {
        return std::ptr::null();
    }

    let idx = id - 1;
    let attr = if modec == b'g' as c_int {
        nvim_hl_table_get_sg_gui(idx)
    } else {
        nvim_hl_table_get_sg_cterm(idx)
    };

    if (flag & ul_mask) != 0 {
        let ul = attr & ul_mask;
        if ul == flag {
            ATTR_TRUE.as_ptr().cast()
        } else {
            std::ptr::null()
        }
    } else if (attr & flag) != 0 {
        ATTR_TRUE.as_ptr().cast()
    } else {
        std::ptr::null()
    }
}

// ============================================================================
// dict2hlattrs - Parse raw Dict into HlAttrs
// ============================================================================

/// Result of dict2hlattrs_full, returning all parsed information.
#[repr(C)]
pub struct Dict2HlAttrsResult {
    pub attrs: HlAttrs,
    pub link_id: c_int,
    pub fallback: bool,
    pub has_error: bool,
    /// Raw value of the "fallback" key (0 if absent, 1 if explicitly true).
    /// Used as version_offset in ns_get_hl: version = hl_valid - version_offset.
    pub version_offset: c_int,
}

/// Helper: check if error is set
unsafe fn error_is_set(err: *const Error) -> bool {
    !err.is_null() && (*err).err_type != -1 // kErrorTypeNone = -1
}

/// Helper: match a key name against a byte string
unsafe fn key_eq(key: &NvimString, name: &[u8]) -> bool {
    if key.size != name.len() {
        return false;
    }
    if key.data.is_null() {
        return false;
    }
    let key_bytes = std::slice::from_raw_parts(key.data as *const u8, key.size);
    key_bytes == name
}

/// Helper: get boolean value from an Object (expects kObjectTypeBoolean)
unsafe fn obj_get_bool(obj: &Object) -> bool {
    obj.data.boolean
}

/// Helper: get integer value from an Object (expects kObjectTypeInteger)
unsafe fn obj_get_int(obj: &Object) -> i64 {
    obj.data.integer
}

/// Helper: apply a boolean flag to a mask
fn apply_flag(mask: &mut i16, flag: i16, value: bool) {
    if value {
        if flag & HL_UNDERLINE_MASK != 0 {
            *mask &= !HL_UNDERLINE_MASK;
        }
        *mask |= flag;
    }
}

/// Parse a raw Dict into HlAttrs.
///
/// This is the Rust implementation of C's dict2hlattrs. It iterates over
/// raw Dict key-value pairs and matches key strings directly.
///
/// # Safety
/// - `dict` must contain valid Dict data (items pointer valid for size elements)
/// - `err` must be a valid Error pointer
unsafe fn dict2hlattrs_impl(
    dict: Dict,
    use_rgb: bool,
    allow_link: bool,
    err: *mut Error,
) -> Dict2HlAttrsResult {
    let mut hlattrs = HlAttrs::new();
    let mut result = Dict2HlAttrsResult {
        attrs: hlattrs,
        link_id: -1,
        fallback: true,
        has_error: false,
        version_offset: 0,
    };

    let mut fg: i32 = -1;
    let mut bg: i32 = -1;
    let mut ctermfg: i32 = -1;
    let mut ctermbg: i32 = -1;
    let mut sp: i32 = -1;
    let mut blend: i32 = -1;
    let mut mask: i16 = 0;
    let mut cterm_mask: i16 = 0;
    let mut cterm_mask_provided = false;
    let mut has_fallback_key = false;

    if dict.items.is_null() || dict.size == 0 {
        return result;
    }

    let items = std::slice::from_raw_parts(dict.items, dict.size);

    for kv in items {
        let key = &kv.key;
        let val = &kv.value;

        // Boolean flags
        if key_eq(key, b"bold") {
            apply_flag(&mut mask, HL_BOLD, obj_get_bool(val));
        } else if key_eq(key, b"italic") {
            apply_flag(&mut mask, HL_ITALIC, obj_get_bool(val));
        } else if key_eq(key, b"reverse") {
            apply_flag(&mut mask, HL_INVERSE, obj_get_bool(val));
        } else if key_eq(key, b"standout") {
            apply_flag(&mut mask, HL_STANDOUT, obj_get_bool(val));
        } else if key_eq(key, b"strikethrough") {
            apply_flag(&mut mask, HL_STRIKETHROUGH, obj_get_bool(val));
        } else if key_eq(key, b"underline") {
            apply_flag(&mut mask, HL_UNDERLINE, obj_get_bool(val));
        } else if key_eq(key, b"undercurl") {
            apply_flag(&mut mask, HL_UNDERCURL, obj_get_bool(val));
        } else if key_eq(key, b"underdouble") {
            apply_flag(&mut mask, HL_UNDERDOUBLE, obj_get_bool(val));
        } else if key_eq(key, b"underdotted") {
            apply_flag(&mut mask, HL_UNDERDOTTED, obj_get_bool(val));
        } else if key_eq(key, b"underdashed") {
            apply_flag(&mut mask, HL_UNDERDASHED, obj_get_bool(val));
        } else if key_eq(key, b"altfont") {
            apply_flag(&mut mask, HL_ALTFONT, obj_get_bool(val));
        } else if key_eq(key, b"nocombine") {
            apply_flag(&mut mask, HL_NOCOMBINE, obj_get_bool(val));
        } else if key_eq(key, b"default") {
            apply_flag(&mut mask, HL_DEFAULT, obj_get_bool(val));
        } else if key_eq(key, b"fg_indexed") {
            if use_rgb {
                apply_flag(&mut mask, HL_FG_INDEXED, obj_get_bool(val));
            }
        } else if key_eq(key, b"bg_indexed") {
            if use_rgb {
                apply_flag(&mut mask, HL_BG_INDEXED, obj_get_bool(val));
            }
        }
        // Color fields
        else if key_eq(key, b"fg") || key_eq(key, b"foreground") {
            static KEY_FG: &[u8] = b"fg\0";
            static KEY_FOREGROUND: &[u8] = b"foreground\0";
            let key_name = if key_eq(key, b"fg") {
                KEY_FG.as_ptr() as *mut c_char
            } else {
                KEY_FOREGROUND.as_ptr() as *mut c_char
            };
            fg = rs_object_to_color(*val, key_name, use_rgb, err);
            if error_is_set(err) {
                result.has_error = true;
                return result;
            }
        } else if key_eq(key, b"bg") || key_eq(key, b"background") {
            static KEY_BG: &[u8] = b"bg\0";
            static KEY_BACKGROUND: &[u8] = b"background\0";
            let key_name = if key_eq(key, b"bg") {
                KEY_BG.as_ptr() as *mut c_char
            } else {
                KEY_BACKGROUND.as_ptr() as *mut c_char
            };
            bg = rs_object_to_color(*val, key_name, use_rgb, err);
            if error_is_set(err) {
                result.has_error = true;
                return result;
            }
        } else if key_eq(key, b"sp") || key_eq(key, b"special") {
            static KEY_SP: &[u8] = b"sp\0";
            static KEY_SPECIAL: &[u8] = b"special\0";
            let key_name = if key_eq(key, b"sp") {
                KEY_SP.as_ptr() as *mut c_char
            } else {
                KEY_SPECIAL.as_ptr() as *mut c_char
            };
            sp = rs_object_to_color(*val, key_name, true, err);
            if error_is_set(err) {
                result.has_error = true;
                return result;
            }
        }
        // Integer fields
        else if key_eq(key, b"blend") {
            let blend0 = obj_get_int(val);
            if blend0 < 0 || blend0 > 100 {
                static FMT: &[u8] = b"Invalid 'blend': expected Integer in range [0, 100]\0";
                api_set_error(err, K_ERROR_TYPE_VALIDATION, FMT.as_ptr() as *const c_char);
                result.has_error = true;
                return result;
            }
            blend = blend0 as i32;
        } else if key_eq(key, b"link") {
            if !allow_link {
                static FMT: &[u8] = b"Invalid Key: 'link'\0";
                api_set_error(err, K_ERROR_TYPE_VALIDATION, FMT.as_ptr() as *const c_char);
                result.has_error = true;
                return result;
            }
            result.link_id = obj_get_int(val) as c_int;
        } else if key_eq(key, b"global_link") {
            if !allow_link {
                static FMT: &[u8] = b"Invalid Key: 'global_link'\0";
                api_set_error(err, K_ERROR_TYPE_VALIDATION, FMT.as_ptr() as *const c_char);
                result.has_error = true;
                return result;
            }
            result.link_id = obj_get_int(val) as c_int;
            mask |= HL_GLOBAL;
        }
        // Cterm dict
        else if key_eq(key, b"cterm") {
            if val.obj_type == ObjectType::Dict as c_int {
                cterm_mask_provided = true;
                let cterm_dict = val.data.dict;
                if !cterm_dict.items.is_null() && cterm_dict.size > 0 {
                    let cterm_items = std::slice::from_raw_parts(cterm_dict.items, cterm_dict.size);
                    for ckv in cterm_items {
                        let ckey = &ckv.key;
                        let cval = &ckv.value;
                        if key_eq(ckey, b"bold") {
                            apply_flag(&mut cterm_mask, HL_BOLD, obj_get_bool(cval));
                        } else if key_eq(ckey, b"italic") {
                            apply_flag(&mut cterm_mask, HL_ITALIC, obj_get_bool(cval));
                        } else if key_eq(ckey, b"reverse") {
                            apply_flag(&mut cterm_mask, HL_INVERSE, obj_get_bool(cval));
                        } else if key_eq(ckey, b"standout") {
                            apply_flag(&mut cterm_mask, HL_STANDOUT, obj_get_bool(cval));
                        } else if key_eq(ckey, b"strikethrough") {
                            apply_flag(&mut cterm_mask, HL_STRIKETHROUGH, obj_get_bool(cval));
                        } else if key_eq(ckey, b"underline") {
                            apply_flag(&mut cterm_mask, HL_UNDERLINE, obj_get_bool(cval));
                        } else if key_eq(ckey, b"undercurl") {
                            apply_flag(&mut cterm_mask, HL_UNDERCURL, obj_get_bool(cval));
                        } else if key_eq(ckey, b"underdouble") {
                            apply_flag(&mut cterm_mask, HL_UNDERDOUBLE, obj_get_bool(cval));
                        } else if key_eq(ckey, b"underdotted") {
                            apply_flag(&mut cterm_mask, HL_UNDERDOTTED, obj_get_bool(cval));
                        } else if key_eq(ckey, b"underdashed") {
                            apply_flag(&mut cterm_mask, HL_UNDERDASHED, obj_get_bool(cval));
                        } else if key_eq(ckey, b"altfont") {
                            apply_flag(&mut cterm_mask, HL_ALTFONT, obj_get_bool(cval));
                        } else if key_eq(ckey, b"nocombine") {
                            apply_flag(&mut cterm_mask, HL_NOCOMBINE, obj_get_bool(cval));
                        }
                    }
                }
            } else if val.obj_type == ObjectType::Array as c_int && val.data.array.size == 0 {
                // empty list from Lua API should clear all cterm attributes
                cterm_mask_provided = true;
            } else {
                static FMT: &[u8] = b"Invalid 'cterm': expected Dict, got %s\0";
                api_set_error(
                    err,
                    K_ERROR_TYPE_VALIDATION,
                    FMT.as_ptr() as *const c_char,
                    nvim_api::rs_api_typename(val.obj_type),
                );
                result.has_error = true;
                return result;
            }
        }
        // Cterm colors
        else if key_eq(key, b"ctermfg") {
            static KEY_CTERMFG: &[u8] = b"ctermfg\0";
            ctermfg = rs_object_to_color(*val, KEY_CTERMFG.as_ptr() as *mut c_char, false, err);
            if error_is_set(err) {
                result.has_error = true;
                return result;
            }
        } else if key_eq(key, b"ctermbg") {
            static KEY_CTERMBG: &[u8] = b"ctermbg\0";
            ctermbg = rs_object_to_color(*val, KEY_CTERMBG.as_ptr() as *mut c_char, false, err);
            if error_is_set(err) {
                result.has_error = true;
                return result;
            }
        }
        // Fallback flag (used by ns_get_hl)
        else if key_eq(key, b"fallback") {
            has_fallback_key = true;
            let fb = obj_get_bool(val);
            result.fallback = fb;
            result.version_offset = fb as c_int;
        }
        // Ignore keys we don't handle (url, force, etc.)
    }

    // Build final HlAttrs
    if use_rgb {
        if !cterm_mask_provided {
            cterm_mask = mask;
        }
        hlattrs.rgb_ae_attr = mask;
        hlattrs.rgb_bg_color = bg;
        hlattrs.rgb_fg_color = fg;
        hlattrs.rgb_sp_color = sp;
        hlattrs.hl_blend = blend;
        hlattrs.cterm_bg_color = if ctermbg == -1 {
            0
        } else {
            (ctermbg + 1) as i16
        };
        hlattrs.cterm_fg_color = if ctermfg == -1 {
            0
        } else {
            (ctermfg + 1) as i16
        };
        hlattrs.cterm_ae_attr = cterm_mask;
    } else {
        hlattrs.cterm_bg_color = if bg == -1 { 0 } else { (bg + 1) as i16 };
        hlattrs.cterm_fg_color = if fg == -1 { 0 } else { (fg + 1) as i16 };
        hlattrs.cterm_ae_attr = mask;
    }

    // If link_id was set and is >= 0, fallback should be true
    // (matches C: if (link_id >= 0) fallback = true)
    if result.link_id >= 0 && !has_fallback_key {
        result.fallback = true;
    }

    result.attrs = hlattrs;
    result
}

/// Parse a raw Dict into HlAttrs. FFI entry point.
///
/// Returns HlAttrs parsed from the dict. link_id is written to *link_id_out
/// if non-null. Error information is written to *err.
///
/// # Safety
/// - All pointers must be valid
#[no_mangle]
pub unsafe extern "C" fn rs_dict2hlattrs(
    dict: Dict,
    use_rgb: bool,
    link_id_out: *mut c_int,
    err: *mut Error,
) -> HlAttrs {
    let allow_link = !link_id_out.is_null();
    let result = dict2hlattrs_impl(dict, use_rgb, allow_link, err);
    if !link_id_out.is_null() {
        *link_id_out = result.link_id;
    }
    result.attrs
}

/// Parse a raw Dict into HlAttrs with full result (for ns_get_hl).
///
/// Returns Dict2HlAttrsResult with attrs, link_id, fallback flag, and error status.
///
/// # Safety
/// - All pointers must be valid
#[no_mangle]
pub unsafe extern "C" fn rs_dict2hlattrs_full(
    dict: Dict,
    use_rgb: bool,
    allow_link: bool,
    err: *mut Error,
) -> Dict2HlAttrsResult {
    dict2hlattrs_impl(dict, use_rgb, allow_link, err)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_blend_full_first() {
        // 100% of rgb1
        assert_eq!(rs_rgb_blend(100, 0xFF0000, 0x00FF00), 0xFF0000);
    }

    #[test]
    fn test_rgb_blend_full_second() {
        // 0% of rgb1 = 100% of rgb2
        assert_eq!(rs_rgb_blend(0, 0xFF0000, 0x00FF00), 0x00FF00);
    }

    #[test]
    fn test_rgb_blend_half() {
        // 50% blend of red and green
        let result = rs_rgb_blend(50, 0xFF0000, 0x00FF00);
        let r = (result >> 16) & 0xFF;
        let g = (result >> 8) & 0xFF;
        let b = result & 0xFF;
        // Should be roughly half of each
        assert_eq!(r, 127); // 255 * 0.5 = 127
        assert_eq!(g, 127); // 255 * 0.5 = 127
        assert_eq!(b, 0);
    }

    #[test]
    fn test_cterm2rgb_ansi_black() {
        assert_eq!(rs_hl_cterm2rgb_color(0), 0x000000);
    }

    #[test]
    fn test_cterm2rgb_ansi_red() {
        assert_eq!(rs_hl_cterm2rgb_color(1), 0xE00000);
    }

    #[test]
    fn test_cterm2rgb_ansi_white() {
        assert_eq!(rs_hl_cterm2rgb_color(15), 0xFFFFFF);
    }

    #[test]
    fn test_cterm2rgb_cube_start() {
        // Color 16 is the first color cube entry (0x00, 0x00, 0x00)
        assert_eq!(rs_hl_cterm2rgb_color(16), 0x000000);
    }

    #[test]
    fn test_cterm2rgb_cube_red() {
        // Color 196 is pure red in the cube (5*36 + 16 = 196)
        assert_eq!(rs_hl_cterm2rgb_color(196), 0xFF0000);
    }

    #[test]
    fn test_cterm2rgb_grey_ramp() {
        // Color 232 is first grey (0x08)
        assert_eq!(rs_hl_cterm2rgb_color(232), 0x080808);
        // Color 255 is last grey (0xEE)
        assert_eq!(rs_hl_cterm2rgb_color(255), 0xEEEEEE);
    }

    #[test]
    fn test_rgb2cterm_black() {
        // Pure black should map to color cube entry 16
        assert_eq!(rs_hl_rgb2cterm_color(0x000000), 16);
    }

    #[test]
    fn test_rgb2cterm_white() {
        // Pure white should map to highest color cube entry
        // 5*36 + 5*6 + 5 + 16 = 231
        assert_eq!(rs_hl_rgb2cterm_color(0xFFFFFF), 231);
    }

    #[test]
    fn test_cterm_blend() {
        // Blending same color should return same color
        let c = rs_cterm_blend(50, 196, 196);
        // Result might not be exactly 196 due to conversion losses
        // but should be close (pure red area)
        assert!(c >= 190 && c <= 200);
    }

    // Tests for color name lookup (unit tests that don't depend on C)
    #[test]
    fn test_str_icmp_equal() {
        assert!(str_icmp("Black", "black"));
        assert!(str_icmp("BLACK", "black"));
        assert!(str_icmp("DarkBlue", "DARKBLUE"));
    }

    #[test]
    fn test_str_icmp_not_equal() {
        assert!(!str_icmp("Black", "White"));
        assert!(!str_icmp("Black", "Blac"));
        assert!(!str_icmp("Black", "Blackx"));
    }

    #[test]
    fn test_lookup_color_256() {
        // Black at index 0 should be 0 for 256 colors
        assert_eq!(lookup_color(0, 256), 0);
        // Blue at index 14 should be 12 for 256 colors
        assert_eq!(lookup_color(14, 256), 12);
        // NONE at index 27 should be -1
        assert_eq!(lookup_color(27, 256), -1);
    }

    #[test]
    fn test_lookup_color_16() {
        // Black at index 0 should be 0 for 16 colors
        assert_eq!(lookup_color(0, 16), 0);
        // DarkBlue at index 1 should be 4 for 16 colors (from _8 table)
        assert_eq!(lookup_color(1, 16), 4);
    }

    #[test]
    fn test_lookup_color_8() {
        // Black at index 0 should be 0 for 8 colors
        assert_eq!(lookup_color(0, 8), 0);
        // DarkGray at index 12 should be 0 (8 & 7 = 0) for 8 colors
        assert_eq!(lookup_color(12, 8), 0);
    }

    // ============================================================================
    // Attribute Entry Tests
    // ============================================================================

    #[test]
    fn test_highlight_init() {
        // Clear and reinit
        rs_clear_hl_tables(true);
        // After init, we should have 1 entry (the dummy at index 0)
        assert_eq!(rs_attr_entry_count(), 1);
    }

    #[test]
    fn test_syn_attr2entry_invalid() {
        // Invalid attr should return default HlAttrs
        let attrs = rs_syn_attr2entry(-1);
        assert_eq!(attrs.rgb_fg_color, -1);
        assert_eq!(attrs.rgb_bg_color, -1);
        assert_eq!(attrs.hl_blend, -1);
    }

    #[test]
    fn test_syn_attr2entry_zero() {
        rs_clear_hl_tables(true);
        // Attr 0 is the dummy entry
        let attrs = rs_syn_attr2entry(0);
        assert_eq!(attrs.rgb_fg_color, -1);
        assert_eq!(attrs.rgb_bg_color, -1);
    }

    #[test]
    fn test_get_attr_entry_new() {
        rs_clear_hl_tables(true);
        let entry = HlEntry {
            attr: HlAttrs {
                rgb_fg_color: 0xFF0000,
                rgb_bg_color: 0x00FF00,
                ..HlAttrs::new()
            },
            kind: HlKind::UI,
            id1: 1,
            id2: 0,
            winid: 0,
        };
        let result = rs_get_attr_entry(entry);
        assert!(result.id > 0);
        assert!(result.is_new);
    }

    #[test]
    fn test_get_attr_entry_existing() {
        rs_clear_hl_tables(true);
        let entry = HlEntry {
            attr: HlAttrs {
                rgb_fg_color: 0x123456,
                rgb_bg_color: 0x654321,
                ..HlAttrs::new()
            },
            kind: HlKind::Syntax,
            id1: 5,
            id2: 0,
            winid: 0,
        };
        let result1 = rs_get_attr_entry(entry);
        assert!(result1.is_new);
        let result2 = rs_get_attr_entry(entry);
        assert!(!result2.is_new);
        assert_eq!(result1.id, result2.id);
    }

    #[test]
    fn test_combine_cache() {
        rs_clear_hl_tables(true);
        let tag = (5 << 16) + 3;
        // Initially not in cache
        assert_eq!(rs_combine_cache_get(tag), -1);
        // Put in cache
        rs_combine_cache_put(tag, 42);
        // Now should be found
        assert_eq!(rs_combine_cache_get(tag), 42);
    }

    #[test]
    fn test_blend_cache() {
        rs_clear_hl_tables(true);
        let tag = (10 << 16) + 7;
        // Test non-through cache
        assert_eq!(rs_blend_cache_get(tag, false), -1);
        rs_blend_cache_put(tag, 99, false);
        assert_eq!(rs_blend_cache_get(tag, false), 99);
        // Through cache should still be empty
        assert_eq!(rs_blend_cache_get(tag, true), -1);
        // Add to through cache
        rs_blend_cache_put(tag, 88, true);
        assert_eq!(rs_blend_cache_get(tag, true), 88);
    }

    #[test]
    fn test_invalidate_blends() {
        rs_clear_hl_tables(true);
        let tag = (20 << 16) + 15;
        rs_blend_cache_put(tag, 77, false);
        rs_blend_cache_put(tag, 66, true);
        assert_eq!(rs_blend_cache_get(tag, false), 77);
        assert_eq!(rs_blend_cache_get(tag, true), 66);
        // Invalidate
        rs_hl_invalidate_blends();
        // Both caches should be empty now
        assert_eq!(rs_blend_cache_get(tag, false), -1);
        assert_eq!(rs_blend_cache_get(tag, true), -1);
    }

    #[test]
    fn test_hlkind_values() {
        // Ensure enum values match C
        assert_eq!(HlKind::Unknown as i32, 0);
        assert_eq!(HlKind::UI as i32, 1);
        assert_eq!(HlKind::Syntax as i32, 2);
        assert_eq!(HlKind::Terminal as i32, 3);
        assert_eq!(HlKind::Combine as i32, 4);
        assert_eq!(HlKind::Blend as i32, 5);
        assert_eq!(HlKind::BlendThrough as i32, 6);
        assert_eq!(HlKind::Invalid as i32, 7);
    }
}
