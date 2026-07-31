use crate::src::nvim::autocmd::{EVENT_SPELLFILEMISSING, apply_autocmds};
use crate::src::nvim::buffer::{buf_is_empty, bufref_valid, set_bufref};
use crate::src::nvim::change::inserted_bytes;
use crate::src::nvim::charset::vim_is_fname_char;
use crate::src::nvim::cursor::{get_cursor_line_len, get_cursor_line_ptr};
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_later};
use crate::src::nvim::ex_cmds::do_sub_msg;
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::garray::{ga_append_via_ptr, ga_clear, ga_clear_strings, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::hashtab::{
    hash_add_item, hash_clear_all, hash_find, hash_hash, hash_init, hash_lookup,
};
use crate::src::nvim::insexpand::{
    ins_compl_add_infercase, ins_compl_check_keys, ins_compl_interrupted,
};
use crate::src::nvim::log::{LOGLVL_ERR, logmsg};
use crate::src::nvim::main::{
    IObuff, curbuf, curtab, curwin, e_invarg, firstbuf, firstwin, got_int, p_enc, p_ic, p_ws,
    starting, sub_nlines, sub_nsubs,
};
use crate::src::nvim::mbyte::{mb_strnicmp, mb_toupper, utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memline::{
    ml_append, ml_close, ml_delete, ml_open, ml_open_file, ml_replace,
};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xmemcpyz, xmemdupz, xstrdup, xstrlcpy};
use crate::src::nvim::message::{
    emsg, msg_end, msg_ext_set_kind, msg_putchar, msg_puts, msg_start, semsg, smsg,
};
use crate::src::nvim::option::{
    copy_option_part, get_option_value, optval_free, set_option_value_give_err, valid_name,
};
use crate::src::nvim::options::{kOptSpell, kOptSpelllang};
use crate::src::nvim::os::fs::os_remove;
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, gettext, memcpy, memmove, memset, snprintf, strcasecmp, strcat, strcmp, strcpy,
    strlen, strncmp,
};
use crate::src::nvim::path::{path_fnamecmp, path_full_compare, path_tail};
use crate::src::nvim::runtime::do_in_runtimepath;
use crate::src::nvim::search::{FORWARD, do_search};
use crate::src::nvim::spellfile::spell_load_file;
use crate::src::nvim::strings::{concat_str, vim_snprintf, vim_strchr, xstrnsave};
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
use crate::src::nvim::window::win_valid_any_tab;

mod chartab;
mod check;
mod lookup;
mod navigate;
mod soundfold;

