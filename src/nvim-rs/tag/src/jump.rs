//! Tag jump orchestration for Neovim C-to-Rust migration
//!
//! This module provides Rust implementations for tag jump preparation,
//! validation, and state management.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ptr_as_ptr)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

/// Return value indicating success
const OK: c_int = 1;
/// Return value indicating failure
const FAIL: c_int = 0;
/// Return value indicating file not found
const NOTAGFILE: c_int = 2;

/// Maximum line size for tag patterns
const LSIZE: usize = 512;

/// UIExtension value for kUIMessages (ui_defs.h)
const K_UI_MESSAGES: c_int = 4;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // Memory functions
    fn xfree(ptr: *mut c_void);
    fn xmalloc(size: usize) -> *mut c_void;

    // String functions
    fn strlen(s: *const c_char) -> usize;
    fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;

    // File functions (direct C)
    fn os_path_exists(path: *const c_char) -> bool;
    fn nvim_has_bufreadcmd(fname: *const c_char) -> bool;

    // Global state accessors
    fn nvim_get_postponed_split() -> c_int;
    fn nvim_set_postponed_split(val: c_int);
    fn nvim_get_g_do_tagpreview() -> c_int;
    fn nvim_set_g_do_tagpreview(val: c_int);

    // Window functions (direct C)
    fn check_can_set_curbuf_forceit(forceit: c_int) -> bool;

}

// =============================================================================
// Jump state structure
// =============================================================================

/// State for a tag jump operation.
///
/// This structure holds all the intermediate state needed during a tag jump,
/// allowing the operation to be split into prepare/execute/cleanup phases.
#[repr(C)]
pub struct JumpTagState {
    /// Copy of the match line buffer
    pub lbuf: *mut c_char,
    /// Length of lbuf (excluding null terminator)
    pub lbuf_len: usize,
    /// Search pattern buffer
    pub pbuf: *mut c_char,
    /// End of pattern in pbuf
    pub pbuf_end: *mut c_char,
    /// Expanded file name (may be allocated)
    pub expanded_fname: *mut c_char,
    /// Original file name pointer (into lbuf)
    pub orig_fname: *const c_char,
    /// Tag file name
    pub tag_fname: *const c_char,
    /// Force flag
    pub forceit: c_int,
    /// Keep help flag
    pub keep_help: bool,
    /// Whether file exists
    pub file_exists: bool,
    /// Current state (for phased execution)
    pub phase: c_int,
    /// Result code
    pub result: c_int,
}

impl Default for JumpTagState {
    fn default() -> Self {
        Self {
            lbuf: ptr::null_mut(),
            lbuf_len: 0,
            pbuf: ptr::null_mut(),
            pbuf_end: ptr::null_mut(),
            expanded_fname: ptr::null_mut(),
            orig_fname: ptr::null(),
            tag_fname: ptr::null(),
            forceit: 0,
            keep_help: false,
            file_exists: false,
            phase: 0,
            result: 0,
        }
    }
}

/// Jump phases
pub mod phase {
    use std::ffi::c_int;

    /// Initial state
    pub const INIT: c_int = 0;
    /// Prepared (buffers allocated, parsed)
    pub const PREPARED: c_int = 1;
    /// File validated
    pub const VALIDATED: c_int = 2;
    /// Ready to execute
    pub const READY: c_int = 3;
    /// Execution complete
    pub const DONE: c_int = 4;
    /// Error state
    pub const ERROR: c_int = -1;
}

// =============================================================================
// State lifecycle
// =============================================================================

/// Create a new jump state.
#[no_mangle]
pub extern "C" fn rs_jump_tag_state_new() -> *mut JumpTagState {
    Box::into_raw(Box::new(JumpTagState::default()))
}

/// Free a jump state and all its resources.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_state_free(state: *mut JumpTagState) {
    if state.is_null() {
        return;
    }

    let state = &mut *state;
    rs_jump_tag_cleanup(state);
    drop(Box::from_raw(state));
}

/// Clean up resources in a jump state (but don't free the state itself).
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_cleanup(state: *mut JumpTagState) {
    if state.is_null() {
        return;
    }

    let state = &mut *state;

    if !state.lbuf.is_null() {
        xfree(state.lbuf as *mut c_void);
        state.lbuf = ptr::null_mut();
    }

    if !state.pbuf.is_null() {
        xfree(state.pbuf as *mut c_void);
        state.pbuf = ptr::null_mut();
    }

    if !state.expanded_fname.is_null() {
        xfree(state.expanded_fname as *mut c_void);
        state.expanded_fname = ptr::null_mut();
    }

    state.pbuf_end = ptr::null_mut();
    state.orig_fname = ptr::null();
    state.tag_fname = ptr::null();
    state.lbuf_len = 0;
    state.phase = phase::INIT;
}

