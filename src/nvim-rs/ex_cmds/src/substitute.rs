//! `:substitute` command implementation.
//!
//! The `:substitute` (`:s`) command performs search and replace operations
//! on buffer text using regular expressions.
//!
//! ## Usage
//! - `:s/pattern/replacement/` - Substitute on current line
//! - `:%s/pattern/replacement/g` - Substitute all occurrences in buffer
//! - `:{range}s/pattern/replacement/flags` - Substitute in range with flags
//!
//! ## Flags
//! - `g` - Global: replace all occurrences on each line
//! - `c` - Confirm: ask for confirmation for each replacement
//! - `i` - Ignore case
//! - `I` - Don't ignore case (match case)
//! - `n` - Count only, don't substitute
//! - `e` - Don't report errors if no match
//! - `p` - Print the last line with a substitution
//! - `l` - Like 'p' but list the line
//! - `#` - Like 'p' but show line number
//!
//! ## Implementation Notes
//!
//! This module provides type definitions and flag parsing. The actual
//! regex matching and text replacement is performed by Neovim's core
//! substitution engine.

use std::ffi::{c_char, c_int};

use nvim_profile::rs_profile_zero;
use nvim_profile::timing::{rs_profile_passed_limit, rs_profile_setlimit};

use crate::range::LineNr;
use crate::ExArgHandle;
use crate::SubIgnoreType;

// =============================================================================
// Phase 3: sub_parse_flags migration constants
// =============================================================================

/// Pattern type: RE_SEARCH (save as search pattern, value 0)
const RE_SEARCH: c_int = 0;
/// Pattern type: RE_SUBST (save as substitute pattern, value 1)
const RE_SUBST: c_int = 1;
/// Pattern type: use last used regexp (RE_LAST from search.h, value 2)
const RE_LAST: c_int = 2;

/// Add search pattern to history (SEARCH_HIS from search.h, value 0x20)
const SEARCH_HIS: c_int = 0x20;
/// History type: HIST_SEARCH = 1 (search command history)
const HIST_SEARCH: c_int = 1;

/// vim_regsub_multi flags (from regexp_defs.h)
const REGSUB_COPY: c_int = 1;
const REGSUB_MAGIC: c_int = 2;
const REGSUB_BACKSLASH: c_int = 4;

/// Maximum valid column (MAXCOL from pos_defs.h)
const MAXCOL: c_int = 0x7fffffff;

/// Maximum line number (MAXLNUM from pos_defs.h; verified by _Static_assert in shim)
const MAXLNUM: c_int = 0x7fff_ffff;

/// No-op extmark operation (kExtmarkNOOP; verified by _Static_assert in shim)
const KEXTMARK_NOOP: c_int = 0;

/// Command index for :~ (tilde) command (CMD_tilde from ex_cmds_enum.generated.h)
const CMD_TILDE: c_int = 554;

/// ExtmarkOp constant for undo (KEXTMARK_UNDO from buffer_defs.h; verified by _Static_assert)
const KEXTMARK_UNDO: c_int = 1;

/// EXFLAG constants (must match C defines)
const EXFLAG_LIST: c_int = 0x01;
const EXFLAG_NR: c_int = 0x02;
const EXFLAG_PRINT: c_int = 0x04;

extern "C" {
    static mut p_gd: c_int;
    static mut p_lz: c_int;
}

extern "C" {
    // show_sub FFI
    fn nvim_excmds_save_set_shortmess_F() -> *mut c_char;
    fn nvim_excmds_restore_shortmess(saved: *mut c_char);
    fn nvim_excmds_get_p_icm_first() -> c_int;
    fn buflist_findnr(nr: c_int) -> *mut crate::BufHandle;
    fn buf_ensure_loaded(buf: *mut crate::BufHandle);
    fn ml_get_buf(buf: *mut crate::BufHandle, lnum: c_int) -> *mut c_char;
    fn ml_get_buf_len(buf: *mut crate::BufHandle, lnum: c_int) -> c_int;
    fn ml_replace_buf(
        buf: *mut crate::BufHandle,
        lnum: c_int,
        line: *mut c_char,
        copy: bool,
        keep_dirty: bool,
    ) -> c_int;
    fn ml_append_buf(
        buf: *mut crate::BufHandle,
        lnum: c_int,
        line: *mut c_char,
        len: c_int,
        newfile: bool,
    ) -> c_int;
    #[link_name = "rs_bufhl_add_hl_pos_offset"]
    fn nvim_excmds_bufhl_add_hl_pos_offset(
        buf: *mut crate::BufHandle,
        ns_id: c_int,
        hl_id: c_int,
        start_lnum: c_int,
        start_col: c_int,
        end_lnum: c_int,
        end_col: c_int,
        offset: c_int,
    );
    fn update_topline(win: *mut crate::WinHandle);
    fn nvim_excmds_curbuf_ml_line_count() -> c_int;
    fn nvim_excmds_preview_lines_size(pl: *const std::ffi::c_void) -> usize;
    fn nvim_excmds_preview_lines_item(
        pl: *const std::ffi::c_void,
        idx: usize,
        start_lnum: *mut c_int,
        start_col: *mut c_int,
        end_lnum: *mut c_int,
        end_col: *mut c_int,
        pre_match: *mut c_int,
    );
    fn nvim_curwin_set_cursor_col(col: c_int);
    fn nvim_get_curbuf() -> *mut crate::BufHandle;

    // sub_joining_lines FFI
    fn nvim_exarg_get_skip(eap: *const ExArgHandle) -> c_int;
    fn nvim_exarg_get_line1(eap: *const ExArgHandle) -> c_int;
    fn nvim_exarg_get_line2(eap: *const ExArgHandle) -> c_int;
    fn nvim_exarg_set_flags(eap: *mut ExArgHandle, flags: c_int);
    fn nvim_curwin_set_cursor_lnum(lnum: c_int);
    fn nvim_curbuf_get_b_ml_ml_line_count() -> c_int;
    fn do_join(
        count: usize,
        insert_space: bool,
        save_undo: bool,
        use_formatoptions: bool,
        setmark: bool,
    ) -> c_int;
    fn aborting() -> c_int;
    fn nvim_docmd_ex_may_print_impl(eap: *mut ExArgHandle);
    fn save_re_pat(idx: c_int, pat: *mut c_char, patlen: usize, magic: c_int);
    fn add_to_history(
        histype: c_int,
        new_entry: *const c_char,
        new_entrylen: usize,
        in_map: bool,
        sep: c_int,
    );
    fn rs_magic_isset() -> c_int;

    // do_sub_msg FFI -- control flow in Rust, formatting/messaging in C
    /// Return messaging() result (1 = messaging on, 0 = off).
    fn messaging() -> c_int;
    /// Format and display the substitution count message (NGETTEXT in C).
    /// Returns true if message was displayed.
    fn nvim_excmds_format_sub_msg(count_only: c_int, nsubs: c_int, nlines: c_int) -> c_int;
    /// emsg(_(e_interr)) wrapper.
    fn nvim_excmds_emsg_interr();

    // sub_grow_buf FFI
    fn xcalloc(count: usize, size: usize) -> *mut std::ffi::c_void;
    fn xrealloc(ptr: *mut std::ffi::c_void, size: usize) -> *mut std::ffi::c_void;
}

// =============================================================================
// Phase 1: regmmatch_T opaque handle infrastructure
// =============================================================================

extern "C" {
    /// Get regmatch->startpos[0].lnum
    pub fn nvim_regmmatch_startpos0_lnum(rm: *mut crate::RegmmatchHandle) -> c_int;
    /// Get regmatch->startpos[0].col
    pub fn nvim_regmmatch_startpos0_col(rm: *mut crate::RegmmatchHandle) -> c_int;
    /// Get regmatch->endpos[0].lnum
    pub fn nvim_regmmatch_endpos0_lnum(rm: *mut crate::RegmmatchHandle) -> c_int;
    /// Get regmatch->endpos[0].col
    pub fn nvim_regmmatch_endpos0_col(rm: *mut crate::RegmmatchHandle) -> c_int;
    /// Set regmatch->rmm_ic
    pub fn nvim_regmmatch_set_rmm_ic(rm: *mut crate::RegmmatchHandle, ic: c_int);
    /// Get regmatch->rmm_ic
    pub fn nvim_regmmatch_get_rmm_ic(rm: *mut crate::RegmmatchHandle) -> c_int;
    /// Call re_multiline(regmatch->regprog)
    pub fn nvim_regmmatch_re_multiline(rm: *mut crate::RegmmatchHandle) -> c_int;
    /// Wrap search_regcomp for do_sub, allocating and returning opaque regmmatch_T*
    pub fn nvim_do_sub_search_regcomp(
        pat: *const c_char,
        patlen: usize,
        which_pat: c_int,
        flags: c_int,
    ) -> *mut crate::RegmmatchHandle;
    /// Wrap vim_regexec_multi for do_sub
    pub fn nvim_do_sub_vim_regexec_multi(
        rm: *mut crate::RegmmatchHandle,
        lnum: c_int,
        col: c_int,
    ) -> c_int;
    /// Wrap vim_regsub_multi for do_sub
    #[link_name = "vim_regsub_multi"]
    pub fn nvim_do_sub_vim_regsub_multi(
        rm: *mut crate::RegmmatchHandle,
        source_lnum: c_int,
        sub_str: *const c_char,
        dest: *mut c_char,
        destlen: c_int,
        flags: c_int,
    ) -> c_int;
    /// Free the regmmatch_T opaque handle (vim_regfree + xfree)
    pub fn nvim_excmds_vim_regfree_multi(rm: *mut std::ffi::c_void);
    /// regtilde: expand ~ in replacement string
    pub fn regtilde(source: *mut c_char, magic: c_int, preview: bool) -> *mut c_char;
}

/// C-compatible layout for subflags_T.
///
/// Must match the C struct exactly:
/// ```c
/// typedef struct {
///   bool do_all;
///   bool do_ask;
///   bool do_count;
///   bool do_error;
///   bool do_print;
///   bool do_list;
///   bool do_number;
///   SubIgnoreType do_ic;  // int enum, aligned to 4 bytes
/// } subflags_T;
/// ```
#[repr(C)]
pub struct CSubFlags {
    pub do_all: bool,
    pub do_ask: bool,
    pub do_count: bool,
    pub do_error: bool,
    pub do_print: bool,
    pub do_list: bool,
    pub do_number: bool,
    pub do_ic: c_int, // SubIgnoreType as int
}

/// Flags for the `:substitute` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SubFlags {
    /// Do multiple substitutions per line (g flag).
    pub do_all: bool,
    /// Ask for confirmation (c flag).
    pub do_ask: bool,
    /// Count only, don't substitute (n flag).
    pub do_count: bool,
    /// If false, ignore errors when no match (e flag).
    pub do_error: bool,
    /// Print last line with subs (p flag).
    pub do_print: bool,
    /// List last line with subs (l flag).
    pub do_list: bool,
    /// List last line with line number (# flag).
    pub do_number: bool,
    /// Case sensitivity mode.
    pub do_ic: SubIgnoreType,
}

impl SubFlags {
    /// Create flags with default values (honor options, errors enabled).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            do_all: false,
            do_ask: false,
            do_count: false,
            do_error: true,
            do_print: false,
            do_list: false,
            do_number: false,
            do_ic: SubIgnoreType::HonorOptions,
        }
    }

    /// Create flags for a simple global substitute.
    #[must_use]
    pub const fn global() -> Self {
        Self {
            do_all: true,
            do_error: true,
            do_ask: false,
            do_count: false,
            do_print: false,
            do_list: false,
            do_number: false,
            do_ic: SubIgnoreType::HonorOptions,
        }
    }

    /// Create flags for count-only mode.
    #[must_use]
    pub const fn count_only() -> Self {
        Self {
            do_all: true,
            do_count: true,
            do_error: true,
            do_ask: false,
            do_print: false,
            do_list: false,
            do_number: false,
            do_ic: SubIgnoreType::HonorOptions,
        }
    }

    /// Parse flags from a flag string.
    ///
    /// # Arguments
    /// * `flags` - A string containing flag characters (e.g., "gc" for global + confirm)
    ///
    /// # Returns
    /// The parsed flags, or an error if an invalid flag is found.
    pub fn parse(flags: &str) -> Result<Self, SubstituteError> {
        let mut result = Self::new();

        for c in flags.chars() {
            match c {
                'g' => result.do_all = true,
                'c' => result.do_ask = true,
                'n' => result.do_count = true,
                'e' => result.do_error = false,
                'p' => result.do_print = true,
                'l' => result.do_list = true,
                '#' => result.do_number = true,
                'i' => result.do_ic = SubIgnoreType::IgnoreCase,
                'I' => result.do_ic = SubIgnoreType::MatchCase,
                'r' => { /* use last search pattern - handled elsewhere */ }
                ' ' | '\t' => { /* skip whitespace */ }
                _ => return Err(SubstituteError::InvalidFlag(c)),
            }
        }

        Ok(result)
    }

    /// Check if this is a counting-only operation.
    #[inline]
    #[must_use]
    pub const fn is_count_only(&self) -> bool {
        self.do_count
    }

    /// Check if confirmation is required.
    #[inline]
    #[must_use]
    pub const fn needs_confirm(&self) -> bool {
        self.do_ask
    }
}

