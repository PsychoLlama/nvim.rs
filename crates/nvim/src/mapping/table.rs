//! The mapping table itself: the hash buckets and the abbrlist.
//!
//! Every mapping is a [`mapblock_T`] on one of [`MAX_MAPHASH`] singly linked
//! lists, hashed by [`map_hash`] on the first byte of its LHS and on whether
//! the mode is a Normal-side or an Insert-side one; abbreviations live on one
//! unhashed list instead.  Both tables exist twice: once globally, in
//! [`MAPHASH`] and [`FIRST_ABBR`], and once per buffer in `b_maphash` and
//! `b_first_abbr`.
//!
//! The functions here create ([`map_add`]), destroy ([`mapblock_free`],
//! [`map_clear_mode`]) and search ([`check_map`], [`map_to_exists_mode`])
//! those lists.  A read-only whole-table walk is [`map_walk`]; the two
//! functions that *delete* while walking hold a [`Cursor`], which owns the
//! link arithmetic an iterator cannot hand out — the address of the previous
//! entry's `m_next`, so that removing and re-hashing are ordinary calls.
//!
//! The entries themselves are `Box`es the lists hold by raw pointer; see the
//! parent module's ownership note.  `impl Drop for MapRhs` lives here rather
//! than beside the struct because releasing a Lua reference is the one part
//! of an RHS that is not plain memory.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::ex_docmd::sourcing_lnum;
use crate::keycodes::Ctrl_C;
use crate::winlayer::Buf;
use core::ffi::{c_char, c_int};
use core::mem::offset_of;
use core::ptr;

/// A NUL-terminated string an FFI callee allocated and handed back.
///
/// Nothing in `mapping/` is freed by hand; this is the one seam where the
/// *other* allocator's memory crosses back into the module — what the survey
/// calls an ABI-crossing free — and giving it a destructor is what keeps the
/// `xfree` out of the code that reads the answer.  `replace_termcodes`,
/// `eval_to_string`, `vim_strsave_escape_ks` and `nlua_funcref_str` all
/// answer this way.
pub(crate) struct COwned(*mut c_char);

impl COwned {
    /// # Safety
    /// `p` must be null, or a NUL-terminated allocation this takes over.
    pub(crate) unsafe fn new(p: *mut c_char) -> Self {
        Self(p)
    }

    /// The bytes, or `None` when the callee answered null.
    pub(crate) fn as_bytes(&self) -> Option<&[u8]> {
        // SAFETY: the constructor's promise -- a NUL-terminated allocation
        // this value owns, so it is live for the borrow.
        (!self.0.is_null()).then(|| unsafe { cstr::bytes_at(self.0) })
    }

    /// The pointer, for a callee that reads it straight back; null when the
    /// answer was null.
    pub(crate) fn as_c_ptr(&self) -> *const c_char {
        self.0
    }

    /// An owned copy, or `None` when the callee answered null.
    pub(crate) fn to_map_str(&self) -> Option<MapStr> {
        self.as_bytes().map(MapStr::new)
    }
}

impl Drop for COwned {
    fn drop(&mut self) {
        // SAFETY: the constructor's promise -- this value owns the block, and
        // `xfree` accepts a null pointer.
        unsafe { xfree(self.0.cast()) };
    }
}

/// A callback is released once, when the last of a simplified pair lets go
/// of it.
impl Drop for MapCallback {
    fn drop(&mut self) {
        // SAFETY: a reference this value owns, and nothing else can hold an
        // `Rc` to it any more -- this is the last one dropping.
        unsafe { api_free_luaref(self.0) };
        self.0 = LUA_NOREF;
    }
}

/// The global abbreviation list; `b_first_abbr` is its per-buffer twin.
pub(crate) static FIRST_ABBR: GlobalCell<*mut mapblock_T> = GlobalCell::new(ptr::null_mut());

/// The global mapping table; `b_maphash` is its per-buffer twin.
pub(crate) static MAPHASH: GlobalCell<[*mut mapblock_T; MAX_MAPHASH]> =
    GlobalCell::new([ptr::null_mut(); MAX_MAPHASH]);

