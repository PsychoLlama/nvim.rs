//! Option query utility functions (Phase 4, pass 1 and 2)
//!
//! Rust implementations of small utility/query functions from option_shim.c.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::if_not_else)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::useless_let_if_seq)]

use nvim_buffer::buf_struct::BufStruct;
use nvim_ex_cmds_types::ExArg;
use std::ffi::{c_char, c_int, c_uint, c_void};

use crate::opt_index::K_OPT_MODIFIABLE;
use crate::storage::{OptVal, String_};
use crate::{BufHandle, OptValType, WinHandle};

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    static mut redraw_tabline: bool;
    static mut need_maketitle: bool;
    static mut curbuf: BufHandle;
}

extern "C" {
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;

    // can_bs
    #[link_name = "rs_bt_prompt"]
    fn nvim_curbuf_is_prompt_via_curbuf(buf: BufHandle) -> bool;

    // get_equalprg
    fn nvim_curbuf_get_b_p_ep() -> *const c_char;

    // get_findfunc
    fn nvim_curbuf_get_b_p_ffu() -> *const c_char;

    // get_ve_flags
    fn nvim_win_get_ve_flags(wp: WinHandle) -> c_uint;

    // vimrc_found
    fn vim_getenv(envname: *const c_char) -> *mut c_char;
    fn FullName_save(fname: *const c_char, force: bool) -> *mut c_char;
    fn os_setenv(name: *const c_char, value: *const c_char, overwrite: c_int) -> c_int;
    fn xfree(ptr: *mut c_char);

    // set_iminsert_global / set_imsearch_global
    static mut p_iminsert: i64;
    static mut p_imsearch: i64;

    // reset_modifiable
    static mut p_ma: c_int;

    // TTY options (Phase 2)
    fn rs_is_tty_option(name: *const c_char) -> c_int;
    fn xmalloc(size: usize) -> *mut c_char;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;

    // key functions (Phase 2)
    fn find_special_key(
        srcp: *mut *const c_char,
        src_len: usize,
        modp: *mut c_int,
        flags: c_int,
        did_simplify: *mut bool,
    ) -> c_int;
    fn rs_ctrl_chr(c: c_int) -> c_int;

    // C string functions (avoid libc dep)
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strlen(s: *const c_char) -> usize;
}

/// Rust-owned storage for the "term" TTY option (formerly C static `p_term`).
pub(crate) static mut P_TERM: *mut c_char = std::ptr::null_mut();
/// Rust-owned storage for the "ttytype" TTY option (formerly C static `p_ttytype`).
pub(crate) static mut P_TTYTYPE: *mut c_char = std::ptr::null_mut();

// BS constants matching option_vars.h (BS_INDENT='i', BS_EOL='l' are valid but not compared directly)
const BS_START: c_int = b's' as c_int;
const BS_NOSTOP: c_int = b'p' as c_int;

/// Check if backspacing over something is allowed.
///
/// `what` is one of BS_INDENT, BS_EOL, BS_START, or BS_NOSTOP.
#[allow(clippy::must_use_candidate)]
#[export_name = "can_bs"]
pub unsafe extern "C" fn rs_can_bs(what: c_int) -> c_int {
    // BS_START is disallowed in prompt buffers
    if what == BS_START && nvim_curbuf_is_prompt_via_curbuf(curbuf) {
        return 0;
    }
    let p_bs = crate::p_bs.cast_const();
    if p_bs.is_null() {
        return 0;
    }
    // Legacy: '2' means allow backspace over everything except nostop
    if *p_bs as u8 == b'2' {
        return c_int::from(what != BS_NOSTOP);
    }
    c_int::from(!vim_strchr(p_bs, what).is_null())
}

/// Get the value of 'equalprg', either the buffer-local one or the global one.
#[allow(clippy::must_use_candidate)]
#[export_name = "get_equalprg"]
pub unsafe extern "C" fn rs_get_equalprg() -> *const c_char {
    let b_p_ep = nvim_curbuf_get_b_p_ep();
    if !b_p_ep.is_null() && *b_p_ep != 0 {
        b_p_ep
    } else {
        crate::p_ep.cast_const()
    }
}

