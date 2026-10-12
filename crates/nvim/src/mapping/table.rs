//! The mapping table itself: the hash buckets and the abbrlist.
//!
//! Every mapping is a [`MapBlock`] in one of [`MAX_MAPHASH`] lists, hashed by
//! [`map_hash`] on the first byte of its LHS and on whether the mode is a
//! Normal-side or an Insert-side one; abbreviations live on one unhashed
//! list instead.  Both tables exist twice: once globally, in
//! [`GLOBAL_MAPS`], and once per buffer in `b_maps`.  A [`MapTable`] names
//! one of them and lends it for one closure.
//!
//! The functions here create ([`map_add`]), destroy ([`map_clear_mode`]) and
//! search ([`check_map`], [`map_to_exists_mode`]) those lists.  A read-only
//! whole-table walk is [`map_walk`].
//!
//! The lists own their entries by value (see [`MapTables`]).  What outlives
//! a borrow of the table -- the match the typeahead found, a simplified
//! mapping's twin -- is held as a [`MapId`] and re-found, so a mapping
//! user code deleted in between answers `None` rather than dangling.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::ex_docmd::sourcing_lnum;
use crate::keycodes::Ctrl_C;
use crate::lua::executor::{nlua_set_sctx_in, release_luaref};
use crate::types::{MapId, MapTables, MapTwin};
use crate::winlayer::Buf;
use core::ffi::c_int;

/// A callback is released once, when the last of a simplified pair lets go
/// of it.
impl Drop for MapCallback {
    fn drop(&mut self) {
        // A reference this value owns, and nothing else can hold an `Rc` to
        // it any more -- this is the last one dropping.
        release_luaref(self.0);
        self.0 = LUA_NOREF;
    }
}

/// The global mapping table and abbreviation list, the global twin of a
/// buffer's `b_maps`.
pub(crate) static GLOBAL_MAPS: GlobalCell<MapTables> = GlobalCell::new(MapTables::new());

/// A fresh mapping id, from the global table's counter.
fn next_map_id() -> MapId {
    GLOBAL_MAPS.with_mut(|maps| {
        let id = maps.next_id;
        maps.next_id += 1;
        MapId(id)
    })
}

/// The modes whose mappings hash on the LHS byte itself.
const NORMAL_SIDE: c_int =
    MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING | MODE_TERMINAL;

/// C's `MAP_HASH`: which bucket a mapping in `mode` whose LHS starts with
/// byte `c1` belongs in.
///
/// Insert- and Cmdline-side mappings hash on the byte with its top bit
/// flipped, which keeps them mostly out of the Normal-side buckets.  `c1` is
/// always a single byte at every call site, so both answers are in range.
pub(crate) fn map_hash(mode: c_int, c1: c_int) -> usize {
    let bucket = if mode & NORMAL_SIDE != 0 {
        c1
    } else {
        c1 ^ 0x80
    };
    usize::try_from(bucket).expect("a mapping's first byte is in range")
}

impl MapTables {
    /// One list in storage order -- oldest first.  `hash` is ignored for the
    /// abbreviation list.
    pub(crate) fn list(&self, hash: usize, abbr: bool) -> &[Box<MapBlock>] {
        if abbr {
            &self.abbr
        } else {
            self.hash.get(hash).map_or(&[], Vec::as_slice)
        }
    }

    /// [`MapTables::list`] for writing, allocating the buckets on the first
    /// mapping.
    #[expect(clippy::vec_box, reason = "the lists' own type, see `MapTables`")]
    pub(crate) fn list_mut(&mut self, hash: usize, abbr: bool) -> &mut Vec<Box<MapBlock>> {
        if abbr {
            return &mut self.abbr;
        }
        if self.hash.is_empty() {
            self.hash.resize_with(MAX_MAPHASH, Vec::new);
        }
        &mut self.hash[hash]
    }

