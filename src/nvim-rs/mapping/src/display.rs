//! Display functions for key mappings.
//!
//! Provides `showmap` (display a mapping to the user) and `rs_map_add`
//! (allocate and initialize a new mapblock, insert into hash/abbr list).

#![allow(clippy::cast_ptr_alignment)] // Intentional: xcalloc result cast to MapblockT

use std::ffi::{c_char, c_int, c_void};

use crate::args::MapArguments;
use crate::{map_hash, BufHandle, MapblockHandle, MapblockT, SctxT};

// =============================================================================
// Constants
// =============================================================================

/// LUA_NOREF: Lua reference that is not set.
const LUA_NOREF: c_int = -2;

/// REMAP_NONE: disable remapping.
const REMAP_NONE: c_int = -1;

/// REMAP_SCRIPT: remap via &lt;SID&gt;.
const REMAP_SCRIPT: c_int = -2;

/// HLF_8: highlight group for Meta/special keys in :map listing.
/// Enum value: HLF_NONE=0, HLF_8=1.
const HLF_8: c_int = 1;

/// Ctrl-C character.
const CTRL_C: u8 = 3;

/// VAR_UNKNOWN: typval_T v_type value for "no value".
const VAR_UNKNOWN: c_int = 0;

// kUIMessages enum value from ui_defs.h
// UIExtension enum: kUICmdline=0, kUIWildmenu=1, kUIMessages=2
const K_UI_MESSAGES: c_int = 2;

// NUMBUFLEN: buffer size for tv_get_string_buf (from eval/typval_defs.h = 65)
const NUMBUFLEN: usize = 65;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Message functions
    fn message_filtered(str_: *const c_char) -> bool;
    fn msg_putchar(c: c_int);
    fn msg_puts(s: *const c_char);
    fn msg_puts_hl(s: *const c_char, hlf: c_int, force: bool);
    fn msg_outtrans_special(str_: *const c_char, at_start: bool, col: c_int) -> c_int;
    fn msg_clr_eos();
    fn last_set_msg(sctx: SctxT);

    // UI/global state
    fn ui_has(ext: c_int) -> bool;

    // Mapping mode chars (Rust export -- same binary)
    fn map_mode_to_chars(mode: c_int, buf: *mut c_char);

    // Lua
    fn nlua_funcref_str(luaref: c_int, arena: *mut c_void) -> *mut c_char;
    fn nlua_set_sctx(ctx: *mut SctxT);

    // Memory
    fn xcalloc(count: usize, size: usize) -> *mut c_char;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_char);

    // Global mapped_ctrl_c
    static mut mapped_ctrl_c: c_int;
    fn nvim_mapping_buf_get_mapped_ctrl_c(buf: BufHandle) -> c_int;
    fn nvim_mapping_buf_set_mapped_ctrl_c(buf: BufHandle, val: c_int);

    // SOURCING_LNUM accessor (in debugger.c)
    fn nvim_dbg_get_sourcing_lnum() -> i64;

    // current_sctx global (non-static in globals.h)
    static mut current_sctx: SctxT;

    // Global flags
    static mut got_int: bool;
    static mut msg_didout: c_int;
    static mut msg_silent: c_int;
    static mut p_verbose: c_int;

    // typval accessors (from eval/typval.c and eval_shim.c)
    fn tv_get_string(tv: *const c_void) -> *const c_char;
    fn tv_get_string_buf(tv: *const c_void, buf: *mut c_char) -> *const c_char;
    fn tv_get_number(tv: *const c_void) -> i64;

    // maphash/abbr list getters
    fn nvim_get_maphash_entry(index: c_int) -> MapblockHandle;
    fn nvim_get_first_abbr() -> MapblockHandle;
    fn nvim_buf_get_maphash_entry(buf: BufHandle, index: c_int) -> MapblockHandle;
    fn nvim_buf_get_first_abbr(buf: BufHandle) -> MapblockHandle;

    // maphash/abbr list setters
    fn nvim_set_maphash_entry(index: c_int, mp: MapblockHandle);
    fn nvim_set_first_abbr(mp: MapblockHandle);
    fn nvim_buf_set_maphash_entry(buf: BufHandle, index: c_int, mp: MapblockHandle);
    fn nvim_buf_set_first_abbr(buf: BufHandle, mp: MapblockHandle);
}

// =============================================================================
// typval helpers
// =============================================================================

