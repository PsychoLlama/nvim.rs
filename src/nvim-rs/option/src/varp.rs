//! Option variable pointer dispatch (get_varp_from / get_varp_scope_from)
//!
//! This module provides Rust implementations of the C functions `get_varp_from`
//! and `get_varp_scope_from`, which resolve the effective pointer to an option
//! variable based on scope flags and the current buffer/window.
//!
//! The implementations call C accessor functions for all struct field access.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::unreadable_literal)]

use std::ffi::{c_char, c_int, c_void};
use std::sync::OnceLock;

use crate::index::OptIndex;
use crate::opt_index::*;
use crate::{BufHandle, WinHandle};

// =============================================================================
// OPT_GLOBAL / OPT_LOCAL flags (from option.h)
// =============================================================================

/// Use global value
const OPT_GLOBAL: c_int = 0x01;
/// Use local value
const OPT_LOCAL: c_int = 0x02;

/// Value meaning "no local undolevel" (buf->b_p_ul sentinel)
const NO_LOCAL_UNDOLEVEL: i64 = -123_456;

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    /// Get p->var from a vimoption_T pointer
    fn nvim_vimoption_get_var(p: *mut c_void) -> *mut c_void;

    /// Get OptIndex for a vimoption_T by pointer arithmetic against options[]
    fn nvim_get_opt_idx_from_ptr(p: *mut c_void) -> OptIndex;

    /// Get sizeof(winopt_T) at runtime
    fn nvim_get_sizeof_winopt_T() -> c_int;

    /// Check if option is hidden (immutable, var points to def_val)
    #[link_name = "rs_option_is_hidden"]
    fn nvim_opt_is_hidden(opt_idx: OptIndex) -> c_int;

    /// Check if option is global-only (no local value)
    #[link_name = "rs_option_is_global_only"]
    fn nvim_option_is_global_only(opt_idx: OptIndex) -> c_int;

    /// Check if option is global-local (has both global and local)
    #[link_name = "rs_option_is_global_local"]
    fn nvim_option_is_global_local(opt_idx: OptIndex) -> c_int;

    /// Check if option is window-local
    #[link_name = "rs_option_is_window_local"]
    fn nvim_option_is_window_local(opt_idx: OptIndex) -> c_int;

    /// Fill offset table for buf_T option fields (kOptCount entries)
    fn nvim_buf_opt_field_offsets(out: *mut isize, len: c_int);

    /// Get address of a win_T option field by OptIndex
    fn nvim_win_get_opt_field_addr(win: WinHandle, idx: OptIndex) -> *mut c_void;

    /// Issue an internal error message
    fn iemsg(msg: *const c_char);
}

// Error message for unknown option in get_varp_from
static E356_MSG: &[u8] = b"E356: get_varp ERROR\0";

// =============================================================================
// buf_T option field offset table (replaces C nvim_buf_get_opt_field_addr)
// =============================================================================

/// Lazily-initialized table mapping OptIndex -> byte offset into buf_T.
/// Initialized once from C via `nvim_buf_opt_field_offsets`.
/// Sentinel value -1 means "not a buf field for this index".
static BUF_FIELD_OFFSETS: OnceLock<[isize; crate::opt_index::K_OPT_COUNT as usize]> =
    OnceLock::new();

/// Get (and lazily initialize) the buf_T field offset table.
pub(crate) fn buf_field_offsets() -> &'static [isize; crate::opt_index::K_OPT_COUNT as usize] {
    BUF_FIELD_OFFSETS.get_or_init(|| {
        let mut table = [-1isize; crate::opt_index::K_OPT_COUNT as usize];
        // Safety: nvim_buf_opt_field_offsets writes exactly K_OPT_COUNT entries.
        unsafe {
            nvim_buf_opt_field_offsets(table.as_mut_ptr(), crate::opt_index::K_OPT_COUNT as c_int);
        }
        table
    })
}