    /// Every entry of the mappings or the abbreviations in upstream's
    /// order: the buckets in turn, each newest first.
    pub(crate) fn walk(&self, abbr: bool) -> impl Iterator<Item = &MapBlock> {
        let lists = if abbr { 1 } else { MAX_MAPHASH };
        (0..lists).flat_map(move |hash| self.list(hash, abbr).iter().rev().map(|mp| &**mp))
    }

    /// Where `twin` is: its list and index.  The twin of a pair lives in the
    /// same table and the same kind of list, in one of the two buckets its
    /// first byte can hash to.
    fn locate(&self, abbr: bool, twin: MapTwin) -> Option<(usize, usize)> {
        let first = usize::from(twin.first);
        let buckets = [first, first ^ 0x80];
        let buckets = if abbr { &buckets[..1] } else { &buckets[..] };
        buckets.iter().copied().find_map(|hash| {
            let at = self
                .list(hash, abbr)
                .iter()
                .position(|mp| mp.m_id == twin.id)?;
            Some((hash, at))
        })
    }

    /// The entry `twin` names, if it is still there.
    pub(crate) fn find(&self, abbr: bool, twin: MapTwin) -> Option<&MapBlock> {
        let (hash, at) = self.locate(abbr, twin)?;
        self.list(hash, abbr).get(at).map(|mp| &**mp)
    }

    /// The LHS of `mp`'s twin, if it has one.
    pub(crate) fn twin_keys(&self, abbr: bool, mp: &MapBlock) -> Option<&MapStr> {
        Some(&self.find(abbr, mp.m_alt?)?.m_keys)
    }

    /// Clear the back-link of the twin `twin` names, which is losing its
    /// other half.
    fn unlink_twin(&mut self, abbr: bool, twin: MapTwin) {
        if let Some((hash, at)) = self.locate(abbr, twin) {
            self.list_mut(hash, abbr)[at].m_alt = None;
        }
    }

    /// Delete one entry, cutting its twin's link to it.
    ///
    /// The entry's strings go with it: the LHS is its own, and the Lua
    /// callback is released only when its twin has let go too.
    pub(crate) fn remove_at(&mut self, hash: usize, abbr: bool, at: usize) {
        let mp = self.list_mut(hash, abbr).remove(at);
        if let Some(twin) = mp.m_alt {
            self.unlink_twin(abbr, twin);
        }
    }

    /// Move one entry to the front of bucket `to` (the back of its list).
    ///
    /// This is the re-hash a `:unmap` of some of an entry's modes forces, and
    /// pushing to the front is what upstream does -- `:map`'s listing order
    /// is Vim-visible, so it is reproduced exactly.
    pub(crate) fn relink(&mut self, from: usize, at: usize, to: usize) {
        let mp = self.list_mut(from, false).remove(at);
        self.list_mut(to, false).push(mp);
    }

    /// Cut the pair `mp`'s entry is half of, on both halves.
    pub(crate) fn cut_pair(&mut self, hash: usize, abbr: bool, at: usize) {
        if let Some(twin) = self.list_mut(hash, abbr)[at].m_alt.take() {
            self.unlink_twin(abbr, twin);
        }
    }

    /// Make the two entries `a` and `b` name each other as twins.
    pub(crate) fn pair(&mut self, abbr: bool, a: MapTwin, b: MapTwin) {
        for (this, other) in [(a, b), (b, a)] {
            if let Some((hash, at)) = self.locate(abbr, this) {
                self.list_mut(hash, abbr)[at].m_alt = Some(other);
            }
        }
    }
}

/// Which pair of tables a walk reads: the global one, or a buffer's.
#[derive(Copy, Clone)]
pub(crate) enum MapTable {
    Global,
    Buffer(Buf),
}

impl MapTable {
    /// The global table, or `buffer`'s when `local`.
    pub(crate) fn of(local: bool, buffer: Buf) -> Self {
        if local {
            MapTable::Buffer(buffer)
        } else {
            MapTable::Global
        }
    }