/// Get the value of 'findfunc', either the buffer-local one or the global one.
#[allow(clippy::must_use_candidate)]
#[export_name = "get_findfunc"]
pub unsafe extern "C" fn rs_get_findfunc() -> *const c_char {
    let b_p_ffu = nvim_curbuf_get_b_p_ffu();
    if !b_p_ffu.is_null() && *b_p_ffu != 0 {
        b_p_ffu
    } else {
        crate::p_ffu.cast_const()
    }
}

/// Get the local or global value of 'backupcopy' flags.
#[allow(clippy::must_use_candidate)]
#[export_name = "get_bkc_flags"]
pub unsafe extern "C" fn rs_get_bkc_flags(buf: BufHandle) -> c_uint {
    let local = (*buf.cast::<BufStruct>()).b_bkc_flags;
    if local != 0 {
        local
    } else {
        crate::bkc_flags
    }
}

/// Get the local or global value of 'formatlistpat'.
#[allow(clippy::must_use_candidate)]
#[export_name = "get_flp_value"]
pub unsafe extern "C" fn rs_get_flp_value(buf: BufHandle) -> *const c_char {
    let b_p_flp = (*buf.cast::<BufStruct>()).b_p_flp;
    if !b_p_flp.is_null() && *b_p_flp != 0 {
        b_p_flp
    } else {
        crate::p_flp.cast_const()
    }
}

// kOptVeFlag constants (from auto/option_vars.generated.h)
const K_OPT_VE_FLAG_NONE: c_uint = 0x10;
const K_OPT_VE_FLAG_NONE_U: c_uint = 0x20;

/// Get the local or global value of 'virtualedit' flags.
#[allow(clippy::must_use_candidate)]
#[export_name = "get_ve_flags"]
pub unsafe extern "C" fn rs_get_ve_flags(wp: WinHandle) -> c_uint {
    let w_ve_flags = nvim_win_get_ve_flags(wp);
    let flags = if w_ve_flags != 0 {
        w_ve_flags
    } else {
        crate::ve_flags
    };
    flags & !(K_OPT_VE_FLAG_NONE | K_OPT_VE_FLAG_NONE_U)
}

/// Redraw the window title and/or tab page text later.
#[export_name = "redraw_titles"]
pub unsafe extern "C" fn rs_redraw_titles() {
    need_maketitle = true;
    redraw_tabline = true;
}

/// Handle vimrc file discovery: set $envname if not already set.
///
/// Called when a vimrc or "VIMINIT" has been found.
#[export_name = "vimrc_found"]
pub unsafe extern "C" fn rs_vimrc_found(fname: *const c_char, envname: *const c_char) {
    if fname.is_null() || envname.is_null() {
        return;
    }
    let p = vim_getenv(envname);
    if p.is_null() {
        // Set $envname to the full path of the first vimrc file found.
        let full = FullName_save(fname, false);
        if !full.is_null() {
            os_setenv(envname, full, 1);
            xfree(full);
        }
    } else {
        xfree(p);
    }
}

/// Set the global value for 'iminsert' to the local value.
#[export_name = "set_iminsert_global"]
pub unsafe extern "C" fn rs_set_iminsert_global(buf: BufHandle) {
    p_iminsert = (*buf.cast::<BufStruct>()).b_p_iminsert;
}

/// Set the global value for 'imsearch' to the local value.
#[export_name = "set_imsearch_global"]
pub unsafe extern "C" fn rs_set_imsearch_global(buf: BufHandle) {
    p_imsearch = (*buf.cast::<BufStruct>()).b_p_imsearch;
}