// =============================================================================
// Preparation phase
// =============================================================================

/// Prepare for a tag jump by parsing the match line.
///
/// This allocates buffers and parses the match line into components.
///
/// # Arguments
///
/// * `state` - Jump state to initialize
/// * `lbuf_arg` - Match line buffer
/// * `forceit` - Force flag for buffer switching
/// * `keep_help` - Whether to keep help flag
///
/// # Returns
///
/// OK on success, FAIL on error.
#[no_mangle]
pub unsafe extern "C" fn rs_jumpto_tag_prepare(
    state: *mut JumpTagState,
    lbuf_arg: *const c_char,
    forceit: c_int,
    keep_help: bool,
) -> c_int {
    if state.is_null() || lbuf_arg.is_null() {
        return FAIL;
    }

    let state = &mut *state;

    // Check if we can set curbuf
    let postponed_split = nvim_get_postponed_split();
    if postponed_split == 0 && !check_can_set_curbuf_forceit(forceit) {
        state.phase = phase::ERROR;
        state.result = FAIL;
        return FAIL;
    }

    // Calculate the length of the match line
    let len = matching_line_len(lbuf_arg) + 1;

    // Allocate and copy the match line
    state.lbuf = xmalloc(len) as *mut c_char;
    if state.lbuf.is_null() {
        state.phase = phase::ERROR;
        state.result = FAIL;
        return FAIL;
    }
    memmove(state.lbuf as *mut c_void, lbuf_arg as *const c_void, len);
    state.lbuf_len = len - 1;

    // Allocate pattern buffer
    state.pbuf = xmalloc(LSIZE) as *mut c_char;
    if state.pbuf.is_null() {
        xfree(state.lbuf as *mut c_void);
        state.lbuf = ptr::null_mut();
        state.phase = phase::ERROR;
        state.result = FAIL;
        return FAIL;
    }

    state.forceit = forceit;
    state.keep_help = keep_help;
    state.phase = phase::PREPARED;

    OK
}

/// Calculate the length of a matching tag line.
///
/// The format is: `<mtt><tag_fname><NUL><NUL><lbuf>` for regular tags.
unsafe fn matching_line_len(lbuf: *const c_char) -> usize {
    let mut p = lbuf.add(1); // Skip mtt byte

    // Skip tag_fname and first NUL
    p = p.add(strlen(p) + 1);

    // Return total length including final part
    let offset = p.offset_from(lbuf) as usize;
    offset + strlen(p)
}

// =============================================================================
// Validation phase
// =============================================================================

/// Validate that the tag file exists.
///
/// # Arguments
///
/// * `state` - Jump state with prepared buffers
///
/// # Returns
///
/// OK if file exists, NOTAGFILE if not found, FAIL on error.
#[no_mangle]
pub unsafe extern "C" fn rs_jumpto_tag_validate(state: *mut JumpTagState) -> c_int {
    if state.is_null() {
        return FAIL;
    }

    let state = &mut *state;

    if state.phase != phase::PREPARED {
        return FAIL;
    }

    // At this point, the C code would have already expanded the filename
    // and stored it in expanded_fname. Check if it exists.
    if state.expanded_fname.is_null() {
        state.phase = phase::ERROR;
        state.result = FAIL;
        return FAIL;
    }

    // Check if file exists or has a BufReadCmd autocmd
    let exists = os_path_exists(state.expanded_fname) || nvim_has_bufreadcmd(state.expanded_fname);

    if !exists {
        state.file_exists = false;
        state.result = NOTAGFILE;
        crate::tag_set_nofile_fname(state.expanded_fname);
        state.phase = phase::ERROR;
        return NOTAGFILE;
    }

    state.file_exists = true;
    state.phase = phase::VALIDATED;
    OK
}

// =============================================================================
// State accessors
// =============================================================================

/// Get the current phase of the jump state.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_get_phase(state: *const JumpTagState) -> c_int {
    if state.is_null() {
        return phase::ERROR;
    }
    (*state).phase
}

/// Set the current phase of the jump state.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_set_phase(state: *mut JumpTagState, new_phase: c_int) {
    if state.is_null() {
        return;
    }
    (*state).phase = new_phase;
}

/// Get the result code from the jump state.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_get_result(state: *const JumpTagState) -> c_int {
    if state.is_null() {
        return FAIL;
    }
    (*state).result
}

