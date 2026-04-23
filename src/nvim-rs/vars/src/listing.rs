//! Variable listing and completion functions for VimL.
//!
//! Phase 3: Migrated from `src/nvim/eval/vars.c`.
//!
//! Functions:
//! - `rs_cat_prefix_varname`: concatenate scope prefix with variable name
//! - `rs_get_user_var_name`: enumerate variable names for tab-completion
//! - `rs_var_redir_str`: append to redirect buffer
//! - `rs_list_hashtable_vars`: list variables in a hashtab with prefix
//! - `rs_list_one_var`: list one variable value
//! - `rs_list_one_var_a`: format and display one variable

#![allow(unsafe_op_in_unsafe_fn)]
#![allow(dead_code)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ref_as_ptr)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::ptr_cast_constness)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::if_not_else)]
#![allow(clippy::ptr_eq)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// VarType constants
// =============================================================================
const VAR_NUMBER: c_int = 1;
const VAR_FUNC: c_int = 3;
const VAR_LIST: c_int = 4;
const VAR_DICT: c_int = 5;
const VAR_STRING: c_int = 2;
const VAR_PARTIAL: c_int = 9;

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    // --- memory ---
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn strlen(s: *const c_char) -> usize;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;

    // --- hashtab iteration ---
    fn nvim_vars_ht_get_used(ht: *mut c_void) -> usize;
    fn nvim_vars_ht_get_array(ht: *mut c_void) -> *mut c_void;
    fn nvim_hashitem_advance(hi: *mut c_void) -> *mut c_void;
    fn nvim_vars_hashitem_get_key(hi: *mut c_void) -> *const c_char;
    fn nvim_hashitem_empty(hi: *mut c_void) -> c_int;
    fn nvim_hi2dictitem(hi: *mut c_void) -> *mut c_void;
    fn nvim_vars_got_int() -> bool;

    // --- dictitem/typval access ---
    fn nvim_dictitem_get_tv(di: *mut c_void) -> *mut c_void;
    fn nvim_vars_dictitem_get_key(di: *mut c_void) -> *const c_char;
    fn nvim_tv_get_type(tv: *mut c_void) -> c_int;
    fn nvim_tv_get_string_val(tv: *mut c_void) -> *mut c_char;
    fn nvim_encode_tv2echo(tv: *mut c_void) -> *mut c_char;

    // --- hashtabs ---
    fn nvim_get_globvarht_ptr() -> *mut c_void;
    fn nvim_prevwin_curwin_buf_vars_ht() -> *mut c_void;
    fn nvim_prevwin_curwin_win_vars_ht() -> *mut c_void;
    fn nvim_curtab_tp_vars_ht() -> *mut c_void;
    fn nvim_vimvars_array_size() -> c_int;
    fn nvim_get_vim_var_name(idx: c_int) -> *const c_char;

    // --- var name buffer ---
    fn nvim_get_varnamebuf_ptr() -> *mut *mut c_char;
    fn nvim_get_varnamebuflen_ptr() -> *mut usize;
    fn nvim_strcpy(dst: *mut c_char, src: *const c_char);

    // --- completion expand ---
    fn nvim_vars_xp_get_pattern(xp: *const c_void) -> *const c_char;

    // --- message system ---
    fn msg_ext_set_kind(kind: *const c_char);
    fn msg_start();
    fn msg_putchar(c: c_int);
    fn msg_puts(s: *const c_char);
    fn msg_puts_len(s: *const c_char, len: isize, attr: c_int, clip: bool);
    fn msg_advance(col: c_int);
    fn msg_outtrans(str_: *const c_char, attr: c_int, stop_at_highlighting: bool);
    fn msg_clr_eos();

    // --- message filter ---
    fn nvim_message_filtered(buf: *const c_char) -> bool;

    // --- string ops ---
    fn nvim_xstrlcpy_iosize(dst: *mut c_char, src: *const c_char);
    fn nvim_xstrlcat_iosize(dst: *mut c_char, src: *const c_char);
    fn nvim_get_iosize() -> c_int;

    // --- redirect ---
    fn nvim_redir_ga_grow(len: c_int);
    fn nvim_redir_ga_append(value: *const c_char, len: c_int);
    fn nvim_get_redir_lval() -> *mut c_void;

    // --- list_arg_vars additions ---
    fn semsg(fmt: *const c_char, ...) -> c_int;
    // xfree already declared above
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn nvim_ends_excmd_char(c: c_int) -> bool;
    fn rs_find_name_end(
        arg: *const c_char,
        expr_start: *mut *const c_char,
        expr_end: *mut *const c_char,
        flags: c_int,
    ) -> *const c_char;
    fn nvim_emsg_severe_set();
    fn nvim_get_name_len(
        arg: *mut *const c_char,
        tofree: *mut *mut c_char,
        evaluate: bool,
        verbose: bool,
    ) -> c_int;
    fn nvim_aborting() -> bool;
    fn nvim_vars_eval_variable_full(
        name: *const c_char,
        len: c_int,
        rettv: *mut c_void,
        dip: *mut *mut c_void,
        verbose: bool,
        no_autoload: bool,
    ) -> c_int;
    fn nvim_vars_handle_subscript_listarg(arg: *mut *const c_char, tv: *mut c_void) -> c_int;
    fn tv_clear(tv: *mut c_void);
    fn nvim_list_buf_vars(first: *mut c_int);
    fn nvim_list_win_vars(first: *mut c_int);
    fn nvim_list_tab_vars(first: *mut c_int);
    fn nvim_list_vim_vars(first: *mut c_int);
    fn nvim_list_script_vars(first: *mut c_int);
    fn nvim_list_func_vars(first: *mut c_int);
    fn rs_ascii_iswhite(c: c_int) -> c_int;
    fn nvim_eap_get_skip_val(eap: *const c_void) -> c_int;
    fn nvim_e738_cant_list() -> *const c_char;
    fn nvim_e_invarg2() -> *const c_char;
    fn nvim_e_trailing_arg() -> *const c_char;

    // --- del_menutrans_vars ---
    fn hash_lock(ht: *mut c_void);
    fn hash_unlock(ht: *mut c_void);
    fn nvim_vars_delete_var(ht: *mut c_void, hi: *mut c_void);
    // nvim_get_globvarht_ptr and xfree already declared above
}

