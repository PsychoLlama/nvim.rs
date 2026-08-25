#![deny(unsafe_op_in_unsafe_fn)]

//! The attribute set: every distinct combination of colours and attributes
//! the screen is currently using, numbered.
//!
//! A screen cell carries an *attribute id*, not colours — one `sattr_T` per
//! cell instead of nine fields, and the UI is told each definition once. Ids
//! are indices into [`ATTRS`], handed out by [`get_attr_entry`] and never
//! reused; id 0 is "no attributes at all" and is the table's first entry.
//!
//! Everything that produces an id goes through that one function, so the
//! table is deduplicated by construction: asking twice for the same colours
//! answers the same id, and only the first ask reaches the UI. When the table
//! fills up (`MAX_TYPENR`) it is thrown away and rebuilt, which is why ids are
//! valid only until [`clear_hl_tables`] runs.
//!
//! Where the ids come from:
//!
//! * a syntax or `:highlight` group ([`hl_get_syn_attr`], and
//!   [`namespace`]'s `hl_get_ui_attr` for the builtin ones),
//! * two ids combined, spelling "this cell is a spelling error *and* a
//!   comment" ([`hl_combine_attr`]),
//! * two ids blended, for `'winblend'`/`'pumblend'` ([`blend`]),
//! * `:terminal` forwarding what the program asked for ([`hl_get_term_attr`]).
//!
//! The last three are memoised ([`cache`]) because they are asked per cell.
//!
//! `hlstate` is the other axis. A UI that asked for `ext_hlstate` wants to
//! know *why* a cell looks the way it does, so each entry records the ids or
//! group it came from and [`hl_inspect`] flattens that back out. Without such
//! a UI the provenance is erased on the way in, which keeps the table smaller.
//!
//! Split for size:
//!
//! * [`cache`] — the memo table type.
//! * [`blend`] — `'winblend'`/`'pumblend'`.
//! * [`dict`] — the API's dictionary form of an attribute set.
//! * [`namespace`] — per-namespace group definitions and the `HLF_*` tables.

use crate::api::private::helpers::{arena_array, arena_dict, cstr_as_string};
use crate::api::ui::{remote_ui_hl_attr_define, remote_ui_hl_group_set};
use crate::drawscreen::screen_invalidate_highlights;
use crate::global_cell::GlobalCell;
use crate::highlight_group::{
    HLF_COUNT, highlight_attr_set_all, highlight_changed, hlf_names, syn_id2name,
};
use crate::main::{highlight_attr, highlight_attr_last};
use crate::memory::{ARENA_EMPTY, arena_finish, arena_mem_free};
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::types::builders::static_cstring;
use crate::types::{
    Arena, Array, Dict, HlAttrs, HlEntry, HlKind, Integer, KeyValuePair, Object, RemoteUI, uint32_t,
};
use crate::ui::ui_call_hl_attr_define;
use cache::AttrCache;
use core::ffi::{CStr, c_char, c_int};
use core::hash::BuildHasherDefault;
use std::collections::HashMap;
use std::ffi::CString;
use std::hash::DefaultHasher;

// Split out for size; the rest of the tree calls all of it as `highlight::*`.
pub mod blend;
pub mod cache;
pub mod dict;
pub mod namespace;

pub use blend::{hl_blend_attrs, hl_invalidate_blends};
pub use dict::{HLATTRS_DICT_SIZE, dict2hlattrs, hl_get_attr_by_id, hlattrs2dict};
pub use namespace::{
    hl_check_ns, hl_get_ui_attr, hl_ns_get_attrs, ns_get_hl, ns_hl_def, update_ns_hl,
    update_window_hl, win_bg_attr, win_check_ns_hl, win_hl_attr,
};

