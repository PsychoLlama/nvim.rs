#![deny(unsafe_op_in_unsafe_fn)]

use crate::arglist::get_arglist_exp;
use crate::ascii::ascii_isdigit;
use crate::charset::{getdigits_int, skipwhite};
use crate::cstr;
use crate::garray::{ga_clear, ga_init};
use crate::global_cell::GlobalCell;
use crate::hashtab::{hash_clear_all, hash_init};
use crate::main::{e_exists, e_invarg, got_int, p_msm, p_verbose};
use crate::mbyte::convert_setup;
use crate::memory::{xfree, xmalloc, xstrlcpy};
use crate::message::{emsg, msg, verbose_enter, verbose_leave};
use crate::message_fmt::c_str;
use crate::os::cshim::{gettext, strncmp, strstr};
use crate::os::fs::{os_isdir, os_path_exists};
use crate::path::{free_wild, path_tail};
use crate::semsg;
use crate::spell::{WordFlags, did_set_spelltab, spell_enc, spelltab};
use crate::strings::{vim_snprintf, vim_strchr};
use crate::types::{
    CMD_spellrare, CMD_spellundo, CMD_spellwrong, CONV_NONE, FAIL, Failed, MAXPATHL, NUL, OK,
    OptInt, OptValType, SPL_FNAME_TMPL, SpellAddType, XDGVarType, buf_T, etype_T, exarg_T,
    file_comparison, fromto_T, garray_T, hashitem_T, hashtab_T, regprog_T, size_t, spelltab_T,
    time_t, vimconv_T,
};
use crate::ui::ui_flush;
use ::libc::{strcmp, strlen};
use core::ffi::CStr;
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
use crate::regexp::{vim_regcomp, vim_regexec_prog, vim_regfree};
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
pub const _ISdigit: ::core::ffi::c_uint = 2048;
pub const kOptValTypeString: OptValType = 2;
pub const ETYPE_SPELL: etype_T = 9;
pub const kXDGDataHome: XDGVarType = 1;
pub const kEqualFiles: file_comparison = 1;
pub use crate::spell::MAXWLEN;
pub const MAXREGIONS: ::core::ffi::c_uint = 8;
pub const WF_KEEPCAP: WordFlags = 128;
pub const WF_FIXCAP: WordFlags = 64;
pub const WF_AFX: WordFlags = 32;
pub const WF_BANNED: WordFlags = 16;
pub const WF_RARE: WordFlags = 8;
pub const WF_REGION: WordFlags = 1;
pub const WF_NOCOMPAFT: WordFlags = 8192;
pub const WF_NOCOMPBEF: WordFlags = 4096;
pub const WF_COMPROOT: WordFlags = 2048;
pub const WF_NOSUGGEST: WordFlags = 1024;
pub const WF_NEEDCOMP: WordFlags = 512;
pub const WF_HAS_AFF: WordFlags = 256;
pub const WFP_COMPFORBID: ::core::ffi::c_uint = 16;
pub const WFP_COMPPERMIT: ::core::ffi::c_uint = 8;
pub const WFP_UP: ::core::ffi::c_uint = 4;
pub const WFP_NC: ::core::ffi::c_uint = 2;
pub const COMP_CHECKTRIPLE: ::core::ffi::c_uint = 8;
pub const COMP_CHECKCASE: ::core::ffi::c_uint = 4;
pub const COMP_CHECKREP: ::core::ffi::c_uint = 2;
pub const COMP_CHECKDUP: ::core::ffi::c_uint = 1;
pub const SP_OTHERERROR: ::core::ffi::c_int = -3;
pub const SP_FORMERROR: ::core::ffi::c_int = -2;
pub const SP_TRUNCERROR: ::core::ffi::c_int = -1;
pub const SPELL_ADD_RARE: SpellAddType = 2;
pub const SPELL_ADD_BAD: SpellAddType = 1;
pub const SPELL_ADD_GOOD: SpellAddType = 0;
pub const BY_FLAGS2: ::core::ffi::c_uint = 3;
pub const BY_FLAGS: ::core::ffi::c_uint = 2;
pub const BY_INDEX: ::core::ffi::c_uint = 1;
pub const BY_NOFLAGS: ::core::ffi::c_uint = 0;
pub const BY_SPECIAL: ::core::ffi::c_uint = 3;
pub const SN_SYLLABLE: ::core::ffi::c_uint = 9;
pub const SN_NOBREAK: ::core::ffi::c_uint = 10;
pub const SN_COMPOUND: ::core::ffi::c_uint = 8;
pub const SN_NOCOMPOUNDSUGS: ::core::ffi::c_uint = 16;
pub const SN_NOSPLITSUGS: ::core::ffi::c_uint = 14;
pub const SN_SUGFILE: ::core::ffi::c_uint = 11;
pub const SN_WORDS: ::core::ffi::c_uint = 13;
pub const SN_MAP: ::core::ffi::c_uint = 7;
pub const SN_SOFO: ::core::ffi::c_uint = 6;
pub const SAL_REM_ACCENTS: ::core::ffi::c_uint = 4;
pub const SAL_COLLAPSE: ::core::ffi::c_uint = 2;
pub const SAL_F0LLOWUP: ::core::ffi::c_uint = 1;
pub const SN_SAL: ::core::ffi::c_uint = 5;
pub const SN_REPSAL: ::core::ffi::c_uint = 12;
pub const SN_REP: ::core::ffi::c_uint = 4;
pub const SN_PREFCOND: ::core::ffi::c_uint = 3;
pub const SN_MIDWORD: ::core::ffi::c_uint = 2;
pub const CF_UPPER: ::core::ffi::c_uint = 2;
pub const CF_WORD: ::core::ffi::c_uint = 1;
pub const SN_CHARFLAGS: ::core::ffi::c_uint = 1;
pub const SN_REGION: ::core::ffi::c_uint = 0;
pub const SN_INFO: ::core::ffi::c_uint = 15;
pub const SN_END: ::core::ffi::c_uint = 255;
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
/// What one `.aff` file declared.
///
/// Parse-time only: `:mkspell` reads the affix and dictionary files into
/// this and the word trees, and [`write::write_vim_spell`] then emits the
/// `.spl` a field at a time. None of the types below is ever written to a
/// file as a struct image, none is named by `tools/ffigen/unit-cdefs.h` or
/// the ABI ledger, and none crosses an `extern` boundary — so none of them
/// carries `repr(C)`, and the compiler is free to lay them out.
#[derive(Clone)]
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
/// One `PFX`/`SFX` line: what to chop, what to add, and when.
#[derive(Copy, Clone)]
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
/// All the `PFX`/`SFX` blocks that share one affix name.
///
/// The `af_pref`/`af_suff` tables key on [`ah_key`](Self::ah_key), which
/// the header owns, so the table's key pointer points *into* the header;
/// [`Self::key`] and [`Self::of_key`] are the two directions.
#[derive(Copy, Clone)]
pub struct affheader_T {
    pub ah_key: [::core::ffi::c_char; 17],
    pub ah_flag: ::core::ffi::c_uint,
    pub ah_newID: ::core::ffi::c_int,
    pub ah_combine: ::core::ffi::c_int,
    pub ah_follows: ::core::ffi::c_int,
    pub ah_first: *mut affentry_T,
}

