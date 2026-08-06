use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::arglist::{check_arg_idx, do_argfile};
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::autocmd::{
    EVENT_BUFADD, EVENT_BUFENTER, EVENT_BUFFILEPOST, EVENT_BUFFILEPRE, EVENT_BUFLEAVE,
    EVENT_BUFWINENTER, EVENT_SHELLCMDPOST, EVENT_SHELLFILTERPOST, apply_autocmds,
    apply_autocmds_retval, augroup_exists, do_doautocmd,
};
use crate::src::nvim::buffer::{
    bt_dontwrite, bt_dontwrite_msg, bt_nofilename, buf_clear_file, buf_ensure_loaded, buf_freeall,
    buf_hide, buf_name_changed, buf_valid, buflist_altfpos, buflist_findfmark, buflist_findname,
    buflist_findnr, buflist_new, bufref_valid, close_buffer, do_autochdir, do_modelines, fileinfo,
    fname_expand, get_winopts, handle_swap_exists, maketitle, no_write_message,
    no_write_message_buf, open_buffer, otherfile, set_buflisted, set_bufref, setaltfname, setfname,
};
use crate::src::nvim::buffer_updates::buf_updates_send_changes;
use crate::src::nvim::bufwrite::{WriteRequest, buf_write};
use crate::src::nvim::change::{
    appended_lines, appended_lines_mark, changed_bytes, changed_lines, del_lines, deleted_lines,
    deleted_lines_mark,
};
use crate::src::nvim::channel::channel_job_running;
use crate::src::nvim::charset::{
    getdigits_int, skiptobin, skiptodigit, skiptohex, skiptowhite, skipwhite, transchar,
    transchar_nonprint, vim_isIDc, vim_isprintc, vim_str2nr,
};
use crate::src::nvim::cmdhist::add_to_history;
use crate::src::nvim::cursor::{
    check_cursor, check_cursor_col, check_cursor_lnum, check_pos, coladvance, get_cursor_line_ptr,
    get_cursor_pos_ptr,
};
use crate::src::nvim::decoration::bufhl_add_hl_pos_offset;
use crate::src::nvim::diff::{diff_buf_add, diff_invalidate};
use crate::src::nvim::digraph::{get_digraph_for_char, keymap_init};
use crate::src::nvim::drawscreen::{
    UPD_NOT_VALID, UPD_SOME_VALID, UPD_VALID, number_width, redraw_curbuf_later, redraw_later,
    show_cursor_info_later, update_screen,
};
use crate::src::nvim::edit::beginline;
use crate::src::nvim::eval::typval::{kCallbackNone, tv_list_len};
use crate::src::nvim::eval::typval::{tv_get_string, tv_list_find_str};
use crate::src::nvim::eval::vars::{get_vim_var_list, get_vim_var_str, set_vim_var_string};
use crate::src::nvim::ex_cmds2::{
    autowrite, autowrite_all, buf_write_all, check_changed, check_fname, dialog_changed,
};
use crate::src::nvim::ex_docmd::{
    before_quit_all, check_nextcmd, dialog_msg, do_cmdline, do_exedit, ex_may_print, not_exiting,
};
use crate::src::nvim::ex_eval::{aborting, should_abort};
use crate::src::nvim::ex_getln::{curbuf_locked, getcmdline_prompt, gotocmdline, text_locked};
use crate::src::nvim::extmark::{extmark_move_region, extmark_splice};
use crate::src::nvim::fileio::{
    buf_check_timestamp, readfile, set_file_options, set_forced_fenc, vim_tempname,
    write_lnum_adjust,
};
use crate::src::nvim::fold::{foldMoveRange, foldUpdate, foldUpdateAll, hasAnyFolding};
use crate::src::nvim::getchar::{AppendToRedobuff, AppendToRedobuffLit};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::help::prepare_help_buffer;
use crate::src::nvim::highlight_group::{HLF_N, HLF_R, syn_check_group};
use crate::src::nvim::indent::{get_indent, get_indent_lnum, set_indent};
use crate::src::nvim::input::prompt_for_input;
use crate::src::nvim::keycodes::{Ctrl_C, Ctrl_E, Ctrl_Y};
use crate::src::nvim::main::{
    Columns, IObuff, KeyTyped, RedrawingDisabled, Rows, State, VIsual, VIsual_active,
    au_new_curbuf, autocmd_busy, bangredo, cmdmod, cmdwin_buf, cmdwin_old_curwin, cmdwin_type,
    cmdwin_win, curbuf, curtab, curwin, did_check_timestamps, disable_fold_update, e_argreq,
    e_backslash, e_bufloaded, e_cannot_switch_to_a_closing_buffer, e_cant_read_file_str, e_curdir,
    e_exists, e_interr, e_invarg, e_invarg2, e_invcmd, e_isadir2, e_modifiable, e_nopresub,
    e_noprev, e_noprevre, e_notmp, e_patnotf2, e_readonly, e_sandbox, e_trailing_arg,
    e_val_too_large_len, e_zerocount, emsg_silent, ex_no_reprint, ex_normal_busy, exiting,
    exmode_active, first_tabpage, firstbuf, firstwin, g_do_tagpreview, getout, global_busy,
    got_int, highlight_match, info_message, keep_help_flag, lastwin, lines_left, msg_buf, msg_col,
    msg_didout, msg_listdo_overwrite, msg_row, msg_scroll, msg_scrolled, msg_scrolled_ign,
    msg_silent, need_check_timestamps, need_wait_return, no_u_sync, no_wait_return, p_awa, p_ch,
    p_confirm, p_cpo, p_cwh, p_dir, p_gd, p_ic, p_icm, p_lz, p_rdt, p_report, p_sh, p_shm, p_shq,
    p_so, p_sol, p_srr, p_stmp, p_ur, p_verbose, p_wa, p_warn, p_window, p_write, quit_more,
    redraw_tabline, sandbox, search_match_endcol, search_match_lines, secure, silent_mode,
    skip_redraw, sub_nlines, sub_nsubs, swap_exists_action, textlock,
};
use crate::src::nvim::mark::{mark_adjust, mark_adjust_nofold, set_last_cursor, setpcmark};
use crate::src::nvim::mbyte::{
    utf_char2bytes, utf_iscomposing_first, utf_ptr2char, utf_ptr2len, utfc_ptr2len,
};
use crate::src::nvim::memline::{
    makeswapname, ml_append, ml_append_buf, ml_clearmarked, ml_delete, ml_delete_flags,
    ml_find_line_or_offset, ml_firstmarked, ml_get, ml_get_buf, ml_get_buf_len, ml_get_len,
    ml_replace, ml_replace_buf, ml_setmarked,
};
use crate::src::nvim::memory::{
    xcalloc, xfree, xmalloc, xmallocz, xmemdupz, xrealloc, xstrdup, xstrlcat, xstrlcpy,
};
use crate::src::nvim::message::{
    emsg, message_filtered, messaging, msg, msg_check_for_delay, msg_clr_eos, msg_end,
    msg_ext_set_kind, msg_multiline, msg_outnum, msg_outtrans, msg_prt_line, msg_putchar, msg_puts,
    msg_puts_hl, msg_sb_eol, msg_start, msg_starthere, msgmore, semsg, set_keep_msg, smsg,
    vim_dialog_yesno, wait_return,
};
use crate::src::nvim::mouse::setmouse;
use crate::src::nvim::r#move::{
    changed_line_abv_curs, changed_window_setting, do_check_cursorbind, invalidate_botline_win,
    scrolldown_clamp, scrollup_clamp, update_topline, validate_cursor,
};
use crate::src::nvim::normal::reset_VIsual;
use crate::src::nvim::ops::do_join;
use crate::src::nvim::option::{
    buf_copy_options, copy_option_part, get_fileformat, magic_isset, set_option_direct, shortmess,
};
use crate::src::nvim::options::{kOptFoldcolumn, kOptInccommand, kOptShortmess};
use crate::src::nvim::os::env::expand_env_save;
use crate::src::nvim::os::fs::{
    os_file_is_writable, os_file_mkdir, os_isdir, os_nodetype, os_path_exists, os_remove,
};
use crate::src::nvim::os::input::{fast_breakcheck, line_breakcheck, os_breakcheck};
use crate::src::nvim::os::libc::{
    __assert_fail, __ctype_b_loc, atoi, atol, gettext, log10, memcpy, memmove, memset, ngettext,
    qsort, snprintf, strcasecmp, strcat, strchr, strcmp, strcoll, strcpy, strlen, strncmp, strtod,
    time,
};
use crate::src::nvim::os::shell::call_shell;
use crate::src::nvim::os::time::os_time;
use crate::src::nvim::path::{fix_fname, invocation_path_tail};
use crate::src::nvim::plines::linetabsize_str;
use crate::src::nvim::plines::{getvcol, plines_m_win_fill};
use crate::src::nvim::pos::{MAXCOL, MAXLNUM, equalpos};
use crate::src::nvim::profile::{profile_passed_limit, profile_setlimit, profile_zero};
use crate::src::nvim::regexp::{
    RE_BOTH, RE_LAST, RE_MAGIC, RE_SEARCH, RE_SUBST, vim_regcomp, vim_regexec, vim_regexec_multi,
    vim_regfree,
};
use crate::src::nvim::regexp::{
    regtilde, skip_regexp, skip_regexp_err, skip_regexp_ex, vim_regsub_multi,
};
use crate::src::nvim::search::{
    SEARCH_HIS, get_search_pat, last_search_pat, save_re_pat, search_regcomp,
};
use crate::src::nvim::spell::parse_spelllang;
use crate::src::nvim::state::{MODE_CMDLINE, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL};
use crate::src::nvim::strings::{
    concat_str, vim_snprintf, vim_snprintf_add, vim_snprintf_safelen, vim_strchr,
    vim_strsave_escaped, xstrnsave,
};
use crate::src::nvim::terminal::{terminal_check_size, terminal_running};
use crate::src::nvim::types::ui::kUIMessages;
use crate::src::nvim::types::{
    AdditionalData, CMD_append, CMD_center, CMD_change, CMD_edit, CMD_left, CMD_right, CMD_saveas,
    CMD_tilde, CMD_wqall, CMD_xall, CMOD_BROWSE, CMOD_CONFIRM, CMOD_KEEPALT, CMOD_KEEPMARKS,
    CMOD_KEEPPATTERNS, CMOD_LOCKMARKS, Callback, Callback_data as C2Rust_Unnamed_5, ExtmarkOp,
    OptInt, OptVal, OptValData, OptValType, String_0, SubReplacementString, Timestamp,
    UndoObjectType, VV_OLDFILES, VV_SWAPCOMMAND, bcount_t, bfa_values, bln_values, buf_T, bufref_T,
    colnr_T, dobuf_action_values, exarg_T, float_T, fmark_T, getf_retvalues, handle_T, int32_t,
    int64_t, linenr_T, list_T, listitem_T, lpos_T, magic_T, pos_T, proftime_T, ptrdiff_t,
    regmatch_T, regmmatch_T, regprog_T, size_t, tabpage_T, time_t, uint8_t, uint64_t, uvarnumber_T,
    varnumber_T, win_T,
};
use crate::src::nvim::ui::{ui_cursor_goto, ui_cursor_shape, ui_has};
use crate::src::nvim::undo::{
    bufIsChanged, curbufIsChanged, u_inssub, u_save, u_save_cursor, u_savecommon, u_savedel,
    u_savesub, u_sync, u_unchanged,
};
use crate::src::nvim::window::{
    check_can_set_curbuf_forceit, check_lnums, curwin_init, win_enter, win_split, win_valid,
    win_valid_any_tab,
};
unsafe extern "C" {
    fn re_multiline(prog: *const regprog_T) -> ::core::ffi::c_int;
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISalpha: C2Rust_Unnamed = 1024;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_int;
pub const EXPAND_NOTHING: C2Rust_Unnamed_17 = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const REGSUB_BACKSLASH: C2Rust_Unnamed_18 = 4;
pub const REGSUB_MAGIC: C2Rust_Unnamed_18 = 2;
pub const REGSUB_COPY: C2Rust_Unnamed_18 = 1;
pub const kOptValTypeString: OptValType = 2;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const GETFILE_OPEN_OTHER: getf_retvalues = -1;
pub const GETFILE_SAME_FILE: getf_retvalues = 0;
pub const GETFILE_NOT_WRITTEN: getf_retvalues = 2;
pub const GETFILE_ERROR: getf_retvalues = 1;
pub const BLN_NOCURWIN: bln_values = 128;
pub const BLN_LISTED: bln_values = 2;
pub const BLN_CURBUF: bln_values = 1;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const BFA_KEEP_UNDO: bfa_values = 4;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const SHM_FILEINFO: C2Rust_Unnamed_21 = 70;
pub const SHM_OVERALL: C2Rust_Unnamed_21 = 79;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const STR2NR_FORCE: C2Rust_Unnamed_22 = 128;
pub const STR2NR_HEX: C2Rust_Unnamed_22 = 4;
pub const STR2NR_OCT: C2Rust_Unnamed_22 = 2;
pub const STR2NR_BIN: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_int;
pub const HIST_SEARCH: C2Rust_Unnamed_23 = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_25 = 4;
pub const BL_SOL: C2Rust_Unnamed_25 = 2;
pub const BL_WHITE: C2Rust_Unnamed_25 = 1;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const VIM_QUESTION: C2Rust_Unnamed_26 = 4;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const VIM_YES: C2Rust_Unnamed_27 = 2;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const ECMD_NOWINENTER: C2Rust_Unnamed_28 = 64;
pub const ECMD_ALTBUF: C2Rust_Unnamed_28 = 32;
pub const ECMD_ADDBUF: C2Rust_Unnamed_28 = 16;
pub const ECMD_FORCEIT: C2Rust_Unnamed_28 = 8;
pub const ECMD_OLDBUF: C2Rust_Unnamed_28 = 4;
pub const ECMD_SET_HELP: C2Rust_Unnamed_28 = 2;
pub const ECMD_HIDE: C2Rust_Unnamed_28 = 1;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_int;
pub const ECMD_ONE: C2Rust_Unnamed_29 = 1;
pub const ECMD_LAST: C2Rust_Unnamed_29 = -1;
pub const ECMD_LASTL: C2Rust_Unnamed_29 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sorti_T {
    pub lnum: linenr_T,
    pub st_u: C2Rust_Unnamed_30,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_30 {
    pub line: C2Rust_Unnamed_32,
    pub num: C2Rust_Unnamed_31,
    pub value_flt: float_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_31 {
    pub value: varnumber_T,
    pub is_number: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_32 {
    pub start_col_nr: varnumber_T,
    pub end_col_nr: varnumber_T,
}
pub const ML_DEL_MESSAGE: C2Rust_Unnamed_39 = 1;
pub const kShellOptRead: C2Rust_Unnamed_42 = 16;
pub const READ_FILTER: C2Rust_Unnamed_37 = 2;
pub const kShellOptFilter: C2Rust_Unnamed_42 = 1;
pub const kShellOptWrite: C2Rust_Unnamed_42 = 32;
pub const kShellOptDoOut: C2Rust_Unnamed_42 = 4;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_36 = 1;
pub const OPT_WINONLY: C2Rust_Unnamed_41 = 8;
pub const READ_NOWINENTER: C2Rust_Unnamed_37 = 128;
pub const READ_KEEP_UNDO: C2Rust_Unnamed_37 = 32;
pub const BCO_ENTER: C2Rust_Unnamed_40 = 1;
pub const CCGD_EXCMD: C2Rust_Unnamed_35 = 16;
pub const CCGD_FORCEIT: C2Rust_Unnamed_35 = 4;
pub const CCGD_MULTWIN: C2Rust_Unnamed_35 = 2;
pub const CCGD_AW: C2Rust_Unnamed_35 = 1;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_36 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SubResult {
    pub start: lpos_T,
    pub end: lpos_T,
    pub pre_match: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_33 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut SubResult,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PreviewLines {
    pub subresults: C2Rust_Unnamed_33,
    pub lines_needed: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct subflags_T {
    pub do_all: bool,
    pub do_ask: bool,
    pub do_count: bool,
    pub do_error: bool,
    pub do_print: bool,
    pub do_list: bool,
    pub do_number: bool,
    pub do_ic: SubIgnoreType,
}
pub type SubIgnoreType = ::core::ffi::c_uint;
pub const kSubMatchCase: SubIgnoreType = 2;
pub const kSubIgnoreCase: SubIgnoreType = 1;
pub const kSubHonorOptions: SubIgnoreType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LineData {
    pub start_col: ::core::ffi::c_int,
    pub start: lpos_T,
    pub end: lpos_T,
    pub matchcols: ::core::ffi::c_int,
    pub matchbytes: bcount_t,
    pub subcols: ::core::ffi::c_int,
    pub subbytes: bcount_t,
    pub lnum_before: linenr_T,
    pub lnum_after: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_34 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut LineData,
}
pub const VGR_FUZZY: C2Rust_Unnamed_43 = 4;
pub const VGR_NOJUMP: C2Rust_Unnamed_43 = 2;
pub const VGR_GLOBAL: C2Rust_Unnamed_43 = 1;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_37 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_39 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_40 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_41 = ::core::ffi::c_uint;
pub const OPT_LOCAL: C2Rust_Unnamed_41 = 2;
pub type C2Rust_Unnamed_42 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_43 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const BF_CHECK_RO: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const BF_NEVERLOADED: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const BF_NOTEDITED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const BF_NEW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const BF_READERR: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const B_IMODE_LMAP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYMAP_INIT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NODE_OTHER: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EXFLAG_LIST: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const EXFLAG_NR: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const EXFLAG_PRINT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const EOL_MAC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CPO_ALTWRITE: ::core::ffi::c_int = 'A' as ::core::ffi::c_int;
pub const CPO_OVERNEW: ::core::ffi::c_int = 'O' as ::core::ffi::c_int;
pub const CPO_REMMARK: ::core::ffi::c_int = 'R' as ::core::ffi::c_int;
pub const CPO_UNDO: ::core::ffi::c_int = 'u' as ::core::ffi::c_int;
static e_non_numeric_argument_to_z: GlobalCell<[::core::ffi::c_char; 33]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
            *b"E144: Non-numeric argument to :z\0",
        )
    });
pub unsafe fn do_ascii(mut _eap: *mut exarg_T) {
    let mut data: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
    let mut len: size_t = utfc_ptr2len(data) as size_t;
    if len == 0 as size_t {
        msg(
            b"NUL\0".as_ptr() as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
        return;
    }
    let mut need_clear: bool = true_0 != 0;
    msg_sb_eol();
    msg_start();
    let mut c: ::core::ffi::c_int = utf_ptr2char(data);
    let mut off: size_t = 0 as size_t;
    if c < 0x80 as ::core::ffi::c_int {
        if c == NL {
            c = NUL;
        }
        let cval: ::core::ffi::c_int = if c == CAR && get_fileformat(curbuf.get()) == EOL_MAC {
            NL
        } else {
            c
        };
        let mut buf1: [::core::ffi::c_char; 20] = [0; 20];
        if vim_isprintc(c) as ::core::ffi::c_int != 0
            && (c < ' ' as ::core::ffi::c_int || c > '~' as ::core::ffi::c_int)
        {
            let mut buf3: [::core::ffi::c_char; 7] = [0; 7];
            transchar_nonprint(curbuf.get(), &raw mut buf3 as *mut ::core::ffi::c_char, c);
            vim_snprintf(
                &raw mut buf1 as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
                b"  <%s>\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut buf3 as *mut ::core::ffi::c_char,
            );
        } else {
            buf1[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        }
        let mut buf2: [::core::ffi::c_char; 20] = [0; 20];
        buf2[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        let dig = get_digraph_for_char(cval);
        if let Some(dig) = &dig {
            vim_snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
                gettext(b"<%s>%s%s  %d,  Hex %02x,  Oct %03o, Digr %s\0".as_ptr()
                    as *const ::core::ffi::c_char),
                transchar(c),
                &raw mut buf1 as *mut ::core::ffi::c_char,
                &raw mut buf2 as *mut ::core::ffi::c_char,
                cval,
                cval,
                cval,
                dig.as_ptr(),
            );
        } else {
            vim_snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
                gettext(b"<%s>%s%s  %d,  Hex %02x,  Octal %03o\0".as_ptr()
                    as *const ::core::ffi::c_char),
                transchar(c),
                &raw mut buf1 as *mut ::core::ffi::c_char,
                &raw mut buf2 as *mut ::core::ffi::c_char,
                cval,
                cval,
                cval,
            );
        }
        msg_multiline(
            cstr_as_string(IObuff.ptr() as *mut ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
            true_0 != 0,
            false_0 != 0,
            &raw mut need_clear,
        );
        off = off.wrapping_add(utf_ptr2len(data) as size_t);
    }
    while off < len {
        c = utf_ptr2char(data.offset(off as isize));
        let mut iobuff_len: size_t = 0 as size_t;
        if off > 0 as size_t {
            let c2rust_fresh0 = iobuff_len;
            iobuff_len = iobuff_len.wrapping_add(1);
            (*IObuff.ptr())[c2rust_fresh0 as usize] = ' ' as ::core::ffi::c_char;
        }
        let c2rust_fresh1 = iobuff_len;
        iobuff_len = iobuff_len.wrapping_add(1);
        (*IObuff.ptr())[c2rust_fresh1 as usize] = '<' as ::core::ffi::c_char;
        if utf_iscomposing_first(c) {
            let c2rust_fresh2 = iobuff_len;
            iobuff_len = iobuff_len.wrapping_add(1);
            (*IObuff.ptr())[c2rust_fresh2 as usize] = ' ' as ::core::ffi::c_char;
        }
        iobuff_len = iobuff_len.wrapping_add(utf_char2bytes(
            c,
            (IObuff.ptr() as *mut ::core::ffi::c_char).offset(iobuff_len as isize),
        ) as size_t);
        let dig_0 = get_digraph_for_char(c);
        if let Some(dig_0) = &dig_0 {
            vim_snprintf(
                (IObuff.ptr() as *mut ::core::ffi::c_char).offset(iobuff_len as isize),
                ::core::mem::size_of::<[::core::ffi::c_char; 1025]>().wrapping_sub(iobuff_len),
                if c < 0x10000 as ::core::ffi::c_int {
                    gettext(
                        b"> %d, Hex %04x, Oct %o, Digr %s\0".as_ptr() as *const ::core::ffi::c_char
                    )
                } else {
                    gettext(
                        b"> %d, Hex %08x, Oct %o, Digr %s\0".as_ptr() as *const ::core::ffi::c_char
                    )
                },
                c,
                c,
                c,
                dig_0.as_ptr(),
            );
        } else {
            vim_snprintf(
                (IObuff.ptr() as *mut ::core::ffi::c_char).offset(iobuff_len as isize),
                ::core::mem::size_of::<[::core::ffi::c_char; 1025]>().wrapping_sub(iobuff_len),
                if c < 0x10000 as ::core::ffi::c_int {
                    gettext(b"> %d, Hex %04x, Octal %o\0".as_ptr() as *const ::core::ffi::c_char)
                } else {
                    gettext(b"> %d, Hex %08x, Octal %o\0".as_ptr() as *const ::core::ffi::c_char)
                },
                c,
                c,
                c,
            );
        }
        msg_multiline(
            cstr_as_string(IObuff.ptr() as *mut ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
            true_0 != 0,
            false_0 != 0,
            &raw mut need_clear,
        );
        off = off.wrapping_add(utf_ptr2len(data.offset(off as isize)) as size_t);
    }
    if need_clear {
        msg_clr_eos();
    }
    msg_end();
}
pub unsafe fn ex_align(mut eap: *mut exarg_T) {
    let mut indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut new_indent: ::core::ffi::c_int = 0;
    if (*curwin.get()).w_onebuf_opt.wo_rl != 0 {
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_right as ::core::ffi::c_int {
            (*eap).cmdidx = CMD_left;
        } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_left as ::core::ffi::c_int {
            (*eap).cmdidx = CMD_right;
        }
    }
    let mut width: ::core::ffi::c_int = atoi((*eap).arg);
    let mut save_curpos: pos_T = (*curwin.get()).w_cursor;
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_left as ::core::ffi::c_int {
        if width >= 0 as ::core::ffi::c_int {
            indent = width;
        }
    } else {
        if width <= 0 as ::core::ffi::c_int {
            width = (*curbuf.get()).b_p_tw as ::core::ffi::c_int;
        }
        if width == 0 as ::core::ffi::c_int && (*curbuf.get()).b_p_wm > 0 as OptInt {
            width = (*curwin.get()).w_view_width - (*curbuf.get()).b_p_wm as ::core::ffi::c_int;
        }
        if width <= 0 as ::core::ffi::c_int {
            width = 80 as ::core::ffi::c_int;
        }
    }
    if u_save((*eap).line1 - 1 as linenr_T, (*eap).line2 + 1 as linenr_T) == FAIL {
        return;
    }
    (*curwin.get()).w_cursor.lnum = (*eap).line1;
    while (*curwin.get()).w_cursor.lnum <= (*eap).line2 {
        's_118: {
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_left as ::core::ffi::c_int {
                new_indent = indent;
            } else {
                let mut has_tab: ::core::ffi::c_int = false_0;
                let mut len: ::core::ffi::c_int = linelen(
                    if (*eap).cmdidx as ::core::ffi::c_int == CMD_right as ::core::ffi::c_int {
                        &raw mut has_tab
                    } else {
                        ::core::ptr::null_mut::<::core::ffi::c_int>()
                    },
                ) - get_indent();
                if len <= 0 as ::core::ffi::c_int {
                    break 's_118;
                } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_center as ::core::ffi::c_int {
                    new_indent = (width - len) / 2 as ::core::ffi::c_int;
                } else {
                    new_indent = width - len;
                    if has_tab != 0 {
                        while new_indent > 0 as ::core::ffi::c_int {
                            set_indent(new_indent, 0 as ::core::ffi::c_int);
                            if linelen(::core::ptr::null_mut::<::core::ffi::c_int>()) <= width {
                                loop {
                                    new_indent += 1;
                                    set_indent(new_indent, 0 as ::core::ffi::c_int);
                                    if linelen(::core::ptr::null_mut::<::core::ffi::c_int>())
                                        > width
                                    {
                                        break;
                                    }
                                }
                                new_indent -= 1;
                                break;
                            } else {
                                new_indent -= 1;
                            }
                        }
                    }
                }
            }
            new_indent = if new_indent > 0 as ::core::ffi::c_int {
                new_indent
            } else {
                0 as ::core::ffi::c_int
            };
            set_indent(new_indent, 0 as ::core::ffi::c_int);
        }
        (*curwin.get()).w_cursor.lnum += 1;
    }
    changed_lines(
        curbuf.get(),
        (*eap).line1,
        0 as colnr_T,
        (*eap).line2 + 1 as linenr_T,
        0 as linenr_T,
        true_0 != 0,
    );
    (*curwin.get()).w_cursor = save_curpos;
    beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
}
unsafe extern "C" fn linelen(mut has_tab: *mut ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut last: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    if *line as ::core::ffi::c_int == NUL {
        return 0 as ::core::ffi::c_int;
    }
    let mut first: *mut ::core::ffi::c_char = skipwhite(line);
    last = first.offset(strlen(first) as isize);
    while last > first
        && ascii_iswhite(*last.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
    {
        last = last.offset(-1);
    }
    let mut save: ::core::ffi::c_char = *last;
    *last = NUL as ::core::ffi::c_char;
    let mut len: ::core::ffi::c_int = linetabsize_str(line);
    if !has_tab.is_null() {
        *has_tab = !vim_strchr(first, TAB).is_null() as ::core::ffi::c_int;
    }
    *last = save;
    return len;
}
static sortbuf1: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static sortbuf2: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static sort_lc: GlobalCell<bool> = GlobalCell::new(false);
static sort_ic: GlobalCell<bool> = GlobalCell::new(false);
static sort_nr: GlobalCell<bool> = GlobalCell::new(false);
static sort_rx: GlobalCell<bool> = GlobalCell::new(false);
static sort_flt: GlobalCell<bool> = GlobalCell::new(false);
static sort_abort: GlobalCell<bool> = GlobalCell::new(false);
unsafe extern "C" fn string_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    if sort_lc.get() {
        return strcoll(
            s1 as *const ::core::ffi::c_char,
            s2 as *const ::core::ffi::c_char,
        );
    }
    return if sort_ic.get() as ::core::ffi::c_int != 0 {
        strcasecmp(
            s1 as *mut ::core::ffi::c_char,
            s2 as *mut ::core::ffi::c_char,
        )
    } else {
        strcmp(
            s1 as *const ::core::ffi::c_char,
            s2 as *const ::core::ffi::c_char,
        )
    };
}
unsafe extern "C" fn sort_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut l1: sorti_T = *(s1 as *mut sorti_T);
    let mut l2: sorti_T = *(s2 as *mut sorti_T);
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if sort_abort.get() {
        return 0 as ::core::ffi::c_int;
    }
    fast_breakcheck();
    if got_int.get() {
        sort_abort.set(true_0 != 0);
    }
    if sort_nr.get() {
        if l1.st_u.num.is_number as ::core::ffi::c_int
            != l2.st_u.num.is_number as ::core::ffi::c_int
        {
            result = if l1.st_u.num.is_number as ::core::ffi::c_int
                > l2.st_u.num.is_number as ::core::ffi::c_int
            {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
        } else {
            result = if l1.st_u.num.value == l2.st_u.num.value {
                0 as ::core::ffi::c_int
            } else if l1.st_u.num.value > l2.st_u.num.value {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
        }
    } else if sort_flt.get() {
        result = if l1.st_u.value_flt == l2.st_u.value_flt {
            0 as ::core::ffi::c_int
        } else if l1.st_u.value_flt > l2.st_u.value_flt {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    } else {
        memcpy(
            sortbuf1.get() as *mut ::core::ffi::c_void,
            ml_get(l1.lnum).offset(l1.st_u.line.start_col_nr as isize)
                as *const ::core::ffi::c_void,
            (l1.st_u.line.end_col_nr - l1.st_u.line.start_col_nr + 1 as varnumber_T) as size_t,
        );
        *(*sortbuf1.ptr()).offset((l1.st_u.line.end_col_nr - l1.st_u.line.start_col_nr) as isize) =
            NUL as ::core::ffi::c_char;
        memcpy(
            sortbuf2.get() as *mut ::core::ffi::c_void,
            ml_get(l2.lnum).offset(l2.st_u.line.start_col_nr as isize)
                as *const ::core::ffi::c_void,
            (l2.st_u.line.end_col_nr - l2.st_u.line.start_col_nr + 1 as varnumber_T) as size_t,
        );
        *(*sortbuf2.ptr()).offset((l2.st_u.line.end_col_nr - l2.st_u.line.start_col_nr) as isize) =
            NUL as ::core::ffi::c_char;
        result = string_compare(
            sortbuf1.get() as *const ::core::ffi::c_void,
            sortbuf2.get() as *const ::core::ffi::c_void,
        );
    }
    if result == 0 as ::core::ffi::c_int {
        return l1.lnum as ::core::ffi::c_int - l2.lnum as ::core::ffi::c_int;
    }
    return result;
}
pub unsafe fn ex_sort(mut eap: *mut exarg_T) {
    let mut old_count: bcount_t = 0;
    let mut new_count: bcount_t = 0;
    let mut lnum_0: linenr_T = 0;
    let mut deleted: linenr_T = 0;
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut maxlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut count: size_t = (((*eap).line2 - (*eap).line1) as size_t).wrapping_add(1 as size_t);
    let mut i: size_t = 0;
    let mut unique: bool = false_0 != 0;
    let mut sort_what: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if count <= 1 as size_t {
        return;
    }
    if u_save((*eap).line1 - 1 as linenr_T, (*eap).line2 + 1 as linenr_T) == FAIL {
        return;
    }
    sortbuf1.set(::core::ptr::null_mut::<::core::ffi::c_char>());
    sortbuf2.set(::core::ptr::null_mut::<::core::ffi::c_char>());
    regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
    let mut nrs: *mut sorti_T =
        xmalloc(count.wrapping_mul(::core::mem::size_of::<sorti_T>())) as *mut sorti_T;
    sort_flt.set(false_0 != 0);
    sort_nr.set(sort_flt.get());
    sort_rx.set(sort_nr.get());
    sort_lc.set(sort_rx.get());
    sort_ic.set(sort_lc.get());
    sort_abort.set(sort_ic.get());
    let mut format_found: size_t = 0 as size_t;
    let mut change_occurred: bool = false_0 != 0;
    let mut p: *mut ::core::ffi::c_char = (*eap).arg;
    '_sortend: {
        while *p as ::core::ffi::c_int != NUL {
            if !ascii_iswhite(*p as ::core::ffi::c_int) {
                if *p as ::core::ffi::c_int == 'i' as ::core::ffi::c_int {
                    sort_ic.set(true_0 != 0);
                } else if *p as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
                    sort_lc.set(true_0 != 0);
                } else if *p as ::core::ffi::c_int == 'r' as ::core::ffi::c_int {
                    sort_rx.set(true_0 != 0);
                } else if *p as ::core::ffi::c_int == 'n' as ::core::ffi::c_int {
                    sort_nr.set(true_0 != 0);
                    format_found = format_found.wrapping_add(1);
                } else if *p as ::core::ffi::c_int == 'f' as ::core::ffi::c_int {
                    sort_flt.set(true_0 != 0);
                    format_found = format_found.wrapping_add(1);
                } else if *p as ::core::ffi::c_int == 'b' as ::core::ffi::c_int {
                    sort_what =
                        STR2NR_BIN as ::core::ffi::c_int + STR2NR_FORCE as ::core::ffi::c_int;
                    format_found = format_found.wrapping_add(1);
                } else if *p as ::core::ffi::c_int == 'o' as ::core::ffi::c_int {
                    sort_what =
                        STR2NR_OCT as ::core::ffi::c_int + STR2NR_FORCE as ::core::ffi::c_int;
                    format_found = format_found.wrapping_add(1);
                } else if *p as ::core::ffi::c_int == 'x' as ::core::ffi::c_int {
                    sort_what =
                        STR2NR_HEX as ::core::ffi::c_int + STR2NR_FORCE as ::core::ffi::c_int;
                    format_found = format_found.wrapping_add(1);
                } else if *p as ::core::ffi::c_int == 'u' as ::core::ffi::c_int {
                    unique = true_0 != 0;
                } else if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
                    break;
                } else if !check_nextcmd(p).is_null() {
                    (*eap).nextcmd = check_nextcmd(p);
                    break;
                } else if !(*p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                    && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                    || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                        && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
                    && regmatch.regprog.is_null()
                {
                    let mut s: *mut ::core::ffi::c_char = skip_regexp_err(
                        p.offset(1 as ::core::ffi::c_int as isize),
                        *p as ::core::ffi::c_int,
                        true_0,
                    );
                    if s.is_null() {
                        break '_sortend;
                    }
                    *s = NUL as ::core::ffi::c_char;
                    if s == p.offset(1 as ::core::ffi::c_int as isize) {
                        if last_search_pat().is_null() {
                            emsg(gettext(&raw const e_noprevre as *const ::core::ffi::c_char));
                            break '_sortend;
                        } else {
                            regmatch.regprog = vim_regcomp(last_search_pat(), RE_MAGIC);
                        }
                    } else {
                        regmatch.regprog =
                            vim_regcomp(p.offset(1 as ::core::ffi::c_int as isize), RE_MAGIC);
                    }
                    if regmatch.regprog.is_null() {
                        break '_sortend;
                    }
                    p = s;
                    regmatch.rm_ic = p_ic.get() != 0;
                } else {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        p,
                    );
                    break '_sortend;
                }
            }
            p = p.offset(1);
        }
        if format_found > 1 as size_t {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        } else {
            sort_nr.set(sort_nr.get() as ::core::ffi::c_int | sort_what != 0);
            let mut lnum: linenr_T = (*eap).line1;
            while lnum <= (*eap).line2 {
                let mut s_0: *mut ::core::ffi::c_char = ml_get(lnum);
                let mut len: ::core::ffi::c_int = ml_get_len(lnum);
                maxlen = if maxlen > len { maxlen } else { len };
                let mut start_col: colnr_T = 0 as colnr_T;
                let mut end_col: colnr_T = len as colnr_T;
                if !regmatch.regprog.is_null()
                    && vim_regexec(&raw mut regmatch, s_0, 0 as colnr_T) as ::core::ffi::c_int != 0
                {
                    if sort_rx.get() {
                        start_col = regmatch.startp[0 as ::core::ffi::c_int as usize]
                            .offset_from(s_0) as colnr_T;
                        end_col = regmatch.endp[0 as ::core::ffi::c_int as usize].offset_from(s_0)
                            as colnr_T;
                    } else {
                        start_col = regmatch.endp[0 as ::core::ffi::c_int as usize].offset_from(s_0)
                            as colnr_T;
                    }
                } else if !regmatch.regprog.is_null() {
                    end_col = 0 as ::core::ffi::c_int as colnr_T;
                }
                if sort_nr.get() as ::core::ffi::c_int != 0
                    || sort_flt.get() as ::core::ffi::c_int != 0
                {
                    let mut s2: *mut ::core::ffi::c_char = s_0.offset(end_col as isize);
                    let mut c: ::core::ffi::c_char = *s2;
                    *s2 = NUL as ::core::ffi::c_char;
                    let mut p_0: *mut ::core::ffi::c_char = s_0.offset(start_col as isize);
                    if sort_nr.get() {
                        if sort_what & STR2NR_HEX as ::core::ffi::c_int != 0 {
                            s_0 = skiptohex(p_0);
                        } else if sort_what & STR2NR_BIN as ::core::ffi::c_int != 0 {
                            s_0 = skiptobin(p_0) as *mut ::core::ffi::c_char;
                        } else {
                            s_0 = skiptodigit(p_0);
                        }
                        if s_0 > p_0
                            && *s_0.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '-' as ::core::ffi::c_int
                        {
                            s_0 = s_0.offset(-1);
                        }
                        if *s_0 as ::core::ffi::c_int == NUL {
                            (*nrs.offset((lnum - (*eap).line1) as isize))
                                .st_u
                                .num
                                .is_number = false_0 != 0;
                            (*nrs.offset((lnum - (*eap).line1) as isize)).st_u.num.value =
                                0 as varnumber_T;
                        } else {
                            (*nrs.offset((lnum - (*eap).line1) as isize))
                                .st_u
                                .num
                                .is_number = true_0 != 0;
                            vim_str2nr(
                                s_0,
                                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                sort_what,
                                &raw mut (*nrs.offset((lnum - (*eap).line1) as isize))
                                    .st_u
                                    .num
                                    .value,
                                ::core::ptr::null_mut::<uvarnumber_T>(),
                                0 as ::core::ffi::c_int,
                                false_0 != 0,
                                ::core::ptr::null_mut::<bool>(),
                            );
                        }
                    } else {
                        s_0 = skipwhite(p_0);
                        if *s_0 as ::core::ffi::c_int == '+' as ::core::ffi::c_int {
                            s_0 = skipwhite(s_0.offset(1 as ::core::ffi::c_int as isize));
                        }
                        if *s_0 as ::core::ffi::c_int == NUL {
                            (*nrs.offset((lnum - (*eap).line1) as isize)).st_u.value_flt =
                                -DBL_MAX as float_T;
                        } else {
                            (*nrs.offset((lnum - (*eap).line1) as isize)).st_u.value_flt =
                                strtod(s_0, ::core::ptr::null_mut::<*mut ::core::ffi::c_char>())
                                    as float_T;
                        }
                    }
                    *s2 = c;
                } else {
                    (*nrs.offset((lnum - (*eap).line1) as isize))
                        .st_u
                        .line
                        .start_col_nr = start_col as varnumber_T;
                    (*nrs.offset((lnum - (*eap).line1) as isize))
                        .st_u
                        .line
                        .end_col_nr = end_col as varnumber_T;
                }
                (*nrs.offset((lnum - (*eap).line1) as isize)).lnum = lnum;
                if !regmatch.regprog.is_null() {
                    fast_breakcheck();
                }
                if got_int.get() {
                    break '_sortend;
                }
                lnum += 1;
            }
            sortbuf1
                .set(xmalloc((maxlen as size_t).wrapping_add(1 as size_t))
                    as *mut ::core::ffi::c_char);
            sortbuf2
                .set(xmalloc((maxlen as size_t).wrapping_add(1 as size_t))
                    as *mut ::core::ffi::c_char);
            qsort(
                nrs as *mut ::core::ffi::c_void,
                count,
                ::core::mem::size_of::<sorti_T>(),
                Some(
                    sort_compare
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                ),
            );
            if !sort_abort.get() {
                old_count = 0 as bcount_t;
                new_count = 0 as bcount_t;
                lnum_0 = (*eap).line2;
                i = 0 as size_t;
                while i < count {
                    let get_lnum: linenr_T = (*nrs.offset(
                        (if (*eap).forceit != 0 {
                            count.wrapping_sub(i).wrapping_sub(1 as size_t)
                        } else {
                            i
                        }) as isize,
                    ))
                    .lnum;
                    if get_lnum + (count as linenr_T - 1 as linenr_T) != lnum_0 {
                        change_occurred = true_0 != 0;
                    }
                    let mut s_1: *mut ::core::ffi::c_char = ml_get(get_lnum);
                    let mut bytelen: colnr_T = ml_get_len(get_lnum) + 1 as colnr_T;
                    old_count += bytelen as bcount_t;
                    if !unique
                        || i == 0 as size_t
                        || string_compare(
                            s_1 as *const ::core::ffi::c_void,
                            sortbuf1.get() as *const ::core::ffi::c_void,
                        ) != 0 as ::core::ffi::c_int
                    {
                        strcpy(sortbuf1.get(), s_1);
                        let c2rust_fresh3 = lnum_0;
                        lnum_0 = lnum_0 + 1;
                        if ml_append(c2rust_fresh3, sortbuf1.get(), 0 as colnr_T, false_0 != 0)
                            == FAIL
                        {
                            break;
                        }
                        new_count += bytelen as bcount_t;
                    }
                    fast_breakcheck();
                    if got_int.get() {
                        break '_sortend;
                    }
                    i = i.wrapping_add(1);
                }
                if i == count {
                    i = 0 as size_t;
                    while i < count {
                        ml_delete((*eap).line1);
                        i = i.wrapping_add(1);
                    }
                } else {
                    count = 0 as size_t;
                }
                deleted = count as linenr_T - (lnum_0 - (*eap).line2);
                if deleted > 0 as linenr_T {
                    mark_adjust(
                        (*eap).line2 - deleted,
                        (*eap).line2,
                        MAXLNUM as ::core::ffi::c_int as linenr_T,
                        -deleted,
                        kExtmarkNOOP,
                    );
                    msgmore(-(deleted as ::core::ffi::c_int));
                } else if deleted < 0 as linenr_T {
                    mark_adjust(
                        (*eap).line2,
                        MAXLNUM as ::core::ffi::c_int as linenr_T,
                        -deleted,
                        0 as linenr_T,
                        kExtmarkNOOP,
                    );
                }
                if change_occurred as ::core::ffi::c_int != 0 || deleted != 0 as linenr_T {
                    extmark_splice(
                        curbuf.get(),
                        (*eap).line1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                        0 as colnr_T,
                        count as ::core::ffi::c_int,
                        0 as colnr_T,
                        old_count,
                        lnum_0 as ::core::ffi::c_int - (*eap).line2 as ::core::ffi::c_int,
                        0 as colnr_T,
                        new_count,
                        kExtmarkUndo,
                    );
                    changed_lines(
                        curbuf.get(),
                        (*eap).line1,
                        0 as colnr_T,
                        (*eap).line2 + 1 as linenr_T,
                        -deleted,
                        true_0 != 0,
                    );
                }
                (*curwin.get()).w_cursor.lnum = (*eap).line1;
                beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
            }
        }
    }
    xfree(nrs as *mut ::core::ffi::c_void);
    xfree(sortbuf1.get() as *mut ::core::ffi::c_void);
    xfree(sortbuf2.get() as *mut ::core::ffi::c_void);
    vim_regfree(regmatch.regprog);
    if got_int.get() {
        emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
    }
}
pub unsafe fn ex_uniq(mut eap: *mut exarg_T) {
    let mut match_continue: bool = false;
    let mut next_is_unmatch: bool = false;
    let mut done_lnum: linenr_T = 0;
    let mut delete_lnum: linenr_T = 0;
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut maxlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut count: linenr_T = (*eap).line2 - (*eap).line1 + 1 as linenr_T;
    let mut keep_only_unique: bool = false_0 != 0;
    let mut keep_only_not_unique: bool = (*eap).forceit != 0;
    let mut deleted: linenr_T = 0 as linenr_T;
    if count <= 1 as linenr_T {
        return;
    }
    if u_save((*eap).line1 - 1 as linenr_T, (*eap).line2 + 1 as linenr_T) == FAIL {
        return;
    }
    sortbuf1.set(::core::ptr::null_mut::<::core::ffi::c_char>());
    regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
    sort_flt.set(false_0 != 0);
    sort_nr.set(sort_flt.get());
    sort_rx.set(sort_nr.get());
    sort_lc.set(sort_rx.get());
    sort_ic.set(sort_lc.get());
    sort_abort.set(sort_ic.get());
    let mut change_occurred: bool = false_0 != 0;
    let mut p: *mut ::core::ffi::c_char = (*eap).arg;
    '_uniqend: {
        while *p as ::core::ffi::c_int != NUL {
            if !ascii_iswhite(*p as ::core::ffi::c_int) {
                if *p as ::core::ffi::c_int == 'i' as ::core::ffi::c_int {
                    sort_ic.set(true_0 != 0);
                } else if *p as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
                    sort_lc.set(true_0 != 0);
                } else if *p as ::core::ffi::c_int == 'r' as ::core::ffi::c_int {
                    sort_rx.set(true_0 != 0);
                } else if *p as ::core::ffi::c_int == 'u' as ::core::ffi::c_int {
                    if !keep_only_not_unique {
                        keep_only_unique = true_0 != 0;
                    }
                } else if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
                    break;
                } else if (*eap).nextcmd.is_null() && !check_nextcmd(p).is_null() {
                    (*eap).nextcmd = check_nextcmd(p);
                    break;
                } else if !(*p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                    && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                    || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                        && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
                    && regmatch.regprog.is_null()
                {
                    let mut s: *mut ::core::ffi::c_char = skip_regexp_err(
                        p.offset(1 as ::core::ffi::c_int as isize),
                        *p as ::core::ffi::c_int,
                        true_0,
                    );
                    if s.is_null() {
                        break '_uniqend;
                    }
                    *s = NUL as ::core::ffi::c_char;
                    if s == p.offset(1 as ::core::ffi::c_int as isize) {
                        if last_search_pat().is_null() {
                            emsg(gettext(&raw const e_noprevre as *const ::core::ffi::c_char));
                            break '_uniqend;
                        } else {
                            regmatch.regprog = vim_regcomp(last_search_pat(), RE_MAGIC);
                        }
                    } else {
                        regmatch.regprog =
                            vim_regcomp(p.offset(1 as ::core::ffi::c_int as isize), RE_MAGIC);
                    }
                    if regmatch.regprog.is_null() {
                        break '_uniqend;
                    }
                    p = s;
                    regmatch.rm_ic = p_ic.get() != 0;
                } else {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        p,
                    );
                    break '_uniqend;
                }
            }
            p = p.offset(1);
        }
        let mut lnum: linenr_T = (*eap).line1;
        while lnum <= (*eap).line2 {
            let mut len: ::core::ffi::c_int = ml_get_len(lnum);
            if maxlen < len {
                maxlen = len;
            }
            if got_int.get() {
                break '_uniqend;
            }
            lnum += 1;
        }
        sortbuf1
            .set(xmalloc((maxlen as size_t).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char);
        match_continue = false_0 != 0;
        next_is_unmatch = false_0 != 0;
        done_lnum = (*eap).line1 - 1 as linenr_T;
        delete_lnum = 0 as linenr_T;
        let mut i: linenr_T = 0 as linenr_T;
        while i < count {
            let mut get_lnum: linenr_T = (*eap).line1 + i;
            let mut s_0: *mut ::core::ffi::c_char = ml_get(get_lnum);
            let mut len_0: ::core::ffi::c_int = ml_get_len(get_lnum);
            let mut start_col: colnr_T = 0 as colnr_T;
            let mut end_col: colnr_T = len_0 as colnr_T;
            if !regmatch.regprog.is_null()
                && vim_regexec(&raw mut regmatch, s_0, 0 as colnr_T) as ::core::ffi::c_int != 0
            {
                if sort_rx.get() {
                    start_col = regmatch.startp[0 as ::core::ffi::c_int as usize].offset_from(s_0)
                        as colnr_T;
                    end_col =
                        regmatch.endp[0 as ::core::ffi::c_int as usize].offset_from(s_0) as colnr_T;
                } else {
                    start_col =
                        regmatch.endp[0 as ::core::ffi::c_int as usize].offset_from(s_0) as colnr_T;
                }
            } else if !regmatch.regprog.is_null() {
                end_col = 0 as ::core::ffi::c_int as colnr_T;
            }
            let mut save_c: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
            if end_col > 0 as ::core::ffi::c_int {
                save_c = *s_0.offset(end_col as isize);
                *s_0.offset(end_col as isize) = NUL as ::core::ffi::c_char;
            }
            let mut is_match: bool = if i > 0 as linenr_T {
                (string_compare(
                    s_0.offset(start_col as isize) as *const ::core::ffi::c_void,
                    sortbuf1.get() as *const ::core::ffi::c_void,
                ) == 0) as ::core::ffi::c_int
            } else {
                false_0
            } != 0;
            delete_lnum = 0 as ::core::ffi::c_int as linenr_T;
            if next_is_unmatch {
                is_match = false_0 != 0;
                next_is_unmatch = false_0 != 0;
            }
            if !keep_only_unique && !keep_only_not_unique {
                if is_match {
                    delete_lnum = get_lnum;
                } else {
                    strcpy(sortbuf1.get(), s_0.offset(start_col as isize));
                }
            } else if keep_only_not_unique {
                if is_match {
                    done_lnum = get_lnum - 1 as linenr_T;
                    delete_lnum = get_lnum;
                    match_continue = true_0 != 0;
                } else {
                    if i > 0 as linenr_T && !match_continue && get_lnum - 1 as linenr_T > done_lnum
                    {
                        delete_lnum = get_lnum - 1 as linenr_T;
                        next_is_unmatch = true_0 != 0;
                    } else if i >= count - 1 as linenr_T {
                        delete_lnum = get_lnum;
                    }
                    match_continue = false_0 != 0;
                    strcpy(sortbuf1.get(), s_0.offset(start_col as isize));
                }
            } else if is_match {
                if !match_continue {
                    delete_lnum = get_lnum - 1 as linenr_T;
                } else {
                    delete_lnum = get_lnum;
                }
                match_continue = true_0 != 0;
            } else {
                if i == 0 as linenr_T && match_continue as ::core::ffi::c_int != 0 {
                    delete_lnum = get_lnum;
                }
                match_continue = false_0 != 0;
                strcpy(sortbuf1.get(), s_0.offset(start_col as isize));
            }
            if end_col > 0 as ::core::ffi::c_int {
                *s_0.offset(end_col as isize) = save_c;
            }
            if delete_lnum > 0 as linenr_T {
                ml_delete(delete_lnum);
                i = (i as ::core::ffi::c_int
                    - (get_lnum - delete_lnum + 1 as linenr_T) as ::core::ffi::c_int)
                    as linenr_T;
                count -= 1;
                deleted += 1;
                change_occurred = true_0 != 0;
            }
            fast_breakcheck();
            if got_int.get() {
                break '_uniqend;
            }
            i += 1;
        }
        mark_adjust(
            (*eap).line2 - deleted,
            (*eap).line2,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            -deleted,
            (if change_occurred as ::core::ffi::c_int != 0 {
                kExtmarkUndo as ::core::ffi::c_int
            } else {
                kExtmarkNOOP as ::core::ffi::c_int
            }) as ExtmarkOp,
        );
        msgmore(-(deleted as ::core::ffi::c_int));
        if change_occurred {
            changed_lines(
                curbuf.get(),
                (*eap).line1,
                0 as colnr_T,
                (*eap).line2 + 1 as linenr_T,
                -deleted,
                true_0 != 0,
            );
        }
        (*curwin.get()).w_cursor.lnum = (*eap).line1;
        beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    }
    xfree(sortbuf1.get() as *mut ::core::ffi::c_void);
    vim_regfree(regmatch.regprog);
    if got_int.get() {
        emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
    }
}
pub unsafe extern "C" fn do_move(
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut dest: linenr_T,
) -> ::core::ffi::c_int {
    if dest >= line1 && dest < line2 {
        emsg(gettext(
            b"E134: Cannot move a range of lines into itself\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if dest == line1 - 1 as linenr_T || dest == line2 {
        (*curwin.get()).w_cursor.lnum = if dest >= line1 {
            dest
        } else {
            dest + (line2 - line1) + 1 as linenr_T
        };
        return OK;
    }
    let mut start_byte: bcount_t = ml_find_line_or_offset(
        curbuf.get(),
        line1,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        true_0 != 0,
    ) as bcount_t;
    let mut end_byte: bcount_t = ml_find_line_or_offset(
        curbuf.get(),
        line2 + 1 as linenr_T,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        true_0 != 0,
    ) as bcount_t;
    let mut extent_byte: bcount_t = end_byte - start_byte;
    let mut dest_byte: bcount_t = ml_find_line_or_offset(
        curbuf.get(),
        dest + 1 as linenr_T,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        true_0 != 0,
    ) as bcount_t;
    let mut num_lines: linenr_T = line2 - line1 + 1 as linenr_T;
    if u_save(dest, dest + 1 as linenr_T) == FAIL {
        return FAIL;
    }
    let mut l: linenr_T = 0;
    let mut extra: linenr_T = 0;
    extra = 0 as ::core::ffi::c_int as linenr_T;
    l = line1;
    while l <= line2 {
        let mut str: *mut ::core::ffi::c_char =
            xstrnsave(ml_get(l + extra), ml_get_len(l + extra) as size_t);
        ml_append(dest + l - line1, str, 0 as colnr_T, false_0 != 0);
        xfree(str as *mut ::core::ffi::c_void);
        if dest < line1 {
            extra += 1;
        }
        l += 1;
    }
    let mut last_line: linenr_T = (*curbuf.get()).b_ml.ml_line_count;
    mark_adjust_nofold(line1, line2, last_line - line2, 0 as linenr_T, kExtmarkNOOP);
    (*disable_fold_update.ptr()) += 1;
    changed_lines(
        curbuf.get(),
        last_line - num_lines + 1 as linenr_T,
        0 as colnr_T,
        last_line + 1 as linenr_T,
        num_lines,
        false_0 != 0,
    );
    (*disable_fold_update.ptr()) -= 1;
    let mut line_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut byte_off: bcount_t = 0 as bcount_t;
    if dest >= line2 {
        mark_adjust_nofold(
            line2 + 1 as linenr_T,
            dest,
            -num_lines,
            0 as linenr_T,
            kExtmarkNOOP,
        );
        let mut tab: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tab.is_null() {
            let mut win: *mut win_T = if tab == curtab.get() {
                firstwin.get()
            } else {
                (*tab).tp_firstwin
            };
            while !win.is_null() {
                if (*win).w_buffer == curbuf.get() {
                    foldMoveRange(win, &raw mut (*win).w_folds, line1, line2, dest);
                }
                win = (*win).w_next;
            }
            tab = (*tab).tp_next as *mut tabpage_T;
        }
        if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*curbuf.get()).b_op_start.lnum = dest - num_lines + 1 as linenr_T;
            (*curbuf.get()).b_op_end.lnum = dest;
        }
        line_off = -num_lines as ::core::ffi::c_int;
        byte_off = -extent_byte;
    } else {
        mark_adjust_nofold(
            dest + 1 as linenr_T,
            line1 - 1 as linenr_T,
            num_lines,
            0 as linenr_T,
            kExtmarkNOOP,
        );
        let mut tab_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tab_0.is_null() {
            let mut win_0: *mut win_T = if tab_0 == curtab.get() {
                firstwin.get()
            } else {
                (*tab_0).tp_firstwin
            };
            while !win_0.is_null() {
                if (*win_0).w_buffer == curbuf.get() {
                    foldMoveRange(
                        win_0,
                        &raw mut (*win_0).w_folds,
                        dest + 1 as linenr_T,
                        line1 - 1 as linenr_T,
                        line2,
                    );
                }
                win_0 = (*win_0).w_next;
            }
            tab_0 = (*tab_0).tp_next as *mut tabpage_T;
        }
        if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*curbuf.get()).b_op_start.lnum = dest + 1 as linenr_T;
            (*curbuf.get()).b_op_end.lnum = dest + num_lines;
        }
    }
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*curbuf.get()).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
        (*curbuf.get()).b_op_start.col = (*curbuf.get()).b_op_end.col;
    }
    mark_adjust_nofold(
        last_line - num_lines + 1 as linenr_T,
        last_line,
        -(last_line - dest - extra),
        0 as linenr_T,
        kExtmarkNOOP,
    );
    (*disable_fold_update.ptr()) += 1;
    changed_lines(
        curbuf.get(),
        last_line - num_lines + 1 as linenr_T,
        0 as colnr_T,
        last_line + 1 as linenr_T,
        -extra,
        false_0 != 0,
    );
    (*disable_fold_update.ptr()) -= 1;
    buf_updates_send_changes(
        curbuf.get(),
        dest + 1 as linenr_T,
        num_lines as int64_t,
        0 as int64_t,
    );
    if u_save(line1 + extra - 1 as linenr_T, line2 + extra + 1 as linenr_T) == FAIL {
        return FAIL;
    }
    l = line1;
    while l <= line2 {
        ml_delete_flags(line1 + extra, ML_DEL_MESSAGE as ::core::ffi::c_int);
        l += 1;
    }
    if global_busy.get() == 0 && num_lines as OptInt > p_report.get() {
        smsg(
            0 as ::core::ffi::c_int,
            ngettext(
                b"%ld line moved\0".as_ptr() as *const ::core::ffi::c_char,
                b"%ld lines moved\0".as_ptr() as *const ::core::ffi::c_char,
                num_lines as ::core::ffi::c_ulong,
            ),
            num_lines as int64_t,
        );
    }
    extmark_move_region(
        curbuf.get(),
        line1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        0 as colnr_T,
        start_byte,
        line2 as ::core::ffi::c_int - line1 as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
        0 as colnr_T,
        extent_byte,
        dest as ::core::ffi::c_int + line_off,
        0 as colnr_T,
        dest_byte + byte_off,
        kExtmarkUndo,
    );
    if dest >= line1 {
        (*curwin.get()).w_cursor.lnum = dest;
    } else {
        (*curwin.get()).w_cursor.lnum = dest + (line2 - line1) + 1 as linenr_T;
    }
    if line1 < dest {
        dest = (dest as ::core::ffi::c_int + (num_lines + 1 as linenr_T) as ::core::ffi::c_int)
            as linenr_T;
        last_line = (*curbuf.get()).b_ml.ml_line_count;
        dest = if dest < last_line + 1 as linenr_T {
            dest
        } else {
            last_line + 1 as linenr_T
        };
        changed_lines(
            curbuf.get(),
            line1,
            0 as colnr_T,
            dest,
            0 as linenr_T,
            false_0 != 0,
        );
    } else {
        changed_lines(
            curbuf.get(),
            dest + 1 as linenr_T,
            0 as colnr_T,
            line1 + num_lines,
            0 as linenr_T,
            false_0 != 0,
        );
    }
    buf_updates_send_changes(
        curbuf.get(),
        line1 + extra,
        0 as int64_t,
        num_lines as int64_t,
    );
    return OK;
}
pub unsafe extern "C" fn ex_copy(mut line1: linenr_T, mut line2: linenr_T, mut n: linenr_T) {
    let mut count: linenr_T = line2 - line1 + 1 as linenr_T;
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*curbuf.get()).b_op_start.lnum = n + 1 as linenr_T;
        (*curbuf.get()).b_op_end.lnum = n + count;
        (*curbuf.get()).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
        (*curbuf.get()).b_op_start.col = (*curbuf.get()).b_op_end.col;
    }
    if u_save(n, n + 1 as linenr_T) == FAIL {
        return;
    }
    (*curwin.get()).w_cursor.lnum = n;
    while line1 <= line2 {
        let mut p: *mut ::core::ffi::c_char = xstrnsave(ml_get(line1), ml_get_len(line1) as size_t);
        ml_append((*curwin.get()).w_cursor.lnum, p, 0 as colnr_T, false_0 != 0);
        xfree(p as *mut ::core::ffi::c_void);
        if line1 == n {
            line1 = (*curwin.get()).w_cursor.lnum;
        }
        line1 += 1;
        if (*curwin.get()).w_cursor.lnum < line1 {
            line1 += 1;
        }
        if (*curwin.get()).w_cursor.lnum < line2 {
            line2 += 1;
        }
        (*curwin.get()).w_cursor.lnum += 1;
    }
    appended_lines_mark(n, count as ::core::ffi::c_int);
    if VIsual_active.get() {
        check_pos(curbuf.get(), VIsual.ptr());
    }
    msgmore(count as ::core::ffi::c_int);
}
static prevcmd: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
unsafe extern "C" fn prevcmd_is_set() -> ::core::ffi::c_int {
    if (*prevcmd.ptr()).is_null() {
        emsg(gettext(&raw const e_noprev as *const ::core::ffi::c_char));
        return false_0;
    }
    return true_0;
}
pub unsafe extern "C" fn do_bang(
    mut addr_count: ::core::ffi::c_int,
    mut eap: *mut exarg_T,
    mut forceit: bool,
    mut do_in: bool,
    mut do_out: bool,
) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut line1: linenr_T = (*eap).line1;
    let mut line2: linenr_T = (*eap).line2;
    let mut newcmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut free_newcmd: bool = false_0 != 0;
    let mut scroll_save: ::core::ffi::c_int = msg_scroll.get();
    if check_secure() {
        return;
    }
    if addr_count == 0 as ::core::ffi::c_int {
        msg_scroll.set(false_0);
        autowrite_all();
        msg_scroll.set(scroll_save);
    }
    let mut ins_prevcmd: bool = forceit;
    let mut trailarg: *mut ::core::ffi::c_char = skipwhite(arg);
    loop {
        let mut len: size_t = strlen(trailarg).wrapping_add(1 as size_t);
        if !newcmd.is_null() {
            len = len.wrapping_add(strlen(newcmd));
        }
        if ins_prevcmd {
            if prevcmd_is_set() == 0 {
                xfree(newcmd as *mut ::core::ffi::c_void);
                return;
            }
            len = len.wrapping_add(strlen(prevcmd.get()));
        }
        let mut t: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
        *t = NUL as ::core::ffi::c_char;
        if !newcmd.is_null() {
            strcat(t, newcmd);
        }
        if ins_prevcmd {
            strcat(t, prevcmd.get());
        }
        let mut p: *mut ::core::ffi::c_char = t.offset(strlen(t) as isize);
        strcat(t, trailarg);
        xfree(newcmd as *mut ::core::ffi::c_void);
        newcmd = t;
        trailarg = ::core::ptr::null_mut::<::core::ffi::c_char>();
        while *p != 0 {
            if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                if p > newcmd
                    && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                {
                    memmove(
                        p.offset(-(1 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        strlen(p).wrapping_add(1 as size_t),
                    );
                } else {
                    trailarg = p;
                    let c2rust_fresh4 = trailarg;
                    trailarg = trailarg.offset(1);
                    *c2rust_fresh4 = NUL as ::core::ffi::c_char;
                    ins_prevcmd = true_0 != 0;
                    break;
                }
            }
            p = p.offset(1);
        }
        if trailarg.is_null() {
            break;
        }
    }
    if strlen(newcmd) > 0 as size_t {
        xfree(prevcmd.get() as *mut ::core::ffi::c_void);
        prevcmd.set(newcmd);
    } else {
        free_newcmd = true_0 != 0;
    }
    '_theend: {
        if bangredo.get() {
            if prevcmd_is_set() == 0 {
                break '_theend;
            } else {
                let mut cmd: *mut ::core::ffi::c_char = vim_strsave_escaped(
                    prevcmd.get(),
                    b"%#\0".as_ptr() as *const ::core::ffi::c_char,
                );
                AppendToRedobuffLit(cmd, -1 as ::core::ffi::c_int);
                xfree(cmd as *mut ::core::ffi::c_void);
                AppendToRedobuff(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
                bangredo.set(false_0 != 0);
            }
        }
        if *p_shq.get() as ::core::ffi::c_int != NUL {
            if free_newcmd {
                xfree(newcmd as *mut ::core::ffi::c_void);
            }
            newcmd = xmalloc(
                strlen(prevcmd.get())
                    .wrapping_add((2 as size_t).wrapping_mul(strlen(p_shq.get())))
                    .wrapping_add(1 as size_t),
            ) as *mut ::core::ffi::c_char;
            strcpy(newcmd, p_shq.get());
            strcat(newcmd, prevcmd.get());
            strcat(newcmd, p_shq.get());
            free_newcmd = true_0 != 0;
        }
        if addr_count == 0 as ::core::ffi::c_int {
            msg_start();
            msg_ext_set_kind(b"shell_cmd\0".as_ptr() as *const ::core::ffi::c_char);
            msg_putchar(':' as ::core::ffi::c_int);
            msg_putchar('!' as ::core::ffi::c_int);
            msg_outtrans(newcmd, 0 as ::core::ffi::c_int, false_0 != 0);
            msg_clr_eos();
            ui_cursor_goto(msg_row.get(), msg_col.get());
            do_shell(newcmd, 0 as ::core::ffi::c_int);
        } else {
            do_filter(line1, line2, eap, newcmd, do_in, do_out);
            apply_autocmds(
                EVENT_SHELLFILTERPOST,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        }
    }
    if free_newcmd {
        xfree(newcmd as *mut ::core::ffi::c_void);
    }
}
unsafe extern "C" fn do_filter(
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut eap: *mut exarg_T,
    mut cmd: *mut ::core::ffi::c_char,
    mut do_in: bool,
    mut do_out: bool,
) {
    let mut read_linecount: linenr_T = 0;
    let mut cmd_buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut itmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut otmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut old_curbuf: *mut buf_T = curbuf.get();
    let mut shell_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let orig_start: pos_T = (*curbuf.get()).b_op_start;
    let orig_end: pos_T = (*curbuf.get()).b_op_end;
    let stmp: ::core::ffi::c_int = p_stmp.get();
    if *cmd as ::core::ffi::c_int == NUL {
        return;
    }
    let save_cmod_flags: ::core::ffi::c_int = (*cmdmod.ptr()).cmod_flags;
    (*cmdmod.ptr()).cmod_flags &= !(CMOD_LOCKMARKS as ::core::ffi::c_int);
    let mut cursor_save: pos_T = (*curwin.get()).w_cursor;
    let mut linecount: linenr_T = line2 - line1 + 1 as linenr_T;
    (*curwin.get()).w_cursor.lnum = line1;
    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    changed_line_abv_curs();
    invalidate_botline_win(curwin.get());
    if do_out {
        shell_flags |= kShellOptDoOut as ::core::ffi::c_int;
    }
    '_filterend: {
        if !do_in && do_out as ::core::ffi::c_int != 0 && stmp == 0 {
            shell_flags |= kShellOptRead as ::core::ffi::c_int;
            (*curwin.get()).w_cursor.lnum = line2;
        } else if do_in as ::core::ffi::c_int != 0 && !do_out && stmp == 0 {
            shell_flags |= kShellOptWrite as ::core::ffi::c_int;
            (*curbuf.get()).b_op_start.lnum = line1;
            (*curbuf.get()).b_op_end.lnum = line2;
        } else if do_in as ::core::ffi::c_int != 0 && do_out as ::core::ffi::c_int != 0 && stmp == 0
        {
            shell_flags |=
                kShellOptRead as ::core::ffi::c_int | kShellOptWrite as ::core::ffi::c_int;
            (*curbuf.get()).b_op_start.lnum = line1;
            (*curbuf.get()).b_op_end.lnum = line2;
            (*curwin.get()).w_cursor.lnum = line2;
        } else if do_in as ::core::ffi::c_int != 0 && {
            itmp = vim_tempname();
            itmp.is_null()
        } || do_out as ::core::ffi::c_int != 0 && {
            otmp = vim_tempname();
            otmp.is_null()
        } {
            emsg(gettext(&raw const e_notmp as *const ::core::ffi::c_char));
            break '_filterend;
        }
        (*no_wait_return.ptr()) += 1;
        if !itmp.is_null()
            && buf_write(
                curbuf.get(),
                itmp,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                line1,
                line2,
                eap,
                WriteRequest::filter(),
            ) == FAIL
        {
            if !ui_has(kUIMessages) {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            (*no_wait_return.ptr()) -= 1;
            if !aborting() {
                semsg(
                    gettext(b"E482: Can't create file %s\0".as_ptr() as *const ::core::ffi::c_char),
                    itmp,
                );
            }
        } else if curbuf.get() == old_curbuf {
            if !do_out {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            cmd_buf = make_filter_cmd(cmd, itmp, otmp, do_in);
            ui_cursor_goto(
                Rows.get() - 1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            '_error: {
                if do_out {
                    if u_save(line2, line2 + 1 as linenr_T) == FAIL {
                        xfree(cmd_buf as *mut ::core::ffi::c_void);
                        break '_error;
                    } else {
                        redraw_curbuf_later(UPD_VALID);
                    }
                }
                read_linecount = (*curbuf.get()).b_ml.ml_line_count;
                call_shell(
                    cmd_buf,
                    kShellOptFilter as ::core::ffi::c_int | shell_flags,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                );
                xfree(cmd_buf as *mut ::core::ffi::c_void);
                did_check_timestamps.set(false_0 != 0);
                need_check_timestamps.set(true_0 != 0);
                os_breakcheck();
                got_int.set(false_0 != 0);
                if do_out {
                    if !otmp.is_null() {
                        if readfile(
                            otmp,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            line2,
                            0 as linenr_T,
                            MAXLNUM as ::core::ffi::c_int as linenr_T,
                            eap,
                            READ_FILTER as ::core::ffi::c_int,
                            false_0 != 0,
                        ) != OK
                        {
                            if !aborting() {
                                msg_putchar('\n' as ::core::ffi::c_int);
                                semsg(
                                    gettext(
                                        &raw const e_cant_read_file_str
                                            as *const ::core::ffi::c_char,
                                    ),
                                    otmp,
                                );
                            }
                            break '_error;
                        } else if curbuf.get() != old_curbuf {
                            break '_filterend;
                        }
                    }
                    read_linecount = (*curbuf.get()).b_ml.ml_line_count - read_linecount;
                    if shell_flags & kShellOptRead as ::core::ffi::c_int != 0 {
                        (*curbuf.get()).b_op_start.lnum = line2 + 1 as linenr_T;
                        (*curbuf.get()).b_op_end.lnum = (*curwin.get()).w_cursor.lnum;
                        appended_lines_mark(line2, read_linecount as ::core::ffi::c_int);
                    }
                    if do_in {
                        if (*cmdmod.ptr()).cmod_flags & CMOD_KEEPMARKS as ::core::ffi::c_int != 0
                            || vim_strchr(p_cpo.get(), CPO_REMMARK).is_null()
                        {
                            if read_linecount >= linecount {
                                mark_adjust(line1, line2, linecount, 0 as linenr_T, kExtmarkNOOP);
                            } else {
                                mark_adjust(
                                    line1,
                                    line1 + read_linecount - 1 as linenr_T,
                                    linecount,
                                    0 as linenr_T,
                                    kExtmarkNOOP,
                                );
                                mark_adjust(
                                    line1 + read_linecount,
                                    line2,
                                    MAXLNUM as ::core::ffi::c_int as linenr_T,
                                    0 as linenr_T,
                                    kExtmarkNOOP,
                                );
                            }
                        }
                        (*curwin.get()).w_cursor.lnum = line1;
                        del_lines(linecount, true_0 != 0);
                        (*curbuf.get()).b_op_start.lnum -= linecount;
                        (*curbuf.get()).b_op_end.lnum -= linecount;
                        write_lnum_adjust(-linecount);
                        foldUpdate(
                            curwin.get(),
                            (*curbuf.get()).b_op_start.lnum,
                            (*curbuf.get()).b_op_end.lnum,
                        );
                    } else {
                        linecount = (*curbuf.get()).b_op_end.lnum - (*curbuf.get()).b_op_start.lnum
                            + 1 as linenr_T;
                        (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_op_end.lnum;
                    }
                    beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
                    (*no_wait_return.ptr()) -= 1;
                    if linecount as OptInt > p_report.get() {
                        if do_in {
                            vim_snprintf(
                                msg_buf.ptr() as *mut ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 480]>(),
                                gettext(
                                    b"%ld lines filtered\0".as_ptr() as *const ::core::ffi::c_char
                                ),
                                linecount as int64_t,
                            );
                            if msg(
                                msg_buf.ptr() as *mut ::core::ffi::c_char,
                                0 as ::core::ffi::c_int,
                            ) as ::core::ffi::c_int
                                != 0
                                && msg_scroll.get() == 0
                            {
                                set_keep_msg(
                                    msg_buf.ptr() as *mut ::core::ffi::c_char,
                                    0 as ::core::ffi::c_int,
                                );
                            }
                        } else {
                            msgmore(linecount as ::core::ffi::c_int);
                        }
                    }
                    break '_filterend;
                }
            }
            (*curwin.get()).w_cursor = cursor_save;
            (*no_wait_return.ptr()) -= 1;
            wait_return(false_0);
        }
    }
    (*cmdmod.ptr()).cmod_flags = save_cmod_flags;
    if curbuf.get() != old_curbuf {
        (*no_wait_return.ptr()) -= 1;
        emsg(gettext(
            b"E135: *Filter* Autocommands must not change current buffer\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
    } else if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0 {
        (*curbuf.get()).b_op_start = orig_start;
        (*curbuf.get()).b_op_end = orig_end;
    }
    if !itmp.is_null() {
        os_remove(itmp);
    }
    if !otmp.is_null() {
        os_remove(otmp);
    }
    xfree(itmp as *mut ::core::ffi::c_void);
    xfree(otmp as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn do_shell(
    mut cmd: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) {
    if check_secure() {
        msg_end();
        return;
    }
    msg_putchar('\r' as ::core::ffi::c_int);
    msg_putchar('\n' as ::core::ffi::c_int);
    if p_warn.get() != 0 && !autocmd_busy.get() && msg_silent.get() == 0 as ::core::ffi::c_int {
        let mut buf: *mut buf_T = firstbuf.get();
        while !buf.is_null() {
            if bufIsChanged(buf) {
                msg_puts(gettext(
                    b"[No write since last change]\n\0".as_ptr() as *const ::core::ffi::c_char
                ));
                break;
            } else {
                buf = (*buf).b_next;
            }
        }
    }
    ui_cursor_goto(msg_row.get(), msg_col.get());
    call_shell(cmd, flags, ::core::ptr::null_mut::<::core::ffi::c_char>());
    if msg_silent.get() == 0 as ::core::ffi::c_int {
        msg_didout.set(true_0 != 0);
    }
    did_check_timestamps.set(false_0 != 0);
    need_check_timestamps.set(true_0 != 0);
    msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
    msg_col.set(0 as ::core::ffi::c_int);
    apply_autocmds(
        EVENT_SHELLCMDPOST,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
}
pub unsafe extern "C" fn make_filter_cmd(
    mut cmd: *mut ::core::ffi::c_char,
    mut itmp: *mut ::core::ffi::c_char,
    mut otmp: *mut ::core::ffi::c_char,
    mut do_in: bool,
) -> *mut ::core::ffi::c_char {
    let mut is_fish_shell: bool = strncmp(
        invocation_path_tail(p_sh.get(), ::core::ptr::null_mut::<size_t>()),
        b"fish\0".as_ptr() as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int;
    let mut is_pwsh: bool = strncmp(
        invocation_path_tail(p_sh.get(), ::core::ptr::null_mut::<size_t>()),
        b"pwsh\0".as_ptr() as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int
        || strncmp(
            invocation_path_tail(p_sh.get(), ::core::ptr::null_mut::<size_t>()),
            b"powershell\0".as_ptr() as *const ::core::ffi::c_char,
            10 as size_t,
        ) == 0 as ::core::ffi::c_int;
    let mut len: size_t = strlen(cmd).wrapping_add(1 as size_t);
    len = (len as ::core::ffi::c_ulong).wrapping_add(
        (if is_fish_shell as ::core::ffi::c_int != 0 {
            ::core::mem::size_of::<[::core::ffi::c_char; 13]>().wrapping_sub(1 as usize)
        } else if !is_pwsh {
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize)
        } else {
            0 as usize
        }) as ::core::ffi::c_ulong,
    ) as size_t;
    if !itmp.is_null() {
        len = (len as ::core::ffi::c_ulong).wrapping_add(
            (if is_pwsh as ::core::ffi::c_int != 0 {
                strlen(itmp)
                    .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 24]>())
                    .wrapping_sub(1 as size_t)
                    .wrapping_add(6 as size_t)
            } else {
                strlen(itmp)
                    .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 10]>())
                    .wrapping_sub(1 as size_t)
            }) as ::core::ffi::c_ulong,
        ) as size_t;
    }
    if do_in as ::core::ffi::c_int != 0 && is_pwsh as ::core::ffi::c_int != 0 {
        len = (len as ::core::ffi::c_ulong).wrapping_add(::core::mem::size_of::<
            [::core::ffi::c_char; 11],
        >() as ::core::ffi::c_ulong) as size_t;
    }
    if !otmp.is_null() {
        len = len.wrapping_add(
            strlen(otmp)
                .wrapping_add(strlen(p_srr.get()))
                .wrapping_add(2 as size_t),
        );
    }
    let buf: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    if is_pwsh {
        if !itmp.is_null() {
            xstrlcpy(
                buf,
                b"& { Get-Content \0".as_ptr() as *const ::core::ffi::c_char,
                len.wrapping_sub(1 as size_t),
            );
            xstrlcat(buf, itmp, len.wrapping_sub(1 as size_t));
            xstrlcat(
                buf,
                b" | & \0".as_ptr() as *const ::core::ffi::c_char,
                len.wrapping_sub(1 as size_t),
            );
            xstrlcat(buf, cmd, len.wrapping_sub(1 as size_t));
            xstrlcat(
                buf,
                b" }\0".as_ptr() as *const ::core::ffi::c_char,
                len.wrapping_sub(1 as size_t),
            );
        } else if do_in {
            xstrlcpy(
                buf,
                b" $input | \0".as_ptr() as *const ::core::ffi::c_char,
                len.wrapping_sub(1 as size_t),
            );
            xstrlcat(buf, cmd, len);
        } else {
            xstrlcpy(buf, cmd, len);
        }
    } else {
        if !itmp.is_null() || !otmp.is_null() {
            let mut fmt: *mut ::core::ffi::c_char = (if is_fish_shell as ::core::ffi::c_int != 0 {
                b"begin; %s; end\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"(%s)\0".as_ptr() as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char;
            vim_snprintf(buf, len, fmt, cmd);
        } else {
            xstrlcpy(buf, cmd, len);
        }
        if !itmp.is_null() {
            xstrlcat(
                buf,
                b" < \0".as_ptr() as *const ::core::ffi::c_char,
                len.wrapping_sub(1 as size_t),
            );
            xstrlcat(buf, itmp, len.wrapping_sub(1 as size_t));
        }
    }
    if !otmp.is_null() {
        append_redir(buf, len, p_srr.get(), otmp);
    }
    return buf;
}
pub unsafe extern "C" fn append_redir(
    buf: *mut ::core::ffi::c_char,
    buflen: size_t,
    opt: *const ::core::ffi::c_char,
    fname: *const ::core::ffi::c_char,
) {
    let end: *mut ::core::ffi::c_char = buf.offset(strlen(buf) as isize);
    let mut p: *const ::core::ffi::c_char = opt;
    loop {
        p = strchr(p, '%' as ::core::ffi::c_int);
        if p.is_null() {
            break;
        }
        if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 's' as ::core::ffi::c_int
        {
            break;
        }
        if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '%' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        p = p.offset(1);
    }
    if !p.is_null() {
        *end = ' ' as ::core::ffi::c_char;
        vim_snprintf(
            end.offset(1 as ::core::ffi::c_int as isize),
            (buflen as ptrdiff_t
                - end
                    .offset(1 as ::core::ffi::c_int as isize)
                    .offset_from(buf)) as size_t,
            opt,
            fname,
        );
    } else {
        vim_snprintf(
            end,
            (buflen as ptrdiff_t - end.offset_from(buf)) as size_t,
            b" %s %s\0".as_ptr() as *const ::core::ffi::c_char,
            opt,
            fname,
        );
    };
}
pub unsafe extern "C" fn print_line_no_prefix(
    mut lnum: linenr_T,
    mut use_number: bool,
    mut list: bool,
) {
    let mut numbuf: [::core::ffi::c_char; 30] = [0; 30];
    if (*curwin.get()).w_onebuf_opt.wo_nu != 0 || use_number as ::core::ffi::c_int != 0 {
        vim_snprintf(
            &raw mut numbuf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
            b"%*d \0".as_ptr() as *const ::core::ffi::c_char,
            number_width(curwin.get()),
            lnum,
        );
        msg_puts_hl(
            &raw mut numbuf as *mut ::core::ffi::c_char,
            HLF_N + 1 as ::core::ffi::c_int,
            false_0 != 0,
        );
    }
    msg_prt_line(ml_get(lnum), list);
}
static global_need_msg_kind: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub unsafe extern "C" fn print_line(
    mut lnum: linenr_T,
    mut use_number: bool,
    mut list: bool,
    mut first: bool,
) {
    let mut save_silent: bool = silent_mode.get();
    if message_filtered(ml_get(lnum)) {
        return;
    }
    silent_mode.set(false_0 != 0);
    info_message.set(true_0 != 0);
    if (global_busy.get() == 0 || global_need_msg_kind.get() as ::core::ffi::c_int != 0)
        && first as ::core::ffi::c_int != 0
    {
        msg_start();
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        global_need_msg_kind.set(false_0 != 0);
    } else if !save_silent {
        msg_putchar('\n' as ::core::ffi::c_int);
    }
    print_line_no_prefix(lnum, use_number, list);
    if save_silent {
        msg_putchar('\n' as ::core::ffi::c_int);
        silent_mode.set(save_silent);
    }
    info_message.set(false_0 != 0);
}
pub unsafe extern "C" fn rename_buffer(
    mut new_fname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut buf: *mut buf_T = curbuf.get();
    apply_autocmds(
        EVENT_BUFFILEPRE,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    if buf != curbuf.get() {
        return FAIL;
    }
    if aborting() {
        return FAIL;
    }
    let mut fname: *mut ::core::ffi::c_char = (*curbuf.get()).b_ffname;
    let mut sfname: *mut ::core::ffi::c_char = (*curbuf.get()).b_sfname;
    let mut xfname: *mut ::core::ffi::c_char = (*curbuf.get()).b_fname;
    (*curbuf.get()).b_ffname = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*curbuf.get()).b_sfname = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if setfname(
        curbuf.get(),
        new_fname,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0 != 0,
    ) == FAIL
    {
        (*curbuf.get()).b_ffname = fname;
        (*curbuf.get()).b_sfname = sfname;
        return FAIL;
    }
    (*curbuf.get()).b_flags |= BF_NOTEDITED;
    if !xfname.is_null() && *xfname as ::core::ffi::c_int != NUL {
        buf = buflist_new(
            fname,
            xfname,
            (*curwin.get()).w_cursor.lnum,
            0 as ::core::ffi::c_int,
        );
        if !buf.is_null()
            && (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            (*curwin.get()).w_alt_fnum = (*buf).handle as ::core::ffi::c_int;
        }
    }
    xfree(fname as *mut ::core::ffi::c_void);
    xfree(sfname as *mut ::core::ffi::c_void);
    apply_autocmds(
        EVENT_BUFFILEPOST,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    do_autochdir();
    return OK;
}
pub unsafe fn ex_file(mut eap: *mut exarg_T) {
    if (*eap).addr_count > 0 as ::core::ffi::c_int
        && (*(*eap).arg as ::core::ffi::c_int != NUL
            || (*eap).line2 > 0 as linenr_T
            || (*eap).addr_count > 1 as ::core::ffi::c_int)
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    if *(*eap).arg as ::core::ffi::c_int != NUL || (*eap).addr_count == 1 as ::core::ffi::c_int {
        if rename_buffer((*eap).arg) == FAIL {
            return;
        }
        redraw_tabline.set(true_0 != 0);
    }
    if *(*eap).arg as ::core::ffi::c_int == NUL || !shortmess(SHM_FILEINFO as ::core::ffi::c_int) {
        fileinfo(false_0, false_0, (*eap).forceit != 0);
    }
}
pub unsafe fn ex_update(mut eap: *mut exarg_T) {
    if curbufIsChanged() as ::core::ffi::c_int != 0
        || !bt_nofilename(curbuf.get())
            && !(*curbuf.get()).b_ffname.is_null()
            && !os_path_exists((*curbuf.get()).b_ffname)
    {
        do_write(eap);
    }
}
pub unsafe fn ex_write(mut eap: *mut exarg_T) {
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_saveas as ::core::ffi::c_int {
        (*eap).line1 = 1 as ::core::ffi::c_int as linenr_T;
        (*eap).line2 = (*curbuf.get()).b_ml.ml_line_count;
    }
    if (*eap).usefilter != 0 {
        do_bang(
            1 as ::core::ffi::c_int,
            eap,
            false_0 != 0,
            true_0 != 0,
            false_0 != 0,
        );
    } else {
        do_write(eap);
    };
}
unsafe extern "C" fn check_writable(mut fname: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if os_nodetype(fname) == NODE_OTHER {
        semsg(
            gettext(b"E503: \"%s\" is not a file or writable device\0".as_ptr()
                as *const ::core::ffi::c_char),
            fname,
        );
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn handle_mkdir_p_arg(
    mut eap: *mut exarg_T,
    mut fname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if (*eap).mkdir_p != 0 && os_file_mkdir(fname, 0o755 as int32_t) < 0 as ::core::ffi::c_int {
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn do_write(mut eap: *mut exarg_T) -> ::core::ffi::c_int {
    let mut other: bool = false;
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut retval: ::core::ffi::c_int = FAIL;
    let mut free_fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut alt_buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    if not_writing() {
        return FAIL;
    }
    let mut ffname: *mut ::core::ffi::c_char = (*eap).arg;
    '_theend: {
        if *ffname as ::core::ffi::c_int == NUL {
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_saveas as ::core::ffi::c_int {
                emsg(gettext(&raw const e_argreq as *const ::core::ffi::c_char));
                break '_theend;
            } else {
                other = false_0 != 0;
            }
        } else {
            fname = ffname;
            free_fname = fix_fname(ffname);
            if !free_fname.is_null() {
                ffname = free_fname;
            }
            other = otherfile(ffname);
        }
        if other {
            if !vim_strchr(p_cpo.get(), CPO_ALTWRITE).is_null()
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_saveas as ::core::ffi::c_int
            {
                alt_buf = setaltfname(ffname, fname, 1 as linenr_T);
            } else {
                alt_buf = buflist_findname(ffname);
            }
            if !alt_buf.is_null() && !(*alt_buf).b_ml.ml_mfp.is_null() {
                emsg(gettext(
                    &raw const e_bufloaded as *const ::core::ffi::c_char,
                ));
                break '_theend;
            }
        }
        if !(!other
            && (bt_dontwrite_msg(curbuf.get()) as ::core::ffi::c_int != 0
                || check_fname() == FAIL
                || check_writable((*curbuf.get()).b_ffname) == FAIL
                || check_readonly(&raw mut (*eap).forceit, curbuf.get()) != 0))
        {
            if !other {
                ffname = (*curbuf.get()).b_ffname;
                fname = (*curbuf.get()).b_fname;
                if ((*eap).line1 != 1 as linenr_T
                    || (*eap).line2 != (*curbuf.get()).b_ml.ml_line_count)
                    && (*eap).forceit == 0
                    && (*eap).append == 0
                    && p_wa.get() == 0
                {
                    if p_confirm.get() != 0
                        || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0
                    {
                        if vim_dialog_yesno(
                            VIM_QUESTION as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            gettext(b"Write partial file?\0".as_ptr() as *const ::core::ffi::c_char),
                            2 as ::core::ffi::c_int,
                        ) != VIM_YES as ::core::ffi::c_int
                        {
                            break '_theend;
                        } else {
                            (*eap).forceit = true_0;
                        }
                    } else {
                        emsg(gettext(b"E140: Use ! to write partial buffer\0".as_ptr()
                            as *const ::core::ffi::c_char));
                        break '_theend;
                    }
                }
            }
            if check_overwrite(eap, curbuf.get(), fname, ffname, other) == OK {
                if (*eap).cmdidx as ::core::ffi::c_int == CMD_saveas as ::core::ffi::c_int
                    && !alt_buf.is_null()
                {
                    let mut was_curbuf: *mut buf_T = curbuf.get();
                    apply_autocmds(
                        EVENT_BUFFILEPRE,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        false_0 != 0,
                        curbuf.get(),
                    );
                    apply_autocmds(
                        EVENT_BUFFILEPRE,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        false_0 != 0,
                        alt_buf,
                    );
                    if curbuf.get() != was_curbuf || aborting() as ::core::ffi::c_int != 0 {
                        retval = FAIL;
                        break '_theend;
                    } else {
                        fname = (*alt_buf).b_fname;
                        (*alt_buf).b_fname = (*curbuf.get()).b_fname;
                        (*curbuf.get()).b_fname = fname;
                        fname = (*alt_buf).b_ffname;
                        (*alt_buf).b_ffname = (*curbuf.get()).b_ffname;
                        (*curbuf.get()).b_ffname = fname;
                        fname = (*alt_buf).b_sfname;
                        (*alt_buf).b_sfname = (*curbuf.get()).b_sfname;
                        (*curbuf.get()).b_sfname = fname;
                        buf_name_changed(curbuf.get());
                        apply_autocmds(
                            EVENT_BUFFILEPOST,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            false_0 != 0,
                            curbuf.get(),
                        );
                        apply_autocmds(
                            EVENT_BUFFILEPOST,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            false_0 != 0,
                            alt_buf,
                        );
                        if (*alt_buf).b_p_bl == 0 {
                            (*alt_buf).b_p_bl = true_0;
                            apply_autocmds(
                                EVENT_BUFADD,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                false_0 != 0,
                                alt_buf,
                            );
                        }
                        if curbuf.get() != was_curbuf || aborting() as ::core::ffi::c_int != 0 {
                            retval = FAIL;
                            break '_theend;
                        } else {
                            if *(*curbuf.get()).b_p_ft as ::core::ffi::c_int == NUL {
                                if augroup_exists(
                                    b"filetypedetect\0".as_ptr() as *const ::core::ffi::c_char
                                ) {
                                    do_doautocmd(
                                        b"filetypedetect BufRead\0".as_ptr()
                                            as *const ::core::ffi::c_char
                                            as *mut ::core::ffi::c_char,
                                        true_0 != 0,
                                        ::core::ptr::null_mut::<bool>(),
                                    );
                                }
                                do_modelines(0 as ::core::ffi::c_int);
                            }
                            fname = (*curbuf.get()).b_sfname;
                        }
                    }
                }
                if handle_mkdir_p_arg(eap, fname) == FAIL {
                    retval = FAIL;
                } else {
                    let mut name_was_missing: ::core::ffi::c_int =
                        (*curbuf.get()).b_ffname.is_null() as ::core::ffi::c_int;
                    retval = buf_write(
                        curbuf.get(),
                        ffname,
                        fname,
                        (*eap).line1,
                        (*eap).line2,
                        eap,
                        WriteRequest {
                            append: (*eap).append != 0,
                            forceit: (*eap).forceit != 0,
                            reset_changed: true,
                            filtering: false,
                        },
                    );
                    if (*eap).cmdidx as ::core::ffi::c_int == CMD_saveas as ::core::ffi::c_int {
                        if retval == OK {
                            (*curbuf.get()).b_p_ro = false_0;
                            redraw_tabline.set(true_0 != 0);
                        }
                    }
                    if (*eap).cmdidx as ::core::ffi::c_int == CMD_saveas as ::core::ffi::c_int
                        || name_was_missing != 0
                    {
                        do_autochdir();
                    }
                }
            }
        }
    }
    xfree(free_fname as *mut ::core::ffi::c_void);
    return retval;
}
pub unsafe extern "C" fn check_overwrite(
    mut eap: *mut exarg_T,
    mut buf: *mut buf_T,
    mut fname: *mut ::core::ffi::c_char,
    mut ffname: *mut ::core::ffi::c_char,
    mut other: bool,
) -> ::core::ffi::c_int {
    if (other as ::core::ffi::c_int != 0
        || !bt_nofilename(buf)
            && ((*buf).b_flags & BF_NOTEDITED != 0
                || (*buf).b_flags & BF_NEW != 0 && vim_strchr(p_cpo.get(), CPO_OVERNEW).is_null()
                || (*buf).b_flags & BF_READERR != 0))
        && p_wa.get() == 0
        && os_path_exists(ffname) as ::core::ffi::c_int != 0
    {
        if (*eap).forceit == 0 && (*eap).append == 0 {
            if os_isdir(ffname) {
                semsg(
                    gettext(&raw const e_isadir2 as *const ::core::ffi::c_char),
                    ffname,
                );
                return FAIL;
            }
            if p_confirm.get() != 0
                || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0
            {
                let mut buff: [::core::ffi::c_char; 1000] = [0; 1000];
                dialog_msg(
                    &raw mut buff as *mut ::core::ffi::c_char,
                    gettext(
                        b"Overwrite existing file \"%s\"?\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    fname,
                );
                if vim_dialog_yesno(
                    VIM_QUESTION as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    &raw mut buff as *mut ::core::ffi::c_char,
                    2 as ::core::ffi::c_int,
                ) != VIM_YES as ::core::ffi::c_int
                {
                    return FAIL;
                }
                (*eap).forceit = true_0;
            } else {
                emsg(gettext(&raw const e_exists as *const ::core::ffi::c_char));
                return FAIL;
            }
        }
        if other as ::core::ffi::c_int != 0 && emsg_silent.get() == 0 {
            let mut dir: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if *p_dir.get() as ::core::ffi::c_int == NUL {
                dir = xmalloc(5 as size_t) as *mut ::core::ffi::c_char;
                strcpy(
                    dir,
                    b".\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
            } else {
                dir = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
                let mut p: *mut ::core::ffi::c_char = p_dir.get();
                copy_option_part(
                    &raw mut p,
                    dir,
                    MAXPATHL as size_t,
                    b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
            }
            let mut swapname: *mut ::core::ffi::c_char =
                makeswapname(fname, ffname, curbuf.get(), dir);
            xfree(dir as *mut ::core::ffi::c_void);
            if os_path_exists(swapname) {
                if p_confirm.get() != 0
                    || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0
                {
                    let mut buff_0: [::core::ffi::c_char; 1000] = [0; 1000];
                    dialog_msg(
                        &raw mut buff_0 as *mut ::core::ffi::c_char,
                        gettext(b"Swap file \"%s\" exists, overwrite anyway?\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        swapname,
                    );
                    if vim_dialog_yesno(
                        VIM_QUESTION as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        &raw mut buff_0 as *mut ::core::ffi::c_char,
                        2 as ::core::ffi::c_int,
                    ) != VIM_YES as ::core::ffi::c_int
                    {
                        xfree(swapname as *mut ::core::ffi::c_void);
                        return FAIL;
                    }
                    (*eap).forceit = true_0;
                } else {
                    semsg(
                        gettext(
                            b"E768: Swap file exists: %s (:silent! overrides)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        swapname,
                    );
                    xfree(swapname as *mut ::core::ffi::c_void);
                    return FAIL;
                }
            }
            xfree(swapname as *mut ::core::ffi::c_void);
        }
    }
    return OK;
}
pub unsafe fn ex_wnext(mut eap: *mut exarg_T) {
    let mut i: ::core::ffi::c_int = 0;
    if *(*eap).cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'n' as ::core::ffi::c_int
    {
        i = (*curwin.get()).w_arg_idx + (*eap).line2 as ::core::ffi::c_int;
    } else {
        i = (*curwin.get()).w_arg_idx - (*eap).line2 as ::core::ffi::c_int;
    }
    (*eap).line1 = 1 as ::core::ffi::c_int as linenr_T;
    (*eap).line2 = (*curbuf.get()).b_ml.ml_line_count;
    if do_write(eap) != FAIL {
        do_argfile(eap, i);
    }
}
pub unsafe fn do_wqall(mut eap: *mut exarg_T) {
    let mut error: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut save_forceit: ::core::ffi::c_int = (*eap).forceit;
    let mut save_exiting: bool = exiting.get();
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_xall as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_wqall as ::core::ffi::c_int
    {
        if before_quit_all(eap) == FAIL {
            return;
        }
        exiting.set(true_0 != 0);
    }
    let mut buf: *mut buf_T = firstbuf.get();
    's_136: while !buf.is_null() {
        's_32: {
            if exiting.get() as ::core::ffi::c_int != 0
                && (*eap).forceit == 0
                && !(*buf).terminal.is_null()
                && channel_job_running((*buf).b_p_channel as uint64_t) as ::core::ffi::c_int != 0
            {
                no_write_message_buf(buf);
                error += 1;
            } else if !bufIsChanged(buf) || bt_dontwrite(buf) as ::core::ffi::c_int != 0 {
                break 's_32;
            }
            if not_writing() {
                error += 1;
                break 's_136;
            } else {
                if (*buf).b_ffname.is_null() {
                    semsg(
                        gettext(b"E141: No file name for buffer %ld\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        (*buf).handle as int64_t,
                    );
                    error += 1;
                } else if check_readonly(&raw mut (*eap).forceit, buf) != 0
                    || check_overwrite(eap, buf, (*buf).b_fname, (*buf).b_ffname, false_0 != 0)
                        == FAIL
                {
                    error += 1;
                } else {
                    let mut bufref: bufref_T = bufref_T::default();
                    set_bufref(&raw mut bufref, buf);
                    if handle_mkdir_p_arg(eap, (*buf).b_fname) == FAIL
                        || buf_write_all(buf, (*eap).forceit != 0) == FAIL
                    {
                        error += 1;
                    }
                    if !bufref_valid(&raw mut bufref) {
                        buf = firstbuf.get();
                    }
                }
                (*eap).forceit = save_forceit;
            }
        }
        buf = (*buf).b_next;
    }
    if exiting.get() {
        if error == 0 {
            getout(0 as ::core::ffi::c_int);
        }
        not_exiting(save_exiting);
    }
}
unsafe extern "C" fn not_writing() -> bool {
    if p_write.get() != 0 {
        return false_0 != 0;
    }
    emsg(gettext(
        b"E142: File not written: Writing is disabled by 'write' option\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    return true_0 != 0;
}
unsafe extern "C" fn check_readonly(
    mut forceit: *mut ::core::ffi::c_int,
    mut buf: *mut buf_T,
) -> ::core::ffi::c_int {
    if *forceit == 0
        && ((*buf).b_p_ro != 0
            || os_path_exists((*buf).b_ffname) as ::core::ffi::c_int != 0
                && os_file_is_writable((*buf).b_ffname) == 0)
    {
        if (p_confirm.get() != 0
            || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0)
            && !(*buf).b_fname.is_null()
        {
            let mut buff: [::core::ffi::c_char; 1000] = [0; 1000];
            if (*buf).b_p_ro != 0 {
                dialog_msg(
                    &raw mut buff as *mut ::core::ffi::c_char,
                    gettext(
                        b"'readonly' option is set for \"%s\".\nDo you wish to write anyway?\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                    (*buf).b_fname,
                );
            } else {
                dialog_msg(
                    &raw mut buff as *mut ::core::ffi::c_char,
                    gettext(
                        b"File permissions of \"%s\" are read-only.\nIt may still be possible to write it.\nDo you wish to try?\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                    (*buf).b_fname,
                );
            }
            if vim_dialog_yesno(
                VIM_QUESTION as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                &raw mut buff as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) == VIM_YES as ::core::ffi::c_int
            {
                *forceit = true_0;
                return false_0;
            }
            return true_0;
        } else if (*buf).b_p_ro != 0 {
            emsg(gettext(&raw const e_readonly as *const ::core::ffi::c_char));
        } else {
            semsg(
                gettext(b"E505: \"%s\" is read-only (add ! to override)\0".as_ptr()
                    as *const ::core::ffi::c_char),
                (*buf).b_fname,
            );
        }
        return true_0;
    }
    return false_0;
}
pub unsafe extern "C" fn getfile(
    mut fnum: ::core::ffi::c_int,
    mut ffname_arg: *mut ::core::ffi::c_char,
    mut sfname_arg: *mut ::core::ffi::c_char,
    mut setpm: bool,
    mut lnum: linenr_T,
    mut forceit: bool,
) -> ::core::ffi::c_int {
    if !check_can_set_curbuf_forceit(forceit as ::core::ffi::c_int) {
        return GETFILE_ERROR as ::core::ffi::c_int;
    }
    let mut ffname: *mut ::core::ffi::c_char = ffname_arg;
    let mut sfname: *mut ::core::ffi::c_char = sfname_arg;
    let mut other: bool = false;
    let mut retval: ::core::ffi::c_int = 0;
    let mut free_me: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if text_locked() {
        return GETFILE_ERROR as ::core::ffi::c_int;
    }
    if curbuf_locked() {
        return GETFILE_ERROR as ::core::ffi::c_int;
    }
    if fnum == 0 as ::core::ffi::c_int {
        fname_expand(curbuf.get(), &raw mut ffname, &raw mut sfname);
        other = otherfile(ffname);
        free_me = ffname;
    } else {
        other = fnum != (*curbuf.get()).handle;
    }
    if other {
        (*no_wait_return.ptr()) += 1;
    }
    '_theend: {
        if other as ::core::ffi::c_int != 0
            && !forceit
            && (*curbuf.get()).b_nwindows == 1 as ::core::ffi::c_int
            && !buf_hide(curbuf.get())
            && curbufIsChanged() as ::core::ffi::c_int != 0
            && autowrite(curbuf.get(), forceit) == FAIL
        {
            if p_confirm.get() != 0 && p_write.get() != 0 {
                dialog_changed(curbuf.get(), false_0 != 0);
            }
            if curbufIsChanged() {
                (*no_wait_return.ptr()) -= 1;
                no_write_message();
                retval = GETFILE_NOT_WRITTEN as ::core::ffi::c_int;
                break '_theend;
            }
        }
        if other {
            (*no_wait_return.ptr()) -= 1;
        }
        if setpm {
            setpcmark();
        }
        if !other {
            if lnum != 0 as linenr_T {
                (*curwin.get()).w_cursor.lnum = lnum;
            }
            check_cursor_lnum(curwin.get());
            beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
            retval = GETFILE_SAME_FILE as ::core::ffi::c_int;
        } else if do_ecmd(
            fnum,
            ffname,
            sfname,
            ::core::ptr::null_mut::<exarg_T>(),
            lnum,
            (if buf_hide(curbuf.get()) as ::core::ffi::c_int != 0 {
                ECMD_HIDE as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) + (if forceit as ::core::ffi::c_int != 0 {
                ECMD_FORCEIT as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }),
            curwin.get(),
        ) == OK
        {
            retval = GETFILE_OPEN_OTHER as ::core::ffi::c_int;
        } else {
            retval = GETFILE_ERROR as ::core::ffi::c_int;
        }
    }
    xfree(free_me as *mut ::core::ffi::c_void);
    return retval;
}
pub unsafe extern "C" fn set_swapcommand(
    mut command: *mut ::core::ffi::c_char,
    mut newlnum: linenr_T,
) -> bool {
    if command.is_null() && newlnum <= 0 as linenr_T
        || *get_vim_var_str(VV_SWAPCOMMAND) as ::core::ffi::c_int != NUL
    {
        return false_0 != 0;
    }
    let valsize: size_t = if !command.is_null() {
        strlen(command).wrapping_add(3 as size_t)
    } else {
        30 as size_t
    };
    let mut val: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    val.data = xmalloc(valsize) as *mut ::core::ffi::c_char;
    val.size = if !command.is_null() {
        vim_snprintf_safelen(
            val.data,
            valsize,
            b":%s\r\0".as_ptr() as *const ::core::ffi::c_char,
            command,
        )
    } else {
        vim_snprintf_safelen(
            val.data,
            valsize,
            b"%ldG\0".as_ptr() as *const ::core::ffi::c_char,
            newlnum as int64_t,
        )
    };
    set_vim_var_string(VV_SWAPCOMMAND, val.data, val.size as ptrdiff_t);
    xfree(val.data as *mut ::core::ffi::c_void);
    return true_0 != 0;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn do_ecmd(
    mut fnum: ::core::ffi::c_int,
    mut ffname: *mut ::core::ffi::c_char,
    mut sfname: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    mut newlnum: linenr_T,
    mut flags: ::core::ffi::c_int,
    mut oldwin: *mut win_T,
) -> ::core::ffi::c_int {
    let mut other_file: bool = false;
    let mut oldbuf: ::core::ffi::c_int = 0;
    let mut auto_buf: bool = false_0 != 0;
    let mut new_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut did_set_swapcommand: bool = false_0 != 0;
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut bufref: bufref_T = bufref_T::default();
    let mut old_curbuf: bufref_T = bufref_T::default();
    let mut free_fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut retval: ::core::ffi::c_int = FAIL;
    let mut topline: linenr_T = 0 as linenr_T;
    let mut newcol: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut solcol: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut command: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut did_get_winopts: bool = false_0 != 0;
    let mut readfile_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut did_inc_redrawing_disabled: bool = false_0 != 0;
    let mut so_ptr: *mut OptInt = if (*curwin.get()).w_onebuf_opt.wo_so >= 0 as OptInt {
        &raw mut (*curwin.get()).w_onebuf_opt.wo_so
    } else {
        p_so.ptr()
    };
    if !eap.is_null() {
        command = (*eap).do_ecmd_cmd;
    }
    set_bufref(&raw mut old_curbuf, curbuf.get());
    '_theend: {
        if fnum != 0 as ::core::ffi::c_int {
            if fnum == (*curbuf.get()).handle {
                return OK;
            }
            other_file = true_0 != 0;
        } else {
            if sfname.is_null() {
                sfname = ffname;
            }
            if flags & (ECMD_ADDBUF as ::core::ffi::c_int | ECMD_ALTBUF as ::core::ffi::c_int) != 0
                && (ffname.is_null() || *ffname as ::core::ffi::c_int == NUL)
            {
                break '_theend;
            } else if ffname.is_null() {
                other_file = true_0 != 0;
            } else if *ffname as ::core::ffi::c_int == NUL && (*curbuf.get()).b_ffname.is_null() {
                other_file = false_0 != 0;
            } else {
                if *ffname as ::core::ffi::c_int == NUL {
                    ffname = (*curbuf.get()).b_ffname;
                    sfname = (*curbuf.get()).b_fname;
                }
                free_fname = fix_fname(ffname);
                if !free_fname.is_null() {
                    ffname = free_fname;
                }
                other_file = otherfile(ffname);
            }
        }
        if !other_file && !(*curbuf.get()).terminal.is_null() {
            check_arg_idx(curwin.get());
            maketitle();
            retval = OK;
        } else if (!other_file && flags & ECMD_OLDBUF as ::core::ffi::c_int == 0
            || (*curbuf.get()).b_nwindows == 1 as ::core::ffi::c_int
                && flags
                    & (ECMD_HIDE as ::core::ffi::c_int
                        | ECMD_ADDBUF as ::core::ffi::c_int
                        | ECMD_ALTBUF as ::core::ffi::c_int)
                    == 0)
            && check_changed(
                curbuf.get(),
                (if p_awa.get() != 0 {
                    CCGD_AW as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) | (if other_file as ::core::ffi::c_int != 0 {
                    0 as ::core::ffi::c_int
                } else {
                    CCGD_MULTWIN as ::core::ffi::c_int
                }) | (if flags & ECMD_FORCEIT as ::core::ffi::c_int != 0 {
                    CCGD_FORCEIT as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) | (if eap.is_null() {
                    0 as ::core::ffi::c_int
                } else {
                    CCGD_EXCMD as ::core::ffi::c_int
                }),
            ) as ::core::ffi::c_int
                != 0
        {
            if fnum == 0 as ::core::ffi::c_int
                && other_file as ::core::ffi::c_int != 0
                && !ffname.is_null()
            {
                setaltfname(
                    ffname,
                    sfname,
                    if newlnum < 0 as linenr_T {
                        0 as linenr_T
                    } else {
                        newlnum
                    },
                );
            }
        } else {
            reset_VIsual();
            if !oldwin.is_null() && !win_valid(oldwin) {
                oldwin = ::core::ptr::null_mut::<win_T>();
            }
            did_set_swapcommand = set_swapcommand(command, newlnum);
            if other_file {
                let prev_alt_fnum: ::core::ffi::c_int = (*curwin.get()).w_alt_fnum;
                if flags & (ECMD_ADDBUF as ::core::ffi::c_int | ECMD_ALTBUF as ::core::ffi::c_int)
                    == 0
                {
                    if (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        (*curwin.get()).w_alt_fnum = (*curbuf.get()).handle as ::core::ffi::c_int;
                    }
                    if !oldwin.is_null() {
                        buflist_altfpos(oldwin);
                    }
                }
                if fnum != 0 {
                    buf = buflist_findnr(fnum);
                } else if flags
                    & (ECMD_ADDBUF as ::core::ffi::c_int | ECMD_ALTBUF as ::core::ffi::c_int)
                    != 0
                {
                    let mut tlnum: linenr_T = 0 as linenr_T;
                    if !command.is_null() {
                        tlnum = atol(command) as linenr_T;
                        if tlnum <= 0 as linenr_T {
                            tlnum = 1 as ::core::ffi::c_int as linenr_T;
                        }
                    }
                    let newbuf: *const buf_T = buflist_new(
                        ffname,
                        sfname,
                        tlnum,
                        BLN_LISTED as ::core::ffi::c_int | BLN_NOCURWIN as ::core::ffi::c_int,
                    );
                    if !newbuf.is_null() && flags & ECMD_ALTBUF as ::core::ffi::c_int != 0 {
                        (*curwin.get()).w_alt_fnum = (*newbuf).handle as ::core::ffi::c_int;
                    }
                    break '_theend;
                } else {
                    buf = buflist_new(
                        ffname,
                        sfname,
                        0 as linenr_T,
                        BLN_CURBUF as ::core::ffi::c_int
                            | (if flags & ECMD_SET_HELP as ::core::ffi::c_int != 0 {
                                0 as ::core::ffi::c_int
                            } else {
                                BLN_LISTED as ::core::ffi::c_int
                            }),
                    );
                    if !oldwin.is_null() {
                        oldwin = curwin.get();
                    }
                    set_bufref(&raw mut old_curbuf, curbuf.get());
                }
                if buf.is_null() {
                    break '_theend;
                } else if (*buf).b_locked_split != 0 {
                    if oldwin.is_null()
                        && !(*curwin.get()).w_buffer.is_null()
                        && (*(*curwin.get()).w_buffer).b_nwindows > 1 as ::core::ffi::c_int
                    {
                        (*(*curwin.get()).w_buffer).b_nwindows -= 1;
                    }
                    emsg(gettext(
                        &raw const e_cannot_switch_to_a_closing_buffer
                            as *const ::core::ffi::c_char,
                    ));
                    break '_theend;
                } else {
                    if (*curwin.get()).w_alt_fnum == (*buf).handle
                        && prev_alt_fnum != 0 as ::core::ffi::c_int
                    {
                        (*curwin.get()).w_alt_fnum = prev_alt_fnum;
                    }
                    if (*buf).b_ml.ml_mfp.is_null() {
                        oldbuf = false_0;
                    } else {
                        oldbuf = true_0;
                        set_bufref(&raw mut bufref, buf);
                        buf_check_timestamp(buf);
                        if !bufref_valid(&raw mut bufref) || curbuf.get() != old_curbuf.br_buf {
                            break '_theend;
                        } else if aborting() {
                            break '_theend;
                        }
                    }
                    if oldbuf != 0 && newlnum == ECMD_LASTL as ::core::ffi::c_int as linenr_T
                        || newlnum == ECMD_LAST as ::core::ffi::c_int as linenr_T
                    {
                        let mut pos: *mut pos_T = &raw mut (*(buflist_findfmark
                            as unsafe extern "C" fn(*mut buf_T) -> *mut fmark_T)(
                            buf
                        ))
                        .mark;
                        newlnum = (*pos).lnum;
                        solcol = (*pos).col as ::core::ffi::c_int;
                    }
                    if buf != curbuf.get() {
                        '_c2rust_label: {
                            if (*cmdwin_buf.ptr()).is_null() {
                            } else {
                                __assert_fail(
                                    b"cmdwin_buf == NULL\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/ex_cmds.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    2549 as ::core::ffi::c_uint,
                                    b"int do_ecmd(int, char *, char *, exarg_T *, linenr_T, int, win_T *)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        let save_cmdwin_type: ::core::ffi::c_int = cmdwin_type.get();
                        let save_cmdwin_win: *mut win_T = cmdwin_win.get();
                        let save_cmdwin_old_curwin: *mut win_T = cmdwin_old_curwin.get();
                        cmdwin_type.set(0 as ::core::ffi::c_int);
                        cmdwin_win.set(::core::ptr::null_mut::<win_T>());
                        cmdwin_old_curwin.set(::core::ptr::null_mut::<win_T>());
                        if !(*buf).b_fname.is_null() {
                            new_name = xstrdup((*buf).b_fname);
                        }
                        let save_au_new_curbuf: bufref_T = au_new_curbuf.get();
                        set_bufref(au_new_curbuf.ptr(), buf);
                        apply_autocmds(
                            EVENT_BUFLEAVE,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            false_0 != 0,
                            curbuf.get(),
                        );
                        cmdwin_type.set(save_cmdwin_type);
                        cmdwin_win.set(save_cmdwin_win);
                        cmdwin_old_curwin.set(save_cmdwin_old_curwin);
                        if !bufref_valid(au_new_curbuf.ptr()) {
                            delbuf_msg(new_name);
                            au_new_curbuf.set(save_au_new_curbuf);
                            break '_theend;
                        } else if aborting() {
                            xfree(new_name as *mut ::core::ffi::c_void);
                            au_new_curbuf.set(save_au_new_curbuf);
                            break '_theend;
                        } else {
                            if buf == curbuf.get() {
                                auto_buf = true_0 != 0;
                            } else {
                                let mut the_curwin: *mut win_T = curwin.get();
                                let mut was_curbuf: *mut buf_T = curbuf.get();
                                (*the_curwin).w_locked = true_0 != 0;
                                (*buf).b_locked += 1;
                                if curbuf.get() == old_curbuf.br_buf {
                                    buf_copy_options(buf, BCO_ENTER as ::core::ffi::c_int);
                                }
                                u_sync(false_0 != 0);
                                let did_decrement: bool = close_buffer(
                                    oldwin,
                                    curbuf.get(),
                                    if flags & ECMD_HIDE as ::core::ffi::c_int != 0
                                        || !(*curbuf.get()).terminal.is_null()
                                            && terminal_running((*curbuf.get()).terminal)
                                                as ::core::ffi::c_int
                                                != 0
                                    {
                                        0 as ::core::ffi::c_int
                                    } else {
                                        DOBUF_UNLOAD as ::core::ffi::c_int
                                    },
                                    false_0 != 0,
                                    false_0 != 0,
                                );
                                if win_valid(the_curwin) {
                                    (*the_curwin).w_locked = false_0 != 0;
                                }
                                (*buf).b_locked -= 1;
                                if aborting() as ::core::ffi::c_int != 0
                                    && !(*curwin.get()).w_buffer.is_null()
                                {
                                    xfree(new_name as *mut ::core::ffi::c_void);
                                    au_new_curbuf.set(save_au_new_curbuf);
                                    break '_theend;
                                } else if !bufref_valid(au_new_curbuf.ptr()) {
                                    delbuf_msg(new_name);
                                    au_new_curbuf.set(save_au_new_curbuf);
                                    break '_theend;
                                } else {
                                    if buf == curbuf.get() {
                                        if did_decrement as ::core::ffi::c_int != 0
                                            && buf_valid(was_curbuf) as ::core::ffi::c_int != 0
                                        {
                                            (*was_curbuf).b_nwindows += 1;
                                        }
                                        if win_valid_any_tab(oldwin) as ::core::ffi::c_int != 0
                                            && (*oldwin).w_buffer.is_null()
                                        {
                                            (*oldwin).w_buffer = was_curbuf;
                                        }
                                        auto_buf = true_0 != 0;
                                    } else {
                                        if (*curwin.get()).w_buffer.is_null()
                                            || (*curwin.get()).w_s
                                                == &raw mut (*(*curwin.get()).w_buffer).b_s
                                        {
                                            (*curwin.get()).w_s = &raw mut (*buf).b_s;
                                        }
                                        (*curwin.get()).w_buffer = buf;
                                        curbuf.set(buf);
                                        (*curbuf.get()).b_nwindows += 1;
                                        if oldbuf == 0 && !eap.is_null() {
                                            set_file_options(true_0 != 0, eap);
                                            set_forced_fenc(eap);
                                        }
                                    }
                                    get_winopts(curbuf.get());
                                    did_get_winopts = true_0 != 0;
                                }
                            }
                            xfree(new_name as *mut ::core::ffi::c_void);
                            au_new_curbuf.set(save_au_new_curbuf);
                        }
                    }
                    (*curwin.get()).w_pcmark.lnum = 1 as ::core::ffi::c_int as linenr_T;
                    (*curwin.get()).w_pcmark.col = 0 as ::core::ffi::c_int as colnr_T;
                }
            } else if flags
                & (ECMD_ADDBUF as ::core::ffi::c_int | ECMD_ALTBUF as ::core::ffi::c_int)
                != 0
                || check_fname() == FAIL
            {
                break '_theend;
            } else {
                oldbuf = flags & ECMD_OLDBUF as ::core::ffi::c_int;
            }
            (*RedrawingDisabled.ptr()) += 1;
            did_inc_redrawing_disabled = true_0 != 0;
            buf = curbuf.get();
            if flags & ECMD_SET_HELP as ::core::ffi::c_int != 0
                || keep_help_flag.get() as ::core::ffi::c_int != 0
            {
                prepare_help_buffer();
            } else if !(*curbuf.get()).b_help {
                set_buflisted(true_0);
            }
            if buf == curbuf.get() {
                if !aborting() {
                    (*curbuf.get()).b_did_filetype = false_0 != 0;
                    if !other_file && oldbuf == 0 {
                        set_last_cursor(curwin.get());
                        if newlnum == ECMD_LAST as ::core::ffi::c_int as linenr_T
                            || newlnum == ECMD_LASTL as ::core::ffi::c_int as linenr_T
                        {
                            newlnum = (*curwin.get()).w_cursor.lnum;
                            solcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                        }
                        buf = curbuf.get();
                        if !(*buf).b_fname.is_null() {
                            new_name = xstrdup((*buf).b_fname);
                        } else {
                            new_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        }
                        set_bufref(&raw mut bufref, buf);
                        if (*curbuf.get()).b_flags & BF_NEVERLOADED == 0
                            && (p_ur.get() < 0 as OptInt
                                || (*curbuf.get()).b_ml.ml_line_count as OptInt <= p_ur.get())
                        {
                            u_sync(false_0 != 0);
                            if u_savecommon(
                                curbuf.get(),
                                0 as linenr_T,
                                (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T,
                                0 as linenr_T,
                                true_0 != 0,
                            ) == FAIL
                            {
                                xfree(new_name as *mut ::core::ffi::c_void);
                                break '_theend;
                            } else {
                                u_unchanged(curbuf.get());
                                buf_freeall(curbuf.get(), BFA_KEEP_UNDO as ::core::ffi::c_int);
                                readfile_flags = READ_KEEP_UNDO as ::core::ffi::c_int;
                            }
                        } else {
                            buf_freeall(curbuf.get(), 0 as ::core::ffi::c_int);
                        }
                        if !bufref_valid(&raw mut bufref) {
                            delbuf_msg(new_name);
                            break '_theend;
                        } else {
                            xfree(new_name as *mut ::core::ffi::c_void);
                            if buf != curbuf.get() {
                                break '_theend;
                            } else if aborting() {
                                break '_theend;
                            } else {
                                buf_clear_file(curbuf.get());
                                (*curbuf.get()).b_op_start.lnum =
                                    0 as ::core::ffi::c_int as linenr_T;
                                (*curbuf.get()).b_op_end.lnum = 0 as ::core::ffi::c_int as linenr_T;
                            }
                        }
                    }
                    retval = OK;
                    if !other_file {
                        (*curbuf.get()).b_flags &= !BF_NOTEDITED;
                    }
                    check_arg_idx(curwin.get());
                    if !auto_buf {
                        curwin_init();
                        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                        while !tp.is_null() {
                            let mut win: *mut win_T = if tp == curtab.get() {
                                firstwin.get()
                            } else {
                                (*tp).tp_firstwin
                            };
                            while !win.is_null() {
                                if (*win).w_buffer == curbuf.get() {
                                    foldUpdateAll(win);
                                }
                                win = (*win).w_next;
                            }
                            tp = (*tp).tp_next as *mut tabpage_T;
                        }
                        do_autochdir();
                        let mut orig_pos: pos_T = (*curwin.get()).w_cursor;
                        topline = (*curwin.get()).w_topline;
                        if oldbuf == 0 {
                            swap_exists_action.set(SEA_DIALOG);
                            (*curbuf.get()).b_flags |= BF_CHECK_RO;
                            if flags & ECMD_NOWINENTER as ::core::ffi::c_int != 0 {
                                readfile_flags |= READ_NOWINENTER as ::core::ffi::c_int;
                            }
                            if should_abort(open_buffer(false_0 != 0, eap, readfile_flags)) {
                                retval = FAIL;
                            }
                            if swap_exists_action.get() == SEA_QUIT {
                                retval = FAIL;
                            }
                            handle_swap_exists(&raw mut old_curbuf);
                        } else {
                            do_modelines(OPT_WINONLY as ::core::ffi::c_int);
                            apply_autocmds_retval(
                                EVENT_BUFENTER,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                false_0 != 0,
                                curbuf.get(),
                                &raw mut retval,
                            );
                            if flags & ECMD_NOWINENTER as ::core::ffi::c_int
                                == 0 as ::core::ffi::c_int
                            {
                                apply_autocmds_retval(
                                    EVENT_BUFWINENTER,
                                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                    false_0 != 0,
                                    curbuf.get(),
                                    &raw mut retval,
                                );
                            }
                        }
                        check_arg_idx(curwin.get());
                        if !equalpos((*curwin.get()).w_cursor, orig_pos) {
                            let mut text: *const ::core::ffi::c_char = get_cursor_line_ptr();
                            if (*curwin.get()).w_cursor.lnum != orig_pos.lnum
                                || (*curwin.get()).w_cursor.col
                                    != skipwhite(text).offset_from(text) as ::core::ffi::c_int
                            {
                                newlnum = (*curwin.get()).w_cursor.lnum;
                                newcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                            }
                        }
                        if (*curwin.get()).w_topline == topline {
                            topline = 0 as ::core::ffi::c_int as linenr_T;
                        }
                        changed_line_abv_curs();
                        maketitle();
                    }
                    if (*curwin.get()).w_onebuf_opt.wo_diff != 0 {
                        diff_buf_add(curbuf.get());
                        diff_invalidate(curbuf.get());
                    }
                    if did_get_winopts as ::core::ffi::c_int != 0
                        && (*curwin.get()).w_onebuf_opt.wo_spell != 0
                        && *(*(*curwin.get()).w_s).b_p_spl as ::core::ffi::c_int != NUL
                    {
                        parse_spelllang(curwin.get());
                    }
                    if command.is_null() {
                        if newcol >= 0 as ::core::ffi::c_int {
                            (*curwin.get()).w_cursor.lnum = newlnum;
                            (*curwin.get()).w_cursor.col = newcol as colnr_T;
                            check_cursor(curwin.get());
                        } else if newlnum > 0 as linenr_T {
                            (*curwin.get()).w_cursor.lnum = newlnum;
                            check_cursor_lnum(curwin.get());
                            if solcol >= 0 as ::core::ffi::c_int && p_sol.get() == 0 {
                                (*curwin.get()).w_cursor.col = solcol as colnr_T;
                                check_cursor_col(curwin.get());
                                (*curwin.get()).w_cursor.coladd =
                                    0 as ::core::ffi::c_int as colnr_T;
                                (*curwin.get()).w_set_curswant = true_0;
                            } else {
                                beginline(
                                    BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int,
                                );
                            }
                        } else {
                            if exmode_active.get() {
                                (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
                            }
                            beginline(
                                BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int,
                            );
                        }
                    }
                    check_lnums(false_0 != 0);
                    if oldbuf != 0 && !auto_buf {
                        let mut msg_scroll_save: ::core::ffi::c_int = msg_scroll.get();
                        if shortmess(SHM_OVERALL as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                            && msg_listdo_overwrite.get() == 0
                            && !exiting.get()
                            && p_verbose.get() == 0 as OptInt
                        {
                            msg_scroll.set(false_0);
                        }
                        if msg_scroll.get() == 0 {
                            msg_check_for_delay(false_0 != 0);
                        }
                        msg_start();
                        msg_scroll.set(msg_scroll_save);
                        msg_scrolled_ign.set(true_0 != 0);
                        if !shortmess(SHM_FILEINFO as ::core::ffi::c_int) {
                            fileinfo(false_0, true_0, false_0 != 0);
                        }
                        msg_scrolled_ign.set(false_0 != 0);
                    }
                    (*curbuf.get()).b_last_used = time(::core::ptr::null_mut::<time_t>());
                    if !command.is_null() {
                        do_cmdline(command, None, NULL_0, DOCMD_VERBOSE as ::core::ffi::c_int);
                    }
                    if (*curbuf.get()).b_kmap_state as ::core::ffi::c_int & KEYMAP_INIT != 0 {
                        keymap_init();
                    }
                    (*RedrawingDisabled.ptr()) -= 1;
                    did_inc_redrawing_disabled = false_0 != 0;
                    if !skip_redraw.get() {
                        let mut n: OptInt = *so_ptr;
                        if topline == 0 as linenr_T && command.is_null() {
                            *so_ptr = 999 as OptInt;
                        }
                        update_topline(curwin.get());
                        (*curwin.get()).w_scbind_pos = plines_m_win_fill(
                            curwin.get(),
                            1 as linenr_T,
                            (*curwin.get()).w_topline,
                        );
                        *so_ptr = n;
                        redraw_curbuf_later(UPD_NOT_VALID);
                    }
                    do_autochdir();
                }
            }
        }
    }
    if bufref_valid(&raw mut old_curbuf) as ::core::ffi::c_int != 0
        && !(*old_curbuf.br_buf).terminal.is_null()
    {
        terminal_check_size((*old_curbuf.br_buf).terminal);
    }
    if (!bufref_valid(&raw mut old_curbuf) || curbuf.get() != old_curbuf.br_buf)
        && !(*curbuf.get()).terminal.is_null()
    {
        terminal_check_size((*curbuf.get()).terminal);
    }
    if did_inc_redrawing_disabled {
        (*RedrawingDisabled.ptr()) -= 1;
    }
    if did_set_swapcommand {
        set_vim_var_string(
            VV_SWAPCOMMAND,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
    }
    xfree(free_fname as *mut ::core::ffi::c_void);
    return retval;
}
unsafe extern "C" fn delbuf_msg(mut name: *mut ::core::ffi::c_char) {
    semsg(
        gettext(
            b"E143: Autocommands unexpectedly deleted new buffer %s\0".as_ptr()
                as *const ::core::ffi::c_char,
        ),
        if name.is_null() {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            name as *const ::core::ffi::c_char
        },
    );
    xfree(name as *mut ::core::ffi::c_void);
    (*au_new_curbuf.ptr()).br_buf = ::core::ptr::null_mut::<buf_T>();
    (*au_new_curbuf.ptr()).br_buf_free_count = 0 as ::core::ffi::c_int;
}
static append_indent: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub unsafe fn ex_append(mut eap: *mut exarg_T) {
    let mut theline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut did_undo: bool = false_0 != 0;
    let mut lnum: linenr_T = (*eap).line2;
    let mut indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut empty: bool = (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0;
    if (*eap).forceit != 0 {
        (*curbuf.get()).b_p_ai = ((*curbuf.get()).b_p_ai == 0) as ::core::ffi::c_int;
    }
    if (*eap).cmdidx as ::core::ffi::c_int != CMD_change as ::core::ffi::c_int
        && (*curbuf.get()).b_p_ai != 0
        && lnum > 0 as linenr_T
    {
        append_indent.set(get_indent_lnum(lnum));
    }
    if (*eap).cmdidx as ::core::ffi::c_int != CMD_append as ::core::ffi::c_int {
        lnum -= 1;
    }
    if empty as ::core::ffi::c_int != 0 && lnum == 1 as linenr_T {
        lnum = 0 as ::core::ffi::c_int as linenr_T;
    }
    State.set(MODE_INSERT);
    if (*curbuf.get()).b_p_iminsert == B_IMODE_LMAP as OptInt {
        (*State.ptr()) |= MODE_LANGMAP;
    }
    loop {
        msg_scroll.set(true_0);
        need_wait_return.set(false_0 != 0);
        if (*curbuf.get()).b_p_ai != 0 {
            if append_indent.get() >= 0 as ::core::ffi::c_int {
                indent = append_indent.get();
                append_indent.set(-1 as ::core::ffi::c_int);
            } else if lnum > 0 as linenr_T {
                indent = get_indent_lnum(lnum);
            }
        }
        if *(*eap).arg as ::core::ffi::c_int == '|' as ::core::ffi::c_int {
            theline = xstrdup((*eap).arg.offset(1 as ::core::ffi::c_int as isize));
            *(*eap).arg = NUL as ::core::ffi::c_char;
        } else if (*eap).ea_getline.is_none() {
            if (*eap).nextcmd.is_null() {
                break;
            }
            p = vim_strchr((*eap).nextcmd, NL);
            if p.is_null() {
                p = (*eap).nextcmd.offset(strlen((*eap).nextcmd) as isize);
            }
            theline = xmemdupz(
                (*eap).nextcmd as *const ::core::ffi::c_void,
                p.offset_from((*eap).nextcmd) as size_t,
            ) as *mut ::core::ffi::c_char;
            if *p as ::core::ffi::c_int != NUL {
                p = p.offset(1);
            } else {
                p = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            (*eap).nextcmd = p;
        } else {
            let mut save_State: ::core::ffi::c_int = State.get();
            State.set(MODE_CMDLINE);
            theline = (*eap).ea_getline.expect("non-null function pointer")(
                if (*(*eap).cstack).cs_looplevel > 0 as ::core::ffi::c_int {
                    -1 as ::core::ffi::c_int
                } else {
                    NUL
                },
                (*eap).cookie,
                indent,
                true_0 != 0,
            );
            State.set(save_State);
        }
        lines_left.set(Rows.get() - 1 as ::core::ffi::c_int);
        if theline.is_null() {
            break;
        }
        let mut vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        p = theline;
        while indent > vcol {
            if *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
                vcol += 1;
            } else {
                if *p as ::core::ffi::c_int != TAB {
                    break;
                }
                vcol += 8 as ::core::ffi::c_int - vcol % 8 as ::core::ffi::c_int;
            }
            p = p.offset(1);
        }
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || !did_undo
                && u_save(
                    lnum,
                    lnum + 1 as linenr_T
                        + (if empty as ::core::ffi::c_int != 0 {
                            1 as linenr_T
                        } else {
                            0 as linenr_T
                        }),
                ) == FAIL
        {
            xfree(theline as *mut ::core::ffi::c_void);
            break;
        } else {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                *theline.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
            }
            did_undo = true_0 != 0;
            ml_append(lnum, theline, 0 as colnr_T, false_0 != 0);
            if empty {
                appended_lines(lnum, 1 as linenr_T);
            } else {
                appended_lines_mark(lnum, 1 as ::core::ffi::c_int);
            }
            xfree(theline as *mut ::core::ffi::c_void);
            lnum += 1;
            if empty {
                ml_delete(2 as linenr_T);
                empty = false_0 != 0;
            }
        }
    }
    State.set(MODE_NORMAL);
    ui_cursor_shape();
    if (*eap).forceit != 0 {
        (*curbuf.get()).b_p_ai = ((*curbuf.get()).b_p_ai == 0) as ::core::ffi::c_int;
    }
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*curbuf.get()).b_op_start.lnum = if (*eap).line2 < (*curbuf.get()).b_ml.ml_line_count {
            (*eap).line2 + 1 as linenr_T
        } else {
            (*curbuf.get()).b_ml.ml_line_count
        };
        if (*eap).cmdidx as ::core::ffi::c_int != CMD_append as ::core::ffi::c_int {
            (*curbuf.get()).b_op_start.lnum -= 1;
        }
        (*curbuf.get()).b_op_end.lnum = if (*eap).line2 < lnum {
            lnum
        } else {
            (*curbuf.get()).b_op_start.lnum
        };
        (*curbuf.get()).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
        (*curbuf.get()).b_op_start.col = (*curbuf.get()).b_op_end.col;
    }
    (*curwin.get()).w_cursor.lnum = lnum;
    check_cursor_lnum(curwin.get());
    beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    need_wait_return.set(false_0 != 0);
    ex_no_reprint.set(true_0 != 0);
}
pub unsafe fn ex_change(mut eap: *mut exarg_T) {
    let mut lnum: linenr_T = 0;
    if (*eap).line2 >= (*eap).line1
        && u_save((*eap).line1 - 1 as linenr_T, (*eap).line2 + 1 as linenr_T) == FAIL
    {
        return;
    }
    if if (*eap).forceit != 0 {
        ((*curbuf.get()).b_p_ai == 0) as ::core::ffi::c_int
    } else {
        (*curbuf.get()).b_p_ai
    } != 0
    {
        append_indent.set(get_indent_lnum((*eap).line1));
    }
    lnum = (*eap).line2;
    while lnum >= (*eap).line1 {
        if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
            break;
        }
        ml_delete((*eap).line1);
        lnum -= 1;
    }
    check_cursor_lnum(curwin.get());
    deleted_lines_mark(
        (*eap).line1,
        (*eap).line2 as ::core::ffi::c_int - lnum as ::core::ffi::c_int,
    );
    (*eap).line2 = (*eap).line1;
    ex_append(eap);
}
pub unsafe fn ex_z(mut eap: *mut exarg_T) {
    let mut bigness: int64_t = 0;
    let mut minus: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut start: linenr_T = 0;
    let mut end: linenr_T = 0;
    let mut curs: linenr_T = 0;
    let mut lnum: linenr_T = (*eap).line2;
    if (*eap).forceit != 0 {
        bigness = (Rows.get() - 1 as ::core::ffi::c_int) as int64_t;
    } else if firstwin.get() == lastwin.get() {
        bigness = ((*curwin.get()).w_onebuf_opt.wo_scr * 2 as OptInt) as int64_t;
    } else {
        bigness = ((*curwin.get()).w_view_height - 3 as ::core::ffi::c_int) as int64_t;
    }
    bigness = if bigness > 1 as int64_t {
        bigness
    } else {
        1 as int64_t
    };
    let mut x: *mut ::core::ffi::c_char = (*eap).arg;
    let mut kind: *mut ::core::ffi::c_char = x;
    if *kind as ::core::ffi::c_int == '-' as ::core::ffi::c_int
        || *kind as ::core::ffi::c_int == '+' as ::core::ffi::c_int
        || *kind as ::core::ffi::c_int == '=' as ::core::ffi::c_int
        || *kind as ::core::ffi::c_int == '^' as ::core::ffi::c_int
        || *kind as ::core::ffi::c_int == '.' as ::core::ffi::c_int
    {
        x = x.offset(1);
    }
    while *x as ::core::ffi::c_int == '-' as ::core::ffi::c_int
        || *x as ::core::ffi::c_int == '+' as ::core::ffi::c_int
    {
        x = x.offset(1);
    }
    if *x as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        if !ascii_isdigit(*x as ::core::ffi::c_int) {
            emsg(gettext(
                (e_non_numeric_argument_to_z.ptr() as *const _) as *const ::core::ffi::c_char,
            ));
            return;
        }
        bigness = atol(x) as int64_t;
        if bigness > (2 as linenr_T * (*curbuf.get()).b_ml.ml_line_count) as int64_t
            || bigness < 0 as int64_t
        {
            bigness = (2 as linenr_T * (*curbuf.get()).b_ml.ml_line_count) as int64_t;
        }
        p_window.set(bigness as ::core::ffi::c_int as OptInt);
        if *kind as ::core::ffi::c_int == '=' as ::core::ffi::c_int {
            bigness += 2 as int64_t;
        }
    }
    if *kind as ::core::ffi::c_int == '-' as ::core::ffi::c_int
        || *kind as ::core::ffi::c_int == '+' as ::core::ffi::c_int
    {
        x = kind.offset(1 as ::core::ffi::c_int as isize);
        while *x as ::core::ffi::c_int == *kind as ::core::ffi::c_int {
            x = x.offset(1);
        }
    }
    match *kind as ::core::ffi::c_int {
        45 => {
            start = lnum - bigness as linenr_T * x.offset_from(kind) as linenr_T + 1 as linenr_T;
            end = start + bigness as linenr_T - 1 as linenr_T;
            curs = end;
        }
        61 => {
            start = lnum - (bigness as linenr_T + 1 as linenr_T) / 2 as linenr_T + 1 as linenr_T;
            end = lnum + (bigness as linenr_T + 1 as linenr_T) / 2 as linenr_T - 1 as linenr_T;
            curs = lnum;
            minus = 1 as ::core::ffi::c_int;
        }
        94 => {
            start = lnum - bigness as linenr_T * 2 as linenr_T;
            end = lnum - bigness as linenr_T;
            curs = lnum - bigness as linenr_T;
        }
        46 => {
            start = lnum - (bigness as linenr_T + 1 as linenr_T) / 2 as linenr_T + 1 as linenr_T;
            end = lnum + (bigness as linenr_T + 1 as linenr_T) / 2 as linenr_T - 1 as linenr_T;
            curs = end;
        }
        _ => {
            start = lnum;
            if *kind as ::core::ffi::c_int == '+' as ::core::ffi::c_int {
                start = (start as ::core::ffi::c_int
                    + (bigness as linenr_T * (x.offset_from(kind) - 1 as isize) as linenr_T
                        + 1 as linenr_T) as ::core::ffi::c_int) as linenr_T;
            } else if (*eap).addr_count == 0 as ::core::ffi::c_int {
                start += 1;
            }
            end = start + bigness as linenr_T - 1 as linenr_T;
            curs = end;
        }
    }
    start = if start > 1 as linenr_T {
        start
    } else {
        1 as linenr_T
    };
    end = if end < (*curbuf.get()).b_ml.ml_line_count {
        end
    } else {
        (*curbuf.get()).b_ml.ml_line_count
    };
    curs = if (if curs > 1 as linenr_T {
        curs
    } else {
        1 as linenr_T
    }) < (*curbuf.get()).b_ml.ml_line_count
    {
        if curs > 1 as linenr_T {
            curs
        } else {
            1 as linenr_T
        }
    } else {
        (*curbuf.get()).b_ml.ml_line_count
    };
    let mut i: linenr_T = start;
    while i <= end {
        if minus != 0 && i == lnum {
            msg_putchar('\n' as ::core::ffi::c_int);
            let mut j: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while j < Columns.get() {
                msg_putchar('-' as ::core::ffi::c_int);
                j += 1;
            }
        }
        print_line(
            i,
            (*eap).flags & EXFLAG_NR != 0,
            (*eap).flags & EXFLAG_LIST != 0,
            i == start,
        );
        if minus != 0 && i == lnum {
            msg_putchar('\n' as ::core::ffi::c_int);
            let mut j_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while j_0 < Columns.get() {
                msg_putchar('-' as ::core::ffi::c_int);
                j_0 += 1;
            }
        }
        i += 1;
    }
    if (*curwin.get()).w_cursor.lnum != curs {
        (*curwin.get()).w_cursor.lnum = curs;
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    }
    ex_no_reprint.set(true_0 != 0);
}
pub unsafe extern "C" fn check_secure() -> bool {
    if secure.get() != 0 {
        secure.set(2 as ::core::ffi::c_int);
        emsg(gettext(&raw const e_curdir as *const ::core::ffi::c_char));
        return true_0 != 0;
    }
    if sandbox.get() != 0 as ::core::ffi::c_int {
        emsg(gettext(&raw const e_sandbox as *const ::core::ffi::c_char));
        return true_0 != 0;
    }
    return false_0 != 0;
}
static old_sub: GlobalCell<SubReplacementString> = GlobalCell::new(SubReplacementString {
    sub: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    timestamp: 0 as Timestamp,
    additional_data: ::core::ptr::null_mut::<AdditionalData>(),
});
static global_need_beginline: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub unsafe extern "C" fn sub_get_replacement(ret_sub: *mut SubReplacementString) {
    *ret_sub = old_sub.get();
}
pub unsafe extern "C" fn sub_set_replacement(mut sub: SubReplacementString) {
    xfree((*old_sub.ptr()).sub as *mut ::core::ffi::c_void);
    if sub.additional_data != (*old_sub.ptr()).additional_data {
        xfree((*old_sub.ptr()).additional_data as *mut ::core::ffi::c_void);
    }
    old_sub.set(sub);
}
unsafe extern "C" fn sub_joining_lines(
    mut eap: *mut exarg_T,
    mut pat: *mut ::core::ffi::c_char,
    mut patlen: size_t,
    mut sub: *const ::core::ffi::c_char,
    mut cmd: *const ::core::ffi::c_char,
    mut save: bool,
    mut keeppatterns: bool,
) -> bool {
    if !pat.is_null()
        && strcmp(pat, b"\\n\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
        && *sub as ::core::ffi::c_int == NUL
        && (*cmd as ::core::ffi::c_int == NUL
            || *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                && (*cmd as ::core::ffi::c_int == 'g' as ::core::ffi::c_int
                    || *cmd as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
                    || *cmd as ::core::ffi::c_int == 'p' as ::core::ffi::c_int
                    || *cmd as ::core::ffi::c_int == '#' as ::core::ffi::c_int))
    {
        if (*eap).skip != 0 {
            return true_0 != 0;
        }
        (*curwin.get()).w_cursor.lnum = (*eap).line1;
        if *cmd as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
            (*eap).flags = EXFLAG_LIST;
        } else if *cmd as ::core::ffi::c_int == '#' as ::core::ffi::c_int {
            (*eap).flags = EXFLAG_NR;
        } else if *cmd as ::core::ffi::c_int == 'p' as ::core::ffi::c_int {
            (*eap).flags = EXFLAG_PRINT;
        }
        let mut joined_lines_count: linenr_T = (*eap).line2 - (*eap).line1
            + 1 as linenr_T
            + (if (*eap).line2 < (*curbuf.get()).b_ml.ml_line_count {
                1 as linenr_T
            } else {
                0 as linenr_T
            });
        if joined_lines_count > 1 as linenr_T {
            do_join(
                joined_lines_count as size_t,
                false_0 != 0,
                true_0 != 0,
                false_0 != 0,
                true_0 != 0,
            );
            sub_nsubs.set((joined_lines_count - 1 as linenr_T) as ::core::ffi::c_int);
            sub_nlines.set(1 as ::core::ffi::c_int as linenr_T);
            do_sub_msg(false_0 != 0);
            ex_may_print(eap);
        }
        if save {
            if !keeppatterns {
                save_re_pat(RE_SUBST as ::core::ffi::c_int, pat, patlen, magic_isset());
            }
            add_to_history(
                HIST_SEARCH as ::core::ffi::c_int,
                ::core::slice::from_raw_parts(pat as *const u8, patlen as usize),
                true_0 != 0,
                NUL as u8,
            );
        }
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn sub_grow_buf(
    mut new_start: *mut *mut ::core::ffi::c_char,
    mut new_start_len: *mut ::core::ffi::c_int,
    mut needed_len: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut new_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*new_start).is_null() {
        *new_start_len = needed_len + 50 as ::core::ffi::c_int;
        *new_start = xcalloc(1 as size_t, *new_start_len as size_t) as *mut ::core::ffi::c_char;
        **new_start = NUL as ::core::ffi::c_char;
        new_end = *new_start;
    } else {
        let mut len: size_t = strlen(*new_start);
        needed_len += len as ::core::ffi::c_int;
        if needed_len > *new_start_len {
            let mut prev_new_start_len: size_t = *new_start_len as size_t;
            *new_start_len = needed_len + 50 as ::core::ffi::c_int;
            let mut added_len: size_t = (*new_start_len as size_t).wrapping_sub(prev_new_start_len);
            *new_start = xrealloc(
                *new_start as *mut ::core::ffi::c_void,
                *new_start_len as size_t,
            ) as *mut ::core::ffi::c_char;
            memset(
                (*new_start).offset(prev_new_start_len as isize) as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                added_len,
            );
        }
        new_end = (*new_start).offset(len as isize);
    }
    return new_end;
}
unsafe extern "C" fn sub_parse_flags(
    mut cmd: *mut ::core::ffi::c_char,
    mut subflags: *mut subflags_T,
    mut which_pat: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if *cmd as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
        cmd = cmd.offset(1);
    } else {
        (*subflags).do_all = p_gd.get() != 0;
        (*subflags).do_ask = false_0 != 0;
        (*subflags).do_error = true_0 != 0;
        (*subflags).do_print = false_0 != 0;
        (*subflags).do_list = false_0 != 0;
        (*subflags).do_count = false_0 != 0;
        (*subflags).do_number = false_0 != 0;
        (*subflags).do_ic = kSubHonorOptions;
    }
    while *cmd != 0 {
        if *cmd as ::core::ffi::c_int == 'g' as ::core::ffi::c_int {
            (*subflags).do_all = !(*subflags).do_all;
        } else if *cmd as ::core::ffi::c_int == 'c' as ::core::ffi::c_int {
            (*subflags).do_ask = !(*subflags).do_ask;
        } else if *cmd as ::core::ffi::c_int == 'n' as ::core::ffi::c_int {
            (*subflags).do_count = true_0 != 0;
        } else if *cmd as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
            (*subflags).do_error = !(*subflags).do_error;
        } else if *cmd as ::core::ffi::c_int == 'r' as ::core::ffi::c_int {
            *which_pat = RE_LAST as ::core::ffi::c_int;
        } else if *cmd as ::core::ffi::c_int == 'p' as ::core::ffi::c_int {
            (*subflags).do_print = true_0 != 0;
        } else if *cmd as ::core::ffi::c_int == '#' as ::core::ffi::c_int {
            (*subflags).do_print = true_0 != 0;
            (*subflags).do_number = true_0 != 0;
        } else if *cmd as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
            (*subflags).do_print = true_0 != 0;
            (*subflags).do_list = true_0 != 0;
        } else if *cmd as ::core::ffi::c_int == 'i' as ::core::ffi::c_int {
            (*subflags).do_ic = kSubIgnoreCase;
        } else {
            if *cmd as ::core::ffi::c_int != 'I' as ::core::ffi::c_int {
                break;
            }
            (*subflags).do_ic = kSubMatchCase;
        }
        cmd = cmd.offset(1);
    }
    if (*subflags).do_count {
        (*subflags).do_ask = false_0 != 0;
    }
    return cmd;
}
unsafe extern "C" fn skip_substitute(
    mut start: *mut ::core::ffi::c_char,
    mut delimiter: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = start;
    while *p.offset(0 as ::core::ffi::c_int as isize) != 0 {
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == delimiter {
            let c2rust_fresh12 = p;
            p = p.offset(1);
            *c2rust_fresh12 = NUL as ::core::ffi::c_char;
            break;
        } else {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            p = p.offset(utfc_ptr2len(p) as isize);
        }
    }
    return p;
}
unsafe extern "C" fn check_regexp_delim(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if *(*__ctype_b_loc()).offset(c as isize) as ::core::ffi::c_int
        & _ISalpha as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        emsg(gettext(
            b"E146: Regular expressions can't be delimited by letters\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn do_sub(
    mut eap: *mut exarg_T,
    timeout: proftime_T,
    cmdpreview_ns: ::core::ffi::c_int,
    cmdpreview_bufnr: handle_T,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut regmatch: regmmatch_T = regmmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    };
    static subflags: GlobalCell<subflags_T> = GlobalCell::new(subflags_T {
        do_all: false_0 != 0,
        do_ask: false_0 != 0,
        do_count: false_0 != 0,
        do_error: true_0 != 0,
        do_print: false_0 != 0,
        do_list: false_0 != 0,
        do_number: false_0 != 0,
        do_ic: kSubHonorOptions,
    });
    let mut pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut sub: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut patlen: size_t = 0 as size_t;
    let mut delimiter: ::core::ffi::c_int = 0;
    let mut has_second_delim: bool = false_0 != 0;
    let mut sublen: ::core::ffi::c_int = 0;
    let mut got_quit: bool = false_0 != 0;
    let mut got_match: bool = false_0 != 0;
    let mut which_pat: ::core::ffi::c_int = 0;
    let mut cmd: *mut ::core::ffi::c_char = (*eap).arg;
    let mut first_line: linenr_T = 0 as linenr_T;
    let mut last_line: linenr_T = 0 as linenr_T;
    let mut old_line_count: linenr_T = (*curbuf.get()).b_ml.ml_line_count;
    let mut sub_firstline: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut endcolumn: bool = false_0 != 0;
    let keeppatterns: bool =
        (*cmdmod.ptr()).cmod_flags & CMOD_KEEPPATTERNS as ::core::ffi::c_int != 0;
    let mut preview_lines: PreviewLines = PreviewLines {
        subresults: C2Rust_Unnamed_33 {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<SubResult>(),
        },
        lines_needed: 0 as linenr_T,
    };
    static pre_hl_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    let mut old_cursor: pos_T = (*curwin.get()).w_cursor;
    let mut start_nsubs: ::core::ffi::c_int = 0;
    let mut did_save: bool = false_0 != 0;
    if global_busy.get() == 0 {
        sub_nsubs.set(0 as ::core::ffi::c_int);
        sub_nlines.set(0 as ::core::ffi::c_int as linenr_T);
    }
    start_nsubs = sub_nsubs.get();
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_tilde as ::core::ffi::c_int {
        which_pat = RE_LAST as ::core::ffi::c_int;
    } else {
        which_pat = RE_SUBST as ::core::ffi::c_int;
    }
    if *(*eap).cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 's' as ::core::ffi::c_int
        && *cmd as ::core::ffi::c_int != NUL
        && !ascii_iswhite(*cmd as ::core::ffi::c_int)
        && vim_strchr(
            b"0123456789cegriIp|\"\0".as_ptr() as *const ::core::ffi::c_char,
            *cmd as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
    {
        if check_regexp_delim(*cmd as ::core::ffi::c_int) == FAIL {
            return 0 as ::core::ffi::c_int;
        }
        if *cmd as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
            cmd = cmd.offset(1);
            if vim_strchr(
                b"/?&\0".as_ptr() as *const ::core::ffi::c_char,
                *cmd as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
            {
                emsg(gettext(
                    &raw const e_backslash as *const ::core::ffi::c_char,
                ));
                return 0 as ::core::ffi::c_int;
            }
            if *cmd as ::core::ffi::c_int != '&' as ::core::ffi::c_int {
                which_pat = RE_SEARCH as ::core::ffi::c_int;
            }
            pat = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            patlen = 0 as size_t;
            let c2rust_fresh6 = cmd;
            cmd = cmd.offset(1);
            delimiter = *c2rust_fresh6 as uint8_t as ::core::ffi::c_int;
            has_second_delim = true_0 != 0;
        } else {
            which_pat = RE_LAST as ::core::ffi::c_int;
            let c2rust_fresh7 = cmd;
            cmd = cmd.offset(1);
            delimiter = *c2rust_fresh7 as uint8_t as ::core::ffi::c_int;
            pat = cmd;
            cmd = skip_regexp_ex(
                cmd,
                delimiter,
                magic_isset() as ::core::ffi::c_int,
                &raw mut (*eap).arg,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ::core::ptr::null_mut::<magic_T>(),
            );
            if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == delimiter {
                let c2rust_fresh8 = cmd;
                cmd = cmd.offset(1);
                *c2rust_fresh8 = NUL as ::core::ffi::c_char;
                has_second_delim = true_0 != 0;
            }
            patlen = strlen(pat);
        }
        let mut p: *mut ::core::ffi::c_char = cmd;
        cmd = skip_substitute(cmd, delimiter);
        sub = xstrdup(p);
        if (*eap).skip == 0 && !keeppatterns && cmdpreview_ns <= 0 as ::core::ffi::c_int {
            sub_set_replacement(SubReplacementString {
                sub: xstrdup(sub),
                timestamp: os_time(),
                additional_data: ::core::ptr::null_mut::<AdditionalData>(),
            });
        }
    } else if (*eap).skip == 0 {
        if (*old_sub.ptr()).sub.is_null() {
            emsg(gettext(&raw const e_nopresub as *const ::core::ffi::c_char));
            return 0 as ::core::ffi::c_int;
        }
        pat = ::core::ptr::null_mut::<::core::ffi::c_char>();
        patlen = 0 as size_t;
        sub = xstrdup((*old_sub.ptr()).sub);
        endcolumn = (*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int;
    }
    if !sub.is_null()
        && sub_joining_lines(
            eap,
            pat,
            patlen,
            sub,
            cmd,
            cmdpreview_ns <= 0 as ::core::ffi::c_int,
            keeppatterns,
        ) as ::core::ffi::c_int
            != 0
    {
        xfree(sub as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    cmd = sub_parse_flags(cmd, subflags.ptr(), &raw mut which_pat);
    let mut save_do_all: bool = (*subflags.ptr()).do_all;
    let mut save_do_ask: bool = (*subflags.ptr()).do_ask;
    cmd = skipwhite(cmd);
    if ascii_isdigit(*cmd as ::core::ffi::c_int) {
        let count_arg: *const ::core::ffi::c_char = cmd;
        i = getdigits_int(&raw mut cmd, false_0 != 0, INT_MAX);
        if i <= 0 as ::core::ffi::c_int
            && (*eap).skip == 0
            && (*subflags.ptr()).do_error as ::core::ffi::c_int != 0
        {
            emsg(gettext(
                &raw const e_zerocount as *const ::core::ffi::c_char,
            ));
            xfree(sub as *mut ::core::ffi::c_void);
            return 0 as ::core::ffi::c_int;
        } else if i == INT_MAX {
            semsg(
                gettext(&raw const e_val_too_large_len as *const ::core::ffi::c_char),
                cmd.offset_from(count_arg) as ::core::ffi::c_int,
                count_arg,
            );
            xfree(sub as *mut ::core::ffi::c_void);
            return 0 as ::core::ffi::c_int;
        }
        (*eap).line1 = (*eap).line2;
        (*eap).line2 = ((*eap).line2 as ::core::ffi::c_int
            + (i as linenr_T - 1 as linenr_T) as ::core::ffi::c_int)
            as linenr_T;
        (*eap).line2 = if (*eap).line2 < (*curbuf.get()).b_ml.ml_line_count {
            (*eap).line2
        } else {
            (*curbuf.get()).b_ml.ml_line_count
        };
    }
    cmd = skipwhite(cmd);
    if *cmd as ::core::ffi::c_int != 0 && *cmd as ::core::ffi::c_int != '"' as ::core::ffi::c_int {
        (*eap).nextcmd = check_nextcmd(cmd);
        if (*eap).nextcmd.is_null() {
            semsg(
                gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                cmd,
            );
            xfree(sub as *mut ::core::ffi::c_void);
            return 0 as ::core::ffi::c_int;
        }
    }
    if (*eap).skip != 0 {
        xfree(sub as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if !(*subflags.ptr()).do_count && (*curbuf.get()).b_p_ma == 0 {
        emsg(gettext(
            &raw const e_modifiable as *const ::core::ffi::c_char,
        ));
        xfree(sub as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if search_regcomp(
        pat,
        patlen,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        RE_SUBST as ::core::ffi::c_int,
        which_pat,
        if cmdpreview_ns > 0 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            SEARCH_HIS as ::core::ffi::c_int
        },
        &raw mut regmatch,
    ) == FAIL
    {
        if (*subflags.ptr()).do_error {
            emsg(gettext(&raw const e_invcmd as *const ::core::ffi::c_char));
        }
        xfree(sub as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if (*subflags.ptr()).do_ic as ::core::ffi::c_uint
        == kSubIgnoreCase as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        regmatch.rmm_ic = true_0;
    } else if (*subflags.ptr()).do_ic as ::core::ffi::c_uint
        == kSubMatchCase as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        regmatch.rmm_ic = false_0;
    }
    sub_firstline = ::core::ptr::null_mut::<::core::ffi::c_char>();
    '_c2rust_label: {
        if !sub.is_null() {
        } else {
            __assert_fail(
                b"sub != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ex_cmds.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3738 as ::core::ffi::c_uint,
                b"int do_sub(exarg_T *, const proftime_T, const int, const handle_T)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if *sub.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '\\' as ::core::ffi::c_int
        && *sub.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '=' as ::core::ffi::c_int
    {
        let mut p_0: *mut ::core::ffi::c_char = xstrdup(sub);
        xfree(sub as *mut ::core::ffi::c_void);
        sub = p_0;
    } else {
        let mut p_1: *mut ::core::ffi::c_char = regtilde(
            sub,
            magic_isset() as ::core::ffi::c_int,
            cmdpreview_ns > 0 as ::core::ffi::c_int,
        );
        if p_1 != sub {
            xfree(sub as *mut ::core::ffi::c_void);
            sub = p_1;
        }
    }
    let mut line2: linenr_T = (*eap).line2;
    let mut lnum: linenr_T = (*eap).line1;
    while lnum <= line2
        && !got_quit
        && !aborting()
        && (cmdpreview_ns <= 0 as ::core::ffi::c_int
            || preview_lines.lines_needed <= p_cwh.get() as linenr_T
            || lnum <= (*curwin.get()).w_botline)
    {
        let mut nmatch: ::core::ffi::c_int = vim_regexec_multi(
            &raw mut regmatch,
            curwin.get(),
            curbuf.get(),
            lnum,
            0 as colnr_T,
            ::core::ptr::null_mut::<proftime_T>(),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        if nmatch != 0 {
            let mut copycol: colnr_T = 0;
            let mut matchcol: colnr_T = 0;
            let mut prev_matchcol: colnr_T = MAXCOL as ::core::ffi::c_int;
            let mut new_end: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut new_start: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut new_start_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut p1: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut did_sub: bool = false_0 != 0;
            let mut lastone: ::core::ffi::c_int = 0;
            let mut nmatch_tl: linenr_T = 0 as linenr_T;
            let mut do_again: ::core::ffi::c_int = 0;
            let mut skip_match: bool = false_0 != 0;
            let mut sub_firstlnum: linenr_T = 0;
            let mut lnum_start: linenr_T = 0 as linenr_T;
            let mut line_matches: C2Rust_Unnamed_34 = C2Rust_Unnamed_34 {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<LineData>(),
            };
            sub_firstlnum = lnum;
            copycol = 0 as ::core::ffi::c_int as colnr_T;
            matchcol = 0 as ::core::ffi::c_int as colnr_T;
            if !got_match {
                setpcmark();
                got_match = true_0 != 0;
            }
            loop {
                let mut current_match: SubResult = SubResult {
                    start: lpos_T {
                        lnum: 0 as linenr_T,
                        col: 0 as colnr_T,
                    },
                    end: lpos_T {
                        lnum: 0 as linenr_T,
                        col: 0 as colnr_T,
                    },
                    pre_match: 0 as linenr_T,
                };
                if regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum > 0 as linenr_T {
                    current_match.pre_match = lnum;
                    lnum += regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum;
                    sub_firstlnum += regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum;
                    nmatch -= regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum
                        as ::core::ffi::c_int;
                    let mut ptr_: *mut *mut ::core::ffi::c_void =
                        &raw mut sub_firstline as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL_0;
                    let _ = *ptr_;
                }
                current_match.start.lnum = sub_firstlnum;
                if lnum > (*curbuf.get()).b_ml.ml_line_count {
                    break;
                }
                if sub_firstline.is_null() {
                    sub_firstline =
                        xstrnsave(ml_get(sub_firstlnum), ml_get_len(sub_firstlnum) as size_t);
                }
                (*curwin.get()).w_cursor.lnum = lnum;
                do_again = false_0;
                '_skip: {
                    if matchcol == prev_matchcol
                        && regmatch.endpos[0 as ::core::ffi::c_int as usize].lnum == 0 as linenr_T
                        && matchcol == regmatch.endpos[0 as ::core::ffi::c_int as usize].col
                    {
                        if *sub_firstline.offset(matchcol as isize) as ::core::ffi::c_int == NUL {
                            skip_match = true_0 != 0;
                        } else {
                            matchcol += utfc_ptr2len(sub_firstline.offset(matchcol as isize));
                        }
                        current_match.start.col = matchcol;
                        current_match.end.lnum = sub_firstlnum;
                        current_match.end.col = matchcol;
                    } else {
                        matchcol = regmatch.endpos[0 as ::core::ffi::c_int as usize].col;
                        prev_matchcol = matchcol;
                        if (*subflags.ptr()).do_count {
                            if nmatch > 1 as ::core::ffi::c_int {
                                matchcol = strlen(sub_firstline) as colnr_T;
                                nmatch = 1 as ::core::ffi::c_int;
                                skip_match = true_0 != 0;
                            }
                            (*sub_nsubs.ptr()) += 1;
                            did_sub = true_0 != 0;
                            if !(*sub.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == '\\' as ::core::ffi::c_int
                                && *sub.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == '=' as ::core::ffi::c_int)
                            {
                                break '_skip;
                            }
                        }
                        if (*subflags.ptr()).do_ask as ::core::ffi::c_int != 0
                            && cmdpreview_ns <= 0 as ::core::ffi::c_int
                        {
                            let mut typed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            let mut save_State: ::core::ffi::c_int = State.get();
                            (*curwin.get()).w_cursor.col =
                                regmatch.startpos[0 as ::core::ffi::c_int as usize].col;
                            if (*curwin.get()).w_onebuf_opt.wo_crb != 0 {
                                do_check_cursorbind();
                            }
                            if !vim_strchr(p_cpo.get(), CPO_UNDO).is_null() {
                                (*no_u_sync.ptr()) += 1;
                            }
                            while (*subflags.ptr()).do_ask {
                                if exmode_active.get() {
                                    print_line_no_prefix(
                                        lnum,
                                        (*subflags.ptr()).do_number,
                                        (*subflags.ptr()).do_list,
                                    );
                                    let mut sc: colnr_T = 0;
                                    let mut ec: colnr_T = 0;
                                    getvcol(
                                        curwin.get(),
                                        &raw mut (*curwin.get()).w_cursor,
                                        &raw mut sc,
                                        ::core::ptr::null_mut::<colnr_T>(),
                                        ::core::ptr::null_mut::<colnr_T>(),
                                    );
                                    (*curwin.get()).w_cursor.col =
                                        (if regmatch.endpos[0 as ::core::ffi::c_int as usize].col
                                            as ::core::ffi::c_int
                                            - 1 as ::core::ffi::c_int
                                            > 0 as ::core::ffi::c_int
                                        {
                                            regmatch.endpos[0 as ::core::ffi::c_int as usize].col
                                                as ::core::ffi::c_int
                                                - 1 as ::core::ffi::c_int
                                        } else {
                                            0 as ::core::ffi::c_int
                                        }) as colnr_T;
                                    getvcol(
                                        curwin.get(),
                                        &raw mut (*curwin.get()).w_cursor,
                                        ::core::ptr::null_mut::<colnr_T>(),
                                        ::core::ptr::null_mut::<colnr_T>(),
                                        &raw mut ec,
                                    );
                                    (*curwin.get()).w_cursor.col =
                                        regmatch.startpos[0 as ::core::ffi::c_int as usize].col;
                                    if (*subflags.ptr()).do_number as ::core::ffi::c_int != 0
                                        || (*curwin.get()).w_onebuf_opt.wo_nu != 0
                                    {
                                        let mut numw: ::core::ffi::c_int =
                                            number_width(curwin.get()) + 1 as ::core::ffi::c_int;
                                        sc += numw;
                                        ec += numw;
                                    }
                                    let mut prompt: *mut ::core::ffi::c_char =
                                        xmallocz((ec as size_t).wrapping_add(1 as size_t))
                                            as *mut ::core::ffi::c_char;
                                    memset(
                                        prompt as *mut ::core::ffi::c_void,
                                        ' ' as ::core::ffi::c_int,
                                        sc as size_t,
                                    );
                                    memset(
                                        prompt.offset(sc as isize) as *mut ::core::ffi::c_void,
                                        '^' as ::core::ffi::c_int,
                                        ((ec - sc) as size_t).wrapping_add(1 as size_t),
                                    );
                                    let mut resp: *mut ::core::ffi::c_char = getcmdline_prompt(
                                        -1 as ::core::ffi::c_int,
                                        prompt,
                                        0 as ::core::ffi::c_int,
                                        EXPAND_NOTHING as ::core::ffi::c_int,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        Callback {
                                            data: C2Rust_Unnamed_5 {
                                                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(
                                                ),
                                            },
                                            type_0: kCallbackNone,
                                        },
                                        false_0 != 0,
                                        ::core::ptr::null_mut::<bool>(),
                                    );
                                    if !ui_has(kUIMessages) {
                                        msg_putchar('\n' as ::core::ffi::c_int);
                                    }
                                    xfree(prompt as *mut ::core::ffi::c_void);
                                    if !resp.is_null() {
                                        typed = *resp as uint8_t as ::core::ffi::c_int;
                                        xfree(resp as *mut ::core::ffi::c_void);
                                    } else {
                                        typed = NUL;
                                    }
                                    if ex_normal_busy.get() != 0 && typed == NUL {
                                        typed = 'q' as ::core::ffi::c_int;
                                    }
                                } else {
                                    let mut orig_line: *mut ::core::ffi::c_char =
                                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                                    let mut len_change: ::core::ffi::c_int =
                                        0 as ::core::ffi::c_int;
                                    let save_p_lz: bool = p_lz.get() != 0;
                                    let mut save_p_fen: ::core::ffi::c_int =
                                        (*curwin.get()).w_onebuf_opt.wo_fen;
                                    (*curwin.get()).w_onebuf_opt.wo_fen = false_0;
                                    let mut temp: ::core::ffi::c_int = RedrawingDisabled.get();
                                    RedrawingDisabled.set(0 as ::core::ffi::c_int);
                                    p_lz.set(false_0);
                                    if !new_start.is_null() {
                                        orig_line =
                                            xstrnsave(ml_get(lnum), ml_get_len(lnum) as size_t);
                                        let mut new_line: *mut ::core::ffi::c_char = concat_str(
                                            new_start,
                                            sub_firstline.offset(copycol as isize),
                                        );
                                        len_change = strlen(new_line) as ::core::ffi::c_int
                                            - strlen(orig_line) as ::core::ffi::c_int;
                                        (*curwin.get()).w_cursor.col += len_change;
                                        ml_replace(lnum, new_line, false_0 != 0);
                                    }
                                    search_match_lines.set(
                                        regmatch.endpos[0 as ::core::ffi::c_int as usize].lnum
                                            - regmatch.startpos[0 as ::core::ffi::c_int as usize]
                                                .lnum,
                                    );
                                    search_match_endcol.set(
                                        (regmatch.endpos[0 as ::core::ffi::c_int as usize].col
                                            as ::core::ffi::c_int
                                            + len_change)
                                            as colnr_T,
                                    );
                                    if search_match_lines.get() == 0 as linenr_T
                                        && search_match_endcol.get() == 0 as ::core::ffi::c_int
                                    {
                                        search_match_endcol.set(1 as ::core::ffi::c_int as colnr_T);
                                    }
                                    highlight_match.set(true_0 != 0);
                                    update_topline(curwin.get());
                                    validate_cursor(curwin.get());
                                    redraw_later(curwin.get(), UPD_SOME_VALID);
                                    show_cursor_info_later(true_0 != 0);
                                    update_screen();
                                    redraw_later(curwin.get(), UPD_SOME_VALID);
                                    (*curwin.get()).w_onebuf_opt.wo_fen = save_p_fen;
                                    let mut p_2: *mut ::core::ffi::c_char = gettext(
                                        b"replace with %s? (y)es/(n)o/(a)ll/(q)uit/(l)ast/scroll up(^E)/down(^Y)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                    snprintf(
                                        IObuff.ptr() as *mut ::core::ffi::c_char,
                                        IOSIZE as size_t,
                                        p_2,
                                        sub,
                                    );
                                    p_2 = xstrdup(IObuff.ptr() as *mut ::core::ffi::c_char);
                                    typed = prompt_for_input(
                                        p_2,
                                        HLF_R,
                                        true_0 != 0,
                                        ::core::ptr::null_mut::<bool>(),
                                    );
                                    highlight_match.set(false_0 != 0);
                                    xfree(p_2 as *mut ::core::ffi::c_void);
                                    msg_didout.set(false_0 != 0);
                                    gotocmdline(true_0 != 0);
                                    p_lz.set(save_p_lz as ::core::ffi::c_int);
                                    RedrawingDisabled.set(temp);
                                    if !orig_line.is_null() {
                                        ml_replace(lnum, orig_line, false_0 != 0);
                                    }
                                }
                                need_wait_return.set(false_0 != 0);
                                if typed == 'q' as ::core::ffi::c_int
                                    || typed == ESC
                                    || typed == Ctrl_C
                                {
                                    got_quit = true_0 != 0;
                                    break;
                                } else {
                                    if typed == 'n' as ::core::ffi::c_int {
                                        break;
                                    }
                                    if typed == 'y' as ::core::ffi::c_int {
                                        break;
                                    }
                                    if typed == 'l' as ::core::ffi::c_int {
                                        (*subflags.ptr()).do_all = false_0 != 0;
                                        line2 = lnum;
                                        break;
                                    } else if typed == 'a' as ::core::ffi::c_int {
                                        (*subflags.ptr()).do_ask = false_0 != 0;
                                        break;
                                    } else if typed == Ctrl_E {
                                        scrollup_clamp();
                                    } else if typed == Ctrl_Y {
                                        scrolldown_clamp();
                                    }
                                }
                            }
                            State.set(save_State);
                            setmouse();
                            if !vim_strchr(p_cpo.get(), CPO_UNDO).is_null() {
                                (*no_u_sync.ptr()) -= 1;
                            }
                            if typed == 'n' as ::core::ffi::c_int {
                                if nmatch > 1 as ::core::ffi::c_int {
                                    matchcol = strlen(sub_firstline) as colnr_T;
                                    skip_match = true_0 != 0;
                                }
                                break '_skip;
                            } else if got_quit {
                                break '_skip;
                            }
                        }
                        (*curwin.get()).w_cursor.col =
                            regmatch.startpos[0 as ::core::ffi::c_int as usize].col;
                        if nmatch as linenr_T
                            > (*curbuf.get()).b_ml.ml_line_count - sub_firstlnum + 1 as linenr_T
                        {
                            nmatch = ((*curbuf.get()).b_ml.ml_line_count - sub_firstlnum
                                + 1 as linenr_T)
                                as ::core::ffi::c_int;
                            current_match.end.lnum = sub_firstlnum + nmatch as linenr_T;
                            skip_match = true_0 != 0;
                            if nmatch < 0 as ::core::ffi::c_int {
                                break '_skip;
                            }
                        }
                        if cmdpreview_ns > 0 as ::core::ffi::c_int && !has_second_delim {
                            current_match.start.col =
                                regmatch.startpos[0 as ::core::ffi::c_int as usize].col;
                            if current_match.end.lnum == 0 as linenr_T {
                                current_match.end.lnum =
                                    sub_firstlnum + nmatch as linenr_T - 1 as linenr_T;
                            }
                            current_match.end.col =
                                regmatch.endpos[0 as ::core::ffi::c_int as usize].col;
                            if nmatch > 1 as ::core::ffi::c_int {
                                sub_firstlnum = (sub_firstlnum as ::core::ffi::c_int
                                    + (nmatch as linenr_T - 1 as linenr_T) as ::core::ffi::c_int)
                                    as linenr_T;
                                xfree(sub_firstline as *mut ::core::ffi::c_void);
                                sub_firstline = xstrnsave(
                                    ml_get(sub_firstlnum),
                                    ml_get_len(sub_firstlnum) as size_t,
                                );
                                if sub_firstlnum <= line2 {
                                    do_again = true_0;
                                } else {
                                    (*subflags.ptr()).do_all = false_0 != 0;
                                }
                            }
                            if skip_match {
                                xfree(sub_firstline as *mut ::core::ffi::c_void);
                                sub_firstline =
                                    xstrdup(b"\0".as_ptr() as *const ::core::ffi::c_char);
                                copycol = 0 as ::core::ffi::c_int as colnr_T;
                            }
                            lnum = (lnum as ::core::ffi::c_int
                                + (nmatch as linenr_T - 1 as linenr_T) as ::core::ffi::c_int)
                                as linenr_T;
                        } else if cmdpreview_ns <= 0 as ::core::ffi::c_int
                            || has_second_delim as ::core::ffi::c_int != 0
                        {
                            lnum_start = lnum;
                            let mut save_ma: ::core::ffi::c_int = (*curbuf.get()).b_p_ma;
                            let mut save_sandbox: ::core::ffi::c_int = sandbox.get();
                            if (*subflags.ptr()).do_count {
                                (*curbuf.get()).b_p_ma = false_0;
                                (*sandbox.ptr()) += 1;
                            }
                            let mut subflags_save: subflags_T = subflags.get();
                            (*textlock.ptr()) += 1;
                            sublen = vim_regsub_multi(
                                &raw mut regmatch,
                                sub_firstlnum
                                    - regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum,
                                sub,
                                sub_firstline,
                                0 as ::core::ffi::c_int,
                                REGSUB_BACKSLASH as ::core::ffi::c_int
                                    | (if magic_isset() as ::core::ffi::c_int != 0 {
                                        REGSUB_MAGIC as ::core::ffi::c_int
                                    } else {
                                        0 as ::core::ffi::c_int
                                    }),
                            );
                            (*textlock.ptr()) -= 1;
                            subflags.set(subflags_save);
                            if sublen == 0 as ::core::ffi::c_int
                                || aborting() as ::core::ffi::c_int != 0
                                || (*subflags.ptr()).do_count as ::core::ffi::c_int != 0
                            {
                                (*curbuf.get()).b_p_ma = save_ma;
                                sandbox.set(save_sandbox);
                            } else {
                                if nmatch == 1 as ::core::ffi::c_int {
                                    p1 = sub_firstline;
                                } else {
                                    let mut lastlnum: linenr_T =
                                        sub_firstlnum + nmatch as linenr_T - 1 as linenr_T;
                                    p1 = ml_get(lastlnum);
                                    nmatch_tl = (nmatch_tl as ::core::ffi::c_int
                                        + (nmatch - 1 as ::core::ffi::c_int))
                                        as linenr_T;
                                }
                                let mut copy_len: ::core::ffi::c_int =
                                    regmatch.startpos[0 as ::core::ffi::c_int as usize].col
                                        as ::core::ffi::c_int
                                        - copycol as ::core::ffi::c_int;
                                new_end = sub_grow_buf(
                                    &raw mut new_start,
                                    &raw mut new_start_len,
                                    strlen(p1) as ::core::ffi::c_int
                                        - regmatch.endpos[0 as ::core::ffi::c_int as usize].col
                                            as ::core::ffi::c_int
                                        + copy_len
                                        + sublen
                                        + 1 as ::core::ffi::c_int,
                                );
                                memmove(
                                    new_end as *mut ::core::ffi::c_void,
                                    sub_firstline.offset(copycol as isize)
                                        as *const ::core::ffi::c_void,
                                    copy_len as size_t,
                                );
                                new_end = new_end.offset(copy_len as isize);
                                if new_start_len - copy_len < sublen {
                                    sublen = new_start_len - copy_len - 1 as ::core::ffi::c_int;
                                }
                                let mut start_col: ::core::ffi::c_int =
                                    new_end.offset_from(new_start) as ::core::ffi::c_int;
                                current_match.start.col = start_col as colnr_T;
                                (*textlock.ptr()) += 1;
                                vim_regsub_multi(
                                    &raw mut regmatch,
                                    sub_firstlnum
                                        - regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum,
                                    sub,
                                    new_end,
                                    sublen,
                                    REGSUB_COPY as ::core::ffi::c_int
                                        | REGSUB_BACKSLASH as ::core::ffi::c_int
                                        | (if magic_isset() as ::core::ffi::c_int != 0 {
                                            REGSUB_MAGIC as ::core::ffi::c_int
                                        } else {
                                            0 as ::core::ffi::c_int
                                        }),
                                );
                                (*textlock.ptr()) -= 1;
                                (*sub_nsubs.ptr()) += 1;
                                did_sub = true_0 != 0;
                                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                                copycol = regmatch.endpos[0 as ::core::ffi::c_int as usize].col;
                                if nmatch > 1 as ::core::ffi::c_int {
                                    sub_firstlnum = (sub_firstlnum as ::core::ffi::c_int
                                        + (nmatch as linenr_T - 1 as linenr_T)
                                            as ::core::ffi::c_int)
                                        as linenr_T;
                                    xfree(sub_firstline as *mut ::core::ffi::c_void);
                                    sub_firstline = xstrnsave(
                                        ml_get(sub_firstlnum),
                                        ml_get_len(sub_firstlnum) as size_t,
                                    );
                                    if sub_firstlnum <= line2 {
                                        do_again = true_0;
                                    } else {
                                        (*subflags.ptr()).do_all = false_0 != 0;
                                    }
                                }
                                if skip_match {
                                    xfree(sub_firstline as *mut ::core::ffi::c_void);
                                    sub_firstline =
                                        xstrdup(b"\0".as_ptr() as *const ::core::ffi::c_char);
                                    copycol = 0 as ::core::ffi::c_int as colnr_T;
                                }
                                let mut replaced_bytes: bcount_t = 0 as bcount_t;
                                let mut start: lpos_T =
                                    regmatch.startpos[0 as ::core::ffi::c_int as usize];
                                let mut end: lpos_T =
                                    regmatch.endpos[0 as ::core::ffi::c_int as usize];
                                i = 0 as ::core::ffi::c_int;
                                while i < nmatch - 1 as ::core::ffi::c_int {
                                    replaced_bytes += strlen(ml_get(lnum_start + i as linenr_T))
                                        as bcount_t
                                        + 1 as bcount_t;
                                    i += 1;
                                }
                                replaced_bytes += (end.col - start.col) as bcount_t;
                                let mut lnum_before_newlines: linenr_T = lnum;
                                p1 = new_end;
                                while *p1 != 0 {
                                    if *p1.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == '\\' as ::core::ffi::c_int
                                        && *p1.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            != NUL
                                    {
                                        sublen -= 1;
                                        memmove(
                                            p1 as *mut ::core::ffi::c_void,
                                            p1.offset(1 as ::core::ffi::c_int as isize)
                                                as *const ::core::ffi::c_void,
                                            strlen(p1.offset(1 as ::core::ffi::c_int as isize))
                                                .wrapping_add(1 as size_t),
                                        );
                                    } else if *p1 as ::core::ffi::c_int == CAR {
                                        if u_inssub(lnum) == OK {
                                            *p1 = NUL as ::core::ffi::c_char;
                                            ml_append(
                                                lnum - 1 as linenr_T,
                                                new_start,
                                                (p1.offset_from(new_start) + 1 as isize) as colnr_T,
                                                false_0 != 0,
                                            );
                                            mark_adjust(
                                                lnum + 1 as linenr_T,
                                                MAXLNUM as ::core::ffi::c_int as linenr_T,
                                                1 as linenr_T,
                                                0 as linenr_T,
                                                kExtmarkNOOP,
                                            );
                                            if (*subflags.ptr()).do_ask {
                                                appended_lines(lnum - 1 as linenr_T, 1 as linenr_T);
                                            } else {
                                                if first_line == 0 as linenr_T {
                                                    first_line = lnum;
                                                }
                                                last_line = lnum + 1 as linenr_T;
                                            }
                                            sub_firstlnum += 1;
                                            lnum += 1;
                                            line2 += 1;
                                            (*curwin.get()).w_cursor.lnum += 1;
                                            memmove(
                                                new_start as *mut ::core::ffi::c_void,
                                                p1.offset(1 as ::core::ffi::c_int as isize)
                                                    as *const ::core::ffi::c_void,
                                                strlen(p1.offset(1 as ::core::ffi::c_int as isize))
                                                    .wrapping_add(1 as size_t),
                                            );
                                            p1 = new_start
                                                .offset(-(1 as ::core::ffi::c_int as isize));
                                        }
                                    } else {
                                        p1 = p1.offset(
                                            (utfc_ptr2len(p1) - 1 as ::core::ffi::c_int) as isize,
                                        );
                                    }
                                    p1 = p1.offset(1);
                                }
                                let mut new_endcol: colnr_T = strlen(new_start) as colnr_T;
                                current_match.end.col = new_endcol;
                                current_match.end.lnum = lnum;
                                let mut matchcols: ::core::ffi::c_int = end.col
                                    as ::core::ffi::c_int
                                    - (if end.lnum == start.lnum {
                                        start.col as ::core::ffi::c_int
                                    } else {
                                        0 as ::core::ffi::c_int
                                    });
                                let mut subcols: ::core::ffi::c_int = new_endcol
                                    as ::core::ffi::c_int
                                    - (if lnum == lnum_start {
                                        start_col
                                    } else {
                                        0 as ::core::ffi::c_int
                                    });
                                if !did_save {
                                    u_save_cursor();
                                    did_save = true_0 != 0;
                                }
                                if line_matches.size == line_matches.capacity {
                                    line_matches.capacity = if line_matches.capacity != 0 {
                                        line_matches.capacity << 1 as ::core::ffi::c_int
                                    } else {
                                        8 as size_t
                                    };
                                    line_matches.items = xrealloc(
                                        line_matches.items as *mut ::core::ffi::c_void,
                                        ::core::mem::size_of::<LineData>()
                                            .wrapping_mul(line_matches.capacity),
                                    )
                                        as *mut LineData;
                                } else {
                                };
                                let c2rust_fresh9 = line_matches.size;
                                line_matches.size = line_matches.size.wrapping_add(1);
                                let mut data: *mut LineData =
                                    line_matches.items.offset(c2rust_fresh9 as isize);
                                (*data).start_col = start_col;
                                (*data).start = start;
                                (*data).end = end;
                                (*data).matchcols = matchcols;
                                (*data).matchbytes = replaced_bytes;
                                (*data).subcols = subcols;
                                (*data).subbytes = (sublen - 1 as ::core::ffi::c_int) as bcount_t;
                                (*data).lnum_before = lnum_before_newlines;
                                (*data).lnum_after = lnum;
                            }
                        }
                    }
                }
                lastone = (skip_match as ::core::ffi::c_int != 0
                    || got_int.get() as ::core::ffi::c_int != 0
                    || got_quit as ::core::ffi::c_int != 0
                    || lnum > line2
                    || !((*subflags.ptr()).do_all as ::core::ffi::c_int != 0 || do_again != 0)
                    || *sub_firstline.offset(matchcol as isize) as ::core::ffi::c_int == NUL
                        && nmatch <= 1 as ::core::ffi::c_int
                        && re_multiline(regmatch.regprog) == 0)
                    as ::core::ffi::c_int;
                nmatch = -1 as ::core::ffi::c_int;
                if lastone != 0
                    || nmatch_tl > 0 as linenr_T
                    || {
                        nmatch = vim_regexec_multi(
                            &raw mut regmatch,
                            curwin.get(),
                            curbuf.get(),
                            sub_firstlnum,
                            matchcol,
                            ::core::ptr::null_mut::<proftime_T>(),
                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        );
                        nmatch == 0 as ::core::ffi::c_int
                    }
                    || regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum > 0 as linenr_T
                {
                    if !new_start.is_null() {
                        strcat(new_start, sub_firstline.offset(copycol as isize));
                        matchcol = strlen(sub_firstline) as colnr_T - matchcol;
                        prev_matchcol = strlen(sub_firstline) as colnr_T - prev_matchcol;
                        if u_savesub(lnum) != OK {
                            break;
                        }
                        ml_replace(lnum, new_start, true_0 != 0);
                        let mut match_idx: size_t = 0 as size_t;
                        while match_idx < line_matches.size {
                            let mut match_0: *mut LineData =
                                line_matches.items.offset(match_idx as isize);
                            extmark_splice(
                                curbuf.get(),
                                (*match_0).lnum_before as ::core::ffi::c_int
                                    - 1 as ::core::ffi::c_int,
                                (*match_0).start_col as colnr_T,
                                (*match_0).end.lnum as ::core::ffi::c_int
                                    - (*match_0).start.lnum as ::core::ffi::c_int,
                                (*match_0).matchcols as colnr_T,
                                (*match_0).matchbytes,
                                (*match_0).lnum_after as ::core::ffi::c_int
                                    - (*match_0).lnum_before as ::core::ffi::c_int,
                                (*match_0).subcols as colnr_T,
                                (*match_0).subbytes,
                                kExtmarkUndo,
                            );
                            match_idx = match_idx.wrapping_add(1);
                        }
                        line_matches.size = 0 as size_t;
                        if nmatch_tl > 0 as linenr_T {
                            lnum += 1;
                            if u_savedel(lnum, nmatch_tl) != OK {
                                break;
                            }
                            i = 0 as ::core::ffi::c_int;
                            while (i as linenr_T) < nmatch_tl {
                                ml_delete(lnum);
                                i += 1;
                            }
                            mark_adjust(
                                lnum,
                                lnum + nmatch_tl - 1 as linenr_T,
                                MAXLNUM as ::core::ffi::c_int as linenr_T,
                                -nmatch_tl,
                                kExtmarkNOOP,
                            );
                            if (*subflags.ptr()).do_ask {
                                deleted_lines(lnum, nmatch_tl);
                            }
                            lnum -= 1;
                            line2 -= nmatch_tl;
                            nmatch_tl = 0 as ::core::ffi::c_int as linenr_T;
                        }
                        if (*subflags.ptr()).do_ask {
                            changed_bytes(lnum, 0 as colnr_T);
                        } else {
                            if first_line == 0 as linenr_T {
                                first_line = lnum;
                            }
                            last_line = lnum + 1 as linenr_T;
                        }
                        sub_firstlnum = lnum;
                        xfree(sub_firstline as *mut ::core::ffi::c_void);
                        sub_firstline = new_start;
                        new_start = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        matchcol = strlen(sub_firstline) as colnr_T - matchcol;
                        prev_matchcol = strlen(sub_firstline) as colnr_T - prev_matchcol;
                        copycol = 0 as ::core::ffi::c_int as colnr_T;
                    }
                    if nmatch == -1 as ::core::ffi::c_int && lastone == 0 {
                        nmatch = vim_regexec_multi(
                            &raw mut regmatch,
                            curwin.get(),
                            curbuf.get(),
                            sub_firstlnum,
                            matchcol,
                            ::core::ptr::null_mut::<proftime_T>(),
                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        );
                    }
                    if nmatch <= 0 as ::core::ffi::c_int {
                        if nmatch == -1 as ::core::ffi::c_int {
                            lnum -= regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum;
                        }
                        if cmdpreview_ns > 0 as ::core::ffi::c_int {
                            let mut match_lines: linenr_T =
                                current_match.end.lnum - current_match.start.lnum + 1 as linenr_T;
                            if preview_lines.subresults.size > 0 as size_t {
                                let mut last: linenr_T = (*preview_lines.subresults.items.offset(
                                    preview_lines
                                        .subresults
                                        .size
                                        .wrapping_sub(0 as size_t)
                                        .wrapping_sub(1 as size_t)
                                        as isize,
                                ))
                                .end
                                .lnum;
                                if last == current_match.start.lnum {
                                    preview_lines.lines_needed = (preview_lines.lines_needed
                                        as ::core::ffi::c_int
                                        + (match_lines - 1 as linenr_T) as ::core::ffi::c_int)
                                        as linenr_T;
                                } else {
                                    preview_lines.lines_needed += match_lines;
                                }
                            } else {
                                preview_lines.lines_needed += match_lines;
                            }
                            if preview_lines.subresults.size == preview_lines.subresults.capacity {
                                preview_lines.subresults.capacity =
                                    if preview_lines.subresults.capacity != 0 {
                                        preview_lines.subresults.capacity << 1 as ::core::ffi::c_int
                                    } else {
                                        8 as size_t
                                    };
                                preview_lines.subresults.items = xrealloc(
                                    preview_lines.subresults.items as *mut ::core::ffi::c_void,
                                    ::core::mem::size_of::<SubResult>()
                                        .wrapping_mul(preview_lines.subresults.capacity),
                                )
                                    as *mut SubResult;
                            } else {
                            };
                            let c2rust_fresh10 = preview_lines.subresults.size;
                            preview_lines.subresults.size =
                                preview_lines.subresults.size.wrapping_add(1);
                            *preview_lines
                                .subresults
                                .items
                                .offset(c2rust_fresh10 as isize) = current_match;
                        }
                        break;
                    }
                }
                if cmdpreview_ns > 0 as ::core::ffi::c_int {
                    let mut match_lines_0: linenr_T =
                        current_match.end.lnum - current_match.start.lnum + 1 as linenr_T;
                    if preview_lines.subresults.size > 0 as size_t {
                        let mut last_0: linenr_T = (*preview_lines.subresults.items.offset(
                            preview_lines
                                .subresults
                                .size
                                .wrapping_sub(0 as size_t)
                                .wrapping_sub(1 as size_t) as isize,
                        ))
                        .end
                        .lnum;
                        if last_0 == current_match.start.lnum {
                            preview_lines.lines_needed = (preview_lines.lines_needed
                                as ::core::ffi::c_int
                                + (match_lines_0 - 1 as linenr_T) as ::core::ffi::c_int)
                                as linenr_T;
                        } else {
                            preview_lines.lines_needed += match_lines_0;
                        }
                    } else {
                        preview_lines.lines_needed += match_lines_0;
                    }
                    if preview_lines.subresults.size == preview_lines.subresults.capacity {
                        preview_lines.subresults.capacity =
                            if preview_lines.subresults.capacity != 0 {
                                preview_lines.subresults.capacity << 1 as ::core::ffi::c_int
                            } else {
                                8 as size_t
                            };
                        preview_lines.subresults.items = xrealloc(
                            preview_lines.subresults.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<SubResult>()
                                .wrapping_mul(preview_lines.subresults.capacity),
                        )
                            as *mut SubResult;
                    } else {
                    };
                    let c2rust_fresh11 = preview_lines.subresults.size;
                    preview_lines.subresults.size = preview_lines.subresults.size.wrapping_add(1);
                    *preview_lines
                        .subresults
                        .items
                        .offset(c2rust_fresh11 as isize) = current_match;
                }
                line_breakcheck();
            }
            if did_sub {
                (*sub_nlines.ptr()) += 1;
            }
            xfree(new_start as *mut ::core::ffi::c_void);
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut sub_firstline as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL_0;
            let _ = *ptr__0;
            xfree(line_matches.items as *mut ::core::ffi::c_void);
            line_matches.capacity = 0 as size_t;
            line_matches.size = line_matches.capacity;
            line_matches.items = ::core::ptr::null_mut::<LineData>();
        }
        line_breakcheck();
        if profile_passed_limit(timeout) {
            got_quit = true_0 != 0;
        }
        lnum += 1;
    }
    (*curbuf.get()).deleted_bytes2 = 0 as size_t;
    if first_line != 0 as linenr_T {
        i = ((*curbuf.get()).b_ml.ml_line_count - old_line_count) as ::core::ffi::c_int;
        changed_lines(
            curbuf.get(),
            first_line,
            0 as colnr_T,
            last_line - i as linenr_T,
            i as linenr_T,
            false_0 != 0,
        );
        let mut num_added: int64_t = (last_line - first_line) as int64_t;
        let mut num_removed: int64_t = num_added - i as int64_t;
        buf_updates_send_changes(curbuf.get(), first_line, num_added, num_removed);
    }
    xfree(sub_firstline as *mut ::core::ffi::c_void);
    if (*subflags.ptr()).do_count {
        (*curwin.get()).w_cursor = old_cursor;
    }
    if sub_nsubs.get() > start_nsubs {
        if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*curbuf.get()).b_op_start.lnum = (*eap).line1;
            (*curbuf.get()).b_op_end.lnum = line2;
            (*curbuf.get()).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
            (*curbuf.get()).b_op_start.col = (*curbuf.get()).b_op_end.col;
        }
        if global_busy.get() == 0 {
            if !(*subflags.ptr()).do_ask {
                if endcolumn {
                    coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
                } else {
                    beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
                }
            }
            if cmdpreview_ns <= 0 as ::core::ffi::c_int
                && !do_sub_msg((*subflags.ptr()).do_count)
                && (*subflags.ptr()).do_ask as ::core::ffi::c_int != 0
                && p_ch.get() > 0 as OptInt
            {
                msg(
                    b"\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
            }
        } else {
            global_need_beginline.set(true_0);
        }
        if (*subflags.ptr()).do_print {
            print_line(
                (*curwin.get()).w_cursor.lnum,
                (*subflags.ptr()).do_number,
                (*subflags.ptr()).do_list,
                true_0 != 0,
            );
        }
    } else if global_busy.get() == 0 {
        if got_int.get() {
            emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
        } else if got_match {
            if p_ch.get() > 0 as OptInt && !ui_has(kUIMessages) {
                msg(
                    b"\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
            }
        } else if (*subflags.ptr()).do_error {
            semsg(
                gettext(&raw const e_patnotf2 as *const ::core::ffi::c_char),
                get_search_pat(),
            );
        }
    }
    if (*subflags.ptr()).do_ask as ::core::ffi::c_int != 0 && hasAnyFolding(curwin.get()) != 0 {
        changed_window_setting(curwin.get());
    }
    vim_regfree(regmatch.regprog);
    xfree(sub as *mut ::core::ffi::c_void);
    (*subflags.ptr()).do_all = save_do_all;
    (*subflags.ptr()).do_ask = save_do_ask;
    let mut retv: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if cmdpreview_ns > 0 as ::core::ffi::c_int && !aborting() {
        if got_quit as ::core::ffi::c_int != 0
            || profile_passed_limit(timeout) as ::core::ffi::c_int != 0
        {
            set_option_direct(
                kOptInccommand,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: String_0 {
                            data: b"\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 1]>()
                                .wrapping_sub(1 as size_t),
                        },
                    },
                },
                0 as ::core::ffi::c_int,
                SID_NONE,
            );
        } else if *p_icm.get() as ::core::ffi::c_int != NUL && !pat.is_null() {
            if pre_hl_id.get() == 0 as ::core::ffi::c_int {
                pre_hl_id.set(syn_check_group(
                    b"Substitute\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
                ));
            }
            retv = show_sub(
                eap,
                old_cursor,
                &raw mut preview_lines,
                pre_hl_id.get(),
                cmdpreview_ns,
                cmdpreview_bufnr,
            );
        }
    }
    xfree(preview_lines.subresults.items as *mut ::core::ffi::c_void);
    preview_lines.subresults.capacity = 0 as size_t;
    preview_lines.subresults.size = preview_lines.subresults.capacity;
    preview_lines.subresults.items = ::core::ptr::null_mut::<SubResult>();
    return retv;
}
pub unsafe extern "C" fn do_sub_msg(mut count_only: bool) -> bool {
    if (sub_nsubs.get() as OptInt > p_report.get()
        && (KeyTyped.get() as ::core::ffi::c_int != 0
            || sub_nlines.get() > 1 as linenr_T
            || p_report.get() < 1 as OptInt)
        || count_only as ::core::ffi::c_int != 0)
        && messaging() as ::core::ffi::c_int != 0
    {
        if got_int.get() {
            strcpy(
                msg_buf.ptr() as *mut ::core::ffi::c_char,
                gettext(b"(Interrupted) \0".as_ptr() as *const ::core::ffi::c_char),
            );
        } else {
            *(msg_buf.ptr() as *mut ::core::ffi::c_char) = NUL as ::core::ffi::c_char;
        }
        let mut msg_single: *mut ::core::ffi::c_char = if count_only as ::core::ffi::c_int != 0 {
            ngettext(
                b"%ld match on %ld line\0".as_ptr() as *const ::core::ffi::c_char,
                b"%ld matches on %ld line\0".as_ptr() as *const ::core::ffi::c_char,
                sub_nsubs.get() as ::core::ffi::c_ulong,
            )
        } else {
            ngettext(
                b"%ld substitution on %ld line\0".as_ptr() as *const ::core::ffi::c_char,
                b"%ld substitutions on %ld line\0".as_ptr() as *const ::core::ffi::c_char,
                sub_nsubs.get() as ::core::ffi::c_ulong,
            )
        };
        let mut msg_plural: *mut ::core::ffi::c_char = if count_only as ::core::ffi::c_int != 0 {
            ngettext(
                b"%ld match on %ld lines\0".as_ptr() as *const ::core::ffi::c_char,
                b"%ld matches on %ld lines\0".as_ptr() as *const ::core::ffi::c_char,
                sub_nsubs.get() as ::core::ffi::c_ulong,
            )
        } else {
            ngettext(
                b"%ld substitution on %ld lines\0".as_ptr() as *const ::core::ffi::c_char,
                b"%ld substitutions on %ld lines\0".as_ptr() as *const ::core::ffi::c_char,
                sub_nsubs.get() as ::core::ffi::c_ulong,
            )
        };
        vim_snprintf_add(
            msg_buf.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 480]>(),
            ngettext(
                msg_single,
                msg_plural,
                sub_nlines.get() as ::core::ffi::c_ulong,
            ),
            sub_nsubs.get() as int64_t,
            sub_nlines.get() as int64_t,
        );
        if msg(
            msg_buf.ptr() as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        ) {
            set_keep_msg(
                msg_buf.ptr() as *mut ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        }
        return true_0 != 0;
    }
    if got_int.get() {
        emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn global_exe_one(cmd: *mut ::core::ffi::c_char, lnum: linenr_T) {
    (*curwin.get()).w_cursor.lnum = lnum;
    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    if *cmd as ::core::ffi::c_int == NUL || *cmd as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
    {
        do_cmdline(
            b"p\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            None,
            NULL_0,
            DOCMD_NOWAIT as ::core::ffi::c_int,
        );
    } else {
        do_cmdline(cmd, None, NULL_0, DOCMD_NOWAIT as ::core::ffi::c_int);
    };
}
pub unsafe fn ex_global(mut eap: *mut exarg_T) {
    let mut lnum: linenr_T = 0;
    let mut type_0: ::core::ffi::c_int = 0;
    let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut delim: ::core::ffi::c_char = 0;
    let mut pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut patlen: size_t = 0;
    let mut regmatch: regmmatch_T = regmmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    };
    if global_busy.get() != 0
        && ((*eap).line1 != 1 as linenr_T || (*eap).line2 != (*curbuf.get()).b_ml.ml_line_count)
    {
        emsg(gettext(
            b"E147: Cannot do :global recursive with a range\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        return;
    }
    if (*eap).forceit != 0 {
        type_0 = 'v' as ::core::ffi::c_int;
    } else {
        type_0 = *(*eap).cmd as uint8_t as ::core::ffi::c_int;
    }
    cmd = (*eap).arg;
    let mut which_pat: ::core::ffi::c_int = RE_LAST as ::core::ffi::c_int;
    if *cmd as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
        cmd = cmd.offset(1);
        if vim_strchr(
            b"/?&\0".as_ptr() as *const ::core::ffi::c_char,
            *cmd as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            emsg(gettext(
                &raw const e_backslash as *const ::core::ffi::c_char,
            ));
            return;
        }
        if *cmd as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
            which_pat = RE_SUBST as ::core::ffi::c_int;
        } else {
            which_pat = RE_SEARCH as ::core::ffi::c_int;
        }
        cmd = cmd.offset(1);
        pat = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        patlen = 0 as size_t;
    } else if *cmd as ::core::ffi::c_int == NUL {
        emsg(gettext(
            b"E148: Regular expression missing from global\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        return;
    } else if check_regexp_delim(*cmd as ::core::ffi::c_int) == FAIL {
        return;
    } else {
        delim = *cmd;
        cmd = cmd.offset(1);
        pat = cmd;
        cmd = skip_regexp_ex(
            cmd,
            delim as ::core::ffi::c_int,
            magic_isset() as ::core::ffi::c_int,
            &raw mut (*eap).arg,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ::core::ptr::null_mut::<magic_T>(),
        );
        if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == delim as ::core::ffi::c_int
        {
            let c2rust_fresh5 = cmd;
            cmd = cmd.offset(1);
            *c2rust_fresh5 = NUL as ::core::ffi::c_char;
        }
        patlen = strlen(pat);
    }
    let mut used_pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if search_regcomp(
        pat,
        patlen,
        &raw mut used_pat,
        RE_BOTH as ::core::ffi::c_int,
        which_pat,
        SEARCH_HIS as ::core::ffi::c_int,
        &raw mut regmatch,
    ) == FAIL
    {
        emsg(gettext(&raw const e_invcmd as *const ::core::ffi::c_char));
        return;
    }
    if global_busy.get() != 0 {
        lnum = (*curwin.get()).w_cursor.lnum;
        let mut match_0: ::core::ffi::c_int = vim_regexec_multi(
            &raw mut regmatch,
            curwin.get(),
            curbuf.get(),
            lnum,
            0 as colnr_T,
            ::core::ptr::null_mut::<proftime_T>(),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        if type_0 == 'g' as ::core::ffi::c_int && match_0 != 0
            || type_0 == 'v' as ::core::ffi::c_int && match_0 == 0
        {
            global_exe_one(cmd, lnum);
        }
    } else {
        let mut ndone: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        lnum = (*eap).line1;
        while lnum <= (*eap).line2 && !got_int.get() {
            let mut match_1: ::core::ffi::c_int = vim_regexec_multi(
                &raw mut regmatch,
                curwin.get(),
                curbuf.get(),
                lnum,
                0 as colnr_T,
                ::core::ptr::null_mut::<proftime_T>(),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            );
            if regmatch.regprog.is_null() {
                break;
            }
            if type_0 == 'g' as ::core::ffi::c_int && match_1 != 0
                || type_0 == 'v' as ::core::ffi::c_int && match_1 == 0
            {
                ml_setmarked(lnum);
                ndone += 1;
            }
            line_breakcheck();
            lnum += 1;
        }
        if got_int.get() {
            msg(
                gettext(&raw const e_interr as *const ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
            );
        } else if ndone == 0 as ::core::ffi::c_int {
            if type_0 == 'v' as ::core::ffi::c_int {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(
                        b"Pattern found in every line: %s\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    used_pat,
                );
            } else {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"Pattern not found: %s\0".as_ptr() as *const ::core::ffi::c_char),
                    used_pat,
                );
            }
        } else {
            global_exe(cmd);
        }
        ml_clearmarked();
    }
    vim_regfree(regmatch.regprog);
}
pub unsafe extern "C" fn global_exe(mut cmd: *mut ::core::ffi::c_char) {
    let mut old_lcount: linenr_T = 0;
    let mut old_buf: *mut buf_T = curbuf.get();
    let mut lnum: linenr_T = 0;
    setpcmark();
    msg_didout.set(true_0 != 0);
    sub_nsubs.set(0 as ::core::ffi::c_int);
    sub_nlines.set(0 as ::core::ffi::c_int as linenr_T);
    global_need_msg_kind.set(true_0 != 0);
    global_need_beginline.set(false_0);
    global_busy.set(1 as ::core::ffi::c_int);
    old_lcount = (*curbuf.get()).b_ml.ml_line_count;
    while !got_int.get()
        && {
            lnum = ml_firstmarked();
            lnum != 0 as linenr_T
        }
        && global_busy.get() == 1 as ::core::ffi::c_int
    {
        global_exe_one(cmd, lnum);
        os_breakcheck();
    }
    global_busy.set(0 as ::core::ffi::c_int);
    if global_need_beginline.get() != 0 {
        beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    } else {
        check_cursor(curwin.get());
    }
    changed_line_abv_curs();
    if msg_col.get() == 0 as ::core::ffi::c_int && msg_scrolled.get() == 0 as ::core::ffi::c_int {
        msg_didout.set(false_0 != 0);
    }
    if !do_sub_msg(false_0 != 0) && curbuf.get() == old_buf {
        msgmore(
            (*curbuf.get()).b_ml.ml_line_count as ::core::ffi::c_int
                - old_lcount as ::core::ffi::c_int,
        );
    }
}
pub unsafe extern "C" fn prepare_tagpreview(mut undo_sync: bool) -> bool {
    if (*curwin.get()).w_onebuf_opt.wo_pvw != 0 {
        return false_0 != 0;
    }
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_onebuf_opt.wo_pvw != 0 {
            win_enter(wp, undo_sync);
            return false_0 != 0;
        }
        wp = (*wp).w_next;
    }
    if win_split(
        if g_do_tagpreview.get() > 0 as ::core::ffi::c_int {
            g_do_tagpreview.get()
        } else {
            0 as ::core::ffi::c_int
        },
        0 as ::core::ffi::c_int,
    ) == FAIL
    {
        return false_0 != 0;
    }
    (*curwin.get()).w_onebuf_opt.wo_pvw = true_0;
    (*curwin.get()).w_onebuf_opt.wo_wfh = true_0;
    (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
    (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
    (*curwin.get()).w_onebuf_opt.wo_diff = false_0;
    set_option_direct(
        kOptFoldcolumn,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"0\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        0 as ::core::ffi::c_int,
        SID_NONE,
    );
    return true_0 != 0;
}
unsafe extern "C" fn show_sub(
    mut eap: *mut exarg_T,
    mut old_cusr: pos_T,
    mut preview_lines: *mut PreviewLines,
    mut hl_id: ::core::ffi::c_int,
    mut cmdpreview_ns: ::core::ffi::c_int,
    mut cmdpreview_bufnr: handle_T,
) -> ::core::ffi::c_int {
    let mut save_shm_p: *mut ::core::ffi::c_char = xstrdup(p_shm.get());
    let mut lines: PreviewLines = *preview_lines;
    let mut orig_buf: *mut buf_T = curbuf.get();
    let mut cmdpreview_buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    set_option_direct(
        kOptShortmess,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"F\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        0 as ::core::ffi::c_int,
        SID_NONE,
    );
    let mut i: size_t = 0 as size_t;
    while i < lines.subresults.size {
        let mut curres: SubResult = *lines.subresults.items.offset(i as isize);
        if curres.start.lnum >= old_cusr.lnum {
            (*curwin.get()).w_cursor.lnum = curres.start.lnum;
            (*curwin.get()).w_cursor.col = curres.start.col;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    update_topline(curwin.get());
    let mut col_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut preview: bool = *p_icm.get() as ::core::ffi::c_int == 's' as ::core::ffi::c_int
        && ((*eap).line1 != old_cusr.lnum || (*eap).line2 != old_cusr.lnum);
    if preview {
        cmdpreview_buf = buflist_findnr(cmdpreview_bufnr as ::core::ffi::c_int);
        '_c2rust_label: {
            if !cmdpreview_buf.is_null() {
            } else {
                __assert_fail(
                    b"cmdpreview_buf != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ex_cmds.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    4886 as ::core::ffi::c_uint,
                    b"int show_sub(exarg_T *, pos_T, PreviewLines *, int, int, handle_T)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if lines.subresults.size > 0 as size_t {
            let mut last_match: SubResult = *lines.subresults.items.offset(
                lines
                    .subresults
                    .size
                    .wrapping_sub(0 as size_t)
                    .wrapping_sub(1 as size_t) as isize,
            );
            let mut highest_lnum: linenr_T = if last_match.start.lnum > last_match.end.lnum {
                last_match.start.lnum
            } else {
                last_match.end.lnum
            };
            '_c2rust_label_0: {
                if highest_lnum > 0 as linenr_T {
                } else {
                    __assert_fail(
                        b"highest_lnum > 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/ex_cmds.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        4892 as ::core::ffi::c_uint,
                        b"int show_sub(exarg_T *, pos_T, PreviewLines *, int, int, handle_T)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            col_width = log10(highest_lnum as ::core::ffi::c_double) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 3 as ::core::ffi::c_int;
        }
    }
    let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut old_line_size: colnr_T = 0 as colnr_T;
    let mut line_size: colnr_T = 0 as colnr_T;
    let mut linenr_preview: linenr_T = 0 as linenr_T;
    let mut linenr_origbuf: linenr_T = 0 as linenr_T;
    let mut next_linenr: linenr_T = 0 as linenr_T;
    let mut matchidx: size_t = 0 as size_t;
    while matchidx < lines.subresults.size {
        let mut match_0: SubResult = *lines.subresults.items.offset(matchidx as isize);
        if !cmdpreview_buf.is_null() {
            let mut p_start: lpos_T = lpos_T {
                lnum: 0 as linenr_T,
                col: match_0.start.col,
            };
            let mut p_end: lpos_T = lpos_T {
                lnum: 0 as linenr_T,
                col: match_0.end.col,
            };
            buf_ensure_loaded(cmdpreview_buf);
            if match_0.pre_match == 0 as linenr_T {
                next_linenr = match_0.start.lnum;
            } else {
                next_linenr = match_0.pre_match;
            }
            if next_linenr == linenr_origbuf {
                next_linenr += 1;
                p_start.lnum = linenr_preview;
                p_end.lnum = linenr_preview;
            }
            while next_linenr <= match_0.end.lnum {
                if next_linenr == match_0.start.lnum {
                    p_start.lnum = linenr_preview + 1 as linenr_T;
                }
                if next_linenr == match_0.end.lnum {
                    p_end.lnum = linenr_preview + 1 as linenr_T;
                }
                let mut line: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                if next_linenr == (*orig_buf).b_ml.ml_line_count + 1 as linenr_T {
                    line = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    line = ml_get_buf(orig_buf, next_linenr);
                    line_size = (ml_get_buf_len(orig_buf, next_linenr)
                        + col_width
                        + 1 as ::core::ffi::c_int) as colnr_T;
                    if line_size > old_line_size {
                        str = xrealloc(
                            str as *mut ::core::ffi::c_void,
                            (line_size as size_t)
                                .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                        ) as *mut ::core::ffi::c_char;
                        old_line_size = line_size;
                    }
                }
                snprintf(
                    str,
                    line_size as size_t,
                    b"|%*d| %s\0".as_ptr() as *const ::core::ffi::c_char,
                    col_width - 3 as ::core::ffi::c_int,
                    next_linenr,
                    line,
                );
                if linenr_preview == 0 as linenr_T {
                    ml_replace_buf(
                        cmdpreview_buf,
                        1 as linenr_T,
                        str,
                        true_0 != 0,
                        false_0 != 0,
                    );
                } else {
                    ml_append_buf(cmdpreview_buf, linenr_preview, str, line_size, false_0 != 0);
                }
                linenr_preview =
                    (linenr_preview as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as linenr_T;
                next_linenr += 1;
            }
            linenr_origbuf = match_0.end.lnum;
            bufhl_add_hl_pos_offset(
                cmdpreview_buf,
                cmdpreview_ns,
                hl_id,
                p_start,
                p_end,
                col_width as colnr_T,
            );
        }
        bufhl_add_hl_pos_offset(
            orig_buf,
            cmdpreview_ns,
            hl_id,
            match_0.start,
            match_0.end,
            0 as colnr_T,
        );
        matchidx = matchidx.wrapping_add(1);
    }
    xfree(str as *mut ::core::ffi::c_void);
    set_option_direct(
        kOptShortmess,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(save_shm_p),
            },
        },
        0 as ::core::ffi::c_int,
        SID_NONE,
    );
    xfree(save_shm_p as *mut ::core::ffi::c_void);
    return if preview as ::core::ffi::c_int != 0 {
        2 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
}
pub unsafe fn ex_substitute(mut eap: *mut exarg_T) {
    do_sub(eap, profile_zero(), 0 as ::core::ffi::c_int, 0 as handle_T);
}
pub unsafe fn ex_substitute_preview(
    mut eap: *mut exarg_T,
    mut cmdpreview_ns: ::core::ffi::c_int,
    mut cmdpreview_bufnr: handle_T,
) -> ::core::ffi::c_int {
    if *(*eap).arg as ::core::ffi::c_int != 0
        && !(*(*eap).arg as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *(*eap).arg as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *(*eap).arg as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *(*eap).arg as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(*(*eap).arg as ::core::ffi::c_int) as ::core::ffi::c_int != 0)
    {
        let mut save_eap: *mut ::core::ffi::c_char = (*eap).arg;
        let mut retv: ::core::ffi::c_int = do_sub(
            eap,
            profile_setlimit(p_rdt.get() as int64_t),
            cmdpreview_ns,
            cmdpreview_bufnr,
        );
        (*eap).arg = save_eap;
        return retv;
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn skip_vimgrep_pat(
    mut p: *mut ::core::ffi::c_char,
    mut s: *mut *mut ::core::ffi::c_char,
    mut flags: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if vim_isIDc(*p as uint8_t as ::core::ffi::c_int) {
        if !s.is_null() {
            *s = p;
        }
        p = skiptowhite(p);
        if !s.is_null() && *p as ::core::ffi::c_int != NUL {
            let c2rust_fresh13 = p;
            p = p.offset(1);
            *c2rust_fresh13 = NUL as ::core::ffi::c_char;
        }
    } else {
        if !s.is_null() {
            *s = p.offset(1 as ::core::ffi::c_int as isize);
        }
        let mut c: ::core::ffi::c_int = *p as uint8_t as ::core::ffi::c_int;
        p = skip_regexp(p.offset(1 as ::core::ffi::c_int as isize), c, true_0);
        if *p as ::core::ffi::c_int != c {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if !s.is_null() {
            *p = NUL as ::core::ffi::c_char;
        }
        p = p.offset(1);
        while *p as ::core::ffi::c_int == 'g' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == 'j' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == 'f' as ::core::ffi::c_int
        {
            if !flags.is_null() {
                if *p as ::core::ffi::c_int == 'g' as ::core::ffi::c_int {
                    *flags |= VGR_GLOBAL as ::core::ffi::c_int;
                } else if *p as ::core::ffi::c_int == 'j' as ::core::ffi::c_int {
                    *flags |= VGR_NOJUMP as ::core::ffi::c_int;
                } else {
                    *flags |= VGR_FUZZY as ::core::ffi::c_int;
                }
            }
            p = p.offset(1);
        }
    }
    return p;
}
pub unsafe fn ex_oldfiles(mut eap: *mut exarg_T) {
    let mut l: *mut list_T = get_vim_var_list(VV_OLDFILES);
    let mut nr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if l.is_null() {
        msg(
            gettext(b"No old files\0".as_ptr() as *const ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
        return;
    }
    msg_start();
    msg_scroll.set(true_0);
    let l_: *mut list_T = l;
    if !l_.is_null() {
        let mut li: *mut listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if got_int.get() {
                break;
            }
            nr += 1;
            let mut fname: *const ::core::ffi::c_char = tv_get_string(&raw mut (*li).li_tv);
            if !message_filtered(fname) {
                msg_outnum(nr);
                msg_puts(b": \0".as_ptr() as *const ::core::ffi::c_char);
                msg_outtrans(
                    tv_get_string(&raw mut (*li).li_tv),
                    0 as ::core::ffi::c_int,
                    false,
                );
                msg_clr_eos();
                msg_putchar('\n' as ::core::ffi::c_int);
                os_breakcheck();
            }
            li = (*li).li_next;
        }
    }
    got_int.set(false_0 != 0);
    if (*cmdmod.ptr()).cmod_flags & CMOD_BROWSE as ::core::ffi::c_int != 0 {
        quit_more.set(false_0 != 0);
        nr = prompt_for_input(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            false_0 != 0,
            ::core::ptr::null_mut::<bool>(),
        );
        msg_starthere();
        if nr > 0 as ::core::ffi::c_int && nr <= tv_list_len(l) {
            let p: *const ::core::ffi::c_char = tv_list_find_str(l, nr - 1 as ::core::ffi::c_int);
            if p.is_null() {
                return;
            }
            let s: *mut ::core::ffi::c_char = expand_env_save(p as *mut ::core::ffi::c_char);
            (*eap).arg = s;
            (*eap).cmdidx = CMD_edit;
            (*cmdmod.ptr()).cmod_flags &= !(CMOD_BROWSE as ::core::ffi::c_int);
            do_exedit(eap, ::core::ptr::null_mut::<win_T>());
            xfree(s as *mut ::core::ffi::c_void);
        }
    }
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const SID_NONE: ::core::ffi::c_int = -6 as ::core::ffi::c_int;
pub const SEA_DIALOG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SEA_QUIT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const DBL_MAX: ::core::ffi::c_double = __DBL_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const __DBL_MAX__: ::core::ffi::c_double = 1.7976931348623157e+308f64;