    /// Whether this is a buffer's table.
    pub(crate) fn is_local(self) -> bool {
        matches!(self, MapTable::Buffer(_))
    }

    /// Lend the table for reading.
    ///
    /// `f` must not reach the mapping tables itself: the global cell checks
    /// that in debug builds, and a buffer's table is borrowed out of the
    /// buffer for the call.
    pub(crate) fn with<R>(self, f: impl FnOnce(&MapTables) -> R) -> R {
        match self {
            MapTable::Global => GLOBAL_MAPS.with(f),
            MapTable::Buffer(buf) => match buf.b_maps.as_deref() {
                Some(tables) => f(tables),
                // A buffer that never had a local mapping.
                None => f(&MapTables::new()),
            },
        }
    }

    /// Lend the table for writing; the same rule as [`MapTable::with`].
    pub(crate) fn with_mut<R>(self, f: impl FnOnce(&mut MapTables) -> R) -> R {
        match self {
            MapTable::Global => GLOBAL_MAPS.with_mut(f),
            MapTable::Buffer(mut buf) => f(buf.b_maps.get_or_insert_with(Box::default)),
        }
    }
}

/// Visit every entry of `table` in upstream's order, stopping at the first
/// `Some`.
///
/// Upstream writes this loop out at eight sites: all [`MAX_MAPHASH`] buckets
/// for mappings, or the one unhashed list for abbreviations.  `visit` sees
/// the table lent, so it must not reach the mapping tables itself.
pub(crate) fn map_walk<T>(
    table: MapTable,
    abbr: bool,
    mut visit: impl FnMut(&MapBlock) -> Option<T>,
) -> Option<T> {
    table.with(|tables| tables.walk(abbr).find_map(&mut visit))
}

/// Where a mapping the typeahead's match found lives: its table, bucket and
/// index when it was found, and its id.
///
/// The match runs no user code, so the index is normally right; an `<expr>`
/// evaluation can delete the mapping, which is why everything after that
/// re-finds it by id.
#[derive(Copy, Clone)]
pub(crate) struct MapAt {
    table: MapTable,
    hash: usize,
    index: usize,
    id: MapId,
}

impl MapAt {
    /// The entry at `index` of bucket `hash` of `table`.
    pub(crate) fn new(table: MapTable, hash: usize, index: usize, mp: &MapBlock) -> Self {
        Self {
            table,
            hash,
            index,
            id: mp.m_id,
        }
    }

    /// Lend the mapping and its table, or answer `None` when it is gone.
    pub(crate) fn with<R>(self, f: impl FnOnce(&MapTables, &MapBlock) -> R) -> Option<R> {
        self.table.with(|tables| {
            let list = tables.list(self.hash, false);
            let mp = match list.get(self.index) {
                Some(mp) if mp.m_id == self.id => mp,
                _ => list.iter().find(|mp| mp.m_id == self.id)?,
            };
            Some(f(tables, mp))
        })
    }
}

/// Lend the maphash buckets for `state` and first byte `c`: the current
/// buffer's, then the global one, each oldest entry first, with the table it
/// came from.
///
/// The typeahead asks once per key, so both are lent in one borrow, and a
/// buffer without local mappings costs one test.
#[inline(always)]
pub(crate) fn with_maphash_lists<R>(
    state: c_int,
    c: c_int,
    f: impl FnOnce(usize, [(MapTable, &[Box<MapBlock>]); 2]) -> R,
) -> R {
    let hash = map_hash(state, c);
    let buf = Buf::current();
    GLOBAL_MAPS.with(|global| {
        let local = buf
            .b_maps
            .as_deref()
            .map_or(&[][..], |tables| tables.list(hash, false));
        let lists = [
            (MapTable::Buffer(buf), local),
            (MapTable::Global, global.list(hash, false)),
        ];
        f(hash, lists)
    })
}

