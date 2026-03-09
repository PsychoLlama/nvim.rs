//! Mapping lookup and translation functions.
//!
//! Provides `check_map` (find mapping for given keys), `map_to_exists`
//! (check if mapping RHS exists), and `translate_mapping` (convert internal
//! key representation to display form).

use std::ffi::{c_char, c_int};

use crate::{
    mapblock_keylen, mapblock_keys, mapblock_luaref, mapblock_mode, mapblock_next, mapblock_str,
    BufHandle, MapblockHandle,
};

// =============================================================================
// Constants
// =============================================================================

const LUA_NOREF: c_int = -2;
const K_SPECIAL: u8 = 0x80;
const KS_MODIFIER: u8 = 252;
const KS_SPECIAL: u8 = 254;
const KS_ZERO: u8 = 255;
const KE_FILLER: u8 = b'X';
const REPTERM_DO_LT: c_int = 2;
const CPO_BSLASH: u8 = b'B';
const CTRL_J: u8 = 10;
const CTRL_V: u8 = 22;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    fn nvim_get_maphash_entry(index: c_int) -> MapblockHandle;
    fn nvim_get_first_abbr() -> MapblockHandle;
    fn nvim_buf_get_maphash_entry(buf: BufHandle, index: c_int) -> MapblockHandle;
    fn nvim_buf_get_first_abbr(buf: BufHandle) -> MapblockHandle;
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_mapping_get_p_cpo() -> *const c_char;
    #[link_name = "vim_strchr"]
    fn rs_vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn replace_termcodes(
        from: *const c_char,
        from_len: usize,
        bufp: *mut *mut c_char,
        sid_arg: c_int,
        flags: c_int,
        did_simplify: *mut bool,
        cpo_val: *const c_char,
    ) -> *mut c_char;
    fn xfree(ptr: *mut c_char);
    fn get_special_key_name(c: c_int, modifiers: c_int) -> *mut c_char;
    fn xstrdup(s: *const c_char) -> *mut c_char;
}

// =============================================================================
// Key encoding helpers
// =============================================================================

/// `TERMCAP2KEY(a, b)` — encode a termcap pair as a negative key code.
#[inline]
const fn termcap2key(a: u8, b: u8) -> c_int {
    -(a as c_int + ((b as c_int) << 8))
}

/// `TO_SPECIAL(a, b)` — decode K_SPECIAL sequence bytes to key code.
#[inline]
const fn to_special(a: u8, b: u8) -> c_int {
    if a == KS_SPECIAL {
        K_SPECIAL as c_int
    } else if a == KS_ZERO {
        termcap2key(KS_ZERO, KE_FILLER)
    } else {
        termcap2key(a, b)
    }
}

/// `IS_SPECIAL(c)` — true when c is a special key code (negative value).
#[inline]
const fn is_special(c: c_int) -> bool {
    c < 0
}

// =============================================================================
// check_map
// =============================================================================

/// Find a mapping for the given keys.
///
/// Searches both buffer-local and global mapping/abbreviation tables.
/// Returns the RHS string of the first matching mapping, or NULL.
///
/// # Safety
/// All pointer parameters must be valid. `keys` must be NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn rs_check_map(
    keys: *mut c_char,
    mode: c_int,
    exact: c_int,
    ign_mod: c_int,
    abbr: c_int,
    mp_ptr: *mut MapblockHandle,
    local_ptr: *mut c_int,
    rhs_lua: *mut c_int,
) -> *mut c_char {
    *rhs_lua = LUA_NOREF;

    let len = libc::strlen(keys.cast()) as c_int;
    let curbuf = nvim_get_curbuf();

    // local=1 (buffer-local first), then local=0 (global)
    for local in (0..=1).rev() {
        for hash in 0..256 {
            let mp = if abbr != 0 {
                if hash > 0 {
                    break; // abbreviations: only one list
                }
                if local != 0 {
                    nvim_buf_get_first_abbr(curbuf)
                } else {
                    nvim_get_first_abbr()
                }
            } else if local != 0 {
                nvim_buf_get_maphash_entry(curbuf, hash)
            } else {
                nvim_get_maphash_entry(hash)
            };

            let mut cur = mp;
            while !cur.is_null() {
                let mp_mode = mapblock_mode(cur);
                let mp_keylen = mapblock_keylen(cur);

                if (mp_mode & mode) != 0 && (exact == 0 || mp_keylen == len) {
                    let mut s = mapblock_keys(cur);
                    let mut keylen = mp_keylen;

                    if ign_mod != 0
                        && keylen >= 3
                        && *s as u8 == K_SPECIAL
                        && *s.add(1) as u8 == KS_MODIFIER
                    {
                        s = s.add(3);
                        keylen -= 3;
                    }

                    let minlen = if keylen < len { keylen } else { len };
                    if libc::strncmp(s, keys, minlen as usize) == 0 {
                        if !mp_ptr.is_null() {
                            *mp_ptr = cur;
                        }
                        if !local_ptr.is_null() {
                            *local_ptr = local;
                        }
                        let luaref = mapblock_luaref(cur);
                        *rhs_lua = luaref;
                        return if luaref == LUA_NOREF {
                            mapblock_str(cur).cast_mut()
                        } else {
                            std::ptr::null_mut()
                        };
                    }
                }
                cur = mapblock_next(cur);
            }
        }
    }

    std::ptr::null_mut()
}

