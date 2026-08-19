#![deny(unsafe_op_in_unsafe_fn)]

use crate::ascii::{ascii_isdigit, ascii_isspace, ascii_iswhite};
use crate::autocmd::{EVENT_BUFREADCMD, has_autocmd};
use crate::buffer::{bt_help, buflist_findname_exp, buflist_findnr, buflist_getfile};
use crate::charset::{ptr2cells, skipdigits, vim_isblankline};
use crate::cmdexpand::{ExpandInit, ExpandOne};
use crate::cursor::check_cursor;
use crate::drawscreen::{UPD_VALID, redraw_later};
use crate::eval::typval::{
    callback_copy, callback_free, kCallbackNone, tv_clear, tv_dict_add_list, tv_dict_add_nr,
    tv_dict_add_str, tv_dict_alloc, tv_dict_alloc_lock, tv_dict_find, tv_dict_get_number,
    tv_dict_get_string, tv_get_number, tv_list_alloc, tv_list_append_dict, tv_list_append_number,
    tv_list_first, tv_list_free,
};
use crate::eval::vars::set_vim_var_string;
use crate::eval::{callback_call, list2fpos, set_ref_in_callback};
use crate::ex_cmds::{getfile, prepare_tagpreview};
use crate::ex_docmd::{do_cmdline_cmd, set_no_hlsearch};
use crate::file_search::{
    vim_findfile, vim_findfile_cleanup, vim_findfile_init, vim_findfile_stopdir,
};
use crate::fileio::vim_fgets;
use crate::fold::foldOpenCursor;
use crate::global_cell::GlobalCell;
use crate::help::help_heuristic;
use crate::input::prompt_for_input;
use crate::insexpand::{ins_compl_check_keys, ins_compl_interrupted};
use crate::main::{
    Columns, IObuff, KeyTyped, RedrawingDisabled, State, cmdmod, curbuf, curtab, curwin, e_invarg,
    e_listreq, emsg_off, fdo_flags, g_do_tagpreview, g_tag_at_cursor, got_int, jop_flags,
    keep_help_flag, magic_overruled, msg_col, msg_didout, msg_scroll, msg_scrolled, msg_silent,
    no_hlsearch, p_enc, p_hf, p_hlg, p_ic, p_scs, p_sft, p_tags, p_tbs, p_tgst, p_tl, p_tr,
    p_verbose, p_ws, postponed_split, postponed_split_flags, sandbox, secure, swb_flags, tc_flags,
    vim_ignored,
};
use crate::mark::{fm_getname, mark_view_make, mark_view_restore, setpcmark};
use crate::mbyte::{convert_setup, mb_strnicmp, string_convert, utfc_ptr2len};
use crate::memory::{xfree, xmalloc, xmemdupz, xstrdup, xstrlcat};
use crate::message::{
    emsg, give_warning, msg, msg_advance, msg_clr_eos, msg_delay, msg_ext_set_kind, msg_outtrans,
    msg_outtrans_len, msg_outtrans_one, msg_putchar, msg_puts, msg_puts_hl, msg_puts_title,
    msg_start, verbose_enter, verbose_leave, wait_return,
};
use crate::r#move::{set_topline, validate_cursor};
use crate::option::{copy_option_part, magic_isset, option_set_callback_func};
use crate::options::{
    kOptFdoFlagTag, kOptJopFlagView, kOptSwbFlagNewtab, kOptSwbFlagUseopen, kOptSwbFlagUsetab,
    kOptSwbFlagVsplit,
};
use crate::optionstr::free_string_option;
use crate::os::cshim::{gettext, snprintf, strncmp, strstr};
use crate::os::fs::{os_fopen, os_path_exists};
use crate::os::input::{fast_breakcheck, line_breakcheck, os_breakcheck};
use crate::path::{
    FreeWild, FullName_save, path_full_compare, path_has_wildcard, simplify_filename, vim_isAbsName,
};
use crate::pos::MAXLNUM;
use crate::quickfix::set_errorlist;
use crate::regexp::{skip_regexp, vim_regcomp, vim_regexec, vim_regfree};
use crate::runtime::do_in_runtimepath;
use crate::search::{do_search, ignorecase, ignorecase_opt};
use crate::state::MODE_INSERT;
use crate::strings::{vim_snprintf, vim_snprintf_safelen, vim_strchr};
use crate::types::ui::kUIMessages;
use crate::types::{
    AdditionalData, Callback, Callback_data as C2Rust_Unnamed_5, FILE, OptInt, Timestamp, buf_T,
    colnr_T, dict_T, dictitem_T, exarg_T, expand_T, file_comparison, fmark_T, fmarkv_T,
    getf_retvalues, getf_values, int64_t, linenr_T, list_T, off_T, optmagic_T, optset_T, pos_T,
    ptrdiff_t, regmatch_T, size_t, taggy_T, typval_T, typval_vval_union, varnumber_T, vimconv_T,
    win_T,
};
use crate::ui::ui_has;
use crate::window::{
    check_can_set_curbuf_forceit, swbuf_goto_win_with_buf, tabpage_index, win_close, win_enter,
    win_split, win_valid,
};
use ::libc::{abort, atoi, fclose, fseeko, ftello, strcasecmp, strcmp, strlen};