/// Result of a substitution operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SubResult {
    /// Number of substitutions made.
    pub count: i32,
    /// Number of lines changed.
    pub lines: i32,
    /// Whether the operation was interrupted.
    pub interrupted: bool,
}

impl SubResult {
    /// Create a new empty result.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            count: 0,
            lines: 0,
            interrupted: false,
        }
    }

    /// Check if any substitutions were made.
    #[inline]
    #[must_use]
    pub const fn has_matches(&self) -> bool {
        self.count > 0
    }

    /// Add a match to the result.
    #[inline]
    pub fn add_match(&mut self) {
        self.count += 1;
    }

    /// Record a changed line.
    #[inline]
    pub fn add_line(&mut self) {
        self.lines += 1;
    }

    /// Mark as interrupted.
    #[inline]
    pub fn set_interrupted(&mut self) {
        self.interrupted = true;
    }
}

/// Statistics for a substitution operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SubStats {
    /// Total number of substitutions across all operations.
    pub total_subs: i32,
    /// Total number of lines changed across all operations.
    pub total_lines: i32,
}

impl SubStats {
    /// Create new statistics.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_subs: 0,
            total_lines: 0,
        }
    }

    /// Add a result to the statistics.
    pub fn add_result(&mut self, result: &SubResult) {
        self.total_subs += result.count;
        self.total_lines += result.lines;
    }

    /// Reset the statistics.
    pub fn reset(&mut self) {
        self.total_subs = 0;
        self.total_lines = 0;
    }
}

/// Error type for substitution operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubstituteError {
    /// Invalid flag character.
    InvalidFlag(char),
    /// Invalid delimiter (alphanumeric).
    InvalidDelimiter(char),
    /// Empty pattern with no previous pattern.
    NoPreviousPattern,
    /// Empty replacement with no previous replacement.
    NoPreviousReplacement,
    /// Invalid regular expression.
    InvalidRegex(String),
    /// Invalid range.
    InvalidRange,
    /// Zero count given.
    ZeroCount,
    /// Operation was interrupted.
    Interrupted,
}

impl std::fmt::Display for SubstituteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubstituteError::InvalidFlag(c) => write!(f, "invalid flag: {c}"),
            SubstituteError::InvalidDelimiter(c) => {
                write!(f, "regular expressions can't be delimited by letters: {c}")
            }
            SubstituteError::NoPreviousPattern => write!(f, "no previous pattern"),
            SubstituteError::NoPreviousReplacement => write!(f, "no previous substitute command"),
            SubstituteError::InvalidRegex(msg) => write!(f, "invalid regex: {msg}"),
            SubstituteError::InvalidRange => write!(f, "invalid range"),
            SubstituteError::ZeroCount => write!(f, "zero count"),
            SubstituteError::Interrupted => write!(f, "interrupted"),
        }
    }
}

impl std::error::Error for SubstituteError {}

/// Check if a character is a valid delimiter.
///
/// Delimiters cannot be alphanumeric characters.
#[inline]
#[must_use]
pub fn is_valid_delimiter(c: char) -> bool {
    !c.is_alphanumeric()
}

/// Position within a match result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MatchPosition {
    /// Line number (1-based).
    pub lnum: LineNr,
    /// Column offset (0-based).
    pub col: i32,
}

impl MatchPosition {
    /// Create a new match position.
    #[must_use]
    pub const fn new(lnum: LineNr, col: i32) -> Self {
        Self { lnum, col }
    }
}

/// A range of matched text for preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MatchRange {
    /// Start position.
    pub start: MatchPosition,
    /// End position.
    pub end: MatchPosition,
}

impl MatchRange {
    /// Create a new match range.
    #[must_use]
    pub const fn new(start: MatchPosition, end: MatchPosition) -> Self {
        Self { start, end }
    }

    /// Check if this is a single-line match.
    #[inline]
    #[must_use]
    pub const fn is_single_line(&self) -> bool {
        self.start.lnum == self.end.lnum
    }

    /// Get the number of lines spanned by this match.
    #[inline]
    #[must_use]
    pub const fn line_span(&self) -> LineNr {
        self.end.lnum - self.start.lnum + 1
    }
}

// =============================================================================
// Phase 1: sub_joining_lines + sub_grow_buf
// =============================================================================

/// Recognize `:%s/\n//` and turn it into a join command, which is much
/// more efficient. Replaces the C `sub_joining_lines` static function.
///
/// Returns 1 (true) if the substitute can be replaced with a join, 0 (false) otherwise.
///
/// # Safety
/// All pointer arguments must be non-null and point to valid C strings.
/// `eap` must be a valid exarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_sub_joining_lines(
    eap: *mut ExArgHandle,
    pat: *const c_char,
    patlen: usize,
    sub: *const c_char,
    cmd: *const c_char,
    save: c_int,
    keeppatterns: c_int,
) -> c_int {
    use std::ffi::CStr;

    // pat must be non-null and equal to "\\n", sub must be NUL, and
    // cmd must be NUL or a single one of 'g', 'l', 'p', '#'.
    if pat.is_null() {
        return 0;
    }

    let pat_bytes = CStr::from_ptr(pat).to_bytes();
    if pat_bytes != b"\\n" {
        return 0;
    }

    // *sub == NUL
    if *sub != 0 {
        return 0;
    }

    // cmd must be NUL or a single recognized flag
    let cmd0 = *cmd as u8;
    let cmd1 = if cmd0 != 0 { *cmd.add(1) as u8 } else { 0 };
    if cmd0 != 0 && (cmd1 != 0 || !matches!(cmd0, b'g' | b'l' | b'p' | b'#')) {
        return 0;
    }

    // Pattern matches - this is a join-optimization candidate
    if nvim_exarg_get_skip(eap) != 0 {
        return 1; // skip mode: pretend we handled it
    }

    let line1 = nvim_exarg_get_line1(eap);
    let line2 = nvim_exarg_get_line2(eap);

    nvim_curwin_set_cursor_lnum(line1);

    // Set eap->flags based on cmd character
    match cmd0 {
        b'l' => nvim_exarg_set_flags(eap, EXFLAG_LIST),
        b'#' => nvim_exarg_set_flags(eap, EXFLAG_NR),
        b'p' => nvim_exarg_set_flags(eap, EXFLAG_PRINT),
        _ => {}
    }

    let ml_line_count = nvim_curbuf_get_b_ml_ml_line_count();
    let joined_lines_count = (line2 - line1 + 1) + if line2 < ml_line_count { 1 } else { 0 };

    if joined_lines_count > 1 {
        do_join(joined_lines_count as usize, false, true, false, true);
        crate::sub_nsubs = joined_lines_count - 1;
        crate::sub_nlines = 1;
        rs_do_sub_msg(false);
        nvim_docmd_ex_may_print_impl(eap);
    }

    if save != 0 {
        if keeppatterns == 0 {
            save_re_pat(RE_SUBST, pat as *mut c_char, patlen, rs_magic_isset());
        }
        add_to_history(HIST_SEARCH, pat, patlen, true, 0);
    }

    1 // true: substitute was handled as join
}