impl affheader_T {
    /// This header's affix name, as the NUL-terminated string `af_pref` and
    /// `af_suff` key on.
    fn key(this: *mut Self) -> *mut ::core::ffi::c_char {
        // SAFETY: the offset lands inside the header; nothing is read.
        unsafe { (&raw mut (*this).ah_key).cast() }
    }

    /// The header a [`key`](Self::key) came from.
    ///
    /// # Safety
    ///
    /// `key` must be a pointer [`key`](Self::key) returned for a header
    /// that is still live.
    unsafe fn of_key(key: *mut ::core::ffi::c_char) -> *mut Self {
        // SAFETY: the caller promises the pointer names a live header's key
        // field, so stepping back over the field's offset lands on it.
        unsafe { key.byte_sub(::core::mem::offset_of!(Self, ah_key)).cast() }
    }
}

/// One `COMPOUNDFLAG` value and the internal id standing in for it.
///
/// `af_comp` keys on [`ci_key`](Self::ci_key) exactly as `af_pref` keys on
/// an affix name; see [`affheader_T`].
#[derive(Copy, Clone)]
pub struct compitem_T {
    pub ci_key: [::core::ffi::c_char; 17],
    pub ci_flag: ::core::ffi::c_uint,
    pub ci_newID: ::core::ffi::c_int,
}

