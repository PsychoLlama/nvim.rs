use crate::semsg_c;
use crate::src::nvim::arglist::get_arglist_exp;
use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::charset::{getdigits_int, skipwhite};
use crate::src::nvim::garray::{ga_clear, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::{hash_clear_all, hash_init};
use crate::src::nvim::main::{IObuff, e_exists, e_invarg, e_isadir2, got_int, p_msm, p_verbose};
use crate::src::nvim::mbyte::convert_setup;
use crate::src::nvim::memory::{xfree, xmalloc, xstrlcpy};
use crate::src::nvim::message::{emsg, msg, verbose_enter, verbose_leave};
use crate::src::nvim::os::fs::{os_isdir, os_path_exists};
use crate::src::nvim::os::libc::{gettext, memset, strcmp, strlen, strncmp, strstr};
use crate::src::nvim::path::{FreeWild, path_tail};
use crate::src::nvim::spell::{did_set_spelltab, spell_enc, spelltab};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr};
use crate::src::nvim::types::{
    CMD_spellrare, CMD_spellundo, CMD_spellwrong, CONV_NONE, OptInt, OptValType, SpellAddType,
    XDGVarType, buf_T, etype_T, exarg_T, file_comparison, fromto_T, garray_T, hashitem_T,
    hashtab_T, regprog_T, size_t, spelltab_T, time_t, uint8_t, vimconv_T,
};
use crate::src::nvim::ui::ui_flush;
mod add;
mod aff;
mod affix;
mod dic;
mod flags;
mod read;
mod sections;
mod sugfile;
mod tables;
mod wordfile;
mod wordtree;
mod write;
use crate::src::nvim::regexp::{vim_regcomp, vim_regexec_prog, vim_regfree};
pub use add::spell_add_word;
use aff::spell_read_aff;
use dic::spell_read_dic;
use flags::spell_free_aff;
use read::spell_reload_one;
pub use read::{spell_load_file, suggest_load_files};
use sugfile::spell_make_sugfile;
use wordfile::spell_read_wordfile;
use wordtree::{
    MSG_COMPRESSING, SpellArena, set_compression_limits, wordnode_T, wordtree_alloc,
    wordtree_compress,
};
use write::write_vim_spell;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const kOptValTypeString: OptValType = 2;
pub const ETYPE_SPELL: etype_T = 9;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const OPT_LOCAL: C2Rust_Unnamed_18 = 2;
pub const kXDGDataHome: XDGVarType = 1;
pub const kEqualFiles: file_comparison = 1;
pub use crate::src::nvim::spell::MAXWLEN;
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
    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1_usize);
pub const VIMSPELLVERSION: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
pub const SNF_REQUIRED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const COMPOUND_MAX_LEN: ::core::ffi::c_int = 100000 as ::core::ffi::c_int;
static e_spell_trunc: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(c"E758: Truncated spell file".as_ptr());
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
    GlobalCell::new(c"E1280: Illegal character in word".as_ptr());
static e_afftrailing: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(c"Trailing text in %s line %d: %s".as_ptr());
static e_affname: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(c"Affix name too long in %s line %d: %s".as_ptr());
pub const MAXLINELEN: ::core::ffi::c_int = 500 as ::core::ffi::c_int;
pub const AFT_CHAR: ::core::ffi::c_int = 0;
pub const AFT_LONG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const AFT_CAPLONG: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const AFT_NUM: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const AH_KEY_LEN: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const PFX_FLAGS: ::core::ffi::c_int = -256 as ::core::ffi::c_int;
pub const CONDIT_COMB: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CONDIT_CFIX: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CONDIT_SUF: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const CONDIT_AFF: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
#[inline(always)]
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
pub unsafe fn ex_mkspell(mut eap: *mut exarg_T) {
    let mut fcount: ::core::ffi::c_int = 0;
    let mut fnames: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut ascii: bool = false_0 != 0;
    if strncmp(arg, c"-ascii".as_ptr(), 6 as size_t) == 0 as ::core::ffi::c_int {
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
                c".add".as_ptr(),
            ) == 0 as ::core::ffi::c_int
        {
            incount = 1 as ::core::ffi::c_int;
            vim_snprintf(
                wfname,
                MAXPATHL as size_t,
                c"%s.spl".as_ptr(),
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
                    c"ascii".as_ptr()
                } else {
                    spell_enc() as *const ::core::ffi::c_char
                },
            );
        } else if len > 4 as ::core::ffi::c_int
            && strcmp(
                (*fnames.offset(0 as ::core::ffi::c_int as isize))
                    .offset(len as isize)
                    .offset(-(4 as ::core::ffi::c_int as isize)),
                c".spl".as_ptr(),
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
                    c"ascii".as_ptr()
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
                c"E751: Output file name must not have region name".as_ptr(),
            ));
        } else if incount > MAXREGIONS as ::core::ffi::c_int {
            semsg_c!(
                gettext(c"E754: Only up to %d regions supported".as_ptr()),
                MAXREGIONS as ::core::ffi::c_int,
            );
        } else if !over_write && os_path_exists(wfname) as ::core::ffi::c_int != 0 {
            emsg(gettext(&raw const e_exists as *const ::core::ffi::c_char));
        } else if os_isdir(wfname) {
            semsg_c!(
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
                        semsg_c!(
                            gettext(c"E755: Invalid region in %s".as_ptr()),
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
                spin.si_conv.vc_type = CONV_NONE;
                spin.si_region = (1 as ::core::ffi::c_int) << i_0;
                vim_snprintf(
                    fname,
                    MAXPATHL as size_t,
                    c"%s.aff".as_ptr(),
                    *innames.offset(i_0 as isize),
                );
                if os_path_exists(fname) {
                    afile[i_0 as usize] = spell_read_aff(&raw mut spin, fname);
                    if afile[i_0 as usize].is_null() {
                        error = true_0 != 0;
                    } else {
                        vim_snprintf(
                            fname,
                            MAXPATHL as size_t,
                            c"%s.dic".as_ptr(),
                            *innames.offset(i_0 as isize),
                        );
                        if spell_read_dic(&raw mut spin, fname, afile[i_0 as usize]) == FAIL {
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
                    gettext(c"Warning: both compounding and NOBREAK specified".as_ptr()),
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
                    gettext(c"Writing spell file %s...".as_ptr()),
                    wfname,
                );
                spell_message(&raw mut spin, IObuff.ptr() as *mut ::core::ffi::c_char);
                error = write_vim_spell(&mut spin, wfname) == FAIL;
                spell_message(&raw mut spin, gettext(c"Done!".as_ptr()));
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    gettext(c"Estimated runtime memory use: %d bytes".as_ptr()),
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
                    spell_free_aff(afile[i_1 as usize]);
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
                    c"E763: Word characters differ between spell files".as_ptr(),
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