/// Allocate or grow the replacement text buffer for :substitute.
/// Replaces the C `sub_grow_buf` static function.
///
/// - If `*new_start` is null: allocates a new buffer of size `needed_len + 50`,
///   zero-initialized, and returns pointer to the start.
/// - Otherwise: if the existing buffer is too small, reallocates it.
///   Returns pointer to the current end of the string in the buffer.
///
/// # Safety
/// `new_start` and `new_start_len` must be non-null valid pointers.
/// `*new_start` must be null or point to an xmalloc-allocated buffer.
#[no_mangle]
pub unsafe extern "C" fn rs_sub_grow_buf(
    new_start: *mut *mut c_char,
    new_start_len: *mut c_int,
    needed_len: c_int,
) -> *mut c_char {
    if (*new_start).is_null() {
        // Initial allocation: needed_len + 50 extra bytes, zero-initialized.
        let alloc_len = (needed_len + 50) as usize;
        *new_start_len = needed_len + 50;
        let ptr = xcalloc(1, alloc_len) as *mut c_char;
        *ptr = 0; // ensure NUL-terminated
        *new_start = ptr;
        ptr
    } else {
        // Find current length of string in the buffer (strlen).
        let current_ptr = *new_start;
        let mut len = 0usize;
        while *current_ptr.add(len) != 0 {
            len += 1;
        }
        let needed = needed_len + len as c_int;
        if needed > *new_start_len {
            let prev_len = *new_start_len as usize;
            *new_start_len = needed + 50;
            let new_alloc_len = *new_start_len as usize;
            *new_start =
                xrealloc(*new_start as *mut std::ffi::c_void, new_alloc_len) as *mut c_char;
            // Zero out the newly added region
            let added = new_alloc_len - prev_len;
            std::ptr::write_bytes((*new_start).add(prev_len), 0, added);
        }
        (*new_start).add(len)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Give message for number of substitutions.
///
/// Replaces the C `do_sub_msg` function. Contains the control-flow logic;
/// delegates NGETTEXT formatting to `nvim_excmds_format_sub_msg` which keeps
/// internationalization in C.
///
/// # Safety
/// Calls C accessor functions.
#[allow(clippy::must_use_candidate)]
#[export_name = "do_sub_msg"]
pub unsafe extern "C" fn rs_do_sub_msg(count_only: bool) -> bool {
    let cur_nsubs = crate::sub_nsubs;
    let cur_nlines = crate::sub_nlines;
    let p_report = crate::p_report;
    let key_typed = crate::KeyTyped;
    let messaging = messaging() != 0;
    let got_int = crate::got_int;

    let threshold_met = (cur_nsubs as i64 > p_report
        && (key_typed || cur_nlines > 1 || p_report < 1))
        || count_only;

    if threshold_met && messaging {
        nvim_excmds_format_sub_msg(c_int::from(count_only), cur_nsubs, cur_nlines);
        return true;
    }
    if got_int {
        nvim_excmds_emsg_interr();
        return true;
    }
    false
}

/// Parse :substitute flags from a C command string. Replaces the C
/// `sub_parse_flags` function.
///
/// - If `*cmd == '&'`, advances cmd and keeps existing flags.
/// - Otherwise resets flags: do_all = p_gd, do_ask = false, do_error = true, etc.
/// - Loops through chars: toggles g/c, sets other flags, breaks on unrecognized.
/// - If do_count, sets do_ask = false.
/// - Returns pointer past consumed flags.
///
/// # Safety
/// `cmd` must be non-null and point to a valid null-terminated C string.
/// `subflags` must be non-null and point to a valid CSubFlags struct.
/// `which_pat` must be non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_sub_parse_flags(
    cmd: *mut c_char,
    subflags: *mut CSubFlags,
    which_pat: *mut c_int,
) -> *mut c_char {
    let gd_val = p_gd != 0;

    // Find trailing options. When '&' is used, keep old options.
    let mut p = cmd;
    if *p == b'&' as c_char {
        p = p.add(1);
    } else {
        (*subflags).do_all = gd_val;
        (*subflags).do_ask = false;
        (*subflags).do_error = true;
        (*subflags).do_print = false;
        (*subflags).do_list = false;
        (*subflags).do_count = false;
        (*subflags).do_number = false;
        (*subflags).do_ic = 0; // kSubHonorOptions
    }

    loop {
        let c = *p as u8;
        if c == b'g' {
            (*subflags).do_all = !(*subflags).do_all;
        } else if c == b'c' {
            (*subflags).do_ask = !(*subflags).do_ask;
        } else if c == b'n' {
            (*subflags).do_count = true;
        } else if c == b'e' {
            (*subflags).do_error = !(*subflags).do_error;
        } else if c == b'r' {
            // use last used regexp
            *which_pat = RE_LAST;
        } else if c == b'p' {
            (*subflags).do_print = true;
        } else if c == b'#' {
            (*subflags).do_print = true;
            (*subflags).do_number = true;
        } else if c == b'l' {
            (*subflags).do_print = true;
            (*subflags).do_list = true;
        } else if c == b'i' {
            // ignore case
            (*subflags).do_ic = 1; // kSubIgnoreCase
        } else if c == b'I' {
            // don't ignore case
            (*subflags).do_ic = 2; // kSubMatchCase
        } else {
            break;
        }
        p = p.add(1);
    }

    if (*subflags).do_count {
        (*subflags).do_ask = false;
    }

    p
}

// =============================================================================
// show_sub implementation
// =============================================================================

/// Shows the effects of the :substitute command being typed ('inccommand').
///
/// If inccommand=split, shows a preview window and later restores the layout.
/// Replaces the C `show_sub` function.
///
/// Returns 1 if preview window isn't needed, 2 if preview window is needed.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_show_sub(
    eap: *const ExArgHandle,
    old_cusr_lnum: c_int,
    _old_cusr_col: c_int,
    preview_lines: *const std::ffi::c_void,
    hl_id: c_int,
    cmdpreview_ns: c_int,
    cmdpreview_bufnr: c_int,
) -> c_int {
    use std::io::Write;

    // Save and disable file info message
    let save_shm_p = nvim_excmds_save_set_shortmess_F();

    let orig_buf = nvim_get_curbuf();
    let num_results = nvim_excmds_preview_lines_size(preview_lines);

    // Place cursor on nearest matching line, to undo do_sub() cursor placement.
    for i in 0..num_results {
        let mut start_lnum: c_int = 0;
        let mut start_col: c_int = 0;
        let mut end_lnum: c_int = 0;
        let mut end_col: c_int = 0;
        let mut pre_match: c_int = 0;
        nvim_excmds_preview_lines_item(
            preview_lines,
            i,
            &mut start_lnum,
            &mut start_col,
            &mut end_lnum,
            &mut end_col,
            &mut pre_match,
        );
        if start_lnum >= old_cusr_lnum {
            nvim_curwin_set_cursor_lnum(start_lnum);
            nvim_curwin_set_cursor_col(start_col);
            break;
        }
    }

    // Update the topline to ensure that main window is on the correct line.
    update_topline(crate::nvim_get_curwin());

    // Width of the "| lnum|..." column which displays the line numbers.
    let mut col_width: c_int = 0;

    let line1 = nvim_exarg_get_line1(eap);
    let line2 = nvim_exarg_get_line2(eap);

    // Use preview window only when inccommand=split and range is not just the current line.
    let preview = nvim_excmds_get_p_icm_first() == b's' as c_int
        && (line1 != old_cusr_lnum || line2 != old_cusr_lnum);

    let mut cmdpreview_buf: *mut crate::BufHandle = std::ptr::null_mut();
    if preview {
        cmdpreview_buf = buflist_findnr(cmdpreview_bufnr);
        // cmdpreview_buf must be non-NULL per the C assert
        if num_results > 0 {
            let last_idx = num_results - 1;
            let mut sl: c_int = 0;
            let mut sc: c_int = 0;
            let mut el: c_int = 0;
            let mut ec: c_int = 0;
            let mut pm: c_int = 0;
            nvim_excmds_preview_lines_item(
                preview_lines,
                last_idx,
                &mut sl,
                &mut sc,
                &mut el,
                &mut ec,
                &mut pm,
            );
            let highest_lnum = sl.max(el);
            if highest_lnum > 0 {
                col_width = (highest_lnum as f64).log10() as c_int + 1 + 3;
            }
        }
    }

    let mut str_buf: *mut c_char = std::ptr::null_mut();
    let mut str_buf_size: usize = 0;
    let mut linenr_preview: c_int = 0;
    let mut linenr_origbuf: c_int = 0;

    for matchidx in 0..num_results {
        let mut start_lnum: c_int = 0;
        let mut start_col: c_int = 0;
        let mut end_lnum: c_int = 0;
        let mut end_col: c_int = 0;
        let mut pre_match: c_int = 0;
        nvim_excmds_preview_lines_item(
            preview_lines,
            matchidx,
            &mut start_lnum,
            &mut start_col,
            &mut end_lnum,
            &mut end_col,
            &mut pre_match,
        );

        if !cmdpreview_buf.is_null() {
            let mut p_start_lnum: c_int = 0;
            let p_start_col: c_int = start_col;
            let mut p_end_lnum: c_int = 0;
            let p_end_col: c_int = end_col;

            buf_ensure_loaded(cmdpreview_buf);

            let mut next_linenr: c_int = if pre_match == 0 {
                start_lnum
            } else {
                pre_match
            };

            // Don't add a line twice
            if next_linenr == linenr_origbuf {
                next_linenr += 1;
                p_start_lnum = linenr_preview;
                p_end_lnum = linenr_preview;
            }

            let orig_buf_line_count = nvim_excmds_curbuf_ml_line_count();

            while next_linenr <= end_lnum {
                if next_linenr == start_lnum {
                    p_start_lnum = linenr_preview + 1;
                }
                if next_linenr == end_lnum {
                    p_end_lnum = linenr_preview + 1;
                }

                let line_ptr: *const c_char;
                let line_size: usize;

                if next_linenr == orig_buf_line_count + 1 {
                    line_ptr = c"".as_ptr();
                    // Need enough for "|col_width| \0"
                    line_size = (col_width as usize) + 4;
                } else {
                    line_ptr = ml_get_buf(orig_buf, next_linenr);
                    let raw_len = ml_get_buf_len(orig_buf, next_linenr);
                    line_size = raw_len as usize + col_width as usize + 2;
                }

                // Reallocate str_buf if not large enough
                if line_size > str_buf_size {
                    str_buf =
                        xrealloc(str_buf as *mut std::ffi::c_void, line_size + 1) as *mut c_char;
                    str_buf_size = line_size + 1;
                }

                // Format: "|{lnum:col_width-3}| {line}"
                let line_str = std::ffi::CStr::from_ptr(line_ptr).to_bytes();
                let num_width = (col_width - 3) as usize;
                let mut formatted = Vec::<u8>::with_capacity(line_size + 1);
                write!(
                    &mut formatted,
                    "|{:>width$}| ",
                    next_linenr,
                    width = num_width
                )
                .ok();
                formatted.extend_from_slice(line_str);
                formatted.push(0); // NUL terminator

                // Copy to str_buf
                let copy_len = formatted.len().min(str_buf_size);
                std::ptr::copy_nonoverlapping(
                    formatted.as_ptr() as *const c_char,
                    str_buf,
                    copy_len,
                );

                if linenr_preview == 0 {
                    ml_replace_buf(cmdpreview_buf, 1, str_buf, true, false);
                } else {
                    ml_append_buf(
                        cmdpreview_buf,
                        linenr_preview,
                        str_buf,
                        line_size as c_int,
                        false,
                    );
                }
                linenr_preview += 1;
                next_linenr += 1;
            }
            linenr_origbuf = end_lnum;

            nvim_excmds_bufhl_add_hl_pos_offset(
                cmdpreview_buf,
                cmdpreview_ns,
                hl_id,
                p_start_lnum,
                p_start_col,
                p_end_lnum,
                p_end_col,
                col_width,
            );
        }

        // Add highlight to original buffer for this match
        nvim_excmds_bufhl_add_hl_pos_offset(
            orig_buf,
            cmdpreview_ns,
            hl_id,
            start_lnum,
            start_col,
            end_lnum,
            end_col,
            0,
        );
    }

    if !str_buf.is_null() {
        crate::xfree(str_buf as *mut std::ffi::c_void);
    }

    nvim_excmds_restore_shortmess(save_shm_p);

    if preview {
        2
    } else {
        1
    }
}

/// Parse substitute flags from a string.
///
/// Returns a bitmask of flags:
/// - bit 0: do_all (g)
/// - bit 1: do_ask (c)
/// - bit 2: do_count (n)
/// - bit 3: do_error (inverted: set if errors should be reported)
/// - bit 4: do_print (p)
/// - bit 5: do_list (l)
/// - bit 6: do_number (#)
/// - bits 7-8: do_ic (0=honor, 1=ignore, 2=match)
///
/// Returns -1 on error.
///
/// # Safety
/// The `flags` pointer must be null or point to a valid null-terminated C string.
pub unsafe extern "C" fn rs_parse_sub_flags(flags: *const std::ffi::c_char) -> c_int {
    if flags.is_null() {
        return 0; // No flags = default
    }

    let flags_str = match std::ffi::CStr::from_ptr(flags).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match SubFlags::parse(flags_str) {
        Ok(f) => {
            let mut result: c_int = 0;
            if f.do_all {
                result |= 1 << 0;
            }
            if f.do_ask {
                result |= 1 << 1;
            }
            if f.do_count {
                result |= 1 << 2;
            }
            if f.do_error {
                result |= 1 << 3;
            }
            if f.do_print {
                result |= 1 << 4;
            }
            if f.do_list {
                result |= 1 << 5;
            }
            if f.do_number {
                result |= 1 << 6;
            }
            result |= (f.do_ic.to_c() & 0x3) << 7;
            result
        }
        Err(_) => -1,
    }
}

/// Check if a delimiter character is valid.
///
/// Returns 1 if valid, 0 if invalid (alphanumeric).
pub extern "C" fn rs_is_valid_delimiter(c: c_int) -> c_int {
    if !(0..=127).contains(&c) {
        return 0;
    }
    c_int::from(is_valid_delimiter(char::from(c as u8)))
}

/// OK/FAIL constants matching C definitions
const OK: c_int = 1;
const FAIL: c_int = 0;

/// Check if a character is a valid regexp delimiter.
///
/// Returns OK (1) if valid, FAIL (0) if the character is alphabetic.
/// Emits an error message on failure.
///
/// # Safety
/// Calls C emsg function.
#[no_mangle]
pub unsafe extern "C" fn rs_check_regexp_delim(c: c_int) -> c_int {
    use crate::emsg;

    // isalpha(c) in C -- check if ASCII alphabetic
    if (c as u8 as char).is_ascii_alphabetic() {
        emsg(c"E146: Regular expressions can't be delimited by letters".as_ptr());
        return FAIL;
    }
    OK
}

/// Skip over the "sub" part in :s/pat/sub/ where `delimiter` is the
/// separating character. Replaces the end delimiter with NUL.
///
/// Returns a pointer past the end delimiter (or to end of string).
///
/// # Safety
/// `start` must point to a valid, writable null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_substitute(
    start: *mut std::ffi::c_char,
    delimiter: c_int,
) -> *mut std::ffi::c_char {
    use crate::utfc_ptr2len;

    let mut p = start;
    while *p != 0 {
        if *p as c_int == delimiter {
            // end delimiter found -- replace it with NUL
            *p = 0;
            p = p.add(1);
            break;
        }
        if *p == b'\\' as i8 && *p.add(1) != 0 {
            // skip escaped characters
            p = p.add(1);
        }
        // MB_PTR_ADV(p)
        let len = utfc_ptr2len(p) as usize;
        p = p.add(len);
    }
    p
}

