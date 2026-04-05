//! Ex command support and abbreviation search.
//!
//! Provides `rs_find_matching_abbr` (find abbreviation matching typed text),
//! `rs_get_maptype` (determine maptype from command character),
//! and `do_exmap`/`ex_map`/`ex_abbreviate`/`ex_unmap`/`ex_mapclear`/`ex_abclear`.

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

// =============================================================================
// Ex command implementations
// =============================================================================

use nvim_ex_cmds_types::ExArgHandle;

extern "C" {
    // Error reporting
    fn emsg(s: *const c_char) -> c_int;
    fn semsg(fmt: *const c_char, ...) -> c_int;

    // Globally-accessible error strings from errors.h
    static e_invarg: [c_char; 0];
    static e_noabbr: [c_char; 0];
    static e_nomap: [c_char; 0];

    // Local mapping error strings (via C accessor functions)
    fn nvim_mapping_e_abbr_exists(abbr: c_int) -> *const c_char;
    fn nvim_mapping_e_global_abbr_exists(abbr: c_int) -> *const c_char;

    // Already-Rust functions called via FFI link
    fn rs_get_map_mode(cmdp: *mut *mut c_char, forceit: c_int) -> c_int;
    fn rs_str_to_mapargs(
        strargs: *const c_char,
        is_unmap: c_int,
        mapargs: *mut crate::args::MapArguments,
    ) -> c_int;
    fn rs_buf_do_map(
        maptype: c_int,
        args: *mut crate::args::MapArguments,
        mode: c_int,
        is_abbrev: c_int,
        buf: BufHandle,
    ) -> c_int;
    fn rs_do_mapclear(cmdp: *mut c_char, arg: *mut c_char, forceit: c_int, abbr: c_int);

    // message output for ex_map
    fn msg_outtrans(str_: *const c_char, hl_id: c_int, hist: bool) -> c_int;
    fn msg_putchar(c: c_int);

    // 'secure' global
    static mut secure: c_int;
}

/// Parse and execute :map/:unmap/:abbreviate commands.
///
/// This is the Rust implementation of the C `do_exmap` static function.
///
/// # Safety
/// `eap` must be a valid non-null ExArg pointer.
unsafe fn rs_do_exmap(eap: ExArgHandle, isabbrev: c_int) {
    use crate::args::MapArguments;
    use std::mem::MaybeUninit;

    let cmdp = (*eap).cmd;
    let mut cmdp_ptr = cmdp;
    let mode = rs_get_map_mode(
        std::ptr::addr_of_mut!(cmdp_ptr),
        c_int::from((*eap).forceit != 0 || isabbrev != 0),
    );

    let maptype = rs_get_maptype(c_int::from(*cmdp_ptr as u8));

    // Zero-initialize MapArguments (Rust side)
    let mut parsed_args: MaybeUninit<MapArguments> = MaybeUninit::zeroed();
    let parsed_args_ptr = parsed_args.as_mut_ptr();

    let result = rs_str_to_mapargs(
        (*eap).arg,
        c_int::from(maptype == MAPTYPE_UNMAP),
        parsed_args_ptr,
    );

    match result {
        0 => {}
        1 => {
            emsg(e_invarg.as_ptr());
            xfree((*parsed_args_ptr).rhs);
            xfree((*parsed_args_ptr).orig_rhs);
            return;
        }
        _ => {
            // Unknown return code from rs_str_to_mapargs - should not happen
            xfree((*parsed_args_ptr).rhs);
            xfree((*parsed_args_ptr).orig_rhs);
            return;
        }
    }

    let buf = nvim_get_curbuf();
    let retval = rs_buf_do_map(maptype, parsed_args_ptr, mode, isabbrev, buf);
    if retval == 1 {
        emsg(e_invarg.as_ptr());
    } else if retval == 2 {
        emsg(if isabbrev != 0 {
            e_noabbr.as_ptr()
        } else {
            e_nomap.as_ptr()
        });
    } else if retval == 5 {
        let lhs = (*parsed_args_ptr).lhs.as_ptr();
        semsg(nvim_mapping_e_abbr_exists(isabbrev), lhs);
    } else if retval == 6 {
        let lhs = (*parsed_args_ptr).lhs.as_ptr();
        semsg(nvim_mapping_e_global_abbr_exists(isabbrev), lhs);
    }

    xfree((*parsed_args_ptr).rhs);
    xfree((*parsed_args_ptr).orig_rhs);
}

/// ":abbreviate" and friends.
///
/// # Safety
/// `eap` must be a valid non-null ExArg pointer.
#[export_name = "ex_abbreviate"]
pub unsafe extern "C" fn rs_ex_abbreviate(eap: ExArgHandle) {
    rs_do_exmap(eap, 1);
}

/// ":map" and friends.
///
/// # Safety
/// `eap` must be a valid non-null ExArg pointer.
#[export_name = "ex_map"]
pub unsafe extern "C" fn rs_ex_map(eap: ExArgHandle) {
    // If we are in a secure mode we print the mappings for security reasons.
    if secure != 0 {
        secure = 2;
        msg_outtrans((*eap).cmd, 0, false);
        msg_putchar(c_int::from(b'\n'));
    }
    rs_do_exmap(eap, 0);
}

/// ":unmap" and friends.
///
/// # Safety
/// `eap` must be a valid non-null ExArg pointer.
#[export_name = "ex_unmap"]
pub unsafe extern "C" fn rs_ex_unmap(eap: ExArgHandle) {
    rs_do_exmap(eap, 0);
}

/// ":mapclear" and friends.
///
/// # Safety
/// `eap` must be a valid non-null ExArg pointer.
#[export_name = "ex_mapclear"]
pub unsafe extern "C" fn rs_ex_mapclear(eap: ExArgHandle) {
    rs_do_mapclear((*eap).cmd, (*eap).arg, (*eap).forceit, 0);
}

/// ":abclear" and friends.
///
/// # Safety
/// `eap` must be a valid non-null ExArg pointer.
#[export_name = "ex_abclear"]
pub unsafe extern "C" fn rs_ex_abclear(eap: ExArgHandle) {
    rs_do_mapclear((*eap).cmd, (*eap).arg, 1, 1);
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