/// Put a new mapping at the front of its list and answer how its twin will
/// name it.
///
/// `args` supplies the right-hand side and the flags; the RHS bundle is
/// copied, its callback shared.  `sid` of 0 means "use `current_sctx`".
#[allow(clippy::too_many_arguments)] // upstream's; the caller has no struct to pass
pub(crate) fn map_add(
    table: MapTable,
    keys: &[u8],
    args: &MapArguments,
    noremap: c_int,
    mode: c_int,
    is_abbr: bool,
    sid: ScriptId,
    lnum: LineNr,
    simplified: bool,
) -> MapTwin {
    // A given `sid` is upstream's "the block was `xcalloc`ed and only these
    // two fields were filled in", not a tweak of `current_sctx`.
    let script_ctx = if sid != 0 {
        ScriptCtx {
            sc_sid: sid,
            sc_lnum: lnum,
            ..ScriptCtx::NONE
        }
    } else {
        let mut ctx = current_sctx.get();
        ctx.sc_lnum += sourcing_lnum();
        nlua_set_sctx_in(&mut ctx);
        ctx
    };
    let mp = MapBlock {
        m_id: next_map_id(),
        m_alt: None,
        m_keys: MapStr::new(keys),
        m_rhs: args.rhs().dup(),
        m_mode: mode,
        m_simplified: simplified,
        m_noremap: noremap,
        m_silent: args.silent,
        m_nowait: args.nowait,
        m_expr: args.expr,
        m_script_ctx: script_ctx,
        m_replace_keycodes: args.replace_keycodes,
    };
    let twin = mp.as_twin();

    // If CTRL-C has been mapped, don't always use it for Interrupting.
    if keys.first().copied().map(c_int::from) == Some(Ctrl_C) {
        match table {
            MapTable::Buffer(mut buf) => buf.b_mapped_ctrl_c |= mode,
            MapTable::Global => mapped_ctrl_c.set(mapped_ctrl_c.get() | mode),
        }
    }

    // Add the new entry in front of the abbrlist or of its maphash bucket.
    let first = keys.first().copied().unwrap_or(0);
    let hash = map_hash(mode, c_int::from(first));
    table.with_mut(|tables| tables.list_mut(hash, is_abbr).push(Box::new(mp)));
    twin
}

/// Clear all mappings (or abbreviations) in `mode`.
///
/// An entry only loses the bits `mode` names; it is deleted when that leaves
/// it with no mode at all, and re-hashed when the bits that are left move it
/// to another bucket.
pub fn map_clear_mode(buffer: Buf, mode: c_int, local: bool, abbr: bool) {
    if local && buffer.b_maps.is_none() {
        return; // never had a local mapping
    }
    MapTable::of(local, buffer).with_mut(|tables| {
        for hash in 0..if abbr { 1 } else { MAX_MAPHASH } {
            // Newest first, as upstream walks; an index below the one being
            // looked at is unmoved by a removal or a re-hash at it.
            let mut at = tables.list(hash, abbr).len();
            while at > 0 {
                at -= 1;
                let mp = &mut tables.list_mut(hash, abbr)[at];
                if mp.m_mode & mode == 0 {
                    continue;
                }
                let left = mp.m_mode & !mode;
                mp.m_mode = left;
                if left == 0 {
                    tables.remove_at(hash, abbr, at);
                    continue;
                }
                // May need to put this entry into another hash list.
                let first = mp.keys().first().copied().unwrap_or(0);
                let new_hash = map_hash(left, c_int::from(first));
                if !abbr && new_hash != hash {
                    tables.relink(hash, at, new_hash);
                }
            }
        }
    });
}

/// Which mode each `hasmapto()`-style mode character stands for.
const MODE_CHARS: [(u8, c_int); 8] = [
    (b'n', MODE_NORMAL),
    (b'v', MODE_VISUAL | MODE_SELECT),
    (b'x', MODE_VISUAL),
    (b's', MODE_SELECT),
    (b'o', MODE_OP_PENDING),
    (b'i', MODE_INSERT),
    (b'l', MODE_LANGMAP),
    (b'c', MODE_CMDLINE),
];

