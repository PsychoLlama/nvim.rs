use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::arglist::get_arglist_exp;
use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::buffer::buflist_findname_exp;
use crate::src::nvim::charset::{getdigits_int, skipdigits, skipwhite};
use crate::src::nvim::drawscreen::{UPD_SOME_VALID, redraw_all_later};
use crate::src::nvim::fileio::{buf_reload, vim_fgets, vim_tempname};
use crate::src::nvim::garray::{
    ga_append, ga_append_via_ptr, ga_clear, ga_concat, ga_grow, ga_init,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::hashtab::{hash_add, hash_clear, hash_clear_all, hash_find, hash_init};
use crate::src::nvim::main::{
    IObuff, NameBuff, curbuf, curwin, e_bufloaded, e_exists, e_invarg, e_isadir2, e_notopen,
    e_notset, got_int, p_enc, p_msm, p_verbose,
};
use crate::src::nvim::mbyte::{
    convert_setup, enc_canonize, mb_ptr2char_adv, mb_toupper, string_convert, utf_head_off,
    utf_ptr2char, utfc_ptr2len,
};
use crate::src::nvim::memory::{xfree, xmalloc, xmemcpyz, xstrdup, xstrlcat, xstrlcpy};
use crate::src::nvim::message::{emsg, msg, semsg, smsg, verbose_enter, verbose_leave};
use crate::src::nvim::option::{copy_option_part, set_option_value_give_err};
use crate::src::nvim::options::kOptSpellfile;
use crate::src::nvim::os::env::home_replace;
use crate::src::nvim::os::fs::{os_fopen, os_isdir, os_mkdir, os_mkdir_recurse, os_path_exists};
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, __ctype_b_loc, __errno_location, atoi, fclose, fprintf, fputc, fseek, ftell,
    gettext, memmove, memset, snprintf, strcat, strcmp, strcpy, strerror, strlen, strncmp, strstr,
};
use crate::src::nvim::os::stdpaths::get_xdg_home;
use crate::src::nvim::path::{
    FreeWild, dir_of_file_exists, path_tail, path_tail_with_sep, vim_ispathsep,
};
use crate::src::nvim::spell::{
    did_set_spelltab, init_spell_chartab, int_wordlist, onecap_copy, spell_casefold, spell_enc,
    spelltab,
};
use crate::src::nvim::strings::{has_non_ascii, vim_snprintf, vim_strchr};
pub use crate::src::nvim::types::{
    __compar_fn_t, __off_t, __off64_t, __time_t, _IO_FILE, _IO_codecvt, _IO_lock_t, _IO_marker,
    _IO_wide_data, AdditionalData, AlignTextPos, Array, AutoPat, AutoPatCmd, AutoPatCmd_S,
    BoolVarValue, Boolean, BufUpdateCallbacks, CMD_index, Callback,
    Callback_data as C2Rust_Unnamed_5, CallbackType, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_2, Dict, ExtmarkUndoObject, FILE, FileComparison, FileID,
    Float, FloatAnchor, FloatRelative, GridView, Integer, Intersection, KeyValuePair, LineGetter,
    LuaRef, MTKey, MTNode, MTPos, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MapHash, MarkTree, Object, ObjectType, OptIndex, OptInt, OptVal,
    OptValData, OptValType, QUEUE, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t,
    Set_uint32_t, Set_uint64_t, SpecialVarValue, SpellAddType, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, String_0, Terminal, Timestamp, TriState,
    VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo,
    WinSplit, WinStyle, Window, XDGVarType, alist_T, auto_event, bhdr_T, blob_T, blobvar_S,
    blocknr_T, buf_T, bufstate_T, chunksize_T, cmd_addr_T, cmdidx_T, colnr_T, cstack_T,
    cstack_T_cs_pend as C2Rust_Unnamed_13, dict_T, dictvar_S, disptick_T, eslist_T, eslist_elem,
    estack_T, estack_T_es_info as C2Rust_Unnamed_16, etype_T, event_T, exarg, exarg_T, except_T,
    except_type_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_3, file_buffer_b_wininfo as C2Rust_Unnamed_11,
    file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, file_comparison, float_T, fmark_T, fmarkv_T,
    frame_S, frame_T, fromto_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T,
    garray_T, handle_T, hash_T, hashitem_T, hashtab_T, iconv_t, idx_T, infoptr_T, int16_t, int32_t,
    int64_t, key_value_pair, langp_T, lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T,
    listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock, mapblock_T, match_T, matchitem,
    matchitem_T, memfile_T, memline_T, mfdirty_T, msglist, msglist_T, mtnode_inner_s, mtnode_s,
    object, object_data as C2Rust_Unnamed_14, partial_S, partial_T, pos_T, pos_save_T, proftime_T,
    ptr_t, qf_info_S, qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T,
    salfirst_T, salitem_T, sattr_T, schar_T, scid_T, sctx_T, size_t, slang_S, slang_T, spelltab_T,
    syn_state, syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T,
    taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header,
    u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8, u_header_uh_alt_prev as C2Rust_Unnamed_7,
    u_header_uh_next as C2Rust_Unnamed_10, u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T,
    uint8_t, uint16_t, uint32_t, uint64_t, uintmax_t, uintptr_t, undo_object, varnumber_T,
    vim_exception, vimconv_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T,
    wline_T, xfmark_T,
};
use crate::src::nvim::ui::ui_flush;
use crate::src::nvim::undo::bufIsChanged;
mod dic;
mod read;
mod sections;
mod sugfile;
mod wordfile;
mod wordtree;
mod write;
use dic::spell_read_dic;
use read::spell_reload_one;
pub use read::{spell_load_file, suggest_load_files};
use sugfile::spell_make_sugfile;
use wordfile::spell_read_wordfile;
use wordtree::{
    MSG_COMPRESSING, SpellArena, set_compression_limits, tree_add_word, valid_spell_word,
    wordnode_T, wordtree_alloc, wordtree_compress,
};
use write::write_vim_spell;
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
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const CMD_spellwrong: CMD_index = 427;
pub const CMD_spellundo: CMD_index = 426;
pub const CMD_spellrare: CMD_index = 425;
pub const kOptValTypeString: OptValType = 2;
pub const ETYPE_SPELL: etype_T = 9;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const CONV_NONE: C2Rust_Unnamed_17 = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const OPT_LOCAL: C2Rust_Unnamed_18 = 2;
pub const kXDGDataHome: XDGVarType = 1;
pub const kEqualFiles: file_comparison = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const MAXWLEN: C2Rust_Unnamed_19 = 254;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const MAXREGIONS: C2Rust_Unnamed_20 = 8;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const WF_KEEPCAP: C2Rust_Unnamed_21 = 128;
pub const WF_FIXCAP: C2Rust_Unnamed_21 = 64;
pub const WF_AFX: C2Rust_Unnamed_21 = 32;
pub const WF_BANNED: C2Rust_Unnamed_21 = 16;
pub const WF_RARE: C2Rust_Unnamed_21 = 8;
pub const WF_REGION: C2Rust_Unnamed_21 = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const WF_NOCOMPAFT: C2Rust_Unnamed_22 = 8192;
pub const WF_NOCOMPBEF: C2Rust_Unnamed_22 = 4096;
pub const WF_COMPROOT: C2Rust_Unnamed_22 = 2048;
pub const WF_NOSUGGEST: C2Rust_Unnamed_22 = 1024;
pub const WF_NEEDCOMP: C2Rust_Unnamed_22 = 512;
pub const WF_HAS_AFF: C2Rust_Unnamed_22 = 256;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const WFP_COMPFORBID: C2Rust_Unnamed_23 = 16;
pub const WFP_COMPPERMIT: C2Rust_Unnamed_23 = 8;
pub const WFP_UP: C2Rust_Unnamed_23 = 4;
pub const WFP_NC: C2Rust_Unnamed_23 = 2;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const COMP_CHECKTRIPLE: C2Rust_Unnamed_24 = 8;
pub const COMP_CHECKCASE: C2Rust_Unnamed_24 = 4;
pub const COMP_CHECKREP: C2Rust_Unnamed_24 = 2;
pub const COMP_CHECKDUP: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_int;
pub const SP_OTHERERROR: C2Rust_Unnamed_25 = -3;
pub const SP_FORMERROR: C2Rust_Unnamed_25 = -2;
pub const SP_TRUNCERROR: C2Rust_Unnamed_25 = -1;
pub const SPELL_ADD_RARE: SpellAddType = 2;
pub const SPELL_ADD_BAD: SpellAddType = 1;
pub const SPELL_ADD_GOOD: SpellAddType = 0;
pub const BY_FLAGS2: C2Rust_Unnamed_28 = 3;
pub const BY_FLAGS: C2Rust_Unnamed_28 = 2;
pub const BY_INDEX: C2Rust_Unnamed_28 = 1;
pub const BY_NOFLAGS: C2Rust_Unnamed_28 = 0;
pub const BY_SPECIAL: C2Rust_Unnamed_28 = 3;
pub const SN_SYLLABLE: C2Rust_Unnamed_30 = 9;
pub const SN_NOBREAK: C2Rust_Unnamed_30 = 10;
pub const SN_COMPOUND: C2Rust_Unnamed_30 = 8;
pub const SN_NOCOMPOUNDSUGS: C2Rust_Unnamed_30 = 16;
pub const SN_NOSPLITSUGS: C2Rust_Unnamed_30 = 14;
pub const SN_SUGFILE: C2Rust_Unnamed_30 = 11;
pub const SN_WORDS: C2Rust_Unnamed_30 = 13;
pub const SN_MAP: C2Rust_Unnamed_30 = 7;
pub const SN_SOFO: C2Rust_Unnamed_30 = 6;
pub const SAL_REM_ACCENTS: C2Rust_Unnamed_29 = 4;
pub const SAL_COLLAPSE: C2Rust_Unnamed_29 = 2;
pub const SAL_F0LLOWUP: C2Rust_Unnamed_29 = 1;
pub const SN_SAL: C2Rust_Unnamed_30 = 5;
pub const SN_REPSAL: C2Rust_Unnamed_30 = 12;
pub const SN_REP: C2Rust_Unnamed_30 = 4;
pub const SN_PREFCOND: C2Rust_Unnamed_30 = 3;
pub const SN_MIDWORD: C2Rust_Unnamed_30 = 2;
pub const CF_UPPER: C2Rust_Unnamed_31 = 2;
pub const CF_WORD: C2Rust_Unnamed_31 = 1;
pub const SN_CHARFLAGS: C2Rust_Unnamed_30 = 1;
pub const SN_REGION: C2Rust_Unnamed_30 = 0;
pub const SN_INFO: C2Rust_Unnamed_30 = 15;
pub const SN_END: C2Rust_Unnamed_30 = 255;
pub struct spellinfo_T {
    pub si_foldroot: *mut wordnode_T,
    pub si_foldwcount: ::core::ffi::c_int,
    pub si_keeproot: *mut wordnode_T,
    pub si_keepwcount: ::core::ffi::c_int,
    pub si_prefroot: *mut wordnode_T,
    pub si_sugtree: ::core::ffi::c_int,
    pub si_arena: SpellArena,
    pub si_did_emsg: ::core::ffi::c_int,
    pub si_compress_cnt: ::core::ffi::c_int,
    pub si_first_free: *mut wordnode_T,
    pub si_free_count: ::core::ffi::c_int,
    pub si_spellbuf: *mut buf_T,
    pub si_ascii: ::core::ffi::c_int,
    pub si_add: ::core::ffi::c_int,
    pub si_clear_chartab: ::core::ffi::c_int,
    pub si_region: ::core::ffi::c_int,
    pub si_conv: vimconv_T,
    pub si_memtot: ::core::ffi::c_int,
    pub si_verbose: ::core::ffi::c_int,
    pub si_msg_count: ::core::ffi::c_int,
    pub si_info: *mut ::core::ffi::c_char,
    pub si_region_count: ::core::ffi::c_int,
    pub si_region_name: [::core::ffi::c_char; 17],
    pub si_rep: garray_T,
    pub si_repsal: garray_T,
    pub si_sal: garray_T,
    pub si_sofofr: *mut ::core::ffi::c_char,
    pub si_sofoto: *mut ::core::ffi::c_char,
    pub si_nosugfile: ::core::ffi::c_int,
    pub si_nosplitsugs: ::core::ffi::c_int,
    pub si_nocompoundsugs: ::core::ffi::c_int,
    pub si_followup: ::core::ffi::c_int,
    pub si_collapse: ::core::ffi::c_int,
    pub si_commonwords: hashtab_T,
    pub si_sugtime: time_t,
    pub si_rem_accents: ::core::ffi::c_int,
    pub si_map: garray_T,
    pub si_midword: *mut ::core::ffi::c_char,
    pub si_compmax: ::core::ffi::c_int,
    pub si_compminlen: ::core::ffi::c_int,
    pub si_compsylmax: ::core::ffi::c_int,
    pub si_compoptions: ::core::ffi::c_int,
    pub si_comppat: garray_T,
    pub si_compflags: *mut ::core::ffi::c_char,
    pub si_nobreak: ::core::ffi::c_char,
    pub si_syllable: *mut ::core::ffi::c_char,
    pub si_prefcond: garray_T,
    pub si_newprefID: ::core::ffi::c_int,
    pub si_newcompID: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct afffile_T {
    pub af_enc: *mut ::core::ffi::c_char,
    pub af_flagtype: ::core::ffi::c_int,
    pub af_rare: ::core::ffi::c_uint,
    pub af_keepcase: ::core::ffi::c_uint,
    pub af_bad: ::core::ffi::c_uint,
    pub af_needaffix: ::core::ffi::c_uint,
    pub af_circumfix: ::core::ffi::c_uint,
    pub af_needcomp: ::core::ffi::c_uint,
    pub af_comproot: ::core::ffi::c_uint,
    pub af_compforbid: ::core::ffi::c_uint,
    pub af_comppermit: ::core::ffi::c_uint,
    pub af_nosuggest: ::core::ffi::c_uint,
    pub af_pfxpostpone: ::core::ffi::c_int,
    pub af_ignoreextra: bool,
    pub af_pref: hashtab_T,
    pub af_suff: hashtab_T,
    pub af_comp: hashtab_T,
}
pub type affentry_T = affentry_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct affentry_S {
    pub ae_next: *mut affentry_T,
    pub ae_chop: *mut ::core::ffi::c_char,
    pub ae_add: *mut ::core::ffi::c_char,
    pub ae_flags: *mut ::core::ffi::c_char,
    pub ae_cond: *mut ::core::ffi::c_char,
    pub ae_prog: *mut regprog_T,
    pub ae_compforbid: ::core::ffi::c_char,
    pub ae_comppermit: ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct affheader_T {
    pub ah_key: [::core::ffi::c_char; 17],
    pub ah_flag: ::core::ffi::c_uint,
    pub ah_newID: ::core::ffi::c_int,
    pub ah_combine: ::core::ffi::c_int,
    pub ah_follows: ::core::ffi::c_int,
    pub ah_first: *mut affentry_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct compitem_T {
    pub ci_key: [::core::ffi::c_char; 17],
    pub ci_flag: ::core::ffi::c_uint,
    pub ci_newID: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const EOF: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const SPL_FNAME_TMPL: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"%s.%s.spl\0") };
pub const SPL_FNAME_ADD: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b".add.\0") };
pub const SPL_FNAME_ASCII: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b".ascii.\0") };
pub const VIMSUGMAGIC: [::core::ffi::c_char; 7] =
    unsafe { ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"VIMsug\0") };
