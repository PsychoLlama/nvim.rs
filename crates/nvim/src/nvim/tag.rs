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
use crate::src::nvim::garray::{ga_clear, ga_grow, ga_init};
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
use crate::src::nvim::memory::{
    xfree, xmalloc, xmemcpyz, xmemdupz, xrealloc, xstrdup, xstrlcat, xstrlcpy,
};
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
    __assert_fail, __ctype_b_loc, abort, atoi, fclose, fseeko, ftello, gettext, memmove, snprintf,
    strcasecmp, strcat, strcmp, strcpy, strlen, strncmp, strstr,
};
use crate::src::nvim::path::{
    FreeWild, FullName_save, path_full_compare, path_has_wildcard, simplify_filename, vim_isAbsName,
};
use crate::src::nvim::pos::{MAXLNUM, clearpos};
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
    getf_values, hashitem_T, hashtab_T, ht_stack_T, int64_t, linenr_T, list_T, list_stack_T,
    listitem_T, off_T, optmagic_T, optset_T, pos_T, ptrdiff_t, regmatch_T, regprog_T, size_t,
    taggy_T, typval_T, typval_vval_union, uint8_t, uint64_t, varnumber_T, vimconv_T, win_T,
    xp_prefix_T,
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
use crate::src::nvim::hashtab::hash_removed;

mod tagfunc;
pub use self::tagfunc::*;
mod stack;
pub use self::stack::*;
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
pub const HLF_W: C2Rust_Unnamed_15 = 26;
pub const HLF_T: C2Rust_Unnamed_15 = 23;
pub const HLF_CM: C2Rust_Unnamed_15 = 11;
pub const HLF_D: C2Rust_Unnamed_15 = 5;
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
static e_tag_stack_empty: GlobalCell<[::core::ffi::c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"E73: Tag stack empty\0")
});
static e_tag_not_found_str: GlobalCell<[::core::ffi::c_char; 24]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E426: Tag not found: %s\0")
});
static e_at_bottom_of_tag_stack: GlobalCell<[::core::ffi::c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E555: At bottom of tag stack\0",
    )
});
static e_at_top_of_tag_stack: GlobalCell<[::core::ffi::c_char; 26]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E556: At top of tag stack\0")
});
static e_cannot_modify_tag_stack_within_tagfunc: GlobalCell<[::core::ffi::c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
            *b"E986: Cannot modify the tag stack within tagfunc\0",
        )
    });
static e_invalid_return_value_from_tagfunc: GlobalCell<[::core::ffi::c_char; 40]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
            *b"E987: Invalid return value from tagfunc\0",
        )
    });
