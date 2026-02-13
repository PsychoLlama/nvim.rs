//! Core mapping engine: `buf_do_map` and `do_map`.
//!
//! `buf_do_map` is the central orchestrator for `:map`, `:unmap`, and listing
//! commands. `do_map` is its thin public wrapper that parses arguments first.

use std::ffi::{c_char, c_int};

use crate::args::MapArguments;
use crate::{
    map_hash, mapblock_keylen, mapblock_keys, mapblock_mode, mapblock_next, mapblock_simplified,
    mapblock_str, BufHandle, MapblockHandle, MAXMAPLEN,
};

// =============================================================================
// Constants
// =============================================================================

const LUA_NOREF: c_int = -2;

// Maptype constants (from mapping.h)
const MAPTYPE_MAP: c_int = 0;
const MAPTYPE_UNMAP: c_int = 1;
const MAPTYPE_NOREMAP: c_int = 2;
const MAPTYPE_UNMAP_LHS: c_int = 3;

// Remap constants (from getchar_defs.h)
const REMAP_YES: c_int = 0;
const REMAP_NONE: c_int = -1;
const REMAP_SCRIPT: c_int = -2;

const CTRL_C: u8 = 3;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Existing accessors
    fn nvim_get_maphash_entry(index: c_int) -> MapblockHandle;
    fn nvim_get_first_abbr() -> MapblockHandle;
    fn nvim_buf_get_maphash_entry(buf: BufHandle, index: c_int) -> MapblockHandle;
    fn nvim_buf_get_first_abbr(buf: BufHandle) -> MapblockHandle;
    fn nvim_get_curbuf() -> BufHandle;

    // Phase 6 C accessors
    fn nvim_showmap(mp: MapblockHandle, local: c_int);
    fn nvim_map_add(
        buf: BufHandle,
        is_buf_local: c_int,
        keys: *const c_char,
        args: *mut MapArguments,
        noremap: c_int,
        mode: c_int,
        is_abbr: c_int,
        sid: c_int,
        lnum: c_int,
        simplified: c_int,
    ) -> MapblockHandle;
    fn nvim_mapblock_reuse(
        mp: MapblockHandle,
        args: *mut MapArguments,
        noremap: c_int,
        mode: c_int,
        simplified: c_int,
    );
    fn nvim_mapblock_set_alt(a: MapblockHandle, b: MapblockHandle);
    fn nvim_mapargs_take_ownership(args: *mut MapArguments);

    fn nvim_get_got_int() -> c_int;
    fn nvim_mapping_set_no_abbr(val: c_int);
    fn nvim_get_mapped_ctrl_c() -> c_int;
    fn nvim_set_mapped_ctrl_c(val: c_int);
    fn nvim_mapping_buf_get_mapped_ctrl_c(buf: BufHandle) -> c_int;
    fn nvim_mapping_buf_set_mapped_ctrl_c(buf: BufHandle, val: c_int);

    fn nvim_msg_start();
    fn nvim_mapping_msg_ext_set_kind_list_cmd();
    fn nvim_mapping_msg_no_mapping(is_abbr: c_int);

    fn nvim_vim_iswordp(p: *const c_char) -> c_int;
    fn nvim_mapping_utfc_ptr2len(p: *const c_char) -> c_int;
    fn rs_skipwhite(p: *const c_char) -> *const c_char;

    fn nvim_mapblock_set_mode(mp: MapblockHandle, mode: c_int);
    fn nvim_mapblock_get_str_len(mp: MapblockHandle) -> c_int;

    fn nvim_mapblock_free_in_list(
        buf: BufHandle,
        hash: c_int,
        is_abbr: c_int,
        is_buf_local: c_int,
        mp: MapblockHandle,
    ) -> MapblockHandle;
    fn nvim_mapblock_rehash(
        buf: BufHandle,
        is_buf_local: c_int,
        old_hash: c_int,
        new_hash: c_int,
        mp: MapblockHandle,
    );

    // For do_map
    fn rs_str_to_mapargs(
        strargs: *const c_char,
        is_unmap: c_int,
        mapargs: *mut MapArguments,
    ) -> c_int;
    fn xfree(ptr: *mut c_char);
}

// =============================================================================
// Helpers
// =============================================================================

