//! `:map` and friends: the command layer over the table.
//!
//! [`buf_do_map`] does all of it — list, add, replace and delete — for one
//! already-parsed [`MapArguments`]; [`do_map`] and [`do_exmap`] are the
//! parsing wrappers, and the `ex_*` entry points are what the command table
//! dispatches to.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::ex_docmd::sourcing_lnum;
use crate::keycodes::Ctrl_C;
use crate::semsg_c;
use crate::types::NUL;
use crate::winlayer::{Buf, Ea};
use core::ffi::{c_char, c_int};
use core::mem::offset_of;
use core::ptr;

/// Whether an abbreviation's LHS is one vi can find the start of.
///
/// If it ends in a keyword character, everything before it must be all
/// keyword characters or all non-keyword ones -- `#i` for `#include` is the
/// point of the rule -- and it may never contain white space.
///
/// # Safety
/// `lhs` must be readable for `len` bytes.
unsafe fn abbrev_lhs_ok(lhs: *const c_char, len: c_int) -> bool {
    let mut same = -1; // count of characters of the same type at the start
    // SAFETY: the caller's promise — `lhs` is readable for `len` bytes, which
    // is where both walks below stop.
    let first = c_int::from(unsafe { vim_iswordp(lhs) });
    let mut last = first;
    // SAFETY: as above.
    let (end, mut p) = unsafe {
        (
            lhs.offset(len as isize),
            lhs.offset(utfc_ptr2len(lhs) as isize),
        )
    };
    let mut n = 1; // number of (multi-byte) characters
    while p < end {
        n += 1;
        // SAFETY: `p` is still below `end`, so it names a byte of `lhs`.
        last = c_int::from(unsafe { vim_iswordp(p) }); // type of the last character
        if same == -1 && last != first {
            same = n - 1;
        }
        // SAFETY: as above.
        p = unsafe { p.offset(utfc_ptr2len(p) as isize) };
    }
    if last != 0 && n > 2 && same >= 0 && same < n - 1 {
        return false;
    }
    // An abbreviation cannot contain white space.
    for n in 0..len {
        // SAFETY: `n` is below `len`, the caller's promised length.
        if ascii_iswhite(c_int::from(unsafe { *lhs.offset(n as isize) })) {
            return false;
        }
    }
    true
}

/// Whether a *global* mapping already claims exactly these `len` keys in any
/// of `mode`, which is what makes a new `<unique>` buffer-local one fail.
///
/// # Safety
/// `lhs` must be readable for `len` bytes.
unsafe fn global_map_exists(mode: c_int, lhs: *const c_char, len: c_int, is_abbrev: bool) -> bool {
    let clashes = |mp: Mb| {
        if got_int.get() {
            return Some(false);
        }
        // Check entries with the same mode.
        // SAFETY: `m_keys` is NUL-terminated and the caller's promise makes
        // `lhs` readable for `len` bytes, so the comparison stays in both.
        let same = mp.m_keylen == len && unsafe { strncmp(mp.m_keys, lhs, len as size_t) } == 0;
        (mp.m_mode & mode != 0 && same).then_some(true)
    };
    // SAFETY: the global tables are live and `clashes` only reads them.
    unsafe { map_walk(MapTable::Global, is_abbrev, clashes) }.unwrap_or(false)
}

/// List the buffer-local mappings that a *global* listing should also show,
/// and answer whether any were printed.
///
/// Without `has_lhs` every entry in `mode` is shown; with it, every entry
/// whose LHS and `lhs` agree as far as the shorter of the two.
///
/// # Safety
/// `lhs` must be readable for `len` bytes.
unsafe fn show_buffer_local(
    buf: Buf,
    mode: c_int,
    lhs: *const c_char,
    len: c_int,
    has_lhs: bool,
    is_abbrev: bool,
) -> bool {
    let mut did_local = false;
    let list = |mp: Mb| {
        if got_int.get() {
            return Some(()); // 'q' typed at the MORE prompt
        }
        if mp.m_simplified == 0 && mp.m_mode & mode != 0 {
            let show = !has_lhs || {
                let n = mp.m_keylen;
                // SAFETY: `m_keys` is NUL-terminated and `lhs` is readable for
                // `len` bytes by the caller's promise.
                unsafe { strncmp(mp.m_keys, lhs, n.min(len) as size_t) == 0 }
            };
            if show {
                // SAFETY: `mp` is an entry of the buffer's live table.
                unsafe { showmap(mp, true) };
                did_local = true;
            }
        }
        None
    };
    // SAFETY: `Buf`'s promise — a live buffer — and `list` only reads.
    unsafe { map_walk::<()>(MapTable::Buffer(buf), is_abbrev, list) };
    did_local
}

