//! Buffer editing command implementation.
//!
//! Provides `rs_do_ecmd`, the Rust port of C `do_ecmd()`. Called when the
//! user runs `:edit`, `:split`, `:buffer`, argument-list navigation,
//! quickfix jumping, etc.

use std::ffi::{c_char, c_int};

use crate::{BufHandle, ExArgHandle, WinHandle};

// =============================================================================
// Constants matching C defines (verified by _Static_assert in ex_cmds_shim.c)
// =============================================================================

const ECMD_HIDE: c_int = 0x01;
const ECMD_SET_HELP: c_int = 0x02;
const ECMD_OLDBUF: c_int = 0x04;
const ECMD_FORCEIT: c_int = 0x08;
const ECMD_ADDBUF: c_int = 0x10;
const ECMD_ALTBUF: c_int = 0x20;
const ECMD_NOWINENTER: c_int = 0x40;

const ECMD_LASTL: c_int = 0;
const ECMD_LAST: c_int = -1;

const BF_NEVERLOADED: c_int = 0x04;
const BF_CHECK_RO: c_int = 0x02;
const BF_NOTEDITED: c_int = 0x08;

const BLN_CURBUF: c_int = 1;
const BLN_LISTED: c_int = 2;
const BLN_NOCURWIN: c_int = 128;

const CCGD_AW: c_int = 0x01;
const CCGD_MULTWIN: c_int = 0x02;
const CCGD_FORCEIT: c_int = 0x04;
const CCGD_EXCMD: c_int = 0x10;

const SEA_DIALOG: c_int = 1;
const SEA_QUIT: c_int = 2;

const KEYMAP_INIT: c_int = 1;
const DOBUF_UNLOAD: c_int = 2;

const READ_KEEP_UNDO: c_int = 0x20;
const READ_NOWINENTER: c_int = 0x80;

const BFA_KEEP_UNDO: c_int = 4;

const BL_WHITE: c_int = 1;
const BL_SOL: c_int = 2;
const BL_FIX: c_int = 4;

