//! Buffer editing command implementation.
//!
//! Provides `rs_do_ecmd`, the Rust port of C `do_ecmd()`. Called when the
//! user runs `:edit`, `:split`, `:buffer`, argument-list navigation,
//! quickfix jumping, etc.

use std::ffi::{c_char, c_int};

use crate::{BufHandle, ExArgHandle, WinHandle};

use libc::atol as c_atol;

// =============================================================================
// Constants matching C defines
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

// Additional constants needed for direct C function calls (Phase 3)
const UPD_NOT_VALID: c_int = 40; // buffer needs complete redraw
const OPT_WINONLY: c_int = 0x08; // only set window-local options
const BCO_ENTER: c_int = 1; // buf_copy_options: entering buffer

// SHM_* flags for shortmess() (from option_vars.h -- ASCII character values)
const SHM_OVERALL: c_int = b'O' as c_int; // 'O': overwrite more messages
const SHM_FILEINFO: c_int = b'F' as c_int; // 'F': no file info messages

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // curbuf field accessors
    fn nvim_excmds_curbuf_get_b_fnum() -> c_int;
    fn nvim_excmds_curbuf_get_ffname() -> *mut c_char;
    fn nvim_excmds_curbuf_get_fname() -> *mut c_char;
    fn nvim_excmds_curbuf_get_b_nwindows() -> c_int;
    fn nvim_ecmd_curbuf_get_b_flags() -> c_int;
    fn nvim_ecmd_curbuf_get_terminal() -> c_int;
    fn nvim_ecmd_curbuf_set_did_filetype(val: c_int);
    fn nvim_ecmd_curbuf_clear_flags(mask: c_int);
    fn nvim_ecmd_curbuf_set_flags(mask: c_int);
    fn nvim_ecmd_curbuf_set_last_used();
    fn nvim_ecmd_curbuf_get_kmap_state() -> c_int;
    fn nvim_ecmd_curbuf_get_help() -> c_int;
    fn nvim_excmds_curbuf_ml_line_count() -> c_int;
    fn nvim_ecmd_curbuf_clear_op_marks();

    // curwin field accessors
    fn nvim_ecmd_curwin_get_cursor(lnum_out: *mut c_int, col_out: *mut c_int);
    fn nvim_ecmd_curwin_set_cursor(lnum: c_int, col: c_int);
    fn nvim_ecmd_curwin_get_cursor_col() -> c_int;
    fn nvim_excmds_curwin_cursor_lnum() -> c_int;
    fn nvim_ecmd_curwin_set_coladd_curswant();
    fn nvim_ecmd_curwin_get_topline() -> c_int;
    fn nvim_ecmd_curwin_get_alt_fnum() -> c_int;
    fn nvim_excmds_set_curwin_alt_fnum(fnum: c_int);
    fn nvim_ecmd_curwin_set_pcmark(lnum: c_int, col: c_int);
    fn nvim_ecmd_curwin_get_effective_p_so() -> c_int;
    fn nvim_ecmd_curwin_set_effective_p_so(val: c_int);
    fn nvim_ecmd_curwin_diff_spell_state(
        diff_out: *mut c_int,
        spell_out: *mut c_int,
        spl_empty_out: *mut c_int,
    );
    fn nvim_ecmd_curwin_set_scbind_pos_from_topline();
    fn nvim_ecmd_curwin_buf_is_null() -> c_int;
    fn nvim_ecmd_curwin_ws_is_own_buf() -> c_int;
    fn nvim_ecmd_curwin_set_ws_to_buf(buf: *mut BufHandle);

    // buf_T opaque handle accessors
    fn nvim_excmds_buf_get_b_fnum(buf: *const BufHandle) -> c_int;
    fn nvim_excmds_buf_get_b_fname(buf: *const BufHandle) -> *const c_char;
    fn nvim_ecmd_buf_has_memfile(buf: *mut BufHandle) -> c_int;
    fn nvim_ecmd_buf_get_locked_split(buf: *mut BufHandle) -> c_int;
    fn nvim_ecmd_buf_inc_nwindows(buf: *mut BufHandle);
    fn nvim_ecmd_buf_inc_locked(buf: *mut BufHandle);
    fn nvim_ecmd_buf_dec_locked(buf: *mut BufHandle);
    fn nvim_ecmd_buf_is_curbuf(buf: *mut BufHandle) -> c_int;
    fn nvim_ecmd_set_curbuf(buf: *mut BufHandle);

    // win_T opaque handle accessors
    fn nvim_ecmd_win_buf_is_null(win: *mut WinHandle) -> c_int;
    fn nvim_ecmd_win_restore_buffer(win: *mut WinHandle, buf: *mut BufHandle);
    fn nvim_ecmd_win_set_locked(win: *mut WinHandle, val: c_int);

    // bufref_T heap handle accessors (exposed as void*)
    fn nvim_ecmd_new_bufref() -> *mut std::ffi::c_void;
    fn xfree(ptr: *mut std::ffi::c_void);
    fn nvim_ecmd_set_bufref_to_curbuf(r: *mut std::ffi::c_void);
    fn set_bufref(ref_: *mut std::ffi::c_void, buf: *mut BufHandle);
    fn bufref_valid(ref_: *mut std::ffi::c_void) -> bool;
    fn nvim_ecmd_bufref_is_curbuf(r: *mut std::ffi::c_void) -> c_int;

    // au_new_curbuf global accessors
    fn nvim_ecmd_au_new_curbuf_set(buf: *mut BufHandle);
    fn nvim_ecmd_au_new_curbuf_valid() -> c_int;
    fn nvim_ecmd_au_new_curbuf_save() -> *mut std::ffi::c_void;
    fn nvim_ecmd_au_new_curbuf_restore(saved: *mut std::ffi::c_void);

    // Buffer operation wrappers (direct C calls)
    #[link_name = "buf_check_timestamp"]
    fn nvim_ex2_buf_check_timestamp(buf: *mut BufHandle);
    fn buf_copy_options(buf: *mut BufHandle, flags: c_int);
    fn buf_freeall(buf: *mut BufHandle, flags: c_int);
    fn buf_clear_file(buf: *mut BufHandle);
    fn close_buffer(
        win: *mut WinHandle,
        buf: *mut BufHandle,
        action: c_int,
        abort_if_last: bool,
        ignore_abort: bool,
    ) -> bool;
    fn open_buffer(read_stdin: bool, eap: *mut ExArgHandle, flags: c_int) -> c_int;
    fn should_abort(retval: c_int) -> c_int;
    fn check_changed(buf: *mut BufHandle, flags: c_int) -> bool;
    fn u_savecommon(
        buf: *mut BufHandle,
        top: c_int,
        bot: c_int,
        newbot: c_int,
        reload: bool,
    ) -> c_int;
    fn u_unchanged(buf: *mut BufHandle);
    fn nvim_u_sync(force: bool);
    fn buf_valid(buf: *mut BufHandle) -> bool;
    fn set_buflisted(on: c_int);
    fn prepare_help_buffer();

    // Window/display wrappers (direct C calls)
    fn curwin_init();
    fn get_winopts(buf: *mut BufHandle);
    fn set_last_cursor(win: *mut WinHandle);
    fn nvim_maketitle();
    fn parse_spelllang(win: *mut WinHandle) -> *mut c_char;
    fn check_arg_idx(win: *mut WinHandle);
    fn do_autochdir();
    fn changed_line_abv_curs();
    fn nvim_excmds_update_topline_curwin();
    fn redraw_curbuf_later(type_: c_int);

    // Cursor manipulation wrappers
    fn nvim_check_cursor();
    fn check_cursor_col(win: *mut WinHandle);
    fn check_fname() -> c_int;
    fn nvim_beginline(flags: c_int);
    fn nvim_ecmd_cursor_eq(lnum: c_int, col: c_int) -> c_int;
    fn nvim_ecmd_cursor_col_skipwhite() -> c_int;

    // Autocmd wrappers
    fn nvim_ecmd_apply_autocmds_bufleave();
    fn nvim_ecmd_apply_autocmds_bufenter_retval(retval: *mut c_int);
    fn nvim_ecmd_apply_autocmds_bufwinenter_retval(retval: *mut c_int);

    // Global state accessors
    fn nvim_inc_RedrawingDisabled();
    fn nvim_dec_RedrawingDisabled();
    fn nvim_get_swap_exists_action() -> c_int;
    fn nvim_set_swap_exists_action(val: c_int);
    fn nvim_ecmd_cmdwin_buf_is_nonnull() -> c_int;
    fn nvim_ecmd_cmdwin_save_clear() -> *mut std::ffi::c_void;
    fn nvim_ecmd_cmdwin_restore_free(bundle: *mut std::ffi::c_void);
    static mut exmode_active: bool;
    fn nvim_get_skip_redraw() -> bool;
    fn nvim_excmds_cmdmod_has_keepalt() -> c_int;
    fn nvim_get_p_sol() -> c_int;
    static mut msg_scroll: c_int;
    fn nvim_get_p_verbose() -> c_int;
    // Misc wrappers
    fn rs_buflist_altfpos(win: *mut WinHandle);
    fn nvim_ecmd_buflist_findfmark(buf: *mut BufHandle, lnum: *mut c_int, col: *mut c_int);
    fn nvim_ecmd_terminal_check_size_cleanup(r: *mut std::ffi::c_void);
    fn nvim_ecmd_handle_swap_exists(old_curbuf_ref: *mut std::ffi::c_void);
    fn setaltfname(ffname: *mut c_char, sfname: *mut c_char, lnum: c_int) -> *mut crate::BufHandle;
    fn rs_delbuf_msg(name: *mut c_char);
    #[link_name = "fix_fname"]
    fn nvim_fix_fname(ffname: *const c_char) -> *mut c_char;
    fn rs_otherfile(ffname: *const c_char) -> bool;
    fn nvim_ecmd_path_fix_case(sfname: *mut c_char);
    fn nvim_ecmd_has_case_insensitive_filename() -> c_int;
    #[link_name = "rs_buflist_findnr"]
    fn nvim_buflist_findnr(fnum: c_int) -> *mut BufHandle;
    fn buflist_new(
        ffname_arg: *mut c_char,
        sfname_arg: *mut c_char,
        lnum: c_int,
        flags: c_int,
    ) -> *mut BufHandle;
    fn set_file_options(set_options: bool, eap: *mut ExArgHandle);
    fn set_forced_fenc(eap: *mut ExArgHandle);
    fn do_modelines(flags: c_int);
    fn keymap_init() -> *mut c_char;
    fn nvim_ecmd_fold_update_all_curbuf_wins();
    fn shortmess(x: c_int) -> bool;
    fn msg_check_for_delay(check_msg_scroll: bool);
    fn msg_start();
    fn nvim_fileinfo_call();
    fn nvim_ecmd_eap_get_do_ecmd_cmd(eap: *mut ExArgHandle) -> *const c_char;
    fn do_cmdline(
        cmdline: *mut c_char,
        fgetline: *mut std::ffi::c_void,
        cookie: *mut std::ffi::c_void,
        flags: c_int,
    ) -> c_int;
    fn nvim_ecmd_clear_swapcommand();

    fn nvim_ecmd_emsg_closing_buffer();
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_ecmd_should_dec_nwindows_on_locked(oldwin: *mut WinHandle) -> c_int;
    fn nvim_ecmd_dec_curwin_buf_nwindows_safe();

    // From existing accessors
    fn nvim_get_curwin() -> *mut WinHandle;
    fn nvim_get_curbuf() -> *mut BufHandle;
    fn aborting() -> c_int;

    // rs_* already used elsewhere
    fn rs_reset_VIsual();
    fn rs_win_valid(win: *mut WinHandle) -> c_int;
    fn rs_win_valid_any_tab(win: *mut WinHandle) -> c_int;
    fn rs_diff_buf_add(buf: *mut BufHandle);
    fn rs_diff_invalidate(buf: *mut BufHandle);
    fn rs_check_lnums(do_curwin: c_int);
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
#[allow(clippy::must_use_candidate)]
#[export_name = "do_ecmd"]
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
            if fnum == nvim_excmds_curbuf_get_b_fnum() {
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
            } else if *ffname == 0 && nvim_excmds_curbuf_get_ffname().is_null() {
                other_file = false;
            } else {
                if *ffname == 0 {
                    // Re-edit with same file name
                    ffname = nvim_excmds_curbuf_get_ffname() as *mut c_char;
                    sfname = nvim_excmds_curbuf_get_fname() as *mut c_char;
                }
                free_fname = nvim_fix_fname(ffname);
                if !free_fname.is_null() {
                    ffname = free_fname;
                }
                other_file = rs_otherfile(ffname);
            }
        }

        // ----------------------------------------------------------------
        // Re-editing a terminal buffer: skip most re-initialization
        // ----------------------------------------------------------------
        if !other_file && nvim_ecmd_curbuf_get_terminal() != 0 {
            check_arg_idx(nvim_get_curwin());
            nvim_maketitle();
            retval = OK;
            break 'outer;
        }

        // ----------------------------------------------------------------
        // Check if we can abandon the current buffer
        // ----------------------------------------------------------------
        let need_check = (!other_file && (flags & ECMD_OLDBUF) == 0)
            || (nvim_excmds_curbuf_get_b_nwindows() == 1
                && (flags & (ECMD_HIDE | ECMD_ADDBUF | ECMD_ALTBUF)) == 0);
        if need_check {
            let ccgd = if crate::p_awa != 0 { CCGD_AW } else { 0 }
                | if other_file { 0 } else { CCGD_MULTWIN }
                | if (flags & ECMD_FORCEIT) != 0 {
                    CCGD_FORCEIT
                } else {
                    0
                }
                | if !eap.is_null() { CCGD_EXCMD } else { 0 };
            if check_changed(nvim_get_curbuf(), ccgd) {
                if fnum == 0 && other_file && !ffname.is_null() {
                    setaltfname(ffname, sfname, if newlnum < 0 { 0 } else { newlnum });
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
        did_set_swapcommand = crate::write::rs_set_swapcommand(command, newlnum);

        // ================================================================
        // other_file: open or find the buffer for the other file
        // ================================================================
        if other_file {
            let prev_alt_fnum = nvim_ecmd_curwin_get_alt_fnum();

            if (flags & (ECMD_ADDBUF | ECMD_ALTBUF)) == 0 {
                if nvim_excmds_cmdmod_has_keepalt() == 0 {
                    nvim_excmds_set_curwin_alt_fnum(nvim_excmds_curbuf_get_b_fnum());
                }
                if !oldwin.is_null() {
                    rs_buflist_altfpos(oldwin);
                }
            }

            if fnum != 0 {
                buf = nvim_buflist_findnr(fnum);
            } else if (flags & (ECMD_ADDBUF | ECMD_ALTBUF)) != 0 {
                // Add buffer to list without editing
                let mut tlnum: c_int = 0;
                if !command.is_null() {
                    tlnum = c_atol(command) as c_int;
                    if tlnum <= 0 {
                        tlnum = 1;
                    }
                }
                let newbuf = buflist_new(ffname, sfname, tlnum, BLN_LISTED | BLN_NOCURWIN);
                if !newbuf.is_null() && (flags & ECMD_ALTBUF) != 0 {
                    nvim_excmds_set_curwin_alt_fnum(nvim_excmds_buf_get_b_fnum(newbuf));
                }
                break 'outer; // retval stays FAIL (goto theend)
            } else {
                buf = buflist_new(
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

            if nvim_excmds_buf_get_b_fnum(buf) == nvim_ecmd_curwin_get_alt_fnum()
                && prev_alt_fnum != 0
            {
                // Reusing buffer: keep old alternate file
                nvim_excmds_set_curwin_alt_fnum(prev_alt_fnum);
            }

            if nvim_ecmd_buf_has_memfile(buf) == 0 {
                oldbuf = 0;
            } else {
                oldbuf = 1;
                set_bufref(bufref, buf);
                nvim_ex2_buf_check_timestamp(buf);
                // Check if autocommands invalidated the buffer or changed curbuf
                if !bufref_valid(bufref) || nvim_ecmd_bufref_is_curbuf(old_curbuf) == 0 {
                    break 'outer;
                }
                if aborting() != 0 {
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
                let fname = nvim_excmds_buf_get_b_fname(buf);
                if !fname.is_null() {
                    new_name = xstrdup(fname);
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
                    rs_delbuf_msg(new_name); // frees new_name
                    new_name = std::ptr::null_mut();
                    nvim_ecmd_au_new_curbuf_restore(save_au);
                    xfree(save_au);
                    break 'outer;
                }
                if aborting() != 0 {
                    xfree(new_name as *mut std::ffi::c_void);
                    new_name = std::ptr::null_mut();
                    nvim_ecmd_au_new_curbuf_restore(save_au);
                    xfree(save_au);
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
                    if nvim_ecmd_bufref_is_curbuf(old_curbuf) != 0 {
                        buf_copy_options(buf, BCO_ENTER);
                    }

                    // Close the link to the current buffer
                    nvim_u_sync(false);
                    let close_flags =
                        if (flags & ECMD_HIDE) != 0 || nvim_ecmd_curbuf_get_terminal() != 0 {
                            0
                        } else {
                            DOBUF_UNLOAD
                        };
                    let did_decrement =
                        close_buffer(oldwin, nvim_get_curbuf(), close_flags, false, false) as c_int;

                    // Autocommands may have closed the window
                    if rs_win_valid(the_curwin) != 0 {
                        nvim_ecmd_win_set_locked(the_curwin, 0);
                    }
                    nvim_ecmd_buf_dec_locked(buf);

                    // Autocmds may abort script processing
                    if aborting() != 0 && nvim_ecmd_curwin_buf_is_null() == 0 {
                        xfree(new_name as *mut std::ffi::c_void);
                        new_name = std::ptr::null_mut();
                        nvim_ecmd_au_new_curbuf_restore(save_au);
                        xfree(save_au);
                        break 'outer;
                    }
                    if nvim_ecmd_au_new_curbuf_valid() == 0 {
                        rs_delbuf_msg(new_name);
                        new_name = std::ptr::null_mut();
                        nvim_ecmd_au_new_curbuf_restore(save_au);
                        xfree(save_au);
                        break 'outer;
                    }

                    if nvim_ecmd_buf_is_curbuf(buf) != 0 {
                        // Already in new buffer after close_buffer()
                        if did_decrement != 0 && buf_valid(was_curbuf) {
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
                            set_file_options(true, eap);
                            set_forced_fenc(eap);
                        }
                    }

                    // Get window options from last time buffer was in this window
                    get_winopts(nvim_get_curbuf());
                    did_get_winopts = true;
                }

                xfree(new_name as *mut std::ffi::c_void);
                new_name = std::ptr::null_mut();
                nvim_ecmd_au_new_curbuf_restore(save_au);
                xfree(save_au);
            }

            // Set pcmark to beginning of buffer
            nvim_ecmd_curwin_set_pcmark(1, 0);
        } else {
            // ============================================================
            // !other_file: re-editing the same file
            // ============================================================
            if (flags & (ECMD_ADDBUF | ECMD_ALTBUF)) != 0 || check_fname() == 0 {
                break 'outer; // retval stays FAIL
            }
            oldbuf = if (flags & ECMD_OLDBUF) != 0 { 1 } else { 0 };
        }

        // ================================================================
        // Disable redraw until cursor is in the right position
        // ================================================================
        nvim_inc_RedrawingDisabled();
        did_inc_redrawing_disabled = true;

        buf = nvim_get_curbuf();

        if (flags & ECMD_SET_HELP) != 0 || crate::keep_help_flag {
            prepare_help_buffer();
        } else if nvim_ecmd_curbuf_get_help() == 0 {
            set_buflisted(1);
        }

        // If autocommands changed buffers, give up
        if nvim_ecmd_buf_is_curbuf(buf) == 0 {
            break 'outer;
        }
        if aborting() != 0 {
            break 'outer;
        }

        // Consider filetype unset
        nvim_ecmd_curbuf_set_did_filetype(0);

        // re-use the buffer case: !other_file && !oldbuf
        if !other_file && oldbuf == 0 {
            set_last_cursor(nvim_get_curwin());
            if newlnum == ECMD_LAST || newlnum == ECMD_LASTL {
                newlnum = nvim_excmds_curwin_cursor_lnum();
                solcol = nvim_ecmd_curwin_get_cursor_col();
            }
            buf = nvim_get_curbuf();
            let fname = nvim_excmds_curbuf_get_fname();
            if !fname.is_null() {
                new_name = xstrdup(fname);
            }
            nvim_ecmd_set_bufref_to_curbuf(bufref);

            // Store current contents for undo if buffer was used before
            let line_count = nvim_excmds_curbuf_ml_line_count();
            let p_ur = crate::p_ur;
            if (nvim_ecmd_curbuf_get_b_flags() & BF_NEVERLOADED) == 0
                && (p_ur < 0 || line_count as i64 <= p_ur)
            {
                nvim_u_sync(false);
                // u_savecommon returns OK(1) on success; check == OK means success
                if u_savecommon(nvim_get_curbuf(), 0, line_count + 1, 0, true) != OK {
                    xfree(new_name as *mut std::ffi::c_void);
                    new_name = std::ptr::null_mut();
                    break 'outer; // retval stays FAIL
                }
                u_unchanged(nvim_get_curbuf());
                buf_freeall(nvim_get_curbuf(), BFA_KEEP_UNDO);
                readfile_flags = READ_KEEP_UNDO;
            } else {
                buf_freeall(nvim_get_curbuf(), 0);
            }

            // If autocommands deleted the buffer, give up
            if !bufref_valid(bufref) {
                rs_delbuf_msg(new_name);
                new_name = std::ptr::null_mut();
                break 'outer;
            }
            xfree(new_name as *mut std::ffi::c_void);
            new_name = std::ptr::null_mut();

            // If autocommands changed buffers, give up
            if nvim_ecmd_buf_is_curbuf(buf) == 0 {
                break 'outer;
            }
            if aborting() != 0 {
                break 'outer;
            }
            buf_clear_file(nvim_get_curbuf());
            nvim_ecmd_curbuf_clear_op_marks();
        }

        // Assume success from here
        retval = OK;

        // Reset not-edit flag so ":write" works
        if !other_file {
            nvim_ecmd_curbuf_clear_flags(BF_NOTEDITED);
        }

        // Check if editing the w_arg_idx file in the argument list
        check_arg_idx(nvim_get_curwin());

        if !auto_buf {
            // Init window before reading file / executing autocmds
            curwin_init();

            // Update automatic folding for all windows showing curbuf
            nvim_ecmd_fold_update_all_curbuf_wins();

            // Change directories when 'acd' is set
            do_autochdir();

            // Save cursor and topline for autocommand comparison
            let mut orig_lnum: c_int = 0;
            let mut orig_col: c_int = 0;
            nvim_ecmd_curwin_get_cursor(&mut orig_lnum, &mut orig_col);
            topline = nvim_ecmd_curwin_get_topline();

            if oldbuf == 0 {
                // Need to read the file
                nvim_set_swap_exists_action(SEA_DIALOG);
                nvim_ecmd_curbuf_set_flags(BF_CHECK_RO);

                if (flags & ECMD_NOWINENTER) != 0 {
                    readfile_flags |= READ_NOWINENTER;
                }
                // open_buffer result is run through should_abort; nonzero means error
                if should_abort(open_buffer(false, eap, readfile_flags)) != 0 {
                    retval = FAIL;
                }

                if nvim_get_swap_exists_action() == SEA_QUIT {
                    retval = FAIL;
                }
                nvim_ecmd_handle_swap_exists(old_curbuf);
            } else {
                // Read modelines (window-local options only)
                do_modelines(OPT_WINONLY);

                nvim_ecmd_apply_autocmds_bufenter_retval(&mut retval);
                if (flags & ECMD_NOWINENTER) == 0 {
                    nvim_ecmd_apply_autocmds_bufwinenter_retval(&mut retval);
                }
            }
            check_arg_idx(nvim_get_curwin());

            // If autocommands changed cursor position or topline, keep it.
            // But not if cursor moved to first non-blank.
            if nvim_ecmd_cursor_eq(orig_lnum, orig_col) == 0 {
                let cur_lnum = nvim_excmds_curwin_cursor_lnum();
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
            changed_line_abv_curs();

            nvim_maketitle();
        }

        // Tell diff about new/updated buffer
        // If window options changed, set spell language
        let mut p_diff: c_int = 0;
        let mut p_spell: c_int = 0;
        let mut spl_empty: c_int = 0;
        nvim_ecmd_curwin_diff_spell_state(&mut p_diff, &mut p_spell, &mut spl_empty);
        if p_diff != 0 {
            rs_diff_buf_add(nvim_get_curbuf());
            rs_diff_invalidate(nvim_get_curbuf());
        }
        if did_get_winopts && p_spell != 0 && spl_empty == 0 {
            parse_spelllang(nvim_get_curwin());
        }

        // Position cursor
        if command.is_null() {
            if newcol >= 0 {
                // Position set by autocommands
                nvim_ecmd_curwin_set_cursor(newlnum, newcol);
                nvim_check_cursor();
            } else if newlnum > 0 {
                // Line number from caller or old position
                nvim_ecmd_curwin_set_cursor(newlnum, nvim_ecmd_curwin_get_cursor_col());
                crate::nvim_check_cursor_lnum_call();
                if solcol >= 0 && nvim_get_p_sol() == 0 {
                    // 'sol' is off: use last known column
                    nvim_ecmd_curwin_set_cursor(nvim_excmds_curwin_cursor_lnum(), solcol);
                    check_cursor_col(nvim_get_curwin());
                    nvim_ecmd_curwin_set_coladd_curswant();
                } else {
                    nvim_beginline(BL_SOL | BL_FIX);
                }
            } else {
                // No line number -- go to last line in Ex mode
                if exmode_active {
                    nvim_ecmd_curwin_set_cursor(nvim_excmds_curbuf_ml_line_count(), 0);
                }
                nvim_beginline(BL_WHITE | BL_FIX);
            }
        }

        // Check cursors in other windows on the same buffer
        rs_check_lnums(0);

        // Show file info for old buffers that weren't re-read
        if oldbuf != 0 && !auto_buf {
            let msg_scroll_save = crate::msg_scroll;

            // 'O' flag in 'cpoptions': overwrite previous file message
            if shortmess(SHM_OVERALL)
                && crate::msg_listdo_overwrite == 0
                && !crate::exiting
                && nvim_get_p_verbose() == 0
            {
                msg_scroll = 0;
            }
            if crate::msg_scroll == 0 {
                msg_check_for_delay(false);
            }
            msg_start();
            msg_scroll = msg_scroll_save;
            crate::msg_scrolled_ign = true;

            if !shortmess(SHM_FILEINFO) {
                nvim_fileinfo_call();
            }

            crate::msg_scrolled_ign = false;
        }

        nvim_ecmd_curbuf_set_last_used();

        if !command.is_null() {
            // DOCMD_VERBOSE = 0x01
            do_cmdline(
                command as *mut c_char,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                1,
            );
        }

        if (nvim_ecmd_curbuf_get_kmap_state() & KEYMAP_INIT) != 0 {
            keymap_init();
        }

        nvim_dec_RedrawingDisabled();
        did_inc_redrawing_disabled = false;

        if !nvim_get_skip_redraw() {
            // Force cursor to center if no topline and no +cmd
            if topline == 0 && command.is_null() {
                nvim_ecmd_curwin_set_effective_p_so(999);
            }
            nvim_excmds_update_topline_curwin();
            nvim_ecmd_curwin_set_scbind_pos_from_topline();
            // Restore original scroll offset
            nvim_ecmd_curwin_set_effective_p_so(orig_p_so);
            redraw_curbuf_later(UPD_NOT_VALID);
        }

        do_autochdir();

        break 'outer; // Normal exit
    } // end 'outer

    // theend: cleanup
    nvim_ecmd_terminal_check_size_cleanup(old_curbuf);

    if did_inc_redrawing_disabled {
        nvim_dec_RedrawingDisabled();
    }

    if did_set_swapcommand {
        nvim_ecmd_clear_swapcommand();
    }

    if !free_fname.is_null() {
        xfree(free_fname as *mut std::ffi::c_void);
    }

    xfree(old_curbuf);
    xfree(bufref);

    retval
}