// =============================================================================
// Phase 2: Additional global/struct accessor declarations for do_sub
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Generic global accessors (replacing do_sub-specific ones)
    fn nvim_get_RedrawingDisabled() -> c_int;
    fn nvim_set_RedrawingDisabled(val: c_int);
    fn nvim_inc_no_u_sync();
    fn nvim_dec_no_u_sync();
    static mut msg_didout: bool;
    fn nvim_set_highlight_match(val: c_int);
    fn nvim_set_search_match_lines(val: c_int);
    fn nvim_set_search_match_endcol(val: c_int);
    static ex_normal_busy: c_int;
    static mut exmode_active: bool;
    static mut need_wait_return: bool;
    fn nvim_get_sandbox() -> c_int;
    fn nvim_inc_sandbox();
    fn nvim_dec_sandbox();
    fn nvim_get_textlock() -> c_int;
    fn nvim_inc_textlock();
    fn nvim_dec_textlock();
    static mut p_ch: i64;
    static p_cpo: *const c_char;
    fn nvim_curwin_get_w_curswant() -> c_int;
    fn nvim_curwin_get_w_botline() -> c_int;
    fn nvim_curwin_get_w_p_crb() -> c_int;
    fn nvim_curwin_get_w_p_fen() -> c_int;
    fn nvim_curwin_set_w_p_fen(val: c_int);
    fn nvim_curbuf_get_b_p_ma() -> c_int;
    fn nvim_curbuf_set_b_p_ma(val: c_int);
    fn nvim_curbuf_set_deleted_bytes2(val: c_int);
    fn nvim_coladvance(col: c_int);
    #[link_name = "u_inssub"]
    fn nvim_u_inssub(lnum: c_int) -> c_int;
    #[link_name = "u_savesub"]
    fn nvim_u_savesub(lnum: c_int) -> c_int;
    #[link_name = "u_savedel"]
    fn nvim_u_savedel2(lnum: c_int, count: c_int) -> c_int;
    #[link_name = "u_save_cursor"]
    fn nvim_u_save_cursor() -> c_int;
    fn do_check_cursorbind();
    fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int;
    static mut p_rdt: i64;
    fn nvim_do_sub_skip_regexp_ex(
        cmd: *mut c_char,
        delim: c_int,
        arg_ptr: *mut *mut c_char,
    ) -> *mut c_char;
    fn check_nextcmd(cmd: *const c_char) -> *mut c_char;
    fn changed_window_setting(wp: *mut crate::WinHandle);
    fn nvim_curwin_get_cursor_col() -> c_int;
    static mut p_cwh: i64;
    fn setpcmark();
    fn nvim_do_sub_getvcol_start_end(
        lnum: c_int,
        start_col: c_int,
        end_col: c_int,
        sc_out: *mut c_int,
        ec_out: *mut c_int,
    );
    fn nvim_do_sub_getcmdline_prompt(prompt_str: *const c_char) -> c_int;
    fn prompt_for_input(
        prompt: *const c_char,
        hl_id: c_int,
        one_key: bool,
        mouse_used: *mut bool,
    ) -> c_int;
    fn nvim_do_sub_update_screen_for_confirm();
    fn nvim_al_gotocmdline(clr: c_int);
    fn number_width(wp: *mut crate::WinHandle) -> c_int;
    fn syn_check_group(name: *const c_char, len: usize) -> c_int;
    fn nvim_excmds_disable_inccommand();
    static p_icm: *const std::ffi::c_char;
    fn nvim_do_sub_set_op_start_end(start_lnum: c_int, end_lnum: c_int);
    fn nvim_curbuf_modifiable() -> bool;
    fn nvim_emsg_nopresub();
    fn nvim_emsg_modifiable();
    fn nvim_excmds_emsg_by_id(id: c_int);
    fn nvim_excmds_emsg_with_arg(id: c_int, arg: *const c_char);
    fn nvim_do_sub_format_confirm_prompt(sub_str: *const c_char) -> *mut c_char;
    fn appended_lines(lnum: c_int, count: c_int);
    fn nvim_do_sub_changed_lines(first: c_int, last: c_int, xtra: c_int);
    fn mark_adjust(line1: c_int, line2: c_int, amount: c_int, amount_after: c_int, op: c_int);
    fn extmark_splice(
        buf: *mut crate::BufHandle,
        start_row: c_int,
        start_col: c_int,
        old_row: c_int,
        old_col: c_int,
        old_byte: i64,
        new_row: c_int,
        new_col: c_int,
        new_byte: i64,
        undo: c_int,
    );
    #[link_name = "buf_updates_send_changes"]
    fn nvim_buf_updates_send_changes(
        buf: *mut crate::BufHandle,
        lnum: c_int,
        added: i64,
        deleted: i64,
    );
    fn line_breakcheck();
    fn nvim_exarg_set_nextcmd(eap: *mut ExArgHandle, p: *const c_char);
    fn nvim_excmds_msg_empty();
    fn nvim_do_sub_save_pat(pat: *const c_char, patlen: usize, which_pat: c_int);
    fn nvim_do_sub_set_replacement(sub_str: *const c_char);
}

// Additional C functions used by do_sub
#[allow(dead_code)]
extern "C" {
    fn nvim_exarg_get_cmdidx(eap: *mut ExArgHandle) -> c_int;
    fn nvim_exarg_get_cmd(eap: *const ExArgHandle) -> *const c_char;
    fn nvim_exarg_get_arg(eap: *const ExArgHandle) -> *const c_char;
    fn nvim_exarg_set_line2(eap: *mut ExArgHandle, line2: c_int);
    fn nvim_curwin_get_cursor_lnum() -> c_int;
    fn semsg(fmt: *const c_char, ...);
    fn rs_hasAnyFolding(win: *mut crate::WinHandle) -> c_int;
    fn nvim_get_curwin() -> *mut crate::WinHandle;
    fn rs_print_line(lnum: c_int, use_number: c_int, list: c_int, first: c_int);
    fn rs_print_line_no_prefix(lnum: c_int, use_number: c_int, list: c_int);
    fn xfree(ptr: *mut std::ffi::c_void);
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xstrnsave(string: *const c_char, len: usize) -> *mut c_char;
    fn ml_get(lnum: c_int) -> *const c_char;
    fn ml_get_len(lnum: c_int) -> c_int;
    fn ml_delete(lnum: c_int) -> c_int;
    fn ml_replace(lnum: c_int, line: *mut c_char, copy: c_int);
    fn ml_append(lnum: c_int, line: *const c_char, len: c_int, newfile: c_int) -> c_int;
    fn changed_bytes(lnum: c_int, col: c_int);
    fn deleted_lines(lnum: c_int, count: c_int);
    fn scrollup_clamp();
    fn scrolldown_clamp();
    fn setmouse();
    fn concat_str(s1: *const c_char, s2: *const c_char) -> *mut c_char;
    fn get_search_pat() -> *const c_char;
    fn skipwhite(p: *const c_char) -> *const c_char;
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;
    fn beginline(flags: c_int);
    fn changed_lines(
        buf: *mut crate::BufHandle,
        lnum: c_int,
        col: c_int,
        lnume: c_int,
        xtra: c_int,
        do_buf_event: c_int,
    );
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn msg(s: *const c_char, hl_id: c_int) -> c_int;
    fn emsg(s: *const c_char) -> c_int;
    fn nvim_cmdmod_has_lockmarks() -> c_int;
    fn nvim_cmdmod_has_keeppatterns() -> c_int;
    fn fast_breakcheck();
}

// =============================================================================
// Substitute lifecycle (Phase 4 migration)
// =============================================================================

extern "C" {
    // ex_substitute_preview accessors
    fn nvim_excmds_arg_has_valid_delim(eap: *const ExArgHandle) -> c_int;
    fn nvim_excmds_eap_arg_restore(eap: *mut ExArgHandle, saved: *mut c_char);
}

// =============================================================================
// old_sub: Rust-owned substitute replacement string state
// (Previously stored as a C static; moved here in Phase 2 migration.)
// =============================================================================

/// Rust-owned equivalent of C's `SubReplacementString old_sub`.
/// These are C-heap-allocated pointers (xmalloc/xfree). nvim is single-threaded.
static mut OLD_SUB: *mut c_char = std::ptr::null_mut();
static mut OLD_SUB_TIMESTAMP: u64 = 0;
static mut OLD_SUB_ADDITIONAL_DATA: *mut std::ffi::c_void = std::ptr::null_mut();

// =============================================================================
// Pure Rust helpers (replacing trivial C character/predicate wrappers)
// =============================================================================

/// Returns true if `c` is ASCII whitespace (space or tab).
#[inline]
fn sub_ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Returns true if `c` is an ASCII decimal digit.
#[inline]
fn sub_ascii_isdigit(c: u8) -> bool {
    c.is_ascii_digit()
}

/// Returns true if `c` could be a substitute flag or digit.
/// Matches C: vim_strchr("0123456789cegriIp|\"", c) != NULL
#[inline]
fn sub_is_flag_or_digit(c: u8) -> bool {
    matches!(
        c,
        b'0'..=b'9' | b'c' | b'e' | b'g' | b'r' | b'i' | b'I' | b'p' | b'|' | b'"'
    )
}

/// Returns true if `*cmd == '\\'` and `cmd[1]` is in `/?&`.
/// # Safety
/// `cmd` must point to at least 2 valid bytes (NUL-terminated string).
#[inline]
unsafe fn sub_is_backslash_delim(cmd: *const c_char) -> bool {
    *cmd == b'\\' as i8 && matches!(*cmd.add(1) as u8, b'/' | b'?' | b'&')
}

// =============================================================================
// Phase 3: do_sub Rust implementation
// =============================================================================

/// Static flags that persist across :substitute calls (matches C `static subflags_T subflags`).
static SUBFLAGS: std::sync::Mutex<CSubFlags> = std::sync::Mutex::new(CSubFlags {
    do_all: false,
    do_ask: false,
    do_count: false,
    do_error: true,
    do_print: false,
    do_list: false,
    do_number: false,
    do_ic: 0, // kSubHonorOptions
});

/// Static highlight group id for inccommand preview (matches C `static int pre_hl_id`).
static PRE_HL_ID: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// SubResult holds a single match result for preview.
struct DoSubResult {
    start_lnum: c_int,
    start_col: c_int,
    end_lnum: c_int,
    end_col: c_int,
    pre_match: c_int,
}

/// PreviewLines accumulates match results for inccommand preview.
pub(crate) struct PreviewLines {
    subresults: Vec<DoSubResult>,
    lines_needed: c_int,
}

impl PreviewLines {
    fn new() -> Self {
        Self {
            subresults: Vec::new(),
            lines_needed: 0,
        }
    }

    fn push(&mut self, result: DoSubResult) {
        let match_lines = result.end_lnum - result.start_lnum + 1;
        if let Some(last) = self.subresults.last() {
            if last.end_lnum == result.start_lnum {
                self.lines_needed += match_lines - 1;
            } else {
                self.lines_needed += match_lines;
            }
        } else {
            self.lines_needed += match_lines;
        }
        self.subresults.push(result);
    }
}

/// C-compatible accessor for PreviewLines (used by rs_show_sub).
/// Returns the size of the subresults vector.
///
/// # Safety
/// `pl` must be a valid non-null pointer to a `PreviewLines` value.
#[no_mangle]
#[allow(private_interfaces)]
pub unsafe extern "C" fn nvim_excmds_preview_lines_size_rust(pl: *const PreviewLines) -> usize {
    (*pl).subresults.len()
}

/// C-compatible accessor for a PreviewLines item.
///
/// # Safety
/// `pl` must be valid and `idx` must be in bounds. All output pointers must be valid.
#[no_mangle]
#[allow(private_interfaces)]
pub unsafe extern "C" fn nvim_excmds_preview_lines_item_rust(
    pl: *const PreviewLines,
    idx: usize,
    start_lnum: *mut c_int,
    start_col: *mut c_int,
    end_lnum: *mut c_int,
    end_col: *mut c_int,
    pre_match: *mut c_int,
) {
    let item = &(&(*pl).subresults)[idx];
    *start_lnum = item.start_lnum;
    *start_col = item.start_col;
    *end_lnum = item.end_lnum;
    *end_col = item.end_col;
    *pre_match = item.pre_match;
}

/// LineData holds per-match extmark data for batch processing.
#[allow(dead_code)]
struct LineData {
    start_col: c_int,
    start_lnum: c_int,
    start_lnum_endpos: c_int,
    start_col_endpos: c_int,
    matchcols: c_int,
    matchbytes: i64,
    subcols: c_int,
    subbytes: i64,
    lnum_before: c_int,
    lnum_after: c_int,
}