#[inline]
fn got_int() -> bool {
    unsafe { nvim_get_got_int() != 0 }
}

#[inline]
fn ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Get the head of a hash/abbr list (global or buffer-local).
unsafe fn get_list_head(
    buf: BufHandle,
    hash: c_int,
    is_abbr: bool,
    is_buf_local: bool,
) -> MapblockHandle {
    if is_abbr {
        if is_buf_local {
            nvim_buf_get_first_abbr(buf)
        } else {
            nvim_get_first_abbr()
        }
    } else if is_buf_local {
        nvim_buf_get_maphash_entry(buf, hash)
    } else {
        nvim_get_maphash_entry(hash)
    }
}

// =============================================================================
// Abbreviation validation
// =============================================================================

/// Validate that an abbreviation LHS has valid word-character patterns.
/// Returns true if valid, false if invalid.
unsafe fn validate_abbrev_lhs(lhs: *const c_char, len: c_int) -> bool {
    // If an abbreviation ends in a keyword character, the
    // rest must be all keyword-char or all non-keyword-char.
    let mut same: c_int = -1;

    let first = nvim_vim_iswordp(lhs);
    let mut last = first;
    let mut p = lhs.add(nvim_mapping_utfc_ptr2len(lhs) as usize);
    let mut n: c_int = 1;
    let end = lhs.add(len as usize);
    while p < end {
        n += 1;
        last = nvim_vim_iswordp(p);
        if same == -1 && last != first {
            same = n - 1;
        }
        p = p.add(nvim_mapping_utfc_ptr2len(p) as usize);
    }
    if last != 0 && n > 2 && same >= 0 && same < n - 1 {
        return false;
    }

    // An abbreviation cannot contain white space.
    for i in 0..len {
        if ascii_iswhite(*lhs.add(i as usize) as u8) {
            return false;
        }
    }

    true
}

// =============================================================================
// buf_do_map
// =============================================================================

