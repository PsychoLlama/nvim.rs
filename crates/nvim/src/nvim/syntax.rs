use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::autocmd::{EVENT_SYNTAX, apply_autocmds};
use crate::src::nvim::buffer::buf_get_changedtick;
use crate::src::nvim::charset::{
    buf_init_chartab, getdigits_int, getdigits_int32, skiptowhite, skipwhite, str_foldcase,
    vim_isprintc, vim_iswordp_buf,
};
use crate::src::nvim::drawscreen::{
    UPD_NOT_VALID, UPD_SOME_VALID, redraw_curbuf_later, redraw_later,
};
use crate::src::nvim::eval::vars::{do_unlet, get_var_value, set_internal_string_var};
use crate::src::nvim::ex_docmd::{
    check_nextcmd, do_cmdline_cmd, ends_excmd, expand_filename, find_nextcmd, separate_nextcmd,
};
use crate::src::nvim::fold::{foldUpdateAll, foldmethodIsSyntax};
use crate::src::nvim::garray::{ga_append_via_ptr, ga_clear, ga_grow, ga_init, ga_set_growsize};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::hashtab::{
    hash_add_item, hash_clear, hash_find, hash_hash, hash_init, hash_lock, hash_lookup,
    hash_remove, hash_unlock,
};
use crate::src::nvim::highlight_group::{
    HLF_D, highlight_group_name, highlight_link_id, highlight_num_groups, init_highlight,
    syn_check_group, syn_id2attr, syn_list_header, syn_name2id, syn_name2id_len,
};
use crate::src::nvim::indent_c::find_start_comment;
use crate::src::nvim::main::{
    Columns, Rows, curbuf, curtab, curwin, display_tick, e_invarg2, e_nogroup, e_notopen,
    empty_string_option, emsg_skip, firstwin, got_int, include_default, include_link, include_none,
    msg_col, p_cpo, re_extmatch_in, re_extmatch_out, reg_do_extmatch,
};
use crate::src::nvim::mbyte::{mb_strcmp_ic, utf_head_off, utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memline::{ml_get, ml_get_buf, ml_get_buf_len, ml_get_len};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xmemcpyz, xstrdup};
use crate::src::nvim::message::{
    emsg, msg, msg_advance, msg_ext_set_kind, msg_outnum, msg_outtrans, msg_outtrans_len,
    msg_putchar, msg_puts, msg_puts_hl, msg_puts_title, semsg,
};
use crate::src::nvim::optionstr::clear_string_option;
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, gettext, memcpy, memmove, memset, qsort, strcasecmp, strcat, strchr, strcmp,
    strcpy, strlen, strncasecmp, strncmp, strpbrk,
};
use crate::src::nvim::path::path_is_absolute;
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::profile::{
    profile_add, profile_cmp, profile_divide, profile_end, profile_msg, profile_start, profile_zero,
};
use crate::src::nvim::regexp::vim_regexec_multi;
use crate::src::nvim::regexp::{ref_extmatch, skip_regexp, unref_extmatch, vim_regcomp_had_eol};
use crate::src::nvim::runtime::{do_source, source_runtime};
use crate::src::nvim::strings::{
    vim_snprintf, vim_strchr, vim_strnsave_up, vim_strsave_up, xstrnsave,
};
use crate::src::nvim::types::{
    CMD_index, OptInt, buf_T, bufstate_T, cmd_addr_T, colnr_T, cstack_T, disptick_T, exarg_T,
    expand_T, garray_T, hash_T, hashitem_T, hashtab_T, int16_t, int32_t, keyvalue_T, linenr_T,
    lpos_T, pos_T, proftime_T, reg_extmatch_T, regmatch_T, regmmatch_T, regprog_T, size_t,
    syn_time_T, synblock_T, synstate_T, uint8_t, uint32_t, uint64_t, varnumber_T, win_T,
};

mod flags;
pub use self::flags::*;

unsafe extern "C" {
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_13 = 2147483647;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const EXPAND_BUF_LEN: C2Rust_Unnamed_15 = 256;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_16 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_16 = 12;
pub const EXPAND_NOTHING: C2Rust_Unnamed_16 = 0;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const NSUBEXP: C2Rust_Unnamed_17 = 10;
pub const CMD_append: CMD_index = 0;
pub const ADDR_LINES: cmd_addr_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sp_syn {
    pub inc_tag: ::core::ffi::c_int,
    pub id: int16_t,
    pub cont_in_list: *mut int16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct keyentry {
    pub ke_next: *mut keyentry_T,
    pub k_syn: sp_syn,
    pub next_list: *mut int16_t,
    pub flags: ::core::ffi::c_int,
    pub k_char: ::core::ffi::c_int,
    pub keyword: [::core::ffi::c_char; 0],
}
pub type keyentry_T = keyentry;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const MAX_HL_ID: C2Rust_Unnamed_20 = 20000;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const DOSO_NONE: C2Rust_Unnamed_21 = 0;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const DIP_ALL: C2Rust_Unnamed_22 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stateitem_T {
    pub si_idx: ::core::ffi::c_int,
    pub si_id: ::core::ffi::c_int,
    pub si_trans_id: ::core::ffi::c_int,
    pub si_m_lnum: ::core::ffi::c_int,
    pub si_m_startcol: ::core::ffi::c_int,
    pub si_m_endpos: lpos_T,
    pub si_h_startpos: lpos_T,
    pub si_h_endpos: lpos_T,
    pub si_eoe_pos: lpos_T,
    pub si_end_idx: ::core::ffi::c_int,
    pub si_ends: ::core::ffi::c_int,
    pub si_attr: ::core::ffi::c_int,
    pub si_flags: ::core::ffi::c_int,
    pub si_seqnr: ::core::ffi::c_int,
    pub si_cchar: ::core::ffi::c_int,
    pub si_cont_list: *mut int16_t,
    pub si_next_list: *mut int16_t,
    pub si_extmatch: *mut reg_extmatch_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct synpat_T {
    pub sp_type: ::core::ffi::c_char,
    pub sp_syncing: bool,
    pub sp_syn_match_id: int16_t,
    pub sp_off_flags: int16_t,
    pub sp_offsets: [::core::ffi::c_int; 7],
    pub sp_flags: ::core::ffi::c_int,
    pub sp_cchar: ::core::ffi::c_int,
    pub sp_ic: ::core::ffi::c_int,
    pub sp_sync_idx: ::core::ffi::c_int,
    pub sp_line_id: ::core::ffi::c_int,
    pub sp_startcol: ::core::ffi::c_int,
    pub sp_cont_list: *mut int16_t,
    pub sp_next_list: *mut int16_t,
    pub sp_syn: sp_syn,
    pub sp_pattern: *mut ::core::ffi::c_char,
    pub sp_prog: *mut regprog_T,
    pub sp_time: syn_time_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syn_cluster_T {
    pub scl_name: *mut ::core::ffi::c_char,
    pub scl_name_u: *mut ::core::ffi::c_char,
    pub scl_list: *mut int16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct subcommand {
    pub name: *mut ::core::ffi::c_char,
    pub func: Option<unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syn_opt_arg_T {
    pub flags: ::core::ffi::c_int,
    pub keyword: bool,
    pub sync_idx: *mut ::core::ffi::c_int,
    pub has_cont_list: bool,
    pub cont_list: *mut int16_t,
    pub cont_in_list: *mut int16_t,
    pub next_list: *mut int16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pat_ptr {
    pub pp_synp: *mut synpat_T,
    pub pp_matchgroup_id: ::core::ffi::c_int,
    pub pp_next: *mut pat_ptr,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct flag {
    pub name: *mut ::core::ffi::c_char,
    pub argtype: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const EXP_CLUSTER: C2Rust_Unnamed_24 = 4;
pub const EXP_SYNC: C2Rust_Unnamed_24 = 3;
pub const EXP_SPELL: C2Rust_Unnamed_24 = 2;
pub const EXP_CASE: C2Rust_Unnamed_24 = 1;
pub const EXP_SUBCMD: C2Rust_Unnamed_24 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct time_entry_T {
    pub total: proftime_T,
    pub count: ::core::ffi::c_int,
    pub match_0: ::core::ffi::c_int,
    pub slowest: proftime_T,
    pub average: proftime_T,
    pub id: ::core::ffi::c_int,
    pub pattern: *mut ::core::ffi::c_char,
}
static namelist1: GlobalCell<[keyvalue_T; 10]> = GlobalCell::new(
    [keyvalue_T {
        key: 0,
        value: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        length: 0,
    }; 10],
);
static namelist2: GlobalCell<[keyvalue_T; 3]> = GlobalCell::new(
    [keyvalue_T {
        key: 0,
        value: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        length: 0,
    }; 3],
);
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EX_XFILE: ::core::ffi::c_uint = 0x8 as ::core::ffi::c_uint;
pub const EX_NOSPC: ::core::ffi::c_uint = 0x10 as ::core::ffi::c_uint;
pub const SYNSPL_DEFAULT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SYNSPL_TOP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SYNSPL_NOTOP: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SYNFLD_START: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SYNFLD_MINIMUM: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SYNTAX_FNAME: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"$VIMRUNTIME/syntax/%s.vim\0")
};
pub const SST_MIN_ENTRIES: ::core::ffi::c_int = 150 as ::core::ffi::c_int;
pub const SST_MAX_ENTRIES: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;
pub const SST_FIX_STATES: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const SST_DIST: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
static did_syntax_onoff: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub const SPO_MS_OFF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SPO_ME_OFF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SPO_HS_OFF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SPO_HE_OFF: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SPO_RS_OFF: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const SPO_RE_OFF: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const SPO_LC_OFF: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const SPO_COUNT: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
static e_illegal_arg: GlobalCell<[::core::ffi::c_char; 27]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E390: Illegal argument: %s\0")
});
static e_contains_argument_not_accepted_here: GlobalCell<[::core::ffi::c_char; 42]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 42], [::core::ffi::c_char; 42]>(
            *b"E395: Contains argument not accepted here\0",
        )
    });
static e_invalid_cchar_value: GlobalCell<[::core::ffi::c_char; 26]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E844: Invalid cchar value\0")
});
static e_trailing_char_after_rsb_str_str: GlobalCell<[::core::ffi::c_char; 37]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
            *b"E890: Trailing char after ']': %s]%s\0",
        )
    });
static spo_name_tab: GlobalCell<[*mut ::core::ffi::c_char; 7]> = GlobalCell::new([
    b"ms=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"me=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"hs=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"he=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"rs=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"re=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"lc=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
]);
pub const SPTYPE_MATCH: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SPTYPE_START: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SPTYPE_END: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SPTYPE_SKIP: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const NONE_IDX: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const SF_CCOMMENT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const SF_MATCH: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MAXKEYWLEN: ::core::ffi::c_int = 80 as ::core::ffi::c_int;
static current_attr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static current_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static current_trans_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static current_flags: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static current_seqnr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static current_sub_char: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub const CLUSTER_REPLACE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CLUSTER_ADD: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CLUSTER_SUBTRACT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SYNID_TOP: ::core::ffi::c_int = 21000 as ::core::ffi::c_int;
pub const SYNID_CONTAINED: ::core::ffi::c_int = 22000 as ::core::ffi::c_int;
pub const SYNID_CLUSTER: ::core::ffi::c_int = 23000 as ::core::ffi::c_int;
pub const MAX_SYN_INC_TAG: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
pub const MAX_CLUSTER_ID: ::core::ffi::c_int = 32767 as ::core::ffi::c_int - SYNID_CLUSTER;
static syn_cmdlinep: GlobalCell<*mut *mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<*mut ::core::ffi::c_char>());
static current_syn_inc_tag: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
static running_syn_inc_tag: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
static dumkey: GlobalCell<keyentry_T> = GlobalCell::new(keyentry_T {
    ke_next: ::core::ptr::null_mut::<keyentry_T>(),
    k_syn: sp_syn {
        inc_tag: 0,
        id: 0,
        cont_in_list: ::core::ptr::null_mut::<int16_t>(),
    },
    next_list: ::core::ptr::null_mut::<int16_t>(),
    flags: 0,
    k_char: 0,
    keyword: [],
});
static keepend_level: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
static msg_no_items: GlobalCell<[::core::ffi::c_char; 40]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"No Syntax items defined for this buffer\0",
    )
});
pub const KEYWORD_IDX: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const ID_LIST_ALL: *mut int16_t = -1 as ::core::ffi::c_int as *mut int16_t;
static next_seqnr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(1 as ::core::ffi::c_int);
static next_match_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static next_match_m_endpos: GlobalCell<lpos_T> = GlobalCell::new(lpos_T { lnum: 0, col: 0 });
static next_match_h_startpos: GlobalCell<lpos_T> = GlobalCell::new(lpos_T { lnum: 0, col: 0 });
static next_match_h_endpos: GlobalCell<lpos_T> = GlobalCell::new(lpos_T { lnum: 0, col: 0 });
static next_match_idx: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static next_match_flags: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static next_match_eos_pos: GlobalCell<lpos_T> = GlobalCell::new(lpos_T { lnum: 0, col: 0 });
static next_match_eoe_pos: GlobalCell<lpos_T> = GlobalCell::new(lpos_T { lnum: 0, col: 0 });
static next_match_end_idx: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static next_match_extmatch: GlobalCell<*mut reg_extmatch_T> =
    GlobalCell::new(::core::ptr::null_mut::<reg_extmatch_T>());
static syn_win: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
static syn_buf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
static syn_block: GlobalCell<*mut synblock_T> =
    GlobalCell::new(::core::ptr::null_mut::<synblock_T>());
static syn_tm: GlobalCell<*mut proftime_T> = GlobalCell::new(::core::ptr::null_mut::<proftime_T>());
static current_lnum: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
static current_col: GlobalCell<colnr_T> = GlobalCell::new(0 as colnr_T);
static current_state_stored: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static current_finished: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static current_state: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);
static current_next_list: GlobalCell<*mut int16_t> =
    GlobalCell::new(::core::ptr::null_mut::<int16_t>());