impl compitem_T {
    /// This item's compound flag, as the NUL-terminated string `af_comp`
    /// keys on.
    fn key(this: *mut Self) -> *mut ::core::ffi::c_char {
        // SAFETY: the offset lands inside the item; nothing is read.
        unsafe { (&raw mut (*this).ci_key).cast() }
    }

    /// The item a [`key`](Self::key) came from.
    ///
    /// # Safety
    ///
    /// `key` must be a pointer [`key`](Self::key) returned for an item that
    /// is still live.
    unsafe fn of_key(key: *mut ::core::ffi::c_char) -> *mut Self {
        // SAFETY: as for `affheader_T::of_key`.
        unsafe { key.byte_sub(::core::mem::offset_of!(Self, ci_key)).cast() }
    }
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const EOF: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SPL_FNAME_ADD: &::core::ffi::CStr = c".add.";
pub const SPL_FNAME_ASCII: &::core::ffi::CStr = c".ascii.";
pub const VIMSUGMAGIC: &::core::ffi::CStr = c"VIMsug";
pub const VIMSUGMAGICL: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const VIMSUGVERSION: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ZERO_FLAG: ::core::ffi::c_int = 65009 as ::core::ffi::c_int;
pub const VIMSPELLMAGIC: &::core::ffi::CStr = c"VIMspell";
pub const VIMSPELLMAGICL: usize =
    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1_usize);
pub const VIMSPELLVERSION: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
pub const SNF_REQUIRED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const COMPOUND_MAX_LEN: ::core::ffi::c_int = 100000 as ::core::ffi::c_int;
static e_spell_trunc: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(c"E758: Truncated spell file".as_ptr());
static e_duplicate_char_in_map_entry: &::core::ffi::CStr = c"E783: Duplicate char in MAP entry";
static e_illegal_character_in_word: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(c"E1280: Illegal character in word".as_ptr());
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
impl spellinfo_T {
    /// A spell file under construction, with nothing read yet.
    ///
    /// The C listed an initialiser and then `memset` the whole struct to
    /// zero over the top of it, so every field really does start at zero;
    /// only the four values [`mkspell`] assigns straight afterwards differ.
    /// Writing it out is also what lets `si_arena` own a `Vec` at all: an
    /// empty `Vec`'s pointer is dangling-but-aligned rather than null, so
    /// `memset`ing over one leaves it invalid.
    fn new() -> Self {
        const NO_GARRAY: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut(),
        };
        Self {
            si_foldroot: ::core::ptr::null_mut(),
            si_foldwcount: 0,
            si_keeproot: ::core::ptr::null_mut(),
            si_keepwcount: 0,
            si_prefroot: ::core::ptr::null_mut(),
            si_sugtree: 0,
            si_arena: SpellArena::new(),
            si_did_emsg: 0,
            si_compress_cnt: 0,
            si_first_free: ::core::ptr::null_mut(),
            si_free_count: 0,
            si_spellbuf: ::core::ptr::null_mut(),
            si_ascii: 0,
            si_add: 0,
            si_clear_chartab: 0,
            si_region: 0,
            si_conv: vimconv_T {
                vc_type: 0,
                vc_factor: 0,
                vc_fd: ::core::ptr::null_mut(),
                vc_fail: false,
            },
            si_memtot: 0,
            si_verbose: 0,
            si_msg_count: 0,
            si_info: ::core::ptr::null_mut(),
            si_region_count: 0,
            si_region_name: [0; 17],
            si_rep: NO_GARRAY,
            si_repsal: NO_GARRAY,
            si_sal: NO_GARRAY,
            si_sofofr: ::core::ptr::null_mut(),
            si_sofoto: ::core::ptr::null_mut(),
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
                ht_array: ::core::ptr::null_mut(),
                ht_smallarray: [hashitem_T {
                    hi_hash: 0,
                    hi_key: ::core::ptr::null_mut(),
                }; 16],
            },
            si_sugtime: 0,
            si_rem_accents: 0,
            si_map: NO_GARRAY,
            si_midword: ::core::ptr::null_mut(),
            si_compmax: 0,
            si_compminlen: 0,
            si_compsylmax: 0,
            si_compoptions: 0,
            si_comppat: NO_GARRAY,
            si_compflags: ::core::ptr::null_mut(),
            si_nobreak: 0,
            si_syllable: ::core::ptr::null_mut(),
            si_prefcond: NO_GARRAY,
            si_newprefID: 0,
            si_newcompID: 0,
        }
    }
}