/// The core mapping engine. Handles `:map`, `:unmap`, `:noremap`, listing,
/// and all mode/abbreviation combinations.
///
/// # Safety
/// All pointer parameters must be valid. `buf` must be a valid buffer handle.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_buf_do_map(
    maptype_in: c_int,
    args: *mut MapArguments,
    mode: c_int,
    is_abbrev: c_int,
    buf: BufHandle,
) -> c_int {
    let is_abbr = is_abbrev != 0;
    let is_buf_local = (*args).buffer;

    let mut mp_result: [MapblockHandle; 2] = [
        MapblockHandle(std::ptr::null_mut()),
        MapblockHandle(std::ptr::null_mut()),
    ];

    let mut unmap_lhs_only = false;
    let mut maptype = maptype_in;
    if maptype == MAPTYPE_UNMAP_LHS {
        unmap_lhs_only = true;
        maptype = MAPTYPE_UNMAP;
    }

    // For ":noremap" don't remap, otherwise do remap.
    let noremap = if (*args).script {
        REMAP_SCRIPT
    } else if maptype == MAPTYPE_NOREMAP {
        REMAP_NONE
    } else {
        REMAP_YES
    };

    let has_lhs = (*args).lhs[0] != 0;
    let has_rhs = (*args).rhs_lua != LUA_NOREF
        || (!(*args).rhs.is_null() && *(*args).rhs != 0)
        || (*args).rhs_is_noop;
    let do_print = !has_lhs || (maptype != MAPTYPE_UNMAP && !has_rhs);

    if do_print {
        nvim_mapping_msg_ext_set_kind_list_cmd();
    }

    // Check for :unmap without argument
    if maptype == MAPTYPE_UNMAP && !has_lhs {
        return finish(args, &mp_result, 1);
    }

    let did_simplify = (*args).alt_lhs_len != 0;

    // The following is done twice if we have two versions of keys
    for keyround in 1..=2 {
        let mut did_it = false;
        let mut did_local = false;
        let keyround1_simplified = keyround == 1 && did_simplify;

        // Determine which LHS to use for this keyround
        let (lhs, len) = get_lhs_for_keyround(args, keyround, did_simplify, do_print);

        // Validate arguments
        if has_lhs {
            if len > MAXMAPLEN as c_int {
                return finish(args, &mp_result, 1);
            }

            if is_abbr && maptype != MAPTYPE_UNMAP && !validate_abbrev_lhs(lhs, len) {
                return finish(args, &mp_result, 1);
            }
        }

        if has_lhs && has_rhs && is_abbr {
            nvim_mapping_set_no_abbr(0); // reset flag: abbreviations now exist
        }

        if do_print {
            nvim_msg_start();
        }

        // Check if a new local mapping wasn't already defined globally.
        if (*args).unique
            && is_buf_local
            && has_lhs
            && has_rhs
            && maptype != MAPTYPE_UNMAP
            && check_global_conflict(buf, is_abbr, mode, lhs, len)
        {
            return finish(args, &mp_result, 6);
        }

        // When listing global mappings, also list buffer-local ones here.
        if !is_buf_local && !has_rhs && maptype != MAPTYPE_UNMAP {
            did_local = list_buf_local_mappings(buf, is_abbr, mode, has_lhs, lhs, len);
        }

        // Find matching entries in hash list
        let result = process_matching_entries(
            buf,
            args,
            maptype,
            mode,
            is_abbr,
            is_buf_local,
            has_lhs,
            has_rhs,
            lhs,
            len,
            noremap,
            keyround,
            keyround1_simplified,
            unmap_lhs_only,
            &mut did_it,
            &mut mp_result,
        );
        if let Some(retval) = result {
            return finish(args, &mp_result, retval);
        }

        // Post-processing after the matching loop
        if maptype == MAPTYPE_UNMAP {
            if !did_it {
                if !keyround1_simplified {
                    return finish(args, &mp_result, 2); // no match
                }
            } else if *lhs as u8 == CTRL_C {
                // If CTRL-C has been unmapped, reuse it for Interrupting.
                if is_buf_local {
                    let cur = nvim_mapping_buf_get_mapped_ctrl_c(buf);
                    nvim_mapping_buf_set_mapped_ctrl_c(buf, cur & !mode);
                } else {
                    let cur = nvim_get_mapped_ctrl_c();
                    nvim_set_mapped_ctrl_c(cur & !mode);
                }
            }
            continue;
        }

        if !has_lhs || !has_rhs {
            // Print entries
            if !did_it && !did_local {
                nvim_mapping_msg_no_mapping(c_int::from(is_abbr));
            }
            return finish(args, &mp_result, 0); // listing finished
        }

        if did_it {
            continue; // already added the new entry
        }

        // Add a new entry to the maphash[] list or abbrlist.
        mp_result[keyround as usize - 1] = nvim_map_add(
            buf,
            c_int::from(is_buf_local),
            lhs,
            args,
            noremap,
            mode,
            c_int::from(is_abbr),
            0, // sid
            0, // lnum
            c_int::from(keyround1_simplified),
        );
    }

    // Link alternates
    if !mp_result[0].is_null() && !mp_result[1].is_null() {
        nvim_mapblock_set_alt(mp_result[0], mp_result[1]);
    }

    finish(args, &mp_result, 0)
}

/// Determine which LHS pointer and length to use for a given keyround.
unsafe fn get_lhs_for_keyround(
    args: *const MapArguments,
    keyround: c_int,
    did_simplify: bool,
    do_print: bool,
) -> (*const c_char, c_int) {
    if keyround == 2 {
        if !did_simplify {
            // This shouldn't happen since the for loop would break,
            // but return the alt_lhs anyway.
            return ((*args).alt_lhs.as_ptr(), (*args).alt_lhs_len as c_int);
        }
        ((*args).alt_lhs.as_ptr(), (*args).alt_lhs_len as c_int)
    } else if did_simplify && do_print {
        // When printing, always use the not-simplified map
        ((*args).alt_lhs.as_ptr(), (*args).alt_lhs_len as c_int)
    } else {
        ((*args).lhs.as_ptr(), (*args).lhs_len as c_int)
    }
}

/// Clean up and return from buf_do_map.
unsafe fn finish(args: *mut MapArguments, mp_result: &[MapblockHandle; 2], retval: c_int) -> c_int {
    if !mp_result[0].is_null() || !mp_result[1].is_null() {
        nvim_mapargs_take_ownership(args);
    }
    retval
}