/// Rust implementation: get address of a buf_T option field by OptIndex.
///
/// Replaces C `nvim_buf_get_opt_field_addr`. Uses a precomputed offset table
/// so no FFI round-trip is needed per call.
///
/// # Safety
/// `buf` must be a valid `buf_T*` and `idx` must be a valid `OptIndex`.
unsafe fn buf_get_opt_field_addr(buf: BufHandle, idx: OptIndex) -> *mut c_void {
    if buf.is_null() {
        return std::ptr::null_mut();
    }
    let offsets = buf_field_offsets();
    let idx_usize = idx as usize;
    if idx_usize >= offsets.len() {
        std::process::abort();
    }
    let offset = offsets[idx_usize];
    if offset < 0 {
        // Unhandled index: mirror the C `default: abort()`.
        std::process::abort();
    }
    buf.cast::<u8>().add(offset as usize).cast::<c_void>()
}

// =============================================================================
// Helper: check if a global-local buf string option is set locally (non-NUL)
// =============================================================================

/// Returns the local buf field address if the string is non-empty, else `p_var`.
///
/// # Safety
/// `buf` must be a valid `buf_T*` and `opt_idx` must map to a `char*` buf field.
unsafe fn buf_str_local_or_global(
    buf: BufHandle,
    opt_idx: OptIndex,
    p_var: *mut c_void,
) -> *mut c_void {
    // The field is a `char*`, so the address is `char**`.
    // We dereference to get `char*` (the string), then dereference again to
    // get the first byte. Non-NUL first byte means the local value is set.
    let field = buf_get_opt_field_addr(buf, opt_idx).cast::<*mut c_char>();
    // *field: the char* string pointer; **field: first char of that string
    if *(*field) != 0 {
        field.cast::<c_void>()
    } else {
        p_var
    }
}

/// Returns the local win field address if the string is non-empty, else `p_var`.
///
/// # Safety
/// `win` must be a valid `win_T*` and `opt_idx` must map to a `char*` win field.
unsafe fn win_str_local_or_global(
    win: WinHandle,
    opt_idx: OptIndex,
    p_var: *mut c_void,
) -> *mut c_void {
    let field = nvim_win_get_opt_field_addr(win, opt_idx).cast::<*mut c_char>();
    if *(*field) != 0 {
        field.cast::<c_void>()
    } else {
        p_var
    }
}

/// Returns the local buf numeric field address if value >= 0, else `p_var`.
///
/// # Safety
/// `buf` must be a valid `buf_T*` and `opt_idx` must map to an `i64` buf field.
unsafe fn buf_num_local_or_global(
    buf: BufHandle,
    opt_idx: OptIndex,
    p_var: *mut c_void,
) -> *mut c_void {
    let field = buf_get_opt_field_addr(buf, opt_idx).cast::<i64>();
    if *field >= 0 {
        field.cast::<c_void>()
    } else {
        p_var
    }
}

/// Returns the local win numeric field address if value >= 0, else `p_var`.
///
/// # Safety
/// `win` must be a valid `win_T*` and `opt_idx` must map to an `i64` win field.
unsafe fn win_num_local_or_global(
    win: WinHandle,
    opt_idx: OptIndex,
    p_var: *mut c_void,
) -> *mut c_void {
    let field = nvim_win_get_opt_field_addr(win, opt_idx).cast::<i64>();
    if *field >= 0 {
        field.cast::<c_void>()
    } else {
        p_var
    }
}

// =============================================================================
// rs_get_varp_from
// =============================================================================