/// Validate `'mkspellmem'` and install the compression limits it names.
///
/// The option is three comma-separated numbers: the arena size at which the
/// first compression run happens, how much more has to be taken before the
/// next one, and how many words may be added in between. The first two are
/// given in megabytes and scaled to arena blocks here; the third is in
/// thousands of words.
pub fn spell_check_msm() -> Result<(), Failed> {
    // SAFETY: `p_msm` holds the option's value, a NUL-terminated string.
    let Some((start, incr, added)) = (unsafe { parse_mkspellmem(p_msm.get()) }) else {
        return Err(Failed);
    };

    let block = wordtree::block_size();
    let start = start * 10 / (block / 102);
    let incr = incr * 102 / (block / 10);
    let added = added * 1024;
    if start == 0 || incr == 0 || added == 0 || incr > start {
        return Err(Failed);
    }
    set_compression_limits(start, incr, added);
    Ok(())
}

/// The three unscaled numbers of a `'mkspellmem'` value, or `None` when the
/// string is not exactly `<digits>,<digits>,<digits>`.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
unsafe fn parse_mkspellmem(
    p: *mut ::core::ffi::c_char,
) -> Option<(::core::ffi::c_int, ::core::ffi::c_int, ::core::ffi::c_int)> {
    // SAFETY: the caller promises a terminated string, and each step below
    // stops at the first byte that is not a digit — the NUL at the latest.
    let mut p = p;
    let start = unsafe { digits_then(&mut p, ',' as ::core::ffi::c_int) }?;
    let incr = unsafe { digits_then(&mut p, ',' as ::core::ffi::c_int) }?;
    let added = unsafe { digits_then(&mut p, NUL) }?;
    Some((start, incr, added))
}

/// Read a decimal number that `sep` must follow, stepping `p` past both.
/// A `sep` of [`NUL`] means "and that must be the end of the string", where
/// there is nothing to step over.
///
/// # Safety
///
/// `p` must point into a NUL-terminated string.
unsafe fn digits_then(
    p: &mut *mut ::core::ffi::c_char,
    sep: ::core::ffi::c_int,
) -> Option<::core::ffi::c_int> {
    // SAFETY: the caller promises the string; `getdigits_int` advances `p`
    // over the digits it consumed and no further.
    if !ascii_isdigit(unsafe { **p } as ::core::ffi::c_int) {
        return None;
    }
    let n = unsafe { getdigits_int(&raw mut *p, true, 0) };
    if unsafe { **p } as ::core::ffi::c_int != sep {
        return None;
    }
    if sep != NUL {
        *p = unsafe { p.add(1) };
    }
    Some(n)
}

/// `:mkspell[!] [-ascii] {outfile} {infile} ...`
///
/// # Safety
///
/// `eap` must be a live excommand.
pub unsafe fn ex_mkspell(eap: *mut exarg_T) {
    // SAFETY: the caller promises the excommand; `get_arglist_exp` fills in
    // the count and the vector, which `free_wild` then releases.
    let mut arg = unsafe { (*eap).arg };
    let mut ascii = false;
    if unsafe { strncmp(arg, c"-ascii".as_ptr(), 6) } == 0 {
        ascii = true;
        arg = unsafe { skipwhite(arg.add(6)) };
    }

    let mut fcount = 0;
    let mut fnames = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    if unsafe { get_arglist_exp(arg, &raw mut fcount, &raw mut fnames, false) }.is_err() {
        return;
    }
    unsafe { mkspell(fcount, fnames, ascii, (*eap).forceit != 0, false) };
    unsafe { free_wild(fcount, fnames) };
}

