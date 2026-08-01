use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::autocmd::{EVENT_SEARCHWRAPPED, apply_autocmds};
use crate::src::nvim::buffer::buf_get_changedtick;
use crate::src::nvim::change::get_leader_len;
use crate::src::nvim::charset::{skipwhite, vim_isfilec, vim_iswordc, vim_iswordp};
use crate::src::nvim::cmdhist::add_to_history;
use crate::src::nvim::cursor::{
    check_cursor, dec_cursor, get_cursor_line_len, get_cursor_line_ptr, inc_cursor,
};
use crate::src::nvim::drawscreen::{
    UPD_INVERTED, UPD_SOME_VALID, UPD_VALID, redraw_all_later, redraw_curbuf_later, redraw_later,
    setcursor, show_cursor_info_later, showmode, update_screen,
};
use crate::src::nvim::eval::typval::tv_list_len;
use crate::src::nvim::eval::typval::{
    tv_check_for_nonnull_dict_arg, tv_dict_add_nr, tv_dict_alloc_ret, tv_dict_find,
    tv_get_number_chk, tv_get_string_chk, tv_list_find,
};
use crate::src::nvim::eval::vars::set_vim_var_nr;
use crate::src::nvim::ex_cmds::{getfile, prepare_tagpreview};
use crate::src::nvim::ex_docmd::set_no_hlsearch;
use crate::src::nvim::ex_getln::gotocmdline;
use crate::src::nvim::file_search::{file_name_in_line, find_file_name_in_path};
use crate::src::nvim::fileio::vim_fgets;
use crate::src::nvim::fold::{foldOpenCursor, hasFolding};
use crate::src::nvim::getchar::char_avail;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::indent_c::is_pos_in_string;
use crate::src::nvim::insexpand::{
    compl_status_adding, compl_status_sol, ctrl_x_mode_not_default, find_word_end, find_word_start,
    ins_compl_add_infercase, ins_compl_check_keys, ins_compl_interrupted, ins_compl_len,
};
use crate::src::nvim::main::{
    Columns, IObuff, KeyStuffed, KeyTyped, Rows, State, VIsual, VIsual_active, VIsual_mode,
    bot_top_msg, called_emsg, cmd_silent, cmdmod, curbuf, curwin, dollar_vcol, e_interr, e_invarg2,
    e_nopresub, e_noprevre, e_patnotf2, emsg_off, fdo_flags, g_do_tagpreview, got_int,
    msg_ext_overwrite, msg_hist_off, msg_nowait, msg_row, msg_scrolled, msg_silent, no_hlsearch,
    no_smartcase, p_cpo, p_def, p_hls, p_ic, p_inc, p_js, p_mat, p_msc, p_ri, p_scs, p_sel, p_siso,
    p_so, p_verbose, p_ws, rc_did_emsg, sc_col, search_match_endcol, search_match_lines,
    searchcmdlen, top_bot_msg,
};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::mbyte::{
    mb_isupper, mb_strcmp_ic, mb_strnicmp, utf_char2bytes, utf_head_off, utf_iscomposing_first,
    utf_ptr2char, utfc_ptr2len,
};
use crate::src::nvim::memline::{decl, inc, incl, ml_get, ml_get_buf, ml_get_buf_len, ml_get_len};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xmemdupz, xstrlcpy};
use crate::src::nvim::message::{
    emsg, give_warning, iemsg, messaging, msg, msg_check, msg_clr_eos, msg_end, msg_ext_set_kind,
    msg_home_replace, msg_outtrans, msg_prt_line, msg_putchar, msg_puts, msg_puts_hl,
    msg_puts_title, msg_start, msg_strtrunc, msg_trunc, semsg, smsg, verbose_enter, verbose_leave,
};
use crate::src::nvim::mouse::setmouse;
use crate::src::nvim::r#move::validate_cursor;
use crate::src::nvim::normal::may_start_select;
use crate::src::nvim::option::{magic_isset, shortmess};
use crate::src::nvim::options::{kOptBoFlagShowmatch, kOptFdoFlagSearch};
use crate::src::nvim::os::fs::os_fopen;
use crate::src::nvim::os::input::{fast_breakcheck, line_breakcheck};
use crate::src::nvim::os::libc::{
    __assert_fail, atol, fclose, gettext, memcpy, memmove, memset, snprintf, strlen, strncmp,
    strncpy, strpbrk, strstr,
};
use crate::src::nvim::os::time::{os_delay, os_time};
use crate::src::nvim::path::path_full_compare;
use crate::src::nvim::plines::getvcol;
use crate::src::nvim::pos::{clearpos, equalpos, lt, ltoreq};
use crate::src::nvim::profile::{profile_passed_limit, profile_setlimit};
use crate::src::nvim::regexp::skip_regexp_ex;
use crate::src::nvim::regexp::vim_regexec_multi;
use crate::src::nvim::state::MODE_SHOWMATCH;
use crate::src::nvim::strings::{reverse_text, vim_snprintf, vim_strchr, xstrnsave};
use crate::src::nvim::types::ui::kUIMessages;
use crate::src::nvim::types::{
    Direction, EvalFuncData, FILE, MotionType, OptInt, SearchOffset, SearchPattern, TriState,
    VarType, VimVarIndex, buf_T, cmdarg_T, colnr_T, dict_T, file_comparison, int64_t, linenr_T,
    list_T, lpos_T, magic_T, oparg_T, pos_T, proftime_T, ptrdiff_t, regmatch_T, regmmatch_T,
    regprog_T, searchit_arg_T, size_t, typval_T, uint8_t, uint64_t, varnumber_T, win_T,
};
use crate::src::nvim::ui::{
    ui_busy_start, ui_busy_stop, ui_cursor_shape, ui_flush, ui_has, vim_beep,
};
use crate::src::nvim::window::{win_enter, win_split, win_valid};

