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
use crate::src::nvim::garray::{ga_clear, ga_clear_strings, ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::hashtab::{hash_add_item, hash_clear, hash_hash, hash_init, hash_lookup};
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
    __assert_fail, __ctype_b_loc, abort, atoi, fclose, fseek, fseeko, ftello, gettext, memmove,
    memset, snprintf, strcasecmp, strcat, strcmp, strcpy, strlen, strncasecmp, strncmp, strstr,
};
use crate::src::nvim::path::{
    FreeWild, FullName_save, path_full_compare, path_has_wildcard, path_tail, simplify_filename,
    vim_isAbsName,
};
use crate::src::nvim::pos::{MAXLNUM, clearpos};
use crate::src::nvim::quickfix::set_errorlist;
use crate::src::nvim::regexp::skip_regexp;
use crate::src::nvim::runtime::do_in_runtimepath;
use crate::src::nvim::search::{do_search, ignorecase, ignorecase_opt};
use crate::src::nvim::state::MODE_INSERT;
use crate::src::nvim::strings::{vim_snprintf, vim_snprintf_safelen, vim_strchr, xstrnsave};
use crate::src::nvim::types::ui::kUIMessages;
use crate::src::nvim::types::{
    __off_t, AdditionalData, Callback, Callback_data as C2Rust_Unnamed_5, Direction, FILE, OptInt,
    SpecialVarValue, Timestamp, VarLockStatus, VarType, VimVarIndex, buf_T, colnr_T, dict_T,
    dictitem_T, exarg_T, expand_T, file_comparison, fmark_T, fmarkv_T, garray_T, getf_retvalues,
    getf_values, hash_T, hashitem_T, hashtab_T, ht_stack_T, int64_t, linenr_T, list_T,
    list_stack_T, listitem_T, off_T, oparg_T, optmagic_T, optset_T, pos_T, ptrdiff_t, regmatch_T,
    regprog_T, sctx_T, searchit_arg_T, size_t, taggy_T, tagname_T, typval_T, typval_vval_union,
    uint8_t, uint64_t, varnumber_T, vimconv_T, win_T, xp_prefix_T,
};
use crate::src::nvim::ui::ui_has;
use crate::src::nvim::window::{
    check_can_set_curbuf_forceit, swbuf_goto_win_with_buf, tabpage_index, win_close, win_enter,
    win_split, win_valid,
};
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tagptrs_T {
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct findtags_state_T {
    pub state: tagsearch_state_T,
    pub stop_searching: bool,
    pub orgpat: *mut pat_T,
    pub lbuf: *mut ::core::ffi::c_char,
    pub lbuf_size: ::core::ffi::c_int,
    pub tag_fname: *mut ::core::ffi::c_char,
    pub fp: *mut FILE,
    pub flags: ::core::ffi::c_int,
    pub tag_file_sorted: ::core::ffi::c_int,
    pub get_searchpat: bool,
    pub help_only: bool,
    pub did_open: bool,
    pub mincount: ::core::ffi::c_int,
    pub linear: bool,
    pub vimconv: vimconv_T,
    pub help_lang: [::core::ffi::c_char; 3],
    pub help_pri: ::core::ffi::c_int,
    pub help_lang_find: *mut ::core::ffi::c_char,
    pub is_txt: bool,
    pub match_count: ::core::ffi::c_int,
    pub ga_match: [garray_T; 16],
    pub ht_match: [hashtab_T; 16],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pat_T {
    pub pat: *mut ::core::ffi::c_char,
    pub len: ::core::ffi::c_int,
    pub head: *mut ::core::ffi::c_char,
    pub headlen: ::core::ffi::c_int,
    pub regmatch: regmatch_T,
}
pub type tagsearch_state_T = ::core::ffi::c_uint;
pub const TS_STEP_FORWARD: tagsearch_state_T = 4;
pub const TS_SKIP_BACK: tagsearch_state_T = 3;
pub const TS_BINARY: tagsearch_state_T = 2;
pub const TS_LINEAR: tagsearch_state_T = 1;
pub const TS_START: tagsearch_state_T = 0;
pub const MT_COUNT: C2Rust_Unnamed_35 = 16;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct findtags_match_args_T {
    pub matchoff: ::core::ffi::c_int,
    pub match_re: bool,
    pub match_no_ic: bool,
    pub has_re: bool,
    pub sortic: bool,
    pub sort_error: bool,
}
pub const MT_GL_OTH: C2Rust_Unnamed_35 = 2;
pub const MT_GL_CUR: C2Rust_Unnamed_35 = 1;
pub const MT_ST_OTH: C2Rust_Unnamed_35 = 3;
pub const MT_ST_CUR: C2Rust_Unnamed_35 = 0;
pub const MT_RE_OFF: C2Rust_Unnamed_35 = 8;
pub const TAG_MATCH_FAIL: tagmatch_status_T = 2;
pub type tags_read_status_T = ::core::ffi::c_uint;
pub const TAGS_READ_IGNORE: tags_read_status_T = 3;
pub const TAGS_READ_EOF: tags_read_status_T = 2;
pub const TAGS_READ_SUCCESS: tags_read_status_T = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tagsearch_info_T {
    pub low_offset: off_T,
    pub high_offset: off_T,
    pub curr_offset: off_T,
    pub curr_offset_used: off_T,
    pub match_offset: off_T,
    pub low_char: ::core::ffi::c_int,
    pub high_char: ::core::ffi::c_int,
}
pub const TAG_MATCH_STOP: tagmatch_status_T = 3;
pub const TAG_MATCH_NEXT: tagmatch_status_T = 4;
pub type tagmatch_status_T = ::core::ffi::c_uint;
pub const TAG_MATCH_SUCCESS: tagmatch_status_T = 1;
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
pub unsafe extern "C" fn did_set_tagfunc(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut retval: ::core::ffi::c_int = 0;
    if (*args).os_flags & OPT_LOCAL as ::core::ffi::c_int != 0 {
        retval = option_set_callback_func((*args).os_newval.string.data, &raw mut (*buf).b_tfu_cb);
    } else {
        retval = option_set_callback_func((*args).os_newval.string.data, tfu_cb.ptr());
        if retval == OK && (*args).os_flags & OPT_GLOBAL as ::core::ffi::c_int == 0 {
            set_buflocal_tfu_callback(buf);
        }
    }
    return if retval == FAIL {
        &raw const e_invarg as *const ::core::ffi::c_char
    } else {
        ::core::ptr::null::<::core::ffi::c_char>()
    };
}
pub unsafe extern "C" fn set_ref_in_tagfunc(mut copyID: ::core::ffi::c_int) -> bool {
    return set_ref_in_callback(
        tfu_cb.ptr(),
        copyID,
        ::core::ptr::null_mut::<*mut ht_stack_T>(),
        ::core::ptr::null_mut::<*mut list_stack_T>(),
    );
}
pub unsafe extern "C" fn set_buflocal_tfu_callback(mut buf: *mut buf_T) {
    callback_free(&raw mut (*buf).b_tfu_cb);
    if (*tfu_cb.ptr()).type_0 as ::core::ffi::c_uint
        != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        callback_copy(&raw mut (*buf).b_tfu_cb, tfu_cb.ptr());
    }
}
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
                tagstack_clear_entry(ptag_entry.ptr());
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
                        tagstack_clear_entry(ptag_entry.ptr());
                        (*ptag_entry.ptr()).tagname = xstrdup(tag);
                    }
                } else {
                    while tagstackidx < tagstacklen {
                        tagstacklen -= 1;
                        tagstack_clear_entry(tagstack.offset(tagstacklen as isize));
                    }
                    tagstacklen += 1;
                    if tagstacklen > TAGSTACKSIZE {
                        tagstacklen = TAGSTACKSIZE;
                        tagstack_clear_entry(tagstack.offset(0 as ::core::ffi::c_int as isize));
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
                        let mut tagp: tagptrs_T = tagptrs_T {
                            tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tagline: 0,
                        };
                        let mut tagp2: tagptrs_T = tagptrs_T {
                            tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tagline: 0,
                        };
                        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while j < num_matches.get() {
                            parse_match(*(*matches.ptr()).offset(j as isize), &raw mut tagp);
                            let mut i_0: ::core::ffi::c_int = idx;
                            while i_0 < new_num_matches {
                                parse_match(*new_matches.offset(i_0 as isize), &raw mut tagp2);
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
                    let mut tagp2_0: tagptrs_T = tagptrs_T {
                        tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        tagline: 0,
                    };
                    (*tagstack.offset(tagstackidx as isize)).cur_match = cur_match;
                    (*tagstack.offset(tagstackidx as isize)).cur_fnum = cur_fnum;
                    if use_tfu as ::core::ffi::c_int != 0
                        && parse_match(
                            *(*matches.ptr()).offset(cur_match as isize),
                            &raw mut tagp2_0,
                        ) == OK
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
unsafe extern "C" fn print_tag_list(
    mut new_tag: bool,
    mut use_tagstack: bool,
    mut num_matches: ::core::ffi::c_int,
    mut matches: *mut *mut ::core::ffi::c_char,
) {
    let mut tagstack: *mut taggy_T = &raw mut (*curwin.get()).w_tagstack as *mut taggy_T;
    let mut tagstackidx: ::core::ffi::c_int = (*curwin.get()).w_tagstackidx;
    let mut tagp: tagptrs_T = tagptrs_T {
        tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagline: 0,
    };
    parse_match(
        *matches.offset(0 as ::core::ffi::c_int as isize),
        &raw mut tagp,
    );
    let mut taglen: ::core::ffi::c_int = if (tagp.tagname_end.offset_from(tagp.tagname)
        + 2 as isize) as ::core::ffi::c_int
        > 18 as ::core::ffi::c_int
    {
        (tagp.tagname_end.offset_from(tagp.tagname) + 2 as isize) as ::core::ffi::c_int
    } else {
        18 as ::core::ffi::c_int
    };
    if taglen > Columns.get() - 25 as ::core::ffi::c_int {
        taglen = MAXCOL as ::core::ffi::c_int;
    }
    if msg_col.get() == 0 as ::core::ffi::c_int {
        msg_didout.set(false_0 != 0);
    }
    msg_ext_set_kind(b"confirm\0".as_ptr() as *const ::core::ffi::c_char);
    msg_start();
    msg_puts_hl(
        gettext(b"  # pri kind tag\0".as_ptr() as *const ::core::ffi::c_char),
        HLF_T as ::core::ffi::c_int,
        false_0 != 0,
    );
    msg_clr_eos();
    taglen_advance(taglen);
    msg_puts_hl(
        gettext(b"file\n\0".as_ptr() as *const ::core::ffi::c_char),
        HLF_T as ::core::ffi::c_int,
        false_0 != 0,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_matches && !got_int.get() {
        parse_match(*matches.offset(i as isize), &raw mut tagp);
        if !new_tag
            && (g_do_tagpreview.get() != 0 as ::core::ffi::c_int
                && i == (*ptag_entry.ptr()).cur_match
                || use_tagstack as ::core::ffi::c_int != 0
                    && i == (*tagstack.offset(tagstackidx as isize)).cur_match)
        {
            *(IObuff.ptr() as *mut ::core::ffi::c_char) = '>' as ::core::ffi::c_char;
        } else {
            *(IObuff.ptr() as *mut ::core::ffi::c_char) = ' ' as ::core::ffi::c_char;
        }
        vim_snprintf(
            (IObuff.ptr() as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize),
            (IOSIZE - 1 as ::core::ffi::c_int) as size_t,
            b"%2d %s \0".as_ptr() as *const ::core::ffi::c_char,
            i + 1 as ::core::ffi::c_int,
            (*mt_names.ptr())[(*(*matches.offset(i as isize))
                .offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & MT_MASK as ::core::ffi::c_int) as usize],
        );
        msg_puts(IObuff.ptr() as *mut ::core::ffi::c_char);
        if !tagp.tagkind.is_null() {
            msg_outtrans_len(
                tagp.tagkind,
                tagp.tagkind_end.offset_from(tagp.tagkind) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        }
        msg_advance(13 as ::core::ffi::c_int);
        msg_outtrans_len(
            tagp.tagname,
            tagp.tagname_end.offset_from(tagp.tagname) as ::core::ffi::c_int,
            HLF_T as ::core::ffi::c_int,
            false_0 != 0,
        );
        msg_putchar(' ' as ::core::ffi::c_int);
        taglen_advance(taglen);
        let mut p: *const ::core::ffi::c_char = tag_full_fname(&raw mut tagp);
        if !p.is_null() {
            msg_outtrans(p, HLF_D as ::core::ffi::c_int, false_0 != 0);
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut p as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
        }
        if msg_col.get() > 0 as ::core::ffi::c_int {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        if got_int.get() {
            break;
        }
        msg_advance(15 as ::core::ffi::c_int);
        let mut command_end: *const ::core::ffi::c_char = tagp.command_end;
        if !command_end.is_null() {
            p = command_end.offset(3 as ::core::ffi::c_int as isize);
            while *p as ::core::ffi::c_int != 0
                && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
            {
                while *p as ::core::ffi::c_int == TAB {
                    p = p.offset(1);
                }
                if strncmp(
                    p,
                    b"file:\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
                    && ascii_isspace(
                        *p.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                {
                    p = p.offset(5 as ::core::ffi::c_int as isize);
                } else if p == tagp.tagkind as *const ::core::ffi::c_char
                    || p.offset(5 as ::core::ffi::c_int as isize)
                        == tagp.tagkind as *const ::core::ffi::c_char
                        && strncmp(
                            p,
                            b"kind:\0".as_ptr() as *const ::core::ffi::c_char,
                            5 as size_t,
                        ) == 0 as ::core::ffi::c_int
                {
                    p = tagp.tagkind_end;
                } else {
                    let mut hl_id: ::core::ffi::c_int = HLF_CM as ::core::ffi::c_int;
                    while *p as ::core::ffi::c_int != 0
                        && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                        && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                    {
                        if msg_col.get() + ptr2cells(p) >= Columns.get() {
                            msg_putchar('\n' as ::core::ffi::c_int);
                            if got_int.get() {
                                break;
                            }
                            msg_advance(15 as ::core::ffi::c_int);
                        }
                        p = msg_outtrans_one(p, hl_id, false_0 != 0);
                        if *p as ::core::ffi::c_int == TAB {
                            msg_puts_hl(
                                b" \0".as_ptr() as *const ::core::ffi::c_char,
                                hl_id,
                                false_0 != 0,
                            );
                            break;
                        } else if *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
                            hl_id = 0 as ::core::ffi::c_int;
                        }
                    }
                }
            }
            if msg_col.get() > 15 as ::core::ffi::c_int {
                msg_putchar('\n' as ::core::ffi::c_int);
                if got_int.get() {
                    break;
                }
                msg_advance(15 as ::core::ffi::c_int);
            }
        } else {
            p = tagp.command;
            while *p as ::core::ffi::c_int != 0
                && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            command_end = p;
        }
        p = tagp.command;
        if *p as ::core::ffi::c_int == '/' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == '?' as ::core::ffi::c_int
        {
            p = p.offset(1);
            if *p as ::core::ffi::c_int == '^' as ::core::ffi::c_int {
                p = p.offset(1);
            }
        }
        while p != command_end && ascii_isspace(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        {
            p = p.offset(1);
        }
        while p != command_end {
            if msg_col.get()
                + (if *p as ::core::ffi::c_int == TAB {
                    1 as ::core::ffi::c_int
                } else {
                    ptr2cells(p)
                })
                > Columns.get()
            {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            if got_int.get() {
                break;
            }
            msg_advance(15 as ::core::ffi::c_int);
            if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == *tagp.command as ::core::ffi::c_int
                    || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int)
            {
                p = p.offset(1);
            }
            if *p as ::core::ffi::c_int == TAB {
                msg_putchar(' ' as ::core::ffi::c_int);
                p = p.offset(1);
            } else {
                p = msg_outtrans_one(p, 0 as ::core::ffi::c_int, false_0 != 0);
            }
            if p == command_end.offset(-(2 as ::core::ffi::c_int as isize))
                && *p as ::core::ffi::c_int == '$' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == *tagp.command as ::core::ffi::c_int
            {
                break;
            }
            if p == command_end.offset(-(1 as ::core::ffi::c_int as isize))
                && *p as ::core::ffi::c_int == *tagp.command as ::core::ffi::c_int
                && (*p as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == '?' as ::core::ffi::c_int)
            {
                break;
            }
        }
        if msg_col.get() != 0 && (!ui_has(kUIMessages) || i < num_matches - 1 as ::core::ffi::c_int)
        {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        os_breakcheck();
        i += 1;
    }
    if got_int.get() {
        got_int.set(false_0 != 0);
    }
}
unsafe extern "C" fn add_llist_tags(
    mut tag: *mut ::core::ffi::c_char,
    mut num_matches: ::core::ffi::c_int,
    mut matches: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut tag_name: [::core::ffi::c_char; 129] = [0; 129];
    let mut tagp: tagptrs_T = tagptrs_T {
        tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagline: 0,
    };
    let mut fname: *mut ::core::ffi::c_char =
        xmalloc((MAXPATHL + 1 as ::core::ffi::c_int) as size_t) as *mut ::core::ffi::c_char;
    let mut cmd: *mut ::core::ffi::c_char =
        xmalloc((CMDBUFFSIZE + 1 as ::core::ffi::c_int) as size_t) as *mut ::core::ffi::c_char;
    let mut list: *mut list_T = tv_list_alloc(0 as ptrdiff_t);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_matches {
        let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        parse_match(*matches.offset(i as isize), &raw mut tagp);
        let mut len: ::core::ffi::c_int = if (tagp.tagname_end.offset_from(tagp.tagname)
            as ::core::ffi::c_int)
            < 128 as ::core::ffi::c_int
        {
            tagp.tagname_end.offset_from(tagp.tagname) as ::core::ffi::c_int
        } else {
            128 as ::core::ffi::c_int
        };
        xmemcpyz(
            &raw mut tag_name as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            tagp.tagname as *const ::core::ffi::c_void,
            len as size_t,
        );
        tag_name[len as usize] = NUL as ::core::ffi::c_char;
        let mut p: *mut ::core::ffi::c_char = tag_full_fname(&raw mut tagp);
        if !p.is_null() {
            xstrlcpy(fname, p, MAXPATHL as size_t);
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut p as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
            let mut lnum: linenr_T = 0 as linenr_T;
            if *(*__ctype_b_loc()).offset(*tagp.command as uint8_t as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
            {
                lnum = atoi(tagp.command) as linenr_T;
            } else {
                let mut cmd_start: *mut ::core::ffi::c_char = tagp.command;
                let mut cmd_end: *mut ::core::ffi::c_char = tagp.command_end;
                if cmd_end.is_null() {
                    p = tagp.command;
                    while *p as ::core::ffi::c_int != 0
                        && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                        && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                    {
                        p = p.offset(1);
                    }
                    cmd_end = p;
                }
                cmd_end = cmd_end.offset(-1);
                if *cmd_start as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                    || *cmd_start as ::core::ffi::c_int == '?' as ::core::ffi::c_int
                {
                    cmd_start = cmd_start.offset(1);
                }
                if *cmd_end as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                    || *cmd_end as ::core::ffi::c_int == '?' as ::core::ffi::c_int
                {
                    cmd_end = cmd_end.offset(-1);
                }
                len = 0 as ::core::ffi::c_int;
                *cmd.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
                if *cmd_start as ::core::ffi::c_int == '^' as ::core::ffi::c_int {
                    strcpy(
                        cmd,
                        b"^\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    );
                    cmd_start = cmd_start.offset(1);
                    len += 1;
                }
                strcat(cmd, b"\\V\0".as_ptr() as *const ::core::ffi::c_char);
                len += 2 as ::core::ffi::c_int;
                let mut cmd_len: ::core::ffi::c_int =
                    if ((cmd_end.offset_from(cmd_start) + 1 as isize) as ::core::ffi::c_int)
                        < 1024 as ::core::ffi::c_int - 5 as ::core::ffi::c_int
                    {
                        (cmd_end.offset_from(cmd_start) + 1 as isize) as ::core::ffi::c_int
                    } else {
                        1024 as ::core::ffi::c_int - 5 as ::core::ffi::c_int
                    };
                snprintf(
                    cmd.offset(len as isize),
                    (CMDBUFFSIZE + 1 as ::core::ffi::c_int - len) as size_t,
                    b"%.*s\0".as_ptr() as *const ::core::ffi::c_char,
                    cmd_len,
                    cmd_start,
                );
                len += cmd_len;
                if *cmd.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                    == '$' as ::core::ffi::c_int
                {
                    *cmd.offset((len - 1 as ::core::ffi::c_int) as isize) =
                        '\\' as ::core::ffi::c_char;
                    *cmd.offset(len as isize) = '$' as ::core::ffi::c_char;
                    len += 1;
                }
                *cmd.offset(len as isize) = NUL as ::core::ffi::c_char;
            }
            dict = tv_dict_alloc();
            tv_list_append_dict(list, dict);
            tv_dict_add_str(
                dict,
                b"text\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                &raw mut tag_name as *mut ::core::ffi::c_char,
            );
            tv_dict_add_str(
                dict,
                b"filename\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                fname,
            );
            tv_dict_add_nr(
                dict,
                b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                lnum as varnumber_T,
            );
            if lnum == 0 as linenr_T {
                tv_dict_add_str(
                    dict,
                    b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                    cmd,
                );
            }
        }
        i += 1;
    }
    vim_snprintf(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        b"ltag %s\0".as_ptr() as *const ::core::ffi::c_char,
        tag,
    );
    set_errorlist(
        curwin.get(),
        list,
        ' ' as ::core::ffi::c_int,
        IObuff.ptr() as *mut ::core::ffi::c_char,
        ::core::ptr::null_mut::<dict_T>(),
    );
    tv_list_free(list);
    let mut ptr__0: *mut *mut ::core::ffi::c_void = &raw mut fname as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL_0;
    let _ = *ptr__0;
    let mut ptr__1: *mut *mut ::core::ffi::c_void = &raw mut cmd as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__1);
    *ptr__1 = NULL_0;
    let _ = *ptr__1;
    return OK;
}
pub unsafe extern "C" fn tag_freematch() {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        tagmatchname.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    let _ = *ptr_;
}
unsafe extern "C" fn taglen_advance(mut l: ::core::ffi::c_int) {
    if l == MAXCOL as ::core::ffi::c_int {
        msg_putchar('\n' as ::core::ffi::c_int);
        msg_advance(24 as ::core::ffi::c_int);
    } else {
        msg_advance(13 as ::core::ffi::c_int + l);
    };
}
pub unsafe fn do_tags(mut _eap: *mut exarg_T) {
    let mut tagstack: *mut taggy_T = &raw mut (*curwin.get()).w_tagstack as *mut taggy_T;
    let mut tagstackidx: ::core::ffi::c_int = (*curwin.get()).w_tagstackidx;
    let mut tagstacklen: ::core::ffi::c_int = (*curwin.get()).w_tagstacklen;
    msg_puts_title(gettext(
        b"\n  # TO tag         FROM line  in file/text\0".as_ptr() as *const ::core::ffi::c_char,
    ));
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < tagstacklen {
        if !(*tagstack.offset(i as isize)).tagname.is_null() {
            let mut name: *mut ::core::ffi::c_char = fm_getname(
                &raw mut (*tagstack.offset(i as isize)).fmark,
                30 as ::core::ffi::c_int,
            );
            if !name.is_null() {
                msg_putchar('\n' as ::core::ffi::c_int);
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    b"%c%2d %2d %-15s %5d  \0".as_ptr() as *const ::core::ffi::c_char,
                    if i == tagstackidx {
                        '>' as ::core::ffi::c_int
                    } else {
                        ' ' as ::core::ffi::c_int
                    },
                    i + 1 as ::core::ffi::c_int,
                    (*tagstack.offset(i as isize)).cur_match + 1 as ::core::ffi::c_int,
                    (*tagstack.offset(i as isize)).tagname,
                    (*tagstack.offset(i as isize)).fmark.mark.lnum,
                );
                msg_outtrans(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
                msg_outtrans(
                    name,
                    if (*tagstack.offset(i as isize)).fmark.fnum == (*curbuf.get()).handle {
                        HLF_D as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    },
                    false_0 != 0,
                );
                xfree(name as *mut ::core::ffi::c_void);
            }
        }
        i += 1;
    }
    if tagstackidx == tagstacklen {
        msg_puts(b"\n>\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
unsafe extern "C" fn tag_strnicmp(
    mut s1: *mut ::core::ffi::c_char,
    mut s2: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    while len > 0 as size_t {
        let mut i: ::core::ffi::c_int =
            (if (*s1 as uint8_t as ::core::ffi::c_int) < 'a' as ::core::ffi::c_int
                || *s1 as uint8_t as ::core::ffi::c_int > 'z' as ::core::ffi::c_int
            {
                *s1 as uint8_t as ::core::ffi::c_int
            } else {
                *s1 as uint8_t as ::core::ffi::c_int
                    - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) - (if (*s2 as uint8_t as ::core::ffi::c_int) < 'a' as ::core::ffi::c_int
                || *s2 as uint8_t as ::core::ffi::c_int > 'z' as ::core::ffi::c_int
            {
                *s2 as uint8_t as ::core::ffi::c_int
            } else {
                *s2 as uint8_t as ::core::ffi::c_int
                    - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            });
        if i != 0 as ::core::ffi::c_int {
            return i;
        }
        if *s1 as ::core::ffi::c_int == NUL {
            break;
        }
        s1 = s1.offset(1);
        s2 = s2.offset(1);
        len = len.wrapping_sub(1);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn prepare_pats(mut pats: *mut pat_T, mut has_re: bool) {
    (*pats).head = (*pats).pat;
    (*pats).headlen = (*pats).len;
    if has_re {
        if *(*pats).pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '^' as ::core::ffi::c_int
        {
            (*pats).head = (*pats).pat.offset(1 as ::core::ffi::c_int as isize);
        } else if *(*pats).pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '\\' as ::core::ffi::c_int
            && *(*pats).pat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '<' as ::core::ffi::c_int
        {
            (*pats).head = (*pats).pat.offset(2 as ::core::ffi::c_int as isize);
        }
        if (*pats).head == (*pats).pat {
            (*pats).headlen = 0 as ::core::ffi::c_int;
        } else {
            (*pats).headlen = 0 as ::core::ffi::c_int;
            while *(*pats).head.offset((*pats).headlen as isize) as ::core::ffi::c_int != NUL {
                if !vim_strchr(
                    if magic_isset() as ::core::ffi::c_int != 0 {
                        b".[~*\\$\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"\\$\0".as_ptr() as *const ::core::ffi::c_char
                    },
                    *(*pats).head.offset((*pats).headlen as isize) as uint8_t as ::core::ffi::c_int,
                )
                .is_null()
                {
                    break;
                }
                (*pats).headlen += 1;
            }
        }
        if p_tl.get() != 0 as OptInt && (*pats).headlen as OptInt > p_tl.get() {
            (*pats).headlen = p_tl.get() as ::core::ffi::c_int;
        }
    }
    if has_re {
        (*pats).regmatch.regprog = vim_regcomp(
            (*pats).pat,
            if magic_isset() as ::core::ffi::c_int != 0 {
                RE_MAGIC
            } else {
                0 as ::core::ffi::c_int
            },
        );
    } else {
        (*pats).regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
    };
}
unsafe extern "C" fn find_tagfunc_tags(
    mut pat: *mut ::core::ffi::c_char,
    mut ga: *mut garray_T,
    mut match_count: *mut ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut buf_ffname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut ntags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut args: [typval_T; 4] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 4];
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut flagString: [::core::ffi::c_char; 4] = [0; 4];
    let mut tag: *mut taggy_T = ::core::ptr::null_mut::<taggy_T>();
    if (*curwin.get()).w_tagstacklen > 0 as ::core::ffi::c_int {
        if (*curwin.get()).w_tagstackidx == (*curwin.get()).w_tagstacklen {
            tag = (&raw mut (*curwin.get()).w_tagstack as *mut taggy_T)
                .offset(((*curwin.get()).w_tagstackidx - 1 as ::core::ffi::c_int) as isize);
        } else {
            tag = (&raw mut (*curwin.get()).w_tagstack as *mut taggy_T)
                .offset((*curwin.get()).w_tagstackidx as isize);
        }
    }
    if *(*curbuf.get()).b_p_tfu as ::core::ffi::c_int == NUL
        || (*curbuf.get()).b_tfu_cb.type_0 as ::core::ffi::c_uint
            == kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return FAIL;
    }
    args[0 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
    args[0 as ::core::ffi::c_int as usize].vval.v_string = pat;
    args[1 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
    args[1 as ::core::ffi::c_int as usize].vval.v_string =
        &raw mut flagString as *mut ::core::ffi::c_char;
    let d: *mut dict_T = tv_dict_alloc_lock(VAR_FIXED);
    if flags & TAG_INS_COMP as ::core::ffi::c_int == 0
        && !tag.is_null()
        && !(*tag).user_data.is_null()
    {
        tv_dict_add_str(
            d,
            b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            (*tag).user_data,
        );
    }
    if !buf_ffname.is_null() {
        tv_dict_add_str(
            d,
            b"buf_ffname\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
            buf_ffname,
        );
    }
    (*d).dv_refcount += 1;
    args[2 as ::core::ffi::c_int as usize].v_type = VAR_DICT;
    args[2 as ::core::ffi::c_int as usize].vval.v_dict = d;
    args[3 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
    vim_snprintf(
        &raw mut flagString as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>(),
        b"%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
        if g_tag_at_cursor.get() as ::core::ffi::c_int != 0 {
            b"c\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        if flags & TAG_INS_COMP as ::core::ffi::c_int != 0 {
            b"i\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        if flags & TAG_REGEXP as ::core::ffi::c_int != 0 {
            b"r\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
    );
    let mut save_pos: pos_T = (*curwin.get()).w_cursor;
    let mut result: ::core::ffi::c_int = callback_call(
        &raw mut (*curbuf.get()).b_tfu_cb,
        3 as ::core::ffi::c_int,
        &raw mut args as *mut typval_T,
        &raw mut rettv,
    ) as ::core::ffi::c_int;
    (*curwin.get()).w_cursor = save_pos;
    check_cursor(curwin.get());
    (*d).dv_refcount -= 1;
    if result == FAIL {
        return FAIL;
    }
    if rettv.v_type as ::core::ffi::c_uint
        == VAR_SPECIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        && rettv.vval.v_special as ::core::ffi::c_uint
            == kSpecialVarNull as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_clear(&raw mut rettv);
        return NOTDONE;
    }
    if rettv.v_type as ::core::ffi::c_uint != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || rettv.vval.v_list.is_null()
    {
        tv_clear(&raw mut rettv);
        emsg(gettext(
            (e_invalid_return_value_from_tagfunc.ptr() as *const _) as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    let mut taglist: *mut list_T = rettv.vval.v_list;
    let l_: *const list_T = taglist;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            let mut res_name: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut res_fname: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut res_cmd: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut res_kind: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut has_extra: bool = false;
            let mut name_only: ::core::ffi::c_int = flags & TAG_NAMES as ::core::ffi::c_int;
            if (*li).li_tv.v_type as ::core::ffi::c_uint
                != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                emsg(gettext(
                    (e_invalid_return_value_from_tagfunc.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ));
                break;
            } else {
                let mut len: size_t = 2 as size_t;
                res_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                res_fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
                res_cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
                res_kind = ::core::ptr::null_mut::<::core::ffi::c_char>();
                let dihi_ht_: *mut hashtab_T = &raw mut (*(*li).li_tv.vval.v_dict).dv_hashtab;
                let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
                let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
                while dihi_todo_ != 0 {
                    if !((*dihi_).hi_key.is_null()
                        || (*dihi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
                    {
                        dihi_todo_ = dihi_todo_.wrapping_sub(1);
                        let di: *mut dictitem_T = (*dihi_)
                            .hi_key
                            .offset(-(17 as ::core::ffi::c_ulong as isize))
                            as *mut dictitem_T;
                        let mut dict_key: *const ::core::ffi::c_char =
                            &raw mut (*di).di_key as *mut ::core::ffi::c_char;
                        let mut tv: *mut typval_T = &raw mut (*di).di_tv;
                        if !((*tv).v_type as ::core::ffi::c_uint
                            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                            || (*tv).vval.v_string.is_null())
                        {
                            len = len.wrapping_add(
                                strlen((*tv).vval.v_string).wrapping_add(1 as size_t),
                            );
                            if strcmp(dict_key, b"name\0".as_ptr() as *const ::core::ffi::c_char)
                                == 0
                            {
                                res_name = (*tv).vval.v_string;
                            } else if strcmp(
                                dict_key,
                                b"filename\0".as_ptr() as *const ::core::ffi::c_char,
                            ) == 0
                            {
                                res_fname = (*tv).vval.v_string;
                            } else if strcmp(
                                dict_key,
                                b"cmd\0".as_ptr() as *const ::core::ffi::c_char,
                            ) == 0
                            {
                                res_cmd = (*tv).vval.v_string;
                            } else {
                                has_extra = true;
                                if strcmp(
                                    dict_key,
                                    b"kind\0".as_ptr() as *const ::core::ffi::c_char,
                                ) == 0
                                {
                                    res_kind = (*tv).vval.v_string;
                                } else {
                                    len = len
                                        .wrapping_add(strlen(dict_key).wrapping_add(1 as size_t));
                                }
                            }
                        }
                    }
                    dihi_ = dihi_.offset(1);
                }
                if has_extra {
                    len = len.wrapping_add(2 as size_t);
                }
                if res_name.is_null() || res_fname.is_null() || res_cmd.is_null() {
                    emsg(gettext(
                        (e_invalid_return_value_from_tagfunc.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ));
                    break;
                } else {
                    let mfp: *mut ::core::ffi::c_char = (if name_only != 0 {
                        xstrdup(res_name) as *mut ::core::ffi::c_void
                    } else {
                        xmalloc(len.wrapping_add(2 as size_t))
                    })
                        as *mut ::core::ffi::c_char;
                    if name_only == 0 {
                        let mut p: *mut ::core::ffi::c_char = mfp;
                        let c2rust_fresh7 = p;
                        p = p.offset(1);
                        *c2rust_fresh7 = (MT_GL_OTH as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                            as ::core::ffi::c_char;
                        let c2rust_fresh8 = p;
                        p = p.offset(1);
                        *c2rust_fresh8 = 0x2 as ::core::ffi::c_char;
                        strcpy(p, res_name);
                        p = p.offset(strlen(p) as isize);
                        let c2rust_fresh9 = p;
                        p = p.offset(1);
                        *c2rust_fresh9 = '\t' as ::core::ffi::c_char;
                        strcpy(p, res_fname);
                        p = p.offset(strlen(p) as isize);
                        let c2rust_fresh10 = p;
                        p = p.offset(1);
                        *c2rust_fresh10 = '\t' as ::core::ffi::c_char;
                        strcpy(p, res_cmd);
                        p = p.offset(strlen(p) as isize);
                        if has_extra {
                            strcpy(
                                p,
                                b";\"\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                            );
                            p = p.offset(strlen(p) as isize);
                            if !res_kind.is_null() {
                                let c2rust_fresh11 = p;
                                p = p.offset(1);
                                *c2rust_fresh11 = '\t' as ::core::ffi::c_char;
                                strcpy(p, res_kind);
                                p = p.offset(strlen(p) as isize);
                            }
                            let dihi_ht__0: *mut hashtab_T =
                                &raw mut (*(*li).li_tv.vval.v_dict).dv_hashtab;
                            let mut dihi_todo__0: size_t = (*dihi_ht__0).ht_used;
                            let mut dihi__0: *mut hashitem_T = (*dihi_ht__0).ht_array;
                            while dihi_todo__0 != 0 {
                                if !((*dihi__0).hi_key.is_null()
                                    || (*dihi__0).hi_key
                                        == &raw const hash_removed as *mut ::core::ffi::c_char)
                                {
                                    dihi_todo__0 = dihi_todo__0.wrapping_sub(1);
                                    let di_0: *mut dictitem_T = (*dihi__0)
                                        .hi_key
                                        .offset(-(17 as ::core::ffi::c_ulong as isize))
                                        as *mut dictitem_T;
                                    let mut dict_key_0: *const ::core::ffi::c_char =
                                        &raw mut (*di_0).di_key as *mut ::core::ffi::c_char;
                                    let mut tv_0: *mut typval_T = &raw mut (*di_0).di_tv;
                                    if !((*tv_0).v_type as ::core::ffi::c_uint
                                        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                                        || (*tv_0).vval.v_string.is_null())
                                    {
                                        if strcmp(
                                            dict_key_0,
                                            b"name\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) != 0
                                        {
                                            if strcmp(
                                                dict_key_0,
                                                b"filename\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ) != 0
                                            {
                                                if strcmp(
                                                    dict_key_0,
                                                    b"cmd\0".as_ptr() as *const ::core::ffi::c_char,
                                                ) != 0
                                                {
                                                    if strcmp(
                                                        dict_key_0,
                                                        b"kind\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ) != 0
                                                    {
                                                        let c2rust_fresh12 = p;
                                                        p = p.offset(1);
                                                        *c2rust_fresh12 =
                                                            '\t' as ::core::ffi::c_char;
                                                        strcpy(
                                                            p,
                                                            dict_key_0 as *mut ::core::ffi::c_char,
                                                        );
                                                        p = p.offset(strlen(p) as isize);
                                                        strcpy(
                                                            p,
                                                            b":\0".as_ptr()
                                                                as *const ::core::ffi::c_char
                                                                as *mut ::core::ffi::c_char,
                                                        );
                                                        p = p.offset(strlen(p) as isize);
                                                        strcpy(p, (*tv_0).vval.v_string);
                                                        p = p.offset(strlen(p) as isize);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                dihi__0 = dihi__0.offset(1);
                            }
                        }
                    }
                    ga_grow(ga, 1 as ::core::ffi::c_int);
                    let c2rust_fresh13 = (*ga).ga_len;
                    (*ga).ga_len = (*ga).ga_len + 1;
                    let c2rust_lvalue_ptr = &raw mut *((*ga).ga_data
                        as *mut *mut ::core::ffi::c_char)
                        .offset(c2rust_fresh13 as isize);
                    *c2rust_lvalue_ptr = mfp;
                    ntags += 1;
                    result = 1 as ::core::ffi::c_int;
                    li = (*li).li_next;
                }
            }
        }
    }
    tv_clear(&raw mut rettv);
    *match_count = ntags;
    return result;
}
unsafe extern "C" fn findtags_state_init(
    mut st: *mut findtags_state_T,
    mut pat: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut mincount: ::core::ffi::c_int,
) {
    (*st).tag_fname =
        xmalloc((MAXPATHL + 1 as ::core::ffi::c_int) as size_t) as *mut ::core::ffi::c_char;
    (*st).fp = ::core::ptr::null_mut::<FILE>();
    (*st).orgpat = xmalloc(::core::mem::size_of::<pat_T>()) as *mut pat_T;
    (*(*st).orgpat).pat = pat;
    (*(*st).orgpat).len = strlen(pat) as ::core::ffi::c_int;
    (*(*st).orgpat).regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
    (*st).flags = flags;
    (*st).tag_file_sorted = NUL;
    (*st).help_lang_find = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*st).is_txt = false_0 != 0;
    (*st).did_open = false_0 != 0;
    (*st).help_only = flags & TAG_HELP as ::core::ffi::c_int != 0;
    (*st).get_searchpat = false_0 != 0;
    (*st).help_lang[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    (*st).help_pri = 0 as ::core::ffi::c_int;
    (*st).mincount = mincount;
    (*st).lbuf_size = LSIZE as ::core::ffi::c_int;
    (*st).lbuf = xmalloc((*st).lbuf_size as size_t) as *mut ::core::ffi::c_char;
    (*st).match_count = 0 as ::core::ffi::c_int;
    (*st).stop_searching = false_0 != 0;
    let mut mtt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while mtt < MT_COUNT as ::core::ffi::c_int {
        ga_init(
            (&raw mut (*st).ga_match as *mut garray_T).offset(mtt as isize),
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            100 as ::core::ffi::c_int,
        );
        hash_init((&raw mut (*st).ht_match as *mut hashtab_T).offset(mtt as isize));
        mtt += 1;
    }
}
unsafe extern "C" fn findtags_state_free(mut st: *mut findtags_state_T) {
    xfree((*st).tag_fname as *mut ::core::ffi::c_void);
    xfree((*st).lbuf as *mut ::core::ffi::c_void);
    vim_regfree((*(*st).orgpat).regmatch.regprog);
    xfree((*st).orgpat as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn findtags_in_help_init(mut st: *mut findtags_state_T) -> bool {
    let mut i: ::core::ffi::c_int = 0;
    if (*st).is_txt {
        strcpy(
            &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
            b"en\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
    } else {
        i = strlen((*st).tag_fname) as ::core::ffi::c_int;
        if i > 3 as ::core::ffi::c_int
            && *(*st)
                .tag_fname
                .offset((i - 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                == '-' as ::core::ffi::c_int
        {
            xmemcpyz(
                &raw mut (*st).help_lang as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                (*st)
                    .tag_fname
                    .offset(i as isize)
                    .offset(-(2 as ::core::ffi::c_int as isize))
                    as *const ::core::ffi::c_void,
                2 as size_t,
            );
        } else {
            strcpy(
                &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
                b"en\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
        }
    }
    if !(*st).help_lang_find.is_null()
        && strcasecmp(
            &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
            (*st).help_lang_find,
        ) != 0 as ::core::ffi::c_int
    {
        return false_0 != 0;
    }
    if (*st).flags & TAG_KEEP_LANG as ::core::ffi::c_int != 0
        && (*st).help_lang_find.is_null()
        && !(*curbuf.get()).b_fname.is_null()
        && {
            i = strlen((*curbuf.get()).b_fname) as ::core::ffi::c_int;
            i > 4 as ::core::ffi::c_int
        }
        && *(*curbuf.get())
            .b_fname
            .offset((i - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            == 'x' as ::core::ffi::c_int
        && *(*curbuf.get())
            .b_fname
            .offset((i - 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
        && strncasecmp(
            (*curbuf.get())
                .b_fname
                .offset(i as isize)
                .offset(-(3 as ::core::ffi::c_int as isize)),
            &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
            2 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        (*st).help_pri = 0 as ::core::ffi::c_int;
    } else {
        (*st).help_pri = 1 as ::core::ffi::c_int;
        let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        s = p_hlg.get();
        while *s as ::core::ffi::c_int != NUL {
            if strncasecmp(
                s,
                &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                break;
            }
            (*st).help_pri += 1;
            s = vim_strchr(s, ',' as ::core::ffi::c_int);
            if s.is_null() {
                break;
            }
            s = s.offset(1);
        }
        if s.is_null() || *s as ::core::ffi::c_int == NUL {
            (*st).help_pri += 1;
            if strcasecmp(
                &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
                b"en\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) != 0 as ::core::ffi::c_int
            {
                (*st).help_pri += 1;
            }
        }
    }
    return true_0 != 0;
}
unsafe extern "C" fn findtags_apply_tfu(
    mut st: *mut findtags_state_T,
    mut pat: *mut ::core::ffi::c_char,
    mut buf_ffname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let use_tfu: bool =
        (*st).flags & TAG_NO_TAGFUNC as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
    if !use_tfu
        || tfu_in_use.get() as ::core::ffi::c_int != 0
        || *(*curbuf.get()).b_p_tfu as ::core::ffi::c_int == NUL
    {
        return NOTDONE;
    }
    tfu_in_use.set(true_0 != 0);
    let mut retval: ::core::ffi::c_int = find_tagfunc_tags(
        pat,
        &raw mut (*st).ga_match as *mut garray_T,
        &raw mut (*st).match_count,
        (*st).flags,
        buf_ffname,
    );
    tfu_in_use.set(false_0 != 0);
    return retval;
}
unsafe extern "C" fn findtags_get_next_line(
    mut st: *mut findtags_state_T,
    mut sinfo_p: *mut tagsearch_info_T,
) -> tags_read_status_T {
    let mut eof: bool = false;
    if (*st).state as ::core::ffi::c_uint == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut offset: off_T =
            (*sinfo_p).low_offset + ((*sinfo_p).high_offset - (*sinfo_p).low_offset) / 2 as off_T;
        if offset == (*sinfo_p).curr_offset {
            return TAGS_READ_EOF;
        } else {
            (*sinfo_p).curr_offset = offset;
        }
    } else if (*st).state as ::core::ffi::c_uint
        == TS_SKIP_BACK as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*sinfo_p).curr_offset -= ((*st).lbuf_size * 2 as ::core::ffi::c_int) as off_T;
        if (*sinfo_p).curr_offset < 0 as off_T {
            (*sinfo_p).curr_offset = 0 as off_T;
            fseek((*st).fp, 0 as ::core::ffi::c_long, SEEK_SET);
            (*st).state = TS_STEP_FORWARD;
        }
    }
    if (*st).state as ::core::ffi::c_uint == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*st).state as ::core::ffi::c_uint
            == TS_SKIP_BACK as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*sinfo_p).curr_offset_used = (*sinfo_p).curr_offset;
        vim_ignored.set(fseeko(
            (*st).fp,
            (*sinfo_p).curr_offset as __off_t,
            SEEK_SET,
        ));
        eof = vim_fgets((*st).lbuf, (*st).lbuf_size, (*st).fp);
        if !eof && (*sinfo_p).curr_offset != 0 as off_T {
            (*sinfo_p).curr_offset = ftello((*st).fp) as off_T;
            if (*sinfo_p).curr_offset == (*sinfo_p).high_offset {
                vim_ignored.set(fseeko((*st).fp, (*sinfo_p).low_offset as __off_t, SEEK_SET));
                (*sinfo_p).curr_offset = (*sinfo_p).low_offset;
            }
            eof = vim_fgets((*st).lbuf, (*st).lbuf_size, (*st).fp);
        }
        while !eof && vim_isblankline((*st).lbuf) as ::core::ffi::c_int != 0 {
            (*sinfo_p).curr_offset = ftello((*st).fp) as off_T;
            eof = vim_fgets((*st).lbuf, (*st).lbuf_size, (*st).fp);
        }
        if eof {
            (*st).state = TS_SKIP_BACK;
            (*sinfo_p).match_offset = ftello((*st).fp) as off_T;
            (*sinfo_p).curr_offset = (*sinfo_p).curr_offset_used;
            return TAGS_READ_IGNORE;
        }
    } else {
        loop {
            eof = vim_fgets((*st).lbuf, (*st).lbuf_size, (*st).fp);
            if !(!eof && vim_isblankline((*st).lbuf) as ::core::ffi::c_int != 0) {
                break;
            }
        }
        if eof {
            return TAGS_READ_EOF;
        }
    }
    return TAGS_READ_SUCCESS;
}
unsafe extern "C" fn findtags_hdr_parse(mut st: *mut findtags_state_T) -> bool {
    if strncmp(
        (*st).lbuf,
        b"!_TAG_\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) != 0 as ::core::ffi::c_int
    {
        return true_0 != 0;
    }
    if strncmp(
        (*st).lbuf,
        b"!_TAG_FILE_SORTED\t\0".as_ptr() as *const ::core::ffi::c_char,
        18 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        (*st).tag_file_sorted =
            *(*st).lbuf.offset(18 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int;
    }
    if strncmp(
        (*st).lbuf,
        b"!_TAG_FILE_ENCODING\t\0".as_ptr() as *const ::core::ffi::c_char,
        20 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        p = (*st).lbuf.offset(20 as ::core::ffi::c_int as isize);
        while *p as ::core::ffi::c_int > ' ' as ::core::ffi::c_int
            && (*p as ::core::ffi::c_int) < 127 as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        *p = NUL as ::core::ffi::c_char;
        convert_setup(
            &raw mut (*st).vimconv,
            (*st).lbuf.offset(20 as ::core::ffi::c_int as isize),
            p_enc.get(),
        );
    }
    return false_0 != 0;
}
unsafe extern "C" fn findtags_start_state_handler(
    mut st: *mut findtags_state_T,
    mut sortic: *mut bool,
    mut sinfo_p: *mut tagsearch_info_T,
) -> bool {
    let noic: bool = (*st).flags & TAG_NOIC as ::core::ffi::c_int != 0;
    if strncmp(
        (*st).lbuf,
        b"!_TAG_\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) <= 0 as ::core::ffi::c_int
        || *(*st).lbuf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '!' as ::core::ffi::c_int
            && (*(*st).lbuf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                >= 'a' as ::core::ffi::c_uint
                && *(*st).lbuf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                    <= 'z' as ::core::ffi::c_uint)
    {
        return findtags_hdr_parse(st);
    }
    if (*st).linear {
        (*st).state = TS_LINEAR;
    } else if (*st).tag_file_sorted == NUL {
        (*st).state = TS_BINARY;
    } else if (*st).tag_file_sorted == '1' as ::core::ffi::c_int {
        (*st).state = TS_BINARY;
    } else if (*st).tag_file_sorted == '2' as ::core::ffi::c_int {
        (*st).state = TS_BINARY;
        *sortic = true_0 != 0;
        (*(*st).orgpat).regmatch.rm_ic = p_ic.get() != 0 || !noic;
    } else {
        (*st).state = TS_LINEAR;
    }
    if (*st).state as ::core::ffi::c_uint == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*(*st).orgpat).regmatch.rm_ic as ::core::ffi::c_int != 0
        && !*sortic
    {
        (*st).linear = true_0 != 0;
        (*st).state = TS_LINEAR;
    }
    if (*st).state as ::core::ffi::c_uint == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if fseeko((*st).fp, 0 as __off_t, SEEK_END) != 0 as ::core::ffi::c_int {
            (*st).state = TS_LINEAR;
        } else {
            let filesize: off_T = ftello((*st).fp);
            vim_ignored.set(fseeko((*st).fp, 0 as __off_t, SEEK_SET));
            (*sinfo_p).low_offset = 0 as off_T;
            (*sinfo_p).low_char = 0 as ::core::ffi::c_int;
            (*sinfo_p).high_offset = filesize;
            (*sinfo_p).curr_offset = 0 as off_T;
            (*sinfo_p).high_char = 0xff as ::core::ffi::c_int;
        }
        return false_0 != 0;
    }
    return true_0 != 0;
}
unsafe extern "C" fn findtags_parse_line(
    mut st: *mut findtags_state_T,
    mut tagpp: *mut tagptrs_T,
    mut margs: *mut findtags_match_args_T,
    mut sinfo_p: *mut tagsearch_info_T,
) -> tagmatch_status_T {
    let mut status: ::core::ffi::c_int = 0;
    if (*(*st).orgpat).headlen != 0 {
        memset(
            tagpp as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<tagptrs_T>(),
        );
        (*tagpp).tagname = (*st).lbuf;
        (*tagpp).tagname_end = vim_strchr((*st).lbuf, TAB);
        if (*tagpp).tagname_end.is_null() {
            return TAG_MATCH_FAIL;
        }
        let mut cmplen: ::core::ffi::c_int =
            (*tagpp).tagname_end.offset_from((*tagpp).tagname) as ::core::ffi::c_int;
        if p_tl.get() != 0 as OptInt && cmplen as OptInt > p_tl.get() {
            cmplen = p_tl.get() as ::core::ffi::c_int;
        }
        if (*st).flags & TAG_REGEXP as ::core::ffi::c_int != 0 && (*(*st).orgpat).headlen < cmplen {
            cmplen = (*(*st).orgpat).headlen;
        } else if (*st).state as ::core::ffi::c_uint
            == TS_LINEAR as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*(*st).orgpat).headlen != cmplen
        {
            return TAG_MATCH_NEXT;
        }
        if (*st).state as ::core::ffi::c_uint
            == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut tagcmp: ::core::ffi::c_int = 0;
            let mut i: ::core::ffi::c_int =
                *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                    as ::core::ffi::c_int;
            if (*margs).sortic {
                i = if (*(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int)
                    < 'a' as ::core::ffi::c_int
                    || *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        > 'z' as ::core::ffi::c_int
                {
                    *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                } else {
                    *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                };
            }
            if i < (*sinfo_p).low_char || i > (*sinfo_p).high_char {
                (*margs).sort_error = true_0 != 0;
            }
            if (*margs).sortic {
                tagcmp = tag_strnicmp((*tagpp).tagname, (*(*st).orgpat).head, cmplen as size_t);
            } else {
                tagcmp = strncmp((*tagpp).tagname, (*(*st).orgpat).head, cmplen as size_t);
            }
            if tagcmp == 0 as ::core::ffi::c_int {
                if cmplen < (*(*st).orgpat).headlen {
                    tagcmp = -1 as ::core::ffi::c_int;
                } else if cmplen > (*(*st).orgpat).headlen {
                    tagcmp = 1 as ::core::ffi::c_int;
                }
            }
            if tagcmp == 0 as ::core::ffi::c_int {
                (*st).state = TS_SKIP_BACK;
                (*sinfo_p).match_offset = (*sinfo_p).curr_offset;
                return TAG_MATCH_NEXT;
            }
            if tagcmp < 0 as ::core::ffi::c_int {
                (*sinfo_p).curr_offset = ftello((*st).fp) as off_T;
                if (*sinfo_p).curr_offset < (*sinfo_p).high_offset {
                    (*sinfo_p).low_offset = (*sinfo_p).curr_offset;
                    if (*margs).sortic {
                        (*sinfo_p).low_char =
                            if (*(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int)
                                < 'a' as ::core::ffi::c_int
                                || *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    > 'z' as ::core::ffi::c_int
                            {
                                *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                            } else {
                                *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                            };
                    } else {
                        (*sinfo_p).low_char =
                            *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int;
                    }
                    return TAG_MATCH_NEXT;
                }
            }
            if tagcmp > 0 as ::core::ffi::c_int && (*sinfo_p).curr_offset != (*sinfo_p).high_offset
            {
                (*sinfo_p).high_offset = (*sinfo_p).curr_offset;
                if (*margs).sortic {
                    (*sinfo_p).high_char =
                        if (*(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int)
                            < 'a' as ::core::ffi::c_int
                            || *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                > 'z' as ::core::ffi::c_int
                        {
                            *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                        } else {
                            *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                        };
                } else {
                    (*sinfo_p).high_char =
                        *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int;
                }
                return TAG_MATCH_NEXT;
            }
            return TAG_MATCH_STOP;
        } else if (*st).state as ::core::ffi::c_uint
            == TS_SKIP_BACK as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            '_c2rust_label: {
                if cmplen >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"cmplen >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/tag.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        1797 as ::core::ffi::c_uint,
                        b"tagmatch_status_T findtags_parse_line(findtags_state_T *, tagptrs_T *, findtags_match_args_T *, tagsearch_info_T *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            if mb_strnicmp((*tagpp).tagname, (*(*st).orgpat).head, cmplen as size_t)
                != 0 as ::core::ffi::c_int
            {
                (*st).state = TS_STEP_FORWARD;
            } else {
                (*sinfo_p).curr_offset = (*sinfo_p).curr_offset_used;
            }
            return TAG_MATCH_NEXT;
        } else if (*st).state as ::core::ffi::c_uint
            == TS_STEP_FORWARD as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            '_c2rust_label_0: {
                if cmplen >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"cmplen >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/tag.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        1807 as ::core::ffi::c_uint,
                        b"tagmatch_status_T findtags_parse_line(findtags_state_T *, tagptrs_T *, findtags_match_args_T *, tagsearch_info_T *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            if mb_strnicmp((*tagpp).tagname, (*(*st).orgpat).head, cmplen as size_t)
                != 0 as ::core::ffi::c_int
            {
                return (if ftello((*st).fp) > (*sinfo_p).match_offset {
                    TAG_MATCH_STOP as ::core::ffi::c_int
                } else {
                    TAG_MATCH_NEXT as ::core::ffi::c_int
                }) as tagmatch_status_T;
            }
        } else {
            '_c2rust_label_1: {
                if cmplen >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"cmplen >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/tag.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        1815 as ::core::ffi::c_uint,
                        b"tagmatch_status_T findtags_parse_line(findtags_state_T *, tagptrs_T *, findtags_match_args_T *, tagsearch_info_T *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            if mb_strnicmp((*tagpp).tagname, (*(*st).orgpat).head, cmplen as size_t)
                != 0 as ::core::ffi::c_int
            {
                return TAG_MATCH_NEXT;
            }
        }
        (*tagpp).fname = (*tagpp)
            .tagname_end
            .offset(1 as ::core::ffi::c_int as isize);
        (*tagpp).fname_end = vim_strchr((*tagpp).fname, TAB);
        if (*tagpp).fname_end.is_null() {
            status = FAIL;
        } else {
            (*tagpp).command = (*tagpp).fname_end.offset(1 as ::core::ffi::c_int as isize);
            status = OK;
        }
    } else {
        status = parse_tag_line((*st).lbuf, tagpp);
    }
    return (if status == FAIL {
        TAG_MATCH_FAIL as ::core::ffi::c_int
    } else {
        TAG_MATCH_SUCCESS as ::core::ffi::c_int
    }) as tagmatch_status_T;
}
unsafe extern "C" fn findtags_matchargs_init(
    mut margs: *mut findtags_match_args_T,
    mut flags: ::core::ffi::c_int,
) {
    (*margs).matchoff = 0 as ::core::ffi::c_int;
    (*margs).match_re = false_0 != 0;
    (*margs).match_no_ic = false_0 != 0;
    (*margs).has_re = flags & TAG_REGEXP as ::core::ffi::c_int != 0;
    (*margs).sortic = false_0 != 0;
    (*margs).sort_error = false_0 != 0;
}
unsafe extern "C" fn findtags_match_tag(
    mut st: *mut findtags_state_T,
    mut tagpp: *mut tagptrs_T,
    mut margs: *mut findtags_match_args_T,
) -> bool {
    let mut match_0: bool = false_0 != 0;
    let mut cmplen: ::core::ffi::c_int =
        (*tagpp).tagname_end.offset_from((*tagpp).tagname) as ::core::ffi::c_int;
    if p_tl.get() != 0 as OptInt && cmplen as OptInt > p_tl.get() {
        cmplen = p_tl.get() as ::core::ffi::c_int;
    }
    if (*(*st).orgpat).len != cmplen {
        match_0 = false_0 != 0;
    } else if (*(*st).orgpat).regmatch.rm_ic {
        '_c2rust_label: {
            if cmplen >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"cmplen >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/tag.rs\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    1869 as ::core::ffi::c_uint,
                    b"_Bool findtags_match_tag(findtags_state_T *, tagptrs_T *, findtags_match_args_T *)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        match_0 = mb_strnicmp((*tagpp).tagname, (*(*st).orgpat).pat, cmplen as size_t)
            == 0 as ::core::ffi::c_int;
        if match_0 {
            (*margs).match_no_ic = strncmp((*tagpp).tagname, (*(*st).orgpat).pat, cmplen as size_t)
                == 0 as ::core::ffi::c_int;
        }
    } else {
        match_0 = strncmp((*tagpp).tagname, (*(*st).orgpat).pat, cmplen as size_t)
            == 0 as ::core::ffi::c_int;
    }
    (*margs).match_re = false_0 != 0;
    if !match_0 && !(*(*st).orgpat).regmatch.regprog.is_null() {
        let mut cc: ::core::ffi::c_char = *(*tagpp).tagname_end;
        *(*tagpp).tagname_end = NUL as ::core::ffi::c_char;
        match_0 = vim_regexec(
            &raw mut (*(*st).orgpat).regmatch,
            (*tagpp).tagname,
            0 as colnr_T,
        );
        if match_0 {
            (*margs).matchoff = (*(*st).orgpat).regmatch.startp[0 as ::core::ffi::c_int as usize]
                .offset_from((*tagpp).tagname)
                as ::core::ffi::c_int;
            if (*(*st).orgpat).regmatch.rm_ic {
                (*(*st).orgpat).regmatch.rm_ic = false_0 != 0;
                (*margs).match_no_ic = vim_regexec(
                    &raw mut (*(*st).orgpat).regmatch,
                    (*tagpp).tagname,
                    0 as colnr_T,
                );
                (*(*st).orgpat).regmatch.rm_ic = true_0 != 0;
            }
        }
        *(*tagpp).tagname_end = cc;
        (*margs).match_re = true_0 != 0;
    }
    return match_0;
}
unsafe extern "C" fn findtags_string_convert(mut st: *mut findtags_state_T) {
    let mut conv_line: *mut ::core::ffi::c_char = string_convert(
        &raw mut (*st).vimconv,
        (*st).lbuf,
        ::core::ptr::null_mut::<size_t>(),
    );
    if conv_line.is_null() {
        return;
    }
    let mut len: ::core::ffi::c_int =
        strlen(conv_line) as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    if len > (*st).lbuf_size {
        xfree((*st).lbuf as *mut ::core::ffi::c_void);
        (*st).lbuf = conv_line;
        (*st).lbuf_size = len;
    } else {
        strcpy((*st).lbuf, conv_line);
        xfree(conv_line as *mut ::core::ffi::c_void);
    };
}
unsafe extern "C" fn findtags_add_match(
    mut st: *mut findtags_state_T,
    mut tagpp: *mut tagptrs_T,
    mut margs: *mut findtags_match_args_T,
    mut buf_ffname: *mut ::core::ffi::c_char,
    mut hash: *mut hash_T,
) {
    let name_only: bool = (*st).flags & TAG_NAMES as ::core::ffi::c_int != 0;
    let mut len: size_t = 0 as size_t;
    let mut mfp_size: size_t = 0 as size_t;
    let mut mfp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut is_current: bool = test_for_current(
        (*tagpp).fname,
        (*tagpp).fname_end,
        (*st).tag_fname,
        buf_ffname,
    ) != 0;
    let mut is_static: bool = test_for_static(tagpp);
    let mut mtt: ::core::ffi::c_int = if is_static as ::core::ffi::c_int != 0 {
        if is_current as ::core::ffi::c_int != 0 {
            MT_ST_CUR as ::core::ffi::c_int
        } else {
            MT_ST_OTH as ::core::ffi::c_int
        }
    } else if is_current as ::core::ffi::c_int != 0 {
        MT_GL_CUR as ::core::ffi::c_int
    } else {
        MT_GL_OTH as ::core::ffi::c_int
    };
    if (*(*st).orgpat).regmatch.rm_ic as ::core::ffi::c_int != 0 && !(*margs).match_no_ic {
        mtt += MT_IC_OFF as ::core::ffi::c_int;
    }
    if (*margs).match_re {
        mtt += MT_RE_OFF as ::core::ffi::c_int;
    }
    if (*st).help_only {
        *(*tagpp).tagname_end = NUL as ::core::ffi::c_char;
        len = (*tagpp).tagname_end.offset_from((*tagpp).tagname) as size_t;
        mfp_size = ::core::mem::size_of::<::core::ffi::c_char>()
            .wrapping_add(len as usize)
            .wrapping_add(10 as usize)
            .wrapping_add(ML_EXTRA as usize)
            .wrapping_add(1 as usize) as size_t;
        mfp = xmalloc(mfp_size) as *mut ::core::ffi::c_char;
        let mut p: *mut ::core::ffi::c_char = mfp;
        strcpy(p, (*tagpp).tagname);
        *p.offset(len as isize) = '@' as ::core::ffi::c_char;
        strcpy(
            p.offset(len as isize)
                .offset(1 as ::core::ffi::c_int as isize),
            &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
        );
        snprintf(
            p.offset(len as isize)
                .offset(1 as ::core::ffi::c_int as isize)
                .offset(ML_EXTRA as isize),
            mfp_size.wrapping_sub(
                len.wrapping_add(1 as size_t)
                    .wrapping_add(ML_EXTRA as size_t),
            ),
            b"%06d\0".as_ptr() as *const ::core::ffi::c_char,
            help_heuristic(
                (*tagpp).tagname,
                if (*margs).match_re as ::core::ffi::c_int != 0 {
                    (*margs).matchoff
                } else {
                    0 as ::core::ffi::c_int
                },
                !(*margs).match_no_ic,
            ) + (*st).help_pri,
        );
        *(*tagpp).tagname_end = TAB as ::core::ffi::c_char;
    } else if name_only {
        if (*st).get_searchpat {
            let mut temp_end: *mut ::core::ffi::c_char = (*tagpp).command;
            if *temp_end as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
                while *temp_end as ::core::ffi::c_int != 0
                    && *temp_end as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                    && *temp_end as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                    && *temp_end as ::core::ffi::c_int != '$' as ::core::ffi::c_int
                {
                    temp_end = temp_end.offset(1);
                }
            }
            if (*tagpp).command.offset(2 as ::core::ffi::c_int as isize) < temp_end {
                len = (temp_end.offset_from((*tagpp).command) - 2 as isize) as size_t;
                mfp = xmalloc(len.wrapping_add(2 as size_t)) as *mut ::core::ffi::c_char;
                xmemcpyz(
                    mfp as *mut ::core::ffi::c_void,
                    (*tagpp).command.offset(2 as ::core::ffi::c_int as isize)
                        as *const ::core::ffi::c_void,
                    len,
                );
            } else {
                mfp = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            (*st).get_searchpat = false_0 != 0;
        } else {
            len = (*tagpp).tagname_end.offset_from((*tagpp).tagname) as size_t;
            mfp = xmalloc(
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_add(len)
                    .wrapping_add(1 as size_t),
            ) as *mut ::core::ffi::c_char;
            xmemcpyz(
                mfp as *mut ::core::ffi::c_void,
                (*tagpp).tagname as *const ::core::ffi::c_void,
                len,
            );
            if State.get() & MODE_INSERT != 0 {
                (*st).get_searchpat = p_sft.get() != 0;
            }
        }
    } else {
        let mut tag_fname_len: size_t = strlen((*st).tag_fname);
        len = tag_fname_len
            .wrapping_add(strlen((*st).lbuf))
            .wrapping_add(3 as size_t);
        mfp = xmalloc(
            ::core::mem::size_of::<::core::ffi::c_char>()
                .wrapping_add(len)
                .wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
        let mut p_0: *mut ::core::ffi::c_char = mfp;
        *p_0.offset(0 as ::core::ffi::c_int as isize) =
            (mtt + 1 as ::core::ffi::c_int) as ::core::ffi::c_char;
        strcpy(
            p_0.offset(1 as ::core::ffi::c_int as isize),
            (*st).tag_fname,
        );
        *p_0.offset(tag_fname_len.wrapping_add(1 as size_t) as isize) =
            TAG_SEP as ::core::ffi::c_char;
        let mut s: *mut ::core::ffi::c_char = p_0
            .offset(1 as ::core::ffi::c_int as isize)
            .offset(tag_fname_len as isize)
            .offset(1 as ::core::ffi::c_int as isize);
        strcpy(s, (*st).lbuf);
    }
    if !mfp.is_null() {
        let mut hi: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
        *hash = hash_hash(mfp);
        hi = hash_lookup(
            (&raw mut (*st).ht_match as *mut hashtab_T).offset(mtt as isize),
            mfp,
            strlen(mfp),
            *hash,
        );
        if (*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
        {
            hash_add_item(
                (&raw mut (*st).ht_match as *mut hashtab_T).offset(mtt as isize),
                hi,
                mfp,
                *hash,
            );
            ga_grow(
                (&raw mut (*st).ga_match as *mut garray_T).offset(mtt as isize),
                1 as ::core::ffi::c_int,
            );
            *((*st).ga_match[mtt as usize].ga_data as *mut *mut ::core::ffi::c_char)
                .offset((*st).ga_match[mtt as usize].ga_len as isize) = mfp;
            (*st).ga_match[mtt as usize].ga_len += 1;
            (*st).match_count += 1;
        } else {
            xfree(mfp as *mut ::core::ffi::c_void);
        }
    }
}
pub const ML_EXTRA: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
unsafe extern "C" fn findtags_get_all_tags(
    mut st: *mut findtags_state_T,
    mut margs: *mut findtags_match_args_T,
    mut buf_ffname: *mut ::core::ffi::c_char,
) {
    let mut tagp: tagptrs_T = tagptrs_T {
        tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagline: 0,
    };
    let mut search_info: tagsearch_info_T = tagsearch_info_T {
        low_offset: 0,
        high_offset: 0,
        curr_offset: 0,
        curr_offset_used: 0,
        match_offset: 0,
        low_char: 0,
        high_char: 0,
    };
    let mut hash: hash_T = 0 as hash_T;
    memset(
        &raw mut search_info as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<tagsearch_info_T>(),
    );
    let mut retval: ::core::ffi::c_int = 0;
    loop {
        if (*st).state as ::core::ffi::c_uint
            == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*st).state as ::core::ffi::c_uint
                == TS_SKIP_BACK as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            line_breakcheck();
        } else {
            fast_breakcheck();
        }
        if (*st).flags & TAG_INS_COMP as ::core::ffi::c_int != 0 {
            ins_compl_check_keys(30 as ::core::ffi::c_int, false_0 != 0);
        }
        if got_int.get() as ::core::ffi::c_int != 0
            || ins_compl_interrupted() as ::core::ffi::c_int != 0
        {
            (*st).stop_searching = true_0 != 0;
            break;
        } else if (*st).mincount == TAG_MANY as ::core::ffi::c_int
            && (*st).match_count >= TAG_MANY as ::core::ffi::c_int
        {
            (*st).stop_searching = true_0 != 0;
            break;
        } else {
            if !(*st).get_searchpat {
                retval = findtags_get_next_line(st, &raw mut search_info) as ::core::ffi::c_int;
                if retval == TAGS_READ_IGNORE as ::core::ffi::c_int {
                    continue;
                }
                if retval == TAGS_READ_EOF as ::core::ffi::c_int {
                    break;
                }
            }
            if (*st).vimconv.vc_type != CONV_NONE as ::core::ffi::c_int {
                findtags_string_convert(st);
            }
            if (*st).state as ::core::ffi::c_uint
                == TS_START as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if !findtags_start_state_handler(st, &raw mut (*margs).sortic, &raw mut search_info)
                {
                    continue;
                }
            }
            if *(*st)
                .lbuf
                .offset(((*st).lbuf_size - 2 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                != NUL
            {
                (*st).lbuf_size *= 2 as ::core::ffi::c_int;
                xfree((*st).lbuf as *mut ::core::ffi::c_void);
                (*st).lbuf = xmalloc((*st).lbuf_size as size_t) as *mut ::core::ffi::c_char;
                if (*st).state as ::core::ffi::c_uint
                    == TS_STEP_FORWARD as ::core::ffi::c_int as ::core::ffi::c_uint
                    || (*st).state as ::core::ffi::c_uint
                        == TS_LINEAR as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    vim_ignored.set(fseeko(
                        (*st).fp,
                        search_info.curr_offset as __off_t,
                        SEEK_SET,
                    ));
                }
                search_info.curr_offset = 0 as off_T;
            } else {
                retval = findtags_parse_line(st, &raw mut tagp, margs, &raw mut search_info)
                    as ::core::ffi::c_int;
                if retval == TAG_MATCH_NEXT as ::core::ffi::c_int {
                    continue;
                }
                if retval == TAG_MATCH_STOP as ::core::ffi::c_int {
                    break;
                }
                if retval == TAG_MATCH_FAIL as ::core::ffi::c_int {
                    semsg(
                        gettext(b"E431: Format error in tags file \"%s\"\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        (*st).tag_fname,
                    );
                    semsg(
                        gettext(b"Before byte %ld\0".as_ptr() as *const ::core::ffi::c_char),
                        ftello((*st).fp) as int64_t,
                    );
                    (*st).stop_searching = true_0 != 0;
                    return;
                }
                if findtags_match_tag(st, &raw mut tagp, margs) {
                    findtags_add_match(st, &raw mut tagp, margs, buf_ffname, &raw mut hash);
                }
            }
        }
    }
}
unsafe extern "C" fn findtags_in_file(
    mut st: *mut findtags_state_T,
    mut _flags: ::core::ffi::c_int,
    mut buf_ffname: *mut ::core::ffi::c_char,
) {
    let mut margs: findtags_match_args_T = findtags_match_args_T {
        matchoff: 0,
        match_re: false,
        match_no_ic: false,
        has_re: false,
        sortic: false,
        sort_error: false,
    };
    (*st).vimconv.vc_type = CONV_NONE as ::core::ffi::c_int;
    (*st).tag_file_sorted = NUL;
    (*st).fp = ::core::ptr::null_mut::<FILE>();
    findtags_matchargs_init(&raw mut margs, (*st).flags);
    if (*curbuf.get()).b_help {
        if !findtags_in_help_init(st) {
            return;
        }
    }
    (*st).fp = os_fopen(
        (*st).tag_fname,
        b"r\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if (*st).fp.is_null() {
        return;
    }
    if p_verbose.get() >= 5 as OptInt {
        verbose_enter();
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"Searching tags file %s\0".as_ptr() as *const ::core::ffi::c_char),
            (*st).tag_fname,
        );
        verbose_leave();
    }
    (*st).did_open = true_0 != 0;
    (*st).state = TS_START;
    findtags_get_all_tags(st, &raw mut margs, buf_ffname);
    if !(*st).fp.is_null() {
        fclose((*st).fp);
        (*st).fp = ::core::ptr::null_mut::<FILE>();
    }
    if (*st).vimconv.vc_type != CONV_NONE as ::core::ffi::c_int {
        convert_setup(
            &raw mut (*st).vimconv,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        );
    }
    if margs.sort_error {
        semsg(
            gettext(b"E432: Tags file not sorted: %s\0".as_ptr() as *const ::core::ffi::c_char),
            (*st).tag_fname,
        );
    }
    if (*st).match_count >= (*st).mincount {
        (*st).stop_searching = true_0 != 0;
    }
}
unsafe extern "C" fn findtags_copy_matches(
    mut st: *mut findtags_state_T,
    mut matchesp: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let name_only: bool = (*st).flags & TAG_NAMES as ::core::ffi::c_int != 0;
    let mut matches: *mut *mut ::core::ffi::c_char = (if (*st).match_count > 0 as ::core::ffi::c_int
    {
        xmalloc(
            ((*st).match_count as size_t)
                .wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
        )
    } else {
        NULL_0
    }) as *mut *mut ::core::ffi::c_char;
    (*st).match_count = 0 as ::core::ffi::c_int;
    let mut mtt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while mtt < MT_COUNT as ::core::ffi::c_int {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*st).ga_match[mtt as usize].ga_len {
            let mut mfp: *mut ::core::ffi::c_char = *((*st).ga_match[mtt as usize].ga_data
                as *mut *mut ::core::ffi::c_char)
                .offset(i as isize);
            if matches.is_null() {
                xfree(mfp as *mut ::core::ffi::c_void);
            } else {
                if !name_only {
                    *mfp = (*mfp as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        as ::core::ffi::c_char;
                    let mut p: *mut ::core::ffi::c_char =
                        mfp.offset(1 as ::core::ffi::c_int as isize);
                    while *p as ::core::ffi::c_int != NUL {
                        if *p as ::core::ffi::c_int == TAG_SEP {
                            *p = NUL as ::core::ffi::c_char;
                        }
                        p = p.offset(1);
                    }
                }
                let c2rust_fresh4 = (*st).match_count;
                (*st).match_count = (*st).match_count + 1;
                let c2rust_lvalue_ptr = &raw mut *matches.offset(c2rust_fresh4 as isize);
                *c2rust_lvalue_ptr = mfp;
            }
            i += 1;
        }
        ga_clear((&raw mut (*st).ga_match as *mut garray_T).offset(mtt as isize));
        hash_clear((&raw mut (*st).ht_match as *mut hashtab_T).offset(mtt as isize));
        mtt += 1;
    }
    *matchesp = matches;
    return (*st).match_count;
}
pub unsafe extern "C" fn find_tags(
    mut pat: *mut ::core::ffi::c_char,
    mut num_matches: *mut ::core::ffi::c_int,
    mut matchesp: *mut *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut mincount: ::core::ffi::c_int,
    mut buf_ffname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut st: findtags_state_T = findtags_state_T {
        state: TS_START,
        stop_searching: false,
        orgpat: ::core::ptr::null_mut::<pat_T>(),
        lbuf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        lbuf_size: 0,
        tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fp: ::core::ptr::null_mut::<FILE>(),
        flags: 0,
        tag_file_sorted: 0,
        get_searchpat: false,
        help_only: false,
        did_open: false,
        mincount: 0,
        linear: false,
        vimconv: vimconv_T {
            vc_type: 0,
            vc_factor: 0,
            vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            vc_fail: false,
        },
        help_lang: [0; 3],
        help_pri: 0,
        help_lang_find: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        is_txt: false,
        match_count: 0,
        ga_match: [garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        }; 16],
        ht_match: [hashtab_T {
            ht_mask: 0,
            ht_used: 0,
            ht_filled: 0,
            ht_changed: 0,
            ht_locked: 0,
            ht_array: ::core::ptr::null_mut::<hashitem_T>(),
            ht_smallarray: [hashitem_T {
                hi_hash: 0,
                hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            }; 16],
        }; 16],
    };
    let mut tn: tagname_T = tagname_T {
        tn_tags: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tn_np: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tn_did_filefind_init: 0,
        tn_hf_idx: 0,
        tn_search_ctx: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut first_file: ::core::ffi::c_int = 0;
    let mut retval: ::core::ffi::c_int = FAIL;
    let mut i: ::core::ffi::c_int = 0;
    let mut saved_pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut findall: ::core::ffi::c_int = (mincount == MAXCOL as ::core::ffi::c_int
        || mincount == TAG_MANY as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    let mut has_re: bool = flags & TAG_REGEXP as ::core::ffi::c_int != 0;
    let mut noic: ::core::ffi::c_int = flags & TAG_NOIC as ::core::ffi::c_int;
    let mut verbose: ::core::ffi::c_int = flags & TAG_VERBOSE as ::core::ffi::c_int;
    let mut save_p_ic: ::core::ffi::c_int = p_ic.get();
    match if (*curbuf.get()).b_tc_flags != 0 {
        (*curbuf.get()).b_tc_flags
    } else {
        tc_flags.get()
    } {
        1 => {}
        2 => {
            p_ic.set(true_0);
        }
        4 => {
            p_ic.set(false_0);
        }
        8 => {
            p_ic.set(ignorecase(pat));
        }
        16 => {
            p_ic.set(ignorecase_opt(pat, true_0, true_0));
        }
        _ => {
            abort();
        }
    }
    let mut help_save: ::core::ffi::c_int = (*curbuf.get()).b_help as ::core::ffi::c_int;
    findtags_state_init(&raw mut st, pat, flags, mincount);
    if st.help_only {
        (*curbuf.get()).b_help = true_0 != 0;
    }
    if (*curbuf.get()).b_help {
        if (*st.orgpat).len > 3 as ::core::ffi::c_int
            && *pat.offset(((*st.orgpat).len - 3 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                == '@' as ::core::ffi::c_int
            && (*pat.offset(((*st.orgpat).len - 2 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_uint
                >= 'A' as ::core::ffi::c_uint
                && *pat.offset(((*st.orgpat).len - 2 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_uint
                    <= 'Z' as ::core::ffi::c_uint
                || *pat.offset(((*st.orgpat).len - 2 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_uint
                    >= 'a' as ::core::ffi::c_uint
                    && *pat.offset(((*st.orgpat).len - 2 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        <= 'z' as ::core::ffi::c_uint)
            && (*pat.offset(((*st.orgpat).len - 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_uint
                >= 'A' as ::core::ffi::c_uint
                && *pat.offset(((*st.orgpat).len - 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_uint
                    <= 'Z' as ::core::ffi::c_uint
                || *pat.offset(((*st.orgpat).len - 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_uint
                    >= 'a' as ::core::ffi::c_uint
                    && *pat.offset(((*st.orgpat).len - 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        <= 'z' as ::core::ffi::c_uint)
        {
            saved_pat = xstrnsave(pat, ((*st.orgpat).len as size_t).wrapping_sub(3 as size_t));
            st.help_lang_find = pat.offset(((*st.orgpat).len - 2 as ::core::ffi::c_int) as isize);
            (*st.orgpat).pat = saved_pat;
            (*st.orgpat).len -= 3 as ::core::ffi::c_int;
        }
    }
    if p_tl.get() != 0 as OptInt && (*st.orgpat).len as OptInt > p_tl.get() {
        (*st.orgpat).len = p_tl.get() as ::core::ffi::c_int;
    }
    let mut save_emsg_off: ::core::ffi::c_int = emsg_off.get();
    emsg_off.set(true_0);
    prepare_pats(st.orgpat, has_re);
    emsg_off.set(save_emsg_off);
    if !(has_re as ::core::ffi::c_int != 0 && (*st.orgpat).regmatch.regprog.is_null()) {
        retval = findtags_apply_tfu(&raw mut st, pat, buf_ffname);
        if retval == NOTDONE {
            retval = FAIL;
            if flags & TAG_KEEP_LANG as ::core::ffi::c_int != 0
                && st.help_lang_find.is_null()
                && !(*curbuf.get()).b_fname.is_null()
                && {
                    i = strlen((*curbuf.get()).b_fname) as ::core::ffi::c_int;
                    i > 4 as ::core::ffi::c_int
                }
                && strcasecmp(
                    (*curbuf.get())
                        .b_fname
                        .offset(i as isize)
                        .offset(-(4 as ::core::ffi::c_int as isize)),
                    b".txt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                st.is_txt = true_0 != 0;
            }
            (*st.orgpat).regmatch.rm_ic = (p_ic.get() != 0 || noic == 0)
                && (findall != 0
                    || (*st.orgpat).headlen == 0 as ::core::ffi::c_int
                    || p_tbs.get() == 0);
            let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while round <= 2 as ::core::ffi::c_int {
                st.linear = (*st.orgpat).headlen == 0 as ::core::ffi::c_int
                    || p_tbs.get() == 0
                    || round == 2 as ::core::ffi::c_int;
                first_file = true_0;
                while get_tagfname(&raw mut tn, first_file, st.tag_fname) == OK {
                    findtags_in_file(&raw mut st, flags, buf_ffname);
                    if st.stop_searching {
                        retval = OK;
                        break;
                    } else {
                        first_file = false_0;
                    }
                }
                tagname_free(&raw mut tn);
                if st.stop_searching as ::core::ffi::c_int != 0
                    || st.linear as ::core::ffi::c_int != 0
                    || p_ic.get() == 0 && noic != 0
                    || (*st.orgpat).regmatch.rm_ic as ::core::ffi::c_int != 0
                {
                    break;
                }
                (*st.orgpat).regmatch.rm_ic = true_0 != 0;
                round += 1;
            }
            if !st.stop_searching {
                if !st.did_open && verbose != 0 {
                    emsg(gettext(
                        b"E433: No tags file\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                }
                retval = OK;
            }
        }
    }
    findtags_state_free(&raw mut st);
    if retval == FAIL {
        st.match_count = 0 as ::core::ffi::c_int;
    }
    *num_matches = findtags_copy_matches(&raw mut st, matchesp);
    (*curbuf.get()).b_help = help_save != 0;
    xfree(saved_pat as *mut ::core::ffi::c_void);
    p_ic.set(save_p_ic);
    return retval;
}
static tag_fnames: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);
unsafe extern "C" fn found_tagfile_cb(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut _cookie: *mut ::core::ffi::c_void,
) -> bool {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_fnames {
        let tag_fname: *mut ::core::ffi::c_char = xstrdup(*fnames.offset(i as isize));
        simplify_filename(tag_fname);
        ga_grow(tag_fnames.ptr(), 1 as ::core::ffi::c_int);
        *((*tag_fnames.ptr()).ga_data as *mut *mut ::core::ffi::c_char)
            .offset((*tag_fnames.ptr()).ga_len as isize) = tag_fname;
        (*tag_fnames.ptr()).ga_len += 1;
        if !all {
            break;
        }
        i += 1;
    }
    return num_fnames > 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn get_tagfname(
    mut tnp: *mut tagname_T,
    mut first: ::core::ffi::c_int,
    mut buf: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if first != 0 {
        memset(
            tnp as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<tagname_T>(),
        );
    }
    if (*curbuf.get()).b_help {
        if first != 0 {
            ga_clear_strings(tag_fnames.ptr());
            ga_init(
                tag_fnames.ptr(),
                ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                10 as ::core::ffi::c_int,
            );
            do_in_runtimepath(
                b"doc/tags doc/tags-??\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                DIP_ALL as ::core::ffi::c_int,
                Some(
                    found_tagfile_cb
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            *mut *mut ::core::ffi::c_char,
                            bool,
                            *mut ::core::ffi::c_void,
                        ) -> bool,
                ),
                NULL_0,
            );
        }
        if (*tnp).tn_hf_idx >= (*tag_fnames.ptr()).ga_len {
            if (*tnp).tn_hf_idx > (*tag_fnames.ptr()).ga_len
                || *p_hf.get() as ::core::ffi::c_int == NUL
            {
                return FAIL;
            }
            (*tnp).tn_hf_idx += 1;
            xstrlcpy(
                buf,
                p_hf.get(),
                (MAXPATHL as size_t).wrapping_sub(
                    ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                ),
            );
            strcpy(
                path_tail(buf),
                b"tags\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            simplify_filename(buf);
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*tag_fnames.ptr()).ga_len {
                if strcmp(
                    buf,
                    *((*tag_fnames.ptr()).ga_data as *mut *mut ::core::ffi::c_char)
                        .offset(i as isize),
                ) == 0 as ::core::ffi::c_int
                {
                    return FAIL;
                }
                i += 1;
            }
        } else {
            let c2rust_fresh5 = (*tnp).tn_hf_idx;
            (*tnp).tn_hf_idx = (*tnp).tn_hf_idx + 1;
            xstrlcpy(
                buf,
                *((*tag_fnames.ptr()).ga_data as *mut *mut ::core::ffi::c_char)
                    .offset(c2rust_fresh5 as isize),
                MAXPATHL as size_t,
            );
        }
        return OK;
    }
    if first != 0 {
        (*tnp).tn_tags = xstrdup(if *(*curbuf.get()).b_p_tags as ::core::ffi::c_int != NUL {
            (*curbuf.get()).b_p_tags
        } else {
            p_tags.get()
        });
        (*tnp).tn_np = (*tnp).tn_tags;
    }
    loop {
        if (*tnp).tn_did_filefind_init != 0 {
            fname = vim_findfile((*tnp).tn_search_ctx);
            if !fname.is_null() {
                break;
            }
            (*tnp).tn_did_filefind_init = false_0;
        } else {
            let mut filename: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            if *(*tnp).tn_np as ::core::ffi::c_int == NUL {
                vim_findfile_cleanup((*tnp).tn_search_ctx);
                (*tnp).tn_search_ctx = NULL_0;
                return FAIL;
            }
            *buf.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
            copy_option_part(
                &raw mut (*tnp).tn_np,
                buf,
                (MAXPATHL - 1 as ::core::ffi::c_int) as size_t,
                b" ,\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            let mut r_ptr: *mut ::core::ffi::c_char = vim_findfile_stopdir(buf);
            filename = path_tail(buf);
            if !r_ptr.is_null() {
                memmove(
                    r_ptr.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                    r_ptr as *const ::core::ffi::c_void,
                    strlen(r_ptr).wrapping_add(1 as size_t),
                );
                r_ptr = r_ptr.offset(1);
            }
            memmove(
                filename.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                filename as *const ::core::ffi::c_void,
                strlen(filename).wrapping_add(1 as size_t),
            );
            let c2rust_fresh6 = filename;
            filename = filename.offset(1);
            *c2rust_fresh6 = NUL as ::core::ffi::c_char;
            (*tnp).tn_search_ctx = vim_findfile_init(
                buf,
                filename,
                strlen(filename),
                r_ptr,
                100 as ::core::ffi::c_int,
                false_0,
                FINDFILE_FILE as ::core::ffi::c_int,
                (*tnp).tn_search_ctx,
                true_0,
                (*curbuf.get()).b_ffname,
            );
            if !(*tnp).tn_search_ctx.is_null() {
                (*tnp).tn_did_filefind_init = true_0;
            }
        }
    }
    strcpy(buf, fname);
    xfree(fname as *mut ::core::ffi::c_void);
    return OK;
}
pub unsafe extern "C" fn tagname_free(mut tnp: *mut tagname_T) {
    xfree((*tnp).tn_tags as *mut ::core::ffi::c_void);
    vim_findfile_cleanup((*tnp).tn_search_ctx);
    (*tnp).tn_search_ctx = NULL_0;
    ga_clear_strings(tag_fnames.ptr());
}
unsafe extern "C" fn parse_tag_line(
    mut lbuf: *mut ::core::ffi::c_char,
    mut tagp: *mut tagptrs_T,
) -> ::core::ffi::c_int {
    (*tagp).tagname = lbuf;
    let mut p: *mut ::core::ffi::c_char = vim_strchr(lbuf, TAB);
    if p.is_null() {
        return FAIL;
    }
    (*tagp).tagname_end = p;
    if *p as ::core::ffi::c_int != NUL {
        p = p.offset(1);
    }
    (*tagp).fname = p;
    p = vim_strchr(p, TAB);
    if p.is_null() {
        return FAIL;
    }
    (*tagp).fname_end = p;
    if *p as ::core::ffi::c_int != NUL {
        p = p.offset(1);
    }
    if *p as ::core::ffi::c_int == NUL {
        return FAIL;
    }
    (*tagp).command = p;
    return OK;
}
unsafe extern "C" fn test_for_static(mut tagp: *mut tagptrs_T) -> bool {
    let mut p: *mut ::core::ffi::c_char = (*tagp).command;
    loop {
        p = vim_strchr(p, '\t' as ::core::ffi::c_int);
        if p.is_null() {
            break;
        }
        p = p.offset(1);
        if strncmp(
            p,
            b"file:\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
unsafe extern "C" fn matching_line_len(lbuf: *const ::core::ffi::c_char) -> size_t {
    let mut p: *const ::core::ffi::c_char = lbuf.offset(1 as ::core::ffi::c_int as isize);
    p = p.offset(strlen(p).wrapping_add(1 as size_t) as isize);
    return (p.offset_from(lbuf) as size_t).wrapping_add(strlen(p));
}
unsafe extern "C" fn parse_match(
    mut lbuf: *mut ::core::ffi::c_char,
    mut tagp: *mut tagptrs_T,
) -> ::core::ffi::c_int {
    (*tagp).tag_fname = lbuf.offset(1 as ::core::ffi::c_int as isize);
    lbuf = lbuf.offset(strlen((*tagp).tag_fname).wrapping_add(2 as size_t) as isize);
    let mut retval: ::core::ffi::c_int = parse_tag_line(lbuf, tagp);
    (*tagp).tagkind = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*tagp).user_data = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*tagp).tagline = 0 as ::core::ffi::c_int as linenr_T;
    (*tagp).command_end = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if retval != OK {
        return retval;
    }
    let mut p: *mut ::core::ffi::c_char = (*tagp).command;
    if find_extra(&raw mut p) == OK {
        (*tagp).command_end = p;
        if p > (*tagp).command
            && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '|' as ::core::ffi::c_int
        {
            (*tagp).command_end = p.offset(-(1 as ::core::ffi::c_int as isize));
        }
        p = p.offset(2 as ::core::ffi::c_int as isize);
        let c2rust_fresh3 = p;
        p = p.offset(1);
        if *c2rust_fresh3 as ::core::ffi::c_int == TAB {
            while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                    && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                || utfc_ptr2len(p) > 1 as ::core::ffi::c_int
            {
                if strncmp(
                    p,
                    b"kind:\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    (*tagp).tagkind = p.offset(5 as ::core::ffi::c_int as isize);
                } else if strncmp(
                    p,
                    b"user_data:\0".as_ptr() as *const ::core::ffi::c_char,
                    10 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    (*tagp).user_data = p.offset(10 as ::core::ffi::c_int as isize);
                } else if strncmp(
                    p,
                    b"line:\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    (*tagp).tagline = atoi(p.offset(5 as ::core::ffi::c_int as isize)) as linenr_T;
                }
                if !(*tagp).tagkind.is_null() && !(*tagp).user_data.is_null() {
                    break;
                }
                let mut pc: *mut ::core::ffi::c_char = vim_strchr(p, ':' as ::core::ffi::c_int);
                let mut pt: *mut ::core::ffi::c_char = vim_strchr(p, '\t' as ::core::ffi::c_int);
                if pc.is_null() || !pt.is_null() && pc > pt {
                    (*tagp).tagkind = p;
                }
                if pt.is_null() {
                    break;
                }
                p = pt;
                p = p.offset(utfc_ptr2len(p) as isize);
            }
        }
    }
    if !(*tagp).tagkind.is_null() {
        p = (*tagp).tagkind;
        while *p as ::core::ffi::c_int != 0
            && *p as ::core::ffi::c_int != '\t' as ::core::ffi::c_int
            && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
            && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
        {
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        (*tagp).tagkind_end = p;
    }
    if !(*tagp).user_data.is_null() {
        p = (*tagp).user_data;
        while *p as ::core::ffi::c_int != 0
            && *p as ::core::ffi::c_int != '\t' as ::core::ffi::c_int
            && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
            && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
        {
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        (*tagp).user_data_end = p;
    }
    return retval;
}
unsafe extern "C" fn tag_full_fname(mut tagp: *mut tagptrs_T) -> *mut ::core::ffi::c_char {
    let mut c: ::core::ffi::c_char = *(*tagp).fname_end;
    *(*tagp).fname_end = NUL as ::core::ffi::c_char;
    let mut fullname: *mut ::core::ffi::c_char =
        expand_tag_fname((*tagp).fname, (*tagp).tag_fname, false_0 != 0);
    *(*tagp).fname_end = c;
    return fullname;
}
unsafe extern "C" fn jumpto_tag(
    mut lbuf_arg: *const ::core::ffi::c_char,
    mut forceit: ::core::ffi::c_int,
    mut keep_help: bool,
) -> ::core::ffi::c_int {
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if postponed_split.get() == 0 as ::core::ffi::c_int && !check_can_set_curbuf_forceit(forceit) {
        return FAIL;
    }
    let mut pbuf_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tofree_fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tagp: tagptrs_T = tagptrs_T {
        tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagline: 0,
    };
    let mut retval: ::core::ffi::c_int = FAIL;
    let mut getfile_result: ::core::ffi::c_int = GETFILE_UNUSED as ::core::ffi::c_int;
    let mut search_options: ::core::ffi::c_int = 0;
    let mut curwin_save: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut full_fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let old_KeyTyped: bool = KeyTyped.get();
    let l_g_do_tagpreview: ::core::ffi::c_int = g_do_tagpreview.get();
    let len: size_t = matching_line_len(lbuf_arg).wrapping_add(1 as size_t);
    let mut lbuf: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    memmove(
        lbuf as *mut ::core::ffi::c_void,
        lbuf_arg as *const ::core::ffi::c_void,
        len,
    );
    let mut pbuf: *mut ::core::ffi::c_char =
        xmalloc(LSIZE as ::core::ffi::c_int as size_t) as *mut ::core::ffi::c_char;
    '_erret: {
        if parse_match(lbuf, &raw mut tagp) == FAIL {
            tagp.fname_end = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            *tagp.fname_end = NUL as ::core::ffi::c_char;
            fname = tagp.fname;
            str = tagp.command;
            pbuf_end = pbuf;
            while *str as ::core::ffi::c_int != 0
                && *str as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                && *str as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
            {
                let c2rust_fresh1 = str;
                str = str.offset(1);
                let c2rust_fresh2 = pbuf_end;
                pbuf_end = pbuf_end.offset(1);
                *c2rust_fresh2 = *c2rust_fresh1;
                if pbuf_end.offset_from(pbuf) + 1 as isize >= LSIZE as ::core::ffi::c_int as isize {
                    break;
                }
            }
            *pbuf_end = NUL as ::core::ffi::c_char;
            str = pbuf;
            if find_extra(&raw mut str) == OK {
                pbuf_end = str;
                *pbuf_end = NUL as ::core::ffi::c_char;
            }
            fname = expand_tag_fname(fname, tagp.tag_fname, true_0 != 0);
            tofree_fname = fname;
            if !os_path_exists(fname)
                && !has_autocmd(EVENT_BUFREADCMD, fname, ::core::ptr::null_mut::<buf_T>())
            {
                retval = NOTAGFILE;
                xfree(nofile_fname.get() as *mut ::core::ffi::c_void);
                nofile_fname.set(xstrdup(fname));
            } else {
                (*RedrawingDisabled.ptr()) += 1;
                if l_g_do_tagpreview != 0 as ::core::ffi::c_int {
                    postponed_split.set(0 as ::core::ffi::c_int);
                    curwin_save = curwin.get();
                    if (*curwin.get()).w_onebuf_opt.wo_pvw == 0 {
                        full_fname = FullName_save(fname, false_0 != 0);
                        fname = full_fname;
                        prepare_tagpreview(true_0 != 0);
                    }
                }
                if postponed_split.get() != 0
                    && swb_flags.get()
                        & (kOptSwbFlagUseopen as ::core::ffi::c_int
                            | kOptSwbFlagUsetab as ::core::ffi::c_int)
                            as ::core::ffi::c_uint
                        != 0
                {
                    let existing_buf: *mut buf_T = buflist_findname_exp(fname);
                    if !existing_buf.is_null() {
                        if !swbuf_goto_win_with_buf(existing_buf).is_null() {
                            getfile_result = GETFILE_SAME_FILE as ::core::ffi::c_int;
                        }
                    }
                }
                if getfile_result == GETFILE_UNUSED as ::core::ffi::c_int
                    && (postponed_split.get() != 0
                        || (*cmdmod.ptr()).cmod_tab != 0 as ::core::ffi::c_int)
                {
                    if swb_flags.get()
                        & kOptSwbFlagVsplit as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    {
                        (*cmdmod.ptr()).cmod_split |= WSP_VERT as ::core::ffi::c_int;
                    }
                    if swb_flags.get()
                        & kOptSwbFlagNewtab as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                        && (*cmdmod.ptr()).cmod_tab == 0 as ::core::ffi::c_int
                    {
                        (*cmdmod.ptr()).cmod_tab =
                            tabpage_index(curtab.get()) + 1 as ::core::ffi::c_int;
                    }
                    if win_split(
                        if postponed_split.get() > 0 as ::core::ffi::c_int {
                            postponed_split.get()
                        } else {
                            0 as ::core::ffi::c_int
                        },
                        postponed_split_flags.get(),
                    ) == FAIL
                    {
                        (*RedrawingDisabled.ptr()) -= 1;
                        break '_erret;
                    } else {
                        (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
                        (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
                    }
                }
                if keep_help {
                    if l_g_do_tagpreview != 0 as ::core::ffi::c_int {
                        keep_help_flag.set(bt_help((*curwin_save).w_buffer));
                    } else {
                        keep_help_flag.set((*curbuf.get()).b_help);
                    }
                }
                if getfile_result == GETFILE_UNUSED as ::core::ffi::c_int {
                    getfile_result = getfile(
                        0 as ::core::ffi::c_int,
                        fname,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        true_0 != 0,
                        0 as linenr_T,
                        forceit != 0,
                    );
                }
                keep_help_flag.set(false_0 != 0);
                if getfile_result <= 0 as ::core::ffi::c_int {
                    (*curwin.get()).w_set_curswant = true_0;
                    postponed_split.set(0 as ::core::ffi::c_int);
                    let save_magic_overruled: optmagic_T = magic_overruled.get();
                    magic_overruled.set(OPTION_MAGIC_OFF);
                    let save_no_hlsearch: bool = no_hlsearch.get();
                    if !vim_strchr(p_cpo.get(), CPO_TAGPAT).is_null() {
                        search_options = 0 as ::core::ffi::c_int;
                    } else {
                        search_options = SEARCH_KEEP as ::core::ffi::c_int;
                    }
                    str = pbuf;
                    if *pbuf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '/' as ::core::ffi::c_int
                        || *pbuf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '?' as ::core::ffi::c_int
                    {
                        str = skip_regexp(
                            pbuf.offset(1 as ::core::ffi::c_int as isize),
                            *pbuf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
                            false_0,
                        )
                        .offset(1 as ::core::ffi::c_int as isize);
                    }
                    if str > pbuf_end.offset(-(1 as ::core::ffi::c_int as isize)) {
                        let mut pbuflen: size_t = pbuf_end.offset_from(pbuf) as size_t;
                        let mut save_p_ws: bool = p_ws.get() != 0;
                        let mut save_p_ic: ::core::ffi::c_int = p_ic.get();
                        let mut save_p_scs: ::core::ffi::c_int = p_scs.get();
                        p_ws.set(true_0);
                        p_ic.set(false_0);
                        p_scs.set(false_0);
                        let mut save_lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
                        (*curwin.get()).w_cursor.lnum = if tagp.tagline > 0 as linenr_T {
                            tagp.tagline - 1 as linenr_T
                        } else {
                            0 as linenr_T
                        };
                        if do_search(
                            ::core::ptr::null_mut::<oparg_T>(),
                            *pbuf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
                            *pbuf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
                            pbuf.offset(1 as ::core::ffi::c_int as isize),
                            pbuflen.wrapping_sub(1 as size_t),
                            1 as ::core::ffi::c_int,
                            search_options,
                            ::core::ptr::null_mut::<searchit_arg_T>(),
                        ) != 0
                        {
                            retval = OK;
                        } else {
                            let mut found: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                            p_ic.set(true_0);
                            if do_search(
                                ::core::ptr::null_mut::<oparg_T>(),
                                *pbuf.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int,
                                *pbuf.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int,
                                pbuf.offset(1 as ::core::ffi::c_int as isize),
                                pbuflen.wrapping_sub(1 as size_t),
                                1 as ::core::ffi::c_int,
                                search_options,
                                ::core::ptr::null_mut::<searchit_arg_T>(),
                            ) == 0
                            {
                                found = 2 as ::core::ffi::c_int;
                                test_for_static(&raw mut tagp);
                                let mut cc: ::core::ffi::c_char = *tagp.tagname_end;
                                *tagp.tagname_end = NUL as ::core::ffi::c_char;
                                pbuflen = snprintf(
                                    pbuf,
                                    LSIZE as ::core::ffi::c_int as size_t,
                                    b"^%s\\s\\*(\0".as_ptr() as *const ::core::ffi::c_char,
                                    tagp.tagname,
                                ) as size_t;
                                if do_search(
                                    ::core::ptr::null_mut::<oparg_T>(),
                                    '/' as ::core::ffi::c_int,
                                    '/' as ::core::ffi::c_int,
                                    pbuf,
                                    pbuflen,
                                    1 as ::core::ffi::c_int,
                                    search_options,
                                    ::core::ptr::null_mut::<searchit_arg_T>(),
                                ) == 0
                                {
                                    pbuflen = snprintf(
                                        pbuf,
                                        LSIZE as ::core::ffi::c_int as size_t,
                                        b"^\\[#a-zA-Z_]\\.\\*\\<%s\\s\\*(\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        tagp.tagname,
                                    ) as size_t;
                                    if do_search(
                                        ::core::ptr::null_mut::<oparg_T>(),
                                        '/' as ::core::ffi::c_int,
                                        '/' as ::core::ffi::c_int,
                                        pbuf,
                                        pbuflen,
                                        1 as ::core::ffi::c_int,
                                        search_options,
                                        ::core::ptr::null_mut::<searchit_arg_T>(),
                                    ) == 0
                                    {
                                        found = 0 as ::core::ffi::c_int;
                                    }
                                }
                                *tagp.tagname_end = cc;
                            }
                            if found == 0 as ::core::ffi::c_int {
                                emsg(gettext(b"E434: Can't find tag pattern\0".as_ptr()
                                    as *const ::core::ffi::c_char));
                                (*curwin.get()).w_cursor.lnum = save_lnum;
                            } else {
                                if found == 2 as ::core::ffi::c_int || save_p_ic == 0 {
                                    msg(
                                        gettext(
                                            b"E435: Couldn't find tag, just guessing!\0".as_ptr()
                                                as *const ::core::ffi::c_char,
                                        ),
                                        0 as ::core::ffi::c_int,
                                    );
                                    if msg_scrolled.get() == 0
                                        && msg_silent.get() == 0 as ::core::ffi::c_int
                                    {
                                        msg_delay(1010 as uint64_t, true_0 != 0);
                                    }
                                }
                                retval = OK;
                            }
                        }
                        p_ws.set(save_p_ws as ::core::ffi::c_int);
                        p_ic.set(save_p_ic);
                        p_scs.set(save_p_scs);
                        check_cursor(curwin.get());
                    } else {
                        let save_secure: ::core::ffi::c_int = secure.get();
                        secure.set(1 as ::core::ffi::c_int);
                        (*sandbox.ptr()) += 1;
                        (*curwin.get()).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
                        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                        do_cmdline_cmd(pbuf);
                        retval = OK;
                        if secure.get() == 2 as ::core::ffi::c_int {
                            wait_return(true_0);
                        }
                        secure.set(save_secure);
                        (*sandbox.ptr()) -= 1;
                    }
                    magic_overruled.set(save_magic_overruled);
                    if search_options != 0 {
                        set_no_hlsearch(save_no_hlsearch);
                    }
                    if getfile_result == GETFILE_OPEN_OTHER as ::core::ffi::c_int {
                        retval = OK;
                    }
                    if retval == OK {
                        if (*curbuf.get()).b_help {
                            set_topline(curwin.get(), (*curwin.get()).w_cursor.lnum);
                        }
                        if fdo_flags.get()
                            & kOptFdoFlagTag as ::core::ffi::c_int as ::core::ffi::c_uint
                            != 0
                            && old_KeyTyped as ::core::ffi::c_int != 0
                        {
                            foldOpenCursor();
                        }
                    }
                    if l_g_do_tagpreview != 0 as ::core::ffi::c_int
                        && curwin.get() != curwin_save
                        && win_valid(curwin_save) as ::core::ffi::c_int != 0
                    {
                        validate_cursor(curwin.get());
                        redraw_later(curwin.get(), UPD_VALID);
                        win_enter(curwin_save, true_0 != 0);
                    }
                    (*RedrawingDisabled.ptr()) -= 1;
                } else {
                    (*RedrawingDisabled.ptr()) -= 1;
                    if postponed_split.get() != 0 {
                        win_close(curwin.get(), false_0 != 0, false_0 != 0);
                        postponed_split.set(0 as ::core::ffi::c_int);
                    }
                }
            }
        }
    }
    g_do_tagpreview.set(0 as ::core::ffi::c_int);
    xfree(lbuf as *mut ::core::ffi::c_void);
    xfree(pbuf as *mut ::core::ffi::c_void);
    xfree(tofree_fname as *mut ::core::ffi::c_void);
    xfree(full_fname as *mut ::core::ffi::c_void);
    return retval;
}
unsafe extern "C" fn expand_tag_fname(
    mut fname: *mut ::core::ffi::c_char,
    tag_fname: *mut ::core::ffi::c_char,
    expand: bool,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded_fname: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut xpc: expand_T = expand_T {
        xp_pattern: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        xp_context: 0,
        xp_pattern_len: 0,
        xp_prefix: XP_PREFIX_NONE,
        xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        xp_luaref: 0,
        xp_script_ctx: sctx_T {
            sc_sid: 0,
            sc_seq: 0,
            sc_lnum: 0,
            sc_chan: 0,
        },
        xp_backslash: 0,
        xp_shell: false,
        xp_numfiles: 0,
        xp_col: 0,
        xp_selected: 0,
        xp_orig: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        xp_files: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        xp_line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        xp_buf: [0; 256],
        xp_search_dir: kDirectionNotSet,
        xp_pre_incsearch_pos: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
    };
    if expand as ::core::ffi::c_int != 0
        && path_has_wildcard(fname) as ::core::ffi::c_int != 0
        && vim_strchr(fname, '`' as ::core::ffi::c_int).is_null()
    {
        ExpandInit(&raw mut xpc);
        xpc.xp_context = EXPAND_FILES as ::core::ffi::c_int;
        expanded_fname = ExpandOne(
            &raw mut xpc,
            fname,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            WILD_LIST_NOTFOUND as ::core::ffi::c_int | WILD_SILENT as ::core::ffi::c_int,
            WILD_EXPAND_FREE as ::core::ffi::c_int,
        );
        if !expanded_fname.is_null() {
            fname = expanded_fname;
        }
    }
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (p_tr.get() != 0 || (*curbuf.get()).b_help as ::core::ffi::c_int != 0)
        && !vim_isAbsName(fname)
        && {
            p = path_tail(tag_fname);
            p != tag_fname
        }
    {
        retval = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        strcpy(retval, tag_fname);
        xstrlcpy(
            retval.offset(p.offset_from(tag_fname) as isize),
            fname,
            (MAXPATHL as isize - p.offset_from(tag_fname)) as size_t,
        );
        simplify_filename(retval);
    } else {
        retval = xstrdup(fname);
    }
    xfree(expanded_fname as *mut ::core::ffi::c_void);
    return retval;
}
unsafe extern "C" fn test_for_current(
    mut fname: *mut ::core::ffi::c_char,
    mut fname_end: *mut ::core::ffi::c_char,
    mut tag_fname: *mut ::core::ffi::c_char,
    mut buf_ffname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = false_0;
    if !buf_ffname.is_null() {
        let mut c: ::core::ffi::c_char = 0;
        c = *fname_end;
        *fname_end = NUL as ::core::ffi::c_char;
        let mut fullname: *mut ::core::ffi::c_char =
            expand_tag_fname(fname, tag_fname, true_0 != 0);
        retval = (path_full_compare(fullname, buf_ffname, true_0 != 0, true_0 != 0)
            as ::core::ffi::c_uint
            & kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint)
            as ::core::ffi::c_int;
        xfree(fullname as *mut ::core::ffi::c_void);
        *fname_end = c;
    }
    return retval;
}
unsafe extern "C" fn find_extra(mut pp: *mut *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut str: *mut ::core::ffi::c_char = *pp;
    let mut first_char: ::core::ffi::c_char = **pp;
    loop {
        if ascii_isdigit(*str as ::core::ffi::c_int) {
            str = skipdigits(str.offset(1 as ::core::ffi::c_int as isize));
        } else if *str as ::core::ffi::c_int == '/' as ::core::ffi::c_int
            || *str as ::core::ffi::c_int == '?' as ::core::ffi::c_int
        {
            str = skip_regexp(
                str.offset(1 as ::core::ffi::c_int as isize),
                *str as ::core::ffi::c_int,
                false_0,
            );
            if *str as ::core::ffi::c_int != first_char as ::core::ffi::c_int {
                str = ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else {
                str = str.offset(1);
            }
        } else {
            str = strstr(str, b"|;\"\0".as_ptr() as *const ::core::ffi::c_char);
            if !str.is_null() {
                str = str.offset(1);
                break;
            }
        }
        if str.is_null()
            || *str as ::core::ffi::c_int != ';' as ::core::ffi::c_int
            || !(ascii_isdigit(*str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
                || *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '/' as ::core::ffi::c_int
                || *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '?' as ::core::ffi::c_int)
        {
            break;
        }
        str = str.offset(1);
        first_char = *str;
    }
    if !str.is_null()
        && strncmp(
            str,
            b";\"\0".as_ptr() as *const ::core::ffi::c_char,
            2 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        *pp = str;
        return OK;
    }
    return FAIL;
}
pub unsafe extern "C" fn tagstack_clear_entry(mut item: *mut taggy_T) {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*item).tagname as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    let _ = *ptr_;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*item).user_data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL_0;
    let _ = *ptr__0;
}
pub unsafe extern "C" fn expand_tags(
    mut tagnames: bool,
    mut pat: *mut ::core::ffi::c_char,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut name_buf_size: size_t = 100 as size_t;
    let mut ret: ::core::ffi::c_int = 0;
    let mut name_buf: *mut ::core::ffi::c_char = xmalloc(name_buf_size) as *mut ::core::ffi::c_char;
    let mut extra_flag: ::core::ffi::c_int = if tagnames as ::core::ffi::c_int != 0 {
        TAG_NAMES as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    if *pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '/' as ::core::ffi::c_int
    {
        ret = find_tags(
            pat.offset(1 as ::core::ffi::c_int as isize),
            num_file,
            file,
            TAG_REGEXP as ::core::ffi::c_int
                | extra_flag
                | TAG_VERBOSE as ::core::ffi::c_int
                | TAG_NO_TAGFUNC as ::core::ffi::c_int,
            TAG_MANY as ::core::ffi::c_int,
            (*curbuf.get()).b_ffname,
        );
    } else {
        ret = find_tags(
            pat,
            num_file,
            file,
            TAG_REGEXP as ::core::ffi::c_int
                | extra_flag
                | TAG_VERBOSE as ::core::ffi::c_int
                | TAG_NO_TAGFUNC as ::core::ffi::c_int
                | TAG_NOIC as ::core::ffi::c_int,
            TAG_MANY as ::core::ffi::c_int,
            (*curbuf.get()).b_ffname,
        );
    }
    if ret == OK && !tagnames {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < *num_file {
            let mut t_p: tagptrs_T = tagptrs_T {
                tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                tagline: 0,
            };
            parse_match(*(*file).offset(i as isize), &raw mut t_p);
            let mut len: size_t = t_p.tagname_end.offset_from(t_p.tagname) as size_t;
            if len > name_buf_size.wrapping_sub(3 as size_t) {
                name_buf_size = len.wrapping_add(3 as size_t);
                let mut buf: *mut ::core::ffi::c_char =
                    xrealloc(name_buf as *mut ::core::ffi::c_void, name_buf_size)
                        as *mut ::core::ffi::c_char;
                name_buf = buf;
            }
            memmove(
                name_buf as *mut ::core::ffi::c_void,
                t_p.tagname as *const ::core::ffi::c_void,
                len,
            );
            let c2rust_fresh14 = len;
            len = len.wrapping_add(1);
            *name_buf.offset(c2rust_fresh14 as isize) = 0 as ::core::ffi::c_char;
            let c2rust_fresh15 = len;
            len = len.wrapping_add(1);
            *name_buf.offset(c2rust_fresh15 as isize) =
                (if !t_p.tagkind.is_null() && *t_p.tagkind as ::core::ffi::c_int != 0 {
                    *t_p.tagkind as ::core::ffi::c_int
                } else {
                    'f' as ::core::ffi::c_int
                }) as ::core::ffi::c_char;
            let c2rust_fresh16 = len;
            len = len.wrapping_add(1);
            *name_buf.offset(c2rust_fresh16 as isize) = 0 as ::core::ffi::c_char;
            memmove(
                (*(*file).offset(i as isize)).offset(len as isize) as *mut ::core::ffi::c_void,
                t_p.fname as *const ::core::ffi::c_void,
                t_p.fname_end.offset_from(t_p.fname) as size_t,
            );
            *(*(*file).offset(i as isize)).offset(
                len.wrapping_add(t_p.fname_end.offset_from(t_p.fname) as size_t) as isize,
            ) = 0 as ::core::ffi::c_char;
            memmove(
                *(*file).offset(i as isize) as *mut ::core::ffi::c_void,
                name_buf as *const ::core::ffi::c_void,
                len,
            );
            i += 1;
        }
    }
    xfree(name_buf as *mut ::core::ffi::c_void);
    return ret;
}
unsafe extern "C" fn add_tag_field(
    mut dict: *mut dict_T,
    mut field_name: *const ::core::ffi::c_char,
    mut start: *const ::core::ffi::c_char,
    mut end: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if !tv_dict_find(dict, field_name, -1 as ptrdiff_t).is_null() {
        if p_verbose.get() > 0 as OptInt {
            verbose_enter();
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Duplicate field name: %s\0".as_ptr() as *const ::core::ffi::c_char),
                field_name,
            );
            verbose_leave();
        }
        return FAIL;
    }
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut buf: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    if !start.is_null() {
        if end.is_null() {
            end = start.offset(strlen(start) as isize);
            while end > start
                && (*end.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\r' as ::core::ffi::c_int
                    || *end.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\n' as ::core::ffi::c_int)
            {
                end = end.offset(-1);
            }
        }
        len = if (end.offset_from(start) as ::core::ffi::c_int)
            < 4096 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        {
            end.offset_from(start) as ::core::ffi::c_int
        } else {
            4096 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        };
        xmemcpyz(
            buf as *mut ::core::ffi::c_void,
            start as *const ::core::ffi::c_void,
            len as size_t,
        );
    }
    *buf.offset(len as isize) = NUL as ::core::ffi::c_char;
    let mut retval: ::core::ffi::c_int = tv_dict_add_str(dict, field_name, strlen(field_name), buf);
    xfree(buf as *mut ::core::ffi::c_void);
    return retval;
}
pub unsafe extern "C" fn get_tags(
    mut list: *mut list_T,
    mut pat: *mut ::core::ffi::c_char,
    mut buf_fname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut num_matches: ::core::ffi::c_int = 0;
    let mut matches: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut tp: tagptrs_T = tagptrs_T {
        tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagline: 0,
    };
    let mut ret: ::core::ffi::c_int = find_tags(
        pat,
        &raw mut num_matches,
        &raw mut matches,
        TAG_REGEXP as ::core::ffi::c_int | TAG_NOIC as ::core::ffi::c_int,
        MAXCOL as ::core::ffi::c_int,
        buf_fname,
    );
    if ret != OK || num_matches <= 0 as ::core::ffi::c_int {
        return ret;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_matches {
        if parse_match(*matches.offset(i as isize), &raw mut tp) == FAIL {
            xfree(*matches.offset(i as isize) as *mut ::core::ffi::c_void);
        } else {
            let mut is_static: bool = test_for_static(&raw mut tp);
            if strncmp(
                tp.tagname,
                b"!_TAG_\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                xfree(*matches.offset(i as isize) as *mut ::core::ffi::c_void);
            } else {
                let mut dict: *mut dict_T = tv_dict_alloc();
                tv_list_append_dict(list, dict);
                let mut full_fname: *mut ::core::ffi::c_char = tag_full_fname(&raw mut tp);
                if add_tag_field(
                    dict,
                    b"name\0".as_ptr() as *const ::core::ffi::c_char,
                    tp.tagname,
                    tp.tagname_end,
                ) == FAIL
                    || add_tag_field(
                        dict,
                        b"filename\0".as_ptr() as *const ::core::ffi::c_char,
                        full_fname,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    ) == FAIL
                    || add_tag_field(
                        dict,
                        b"cmd\0".as_ptr() as *const ::core::ffi::c_char,
                        tp.command,
                        tp.command_end,
                    ) == FAIL
                    || add_tag_field(
                        dict,
                        b"kind\0".as_ptr() as *const ::core::ffi::c_char,
                        tp.tagkind,
                        if !tp.tagkind.is_null() {
                            tp.tagkind_end
                        } else {
                            ::core::ptr::null_mut::<::core::ffi::c_char>()
                        },
                    ) == FAIL
                    || tv_dict_add_nr(
                        dict,
                        b"static\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                            .wrapping_sub(1 as size_t),
                        is_static as varnumber_T,
                    ) == FAIL
                {
                    ret = FAIL;
                }
                xfree(full_fname as *mut ::core::ffi::c_void);
                if !tp.command_end.is_null() {
                    let mut p: *mut ::core::ffi::c_char =
                        tp.command_end.offset(3 as ::core::ffi::c_int as isize);
                    while *p as ::core::ffi::c_int != NUL
                        && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                        && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                    {
                        if p == tp.tagkind
                            || p.offset(5 as ::core::ffi::c_int as isize) == tp.tagkind
                                && strncmp(
                                    p,
                                    b"kind:\0".as_ptr() as *const ::core::ffi::c_char,
                                    5 as size_t,
                                ) == 0 as ::core::ffi::c_int
                        {
                            p = tp.tagkind_end.offset(-(1 as ::core::ffi::c_int as isize));
                        } else if strncmp(
                            p,
                            b"file:\0".as_ptr() as *const ::core::ffi::c_char,
                            5 as size_t,
                        ) == 0 as ::core::ffi::c_int
                        {
                            p = p.offset(4 as ::core::ffi::c_int as isize);
                        } else if !ascii_iswhite(*p as ::core::ffi::c_int) {
                            let mut len: ::core::ffi::c_int = 0;
                            let mut n: *mut ::core::ffi::c_char = p;
                            while *p as ::core::ffi::c_int != NUL
                                && *p as ::core::ffi::c_int >= ' ' as ::core::ffi::c_int
                                && (*p as ::core::ffi::c_int) < 127 as ::core::ffi::c_int
                                && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
                            {
                                p = p.offset(1);
                            }
                            len = p.offset_from(n) as ::core::ffi::c_int;
                            if *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                                && len > 0 as ::core::ffi::c_int
                            {
                                p = p.offset(1);
                                let mut s: *mut ::core::ffi::c_char = p;
                                while *p as ::core::ffi::c_int != NUL
                                    && *p as uint8_t as ::core::ffi::c_int
                                        >= ' ' as ::core::ffi::c_int
                                {
                                    p = p.offset(1);
                                }
                                *n.offset(len as isize) = NUL as ::core::ffi::c_char;
                                if add_tag_field(dict, n, s, p) == FAIL {
                                    ret = FAIL;
                                }
                                *n.offset(len as isize) = ':' as ::core::ffi::c_char;
                            } else {
                                while *p as ::core::ffi::c_int != NUL
                                    && *p as uint8_t as ::core::ffi::c_int
                                        >= ' ' as ::core::ffi::c_int
                                {
                                    p = p.offset(1);
                                }
                            }
                            if *p as ::core::ffi::c_int == NUL {
                                break;
                            }
                        }
                        p = p.offset(utfc_ptr2len(p) as isize);
                    }
                }
                xfree(*matches.offset(i as isize) as *mut ::core::ffi::c_void);
            }
        }
        i += 1;
    }
    xfree(matches as *mut ::core::ffi::c_void);
    return ret;
}
unsafe extern "C" fn get_tag_details(mut tag: *mut taggy_T, mut retdict: *mut dict_T) {
    tv_dict_add_str(
        retdict,
        b"tagname\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (*tag).tagname,
    );
    tv_dict_add_nr(
        retdict,
        b"matchnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        ((*tag).cur_match + 1 as ::core::ffi::c_int) as varnumber_T,
    );
    tv_dict_add_nr(
        retdict,
        b"bufnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        (*tag).cur_fnum as varnumber_T,
    );
    if !(*tag).user_data.is_null() {
        tv_dict_add_str(
            retdict,
            b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            (*tag).user_data,
        );
    }
    let mut pos: *mut list_T = tv_list_alloc(4 as ptrdiff_t);
    tv_dict_add_list(
        retdict,
        b"from\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        pos,
    );
    let mut fmark: *mut fmark_T = &raw mut (*tag).fmark;
    tv_list_append_number(
        pos,
        (if (*fmark).fnum != -1 as ::core::ffi::c_int {
            (*fmark).fnum
        } else {
            0 as ::core::ffi::c_int
        }) as varnumber_T,
    );
    tv_list_append_number(pos, (*fmark).mark.lnum as varnumber_T);
    tv_list_append_number(
        pos,
        (if (*fmark).mark.col == MAXCOL as ::core::ffi::c_int {
            MAXCOL as ::core::ffi::c_int
        } else {
            (*fmark).mark.col as ::core::ffi::c_int + 1 as ::core::ffi::c_int
        }) as varnumber_T,
    );
    tv_list_append_number(pos, (*fmark).mark.coladd as varnumber_T);
}
pub unsafe extern "C" fn get_tagstack(mut wp: *mut win_T, mut retdict: *mut dict_T) {
    tv_dict_add_nr(
        retdict,
        b"length\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        (*wp).w_tagstacklen as varnumber_T,
    );
    tv_dict_add_nr(
        retdict,
        b"curidx\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        ((*wp).w_tagstackidx + 1 as ::core::ffi::c_int) as varnumber_T,
    );
    let mut l: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
    tv_dict_add_list(
        retdict,
        b"items\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        l,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*wp).w_tagstacklen {
        let mut d: *mut dict_T = tv_dict_alloc();
        tv_list_append_dict(l, d);
        get_tag_details(
            (&raw mut (*wp).w_tagstack as *mut taggy_T).offset(i as isize),
            d,
        );
        i += 1;
    }
}
unsafe extern "C" fn tagstack_clear(mut wp: *mut win_T) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*wp).w_tagstacklen {
        tagstack_clear_entry((&raw mut (*wp).w_tagstack as *mut taggy_T).offset(i as isize));
        i += 1;
    }
    (*wp).w_tagstacklen = 0 as ::core::ffi::c_int;
    (*wp).w_tagstackidx = 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tagstack_shift(mut wp: *mut win_T) {
    let mut tagstack: *mut taggy_T = &raw mut (*wp).w_tagstack as *mut taggy_T;
    tagstack_clear_entry(tagstack.offset(0 as ::core::ffi::c_int as isize));
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i < (*wp).w_tagstacklen {
        *tagstack.offset((i - 1 as ::core::ffi::c_int) as isize) = *tagstack.offset(i as isize);
        i += 1;
    }
    (*wp).w_tagstacklen -= 1;
}
unsafe extern "C" fn tagstack_push_item(
    mut wp: *mut win_T,
    mut tagname: *mut ::core::ffi::c_char,
    mut cur_fnum: ::core::ffi::c_int,
    mut cur_match: ::core::ffi::c_int,
    mut mark: pos_T,
    mut fnum: ::core::ffi::c_int,
    mut user_data: *mut ::core::ffi::c_char,
) {
    let mut tagstack: *mut taggy_T = &raw mut (*wp).w_tagstack as *mut taggy_T;
    let mut idx: ::core::ffi::c_int = (*wp).w_tagstacklen;
    if idx >= TAGSTACKSIZE {
        tagstack_shift(wp);
        idx = TAGSTACKSIZE - 1 as ::core::ffi::c_int;
    }
    (*wp).w_tagstacklen += 1;
    (*tagstack.offset(idx as isize)).tagname = tagname;
    (*tagstack.offset(idx as isize)).cur_fnum = cur_fnum;
    (*tagstack.offset(idx as isize)).cur_match = cur_match;
    (*tagstack.offset(idx as isize)).cur_match =
        if (*tagstack.offset(idx as isize)).cur_match > 0 as ::core::ffi::c_int {
            (*tagstack.offset(idx as isize)).cur_match
        } else {
            0 as ::core::ffi::c_int
        };
    (*tagstack.offset(idx as isize)).fmark.mark = mark;
    (*tagstack.offset(idx as isize)).fmark.fnum = fnum;
    (*tagstack.offset(idx as isize)).fmark.view = fmarkv_T {
        topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
        skipcol: 0 as colnr_T,
    };
    (*tagstack.offset(idx as isize)).user_data = user_data;
}
unsafe extern "C" fn tagstack_push_items(mut wp: *mut win_T, mut l: *mut list_T) {
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut tagname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mark: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut fnum: ::core::ffi::c_int = 0;
    let mut li: *mut listitem_T = tv_list_first(l);
    while !li.is_null() {
        if !((*li).li_tv.v_type as ::core::ffi::c_uint
            != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*li).li_tv.vval.v_dict.is_null())
        {
            let mut itemdict: *mut dict_T = (*li).li_tv.vval.v_dict;
            di = tv_dict_find(
                itemdict,
                b"from\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                if list2fpos(
                    &raw mut (*di).di_tv,
                    &raw mut mark,
                    &raw mut fnum,
                    ::core::ptr::null_mut::<colnr_T>(),
                    false_0 != 0,
                ) == OK
                {
                    tagname = tv_dict_get_string(
                        itemdict,
                        b"tagname\0".as_ptr() as *const ::core::ffi::c_char,
                        true_0 != 0,
                    );
                    if !tagname.is_null() {
                        if mark.col > 0 as ::core::ffi::c_int {
                            mark.col -= 1;
                        }
                        tagstack_push_item(
                            wp,
                            tagname,
                            tv_dict_get_number(
                                itemdict,
                                b"bufnr\0".as_ptr() as *const ::core::ffi::c_char,
                            ) as ::core::ffi::c_int,
                            tv_dict_get_number(
                                itemdict,
                                b"matchnr\0".as_ptr() as *const ::core::ffi::c_char,
                            ) as ::core::ffi::c_int
                                - 1 as ::core::ffi::c_int,
                            mark,
                            fnum,
                            tv_dict_get_string(
                                itemdict,
                                b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
                                true_0 != 0,
                            ),
                        );
                    }
                }
            }
        }
        li = (*li).li_next;
    }
}
unsafe extern "C" fn tagstack_set_curidx(mut wp: *mut win_T, mut curidx: ::core::ffi::c_int) {
    (*wp).w_tagstackidx = curidx;
    (*wp).w_tagstackidx = if (if (*wp).w_tagstackidx > 0 as ::core::ffi::c_int {
        (*wp).w_tagstackidx
    } else {
        0 as ::core::ffi::c_int
    }) < (*wp).w_tagstacklen
    {
        if (*wp).w_tagstackidx > 0 as ::core::ffi::c_int {
            (*wp).w_tagstackidx
        } else {
            0 as ::core::ffi::c_int
        }
    } else {
        (*wp).w_tagstacklen
    };
}
pub unsafe extern "C" fn set_tagstack(
    mut wp: *mut win_T,
    mut d: *const dict_T,
    mut action: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if tfu_in_use.get() {
        emsg(gettext(
            (e_cannot_modify_tag_stack_within_tagfunc.ptr() as *const _)
                as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    di = tv_dict_find(
        d,
        b"items\0".as_ptr() as *const ::core::ffi::c_char,
        -1 as ptrdiff_t,
    );
    if !di.is_null() {
        if (*di).di_tv.v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
            return FAIL;
        }
        l = (*di).di_tv.vval.v_list;
    }
    di = tv_dict_find(
        d,
        b"curidx\0".as_ptr() as *const ::core::ffi::c_char,
        -1 as ptrdiff_t,
    );
    if !di.is_null() {
        tagstack_set_curidx(
            wp,
            tv_get_number(&raw mut (*di).di_tv) as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        );
    }
    if action == 't' as ::core::ffi::c_int {
        let tagstack: *mut taggy_T = &raw mut (*wp).w_tagstack as *mut taggy_T;
        let tagstackidx: ::core::ffi::c_int = (*wp).w_tagstackidx;
        let mut tagstacklen: ::core::ffi::c_int = (*wp).w_tagstacklen;
        while tagstackidx < tagstacklen {
            tagstacklen -= 1;
            tagstack_clear_entry(tagstack.offset(tagstacklen as isize));
        }
        (*wp).w_tagstacklen = tagstacklen;
    }
    if !l.is_null() {
        if action == 'r' as ::core::ffi::c_int {
            tagstack_clear(wp);
        }
        tagstack_push_items(wp, l);
        (*wp).w_tagstackidx = (*wp).w_tagstacklen;
    }
    return OK;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
