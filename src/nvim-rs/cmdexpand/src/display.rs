//! Display functions for command-line completion matches.
//!
//! Provides `showmatches` (and inner helper `showmatches_oneline`).

use libc::{c_char, c_int};

use crate::ExpandHandle;

// =============================================================================
// Constants
// =============================================================================

/// `HLF_D` — directories in CTRL-D listing (`highlight_defs.h` enum value 5).
const HLF_D: c_int = 5;

/// `HLF_T` — titles for ":set all" etc. (`highlight_defs.h` enum value 23).
const HLF_T: c_int = 23;

/// `EXPAND_OK` return value from `showmatches` (success).
const EXPAND_OK: c_int = -1;

/// `EXPAND_TAGS_LISTFILES` context (tags listing files mode).
const EXPAND_TAGS_LISTFILES: c_int = 11;

/// `EXPAND_FILES` context.
const EXPAND_FILES: c_int = 13;

/// `EXPAND_SHELLCMD` context.
const EXPAND_SHELLCMD: c_int = 33;

/// `EXPAND_BUFFERS` context.
const EXPAND_BUFFERS: c_int = 5;

/// `EXPAND_LUA` context.
const EXPAND_LUA: c_int = 74;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    /// Get `cmdbuff` from `get_cmdline_info()`.
    fn nvim_cmdexpand_get_cmdbuff() -> *mut c_char;

    /// Get `cmdpos` from `get_cmdline_info()`.
    fn nvim_cmdexpand_get_cmdpos() -> c_int;

    /// `got_int` global.
    static mut got_int: bool;

    /// `cmdline_row` global.
    static mut cmdline_row: c_int;

    /// `msg_row` global.
    static msg_row: c_int;

    /// `msg_col` global.
    static msg_col: c_int;

    /// `msg_didany` global.
    static mut msg_didany: bool;

    /// Direct: `msg_start()`.
    fn msg_start();

    /// Wrapper for `msg_putchar(c)`.
    fn nvim_cmdexpand_msg_putchar(c: c_int);

    /// Direct: `ui_flush()`.
    fn ui_flush();

    /// Set kind for `msg_ext`.
    fn nvim_cmdexpand_msg_ext_set_kind(kind: *const c_char);

    /// `Columns` global.
    static Columns: c_int;

    /// Wrapper for `FreeWild(count, files)`.
    fn nvim_cmdexpand_free_wild(count: c_int, files: *mut *mut c_char);

    /// Clear the cmdline completion PUM state.
    fn nvim_cmdexpand_pum_clear();

    /// Set `compl_selected`.
    fn nvim_set_compl_selected(val: c_int);

    /// Create cmdline PUM from matches and display it.
    fn nvim_cmdexpand_pum_create_from_matches(
        xp: *mut crate::ExpandT,
        matches: *mut *mut c_char,
        num_matches: c_int,
        showtail: c_int,
        noselect: c_int,
    );

    /// Display the cmdline PUM (already-created).
    fn nvim_cmdexpand_pum_display(changed_array: c_int);

    /// Draw the wildmenu status-bar display.
    fn nvim_cmdexpand_redraw_wildmenu_ex(
        xp: *mut crate::ExpandT,
        num_matches: c_int,
        matches: *mut *mut c_char,
        findex: c_int,
        showtail: c_int,
    );

    // msg_col declared above as static

    /// Wrapper for `msg_clr_eos()`.
    fn nvim_cmdexpand_msg_clr_eos();

    /// Wrapper for `msg_outtrans(str, attr, maxcol)`. Returns column after output.
    fn nvim_cmdexpand_msg_outtrans(str_: *const c_char, attr: c_int, maxcol: c_int) -> c_int;

    /// Wrapper for `msg_outtrans_long(str, attr)`.
    fn nvim_cmdexpand_msg_outtrans_long(str_: *const c_char, attr: c_int);

    /// Wrapper for `msg_advance(col)`.
    fn nvim_cmdexpand_msg_advance(col: c_int);

    /// Direct: `msg_puts(s)`.
    fn msg_puts(s: *const c_char);

    /// Wrapper for `msg_puts_hl(str, attr, maxcol)`.
    fn nvim_cmdexpand_msg_puts_hl(str_: *const c_char, attr: c_int, maxcol: c_int);

    /// Replace `$HOME` with `~` in `s`, write into `NameBuff`, return pointer.
    fn nvim_cmdexpand_home_replace_match(s: *const c_char) -> *const c_char;

    /// Wrapper for `expand_env_save_opt(str, true)`.
    fn nvim_cmdexpand_expand_env_save_opt(str_: *const c_char) -> *mut c_char;

    /// Wrapper for `backslash_halve_save(str)`.
    fn nvim_cmdexpand_backslash_halve_save(str_: *const c_char) -> *mut c_char;

    /// Wrapper for `os_isdir(str)`.
    fn nvim_cmdexpand_os_isdir(str_: *const c_char) -> c_int;

    /// Wrapper for `vim_strsize(str)`.
    fn nvim_cmdexpand_vim_strsize(str_: *const c_char) -> c_int;

    /// Return either `gettail(matches[m])` (if `showtail`) or `matches[m]`.
    fn nvim_cmdexpand_show_match(
        matches: *mut *mut c_char,
        m: c_int,
        showtail: c_int,
    ) -> *mut c_char;

    /// Check if cmdline PUM should be used.
    fn nvim_cmdexpand_compl_use_pum(need_wildmenu: c_int) -> c_int;

    /// Wrapper for `nlua_expand_pat(xp)`.
    fn nvim_cmdexpand_nlua_expand_pat(xp: *mut crate::ExpandT);

    /// Check if file matches should show tail of path.
    fn rs_expand_showtail(xp: ExpandHandle) -> c_int;

    /// Get `cmd_showtail` global.
    fn nvim_get_cmd_showtail() -> c_int;

    fn xfree(ptr: *mut libc::c_void);
}