/// Give `mp` the right-hand side and flags in `args`, reusing the block a
/// `:map` of an existing LHS would otherwise have to allocate.
///
/// # Safety
/// `mp` must be a live mapblock whose mode bits are already cleared, and
/// `args` a live [`MapArguments`] whose three owning fields this takes.
unsafe fn reuse_mapblock(
    mp: Mb,
    args: *mut MapArguments,
    noremap: c_int,
    mode: c_int,
    simplified: bool,
) {
    let mut mp = mp;
    // SAFETY: the caller's promise — `args` is a live `MapArguments` whose
    // three owning fields move into `mp` here.
    let args = unsafe { Live::new(args) };
    let alt = mp.m_alt;
    if !alt.is_null() {
        // SAFETY: a non-null `m_alt` is the live twin that shares this RHS.
        unsafe { (*alt).m_alt = ptr::null_mut() };
        mp.m_alt = ptr::null_mut();
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
    mp.m_str = args.rhs;
    mp.m_orig_str = args.orig_rhs;
    mp.m_luaref = args.rhs_lua;
    mp.m_noremap = noremap;
    mp.m_nowait = args.nowait as c_char;
    mp.m_silent = args.silent as c_char;
    mp.m_mode = mode;
    mp.m_simplified = c_int::from(simplified);
    mp.m_expr = args.expr as c_char;
    mp.m_replace_keycodes = args.replace_keycodes;
    mp.m_script_ctx = current_sctx.get();
    mp.m_script_ctx.sc_lnum += sourcing_lnum();
    // Off `raw()`, not off a `Deref`: the address has to outlive the borrow
    // that produced it.
    let sctx = mp.field_ptr(core::mem::offset_of!(mapblock_T, m_script_ctx));
    // SAFETY: the entry's own field, and `mp` is live.
    unsafe { nlua_set_sctx(sctx) };
    mp.m_desc = args.desc;
}

/// Set or remove a mapping or abbreviation in `buf`, or display matching
/// ones.
///
/// `maptype` is one of the `MAPTYPE_*` values and `args` is already parsed
/// and termcode-replaced: whitespace, `<` and `>` in the two halves are
/// literal by the time this sees them.
///
/// Answers 0 on success, or 1 for invalid arguments, 2 for no match, 5 for a
/// `<unique>` clash and 6 for a buffer-local `<unique>` entry clashing with a
/// global one.
///
/// # Safety
/// `args` must be live.
pub(crate) unsafe fn buf_do_map(
    mut maptype: c_int,
    args: *mut MapArguments,
    mode: c_int,
    is_abbrev: bool,
    buf: Buf,
) -> c_int {
    // The buffer's own tables are reached through the one raw pointer, not
    // through `Buf`'s `DerefMut`: `buf_table` points into `b_maphash`, and a
    // fresh `&mut buf_T` taken later would invalidate it.
    let bufp = buf.raw();
    let mut retval = 0;

    // If <buffer> was given we search the buffer's mappings, not the
    // global ones.
    // SAFETY: `Buf`'s promise — a live buffer.  `&raw` reads nothing, and both
    // addresses come off the one raw pointer rather than off a `&mut`.
    let buf_table: *mut *mut mapblock_T = unsafe { &raw mut (*bufp).b_maphash }.cast();
    // SAFETY: as above.
    let buf_abbrs: *mut *mut mapblock_T = unsafe { &raw mut (*bufp).b_first_abbr };
    // SAFETY: the caller's promise — `args` is a live `MapArguments`.
    let mut margs = unsafe { Ma::new(args) };
    let map_table = if margs.buffer {
        buf_table
    } else {
        global_map_heads()
    };
    let abbr_table = if margs.buffer {
        buf_abbrs
    } else {
        global_abbr_head()
    };
    let mut mp_result: [*mut mapblock_T; 2] = [ptr::null_mut(); 2];

    let unmap_lhs_only = maptype == MAPTYPE_UNMAP_LHS as c_int;
    if unmap_lhs_only {
        maptype = MAPTYPE_UNMAP as c_int;
    }
    let is_unmap = maptype == MAPTYPE_UNMAP as c_int;

    // For ":noremap" don't remap, otherwise do remap.
    let noremap = if margs.script {
        REMAP_SCRIPT
    } else if maptype == MAPTYPE_NOREMAP as c_int {
        REMAP_NONE
    } else {
        REMAP_YES
    };

    let has_lhs = c_int::from(margs.lhs[0]) != NUL;
    // SAFETY: `rhs` is either a NUL-terminated allocation the parse made or
    // unread, because `rhs_lua` short-circuits the test.
    let has_rhs = margs.rhs_lua != LUA_NOREF
        || unsafe { c_int::from(*margs.rhs) } != NUL
        || margs.rhs_is_noop;
    let do_print = !has_lhs || (!is_unmap && !has_rhs);
    if do_print {
        // SAFETY: a static NUL-terminated kind name.
        unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    }

    'theend: {
        // Check for :unmap without argument.
        if is_unmap && !has_lhs {
            retval = 1;
            break 'theend;
        }

        // Both LHS buffers are fields of the caller's struct, so their
        // addresses are taken off `raw()` rather than off a `Deref`.
        let plain_lhs: *const c_char = margs.field_ptr(offset_of!(MapArguments, lhs));
        let alt_lhs: *const c_char = margs.field_ptr(offset_of!(MapArguments, alt_lhs));
        let mut lhs = plain_lhs;
        let did_simplify = margs.alt_lhs_len != 0;

        // The following is done twice if we have two versions of the keys.
        for keyround in 1..=2 {
            let mut did_it = false;
            let mut did_local = false;
            let keyround1_simplified = keyround == 1 && did_simplify;
            let mut len = margs.lhs_len as c_int;

            if keyround == 2 {
                if !did_simplify {
                    break;
                }
                lhs = alt_lhs;
                len = margs.alt_lhs_len as c_int;
            } else if did_simplify && do_print {
                // When printing always use the not-simplified map.
                lhs = alt_lhs;
                len = margs.alt_lhs_len as c_int;
            }

            // Check arguments and translate function keys.
            if has_lhs {
                if len > MAXMAPLEN as c_int {
                    retval = 1;
                    break 'theend;
                }
                // SAFETY: `lhs` names one of the struct's own LHS buffers,
                // whose `len` bytes the parse filled in.
                if is_abbrev && !is_unmap && !unsafe { abbrev_lhs_ok(lhs, len) } {
                    retval = 1;
                    break 'theend;
                }
            }

            if has_lhs && has_rhs && is_abbrev {
                // We are adding an abbreviation, so reset the flag that
                // says there are none.
                no_abbr.set(false);
            }

            if do_print {
                // SAFETY: starts a message; reads nothing of ours.
                unsafe { msg_start() };
            }

            // Check that a new local mapping was not already defined
            // globally.
            // SAFETY: as above — `lhs` is `len` readable bytes.
            let clash = margs.unique
                && map_table == buf_table
                && has_lhs
                && has_rhs
                && !is_unmap
                // SAFETY (this body): the caller's promise -- `args` is a live
                // `MapArguments` and `buf` a live buffer.
                && unsafe { global_map_exists(mode, lhs, len, is_abbrev) };
            if clash {
                retval = 6;
                break 'theend;
            }

            // When listing global mappings, also list buffer-local ones.
            if map_table != buf_table && !has_rhs && !is_unmap {
                // SAFETY: as above, and `buf` is live.
                did_local = unsafe { show_buffer_local(buf, mode, lhs, len, has_lhs, is_abbrev) };
            }

            // Find a matching entry. For :unmap we may loop twice: once
            // for an entry with a matching "from" part, and if that fails
            // once for one with a matching "to" part, so that ":ab foo
            // bar" can be undone by ":unab foo" -- where "foo" has itself
            // been replaced by "bar".
            let num_rounds = if is_unmap && !unmap_lhs_only { 2 } else { 1 };
            let mut round = 0;
            while round < num_rounds && !did_it && !got_int.get() {
                let (hash_start, hash_end) = if (round == 0 && has_lhs) || is_abbrev {
                    // Just one hash.
                    let start = if is_abbrev {
                        0
                    } else {
                        // SAFETY: `lhs` is a filled-in LHS buffer, so its
                        // first byte is there.
                        map_hash(mode, c_int::from(unsafe { *lhs } as u8))
                    };
                    (start, start + 1)
                } else {
                    (0, MAX_MAPHASH)
                };

                let mut hash = hash_start;
                while hash < hash_end && !got_int.get() {
                    // SAFETY: `hash` is below `MAX_MAPHASH`, the length of the
                    // mapping table.
                    let mut mpp: *mut *mut mapblock_T = if is_abbrev {
                        abbr_table
                    } else {
                        unsafe { map_table.add(hash) }
                    };
                    // SAFETY: the list heads are live and hold live entries.
                    let mut mp = unsafe { *mpp };
                    // Upstream's two `break`s leave *this* loop and
                    // resume at the next hash bucket, not at the next
                    // round.
                    'entries: while !mp.is_null() && !got_int.get() {
                        // Whether to step `mpp` past this entry before
                        // reading the next one: upstream's bare
                        // `continue` resumes at `*mpp` instead, because
                        // the entry it was pointing at is gone.
                        let mut advance = true;
                        // SAFETY: `mp` is a non-null entry of the live list
                        // `mpp` walks, and stays live until this body frees
                        // it.  Its `m_next` address is taken off the raw
                        // pointer at the tail, never off a `Deref`.
                        let entry = unsafe { Mb::new(mp) };
                        'entry: {
                            if entry.m_mode & mode == 0 {
                                break 'entry; // skip the wrong mode
                            }
                            if !has_lhs {
                                // Show all entries.
                                if entry.m_simplified == 0 {
                                    // SAFETY: `entry` is a live mapblock.
                                    unsafe { showmap(entry, map_table != global_map_heads()) };
                                    did_it = true;
                                }
                                break 'entry;
                            }

                            // Do we have a match? On the second round,
                            // try to unmap the "rhs" string.
                            // SAFETY: `m_str` and `m_keys` are the entry's own
                            // NUL-terminated strings.
                            let (n, p) = if round != 0 {
                                (unsafe { strlen(entry.m_str) } as c_int, entry.m_str)
                            } else {
                                (entry.m_keylen, entry.m_keys)
                            };
                            // SAFETY: as above, and `lhs` is `len` bytes.
                            if unsafe { strncmp(p, lhs, n.min(len) as size_t) } != 0 {
                                break 'entry;
                            }

                            if is_unmap {
                                // Delete the entry, but only on a full
                                // match. For abbreviations we ignore
                                // trailing space when matching the "lhs",
                                // since an abbreviation cannot have any.
                                // SAFETY: `n <= len` guards the step, and the
                                // LHS buffer is NUL-terminated.
                                let trailing = n <= len
                                    && unsafe { c_int::from(*skipwhite(lhs.add(n as usize))) }
                                        == NUL;
                                if n != len && !(is_abbrev && round == 0 && trailing) {
                                    break 'entry;
                                }
                                // In the keyround for simplified keys,
                                // don't unmap a mapping without the
                                // m_simplified flag.
                                if keyround1_simplified && entry.m_simplified == 0 {
                                    break 'entries;
                                }
                                // Reset the indicated mode bits; if
                                // nothing is left the entry is deleted
                                // below.
                                // SAFETY: a live entry of the list.
                                unsafe { (*mp).m_mode &= !mode };
                                did_it = true;
                            } else if !has_rhs {
                                // Show the matching entry.
                                if entry.m_simplified == 0 {
                                    // SAFETY: `entry` is a live mapblock.
                                    unsafe { showmap(entry, map_table != global_map_heads()) };
                                    did_it = true;
                                }
                            } else if n != len {
                                break 'entry; // the new entry is ambiguous
                            } else if keyround1_simplified && entry.m_simplified == 0 {
                                // In the keyround for simplified keys,
                                // don't replace a mapping without the
                                // m_simplified flag.
                                did_it = true;
                                break 'entries;
                            } else if margs.unique {
                                retval = 5;
                                break 'theend;
                            } else {
                                // A new rhs for an existing entry.
                                // SAFETY: a live entry of the list.
                                unsafe { (*mp).m_mode &= !mode }; // remove mode bits
                                if entry.m_mode == 0 && !did_it {
                                    // SAFETY: as above, and `args` is the
                                    // caller's live `MapArguments`.
                                    unsafe {
                                        reuse_mapblock(
                                            entry,
                                            args,
                                            noremap,
                                            mode,
                                            keyround1_simplified,
                                        );
                                    }
                                    mp_result[keyround - 1] = mp;
                                    did_it = true;
                                }
                            }

                            if entry.m_mode == 0 {
                                // SAFETY: `mpp` holds this entry, which is
                                // unlinked and freed here.
                                unsafe { mapblock_free(mpp) }; // the entry can go
                                advance = false;
                                break 'entry;
                            }

                            // May need to put this entry into another
                            // hash list.
                            // SAFETY: `m_keys` is NUL-terminated, so byte 0 is
                            // readable.
                            let first = unsafe { *entry.m_keys } as u8;
                            let new_hash = map_hash(entry.m_mode, c_int::from(first));
                            if !is_abbrev && new_hash != hash {
                                // SAFETY: `new_hash` is below `MAX_MAPHASH`,
                                // and `mpp` is the link holding this entry.
                                unsafe {
                                    *mpp = (*mp).m_next;
                                    let head = map_table.add(new_hash);
                                    (*mp).m_next = *head;
                                    *head = mp;
                                }
                                advance = false;
                            }
                        }
                        // SAFETY: `mp` is still linked when `advance` holds,
                        // and `mpp` always names a live link.
                        unsafe {
                            if advance {
                                mpp = &raw mut (*mp).m_next;
                            }
                            mp = *mpp;
                        }
                    }
                    hash += 1;
                }
                round += 1;
            }

            if is_unmap {
                if !did_it {
                    if !keyround1_simplified {
                        retval = 2; // no match
                    }
                // SAFETY: `lhs` names a filled-in LHS buffer.
                } else if unsafe { c_int::from(*lhs) } == Ctrl_C {
                    // CTRL-C has been unmapped, reuse it for Interrupting.
                    if map_table == buf_table {
                        // SAFETY: `Buf`'s promise — a live buffer.
                        unsafe { (*bufp).b_mapped_ctrl_c &= !mode };
                    } else {
                        mapped_ctrl_c.set(mapped_ctrl_c.get() & !mode);
                    }
                }
                continue;
            }

            if !has_lhs || !has_rhs {
                // Print entries.
                if !did_it && !did_local {
                    let text = if is_abbrev {
                        c"No abbreviation found".as_ptr()
                    } else {
                        c"No mapping found".as_ptr()
                    };
                    // SAFETY: a static NUL-terminated message.
                    unsafe { msg(gettext(text), 0) };
                }
                break 'theend; // listing finished
            }

            if did_it {
                continue; // the new entry has been added already
            }

            // Get here when adding a new entry to the maphash list or the
            // abbrlist.
            // SAFETY: `buf` is live, both tables name live storage, `lhs` is
            // the NUL-terminated LHS buffer and `args` the caller's struct.
            mp_result[keyround - 1] = unsafe {
                map_add(
                    buf,
                    map_table,
                    abbr_table,
                    lhs,
                    args,
                    noremap,
                    mode,
                    is_abbrev,
                    0, // sid
                    0, // lnum
                    keyround1_simplified,
                )
            };
        }

        if !mp_result[0].is_null() && !mp_result[1].is_null() {
            // SAFETY: both are entries `map_add`/`reuse_mapblock` just linked.
            unsafe {
                (*mp_result[0]).m_alt = mp_result[1];
                (*mp_result[1]).m_alt = mp_result[0];
            }
        }
    }

    // Whatever was stored in a mapblock is now owned by it.
    if !mp_result[0].is_null() || !mp_result[1].is_null() {
        margs.rhs = ptr::null_mut();
        margs.orig_rhs = ptr::null_mut();
        margs.rhs_lua = LUA_NOREF;
        margs.desc = ptr::null_mut();
    }
    retval
}