/// The global mapping table as a row of list *heads*, one per hash bucket.
///
/// A `*mut *mut mapblock_T` rather than a borrow: the whole family walks and
/// unlinks through the link itself, and a `&mut` to the array would
/// invalidate a cursor that points into it — which is what
/// [`map_clear_mode`] does when a re-hash moves an entry.
pub(crate) fn global_map_heads() -> *mut *mut mapblock_T {
    MAPHASH.ptr().cast()
}

/// The head of the global abbreviation list. See [`global_map_heads`].
pub(crate) fn global_abbr_head() -> *mut *mut mapblock_T {
    FIRST_ABBR.ptr()
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
    (if mode & NORMAL_SIDE != 0 {
        c1
    } else {
        c1 ^ 0x80
    }) as usize
}

/// Get the start of the hashed map list for `state` and first character `c`.
pub fn get_maphash_list(state: c_int, c: c_int) -> *mut mapblock_T {
    MAPHASH.with(|table| table[map_hash(state, c)])
}

/// Get the buffer-local hashed map list for `state` and first character `c`.
///
/// # Safety
/// `curbuf` must be a live buffer.
pub unsafe fn get_buf_maphash_list(state: c_int, c: c_int) -> *mut mapblock_T {
    // SAFETY (this body): the caller's promise -- `curbuf` is a live buffer.
    unsafe { (*curbuf.get()).b_maphash[map_hash(state, c)] }
}

/// Which pair of tables a walk reads: the global one, or a buffer's.
#[derive(Copy, Clone)]
pub(crate) enum MapTable {
    Global,
    Buffer(Buf),
}

impl MapTable {
    /// The head of the list this table keeps at `hash`, or of its single
    /// abbreviation list.
    ///
    /// Safe: the global tables are statics, and a `Buffer` names a live
    /// buffer by [`Buf`]'s promise.
    fn head(self, hash: usize, abbr: bool) -> *mut mapblock_T {
        match (self, abbr) {
            (MapTable::Global, true) => FIRST_ABBR.get(),
            (MapTable::Global, false) => MAPHASH.with(|table| table[hash]),
            (MapTable::Buffer(buf), true) => buf.b_first_abbr,
            (MapTable::Buffer(buf), false) => buf.b_maphash[hash],
        }
    }
}

/// Visit every entry of `table` in upstream's order, stopping at the first
/// `Some`.
///
/// Upstream writes this loop out at eight sites: all [`MAX_MAPHASH`] buckets
/// for mappings, or the one unhashed list for abbreviations, each head read
/// when the walk reaches it.  Only for walks that leave the lists alone.
///
/// # Safety
/// `table` must name live storage, and `visit` must not unlink or free any
/// entry.
pub(crate) unsafe fn map_walk<T>(
    table: MapTable,
    abbr: bool,
    mut visit: impl FnMut(Mb) -> Option<T>,
) -> Option<T> {
    for hash in 0..if abbr { 1 } else { MAX_MAPHASH } {
        let mut head = table.head(hash, abbr);
        while !head.is_null() {
            // SAFETY: a non-null entry of one of `table`'s lists, which the
            // caller has promised `visit` neither unlinks nor frees, so it
            // stays live across the call that reads `m_next` off it.
            let mp = unsafe { Mb::new(head) };
            if let Some(answer) = visit(mp) {
                return Some(answer);
            }
            head = mp.m_next;
        }
    }
    None
}

/// Delete one entry from the abbrlist or [`MAPHASH`].
///
/// `mpp` is the address of the *previous* entry's `m_next` field (or of the
/// list head), which is how the entry is unlinked without walking again.
///
/// The entry's strings go with it: the LHS is its own, and the RHS bundle is
/// released only when its twin has let go too.
///
/// # Safety
/// `mpp` must point at a non-null entry of a live list.
pub(crate) unsafe fn mapblock_free(mpp: *mut *mut mapblock_T) {
    // SAFETY: the caller's promise -- `mpp` names a live link holding a live
    // entry, which this unlinks and then takes back into its `Box`.  `mpp` is
    // the *previous* entry's field, so the write cannot disturb it.
    let mp = unsafe {
        let mp = Box::from_raw(*mpp);
        *mpp = mp.m_next;
        mp
    };
    let alt = mp.m_alt;
    if !alt.is_null() {
        // SAFETY: a non-null `m_alt` is the live twin; clear its back-link so
        // it does not name this entry once the `Box` below is gone.
        unsafe { (*alt).m_alt = ptr::null_mut() };
    }
    drop(mp);
}