crate::flag_set! {
    /// The attribute bits an `HlAttrs`' `rgb_ae_attr`/`cterm_ae_attr` carry.
    ///
    /// The sign bit stays clear: a negative attribute value is an invalid
    /// one.
    ///
    /// **Five of these are not bits.** The underline styles share the three
    /// bits of [`UNDERLINE_MASK`](Self::UNDERLINE_MASK) and are an
    /// enumeration inside a flag word: `UNDERDOUBLE` is `UNDERLINE |
    /// UNDERCURL`, so [`has`](Self::has) answers *yes* for a style that is
    /// not set. Ask with `masked(UNDERLINE_MASK) == …`, which is what every
    /// site here does.
    pub struct HlAttrFlags;

    const INVERSE = 0x01;
    const BOLD = 0x02;
    const ITALIC = 0x04;
    /// The three bits the underline styles share: at most one style at a
    /// time.
    const UNDERLINE_MASK = 0x38;
    const UNDERLINE = 0x08;
    const UNDERCURL = 0x10;
    const UNDERDOUBLE = 0x18;
    const UNDERDOTTED = 0x20;
    const UNDERDASHED = 0x28;
    const STANDOUT = 0x40;
    const STRIKETHROUGH = 0x80;
    const ALTFONT = 0x100;
    const DIM = 0x200;
    const NOCOMBINE = 0x400;
    const BG_INDEXED = 0x800;
    const FG_INDEXED = 0x1000;
    const DEFAULT = 0x2000;
    const GLOBAL = 0x4000;
    const BLINK = 0x8000;
    /// The SGR attribute, unrelated to `SynFlags::CONCEAL`.
    const CONCEALED = 0x1_0000;
    const OVERLINE = 0x2_0000;
}

// The `HLF_*` builtin-group indices this family names. The full list lives
// with `hlf_names`; these are the ones the code here singles out.

// What an entry was made from, recorded for `ext_hlstate`.
pub(crate) const kHlUnknown: HlKind = 0;
pub(crate) const kHlUI: HlKind = 1;
pub(crate) const kHlSyntax: HlKind = 2;
pub(crate) const kHlTerminal: HlKind = 3;
pub(crate) const kHlCombine: HlKind = 4;
pub(crate) const kHlBlend: HlKind = 5;
pub(crate) const kHlBlendThrough: HlKind = 6;
pub(crate) const kHlInvalid: HlKind = 7;

/// An attribute set with nothing set. -1 is "unset" for an RGB colour and 0
/// for a cterm one, which is why the two halves do not look alike.
pub const HLATTRS_INIT: HlAttrs = HlAttrs {
    rgb_ae_attr: HlAttrFlags::NONE,
    cterm_ae_attr: HlAttrFlags::NONE,
    rgb_fg_color: -1,
    rgb_bg_color: -1,
    rgb_sp_color: -1,
    cterm_fg_color: 0,
    cterm_bg_color: 0,
    hl_blend: -1,
    url: -1,
};

/// How many entries the table may hold before it is thrown away and rebuilt:
/// an id has to fit a `sattr_T`.
const MAX_TYPENR: usize = 65535;

/// Does any UI want to know where an attribute came from (`ext_hlstate`)?
/// Until one does, the provenance fields are erased on the way in.
static HLSTATE_ACTIVE: GlobalCell<bool> = GlobalCell::new(false);

/// The attribute sets, by id.
static ATTRS: GlobalCell<AttrTable> = GlobalCell::new(AttrTable::new());

/// Results of [`hl_combine_attr`], by the pair that produced them.
static COMBINE: GlobalCell<AttrCache> = GlobalCell::new(AttrCache::new());

/// The combine cache, by address: it is read and written once per cell on
/// the draw path, so the lookup goes straight at it rather than through a
/// borrow.
fn combine_cache() -> *mut AttrCache {
    COMBINE.ptr()
}

/// The attribute the built-in highlight group `hlf` resolves to.
///
/// One `c_int` out of the table rather than a copy of all 76 of them: this
/// is asked per spell run and per UI group.
pub(crate) fn default_hl_attr(hlf: usize) -> c_int {
    // SAFETY: reads one element of a `static`'s array.
    unsafe { (*highlight_attr.ptr())[hlf] }
}

/// The URLs entries refer to by index (OSC 8 hyperlinks).
static URLS: GlobalCell<UrlTable> = GlobalCell::new(UrlTable::new());

/// Small keys, never iterated, so a fixed-seed hasher is enough and is
/// constructible in a `static`.
type Table<K, V> = HashMap<K, V, BuildHasherDefault<DefaultHasher>>;

/// The attribute sets in id order, plus the reverse lookup that makes
/// [`get_attr_entry`] deduplicating.
///
/// An index map rather than a plain map because both directions are used: the
/// screen has ids and wants colours, and everything that produces attributes
/// has colours and wants an id.
struct AttrTable {
    entries: Vec<HlEntry>,
    ids: Table<HlEntry, uint32_t>,
}