/// Reset the 'modifiable' option and its default value.
#[export_name = "reset_modifiable"]
pub unsafe extern "C" fn rs_reset_modifiable() {
    (*curbuf.cast::<BufStruct>()).b_p_ma = 0;
    p_ma = 0;
    crate::defaults::rs_change_option_default(
        K_OPT_MODIFIABLE as c_int,
        OptVal {
            type_: OptValType::Boolean,
            data: crate::storage::OptValData { boolean: 0 },
        },
    );
}

// =============================================================================
// Phase 2: TTY and key-related functions
// =============================================================================

// NUMBUFLEN matches vim_defs.h
const NUMBUFLEN: usize = 65;

/// Return allocated OptVal for a TTY option name (t_Co, term, ttytype, t_xx).
///
/// Returns NIL_OPTVAL if name is not a TTY option.
#[allow(clippy::must_use_candidate)]
#[export_name = "get_tty_option"]
pub unsafe extern "C" fn rs_get_tty_option(name: *const c_char) -> OptVal {
    if name.is_null() {
        return OptVal::nil();
    }

    let value: *mut c_char;

    // "t_Co"
    if strcmp(name, c"t_Co".as_ptr()) == 0 {
        let t_colors = crate::t_colors;
        if t_colors <= 1 {
            value = xstrdup(c"".as_ptr());
        } else {
            value = xmalloc(NUMBUFLEN);
            snprintf(value, NUMBUFLEN, c"%d".as_ptr(), t_colors);
        }
    } else if strcmp(name, c"term".as_ptr()) == 0 {
        value = if !P_TERM.is_null() {
            xstrdup(P_TERM)
        } else {
            xstrdup(c"nvim".as_ptr())
        };
    } else if strcmp(name, c"ttytype".as_ptr()) == 0 {
        value = if !P_TTYTYPE.is_null() {
            xstrdup(P_TTYTYPE)
        } else {
            xstrdup(c"nvim".as_ptr())
        };
    } else if rs_is_tty_option(name) != 0 {
        // All other t_* options were removed; return empty string
        value = xstrdup(c"".as_ptr());
    } else {
        return OptVal::nil();
    }

    // Build string OptVal (CSTR_AS_OPTVAL semantics: owns the string)
    let len = strlen(value);
    OptVal {
        type_: OptValType::String,
        data: crate::storage::OptValData {
            string: String_ {
                data: value,
                size: len,
            },
        },
    }
}

/// Set a TTY option. Returns true if the name is a settable TTY option.
///
/// Takes ownership of `value` on success (as in the C implementation).
#[export_name = "set_tty_option"]
pub unsafe extern "C" fn rs_set_tty_option(name: *const c_char, value: *mut c_char) -> bool {
    if name.is_null() {
        return false;
    }
    if strcmp(name, c"term".as_ptr()) == 0 {
        if !P_TERM.is_null() {
            xfree(P_TERM.cast());
        }
        P_TERM = value;
        return true;
    }
    if strcmp(name, c"ttytype".as_ptr()) == 0 {
        if !P_TTYTYPE.is_null() {
            xfree(P_TTYTYPE.cast());
        }
        P_TTYTYPE = value;
        return true;
    }
    false
}

// FSK_* flags matching keycodes/src/lib.rs
const FSK_KEYCODE: c_int = 0x01;
const FSK_KEEP_X_KEY: c_int = 0x02;
const FSK_SIMPLIFY: c_int = 0x08;

// TERMCAP2KEY macro: -(a + (b << 8))
#[inline]
const fn termcap2key(a: u8, b: u8) -> c_int {
    -((a as c_int) + ((b as c_int) << 8))
}

// KS_ZERO = 255, KE_FILLER = b'X'
const K_ZERO: c_int = termcap2key(255, b'X');

