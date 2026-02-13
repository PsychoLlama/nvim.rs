//! Mapping mutation primitives.
//!
//! Provides `map_clear_mode` and `do_mapclear` which clear mappings
//! matching a given mode from the global or buffer-local tables.

use std::ffi::{c_char, c_int};

use crate::{map_hash, mapblock_keys, mapblock_mode, mapblock_next, BufHandle, MapblockHandle};

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    fn nvim_get_maphash_entry(index: c_int) -> MapblockHandle;
    fn nvim_get_first_abbr() -> MapblockHandle;
    fn nvim_buf_get_maphash_entry(buf: BufHandle, index: c_int) -> MapblockHandle;
    fn nvim_buf_get_first_abbr(buf: BufHandle) -> MapblockHandle;
    fn nvim_get_curbuf() -> BufHandle;

    fn nvim_set_maphash_entry(index: c_int, mp: MapblockHandle);
    fn nvim_set_first_abbr(mp: MapblockHandle);
    fn nvim_buf_set_maphash_entry(buf: BufHandle, index: c_int, mp: MapblockHandle);
    fn nvim_buf_set_first_abbr(buf: BufHandle, mp: MapblockHandle);

    fn nvim_mapblock_set_next(mp: MapblockHandle, next: MapblockHandle);
    fn nvim_mapblock_set_mode(mp: MapblockHandle, mode: c_int);
    fn nvim_mapblock_free(mp: MapblockHandle);

    fn rs_get_map_mode(cmdp: *mut *mut c_char, forceit: c_int) -> c_int;
    fn nvim_mapping_emsg_invarg();
}

// =============================================================================
// map_clear_mode
// =============================================================================

/// Get the head of a hash/abbr list.
unsafe fn get_list_head(buf: BufHandle, hash: c_int, abbr: bool, local: bool) -> MapblockHandle {
    if abbr {
        if local {
            nvim_buf_get_first_abbr(buf)
        } else {
            nvim_get_first_abbr()
        }
    } else if local {
        nvim_buf_get_maphash_entry(buf, hash)
    } else {
        nvim_get_maphash_entry(hash)
    }
}

/// Set the head of a hash/abbr list.
unsafe fn set_list_head(buf: BufHandle, hash: c_int, abbr: bool, local: bool, mp: MapblockHandle) {
    if abbr {
        if local {
            nvim_buf_set_first_abbr(buf, mp);
        } else {
            nvim_set_first_abbr(mp);
        }
    } else if local {
        nvim_buf_set_maphash_entry(buf, hash, mp);
    } else {
        nvim_set_maphash_entry(hash, mp);
    }
}

/// Remove `target` from a singly-linked list whose head is identified
/// by (buf, hash, abbr, local). Updates the head or the previous node's
/// m_next pointer as needed.
unsafe fn unlink_from_list(
    buf: BufHandle,
    hash: c_int,
    abbr: bool,
    local: bool,
    target: MapblockHandle,
) {
    let head = get_list_head(buf, hash, abbr, local);

    // If target is the head
    if head == target {
        set_list_head(buf, hash, abbr, local, mapblock_next(target));
        return;
    }

    // Walk to find the predecessor
    let mut prev = head;
    while !prev.is_null() {
        let next = mapblock_next(prev);
        if next == target {
            nvim_mapblock_set_next(prev, mapblock_next(target));
            return;
        }
        prev = next;
    }
}

/// Insert `mp` at the head of a hash bucket list.
unsafe fn insert_at_head(buf: BufHandle, new_hash: c_int, local: bool, mp: MapblockHandle) {
    let old_head = if local {
        nvim_buf_get_maphash_entry(buf, new_hash)
    } else {
        nvim_get_maphash_entry(new_hash)
    };
    nvim_mapblock_set_next(mp, old_head);
    if local {
        nvim_buf_set_maphash_entry(buf, new_hash, mp);
    } else {
        nvim_set_maphash_entry(new_hash, mp);
    }
}

/// Clear all mappings matching `mode` from the global or buffer-local tables.
///
/// For each entry whose mode overlaps with `mode`:
/// - The matching mode bits are removed.
/// - If no mode bits remain, the entry is freed.
/// - Otherwise the entry may be re-hashed if the first-char hash changed.
///
/// # Safety
/// `buf` must be a valid buffer handle.
#[no_mangle]
pub unsafe extern "C" fn rs_map_clear_mode(buf: BufHandle, mode: c_int, local: c_int, abbr: c_int) {
    let is_local = local != 0;
    let is_abbr = abbr != 0;

    for hash in 0..256 {
        if is_abbr && hash > 0 {
            break; // only one abbreviation list
        }

        // Collect entries that match the mode — we process them in a separate
        // pass to avoid iterator-invalidation problems when unlinking/re-hashing.
        let mut to_process: Vec<MapblockHandle> = Vec::new();
        let mut cur = get_list_head(buf, hash, is_abbr, is_local);
        while !cur.is_null() {
            if (mapblock_mode(cur) & mode) != 0 {
                to_process.push(cur);
            }
            cur = mapblock_next(cur);
        }

        for mp in to_process {
            let old_mode = mapblock_mode(mp);
            let new_mode = old_mode & !mode;

            if new_mode == 0 {
                // Entry should be fully removed and freed.
                unlink_from_list(buf, hash, is_abbr, is_local, mp);
                nvim_mapblock_free(mp);
            } else {
                // Update mode bits.
                nvim_mapblock_set_mode(mp, new_mode);

                // Check if entry needs to move to a different hash bucket.
                if !is_abbr {
                    let keys = mapblock_keys(mp);
                    let first_char = if keys.is_null() { 0u8 } else { *keys as u8 };
                    let new_hash = map_hash(new_mode, first_char);
                    if new_hash != hash {
                        unlink_from_list(buf, hash, false, is_local, mp);
                        insert_at_head(buf, new_hash, is_local, mp);
                    }
                }
            }
        }
    }
}

// =============================================================================
// do_mapclear
// =============================================================================

/// Parse and execute a `:mapclear` command.
///
/// # Safety
/// `cmdp` and `arg` must be valid NUL-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_do_mapclear(
    cmdp: *mut c_char,
    arg: *mut c_char,
    forceit: c_int,
    abbr: c_int,
) {
    let is_local = libc::strcmp(arg, c"<buffer>".as_ptr()) == 0;
    if !is_local && *arg != 0 {
        nvim_mapping_emsg_invarg();
        return;
    }

    let mut cmd = cmdp;
    let mode = rs_get_map_mode(std::ptr::addr_of_mut!(cmd), forceit);
    let buf = nvim_get_curbuf();
    rs_map_clear_mode(buf, mode, c_int::from(is_local), abbr);
}
