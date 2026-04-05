//! Expression mapping evaluation and abbreviation checking.
//!
//! Provides `rs_eval_map_expr` (evaluate RHS expression of a mapping)
//! and `rs_check_abbr` (check for and expand abbreviations).

use std::ffi::{c_char, c_int};

use nvim_api::{Array, Error as NvimError, NvimString, Object};

use crate::MapblockHandle;

// =============================================================================
// Constants
// =============================================================================

/// LUA_NOREF: Lua reference that is not set.
const LUA_NOREF: c_int = -2;

/// kRetObject: LuaRetMode value for returning any Object.
const K_RET_OBJECT: c_int = 0;

/// kObjectTypeString: ObjectType value for String.
const K_OBJECT_TYPE_STRING: c_int = 4;

/// kErrorTypeNone: no error.
const K_ERROR_TYPE_NONE: c_int = -1;

/// REPTERM_DO_LT: replace_termcodes flag to replace <lt>.
const REPTERM_DO_LT: c_int = 2;

/// K_SPECIAL: first byte of a special key code.
const K_SPECIAL: u8 = 0x80;

/// KS_SPECIAL: second byte for escaped K_SPECIAL.
const KS_SPECIAL: u8 = 254;

/// KS_ZERO: second byte for NUL.
const KS_ZERO: u8 = 255;

/// KE_FILLER: filler byte after KS_SPECIAL, KS_ZERO.
const KE_FILLER: u8 = b'X';

/// Ctrl_RSB: ] Right Square Bracket (29).
const CTRL_RSB: c_int = 29;

/// Ctrl_H: backspace (8).
const CTRL_H: u8 = 8;

/// Ctrl_V: literal-next (22).
const CTRL_V: u8 = 22;

/// ABBR_OFF: multi-byte char offset added to avoid CTRL-V.
const ABBR_OFF: c_int = 0x100;

/// MB_MAXBYTES: max bytes in a multi-byte character.
const MB_MAXBYTES: usize = 21;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Memory
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_char);

    // vim_unescape_ks: unescape K_SPECIAL sequences (via mapping.c shim)
    fn nvim_mapping_vim_unescape_ks(s: *mut c_char);

    // Globals accessible as extern statics
    static mut expr_map_lock: c_int;
    static mut msg_col: c_int;
    static mut msg_row: c_int;

    // p_cpo accessor (file-static in options, exposed via mapping.c)
    fn nvim_mapping_get_p_cpo() -> *const c_char;

    // Cursor save/restore via C shim (curwin->w_cursor is in a complex struct)
    fn nvim_mapping_get_curwin_cursor(lnum: *mut i32, col: *mut c_int, coladd: *mut c_int);
    fn nvim_mapping_set_curwin_cursor(lnum: i32, col: c_int, coladd: c_int);

    // set v:char
    fn set_vim_var_char(c: c_int);

    // Lua call
    fn nlua_call_ref(
        ref_: c_int,
        name: *const c_char,
        args: Array,
        mode: c_int,
        arena: *mut std::ffi::c_void,
        err: *mut NvimError,
    ) -> Object;

    // api_free_object: free an Object (already exported from nvim-api crate)
    fn api_free_object(value: Object);

    // api_clear_error: clear an Error struct
    fn api_clear_error(err: *mut NvimError);

    // semsg_lua_err: emit "E5108: <msg>" via C shim
    fn nvim_mapping_semsg_lua_err(msg: *mut c_char);

    // eval_to_string: evaluate a Vimscript expression to a string
    fn eval_to_string(arg: *mut c_char, join_list: bool, use_simple_function: bool) -> *mut c_char;

    // replace_termcodes / vim_strsave_escape_ks
    fn replace_termcodes(
        from: *const c_char,
        from_len: usize,
        bufp: *mut *mut c_char,
        sid_arg: c_int,
        flags: c_int,
        did_simplify: *mut bool,
        cpo_val: *const c_char,
    ) -> *mut c_char;
    fn vim_strsave_escape_ks(p: *mut c_char) -> *mut c_char;

    // string_to_cstr: NvimString → *mut c_char (xstrndup)
    fn string_to_cstr(str_: NvimString) -> *mut c_char;

    // Abbreviation check helpers
    fn noremap_keys() -> bool;
    fn mb_prevptr(line: *mut c_char, p: *mut c_char) -> *mut c_char;
    fn vim_iswordp(p: *const c_char) -> bool;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn ins_typebuf(
        str_: *mut c_char,
        noremap: c_int,
        offset: c_int,
        nottyped: bool,
        silent: bool,
    ) -> c_int;

    // typebuf.tb_no_abbr_cnt accessors
    fn nvim_mapping_get_typebuf_no_abbr_cnt() -> c_int;
    fn nvim_mapping_add_typebuf_no_abbr_cnt(delta: c_int);
}

// =============================================================================
// Helper: ascii_isspace
// =============================================================================