/// Perform the :substitute command. This is the Rust port of C `do_sub`.
///
/// Returns 0 normally, or 1/2 for inccommand preview.
///
/// # Safety
/// All pointers must be valid. Calls numerous C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_do_sub(
    eap: *mut ExArgHandle,
    cmdpreview_ns: c_int,
    cmdpreview_bufnr: c_int,
    use_rdt: c_int,
) -> c_int {
    // Set up the timeout
    let timeout: u64 = if use_rdt != 0 {
        rs_profile_setlimit(p_rdt)
    } else {
        rs_profile_zero()
    };

    let mut which_pat: c_int;

    // Determine which_pat
    let cmdidx = nvim_exarg_get_cmdidx(eap);
    if cmdidx == CMD_TILDE {
        which_pat = RE_LAST;
    } else {
        which_pat = RE_SUBST;
    }

    let mut cmd = nvim_exarg_get_arg(eap) as *mut c_char;
    let keeppatterns = nvim_cmdmod_has_keeppatterns() != 0;

    let mut pat: *const c_char = std::ptr::null();
    let mut patlen: usize = 0;
    let mut sub: *mut c_char = std::ptr::null_mut();
    let mut _has_second_delim = false;
    #[allow(unused_assignments)]
    let mut delimiter: c_int = 0;

    // Check if we have a new pattern
    let is_s_cmd = *nvim_exarg_get_cmd(eap) as u8 == b's';
    let cmd0 = *cmd as u8;
    let cmd_is_new_pattern =
        is_s_cmd && cmd0 != 0 && !sub_ascii_iswhite(cmd0) && !sub_is_flag_or_digit(cmd0);

    if cmd_is_new_pattern {
        // Check delimiter validity
        if rs_check_regexp_delim(*cmd as c_int) == 0 {
            return 0;
        }

        if sub_is_backslash_delim(cmd) {
            // "\/ sub/" or "\? sub?" style
            cmd = cmd.add(1);
            let dc = *cmd as u8;
            if dc != b'&' {
                which_pat = RE_SEARCH;
            }
            pat = c"".as_ptr();
            patlen = 0;
            delimiter = *cmd as c_int;
            cmd = cmd.add(1);
            _has_second_delim = true;
        } else {
            which_pat = RE_LAST;
            delimiter = *cmd as c_int;
            cmd = cmd.add(1);
            pat = cmd;
            // Skip to end of pattern, updating eap->arg in place
            let mut arg_ptr = nvim_exarg_get_arg(eap) as *mut c_char;
            cmd = nvim_do_sub_skip_regexp_ex(cmd, delimiter, &mut arg_ptr);
            if *cmd == delimiter as i8 {
                *cmd = 0; // replace end delimiter with NUL
                cmd = cmd.add(1);
                _has_second_delim = true;
            }
            patlen = strlen_c(pat);
        }

        // Skip the substitution string
        let sub_start = cmd;
        cmd = rs_skip_substitute(cmd, delimiter);
        sub = xstrdup(sub_start);

        if nvim_exarg_get_skip(eap) == 0 && !keeppatterns && cmdpreview_ns <= 0 {
            nvim_do_sub_set_replacement(sub);
        }
    } else if nvim_exarg_get_skip(eap) == 0 {
        // Use previous pattern and substitution
        if OLD_SUB.is_null() {
            nvim_emsg_nopresub();
            return 0;
        }
        pat = std::ptr::null();
        patlen = 0;
        sub = xstrdup(OLD_SUB);

        // Vi compatibility: if last command used "$", keep cursor in last column
        let endcolumn = nvim_curwin_get_w_curswant() == MAXCOL;
        let _ = endcolumn; // Used later after loop
    }

    // Determine endcolumn now (before sub_joining_lines)
    let endcolumn = if !cmd_is_new_pattern && nvim_exarg_get_skip(eap) == 0 {
        nvim_curwin_get_w_curswant() == MAXCOL
    } else {
        false
    };

    // Check for join optimization (:%s/\n//g -> join)
    if !sub.is_null()
        && rs_sub_joining_lines(
            eap,
            pat,
            patlen,
            sub,
            cmd,
            if cmdpreview_ns <= 0 { 1 } else { 0 },
            if keeppatterns { 1 } else { 0 },
        ) != 0
    {
        xfree(sub as *mut std::ffi::c_void);
        return 0;
    }

    // Parse flags
    let mut subflags_local: CSubFlags;
    {
        let mut sf = SUBFLAGS.lock().unwrap();
        // Clone current flags to local
        subflags_local = CSubFlags {
            do_all: sf.do_all,
            do_ask: sf.do_ask,
            do_count: sf.do_count,
            do_error: sf.do_error,
            do_print: sf.do_print,
            do_list: sf.do_list,
            do_number: sf.do_number,
            do_ic: sf.do_ic,
        };
        // Parse flags from cmd, modifying subflags_local and which_pat
        cmd = rs_sub_parse_flags(cmd, &mut subflags_local, &mut which_pat);
        // Write back to static
        sf.do_all = subflags_local.do_all;
        sf.do_ask = subflags_local.do_ask;
        sf.do_count = subflags_local.do_count;
        sf.do_error = subflags_local.do_error;
        sf.do_print = subflags_local.do_print;
        sf.do_list = subflags_local.do_list;
        sf.do_number = subflags_local.do_number;
        sf.do_ic = subflags_local.do_ic;
    }

    let save_do_all = subflags_local.do_all;
    let save_do_ask = subflags_local.do_ask;

    // Parse trailing count
    cmd = skipwhite(cmd) as *mut c_char;
    if sub_ascii_isdigit(*cmd as u8) {
        let i = getdigits_int(&mut cmd, true, c_int::MAX);
        if i <= 0 && nvim_exarg_get_skip(eap) == 0 && subflags_local.do_error {
            nvim_excmds_emsg_by_id(7); // e_zerocount
            xfree(sub as *mut std::ffi::c_void);
            return 0;
        } else if i == c_int::MAX {
            let buf_str = std::ffi::CString::new(format!("{}", i)).unwrap_or_default();
            nvim_excmds_emsg_with_arg(5, buf_str.as_ptr()); // semsg_val_too_large
            xfree(sub as *mut std::ffi::c_void);
            return 0;
        }
        let line1 = nvim_exarg_get_line1(eap);
        nvim_exarg_set_line2(eap, line1); // eap->line1 = eap->line2
        let new_line2 = line1 + i - 1;
        let buf_line_count = nvim_excmds_curbuf_ml_line_count();
        nvim_exarg_set_line2(eap, new_line2.min(buf_line_count));
    }

    // Check for trailing command or garbage
    cmd = skipwhite(cmd) as *mut c_char;
    if *cmd != 0 && *cmd != b'"' as i8 {
        let nextcmd = check_nextcmd(cmd);
        if nextcmd.is_null() {
            nvim_excmds_emsg_with_arg(4, cmd); // semsg_trailing
            xfree(sub as *mut std::ffi::c_void);
            return 0;
        }
        nvim_exarg_set_nextcmd(eap, nextcmd);
    }

    if nvim_exarg_get_skip(eap) != 0 {
        xfree(sub as *mut std::ffi::c_void);
        return 0;
    }

    if !subflags_local.do_count && !nvim_curbuf_modifiable() {
        nvim_emsg_modifiable();
        xfree(sub as *mut std::ffi::c_void);
        return 0;
    }

    // Compile the regex
    let regmatch = nvim_do_sub_search_regcomp(
        pat,
        patlen,
        which_pat,
        if cmdpreview_ns > 0 { 0 } else { SEARCH_HIS },
    );
    if regmatch.is_null() {
        if subflags_local.do_error {
            nvim_excmds_emsg_by_id(5); // e_invcmd
        }
        xfree(sub as *mut std::ffi::c_void);
        return 0;
    }

    // Apply i/I flags to ignore-case setting
    let do_ic = subflags_local.do_ic;
    if do_ic == 1 {
        // kSubIgnoreCase
        nvim_regmmatch_set_rmm_ic(regmatch, 1);
    } else if do_ic == 2 {
        // kSubMatchCase
        nvim_regmmatch_set_rmm_ic(regmatch, 0);
    }

    // Save sub_firstline (not allocated yet)
    let mut sub_firstline: *mut c_char = std::ptr::null_mut();

    // Process the substitution string: expand ~ or copy
    let sub = if !sub.is_null() && *sub == b'\\' as i8 && *sub.add(1) == b'=' as i8 {
        // Expression substitute: copy it
        let p = xstrdup(sub);
        xfree(sub as *mut std::ffi::c_void);
        p
    } else {
        let p = regtilde(sub, rs_magic_isset(), cmdpreview_ns > 0);
        if p != sub {
            xfree(sub as *mut std::ffi::c_void);
        }
        p
    };

    // Save current globals for the loop
    let old_line_count = nvim_excmds_curbuf_ml_line_count();
    let start_nsubs = crate::sub_nsubs;

    if crate::global_busy == 0 {
        crate::sub_nsubs = 0;
        crate::sub_nlines = 0;
    }

    let mut first_line: c_int = 0;
    let mut last_line: c_int = 0;
    let mut got_quit = false;
    let mut got_match = false;
    let mut did_save = false;

    // Save old cursor position
    let old_cursor_lnum = nvim_curwin_get_cursor_lnum();
    let old_cursor_col = nvim_curwin_get_cursor_col();

    // Inccommand preview data
    let mut preview_lines = PreviewLines::new();

    let mut line2 = nvim_exarg_get_line2(eap);

    // Main substitution loop
    let mut lnum = nvim_exarg_get_line1(eap);
    while lnum <= line2
        && !got_quit
        && aborting() == 0
        && (cmdpreview_ns <= 0
            || preview_lines.lines_needed <= p_cwh as c_int
            || lnum <= nvim_curwin_get_w_botline())
    {
        let nmatch = nvim_do_sub_vim_regexec_multi(regmatch, lnum, 0);
        if nmatch > 0 {
            // Track per-line extmark data
            let mut line_matches: Vec<LineData> = Vec::new();

            let mut copycol: c_int = 0;
            let mut matchcol: c_int = 0;
            let mut prev_matchcol: c_int = MAXCOL;
            let mut new_start: *mut c_char = std::ptr::null_mut();
            let mut new_start_len: c_int = 0;
            let mut did_sub = false;
            let mut lastone: bool;
            let mut nmatch_tl: c_int = 0;
            let mut nmatch = nmatch;
            let mut do_again: bool;
            let mut skip_match = false;
            let mut sub_firstlnum: c_int = lnum;
            let mut lnum_start: c_int = 0;

            if !got_match {
                setpcmark();
                got_match = true;
            }

            // Inner loop: process all matches on this line
            loop {
                #[allow(unused_assignments)]
                let mut cur_start_lnum: c_int = 0;
                let mut cur_start_col: c_int = 0;
                let mut cur_end_lnum: c_int = 0;
                let mut cur_end_col: c_int = 0;
                let mut cur_pre_match: c_int = 0;

                // Advance lnum to where match starts
                let startpos0_lnum = nvim_regmmatch_startpos0_lnum(regmatch);
                if startpos0_lnum > 0 {
                    cur_pre_match = lnum;
                    lnum += startpos0_lnum;
                    sub_firstlnum += startpos0_lnum;
                    nmatch -= startpos0_lnum;
                    xfree(sub_firstline as *mut std::ffi::c_void);
                    sub_firstline = std::ptr::null_mut();
                }

                cur_start_lnum = sub_firstlnum;

                // Check if match is after last line
                if lnum > nvim_excmds_curbuf_ml_line_count() {
                    break;
                }

                if sub_firstline.is_null() {
                    let line = ml_get(sub_firstlnum);
                    sub_firstline = xstrnsave(line, ml_get_len(sub_firstlnum) as usize);
                }

                nvim_curwin_set_cursor_lnum(lnum);
                do_again = false;

                // 1. Empty match handling
                let endpos0_lnum = nvim_regmmatch_endpos0_lnum(regmatch);
                let endpos0_col = nvim_regmmatch_endpos0_col(regmatch);
                let startpos0_col = nvim_regmmatch_startpos0_col(regmatch);

                if matchcol == prev_matchcol && endpos0_lnum == 0 && matchcol == endpos0_col {
                    let c = *sub_firstline.add(matchcol as usize) as u8;
                    if c == 0 {
                        skip_match = true;
                    } else {
                        matchcol += utfc_ptr2len(sub_firstline.add(matchcol as usize));
                    }
                    cur_start_col = matchcol;
                    cur_end_lnum = sub_firstlnum;
                    cur_end_col = matchcol;
                    // goto skip
                    // fall through to skip section
                } else {
                    matchcol = endpos0_col;
                    prev_matchcol = matchcol;

                    // 2. Count-only mode
                    if subflags_local.do_count {
                        if nmatch > 1 {
                            matchcol = strlen_c(sub_firstline) as c_int;
                            nmatch = 1;
                            skip_match = true;
                        }
                        crate::sub_nsubs += 1;
                        did_sub = true;
                        if !(*sub == b'\\' as i8 && *sub.add(1) == b'=' as i8) {
                            // goto skip
                            cur_start_col = startpos0_col;
                            cur_end_lnum = sub_firstlnum;
                            cur_end_col = endpos0_col;
                            // fall through to skip
                        } else {
                            goto_sub_main(
                                &mut subflags_local,
                                regmatch,
                                sub,
                                &mut sub_firstlnum,
                                &mut sub_firstline,
                                &mut lnum,
                                &mut line2,
                                &mut matchcol,
                                &mut copycol,
                                &mut new_start,
                                &mut new_start_len,
                                &mut nmatch,
                                &mut nmatch_tl,
                                &mut did_sub,
                                &mut did_save,
                                &mut skip_match,
                                &mut first_line,
                                &mut last_line,
                                &mut line_matches,
                                &mut do_again,
                                &mut lnum_start,
                                &mut cur_start_lnum,
                                &mut cur_start_col,
                                &mut cur_end_lnum,
                                &mut cur_end_col,
                                cmdpreview_ns,
                                eap,
                            );
                        }
                    } else {
                        // do_ask handling and substitution
                        if subflags_local.do_ask && cmdpreview_ns <= 0 {
                            let typed = handle_do_ask(
                                &mut subflags_local,
                                regmatch,
                                sub,
                                lnum,
                                sub_firstlnum,
                                &mut lnum_start,
                                &mut copycol,
                                new_start,
                                sub_firstline,
                                &mut line2,
                                &mut nmatch,
                                &mut matchcol,
                                &mut skip_match,
                                &mut got_quit,
                                eap,
                            );
                            if typed < 0 || got_quit {
                                cur_start_col = startpos0_col;
                                cur_end_lnum = sub_firstlnum;
                                cur_end_col = endpos0_col;
                            } else if typed == b'n' as c_int {
                                if nmatch > 1 {
                                    matchcol = strlen_c(sub_firstline) as c_int;
                                    skip_match = true;
                                }
                                cur_start_col = startpos0_col;
                                cur_end_lnum = sub_firstlnum;
                                cur_end_col = endpos0_col;
                            } else {
                                // y or a - do substitution
                                goto_sub_main(
                                    &mut subflags_local,
                                    regmatch,
                                    sub,
                                    &mut sub_firstlnum,
                                    &mut sub_firstline,
                                    &mut lnum,
                                    &mut line2,
                                    &mut matchcol,
                                    &mut copycol,
                                    &mut new_start,
                                    &mut new_start_len,
                                    &mut nmatch,
                                    &mut nmatch_tl,
                                    &mut did_sub,
                                    &mut did_save,
                                    &mut skip_match,
                                    &mut first_line,
                                    &mut last_line,
                                    &mut line_matches,
                                    &mut do_again,
                                    &mut lnum_start,
                                    &mut cur_start_lnum,
                                    &mut cur_start_col,
                                    &mut cur_end_lnum,
                                    &mut cur_end_col,
                                    cmdpreview_ns,
                                    eap,
                                );
                            }
                        } else {
                            // Normal substitution
                            goto_sub_main(
                                &mut subflags_local,
                                regmatch,
                                sub,
                                &mut sub_firstlnum,
                                &mut sub_firstline,
                                &mut lnum,
                                &mut line2,
                                &mut matchcol,
                                &mut copycol,
                                &mut new_start,
                                &mut new_start_len,
                                &mut nmatch,
                                &mut nmatch_tl,
                                &mut did_sub,
                                &mut did_save,
                                &mut skip_match,
                                &mut first_line,
                                &mut last_line,
                                &mut line_matches,
                                &mut do_again,
                                &mut lnum_start,
                                &mut cur_start_lnum,
                                &mut cur_start_col,
                                &mut cur_end_lnum,
                                &mut cur_end_col,
                                cmdpreview_ns,
                                eap,
                            );
                        }
                    }
                }

                // Skip label equivalent - determine lastone
                lastone = skip_match
                    || crate::got_int
                    || got_quit
                    || lnum > line2
                    || !(subflags_local.do_all || do_again)
                    || (*sub_firstline.add(matchcol as usize) == 0
                        && nmatch <= 1
                        && nvim_regmmatch_re_multiline(regmatch) == 0);
                let prev_nmatch = nmatch;
                nmatch = -1;

                // Replace line if needed
                if lastone
                    || nmatch_tl > 0
                    || {
                        let m = nvim_do_sub_vim_regexec_multi(regmatch, sub_firstlnum, matchcol);
                        nmatch = m;
                        m == 0
                    }
                    || nvim_regmmatch_startpos0_lnum(regmatch) > 0
                {
                    if !new_start.is_null() {
                        // Append rest of line
                        let rest = sub_firstline.add(copycol as usize);
                        let new_start_len_str = strlen_c(new_start);
                        let rest_len = strlen_c(rest);
                        // Manually append (we need to strcat equivalent)
                        let dst = new_start.add(new_start_len_str);
                        std::ptr::copy_nonoverlapping(rest, dst, rest_len + 1);

                        matchcol = strlen_c(sub_firstline) as c_int - matchcol;
                        prev_matchcol = strlen_c(sub_firstline) as c_int - prev_matchcol;

                        if nvim_u_savesub(lnum) == 0 {
                            break;
                        }
                        ml_replace(lnum, new_start, 1);

                        // Process extmarks
                        let curbuf_for_splice = nvim_get_curbuf();
                        for md in &line_matches {
                            extmark_splice(
                                curbuf_for_splice,
                                md.lnum_before - 1,
                                md.start_col,
                                md.start_lnum_endpos - md.start_lnum,
                                md.matchcols,
                                md.matchbytes,
                                md.lnum_after - md.lnum_before,
                                md.subcols,
                                md.subbytes,
                                KEXTMARK_UNDO,
                            );
                        }
                        line_matches.clear();

                        if nmatch_tl > 0 {
                            lnum += 1;
                            if nvim_u_savedel2(lnum, nmatch_tl) == 0 {
                                break;
                            }
                            for _ in 0..nmatch_tl {
                                ml_delete(lnum);
                            }
                            mark_adjust(
                                lnum,
                                lnum + nmatch_tl - 1,
                                MAXLNUM,
                                -nmatch_tl,
                                KEXTMARK_NOOP,
                            );
                            if subflags_local.do_ask {
                                deleted_lines(lnum, nmatch_tl);
                            }
                            lnum -= 1;
                            line2 -= nmatch_tl;
                            nmatch_tl = 0;
                        }

                        if subflags_local.do_ask {
                            changed_bytes(lnum, 0);
                        } else {
                            if first_line == 0 {
                                first_line = lnum;
                            }
                            last_line = lnum + 1;
                        }

                        sub_firstlnum = lnum;
                        xfree(sub_firstline as *mut std::ffi::c_void);
                        sub_firstline = new_start;
                        new_start = std::ptr::null_mut();
                        matchcol = strlen_c(sub_firstline) as c_int - matchcol;
                        prev_matchcol = strlen_c(sub_firstline) as c_int - prev_matchcol;
                        copycol = 0;
                    }

                    if nmatch == -1 && !lastone {
                        nmatch = nvim_do_sub_vim_regexec_multi(regmatch, sub_firstlnum, matchcol);
                    }

                    // Break if no more matches
                    if nmatch <= 0 {
                        if prev_nmatch == -1 {
                            lnum -= nvim_regmmatch_startpos0_lnum(regmatch);
                        }
                        // PUSH_PREVIEW_LINES
                        if cmdpreview_ns > 0 {
                            if cur_end_lnum == 0 {
                                cur_end_lnum = cur_start_lnum;
                            }
                            preview_lines.push(DoSubResult {
                                start_lnum: cur_start_lnum,
                                start_col: cur_start_col,
                                end_lnum: cur_end_lnum,
                                end_col: cur_end_col,
                                pre_match: cur_pre_match,
                            });
                        }
                        break;
                    }
                }

                // PUSH_PREVIEW_LINES (for when we're continuing)
                if cmdpreview_ns > 0 {
                    if cur_end_lnum == 0 {
                        cur_end_lnum = cur_start_lnum;
                    }
                    preview_lines.push(DoSubResult {
                        start_lnum: cur_start_lnum,
                        start_col: cur_start_col,
                        end_lnum: cur_end_lnum,
                        end_col: cur_end_col,
                        pre_match: cur_pre_match,
                    });
                }

                line_breakcheck();
            } // end inner while

            if did_sub {
                crate::sub_nlines += 1;
            }
            xfree(new_start as *mut std::ffi::c_void);
            xfree(sub_firstline as *mut std::ffi::c_void);
            sub_firstline = std::ptr::null_mut();
        }

        line_breakcheck();

        if rs_profile_passed_limit(timeout) {
            got_quit = true;
        }

        lnum += 1;
    }

    // Post-loop cleanup
    nvim_curbuf_set_deleted_bytes2(0);

    if first_line != 0 {
        let i = nvim_excmds_curbuf_ml_line_count() - old_line_count;
        nvim_do_sub_changed_lines(first_line, last_line - i, i);

        let num_added = (last_line - first_line) as i64;
        let num_removed = num_added - i as i64;
        nvim_buf_updates_send_changes(nvim_get_curbuf(), first_line, num_added, num_removed);
    }

    xfree(sub_firstline as *mut std::ffi::c_void);

    // Restore cursor if count-only
    if subflags_local.do_count {
        nvim_curwin_set_cursor_lnum(old_cursor_lnum);
        nvim_curwin_set_cursor_col(old_cursor_col);
    }

    let cur_sub_nsubs = crate::sub_nsubs;
    if cur_sub_nsubs > start_nsubs {
        if nvim_cmdmod_has_lockmarks() == 0 {
            nvim_do_sub_set_op_start_end(nvim_exarg_get_line1(eap), line2);
        }

        if crate::global_busy == 0 {
            if !subflags_local.do_ask {
                if endcolumn {
                    nvim_coladvance(MAXCOL);
                } else {
                    // BL_WHITE | BL_FIX = 1 | 4 = 5
                    beginline(5);
                }
            }
            if cmdpreview_ns <= 0
                && !rs_do_sub_msg(subflags_local.do_count)
                && subflags_local.do_ask
                && p_ch > 0
            {
                nvim_excmds_emsg_by_id(8); // msg_empty
            }
        } else {
            crate::global_need_beginline = 1;
        }
        if subflags_local.do_print {
            rs_print_line(
                nvim_curwin_get_cursor_lnum(),
                subflags_local.do_number as c_int,
                subflags_local.do_list as c_int,
                1,
            );
        }
    } else if crate::global_busy == 0 {
        if crate::got_int {
            nvim_excmds_emsg_interr();
        } else if got_match {
            if p_ch > 0 {
                nvim_excmds_emsg_by_id(8); // msg_empty
            }
        } else if subflags_local.do_error {
            let pat_str = get_search_pat();
            nvim_excmds_emsg_with_arg(3, pat_str); // semsg_patnotf2
        }
    }

    if subflags_local.do_ask && rs_hasAnyFolding(nvim_get_curwin()) != 0 {
        changed_window_setting(crate::nvim_get_curwin());
    }

    nvim_excmds_vim_regfree_multi(regmatch as *mut std::ffi::c_void);
    xfree(sub as *mut std::ffi::c_void);

    // Restore saved flags
    {
        let mut sf = SUBFLAGS.lock().unwrap();
        sf.do_all = save_do_all;
        sf.do_ask = save_do_ask;
    }

    let mut retv: c_int = 0;

    // Show inccommand preview
    if cmdpreview_ns > 0 && aborting() == 0 {
        if got_quit || rs_profile_passed_limit(timeout) {
            nvim_excmds_disable_inccommand();
        } else if !p_icm.is_null() && *p_icm != 0 && !pat.is_null() {
            let mut hl_id = PRE_HL_ID.load(std::sync::atomic::Ordering::Relaxed);
            if hl_id == 0 {
                hl_id = syn_check_group(c"Substitute".as_ptr(), 10);
                PRE_HL_ID.store(hl_id, std::sync::atomic::Ordering::Relaxed);
            }
            // Call rs_show_sub with the preview_lines
            retv = rs_show_sub(
                eap,
                old_cursor_lnum,
                old_cursor_col,
                &preview_lines as *const PreviewLines as *const std::ffi::c_void,
                hl_id,
                cmdpreview_ns,
                cmdpreview_bufnr,
            );
        }
    }

    retv
}