// =============================================================================
// cat_prefix_varname and get_user_var_name
// =============================================================================

/// Concatenate a scope prefix char with a variable name.
///
/// Matches C `cat_prefix_varname`. Uses a static buffer managed via C accessors.
///
/// # Safety
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cat_prefix_varname(prefix: c_int, name: *const c_char) -> *mut c_char {
    let len = strlen(name) + 3;

    let varnamebuf_ptr = nvim_get_varnamebuf_ptr();
    let varnamebuflen_ptr = nvim_get_varnamebuflen_ptr();

    if len > *varnamebuflen_ptr {
        xfree(*varnamebuf_ptr as *mut c_void);
        let new_len = len + 10;
        *varnamebuf_ptr = xmalloc(new_len) as *mut c_char;
        *varnamebuflen_ptr = new_len;
    }

    let buf = *varnamebuf_ptr;
    *buf = prefix as c_char;
    *buf.add(1) = b':' as c_char;
    nvim_strcpy(buf.add(2), name);
    buf
}

/// Enumerate variable names for tab-completion.
///
/// Matches C `get_user_var_name`. Uses static counters (managed here with
/// static mut since this is single-threaded Neovim).
///
/// # Safety
/// `xp` must be a valid `expand_T*`. `idx` must be monotonically increasing
/// from 0.
#[no_mangle]
pub unsafe extern "C" fn rs_get_user_var_name(xp: *const c_void, idx: c_int) -> *mut c_char {
    static mut GDONE: usize = 0;
    static mut BDONE: usize = 0;
    static mut WDONE: usize = 0;
    static mut TDONE: usize = 0;
    static mut VIDX: usize = 0;
    static mut HI: *mut c_void = std::ptr::null_mut();

    if idx == 0 {
        GDONE = 0;
        BDONE = 0;
        WDONE = 0;
        VIDX = 0;
        TDONE = 0;
    }

    // Global variables
    let globvarht = nvim_get_globvarht_ptr();
    let globvarht_used = nvim_vars_ht_get_used(globvarht);
    if GDONE < globvarht_used {
        if GDONE == 0 {
            HI = nvim_vars_ht_get_array(globvarht);
        } else {
            HI = nvim_hashitem_advance(HI);
        }
        while nvim_hashitem_empty(HI) != 0 {
            HI = nvim_hashitem_advance(HI);
        }
        GDONE += 1;
        let xp_pattern = nvim_vars_xp_get_pattern(xp);
        let g_prefix = b"g:\0".as_ptr() as *const c_char;
        let hi_key = nvim_vars_hashitem_get_key(HI);
        if strncmp(g_prefix, xp_pattern, 2) == 0 {
            return rs_cat_prefix_varname(b'g' as c_int, hi_key);
        }
        return hi_key as *mut c_char;
    }

    // b: variables (prevwin_curwin buffer)
    let buf_ht = nvim_prevwin_curwin_buf_vars_ht();
    let buf_ht_used = nvim_vars_ht_get_used(buf_ht);
    if BDONE < buf_ht_used {
        if BDONE == 0 {
            HI = nvim_vars_ht_get_array(buf_ht);
        } else {
            HI = nvim_hashitem_advance(HI);
        }
        while nvim_hashitem_empty(HI) != 0 {
            HI = nvim_hashitem_advance(HI);
        }
        BDONE += 1;
        return rs_cat_prefix_varname(b'b' as c_int, nvim_vars_hashitem_get_key(HI));
    }

    // w: variables (prevwin_curwin)
    let win_ht = nvim_prevwin_curwin_win_vars_ht();
    let win_ht_used = nvim_vars_ht_get_used(win_ht);
    if WDONE < win_ht_used {
        if WDONE == 0 {
            HI = nvim_vars_ht_get_array(win_ht);
        } else {
            HI = nvim_hashitem_advance(HI);
        }
        while nvim_hashitem_empty(HI) != 0 {
            HI = nvim_hashitem_advance(HI);
        }
        WDONE += 1;
        return rs_cat_prefix_varname(b'w' as c_int, nvim_vars_hashitem_get_key(HI));
    }

    // t: variables
    let tab_ht = nvim_curtab_tp_vars_ht();
    let tab_ht_used = nvim_vars_ht_get_used(tab_ht);
    if TDONE < tab_ht_used {
        if TDONE == 0 {
            HI = nvim_vars_ht_get_array(tab_ht);
        } else {
            HI = nvim_hashitem_advance(HI);
        }
        while nvim_hashitem_empty(HI) != 0 {
            HI = nvim_hashitem_advance(HI);
        }
        TDONE += 1;
        return rs_cat_prefix_varname(b't' as c_int, nvim_vars_hashitem_get_key(HI));
    }

    // v: variables
    let vimvars_size = nvim_vimvars_array_size() as usize;
    if VIDX < vimvars_size {
        let name = nvim_get_vim_var_name(VIDX as c_int);
        VIDX += 1;
        return rs_cat_prefix_varname(b'v' as c_int, name);
    }

    // Done: free varnamebuf
    let varnamebuf_ptr = nvim_get_varnamebuf_ptr();
    let varnamebuflen_ptr = nvim_get_varnamebuflen_ptr();
    xfree(*varnamebuf_ptr as *mut c_void);
    *varnamebuf_ptr = std::ptr::null_mut();
    *varnamebuflen_ptr = 0;
    std::ptr::null_mut()
}