impl AttrTable {
    const fn new() -> Self {
        Self {
            entries: Vec::new(),
            ids: HashMap::with_hasher(BuildHasherDefault::new()),
        }
    }

    /// `entry`'s id, and whether it had to be added.
    fn put(&mut self, entry: HlEntry) -> (c_int, bool) {
        if let Some(&id) = self.ids.get(&entry) {
            return (id as c_int, false);
        }
        let id = self.entries.len() as uint32_t;
        self.entries.push(entry);
        self.ids.insert(entry, id);
        (id as c_int, true)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    /// The entry `id` names.
    ///
    /// Panics on an id the table never handed out. Upstream indexed the array
    /// unchecked here; a panic is the honest answer for the case it read out
    /// of bounds in.
    fn at(&self, id: c_int) -> HlEntry {
        self.entries[id as usize]
    }

    /// The entry `id` names, or `None` for anything outside the table — an id
    /// from before the last rebuild, or the 0 sentinel.
    fn live(&self, id: c_int) -> Option<HlEntry> {
        if id <= 0 || id as usize >= self.entries.len() {
            None
        } else {
            Some(self.entries[id as usize])
        }
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.ids.clear();
    }
}

/// The interned URLs, by index. Same index-map shape as [`AttrTable`]: an
/// attribute set stores the index, and the UI is sent the string.
struct UrlTable {
    urls: Vec<CString>,
    ids: Table<CString, uint32_t>,
}

impl UrlTable {
    const fn new() -> Self {
        Self {
            urls: Vec::new(),
            ids: HashMap::with_hasher(BuildHasherDefault::new()),
        }
    }

    fn intern(&mut self, url: &CStr) -> uint32_t {
        if let Some(&id) = self.ids.get(url) {
            return id;
        }
        let id = self.urls.len() as uint32_t;
        self.urls.push(url.to_owned());
        self.ids.insert(url.to_owned(), id);
        id
    }