// The carve of the transpiled module; see each child's docs.
mod pattern;
pub use self::pattern::*;
mod find;
pub use self::find::*;
mod command;
pub use self::command::*;
mod matchpair;
pub use self::matchpair::*;
mod stat;
pub use self::stat::*;
unsafe extern "C" {
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub const VAR_LIST: VarType = 4;
pub const VAR_UNKNOWN: VarType = 0;
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_12 = 2147483647;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const HLF_R: C2Rust_Unnamed_13 = 18;
pub const HLF_N: C2Rust_Unnamed_13 = 12;
pub const HLF_D: C2Rust_Unnamed_13 = 5;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const MAGIC_ALL: magic_T = 4;
pub const MAGIC_ON: magic_T = 3;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_14 = 4096;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_17 = 83;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_17 = 67;
pub const SHM_SEARCH: C2Rust_Unnamed_17 = 115;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_int;
pub const HIST_SEARCH: C2Rust_Unnamed_18 = 1;
pub const VV_SEARCHFORWARD: VimVarIndex = 56;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const FNAME_REL: C2Rust_Unnamed_20 = 16;
pub const FNAME_INCL: C2Rust_Unnamed_20 = 8;
pub const FNAME_EXP: C2Rust_Unnamed_20 = 2;
pub const kMTLineWise: MotionType = 1;
pub const kEqualFiles: file_comparison = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const CHECK_PATH: C2Rust_Unnamed_22 = 3;
pub const FIND_DEFINE: C2Rust_Unnamed_22 = 2;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const ACTION_EXPAND: C2Rust_Unnamed_23 = 5;
pub const ACTION_SHOW_ALL: C2Rust_Unnamed_23 = 4;
pub const ACTION_SPLIT: C2Rust_Unnamed_23 = 3;
pub const ACTION_SHOW: C2Rust_Unnamed_23 = 1;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const SEARCH_COL: C2Rust_Unnamed_24 = 4096;
pub const SEARCH_PEEK: C2Rust_Unnamed_24 = 2048;
pub const SEARCH_KEEP: C2Rust_Unnamed_24 = 1024;
pub const SEARCH_MARK: C2Rust_Unnamed_24 = 512;
pub const SEARCH_START: C2Rust_Unnamed_24 = 256;
pub const SEARCH_NOOF: C2Rust_Unnamed_24 = 128;
pub const SEARCH_END: C2Rust_Unnamed_24 = 64;
pub const SEARCH_HIS: C2Rust_Unnamed_24 = 32;
pub const SEARCH_OPT: C2Rust_Unnamed_24 = 16;
pub const SEARCH_MSG: C2Rust_Unnamed_24 = 12;
pub const SEARCH_ECHO: C2Rust_Unnamed_24 = 2;
pub const SEARCH_REV: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const FM_BLOCKSTOP: C2Rust_Unnamed_25 = 4;
pub const FM_FORWARD: C2Rust_Unnamed_25 = 2;
pub const FM_BACKWARD: C2Rust_Unnamed_25 = 1;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const RE_LAST: C2Rust_Unnamed_26 = 2;
pub const RE_BOTH: C2Rust_Unnamed_26 = 2;
pub const RE_SUBST: C2Rust_Unnamed_26 = 1;
pub const RE_SEARCH: C2Rust_Unnamed_26 = 0;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const SEARCH_STAT_DEF_TIMEOUT: C2Rust_Unnamed_27 = 40;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const SEARCH_STAT_BUF_LEN: C2Rust_Unnamed_28 = 16;
pub const LSIZE: C2Rust_Unnamed_29 = 512;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SearchedFile {
    pub fp: *mut FILE,
    pub name: *mut ::core::ffi::c_char,
    pub lnum: linenr_T,
    pub matched: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CPO_SEARCH: ::core::ffi::c_int = 'c' as ::core::ffi::c_int;
pub const CPO_SHOWMATCH: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const CPO_MATCHBSL: ::core::ffi::c_int = 'M' as ::core::ffi::c_int;
pub const CPO_LINEOFF: ::core::ffi::c_int = 'o' as ::core::ffi::c_int;
pub const CPO_MATCH: ::core::ffi::c_int = '%' as ::core::ffi::c_int;
pub const CPO_SCOLON: ::core::ffi::c_int = ';' as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
static lastc: GlobalCell<[uint8_t; 2]> = GlobalCell::new([NUL as uint8_t, NUL as uint8_t]);
static lastcdir: GlobalCell<Direction> = GlobalCell::new(FORWARD);
static last_t_cmd: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
static lastc_bytes: GlobalCell<[::core::ffi::c_char; 33]> = GlobalCell::new([0; 33]);
static lastc_bytelen: GlobalCell<::core::ffi::c_int> = GlobalCell::new(1 as ::core::ffi::c_int);
unsafe extern "C" fn get_line_and_copy(
    mut lnum: linenr_T,
    mut buf: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut line: *mut ::core::ffi::c_char = ml_get(lnum);
    xstrlcpy(buf, line, LSIZE as ::core::ffi::c_int as size_t);
    return buf;
}
pub unsafe extern "C" fn find_pattern_in_path(
    mut ptr: *mut ::core::ffi::c_char,
    mut dir: Direction,
    mut len: size_t,
    mut whole: bool,
    mut skip_comments: bool,
    mut type_0: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut action: ::core::ffi::c_int,
    mut start_lnum: linenr_T,
    mut end_lnum: linenr_T,
    mut forceit: bool,
    mut silent: bool,
) {
    let mut inc_opt: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut old_files: ::core::ffi::c_int = 0;
    let mut depth: ::core::ffi::c_int = 0;
    let mut lnum: linenr_T = 0;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut c2rust_current_block: u64;
    let mut files: *mut SearchedFile = ::core::ptr::null_mut::<SearchedFile>();
    let mut bigger: *mut SearchedFile = ::core::ptr::null_mut::<SearchedFile>();
    let mut max_path_depth: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
    let mut match_count: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut new_fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut curr_fname: *mut ::core::ffi::c_char = (*curbuf.get()).b_fname;
    let mut prev_fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut depth_displayed: ::core::ffi::c_int = 0;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut define_matched: bool = false;
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut incl_regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut def_regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut matched: bool = false_0 != 0;
    let mut did_show: bool = false_0 != 0;
    let mut found: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut already: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut startp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut curwin_save: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let l_g_do_tagpreview: ::core::ffi::c_int = g_do_tagpreview.get();
    regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
    incl_regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
    def_regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
    let mut file_line: *mut ::core::ffi::c_char =
        xmalloc(LSIZE as ::core::ffi::c_int as size_t) as *mut ::core::ffi::c_char;
    '_fpip_end: {
        if type_0 != CHECK_PATH as ::core::ffi::c_int
            && type_0 != FIND_DEFINE as ::core::ffi::c_int
            && !compl_status_sol()
        {
            let mut patsize: size_t = len.wrapping_add(5 as size_t);
            let mut pat: *mut ::core::ffi::c_char = xmalloc(patsize) as *mut ::core::ffi::c_char;
            '_c2rust_label: {
                if len <= 2147483647 as ::core::ffi::c_int as size_t {
                } else {
                    __assert_fail(
                        b"len <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/search.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        2966 as ::core::ffi::c_uint,
                        b"void find_pattern_in_path(char *, Direction, size_t, _Bool, _Bool, int, int, int, linenr_T, linenr_T, _Bool, _Bool)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            snprintf(
                pat,
                patsize,
                if whole as ::core::ffi::c_int != 0 {
                    b"\\<%.*s\\>\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"%.*s\0".as_ptr() as *const ::core::ffi::c_char
                },
                len as ::core::ffi::c_int,
                ptr,
            );
            regmatch.rm_ic = ignorecase(pat) != 0;
            regmatch.regprog = vim_regcomp(
                pat,
                if magic_isset() as ::core::ffi::c_int != 0 {
                    RE_MAGIC
                } else {
                    0 as ::core::ffi::c_int
                },
            );
            xfree(pat as *mut ::core::ffi::c_void);
            if regmatch.regprog.is_null() {
                break '_fpip_end;
            }
        }
        inc_opt = if *(*curbuf.get()).b_p_inc as ::core::ffi::c_int == NUL {
            p_inc.get()
        } else {
            (*curbuf.get()).b_p_inc
        };
        if *inc_opt as ::core::ffi::c_int != NUL {
            incl_regmatch.regprog = vim_regcomp(
                inc_opt,
                if magic_isset() as ::core::ffi::c_int != 0 {
                    RE_MAGIC
                } else {
                    0 as ::core::ffi::c_int
                },
            );
            if incl_regmatch.regprog.is_null() {
                break '_fpip_end;
            } else {
                incl_regmatch.rm_ic = false_0 != 0;
            }
        }
        if type_0 == FIND_DEFINE as ::core::ffi::c_int
            && (*(*curbuf.get()).b_p_def as ::core::ffi::c_int != NUL
                || *p_def.get() as ::core::ffi::c_int != NUL)
        {
            def_regmatch.regprog = vim_regcomp(
                if *(*curbuf.get()).b_p_def as ::core::ffi::c_int == NUL {
                    p_def.get()
                } else {
                    (*curbuf.get()).b_p_def
                },
                if magic_isset() as ::core::ffi::c_int != 0 {
                    RE_MAGIC
                } else {
                    0 as ::core::ffi::c_int
                },
            );
            if def_regmatch.regprog.is_null() {
                break '_fpip_end;
            } else {
                def_regmatch.rm_ic = false_0 != 0;
            }
        }
        files = xcalloc(
            max_path_depth as size_t,
            ::core::mem::size_of::<SearchedFile>(),
        ) as *mut SearchedFile;
        old_files = max_path_depth;
        depth_displayed = -1 as ::core::ffi::c_int;
        depth = depth_displayed;
        end_lnum = if end_lnum < (*curbuf.get()).b_ml.ml_line_count {
            end_lnum
        } else {
            (*curbuf.get()).b_ml.ml_line_count
        };
        lnum = if start_lnum < end_lnum {
            start_lnum
        } else {
            end_lnum
        };
        line = get_line_and_copy(lnum, file_line);
        's_1511: loop {
            if !incl_regmatch.regprog.is_null()
                && vim_regexec(&raw mut incl_regmatch, line, 0 as colnr_T) as ::core::ffi::c_int
                    != 0
            {
                let mut p_fname: *mut ::core::ffi::c_char = if curr_fname == (*curbuf.get()).b_fname
                {
                    (*curbuf.get()).b_ffname
                } else {
                    curr_fname
                };
                if !strstr(inc_opt, b"\\zs\0".as_ptr() as *const ::core::ffi::c_char).is_null() {
                    new_fname = find_file_name_in_path(
                        incl_regmatch.startp[0 as ::core::ffi::c_int as usize],
                        incl_regmatch.endp[0 as ::core::ffi::c_int as usize]
                            .offset_from(incl_regmatch.startp[0 as ::core::ffi::c_int as usize])
                            as size_t,
                        FNAME_EXP as ::core::ffi::c_int
                            | FNAME_INCL as ::core::ffi::c_int
                            | FNAME_REL as ::core::ffi::c_int,
                        1 as ::core::ffi::c_long,
                        p_fname,
                    );
                } else {
                    new_fname = file_name_in_line(
                        incl_regmatch.endp[0 as ::core::ffi::c_int as usize],
                        0 as ::core::ffi::c_int,
                        FNAME_EXP as ::core::ffi::c_int
                            | FNAME_INCL as ::core::ffi::c_int
                            | FNAME_REL as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                        p_fname,
                        ::core::ptr::null_mut::<linenr_T>(),
                    );
                }
                let mut already_searched: bool = false_0 != 0;
                if !new_fname.is_null() {
                    i = 0 as ::core::ffi::c_int;
                    loop {
                        if i == depth + 1 as ::core::ffi::c_int {
                            i = old_files;
                        }
                        if i == max_path_depth {
                            break;
                        }
                        if path_full_compare(
                            new_fname,
                            (*files.offset(i as isize)).name,
                            true_0 != 0,
                            true_0 != 0,
                        ) as ::core::ffi::c_uint
                            & kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
                            != 0
                        {
                            if type_0 != CHECK_PATH as ::core::ffi::c_int
                                && action == ACTION_SHOW_ALL as ::core::ffi::c_int
                                && (*files.offset(i as isize)).matched != 0
                            {
                                msg_putchar('\n' as ::core::ffi::c_int);
                                if !got_int.get() {
                                    msg_home_replace(new_fname);
                                    msg_puts(gettext(
                                        b" (includes previously listed match)\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                    ));
                                    prev_fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                }
                            }
                            let mut ptr_: *mut *mut ::core::ffi::c_void =
                                &raw mut new_fname as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr_);
                            *ptr_ = NULL_0;
                            let _ = *ptr_;
                            already_searched = true_0 != 0;
                            break;
                        } else {
                            i += 1;
                        }
                    }
                }
                if type_0 == CHECK_PATH as ::core::ffi::c_int
                    && (action == ACTION_SHOW_ALL as ::core::ffi::c_int
                        || new_fname.is_null() && !already_searched)
                {
                    if did_show {
                        msg_putchar('\n' as ::core::ffi::c_int);
                    } else {
                        gotocmdline(true_0 != 0);
                        msg_puts_title(gettext(
                            b"--- Included files \0".as_ptr() as *const ::core::ffi::c_char
                        ));
                        if action != ACTION_SHOW_ALL as ::core::ffi::c_int {
                            msg_puts_title(gettext(
                                b"not found \0".as_ptr() as *const ::core::ffi::c_char
                            ));
                        }
                        msg_puts_title(gettext(
                            b"in path ---\n\0".as_ptr() as *const ::core::ffi::c_char
                        ));
                    }
                    did_show = true_0 != 0;
                    while depth_displayed < depth && !got_int.get() {
                        depth_displayed += 1;
                        i = 0 as ::core::ffi::c_int;
                        while i < depth_displayed {
                            msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
                            i += 1;
                        }
                        msg_home_replace((*files.offset(depth_displayed as isize)).name);
                        msg_puts(b" -->\n\0".as_ptr() as *const ::core::ffi::c_char);
                    }
                    if !got_int.get() {
                        i = 0 as ::core::ffi::c_int;
                        while i <= depth_displayed {
                            msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
                            i += 1;
                        }
                        if !new_fname.is_null() {
                            msg_outtrans(new_fname, HLF_D as ::core::ffi::c_int, false_0 != 0);
                        } else {
                            if !strstr(inc_opt, b"\\zs\0".as_ptr() as *const ::core::ffi::c_char)
                                .is_null()
                            {
                                p = incl_regmatch.startp[0 as ::core::ffi::c_int as usize];
                                i = incl_regmatch.endp[0 as ::core::ffi::c_int as usize]
                                    .offset_from(
                                        incl_regmatch.startp[0 as ::core::ffi::c_int as usize],
                                    ) as ::core::ffi::c_int;
                            } else {
                                p = incl_regmatch.endp[0 as ::core::ffi::c_int as usize];
                                while *p as ::core::ffi::c_int != 0
                                    && !vim_isfilec(*p as uint8_t as ::core::ffi::c_int)
                                {
                                    p = p.offset(1);
                                }
                                i = 0 as ::core::ffi::c_int;
                                while vim_isfilec(
                                    *p.offset(i as isize) as uint8_t as ::core::ffi::c_int
                                ) {
                                    i += 1;
                                }
                            }
                            if i == 0 as ::core::ffi::c_int {
                                p = incl_regmatch.endp[0 as ::core::ffi::c_int as usize];
                                i = strlen(p) as ::core::ffi::c_int;
                            } else if p > line {
                                if *p.offset(-1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == '"' as ::core::ffi::c_int
                                    || *p.offset(-1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == '<' as ::core::ffi::c_int
                                {
                                    p = p.offset(-1);
                                    i += 1;
                                }
                                if *p.offset(i as isize) as ::core::ffi::c_int
                                    == '"' as ::core::ffi::c_int
                                    || *p.offset(i as isize) as ::core::ffi::c_int
                                        == '>' as ::core::ffi::c_int
                                {
                                    i += 1;
                                }
                            }
                            let mut save_char: ::core::ffi::c_char = *p.offset(i as isize);
                            *p.offset(i as isize) = NUL as ::core::ffi::c_char;
                            msg_outtrans(p, HLF_D as ::core::ffi::c_int, false_0 != 0);
                            *p.offset(i as isize) = save_char;
                        }
                        if new_fname.is_null() && action == ACTION_SHOW_ALL as ::core::ffi::c_int {
                            if already_searched {
                                msg_puts(gettext(
                                    b"  (Already listed)\0".as_ptr() as *const ::core::ffi::c_char
                                ));
                            } else {
                                msg_puts(gettext(
                                    b"  NOT FOUND\0".as_ptr() as *const ::core::ffi::c_char
                                ));
                            }
                        }
                    }
                }
                if !new_fname.is_null() {
                    if depth + 1 as ::core::ffi::c_int == old_files {
                        bigger = xmalloc(
                            (max_path_depth as size_t)
                                .wrapping_mul(2 as size_t)
                                .wrapping_mul(::core::mem::size_of::<SearchedFile>()),
                        ) as *mut SearchedFile;
                        i = 0 as ::core::ffi::c_int;
                        while i <= depth {
                            *bigger.offset(i as isize) = *files.offset(i as isize);
                            i += 1;
                        }
                        i = depth + 1 as ::core::ffi::c_int;
                        while i < old_files + max_path_depth {
                            (*bigger.offset(i as isize)).fp = ::core::ptr::null_mut::<FILE>();
                            (*bigger.offset(i as isize)).name =
                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                            (*bigger.offset(i as isize)).lnum = 0 as ::core::ffi::c_int as linenr_T;
                            (*bigger.offset(i as isize)).matched = false_0;
                            i += 1;
                        }
                        i = old_files;
                        while i < max_path_depth {
                            *bigger.offset((i + max_path_depth) as isize) =
                                *files.offset(i as isize);
                            i += 1;
                        }
                        old_files += max_path_depth;
                        max_path_depth *= 2 as ::core::ffi::c_int;
                        xfree(files as *mut ::core::ffi::c_void);
                        files = bigger;
                    }
                    (*files.offset((depth + 1 as ::core::ffi::c_int) as isize)).fp =
                        os_fopen(new_fname, b"r\0".as_ptr() as *const ::core::ffi::c_char);
                    if (*files.offset((depth + 1 as ::core::ffi::c_int) as isize))
                        .fp
                        .is_null()
                    {
                        xfree(new_fname as *mut ::core::ffi::c_void);
                    } else {
                        depth += 1;
                        if depth == old_files {
                            xfree(
                                (*files.offset(old_files as isize)).name
                                    as *mut ::core::ffi::c_void,
                            );
                            old_files += 1;
                        }
                        curr_fname = new_fname;
                        (*files.offset(depth as isize)).name = curr_fname;
                        (*files.offset(depth as isize)).lnum = 0 as ::core::ffi::c_int as linenr_T;
                        (*files.offset(depth as isize)).matched = false_0;
                        if action == ACTION_EXPAND as ::core::ffi::c_int
                            && !shortmess(SHM_COMPLETIONSCAN as ::core::ffi::c_int)
                            && !silent
                        {
                            msg_hist_off.set(true_0 != 0);
                            vim_snprintf(
                                IObuff.ptr() as *mut ::core::ffi::c_char,
                                IOSIZE as size_t,
                                gettext(b"Scanning included file: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                new_fname,
                            );
                            msg_trunc(
                                IObuff.ptr() as *mut ::core::ffi::c_char,
                                true_0 != 0,
                                HLF_R as ::core::ffi::c_int,
                            );
                        } else if p_verbose.get() >= 5 as OptInt {
                            verbose_enter();
                            smsg(
                                0 as ::core::ffi::c_int,
                                gettext(b"Searching included file %s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                new_fname,
                            );
                            verbose_leave();
                        }
                    }
                }
                c2rust_current_block = 9985465603744958559;
            } else {
                p = line;
                c2rust_current_block = 2704434626355109080;
            }
            loop {
                match c2rust_current_block {
                    2704434626355109080 => {
                        define_matched = false_0 != 0;
                        if !def_regmatch.regprog.is_null()
                            && vim_regexec(&raw mut def_regmatch, line, 0 as colnr_T)
                                as ::core::ffi::c_int
                                != 0
                        {
                            p = def_regmatch.endp[0 as ::core::ffi::c_int as usize];
                            while *p as ::core::ffi::c_int != 0
                                && !vim_iswordc(*p as uint8_t as ::core::ffi::c_int)
                            {
                                p = p.offset(1);
                            }
                            define_matched = true_0 != 0;
                        }
                        if def_regmatch.regprog.is_null()
                            || define_matched as ::core::ffi::c_int != 0
                        {
                            if define_matched as ::core::ffi::c_int != 0
                                || compl_status_sol() as ::core::ffi::c_int != 0
                            {
                                startp = skipwhite(p);
                                if p_ic.get() != 0 {
                                    matched = mb_strnicmp(startp, ptr, len) == 0;
                                } else {
                                    matched = strncmp(startp, ptr, len) == 0;
                                }
                                if matched as ::core::ffi::c_int != 0
                                    && define_matched as ::core::ffi::c_int != 0
                                    && whole as ::core::ffi::c_int != 0
                                    && vim_iswordc(*startp.offset(len as isize) as uint8_t
                                        as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                        != 0
                                {
                                    matched = false_0 != 0;
                                }
                            } else if !regmatch.regprog.is_null()
                                && vim_regexec(
                                    &raw mut regmatch,
                                    line,
                                    p.offset_from(line) as colnr_T,
                                ) as ::core::ffi::c_int
                                    != 0
                            {
                                matched = true_0 != 0;
                                startp = regmatch.startp[0 as ::core::ffi::c_int as usize];
                                if skip_comments {
                                    if (*line as ::core::ffi::c_int != '#' as ::core::ffi::c_int
                                        || strncmp(
                                            skipwhite(
                                                line.offset(1 as ::core::ffi::c_int as isize),
                                            ),
                                            b"define\0".as_ptr() as *const ::core::ffi::c_char,
                                            6 as size_t,
                                        ) != 0 as ::core::ffi::c_int)
                                        && get_leader_len(
                                            line,
                                            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                                            false_0 != 0,
                                            true_0 != 0,
                                        ) != 0
                                    {
                                        matched = false_0 != 0;
                                    }
                                    p = skipwhite(line);
                                    if matched as ::core::ffi::c_int != 0
                                        || *p.offset(0 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '/' as ::core::ffi::c_int
                                            && *p.offset(1 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                == '*' as ::core::ffi::c_int
                                        || *p.offset(0 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '*' as ::core::ffi::c_int
                                    {
                                        p = line;
                                        while *p as ::core::ffi::c_int != 0 && p < startp {
                                            if matched as ::core::ffi::c_int != 0
                                                && *p.offset(0 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_int
                                                    == '/' as ::core::ffi::c_int
                                                && (*p.offset(1 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_int
                                                    == '*' as ::core::ffi::c_int
                                                    || *p.offset(1 as ::core::ffi::c_int as isize)
                                                        as ::core::ffi::c_int
                                                        == '/' as ::core::ffi::c_int)
                                            {
                                                matched = false_0 != 0;
                                                if *p.offset(1 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_int
                                                    == '/' as ::core::ffi::c_int
                                                {
                                                    break;
                                                }
                                                p = p.offset(1);
                                            } else if !matched
                                                && *p.offset(0 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_int
                                                    == '*' as ::core::ffi::c_int
                                                && *p.offset(1 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_int
                                                    == '/' as ::core::ffi::c_int
                                            {
                                                matched = true_0 != 0;
                                                p = p.offset(1);
                                            }
                                            p = p.offset(1);
                                        }
                                    }
                                }
                            }
                        }
                        c2rust_current_block = 9985465603744958559;
                    }
                    _ => {
                        if !matched {
                            break;
                        }
                        '_exit_matched: {
                            if action == ACTION_EXPAND as ::core::ffi::c_int {
                                let mut cont_s_ipos: bool = false_0 != 0;
                                if depth == -1 as ::core::ffi::c_int
                                    && lnum == (*curwin.get()).w_cursor.lnum
                                {
                                    break 's_1511;
                                }
                                found = true_0 != 0;
                                p = startp;
                                let mut aux: *mut ::core::ffi::c_char = p;
                                if compl_status_adding() as ::core::ffi::c_int != 0
                                    && strlen(p) as ::core::ffi::c_int >= ins_compl_len()
                                {
                                    p = p.offset(ins_compl_len() as isize);
                                    if vim_iswordp(p) {
                                        break '_exit_matched;
                                    } else {
                                        p = find_word_start(p);
                                    }
                                }
                                p = find_word_end(p);
                                i = p.offset_from(aux) as ::core::ffi::c_int;
                                if compl_status_adding() as ::core::ffi::c_int != 0
                                    && i == ins_compl_len()
                                {
                                    strncpy(
                                        IObuff.ptr() as *mut ::core::ffi::c_char,
                                        aux,
                                        i as size_t,
                                    );
                                    if depth < 0 as ::core::ffi::c_int {
                                        if lnum >= end_lnum {
                                            break '_exit_matched;
                                        } else {
                                            lnum += 1;
                                            line = get_line_and_copy(lnum, file_line);
                                        }
                                    } else {
                                        line = file_line;
                                        if vim_fgets(
                                            line,
                                            LSIZE as ::core::ffi::c_int,
                                            (*files.offset(depth as isize)).fp,
                                        ) {
                                            break '_exit_matched;
                                        }
                                    }
                                    p = skipwhite(line);
                                    aux = p;
                                    already = aux;
                                    p = find_word_start(p);
                                    p = find_word_end(p);
                                    if p > aux {
                                        if *aux as ::core::ffi::c_int != ')' as ::core::ffi::c_int
                                            && (*IObuff.ptr())
                                                [(i - 1 as ::core::ffi::c_int) as usize]
                                                as ::core::ffi::c_int
                                                != TAB
                                        {
                                            if (*IObuff.ptr())
                                                [(i - 1 as ::core::ffi::c_int) as usize]
                                                as ::core::ffi::c_int
                                                != ' ' as ::core::ffi::c_int
                                            {
                                                let c2rust_fresh9 = i;
                                                i = i + 1;
                                                (*IObuff.ptr())[c2rust_fresh9 as usize] =
                                                    ' ' as ::core::ffi::c_char;
                                            }
                                            if p_js.get() != 0
                                                && ((*IObuff.ptr())
                                                    [(i - 2 as ::core::ffi::c_int) as usize]
                                                    as ::core::ffi::c_int
                                                    == '.' as ::core::ffi::c_int
                                                    || (*IObuff.ptr())
                                                        [(i - 2 as ::core::ffi::c_int) as usize]
                                                        as ::core::ffi::c_int
                                                        == '?' as ::core::ffi::c_int
                                                    || (*IObuff.ptr())
                                                        [(i - 2 as ::core::ffi::c_int) as usize]
                                                        as ::core::ffi::c_int
                                                        == '!' as ::core::ffi::c_int)
                                            {
                                                let c2rust_fresh10 = i;
                                                i = i + 1;
                                                (*IObuff.ptr())[c2rust_fresh10 as usize] =
                                                    ' ' as ::core::ffi::c_char;
                                            }
                                        }
                                        if p.offset_from(aux) >= (IOSIZE - i) as isize {
                                            p = aux
                                                .offset(IOSIZE as isize)
                                                .offset(-(i as isize))
                                                .offset(-(1 as ::core::ffi::c_int as isize));
                                        }
                                        strncpy(
                                            (IObuff.ptr() as *mut ::core::ffi::c_char)
                                                .offset(i as isize),
                                            aux,
                                            p.offset_from(aux) as size_t,
                                        );
                                        i += p.offset_from(aux) as ::core::ffi::c_int;
                                        cont_s_ipos = true_0 != 0;
                                    }
                                    (*IObuff.ptr())[i as usize] = NUL as ::core::ffi::c_char;
                                    aux = IObuff.ptr() as *mut ::core::ffi::c_char;
                                    if i == ins_compl_len() {
                                        break '_exit_matched;
                                    }
                                }
                                let add_r: ::core::ffi::c_int = ins_compl_add_infercase(
                                    aux,
                                    i,
                                    p_ic.get() != 0,
                                    if curr_fname == (*curbuf.get()).b_fname {
                                        ::core::ptr::null_mut::<::core::ffi::c_char>()
                                    } else {
                                        curr_fname
                                    },
                                    dir,
                                    cont_s_ipos,
                                    0 as ::core::ffi::c_int,
                                );
                                if add_r == OK {
                                    dir = FORWARD;
                                } else if add_r == FAIL {
                                    break 's_1511;
                                }
                            } else if action == ACTION_SHOW_ALL as ::core::ffi::c_int {
                                found = true_0 != 0;
                                if !did_show {
                                    gotocmdline(true_0 != 0);
                                }
                                if curr_fname != prev_fname {
                                    if did_show {
                                        msg_putchar('\n' as ::core::ffi::c_int);
                                    }
                                    if !got_int.get() {
                                        msg_home_replace(curr_fname);
                                    }
                                    prev_fname = curr_fname;
                                }
                                did_show = true_0 != 0;
                                if !got_int.get() {
                                    let c2rust_fresh11 = match_count;
                                    match_count = match_count + 1;
                                    show_pat_in_path(
                                        line,
                                        type_0,
                                        true_0 != 0,
                                        action,
                                        if depth == -1 as ::core::ffi::c_int {
                                            ::core::ptr::null_mut::<FILE>()
                                        } else {
                                            (*files.offset(depth as isize)).fp
                                        },
                                        if depth == -1 as ::core::ffi::c_int {
                                            &raw mut lnum
                                        } else {
                                            &raw mut (*files.offset(depth as isize)).lnum
                                        },
                                        c2rust_fresh11,
                                    );
                                }
                                i = 0 as ::core::ffi::c_int;
                                while i <= depth {
                                    (*files.offset(i as isize)).matched = true_0;
                                    i += 1;
                                }
                            } else {
                                count -= 1;
                                if count <= 0 as ::core::ffi::c_int {
                                    found = true_0 != 0;
                                    if depth == -1 as ::core::ffi::c_int
                                        && lnum == (*curwin.get()).w_cursor.lnum
                                        && l_g_do_tagpreview == 0 as ::core::ffi::c_int
                                    {
                                        emsg(gettext(b"E387: Match is on current line\0".as_ptr()
                                            as *const ::core::ffi::c_char));
                                    } else if action == ACTION_SHOW as ::core::ffi::c_int {
                                        show_pat_in_path(
                                            line,
                                            type_0,
                                            did_show,
                                            action,
                                            if depth == -1 as ::core::ffi::c_int {
                                                ::core::ptr::null_mut::<FILE>()
                                            } else {
                                                (*files.offset(depth as isize)).fp
                                            },
                                            if depth == -1 as ::core::ffi::c_int {
                                                &raw mut lnum
                                            } else {
                                                &raw mut (*files.offset(depth as isize)).lnum
                                            },
                                            1 as ::core::ffi::c_int,
                                        );
                                        did_show = true_0 != 0;
                                    } else {
                                        if l_g_do_tagpreview != 0 as ::core::ffi::c_int {
                                            curwin_save = curwin.get();
                                            prepare_tagpreview(true_0 != 0);
                                        }
                                        if action == ACTION_SPLIT as ::core::ffi::c_int {
                                            if win_split(
                                                0 as ::core::ffi::c_int,
                                                0 as ::core::ffi::c_int,
                                            ) == FAIL
                                            {
                                                break 's_1511;
                                            }
                                            (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
                                            (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
                                        }
                                        if depth == -1 as ::core::ffi::c_int {
                                            if l_g_do_tagpreview != 0 as ::core::ffi::c_int {
                                                if !win_valid(curwin_save) {
                                                    break 's_1511;
                                                }
                                                if !(getfile(
                                                    (*(*curwin_save).w_buffer).handle
                                                        as ::core::ffi::c_int,
                                                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                                    true,
                                                    lnum,
                                                    forceit,
                                                ) <= 0 as ::core::ffi::c_int)
                                                {
                                                    break 's_1511;
                                                }
                                            } else {
                                                setpcmark();
                                            }
                                            (*curwin.get()).w_cursor.lnum = lnum;
                                            check_cursor(curwin.get());
                                        } else {
                                            if !(getfile(
                                                0 as ::core::ffi::c_int,
                                                (*files.offset(depth as isize)).name,
                                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                                true,
                                                (*files.offset(depth as isize)).lnum,
                                                forceit,
                                            ) <= 0 as ::core::ffi::c_int)
                                            {
                                                break 's_1511;
                                            }
                                            (*curwin.get()).w_cursor.lnum =
                                                (*files.offset(depth as isize)).lnum;
                                        }
                                    }
                                    if action != ACTION_SHOW as ::core::ffi::c_int {
                                        (*curwin.get()).w_cursor.col =
                                            startp.offset_from(line) as colnr_T;
                                        (*curwin.get()).w_set_curswant = true_0;
                                    }
                                    if l_g_do_tagpreview != 0 as ::core::ffi::c_int
                                        && curwin.get() != curwin_save
                                        && win_valid(curwin_save) as ::core::ffi::c_int != 0
                                    {
                                        validate_cursor(curwin.get());
                                        redraw_later(curwin.get(), UPD_VALID);
                                        win_enter(curwin_save, true_0 != 0);
                                    }
                                    break 's_1511;
                                }
                            }
                        }
                        matched = false_0 != 0;
                        if def_regmatch.regprog.is_null()
                            && action == ACTION_EXPAND as ::core::ffi::c_int
                            && !compl_status_sol()
                            && *startp as ::core::ffi::c_int != NUL
                            && *startp.offset(utfc_ptr2len(startp) as isize) as ::core::ffi::c_int
                                != NUL
                        {
                            c2rust_current_block = 2704434626355109080;
                        } else {
                            break;
                        }
                    }
                }
            }
            line_breakcheck();
            if action == ACTION_EXPAND as ::core::ffi::c_int {
                ins_compl_check_keys(30 as ::core::ffi::c_int, false_0 != 0);
            }
            if got_int.get() as ::core::ffi::c_int != 0
                || ins_compl_interrupted() as ::core::ffi::c_int != 0
            {
                break;
            }
            while depth >= 0 as ::core::ffi::c_int && already.is_null() && {
                line = file_line;
                vim_fgets(
                    line,
                    LSIZE as ::core::ffi::c_int,
                    (*files.offset(depth as isize)).fp,
                ) as ::core::ffi::c_int
                    != 0
            } {
                fclose((*files.offset(depth as isize)).fp);
                old_files -= 1;
                (*files.offset(old_files as isize)).name = (*files.offset(depth as isize)).name;
                (*files.offset(old_files as isize)).matched =
                    (*files.offset(depth as isize)).matched;
                depth -= 1;
                curr_fname = if depth == -1 as ::core::ffi::c_int {
                    (*curbuf.get()).b_fname
                } else {
                    (*files.offset(depth as isize)).name
                };
                depth_displayed = if depth_displayed < depth {
                    depth_displayed
                } else {
                    depth
                };
            }
            if depth >= 0 as ::core::ffi::c_int {
                (*files.offset(depth as isize)).lnum += 1;
                i = strlen(line) as ::core::ffi::c_int;
                if i > 0 as ::core::ffi::c_int
                    && *line.offset((i - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                        == '\n' as ::core::ffi::c_int
                {
                    i -= 1;
                    *line.offset(i as isize) = NUL as ::core::ffi::c_char;
                }
                if i > 0 as ::core::ffi::c_int
                    && *line.offset((i - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                        == '\r' as ::core::ffi::c_int
                {
                    i -= 1;
                    *line.offset(i as isize) = NUL as ::core::ffi::c_char;
                }
            } else if already.is_null() {
                lnum += 1;
                if lnum > end_lnum {
                    break;
                }
                line = get_line_and_copy(lnum, file_line);
            }
            already = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        i = 0 as ::core::ffi::c_int;
        while i <= depth {
            fclose((*files.offset(i as isize)).fp);
            xfree((*files.offset(i as isize)).name as *mut ::core::ffi::c_void);
            i += 1;
        }
        i = old_files;
        while i < max_path_depth {
            xfree((*files.offset(i as isize)).name as *mut ::core::ffi::c_void);
            i += 1;
        }
        xfree(files as *mut ::core::ffi::c_void);
        if type_0 == CHECK_PATH as ::core::ffi::c_int {
            if !did_show {
                if action != ACTION_SHOW_ALL as ::core::ffi::c_int {
                    msg(
                        gettext(b"All included files were found\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        0 as ::core::ffi::c_int,
                    );
                } else {
                    msg(
                        gettext(b"No included files\0".as_ptr() as *const ::core::ffi::c_char),
                        0 as ::core::ffi::c_int,
                    );
                }
            }
        } else if !found && action != ACTION_EXPAND as ::core::ffi::c_int && !silent {
            if got_int.get() as ::core::ffi::c_int != 0
                || ins_compl_interrupted() as ::core::ffi::c_int != 0
            {
                emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
            } else if type_0 == FIND_DEFINE as ::core::ffi::c_int {
                emsg(gettext(
                    b"E388: Couldn't find definition\0".as_ptr() as *const ::core::ffi::c_char
                ));
            } else {
                emsg(gettext(
                    b"E389: Couldn't find pattern\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
        }
        if action == ACTION_SHOW as ::core::ffi::c_int
            || action == ACTION_SHOW_ALL as ::core::ffi::c_int
        {
            msg_end();
        }
    }
    xfree(file_line as *mut ::core::ffi::c_void);
    vim_regfree(regmatch.regprog);
    vim_regfree(incl_regmatch.regprog);
    vim_regfree(def_regmatch.regprog);
}
unsafe extern "C" fn show_pat_in_path(
    mut line: *mut ::core::ffi::c_char,
    mut type_0: ::core::ffi::c_int,
    mut did_show: bool,
    mut action: ::core::ffi::c_int,
    mut fp: *mut FILE,
    mut lnum: *mut linenr_T,
    mut count: ::core::ffi::c_int,
) {
    if did_show {
        msg_putchar('\n' as ::core::ffi::c_int);
    } else if msg_silent.get() == 0 {
        gotocmdline(true_0 != 0);
    }
    if got_int.get() {
        return;
    }
    let mut linelen: size_t = strlen(line);
    loop {
        let mut p: *mut ::core::ffi::c_char = line
            .offset(linelen as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
        if !fp.is_null() {
            if p >= line && *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                p = p.offset(-1);
            }
            if p >= line && *p as ::core::ffi::c_int == '\r' as ::core::ffi::c_int {
                p = p.offset(-1);
            }
            *p.offset(1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        }
        if action == ACTION_SHOW_ALL as ::core::ffi::c_int {
            snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b"%3d: \0".as_ptr() as *const ::core::ffi::c_char,
                count,
            );
            msg_puts(IObuff.ptr() as *mut ::core::ffi::c_char);
            snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b"%4d\0".as_ptr() as *const ::core::ffi::c_char,
                *lnum,
            );
            msg_puts_hl(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                HLF_N as ::core::ffi::c_int,
                false_0 != 0,
            );
            msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        }
        msg_prt_line(line, false_0 != 0);
        if got_int.get() as ::core::ffi::c_int != 0
            || type_0 != FIND_DEFINE as ::core::ffi::c_int
            || p < line
            || *p as ::core::ffi::c_int != '\\' as ::core::ffi::c_int
        {
            break;
        }
        if !fp.is_null() {
            if vim_fgets(line, LSIZE as ::core::ffi::c_int, fp) {
                break;
            }
            linelen = strlen(line);
            *lnum += 1;
        } else {
            *lnum += 1;
            if *lnum > (*curbuf.get()).b_ml.ml_line_count {
                break;
            }
            line = ml_get(*lnum);
            linelen = ml_get_len(*lnum) as size_t;
        }
        msg_putchar('\n' as ::core::ffi::c_int);
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