static current_next_flags: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
static current_line_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static syn_time_on: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub unsafe extern "C" fn syn_set_timeout(mut tm: *mut proftime_T) {
    syn_tm.set(tm);
}
pub unsafe extern "C" fn syntax_start(mut wp: *mut win_T, mut lnum: linenr_T) {
    let mut last_valid: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    let mut last_min_valid: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    let mut sp: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    let mut prev: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    let mut first_stored: linenr_T = 0;
    let mut dist: ::core::ffi::c_int = 0;
    static changedtick: GlobalCell<varnumber_T> = GlobalCell::new(0 as varnumber_T);
    current_sub_char.set(NUL);
    if syn_block.get() != (*wp).w_s
        || syn_buf.get() != (*wp).w_buffer
        || changedtick.get() != buf_get_changedtick(syn_buf.get())
    {
        invalidate_current_state();
        syn_buf.set((*wp).w_buffer);
        syn_block.set((*wp).w_s);
    }
    changedtick.set(buf_get_changedtick(syn_buf.get()));
    syn_win.set(wp);
    syn_stack_alloc();
    if (*syn_block.get()).b_sst_array.is_null() {
        return;
    }
    (*syn_block.get()).b_sst_lasttick = display_tick.get();
    if (*current_state.ptr()).ga_itemsize != 0 as ::core::ffi::c_int
        && current_lnum.get() < lnum
        && current_lnum.get() < (*syn_buf.get()).b_ml.ml_line_count
    {
        syn_finish_line(false_0 != 0);
        if !current_state_stored.get() {
            (*current_lnum.ptr()) += 1;
            store_current_state();
        }
        if current_lnum.get() != lnum {
            invalidate_current_state();
        }
    } else {
        invalidate_current_state();
    }
    if (*current_state.ptr()).ga_itemsize == 0 as ::core::ffi::c_int
        && !(*syn_block.get()).b_sst_array.is_null()
    {
        let mut p: *mut synstate_T = (*syn_block.get()).b_sst_first;
        while !p.is_null() {
            if (*p).sst_lnum > lnum {
                break;
            }
            if (*p).sst_change_lnum == 0 as linenr_T {
                last_valid = p;
                if (*p).sst_lnum >= lnum - (*syn_block.get()).b_syn_sync_minlines {
                    last_min_valid = p;
                }
            }
            p = (*p).sst_next;
        }
        if !last_min_valid.is_null() {
            load_current_state(last_min_valid);
        }
    }
    if (*current_state.ptr()).ga_itemsize == 0 as ::core::ffi::c_int {
        syn_sync(wp, lnum, last_valid);
        if current_lnum.get() == 1 as linenr_T {
            first_stored = 1 as ::core::ffi::c_int as linenr_T;
        } else {
            first_stored = current_lnum.get() + (*syn_block.get()).b_syn_sync_minlines;
        }
    } else {
        first_stored = current_lnum.get();
    }
    if (*syn_block.get()).b_sst_len <= Rows.get() {
        dist = 999999 as ::core::ffi::c_int;
    } else {
        dist = ((*syn_buf.get()).b_ml.ml_line_count
            / ((*syn_block.get()).b_sst_len as linenr_T - Rows.get() as linenr_T)
            + 1 as linenr_T) as ::core::ffi::c_int;
    }
    while current_lnum.get() < lnum {
        syn_start_line();
        syn_finish_line(false_0 != 0);
        (*current_lnum.ptr()) += 1;
        if current_lnum.get() >= first_stored {
            if prev.is_null() {
                prev = syn_stack_find_entry(current_lnum.get() - 1 as linenr_T);
            }
            if prev.is_null() {
                sp = (*syn_block.get()).b_sst_first;
            } else {
                sp = prev;
            }
            while !sp.is_null() && (*sp).sst_lnum < current_lnum.get() {
                sp = (*sp).sst_next;
            }
            if !sp.is_null()
                && (*sp).sst_lnum == current_lnum.get()
                && syn_stack_equal(sp) as ::core::ffi::c_int != 0
            {
                let mut parsed_lnum: linenr_T = current_lnum.get();
                prev = sp;
                while !sp.is_null() && (*sp).sst_change_lnum <= parsed_lnum {
                    if (*sp).sst_lnum <= lnum {
                        prev = sp;
                    } else if (*sp).sst_change_lnum == 0 as linenr_T {
                        break;
                    }
                    (*sp).sst_change_lnum = 0 as ::core::ffi::c_int as linenr_T;
                    sp = (*sp).sst_next;
                }
                load_current_state(prev);
            } else if prev.is_null()
                || current_lnum.get() == lnum
                || current_lnum.get() >= (*prev).sst_lnum + dist as linenr_T
            {
                prev = store_current_state();
            }
        }
        line_breakcheck();
        if !got_int.get() {
            continue;
        }
        current_lnum.set(lnum);
        break;
    }
    syn_start_line();
}
unsafe extern "C" fn clear_syn_state(mut p: *mut synstate_T) {
    if (*p).sst_stacksize > SST_FIX_STATES {
        let mut _gap: *mut garray_T = &raw mut (*p).sst_union.sst_ga;
        if !(*_gap).ga_data.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*_gap).ga_len {
                let mut _item: *mut bufstate_T =
                    ((*_gap).ga_data as *mut bufstate_T).offset(i as isize);
                unref_extmatch((*_item).bs_extmatch);
                i += 1;
            }
        }
        ga_clear(_gap);
    } else {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*p).sst_stacksize {
            unref_extmatch((*p).sst_union.sst_stack[i_0 as usize].bs_extmatch);
            i_0 += 1;
        }
    };
}
unsafe extern "C" fn clear_current_state() {
    let mut _gap: *mut garray_T = current_state.ptr();
    if !(*_gap).ga_data.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*_gap).ga_len {
            let mut _item: *mut stateitem_T =
                ((*_gap).ga_data as *mut stateitem_T).offset(i as isize);
            unref_extmatch((*_item).si_extmatch);
            i += 1;
        }
    }
    ga_clear(_gap);
}
unsafe extern "C" fn syn_sync(
    mut wp: *mut win_T,
    mut start_lnum: linenr_T,
    mut last_valid: *mut synstate_T,
) {
    let mut cursor_save: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut lnum: linenr_T = 0;
    let mut break_lnum: linenr_T = 0;
    let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
    let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
    let mut found_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut found_match_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut found_current_lnum: linenr_T = 0 as linenr_T;
    let mut found_current_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut found_m_endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
    invalidate_current_state();
    if (*syn_block.get()).b_syn_sync_minlines > start_lnum {
        start_lnum = 1 as ::core::ffi::c_int as linenr_T;
    } else {
        if (*syn_block.get()).b_syn_sync_minlines == 1 as linenr_T {
            lnum = 1 as ::core::ffi::c_int as linenr_T;
        } else if (*syn_block.get()).b_syn_sync_minlines < 10 as linenr_T {
            lnum = (*syn_block.get()).b_syn_sync_minlines * 2 as linenr_T;
        } else {
            lnum = (*syn_block.get()).b_syn_sync_minlines * 3 as linenr_T / 2 as linenr_T;
        }
        if (*syn_block.get()).b_syn_sync_maxlines != 0 as linenr_T
            && lnum > (*syn_block.get()).b_syn_sync_maxlines
        {
            lnum = (*syn_block.get()).b_syn_sync_maxlines;
        }
        if lnum >= start_lnum {
            start_lnum = 1 as ::core::ffi::c_int as linenr_T;
        } else {
            start_lnum -= lnum;
        }
    }
    current_lnum.set(start_lnum);
    if (*syn_block.get()).b_syn_sync_flags & SF_CCOMMENT != 0 {
        let mut curwin_save: *mut win_T = curwin.get();
        curwin.set(wp);
        let mut curbuf_save: *mut buf_T = curbuf.get();
        curbuf.set(syn_buf.get());
        while start_lnum > 1 as linenr_T {
            let mut l: *mut ::core::ffi::c_char = ml_get(start_lnum - 1 as linenr_T);
            if *l as ::core::ffi::c_int == NUL
                || *l
                    .offset(ml_get_len(start_lnum - 1 as linenr_T) as isize)
                    .offset(-(1 as ::core::ffi::c_int as isize))
                    as ::core::ffi::c_int
                    != '\\' as ::core::ffi::c_int
            {
                break;
            }
            start_lnum -= 1;
        }
        current_lnum.set(start_lnum);
        cursor_save = (*wp).w_cursor;
        (*wp).w_cursor.lnum = start_lnum;
        (*wp).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        if !find_start_comment((*syn_block.get()).b_syn_sync_maxlines as ::core::ffi::c_int)
            .is_null()
        {
            let mut idx: ::core::ffi::c_int = (*syn_block.get()).b_syn_patterns.ga_len;
            loop {
                idx -= 1;
                if idx < 0 as ::core::ffi::c_int {
                    break;
                }
                if !((*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                    .offset(idx as isize))
                .sp_syn
                .id as ::core::ffi::c_int
                    == (*syn_block.get()).b_syn_sync_id as ::core::ffi::c_int
                    && (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                        .offset(idx as isize))
                    .sp_type as ::core::ffi::c_int
                        == SPTYPE_START)
                {
                    continue;
                }
                validate_current_state();
                push_current_state(idx);
                update_si_attr((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int);
                break;
            }
        }
        (*wp).w_cursor = cursor_save;
        curwin.set(curwin_save);
        curbuf.set(curbuf_save);
    } else if (*syn_block.get()).b_syn_sync_flags & SF_MATCH != 0 {
        if (*syn_block.get()).b_syn_sync_maxlines != 0 as linenr_T
            && start_lnum > (*syn_block.get()).b_syn_sync_maxlines
        {
            break_lnum = start_lnum - (*syn_block.get()).b_syn_sync_maxlines;
        } else {
            break_lnum = 0 as ::core::ffi::c_int as linenr_T;
        }
        found_m_endpos.lnum = 0 as ::core::ffi::c_int as linenr_T;
        found_m_endpos.col = 0 as ::core::ffi::c_int as colnr_T;
        let mut end_lnum: linenr_T = start_lnum;
        lnum = start_lnum;
        loop {
            lnum -= 1;
            if lnum <= break_lnum {
                break;
            }
            line_breakcheck();
            if got_int.get() {
                invalidate_current_state();
                current_lnum.set(start_lnum);
                break;
            } else if !last_valid.is_null() && lnum == (*last_valid).sst_lnum {
                load_current_state(last_valid);
                break;
            } else {
                if lnum > 1 as linenr_T && syn_match_linecont(lnum - 1 as linenr_T) != 0 {
                    continue;
                }
                validate_current_state();
                current_lnum.set(lnum);
                while current_lnum.get() < end_lnum {
                    syn_start_line();
                    loop {
                        let mut had_sync_point: bool = syn_finish_line(true_0 != 0);
                        if !(had_sync_point as ::core::ffi::c_int != 0
                            && (*current_state.ptr()).ga_len != 0)
                        {
                            break;
                        }
                        cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(
                            ((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize,
                        );
                        if (*cur_si).si_m_endpos.lnum > start_lnum {
                            current_lnum.set(end_lnum);
                            break;
                        } else {
                            if (*cur_si).si_idx < 0 as ::core::ffi::c_int {
                                found_flags = 0 as ::core::ffi::c_int;
                                found_match_idx = KEYWORD_IDX;
                            } else {
                                spp = ((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                                    .offset((*cur_si).si_idx as isize);
                                found_flags = (*spp).sp_flags;
                                found_match_idx = (*spp).sp_sync_idx;
                            }
                            found_current_lnum = current_lnum.get();
                            found_current_col = current_col.get() as ::core::ffi::c_int;
                            found_m_endpos = (*cur_si).si_m_endpos;
                            if found_m_endpos.lnum > current_lnum.get() {
                                current_lnum.set(found_m_endpos.lnum);
                                current_col.set(found_m_endpos.col);
                                if current_lnum.get() >= end_lnum {
                                    break;
                                }
                            } else if found_m_endpos.col > current_col.get() {
                                current_col.set(found_m_endpos.col);
                            } else {
                                (*current_col.ptr()) += 1;
                            }
                            let mut prev_current_col: colnr_T = current_col.get();
                            if *syn_getcurline().offset(current_col.get() as isize)
                                as ::core::ffi::c_int
                                != NUL
                            {
                                (*current_col.ptr()) += 1;
                            }
                            check_state_ends();
                            current_col.set(prev_current_col);
                        }
                    }
                    (*current_lnum.ptr()) += 1;
                }
                if found_flags != 0 {
                    clear_current_state();
                    if found_match_idx >= 0 as ::core::ffi::c_int {
                        push_current_state(found_match_idx);
                        update_si_attr((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int);
                    }
                    if found_flags & HL_SYNC_HERE != 0 {
                        current_lnum.set(found_m_endpos.lnum);
                        current_col.set(found_m_endpos.col);
                        if !((*current_state.ptr()).ga_len <= 0 as ::core::ffi::c_int) {
                            cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(
                                ((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize,
                            );
                            (*cur_si).si_h_startpos.lnum = found_current_lnum;
                            (*cur_si).si_h_startpos.col = found_current_col as colnr_T;
                            update_si_end(cur_si, current_col.get(), true_0 != 0);
                            check_keepend();
                        }
                        syn_finish_line(false_0 != 0);
                        (*current_lnum.ptr()) += 1;
                    } else {
                        current_lnum.set(start_lnum);
                    }
                    break;
                } else {
                    end_lnum = lnum;
                    invalidate_current_state();
                }
            }
        }
        if lnum <= break_lnum {
            invalidate_current_state();
            current_lnum.set(break_lnum + 1 as linenr_T);
        }
    }
    validate_current_state();
}
unsafe extern "C" fn save_chartab(mut chartab: *mut ::core::ffi::c_char) {
    if (*syn_block.get()).b_syn_isk == empty_string_option.ptr() as *mut ::core::ffi::c_char {
        return;
    }
    memmove(
        chartab as *mut ::core::ffi::c_void,
        &raw mut (*syn_buf.get()).b_chartab as *mut uint64_t as *const ::core::ffi::c_void,
        32 as ::core::ffi::c_int as size_t,
    );
    memmove(
        &raw mut (*syn_buf.get()).b_chartab as *mut uint64_t as *mut ::core::ffi::c_void,
        &raw mut (*(*syn_win.get()).w_s).b_syn_chartab as *mut uint8_t
            as *const ::core::ffi::c_void,
        32 as ::core::ffi::c_int as size_t,
    );
}
unsafe extern "C" fn restore_chartab(mut chartab: *mut ::core::ffi::c_char) {
    if (*(*syn_win.get()).w_s).b_syn_isk != empty_string_option.ptr() as *mut ::core::ffi::c_char {
        memmove(
            &raw mut (*syn_buf.get()).b_chartab as *mut uint64_t as *mut ::core::ffi::c_void,
            chartab as *const ::core::ffi::c_void,
            32 as ::core::ffi::c_int as size_t,
        );
    }
}
unsafe extern "C" fn syn_match_linecont(mut lnum: linenr_T) -> ::core::ffi::c_int {
    if (*syn_block.get()).b_syn_linecont_prog.is_null() {
        return false_0;
    }
    let mut regmatch: regmmatch_T = regmmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    };
    let mut buf_chartab: [::core::ffi::c_char; 32] = [0; 32];
    save_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
    regmatch.rmm_ic = (*syn_block.get()).b_syn_linecont_ic;
    regmatch.regprog = (*syn_block.get()).b_syn_linecont_prog;
    let mut r: ::core::ffi::c_int = syn_regexec(
        &raw mut regmatch,
        lnum,
        0 as colnr_T,
        &raw mut (*syn_block.get()).b_syn_linecont_time,
    ) as ::core::ffi::c_int;
    (*syn_block.get()).b_syn_linecont_prog = regmatch.regprog;
    restore_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
    return r;
}
unsafe extern "C" fn syn_start_line() {
    current_finished.set(false_0 != 0);
    current_col.set(0 as ::core::ffi::c_int as colnr_T);
    if !((*current_state.ptr()).ga_len <= 0 as ::core::ffi::c_int) {
        syn_update_ends(true_0 != 0);
        check_state_ends();
    }
    next_match_idx.set(-1 as ::core::ffi::c_int);
    (*current_line_id.ptr()) += 1;
    next_seqnr.set(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn syn_update_ends(mut startofline: bool) {
    let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
    if startofline {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*current_state.ptr()).ga_len {
            cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize);
            if (*cur_si).si_idx >= 0 as ::core::ffi::c_int
                && (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                    .offset((*cur_si).si_idx as isize))
                .sp_type as ::core::ffi::c_int
                    == SPTYPE_MATCH
                && (*cur_si).si_m_endpos.lnum < current_lnum.get()
            {
                (*cur_si).si_flags |= HL_MATCHCONT;
                (*cur_si).si_m_endpos.lnum = 0 as ::core::ffi::c_int as linenr_T;
                (*cur_si).si_m_endpos.col = 0 as ::core::ffi::c_int as colnr_T;
                (*cur_si).si_h_endpos = (*cur_si).si_m_endpos;
                (*cur_si).si_ends = true_0;
            }
            i += 1;
        }
    }
    let mut i_0: ::core::ffi::c_int = (*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int;
    if keepend_level.get() >= 0 as ::core::ffi::c_int {
        while i_0 > keepend_level.get() {
            if (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i_0 as isize)).si_flags
                & HL_EXTEND
                != 0
            {
                break;
            }
            i_0 -= 1;
        }
    }
    let mut seen_keepend: bool = false_0 != 0;
    while i_0 < (*current_state.ptr()).ga_len {
        cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i_0 as isize);
        if (*cur_si).si_flags & HL_KEEPEND != 0
            || seen_keepend as ::core::ffi::c_int != 0 && !startofline
            || i_0 == (*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int
                && startofline as ::core::ffi::c_int != 0
        {
            (*cur_si).si_h_startpos.col = 0 as ::core::ffi::c_int as colnr_T;
            (*cur_si).si_h_startpos.lnum = current_lnum.get();
            if (*cur_si).si_flags & HL_MATCHCONT == 0 {
                update_si_end(cur_si, current_col.get(), !startofline);
            }
            if !startofline && (*cur_si).si_flags & HL_KEEPEND != 0 {
                seen_keepend = true_0 != 0;
            }
        }
        i_0 += 1;
    }
    check_keepend();
}
unsafe extern "C" fn syn_stack_free_block(mut block: *mut synblock_T) {
    if (*block).b_sst_array.is_null() {
        return;
    }
    let mut p: *mut synstate_T = (*block).b_sst_first;
    while !p.is_null() {
        clear_syn_state(p);
        p = (*p).sst_next;
    }
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*block).b_sst_array as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    (*block).b_sst_first = ::core::ptr::null_mut::<synstate_T>();
    (*block).b_sst_len = 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn syn_stack_free_all(mut block: *mut synblock_T) {
    syn_stack_free_block(block);
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_s == block && foldmethodIsSyntax(wp) as ::core::ffi::c_int != 0 {
            foldUpdateAll(wp);
        }
        wp = (*wp).w_next;
    }
}
unsafe extern "C" fn syn_stack_alloc() {
    let mut len: ::core::ffi::c_int = (*syn_buf.get()).b_ml.ml_line_count as ::core::ffi::c_int
        / SST_DIST
        + Rows.get() * 2 as ::core::ffi::c_int;
    if len < SST_MIN_ENTRIES {
        len = SST_MIN_ENTRIES;
    } else if len > SST_MAX_ENTRIES {
        len = SST_MAX_ENTRIES;
    }
    if (*syn_block.get()).b_sst_len > len * 2 as ::core::ffi::c_int
        || (*syn_block.get()).b_sst_len < len
    {
        len = (*syn_buf.get()).b_ml.ml_line_count as ::core::ffi::c_int;
        len =
            (len + len / 2 as ::core::ffi::c_int) / SST_DIST + Rows.get() * 2 as ::core::ffi::c_int;
        if len < SST_MIN_ENTRIES {
            len = SST_MIN_ENTRIES;
        } else if len > SST_MAX_ENTRIES {
            len = SST_MAX_ENTRIES;
        }
        if !(*syn_block.get()).b_sst_array.is_null() {
            while (*syn_block.get()).b_sst_len - (*syn_block.get()).b_sst_freecount
                + 2 as ::core::ffi::c_int
                > len
                && syn_stack_cleanup() as ::core::ffi::c_int != 0
            {}
            if len
                < (*syn_block.get()).b_sst_len - (*syn_block.get()).b_sst_freecount
                    + 2 as ::core::ffi::c_int
            {
                len = (*syn_block.get()).b_sst_len - (*syn_block.get()).b_sst_freecount
                    + 2 as ::core::ffi::c_int;
            }
        }
        '_c2rust_label: {
            if len >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"len >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/syntax.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    926 as ::core::ffi::c_uint,
                    b"void syn_stack_alloc(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut sstp: *mut synstate_T =
            xcalloc(len as size_t, ::core::mem::size_of::<synstate_T>()) as *mut synstate_T;
        let mut to: *mut synstate_T = sstp.offset(-(1 as ::core::ffi::c_int as isize));
        if !(*syn_block.get()).b_sst_array.is_null() {
            let mut from: *mut synstate_T = (*syn_block.get()).b_sst_first;
            while !from.is_null() {
                to = to.offset(1);
                *to = *from;
                (*to).sst_next = to.offset(1 as ::core::ffi::c_int as isize);
                from = (*from).sst_next;
            }
        }
        if to != sstp.offset(-(1 as ::core::ffi::c_int as isize)) {
            (*to).sst_next = ::core::ptr::null_mut::<synstate_T>();
            (*syn_block.get()).b_sst_first = sstp;
            (*syn_block.get()).b_sst_freecount =
                len - to.offset_from(sstp) as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        } else {
            (*syn_block.get()).b_sst_first = ::core::ptr::null_mut::<synstate_T>();
            (*syn_block.get()).b_sst_freecount = len;
        }
        (*syn_block.get()).b_sst_firstfree = to.offset(1 as ::core::ffi::c_int as isize);
        loop {
            to = to.offset(1);
            if to >= sstp.offset(len as isize) {
                break;
            }
            (*to).sst_next = to.offset(1 as ::core::ffi::c_int as isize);
        }
        (*sstp
            .offset(len as isize)
            .offset(-(1 as ::core::ffi::c_int as isize)))
        .sst_next = ::core::ptr::null_mut::<synstate_T>();
        xfree((*syn_block.get()).b_sst_array as *mut ::core::ffi::c_void);
        (*syn_block.get()).b_sst_array = sstp;
        (*syn_block.get()).b_sst_len = len;
    }
}
pub unsafe extern "C" fn syn_stack_apply_changes(mut buf: *mut buf_T) {
    syn_stack_apply_changes_block(&raw mut (*buf).b_s, buf);
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer == buf && (*wp).w_s != &raw mut (*buf).b_s {
            syn_stack_apply_changes_block((*wp).w_s, buf);
        }
        wp = (*wp).w_next;
    }
}
unsafe extern "C" fn syn_stack_apply_changes_block(
    mut block: *mut synblock_T,
    mut buf: *mut buf_T,
) {
    let mut prev: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    let mut p: *mut synstate_T = (*block).b_sst_first;
    while !p.is_null() {
        if (*p).sst_lnum + (*block).b_syn_sync_linebreaks > (*buf).b_mod_top {
            let mut n: linenr_T = (*p).sst_lnum + (*buf).b_mod_xlines;
            if n <= (*buf).b_mod_bot {
                let mut np: *mut synstate_T = (*p).sst_next;
                if prev.is_null() {
                    (*block).b_sst_first = np;
                } else {
                    (*prev).sst_next = np;
                }
                syn_stack_free_entry(block, p);
                p = np;
                continue;
            } else {
                if (*p).sst_change_lnum != 0 as linenr_T && (*p).sst_change_lnum > (*buf).b_mod_top
                {
                    if (*p).sst_change_lnum + (*buf).b_mod_xlines > (*buf).b_mod_top {
                        (*p).sst_change_lnum += (*buf).b_mod_xlines;
                    } else {
                        (*p).sst_change_lnum = (*buf).b_mod_top;
                    }
                }
                if (*p).sst_change_lnum == 0 as linenr_T || (*p).sst_change_lnum < (*buf).b_mod_bot
                {
                    (*p).sst_change_lnum = (*buf).b_mod_bot;
                }
                (*p).sst_lnum = n;
            }
        }
        prev = p;
        p = (*p).sst_next;
    }
}
unsafe extern "C" fn syn_stack_cleanup() -> bool {
    let mut prev: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    let mut tick: disptick_T = 0;
    let mut dist: ::core::ffi::c_int = 0;
    let mut retval: bool = false_0 != 0;
    if (*syn_block.get()).b_sst_first.is_null() {
        return retval;
    }
    if (*syn_block.get()).b_sst_len <= Rows.get() {
        dist = 999999 as ::core::ffi::c_int;
    } else {
        dist = ((*syn_buf.get()).b_ml.ml_line_count
            / ((*syn_block.get()).b_sst_len as linenr_T - Rows.get() as linenr_T)
            + 1 as linenr_T) as ::core::ffi::c_int;
    }
    tick = (*syn_block.get()).b_sst_lasttick;
    let mut above: bool = false_0 != 0;
    prev = (*syn_block.get()).b_sst_first;
    let mut p: *mut synstate_T = (*prev).sst_next;
    while !p.is_null() {
        if (*prev).sst_lnum + dist as linenr_T > (*p).sst_lnum {
            if (*p).sst_tick > (*syn_block.get()).b_sst_lasttick {
                if !above || (*p).sst_tick < tick {
                    tick = (*p).sst_tick;
                }
                above = true_0 != 0;
            } else if !above && (*p).sst_tick < tick {
                tick = (*p).sst_tick;
            }
        }
        prev = p;
        p = (*p).sst_next;
    }
    prev = (*syn_block.get()).b_sst_first;
    let mut p_0: *mut synstate_T = (*prev).sst_next;
    while !p_0.is_null() {
        if (*p_0).sst_tick == tick && (*prev).sst_lnum + dist as linenr_T > (*p_0).sst_lnum {
            (*prev).sst_next = (*p_0).sst_next;
            syn_stack_free_entry(syn_block.get(), p_0);
            p_0 = prev;
            retval = true_0 != 0;
        }
        prev = p_0;
        p_0 = (*p_0).sst_next;
    }
    return retval;
}
unsafe extern "C" fn syn_stack_free_entry(mut block: *mut synblock_T, mut p: *mut synstate_T) {
    clear_syn_state(p);
    (*p).sst_next = (*block).b_sst_firstfree;
    (*block).b_sst_firstfree = p;
    (*block).b_sst_freecount += 1;
}
unsafe extern "C" fn syn_stack_find_entry(mut lnum: linenr_T) -> *mut synstate_T {
    let mut prev: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    let mut p: *mut synstate_T = (*syn_block.get()).b_sst_first;
    while !p.is_null() {
        if (*p).sst_lnum == lnum {
            return p;
        }
        if (*p).sst_lnum > lnum {
            break;
        }
        prev = p;
        p = (*p).sst_next;
    }
    return prev;
}
unsafe extern "C" fn store_current_state() -> *mut synstate_T {
    let mut i: ::core::ffi::c_int = 0;
    let mut p: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    let mut bp: *mut bufstate_T = ::core::ptr::null_mut::<bufstate_T>();
    let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
    let mut sp: *mut synstate_T = syn_stack_find_entry(current_lnum.get());
    i = (*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize);
        if (*cur_si).si_h_startpos.lnum >= current_lnum.get()
            || (*cur_si).si_m_endpos.lnum >= current_lnum.get()
            || (*cur_si).si_h_endpos.lnum >= current_lnum.get()
            || (*cur_si).si_end_idx != 0 && (*cur_si).si_eoe_pos.lnum >= current_lnum.get()
        {
            break;
        }
        i -= 1;
    }
    if i >= 0 as ::core::ffi::c_int {
        if !sp.is_null() {
            if (*syn_block.get()).b_sst_first == sp {
                (*syn_block.get()).b_sst_first = (*sp).sst_next;
            } else {
                p = (*syn_block.get()).b_sst_first;
                while !p.is_null() {
                    if (*p).sst_next == sp {
                        break;
                    }
                    p = (*p).sst_next;
                }
                if !p.is_null() {
                    (*p).sst_next = (*sp).sst_next;
                }
            }
            syn_stack_free_entry(syn_block.get(), sp);
            sp = ::core::ptr::null_mut::<synstate_T>();
        }
    } else if sp.is_null() || (*sp).sst_lnum != current_lnum.get() {
        if (*syn_block.get()).b_sst_freecount == 0 as ::core::ffi::c_int {
            syn_stack_cleanup();
            sp = syn_stack_find_entry(current_lnum.get());
        }
        if (*syn_block.get()).b_sst_freecount == 0 as ::core::ffi::c_int {
            sp = ::core::ptr::null_mut::<synstate_T>();
        } else {
            p = (*syn_block.get()).b_sst_firstfree;
            (*syn_block.get()).b_sst_firstfree = (*p).sst_next;
            (*syn_block.get()).b_sst_freecount -= 1;
            if sp.is_null() {
                (*p).sst_next = (*syn_block.get()).b_sst_first;
                (*syn_block.get()).b_sst_first = p;
            } else {
                (*p).sst_next = (*sp).sst_next;
                (*sp).sst_next = p;
            }
            sp = p;
            (*sp).sst_stacksize = 0 as ::core::ffi::c_int;
            (*sp).sst_lnum = current_lnum.get();
        }
    }
    if !sp.is_null() {
        clear_syn_state(sp);
        (*sp).sst_stacksize = (*current_state.ptr()).ga_len;
        if (*current_state.ptr()).ga_len > SST_FIX_STATES {
            ga_init(
                &raw mut (*sp).sst_union.sst_ga,
                ::core::mem::size_of::<bufstate_T>() as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            ga_grow(
                &raw mut (*sp).sst_union.sst_ga,
                (*current_state.ptr()).ga_len,
            );
            (*sp).sst_union.sst_ga.ga_len = (*current_state.ptr()).ga_len;
            bp = (*sp).sst_union.sst_ga.ga_data as *mut bufstate_T;
        } else {
            bp = &raw mut (*sp).sst_union.sst_stack as *mut bufstate_T;
        }
        i = 0 as ::core::ffi::c_int;
        while i < (*sp).sst_stacksize {
            (*bp.offset(i as isize)).bs_idx =
                (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_idx;
            (*bp.offset(i as isize)).bs_flags =
                (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_flags;
            (*bp.offset(i as isize)).bs_seqnr =
                (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_seqnr;
            (*bp.offset(i as isize)).bs_cchar =
                (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_cchar;
            (*bp.offset(i as isize)).bs_extmatch = ref_extmatch(
                (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                    .si_extmatch,
            );
            i += 1;
        }
        (*sp).sst_next_flags = current_next_flags.get();
        (*sp).sst_next_list = current_next_list.get();
        (*sp).sst_tick = display_tick.get();
        (*sp).sst_change_lnum = 0 as ::core::ffi::c_int as linenr_T;
    }
    current_state_stored.set(true_0 != 0);
    return sp;
}
unsafe extern "C" fn load_current_state(mut from: *mut synstate_T) {
    let mut bp: *mut bufstate_T = ::core::ptr::null_mut::<bufstate_T>();
    clear_current_state();
    validate_current_state();
    keepend_level.set(-1 as ::core::ffi::c_int);
    if (*from).sst_stacksize != 0 {
        ga_grow(current_state.ptr(), (*from).sst_stacksize);
        if (*from).sst_stacksize > SST_FIX_STATES {
            bp = (*from).sst_union.sst_ga.ga_data as *mut bufstate_T;
        } else {
            bp = &raw mut (*from).sst_union.sst_stack as *mut bufstate_T;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*from).sst_stacksize {
            (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_idx =
                (*bp.offset(i as isize)).bs_idx;
            (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_flags =
                (*bp.offset(i as isize)).bs_flags;
            (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_seqnr =
                (*bp.offset(i as isize)).bs_seqnr;
            (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_cchar =
                (*bp.offset(i as isize)).bs_cchar;
            (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                .si_extmatch = ref_extmatch((*bp.offset(i as isize)).bs_extmatch);
            if keepend_level.get() < 0 as ::core::ffi::c_int
                && (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                    .si_flags
                    & HL_KEEPEND
                    != 0
            {
                keepend_level.set(i);
            }
            (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_ends =
                false_0;
            (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_m_lnum =
                0 as ::core::ffi::c_int;
            if (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_idx
                >= 0 as ::core::ffi::c_int
            {
                (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                    .si_next_list = (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                    .offset(
                        (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                            .si_idx as isize,
                    ))
                .sp_next_list;
            } else {
                (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                    .si_next_list = ::core::ptr::null_mut::<int16_t>();
            }
            update_si_attr(i);
            i += 1;
        }
        (*current_state.ptr()).ga_len = (*from).sst_stacksize;
    }
    current_next_list.set((*from).sst_next_list);
    current_next_flags.set((*from).sst_next_flags);
    current_lnum.set((*from).sst_lnum);
}
unsafe extern "C" fn syn_stack_equal(mut sp: *mut synstate_T) -> bool {
    let mut bp: *mut bufstate_T = ::core::ptr::null_mut::<bufstate_T>();
    if (*sp).sst_stacksize != (*current_state.ptr()).ga_len
        || (*sp).sst_next_list != current_next_list.get()
    {
        return false_0 != 0;
    }
    if (*sp).sst_stacksize > SST_FIX_STATES {
        bp = (*sp).sst_union.sst_ga.ga_data as *mut bufstate_T;
    } else {
        bp = &raw mut (*sp).sst_union.sst_stack as *mut bufstate_T;
    }
    let mut i: ::core::ffi::c_int = 0;
    i = (*current_state.ptr()).ga_len;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        if (*bp.offset(i as isize)).bs_idx
            != (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_idx
        {
            break;
        }
        if (*bp.offset(i as isize)).bs_extmatch
            == (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                .si_extmatch
        {
            continue;
        }
        let mut bsx: *mut reg_extmatch_T = (*bp.offset(i as isize)).bs_extmatch;
        let mut six: *mut reg_extmatch_T =
            (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_extmatch;
        if bsx.is_null() || six.is_null() {
            break;
        }
        let mut j: ::core::ffi::c_int = 0;
        j = 0 as ::core::ffi::c_int;
        while j < NSUBEXP as ::core::ffi::c_int {
            if (*bsx).matches[j as usize] != (*six).matches[j as usize] {
                if (*bsx).matches[j as usize].is_null() || (*six).matches[j as usize].is_null() {
                    break;
                }
                if mb_strcmp_ic(
                    (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T).offset(
                        (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                            .si_idx as isize,
                    ))
                    .sp_ic
                        != 0,
                    (*bsx).matches[j as usize] as *const ::core::ffi::c_char,
                    (*six).matches[j as usize] as *const ::core::ffi::c_char,
                ) != 0 as ::core::ffi::c_int
                {
                    break;
                }
            }
            j += 1;
        }
        if j != NSUBEXP as ::core::ffi::c_int {
            break;
        }
    }
    return if i < 0 as ::core::ffi::c_int {
        true_0
    } else {
        false_0
    } != 0;
}
pub unsafe extern "C" fn syntax_end_parsing(mut wp: *mut win_T, mut lnum: linenr_T) {
    let mut sp: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    if syn_block.get() != (*wp).w_s {
        return;
    }
    sp = syn_stack_find_entry(lnum);
    if !sp.is_null() && (*sp).sst_lnum < lnum {
        sp = (*sp).sst_next;
    }
    if !sp.is_null() && (*sp).sst_change_lnum != 0 as linenr_T {
        (*sp).sst_change_lnum = lnum;
    }
}
unsafe extern "C" fn invalidate_current_state() {
    clear_current_state();
    (*current_state.ptr()).ga_itemsize = 0 as ::core::ffi::c_int;
    current_next_list.set(::core::ptr::null_mut::<int16_t>());
    keepend_level.set(-1 as ::core::ffi::c_int);
}
unsafe extern "C" fn validate_current_state() {
    (*current_state.ptr()).ga_itemsize =
        ::core::mem::size_of::<stateitem_T>() as ::core::ffi::c_int;
    ga_set_growsize(current_state.ptr(), 3 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn syntax_check_changed(mut lnum: linenr_T) -> bool {
    let mut retval: bool = true_0 != 0;
    let mut sp: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
    if (*current_state.ptr()).ga_itemsize != 0 as ::core::ffi::c_int
        && lnum == current_lnum.get() + 1 as linenr_T
    {
        sp = syn_stack_find_entry(lnum);
        if !sp.is_null() && (*sp).sst_lnum == lnum {
            syn_finish_line(false_0 != 0);
            if syn_stack_equal(sp) {
                retval = false_0 != 0;
            }
            (*current_lnum.ptr()) += 1;
            store_current_state();
        }
    }
    return retval;
}
unsafe extern "C" fn syn_finish_line(syncing: bool) -> bool {
    while !current_finished.get() {
        syn_current_attr(
            syncing,
            false_0 != 0,
            ::core::ptr::null_mut::<bool>(),
            false_0 != 0,
        );
        if syncing as ::core::ffi::c_int != 0 && (*current_state.ptr()).ga_len != 0 {
            let cur_si: *const stateitem_T = ((*current_state.ptr()).ga_data as *mut stateitem_T)
                .offset(((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize);
            if (*cur_si).si_idx >= 0 as ::core::ffi::c_int
                && (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                    .offset((*cur_si).si_idx as isize))
                .sp_flags
                    & (HL_SYNC_HERE | HL_SYNC_THERE)
                    != 0
            {
                return true_0 != 0;
            }
            let prev_current_col: colnr_T = current_col.get();
            if *syn_getcurline().offset(current_col.get() as isize) as ::core::ffi::c_int != NUL {
                (*current_col.ptr()) += 1;
            }
            check_state_ends();
            current_col.set(prev_current_col);
        }
        (*current_col.ptr()) += 1;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn get_syntax_attr(
    col: colnr_T,
    can_spell: *mut bool,
    keep_state: bool,
) -> ::core::ffi::c_int {
    let mut attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !can_spell.is_null() {
        *can_spell = if (*syn_block.get()).b_syn_spell == SYNSPL_DEFAULT {
            ((*syn_block.get()).b_spell_cluster_id == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
        } else {
            ((*syn_block.get()).b_syn_spell == SYNSPL_TOP) as ::core::ffi::c_int
        } != 0;
    }
    if (*syn_block.get()).b_sst_array.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*syn_buf.get()).b_p_smc > 0 as OptInt && col >= (*syn_buf.get()).b_p_smc as colnr_T {
        clear_current_state();
        current_id.set(0 as ::core::ffi::c_int);
        current_trans_id.set(0 as ::core::ffi::c_int);
        current_flags.set(0 as ::core::ffi::c_int);
        current_seqnr.set(0 as ::core::ffi::c_int);
        return 0 as ::core::ffi::c_int;
    }
    if (*current_state.ptr()).ga_itemsize == 0 as ::core::ffi::c_int {
        validate_current_state();
    }
    while current_col.get() <= col {
        attr = syn_current_attr(
            false_0 != 0,
            true_0 != 0,
            can_spell,
            if current_col.get() == col {
                keep_state as ::core::ffi::c_int
            } else {
                false_0
            } != 0,
        );
        (*current_col.ptr()) += 1;
    }
    return attr;
}
unsafe extern "C" fn syn_current_attr(
    syncing: bool,
    displaying: bool,
    can_spell: *mut bool,
    keep_state: bool,
) -> ::core::ffi::c_int {
    let mut endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut hl_startpos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut hl_endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut eos_pos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut eoe_pos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut end_idx: ::core::ffi::c_int = 0;
    let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
    let mut sip: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
    let mut startcol: ::core::ffi::c_int = 0;
    let mut endcol: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0;
    let mut cchar: ::core::ffi::c_int = 0;
    let mut next_list: *mut int16_t = ::core::ptr::null_mut::<int16_t>();
    let mut found_match: bool = false;
    static try_next_column: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut regmatch: regmmatch_T = regmmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    };
    let mut pos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut cur_extmatch: *mut reg_extmatch_T = ::core::ptr::null_mut::<reg_extmatch_T>();
    let mut buf_chartab: [::core::ffi::c_char; 32] = [0; 32];
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut keep_next_list: bool = false;
    let mut zero_width_next_list: bool = false_0 != 0;
    let mut zero_width_next_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    line = syn_getcurline();
    if *line.offset(current_col.get() as isize) as ::core::ffi::c_int == NUL
        && current_col.get() != 0 as ::core::ffi::c_int
    {
        if next_match_idx.get() >= 0 as ::core::ffi::c_int
            && next_match_col.get() >= current_col.get()
            && next_match_col.get() != MAXCOL as ::core::ffi::c_int
        {
            push_next_match();
        }
        current_finished.set(true_0 != 0);
        current_state_stored.set(false_0 != 0);
        return 0 as ::core::ffi::c_int;
    }
    if *line.offset(current_col.get() as isize) as ::core::ffi::c_int == NUL
        || *line
            .offset((current_col.get() as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            == NUL
    {
        current_finished.set(true_0 != 0);
        current_state_stored.set(false_0 != 0);
    }
    if try_next_column.get() {
        next_match_idx.set(-1 as ::core::ffi::c_int);
        try_next_column.set(false_0 != 0);
    }
    let do_keywords: bool = !syncing
        && ((*syn_block.get()).b_keywtab.ht_used > 0 as size_t
            || (*syn_block.get()).b_keywtab_ic.ht_used > 0 as size_t);
    ga_init(
        &raw mut zero_width_next_ga,
        ::core::mem::size_of::<::core::ffi::c_int>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    save_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
    loop {
        found_match = false_0 != 0;
        keep_next_list = false_0 != 0;
        let mut syn_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*current_state.ptr()).ga_len != 0 {
            cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T)
                .offset(((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize);
        } else {
            cur_si = ::core::ptr::null_mut::<stateitem_T>();
        }
        if (*syn_block.get()).b_syn_containedin != 0
            || cur_si.is_null()
            || !(*cur_si).si_cont_list.is_null()
        {
            if do_keywords {
                line = syn_getcurline();
                let mut cur_pos: *const ::core::ffi::c_char =
                    line.offset(current_col.get() as isize);
                if vim_iswordp_buf(cur_pos, syn_buf.get()) as ::core::ffi::c_int != 0
                    && (current_col.get() == 0 as ::core::ffi::c_int
                        || !vim_iswordp_buf(
                            cur_pos.offset(-(1 as ::core::ffi::c_int as isize)).offset(
                                -(utf_head_off(
                                    line,
                                    cur_pos.offset(-(1 as ::core::ffi::c_int as isize)),
                                ) as isize),
                            ),
                            syn_buf.get(),
                        ))
                {
                    syn_id = check_keyword_id(
                        line,
                        current_col.get(),
                        &raw mut endcol,
                        &raw mut flags,
                        &raw mut next_list,
                        cur_si,
                        &raw mut cchar,
                    );
                    if syn_id != 0 as ::core::ffi::c_int {
                        push_current_state(KEYWORD_IDX);
                        cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(
                            ((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize,
                        );
                        (*cur_si).si_m_startcol = current_col.get() as ::core::ffi::c_int;
                        (*cur_si).si_h_startpos.lnum = current_lnum.get();
                        (*cur_si).si_h_startpos.col = 0 as ::core::ffi::c_int as colnr_T;
                        (*cur_si).si_m_endpos.lnum = current_lnum.get();
                        (*cur_si).si_m_endpos.col = endcol as colnr_T;
                        (*cur_si).si_h_endpos.lnum = current_lnum.get();
                        (*cur_si).si_h_endpos.col = endcol as colnr_T;
                        (*cur_si).si_ends = true_0;
                        (*cur_si).si_end_idx = 0 as ::core::ffi::c_int;
                        (*cur_si).si_flags = flags;
                        let c2rust_fresh3 = next_seqnr.get();
                        next_seqnr.set(next_seqnr.get() + 1);
                        (*cur_si).si_seqnr = c2rust_fresh3;
                        (*cur_si).si_cchar = cchar;
                        if (*current_state.ptr()).ga_len > 1 as ::core::ffi::c_int {
                            (*cur_si).si_flags |=
                                (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(
                                    ((*current_state.ptr()).ga_len - 2 as ::core::ffi::c_int)
                                        as isize,
                                ))
                                .si_flags
                                    & HL_CONCEAL;
                        }
                        (*cur_si).si_id = syn_id;
                        (*cur_si).si_trans_id = syn_id;
                        if flags & HL_TRANSP != 0 {
                            if (*current_state.ptr()).ga_len < 2 as ::core::ffi::c_int {
                                (*cur_si).si_attr = 0 as ::core::ffi::c_int;
                                (*cur_si).si_trans_id = 0 as ::core::ffi::c_int;
                            } else {
                                (*cur_si).si_attr =
                                    (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(
                                        ((*current_state.ptr()).ga_len - 2 as ::core::ffi::c_int)
                                            as isize,
                                    ))
                                    .si_attr;
                                (*cur_si).si_trans_id =
                                    (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(
                                        ((*current_state.ptr()).ga_len - 2 as ::core::ffi::c_int)
                                            as isize,
                                    ))
                                    .si_trans_id;
                            }
                        } else {
                            (*cur_si).si_attr = syn_id2attr(syn_id);
                        }
                        (*cur_si).si_cont_list = ::core::ptr::null_mut::<int16_t>();
                        (*cur_si).si_next_list = next_list;
                        check_keepend();
                    }
                }
            }
            if syn_id == 0 as ::core::ffi::c_int && (*syn_block.get()).b_syn_patterns.ga_len != 0 {
                if next_match_idx.get() < 0 as ::core::ffi::c_int
                    || next_match_col.get() < current_col.get()
                {
                    next_match_idx.set(0 as ::core::ffi::c_int);
                    next_match_col.set(MAXCOL as ::core::ffi::c_int);
                    let mut idx: ::core::ffi::c_int = (*syn_block.get()).b_syn_patterns.ga_len;
                    loop {
                        idx -= 1;
                        if idx < 0 as ::core::ffi::c_int {
                            break;
                        }
                        let spp: *mut synpat_T = ((*syn_block.get()).b_syn_patterns.ga_data
                            as *mut synpat_T)
                            .offset(idx as isize);
                        if !((*spp).sp_syncing as ::core::ffi::c_int
                            == syncing as ::core::ffi::c_int
                            && (displaying as ::core::ffi::c_int != 0
                                || (*spp).sp_flags & HL_DISPLAY == 0)
                            && ((*spp).sp_type as ::core::ffi::c_int == SPTYPE_MATCH
                                || (*spp).sp_type as ::core::ffi::c_int == SPTYPE_START)
                            && (if !(*current_next_list.ptr()).is_null() {
                                in_id_list(
                                    ::core::ptr::null_mut::<stateitem_T>(),
                                    current_next_list.get(),
                                    &raw mut (*spp).sp_syn,
                                    0 as ::core::ffi::c_int,
                                )
                            } else {
                                if cur_si.is_null() {
                                    ((*spp).sp_flags & HL_CONTAINED == 0) as ::core::ffi::c_int
                                } else {
                                    in_id_list(
                                        cur_si,
                                        (*cur_si).si_cont_list,
                                        &raw mut (*spp).sp_syn,
                                        (*spp).sp_flags,
                                    )
                                }
                            }) != 0)
                        {
                            continue;
                        }
                        if (*spp).sp_line_id == current_line_id.get()
                            && (*spp).sp_startcol >= next_match_col.get()
                        {
                            continue;
                        }
                        (*spp).sp_line_id = current_line_id.get();
                        let mut lc_col: colnr_T =
                            current_col.get() - (*spp).sp_offsets[SPO_LC_OFF as usize] as colnr_T;
                        if lc_col < 0 as ::core::ffi::c_int {
                            lc_col = 0 as ::core::ffi::c_int as colnr_T;
                        }
                        regmatch.rmm_ic = (*spp).sp_ic;
                        regmatch.regprog = (*spp).sp_prog;
                        let mut r: ::core::ffi::c_int = syn_regexec(
                            &raw mut regmatch,
                            current_lnum.get(),
                            lc_col,
                            &raw mut (*spp).sp_time,
                        )
                            as ::core::ffi::c_int;
                        (*spp).sp_prog = regmatch.regprog;
                        if r == 0 {
                            (*spp).sp_startcol = MAXCOL as ::core::ffi::c_int;
                        } else {
                            syn_add_start_off(
                                &raw mut pos,
                                &raw mut regmatch,
                                spp,
                                SPO_MS_OFF,
                                -1 as ::core::ffi::c_int,
                            );
                            if pos.lnum > current_lnum.get() {
                                (*spp).sp_startcol = MAXCOL as ::core::ffi::c_int;
                            } else {
                                startcol = pos.col as ::core::ffi::c_int;
                                (*spp).sp_startcol = startcol;
                                if startcol >= next_match_col.get() {
                                    continue;
                                }
                                if did_match_already(idx, &raw mut zero_width_next_ga) {
                                    try_next_column.set(true_0 != 0);
                                } else {
                                    endpos.lnum =
                                        regmatch.endpos[0 as ::core::ffi::c_int as usize].lnum;
                                    endpos.col =
                                        regmatch.endpos[0 as ::core::ffi::c_int as usize].col;
                                    syn_add_start_off(
                                        &raw mut hl_startpos,
                                        &raw mut regmatch,
                                        spp,
                                        SPO_HS_OFF,
                                        -1 as ::core::ffi::c_int,
                                    );
                                    syn_add_end_off(
                                        &raw mut eos_pos,
                                        &raw mut regmatch,
                                        spp,
                                        SPO_RS_OFF,
                                        0 as ::core::ffi::c_int,
                                    );
                                    unref_extmatch(cur_extmatch);
                                    cur_extmatch = re_extmatch_out.get();
                                    re_extmatch_out.set(::core::ptr::null_mut::<reg_extmatch_T>());
                                    flags = 0 as ::core::ffi::c_int;
                                    eoe_pos.lnum = 0 as ::core::ffi::c_int as linenr_T;
                                    eoe_pos.col = 0 as ::core::ffi::c_int as colnr_T;
                                    end_idx = 0 as ::core::ffi::c_int;
                                    hl_endpos.lnum = 0 as ::core::ffi::c_int as linenr_T;
                                    if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_START
                                        && (*spp).sp_flags & HL_ONELINE != 0
                                    {
                                        let mut startpos: lpos_T = lpos_T { lnum: 0, col: 0 };
                                        startpos = endpos;
                                        find_endpos(
                                            idx,
                                            &raw mut startpos,
                                            &raw mut endpos,
                                            &raw mut hl_endpos,
                                            &raw mut flags,
                                            &raw mut eoe_pos,
                                            &raw mut end_idx,
                                            cur_extmatch,
                                        );
                                        if endpos.lnum == 0 as linenr_T {
                                            continue;
                                        }
                                    } else if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_MATCH {
                                        syn_add_end_off(
                                            &raw mut hl_endpos,
                                            &raw mut regmatch,
                                            spp,
                                            SPO_HE_OFF,
                                            0 as ::core::ffi::c_int,
                                        );
                                        syn_add_end_off(
                                            &raw mut endpos,
                                            &raw mut regmatch,
                                            spp,
                                            SPO_ME_OFF,
                                            0 as ::core::ffi::c_int,
                                        );
                                        if endpos.lnum == current_lnum.get()
                                            && (endpos.col + syncing as ::core::ffi::c_int)
                                                < startcol
                                        {
                                            if regmatch.startpos[0 as ::core::ffi::c_int as usize]
                                                .col
                                                == regmatch.endpos[0 as ::core::ffi::c_int as usize]
                                                    .col
                                            {
                                                try_next_column.set(true_0 != 0);
                                            }
                                            continue;
                                        }
                                    }
                                    if hl_startpos.lnum == current_lnum.get()
                                        && hl_startpos.col < startcol
                                    {
                                        hl_startpos.col = startcol as colnr_T;
                                    }
                                    limit_pos_zero(&raw mut hl_endpos, &raw mut endpos);
                                    next_match_idx.set(idx);
                                    next_match_col.set(startcol);
                                    next_match_m_endpos.set(endpos);
                                    next_match_h_endpos.set(hl_endpos);
                                    next_match_h_startpos.set(hl_startpos);
                                    next_match_flags.set(flags);
                                    next_match_eos_pos.set(eos_pos);
                                    next_match_eoe_pos.set(eoe_pos);
                                    next_match_end_idx.set(end_idx);
                                    unref_extmatch(next_match_extmatch.get());
                                    next_match_extmatch.set(cur_extmatch);
                                    cur_extmatch = ::core::ptr::null_mut::<reg_extmatch_T>();
                                }
                            }
                        }
                    }
                }
                if next_match_idx.get() >= 0 as ::core::ffi::c_int
                    && next_match_col.get() == current_col.get()
                {
                    let mut lspp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
                    lspp = ((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                        .offset(next_match_idx.get() as isize);
                    if (*next_match_m_endpos.ptr()).lnum == current_lnum.get()
                        && (*next_match_m_endpos.ptr()).col == current_col.get()
                        && !(*lspp).sp_next_list.is_null()
                    {
                        current_next_list.set((*lspp).sp_next_list);
                        current_next_flags.set((*lspp).sp_flags);
                        keep_next_list = true_0 != 0;
                        zero_width_next_list = true_0 != 0;
                        ga_grow(&raw mut zero_width_next_ga, 1 as ::core::ffi::c_int);
                        *(zero_width_next_ga.ga_data as *mut ::core::ffi::c_int)
                            .offset(zero_width_next_ga.ga_len as isize) = next_match_idx.get();
                        zero_width_next_ga.ga_len += 1;
                        next_match_idx.set(-1 as ::core::ffi::c_int);
                    } else {
                        cur_si = push_next_match();
                    }
                    found_match = true_0 != 0;
                }
            }
        }
        if !(*current_next_list.ptr()).is_null() && !keep_next_list {
            if !found_match {
                line = syn_getcurline();
                if current_next_flags.get() & HL_SKIPWHITE != 0
                    && ascii_iswhite(*line.offset(current_col.get() as isize) as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        != 0
                    || current_next_flags.get() & HL_SKIPEMPTY != 0
                        && *line as ::core::ffi::c_int == NUL
                {
                    break;
                }
            }
            current_next_list.set(::core::ptr::null_mut::<int16_t>());
            next_match_idx.set(-1 as ::core::ffi::c_int);
            if !zero_width_next_list {
                found_match = true_0 != 0;
            }
        }
        if !found_match {
            break;
        }
    }
    restore_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
    current_attr.set(0 as ::core::ffi::c_int);
    current_id.set(0 as ::core::ffi::c_int);
    current_trans_id.set(0 as ::core::ffi::c_int);
    current_flags.set(0 as ::core::ffi::c_int);
    current_seqnr.set(0 as ::core::ffi::c_int);
    if !cur_si.is_null() {
        let mut idx_0: ::core::ffi::c_int = (*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int;
        while idx_0 >= 0 as ::core::ffi::c_int {
            sip = ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(idx_0 as isize);
            if (current_lnum.get() > (*sip).si_h_startpos.lnum
                || current_lnum.get() == (*sip).si_h_startpos.lnum
                    && current_col.get() >= (*sip).si_h_startpos.col)
                && ((*sip).si_h_endpos.lnum == 0 as linenr_T
                    || current_lnum.get() < (*sip).si_h_endpos.lnum
                    || current_lnum.get() == (*sip).si_h_endpos.lnum
                        && current_col.get() < (*sip).si_h_endpos.col)
            {
                current_attr.set((*sip).si_attr);
                current_id.set((*sip).si_id);
                current_trans_id.set((*sip).si_trans_id);
                current_flags.set((*sip).si_flags);
                current_seqnr.set((*sip).si_seqnr);
                current_sub_char.set((*sip).si_cchar);
                break;
            } else {
                idx_0 -= 1;
            }
        }
        if !can_spell.is_null() {
            let mut sps: sp_syn = sp_syn {
                inc_tag: 0,
                id: 0,
                cont_in_list: ::core::ptr::null_mut::<int16_t>(),
            };
            if (*syn_block.get()).b_spell_cluster_id == 0 as ::core::ffi::c_int {
                if (*syn_block.get()).b_nospell_cluster_id == 0 as ::core::ffi::c_int
                    || current_trans_id.get() == 0 as ::core::ffi::c_int
                {
                    *can_spell = (*syn_block.get()).b_syn_spell != SYNSPL_NOTOP;
                } else {
                    sps.inc_tag = 0 as ::core::ffi::c_int;
                    sps.id = (*syn_block.get()).b_nospell_cluster_id as int16_t;
                    sps.cont_in_list = ::core::ptr::null_mut::<int16_t>();
                    *can_spell = in_id_list(
                        sip,
                        (*sip).si_cont_list,
                        &raw mut sps,
                        0 as ::core::ffi::c_int,
                    ) == 0;
                }
            } else if current_trans_id.get() == 0 as ::core::ffi::c_int {
                *can_spell = (*syn_block.get()).b_syn_spell == SYNSPL_TOP;
            } else {
                sps.inc_tag = 0 as ::core::ffi::c_int;
                sps.id = (*syn_block.get()).b_spell_cluster_id as int16_t;
                sps.cont_in_list = ::core::ptr::null_mut::<int16_t>();
                *can_spell = in_id_list(
                    sip,
                    (*sip).si_cont_list,
                    &raw mut sps,
                    0 as ::core::ffi::c_int,
                ) != 0;
                if (*syn_block.get()).b_nospell_cluster_id != 0 as ::core::ffi::c_int {
                    sps.id = (*syn_block.get()).b_nospell_cluster_id as int16_t;
                    if in_id_list(
                        sip,
                        (*sip).si_cont_list,
                        &raw mut sps,
                        0 as ::core::ffi::c_int,
                    ) != 0
                    {
                        *can_spell = false_0 != 0;
                    }
                }
            }
        }
        if !syncing && !keep_state {
            check_state_ends();
            if !((*current_state.ptr()).ga_len <= 0 as ::core::ffi::c_int)
                && *syn_getcurline().offset(current_col.get() as isize) as ::core::ffi::c_int != NUL
            {
                (*current_col.ptr()) += 1;
                check_state_ends();
                (*current_col.ptr()) -= 1;
            }
        }
    } else if !can_spell.is_null() {
        *can_spell = if (*syn_block.get()).b_syn_spell == SYNSPL_DEFAULT {
            ((*syn_block.get()).b_spell_cluster_id == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
        } else {
            ((*syn_block.get()).b_syn_spell == SYNSPL_TOP) as ::core::ffi::c_int
        } != 0;
    }
    if !(*current_next_list.ptr()).is_null()
        && {
            line = syn_getcurline();
            *line.offset(current_col.get() as isize) as ::core::ffi::c_int != NUL
        }
        && *line
            .offset((current_col.get() as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            == NUL
        && current_next_flags.get() & (HL_SKIPNL | HL_SKIPEMPTY) == 0
    {
        current_next_list.set(::core::ptr::null_mut::<int16_t>());
    }
    if !(zero_width_next_ga.ga_len <= 0 as ::core::ffi::c_int) {
        ga_clear(&raw mut zero_width_next_ga);
    }
    unref_extmatch(re_extmatch_out.get());
    re_extmatch_out.set(::core::ptr::null_mut::<reg_extmatch_T>());
    unref_extmatch(cur_extmatch);
    return current_attr.get();
}
unsafe extern "C" fn did_match_already(
    mut idx: ::core::ffi::c_int,
    mut gap: *mut garray_T,
) -> bool {
    let mut i: ::core::ffi::c_int = (*current_state.ptr()).ga_len;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        if (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_m_startcol
            == current_col.get()
            && (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_m_lnum
                == current_lnum.get() as ::core::ffi::c_int
            && (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_idx
                == idx
        {
            return true_0 != 0;
        }
    }
    let mut i_0: ::core::ffi::c_int = (*gap).ga_len;
    loop {
        i_0 -= 1;
        if i_0 < 0 as ::core::ffi::c_int {
            break;
        }
        if *((*gap).ga_data as *mut ::core::ffi::c_int).offset(i_0 as isize) == idx {
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
unsafe extern "C" fn push_next_match() -> *mut stateitem_T {
    let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
    let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
    let mut save_flags: ::core::ffi::c_int = 0;
    spp = ((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
        .offset(next_match_idx.get() as isize);
    push_current_state(next_match_idx.get());
    cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T)
        .offset(((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize);
    (*cur_si).si_h_startpos = next_match_h_startpos.get();
    (*cur_si).si_m_startcol = current_col.get() as ::core::ffi::c_int;
    (*cur_si).si_m_lnum = current_lnum.get() as ::core::ffi::c_int;
    (*cur_si).si_flags = (*spp).sp_flags;
    let c2rust_fresh4 = next_seqnr.get();
    next_seqnr.set(next_seqnr.get() + 1);
    (*cur_si).si_seqnr = c2rust_fresh4;
    (*cur_si).si_cchar = (*spp).sp_cchar;
    if (*current_state.ptr()).ga_len > 1 as ::core::ffi::c_int {
        (*cur_si).si_flags |= (*((*current_state.ptr()).ga_data as *mut stateitem_T)
            .offset(((*current_state.ptr()).ga_len - 2 as ::core::ffi::c_int) as isize))
        .si_flags
            & HL_CONCEAL;
    }
    (*cur_si).si_next_list = (*spp).sp_next_list;
    (*cur_si).si_extmatch = ref_extmatch(next_match_extmatch.get());
    if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_START && (*spp).sp_flags & HL_ONELINE == 0 {
        update_si_end(cur_si, (*next_match_m_endpos.ptr()).col, true_0 != 0);
        check_keepend();
    } else {
        (*cur_si).si_m_endpos = next_match_m_endpos.get();
        (*cur_si).si_h_endpos = next_match_h_endpos.get();
        (*cur_si).si_ends = true_0;
        (*cur_si).si_flags |= next_match_flags.get();
        (*cur_si).si_eoe_pos = next_match_eoe_pos.get();
        (*cur_si).si_end_idx = next_match_end_idx.get();
    }
    if keepend_level.get() < 0 as ::core::ffi::c_int && (*cur_si).si_flags & HL_KEEPEND != 0 {
        keepend_level.set((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int);
    }
    check_keepend();
    update_si_attr((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int);
    save_flags = (*cur_si).si_flags & (HL_CONCEAL | HL_CONCEALENDS);
    if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_START
        && (*spp).sp_syn_match_id as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        push_current_state(next_match_idx.get());
        cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T)
            .offset(((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize);
        (*cur_si).si_h_startpos = next_match_h_startpos.get();
        (*cur_si).si_m_startcol = current_col.get() as ::core::ffi::c_int;
        (*cur_si).si_m_lnum = current_lnum.get() as ::core::ffi::c_int;
        (*cur_si).si_m_endpos = next_match_eos_pos.get();
        (*cur_si).si_h_endpos = next_match_eos_pos.get();
        (*cur_si).si_ends = true_0;
        (*cur_si).si_end_idx = 0 as ::core::ffi::c_int;
        (*cur_si).si_flags = HL_MATCH;
        let c2rust_fresh5 = next_seqnr.get();
        next_seqnr.set(next_seqnr.get() + 1);
        (*cur_si).si_seqnr = c2rust_fresh5;
        (*cur_si).si_flags |= save_flags;
        if (*cur_si).si_flags & HL_CONCEALENDS != 0 {
            (*cur_si).si_flags |= HL_CONCEAL;
        }
        (*cur_si).si_next_list = ::core::ptr::null_mut::<int16_t>();
        check_keepend();
        update_si_attr((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int);
    }
    next_match_idx.set(-1 as ::core::ffi::c_int);
    return cur_si;
}
unsafe extern "C" fn check_state_ends() {
    let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
    let mut had_extend: ::core::ffi::c_int = 0;
    cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T)
        .offset(((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize);
    while (*cur_si).si_ends != 0
        && ((*cur_si).si_m_endpos.lnum < current_lnum.get()
            || (*cur_si).si_m_endpos.lnum == current_lnum.get()
                && (*cur_si).si_m_endpos.col <= current_col.get())
    {
        if (*cur_si).si_end_idx != 0
            && ((*cur_si).si_eoe_pos.lnum > current_lnum.get()
                || (*cur_si).si_eoe_pos.lnum == current_lnum.get()
                    && (*cur_si).si_eoe_pos.col > current_col.get())
        {
            (*cur_si).si_idx = (*cur_si).si_end_idx;
            (*cur_si).si_end_idx = 0 as ::core::ffi::c_int;
            (*cur_si).si_m_endpos = (*cur_si).si_eoe_pos;
            (*cur_si).si_h_endpos = (*cur_si).si_eoe_pos;
            (*cur_si).si_flags |= HL_MATCH;
            let c2rust_fresh0 = next_seqnr.get();
            next_seqnr.set(next_seqnr.get() + 1);
            (*cur_si).si_seqnr = c2rust_fresh0;
            if (*cur_si).si_flags & HL_CONCEALENDS != 0 {
                (*cur_si).si_flags |= HL_CONCEAL;
            }
            update_si_attr((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int);
            current_next_list.set(::core::ptr::null_mut::<int16_t>());
            next_match_idx.set(0 as ::core::ffi::c_int);
            next_match_col.set(MAXCOL as ::core::ffi::c_int);
            break;
        } else {
            current_next_list.set((*cur_si).si_next_list);
            current_next_flags.set((*cur_si).si_flags);
            if current_next_flags.get() & (HL_SKIPNL | HL_SKIPEMPTY) == 0
                && *syn_getcurline().offset(current_col.get() as isize) as ::core::ffi::c_int == NUL
            {
                current_next_list.set(::core::ptr::null_mut::<int16_t>());
            }
            had_extend = (*cur_si).si_flags & HL_EXTEND;
            pop_current_state();
            if (*current_state.ptr()).ga_len <= 0 as ::core::ffi::c_int {
                break;
            }
            if had_extend != 0 && keepend_level.get() >= 0 as ::core::ffi::c_int {
                syn_update_ends(false_0 != 0);
                if (*current_state.ptr()).ga_len <= 0 as ::core::ffi::c_int {
                    break;
                }
            }
            cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T)
                .offset(((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize);
            if !((*cur_si).si_idx >= 0 as ::core::ffi::c_int
                && (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                    .offset((*cur_si).si_idx as isize))
                .sp_type as ::core::ffi::c_int
                    == SPTYPE_START
                && (*cur_si).si_flags & (HL_MATCH | HL_KEEPEND) == 0)
            {
                continue;
            }
            update_si_end(cur_si, current_col.get(), true_0 != 0);
            check_keepend();
            if current_next_flags.get() & HL_HAS_EOL != 0
                && keepend_level.get() < 0 as ::core::ffi::c_int
                && *syn_getcurline().offset(current_col.get() as isize) as ::core::ffi::c_int == NUL
            {
                break;
            }
        }
    }
}
unsafe extern "C" fn update_si_attr(mut idx: ::core::ffi::c_int) {
    let mut sip: *mut stateitem_T =
        ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(idx as isize);
    let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
    if (*sip).si_idx < 0 as ::core::ffi::c_int {
        return;
    }
    spp =
        ((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T).offset((*sip).si_idx as isize);
    if (*sip).si_flags & HL_MATCH != 0 {
        (*sip).si_id = (*spp).sp_syn_match_id as ::core::ffi::c_int;
    } else {
        (*sip).si_id = (*spp).sp_syn.id as ::core::ffi::c_int;
    }
    (*sip).si_attr = syn_id2attr((*sip).si_id);
    (*sip).si_trans_id = (*sip).si_id;
    if (*sip).si_flags & HL_MATCH != 0 {
        (*sip).si_cont_list = ::core::ptr::null_mut::<int16_t>();
    } else {
        (*sip).si_cont_list = (*spp).sp_cont_list;
    }
    if (*spp).sp_flags & HL_TRANSP != 0 && (*sip).si_flags & HL_MATCH == 0 {
        if idx == 0 as ::core::ffi::c_int {
            (*sip).si_attr = 0 as ::core::ffi::c_int;
            (*sip).si_trans_id = 0 as ::core::ffi::c_int;
            if (*sip).si_cont_list.is_null() {
                (*sip).si_cont_list = ID_LIST_ALL;
            }
        } else {
            (*sip).si_attr = (*((*current_state.ptr()).ga_data as *mut stateitem_T)
                .offset((idx - 1 as ::core::ffi::c_int) as isize))
            .si_attr;
            (*sip).si_trans_id = (*((*current_state.ptr()).ga_data as *mut stateitem_T)
                .offset((idx - 1 as ::core::ffi::c_int) as isize))
            .si_trans_id;
            if (*sip).si_cont_list.is_null() {
                (*sip).si_flags |= HL_TRANS_CONT;
                (*sip).si_cont_list = (*((*current_state.ptr()).ga_data as *mut stateitem_T)
                    .offset((idx - 1 as ::core::ffi::c_int) as isize))
                .si_cont_list;
            }
        }
    }
}
unsafe extern "C" fn check_keepend() {
    let mut i: ::core::ffi::c_int = 0;
    let mut maxpos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut maxpos_h: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut sip: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
    if keepend_level.get() < 0 as ::core::ffi::c_int {
        return;
    }
    i = (*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int;
    while i > keepend_level.get() {
        if (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_flags
            & HL_EXTEND
            != 0
        {
            break;
        }
        i -= 1;
    }
    maxpos.lnum = 0 as ::core::ffi::c_int as linenr_T;
    maxpos.col = 0 as ::core::ffi::c_int as colnr_T;
    maxpos_h.lnum = 0 as ::core::ffi::c_int as linenr_T;
    maxpos_h.col = 0 as ::core::ffi::c_int as colnr_T;
    while i < (*current_state.ptr()).ga_len {
        sip = ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize);
        if maxpos.lnum != 0 as linenr_T {
            limit_pos_zero(&raw mut (*sip).si_m_endpos, &raw mut maxpos);
            limit_pos_zero(&raw mut (*sip).si_h_endpos, &raw mut maxpos_h);
            limit_pos_zero(&raw mut (*sip).si_eoe_pos, &raw mut maxpos);
            (*sip).si_ends = true_0;
        }
        if (*sip).si_ends != 0 && (*sip).si_flags & HL_KEEPEND != 0 {
            if maxpos.lnum == 0 as linenr_T
                || maxpos.lnum > (*sip).si_m_endpos.lnum
                || maxpos.lnum == (*sip).si_m_endpos.lnum && maxpos.col > (*sip).si_m_endpos.col
            {
                maxpos = (*sip).si_m_endpos;
            }
            if maxpos_h.lnum == 0 as linenr_T
                || maxpos_h.lnum > (*sip).si_h_endpos.lnum
                || maxpos_h.lnum == (*sip).si_h_endpos.lnum && maxpos_h.col > (*sip).si_h_endpos.col
            {
                maxpos_h = (*sip).si_h_endpos;
            }
        }
        i += 1;
    }
}
unsafe extern "C" fn update_si_end(
    mut sip: *mut stateitem_T,
    mut startcol: ::core::ffi::c_int,
    mut force: bool,
) {
    let mut hl_endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut end_endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
    if (*sip).si_idx < 0 as ::core::ffi::c_int {
        return;
    }
    if !force && (*sip).si_m_endpos.lnum >= current_lnum.get() {
        return;
    }
    let mut end_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut startpos: lpos_T = lpos_T {
        lnum: current_lnum.get(),
        col: startcol as colnr_T,
    };
    let mut endpos: lpos_T = lpos_T {
        lnum: 0 as linenr_T,
        col: 0,
    };
    find_endpos(
        (*sip).si_idx,
        &raw mut startpos,
        &raw mut endpos,
        &raw mut hl_endpos,
        &raw mut (*sip).si_flags,
        &raw mut end_endpos,
        &raw mut end_idx,
        (*sip).si_extmatch,
    );
    if endpos.lnum == 0 as linenr_T {
        if (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
            .offset((*sip).si_idx as isize))
        .sp_flags
            & HL_ONELINE
            != 0
        {
            (*sip).si_ends = true_0;
            (*sip).si_m_endpos.lnum = current_lnum.get();
            (*sip).si_m_endpos.col = syn_getcurline_len();
        } else {
            (*sip).si_ends = false_0;
            (*sip).si_m_endpos.lnum = 0 as ::core::ffi::c_int as linenr_T;
        }
        (*sip).si_h_endpos = (*sip).si_m_endpos;
    } else {
        (*sip).si_m_endpos = endpos;
        (*sip).si_h_endpos = hl_endpos;
        (*sip).si_eoe_pos = end_endpos;
        (*sip).si_ends = true_0;
        (*sip).si_end_idx = end_idx;
    };
}
unsafe extern "C" fn push_current_state(mut idx: ::core::ffi::c_int) {
    let mut p: *mut stateitem_T =
        ga_append_via_ptr(current_state.ptr(), ::core::mem::size_of::<stateitem_T>())
            as *mut stateitem_T;
    memset(
        p as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<stateitem_T>(),
    );
    (*p).si_idx = idx;
}
unsafe extern "C" fn pop_current_state() {
    if !((*current_state.ptr()).ga_len <= 0 as ::core::ffi::c_int) {
        unref_extmatch(
            (*((*current_state.ptr()).ga_data as *mut stateitem_T)
                .offset(((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .si_extmatch,
        );
        (*current_state.ptr()).ga_len -= 1;
    }
    next_match_idx.set(-1 as ::core::ffi::c_int);
    if keepend_level.get() >= (*current_state.ptr()).ga_len {
        keepend_level.set(-1 as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn find_endpos(
    mut idx: ::core::ffi::c_int,
    mut startpos: *mut lpos_T,
    mut m_endpos: *mut lpos_T,
    mut hl_endpos: *mut lpos_T,
    mut flagsp: *mut ::core::ffi::c_int,
    mut end_endpos: *mut lpos_T,
    mut end_idx: *mut ::core::ffi::c_int,
    mut start_ext: *mut reg_extmatch_T,
) {
    let mut spp_skip: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
    let mut best_idx: ::core::ffi::c_int = 0;
    let mut regmatch: regmmatch_T = regmmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    };
    let mut best_regmatch: regmmatch_T = regmmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    };
    let mut pos: lpos_T = lpos_T { lnum: 0, col: 0 };
    let mut had_match: bool = false_0 != 0;
    let mut buf_chartab: [::core::ffi::c_char; 32] = [0; 32];
    if idx < 0 as ::core::ffi::c_int {
        return;
    }
    let mut spp: *mut synpat_T =
        ((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
    if (*spp).sp_type as ::core::ffi::c_int != SPTYPE_START {
        *hl_endpos = *startpos;
        return;
    }
    loop {
        spp = ((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
        if (*spp).sp_type as ::core::ffi::c_int != SPTYPE_START {
            break;
        }
        idx += 1;
    }
    if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_SKIP {
        spp_skip = spp;
        idx += 1;
    } else {
        spp_skip = ::core::ptr::null_mut::<synpat_T>();
    }
    unref_extmatch(re_extmatch_in.get());
    re_extmatch_in.set(ref_extmatch(start_ext));
    let mut matchcol: colnr_T = (*startpos).col;
    let mut start_idx: ::core::ffi::c_int = idx;
    best_regmatch.startpos[0 as ::core::ffi::c_int as usize].col =
        0 as ::core::ffi::c_int as colnr_T;
    save_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
    loop {
        best_idx = -1 as ::core::ffi::c_int;
        idx = start_idx;
        while idx < (*syn_block.get()).b_syn_patterns.ga_len {
            let mut lc_col: ::core::ffi::c_int = matchcol as ::core::ffi::c_int;
            spp = ((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
            if (*spp).sp_type as ::core::ffi::c_int != SPTYPE_END {
                break;
            }
            lc_col -= (*spp).sp_offsets[SPO_LC_OFF as usize];
            if lc_col < 0 as ::core::ffi::c_int {
                lc_col = 0 as ::core::ffi::c_int;
            }
            regmatch.rmm_ic = (*spp).sp_ic;
            regmatch.regprog = (*spp).sp_prog;
            let mut r: bool = syn_regexec(
                &raw mut regmatch,
                (*startpos).lnum,
                lc_col as colnr_T,
                &raw mut (*spp).sp_time,
            );
            (*spp).sp_prog = regmatch.regprog;
            if r {
                if best_idx == -1 as ::core::ffi::c_int
                    || regmatch.startpos[0 as ::core::ffi::c_int as usize].col
                        < best_regmatch.startpos[0 as ::core::ffi::c_int as usize].col
                {
                    best_idx = idx;
                    best_regmatch.startpos[0 as ::core::ffi::c_int as usize] =
                        regmatch.startpos[0 as ::core::ffi::c_int as usize];
                    best_regmatch.endpos[0 as ::core::ffi::c_int as usize] =
                        regmatch.endpos[0 as ::core::ffi::c_int as usize];
                }
            }
            idx += 1;
        }
        if best_idx == -1 as ::core::ffi::c_int {
            break;
        }
        if !spp_skip.is_null() {
            let mut lc_col_0: ::core::ffi::c_int =
                matchcol as ::core::ffi::c_int - (*spp_skip).sp_offsets[SPO_LC_OFF as usize];
            if lc_col_0 < 0 as ::core::ffi::c_int {
                lc_col_0 = 0 as ::core::ffi::c_int;
            }
            regmatch.rmm_ic = (*spp_skip).sp_ic;
            regmatch.regprog = (*spp_skip).sp_prog;
            let mut r_0: ::core::ffi::c_int = syn_regexec(
                &raw mut regmatch,
                (*startpos).lnum,
                lc_col_0 as colnr_T,
                &raw mut (*spp_skip).sp_time,
            ) as ::core::ffi::c_int;
            (*spp_skip).sp_prog = regmatch.regprog;
            if r_0 != 0
                && regmatch.startpos[0 as ::core::ffi::c_int as usize].col
                    <= best_regmatch.startpos[0 as ::core::ffi::c_int as usize].col
            {
                syn_add_end_off(
                    &raw mut pos,
                    &raw mut regmatch,
                    spp_skip,
                    SPO_ME_OFF,
                    1 as ::core::ffi::c_int,
                );
                if pos.lnum > (*startpos).lnum {
                    break;
                }
                let mut line_len: ::core::ffi::c_int =
                    ml_get_buf_len(syn_buf.get(), (*startpos).lnum);
                if pos.col <= matchcol {
                    matchcol += 1;
                } else if pos.col <= regmatch.endpos[0 as ::core::ffi::c_int as usize].col {
                    matchcol = pos.col;
                } else {
                    matchcol = regmatch.endpos[0 as ::core::ffi::c_int as usize].col;
                    while matchcol < line_len && matchcol < pos.col {
                        matchcol += 1;
                    }
                }
                if matchcol >= line_len {
                    break;
                } else {
                    continue;
                }
            }
        }
        spp =
            ((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T).offset(best_idx as isize);
        syn_add_end_off(
            m_endpos,
            &raw mut best_regmatch,
            spp,
            SPO_ME_OFF,
            1 as ::core::ffi::c_int,
        );
        if (*m_endpos).lnum == (*startpos).lnum && (*m_endpos).col < (*startpos).col {
            (*m_endpos).col = (*startpos).col;
        }
        syn_add_end_off(
            end_endpos,
            &raw mut best_regmatch,
            spp,
            SPO_HE_OFF,
            1 as ::core::ffi::c_int,
        );
        if (*end_endpos).lnum == (*startpos).lnum && (*end_endpos).col < (*startpos).col {
            (*end_endpos).col = (*startpos).col;
        }
        limit_pos(end_endpos, m_endpos);
        if (*spp).sp_syn_match_id as ::core::ffi::c_int != (*spp).sp_syn.id as ::core::ffi::c_int
            && (*spp).sp_syn_match_id as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        {
            *end_idx = best_idx;
            if (*spp).sp_off_flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << SPO_RE_OFF + SPO_COUNT
                != 0
            {
                (*hl_endpos).lnum = best_regmatch.endpos[0 as ::core::ffi::c_int as usize].lnum;
                (*hl_endpos).col = best_regmatch.endpos[0 as ::core::ffi::c_int as usize].col;
            } else {
                (*hl_endpos).lnum = best_regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum;
                (*hl_endpos).col = best_regmatch.startpos[0 as ::core::ffi::c_int as usize].col;
            }
            (*hl_endpos).col += (*spp).sp_offsets[SPO_RE_OFF as usize];
            if (*hl_endpos).lnum == (*startpos).lnum && (*hl_endpos).col < (*startpos).col {
                (*hl_endpos).col = (*startpos).col;
            }
            limit_pos(hl_endpos, m_endpos);
            *m_endpos = *hl_endpos;
        } else {
            *end_idx = 0 as ::core::ffi::c_int;
            *hl_endpos = *end_endpos;
        }
        *flagsp = (*spp).sp_flags;
        had_match = true_0 != 0;
        break;
    }
    if !had_match {
        (*m_endpos).lnum = 0 as ::core::ffi::c_int as linenr_T;
    }
    restore_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
    unref_extmatch(re_extmatch_in.get());
    re_extmatch_in.set(::core::ptr::null_mut::<reg_extmatch_T>());
}
unsafe extern "C" fn limit_pos(mut pos: *mut lpos_T, mut limit: *mut lpos_T) {
    if (*pos).lnum > (*limit).lnum {
        *pos = *limit;
    } else if (*pos).lnum == (*limit).lnum && (*pos).col > (*limit).col {
        (*pos).col = (*limit).col;
    }
}
unsafe extern "C" fn limit_pos_zero(mut pos: *mut lpos_T, mut limit: *mut lpos_T) {
    if (*pos).lnum == 0 as linenr_T {
        *pos = *limit;
    } else {
        limit_pos(pos, limit);
    };
}
unsafe extern "C" fn syn_add_end_off(
    mut result: *mut lpos_T,
    mut regmatch: *mut regmmatch_T,
    mut spp: *mut synpat_T,
    mut idx: ::core::ffi::c_int,
    mut extra: ::core::ffi::c_int,
) {
    let mut col: ::core::ffi::c_int = 0;
    let mut off: ::core::ffi::c_int = 0;
    let mut base: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*spp).sp_off_flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << idx != 0 {
        (*result).lnum = (*regmatch).startpos[0 as ::core::ffi::c_int as usize].lnum;
        col = (*regmatch).startpos[0 as ::core::ffi::c_int as usize].col as ::core::ffi::c_int;
        off = (*spp).sp_offsets[idx as usize] + extra;
    } else {
        (*result).lnum = (*regmatch).endpos[0 as ::core::ffi::c_int as usize].lnum;
        col = (*regmatch).endpos[0 as ::core::ffi::c_int as usize].col as ::core::ffi::c_int;
        off = (*spp).sp_offsets[idx as usize];
    }
    if (*result).lnum > (*syn_buf.get()).b_ml.ml_line_count {
        col = 0 as ::core::ffi::c_int;
    } else if off != 0 as ::core::ffi::c_int {
        base = ml_get_buf(syn_buf.get(), (*result).lnum);
        p = base.offset(col as isize);
        if off > 0 as ::core::ffi::c_int {
            loop {
                let c2rust_fresh1 = off;
                off = off - 1;
                if !(c2rust_fresh1 > 0 as ::core::ffi::c_int && *p as ::core::ffi::c_int != NUL) {
                    break;
                }
                p = p.offset(utfc_ptr2len(p) as isize);
            }
        } else {
            loop {
                let c2rust_fresh2 = off;
                off = off + 1;
                if !(c2rust_fresh2 < 0 as ::core::ffi::c_int && base < p) {
                    break;
                }
                p = p.offset(
                    -((utf_head_off(base, p.offset(-(1 as ::core::ffi::c_int as isize)))
                        + 1 as ::core::ffi::c_int) as isize),
                );
            }
        }
        col = p.offset_from(base) as ::core::ffi::c_int;
    }
    (*result).col = col as colnr_T;
}
unsafe extern "C" fn syn_add_start_off(
    mut result: *mut lpos_T,
    mut regmatch: *mut regmmatch_T,
    mut spp: *mut synpat_T,
    mut idx: ::core::ffi::c_int,
    mut extra: ::core::ffi::c_int,
) {
    let mut col: ::core::ffi::c_int = 0;
    let mut off: ::core::ffi::c_int = 0;
    let mut base: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*spp).sp_off_flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << idx + SPO_COUNT != 0
    {
        (*result).lnum = (*regmatch).endpos[0 as ::core::ffi::c_int as usize].lnum;
        col = (*regmatch).endpos[0 as ::core::ffi::c_int as usize].col as ::core::ffi::c_int;
        off = (*spp).sp_offsets[idx as usize] + extra;
    } else {
        (*result).lnum = (*regmatch).startpos[0 as ::core::ffi::c_int as usize].lnum;
        col = (*regmatch).startpos[0 as ::core::ffi::c_int as usize].col as ::core::ffi::c_int;
        off = (*spp).sp_offsets[idx as usize];
    }
    if (*result).lnum > (*syn_buf.get()).b_ml.ml_line_count {
        (*result).lnum = (*syn_buf.get()).b_ml.ml_line_count;
        col = ml_get_buf_len(syn_buf.get(), (*result).lnum) as ::core::ffi::c_int;
    }
    if off != 0 as ::core::ffi::c_int {
        base = ml_get_buf(syn_buf.get(), (*result).lnum);
        p = base.offset(col as isize);
        if off > 0 as ::core::ffi::c_int {
            loop {
                let c2rust_fresh6 = off;
                off = off - 1;
                if !(c2rust_fresh6 != 0 && *p as ::core::ffi::c_int != NUL) {
                    break;
                }
                p = p.offset(utfc_ptr2len(p) as isize);
            }
        } else {
            loop {
                let c2rust_fresh7 = off;
                off = off + 1;
                if !(c2rust_fresh7 != 0 && base < p) {
                    break;
                }
                p = p.offset(
                    -((utf_head_off(base, p.offset(-(1 as ::core::ffi::c_int as isize)))
                        + 1 as ::core::ffi::c_int) as isize),
                );
            }
        }
        col = p.offset_from(base) as ::core::ffi::c_int;
    }
    (*result).col = col as colnr_T;
}
unsafe extern "C" fn syn_getcurline() -> *mut ::core::ffi::c_char {
    return ml_get_buf(syn_buf.get(), current_lnum.get());
}
unsafe extern "C" fn syn_getcurline_len() -> colnr_T {
    return ml_get_buf_len(syn_buf.get(), current_lnum.get());
}
unsafe extern "C" fn syn_regexec(
    mut rmp: *mut regmmatch_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut st: *mut syn_time_T,
) -> bool {
    let mut timed_out: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut pt: proftime_T = 0;
    let l_syn_time_on: bool = syn_time_on.get();
    if l_syn_time_on {
        pt = profile_start();
    }
    if (*rmp).regprog.is_null() {
        return false_0 != 0;
    }
    (*rmp).rmm_maxcol = (*syn_buf.get()).b_p_smc as colnr_T;
    let mut r: ::core::ffi::c_int = vim_regexec_multi(
        rmp,
        syn_win.get(),
        syn_buf.get(),
        lnum,
        col,
        syn_tm.get(),
        &raw mut timed_out,
    );
    if l_syn_time_on {
        pt = profile_end(pt);
        (*st).total = profile_add((*st).total, pt);
        if profile_cmp(pt, (*st).slowest) < 0 as ::core::ffi::c_int {
            (*st).slowest = pt;
        }
        (*st).count += 1;
        if r > 0 as ::core::ffi::c_int {
            (*st).match_0 += 1;
        }
    }
    if timed_out != 0 && !(*(*syn_win.get()).w_s).b_syn_slow {
        (*(*syn_win.get()).w_s).b_syn_slow = true_0 != 0;
        msg(
            gettext(
                b"'redrawtime' exceeded, syntax highlighting disabled\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            0 as ::core::ffi::c_int,
        );
    }
    if r > 0 as ::core::ffi::c_int {
        (*rmp).startpos[0 as ::core::ffi::c_int as usize].lnum += lnum;
        (*rmp).endpos[0 as ::core::ffi::c_int as usize].lnum += lnum;
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn check_keyword_id(
    line: *mut ::core::ffi::c_char,
    startcol: ::core::ffi::c_int,
    endcolp: *mut ::core::ffi::c_int,
    flagsp: *mut ::core::ffi::c_int,
    next_listp: *mut *mut int16_t,
    cur_si: *mut stateitem_T,
    ccharp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let kwp: *mut ::core::ffi::c_char = line.offset(startcol as isize);
    let mut kwlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        kwlen += utfc_ptr2len(kwp.offset(kwlen as isize));
        if !vim_iswordp_buf(kwp.offset(kwlen as isize), syn_buf.get()) {
            break;
        }
    }
    if kwlen > MAXKEYWLEN {
        return 0 as ::core::ffi::c_int;
    }
    let mut keyword: [::core::ffi::c_char; 81] = [0; 81];
    xmemcpyz(
        &raw mut keyword as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        kwp as *const ::core::ffi::c_void,
        kwlen as size_t,
    );
    let mut kp: *mut keyentry_T = ::core::ptr::null_mut::<keyentry_T>();
    if (*syn_block.get()).b_keywtab.ht_used != 0 as size_t {
        kp = match_keyword(
            &raw mut keyword as *mut ::core::ffi::c_char,
            &raw mut (*syn_block.get()).b_keywtab,
            cur_si,
        );
    }
    if kp.is_null() && (*syn_block.get()).b_keywtab_ic.ht_used != 0 as size_t {
        str_foldcase(
            kwp,
            kwlen,
            &raw mut keyword as *mut ::core::ffi::c_char,
            MAXKEYWLEN + 1 as ::core::ffi::c_int,
        );
        kp = match_keyword(
            &raw mut keyword as *mut ::core::ffi::c_char,
            &raw mut (*syn_block.get()).b_keywtab_ic,
            cur_si,
        );
    }
    if !kp.is_null() {
        *endcolp = startcol + kwlen;
        *flagsp = (*kp).flags;
        *next_listp = (*kp).next_list;
        *ccharp = (*kp).k_char;
        return (*kp).k_syn.id as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn match_keyword(
    mut keyword: *mut ::core::ffi::c_char,
    mut ht: *mut hashtab_T,
    mut cur_si: *mut stateitem_T,
) -> *mut keyentry_T {
    let mut hi: *mut hashitem_T = hash_find(ht, keyword);
    if !((*hi).hi_key.is_null()
        || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
    {
        let mut kp: *mut keyentry_T = (*hi).hi_key.offset(
            -((&raw mut (*dumkey.ptr()).keyword as *mut ::core::ffi::c_char)
                .offset_from(dumkey.ptr() as *mut ::core::ffi::c_char) as isize),
        ) as *mut keyentry_T;
        while !kp.is_null() {
            if if !(*current_next_list.ptr()).is_null() {
                in_id_list(
                    ::core::ptr::null_mut::<stateitem_T>(),
                    current_next_list.get(),
                    &raw mut (*kp).k_syn,
                    0 as ::core::ffi::c_int,
                )
            } else if cur_si.is_null() {
                ((*kp).flags & HL_CONTAINED == 0) as ::core::ffi::c_int
            } else {
                in_id_list(
                    cur_si,
                    (*cur_si).si_cont_list,
                    &raw mut (*kp).k_syn,
                    (*kp).flags,
                )
            } != 0
            {
                return kp;
            }
            kp = (*kp).ke_next;
        }
    }
    return ::core::ptr::null_mut::<keyentry_T>();
}
unsafe extern "C" fn syn_cmd_conceal(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*eap).nextcmd = find_nextcmd(arg);
    if (*eap).skip != 0 {
        return;
    }
    next = skiptowhite(arg);
    if *arg as ::core::ffi::c_int == NUL {
        if (*(*curwin.get()).w_s).b_syn_conceal != 0 {
            msg(
                b"syntax conceal on\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        } else {
            msg(
                b"syntax conceal off\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        }
    } else if strncasecmp(
        arg,
        b"on\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        2 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && next.offset_from(arg) == 2 as isize
    {
        (*(*curwin.get()).w_s).b_syn_conceal = true_0;
    } else if strncasecmp(
        arg,
        b"off\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        3 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && next.offset_from(arg) == 3 as isize
    {
        (*(*curwin.get()).w_s).b_syn_conceal = false_0;
    } else {
        semsg(
            gettext((e_illegal_arg.ptr() as *const _) as *const ::core::ffi::c_char),
            arg,
        );
    };
}
unsafe extern "C" fn syn_cmd_case(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*eap).nextcmd = find_nextcmd(arg);
    if (*eap).skip != 0 {
        return;
    }
    next = skiptowhite(arg);
    if *arg as ::core::ffi::c_int == NUL {
        if (*(*curwin.get()).w_s).b_syn_ic != 0 {
            msg(
                b"syntax case ignore\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        } else {
            msg(
                b"syntax case match\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        }
    } else if strncasecmp(
        arg,
        b"match\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        5 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && next.offset_from(arg) == 5 as isize
    {
        (*(*curwin.get()).w_s).b_syn_ic = false_0;
    } else if strncasecmp(
        arg,
        b"ignore\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        6 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && next.offset_from(arg) == 6 as isize
    {
        (*(*curwin.get()).w_s).b_syn_ic = true_0;
    } else {
        semsg(
            gettext((e_illegal_arg.ptr() as *const _) as *const ::core::ffi::c_char),
            arg,
        );
    };
}
unsafe extern "C" fn syn_cmd_foldlevel(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut arg_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*eap).nextcmd = find_nextcmd(arg);
    if (*eap).skip != 0 {
        return;
    }
    if *arg as ::core::ffi::c_int == NUL {
        match (*(*curwin.get()).w_s).b_syn_foldlevel {
            SYNFLD_START => {
                msg(
                    b"syntax foldlevel start\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
            }
            SYNFLD_MINIMUM => {
                msg(
                    b"syntax foldlevel minimum\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
            }
            _ => {}
        }
        return;
    }
    arg_end = skiptowhite(arg);
    if strncasecmp(
        arg,
        b"start\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        5 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && arg_end.offset_from(arg) == 5 as isize
    {
        (*(*curwin.get()).w_s).b_syn_foldlevel = SYNFLD_START;
    } else if strncasecmp(
        arg,
        b"minimum\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        7 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && arg_end.offset_from(arg) == 7 as isize
    {
        (*(*curwin.get()).w_s).b_syn_foldlevel = SYNFLD_MINIMUM;
    } else {
        semsg(
            gettext((e_illegal_arg.ptr() as *const _) as *const ::core::ffi::c_char),
            arg,
        );
        return;
    }
    arg = skipwhite(arg_end);
    if *arg as ::core::ffi::c_int != NUL {
        semsg(
            gettext((e_illegal_arg.ptr() as *const _) as *const ::core::ffi::c_char),
            arg,
        );
    }
}
unsafe extern "C" fn syn_cmd_spell(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*eap).nextcmd = find_nextcmd(arg);
    if (*eap).skip != 0 {
        return;
    }
    next = skiptowhite(arg);
    if *arg as ::core::ffi::c_int == NUL {
        if (*(*curwin.get()).w_s).b_syn_spell == SYNSPL_TOP {
            msg(
                b"syntax spell toplevel\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        } else if (*(*curwin.get()).w_s).b_syn_spell == SYNSPL_NOTOP {
            msg(
                b"syntax spell notoplevel\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        } else {
            msg(
                b"syntax spell default\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        }
    } else if strncasecmp(
        arg,
        b"toplevel\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        8 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && next.offset_from(arg) == 8 as isize
    {
        (*(*curwin.get()).w_s).b_syn_spell = SYNSPL_TOP;
    } else if strncasecmp(
        arg,
        b"notoplevel\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        10 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && next.offset_from(arg) == 10 as isize
    {
        (*(*curwin.get()).w_s).b_syn_spell = SYNSPL_NOTOP;
    } else if strncasecmp(
        arg,
        b"default\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        7 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && next.offset_from(arg) == 7 as isize
    {
        (*(*curwin.get()).w_s).b_syn_spell = SYNSPL_DEFAULT;
    } else {
        semsg(
            gettext((e_illegal_arg.ptr() as *const _) as *const ::core::ffi::c_char),
            arg,
        );
        return;
    }
    redraw_later(curwin.get(), UPD_NOT_VALID);
}
unsafe extern "C" fn syn_cmd_iskeyword(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut save_chartab_0: [::core::ffi::c_char; 32] = [0; 32];
    let mut save_isk: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*eap).skip != 0 {
        return;
    }
    arg = skipwhite(arg);
    if *arg as ::core::ffi::c_int == NUL {
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        if (*(*curwin.get()).w_s).b_syn_isk != empty_string_option.ptr() as *mut ::core::ffi::c_char
        {
            msg_puts(b"syntax iskeyword \0".as_ptr() as *const ::core::ffi::c_char);
            msg_outtrans(
                (*(*curwin.get()).w_s).b_syn_isk,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        } else {
            msg_outtrans(
                gettext(b"syntax iskeyword not set\0".as_ptr() as *const ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        }
    } else if strncasecmp(
        arg,
        b"clear\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        5 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        memmove(
            &raw mut (*(*curwin.get()).w_s).b_syn_chartab as *mut uint8_t
                as *mut ::core::ffi::c_void,
            &raw mut (*curbuf.get()).b_chartab as *mut uint64_t as *const ::core::ffi::c_void,
            32 as ::core::ffi::c_int as size_t,
        );
        clear_string_option(&raw mut (*(*curwin.get()).w_s).b_syn_isk);
    } else {
        memmove(
            &raw mut save_chartab_0 as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            &raw mut (*curbuf.get()).b_chartab as *mut uint64_t as *const ::core::ffi::c_void,
            32 as ::core::ffi::c_int as size_t,
        );
        save_isk = (*curbuf.get()).b_p_isk;
        (*curbuf.get()).b_p_isk = xstrdup(arg);
        buf_init_chartab(curbuf.get(), false);
        memmove(
            &raw mut (*(*curwin.get()).w_s).b_syn_chartab as *mut uint8_t
                as *mut ::core::ffi::c_void,
            &raw mut (*curbuf.get()).b_chartab as *mut uint64_t as *const ::core::ffi::c_void,
            32 as ::core::ffi::c_int as size_t,
        );
        memmove(
            &raw mut (*curbuf.get()).b_chartab as *mut uint64_t as *mut ::core::ffi::c_void,
            &raw mut save_chartab_0 as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            32 as ::core::ffi::c_int as size_t,
        );
        clear_string_option(&raw mut (*(*curwin.get()).w_s).b_syn_isk);
        (*(*curwin.get()).w_s).b_syn_isk = (*curbuf.get()).b_p_isk;
        (*curbuf.get()).b_p_isk = save_isk;
    }
    redraw_later(curwin.get(), UPD_NOT_VALID);
}
pub unsafe extern "C" fn syntax_clear(mut block: *mut synblock_T) {
    (*block).b_syn_error = false_0 != 0;
    (*block).b_syn_slow = false_0 != 0;
    (*block).b_syn_ic = false_0;
    (*block).b_syn_foldlevel = SYNFLD_START;
    (*block).b_syn_spell = SYNSPL_DEFAULT;
    (*block).b_syn_containedin = false_0;
    (*block).b_syn_conceal = false_0;
    clear_keywtab(&raw mut (*block).b_keywtab);
    clear_keywtab(&raw mut (*block).b_keywtab_ic);
    let mut i: ::core::ffi::c_int = (*block).b_syn_patterns.ga_len;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        syn_clear_pattern(block, i);
    }
    ga_clear(&raw mut (*block).b_syn_patterns);
    let mut i_0: ::core::ffi::c_int = (*block).b_syn_clusters.ga_len;
    loop {
        i_0 -= 1;
        if i_0 < 0 as ::core::ffi::c_int {
            break;
        }
        syn_clear_cluster(block, i_0);
    }
    ga_clear(&raw mut (*block).b_syn_clusters);
    (*block).b_spell_cluster_id = 0 as ::core::ffi::c_int;
    (*block).b_nospell_cluster_id = 0 as ::core::ffi::c_int;
    (*block).b_syn_sync_flags = 0 as ::core::ffi::c_int;
    (*block).b_syn_sync_minlines = 0 as ::core::ffi::c_int as linenr_T;
    (*block).b_syn_sync_maxlines = 0 as ::core::ffi::c_int as linenr_T;
    (*block).b_syn_sync_linebreaks = 0 as ::core::ffi::c_int as linenr_T;
    vim_regfree((*block).b_syn_linecont_prog);
    (*block).b_syn_linecont_prog = ::core::ptr::null_mut::<regprog_T>();
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*block).b_syn_linecont_pat as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    (*block).b_syn_folditems = 0 as ::core::ffi::c_int;
    clear_string_option(&raw mut (*block).b_syn_isk);
    syn_stack_free_all(block);
    invalidate_current_state();
    running_syn_inc_tag.set(0 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn reset_synblock(mut wp: *mut win_T) {
    if (*wp).w_s != &raw mut (*(*wp).w_buffer).b_s {
        syntax_clear((*wp).w_s);
        xfree((*wp).w_s as *mut ::core::ffi::c_void);
        (*wp).w_s = &raw mut (*(*wp).w_buffer).b_s;
    }
}
unsafe extern "C" fn syntax_sync_clear() {
    let mut i: ::core::ffi::c_int = (*(*curwin.get()).w_s).b_syn_patterns.ga_len;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        if (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T).offset(i as isize))
            .sp_syncing
        {
            syn_remove_pattern((*curwin.get()).w_s, i);
        }
    }
    (*(*curwin.get()).w_s).b_syn_sync_flags = 0 as ::core::ffi::c_int;
    (*(*curwin.get()).w_s).b_syn_sync_minlines = 0 as ::core::ffi::c_int as linenr_T;
    (*(*curwin.get()).w_s).b_syn_sync_maxlines = 0 as ::core::ffi::c_int as linenr_T;
    (*(*curwin.get()).w_s).b_syn_sync_linebreaks = 0 as ::core::ffi::c_int as linenr_T;
    vim_regfree((*(*curwin.get()).w_s).b_syn_linecont_prog);
    (*(*curwin.get()).w_s).b_syn_linecont_prog = ::core::ptr::null_mut::<regprog_T>();
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*(*curwin.get()).w_s).b_syn_linecont_pat as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    clear_string_option(&raw mut (*(*curwin.get()).w_s).b_syn_isk);
    syn_stack_free_all((*curwin.get()).w_s);
}
unsafe extern "C" fn syn_remove_pattern(mut block: *mut synblock_T, mut idx: ::core::ffi::c_int) {
    let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
    spp = ((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
    if (*spp).sp_flags & HL_FOLD != 0 {
        (*block).b_syn_folditems -= 1;
    }
    syn_clear_pattern(block, idx);
    memmove(
        spp as *mut ::core::ffi::c_void,
        spp.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        ::core::mem::size_of::<synpat_T>().wrapping_mul(
            ((*block).b_syn_patterns.ga_len - idx - 1 as ::core::ffi::c_int) as size_t,
        ),
    );
    (*block).b_syn_patterns.ga_len -= 1;
}
unsafe extern "C" fn syn_clear_pattern(mut block: *mut synblock_T, mut i: ::core::ffi::c_int) {
    xfree(
        (*((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(i as isize)).sp_pattern
            as *mut ::core::ffi::c_void,
    );
    vim_regfree((*((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(i as isize)).sp_prog);
    if i == 0 as ::core::ffi::c_int
        || (*((*block).b_syn_patterns.ga_data as *mut synpat_T)
            .offset((i - 1 as ::core::ffi::c_int) as isize))
        .sp_type as ::core::ffi::c_int
            != SPTYPE_START
    {
        xfree(
            (*((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(i as isize)).sp_cont_list
                as *mut ::core::ffi::c_void,
        );
        xfree(
            (*((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(i as isize)).sp_next_list
                as *mut ::core::ffi::c_void,
        );
        xfree(
            (*((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(i as isize))
                .sp_syn
                .cont_in_list as *mut ::core::ffi::c_void,
        );
    }
}
unsafe extern "C" fn syn_clear_cluster(mut block: *mut synblock_T, mut i: ::core::ffi::c_int) {
    xfree(
        (*((*block).b_syn_clusters.ga_data as *mut syn_cluster_T).offset(i as isize)).scl_name
            as *mut ::core::ffi::c_void,
    );
    xfree(
        (*((*block).b_syn_clusters.ga_data as *mut syn_cluster_T).offset(i as isize)).scl_name_u
            as *mut ::core::ffi::c_void,
    );
    xfree(
        (*((*block).b_syn_clusters.ga_data as *mut syn_cluster_T).offset(i as isize)).scl_list
            as *mut ::core::ffi::c_void,
    );
}
unsafe extern "C" fn syn_cmd_clear(mut eap: *mut exarg_T, mut syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut arg_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut id: ::core::ffi::c_int = 0;
    (*eap).nextcmd = find_nextcmd(arg);
    if (*eap).skip != 0 {
        return;
    }
    if (*(*curwin.get()).w_s).b_syn_topgrp != 0 as ::core::ffi::c_int {
        return;
    }
    if ends_excmd(*arg as ::core::ffi::c_int) != 0 {
        if syncing != 0 {
            syntax_sync_clear();
        } else {
            syntax_clear((*curwin.get()).w_s);
            if (*curwin.get()).w_s == &raw mut (*(*curwin.get()).w_buffer).b_s {
                do_unlet(
                    b"b:current_syntax\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
                    true_0 != 0,
                );
            }
            do_unlet(
                b"w:current_syntax\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
                true_0 != 0,
            );
        }
    } else {
        while ends_excmd(*arg as ::core::ffi::c_int) == 0 {
            arg_end = skiptowhite(arg);
            if *arg as ::core::ffi::c_int == '@' as ::core::ffi::c_int {
                id = syn_scl_namen2id(
                    arg.offset(1 as ::core::ffi::c_int as isize),
                    (arg_end.offset_from(arg) - 1 as isize) as ::core::ffi::c_int,
                );
                if id == 0 as ::core::ffi::c_int {
                    semsg(
                        gettext(b"E391: No such syntax cluster: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        arg,
                    );
                    break;
                } else {
                    let mut scl_id: ::core::ffi::c_int = id - SYNID_CLUSTER;
                    let mut ptr_: *mut *mut ::core::ffi::c_void =
                        &raw mut (*((*(*curwin.get()).w_s).b_syn_clusters.ga_data
                            as *mut syn_cluster_T)
                            .offset(scl_id as isize))
                        .scl_list as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL;
                    let _ = *ptr_;
                }
            } else {
                id = syn_name2id_len(arg, arg_end.offset_from(arg) as size_t);
                if id == 0 as ::core::ffi::c_int {
                    semsg(
                        gettext(&raw const e_nogroup as *const ::core::ffi::c_char),
                        arg,
                    );
                    break;
                } else {
                    syn_clear_one(id, syncing != 0);
                }
            }
            arg = skipwhite(arg_end);
        }
    }
    redraw_curbuf_later(UPD_SOME_VALID);
    syn_stack_free_all((*curwin.get()).w_s);
}
unsafe extern "C" fn syn_clear_one(id: ::core::ffi::c_int, syncing: bool) {
    let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
    if !syncing {
        syn_clear_keyword(id, &raw mut (*(*curwin.get()).w_s).b_keywtab);
        syn_clear_keyword(id, &raw mut (*(*curwin.get()).w_s).b_keywtab_ic);
    }
    let mut idx: ::core::ffi::c_int = (*(*curwin.get()).w_s).b_syn_patterns.ga_len;
    loop {
        idx -= 1;
        if idx < 0 as ::core::ffi::c_int {
            break;
        }
        spp = ((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
        if (*spp).sp_syn.id as ::core::ffi::c_int != id
            || (*spp).sp_syncing as ::core::ffi::c_int != syncing as ::core::ffi::c_int
        {
            continue;
        }
        syn_remove_pattern((*curwin.get()).w_s, idx);
    }
}
unsafe extern "C" fn syn_cmd_on(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    syn_cmd_onoff(
        eap,
        b"syntax\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
}
unsafe extern "C" fn syn_cmd_reset(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    (*eap).nextcmd = check_nextcmd((*eap).arg);
    if (*eap).skip == 0 {
        init_highlight(true_0 != 0, true_0 != 0);
    }
}
unsafe extern "C" fn syn_cmd_manual(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    syn_cmd_onoff(
        eap,
        b"manual\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
}
unsafe extern "C" fn syn_cmd_off(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    syn_cmd_onoff(
        eap,
        b"nosyntax\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
}
unsafe extern "C" fn syn_cmd_onoff(mut eap: *mut exarg_T, mut name: *mut ::core::ffi::c_char) {
    (*eap).nextcmd = check_nextcmd((*eap).arg);
    if (*eap).skip == 0 {
        did_syntax_onoff.set(true_0 != 0);
        let mut buf: [::core::ffi::c_char; 100] = [0; 100];
        memcpy(
            &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            b"so \0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            4 as size_t,
        );
        vim_snprintf(
            (&raw mut buf as *mut ::core::ffi::c_char).offset(3 as ::core::ffi::c_int as isize),
            ::core::mem::size_of::<[::core::ffi::c_char; 100]>().wrapping_sub(3 as size_t),
            SYNTAX_FNAME.as_ptr(),
            name,
        );
        do_cmdline_cmd(&raw mut buf as *mut ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn syn_maybe_enable() {
    if !did_syntax_onoff.get() {
        let mut ea: exarg_T = exarg_T {
            arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            arglens: ::core::ptr::null_mut::<size_t>(),
            argc: 0,
            nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdidx: CMD_append,
            argt: 0,
            skip: 0,
            forceit: 0,
            addr_count: 0,
            line1: 0,
            line2: 0,
            addr_type: ADDR_LINES,
            flags: 0,
            do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            do_ecmd_lnum: 0,
            append: 0,
            usefilter: 0,
            amount: 0,
            regname: 0,
            force_bin: 0,
            read_edit: 0,
            mkdir_p: 0,
            force_ff: 0,
            force_enc: 0,
            bad_char: 0,
            useridx: 0,
            errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ea_getline: None,
            cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            cstack: ::core::ptr::null_mut::<cstack_T>(),
        };
        ea.arg = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        ea.skip = false_0;
        syn_cmd_on(&raw mut ea, false_0);
    }
}
unsafe extern "C" fn syn_cmd_list(mut eap: *mut exarg_T, mut syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut arg_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*eap).nextcmd = find_nextcmd(arg);
    if (*eap).skip != 0 {
        return;
    }
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    if !syntax_present(curwin.get()) {
        msg(
            gettext(msg_no_items.ptr() as *mut ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
        return;
    }
    if syncing != 0 {
        if (*(*curwin.get()).w_s).b_syn_sync_flags & SF_CCOMMENT != 0 {
            msg_puts(gettext(
                b"syncing on C-style comments\0".as_ptr() as *const ::core::ffi::c_char
            ));
            syn_lines_msg();
            syn_match_msg();
        } else if (*(*curwin.get()).w_s).b_syn_sync_flags & SF_MATCH != 0 {
            msg_puts_title(gettext(
                b"\n--- Syntax sync items ---\0".as_ptr() as *const ::core::ffi::c_char
            ));
            if (*(*curwin.get()).w_s).b_syn_sync_minlines > 0 as linenr_T
                || (*(*curwin.get()).w_s).b_syn_sync_maxlines > 0 as linenr_T
                || (*(*curwin.get()).w_s).b_syn_sync_linebreaks > 0 as linenr_T
            {
                msg_puts(gettext(
                    b"\nsyncing on items\0".as_ptr() as *const ::core::ffi::c_char
                ));
                syn_lines_msg();
                syn_match_msg();
            }
            let mut id: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while id <= highlight_num_groups() && !got_int.get() {
                syn_list_one(id, syncing != 0, false_0 != 0);
                id += 1;
            }
        } else if (*(*curwin.get()).w_s).b_syn_sync_minlines == 0 as linenr_T {
            msg_puts(gettext(
                b"no syncing\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else {
            if (*(*curwin.get()).w_s).b_syn_sync_minlines
                == MAXLNUM as ::core::ffi::c_int as linenr_T
            {
                msg_puts(gettext(
                    b"syncing starts at the first line\0".as_ptr() as *const ::core::ffi::c_char
                ));
            } else {
                msg_puts(gettext(
                    b"syncing starts \0".as_ptr() as *const ::core::ffi::c_char
                ));
                msg_outnum((*(*curwin.get()).w_s).b_syn_sync_minlines as ::core::ffi::c_int);
                msg_puts(gettext(
                    b" lines before top line\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
            syn_match_msg();
        }
        return;
    }
    msg_puts_title(gettext(
        b"\n--- Syntax items ---\0".as_ptr() as *const ::core::ffi::c_char
    ));
    if ends_excmd(*arg as ::core::ffi::c_int) != 0 {
        let mut id_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while id_0 <= highlight_num_groups() && !got_int.get() {
            syn_list_one(id_0, syncing != 0, false_0 != 0);
            id_0 += 1;
        }
        let mut id_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while id_1 < (*(*curwin.get()).w_s).b_syn_clusters.ga_len && !got_int.get() {
            syn_list_cluster(id_1);
            id_1 += 1;
        }
    } else {
        while ends_excmd(*arg as ::core::ffi::c_int) == 0 && !got_int.get() {
            arg_end = skiptowhite(arg);
            if *arg as ::core::ffi::c_int == '@' as ::core::ffi::c_int {
                let mut id_2: ::core::ffi::c_int = syn_scl_namen2id(
                    arg.offset(1 as ::core::ffi::c_int as isize),
                    (arg_end.offset_from(arg) - 1 as isize) as ::core::ffi::c_int,
                );
                if id_2 == 0 as ::core::ffi::c_int {
                    semsg(
                        gettext(b"E392: No such syntax cluster: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        arg,
                    );
                } else {
                    syn_list_cluster(id_2 - SYNID_CLUSTER);
                }
            } else {
                let mut id_3: ::core::ffi::c_int =
                    syn_name2id_len(arg, arg_end.offset_from(arg) as size_t);
                if id_3 == 0 as ::core::ffi::c_int {
                    semsg(
                        gettext(&raw const e_nogroup as *const ::core::ffi::c_char),
                        arg,
                    );
                } else {
                    syn_list_one(id_3, syncing != 0, true_0 != 0);
                }
            }
            arg = skipwhite(arg_end);
        }
    }
    (*eap).nextcmd = check_nextcmd(arg);
}
unsafe extern "C" fn syn_lines_msg() {
    if (*(*curwin.get()).w_s).b_syn_sync_maxlines > 0 as linenr_T
        || (*(*curwin.get()).w_s).b_syn_sync_minlines > 0 as linenr_T
    {
        msg_puts(b"; \0".as_ptr() as *const ::core::ffi::c_char);
        if (*(*curwin.get()).w_s).b_syn_sync_minlines == MAXLNUM as ::core::ffi::c_int as linenr_T {
            msg_puts(gettext(
                b"from the first line\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else {
            if (*(*curwin.get()).w_s).b_syn_sync_minlines > 0 as linenr_T {
                msg_puts(gettext(b"minimal \0".as_ptr() as *const ::core::ffi::c_char));
                msg_outnum((*(*curwin.get()).w_s).b_syn_sync_minlines as ::core::ffi::c_int);
                if (*(*curwin.get()).w_s).b_syn_sync_maxlines != 0 {
                    msg_puts(b", \0".as_ptr() as *const ::core::ffi::c_char);
                }
            }
            if (*(*curwin.get()).w_s).b_syn_sync_maxlines > 0 as linenr_T {
                msg_puts(gettext(b"maximal \0".as_ptr() as *const ::core::ffi::c_char));
                msg_outnum((*(*curwin.get()).w_s).b_syn_sync_maxlines as ::core::ffi::c_int);
            }
            msg_puts(gettext(
                b" lines before top line\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
    }
}
unsafe extern "C" fn syn_match_msg() {
    if (*(*curwin.get()).w_s).b_syn_sync_linebreaks > 0 as linenr_T {
        msg_puts(gettext(b"; match \0".as_ptr() as *const ::core::ffi::c_char));
        msg_outnum((*(*curwin.get()).w_s).b_syn_sync_linebreaks as ::core::ffi::c_int);
        msg_puts(gettext(
            b" line breaks\0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
}
static last_matchgroup: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
unsafe extern "C" fn syn_list_one(id: ::core::ffi::c_int, syncing: bool, link_only: bool) {
    let mut did_header: bool = false_0 != 0;
    let hl_id: ::core::ffi::c_int = HLF_D;
    if !syncing {
        did_header = syn_list_keywords(
            id,
            &raw mut (*(*curwin.get()).w_s).b_keywtab,
            false_0 != 0,
            hl_id,
        );
        did_header = syn_list_keywords(
            id,
            &raw mut (*(*curwin.get()).w_s).b_keywtab_ic,
            did_header,
            hl_id,
        );
    }
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while idx < (*(*curwin.get()).w_s).b_syn_patterns.ga_len && !got_int.get() {
        let spp: *const synpat_T =
            ((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
        if !((*spp).sp_syn.id as ::core::ffi::c_int != id
            || (*spp).sp_syncing as ::core::ffi::c_int != syncing as ::core::ffi::c_int)
        {
            syn_list_header(did_header, 0 as ::core::ffi::c_int, id, true_0 != 0);
            did_header = true_0 != 0;
            last_matchgroup.set(0 as ::core::ffi::c_int);
            if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_MATCH {
                put_pattern(
                    b"match\0".as_ptr() as *const ::core::ffi::c_char,
                    ' ' as ::core::ffi::c_int,
                    spp,
                    hl_id,
                );
                msg_putchar(' ' as ::core::ffi::c_int);
            } else if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_START {
                while (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                    .offset(idx as isize))
                .sp_type as ::core::ffi::c_int
                    == SPTYPE_START
                {
                    let c2rust_fresh8 = idx;
                    idx = idx + 1;
                    put_pattern(
                        b"start\0".as_ptr() as *const ::core::ffi::c_char,
                        '=' as ::core::ffi::c_int,
                        ((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(c2rust_fresh8 as isize),
                        hl_id,
                    );
                }
                if (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                    .offset(idx as isize))
                .sp_type as ::core::ffi::c_int
                    == SPTYPE_SKIP
                {
                    let c2rust_fresh9 = idx;
                    idx = idx + 1;
                    put_pattern(
                        b"skip\0".as_ptr() as *const ::core::ffi::c_char,
                        '=' as ::core::ffi::c_int,
                        ((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(c2rust_fresh9 as isize),
                        hl_id,
                    );
                }
                while idx < (*(*curwin.get()).w_s).b_syn_patterns.ga_len
                    && (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                        .offset(idx as isize))
                    .sp_type as ::core::ffi::c_int
                        == SPTYPE_END
                {
                    let c2rust_fresh10 = idx;
                    idx = idx + 1;
                    put_pattern(
                        b"end\0".as_ptr() as *const ::core::ffi::c_char,
                        '=' as ::core::ffi::c_int,
                        ((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(c2rust_fresh10 as isize),
                        hl_id,
                    );
                }
                idx -= 1;
                msg_putchar(' ' as ::core::ffi::c_int);
            }
            syn_list_flags(
                namelist1.ptr() as *mut keyvalue_T,
                ::core::mem::size_of::<[keyvalue_T; 10]>()
                    .wrapping_div(::core::mem::size_of::<keyvalue_T>())
                    .wrapping_div(
                        (::core::mem::size_of::<[keyvalue_T; 10]>()
                            .wrapping_rem(::core::mem::size_of::<keyvalue_T>())
                            == 0) as ::core::ffi::c_int as size_t,
                    ),
                (*spp).sp_flags,
                hl_id,
            );
            if !(*spp).sp_cont_list.is_null() {
                put_id_list(
                    b"contains\0".as_ptr() as *const ::core::ffi::c_char,
                    (*spp).sp_cont_list,
                    hl_id,
                );
            }
            if !(*spp).sp_syn.cont_in_list.is_null() {
                put_id_list(
                    b"containedin\0".as_ptr() as *const ::core::ffi::c_char,
                    (*spp).sp_syn.cont_in_list,
                    hl_id,
                );
            }
            if !(*spp).sp_next_list.is_null() {
                put_id_list(
                    b"nextgroup\0".as_ptr() as *const ::core::ffi::c_char,
                    (*spp).sp_next_list,
                    hl_id,
                );
                syn_list_flags(
                    namelist2.ptr() as *mut keyvalue_T,
                    ::core::mem::size_of::<[keyvalue_T; 3]>()
                        .wrapping_div(::core::mem::size_of::<keyvalue_T>())
                        .wrapping_div(
                            (::core::mem::size_of::<[keyvalue_T; 3]>()
                                .wrapping_rem(::core::mem::size_of::<keyvalue_T>())
                                == 0) as ::core::ffi::c_int as size_t,
                        ),
                    (*spp).sp_flags,
                    hl_id,
                );
            }
            if (*spp).sp_flags & (HL_SYNC_HERE | HL_SYNC_THERE) != 0 {
                if (*spp).sp_flags & HL_SYNC_HERE != 0 {
                    msg_puts_hl(
                        b"grouphere\0".as_ptr() as *const ::core::ffi::c_char,
                        hl_id,
                        false_0 != 0,
                    );
                } else {
                    msg_puts_hl(
                        b"groupthere\0".as_ptr() as *const ::core::ffi::c_char,
                        hl_id,
                        false_0 != 0,
                    );
                }
                msg_putchar(' ' as ::core::ffi::c_int);
                if (*spp).sp_sync_idx >= 0 as ::core::ffi::c_int {
                    msg_outtrans(
                        highlight_group_name(
                            (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset((*spp).sp_sync_idx as isize))
                            .sp_syn
                            .id as ::core::ffi::c_int
                                - 1 as ::core::ffi::c_int,
                        ),
                        0 as ::core::ffi::c_int,
                        false_0 != 0,
                    );
                } else {
                    msg_puts(b"NONE\0".as_ptr() as *const ::core::ffi::c_char);
                }
                msg_putchar(' ' as ::core::ffi::c_int);
            }
        }
        idx += 1;
    }
    if highlight_link_id(id - 1 as ::core::ffi::c_int) != 0
        && (did_header as ::core::ffi::c_int != 0 || link_only as ::core::ffi::c_int != 0)
        && !got_int.get()
    {
        syn_list_header(did_header, 0 as ::core::ffi::c_int, id, true_0 != 0);
        msg_puts_hl(
            b"links to\0".as_ptr() as *const ::core::ffi::c_char,
            hl_id,
            false_0 != 0,
        );
        msg_putchar(' ' as ::core::ffi::c_int);
        msg_outtrans(
            highlight_group_name(
                highlight_link_id(id - 1 as ::core::ffi::c_int) - 1 as ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
            false_0 != 0,
        );
    }
}
unsafe extern "C" fn syn_list_flags(
    mut nlist: *mut keyvalue_T,
    mut nr_entries: size_t,
    mut flags: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
) {
    let mut i: size_t = 0 as size_t;
    while i < nr_entries {
        if flags & (*nlist.offset(i as isize)).key != 0 {
            msg_puts_hl((*nlist.offset(i as isize)).value, hl_id, false_0 != 0);
            msg_putchar(' ' as ::core::ffi::c_int);
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn syn_list_cluster(mut id: ::core::ffi::c_int) {
    let mut endcol: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
    msg_putchar('\n' as ::core::ffi::c_int);
    msg_outtrans(
        (*((*(*curwin.get()).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
            .offset(id as isize))
        .scl_name,
        0 as ::core::ffi::c_int,
        false_0 != 0,
    );
    if msg_col.get() >= endcol {
        endcol = msg_col.get() + 1 as ::core::ffi::c_int;
    }
    if Columns.get() <= endcol {
        endcol = Columns.get() - 1 as ::core::ffi::c_int;
    }
    msg_advance(endcol);
    if !(*((*(*curwin.get()).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T).offset(id as isize))
        .scl_list
        .is_null()
    {
        put_id_list(
            b"cluster\0".as_ptr() as *const ::core::ffi::c_char,
            (*((*(*curwin.get()).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                .offset(id as isize))
            .scl_list,
            HLF_D,
        );
    } else {
        msg_puts_hl(
            b"cluster\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_D,
            false_0 != 0,
        );
        msg_puts(b"=NONE\0".as_ptr() as *const ::core::ffi::c_char);
    };
}
unsafe extern "C" fn put_id_list(
    name: *const ::core::ffi::c_char,
    list: *const int16_t,
    hl_id: ::core::ffi::c_int,
) {
    msg_puts_hl(name, hl_id, false_0 != 0);
    msg_putchar('=' as ::core::ffi::c_int);
    let mut p: *const int16_t = list;
    while *p != 0 {
        if *p as ::core::ffi::c_int >= MAX_HL_ID as ::core::ffi::c_int
            && (*p as ::core::ffi::c_int) < SYNID_TOP
        {
            if *p.offset(1 as ::core::ffi::c_int as isize) != 0 {
                msg_puts(b"ALLBUT\0".as_ptr() as *const ::core::ffi::c_char);
            } else {
                msg_puts(b"ALL\0".as_ptr() as *const ::core::ffi::c_char);
            }
        } else if *p as ::core::ffi::c_int >= SYNID_TOP
            && (*p as ::core::ffi::c_int) < SYNID_CONTAINED
        {
            msg_puts(b"TOP\0".as_ptr() as *const ::core::ffi::c_char);
        } else if *p as ::core::ffi::c_int >= SYNID_CONTAINED
            && (*p as ::core::ffi::c_int) < SYNID_CLUSTER
        {
            msg_puts(b"CONTAINED\0".as_ptr() as *const ::core::ffi::c_char);
        } else if *p as ::core::ffi::c_int >= SYNID_CLUSTER {
            let mut scl_id: ::core::ffi::c_int = *p as ::core::ffi::c_int - SYNID_CLUSTER;
            msg_putchar('@' as ::core::ffi::c_int);
            msg_outtrans(
                (*((*(*curwin.get()).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                    .offset(scl_id as isize))
                .scl_name,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        } else {
            msg_outtrans(
                highlight_group_name(*p as ::core::ffi::c_int - 1 as ::core::ffi::c_int),
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        }
        if *p.offset(1 as ::core::ffi::c_int as isize) != 0 {
            msg_putchar(',' as ::core::ffi::c_int);
        }
        p = p.offset(1);
    }
    msg_putchar(' ' as ::core::ffi::c_int);
}
unsafe extern "C" fn put_pattern(
    s: *const ::core::ffi::c_char,
    c: ::core::ffi::c_int,
    spp: *const synpat_T,
    hl_id: ::core::ffi::c_int,
) {
    static sepchars: GlobalCell<*const ::core::ffi::c_char> =
        GlobalCell::new(b"/+=-#@\"|'^&\0".as_ptr() as *const ::core::ffi::c_char);
    let mut i: ::core::ffi::c_int = 0;
    if last_matchgroup.get() != (*spp).sp_syn_match_id as ::core::ffi::c_int {
        last_matchgroup.set((*spp).sp_syn_match_id as ::core::ffi::c_int);
        msg_puts_hl(
            b"matchgroup\0".as_ptr() as *const ::core::ffi::c_char,
            hl_id,
            false_0 != 0,
        );
        msg_putchar('=' as ::core::ffi::c_int);
        if last_matchgroup.get() == 0 as ::core::ffi::c_int {
            msg_outtrans(
                b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        } else {
            msg_outtrans(
                highlight_group_name(last_matchgroup.get() - 1 as ::core::ffi::c_int),
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        }
        msg_putchar(' ' as ::core::ffi::c_int);
    }
    msg_puts_hl(s, hl_id, false_0 != 0);
    msg_putchar(c);
    i = 0 as ::core::ffi::c_int;
    while !vim_strchr(
        (*spp).sp_pattern,
        *(*sepchars.ptr()).offset(i as isize) as uint8_t as ::core::ffi::c_int,
    )
    .is_null()
    {
        i += 1;
        if *(*sepchars.ptr()).offset(i as isize) as ::core::ffi::c_int != NUL {
            continue;
        }
        i = 0 as ::core::ffi::c_int;
        break;
    }
    msg_putchar(*(*sepchars.ptr()).offset(i as isize) as ::core::ffi::c_int);
    msg_outtrans((*spp).sp_pattern, 0 as ::core::ffi::c_int, false_0 != 0);
    msg_putchar(*(*sepchars.ptr()).offset(i as isize) as ::core::ffi::c_int);
    let mut first: bool = true_0 != 0;
    i = 0 as ::core::ffi::c_int;
    while i < SPO_COUNT {
        let mask: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << i;
        if (*spp).sp_off_flags as ::core::ffi::c_int & mask + (mask << SPO_COUNT) != 0 {
            if !first {
                msg_putchar(',' as ::core::ffi::c_int);
            }
            msg_puts((*spo_name_tab.ptr())[i as usize] as *const ::core::ffi::c_char);
            let n: ::core::ffi::c_int = (*spp).sp_offsets[i as usize];
            if i != SPO_LC_OFF {
                if (*spp).sp_off_flags as ::core::ffi::c_int & mask != 0 {
                    msg_putchar('s' as ::core::ffi::c_int);
                } else {
                    msg_putchar('e' as ::core::ffi::c_int);
                }
                if n > 0 as ::core::ffi::c_int {
                    msg_putchar('+' as ::core::ffi::c_int);
                }
            }
            if n != 0 || i == SPO_LC_OFF {
                msg_outnum(n);
            }
            first = false_0 != 0;
        }
        i += 1;
    }
    msg_putchar(' ' as ::core::ffi::c_int);
}
unsafe extern "C" fn syn_list_keywords(
    id: ::core::ffi::c_int,
    ht: *const hashtab_T,
    mut did_header: bool,
    hl_id: ::core::ffi::c_int,
) -> bool {
    let mut prev_contained: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut prev_next_list: *const int16_t = ::core::ptr::null::<int16_t>();
    let mut prev_cont_in_list: *const int16_t = ::core::ptr::null::<int16_t>();
    let mut prev_skipnl: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut prev_skipwhite: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut prev_skipempty: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut todo: size_t = (*ht).ht_used;
    let mut hi: *const hashitem_T = (*ht).ht_array;
    while todo > 0 as size_t && !got_int.get() {
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            todo = todo.wrapping_sub(1);
            let mut kp: *mut keyentry_T = (*hi).hi_key.offset(
                -((&raw mut (*dumkey.ptr()).keyword as *mut ::core::ffi::c_char)
                    .offset_from(dumkey.ptr() as *mut ::core::ffi::c_char)
                    as isize),
            ) as *mut keyentry_T;
            while !kp.is_null() && !got_int.get() {
                if (*kp).k_syn.id as ::core::ffi::c_int == id {
                    let mut outlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut force_newline: bool = false_0 != 0;
                    if prev_contained != (*kp).flags & HL_CONTAINED
                        || prev_skipnl != (*kp).flags & HL_SKIPNL
                        || prev_skipwhite != (*kp).flags & HL_SKIPWHITE
                        || prev_skipempty != (*kp).flags & HL_SKIPEMPTY
                        || prev_cont_in_list != (*kp).k_syn.cont_in_list as *const int16_t
                        || prev_next_list != (*kp).next_list as *const int16_t
                    {
                        force_newline = true_0 != 0;
                    } else {
                        outlen = strlen(&raw mut (*kp).keyword as *mut ::core::ffi::c_char)
                            as ::core::ffi::c_int;
                    }
                    if syn_list_header(did_header, outlen, id, force_newline) {
                        prev_contained = 0 as ::core::ffi::c_int;
                        prev_next_list = ::core::ptr::null::<int16_t>();
                        prev_cont_in_list = ::core::ptr::null::<int16_t>();
                        prev_skipnl = 0 as ::core::ffi::c_int;
                        prev_skipwhite = 0 as ::core::ffi::c_int;
                        prev_skipempty = 0 as ::core::ffi::c_int;
                    }
                    did_header = true_0 != 0;
                    if prev_contained != (*kp).flags & HL_CONTAINED {
                        msg_puts_hl(
                            b"contained\0".as_ptr() as *const ::core::ffi::c_char,
                            hl_id,
                            false_0 != 0,
                        );
                        msg_putchar(' ' as ::core::ffi::c_int);
                        prev_contained = (*kp).flags & HL_CONTAINED;
                    }
                    if (*kp).k_syn.cont_in_list != prev_cont_in_list as *mut int16_t {
                        put_id_list(
                            b"containedin\0".as_ptr() as *const ::core::ffi::c_char,
                            (*kp).k_syn.cont_in_list,
                            hl_id,
                        );
                        msg_putchar(' ' as ::core::ffi::c_int);
                        prev_cont_in_list = (*kp).k_syn.cont_in_list;
                    }
                    if (*kp).next_list != prev_next_list as *mut int16_t {
                        put_id_list(
                            b"nextgroup\0".as_ptr() as *const ::core::ffi::c_char,
                            (*kp).next_list,
                            hl_id,
                        );
                        msg_putchar(' ' as ::core::ffi::c_int);
                        prev_next_list = (*kp).next_list;
                        if (*kp).flags & HL_SKIPNL != 0 {
                            msg_puts_hl(
                                b"skipnl\0".as_ptr() as *const ::core::ffi::c_char,
                                hl_id,
                                false_0 != 0,
                            );
                            msg_putchar(' ' as ::core::ffi::c_int);
                            prev_skipnl = (*kp).flags & HL_SKIPNL;
                        }
                        if (*kp).flags & HL_SKIPWHITE != 0 {
                            msg_puts_hl(
                                b"skipwhite\0".as_ptr() as *const ::core::ffi::c_char,
                                hl_id,
                                false_0 != 0,
                            );
                            msg_putchar(' ' as ::core::ffi::c_int);
                            prev_skipwhite = (*kp).flags & HL_SKIPWHITE;
                        }
                        if (*kp).flags & HL_SKIPEMPTY != 0 {
                            msg_puts_hl(
                                b"skipempty\0".as_ptr() as *const ::core::ffi::c_char,
                                hl_id,
                                false_0 != 0,
                            );
                            msg_putchar(' ' as ::core::ffi::c_int);
                            prev_skipempty = (*kp).flags & HL_SKIPEMPTY;
                        }
                    }
                    msg_outtrans(
                        &raw mut (*kp).keyword as *mut ::core::ffi::c_char,
                        0 as ::core::ffi::c_int,
                        false_0 != 0,
                    );
                }
                kp = (*kp).ke_next;
            }
        }
        hi = hi.offset(1);
    }
    return did_header;
}
unsafe extern "C" fn syn_clear_keyword(mut id: ::core::ffi::c_int, mut ht: *mut hashtab_T) {
    hash_lock(ht);
    let mut todo: ::core::ffi::c_int = (*ht).ht_used as ::core::ffi::c_int;
    let mut hi: *mut hashitem_T = (*ht).ht_array;
    while todo > 0 as ::core::ffi::c_int {
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            todo -= 1;
            let mut kp_prev: *mut keyentry_T = ::core::ptr::null_mut::<keyentry_T>();
            let mut kp: *mut keyentry_T = (*hi).hi_key.offset(
                -((&raw mut (*dumkey.ptr()).keyword as *mut ::core::ffi::c_char)
                    .offset_from(dumkey.ptr() as *mut ::core::ffi::c_char)
                    as isize),
            ) as *mut keyentry_T;
            while !kp.is_null() {
                if (*kp).k_syn.id as ::core::ffi::c_int == id {
                    let mut kp_next: *mut keyentry_T = (*kp).ke_next;
                    if kp_prev.is_null() {
                        if kp_next.is_null() {
                            hash_remove(ht, hi);
                        } else {
                            (*hi).hi_key = &raw mut (*kp_next).keyword as *mut ::core::ffi::c_char;
                        }
                    } else {
                        (*kp_prev).ke_next = kp_next;
                    }
                    xfree((*kp).next_list as *mut ::core::ffi::c_void);
                    xfree((*kp).k_syn.cont_in_list as *mut ::core::ffi::c_void);
                    xfree(kp as *mut ::core::ffi::c_void);
                    kp = kp_next;
                } else {
                    kp_prev = kp;
                    kp = (*kp).ke_next;
                }
            }
        }
        hi = hi.offset(1);
    }
    hash_unlock(ht);
}
unsafe extern "C" fn clear_keywtab(mut ht: *mut hashtab_T) {
    let mut kp_next: *mut keyentry_T = ::core::ptr::null_mut::<keyentry_T>();
    let mut todo: ::core::ffi::c_int = (*ht).ht_used as ::core::ffi::c_int;
    let mut hi: *mut hashitem_T = (*ht).ht_array;
    while todo > 0 as ::core::ffi::c_int {
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            todo -= 1;
            let mut kp: *mut keyentry_T = (*hi).hi_key.offset(
                -((&raw mut (*dumkey.ptr()).keyword as *mut ::core::ffi::c_char)
                    .offset_from(dumkey.ptr() as *mut ::core::ffi::c_char)
                    as isize),
            ) as *mut keyentry_T;
            while !kp.is_null() {
                kp_next = (*kp).ke_next;
                xfree((*kp).next_list as *mut ::core::ffi::c_void);
                xfree((*kp).k_syn.cont_in_list as *mut ::core::ffi::c_void);
                xfree(kp as *mut ::core::ffi::c_void);
                kp = kp_next;
            }
        }
        hi = hi.offset(1);
    }
    hash_clear(ht);
    hash_init(ht);
}
unsafe extern "C" fn add_keyword(
    name: *mut ::core::ffi::c_char,
    mut namelen: size_t,
    id: ::core::ffi::c_int,
    flags: ::core::ffi::c_int,
    cont_in_list: *mut int16_t,
    next_list: *mut int16_t,
    conceal_char: ::core::ffi::c_int,
) {
    let mut name_folded: [::core::ffi::c_char; 81] = [0; 81];
    let mut name_ic: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut name_iclen: size_t = 0;
    if (*(*curwin.get()).w_s).b_syn_ic != 0 {
        name_ic = str_foldcase(
            name,
            namelen as ::core::ffi::c_int,
            &raw mut name_folded as *mut ::core::ffi::c_char,
            MAXKEYWLEN + 1 as ::core::ffi::c_int,
        );
        name_iclen = strlen(name_ic);
    } else {
        name_ic = name;
        name_iclen = namelen;
    }
    let kp: *mut keyentry_T = xmalloc(
        (40 as size_t)
            .wrapping_add(name_iclen)
            .wrapping_add(1 as size_t),
    ) as *mut keyentry_T;
    strcpy(
        &raw mut (*kp).keyword as *mut ::core::ffi::c_char,
        name_ic as *mut ::core::ffi::c_char,
    );
    (*kp).k_syn.id = id as int16_t;
    (*kp).k_syn.inc_tag = current_syn_inc_tag.get();
    (*kp).flags = flags;
    (*kp).k_char = conceal_char;
    (*kp).k_syn.cont_in_list = copy_id_list(cont_in_list);
    if !cont_in_list.is_null() {
        (*(*curwin.get()).w_s).b_syn_containedin = true_0;
    }
    (*kp).next_list = copy_id_list(next_list);
    let hash: hash_T = hash_hash(&raw mut (*kp).keyword as *mut ::core::ffi::c_char);
    let ht: *mut hashtab_T = if (*(*curwin.get()).w_s).b_syn_ic != 0 {
        &raw mut (*(*curwin.get()).w_s).b_keywtab_ic
    } else {
        &raw mut (*(*curwin.get()).w_s).b_keywtab
    };
    let hi: *mut hashitem_T = hash_lookup(
        ht,
        &raw mut (*kp).keyword as *mut ::core::ffi::c_char,
        strlen(&raw mut (*kp).keyword as *mut ::core::ffi::c_char),
        hash,
    );
    if (*hi).hi_key.is_null() || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
    {
        (*kp).ke_next = ::core::ptr::null_mut::<keyentry_T>();
        hash_add_item(
            ht,
            hi,
            &raw mut (*kp).keyword as *mut ::core::ffi::c_char,
            hash,
        );
    } else {
        (*kp).ke_next = (*hi).hi_key.offset(
            -((&raw mut (*dumkey.ptr()).keyword as *mut ::core::ffi::c_char)
                .offset_from(dumkey.ptr() as *mut ::core::ffi::c_char) as isize),
        ) as *mut keyentry_T;
        (*hi).hi_key = &raw mut (*kp).keyword as *mut ::core::ffi::c_char;
    };
}
unsafe extern "C" fn get_group_name(
    mut arg: *mut ::core::ffi::c_char,
    mut name_end: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    *name_end = skiptowhite(arg);
    let mut rest: *mut ::core::ffi::c_char = skipwhite(*name_end);
    if ends_excmd(*arg as ::core::ffi::c_int) != 0 || *rest as ::core::ffi::c_int == NUL {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return rest;
}
unsafe extern "C" fn get_syn_options(
    mut arg: *mut ::core::ffi::c_char,
    mut opt: *mut syn_opt_arg_T,
    mut conceal_char: *mut ::core::ffi::c_int,
    mut skip: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut fidx: ::core::ffi::c_int = 0;
    static flagtab: GlobalCell<[flag; 19]> = GlobalCell::new([
        flag {
            name: b"cCoOnNtTaAiInNeEdD\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_CONTAINED,
        },
        flag {
            name: b"oOnNeElLiInNeE\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_ONELINE,
        },
        flag {
            name: b"kKeEeEpPeEnNdD\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_KEEPEND,
        },
        flag {
            name: b"eExXtTeEnNdD\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_EXTEND,
        },
        flag {
            name: b"eExXcClLuUdDeEnNlL\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_EXCLUDENL,
        },
        flag {
            name: b"tTrRaAnNsSpPaArReEnNtT\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_TRANSP,
        },
        flag {
            name: b"sSkKiIpPnNlL\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_SKIPNL,
        },
        flag {
            name: b"sSkKiIpPwWhHiItTeE\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_SKIPWHITE,
        },
        flag {
            name: b"sSkKiIpPeEmMpPtTyY\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_SKIPEMPTY,
        },
        flag {
            name: b"gGrRoOuUpPhHeErReE\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_SYNC_HERE,
        },
        flag {
            name: b"gGrRoOuUpPtThHeErReE\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_SYNC_THERE,
        },
        flag {
            name: b"dDiIsSpPlLaAyY\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_DISPLAY,
        },
        flag {
            name: b"fFoOlLdD\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_FOLD,
        },
        flag {
            name: b"cCoOnNcCeEaAlL\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_CONCEAL,
        },
        flag {
            name: b"cCoOnNcCeEaAlLeEnNdDsS\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 0 as ::core::ffi::c_int,
            flags: HL_CONCEALENDS,
        },
        flag {
            name: b"cCcChHaArR\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 11 as ::core::ffi::c_int,
            flags: 0 as ::core::ffi::c_int,
        },
        flag {
            name: b"cCoOnNtTaAiInNsS\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 1 as ::core::ffi::c_int,
            flags: 0 as ::core::ffi::c_int,
        },
        flag {
            name: b"cCoOnNtTaAiInNeEdDiInN\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 2 as ::core::ffi::c_int,
            flags: 0 as ::core::ffi::c_int,
        },
        flag {
            name: b"nNeExXtTgGrRoOuUpP\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            argtype: 3 as ::core::ffi::c_int,
            flags: 0 as ::core::ffi::c_int,
        },
    ]);
    static first_letters: GlobalCell<*const ::core::ffi::c_char> =
        GlobalCell::new(b"cCoOkKeEtTsSgGdDfFnN\0".as_ptr() as *const ::core::ffi::c_char);
    if arg.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if (*(*curwin.get()).w_s).b_syn_conceal != 0 {
        (*opt).flags |= HL_CONCEAL;
    }
    while !strchr(first_letters.get(), *arg as ::core::ffi::c_int).is_null() {
        fidx = ::core::mem::size_of::<[flag; 19]>()
            .wrapping_div(::core::mem::size_of::<flag>())
            .wrapping_div(
                (::core::mem::size_of::<[flag; 19]>().wrapping_rem(::core::mem::size_of::<flag>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int;
        loop {
            fidx -= 1;
            if fidx < 0 as ::core::ffi::c_int {
                break;
            }
            let mut p: *mut ::core::ffi::c_char = (*flagtab.ptr())[fidx as usize].name;
            let mut i: ::core::ffi::c_int = 0;
            i = 0 as ::core::ffi::c_int;
            len = 0 as ::core::ffi::c_int;
            while *p.offset(i as isize) as ::core::ffi::c_int != NUL {
                if *arg.offset(len as isize) as ::core::ffi::c_int
                    != *p.offset(i as isize) as ::core::ffi::c_int
                    && *arg.offset(len as isize) as ::core::ffi::c_int
                        != *p.offset((i + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                {
                    break;
                }
                i += 2 as ::core::ffi::c_int;
                len += 1;
            }
            if !(*p.offset(i as isize) as ::core::ffi::c_int == NUL
                && (ascii_iswhite(*arg.offset(len as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
                    || (if (*flagtab.ptr())[fidx as usize].argtype > 0 as ::core::ffi::c_int {
                        (*arg.offset(len as isize) as ::core::ffi::c_int
                            == '=' as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                    } else {
                        ends_excmd(*arg.offset(len as isize) as ::core::ffi::c_int)
                    }) != 0))
            {
                continue;
            }
            if (*opt).keyword as ::core::ffi::c_int != 0
                && ((*flagtab.ptr())[fidx as usize].flags == HL_DISPLAY
                    || (*flagtab.ptr())[fidx as usize].flags == HL_FOLD
                    || (*flagtab.ptr())[fidx as usize].flags == HL_EXTEND)
            {
                fidx = -1 as ::core::ffi::c_int;
            }
            break;
        }
        if fidx < 0 as ::core::ffi::c_int {
            break;
        }
        if (*flagtab.ptr())[fidx as usize].argtype == 1 as ::core::ffi::c_int {
            if !(*opt).has_cont_list {
                emsg(gettext(
                    (e_contains_argument_not_accepted_here.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ));
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            if get_id_list(
                &raw mut arg,
                8 as ::core::ffi::c_int,
                &raw mut (*opt).cont_list,
                skip != 0,
            ) == FAIL
            {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        } else if (*flagtab.ptr())[fidx as usize].argtype == 2 as ::core::ffi::c_int {
            if get_id_list(
                &raw mut arg,
                11 as ::core::ffi::c_int,
                &raw mut (*opt).cont_in_list,
                skip != 0,
            ) == FAIL
            {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        } else if (*flagtab.ptr())[fidx as usize].argtype == 3 as ::core::ffi::c_int {
            if get_id_list(
                &raw mut arg,
                9 as ::core::ffi::c_int,
                &raw mut (*opt).next_list,
                skip != 0,
            ) == FAIL
            {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        } else if (*flagtab.ptr())[fidx as usize].argtype == 11 as ::core::ffi::c_int
            && *arg.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '=' as ::core::ffi::c_int
        {
            *conceal_char = utf_ptr2char(arg.offset(6 as ::core::ffi::c_int as isize));
            arg = arg.offset(
                (utfc_ptr2len(arg.offset(6 as ::core::ffi::c_int as isize))
                    - 1 as ::core::ffi::c_int) as isize,
            );
            if !vim_isprintc(*conceal_char) {
                emsg(gettext(
                    (e_invalid_cchar_value.ptr() as *const _) as *const ::core::ffi::c_char,
                ));
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            arg = skipwhite(arg.offset(7 as ::core::ffi::c_int as isize));
        } else {
            (*opt).flags |= (*flagtab.ptr())[fidx as usize].flags;
            arg = skipwhite(arg.offset(len as isize));
            if (*flagtab.ptr())[fidx as usize].flags == HL_SYNC_HERE
                || (*flagtab.ptr())[fidx as usize].flags == HL_SYNC_THERE
            {
                if (*opt).sync_idx.is_null() {
                    emsg(gettext(b"E393: group[t]here not accepted here\0".as_ptr()
                        as *const ::core::ffi::c_char));
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                let mut gname_start: *mut ::core::ffi::c_char = arg;
                arg = skiptowhite(arg);
                if gname_start == arg {
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                let mut gname: *mut ::core::ffi::c_char =
                    xstrnsave(gname_start, arg.offset_from(gname_start) as size_t);
                if strcmp(gname, b"NONE\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    *(*opt).sync_idx = NONE_IDX;
                } else {
                    let mut syn_id: ::core::ffi::c_int = syn_name2id(gname);
                    let mut i_0: ::core::ffi::c_int = 0;
                    i_0 = (*(*curwin.get()).w_s).b_syn_patterns.ga_len;
                    loop {
                        i_0 -= 1;
                        if i_0 < 0 as ::core::ffi::c_int {
                            break;
                        }
                        if !((*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(i_0 as isize))
                        .sp_syn
                        .id as ::core::ffi::c_int
                            == syn_id
                            && (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(i_0 as isize))
                            .sp_type as ::core::ffi::c_int
                                == SPTYPE_START)
                        {
                            continue;
                        }
                        *(*opt).sync_idx = i_0;
                        break;
                    }
                    if i_0 < 0 as ::core::ffi::c_int {
                        semsg(
                            gettext(b"E394: Didn't find region item for %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            gname,
                        );
                        xfree(gname as *mut ::core::ffi::c_void);
                        return ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                }
                xfree(gname as *mut ::core::ffi::c_void);
                arg = skipwhite(arg);
            } else if (*flagtab.ptr())[fidx as usize].flags == HL_FOLD
                && foldmethodIsSyntax(curwin.get()) as ::core::ffi::c_int != 0
            {
                foldUpdateAll(curwin.get());
            }
        }
    }
    return arg;
}
unsafe extern "C" fn syn_incl_toplevel(
    mut id: ::core::ffi::c_int,
    mut flagsp: *mut ::core::ffi::c_int,
) {
    if *flagsp & HL_CONTAINED != 0 || (*(*curwin.get()).w_s).b_syn_topgrp == 0 as ::core::ffi::c_int
    {
        return;
    }
    *flagsp |= HL_CONTAINED | HL_INCLUDED_TOPLEVEL;
    if (*(*curwin.get()).w_s).b_syn_topgrp >= SYNID_CLUSTER {
        let mut grp_list: *mut int16_t =
            xmalloc((2 as size_t).wrapping_mul(::core::mem::size_of::<int16_t>())) as *mut int16_t;
        let mut tlg_id: ::core::ffi::c_int = (*(*curwin.get()).w_s).b_syn_topgrp - SYNID_CLUSTER;
        *grp_list.offset(0 as ::core::ffi::c_int as isize) = id as int16_t;
        *grp_list.offset(1 as ::core::ffi::c_int as isize) = 0 as int16_t;
        syn_combine_list(
            &raw mut (*((*(*curwin.get()).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                .offset(tlg_id as isize))
            .scl_list,
            &raw mut grp_list,
            CLUSTER_ADD,
        );
    }
}
unsafe extern "C" fn syn_cmd_include(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut sgl_id: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut group_name_end: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut source: bool = false_0 != 0;
    (*eap).nextcmd = find_nextcmd(arg);
    if (*eap).skip != 0 {
        return;
    }
    if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '@' as ::core::ffi::c_int
    {
        arg = arg.offset(1);
        let mut rest: *mut ::core::ffi::c_char = get_group_name(arg, &raw mut group_name_end);
        if rest.is_null() {
            emsg(gettext(
                b"E397: Filename required\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return;
        }
        sgl_id = syn_check_cluster(arg, group_name_end.offset_from(arg) as ::core::ffi::c_int);
        if sgl_id == 0 as ::core::ffi::c_int {
            return;
        }
        (*eap).arg = rest;
    }
    (*eap).argt = ((*eap).argt as ::core::ffi::c_uint | (EX_XFILE | EX_NOSPC)) as uint32_t;
    separate_nextcmd(eap);
    if *(*eap).arg as ::core::ffi::c_int == '<' as ::core::ffi::c_int
        || *(*eap).arg as ::core::ffi::c_int == '$' as ::core::ffi::c_int
        || path_is_absolute((*eap).arg) as ::core::ffi::c_int != 0
    {
        source = true_0 != 0;
        if expand_filename(eap, syn_cmdlinep.get(), &raw mut errormsg) == FAIL {
            if !errormsg.is_null() {
                emsg(errormsg);
            }
            return;
        }
    }
    if running_syn_inc_tag.get() >= MAX_SYN_INC_TAG {
        emsg(gettext(
            b"E847: Too many syntax includes\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return;
    }
    let mut prev_syn_inc_tag: ::core::ffi::c_int = current_syn_inc_tag.get();
    (*running_syn_inc_tag.ptr()) += 1;
    current_syn_inc_tag.set(running_syn_inc_tag.get());
    let mut prev_toplvl_grp: ::core::ffi::c_int = (*(*curwin.get()).w_s).b_syn_topgrp;
    (*(*curwin.get()).w_s).b_syn_topgrp = sgl_id;
    if if source as ::core::ffi::c_int != 0 {
        (do_source(
            (*eap).arg,
            false_0 != 0,
            DOSO_NONE as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ) == FAIL) as ::core::ffi::c_int
    } else {
        (source_runtime((*eap).arg, DIP_ALL as ::core::ffi::c_int) == FAIL) as ::core::ffi::c_int
    } != 0
    {
        semsg(
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            (*eap).arg,
        );
    }
    (*(*curwin.get()).w_s).b_syn_topgrp = prev_toplvl_grp;
    current_syn_inc_tag.set(prev_syn_inc_tag);
}
unsafe extern "C" fn syn_cmd_keyword(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut group_name_end: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut syn_id: ::core::ffi::c_int = 0;
    let mut keyword_copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut syn_opt_arg: syn_opt_arg_T = syn_opt_arg_T {
        flags: 0,
        keyword: false,
        sync_idx: ::core::ptr::null_mut::<::core::ffi::c_int>(),
        has_cont_list: false,
        cont_list: ::core::ptr::null_mut::<int16_t>(),
        cont_in_list: ::core::ptr::null_mut::<int16_t>(),
        next_list: ::core::ptr::null_mut::<int16_t>(),
    };
    let mut conceal_char: ::core::ffi::c_int = NUL;
    let mut rest: *mut ::core::ffi::c_char = get_group_name(arg, &raw mut group_name_end);
    if !rest.is_null() {
        if (*eap).skip != 0 {
            syn_id = -1 as ::core::ffi::c_int;
        } else {
            syn_id = syn_check_group(arg, group_name_end.offset_from(arg) as size_t);
        }
        if syn_id != 0 as ::core::ffi::c_int {
            keyword_copy =
                xmalloc(strlen(rest).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        }
        if !keyword_copy.is_null() {
            syn_opt_arg.flags = 0 as ::core::ffi::c_int;
            syn_opt_arg.keyword = true_0 != 0;
            syn_opt_arg.sync_idx = ::core::ptr::null_mut::<::core::ffi::c_int>();
            syn_opt_arg.has_cont_list = false_0 != 0;
            syn_opt_arg.cont_in_list = ::core::ptr::null_mut::<int16_t>();
            syn_opt_arg.next_list = ::core::ptr::null_mut::<int16_t>();
            let mut cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut p: *mut ::core::ffi::c_char = keyword_copy;
            while !rest.is_null() && ends_excmd(*rest as ::core::ffi::c_int) == 0 {
                rest = get_syn_options(
                    rest,
                    &raw mut syn_opt_arg,
                    &raw mut conceal_char,
                    (*eap).skip,
                );
                if rest.is_null() || ends_excmd(*rest as ::core::ffi::c_int) != 0 {
                    break;
                }
                while *rest as ::core::ffi::c_int != NUL
                    && !ascii_iswhite(*rest as ::core::ffi::c_int)
                {
                    if *rest as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                        && *rest.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != NUL
                    {
                        rest = rest.offset(1);
                    }
                    let c2rust_fresh11 = rest;
                    rest = rest.offset(1);
                    let c2rust_fresh12 = p;
                    p = p.offset(1);
                    *c2rust_fresh12 = *c2rust_fresh11;
                }
                let c2rust_fresh13 = p;
                p = p.offset(1);
                *c2rust_fresh13 = NUL as ::core::ffi::c_char;
                cnt += 1;
                rest = skipwhite(rest);
            }
            '_error: {
                if (*eap).skip == 0 {
                    syn_incl_toplevel(syn_id, &raw mut syn_opt_arg.flags);
                    let mut kwlen: size_t = 0 as size_t;
                    let mut kw: *mut ::core::ffi::c_char = keyword_copy;
                    loop {
                        cnt -= 1;
                        if cnt < 0 as ::core::ffi::c_int {
                            break '_error;
                        }
                        p = vim_strchr(kw, '[' as ::core::ffi::c_int);
                        loop {
                            if p.is_null() {
                                kwlen = strlen(kw);
                            } else {
                                *p = NUL as ::core::ffi::c_char;
                                kwlen = p.offset_from(kw) as size_t;
                            }
                            add_keyword(
                                kw,
                                kwlen,
                                syn_id,
                                syn_opt_arg.flags,
                                syn_opt_arg.cont_in_list,
                                syn_opt_arg.next_list,
                                conceal_char,
                            );
                            if p.is_null() {
                                break;
                            }
                            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == NUL
                            {
                                semsg(
                                    gettext(b"E789: Missing ']': %s\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                    kw,
                                );
                                break '_error;
                            } else if *p.offset(1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == ']' as ::core::ffi::c_int
                            {
                                if *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                    != NUL
                                {
                                    semsg(
                                        gettext(
                                            (e_trailing_char_after_rsb_str_str.ptr() as *const _)
                                                as *const ::core::ffi::c_char,
                                        ),
                                        kw,
                                        p.offset(2 as ::core::ffi::c_int as isize),
                                    );
                                    break '_error;
                                } else {
                                    kw = p.offset(1 as ::core::ffi::c_int as isize);
                                    kwlen = 1 as size_t;
                                    break;
                                }
                            } else {
                                let l: ::core::ffi::c_int =
                                    utfc_ptr2len(p.offset(1 as ::core::ffi::c_int as isize));
                                memmove(
                                    p as *mut ::core::ffi::c_void,
                                    p.offset(1 as ::core::ffi::c_int as isize)
                                        as *const ::core::ffi::c_void,
                                    l as size_t,
                                );
                                p = p.offset(l as isize);
                            }
                        }
                        kw = kw.offset(kwlen.wrapping_add(1 as size_t) as isize);
                    }
                }
            }
            xfree(keyword_copy as *mut ::core::ffi::c_void);
            xfree(syn_opt_arg.cont_in_list as *mut ::core::ffi::c_void);
            xfree(syn_opt_arg.next_list as *mut ::core::ffi::c_void);
        }
    }
    if !rest.is_null() {
        (*eap).nextcmd = check_nextcmd(rest);
    } else {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            arg,
        );
    }
    redraw_curbuf_later(UPD_SOME_VALID);
    syn_stack_free_all((*curwin.get()).w_s);
}
unsafe extern "C" fn syn_cmd_match(mut eap: *mut exarg_T, mut syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut group_name_end: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut item: synpat_T = synpat_T {
        sp_type: 0,
        sp_syncing: false,
        sp_syn_match_id: 0,
        sp_off_flags: 0,
        sp_offsets: [0; 7],
        sp_flags: 0,
        sp_cchar: 0,
        sp_ic: 0,
        sp_sync_idx: 0,
        sp_line_id: 0,
        sp_startcol: 0,
        sp_cont_list: ::core::ptr::null_mut::<int16_t>(),
        sp_next_list: ::core::ptr::null_mut::<int16_t>(),
        sp_syn: sp_syn {
            inc_tag: 0,
            id: 0,
            cont_in_list: ::core::ptr::null_mut::<int16_t>(),
        },
        sp_pattern: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        sp_prog: ::core::ptr::null_mut::<regprog_T>(),
        sp_time: syn_time_T {
            total: 0,
            slowest: 0,
            count: 0,
            match_0: 0,
        },
    };
    let mut syn_id: ::core::ffi::c_int = 0;
    let mut syn_opt_arg: syn_opt_arg_T = syn_opt_arg_T {
        flags: 0,
        keyword: false,
        sync_idx: ::core::ptr::null_mut::<::core::ffi::c_int>(),
        has_cont_list: false,
        cont_list: ::core::ptr::null_mut::<int16_t>(),
        cont_in_list: ::core::ptr::null_mut::<int16_t>(),
        next_list: ::core::ptr::null_mut::<int16_t>(),
    };
    let mut sync_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut conceal_char: ::core::ffi::c_int = NUL;
    let mut rest: *mut ::core::ffi::c_char = get_group_name(arg, &raw mut group_name_end);
    syn_opt_arg.flags = 0 as ::core::ffi::c_int;
    syn_opt_arg.keyword = false_0 != 0;
    syn_opt_arg.sync_idx = if syncing != 0 {
        &raw mut sync_idx
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_int>()
    };
    syn_opt_arg.has_cont_list = true_0 != 0;
    syn_opt_arg.cont_list = ::core::ptr::null_mut::<int16_t>();
    syn_opt_arg.cont_in_list = ::core::ptr::null_mut::<int16_t>();
    syn_opt_arg.next_list = ::core::ptr::null_mut::<int16_t>();
    rest = get_syn_options(
        rest,
        &raw mut syn_opt_arg,
        &raw mut conceal_char,
        (*eap).skip,
    );
    init_syn_patterns();
    memset(
        &raw mut item as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<synpat_T>(),
    );
    rest = get_syn_pattern(rest, &raw mut item);
    if vim_regcomp_had_eol() != 0 && syn_opt_arg.flags & HL_EXCLUDENL == 0 {
        syn_opt_arg.flags |= HL_HAS_EOL;
    }
    rest = get_syn_options(
        rest,
        &raw mut syn_opt_arg,
        &raw mut conceal_char,
        (*eap).skip,
    );
    if !rest.is_null() {
        (*eap).nextcmd = check_nextcmd(rest);
        if ends_excmd(*rest as ::core::ffi::c_int) == 0 || (*eap).skip != 0 {
            rest = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            syn_id = syn_check_group(arg, group_name_end.offset_from(arg) as size_t);
            if syn_id != 0 as ::core::ffi::c_int {
                syn_incl_toplevel(syn_id, &raw mut syn_opt_arg.flags);
                let mut spp: *mut synpat_T = ga_append_via_ptr(
                    &raw mut (*(*curwin.get()).w_s).b_syn_patterns,
                    ::core::mem::size_of::<synpat_T>(),
                ) as *mut synpat_T;
                *spp = item;
                (*spp).sp_syncing = syncing != 0;
                (*spp).sp_type = SPTYPE_MATCH as ::core::ffi::c_char;
                (*spp).sp_syn.id = syn_id as int16_t;
                (*spp).sp_syn.inc_tag = current_syn_inc_tag.get();
                (*spp).sp_flags = syn_opt_arg.flags;
                (*spp).sp_sync_idx = sync_idx;
                (*spp).sp_cont_list = syn_opt_arg.cont_list;
                (*spp).sp_syn.cont_in_list = syn_opt_arg.cont_in_list;
                (*spp).sp_cchar = conceal_char;
                if !syn_opt_arg.cont_in_list.is_null() {
                    (*(*curwin.get()).w_s).b_syn_containedin = true_0;
                }
                (*spp).sp_next_list = syn_opt_arg.next_list;
                if syn_opt_arg.flags & (HL_SYNC_HERE | HL_SYNC_THERE) != 0 {
                    (*(*curwin.get()).w_s).b_syn_sync_flags |= SF_MATCH;
                }
                if syn_opt_arg.flags & HL_FOLD != 0 {
                    (*(*curwin.get()).w_s).b_syn_folditems += 1;
                }
                redraw_curbuf_later(UPD_SOME_VALID);
                syn_stack_free_all((*curwin.get()).w_s);
                return;
            }
        }
    }
    vim_regfree(item.sp_prog);
    xfree(item.sp_pattern as *mut ::core::ffi::c_void);
    xfree(syn_opt_arg.cont_list as *mut ::core::ffi::c_void);
    xfree(syn_opt_arg.cont_in_list as *mut ::core::ffi::c_void);
    xfree(syn_opt_arg.next_list as *mut ::core::ffi::c_void);
    if rest.is_null() {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            arg,
        );
    }
}
unsafe extern "C" fn syn_cmd_region(mut eap: *mut exarg_T, mut syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut group_name_end: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut rest: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut item: ::core::ffi::c_int = 0;
    let mut pat_ptrs: [*mut pat_ptr; 3] = [::core::ptr::null_mut::<pat_ptr>(); 3];
    let mut ppp: *mut pat_ptr = ::core::ptr::null_mut::<pat_ptr>();
    let mut ppp_next: *mut pat_ptr = ::core::ptr::null_mut::<pat_ptr>();
    let mut pat_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut syn_id: ::core::ffi::c_int = 0;
    let mut matchgroup_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut not_enough: bool = false_0 != 0;
    let mut illegal: bool = false_0 != 0;
    let mut success: bool = false_0 != 0;
    let mut syn_opt_arg: syn_opt_arg_T = syn_opt_arg_T {
        flags: 0,
        keyword: false,
        sync_idx: ::core::ptr::null_mut::<::core::ffi::c_int>(),
        has_cont_list: false,
        cont_list: ::core::ptr::null_mut::<int16_t>(),
        cont_in_list: ::core::ptr::null_mut::<int16_t>(),
        next_list: ::core::ptr::null_mut::<int16_t>(),
    };
    let mut conceal_char: ::core::ffi::c_int = NUL;
    rest = get_group_name(arg, &raw mut group_name_end);
    pat_ptrs[0 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<pat_ptr>();
    pat_ptrs[1 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<pat_ptr>();
    pat_ptrs[2 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<pat_ptr>();
    init_syn_patterns();
    syn_opt_arg.flags = 0 as ::core::ffi::c_int;
    syn_opt_arg.keyword = false_0 != 0;
    syn_opt_arg.sync_idx = ::core::ptr::null_mut::<::core::ffi::c_int>();
    syn_opt_arg.has_cont_list = true_0 != 0;
    syn_opt_arg.cont_list = ::core::ptr::null_mut::<int16_t>();
    syn_opt_arg.cont_in_list = ::core::ptr::null_mut::<int16_t>();
    syn_opt_arg.next_list = ::core::ptr::null_mut::<int16_t>();
    while !rest.is_null() && ends_excmd(*rest as ::core::ffi::c_int) == 0 {
        rest = get_syn_options(
            rest,
            &raw mut syn_opt_arg,
            &raw mut conceal_char,
            (*eap).skip,
        );
        if rest.is_null() || ends_excmd(*rest as ::core::ffi::c_int) != 0 {
            break;
        }
        key_end = rest;
        while *key_end as ::core::ffi::c_int != 0
            && !ascii_iswhite(*key_end as ::core::ffi::c_int)
            && *key_end as ::core::ffi::c_int != '=' as ::core::ffi::c_int
        {
            key_end = key_end.offset(1);
        }
        xfree(key as *mut ::core::ffi::c_void);
        key = vim_strnsave_up(rest, key_end.offset_from(rest) as size_t);
        if strcmp(key, b"MATCHGROUP\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            item = ITEM_MATCHGROUP;
        } else if strcmp(key, b"START\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            item = ITEM_START;
        } else if strcmp(key, b"END\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            item = ITEM_END;
        } else {
            if strcmp(key, b"SKIP\0".as_ptr() as *const ::core::ffi::c_char)
                != 0 as ::core::ffi::c_int
            {
                break;
            }
            if !pat_ptrs[ITEM_SKIP as usize].is_null() {
                illegal = true_0 != 0;
                break;
            } else {
                item = ITEM_SKIP;
            }
        }
        rest = skipwhite(key_end);
        if *rest as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
            rest = ::core::ptr::null_mut::<::core::ffi::c_char>();
            semsg(
                gettext(b"E398: Missing '=': %s\0".as_ptr() as *const ::core::ffi::c_char),
                arg,
            );
            break;
        } else {
            rest = skipwhite(rest.offset(1 as ::core::ffi::c_int as isize));
            if *rest as ::core::ffi::c_int == NUL {
                not_enough = true_0 != 0;
                break;
            } else if item == ITEM_MATCHGROUP {
                let mut p: *mut ::core::ffi::c_char = skiptowhite(rest);
                if p.offset_from(rest) == 4 as isize
                    && strncmp(
                        rest,
                        b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
                        4 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    || (*eap).skip != 0
                {
                    matchgroup_id = 0 as ::core::ffi::c_int;
                } else {
                    matchgroup_id = syn_check_group(rest, p.offset_from(rest) as size_t);
                    if matchgroup_id == 0 as ::core::ffi::c_int {
                        illegal = true_0 != 0;
                        break;
                    }
                }
                rest = skipwhite(p);
            } else {
                ppp = xmalloc(::core::mem::size_of::<pat_ptr>()) as *mut pat_ptr;
                (*ppp).pp_next = pat_ptrs[item as usize] as *mut pat_ptr;
                pat_ptrs[item as usize] = ppp as *mut pat_ptr;
                (*ppp).pp_synp =
                    xcalloc(1 as size_t, ::core::mem::size_of::<synpat_T>()) as *mut synpat_T;
                if item == ITEM_START {
                    reg_do_extmatch.set(REX_SET);
                } else {
                    '_c2rust_label: {
                        if item == 1 as ::core::ffi::c_int || item == 2 as ::core::ffi::c_int {
                        } else {
                            __assert_fail(
                                b"item == ITEM_SKIP || item == ITEM_END\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/syntax.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                4333 as ::core::ffi::c_uint,
                                b"void syn_cmd_region(exarg_T *, int)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    reg_do_extmatch.set(REX_USE);
                }
                rest = get_syn_pattern(rest, (*ppp).pp_synp);
                reg_do_extmatch.set(0 as ::core::ffi::c_int);
                if item == ITEM_END
                    && vim_regcomp_had_eol() != 0
                    && syn_opt_arg.flags & HL_EXCLUDENL == 0
                {
                    (*(*ppp).pp_synp).sp_flags |= HL_HAS_EOL;
                }
                (*ppp).pp_matchgroup_id = matchgroup_id;
                pat_count += 1;
            }
        }
    }
    xfree(key as *mut ::core::ffi::c_void);
    if illegal as ::core::ffi::c_int != 0 || not_enough as ::core::ffi::c_int != 0 {
        rest = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if !rest.is_null()
        && (pat_ptrs[ITEM_START as usize].is_null() || pat_ptrs[ITEM_END as usize].is_null())
    {
        not_enough = true_0 != 0;
        rest = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if !rest.is_null() {
        (*eap).nextcmd = check_nextcmd(rest);
        if ends_excmd(*rest as ::core::ffi::c_int) == 0 || (*eap).skip != 0 {
            rest = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            ga_grow(&raw mut (*(*curwin.get()).w_s).b_syn_patterns, pat_count);
            syn_id = syn_check_group(arg, group_name_end.offset_from(arg) as size_t);
            if syn_id != 0 as ::core::ffi::c_int {
                syn_incl_toplevel(syn_id, &raw mut syn_opt_arg.flags);
                let mut idx: ::core::ffi::c_int = (*(*curwin.get()).w_s).b_syn_patterns.ga_len;
                item = ITEM_START;
                while item <= ITEM_END {
                    ppp = pat_ptrs[item as usize] as *mut pat_ptr;
                    while !ppp.is_null() {
                        *((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize) = *(*ppp).pp_synp;
                        (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize))
                        .sp_syncing = syncing != 0;
                        (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize))
                        .sp_type = (if item == ITEM_START {
                            SPTYPE_START
                        } else if item == ITEM_SKIP {
                            SPTYPE_SKIP
                        } else {
                            SPTYPE_END
                        }) as ::core::ffi::c_char;
                        (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize))
                        .sp_flags |= syn_opt_arg.flags;
                        (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize))
                        .sp_syn
                        .id = syn_id as int16_t;
                        (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize))
                        .sp_syn
                        .inc_tag = current_syn_inc_tag.get();
                        (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize))
                        .sp_syn_match_id = (*ppp).pp_matchgroup_id as int16_t;
                        (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize))
                        .sp_cchar = conceal_char;
                        if item == ITEM_START {
                            (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(idx as isize))
                            .sp_cont_list = syn_opt_arg.cont_list;
                            (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(idx as isize))
                            .sp_syn
                            .cont_in_list = syn_opt_arg.cont_in_list;
                            if !syn_opt_arg.cont_in_list.is_null() {
                                (*(*curwin.get()).w_s).b_syn_containedin = true_0;
                            }
                            (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(idx as isize))
                            .sp_next_list = syn_opt_arg.next_list;
                        }
                        (*(*curwin.get()).w_s).b_syn_patterns.ga_len += 1;
                        idx += 1;
                        if syn_opt_arg.flags & HL_FOLD != 0 {
                            (*(*curwin.get()).w_s).b_syn_folditems += 1;
                        }
                        ppp = (*ppp).pp_next;
                    }
                    item += 1;
                }
                redraw_curbuf_later(UPD_SOME_VALID);
                syn_stack_free_all((*curwin.get()).w_s);
                success = true_0 != 0;
            }
        }
    }
    item = ITEM_START;
    while item <= ITEM_END {
        ppp = pat_ptrs[item as usize] as *mut pat_ptr;
        while !ppp.is_null() {
            if !success && !(*ppp).pp_synp.is_null() {
                vim_regfree((*(*ppp).pp_synp).sp_prog);
                xfree((*(*ppp).pp_synp).sp_pattern as *mut ::core::ffi::c_void);
            }
            xfree((*ppp).pp_synp as *mut ::core::ffi::c_void);
            ppp_next = (*ppp).pp_next;
            xfree(ppp as *mut ::core::ffi::c_void);
            ppp = ppp_next;
        }
        item += 1;
    }
    if !success {
        xfree(syn_opt_arg.cont_list as *mut ::core::ffi::c_void);
        xfree(syn_opt_arg.cont_in_list as *mut ::core::ffi::c_void);
        xfree(syn_opt_arg.next_list as *mut ::core::ffi::c_void);
        if not_enough {
            semsg(
                gettext(b"E399: Not enough arguments: syntax region %s\0".as_ptr()
                    as *const ::core::ffi::c_char),
                arg,
            );
        } else if illegal as ::core::ffi::c_int != 0 || rest.is_null() {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                arg,
            );
        }
    }
}
pub const ITEM_START: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ITEM_SKIP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ITEM_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ITEM_MATCHGROUP: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
unsafe extern "C" fn syn_compare_stub(
    v1: *const ::core::ffi::c_void,
    v2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let s1: *const int16_t = v1 as *const int16_t;
    let s2: *const int16_t = v2 as *const int16_t;
    return if *s1 as ::core::ffi::c_int > *s2 as ::core::ffi::c_int {
        1 as ::core::ffi::c_int
    } else if (*s1 as ::core::ffi::c_int) < *s2 as ::core::ffi::c_int {
        -1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
}
unsafe extern "C" fn syn_combine_list(
    clstr1: *mut *mut int16_t,
    clstr2: *mut *mut int16_t,
    list_op: ::core::ffi::c_int,
) {
    let mut count1: size_t = 0 as size_t;
    let mut count2: size_t = 0 as size_t;
    let mut g1: *const int16_t = ::core::ptr::null::<int16_t>();
    let mut g2: *const int16_t = ::core::ptr::null::<int16_t>();
    let mut clstr: *mut int16_t = ::core::ptr::null_mut::<int16_t>();
    if (*clstr2).is_null() {
        return;
    }
    if (*clstr1).is_null() || list_op == CLUSTER_REPLACE {
        if list_op == CLUSTER_REPLACE {
            xfree(*clstr1 as *mut ::core::ffi::c_void);
        }
        if list_op == CLUSTER_REPLACE || list_op == CLUSTER_ADD {
            *clstr1 = *clstr2;
        } else {
            xfree(*clstr2 as *mut ::core::ffi::c_void);
        }
        return;
    }
    g1 = *clstr1;
    while *g1 != 0 {
        count1 = count1.wrapping_add(1);
        g1 = g1.offset(1);
    }
    g2 = *clstr2;
    while *g2 != 0 {
        count2 = count2.wrapping_add(1);
        g2 = g2.offset(1);
    }
    qsort(
        *clstr1 as *mut ::core::ffi::c_void,
        count1,
        ::core::mem::size_of::<int16_t>(),
        Some(
            syn_compare_stub
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    qsort(
        *clstr2 as *mut ::core::ffi::c_void,
        count2,
        ::core::mem::size_of::<int16_t>(),
        Some(
            syn_compare_stub
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while round <= 2 as ::core::ffi::c_int {
        g1 = *clstr1;
        g2 = *clstr2;
        let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while *g1 as ::core::ffi::c_int != 0 && *g2 as ::core::ffi::c_int != 0 {
            if (*g1 as ::core::ffi::c_int) < *g2 as ::core::ffi::c_int {
                if round == 2 as ::core::ffi::c_int {
                    *clstr.offset(count as isize) = *g1;
                }
                count += 1;
                g1 = g1.offset(1);
            } else {
                if list_op == CLUSTER_ADD {
                    if round == 2 as ::core::ffi::c_int {
                        *clstr.offset(count as isize) = *g2;
                    }
                    count += 1;
                }
                if *g1 as ::core::ffi::c_int == *g2 as ::core::ffi::c_int {
                    g1 = g1.offset(1);
                }
                g2 = g2.offset(1);
            }
        }
        while *g1 != 0 {
            if round == 2 as ::core::ffi::c_int {
                *clstr.offset(count as isize) = *g1;
            }
            g1 = g1.offset(1);
            count += 1;
        }
        if list_op == CLUSTER_ADD {
            while *g2 != 0 {
                if round == 2 as ::core::ffi::c_int {
                    *clstr.offset(count as isize) = *g2;
                }
                g2 = g2.offset(1);
                count += 1;
            }
        }
        if round == 1 as ::core::ffi::c_int {
            if count == 0 as ::core::ffi::c_int {
                clstr = ::core::ptr::null_mut::<int16_t>();
                break;
            } else {
                clstr = xmalloc(
                    (count as size_t)
                        .wrapping_add(1 as size_t)
                        .wrapping_mul(::core::mem::size_of::<int16_t>()),
                ) as *mut int16_t;
                *clstr.offset(count as isize) = 0 as int16_t;
            }
        }
        round += 1;
    }
    xfree(*clstr1 as *mut ::core::ffi::c_void);
    xfree(*clstr2 as *mut ::core::ffi::c_void);
    *clstr1 = clstr;
}
unsafe extern "C" fn syn_scl_name2id(mut name: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut name_u: *mut ::core::ffi::c_char = vim_strsave_up(name);
    let mut i: ::core::ffi::c_int = 0;
    i = (*(*curwin.get()).w_s).b_syn_clusters.ga_len;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        if !(*((*(*curwin.get()).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
            .offset(i as isize))
        .scl_name_u
        .is_null()
            && strcmp(
                name_u,
                (*((*(*curwin.get()).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                    .offset(i as isize))
                .scl_name_u,
            ) == 0 as ::core::ffi::c_int
        {
            break;
        }
    }
    xfree(name_u as *mut ::core::ffi::c_void);
    return if i < 0 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        i + SYNID_CLUSTER
    };
}
unsafe extern "C" fn syn_scl_namen2id(
    mut linep: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut name: *mut ::core::ffi::c_char = xstrnsave(linep, len as size_t);
    let mut id: ::core::ffi::c_int = syn_scl_name2id(name);
    xfree(name as *mut ::core::ffi::c_void);
    return id;
}
unsafe extern "C" fn syn_check_cluster(
    mut pp: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut name: *mut ::core::ffi::c_char = xstrnsave(pp, len as size_t);
    let mut id: ::core::ffi::c_int = syn_scl_name2id(name);
    if id == 0 as ::core::ffi::c_int {
        id = syn_add_cluster(name);
    } else {
        xfree(name as *mut ::core::ffi::c_void);
    }
    return id;
}
unsafe extern "C" fn syn_add_cluster(mut name: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    if (*(*curwin.get()).w_s).b_syn_clusters.ga_data.is_null() {
        (*(*curwin.get()).w_s).b_syn_clusters.ga_itemsize =
            ::core::mem::size_of::<syn_cluster_T>() as ::core::ffi::c_int;
        ga_set_growsize(
            &raw mut (*(*curwin.get()).w_s).b_syn_clusters,
            10 as ::core::ffi::c_int,
        );
    }
    let mut len: ::core::ffi::c_int = (*(*curwin.get()).w_s).b_syn_clusters.ga_len;
    if len >= MAX_CLUSTER_ID {
        emsg(gettext(
            b"E848: Too many syntax clusters\0".as_ptr() as *const ::core::ffi::c_char
        ));
        xfree(name as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    let mut scp: *mut syn_cluster_T = ga_append_via_ptr(
        &raw mut (*(*curwin.get()).w_s).b_syn_clusters,
        ::core::mem::size_of::<syn_cluster_T>(),
    ) as *mut syn_cluster_T;
    memset(
        scp as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<syn_cluster_T>(),
    );
    (*scp).scl_name = name;
    (*scp).scl_name_u = vim_strsave_up(name);
    (*scp).scl_list = ::core::ptr::null_mut::<int16_t>();
    if strcasecmp(
        name,
        b"Spell\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        (*(*curwin.get()).w_s).b_spell_cluster_id = len + SYNID_CLUSTER;
    }
    if strcasecmp(
        name,
        b"NoSpell\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        (*(*curwin.get()).w_s).b_nospell_cluster_id = len + SYNID_CLUSTER;
    }
    return len + SYNID_CLUSTER;
}
unsafe extern "C" fn syn_cmd_cluster(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut group_name_end: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut got_clstr: bool = false_0 != 0;
    let mut opt_len: ::core::ffi::c_int = 0;
    let mut list_op: ::core::ffi::c_int = 0;
    (*eap).nextcmd = find_nextcmd(arg);
    if (*eap).skip != 0 {
        return;
    }
    let mut rest: *mut ::core::ffi::c_char = get_group_name(arg, &raw mut group_name_end);
    if !rest.is_null() {
        let mut scl_id: ::core::ffi::c_int =
            syn_check_cluster(arg, group_name_end.offset_from(arg) as ::core::ffi::c_int);
        if scl_id == 0 as ::core::ffi::c_int {
            return;
        }
        scl_id -= SYNID_CLUSTER;
        loop {
            if strncasecmp(
                rest,
                b"add\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                3 as ::core::ffi::c_int as size_t,
            ) == 0 as ::core::ffi::c_int
                && (ascii_iswhite(
                    *rest.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
                    || *rest.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int)
            {
                opt_len = 3 as ::core::ffi::c_int;
                list_op = CLUSTER_ADD;
            } else if strncasecmp(
                rest,
                b"remove\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                6 as ::core::ffi::c_int as size_t,
            ) == 0 as ::core::ffi::c_int
                && (ascii_iswhite(
                    *rest.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
                    || *rest.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int)
            {
                opt_len = 6 as ::core::ffi::c_int;
                list_op = CLUSTER_SUBTRACT;
            } else {
                if !(strncasecmp(
                    rest,
                    b"contains\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    8 as ::core::ffi::c_int as size_t,
                ) == 0 as ::core::ffi::c_int
                    && (ascii_iswhite(
                        *rest.offset(8 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                        || *rest.offset(8 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '=' as ::core::ffi::c_int))
                {
                    break;
                }
                opt_len = 8 as ::core::ffi::c_int;
                list_op = CLUSTER_REPLACE;
            }
            let mut clstr_list: *mut int16_t = ::core::ptr::null_mut::<int16_t>();
            if get_id_list(
                &raw mut rest,
                opt_len,
                &raw mut clstr_list,
                (*eap).skip != 0,
            ) == FAIL
            {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    rest,
                );
                break;
            } else {
                if scl_id >= 0 as ::core::ffi::c_int {
                    syn_combine_list(
                        &raw mut (*((*(*curwin.get()).w_s).b_syn_clusters.ga_data
                            as *mut syn_cluster_T)
                            .offset(scl_id as isize))
                        .scl_list,
                        &raw mut clstr_list,
                        list_op,
                    );
                } else {
                    xfree(clstr_list as *mut ::core::ffi::c_void);
                }
                got_clstr = true_0 != 0;
            }
        }
        if got_clstr {
            redraw_curbuf_later(UPD_SOME_VALID);
            syn_stack_free_all((*curwin.get()).w_s);
        }
    }
    if !got_clstr {
        emsg(gettext(
            b"E400: No cluster specified\0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
    if rest.is_null() || ends_excmd(*rest as ::core::ffi::c_int) == 0 {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            arg,
        );
    }
}
unsafe extern "C" fn init_syn_patterns() {
    (*(*curwin.get()).w_s).b_syn_patterns.ga_itemsize =
        ::core::mem::size_of::<synpat_T>() as ::core::ffi::c_int;
    ga_set_growsize(
        &raw mut (*(*curwin.get()).w_s).b_syn_patterns,
        10 as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn get_syn_pattern(
    mut arg: *mut ::core::ffi::c_char,
    mut ci: *mut synpat_T,
) -> *mut ::core::ffi::c_char {
    let mut idx: ::core::ffi::c_int = 0;
    if arg.is_null()
        || *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        || *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        || *arg.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut end: *mut ::core::ffi::c_char = skip_regexp(
        arg.offset(1 as ::core::ffi::c_int as isize),
        *arg as ::core::ffi::c_int,
        true_0,
    );
    if *end as ::core::ffi::c_int != *arg as ::core::ffi::c_int {
        semsg(
            gettext(
                b"E401: Pattern delimiter not found: %s\0".as_ptr() as *const ::core::ffi::c_char
            ),
            arg,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    (*ci).sp_pattern = xstrnsave(
        arg.offset(1 as ::core::ffi::c_int as isize),
        (end.offset_from(arg) as size_t).wrapping_sub(1 as size_t),
    );
    let mut cpo_save: *mut ::core::ffi::c_char = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
    (*ci).sp_prog = vim_regcomp((*ci).sp_pattern, RE_MAGIC);
    p_cpo.set(cpo_save);
    if (*ci).sp_prog.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    (*ci).sp_ic = (*(*curwin.get()).w_s).b_syn_ic;
    syn_clear_time(&raw mut (*ci).sp_time);
    end = end.offset(1);
    loop {
        idx = SPO_COUNT;
        loop {
            idx -= 1;
            if idx < 0 as ::core::ffi::c_int {
                break;
            }
            if strncmp(
                end,
                (*spo_name_tab.ptr())[idx as usize] as *const ::core::ffi::c_char,
                3 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                break;
            }
        }
        if idx >= 0 as ::core::ffi::c_int {
            let mut p: *mut ::core::ffi::c_int =
                (&raw mut (*ci).sp_offsets as *mut ::core::ffi::c_int).offset(idx as isize);
            if idx != SPO_LC_OFF {
                match *end.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                    115 | 98 => {}
                    101 => {
                        idx += SPO_COUNT;
                    }
                    _ => {
                        idx = -1 as ::core::ffi::c_int;
                    }
                }
            }
            if idx >= 0 as ::core::ffi::c_int {
                (*ci).sp_off_flags = ((*ci).sp_off_flags as ::core::ffi::c_int
                    | ((1 as ::core::ffi::c_int) << idx) as int16_t as ::core::ffi::c_int)
                    as int16_t;
                if idx == SPO_LC_OFF {
                    end = end.offset(3 as ::core::ffi::c_int as isize);
                    *p = getdigits_int(&raw mut end, true_0 != 0, 0 as ::core::ffi::c_int);
                    if (*ci).sp_off_flags as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_int) << SPO_MS_OFF
                        == 0
                    {
                        (*ci).sp_off_flags = ((*ci).sp_off_flags as ::core::ffi::c_int
                            | (1 as ::core::ffi::c_int) << SPO_MS_OFF)
                            as int16_t;
                        (*ci).sp_offsets[SPO_MS_OFF as usize] = *p;
                    }
                } else {
                    end = end.offset(4 as ::core::ffi::c_int as isize);
                    if *end as ::core::ffi::c_int == '+' as ::core::ffi::c_int {
                        end = end.offset(1);
                        *p = getdigits_int(&raw mut end, true_0 != 0, 0 as ::core::ffi::c_int);
                    } else if *end as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                        end = end.offset(1);
                        *p = -getdigits_int(&raw mut end, true_0 != 0, 0 as ::core::ffi::c_int);
                    }
                }
                if *end as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
                    break;
                }
                end = end.offset(1);
            }
        }
        if idx < 0 as ::core::ffi::c_int {
            break;
        }
    }
    if ends_excmd(*end as ::core::ffi::c_int) == 0 && !ascii_iswhite(*end as ::core::ffi::c_int) {
        semsg(
            gettext(b"E402: Garbage after pattern: %s\0".as_ptr() as *const ::core::ffi::c_char),
            arg,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return skipwhite(end);
}
unsafe extern "C" fn syn_cmd_sync(mut eap: *mut exarg_T, mut _syncing: ::core::ffi::c_int) {
    let mut arg_start: *mut ::core::ffi::c_char = (*eap).arg;
    let mut key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut illegal: bool = false_0 != 0;
    let mut finished: bool = false_0 != 0;
    if ends_excmd(*arg_start as ::core::ffi::c_int) != 0 {
        syn_cmd_list(eap, true_0);
        return;
    }
    while ends_excmd(*arg_start as ::core::ffi::c_int) == 0 {
        let mut arg_end: *mut ::core::ffi::c_char = skiptowhite(arg_start);
        let mut next_arg: *mut ::core::ffi::c_char = skipwhite(arg_end);
        xfree(key as *mut ::core::ffi::c_void);
        key = vim_strnsave_up(arg_start, arg_end.offset_from(arg_start) as size_t);
        if strcmp(key, b"CCOMMENT\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            if (*eap).skip == 0 {
                (*(*curwin.get()).w_s).b_syn_sync_flags |= SF_CCOMMENT;
            }
            if ends_excmd(*next_arg as ::core::ffi::c_int) == 0 {
                arg_end = skiptowhite(next_arg);
                if (*eap).skip == 0 {
                    (*(*curwin.get()).w_s).b_syn_sync_id =
                        syn_check_group(next_arg, arg_end.offset_from(next_arg) as size_t)
                            as int16_t;
                }
                next_arg = skipwhite(arg_end);
            } else if (*eap).skip == 0 {
                (*(*curwin.get()).w_s).b_syn_sync_id =
                    syn_name2id(b"Comment\0".as_ptr() as *const ::core::ffi::c_char) as int16_t;
            }
        } else if strncmp(
            key,
            b"LINES\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
            || strncmp(
                key,
                b"MINLINES\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            || strncmp(
                key,
                b"MAXLINES\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            || strncmp(
                key,
                b"LINEBREAKS\0".as_ptr() as *const ::core::ffi::c_char,
                10 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            if *key.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'S' as ::core::ffi::c_int
            {
                arg_end = key.offset(6 as ::core::ffi::c_int as isize);
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'L' as ::core::ffi::c_int
            {
                arg_end = key.offset(11 as ::core::ffi::c_int as isize);
            } else {
                arg_end = key.offset(9 as ::core::ffi::c_int as isize);
            }
            if *arg_end.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '=' as ::core::ffi::c_int
                || !ascii_isdigit(*arg_end as ::core::ffi::c_int)
            {
                illegal = true_0 != 0;
                break;
            } else {
                let mut n: linenr_T = getdigits_int32(&raw mut arg_end, false_0 != 0, 0 as int32_t);
                if (*eap).skip == 0 {
                    if *key.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'B' as ::core::ffi::c_int
                    {
                        (*(*curwin.get()).w_s).b_syn_sync_linebreaks = n;
                    } else if *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'A' as ::core::ffi::c_int
                    {
                        (*(*curwin.get()).w_s).b_syn_sync_maxlines = n;
                    } else {
                        (*(*curwin.get()).w_s).b_syn_sync_minlines = n;
                    }
                }
            }
        } else if strcmp(key, b"FROMSTART\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            if (*eap).skip == 0 {
                (*(*curwin.get()).w_s).b_syn_sync_minlines =
                    MAXLNUM as ::core::ffi::c_int as linenr_T;
                (*(*curwin.get()).w_s).b_syn_sync_maxlines = 0 as ::core::ffi::c_int as linenr_T;
            }
        } else if strcmp(key, b"LINECONT\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            if *next_arg as ::core::ffi::c_int == NUL {
                illegal = true_0 != 0;
                break;
            } else if !(*(*curwin.get()).w_s).b_syn_linecont_pat.is_null() {
                emsg(gettext(
                    b"E403: syntax sync: line continuations pattern specified twice\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                finished = true_0 != 0;
                break;
            } else {
                arg_end = skip_regexp(
                    next_arg.offset(1 as ::core::ffi::c_int as isize),
                    *next_arg as ::core::ffi::c_int,
                    true_0,
                );
                if *arg_end as ::core::ffi::c_int != *next_arg as ::core::ffi::c_int {
                    illegal = true_0 != 0;
                    break;
                } else {
                    if (*eap).skip == 0 {
                        (*(*curwin.get()).w_s).b_syn_linecont_pat = xstrnsave(
                            next_arg.offset(1 as ::core::ffi::c_int as isize),
                            (arg_end.offset_from(next_arg) as size_t).wrapping_sub(1 as size_t),
                        );
                        (*(*curwin.get()).w_s).b_syn_linecont_ic = (*(*curwin.get()).w_s).b_syn_ic;
                        let mut cpo_save: *mut ::core::ffi::c_char = p_cpo.get();
                        p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
                        (*(*curwin.get()).w_s).b_syn_linecont_prog =
                            vim_regcomp((*(*curwin.get()).w_s).b_syn_linecont_pat, RE_MAGIC);
                        p_cpo.set(cpo_save);
                        syn_clear_time(&raw mut (*(*curwin.get()).w_s).b_syn_linecont_time);
                        if (*(*curwin.get()).w_s).b_syn_linecont_prog.is_null() {
                            let mut ptr_: *mut *mut ::core::ffi::c_void =
                                &raw mut (*(*curwin.get()).w_s).b_syn_linecont_pat
                                    as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr_);
                            *ptr_ = NULL;
                            let _ = *ptr_;
                            finished = true_0 != 0;
                            break;
                        }
                    }
                    next_arg = skipwhite(arg_end.offset(1 as ::core::ffi::c_int as isize));
                }
            }
        } else {
            (*eap).arg = next_arg;
            if strcmp(key, b"MATCH\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                syn_cmd_match(eap, true_0);
            } else if strcmp(key, b"REGION\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                syn_cmd_region(eap, true_0);
            } else if strcmp(key, b"CLEAR\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                syn_cmd_clear(eap, true_0);
            } else {
                illegal = true_0 != 0;
            }
            finished = true_0 != 0;
            break;
        }
        arg_start = next_arg;
    }
    xfree(key as *mut ::core::ffi::c_void);
    if illegal {
        semsg(
            gettext(b"E404: Illegal arguments: %s\0".as_ptr() as *const ::core::ffi::c_char),
            arg_start,
        );
    } else if !finished {
        (*eap).nextcmd = check_nextcmd(arg_start);
        redraw_curbuf_later(UPD_SOME_VALID);
        syn_stack_free_all((*curwin.get()).w_s);
    }
}
unsafe extern "C" fn get_id_list(
    arg: *mut *mut ::core::ffi::c_char,
    keylen: ::core::ffi::c_int,
    list: *mut *mut int16_t,
    skip: bool,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut total_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut retval: *mut int16_t = ::core::ptr::null_mut::<int16_t>();
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut id: ::core::ffi::c_int = 0;
    let mut failed: bool = false_0 != 0;
    let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while round <= 2 as ::core::ffi::c_int {
        p = skipwhite((*arg).offset(keylen as isize));
        if *p as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
            semsg(
                gettext(b"E405: Missing equal sign: %s\0".as_ptr() as *const ::core::ffi::c_char),
                *arg,
            );
            break;
        } else {
            p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
            if ends_excmd(*p as ::core::ffi::c_int) != 0 {
                semsg(
                    gettext(b"E406: Empty argument: %s\0".as_ptr() as *const ::core::ffi::c_char),
                    *arg,
                );
                break;
            } else {
                let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                loop {
                    end = p;
                    while *end as ::core::ffi::c_int != 0
                        && !ascii_iswhite(*end as ::core::ffi::c_int)
                        && *end as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                    {
                        end = end.offset(1);
                    }
                    let name: *mut ::core::ffi::c_char =
                        xmalloc((end.offset_from(p) as size_t).wrapping_add(3 as size_t))
                            as *mut ::core::ffi::c_char;
                    xmemcpyz(
                        name.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        end.offset_from(p) as size_t,
                    );
                    if strcmp(
                        name.offset(1 as ::core::ffi::c_int as isize),
                        b"ALLBUT\0".as_ptr() as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                        || strcmp(
                            name.offset(1 as ::core::ffi::c_int as isize),
                            b"ALL\0".as_ptr() as *const ::core::ffi::c_char,
                        ) == 0 as ::core::ffi::c_int
                        || strcmp(
                            name.offset(1 as ::core::ffi::c_int as isize),
                            b"TOP\0".as_ptr() as *const ::core::ffi::c_char,
                        ) == 0 as ::core::ffi::c_int
                        || strcmp(
                            name.offset(1 as ::core::ffi::c_int as isize),
                            b"CONTAINED\0".as_ptr() as *const ::core::ffi::c_char,
                        ) == 0 as ::core::ffi::c_int
                    {
                        if (if (**arg as ::core::ffi::c_int) < 'a' as ::core::ffi::c_int
                            || **arg as ::core::ffi::c_int > 'z' as ::core::ffi::c_int
                        {
                            **arg as ::core::ffi::c_int
                        } else {
                            **arg as ::core::ffi::c_int
                                - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                        }) != 'C' as ::core::ffi::c_int
                        {
                            semsg(
                                gettext(b"E407: %s not allowed here\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                name.offset(1 as ::core::ffi::c_int as isize),
                            );
                            failed = true_0 != 0;
                            xfree(name as *mut ::core::ffi::c_void);
                            break;
                        } else if count != 0 as ::core::ffi::c_int {
                            semsg(
                                gettext(b"E408: %s must be first in contains list\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                name.offset(1 as ::core::ffi::c_int as isize),
                            );
                            failed = true_0 != 0;
                            xfree(name as *mut ::core::ffi::c_void);
                            break;
                        } else {
                            if *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == 'A' as ::core::ffi::c_int
                            {
                                id = MAX_HL_ID as ::core::ffi::c_int;
                            } else if *name.offset(1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == 'T' as ::core::ffi::c_int
                            {
                                id = SYNID_TOP;
                            } else {
                                id = SYNID_CONTAINED;
                            }
                            id += current_syn_inc_tag.get();
                        }
                    } else if *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '@' as ::core::ffi::c_int
                    {
                        if skip {
                            id = -1 as ::core::ffi::c_int;
                        } else {
                            id = syn_check_cluster(
                                name.offset(2 as ::core::ffi::c_int as isize),
                                (end.offset_from(p) - 1 as isize) as ::core::ffi::c_int,
                            );
                        }
                    } else if strpbrk(
                        name.offset(1 as ::core::ffi::c_int as isize),
                        b"\\.*^$~[\0".as_ptr() as *const ::core::ffi::c_char,
                    )
                    .is_null()
                    {
                        id = syn_check_group(
                            name.offset(1 as ::core::ffi::c_int as isize),
                            end.offset_from(p) as size_t,
                        );
                    } else {
                        *name = '^' as ::core::ffi::c_char;
                        strcat(name, b"$\0".as_ptr() as *const ::core::ffi::c_char);
                        regmatch.regprog = vim_regcomp(name, RE_MAGIC);
                        if regmatch.regprog.is_null() {
                            failed = true_0 != 0;
                            xfree(name as *mut ::core::ffi::c_void);
                            break;
                        } else {
                            regmatch.rm_ic = true_0 != 0;
                            id = 0 as ::core::ffi::c_int;
                            let mut i: ::core::ffi::c_int = highlight_num_groups();
                            loop {
                                i -= 1;
                                if i < 0 as ::core::ffi::c_int {
                                    break;
                                }
                                if vim_regexec(
                                    &raw mut regmatch,
                                    highlight_group_name(i),
                                    0 as colnr_T,
                                ) {
                                    if round == 2 as ::core::ffi::c_int {
                                        if count >= total_count {
                                            xfree(retval as *mut ::core::ffi::c_void);
                                            round = 1 as ::core::ffi::c_int;
                                        } else {
                                            *retval.offset(count as isize) =
                                                (i + 1 as ::core::ffi::c_int) as int16_t;
                                        }
                                    }
                                    count += 1;
                                    id = -1 as ::core::ffi::c_int;
                                }
                            }
                            vim_regfree(regmatch.regprog);
                        }
                    }
                    xfree(name as *mut ::core::ffi::c_void);
                    if id == 0 as ::core::ffi::c_int {
                        semsg(
                            gettext(b"E409: Unknown group name: %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            p,
                        );
                        failed = true_0 != 0;
                        break;
                    } else {
                        if id > 0 as ::core::ffi::c_int {
                            if round == 2 as ::core::ffi::c_int {
                                if count >= total_count {
                                    xfree(retval as *mut ::core::ffi::c_void);
                                    round = 1 as ::core::ffi::c_int;
                                } else {
                                    *retval.offset(count as isize) = id as int16_t;
                                }
                            }
                            count += 1;
                        }
                        p = skipwhite(end);
                        if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
                            break;
                        }
                        p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
                        if ends_excmd(*p as ::core::ffi::c_int) != 0 {
                            break;
                        }
                    }
                }
                if failed {
                    break;
                }
                if round == 1 as ::core::ffi::c_int {
                    retval = xmalloc(
                        (count as size_t)
                            .wrapping_add(1 as size_t)
                            .wrapping_mul(::core::mem::size_of::<int16_t>()),
                    ) as *mut int16_t;
                    *retval.offset(count as isize) = 0 as int16_t;
                    total_count = count;
                }
                round += 1;
            }
        }
    }
    *arg = p;
    if failed as ::core::ffi::c_int != 0 || retval.is_null() {
        xfree(retval as *mut ::core::ffi::c_void);
        return FAIL;
    }
    if (*list).is_null() {
        *list = retval;
    } else {
        xfree(retval as *mut ::core::ffi::c_void);
    }
    return OK;
}
unsafe extern "C" fn copy_id_list(list: *const int16_t) -> *mut int16_t {
    if list.is_null() {
        return ::core::ptr::null_mut::<int16_t>();
    }
    let mut count: ::core::ffi::c_int = 0;
    count = 0 as ::core::ffi::c_int;
    while *list.offset(count as isize) != 0 {
        count += 1;
    }
    let len: size_t = (count as size_t)
        .wrapping_add(1 as size_t)
        .wrapping_mul(::core::mem::size_of::<int16_t>());
    let retval: *mut int16_t = xmalloc(len) as *mut int16_t;
    memmove(
        retval as *mut ::core::ffi::c_void,
        list as *const ::core::ffi::c_void,
        len,
    );
    return retval;
}
unsafe extern "C" fn in_id_list(
    mut cur_si: *mut stateitem_T,
    mut list: *mut int16_t,
    mut ssp: *mut sp_syn,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = 0;
    let mut id: int16_t = (*ssp).id;
    static depth: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    if !cur_si.is_null() && !(*ssp).cont_in_list.is_null() && (*cur_si).si_flags & HL_MATCH == 0 {
        while (*cur_si).si_flags & HL_TRANS_CONT != 0
            && cur_si > (*current_state.ptr()).ga_data as *mut stateitem_T
        {
            cur_si = cur_si.offset(-1);
        }
        if (*cur_si).si_idx >= 0 as ::core::ffi::c_int
            && in_id_list(
                ::core::ptr::null_mut::<stateitem_T>(),
                (*ssp).cont_in_list,
                &raw mut (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                    .offset((*cur_si).si_idx as isize))
                .sp_syn,
                (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                    .offset((*cur_si).si_idx as isize))
                .sp_flags,
            ) != 0
        {
            return true_0;
        }
    }
    if list.is_null() {
        return false_0;
    }
    if list == ID_LIST_ALL {
        return (flags & HL_CONTAINED == 0) as ::core::ffi::c_int;
    }
    let mut toplevel: bool = flags & HL_CONTAINED == 0 || flags & HL_INCLUDED_TOPLEVEL != 0;
    let mut item: int16_t = *list;
    if item as ::core::ffi::c_int >= MAX_HL_ID as ::core::ffi::c_int
        && (item as ::core::ffi::c_int) < SYNID_CLUSTER
    {
        if (item as ::core::ffi::c_int) < SYNID_TOP {
            if item as ::core::ffi::c_int - MAX_HL_ID as ::core::ffi::c_int != (*ssp).inc_tag {
                return false_0;
            }
        } else if (item as ::core::ffi::c_int) < SYNID_CONTAINED {
            if item as ::core::ffi::c_int - SYNID_TOP != (*ssp).inc_tag || !toplevel {
                return false_0;
            }
        } else if item as ::core::ffi::c_int - SYNID_CONTAINED != (*ssp).inc_tag
            || toplevel as ::core::ffi::c_int != 0
        {
            return false_0;
        }
        list = list.offset(1);
        item = *list;
        retval = false_0;
    } else {
        retval = true_0;
    }
    while item as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        if item as ::core::ffi::c_int == id as ::core::ffi::c_int {
            return retval;
        }
        if item as ::core::ffi::c_int >= SYNID_CLUSTER {
            let mut scl_list: *mut int16_t = (*((*syn_block.get()).b_syn_clusters.ga_data
                as *mut syn_cluster_T)
                .offset((item as ::core::ffi::c_int - SYNID_CLUSTER) as isize))
            .scl_list;
            if !scl_list.is_null() && depth.get() < 30 as ::core::ffi::c_int {
                (*depth.ptr()) += 1;
                let mut r: ::core::ffi::c_int =
                    in_id_list(::core::ptr::null_mut::<stateitem_T>(), scl_list, ssp, flags);
                (*depth.ptr()) -= 1;
                if r != 0 {
                    return retval;
                }
            }
        }
        list = list.offset(1);
        item = *list;
    }
    return (retval == 0) as ::core::ffi::c_int;
}
static subcommands: GlobalCell<[subcommand; 19]> = GlobalCell::new([
    subcommand {
        name: b"case\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_case as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"clear\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_clear as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"cluster\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_cluster as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"conceal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_conceal as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"enable\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_on as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"foldlevel\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(
            syn_cmd_foldlevel as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> (),
        ),
    },
    subcommand {
        name: b"include\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_include as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"iskeyword\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(
            syn_cmd_iskeyword as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> (),
        ),
    },
    subcommand {
        name: b"keyword\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_keyword as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"list\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_list as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"manual\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_manual as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"match\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_match as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"on\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_on as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"off\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_off as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"region\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_region as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"reset\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_reset as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"spell\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_spell as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"sync\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_sync as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_list as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
]);
pub unsafe fn ex_syntax(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut subcmd_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    syn_cmdlinep.set((*eap).cmdlinep);
    subcmd_end = arg;
    while *subcmd_end as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *subcmd_end as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || *subcmd_end as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && *subcmd_end as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
    {
        subcmd_end = subcmd_end.offset(1);
    }
    let subcmd_name: *mut ::core::ffi::c_char =
        xstrnsave(arg, subcmd_end.offset_from(arg) as size_t);
    if (*eap).skip != 0 {
        (*emsg_skip.ptr()) += 1;
    }
    let mut i: size_t = 0;
    i = 0 as size_t;
    while i < ::core::mem::size_of::<[subcommand; 19]>()
        .wrapping_div(::core::mem::size_of::<subcommand>())
        .wrapping_div(
            (::core::mem::size_of::<[subcommand; 19]>()
                .wrapping_rem(::core::mem::size_of::<subcommand>())
                == 0) as ::core::ffi::c_int as usize,
        )
    {
        if strcmp(subcmd_name, (*subcommands.ptr())[i as usize].name) == 0 as ::core::ffi::c_int {
            (*eap).arg = skipwhite(subcmd_end);
            (*subcommands.ptr())[i as usize]
                .func
                .expect("non-null function pointer")(eap, false_0);
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if i == ::core::mem::size_of::<[subcommand; 19]>()
        .wrapping_div(::core::mem::size_of::<subcommand>())
        .wrapping_div(
            (::core::mem::size_of::<[subcommand; 19]>()
                .wrapping_rem(::core::mem::size_of::<subcommand>())
                == 0) as ::core::ffi::c_int as usize,
        )
    {
        semsg(
            gettext(
                b"E410: Invalid :syntax subcommand: %s\0".as_ptr() as *const ::core::ffi::c_char
            ),
            subcmd_name,
        );
    }
    xfree(subcmd_name as *mut ::core::ffi::c_void);
    if (*eap).skip != 0 {
        (*emsg_skip.ptr()) -= 1;
    }
}
pub unsafe fn ex_ownsyntax(mut eap: *mut exarg_T) {
    if (*curwin.get()).w_s == &raw mut (*(*curwin.get()).w_buffer).b_s {
        (*curwin.get()).w_s =
            xcalloc(1 as size_t, ::core::mem::size_of::<synblock_T>()) as *mut synblock_T;
        hash_init(&raw mut (*(*curwin.get()).w_s).b_keywtab);
        hash_init(&raw mut (*(*curwin.get()).w_s).b_keywtab_ic);
        (*curwin.get()).w_onebuf_opt.wo_spell = false_0;
        clear_string_option(&raw mut (*(*curwin.get()).w_s).b_p_spc);
        clear_string_option(&raw mut (*(*curwin.get()).w_s).b_p_spf);
        clear_string_option(&raw mut (*(*curwin.get()).w_s).b_p_spl);
        clear_string_option(&raw mut (*(*curwin.get()).w_s).b_p_spo);
        clear_string_option(&raw mut (*(*curwin.get()).w_s).b_syn_isk);
    }
    let mut old_value: *mut ::core::ffi::c_char =
        get_var_value(b"b:current_syntax\0".as_ptr() as *const ::core::ffi::c_char);
    if !old_value.is_null() {
        old_value = xstrdup(old_value);
    }
    apply_autocmds(
        EVENT_SYNTAX,
        (*eap).arg,
        (*curbuf.get()).b_fname,
        true_0 != 0,
        curbuf.get(),
    );
    let mut new_value: *mut ::core::ffi::c_char =
        get_var_value(b"b:current_syntax\0".as_ptr() as *const ::core::ffi::c_char);
    if !new_value.is_null() {
        set_internal_string_var(
            b"w:current_syntax\0".as_ptr() as *const ::core::ffi::c_char,
            new_value,
        );
    }
    if old_value.is_null() {
        do_unlet(
            b"b:current_syntax\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
            true_0 != 0,
        );
    } else {
        set_internal_string_var(
            b"b:current_syntax\0".as_ptr() as *const ::core::ffi::c_char,
            old_value,
        );
        xfree(old_value as *mut ::core::ffi::c_void);
    };
}
pub unsafe extern "C" fn syntax_present(mut win: *mut win_T) -> bool {
    return (*(*win).w_s).b_syn_patterns.ga_len != 0 as ::core::ffi::c_int
        || (*(*win).w_s).b_syn_clusters.ga_len != 0 as ::core::ffi::c_int
        || (*(*win).w_s).b_keywtab.ht_used > 0 as size_t
        || (*(*win).w_s).b_keywtab_ic.ht_used > 0 as size_t;
}
static expand_what: GlobalCell<C2Rust_Unnamed_24> = GlobalCell::new(EXP_SUBCMD);
pub unsafe extern "C" fn reset_expand_highlight() {
    include_none.set(0 as ::core::ffi::c_int);
    include_default.set(include_none.get());
    include_link.set(include_default.get());
}
pub unsafe extern "C" fn set_context_in_echohl_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) {
    (*xp).xp_context = EXPAND_HIGHLIGHT as ::core::ffi::c_int;
    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
    include_none.set(1 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn set_context_in_syntax_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) {
    (*xp).xp_context = EXPAND_SYNTAX as ::core::ffi::c_int;
    expand_what.set(EXP_SUBCMD);
    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
    include_link.set(0 as ::core::ffi::c_int);
    include_default.set(0 as ::core::ffi::c_int);
    if *arg as ::core::ffi::c_int == NUL {
        return;
    }
    let mut p: *const ::core::ffi::c_char = skiptowhite(arg);
    if *p as ::core::ffi::c_int == NUL {
        return;
    }
    (*xp).xp_pattern = skipwhite(p);
    if *skiptowhite((*xp).xp_pattern) as ::core::ffi::c_int != NUL {
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    } else if strncasecmp(
        arg as *mut ::core::ffi::c_char,
        b"case\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        p.offset_from(arg) as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        expand_what.set(EXP_CASE);
    } else if strncasecmp(
        arg as *mut ::core::ffi::c_char,
        b"spell\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        p.offset_from(arg) as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        expand_what.set(EXP_SPELL);
    } else if strncasecmp(
        arg as *mut ::core::ffi::c_char,
        b"sync\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        p.offset_from(arg) as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        expand_what.set(EXP_SYNC);
    } else if strncasecmp(
        arg as *mut ::core::ffi::c_char,
        b"list\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        p.offset_from(arg) as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        p = skipwhite(p);
        if *p as ::core::ffi::c_int == '@' as ::core::ffi::c_int {
            expand_what.set(EXP_CLUSTER);
        } else {
            (*xp).xp_context = EXPAND_HIGHLIGHT as ::core::ffi::c_int;
        }
    } else if strncasecmp(
        arg as *mut ::core::ffi::c_char,
        b"keyword\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        p.offset_from(arg) as size_t,
    ) == 0 as ::core::ffi::c_int
        || strncasecmp(
            arg as *mut ::core::ffi::c_char,
            b"region\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            p.offset_from(arg) as size_t,
        ) == 0 as ::core::ffi::c_int
        || strncasecmp(
            arg as *mut ::core::ffi::c_char,
            b"match\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            p.offset_from(arg) as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        (*xp).xp_context = EXPAND_HIGHLIGHT as ::core::ffi::c_int;
    } else {
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    };
}
pub unsafe extern "C" fn get_syntax_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    match expand_what.get() as ::core::ffi::c_uint {
        0 => {
            if idx < 0 as ::core::ffi::c_int
                || idx
                    >= ::core::mem::size_of::<[subcommand; 19]>()
                        .wrapping_div(::core::mem::size_of::<subcommand>())
                        .wrapping_div(
                            (::core::mem::size_of::<[subcommand; 19]>()
                                .wrapping_rem(::core::mem::size_of::<subcommand>())
                                == 0) as ::core::ffi::c_int as usize,
                        ) as ::core::ffi::c_int
            {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            return (*subcommands.ptr())[idx as usize].name;
        }
        1 => {
            static case_args: GlobalCell<[*mut ::core::ffi::c_char; 3]> = GlobalCell::new([
                b"match\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"ignore\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ]);
            return (*case_args.ptr())[idx as usize];
        }
        2 => {
            static spell_args: GlobalCell<[*mut ::core::ffi::c_char; 4]> = GlobalCell::new([
                b"toplevel\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"notoplevel\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"default\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ]);
            return (*spell_args.ptr())[idx as usize];
        }
        3 => {
            static sync_args: GlobalCell<[*mut ::core::ffi::c_char; 11]> = GlobalCell::new([
                b"ccomment\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"clear\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"fromstart\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"linebreaks=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"linecont\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"lines=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"match\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"maxlines=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"minlines=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"region\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ]);
            return (*sync_args.ptr())[idx as usize];
        }
        4 => {
            if idx < (*(*curwin.get()).w_s).b_syn_clusters.ga_len {
                vim_snprintf(
                    &raw mut (*xp).xp_buf as *mut ::core::ffi::c_char,
                    EXPAND_BUF_LEN as ::core::ffi::c_int as size_t,
                    b"@%s\0".as_ptr() as *const ::core::ffi::c_char,
                    (*((*(*curwin.get()).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                        .offset(idx as isize))
                    .scl_name,
                );
                return &raw mut (*xp).xp_buf as *mut ::core::ffi::c_char;
            } else {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        }
        _ => {}
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn syn_get_id(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut trans: ::core::ffi::c_int,
    mut spellp: *mut bool,
    mut keep_state: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if wp != syn_win.get()
        || (*wp).w_buffer != syn_buf.get()
        || lnum != current_lnum.get()
        || col < current_col.get()
    {
        syntax_start(wp, lnum);
    } else if col > current_col.get() {
        next_match_idx.set(-1 as ::core::ffi::c_int);
    }
    get_syntax_attr(col, spellp, keep_state != 0);
    return if trans != 0 {
        current_trans_id.get()
    } else {
        current_id.get()
    };
}
pub unsafe extern "C" fn get_syntax_info(
    mut seqnrp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    *seqnrp = current_seqnr.get();
    return current_flags.get();
}
pub unsafe extern "C" fn syn_get_sub_char() -> ::core::ffi::c_int {
    return current_sub_char.get();
}
pub unsafe extern "C" fn syn_get_stack_item(mut i: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if i >= (*current_state.ptr()).ga_len {
        invalidate_current_state();
        current_col.set(MAXCOL as ::core::ffi::c_int as colnr_T);
        return -1 as ::core::ffi::c_int;
    }
    return (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_id;
}
unsafe extern "C" fn syn_cur_foldlevel() -> ::core::ffi::c_int {
    let mut level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*current_state.ptr()).ga_len {
        if (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_flags
            & HL_FOLD
            != 0
        {
            level += 1;
        }
        i += 1;
    }
    return level;
}
pub unsafe extern "C" fn syn_get_foldlevel(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
) -> ::core::ffi::c_int {
    let mut level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*(*wp).w_s).b_syn_folditems != 0 as ::core::ffi::c_int
        && !(*(*wp).w_s).b_syn_error
        && !(*(*wp).w_s).b_syn_slow
    {
        syntax_start(wp, lnum);
        level = syn_cur_foldlevel();
        if (*(*wp).w_s).b_syn_foldlevel == SYNFLD_MINIMUM {
            let mut cur_level: ::core::ffi::c_int = level;
            let mut low_level: ::core::ffi::c_int = cur_level;
            while !current_finished.get() {
                syn_current_attr(
                    false_0 != 0,
                    false_0 != 0,
                    ::core::ptr::null_mut::<bool>(),
                    false_0 != 0,
                );
                cur_level = syn_cur_foldlevel();
                if cur_level < low_level {
                    low_level = cur_level;
                } else if cur_level > low_level {
                    level = low_level;
                }
                (*current_col.ptr()) += 1;
            }
        }
    }
    if level as OptInt > (*wp).w_onebuf_opt.wo_fdn {
        level = (*wp).w_onebuf_opt.wo_fdn as ::core::ffi::c_int;
        if level < 0 as ::core::ffi::c_int {
            level = 0 as ::core::ffi::c_int;
        }
    }
    return level;
}
pub unsafe fn ex_syntime(mut eap: *mut exarg_T) {
    if strcmp((*eap).arg, b"on\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        syn_time_on.set(true_0 != 0);
    } else if strcmp((*eap).arg, b"off\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        syn_time_on.set(false_0 != 0);
    } else if strcmp(
        (*eap).arg,
        b"clear\0".as_ptr() as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        syntime_clear();
    } else if strcmp(
        (*eap).arg,
        b"report\0".as_ptr() as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        syntime_report();
    } else {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            (*eap).arg,
        );
    };
}
unsafe extern "C" fn syn_clear_time(mut st: *mut syn_time_T) {
    (*st).total = profile_zero();
    (*st).slowest = profile_zero();
    (*st).count = 0 as ::core::ffi::c_int;
    (*st).match_0 = 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn syntime_clear() {
    let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
    if !syntax_present(curwin.get()) {
        msg(
            gettext(msg_no_items.ptr() as *mut ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
        return;
    }
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while idx < (*(*curwin.get()).w_s).b_syn_patterns.ga_len {
        spp = ((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
        syn_clear_time(&raw mut (*spp).sp_time);
        idx += 1;
    }
}
pub unsafe extern "C" fn get_syntime_arg(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    match idx {
        0 => {
            return b"on\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        1 => {
            return b"off\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        2 => {
            return b"clear\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        3 => {
            return b"report\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        _ => {}
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn syn_compare_syntime(
    mut v1: *const ::core::ffi::c_void,
    mut v2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut s1: *const time_entry_T = v1 as *const time_entry_T;
    let mut s2: *const time_entry_T = v2 as *const time_entry_T;
    return profile_cmp((*s1).total, (*s2).total);
}
unsafe extern "C" fn syntime_report() {
    if !syntax_present(curwin.get()) {
        msg(
            gettext(msg_no_items.ptr() as *mut ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
        return;
    }
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<time_entry_T>() as ::core::ffi::c_int,
        50 as ::core::ffi::c_int,
    );
    let mut total_total: proftime_T = profile_zero();
    let mut total_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *mut time_entry_T = ::core::ptr::null_mut::<time_entry_T>();
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while idx < (*(*curwin.get()).w_s).b_syn_patterns.ga_len {
        let mut spp: *mut synpat_T =
            ((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
        if (*spp).sp_time.count > 0 as ::core::ffi::c_int {
            p = ga_append_via_ptr(&raw mut ga, ::core::mem::size_of::<time_entry_T>())
                as *mut time_entry_T;
            (*p).total = (*spp).sp_time.total;
            total_total = profile_add(total_total, (*spp).sp_time.total);
            (*p).count = (*spp).sp_time.count;
            (*p).match_0 = (*spp).sp_time.match_0;
            total_count += (*spp).sp_time.count;
            (*p).slowest = (*spp).sp_time.slowest;
            let mut tm: proftime_T = profile_divide((*spp).sp_time.total, (*spp).sp_time.count);
            (*p).average = tm;
            (*p).id = (*spp).sp_syn.id as ::core::ffi::c_int;
            (*p).pattern = (*spp).sp_pattern;
        }
        idx += 1;
    }
    if ga.ga_len > 1 as ::core::ffi::c_int {
        qsort(
            ga.ga_data,
            ga.ga_len as size_t,
            ::core::mem::size_of::<time_entry_T>(),
            Some(
                syn_compare_syntime
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
    }
    msg_puts_title(gettext(
        b"  TOTAL      COUNT  MATCH   SLOWEST     AVERAGE   NAME               PATTERN\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
    let mut idx_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while idx_0 < ga.ga_len && !got_int.get() {
        p = (ga.ga_data as *mut time_entry_T).offset(idx_0 as isize);
        msg_puts(profile_msg((*p).total));
        msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        msg_advance(13 as ::core::ffi::c_int);
        msg_outnum((*p).count);
        msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        msg_advance(20 as ::core::ffi::c_int);
        msg_outnum((*p).match_0);
        msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        msg_advance(26 as ::core::ffi::c_int);
        msg_puts(profile_msg((*p).slowest));
        msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        msg_advance(38 as ::core::ffi::c_int);
        msg_puts(profile_msg((*p).average));
        msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        msg_advance(50 as ::core::ffi::c_int);
        msg_outtrans(
            highlight_group_name((*p).id - 1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
            false_0 != 0,
        );
        msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        msg_advance(69 as ::core::ffi::c_int);
        let mut len: ::core::ffi::c_int = 0;
        if Columns.get() < 80 as ::core::ffi::c_int {
            len = 20 as ::core::ffi::c_int;
        } else {
            len = Columns.get() - 70 as ::core::ffi::c_int;
        }
        let mut patlen: ::core::ffi::c_int = strlen((*p).pattern) as ::core::ffi::c_int;
        len = if len < patlen { len } else { patlen };
        msg_outtrans_len((*p).pattern, len, 0 as ::core::ffi::c_int, false_0 != 0);
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        idx_0 += 1;
    }
    ga_clear(&raw mut ga);
    if !got_int.get() {
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        msg_puts(profile_msg(total_total));
        msg_advance(13 as ::core::ffi::c_int);
        msg_outnum(total_count);
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const REX_SET: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const REX_USE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
unsafe extern "C" fn c2rust_run_static_initializers() {
    namelist1.set([
        keyvalue_T {
            key: HL_DISPLAY,
            value: b"display\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_CONTAINED,
            value: b"contained\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_ONELINE,
            value: b"oneline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_KEEPEND,
            value: b"keepend\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_EXTEND,
            value: b"extend\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_EXCLUDENL,
            value: b"excludenl\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_TRANSP,
            value: b"transparent\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_FOLD,
            value: b"fold\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_CONCEAL,
            value: b"conceal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_CONCEALENDS,
            value: b"concealends\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
        },
    ]);
    namelist2.set([
        keyvalue_T {
            key: HL_SKIPWHITE,
            value: b"skipwhite\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_SKIPNL,
            value: b"skipnl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        },
        keyvalue_T {
            key: HL_SKIPEMPTY,
            value: b"skipempty\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            length: ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        },
    ]);
}
#[used]
#[cfg_attr(target_os = "linux", unsafe(link_section = ".init_array"))]
#[cfg_attr(target_os = "windows", unsafe(link_section = ".CRT$XIB"))]
#[cfg_attr(target_os = "macos", unsafe(link_section = "__DATA,__mod_init_func"))]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [c2rust_run_static_initializers];
