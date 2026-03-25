//! Normal mode additional character handling.
//!
//! This module provides the Rust implementation of `normal_get_additional_char()`
//! from `src/nvim/normal.c`. Handles reading second/third characters for
//! multi-key commands like `r`, `f`, `g'`, digraphs, and composing characters.

use std::ffi::c_int;

use crate::dispatch::types::NormalStateHandle;
use crate::execute::NV_LANG;
use crate::types::{CmdargT, NormalState};
use crate::{CapHandle, OapHandle};

/// Cast `NormalStateHandle` to a typed `*mut NormalState`.
///
/// # Safety
/// The handle must be a valid non-null `NormalState*`.
#[inline]
unsafe fn ns(s: NormalStateHandle) -> *mut NormalState {
    s.as_ptr().cast::<NormalState>()
}

// =============================================================================
// Constants (verified with _Static_assert in normal.c)
// =============================================================================

const MODE_REPLACE: c_int = 0x110;
const MODE_LREPLACE: c_int = 0x120;
const MODE_LANGMAP: c_int = 0x20;
const MODE_NORMAL_BUSY: c_int = 0x1001;
const B_IMODE_LMAP: c_int = 1;
const CPO_DIGRAPH: c_int = b'D' as c_int;
const CTRL_BSL: c_int = 28; // Ctrl-backslash
const CTRL_K: c_int = 11; // Ctrl-K
const CTRL_N: c_int = 14;
const CTRL_G: c_int = 7;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    static mut State: c_int;

    // cmdarg_T accessors
    fn nvim_cap_get_cmdchar(cap: CapHandle) -> c_int;
    fn nvim_cap_get_nchar(cap: CapHandle) -> c_int;

    // oparg_T accessors
    fn nvim_oap_set_op_type(oap: OapHandle, val: c_int);

    // Phase 2B wrappers
    fn nvim_plain_vgetc_wrapper() -> c_int;
    fn nvim_langmap_adjust(c: c_int, condition: bool) -> c_int;
    fn nvim_add_to_showcmd_wrapper(c: c_int) -> bool;
    fn nvim_del_from_showcmd_wrapper(n: c_int);
    fn nvim_inc_no_mapping();
    fn nvim_dec_no_mapping();
    fn nvim_inc_allow_keys();
    fn nvim_dec_allow_keys();
    fn nvim_set_did_cursorhold(val: bool);
    fn nvim_get_curbuf_b_p_iminsert() -> c_int;
    fn nvim_ui_cursor_shape_no_check_conceal();
    fn nvim_get_digraph(flag: bool) -> c_int;
    fn nvim_vpeekc_wrapper() -> c_int;
    fn nvim_do_sleep_wrapper(ms: c_int, allow_int: bool);
    fn nvim_vim_strchr_p_cpo(c: c_int) -> bool;
    fn nvim_vungetc_wrapper(c: c_int);
    fn nvim_get_op_type_wrapper(c1: c_int, c2: c_int) -> c_int;
    fn nvim_get_p_ttm() -> i64;
    fn nvim_get_p_tm() -> i64;

    // Phase 6: composing char handler accessors
    fn nvim_cap_get_nchar_len(cap: CapHandle) -> c_int;
    fn nvim_cap_get_nchar_composing_ptr(cap: CapHandle) -> *const std::ffi::c_char; // search.c
    fn nvim_utf_iscomposing_check(prev: c_int, c: c_int, state_ptr: *mut i32) -> bool;
    fn nvim_utf_char2len_wrapper(c: c_int) -> c_int;
    fn nvim_utf_char2bytes_wrapper(c: c_int, buf: *mut std::ffi::c_char) -> c_int;
    fn nvim_get_MB_BYTE2LEN(c: c_int) -> c_int;
    fn nvim_gotchars_ignore_wrapper();

    // find_command (already in Rust, but we need the C wrapper)
    fn rs_find_command(cmdchar: c_int) -> c_int;
}

/// Which character field to read/write through the opaque handle.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CharTarget {
    Nchar,
    ExtraChar,
}

impl CharTarget {
    /// Read the target character from the cmdarg.
    unsafe fn get(self, ca: CapHandle) -> c_int {
        match self {
            Self::Nchar => nvim_cap_get_nchar(ca),
            Self::ExtraChar => (*ca.cast::<CmdargT>()).extra_char,
        }
    }

    /// Write the target character to the cmdarg.
    unsafe fn set(self, ca: CapHandle, val: c_int) {
        match self {
            Self::Nchar => (*ca.cast::<CmdargT>()).nchar = val,
            Self::ExtraChar => (*ca.cast::<CmdargT>()).extra_char = val,
        }
    }
}