/// Rust port of `get_varp_from`.
///
/// Returns a `*mut c_void` pointing to the effective option variable storage
/// for the given `vimoption_T *p`, `buf_T *buf`, and `win_T *win`.
///
/// For hidden and global-only options: returns `p->var`.
/// For global-local options: returns the local field if it has been set,
///   otherwise falls back to `p->var`.
/// For purely local options: returns the buf/win field address.
///
/// # Safety
/// All pointer arguments must be valid. `p` must point to a `vimoption_T`
/// within the global `options[]` array. `buf` and `win` must be non-null.
#[allow(clippy::must_use_candidate)]
#[export_name = "get_varp_from"]
pub unsafe extern "C" fn rs_get_varp_from(
    p: *mut c_void,
    buf: BufHandle,
    win: WinHandle,
) -> *mut c_void {
    let opt_idx = nvim_get_opt_idx_from_ptr(p);
    let p_var = nvim_vimoption_get_var(p);

    // Hidden options and global-only options always use p->var
    if nvim_opt_is_hidden(opt_idx) != 0 || nvim_option_is_global_only(opt_idx) != 0 {
        return p_var;
    }

    match opt_idx {
        // -----------------------------------------------------------------------
        // Global-local buf string options: use local if non-NUL, else global
        // -----------------------------------------------------------------------
        K_OPT_EQUALPRG | K_OPT_KEYWORDPRG | K_OPT_PATH | K_OPT_TAGS | K_OPT_TAGCASE
        | K_OPT_BACKUPCOPY | K_OPT_DEFINE | K_OPT_INCLUDE | K_OPT_COMPLETEOPT
        | K_OPT_DICTIONARY | K_OPT_DIFFANCHORS | K_OPT_THESAURUS | K_OPT_THESAURUSFUNC
        | K_OPT_FORMATPRG | K_OPT_FINDFUNC | K_OPT_ERRORFORMAT | K_OPT_GREPFORMAT
        | K_OPT_GREPPRG | K_OPT_MAKEPRG | K_OPT_LISPWORDS | K_OPT_MAKEENCODING => {
            buf_str_local_or_global(buf, opt_idx, p_var)
        }

        // -----------------------------------------------------------------------
        // Global-local win string options: use local if non-NUL, else global
        // -----------------------------------------------------------------------
        K_OPT_SHOWBREAK | K_OPT_STATUSLINE | K_OPT_WINBAR | K_OPT_FILLCHARS | K_OPT_LISTCHARS
        | K_OPT_VIRTUALEDIT => win_str_local_or_global(win, opt_idx, p_var),

        // -----------------------------------------------------------------------
        // Global-local buf numeric options: use local if >= 0, else global
        // -----------------------------------------------------------------------
        K_OPT_AUTOCOMPLETE | K_OPT_AUTOREAD => buf_num_local_or_global(buf, opt_idx, p_var),

        // -----------------------------------------------------------------------
        // Global-local win numeric options: use local if >= 0, else global
        // -----------------------------------------------------------------------
        K_OPT_SIDESCROLLOFF | K_OPT_SCROLLOFF => win_num_local_or_global(win, opt_idx, p_var),

        // -----------------------------------------------------------------------
        // Undolevels: global-local with a special sentinel value
        // -----------------------------------------------------------------------
        K_OPT_UNDOLEVELS => {
            let field = buf_get_opt_field_addr(buf, opt_idx).cast::<i64>();
            // NO_LOCAL_UNDOLEVEL is the sentinel meaning "use global value"
            if *field == NO_LOCAL_UNDOLEVEL {
                p_var
            } else {
                field.cast::<c_void>()
            }
        }

        // -----------------------------------------------------------------------
        // Window-local only options (includes spell via win->w_s)
        // -----------------------------------------------------------------------
        K_OPT_ARABIC | K_OPT_LIST | K_OPT_SPELL | K_OPT_CURSORCOLUMN | K_OPT_CURSORLINE
        | K_OPT_CURSORLINEOPT | K_OPT_COLORCOLUMN | K_OPT_DIFF | K_OPT_EVENTIGNOREWIN
        | K_OPT_FOLDCOLUMN | K_OPT_FOLDENABLE | K_OPT_FOLDIGNORE | K_OPT_FOLDLEVEL
        | K_OPT_FOLDMETHOD | K_OPT_FOLDMINLINES | K_OPT_FOLDNESTMAX | K_OPT_FOLDEXPR
        | K_OPT_FOLDTEXT | K_OPT_FOLDMARKER | K_OPT_NUMBER | K_OPT_RELATIVENUMBER
        | K_OPT_NUMBERWIDTH | K_OPT_WINFIXBUF | K_OPT_WINFIXHEIGHT | K_OPT_WINFIXWIDTH
        | K_OPT_PREVIEWWINDOW | K_OPT_LHISTORY | K_OPT_RIGHTLEFT | K_OPT_RIGHTLEFTCMD
        | K_OPT_SCROLL | K_OPT_SMOOTHSCROLL | K_OPT_WRAP | K_OPT_LINEBREAK | K_OPT_BREAKINDENT
        | K_OPT_BREAKINDENTOPT | K_OPT_SCROLLBIND | K_OPT_CURSORBIND | K_OPT_CONCEALCURSOR
        | K_OPT_CONCEALLEVEL | K_OPT_SIGNCOLUMN | K_OPT_WINHIGHLIGHT | K_OPT_WINBLEND
        | K_OPT_STATUSCOLUMN | K_OPT_SPELLCAPCHECK | K_OPT_SPELLFILE | K_OPT_SPELLLANG
        | K_OPT_SPELLOPTIONS => nvim_win_get_opt_field_addr(win, opt_idx),

        // -----------------------------------------------------------------------
        // Buffer-local only options
        // -----------------------------------------------------------------------
        K_OPT_AUTOINDENT | K_OPT_BINARY | K_OPT_BOMB | K_OPT_BUFHIDDEN | K_OPT_BUFTYPE
        | K_OPT_BUFLISTED | K_OPT_BUSY | K_OPT_CHANNEL | K_OPT_COPYINDENT | K_OPT_CINDENT
        | K_OPT_CINKEYS | K_OPT_CINOPTIONS | K_OPT_CINSCOPEDECLS | K_OPT_CINWORDS
        | K_OPT_COMMENTS | K_OPT_COMMENTSTRING | K_OPT_COMPLETE | K_OPT_COMPLETEFUNC
        | K_OPT_OMNIFUNC | K_OPT_ENDOFFILE | K_OPT_ENDOFLINE | K_OPT_FIXENDOFLINE
        | K_OPT_EXPANDTAB | K_OPT_FILEENCODING | K_OPT_FILEFORMAT | K_OPT_FILETYPE
        | K_OPT_FORMATOPTIONS | K_OPT_FORMATLISTPAT | K_OPT_IMINSERT | K_OPT_IMSEARCH
        | K_OPT_INFERCASE | K_OPT_ISKEYWORD | K_OPT_INCLUDEEXPR | K_OPT_INDENTEXPR
        | K_OPT_INDENTKEYS | K_OPT_FORMATEXPR | K_OPT_LISP | K_OPT_LISPOPTIONS | K_OPT_MODELINE
        | K_OPT_MATCHPAIRS | K_OPT_MODIFIABLE | K_OPT_MODIFIED | K_OPT_NRFORMATS
        | K_OPT_PRESERVEINDENT | K_OPT_QUOTEESCAPE | K_OPT_READONLY | K_OPT_SCROLLBACK
        | K_OPT_SMARTINDENT | K_OPT_SOFTTABSTOP | K_OPT_SUFFIXESADD | K_OPT_SWAPFILE
        | K_OPT_SYNMAXCOL | K_OPT_SYNTAX | K_OPT_SHIFTWIDTH | K_OPT_TAGFUNC | K_OPT_TABSTOP
        | K_OPT_TEXTWIDTH | K_OPT_UNDOFILE | K_OPT_WRAPMARGIN | K_OPT_VARSOFTTABSTOP
        | K_OPT_VARTABSTOP | K_OPT_KEYMAP => buf_get_opt_field_addr(buf, opt_idx),

        _ => {
            iemsg(E356_MSG.as_ptr().cast::<c_char>());
            // Fallback: always return a valid pointer (same as C)
            buf_get_opt_field_addr(buf, K_OPT_WRAPMARGIN)
        }
    }
}