/// Set the result code in the jump state.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_set_result(state: *mut JumpTagState, result: c_int) {
    if state.is_null() {
        return;
    }
    (*state).result = result;
}

/// Get the match line buffer.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_get_lbuf(state: *const JumpTagState) -> *mut c_char {
    if state.is_null() {
        return ptr::null_mut();
    }
    (*state).lbuf
}

/// Get the pattern buffer.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_get_pbuf(state: *const JumpTagState) -> *mut c_char {
    if state.is_null() {
        return ptr::null_mut();
    }
    (*state).pbuf
}

/// Get the expanded file name.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_get_fname(state: *const JumpTagState) -> *mut c_char {
    if state.is_null() {
        return ptr::null_mut();
    }
    (*state).expanded_fname
}

/// Set the expanded file name.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_set_fname(state: *mut JumpTagState, fname: *mut c_char) {
    if state.is_null() {
        return;
    }
    (*state).expanded_fname = fname;
}

/// Get the force flag.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_get_forceit(state: *const JumpTagState) -> c_int {
    if state.is_null() {
        return 0;
    }
    (*state).forceit
}

/// Get the keep_help flag.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_get_keep_help(state: *const JumpTagState) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).keep_help
}

/// Check if the file was validated as existing.
#[no_mangle]
pub unsafe extern "C" fn rs_jump_tag_file_exists(state: *const JumpTagState) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).file_exists
}

// =============================================================================
// Helper functions for tag file opening
// =============================================================================

/// Check if a tag file can be opened.
///
/// This validates that the file exists and can potentially be read.
///
/// # Arguments
///
/// * `fname` - File name to check
///
/// # Returns
///
/// true if the file can be opened, false otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_can_open_tag_file(fname: *const c_char) -> bool {
    if fname.is_null() {
        return false;
    }

    os_path_exists(fname) || nvim_has_bufreadcmd(fname)
}

// =============================================================================
// Global state management
// =============================================================================

/// Get the postponed_split global.
#[no_mangle]
pub extern "C" fn rs_get_postponed_split() -> c_int {
    unsafe { nvim_get_postponed_split() }
}

/// Set the postponed_split global.
#[no_mangle]
pub unsafe extern "C" fn rs_set_postponed_split(val: c_int) {
    nvim_set_postponed_split(val);
}

/// Get the g_do_tagpreview global.
#[no_mangle]
pub extern "C" fn rs_get_g_do_tagpreview() -> c_int {
    unsafe { nvim_get_g_do_tagpreview() }
}

/// Set the g_do_tagpreview global.
#[no_mangle]
pub unsafe extern "C" fn rs_set_g_do_tagpreview(val: c_int) {
    nvim_set_g_do_tagpreview(val);
}

/// Reset g_do_tagpreview to 0 (cleanup after tag operation).
#[no_mangle]
pub unsafe extern "C" fn rs_reset_tagpreview() {
    nvim_set_g_do_tagpreview(0);
}

// =============================================================================
// Phase 10: rs_jumpto_tag — full tag jump orchestration
// =============================================================================

// Import Rust functions from sibling modules via extern "C"
extern "C" {
    fn rs_parse_match(lbuf: *mut c_char, tagp: *mut c_void) -> c_int;
    fn rs_test_for_static(tagp: *const c_void) -> bool;
    fn rs_find_extra(pp: *mut *mut c_char) -> c_int;
    fn rs_expand_tag_fname(fname: *mut c_char, tag_fname: *mut c_char, expand: bool)
        -> *mut c_char;
}

// =============================================================================
// Phase 3: rs_tag_jumpto_execute — migrated from nvim_tag_jumpto_execute in C
// =============================================================================