/// Translate a <> key name or t_xx string into a key code.
///
/// `arg_arg` points after the '<' (or to start of "t_xx").
/// `len` is the length available.
/// `has_lt` indicates whether the '<' was present.
///
/// Returns key code, or 0 if not recognized.
unsafe fn find_key_len(arg_arg: *const c_char, len: usize, has_lt: bool) -> c_int {
    let mut key: c_int = 0;
    let arg = arg_arg;

    // t_xx format: don't use get_special_key_code (it calls add_termcap_entry)
    if len >= 4 && *arg as u8 == b't' && *arg.add(1) as u8 == b'_' {
        if !has_lt || *arg.add(4) as u8 == b'>' {
            key = termcap2key(*arg.add(2) as u8, *arg.add(3) as u8);
        }
    } else if has_lt {
        // Put arg at the '<'
        let at_lt = arg.sub(1);
        let mut p: *const c_char = at_lt;
        let mut modifiers: c_int = 0;
        key = find_special_key(
            &mut p,
            len + 1,
            &mut modifiers,
            FSK_KEYCODE | FSK_KEEP_X_KEY | FSK_SIMPLIFY,
            std::ptr::null_mut(),
        );
        if modifiers != 0 {
            // can't handle modifiers here
            key = 0;
        }
    }
    key
}

/// Convert a key name or string into a key value.
///
/// Used for 'cedit', 'wildchar' and 'wildcharm' options.
#[allow(clippy::must_use_candidate)]
#[export_name = "string_to_key"]
pub unsafe extern "C" fn rs_string_to_key(arg: *mut c_char) -> c_int {
    if arg.is_null() || *arg == 0 {
        return 0;
    }
    if *arg as u8 == b'<' && *arg.add(1) != 0 {
        return find_key_len(arg.add(1), strlen(arg), true);
    }
    if *arg as u8 == b'^' && *arg.add(1) != 0 {
        let key = rs_ctrl_chr(c_int::from(*arg.add(1) as u8));
        // ^@ is <Nul>
        if key == 0 {
            K_ZERO
        } else {
            key
        }
    } else {
        c_int::from(*arg as u8)
    }
}

// =============================================================================
// Phase 6: Utility functions (check_blending, do_syntax_autocmd,
//           do_spelllang_source, get_fileformat_force)
// =============================================================================

extern "C" {
    // check_blending
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_shadow(wp: WinHandle) -> bool;
    fn nvim_win_set_grid_blending(wp: WinHandle, val: bool);

    // do_syntax_autocmd
    fn nvim_apply_syntax_autocmd(buf: BufHandle, force: bool);

    // do_spelllang_source
    fn nvim_win_get_b_p_spl(win: WinHandle) -> *const c_char;
    #[link_name = "source_runtime_vim_lua"]
    fn nvim_ex2_source_runtime_vim_lua(name: *const c_char, flags: c_int) -> c_int;

}

/// EOL constants matching option_vars.h
const EOL_UNIX: c_int = 0;
const EOL_DOS: c_int = 1;
const EOL_MAC: c_int = 2;

/// FORCE_BIN constant matching ex_cmds_defs.h
const FORCE_BIN: c_int = 1;

/// DIP_ALL constant matching runtime.h
const DIP_ALL: c_int = 0x01;

/// Update w_grid_alloc.blending based on winbl and floating/shadow config.
///
/// Translation of C `check_blending`.
#[export_name = "check_blending"]
pub unsafe extern "C" fn rs_check_blending(wp: WinHandle) {
    let winbl = crate::win_ref(wp).w_p_winbl();
    let floating = nvim_win_get_floating(wp) != 0;
    let shadow = nvim_win_get_config_shadow(wp);
    let blending = winbl > 0 || (floating && shadow);
    nvim_win_set_grid_blending(wp, blending);
}

/// Recursion counter for do_syntax_autocmd (safe: single-threaded).
static mut SYN_RECURSIVE: c_int = 0;