/// Set or remove a mapping or an abbreviation in the current buffer, or
/// display the matching ones.
///
/// ```vim
/// map[!]                          " show all key mappings
/// map[!] {lhs}                    " show key mapping for {lhs}
/// map[!] {lhs} {rhs}              " set key mapping for {lhs} to {rhs}
/// noremap[!] {lhs} {rhs}          " same, but no remapping for {rhs}
/// unmap[!] {lhs}                  " remove key mapping for {lhs}
/// abbr                            " show all abbreviations
/// abbr {lhs}                      " show abbreviations for {lhs}
/// abbr {lhs} {rhs}                " set abbreviation for {lhs} to {rhs}
/// noreabbr {lhs} {rhs}            " same, but no remapping for {rhs}
/// unabbr {lhs}                    " remove abbreviation for {lhs}
/// ```
///
/// `arg` is everything after the initial `:[x][nore]map` and is modified in
/// place.  `mode` is a set of mode bits; see `get_map_mode`.  For the answer,
/// see [`buf_do_map`].
///
/// # Safety
/// `arg` must be a live, writable, NUL-terminated string.
pub unsafe fn do_map(maptype: c_int, arg: *mut c_char, mode: c_int, is_abbrev: bool) -> c_int {
    let mut parsed_args = MAP_ARGUMENTS_INIT;
    let parsed = &raw mut parsed_args;
    let is_unmap = maptype == MAPTYPE_UNMAP as c_int;
    // SAFETY: the caller's promise — `arg` is live and NUL-terminated — and
    // `parsed_args` is this frame's own struct, which outlives both calls.
    let mut result = unsafe { str_to_mapargs(arg, is_unmap, parsed) };
    if result == 0 {
        // SAFETY: as above.
        result = unsafe { buf_do_map(maptype, parsed, mode, is_abbrev, cur_buf()) };
    }
    // SAFETY: whatever a mapblock took ownership of was nulled out by
    // `buf_do_map`; the rest is this frame's to free.
    unsafe {
        xfree(parsed_args.rhs.cast());
        xfree(parsed_args.orig_rhs.cast());
    }
    result
}

