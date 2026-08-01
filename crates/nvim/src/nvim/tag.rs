#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::ascii::{ascii_isdigit, ascii_isspace, ascii_iswhite};
use crate::src::nvim::autocmd::{EVENT_BUFREADCMD, has_autocmd};
use crate::src::nvim::buffer::{bt_help, buflist_findname_exp, buflist_findnr, buflist_getfile};
use crate::src::nvim::charset::{ptr2cells, skipdigits, vim_isblankline};
use crate::src::nvim::cmdexpand::{ExpandInit, ExpandOne};
use crate::src::nvim::cursor::check_cursor;
use crate::src::nvim::drawscreen::{UPD_VALID, redraw_later};
use crate::src::nvim::eval::typval::{
    callback_copy, callback_free, tv_clear, tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str,
    tv_dict_alloc, tv_dict_alloc_lock, tv_dict_find, tv_dict_get_number, tv_dict_get_string,
    tv_get_number, tv_list_alloc, tv_list_append_dict, tv_list_append_number, tv_list_free,
};
use crate::src::nvim::eval::typval::{kCallbackNone, tv_list_first};
use crate::src::nvim::eval::vars::set_vim_var_string;
use crate::src::nvim::eval::{callback_call, list2fpos, set_ref_in_callback};
use crate::src::nvim::ex_cmds::{getfile, prepare_tagpreview};
use crate::src::nvim::ex_docmd::{do_cmdline_cmd, set_no_hlsearch};
use crate::src::nvim::file_search::{
    vim_findfile, vim_findfile_cleanup, vim_findfile_init, vim_findfile_stopdir,
};
use crate::src::nvim::fileio::vim_fgets;
use crate::src::nvim::fold::foldOpenCursor;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::help::help_heuristic;
use crate::src::nvim::input::prompt_for_input;
use crate::src::nvim::insexpand::{ins_compl_check_keys, ins_compl_interrupted};
use crate::src::nvim::main::{
    Columns, IObuff, KeyTyped, RedrawingDisabled, State, cmdmod, curbuf, curtab, curwin, e_invarg,
    e_listreq, emsg_off, fdo_flags, g_do_tagpreview, g_tag_at_cursor, got_int, jop_flags,
    keep_help_flag, magic_overruled, msg_col, msg_didout, msg_scroll, msg_scrolled, msg_silent,
    no_hlsearch, p_cpo, p_enc, p_hf, p_hlg, p_ic, p_scs, p_sft, p_tags, p_tbs, p_tgst, p_tl, p_tr,
    p_verbose, p_ws, postponed_split, postponed_split_flags, sandbox, secure, swb_flags, tc_flags,
    vim_ignored,
};
use crate::src::nvim::mark::{fm_getname, mark_view_make, mark_view_restore, setpcmark};
use crate::src::nvim::mbyte::{convert_setup, mb_strnicmp, string_convert, utfc_ptr2len};
use crate::src::nvim::memory::{xfree, xmalloc, xmemdupz, xstrdup, xstrlcat};
use crate::src::nvim::message::{
    emsg, give_warning, msg, msg_advance, msg_clr_eos, msg_delay, msg_ext_set_kind, msg_outtrans,
    msg_outtrans_len, msg_outtrans_one, msg_putchar, msg_puts, msg_puts_hl, msg_puts_title,
    msg_start, semsg, smsg, verbose_enter, verbose_leave, wait_return,
};
use crate::src::nvim::r#move::{set_topline, validate_cursor};
use crate::src::nvim::option::{copy_option_part, magic_isset, option_set_callback_func};
use crate::src::nvim::options::{
    kOptFdoFlagTag, kOptJopFlagView, kOptSwbFlagNewtab, kOptSwbFlagUseopen, kOptSwbFlagUsetab,
    kOptSwbFlagVsplit,
};
use crate::src::nvim::optionstr::free_string_option;
use crate::src::nvim::os::fs::{os_fopen, os_path_exists};
use crate::src::nvim::os::input::{fast_breakcheck, line_breakcheck, os_breakcheck};
use crate::src::nvim::os::libc::{
    abort, atoi, fclose, fseeko, ftello, gettext, snprintf, strcasecmp, strcmp, strlen, strncmp,
    strstr,
};
use crate::src::nvim::path::{
    FreeWild, FullName_save, path_full_compare, path_has_wildcard, simplify_filename, vim_isAbsName,
};
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::quickfix::set_errorlist;
use crate::src::nvim::regexp::skip_regexp;
use crate::src::nvim::runtime::do_in_runtimepath;
use crate::src::nvim::search::{do_search, ignorecase, ignorecase_opt};
use crate::src::nvim::state::MODE_INSERT;
use crate::src::nvim::strings::{vim_snprintf, vim_snprintf_safelen, vim_strchr};
use crate::src::nvim::types::ui::kUIMessages;
use crate::src::nvim::types::{
    AdditionalData, Callback, Callback_data as C2Rust_Unnamed_5, Direction, FILE, OptInt,
    SpecialVarValue, Timestamp, VarLockStatus, VarType, VimVarIndex, buf_T, colnr_T, dict_T,
    dictitem_T, exarg_T, expand_T, file_comparison, fmark_T, fmarkv_T, garray_T, getf_retvalues,
    getf_values, int64_t, linenr_T, list_T, off_T, optmagic_T, optset_T, pos_T, ptrdiff_t,
    regmatch_T, regprog_T, size_t, taggy_T, typval_T, typval_vval_union, varnumber_T, vimconv_T,
    win_T, xp_prefix_T,
};
use crate::src::nvim::ui::ui_has;
use crate::src::nvim::window::{
    check_can_set_curbuf_forceit, swbuf_goto_win_with_buf, tabpage_index, win_close, win_enter,
    win_split, win_valid,
};

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
unsafe extern "C" {
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const VAR_SPECIAL: VarType = 8;
pub const VAR_DICT: VarType = 5;
pub const VAR_LIST: VarType = 4;
pub const VAR_STRING: VarType = 2;
pub const VAR_UNKNOWN: VarType = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_14 = 2147483647;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const EXPAND_FILES: C2Rust_Unnamed_16 = 2;
pub const OPTION_MAGIC_OFF: optmagic_T = 2;
pub const GETF_SETMARK: getf_values = 1;
pub const GETFILE_UNUSED: getf_retvalues = 8;
pub const GETFILE_OPEN_OTHER: getf_retvalues = -1;
pub const GETFILE_SAME_FILE: getf_retvalues = 0;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const WILD_EXPAND_FREE: C2Rust_Unnamed_22 = 2;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const WILD_SILENT: C2Rust_Unnamed_23 = 64;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_23 = 1;
pub const VV_SWAPCOMMAND: VimVarIndex = 49;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const CONV_NONE: C2Rust_Unnamed_25 = 0;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const FINDFILE_FILE: C2Rust_Unnamed_26 = 0;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const OPT_LOCAL: C2Rust_Unnamed_28 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_28 = 1;
pub const kEqualFiles: file_comparison = 1;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const DIP_ALL: C2Rust_Unnamed_29 = 1;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const SEARCH_KEEP: C2Rust_Unnamed_30 = 1024;
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
pub const WSP_VERT: C2Rust_Unnamed_34 = 2;
pub const MT_IC_OFF: C2Rust_Unnamed_35 = 4;
pub const MT_MASK: C2Rust_Unnamed_35 = 7;
pub const MT_COUNT: C2Rust_Unnamed_35 = 16;
pub const MT_GL_OTH: C2Rust_Unnamed_35 = 2;
pub const MT_GL_CUR: C2Rust_Unnamed_35 = 1;
pub const MT_ST_OTH: C2Rust_Unnamed_35 = 3;
pub const MT_ST_CUR: C2Rust_Unnamed_35 = 0;
pub const MT_RE_OFF: C2Rust_Unnamed_35 = 8;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const CMDBUFFSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL_0,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const TAGSTACKSIZE: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CPO_TAGPAT: ::core::ffi::c_int = 't' as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
static mt_names: GlobalCell<[*mut ::core::ffi::c_char; 8]> = GlobalCell::new([
    b"FSC\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"F C\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"F  \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"FS \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b" SC\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"  C\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"   \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b" S \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
]);
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
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