/// When 'syntax' is set, trigger EVENT_SYNTAX autocmds.
///
/// Translation of C `do_syntax_autocmd`.
#[no_mangle]
pub unsafe extern "C" fn rs_do_syntax_autocmd(buf: BufHandle, value_changed: c_int) {
    const BF_SYN_SET: c_int = 0x200;
    SYN_RECURSIVE += 1;
    let force = value_changed != 0 || SYN_RECURSIVE == 1;
    nvim_apply_syntax_autocmd(buf, force);
    (*buf.cast::<BufStruct>()).b_flags |= BF_SYN_SET;
    SYN_RECURSIVE -= 1;
}

/// Source spell/LANG.* runtime files for the window's current spelllang.
///
/// Translation of C `do_spelllang_source`.
#[no_mangle]
pub unsafe extern "C" fn rs_do_spelllang_source(win: WinHandle) {
    let q_ptr = nvim_win_get_b_p_spl(win);
    if q_ptr.is_null() {
        return;
    }

    // Skip "cjk," prefix if present
    let q = if *q_ptr as u8 == b'c'
        && *q_ptr.add(1) as u8 == b'j'
        && *q_ptr.add(2) as u8 == b'k'
        && *q_ptr.add(3) as u8 == b','
    {
        q_ptr.add(4)
    } else {
        q_ptr
    };

    // Find end of first language name (alphanumeric or '-' only)
    let mut p = q;
    while *p != 0 {
        let c = *p as u8;
        if !c.is_ascii_alphanumeric() && c != b'-' {
            break;
        }
        p = p.add(1);
    }

    if p > q {
        let lang_len = p.offset_from(q) as usize;
        // Build "spell/<lang>.*" path
        let mut fname = [0u8; 200];
        let prefix = b"spell/";
        let suffix = b".*";
        let prefix_len = prefix.len();
        let total_len = prefix_len + lang_len + suffix.len();
        if total_len < fname.len() {
            fname[..prefix_len].copy_from_slice(prefix);
            std::ptr::copy_nonoverlapping(q, fname.as_mut_ptr().add(prefix_len).cast(), lang_len);
            fname[prefix_len + lang_len..prefix_len + lang_len + suffix.len()]
                .copy_from_slice(suffix);
            fname[total_len] = 0;
            nvim_ex2_source_runtime_vim_lua(fname.as_ptr().cast(), DIP_ALL);
        }
    }
}

/// Get file format override considering ++ff and ++bin command arguments.
///
/// Translation of C `get_fileformat_force`. Returns EOL_UNIX, EOL_DOS, or EOL_MAC.
#[allow(clippy::must_use_candidate)]
#[export_name = "get_fileformat_force"]
pub unsafe extern "C" fn rs_get_fileformat_force(buf: BufHandle, eap: *const ExArg) -> c_int {
    let force_ff = (*eap).force_ff;
    let c = if force_ff != 0 {
        force_ff
    } else {
        let force_bin = (*eap).force_bin;
        let is_bin = if force_bin != 0 {
            force_bin == FORCE_BIN
        } else {
            (*buf.cast::<BufStruct>()).b_p_bin != 0
        };
        if is_bin {
            return EOL_UNIX;
        }
        let ff = (*buf.cast::<BufStruct>()).b_p_ff;
        if ff.is_null() {
            0
        } else {
            c_int::from(*ff as u8)
        }
    };

    match c as u8 {
        b'u' => EOL_UNIX,
        b'm' => EOL_MAC,
        _ => EOL_DOS,
    }
}

// =============================================================================
// vimoption2dict cluster
// =============================================================================

use crate::storage::SctxT;
use nvim_api::{Dict, Error, NvimString, Object};

/// OptIndex type matching C (c_int alias).
type OptIndex = c_int;

// kOptFlag* bitmask constants for vimoption2dict
const K_OPT_FLAG_WAS_SET: u32 = 1 << 3;
const K_OPT_FLAG_COMMA: u32 = 1 << 10;
const K_OPT_FLAG_NO_DUP: u32 = 1 << 12;
const K_OPT_FLAG_FLAG_LIST: u32 = 1 << 13;

// Scope constants
const K_OPT_SCOPE_WIN: c_int = 1;
const K_OPT_SCOPE_BUF: c_int = 2;