/// Clear all mappings (`:mapclear`) or abbreviations (`:abclear`).
///
/// # Safety
/// `cmdp` and `arg` must be live, NUL-terminated strings.
unsafe fn do_mapclear(mut cmdp: *mut c_char, arg: *mut c_char, forceit: bool, abbr: bool) {
    // SAFETY: the caller's promise — `arg` is live and NUL-terminated.
    let local = unsafe { strcmp(arg, c"<buffer>".as_ptr()) } == 0;
    // SAFETY: as above.
    if !local && unsafe { c_int::from(*arg) } != NUL {
        // SAFETY: `e_invarg` is a static NUL-terminated message.
        unsafe { emsg(gettext((&raw const e_invarg).cast())) };
        return;
    }
    // SAFETY: the caller's promise — `cmdp` is a live command name — and
    // `curbuf` is live, so its tables are ours to clear.
    unsafe {
        let mode = get_map_mode(&raw mut cmdp, forceit);
        map_clear_mode(cur_buf(), mode, local, abbr);
    }
}

/// Add a mapping, copying both strings so that read-only ones can be used.
///
/// # Safety
/// `lhs` and `rhs` must be live, NUL-terminated strings.
pub unsafe fn add_map(lhs: *mut c_char, rhs: *mut c_char, mode: c_int, buffer: bool) {
    let mut args = MAP_ARGUMENTS_INIT;
    let parsed = &raw mut args;
    let cpo = p_cpo.get();
    let noremap = MAPTYPE_NOREMAP as c_int;
    // SAFETY: the caller's promise — both strings are live and
    // NUL-terminated — and `args` is this frame's own struct.
    unsafe {
        let (lhs_len, rhs_len) = (strlen(lhs), strlen(rhs));
        set_maparg_lhs_rhs(lhs, lhs_len, rhs, rhs_len, LUA_NOREF, cpo, parsed);
    }
    args.buffer = buffer;

    // SAFETY: as above; `curbuf` is live.
    unsafe { buf_do_map(noremap, parsed, mode, false, cur_buf()) };
    // SAFETY: whatever the mapping took ownership of was nulled out.
    unsafe {
        xfree(args.rhs.cast());
        xfree(args.orig_rhs.cast());
    }
}