// =============================================================================
// showmatches_oneline (internal helper)
// =============================================================================

/// Display one line of completion matches (multiple matches per line).
///
/// Used by wildmode=list and CTRL-D.
///
/// # Safety
///
/// `xp` and `matches` must be valid for the lifetime of this call.
unsafe fn showmatches_oneline(
    xp: *mut crate::ExpandT,
    matches: *mut *mut c_char,
    num_matches: c_int,
    lines: c_int,
    linenr: c_int,
    maxlen: c_int,
    showtail: bool,
) {
    let mut lastlen: c_int = 999;
    let mut j = linenr;
    while j < num_matches {
        if (*xp).xp_context == EXPAND_TAGS_LISTFILES {
            nvim_cmdexpand_msg_outtrans(*matches.add(j as usize), HLF_D, 0);
            // p points to the string after the NUL of the tagname
            let tag = *matches.add(j as usize);
            let tag_len = libc::strlen(tag);
            let p = tag.add(tag_len + 1);
            nvim_cmdexpand_msg_advance(maxlen + 1);
            msg_puts(p.cast::<c_char>());
            nvim_cmdexpand_msg_advance(maxlen + 3);
            nvim_cmdexpand_msg_outtrans_long(p.add(2).cast::<c_char>(), HLF_D);
            break;
        }

        // Advance to proper column
        let mut i = maxlen - lastlen;
        i -= 1;
        while i >= 0 {
            nvim_cmdexpand_msg_putchar(c_int::from(b' '));
            i -= 1;
        }

        let xp_context = (*xp).xp_context;
        let is_file_like = xp_context == EXPAND_FILES
            || xp_context == EXPAND_SHELLCMD
            || xp_context == EXPAND_BUFFERS;

        let isdir;
        let p: *const c_char;

        if is_file_like {
            // Determine if the match is a directory
            let match_str = *matches.add(j as usize);
            if (*xp).xp_numfiles == -1 {
                // Expansion was done here: file names are literal.
                isdir = nvim_cmdexpand_os_isdir(match_str) != 0;
            } else {
                // Expansion was done before: special chars were escaped, $HOME replaced.
                // Need to halve backslashes and unescape.
                let exp_path = nvim_cmdexpand_expand_env_save_opt(match_str);
                let path: *const c_char = if exp_path.is_null() {
                    match_str.cast::<c_char>()
                } else {
                    exp_path.cast::<c_char>()
                };
                let halved = nvim_cmdexpand_backslash_halve_save(path);
                isdir = nvim_cmdexpand_os_isdir(halved) != 0;
                xfree(exp_path.cast::<libc::c_void>());
                if halved != path.cast_mut() {
                    xfree(halved.cast::<libc::c_void>());
                }
            }

            if showtail {
                p = nvim_cmdexpand_show_match(matches, j, 1).cast::<c_char>();
            } else {
                p = nvim_cmdexpand_home_replace_match(match_str);
            }
        } else {
            isdir = false;
            p = nvim_cmdexpand_show_match(matches, j, c_int::from(showtail)).cast::<c_char>();
        }

        lastlen = nvim_cmdexpand_msg_outtrans(p, if isdir { HLF_D } else { 0 }, 0);
        j += lines;
    }

    if msg_col > 0 {
        // When not wrapped around
        nvim_cmdexpand_msg_clr_eos();
        nvim_cmdexpand_msg_putchar(c_int::from(b'\n'));
    }
}

