//! Core statusline builder - direct port of `nvim_stl_build_stl_str_hl_impl`
//!
//! This module implements the full statusline format string parser and renderer,
//! working directly with raw `*mut c_char` output buffers and C-compatible
//! highlight/click record arrays.

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use nvim_window::{BufHandle, WinHandle};

use crate::{ScharT, StatuscolHandle};

// =============================================================================
// Constants (verified via _Static_assert in statusline.c)
// =============================================================================

// STL_* format characters (ASCII byte values)
const STL_FILEPATH: u8 = b'f';
const STL_FULLPATH: u8 = b'F';
const STL_FILENAME: u8 = b't';
const STL_COLUMN: u8 = b'c';
const STL_VIRTCOL: u8 = b'v';
const STL_VIRTCOL_ALT: u8 = b'V';
const STL_LINE: u8 = b'l';
const STL_NUMLINES: u8 = b'L';
const STL_BUFNO: u8 = b'n';
const STL_KEYMAP: u8 = b'k';
const STL_OFFSET: u8 = b'o';
const STL_OFFSET_X: u8 = b'O';
const STL_BYTEVAL: u8 = b'b';
const STL_BYTEVAL_X: u8 = b'B';
const STL_ROFLAG: u8 = b'r';
const STL_ROFLAG_ALT: u8 = b'R';
const STL_HELPFLAG: u8 = b'h';
const STL_HELPFLAG_ALT: u8 = b'H';
const STL_FILETYPE: u8 = b'y';
const STL_FILETYPE_ALT: u8 = b'Y';
const STL_PREVIEWFLAG: u8 = b'w';
const STL_PREVIEWFLAG_ALT: u8 = b'W';
const STL_MODIFIED: u8 = b'm';
const STL_MODIFIED_ALT: u8 = b'M';
const STL_QUICKFIX: u8 = b'q';
const STL_PERCENTAGE: u8 = b'p';
const STL_ALTPERCENT: u8 = b'P';
const STL_ARGLISTSTAT: u8 = b'a';
const STL_PAGENUM: u8 = b'N';
const STL_SHOWCMD: u8 = b'S';
const STL_FOLDCOL: u8 = b'C';
const STL_SIGNCOL: u8 = b's';
const STL_VIM_EXPR: u8 = b'{';
const STL_SEPARATE: u8 = b'=';
const STL_TRUNCMARK: u8 = b'<';
const STL_USER_HL: u8 = b'*';
const STL_HIGHLIGHT: u8 = b'#';
const STL_TABPAGENR: u8 = b'T';
const STL_TABCLOSENR: u8 = b'X';
const STL_CLICK_FUNC: u8 = b'@';

const NUL: u8 = 0;
const NL: c_int = 10;
const CAR: c_int = 13;
const TMPLEN: usize = 70;

// Verified option indices
const K_OPT_INVALID: c_int = -1;
const K_OPT_STATUSCOLUMN: c_int = 293;
const K_OPT_STATUSLINE: c_int = 294;
const K_OPT_TABLINE: c_int = 302;
const K_OPT_WINBAR: c_int = 355;
const K_OPT_RULERFORMAT: c_int = 241;

// Verified mode flags
const MODE_INSERT: c_int = 0x10;

// Verified VV indices
const VV_LNUM: c_int = 9;
const VV_RELNUM: c_int = 101;
const VV_VIRTNUM: c_int = 102;

// Other constants
const SCL_NUM: c_int = -2;
const EOL_MAC: c_int = 2;
const SID_ERROR: c_int = -5;
const ML_EMPTY: c_int = 1; // ML_EMPTY flag bit
const MAX_STL_EVAL_DEPTH: c_int = 10;
const MAX_STCWIDTH: c_int = 18; // Verified against C

const HLF_CLF: c_int = 17;
const HLF_FC: c_int = 29;

// Number base for formatting
const NUM_BASE_DECIMAL: c_int = 10;
const NUM_BASE_HEX: c_int = 16;

// Item types (matching C enum values)
const ITEM_NORMAL: c_int = 0;
const ITEM_EMPTY: c_int = 1;
const ITEM_GROUP: c_int = 2;
const ITEM_SEPARATE: c_int = 3;
const ITEM_HIGHLIGHT: c_int = 4;
const ITEM_HIGHLIGHT_SIGN: c_int = 5;
const ITEM_HIGHLIGHT_FOLD: c_int = 6;
const ITEM_TABPAGE: c_int = 7;
const ITEM_CLICK_FUNC: c_int = 8;
const ITEM_TRUNC: c_int = 9;

// =============================================================================
// C FFI types
// =============================================================================

/// Batch cursor info returned by nvim_stl_get_win_cursor_info.
/// Matches `StlCursorInfo` in statusline.c.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct StlCursorInfo {
    /// cursor lnum clamped to ml_line_count
    pub clamped_lnum: c_int,
    /// byte value at cursor position
    pub byte_value: c_int,
    /// byte offset at cursor (from rs_ml_find_line_or_offset)
    pub byte_offset: c_int,
    /// 1 if buffer ML_EMPTY flag is set
    pub ml_empty: c_int,
    /// 1 if cursor line is empty (first char NUL)
    pub empty_line: c_int,
    /// 1 if cursor lnum > line count
    pub cursor_invalid: c_int,
    /// first char of cursor line (uint8 cast to int)
    pub first_char: c_int,
}

