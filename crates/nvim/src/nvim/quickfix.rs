use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::arglist::get_arglist_exp;
use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::autocmd::{
    EVENT_BUFREADPOST, EVENT_BUFWINENTER, EVENT_FILETYPE, EVENT_QUICKFIXCMDPOST,
    EVENT_QUICKFIXCMDPRE, apply_autocmds, aucmd_prepbuf, aucmd_restbuf,
};
use crate::src::nvim::autocmd::{
    au_event_disable, au_event_restore, block_autocmds, unblock_autocmds,
};
use crate::src::nvim::buffer::{
    bt_help, bt_normal, bt_quickfix, buf_valid, buflist_findname_exp, buflist_findnr, buflist_new,
    bufref_valid, close_buffer, set_bufref, setfname, wipe_buffer,
};
use crate::src::nvim::buffer::{buflist_getfile, do_modelines, no_write_message};
use crate::src::nvim::change::changed_lines;
use crate::src::nvim::charset::{skipdigits, skipwhite, vim_isprintc};
use crate::src::nvim::cursor::{check_cursor, coladvance};
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, UPD_VALID, redraw_buf_later, redraw_later};
use crate::src::nvim::drawscreen::{redraw_curbuf_later, update_screen};
use crate::src::nvim::edit::beginline;
use crate::src::nvim::eval::typval::{
    callback_copy, callback_free, callback_put, kCallbackNone, tv_clear, tv_copy, tv_dict_add,
    tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str, tv_dict_add_tv, tv_dict_alloc,
    tv_dict_alloc_lock, tv_dict_alloc_ret, tv_dict_find, tv_dict_get_bool, tv_dict_get_number,
    tv_dict_get_string, tv_dict_get_tv, tv_dict_item_alloc_len, tv_dict_item_free, tv_dict_unref,
    tv_free, tv_get_number_chk, tv_get_string_chk, tv_list_alloc, tv_list_alloc_ret,
    tv_list_append_dict,
};
use crate::src::nvim::eval::typval::{tv_list_first, tv_list_len, tv_list_ref};
use crate::src::nvim::eval::vars::set_internal_string_var;
use crate::src::nvim::eval::window::{find_win_by_nr_or_id, win_id2wp};
use crate::src::nvim::eval::{
    callback_call, callback_from_typval, eval_expr, set_ref_in_callback, set_ref_in_item,
};
use crate::src::nvim::ex_cmds::do_ecmd;
use crate::src::nvim::ex_cmds::{append_redir, check_secure, do_shell, skip_vimgrep_pat};
use crate::src::nvim::ex_cmds2::autowrite_all;
use crate::src::nvim::ex_cmds2::can_abandon;
use crate::src::nvim::ex_docmd::{do_cmdline_cmd, ex_cd, is_loclist_cmd};
use crate::src::nvim::ex_eval::{aborting, enter_cleanup, leave_cleanup};
use crate::src::nvim::ex_getln::get_list_range;
use crate::src::nvim::extmark::extmark_splice;
use crate::src::nvim::fileio::shorten_buf_fname;
use crate::src::nvim::fileio::{readfile, shorten_fnames, vim_fgets, vim_tempname};
use crate::src::nvim::fold::{foldOpenCursor, foldUpdateAll};
use crate::src::nvim::fuzzy::fuzzy_match;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::help::check_help_lang;
use crate::src::nvim::highlight_group::syn_name2id;
use crate::src::nvim::main::{
    Columns, IObuff, KeyTyped, NameBuff, cmdline_row, cmdmod, e_au_recursive,
    e_buffer_is_not_loaded, e_dictreq, e_invalpat, e_invarg, e_invarg2, e_invrange, e_listreq,
    e_loclist, e_no_errors, e_nomatch, e_nomatch2, e_noprevre, e_notmp, e_openerrf, e_readerrf,
    e_string_required, e_trailing_arg, e_winfixbuf_cannot_go_to_buffer, empty_string_option,
    fdo_flags, got_int, msg_col, msg_didout, msg_nowait, msg_scroll, msg_scrolled, must_redraw,
    p_ch, p_chi, p_cpo, p_ef, p_efm, p_enc, p_gefm, p_gp, p_hh, p_ic, p_mef, p_menc, p_mls, p_qftf,
    p_rtp, p_shq, p_sp, p_swb, restart_edit, swb_flags, textlock,
};
use crate::src::nvim::main::{curbuf, curtab, curwin, first_tabpage, firstwin, lastwin, prevwin};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::mbyte::{convert_setup, remove_bom, string_convert};
use crate::src::nvim::memfile::mf_fname;
use crate::src::nvim::memline::{check_need_swap, ml_delete};
use crate::src::nvim::memline::{ml_append_buf, ml_get_buf, ml_get_buf_len, ml_open};
use crate::src::nvim::memory::{
    strequal, xcalloc, xfree, xmalloc, xmallocz, xrealloc, xstrdup, xstrlcat, xstrlcpy,
};
use crate::src::nvim::message::{
    emsg, internal_error, message_filtered, msg, msg_clr_eos, msg_ext_set_kind, msg_keep,
    msg_outtrans, msg_prt_line, msg_putchar, msg_puts, msg_puts_hl, msg_start, msg_strtrunc, semsg,
    smsg, trunc_string,
};
use crate::src::nvim::r#move::update_topline;
use crate::src::nvim::normal::reset_VIsual_and_resel;
use crate::src::nvim::ops::get_region_bytecount;
use crate::src::nvim::option::buf_copy_options;
use crate::src::nvim::option::{
    copy_option_part, option_set_callback_func, set_option_direct, set_option_value_give_err,
    shortmess, skip_to_option_part,
};
use crate::src::nvim::options::{
    kOptBufhidden, kOptBuftype, kOptCpoptions, kOptErrorfile, kOptFdoFlagQuickfix, kOptFiletype,
    kOptFoldmethod, kOptSwapfile, kOptSwbFlagUselast, kOptSwbFlagUsetab,
};
use crate::src::nvim::optionstr::free_string_option;
use crate::src::nvim::os::env::{expand_env, os_get_pid};
use crate::src::nvim::os::fs::{
    os_dirname, os_fileinfo_link, os_fopen, os_isdir, os_open_stdin_fd, os_path_exists, os_remove,
};
use crate::src::nvim::os::input::{line_breakcheck, os_breakcheck};
use crate::src::nvim::os::libc::{
    __assert_fail, __errno_location, abort, abs, atoi, atol, fclose, fdopen, ferror, fgets,
    gettext, memset, snprintf, strcmp, strlen, strncasecmp, time,
};
use crate::src::nvim::path::{
    FreeWild, PATHSEP, after_pathsep, concat_fnames, fix_fname, gen_expand_wildcards,
    path_fnamecmp, path_is_absolute, path_tail, path_try_shorten_fname, vim_isAbsName,
};
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::regexp::{vim_regcomp, vim_regexec, vim_regexec_multi, vim_regfree};
use crate::src::nvim::search::{
    BACKWARD, BACKWARD_FILE, FORWARD, FORWARD_FILE, do_search, last_search_pat,
};
use crate::src::nvim::strings::{has_non_ascii, vim_snprintf, vim_snprintf_safelen, vim_strchr};
use crate::src::nvim::types::builders::static_cstring;
use crate::src::nvim::types::{
    CMD_index, Callback, Callback_data as C2Rust_Unnamed_6, DirStack, Direction, EvalFuncData,
    ExtmarkOp, FILE, FileInfo, ListLenSpecials, OptInt, OptVal, OptValData, OptValType,
    QFLT_INTERNAL, QFLT_LOCATION, QFLT_QUICKFIX, TriState, VarLockStatus, VarType, aco_save_T,
    bln_values, buf_T, bufref_T, cleanup_T, cmd_addr_T, cmdidx_T, colnr_T, cstack_T, dict_T,
    dictitem_T, dobuf_action_values, exarg, exarg_T, except_T, getf_values, handle_T, ht_stack_T,
    linenr_T, list_T, list_stack_T, listitem_T, lpos_T, optset_T, pos_T, proftime_T, ptrdiff_t,
    qf_info_T, qf_list_T, qfline_T, qfltype_T, regmatch_T, regmmatch_T, regprog_T, scid_T, size_t,
    tabpage_T, time_t, typval_T, typval_vval_union, uint32_t, varnumber_T, vimconv_T, win_T,
};
use crate::src::nvim::ui::ui_flush;
use crate::src::nvim::undo::u_clearallandblockfree;
use crate::src::nvim::window::{
    check_can_set_curbuf_forceit, check_lnums, tabline_height, win_setheight, win_setwidth,
    win_split,
};
use crate::src::nvim::window::{goto_tabpage_win, win_close, win_enter, win_goto, win_valid};

