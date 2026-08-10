//! The Ex commands that act on the buffer's *text*.
//!
//! Carved by what the command does to it:
//!
//! | child | what |
//! | --- | --- |
//! | [`text`] | `:left`/`:right`/`:center` and `:ascii` |
//! | [`sort`] | `:sort` and `:uniq` |
//! | [`lines`] | `:move` and `:copy` |
//! | [`filter`] | `:!`, `:range!`, `:shell` and `:print` |
//! | [`write`] | `:write`/`:update`/`:wall`/`:wq` and their guards |
//! | [`ecmd`] | `do_ecmd`: every command that changes which file a window shows |
//! | [`append`] | `:append`/`:insert`/`:change`/`:z` |
//! | [`global`] | `:global`/`:vglobal` |
//! | [`subst`] | `:substitute`, split again around its 1,220-line engine |
//!
//! What stays here is what the children share -- the flag constants, the
//! `sorti_T`/`SubResult`/`LineData` layouts, and `check_secure`,
//! `prepare_tagpreview`, `skip_vimgrep_pat` and `ex_oldfiles`, four helpers
//! that belong to no one command and that other modules import by name.
//!
//! The family's process-wide state lives with the code that drives it:
//! `prevcmd` and `global_need_msg_kind` in [`filter`], the seven `sort_*`
//! flags in [`sort`], `append_indent` in [`append`], and `old_sub` plus
//! `global_need_beginline` in [`subst`] -- the last two are the only ones
//! read from outside their own child.

