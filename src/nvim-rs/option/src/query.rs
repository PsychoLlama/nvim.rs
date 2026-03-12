//! Option query utility functions (Phase 4, pass 1 and 2)
//!
//! Rust implementations of small utility/query functions from option_shim.c.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::if_not_else)]
#![allow(clippy::borrow_as_ptr)]

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
}

extern "C" {
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;

    // can_bs
    fn nvim_option_get_p_bs() -> *const c_char;
    fn nvim_curbuf_is_prompt() -> c_int;

    // get_equalprg
    fn nvim_curbuf_get_b_p_ep() -> *const c_char;
    fn nvim_option_get_p_ep() -> *const c_char;

    // get_findfunc
    fn nvim_curbuf_get_b_p_ffu() -> *const c_char;
    fn nvim_option_get_p_ffu() -> *const c_char;

    // get_bkc_flags
    fn nvim_get_bkc_flags() -> c_uint;
    fn nvim_buf_get_bkc_flags(buf: BufHandle) -> c_uint;

    // get_flp_value
    fn nvim_get_p_flp() -> *const c_char;
    fn nvim_buf_get_p_flp(buf: BufHandle) -> *const c_char;

    // get_ve_flags
    fn nvim_get_ve_flags_global() -> c_uint;
    fn nvim_win_get_ve_flags(wp: WinHandle) -> c_uint;

    // vimrc_found
    fn nvim_option_vim_getenv(envname: *const c_char) -> *mut c_char;
    fn nvim_option_FullName_save(fname: *const c_char, force: bool) -> *mut c_char;
    fn nvim_option_os_setenv(name: *const c_char, value: *const c_char, overwrite: c_int) -> c_int;
    fn xfree(ptr: *mut c_char);

    // set_iminsert_global / set_imsearch_global
    fn nvim_set_p_iminsert(v: i64);
    fn nvim_set_p_imsearch(v: i64);
    fn nvim_buf_get_b_p_iminsert(buf: BufHandle) -> i64;
    fn nvim_buf_get_b_p_imsearch(buf: BufHandle) -> i64;

    // reset_modifiable
    fn nvim_option_set_p_ma(v: c_int);
    fn nvim_curbuf_set_b_p_ma(v: c_int);
    fn nvim_change_option_default_bool(opt_idx: c_int, value: c_int);

    // TTY options (Phase 2)
    fn nvim_option_get_t_colors() -> c_int;
    fn nvim_option_get_p_term() -> *const c_char;
    fn nvim_option_get_p_ttytype() -> *const c_char;
    fn nvim_option_set_p_term(val: *mut c_char);
    fn nvim_option_set_p_ttytype(val: *mut c_char);
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

// BS constants matching option_vars.h (BS_INDENT='i', BS_EOL='l' are valid but not compared directly)
const BS_START: c_int = b's' as c_int;
const BS_NOSTOP: c_int = b'p' as c_int;

/// Check if backspacing over something is allowed.
///
/// `what` is one of BS_INDENT, BS_EOL, BS_START, or BS_NOSTOP.
#[no_mangle]
pub unsafe extern "C" fn rs_can_bs(what: c_int) -> c_int {
    // BS_START is disallowed in prompt buffers
    if what == BS_START && nvim_curbuf_is_prompt() != 0 {
        return 0;
    }
    let p_bs = nvim_option_get_p_bs();
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
#[no_mangle]
pub unsafe extern "C" fn rs_get_equalprg() -> *const c_char {
    let b_p_ep = nvim_curbuf_get_b_p_ep();
    if !b_p_ep.is_null() && *b_p_ep != 0 {
        b_p_ep
    } else {
        nvim_option_get_p_ep()
    }
}

/// Get the value of 'findfunc', either the buffer-local one or the global one.
#[no_mangle]
pub unsafe extern "C" fn rs_get_findfunc() -> *const c_char {
    let b_p_ffu = nvim_curbuf_get_b_p_ffu();
    if !b_p_ffu.is_null() && *b_p_ffu != 0 {
        b_p_ffu
    } else {
        nvim_option_get_p_ffu()
    }
}

/// Get the local or global value of 'backupcopy' flags.
#[no_mangle]
pub unsafe extern "C" fn rs_get_bkc_flags(buf: BufHandle) -> c_uint {
    let local = nvim_buf_get_bkc_flags(buf);
    if local != 0 {
        local
    } else {
        nvim_get_bkc_flags()
    }
}

/// Get the local or global value of 'formatlistpat'.
#[no_mangle]
pub unsafe extern "C" fn rs_get_flp_value(buf: BufHandle) -> *const c_char {
    let b_p_flp = nvim_buf_get_p_flp(buf);
    if !b_p_flp.is_null() && *b_p_flp != 0 {
        b_p_flp
    } else {
        nvim_get_p_flp()
    }
}

// kOptVeFlag constants (from auto/option_vars.generated.h)
const K_OPT_VE_FLAG_NONE: c_uint = 0x10;
const K_OPT_VE_FLAG_NONE_U: c_uint = 0x20;

/// Get the local or global value of 'virtualedit' flags.
#[no_mangle]
pub unsafe extern "C" fn rs_get_ve_flags(wp: WinHandle) -> c_uint {
    let w_ve_flags = nvim_win_get_ve_flags(wp);
    let flags = if w_ve_flags != 0 {
        w_ve_flags
    } else {
        nvim_get_ve_flags_global()
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
    let p = nvim_option_vim_getenv(envname);
    if p.is_null() {
        // Set $envname to the full path of the first vimrc file found.
        let full = nvim_option_FullName_save(fname, false);
        if !full.is_null() {
            nvim_option_os_setenv(envname, full, 1);
            xfree(full);
        }
    } else {
        xfree(p);
    }
}

/// Set the global value for 'iminsert' to the local value.
#[export_name = "set_iminsert_global"]
pub unsafe extern "C" fn rs_set_iminsert_global(buf: BufHandle) {
    let val = nvim_buf_get_b_p_iminsert(buf);
    nvim_set_p_iminsert(val);
}

/// Set the global value for 'imsearch' to the local value.
#[export_name = "set_imsearch_global"]
pub unsafe extern "C" fn rs_set_imsearch_global(buf: BufHandle) {
    let val = nvim_buf_get_b_p_imsearch(buf);
    nvim_set_p_imsearch(val);
}

/// Reset the 'modifiable' option and its default value.
#[export_name = "reset_modifiable"]
pub unsafe extern "C" fn rs_reset_modifiable() {
    nvim_curbuf_set_b_p_ma(0);
    nvim_option_set_p_ma(0);
    nvim_change_option_default_bool(K_OPT_MODIFIABLE, 0);
}

// =============================================================================
// Phase 2: TTY and key-related functions
// =============================================================================

// NUMBUFLEN matches vim_defs.h
const NUMBUFLEN: usize = 65;

/// Return allocated OptVal for a TTY option name (t_Co, term, ttytype, t_xx).
///
/// Returns NIL_OPTVAL if name is not a TTY option.
#[no_mangle]
pub unsafe extern "C" fn rs_get_tty_option(name: *const c_char) -> OptVal {
    if name.is_null() {
        return OptVal::nil();
    }

    let value: *mut c_char;

    // "t_Co"
    if strcmp(name, c"t_Co".as_ptr()) == 0 {
        let t_colors = nvim_option_get_t_colors();
        if t_colors <= 1 {
            value = xstrdup(c"".as_ptr());
        } else {
            value = xmalloc(NUMBUFLEN);
            snprintf(value, NUMBUFLEN, c"%d".as_ptr(), t_colors);
        }
    } else if strcmp(name, c"term".as_ptr()) == 0 {
        let p_term = nvim_option_get_p_term();
        value = if !p_term.is_null() {
            xstrdup(p_term)
        } else {
            xstrdup(c"nvim".as_ptr())
        };
    } else if strcmp(name, c"ttytype".as_ptr()) == 0 {
        let p_ttytype = nvim_option_get_p_ttytype();
        value = if !p_ttytype.is_null() {
            xstrdup(p_ttytype)
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
#[no_mangle]
pub unsafe extern "C" fn rs_set_tty_option(name: *const c_char, value: *mut c_char) -> bool {
    if name.is_null() {
        return false;
    }
    if strcmp(name, c"term".as_ptr()) == 0 {
        let old = nvim_option_get_p_term();
        if !old.is_null() {
            xfree(old.cast_mut());
        }
        nvim_option_set_p_term(value);
        return true;
    }
    if strcmp(name, c"ttytype".as_ptr()) == 0 {
        let old = nvim_option_get_p_ttytype();
        if !old.is_null() {
            xfree(old.cast_mut());
        }
        nvim_option_set_p_ttytype(value);
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
#[no_mangle]
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
    fn nvim_win_get_p_winbl(wp: WinHandle) -> c_int;
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_shadow(wp: WinHandle) -> bool;
    fn nvim_win_set_grid_blending(wp: WinHandle, val: bool);

    // do_syntax_autocmd
    fn nvim_buf_set_b_flags_syn_set(buf: BufHandle);
    fn nvim_apply_syntax_autocmd(buf: BufHandle, force: bool);

    // do_spelllang_source
    fn nvim_win_get_b_p_spl(win: WinHandle) -> *const c_char;
    #[link_name = "source_runtime_vim_lua"]
    fn nvim_ex2_source_runtime_vim_lua(name: *const c_char, flags: c_int) -> c_int;

    // get_fileformat_force (nvim_eap_get_force_ff/bin are in ex_docmd.c)
    fn nvim_eap_get_force_ff(eap: *const c_void) -> c_int;
    fn nvim_eap_get_force_bin(eap: *const c_void) -> c_int;
    fn nvim_option_buf_get_b_p_bin(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_p_ff_first(buf: BufHandle) -> c_int;
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
    let winbl = nvim_win_get_p_winbl(wp);
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
    SYN_RECURSIVE += 1;
    let force = value_changed != 0 || SYN_RECURSIVE == 1;
    nvim_apply_syntax_autocmd(buf, force);
    nvim_buf_set_b_flags_syn_set(buf);
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
#[no_mangle]
pub unsafe extern "C" fn rs_get_fileformat_force(buf: BufHandle, eap: *const c_void) -> c_int {
    let force_ff = nvim_eap_get_force_ff(eap);
    let c = if force_ff != 0 {
        force_ff
    } else {
        let force_bin = nvim_eap_get_force_bin(eap);
        let b_p_bin = nvim_option_buf_get_b_p_bin(buf);
        let is_bin = if force_bin != 0 {
            force_bin == FORCE_BIN
        } else {
            b_p_bin != 0
        };
        if is_bin {
            return EOL_UNIX;
        }
        nvim_buf_get_b_p_ff_first(buf)
    };

    match c as u8 {
        b'u' => EOL_UNIX,
        b'm' => EOL_MAC,
        _ => EOL_DOS,
    }
}