/// Helper: strlen for a C string pointer.
unsafe fn strlen_c(s: *const c_char) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

/// Handle the do_ask UI loop for :s/foo/bar/c interactive confirmation.
/// Returns the typed character, or -1 if got_quit.
#[allow(clippy::too_many_arguments)]
unsafe fn handle_do_ask(
    subflags: &mut CSubFlags,
    regmatch: *mut crate::RegmmatchHandle,
    sub: *const c_char,
    lnum: c_int,
    _sub_firstlnum: c_int,
    lnum_start: &mut c_int,
    copycol: &mut c_int,
    new_start: *mut c_char,
    sub_firstline: *const c_char,
    line2: &mut c_int,
    nmatch: &mut c_int,
    matchcol: &mut c_int,
    skip_match: &mut bool,
    got_quit: &mut bool,
    eap: *mut ExArgHandle,
) -> c_int {
    let _ = eap;
    let _ = lnum_start;

    let save_state = nvim_curwin_get_cursor_col(); // store State proxy
    let startpos0_col = nvim_regmmatch_startpos0_col(regmatch);
    nvim_curwin_set_cursor_col(startpos0_col);

    if nvim_curwin_get_w_p_crb() != 0 {
        do_check_cursorbind();
    }

    if !vim_strchr(p_cpo, b'u' as c_int).is_null() {
        nvim_inc_no_u_sync();
    }

    let mut typed: c_int = 0;

    while subflags.do_ask {
        if exmode_active {
            // Exmode: use getcmdline_prompt
            rs_print_line_no_prefix(lnum, subflags.do_number as c_int, subflags.do_list as c_int);

            let endpos0_col = nvim_regmmatch_endpos0_col(regmatch);
            let mut sc: c_int = 0;
            let mut ec: c_int = 0;
            nvim_do_sub_getvcol_start_end(
                lnum,
                startpos0_col,
                0_i32.max(endpos0_col - 1),
                &mut sc,
                &mut ec,
            );
            nvim_curwin_set_cursor_col(startpos0_col);

            if subflags.do_number || nvim_curwin_get_w_p_nu() != 0 {
                // We can't easily get w_p_nu in isolation - use nvim_curwin_get_w_p_nu from lib
                let numw = number_width(crate::nvim_get_curwin()) + 1;
                sc += numw;
                ec += numw;
            }

            // Build prompt: sc spaces + (ec-sc+1) carets
            let prompt_len = (ec + 1) as usize + 1;
            let prompt = libc_alloc(prompt_len);
            std::ptr::write_bytes(prompt, b' ', sc as usize);
            std::ptr::write_bytes(prompt.add(sc as usize), b'^', (ec - sc + 1) as usize);
            *prompt.add(ec as usize + 1) = 0;

            typed = nvim_do_sub_getcmdline_prompt(prompt);
            libc_free(prompt);

            if ex_normal_busy != 0 && typed == 0 {
                typed = b'q' as c_int;
            }
        } else {
            // Normal mode: show highlighted match, then prompt
            let mut orig_line: *mut c_char = std::ptr::null_mut();
            let mut len_change: c_int = 0;
            let save_p_lz = p_lz;
            let save_p_fen = nvim_curwin_get_w_p_fen();

            nvim_curwin_set_w_p_fen(0);

            let temp = nvim_get_RedrawingDisabled();
            nvim_set_RedrawingDisabled(0);

            p_lz = 0;

            if !new_start.is_null() {
                let orig = ml_get(lnum);
                let orig_len = ml_get_len(lnum);
                orig_line = xstrnsave(orig, orig_len as usize);
                let new_line = concat_str(new_start, sub_firstline.add(*copycol as usize));
                len_change = strlen_c(new_line) as c_int - strlen_c(orig_line) as c_int;
                let cur_col = nvim_curwin_get_cursor_col() + len_change;
                nvim_curwin_set_cursor_col(cur_col);
                ml_replace(lnum, new_line, 0);
            }

            let endpos0_lnum = nvim_regmmatch_endpos0_lnum(regmatch);
            let endpos0_col = nvim_regmmatch_endpos0_col(regmatch);
            let match_endcol = endpos0_col + len_change;
            let match_lines = endpos0_lnum - startpos0_col; // actually endpos0_lnum - startpos0_lnum
            nvim_set_search_match_lines(endpos0_lnum - nvim_regmmatch_startpos0_lnum(regmatch));
            nvim_set_search_match_endcol(match_endcol);
            if nvim_regmmatch_endpos0_lnum(regmatch) - nvim_regmmatch_startpos0_lnum(regmatch) == 0
                && match_endcol == 0
            {
                nvim_set_search_match_endcol(1);
            }
            nvim_set_highlight_match(1);

            nvim_do_sub_update_screen_for_confirm();

            nvim_curwin_set_w_p_fen(save_p_fen);

            let prompt_str = nvim_do_sub_format_confirm_prompt(sub);
            typed = prompt_for_input(prompt_str, 18, true, std::ptr::null_mut());
            nvim_set_highlight_match(0);
            xfree(prompt_str as *mut std::ffi::c_void);

            msg_didout = false;
            nvim_al_gotocmdline(1);
            p_lz = save_p_lz;
            nvim_set_RedrawingDisabled(temp);

            if !orig_line.is_null() {
                ml_replace(lnum, orig_line, 0);
            }
            let _ = match_lines;
            let _ = save_state;
        }

        need_wait_return = false;
        if typed == b'q' as c_int || typed == 27 /* ESC */ || typed == 3
        /* Ctrl-C */
        {
            *got_quit = true;
            break;
        }
        if typed == b'n' as c_int {
            break;
        }
        if typed == b'y' as c_int {
            break;
        }
        if typed == b'l' as c_int {
            subflags.do_all = false;
            *line2 = lnum;
            break;
        }
        if typed == b'a' as c_int {
            subflags.do_ask = false;
            break;
        }
        if typed == 5
        /* Ctrl-E */
        {
            scrollup_clamp();
        } else if typed == 25
        /* Ctrl-Y */
        {
            scrolldown_clamp();
        }
    }

    // Restore state
    setmouse();
    if !vim_strchr(p_cpo, b'u' as c_int).is_null() {
        nvim_dec_no_u_sync();
    }

    if typed == b'n' as c_int && *nmatch > 1 {
        *matchcol = strlen_c(sub_firstline) as c_int;
        *skip_match = true;
    }

    typed
}