/// Returns true if `c` is ASCII whitespace (space, tab, NL, FF, CR).
#[inline]
const fn ascii_isspace(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | 0x0c | b'\r')
}

// =============================================================================
// Helper: k_second / k_third
// =============================================================================

/// Get second byte of K_SPECIAL three-byte sequence.
/// Matches C macro: K_SECOND(c).
#[inline]
fn k_second(c: c_int) -> u8 {
    if c == c_int::from(K_SPECIAL) {
        KS_SPECIAL
    } else if c == 0 {
        KS_ZERO
    } else {
        // KEY2TERMCAP0(c): ((-c) & 0xff)
        ((-c) & 0xff) as u8
    }
}

/// Get third byte of K_SPECIAL three-byte sequence.
/// Matches C macro: K_THIRD(c).
#[inline]
fn k_third(c: c_int) -> u8 {
    if c == c_int::from(K_SPECIAL) || c == 0 {
        KE_FILLER
    } else {
        // KEY2TERMCAP1(c): (((unsigned)(-c) >> 8) & 0xff)
        (((-c) as u32 >> 8) & 0xff) as u8
    }
}

// =============================================================================
// rs_eval_map_expr
// =============================================================================

/// Evaluate the RHS of a mapping or abbreviation and escape special chars.
///
/// This is the Rust replacement for `eval_map_expr` in mapping.c.
/// Careful: after this `mp` may be invalid if the mapping was deleted.
///
/// @param mp  The mapblock to evaluate.
/// @param c   NUL or typed character for abbreviation.
///
/// # Safety
/// `mp` must be a valid non-null mapblock pointer.
#[export_name = "eval_map_expr"]
pub unsafe extern "C" fn rs_eval_map_expr(mp: MapblockHandle, c: c_int) -> *mut c_char {
    let mut expr: *mut c_char = std::ptr::null_mut();

    // Remove escaping of K_SPECIAL, because "str" is in typeahead format.
    let m_luaref = (*mp).m_luaref;
    if m_luaref == LUA_NOREF {
        expr = xstrdup((*mp).m_str);
        nvim_mapping_vim_unescape_ks(expr);
    }

    let replace_keycodes = (*mp).m_replace_keycodes;

    // Forbid changing text or using ":normal" to avoid bad side effects.
    // Also restore the cursor position.
    expr_map_lock += 1;
    set_vim_var_char(c);

    // Save cursor position and message state
    let mut save_lnum: i32 = 0;
    let mut save_col: c_int = 0;
    let mut save_coladd: c_int = 0;
    nvim_mapping_get_curwin_cursor(
        std::ptr::addr_of_mut!(save_lnum),
        std::ptr::addr_of_mut!(save_col),
        std::ptr::addr_of_mut!(save_coladd),
    );
    let save_msg_col = msg_col;
    let save_msg_row = msg_row;

    let mut p: *mut c_char = std::ptr::null_mut();

    if m_luaref == LUA_NOREF {
        p = eval_to_string(expr, false, false);
        xfree(expr);
    } else {
        let mut err = NvimError {
            err_type: K_ERROR_TYPE_NONE,
            msg: std::ptr::null_mut(),
        };
        let args = Array {
            size: 0,
            capacity: 0,
            items: std::ptr::null_mut(),
        };
        let ret = nlua_call_ref(
            m_luaref,
            std::ptr::null(),
            args,
            K_RET_OBJECT,
            std::ptr::null_mut(),
            std::ptr::addr_of_mut!(err),
        );
        if ret.obj_type == K_OBJECT_TYPE_STRING {
            p = string_to_cstr(ret.data.string);
        }
        api_free_object(ret);
        if err.err_type != K_ERROR_TYPE_NONE {
            nvim_mapping_semsg_lua_err(err.msg);
            api_clear_error(std::ptr::addr_of_mut!(err));
        }
    }

    expr_map_lock -= 1;
    nvim_mapping_set_curwin_cursor(save_lnum, save_col, save_coladd);
    msg_col = save_msg_col;
    msg_row = save_msg_row;

    if p.is_null() {
        return std::ptr::null_mut();
    }

    let mut res: *mut c_char = std::ptr::null_mut();

    if replace_keycodes {
        replace_termcodes(
            p,
            libc::strlen(p.cast()),
            std::ptr::addr_of_mut!(res),
            0,
            REPTERM_DO_LT,
            std::ptr::null_mut(),
            nvim_mapping_get_p_cpo(),
        );
    } else {
        // Escape K_SPECIAL in the result to be able to use as typeahead.
        res = vim_strsave_escape_ks(p);
    }
    xfree(p);

    res
}

// =============================================================================
// rs_check_abbr
// =============================================================================

