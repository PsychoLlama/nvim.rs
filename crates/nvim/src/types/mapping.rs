#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;
use core::ffi::{CStr, c_char};
use std::rc::Rc;

/// The NUL an empty [`MapStr`] hands out instead of owning one.
static EMPTY_MAP_STR: [u8; 1] = [0];

/// An owned, NUL-terminated byte string: what the mapping tables keep instead
/// of the `xstrdup`ed `char *` upstream carries.
///
/// The buffer always ends in a NUL, so [`MapStr::as_ptr`] hands a C string to
/// the callees that still take one, and [`MapStr::as_bytes`] is everything
/// before it.  Mapping text never contains an interior NUL — a typed NUL
/// reaches the tables as the three-byte `K_SPECIAL` escape — so the two views
/// agree.
///
/// The empty string owns nothing.  That is not a micro-optimisation: a
/// `<Nop>` mapping's RHS is empty, and the mapping table's per-entry heap is
/// exactly what `inbench`'s bucket walk measures (see the module docs of
/// `mapping`).
pub(crate) struct MapStr(Option<Box<[u8]>>);

impl MapStr {
    /// Copy `bytes`, appending the NUL.
    pub(crate) fn new(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            return Self(None);
        }
        let mut owned = Vec::with_capacity(bytes.len() + 1);
        owned.extend_from_slice(bytes);
        owned.push(0);
        Self(Some(owned.into_boxed_slice()))
    }

    /// The empty string, which owns nothing.
    pub(crate) fn empty() -> Self {
        Self(None)
    }

    /// The content, without the trailing NUL.
    #[inline]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        match &self.0 {
            Some(owned) => &owned[..owned.len() - 1],
            None => &[],
        }
    }

    /// The content *with* its NUL, which is how an index-for-pointer
    /// translation keeps a walk that stops on the terminator byte-exact.
    pub(crate) fn as_bytes_with_nul(&self) -> &[u8] {
        self.0.as_deref().unwrap_or(&EMPTY_MAP_STR)
    }

    /// The content as a C string, which is what the message layer takes.
    pub(crate) fn as_cstr(&self) -> &CStr {
        crate::cstr::in_bytes(self.as_bytes_with_nul())
    }

    /// A C string for the callees that still take one.
    pub(crate) fn as_ptr(&self) -> *const c_char {
        self.as_bytes_with_nul().as_ptr().cast()
    }

    /// How many bytes the content is.
    pub(crate) fn len(&self) -> usize {
        self.0.as_deref().map_or(0, |owned| owned.len() - 1)
    }

    /// Whether the content is empty.
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_none()
    }
}

impl Clone for MapStr {
    fn clone(&self) -> Self {
        Self::new(self.as_bytes())
    }
}

/// A Lua callback a mapping calls instead of inserting its right-hand side.
///
/// The one part of a mapping a simplified pair genuinely has to *share*: a
/// reference into Lua's registry cannot be duplicated the way a string can,
/// and it must be released exactly once.  `impl Drop` is in `mapping::table`,
/// because releasing it is the one thing about a mapping that is not plain
/// memory.  A mapping without a callback pays nothing for this.
pub(crate) struct MapCallback(pub(crate) LuaRef);

/// The right-hand side of a mapping: three strings and an optional callback.
///
/// Held *by value* on the entry.  Upstream keeps one copy per simplified pair
/// and leaves it owned by whichever of the two is freed second, using `m_alt`
/// as the "the other one still owns them" flag; here the twin gets its own
/// copy of the strings — the pair is rare, the strings are short, and a
/// shared box would be an extra allocation and an extra indirection on every
/// entry of every bucket walk.  Only [`MapCallback`] is shared, and `m_alt`
/// is now no more than the twin's address.
pub(crate) struct MapRhs {
    /// The RHS in typeahead form: termcodes replaced, `K_SPECIAL` escaped.
    pub(crate) str: MapStr,
    /// The RHS as it was written, which is `maparg()`'s compatible answer.
    pub(crate) orig_str: MapStr,
    /// `:map <desc>`'s text, or `None`.
    pub(crate) desc: Option<MapStr>,
    /// The Lua callback, shared with the twin.
    pub(crate) lua: Option<Rc<MapCallback>>,
}

impl MapRhs {
    /// A second entry's copy: the strings are duplicated, the callback is
    /// shared.
    pub(crate) fn dup(&self) -> Self {
        Self {
            str: self.str.clone(),
            orig_str: self.orig_str.clone(),
            desc: self.desc.clone(),
            lua: self.lua.clone(),
        }
    }

    /// The Lua callback's reference, or `LUA_NOREF`.
    pub(crate) fn luaref(&self) -> LuaRef {
        self.lua
            .as_ref()
            .map_or(crate::mapping::LUA_NOREF, |lua| lua.0)
    }
}