// =============================================================================
// var_redir_str
// =============================================================================

/// Append to the redirect buffer.
///
/// Matches C `var_redir_str`.
///
/// # Safety
/// `value` must be valid for `value_len` bytes (or null-terminated if value_len == -1).
#[no_mangle]
pub unsafe extern "C" fn rs_var_redir_str(value: *const c_char, value_len: c_int) {
    if nvim_get_redir_lval().is_null() {
        return;
    }

    let len: c_int = if value_len == -1 {
        strlen(value) as c_int
    } else {
        value_len
    };

    nvim_redir_ga_grow(len);
    nvim_redir_ga_append(value, len);
}

// =============================================================================
// list_one_var_a, list_one_var, list_hashtable_vars
// =============================================================================

/// Format and display one variable.
///
/// Matches C `list_one_var_a`.
///
/// # Safety
/// All pointer arguments must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_list_one_var_a(
    prefix: *const c_char,
    name: *const c_char,
    name_len: isize,
    var_type: c_int,
    string: *const c_char,
    first: *mut c_int,
) {
    if *first != 0 {
        let kind = b"list_cmd\0".as_ptr() as *const c_char;
        msg_ext_set_kind(kind);
        msg_start();
    } else {
        msg_putchar(b'\n' as c_int);
    }

    // don't use msg() to avoid overwriting "v:statusmsg"
    let first_prefix = *prefix as u8;
    if first_prefix != 0 {
        msg_puts(prefix);
    }
    if !name.is_null() {
        msg_puts_len(name, name_len, 0, false);
    }
    msg_putchar(b' ' as c_int);
    msg_advance(22);

    let mut s = string;

    if var_type == VAR_NUMBER {
        msg_putchar(b'#' as c_int);
    } else if var_type == VAR_FUNC || var_type == VAR_PARTIAL {
        msg_putchar(b'*' as c_int);
    } else if var_type == VAR_LIST {
        msg_putchar(b'[' as c_int);
        if !s.is_null() && *s as u8 == b'[' {
            s = s.add(1);
        }
    } else if var_type == VAR_DICT {
        msg_putchar(b'{' as c_int);
        if !s.is_null() && *s as u8 == b'{' {
            s = s.add(1);
        }
    } else {
        msg_putchar(b' ' as c_int);
    }

    if s.is_null() {
        let empty = b"\0".as_ptr() as *const c_char;
        msg_outtrans(empty, 0, false);
    } else {
        msg_outtrans(s, 0, false);
    }

    if var_type == VAR_FUNC || var_type == VAR_PARTIAL {
        let parens = b"()\0".as_ptr() as *const c_char;
        msg_puts(parens);
    }

    if *first != 0 {
        msg_clr_eos();
        *first = 0;
    }
}