// =============================================================================
// map_to_exists
// =============================================================================

/// Check if a mapping exists for the given string and mode characters.
///
/// Calls `replace_termcodes` on the input, converts mode characters to mode
/// bits, and delegates to `rs_map_to_exists_mode`.
///
/// # Safety
/// `str` and `modechars` must be valid NUL-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_map_to_exists_str(
    str: *const c_char,
    modechars: *const c_char,
    abbr: c_int,
) -> c_int {
    let p_cpo = nvim_mapping_get_p_cpo();

    let mut buf: *mut c_char = std::ptr::null_mut();
    let rhs = replace_termcodes(
        str,
        libc::strlen(str.cast()),
        std::ptr::addr_of_mut!(buf),
        0,
        REPTERM_DO_LT,
        std::ptr::null_mut(),
        p_cpo,
    );

    let mut mode: c_int = 0;

    // Parse mode characters
    macro_rules! mapmode {
        ($chr:expr, $flags:expr) => {
            if !libc::strchr(modechars.cast(), c_int::from($chr)).is_null() {
                mode |= $flags;
            }
        };
    }
    mapmode!(b'n', crate::MODE_NORMAL);
    mapmode!(b'v', crate::MODE_VISUAL | crate::MODE_SELECT);
    mapmode!(b'x', crate::MODE_VISUAL);
    mapmode!(b's', crate::MODE_SELECT);
    mapmode!(b'o', crate::MODE_OP_PENDING);
    mapmode!(b'i', crate::MODE_INSERT);
    mapmode!(b'l', crate::MODE_LANGMAP);
    mapmode!(b'c', crate::MODE_CMDLINE);

    let retval = crate::rs_map_to_exists_mode(rhs, mode, abbr);
    xfree(buf);

    retval
}

// =============================================================================
// translate_mapping
// =============================================================================

/// Translate an internal mapping key representation to display form.
///
/// Converts special key sequences (like K_SPECIAL bytes) into human-readable
/// `<Key>` notation. The caller must free the returned string with `xfree`.
///
/// # Safety
/// `str_in` and `cpo_val` must be valid NUL-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_translate_mapping(
    str_in: *const c_char,
    cpo_val: *const c_char,
) -> *mut c_char {
    let mut result = Vec::with_capacity(40);
    let mut str = str_in.cast::<u8>();

    let cpo_bslash = !rs_vim_strchr(cpo_val, c_int::from(CPO_BSLASH)).is_null();

    while *str != 0 {
        let mut c = c_int::from(*str);

        if c == c_int::from(K_SPECIAL) && *str.add(1) != 0 && *str.add(2) != 0 {
            let mut modifiers: c_int = 0;

            if *str.add(1) == KS_MODIFIER {
                str = str.add(1);
                str = str.add(1);
                modifiers = c_int::from(*str);
                str = str.add(1);
                c = c_int::from(*str);
            }

            if c == c_int::from(K_SPECIAL) && *str.add(1) != 0 && *str.add(2) != 0 {
                c = to_special(*str.add(1), *str.add(2));
                if c == termcap2key(KS_ZERO, KE_FILLER) {
                    // display <Nul> as ^@
                    c = 0;
                }
                str = str.add(2);
            }

            if is_special(c) || modifiers != 0 {
                // Special key — get its name from the existing Rust function
                let name = get_special_key_name(c, modifiers);
                if !name.is_null() {
                    let mut p = name.cast::<u8>();
                    while *p != 0 {
                        result.push(*p);
                        p = p.add(1);
                    }
                }
                str = str.add(1);
                continue;
            }
        }

        let c_u8 = c as u8;
        if c_u8 == b' '
            || c_u8 == b'\t'
            || c_u8 == CTRL_J
            || c_u8 == CTRL_V
            || c_u8 == b'<'
            || (c_u8 == b'\\' && !cpo_bslash)
        {
            result.push(if cpo_bslash { CTRL_V } else { b'\\' });
        }

        if c != 0 {
            result.push(c_u8);
        }

        str = str.add(1);
    }

    // NUL-terminate and return as allocated C string
    result.push(0);
    xstrdup(result.as_ptr().cast::<c_char>())
}
