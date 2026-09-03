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
use core::ffi::c_char;
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
    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.as_bytes_with_nul()
            .split_last()
            .map_or(&[], |(_nul, text)| text)
    }

    /// The content *with* its NUL, which is how an index-for-pointer
    /// translation keeps a walk that stops on the terminator byte-exact.
    pub(crate) fn as_bytes_with_nul(&self) -> &[u8] {
        self.0.as_deref().unwrap_or(&EMPTY_MAP_STR)
    }

    /// A C string for the callees that still take one.
    pub(crate) fn as_ptr(&self) -> *const c_char {
        self.as_bytes_with_nul().as_ptr().cast()
    }

    /// A C string for the callees whose signature says `*mut` but that only
    /// read (`ins_typebuf`, `msg_outtrans_special`).
    pub(crate) fn as_mut_ptr(&self) -> *mut c_char {
        self.as_ptr().cast_mut()
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

/// One mapping or abbreviation.
///
/// The two `*mut mapblock_T` are the intrusive list links, not ownership: an
/// entry is a `Box` its list holds by raw pointer, because both the
/// delete-walk and `getchar`'s match loop keep the address of a link across
/// calls.  Everything else the entry owns is typed — see [`MapStr`] and
/// [`MapRhs`].
pub struct mapblock {
    /// The next entry on the same hash bucket or abbreviation list.
    pub(crate) m_next: *mut mapblock_T,
    /// The unsimplified twin of a simplified mapping, or null.
    pub(crate) m_alt: *mut mapblock_T,
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
    pub(crate) m_script_ctx: sctx_T,
    pub(crate) m_replace_keycodes: bool,
}

impl mapblock {
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
}

pub type mapblock_T = mapblock;