/// A position in one mapping list: the address of the link that holds the
/// current entry.
///
/// The delete-walks need what an iterator cannot give them — a handle that
/// survives *removing* what it points at, because upstream's `continue`
/// resumes at `*mpp` rather than at the next entry.  Holding the link rather
/// than the entry is what makes that work, and it is also why a `Cursor` may
/// point into `b_maphash` itself: taking a `&mut buf_T` while one is live
/// would invalidate it, so every table this walks is reached through one raw
/// pointer.
///
/// Construction is the unsafe step, once; every method after it is ordinary
/// checked code.
pub(crate) struct Cursor(*mut *mut mapblock_T);

impl Cursor {
    /// # Safety
    /// `link` must be the address of a live list head or of a live entry's
    /// `m_next`, and must stay live for as long as the cursor is used.
    pub(crate) unsafe fn at(link: *mut *mut mapblock_T) -> Self {
        Self(link)
    }

    /// The entry this link holds, or `None` at the end of the list.
    pub(crate) fn entry(&self) -> Option<Mb> {
        // SAFETY: the constructor's promise -- a live link, whose entry is
        // live until this cursor removes it.
        let mp = unsafe { *self.0 };
        // SAFETY: as above.
        (!mp.is_null()).then(|| unsafe { Mb::new(mp) })
    }

    /// Step past the current entry, which must exist.
    pub(crate) fn advance(&mut self) {
        let mp = self
            .entry()
            .expect("advance past the end of a mapping list");
        self.0 = mp.field_ptr(offset_of!(mapblock_T, m_next));
    }

    /// Unlink and free the current entry; the cursor stays put, now holding
    /// whatever followed it.
    pub(crate) fn remove(&mut self) {
        // SAFETY: the constructor's promise -- a live link holding a live
        // entry, which `mapblock_free` unlinks through this same link.
        unsafe { mapblock_free(self.0) };
    }

    /// Move the current entry to the front of the list at `head`; the cursor
    /// stays put, now holding whatever followed it.
    ///
    /// This is the re-hash a `:unmap` of some of an entry's modes forces, and
    /// pushing to the front is what upstream does — `:map`'s listing order is
    /// Vim-visible, so it is reproduced exactly.
    ///
    /// # Safety
    /// `head` must be the address of a live list head.
    pub(crate) unsafe fn relink_to(&mut self, head: *mut *mut mapblock_T) {
        let mut mp = self.entry().expect("re-hash at the end of a mapping list");
        // SAFETY: the two promises -- this cursor's link and the caller's
        // head are both live, and `mp` is the entry both will hold.
        unsafe {
            *self.0 = mp.m_next;
            mp.m_next = *head;
            *head = mp.raw();
        }
    }
}