/// A mapping's identity, which no other mapping ever takes, even after it
/// is deleted.
///
/// What a holder keeps instead of an address: a mapping can be deleted
/// under anything that lets user code run (an `<expr>` that `:unmap`s
/// itself), and a lookup by id then answers `None` where an address would
/// dangle.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct MapId(pub(crate) u64);

/// The unsimplified twin of a simplified mapping (or the other way round):
/// its id, and the first byte of its LHS, which is what names the two hash
/// buckets it can be in.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct MapTwin {
    pub(crate) id: MapId,
    pub(crate) first: u8,
}

/// One mapping or abbreviation.
///
/// Owned by the list it is on -- a hash bucket of a [`MapTables`], or its
/// abbreviation list -- by value. The twin of a `<C-H>`-style pair is named
/// by id, not by address; everything else the entry holds is its own.
pub struct MapBlock {
    /// This entry's identity.
    pub(crate) m_id: MapId,
    /// The unsimplified twin of a simplified mapping, or the other way round.
    pub(crate) m_alt: Option<MapTwin>,
    /// The LHS, in typeahead form.
    pub(crate) m_keys: MapStr,
    /// The RHS: three strings and, when there is one, a shared callback.
    pub(crate) m_rhs: MapRhs,
    /// The modes this mapping applies in.
    pub(crate) m_mode: ::core::ffi::c_int,
    /// Whether this is the `<C-H>`-style simplified half of a pair.
    pub(crate) m_simplified: bool,
    /// `REMAP_YES`, `REMAP_NONE` or `REMAP_SCRIPT`.
    pub(crate) m_noremap: ::core::ffi::c_int,
    pub(crate) m_silent: bool,
    pub(crate) m_nowait: bool,
    pub(crate) m_expr: bool,
    pub(crate) m_script_ctx: ScriptCtx,
    pub(crate) m_replace_keycodes: bool,
}

/// A copy for a listing that has to let go of the table before it prints:
/// the strings are duplicated, the callback is shared, the identity kept.
impl Clone for MapBlock {
    fn clone(&self) -> Self {
        Self {
            m_id: self.m_id,
            m_alt: self.m_alt,
            m_keys: self.m_keys.clone(),
            m_rhs: self.m_rhs.dup(),
            m_mode: self.m_mode,
            m_simplified: self.m_simplified,
            m_noremap: self.m_noremap,
            m_silent: self.m_silent,
            m_nowait: self.m_nowait,
            m_expr: self.m_expr,
            m_script_ctx: self.m_script_ctx,
            m_replace_keycodes: self.m_replace_keycodes,
        }
    }
}

impl MapBlock {
    /// The LHS, without its NUL.
    pub(crate) fn keys(&self) -> &[u8] {
        self.m_keys.as_bytes()
    }

    /// The RHS, without its NUL.
    pub(crate) fn rhs(&self) -> &[u8] {
        self.m_rhs.str.as_bytes()
    }

    /// The Lua callback, or `LUA_NOREF`.
    pub(crate) fn luaref(&self) -> LuaRef {
        self.m_rhs.luaref()
    }

    /// How this entry is named from its twin.
    pub(crate) fn as_twin(&self) -> MapTwin {
        MapTwin {
            id: self.m_id,
            first: self.keys().first().copied().unwrap_or(0),
        }
    }
}

/// One mapping table: [`MAX_MAPHASH`](crate::mapping::MAX_MAPHASH) hash
/// buckets and the abbreviation list. The editor has one globally and one per
/// buffer (`b_maps`).
///
/// Each list keeps its entries oldest first, so a new mapping is a `push` and
/// upstream's head-first walk -- newest first, which is the order `:map`
/// lists in -- is a reverse iteration. The entries are boxed so that
/// deleting one from a long bucket shifts pointers, not mappings. The buckets are allocated on the first
/// mapping, so a buffer without any costs two empty vectors.
#[expect(
    clippy::vec_box,
    reason = "boxed so that deleting from a 5,000-entry bucket shifts pointers"
)]
pub(crate) struct MapTables {
    pub(crate) hash: Vec<Vec<Box<MapBlock>>>,
    pub(crate) abbr: Vec<Box<MapBlock>>,
    /// The id the next mapping gets, kept by the global table for every
    /// table: ids are never reused, so one kept across user code can only
    /// find the mapping it was taken from.
    pub(crate) next_id: u64,
}

impl MapTables {
    /// No mappings at all.
    pub(crate) const fn new() -> Self {
        Self {
            hash: Vec::new(),
            abbr: Vec::new(),
            next_id: 1,
        }
    }
}

impl Default for MapTables {
    fn default() -> Self {
        Self::new()
    }
}