/// Check if a global mapping conflicts with a new unique local mapping.
/// Returns true if conflict found.
unsafe fn check_global_conflict(
    _buf: BufHandle,
    is_abbr: bool,
    mode: c_int,
    lhs: *const c_char,
    len: c_int,
) -> bool {
    for hash in 0..256 {
        if got_int() {
            break;
        }
        if is_abbr && hash > 0 {
            break; // only one abbreviation list
        }

        let mp_head = if is_abbr {
            nvim_get_first_abbr()
        } else {
            nvim_get_maphash_entry(hash)
        };

        let mut mp = mp_head;
        while !mp.is_null() && !got_int() {
            if (mapblock_mode(mp) & mode) != 0
                && mapblock_keylen(mp) == len
                && libc::strncmp(mapblock_keys(mp), lhs, len as usize) == 0
            {
                return true;
            }
            mp = mapblock_next(mp);
        }
    }
    false
}

/// List buffer-local mappings when listing global mappings.
/// Returns true if any were displayed.
unsafe fn list_buf_local_mappings(
    buf: BufHandle,
    is_abbr: bool,
    mode: c_int,
    has_lhs: bool,
    lhs: *const c_char,
    len: c_int,
) -> bool {
    let mut did_local = false;

    for hash in 0..256 {
        if got_int() {
            break;
        }
        if is_abbr && hash > 0 {
            break;
        }

        let mp_head = if is_abbr {
            nvim_buf_get_first_abbr(buf)
        } else {
            nvim_buf_get_maphash_entry(buf, hash)
        };

        let mut mp = mp_head;
        while !mp.is_null() && !got_int() {
            if !mapblock_simplified(mp) && (mapblock_mode(mp) & mode) != 0 {
                if has_lhs {
                    let n = mapblock_keylen(mp);
                    let minlen = if n < len { n } else { len };
                    if libc::strncmp(mapblock_keys(mp), lhs, minlen as usize) == 0 {
                        nvim_showmap(mp, 1);
                        did_local = true;
                    }
                } else {
                    nvim_showmap(mp, 1);
                    did_local = true;
                }
            }
            mp = mapblock_next(mp);
        }
    }

    did_local
}