// =============================================================================
// showmatches helpers
// =============================================================================

/// Resolved expansion state returned by `resolve_expand_state`.
struct ExpandState {
    num_matches: c_int,
    matches: *mut *mut c_char,
    showtail: bool,
    /// Whether the matches were freshly expanded (caller must free them).
    freshly_expanded: bool,
}

/// Resolve the match list for `showmatches`, running expansion if needed.
///
/// Returns `Err(retval)` when expansion fails.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` pointer.
unsafe fn resolve_expand_state(xp: *mut crate::ExpandT) -> Result<ExpandState, c_int> {
    if (*xp).xp_numfiles == -1 {
        crate::rs_set_expand_context(xp);
        if (*xp).xp_context == EXPAND_LUA {
            nvim_cmdexpand_nlua_expand_pat(xp);
        }
        let cmdbuff = nvim_cmdexpand_get_cmdbuff();
        let cmdpos = nvim_cmdexpand_get_cmdpos();
        let mut out_num_matches: c_int = 0;
        let mut out_matches: *mut *mut c_char = std::ptr::null_mut();
        let retval = crate::rs_expand_cmdline(
            xp,
            cmdbuff,
            cmdpos,
            &raw mut out_num_matches,
            &raw mut out_matches,
        );
        if retval != EXPAND_OK {
            return Err(retval);
        }
        Ok(ExpandState {
            num_matches: out_num_matches,
            matches: out_matches,
            showtail: rs_expand_showtail(xp) != 0,
            freshly_expanded: true,
        })
    } else {
        Ok(ExpandState {
            num_matches: (*xp).xp_numfiles,
            matches: (*xp).xp_files,
            showtail: nvim_get_cmd_showtail() != 0,
            freshly_expanded: false,
        })
    }
}

