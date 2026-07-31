use crate::src::nvim::change::inserted_bytes;
use crate::src::nvim::cursor::{get_cursor_line_len, get_cursor_line_ptr};
use crate::src::nvim::ex_cmds::do_sub_msg;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{curwin, got_int, p_ws, sub_nlines, sub_nsubs};
use crate::src::nvim::memline::ml_replace;
use crate::src::nvim::memory::{xfree, xmalloc};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::os::libc::{gettext, memmove, snprintf, strcat, strcpy, strlen, strncmp};
use crate::src::nvim::search::do_search;
pub use crate::src::nvim::types::{
    __time_t, AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, CMD_index, Callback,
    Callback_data as C2Rust_Unnamed_4, CallbackType, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorPriorityInternal, DecorRange,
    DecorRange_data as C2Rust_Unnamed_17, DecorRange_data_ui as C2Rust_Unnamed_18, DecorRangeKind,
    DecorRangeSlot, DecorSignHighlight, DecorState, DecorState_ranges_i as C2Rust_Unnamed_19,
    DecorState_slots as C2Rust_Unnamed_20, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_1,
    Direction, DoInRuntimepathCB, ExtmarkUndoObject, FileComparison, FileID, FloatAnchor,
    FloatRelative, GridView, Intersection, LineGetter, LuaRef, MTKey, MTNode, MTPos,
    Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MapHash,
    MarkTree, MarkTreeIter, MarkTreeIter_s as C2Rust_Unnamed_13, MotionType, OptIndex, OptInt,
    OptVal, OptValData, OptValType, QUEUE, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t,
    Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_11, String_0, Terminal, Timestamp, TriState,
    VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo,
    WinSplit, WinStyle, Window, alist_T, auto_event, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T,
    bufref_T, bufstate_T, chunksize_T, cmd_addr_T, cmdidx_T, colnr_T, cstack_T,
    cstack_T_cs_pend as C2Rust_Unnamed_14, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T,
    eslist_T, eslist_elem, event_T, exarg, exarg_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_2, file_buffer_b_wininfo as C2Rust_Unnamed_10,
    file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, file_comparison, float_T, fmark_T, fmarkv_T,
    frame_S, frame_T, fromto_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T,
    garray_T, handle_T, hash_T, hashitem_T, hashtab_T, hlf_T, idx_T, infoptr_T, int16_t, int32_t,
    int64_t, intptr_t, langp_T, lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S,
    listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock, mapblock_T, match_T, matchitem,
    matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s, mtnode_s, oparg_T, partial_S,
    partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue,
    reg_extmatch_T, regmatch_T, regmmatch_T, regprog, regprog_T, salfirst_T, salitem_T, sattr_T,
    schar_T, scid_T, sctx_T, searchit_arg_T, size_t, slang_S, slang_T, smt_T, spelltab_T,
    syn_state, syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T, synblock_T, synstate_T,
    tabpage_S, tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry,
    u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_7,
    u_header_uh_alt_prev as C2Rust_Unnamed_6, u_header_uh_next as C2Rust_Unnamed_9,
    u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T, uint8_t, uint16_t, uint32_t, uint64_t,
    undo_object, varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T,
    wline_T, wordcount_T, xfmark_T,
};
use crate::src::nvim::undo::u_save_cursor;

mod chartab;
mod check;
mod dump;
mod lang;
mod lookup;
mod navigate;
mod slang;
mod soundfold;