/// Read `.aff`/`.dic` pairs or word lists and write the `.spl` they make.
///
/// `fnames[0]` names the output; the rest are inputs, one per region. A
/// single name is both the input stem and, with a suffix, the output.
///
/// # Safety
///
/// `fnames` must hold `fcount` NUL-terminated paths.
unsafe fn mkspell(
    fcount: ::core::ffi::c_int,
    fnames: *mut *mut ::core::ffi::c_char,
    ascii: bool,
    over_write: bool,
    added_word: bool,
) {
    let mut fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut afile = [::core::ptr::null_mut::<afffile_T>(); MAXREGIONS as usize];
    let mut spin = spellinfo_T::new();
    spin.si_verbose = !added_word as ::core::ffi::c_int;
    spin.si_ascii = ascii as ::core::ffi::c_int;
    spin.si_followup = 1;
    spin.si_rem_accents = 1;

    // SAFETY: the caller promises the paths; `wfname` and `fname` are
    // MAXPATHL buffers, which is the bound every writer below is given.
    let entry_size = ::core::mem::size_of::<fromto_T>() as ::core::ffi::c_int;
    unsafe { ga_init(&raw mut spin.si_rep, entry_size, 20) };
    unsafe { ga_init(&raw mut spin.si_repsal, entry_size, 20) };
    unsafe { ga_init(&raw mut spin.si_sal, entry_size, 20) };
    unsafe { ga_init(&raw mut spin.si_map, 1, 100) };
    let ptr_size = ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int;
    unsafe { ga_init(&raw mut spin.si_comppat, ptr_size, 20) };
    unsafe { ga_init(&raw mut spin.si_prefcond, ptr_size, 50) };
    unsafe { hash_init(&raw mut spin.si_commonwords) };
    spin.si_newcompID = 127;

    // With one name it is both the input stem and the output; with
    // more, the first is the output and the rest are the inputs.
    let innames = unsafe { fnames.offset(if fcount == 1 { 0 } else { 1 }) };
    let mut incount = fcount - 1;
    let wfname = unsafe { xmalloc(MAXPATHL as size_t) }.cast::<::core::ffi::c_char>();
    if fcount >= 1 {
        incount = unsafe { output_name(wfname, fnames, fcount, spin.si_ascii != 0, incount) };
        if !unsafe { strstr(path_tail(wfname), SPL_FNAME_ASCII.as_ptr()) }.is_null() {
            spin.si_ascii = 1;
        }
        if !unsafe { strstr(path_tail(wfname), SPL_FNAME_ADD.as_ptr()) }.is_null() {
            spin.si_add = 1;
        }
    }

    '_theend: {
        if !unsafe { output_is_writable(wfname, incount, over_write) } {
            break '_theend;
        }
        fname = unsafe { xmalloc(MAXPATHL as size_t) }.cast::<::core::ffi::c_char>();
        if !unsafe { read_region_names(&mut spin, innames, incount) } {
            break '_theend;
        }
        spin.si_region_count = incount;
        spin.si_foldroot = wordtree_alloc(&mut spin);
        spin.si_keeproot = wordtree_alloc(&mut spin);
        spin.si_prefroot = wordtree_alloc(&mut spin);
        if spin.si_add == 0 {
            spin.si_clear_chartab = 1;
        }

        let mut error = unsafe { read_inputs(&mut spin, innames, incount, fname, &mut afile) };
        if !spin.si_compflags.is_null() && spin.si_nobreak != 0 {
            let text = c"Warning: both compounding and NOBREAK specified";
            msg(gettext(text), 0);
        }
        if !error && !got_int.get() {
            spell_message(&spin, MSG_COMPRESSING);
            let root = spin.si_foldroot;
            unsafe { wordtree_compress(&mut spin, root, c"case-folded") };
            let root = spin.si_keeproot;
            unsafe { wordtree_compress(&mut spin, root, c"keep-case") };
            let root = spin.si_prefroot;
            unsafe { wordtree_compress(&mut spin, root, c"prefixes") };
        }
        if !error && !got_int.get() {
            let name = unsafe { CStr::from_ptr(wfname) }.to_string_lossy();
            spell_message_fmt(&spin, format_args!("Writing spell file {name}..."));
            error = unsafe { write_vim_spell(&mut spin, wfname) }.is_err();
            spell_message(&spin, c"Done!");
            let used = spin.si_memtot;
            spell_message_fmt(
                &spin,
                format_args!("Estimated runtime memory use: {used} bytes"),
            );
            if !error {
                unsafe { spell_reload_one(wfname, added_word) };
            }
        }

        unsafe { ga_clear(&raw mut spin.si_rep) };
        unsafe { ga_clear(&raw mut spin.si_repsal) };
        unsafe { ga_clear(&raw mut spin.si_sal) };
        unsafe { ga_clear(&raw mut spin.si_map) };
        unsafe { ga_clear(&raw mut spin.si_comppat) };
        unsafe { ga_clear(&raw mut spin.si_prefcond) };
        unsafe { hash_clear_all(&raw mut spin.si_commonwords, 0) };
        for aff in afile.iter().take(incount as usize) {
            if !aff.is_null() {
                unsafe { spell_free_aff(*aff) };
            }
        }
        // The `.sug` pass reads the file back and builds its own tree,
        // so the arena goes first.
        spin.si_arena.clear();
        if spin.si_sugtime != 0 && !error && !got_int.get() {
            unsafe { spell_make_sugfile(&mut spin, wfname) };
        }
    }

    unsafe { xfree(fname.cast()) };
    unsafe { xfree(wfname.cast()) };
}

