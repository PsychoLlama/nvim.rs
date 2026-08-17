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
use core::ffi::{c_char, c_int};
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
    unsafe {
        let mut same = -1; // count of characters of the same type at the start
        let first = c_int::from(vim_iswordp(lhs));
        let mut last = first;
        let mut p = lhs.offset(utfc_ptr2len(lhs) as isize);
        let mut n = 1; // number of (multi-byte) characters
        while p < lhs.offset(len as isize) {
            n += 1;
            last = c_int::from(vim_iswordp(p)); // type of the last character
            if same == -1 && last != first {
                same = n - 1;
            }
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        if last != 0 && n > 2 && same >= 0 && same < n - 1 {
            return false;
        }
        // An abbreviation cannot contain white space.
        for n in 0..len {
            if ascii_iswhite(c_int::from(*lhs.offset(n as isize))) {
                return false;
            }
        }
        true
    }
}

/// Whether a *global* mapping already claims exactly these `len` keys in any
/// of `mode`, which is what makes a new `<unique>` buffer-local one fail.
///
/// # Safety
/// `lhs` must be readable for `len` bytes.
unsafe fn global_map_exists(mode: c_int, lhs: *const c_char, len: c_int, is_abbrev: bool) -> bool {
    unsafe {
        map_walk(MapTable::Global, is_abbrev, |mp| {
            if got_int.get() {
                return Some(false);
            }
            // Check entries with the same mode.
            ((*mp).m_mode & mode != 0
                && (*mp).m_keylen == len
                && strncmp((*mp).m_keys, lhs, len as size_t) == 0)
                .then_some(true)
        })
        .unwrap_or(false)
    }
}

/// List the buffer-local mappings that a *global* listing should also show,
/// and answer whether any were printed.
///
/// Without `has_lhs` every entry in `mode` is shown; with it, every entry
/// whose LHS and `lhs` agree as far as the shorter of the two.
///
/// # Safety
/// `buf` must be live and `lhs` readable for `len` bytes.
unsafe fn show_buffer_local(
    buf: *mut buf_T,
    mode: c_int,
    lhs: *const c_char,
    len: c_int,
    has_lhs: bool,
    is_abbrev: bool,
) -> bool {
    unsafe {
        let mut did_local = false;
        map_walk::<()>(MapTable::Buffer(buf), is_abbrev, |mp| {
            if got_int.get() {
                return Some(()); // 'q' typed at the MORE prompt
            }
            if (*mp).m_simplified == 0 && (*mp).m_mode & mode != 0 {
                let show = !has_lhs || {
                    let n = (*mp).m_keylen;
                    strncmp((*mp).m_keys, lhs, n.min(len) as size_t) == 0
                };
                if show {
                    showmap(mp, true);
                    did_local = true;
                }
            }
            None
        });
        did_local
    }
}