pub use chartab::{
    allcap_copy, byte_in_str, captype, clear_spell_chartab, init_spell_chartab, make_case_word,
    nofold_len, onecap_copy, spell_casefold, spell_iswordp, spell_iswordp_nmw,
};
pub use check::{
    check_need_cap, expand_spelling, no_spell_checking, spell_check, spell_check_window,
    spell_expand_check_cap, spell_to_word_end, spell_valid_case, spell_word_start,
};
pub use dump::{ex_spelldump, ex_spellinfo, spell_dump_compl};
pub use lang::{
    compile_cap_prog, did_set_spell_option, parse_spelllang, spell_delete_wordlist, spell_enc,
    spell_free_all, spell_reload, valid_spellfile, valid_spelllang,
};
pub use lookup::{can_compound, match_checkcompoundpattern, match_compoundrule, valid_word_prefix};
pub use navigate::{spell_cat_line, spell_move_to};
use slang::count_syllables;
pub use slang::{
    close_spellbuf, count_common_word, init_syl_tab, open_spellbuf, slang_alloc, slang_clear,
    slang_clear_sug, slang_free,
};
pub use soundfold::{eval_soundfold, spell_soundfold};
unsafe extern "C" {
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec_prog(
        prog: *mut *mut regprog_T,
        ignore_case: bool,
        line: *const ::core::ffi::c_char,
        col: colnr_T,
    ) -> bool;
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_12 = 2147483647;
pub const HLF_COUNT: hlf_T = 76;
pub const HLF_SPL: hlf_T = 40;
pub const HLF_SPR: hlf_T = 39;
pub const HLF_SPC: hlf_T = 38;
pub const HLF_SPB: hlf_T = 37;
pub const kOptValTypeBoolean: OptValType = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const SHM_SEARCH: C2Rust_Unnamed_16 = 115;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const MB_MAXBYTES: C2Rust_Unnamed_22 = 21;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const OPT_LOCAL: C2Rust_Unnamed_23 = 2;
pub const kEqualFiles: file_comparison = 1;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const DIP_ALL: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const SEARCH_KEEP: C2Rust_Unnamed_25 = 1024;
pub const MAXWLEN: usize = 254;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const WF_CAPMASK: C2Rust_Unnamed_27 = 198;
pub const WF_KEEPCAP: C2Rust_Unnamed_27 = 128;
pub const WF_FIXCAP: C2Rust_Unnamed_27 = 64;
pub const WF_BANNED: C2Rust_Unnamed_27 = 16;
pub const WF_RARE: C2Rust_Unnamed_27 = 8;
pub const WF_ALLCAP: C2Rust_Unnamed_27 = 4;
pub const WF_ONECAP: C2Rust_Unnamed_27 = 2;
pub const WF_REGION: C2Rust_Unnamed_27 = 1;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const WF_NOCOMPAFT: C2Rust_Unnamed_28 = 8192;
pub const WF_NOCOMPBEF: C2Rust_Unnamed_28 = 4096;
pub const WF_COMPROOT: C2Rust_Unnamed_28 = 2048;
pub const WF_NEEDCOMP: C2Rust_Unnamed_28 = 512;
pub const WF_HAS_AFF: C2Rust_Unnamed_28 = 256;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const WF_PFX_NC: C2Rust_Unnamed_29 = 33554432;
pub const WF_RAREPFX: C2Rust_Unnamed_29 = 16777216;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_int;
pub const SP_FORMERROR: C2Rust_Unnamed_30 = -2;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const REGION_ALL: C2Rust_Unnamed_31 = 255;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const MAXWORDCOUNT: C2Rust_Unnamed_32 = 65535;
pub const SMT_RARE: smt_T = 2;
pub const SMT_BAD: smt_T = 1;
pub const SMT_ALL: smt_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct matchinf_T {
    pub mi_lp: *mut langp_T,
    pub mi_word: *mut ::core::ffi::c_char,
    pub mi_end: *mut ::core::ffi::c_char,
    pub mi_fend: *mut ::core::ffi::c_char,
    pub mi_cend: *mut ::core::ffi::c_char,
    pub mi_fword: [::core::ffi::c_char; 255],
    pub mi_fwordlen: ::core::ffi::c_int,
    pub mi_prefarridx: ::core::ffi::c_int,
    pub mi_prefcnt: ::core::ffi::c_int,
    pub mi_prefixlen: ::core::ffi::c_int,
    pub mi_cprefixlen: ::core::ffi::c_int,
    pub mi_compoff: ::core::ffi::c_int,
    pub mi_compflags: [uint8_t; 254],
    pub mi_complen: ::core::ffi::c_int,
    pub mi_compextra: ::core::ffi::c_int,
    pub mi_result: ::core::ffi::c_int,
    pub mi_capflags: ::core::ffi::c_int,
    pub mi_win: *mut win_T,
    pub mi_result2: ::core::ffi::c_int,
    pub mi_end2: *mut ::core::ffi::c_char,
}
pub const SP_RARE: C2Rust_Unnamed_33 = 0;
pub const SP_OK: C2Rust_Unnamed_33 = 1;
pub const SP_BANNED: C2Rust_Unnamed_33 = -1;
pub const SP_BAD: C2Rust_Unnamed_33 = 3;
pub const FIND_COMPOUND: C2Rust_Unnamed_34 = 3;
pub const SP_LOCAL: C2Rust_Unnamed_33 = 2;
pub const FIND_KEEPCOMPOUND: C2Rust_Unnamed_34 = 4;
pub const FIND_KEEPWORD: C2Rust_Unnamed_34 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syl_item_T {
    pub sy_chars: [::core::ffi::c_char; 30],
    pub sy_len: ::core::ffi::c_int,
}
pub const FIND_PREFIX: C2Rust_Unnamed_34 = 2;
pub const FIND_FOLDWORD: C2Rust_Unnamed_34 = 0;
pub const CHAR_OTHER: C2Rust_Unnamed_35 = 0;
pub const CHAR_UPPER: C2Rust_Unnamed_35 = 1;
pub const CHAR_DIGIT: C2Rust_Unnamed_35 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct spelload_T {
    pub sl_lang: [::core::ffi::c_char; 255],
    pub sl_slang: *mut slang_T,
    pub sl_nobreak: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_33 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub static first_lang: GlobalCell<*mut slang_T> =
    GlobalCell::new(::core::ptr::null_mut::<slang_T>());
pub static int_wordlist: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub const SY_MAXLEN: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
pub static spelltab: GlobalCell<spelltab_T> = GlobalCell::new(spelltab_T {
    st_isw: [false; 256],
    st_isu: [false; 256],
    st_fold: [0; 256],
    st_upper: [0; 256],
});
pub static did_set_spelltab: GlobalCell<bool> = GlobalCell::new(false);
pub static e_format: GlobalCell<*mut ::core::ffi::c_char> = GlobalCell::new(
    b"E759: Format error in spell file\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
);
pub static repl_from: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static repl_to: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub unsafe fn ex_spellrepall(mut _eap: *mut exarg_T) {
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    let mut save_ws: bool = p_ws.get() != 0;
    let mut prev_lnum: linenr_T = 0 as linenr_T;
    if (*repl_from.ptr()).is_null() || (*repl_to.ptr()).is_null() {
        emsg(gettext(
            b"E752: No previous spell replacement\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return;
    }
    let repl_from_len: size_t = strlen(repl_from.get());
    let repl_to_len: size_t = strlen(repl_to.get());
    let addlen: int64_t = repl_to_len as int64_t - repl_from_len as int64_t;
    let frompatsize: size_t = repl_from_len.wrapping_add(7 as size_t);
    let mut frompat: *mut ::core::ffi::c_char = xmalloc(frompatsize) as *mut ::core::ffi::c_char;
    let mut frompatlen: size_t = snprintf(
        frompat,
        frompatsize,
        b"\\V\\<%s\\>\0".as_ptr() as *const ::core::ffi::c_char,
        repl_from.get(),
    ) as size_t;
    p_ws.set(false_0);
    sub_nsubs.set(0 as ::core::ffi::c_int);
    sub_nlines.set(0 as ::core::ffi::c_int as linenr_T);
    (*curwin.get()).w_cursor.lnum = 0 as ::core::ffi::c_int as linenr_T;
    while !got_int.get() {
        if do_search(
            ::core::ptr::null_mut::<oparg_T>(),
            '/' as ::core::ffi::c_int,
            '/' as ::core::ffi::c_int,
            frompat,
            frompatlen,
            1 as ::core::ffi::c_int,
            SEARCH_KEEP as ::core::ffi::c_int,
            ::core::ptr::null_mut::<searchit_arg_T>(),
        ) == 0 as ::core::ffi::c_int
            || u_save_cursor() == FAIL
        {
            break;
        }
        let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        if addlen <= 0 as int64_t
            || strncmp(
                line.offset((*curwin.get()).w_cursor.col as isize),
                repl_to.get(),
                repl_to_len,
            ) != 0 as ::core::ffi::c_int
        {
            let mut p: *mut ::core::ffi::c_char = xmalloc(
                ((get_cursor_line_len() as int64_t + addlen) as size_t).wrapping_add(1 as size_t),
            ) as *mut ::core::ffi::c_char;
            memmove(
                p as *mut ::core::ffi::c_void,
                line as *const ::core::ffi::c_void,
                (*curwin.get()).w_cursor.col as size_t,
            );
            strcpy(
                p.offset((*curwin.get()).w_cursor.col as isize),
                repl_to.get(),
            );
            strcat(
                p,
                line.offset((*curwin.get()).w_cursor.col as isize)
                    .offset(repl_from_len as isize),
            );
            ml_replace((*curwin.get()).w_cursor.lnum, p, false_0 != 0);
            inserted_bytes(
                (*curwin.get()).w_cursor.lnum,
                (*curwin.get()).w_cursor.col,
                repl_from_len as ::core::ffi::c_int,
                repl_to_len as ::core::ffi::c_int,
            );
            if (*curwin.get()).w_cursor.lnum != prev_lnum {
                (*sub_nlines.ptr()) += 1;
                prev_lnum = (*curwin.get()).w_cursor.lnum;
            }
            (*sub_nsubs.ptr()) += 1;
        }
        (*curwin.get()).w_cursor.col += repl_to_len as colnr_T;
    }
    p_ws.set(save_ws as ::core::ffi::c_int);
    (*curwin.get()).w_cursor = pos;
    xfree(frompat as *mut ::core::ffi::c_void);
    if sub_nsubs.get() == 0 as ::core::ffi::c_int {
        semsg(
            gettext(b"E753: Not found: %s\0".as_ptr() as *const ::core::ffi::c_char),
            repl_from.get(),
        );
    } else {
        do_sub_msg(false_0 != 0);
    };
}
pub const DUMPFLAG_KEEPCASE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DUMPFLAG_COUNT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DUMPFLAG_ICASE: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const DUMPFLAG_ONECAP: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const DUMPFLAG_ALLCAP: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const SPL_FNAME_TMPL: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"%s.%s.spl\0") };
pub const WC_KEY_OFF: ::core::ffi::c_ulong = 2 as ::core::ffi::c_ulong;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