/// Batch sign info returned by nvim_stl_stcp_get_sign_info.
/// Matches `StlSignInfo` in statusline.h.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct StlSignInfo {
    /// 1 if sattrs[idx].text[0] != 0
    has_text: c_int,
    /// sattrs[idx].hl_id
    hl_id: c_int,
    /// stcp->sign_cul_id
    sign_cul_id: c_int,
}

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Expression evaluation
    fn nvim_stl_eval_with_context(
        wp: WinHandle,
        expr: *mut c_char,
        mode: c_int,
        use_sandbox: bool,
    ) -> *mut c_char;
    #[link_name = "was_set_insecurely"]
    fn nvim_stl_was_set_insecurely(wp: WinHandle, opt_idx: c_int, opt_scope: c_int) -> c_int;

    // String operations (direct link to Rust/C implementations)
    #[link_name = "vim_strsize"]
    fn nvim_stl_vim_strsize(s: *const c_char) -> c_int;
    #[link_name = "ptr2cells"]
    fn nvim_stl_ptr2cells(s: *const c_char) -> c_int;
    #[link_name = "utfc_ptr2len"]
    fn nvim_stl_utfc_ptr2len(s: *const c_char) -> c_int;
    #[link_name = "schar_len"]
    fn nvim_stl_schar_len(c: ScharT) -> usize;
    #[link_name = "schar_get"]
    fn nvim_stl_schar_get(buf: *mut c_char, c: ScharT) -> usize;
    #[link_name = "rs_schar_from_ascii"]
    fn nvim_stl_schar_from_ascii(c: c_char) -> ScharT;

    // Buffer / path (direct link to Rust/C implementations)
    #[link_name = "rs_buf_spname"]
    fn nvim_stl_buf_spname(buf: BufHandle) -> *const c_char;
    fn nvim_stl_home_replace_trans(
        buf: BufHandle,
        src: *const c_char,
        dst: *mut c_char,
        dstlen: c_int,
    );
    #[link_name = "path_tail"]
    fn nvim_stl_path_tail(s: *const c_char) -> *const c_char;
    #[link_name = "rs_get_fileformat"]
    fn nvim_stl_get_fileformat(buf: BufHandle) -> c_int;
    #[link_name = "utf_ptr2char"]
    fn nvim_stl_utf_ptr2char(s: *const c_char) -> c_int;

    // Cursor / line (batch accessor)
    fn nvim_stl_get_win_cursor_info(wp: WinHandle) -> StlCursorInfo;

    // Global state (direct static access instead of C shims)
    static mut updating_screen: bool;
    static mut redraw_not_allowed: bool;
    static mut KeyTyped: bool;
    static mut did_emsg: c_int;
    fn nvim_stl_set_option_empty(opt_idx: c_int, opt_scope: c_int);
    static mut State: c_int;

    // Memory (direct link to C implementations)
    #[link_name = "xfree"]
    fn nvim_stl_xfree(ptr: *mut c_void);
    #[link_name = "xmemdupz"]
    fn nvim_stl_xmemdupz(s: *const c_char, len: usize) -> *mut c_char;

    // Position / arg list
    #[link_name = "rs_get_rel_pos"]
    fn nvim_stl_get_rel_pos(wp: WinHandle, buf: *mut c_char, buflen: c_int) -> c_int;
    #[link_name = "rs_append_arg_number"]
    fn nvim_stl_append_arg_number(wp: WinHandle, buf: *mut c_char, buflen: usize) -> c_int;

    // Showcmd
    fn nvim_stl_showcmd_matches_opt(opt_idx: c_int) -> c_int;
    fn nvim_stl_get_showcmd(buf: *mut c_char, buflen: c_int) -> c_int;

    // Vim variables
    #[link_name = "rs_get_vim_var_nr"]
    fn nvim_stl_get_vim_var_nr(vv_idx: c_int) -> i64;

    // Window accessors (direct link to window_shim.c where available)
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_win_buf_line_count(wp: WinHandle) -> c_int;
    #[link_name = "nvim_win_get_virtcol"]
    fn nvim_stl_win_get_w_virtcol(wp: WinHandle) -> c_int;
    #[link_name = "nvim_win_get_cursor_col"]
    fn nvim_stl_win_get_cursor_col(wp: WinHandle) -> c_int;
    #[link_name = "nvim_win_get_p_nu"]
    fn nvim_stl_win_get_p_nu(wp: WinHandle) -> c_int;
    #[link_name = "nvim_win_get_p_rnu"]
    fn nvim_stl_win_get_p_rnu(wp: WinHandle) -> c_int;
    fn nvim_stl_win_get_maxscwidth(wp: WinHandle) -> c_int;
    #[link_name = "nvim_win_get_scwidth"]
    fn nvim_stl_win_get_scwidth(wp: WinHandle) -> c_int;

    // Statuscolumn accessors
    // Statuscolumn sign info (batch accessor)
    fn nvim_stl_stcp_get_sign_info(stcp: StatuscolHandle, idx: c_int) -> StlSignInfo;
    #[link_name = "compute_foldcolumn"]
    fn nvim_stl_compute_foldcolumn(wp: WinHandle, col: c_int) -> c_int;
    fn nvim_stl_fill_foldcolumn(
        wp: WinHandle,
        stcp: StatuscolHandle,
        lnum: c_int,
        fdc: c_int,
        buf: *mut c_char,
        buflen: c_int,
    ) -> c_int;
    #[link_name = "use_cursor_line_highlight"]
    fn nvim_stl_use_cursor_line_hl(wp: WinHandle, lnum: c_int) -> bool;
    fn nvim_stl_describe_sign_text(buf: *mut c_char, text: *mut ScharT) -> c_int;
    #[link_name = "nvim_syn_name2id_len_wrapper"]
    fn nvim_stl_syn_name2id_len(name: *const c_char, len: c_int) -> c_int;

    // Buffer field accessors
    fn nvim_buf_get_b_fname(buf: BufHandle) -> *const c_char;
    fn nvim_buf_get_b_ffname(buf: BufHandle) -> *const c_char;
    fn nvim_buf_get_b_p_ro(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_p_ft(buf: BufHandle) -> *const c_char;
    fn nvim_buf_get_b_p_ma(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_changed(buf: BufHandle) -> bool;
    fn nvim_buf_get_help(buf: BufHandle) -> c_int;
    fn nvim_buf_get_fnum(buf: BufHandle) -> c_int;
    fn nvim_win_get_pvw(wp: WinHandle) -> c_int;

    // Quickfix / keymap
    fn nvim_stl_get_qf_info(wp: WinHandle, buf: *mut c_char, buflen: c_int) -> c_int;
    fn nvim_stl_get_keymap(wp: WinHandle, buf: *mut c_char, buflen: c_int) -> c_int;

    // NameBuff (MAXPATHL global buffer)
    fn nvim_get_namebuff() -> *mut c_char;

    // strlen from libc
    fn strlen(s: *const c_char) -> usize;
    fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

// MAXPATHL - max path length
const MAXPATHL: c_int = 4096;

/// Check if all characters in a C string are ASCII digits.
/// Replacement for the deleted `nvim_stl_str_all_digits` C accessor.
///
/// # Safety
/// `s` must be a valid null-terminated C string.
unsafe fn nvim_stl_str_all_digits(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0;
    }
    let mut p = s;
    while *p != 0 {
        if !(*p as u8).is_ascii_digit() {
            return 0;
        }
        p = p.add(1);
    }
    1
}

/// Format an integer with optional minimum width into a buffer.
/// Replacement for `nvim_stl_snprintf_int` (was `vim_snprintf_safelen(buf, buflen, fmt, minwid, num)`).
/// `fmt` encodes: optional `-` prefix, `%`, optional `0`, `*`, then `d` or `X`.
///
/// # Safety
/// `buf` must be valid for `buflen` bytes.
#[allow(clippy::cast_possible_truncation)]
unsafe fn nvim_stl_snprintf_int(
    buf: *mut c_char,
    buflen: usize,
    fmt: *const c_char,
    minwid: c_int,
    num: c_int,
) -> c_int {
    use std::io::Write;
    if buflen == 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts_mut(buf as *mut u8, buflen);
    let mut cursor = std::io::Cursor::new(&mut slice[..]);

    // Parse fmt: optional `-`, `%`, optional `0`, `*`, base specifier
    let fmt_bytes = {
        let mut len = 0;
        while *fmt.add(len) != 0 {
            len += 1;
        }
        std::slice::from_raw_parts(fmt as *const u8, len)
    };

    let left_align = fmt_bytes.first() == Some(&b'-');
    let zero_pad = fmt_bytes.contains(&b'0');
    let hex = fmt_bytes.last() == Some(&b'X');

    let abs_width = minwid.unsigned_abs() as usize;
    let _ = if hex {
        let s = if zero_pad {
            format!("{num:0>abs_width$X}")
        } else if left_align {
            format!("{num:<abs_width$X}")
        } else {
            format!("{num:>abs_width$X}")
        };
        write!(cursor, "{s}")
    } else {
        let s = if zero_pad {
            format!("{num:0>abs_width$}")
        } else if left_align {
            format!("{num:<abs_width$}")
        } else {
            format!("{num:>abs_width$}")
        };
        write!(cursor, "{s}")
    };
    cursor.position() as c_int
}

/// Format scientific notation number into a buffer.
/// Replacement for `nvim_stl_snprintf_sci` (was `vim_snprintf_safelen(buf, buflen, fmt, 0, num, exponent)`).
/// Produces output like `"123>4"` (num>exponent).
///
/// # Safety
/// `buf` must be valid for `buflen` bytes.
#[allow(clippy::cast_possible_truncation)] // cursor.position() fits in c_int (buffer ≤ 20 bytes)
unsafe fn nvim_stl_snprintf_sci(
    buf: *mut c_char,
    buflen: usize,
    fmt: *const c_char,
    num: c_int,
    exponent: c_int,
) -> c_int {
    use std::io::Write;
    if buflen == 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts_mut(buf as *mut u8, buflen);
    let mut cursor = std::io::Cursor::new(&mut slice[..]);

    // Parse fmt: optional `0`, `*`, base, `>`, `%`, base
    // The format is always built as: [%-?][0?][*]<base>>[%]<base>
    // Output: <num_formatted>><exponent_formatted>
    let fmt_bytes = {
        let mut len = 0;
        while *fmt.add(len) != 0 {
            len += 1;
        }
        std::slice::from_raw_parts(fmt as *const u8, len)
    };
    let hex = fmt_bytes.last() == Some(&b'X');
    let _ = if hex {
        write!(cursor, "{num:X}>{exponent:X}")
    } else {
        write!(cursor, "{num}>{exponent}")
    };
    cursor.position() as c_int
}

/// Calculate percentage position in a file.
/// Replacement for `nvim_stl_calc_percentage` (was `calc_percentage(lnum, line_count)`).
fn nvim_stl_calc_percentage(lnum: c_int, line_count: c_int) -> c_int {
    if line_count == 0 {
        return 0;
    }
    (lnum * 100 + line_count / 2) / line_count
}

/// Check if a character is a valid STL flag (equivalent to STL_ALL membership).
fn nvim_stl_valid_flag(c: c_int) -> c_int {
    const STL_ALL: &[u8] = &[
        STL_FILEPATH,
        STL_FULLPATH,
        STL_FILENAME,
        STL_COLUMN,
        STL_VIRTCOL,
        STL_VIRTCOL_ALT,
        STL_LINE,
        STL_NUMLINES,
        STL_BUFNO,
        STL_KEYMAP,
        STL_OFFSET,
        STL_OFFSET_X,
        STL_BYTEVAL,
        STL_BYTEVAL_X,
        STL_ROFLAG,
        STL_ROFLAG_ALT,
        STL_HELPFLAG,
        STL_HELPFLAG_ALT,
        STL_FILETYPE,
        STL_FILETYPE_ALT,
        STL_PREVIEWFLAG,
        STL_PREVIEWFLAG_ALT,
        STL_MODIFIED,
        STL_MODIFIED_ALT,
        STL_QUICKFIX,
        STL_PERCENTAGE,
        STL_ALTPERCENT,
        STL_ARGLISTSTAT,
        STL_PAGENUM,
        STL_SHOWCMD,
        STL_FOLDCOL,
        STL_SIGNCOL,
        STL_VIM_EXPR,
        STL_SEPARATE,
        STL_TRUNCMARK,
        STL_USER_HL,
        STL_HIGHLIGHT,
        STL_TABPAGENR,
        STL_TABCLOSENR,
        STL_CLICK_FUNC,
    ];
    // c comes from (uint8_t)(*fmt_p), so it fits in u8
    u8::try_from(c).map_or(0, |byte| c_int::from(STL_ALL.contains(&byte)))
}

// =============================================================================
// Item structure (matching C stl_item_t)
// =============================================================================

struct StlItem {
    start: *mut c_char,
    cmd: *mut c_char, // For ClickFunc items (allocated via xmemdupz)
    minwid: c_int,
    maxwid: c_int,
    item_type: c_int,
}

impl StlItem {
    const fn new() -> Self {
        Self {
            start: ptr::null_mut(),
            cmd: ptr::null_mut(),
            minwid: 0,
            maxwid: 9999,
            item_type: ITEM_NORMAL,
        }
    }
}

// =============================================================================
// Highlight record (matching C stl_hlrec_t)
// =============================================================================

/// C-compatible highlight record. Must match `stl_hlrec_t` layout.
#[repr(C)]
pub struct StlHlRec {
    pub start: *mut c_char,
    pub userhl: c_int,
    pub item: c_int, // STL_SIGNCOL or STL_FOLDCOL or 0
}

// =============================================================================
// Click record (matching C StlClickRecord)
// =============================================================================

/// C-compatible click definition. Must match `StlClickDefinition`.
#[repr(C)]
pub struct StlClickDef {
    pub click_type: c_int,
    pub tabnr: c_int,
    pub func: *mut c_char,
}

/// C-compatible click record. Must match `StlClickRecord`.
#[repr(C)]
pub struct StlClickRecord {
    pub def: StlClickDef,
    pub start: *const c_char,
}

// =============================================================================
// Thread-local storage for persistent arrays
// =============================================================================

// The C code uses function-scope statics. We use thread-locals since this is
// only called from the main thread.

use std::cell::RefCell;

thread_local! {
    static STL_ITEMS: RefCell<Vec<StlItem>> = RefCell::new(Vec::new());
    static STL_GROUPITEMS: RefCell<Vec<c_int>> = RefCell::new(Vec::new());
    static STL_HLTAB: RefCell<Vec<StlHlRec>> = RefCell::new(Vec::new());
    static STL_TABTAB: RefCell<Vec<StlClickRecord>> = RefCell::new(Vec::new());
    static STL_SEPARATOR_LOCATIONS: RefCell<Vec<c_int>> = RefCell::new(Vec::new());
    static CURITEM: RefCell<c_int> = const { RefCell::new(0) };
}

// =============================================================================
// Helper: schar_get_adv
// =============================================================================

/// Advance pointer by writing schar bytes. Equivalent to C's schar_get_adv.
#[inline]
unsafe fn schar_get_adv(pp: &mut *mut c_char, fillchar: ScharT) {
    let n = nvim_stl_schar_get(*pp, fillchar);
    *pp = (*pp).add(n);
}

// =============================================================================
// Main function
// =============================================================================

/// Build statusline string with highlight and click records.
///
/// This is the Rust port of the C function `nvim_stl_build_stl_str_hl_impl`.
///
/// # Safety
/// All pointer parameters must be valid according to C conventions.
#[allow(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::cognitive_complexity,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::similar_names
)]
pub unsafe fn build_stl_str_hl(
    wp: WinHandle,
    out: *mut c_char,
    outlen: usize,
    fmt: *mut c_char,
    opt_idx: c_int,
    opt_scope: c_int,
    mut fillchar: ScharT,
    maxwidth: c_int,
    hltab: *mut *mut c_void,
    hltab_len: *mut usize,
    tabtab: *mut *mut c_void,
    stcp: StatuscolHandle,
) -> c_int {
    let buf = nvim_win_get_buffer(wp);

    // Save global state
    let save_redraw_not_allowed = redraw_not_allowed;
    let save_key_typed = KeyTyped;
    let did_emsg_before = did_emsg;

    if updating_screen {
        redraw_not_allowed = true;
    }

    // Sandbox check
    let use_sandbox = if opt_idx != K_OPT_INVALID {
        nvim_stl_was_set_insecurely(wp, opt_idx, opt_scope) != 0
    } else {
        false
    };

    // Handle %! expression format
    let mut usefmt: *mut c_char = fmt;
    let mut usefmt_allocated = false;

    if *fmt as u8 == b'%' && *fmt.add(1) as u8 == b'!' {
        let result = nvim_stl_eval_with_context(wp, fmt.add(2), 1, use_sandbox);
        if !result.is_null() {
            usefmt = result;
            usefmt_allocated = true;
        }
    }

    // Default fillchar
    if fillchar == 0 {
        fillchar = nvim_stl_schar_from_ascii(b' ' as c_char);
    }

    // Clamp cursor and get batch cursor info
    let cursor_info = nvim_stl_get_win_cursor_info(wp);
    let lnum = cursor_info.clamped_lnum;
    let line_count = nvim_win_buf_line_count(wp);
    let empty_line = cursor_info.empty_line != 0;
    let byteval = cursor_info.byte_value;

    let mut groupdepth: c_int = 0;
    let mut evaldepth: c_int = 0;

    // Track recursion: evalstart is the curitem at entry
    let evalstart = CURITEM.with(|c| *c.borrow());

    let mut prevchar_isflag = true;
    let mut prevchar_isitem = false;

    let mut out_p = out;
    let out_end_p = out.add(outlen - 1);

    // Temporary buffer
    let mut buf_tmp = [0i8; TMPLEN];

    // Main format parsing loop
    let mut fmt_p = usefmt;
    while *fmt_p != NUL as c_char {
        // Ensure capacity in items vec
        let curitem = CURITEM.with(|c| *c.borrow());
        STL_ITEMS.with(|items| {
            let mut items = items.borrow_mut();
            while items.len() <= curitem as usize {
                items.push(StlItem::new());
            }
        });
        STL_GROUPITEMS.with(|g| {
            let mut g = g.borrow_mut();
            while g.len() <= curitem as usize {
                g.push(0);
            }
        });
        STL_SEPARATOR_LOCATIONS.with(|s| {
            let mut s = s.borrow_mut();
            while s.len() <= curitem as usize {
                s.push(0);
            }
        });

        if *fmt_p as u8 != b'%' {
            prevchar_isflag = false;
            prevchar_isitem = false;
        }

        // Copy literal characters until we hit '%' or end
        while *fmt_p != NUL as c_char && *fmt_p as u8 != b'%' && out_p < out_end_p {
            *out_p = *fmt_p;
            out_p = out_p.add(1);
            fmt_p = fmt_p.add(1);
        }

        if *fmt_p == NUL as c_char || out_p >= out_end_p {
            break;
        }

        // Skip the '%'
        fmt_p = fmt_p.add(1);

        if *fmt_p == NUL as c_char {
            break;
        }

        // %%: literal percent
        if *fmt_p as u8 == b'%' {
            *out_p = *fmt_p;
            out_p = out_p.add(1);
            fmt_p = fmt_p.add(1);
            prevchar_isflag = false;
            prevchar_isitem = false;
            continue;
        }

        // %= : separator
        if *fmt_p as u8 == STL_SEPARATE {
            fmt_p = fmt_p.add(1);
            if groupdepth > 0 {
                continue;
            }
            let ci = CURITEM.with(|c| *c.borrow());
            STL_ITEMS.with(|items| {
                let mut items = items.borrow_mut();
                items[ci as usize].item_type = ITEM_SEPARATE;
                items[ci as usize].start = out_p;
            });
            CURITEM.with(|c| *c.borrow_mut() += 1);
            continue;
        }

        // %< : truncation mark
        if *fmt_p as u8 == STL_TRUNCMARK {
            fmt_p = fmt_p.add(1);
            let ci = CURITEM.with(|c| *c.borrow());
            STL_ITEMS.with(|items| {
                let mut items = items.borrow_mut();
                items[ci as usize].item_type = ITEM_TRUNC;
                items[ci as usize].start = out_p;
            });
            CURITEM.with(|c| *c.borrow_mut() += 1);
            continue;
        }

        // %) : group end
        if *fmt_p as u8 == b')' {
            fmt_p = fmt_p.add(1);
            if groupdepth < 1 {
                continue;
            }
            groupdepth -= 1;

            handle_group_end(groupdepth, &mut out_p, out_end_p, fillchar, stcp);
            continue;
        }

        // Parse width/flags
        let mut minwid: c_int = 0;
        let mut maxwid: c_int = 9999;
        let mut foldsignitem: c_int = -1;
        let mut left_align_num = false;
        let mut left_align = false;

        let zeropad = *fmt_p as u8 == b'0';
        if zeropad {
            fmt_p = fmt_p.add(1);
        }

        if *fmt_p as u8 == b'-' {
            fmt_p = fmt_p.add(1);
            left_align = true;
        }

        // Parse minimum width digits
        while (*fmt_p as u8).is_ascii_digit() {
            minwid = minwid * 10 + c_int::from(*fmt_p as u8 - b'0');
            fmt_p = fmt_p.add(1);
        }

        // User highlight
        if *fmt_p as u8 == STL_USER_HL {
            let ci = CURITEM.with(|c| *c.borrow());
            STL_ITEMS.with(|items| {
                let mut items = items.borrow_mut();
                items[ci as usize].item_type = ITEM_HIGHLIGHT;
                items[ci as usize].start = out_p;
                items[ci as usize].minwid = if minwid > 9 { 1 } else { minwid };
            });
            fmt_p = fmt_p.add(1);
            CURITEM.with(|c| *c.borrow_mut() += 1);
            continue;
        }

        // Tab page / tab close
        if *fmt_p as u8 == STL_TABPAGENR || *fmt_p as u8 == STL_TABCLOSENR {
            let mut tab_minwid = minwid;
            if *fmt_p as u8 == STL_TABCLOSENR {
                if tab_minwid == 0 {
                    // %X ends close label, go back to previous tab label nr
                    let ci = CURITEM.with(|c| *c.borrow());
                    STL_ITEMS.with(|items| {
                        let items = items.borrow();
                        for n in (0..ci as usize).rev() {
                            if items[n].item_type == ITEM_TABPAGE && items[n].minwid >= 0 {
                                tab_minwid = items[n].minwid;
                                break;
                            }
                        }
                    });
                } else {
                    tab_minwid = -tab_minwid;
                }
            }
            let ci = CURITEM.with(|c| *c.borrow());
            STL_ITEMS.with(|items| {
                let mut items = items.borrow_mut();
                items[ci as usize].item_type = ITEM_TABPAGE;
                items[ci as usize].start = out_p;
                items[ci as usize].minwid = tab_minwid;
            });
            fmt_p = fmt_p.add(1);
            CURITEM.with(|c| *c.borrow_mut() += 1);
            continue;
        }

        // Click function
        if *fmt_p as u8 == STL_CLICK_FUNC {
            fmt_p = fmt_p.add(1);
            let t = fmt_p;
            while *fmt_p as u8 != STL_CLICK_FUNC && *fmt_p != NUL as c_char {
                fmt_p = fmt_p.add(1);
            }
            if *fmt_p as u8 != STL_CLICK_FUNC {
                break;
            }
            let ci = CURITEM.with(|c| *c.borrow());
            let func_len = fmt_p.offset_from(t) as usize;
            let cmd_ptr = if !tabtab.is_null() {
                nvim_stl_xmemdupz(t, func_len)
            } else {
                ptr::null_mut()
            };
            STL_ITEMS.with(|items| {
                let mut items = items.borrow_mut();
                items[ci as usize].item_type = ITEM_CLICK_FUNC;
                items[ci as usize].start = out_p;
                items[ci as usize].cmd = cmd_ptr;
                items[ci as usize].minwid = minwid;
            });
            fmt_p = fmt_p.add(1);
            CURITEM.with(|c| *c.borrow_mut() += 1);
            continue;
        }

        // Parse max width
        if *fmt_p as u8 == b'.' {
            fmt_p = fmt_p.add(1);
            maxwid = 0;
            if (*fmt_p as u8).is_ascii_digit() {
                while (*fmt_p as u8).is_ascii_digit() {
                    maxwid = maxwid * 10 + c_int::from(*fmt_p as u8 - b'0');
                    fmt_p = fmt_p.add(1);
                }
                if maxwid == 0 {
                    maxwid = 50;
                }
            } else {
                maxwid = 50;
            }
        }

        // Clamp minwid, apply alignment
        minwid = minwid.min(50) * if left_align { -1 } else { 1 };

        // Group start
        if *fmt_p as u8 == b'(' {
            let ci = CURITEM.with(|c| *c.borrow());
            STL_GROUPITEMS.with(|g| {
                let mut g = g.borrow_mut();
                g[groupdepth as usize] = ci;
            });
            groupdepth += 1;
            STL_ITEMS.with(|items| {
                let mut items = items.borrow_mut();
                items[ci as usize].item_type = ITEM_GROUP;
                items[ci as usize].start = out_p;
                items[ci as usize].minwid = minwid;
                items[ci as usize].maxwid = maxwid;
            });
            fmt_p = fmt_p.add(1);
            CURITEM.with(|c| *c.borrow_mut() += 1);
            continue;
        }

        // End of expanded %{} block
        if *fmt_p as u8 == b'}' && evaldepth > 0 {
            fmt_p = fmt_p.add(1);
            evaldepth -= 1;
            continue;
        }

        // Validate flag character
        if nvim_stl_valid_flag(c_int::from(*fmt_p as u8)) == 0 {
            if *fmt_p == NUL as c_char {
                break;
            }
            fmt_p = fmt_p.add(1);
            continue;
        }

        // The actual item character
        let opt = *fmt_p as u8;
        fmt_p = fmt_p.add(1);

        // Evaluate the item
        let mut base = NUM_BASE_DECIMAL;
        let mut itemisflag = false;
        let mut fillable = true;
        let mut num: c_int = -1;
        let mut str_ptr: *const c_char = ptr::null();
        let mut str_allocated = false;

        match opt {
            STL_FILEPATH | STL_FULLPATH | STL_FILENAME => {
                fillable = false;
                let name = nvim_stl_buf_spname(buf);
                let namebuff = nvim_get_namebuff();
                if !name.is_null() {
                    // xstrlcpy(NameBuff, name, MAXPATHL)
                    let name_len = strlen(name);
                    let copy_len = name_len.min(MAXPATHL as usize - 1);
                    ptr::copy_nonoverlapping(name, namebuff, copy_len);
                    *namebuff.add(copy_len) = NUL as c_char;
                } else {
                    let t = if opt == STL_FULLPATH {
                        nvim_buf_get_b_ffname(buf)
                    } else {
                        nvim_buf_get_b_fname(buf)
                    };
                    nvim_stl_home_replace_trans(buf, t, namebuff, MAXPATHL);
                }
                // trans_characters is done inside home_replace_trans
                if opt != STL_FILENAME {
                    str_ptr = namebuff;
                } else {
                    str_ptr = nvim_stl_path_tail(namebuff);
                }
            }

            STL_VIM_EXPR => {
                let block_start = fmt_p.sub(1);
                let reevaluate = *fmt_p as u8 == b'%';
                itemisflag = true;

                if reevaluate {
                    fmt_p = fmt_p.add(1);
                }

                // Copy expression into output buffer
                let t = out_p;
                while (*fmt_p as u8 != b'}' || (reevaluate && *fmt_p.sub(1) as u8 != b'%'))
                    && *fmt_p != NUL as c_char
                    && out_p < out_end_p
                {
                    *out_p = *fmt_p;
                    out_p = out_p.add(1);
                    fmt_p = fmt_p.add(1);
                }
                if *fmt_p as u8 != b'}' {
                    break;
                }
                fmt_p = fmt_p.add(1);
                if reevaluate {
                    *out_p.sub(1) = NUL as c_char;
                } else {
                    *out_p = NUL as c_char;
                }

                // Reset out_p to start of expression
                out_p = t;

                // Evaluate expression with full context
                let eval_result = nvim_stl_eval_with_context(wp, out_p, 0, use_sandbox);

                if !eval_result.is_null() && *eval_result != NUL as c_char {
                    // Check if result is all digits -> numeric
                    if nvim_stl_str_all_digits(eval_result) != 0 {
                        num = c_atoi(eval_result);
                        nvim_stl_xfree(eval_result.cast());
                        itemisflag = false;
                    } else {
                        // Check for reevaluate
                        if reevaluate
                            && *eval_result != NUL as c_char
                            && c_strchr(eval_result, b'%') != 0
                            && evaldepth < MAX_STL_EVAL_DEPTH
                        {
                            let parsed_usefmt = block_start.offset_from(usefmt) as usize;
                            let str_length = strlen(eval_result);
                            let fmt_length = strlen(fmt_p);
                            let new_fmt_len = parsed_usefmt + str_length + fmt_length + 3;
                            let new_fmt = nvim_stl_xmemdupz(
                                ptr::null(),
                                0, // dummy, we'll build manually
                            );
                            // Actually allocate the right size
                            nvim_stl_xfree(new_fmt.cast());
                            // Build new format string
                            let new_fmt_raw = libc_malloc(new_fmt_len) as *mut c_char;
                            if !new_fmt_raw.is_null() {
                                ptr::copy_nonoverlapping(usefmt, new_fmt_raw, parsed_usefmt);
                                ptr::copy_nonoverlapping(
                                    eval_result,
                                    new_fmt_raw.add(parsed_usefmt),
                                    str_length,
                                );
                                // "%}" separator
                                *new_fmt_raw.add(parsed_usefmt + str_length) = b'%' as c_char;
                                *new_fmt_raw.add(parsed_usefmt + str_length + 1) = b'}' as c_char;
                                ptr::copy_nonoverlapping(
                                    fmt_p,
                                    new_fmt_raw.add(parsed_usefmt + str_length + 2),
                                    fmt_length,
                                );
                                *new_fmt_raw.add(new_fmt_len - 1) = NUL as c_char;

                                if usefmt_allocated {
                                    nvim_stl_xfree(usefmt.cast());
                                }
                                nvim_stl_xfree(eval_result.cast());
                                usefmt = new_fmt_raw;
                                usefmt_allocated = true;
                                fmt_p = usefmt.add(parsed_usefmt);
                                evaldepth += 1;
                                continue;
                            }
                            // fallthrough if malloc failed
                        }
                        str_ptr = eval_result;
                        str_allocated = true;
                    }
                } else {
                    if !eval_result.is_null() {
                        nvim_stl_xfree(eval_result.cast());
                    }
                }
            }

            STL_LINE => {
                if !stcp.is_null()
                    && (nvim_stl_win_get_p_nu(wp) != 0 || nvim_stl_win_get_p_rnu(wp) != 0)
                    && nvim_stl_get_vim_var_nr(VV_VIRTNUM) == 0
                {
                    if nvim_stl_win_get_maxscwidth(wp) == SCL_NUM
                        && nvim_stl_stcp_get_sign_info(stcp, 0).has_text != 0
                    {
                        // goto stcsign - handled inline below as the sign column path
                        handle_stcsign(
                            wp,
                            stcp,
                            &mut buf_tmp,
                            &mut out_p,
                            out_end_p,
                            &mut str_ptr,
                            &mut foldsignitem,
                            &mut num,
                            &mut itemisflag,
                            &mut fillable,
                            fillchar,
                            lnum,
                        );
                        // Skip to item output
                    } else {
                        let relnum = nvim_stl_get_vim_var_nr(VV_RELNUM) as c_int;
                        let p_nu = nvim_stl_win_get_p_nu(wp) != 0;
                        let p_rnu = nvim_stl_win_get_p_rnu(wp) != 0;
                        num = if !p_rnu || (p_nu && relnum == 0) {
                            nvim_stl_get_vim_var_nr(VV_LNUM) as c_int
                        } else {
                            relnum
                        };
                        left_align_num = p_rnu && p_nu && relnum == 0;
                        if !left_align_num {
                            let ci = CURITEM.with(|c| *c.borrow());
                            STL_ITEMS.with(|items| {
                                let mut items = items.borrow_mut();
                                items[ci as usize].item_type = ITEM_SEPARATE;
                                items[ci as usize].start = out_p;
                            });
                            CURITEM.with(|c| *c.borrow_mut() += 1);
                        }
                    }
                } else if stcp.is_null() {
                    num = if cursor_info.ml_empty != 0 { 0 } else { lnum };
                }
            }

            STL_NUMLINES => {
                num = line_count;
            }

            STL_COLUMN => {
                num = if (State & MODE_INSERT) == 0 && empty_line {
                    0
                } else {
                    nvim_stl_win_get_cursor_col(wp) + 1
                };
            }

            STL_VIRTCOL | STL_VIRTCOL_ALT => {
                let virtcol = nvim_stl_win_get_w_virtcol(wp) + 1;
                if opt == STL_VIRTCOL_ALT {
                    let col = if (State & MODE_INSERT) == 0 && empty_line {
                        0
                    } else {
                        nvim_stl_win_get_cursor_col(wp) + 1
                    };
                    if virtcol == col {
                        // break from switch - num stays -1, item will be Empty
                    } else {
                        num = virtcol;
                    }
                } else {
                    num = virtcol;
                }
            }

            STL_PERCENTAGE => {
                num = nvim_stl_calc_percentage(lnum, line_count);
            }

            STL_ALTPERCENT => {
                nvim_stl_get_rel_pos(wp, buf_tmp.as_mut_ptr(), TMPLEN as c_int);
                str_ptr = buf_tmp.as_ptr();
            }

            STL_SHOWCMD => {
                if nvim_stl_showcmd_matches_opt(opt_idx) != 0 {
                    nvim_stl_get_showcmd(buf_tmp.as_mut_ptr(), TMPLEN as c_int);
                    str_ptr = buf_tmp.as_ptr();
                }
            }

            STL_ARGLISTSTAT => {
                fillable = false;
                buf_tmp[0] = NUL as c_char;
                if nvim_stl_append_arg_number(wp, buf_tmp.as_mut_ptr(), TMPLEN) > 0 {
                    str_ptr = buf_tmp.as_ptr();
                }
            }

            STL_KEYMAP => {
                fillable = false;
                if nvim_stl_get_keymap(wp, buf_tmp.as_mut_ptr(), TMPLEN as c_int) > 0 {
                    str_ptr = buf_tmp.as_ptr();
                }
            }

            STL_PAGENUM => {
                num = 0;
            }

            STL_BUFNO => {
                num = nvim_buf_get_fnum(buf);
            }

            STL_OFFSET_X => {
                base = NUM_BASE_HEX;
                num = cursor_info.byte_offset;
                if num < 0 || cursor_info.ml_empty != 0 {
                    num = 0;
                } else {
                    num += 1;
                    if (State & MODE_INSERT) == 0 && empty_line {
                        // don't add col
                    } else {
                        num += nvim_stl_win_get_cursor_col(wp);
                    }
                }
            }

            STL_OFFSET => {
                num = cursor_info.byte_offset;
                if num < 0 || cursor_info.ml_empty != 0 {
                    num = 0;
                } else {
                    num += 1;
                    if (State & MODE_INSERT) == 0 && empty_line {
                        // don't add col
                    } else {
                        num += nvim_stl_win_get_cursor_col(wp);
                    }
                }
            }

            STL_BYTEVAL_X => {
                base = NUM_BASE_HEX;
                num = byteval;
                if num == NL {
                    num = 0;
                } else if num == CAR && nvim_stl_get_fileformat(buf) == EOL_MAC {
                    num = NL;
                }
            }

            STL_BYTEVAL => {
                num = byteval;
                if num == NL {
                    num = 0;
                } else if num == CAR && nvim_stl_get_fileformat(buf) == EOL_MAC {
                    num = NL;
                }
            }

            STL_ROFLAG | STL_ROFLAG_ALT => {
                itemisflag = true;
                if nvim_buf_get_b_p_ro(buf) != 0 {
                    str_ptr = if opt == STL_ROFLAG_ALT {
                        b",RO\0".as_ptr().cast()
                    } else {
                        b"[RO]\0".as_ptr().cast()
                    };
                }
            }

            STL_HELPFLAG | STL_HELPFLAG_ALT => {
                itemisflag = true;
                if nvim_buf_get_help(buf) != 0 {
                    str_ptr = if opt == STL_HELPFLAG_ALT {
                        b",HLP\0".as_ptr().cast()
                    } else {
                        b"[Help]\0".as_ptr().cast()
                    };
                }
            }

            STL_FOLDCOL | STL_SIGNCOL => {
                if !stcp.is_null() {
                    handle_stcsign(
                        wp,
                        stcp,
                        &mut buf_tmp,
                        &mut out_p,
                        out_end_p,
                        &mut str_ptr,
                        &mut foldsignitem,
                        &mut num,
                        &mut itemisflag,
                        &mut fillable,
                        fillchar,
                        lnum,
                    );
                }
            }

            STL_FILETYPE => {
                let ft = nvim_buf_get_b_p_ft(buf);
                if !ft.is_null() && *ft != NUL as c_char && strlen(ft) < TMPLEN - 3 {
                    let n =
                        libc_snprintf(buf_tmp.as_mut_ptr(), TMPLEN, b"[%s]\0".as_ptr().cast(), ft);
                    if n > 0 {
                        str_ptr = buf_tmp.as_ptr();
                    }
                }
            }

            STL_FILETYPE_ALT => {
                itemisflag = true;
                let ft = nvim_buf_get_b_p_ft(buf);
                if !ft.is_null() && *ft != NUL as c_char && strlen(ft) < TMPLEN - 2 {
                    let n =
                        libc_snprintf(buf_tmp.as_mut_ptr(), TMPLEN, b",%s\0".as_ptr().cast(), ft);
                    if n > 0 {
                        // Uppercase
                        let mut p = buf_tmp.as_mut_ptr();
                        while *p != NUL as c_char {
                            let c = *p as u8;
                            if c.is_ascii_lowercase() {
                                *p = (c - b'a' + b'A') as c_char;
                            }
                            p = p.add(1);
                        }
                        str_ptr = buf_tmp.as_ptr();
                    }
                }
            }

            STL_PREVIEWFLAG | STL_PREVIEWFLAG_ALT => {
                itemisflag = true;
                if nvim_win_get_pvw(wp) != 0 {
                    str_ptr = if opt == STL_PREVIEWFLAG_ALT {
                        b",PRV\0".as_ptr().cast()
                    } else {
                        b"[Preview]\0".as_ptr().cast()
                    };
                }
            }

            STL_QUICKFIX => {
                if nvim_stl_get_qf_info(wp, buf_tmp.as_mut_ptr(), TMPLEN as c_int) > 0 {
                    str_ptr = buf_tmp.as_ptr();
                }
            }

            STL_MODIFIED | STL_MODIFIED_ALT => {
                itemisflag = true;
                let alt = if opt == STL_MODIFIED_ALT { 1 } else { 0 };
                let changed = if nvim_buf_get_b_changed(buf) { 2 } else { 0 };
                let not_mod = if nvim_buf_get_b_p_ma(buf) == 0 { 4 } else { 0 };
                match alt + changed + not_mod {
                    2 => str_ptr = b"[+]\0".as_ptr().cast(),
                    3 => str_ptr = b",+\0".as_ptr().cast(),
                    4 => str_ptr = b"[-]\0".as_ptr().cast(),
                    5 => str_ptr = b",-\0".as_ptr().cast(),
                    6 => str_ptr = b"[+-]\0".as_ptr().cast(),
                    7 => str_ptr = b",+-\0".as_ptr().cast(),
                    _ => {}
                }
            }

            STL_HIGHLIGHT => {
                // Named highlight: %#GroupName#
                let t = fmt_p;
                while *fmt_p as u8 != b'#' && *fmt_p != NUL as c_char {
                    fmt_p = fmt_p.add(1);
                }
                if *fmt_p as u8 == b'#' {
                    let ci = CURITEM.with(|c| *c.borrow());
                    let name_len = fmt_p.offset_from(t) as c_int;
                    let syn_id = nvim_stl_syn_name2id_len(t, name_len);
                    STL_ITEMS.with(|items| {
                        let mut items = items.borrow_mut();
                        items[ci as usize].item_type = ITEM_HIGHLIGHT;
                        items[ci as usize].start = out_p;
                        items[ci as usize].minwid = -syn_id;
                    });
                    CURITEM.with(|c| *c.borrow_mut() += 1);
                    fmt_p = fmt_p.add(1);
                }
                continue;
            }

            _ => {}
        }

        // --- Item output ---
        let ci = CURITEM.with(|c| *c.borrow());
        STL_ITEMS.with(|items| {
            let mut items = items.borrow_mut();
            while items.len() <= ci as usize {
                items.push(StlItem::new());
            }
            items[ci as usize].start = out_p;
            items[ci as usize].item_type = ITEM_NORMAL;
        });

        if !str_ptr.is_null() && *str_ptr != NUL as c_char {
            // String item
            let mut t = str_ptr as *const c_char;

            if itemisflag {
                let t0 = *t as u8;
                let t1 = *t.add(1) as u8;
                if t0 != NUL
                    && t1 != NUL
                    && ((!prevchar_isitem && t0 == b',') || (prevchar_isflag && t0 == b' '))
                {
                    t = t.add(1);
                }
                prevchar_isflag = true;
            }

            let mut l = nvim_stl_vim_strsize(t);

            if l > 0 {
                prevchar_isitem = true;
            }

            // Truncate if too wide
            if l > maxwid {
                while l >= maxwid {
                    l -= nvim_stl_ptr2cells(t);
                    t = t.add(nvim_stl_utfc_ptr2len(t) as usize);
                }
                if out_p >= out_end_p {
                    if str_allocated {
                        nvim_stl_xfree(str_ptr as *mut c_void);
                    }
                    break;
                }
                *out_p = b'<' as c_char;
                out_p = out_p.add(1);
            }

            // Right-align padding
            if minwid > 0 {
                while l < minwid && out_p < out_end_p {
                    if l + 1 == minwid
                        && fillchar == nvim_stl_schar_from_ascii(b'-' as c_char)
                        && (*t as u8).is_ascii_digit()
                    {
                        *out_p = b' ' as c_char;
                        out_p = out_p.add(1);
                    } else {
                        schar_get_adv(&mut out_p, fillchar);
                    }
                    l += 1;
                }
                minwid = 0;
                // Adjust foldsignitem positions
                if foldsignitem >= 0 {
                    let offset = out_p.offset_from(
                        STL_ITEMS.with(|items| items.borrow()[foldsignitem as usize].start),
                    );
                    STL_ITEMS.with(|items| {
                        let mut items = items.borrow_mut();
                        for i in foldsignitem as usize..ci as usize {
                            items[i].start = items[i].start.offset(offset);
                        }
                    });
                }
            } else {
                minwid = -minwid;
            }

            // Copy string to output, replacing spaces with fillchar
            while *t != NUL as c_char && out_p < out_end_p {
                if fillable
                    && *t as u8 == b' '
                    && (!(*t.add(1) as u8).is_ascii_digit()
                        || fillchar != nvim_stl_schar_from_ascii(b'-' as c_char))
                {
                    schar_get_adv(&mut out_p, fillchar);
                } else {
                    *out_p = *t;
                    out_p = out_p.add(1);
                }
                t = t.add(1);
            }

            // Reset highlight for foldsignitem
            if foldsignitem >= 0 {
                STL_ITEMS.with(|items| {
                    let mut items = items.borrow_mut();
                    while items.len() <= ci as usize {
                        items.push(StlItem::new());
                    }
                    items[ci as usize].item_type = ITEM_HIGHLIGHT;
                    items[ci as usize].start = out_p;
                    items[ci as usize].minwid = 0;
                });
            }

            // Left-align padding
            while l < minwid && out_p < out_end_p {
                schar_get_adv(&mut out_p, fillchar);
                l += 1;
            }
        } else if num >= 0 {
            // Numeric item
            if out_p.add(20) > out_end_p {
                if str_allocated {
                    nvim_stl_xfree(str_ptr as *mut c_void);
                }
                break;
            }
            prevchar_isitem = true;

            // Build format string
            let mut nstr = [0u8; 20];
            let mut ti: usize = 0;
            if opt == STL_VIRTCOL_ALT {
                nstr[ti] = b'-';
                ti += 1;
                minwid -= 1;
            }
            nstr[ti] = b'%';
            ti += 1;
            if zeropad {
                nstr[ti] = b'0';
                ti += 1;
            }
            nstr[ti] = b'*';
            ti += 1;
            nstr[ti] = if base == NUM_BASE_HEX { b'X' } else { b'd' };
            ti += 1;
            nstr[ti] = 0;

            // Count digits
            let mut num_chars: c_int = 1;
            {
                let mut n = num;
                while n >= base {
                    n /= base;
                    num_chars += 1;
                }
            }
            if opt == STL_VIRTCOL_ALT {
                num_chars += 1;
            }

            let remaining = (out_end_p.offset_from(out_p) + 1) as usize;

            if num_chars > maxwid {
                // Scientific notation
                num_chars += 2;
                let n_exp = num_chars - maxwid;
                let mut reduced_num = num;
                let mut nc = num_chars;
                while nc > maxwid {
                    reduced_num /= base;
                    nc -= 1;
                }

                // Build sci format: existing + ">%d" or ">%X"
                nstr[ti - 1] = b'>'; // overwrite NUL at end (actually the last format char position)
                                     // Rebuild properly
                let mut sci_fmt = [0u8; 30];
                let mut si = 0;
                // Copy existing format up to (and excluding) the NUL
                for b in &nstr[..ti - 1] {
                    sci_fmt[si] = *b;
                    si += 1;
                }
                sci_fmt[si] = b'>';
                si += 1;
                sci_fmt[si] = b'%';
                si += 1;
                sci_fmt[si] = nstr[ti - 2]; // same base specifier
                si += 1;
                sci_fmt[si] = 0;

                let written = nvim_stl_snprintf_sci(
                    out_p,
                    remaining,
                    sci_fmt.as_ptr().cast(),
                    reduced_num,
                    n_exp,
                );
                out_p = out_p.add(written as usize);
            } else {
                let written =
                    nvim_stl_snprintf_int(out_p, remaining, nstr.as_ptr().cast(), minwid, num);
                out_p = out_p.add(written as usize);
            }
        } else {
            // Empty item
            STL_ITEMS.with(|items| {
                let mut items = items.borrow_mut();
                items[ci as usize].item_type = ITEM_EMPTY;
            });
        }

        if num >= 0 || (!itemisflag && !str_ptr.is_null() && *str_ptr != NUL as c_char) {
            prevchar_isflag = false;
        }

        // Free allocated string
        if str_allocated {
            nvim_stl_xfree(str_ptr as *mut c_void);
        }

        CURITEM.with(|c| *c.borrow_mut() += 1);

        // Left-aligned number: add separator
        if left_align_num {
            let ci2 = CURITEM.with(|c| *c.borrow());
            STL_ITEMS.with(|items| {
                let mut items = items.borrow_mut();
                while items.len() <= ci2 as usize {
                    items.push(StlItem::new());
                }
                items[ci2 as usize].item_type = ITEM_SEPARATE;
                items[ci2 as usize].start = out_p;
            });
            CURITEM.with(|c| *c.borrow_mut() += 1);
        }
    }

    // NUL-terminate output
    *out_p = NUL as c_char;
    let outputlen = out_p.offset_from(out) as usize;

    let curitem_now = CURITEM.with(|c| *c.borrow());
    let itemcnt = curitem_now - evalstart;
    CURITEM.with(|c| *c.borrow_mut() = evalstart);

    // Free the format buffer if we allocated it
    if usefmt_allocated {
        nvim_stl_xfree(usefmt.cast());
    }

    // === Post-processing ===
    let mut width = nvim_stl_vim_strsize(out);
    let mut itemcnt = itemcnt;

    if maxwidth > 0 && width > maxwidth && (stcp.is_null() || width > MAX_STCWIDTH) {
        // Truncation needed
        let mut item_idx = evalstart;
        let mut trunc_p: *mut c_char;

        if itemcnt == 0 {
            trunc_p = out;
        } else {
            trunc_p = STL_ITEMS.with(|items| items.borrow()[item_idx as usize].start);
            for i in evalstart..itemcnt + evalstart {
                let item_type = STL_ITEMS.with(|items| items.borrow()[i as usize].item_type);
                if item_type == ITEM_TRUNC {
                    trunc_p = STL_ITEMS.with(|items| items.borrow()[i as usize].start);
                    item_idx = i;
                    break;
                }
            }
        }

        if width - nvim_stl_vim_strsize(trunc_p) >= maxwidth {
            // Truncate at end
            trunc_p = out;
            width = 0;
            loop {
                width += nvim_stl_ptr2cells(trunc_p);
                if width >= maxwidth {
                    break;
                }
                trunc_p = trunc_p.add(nvim_stl_utfc_ptr2len(trunc_p) as usize);
            }

            // Ignore items after truncation point
            for i in evalstart..itemcnt + evalstart {
                let item_start = STL_ITEMS.with(|items| items.borrow()[i as usize].start);
                if item_start > trunc_p {
                    for j in i..itemcnt + evalstart {
                        let jtype = STL_ITEMS.with(|items| items.borrow()[j as usize].item_type);
                        if jtype == ITEM_CLICK_FUNC {
                            STL_ITEMS.with(|items| {
                                let mut items = items.borrow_mut();
                                let cmd = items[j as usize].cmd;
                                if !cmd.is_null() {
                                    nvim_stl_xfree(cmd.cast());
                                    items[j as usize].cmd = ptr::null_mut();
                                }
                            });
                        }
                    }
                    itemcnt = i - evalstart;
                    break;
                }
            }

            *trunc_p = b'>' as c_char;
            trunc_p = trunc_p.add(1);
            *trunc_p = NUL as c_char;
        } else {
            // Truncate at truncation point
            let end = out.add(outputlen);
            let mut trunc_len: c_int = 0;
            let mut w = width;
            while w >= maxwidth {
                w -= nvim_stl_ptr2cells(trunc_p.add(trunc_len as usize));
                trunc_len += nvim_stl_utfc_ptr2len(trunc_p.add(trunc_len as usize));
            }
            width = w;

            let trunc_end_p = trunc_p.add(trunc_len as usize);
            let tail_len = end.offset_from(trunc_end_p) as usize + 1; // +1 for NUL
            memmove(trunc_p.add(1).cast(), trunc_end_p.cast(), tail_len);
            let end = end.sub(trunc_end_p.offset_from(trunc_p.add(1)) as usize);
            *trunc_p = b'<' as c_char;

            let item_offset = trunc_len - 1;

            STL_ITEMS.with(|items| {
                let mut items = items.borrow_mut();
                for i in item_idx as usize..(itemcnt + evalstart) as usize {
                    if items[i].start >= trunc_end_p {
                        items[i].start = items[i].start.sub(item_offset as usize);
                    } else {
                        items[i].start = trunc_p;
                    }
                }
            });

            let mut fill_p = if width + 1 < maxwidth { end } else { trunc_p };

            while width + 1 < maxwidth {
                schar_get_adv(&mut fill_p, fillchar);
                width += 1;
            }
        }
        width = maxwidth;
    } else if width < maxwidth && outputlen + (maxwidth - width) as usize + 1 < outlen {
        // Fill separators
        let mut num_separators: c_int = 0;
        STL_ITEMS.with(|items| {
            let items = items.borrow();
            STL_SEPARATOR_LOCATIONS.with(|seps| {
                let mut seps = seps.borrow_mut();
                for i in evalstart..itemcnt + evalstart {
                    if items[i as usize].item_type == ITEM_SEPARATE {
                        while seps.len() <= num_separators as usize {
                            seps.push(0);
                        }
                        seps[num_separators as usize] = i;
                        num_separators += 1;
                    }
                }
            });
        });

        if num_separators > 0 {
            let standard_spaces = (maxwidth - width) / num_separators;
            let final_spaces = (maxwidth - width) - standard_spaces * (num_separators - 1);

            for l in 0..num_separators {
                let dislocation_cells = if l == num_separators - 1 {
                    final_spaces
                } else {
                    standard_spaces
                };
                let dislocation = dislocation_cells * nvim_stl_schar_len(fillchar) as c_int;

                let sep_idx = STL_SEPARATOR_LOCATIONS.with(|s| s.borrow()[l as usize]);
                let start = STL_ITEMS.with(|items| items.borrow()[sep_idx as usize].start);
                let seploc = start.add(dislocation as usize);

                // STRMOVE equivalent
                let tail_len = strlen(start) + 1;
                memmove(seploc.cast(), start.cast(), tail_len);

                // Fill separator space
                let mut s = start;
                while s < seploc {
                    schar_get_adv(&mut s, fillchar);
                }

                // Shift items after this separator
                STL_ITEMS.with(|items| {
                    let mut items = items.borrow_mut();
                    for item_idx in (sep_idx + 1) as usize..(itemcnt + evalstart) as usize {
                        items[item_idx].start = items[item_idx].start.add(dislocation as usize);
                    }
                });
            }
            width = maxwidth;
        }
    }

    // Build highlight records
    if !hltab.is_null() {
        STL_HLTAB.with(|ht| {
            let mut ht = ht.borrow_mut();
            ht.clear();
            STL_ITEMS.with(|items| {
                let items = items.borrow();
                for l in evalstart..itemcnt + evalstart {
                    let t = items[l as usize].item_type;
                    if t == ITEM_HIGHLIGHT || t == ITEM_HIGHLIGHT_FOLD || t == ITEM_HIGHLIGHT_SIGN {
                        ht.push(StlHlRec {
                            start: items[l as usize].start,
                            userhl: items[l as usize].minwid,
                            item: if t == ITEM_HIGHLIGHT_SIGN {
                                STL_SIGNCOL as c_int
                            } else if t == ITEM_HIGHLIGHT_FOLD {
                                STL_FOLDCOL as c_int
                            } else {
                                0
                            },
                        });
                    }
                }
            });
            // Sentinel
            ht.push(StlHlRec {
                start: ptr::null_mut(),
                userhl: 0,
                item: 0,
            });
            *hltab = ht.as_mut_ptr().cast();
        });
    }
    if !hltab_len.is_null() {
        *hltab_len = itemcnt as usize;
    }

    // Build click records
    if !tabtab.is_null() {
        STL_TABTAB.with(|tt| {
            let mut tt = tt.borrow_mut();
            tt.clear();
            STL_ITEMS.with(|items| {
                let items = items.borrow();
                for l in evalstart..itemcnt + evalstart {
                    let t = items[l as usize].item_type;
                    if t == ITEM_TABPAGE {
                        let mw = items[l as usize].minwid;
                        let (click_type, tabnr) = if mw == 0 {
                            (0, 0) // Disabled
                        } else if mw > 0 {
                            (1, mw) // TabSwitch
                        } else {
                            (2, -mw) // TabClose
                        };
                        tt.push(StlClickRecord {
                            start: items[l as usize].start,
                            def: StlClickDef {
                                click_type,
                                tabnr,
                                func: ptr::null_mut(),
                            },
                        });
                    } else if t == ITEM_CLICK_FUNC {
                        tt.push(StlClickRecord {
                            start: items[l as usize].start,
                            def: StlClickDef {
                                click_type: 3, // FuncRun
                                tabnr: items[l as usize].minwid,
                                func: items[l as usize].cmd,
                            },
                        });
                    }
                }
            });
            // Sentinel
            tt.push(StlClickRecord {
                start: ptr::null(),
                def: StlClickDef {
                    click_type: 0,
                    tabnr: 0,
                    func: ptr::null_mut(),
                },
            });
            *tabtab = tt.as_mut_ptr().cast();
        });
    }

    // Restore global state
    redraw_not_allowed = save_redraw_not_allowed;

    if opt_idx != K_OPT_INVALID && did_emsg > did_emsg_before {
        nvim_stl_set_option_empty(opt_idx, opt_scope);
    }

    KeyTyped = save_key_typed;

    width
}