/// Write the output path into `wfname` and say how many inputs there are.
///
/// A lone `foo.add` writes `foo.add.spl`; any other lone name is a stem the
/// encoding is appended to; a first name already ending in `.spl` is taken
/// as it stands. `default_incount` is what to report when the first name
/// was only the output.
///
/// # Safety
///
/// `wfname` must have room for [`MAXPATHL`] bytes and `fnames` hold
/// `fcount` NUL-terminated paths, at least one.
unsafe fn output_name(
    wfname: *mut ::core::ffi::c_char,
    fnames: *mut *mut ::core::ffi::c_char,
    fcount: ::core::ffi::c_int,
    ascii: bool,
    default_incount: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    // SAFETY: the caller promises the paths and the buffer's size, which is
    // the bound both writers below are given.
    let first = unsafe { *fnames };
    let len = unsafe { strlen(first) };
    let ends_with = |ext: &::core::ffi::CStr| {
        len > 4 && unsafe { strcmp(first.add(len).sub(4), ext.as_ptr()) } == 0
    };
    let enc = if ascii {
        c"ascii".as_ptr()
    } else {
        unsafe { spell_enc() }.cast::<::core::ffi::c_char>()
    };

    if fcount == 1 {
        if ends_with(c".add") {
            unsafe { vim_snprintf(wfname, MAXPATHL as size_t, c"%s.spl".as_ptr(), first) };
        } else {
            let fmt = SPL_FNAME_TMPL.as_ptr();
            unsafe { vim_snprintf(wfname, MAXPATHL as size_t, fmt, first, enc) };
        }
        return 1;
    }
    if ends_with(c".spl") {
        unsafe { xstrlcpy(wfname, first, MAXPATHL as size_t) };
    } else {
        let fmt = SPL_FNAME_TMPL.as_ptr();
        unsafe { vim_snprintf(wfname, MAXPATHL as size_t, fmt, first, enc) };
    }
    default_incount
}

/// Can `:mkspell` write here? Reports why not, if not.
///
/// # Safety
///
/// `wfname` must be a NUL-terminated path.
unsafe fn output_is_writable(
    wfname: *mut ::core::ffi::c_char,
    incount: ::core::ffi::c_int,
    over_write: bool,
) -> bool {
    // SAFETY: the caller promises the path.
    if incount <= 0 {
        emsg(gettext(e_invarg));
    } else if !unsafe { vim_strchr(path_tail(wfname), '_' as ::core::ffi::c_int) }.is_null() {
        emsg(gettext(c"E751: Output file name must not have region name"));
    } else if incount > MAXREGIONS as ::core::ffi::c_int {
        semsg!(
            "E754: Only up to {} regions supported",
            MAXREGIONS as ::core::ffi::c_int
        );
    } else if !over_write && unsafe { os_path_exists(wfname) } {
        emsg(gettext(e_exists));
    } else if unsafe { os_isdir(wfname) } {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let wfname = unsafe { c_str(wfname) };
        semsg!("E17: \"{wfname}\" is a directory");
    } else {
        return true;
    }
    false
}