/// Give `mp` the right-hand side and flags in `args`, reusing the block a
/// `:map` of an existing LHS would otherwise have to allocate.
///
/// # Safety
/// `mp` must be a live mapblock whose mode bits are already cleared, and
/// `args` a live [`MapArguments`] whose three owning fields this takes.
unsafe fn reuse_mapblock(
    mp: *mut mapblock_T,
    args: *mut MapArguments,
    noremap: c_int,
    mode: c_int,
    simplified: bool,
) {
    unsafe {
        if !(*mp).m_alt.is_null() {
            (*(*mp).m_alt).m_alt = ptr::null_mut();
            (*mp).m_alt = ptr::null_mut();
        } else {
            if (*mp).m_luaref != LUA_NOREF {
                api_free_luaref((*mp).m_luaref);
                (*mp).m_luaref = LUA_NOREF;
            }
            xfree((*mp).m_str.cast());
            xfree((*mp).m_orig_str.cast());
            xfree((*mp).m_desc.cast());
        }
        (*mp).m_str = (*args).rhs;
        (*mp).m_orig_str = (*args).orig_rhs;
        (*mp).m_luaref = (*args).rhs_lua;
        (*mp).m_noremap = noremap;
        (*mp).m_nowait = (*args).nowait as c_char;
        (*mp).m_silent = (*args).silent as c_char;
        (*mp).m_mode = mode;
        (*mp).m_simplified = c_int::from(simplified);
        (*mp).m_expr = (*args).expr as c_char;
        (*mp).m_replace_keycodes = (*args).replace_keycodes;
        (*mp).m_script_ctx = current_sctx.get();
        (*mp).m_script_ctx.sc_lnum += sourcing_lnum();
        nlua_set_sctx(&raw mut (*mp).m_script_ctx);
        (*mp).m_desc = (*args).desc;
    }
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
/// `args` and `buf` must be live.
pub(crate) unsafe fn buf_do_map(
    mut maptype: c_int,
    args: *mut MapArguments,
    mode: c_int,
    is_abbrev: bool,
    buf: *mut buf_T,
) -> c_int {
    unsafe {
        let mut retval = 0;

        // If <buffer> was given we search the buffer's mappings, not the
        // global ones.
        let buf_table: *mut *mut mapblock_T = (&raw mut (*buf).b_maphash).cast();
        let map_table: *mut *mut mapblock_T = if (*args).buffer {
            buf_table
        } else {
            MAPHASH.ptr().cast()
        };
        let abbr_table: *mut *mut mapblock_T = if (*args).buffer {
            &raw mut (*buf).b_first_abbr
        } else {
            FIRST_ABBR.ptr()
        };
        let mut mp_result: [*mut mapblock_T; 2] = [ptr::null_mut(); 2];

        let unmap_lhs_only = maptype == MAPTYPE_UNMAP_LHS as c_int;
        if unmap_lhs_only {
            maptype = MAPTYPE_UNMAP as c_int;
        }
        let is_unmap = maptype == MAPTYPE_UNMAP as c_int;

        // For ":noremap" don't remap, otherwise do remap.
        let noremap = if (*args).script {
            REMAP_SCRIPT
        } else if maptype == MAPTYPE_NOREMAP as c_int {
            REMAP_NONE
        } else {
            REMAP_YES
        };

        let has_lhs = c_int::from((*args).lhs[0]) != NUL;
        let has_rhs =
            (*args).rhs_lua != LUA_NOREF || c_int::from(*(*args).rhs) != NUL || (*args).rhs_is_noop;
        let do_print = !has_lhs || (!is_unmap && !has_rhs);
        if do_print {
            msg_ext_set_kind(c"list_cmd".as_ptr());
        }

        'theend: {
            // Check for :unmap without argument.
            if is_unmap && !has_lhs {
                retval = 1;
                break 'theend;
            }

            let mut lhs: *const c_char = (&raw const (*args).lhs).cast();
            let did_simplify = (*args).alt_lhs_len != 0;

            // The following is done twice if we have two versions of the keys.
            for keyround in 1..=2 {
                let mut did_it = false;
                let mut did_local = false;
                let keyround1_simplified = keyround == 1 && did_simplify;
                let mut len = (*args).lhs_len as c_int;

                if keyround == 2 {
                    if !did_simplify {
                        break;
                    }
                    lhs = (&raw const (*args).alt_lhs).cast();
                    len = (*args).alt_lhs_len as c_int;
                } else if did_simplify && do_print {
                    // When printing always use the not-simplified map.
                    lhs = (&raw const (*args).alt_lhs).cast();
                    len = (*args).alt_lhs_len as c_int;
                }

                // Check arguments and translate function keys.
                if has_lhs {
                    if len > MAXMAPLEN as c_int {
                        retval = 1;
                        break 'theend;
                    }
                    if is_abbrev && !is_unmap && !abbrev_lhs_ok(lhs, len) {
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
                    msg_start();
                }

                // Check that a new local mapping was not already defined
                // globally.
                if (*args).unique
                    && map_table == buf_table
                    && has_lhs
                    && has_rhs
                    && !is_unmap
                    && global_map_exists(mode, lhs, len, is_abbrev)
                {
                    retval = 6;
                    break 'theend;
                }

                // When listing global mappings, also list buffer-local ones.
                if map_table != buf_table && !has_rhs && !is_unmap {
                    did_local = show_buffer_local(buf, mode, lhs, len, has_lhs, is_abbrev);
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
                            map_hash(mode, c_int::from(*lhs as u8))
                        };
                        (start, start + 1)
                    } else {
                        (0, MAX_MAPHASH)
                    };

                    let mut hash = hash_start;
                    while hash < hash_end && !got_int.get() {
                        let mut mpp: *mut *mut mapblock_T = if is_abbrev {
                            abbr_table
                        } else {
                            map_table.add(hash)
                        };
                        let mut mp = *mpp;
                        // Upstream's two `break`s leave *this* loop and
                        // resume at the next hash bucket, not at the next
                        // round.
                        'entries: while !mp.is_null() && !got_int.get() {
                            // Whether to step `mpp` past this entry before
                            // reading the next one: upstream's bare
                            // `continue` resumes at `*mpp` instead, because
                            // the entry it was pointing at is gone.
                            let mut advance = true;
                            'entry: {
                                if (*mp).m_mode & mode == 0 {
                                    break 'entry; // skip the wrong mode
                                }
                                if !has_lhs {
                                    // Show all entries.
                                    if (*mp).m_simplified == 0 {
                                        showmap(mp, map_table != MAPHASH.ptr().cast());
                                        did_it = true;
                                    }
                                    break 'entry;
                                }

                                // Do we have a match? On the second round,
                                // try to unmap the "rhs" string.
                                let (n, p) = if round != 0 {
                                    (strlen((*mp).m_str) as c_int, (*mp).m_str)
                                } else {
                                    ((*mp).m_keylen, (*mp).m_keys)
                                };
                                if strncmp(p, lhs, n.min(len) as size_t) != 0 {
                                    break 'entry;
                                }

                                if is_unmap {
                                    // Delete the entry, but only on a full
                                    // match. For abbreviations we ignore
                                    // trailing space when matching the "lhs",
                                    // since an abbreviation cannot have any.
                                    if n != len
                                        && (!is_abbrev
                                            || round != 0
                                            || n > len
                                            || c_int::from(*skipwhite(lhs.add(n as usize))) != NUL)
                                    {
                                        break 'entry;
                                    }
                                    // In the keyround for simplified keys,
                                    // don't unmap a mapping without the
                                    // m_simplified flag.
                                    if keyround1_simplified && (*mp).m_simplified == 0 {
                                        break 'entries;
                                    }
                                    // Reset the indicated mode bits; if
                                    // nothing is left the entry is deleted
                                    // below.
                                    (*mp).m_mode &= !mode;
                                    did_it = true;
                                } else if !has_rhs {
                                    // Show the matching entry.
                                    if (*mp).m_simplified == 0 {
                                        showmap(mp, map_table != MAPHASH.ptr().cast());
                                        did_it = true;
                                    }
                                } else if n != len {
                                    break 'entry; // the new entry is ambiguous
                                } else if keyround1_simplified && (*mp).m_simplified == 0 {
                                    // In the keyround for simplified keys,
                                    // don't replace a mapping without the
                                    // m_simplified flag.
                                    did_it = true;
                                    break 'entries;
                                } else if (*args).unique {
                                    retval = 5;
                                    break 'theend;
                                } else {
                                    // A new rhs for an existing entry.
                                    (*mp).m_mode &= !mode; // remove mode bits
                                    if (*mp).m_mode == 0 && !did_it {
                                        reuse_mapblock(
                                            mp,
                                            args,
                                            noremap,
                                            mode,
                                            keyround1_simplified,
                                        );
                                        mp_result[keyround - 1] = mp;
                                        did_it = true;
                                    }
                                }

                                if (*mp).m_mode == 0 {
                                    mapblock_free(mpp); // the entry can go
                                    advance = false;
                                    break 'entry;
                                }

                                // May need to put this entry into another
                                // hash list.
                                let new_hash =
                                    map_hash((*mp).m_mode, c_int::from(*(*mp).m_keys as u8));
                                if !is_abbrev && new_hash != hash {
                                    *mpp = (*mp).m_next;
                                    (*mp).m_next = *map_table.add(new_hash);
                                    *map_table.add(new_hash) = mp;
                                    advance = false;
                                }
                            }
                            if advance {
                                mpp = &raw mut (*mp).m_next;
                            }
                            mp = *mpp;
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
                    } else if c_int::from(*lhs) == Ctrl_C {
                        // CTRL-C has been unmapped, reuse it for Interrupting.
                        if map_table == buf_table {
                            (*buf).b_mapped_ctrl_c &= !mode;
                        } else {
                            *mapped_ctrl_c.ptr() &= !mode;
                        }
                    }
                    continue;
                }

                if !has_lhs || !has_rhs {
                    // Print entries.
                    if !did_it && !did_local {
                        msg(
                            gettext(if is_abbrev {
                                c"No abbreviation found".as_ptr()
                            } else {
                                c"No mapping found".as_ptr()
                            }),
                            0,
                        );
                    }
                    break 'theend; // listing finished
                }

                if did_it {
                    continue; // the new entry has been added already
                }

                // Get here when adding a new entry to the maphash list or the
                // abbrlist.
                mp_result[keyround - 1] = map_add(
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
                );
            }

            if !mp_result[0].is_null() && !mp_result[1].is_null() {
                (*mp_result[0]).m_alt = mp_result[1];
                (*mp_result[1]).m_alt = mp_result[0];
            }
        }

        // Whatever was stored in a mapblock is now owned by it.
        if !mp_result[0].is_null() || !mp_result[1].is_null() {
            (*args).rhs = ptr::null_mut();
            (*args).orig_rhs = ptr::null_mut();
            (*args).rhs_lua = LUA_NOREF;
            (*args).desc = ptr::null_mut();
        }
        retval
    }
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
    unsafe {
        let mut parsed_args: MapArguments = core::mem::zeroed();
        let mut result =
            str_to_mapargs(arg, maptype == MAPTYPE_UNMAP as c_int, &raw mut parsed_args);
        if result == 0 {
            result = buf_do_map(maptype, &raw mut parsed_args, mode, is_abbrev, curbuf.get());
        }
        xfree(parsed_args.rhs.cast());
        xfree(parsed_args.orig_rhs.cast());
        result
    }
}