/// Allocate a zero-initialized buffer of size n bytes using C xcalloc.
unsafe fn libc_alloc(n: usize) -> *mut c_char {
    xcalloc(n, 1) as *mut c_char
}

/// Free memory allocated by C.
unsafe fn libc_free(p: *mut c_char) {
    xfree(p as *mut std::ffi::c_void);
}

/// Helper: get w_p_nu from curwin.
unsafe fn nvim_curwin_get_w_p_nu() -> c_int {
    extern "C" {
        fn nvim_curwin_get_w_p_nu() -> c_int;
    }
    nvim_curwin_get_w_p_nu()
}

/// Do the actual substitution for one match. This is the "step 3" logic
/// from do_sub. Returns (cur_end_lnum, cur_end_col) via out params.
#[allow(clippy::too_many_arguments)]
unsafe fn goto_sub_main(
    subflags: &mut CSubFlags,
    regmatch: *mut crate::RegmmatchHandle,
    sub: *const c_char,
    sub_firstlnum: &mut c_int,
    sub_firstline: &mut *mut c_char,
    lnum: &mut c_int,
    line2: &mut c_int,
    _matchcol: &mut c_int,
    copycol: &mut c_int,
    new_start: &mut *mut c_char,
    new_start_len: &mut c_int,
    nmatch: &mut c_int,
    nmatch_tl: &mut c_int,
    did_sub: &mut bool,
    did_save: &mut bool,
    skip_match: &mut bool,
    first_line: &mut c_int,
    last_line: &mut c_int,
    line_matches: &mut Vec<LineData>,
    do_again: &mut bool,
    lnum_start: &mut c_int,
    _cur_start_lnum: &mut c_int,
    cur_start_col: &mut c_int,
    cur_end_lnum: &mut c_int,
    cur_end_col: &mut c_int,
    cmdpreview_ns: c_int,
    _eap: *mut ExArgHandle,
) {
    let startpos0_col = nvim_regmmatch_startpos0_col(regmatch);
    let startpos0_lnum = nvim_regmmatch_startpos0_lnum(regmatch);
    let endpos0_col = nvim_regmmatch_endpos0_col(regmatch);
    let endpos0_lnum = nvim_regmmatch_endpos0_lnum(regmatch);

    // Move cursor to start of match
    nvim_curwin_set_cursor_col(startpos0_col);

    // Clamp nmatch to available lines
    let buf_line_count = nvim_excmds_curbuf_ml_line_count();
    if *nmatch > buf_line_count - *sub_firstlnum + 1 {
        *nmatch = buf_line_count - *sub_firstlnum + 1;
        *cur_end_lnum = *sub_firstlnum + *nmatch;
        *skip_match = true;
        if *nmatch < 0 {
            return;
        }
    }

    // Preview mode: just record the match, don't substitute
    if cmdpreview_ns > 0 && !(*sub == b'\\' as i8 && *sub.add(1) == b'=' as i8) {
        *cur_start_col = startpos0_col;
        if *cur_end_lnum == 0 {
            *cur_end_lnum = *sub_firstlnum + *nmatch - 1;
        }
        *cur_end_col = endpos0_col;

        // ADJUST_SUB_FIRSTLNUM
        adjust_sub_firstlnum(
            nmatch,
            sub_firstlnum,
            sub_firstline,
            line2,
            do_again,
            skip_match,
        );
        *lnum += *nmatch - 1;
        return;
    }

    // 3. Do actual substitution
    *lnum_start = *lnum;
    let save_ma = nvim_curbuf_get_b_p_ma();
    let save_sandbox = nvim_get_sandbox();

    if subflags.do_count {
        nvim_curbuf_set_b_p_ma(0);
        nvim_inc_sandbox();
    }

    let subflags_save = CSubFlags {
        do_all: subflags.do_all,
        do_ask: subflags.do_ask,
        do_count: subflags.do_count,
        do_error: subflags.do_error,
        do_print: subflags.do_print,
        do_list: subflags.do_list,
        do_number: subflags.do_number,
        do_ic: subflags.do_ic,
    };

    nvim_inc_textlock();
    let source_lnum = *sub_firstlnum - startpos0_lnum;
    // Measurement call: pass sub_firstline as dest with destlen=0 to avoid null-dest error.
    // vim_regsub_multi with destlen=0 just measures the required length without modifying dest.
    let mut sublen = nvim_do_sub_vim_regsub_multi(
        regmatch,
        source_lnum,
        sub,
        *sub_firstline,
        0,
        REGSUB_BACKSLASH
            | if rs_magic_isset() != 0 {
                REGSUB_MAGIC
            } else {
                0
            },
    );
    nvim_dec_textlock();

    // Restore flags from any recursive call
    subflags.do_all = subflags_save.do_all;
    subflags.do_ask = subflags_save.do_ask;
    subflags.do_count = subflags_save.do_count;
    subflags.do_error = subflags_save.do_error;
    subflags.do_print = subflags_save.do_print;
    subflags.do_list = subflags_save.do_list;
    subflags.do_number = subflags_save.do_number;
    subflags.do_ic = subflags_save.do_ic;

    if sublen == 0 || aborting() != 0 || subflags.do_count {
        nvim_curbuf_set_b_p_ma(save_ma);
        nvim_dec_sandbox();
        // Undo sandbox increment
        for _ in 0..(nvim_get_sandbox() - save_sandbox) {
            nvim_dec_sandbox();
        }
        return;
    }

    // Restore sandbox
    nvim_curbuf_set_b_p_ma(save_ma);
    while nvim_get_sandbox() > save_sandbox {
        nvim_dec_sandbox();
    }

    // Get the line for the last matched line
    let p1: *const c_char = if *nmatch == 1 {
        *sub_firstline
    } else {
        let tl_lnum = *sub_firstlnum + *nmatch - 1;
        *nmatch_tl += *nmatch - 1;
        ml_get(tl_lnum)
    };

    let copy_len = startpos0_col - *copycol;
    let p1_len = strlen_c(p1) as c_int;
    let needed = p1_len - endpos0_col + copy_len + sublen + 1;
    let mut new_end = rs_sub_grow_buf(new_start, new_start_len, needed);

    // Copy text before match
    std::ptr::copy_nonoverlapping(
        (*sub_firstline).add(*copycol as usize),
        new_end,
        copy_len as usize,
    );
    new_end = new_end.add(copy_len as usize);

    if *new_start_len - copy_len < sublen {
        sublen = *new_start_len - copy_len - 1;
    }

    let start_col = new_end.offset_from(*new_start) as c_int;
    *cur_start_col = start_col;

    nvim_inc_textlock();
    nvim_do_sub_vim_regsub_multi(
        regmatch,
        source_lnum,
        sub,
        new_end,
        sublen,
        REGSUB_COPY
            | REGSUB_BACKSLASH
            | if rs_magic_isset() != 0 {
                REGSUB_MAGIC
            } else {
                0
            },
    );
    nvim_dec_textlock();
    crate::sub_nsubs += 1;
    *did_sub = true;

    // Move cursor to start of line
    nvim_curwin_set_cursor_col(0);

    // Remember next char to copy
    *copycol = endpos0_col;

    // ADJUST_SUB_FIRSTLNUM
    adjust_sub_firstlnum(
        nmatch,
        sub_firstlnum,
        sub_firstline,
        line2,
        do_again,
        skip_match,
    );

    // Calculate bytes replaced
    let mut replaced_bytes: i64 = 0;
    for i in 0..(*nmatch - 1) {
        let line_len = ml_get_len(*lnum_start + i);
        replaced_bytes += line_len as i64 + 1;
    }
    replaced_bytes += endpos0_col as i64 - startpos0_col as i64;

    let lnum_before_newlines = *lnum;

    // Process CTRL-M chars -> actual line breaks
    let mut p_iter = new_end;
    while *p_iter != 0 {
        if *p_iter == b'\\' as i8 && *p_iter.add(1) != 0 {
            sublen -= 1;
            // STRMOVE(p_iter, p_iter+1)
            let src = p_iter.add(1);
            let len = strlen_c(src) + 1;
            std::ptr::copy(src, p_iter, len);
        } else if *p_iter == b'\r' as i8 {
            if nvim_u_inssub(*lnum) != 0 {
                *p_iter = 0; // truncate at CR
                let new_start_ptr = *new_start;
                let append_len = (p_iter.offset_from(new_start_ptr) + 1) as c_int;
                ml_append(*lnum - 1, new_start_ptr, append_len, 0);
                mark_adjust(*lnum + 1, MAXLNUM, 1, 0, KEXTMARK_NOOP);

                if subflags.do_ask {
                    appended_lines(*lnum - 1, 1);
                } else {
                    if *first_line == 0 {
                        *first_line = *lnum;
                    }
                    *last_line = *lnum + 1;
                }
                *sub_firstlnum += 1;
                *lnum += 1;
                *line2 += 1;
                nvim_curwin_set_cursor_lnum(nvim_curwin_get_cursor_lnum() + 1);

                // STRMOVE(new_start, p_iter+1)
                let src = p_iter.add(1);
                let len = strlen_c(src) + 1;
                std::ptr::copy(src, *new_start, len);
                p_iter = (*new_start).sub(1); // will be incremented
            }
        } else {
            let advance = utfc_ptr2len(p_iter) - 1;
            if advance > 0 {
                p_iter = p_iter.add(advance as usize);
            }
        }
        p_iter = p_iter.add(1);
    }

    let new_endcol = strlen_c(*new_start) as c_int;
    *cur_end_col = new_endcol;
    *cur_end_lnum = *lnum;

    let matchcols = endpos0_col
        - if endpos0_lnum == startpos0_lnum {
            startpos0_col
        } else {
            0
        };
    let subcols = new_endcol - if *lnum == *lnum_start { start_col } else { 0 };

    if !*did_save {
        let _ = nvim_u_save_cursor();
        *did_save = true;
    }

    // Store extmark data for this match
    line_matches.push(LineData {
        start_col,
        start_lnum: startpos0_lnum,
        start_lnum_endpos: endpos0_lnum,
        start_col_endpos: endpos0_col,
        matchcols,
        matchbytes: replaced_bytes,
        subcols,
        subbytes: sublen as i64 - 1,
        lnum_before: lnum_before_newlines,
        lnum_after: *lnum,
    });
}