extern "C" {
    fn nvim_tag_get_KeyTyped() -> bool;
    fn nvim_tag_get_curwin() -> *mut c_void;
    // Phase 2: fine-grained jumpto helpers (replacing composite C functions)
    fn nvim_tag_inc_RedrawingDisabled();
    fn nvim_tag_curwin_pvw() -> bool;
    fn nvim_tag_fullname_save(fname: *mut c_char) -> *mut c_char;
    fn prepare_tagpreview(undo_sync: bool) -> bool;
    fn nvim_tag_swb_has_useopen_or_usetab() -> bool;
    fn nvim_tag_buflist_findname_exp(fname: *mut c_char) -> *mut c_void;
    fn nvim_tag_swbuf_goto_win_with_buf(buf: *mut c_void) -> bool;
    fn win_split(size: c_int, flags: c_int) -> c_int;
    fn nvim_tag_get_postponed_split_flags() -> c_int;
    fn nvim_tag_reset_binding_curwin();
    fn nvim_tag_set_keep_help_flag(val: bool);
    fn nvim_tag_bt_help_saved_win(win: *const c_void) -> bool;
    fn getfile(
        fnum: c_int,
        ffname: *mut c_char,
        sfname: *mut c_char,
        setpm: bool,
        lnum: i32,
        forceit: bool,
    ) -> c_int;
    fn nvim_tag_get_cmdmod_tab() -> c_int;
    // Accessors for rs_tag_jumpto_run_search (Phase 1)
    fn nvim_tag_set_curswant(val: bool);
    fn nvim_tag_get_magic_overruled() -> c_int;
    fn nvim_tag_set_magic_overruled(val: c_int);
    fn nvim_tag_get_no_hlsearch() -> bool;
    #[link_name = "set_no_hlsearch"]
    fn nvim_tag_set_no_hlsearch_val(val: bool);
    fn nvim_tag_cpo_has_tagpat() -> bool;
    fn nvim_tag_get_p_ws() -> bool;
    fn nvim_tag_set_p_ws(val: bool);
    fn nvim_tag_get_p_ic() -> c_int;
    fn nvim_tag_get_p_scs() -> c_int;
    fn nvim_tag_set_p_scs(val: c_int);
    fn nvim_set_p_ic(val: c_int);
    fn nvim_tag_get_cursor_lnum() -> i32;
    fn nvim_tag_set_cursor_lnum(val: i32);
    fn nvim_tag_set_cursor_start();
    fn nvim_tag_get_secure() -> c_int;
    fn nvim_tag_set_secure(val: c_int);
    fn nvim_tag_inc_sandbox();
    fn nvim_tag_dec_sandbox();
    fn nvim_tag_skip_regexp(p: *mut c_char, delim: c_int) -> *mut c_char;
    fn do_search(
        oap: *mut c_void,
        dirc: c_int,
        search_delim: c_int,
        pat: *mut c_char,
        patlen: usize,
        count: c_int,
        options: c_int,
        sia: *mut c_void,
    ) -> c_int;
    fn do_cmdline_cmd(cmd: *const c_char) -> c_int;
    fn wait_return(redraw: bool);
    fn nvim_tag_check_cursor();
    fn emsg(s: *const c_char) -> c_int;
    fn msg(s: *const c_char, hl_id: c_int) -> c_int;
    fn nvim_tag_get_msg_scrolled() -> c_int;
    fn nvim_tag_get_msg_silent() -> c_int;
    fn ui_has(ext: c_int) -> bool;
    fn ui_flush();
    fn os_delay(ms: u64, ignoreinput: bool);
    // Post-jump helpers (Phase 3 — migrated to Rust inline)
    fn nvim_curbuf_is_help() -> c_int;
    fn nvim_tag_set_topline_curwin();
    fn nvim_get_fdo_flags() -> u32;
    fn rs_foldOpenCursor();
    fn rs_win_valid(win: *const c_void) -> bool;
    fn nvim_validate_cursor();
    fn nvim_redraw_later_wrapper(wp: *mut c_void, typ: c_int);
    fn nvim_win_enter(wp: *mut c_void, undo_sync: c_int);
    fn nvim_tag_dec_RedrawingDisabled();
    fn nvim_tag_win_close_curwin();
}

use crate::parse::TagPtrs;

// =============================================================================
// Phase 1: rs_tag_jumpto_run_search — migrated from nvim_tag_jumpto_run_search in C
// =============================================================================