    fn clear(&mut self) {
        self.urls.clear();
        self.ids.clear();
    }
}

/// Puts the id-0 entry in place: "no attributes", which every unhighlighted
/// cell carries.
pub fn highlight_init() {
    ATTRS.with_mut(|attrs| {
        attrs.put(HlEntry {
            attr: HLATTRS_INIT,
            kind: kHlInvalid,
            id1: 0,
            id2: 0,
        })
    });
}

/// Turns on `ext_hlstate` bookkeeping. Answers whether the tables had to be
/// rebuilt — everything already in them was recorded without provenance.
///
/// # Safety
/// Rebuilds the highlight tables and forces a redraw; main thread only.
pub unsafe fn highlight_use_hlstate() -> bool {
    if HLSTATE_ACTIVE.get() {
        return false;
    }
    HLSTATE_ACTIVE.set(true);
    // SAFETY: the editor's own tables.
    unsafe { clear_hl_tables(true) };
    true
}

/// The id for `entry`, adding it to the table if it is new.
///
/// Answers 0 — the empty attribute set — when the table is full and cannot be
/// rebuilt, which is the only way this fails.
///
/// # Safety
/// May rebuild the highlight tables and emit a UI event; main thread only.
pub(crate) unsafe fn get_attr_entry(mut entry: HlEntry) -> c_int {
    // Set while the table is being rebuilt from inside this function, so that
    // a rebuild triggered by the rebuild gives up instead of recursing.
    static REBUILDING: GlobalCell<bool> = GlobalCell::new(false);

    if !HLSTATE_ACTIVE.get() {
        // Nothing will read the provenance; erase it and keep the table small.
        entry.kind = kHlUnknown;
        entry.id1 = 0;
        entry.id2 = 0;
    }

    let mut retried = false;
    let id = loop {
        let (id, is_new) = ATTRS.with_mut(|attrs| attrs.put(entry));
        if !is_new {
            return id;
        }
        if ATTRS.with(AttrTable::len) <= MAX_TYPENR {
            break id;
        }

        // Out of attribute entries: throw the table away and let every group
        // compute a fresh one. Twice round means we are really out.
        if REBUILDING.get() || retried {
            let msg = c"E424: Too many different highlighting attributes in use";
            // SAFETY: the caller's editor state.
            unsafe { emsg(gettext(msg.as_ptr())) };
            return 0;
        }
        REBUILDING.set(true);
        // SAFETY: as above.
        unsafe { clear_hl_tables(true) };
        REBUILDING.set(false);
        if entry.kind == kHlCombine {
            // The ids this entry combines are gone, so it means nothing now.
            return 0;
        }
        retried = true;
    };

    // A new id: tell the UIs what it looks like.
    // SAFETY: the arena is local and the event only borrows the array.
    unsafe {
        let mut arena = ARENA_EMPTY;
        let inspect = hl_inspect(id, &raw mut arena);
        // Internally there is one attribute set for cterm and rgb;
        // `remote_ui_hl_attr_define` is where they part company.
        ui_call_hl_attr_define(Integer::from(id), entry.attr, entry.attr, inspect);
        arena_mem_free(arena_finish(&raw mut arena));
    }
    id
}

/// Sends a newly connected UI the whole table, plus the current attribute of
/// every builtin group.
///
/// # Safety
/// `ui` is a live remote UI; main thread only.
pub unsafe fn ui_send_all_hls(ui: *mut RemoteUI) {
    // SAFETY: the caller's UI; each iteration's arena is local to it.
    unsafe {
        // The bound is re-read each time round, as upstream's `for` did:
        // sending an event is not supposed to touch the table, and if one
        // ever did this stops rather than reading past the end.
        let mut i = 1;
        while i < ATTRS.with(AttrTable::len) {
            let mut arena = ARENA_EMPTY;
            let inspect = hl_inspect(i as c_int, &raw mut arena);
            let attr = ATTRS.with(|attrs| attrs.at(i as c_int)).attr;
            remote_ui_hl_attr_define(ui, i as Integer, attr, attr, inspect);
            arena_mem_free(arena_finish(&raw mut arena));
            i += 1;
        }
        for hlf in 0..HLF_COUNT as usize {
            let name = cstr_as_string(hlf_names[hlf]);
            let attr = Integer::from(default_hl_attr(hlf));
            remote_ui_hl_group_set(ui, name, attr);
        }
    }
}

/// The id for syntax group `idx`'s attributes, as namespace `ns_id` sees it.
///
/// Attributes that are entirely unset answer 0 rather than an entry of their
/// own — but only in the global namespace, where "unset" and "not defined in
/// this namespace" are the same thing.
///
/// # Safety
/// As [`get_attr_entry`].
pub unsafe fn hl_get_syn_attr(ns_id: c_int, idx: c_int, at_en: HlAttrs) -> c_int {
    // TODO(bfredl): should we do this unconditionally
    let anything_set = at_en.cterm_fg_color != 0
        || at_en.cterm_bg_color != 0
        || at_en.rgb_fg_color != -1
        || at_en.rgb_bg_color != -1
        || at_en.rgb_sp_color != -1
        || at_en.cterm_ae_attr != HlAttrFlags::NONE
        || at_en.rgb_ae_attr != HlAttrFlags::NONE
        || ns_id != 0;
    if !anything_set {
        return 0;
    }
    // SAFETY: the caller's editor state.
    unsafe {
        get_attr_entry(HlEntry {
            attr: at_en,
            kind: kHlSyntax,
            id1: idx,
            id2: ns_id,
        })
    }
}

/// `attr` with `'winblend'` applied, unless it carries a `blend=` of its own
/// — an explicit `blend=` on the group wins over the window's.
///
/// # Safety
/// `attr` must be an id this table handed out; main thread only.
pub unsafe fn hl_apply_winblend(winbl: c_int, attr: c_int) -> c_int {
    let mut entry = ATTRS.with(|attrs| attrs.at(attr));
    if entry.attr.hl_blend != -1 || winbl <= 0 {
        return attr;
    }
    entry.attr.hl_blend = winbl;
    // SAFETY: the caller's editor state.
    unsafe { get_attr_entry(entry) }
}

/// The id for plain `HlAttrFlags::UNDERLINE`, which is what a URL is drawn with.
///
/// # Safety
/// As [`get_attr_entry`].
pub unsafe fn hl_get_underline() -> c_int {
    let mut attrs = HLATTRS_INIT;
    attrs.cterm_ae_attr = HlAttrFlags::UNDERLINE;
    attrs.rgb_ae_attr = HlAttrFlags::UNDERLINE;
    // SAFETY: the caller's editor state.
    unsafe {
        get_attr_entry(HlEntry {
            attr: attrs,
            kind: kHlUI,
            id1: 0,
            id2: 0,
        })
    }
}

/// `attr` combined with an entry carrying `url`.
///
/// The URL is interned, so the same target used a thousand times costs one
/// string and one attribute entry.
///
/// # Safety
/// `url` is NUL-terminated; as [`get_attr_entry`] otherwise.
pub unsafe fn hl_add_url(attr: c_int, url: *const c_char) -> c_int {
    let mut attrs = HLATTRS_INIT;
    // SAFETY: the caller's string.
    let url = unsafe { CStr::from_ptr(url) };
    attrs.url = URLS.with_mut(|urls| urls.intern(url)) as i32;
    // SAFETY: the caller's editor state.
    unsafe {
        let with_url = get_attr_entry(HlEntry {
            attr: attrs,
            kind: kHlUI,
            id1: 0,
            id2: 0,
        });
        hl_combine_attr(attr, with_url)
    }
}

/// The URL at `index`. Panics on an index no entry ever stored (upstream
/// asserted the table was allocated and then read it unchecked).
///
/// The pointer borrows the table, which is stable until [`clear_hl_tables`]
/// runs — the same lifetime upstream's had.
pub fn hl_get_url(index: uint32_t) -> *const c_char {
    URLS.with(|urls| urls.urls[index as usize].as_ptr())
}

/// The id for attributes a `:terminal` program asked for directly.
///
/// # Safety
/// As [`get_attr_entry`].
pub unsafe fn hl_get_term_attr(attrs: HlAttrs) -> c_int {
    // SAFETY: the caller's editor state.
    unsafe {
        get_attr_entry(HlEntry {
            attr: attrs,
            kind: kHlTerminal,
            id1: 0,
            id2: 0,
        })
    }
}

/// Empties every attribute table, invalidating every id in existence.
///
/// With `reinit` the table is put back into a usable state and everything on
/// screen is recomputed; without it this is the free-all-memory path, which
/// also drops the namespace definitions.
///
/// # Safety
/// Forces a full redraw; main thread only.
pub unsafe fn clear_hl_tables(reinit: bool) {
    URLS.with_mut(UrlTable::clear);
    ATTRS.with_mut(AttrTable::clear);
    COMBINE.with_mut(AttrCache::clear);
    blend::clear_caches();
    if !reinit {
        namespace::clear_ns_defs();
        return;
    }
    highlight_init();
    // No group's attribute matches its remembered one any more.
    highlight_attr_last.set([-1; HLF_COUNT as usize]);
    // SAFETY: the editor's own tables.
    unsafe {
        highlight_attr_set_all();
        highlight_changed();
        screen_invalidate_highlights();
    }
}

/// Combines two attribute-bit masks, `prim_ae` winning.
///
/// The underline styles share a field, so a style in `prim_ae` replaces the
/// one in `char_ae` rather than or-ing with it; everything else is a union.
fn hl_combine_ae(char_ae: HlAttrFlags, prim_ae: HlAttrFlags) -> HlAttrFlags {
    let char_ul = char_ae.masked(HlAttrFlags::UNDERLINE_MASK);
    let prim_ul = prim_ae.masked(HlAttrFlags::UNDERLINE_MASK);
    let new_ul = if prim_ul.is_empty() { char_ul } else { prim_ul };
    char_ae.without(HlAttrFlags::UNDERLINE_MASK)
        | prim_ae.without(HlAttrFlags::UNDERLINE_MASK)
        | new_ul
}

/// The id for `char_attr` overlaid with `prim_attr`.
///
/// This is how "a spelling error inside a comment" gets one id: the
/// character's own attributes are the base and the special ones override.
/// Memoised, because the screen asks per cell and there tend to be a lot of
/// spelling mistakes.
///
/// # Safety
/// Both ids must be ones this table handed out; main thread only.
pub unsafe fn hl_combine_attr(char_attr: c_int, prim_attr: c_int) -> c_int {
    if char_attr == 0 {
        return prim_attr;
    } else if prim_attr == 0 {
        return char_attr;
    }

    // SAFETY: the caller's ids and the editor's own tables.
    unsafe {
        let cached = (*combine_cache()).get(char_attr, prim_attr);
        if cached > 0 {
            return cached;
        }

        let char_aep = syn_attr2entry(char_attr);
        let prim_aep = syn_attr2entry(prim_attr);

        // Start from the low-priority set and override below.
        let mut new_en = char_aep;
        new_en.cterm_ae_attr = if prim_aep.cterm_ae_attr.has(HlAttrFlags::NOCOMBINE) {
            prim_aep.cterm_ae_attr
        } else {
            hl_combine_ae(new_en.cterm_ae_attr, prim_aep.cterm_ae_attr)
        };
        new_en.rgb_ae_attr = if prim_aep.rgb_ae_attr.has(HlAttrFlags::NOCOMBINE) {
            prim_aep.rgb_ae_attr
        } else {
            hl_combine_ae(new_en.rgb_ae_attr, prim_aep.rgb_ae_attr)
        };

        // Taking a colour from `prim_aep` takes its "this is a palette index"
        // bit with it, which is why each of these clears the flag unless the
        // overriding set had it too.
        let inherit = |mask: &mut HlAttrFlags, flag: HlAttrFlags| {
            if !prim_aep.rgb_ae_attr.has(flag) {
                mask.clear(flag);
            }
        };
        if prim_aep.cterm_fg_color > 0 {
            new_en.cterm_fg_color = prim_aep.cterm_fg_color;
            inherit(&mut new_en.rgb_ae_attr, HlAttrFlags::FG_INDEXED);
        }
        if prim_aep.cterm_bg_color > 0 {
            new_en.cterm_bg_color = prim_aep.cterm_bg_color;
            inherit(&mut new_en.rgb_ae_attr, HlAttrFlags::BG_INDEXED);
        }
        if prim_aep.rgb_fg_color >= 0 {
            new_en.rgb_fg_color = prim_aep.rgb_fg_color;
            inherit(&mut new_en.rgb_ae_attr, HlAttrFlags::FG_INDEXED);
        }
        if prim_aep.rgb_bg_color >= 0 {
            new_en.rgb_bg_color = prim_aep.rgb_bg_color;
            inherit(&mut new_en.rgb_ae_attr, HlAttrFlags::BG_INDEXED);
        }
        if prim_aep.rgb_sp_color >= 0 {
            new_en.rgb_sp_color = prim_aep.rgb_sp_color;
        }
        if prim_aep.hl_blend >= 0 {
            new_en.hl_blend = prim_aep.hl_blend;
        }
        // A URL already on the cell is not replaced by the one overlaying it.
        if new_en.url == -1 && prim_aep.url >= 0 {
            new_en.url = prim_aep.url;
        }

        let id = get_attr_entry(HlEntry {
            attr: new_en,
            kind: kHlCombine,
            id1: char_attr,
            id2: prim_attr,
        });
        if id > 0 {
            (*combine_cache()).insert(char_attr, prim_attr, id);
        }
        id
    }
}

/// The number of ids handed out so far, counting the id-0 sentinel. Every id
/// below this is a live entry.
pub fn attr_entry_count() -> c_int {
    ATTRS.with(AttrTable::len) as c_int
}

/// The attributes id `attr` names.
///
/// Unlike [`AttrTable::at`] this tolerates any integer: an id past the end
/// means the tables were rebuilt under the caller, and the empty set is the
/// answer the drawing code expects for it.
#[inline]
pub fn syn_attr2entry(attr: c_int) -> HlAttrs {
    ATTRS.with(|attrs| attrs.live(attr).map_or(HLATTRS_INIT, |entry| entry.attr))
}

/// What id `attr` was built from, as the `ext_hlstate` UI event carries it:
/// an array of `{ kind, hi_name, ui_name, id }` dicts, innermost first.
///
/// Empty unless some UI asked for `ext_hlstate`.
///
/// # Safety
/// `arena` is null or a live arena; main thread only.
pub unsafe fn hl_inspect(attr: c_int, arena: *mut Arena) -> Array {
    if !HLSTATE_ACTIVE.get() {
        return Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut(),
        };
    }
    // SAFETY: the caller's arena.
    unsafe {
        let mut ret = arena_array(arena, hl_inspect_size(attr));
        hl_inspect_impl(&mut ret, attr, arena);
        ret
    }
}