pub const VIMSUGMAGICL: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const VIMSUGVERSION: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ZERO_FLAG: ::core::ffi::c_int = 65009 as ::core::ffi::c_int;
pub const VIMSPELLMAGIC: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"VIMspell\0") };
pub const VIMSPELLMAGICL: usize =
    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize);
pub const VIMSPELLVERSION: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
pub const SNF_REQUIRED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const COMPOUND_MAX_LEN: ::core::ffi::c_int = 100000 as ::core::ffi::c_int;
static e_spell_trunc: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"E758: Truncated spell file\0".as_ptr() as *const ::core::ffi::c_char);
static e_error_while_reading_sug_file_str: GlobalCell<[::core::ffi::c_char; 40]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
            *b"E782: Error while reading .sug file: %s\0",
        )
    });
static e_duplicate_char_in_map_entry: GlobalCell<[::core::ffi::c_char; 34]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
            *b"E783: Duplicate char in MAP entry\0",
        )
    });
static e_illegal_character_in_word: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"E1280: Illegal character in word\0".as_ptr() as *const ::core::ffi::c_char);
static e_afftrailing: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"Trailing text in %s line %d: %s\0".as_ptr() as *const ::core::ffi::c_char);
static e_affname: GlobalCell<*const ::core::ffi::c_char> = GlobalCell::new(
    b"Affix name too long in %s line %d: %s\0".as_ptr() as *const ::core::ffi::c_char,
);
pub const MAXLINELEN: ::core::ffi::c_int = 500 as ::core::ffi::c_int;
pub const AFT_CHAR: ::core::ffi::c_int = 0;
pub const AFT_LONG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const AFT_CAPLONG: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const AFT_NUM: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const AH_KEY_LEN: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
#[inline(always)]
unsafe fn getroom(spin: *mut spellinfo_T, len: size_t, align: bool) -> *mut ::core::ffi::c_void {
    unsafe { (*spin).si_arena.alloc_bytes(len as usize, align).cast() }
}
unsafe fn getroom_save(
    spin: *mut spellinfo_T,
    s: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe { (*spin).si_arena.save_str(s) }
}
pub const PFX_FLAGS: ::core::ffi::c_int = -256 as ::core::ffi::c_int;
pub const CONDIT_COMB: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CONDIT_CFIX: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CONDIT_SUF: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const CONDIT_AFF: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub unsafe fn spell_check_msm() -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = p_msm.get();
    if !ascii_isdigit(*p as ::core::ffi::c_int) {
        return FAIL;
    }
    let mut start: ::core::ffi::c_int =
        getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int) * 10 as ::core::ffi::c_int
            / (wordtree::block_size() / 102 as ::core::ffi::c_int);
    if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
        return FAIL;
    }
    p = p.offset(1);
    if !ascii_isdigit(*p as ::core::ffi::c_int) {
        return FAIL;
    }
    let mut incr: ::core::ffi::c_int =
        getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int) * 102 as ::core::ffi::c_int
            / (wordtree::block_size() / 10 as ::core::ffi::c_int);
    if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
        return FAIL;
    }
    p = p.offset(1);
    if !ascii_isdigit(*p as ::core::ffi::c_int) {
        return FAIL;
    }
    let mut added: ::core::ffi::c_int =
        getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int)
            * 1024 as ::core::ffi::c_int;
    if *p as ::core::ffi::c_int != NUL {
        return FAIL;
    }
    if start == 0 as ::core::ffi::c_int
        || incr == 0 as ::core::ffi::c_int
        || added == 0 as ::core::ffi::c_int
        || incr > start
    {
        return FAIL;
    }
    set_compression_limits(start, incr, added);
    return OK;
}
unsafe fn spell_read_aff(
    mut spin: *mut spellinfo_T,
    mut fname: *mut ::core::ffi::c_char,
) -> *mut afffile_T {
    let mut rline: [::core::ffi::c_char; 500] = [0; 500];
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut items: [*mut ::core::ffi::c_char; 30] =
        [::core::ptr::null_mut::<::core::ffi::c_char>(); 30];
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut lnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cur_aff: *mut affheader_T = ::core::ptr::null_mut::<affheader_T>();
    let mut did_postpone_prefix: bool = false_0 != 0;
    let mut aff_todo: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tp: *mut hashtab_T = ::core::ptr::null_mut::<hashtab_T>();
    let mut low: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fol: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut upp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut found_map: bool = false_0 != 0;
    let mut hi: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
    let mut compminlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut compsylmax: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut compoptions: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut compmax: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut compflags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut midword: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut syllable: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut sofofrom: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut sofoto: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fd: *mut FILE = os_fopen(fname, b"r\0".as_ptr() as *const ::core::ffi::c_char);
    if fd.is_null() {
        semsg(
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            fname,
        );
        return ::core::ptr::null_mut::<afffile_T>();
    }
    vim_snprintf(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        gettext(b"Reading affix file %s...\0".as_ptr() as *const ::core::ffi::c_char),
        fname,
    );
    spell_message(spin, IObuff.ptr() as *mut ::core::ffi::c_char);
    let mut do_rep: bool = (*spin).si_rep.ga_len <= 0 as ::core::ffi::c_int;
    let mut do_repsal: bool = (*spin).si_repsal.ga_len <= 0 as ::core::ffi::c_int;
    let mut do_sal: bool = (*spin).si_sal.ga_len <= 0 as ::core::ffi::c_int;
    let mut do_mapline: bool = (*spin).si_map.ga_len <= 0 as ::core::ffi::c_int;
    let mut aff: *mut afffile_T =
        getroom(spin, ::core::mem::size_of::<afffile_T>(), true_0 != 0) as *mut afffile_T;
    hash_init(&raw mut (*aff).af_pref);
    hash_init(&raw mut (*aff).af_suff);
    hash_init(&raw mut (*aff).af_comp);
    while !vim_fgets(&raw mut rline as *mut ::core::ffi::c_char, MAXLINELEN, fd) && !got_int.get() {
        line_breakcheck();
        lnum += 1;
        if *(&raw mut rline as *mut ::core::ffi::c_char) as ::core::ffi::c_int
            == '#' as ::core::ffi::c_int
        {
            continue;
        }
        xfree(pc as *mut ::core::ffi::c_void);
        if (*spin).si_conv.vc_type != CONV_NONE as ::core::ffi::c_int {
            pc = string_convert(
                &raw mut (*spin).si_conv,
                &raw mut rline as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<size_t>(),
            );
            if pc.is_null() {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"Conversion failure for word in %s line %d: %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    fname,
                    lnum,
                    &raw mut rline as *mut ::core::ffi::c_char,
                );
                continue;
            } else {
                line = pc;
            }
        } else {
            pc = ::core::ptr::null_mut::<::core::ffi::c_char>();
            line = &raw mut rline as *mut ::core::ffi::c_char;
        }
        let mut itemcnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        p = line;
        loop {
            while *p as ::core::ffi::c_int != NUL
                && *p as uint8_t as ::core::ffi::c_int <= ' ' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
            if itemcnt == MAXITEMCNT {
                break;
            }
            let c2rust_fresh33 = itemcnt;
            itemcnt = itemcnt + 1;
            let c2rust_lvalue_ptr = &raw mut items[c2rust_fresh33 as usize];
            *c2rust_lvalue_ptr = p as *mut ::core::ffi::c_char;
            if itemcnt == 2 as ::core::ffi::c_int
                && spell_info_item(
                    items[0 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                ) as ::core::ffi::c_int
                    != 0
            {
                while *p as uint8_t as ::core::ffi::c_int >= ' ' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == TAB
                {
                    p = p.offset(1);
                }
            } else {
                while *p as uint8_t as ::core::ffi::c_int > ' ' as ::core::ffi::c_int {
                    p = p.offset(1);
                }
            }
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
            let c2rust_fresh34 = p;
            p = p.offset(1);
            *c2rust_fresh34 = NUL as ::core::ffi::c_char;
        }
        if itemcnt <= 0 as ::core::ffi::c_int {
            continue;
        }
        if is_aff_rule(
            &raw mut items as *mut *mut ::core::ffi::c_char,
            itemcnt,
            b"SET\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            2 as ::core::ffi::c_int,
        ) as ::core::ffi::c_int
            != 0
            && (*aff).af_enc.is_null()
        {
            (*aff).af_enc =
                enc_canonize(items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char);
            if (*spin).si_ascii == 0
                && convert_setup(&raw mut (*spin).si_conv, (*aff).af_enc, p_enc.get()) == FAIL
            {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"Conversion in %s not supported: from %s to %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    fname,
                    (*aff).af_enc,
                    p_enc.get(),
                );
            }
            (*spin).si_conv.vc_fail = true_0 != 0;
        } else if is_aff_rule(
            &raw mut items as *mut *mut ::core::ffi::c_char,
            itemcnt,
            b"FLAG\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            2 as ::core::ffi::c_int,
        ) as ::core::ffi::c_int
            != 0
            && (*aff).af_flagtype == AFT_CHAR
        {
            if strcmp(
                items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                b"long\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*aff).af_flagtype = AFT_LONG;
            } else if strcmp(
                items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                b"num\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*aff).af_flagtype = AFT_NUM;
            } else if strcmp(
                items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                b"caplong\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*aff).af_flagtype = AFT_CAPLONG;
            } else {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"Invalid value for FLAG in %s line %d: %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    fname,
                    lnum,
                    items[1 as ::core::ffi::c_int as usize],
                );
            }
            if (*aff).af_rare != 0 as ::core::ffi::c_uint
                || (*aff).af_keepcase != 0 as ::core::ffi::c_uint
                || (*aff).af_bad != 0 as ::core::ffi::c_uint
                || (*aff).af_needaffix != 0 as ::core::ffi::c_uint
                || (*aff).af_circumfix != 0 as ::core::ffi::c_uint
                || (*aff).af_needcomp != 0 as ::core::ffi::c_uint
                || (*aff).af_comproot != 0 as ::core::ffi::c_uint
                || (*aff).af_nosuggest != 0 as ::core::ffi::c_uint
                || !compflags.is_null()
                || (*aff).af_suff.ht_used > 0 as size_t
                || (*aff).af_pref.ht_used > 0 as size_t
            {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"FLAG after using flags in %s line %d: %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    fname,
                    lnum,
                    items[1 as ::core::ffi::c_int as usize],
                );
            }
        } else if spell_info_item(
            items[0 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
        ) as ::core::ffi::c_int
            != 0
            && itemcnt > 1 as ::core::ffi::c_int
        {
            p = getroom(
                spin,
                (if (*spin).si_info.is_null() {
                    0 as size_t
                } else {
                    strlen((*spin).si_info)
                })
                .wrapping_add(strlen(
                    items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                ))
                .wrapping_add(strlen(
                    items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                ))
                .wrapping_add(3 as size_t),
                false_0 != 0,
            ) as *mut ::core::ffi::c_char;
            if !(*spin).si_info.is_null() {
                strcpy(p, (*spin).si_info);
                strcat(p, b"\n\0".as_ptr() as *const ::core::ffi::c_char);
            }
            strcat(
                p,
                items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
            );
            strcat(p, b" \0".as_ptr() as *const ::core::ffi::c_char);
            strcat(
                p,
                items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
            );
            (*spin).si_info = p;
        } else if is_aff_rule(
            &raw mut items as *mut *mut ::core::ffi::c_char,
            itemcnt,
            b"MIDWORD\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            2 as ::core::ffi::c_int,
        ) as ::core::ffi::c_int
            != 0
            && midword.is_null()
        {
            midword = getroom_save(
                spin,
                items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
            );
        } else {
            if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"TRY\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) {
                continue;
            }
            if (is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"RAR\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                || is_aff_rule(
                    &raw mut items as *mut *mut ::core::ffi::c_char,
                    itemcnt,
                    b"RARE\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    2 as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0)
                && (*aff).af_rare == 0 as ::core::ffi::c_uint
            {
                (*aff).af_rare = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
            } else if (is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"KEP\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                || is_aff_rule(
                    &raw mut items as *mut *mut ::core::ffi::c_char,
                    itemcnt,
                    b"KEEPCASE\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    2 as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0)
                && (*aff).af_keepcase == 0 as ::core::ffi::c_uint
            {
                (*aff).af_keepcase = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
            } else if (is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"BAD\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                || is_aff_rule(
                    &raw mut items as *mut *mut ::core::ffi::c_char,
                    itemcnt,
                    b"FORBIDDENWORD\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    2 as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0)
                && (*aff).af_bad == 0 as ::core::ffi::c_uint
            {
                (*aff).af_bad = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"NEEDAFFIX\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && (*aff).af_needaffix == 0 as ::core::ffi::c_uint
            {
                (*aff).af_needaffix = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"CIRCUMFIX\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && (*aff).af_circumfix == 0 as ::core::ffi::c_uint
            {
                (*aff).af_circumfix = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"NOSUGGEST\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && (*aff).af_nosuggest == 0 as ::core::ffi::c_uint
            {
                (*aff).af_nosuggest = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
            } else if (is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"NEEDCOMPOUND\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                || is_aff_rule(
                    &raw mut items as *mut *mut ::core::ffi::c_char,
                    itemcnt,
                    b"ONLYINCOMPOUND\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    2 as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0)
                && (*aff).af_needcomp == 0 as ::core::ffi::c_uint
            {
                (*aff).af_needcomp = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDROOT\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && (*aff).af_comproot == 0 as ::core::ffi::c_uint
            {
                (*aff).af_comproot = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDFORBIDFLAG\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && (*aff).af_compforbid == 0 as ::core::ffi::c_uint
            {
                (*aff).af_compforbid = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
                if (*aff).af_pref.ht_used > 0 as size_t {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(
                            b"Defining COMPOUNDFORBIDFLAG after PFX item may give wrong results in %s line %d\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        fname,
                        lnum,
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDPERMITFLAG\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && (*aff).af_comppermit == 0 as ::core::ffi::c_uint
            {
                (*aff).af_comppermit = affitem2flag(
                    (*aff).af_flagtype,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    fname,
                    lnum,
                );
                if (*aff).af_pref.ht_used > 0 as size_t {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(
                            b"Defining COMPOUNDPERMITFLAG after PFX item may give wrong results in %s line %d\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        fname,
                        lnum,
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDFLAG\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && compflags.is_null()
            {
                p = getroom(
                    spin,
                    strlen(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char)
                        .wrapping_add(2 as size_t),
                    false_0 != 0,
                ) as *mut ::core::ffi::c_char;
                strcpy(p, items[1 as ::core::ffi::c_int as usize]);
                strcat(p, b"+\0".as_ptr() as *const ::core::ffi::c_char);
                compflags = p;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDRULES\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) {
                if atoi(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"Wrong COMPOUNDRULES value in %s line %d: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        fname,
                        lnum,
                        items[1 as ::core::ffi::c_int as usize],
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDRULE\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) {
                if !compflags.is_null()
                    || *skipdigits(
                        items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    ) as ::core::ffi::c_int
                        != NUL
                {
                    let mut l: ::core::ffi::c_int = strlen(
                        items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    ) as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int;
                    if !compflags.is_null() {
                        l += strlen(compflags) as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
                    }
                    p = getroom(spin, l as size_t, false_0 != 0) as *mut ::core::ffi::c_char;
                    if !compflags.is_null() {
                        strcpy(p, compflags);
                        strcat(p, b"/\0".as_ptr() as *const ::core::ffi::c_char);
                    }
                    strcat(
                        p,
                        items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    );
                    compflags = p;
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDWORDMAX\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && compmax == 0 as ::core::ffi::c_int
            {
                compmax =
                    atoi(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char);
                if compmax == 0 as ::core::ffi::c_int {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"Wrong COMPOUNDWORDMAX value in %s line %d: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        fname,
                        lnum,
                        items[1 as ::core::ffi::c_int as usize],
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDMIN\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && compminlen == 0 as ::core::ffi::c_int
            {
                compminlen =
                    atoi(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char);
                if compminlen == 0 as ::core::ffi::c_int {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"Wrong COMPOUNDMIN value in %s line %d: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        fname,
                        lnum,
                        items[1 as ::core::ffi::c_int as usize],
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"COMPOUNDSYLMAX\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && compsylmax == 0 as ::core::ffi::c_int
            {
                compsylmax =
                    atoi(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char);
                if compsylmax == 0 as ::core::ffi::c_int {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"Wrong COMPOUNDSYLMAX value in %s line %d: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        fname,
                        lnum,
                        items[1 as ::core::ffi::c_int as usize],
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"CHECKCOMPOUNDDUP\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                compoptions |= COMP_CHECKDUP as ::core::ffi::c_int;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"CHECKCOMPOUNDREP\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                compoptions |= COMP_CHECKREP as ::core::ffi::c_int;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"CHECKCOMPOUNDCASE\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                compoptions |= COMP_CHECKCASE as ::core::ffi::c_int;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"CHECKCOMPOUNDTRIPLE\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                compoptions |= COMP_CHECKTRIPLE as ::core::ffi::c_int;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"CHECKCOMPOUNDPATTERN\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) {
                if atoi(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(
                            b"Wrong CHECKCOMPOUNDPATTERN value in %s line %d: %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        fname,
                        lnum,
                        items[1 as ::core::ffi::c_int as usize],
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"CHECKCOMPOUNDPATTERN\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                3 as ::core::ffi::c_int,
            ) {
                let mut gap: *mut garray_T = &raw mut (*spin).si_comppat;
                let mut i: ::core::ffi::c_int = 0;
                i = 0 as ::core::ffi::c_int;
                while i < (*gap).ga_len - 1 as ::core::ffi::c_int {
                    if strcmp(
                        *((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize),
                        items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                        && strcmp(
                            *((*gap).ga_data as *mut *mut ::core::ffi::c_char)
                                .offset((i + 1 as ::core::ffi::c_int) as isize),
                            items[2 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                        ) == 0 as ::core::ffi::c_int
                    {
                        break;
                    }
                    i += 2 as ::core::ffi::c_int;
                }
                if i >= (*gap).ga_len {
                    ga_grow(gap, 2 as ::core::ffi::c_int);
                    let c2rust_fresh35 = (*gap).ga_len;
                    (*gap).ga_len = (*gap).ga_len + 1;
                    let c2rust_lvalue_ptr_0 = &raw mut *((*gap).ga_data
                        as *mut *mut ::core::ffi::c_char)
                        .offset(c2rust_fresh35 as isize);
                    *c2rust_lvalue_ptr_0 = getroom_save(
                        spin,
                        items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    );
                    let c2rust_fresh36 = (*gap).ga_len;
                    (*gap).ga_len = (*gap).ga_len + 1;
                    let c2rust_lvalue_ptr_1 = &raw mut *((*gap).ga_data
                        as *mut *mut ::core::ffi::c_char)
                        .offset(c2rust_fresh36 as isize);
                    *c2rust_lvalue_ptr_1 = getroom_save(
                        spin,
                        items[2 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"SYLLABLE\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && syllable.is_null()
            {
                syllable = getroom_save(
                    spin,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                );
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"NOBREAK\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                (*spin).si_nobreak = true_0 as ::core::ffi::c_char;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"NOSPLITSUGS\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                (*spin).si_nosplitsugs = true_0;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"NOCOMPOUNDSUGS\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                (*spin).si_nocompoundsugs = true_0;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"NOSUGFILE\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                (*spin).si_nosugfile = true_0;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"PFXPOSTPONE\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                (*aff).af_pfxpostpone = true_0;
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"IGNOREEXTRA\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) {
                (*aff).af_ignoreextra = true_0 != 0;
            } else if (strcmp(
                items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                b"PFX\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
                || strcmp(
                    items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    b"SFX\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int)
                && aff_todo == 0 as ::core::ffi::c_int
                && itemcnt >= 4 as ::core::ffi::c_int
            {
                let mut lasti: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
                let mut key: [::core::ffi::c_char; 17] = [0; 17];
                if *items[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    == 'P' as ::core::ffi::c_int
                {
                    tp = &raw mut (*aff).af_pref;
                } else {
                    tp = &raw mut (*aff).af_suff;
                }
                xstrlcpy(
                    &raw mut key as *mut ::core::ffi::c_char,
                    items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    AH_KEY_LEN as size_t,
                );
                hi = hash_find(tp, &raw mut key as *mut ::core::ffi::c_char);
                if !((*hi).hi_key.is_null()
                    || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
                {
                    cur_aff = (*hi).hi_key as *mut affheader_T;
                    if (*cur_aff).ah_combine
                        != (*items[2 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                            == 'Y' as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                    {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(
                                b"Different combining flag in continued affix block in %s line %d: %s\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            ),
                            fname,
                            lnum,
                            items[1 as ::core::ffi::c_int as usize],
                        );
                    }
                    if (*cur_aff).ah_follows == 0 {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(b"Duplicate affix in %s line %d: %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            fname,
                            lnum,
                            items[1 as ::core::ffi::c_int as usize],
                        );
                    }
                } else {
                    cur_aff = getroom(spin, ::core::mem::size_of::<affheader_T>(), true_0 != 0)
                        as *mut affheader_T;
                    (*cur_aff).ah_flag = affitem2flag(
                        (*aff).af_flagtype,
                        items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                        fname,
                        lnum,
                    );
                    if (*cur_aff).ah_flag == 0 as ::core::ffi::c_uint
                        || strlen(
                            items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                        ) >= AH_KEY_LEN as size_t
                    {
                        break;
                    }
                    if (*cur_aff).ah_flag == (*aff).af_bad
                        || (*cur_aff).ah_flag == (*aff).af_rare
                        || (*cur_aff).ah_flag == (*aff).af_keepcase
                        || (*cur_aff).ah_flag == (*aff).af_needaffix
                        || (*cur_aff).ah_flag == (*aff).af_circumfix
                        || (*cur_aff).ah_flag == (*aff).af_nosuggest
                        || (*cur_aff).ah_flag == (*aff).af_needcomp
                        || (*cur_aff).ah_flag == (*aff).af_comproot
                    {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(
                                b"Affix also used for BAD/RARE/KEEPCASE/NEEDAFFIX/NEEDCOMPOUND/NOSUGGEST in %s line %d: %s\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            ),
                            fname,
                            lnum,
                            items[1 as ::core::ffi::c_int as usize],
                        );
                    }
                    strcpy(
                        &raw mut (*cur_aff).ah_key as *mut ::core::ffi::c_char,
                        items[1 as ::core::ffi::c_int as usize],
                    );
                    hash_add(tp, &raw mut (*cur_aff).ah_key as *mut ::core::ffi::c_char);
                    (*cur_aff).ah_combine = (*items[2 as ::core::ffi::c_int as usize]
                        as ::core::ffi::c_int
                        == 'Y' as ::core::ffi::c_int)
                        as ::core::ffi::c_int;
                }
                if itemcnt > lasti
                    && strcmp(
                        items[lasti as usize] as *const ::core::ffi::c_char,
                        b"S\0".as_ptr() as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                {
                    lasti += 1;
                    (*cur_aff).ah_follows = true_0;
                } else {
                    (*cur_aff).ah_follows = false_0;
                }
                if itemcnt > lasti
                    && !(*aff).af_ignoreextra
                    && *items[lasti as usize] as ::core::ffi::c_int != '#' as ::core::ffi::c_int
                {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(e_afftrailing.get()),
                        fname,
                        lnum,
                        items[lasti as usize],
                    );
                }
                if strcmp(
                    items[2 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    b"Y\0".as_ptr() as *const ::core::ffi::c_char,
                ) != 0 as ::core::ffi::c_int
                    && strcmp(
                        items[2 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                        b"N\0".as_ptr() as *const ::core::ffi::c_char,
                    ) != 0 as ::core::ffi::c_int
                {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"Expected Y or N in %s line %d: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        fname,
                        lnum,
                        items[2 as ::core::ffi::c_int as usize],
                    );
                }
                if *items[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    == 'P' as ::core::ffi::c_int
                    && (*aff).af_pfxpostpone != 0
                {
                    if (*cur_aff).ah_newID == 0 as ::core::ffi::c_int {
                        check_renumber(spin);
                        (*spin).si_newprefID += 1;
                        (*cur_aff).ah_newID = (*spin).si_newprefID;
                        did_postpone_prefix = false_0 != 0;
                    } else {
                        did_postpone_prefix = true_0 != 0;
                    }
                }
                aff_todo =
                    atoi(items[3 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char);
            } else if (strcmp(
                items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                b"PFX\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
                || strcmp(
                    items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    b"SFX\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int)
                && aff_todo > 0 as ::core::ffi::c_int
                && strcmp(
                    &raw mut (*cur_aff).ah_key as *mut ::core::ffi::c_char,
                    items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                && itemcnt >= 5 as ::core::ffi::c_int
            {
                let mut aff_entry: *mut affentry_T = ::core::ptr::null_mut::<affentry_T>();
                let mut lasti_0: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
                if itemcnt > lasti_0
                    && *items[lasti_0 as usize] as ::core::ffi::c_int != '#' as ::core::ffi::c_int
                    && (strcmp(
                        items[lasti_0 as usize] as *const ::core::ffi::c_char,
                        b"-\0".as_ptr() as *const ::core::ffi::c_char,
                    ) != 0 as ::core::ffi::c_int
                        || itemcnt != lasti_0 + 1 as ::core::ffi::c_int)
                {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(e_afftrailing.get()),
                        fname,
                        lnum,
                        items[lasti_0 as usize],
                    );
                }
                aff_todo -= 1;
                aff_entry = getroom(spin, ::core::mem::size_of::<affentry_T>(), true_0 != 0)
                    as *mut affentry_T;
                if strcmp(
                    items[2 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    b"0\0".as_ptr() as *const ::core::ffi::c_char,
                ) != 0 as ::core::ffi::c_int
                {
                    (*aff_entry).ae_chop = getroom_save(
                        spin,
                        items[2 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    );
                }
                if strcmp(
                    items[3 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    b"0\0".as_ptr() as *const ::core::ffi::c_char,
                ) != 0 as ::core::ffi::c_int
                {
                    (*aff_entry).ae_add = getroom_save(
                        spin,
                        items[3 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    );
                    (*aff_entry).ae_flags =
                        vim_strchr((*aff_entry).ae_add, '/' as ::core::ffi::c_int);
                    if !(*aff_entry).ae_flags.is_null() {
                        let c2rust_fresh37 = (*aff_entry).ae_flags;
                        (*aff_entry).ae_flags = (*aff_entry).ae_flags.offset(1);
                        *c2rust_fresh37 = NUL as ::core::ffi::c_char;
                        aff_process_flags(aff, aff_entry);
                    }
                }
                if (*spin).si_ascii == 0
                    || !(has_non_ascii((*aff_entry).ae_chop) as ::core::ffi::c_int != 0
                        || has_non_ascii((*aff_entry).ae_add) as ::core::ffi::c_int != 0)
                {
                    (*aff_entry).ae_next = (*cur_aff).ah_first;
                    (*cur_aff).ah_first = aff_entry;
                    if strcmp(
                        items[4 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                        b".\0".as_ptr() as *const ::core::ffi::c_char,
                    ) != 0 as ::core::ffi::c_int
                    {
                        let mut buf: [::core::ffi::c_char; 500] = [0; 500];
                        (*aff_entry).ae_cond = getroom_save(
                            spin,
                            items[4 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                        );
                        snprintf(
                            &raw mut buf as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 500]>(),
                            if *items[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                                == 'P' as ::core::ffi::c_int
                            {
                                b"^%s\0".as_ptr() as *const ::core::ffi::c_char
                            } else {
                                b"%s$\0".as_ptr() as *const ::core::ffi::c_char
                            },
                            items[4 as ::core::ffi::c_int as usize],
                        );
                        (*aff_entry).ae_prog = vim_regcomp(
                            &raw mut buf as *mut ::core::ffi::c_char,
                            RE_MAGIC + RE_STRING + RE_STRICT,
                        );
                        if (*aff_entry).ae_prog.is_null() {
                            smsg(
                                0 as ::core::ffi::c_int,
                                gettext(b"Broken condition in %s line %d: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                fname,
                                lnum,
                                items[4 as ::core::ffi::c_int as usize],
                            );
                        }
                    }
                    if *items[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                        == 'P' as ::core::ffi::c_int
                        && (*aff).af_pfxpostpone != 0
                        && (*aff_entry).ae_flags.is_null()
                    {
                        let mut upper: bool = false_0 != 0;
                        if !(*aff_entry).ae_chop.is_null()
                            && !(*aff_entry).ae_add.is_null()
                            && *(*aff_entry)
                                .ae_chop
                                .offset(utfc_ptr2len((*aff_entry).ae_chop) as isize)
                                as ::core::ffi::c_int
                                == NUL
                        {
                            let mut c: ::core::ffi::c_int = utf_ptr2char((*aff_entry).ae_chop);
                            let mut c_up: ::core::ffi::c_int = if c >= 128 as ::core::ffi::c_int {
                                mb_toupper(c)
                            } else {
                                (*spelltab.ptr()).st_upper[c as usize] as ::core::ffi::c_int
                            };
                            if c_up != c
                                && ((*aff_entry).ae_cond.is_null()
                                    || utf_ptr2char((*aff_entry).ae_cond) == c)
                            {
                                p = (*aff_entry)
                                    .ae_add
                                    .offset(strlen((*aff_entry).ae_add) as isize);
                                p = p.offset(
                                    -((utf_head_off(
                                        (*aff_entry).ae_add,
                                        p.offset(-(1 as ::core::ffi::c_int as isize)),
                                    ) + 1 as ::core::ffi::c_int)
                                        as isize),
                                );
                                if utf_ptr2char(p) == c_up {
                                    upper = true_0 != 0;
                                    (*aff_entry).ae_chop =
                                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                                    *p = NUL as ::core::ffi::c_char;
                                    if !(*aff_entry).ae_cond.is_null() {
                                        let mut buf_0: [::core::ffi::c_char; 500] = [0; 500];
                                        onecap_copy(
                                            items[4 as ::core::ffi::c_int as usize]
                                                as *const ::core::ffi::c_char,
                                            &raw mut buf_0 as *mut ::core::ffi::c_char,
                                            true_0 != 0,
                                        );
                                        (*aff_entry).ae_cond = getroom_save(
                                            spin,
                                            &raw mut buf_0 as *mut ::core::ffi::c_char,
                                        );
                                        if !(*aff_entry).ae_cond.is_null() {
                                            snprintf(
                                                &raw mut buf_0 as *mut ::core::ffi::c_char,
                                                MAXLINELEN as size_t,
                                                b"^%s\0".as_ptr() as *const ::core::ffi::c_char,
                                                (*aff_entry).ae_cond,
                                            );
                                            vim_regfree((*aff_entry).ae_prog);
                                            (*aff_entry).ae_prog = vim_regcomp(
                                                &raw mut buf_0 as *mut ::core::ffi::c_char,
                                                RE_MAGIC + RE_STRING,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        if (*aff_entry).ae_chop.is_null() {
                            let mut idx: ::core::ffi::c_int = 0;
                            idx = (*spin).si_prefcond.ga_len - 1 as ::core::ffi::c_int;
                            while idx >= 0 as ::core::ffi::c_int {
                                p = *((*spin).si_prefcond.ga_data as *mut *mut ::core::ffi::c_char)
                                    .offset(idx as isize);
                                if str_equal(p, (*aff_entry).ae_cond) {
                                    break;
                                }
                                idx -= 1;
                            }
                            if idx < 0 as ::core::ffi::c_int {
                                idx = (*spin).si_prefcond.ga_len;
                                let mut pp: *mut *mut ::core::ffi::c_char = ga_append_via_ptr(
                                    &raw mut (*spin).si_prefcond,
                                    ::core::mem::size_of::<*mut ::core::ffi::c_char>(),
                                )
                                    as *mut *mut ::core::ffi::c_char;
                                *pp = if (*aff_entry).ae_cond.is_null() {
                                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                                } else {
                                    getroom_save(spin, (*aff_entry).ae_cond)
                                };
                            }
                            if (*aff_entry).ae_add.is_null() {
                                p = b"\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            } else {
                                p = (*aff_entry).ae_add;
                            }
                            let mut n: ::core::ffi::c_int = PFX_FLAGS;
                            if (*cur_aff).ah_combine == 0 {
                                n |= WFP_NC as ::core::ffi::c_int;
                            }
                            if upper {
                                n |= WFP_UP as ::core::ffi::c_int;
                            }
                            if (*aff_entry).ae_comppermit != 0 {
                                n |= WFP_COMPPERMIT as ::core::ffi::c_int;
                            }
                            if (*aff_entry).ae_compforbid != 0 {
                                n |= WFP_COMPFORBID as ::core::ffi::c_int;
                            }
                            let prefroot = (*spin).si_prefroot;
                            let newID = (*cur_aff).ah_newID;
                            tree_add_word(&mut *spin, p, prefroot, n, idx, newID);
                            did_postpone_prefix = true_0 != 0;
                        }
                        if aff_todo == 0 as ::core::ffi::c_int && !did_postpone_prefix {
                            (*spin).si_newprefID -= 1;
                            (*cur_aff).ah_newID = 0 as ::core::ffi::c_int;
                        }
                    }
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"FOL\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && fol.is_null()
            {
                fol =
                    xstrdup(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char);
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"LOW\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && low.is_null()
            {
                low =
                    xstrdup(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char);
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"UPP\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && upp.is_null()
            {
                upp =
                    xstrdup(items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char);
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"REP\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                || is_aff_rule(
                    &raw mut items as *mut *mut ::core::ffi::c_char,
                    itemcnt,
                    b"REPSAL\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    2 as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0
            {
                if *(*__ctype_b_loc()).offset(
                    *items[1 as ::core::ffi::c_int as usize] as uint8_t as ::core::ffi::c_int
                        as isize,
                ) as ::core::ffi::c_int
                    & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0
                {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"Expected REP(SAL) count in %s line %d\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        fname,
                        lnum,
                    );
                }
            } else if (strcmp(
                items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                b"REP\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
                || strcmp(
                    items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    b"REPSAL\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int)
                && itemcnt >= 3 as ::core::ffi::c_int
            {
                if itemcnt > 3 as ::core::ffi::c_int
                    && *items[3 as ::core::ffi::c_int as usize]
                        .offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        != '#' as ::core::ffi::c_int
                {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(e_afftrailing.get()),
                        fname,
                        lnum,
                        items[3 as ::core::ffi::c_int as usize],
                    );
                }
                if if *items[0 as ::core::ffi::c_int as usize]
                    .offset(3 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == 'S' as ::core::ffi::c_int
                {
                    do_repsal as ::core::ffi::c_int
                } else {
                    do_rep as ::core::ffi::c_int
                } != 0
                {
                    p = items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char;
                    while *p as ::core::ffi::c_int != NUL {
                        if *p as ::core::ffi::c_int == '_' as ::core::ffi::c_int {
                            *p = ' ' as ::core::ffi::c_char;
                        }
                        p = p.offset(utfc_ptr2len(p) as isize);
                    }
                    p = items[2 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char;
                    while *p as ::core::ffi::c_int != NUL {
                        if *p as ::core::ffi::c_int == '_' as ::core::ffi::c_int {
                            *p = ' ' as ::core::ffi::c_char;
                        }
                        p = p.offset(utfc_ptr2len(p) as isize);
                    }
                    add_fromto(
                        spin,
                        if *items[0 as ::core::ffi::c_int as usize]
                            .offset(3 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == 'S' as ::core::ffi::c_int
                        {
                            &raw mut (*spin).si_repsal
                        } else {
                            &raw mut (*spin).si_rep
                        },
                        items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                        items[2 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                    );
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"MAP\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) {
                if !found_map {
                    found_map = true_0 != 0;
                    if *(*__ctype_b_loc()).offset(
                        *items[1 as ::core::ffi::c_int as usize] as uint8_t as ::core::ffi::c_int
                            as isize,
                    ) as ::core::ffi::c_int
                        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort
                            as ::core::ffi::c_int
                        == 0
                    {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(b"Expected MAP count in %s line %d\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            fname,
                            lnum,
                        );
                    }
                } else if do_mapline {
                    p = items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char;
                    while *p as ::core::ffi::c_int != NUL {
                        let mut c_0: ::core::ffi::c_int =
                            mb_ptr2char_adv(&raw mut p as *mut *const ::core::ffi::c_char);
                        if !((*spin).si_map.ga_len <= 0 as ::core::ffi::c_int)
                            && !vim_strchr(
                                (*spin).si_map.ga_data as *const ::core::ffi::c_char,
                                c_0,
                            )
                            .is_null()
                            || !vim_strchr(p, c_0).is_null()
                        {
                            smsg(
                                0 as ::core::ffi::c_int,
                                gettext(b"Duplicate character in MAP in %s line %d\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                fname,
                                lnum,
                            );
                        }
                    }
                    ga_concat(
                        &raw mut (*spin).si_map,
                        items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                    );
                    ga_append(&raw mut (*spin).si_map, '/' as uint8_t);
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"SAL\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                3 as ::core::ffi::c_int,
            ) {
                if do_sal {
                    if strcmp(
                        items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                        b"followup\0".as_ptr() as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        (*spin).si_followup = sal_to_bool(
                            items[2 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                        ) as ::core::ffi::c_int;
                    } else if strcmp(
                        items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                        b"collapse_result\0".as_ptr() as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        (*spin).si_collapse = sal_to_bool(
                            items[2 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                        ) as ::core::ffi::c_int;
                    } else if strcmp(
                        items[1 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                        b"remove_accents\0".as_ptr() as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        (*spin).si_rem_accents = sal_to_bool(
                            items[2 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                        ) as ::core::ffi::c_int;
                    } else {
                        add_fromto(
                            spin,
                            &raw mut (*spin).si_sal,
                            items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                            (if strcmp(
                                items[2 as ::core::ffi::c_int as usize]
                                    as *const ::core::ffi::c_char,
                                b"_\0".as_ptr() as *const ::core::ffi::c_char,
                            ) == 0 as ::core::ffi::c_int
                            {
                                b"\0".as_ptr() as *const ::core::ffi::c_char
                            } else {
                                items[2 as ::core::ffi::c_int as usize]
                                    as *const ::core::ffi::c_char
                            }) as *mut ::core::ffi::c_char,
                        );
                    }
                }
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"SOFOFROM\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && sofofrom.is_null()
            {
                sofofrom = getroom_save(
                    spin,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                );
            } else if is_aff_rule(
                &raw mut items as *mut *mut ::core::ffi::c_char,
                itemcnt,
                b"SOFOTO\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                && sofoto.is_null()
            {
                sofoto = getroom_save(
                    spin,
                    items[1 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_char,
                );
            } else if strcmp(
                items[0 as ::core::ffi::c_int as usize] as *const ::core::ffi::c_char,
                b"COMMON\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                let mut i_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                while i_0 < itemcnt {
                    if (*hash_find(
                        &raw mut (*spin).si_commonwords,
                        items[i_0 as usize] as *const ::core::ffi::c_char,
                    ))
                    .hi_key
                    .is_null()
                        || (*hash_find(
                            &raw mut (*spin).si_commonwords,
                            items[i_0 as usize] as *const ::core::ffi::c_char,
                        ))
                        .hi_key
                            == &raw const hash_removed as *mut ::core::ffi::c_char
                    {
                        p = xstrdup(items[i_0 as usize] as *const ::core::ffi::c_char);
                        hash_add(&raw mut (*spin).si_commonwords, p);
                    }
                    i_0 += 1;
                }
            } else {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(
                        b"Unrecognized or duplicate item in %s line %d: %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    fname,
                    lnum,
                    items[0 as ::core::ffi::c_int as usize],
                );
            }
        }
    }
    if !fol.is_null() || !low.is_null() || !upp.is_null() {
        if (*spin).si_clear_chartab != 0 {
            init_spell_chartab();
            (*spin).si_clear_chartab = false_0;
        }
        xfree(fol as *mut ::core::ffi::c_void);
        xfree(low as *mut ::core::ffi::c_void);
        xfree(upp as *mut ::core::ffi::c_void);
    }
    if compmax != 0 as ::core::ffi::c_int {
        aff_check_number(
            (*spin).si_compmax,
            compmax,
            b"COMPOUNDWORDMAX\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        (*spin).si_compmax = compmax;
    }
    if compminlen != 0 as ::core::ffi::c_int {
        aff_check_number(
            (*spin).si_compminlen,
            compminlen,
            b"COMPOUNDMIN\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        (*spin).si_compminlen = compminlen;
    }
    if compsylmax != 0 as ::core::ffi::c_int {
        if syllable.is_null() {
            smsg(
                0 as ::core::ffi::c_int,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                gettext(b"COMPOUNDSYLMAX used without SYLLABLE\0".as_ptr()
                    as *const ::core::ffi::c_char),
            );
        }
        aff_check_number(
            (*spin).si_compsylmax,
            compsylmax,
            b"COMPOUNDSYLMAX\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        (*spin).si_compsylmax = compsylmax;
    }
    if compoptions != 0 as ::core::ffi::c_int {
        aff_check_number(
            (*spin).si_compoptions,
            compoptions,
            b"COMPOUND options\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        );
        (*spin).si_compoptions |= compoptions;
    }
    if !compflags.is_null() {
        process_compflags(spin, aff, compflags);
    }
    if (*spin).si_newcompID < (*spin).si_newprefID {
        if (*spin).si_newcompID == 127 as ::core::ffi::c_int
            || (*spin).si_newcompID == 255 as ::core::ffi::c_int
        {
            msg(
                gettext(b"Too many postponed prefixes\0".as_ptr() as *const ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
            );
        } else if (*spin).si_newprefID == 0 as ::core::ffi::c_int
            || (*spin).si_newprefID == 127 as ::core::ffi::c_int
        {
            msg(
                gettext(b"Too many compound flags\0".as_ptr() as *const ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
            );
        } else {
            msg(
                gettext(
                    b"Too many postponed prefixes and/or compound flags\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                0 as ::core::ffi::c_int,
            );
        }
    }
    if !syllable.is_null() {
        aff_check_string(
            (*spin).si_syllable,
            syllable,
            b"SYLLABLE\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        (*spin).si_syllable = syllable;
    }
    if !sofofrom.is_null() || !sofoto.is_null() {
        if sofofrom.is_null() || sofoto.is_null() {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Missing SOFO%s line in %s\0".as_ptr() as *const ::core::ffi::c_char),
                if sofofrom.is_null() {
                    b"FROM\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"TO\0".as_ptr() as *const ::core::ffi::c_char
                },
                fname,
            );
        } else if !((*spin).si_sal.ga_len <= 0 as ::core::ffi::c_int) {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Both SAL and SOFO lines in %s\0".as_ptr() as *const ::core::ffi::c_char),
                fname,
            );
        } else {
            aff_check_string(
                (*spin).si_sofofr,
                sofofrom,
                b"SOFOFROM\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            aff_check_string(
                (*spin).si_sofoto,
                sofoto,
                b"SOFOTO\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            (*spin).si_sofofr = sofofrom;
            (*spin).si_sofoto = sofoto;
        }
    }
    if !midword.is_null() {
        aff_check_string(
            (*spin).si_midword,
            midword,
            b"MIDWORD\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        (*spin).si_midword = midword;
    }
    xfree(pc as *mut ::core::ffi::c_void);
    fclose(fd);
    return aff;
}
pub const MAXITEMCNT: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
unsafe fn is_aff_rule(
    mut items: *mut *mut ::core::ffi::c_char,
    mut itemcnt: ::core::ffi::c_int,
    mut rulename: *mut ::core::ffi::c_char,
    mut mincount: ::core::ffi::c_int,
) -> bool {
    return strcmp(*items.offset(0 as ::core::ffi::c_int as isize), rulename)
        == 0 as ::core::ffi::c_int
        && (itemcnt == mincount
            || itemcnt > mincount
                && *(*items.offset(mincount as isize)).offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == '#' as ::core::ffi::c_int);
}
unsafe fn aff_process_flags(mut affile: *mut afffile_T, mut entry: *mut affentry_T) {
    if !(*entry).ae_flags.is_null()
        && ((*affile).af_compforbid != 0 as ::core::ffi::c_uint
            || (*affile).af_comppermit != 0 as ::core::ffi::c_uint)
    {
        let mut p: *mut ::core::ffi::c_char = (*entry).ae_flags;
        while *p as ::core::ffi::c_int != NUL {
            let mut prevp: *mut ::core::ffi::c_char = p;
            let mut flag: ::core::ffi::c_uint = get_affitem((*affile).af_flagtype, &raw mut p);
            if flag == (*affile).af_comppermit || flag == (*affile).af_compforbid {
                memmove(
                    prevp as *mut ::core::ffi::c_void,
                    p as *const ::core::ffi::c_void,
                    strlen(p).wrapping_add(1 as size_t),
                );
                p = prevp;
                if flag == (*affile).af_comppermit {
                    (*entry).ae_comppermit = true_0 as ::core::ffi::c_char;
                } else {
                    (*entry).ae_compforbid = true_0 as ::core::ffi::c_char;
                }
            }
            if (*affile).af_flagtype == AFT_NUM
                && *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
        }
        if *(*entry).ae_flags as ::core::ffi::c_int == NUL {
            (*entry).ae_flags = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    }
}
unsafe fn spell_info_item(mut s: *mut ::core::ffi::c_char) -> bool {
    return strcmp(s, b"NAME\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
        || strcmp(s, b"HOME\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
        || strcmp(s, b"VERSION\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        || strcmp(s, b"AUTHOR\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        || strcmp(s, b"EMAIL\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
        || strcmp(s, b"COPYRIGHT\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int;
}
unsafe fn affitem2flag(
    mut flagtype: ::core::ffi::c_int,
    mut item: *mut ::core::ffi::c_char,
    mut fname: *mut ::core::ffi::c_char,
    mut lnum: ::core::ffi::c_int,
) -> ::core::ffi::c_uint {
    let mut p: *mut ::core::ffi::c_char = item;
    let mut res: ::core::ffi::c_uint = get_affitem(flagtype, &raw mut p);
    if res == 0 as ::core::ffi::c_uint {
        if flagtype == AFT_NUM {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Flag is not a number in %s line %d: %s\0".as_ptr()
                    as *const ::core::ffi::c_char),
                fname,
                lnum,
                item,
            );
        } else {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Illegal flag in %s line %d: %s\0".as_ptr() as *const ::core::ffi::c_char),
                fname,
                lnum,
                item,
            );
        }
    }
    if *p as ::core::ffi::c_int != NUL {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(e_affname.get()),
            fname,
            lnum,
            item,
        );
        return 0 as ::core::ffi::c_uint;
    }
    return res;
}
unsafe fn get_affitem(
    mut flagtype: ::core::ffi::c_int,
    mut pp: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_uint {
    let mut res: ::core::ffi::c_int = 0;
    if flagtype == AFT_NUM {
        if !ascii_isdigit(**pp as ::core::ffi::c_int) {
            *pp = (*pp).offset(1);
            return 0 as ::core::ffi::c_uint;
        }
        res = getdigits_int(pp, true_0 != 0, 0 as ::core::ffi::c_int);
        if res == 0 as ::core::ffi::c_int {
            res = ZERO_FLAG;
        }
    } else {
        res = mb_ptr2char_adv(pp as *mut *const ::core::ffi::c_char);
        if flagtype == AFT_LONG
            || flagtype == AFT_CAPLONG
                && res >= 'A' as ::core::ffi::c_int
                && res <= 'Z' as ::core::ffi::c_int
        {
            if **pp as ::core::ffi::c_int == NUL {
                return 0 as ::core::ffi::c_uint;
            }
            res = mb_ptr2char_adv(pp as *mut *const ::core::ffi::c_char)
                + (res << 16 as ::core::ffi::c_int);
        }
    }
    return res as ::core::ffi::c_uint;
}
unsafe fn process_compflags(
    mut spin: *mut spellinfo_T,
    mut aff: *mut afffile_T,
    mut compflags: *mut ::core::ffi::c_char,
) {
    let mut ci: *mut compitem_T = ::core::ptr::null_mut::<compitem_T>();
    let mut id: ::core::ffi::c_int = 0;
    let mut key: [::core::ffi::c_char; 17] = [0; 17];
    let mut len: ::core::ffi::c_int =
        strlen(compflags) as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    if !(*spin).si_compflags.is_null() {
        len += strlen((*spin).si_compflags) as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    }
    let mut p: *mut ::core::ffi::c_char =
        getroom(spin, len as size_t, false_0 != 0) as *mut ::core::ffi::c_char;
    if !(*spin).si_compflags.is_null() {
        strcpy(p, (*spin).si_compflags);
        strcat(p, b"/\0".as_ptr() as *const ::core::ffi::c_char);
    }
    (*spin).si_compflags = p;
    let mut tp: *mut uint8_t = (p as *mut uint8_t).offset(strlen(p) as isize);
    p = compflags;
    while *p as ::core::ffi::c_int != NUL {
        if !vim_strchr(
            b"/?*+[]\0".as_ptr() as *const ::core::ffi::c_char,
            *p as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            let c2rust_fresh38 = p;
            p = p.offset(1);
            let c2rust_fresh39 = tp;
            tp = tp.offset(1);
            *c2rust_fresh39 = *c2rust_fresh38 as uint8_t;
        } else {
            let mut prevp: *mut ::core::ffi::c_char = p;
            let mut flag: ::core::ffi::c_uint = get_affitem((*aff).af_flagtype, &raw mut p);
            if flag != 0 as ::core::ffi::c_uint {
                xmemcpyz(
                    &raw mut key as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                    prevp as *const ::core::ffi::c_void,
                    p.offset_from(prevp) as size_t,
                );
                let mut hi: *mut hashitem_T = hash_find(
                    &raw mut (*aff).af_comp,
                    &raw mut key as *mut ::core::ffi::c_char,
                );
                if !((*hi).hi_key.is_null()
                    || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
                {
                    id = (*((*hi).hi_key as *mut compitem_T)).ci_newID;
                } else {
                    ci = getroom(spin, ::core::mem::size_of::<compitem_T>(), true_0 != 0)
                        as *mut compitem_T;
                    strcpy(
                        &raw mut (*ci).ci_key as *mut ::core::ffi::c_char,
                        &raw mut key as *mut ::core::ffi::c_char,
                    );
                    (*ci).ci_flag = flag;
                    loop {
                        check_renumber(spin);
                        let c2rust_fresh40 = (*spin).si_newcompID;
                        (*spin).si_newcompID = (*spin).si_newcompID - 1;
                        id = c2rust_fresh40;
                        if vim_strchr(b"/?*+[]\\-^\0".as_ptr() as *const ::core::ffi::c_char, id)
                            .is_null()
                        {
                            break;
                        }
                    }
                    (*ci).ci_newID = id;
                    hash_add(
                        &raw mut (*aff).af_comp,
                        &raw mut (*ci).ci_key as *mut ::core::ffi::c_char,
                    );
                }
                let c2rust_fresh41 = tp;
                tp = tp.offset(1);
                *c2rust_fresh41 = id as uint8_t;
            }
            if (*aff).af_flagtype == AFT_NUM
                && *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
        }
    }
    *tp = NUL as uint8_t;
}
unsafe fn check_renumber(mut spin: *mut spellinfo_T) {
    if (*spin).si_newprefID == (*spin).si_newcompID
        && (*spin).si_newcompID < 128 as ::core::ffi::c_int
    {
        (*spin).si_newprefID = 127 as ::core::ffi::c_int;
        (*spin).si_newcompID = 255 as ::core::ffi::c_int;
    }
}
unsafe fn flag_in_afflist(
    mut flagtype: ::core::ffi::c_int,
    mut afflist: *mut ::core::ffi::c_char,
    mut flag: ::core::ffi::c_uint,
) -> bool {
    match flagtype {
        AFT_CHAR => return !vim_strchr(afflist, flag as ::core::ffi::c_int).is_null(),
        AFT_CAPLONG | AFT_LONG => {
            let mut p: *mut ::core::ffi::c_char = afflist;
            while *p as ::core::ffi::c_int != NUL {
                let mut n: ::core::ffi::c_uint =
                    mb_ptr2char_adv(&raw mut p as *mut *const ::core::ffi::c_char)
                        as ::core::ffi::c_uint;
                if (flagtype == AFT_LONG
                    || n >= 'A' as ::core::ffi::c_uint && n <= 'Z' as ::core::ffi::c_uint)
                    && *p as ::core::ffi::c_int != NUL
                {
                    n = (mb_ptr2char_adv(&raw mut p as *mut *const ::core::ffi::c_char)
                        as ::core::ffi::c_uint)
                        .wrapping_add(n << 16 as ::core::ffi::c_int);
                }
                if n == flag {
                    return true_0 != 0;
                }
            }
        }
        AFT_NUM => {
            let mut p_0: *mut ::core::ffi::c_char = afflist;
            while *p_0 as ::core::ffi::c_int != NUL {
                let mut digits: ::core::ffi::c_int =
                    getdigits_int(&raw mut p_0, true_0 != 0, 0 as ::core::ffi::c_int);
                '_c2rust_label: {
                    if digits >= 0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                            b"digits >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/spellfile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            2954 as ::core::ffi::c_uint,
                            b"_Bool flag_in_afflist(int, char *, unsigned int)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                let mut n_0: ::core::ffi::c_uint = digits as ::core::ffi::c_uint;
                if n_0 == 0 as ::core::ffi::c_uint {
                    n_0 = ZERO_FLAG as ::core::ffi::c_uint;
                }
                if n_0 == flag {
                    return true_0 != 0;
                }
                if *p_0 as ::core::ffi::c_int != NUL {
                    p_0 = p_0.offset(1);
                }
            }
        }
        _ => {}
    }
    return false_0 != 0;
}
unsafe fn aff_check_number(
    mut spinval: ::core::ffi::c_int,
    mut affval: ::core::ffi::c_int,
    mut name: *mut ::core::ffi::c_char,
) {
    if spinval != 0 as ::core::ffi::c_int && spinval != affval {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(
                b"%s value differs from what is used in another .aff file\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            name,
        );
    }
}
unsafe fn aff_check_string(
    mut spinval: *mut ::core::ffi::c_char,
    mut affval: *mut ::core::ffi::c_char,
    mut name: *mut ::core::ffi::c_char,
) {
    if !spinval.is_null() && strcmp(spinval, affval) != 0 as ::core::ffi::c_int {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(
                b"%s value differs from what is used in another .aff file\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            name,
        );
    }
}
unsafe fn str_equal(mut s1: *mut ::core::ffi::c_char, mut s2: *mut ::core::ffi::c_char) -> bool {
    if s1.is_null() || s2.is_null() {
        return s1 == s2;
    }
    return strcmp(s1, s2) == 0 as ::core::ffi::c_int;
}
unsafe fn add_fromto(
    mut spin: *mut spellinfo_T,
    mut gap: *mut garray_T,
    mut from: *mut ::core::ffi::c_char,
    mut to: *mut ::core::ffi::c_char,
) {
    let mut word: [::core::ffi::c_char; 254] = [0; 254];
    let mut ftp: *mut fromto_T =
        ga_append_via_ptr(gap, ::core::mem::size_of::<fromto_T>()) as *mut fromto_T;
    spell_casefold(
        curwin.get(),
        from,
        strlen(from) as ::core::ffi::c_int,
        &raw mut word as *mut ::core::ffi::c_char,
        MAXWLEN as ::core::ffi::c_int,
    );
    (*ftp).ft_from = getroom_save(spin, &raw mut word as *mut ::core::ffi::c_char);
    spell_casefold(
        curwin.get(),
        to,
        strlen(to) as ::core::ffi::c_int,
        &raw mut word as *mut ::core::ffi::c_char,
        MAXWLEN as ::core::ffi::c_int,
    );
    (*ftp).ft_to = getroom_save(spin, &raw mut word as *mut ::core::ffi::c_char);
}
unsafe fn sal_to_bool(mut s: *mut ::core::ffi::c_char) -> bool {
    return strcmp(s, b"1\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
        || strcmp(s, b"true\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int;
}
unsafe fn spell_free_aff(mut aff: *mut afffile_T) {
    xfree((*aff).af_enc as *mut ::core::ffi::c_void);
    let mut ht: *mut hashtab_T = &raw mut (*aff).af_pref;
    loop {
        let mut todo: ::core::ffi::c_int = (*ht).ht_used as ::core::ffi::c_int;
        let mut hi: *mut hashitem_T = (*ht).ht_array;
        while todo > 0 as ::core::ffi::c_int {
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                todo -= 1;
                let mut ah: *mut affheader_T = (*hi).hi_key as *mut affheader_T;
                let mut ae: *mut affentry_T = (*ah).ah_first;
                while !ae.is_null() {
                    vim_regfree((*ae).ae_prog);
                    ae = (*ae).ae_next;
                }
            }
            hi = hi.offset(1);
        }
        if ht == &raw mut (*aff).af_suff {
            break;
        }
        ht = &raw mut (*aff).af_suff;
    }
    hash_clear(&raw mut (*aff).af_pref);
    hash_clear(&raw mut (*aff).af_suff);
    hash_clear(&raw mut (*aff).af_comp);
}
pub unsafe fn ex_mkspell(mut eap: *mut exarg_T) {
    let mut fcount: ::core::ffi::c_int = 0;
    let mut fnames: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut ascii: bool = false_0 != 0;
    if strncmp(
        arg,
        b"-ascii\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        ascii = true_0 != 0;
        arg = skipwhite(arg.offset(6 as ::core::ffi::c_int as isize));
    }
    if get_arglist_exp(arg, &raw mut fcount, &raw mut fnames, false_0 != 0) != OK {
        return;
    }
    mkspell(fcount, fnames, ascii, (*eap).forceit != 0, false_0 != 0);
    FreeWild(fcount, fnames);
}
unsafe fn mkspell(
    mut fcount: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut ascii: bool,
    mut over_write: bool,
    mut added_word: bool,
) {
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut afile: [*mut afffile_T; 8] = [::core::ptr::null_mut::<afffile_T>(); 8];
    let mut error: bool = false_0 != 0;
    let mut spin: spellinfo_T = spellinfo_T {
        si_foldroot: ::core::ptr::null_mut::<wordnode_T>(),
        si_foldwcount: 0,
        si_keeproot: ::core::ptr::null_mut::<wordnode_T>(),
        si_keepwcount: 0,
        si_prefroot: ::core::ptr::null_mut::<wordnode_T>(),
        si_sugtree: 0,
        si_arena: SpellArena::new(),
        si_did_emsg: 0,
        si_compress_cnt: 0,
        si_first_free: ::core::ptr::null_mut::<wordnode_T>(),
        si_free_count: 0,
        si_spellbuf: ::core::ptr::null_mut::<buf_T>(),
        si_ascii: 0,
        si_add: 0,
        si_clear_chartab: 0,
        si_region: 0,
        si_conv: vimconv_T {
            vc_type: 0,
            vc_factor: 0,
            vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            vc_fail: false,
        },
        si_memtot: 0,
        si_verbose: 0,
        si_msg_count: 0,
        si_info: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        si_region_count: 0,
        si_region_name: [0; 17],
        si_rep: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        si_repsal: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        si_sal: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        si_sofofr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        si_sofoto: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        si_nosugfile: 0,
        si_nosplitsugs: 0,
        si_nocompoundsugs: 0,
        si_followup: 0,
        si_collapse: 0,
        si_commonwords: hashtab_T {
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
        },
        si_sugtime: 0,
        si_rem_accents: 0,
        si_map: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        si_midword: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        si_compmax: 0,
        si_compminlen: 0,
        si_compsylmax: 0,
        si_compoptions: 0,
        si_comppat: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        si_compflags: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        si_nobreak: 0,
        si_syllable: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        si_prefcond: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        si_newprefID: 0,
        si_newcompID: 0,
    };
    memset(
        &raw mut spin as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<spellinfo_T>(),
    );
    spin.si_verbose = !added_word as ::core::ffi::c_int;
    spin.si_ascii = ascii as ::core::ffi::c_int;
    spin.si_followup = true_0;
    spin.si_rem_accents = true_0;
    ga_init(
        &raw mut spin.si_rep,
        ::core::mem::size_of::<fromto_T>() as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
    );
    ga_init(
        &raw mut spin.si_repsal,
        ::core::mem::size_of::<fromto_T>() as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
    );
    ga_init(
        &raw mut spin.si_sal,
        ::core::mem::size_of::<fromto_T>() as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
    );
    ga_init(
        &raw mut spin.si_map,
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        100 as ::core::ffi::c_int,
    );
    ga_init(
        &raw mut spin.si_comppat,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
    );
    ga_init(
        &raw mut spin.si_prefcond,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        50 as ::core::ffi::c_int,
    );
    hash_init(&raw mut spin.si_commonwords);
    spin.si_newcompID = 127 as ::core::ffi::c_int;
    let mut innames: *mut *mut ::core::ffi::c_char = fnames.offset(
        (if fcount == 1 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        }) as isize,
    );
    let mut incount: ::core::ffi::c_int = fcount - 1 as ::core::ffi::c_int;
    let mut wfname: *mut ::core::ffi::c_char =
        xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    if fcount >= 1 as ::core::ffi::c_int {
        let mut len: ::core::ffi::c_int =
            strlen(*fnames.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
        if fcount == 1 as ::core::ffi::c_int
            && len > 4 as ::core::ffi::c_int
            && strcmp(
                (*fnames.offset(0 as ::core::ffi::c_int as isize))
                    .offset(len as isize)
                    .offset(-(4 as ::core::ffi::c_int as isize)),
                b".add\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            incount = 1 as ::core::ffi::c_int;
            vim_snprintf(
                wfname,
                MAXPATHL as size_t,
                b"%s.spl\0".as_ptr() as *const ::core::ffi::c_char,
                *fnames.offset(0 as ::core::ffi::c_int as isize),
            );
        } else if fcount == 1 as ::core::ffi::c_int {
            incount = 1 as ::core::ffi::c_int;
            vim_snprintf(
                wfname,
                MAXPATHL as size_t,
                SPL_FNAME_TMPL.as_ptr(),
                *fnames.offset(0 as ::core::ffi::c_int as isize),
                if spin.si_ascii != 0 {
                    b"ascii\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    spell_enc() as *const ::core::ffi::c_char
                },
            );
        } else if len > 4 as ::core::ffi::c_int
            && strcmp(
                (*fnames.offset(0 as ::core::ffi::c_int as isize))
                    .offset(len as isize)
                    .offset(-(4 as ::core::ffi::c_int as isize)),
                b".spl\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            xstrlcpy(
                wfname,
                *fnames.offset(0 as ::core::ffi::c_int as isize),
                MAXPATHL as size_t,
            );
        } else {
            vim_snprintf(
                wfname,
                MAXPATHL as size_t,
                SPL_FNAME_TMPL.as_ptr(),
                *fnames.offset(0 as ::core::ffi::c_int as isize),
                if spin.si_ascii != 0 {
                    b"ascii\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    spell_enc() as *const ::core::ffi::c_char
                },
            );
        }
        if !strstr(path_tail(wfname), SPL_FNAME_ASCII.as_ptr()).is_null() {
            spin.si_ascii = true_0;
        }
        if !strstr(path_tail(wfname), SPL_FNAME_ADD.as_ptr()).is_null() {
            spin.si_add = true_0;
        }
    }
    '_theend: {
        if incount <= 0 as ::core::ffi::c_int {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        } else if !vim_strchr(path_tail(wfname), '_' as ::core::ffi::c_int).is_null() {
            emsg(gettext(
                b"E751: Output file name must not have region name\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
        } else if incount > MAXREGIONS as ::core::ffi::c_int {
            semsg(
                gettext(b"E754: Only up to %d regions supported\0".as_ptr()
                    as *const ::core::ffi::c_char),
                MAXREGIONS as ::core::ffi::c_int,
            );
        } else if !over_write && os_path_exists(wfname) as ::core::ffi::c_int != 0 {
            emsg(gettext(&raw const e_exists as *const ::core::ffi::c_char));
        } else if os_isdir(wfname) {
            semsg(
                gettext(&raw const e_isadir2 as *const ::core::ffi::c_char),
                wfname,
            );
        } else {
            fname = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < incount {
                afile[i as usize] = ::core::ptr::null_mut::<afffile_T>();
                if incount > 1 as ::core::ffi::c_int {
                    let mut len_0: ::core::ffi::c_int =
                        strlen(*innames.offset(i as isize)) as ::core::ffi::c_int;
                    if strlen(path_tail(*innames.offset(i as isize))) < 5 as size_t
                        || *(*innames.offset(i as isize))
                            .offset((len_0 - 3 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            != '_' as ::core::ffi::c_int
                    {
                        semsg(
                            gettext(b"E755: Invalid region in %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            *innames.offset(i as isize),
                        );
                        break '_theend;
                    } else {
                        spin.si_region_name[(i * 2 as ::core::ffi::c_int) as usize] =
                            (if (*(*innames.offset(i as isize))
                                .offset((len_0 - 2 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int)
                                < 'A' as ::core::ffi::c_int
                                || *(*innames.offset(i as isize))
                                    .offset((len_0 - 2 as ::core::ffi::c_int) as isize)
                                    as ::core::ffi::c_int
                                    > 'Z' as ::core::ffi::c_int
                            {
                                *(*innames.offset(i as isize))
                                    .offset((len_0 - 2 as ::core::ffi::c_int) as isize)
                                    as ::core::ffi::c_int
                            } else {
                                *(*innames.offset(i as isize))
                                    .offset((len_0 - 2 as ::core::ffi::c_int) as isize)
                                    as ::core::ffi::c_int
                                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                            }) as uint8_t as ::core::ffi::c_char;
                        spin.si_region_name
                            [(i * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize] =
                            (if (*(*innames.offset(i as isize))
                                .offset((len_0 - 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int)
                                < 'A' as ::core::ffi::c_int
                                || *(*innames.offset(i as isize))
                                    .offset((len_0 - 1 as ::core::ffi::c_int) as isize)
                                    as ::core::ffi::c_int
                                    > 'Z' as ::core::ffi::c_int
                            {
                                *(*innames.offset(i as isize))
                                    .offset((len_0 - 1 as ::core::ffi::c_int) as isize)
                                    as ::core::ffi::c_int
                            } else {
                                *(*innames.offset(i as isize))
                                    .offset((len_0 - 1 as ::core::ffi::c_int) as isize)
                                    as ::core::ffi::c_int
                                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                            }) as uint8_t as ::core::ffi::c_char;
                    }
                }
                i += 1;
            }
            spin.si_region_count = incount;
            spin.si_foldroot = wordtree_alloc(&mut spin);
            spin.si_keeproot = wordtree_alloc(&mut spin);
            spin.si_prefroot = wordtree_alloc(&mut spin);
            if spin.si_add == 0 {
                spin.si_clear_chartab = true_0;
            }
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < incount && !error {
                spin.si_conv.vc_type = CONV_NONE as ::core::ffi::c_int;
                spin.si_region = (1 as ::core::ffi::c_int) << i_0;
                vim_snprintf(
                    fname,
                    MAXPATHL as size_t,
                    b"%s.aff\0".as_ptr() as *const ::core::ffi::c_char,
                    *innames.offset(i_0 as isize),
                );
                if os_path_exists(fname) {
                    afile[i_0 as usize] = spell_read_aff(&raw mut spin, fname) as *mut afffile_T;
                    if afile[i_0 as usize].is_null() {
                        error = true_0 != 0;
                    } else {
                        vim_snprintf(
                            fname,
                            MAXPATHL as size_t,
                            b"%s.dic\0".as_ptr() as *const ::core::ffi::c_char,
                            *innames.offset(i_0 as isize),
                        );
                        if spell_read_dic(
                            &raw mut spin,
                            fname,
                            afile[i_0 as usize] as *mut afffile_T,
                        ) == FAIL
                        {
                            error = true_0 != 0;
                        }
                    }
                } else if spell_read_wordfile(&raw mut spin, *innames.offset(i_0 as isize)) == FAIL
                {
                    error = true_0 != 0;
                }
                convert_setup(
                    &raw mut spin.si_conv,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                );
                i_0 += 1;
            }
            if !spin.si_compflags.is_null() && spin.si_nobreak as ::core::ffi::c_int != 0 {
                msg(
                    gettext(
                        b"Warning: both compounding and NOBREAK specified\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    0 as ::core::ffi::c_int,
                );
            }
            if !error && !got_int.get() {
                spell_message(&raw mut spin, gettext(MSG_COMPRESSING.as_ptr()));
                let root = spin.si_foldroot;
                wordtree_compress(&mut spin, root, c"case-folded");
                let root = spin.si_keeproot;
                wordtree_compress(&mut spin, root, c"keep-case");
                let root = spin.si_prefroot;
                wordtree_compress(&mut spin, root, c"prefixes");
            }
            if !error && !got_int.get() {
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    gettext(b"Writing spell file %s...\0".as_ptr() as *const ::core::ffi::c_char),
                    wfname,
                );
                spell_message(&raw mut spin, IObuff.ptr() as *mut ::core::ffi::c_char);
                error = write_vim_spell(&mut spin, wfname) == FAIL;
                spell_message(
                    &raw mut spin,
                    gettext(b"Done!\0".as_ptr() as *const ::core::ffi::c_char),
                );
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    gettext(b"Estimated runtime memory use: %d bytes\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    spin.si_memtot,
                );
                spell_message(&raw mut spin, IObuff.ptr() as *mut ::core::ffi::c_char);
                if !error {
                    spell_reload_one(wfname, added_word);
                }
            }
            ga_clear(&raw mut spin.si_rep);
            ga_clear(&raw mut spin.si_repsal);
            ga_clear(&raw mut spin.si_sal);
            ga_clear(&raw mut spin.si_map);
            ga_clear(&raw mut spin.si_comppat);
            ga_clear(&raw mut spin.si_prefcond);
            hash_clear_all(&raw mut spin.si_commonwords, 0 as ::core::ffi::c_uint);
            let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_1 < incount {
                if !afile[i_1 as usize].is_null() {
                    spell_free_aff(afile[i_1 as usize] as *mut afffile_T);
                }
                i_1 += 1;
            }
            spin.si_arena.clear();
            if spin.si_sugtime != 0 as time_t && !error && !got_int.get() {
                spell_make_sugfile(&mut spin, wfname);
            }
        }
    }
    xfree(fname as *mut ::core::ffi::c_void);
    xfree(wfname as *mut ::core::ffi::c_void);
}
unsafe fn spell_message(mut spin: *const spellinfo_T, mut str: *mut ::core::ffi::c_char) {
    if (*spin).si_verbose != 0 || p_verbose.get() > 2 as OptInt {
        if (*spin).si_verbose == 0 {
            verbose_enter();
        }
        msg(str, 0 as ::core::ffi::c_int);
        ui_flush();
        if (*spin).si_verbose == 0 {
            verbose_leave();
        }
    }
}
pub unsafe fn ex_spell(mut eap: *mut exarg_T) {
    spell_add_word(
        (*eap).arg,
        strlen((*eap).arg) as ::core::ffi::c_int,
        (if (*eap).cmdidx as ::core::ffi::c_int == CMD_spellwrong as ::core::ffi::c_int {
            SPELL_ADD_BAD as ::core::ffi::c_int
        } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_spellrare as ::core::ffi::c_int {
            SPELL_ADD_RARE as ::core::ffi::c_int
        } else {
            SPELL_ADD_GOOD as ::core::ffi::c_int
        }) as SpellAddType,
        if (*eap).forceit != 0 {
            0 as ::core::ffi::c_int
        } else {
            (*eap).line2 as ::core::ffi::c_int
        },
        (*eap).cmdidx as ::core::ffi::c_int == CMD_spellundo as ::core::ffi::c_int,
    );
}
pub unsafe fn spell_add_word(
    mut word: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut what: SpellAddType,
    mut idx: ::core::ffi::c_int,
    mut undo: bool,
) {
    let mut fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut new_spf: bool = false_0 != 0;
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fnamebuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line: [::core::ffi::c_char; 508] = [0; 508];
    let mut spf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !valid_spell_word(word, word.offset(len as isize)) {
        emsg(gettext(e_illegal_character_in_word.get()));
        return;
    }
    if idx == 0 as ::core::ffi::c_int {
        if (*int_wordlist.ptr()).is_null() {
            int_wordlist.set(vim_tempname());
            if (*int_wordlist.ptr()).is_null() {
                return;
            }
        }
        fname = int_wordlist.get();
    } else {
        let mut i: ::core::ffi::c_int = 0;
        if *(*(*curwin.get()).w_s).b_p_spf as ::core::ffi::c_int == NUL {
            init_spellfile();
            new_spf = true_0 != 0;
        }
        if *(*(*curwin.get()).w_s).b_p_spf as ::core::ffi::c_int == NUL {
            semsg(
                gettext(&raw const e_notset as *const ::core::ffi::c_char),
                b"spellfile\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        fnamebuf = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        spf = (*(*curwin.get()).w_s).b_p_spf;
        i = 1 as ::core::ffi::c_int;
        while *spf as ::core::ffi::c_int != NUL {
            copy_option_part(
                &raw mut spf,
                fnamebuf,
                MAXPATHL as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            if i == idx {
                break;
            }
            if *spf as ::core::ffi::c_int == NUL {
                semsg(
                    gettext(b"E765: 'spellfile' does not have %d entries\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    idx,
                );
                xfree(fnamebuf as *mut ::core::ffi::c_void);
                return;
            }
            i += 1;
        }
        buf = buflist_findname_exp(fnamebuf);
        if !buf.is_null() && (*buf).b_ml.ml_mfp.is_null() {
            buf = ::core::ptr::null_mut::<buf_T>();
        }
        if !buf.is_null() && bufIsChanged(buf) as ::core::ffi::c_int != 0 {
            emsg(gettext(
                &raw const e_bufloaded as *const ::core::ffi::c_char,
            ));
            xfree(fnamebuf as *mut ::core::ffi::c_void);
            return;
        }
        fname = fnamebuf;
    }
    if what as ::core::ffi::c_uint == SPELL_ADD_BAD as ::core::ffi::c_int as ::core::ffi::c_uint
        || undo as ::core::ffi::c_int != 0
    {
        let mut fpos_next: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut fpos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        fd = os_fopen(fname, b"r\0".as_ptr() as *const ::core::ffi::c_char);
        if !fd.is_null() {
            while !vim_fgets(
                &raw mut line as *mut ::core::ffi::c_char,
                MAXWLEN as ::core::ffi::c_int * 2 as ::core::ffi::c_int,
                fd,
            ) {
                fpos = fpos_next;
                fpos_next = ftell(fd) as ::core::ffi::c_int;
                if fpos_next < 0 as ::core::ffi::c_int {
                    break;
                }
                if !(strncmp(
                    word,
                    &raw mut line as *mut ::core::ffi::c_char,
                    len as size_t,
                ) == 0 as ::core::ffi::c_int
                    && (line[len as usize] as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                        || (line[len as usize] as uint8_t as ::core::ffi::c_int)
                            < ' ' as ::core::ffi::c_int))
                {
                    continue;
                }
                fclose(fd);
                fd = os_fopen(fname, b"r+\0".as_ptr() as *const ::core::ffi::c_char);
                if fd.is_null() {
                    break;
                }
                if fseek(fd, fpos as ::core::ffi::c_long, SEEK_SET) == 0 as ::core::ffi::c_int {
                    fputc('#' as ::core::ffi::c_int, fd);
                    if undo {
                        home_replace(
                            ::core::ptr::null::<buf_T>(),
                            fname,
                            NameBuff.ptr() as *mut ::core::ffi::c_char,
                            MAXPATHL as size_t,
                            true_0 != 0,
                        );
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(b"Word '%.*s' removed from %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            len,
                            word,
                            NameBuff.ptr() as *mut ::core::ffi::c_char,
                        );
                    }
                }
                if fseek(fd, fpos_next as ::core::ffi::c_long, SEEK_SET) == 0 as ::core::ffi::c_int
                {
                    continue;
                }
                semsg(
                    b"%s: %s\0".as_ptr() as *const ::core::ffi::c_char,
                    gettext(b"Seek error in spellfile\0".as_ptr() as *const ::core::ffi::c_char),
                    strerror(*__errno_location()),
                );
                break;
            }
            if !fd.is_null() {
                fclose(fd);
            }
        }
    }
    if !undo {
        fd = os_fopen(fname, b"a\0".as_ptr() as *const ::core::ffi::c_char);
        if fd.is_null() && new_spf as ::core::ffi::c_int != 0 {
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if !dir_of_file_exists(fname) && {
                p = path_tail_with_sep(fname);
                p != fname
            } {
                let mut c: ::core::ffi::c_char = *p;
                *p = NUL as ::core::ffi::c_char;
                os_mkdir(fname, 0o755 as int32_t);
                *p = c;
                fd = os_fopen(fname, b"a\0".as_ptr() as *const ::core::ffi::c_char);
            }
        }
        if fd.is_null() {
            semsg(
                gettext(&raw const e_notopen as *const ::core::ffi::c_char),
                fname,
            );
        } else {
            if what as ::core::ffi::c_uint
                == SPELL_ADD_BAD as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                fprintf(
                    fd,
                    b"%.*s/!\n\0".as_ptr() as *const ::core::ffi::c_char,
                    len,
                    word,
                );
            } else if what as ::core::ffi::c_uint
                == SPELL_ADD_RARE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                fprintf(
                    fd,
                    b"%.*s/?\n\0".as_ptr() as *const ::core::ffi::c_char,
                    len,
                    word,
                );
            } else {
                fprintf(
                    fd,
                    b"%.*s\n\0".as_ptr() as *const ::core::ffi::c_char,
                    len,
                    word,
                );
            }
            fclose(fd);
            home_replace(
                ::core::ptr::null::<buf_T>(),
                fname,
                NameBuff.ptr() as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                true_0 != 0,
            );
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Word '%.*s' added to %s\0".as_ptr() as *const ::core::ffi::c_char),
                len,
                word,
                NameBuff.ptr() as *mut ::core::ffi::c_char,
            );
        }
    }
    if !fd.is_null() {
        mkspell(
            1 as ::core::ffi::c_int,
            &raw mut fname,
            false_0 != 0,
            true_0 != 0,
            true_0 != 0,
        );
        if !buf.is_null() {
            buf_reload(buf, (*buf).b_orig_mode, false_0 != 0);
        }
        redraw_all_later(UPD_SOME_VALID);
    }
    xfree(fnamebuf as *mut ::core::ffi::c_void);
}
unsafe fn init_spellfile() {
    let mut lend: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut aspath: bool = false_0 != 0;
    let mut lstart: *mut ::core::ffi::c_char = (*curbuf.get()).b_s.b_p_spl;
    if *(*(*curwin.get()).w_s).b_p_spl as ::core::ffi::c_int == NUL
        || (*(*curwin.get()).w_s).b_langp.ga_len <= 0 as ::core::ffi::c_int
    {
        return;
    }
    lend = (*(*curwin.get()).w_s).b_p_spl;
    while *lend as ::core::ffi::c_int != NUL
        && vim_strchr(
            b",._\0".as_ptr() as *const ::core::ffi::c_char,
            *lend as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
    {
        if vim_ispathsep(*lend as ::core::ffi::c_int) {
            aspath = true_0 != 0;
            lstart = lend.offset(1 as ::core::ffi::c_int as isize);
        }
        lend = lend.offset(1);
    }
    let mut buf: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    let mut buf_len: size_t = MAXPATHL as size_t;
    if !aspath {
        let mut xdg_path: *mut ::core::ffi::c_char = get_xdg_home(kXDGDataHome);
        xstrlcpy(buf, xdg_path, buf_len);
        xfree(xdg_path as *mut ::core::ffi::c_void);
        xstrlcat(
            buf,
            b"/site/spell\0".as_ptr() as *const ::core::ffi::c_char,
            buf_len,
        );
        let mut failed_dir: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        if os_mkdir_recurse(
            buf,
            0o755 as int32_t,
            &raw mut failed_dir,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        ) != 0 as ::core::ffi::c_int
        {
            xfree(buf as *mut ::core::ffi::c_void);
            xfree(failed_dir as *mut ::core::ffi::c_void);
            return;
        }
    } else {
        if lend.offset_from((*curbuf.get()).b_s.b_p_spl) as size_t >= buf_len {
            xfree(buf as *mut ::core::ffi::c_void);
            return;
        }
        xmemcpyz(
            buf as *mut ::core::ffi::c_void,
            (*curbuf.get()).b_s.b_p_spl as *const ::core::ffi::c_void,
            lend.offset_from((*curbuf.get()).b_s.b_p_spl) as size_t,
        );
    }
    vim_snprintf(
        buf.offset(strlen(buf) as isize),
        buf_len.wrapping_sub(strlen(buf)),
        b"/%.*s\0".as_ptr() as *const ::core::ffi::c_char,
        lend.offset_from(lstart) as ::core::ffi::c_int,
        lstart,
    );
    let mut fname: *mut ::core::ffi::c_char = (*(*((*(*curwin.get()).w_s).b_langp.ga_data
        as *mut langp_T)
        .offset(0 as ::core::ffi::c_int as isize))
    .lp_slang)
        .sl_fname;
    let mut enc_suffix: *const ::core::ffi::c_char = if !fname.is_null()
        && !strstr(
            path_tail(fname),
            b".ascii.\0".as_ptr() as *const ::core::ffi::c_char,
        )
        .is_null()
    {
        b"ascii\0".as_ptr() as *const ::core::ffi::c_char
    } else {
        spell_enc() as *const ::core::ffi::c_char
    };
    vim_snprintf(
        buf.offset(strlen(buf) as isize),
        buf_len.wrapping_sub(strlen(buf)),
        b".%s.add\0".as_ptr() as *const ::core::ffi::c_char,
        enc_suffix,
    );
    set_option_value_give_err(
        kOptSpellfile,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(buf),
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
    );
    xfree(buf as *mut ::core::ffi::c_void);
}
unsafe fn set_spell_finish(mut new_st: *mut spelltab_T) -> ::core::ffi::c_int {
    if did_set_spelltab.get() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 256 as ::core::ffi::c_int {
            if (*spelltab.ptr()).st_isw[i as usize] as ::core::ffi::c_int
                != (*new_st).st_isw[i as usize] as ::core::ffi::c_int
                || (*spelltab.ptr()).st_isu[i as usize] as ::core::ffi::c_int
                    != (*new_st).st_isu[i as usize] as ::core::ffi::c_int
                || (*spelltab.ptr()).st_fold[i as usize] as ::core::ffi::c_int
                    != (*new_st).st_fold[i as usize] as ::core::ffi::c_int
                || (*spelltab.ptr()).st_upper[i as usize] as ::core::ffi::c_int
                    != (*new_st).st_upper[i as usize] as ::core::ffi::c_int
            {
                emsg(gettext(
                    b"E763: Word characters differ between spell files\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                return FAIL;
            }
            i += 1;
        }
    } else {
        spelltab.set(*new_st);
        did_set_spelltab.set(true_0 != 0);
    }
    return OK;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RE_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const RE_STRICT: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