#![deny(unsafe_op_in_unsafe_fn)]

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
    getdigits_int, skiptobin, skiptodigit, skiptohex, skiptowhite, skipwhite, vim_isIDc, vim_str2nr,
};
use crate::src::nvim::cmdhist::add_to_history;
use crate::src::nvim::cursor::{
    check_cursor, check_cursor_col, check_cursor_lnum, coladvance, get_cursor_line_ptr,
};
use crate::src::nvim::decoration::bufhl_add_hl_pos_offset;
use crate::src::nvim::diff::{diff_buf_add, diff_invalidate};
use crate::src::nvim::digraph::keymap_init;
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
use crate::src::nvim::extmark::extmark_splice;
use crate::src::nvim::fileio::{
    buf_check_timestamp, readfile, set_file_options, set_forced_fenc, vim_tempname,
    write_lnum_adjust,
};
use crate::src::nvim::fold::{foldUpdate, foldUpdateAll, hasAnyFolding};
use crate::src::nvim::getchar::{AppendToRedobuff, AppendToRedobuffLit};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::help::prepare_help_buffer;
use crate::src::nvim::highlight_group::{HLF_N, HLF_R, syn_check_group};
use crate::src::nvim::indent::get_indent_lnum;
use crate::src::nvim::input::prompt_for_input;
use crate::src::nvim::keycodes::{Ctrl_C, Ctrl_E, Ctrl_Y};
use crate::src::nvim::main::{
    Columns, IObuff, KeyTyped, RedrawingDisabled, Rows, State, au_new_curbuf, autocmd_busy,
    bangredo, cmdmod, cmdwin_buf, cmdwin_old_curwin, cmdwin_type, cmdwin_win, curbuf, curtab,
    curwin, did_check_timestamps, e_argreq, e_backslash, e_bufloaded,
    e_cannot_switch_to_a_closing_buffer, e_cant_read_file_str, e_curdir, e_exists, e_interr,
    e_invarg, e_invarg2, e_invcmd, e_isadir2, e_modifiable, e_nopresub, e_noprev, e_noprevre,
    e_notmp, e_patnotf2, e_readonly, e_sandbox, e_trailing_arg, e_val_too_large_len, e_zerocount,
    emsg_silent, ex_no_reprint, ex_normal_busy, exiting, exmode_active, first_tabpage, firstbuf,
    firstwin, g_do_tagpreview, getout, global_busy, got_int, highlight_match, info_message,
    keep_help_flag, lastwin, lines_left, msg_buf, msg_col, msg_didout, msg_listdo_overwrite,
    msg_row, msg_scroll, msg_scrolled, msg_scrolled_ign, msg_silent, need_check_timestamps,
    need_wait_return, no_u_sync, no_wait_return, p_awa, p_ch, p_confirm, p_cpo, p_cwh, p_dir, p_gd,
    p_ic, p_icm, p_lz, p_rdt, p_report, p_sh, p_shm, p_shq, p_so, p_sol, p_srr, p_stmp, p_ur,
    p_verbose, p_wa, p_warn, p_window, p_write, quit_more, redraw_tabline, sandbox,
    search_match_endcol, search_match_lines, secure, silent_mode, skip_redraw, sub_nlines,
    sub_nsubs, swap_exists_action, textlock,
};
use crate::src::nvim::mark::{mark_adjust, set_last_cursor, setpcmark};
use crate::src::nvim::mbyte::utfc_ptr2len;
use crate::src::nvim::memline::{
    makeswapname, ml_append, ml_append_buf, ml_clearmarked, ml_delete, ml_firstmarked, ml_get,
    ml_get_buf, ml_get_buf_len, ml_get_len, ml_replace, ml_replace_buf, ml_setmarked,
};
use crate::src::nvim::memory::{
    xcalloc, xfree, xmalloc, xmallocz, xmemdupz, xrealloc, xstrdup, xstrlcat, xstrlcpy,
};
use crate::src::nvim::message::{
    emsg, message_filtered, messaging, msg, msg_check_for_delay, msg_clr_eos, msg_end,
    msg_ext_set_kind, msg_outnum, msg_outtrans, msg_prt_line, msg_putchar, msg_puts, msg_puts_hl,
    msg_start, msg_starthere, msgmore, set_keep_msg, vim_dialog_yesno, wait_return,
};
use crate::src::nvim::mouse::setmouse;
use crate::src::nvim::r#move::{
    changed_line_abv_curs, changed_window_setting, do_check_cursorbind, invalidate_botline_win,
    scrolldown_clamp, scrollup_clamp, update_topline, validate_cursor,
};
use crate::src::nvim::normal::reset_VIsual;
use crate::src::nvim::ops::do_join;
use crate::src::nvim::option::{
    buf_copy_options, copy_option_part, magic_isset, set_option_direct, shortmess,
};
use crate::src::nvim::options::{kOptFoldcolumn, kOptInccommand, kOptShortmess};
use crate::src::nvim::os::env::expand_env_save;
use crate::src::nvim::os::fs::{
    os_file_is_writable, os_file_mkdir, os_isdir, os_nodetype, os_path_exists, os_remove,
};
use crate::src::nvim::os::input::{fast_breakcheck, line_breakcheck, os_breakcheck};
use crate::src::nvim::os::libc::{
    __ctype_b_loc, atol, gettext, log10, memcpy, memmove, memset, ngettext, qsort, snprintf,
    strcasecmp, strcat, strchr, strcmp, strcoll, strcpy, strlen, strncmp, strtod, time,
};
use crate::src::nvim::os::shell::call_shell;
use crate::src::nvim::os::time::os_time;
use crate::src::nvim::path::{fix_fname, invocation_path_tail};
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
use crate::{semsg_c, smsg_c};

// The carve of the transpiled module; see each child's docs.
mod append;
mod ecmd;
mod filter;
mod global;
mod lines;
mod sort;
mod subst;
mod text;
mod write;

pub use self::append::*;
pub use self::ecmd::*;
pub use self::filter::*;
pub use self::global::*;
pub use self::lines::*;
pub use self::sort::*;
pub use self::subst::*;
pub use self::text::*;
pub use self::write::*;

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
pub unsafe extern "C" fn check_secure() -> bool {
    unsafe {
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
}
pub unsafe extern "C" fn prepare_tagpreview(mut undo_sync: bool) -> bool {
    unsafe {
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
                        data: c"0".as_ptr() as *mut ::core::ffi::c_char,
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
}
pub unsafe extern "C" fn skip_vimgrep_pat(
    mut p: *mut ::core::ffi::c_char,
    mut s: *mut *mut ::core::ffi::c_char,
    mut flags: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
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
}
pub unsafe fn ex_oldfiles(mut eap: *mut exarg_T) {
    unsafe {
        let mut l: *mut list_T = get_vim_var_list(VV_OLDFILES);
        let mut nr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if l.is_null() {
            msg(gettext(c"No old files".as_ptr()), 0 as ::core::ffi::c_int);
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
                    msg_puts(c": ".as_ptr());
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
                let p: *const ::core::ffi::c_char =
                    tv_list_find_str(l, nr - 1 as ::core::ffi::c_int);
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