/// Read, adjust and handle the second/third character once the target field is known.
///
/// # Safety
/// All handles must be valid pointers into the NormalState.
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
unsafe fn read_target_char(
    s: NormalStateHandle,
    ca: CapHandle,
    oa: OapHandle,
    target: CharTarget,
    repl: bool,
    lit: bool,
    lang: bool,
    flags: c_int,
    cmdchar: c_int,
) {
    let mut langmap_active = false;
    if repl {
        State = MODE_REPLACE;
        nvim_ui_cursor_shape_no_check_conceal();
    }
    if lang && nvim_get_curbuf_b_p_iminsert() == B_IMODE_LMAP {
        nvim_dec_no_mapping();
        nvim_dec_allow_keys();
        if repl {
            State = MODE_LREPLACE;
        } else {
            State = MODE_LANGMAP;
        }
        langmap_active = true;
    }

    let ch = nvim_plain_vgetc_wrapper();
    target.set(ca, ch);

    if langmap_active {
        nvim_inc_no_mapping();
        nvim_inc_allow_keys();
    }
    State = MODE_NORMAL_BUSY;
    (*ns(s)).need_flushbuf |= nvim_add_to_showcmd_wrapper(target.get(ca));

    if !lit {
        // Typing CTRL-K gets a digraph.
        if target.get(ca) == CTRL_K
            && ((flags & NV_LANG != 0) || target == CharTarget::ExtraChar)
            && !nvim_vim_strchr_p_cpo(CPO_DIGRAPH)
        {
            let c = nvim_get_digraph(false);
            (*ns(s)).c = c;
            if c > 0 {
                target.set(ca, c);
                nvim_del_from_showcmd_wrapper(3);
                (*ns(s)).need_flushbuf |= nvim_add_to_showcmd_wrapper(target.get(ca));
            }
        }

        // Adjust chars > 127, except after "tTfFr" commands.
        let adjusted = nvim_langmap_adjust(target.get(ca), !lang);
        target.set(ca, adjusted);
    }

    // When the next character is CTRL-\ a following CTRL-N means the
    // command is aborted and we go to Normal mode.
    let nchar = nvim_cap_get_nchar(ca);
    let extra_char = (*ca.cast::<CmdargT>()).extra_char;

    if target == CharTarget::ExtraChar
        && nchar == CTRL_BSL
        && (extra_char == CTRL_N || extra_char == CTRL_G)
    {
        (*ca.cast::<CmdargT>()).cmdchar = CTRL_BSL;
        (*ca.cast::<CmdargT>()).nchar = extra_char;
        (*ns(s)).idx = rs_find_command(CTRL_BSL);
    } else if (nchar == c_int::from(b'n') || nchar == c_int::from(b'N'))
        && cmdchar == c_int::from(b'g')
    {
        let op_type = nvim_get_op_type_wrapper(target.get(ca), 0);
        nvim_oap_set_op_type(oa, op_type);
    } else if target.get(ca) == CTRL_BSL {
        #[allow(clippy::cast_possible_truncation)]
        let mut towait: c_int = if nvim_get_p_ttm() >= 0 {
            nvim_get_p_ttm() as c_int
        } else {
            nvim_get_p_tm() as c_int
        };

        // Busy wait when typing "f<C-\>" and then something else.
        loop {
            let peeked = nvim_vpeekc_wrapper();
            (*ns(s)).c = peeked;
            if peeked > 0 || towait <= 0 {
                break;
            }
            let sleep_ms = if towait > 50 { 50 } else { towait };
            nvim_do_sleep_wrapper(sleep_ms, false);
            towait -= 50;
        }
        if (*ns(s)).c > 0 {
            let c = nvim_plain_vgetc_wrapper();
            (*ns(s)).c = c;
            if c != CTRL_N && c != CTRL_G {
                nvim_vungetc_wrapper(c);
            } else {
                (*ca.cast::<CmdargT>()).cmdchar = CTRL_BSL;
                (*ca.cast::<CmdargT>()).nchar = c;
                let new_idx = rs_find_command(CTRL_BSL);
                debug_assert!(new_idx >= 0);
                (*ns(s)).idx = new_idx;
            }
        }
    }

    if lang {
        // Handle composing characters.
        rs_normal_handle_composing_chars(s);
    }
}