// The carve of the transpiled module; see each child's docs.
mod scan;
pub use self::scan::*;
mod parse;
pub(crate) use self::parse::*;
mod collect;
mod tagfile;
pub(crate) use self::tagfile::*;
mod jump;
pub(crate) use self::jump::*;
mod list;
pub(crate) use self::list::*;
mod query;
pub use self::query::*;

mod tagfunc;
pub use self::tagfunc::*;
mod stack;
pub use self::stack::*;
mod command;
pub use self::command::*;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const OPTION_MAGIC_OFF: optmagic_T = 2;
pub const GETF_SETMARK: getf_values = 1;
pub const GETFILE_OPEN_OTHER: getf_retvalues = -1;
pub const GETFILE_SAME_FILE: getf_retvalues = 0;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const FINDFILE_FILE: C2Rust_Unnamed_26 = 0;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const kEqualFiles: file_comparison = 1;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const LSIZE: C2Rust_Unnamed_31 = 512;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const DT_LTAG: C2Rust_Unnamed_32 = 11;
pub const DT_JUMP: C2Rust_Unnamed_32 = 9;
pub const DT_HELP: C2Rust_Unnamed_32 = 8;
pub const DT_SELECT: C2Rust_Unnamed_32 = 7;
pub const DT_FIRST: C2Rust_Unnamed_32 = 5;
pub const DT_PREV: C2Rust_Unnamed_32 = 4;
pub const DT_NEXT: C2Rust_Unnamed_32 = 3;
pub const DT_POP: C2Rust_Unnamed_32 = 2;
pub const DT_TAG: C2Rust_Unnamed_32 = 1;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const TAG_MANY: C2Rust_Unnamed_33 = 300;
pub const TAG_NO_TAGFUNC: C2Rust_Unnamed_33 = 256;
pub const TAG_KEEP_LANG: C2Rust_Unnamed_33 = 128;
pub const TAG_INS_COMP: C2Rust_Unnamed_33 = 64;
pub const TAG_VERBOSE: C2Rust_Unnamed_33 = 32;
pub const TAG_NOIC: C2Rust_Unnamed_33 = 8;
pub const TAG_REGEXP: C2Rust_Unnamed_33 = 4;
pub const TAG_NAMES: C2Rust_Unnamed_33 = 2;
pub const TAG_HELP: C2Rust_Unnamed_33 = 1;
/// The parts of one tags-file line, as pointers into the buffer holding it.
///
/// Each field is bracketed by a start and an `_end` pointer; the parts a
/// line need not have (the kind, the user data) are NULL when absent. The
/// buffer is the caller's, and outlives every `TagParts` taken from it.
#[derive(Copy, Clone, Default)]
pub struct TagParts {
    pub tagname: *mut ::core::ffi::c_char,
    pub tagname_end: *mut ::core::ffi::c_char,
    pub fname: *mut ::core::ffi::c_char,
    pub fname_end: *mut ::core::ffi::c_char,
    pub command: *mut ::core::ffi::c_char,
    pub command_end: *mut ::core::ffi::c_char,
    pub tag_fname: *mut ::core::ffi::c_char,
    pub tagkind: *mut ::core::ffi::c_char,
    pub tagkind_end: *mut ::core::ffi::c_char,
    pub user_data: *mut ::core::ffi::c_char,
    pub user_data_end: *mut ::core::ffi::c_char,
    pub tagline: linenr_T,
}
pub const MT_IC_OFF: C2Rust_Unnamed_35 = 4;
pub const MT_MASK: C2Rust_Unnamed_35 = 7;
pub const MT_COUNT: C2Rust_Unnamed_35 = 16;
pub const MT_GL_OTH: C2Rust_Unnamed_35 = 2;
pub const MT_GL_CUR: C2Rust_Unnamed_35 = 1;
pub const MT_ST_OTH: C2Rust_Unnamed_35 = 3;
pub const MT_ST_CUR: C2Rust_Unnamed_35 = 0;
pub const MT_RE_OFF: C2Rust_Unnamed_35 = 8;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const CMDBUFFSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const TAGSTACKSIZE: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const NOTAGFILE: ::core::ffi::c_int = 99 as ::core::ffi::c_int;
static nofile_fname: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static tagmatchname: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static ptag_entry: GlobalCell<taggy_T> = GlobalCell::new(taggy_T {
    tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    fmark: fmark_T {
        mark: pos_T {
            lnum: 0 as linenr_T,
            col: 0 as colnr_T,
            coladd: 0 as colnr_T,
        },
        fnum: 0 as ::core::ffi::c_int,
        timestamp: 0 as Timestamp,
        view: fmarkv_T {
            topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
            skipcol: 0 as colnr_T,
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    cur_match: 0 as ::core::ffi::c_int,
    cur_fnum: 0 as ::core::ffi::c_int,
    user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
});
pub const TAG_SEP: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
/// Forget which tag the remembered matches are for.
///
/// # Safety
/// Must not be called while a match is still being read.
pub unsafe fn tag_freematch() {
    // SAFETY: the name is ours, or NULL.
    unsafe { xfree(tagmatchname.get().cast()) };
    tagmatchname.set(::core::ptr::null_mut());
}
pub const ML_EXTRA: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