const OK: c_int = 1;
const FAIL: c_int = 0;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // curbuf field accessors
    fn nvim_ecmd_curbuf_get_b_fnum() -> c_int;
    fn nvim_ecmd_curbuf_get_ffname() -> *const c_char;
    fn nvim_ecmd_curbuf_get_fname() -> *const c_char;
    fn nvim_ecmd_curbuf_get_nwindows() -> c_int;
    fn nvim_ecmd_curbuf_get_b_flags() -> c_int;
    fn nvim_ecmd_curbuf_get_terminal() -> c_int;
    fn nvim_ecmd_curbuf_set_did_filetype(val: c_int);
    fn nvim_ecmd_curbuf_clear_flags(mask: c_int);
    fn nvim_ecmd_curbuf_set_flags(mask: c_int);
    fn nvim_ecmd_curbuf_set_last_used();
    fn nvim_ecmd_curbuf_get_kmap_state() -> c_int;
    fn nvim_ecmd_curbuf_get_help() -> c_int;
    fn nvim_ecmd_curbuf_get_line_count() -> c_int;
    fn nvim_ecmd_curbuf_clear_op_marks();

    // curwin field accessors
    fn nvim_ecmd_curwin_get_cursor(lnum_out: *mut c_int, col_out: *mut c_int);
    fn nvim_ecmd_curwin_set_cursor(lnum: c_int, col: c_int);
    fn nvim_ecmd_curwin_get_cursor_col() -> c_int;
    fn nvim_ecmd_curwin_get_cursor_lnum() -> c_int;
    fn nvim_ecmd_curwin_set_cursor_coladd(val: c_int);
    fn nvim_ecmd_curwin_set_w_set_curswant(val: c_int);
    fn nvim_ecmd_curwin_get_topline() -> c_int;
    fn nvim_ecmd_curwin_get_alt_fnum() -> c_int;
    fn nvim_ecmd_curwin_set_alt_fnum(fnum: c_int);
    fn nvim_ecmd_curwin_set_pcmark(lnum: c_int, col: c_int);
    fn nvim_ecmd_curwin_get_effective_p_so() -> c_int;
    fn nvim_ecmd_curwin_set_effective_p_so(val: c_int);
    fn nvim_ecmd_curwin_get_p_diff() -> c_int;
    fn nvim_ecmd_curwin_get_p_spell() -> c_int;
    fn nvim_ecmd_curwin_spl_is_empty() -> c_int;
    fn nvim_ecmd_curwin_set_scbind_pos_from_topline();
    fn nvim_ecmd_curwin_buf_is_null() -> c_int;
    fn nvim_ecmd_curwin_ws_is_own_buf() -> c_int;
    fn nvim_ecmd_curwin_set_ws_to_buf(buf: *mut BufHandle);

    // buf_T opaque handle accessors
    fn nvim_ecmd_buf_get_b_fnum(buf: *mut BufHandle) -> c_int;
    fn nvim_ecmd_buf_get_fname(buf: *mut BufHandle) -> *const c_char;
    fn nvim_ecmd_buf_has_memfile(buf: *mut BufHandle) -> c_int;
    fn nvim_ecmd_buf_get_locked_split(buf: *mut BufHandle) -> c_int;
    fn nvim_ecmd_buf_inc_nwindows(buf: *mut BufHandle);
    fn nvim_ecmd_buf_inc_locked(buf: *mut BufHandle);
    fn nvim_ecmd_buf_dec_locked(buf: *mut BufHandle);
    fn nvim_ecmd_buf_is_curbuf(buf: *mut BufHandle) -> c_int;
    fn nvim_ecmd_set_curbuf(buf: *mut BufHandle);
    fn nvim_ecmd_buf_is_alt(buf: *mut BufHandle) -> c_int;

    // win_T opaque handle accessors
    fn nvim_ecmd_win_buf_is_null(win: *mut WinHandle) -> c_int;
    fn nvim_ecmd_win_restore_buffer(win: *mut WinHandle, buf: *mut BufHandle);
    fn nvim_ecmd_win_set_locked(win: *mut WinHandle, val: c_int);

    // bufref_T heap handle accessors (exposed as void*)
    fn nvim_ecmd_new_bufref() -> *mut std::ffi::c_void;
    fn nvim_ecmd_free_bufref(r: *mut std::ffi::c_void);
    fn nvim_ecmd_set_bufref_to_curbuf(r: *mut std::ffi::c_void);
    fn nvim_ecmd_set_bufref_to_buf(r: *mut std::ffi::c_void, buf: *mut BufHandle);
    fn nvim_ecmd_bufref_valid(r: *mut std::ffi::c_void) -> c_int;
    fn nvim_ecmd_bufref_is_curbuf(r: *mut std::ffi::c_void) -> c_int;

    // au_new_curbuf global accessors
    fn nvim_ecmd_au_new_curbuf_set(buf: *mut BufHandle);
    fn nvim_ecmd_au_new_curbuf_valid() -> c_int;
    fn nvim_ecmd_au_new_curbuf_save() -> *mut std::ffi::c_void;
    fn nvim_ecmd_au_new_curbuf_restore(saved: *mut std::ffi::c_void);

    // Buffer operation wrappers
    fn nvim_ecmd_buf_check_timestamp(buf: *mut BufHandle);
    fn nvim_ecmd_buf_copy_options(buf: *mut BufHandle);
    fn nvim_ecmd_buf_freeall(flags: c_int);
    fn nvim_ecmd_buf_clear_file();
    fn nvim_ecmd_close_buffer(oldwin: *mut WinHandle, flags: c_int) -> c_int;
    fn nvim_ecmd_open_buffer(eap: *mut ExArgHandle, flags: c_int) -> c_int;
    fn nvim_ecmd_check_changed(flags: c_int) -> c_int;
    fn nvim_ecmd_u_savecommon(line_count: c_int) -> c_int;
    fn nvim_ecmd_u_unchanged();
    fn nvim_ecmd_u_sync();
    fn nvim_ecmd_buf_valid(buf: *mut BufHandle) -> c_int;
    fn nvim_ecmd_set_buflisted(val: c_int);
    fn nvim_ecmd_prepare_help_buffer();

    // Window/display wrappers
    fn nvim_ecmd_curwin_init();
    fn nvim_ecmd_get_winopts();
    fn nvim_ecmd_set_last_cursor();
    fn nvim_ecmd_maketitle();
    fn nvim_ecmd_parse_spelllang();
    fn nvim_ecmd_check_arg_idx();
    fn nvim_ecmd_do_autochdir();
    fn nvim_ecmd_changed_line_abv_curs();
    fn nvim_ecmd_update_topline();
    fn nvim_ecmd_redraw_curbuf_later();

    // Cursor manipulation wrappers
    fn nvim_ecmd_check_cursor();
    fn nvim_ecmd_check_cursor_lnum();
    fn nvim_ecmd_check_cursor_col();
    fn nvim_ecmd_check_fname() -> c_int;
    fn nvim_ecmd_beginline(flags: c_int);
    fn nvim_ecmd_cursor_eq(lnum: c_int, col: c_int) -> c_int;
    fn nvim_ecmd_cursor_col_skipwhite() -> c_int;

    // Autocmd wrappers
    fn nvim_ecmd_apply_autocmds_bufleave();
    fn nvim_ecmd_apply_autocmds_bufenter_retval(retval: *mut c_int);
    fn nvim_ecmd_apply_autocmds_bufwinenter_retval(retval: *mut c_int);

    // Global state accessors
    fn nvim_ecmd_inc_RedrawingDisabled();
    fn nvim_ecmd_dec_RedrawingDisabled();
    fn nvim_ecmd_get_swap_exists_action() -> c_int;
    fn nvim_ecmd_set_swap_exists_action(val: c_int);
    fn nvim_ecmd_cmdwin_buf_is_nonnull() -> c_int;
    fn nvim_ecmd_cmdwin_save_clear() -> *mut std::ffi::c_void;
    fn nvim_ecmd_cmdwin_restore_free(bundle: *mut std::ffi::c_void);
    fn nvim_ecmd_get_exmode_active() -> c_int;
    fn nvim_ecmd_get_skip_redraw() -> c_int;
    fn nvim_ecmd_get_keep_help_flag() -> c_int;
    fn nvim_ecmd_cmdmod_has_keepalt() -> c_int;
    fn nvim_ecmd_get_p_awa() -> c_int;
    fn nvim_ecmd_get_p_sol() -> c_int;
    fn nvim_ecmd_get_msg_scroll() -> c_int;
    fn nvim_ecmd_set_msg_scroll(val: c_int);
    fn nvim_ecmd_set_msg_scrolled_ign(val: c_int);
    fn nvim_ecmd_get_msg_listdo_overwrite() -> c_int;
    fn nvim_ecmd_get_exiting() -> c_int;
    fn nvim_ecmd_get_p_verbose() -> c_int;
    fn nvim_ecmd_get_p_ur() -> i64;

    // Misc wrappers
    fn nvim_ecmd_buflist_altfpos(win: *mut WinHandle);
    fn nvim_ecmd_buflist_findfmark(buf: *mut BufHandle, lnum: *mut c_int, col: *mut c_int);
    fn nvim_ecmd_terminal_check_size_bufref(r: *mut std::ffi::c_void);
    fn nvim_ecmd_terminal_check_size_curbuf();
    fn nvim_ecmd_handle_swap_exists(old_curbuf_ref: *mut std::ffi::c_void);
    fn nvim_ecmd_setaltfname(ffname: *mut c_char, sfname: *mut c_char, lnum: c_int);
    fn nvim_ecmd_delbuf_msg(name: *mut c_char);
    fn nvim_ecmd_fix_fname(ffname: *mut c_char) -> *mut c_char;
    fn nvim_ecmd_otherfile(ffname: *mut c_char) -> c_int;
    fn nvim_ecmd_path_fix_case(sfname: *mut c_char);
    fn nvim_ecmd_has_case_insensitive_filename() -> c_int;
    fn nvim_ecmd_buflist_findnr(fnum: c_int) -> *mut BufHandle;
    fn nvim_ecmd_buflist_new(
        ffname: *mut c_char,
        sfname: *mut c_char,
        lnum: c_int,
        flags: c_int,
    ) -> *mut BufHandle;
    fn nvim_ecmd_set_file_options(eap: *mut ExArgHandle);
    fn nvim_ecmd_do_modelines();
    fn nvim_ecmd_keymap_init();
    fn nvim_ecmd_fold_update_all_curbuf_wins();
    fn nvim_ecmd_shortmess_overall() -> c_int;
    fn nvim_ecmd_shortmess_fileinfo() -> c_int;
    fn nvim_ecmd_msg_check_for_delay();
    fn nvim_ecmd_msg_start();
    fn nvim_ecmd_fileinfo();
    fn nvim_ecmd_eap_get_do_ecmd_cmd(eap: *mut ExArgHandle) -> *const c_char;
    fn nvim_ecmd_do_cmdline(command: *const c_char);
    fn nvim_ecmd_clear_swapcommand();
    fn nvim_ecmd_set_swapcommand(command: *const c_char, newlnum: c_int) -> c_int;
    fn nvim_ecmd_emsg_closing_buffer();
    fn nvim_ecmd_xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_ecmd_xfree(p: *mut std::ffi::c_void);
    fn nvim_ecmd_atol(s: *const c_char) -> c_int;
    fn nvim_ecmd_curbuf_changed_from_bufref(old_curbuf_ref: *mut std::ffi::c_void) -> c_int;
    fn nvim_ecmd_curbuf_is_old_buf(old_curbuf_ref: *mut std::ffi::c_void) -> c_int;
    fn nvim_ecmd_should_dec_nwindows_on_locked(oldwin: *mut WinHandle) -> c_int;
    fn nvim_ecmd_dec_curwin_buf_nwindows_safe();

    // From existing accessors
    fn nvim_get_curwin() -> *mut WinHandle;
    fn nvim_get_curbuf() -> *mut BufHandle;
    fn nvim_excmds_aborting() -> c_int;

    // rs_* already used elsewhere
    fn rs_reset_VIsual();
    fn rs_win_valid(win: *mut WinHandle) -> c_int;
    fn rs_win_valid_any_tab(win: *mut WinHandle) -> c_int;
    fn rs_diff_buf_add(buf: *mut BufHandle);
    fn rs_diff_invalidate(buf: *mut BufHandle);
    fn rs_check_lnums(do_curwin: c_int);

    fn xfree(p: *mut std::ffi::c_void);
}