/// Process the matching entries loop (the core inner loop of buf_do_map).
///
/// Returns `Some(retval)` if we should return early with that retval,
/// or `None` if processing should continue normally.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::fn_params_excessive_bools)]
unsafe fn process_matching_entries(
    buf: BufHandle,
    args: *mut MapArguments,
    maptype: c_int,
    mode: c_int,
    is_abbr: bool,
    is_buf_local: bool,
    has_lhs: bool,
    has_rhs: bool,
    lhs: *const c_char,
    len: c_int,
    noremap: c_int,
    keyround: c_int,
    keyround1_simplified: bool,
    unmap_lhs_only: bool,
    did_it: &mut bool,
    mp_result: &mut [MapblockHandle; 2],
) -> Option<c_int> {
    // For :unmap we may loop two times: once for LHS match, once for RHS match.
    let num_rounds = if maptype == MAPTYPE_UNMAP && !unmap_lhs_only {
        2
    } else {
        1
    };

    for round in 0..num_rounds {
        if *did_it || got_int() {
            break;
        }

        let (hash_start, hash_end) = if (round == 0 && has_lhs) || is_abbr {
            let h = if is_abbr {
                0
            } else {
                map_hash(mode, *lhs as u8)
            };
            (h, h + 1)
        } else {
            (0, 256)
        };

        for hash in hash_start..hash_end {
            if got_int() {
                break;
            }

            let mut mp = get_list_head(buf, hash, is_abbr, is_buf_local);

            while !mp.is_null() && !got_int() {
                if (mapblock_mode(mp) & mode) == 0 {
                    // Skip entries with wrong mode
                    mp = mapblock_next(mp);
                    continue;
                }

                if !has_lhs {
                    // Show all entries
                    if !mapblock_simplified(mp) {
                        nvim_showmap(mp, c_int::from(is_buf_local));
                        *did_it = true;
                    }
                    mp = mapblock_next(mp);
                    continue;
                }

                // Do we have a match?
                let (n, p) = if round != 0 {
                    // Second round: try unmap "rhs" string
                    (nvim_mapblock_get_str_len(mp), mapblock_str(mp))
                } else {
                    (mapblock_keylen(mp), mapblock_keys(mp))
                };

                let minlen = if n < len { n } else { len };
                if libc::strncmp(p, lhs, minlen as usize) != 0 {
                    mp = mapblock_next(mp);
                    continue;
                }

                // We have a prefix match. Handle based on maptype.
                if maptype == MAPTYPE_UNMAP {
                    // Delete entry.
                    // Only accept a full match. For abbreviations we ignore
                    // trailing space when matching with the "lhs".
                    if n != len
                        && (!is_abbr
                            || round != 0
                            || n > len
                            || *rs_skipwhite(lhs.add(n as usize)) != 0)
                    {
                        mp = mapblock_next(mp);
                        continue;
                    }
                    // In keyround for simplified keys, don't unmap
                    // a mapping without m_simplified flag.
                    if keyround1_simplified && !mapblock_simplified(mp) {
                        break;
                    }
                    // Reset the indicated mode bits.
                    nvim_mapblock_set_mode(mp, mapblock_mode(mp) & !mode);
                    *did_it = true;
                } else if !has_rhs {
                    // Show matching entry
                    if !mapblock_simplified(mp) {
                        nvim_showmap(mp, c_int::from(is_buf_local));
                        *did_it = true;
                    }
                } else if n != len {
                    // New entry is ambiguous
                    mp = mapblock_next(mp);
                    continue;
                } else if keyround1_simplified && !mapblock_simplified(mp) {
                    // In keyround for simplified keys, don't replace
                    // a mapping without m_simplified flag.
                    *did_it = true;
                    break;
                } else if (*args).unique {
                    return Some(5); // entry not unique
                } else {
                    // New rhs for existing entry
                    nvim_mapblock_set_mode(mp, mapblock_mode(mp) & !mode);
                    if mapblock_mode(mp) == 0 && !*did_it {
                        // Reuse entry
                        nvim_mapblock_reuse(
                            mp,
                            args,
                            noremap,
                            mode,
                            c_int::from(keyround1_simplified),
                        );
                        mp_result[keyround as usize - 1] = mp;
                        *did_it = true;
                    }
                }

                // Check if entry should be deleted (mode == 0)
                if mapblock_mode(mp) == 0 {
                    mp = nvim_mapblock_free_in_list(
                        buf,
                        hash,
                        c_int::from(is_abbr),
                        c_int::from(is_buf_local),
                        mp,
                    );
                    continue;
                }

                // May need to put this entry into another hash list.
                if !is_abbr {
                    let keys = mapblock_keys(mp);
                    let first_char = if keys.is_null() { 0u8 } else { *keys as u8 };
                    let new_hash = map_hash(mapblock_mode(mp), first_char);
                    if new_hash != hash {
                        nvim_mapblock_rehash(buf, c_int::from(is_buf_local), hash, new_hash, mp);
                        // After rehash, the current position in the old hash
                        // list has been updated. Re-read the head.
                        mp = get_list_head(buf, hash, is_abbr, is_buf_local);
                        continue;
                    }
                }

                mp = mapblock_next(mp);
            }
        }
    }

    None
}

// =============================================================================
// do_map
// =============================================================================

/// Parse and execute a `:map`/`:unmap`/`:noremap` command.
///
/// # Safety
/// `arg` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_do_map(
    maptype: c_int,
    arg: *mut c_char,
    mode: c_int,
    is_abbrev: c_int,
) -> c_int {
    let mut parsed_args: MapArguments = std::mem::zeroed();
    parsed_args.rhs_lua = LUA_NOREF;

    let result = rs_str_to_mapargs(
        arg,
        c_int::from(maptype == MAPTYPE_UNMAP),
        &raw mut parsed_args,
    );
    if result != 0 {
        // Invalid arguments — clean up and return
        xfree(parsed_args.rhs);
        xfree(parsed_args.orig_rhs);
        return result;
    }

    let buf = nvim_get_curbuf();
    let retval = rs_buf_do_map(maptype, &raw mut parsed_args, mode, is_abbrev, buf);

    xfree(parsed_args.rhs);
    xfree(parsed_args.orig_rhs);
    retval
}