/// Inline ADJUST_SUB_FIRSTLNUM macro logic.
unsafe fn adjust_sub_firstlnum(
    nmatch: &mut c_int,
    sub_firstlnum: &mut c_int,
    sub_firstline: &mut *mut c_char,
    line2: &mut c_int,
    do_again: &mut bool,
    skip_match: &mut bool,
) {
    if *nmatch > 1 {
        *sub_firstlnum += *nmatch - 1;
        xfree(*sub_firstline as *mut std::ffi::c_void);
        let line = ml_get(*sub_firstlnum);
        *sub_firstline = xstrnsave(line, ml_get_len(*sub_firstlnum) as usize);
        if *sub_firstlnum <= *line2 {
            *do_again = true;
        } else {
            // subflags.do_all = false -- handled by caller
        }
    }
    if *skip_match {
        xfree(*sub_firstline as *mut std::ffi::c_void);
        *sub_firstline = xstrdup(c"".as_ptr());
    }
}

// =============================================================================
// C-compatible opaque type matching SubReplacementString:
// char*, uint64_t (Timestamp), AdditionalData* (opaque).
// =============================================================================

/// C-compatible opaque type matching SubReplacementString:
/// char*, uint64_t (Timestamp), AdditionalData* (opaque).
#[repr(C)]
pub struct SubReplacementStringC {
    pub sub: *mut c_char,
    pub timestamp: u64,
    pub additional_data: *mut std::ffi::c_void,
}

/// Get old substitute replacement string. Replaces C `sub_get_replacement`.
///
/// # Safety
/// `ret_sub` must be a valid pointer to a SubReplacementStringC.
#[no_mangle]
pub unsafe extern "C" fn rs_sub_get_replacement(ret_sub: *mut SubReplacementStringC) {
    (*ret_sub).sub = OLD_SUB;
    (*ret_sub).timestamp = OLD_SUB_TIMESTAMP;
    (*ret_sub).additional_data = OLD_SUB_ADDITIONAL_DATA;
}

/// Set substitute string and timestamp. Replaces C `sub_set_replacement`.
///
/// Called from C with three separate arguments (matching the C calling convention
/// for the thin wrapper that extracts the struct fields).
///
/// # Safety
/// `sub_ptr` must be C-allocated (or NULL), `additional_data` must be C-allocated (or NULL).
#[no_mangle]
pub unsafe extern "C" fn rs_sub_set_replacement(
    sub_ptr: *mut c_char,
    timestamp: u64,
    additional_data: *mut std::ffi::c_void,
) {
    // Free old values before overwriting (matching C semantics in nvim_excmds_old_sub_set).
    extern "C" {
        fn xfree(ptr: *mut std::ffi::c_void);
    }
    xfree(OLD_SUB as *mut std::ffi::c_void);
    if OLD_SUB_ADDITIONAL_DATA != additional_data {
        xfree(OLD_SUB_ADDITIONAL_DATA);
    }
    OLD_SUB = sub_ptr;
    OLD_SUB_TIMESTAMP = timestamp;
    OLD_SUB_ADDITIONAL_DATA = additional_data;
}

/// EXITFREE cleanup for old_sub. Replaces C `free_old_sub`.
///
/// # Safety
/// Frees Rust-owned old_sub state.
#[export_name = "free_old_sub"]
pub unsafe extern "C" fn rs_free_old_sub() {
    rs_sub_set_replacement(std::ptr::null_mut(), 0, std::ptr::null_mut());
}

/// `:substitute` entry point. Replaces C `ex_substitute`.
///
/// # Safety
/// `eap` must be a valid exarg_T pointer.
#[export_name = "ex_substitute"]
pub unsafe extern "C" fn rs_ex_substitute(eap: *mut ExArgHandle) {
    rs_do_sub(eap, 0, 0, 0);
}

/// `:substitute` inccommand preview callback. Replaces C `ex_substitute_preview`.
///
/// Returns 0, 1, or 2 (see cmdpreview_may_show()).
///
/// # Safety
/// `eap` must be a valid exarg_T pointer.
#[export_name = "ex_substitute_preview"]
pub unsafe extern "C" fn rs_ex_substitute_preview(
    eap: *mut ExArgHandle,
    cmdpreview_ns: c_int,
    cmdpreview_bufnr: c_int,
) -> c_int {
    // Only preview once the pattern delimiter has been typed:
    // proceed when *eap->arg is non-NUL and NOT alphanumeric (a valid delimiter).
    if nvim_excmds_arg_has_valid_delim(eap) != 0 {
        let save_eap = nvim_exarg_get_arg(eap as *const ExArgHandle) as *mut c_char;
        let retv = rs_do_sub(eap, cmdpreview_ns, cmdpreview_bufnr, 1);
        nvim_excmds_eap_arg_restore(eap, save_eap);
        return retv;
    }
    0
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_flags_new() {
        let flags = SubFlags::new();
        assert!(!flags.do_all);
        assert!(!flags.do_ask);
        assert!(!flags.do_count);
        assert!(flags.do_error); // Default is to report errors
        assert!(!flags.do_print);
        assert!(!flags.do_list);
        assert!(!flags.do_number);
        assert_eq!(flags.do_ic, SubIgnoreType::HonorOptions);
    }

    #[test]
    fn test_sub_flags_global() {
        let flags = SubFlags::global();
        assert!(flags.do_all);
        assert!(!flags.do_ask);
    }

    #[test]
    fn test_sub_flags_count_only() {
        let flags = SubFlags::count_only();
        assert!(flags.do_all);
        assert!(flags.do_count);
        assert!(flags.is_count_only());
    }

    #[test]
    fn test_sub_flags_parse() {
        // Empty string
        let flags = SubFlags::parse("").unwrap();
        assert!(!flags.do_all);

        // Global flag
        let flags = SubFlags::parse("g").unwrap();
        assert!(flags.do_all);

        // Multiple flags
        let flags = SubFlags::parse("gc").unwrap();
        assert!(flags.do_all);
        assert!(flags.do_ask);
        assert!(flags.needs_confirm());

        // Case flags
        let flags = SubFlags::parse("gi").unwrap();
        assert!(flags.do_all);
        assert_eq!(flags.do_ic, SubIgnoreType::IgnoreCase);

        let flags = SubFlags::parse("gI").unwrap();
        assert!(flags.do_all);
        assert_eq!(flags.do_ic, SubIgnoreType::MatchCase);

        // Error suppression
        let flags = SubFlags::parse("e").unwrap();
        assert!(!flags.do_error);

        // Print flags
        let flags = SubFlags::parse("p").unwrap();
        assert!(flags.do_print);

        let flags = SubFlags::parse("l").unwrap();
        assert!(flags.do_list);

        let flags = SubFlags::parse("#").unwrap();
        assert!(flags.do_number);
    }

    #[test]
    fn test_sub_flags_parse_invalid() {
        // Invalid flag character
        let result = SubFlags::parse("gx");
        assert!(matches!(result, Err(SubstituteError::InvalidFlag('x'))));
    }

    #[test]
    fn test_sub_result() {
        let mut result = SubResult::new();
        assert!(!result.has_matches());

        result.add_match();
        assert!(result.has_matches());
        assert_eq!(result.count, 1);

        result.add_line();
        assert_eq!(result.lines, 1);

        result.set_interrupted();
        assert!(result.interrupted);
    }

    #[test]
    fn test_sub_stats() {
        let mut stats = SubStats::new();
        assert_eq!(stats.total_subs, 0);

        let result = SubResult {
            count: 5,
            lines: 3,
            interrupted: false,
        };
        stats.add_result(&result);
        assert_eq!(stats.total_subs, 5);
        assert_eq!(stats.total_lines, 3);

        stats.reset();
        assert_eq!(stats.total_subs, 0);
        assert_eq!(stats.total_lines, 0);
    }

    #[test]
    fn test_is_valid_delimiter() {
        // Valid delimiters
        assert!(is_valid_delimiter('/'));
        assert!(is_valid_delimiter('#'));
        assert!(is_valid_delimiter('@'));
        assert!(is_valid_delimiter('!'));
        assert!(is_valid_delimiter(':'));

        // Invalid delimiters (alphanumeric)
        assert!(!is_valid_delimiter('a'));
        assert!(!is_valid_delimiter('Z'));
        assert!(!is_valid_delimiter('0'));
        assert!(!is_valid_delimiter('9'));
    }

    #[test]
    fn test_match_position() {
        let pos = MatchPosition::new(10, 5);
        assert_eq!(pos.lnum, 10);
        assert_eq!(pos.col, 5);
    }

    #[test]
    fn test_match_range() {
        let start = MatchPosition::new(10, 0);
        let end = MatchPosition::new(10, 5);
        let range = MatchRange::new(start, end);

        assert!(range.is_single_line());
        assert_eq!(range.line_span(), 1);

        // Multi-line range
        let end = MatchPosition::new(12, 5);
        let range = MatchRange::new(start, end);

        assert!(!range.is_single_line());
        assert_eq!(range.line_span(), 3);
    }

    #[test]
    fn test_substitute_error_display() {
        let err = SubstituteError::InvalidFlag('x');
        assert_eq!(format!("{err}"), "invalid flag: x");

        let err = SubstituteError::InvalidDelimiter('a');
        assert!(format!("{err}").contains("delimited by letters"));

        let err = SubstituteError::NoPreviousPattern;
        assert_eq!(format!("{err}"), "no previous pattern");

        let err = SubstituteError::ZeroCount;
        assert_eq!(format!("{err}"), "zero count");
    }

    #[test]
    fn test_rs_is_valid_delimiter() {
        assert_eq!(rs_is_valid_delimiter(b'/' as c_int), 1);
        assert_eq!(rs_is_valid_delimiter(b'#' as c_int), 1);
        assert_eq!(rs_is_valid_delimiter(b'a' as c_int), 0);
        assert_eq!(rs_is_valid_delimiter(b'0' as c_int), 0);
    }

    #[test]
    fn test_rs_parse_sub_flags() {
        use std::ffi::CString;

        let flags = CString::new("g").unwrap();
        let result = unsafe { rs_parse_sub_flags(flags.as_ptr()) };
        assert!(result >= 0);
        assert_eq!(result & 1, 1); // do_all

        let flags = CString::new("gc").unwrap();
        let result = unsafe { rs_parse_sub_flags(flags.as_ptr()) };
        assert!(result >= 0);
        assert_eq!(result & 1, 1); // do_all
        assert_eq!(result & 2, 2); // do_ask

        // Invalid flag
        let flags = CString::new("gx").unwrap();
        let result = unsafe { rs_parse_sub_flags(flags.as_ptr()) };
        assert_eq!(result, -1);
    }
}