// The carve of the transpiled module; see each child's docs.
mod efm;
pub(crate) use self::efm::*;
mod parse;
pub(crate) use self::parse::*;
mod read;
pub use self::read::*;
mod stack;
pub use self::stack::*;
mod list;
pub use self::list::*;
mod entry;
pub use self::entry::*;
mod switchbuf;
pub(crate) use self::switchbuf::*;
mod jump;
pub use self::jump::*;
mod display;
pub use self::display::*;
mod window;
pub use self::window::*;
mod fill;
pub(crate) use self::fill::*;
mod navigate;
pub use self::navigate::*;
mod vimgrep;
pub use self::vimgrep::*;
mod make;
pub use self::make::*;
mod helpgrep;
pub use self::helpgrep::*;
mod cmds;
pub use self::cmds::*;
mod getprops;
pub(crate) use self::getprops::*;
mod setprops;
pub use self::setprops::*;
mod eval;
pub use self::eval::*;
pub const kFalse: TriState = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const VAR_FLOAT: VarType = 6;
pub const VAR_DICT: VarType = 5;
pub const VAR_LIST: VarType = 4;
pub const VAR_STRING: VarType = 2;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_UNKNOWN: VarType = 0;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kDirectionNotSet: Direction = 0;
pub const CMD_vimgrepadd: CMD_index = 511;
pub const CMD_make: CMD_index = 274;
pub const CMD_helpgrep: CMD_index = 178;
pub const CMD_lvimgrepadd: CMD_index = 268;
pub const CMD_lrewind: CMD_index = 261;
pub const CMD_lpfile: CMD_index = 260;
pub const CMD_lprevious: CMD_index = 259;
pub const CMD_lolder: CMD_index = 257;
pub const CMD_lnfile: CMD_index = 252;
pub const CMD_lnext: CMD_index = 250;
pub const CMD_lmake: CMD_index = 248;
pub const CMD_ll: CMD_index = 243;
pub const CMD_lhelpgrep: CMD_index = 241;
pub const CMD_lgrepadd: CMD_index = 240;
pub const CMD_lgrep: CMD_index = 239;
pub const CMD_lfirst: CMD_index = 235;
pub const CMD_lfdo: CMD_index = 234;
pub const CMD_lfile: CMD_index = 233;
pub const CMD_lexpr: CMD_index = 232;
pub const CMD_ldo: CMD_index = 228;
pub const CMD_lcd: CMD_index = 225;
pub const CMD_lbelow: CMD_index = 223;
pub const CMD_lbuffer: CMD_index = 221;
pub const CMD_lafter: CMD_index = 219;
pub const CMD_laddfile: CMD_index = 218;
pub const CMD_laddbuffer: CMD_index = 217;
pub const CMD_laddexpr: CMD_index = 216;
pub const CMD_labove: CMD_index = 214;
pub const CMD_lNfile: CMD_index = 212;
pub const CMD_lNext: CMD_index = 211;
pub const CMD_grepadd: CMD_index = 173;
pub const CMD_grep: CMD_index = 172;
pub const CMD_crewind: CMD_index = 104;
pub const CMD_cpfile: CMD_index = 102;
pub const CMD_cprevious: CMD_index = 101;
pub const CMD_colder: CMD_index = 91;
pub const CMD_cnfile: CMD_index = 86;
pub const CMD_cnext: CMD_index = 84;
pub const CMD_cfirst: CMD_index = 67;
pub const CMD_cfdo: CMD_index = 66;
pub const CMD_cfile: CMD_index = 65;
pub const CMD_cexpr: CMD_index = 64;
pub const CMD_cdo: CMD_index = 62;
pub const CMD_cd: CMD_index = 61;
pub const CMD_cc: CMD_index = 59;
pub const CMD_cbelow: CMD_index = 57;
pub const CMD_cbefore: CMD_index = 56;
pub const CMD_cbuffer: CMD_index = 55;
pub const CMD_cafter: CMD_index = 52;
pub const CMD_caddfile: CMD_index = 51;
pub const CMD_caddexpr: CMD_index = 50;
pub const CMD_caddbuffer: CMD_index = 49;
pub const CMD_cabove: CMD_index = 48;
pub const CMD_cNfile: CMD_index = 45;
pub const CMD_cNext: CMD_index = 44;
pub const ADDR_LINES: cmd_addr_T = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const CMOD_HIDE: C2Rust_Unnamed_16 = 32;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const HLF_QFL: C2Rust_Unnamed_17 = 58;
pub const HLF_N: C2Rust_Unnamed_17 = 12;
pub const HLF_D: C2Rust_Unnamed_17 = 5;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const GETF_SWITCH: getf_values = 4;
pub const GETF_SETMARK: getf_values = 1;
pub const BLN_NOOPT: bln_values = 16;
pub const BLN_DUMMY: bln_values = 4;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const SHM_OVERALL: C2Rust_Unnamed_20 = 79;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_22 = 4;
pub const BL_WHITE: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const CONV_NONE: C2Rust_Unnamed_23 = 0;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const ECMD_NOWINENTER: C2Rust_Unnamed_24 = 64;
pub const ECMD_OLDBUF: C2Rust_Unnamed_24 = 4;
pub const ECMD_SET_HELP: C2Rust_Unnamed_24 = 2;
pub const ECMD_HIDE: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_int;
pub const ECMD_ONE: C2Rust_Unnamed_25 = 1;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const READ_DUMMY: C2Rust_Unnamed_26 = 16;
pub const READ_NEW: C2Rust_Unnamed_26 = 1;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const FUZZY_MATCH_MAX_LEN: C2Rust_Unnamed_27 = 1024;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const BCO_NOHELP: C2Rust_Unnamed_28 = 4;
pub const BCO_ENTER: C2Rust_Unnamed_28 = 1;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const OPT_NOWIN: C2Rust_Unnamed_29 = 16;
pub const OPT_LOCAL: C2Rust_Unnamed_29 = 2;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const EW_SILENT: C2Rust_Unnamed_30 = 32;
pub const EW_FILE: C2Rust_Unnamed_30 = 2;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const VGR_FUZZY: C2Rust_Unnamed_31 = 4;
pub const VGR_NOJUMP: C2Rust_Unnamed_31 = 2;
pub const VGR_GLOBAL: C2Rust_Unnamed_31 = 1;
pub const QF_FAIL: C2Rust_Unnamed_34 = 0;
pub const QF_OK: C2Rust_Unnamed_34 = 1;
pub const QF_END_OF_INPUT: C2Rust_Unnamed_34 = 2;
pub const QF_IGNORE_LINE: C2Rust_Unnamed_34 = 4;
pub const QF_MULTISCAN: C2Rust_Unnamed_34 = 5;
pub const QF_NOMEM: C2Rust_Unnamed_34 = 3;
pub const QF_ABORT: C2Rust_Unnamed_34 = 6;
pub const WSP_ABOVE: C2Rust_Unnamed_33 = 128;
pub const WSP_NEWLOC: C2Rust_Unnamed_33 = 256;
pub const WSP_HELP: C2Rust_Unnamed_33 = 32;
pub const WSP_TOP: C2Rust_Unnamed_33 = 8;
pub const SEARCH_KEEP: C2Rust_Unnamed_32 = 1024;
pub const WSP_QUICKFIX: C2Rust_Unnamed_33 = 1024;
pub const WSP_BELOW: C2Rust_Unnamed_33 = 64;
pub const WSP_BOT: C2Rust_Unnamed_33 = 16;
pub const WSP_VERT: C2Rust_Unnamed_33 = 2;
pub const QF_WINHEIGHT: C2Rust_Unnamed_35 = 10;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vgr_args_T {
    pub tomatch: ::core::ffi::c_int,
    pub spat: *mut ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
    pub fnames: *mut *mut ::core::ffi::c_char,
    pub fcount: ::core::ffi::c_int,
    pub regmatch: regmmatch_T,
    pub qf_title: *mut ::core::ffi::c_char,
}
pub const QF_GETLIST_QFTF: C2Rust_Unnamed_36 = 2048;
pub const QF_GETLIST_NONE: C2Rust_Unnamed_36 = 0;
pub const QF_GETLIST_QFBUFNR: C2Rust_Unnamed_36 = 1024;
pub const QF_GETLIST_FILEWINID: C2Rust_Unnamed_36 = 512;
pub const QF_GETLIST_TICK: C2Rust_Unnamed_36 = 256;
pub const QF_GETLIST_SIZE: C2Rust_Unnamed_36 = 128;
pub const QF_GETLIST_IDX: C2Rust_Unnamed_36 = 64;
pub const QF_GETLIST_ITEMS: C2Rust_Unnamed_36 = 2;
pub const QF_GETLIST_ID: C2Rust_Unnamed_36 = 32;
pub const QF_GETLIST_CONTEXT: C2Rust_Unnamed_36 = 16;
pub const QF_GETLIST_WINID: C2Rust_Unnamed_36 = 8;
pub const QF_GETLIST_NR: C2Rust_Unnamed_36 = 4;
pub const QF_GETLIST_TITLE: C2Rust_Unnamed_36 = 1;
pub const QF_GETLIST_ALL: C2Rust_Unnamed_36 = 4095;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const CMDBUFFSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const INVALID_QFIDX: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const INVALID_QFBUFNR: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static e_no_more_items: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"E553: No more items\0".as_ptr() as *const ::core::ffi::c_char);
static e_current_quickfix_list_was_changed: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(
        b"E925: Current quickfix list was changed\0".as_ptr() as *const ::core::ffi::c_char
    );
static e_current_location_list_was_changed: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(
        b"E926: Current location list was changed\0".as_ptr() as *const ::core::ffi::c_char
    );
static qftf_cb: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: C2Rust_Unnamed_6 {
        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    type_0: kCallbackNone,
});
static qfFile_hl_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static qfSep_hl_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static qfLine_hl_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const BF_NEW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const BF_DUMMY: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const BUF_HAS_QF_ENTRY: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const BUF_HAS_LL_ENTRY: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const EINTR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RE_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
