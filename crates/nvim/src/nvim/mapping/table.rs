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

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::ex_docmd::sourcing_lnum;
use crate::src::nvim::keycodes::Ctrl_C;
use core::ffi::{c_char, c_int};
use core::ptr;

/// The global abbreviation list; `b_first_abbr` is its per-buffer twin.
pub(crate) static FIRST_ABBR: GlobalCell<*mut mapblock_T> = GlobalCell::new(ptr::null_mut());

/// The global mapping table; `b_maphash` is its per-buffer twin.
pub(crate) static MAPHASH: GlobalCell<[*mut mapblock_T; MAX_MAPHASH]> =
    GlobalCell::new([ptr::null_mut(); MAX_MAPHASH]);

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
    unsafe { (*curbuf.get()).b_maphash[map_hash(state, c)] }
}

/// Which pair of tables a walk reads: the global one, or a buffer's.
#[derive(Copy, Clone)]
pub(crate) enum MapTable {
    Global,
    Buffer(*mut buf_T),
}

impl MapTable {
    /// The head of the list this table keeps at `hash`, or of its single
    /// abbreviation list.
    ///
    /// # Safety
    /// A `Buffer` must name a live buffer.
    unsafe fn head(self, hash: usize, abbr: bool) -> *mut mapblock_T {
        unsafe {
            match (self, abbr) {
                (MapTable::Global, true) => FIRST_ABBR.get(),
                (MapTable::Global, false) => MAPHASH.with(|table| table[hash]),
                (MapTable::Buffer(buf), true) => (*buf).b_first_abbr,
                (MapTable::Buffer(buf), false) => (*buf).b_maphash[hash],
            }
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
    mut visit: impl FnMut(*mut mapblock_T) -> Option<T>,
) -> Option<T> {
    unsafe {
        for hash in 0..if abbr { 1 } else { MAX_MAPHASH } {
            let mut mp = table.head(hash, abbr);
            while !mp.is_null() {
                if let Some(answer) = visit(mp) {
                    return Some(answer);
                }
                mp = (*mp).m_next;
            }
        }
        None
    }
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
    unsafe {
        let mp = *mpp;
        xfree((*mp).m_keys.cast());
        if !(*mp).m_alt.is_null() {
            (*(*mp).m_alt).m_alt = ptr::null_mut();
        } else {
            if (*mp).m_luaref != LUA_NOREF {
                api_free_luaref((*mp).m_luaref);
                (*mp).m_luaref = LUA_NOREF;
            }
            xfree((*mp).m_str.cast());
            xfree((*mp).m_orig_str.cast());
            xfree((*mp).m_desc.cast());
        }
        *mpp = (*mp).m_next;
        xfree(mp.cast());
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
    buf: *mut buf_T,
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
    unsafe {
        let mp: *mut mapblock_T = xcalloc(1, size_of::<mapblock_T>()).cast();

        // If CTRL-C has been mapped, don't always use it for Interrupting.
        if c_int::from(*keys) == Ctrl_C {
            if map_table == (&raw mut (*buf).b_maphash).cast() {
                (*buf).b_mapped_ctrl_c |= mode;
            } else {
                *mapped_ctrl_c.ptr() |= mode;
            }
        }

        (*mp).m_keys = xstrdup(keys);
        (*mp).m_str = (*args).rhs;
        (*mp).m_orig_str = (*args).orig_rhs;
        (*mp).m_luaref = (*args).rhs_lua;
        (*mp).m_keylen = strlen((*mp).m_keys) as c_int;
        (*mp).m_noremap = noremap;
        (*mp).m_nowait = (*args).nowait as c_char;
        (*mp).m_silent = (*args).silent as c_char;
        (*mp).m_mode = mode;
        (*mp).m_simplified = c_int::from(simplified);
        (*mp).m_expr = (*args).expr as c_char;
        (*mp).m_replace_keycodes = (*args).replace_keycodes;
        if sid != 0 {
            (*mp).m_script_ctx.sc_sid = sid;
            (*mp).m_script_ctx.sc_lnum = lnum;
        } else {
            (*mp).m_script_ctx = current_sctx.get();
            (*mp).m_script_ctx.sc_lnum += sourcing_lnum();
            nlua_set_sctx(&raw mut (*mp).m_script_ctx);
        }
        (*mp).m_desc = (*args).desc;

        // Add the new entry in front of the abbrlist or of its maphash bucket.
        if is_abbr {
            (*mp).m_next = *abbr_table;
            *abbr_table = mp;
        } else {
            let n = map_hash((*mp).m_mode, c_int::from(*(*mp).m_keys as u8));
            (*mp).m_next = *map_table.add(n);
            *map_table.add(n) = mp;
        }
        mp
    }
}

/// Clear all mappings (or abbreviations) in `mode`.
///
/// An entry only loses the bits `mode` names; it is deleted when that leaves
/// it with no mode at all, and re-hashed when the bits that are left move it
/// to another bucket.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn map_clear_mode(buf: *mut buf_T, mode: c_int, local: bool, abbr: bool) {
    unsafe {
        for hash in 0..if abbr { 1 } else { MAX_MAPHASH } {
            let mut mpp: *mut *mut mapblock_T = match (local, abbr) {
                (true, true) => &raw mut (*buf).b_first_abbr,
                (false, true) => FIRST_ABBR.ptr(),
                (true, false) => (&raw mut (*buf).b_maphash)
                    .cast::<*mut mapblock_T>()
                    .add(hash),
                (false, false) => MAPHASH.ptr().cast::<*mut mapblock_T>().add(hash),
            };
            while !(*mpp).is_null() {
                let mp = *mpp;
                if (*mp).m_mode & mode != 0 {
                    (*mp).m_mode &= !mode;
                    if (*mp).m_mode == 0 {
                        mapblock_free(mpp);
                        continue; // continue with *mpp
                    }
                    // May need to put this entry into another hash list.
                    let new_hash = map_hash((*mp).m_mode, c_int::from(*(*mp).m_keys as u8));
                    if !abbr && new_hash != hash {
                        *mpp = (*mp).m_next;
                        // Through raw pointers, not `with_mut`: `mpp` may
                        // itself point into one of these two tables, and a
                        // `&mut` to the whole array would invalidate it.
                        let head: *mut *mut mapblock_T = if local {
                            (&raw mut (*buf).b_maphash).cast()
                        } else {
                            MAPHASH.ptr().cast()
                        };
                        (*mp).m_next = *head.add(new_hash);
                        *head.add(new_hash) = mp;
                        continue; // continue with *mpp
                    }
                }
                mpp = &raw mut (*mp).m_next;
            }
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
    unsafe {
        let mut buf: *mut c_char = ptr::null_mut();
        let rhs = replace_termcodes(
            str,
            strlen(str),
            &raw mut buf,
            0,
            REPTERM_DO_LT as c_int,
            ptr::null_mut(),
            p_cpo.get(),
        );

        let mut mode = 0;
        for (ch, flags) in MODE_CHARS {
            if !strchr(modechars, c_int::from(ch)).is_null() {
                mode |= flags;
            }
        }

        let retval = map_to_exists_mode(rhs, mode, abbr);
        xfree(buf.cast());
        retval
    }
}

/// Whether any mapping in `mode` has `rhs` as a substring of its RHS.
///
/// Global mappings are searched first, then the current buffer's.
///
/// # Safety
/// `rhs` must be live and NUL-terminated, and `curbuf` a live buffer.
pub unsafe fn map_to_exists_mode(rhs: *const c_char, mode: c_int, abbr: bool) -> bool {
    unsafe {
        // Do it twice: once for global maps and once for local maps.
        for table in [MapTable::Global, MapTable::Buffer(curbuf.get())] {
            let found = map_walk(table, abbr, |mp| {
                ((*mp).m_mode & mode != 0 && !strstr((*mp).m_str, rhs).is_null()).then_some(())
            });
            if found.is_some() {
                return true;
            }
        }
        false
    }
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
    unsafe {
        let len = strlen(keys) as c_int;
        for (local, table) in [
            (true, MapTable::Buffer(curbuf.get())),
            (false, MapTable::Global),
        ] {
            let found = map_walk(table, abbr, |mp| {
                // Skip entries with the wrong mode, the wrong length, and the
                // ones that do not match.
                if (*mp).m_mode & mode == 0 || (exact && (*mp).m_keylen != len) {
                    return None;
                }
                let mut s = (*mp).m_keys;
                let mut keylen = (*mp).m_keylen;
                if ign_mod
                    && keylen >= 3
                    && c_int::from(*s as u8) == K_SPECIAL
                    && c_int::from(*s.add(1) as u8) == KS_MODIFIER
                {
                    s = s.add(3);
                    keylen -= 3;
                }
                let minlen = keylen.min(len);
                (strncmp(s, keys, minlen as size_t) == 0).then(|| MapMatch {
                    mp,
                    local,
                    rhs: if (*mp).m_luaref == LUA_NOREF {
                        (*mp).m_str
                    } else {
                        ptr::null_mut()
                    },
                    rhs_lua: (*mp).m_luaref,
                })
            });
            if found.is_some() {
                return found;
            }
        }
        None
    }
}