static e_window_unexpectedly_close_while_searching_for_tags: GlobalCell<[::core::ffi::c_char; 59]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 59], [::core::ffi::c_char; 59]>(
            *b"E1299: Window unexpectedly closed while searching for tags\0",
        )
    });
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
static tfu_in_use: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static tfu_cb: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: C2Rust_Unnamed_5 {
        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    type_0: kCallbackNone,
});
pub const TAG_SEP: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub unsafe extern "C" fn do_tag(
    mut tag: *mut ::core::ffi::c_char,
    mut type_0: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut forceit: ::core::ffi::c_int,
    mut verbose: bool,
) {
    let mut tagstack: *mut taggy_T = &raw mut (*curwin.get()).w_tagstack as *mut taggy_T;
    let mut tagstackidx: ::core::ffi::c_int = (*curwin.get()).w_tagstackidx;
    let mut tagstacklen: ::core::ffi::c_int = (*curwin.get()).w_tagstacklen;
    let mut cur_match: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cur_fnum: ::core::ffi::c_int = (*curbuf.get()).handle as ::core::ffi::c_int;
    let mut oldtagstackidx: ::core::ffi::c_int = tagstackidx;
    let mut prevtagstackidx: ::core::ffi::c_int = tagstackidx;
    let mut new_tag: bool = false_0 != 0;
    let mut no_regexp: bool = false_0 != 0;
    let mut error_cur_match: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut save_pos: bool = false_0 != 0;
    let mut saved_fmark: fmark_T = fmark_T {
        mark: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        fnum: 0,
        timestamp: 0,
        view: fmarkv_T {
            topline_offset: 0,
            skipcol: 0,
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    };
    let mut new_num_matches: ::core::ffi::c_int = 0;
    let mut new_matches: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut use_tagstack: bool = false;
    let mut skip_msg: bool = false_0 != 0;
    let mut buf_ffname: *mut ::core::ffi::c_char = (*curbuf.get()).b_ffname;
    let mut use_tfu: bool = true_0 != 0;
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static num_matches: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    static max_num_matches: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    static matches: GlobalCell<*mut *mut ::core::ffi::c_char> =
        GlobalCell::new(::core::ptr::null_mut::<*mut ::core::ffi::c_char>());
    static flags: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    if tfu_in_use.get() {
        emsg(gettext(
            (e_cannot_modify_tag_stack_within_tagfunc.ptr() as *const _)
                as *const ::core::ffi::c_char,
        ));
        return;
    }
    if postponed_split.get() == 0 as ::core::ffi::c_int && !check_can_set_curbuf_forceit(forceit) {
        return;
    }
    if type_0 == DT_HELP as ::core::ffi::c_int {
        type_0 = DT_TAG as ::core::ffi::c_int;
        no_regexp = true_0 != 0;
        use_tfu = false_0 != 0;
    }
    let mut prev_num_matches: ::core::ffi::c_int = num_matches.get();
    free_string_option(nofile_fname.get());
    nofile_fname.set(::core::ptr::null_mut::<::core::ffi::c_char>());
    clearpos(&mut saved_fmark.mark);
    saved_fmark.fnum = 0 as ::core::ffi::c_int;
    saved_fmark.view = fmarkv_T {
        topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
        skipcol: 0 as colnr_T,
    };
    '_c2rust_label: {
        if !tag.is_null() {
        } else {
            __assert_fail(
                b"tag != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/tag.rs\0".as_ptr() as *const ::core::ffi::c_char,
                349 as ::core::ffi::c_uint,
                b"void do_tag(char *, int, int, int, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_end_do_tag: {
        if p_tgst.get() == 0 && *tag as ::core::ffi::c_int != NUL {
            use_tagstack = false_0 != 0;
            new_tag = true_0 != 0;
            if g_do_tagpreview.get() != 0 as ::core::ffi::c_int {
                tagstack_clear_entry(&mut *ptag_entry.ptr());
                (*ptag_entry.ptr()).tagname = xstrdup(tag);
            }
        } else {
            if g_do_tagpreview.get() != 0 as ::core::ffi::c_int {
                use_tagstack = false_0 != 0;
            } else {
                use_tagstack = true_0 != 0;
            }
            if *tag as ::core::ffi::c_int != NUL
                && (type_0 == DT_TAG as ::core::ffi::c_int
                    || type_0 == DT_SELECT as ::core::ffi::c_int
                    || type_0 == DT_JUMP as ::core::ffi::c_int
                    || type_0 == DT_LTAG as ::core::ffi::c_int)
            {
                if g_do_tagpreview.get() != 0 as ::core::ffi::c_int {
                    if !(*ptag_entry.ptr()).tagname.is_null()
                        && strcmp((*ptag_entry.ptr()).tagname, tag) == 0 as ::core::ffi::c_int
                    {
                        cur_match = (*ptag_entry.ptr()).cur_match;
                        cur_fnum = (*ptag_entry.ptr()).cur_fnum;
                    } else {
                        tagstack_clear_entry(&mut *ptag_entry.ptr());
                        (*ptag_entry.ptr()).tagname = xstrdup(tag);
                    }
                } else {
                    while tagstackidx < tagstacklen {
                        tagstacklen -= 1;
                        tagstack_clear_entry(&mut *tagstack.offset(tagstacklen as isize));
                    }
                    tagstacklen += 1;
                    if tagstacklen > TAGSTACKSIZE {
                        tagstacklen = TAGSTACKSIZE;
                        tagstack_clear_entry(
                            &mut *tagstack.offset(0 as ::core::ffi::c_int as isize),
                        );
                        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                        while i < tagstacklen {
                            *tagstack.offset((i - 1 as ::core::ffi::c_int) as isize) =
                                *tagstack.offset(i as isize);
                            i += 1;
                        }
                        tagstackidx -= 1;
                        let c2rust_lvalue_ptr =
                            &raw mut (*tagstack.offset(tagstackidx as isize)).user_data;
                        *c2rust_lvalue_ptr = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                    (*tagstack.offset(tagstackidx as isize)).tagname = xstrdup(tag);
                    (*curwin.get()).w_tagstacklen = tagstacklen;
                    save_pos = true_0 != 0;
                }
                new_tag = true_0 != 0;
            } else if if g_do_tagpreview.get() != 0 as ::core::ffi::c_int {
                (*ptag_entry.ptr()).tagname.is_null() as ::core::ffi::c_int
            } else {
                (tagstacklen == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            } != 0
            {
                emsg(gettext(
                    (e_tag_stack_empty.ptr() as *const _) as *const ::core::ffi::c_char,
                ));
                break '_end_do_tag;
            } else if type_0 == DT_POP as ::core::ffi::c_int {
                let old_KeyTyped: bool = KeyTyped.get();
                tagstackidx -= count;
                if tagstackidx < 0 as ::core::ffi::c_int {
                    emsg(gettext(
                        (e_at_bottom_of_tag_stack.ptr() as *const _) as *const ::core::ffi::c_char,
                    ));
                    if tagstackidx + count == 0 as ::core::ffi::c_int {
                        tagstackidx = 0 as ::core::ffi::c_int;
                        break '_end_do_tag;
                    } else {
                        tagstackidx = 0 as ::core::ffi::c_int;
                    }
                } else if tagstackidx >= tagstacklen {
                    emsg(gettext(
                        (e_at_top_of_tag_stack.ptr() as *const _) as *const ::core::ffi::c_char,
                    ));
                    break '_end_do_tag;
                }
                saved_fmark = (*tagstack.offset(tagstackidx as isize)).fmark;
                if saved_fmark.fnum != (*curbuf.get()).handle {
                    if buflist_getfile(
                        saved_fmark.fnum,
                        saved_fmark.mark.lnum,
                        GETF_SETMARK as ::core::ffi::c_int,
                        forceit,
                    ) == FAIL
                    {
                        tagstackidx = oldtagstackidx;
                        break '_end_do_tag;
                    } else {
                        (*curwin.get()).w_cursor.lnum = saved_fmark.mark.lnum;
                    }
                } else {
                    setpcmark();
                    (*curwin.get()).w_cursor.lnum = saved_fmark.mark.lnum;
                }
                (*curwin.get()).w_cursor.col = saved_fmark.mark.col;
                (*curwin.get()).w_set_curswant = true_0;
                if jop_flags.get() & kOptJopFlagView as ::core::ffi::c_int as ::core::ffi::c_uint
                    != 0
                {
                    mark_view_restore(&raw mut saved_fmark);
                }
                check_cursor(curwin.get());
                if fdo_flags.get() & kOptFdoFlagTag as ::core::ffi::c_int as ::core::ffi::c_uint
                    != 0
                    && old_KeyTyped as ::core::ffi::c_int != 0
                {
                    foldOpenCursor();
                }
                FreeWild(num_matches.get(), matches.get());
                num_matches.set(0 as ::core::ffi::c_int);
                tag_freematch();
                break '_end_do_tag;
            } else if type_0 == DT_TAG as ::core::ffi::c_int
                || type_0 == DT_LTAG as ::core::ffi::c_int
            {
                if g_do_tagpreview.get() != 0 as ::core::ffi::c_int {
                    cur_match = (*ptag_entry.ptr()).cur_match;
                    cur_fnum = (*ptag_entry.ptr()).cur_fnum;
                } else {
                    save_pos = true_0 != 0;
                    tagstackidx += count - 1 as ::core::ffi::c_int;
                    if tagstackidx >= tagstacklen {
                        tagstackidx = tagstacklen - 1 as ::core::ffi::c_int;
                        emsg(gettext(
                            (e_at_top_of_tag_stack.ptr() as *const _) as *const ::core::ffi::c_char,
                        ));
                        save_pos = false_0 != 0;
                    } else if tagstackidx < 0 as ::core::ffi::c_int {
                        emsg(gettext(
                            (e_at_bottom_of_tag_stack.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ));
                        tagstackidx = 0 as ::core::ffi::c_int;
                        break '_end_do_tag;
                    }
                    cur_match = (*tagstack.offset(tagstackidx as isize)).cur_match;
                    cur_fnum = (*tagstack.offset(tagstackidx as isize)).cur_fnum;
                }
                new_tag = true_0 != 0;
            } else {
                prevtagstackidx = tagstackidx;
                if g_do_tagpreview.get() != 0 as ::core::ffi::c_int {
                    cur_match = (*ptag_entry.ptr()).cur_match;
                    cur_fnum = (*ptag_entry.ptr()).cur_fnum;
                } else {
                    tagstackidx -= 1;
                    if tagstackidx < 0 as ::core::ffi::c_int {
                        tagstackidx = 0 as ::core::ffi::c_int;
                    }
                    cur_match = (*tagstack.offset(tagstackidx as isize)).cur_match;
                    cur_fnum = (*tagstack.offset(tagstackidx as isize)).cur_fnum;
                }
                match type_0 {
                    5 => {
                        cur_match = count - 1 as ::core::ffi::c_int;
                    }
                    7 | 9 | 6 => {
                        cur_match = MAXCOL as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                    }
                    3 => {
                        cur_match += count;
                    }
                    4 => {
                        cur_match -= count;
                    }
                    _ => {}
                }
                if cur_match >= MAXCOL as ::core::ffi::c_int {
                    cur_match = MAXCOL as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                } else if cur_match < 0 as ::core::ffi::c_int {
                    emsg(gettext(
                        b"E425: Cannot go before first matching tag\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ));
                    skip_msg = true_0 != 0;
                    cur_match = 0 as ::core::ffi::c_int;
                    cur_fnum = (*curbuf.get()).handle as ::core::ffi::c_int;
                }
            }
            if g_do_tagpreview.get() != 0 as ::core::ffi::c_int {
                if type_0 != DT_SELECT as ::core::ffi::c_int
                    && type_0 != DT_JUMP as ::core::ffi::c_int
                {
                    (*ptag_entry.ptr()).cur_match = cur_match;
                    (*ptag_entry.ptr()).cur_fnum = cur_fnum;
                }
            } else {
                saved_fmark = (*tagstack.offset(tagstackidx as isize)).fmark;
                if save_pos {
                    (*tagstack.offset(tagstackidx as isize)).fmark.mark = (*curwin.get()).w_cursor;
                    (*tagstack.offset(tagstackidx as isize)).fmark.fnum =
                        (*curbuf.get()).handle as ::core::ffi::c_int;
                    (*tagstack.offset(tagstackidx as isize)).fmark.view =
                        mark_view_make(curwin.get(), (*curwin.get()).w_cursor);
                }
                (*curwin.get()).w_tagstackidx = tagstackidx;
                if type_0 != DT_SELECT as ::core::ffi::c_int
                    && type_0 != DT_JUMP as ::core::ffi::c_int
                {
                    (*curwin.get()).w_tagstack[tagstackidx as usize].cur_match = cur_match;
                    (*curwin.get()).w_tagstack[tagstackidx as usize].cur_fnum = cur_fnum;
                }
            }
        }
        if cur_fnum != (*curbuf.get()).handle {
            let mut buf: *mut buf_T = buflist_findnr(cur_fnum);
            if !buf.is_null() {
                buf_ffname = (*buf).b_ffname;
            }
        }
        loop {
            let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if use_tagstack {
                name = xstrdup((*tagstack.offset(tagstackidx as isize)).tagname);
                xfree(tofree as *mut ::core::ffi::c_void);
                tofree = name;
            } else if g_do_tagpreview.get() != 0 as ::core::ffi::c_int {
                name = (*ptag_entry.ptr()).tagname;
            } else {
                name = tag;
            }
            let mut other_name: bool = (*tagmatchname.ptr()).is_null()
                || strcmp(tagmatchname.get(), name) != 0 as ::core::ffi::c_int;
            if new_tag as ::core::ffi::c_int != 0
                || cur_match >= num_matches.get()
                    && max_num_matches.get() != MAXCOL as ::core::ffi::c_int
                || other_name as ::core::ffi::c_int != 0
            {
                if other_name {
                    xfree(tagmatchname.get() as *mut ::core::ffi::c_void);
                    tagmatchname.set(xstrdup(name));
                }
                if type_0 == DT_SELECT as ::core::ffi::c_int
                    || type_0 == DT_JUMP as ::core::ffi::c_int
                    || type_0 == DT_LTAG as ::core::ffi::c_int
                {
                    cur_match = MAXCOL as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                }
                max_num_matches.set(if type_0 == DT_TAG as ::core::ffi::c_int {
                    MAXCOL as ::core::ffi::c_int
                } else {
                    cur_match + 1 as ::core::ffi::c_int
                });
                if !no_regexp && *name as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
                    flags.set(TAG_REGEXP as ::core::ffi::c_int);
                    name = name.offset(1);
                } else {
                    flags.set(TAG_NOIC as ::core::ffi::c_int);
                }
                (*flags.ptr()) |= if verbose as ::core::ffi::c_int != 0 {
                    TAG_VERBOSE as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
                (*flags.ptr()) |= if !use_tfu {
                    TAG_NO_TAGFUNC as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
                if find_tags(
                    name,
                    &raw mut new_num_matches,
                    &raw mut new_matches,
                    flags.get(),
                    max_num_matches.get(),
                    buf_ffname,
                ) == OK
                    && new_num_matches < max_num_matches.get()
                {
                    max_num_matches.set(MAXCOL as ::core::ffi::c_int);
                }
                if tagstack != &raw mut (*curwin.get()).w_tagstack as *mut taggy_T {
                    emsg(gettext(
                        (e_window_unexpectedly_close_while_searching_for_tags.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ));
                    FreeWild(new_num_matches, new_matches);
                    break '_end_do_tag;
                } else {
                    if !new_tag && !other_name {
                        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut tagp: TagParts = TagParts::default();
                        let mut tagp2: TagParts = TagParts::default();
                        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while j < num_matches.get() {
                            parse_match(*(*matches.ptr()).offset(j as isize), &mut tagp);
                            let mut i_0: ::core::ffi::c_int = idx;
                            while i_0 < new_num_matches {
                                parse_match(*new_matches.offset(i_0 as isize), &mut tagp2);
                                if strcmp(tagp.tagname, tagp2.tagname) == 0 as ::core::ffi::c_int {
                                    let mut p: *mut ::core::ffi::c_char =
                                        *new_matches.offset(i_0 as isize);
                                    let mut k: ::core::ffi::c_int = i_0;
                                    while k > idx {
                                        *new_matches.offset(k as isize) = *new_matches
                                            .offset((k - 1 as ::core::ffi::c_int) as isize);
                                        k -= 1;
                                    }
                                    let c2rust_fresh0 = idx;
                                    idx = idx + 1;
                                    let c2rust_lvalue_ptr_0 =
                                        &raw mut *new_matches.offset(c2rust_fresh0 as isize);
                                    *c2rust_lvalue_ptr_0 = p;
                                    break;
                                } else {
                                    i_0 += 1;
                                }
                            }
                            j += 1;
                        }
                    }
                    FreeWild(num_matches.get(), matches.get());
                    num_matches.set(new_num_matches);
                    matches.set(new_matches);
                }
            }
            if num_matches.get() <= 0 as ::core::ffi::c_int {
                if verbose {
                    semsg(
                        gettext(
                            (e_tag_not_found_str.ptr() as *const _) as *const ::core::ffi::c_char,
                        ),
                        name,
                    );
                }
                g_do_tagpreview.set(0 as ::core::ffi::c_int);
                break '_end_do_tag;
            } else {
                let mut ask_for_selection: bool = false_0 != 0;
                if type_0 == DT_TAG as ::core::ffi::c_int && *tag as ::core::ffi::c_int != NUL {
                    cur_match = if count > 0 as ::core::ffi::c_int {
                        count - 1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    };
                } else if type_0 == DT_SELECT as ::core::ffi::c_int
                    || type_0 == DT_JUMP as ::core::ffi::c_int
                        && num_matches.get() > 1 as ::core::ffi::c_int
                {
                    print_tag_list(new_tag, use_tagstack, num_matches.get(), matches.get());
                    ask_for_selection = true_0 != 0;
                } else if type_0 == DT_LTAG as ::core::ffi::c_int {
                    if add_llist_tags(tag, num_matches.get(), matches.get()) == FAIL {
                        break '_end_do_tag;
                    }
                    cur_match = 0 as ::core::ffi::c_int;
                }
                if ask_for_selection {
                    let mut i_1: ::core::ffi::c_int = prompt_for_input(
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        0 as ::core::ffi::c_int,
                        false_0 != 0,
                        ::core::ptr::null_mut::<bool>(),
                    );
                    if i_1 <= 0 as ::core::ffi::c_int
                        || i_1 > num_matches.get()
                        || got_int.get() as ::core::ffi::c_int != 0
                    {
                        if use_tagstack {
                            (*tagstack.offset(tagstackidx as isize)).fmark = saved_fmark;
                            tagstackidx = prevtagstackidx;
                        }
                        break '_end_do_tag;
                    } else {
                        cur_match = i_1 - 1 as ::core::ffi::c_int;
                    }
                }
                if cur_match >= num_matches.get() {
                    if (type_0 == DT_NEXT as ::core::ffi::c_int
                        || type_0 == DT_FIRST as ::core::ffi::c_int)
                        && (*nofile_fname.ptr()).is_null()
                    {
                        if num_matches.get() == 1 as ::core::ffi::c_int {
                            emsg(gettext(b"E427: There is only one matching tag\0".as_ptr()
                                as *const ::core::ffi::c_char));
                        } else {
                            emsg(gettext(
                                b"E428: Cannot go beyond last matching tag\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            ));
                        }
                        skip_msg = true_0 != 0;
                    }
                    cur_match = num_matches.get() - 1 as ::core::ffi::c_int;
                }
                if use_tagstack {
                    let mut tagp2_0: TagParts = TagParts::default();
                    (*tagstack.offset(tagstackidx as isize)).cur_match = cur_match;
                    (*tagstack.offset(tagstackidx as isize)).cur_fnum = cur_fnum;
                    if use_tfu as ::core::ffi::c_int != 0
                        && parse_match(*(*matches.ptr()).offset(cur_match as isize), &mut tagp2_0)
                        && !tagp2_0.user_data.is_null()
                    {
                        let mut ptr_: *mut *mut ::core::ffi::c_void =
                            &raw mut (*tagstack.offset(tagstackidx as isize)).user_data
                                as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr_);
                        *ptr_ = NULL_0;
                        let _ = *ptr_;
                        (*tagstack.offset(tagstackidx as isize)).user_data = xmemdupz(
                            tagp2_0.user_data as *const ::core::ffi::c_void,
                            tagp2_0.user_data_end.offset_from(tagp2_0.user_data) as size_t,
                        )
                            as *mut ::core::ffi::c_char;
                    }
                    tagstackidx += 1;
                } else if g_do_tagpreview.get() != 0 as ::core::ffi::c_int {
                    (*ptag_entry.ptr()).cur_match = cur_match;
                    (*ptag_entry.ptr()).cur_fnum = cur_fnum;
                }
                if !(*nofile_fname.ptr()).is_null() && error_cur_match != cur_match {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(
                            b"File \"%s\" does not exist\0".as_ptr() as *const ::core::ffi::c_char
                        ),
                        nofile_fname.get(),
                    );
                }
                let mut ic: bool = *(*(*matches.ptr()).offset(cur_match as isize))
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & MT_IC_OFF as ::core::ffi::c_int
                    != 0;
                if type_0 != DT_TAG as ::core::ffi::c_int
                    && type_0 != DT_SELECT as ::core::ffi::c_int
                    && type_0 != DT_JUMP as ::core::ffi::c_int
                    && (num_matches.get() > 1 as ::core::ffi::c_int
                        || ic as ::core::ffi::c_int != 0)
                    && !skip_msg
                {
                    snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
                        gettext(b"tag %d of %d%s\0".as_ptr() as *const ::core::ffi::c_char),
                        cur_match + 1 as ::core::ffi::c_int,
                        num_matches.get(),
                        if max_num_matches.get() != MAXCOL as ::core::ffi::c_int {
                            gettext(b" or more\0".as_ptr() as *const ::core::ffi::c_char)
                                as *const ::core::ffi::c_char
                        } else {
                            b"\0".as_ptr() as *const ::core::ffi::c_char
                        },
                    );
                    if ic {
                        xstrlcat(
                            IObuff.ptr() as *mut ::core::ffi::c_char,
                            gettext(b"  Using tag with different case!\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            IOSIZE as size_t,
                        );
                    }
                    if (num_matches.get() > prev_num_matches || new_tag as ::core::ffi::c_int != 0)
                        && num_matches.get() > 1 as ::core::ffi::c_int
                    {
                        msg(
                            IObuff.ptr() as *mut ::core::ffi::c_char,
                            if ic as ::core::ffi::c_int != 0 {
                                HLF_W as ::core::ffi::c_int
                            } else {
                                0 as ::core::ffi::c_int
                            },
                        );
                        msg_scroll.set(true_0);
                    } else {
                        give_warning(IObuff.ptr() as *mut ::core::ffi::c_char, ic, true_0 != 0);
                    }
                    if ic as ::core::ffi::c_int != 0
                        && msg_scrolled.get() == 0
                        && msg_silent.get() == 0 as ::core::ffi::c_int
                    {
                        msg_delay(1007 as uint64_t, true_0 != 0);
                    }
                }
                let mut IObufflen: size_t = vim_snprintf_safelen(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    b":ta %s\r\0".as_ptr() as *const ::core::ffi::c_char,
                    name,
                );
                set_vim_var_string(
                    VV_SWAPCOMMAND,
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IObufflen as ptrdiff_t,
                );
                let mut i_2: ::core::ffi::c_int = jumpto_tag(
                    *(*matches.ptr()).offset(cur_match as isize),
                    forceit,
                    true_0 != 0,
                );
                set_vim_var_string(
                    VV_SWAPCOMMAND,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    -1 as ptrdiff_t,
                );
                if i_2 == NOTAGFILE {
                    if type_0 == DT_PREV as ::core::ffi::c_int
                        && cur_match > 0 as ::core::ffi::c_int
                        || (type_0 == DT_TAG as ::core::ffi::c_int
                            || type_0 == DT_NEXT as ::core::ffi::c_int
                            || type_0 == DT_FIRST as ::core::ffi::c_int)
                            && (max_num_matches.get() != MAXCOL as ::core::ffi::c_int
                                || cur_match < num_matches.get() - 1 as ::core::ffi::c_int)
                    {
                        error_cur_match = cur_match;
                        if use_tagstack {
                            tagstackidx -= 1;
                        }
                        if type_0 == DT_PREV as ::core::ffi::c_int {
                            cur_match -= 1;
                        } else {
                            type_0 = DT_NEXT as ::core::ffi::c_int;
                            cur_match += 1;
                        }
                    } else {
                        semsg(
                            gettext(b"E429: File \"%s\" does not exist\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            nofile_fname.get(),
                        );
                        break '_end_do_tag;
                    }
                } else {
                    if use_tagstack as ::core::ffi::c_int != 0
                        && tagstackidx > (*curwin.get()).w_tagstacklen
                    {
                        tagstackidx = (*curwin.get()).w_tagstackidx;
                    }
                    break '_end_do_tag;
                }
            }
        }
    }
    if use_tagstack as ::core::ffi::c_int != 0 && tagstackidx <= (*curwin.get()).w_tagstacklen {
        (*curwin.get()).w_tagstackidx = tagstackidx;
    }
    postponed_split.set(0 as ::core::ffi::c_int);
    g_do_tagpreview.set(0 as ::core::ffi::c_int);
    xfree(tofree as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn tag_freematch() {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        tagmatchname.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    let _ = *ptr_;
}
pub const ML_EXTRA: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
