//! Ex command support and abbreviation search.
//!
//! Provides `rs_find_matching_abbr` (find abbreviation matching typed text)
//! and `rs_get_maptype` (determine maptype from command character).

use std::ffi::{c_char, c_int};

use crate::{
    mapblock_keylen, mapblock_keys, mapblock_mode, mapblock_next, BufHandle, MapblockHandle,
};

// =============================================================================
// Constants
// =============================================================================

const K_SPECIAL: u8 = 0x80;

const MAPTYPE_MAP: c_int = 0;
const MAPTYPE_UNMAP: c_int = 1;
const MAPTYPE_NOREMAP: c_int = 2;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    fn nvim_get_first_abbr() -> MapblockHandle;
    fn nvim_buf_get_first_abbr(buf: BufHandle) -> MapblockHandle;
    fn nvim_get_curbuf() -> BufHandle;

    fn nvim_mapping_vim_unescape_ks(s: *mut c_char);
    fn nvim_mapping_get_state() -> c_int;

    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_char);
}

// =============================================================================
// rs_get_maptype
// =============================================================================

/// Determine the maptype from the command name character.
///
/// - 'n' → NOREMAP
/// - 'u' → UNMAP
/// - anything else → MAP
#[no_mangle]
pub extern "C" fn rs_get_maptype(cmdchar: c_int) -> c_int {
    match cmdchar as u8 {
        b'n' => MAPTYPE_NOREMAP,
        b'u' => MAPTYPE_UNMAP,
        _ => MAPTYPE_MAP,
    }
}

// =============================================================================
// rs_find_matching_abbr
// =============================================================================

/// Search buffer-local and global abbreviation lists for a match.
///
/// Given the typed text `ptr` of length `len` bytes, search the abbreviation
/// lists for an entry whose key matches. First searches buffer-local
/// abbreviations, then global ones.
///
/// Returns the matching mapblock handle, or null if no match found.
///
/// # Safety
/// `ptr` must be a valid pointer to at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_find_matching_abbr(ptr: *const c_char, len: c_int) -> MapblockHandle {
    let state = nvim_mapping_get_state();
    let curbuf = nvim_get_curbuf();

    // Search buffer-local first, then global
    let buf_abbr = nvim_buf_get_first_abbr(curbuf);
    let global_abbr = nvim_get_first_abbr();

    let mut mp = buf_abbr;
    let mut mp2 = global_abbr;

    // If buffer-local list is empty, start with global
    if mp.is_null() {
        mp = mp2;
        mp2 = std::ptr::null_mut();
    }

    while !mp.is_null() {
        let qlen = mapblock_keylen(mp);
        let q = mapblock_keys(mp);

        // Check if keys contain K_SPECIAL that needs unescaping
        let has_special = has_k_special(q);

        let (match_q, match_qlen, allocated) = if has_special {
            let dup = xstrdup(q);
            nvim_mapping_vim_unescape_ks(dup);
            let new_len = libc::strlen(dup.cast()) as c_int;
            (dup, new_len, true)
        } else {
            (q.cast_mut(), qlen, false)
        };

        // Check mode and key match
        let is_match = (mapblock_mode(mp) & state) != 0
            && match_qlen == len
            && libc::strncmp(match_q, ptr, len as usize) == 0;

        if allocated {
            xfree(match_q);
        }

        if is_match {
            return mp;
        }

        // Advance: when we reach end of first list, switch to second
        let next = mapblock_next(mp);
        if next.is_null() && !mp2.is_null() {
            mp = mp2;
            mp2 = std::ptr::null_mut();
        } else {
            mp = next;
        }
    }

    std::ptr::null_mut()
}

/// Check if a C string contains K_SPECIAL byte.
unsafe fn has_k_special(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }
    let mut p = s.cast::<u8>();
    while *p != 0 {
        if *p == K_SPECIAL {
            return true;
        }
        p = p.add(1);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_maptype() {
        assert_eq!(rs_get_maptype(c_int::from(b'n')), MAPTYPE_NOREMAP);
        assert_eq!(rs_get_maptype(c_int::from(b'u')), MAPTYPE_UNMAP);
        assert_eq!(rs_get_maptype(c_int::from(b'm')), MAPTYPE_MAP);
        assert_eq!(rs_get_maptype(c_int::from(b'a')), MAPTYPE_MAP);
        assert_eq!(rs_get_maptype(0), MAPTYPE_MAP);
    }
}