// OPT_GLOBAL / OPT_LOCAL flags
const OPT_GLOBAL_FLAG: c_int = 0x01;
const OPT_LOCAL_FLAG: c_int = 0x02;

// kOptValType* constants (must match C enum)
const K_OPT_VAL_TYPE_NIL: c_int = -1;
const K_OPT_VAL_TYPE_BOOLEAN: c_int = 0;
const K_OPT_VAL_TYPE_NUMBER: c_int = 1;
const K_OPT_VAL_TYPE_STRING: c_int = 2;

// kOptCount is not a Rust constant, so we read it from C
// kOptInvalid = -1
const K_OPT_INVALID: c_int = -1;

extern "C" {
    // Option scope query
    fn option_has_scope(opt_idx: OptIndex, scope: c_int) -> c_int;
    fn nvim_get_option_scope_idx(opt_idx: OptIndex, scope: c_int) -> c_int;

    // Option metadata accessors
    fn nvim_option_get_fullname(opt_idx: OptIndex) -> *const c_char;
    fn nvim_option_get_shortname(opt_idx: OptIndex) -> *const c_char;
    fn nvim_get_option_flags(opt_idx: OptIndex) -> u32;
    fn rs_option_is_global_local(opt_idx: OptIndex) -> c_int;
    fn rs_option_get_type(opt_idx: OptIndex) -> c_int;

    // Script context accessors (pointer-based, avoids sret ABI hazard for 24-byte sctx_T).
    // nvim_get_option_script_ctx_ptr already exists in option_shim.c (global path).
    fn nvim_get_option_script_ctx_ptr(opt_idx: OptIndex) -> *mut SctxT;
    // scope-index based script context pointer accessors (scope_idx = scope_specific index)
    fn nvim_get_win_p_script_ctx_ptr(win: *const c_void, scope_idx: c_int) -> *mut SctxT;
    fn nvim_get_buf_p_script_ctx_ptr(buf: *const c_void, scope_idx: c_int) -> *mut SctxT;

    // def_val accessor
    fn nvim_get_option_def_val_data_ptr(opt_idx: OptIndex) -> *const c_void;
    fn rs_optval_from_varp(opt_idx: OptIndex, varp: *mut c_void) -> crate::storage::OptVal;

    // optval -> Object conversion (implemented in value.rs, exported via #[no_mangle])
    fn rs_optval_as_object(o: crate::storage::OptVal) -> Object;

    // Option lookup
    fn nvim_find_option_len_hash(name: *const c_char, len: usize) -> OptIndex;

    // API Error helpers (matches VALIDATE_S macro behavior)
    fn api_err_invalid(
        err: *mut Error,
        name: *const c_char,
        val_s: *const c_char,
        val_n: i64,
        is_string: bool,
    );

    // Arena arena_dict
    fn rs_arena_dict(arena: *mut c_void, max_size: usize) -> Dict;

    // Dict put
    fn rs_dict_put_static(dict: *mut Dict, key: *const c_char, value: Object);
}

/// Return a static C string for the option value type.
unsafe fn optval_type_name(type_: c_int) -> *const c_char {
    match type_ {
        K_OPT_VAL_TYPE_NIL => c"nil".as_ptr(),
        K_OPT_VAL_TYPE_BOOLEAN => c"boolean".as_ptr(),
        K_OPT_VAL_TYPE_NUMBER => c"number".as_ptr(),
        K_OPT_VAL_TYPE_STRING => c"string".as_ptr(),
        _ => c"unknown".as_ptr(),
    }
}

/// C string for static keys.
macro_rules! ckey {
    ($s:literal) => {
        concat!($s, "\0").as_ptr().cast::<c_char>()
    };
}

