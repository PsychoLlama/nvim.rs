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

// The carve of the transpiled module; see each child's docs.
mod state;
pub use self::state::*;
mod stack;
pub use self::stack::*;
mod sync;
pub(crate) use self::sync::*;
mod attr;
pub use self::attr::*;
mod items;
pub(crate) use self::items::*;
mod endpos;
pub(crate) use self::endpos::*;
mod command;
pub use self::command::*;
mod list;
pub(crate) use self::list::*;
mod keyword;
pub(crate) use self::keyword::*;
mod define;
pub(crate) use self::define::*;
mod cluster;
pub(crate) use self::cluster::*;
mod options;
pub(crate) use self::options::*;
mod query;
pub use self::query::*;
mod syntime;
pub use self::syntime::*;

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
pub const ITEM_START: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ITEM_SKIP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ITEM_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ITEM_MATCHGROUP: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
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