/// Check for an abbreviation. Cursor is at ptr[col].
///
/// When inserting, mincol is where insert started.
/// For the command line, mincol is what is to be skipped over.
/// "c" is the character typed before check_abbr was called. It may have
/// ABBR_OFF added to avoid prepending a CTRL-V to it.
///
/// Returns true if there is an abbreviation, false if not.
///
/// # Safety
/// `ptr` must be a valid pointer to at least `col` bytes.
#[allow(clippy::too_many_lines)]
#[export_name = "check_abbr"]
pub unsafe extern "C" fn rs_check_abbr(
    c: c_int,
    ptr: *mut c_char,
    col: c_int,
    mincol: c_int,
) -> bool {
    let mut tb = [0u8; MB_MAXBYTES + 4];
    let mut clen: c_int;

    if nvim_mapping_get_typebuf_no_abbr_cnt() > 0 {
        // abbrev. are not recursive
        return false;
    }

    // no remapping implies no abbreviation, except for CTRL-]
    if noremap_keys() && c != CTRL_RSB {
        return false;
    }

    // Check for word before the cursor
    if col == 0 {
        // cannot be an abbr.
        return false;
    }

    let scol: c_int;

    {
        let mut is_id = true;
        let vim_abbr: bool;
        let mut p = mb_prevptr(ptr, ptr.add(col as usize));
        if vim_iswordp(p) {
            vim_abbr = false; // vi compatible abbr.
            if p > ptr {
                is_id = vim_iswordp(mb_prevptr(ptr, p));
            }
        } else {
            vim_abbr = true; // Vim added abbr.
        }
        clen = 1;
        while p > ptr.add(mincol as usize) {
            p = mb_prevptr(ptr, p);
            if ascii_isspace(*p as u8) || (!vim_abbr && is_id != vim_iswordp(p)) {
                p = p.add(utfc_ptr2len(p) as usize);
                break;
            }
            clen += 1;
        }
        scol = p.offset_from(ptr) as c_int;
    }

    let scol = scol.max(mincol);

    if scol < col {
        // there is a word in front of the cursor
        let word_ptr = ptr.add(scol as usize);
        let len = col - scol;
        let mp = crate::exmap::rs_find_matching_abbr(word_ptr, len);
        if !mp.is_null() {
            // Found a match.
            // Insert the rest of the abbreviation in typebuf.tb_buf[].
            let mut j: usize = 0;

            // Bind c as mut so we can modify it below
            let mut c = c;

            if c != CTRL_RSB {
                // special key code, split up
                if c < 0 || c == c_int::from(K_SPECIAL) {
                    // IS_SPECIAL(c) || c == K_SPECIAL
                    tb[j] = K_SPECIAL;
                    j += 1;
                    tb[j] = k_second(c);
                    j += 1;
                    tb[j] = k_third(c);
                    j += 1;
                } else {
                    if c < ABBR_OFF && !(0x20..=0x7e).contains(&c) {
                        tb[j] = CTRL_V; // special char needs CTRL-V
                        j += 1;
                    }
                    // if ABBR_OFF has been added, remove it here
                    if c >= ABBR_OFF {
                        c -= ABBR_OFF;
                    }
                    let newlen =
                        utf_char2bytes(c, tb.as_mut_ptr().add(j).cast::<c_char>()) as usize;
                    tb[j + newlen] = 0;
                    // Need to escape K_SPECIAL
                    let escaped = vim_strsave_escape_ks(tb.as_mut_ptr().add(j).cast::<c_char>());
                    if !escaped.is_null() {
                        let esc_len = libc::strlen(escaped.cast());
                        std::ptr::copy_nonoverlapping(
                            escaped.cast::<u8>(),
                            tb.as_mut_ptr().add(j),
                            esc_len,
                        );
                        j += esc_len;
                        xfree(escaped);
                    }
                }
                tb[j] = 0;
                // insert the last typed char
                ins_typebuf(
                    tb.as_mut_ptr().cast::<c_char>(),
                    1,
                    0,
                    true,
                    (*mp).m_silent != 0,
                );
            }

            // copy values here: calling eval_map_expr() may make mp invalid!
            let noremap = (*mp).m_noremap;
            let silent = (*mp).m_silent != 0;
            let expr = (*mp).m_expr != 0;

            let s: *mut c_char = if expr {
                rs_eval_map_expr(mp, c)
            } else {
                (*mp).m_str
            };

            if !s.is_null() {
                // insert the to-string
                ins_typebuf(s, noremap, 0, true, silent);
                // no abbrev. for these chars
                let slen = libc::strlen(s.cast()) as c_int;
                nvim_mapping_add_typebuf_no_abbr_cnt(slen + j as c_int + 1);
                if expr {
                    xfree(s);
                }
            }

            tb[0] = CTRL_H;
            tb[1] = 0;
            let mut del = clen; // Delete characters instead of bytes
            while del > 0 {
                ins_typebuf(tb.as_mut_ptr().cast::<c_char>(), 1, 0, true, silent);
                del -= 1;
            }
            return true;
        }
    }

    false
}