// =============================================================================
// rs_do_ecmd -- Rust implementation of do_ecmd
// =============================================================================

/// Start editing a new file.
///
/// Rust implementation of C `do_ecmd()`. Called by the thin C wrapper below.
///
/// Uses `'outer: loop { ... break 'outer; }` to simulate C's `goto theend`.
///
/// # Safety
/// Called from C; all pointer parameters may be null except as noted.
#[allow(clippy::never_loop)] // 'outer: loop used to simulate C's goto theend
#[allow(unused_assignments)] // variables mirror C locals; some intermediate assignments are intentional
#[no_mangle]
pub unsafe extern "C" fn rs_do_ecmd(
    fnum: c_int,
    ffname: *mut c_char,
    sfname: *mut c_char,
    eap: *mut ExArgHandle,
    newlnum: c_int,
    flags: c_int,
    oldwin: *mut WinHandle,
) -> c_int {
    // Mutable locals mirroring C's local variables
    let mut ffname = ffname;
    let mut sfname = sfname;
    let mut oldwin = oldwin;
    let mut newlnum = newlnum;

    let mut other_file: bool = false;
    let mut oldbuf: c_int = 0;
    let mut auto_buf = false;
    let mut new_name: *mut c_char = std::ptr::null_mut();
    let mut did_set_swapcommand = false;
    let mut buf: *mut BufHandle = std::ptr::null_mut();
    let mut retval = FAIL;
    let mut topline: c_int = 0;
    let mut newcol: c_int = -1;
    let mut solcol: c_int = -1;
    let mut did_get_winopts = false;
    let mut readfile_flags: c_int = 0;
    let mut did_inc_redrawing_disabled = false;
    let mut free_fname: *mut c_char = std::ptr::null_mut();

    // Save effective scroll offset (curwin->w_p_so or p_so)
    let orig_p_so = nvim_ecmd_curwin_get_effective_p_so();

    // Get the post-load command from eap
    let command: *const c_char = if !eap.is_null() {
        nvim_ecmd_eap_get_do_ecmd_cmd(eap)
    } else {
        std::ptr::null()
    };

    // Allocate heap bufrefs to avoid stack-allocated C structs in FFI
    let old_curbuf = nvim_ecmd_new_bufref();
    let bufref = nvim_ecmd_new_bufref();
    nvim_ecmd_set_bufref_to_curbuf(old_curbuf);

    // Use `'outer: loop` to simulate C's `goto theend` pattern
    'outer: loop {
        // ----------------------------------------------------------------
        // Determine other_file
        // ----------------------------------------------------------------
        if fnum != 0 {
            if fnum == nvim_ecmd_curbuf_get_b_fnum() {
                // File already being edited -- nothing to do
                retval = OK;
                break 'outer;
            }
            other_file = true;
        } else {
            if sfname.is_null() {
                sfname = ffname;
            }
            if nvim_ecmd_has_case_insensitive_filename() != 0 && !sfname.is_null() {
                nvim_ecmd_path_fix_case(sfname);
            }
            if (flags & (ECMD_ADDBUF | ECMD_ALTBUF)) != 0 && (ffname.is_null() || *ffname == 0) {
                break 'outer; // retval stays FAIL
            }
            if ffname.is_null() {
                other_file = true;
            } else if *ffname == 0 && nvim_ecmd_curbuf_get_ffname().is_null() {
                other_file = false;
            } else {
                if *ffname == 0 {
                    // Re-edit with same file name
                    ffname = nvim_ecmd_curbuf_get_ffname() as *mut c_char;
                    sfname = nvim_ecmd_curbuf_get_fname() as *mut c_char;
                }
                free_fname = nvim_ecmd_fix_fname(ffname);
                if !free_fname.is_null() {
                    ffname = free_fname;
                }
                other_file = nvim_ecmd_otherfile(ffname) != 0;
            }
        }

        // ----------------------------------------------------------------
        // Re-editing a terminal buffer: skip most re-initialization
        // ----------------------------------------------------------------
        if !other_file && nvim_ecmd_curbuf_get_terminal() != 0 {
            nvim_ecmd_check_arg_idx();
            nvim_ecmd_maketitle();
            retval = OK;
            break 'outer;
        }

        // ----------------------------------------------------------------
        // Check if we can abandon the current buffer
        // ----------------------------------------------------------------
        let need_check = (!other_file && (flags & ECMD_OLDBUF) == 0)
            || (nvim_ecmd_curbuf_get_nwindows() == 1
                && (flags & (ECMD_HIDE | ECMD_ADDBUF | ECMD_ALTBUF)) == 0);
        if need_check {
            let ccgd = if nvim_ecmd_get_p_awa() != 0 {
                CCGD_AW
            } else {
                0
            } | if other_file { 0 } else { CCGD_MULTWIN }
                | if (flags & ECMD_FORCEIT) != 0 {
                    CCGD_FORCEIT
                } else {
                    0
                }
                | if !eap.is_null() { CCGD_EXCMD } else { 0 };
            if nvim_ecmd_check_changed(ccgd) != 0 {
                if fnum == 0 && other_file && !ffname.is_null() {
                    nvim_ecmd_setaltfname(ffname, sfname, if newlnum < 0 { 0 } else { newlnum });
                }
                break 'outer; // retval stays FAIL
            }
        }

        // End Visual mode (may fire ModeChanged autocmd)
        rs_reset_VIsual();

        // Autocommands may have freed oldwin
        if !oldwin.is_null() && rs_win_valid(oldwin) == 0 {
            oldwin = std::ptr::null_mut();
        }

        // Set up v:swapcommand
        did_set_swapcommand = nvim_ecmd_set_swapcommand(command, newlnum) != 0;

        // ================================================================
        // other_file: open or find the buffer for the other file
        // ================================================================
        if other_file {
            let prev_alt_fnum = nvim_ecmd_curwin_get_alt_fnum();

            if (flags & (ECMD_ADDBUF | ECMD_ALTBUF)) == 0 {
                if nvim_ecmd_cmdmod_has_keepalt() == 0 {
                    nvim_ecmd_curwin_set_alt_fnum(nvim_ecmd_curbuf_get_b_fnum());
                }
                if !oldwin.is_null() {
                    nvim_ecmd_buflist_altfpos(oldwin);
                }
            }

            if fnum != 0 {
                buf = nvim_ecmd_buflist_findnr(fnum);
            } else if (flags & (ECMD_ADDBUF | ECMD_ALTBUF)) != 0 {
                // Add buffer to list without editing
                let mut tlnum: c_int = 0;
                if !command.is_null() {
                    tlnum = nvim_ecmd_atol(command);
                    if tlnum <= 0 {
                        tlnum = 1;
                    }
                }
                let newbuf =
                    nvim_ecmd_buflist_new(ffname, sfname, tlnum, BLN_LISTED | BLN_NOCURWIN);
                if !newbuf.is_null() && (flags & ECMD_ALTBUF) != 0 {
                    nvim_ecmd_curwin_set_alt_fnum(nvim_ecmd_buf_get_b_fnum(newbuf));
                }
                break 'outer; // retval stays FAIL (goto theend)
            } else {
                buf = nvim_ecmd_buflist_new(
                    ffname,
                    sfname,
                    0,
                    BLN_CURBUF
                        | if (flags & ECMD_SET_HELP) != 0 {
                            0
                        } else {
                            BLN_LISTED
                        },
                );
                // Autocmds may change curwin and curbuf
                if !oldwin.is_null() {
                    oldwin = nvim_get_curwin();
                }
                nvim_ecmd_set_bufref_to_curbuf(old_curbuf);
            }

            if buf.is_null() {
                break 'outer; // retval stays FAIL
            }

            // autocommands try to edit a closing buffer -- abort
            if nvim_ecmd_buf_get_locked_split(buf) != 0 {
                if nvim_ecmd_should_dec_nwindows_on_locked(oldwin) != 0 {
                    nvim_ecmd_dec_curwin_buf_nwindows_safe();
                }
                nvim_ecmd_emsg_closing_buffer();
                break 'outer; // retval stays FAIL
            }

            if nvim_ecmd_buf_is_alt(buf) != 0 && prev_alt_fnum != 0 {
                // Reusing buffer: keep old alternate file
                nvim_ecmd_curwin_set_alt_fnum(prev_alt_fnum);
            }

            if nvim_ecmd_buf_has_memfile(buf) == 0 {
                oldbuf = 0;
            } else {
                oldbuf = 1;
                nvim_ecmd_set_bufref_to_buf(bufref, buf);
                nvim_ecmd_buf_check_timestamp(buf);
                // Check if autocommands invalidated the buffer or changed curbuf
                if nvim_ecmd_bufref_valid(bufref) == 0
                    || nvim_ecmd_curbuf_changed_from_bufref(old_curbuf) != 0
                {
                    break 'outer;
                }
                if nvim_excmds_aborting() != 0 {
                    break 'outer;
                }
            }

            // May jump to last used line number for a loaded buffer
            if (oldbuf != 0 && newlnum == ECMD_LASTL) || newlnum == ECMD_LAST {
                let mut lnum = 0;
                let mut col = 0;
                nvim_ecmd_buflist_findfmark(buf, &mut lnum, &mut col);
                newlnum = lnum;
                solcol = col;
            }

            // Make the (new) buffer the one used by the current window
            if nvim_ecmd_buf_is_curbuf(buf) == 0 {
                // cmdwin_buf must be NULL
                debug_assert!(
                    nvim_ecmd_cmdwin_buf_is_nonnull() == 0,
                    "cmdwin_buf must be NULL when switching buffers"
                );

                // Save and clear cmdwin state for BufLeave
                let cmdwin_state = nvim_ecmd_cmdwin_save_clear();

                // Track new_name for delbuf_msg
                let fname = nvim_ecmd_buf_get_fname(buf);
                if !fname.is_null() {
                    new_name = nvim_ecmd_xstrdup(fname);
                }

                // Save au_new_curbuf and point it at the new buffer
                let save_au = nvim_ecmd_au_new_curbuf_save();
                nvim_ecmd_au_new_curbuf_set(buf);

                // Fire BufLeave for the current (old) buffer
                nvim_ecmd_apply_autocmds_bufleave();

                // Restore cmdwin state
                nvim_ecmd_cmdwin_restore_free(cmdwin_state);

                if nvim_ecmd_au_new_curbuf_valid() == 0 {
                    // New buffer has been deleted
                    nvim_ecmd_delbuf_msg(new_name); // frees new_name
                    new_name = std::ptr::null_mut();
                    nvim_ecmd_au_new_curbuf_restore(save_au);
                    nvim_ecmd_xfree(save_au);
                    break 'outer;
                }
                if nvim_excmds_aborting() != 0 {
                    nvim_ecmd_xfree(new_name as *mut std::ffi::c_void);
                    new_name = std::ptr::null_mut();
                    nvim_ecmd_au_new_curbuf_restore(save_au);
                    nvim_ecmd_xfree(save_au);
                    break 'outer;
                }

                if nvim_ecmd_buf_is_curbuf(buf) != 0 {
                    // Already in new buffer (autocmd moved us there)
                    auto_buf = true;
                } else {
                    let the_curwin = nvim_get_curwin();
                    let was_curbuf = nvim_get_curbuf();

                    // Lock window and buffer to prevent autocmds from closing
                    nvim_ecmd_win_set_locked(the_curwin, 1);
                    nvim_ecmd_buf_inc_locked(buf);

                    // Copy options if curbuf is still the old curbuf
                    if nvim_ecmd_curbuf_is_old_buf(old_curbuf) != 0 {
                        nvim_ecmd_buf_copy_options(buf);
                    }

                    // Close the link to the current buffer
                    nvim_ecmd_u_sync();
                    let close_flags =
                        if (flags & ECMD_HIDE) != 0 || nvim_ecmd_curbuf_get_terminal() != 0 {
                            0
                        } else {
                            DOBUF_UNLOAD
                        };
                    let did_decrement = nvim_ecmd_close_buffer(oldwin, close_flags);

                    // Autocommands may have closed the window
                    if rs_win_valid(the_curwin) != 0 {
                        nvim_ecmd_win_set_locked(the_curwin, 0);
                    }
                    nvim_ecmd_buf_dec_locked(buf);

                    // Autocmds may abort script processing
                    if nvim_excmds_aborting() != 0 && nvim_ecmd_curwin_buf_is_null() == 0 {
                        nvim_ecmd_xfree(new_name as *mut std::ffi::c_void);
                        new_name = std::ptr::null_mut();
                        nvim_ecmd_au_new_curbuf_restore(save_au);
                        nvim_ecmd_xfree(save_au);
                        break 'outer;
                    }
                    if nvim_ecmd_au_new_curbuf_valid() == 0 {
                        nvim_ecmd_delbuf_msg(new_name);
                        new_name = std::ptr::null_mut();
                        nvim_ecmd_au_new_curbuf_restore(save_au);
                        nvim_ecmd_xfree(save_au);
                        break 'outer;
                    }

                    if nvim_ecmd_buf_is_curbuf(buf) != 0 {
                        // Already in new buffer after close_buffer()
                        if did_decrement != 0 && nvim_ecmd_buf_valid(was_curbuf) != 0 {
                            nvim_ecmd_buf_inc_nwindows(was_curbuf);
                        }
                        if rs_win_valid_any_tab(oldwin) != 0
                            && nvim_ecmd_win_buf_is_null(oldwin) != 0
                        {
                            nvim_ecmd_win_restore_buffer(oldwin, was_curbuf);
                        }
                        auto_buf = true;
                    } else {
                        // Set synblock if it's pointing to old buffer's synblock
                        if nvim_ecmd_curwin_buf_is_null() != 0
                            || nvim_ecmd_curwin_ws_is_own_buf() != 0
                        {
                            nvim_ecmd_curwin_set_ws_to_buf(buf);
                        }
                        // Set curbuf = buf, curwin->w_buffer = buf, b_nwindows++
                        nvim_ecmd_set_curbuf(buf);

                        // Set 'fileformat', 'binary', 'fenc' when forced
                        if oldbuf == 0 && !eap.is_null() {
                            nvim_ecmd_set_file_options(eap);
                        }
                    }

                    // Get window options from last time buffer was in this window
                    nvim_ecmd_get_winopts();
                    did_get_winopts = true;
                }

                nvim_ecmd_xfree(new_name as *mut std::ffi::c_void);
                new_name = std::ptr::null_mut();
                nvim_ecmd_au_new_curbuf_restore(save_au);
                nvim_ecmd_xfree(save_au);
            }

            // Set pcmark to beginning of buffer
            nvim_ecmd_curwin_set_pcmark(1, 0);
        } else {
            // ============================================================
            // !other_file: re-editing the same file
            // ============================================================
            if (flags & (ECMD_ADDBUF | ECMD_ALTBUF)) != 0 || nvim_ecmd_check_fname() == 0 {
                break 'outer; // retval stays FAIL
            }
            oldbuf = if (flags & ECMD_OLDBUF) != 0 { 1 } else { 0 };
        }

        // ================================================================
        // Disable redraw until cursor is in the right position
        // ================================================================
        nvim_ecmd_inc_RedrawingDisabled();
        did_inc_redrawing_disabled = true;

        buf = nvim_get_curbuf();

        if (flags & ECMD_SET_HELP) != 0 || nvim_ecmd_get_keep_help_flag() != 0 {
            nvim_ecmd_prepare_help_buffer();
        } else if nvim_ecmd_curbuf_get_help() == 0 {
            nvim_ecmd_set_buflisted(1);
        }

        // If autocommands changed buffers, give up
        if nvim_ecmd_buf_is_curbuf(buf) == 0 {
            break 'outer;
        }
        if nvim_excmds_aborting() != 0 {
            break 'outer;
        }

        // Consider filetype unset
        nvim_ecmd_curbuf_set_did_filetype(0);

        // re-use the buffer case: !other_file && !oldbuf
        if !other_file && oldbuf == 0 {
            nvim_ecmd_set_last_cursor();
            if newlnum == ECMD_LAST || newlnum == ECMD_LASTL {
                newlnum = nvim_ecmd_curwin_get_cursor_lnum();
                solcol = nvim_ecmd_curwin_get_cursor_col();
            }
            buf = nvim_get_curbuf();
            let fname = nvim_ecmd_curbuf_get_fname();
            if !fname.is_null() {
                new_name = nvim_ecmd_xstrdup(fname);
            }
            nvim_ecmd_set_bufref_to_curbuf(bufref);

            // Store current contents for undo if buffer was used before
            let line_count = nvim_ecmd_curbuf_get_line_count();
            let p_ur = nvim_ecmd_get_p_ur();
            if (nvim_ecmd_curbuf_get_b_flags() & BF_NEVERLOADED) == 0
                && (p_ur < 0 || line_count as i64 <= p_ur)
            {
                nvim_ecmd_u_sync();
                if nvim_ecmd_u_savecommon(line_count) == 0 {
                    nvim_ecmd_xfree(new_name as *mut std::ffi::c_void);
                    new_name = std::ptr::null_mut();
                    break 'outer; // retval stays FAIL
                }
                nvim_ecmd_u_unchanged();
                nvim_ecmd_buf_freeall(BFA_KEEP_UNDO);
                readfile_flags = READ_KEEP_UNDO;
            } else {
                nvim_ecmd_buf_freeall(0);
            }

            // If autocommands deleted the buffer, give up
            if nvim_ecmd_bufref_valid(bufref) == 0 {
                nvim_ecmd_delbuf_msg(new_name);
                new_name = std::ptr::null_mut();
                break 'outer;
            }
            nvim_ecmd_xfree(new_name as *mut std::ffi::c_void);
            new_name = std::ptr::null_mut();

            // If autocommands changed buffers, give up
            if nvim_ecmd_buf_is_curbuf(buf) == 0 {
                break 'outer;
            }
            if nvim_excmds_aborting() != 0 {
                break 'outer;
            }
            nvim_ecmd_buf_clear_file();
            nvim_ecmd_curbuf_clear_op_marks();
        }

        // Assume success from here
        retval = OK;

        // Reset not-edit flag so ":write" works
        if !other_file {
            nvim_ecmd_curbuf_clear_flags(BF_NOTEDITED);
        }

        // Check if editing the w_arg_idx file in the argument list
        nvim_ecmd_check_arg_idx();

        if !auto_buf {
            // Init window before reading file / executing autocmds
            nvim_ecmd_curwin_init();

            // Update automatic folding for all windows showing curbuf
            nvim_ecmd_fold_update_all_curbuf_wins();

            // Change directories when 'acd' is set
            nvim_ecmd_do_autochdir();

            // Save cursor and topline for autocommand comparison
            let mut orig_lnum: c_int = 0;
            let mut orig_col: c_int = 0;
            nvim_ecmd_curwin_get_cursor(&mut orig_lnum, &mut orig_col);
            topline = nvim_ecmd_curwin_get_topline();

            if oldbuf == 0 {
                // Need to read the file
                nvim_ecmd_set_swap_exists_action(SEA_DIALOG);
                nvim_ecmd_curbuf_set_flags(BF_CHECK_RO);

                if (flags & ECMD_NOWINENTER) != 0 {
                    readfile_flags |= READ_NOWINENTER;
                }
                if nvim_ecmd_open_buffer(eap, readfile_flags) != 0 {
                    retval = FAIL;
                }

                if nvim_ecmd_get_swap_exists_action() == SEA_QUIT {
                    retval = FAIL;
                }
                nvim_ecmd_handle_swap_exists(old_curbuf);
            } else {
                // Read modelines (window-local options only)
                nvim_ecmd_do_modelines();

                nvim_ecmd_apply_autocmds_bufenter_retval(&mut retval);
                if (flags & ECMD_NOWINENTER) == 0 {
                    nvim_ecmd_apply_autocmds_bufwinenter_retval(&mut retval);
                }
            }
            nvim_ecmd_check_arg_idx();

            // If autocommands changed cursor position or topline, keep it.
            // But not if cursor moved to first non-blank.
            if nvim_ecmd_cursor_eq(orig_lnum, orig_col) == 0 {
                let cur_lnum = nvim_ecmd_curwin_get_cursor_lnum();
                let cur_col = nvim_ecmd_curwin_get_cursor_col();
                let skipwhite_col = nvim_ecmd_cursor_col_skipwhite();
                if cur_lnum != orig_lnum || cur_col != skipwhite_col {
                    newlnum = cur_lnum;
                    newcol = cur_col;
                }
            }
            if nvim_ecmd_curwin_get_topline() == topline {
                topline = 0;
            }

            // Even when cursor didn't move, recompute topline
            nvim_ecmd_changed_line_abv_curs();

            nvim_ecmd_maketitle();
        }

        // Tell diff about new/updated buffer
        if nvim_ecmd_curwin_get_p_diff() != 0 {
            rs_diff_buf_add(nvim_get_curbuf());
            rs_diff_invalidate(nvim_get_curbuf());
        }

        // If window options changed, set spell language
        if did_get_winopts
            && nvim_ecmd_curwin_get_p_spell() != 0
            && nvim_ecmd_curwin_spl_is_empty() == 0
        {
            nvim_ecmd_parse_spelllang();
        }

        // Position cursor
        if command.is_null() {
            if newcol >= 0 {
                // Position set by autocommands
                nvim_ecmd_curwin_set_cursor(newlnum, newcol);
                nvim_ecmd_check_cursor();
            } else if newlnum > 0 {
                // Line number from caller or old position
                nvim_ecmd_curwin_set_cursor(newlnum, nvim_ecmd_curwin_get_cursor_col());
                nvim_ecmd_check_cursor_lnum();
                if solcol >= 0 && nvim_ecmd_get_p_sol() == 0 {
                    // 'sol' is off: use last known column
                    nvim_ecmd_curwin_set_cursor(nvim_ecmd_curwin_get_cursor_lnum(), solcol);
                    nvim_ecmd_check_cursor_col();
                    nvim_ecmd_curwin_set_cursor_coladd(0);
                    nvim_ecmd_curwin_set_w_set_curswant(1);
                } else {
                    nvim_ecmd_beginline(BL_SOL | BL_FIX);
                }
            } else {
                // No line number -- go to last line in Ex mode
                if nvim_ecmd_get_exmode_active() != 0 {
                    nvim_ecmd_curwin_set_cursor(nvim_ecmd_curbuf_get_line_count(), 0);
                }
                nvim_ecmd_beginline(BL_WHITE | BL_FIX);
            }
        }

        // Check cursors in other windows on the same buffer
        rs_check_lnums(0);

        // Show file info for old buffers that weren't re-read
        if oldbuf != 0 && !auto_buf {
            let msg_scroll_save = nvim_ecmd_get_msg_scroll();

            // 'O' flag in 'cpoptions': overwrite previous file message
            if nvim_ecmd_shortmess_overall() != 0
                && nvim_ecmd_get_msg_listdo_overwrite() == 0
                && nvim_ecmd_get_exiting() == 0
                && nvim_ecmd_get_p_verbose() == 0
            {
                nvim_ecmd_set_msg_scroll(0);
            }
            if nvim_ecmd_get_msg_scroll() == 0 {
                nvim_ecmd_msg_check_for_delay();
            }
            nvim_ecmd_msg_start();
            nvim_ecmd_set_msg_scroll(msg_scroll_save);
            nvim_ecmd_set_msg_scrolled_ign(1);

            if nvim_ecmd_shortmess_fileinfo() == 0 {
                nvim_ecmd_fileinfo();
            }

            nvim_ecmd_set_msg_scrolled_ign(0);
        }

        nvim_ecmd_curbuf_set_last_used();

        if !command.is_null() {
            nvim_ecmd_do_cmdline(command);
        }

        if (nvim_ecmd_curbuf_get_kmap_state() & KEYMAP_INIT) != 0 {
            nvim_ecmd_keymap_init();
        }

        nvim_ecmd_dec_RedrawingDisabled();
        did_inc_redrawing_disabled = false;

        if nvim_ecmd_get_skip_redraw() == 0 {
            // Force cursor to center if no topline and no +cmd
            if topline == 0 && command.is_null() {
                nvim_ecmd_curwin_set_effective_p_so(999);
            }
            nvim_ecmd_update_topline();
            nvim_ecmd_curwin_set_scbind_pos_from_topline();
            // Restore original scroll offset
            nvim_ecmd_curwin_set_effective_p_so(orig_p_so);
            nvim_ecmd_redraw_curbuf_later();
        }

        nvim_ecmd_do_autochdir();

        break 'outer; // Normal exit
    } // end 'outer

    // theend: cleanup
    nvim_ecmd_terminal_check_size_bufref(old_curbuf);
    if nvim_ecmd_bufref_valid(old_curbuf) == 0 || nvim_ecmd_bufref_is_curbuf(old_curbuf) == 0 {
        nvim_ecmd_terminal_check_size_curbuf();
    }

    if did_inc_redrawing_disabled {
        nvim_ecmd_dec_RedrawingDisabled();
    }

    if did_set_swapcommand {
        nvim_ecmd_clear_swapcommand();
    }

    if !free_fname.is_null() {
        xfree(free_fname as *mut std::ffi::c_void);
    }

    nvim_ecmd_free_bufref(old_curbuf);
    nvim_ecmd_free_bufref(bufref);

    retval
}