/// Rust port of the search/cmdline execution phase after a successful getfile.
///
/// Mirrors `nvim_tag_jumpto_run_search` in C exactly:
/// - Sets `curwin->w_set_curswant`
/// - Saves/restores `magic_overruled`, `no_hlsearch`, `p_ws`, `p_ic`, `p_scs`
/// - Tries regexp search with up to 4 fallback patterns
/// - Or executes ex-command in sandbox
///
/// # Safety
/// All pointer arguments must be valid.
#[allow(clippy::too_many_lines)]
unsafe fn rs_tag_jumpto_run_search(
    pbuf: *mut c_char,
    pbuf_end: *mut c_char,
    lbuf: *mut c_char,
) -> c_int {
    /// SEARCH_KEEP = 0x400 (verified by _Static_assert in tag_shim.c)
    const SEARCH_KEEP: c_int = 0x400;
    /// OPTION_MAGIC_OFF = 2 (verified by _Static_assert in tag_shim.c)
    const OPTION_MAGIC_OFF: c_int = 2;
    /// LSIZE = 512 (verified by _Static_assert in tag_shim.c)
    const LSIZE: usize = 512;

    let mut retval = FAIL;

    nvim_tag_set_curswant(true);
    nvim_set_postponed_split(0);

    let save_magic_overruled = nvim_tag_get_magic_overruled();
    nvim_tag_set_magic_overruled(OPTION_MAGIC_OFF);
    let save_no_hlsearch = nvim_tag_get_no_hlsearch();

    let search_options: c_int = if nvim_tag_cpo_has_tagpat() {
        0
    } else {
        SEARCH_KEEP
    };

    // Check if the command is a search pattern (starts with '/' or '?')
    let mut str_p = pbuf;
    let first_byte = *pbuf as u8;
    if first_byte == b'/' || first_byte == b'?' {
        str_p = nvim_tag_skip_regexp(pbuf.add(1), c_int::from(first_byte as i8)).add(1);
    }

    if str_p > pbuf_end.sub(1) {
        // Re-parse lbuf to get tagp (tagline, tagname, tagname_end)
        let mut tagp = TagPtrs::default();
        if rs_parse_match(lbuf, (&raw mut tagp).cast::<c_void>()) == FAIL {
            nvim_tag_set_magic_overruled(save_magic_overruled);
            if search_options != 0 {
                nvim_tag_set_no_hlsearch_val(save_no_hlsearch);
            }
            return FAIL;
        }

        let pbuflen = pbuf_end.offset_from(pbuf) as usize;

        let save_p_ws = nvim_tag_get_p_ws();
        let save_p_ic = nvim_tag_get_p_ic();
        let save_p_scs = nvim_tag_get_p_scs();
        nvim_tag_set_p_ws(true);
        nvim_set_p_ic(0); // false
        nvim_tag_set_p_scs(0); // false
        let save_lnum = nvim_tag_get_cursor_lnum();

        // Set cursor to tagline - 1 (or 0 if tagline <= 0)
        let new_lnum = if tagp.tagline > 0 {
            tagp.tagline - 1
        } else {
            0
        };
        nvim_tag_set_cursor_lnum(new_lnum);

        if do_search(
            ptr::null_mut(),
            c_int::from(first_byte as i8),
            c_int::from(first_byte as i8),
            pbuf.add(1),
            pbuflen - 1,
            1,
            search_options,
            ptr::null_mut(),
        ) != 0
        {
            retval = OK;
        } else {
            let mut found: c_int = 1;
            nvim_set_p_ic(1); // true
            if do_search(
                ptr::null_mut(),
                c_int::from(first_byte as i8),
                c_int::from(first_byte as i8),
                pbuf.add(1),
                pbuflen - 1,
                1,
                search_options,
                ptr::null_mut(),
            ) == 0
            {
                found = 2;
                rs_test_for_static((&raw const tagp).cast::<c_void>());
                let cc = *tagp.tagname_end;
                *tagp.tagname_end = 0;

                // Build fallback pattern: "^tagname\s\*("
                let new_len = libc::snprintf(pbuf, LSIZE, c"^%s\\s\\*(".as_ptr(), tagp.tagname);
                let new_pbuflen = new_len as usize;

                if do_search(
                    ptr::null_mut(),
                    b'/' as c_int,
                    b'/' as c_int,
                    pbuf,
                    new_pbuflen,
                    1,
                    search_options,
                    ptr::null_mut(),
                ) == 0
                {
                    // Build wider fallback: "^[#a-zA-Z_].*\<tagname\s\*("
                    let new_len2 = libc::snprintf(
                        pbuf,
                        LSIZE,
                        c"^\\[#a-zA-Z_]\\.\\*\\<%s\\s\\*(".as_ptr(),
                        tagp.tagname,
                    );
                    let new_pbuflen2 = new_len2 as usize;
                    if do_search(
                        ptr::null_mut(),
                        b'/' as c_int,
                        b'/' as c_int,
                        pbuf,
                        new_pbuflen2,
                        1,
                        search_options,
                        ptr::null_mut(),
                    ) == 0
                    {
                        found = 0;
                    }
                }
                *tagp.tagname_end = cc;
            }

            if found == 0 {
                emsg(c"E434: Can't find tag pattern".as_ptr());
                nvim_tag_set_cursor_lnum(save_lnum);
            } else {
                if found == 2 || save_p_ic == 0 {
                    msg(c"E435: Couldn't find tag, just guessing!".as_ptr(), 0);
                    // Only delay if not scrolled and not silent and no UI messages
                    if nvim_tag_get_msg_scrolled() == 0
                        && nvim_tag_get_msg_silent() == 0
                        && !ui_has(K_UI_MESSAGES)
                    {
                        ui_flush();
                        os_delay(1010, true);
                    }
                }
                retval = OK;
            }
        }

        nvim_tag_set_p_ws(save_p_ws);
        nvim_set_p_ic(save_p_ic);
        nvim_tag_set_p_scs(save_p_scs);
        nvim_tag_check_cursor();
    } else {
        // Ex-command branch
        let save_secure = nvim_tag_get_secure();
        nvim_tag_set_secure(1);
        nvim_tag_inc_sandbox();
        nvim_tag_set_cursor_start();
        do_cmdline_cmd(pbuf);
        retval = OK;

        if nvim_tag_get_secure() == 2 {
            wait_return(true);
        }
        nvim_tag_set_secure(save_secure);
        nvim_tag_dec_sandbox();
    }

    nvim_tag_set_magic_overruled(save_magic_overruled);
    if search_options != 0 {
        nvim_tag_set_no_hlsearch_val(save_no_hlsearch);
    }

    retval
}