/// `:map`, `:abbrev` and every prefixed variant of either, from the command
/// table's point of view: work out the mode and the map type from the command
/// name, then report whatever [`buf_do_map`] answers.
///
/// # Safety
/// `eap` must be a live `exarg_T`.
unsafe fn do_exmap(eap: *mut exarg_T, isabbrev: bool) {
    // SAFETY: the caller's promise — `eap` is a live `exarg_T`.
    let eap = unsafe { Ea::new(eap) };
    let mut cmdp = eap.cmd;
    // SAFETY: `cmd` is the command name the dispatcher matched, so it is live
    // and NUL-terminated.
    let mode = unsafe { get_map_mode(&raw mut cmdp, eap.forceit != 0 || isabbrev) };

    // SAFETY: `get_map_mode` left `cmdp` inside the same name.
    let maptype = match unsafe { *cmdp } as u8 {
        b'n' => MAPTYPE_NOREMAP as c_int,
        b'u' => MAPTYPE_UNMAP as c_int,
        _ => MAPTYPE_MAP as c_int,
    };
    let mut parsed_args = MAP_ARGUMENTS_INIT;
    let parsed = &raw mut parsed_args;
    let is_unmap = maptype == MAPTYPE_UNMAP as c_int;
    // SAFETY: `arg` is the command's own NUL-terminated argument, and
    // `parsed_args` is this frame's struct.
    if unsafe { str_to_mapargs(eap.arg, is_unmap, parsed) } != 0 {
        // SAFETY: a static NUL-terminated message.
        unsafe { emsg(gettext((&raw const e_invarg).cast())) }; // invalid arguments
    } else {
        let lhs = (&raw mut parsed_args.lhs).cast::<c_char>();
        // SAFETY: as above; `curbuf` is live.
        let answer = unsafe { buf_do_map(maptype, parsed, mode, isabbrev, cur_buf()) };
        // SAFETY: every message below is a static format with the
        // NUL-terminated `lhs` buffer as its only argument.
        unsafe {
            match answer {
                1 => {
                    emsg(gettext((&raw const e_invarg).cast()));
                }
                2 => {
                    emsg(gettext(if isabbrev {
                        (&raw const e_noabbr).cast()
                    } else {
                        (&raw const e_nomap).cast()
                    }));
                }
                5 => {
                    semsg_c!(
                        gettext(if isabbrev {
                            E_ABBREVIATION_ALREADY_EXISTS_FOR_STR.as_ptr()
                        } else {
                            E_MAPPING_ALREADY_EXISTS_FOR_STR.as_ptr()
                        }),
                        lhs,
                    );
                }
                6 => {
                    semsg_c!(
                        gettext(if isabbrev {
                            E_GLOBAL_ABBREVIATION_ALREADY_EXISTS_FOR_STR.as_ptr()
                        } else {
                            E_GLOBAL_MAPPING_ALREADY_EXISTS_FOR_STR.as_ptr()
                        }),
                        lhs,
                    );
                }
                _ => {}
            }
        }
    }
    // SAFETY: whatever a mapblock took ownership of was nulled out.
    unsafe {
        xfree(parsed_args.rhs.cast());
        xfree(parsed_args.orig_rhs.cast());
    }
}