// =============================================================================
// rs_get_varp_scope_from
// =============================================================================

/// Rust port of `get_varp_scope_from`.
///
/// Adds scope-flag routing on top of `rs_get_varp_from`.
///
/// - `OPT_GLOBAL` and not global-only: for window-local options, returns
///   `GLOBAL_WO(get_varp_from(...))` (the global shadow stored after `winopt_T`);
///   otherwise returns `p->var`.
/// - `OPT_LOCAL` for global-local options: returns the local buf/win field directly.
/// - Otherwise: delegates to `rs_get_varp_from`.
///
/// # Safety
/// All pointer arguments must be valid.
#[allow(clippy::must_use_candidate)]
#[export_name = "get_varp_scope_from"]
pub unsafe extern "C" fn rs_get_varp_scope_from(
    p: *mut c_void,
    opt_flags: c_int,
    buf: BufHandle,
    win: WinHandle,
) -> *mut c_void {
    let opt_idx = nvim_get_opt_idx_from_ptr(p);
    let p_var = nvim_vimoption_get_var(p);

    if (opt_flags & OPT_GLOBAL) != 0 && nvim_option_is_global_only(opt_idx) == 0 {
        if nvim_option_is_window_local(opt_idx) != 0 {
            // GLOBAL_WO(get_varp_from(p, buf, win))
            // = (char *)get_varp_from(...) + sizeof(winopt_T)
            let varp = rs_get_varp_from(p, buf, win).cast::<c_char>();
            let sizeof_winopt = nvim_get_sizeof_winopt_T() as usize;
            return varp.add(sizeof_winopt).cast::<c_void>();
        }
        return p_var;
    }

    if (opt_flags & OPT_LOCAL) != 0 && nvim_option_is_global_local(opt_idx) != 0 {
        // Return the local field address directly for global-local options.
        // These are the same fields as rs_get_varp_from, but always returned
        // regardless of whether the local value has been set.
        return match opt_idx {
            K_OPT_FORMATPRG | K_OPT_FINDFUNC | K_OPT_ERRORFORMAT | K_OPT_GREPFORMAT
            | K_OPT_GREPPRG | K_OPT_MAKEPRG | K_OPT_EQUALPRG | K_OPT_KEYWORDPRG | K_OPT_PATH
            | K_OPT_AUTOCOMPLETE | K_OPT_AUTOREAD | K_OPT_TAGS | K_OPT_TAGCASE | K_OPT_DEFINE
            | K_OPT_INCLUDE | K_OPT_COMPLETEOPT | K_OPT_DICTIONARY | K_OPT_DIFFANCHORS
            | K_OPT_THESAURUS | K_OPT_THESAURUSFUNC | K_OPT_TAGFUNC | K_OPT_UNDOLEVELS
            | K_OPT_LISPWORDS | K_OPT_BACKUPCOPY | K_OPT_MAKEENCODING => {
                buf_get_opt_field_addr(buf, opt_idx)
            }

            K_OPT_SIDESCROLLOFF | K_OPT_SCROLLOFF | K_OPT_SHOWBREAK | K_OPT_STATUSLINE
            | K_OPT_WINBAR | K_OPT_FILLCHARS | K_OPT_LISTCHARS | K_OPT_VIRTUALEDIT => {
                nvim_win_get_opt_field_addr(win, opt_idx)
            }

            _ => {
                // OPT_LOCAL set but option not recognized as global-local:
                // should not happen. Abort as the C version does.
                std::process::abort()
            }
        };
    }

    rs_get_varp_from(p, buf, win)
}