/// Put a new mapping at the front of its list and return it.
///
/// `args` supplies the right-hand side and the flags; the RHS bundle is
/// *shared* with the parse, not copied, which is how a simplified mapping and
/// its twin come to hold one `Rc`.  `sid` of 0 means "use `current_sctx`".
///
/// # Safety
/// Both table pointers must name live storage.
#[allow(clippy::too_many_arguments)] // upstream's; the caller has no struct to pass
pub(crate) unsafe fn map_add(
    buf: Buf,
    map_table: *mut *mut mapblock_T,
    abbr_table: *mut *mut mapblock_T,
    keys: &[u8],
    args: &MapArguments,
    noremap: c_int,
    mode: c_int,
    is_abbr: bool,
    sid: scid_T,
    lnum: linenr_T,
    simplified: bool,
) -> *mut mapblock_T {
    // The buffer's tables are reached through the one raw pointer, not
    // through `Buf`'s `DerefMut`: `map_table` already points into
    // `b_maphash`, and a fresh `&mut buf_T` would invalidate it.
    let buf = buf.raw();
    // A given `sid` is upstream's "the block was `xcalloc`ed and only these
    // two fields were filled in", not a tweak of `current_sctx`.
    let script_ctx = if sid != 0 {
        sctx_T {
            sc_sid: sid,
            sc_lnum: lnum,
            ..sctx_T::NONE
        }
    } else {
        let mut ctx = current_sctx.get();
        ctx.sc_lnum += sourcing_lnum();
        // SAFETY: `ctx` is this frame's own, and `nlua_set_sctx` only rewrites
        // the script id in place.
        unsafe { nlua_set_sctx(&raw mut ctx) };
        ctx
    };
    let mut mp = Box::new(mapblock_T {
        m_next: ptr::null_mut(),
        m_alt: ptr::null_mut(),
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
    });

    // If CTRL-C has been mapped, don't always use it for Interrupting.
    if keys.first().copied().map(c_int::from) == Some(Ctrl_C) {
        // SAFETY: `Buf`'s promise -- a live buffer's own field address; no
        // read happens.
        if map_table == unsafe { &raw mut (*buf).b_maphash }.cast() {
            // SAFETY: as above.
            unsafe { (*buf).b_mapped_ctrl_c |= mode };
        } else {
            mapped_ctrl_c.set(mapped_ctrl_c.get() | mode);
        }
    }

    // Add the new entry in front of the abbrlist or of its maphash bucket.
    // SAFETY: the caller's promise -- both tables name live storage, and
    // `map_table` has `MAX_MAPHASH` entries.
    unsafe {
        let head = if is_abbr {
            abbr_table
        } else {
            let first = keys.first().copied().unwrap_or(0);
            map_table.add(map_hash(mode, c_int::from(first)))
        };
        mp.m_next = *head;
        let raw = Box::into_raw(mp);
        *head = raw;
        raw
    }
}