/// `:abbreviate` and friends.
///
/// # Safety
/// `eap` must be a live `exarg_T`.
pub unsafe fn ex_abbreviate(eap: *mut exarg_T) {
    // SAFETY (this body): the caller's promise -- `eap` is a live `exarg_T`.
    unsafe { do_exmap(eap, true) } // almost the same as mapping
}

/// `:map` and friends.
///
/// # Safety
/// `eap` must be a live `exarg_T`.
pub unsafe fn ex_map(eap: *mut exarg_T) {
    // In a secure mode we print the mappings, for security reasons.
    if secure.get() != 0 {
        secure.set(2);
        // SAFETY: the caller's promise — `eap` is live, so `cmd` is its own
        // NUL-terminated command name.
        unsafe {
            msg_outtrans((*eap).cmd, 0, false);
            msg_putchar(c_int::from(b'\n'));
        }
    }
    // SAFETY: as above.
    unsafe { do_exmap(eap, false) };
}

/// `:unmap` and friends.
///
/// # Safety
/// `eap` must be a live `exarg_T`.
pub unsafe fn ex_unmap(eap: *mut exarg_T) {
    // SAFETY (this body): as [`ex_abbreviate`].
    unsafe { do_exmap(eap, false) }
}

/// `:mapclear` and friends.
///
/// # Safety
/// `eap` must be a live `exarg_T`.
pub unsafe fn ex_mapclear(eap: *mut exarg_T) {
    // SAFETY: the caller's promise — `eap` is a live `exarg_T`, so `cmd` and
    // `arg` are its own NUL-terminated strings.
    unsafe { do_mapclear((*eap).cmd, (*eap).arg, (*eap).forceit != 0, false) }
}

/// `:abclear` and friends.
///
/// # Safety
/// `eap` must be a live `exarg_T`.
pub unsafe fn ex_abclear(eap: *mut exarg_T) {
    // SAFETY: as [`ex_mapclear`].
    unsafe { do_mapclear((*eap).cmd, (*eap).arg, true, true) }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
