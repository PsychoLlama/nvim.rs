use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::change::inserted_bytes;
use crate::src::nvim::charset::{getdigits_int, rl_mirror_ascii, skiptowhite, skipwhite};
use crate::src::nvim::cursor::{get_cursor_line_len, get_cursor_line_ptr};
use crate::src::nvim::eval::typval::tv_list_unref;
use crate::src::nvim::eval::vars::{eval_spell_expr, get_spellword};
use crate::src::nvim::fileio::vim_fgets;
use crate::src::nvim::garray::{ga_clear, ga_grow, ga_init};
use crate::src::nvim::getchar::{
    AppendCharToRedobuff, AppendToRedobuff, AppendToRedobuffLit, ResetRedobuff, beep_flush, vgetc,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::{hash_clear_all, hash_init};
use crate::src::nvim::input::prompt_for_input;
use crate::src::nvim::main::{
    IObuff, Rows, VIsual, VIsual_active, cmdline_row, cmdmsg_rl, curbuf, curwin, e_no_spell,
    e_notopen, got_int, lines_left, mouse_row, msg_col, msg_row, msg_scroll, p_sps, p_verbose,
};
use crate::src::nvim::mbyte::{mb_isupper, utf_head_off, utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memline::ml_replace;
use crate::src::nvim::memory::{xfree, xmalloc, xmemcpyz, xstrdup, xstrlcpy};
use crate::src::nvim::message::{
    emsg, msg, msg_advance, msg_clr_eos, msg_ext_set_kind, msg_putchar, msg_puts, msg_start, semsg,
    smsg,
};
use crate::src::nvim::normal::end_visual_mode;
use crate::src::nvim::option::copy_option_part;
use crate::src::nvim::options::kOptBoFlagSpell;
use crate::src::nvim::os::fs::os_fopen;
use crate::src::nvim::os::input::{line_breakcheck, os_breakcheck};
use crate::src::nvim::os::libc::{
    __assert_fail, atoi, fclose, gettext, memmove, memset, strcasecmp, strcat, strcmp, strcpy,
    strlen, strncmp,
};
use crate::src::nvim::search::FORWARD;
use crate::src::nvim::spell::{
    captype, check_need_cap, make_case_word, parse_spelllang, repl_from, repl_to, spell_casefold,
    spell_check, spell_iswordp_nmw, spell_move_to, spell_soundfold, spelltab,
};
use crate::src::nvim::spellfile::suggest_load_files;
use crate::src::nvim::strings::{vim_snprintf, vim_strchr, xstrnsave};
use crate::src::nvim::types::ui::kUIMessages;
pub use crate::src::nvim::types::{
    __compar_fn_t, __off_t, __off64_t, __time_t, _IO_FILE, _IO_codecvt, _IO_lock_t, _IO_marker,
    _IO_wide_data, AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback,
    Callback_data as C2Rust_Unnamed_4, CallbackType, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_1, ExtmarkUndoObject, FILE, FileID, FloatAnchor,
    FloatRelative, GridView, Intersection, LuaRef, MTKey, MTNode, MTPos, Map_int64_t_int64_t,
    Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MapHash, MarkTree, OptInt, QUEUE,
    ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_11, Terminal,
    Timestamp, UIExtension, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk,
    VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle, Window, alist_T, bhdr_T, blob_T,
    blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, colnr_T, dict_T, dictvar_S, disptick_T,
    extmark_undo_vec_t, fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_2,
    file_buffer_b_wininfo as C2Rust_Unnamed_10, file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    fromto_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T, handle_T,
    hash_T, hashitem_T, hashtab_T, hlf_T, idx_T, infoptr_T, int16_t, int32_t, int64_t, langp_T,
    lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T,
    llpos_T, lpos_T, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T,
    mfdirty_T, mtnode_inner_s, mtnode_s, partial_S, partial_T, pos_T, pos_save_T, proftime_T,
    ptr_t, qf_info_S, qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T,
    salfirst_T, sattr_T, schar_T, scid_T, sctx_T, size_t, slang_S, slang_T, smt_T, spelltab_T,
    syn_state, syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T, synblock_T, synstate_T,
    taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header,
    u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_7, u_header_uh_alt_prev as C2Rust_Unnamed_6,
    u_header_uh_next as C2Rust_Unnamed_9, u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T,
    uint8_t, uint16_t, uint32_t, uint64_t, undo_object, varnumber_T, virt_line, visualinfo_T,
    win_T, window_S, wininfo_S, winopt_T, wline_T, wordcount_T, xfmark_T,
};
use crate::src::nvim::ui::{ui_has, vim_beep};
use crate::src::nvim::undo::u_save_cursor;
pub mod collect;
pub mod score;
pub mod soundalike;
pub mod walk;

pub use crate::src::nvim::spellsuggest::walk::suggest_trie_walk;

/// The spell languages the current window has loaded, in the order
/// `'spelllang'` put them in.
///
/// # Safety
///
/// The current window must have its spell state set up, which it has
/// whenever `'spell'` is on.
pub unsafe fn window_langs<'a>() -> &'a mut [langp_T] {
    // SAFETY: the caller guarantees the window's spell state; an empty
    // garray has a null data pointer, which `from_raw_parts_mut` rejects
    // even at length zero.
    unsafe {
        let gap = &raw const (*(*curwin.get()).w_s).b_langp;
        if (*gap).ga_data.is_null() || (*gap).ga_len <= 0 {
            &mut []
        } else {
            ::core::slice::from_raw_parts_mut(
                (*gap).ga_data as *mut langp_T,
                (*gap).ga_len as usize,
            )
        }
    }
}

use crate::src::nvim::spellsuggest::collect::{
    add_banned, add_suggestion, check_suggestions, cleanup_suggestions, rescore_suggestions,
    score_combine, score_comp_sal,
};
use crate::src::nvim::spellsuggest::soundalike::{
    suggest_try_soundalike, suggest_try_soundalike_finish, suggest_try_soundalike_prep,
};
pub const VAR_LIST: VarType = 4;
pub const HLF_COUNT: hlf_T = 76;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXWLEN: C2Rust_Unnamed_14 = 254;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const WF_CAPMASK: C2Rust_Unnamed_15 = 198;
pub const WF_KEEPCAP: C2Rust_Unnamed_15 = 128;
pub const WF_BANNED: C2Rust_Unnamed_15 = 16;
pub const WF_RARE: C2Rust_Unnamed_15 = 8;
pub const WF_ALLCAP: C2Rust_Unnamed_15 = 4;
pub const WF_ONECAP: C2Rust_Unnamed_15 = 2;
pub const WF_REGION: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const WF_NOSUGGEST: C2Rust_Unnamed_16 = 1024;
pub const WF_NEEDCOMP: C2Rust_Unnamed_16 = 512;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const WF_RAREPFX: C2Rust_Unnamed_17 = 16777216;
pub const SMT_ALL: smt_T = 0;
pub const SPS_BEST: C2Rust_Unnamed_26 = 1;
pub const SPS_DOUBLE: C2Rust_Unnamed_26 = 4;
pub const SPS_FAST: C2Rust_Unnamed_26 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct suginfo_T {
    pub su_ga: garray_T,
    pub su_maxcount: ::core::ffi::c_int,
    pub su_maxscore: ::core::ffi::c_int,
    pub su_sfmaxscore: ::core::ffi::c_int,
    pub su_sga: garray_T,
    pub su_badptr: *mut ::core::ffi::c_char,
    pub su_badlen: ::core::ffi::c_int,
    pub su_badflags: ::core::ffi::c_int,
    pub su_badword: [::core::ffi::c_char; 254],
    pub su_fbadword: [::core::ffi::c_char; 254],
    pub su_sal_badword: [::core::ffi::c_char; 254],
    pub su_banned: hashtab_T,
    pub su_sallang: *mut slang_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct suggest_T {
    pub st_word: *mut ::core::ffi::c_char,
    pub st_wordlen: ::core::ffi::c_int,
    pub st_orglen: ::core::ffi::c_int,
    pub st_score: ::core::ffi::c_int,
    pub st_altscore: ::core::ffi::c_int,
    pub st_salscore: bool,
    pub st_had_bonus: bool,
    pub st_slang: *mut slang_T,
}
pub const SCORE_INS: C2Rust_Unnamed_18 = 96;
pub const SCORE_MAXMAX: C2Rust_Unnamed_22 = 999999;
pub const SCORE_DEL: C2Rust_Unnamed_18 = 94;
pub const SCORE_SWAP: C2Rust_Unnamed_18 = 75;
pub const SCORE_SUBST: C2Rust_Unnamed_18 = 93;
pub const SCORE_SIMILAR: C2Rust_Unnamed_18 = 33;
pub const SCORE_ICASE: C2Rust_Unnamed_18 = 52;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sftword_T {
    pub sft_score: int16_t,
    pub sft_word: [uint8_t; 0],
}
pub const SCORE_REP: C2Rust_Unnamed_18 = 65;
pub const SCORE_SWAP3: C2Rust_Unnamed_18 = 110;
pub const SCORE_INSDUP: C2Rust_Unnamed_18 = 67;
pub const SCORE_DELDUP: C2Rust_Unnamed_18 = 66;
pub const SCORE_DELCOMP: C2Rust_Unnamed_18 = 28;
pub const SCORE_INSCOMP: C2Rust_Unnamed_18 = 30;
pub const SCORE_SUBCOMP: C2Rust_Unnamed_18 = 33;
pub const SCORE_SPLIT: C2Rust_Unnamed_18 = 149;
pub const SCORE_COMMON3: C2Rust_Unnamed_20 = 50;
pub const SCORE_COMMON2: C2Rust_Unnamed_20 = 40;
pub const SCORE_THRES3: C2Rust_Unnamed_20 = 100;
pub const SCORE_COMMON1: C2Rust_Unnamed_20 = 30;
pub const SCORE_THRES2: C2Rust_Unnamed_20 = 10;
pub const SCORE_SPLIT_NO: C2Rust_Unnamed_18 = 249;
pub const SCORE_NONWORD: C2Rust_Unnamed_18 = 103;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct limitscore_T {
    pub badi: ::core::ffi::c_int,
    pub goodi: ::core::ffi::c_int,
    pub score: ::core::ffi::c_int,
}
pub const SCORE_LIMITMAX: C2Rust_Unnamed_22 = 350;
pub const SCORE_REGION: C2Rust_Unnamed_18 = 200;
pub const SCORE_RARE: C2Rust_Unnamed_18 = 180;
pub const SCORE_SFMAX3: C2Rust_Unnamed_21 = 400;
pub const SCORE_SFMAX2: C2Rust_Unnamed_21 = 300;
pub const SCORE_MAXINIT: C2Rust_Unnamed_19 = 350;
pub const SCORE_SFMAX1: C2Rust_Unnamed_21 = 200;
pub const SCORE_FILE: C2Rust_Unnamed_19 = 30;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const WC_KEY_OFF: ::core::ffi::c_ulong = 2 as ::core::ffi::c_ulong;
pub const WF_MIXCAP: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub(crate) static spell_suggest_timeout: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(5000 as ::core::ffi::c_int);
pub(crate) unsafe fn badword_captype(
    mut word: *mut ::core::ffi::c_char,
    mut end: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = captype(word, end);
    if flags & WF_KEEPCAP as ::core::ffi::c_int == 0 {
        return flags;
    }
    let mut l: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut u: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut first: bool = false_0 != 0;
    let mut p: *mut ::core::ffi::c_char = word;
    while p < end {
        let mut c: ::core::ffi::c_int = utf_ptr2char(p);
        if if c >= 128 as ::core::ffi::c_int {
            mb_isupper(c) as ::core::ffi::c_int
        } else {
            (*spelltab.ptr()).st_isu[c as usize] as ::core::ffi::c_int
        } != 0
        {
            u += 1;
            if p == word {
                first = true_0 != 0;
            }
        } else {
            l += 1;
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    if u > l && u > 2 as ::core::ffi::c_int {
        flags |= WF_ALLCAP as ::core::ffi::c_int;
    } else if first {
        flags |= WF_ONECAP as ::core::ffi::c_int;
    }
    if u >= 2 as ::core::ffi::c_int && l >= 2 as ::core::ffi::c_int {
        flags |= WF_MIXCAP;
    }
    return flags;
}
pub static sps_flags: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(SPS_BEST as ::core::ffi::c_int);
static sps_limit: GlobalCell<::core::ffi::c_int> = GlobalCell::new(9999 as ::core::ffi::c_int);
pub unsafe fn spell_check_sps() -> ::core::ffi::c_int {
    let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
    sps_flags.set(0 as ::core::ffi::c_int);
    sps_limit.set(9999 as ::core::ffi::c_int);
    let mut p: *mut ::core::ffi::c_char = p_sps.get();
    while *p as ::core::ffi::c_int != NUL {
        copy_option_part(
            &raw mut p,
            &raw mut buf as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        let mut f: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if ascii_isdigit(*(&raw mut buf as *mut ::core::ffi::c_char) as ::core::ffi::c_int) {
            let mut s: *mut ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
            sps_limit.set(getdigits_int(
                &raw mut s,
                true_0 != 0,
                0 as ::core::ffi::c_int,
            ));
            if *s as ::core::ffi::c_int != NUL && !ascii_isdigit(*s as ::core::ffi::c_int) {
                f = -1 as ::core::ffi::c_int;
            }
        } else if strcmp(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"best\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            f = SPS_BEST as ::core::ffi::c_int;
        } else if strcmp(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"fast\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            f = SPS_FAST as ::core::ffi::c_int;
        } else if strcmp(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"double\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            f = SPS_DOUBLE as ::core::ffi::c_int;
        } else if strncmp(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"expr:\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) != 0 as ::core::ffi::c_int
            && strncmp(
                &raw mut buf as *mut ::core::ffi::c_char,
                b"file:\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) != 0 as ::core::ffi::c_int
            && (strncmp(
                &raw mut buf as *mut ::core::ffi::c_char,
                b"timeout:\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) != 0 as ::core::ffi::c_int
                || !ascii_isdigit(buf[8 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
                    && !(buf[8 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                        == '-' as ::core::ffi::c_int
                        && ascii_isdigit(
                            buf[9 as ::core::ffi::c_int as usize] as ::core::ffi::c_int,
                        ) as ::core::ffi::c_int
                            != 0))
        {
            f = -1 as ::core::ffi::c_int;
        }
        if f == -1 as ::core::ffi::c_int
            || sps_flags.get() != 0 as ::core::ffi::c_int && f != 0 as ::core::ffi::c_int
        {
            sps_flags.set(SPS_BEST as ::core::ffi::c_int);
            sps_limit.set(9999 as ::core::ffi::c_int);
            return FAIL;
        }
        if f != 0 as ::core::ffi::c_int {
            sps_flags.set(f);
        }
    }
    if sps_flags.get() == 0 as ::core::ffi::c_int {
        sps_flags.set(SPS_BEST as ::core::ffi::c_int);
    }
    return OK;
}
pub unsafe fn spell_suggest(mut count: ::core::ffi::c_int) {
    let mut need_cap: ::core::ffi::c_int = 0;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut sug: suginfo_T = suginfo_T {
        su_ga: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        su_maxcount: 0,
        su_maxscore: 0,
        su_sfmaxscore: 0,
        su_sga: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        su_badptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        su_badlen: 0,
        su_badflags: 0,
        su_badword: [0; 254],
        su_fbadword: [0; 254],
        su_sal_badword: [0; 254],
        su_banned: hashtab_T {
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
        su_sallang: ::core::ptr::null_mut::<slang_T>(),
    };
    let mut limit: ::core::ffi::c_int = 0;
    let mut selected: ::core::ffi::c_int = 0;
    let mut prev_cursor: pos_T = (*curwin.get()).w_cursor;
    let mut mouse_used: bool = false_0 != 0;
    let mut badlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut msg_scroll_save: ::core::ffi::c_int = msg_scroll.get();
    let wo_spell_save: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_spell;
    if (*curwin.get()).w_onebuf_opt.wo_spell == 0 {
        parse_spelllang(curwin.get());
        (*curwin.get()).w_onebuf_opt.wo_spell = true_0;
    }
    '_skip: {
        if *(*(*curwin.get()).w_s).b_p_spl as ::core::ffi::c_int == NUL {
            emsg(gettext(&raw const e_no_spell as *const ::core::ffi::c_char));
        } else {
            if VIsual_active.get() {
                if (*curwin.get()).w_cursor.lnum != (*VIsual.ptr()).lnum {
                    vim_beep(kOptBoFlagSpell as ::core::ffi::c_int as ::core::ffi::c_uint);
                    break '_skip;
                } else {
                    badlen = (*curwin.get()).w_cursor.col - (*VIsual.ptr()).col;
                    if badlen < 0 as ::core::ffi::c_int {
                        badlen = -badlen;
                    } else {
                        (*curwin.get()).w_cursor.col = (*VIsual.ptr()).col;
                    }
                    badlen += 1;
                    end_visual_mode();
                    badlen = if badlen < get_cursor_line_len() - (*curwin.get()).w_cursor.col {
                        badlen
                    } else {
                        get_cursor_line_len() - (*curwin.get()).w_cursor.col as ::core::ffi::c_int
                    };
                }
            } else if spell_move_to(
                curwin.get(),
                FORWARD as ::core::ffi::c_int,
                SMT_ALL,
                true_0 != 0,
                ::core::ptr::null_mut::<hlf_T>(),
            ) == 0 as size_t
                || (*curwin.get()).w_cursor.col > prev_cursor.col
            {
                (*curwin.get()).w_cursor = prev_cursor;
                let mut curline: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                let mut p: *mut ::core::ffi::c_char =
                    curline.offset((*curwin.get()).w_cursor.col as isize);
                while p > curline && spell_iswordp_nmw(p, curwin.get()) as ::core::ffi::c_int != 0 {
                    p = p.offset(
                        -((utf_head_off(curline, p.offset(-(1 as ::core::ffi::c_int as isize)))
                            + 1 as ::core::ffi::c_int) as isize),
                    );
                }
                while *p as ::core::ffi::c_int != NUL && !spell_iswordp_nmw(p, curwin.get()) {
                    p = p.offset(utfc_ptr2len(p) as isize);
                }
                if !spell_iswordp_nmw(p, curwin.get()) {
                    beep_flush();
                    break '_skip;
                } else {
                    (*curwin.get()).w_cursor.col = p.offset_from(curline) as colnr_T;
                }
            }
            need_cap = check_need_cap(
                curwin.get(),
                (*curwin.get()).w_cursor.lnum,
                (*curwin.get()).w_cursor.col,
            ) as ::core::ffi::c_int;
            line = xstrnsave(get_cursor_line_ptr(), get_cursor_line_len() as size_t);
            spell_suggest_timeout.set(5000 as ::core::ffi::c_int);
            sug = suginfo_T {
                su_ga: garray_T {
                    ga_len: 0,
                    ga_maxlen: 0,
                    ga_itemsize: 0,
                    ga_growsize: 0,
                    ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                },
                su_maxcount: 0,
                su_maxscore: 0,
                su_sfmaxscore: 0,
                su_sga: garray_T {
                    ga_len: 0,
                    ga_maxlen: 0,
                    ga_itemsize: 0,
                    ga_growsize: 0,
                    ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                },
                su_badptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                su_badlen: 0,
                su_badflags: 0,
                su_badword: [0; 254],
                su_fbadword: [0; 254],
                su_sal_badword: [0; 254],
                su_banned: hashtab_T {
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
                su_sallang: ::core::ptr::null_mut::<slang_T>(),
            };
            limit = if sps_limit.get() < Rows.get() - 2 as ::core::ffi::c_int {
                sps_limit.get()
            } else {
                Rows.get() - 2 as ::core::ffi::c_int
            };
            spell_find_suggest(
                line.offset((*curwin.get()).w_cursor.col as isize),
                badlen,
                &raw mut sug,
                limit,
                true_0 != 0,
                need_cap != 0,
                true_0 != 0,
            );
            selected = count;
            msg_ext_set_kind(b"confirm\0".as_ptr() as *const ::core::ffi::c_char);
            if sug.su_ga.ga_len <= 0 as ::core::ffi::c_int {
                msg(
                    gettext(b"No suggestions\0".as_ptr() as *const ::core::ffi::c_char),
                    0 as ::core::ffi::c_int,
                );
            } else if count > 0 as ::core::ffi::c_int {
                if count > sug.su_ga.ga_len {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"Only %ld suggestions\0".as_ptr() as *const ::core::ffi::c_char),
                        sug.su_ga.ga_len as int64_t,
                    );
                }
            } else {
                cmdmsg_rl.set((*curwin.get()).w_onebuf_opt.wo_rl != 0);
                msg_start();
                msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
                lines_left.set(Rows.get());
                let mut fmt: *mut ::core::ffi::c_char =
                    gettext(b"Change \"%.*s\" to:\0".as_ptr() as *const ::core::ffi::c_char);
                if cmdmsg_rl.get() as ::core::ffi::c_int != 0
                    && strncmp(
                        fmt,
                        b"Change\0".as_ptr() as *const ::core::ffi::c_char,
                        6 as size_t,
                    ) == 0 as ::core::ffi::c_int
                {
                    fmt = b":ot \"%.*s\" egnahC\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                }
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    fmt,
                    sug.su_badlen,
                    sug.su_badptr,
                );
                msg_puts(IObuff.ptr() as *mut ::core::ffi::c_char);
                msg_clr_eos();
                msg_putchar('\n' as ::core::ffi::c_int);
                msg_scroll.set(true_0);
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < sug.su_ga.ga_len {
                    let mut stp: *mut suggest_T =
                        (sug.su_ga.ga_data as *mut suggest_T).offset(i as isize);
                    let mut wcopy: [::core::ffi::c_char; 256] = [0; 256];
                    xstrlcpy(
                        &raw mut wcopy as *mut ::core::ffi::c_char,
                        (*stp).st_word,
                        (MAXWLEN as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t,
                    );
                    let mut el: ::core::ffi::c_int = sug.su_badlen - (*stp).st_orglen;
                    if el > 0 as ::core::ffi::c_int
                        && (*stp).st_wordlen + el <= MAXWLEN as ::core::ffi::c_int
                    {
                        '_c2rust_label: {
                            if !sug.su_badptr.is_null() {
                            } else {
                                __assert_fail(
                                    b"sug.su_badptr != NULL\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/spellsuggest.rs\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    552 as ::core::ffi::c_uint,
                                    b"void spell_suggest(int)\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        xmemcpyz(
                            (&raw mut wcopy as *mut ::core::ffi::c_char)
                                .offset((*stp).st_wordlen as isize)
                                as *mut ::core::ffi::c_void,
                            sug.su_badptr.offset((*stp).st_orglen as isize)
                                as *const ::core::ffi::c_void,
                            el as size_t,
                        );
                    }
                    vim_snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        b"%2d\0".as_ptr() as *const ::core::ffi::c_char,
                        i + 1 as ::core::ffi::c_int,
                    );
                    if cmdmsg_rl.get() {
                        rl_mirror_ascii(
                            IObuff.ptr() as *mut ::core::ffi::c_char,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        );
                    }
                    msg_puts(IObuff.ptr() as *mut ::core::ffi::c_char);
                    vim_snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        b" \"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
                        &raw mut wcopy as *mut ::core::ffi::c_char,
                    );
                    msg_puts(IObuff.ptr() as *mut ::core::ffi::c_char);
                    if sug.su_badlen < (*stp).st_orglen {
                        vim_snprintf(
                            IObuff.ptr() as *mut ::core::ffi::c_char,
                            IOSIZE as size_t,
                            gettext(b" < \"%.*s\"\0".as_ptr() as *const ::core::ffi::c_char),
                            (*stp).st_orglen,
                            sug.su_badptr,
                        );
                        msg_puts(IObuff.ptr() as *mut ::core::ffi::c_char);
                    }
                    if p_verbose.get() > 0 as OptInt {
                        if sps_flags.get()
                            & (SPS_DOUBLE as ::core::ffi::c_int | SPS_BEST as ::core::ffi::c_int)
                            != 0
                        {
                            vim_snprintf(
                                IObuff.ptr() as *mut ::core::ffi::c_char,
                                IOSIZE as size_t,
                                b" (%s%d - %d)\0".as_ptr() as *const ::core::ffi::c_char,
                                if (*stp).st_salscore as ::core::ffi::c_int != 0 {
                                    b"s \0".as_ptr() as *const ::core::ffi::c_char
                                } else {
                                    b"\0".as_ptr() as *const ::core::ffi::c_char
                                },
                                (*stp).st_score,
                                (*stp).st_altscore,
                            );
                        } else {
                            vim_snprintf(
                                IObuff.ptr() as *mut ::core::ffi::c_char,
                                IOSIZE as size_t,
                                b" (%d)\0".as_ptr() as *const ::core::ffi::c_char,
                                (*stp).st_score,
                            );
                        }
                        if cmdmsg_rl.get() {
                            rl_mirror_ascii(
                                (IObuff.ptr() as *mut ::core::ffi::c_char)
                                    .offset(1 as ::core::ffi::c_int as isize),
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            );
                        }
                        msg_advance(30 as ::core::ffi::c_int);
                        msg_puts(IObuff.ptr() as *mut ::core::ffi::c_char);
                    }
                    if !ui_has(kUIMessages) || i < sug.su_ga.ga_len - 1 as ::core::ffi::c_int {
                        msg_putchar('\n' as ::core::ffi::c_int);
                    }
                    i += 1;
                }
                cmdmsg_rl.set(false_0 != 0);
                msg_col.set(0 as ::core::ffi::c_int);
                selected = prompt_for_input(
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                    &raw mut mouse_used,
                );
                if mouse_used {
                    selected = sug.su_ga.ga_len + 1 as ::core::ffi::c_int
                        - (cmdline_row.get() - mouse_row.get());
                }
                lines_left.set(Rows.get());
                msg_scroll.set(msg_scroll_save);
            }
            if selected > 0 as ::core::ffi::c_int
                && selected <= sug.su_ga.ga_len
                && u_save_cursor() == OK
            {
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    repl_from.ptr() as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL;
                let _ = *ptr_;
                let mut ptr__0: *mut *mut ::core::ffi::c_void =
                    repl_to.ptr() as *mut *mut ::core::ffi::c_void;
                xfree(*ptr__0);
                *ptr__0 = NULL;
                let _ = *ptr__0;
                let mut stp_0: *mut suggest_T = (sug.su_ga.ga_data as *mut suggest_T)
                    .offset((selected - 1 as ::core::ffi::c_int) as isize);
                if sug.su_badlen > (*stp_0).st_orglen {
                    repl_from.set(xstrnsave(sug.su_badptr, sug.su_badlen as size_t));
                    vim_snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        b"%s%.*s\0".as_ptr() as *const ::core::ffi::c_char,
                        (*stp_0).st_word,
                        sug.su_badlen - (*stp_0).st_orglen,
                        sug.su_badptr.offset((*stp_0).st_orglen as isize),
                    );
                    repl_to.set(xstrdup(IObuff.ptr() as *mut ::core::ffi::c_char));
                } else {
                    repl_from.set(xstrnsave(sug.su_badptr, (*stp_0).st_orglen as size_t));
                    repl_to.set(xstrdup((*stp_0).st_word));
                }
                let mut p_0: *mut ::core::ffi::c_char = xmalloc(
                    strlen(line)
                        .wrapping_sub((*stp_0).st_orglen as size_t)
                        .wrapping_add((*stp_0).st_wordlen as size_t)
                        .wrapping_add(1 as size_t),
                )
                    as *mut ::core::ffi::c_char;
                let mut c: ::core::ffi::c_int =
                    sug.su_badptr.offset_from(line) as ::core::ffi::c_int;
                memmove(
                    p_0 as *mut ::core::ffi::c_void,
                    line as *const ::core::ffi::c_void,
                    c as size_t,
                );
                strcpy(p_0.offset(c as isize), (*stp_0).st_word);
                strcat(p_0, sug.su_badptr.offset((*stp_0).st_orglen as isize));
                ResetRedobuff();
                AppendToRedobuff(b"ciw\0".as_ptr() as *const ::core::ffi::c_char);
                AppendToRedobuffLit(
                    p_0.offset(c as isize),
                    (*stp_0).st_wordlen + sug.su_badlen - (*stp_0).st_orglen,
                );
                AppendCharToRedobuff(ESC);
                ml_replace((*curwin.get()).w_cursor.lnum, p_0, false_0 != 0);
                (*curwin.get()).w_cursor.col = c as colnr_T;
                inserted_bytes(
                    (*curwin.get()).w_cursor.lnum,
                    c as colnr_T,
                    (*stp_0).st_orglen,
                    (*stp_0).st_wordlen,
                );
            } else {
                (*curwin.get()).w_cursor = prev_cursor;
            }
            spell_find_cleanup(&raw mut sug);
            xfree(line as *mut ::core::ffi::c_void);
        }
    }
    (*curwin.get()).w_onebuf_opt.wo_spell = wo_spell_save;
}
pub unsafe fn spell_suggest_list(
    mut gap: *mut garray_T,
    mut word: *mut ::core::ffi::c_char,
    mut maxcount: ::core::ffi::c_int,
    mut need_cap: bool,
    mut interactive: bool,
) {
    let mut sug: suginfo_T = suginfo_T {
        su_ga: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        su_maxcount: 0,
        su_maxscore: 0,
        su_sfmaxscore: 0,
        su_sga: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        su_badptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        su_badlen: 0,
        su_badflags: 0,
        su_badword: [0; 254],
        su_fbadword: [0; 254],
        su_sal_badword: [0; 254],
        su_banned: hashtab_T {
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
        su_sallang: ::core::ptr::null_mut::<slang_T>(),
    };
    spell_find_suggest(
        word,
        0 as ::core::ffi::c_int,
        &raw mut sug,
        maxcount,
        false_0 != 0,
        need_cap,
        interactive,
    );
    ga_init(
        gap,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        sug.su_ga.ga_len + 1 as ::core::ffi::c_int,
    );
    ga_grow(gap, sug.su_ga.ga_len);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < sug.su_ga.ga_len {
        let mut stp: *mut suggest_T = (sug.su_ga.ga_data as *mut suggest_T).offset(i as isize);
        let mut wcopy: *mut ::core::ffi::c_char = xmalloc(
            ((*stp).st_wordlen as size_t)
                .wrapping_add(strlen(sug.su_badptr.offset((*stp).st_orglen as isize)))
                .wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
        strcpy(wcopy, (*stp).st_word);
        strcpy(
            wcopy.offset((*stp).st_wordlen as isize),
            sug.su_badptr.offset((*stp).st_orglen as isize),
        );
        let c2rust_fresh26 = (*gap).ga_len;
        (*gap).ga_len = (*gap).ga_len + 1;
        let c2rust_lvalue_ptr = &raw mut *((*gap).ga_data as *mut *mut ::core::ffi::c_char)
            .offset(c2rust_fresh26 as isize);
        *c2rust_lvalue_ptr = wcopy;
        i += 1;
    }
    spell_find_cleanup(&raw mut sug);
}
unsafe fn spell_find_suggest(
    mut badptr: *mut ::core::ffi::c_char,
    mut badlen: ::core::ffi::c_int,
    mut su: *mut suginfo_T,
    mut maxcount: ::core::ffi::c_int,
    mut banbadword: bool,
    mut need_cap: bool,
    mut interactive: bool,
) {
    let mut attr: hlf_T = HLF_COUNT;
    let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut do_combine: bool = false_0 != 0;
    static expr_busy: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut did_intern: bool = false_0 != 0;
    memset(
        su as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<suginfo_T>(),
    );
    ga_init(
        &raw mut (*su).su_ga,
        ::core::mem::size_of::<suggest_T>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    ga_init(
        &raw mut (*su).su_sga,
        ::core::mem::size_of::<suggest_T>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    if *badptr as ::core::ffi::c_int == NUL {
        return;
    }
    hash_init(&raw mut (*su).su_banned);
    (*su).su_badptr = badptr;
    if badlen != 0 as ::core::ffi::c_int {
        (*su).su_badlen = badlen;
    } else {
        let mut tmplen: size_t = spell_check(
            curwin.get(),
            (*su).su_badptr,
            &raw mut attr,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            false_0 != 0,
        );
        '_c2rust_label: {
            if tmplen <= 2147483647 as ::core::ffi::c_int as size_t {
            } else {
                __assert_fail(
                    b"tmplen <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/spellsuggest.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    715 as ::core::ffi::c_uint,
                    b"void spell_find_suggest(char *, int, suginfo_T *, int, _Bool, _Bool, _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        (*su).su_badlen = tmplen as ::core::ffi::c_int;
    }
    (*su).su_maxcount = maxcount;
    (*su).su_maxscore = SCORE_MAXINIT as ::core::ffi::c_int;
    (*su).su_badlen = if (*su).su_badlen < MAXWLEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
        (*su).su_badlen
    } else {
        MAXWLEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int
    };
    xmemcpyz(
        &raw mut (*su).su_badword as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        (*su).su_badptr as *const ::core::ffi::c_void,
        (*su).su_badlen as size_t,
    );
    spell_casefold(
        curwin.get(),
        (*su).su_badptr,
        (*su).su_badlen,
        &raw mut (*su).su_fbadword as *mut ::core::ffi::c_char,
        MAXWLEN as ::core::ffi::c_int,
    );
    (*su).su_fbadword[(*su).su_badlen as usize] = NUL as ::core::ffi::c_char;
    (*su).su_badflags = badword_captype(
        (*su).su_badptr,
        (*su).su_badptr.offset((*su).su_badlen as isize),
    );
    if need_cap {
        (*su).su_badflags |= WF_ONECAP as ::core::ffi::c_int;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*curbuf.get()).b_s.b_langp.ga_len {
        let mut lp: *mut langp_T =
            ((*curbuf.get()).b_s.b_langp.ga_data as *mut langp_T).offset(i as isize);
        if !(*lp).lp_sallang.is_null() {
            (*su).su_sallang = (*lp).lp_sallang;
            break;
        } else {
            i += 1;
        }
    }
    if !(*su).su_sallang.is_null() {
        spell_soundfold(
            (*su).su_sallang,
            &raw mut (*su).su_fbadword as *mut ::core::ffi::c_char,
            true_0 != 0,
            &raw mut (*su).su_sal_badword as *mut ::core::ffi::c_char,
        );
    }
    let mut c: ::core::ffi::c_int = utf_ptr2char((*su).su_badptr);
    if (if c >= 128 as ::core::ffi::c_int {
        mb_isupper(c) as ::core::ffi::c_int
    } else {
        (*spelltab.ptr()).st_isu[c as usize] as ::core::ffi::c_int
    }) == 0
        && attr as ::core::ffi::c_uint == HLF_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        make_case_word(
            &raw mut (*su).su_badword as *mut ::core::ffi::c_char,
            &raw mut buf as *mut ::core::ffi::c_char,
            WF_ONECAP as ::core::ffi::c_int,
        );
        add_suggestion(
            su,
            &raw mut (*su).su_ga,
            &raw mut buf as *mut ::core::ffi::c_char,
            (*su).su_badlen,
            SCORE_ICASE as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            true_0 != 0,
            (*su).su_sallang,
            false_0 != 0,
        );
    }
    if banbadword {
        add_banned(su, &raw mut (*su).su_badword as *mut ::core::ffi::c_char);
    }
    let mut sps_copy: *mut ::core::ffi::c_char = xstrdup(p_sps.get());
    let mut p: *mut ::core::ffi::c_char = sps_copy;
    while *p as ::core::ffi::c_int != NUL {
        copy_option_part(
            &raw mut p,
            &raw mut buf as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if strncmp(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"expr:\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            if !expr_busy.get() {
                expr_busy.set(true_0 != 0);
                spell_suggest_expr(
                    su,
                    (&raw mut buf as *mut ::core::ffi::c_char)
                        .offset(5 as ::core::ffi::c_int as isize),
                );
                expr_busy.set(false_0 != 0);
            }
        } else if strncmp(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"file:\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            spell_suggest_file(
                su,
                (&raw mut buf as *mut ::core::ffi::c_char).offset(5 as ::core::ffi::c_int as isize),
            );
        } else if strncmp(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"timeout:\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            spell_suggest_timeout.set(atoi(
                (&raw mut buf as *mut ::core::ffi::c_char).offset(8 as ::core::ffi::c_int as isize),
            ));
        } else if !did_intern {
            spell_suggest_intern(su, interactive);
            if sps_flags.get() & SPS_DOUBLE as ::core::ffi::c_int != 0 {
                do_combine = true_0 != 0;
            }
            did_intern = true_0 != 0;
        }
    }
    xfree(sps_copy as *mut ::core::ffi::c_void);
    if do_combine {
        score_combine(su);
    }
}
unsafe fn spell_suggest_expr(mut su: *mut suginfo_T, mut expr: *mut ::core::ffi::c_char) {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let list: *mut list_T =
        eval_spell_expr(&raw mut (*su).su_badword as *mut ::core::ffi::c_char, expr);
    if !list.is_null() {
        let l_: *mut list_T = list;
        if !l_.is_null() {
            let mut li: *mut listitem_T = (*l_).lv_first;
            while !li.is_null() {
                if (*li).li_tv.v_type as ::core::ffi::c_uint
                    == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut score: ::core::ffi::c_int =
                        get_spellword((*li).li_tv.vval.v_list, &raw mut p);
                    if score >= 0 as ::core::ffi::c_int && score <= (*su).su_maxscore {
                        add_suggestion(
                            su,
                            &raw mut (*su).su_ga,
                            p,
                            (*su).su_badlen,
                            score,
                            0 as ::core::ffi::c_int,
                            true,
                            (*su).su_sallang,
                            false,
                        );
                    }
                }
                li = (*li).li_next;
            }
        }
        tv_list_unref(list);
    }
    check_suggestions(su, &raw mut (*su).su_ga);
    cleanup_suggestions(&raw mut (*su).su_ga, (*su).su_maxscore, (*su).su_maxcount);
}
unsafe fn spell_suggest_file(mut su: *mut suginfo_T, mut fname: *mut ::core::ffi::c_char) {
    let mut line: [::core::ffi::c_char; 508] = [0; 508];
    let mut len: ::core::ffi::c_int = 0;
    let mut cword: [::core::ffi::c_char; 254] = [0; 254];
    let mut fd: *mut FILE = os_fopen(fname, b"r\0".as_ptr() as *const ::core::ffi::c_char);
    if fd.is_null() {
        semsg(
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            fname,
        );
        return;
    }
    while !vim_fgets(
        &raw mut line as *mut ::core::ffi::c_char,
        MAXWLEN as ::core::ffi::c_int * 2 as ::core::ffi::c_int,
        fd,
    ) && !got_int.get()
    {
        line_breakcheck();
        let mut p: *mut ::core::ffi::c_char = vim_strchr(
            &raw mut line as *mut ::core::ffi::c_char,
            '/' as ::core::ffi::c_int,
        );
        if p.is_null() {
            continue;
        }
        let c2rust_fresh25 = p;
        p = p.offset(1);
        *c2rust_fresh25 = NUL as ::core::ffi::c_char;
        if strcasecmp(
            &raw mut (*su).su_badword as *mut ::core::ffi::c_char,
            &raw mut line as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            len = 0 as ::core::ffi::c_int;
            while (*p.offset(len as isize) as uint8_t as ::core::ffi::c_int)
                >= ' ' as ::core::ffi::c_int
            {
                len += 1;
            }
            *p.offset(len as isize) = NUL as ::core::ffi::c_char;
            if captype(p, ::core::ptr::null::<::core::ffi::c_char>()) == 0 as ::core::ffi::c_int {
                make_case_word(
                    p,
                    &raw mut cword as *mut ::core::ffi::c_char,
                    (*su).su_badflags,
                );
                p = &raw mut cword as *mut ::core::ffi::c_char;
            }
            add_suggestion(
                su,
                &raw mut (*su).su_ga,
                p,
                (*su).su_badlen,
                SCORE_FILE as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                true_0 != 0,
                (*su).su_sallang,
                false_0 != 0,
            );
        }
    }
    fclose(fd);
    check_suggestions(su, &raw mut (*su).su_ga);
    cleanup_suggestions(&raw mut (*su).su_ga, (*su).su_maxscore, (*su).su_maxcount);
}
unsafe fn spell_suggest_intern(mut su: *mut suginfo_T, mut interactive: bool) {
    suggest_load_files();
    suggest_try_special(su);
    suggest_try_change(su);
    if sps_flags.get() & SPS_DOUBLE as ::core::ffi::c_int != 0 {
        score_comp_sal(su);
    }
    if sps_flags.get() & SPS_FAST as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        if sps_flags.get() & SPS_BEST as ::core::ffi::c_int != 0 {
            rescore_suggestions(su);
        }
        suggest_try_soundalike_prep();
        (*su).su_maxscore = SCORE_SFMAX1 as ::core::ffi::c_int;
        (*su).su_sfmaxscore = SCORE_MAXINIT as ::core::ffi::c_int * 3 as ::core::ffi::c_int;
        suggest_try_soundalike(su);
        if (*su).su_ga.ga_len
            < (if (*su).su_maxcount < 130 as ::core::ffi::c_int {
                150 as ::core::ffi::c_int
            } else {
                (*su).su_maxcount + 20 as ::core::ffi::c_int
            })
        {
            (*su).su_maxscore = SCORE_SFMAX2 as ::core::ffi::c_int;
            suggest_try_soundalike(su);
            if (*su).su_ga.ga_len
                < (if (*su).su_maxcount < 130 as ::core::ffi::c_int {
                    150 as ::core::ffi::c_int
                } else {
                    (*su).su_maxcount + 20 as ::core::ffi::c_int
                })
            {
                (*su).su_maxscore = SCORE_SFMAX3 as ::core::ffi::c_int;
                suggest_try_soundalike(su);
            }
        }
        (*su).su_maxscore = (*su).su_sfmaxscore;
        suggest_try_soundalike_finish();
    }
    os_breakcheck();
    if interactive as ::core::ffi::c_int != 0 && got_int.get() as ::core::ffi::c_int != 0 {
        vgetc();
        got_int.set(false_0 != 0);
    }
    if sps_flags.get() & SPS_DOUBLE as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        && (*su).su_ga.ga_len != 0 as ::core::ffi::c_int
    {
        if sps_flags.get() & SPS_BEST as ::core::ffi::c_int != 0 {
            rescore_suggestions(su);
        }
        check_suggestions(su, &raw mut (*su).su_ga);
        cleanup_suggestions(&raw mut (*su).su_ga, (*su).su_maxscore, (*su).su_maxcount);
    }
}
unsafe fn spell_find_cleanup(mut su: *mut suginfo_T) {
    let mut _gap: *mut garray_T = &raw mut (*su).su_ga;
    if !(*_gap).ga_data.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*_gap).ga_len {
            let mut _item: *mut suggest_T = ((*_gap).ga_data as *mut suggest_T).offset(i as isize);
            xfree((*_item).st_word as *mut ::core::ffi::c_void);
            i += 1;
        }
    }
    ga_clear(_gap);
    let mut _gap_0: *mut garray_T = &raw mut (*su).su_sga;
    if !(*_gap_0).ga_data.is_null() {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*_gap_0).ga_len {
            let mut _item_0: *mut suggest_T =
                ((*_gap_0).ga_data as *mut suggest_T).offset(i_0 as isize);
            xfree((*_item_0).st_word as *mut ::core::ffi::c_void);
            i_0 += 1;
        }
    }
    ga_clear(_gap_0);
    hash_clear_all(&raw mut (*su).su_banned, 0 as ::core::ffi::c_uint);
}
unsafe fn suggest_try_special(mut su: *mut suginfo_T) {
    let mut word: [::core::ffi::c_char; 254] = [0; 254];
    let mut p: *mut ::core::ffi::c_char =
        skiptowhite(&raw mut (*su).su_fbadword as *mut ::core::ffi::c_char);
    let mut len: size_t =
        p.offset_from(&raw mut (*su).su_fbadword as *mut ::core::ffi::c_char) as size_t;
    p = skipwhite(p);
    if strlen(p) == len
        && strncmp(
            &raw mut (*su).su_fbadword as *mut ::core::ffi::c_char,
            p,
            len,
        ) == 0 as ::core::ffi::c_int
    {
        let mut c: ::core::ffi::c_char = (*su).su_fbadword[len as usize];
        (*su).su_fbadword[len as usize] = NUL as ::core::ffi::c_char;
        make_case_word(
            &raw mut (*su).su_fbadword as *mut ::core::ffi::c_char,
            &raw mut word as *mut ::core::ffi::c_char,
            (*su).su_badflags,
        );
        (*su).su_fbadword[len as usize] = c;
        add_suggestion(
            su,
            &raw mut (*su).su_ga,
            &raw mut word as *mut ::core::ffi::c_char,
            (*su).su_badlen,
            (3 as ::core::ffi::c_int * SCORE_REP as ::core::ffi::c_int + 0 as ::core::ffi::c_int)
                / 4 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            true_0 != 0,
            (*su).su_sallang,
            false_0 != 0,
        );
    }
}
unsafe fn suggest_try_change(mut su: *mut suginfo_T) {
    let mut fword: [::core::ffi::c_char; 254] = [0; 254];
    strcpy(
        &raw mut fword as *mut ::core::ffi::c_char,
        &raw mut (*su).su_fbadword as *mut ::core::ffi::c_char,
    );
    let mut n: ::core::ffi::c_int =
        strlen(&raw mut fword as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = (*su).su_badptr.offset((*su).su_badlen as isize);
    spell_casefold(
        curwin.get(),
        p,
        strlen(p) as ::core::ffi::c_int,
        (&raw mut fword as *mut ::core::ffi::c_char).offset(n as isize),
        MAXWLEN as ::core::ffi::c_int - n,
    );
    n = strlen((*su).su_badptr) as ::core::ffi::c_int;
    if n < MAXWLEN as ::core::ffi::c_int {
        fword[n as usize] = NUL as ::core::ffi::c_char;
    }
    let mut lpi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lpi < (*(*curwin.get()).w_s).b_langp.ga_len {
        let mut lp: *mut langp_T =
            ((*(*curwin.get()).w_s).b_langp.ga_data as *mut langp_T).offset(lpi as isize);
        if !(*(*lp).lp_slang).sl_fbyts.is_null() {
            suggest_trie_walk(
                su,
                lp,
                &raw mut fword as *mut ::core::ffi::c_char,
                false_0 != 0,
            );
        }
        lpi += 1;
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