/// Whether any mapping in the modes `modechars` names has `text` in its RHS.
///
/// Termcap codes are recognised in `text`.  Buffer-local mappings count.
pub(crate) fn map_to_exists(text: &[u8], modechars: &[u8], abbr: bool) -> bool {
    let dolt = REPTERM_DO_LT.cast_signed();
    let rhs = p_cpo(|cpo| replace_termcodes(text, 0, dolt, None, cpo, None)).unwrap_or_default();

    let mut mode = 0;
    for (ch, flags) in MODE_CHARS {
        if modechars.contains(&ch) {
            mode |= flags;
        }
    }

    // A C reader of the translation stops at its first NUL.
    map_to_exists_mode(
        rhs.split(|&b| b == 0).next().unwrap_or_default(),
        mode,
        abbr,
    )
}

/// C's `strstr`, over slices: whether `haystack` contains `needle`.
///
/// The empty needle matches, as `strstr`'s does.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    needle.is_empty()
        || (haystack.len() >= needle.len() && haystack.windows(needle.len()).any(|at| at == needle))
}

/// Whether any mapping in `mode` has `rhs` as a substring of its RHS.
///
/// Global mappings are searched first, then the current buffer's.
pub fn map_to_exists_mode(rhs: &[u8], mode: c_int, abbr: bool) -> bool {
    let cur = Buf::current();
    // Do it twice: once for global maps and once for local maps.
    [MapTable::Global, MapTable::Buffer(cur)]
        .into_iter()
        .any(|table| {
            let visit =
                |mp: &MapBlock| (mp.m_mode & mode != 0 && contains(mp.rhs(), rhs)).then_some(());
            map_walk(table, abbr, visit).is_some()
        })
}

/// Check `keys` against the LHS of every mapping in `mode`, and lend the
/// first that matches to `found`, with whether it came from the buffer-local
/// table.
///
/// Buffer-local mappings are searched first.  Without `exact` a mapping whose
/// LHS is merely a *prefix* of `keys` — or the other way round — counts, which
/// is what `mapcheck()` asks for.  `ign_mod` skips a leading modifier escape
/// in the stored LHS before comparing.
pub(crate) fn check_map<R>(
    keys: &[u8],
    mode: c_int,
    exact: bool,
    ign_mod: bool,
    abbr: bool,
    found: impl FnOnce(&MapBlock, bool) -> R,
) -> Option<R> {
    let matches = |mp: &MapBlock| {
        // Skip entries with the wrong mode, the wrong length, and the ones
        // that do not match.
        if mp.m_mode & mode == 0 {
            return false;
        }
        let mut lhs = mp.keys();
        if exact && lhs.len() != keys.len() {
            return false;
        }
        // A three-byte modifier escape at the front of the stored LHS is
        // skipped when the caller asked for it.
        let modifier = matches!(lhs, [a, b, _, ..]
            if c_int::from(*a) == K_SPECIAL && c_int::from(*b) == KS_MODIFIER);
        if ign_mod && modifier {
            lhs = &lhs[3..];
        }
        let minlen = lhs.len().min(keys.len());
        lhs[..minlen] == keys[..minlen]
    };
    let mut found = Some(found);
    for (local, table) in [
        (true, MapTable::Buffer(Buf::current())),
        (false, MapTable::Global),
    ] {
        let answer = table.with(|tables| {
            let lists = if abbr { 1 } else { MAX_MAPHASH };
            let mp = (0..lists).find_map(|hash| {
                let list = tables.list(hash, abbr);
                list.iter().rev().find(|mp| matches(mp))
            })?;
            Some(found.take().expect("lent once")(mp, local))
        });
        if answer.is_some() {
            return answer;
        }
    }
    None
}