/// Clear all mappings (`:mapclear`) or abbreviations (`:abclear`).
///
/// # Safety
/// `cmdp` and `arg` must be live, NUL-terminated strings.
unsafe fn do_mapclear(mut cmdp: *mut c_char, arg: *mut c_char, forceit: bool, abbr: bool) {
    unsafe {
        let local = strcmp(arg, c"<buffer>".as_ptr()) == 0;
        if !local && c_int::from(*arg) != NUL {
            emsg(gettext((&raw const e_invarg).cast()));
            return;
        }
        let mode = get_map_mode(&raw mut cmdp, forceit);
        map_clear_mode(curbuf.get(), mode, local, abbr);
    }
}

/// Add a mapping, copying both strings so that read-only ones can be used.
///
/// # Safety
/// `lhs` and `rhs` must be live, NUL-terminated strings.
pub unsafe fn add_map(lhs: *mut c_char, rhs: *mut c_char, mode: c_int, buffer: bool) {
    unsafe {
        let mut args = MAP_ARGUMENTS_INIT;
        set_maparg_lhs_rhs(
            lhs,
            strlen(lhs),
            rhs,
            strlen(rhs),
            LUA_NOREF,
            p_cpo.get(),
            &raw mut args,
        );
        args.buffer = buffer;

        buf_do_map(
            MAPTYPE_NOREMAP as c_int,
            &raw mut args,
            mode,
            false,
            curbuf.get(),
        );
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
    unsafe {
        let mut cmdp = (*eap).cmd;
        let mode = get_map_mode(&raw mut cmdp, (*eap).forceit != 0 || isabbrev);

        let maptype = match *cmdp as u8 {
            b'n' => MAPTYPE_NOREMAP as c_int,
            b'u' => MAPTYPE_UNMAP as c_int,
            _ => MAPTYPE_MAP as c_int,
        };
        let mut parsed_args: MapArguments = core::mem::zeroed();
        if str_to_mapargs(
            (*eap).arg,
            maptype == MAPTYPE_UNMAP as c_int,
            &raw mut parsed_args,
        ) != 0
        {
            emsg(gettext((&raw const e_invarg).cast())); // invalid arguments
        } else {
            let lhs = (&raw mut parsed_args.lhs).cast::<c_char>();
            match buf_do_map(maptype, &raw mut parsed_args, mode, isabbrev, curbuf.get()) {
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
        xfree(parsed_args.rhs.cast());
        xfree(parsed_args.orig_rhs.cast());
    }
}

/// `:abbreviate` and friends.
///
/// # Safety
/// `eap` must be a live `exarg_T`.
pub unsafe fn ex_abbreviate(eap: *mut exarg_T) {
    unsafe { do_exmap(eap, true) } // almost the same as mapping
}

/// `:map` and friends.
///
/// # Safety
/// `eap` must be a live `exarg_T`.
pub unsafe fn ex_map(eap: *mut exarg_T) {
    unsafe {
        // In a secure mode we print the mappings, for security reasons.
        if secure.get() != 0 {
            secure.set(2);
            msg_outtrans((*eap).cmd, 0, false);
            msg_putchar(c_int::from(b'\n'));
        }
        do_exmap(eap, false);
    }
}

/// `:unmap` and friends.
///
/// # Safety
/// `eap` must be a live `exarg_T`.
pub unsafe fn ex_unmap(eap: *mut exarg_T) {
    unsafe { do_exmap(eap, false) }
}

/// `:mapclear` and friends.
///
/// # Safety
/// `eap` must be a live `exarg_T`.
pub unsafe fn ex_mapclear(eap: *mut exarg_T) {
    unsafe { do_mapclear((*eap).cmd, (*eap).arg, (*eap).forceit != 0, false) }
}

/// `:abclear` and friends.
///
/// # Safety
/// `eap` must be a live `exarg_T`.
pub unsafe fn ex_abclear(eap: *mut exarg_T) {
    unsafe { do_mapclear((*eap).cmd, (*eap).arg, true, true) }
}