/// Fill in `si_region_name` from the `_xy` each input name ends with.
///
/// A single input covers every region and carries no suffix. Returns false,
/// having reported the name, when one of several does not carry one.
///
/// # Safety
///
/// `innames` must hold `incount` NUL-terminated paths.
unsafe fn read_region_names(
    spin: &mut spellinfo_T,
    innames: *mut *mut ::core::ffi::c_char,
    incount: ::core::ffi::c_int,
) -> bool {
    if incount <= 1 {
        return true;
    }
    // SAFETY: the caller promises the paths; the two bytes read below are
    // the last two before the terminator of a name at least five long.
    for i in 0..incount as usize {
        let name = unsafe { *innames.add(i) };
        let len = unsafe { strlen(name) };
        if unsafe { strlen(path_tail(name)) } < 5
            || unsafe { *name.add(len - 3) } != b'_' as ::core::ffi::c_char
        {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let name = unsafe { c_str(name) };
            semsg!("E755: Invalid region in {name}");
            return false;
        }
        spin.si_region_name[i * 2] = to_lower_ascii(unsafe { *name.add(len - 2) });
        spin.si_region_name[i * 2 + 1] = to_lower_ascii(unsafe { *name.add(len - 1) });
    }
    true
}

/// Lower-case one ASCII letter, leaving every other byte — including the
/// high half, where `c_char` is negative — alone.
fn to_lower_ascii(c: ::core::ffi::c_char) -> ::core::ffi::c_char {
    if c < b'A' as ::core::ffi::c_char || c > b'Z' as ::core::ffi::c_char {
        c
    } else {
        c + (b'a' - b'A') as ::core::ffi::c_char
    }
}

/// Read every input into `spin`: an `.aff`/`.dic` pair where an `.aff`
/// exists, a plain word list otherwise. Returns whether one of them failed,
/// which stops the rest.
///
/// # Safety
///
/// `innames` must hold `incount` NUL-terminated paths, `fname` must have
/// room for [`MAXPATHL`] bytes, and `afile` at least `incount` slots.
unsafe fn read_inputs(
    spin: &mut spellinfo_T,
    innames: *mut *mut ::core::ffi::c_char,
    incount: ::core::ffi::c_int,
    fname: *mut ::core::ffi::c_char,
    afile: &mut [*mut afffile_T],
) -> bool {
    // SAFETY: the caller promises the paths and the buffer's size, which is
    // the bound `vim_snprintf` is given.
    for (i, aff) in afile.iter_mut().enumerate().take(incount as usize) {
        spin.si_conv.vc_type = CONV_NONE;
        spin.si_region = 1 << i;
        let stem = unsafe { *innames.add(i) };
        unsafe { vim_snprintf(fname, MAXPATHL as size_t, c"%s.aff".as_ptr(), stem) };

        let failed = if unsafe { os_path_exists(fname) } {
            *aff = unsafe { spell_read_aff(spin, fname) };
            aff.is_null() || {
                unsafe { vim_snprintf(fname, MAXPATHL as size_t, c"%s.dic".as_ptr(), stem) };
                unsafe { spell_read_dic(spin, fname, *aff).is_err() }
            }
        } else {
            unsafe { spell_read_wordfile(spin, stem).is_err() }
        };
        let none = ::core::ptr::null_mut();
        let _ = unsafe { convert_setup(&raw mut spin.si_conv, none, none) };
        if failed {
            return true;
        }
    }
    false
}

/// Show `text` while `:mkspell` runs, quietly unless it was asked to be
/// verbose or `'verbose'` is high enough to want it anyway.
fn spell_message(spin: &spellinfo_T, text: &CStr) {
    if spin.si_verbose == 0 && p_verbose.get() <= 2 as OptInt {
        return;
    }
    let quiet = spin.si_verbose == 0;
    // SAFETY: `text` is NUL-terminated and `msg` copies what it keeps.
    if quiet {
        unsafe { verbose_enter() };
    }
    msg(text, 0);
    unsafe { ui_flush() };
    if quiet {
        unsafe { verbose_leave() };
    }
}