/// Display the list view of completion matches (for `wildmode=list`/CTRL-D).
///
/// # Safety
///
/// All pointer arguments must be valid for the duration of this call.
#[allow(clippy::too_many_lines)]
unsafe fn showmatches_list(
    xp: *mut crate::ExpandT,
    matches: *mut *mut c_char,
    num_matches: c_int,
    showtail: bool,
) {
    // Find the length of the longest match for column layout
    let mut maxlen: c_int = 0;
    for i in 0..num_matches {
        let len = if !showtail
            && ((*xp).xp_context == EXPAND_FILES
                || (*xp).xp_context == EXPAND_SHELLCMD
                || (*xp).xp_context == EXPAND_BUFFERS)
        {
            let s = nvim_cmdexpand_home_replace_match(*matches.add(i as usize));
            nvim_cmdexpand_vim_strsize(s)
        } else {
            let s = nvim_cmdexpand_show_match(matches, i, c_int::from(showtail));
            nvim_cmdexpand_vim_strsize(s)
        };
        if len > maxlen {
            maxlen = len;
        }
    }

    let columns = Columns;
    let lines;

    if (*xp).xp_context == EXPAND_TAGS_LISTFILES {
        lines = num_matches;
    } else {
        // Compute number of columns and lines
        let maxlen_with_gap = maxlen + 2; // two spaces between names
        let mut cols = (columns + 2) / maxlen_with_gap;
        if cols < 1 {
            cols = 1;
        }
        lines = (num_matches + cols - 1) / cols;
        maxlen = maxlen_with_gap;
    }

    if (*xp).xp_context == EXPAND_TAGS_LISTFILES {
        nvim_cmdexpand_msg_puts_hl(c"tagname".as_ptr(), HLF_T, 0);
        nvim_cmdexpand_msg_clr_eos();
        nvim_cmdexpand_msg_advance(maxlen - 3);
        nvim_cmdexpand_msg_puts_hl(c" kind file\n".as_ptr(), HLF_T, 0);
    }

    // List the matches line by line
    for i in 0..lines {
        showmatches_oneline(xp, matches, num_matches, lines, i, maxlen, showtail);
        if got_int {
            got_int = false;
            break;
        }
    }

    // Redraw the command below the listed lines
    cmdline_row = msg_row;
}

// =============================================================================
// showmatches
// =============================================================================

/// Display completion matches.
///
/// Returns `EXPAND_NOTHING` when the character that triggered expansion should be
/// inserted as a normal character.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` pointer. All match arrays and strings must
/// remain valid for the duration of this call.
///
/// # Panics
///
/// Does not panic under normal operation; asserts the `xp` pointer is non-null.
#[unsafe(export_name = "showmatches")]
pub unsafe extern "C" fn rs_showmatches(
    xp: *mut crate::ExpandT,
    display_wildmenu: bool,
    display_list: bool,
    noselect: bool,
) -> c_int {
    let state = match resolve_expand_state(xp) {
        Ok(s) => s,
        Err(retval) => return retval,
    };
    let ExpandState {
        num_matches,
        matches,
        showtail,
        freshly_expanded,
    } = state;

    // Use PUM if applicable
    if nvim_cmdexpand_compl_use_pum(c_int::from(display_wildmenu && !display_list)) != 0 {
        nvim_cmdexpand_pum_create_from_matches(
            xp,
            matches,
            num_matches,
            c_int::from(showtail),
            c_int::from(noselect),
        );
        nvim_set_compl_selected(if noselect { -1 } else { 0 });
        nvim_cmdexpand_pum_clear();
        nvim_cmdexpand_pum_display(1);
        return EXPAND_OK;
    }

    if display_list {
        msg_didany = false; // lines_left will be set
        msg_start(); // prepare for paging
        nvim_cmdexpand_msg_putchar(c_int::from(b'\n'));
        ui_flush();
        cmdline_row = msg_row;
        msg_didany = false; // lines_left will be set again
        nvim_cmdexpand_msg_ext_set_kind(c"wildlist".as_ptr());
        msg_start(); // prepare for paging
    }

    if got_int {
        got_int = false; // only interrupt the completion, not the cmd line
    } else if display_wildmenu && !display_list {
        // Display status-bar wildmenu
        nvim_cmdexpand_redraw_wildmenu_ex(
            xp,
            num_matches,
            matches,
            if noselect { -1 } else { 0 },
            c_int::from(showtail),
        );
    } else if display_list {
        showmatches_list(xp, matches, num_matches, showtail);
    }

    if freshly_expanded {
        nvim_cmdexpand_free_wild(num_matches, matches);
    }

    EXPAND_OK
}