/// How many entries [`hl_inspect_impl`] will produce for `attr`. Combinations
/// are flattened, so this recurses exactly as the filling does.
fn hl_inspect_size(attr: c_int) -> usize {
    let Some(entry) = ATTRS.with(|attrs| attrs.live(attr)) else {
        return 0;
    };
    match entry.kind {
        kHlCombine | kHlBlend | kHlBlendThrough => {
            hl_inspect_size(entry.id1) + hl_inspect_size(entry.id2)
        }
        _ => 1,
    }
}

/// Appends `attr`'s provenance to `arr`.
///
/// # Safety
/// `arr` must have room for [`hl_inspect_size`] more entries, and `arena` is
/// null or live.
unsafe fn hl_inspect_impl(arr: &mut Array, attr: c_int, arena: *mut Arena) {
    let Some(entry) = ATTRS.with(|attrs| attrs.live(attr)) else {
        return;
    };
    // SAFETY: the caller's arena and array.
    unsafe {
        let mut item = match entry.kind {
            kHlSyntax => {
                let mut item = arena_dict(arena, 3);
                put(&mut item, c"kind", Object::literal("syntax"));
                put(&mut item, c"hi_name", name_object(syn_id2name(entry.id1)));
                item
            }
            kHlUI => {
                let mut item = arena_dict(arena, 4);
                put(&mut item, c"kind", Object::literal("ui"));
                // -1 is `Normal`, which is not one of the `hlf_names`.
                let ui_name = if entry.id1 == -1 {
                    c"Normal".as_ptr()
                } else {
                    hlf_names[entry.id1 as usize]
                };
                put(&mut item, c"ui_name", name_object(ui_name));
                put(&mut item, c"hi_name", name_object(syn_id2name(entry.id2)));
                item
            }
            kHlTerminal => {
                let mut item = arena_dict(arena, 2);
                put(&mut item, c"kind", Object::literal("term"));
                item
            }
            kHlCombine | kHlBlend | kHlBlendThrough => {
                // Combination is associative, so flatten it to an array.
                hl_inspect_impl(arr, entry.id1, arena);
                hl_inspect_impl(arr, entry.id2, arena);
                return;
            }
            // kHlUnknown and kHlInvalid: nothing to say about the entry.
            _ => return,
        };
        put(&mut item, c"id", Object::integer(Integer::from(attr)));
        *arr.items.add(arr.size) = Object::dict(item);
        arr.size += 1;
    }
}