/// List one variable value.
///
/// Matches C `list_one_var`.
///
/// # Safety
/// `v` must be a valid `dictitem_T*`, `prefix` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_list_one_var(v: *mut c_void, prefix: *const c_char, first: *mut c_int) {
    let tv_ptr = nvim_dictitem_get_tv(v);
    let s = nvim_encode_tv2echo(tv_ptr);

    // get di_key: it's at the start of dictitem_T (opaque struct, but we have a key accessor)
    let di_key = nvim_vars_dictitem_get_key(v);
    let key_len = strlen(di_key) as isize;
    let var_type = nvim_tv_get_type(tv_ptr);

    let string_arg: *const c_char = if s.is_null() {
        b"\0".as_ptr() as *const c_char
    } else {
        s
    };

    rs_list_one_var_a(prefix, di_key, key_len, var_type, string_arg, first);
    xfree(s as *mut c_void);
}

/// List variables in a hashtab with prefix.
///
/// Matches C `list_hashtable_vars`.
///
/// # Safety
/// `ht` must be a valid `hashtab_T*`, `prefix` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_list_hashtable_vars(
    ht: *mut c_void,
    prefix: *const c_char,
    empty: c_int,
    first: *mut c_int,
) {
    let iosize = nvim_get_iosize() as usize;
    // Allocate a buffer for filtering
    let buf = xmalloc(iosize) as *mut c_char;

    let mut todo = nvim_vars_ht_get_used(ht) as isize;
    let mut hi = nvim_vars_ht_get_array(ht);

    while todo > 0 && !nvim_vars_got_int() {
        if nvim_hashitem_empty(hi) == 0 {
            todo -= 1;
            let di = nvim_hi2dictitem(hi);

            // Build "prefix + di_key" into buf for filtering
            nvim_xstrlcpy_iosize(buf, prefix);
            nvim_xstrlcat_iosize(buf, nvim_vars_hashitem_get_key(hi));

            if !nvim_message_filtered(buf) {
                let tv_ptr = nvim_dictitem_get_tv(di);
                let tv_type = nvim_tv_get_type(tv_ptr);
                if empty != 0 || tv_type != VAR_STRING || !nvim_tv_get_string_val(tv_ptr).is_null()
                {
                    rs_list_one_var(di, prefix, first);
                }
            }
        }
        hi = nvim_hashitem_advance(hi);
    }

    xfree(buf as *mut c_void);
}

// =============================================================================
// Phase 12: list_arg_vars and del_menutrans_vars
// =============================================================================

// Constants
const TYPVAL_SIZE: usize = 24;
const FNE_INCL_BR: c_int = 1;
const FNE_CHECK_START: c_int = 2;
const OK: c_int = 1;
const FAIL: c_int = 0;