/// Rust implementation of the jumpto_execute phase — migrated from C.
///
/// Handles:
/// - RedrawingDisabled management
/// - Preview window setup
/// - Window splitting (swb + CTRL-W CTRL-])
/// - File loading (getfile)
/// - Search/command execution
/// - Cursor positioning and fold opening
/// - Cleanup on error
///
/// Phase 2: jumpto helpers inlined here; composite C functions deleted.
///
/// # Safety
/// All pointer arguments must be valid C strings.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_tag_jumpto_execute(
    mut fname: *mut c_char,
    pbuf: *mut c_char,
    pbuf_end: *mut c_char,
    lbuf: *mut c_char,
    forceit: c_int,
    keep_help: bool,
) -> c_int {
    /// GETFILE_SAME_FILE = 0 (verified by _Static_assert in tag_shim.c)
    const GETFILE_SAME_FILE: c_int = 0;
    /// GETFILE_OPEN_OTHER = -1 (verified by _Static_assert in tag_shim.c)
    const GETFILE_OPEN_OTHER: c_int = -1;
    /// GETFILE_UNUSED = 8 (verified by _Static_assert in tag_shim.c)
    const GETFILE_UNUSED: c_int = 8;

    let old_key_typed = nvim_tag_get_KeyTyped();
    let l_g_do_tagpreview = nvim_get_g_do_tagpreview();

    // Increment RedrawingDisabled
    nvim_tag_inc_RedrawingDisabled();

    // Save curwin for preview window tracking
    let curwin_save = nvim_tag_get_curwin();

    // --- Inline nvim_tag_jumpto_preview_setup ---
    // If g_do_tagpreview is set: clear postponed_split.
    // If curwin is the preview window, no expansion needed; otherwise expand fname.
    let full_fname: *mut c_char = if l_g_do_tagpreview != 0 {
        nvim_set_postponed_split(0);
        if nvim_tag_curwin_pvw() {
            ptr::null_mut()
        } else {
            let full = nvim_tag_fullname_save(fname);
            prepare_tagpreview(true);
            full
        }
    } else {
        ptr::null_mut()
    };
    if !full_fname.is_null() {
        fname = full_fname;
    }

    // --- Inline nvim_tag_jumpto_check_swb ---
    // If postponed_split && swb has useopen/usetab, try to switch to existing buf.
    let mut getfile_result: c_int =
        if nvim_get_postponed_split() != 0 && nvim_tag_swb_has_useopen_or_usetab() {
            let existing_buf = nvim_tag_buflist_findname_exp(fname);
            if !existing_buf.is_null() && nvim_tag_swbuf_goto_win_with_buf(existing_buf) {
                GETFILE_SAME_FILE
            } else {
                GETFILE_UNUSED
            }
        } else {
            GETFILE_UNUSED
        };

    // --- Inline nvim_tag_jumpto_maybe_split ---
    // Split window if postponed_split or cmdmod.cmod_tab is set and not already switched.
    if getfile_result == GETFILE_UNUSED
        && (nvim_get_postponed_split() != 0 || nvim_tag_get_cmdmod_tab() != 0)
    {
        let split_size = if nvim_get_postponed_split() > 0 {
            nvim_get_postponed_split()
        } else {
            0
        };
        if win_split(split_size, nvim_tag_get_postponed_split_flags()) == FAIL {
            // Decrement RedrawingDisabled (was incremented above)
            nvim_tag_dec_RedrawingDisabled();
            xfree(full_fname.cast::<c_void>());
            return FAIL;
        }
        nvim_tag_reset_binding_curwin();
    }

    // --- Inline nvim_tag_jumpto_set_keep_help ---
    if keep_help {
        if l_g_do_tagpreview != 0 {
            nvim_tag_set_keep_help_flag(nvim_tag_bt_help_saved_win(curwin_save));
        } else {
            nvim_tag_set_keep_help_flag(nvim_curbuf_is_help() != 0);
        }
    }

    // --- Inline nvim_tag_jumpto_load_file ---
    // Load file if not already done by swb switch.
    if getfile_result == GETFILE_UNUSED {
        getfile_result = getfile(0, fname, ptr::null_mut(), true, 0, forceit != 0);
        nvim_tag_set_keep_help_flag(false);
    }

    // GETFILE_SUCCESS(x) is (x) <= 0
    let retval = if getfile_result <= 0 {
        // Run search or ex-command (sets result to OK or FAIL).
        // GETFILE_OPEN_OTHER always yields OK regardless of search outcome.
        let raw_search = rs_tag_jumpto_run_search(pbuf, pbuf_end, lbuf);
        let search_retval = if getfile_result == GETFILE_OPEN_OTHER {
            OK
        } else {
            raw_search
        };

        if search_retval == OK {
            // Inline Rust port of nvim_tag_jumpto_post_success (Phase 3)
            if nvim_curbuf_is_help() != 0 {
                nvim_tag_set_topline_curwin();
            }
            // kOptFdoFlagTag = 0x80 (verified by _Static_assert in tag_shim.c)
            if (nvim_get_fdo_flags() & 0x80) != 0 && old_key_typed {
                rs_foldOpenCursor();
            }
            if l_g_do_tagpreview != 0 {
                let curwin_now = nvim_tag_get_curwin();
                if curwin_now != curwin_save && rs_win_valid(curwin_save) {
                    // UPD_VALID = 10 (from drawscreen.h)
                    nvim_validate_cursor();
                    nvim_redraw_later_wrapper(curwin_now, 10);
                    nvim_win_enter(curwin_save, 1);
                }
            }
            nvim_tag_dec_RedrawingDisabled();
        } else {
            // Inline Rust port of nvim_tag_jumpto_post_fail (Phase 3)
            nvim_tag_dec_RedrawingDisabled();
            if nvim_get_postponed_split() != 0 {
                nvim_tag_win_close_curwin();
                nvim_set_postponed_split(0);
            }
        }

        search_retval
    } else {
        // Inline Rust port of nvim_tag_jumpto_post_fail (Phase 3)
        nvim_tag_dec_RedrawingDisabled();
        if nvim_get_postponed_split() != 0 {
            nvim_tag_win_close_curwin();
            nvim_set_postponed_split(0);
        }
        FAIL
    };

    xfree(full_fname.cast::<c_void>());
    retval
}