/// [`spell_message`] with a compile-checked format string.
///
/// Every progress line upstream formats goes through the shared `IObuff`,
/// which `msg` -- and the autocommands it can run -- may write in the middle
/// of the report. The message is built here and owned until it is shown.
pub(super) fn spell_message_fmt(spin: &spellinfo_T, args: core::fmt::Arguments<'_>) {
    if spin.si_verbose == 0 && p_verbose.get() <= 2 as OptInt {
        return;
    }
    spell_message(spin, &cstr::owned(args.to_string().as_bytes()));
}

/// `:spellgood`, `:spellwrong`, `:spellrare` and their `:spell*undo` forms.
///
/// # Safety
///
/// `eap` must be a live excommand.
pub unsafe fn ex_spell(eap: *mut exarg_T) {
    // SAFETY: the caller promises the excommand.
    let (cmdidx, forceit, line2, arg) =
        unsafe { ((*eap).cmdidx, (*eap).forceit, (*eap).line2, (*eap).arg) };

    let kind = if cmdidx == CMD_spellwrong {
        SPELL_ADD_BAD
    } else if cmdidx == CMD_spellrare {
        SPELL_ADD_RARE
    } else {
        SPELL_ADD_GOOD
    };
    // `:N spellgood` files the word in the Nth 'spellfile'; `!` means the
    // internal word list instead.
    let which = if forceit != 0 {
        0
    } else {
        line2 as ::core::ffi::c_int
    };

    // SAFETY: `arg` is the excommand's NUL-terminated argument.
    let len = unsafe { strlen(arg) } as ::core::ffi::c_int;
    let undo = cmdidx == CMD_spellundo;
    unsafe { spell_add_word(arg, len, kind as SpellAddType, which, undo) };
}

/// Adopt `new_st` as the word character table, or check that it agrees with
/// the one already in force.
///
/// The table is global, so two spell files that disagree about which bytes
/// are word characters cannot be loaded together.
fn set_spell_finish(new_st: &spelltab_T) -> Result<(), Failed> {
    if !did_set_spelltab.get() {
        spelltab.set(*new_st);
        did_set_spelltab.set(true);
        return Ok(());
    }
    let agrees = spelltab.with(|st| {
        (0..256).all(|i| {
            st.st_isw[i] == new_st.st_isw[i]
                && st.st_isu[i] == new_st.st_isu[i]
                && st.st_fold[i] == new_st.st_fold[i]
                && st.st_upper[i] == new_st.st_upper[i]
        })
    });
    if !agrees {
        emsg(gettext(c"E763: Word characters differ between spell files"));
        return Err(Failed);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The affix tables store a pointer *into* the header, so the way back
    /// is the field's offset — which is why dropping `repr(C)` from these
    /// parse-time structs costs nothing.
    #[test]
    fn an_affix_key_names_the_header_it_came_from() {
        let mut ah = affheader_T {
            ah_key: [0; 17],
            ah_flag: 0,
            ah_newID: 0,
            ah_combine: 0,
            ah_follows: 0,
            ah_first: ::core::ptr::null_mut(),
        };
        let at = &raw mut ah;
        let key = affheader_T::key(at);
        assert_eq!(key, (&raw mut ah.ah_key).cast());
        // SAFETY: `key` is this header's key field and the header is alive.
        assert_eq!(unsafe { affheader_T::of_key(key) }, at);
    }

    #[test]
    fn a_compound_key_names_the_item_it_came_from() {
        let mut ci = compitem_T {
            ci_key: [0; 17],
            ci_flag: 0,
            ci_newID: 0,
        };
        let at = &raw mut ci;
        let key = compitem_T::key(at);
        assert_eq!(key, (&raw mut ci.ci_key).cast());
        // SAFETY: `key` is this item's key field and the item is alive.
        assert_eq!(unsafe { compitem_T::of_key(key) }, at);
    }
}
