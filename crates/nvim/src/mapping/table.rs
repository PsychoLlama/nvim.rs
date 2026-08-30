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
//! functions that *delete* while walking keep their own loop, because they
//! need the address of the previous entry's `m_next` and an iterator cannot
//! hand that out.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::ex_docmd::sourcing_lnum;
use crate::keycodes::Ctrl_C;
use crate::winlayer::Buf;
use core::ffi::{c_char, c_int};
use core::ptr;

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
/// A simplified mapping and its unsimplified twin share one RHS, held by
/// whichever of the pair is freed second: `m_alt` non-null means the twin is
/// still alive and owns the strings.
///
/// # Safety
/// `mpp` must point at a non-null entry of a live list.
pub(crate) unsafe fn mapblock_free(mpp: *mut *mut mapblock_T) {
    // SAFETY: the caller's promise — `mpp` names a live link, and the entry it
    // holds is live until this function frees it.  `mpp` itself is the
    // *previous* entry's field, so writing through `mp` cannot disturb it.
    let mut mp = unsafe { Mb::new(*mpp) };
    // SAFETY: `m_keys` is this entry's own `xstrdup`, freed once.
    unsafe { xfree(mp.m_keys.cast()) };
    let alt = mp.m_alt;
    if !alt.is_null() {
        // SAFETY: a non-null `m_alt` is the live twin that shares this RHS;
        // clearing its back-link leaves it owning the strings.
        unsafe { (*alt).m_alt = ptr::null_mut() };
    } else {
        if mp.m_luaref != LUA_NOREF {
            // SAFETY: a reference this entry owns, released once.
            unsafe { api_free_luaref(mp.m_luaref) };
            mp.m_luaref = LUA_NOREF;
        }
        let (str, orig, desc) = (mp.m_str, mp.m_orig_str, mp.m_desc);
        // SAFETY: the three strings this entry owns once its twin is gone.
        unsafe {
            xfree(str.cast());
            xfree(orig.cast());
            xfree(desc.cast());
        }
    }
    let next = mp.m_next;
    // SAFETY: `mpp` is the live link that held `mp`; unlink, then free.
    unsafe {
        *mpp = next;
        xfree(mp.raw().cast());
    }
}

