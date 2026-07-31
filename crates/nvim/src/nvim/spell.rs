use crate::src::nvim::buffer::buf_is_empty;
use crate::src::nvim::change::inserted_bytes;
use crate::src::nvim::cursor::{get_cursor_line_len, get_cursor_line_ptr};
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_later};
use crate::src::nvim::ex_cmds::do_sub_msg;
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_find;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::insexpand::{
    ins_compl_add_infercase, ins_compl_check_keys, ins_compl_interrupted,
};
use crate::src::nvim::main::{IObuff, curbuf, curwin, got_int, p_ic, p_ws, sub_nlines, sub_nsubs};
use crate::src::nvim::mbyte::{mb_strnicmp, mb_toupper, utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memline::{ml_append, ml_delete, ml_replace};
use crate::src::nvim::memory::{xfree, xmalloc, xstrlcpy};
use crate::src::nvim::message::{
    emsg, msg_end, msg_ext_set_kind, msg_putchar, msg_puts, msg_start, semsg,
};
use crate::src::nvim::option::{get_option_value, optval_free, set_option_value_give_err};
use crate::src::nvim::options::{kOptSpell, kOptSpelllang};
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, gettext, memmove, snprintf, strcat, strcmp, strcpy, strlen, strncmp,
};
use crate::src::nvim::search::{FORWARD, do_search};
use crate::src::nvim::strings::vim_snprintf;
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
pub unsafe fn ex_spellinfo(mut _eap: *mut exarg_T) {
    if no_spell_checking(curwin.get()) {
        return;
    }
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    msg_start();
    let mut lpi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lpi < (*(*curwin.get()).w_s).b_langp.ga_len && !got_int.get() {
        let lp: *mut langp_T =
            ((*(*curwin.get()).w_s).b_langp.ga_data as *mut langp_T).offset(lpi as isize);
        msg_puts(b"file: \0".as_ptr() as *const ::core::ffi::c_char);
        msg_puts((*(*lp).lp_slang).sl_fname);
        let p: *const ::core::ffi::c_char = (*(*lp).lp_slang).sl_info;
        if lpi < (*(*curwin.get()).w_s).b_langp.ga_len || !p.is_null() {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        if !p.is_null() {
            msg_puts(p);
            if lpi < (*(*curwin.get()).w_s).b_langp.ga_len - 1 as ::core::ffi::c_int {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
        }
        lpi += 1;
    }
    msg_end();
}
pub const DUMPFLAG_KEEPCASE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DUMPFLAG_COUNT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DUMPFLAG_ICASE: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const DUMPFLAG_ONECAP: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const DUMPFLAG_ALLCAP: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub unsafe fn ex_spelldump(mut eap: *mut exarg_T) {
    if no_spell_checking(curwin.get()) {
        return;
    }
    let mut spl: OptVal = get_option_value(kOptSpelllang, OPT_LOCAL as ::core::ffi::c_int);
    do_cmdline_cmd(b"new\0".as_ptr() as *const ::core::ffi::c_char);
    set_option_value_give_err(
        kOptSpell,
        OptVal {
            type_0: kOptValTypeBoolean,
            data: OptValData { boolean: kTrue },
        },
        OPT_LOCAL as ::core::ffi::c_int,
    );
    set_option_value_give_err(kOptSpelllang, spl, OPT_LOCAL as ::core::ffi::c_int);
    optval_free(spl);
    if !buf_is_empty(curbuf.get()) {
        return;
    }
    spell_dump_compl(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<Direction>(),
        if (*eap).forceit != 0 {
            DUMPFLAG_COUNT
        } else {
            0 as ::core::ffi::c_int
        },
    );
    if (*curbuf.get()).b_ml.ml_line_count > 1 as linenr_T {
        ml_delete((*curbuf.get()).b_ml.ml_line_count);
    }
    redraw_later(curwin.get(), UPD_NOT_VALID);
}
pub unsafe fn spell_dump_compl(
    mut pat: *mut ::core::ffi::c_char,
    mut ic: ::core::ffi::c_int,
    mut dir: *mut Direction,
    mut dumpflags_arg: ::core::ffi::c_int,
) {
    let mut arridx: [idx_T; 254] = [0; 254];
    let mut curi: [::core::ffi::c_int; 254] = [0; 254];
    let mut word: [::core::ffi::c_char; 254] = [0; 254];
    let mut lnum: linenr_T = 0 as linenr_T;
    let mut region_names: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut do_region: bool = true_0 != 0;
    let mut dumpflags: ::core::ffi::c_int = dumpflags_arg;
    if !pat.is_null() {
        if ic != 0 {
            dumpflags |= DUMPFLAG_ICASE;
        } else {
            let mut n: ::core::ffi::c_int =
                captype(pat, ::core::ptr::null::<::core::ffi::c_char>());
            if n == WF_ONECAP as ::core::ffi::c_int {
                dumpflags |= DUMPFLAG_ONECAP;
            } else if n == WF_ALLCAP as ::core::ffi::c_int
                && strlen(pat) as ::core::ffi::c_int > utfc_ptr2len(pat)
            {
                dumpflags |= DUMPFLAG_ALLCAP;
            }
        }
    }
    let mut lpi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lpi < (*(*curwin.get()).w_s).b_langp.ga_len {
        let mut lp: *mut langp_T =
            ((*(*curwin.get()).w_s).b_langp.ga_data as *mut langp_T).offset(lpi as isize);
        let mut p: *mut ::core::ffi::c_char =
            &raw mut (*(*lp).lp_slang).sl_regions as *mut ::core::ffi::c_char;
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if region_names.is_null() {
                region_names = p;
            } else if strcmp(region_names, p) != 0 as ::core::ffi::c_int {
                do_region = false_0 != 0;
                break;
            }
        }
        lpi += 1;
    }
    if do_region as ::core::ffi::c_int != 0 && !region_names.is_null() && pat.is_null() {
        vim_snprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            b"/regions=%s\0".as_ptr() as *const ::core::ffi::c_char,
            region_names,
        );
        let c2rust_fresh12 = lnum;
        lnum = lnum + 1;
        ml_append(
            c2rust_fresh12,
            IObuff.ptr() as *mut ::core::ffi::c_char,
            0 as colnr_T,
            false_0 != 0,
        );
    } else {
        do_region = false_0 != 0;
    }
    let mut lpi_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lpi_0 < (*(*curwin.get()).w_s).b_langp.ga_len {
        let mut lp_0: *mut langp_T =
            ((*(*curwin.get()).w_s).b_langp.ga_data as *mut langp_T).offset(lpi_0 as isize);
        let mut slang: *mut slang_T = (*lp_0).lp_slang;
        if !(*slang).sl_fbyts.is_null() {
            if pat.is_null() {
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    b"# file: %s\0".as_ptr() as *const ::core::ffi::c_char,
                    (*slang).sl_fname,
                );
                let c2rust_fresh13 = lnum;
                lnum = lnum + 1;
                ml_append(
                    c2rust_fresh13,
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    0 as colnr_T,
                    false_0 != 0,
                );
            }
            let mut patlen: ::core::ffi::c_int = 0;
            if !pat.is_null() && (*slang).sl_pbyts.is_null() {
                patlen = strlen(pat) as ::core::ffi::c_int;
            } else {
                patlen = -1 as ::core::ffi::c_int;
            }
            let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while round <= 2 as ::core::ffi::c_int {
                let mut byts: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
                let mut idxs: *mut idx_T = ::core::ptr::null_mut::<idx_T>();
                if round == 1 as ::core::ffi::c_int {
                    dumpflags &= !DUMPFLAG_KEEPCASE;
                    byts = (*slang).sl_fbyts;
                    idxs = (*slang).sl_fidxs;
                } else {
                    dumpflags |= DUMPFLAG_KEEPCASE;
                    byts = (*slang).sl_kbyts;
                    idxs = (*slang).sl_kidxs;
                }
                if !byts.is_null() {
                    let mut depth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    arridx[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int as idx_T;
                    curi[0 as ::core::ffi::c_int as usize] = 1 as ::core::ffi::c_int;
                    while depth >= 0 as ::core::ffi::c_int
                        && !got_int.get()
                        && (pat.is_null() || !ins_compl_interrupted())
                    {
                        if curi[depth as usize]
                            > *byts.offset(arridx[depth as usize] as isize) as ::core::ffi::c_int
                        {
                            depth -= 1;
                            line_breakcheck();
                            ins_compl_check_keys(50 as ::core::ffi::c_int, false_0 != 0);
                        } else {
                            let mut n_0: ::core::ffi::c_int =
                                arridx[depth as usize] as ::core::ffi::c_int + curi[depth as usize];
                            curi[depth as usize] += 1;
                            let mut c: ::core::ffi::c_int =
                                *byts.offset(n_0 as isize) as ::core::ffi::c_int;
                            if c == 0 as ::core::ffi::c_int
                                || depth >= MAXWLEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                            {
                                let mut flags: ::core::ffi::c_int = *idxs.offset(n_0 as isize);
                                if (round == 2 as ::core::ffi::c_int
                                    || flags & WF_KEEPCAP as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int)
                                    && flags & WF_NEEDCOMP as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    && (do_region as ::core::ffi::c_int != 0
                                        || flags & WF_REGION as ::core::ffi::c_int
                                            == 0 as ::core::ffi::c_int
                                        || flags as ::core::ffi::c_uint >> 16 as ::core::ffi::c_int
                                            & (*lp_0).lp_region as ::core::ffi::c_uint
                                            != 0 as ::core::ffi::c_uint)
                                {
                                    word[depth as usize] = NUL as ::core::ffi::c_char;
                                    if !do_region {
                                        flags &= !(WF_REGION as ::core::ffi::c_int);
                                    }
                                    c = (flags as ::core::ffi::c_uint >> 24 as ::core::ffi::c_int)
                                        as ::core::ffi::c_int;
                                    if c == 0 as ::core::ffi::c_int
                                        || curi[depth as usize] == 2 as ::core::ffi::c_int
                                    {
                                        dump_word(
                                            slang,
                                            &raw mut word as *mut ::core::ffi::c_char,
                                            pat,
                                            dir,
                                            dumpflags,
                                            flags,
                                            lnum,
                                        );
                                        if pat.is_null() {
                                            lnum += 1;
                                        }
                                    }
                                    if c != 0 as ::core::ffi::c_int {
                                        lnum = dump_prefixes(
                                            slang,
                                            &raw mut word as *mut ::core::ffi::c_char,
                                            pat,
                                            dir,
                                            dumpflags,
                                            flags,
                                            lnum,
                                        );
                                    }
                                }
                            } else {
                                let c2rust_fresh14 = depth;
                                depth = depth + 1;
                                word[c2rust_fresh14 as usize] = c as ::core::ffi::c_char;
                                arridx[depth as usize] = *idxs.offset(n_0 as isize);
                                curi[depth as usize] = 1 as ::core::ffi::c_int;
                                '_c2rust_label: {
                                    if depth >= 0 as ::core::ffi::c_int {
                                    } else {
                                        __assert_fail(
                                            b"depth >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                                            b"src/nvim/spell.rs\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            3396 as ::core::ffi::c_uint,
                                            b"void spell_dump_compl(char *, int, Direction *, int)\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        );
                                    }
                                };
                                if depth <= patlen
                                    && mb_strnicmp(
                                        &raw mut word as *mut ::core::ffi::c_char,
                                        pat,
                                        depth as size_t,
                                    ) != 0 as ::core::ffi::c_int
                                {
                                    depth -= 1;
                                }
                            }
                        }
                    }
                }
                round += 1;
            }
        }
        lpi_0 += 1;
    }
}
unsafe fn dump_word(
    mut slang: *mut slang_T,
    mut word: *mut ::core::ffi::c_char,
    mut pat: *mut ::core::ffi::c_char,
    mut dir: *mut Direction,
    mut dumpflags: ::core::ffi::c_int,
    mut wordflags: ::core::ffi::c_int,
    mut lnum: linenr_T,
) {
    let mut keepcap: bool = false_0 != 0;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cword: [::core::ffi::c_char; 254] = [0; 254];
    let mut badword: [::core::ffi::c_char; 264] = [0; 264];
    let mut flags: ::core::ffi::c_int = wordflags;
    if dumpflags & DUMPFLAG_ONECAP != 0 {
        flags |= WF_ONECAP as ::core::ffi::c_int;
    }
    if dumpflags & DUMPFLAG_ALLCAP != 0 {
        flags |= WF_ALLCAP as ::core::ffi::c_int;
    }
    if dumpflags & DUMPFLAG_KEEPCASE == 0 as ::core::ffi::c_int
        && flags & WF_CAPMASK as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        make_case_word(word, &raw mut cword as *mut ::core::ffi::c_char, flags);
        p = &raw mut cword as *mut ::core::ffi::c_char;
    } else {
        p = word;
        if dumpflags & DUMPFLAG_KEEPCASE != 0
            && (captype(word, ::core::ptr::null::<::core::ffi::c_char>())
                & WF_KEEPCAP as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                || flags & WF_FIXCAP as ::core::ffi::c_int != 0 as ::core::ffi::c_int)
        {
            keepcap = true_0 != 0;
        }
    }
    let mut tw: *mut ::core::ffi::c_char = p;
    if pat.is_null() {
        if flags
            & (WF_BANNED as ::core::ffi::c_int
                | WF_RARE as ::core::ffi::c_int
                | WF_REGION as ::core::ffi::c_int)
            != 0
            || keepcap as ::core::ffi::c_int != 0
        {
            strcpy(&raw mut badword as *mut ::core::ffi::c_char, p);
            strcat(
                &raw mut badword as *mut ::core::ffi::c_char,
                b"/\0".as_ptr() as *const ::core::ffi::c_char,
            );
            if keepcap {
                strcat(
                    &raw mut badword as *mut ::core::ffi::c_char,
                    b"=\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            if flags & WF_BANNED as ::core::ffi::c_int != 0 {
                strcat(
                    &raw mut badword as *mut ::core::ffi::c_char,
                    b"!\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else if flags & WF_RARE as ::core::ffi::c_int != 0 {
                strcat(
                    &raw mut badword as *mut ::core::ffi::c_char,
                    b"?\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            if flags & WF_REGION as ::core::ffi::c_int != 0 {
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < 7 as ::core::ffi::c_int {
                    if flags & (0x10000 as ::core::ffi::c_int) << i != 0 {
                        let badword_len: size_t =
                            strlen(&raw mut badword as *mut ::core::ffi::c_char);
                        snprintf(
                            (&raw mut badword as *mut ::core::ffi::c_char)
                                .offset(badword_len as isize),
                            ::core::mem::size_of::<[::core::ffi::c_char; 264]>()
                                .wrapping_sub(badword_len),
                            b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                            i + 1 as ::core::ffi::c_int,
                        );
                    }
                    i += 1;
                }
            }
            p = &raw mut badword as *mut ::core::ffi::c_char;
        }
        if dumpflags & DUMPFLAG_COUNT != 0 {
            let mut hi: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
            hi = hash_find(&raw mut (*slang).sl_wordcount, tw);
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    b"%s\t%d\0".as_ptr() as *const ::core::ffi::c_char,
                    tw,
                    (*((*hi).hi_key.offset(-(WC_KEY_OFF as isize)) as *mut wordcount_T)).wc_count
                        as ::core::ffi::c_int,
                );
                p = IObuff.ptr() as *mut ::core::ffi::c_char;
            }
        }
        ml_append(lnum, p, 0 as colnr_T, false_0 != 0);
    } else if (if dumpflags & DUMPFLAG_ICASE != 0 {
        (mb_strnicmp(p, pat, strlen(pat)) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
    } else {
        (strncmp(p, pat, strlen(pat)) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
    }) != 0
        && ins_compl_add_infercase(
            p,
            strlen(p) as ::core::ffi::c_int,
            p_ic.get() != 0,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            *dir,
            false_0 != 0,
            0 as ::core::ffi::c_int,
        ) == OK
    {
        *dir = FORWARD;
    }
}
unsafe fn dump_prefixes(
    mut slang: *mut slang_T,
    mut word: *mut ::core::ffi::c_char,
    mut pat: *mut ::core::ffi::c_char,
    mut dir: *mut Direction,
    mut dumpflags: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut startlnum: linenr_T,
) -> linenr_T {
    let mut arridx: [idx_T; 254] = [0; 254];
    let mut curi: [::core::ffi::c_int; 254] = [0; 254];
    let mut prefix: [::core::ffi::c_char; 254] = [0; 254];
    let mut word_up: [::core::ffi::c_char; 254] = [0; 254];
    let mut has_word_up: bool = false_0 != 0;
    let mut lnum: linenr_T = startlnum;
    let mut c: ::core::ffi::c_int = utf_ptr2char(word);
    if (if c >= 128 as ::core::ffi::c_int {
        mb_toupper(c)
    } else {
        (*spelltab.ptr()).st_upper[c as usize] as ::core::ffi::c_int
    }) != c
    {
        onecap_copy(
            word,
            &raw mut word_up as *mut ::core::ffi::c_char,
            true_0 != 0,
        );
        has_word_up = true_0 != 0;
    }
    let mut byts: *mut uint8_t = (*slang).sl_pbyts;
    let mut idxs: *mut idx_T = (*slang).sl_pidxs;
    if !byts.is_null() {
        let mut depth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        arridx[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int as idx_T;
        curi[0 as ::core::ffi::c_int as usize] = 1 as ::core::ffi::c_int;
        while depth >= 0 as ::core::ffi::c_int && !got_int.get() {
            let mut n: ::core::ffi::c_int = arridx[depth as usize] as ::core::ffi::c_int;
            let mut len: ::core::ffi::c_int = *byts.offset(n as isize) as ::core::ffi::c_int;
            if curi[depth as usize] > len {
                depth -= 1;
                line_breakcheck();
            } else {
                n += curi[depth as usize];
                curi[depth as usize] += 1;
                c = *byts.offset(n as isize) as ::core::ffi::c_int;
                if c == 0 as ::core::ffi::c_int {
                    let mut i: ::core::ffi::c_int = 0;
                    i = 1 as ::core::ffi::c_int;
                    while i < len {
                        if *byts.offset((n + i) as isize) as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_int
                        {
                            break;
                        }
                        i += 1;
                    }
                    curi[depth as usize] += i - 1 as ::core::ffi::c_int;
                    c = valid_word_prefix(i, n, flags, word, slang, false_0 != 0);
                    if c != 0 as ::core::ffi::c_int {
                        xstrlcpy(
                            (&raw mut prefix as *mut ::core::ffi::c_char).offset(depth as isize),
                            word,
                            (MAXWLEN as ::core::ffi::c_int - depth) as size_t,
                        );
                        dump_word(
                            slang,
                            &raw mut prefix as *mut ::core::ffi::c_char,
                            pat,
                            dir,
                            dumpflags,
                            if c & WF_RAREPFX as ::core::ffi::c_int != 0 {
                                flags | WF_RARE as ::core::ffi::c_int
                            } else {
                                flags
                            },
                            lnum,
                        );
                        if lnum != 0 as linenr_T {
                            lnum += 1;
                        }
                    }
                    if has_word_up {
                        c = valid_word_prefix(
                            i,
                            n,
                            flags,
                            &raw mut word_up as *mut ::core::ffi::c_char,
                            slang,
                            true_0 != 0,
                        );
                        if c != 0 as ::core::ffi::c_int {
                            xstrlcpy(
                                (&raw mut prefix as *mut ::core::ffi::c_char)
                                    .offset(depth as isize),
                                &raw mut word_up as *mut ::core::ffi::c_char,
                                (MAXWLEN as ::core::ffi::c_int - depth) as size_t,
                            );
                            dump_word(
                                slang,
                                &raw mut prefix as *mut ::core::ffi::c_char,
                                pat,
                                dir,
                                dumpflags,
                                if c & WF_RAREPFX as ::core::ffi::c_int != 0 {
                                    flags | WF_RARE as ::core::ffi::c_int
                                } else {
                                    flags
                                },
                                lnum,
                            );
                            if lnum != 0 as linenr_T {
                                lnum += 1;
                            }
                        }
                    }
                } else {
                    let c2rust_fresh15 = depth;
                    depth = depth + 1;
                    prefix[c2rust_fresh15 as usize] = c as ::core::ffi::c_char;
                    arridx[depth as usize] = *idxs.offset(n as isize);
                    curi[depth as usize] = 1 as ::core::ffi::c_int;
                }
            }
        }
    }
    return lnum;
}
pub const SPL_FNAME_TMPL: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"%s.%s.spl\0") };
pub const WC_KEY_OFF: ::core::ffi::c_ulong = 2 as ::core::ffi::c_ulong;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