pub use chartab::{
    allcap_copy, byte_in_str, captype, clear_spell_chartab, init_spell_chartab, make_case_word,
    nofold_len, onecap_copy, spell_casefold, spell_iswordp, spell_iswordp_nmw,
};
pub use check::{
    check_need_cap, expand_spelling, no_spell_checking, spell_check, spell_check_window,
    spell_expand_check_cap, spell_to_word_end, spell_valid_case, spell_word_start,
};
pub use lookup::{can_compound, match_checkcompoundpattern, match_compoundrule, valid_word_prefix};
pub use navigate::{spell_cat_line, spell_move_to};
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
unsafe fn spell_load_lang(mut lang: *mut ::core::ffi::c_char) {
    let mut fname_enc: [::core::ffi::c_char; 85] = [0; 85];
    let mut r: ::core::ffi::c_int = 0;
    let mut sl: spelload_T = spelload_T {
        sl_lang: [0; 255],
        sl_slang: ::core::ptr::null_mut::<slang_T>(),
        sl_nobreak: 0,
    };
    strcpy(&raw mut sl.sl_lang as *mut ::core::ffi::c_char, lang);
    sl.sl_slang = ::core::ptr::null_mut::<slang_T>();
    sl.sl_nobreak = false_0;
    (*curbuf.get()).b_locked += 1;
    let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while round <= 2 as ::core::ffi::c_int {
        vim_snprintf(
            &raw mut fname_enc as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 85]>().wrapping_sub(5 as size_t),
            b"spell/%s.%s.spl\0".as_ptr() as *const ::core::ffi::c_char,
            lang,
            spell_enc(),
        );
        r = do_in_runtimepath(
            &raw mut fname_enc as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            Some(
                spell_load_cb
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            &raw mut sl as *mut ::core::ffi::c_void,
        );
        if !(r == FAIL
            && *(&raw mut sl.sl_lang as *mut ::core::ffi::c_char) as ::core::ffi::c_int != NUL)
        {
            break;
        }
        vim_snprintf(
            &raw mut fname_enc as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 85]>().wrapping_sub(5 as size_t),
            b"spell/%s.ascii.spl\0".as_ptr() as *const ::core::ffi::c_char,
            lang,
        );
        r = do_in_runtimepath(
            &raw mut fname_enc as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            Some(
                spell_load_cb
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            &raw mut sl as *mut ::core::ffi::c_void,
        );
        if !(r == FAIL
            && *(&raw mut sl.sl_lang as *mut ::core::ffi::c_char) as ::core::ffi::c_int != NUL
            && round == 1 as ::core::ffi::c_int
            && apply_autocmds(
                EVENT_SPELLFILEMISSING,
                lang,
                (*curbuf.get()).b_fname,
                false_0 != 0,
                curbuf.get(),
            ) as ::core::ffi::c_int
                != 0)
        {
            break;
        }
        round += 1;
    }
    if r == FAIL {
        if starting.get() != 0 {
            let mut autocmd_buf: [::core::ffi::c_char; 512] = [0 as ::core::ffi::c_char; 512];
            snprintf(
                &raw mut autocmd_buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 512]>(),
                b"autocmd VimEnter * call v:lua.require'nvim.spellfile'.get('%s')|set spell\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                lang,
            );
            do_cmdline_cmd(&raw mut autocmd_buf as *mut ::core::ffi::c_char);
        } else {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(
                    b"Warning: Cannot find word list \"%s.%s.spl\" or \"%s.ascii.spl\"\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                lang,
                spell_enc(),
                lang,
            );
        }
    } else if !sl.sl_slang.is_null() {
        strcpy(
            (&raw mut fname_enc as *mut ::core::ffi::c_char)
                .offset(strlen(&raw mut fname_enc as *mut ::core::ffi::c_char) as isize)
                .offset(-(3 as ::core::ffi::c_int as isize)),
            b"add.spl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        do_in_runtimepath(
            &raw mut fname_enc as *mut ::core::ffi::c_char,
            DIP_ALL as ::core::ffi::c_int,
            Some(
                spell_load_cb
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            &raw mut sl as *mut ::core::ffi::c_void,
        );
    }
    (*curbuf.get()).b_locked -= 1;
}
pub unsafe fn spell_enc() -> *mut ::core::ffi::c_char {
    if strlen(p_enc.get()) < 60 as size_t
        && strcmp(
            p_enc.get(),
            b"iso-8859-15\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0 as ::core::ffi::c_int
    {
        return p_enc.get();
    }
    return b"latin1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
}
unsafe fn int_wordlist_spl(mut fname: *mut ::core::ffi::c_char) {
    vim_snprintf(
        fname,
        MAXPATHL as size_t,
        SPL_FNAME_TMPL.as_ptr(),
        int_wordlist.get(),
        spell_enc(),
    );
}
pub unsafe fn slang_alloc(mut lang: *mut ::core::ffi::c_char) -> *mut slang_T {
    let mut lp: *mut slang_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<slang_T>()) as *mut slang_T;
    if !lang.is_null() {
        (*lp).sl_name = xstrdup(lang);
    }
    ga_init(
        &raw mut (*lp).sl_rep,
        ::core::mem::size_of::<fromto_T>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    ga_init(
        &raw mut (*lp).sl_repsal,
        ::core::mem::size_of::<fromto_T>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    (*lp).sl_compmax = MAXWLEN as ::core::ffi::c_int;
    (*lp).sl_compsylmax = MAXWLEN as ::core::ffi::c_int;
    hash_init(&raw mut (*lp).sl_wordcount);
    return lp;
}
pub unsafe fn slang_free(mut lp: *mut slang_T) {
    xfree((*lp).sl_name as *mut ::core::ffi::c_void);
    xfree((*lp).sl_fname as *mut ::core::ffi::c_void);
    slang_clear(lp);
    xfree(lp as *mut ::core::ffi::c_void);
}
unsafe fn free_salitem(mut smp: *mut salitem_T) {
    xfree((*smp).sm_lead as *mut ::core::ffi::c_void);
    xfree((*smp).sm_to as *mut ::core::ffi::c_void);
    xfree((*smp).sm_lead_w as *mut ::core::ffi::c_void);
    xfree((*smp).sm_oneof_w as *mut ::core::ffi::c_void);
    xfree((*smp).sm_to_w as *mut ::core::ffi::c_void);
}
unsafe fn free_fromto(mut ftp: *mut fromto_T) {
    xfree((*ftp).ft_from as *mut ::core::ffi::c_void);
    xfree((*ftp).ft_to as *mut ::core::ffi::c_void);
}
pub unsafe fn slang_clear(mut lp: *mut slang_T) {
    let mut gap: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_fbyts as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_kbyts as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL;
    let _ = *ptr__0;
    let mut ptr__1: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_pbyts as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__1);
    *ptr__1 = NULL;
    let _ = *ptr__1;
    let mut ptr__2: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_fidxs as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__2);
    *ptr__2 = NULL;
    let _ = *ptr__2;
    let mut ptr__3: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_kidxs as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__3);
    *ptr__3 = NULL;
    let _ = *ptr__3;
    let mut ptr__4: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_pidxs as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__4);
    *ptr__4 = NULL;
    let _ = *ptr__4;
    let mut _gap: *mut garray_T = &raw mut (*lp).sl_rep;
    if !(*_gap).ga_data.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*_gap).ga_len {
            let mut _item: *mut fromto_T = ((*_gap).ga_data as *mut fromto_T).offset(i as isize);
            free_fromto(_item);
            i += 1;
        }
    }
    ga_clear(_gap);
    let mut _gap_0: *mut garray_T = &raw mut (*lp).sl_repsal;
    if !(*_gap_0).ga_data.is_null() {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*_gap_0).ga_len {
            let mut _item_0: *mut fromto_T =
                ((*_gap_0).ga_data as *mut fromto_T).offset(i_0 as isize);
            free_fromto(_item_0);
            i_0 += 1;
        }
    }
    ga_clear(_gap_0);
    gap = &raw mut (*lp).sl_sal;
    if (*lp).sl_sofo {
        let mut _gap_1: *mut garray_T = gap;
        if !(*_gap_1).ga_data.is_null() {
            let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_1 < (*_gap_1).ga_len {
                let mut _item_1: *mut *mut ::core::ffi::c_void =
                    ((*_gap_1).ga_data as *mut *mut ::core::ffi::c_void).offset(i_1 as isize);
                xfree(*_item_1);
                i_1 += 1;
            }
        }
        ga_clear(_gap_1);
    } else {
        let mut _gap_2: *mut garray_T = gap;
        if !(*_gap_2).ga_data.is_null() {
            let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_2 < (*_gap_2).ga_len {
                let mut _item_2: *mut salitem_T =
                    ((*_gap_2).ga_data as *mut salitem_T).offset(i_2 as isize);
                free_salitem(_item_2);
                i_2 += 1;
            }
        }
        ga_clear(_gap_2);
    }
    let mut i_3: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_3 < (*lp).sl_prefixcnt {
        vim_regfree(*(*lp).sl_prefprog.offset(i_3 as isize));
        i_3 += 1;
    }
    (*lp).sl_prefixcnt = 0 as ::core::ffi::c_int;
    let mut ptr__5: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_prefprog as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__5);
    *ptr__5 = NULL;
    let _ = *ptr__5;
    let mut ptr__6: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_info as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__6);
    *ptr__6 = NULL;
    let _ = *ptr__6;
    let mut ptr__7: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_midword as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__7);
    *ptr__7 = NULL;
    let _ = *ptr__7;
    vim_regfree((*lp).sl_compprog);
    (*lp).sl_compprog = ::core::ptr::null_mut::<regprog_T>();
    let mut ptr__8: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_comprules as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__8);
    *ptr__8 = NULL;
    let _ = *ptr__8;
    let mut ptr__9: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_compstartflags as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__9);
    *ptr__9 = NULL;
    let _ = *ptr__9;
    let mut ptr__10: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_compallflags as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__10);
    *ptr__10 = NULL;
    let _ = *ptr__10;
    let mut ptr__11: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_syllable as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__11);
    *ptr__11 = NULL;
    let _ = *ptr__11;
    ga_clear(&raw mut (*lp).sl_syl_items);
    ga_clear_strings(&raw mut (*lp).sl_comppat);
    hash_clear_all(
        &raw mut (*lp).sl_wordcount,
        WC_KEY_OFF as ::core::ffi::c_uint,
    );
    hash_init(&raw mut (*lp).sl_wordcount);
    hash_clear_all(&raw mut (*lp).sl_map_hash, 0 as ::core::ffi::c_uint);
    slang_clear_sug(lp);
    (*lp).sl_compmax = MAXWLEN as ::core::ffi::c_int;
    (*lp).sl_compminlen = 0 as ::core::ffi::c_int;
    (*lp).sl_compsylmax = MAXWLEN as ::core::ffi::c_int;
    (*lp).sl_regions[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
}
pub unsafe fn slang_clear_sug(mut lp: *mut slang_T) {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_sbyts as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_sidxs as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL;
    let _ = *ptr__0;
    close_spellbuf((*lp).sl_sugbuf);
    (*lp).sl_sugbuf = ::core::ptr::null_mut::<buf_T>();
    (*lp).sl_sugloaded = false_0 != 0;
    (*lp).sl_sugtime = 0 as time_t;
}
unsafe extern "C" fn spell_load_cb(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut cookie: *mut ::core::ffi::c_void,
) -> bool {
    let mut slp: *mut spelload_T = cookie as *mut spelload_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_fnames {
        let mut slang: *mut slang_T = spell_load_file(
            *fnames.offset(i as isize),
            &raw mut (*slp).sl_lang as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<slang_T>(),
            false_0 != 0,
        );
        if !slang.is_null() {
            if (*slp).sl_nobreak != 0 && (*slang).sl_add as ::core::ffi::c_int != 0 {
                (*slang).sl_nobreak = true_0 != 0;
            } else if (*slang).sl_nobreak {
                (*slp).sl_nobreak = true_0;
            }
            (*slp).sl_slang = slang;
            if !all {
                break;
            }
        }
        i += 1;
    }
    return num_fnames > 0 as ::core::ffi::c_int;
}
pub unsafe fn count_common_word(
    mut lp: *mut slang_T,
    mut word: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut count: uint8_t,
) {
    let mut buf: [::core::ffi::c_char; 254] = [0; 254];
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if len == -1 as ::core::ffi::c_int {
        p = word;
    } else if len >= MAXWLEN as ::core::ffi::c_int {
        return;
    } else {
        xmemcpyz(
            &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            word as *const ::core::ffi::c_void,
            len as size_t,
        );
        p = &raw mut buf as *mut ::core::ffi::c_char;
    }
    let mut hash: hash_T = hash_hash(p);
    let p_len: size_t = strlen(p);
    let mut hi: *mut hashitem_T = hash_lookup(&raw mut (*lp).sl_wordcount, p, p_len, hash);
    if (*hi).hi_key.is_null() || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
    {
        let mut wc: *mut wordcount_T =
            xmalloc((2 as size_t).wrapping_add(p_len).wrapping_add(1 as size_t))
                as *mut wordcount_T;
        memcpy(
            &raw mut (*wc).wc_word as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            p_len.wrapping_add(1 as size_t),
        );
        (*wc).wc_count = count as uint16_t;
        hash_add_item(
            &raw mut (*lp).sl_wordcount,
            hi,
            &raw mut (*wc).wc_word as *mut ::core::ffi::c_char,
            hash,
        );
    } else {
        let mut wc_0: *mut wordcount_T =
            (*hi).hi_key.offset(-(WC_KEY_OFF as isize)) as *mut wordcount_T;
        (*wc_0).wc_count =
            ((*wc_0).wc_count as ::core::ffi::c_int + count as ::core::ffi::c_int) as uint16_t;
        if ((*wc_0).wc_count as ::core::ffi::c_int) < count as ::core::ffi::c_int {
            (*wc_0).wc_count = MAXWORDCOUNT as ::core::ffi::c_int as uint16_t;
        }
    };
}
pub unsafe fn init_syl_tab(mut slang: *mut slang_T) -> ::core::ffi::c_int {
    ga_init(
        &raw mut (*slang).sl_syl_items,
        ::core::mem::size_of::<syl_item_T>() as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
    );
    let mut p: *mut ::core::ffi::c_char =
        vim_strchr((*slang).sl_syllable, '/' as ::core::ffi::c_int);
    while !p.is_null() {
        let c2rust_fresh5 = p;
        p = p.offset(1);
        *c2rust_fresh5 = NUL as ::core::ffi::c_char;
        if *p as ::core::ffi::c_int == NUL {
            break;
        }
        let mut s: *mut ::core::ffi::c_char = p;
        p = vim_strchr(p, '/' as ::core::ffi::c_int);
        let mut l: ::core::ffi::c_int = 0;
        if p.is_null() {
            l = strlen(s) as ::core::ffi::c_int;
        } else {
            l = p.offset_from(s) as ::core::ffi::c_int;
        }
        if l >= SY_MAXLEN {
            return SP_FORMERROR as ::core::ffi::c_int;
        }
        let mut syl: *mut syl_item_T = ga_append_via_ptr(
            &raw mut (*slang).sl_syl_items,
            ::core::mem::size_of::<syl_item_T>(),
        ) as *mut syl_item_T;
        xmemcpyz(
            &raw mut (*syl).sy_chars as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            s as *const ::core::ffi::c_void,
            l as size_t,
        );
        (*syl).sy_len = l;
    }
    return OK;
}
unsafe fn count_syllables(
    mut slang: *mut slang_T,
    mut word: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if (*slang).sl_syllable.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let mut cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut skip: bool = false_0 != 0;
    let mut len: ::core::ffi::c_int = 0;
    let mut p: *const ::core::ffi::c_char = word;
    while *p as ::core::ffi::c_int != NUL {
        if *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
            len = 1 as ::core::ffi::c_int;
            cnt = 0 as ::core::ffi::c_int;
        } else {
            len = 0 as ::core::ffi::c_int;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*slang).sl_syl_items.ga_len {
                let mut syl: *mut syl_item_T =
                    ((*slang).sl_syl_items.ga_data as *mut syl_item_T).offset(i as isize);
                if (*syl).sy_len > len
                    && strncmp(
                        p,
                        &raw mut (*syl).sy_chars as *mut ::core::ffi::c_char,
                        (*syl).sy_len as size_t,
                    ) == 0 as ::core::ffi::c_int
                {
                    len = (*syl).sy_len;
                }
                i += 1;
            }
            if len != 0 as ::core::ffi::c_int {
                cnt += 1;
                skip = false_0 != 0;
            } else {
                let mut c: ::core::ffi::c_int = utf_ptr2char(p);
                len = utfc_ptr2len(p);
                if vim_strchr((*slang).sl_syllable, c).is_null() {
                    skip = false_0 != 0;
                } else if !skip {
                    cnt += 1;
                    skip = true_0 != 0;
                }
            }
        }
        p = p.offset(len as isize);
    }
    return cnt;
}
pub unsafe fn parse_spelllang(mut wp: *mut win_T) -> *mut ::core::ffi::c_char {
    let mut spf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut region_cp: [::core::ffi::c_char; 3] = [0; 3];
    let mut lang: [::core::ffi::c_char; 255] = [0; 255];
    let mut spf_name: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut use_region: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dont_use_region: bool = false_0 != 0;
    let mut nobreak: bool = false_0 != 0;
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut ret_msg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    set_bufref(&raw mut bufref, (*wp).w_buffer);
    if recursive.get() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    recursive.set(true_0 != 0);
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<langp_T>() as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
    );
    clear_midword(wp);
    let mut spl_copy: *mut ::core::ffi::c_char = xstrdup((*(*wp).w_s).b_p_spl);
    (*(*wp).w_s).b_cjk = 0 as ::core::ffi::c_int;
    let mut splp: *mut ::core::ffi::c_char = spl_copy;
    '_theend: {
        while *splp as ::core::ffi::c_int != NUL {
            let mut len: ::core::ffi::c_int = copy_option_part(
                &raw mut splp,
                &raw mut lang as *mut ::core::ffi::c_char,
                MAXWLEN as ::core::ffi::c_int as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) as ::core::ffi::c_int;
            let mut region: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            if !valid_spelllang(&raw mut lang as *mut ::core::ffi::c_char) {
                continue;
            }
            if strcmp(
                &raw mut lang as *mut ::core::ffi::c_char,
                b"cjk\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*(*wp).w_s).b_cjk = 1 as ::core::ffi::c_int;
            } else {
                let mut slang: *mut slang_T = ::core::ptr::null_mut::<slang_T>();
                let mut filename: bool = false;
                if len > 4 as ::core::ffi::c_int
                    && path_fnamecmp(
                        (&raw mut lang as *mut ::core::ffi::c_char)
                            .offset(len as isize)
                            .offset(-(4 as ::core::ffi::c_int as isize)),
                        b".spl\0".as_ptr() as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                {
                    filename = true_0 != 0;
                    let mut p: *mut ::core::ffi::c_char = vim_strchr(
                        path_tail(&raw mut lang as *mut ::core::ffi::c_char),
                        '_' as ::core::ffi::c_int,
                    );
                    if !p.is_null()
                        && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            >= 'A' as ::core::ffi::c_uint
                            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                <= 'Z' as ::core::ffi::c_uint
                            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                >= 'a' as ::core::ffi::c_uint
                                && *p.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_uint
                                    <= 'z' as ::core::ffi::c_uint)
                        && (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            >= 'A' as ::core::ffi::c_uint
                            && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                <= 'Z' as ::core::ffi::c_uint
                            || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                >= 'a' as ::core::ffi::c_uint
                                && *p.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_uint
                                    <= 'z' as ::core::ffi::c_uint)
                        && !(*p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            >= 'A' as ::core::ffi::c_uint
                            && *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                <= 'Z' as ::core::ffi::c_uint
                            || *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                >= 'a' as ::core::ffi::c_uint
                                && *p.offset(3 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_uint
                                    <= 'z' as ::core::ffi::c_uint)
                    {
                        xstrlcpy(
                            &raw mut region_cp as *mut ::core::ffi::c_char,
                            p.offset(1 as ::core::ffi::c_int as isize),
                            3 as size_t,
                        );
                        memmove(
                            p as *mut ::core::ffi::c_void,
                            p.offset(3 as ::core::ffi::c_int as isize)
                                as *const ::core::ffi::c_void,
                            (len as isize
                                - p.offset_from(&raw mut lang as *mut ::core::ffi::c_char)
                                - 2 as isize) as size_t,
                        );
                        region = &raw mut region_cp as *mut ::core::ffi::c_char;
                    } else {
                        dont_use_region = true_0 != 0;
                    }
                    slang = first_lang.get();
                    while !slang.is_null() {
                        if path_full_compare(
                            &raw mut lang as *mut ::core::ffi::c_char,
                            (*slang).sl_fname,
                            false_0 != 0,
                            true_0 != 0,
                        ) as ::core::ffi::c_uint
                            == kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            break;
                        }
                        slang = (*slang).sl_next;
                    }
                } else {
                    filename = false_0 != 0;
                    if len > 3 as ::core::ffi::c_int
                        && lang[(len - 3 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
                            == '_' as ::core::ffi::c_int
                    {
                        region = (&raw mut lang as *mut ::core::ffi::c_char)
                            .offset(len as isize)
                            .offset(-(2 as ::core::ffi::c_int as isize));
                        lang[(len - 3 as ::core::ffi::c_int) as usize] = NUL as ::core::ffi::c_char;
                    } else {
                        dont_use_region = true_0 != 0;
                    }
                    slang = first_lang.get();
                    while !slang.is_null() {
                        if strcasecmp(&raw mut lang as *mut ::core::ffi::c_char, (*slang).sl_name)
                            == 0 as ::core::ffi::c_int
                        {
                            break;
                        }
                        slang = (*slang).sl_next;
                    }
                }
                if !region.is_null() {
                    if !use_region.is_null()
                        && strcmp(region, use_region) != 0 as ::core::ffi::c_int
                    {
                        dont_use_region = true_0 != 0;
                    }
                    use_region = region;
                }
                if slang.is_null() {
                    if filename {
                        spell_load_file(
                            &raw mut lang as *mut ::core::ffi::c_char,
                            &raw mut lang as *mut ::core::ffi::c_char,
                            ::core::ptr::null_mut::<slang_T>(),
                            false_0 != 0,
                        );
                    } else {
                        spell_load_lang(&raw mut lang as *mut ::core::ffi::c_char);
                        if !bufref_valid(&raw mut bufref) || !win_valid_any_tab(wp) {
                            ret_msg = b"E797: SpellFileMissing autocommand deleted buffer\0"
                                .as_ptr()
                                as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                            break '_theend;
                        }
                    }
                }
                slang = first_lang.get();
                while !slang.is_null() {
                    if if filename as ::core::ffi::c_int != 0 {
                        (path_full_compare(
                            &raw mut lang as *mut ::core::ffi::c_char,
                            (*slang).sl_fname,
                            false_0 != 0,
                            true_0 != 0,
                        ) as ::core::ffi::c_uint
                            == kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint)
                            as ::core::ffi::c_int
                    } else {
                        (strcasecmp(&raw mut lang as *mut ::core::ffi::c_char, (*slang).sl_name)
                            == 0 as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                    } != 0
                    {
                        let mut region_mask: ::core::ffi::c_int = REGION_ALL as ::core::ffi::c_int;
                        if !filename && !region.is_null() {
                            let mut c: ::core::ffi::c_int = find_region(
                                &raw mut (*slang).sl_regions as *mut ::core::ffi::c_char,
                                region,
                            );
                            if c == REGION_ALL as ::core::ffi::c_int {
                                if (*slang).sl_add {
                                    if *(&raw mut (*slang).sl_regions as *mut ::core::ffi::c_char)
                                        as ::core::ffi::c_int
                                        != NUL
                                    {
                                        region_mask = 0 as ::core::ffi::c_int;
                                    }
                                } else {
                                    smsg(
                                        0 as ::core::ffi::c_int,
                                        gettext(b"Warning: region %s not supported\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                        region,
                                    );
                                }
                            } else {
                                region_mask = (1 as ::core::ffi::c_int) << c;
                            }
                        }
                        if region_mask != 0 as ::core::ffi::c_int {
                            let mut p_: *mut langp_T =
                                ga_append_via_ptr(&raw mut ga, ::core::mem::size_of::<langp_T>())
                                    as *mut langp_T;
                            (*p_).lp_slang = slang;
                            (*p_).lp_region = region_mask;
                            use_midword(slang, wp);
                            if (*slang).sl_nobreak {
                                nobreak = true_0 != 0;
                            }
                        }
                    }
                    slang = (*slang).sl_next;
                }
            }
        }
        spf = (*(*curwin.get()).w_s).b_p_spf;
        let mut round: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while round == 0 as ::core::ffi::c_int || *spf as ::core::ffi::c_int != NUL {
            's_377: {
                if round == 0 as ::core::ffi::c_int {
                    if (*int_wordlist.ptr()).is_null() {
                        break 's_377;
                    } else {
                        int_wordlist_spl(&raw mut spf_name as *mut ::core::ffi::c_char);
                    }
                } else {
                    let mut len_0: ::core::ffi::c_int = copy_option_part(
                        &raw mut spf,
                        &raw mut spf_name as *mut ::core::ffi::c_char,
                        (MAXPATHL - 4 as ::core::ffi::c_int) as size_t,
                        b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    ) as ::core::ffi::c_int;
                    strcpy(
                        (&raw mut spf_name as *mut ::core::ffi::c_char).offset(len_0 as isize),
                        b".spl\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    );
                    let mut c_0: ::core::ffi::c_int = 0;
                    c_0 = 0 as ::core::ffi::c_int;
                    while c_0 < ga.ga_len {
                        let mut p_0: *mut ::core::ffi::c_char =
                            (*(*(ga.ga_data as *mut langp_T).offset(c_0 as isize)).lp_slang)
                                .sl_fname;
                        if !p_0.is_null()
                            && path_full_compare(
                                &raw mut spf_name as *mut ::core::ffi::c_char,
                                p_0,
                                false_0 != 0,
                                true_0 != 0,
                            ) as ::core::ffi::c_uint
                                == kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            break;
                        }
                        c_0 += 1;
                    }
                    if c_0 < ga.ga_len {
                        break 's_377;
                    }
                }
                let mut slang_0: *mut slang_T = ::core::ptr::null_mut::<slang_T>();
                slang_0 = first_lang.get();
                while !slang_0.is_null() {
                    if path_full_compare(
                        &raw mut spf_name as *mut ::core::ffi::c_char,
                        (*slang_0).sl_fname,
                        false_0 != 0,
                        true_0 != 0,
                    ) as ::core::ffi::c_uint
                        == kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        break;
                    }
                    slang_0 = (*slang_0).sl_next;
                }
                if slang_0.is_null() {
                    if round == 0 as ::core::ffi::c_int {
                        strcpy(
                            &raw mut lang as *mut ::core::ffi::c_char,
                            b"internal wordlist\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                        );
                    } else {
                        xstrlcpy(
                            &raw mut lang as *mut ::core::ffi::c_char,
                            path_tail(&raw mut spf_name as *mut ::core::ffi::c_char),
                            (MAXWLEN as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t,
                        );
                        let mut p_1: *mut ::core::ffi::c_char = vim_strchr(
                            &raw mut lang as *mut ::core::ffi::c_char,
                            '.' as ::core::ffi::c_int,
                        );
                        if !p_1.is_null() {
                            *p_1 = NUL as ::core::ffi::c_char;
                        }
                    }
                    slang_0 = spell_load_file(
                        &raw mut spf_name as *mut ::core::ffi::c_char,
                        &raw mut lang as *mut ::core::ffi::c_char,
                        ::core::ptr::null_mut::<slang_T>(),
                        true_0 != 0,
                    );
                    if !slang_0.is_null() && nobreak as ::core::ffi::c_int != 0 {
                        (*slang_0).sl_nobreak = true_0 != 0;
                    }
                }
                if !slang_0.is_null() {
                    let mut region_mask_0: ::core::ffi::c_int = REGION_ALL as ::core::ffi::c_int;
                    if !use_region.is_null() && !dont_use_region {
                        let mut c_1: ::core::ffi::c_int = find_region(
                            &raw mut (*slang_0).sl_regions as *mut ::core::ffi::c_char,
                            use_region,
                        );
                        if c_1 != REGION_ALL as ::core::ffi::c_int {
                            region_mask_0 = (1 as ::core::ffi::c_int) << c_1;
                        } else if *(&raw mut (*slang_0).sl_regions as *mut ::core::ffi::c_char)
                            as ::core::ffi::c_int
                            != NUL
                        {
                            region_mask_0 = 0 as ::core::ffi::c_int;
                        }
                    }
                    if region_mask_0 != 0 as ::core::ffi::c_int {
                        let mut p__0: *mut langp_T =
                            ga_append_via_ptr(&raw mut ga, ::core::mem::size_of::<langp_T>())
                                as *mut langp_T;
                        (*p__0).lp_slang = slang_0;
                        (*p__0).lp_sallang = ::core::ptr::null_mut::<slang_T>();
                        (*p__0).lp_replang = ::core::ptr::null_mut::<slang_T>();
                        (*p__0).lp_region = region_mask_0;
                        use_midword(slang_0, wp);
                    }
                }
            }
            round += 1;
        }
        ga_clear(&raw mut (*(*wp).w_s).b_langp);
        (*(*wp).w_s).b_langp = ga;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < ga.ga_len {
            let mut lp: *mut langp_T = (ga.ga_data as *mut langp_T).offset(i as isize);
            if !((*(*lp).lp_slang).sl_sal.ga_len <= 0 as ::core::ffi::c_int) {
                (*lp).lp_sallang = (*lp).lp_slang;
            } else {
                let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while j < ga.ga_len {
                    let mut lp2: *mut langp_T = (ga.ga_data as *mut langp_T).offset(j as isize);
                    if !((*(*lp2).lp_slang).sl_sal.ga_len <= 0 as ::core::ffi::c_int)
                        && strncmp(
                            (*(*lp).lp_slang).sl_name,
                            (*(*lp2).lp_slang).sl_name,
                            2 as size_t,
                        ) == 0 as ::core::ffi::c_int
                    {
                        (*lp).lp_sallang = (*lp2).lp_slang;
                        break;
                    } else {
                        j += 1;
                    }
                }
            }
            if !((*(*lp).lp_slang).sl_rep.ga_len <= 0 as ::core::ffi::c_int) {
                (*lp).lp_replang = (*lp).lp_slang;
            } else {
                let mut j_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while j_0 < ga.ga_len {
                    let mut lp2_0: *mut langp_T = (ga.ga_data as *mut langp_T).offset(j_0 as isize);
                    if !((*(*lp2_0).lp_slang).sl_rep.ga_len <= 0 as ::core::ffi::c_int)
                        && strncmp(
                            (*(*lp).lp_slang).sl_name,
                            (*(*lp2_0).lp_slang).sl_name,
                            2 as size_t,
                        ) == 0 as ::core::ffi::c_int
                    {
                        (*lp).lp_replang = (*lp2_0).lp_slang;
                        break;
                    } else {
                        j_0 += 1;
                    }
                }
            }
            i += 1;
        }
        redraw_later(wp, UPD_NOT_VALID);
    }
    xfree(spl_copy as *mut ::core::ffi::c_void);
    recursive.set(false_0 != 0);
    return ret_msg;
}
unsafe fn clear_midword(mut wp: *mut win_T) {
    memset(
        &raw mut (*(*wp).w_s).b_spell_ismw as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[bool; 256]>(),
    );
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*(*wp).w_s).b_spell_ismw_mb as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
}
unsafe fn use_midword(mut lp: *mut slang_T, mut wp: *mut win_T) {
    if (*lp).sl_midword.is_null() {
        return;
    }
    let mut p: *mut ::core::ffi::c_char = (*lp).sl_midword;
    while *p as ::core::ffi::c_int != NUL {
        let c: ::core::ffi::c_int = utf_ptr2char(p);
        let l: ::core::ffi::c_int = utfc_ptr2len(p);
        if c < 256 as ::core::ffi::c_int && l <= 2 as ::core::ffi::c_int {
            (*(*wp).w_s).b_spell_ismw[c as usize] = true_0 != 0;
        } else if (*(*wp).w_s).b_spell_ismw_mb.is_null() {
            (*(*wp).w_s).b_spell_ismw_mb =
                xmemdupz(p as *const ::core::ffi::c_void, l as size_t) as *mut ::core::ffi::c_char;
        } else {
            let n: ::core::ffi::c_int = strlen((*(*wp).w_s).b_spell_ismw_mb) as ::core::ffi::c_int;
            let mut bp: *mut ::core::ffi::c_char = xstrnsave(
                (*(*wp).w_s).b_spell_ismw_mb,
                (n as size_t).wrapping_add(l as size_t),
            );
            xfree((*(*wp).w_s).b_spell_ismw_mb as *mut ::core::ffi::c_void);
            (*(*wp).w_s).b_spell_ismw_mb = bp;
            xmemcpyz(
                bp.offset(n as isize) as *mut ::core::ffi::c_void,
                p as *const ::core::ffi::c_void,
                l as size_t,
            );
        }
        p = p.offset(l as isize);
    }
}
unsafe fn find_region(
    mut rp: *const ::core::ffi::c_char,
    mut region: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    loop {
        if *rp.offset(i as isize) as ::core::ffi::c_int == NUL {
            return REGION_ALL as ::core::ffi::c_int;
        }
        if *rp.offset(i as isize) as ::core::ffi::c_int
            == *region.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            && *rp.offset((i + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                == *region.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        {
            break;
        }
        i += 2 as ::core::ffi::c_int;
    }
    return i / 2 as ::core::ffi::c_int;
}
pub unsafe fn spell_delete_wordlist() {
    if (*int_wordlist.ptr()).is_null() {
        return;
    }
    let mut fname: [::core::ffi::c_char; 4096] = [0 as ::core::ffi::c_char; 4096];
    os_remove(int_wordlist.get());
    int_wordlist_spl(&raw mut fname as *mut ::core::ffi::c_char);
    os_remove(&raw mut fname as *mut ::core::ffi::c_char);
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        int_wordlist.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
}
pub unsafe fn spell_free_all() {
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        ga_clear(&raw mut (*buf).b_s.b_langp);
        buf = (*buf).b_next;
    }
    while !(*first_lang.ptr()).is_null() {
        let mut slang: *mut slang_T = first_lang.get();
        first_lang.set((*slang).sl_next);
        slang_free(slang);
    }
    spell_delete_wordlist();
    let mut ptr_: *mut *mut ::core::ffi::c_void = repl_to.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        repl_from.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL;
    let _ = *ptr__0;
}
pub unsafe fn spell_reload() {
    init_spell_chartab();
    spell_free_all();
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if *(*(*wp).w_s).b_p_spl as ::core::ffi::c_int != NUL {
            if (*wp).w_onebuf_opt.wo_spell != 0 {
                parse_spelllang(wp);
                break;
            }
        }
        wp = (*wp).w_next;
    }
}
pub unsafe fn open_spellbuf() -> *mut buf_T {
    let mut buf: *mut buf_T = xcalloc(1 as size_t, ::core::mem::size_of::<buf_T>()) as *mut buf_T;
    (*buf).b_spell = true_0 != 0;
    (*buf).b_p_swf = true_0;
    if ml_open(buf) == FAIL {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"open_spellbuf\0".as_ptr() as *const ::core::ffi::c_char,
            2387 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error opening a new memline\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    ml_open_file(buf);
    return buf;
}
pub unsafe fn close_spellbuf(mut buf: *mut buf_T) {
    if buf.is_null() {
        return;
    }
    ml_close(buf, true_0);
    xfree(buf as *mut ::core::ffi::c_void);
}
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
pub unsafe fn valid_spelllang(mut val: *const ::core::ffi::c_char) -> bool {
    return valid_name(val, b".-_,@\0".as_ptr() as *const ::core::ffi::c_char);
}
pub unsafe fn valid_spellfile(mut val: *const ::core::ffi::c_char) -> bool {
    let mut spf_name: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut spf: *mut ::core::ffi::c_char = val as *mut ::core::ffi::c_char;
    while *spf as ::core::ffi::c_int != NUL {
        let mut l: size_t = copy_option_part(
            &raw mut spf,
            &raw mut spf_name as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if l >= (MAXPATHL - 4 as ::core::ffi::c_int) as size_t
            || l < 4 as size_t
            || strcmp(
                (&raw mut spf_name as *mut ::core::ffi::c_char)
                    .offset(l as isize)
                    .offset(-(4 as ::core::ffi::c_int as isize)),
                b".add\0".as_ptr() as *const ::core::ffi::c_char,
            ) != 0 as ::core::ffi::c_int
        {
            return false_0 != 0;
        }
        let mut s: *mut ::core::ffi::c_char = &raw mut spf_name as *mut ::core::ffi::c_char;
        while *s as ::core::ffi::c_int != NUL {
            if !vim_is_fname_char(*s as uint8_t as ::core::ffi::c_int) {
                return false_0 != 0;
            }
            s = s.offset(1);
        }
    }
    return true_0 != 0;
}
pub unsafe fn did_set_spell_option() -> *const ::core::ffi::c_char {
    let mut errmsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer == curbuf.get() && (*wp).w_onebuf_opt.wo_spell != 0 {
            errmsg = parse_spelllang(wp);
            break;
        } else {
            wp = (*wp).w_next;
        }
    }
    return errmsg;
}
pub unsafe fn compile_cap_prog(mut synblock: *mut synblock_T) -> *const ::core::ffi::c_char {
    let mut rp: *mut regprog_T = (*synblock).b_cap_prog;
    if (*synblock).b_p_spc.is_null() || *(*synblock).b_p_spc as ::core::ffi::c_int == NUL {
        (*synblock).b_cap_prog = ::core::ptr::null_mut::<regprog_T>();
    } else {
        let mut re: *mut ::core::ffi::c_char = concat_str(
            b"^\0".as_ptr() as *const ::core::ffi::c_char,
            (*synblock).b_p_spc,
        );
        (*synblock).b_cap_prog = vim_regcomp(re, RE_MAGIC);
        xfree(re as *mut ::core::ffi::c_void);
        if (*synblock).b_cap_prog.is_null() {
            (*synblock).b_cap_prog = rp;
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
    }
    vim_regfree(rp);
    return ::core::ptr::null::<::core::ffi::c_char>();
}
pub const SPL_FNAME_TMPL: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"%s.%s.spl\0") };
pub const WC_KEY_OFF: ::core::ffi::c_ulong = 2 as ::core::ffi::c_ulong;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