/// Get the v_type field of a typval_T (offset 0, c_int).
///
/// # Safety
/// `tv` must be a valid non-null typval_T pointer.
#[inline]
unsafe fn tv_type(tv: *const c_void) -> c_int {
    *(tv.cast::<c_int>())
}

/// Get a pointer to argvars[i].
///
/// typval_T is 16 bytes: v_type(4) + v_lock(4) + vval(8).
///
/// # Safety
/// `argvars` must point to an array of at least `i+1` typval_T values.
#[inline]
unsafe fn tv_idx(argvars: *const c_void, i: usize) -> *const c_void {
    argvars.cast::<u8>().add(i * 16).cast()
}

/// Set the vval.v_number field of a typval_T (offset 8, i64).
///
/// # Safety
/// `tv` must be a valid non-null typval_T pointer.
#[inline]
#[allow(clippy::cast_ptr_alignment)]
unsafe fn tv_set_number(tv: *mut c_void, val: i64) {
    // SAFETY: typval_T layout guarantees v_number at offset 8, aligned to 8 bytes.
    *tv.cast::<u8>().add(8).cast::<i64>() = val;
}

// =============================================================================
// showmap
// =============================================================================

/// Display a mapping to the user via msg_* functions.
///
/// # Safety
/// `mp` must be a valid non-null mapblock pointer.
#[export_name = "showmap"]
pub unsafe extern "C" fn rs_showmap(mp: MapblockHandle, local: bool) {
    if mp.is_null() {
        return;
    }

    let m_keys = (*mp).m_keys;
    let m_str = (*mp).m_str;
    let m_desc = (*mp).m_desc;

    // Filter check: if all relevant strings are filtered, skip display.
    if message_filtered(m_keys)
        && message_filtered(m_str)
        && (m_desc.is_null() || message_filtered(m_desc))
    {
        return;
    }

    // When ext_messages is active, msg_didout is never set.
    if msg_didout != 0 || msg_silent != 0 || ui_has(K_UI_MESSAGES) {
        msg_putchar(c_int::from(b'\n'));
        if got_int {
            return;
        }
    }

    let mut mapchars = [0i8; 7];
    map_mode_to_chars((*mp).m_mode, mapchars.as_mut_ptr());
    let mapchars_ptr = mapchars.as_ptr();
    msg_puts(mapchars_ptr);

    // Count the chars written (length of NUL-terminated mapchars string)
    let mut len = libc::strlen(mapchars_ptr.cast()) as usize;

    while {
        len += 1;
        len <= 3
    } {
        msg_putchar(c_int::from(b' '));
    }

    // Display the LHS. Get length of what we write.
    len = msg_outtrans_special(m_keys, true, 0) as usize;
    loop {
        msg_putchar(c_int::from(b' ')); // pad with blanks
        len += 1;
        if len >= 12 {
            break;
        }
    }

    let noremap = (*mp).m_noremap;
    if noremap == REMAP_NONE {
        msg_puts_hl(c"*".as_ptr(), HLF_8, false);
    } else if noremap == REMAP_SCRIPT {
        msg_puts_hl(c"&".as_ptr(), HLF_8, false);
    } else {
        msg_putchar(c_int::from(b' '));
    }

    if local {
        msg_putchar(c_int::from(b'@'));
    } else {
        msg_putchar(c_int::from(b' '));
    }

    // Display RHS
    let m_luaref = (*mp).m_luaref;
    if m_luaref != LUA_NOREF {
        let str_ = nlua_funcref_str(m_luaref, std::ptr::null_mut());
        msg_puts_hl(str_, HLF_8, false);
        xfree(str_);
    } else if *m_str == 0 {
        // NUL => <Nop>
        msg_puts_hl(c"<Nop>".as_ptr(), HLF_8, false);
    } else {
        msg_outtrans_special(m_str, false, 0);
    }

    if !m_desc.is_null() {
        msg_puts(c"\n                 ".as_ptr()); // shift to same level
        msg_puts(m_desc);
    }

    if p_verbose > 0 {
        last_set_msg((*mp).m_script_ctx);
    }

    msg_clr_eos();
}

// =============================================================================
// rs_map_add
// =============================================================================