/// Get an additional character for a normal-mode command.
///
/// This handles reading second/third characters for multi-key commands
/// like `r<char>`, `f<char>`, `g'<mark>`, digraphs, and composing characters.
///
/// # Safety
/// `s` must be a valid NormalState pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_normal_get_additional_char(s: NormalStateHandle) {
    let sp = ns(s);
    let ca: CapHandle = (&raw mut (*sp).ca).cast();
    let oa: OapHandle = (&raw mut (*sp).oa).cast();

    let mut repl = false;
    let mut lit = false;

    nvim_inc_no_mapping();
    nvim_inc_allow_keys();
    nvim_set_did_cursorhold(true);

    let cmdchar = nvim_cap_get_cmdchar(ca);
    let idx = (*sp).idx;

    // Determine which character field to use and whether we need a third char.
    let cp: Option<CharTarget> = if cmdchar == c_int::from(b'g') {
        // For 'g' get the next character now.
        let nchar = nvim_plain_vgetc_wrapper();
        let nchar = nvim_langmap_adjust(nchar, true);
        (*ca.cast::<CmdargT>()).nchar = nchar;
        (*ns(s)).need_flushbuf |= nvim_add_to_showcmd_wrapper(nchar);

        if nchar == c_int::from(b'r')
            || nchar == c_int::from(b'\'')
            || nchar == c_int::from(b'`')
            || nchar == CTRL_BSL
        {
            if nchar == c_int::from(b'r') {
                repl = true;
            } else {
                lit = true;
            }
            Some(CharTarget::ExtraChar)
        } else {
            None // no third character needed
        }
    } else {
        if cmdchar == c_int::from(b'r') {
            repl = true;
        }
        Some(CharTarget::Nchar)
    };

    let flags = crate::dispatch::table::rs_table_get_cmd_flags(idx);
    let lang = repl || (flags & NV_LANG != 0);

    // Get a second or third character.
    if let Some(target) = cp {
        read_target_char(s, ca, oa, target, repl, lit, lang, flags, cmdchar);
    }

    nvim_dec_no_mapping();
    nvim_dec_allow_keys();
}

/// Maximum byte length of nchar_composing buffer (normal_defs.h MAX_SCHAR_SIZE = 32).
const MAX_SCHAR_SIZE: usize = 32;
/// MAX_SCHAR_SIZE as c_int for comparisons with C int values.
const MAX_SCHAR_SIZE_INT: c_int = 32;

/// Handle composing characters after reading nchar in normal mode.
///
/// This is the Rust implementation of `nvim_normal_handle_composing_chars()`
/// from `normal_shim.c`. Reads composing code points that follow `ca.nchar`
/// and accumulates them into `ca.nchar_composing`.
///
/// # Safety
/// `s` must be a valid NormalState pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_normal_handle_composing_chars(s: NormalStateHandle) {
    let sp = ns(s);
    let ca: CapHandle = (&raw mut (*sp).ca).cast();

    nvim_dec_no_mapping();

    // GraphemeState is int32_t; GRAPHEME_STATE_INIT = 0.
    let mut grapheme_state: i32 = 0;
    let mut prev_code = nvim_cap_get_nchar(ca);

    loop {
        let peeked = nvim_vpeekc_wrapper();
        (*sp).c = peeked;
        if peeked <= 0 {
            break;
        }
        // Check the while condition: c >= 0x100 || MB_BYTE2LEN(c) > 1
        // MB_BYTE2LEN is safe for 0..=255; peeked may be > 255 for special keys.
        let is_mb = peeked >= 0x100 || nvim_get_MB_BYTE2LEN(peeked & 0xff) > 1;
        if !is_mb {
            break;
        }

        let c = nvim_plain_vgetc_wrapper();
        (*sp).c = c;

        if !nvim_utf_iscomposing_check(prev_code, c, std::ptr::addr_of_mut!(grapheme_state)) {
            nvim_vungetc_wrapper(c);
            break;
        }

        // Ensure nchar_composing holds the base character first.
        if nvim_cap_get_nchar_len(ca) == 0 {
            let nchar = nvim_cap_get_nchar(ca);
            // nchar_composing is mutable; the const is a C API convention.
            let composing_ptr = nvim_cap_get_nchar_composing_ptr(ca).cast_mut();
            let written = nvim_utf_char2bytes_wrapper(nchar, composing_ptr);
            (*ca.cast::<CmdargT>()).nchar_len = written;
        }

        // Append composing character if it fits.
        let cur_len = nvim_cap_get_nchar_len(ca);
        let extra = nvim_utf_char2len_wrapper(c);
        if cur_len + extra < MAX_SCHAR_SIZE_INT {
            let composing_ptr = nvim_cap_get_nchar_composing_ptr(ca).cast_mut();
            let offset = usize::try_from(cur_len).unwrap_or(0);
            let written = nvim_utf_char2bytes_wrapper(c, composing_ptr.add(offset));
            (*ca.cast::<CmdargT>()).nchar_len = cur_len + written;
        }
        prev_code = c;
    }

    // NUL-terminate nchar_composing.
    {
        let composing_ptr = nvim_cap_get_nchar_composing_ptr(ca).cast_mut();
        let cur_len = nvim_cap_get_nchar_len(ca);
        let offset = usize::try_from(cur_len)
            .unwrap_or(0)
            .min(MAX_SCHAR_SIZE - 1);
        *composing_ptr.add(offset) = 0;
    }

    nvim_inc_no_mapping();
    nvim_gotchars_ignore_wrapper(); // wraps no_u_sync++ / gotchars_ignore() / no_u_sync--
}