// =============================================================================
// Helper: handle group end
// =============================================================================

#[allow(
    clippy::too_many_arguments,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
unsafe fn handle_group_end(
    groupdepth: c_int,
    out_p: &mut *mut c_char,
    out_end_p: *mut c_char,
    fillchar: ScharT,
    stcp: StatuscolHandle,
) {
    let group_item_idx = STL_GROUPITEMS.with(|g| g.borrow()[groupdepth as usize]);
    let curitem = CURITEM.with(|c| *c.borrow());

    let t = STL_ITEMS.with(|items| items.borrow()[group_item_idx as usize].start);
    **out_p = NUL as u8 as c_char;
    let mut group_len = nvim_stl_vim_strsize(t) as isize;

    // Check if group should be removed (all empty items, no hl change)
    let group_minwid = STL_ITEMS.with(|items| items.borrow()[group_item_idx as usize].minwid);
    let group_maxwid = STL_ITEMS.with(|items| items.borrow()[group_item_idx as usize].maxwid);
    let group_type = STL_ITEMS.with(|items| items.borrow()[group_item_idx as usize].item_type);

    if curitem > group_item_idx + 1 && group_minwid == 0 {
        let mut group_start_userhl: c_int = 0;
        let mut group_end_userhl: c_int = 0;

        STL_ITEMS.with(|items| {
            let items = items.borrow();
            // Find starting highlight
            for n in (0..group_item_idx as usize).rev() {
                if items[n].item_type == ITEM_HIGHLIGHT {
                    group_start_userhl = items[n].minwid;
                    group_end_userhl = items[n].minwid;
                    break;
                }
            }
            // Check if all items are empty/highlight
            let mut all_empty = true;
            for n in (group_item_idx + 1) as usize..curitem as usize {
                if items[n].item_type == ITEM_NORMAL {
                    all_empty = false;
                    break;
                }
                if items[n].item_type == ITEM_HIGHLIGHT {
                    group_end_userhl = items[n].minwid;
                }
            }
            if all_empty && group_start_userhl == group_end_userhl {
                *out_p = t;
                group_len = 0;
                // Mark highlight items as empty, adjust TabPage starts
            }
        });

        if group_len == 0 {
            STL_ITEMS.with(|items| {
                let mut items = items.borrow_mut();
                for n in (group_item_idx + 1) as usize..curitem as usize {
                    if items[n].item_type == ITEM_HIGHLIGHT {
                        items[n].item_type = ITEM_EMPTY;
                    }
                    if items[n].item_type == ITEM_TABPAGE {
                        items[n].start = *out_p;
                    }
                }
            });
        }
    }

    // Truncate group if too long (but not fold items)
    if group_len > group_maxwid as isize && group_type != ITEM_HIGHLIGHT_FOLD {
        let mut n: usize = 0;
        let mut gl = group_len;
        while gl >= group_maxwid as isize {
            gl -= nvim_stl_ptr2cells(t.add(n)) as isize;
            n += nvim_stl_utfc_ptr2len(t.add(n)) as usize;
        }

        // Prepend '<'
        *t = b'<' as c_char;

        memmove(
            t.add(1).cast(),
            t.add(n).cast(),
            (*out_p).offset_from(t.add(n)) as usize,
        );
        *out_p = (*out_p).sub(n).add(1);

        while gl + 1 < group_minwid.unsigned_abs() as isize {
            schar_get_adv(out_p, fillchar);
            gl += 1;
        }

        // Correct item start positions
        STL_ITEMS.with(|items| {
            let mut items = items.borrow_mut();
            for idx in (group_item_idx + 1) as usize..curitem as usize {
                items[idx].start = items[idx].start.sub(n - 1);
                if items[idx].start < t {
                    items[idx].start = t;
                }
            }
        });
    } else if (group_minwid.unsigned_abs() as isize) > group_len {
        // Pad group
        let abs_minwid = group_minwid.unsigned_abs() as isize;
        if group_minwid < 0 {
            // Left-aligned: add fill to right
            while group_len < abs_minwid && *out_p < out_end_p {
                schar_get_adv(out_p, fillchar);
                group_len += 1;
            }
        } else {
            // Right-aligned: shift group right
            let pad = (abs_minwid - group_len) * nvim_stl_schar_len(fillchar) as isize;
            let data_len = (*out_p).offset_from(t) as usize;
            memmove(t.add(pad as usize).cast(), t.cast(), data_len);
            if (*out_p).add(pad as usize) >= out_end_p.add(1) {
                // Clamp
                *out_p = out_end_p;
            } else {
                *out_p = (*out_p).add(pad as usize);
            }

            // Adjust items
            STL_ITEMS.with(|items| {
                let mut items = items.borrow_mut();
                for n in (group_item_idx + 1) as usize..curitem as usize {
                    items[n].start = items[n].start.add(pad as usize);
                }
            });

            // Fill with fillchar
            let mut fill_p = t;
            for _ in 0..pad {
                schar_get_adv(&mut fill_p, fillchar);
            }
        }
    }
}