/// Allocate and initialize a new mapblock, insert into hash/abbr list.
///
/// This is the Rust replacement for the C `map_add` static function and
/// the `nvim_map_add` accessor. Uses the `MapArguments` fields for rhs,
/// rhs_lua, orig_rhs, expr, silent, nowait, replace_keycodes, and desc.
///
/// When `sid == 0`, uses the current script context (`current_sctx` + `SOURCING_LNUM`).
///
/// # Safety
/// All pointer arguments must be valid. `keys` must be a NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_map_add(
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
) -> MapblockHandle {
    // xcalloc returns *mut c_char; cast to *mut MapblockT (safe, well-aligned allocation)
    let mp = xcalloc(1, std::mem::size_of::<MapblockT>()).cast::<MapblockT>();

    // If CTRL-C has been mapped, don't always use it for Interrupting.
    if !keys.is_null() && *keys as u8 == CTRL_C {
        if is_buf_local != 0 {
            let cur = nvim_mapping_buf_get_mapped_ctrl_c(buf);
            nvim_mapping_buf_set_mapped_ctrl_c(buf, cur | mode);
        } else {
            mapped_ctrl_c |= mode;
        }
    }

    (*mp).m_keys = xstrdup(keys);
    (*mp).m_str = (*args).rhs;
    (*mp).m_orig_str = (*args).orig_rhs;
    (*mp).m_luaref = (*args).rhs_lua;
    (*mp).m_keylen = libc::strlen((*mp).m_keys.cast()) as c_int;
    (*mp).m_noremap = noremap;
    (*mp).m_nowait = c_char::from((*args).nowait);
    (*mp).m_silent = c_char::from((*args).silent);
    (*mp).m_mode = mode;
    (*mp).m_simplified = c_int::from(simplified != 0);
    (*mp).m_expr = c_char::from((*args).expr);
    (*mp).m_replace_keycodes = (*args).replace_keycodes;
    (*mp).m_desc = (*args).desc;

    if sid != 0 {
        (*mp).m_script_ctx.sc_sid = sid;
        (*mp).m_script_ctx.sc_lnum = lnum;
    } else {
        (*mp).m_script_ctx = current_sctx;
        (*mp).m_script_ctx.sc_lnum += nvim_dbg_get_sourcing_lnum() as c_int;
        nlua_set_sctx(std::ptr::addr_of_mut!((*mp).m_script_ctx));
    }

    // Add the new entry in front of the abbrlist or maphash[] list.
    if is_abbr != 0 {
        // Prepend to abbr list
        let abbr_head = if is_buf_local != 0 {
            nvim_buf_get_first_abbr(buf)
        } else {
            nvim_get_first_abbr()
        };
        (*mp).m_next = abbr_head;
        if is_buf_local != 0 {
            nvim_buf_set_first_abbr(buf, mp);
        } else {
            nvim_set_first_abbr(mp);
        }
    } else {
        // Prepend to maphash[] bucket
        let n = map_hash(mode, *(*mp).m_keys as u8);
        let bucket_head = if is_buf_local != 0 {
            nvim_buf_get_maphash_entry(buf, n)
        } else {
            nvim_get_maphash_entry(n)
        };
        (*mp).m_next = bucket_head;
        if is_buf_local != 0 {
            nvim_buf_set_maphash_entry(buf, n, mp);
        } else {
            nvim_set_maphash_entry(n, mp);
        }
    }

    mp
}

// =============================================================================
// f_hasmapto
// =============================================================================

/// "hasmapto()" VimL function.
///
/// Checks if a mapping exists with the given RHS. Arguments:
///   argvars[0]: string - the RHS to look for
///   argvars[1]: string - mode characters (default "nvo")
///   argvars[2]: number - if non-zero, check abbreviations
///
/// # Safety
/// `argvars` and `rettv` must be valid `typval_T*` pointers.
#[export_name = "f_hasmapto"]
pub unsafe extern "C" fn rs_f_hasmapto(
    argvars: *mut c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let name = tv_get_string(tv_idx(argvars, 0));
    let mut abbr: c_int = 0;
    let mut buf = [0i8; NUMBUFLEN];

    let mode: *const c_char = if tv_type(tv_idx(argvars, 1)) == VAR_UNKNOWN {
        c"nvo".as_ptr()
    } else {
        let m = tv_get_string_buf(tv_idx(argvars, 1), buf.as_mut_ptr());
        if tv_type(tv_idx(argvars, 2)) != VAR_UNKNOWN {
            abbr = tv_get_number(tv_idx(argvars, 2)) as c_int;
        }
        m
    };

    // map_to_exists is exported from lookup.rs as `map_to_exists`
    let result = crate::lookup::rs_map_to_exists_str(name, mode, abbr);
    tv_set_number(rettv, i64::from(result));
}
