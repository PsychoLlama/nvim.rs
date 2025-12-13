//! Highlight and color manipulation functions for Neovim
//!
//! This crate provides color blending and conversion functions used by the
//! highlight system. It also manages the highlight attribute entry table
//! that maps attribute IDs to their properties.

use std::collections::HashMap;
use std::ffi::{c_char, c_int, CStr};
use std::sync::{LazyLock, Mutex};

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

    // Highlight functions that need C interop
    /// Get or create a syntax attribute entry
    fn hl_get_syn_attr(ns_id: c_int, idx: c_int, at_en: HlAttrs) -> c_int;
    /// Set need_highlight_changed global
    fn nvim_set_need_highlight_changed(value: bool);
    /// Call update_ns_hl to refresh namespace highlight attributes
    fn nvim_update_ns_hl(ns_id: c_int);

    // hl_table (HlGroup array) accessors from highlight_group.c
    /// Get the number of highlight groups
    fn highlight_num_groups() -> c_int;
    /// Get the link target ID of a highlight group (0-based index)
    fn highlight_link_id(id: c_int) -> c_int;
    /// Get the attribute ID (sg_attr) of a highlight group (0-based index)
    fn highlight_group_attr(id: c_int) -> c_int;
    /// Check if a highlight group has been cleared (0-based index)
    fn highlight_group_cleared(id: c_int) -> bool;
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
pub const HLF_COUNT: usize = 78;

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
#[no_mangle]
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
#[no_mangle]
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
pub extern "C" fn rs_ns_hl_def(
    ns_id: c_int,
    hl_id: c_int,
    attrs: HlAttrs,
    link_id: c_int,
) -> bool {
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
        unsafe { hl_get_syn_attr(ns_id, hl_id, attrs) }
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
        unsafe { hl_get_syn_attr(ns_id, hl_id, attrs) }
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
#[no_mangle]
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
        // Call C to update namespace highlights (involves Lua/syntax lookups)
        unsafe { nvim_update_ns_hl(ns) };

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
#[no_mangle]
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
        new_en.rgb_ae_attr &=
            (!HL_FG_INDEXED) | (prim_aep.rgb_ae_attr & HL_FG_INDEXED);
    }

    // Override cterm background color if primary has one
    if prim_aep.cterm_bg_color > 0 {
        new_en.cterm_bg_color = prim_aep.cterm_bg_color;
        new_en.rgb_ae_attr &=
            (!HL_BG_INDEXED) | (prim_aep.rgb_ae_attr & HL_BG_INDEXED);
    }

    // Override rgb foreground color if primary has one
    if prim_aep.rgb_fg_color >= 0 {
        new_en.rgb_fg_color = prim_aep.rgb_fg_color;
        new_en.rgb_ae_attr &=
            (!HL_FG_INDEXED) | (prim_aep.rgb_ae_attr & HL_FG_INDEXED);
    }

    // Override rgb background color if primary has one
    if prim_aep.rgb_bg_color >= 0 {
        new_en.rgb_bg_color = prim_aep.rgb_bg_color;
        new_en.rgb_ae_attr &=
            (!HL_BG_INDEXED) | (prim_aep.rgb_ae_attr & HL_BG_INDEXED);
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

/// Internal cterm to RGB conversion
fn cterm2rgb_internal(nr: c_int) -> c_int {
    if nr < 16 {
        let entry = &ANSI_TABLE[nr as usize];
        return (c_int::from(entry[0]) << 16) | (c_int::from(entry[1]) << 8) | c_int::from(entry[2]);
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
fn rgb2cterm_internal(rgb: c_int) -> i16 {
    let r = ((rgb >> 16) & 0xFF) as i32;
    let g = ((rgb >> 8) & 0xFF) as i32;
    let b = (rgb & 0xFF) as i32;

    // Check for grey
    if r == g && g == b {
        // Find closest grey
        if r < 4 {
            return 16; // black in cube
        }
        if r > 243 {
            return 231; // white in cube
        }
        // Find closest grey ramp value
        let grey_idx = ((r - 8) / 10) as usize;
        if grey_idx < 24 {
            return (232 + grey_idx) as i16;
        }
    }

    // Find closest color cube value
    let r_idx = closest_cube_idx(r);
    let g_idx = closest_cube_idx(g);
    let b_idx = closest_cube_idx(b);

    (16 + r_idx * 36 + g_idx * 6 + b_idx) as i16
}

/// Find closest 6x6x6 color cube index for a component value
fn closest_cube_idx(val: i32) -> i32 {
    if val < 48 {
        0
    } else if val < 115 {
        1
    } else if val < 155 {
        2
    } else if val < 195 {
        3
    } else if val < 235 {
        4
    } else {
        5
    }
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
    [0, 0, 0, 1],         // black
    [224, 0, 0, 2],       // dark red
    [0, 224, 0, 3],       // dark green
    [224, 224, 0, 4],     // dark yellow / brown
    [0, 0, 224, 5],       // dark blue
    [224, 0, 224, 6],     // dark magenta
    [0, 224, 224, 7],     // dark cyan
    [224, 224, 224, 8],   // light grey
    [128, 128, 128, 9],   // dark grey
    [255, 64, 64, 10],    // light red
    [64, 255, 64, 11],    // light green
    [255, 255, 64, 12],   // light yellow
    [64, 64, 255, 13],    // light blue
    [255, 64, 255, 14],   // light magenta
    [64, 255, 255, 15],   // light cyan
    [255, 255, 255, 16],  // white
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
        return (c_int::from(entry[0]) << 16) | (c_int::from(entry[1]) << 8) | c_int::from(entry[2]);
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
    if idx < 0 || idx >= 28 {
        return LookupColorResult { color: -1, bold: -1 };
    }
    let idx = idx as usize;

    let t_colors = nvim_get_t_colors();

    // Use the _16 table to check if it's a valid color name.
    let color_16 = COLOR_NUMBERS_16[idx];
    if color_16 < 0 {
        return LookupColorResult { color: -1, bold: -1 };
    }

    if t_colors == 8 {
        // t_Co is 8: use the 8 colors table
        let color = COLOR_NUMBERS_8[idx];
        let bold = if foreground {
            // set/reset bold attribute to get light foreground
            // colors (on some terminals, e.g. "linux")
            if color & 8 != 0 { 1 } else { 0 }
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
#[no_mangle]
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

/// RGB color name table - 707 entries from rgb.txt
/// Names are sorted alphabetically for binary search
const RGB_COLOR_NAME_TABLE: &[(&str, c_int)] = &[
    ("AliceBlue", 0xf0f8ff),
    ("AntiqueWhite", 0xfaebd7),
    ("AntiqueWhite1", 0xffefdb),
    ("AntiqueWhite2", 0xeedfcc),
    ("AntiqueWhite3", 0xcdc0b0),
    ("AntiqueWhite4", 0x8b8378),
    ("Aqua", 0x00ffff),
    ("Aquamarine", 0x7fffd4),
    ("Aquamarine1", 0x7fffd4),
    ("Aquamarine2", 0x76eec6),
    ("Aquamarine3", 0x66cdaa),
    ("Aquamarine4", 0x458b74),
    ("Azure", 0xf0ffff),
    ("Azure1", 0xf0ffff),
    ("Azure2", 0xe0eeee),
    ("Azure3", 0xc1cdcd),
    ("Azure4", 0x838b8b),
    ("Beige", 0xf5f5dc),
    ("Bisque", 0xffe4c4),
    ("Bisque1", 0xffe4c4),
    ("Bisque2", 0xeed5b7),
    ("Bisque3", 0xcdb79e),
    ("Bisque4", 0x8b7d6b),
    ("Black", 0x000000),
    ("BlanchedAlmond", 0xffebcd),
    ("Blue", 0x0000ff),
    ("Blue1", 0x0000ff),
    ("Blue2", 0x0000ee),
    ("Blue3", 0x0000cd),
    ("Blue4", 0x00008b),
    ("BlueViolet", 0x8a2be2),
    ("Brown", 0xa52a2a),
    ("Brown1", 0xff4040),
    ("Brown2", 0xee3b3b),
    ("Brown3", 0xcd3333),
    ("Brown4", 0x8b2323),
    ("BurlyWood", 0xdeb887),
    ("Burlywood1", 0xffd39b),
    ("Burlywood2", 0xeec591),
    ("Burlywood3", 0xcdaa7d),
    ("Burlywood4", 0x8b7355),
    ("CadetBlue", 0x5f9ea0),
    ("CadetBlue1", 0x98f5ff),
    ("CadetBlue2", 0x8ee5ee),
    ("CadetBlue3", 0x7ac5cd),
    ("CadetBlue4", 0x53868b),
    ("ChartReuse", 0x7fff00),
    ("Chartreuse1", 0x7fff00),
    ("Chartreuse2", 0x76ee00),
    ("Chartreuse3", 0x66cd00),
    ("Chartreuse4", 0x458b00),
    ("Chocolate", 0xd2691e),
    ("Chocolate1", 0xff7f24),
    ("Chocolate2", 0xee7621),
    ("Chocolate3", 0xcd661d),
    ("Chocolate4", 0x8b4513),
    ("Coral", 0xff7f50),
    ("Coral1", 0xff7256),
    ("Coral2", 0xee6a50),
    ("Coral3", 0xcd5b45),
    ("Coral4", 0x8b3e2f),
    ("CornFlowerBlue", 0x6495ed),
    ("Cornsilk", 0xfff8dc),
    ("Cornsilk1", 0xfff8dc),
    ("Cornsilk2", 0xeee8cd),
    ("Cornsilk3", 0xcdc8b1),
    ("Cornsilk4", 0x8b8878),
    ("Crimson", 0xdc143c),
    ("Cyan", 0x00ffff),
    ("Cyan1", 0x00ffff),
    ("Cyan2", 0x00eeee),
    ("Cyan3", 0x00cdcd),
    ("Cyan4", 0x008b8b),
    ("DarkBlue", 0x00008b),
    ("DarkCyan", 0x008b8b),
    ("DarkGoldenrod", 0xb8860b),
    ("DarkGoldenrod1", 0xffb90f),
    ("DarkGoldenrod2", 0xeead0e),
    ("DarkGoldenrod3", 0xcd950c),
    ("DarkGoldenrod4", 0x8b6508),
    ("DarkGray", 0xa9a9a9),
    ("DarkGreen", 0x006400),
    ("DarkGrey", 0xa9a9a9),
    ("DarkKhaki", 0xbdb76b),
    ("DarkMagenta", 0x8b008b),
    ("DarkOliveGreen", 0x556b2f),
    ("DarkOliveGreen1", 0xcaff70),
    ("DarkOliveGreen2", 0xbcee68),
    ("DarkOliveGreen3", 0xa2cd5a),
    ("DarkOliveGreen4", 0x6e8b3d),
    ("DarkOrange", 0xff8c00),
    ("DarkOrange1", 0xff7f00),
    ("DarkOrange2", 0xee7600),
    ("DarkOrange3", 0xcd6600),
    ("DarkOrange4", 0x8b4500),
    ("DarkOrchid", 0x9932cc),
    ("DarkOrchid1", 0xbf3eff),
    ("DarkOrchid2", 0xb23aee),
    ("DarkOrchid3", 0x9a32cd),
    ("DarkOrchid4", 0x68228b),
    ("DarkRed", 0x8b0000),
    ("DarkSalmon", 0xe9967a),
    ("DarkSeaGreen", 0x8fbc8f),
    ("DarkSeaGreen1", 0xc1ffc1),
    ("DarkSeaGreen2", 0xb4eeb4),
    ("DarkSeaGreen3", 0x9bcd9b),
    ("DarkSeaGreen4", 0x698b69),
    ("DarkSlateBlue", 0x483d8b),
    ("DarkSlateGray", 0x2f4f4f),
    ("DarkSlateGray1", 0x97ffff),
    ("DarkSlateGray2", 0x8deeee),
    ("DarkSlateGray3", 0x79cdcd),
    ("DarkSlateGray4", 0x528b8b),
    ("DarkSlateGrey", 0x2f4f4f),
    ("DarkTurquoise", 0x00ced1),
    ("DarkViolet", 0x9400d3),
    ("DarkYellow", 0xbbbb00),
    ("DeepPink", 0xff1493),
    ("DeepPink1", 0xff1493),
    ("DeepPink2", 0xee1289),
    ("DeepPink3", 0xcd1076),
    ("DeepPink4", 0x8b0a50),
    ("DeepSkyBlue", 0x00bfff),
    ("DeepSkyBlue1", 0x00bfff),
    ("DeepSkyBlue2", 0x00b2ee),
    ("DeepSkyBlue3", 0x009acd),
    ("DeepSkyBlue4", 0x00688b),
    ("DimGray", 0x696969),
    ("DimGrey", 0x696969),
    ("DodgerBlue", 0x1e90ff),
    ("DodgerBlue1", 0x1e90ff),
    ("DodgerBlue2", 0x1c86ee),
    ("DodgerBlue3", 0x1874cd),
    ("DodgerBlue4", 0x104e8b),
    ("Firebrick", 0xb22222),
    ("Firebrick1", 0xff3030),
    ("Firebrick2", 0xee2c2c),
    ("Firebrick3", 0xcd2626),
    ("Firebrick4", 0x8b1a1a),
    ("FloralWhite", 0xfffaf0),
    ("ForestGreen", 0x228b22),
    ("Fuchsia", 0xff00ff),
    ("Gainsboro", 0xdcdcdc),
    ("GhostWhite", 0xf8f8ff),
    ("Gold", 0xffd700),
    ("Gold1", 0xffd700),
    ("Gold2", 0xeec900),
    ("Gold3", 0xcdad00),
    ("Gold4", 0x8b7500),
    ("Goldenrod", 0xdaa520),
    ("Goldenrod1", 0xffc125),
    ("Goldenrod2", 0xeeb422),
    ("Goldenrod3", 0xcd9b1d),
    ("Goldenrod4", 0x8b6914),
    ("Gray", 0x808080),
    ("Gray0", 0x000000),
    ("Gray1", 0x030303),
    ("Gray10", 0x1a1a1a),
    ("Gray100", 0xffffff),
    ("Gray11", 0x1c1c1c),
    ("Gray12", 0x1f1f1f),
    ("Gray13", 0x212121),
    ("Gray14", 0x242424),
    ("Gray15", 0x262626),
    ("Gray16", 0x292929),
    ("Gray17", 0x2b2b2b),
    ("Gray18", 0x2e2e2e),
    ("Gray19", 0x303030),
    ("Gray2", 0x050505),
    ("Gray20", 0x333333),
    ("Gray21", 0x363636),
    ("Gray22", 0x383838),
    ("Gray23", 0x3b3b3b),
    ("Gray24", 0x3d3d3d),
    ("Gray25", 0x404040),
    ("Gray26", 0x424242),
    ("Gray27", 0x454545),
    ("Gray28", 0x474747),
    ("Gray29", 0x4a4a4a),
    ("Gray3", 0x080808),
    ("Gray30", 0x4d4d4d),
    ("Gray31", 0x4f4f4f),
    ("Gray32", 0x525252),
    ("Gray33", 0x545454),
    ("Gray34", 0x575757),
    ("Gray35", 0x595959),
    ("Gray36", 0x5c5c5c),
    ("Gray37", 0x5e5e5e),
    ("Gray38", 0x616161),
    ("Gray39", 0x636363),
    ("Gray4", 0x0a0a0a),
    ("Gray40", 0x666666),
    ("Gray41", 0x696969),
    ("Gray42", 0x6b6b6b),
    ("Gray43", 0x6e6e6e),
    ("Gray44", 0x707070),
    ("Gray45", 0x737373),
    ("Gray46", 0x757575),
    ("Gray47", 0x787878),
    ("Gray48", 0x7a7a7a),
    ("Gray49", 0x7d7d7d),
    ("Gray5", 0x0d0d0d),
    ("Gray50", 0x7f7f7f),
    ("Gray51", 0x828282),
    ("Gray52", 0x858585),
    ("Gray53", 0x878787),
    ("Gray54", 0x8a8a8a),
    ("Gray55", 0x8c8c8c),
    ("Gray56", 0x8f8f8f),
    ("Gray57", 0x919191),
    ("Gray58", 0x949494),
    ("Gray59", 0x969696),
    ("Gray6", 0x0f0f0f),
    ("Gray60", 0x999999),
    ("Gray61", 0x9c9c9c),
    ("Gray62", 0x9e9e9e),
    ("Gray63", 0xa1a1a1),
    ("Gray64", 0xa3a3a3),
    ("Gray65", 0xa6a6a6),
    ("Gray66", 0xa8a8a8),
    ("Gray67", 0xababab),
    ("Gray68", 0xadadad),
    ("Gray69", 0xb0b0b0),
    ("Gray7", 0x121212),
    ("Gray70", 0xb3b3b3),
    ("Gray71", 0xb5b5b5),
    ("Gray72", 0xb8b8b8),
    ("Gray73", 0xbababa),
    ("Gray74", 0xbdbdbd),
    ("Gray75", 0xbfbfbf),
    ("Gray76", 0xc2c2c2),
    ("Gray77", 0xc4c4c4),
    ("Gray78", 0xc7c7c7),
    ("Gray79", 0xc9c9c9),
    ("Gray8", 0x141414),
    ("Gray80", 0xcccccc),
    ("Gray81", 0xcfcfcf),
    ("Gray82", 0xd1d1d1),
    ("Gray83", 0xd4d4d4),
    ("Gray84", 0xd6d6d6),
    ("Gray85", 0xd9d9d9),
    ("Gray86", 0xdbdbdb),
    ("Gray87", 0xdedede),
    ("Gray88", 0xe0e0e0),
    ("Gray89", 0xe3e3e3),
    ("Gray9", 0x171717),
    ("Gray90", 0xe5e5e5),
    ("Gray91", 0xe8e8e8),
    ("Gray92", 0xebebeb),
    ("Gray93", 0xededed),
    ("Gray94", 0xf0f0f0),
    ("Gray95", 0xf2f2f2),
    ("Gray96", 0xf5f5f5),
    ("Gray97", 0xf7f7f7),
    ("Gray98", 0xfafafa),
    ("Gray99", 0xfcfcfc),
    ("Green", 0x008000),
    ("Green1", 0x00ff00),
    ("Green2", 0x00ee00),
    ("Green3", 0x00cd00),
    ("Green4", 0x008b00),
    ("GreenYellow", 0xadff2f),
    ("Grey", 0x808080),
    ("Grey0", 0x000000),
    ("Grey1", 0x030303),
    ("Grey10", 0x1a1a1a),
    ("Grey100", 0xffffff),
    ("Grey11", 0x1c1c1c),
    ("Grey12", 0x1f1f1f),
    ("Grey13", 0x212121),
    ("Grey14", 0x242424),
    ("Grey15", 0x262626),
    ("Grey16", 0x292929),
    ("Grey17", 0x2b2b2b),
    ("Grey18", 0x2e2e2e),
    ("Grey19", 0x303030),
    ("Grey2", 0x050505),
    ("Grey20", 0x333333),
    ("Grey21", 0x363636),
    ("Grey22", 0x383838),
    ("Grey23", 0x3b3b3b),
    ("Grey24", 0x3d3d3d),
    ("Grey25", 0x404040),
    ("Grey26", 0x424242),
    ("Grey27", 0x454545),
    ("Grey28", 0x474747),
    ("Grey29", 0x4a4a4a),
    ("Grey3", 0x080808),
    ("Grey30", 0x4d4d4d),
    ("Grey31", 0x4f4f4f),
    ("Grey32", 0x525252),
    ("Grey33", 0x545454),
    ("Grey34", 0x575757),
    ("Grey35", 0x595959),
    ("Grey36", 0x5c5c5c),
    ("Grey37", 0x5e5e5e),
    ("Grey38", 0x616161),
    ("Grey39", 0x636363),
    ("Grey4", 0x0a0a0a),
    ("Grey40", 0x666666),
    ("Grey41", 0x696969),
    ("Grey42", 0x6b6b6b),
    ("Grey43", 0x6e6e6e),
    ("Grey44", 0x707070),
    ("Grey45", 0x737373),
    ("Grey46", 0x757575),
    ("Grey47", 0x787878),
    ("Grey48", 0x7a7a7a),
    ("Grey49", 0x7d7d7d),
    ("Grey5", 0x0d0d0d),
    ("Grey50", 0x7f7f7f),
    ("Grey51", 0x828282),
    ("Grey52", 0x858585),
    ("Grey53", 0x878787),
    ("Grey54", 0x8a8a8a),
    ("Grey55", 0x8c8c8c),
    ("Grey56", 0x8f8f8f),
    ("Grey57", 0x919191),
    ("Grey58", 0x949494),
    ("Grey59", 0x969696),
    ("Grey6", 0x0f0f0f),
    ("Grey60", 0x999999),
    ("Grey61", 0x9c9c9c),
    ("Grey62", 0x9e9e9e),
    ("Grey63", 0xa1a1a1),
    ("Grey64", 0xa3a3a3),
    ("Grey65", 0xa6a6a6),
    ("Grey66", 0xa8a8a8),
    ("Grey67", 0xababab),
    ("Grey68", 0xadadad),
    ("Grey69", 0xb0b0b0),
    ("Grey7", 0x121212),
    ("Grey70", 0xb3b3b3),
    ("Grey71", 0xb5b5b5),
    ("Grey72", 0xb8b8b8),
    ("Grey73", 0xbababa),
    ("Grey74", 0xbdbdbd),
    ("Grey75", 0xbfbfbf),
    ("Grey76", 0xc2c2c2),
    ("Grey77", 0xc4c4c4),
    ("Grey78", 0xc7c7c7),
    ("Grey79", 0xc9c9c9),
    ("Grey8", 0x141414),
    ("Grey80", 0xcccccc),
    ("Grey81", 0xcfcfcf),
    ("Grey82", 0xd1d1d1),
    ("Grey83", 0xd4d4d4),
    ("Grey84", 0xd6d6d6),
    ("Grey85", 0xd9d9d9),
    ("Grey86", 0xdbdbdb),
    ("Grey87", 0xdedede),
    ("Grey88", 0xe0e0e0),
    ("Grey89", 0xe3e3e3),
    ("Grey9", 0x171717),
    ("Grey90", 0xe5e5e5),
    ("Grey91", 0xe8e8e8),
    ("Grey92", 0xebebeb),
    ("Grey93", 0xededed),
    ("Grey94", 0xf0f0f0),
    ("Grey95", 0xf2f2f2),
    ("Grey96", 0xf5f5f5),
    ("Grey97", 0xf7f7f7),
    ("Grey98", 0xfafafa),
    ("Grey99", 0xfcfcfc),
    ("Honeydew", 0xf0fff0),
    ("Honeydew1", 0xf0fff0),
    ("Honeydew2", 0xe0eee0),
    ("Honeydew3", 0xc1cdc1),
    ("Honeydew4", 0x838b83),
    ("HotPink", 0xff69b4),
    ("HotPink1", 0xff6eb4),
    ("HotPink2", 0xee6aa7),
    ("HotPink3", 0xcd6090),
    ("HotPink4", 0x8b3a62),
    ("IndianRed", 0xcd5c5c),
    ("IndianRed1", 0xff6a6a),
    ("IndianRed2", 0xee6363),
    ("IndianRed3", 0xcd5555),
    ("IndianRed4", 0x8b3a3a),
    ("Indigo", 0x4b0082),
    ("Ivory", 0xfffff0),
    ("Ivory1", 0xfffff0),
    ("Ivory2", 0xeeeee0),
    ("Ivory3", 0xcdcdc1),
    ("Ivory4", 0x8b8b83),
    ("Khaki", 0xf0e68c),
    ("Khaki1", 0xfff68f),
    ("Khaki2", 0xeee685),
    ("Khaki3", 0xcdc673),
    ("Khaki4", 0x8b864e),
    ("Lavender", 0xe6e6fa),
    ("LavenderBlush", 0xfff0f5),
    ("LavenderBlush1", 0xfff0f5),
    ("LavenderBlush2", 0xeee0e5),
    ("LavenderBlush3", 0xcdc1c5),
    ("LavenderBlush4", 0x8b8386),
    ("LawnGreen", 0x7cfc00),
    ("LemonChiffon", 0xfffacd),
    ("LemonChiffon1", 0xfffacd),
    ("LemonChiffon2", 0xeee9bf),
    ("LemonChiffon3", 0xcdc9a5),
    ("LemonChiffon4", 0x8b8970),
    ("LightBlue", 0xadd8e6),
    ("LightBlue1", 0xbfefff),
    ("LightBlue2", 0xb2dfee),
    ("LightBlue3", 0x9ac0cd),
    ("LightBlue4", 0x68838b),
    ("LightCoral", 0xf08080),
    ("LightCyan", 0xe0ffff),
    ("LightCyan1", 0xe0ffff),
    ("LightCyan2", 0xd1eeee),
    ("LightCyan3", 0xb4cdcd),
    ("LightCyan4", 0x7a8b8b),
    ("LightGoldenrod", 0xeedd82),
    ("LightGoldenrod1", 0xffec8b),
    ("LightGoldenrod2", 0xeedc82),
    ("LightGoldenrod3", 0xcdbe70),
    ("LightGoldenrod4", 0x8b814c),
    ("LightGoldenrodYellow", 0xfafad2),
    ("LightGray", 0xd3d3d3),
    ("LightGreen", 0x90ee90),
    ("LightGrey", 0xd3d3d3),
    ("LightMagenta", 0xffbbff),
    ("LightPink", 0xffb6c1),
    ("LightPink1", 0xffaeb9),
    ("LightPink2", 0xeea2ad),
    ("LightPink3", 0xcd8c95),
    ("LightPink4", 0x8b5f65),
    ("LightRed", 0xffbbbb),
    ("LightSalmon", 0xffa07a),
    ("LightSalmon1", 0xffa07a),
    ("LightSalmon2", 0xee9572),
    ("LightSalmon3", 0xcd8162),
    ("LightSalmon4", 0x8b5742),
    ("LightSeaGreen", 0x20b2aa),
    ("LightSkyBlue", 0x87cefa),
    ("LightSkyBlue1", 0xb0e2ff),
    ("LightSkyBlue2", 0xa4d3ee),
    ("LightSkyBlue3", 0x8db6cd),
    ("LightSkyBlue4", 0x607b8b),
    ("LightSlateBlue", 0x8470ff),
    ("LightSlateGray", 0x778899),
    ("LightSlateGrey", 0x778899),
    ("LightSteelBlue", 0xb0c4de),
    ("LightSteelBlue1", 0xcae1ff),
    ("LightSteelBlue2", 0xbcd2ee),
    ("LightSteelBlue3", 0xa2b5cd),
    ("LightSteelBlue4", 0x6e7b8b),
    ("LightYellow", 0xffffe0),
    ("LightYellow1", 0xffffe0),
    ("LightYellow2", 0xeeeed1),
    ("LightYellow3", 0xcdcdb4),
    ("LightYellow4", 0x8b8b7a),
    ("Lime", 0x00ff00),
    ("LimeGreen", 0x32cd32),
    ("Linen", 0xfaf0e6),
    ("Magenta", 0xff00ff),
    ("Magenta1", 0xff00ff),
    ("Magenta2", 0xee00ee),
    ("Magenta3", 0xcd00cd),
    ("Magenta4", 0x8b008b),
    ("Maroon", 0x800000),
    ("Maroon1", 0xff34b3),
    ("Maroon2", 0xee30a7),
    ("Maroon3", 0xcd2990),
    ("Maroon4", 0x8b1c62),
    ("MediumAquamarine", 0x66cdaa),
    ("MediumBlue", 0x0000cd),
    ("MediumOrchid", 0xba55d3),
    ("MediumOrchid1", 0xe066ff),
    ("MediumOrchid2", 0xd15fee),
    ("MediumOrchid3", 0xb452cd),
    ("MediumOrchid4", 0x7a378b),
    ("MediumPurple", 0x9370db),
    ("MediumPurple1", 0xab82ff),
    ("MediumPurple2", 0x9f79ee),
    ("MediumPurple3", 0x8968cd),
    ("MediumPurple4", 0x5d478b),
    ("MediumSeaGreen", 0x3cb371),
    ("MediumSlateBlue", 0x7b68ee),
    ("MediumSpringGreen", 0x00fa9a),
    ("MediumTurquoise", 0x48d1cc),
    ("MediumVioletRed", 0xc71585),
    ("MidnightBlue", 0x191970),
    ("MintCream", 0xf5fffa),
    ("MistyRose", 0xffe4e1),
    ("MistyRose1", 0xffe4e1),
    ("MistyRose2", 0xeed5d2),
    ("MistyRose3", 0xcdb7b5),
    ("MistyRose4", 0x8b7d7b),
    ("Moccasin", 0xffe4b5),
    ("NavajoWhite", 0xffdead),
    ("NavajoWhite1", 0xffdead),
    ("NavajoWhite2", 0xeecfa1),
    ("NavajoWhite3", 0xcdb38b),
    ("NavajoWhite4", 0x8b795e),
    ("Navy", 0x000080),
    ("NavyBlue", 0x000080),
    ("NvimDarkBlue", 0x004c73),
    ("NvimDarkCyan", 0x007373),
    ("NvimDarkGray1", 0x07080d),
    ("NvimDarkGray2", 0x14161b),
    ("NvimDarkGray3", 0x2c2e33),
    ("NvimDarkGray4", 0x4f5258),
    ("NvimDarkGreen", 0x005523),
    ("NvimDarkGrey1", 0x07080d),
    ("NvimDarkGrey2", 0x14161b),
    ("NvimDarkGrey3", 0x2c2e33),
    ("NvimDarkGrey4", 0x4f5258),
    ("NvimDarkMagenta", 0x470045),
    ("NvimDarkRed", 0x590008),
    ("NvimDarkYellow", 0x6b5300),
    ("NvimLightBlue", 0xa6dbff),
    ("NvimLightCyan", 0x8cf8f7),
    ("NvimLightGray1", 0xeef1f8),
    ("NvimLightGray2", 0xe0e2ea),
    ("NvimLightGray3", 0xc4c6cd),
    ("NvimLightGray4", 0x9b9ea4),
    ("NvimLightGreen", 0xb3f6c0),
    ("NvimLightGrey1", 0xeef1f8),
    ("NvimLightGrey2", 0xe0e2ea),
    ("NvimLightGrey3", 0xc4c6cd),
    ("NvimLightGrey4", 0x9b9ea4),
    ("NvimLightMagenta", 0xffcaff),
    ("NvimLightRed", 0xffc0b9),
    ("NvimLightYellow", 0xfce094),
    ("OldLace", 0xfdf5e6),
    ("Olive", 0x808000),
    ("OliveDrab", 0x6b8e23),
    ("OliveDrab1", 0xc0ff3e),
    ("OliveDrab2", 0xb3ee3a),
    ("OliveDrab3", 0x9acd32),
    ("OliveDrab4", 0x698b22),
    ("Orange", 0xffa500),
    ("Orange1", 0xffa500),
    ("Orange2", 0xee9a00),
    ("Orange3", 0xcd8500),
    ("Orange4", 0x8b5a00),
    ("OrangeRed", 0xff4500),
    ("OrangeRed1", 0xff4500),
    ("OrangeRed2", 0xee4000),
    ("OrangeRed3", 0xcd3700),
    ("OrangeRed4", 0x8b2500),
    ("Orchid", 0xda70d6),
    ("Orchid1", 0xff83fa),
    ("Orchid2", 0xee7ae9),
    ("Orchid3", 0xcd69c9),
    ("Orchid4", 0x8b4789),
    ("PaleGoldenrod", 0xeee8aa),
    ("PaleGreen", 0x98fb98),
    ("PaleGreen1", 0x9aff9a),
    ("PaleGreen2", 0x90ee90),
    ("PaleGreen3", 0x7ccd7c),
    ("PaleGreen4", 0x548b54),
    ("PaleTurquoise", 0xafeeee),
    ("PaleTurquoise1", 0xbbffff),
    ("PaleTurquoise2", 0xaeeeee),
    ("PaleTurquoise3", 0x96cdcd),
    ("PaleTurquoise4", 0x668b8b),
    ("PaleVioletRed", 0xdb7093),
    ("PaleVioletRed1", 0xff82ab),
    ("PaleVioletRed2", 0xee799f),
    ("PaleVioletRed3", 0xcd6889),
    ("PaleVioletRed4", 0x8b475d),
    ("PapayaWhip", 0xffefd5),
    ("PeachPuff", 0xffdab9),
    ("PeachPuff1", 0xffdab9),
    ("PeachPuff2", 0xeecbad),
    ("PeachPuff3", 0xcdaf95),
    ("PeachPuff4", 0x8b7765),
    ("Peru", 0xcd853f),
    ("Pink", 0xffc0cb),
    ("Pink1", 0xffb5c5),
    ("Pink2", 0xeea9b8),
    ("Pink3", 0xcd919e),
    ("Pink4", 0x8b636c),
    ("Plum", 0xdda0dd),
    ("Plum1", 0xffbbff),
    ("Plum2", 0xeeaeee),
    ("Plum3", 0xcd96cd),
    ("Plum4", 0x8b668b),
    ("PowderBlue", 0xb0e0e6),
    ("Purple", 0x800080),
    ("Purple1", 0x9b30ff),
    ("Purple2", 0x912cee),
    ("Purple3", 0x7d26cd),
    ("Purple4", 0x551a8b),
    ("RebeccaPurple", 0x663399),
    ("Red", 0xff0000),
    ("Red1", 0xff0000),
    ("Red2", 0xee0000),
    ("Red3", 0xcd0000),
    ("Red4", 0x8b0000),
    ("RosyBrown", 0xbc8f8f),
    ("RosyBrown1", 0xffc1c1),
    ("RosyBrown2", 0xeeb4b4),
    ("RosyBrown3", 0xcd9b9b),
    ("RosyBrown4", 0x8b6969),
    ("RoyalBlue", 0x4169e1),
    ("RoyalBlue1", 0x4876ff),
    ("RoyalBlue2", 0x436eee),
    ("RoyalBlue3", 0x3a5fcd),
    ("RoyalBlue4", 0x27408b),
    ("SaddleBrown", 0x8b4513),
    ("Salmon", 0xfa8072),
    ("Salmon1", 0xff8c69),
    ("Salmon2", 0xee8262),
    ("Salmon3", 0xcd7054),
    ("Salmon4", 0x8b4c39),
    ("SandyBrown", 0xf4a460),
    ("SeaGreen", 0x2e8b57),
    ("SeaGreen1", 0x54ff9f),
    ("SeaGreen2", 0x4eee94),
    ("SeaGreen3", 0x43cd80),
    ("SeaGreen4", 0x2e8b57),
    ("SeaShell", 0xfff5ee),
    ("Seashell1", 0xfff5ee),
    ("Seashell2", 0xeee5de),
    ("Seashell3", 0xcdc5bf),
    ("Seashell4", 0x8b8682),
    ("Sienna", 0xa0522d),
    ("Sienna1", 0xff8247),
    ("Sienna2", 0xee7942),
    ("Sienna3", 0xcd6839),
    ("Sienna4", 0x8b4726),
    ("Silver", 0xc0c0c0),
    ("SkyBlue", 0x87ceeb),
    ("SkyBlue1", 0x87ceff),
    ("SkyBlue2", 0x7ec0ee),
    ("SkyBlue3", 0x6ca6cd),
    ("SkyBlue4", 0x4a708b),
    ("SlateBlue", 0x6a5acd),
    ("SlateBlue1", 0x836fff),
    ("SlateBlue2", 0x7a67ee),
    ("SlateBlue3", 0x6959cd),
    ("SlateBlue4", 0x473c8b),
    ("SlateGray", 0x708090),
    ("SlateGray1", 0xc6e2ff),
    ("SlateGray2", 0xb9d3ee),
    ("SlateGray3", 0x9fb6cd),
    ("SlateGray4", 0x6c7b8b),
    ("SlateGrey", 0x708090),
    ("Snow", 0xfffafa),
    ("Snow1", 0xfffafa),
    ("Snow2", 0xeee9e9),
    ("Snow3", 0xcdc9c9),
    ("Snow4", 0x8b8989),
    ("SpringGreen", 0x00ff7f),
    ("SpringGreen1", 0x00ff7f),
    ("SpringGreen2", 0x00ee76),
    ("SpringGreen3", 0x00cd66),
    ("SpringGreen4", 0x008b45),
    ("SteelBlue", 0x4682b4),
    ("SteelBlue1", 0x63b8ff),
    ("SteelBlue2", 0x5cacee),
    ("SteelBlue3", 0x4f94cd),
    ("SteelBlue4", 0x36648b),
    ("Tan", 0xd2b48c),
    ("Tan1", 0xffa54f),
    ("Tan2", 0xee9a49),
    ("Tan3", 0xcd853f),
    ("Tan4", 0x8b5a2b),
    ("Teal", 0x008080),
    ("Thistle", 0xd8bfd8),
    ("Thistle1", 0xffe1ff),
    ("Thistle2", 0xeed2ee),
    ("Thistle3", 0xcdb5cd),
    ("Thistle4", 0x8b7b8b),
    ("Tomato", 0xff6347),
    ("Tomato1", 0xff6347),
    ("Tomato2", 0xee5c42),
    ("Tomato3", 0xcd4f39),
    ("Tomato4", 0x8b3626),
    ("Turquoise", 0x40e0d0),
    ("Turquoise1", 0x00f5ff),
    ("Turquoise2", 0x00e5ee),
    ("Turquoise3", 0x00c5cd),
    ("Turquoise4", 0x00868b),
    ("Violet", 0xee82ee),
    ("VioletRed", 0xd02090),
    ("VioletRed1", 0xff3e96),
    ("VioletRed2", 0xee3a8c),
    ("VioletRed3", 0xcd3278),
    ("VioletRed4", 0x8b2252),
    ("WebGray", 0x808080),
    ("WebGreen", 0x008000),
    ("WebGrey", 0x808080),
    ("WebMaroon", 0x800000),
    ("WebPurple", 0x800080),
    ("Wheat", 0xf5deb3),
    ("Wheat1", 0xffe7ba),
    ("Wheat2", 0xeed8ae),
    ("Wheat3", 0xcdba96),
    ("Wheat4", 0x8b7e66),
    ("White", 0xffffff),
    ("WhiteSmoke", 0xf5f5f5),
    ("X11Gray", 0xbebebe),
    ("X11Green", 0x00ff00),
    ("X11Grey", 0xbebebe),
    ("X11Maroon", 0xb03060),
    ("X11Purple", 0xa020f0),
    ("Yellow", 0xffff00),
    ("Yellow1", 0xffff00),
    ("Yellow2", 0xeeee00),
    ("Yellow3", 0xcdcd00),
    ("Yellow4", 0x8b8b00),
    ("YellowGreen", 0x9acd32),
];

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
    let mut lo = 0usize;
    let mut hi = RGB_COLOR_NAME_TABLE.len();

    while lo < hi {
        let mid = (lo + hi) / 2;
        let (table_name, color) = RGB_COLOR_NAME_TABLE[mid];
        match name.to_ascii_lowercase().cmp(&table_name.to_ascii_lowercase()) {
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

// ============================================================================
// Color Name Tables (C-string versions for FFI)
// ============================================================================

/// Color name table as C-strings for FFI
/// This is a parallel table to RGB_COLOR_NAME_TABLE with null-terminated strings
const RGB_COLOR_NAME_TABLE_CSTR: &[&CStr] = &[
    c"AliceBlue",
    c"AntiqueWhite",
    c"AntiqueWhite1",
    c"AntiqueWhite2",
    c"AntiqueWhite3",
    c"AntiqueWhite4",
    c"Aqua",
    c"Aquamarine",
    c"Aquamarine1",
    c"Aquamarine2",
    c"Aquamarine3",
    c"Aquamarine4",
    c"Azure",
    c"Azure1",
    c"Azure2",
    c"Azure3",
    c"Azure4",
    c"Beige",
    c"Bisque",
    c"Bisque1",
    c"Bisque2",
    c"Bisque3",
    c"Bisque4",
    c"Black",
    c"BlanchedAlmond",
    c"Blue",
    c"Blue1",
    c"Blue2",
    c"Blue3",
    c"Blue4",
    c"BlueViolet",
    c"Brown",
    c"Brown1",
    c"Brown2",
    c"Brown3",
    c"Brown4",
    c"BurlyWood",
    c"Burlywood1",
    c"Burlywood2",
    c"Burlywood3",
    c"Burlywood4",
    c"CadetBlue",
    c"CadetBlue1",
    c"CadetBlue2",
    c"CadetBlue3",
    c"CadetBlue4",
    c"ChartReuse",
    c"Chartreuse1",
    c"Chartreuse2",
    c"Chartreuse3",
    c"Chartreuse4",
    c"Chocolate",
    c"Chocolate1",
    c"Chocolate2",
    c"Chocolate3",
    c"Chocolate4",
    c"Coral",
    c"Coral1",
    c"Coral2",
    c"Coral3",
    c"Coral4",
    c"CornFlowerBlue",
    c"Cornsilk",
    c"Cornsilk1",
    c"Cornsilk2",
    c"Cornsilk3",
    c"Cornsilk4",
    c"Crimson",
    c"Cyan",
    c"Cyan1",
    c"Cyan2",
    c"Cyan3",
    c"Cyan4",
    c"DarkBlue",
    c"DarkCyan",
    c"DarkGoldenrod",
    c"DarkGoldenrod1",
    c"DarkGoldenrod2",
    c"DarkGoldenrod3",
    c"DarkGoldenrod4",
    c"DarkGray",
    c"DarkGreen",
    c"DarkGrey",
    c"DarkKhaki",
    c"DarkMagenta",
    c"DarkOliveGreen",
    c"DarkOliveGreen1",
    c"DarkOliveGreen2",
    c"DarkOliveGreen3",
    c"DarkOliveGreen4",
    c"DarkOrange",
    c"DarkOrange1",
    c"DarkOrange2",
    c"DarkOrange3",
    c"DarkOrange4",
    c"DarkOrchid",
    c"DarkOrchid1",
    c"DarkOrchid2",
    c"DarkOrchid3",
    c"DarkOrchid4",
    c"DarkRed",
    c"DarkSalmon",
    c"DarkSeaGreen",
    c"DarkSeaGreen1",
    c"DarkSeaGreen2",
    c"DarkSeaGreen3",
    c"DarkSeaGreen4",
    c"DarkSlateBlue",
    c"DarkSlateGray",
    c"DarkSlateGray1",
    c"DarkSlateGray2",
    c"DarkSlateGray3",
    c"DarkSlateGray4",
    c"DarkSlateGrey",
    c"DarkTurquoise",
    c"DarkViolet",
    c"DarkYellow",
    c"DeepPink",
    c"DeepPink1",
    c"DeepPink2",
    c"DeepPink3",
    c"DeepPink4",
    c"DeepSkyBlue",
    c"DeepSkyBlue1",
    c"DeepSkyBlue2",
    c"DeepSkyBlue3",
    c"DeepSkyBlue4",
    c"DimGray",
    c"DimGrey",
    c"DodgerBlue",
    c"DodgerBlue1",
    c"DodgerBlue2",
    c"DodgerBlue3",
    c"DodgerBlue4",
    c"Firebrick",
    c"Firebrick1",
    c"Firebrick2",
    c"Firebrick3",
    c"Firebrick4",
    c"FloralWhite",
    c"ForestGreen",
    c"Fuchsia",
    c"Gainsboro",
    c"GhostWhite",
    c"Gold",
    c"Gold1",
    c"Gold2",
    c"Gold3",
    c"Gold4",
    c"Goldenrod",
    c"Goldenrod1",
    c"Goldenrod2",
    c"Goldenrod3",
    c"Goldenrod4",
    c"Gray",
    c"Gray0",
    c"Gray1",
    c"Gray10",
    c"Gray100",
    c"Gray11",
    c"Gray12",
    c"Gray13",
    c"Gray14",
    c"Gray15",
    c"Gray16",
    c"Gray17",
    c"Gray18",
    c"Gray19",
    c"Gray2",
    c"Gray20",
    c"Gray21",
    c"Gray22",
    c"Gray23",
    c"Gray24",
    c"Gray25",
    c"Gray26",
    c"Gray27",
    c"Gray28",
    c"Gray29",
    c"Gray3",
    c"Gray30",
    c"Gray31",
    c"Gray32",
    c"Gray33",
    c"Gray34",
    c"Gray35",
    c"Gray36",
    c"Gray37",
    c"Gray38",
    c"Gray39",
    c"Gray4",
    c"Gray40",
    c"Gray41",
    c"Gray42",
    c"Gray43",
    c"Gray44",
    c"Gray45",
    c"Gray46",
    c"Gray47",
    c"Gray48",
    c"Gray49",
    c"Gray5",
    c"Gray50",
    c"Gray51",
    c"Gray52",
    c"Gray53",
    c"Gray54",
    c"Gray55",
    c"Gray56",
    c"Gray57",
    c"Gray58",
    c"Gray59",
    c"Gray6",
    c"Gray60",
    c"Gray61",
    c"Gray62",
    c"Gray63",
    c"Gray64",
    c"Gray65",
    c"Gray66",
    c"Gray67",
    c"Gray68",
    c"Gray69",
    c"Gray7",
    c"Gray70",
    c"Gray71",
    c"Gray72",
    c"Gray73",
    c"Gray74",
    c"Gray75",
    c"Gray76",
    c"Gray77",
    c"Gray78",
    c"Gray79",
    c"Gray8",
    c"Gray80",
    c"Gray81",
    c"Gray82",
    c"Gray83",
    c"Gray84",
    c"Gray85",
    c"Gray86",
    c"Gray87",
    c"Gray88",
    c"Gray89",
    c"Gray9",
    c"Gray90",
    c"Gray91",
    c"Gray92",
    c"Gray93",
    c"Gray94",
    c"Gray95",
    c"Gray96",
    c"Gray97",
    c"Gray98",
    c"Gray99",
    c"Green",
    c"Green1",
    c"Green2",
    c"Green3",
    c"Green4",
    c"GreenYellow",
    c"Grey",
    c"Grey0",
    c"Grey1",
    c"Grey10",
    c"Grey100",
    c"Grey11",
    c"Grey12",
    c"Grey13",
    c"Grey14",
    c"Grey15",
    c"Grey16",
    c"Grey17",
    c"Grey18",
    c"Grey19",
    c"Grey2",
    c"Grey20",
    c"Grey21",
    c"Grey22",
    c"Grey23",
    c"Grey24",
    c"Grey25",
    c"Grey26",
    c"Grey27",
    c"Grey28",
    c"Grey29",
    c"Grey3",
    c"Grey30",
    c"Grey31",
    c"Grey32",
    c"Grey33",
    c"Grey34",
    c"Grey35",
    c"Grey36",
    c"Grey37",
    c"Grey38",
    c"Grey39",
    c"Grey4",
    c"Grey40",
    c"Grey41",
    c"Grey42",
    c"Grey43",
    c"Grey44",
    c"Grey45",
    c"Grey46",
    c"Grey47",
    c"Grey48",
    c"Grey49",
    c"Grey5",
    c"Grey50",
    c"Grey51",
    c"Grey52",
    c"Grey53",
    c"Grey54",
    c"Grey55",
    c"Grey56",
    c"Grey57",
    c"Grey58",
    c"Grey59",
    c"Grey6",
    c"Grey60",
    c"Grey61",
    c"Grey62",
    c"Grey63",
    c"Grey64",
    c"Grey65",
    c"Grey66",
    c"Grey67",
    c"Grey68",
    c"Grey69",
    c"Grey7",
    c"Grey70",
    c"Grey71",
    c"Grey72",
    c"Grey73",
    c"Grey74",
    c"Grey75",
    c"Grey76",
    c"Grey77",
    c"Grey78",
    c"Grey79",
    c"Grey8",
    c"Grey80",
    c"Grey81",
    c"Grey82",
    c"Grey83",
    c"Grey84",
    c"Grey85",
    c"Grey86",
    c"Grey87",
    c"Grey88",
    c"Grey89",
    c"Grey9",
    c"Grey90",
    c"Grey91",
    c"Grey92",
    c"Grey93",
    c"Grey94",
    c"Grey95",
    c"Grey96",
    c"Grey97",
    c"Grey98",
    c"Grey99",
    c"Honeydew",
    c"Honeydew1",
    c"Honeydew2",
    c"Honeydew3",
    c"Honeydew4",
    c"HotPink",
    c"HotPink1",
    c"HotPink2",
    c"HotPink3",
    c"HotPink4",
    c"IndianRed",
    c"IndianRed1",
    c"IndianRed2",
    c"IndianRed3",
    c"IndianRed4",
    c"Indigo",
    c"Ivory",
    c"Ivory1",
    c"Ivory2",
    c"Ivory3",
    c"Ivory4",
    c"Khaki",
    c"Khaki1",
    c"Khaki2",
    c"Khaki3",
    c"Khaki4",
    c"Lavender",
    c"LavenderBlush",
    c"LavenderBlush1",
    c"LavenderBlush2",
    c"LavenderBlush3",
    c"LavenderBlush4",
    c"LawnGreen",
    c"LemonChiffon",
    c"LemonChiffon1",
    c"LemonChiffon2",
    c"LemonChiffon3",
    c"LemonChiffon4",
    c"LightBlue",
    c"LightBlue1",
    c"LightBlue2",
    c"LightBlue3",
    c"LightBlue4",
    c"LightCoral",
    c"LightCyan",
    c"LightCyan1",
    c"LightCyan2",
    c"LightCyan3",
    c"LightCyan4",
    c"LightGoldenrod",
    c"LightGoldenrod1",
    c"LightGoldenrod2",
    c"LightGoldenrod3",
    c"LightGoldenrod4",
    c"LightGoldenrodYellow",
    c"LightGray",
    c"LightGreen",
    c"LightGrey",
    c"LightMagenta",
    c"LightPink",
    c"LightPink1",
    c"LightPink2",
    c"LightPink3",
    c"LightPink4",
    c"LightRed",
    c"LightSalmon",
    c"LightSalmon1",
    c"LightSalmon2",
    c"LightSalmon3",
    c"LightSalmon4",
    c"LightSeaGreen",
    c"LightSkyBlue",
    c"LightSkyBlue1",
    c"LightSkyBlue2",
    c"LightSkyBlue3",
    c"LightSkyBlue4",
    c"LightSlateBlue",
    c"LightSlateGray",
    c"LightSlateGrey",
    c"LightSteelBlue",
    c"LightSteelBlue1",
    c"LightSteelBlue2",
    c"LightSteelBlue3",
    c"LightSteelBlue4",
    c"LightYellow",
    c"LightYellow1",
    c"LightYellow2",
    c"LightYellow3",
    c"LightYellow4",
    c"Lime",
    c"LimeGreen",
    c"Linen",
    c"Magenta",
    c"Magenta1",
    c"Magenta2",
    c"Magenta3",
    c"Magenta4",
    c"Maroon",
    c"Maroon1",
    c"Maroon2",
    c"Maroon3",
    c"Maroon4",
    c"MediumAquamarine",
    c"MediumBlue",
    c"MediumOrchid",
    c"MediumOrchid1",
    c"MediumOrchid2",
    c"MediumOrchid3",
    c"MediumOrchid4",
    c"MediumPurple",
    c"MediumPurple1",
    c"MediumPurple2",
    c"MediumPurple3",
    c"MediumPurple4",
    c"MediumSeaGreen",
    c"MediumSlateBlue",
    c"MediumSpringGreen",
    c"MediumTurquoise",
    c"MediumVioletRed",
    c"MidnightBlue",
    c"MintCream",
    c"MistyRose",
    c"MistyRose1",
    c"MistyRose2",
    c"MistyRose3",
    c"MistyRose4",
    c"Moccasin",
    c"NavajoWhite",
    c"NavajoWhite1",
    c"NavajoWhite2",
    c"NavajoWhite3",
    c"NavajoWhite4",
    c"Navy",
    c"NavyBlue",
    c"NvimDarkBlue",
    c"NvimDarkCyan",
    c"NvimDarkGray1",
    c"NvimDarkGray2",
    c"NvimDarkGray3",
    c"NvimDarkGray4",
    c"NvimDarkGreen",
    c"NvimDarkGrey1",
    c"NvimDarkGrey2",
    c"NvimDarkGrey3",
    c"NvimDarkGrey4",
    c"NvimDarkMagenta",
    c"NvimDarkRed",
    c"NvimDarkYellow",
    c"NvimLightBlue",
    c"NvimLightCyan",
    c"NvimLightGray1",
    c"NvimLightGray2",
    c"NvimLightGray3",
    c"NvimLightGray4",
    c"NvimLightGreen",
    c"NvimLightGrey1",
    c"NvimLightGrey2",
    c"NvimLightGrey3",
    c"NvimLightGrey4",
    c"NvimLightMagenta",
    c"NvimLightRed",
    c"NvimLightYellow",
    c"OldLace",
    c"Olive",
    c"OliveDrab",
    c"OliveDrab1",
    c"OliveDrab2",
    c"OliveDrab3",
    c"OliveDrab4",
    c"Orange",
    c"Orange1",
    c"Orange2",
    c"Orange3",
    c"Orange4",
    c"OrangeRed",
    c"OrangeRed1",
    c"OrangeRed2",
    c"OrangeRed3",
    c"OrangeRed4",
    c"Orchid",
    c"Orchid1",
    c"Orchid2",
    c"Orchid3",
    c"Orchid4",
    c"PaleGoldenrod",
    c"PaleGreen",
    c"PaleGreen1",
    c"PaleGreen2",
    c"PaleGreen3",
    c"PaleGreen4",
    c"PaleTurquoise",
    c"PaleTurquoise1",
    c"PaleTurquoise2",
    c"PaleTurquoise3",
    c"PaleTurquoise4",
    c"PaleVioletRed",
    c"PaleVioletRed1",
    c"PaleVioletRed2",
    c"PaleVioletRed3",
    c"PaleVioletRed4",
    c"PapayaWhip",
    c"PeachPuff",
    c"PeachPuff1",
    c"PeachPuff2",
    c"PeachPuff3",
    c"PeachPuff4",
    c"Peru",
    c"Pink",
    c"Pink1",
    c"Pink2",
    c"Pink3",
    c"Pink4",
    c"Plum",
    c"Plum1",
    c"Plum2",
    c"Plum3",
    c"Plum4",
    c"PowderBlue",
    c"Purple",
    c"Purple1",
    c"Purple2",
    c"Purple3",
    c"Purple4",
    c"RebeccaPurple",
    c"Red",
    c"Red1",
    c"Red2",
    c"Red3",
    c"Red4",
    c"RosyBrown",
    c"RosyBrown1",
    c"RosyBrown2",
    c"RosyBrown3",
    c"RosyBrown4",
    c"RoyalBlue",
    c"RoyalBlue1",
    c"RoyalBlue2",
    c"RoyalBlue3",
    c"RoyalBlue4",
    c"SaddleBrown",
    c"Salmon",
    c"Salmon1",
    c"Salmon2",
    c"Salmon3",
    c"Salmon4",
    c"SandyBrown",
    c"SeaGreen",
    c"SeaGreen1",
    c"SeaGreen2",
    c"SeaGreen3",
    c"SeaGreen4",
    c"SeaShell",
    c"Seashell1",
    c"Seashell2",
    c"Seashell3",
    c"Seashell4",
    c"Sienna",
    c"Sienna1",
    c"Sienna2",
    c"Sienna3",
    c"Sienna4",
    c"Silver",
    c"SkyBlue",
    c"SkyBlue1",
    c"SkyBlue2",
    c"SkyBlue3",
    c"SkyBlue4",
    c"SlateBlue",
    c"SlateBlue1",
    c"SlateBlue2",
    c"SlateBlue3",
    c"SlateBlue4",
    c"SlateGray",
    c"SlateGray1",
    c"SlateGray2",
    c"SlateGray3",
    c"SlateGray4",
    c"SlateGrey",
    c"Snow",
    c"Snow1",
    c"Snow2",
    c"Snow3",
    c"Snow4",
    c"SpringGreen",
    c"SpringGreen1",
    c"SpringGreen2",
    c"SpringGreen3",
    c"SpringGreen4",
    c"SteelBlue",
    c"SteelBlue1",
    c"SteelBlue2",
    c"SteelBlue3",
    c"SteelBlue4",
    c"Tan",
    c"Tan1",
    c"Tan2",
    c"Tan3",
    c"Tan4",
    c"Teal",
    c"Thistle",
    c"Thistle1",
    c"Thistle2",
    c"Thistle3",
    c"Thistle4",
    c"Tomato",
    c"Tomato1",
    c"Tomato2",
    c"Tomato3",
    c"Tomato4",
    c"Turquoise",
    c"Turquoise1",
    c"Turquoise2",
    c"Turquoise3",
    c"Turquoise4",
    c"Violet",
    c"VioletRed",
    c"VioletRed1",
    c"VioletRed2",
    c"VioletRed3",
    c"VioletRed4",
    c"WebGray",
    c"WebGreen",
    c"WebGrey",
    c"WebMaroon",
    c"WebPurple",
    c"Wheat",
    c"Wheat1",
    c"Wheat2",
    c"Wheat3",
    c"Wheat4",
    c"White",
    c"WhiteSmoke",
    c"X11Gray",
    c"X11Green",
    c"X11Grey",
    c"X11Maroon",
    c"X11Purple",
    c"Yellow",
    c"Yellow1",
    c"Yellow2",
    c"Yellow3",
    c"Yellow4",
    c"YellowGreen",
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
#[no_mangle]
pub unsafe extern "C" fn rs_coloridx_to_name(
    idx: c_int,
    val: c_int,
    hexbuf: *mut c_char,
) -> *const c_char {
    if idx >= 0 {
        // Valid index into color table
        let idx_usize = idx as usize;
        if idx_usize < RGB_COLOR_NAME_TABLE_CSTR.len() {
            return RGB_COLOR_NAME_TABLE_CSTR[idx_usize].as_ptr();
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