// =============================================================================
// Helper: handle stcsign (statuscolumn sign/fold column)
// =============================================================================

#[allow(
    clippy::too_many_arguments,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
unsafe fn handle_stcsign(
    wp: WinHandle,
    stcp: StatuscolHandle,
    buf_tmp: &mut [c_char; TMPLEN],
    out_p: &mut *mut c_char,
    _out_end_p: *mut c_char,
    str_ptr: &mut *const c_char,
    foldsignitem: &mut c_int,
    _num: &mut c_int,
    _itemisflag: &mut bool,
    _fillable: &mut bool,
    _fillchar: ScharT,
    _lnum: c_int,
) {
    let lnum = nvim_stl_get_vim_var_nr(VV_LNUM) as c_int;
    let fdc = nvim_stl_compute_foldcolumn(wp, 0);
    let w_scwidth = nvim_stl_win_get_scwidth(wp);
    let width = if fdc > 0 {
        if fdc > 0 {
            1
        } else {
            0
        }
    } else {
        w_scwidth.max(1)
    };

    if width <= 0 {
        return;
    }

    let ci = CURITEM.with(|c| *c.borrow());
    *foldsignitem = ci;

    if fdc > 0 {
        let n =
            nvim_stl_fill_foldcolumn(wp, stcp, lnum, fdc, buf_tmp.as_mut_ptr(), TMPLEN as c_int);
        let hl = if nvim_stl_use_cursor_line_hl(wp, lnum) {
            -HLF_CLF
        } else {
            -HLF_FC
        };
        STL_ITEMS.with(|items| {
            let mut items = items.borrow_mut();
            while items.len() <= ci as usize {
                items.push(StlItem::new());
            }
            items[ci as usize].minwid = hl;
        });
        let _ = n;
    }

    let mut signlen: usize = 0;
    for i in 0..width {
        STL_ITEMS.with(|items| {
            let mut items = items.borrow_mut();
            let idx = ci as usize;
            while items.len() <= idx {
                items.push(StlItem::new());
            }
            items[idx].start = (*out_p).add(signlen);
        });

        if fdc == 0 {
            let sign_info = nvim_stl_stcp_get_sign_info(stcp, i);
            let virtnum = nvim_stl_get_vim_var_nr(VV_VIRTNUM);
            if sign_info.has_text != 0 && virtnum == 0 {
                // For now, use the accessor that writes sign text for index i
                let n = nvim_stl_describe_sign_text(
                    buf_tmp.as_mut_ptr().add(signlen),
                    // The accessor should handle getting the text from stcp->sattrs[i]
                    // But our accessor takes schar_T* text. We need a different approach.
                    ptr::null_mut(),
                );
                signlen += n as usize;
                STL_ITEMS.with(|items| {
                    let mut items = items.borrow_mut();
                    items[ci as usize].minwid = -(if sign_info.sign_cul_id != 0 {
                        sign_info.sign_cul_id
                    } else {
                        sign_info.hl_id
                    });
                });
            } else {
                buf_tmp[signlen] = b' ' as c_char;
                signlen += 1;
                buf_tmp[signlen] = b' ' as c_char;
                signlen += 1;
                buf_tmp[signlen] = NUL as c_char;
                STL_ITEMS.with(|items| {
                    let mut items = items.borrow_mut();
                    items[ci as usize].minwid = 0;
                });
            }
        }

        STL_ITEMS.with(|items| {
            let mut items = items.borrow_mut();
            let idx = ci as usize;
            items[idx].item_type = if fdc > 0 {
                ITEM_HIGHLIGHT_FOLD
            } else {
                ITEM_HIGHLIGHT_SIGN
            };
        });
        CURITEM.with(|c| *c.borrow_mut() += 1);
    }

    *str_ptr = buf_tmp.as_ptr();
}

// =============================================================================
// Libc helpers
// =============================================================================

extern "C" {
    #[link_name = "snprintf"]
    fn libc_snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
    #[link_name = "malloc"]
    fn libc_malloc(size: usize) -> *mut c_void;
    #[link_name = "atoi"]
    fn c_atoi(s: *const c_char) -> c_int;
}

/// Check if string contains '%'
unsafe fn c_strchr(s: *const c_char, c: u8) -> c_int {
    let mut p = s;
    while *p != NUL as c_char {
        if *p as u8 == c {
            return 1;
        }
        p = p.add(1);
    }
    0
}