/// Put a new mapping at the front of its list and return it.
///
/// `args` supplies `rhs`, `rhs_lua`, `orig_rhs`, `expr`, `silent`, `nowait`,
/// `replace_keycodes` and `desc`; the three string fields are *taken*, not
/// copied, so the caller must not free them afterwards.  `sid` of 0 means
/// "use `current_sctx`".
///
/// # Safety
/// Every pointer argument must be live, and `keys` NUL-terminated.
#[allow(clippy::too_many_arguments)] // upstream's; the caller has no struct to pass
pub(crate) unsafe fn map_add(
    buf: Buf,
    map_table: *mut *mut mapblock_T,
    abbr_table: *mut *mut mapblock_T,
    keys: *const c_char,
    args: *mut MapArguments,
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
    // SAFETY: `xcalloc` never returns null, and the zeroed block is a valid
    // `mapblock_T` that this function goes on to fill in and link.
    let mut mp = unsafe { Mb::new(xcalloc(1, size_of::<mapblock_T>()).cast()) };

    // If CTRL-C has been mapped, don't always use it for Interrupting.
    // SAFETY: `keys` is the caller's NUL-terminated string, and `buf` a live
    // buffer.
    if unsafe { c_int::from(*keys) } == Ctrl_C {
        // SAFETY: a live buffer's own field address; no read happens.
        if map_table == unsafe { &raw mut (*buf).b_maphash }.cast() {
            // SAFETY: as above.
            unsafe { (*buf).b_mapped_ctrl_c |= mode };
        } else {
            mapped_ctrl_c.set(mapped_ctrl_c.get() | mode);
        }
    }

    // SAFETY: `keys` is NUL-terminated and `args` a live `MapArguments`; the
    // three string fields move into the new entry, which now owns them.
    let args = unsafe { Live::new(args) };
    // SAFETY: as above.
    mp.m_keys = unsafe { xstrdup(keys) };
    mp.m_str = args.rhs;
    mp.m_orig_str = args.orig_rhs;
    mp.m_luaref = args.rhs_lua;
    // SAFETY: the `xstrdup` just above.
    mp.m_keylen = unsafe { strlen(mp.m_keys) } as c_int;
    mp.m_noremap = noremap;
    mp.m_nowait = args.nowait as c_char;
    mp.m_silent = args.silent as c_char;
    mp.m_mode = mode;
    mp.m_simplified = c_int::from(simplified);
    mp.m_expr = args.expr as c_char;
    mp.m_replace_keycodes = args.replace_keycodes;
    if sid != 0 {
        mp.m_script_ctx.sc_sid = sid;
        mp.m_script_ctx.sc_lnum = lnum;
    } else {
        mp.m_script_ctx = current_sctx.get();
        mp.m_script_ctx.sc_lnum += sourcing_lnum();
        // Off `raw()`, not off a `Deref`: the address has to outlive the
        // borrow that produced it.
        let sctx = mp.field_ptr(core::mem::offset_of!(mapblock_T, m_script_ctx));
        // SAFETY: the entry's own field, and `mp` is live.
        unsafe { nlua_set_sctx(sctx) };
    }
    mp.m_desc = args.desc;

    // Add the new entry in front of the abbrlist or of its maphash bucket.
    if is_abbr {
        // SAFETY: the caller's promise — `abbr_table` names a live link.
        mp.m_next = unsafe { *abbr_table };
        unsafe { *abbr_table = mp.raw() };
    } else {
        // SAFETY: `m_keys` is the NUL-terminated copy made above, so its first
        // byte is readable, and `map_table` has `MAX_MAPHASH` entries.
        let n = map_hash(mp.m_mode, c_int::from(unsafe { *mp.m_keys } as u8));
        unsafe {
            mp.m_next = *map_table.add(n);
            *map_table.add(n) = mp.raw();
        }
    }
    mp.raw()
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
        // SAFETY: `hash` is below `MAX_MAPHASH`, the length of both tables.
        let mut mpp = if abbr {
            abbr_head
        } else {
            unsafe { map_heads.add(hash) }
        };
        loop {
            // SAFETY: the caller's promise — nothing else reaches these lists
            // while this runs, so `mpp` and everything it links to are live.
            let mp = unsafe { *mpp };
            if mp.is_null() {
                break;
            }
            // SAFETY: as above.
            let m_mode = unsafe { (*mp).m_mode };
            if m_mode & mode != 0 {
                let left = m_mode & !mode;
                // SAFETY: as above.
                unsafe { (*mp).m_mode = left };
                if left == 0 {
                    // SAFETY: as above; `mpp` holds the entry being deleted.
                    unsafe { mapblock_free(mpp) };
                    continue; // continue with *mpp
                }
                // May need to put this entry into another hash list.
                // SAFETY: `m_keys` is NUL-terminated, so byte 0 is readable.
                let first = unsafe { *(*mp).m_keys } as u8;
                let new_hash = map_hash(left, c_int::from(first));
                if !abbr && new_hash != hash {
                    // SAFETY: as above; `new_hash` is below `MAX_MAPHASH`.
                    unsafe {
                        *mpp = (*mp).m_next;
                        let head = map_heads.add(new_hash);
                        (*mp).m_next = *head;
                        *head = mp;
                    }
                    continue; // continue with *mpp
                }
            }
            // SAFETY: `mp`'s own field; `&raw` reads nothing.
            mpp = unsafe { &raw mut (*mp).m_next };
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
    // allocation `replace_termcodes` may leave in `buf` is freed below.
    let len = unsafe { strlen(str) };
    // SAFETY: as above.
    let rhs = unsafe { replace_termcodes(str, len, out, 0, dolt, simplify, cpo) };

    let mut mode = 0;
    for (ch, flags) in MODE_CHARS {
        // SAFETY: the caller's promise — `modechars` is NUL-terminated.
        if !unsafe { strchr(modechars, c_int::from(ch)) }.is_null() {
            mode |= flags;
        }
    }

    // SAFETY: `rhs` is `replace_termcodes`'s NUL-terminated answer, and `buf`
    // its allocation, freed once.
    let retval = unsafe { map_to_exists_mode(rhs, mode, abbr) };
    unsafe { xfree(buf.cast()) };
    retval
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
    for table in [MapTable::Global, MapTable::Buffer(cur)] {
        let visit = |mp: Mb| {
            // SAFETY: `m_str` and `rhs` are both NUL-terminated.
            let hit = mp.m_mode & mode != 0 && !unsafe { strstr(mp.m_str, rhs) }.is_null();
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
    /// Its RHS, or null when the mapping is a Lua callback.
    pub rhs: *mut c_char,
    /// Its Lua callback, or `LUA_NOREF`.
    pub rhs_lua: LuaRef,
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
    keys: *mut c_char,
    mode: c_int,
    exact: bool,
    ign_mod: bool,
    abbr: bool,
) -> Option<MapMatch> {
    // SAFETY: the caller's promise — `keys` is NUL-terminated, and `curbuf` a
    // live buffer.
    let len = unsafe { strlen(keys) } as c_int;
    // SAFETY: as above.
    let cur = unsafe { Buf::current() };
    let visit = |local: bool| {
        move |mp: Mb| {
            // Skip entries with the wrong mode, the wrong length, and the
            // ones that do not match.
            if mp.m_mode & mode == 0 || (exact && mp.m_keylen != len) {
                return None;
            }
            let mut s = mp.m_keys;
            let mut keylen = mp.m_keylen;
            // SAFETY: `m_keys` is NUL-terminated and `m_keylen` is its length,
            // so the two leading bytes a modifier escape needs are there
            // whenever `keylen >= 3`, and so is the `s.add(3)` tail.
            let modifier = keylen >= 3
                && unsafe {
                    c_int::from(*s as u8) == K_SPECIAL
                        && c_int::from(*s.add(1) as u8) == KS_MODIFIER
                };
            if ign_mod && modifier {
                // SAFETY: as above.
                s = unsafe { s.add(3) };
                keylen -= 3;
            }
            let minlen = keylen.min(len);
            // SAFETY: both strings are NUL-terminated and `minlen` is no
            // longer than either.
            let hit = unsafe { cstr::prefix_eq(s, keys, minlen as size_t) };
            hit.then(|| MapMatch {
                mp: mp.raw(),
                local,
                rhs: if mp.m_luaref == LUA_NOREF {
                    mp.m_str
                } else {
                    ptr::null_mut()
                },
                rhs_lua: mp.m_luaref,
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