/// Clear all mappings (or abbreviations) in `mode`.
///
/// An entry only loses the bits `mode` names; it is deleted when that leaves
/// it with no mode at all, and re-hashed when the bits that are left move it
/// to another bucket.
///
/// # Safety
/// The lists this walks must not be reached from anywhere else while it
/// runs.
pub unsafe fn map_clear_mode(buf: Buf, mode: c_int, local: bool, abbr: bool) {
    // As in [`map_add`]: `mpp` points into `b_maphash`, so the tables are
    // reached through the one raw pointer rather than through `DerefMut`.
    let buf = buf.raw();
    // SAFETY: `Buf`'s promise — a live buffer.  `&raw` reads nothing, and both
    // addresses come off the one raw pointer rather than off a `&mut`.
    let local_abbr = unsafe { &raw mut (*buf).b_first_abbr };
    // SAFETY: as above.
    let local_maps = unsafe { &raw mut (*buf).b_maphash }.cast::<*mut mapblock_T>();
    // Through raw pointers, not `with_mut`: `mpp` may itself point into one of
    // these two tables, and a `&mut` to the whole array would invalidate it.
    let (abbr_head, map_heads) = if local {
        (local_abbr, local_maps)
    } else {
        (global_abbr_head(), global_map_heads())
    };
    for hash in 0..if abbr { 1 } else { MAX_MAPHASH } {
        // SAFETY: `hash` is below `MAX_MAPHASH`, the length of both tables,
        // and the caller has promised nothing else reaches these lists.
        let mut at = unsafe { Cursor::at(if abbr { abbr_head } else { map_heads.add(hash) }) };
        while let Some(mut mp) = at.entry() {
            let m_mode = mp.m_mode;
            if m_mode & mode != 0 {
                let left = m_mode & !mode;
                mp.m_mode = left;
                if left == 0 {
                    at.remove();
                    continue;
                }
                // May need to put this entry into another hash list.
                let first = mp.keys().first().copied().unwrap_or(0);
                let new_hash = map_hash(left, c_int::from(first));
                if !abbr && new_hash != hash {
                    // SAFETY: `new_hash` is below `MAX_MAPHASH`.
                    unsafe { at.relink_to(map_heads.add(new_hash)) };
                    continue;
                }
            }
            at.advance();
        }
    }
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

/// Whether any mapping in the modes `modechars` names has `str` in its RHS.
///
/// Termcap codes are recognised in `str`.  Buffer-local mappings count.
///
/// # Safety
/// Both strings must be live and NUL-terminated.
pub(crate) unsafe fn map_to_exists(
    str: *const c_char,
    modechars: *const c_char,
    abbr: bool,
) -> bool {
    let mut buf: *mut c_char = ptr::null_mut();
    let out = &raw mut buf;
    let cpo = p_cpo.get();
    let dolt = REPTERM_DO_LT as c_int;
    let simplify = ptr::null_mut();
    // SAFETY: the caller's promise — `str` is live and NUL-terminated.  The
    // allocation `replace_termcodes` may leave in `buf` is the guard's.
    let (rhs, _owned) = unsafe {
        let len = cstr::bytes_at(str).len();
        let rhs = replace_termcodes(str, len, out, 0, dolt, simplify, cpo);
        (rhs, COwned::new(buf))
    };

    let mut mode = 0;
    for (ch, flags) in MODE_CHARS {
        // SAFETY: the caller's promise — `modechars` is NUL-terminated.
        if !unsafe { strchr(modechars, c_int::from(ch)) }.is_null() {
            mode |= flags;
        }
    }

    // SAFETY: `rhs` is `replace_termcodes`'s NUL-terminated answer, and `buf`
    // its allocation, which the guard releases.
    unsafe { map_to_exists_mode(rhs, mode, abbr) }
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
///
/// # Safety
/// `rhs` must be live and NUL-terminated, and `curbuf` a live buffer.
pub unsafe fn map_to_exists_mode(rhs: *const c_char, mode: c_int, abbr: bool) -> bool {
    // SAFETY: the caller's promise — `curbuf` is a live buffer.
    let cur = unsafe { Buf::current() };
    // Do it twice: once for global maps and once for local maps.
    // SAFETY: the caller's promise — `rhs` is live and NUL-terminated.
    let needle = unsafe { cstr::bytes_at(rhs) };
    for table in [MapTable::Global, MapTable::Buffer(cur)] {
        let visit = |mp: Mb| {
            let hit = mp.m_mode & mode != 0 && contains(mp.rhs(), needle);
            hit.then_some(())
        };
        // SAFETY: the tables are live and `visit` only reads.
        if unsafe { map_walk(table, abbr, visit) }.is_some() {
            return true;
        }
    }
    false
}

/// What [`check_map`] found.
pub(crate) struct MapMatch {
    /// The matching mapblock.
    pub mp: *mut mapblock_T,
    /// Whether it came from the buffer-local table.
    pub local: bool,
}

/// Check `keys` against the LHS of every mapping in `mode`.
///
/// Buffer-local mappings are searched first.  Without `exact` a mapping whose
/// LHS is merely a *prefix* of `keys` — or the other way round — counts, which
/// is what `mapcheck()` asks for.  `ign_mod` skips a leading modifier escape
/// in the stored LHS before comparing.
///
/// # Safety
/// `keys` must be live and NUL-terminated, and `curbuf` a live buffer.
pub(crate) unsafe fn check_map(
    keys: *const c_char,
    mode: c_int,
    exact: bool,
    ign_mod: bool,
    abbr: bool,
) -> Option<MapMatch> {
    // SAFETY: the caller's promise — `keys` is NUL-terminated, and `curbuf` a
    // live buffer.
    let keys = unsafe { cstr::bytes_at(keys) };
    // SAFETY: as above.
    let cur = unsafe { Buf::current() };
    let visit = |local: bool| {
        move |mp: Mb| {
            // Skip entries with the wrong mode, the wrong length, and the
            // ones that do not match.
            if mp.m_mode & mode == 0 || (exact && mp.m_keys.len() != keys.len()) {
                return None;
            }
            let mut lhs = mp.keys();
            // A three-byte modifier escape at the front of the stored LHS is
            // skipped when the caller asked for it.
            let modifier = matches!(lhs, [a, b, _, ..]
                if c_int::from(*a) == K_SPECIAL && c_int::from(*b) == KS_MODIFIER);
            if ign_mod && modifier {
                lhs = &lhs[3..];
            }
            let minlen = lhs.len().min(keys.len());
            (lhs[..minlen] == keys[..minlen]).then(|| MapMatch {
                mp: mp.raw(),
                local,
            })
        }
    };
    for (local, table) in [(true, MapTable::Buffer(cur)), (false, MapTable::Global)] {
        // SAFETY: the tables are live and the visitor only reads.
        let found = unsafe { map_walk(table, abbr, visit(local)) };
        if found.is_some() {
            return found;
        }
    }
    None
}