/// List variables named in `:let` args.
///
/// Matches C `list_arg_vars`. Returns pointer past last consumed char.
///
/// # Safety
/// `eap` must be a valid exarg_T pointer or null.
/// `arg` must be a valid C string.
/// `first` must be a valid int pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_list_arg_vars(
    eap: *const c_void,
    arg: *const c_char,
    first: *mut c_int,
) -> *const c_char {
    let mut arg = arg;
    let mut error = false;

    while !nvim_ends_excmd_char(*arg as c_int) && !nvim_vars_got_int() {
        if error || nvim_eap_get_skip_val(eap) != 0 {
            arg = rs_find_name_end(
                arg,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                FNE_INCL_BR | FNE_CHECK_START,
            );
            if rs_ascii_iswhite(*arg as c_int) == 0 && !nvim_ends_excmd_char(*arg as c_int) {
                nvim_emsg_severe_set();
                semsg(nvim_e_trailing_arg(), arg);
                break;
            }
        } else {
            let name_start = arg;
            let mut tofree: *mut c_char = std::ptr::null_mut();
            let len = nvim_get_name_len(
                std::ptr::addr_of_mut!(arg),
                std::ptr::addr_of_mut!(tofree),
                true,
                true,
            );

            if len <= 0 {
                if len < 0 && !nvim_aborting() {
                    nvim_emsg_severe_set();
                    semsg(nvim_e_invarg2(), arg);
                    if !tofree.is_null() {
                        xfree(tofree as *mut c_void);
                    }
                    break;
                }
                error = true;
            } else {
                // name is tofree if non-null, else arg (before get_name_len advanced it)
                let name: *const c_char = if !tofree.is_null() {
                    tofree
                } else {
                    name_start
                };

                let mut tv_buf = [0u8; TYPVAL_SIZE];
                let tv = tv_buf.as_mut_ptr() as *mut c_void;

                if nvim_vars_eval_variable_full(name, len, tv, std::ptr::null_mut(), true, false)
                    == FAIL
                {
                    error = true;
                } else {
                    let arg_subsc = arg;
                    if nvim_vars_handle_subscript_listarg(std::ptr::addr_of_mut!(arg), tv) == FAIL {
                        error = true;
                    } else {
                        if arg == arg_subsc && len == 2 && *name.add(1) == b':' as c_char {
                            // Scope prefix only (e.g. "g:") - list all vars in that scope
                            match *name as u8 {
                                b'g' => rs_list_hashtable_vars(
                                    nvim_get_globvarht_ptr(),
                                    b"\0".as_ptr() as *const c_char,
                                    1,
                                    first,
                                ),
                                b'b' => nvim_list_buf_vars(first),
                                b'w' => nvim_list_win_vars(first),
                                b't' => nvim_list_tab_vars(first),
                                b'v' => nvim_list_vim_vars(first),
                                b's' => nvim_list_script_vars(first),
                                b'l' => nvim_list_func_vars(first),
                                _ => {
                                    semsg(nvim_e738_cant_list(), name);
                                }
                            }
                        } else {
                            let s = nvim_encode_tv2echo(tv);
                            let used_name: *const c_char =
                                if arg == arg_subsc { name } else { name_start };
                            let name_size: isize = if used_name == tofree as *const c_char {
                                strlen(used_name) as isize
                            } else {
                                arg.offset_from(used_name)
                            };
                            let s_str: *const c_char = if s.is_null() {
                                b"\0".as_ptr() as *const c_char
                            } else {
                                s
                            };
                            // get tv_type from tv_buf offset 0 (v_type is c_int at offset 0)
                            let tv_type = *(tv as *const c_int);
                            rs_list_one_var_a(
                                b"\0".as_ptr() as *const c_char,
                                used_name,
                                name_size,
                                tv_type,
                                s_str,
                                first,
                            );
                            xfree(s as *mut c_void);
                        }
                        tv_clear(tv);
                    }
                }
            }
            xfree(tofree as *mut c_void);
        }
        arg = skipwhite(arg);
    }

    arg
}

/// Delete all "menutrans_" variables from the global variable table.
///
/// Matches C `del_menutrans_vars`.
///
/// # Safety
/// Must be called when the global variable state is valid.
#[no_mangle]
pub unsafe extern "C" fn rs_del_menutrans_vars() {
    let ht = nvim_get_globvarht_ptr();
    hash_lock(ht);
    let mut todo = nvim_vars_ht_get_used(ht) as isize;
    let mut hi = nvim_vars_ht_get_array(ht);
    let prefix = b"menutrans_\0".as_ptr() as *const c_char;
    while todo > 0 {
        if nvim_hashitem_empty(hi) == 0 {
            todo -= 1;
            let key = nvim_vars_hashitem_get_key(hi);
            if strncmp(key, prefix, 10) == 0 {
                nvim_vars_delete_var(ht, hi);
            }
        }
        hi = nvim_hashitem_advance(hi);
    }
    hash_unlock(ht);
}