/// A group name, borrowed rather than copied.
///
/// # Safety
/// `name` is null or NUL-terminated.
unsafe fn name_object(name: *const c_char) -> Object {
    // SAFETY: the caller's string.
    Object::string(unsafe { cstr_as_string(name) })
}

/// Appends `key: value` to an arena dict.
///
/// # Safety
/// `dict.items` must have room for one more entry.
unsafe fn put(dict: &mut Dict, key: &'static CStr, value: Object) {
    assert!(dict.size < dict.capacity, "hl_inspect dict overflow");
    // SAFETY: the assert above kept the index inside the arena block.
    unsafe {
        *dict.items.add(dict.size) = KeyValuePair {
            key: static_cstring(key),
            value,
        };
    }
    dict.size += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_deduplicates_and_numbers_from_zero() {
        let mut table = AttrTable::new();
        let entry = |id1| HlEntry {
            attr: HLATTRS_INIT,
            kind: kHlSyntax,
            id1,
            id2: 0,
        };
        assert_eq!(table.put(entry(1)), (0, true));
        assert_eq!(table.put(entry(2)), (1, true));
        assert_eq!(table.put(entry(1)), (0, false));
        assert_eq!(table.len(), 2);
        assert_eq!(table.at(1).id1, 2);
        assert!(table.live(0).is_none());
        assert!(table.live(2).is_none());
        table.clear();
        assert_eq!(table.put(entry(2)), (0, true));
    }

    #[test]
    fn urls_are_interned_by_value() {
        let mut table = UrlTable::new();
        assert_eq!(table.intern(c"https://a"), 0);
        assert_eq!(table.intern(c"https://b"), 1);
        assert_eq!(table.intern(c"https://a"), 0);
        table.clear();
        assert_eq!(table.intern(c"https://b"), 0);
    }

    #[test]
    fn combining_underline_styles_replaces_rather_than_ors() {
        assert_eq!(
            hl_combine_ae(HlAttrFlags::UNDERLINE, HlAttrFlags::UNDERCURL),
            HlAttrFlags::UNDERCURL
        );
        assert_eq!(
            hl_combine_ae(HlAttrFlags::UNDERCURL, HlAttrFlags::NONE),
            HlAttrFlags::UNDERCURL
        );
        assert_eq!(
            hl_combine_ae(HlAttrFlags::BOLD, HlAttrFlags::ITALIC),
            HlAttrFlags::BOLD | HlAttrFlags::ITALIC
        );
    }
}