/// Build a Dict for one vimoption.
///
/// Mirrors `vimoption2dict` in option_shim.c.
///
/// # Safety
/// All pointers must be valid for their respective lifetimes.
#[allow(clippy::too_many_lines)]
unsafe fn vimoption2dict_rs(
    opt_idx: OptIndex,
    opt_flags: c_int,
    buf: *const c_void,
    win: *const c_void,
    arena: *mut c_void,
) -> Dict {
    let mut dict = rs_arena_dict(arena, 13);

    let fullname = nvim_option_get_fullname(opt_idx);
    let shortname = nvim_option_get_shortname(opt_idx);

    rs_dict_put_static(
        &mut dict,
        ckey!("name"),
        Object::string(NvimString {
            data: fullname.cast_mut(),
            size: libc::strlen(fullname),
        }),
    );
    rs_dict_put_static(
        &mut dict,
        ckey!("shortname"),
        Object::string(NvimString {
            data: shortname.cast_mut(),
            size: libc::strlen(shortname),
        }),
    );

    let scope_str = if option_has_scope(opt_idx, K_OPT_SCOPE_BUF) != 0 {
        c"buf".as_ptr()
    } else if option_has_scope(opt_idx, K_OPT_SCOPE_WIN) != 0 {
        c"win".as_ptr()
    } else {
        c"global".as_ptr()
    };
    rs_dict_put_static(
        &mut dict,
        ckey!("scope"),
        Object::string(NvimString {
            data: scope_str.cast_mut(),
            size: libc::strlen(scope_str),
        }),
    );

    let flags = nvim_get_option_flags(opt_idx);
    rs_dict_put_static(
        &mut dict,
        ckey!("global_local"),
        Object::boolean(rs_option_is_global_local(opt_idx) != 0),
    );
    rs_dict_put_static(
        &mut dict,
        ckey!("commalist"),
        Object::boolean(flags & K_OPT_FLAG_COMMA != 0),
    );
    rs_dict_put_static(
        &mut dict,
        ckey!("flaglist"),
        Object::boolean(flags & K_OPT_FLAG_FLAG_LIST != 0),
    );
    rs_dict_put_static(
        &mut dict,
        ckey!("was_set"),
        Object::boolean(flags & K_OPT_FLAG_WAS_SET != 0),
    );

    // Determine script context for this option.
    // Read through pointers to avoid sret ABI hazard with 24-byte sctx_T on x86-64 SysV.
    // A null pointer means "not set"; fall back to zeroed SctxT.
    let zeroed = SctxT {
        sc_sid: 0,
        sc_seq: 0,
        sc_lnum: 0,
        sc_chan: 0,
    };
    let script_ctx = if opt_flags == OPT_GLOBAL_FLAG {
        let p = nvim_get_option_script_ctx_ptr(opt_idx);
        if p.is_null() {
            zeroed
        } else {
            *p
        }
    } else {
        // OPT_LOCAL or fallback mode
        let mut sctx = zeroed;
        if option_has_scope(opt_idx, K_OPT_SCOPE_BUF) != 0 {
            let idx = nvim_get_option_scope_idx(opt_idx, K_OPT_SCOPE_BUF);
            let p = nvim_get_buf_p_script_ctx_ptr(buf, idx);
            if !p.is_null() {
                sctx = *p;
            }
        }
        if option_has_scope(opt_idx, K_OPT_SCOPE_WIN) != 0 {
            let idx = nvim_get_option_scope_idx(opt_idx, K_OPT_SCOPE_WIN);
            let p = nvim_get_win_p_script_ctx_ptr(win, idx);
            if !p.is_null() {
                sctx = *p;
            }
        }
        if opt_flags != OPT_LOCAL_FLAG && sctx.sc_sid == 0 {
            let p = nvim_get_option_script_ctx_ptr(opt_idx);
            if !p.is_null() {
                sctx = *p;
            }
        }
        sctx
    };

    rs_dict_put_static(
        &mut dict,
        ckey!("last_set_sid"),
        Object::integer(i64::from(script_ctx.sc_sid)),
    );
    rs_dict_put_static(
        &mut dict,
        ckey!("last_set_linenr"),
        Object::integer(i64::from(script_ctx.sc_lnum)),
    );
    rs_dict_put_static(
        &mut dict,
        ckey!("last_set_chan"),
        Object::integer(script_ctx.sc_chan as i64),
    );

    let type_name = optval_type_name(rs_option_get_type(opt_idx));
    rs_dict_put_static(
        &mut dict,
        ckey!("type"),
        Object::string(NvimString {
            data: type_name.cast_mut(),
            size: libc::strlen(type_name),
        }),
    );

    // def_val: read default value from options[opt_idx].def_val
    let def_varp = nvim_get_option_def_val_data_ptr(opt_idx)
        .cast_mut()
        .cast::<c_void>();
    let def_val = rs_optval_from_varp(opt_idx, def_varp);
    rs_dict_put_static(&mut dict, ckey!("default"), rs_optval_as_object(def_val));

    rs_dict_put_static(
        &mut dict,
        ckey!("allows_duplicates"),
        Object::boolean(flags & K_OPT_FLAG_NO_DUP == 0),
    );

    dict
}