/// Main tag jump function — replaces `jumpto_tag()` in C.
///
/// Parses the match line, expands the filename, checks file existence,
/// then delegates the complex execution phase (window management, search,
/// command execution) to `rs_tag_jumpto_execute`.
///
/// # Safety
/// - `lbuf_arg` must be a valid match line string
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_jumpto_tag(
    lbuf_arg: *const c_char,
    forceit: c_int,
    keep_help: bool,
) -> c_int {
    // Check prerequisites
    let postponed_split = nvim_get_postponed_split();
    if postponed_split == 0 && !check_can_set_curbuf_forceit(forceit) {
        return FAIL;
    }

    // Calculate match line length and copy it
    let len = matching_line_len(lbuf_arg) + 1;
    let lbuf = xmalloc(len).cast::<c_char>();
    memmove(lbuf.cast(), lbuf_arg.cast(), len);

    // Allocate pattern buffer
    let pbuf = xmalloc(LSIZE).cast::<c_char>();

    // Parse the match line
    let mut tagp = TagPtrs::default();
    if rs_parse_match(lbuf, (&raw mut tagp).cast()) == FAIL {
        xfree(lbuf.cast());
        xfree(pbuf.cast());
        nvim_set_g_do_tagpreview(0);
        return FAIL;
    }

    // Temporarily truncate filename so it can be used as a string.
    // Save and restore the original character so lbuf remains intact for
    // rs_tag_jumpto_execute which re-parses it.
    let saved_fname_end = *tagp.fname_end;
    *tagp.fname_end = 0;
    let fname = tagp.fname;

    // Copy the command to pbuf[], remove trailing CR/NL
    let mut str_p = tagp.command;
    let mut pbuf_end = pbuf;
    while *str_p != 0 && *str_p != b'\n' as c_char && *str_p != b'\r' as c_char {
        *pbuf_end = *str_p;
        pbuf_end = pbuf_end.add(1);
        str_p = str_p.add(1);
        if pbuf_end.offset_from(pbuf) as usize + 1 >= LSIZE {
            break;
        }
    }
    *pbuf_end = 0;

    // Remove the "<Tab>fieldname:value" stuff
    let mut extra_str = pbuf;
    if rs_find_extra(&raw mut extra_str) == OK {
        pbuf_end = extra_str;
        *pbuf_end = 0;
    }

    // Expand filename
    let expanded_fname = rs_expand_tag_fname(fname, tagp.tag_fname, true);

    // Restore the original character so lbuf can be re-parsed
    *tagp.fname_end = saved_fname_end;

    // Check if file exists
    if !os_path_exists(expanded_fname) && !nvim_has_bufreadcmd(expanded_fname) {
        crate::tag_set_nofile_fname(expanded_fname);
        xfree(lbuf.cast());
        xfree(pbuf.cast());
        xfree(expanded_fname.cast());
        nvim_set_g_do_tagpreview(0);
        return NOTAGFILE;
    }

    // Execute the jump: window management, file loading, search/command execution
    let retval = rs_tag_jumpto_execute(expanded_fname, pbuf, pbuf_end, lbuf, forceit, keep_help);

    // Cleanup
    nvim_set_g_do_tagpreview(0);
    xfree(lbuf.cast());
    xfree(pbuf.cast());
    xfree(expanded_fname.cast());

    retval
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jump_state_default() {
        let state = JumpTagState::default();
        assert!(state.lbuf.is_null());
        assert!(state.pbuf.is_null());
        assert!(state.expanded_fname.is_null());
        assert_eq!(state.phase, phase::INIT);
        assert_eq!(state.result, 0);
    }

    #[test]
    fn test_jump_state_new() {
        let state = rs_jump_tag_state_new();
        assert!(!state.is_null());
        unsafe {
            assert!((*state).lbuf.is_null());
            assert_eq!((*state).phase, phase::INIT);
            // Note: Can't call rs_jump_tag_state_free without C runtime
        }
    }

    #[test]
    fn test_null_safety() {
        unsafe {
            // Cleanup with null
            rs_jump_tag_cleanup(ptr::null_mut());

            // Prepare with null state
            assert_eq!(
                rs_jumpto_tag_prepare(ptr::null_mut(), c"test".as_ptr(), 0, false),
                FAIL
            );

            // Prepare with null lbuf
            let state = rs_jump_tag_state_new();
            assert_eq!(rs_jumpto_tag_prepare(state, ptr::null(), 0, false), FAIL);

            // Validate with null
            assert_eq!(rs_jumpto_tag_validate(ptr::null_mut()), FAIL);

            // Accessors with null
            assert_eq!(rs_jump_tag_get_phase(ptr::null()), phase::ERROR);
            rs_jump_tag_set_phase(ptr::null_mut(), phase::DONE);
            assert_eq!(rs_jump_tag_get_result(ptr::null()), FAIL);
            rs_jump_tag_set_result(ptr::null_mut(), OK);
            assert!(rs_jump_tag_get_lbuf(ptr::null()).is_null());
            assert!(rs_jump_tag_get_pbuf(ptr::null()).is_null());
            assert!(rs_jump_tag_get_fname(ptr::null()).is_null());
            rs_jump_tag_set_fname(ptr::null_mut(), ptr::null_mut());
            assert_eq!(rs_jump_tag_get_forceit(ptr::null()), 0);
            assert!(!rs_jump_tag_get_keep_help(ptr::null()));
            assert!(!rs_jump_tag_file_exists(ptr::null()));
        }
    }

    #[test]
    fn test_can_open_tag_file_null() {
        unsafe {
            assert!(!rs_can_open_tag_file(ptr::null()));
        }
    }

    #[test]
    fn test_phase_constants() {
        assert_eq!(phase::INIT, 0);
        assert_eq!(phase::PREPARED, 1);
        assert_eq!(phase::VALIDATED, 2);
        assert_eq!(phase::READY, 3);
        assert_eq!(phase::DONE, 4);
        assert_eq!(phase::ERROR, -1);
    }

    #[test]
    fn test_result_constants() {
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
        assert_eq!(NOTAGFILE, 2);
    }
}