/// Rust implementation of get_vimoption.
///
/// Looks up option by name and returns a Dict of option metadata.
///
/// # Safety
/// All pointer arguments must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_get_vimoption(
    name_data: *const c_char,
    name_size: usize,
    opt_flags: c_int,
    buf: *const c_void,
    win: *const c_void,
    arena: *mut c_void,
    err: *mut Error,
) -> Dict {
    let opt_idx = nvim_find_option_len_hash(name_data, name_size);
    if opt_idx == K_OPT_INVALID {
        // Match VALIDATE_S(opt_idx != kOptInvalid, "option (not found)", name.data, {...})
        api_err_invalid(err, c"option (not found)".as_ptr(), name_data, 0, true);
        return Dict::empty();
    }
    vimoption2dict_rs(opt_idx, opt_flags, buf, win, arena)
}

// C helpers for rs_get_winbuf_options
extern "C" {
    fn nvim_tv_dict_alloc_for_winbuf() -> *mut c_void;
    fn nvim_get_varp_current(opt_idx: OptIndex) -> *mut c_void;
    fn nvim_dict_add_option_varp(dict: *mut c_void, opt_idx: OptIndex, varp: *mut c_void);
}

/// Rust implementation of get_winbuf_options.
/// Returns a dict_T* containing all buffer-local or window-local options.
///
/// # Safety
/// Calls C allocation and dict functions.
#[no_mangle]
pub unsafe extern "C" fn rs_get_winbuf_options(bufopt: c_int) -> *mut c_void {
    use crate::opt_index::K_OPT_COUNT;
    let d = nvim_tv_dict_alloc_for_winbuf();
    for opt_idx in 0..K_OPT_COUNT {
        let scope = if bufopt != 0 {
            K_OPT_SCOPE_BUF
        } else {
            K_OPT_SCOPE_WIN
        };
        if option_has_scope(opt_idx, scope) == 0 {
            continue;
        }
        let varp = nvim_get_varp_current(opt_idx);
        if !varp.is_null() {
            nvim_dict_add_option_varp(d, opt_idx, varp);
        }
    }
    d
}

/// Rust implementation of get_all_vimoptions.
/// Returns a Dict containing metadata for all options.
///
/// # Safety
/// `arena` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_get_all_vimoptions(arena: *mut c_void) -> Dict {
    use crate::opt_index::K_OPT_COUNT;
    let count = K_OPT_COUNT as usize;
    let mut retval = rs_arena_dict(arena, count);
    for opt_idx in 0..K_OPT_COUNT {
        let fullname = nvim_option_get_fullname(opt_idx);
        let opt_dict = vimoption2dict_rs(
            opt_idx,
            OPT_GLOBAL_FLAG,
            std::ptr::null(),
            std::ptr::null(),
            arena,
        );
        rs_dict_put_static(&mut retval, fullname, Object::dict(opt_dict));
    }
    retval
}
