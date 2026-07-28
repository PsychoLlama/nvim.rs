use crate::src::nvim::api::private::helpers::api_clear_error;
use crate::src::nvim::api::win_config::parse_winborder;
use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::autocmd::{check_ei, get_event_name_no_group};
use crate::src::nvim::charset::{
    buf_init_chartab, char2cells, check_isopt, getdigits_int, hexhex2nr, init_chartab, ptr2cells,
    transchar, transchar_byte,
};
use crate::src::nvim::cmdexpand::ExpandGeneric;
use crate::src::nvim::cursor::coladvance;
use crate::src::nvim::cursor_shape::parse_shape_opt;
use crate::src::nvim::diff::{diffanchors_changed, diffopt_changed};
use crate::src::nvim::digraph::keymap_init;
use crate::src::nvim::drawscreen::{
    comp_col, redraw_all_later, redraw_buf_later, redraw_curbuf_later, redraw_later, redrawWinline,
    status_redraw_buf,
};
use crate::src::nvim::eval::userfunc::get_scriptlocal_funcname;
use crate::src::nvim::eval::vars::{do_unlet, get_var_value};
use crate::src::nvim::ex_getln::check_opt_wim;
use crate::src::nvim::fold::{
    foldUpdateAll, foldmethodIsDiff, foldmethodIsExpr, foldmethodIsIndent, foldmethodIsMarker,
    newFoldLevel,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{schar_from_char, schar_from_str};
use crate::src::nvim::highlight_group::{get_highlight_name, init_highlight};
use crate::src::nvim::indent::{briopt_check, tabstop_set};
use crate::src::nvim::indent_c::parse_cino;
use crate::src::nvim::insexpand::set_cpt_callbacks;
use crate::src::nvim::main::{
    IObuff, VIsual_active, bkc_flags, breakat_flags, cia_flags, cmdpreview, cot_flags, curtab,
    curwin, didset_vim, didset_vimruntime, e_invalid_format_string_single_percent_s, e_invarg,
    e_leadtab_requires_tab, e_modifiable, e_unsupportedoption, empty_string_option, first_tabpage,
    firstbuf, firstwin, km_startsel, km_stopsel, p_bex, p_bg, p_bkc, p_breakat, p_bs, p_cia, p_cot,
    p_ei, p_enc, p_fcs, p_fenc, p_hlg, p_isk, p_km, p_lcs, p_mousescroll, p_mousescroll_hor,
    p_mousescroll_vert, p_pm, p_pumborder, p_ruf, p_shada, p_tc, p_ve, p_winborder, ru_wid, secure,
    spo_flags, ssop_flags, stl_syntax, tc_flags, ve_flags,
};
use crate::src::nvim::mark::free_fmark;
use crate::src::nvim::mbyte::{
    enc_canonize, get_encoding_name, utf_ptr2char, utfc_ptr2len, utfc_ptr2schar,
};
use crate::src::nvim::memline::ml_setflags;
use crate::src::nvim::memory::{strequal, xfree, xmalloc, xmemdupz, xstrdup};
use crate::src::nvim::message::{
    messagesopt_changed, msg_grid_validate, verbose_open, verbose_stop,
};
use crate::src::nvim::r#move::validate_virtcol;
use crate::src::nvim::option::{
    copy_option_part, did_set_title, fill_culopt_flags, get_fileformat, get_option,
    get_option_default, get_option_varp_scope_from, kOptFlagComma, kOptFlagNDname, kOptFlagNFname,
    kOptFlagOneComma, p_vfile, parse_winhl_opt, redraw_titles, set_iminsert_global,
    set_imsearch_global, set_option_direct, skip_to_option_part, valid_name,
};
use crate::src::nvim::options::{
    kOptAmbiwidth, kOptBackupcopy, kOptBelloff, kOptBkcFlagAuto, kOptBkcFlagNo, kOptBkcFlagYes,
    kOptCasemap, kOptClipboard, kOptComments, kOptCompleteopt, kOptDisplay, kOptFileformat,
    kOptFileformats, kOptFoldopen, kOptJumpoptions, kOptRedrawdebug, kOptSessionoptions,
    kOptSsopFlagCurdir, kOptSsopFlagSesdir, kOptStatusline, kOptSwitchbuf, kOptTabclose,
    kOptTagcase, kOptTermpastefilter, kOptViewoptions, kOptVirtualedit, kOptWildoptions,
    opt_bh_values, opt_bkc_values, opt_bt_values, opt_cot_values, opt_dip_algorithm_values,
    opt_dip_inline_values, opt_ff_values, opt_scl_values, opt_spo_values, opt_ssop_values,
    opt_tc_values, opt_ve_values,
};
use crate::src::nvim::os::env::vim_unsetenv_ext;
use crate::src::nvim::os::libc::{
    __assert_fail, gettext, memcmp, memset, snprintf, strcmp, strlen, strncmp, strpbrk, strstr,
};
use crate::src::nvim::os::time::os_time;
use crate::src::nvim::shada::get_shada_parameter;
use crate::src::nvim::spell::{
    compile_cap_prog, did_set_spell_option, spell_reload, valid_spellfile, valid_spelllang,
};
use crate::src::nvim::spellfile::spell_check_msm;
use crate::src::nvim::spellsuggest::spell_check_sps;
use crate::src::nvim::strings::{vim_snprintf, vim_strchr};
use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, CharsOption, CompleteListItemGetter, Error, ErrorType,
    FloatAnchor, FloatRelative, OptIndex, OptInt, OptVal, OptValData, OptValType, String_0,
    Terminal, VirtText, VirtTextChunk, WinConfig, WinSplit, WinStyle, buf_T, colnr_T, expand_T,
    fcs_chars_T, fmark_T, fmarkv_T, int64_t, lcs_chars_T, linenr_T, lpos_T, optexpand_T, optset_T,
    pos_T, regmatch_T, schar_T, size_t, tabpage_T, uint8_t, uint32_t, vimoption_T, win_T,
};
use crate::src::nvim::window::{check_colorcolumn, global_stl_height};
use crate::src::nvim::winfloat::win_config_float;
use core::ffi::{c_char, c_double, c_int, c_uchar, c_uint, c_void};
unsafe extern "C" {
    fn vim_regexec(rmp: *mut regmatch_T, line: *const c_char, col: colnr_T) -> bool;
    fn terminal_notify_theme(term: *mut Terminal, dark: bool);
}
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitLeft: WinSplit = 0;
pub const kFloatRelativeEditor: FloatRelative = 0;
pub const kErrorTypeNone: ErrorType = -1;
pub type C2Rust_Unnamed_12 = c_uint;
pub const MAXLNUM: C2Rust_Unnamed_12 = 2147483647;
pub type C2Rust_Unnamed_13 = c_uint;
pub const kZIndexFloatDefault: C2Rust_Unnamed_13 = 50;
pub const kOptValTypeString: OptValType = 2;
pub type C2Rust_Unnamed_15 = c_uint;
pub const STL_CLICK_FUNC: C2Rust_Unnamed_15 = 64;
pub const STL_TABCLOSENR: C2Rust_Unnamed_15 = 88;
pub const STL_TABPAGENR: C2Rust_Unnamed_15 = 84;
pub const STL_HIGHLIGHT_COMB: C2Rust_Unnamed_15 = 36;
pub const STL_HIGHLIGHT: C2Rust_Unnamed_15 = 35;
pub const STL_USER_HL: C2Rust_Unnamed_15 = 42;
pub const STL_TRUNCMARK: C2Rust_Unnamed_15 = 60;
pub const STL_SEPARATE: C2Rust_Unnamed_15 = 61;
pub const STL_VIM_EXPR: C2Rust_Unnamed_15 = 123;
pub const STL_SIGNCOL: C2Rust_Unnamed_15 = 115;
pub const STL_FOLDCOL: C2Rust_Unnamed_15 = 67;
pub const STL_SHOWCMD: C2Rust_Unnamed_15 = 83;
pub const STL_PAGENUM: C2Rust_Unnamed_15 = 78;
pub const STL_ARGLISTSTAT: C2Rust_Unnamed_15 = 97;
pub const STL_ALTPERCENT: C2Rust_Unnamed_15 = 80;
pub const STL_PERCENTAGE: C2Rust_Unnamed_15 = 112;
pub const STL_QUICKFIX: C2Rust_Unnamed_15 = 113;
pub const STL_MODIFIED_ALT: C2Rust_Unnamed_15 = 77;
pub const STL_MODIFIED: C2Rust_Unnamed_15 = 109;
pub const STL_PREVIEWFLAG_ALT: C2Rust_Unnamed_15 = 87;
pub const STL_PREVIEWFLAG: C2Rust_Unnamed_15 = 119;
pub const STL_FILETYPE_ALT: C2Rust_Unnamed_15 = 89;
pub const STL_FILETYPE: C2Rust_Unnamed_15 = 121;
pub const STL_HELPFLAG_ALT: C2Rust_Unnamed_15 = 72;
pub const STL_HELPFLAG: C2Rust_Unnamed_15 = 104;
pub const STL_ROFLAG_ALT: C2Rust_Unnamed_15 = 82;
pub const STL_ROFLAG: C2Rust_Unnamed_15 = 114;
pub const STL_BYTEVAL_X: C2Rust_Unnamed_15 = 66;
pub const STL_BYTEVAL: C2Rust_Unnamed_15 = 98;
pub const STL_OFFSET_X: C2Rust_Unnamed_15 = 79;
pub const STL_OFFSET: C2Rust_Unnamed_15 = 111;
pub const STL_KEYMAP: C2Rust_Unnamed_15 = 107;
pub const STL_BUFNO: C2Rust_Unnamed_15 = 110;
pub const STL_NUMLINES: C2Rust_Unnamed_15 = 76;
pub const STL_LINE: C2Rust_Unnamed_15 = 108;
pub const STL_VIRTCOL_ALT: C2Rust_Unnamed_15 = 86;
pub const STL_VIRTCOL: C2Rust_Unnamed_15 = 118;
pub const STL_COLUMN: C2Rust_Unnamed_15 = 99;
pub const STL_FILENAME: C2Rust_Unnamed_15 = 116;
pub const STL_FULLPATH: C2Rust_Unnamed_15 = 70;
pub const STL_FILEPATH: C2Rust_Unnamed_15 = 102;
pub type C2Rust_Unnamed_18 = c_uint;
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_18 = 83;
pub const SHM_FILEINFO: C2Rust_Unnamed_18 = 70;
pub const SHM_RECORDING: C2Rust_Unnamed_18 = 113;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_18 = 67;
pub const SHM_COMPLETIONMENU: C2Rust_Unnamed_18 = 99;
pub const SHM_INTRO: C2Rust_Unnamed_18 = 73;
pub const SHM_ATTENTION: C2Rust_Unnamed_18 = 65;
pub const SHM_SEARCH: C2Rust_Unnamed_18 = 115;
pub const SHM_OVERALL: C2Rust_Unnamed_18 = 79;
pub const SHM_OVER: C2Rust_Unnamed_18 = 111;
pub const SHM_TRUNCALL: C2Rust_Unnamed_18 = 84;
pub const SHM_TRUNC: C2Rust_Unnamed_18 = 116;
pub const SHM_WRITE: C2Rust_Unnamed_18 = 87;
pub const SHM_ABBREVIATIONS: C2Rust_Unnamed_18 = 97;
pub const SHM_WRI: C2Rust_Unnamed_18 = 119;
pub const SHM_LINES: C2Rust_Unnamed_18 = 108;
pub const SHM_MOD: C2Rust_Unnamed_18 = 109;
pub const SHM_RO: C2Rust_Unnamed_18 = 114;
pub type C2Rust_Unnamed_19 = c_uint;
pub const UPD_NOT_VALID: C2Rust_Unnamed_19 = 40;
pub const UPD_INVERTED: C2Rust_Unnamed_19 = 20;
pub const UPD_VALID: C2Rust_Unnamed_19 = 10;
pub type C2Rust_Unnamed_20 = c_uint;
pub const CPT_MENU: C2Rust_Unnamed_20 = 2;
pub const CPT_KIND: C2Rust_Unnamed_20 = 1;
pub const CPT_ABBR: C2Rust_Unnamed_20 = 0;
pub type C2Rust_Unnamed_21 = c_uint;
pub const OPT_LOCAL: C2Rust_Unnamed_21 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_21 = 1;
pub const kListchars: CharsOption = 1;
pub const kFillchars: CharsOption = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct chars_tab {
    pub cp: *mut schar_T,
    pub name: String_0,
    pub def: *const c_char,
    pub fallback: *const c_char,
}
pub const LSIZE: C2Rust_Unnamed_22 = 512;
pub type C2Rust_Unnamed_22 = c_uint;
pub const __ASSERT_FUNCTION: [c_char; 74] = unsafe {
    ::core::mem::transmute::<[u8; 74], [c_char; 74]>(
        *b"int opt_strings_flags(const char *, const char **, unsigned int *, _Bool)\0",
    )
};
pub const NULL: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub const B_IMODE_USE_INSERT: c_int = -1 as c_int;
pub const B_IMODE_NONE: c_int = 0 as c_int;
pub const B_IMODE_LMAP: c_int = 1 as c_int;
pub const OK: c_int = 1 as c_int;
pub const FAIL: c_int = 0 as c_int;
pub const NUL: c_int = '\0' as c_int;
pub const HIGHLIGHT_INIT: [c_char; 779] = unsafe {
    ::core::mem::transmute::<
        [u8; 779],
        [c_char; 779],
    >(
        *b"8:SpecialKey,~:EndOfBuffer,z:TermCursor,@:NonText,d:Directory,e:ErrorMsg,i:IncSearch,l:Search,y:CurSearch,m:MoreMsg,M:ModeMsg,n:LineNr,a:LineNrAbove,b:LineNrBelow,N:CursorLineNr,G:CursorLineSign,O:CursorLineFold,r:Question,s:StatusLine,S:StatusLineNC,c:VertSplit,t:Title,v:Visual,V:VisualNOS,w:WarningMsg,W:WildMenu,f:Folded,F:FoldColumn,A:DiffAdd,C:DiffChange,D:DiffDelete,T:DiffText,E:DiffTextAdd,>:SignColumn,-:Conceal,B:SpellBad,P:SpellCap,R:SpellRare,L:SpellLocal,+:Pmenu,=:PmenuSel,k:PmenuMatch,<:PmenuMatchSel,[:PmenuKind,]:PmenuKindSel,{:PmenuExtra,}:PmenuExtraSel,x:PmenuSbar,X:PmenuThumb,*:TabLine,#:TabLineSel,_:TabLineFill,!:CursorColumn,.:CursorLine,o:ColorColumn,q:QuickFixLine,z:StatusLineTerm,Z:StatusLineTermNC,g:MsgArea,h:ComplMatchIns,0:Whitespace,I:PreInsert\0",
    )
};
pub const EOL_MAC: c_int = 2 as c_int;
pub const FO_ALL: [c_char; 22] =
    unsafe { ::core::mem::transmute::<[u8; 22], [c_char; 22]>(*b"tcro/q2vlb1mMBn,aw]jp\0") };
pub const CPO_VI: [c_char; 47] = unsafe {
    ::core::mem::transmute::<[u8; 47], [c_char; 47]>(
        *b"aAbBcCdDeEfFiIJKlLmMnoOpPqrRsStuvWxXyZ$!%+>;~_\0",
    )
};
pub const WW_ALL: [c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [c_char; 10]>(*b"bshl<>[]~\0") };
pub const MOUSE_ALL: [c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [c_char; 8]>(*b"anvichr\0") };
pub const MOUSESCROLL_VERT_DFLT: c_int = 3 as c_int;
pub const MOUSESCROLL_HOR_DFLT: c_int = 6 as c_int;
pub const COCU_ALL: [c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [c_char; 5]>(*b"nvic\0") };
pub const COM_ALL: [c_char; 11] =
    unsafe { ::core::mem::transmute::<[u8; 11], [c_char; 11]>(*b"nbsmexflrO\0") };
pub const SCL_NO: c_int = -1 as c_int;
pub const SCL_NUM: c_int = -2 as c_int;
pub const SHAPE_CURSOR: c_int = 2 as c_int;
pub const IOSIZE: c_int = 1024 as c_int + 1 as c_int;
pub const SID_NONE: c_int = -6 as c_int;
pub const STL_IN_ICON: c_int = 1 as c_int;
pub const STL_IN_TITLE: c_int = 2 as c_int;
static e_illegal_character_after_chr: GlobalCell<[c_char; 35]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 35], [c_char; 35]>(*b"E535: Illegal character after <%c>\0")
});
static e_comma_required: GlobalCell<[c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [c_char; 21]>(*b"E536: Comma required\0")
});
static e_unclosed_expression_sequence: GlobalCell<[c_char; 35]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 35], [c_char; 35]>(*b"E540: Unclosed expression sequence\0")
});
static e_unbalanced_groups: GlobalCell<[c_char; 24]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], [c_char; 24]>(*b"E542: Unbalanced groups\0")
});
static e_backupext_and_patchmode_are_equal: GlobalCell<[c_char; 44]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 44], [c_char; 44]>(
        *b"E589: 'backupext' and 'patchmode' are equal\0",
    )
});
static e_showbreak_contains_unprintable_or_wide_character: GlobalCell<[c_char; 57]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 57], [c_char; 57]>(
            *b"E595: 'showbreak' contains unprintable or wide character\0",
        )
    });
static e_wrong_number_of_characters_for_field_str: GlobalCell<[c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [c_char; 49]>(
            *b"E1511: Wrong number of characters for field \"%s\"\0",
        )
    });
static e_wrong_character_width_for_field_str: GlobalCell<[c_char; 44]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 44], [c_char; 44]>(
        *b"E1512: Wrong character width for field \"%s\"\0",
    )
});
static SHM_ALL: GlobalCell<[c_char; 23]> = GlobalCell::new([
    SHM_RO as c_int as c_char,
    SHM_MOD as c_int as c_char,
    SHM_LINES as c_int as c_char,
    SHM_WRI as c_int as c_char,
    SHM_ABBREVIATIONS as c_int as c_char,
    SHM_WRITE as c_int as c_char,
    SHM_TRUNC as c_int as c_char,
    SHM_TRUNCALL as c_int as c_char,
    SHM_OVER as c_int as c_char,
    SHM_OVERALL as c_int as c_char,
    SHM_SEARCH as c_int as c_char,
    SHM_ATTENTION as c_int as c_char,
    SHM_INTRO as c_int as c_char,
    SHM_COMPLETIONMENU as c_int as c_char,
    SHM_COMPLETIONSCAN as c_int as c_char,
    SHM_RECORDING as c_int as c_char,
    SHM_FILEINFO as c_int as c_char,
    SHM_SEARCHCOUNT as c_int as c_char,
    'n' as c_char,
    'f' as c_char,
    'x' as c_char,
    'i' as c_char,
    0 as c_char,
]);
pub unsafe extern "C" fn didset_string_options() {
    check_str_opt(kOptCasemap, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptBackupcopy, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptBelloff, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptCompleteopt, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptSessionoptions, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptViewoptions, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptFoldopen, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptDisplay, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptJumpoptions, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptRedrawdebug, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptTagcase, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptTermpastefilter, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptVirtualedit, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptSwitchbuf, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptTabclose, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptWildoptions, ::core::ptr::null_mut::<*mut c_char>());
    check_str_opt(kOptClipboard, ::core::ptr::null_mut::<*mut c_char>());
}
pub unsafe extern "C" fn illegal_char(
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
    mut c: c_int,
) -> *mut c_char {
    if errbuf.is_null() {
        return b"\0".as_ptr() as *const c_char as *mut c_char;
    }
    vim_snprintf(
        errbuf,
        errbuflen,
        gettext(b"E539: Illegal character <%s>\0".as_ptr() as *const c_char),
        transchar(c),
    );
    return errbuf;
}
unsafe extern "C" fn illegal_char_after_chr(
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
    mut c: c_int,
) -> *mut c_char {
    if errbuf.is_null() {
        return b"\0".as_ptr() as *const c_char as *mut c_char;
    }
    vim_snprintf(
        errbuf,
        errbuflen,
        gettext((e_illegal_character_after_chr.ptr() as *const _) as *const c_char),
        c,
    );
    return errbuf;
}
pub unsafe extern "C" fn check_buf_options(mut buf: *mut buf_T) {
    check_string_option(&raw mut (*buf).b_p_bh);
    check_string_option(&raw mut (*buf).b_p_bt);
    check_string_option(&raw mut (*buf).b_p_fenc);
    check_string_option(&raw mut (*buf).b_p_ff);
    check_string_option(&raw mut (*buf).b_p_def);
    check_string_option(&raw mut (*buf).b_p_inc);
    check_string_option(&raw mut (*buf).b_p_inex);
    check_string_option(&raw mut (*buf).b_p_inde);
    check_string_option(&raw mut (*buf).b_p_indk);
    check_string_option(&raw mut (*buf).b_p_fp);
    check_string_option(&raw mut (*buf).b_p_fex);
    check_string_option(&raw mut (*buf).b_p_kp);
    check_string_option(&raw mut (*buf).b_p_mps);
    check_string_option(&raw mut (*buf).b_p_fo);
    check_string_option(&raw mut (*buf).b_p_flp);
    check_string_option(&raw mut (*buf).b_p_isk);
    check_string_option(&raw mut (*buf).b_p_com);
    check_string_option(&raw mut (*buf).b_p_cms);
    check_string_option(&raw mut (*buf).b_p_nf);
    check_string_option(&raw mut (*buf).b_p_qe);
    check_string_option(&raw mut (*buf).b_p_syn);
    check_string_option(&raw mut (*buf).b_s.b_syn_isk);
    check_string_option(&raw mut (*buf).b_s.b_p_spc);
    check_string_option(&raw mut (*buf).b_s.b_p_spf);
    check_string_option(&raw mut (*buf).b_s.b_p_spl);
    check_string_option(&raw mut (*buf).b_s.b_p_spo);
    check_string_option(&raw mut (*buf).b_p_sua);
    check_string_option(&raw mut (*buf).b_p_cink);
    check_string_option(&raw mut (*buf).b_p_cino);
    parse_cino(buf);
    check_string_option(&raw mut (*buf).b_p_lop);
    check_string_option(&raw mut (*buf).b_p_ft);
    check_string_option(&raw mut (*buf).b_p_cinw);
    check_string_option(&raw mut (*buf).b_p_cinsd);
    check_string_option(&raw mut (*buf).b_p_cot);
    check_string_option(&raw mut (*buf).b_p_cpt);
    check_string_option(&raw mut (*buf).b_p_cfu);
    check_string_option(&raw mut (*buf).b_p_ofu);
    check_string_option(&raw mut (*buf).b_p_keymap);
    check_string_option(&raw mut (*buf).b_p_gefm);
    check_string_option(&raw mut (*buf).b_p_gp);
    check_string_option(&raw mut (*buf).b_p_mp);
    check_string_option(&raw mut (*buf).b_p_efm);
    check_string_option(&raw mut (*buf).b_p_ep);
    check_string_option(&raw mut (*buf).b_p_path);
    check_string_option(&raw mut (*buf).b_p_tags);
    check_string_option(&raw mut (*buf).b_p_ffu);
    check_string_option(&raw mut (*buf).b_p_tfu);
    check_string_option(&raw mut (*buf).b_p_tc);
    check_string_option(&raw mut (*buf).b_p_dict);
    check_string_option(&raw mut (*buf).b_p_dia);
    check_string_option(&raw mut (*buf).b_p_tsr);
    check_string_option(&raw mut (*buf).b_p_tsrfu);
    check_string_option(&raw mut (*buf).b_p_lw);
    check_string_option(&raw mut (*buf).b_p_bkc);
    check_string_option(&raw mut (*buf).b_p_menc);
    check_string_option(&raw mut (*buf).b_p_vsts);
    check_string_option(&raw mut (*buf).b_p_vts);
}
pub unsafe extern "C" fn free_string_option(mut p: *mut c_char) {
    if p != empty_string_option.ptr() as *mut c_char {
        xfree(p as *mut c_void);
    }
}
pub unsafe extern "C" fn clear_string_option(mut pp: *mut *mut c_char) {
    if *pp != empty_string_option.ptr() as *mut c_char {
        xfree(*pp as *mut c_void);
    }
    *pp = empty_string_option.ptr() as *mut c_char;
}
pub unsafe extern "C" fn check_string_option(mut pp: *mut *mut c_char) {
    if (*pp).is_null() {
        *pp = empty_string_option.ptr() as *mut c_char;
    }
}
unsafe extern "C" fn valid_filetype(mut val: *const c_char) -> bool {
    return valid_name(val, b".-_\0".as_ptr() as *const c_char);
}
pub unsafe extern "C" fn check_signcolumn(mut scl: *mut c_char, mut wp: *mut win_T) -> c_int {
    let mut val: *mut c_char = empty_string_option.ptr() as *mut c_char;
    if !scl.is_null() {
        val = scl;
    } else if !wp.is_null() {
        val = (*wp).w_onebuf_opt.wo_scl;
    }
    if *val as c_int == NUL {
        return FAIL;
    }
    if opt_strings_flags(
        val,
        opt_scl_values.ptr() as *mut *const c_char,
        ::core::ptr::null_mut::<c_uint>(),
        false_0 != 0,
    ) == OK
    {
        if wp.is_null() {
            return OK;
        }
        if strncmp(val, b"no\0".as_ptr() as *const c_char, 2 as size_t) == 0 {
            (*wp).w_maxscwidth = SCL_NO;
            (*wp).w_minscwidth = (*wp).w_maxscwidth;
        } else if strncmp(val, b"nu\0".as_ptr() as *const c_char, 2 as size_t) == 0
            && ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
        {
            (*wp).w_maxscwidth = SCL_NUM;
            (*wp).w_minscwidth = (*wp).w_maxscwidth;
        } else if strncmp(val, b"yes:\0".as_ptr() as *const c_char, 4 as size_t) == 0 {
            (*wp).w_maxscwidth = *val.offset(4 as c_int as isize) as c_int - '0' as c_int;
            (*wp).w_minscwidth = (*wp).w_maxscwidth;
        } else if *val as c_int == 'y' as c_int {
            (*wp).w_maxscwidth = 1 as c_int;
            (*wp).w_minscwidth = (*wp).w_maxscwidth;
        } else if strncmp(val, b"auto:\0".as_ptr() as *const c_char, 5 as size_t) == 0 {
            (*wp).w_minscwidth = 0 as c_int;
            (*wp).w_maxscwidth = *val.offset(5 as c_int as isize) as c_int - '0' as c_int;
        } else {
            (*wp).w_minscwidth = 0 as c_int;
            (*wp).w_maxscwidth = 1 as c_int;
        }
    } else {
        if strncmp(val, b"auto:\0".as_ptr() as *const c_char, 5 as size_t) != 0 as c_int
            || strlen(val) != 8 as size_t
            || !ascii_isdigit(*val.offset(5 as c_int as isize) as c_int)
            || *val.offset(6 as c_int as isize) as c_int != '-' as c_int
            || !ascii_isdigit(*val.offset(7 as c_int as isize) as c_int)
        {
            return FAIL;
        }
        let mut min: c_int = *val.offset(5 as c_int as isize) as c_int - '0' as c_int;
        let mut max: c_int = *val.offset(7 as c_int as isize) as c_int - '0' as c_int;
        if min < 1 as c_int || max < 2 as c_int || min > 8 as c_int || min >= max {
            return FAIL;
        }
        if wp.is_null() {
            return OK;
        }
        (*wp).w_minscwidth = min;
        (*wp).w_maxscwidth = max;
    }
    let mut scwidth: c_int = if (*wp).w_minscwidth <= 0 as c_int {
        0 as c_int
    } else if (*wp).w_maxscwidth < (*wp).w_scwidth {
        (*wp).w_maxscwidth
    } else {
        (*wp).w_scwidth
    };
    (*wp).w_scwidth = if (*wp).w_minscwidth > scwidth {
        (*wp).w_minscwidth
    } else {
        scwidth
    };
    return OK;
}
pub unsafe extern "C" fn check_stl_option(mut s: *mut c_char) -> *const c_char {
    let mut groupdepth: c_int = 0 as c_int;
    static errbuf: GlobalCell<[c_char; 80]> = GlobalCell::new([0; 80]);
    while *s != 0 {
        while *s as c_int != 0 && *s as c_int != '%' as c_int {
            s = s.offset(1);
        }
        if *s == 0 {
            break;
        }
        s = s.offset(1);
        if *s as c_int == '%' as c_int
            || *s as c_int == STL_TRUNCMARK as c_int
            || *s as c_int == STL_SEPARATE as c_int
        {
            s = s.offset(1);
        } else if *s as c_int == ')' as c_int {
            s = s.offset(1);
            groupdepth -= 1;
            if groupdepth < 0 as c_int {
                break;
            }
        } else {
            if *s as c_int == '-' as c_int {
                s = s.offset(1);
            }
            while ascii_isdigit(*s as c_int) {
                s = s.offset(1);
            }
            if *s as c_int == STL_USER_HL as c_int {
                continue;
            }
            if *s as c_int == '.' as c_int {
                s = s.offset(1);
                while *s as c_int != 0 && ascii_isdigit(*s as c_int) as c_int != 0 {
                    s = s.offset(1);
                }
            }
            if *s as c_int == '(' as c_int {
                groupdepth += 1;
            } else {
                let mut c2rust_lvalue: [c_char; 45] = [
                    STL_FILEPATH as c_int as c_char,
                    STL_FULLPATH as c_int as c_char,
                    STL_FILENAME as c_int as c_char,
                    STL_COLUMN as c_int as c_char,
                    STL_VIRTCOL as c_int as c_char,
                    STL_VIRTCOL_ALT as c_int as c_char,
                    STL_LINE as c_int as c_char,
                    STL_NUMLINES as c_int as c_char,
                    STL_BUFNO as c_int as c_char,
                    STL_KEYMAP as c_int as c_char,
                    STL_OFFSET as c_int as c_char,
                    STL_OFFSET_X as c_int as c_char,
                    STL_BYTEVAL as c_int as c_char,
                    STL_BYTEVAL_X as c_int as c_char,
                    STL_ROFLAG as c_int as c_char,
                    STL_ROFLAG_ALT as c_int as c_char,
                    STL_HELPFLAG as c_int as c_char,
                    STL_HELPFLAG_ALT as c_int as c_char,
                    STL_FILETYPE as c_int as c_char,
                    STL_FILETYPE_ALT as c_int as c_char,
                    STL_PREVIEWFLAG as c_int as c_char,
                    STL_PREVIEWFLAG_ALT as c_int as c_char,
                    STL_MODIFIED as c_int as c_char,
                    STL_MODIFIED_ALT as c_int as c_char,
                    STL_QUICKFIX as c_int as c_char,
                    STL_PERCENTAGE as c_int as c_char,
                    STL_ALTPERCENT as c_int as c_char,
                    STL_ARGLISTSTAT as c_int as c_char,
                    STL_PAGENUM as c_int as c_char,
                    STL_SHOWCMD as c_int as c_char,
                    STL_FOLDCOL as c_int as c_char,
                    STL_SIGNCOL as c_int as c_char,
                    STL_VIM_EXPR as c_int as c_char,
                    STL_SEPARATE as c_int as c_char,
                    STL_TRUNCMARK as c_int as c_char,
                    STL_USER_HL as c_int as c_char,
                    STL_HIGHLIGHT as c_int as c_char,
                    STL_HIGHLIGHT_COMB as c_int as c_char,
                    STL_TABPAGENR as c_int as c_char,
                    STL_TABCLOSENR as c_int as c_char,
                    STL_CLICK_FUNC as c_int as c_char,
                    STL_TABPAGENR as c_int as c_char,
                    STL_TABCLOSENR as c_int as c_char,
                    STL_CLICK_FUNC as c_int as c_char,
                    0 as c_char,
                ];
                if vim_strchr(
                    &raw mut c2rust_lvalue as *mut c_char,
                    *s as uint8_t as c_int,
                )
                .is_null()
                {
                    return illegal_char(
                        errbuf.ptr() as *mut c_char,
                        ::core::mem::size_of::<[c_char; 80]>(),
                        *s as uint8_t as c_int,
                    );
                }
                if *s as c_int == '{' as c_int {
                    s = s.offset(1);
                    let mut reevaluate: bool = *s as c_int == '%' as c_int;
                    if reevaluate as c_int != 0 && {
                        s = s.offset(1);
                        *s as c_int == '}' as c_int
                    } {
                        return illegal_char(
                            errbuf.ptr() as *mut c_char,
                            ::core::mem::size_of::<[c_char; 80]>(),
                            '}' as c_int,
                        );
                    }
                    while (*s as c_int != '}' as c_int
                        || reevaluate as c_int != 0
                            && *s.offset(-1 as c_int as isize) as c_int != '%' as c_int)
                        && *s as c_int != 0
                    {
                        s = s.offset(1);
                    }
                    if *s as c_int != '}' as c_int {
                        return (e_unclosed_expression_sequence.ptr() as *const _) as *const c_char;
                    }
                }
            }
        }
    }
    if groupdepth != 0 as c_int {
        return (e_unbalanced_groups.ptr() as *const _) as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn check_illegal_path_names(
    mut val: *mut c_char,
    mut flags: uint32_t,
) -> bool {
    return flags & kOptFlagNFname as c_int as uint32_t != 0
        && !strpbrk(
            val,
            if secure.get() != 0 {
                b"/\\*?[|;&<>\r\n\0".as_ptr() as *const c_char
            } else {
                b"/\\*?[<>\r\n\0".as_ptr() as *const c_char
            },
        )
        .is_null()
        || flags & kOptFlagNDname as c_int as uint32_t != 0
            && !strpbrk(val, b"*?[|;&<>\r\n\0".as_ptr() as *const c_char).is_null();
}
unsafe extern "C" fn did_set_opt_flags(
    mut val: *mut c_char,
    mut values: *mut *const c_char,
    mut flagp: *mut c_uint,
    mut list: bool,
) -> *const c_char {
    if opt_strings_flags(val, values, flagp, list) != OK {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
unsafe extern "C" fn opt_values(
    mut idx: OptIndex,
    mut values_len: *mut size_t,
) -> *mut *const c_char {
    let mut idx1: OptIndex = (if idx as c_int == kOptViewoptions as c_int {
        kOptSessionoptions as c_int
    } else if idx as c_int == kOptFileformats as c_int {
        kOptFileformat as c_int
    } else {
        idx as c_int
    }) as OptIndex;
    let mut opt: *mut vimoption_T = get_option(idx1);
    if !values_len.is_null() {
        *values_len = (*opt).values_len;
    }
    return (*opt).values;
}
unsafe extern "C" fn check_str_opt(mut idx: OptIndex, mut varp: *mut *mut c_char) -> c_int {
    let mut opt: *mut vimoption_T = get_option(idx);
    if varp.is_null() {
        varp = (*opt).var as *mut *mut c_char;
    }
    let mut list: bool =
        (*opt).flags & (kOptFlagComma as c_int | kOptFlagOneComma as c_int) as uint32_t != 0;
    let mut values: *mut *const c_char = opt_values(idx, ::core::ptr::null_mut::<size_t>());
    return opt_strings_flags(*varp, values, (*opt).flags_var, list);
}
pub unsafe extern "C" fn expand_set_str_generic(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    let mut values_len: size_t = 0;
    let mut values: *mut *const c_char = opt_values((*args).oe_idx, &raw mut values_len);
    return expand_set_opt_string(args, values, values_len, numMatches, matches);
}
pub unsafe extern "C" fn did_set_str_generic(mut args: *mut optset_T) -> *const c_char {
    return if check_str_opt((*args).os_idx, (*args).os_varp as *mut *mut c_char) != OK {
        &raw const e_invarg as *const c_char
    } else {
        ::core::ptr::null::<c_char>()
    };
}
unsafe extern "C" fn did_set_option_listflag(
    mut val: *mut c_char,
    mut flags: *mut c_char,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
) -> *const c_char {
    let mut s: *mut c_char = val;
    while *s != 0 {
        if vim_strchr(flags, *s as uint8_t as c_int).is_null() {
            return illegal_char(errbuf, errbuflen, *s as uint8_t as c_int);
        }
        s = s.offset(1);
    }
    return ::core::ptr::null::<c_char>();
}
unsafe extern "C" fn expand_set_opt_string(
    mut args: *mut optexpand_T,
    mut values: *mut *const c_char,
    mut numValues: size_t,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    let mut regmatch: *mut regmatch_T = (*args).oe_regmatch;
    let mut include_orig_val: bool = (*args).oe_include_orig_val;
    let mut option_val: *mut c_char = (*args).oe_opt_value;
    *matches = xmalloc(
        ::core::mem::size_of::<*mut c_char>().wrapping_mul(numValues.wrapping_add(1 as size_t)),
    ) as *mut *mut c_char;
    let mut count: c_int = 0 as c_int;
    if include_orig_val as c_int != 0 && *option_val as c_int != NUL {
        let c2rust_fresh0 = count;
        count = count + 1;
        let c2rust_lvalue_ptr = &raw mut *(*matches).offset(c2rust_fresh0 as isize);
        *c2rust_lvalue_ptr = xstrdup(option_val);
    }
    let mut val: *mut *const c_char = values;
    while !(*val).is_null() {
        's_27: {
            if **val as c_int != NUL {
                if include_orig_val as c_int != 0 && *option_val as c_int != NUL {
                    if strcmp(*val, option_val) == 0 as c_int {
                        break 's_27;
                    }
                }
                if vim_regexec(regmatch, *val, 0 as colnr_T) {
                    let c2rust_fresh1 = count;
                    count = count + 1;
                    let c2rust_lvalue_ptr_0 = &raw mut *(*matches).offset(c2rust_fresh1 as isize);
                    *c2rust_lvalue_ptr_0 = xstrdup(*val);
                }
            }
        }
        val = val.offset(1);
    }
    if count == 0 as c_int {
        let mut ptr_: *mut *mut c_void = matches as *mut *mut c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        return FAIL;
    }
    *numMatches = count;
    return OK;
}
static set_opt_callback_orig_option: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
static set_opt_callback_func: GlobalCell<
    Option<unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char>,
> = GlobalCell::new(None);
unsafe extern "C" fn expand_set_opt_callback(mut xp: *mut expand_T, mut idx: c_int) -> *mut c_char {
    if idx == 0 as c_int {
        if !(*set_opt_callback_orig_option.ptr()).is_null() {
            return set_opt_callback_orig_option.get();
        } else {
            return b"\0".as_ptr() as *const c_char as *mut c_char;
        }
    }
    return (*set_opt_callback_func.ptr()).expect("non-null function pointer")(
        xp,
        idx - 1 as c_int,
    );
}
unsafe extern "C" fn expand_set_opt_generic(
    mut args: *mut optexpand_T,
    mut func: CompleteListItemGetter,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    set_opt_callback_orig_option.set(if (*args).oe_include_orig_val as c_int != 0 {
        (*args).oe_opt_value
    } else {
        ::core::ptr::null_mut::<c_char>()
    });
    set_opt_callback_func
        .set(func as Option<unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char>);
    ExpandGeneric(
        b"\0".as_ptr() as *const c_char,
        (*args).oe_xp,
        (*args).oe_regmatch,
        matches,
        numMatches,
        Some(expand_set_opt_callback as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char),
        false_0 != 0,
    );
    set_opt_callback_orig_option.set(::core::ptr::null_mut::<c_char>());
    set_opt_callback_func.set(None);
    return OK;
}
unsafe extern "C" fn expand_set_opt_listflag(
    mut args: *mut optexpand_T,
    mut flags: *mut c_char,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    let mut option_val: *mut c_char = (*args).oe_opt_value;
    let mut cmdline_val: *mut c_char = (*args).oe_set_arg;
    let mut append: bool = (*args).oe_append;
    let mut include_orig_val: bool =
        (*args).oe_include_orig_val as c_int != 0 && *option_val as c_int != NUL;
    let mut num_flags: size_t = strlen(flags);
    *matches = xmalloc(
        ::core::mem::size_of::<*mut c_char>().wrapping_mul(num_flags.wrapping_add(1 as size_t)),
    ) as *mut *mut c_char;
    let mut count: c_int = 0 as c_int;
    if include_orig_val {
        let c2rust_fresh7 = count;
        count = count + 1;
        let c2rust_lvalue_ptr = &raw mut *(*matches).offset(c2rust_fresh7 as isize);
        *c2rust_lvalue_ptr = xstrdup(option_val);
    }
    let mut flag: *mut c_char = flags;
    while *flag as c_int != NUL {
        if !(append as c_int != 0 && !vim_strchr(option_val, *flag as c_int).is_null()) {
            if vim_strchr(cmdline_val, *flag as c_int).is_null() {
                if !(include_orig_val as c_int != 0
                    && *option_val.offset(1 as c_int as isize) as c_int == NUL
                    && *flag as c_int == *option_val.offset(0 as c_int as isize) as c_int)
                {
                    let c2rust_fresh8 = count;
                    count = count + 1;
                    let c2rust_lvalue_ptr_0 = &raw mut *(*matches).offset(c2rust_fresh8 as isize);
                    *c2rust_lvalue_ptr_0 =
                        xmemdupz(flag as *const c_void, 1 as size_t) as *mut c_char;
                }
            }
        }
        flag = flag.offset(1);
    }
    if count == 0 as c_int {
        let mut ptr_: *mut *mut c_void = matches as *mut *mut c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        return FAIL;
    }
    *numMatches = count;
    return OK;
}
pub unsafe extern "C" fn did_set_ambiwidth(mut args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    return check_chars_options();
}
pub unsafe extern "C" fn did_set_emoji(mut _args: *mut optset_T) -> *const c_char {
    if check_str_opt(kOptAmbiwidth, ::core::ptr::null_mut::<*mut c_char>()) != OK {
        return &raw const e_invarg as *const c_char;
    }
    return check_chars_options();
}
pub unsafe extern "C" fn did_set_background(mut args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    if *(*args).os_oldval.string.data.offset(0 as c_int as isize) as c_int == *p_bg.get() as c_int {
        return ::core::ptr::null::<c_char>();
    }
    let mut dark: c_int = (*p_bg.get() as c_int == 'd' as c_int) as c_int;
    init_highlight(false_0 != 0, false_0 != 0);
    if dark != (*p_bg.get() as c_int == 'd' as c_int) as c_int
        && !get_var_value(b"g:colors_name\0".as_ptr() as *const c_char).is_null()
    {
        do_unlet(
            b"g:colors_name\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 14]>().wrapping_sub(1 as size_t),
            true_0 != 0,
        );
        free_string_option(p_bg.get());
        p_bg.set(xstrdup(if dark != 0 {
            b"dark\0".as_ptr() as *const c_char
        } else {
            b"light\0".as_ptr() as *const c_char
        }));
        check_string_option(p_bg.ptr());
        init_highlight(false_0 != 0, false_0 != 0);
    }
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if !(*buf).terminal.is_null() {
            terminal_notify_theme((*buf).terminal, dark != 0);
        }
        buf = (*buf).b_next;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_backspace(mut args: *mut optset_T) -> *const c_char {
    if ascii_isdigit(*p_bs.get() as c_int) {
        if *p_bs.get() as c_int != '2' as c_int {
            return &raw const e_invarg as *const c_char;
        }
        return ::core::ptr::null::<c_char>();
    }
    return did_set_str_generic(args);
}
pub unsafe extern "C" fn did_set_backupcopy(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut oldval: *const c_char = (*args).os_oldval.string.data;
    let mut opt_flags: c_int = (*args).os_flags;
    let mut bkc: *mut c_char = p_bkc.get();
    let mut flags: *mut c_uint = bkc_flags.ptr();
    if opt_flags & OPT_LOCAL as c_int != 0 {
        bkc = (*buf).b_p_bkc;
        flags = &raw mut (*buf).b_bkc_flags;
    } else if opt_flags & OPT_GLOBAL as c_int == 0 {
        (*buf).b_bkc_flags = 0 as c_uint;
    }
    if opt_flags & OPT_LOCAL as c_int != 0 && *bkc as c_int == NUL {
        *flags = 0 as c_uint;
    } else {
        if opt_strings_flags(
            bkc,
            opt_bkc_values.ptr() as *mut *const c_char,
            flags,
            true_0 != 0,
        ) != OK
        {
            return &raw const e_invarg as *const c_char;
        }
        if (*flags & kOptBkcFlagAuto as c_int as c_uint != 0 as c_uint) as c_int
            + (*flags & kOptBkcFlagYes as c_int as c_uint != 0 as c_uint) as c_int
            + (*flags & kOptBkcFlagNo as c_int as c_uint != 0 as c_uint) as c_int
            != 1 as c_int
        {
            opt_strings_flags(
                oldval,
                opt_bkc_values.ptr() as *mut *const c_char,
                flags,
                true_0 != 0,
            );
            return &raw const e_invarg as *const c_char;
        }
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_backupext_or_patchmode(mut _args: *mut optset_T) -> *const c_char {
    if strcmp(
        if *p_bex.get() as c_int == '.' as c_int {
            (*p_bex.ptr()).offset(1 as c_int as isize)
        } else {
            p_bex.get()
        },
        if *p_pm.get() as c_int == '.' as c_int {
            (*p_pm.ptr()).offset(1 as c_int as isize)
        } else {
            p_pm.get()
        },
    ) == 0 as c_int
    {
        return (e_backupext_and_patchmode_are_equal.ptr() as *const _) as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_breakat(mut _args: *mut optset_T) -> *const c_char {
    let mut i: c_int = 0 as c_int;
    while i < 256 as c_int {
        (*breakat_flags.ptr())[i as usize] = false_0 as c_char;
        i += 1;
    }
    if !(*p_breakat.ptr()).is_null() {
        let mut p: *mut c_char = p_breakat.get();
        while *p != 0 {
            (*breakat_flags.ptr())[*p as uint8_t as usize] = true_0 as c_char;
            p = p.offset(1);
        }
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_breakindentopt(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if briopt_check(
        *varp,
        if varp == &raw mut (*win).w_onebuf_opt.wo_briopt {
            win
        } else {
            ::core::ptr::null_mut::<win_T>()
        },
    ) as c_int
        == FAIL
    {
        return &raw const e_invarg as *const c_char;
    }
    if varp == &raw mut (*win).w_onebuf_opt.wo_briopt && (*win).w_briopt_list != 0 {
        redraw_all_later(UPD_NOT_VALID as c_int);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_bufhidden(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    return did_set_opt_flags(
        (*buf).b_p_bh,
        opt_bh_values.ptr() as *mut *const c_char,
        ::core::ptr::null_mut::<c_uint>(),
        false_0 != 0,
    );
}
pub unsafe extern "C" fn did_set_buftype(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if !(*buf).terminal.is_null()
        && *(*buf).b_p_bt.offset(0 as c_int as isize) as c_int != 't' as c_int
        || (*buf).terminal.is_null()
            && *(*buf).b_p_bt.offset(0 as c_int as isize) as c_int == 't' as c_int
        || opt_strings_flags(
            (*buf).b_p_bt,
            opt_bt_values.ptr() as *mut *const c_char,
            ::core::ptr::null_mut::<c_uint>(),
            false_0 != 0,
        ) != OK
    {
        return &raw const e_invarg as *const c_char;
    }
    if *(*buf).b_p_bt.offset(0 as c_int as isize) as c_int == 'p' as c_int {
        set_option_direct(
            kOptComments,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: b"\0".as_ptr() as *const c_char as *mut c_char,
                        size: ::core::mem::size_of::<[c_char; 1]>().wrapping_sub(1 as size_t),
                    },
                },
            },
            OPT_LOCAL as c_int,
            SID_NONE,
        );
        let mut next_prompt: pos_T = pos_T {
            lnum: (*buf).b_ml.ml_line_count,
            col: (*buf).b_prompt_start.mark.col,
            coladd: 0 as colnr_T,
        };
        let fmarkp___: *mut fmark_T = &raw mut (*buf).b_prompt_start;
        free_fmark(*fmarkp___);
        let fmarkp__: *mut fmark_T = fmarkp___;
        (*fmarkp__).mark = next_prompt;
        (*fmarkp__).fnum = 0 as c_int;
        (*fmarkp__).timestamp = os_time();
        (*fmarkp__).view = fmarkv_T {
            topline_offset: MAXLNUM as c_int as linenr_T,
            skipcol: 0 as colnr_T,
        };
        (*fmarkp__).additional_data = ::core::ptr::null_mut::<AdditionalData>();
    }
    if (*win).w_status_height != 0 || global_stl_height() != 0 {
        (*win).w_redr_status = true_0 != 0;
        redraw_later(win, UPD_VALID as c_int);
    }
    (*buf).b_help = *(*buf).b_p_bt.offset(0 as c_int as isize) as c_int == 'h' as c_int;
    redraw_titles();
    return ::core::ptr::null::<c_char>();
}
unsafe extern "C" fn did_set_global_chars_option(
    mut win: *mut win_T,
    mut val: *mut c_char,
    mut what: CharsOption,
    mut opt_flags: c_int,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
) -> *const c_char {
    let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
    let mut local_ptr: *mut *mut c_char = if what as c_uint == kListchars as c_int as c_uint {
        &raw mut (*win).w_onebuf_opt.wo_lcs
    } else {
        &raw mut (*win).w_onebuf_opt.wo_fcs
    };
    errmsg = set_chars_option(
        win,
        val,
        what,
        **local_ptr as c_int == NUL || opt_flags & OPT_GLOBAL as c_int == 0,
        errbuf,
        errbuflen,
    );
    if !errmsg.is_null() {
        return errmsg;
    }
    if opt_flags & OPT_GLOBAL as c_int == 0 {
        clear_string_option(local_ptr);
    }
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            let mut opt: *mut c_char = if what as c_uint == kListchars as c_int as c_uint {
                (*wp).w_onebuf_opt.wo_lcs
            } else {
                (*wp).w_onebuf_opt.wo_fcs
            };
            if *opt as c_int == NUL {
                set_chars_option(wp, opt, what, true_0 != 0, errbuf, errbuflen);
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    redraw_all_later(UPD_NOT_VALID as c_int);
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_chars_option(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
    if varp == p_lcs.ptr() {
        errmsg = did_set_global_chars_option(
            win,
            *varp,
            kListchars,
            (*args).os_flags,
            (*args).os_errbuf,
            (*args).os_errbuflen,
        );
    } else if varp == p_fcs.ptr() {
        errmsg = did_set_global_chars_option(
            win,
            *varp,
            kFillchars,
            (*args).os_flags,
            (*args).os_errbuf,
            (*args).os_errbuflen,
        );
    } else if varp == &raw mut (*win).w_onebuf_opt.wo_lcs {
        errmsg = set_chars_option(
            win,
            *varp,
            kListchars,
            true_0 != 0,
            (*args).os_errbuf,
            (*args).os_errbuflen,
        );
    } else if varp == &raw mut (*win).w_onebuf_opt.wo_fcs {
        errmsg = set_chars_option(
            win,
            *varp,
            kFillchars,
            true_0 != 0,
            (*args).os_errbuf,
            (*args).os_errbuflen,
        );
    }
    return errmsg;
}
pub unsafe extern "C" fn expand_set_chars_option(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    let mut varp: *mut *mut c_char = (*args).oe_varp as *mut *mut c_char;
    let mut is_lcs: bool =
        varp == p_lcs.ptr() || varp == &raw mut (*curwin.get()).w_onebuf_opt.wo_lcs;
    return expand_set_opt_generic(
        args,
        if is_lcs as c_int != 0 {
            Some(get_listchars_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char)
        } else {
            Some(get_fillchars_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char)
        },
        numMatches,
        matches,
    );
}
pub unsafe extern "C" fn did_set_cinoptions(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    parse_cino(buf);
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_colorcolumn(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    return check_colorcolumn(
        *varp,
        if varp == &raw mut (*win).w_onebuf_opt.wo_cc {
            win
        } else {
            ::core::ptr::null_mut::<win_T>()
        },
    );
}
pub unsafe extern "C" fn did_set_comments(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut errmsg: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut s: *mut c_char = *varp;
    while *s != 0 {
        while *s as c_int != 0 && *s as c_int != ':' as c_int {
            if vim_strchr(COM_ALL.as_ptr(), *s as uint8_t as c_int).is_null()
                && !ascii_isdigit(*s as c_int)
                && *s as c_int != '-' as c_int
            {
                errmsg = illegal_char(
                    (*args).os_errbuf,
                    (*args).os_errbuflen,
                    *s as uint8_t as c_int,
                );
                break;
            } else {
                s = s.offset(1);
            }
        }
        let c2rust_fresh4 = s;
        s = s.offset(1);
        if *c2rust_fresh4 as c_int == NUL {
            errmsg = b"E524: Missing colon\0".as_ptr() as *const c_char as *mut c_char;
        } else if *s as c_int == ',' as c_int || *s as c_int == NUL {
            errmsg = b"E525: Zero length string\0".as_ptr() as *const c_char as *mut c_char;
        }
        if !errmsg.is_null() {
            break;
        }
        while *s as c_int != 0 && *s as c_int != ',' as c_int {
            if *s as c_int == '\\' as c_int && *s.offset(1 as c_int as isize) as c_int != NUL {
                s = s.offset(1);
            }
            s = s.offset(1);
        }
        s = skip_to_option_part(s);
    }
    return errmsg;
}
pub unsafe extern "C" fn did_set_commentstring(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if **varp as c_int != NUL && strstr(*varp, b"%s\0".as_ptr() as *const c_char).is_null() {
        return b"E537: 'commentstring' must be empty or contain %s\0".as_ptr() as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_complete(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut buffer: [c_char; 512] = [0; 512];
    let mut char_before: uint8_t = NUL as uint8_t;
    let mut p: *mut c_char = *varp;
    while *p != 0 {
        memset(
            &raw mut buffer as *mut c_char as *mut c_void,
            0 as c_int,
            LSIZE as c_int as size_t,
        );
        let mut buf_ptr: *mut c_char = &raw mut buffer as *mut c_char;
        let mut escape: c_int = 0 as c_int;
        while *p as c_int != 0
            && (*p as c_int != ',' as c_int || escape != 0)
            && buf_ptr
                < (&raw mut buffer as *mut c_char)
                    .offset(LSIZE as c_int as isize)
                    .offset(-(1 as c_int as isize))
        {
            if *p as c_int == '\\' as c_int
                && *p.offset(1 as c_int as isize) as c_int == ',' as c_int
            {
                escape = 1 as c_int;
                p = p.offset(1);
            } else {
                escape = 0 as c_int;
                let c2rust_fresh5 = buf_ptr;
                buf_ptr = buf_ptr.offset(1);
                *c2rust_fresh5 = *p;
            }
            p = p.offset(1);
        }
        *buf_ptr = NUL as c_char;
        if vim_strchr(
            b".wbuksid]tUfFo\0".as_ptr() as *const c_char,
            *(&raw mut buffer as *mut c_char) as uint8_t as c_int,
        )
        .is_null()
        {
            return illegal_char(
                (*args).os_errbuf,
                (*args).os_errbuflen,
                *(&raw mut buffer as *mut c_char) as uint8_t as c_int,
            );
        }
        if vim_strchr(
            b"ksF\0".as_ptr() as *const c_char,
            *(&raw mut buffer as *mut c_char) as uint8_t as c_int,
        )
        .is_null()
            && *(&raw mut buffer as *mut c_char).offset(1 as c_int as isize) as c_int != NUL
            && *(&raw mut buffer as *mut c_char).offset(1 as c_int as isize) as c_int
                != '^' as c_int
        {
            char_before = *(&raw mut buffer as *mut c_char) as uint8_t;
        } else {
            let mut t: *mut c_char = ::core::ptr::null_mut::<c_char>();
            t = vim_strchr(&raw mut buffer as *mut c_char, '^' as c_int);
            if !t.is_null() {
                let c2rust_fresh6 = t;
                t = t.offset(1);
                *c2rust_fresh6 = NUL as c_char;
                if *t == 0 {
                    char_before = '^' as uint8_t;
                } else {
                    while *t != 0 {
                        if !ascii_isdigit(*t as c_int) {
                            char_before = '^' as uint8_t;
                            break;
                        } else {
                            t = t.offset(1);
                        }
                    }
                }
            }
        }
        if char_before as c_int != NUL {
            if !(*args).os_errbuf.is_null() {
                return illegal_char_after_chr(
                    (*args).os_errbuf,
                    (*args).os_errbuflen,
                    char_before as c_int,
                );
            }
            return ::core::ptr::null::<c_char>();
        }
        while *p as c_int == ',' as c_int || *p as c_int == ' ' as c_int {
            p = p.offset(1);
        }
    }
    if set_cpt_callbacks(args) != OK {
        return illegal_char_after_chr((*args).os_errbuf, (*args).os_errbuflen, 'F' as c_int);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_completeitemalign(mut _args: *mut optset_T) -> *const c_char {
    let mut p: *mut c_char = p_cia.get();
    let mut new_cia_flags: c_uint = 0 as c_uint;
    let mut seen: [bool; 3] = [false_0 != 0, false_0 != 0, false_0 != 0];
    let mut count: c_int = 0 as c_int;
    let mut buf: [c_char; 10] = [0; 10];
    while *p != 0 {
        copy_option_part(
            &raw mut p,
            &raw mut buf as *mut c_char,
            ::core::mem::size_of::<[c_char; 10]>(),
            b",\0".as_ptr() as *const c_char as *mut c_char,
        );
        if count >= 3 as c_int {
            return &raw const e_invarg as *const c_char;
        }
        if strequal(
            &raw mut buf as *mut c_char,
            b"abbr\0".as_ptr() as *const c_char,
        ) {
            if seen[CPT_ABBR as c_int as usize] {
                return &raw const e_invarg as *const c_char;
            }
            new_cia_flags = new_cia_flags
                .wrapping_mul(10 as c_uint)
                .wrapping_add(CPT_ABBR as c_int as c_uint);
            seen[CPT_ABBR as c_int as usize] = true_0 != 0;
            count += 1;
        } else if strequal(
            &raw mut buf as *mut c_char,
            b"kind\0".as_ptr() as *const c_char,
        ) {
            if seen[CPT_KIND as c_int as usize] {
                return &raw const e_invarg as *const c_char;
            }
            new_cia_flags = new_cia_flags
                .wrapping_mul(10 as c_uint)
                .wrapping_add(CPT_KIND as c_int as c_uint);
            seen[CPT_KIND as c_int as usize] = true_0 != 0;
            count += 1;
        } else if strequal(
            &raw mut buf as *mut c_char,
            b"menu\0".as_ptr() as *const c_char,
        ) {
            if seen[CPT_MENU as c_int as usize] {
                return &raw const e_invarg as *const c_char;
            }
            new_cia_flags = new_cia_flags
                .wrapping_mul(10 as c_uint)
                .wrapping_add(CPT_MENU as c_int as c_uint);
            seen[CPT_MENU as c_int as usize] = true_0 != 0;
            count += 1;
        } else {
            return &raw const e_invarg as *const c_char;
        }
    }
    if new_cia_flags == 0 as c_uint || count != 3 as c_int {
        return &raw const e_invarg as *const c_char;
    }
    cia_flags.set(new_cia_flags);
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_completeopt(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut cot: *mut c_char = p_cot.get();
    let mut flags: *mut c_uint = cot_flags.ptr();
    if (*args).os_flags & OPT_LOCAL as c_int != 0 {
        cot = (*buf).b_p_cot;
        flags = &raw mut (*buf).b_cot_flags;
    } else if (*args).os_flags & OPT_GLOBAL as c_int == 0 {
        (*buf).b_cot_flags = 0 as c_uint;
    }
    if opt_strings_flags(
        cot,
        opt_cot_values.ptr() as *mut *const c_char,
        flags,
        true_0 != 0,
    ) != OK
    {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_concealcursor(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    return did_set_option_listflag(
        *varp,
        COCU_ALL.as_ptr() as *mut c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}
pub unsafe extern "C" fn expand_set_concealcursor(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    return expand_set_opt_listflag(args, COCU_ALL.as_ptr() as *mut c_char, numMatches, matches);
}
pub unsafe extern "C" fn did_set_cpoptions(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    return did_set_option_listflag(
        *varp,
        CPO_VI.as_ptr() as *mut c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}
pub unsafe extern "C" fn expand_set_cpoptions(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    return expand_set_opt_listflag(args, CPO_VI.as_ptr() as *mut c_char, numMatches, matches);
}
pub unsafe extern "C" fn did_set_cursorlineopt(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if **varp as c_int == NUL || fill_culopt_flags(*varp, win) != OK {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_diffanchors(mut args: *mut optset_T) -> *const c_char {
    if diffanchors_changed((*args).os_flags & OPT_LOCAL as c_int != 0) == FAIL {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_diffopt(mut _args: *mut optset_T) -> *const c_char {
    return if diffopt_changed() == FAIL {
        &raw const e_invarg as *const c_char
    } else {
        ::core::ptr::null::<c_char>()
    };
}
pub unsafe extern "C" fn expand_set_diffopt(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    let mut xp: *mut expand_T = (*args).oe_xp;
    if (*xp).xp_pattern > (*args).oe_set_arg
        && *(*xp).xp_pattern.offset(-(1 as c_int as isize)) as c_int == ':' as c_int
    {
        let algo_len: size_t = strlen(b"algorithm:\0".as_ptr() as *const c_char);
        if (*xp).xp_pattern.offset_from((*args).oe_set_arg) >= algo_len as c_int as isize
            && strncmp(
                (*xp).xp_pattern.offset(-(algo_len as isize)),
                b"algorithm:\0".as_ptr() as *const c_char,
                algo_len,
            ) == 0 as c_int
        {
            return expand_set_opt_string(
                args,
                opt_dip_algorithm_values.ptr() as *mut *const c_char,
                ::core::mem::size_of::<[*const c_char; 5]>()
                    .wrapping_div(::core::mem::size_of::<*const c_char>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*const c_char; 5]>()
                            .wrapping_rem(::core::mem::size_of::<*const c_char>())
                            == 0) as c_int as size_t,
                    )
                    .wrapping_sub(1 as size_t),
                numMatches,
                matches,
            );
        }
        let inline_len: size_t = strlen(b"inline:\0".as_ptr() as *const c_char);
        if (*xp).xp_pattern.offset_from((*args).oe_set_arg) >= inline_len as c_int as isize
            && strncmp(
                (*xp).xp_pattern.offset(-(inline_len as isize)),
                b"inline:\0".as_ptr() as *const c_char,
                inline_len,
            ) == 0 as c_int
        {
            return expand_set_opt_string(
                args,
                opt_dip_inline_values.ptr() as *mut *const c_char,
                ::core::mem::size_of::<[*const c_char; 5]>()
                    .wrapping_div(::core::mem::size_of::<*const c_char>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*const c_char; 5]>()
                            .wrapping_rem(::core::mem::size_of::<*const c_char>())
                            == 0) as c_int as size_t,
                    )
                    .wrapping_sub(1 as size_t),
                numMatches,
                matches,
            );
        }
        return FAIL;
    }
    return expand_set_str_generic(args, numMatches, matches);
}
pub unsafe extern "C" fn did_set_display(mut args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    init_chartab();
    msg_grid_validate();
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_encoding(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut opt_flags: c_int = (*args).os_flags;
    let mut gvarp: *mut *mut c_char = get_option_varp_scope_from(
        (*args).os_idx,
        OPT_GLOBAL as c_int,
        buf,
        ::core::ptr::null_mut::<win_T>(),
    ) as *mut *mut c_char;
    if gvarp == p_fenc.ptr() {
        if (*buf).b_p_ma == 0 && opt_flags != OPT_GLOBAL as c_int {
            return &raw const e_modifiable as *const c_char;
        }
        if !vim_strchr(*varp, ',' as c_int).is_null() {
            return &raw const e_invarg as *const c_char;
        }
        redraw_titles();
        ml_setflags(buf);
    }
    let mut p: *mut c_char = enc_canonize(*varp);
    xfree(*varp as *mut c_void);
    *varp = p;
    if varp == p_enc.ptr() {
        if strcmp(p_enc.get(), b"utf-8\0".as_ptr() as *const c_char) != 0 as c_int {
            return &raw const e_unsupportedoption as *const c_char;
        }
        spell_reload();
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn expand_set_encoding(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    return expand_set_opt_generic(
        args,
        Some(get_encoding_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char),
        numMatches,
        matches,
    );
}
pub unsafe extern "C" fn did_set_eventignore(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if check_ei(*varp) == FAIL {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
static expand_eiw: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
unsafe extern "C" fn get_eventignore_name(mut xp: *mut expand_T, mut idx: c_int) -> *mut c_char {
    let mut subtract: bool = *(*xp).xp_pattern as c_int == '-' as c_int;
    if !subtract && idx == 0 as c_int {
        return b"all\0".as_ptr() as *const c_char as *mut c_char;
    }
    let mut name: *mut c_char =
        get_event_name_no_group(xp, idx - 1 as c_int + subtract as c_int, expand_eiw.get());
    if name.is_null() {
        return ::core::ptr::null_mut::<c_char>();
    }
    snprintf(
        IObuff.ptr() as *mut c_char,
        IOSIZE as size_t,
        b"%s%s\0".as_ptr() as *const c_char,
        if subtract as c_int != 0 {
            b"-\0".as_ptr() as *const c_char
        } else {
            b"\0".as_ptr() as *const c_char
        },
        name,
    );
    return IObuff.ptr() as *mut c_char;
}
pub unsafe extern "C" fn expand_set_eventignore(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    expand_eiw.set((*args).oe_varp != p_ei.ptr() as *mut c_char);
    return expand_set_opt_generic(
        args,
        Some(get_eventignore_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char),
        numMatches,
        matches,
    );
}
pub unsafe extern "C" fn did_set_fileformat(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut oldval: *const c_char = (*args).os_oldval.string.data;
    let mut opt_flags: c_int = (*args).os_flags;
    if (*buf).b_p_ma == 0 && opt_flags & OPT_GLOBAL as c_int == 0 {
        return &raw const e_modifiable as *const c_char;
    }
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    redraw_titles();
    ml_setflags(buf);
    if get_fileformat(buf) == EOL_MAC || *oldval as c_int == 'm' as c_int {
        redraw_buf_later(buf, UPD_NOT_VALID as c_int);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn get_fileformat_name(
    mut _xp: *mut expand_T,
    mut idx: c_int,
) -> *mut c_char {
    if idx
        >= ::core::mem::size_of::<[*const c_char; 4]>()
            .wrapping_div(::core::mem::size_of::<*const c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const c_char; 4]>()
                    .wrapping_rem(::core::mem::size_of::<*const c_char>())
                    == 0) as c_int as usize,
            ) as c_int
    {
        return ::core::ptr::null_mut::<c_char>();
    }
    return (*opt_ff_values.ptr())[idx as usize] as *mut c_char;
}
pub unsafe extern "C" fn did_set_filetype_or_syntax(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if !valid_filetype(*varp) {
        return &raw const e_invarg as *const c_char;
    }
    (*args).os_value_changed = strcmp((*args).os_oldval.string.data, *varp) != 0 as c_int;
    (*args).os_value_checked = true_0 != 0;
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_foldexpr(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    did_set_optexpr(args);
    if foldmethodIsExpr(win) {
        foldUpdateAll(win);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_foldignore(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if foldmethodIsIndent(win) {
        foldUpdateAll(win);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_foldmarker(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut p: *mut c_char = vim_strchr(*varp, ',' as c_int);
    if p.is_null() {
        return (e_comma_required.ptr() as *const _) as *const c_char;
    }
    if p == *varp || *p.offset(1 as c_int as isize) as c_int == NUL {
        return &raw const e_invarg as *const c_char;
    }
    if foldmethodIsMarker(win) {
        foldUpdateAll(win);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_foldmethod(mut args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    foldUpdateAll(win);
    if foldmethodIsDiff(win) {
        newFoldLevel();
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_formatoptions(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    return did_set_option_listflag(
        *varp,
        FO_ALL.as_ptr() as *mut c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}
pub unsafe extern "C" fn expand_set_formatoptions(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    return expand_set_opt_listflag(args, FO_ALL.as_ptr() as *mut c_char, numMatches, matches);
}
pub unsafe extern "C" fn did_set_guicursor(mut _args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = parse_shape_opt(SHAPE_CURSOR);
    if !errmsg.is_null() {
        return errmsg;
    }
    if VIsual_active.get() {
        redrawWinline(curwin.get(), (*curwin.get()).w_cursor.lnum);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_helpfile(mut _args: *mut optset_T) -> *const c_char {
    if didset_vim.get() {
        vim_unsetenv_ext(b"VIM\0".as_ptr() as *const c_char);
    }
    if didset_vimruntime.get() {
        vim_unsetenv_ext(b"VIMRUNTIME\0".as_ptr() as *const c_char);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_helplang(mut _args: *mut optset_T) -> *const c_char {
    let mut s: *mut c_char = p_hlg.get();
    while *s as c_int != NUL {
        if *s.offset(1 as c_int as isize) as c_int == NUL
            || (*s.offset(2 as c_int as isize) as c_int != ',' as c_int
                || *s.offset(3 as c_int as isize) as c_int == NUL)
                && *s.offset(2 as c_int as isize) as c_int != NUL
        {
            return &raw const e_invarg as *const c_char;
        }
        if *s.offset(2 as c_int as isize) as c_int == NUL {
            break;
        }
        s = s.offset(3 as c_int as isize);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_highlight(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if strcmp(*varp, HIGHLIGHT_INIT.as_ptr()) != 0 as c_int {
        return &raw const e_unsupportedoption as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_iconstring(mut args: *mut optset_T) -> *const c_char {
    return did_set_titleiconstring(args, STL_IN_ICON);
}
pub unsafe extern "C" fn did_set_inccommand(mut args: *mut optset_T) -> *const c_char {
    if cmdpreview.get() {
        return &raw const e_invarg as *const c_char;
    }
    return did_set_str_generic(args);
}
pub unsafe extern "C" fn did_set_iskeyword(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if varp == p_isk.ptr() {
        if check_isopt(*varp) == FAIL {
            return &raw const e_invarg as *const c_char;
        }
    } else {
        return did_set_isopt(args);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_isopt(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    if !buf_init_chartab(buf, true) {
        (*args).os_restore_chartab = true_0 != 0;
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_keymap(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut opt_flags: c_int = (*args).os_flags;
    if !valid_filetype(*varp) {
        return &raw const e_invarg as *const c_char;
    }
    let mut secure_save: c_int = secure.get();
    secure.set(0 as c_int);
    let mut errmsg: *const c_char = keymap_init();
    secure.set(secure_save);
    (*args).os_value_checked = true_0 != 0;
    if errmsg.is_null() {
        if *(*buf).b_p_keymap as c_int != NUL {
            (*buf).b_p_iminsert = B_IMODE_LMAP as OptInt;
            if (*buf).b_p_imsearch != B_IMODE_USE_INSERT as OptInt {
                (*buf).b_p_imsearch = B_IMODE_LMAP as OptInt;
            }
        } else {
            if (*buf).b_p_iminsert == B_IMODE_LMAP as OptInt {
                (*buf).b_p_iminsert = B_IMODE_NONE as OptInt;
            }
            if (*buf).b_p_imsearch == B_IMODE_LMAP as OptInt {
                (*buf).b_p_imsearch = B_IMODE_USE_INSERT as OptInt;
            }
        }
        if opt_flags & OPT_LOCAL as c_int == 0 as c_int {
            set_iminsert_global(buf);
            set_imsearch_global(buf);
        }
        status_redraw_buf(buf);
    }
    return errmsg;
}
pub unsafe extern "C" fn did_set_keymodel(mut args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    km_stopsel.set(!vim_strchr(p_km.get(), 'o' as c_int).is_null());
    km_startsel.set(!vim_strchr(p_km.get(), 'a' as c_int).is_null());
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_lispoptions(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if **varp as c_int != NUL
        && strcmp(*varp, b"expr:0\0".as_ptr() as *const c_char) != 0 as c_int
        && strcmp(*varp, b"expr:1\0".as_ptr() as *const c_char) != 0 as c_int
    {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_matchpairs(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut p: *mut c_char = *varp;
    while *p as c_int != NUL {
        let mut x2: c_int = -1 as c_int;
        let mut x3: c_int = -1 as c_int;
        p = p.offset(utfc_ptr2len(p) as isize);
        if *p as c_int != NUL {
            let c2rust_fresh9 = p;
            p = p.offset(1);
            x2 = *c2rust_fresh9 as c_uchar as c_int;
        }
        if *p as c_int != NUL {
            x3 = utf_ptr2char(p);
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        if x2 != ':' as c_int
            || x3 == -1 as c_int
            || *p as c_int != NUL && *p as c_int != ',' as c_int
        {
            return &raw const e_invarg as *const c_char;
        }
        if *p as c_int == NUL {
            break;
        }
        p = p.offset(1);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_messagesopt(mut _args: *mut optset_T) -> *const c_char {
    if messagesopt_changed() == FAIL {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_mkspellmem(mut _args: *mut optset_T) -> *const c_char {
    if spell_check_msm() != OK {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_mouse(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    return did_set_option_listflag(
        *varp,
        MOUSE_ALL.as_ptr() as *mut c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}
pub unsafe extern "C" fn expand_set_mouse(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    return expand_set_opt_listflag(args, MOUSE_ALL.as_ptr() as *mut c_char, numMatches, matches);
}
pub unsafe extern "C" fn did_set_mousescroll(mut _args: *mut optset_T) -> *const c_char {
    let mut vertical: OptInt = -1 as OptInt;
    let mut horizontal: OptInt = -1 as OptInt;
    let mut string: *mut c_char = p_mousescroll.get();
    loop {
        let mut end: *mut c_char = vim_strchr(string, ',' as c_int);
        let mut length: size_t = if !end.is_null() {
            end.offset_from(string) as size_t
        } else {
            strlen(string)
        };
        if length <= 4 as size_t {
            return &raw const e_invarg as *const c_char;
        }
        let mut direction: *mut OptInt = ::core::ptr::null_mut::<OptInt>();
        if memcmp(
            string as *const c_void,
            b"ver:\0".as_ptr() as *const c_char as *const c_void,
            4 as size_t,
        ) == 0 as c_int
        {
            direction = &raw mut vertical;
        } else if memcmp(
            string as *const c_void,
            b"hor:\0".as_ptr() as *const c_char as *const c_void,
            4 as size_t,
        ) == 0 as c_int
        {
            direction = &raw mut horizontal;
        } else {
            return &raw const e_invarg as *const c_char;
        }
        if *direction != -1 as OptInt {
            return &raw const e_invarg as *const c_char;
        }
        let mut i: size_t = 4 as size_t;
        while i < length {
            if !ascii_isdigit(*string.offset(i as isize) as c_int) {
                return b"E5080: Digit expected\0".as_ptr() as *const c_char;
            }
            i = i.wrapping_add(1);
        }
        string = string.offset(4 as c_int as isize);
        *direction = getdigits_int(&raw mut string, false_0 != 0, -1 as c_int) as OptInt;
        if *direction == -1 as OptInt {
            return &raw const e_invarg as *const c_char;
        }
        if end.is_null() {
            break;
        }
        string = end.offset(1 as c_int as isize);
    }
    p_mousescroll_vert.set(if vertical == -1 as OptInt {
        MOUSESCROLL_VERT_DFLT as OptInt
    } else {
        vertical
    });
    p_mousescroll_hor.set(if horizontal == -1 as OptInt {
        MOUSESCROLL_HOR_DFLT as OptInt
    } else {
        horizontal
    });
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_optexpr(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut name: *mut c_char = get_scriptlocal_funcname(*varp);
    if !name.is_null() {
        free_string_option(*varp);
        *varp = name;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_rulerformat(mut args: *mut optset_T) -> *const c_char {
    return did_set_statustabline_rulerformat(args, true_0 != 0, false_0 != 0);
}
pub unsafe extern "C" fn did_set_selection(mut args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    if VIsual_active.get() {
        redraw_curbuf_later(UPD_INVERTED as c_int);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_sessionoptions(mut args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    if ssop_flags.get() & kOptSsopFlagCurdir as c_int as c_uint != 0
        && ssop_flags.get() & kOptSsopFlagSesdir as c_int as c_uint != 0
    {
        let mut oldval: *const c_char = (*args).os_oldval.string.data;
        opt_strings_flags(
            oldval,
            opt_ssop_values.ptr() as *mut *const c_char,
            ssop_flags.ptr(),
            true_0 != 0,
        );
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_shada(mut args: *mut optset_T) -> *const c_char {
    let mut errbuf: *mut c_char = (*args).os_errbuf;
    let mut errbuflen: size_t = (*args).os_errbuflen;
    let mut s: *mut c_char = p_shada.get();
    while *s != 0 {
        if vim_strchr(
            b"!\"%'/:<@cfhnrs\0".as_ptr() as *const c_char,
            *s as uint8_t as c_int,
        )
        .is_null()
        {
            return illegal_char(errbuf, errbuflen, *s as uint8_t as c_int);
        }
        if *s as c_int == 'n' as c_int {
            break;
        }
        if *s as c_int == 'r' as c_int {
            loop {
                s = s.offset(1);
                if !(*s as c_int != 0 && *s as c_int != ',' as c_int) {
                    break;
                }
            }
        } else if *s as c_int == '%' as c_int {
            loop {
                s = s.offset(1);
                if !ascii_isdigit(*s as c_int) {
                    break;
                }
            }
        } else if *s as c_int == '!' as c_int
            || *s as c_int == 'h' as c_int
            || *s as c_int == 'c' as c_int
        {
            s = s.offset(1);
        } else {
            loop {
                s = s.offset(1);
                if !ascii_isdigit(*s as c_int) {
                    break;
                }
            }
            if !ascii_isdigit(*s.offset(-(1 as c_int as isize)) as c_int) {
                if !errbuf.is_null() {
                    vim_snprintf(
                        errbuf,
                        errbuflen,
                        gettext(b"E526: Missing number after <%s>\0".as_ptr() as *const c_char),
                        transchar_byte(*s.offset(-(1 as c_int as isize)) as uint8_t as c_int),
                    );
                    return errbuf;
                } else {
                    return b"\0".as_ptr() as *const c_char;
                }
            }
        }
        if *s as c_int == ',' as c_int {
            s = s.offset(1);
        } else if *s != 0 {
            if !errbuf.is_null() {
                return b"E527: Missing comma\0".as_ptr() as *const c_char;
            } else {
                return b"\0".as_ptr() as *const c_char;
            }
        }
    }
    if *p_shada.get() as c_int != 0 && get_shada_parameter('\'' as c_int) < 0 as c_int {
        return b"E528: Must specify a ' value\0".as_ptr() as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_shellpipe_redir(mut args: *mut optset_T) -> *const c_char {
    let mut seen: bool = false_0 != 0;
    let mut p: *mut c_char = (*args).os_newval.string.data;
    while *p as c_int != NUL {
        if *p as c_int == '%' as c_int {
            if *p.offset(1 as c_int as isize) as c_int == NUL {
                return &raw const e_invalid_format_string_single_percent_s as *const c_char;
            }
            if *p.offset(1 as c_int as isize) as c_int == '%' as c_int {
                p = p.offset(1);
            } else if *p.offset(1 as c_int as isize) as c_int == 's' as c_int {
                if seen {
                    return &raw const e_invalid_format_string_single_percent_s as *const c_char;
                }
                seen = true_0 != 0;
                p = p.offset(1);
            } else {
                return &raw const e_invalid_format_string_single_percent_s as *const c_char;
            }
        }
        p = p.offset(1);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_shortmess(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    return did_set_option_listflag(
        *varp,
        SHM_ALL.ptr() as *mut c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}
pub unsafe extern "C" fn expand_set_shortmess(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    return expand_set_opt_listflag(args, SHM_ALL.ptr() as *mut c_char, numMatches, matches);
}
pub unsafe extern "C" fn did_set_showbreak(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut s: *mut c_char = *varp;
    while *s != 0 {
        if ptr2cells(s) != 1 as c_int {
            return (e_showbreak_contains_unprintable_or_wide_character.ptr() as *const _)
                as *const c_char;
        }
        s = s.offset(utfc_ptr2len(s) as isize);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_showcmdloc(mut args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if errmsg.is_null() {
        comp_col();
    }
    return errmsg;
}
pub unsafe extern "C" fn did_set_signcolumn(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut oldval: *const c_char = (*args).os_oldval.string.data;
    if check_signcolumn(
        *varp,
        if varp == &raw mut (*win).w_onebuf_opt.wo_scl {
            win
        } else {
            ::core::ptr::null_mut::<win_T>()
        },
    ) != OK
    {
        return &raw const e_invarg as *const c_char;
    }
    if *oldval as c_int == 'n' as c_int
        && *oldval.offset(1 as c_int as isize) as c_int == 'u' as c_int
        || (*win).w_minscwidth == SCL_NUM
    {
        (*win).w_nrwidth_line_count = 0 as c_int as linenr_T;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_spellcapcheck(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    return compile_cap_prog((*win).w_s);
}
pub unsafe extern "C" fn did_set_spellfile(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if !valid_spellfile(*varp) {
        return &raw const e_invarg as *const c_char;
    }
    return did_set_spell_option();
}
pub unsafe extern "C" fn did_set_spelllang(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if !valid_spelllang(*varp) {
        return &raw const e_invarg as *const c_char;
    }
    return did_set_spell_option();
}
pub unsafe extern "C" fn did_set_spelloptions(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut opt_flags: c_int = (*args).os_flags;
    let mut val: *const c_char = (*args).os_newval.string.data;
    if opt_flags & OPT_LOCAL as c_int == 0
        && opt_strings_flags(
            val,
            opt_spo_values.ptr() as *mut *const c_char,
            spo_flags.ptr(),
            true_0 != 0,
        ) != OK
    {
        return &raw const e_invarg as *const c_char;
    }
    if opt_flags & OPT_GLOBAL as c_int == 0
        && opt_strings_flags(
            val,
            opt_spo_values.ptr() as *mut *const c_char,
            &raw mut (*(*win).w_s).b_p_spo_flags,
            true_0 != 0,
        ) != OK
    {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_spellsuggest(mut _args: *mut optset_T) -> *const c_char {
    if spell_check_sps() != OK {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_statuscolumn(mut args: *mut optset_T) -> *const c_char {
    return did_set_statustabline_rulerformat(args, false_0 != 0, true_0 != 0);
}
pub unsafe extern "C" fn did_set_statusline(mut args: *mut optset_T) -> *const c_char {
    return did_set_statustabline_rulerformat(args, false_0 != 0, false_0 != 0);
}
unsafe extern "C" fn did_set_statustabline_rulerformat(
    mut args: *mut optset_T,
    mut rulerformat: bool,
    mut statuscolumn: bool,
) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if rulerformat {
        ru_wid.set(0 as c_int);
    } else if statuscolumn {
        (*win).w_nrwidth_line_count = 0 as c_int as linenr_T;
    }
    let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
    let mut s: *mut c_char = *varp;
    let mut is_stl: bool = (*args).os_idx as c_int == kOptStatusline as c_int;
    if is_stl as c_int != 0
        && ((*args).os_flags & OPT_GLOBAL as c_int != 0
            || (*args).os_flags & OPT_LOCAL as c_int == 0)
        && *s.offset(0 as c_int as isize) as c_int == NUL
    {
        xfree(*varp as *mut c_void);
        *varp = xstrdup(
            get_option_default((*args).os_idx, (*args).os_flags)
                .data
                .string
                .data,
        );
        s = *varp;
    }
    if is_stl as c_int != 0 && !win.is_null() && (*win).w_floating as c_int != 0 {
        win_config_float(win, (*win).w_config);
    }
    if rulerformat as c_int != 0 && *s as c_int == '%' as c_int {
        s = s.offset(1);
        if *s as c_int == '-' as c_int {
            s = s.offset(1);
        }
        let mut wid: c_int = getdigits_int(&raw mut s, true_0 != 0, 0 as c_int);
        if wid != 0 && *s as c_int == '(' as c_int && {
            errmsg = check_stl_option(p_ruf.get());
            errmsg.is_null()
        } {
            ru_wid.set(wid);
        } else if *(*varp).offset(1 as c_int as isize) as c_int != '!' as c_int {
            errmsg = check_stl_option(p_ruf.get());
        }
    } else if rulerformat as c_int != 0
        || *s.offset(0 as c_int as isize) as c_int != '%' as c_int
        || *s.offset(1 as c_int as isize) as c_int != '!' as c_int
    {
        errmsg = check_stl_option(s);
    }
    if rulerformat as c_int != 0 && errmsg.is_null() {
        comp_col();
    }
    return errmsg;
}
pub unsafe extern "C" fn did_set_tabline(mut args: *mut optset_T) -> *const c_char {
    return did_set_statustabline_rulerformat(args, false_0 != 0, false_0 != 0);
}
pub unsafe extern "C" fn did_set_tagcase(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut opt_flags: c_int = (*args).os_flags;
    let mut flags: *mut c_uint = ::core::ptr::null_mut::<c_uint>();
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if opt_flags & OPT_LOCAL as c_int != 0 {
        p = (*buf).b_p_tc;
        flags = &raw mut (*buf).b_tc_flags;
    } else {
        p = p_tc.get();
        flags = tc_flags.ptr();
    }
    if opt_flags & OPT_LOCAL as c_int != 0 && *p as c_int == NUL {
        *flags = 0 as c_uint;
    } else if opt_strings_flags(
        p,
        opt_tc_values.ptr() as *mut *const c_char,
        flags,
        false_0 != 0,
    ) != OK
    {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
unsafe extern "C" fn did_set_titleiconstring(
    mut args: *mut optset_T,
    mut flagval: c_int,
) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if !vim_strchr(*varp, '%' as c_int).is_null() && check_stl_option(*varp).is_null() {
        (*stl_syntax.ptr()) |= flagval;
    } else {
        (*stl_syntax.ptr()) &= !flagval;
    }
    did_set_title();
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_titlestring(mut args: *mut optset_T) -> *const c_char {
    return did_set_titleiconstring(args, STL_IN_TITLE);
}
pub unsafe extern "C" fn did_set_varsofttabstop(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if *(*varp).offset(0 as c_int as isize) == 0
        || *(*varp).offset(0 as c_int as isize) as c_int == '0' as c_int
            && *(*varp).offset(1 as c_int as isize) == 0
    {
        let mut ptr_: *mut *mut c_void = &raw mut (*buf).b_p_vsts_array as *mut *mut c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        return ::core::ptr::null::<c_char>();
    }
    let mut cp: *mut c_char = *varp;
    while *cp != 0 {
        if !ascii_isdigit(*cp as c_int) {
            if !(*cp as c_int == ',' as c_int
                && cp > *varp
                && *cp.offset(-(1 as c_int as isize)) as c_int != ',' as c_int)
            {
                return &raw const e_invarg as *const c_char;
            }
        }
        cp = cp.offset(1);
    }
    let mut oldarray: *mut colnr_T = (*buf).b_p_vsts_array;
    if tabstop_set(*varp, &raw mut (*buf).b_p_vsts_array) {
        xfree(oldarray as *mut c_void);
    } else {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_vartabstop(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if *(*varp).offset(0 as c_int as isize) == 0
        || *(*varp).offset(0 as c_int as isize) as c_int == '0' as c_int
            && *(*varp).offset(1 as c_int as isize) == 0
    {
        let mut ptr_: *mut *mut c_void = &raw mut (*buf).b_p_vts_array as *mut *mut c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        return ::core::ptr::null::<c_char>();
    }
    let mut cp: *mut c_char = *varp;
    while *cp != 0 {
        if !ascii_isdigit(*cp as c_int) {
            if !(*cp as c_int == ',' as c_int
                && cp > *varp
                && *cp.offset(-(1 as c_int as isize)) as c_int != ',' as c_int)
            {
                return &raw const e_invarg as *const c_char;
            }
        }
        cp = cp.offset(1);
    }
    let mut oldarray: *mut colnr_T = (*buf).b_p_vts_array;
    if tabstop_set(*varp, &raw mut (*buf).b_p_vts_array) {
        xfree(oldarray as *mut c_void);
        if foldmethodIsIndent(win) {
            foldUpdateAll(win);
        }
    } else {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_verbosefile(mut _args: *mut optset_T) -> *const c_char {
    verbose_stop();
    if *p_vfile.get() as c_int != NUL && verbose_open() == FAIL {
        return &raw const e_invarg as *const c_char as *mut c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_virtualedit(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut ve: *mut c_char = p_ve.get();
    let mut flags: *mut c_uint = ve_flags.ptr();
    if (*args).os_flags & OPT_LOCAL as c_int != 0 {
        ve = (*win).w_onebuf_opt.wo_ve;
        flags = &raw mut (*win).w_onebuf_opt.wo_ve_flags;
    }
    if (*args).os_flags & OPT_LOCAL as c_int != 0 && *ve as c_int == NUL {
        *flags = 0 as c_uint;
    } else if opt_strings_flags(
        ve,
        opt_ve_values.ptr() as *mut *const c_char,
        flags,
        true_0 != 0,
    ) != OK
    {
        return &raw const e_invarg as *const c_char;
    } else if strcmp(ve, (*args).os_oldval.string.data) != 0 as c_int {
        validate_virtcol(win);
        coladvance(win, (*win).w_virtcol);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_whichwrap(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    return did_set_option_listflag(
        *varp,
        b"bshl<>[]~,\0".as_ptr() as *const c_char as *mut c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}
pub unsafe extern "C" fn expand_set_whichwrap(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    return expand_set_opt_listflag(args, WW_ALL.as_ptr() as *mut c_char, numMatches, matches);
}
pub unsafe extern "C" fn did_set_wildmode(mut _args: *mut optset_T) -> *const c_char {
    if check_opt_wim() == FAIL {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_winbar(mut args: *mut optset_T) -> *const c_char {
    return did_set_statustabline_rulerformat(args, false_0 != 0, false_0 != 0);
}
unsafe extern "C" fn parse_border_opt(mut border_opt: *mut c_char) -> bool {
    let mut fconfig: WinConfig = WinConfig {
        window: 0,
        bufpos: lpos_T {
            lnum: -1 as linenr_T,
            col: 0 as colnr_T,
        },
        height: 0 as c_int,
        width: 0 as c_int,
        row: 0 as c_int as c_double,
        col: 0 as c_int as c_double,
        anchor: 0 as FloatAnchor,
        relative: kFloatRelativeEditor,
        external: false_0 != 0,
        focusable: true_0 != 0,
        mouse: true_0 != 0,
        split: kWinSplitLeft,
        zindex: kZIndexFloatDefault as c_int,
        style: kWinStyleUnused,
        border: false,
        shadow: false,
        border_chars: [[0; 32]; 8],
        border_hl_ids: [0; 8],
        border_attr: [0; 8],
        title: false,
        title_pos: kAlignLeft,
        title_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        title_width: 0,
        footer: false,
        footer_pos: kAlignLeft,
        footer_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        footer_width: 0,
        noautocmd: false_0 != 0,
        fixed: false_0 != 0,
        hide: false_0 != 0,
        _cmdline_offset: INT_MAX,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<c_char>(),
    };
    let mut result: bool = true_0 != 0;
    if !parse_winborder(&raw mut fconfig, border_opt, &raw mut err) {
        result = false_0 != 0;
    }
    api_clear_error(&raw mut err);
    return result;
}
pub unsafe extern "C" fn did_set_winborder(mut _args: *mut optset_T) -> *const c_char {
    if !parse_border_opt(p_winborder.get()) {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_pumborder(mut _args: *mut optset_T) -> *const c_char {
    if !parse_border_opt(p_pumborder.get()) {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_winhighlight(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if !parse_winhl_opt(
        *varp,
        if varp == &raw mut (*win).w_onebuf_opt.wo_winhl {
            win
        } else {
            ::core::ptr::null_mut::<win_T>()
        },
    ) {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn expand_set_winhighlight(
    mut args: *mut optexpand_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    return expand_set_opt_generic(
        args,
        Some(get_highlight_name as unsafe extern "C" fn(*mut expand_T, c_int) -> *mut c_char),
        numMatches,
        matches,
    );
}
unsafe extern "C" fn opt_strings_flags(
    mut val: *const c_char,
    mut values: *mut *const c_char,
    mut flagp: *mut c_uint,
    mut list: bool,
) -> c_int {
    let mut new_flags: c_uint = 0 as c_uint;
    let mut iter_one: bool = *val as c_int == NUL && !list;
    while *val as c_int != 0 || iter_one as c_int != 0 {
        let mut i: c_uint = 0 as c_uint;
        loop {
            if (*values.offset(i as isize)).is_null() {
                return FAIL;
            }
            let mut len: size_t = strlen(*values.offset(i as isize));
            if strncmp(*values.offset(i as isize), val, len) == 0 as c_int
                && (list as c_int != 0 && *val.offset(len as isize) as c_int == ',' as c_int
                    || *val.offset(len as isize) as c_int == NUL)
            {
                val = val.offset(len.wrapping_add(
                    (*val.offset(len as isize) as c_int == ',' as c_int) as c_int as size_t,
                ) as isize);
                '_c2rust_label: {
                    if (i as usize) < ::core::mem::size_of::<c_uint>().wrapping_mul(8 as usize) {
                    } else {
                        __assert_fail(
                            b"i < sizeof(new_flags) * 8\0".as_ptr() as *const c_char,
                            b"src/nvim/optionstr.rs\0".as_ptr() as *const c_char,
                            2192 as c_uint,
                            __ASSERT_FUNCTION.as_ptr(),
                        );
                    }
                };
                new_flags |= (1 as c_uint) << i;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
        if iter_one {
            break;
        }
    }
    if !flagp.is_null() {
        *flagp = new_flags;
    }
    return OK;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn check_ff_value(mut p: *mut c_char) -> c_int {
    return opt_strings_flags(
        p,
        opt_ff_values.ptr() as *mut *const c_char,
        ::core::ptr::null_mut::<c_uint>(),
        false_0 != 0,
    );
}
static e_conflicts_with_value_of_listchars: GlobalCell<[c_char; 42]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 42], [c_char; 42]>(
        *b"E834: Conflicts with value of 'listchars'\0",
    )
});
static e_conflicts_with_value_of_fillchars: GlobalCell<[c_char; 42]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 42], [c_char; 42]>(
        *b"E835: Conflicts with value of 'fillchars'\0",
    )
});
unsafe extern "C" fn get_encoded_char_adv(mut p: *mut *const c_char) -> schar_T {
    let mut s: *const c_char = *p;
    if *s.offset(0 as c_int as isize) as c_int == '\\' as c_int
        && (*s.offset(1 as c_int as isize) as c_int == 'x' as c_int
            || *s.offset(1 as c_int as isize) as c_int == 'u' as c_int
            || *s.offset(1 as c_int as isize) as c_int == 'U' as c_int)
    {
        let mut num: int64_t = 0 as int64_t;
        let mut bytes: c_int = if *s.offset(1 as c_int as isize) as c_int == 'x' as c_int {
            1 as c_int
        } else if *s.offset(1 as c_int as isize) as c_int == 'u' as c_int {
            2 as c_int
        } else {
            4 as c_int
        };
        while bytes > 0 as c_int {
            *p = (*p).offset(2 as c_int as isize);
            let mut n: c_int = hexhex2nr(*p);
            if n < 0 as c_int {
                return 0 as schar_T;
            }
            num = num * 256 as int64_t + n as int64_t;
            bytes -= 1;
        }
        *p = (*p).offset(2 as c_int as isize);
        return if char2cells(num as c_int) > 1 as c_int {
            0 as schar_T
        } else {
            schar_from_char(num as c_int)
        };
    }
    let mut clen: c_int = utfc_ptr2len(s);
    let mut firstc: c_int = 0;
    let mut c: schar_T = utfc_ptr2schar(s, &raw mut firstc);
    *p = (*p).offset(clen as isize);
    return if clen == 1 as c_int && firstc > 127 as c_int || char2cells(firstc) > 1 as c_int {
        0 as schar_T
    } else {
        c
    };
}
static fcs_chars: GlobalCell<fcs_chars_T> = GlobalCell::new(fcs_chars_T {
    stl: 0,
    stlnc: 0,
    wbr: 0,
    horiz: 0,
    horizup: 0,
    horizdown: 0,
    vert: 0,
    vertleft: 0,
    vertright: 0,
    verthoriz: 0,
    fold: 0,
    foldopen: 0,
    foldclosed: 0,
    foldsep: 0,
    foldinner: 0,
    diff: 0,
    msgsep: 0,
    eob: 0,
    lastline: 0,
    trunc: 0,
    truncrl: 0,
});
static fcs_tab: GlobalCell<[chars_tab; 21]> = GlobalCell::new(
    [chars_tab {
        cp: ::core::ptr::null_mut::<schar_T>(),
        name: String_0 {
            data: ::core::ptr::null_mut::<c_char>(),
            size: 0,
        },
        def: ::core::ptr::null::<c_char>(),
        fallback: ::core::ptr::null::<c_char>(),
    }; 21],
);
static lcs_chars: GlobalCell<lcs_chars_T> = GlobalCell::new(lcs_chars_T {
    eol: 0,
    ext: 0,
    prec: 0,
    nbsp: 0,
    space: 0,
    tab1: 0,
    tab2: 0,
    tab3: 0,
    leadtab1: 0,
    leadtab2: 0,
    leadtab3: 0,
    lead: 0,
    trail: 0,
    multispace: ::core::ptr::null_mut::<schar_T>(),
    leadmultispace: ::core::ptr::null_mut::<schar_T>(),
    conceal: 0,
});
static lcs_tab: GlobalCell<[chars_tab; 12]> = GlobalCell::new(
    [chars_tab {
        cp: ::core::ptr::null_mut::<schar_T>(),
        name: String_0 {
            data: ::core::ptr::null_mut::<c_char>(),
            size: 0,
        },
        def: ::core::ptr::null::<c_char>(),
        fallback: ::core::ptr::null::<c_char>(),
    }; 12],
);
unsafe extern "C" fn field_value_err(
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
    mut fmt: *const c_char,
    mut field: *const c_char,
) -> *mut c_char {
    if errbuf.is_null() {
        return b"\0".as_ptr() as *const c_char as *mut c_char;
    }
    vim_snprintf(errbuf, errbuflen, gettext(fmt), field);
    return errbuf;
}
pub unsafe extern "C" fn set_chars_option(
    mut wp: *mut win_T,
    mut value: *const c_char,
    mut what: CharsOption,
    mut apply: bool,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
) -> *const c_char {
    let mut last_multispace: *const c_char = ::core::ptr::null::<c_char>();
    let mut last_lmultispace: *const c_char = ::core::ptr::null::<c_char>();
    let mut multispace_len: c_int = 0 as c_int;
    let mut lead_multispace_len: c_int = 0 as c_int;
    let mut tab: *const chars_tab = ::core::ptr::null::<chars_tab>();
    let mut entries: c_int = 0;
    if what as c_uint == kListchars as c_int as c_uint {
        tab = (lcs_tab.ptr() as *const _) as *const chars_tab;
        entries = ::core::mem::size_of::<[chars_tab; 12]>()
            .wrapping_div(::core::mem::size_of::<chars_tab>())
            .wrapping_div(
                (::core::mem::size_of::<[chars_tab; 12]>()
                    .wrapping_rem(::core::mem::size_of::<chars_tab>())
                    == 0) as c_int as usize,
            ) as c_int;
        if *(*wp).w_onebuf_opt.wo_lcs.offset(0 as c_int as isize) as c_int == NUL {
            value = p_lcs.get();
        }
    } else {
        tab = (fcs_tab.ptr() as *const _) as *const chars_tab;
        entries = ::core::mem::size_of::<[chars_tab; 21]>()
            .wrapping_div(::core::mem::size_of::<chars_tab>())
            .wrapping_div(
                (::core::mem::size_of::<[chars_tab; 21]>()
                    .wrapping_rem(::core::mem::size_of::<chars_tab>())
                    == 0) as c_int as usize,
            ) as c_int;
        if *(*wp).w_onebuf_opt.wo_fcs.offset(0 as c_int as isize) as c_int == NUL {
            value = p_fcs.get();
        }
    }
    let mut round: c_int = 0 as c_int;
    while round
        <= (if apply as c_int != 0 {
            1 as c_int
        } else {
            0 as c_int
        })
    {
        let mut has_tab: bool = false_0 != 0;
        let mut has_leadtab: bool = false_0 != 0;
        if round > 0 as c_int {
            let mut i: c_int = 0 as c_int;
            while i < entries {
                if !(*tab.offset(i as isize)).cp.is_null() {
                    *(*tab.offset(i as isize)).cp = schar_from_str(
                        if !(*tab.offset(i as isize)).def.is_null()
                            && ptr2cells((*tab.offset(i as isize)).def) == 1 as c_int
                        {
                            (*tab.offset(i as isize)).def
                        } else {
                            (*tab.offset(i as isize)).fallback
                        },
                    );
                }
                i += 1;
            }
            if what as c_uint == kListchars as c_int as c_uint {
                (*lcs_chars.ptr()).tab1 = NUL as schar_T;
                (*lcs_chars.ptr()).tab3 = NUL as schar_T;
                (*lcs_chars.ptr()).leadtab1 = NUL as schar_T;
                (*lcs_chars.ptr()).leadtab3 = NUL as schar_T;
                if multispace_len > 0 as c_int {
                    (*lcs_chars.ptr()).multispace = xmalloc(
                        (multispace_len as size_t)
                            .wrapping_add(1 as size_t)
                            .wrapping_mul(::core::mem::size_of::<schar_T>()),
                    ) as *mut schar_T;
                    *(*lcs_chars.ptr())
                        .multispace
                        .offset(multispace_len as isize) = NUL as schar_T;
                } else {
                    (*lcs_chars.ptr()).multispace = ::core::ptr::null_mut::<schar_T>();
                }
                if lead_multispace_len > 0 as c_int {
                    (*lcs_chars.ptr()).leadmultispace = xmalloc(
                        (lead_multispace_len as size_t)
                            .wrapping_add(1 as size_t)
                            .wrapping_mul(::core::mem::size_of::<schar_T>()),
                    ) as *mut schar_T;
                    *(*lcs_chars.ptr())
                        .leadmultispace
                        .offset(lead_multispace_len as isize) = NUL as schar_T;
                } else {
                    (*lcs_chars.ptr()).leadmultispace = ::core::ptr::null_mut::<schar_T>();
                }
            }
        }
        let mut p: *const c_char = value;
        while *p != 0 {
            let mut i_0: c_int = 0;
            i_0 = 0 as c_int;
            while i_0 < entries {
                if !(strncmp(
                    p,
                    (*tab.offset(i_0 as isize)).name.data,
                    (*tab.offset(i_0 as isize)).name.size,
                ) == 0 as c_int
                    && *p.offset((*tab.offset(i_0 as isize)).name.size as isize) as c_int
                        == ':' as c_int)
                {
                    i_0 += 1;
                } else {
                    let mut s: *const c_char = p
                        .offset((*tab.offset(i_0 as isize)).name.size as isize)
                        .offset(1 as c_int as isize);
                    if what as c_uint == kListchars as c_int as c_uint
                        && strcmp(
                            (*tab.offset(i_0 as isize)).name.data,
                            b"multispace\0".as_ptr() as *const c_char,
                        ) == 0 as c_int
                    {
                        if round == 0 as c_int {
                            last_multispace = p;
                            multispace_len = 0 as c_int;
                            while *s as c_int != NUL && *s as c_int != ',' as c_int {
                                let mut c1: schar_T = get_encoded_char_adv(&raw mut s);
                                if c1 == 0 as schar_T {
                                    return field_value_err(
                                        errbuf,
                                        errbuflen,
                                        (e_wrong_character_width_for_field_str.ptr() as *const _)
                                            as *const c_char,
                                        (*tab.offset(i_0 as isize)).name.data,
                                    );
                                }
                                multispace_len += 1;
                            }
                            if multispace_len == 0 as c_int {
                                return field_value_err(
                                    errbuf,
                                    errbuflen,
                                    (e_wrong_number_of_characters_for_field_str.ptr() as *const _)
                                        as *const c_char,
                                    (*tab.offset(i_0 as isize)).name.data,
                                );
                            }
                        } else {
                            let mut multispace_pos: c_int = 0 as c_int;
                            while *s as c_int != NUL && *s as c_int != ',' as c_int {
                                let mut c1_0: schar_T = get_encoded_char_adv(&raw mut s);
                                if p == last_multispace {
                                    let c2rust_fresh2 = multispace_pos;
                                    multispace_pos = multispace_pos + 1;
                                    *(*lcs_chars.ptr()).multispace.offset(c2rust_fresh2 as isize) =
                                        c1_0;
                                }
                            }
                        }
                        p = s;
                        break;
                    } else if what as c_uint == kListchars as c_int as c_uint
                        && strcmp(
                            (*tab.offset(i_0 as isize)).name.data,
                            b"leadmultispace\0".as_ptr() as *const c_char,
                        ) == 0 as c_int
                    {
                        if round == 0 as c_int {
                            last_lmultispace = p;
                            lead_multispace_len = 0 as c_int;
                            while *s as c_int != NUL && *s as c_int != ',' as c_int {
                                let mut c1_1: schar_T = get_encoded_char_adv(&raw mut s);
                                if c1_1 == 0 as schar_T {
                                    return field_value_err(
                                        errbuf,
                                        errbuflen,
                                        (e_wrong_character_width_for_field_str.ptr() as *const _)
                                            as *const c_char,
                                        (*tab.offset(i_0 as isize)).name.data,
                                    );
                                }
                                lead_multispace_len += 1;
                            }
                            if lead_multispace_len == 0 as c_int {
                                return field_value_err(
                                    errbuf,
                                    errbuflen,
                                    (e_wrong_number_of_characters_for_field_str.ptr() as *const _)
                                        as *const c_char,
                                    (*tab.offset(i_0 as isize)).name.data,
                                );
                            }
                        } else {
                            let mut multispace_pos_0: c_int = 0 as c_int;
                            while *s as c_int != NUL && *s as c_int != ',' as c_int {
                                let mut c1_2: schar_T = get_encoded_char_adv(&raw mut s);
                                if p == last_lmultispace {
                                    let c2rust_fresh3 = multispace_pos_0;
                                    multispace_pos_0 = multispace_pos_0 + 1;
                                    *(*lcs_chars.ptr())
                                        .leadmultispace
                                        .offset(c2rust_fresh3 as isize) = c1_2;
                                }
                            }
                        }
                        p = s;
                        break;
                    } else {
                        if *s as c_int == NUL {
                            return field_value_err(
                                errbuf,
                                errbuflen,
                                (e_wrong_number_of_characters_for_field_str.ptr() as *const _)
                                    as *const c_char,
                                (*tab.offset(i_0 as isize)).name.data,
                            );
                        }
                        let mut c1_3: schar_T = get_encoded_char_adv(&raw mut s);
                        if c1_3 == 0 as schar_T {
                            return field_value_err(
                                errbuf,
                                errbuflen,
                                (e_wrong_character_width_for_field_str.ptr() as *const _)
                                    as *const c_char,
                                (*tab.offset(i_0 as isize)).name.data,
                            );
                        }
                        let mut c2: schar_T = 0 as schar_T;
                        let mut c3: schar_T = 0 as schar_T;
                        if (*tab.offset(i_0 as isize)).cp == &raw mut (*lcs_chars.ptr()).tab2
                            || (*tab.offset(i_0 as isize)).cp
                                == &raw mut (*lcs_chars.ptr()).leadtab2
                        {
                            if *s as c_int == NUL {
                                return field_value_err(
                                    errbuf,
                                    errbuflen,
                                    (e_wrong_number_of_characters_for_field_str.ptr() as *const _)
                                        as *const c_char,
                                    (*tab.offset(i_0 as isize)).name.data,
                                );
                            }
                            c2 = get_encoded_char_adv(&raw mut s);
                            if c2 == 0 as schar_T {
                                return field_value_err(
                                    errbuf,
                                    errbuflen,
                                    (e_wrong_character_width_for_field_str.ptr() as *const _)
                                        as *const c_char,
                                    (*tab.offset(i_0 as isize)).name.data,
                                );
                            }
                            if !(*s as c_int == ',' as c_int || *s as c_int == NUL) {
                                c3 = get_encoded_char_adv(&raw mut s);
                                if c3 == 0 as schar_T {
                                    return field_value_err(
                                        errbuf,
                                        errbuflen,
                                        (e_wrong_character_width_for_field_str.ptr() as *const _)
                                            as *const c_char,
                                        (*tab.offset(i_0 as isize)).name.data,
                                    );
                                }
                            }
                            if (*tab.offset(i_0 as isize)).cp == &raw mut (*lcs_chars.ptr()).tab2 {
                                has_tab = true_0 != 0;
                            } else {
                                has_leadtab = true_0 != 0;
                            }
                        }
                        if *s as c_int == ',' as c_int || *s as c_int == NUL {
                            if round > 0 as c_int {
                                if (*tab.offset(i_0 as isize)).cp
                                    == &raw mut (*lcs_chars.ptr()).tab2
                                {
                                    (*lcs_chars.ptr()).tab1 = c1_3;
                                    (*lcs_chars.ptr()).tab2 = c2;
                                    (*lcs_chars.ptr()).tab3 = c3;
                                } else if (*tab.offset(i_0 as isize)).cp
                                    == &raw mut (*lcs_chars.ptr()).leadtab2
                                {
                                    (*lcs_chars.ptr()).leadtab1 = c1_3;
                                    (*lcs_chars.ptr()).leadtab2 = c2;
                                    (*lcs_chars.ptr()).leadtab3 = c3;
                                } else if !(*tab.offset(i_0 as isize)).cp.is_null() {
                                    *(*tab.offset(i_0 as isize)).cp = c1_3;
                                }
                            }
                            p = s;
                            break;
                        } else {
                            return field_value_err(
                                errbuf,
                                errbuflen,
                                (e_wrong_number_of_characters_for_field_str.ptr() as *const _)
                                    as *const c_char,
                                (*tab.offset(i_0 as isize)).name.data,
                            );
                        }
                    }
                }
            }
            if i_0 == entries {
                return &raw const e_invarg as *const c_char;
            }
            if *p as c_int == ',' as c_int {
                p = p.offset(1);
            }
        }
        if what as c_uint == kListchars as c_int as c_uint && has_leadtab as c_int != 0 && !has_tab
        {
            return &raw const e_leadtab_requires_tab as *const c_char;
        }
        round += 1;
    }
    if apply {
        if what as c_uint == kListchars as c_int as c_uint {
            xfree((*wp).w_p_lcs_chars.multispace as *mut c_void);
            xfree((*wp).w_p_lcs_chars.leadmultispace as *mut c_void);
            (*wp).w_p_lcs_chars = lcs_chars.get();
        } else {
            (*wp).w_p_fcs_chars = fcs_chars.get();
        }
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn get_fillchars_name(mut _xp: *mut expand_T, mut idx: c_int) -> *mut c_char {
    if idx < 0 as c_int
        || idx
            >= ::core::mem::size_of::<[chars_tab; 21]>()
                .wrapping_div(::core::mem::size_of::<chars_tab>())
                .wrapping_div(
                    (::core::mem::size_of::<[chars_tab; 21]>()
                        .wrapping_rem(::core::mem::size_of::<chars_tab>())
                        == 0) as c_int as usize,
                ) as c_int
    {
        return ::core::ptr::null_mut::<c_char>();
    }
    return (*fcs_tab.ptr())[idx as usize].name.data;
}
pub unsafe extern "C" fn get_listchars_name(mut _xp: *mut expand_T, mut idx: c_int) -> *mut c_char {
    if idx < 0 as c_int
        || idx
            >= ::core::mem::size_of::<[chars_tab; 12]>()
                .wrapping_div(::core::mem::size_of::<chars_tab>())
                .wrapping_div(
                    (::core::mem::size_of::<[chars_tab; 12]>()
                        .wrapping_rem(::core::mem::size_of::<chars_tab>())
                        == 0) as c_int as usize,
                ) as c_int
    {
        return ::core::ptr::null_mut::<c_char>();
    }
    return (*lcs_tab.ptr())[idx as usize].name.data;
}
pub unsafe extern "C" fn check_chars_options() -> *const c_char {
    if !set_chars_option(
        curwin.get(),
        p_lcs.get(),
        kListchars,
        false_0 != 0,
        ::core::ptr::null_mut::<c_char>(),
        0 as size_t,
    )
    .is_null()
    {
        return (e_conflicts_with_value_of_listchars.ptr() as *const _) as *const c_char;
    }
    if !set_chars_option(
        curwin.get(),
        p_fcs.get(),
        kFillchars,
        false_0 != 0,
        ::core::ptr::null_mut::<c_char>(),
        0 as size_t,
    )
    .is_null()
    {
        return (e_conflicts_with_value_of_fillchars.ptr() as *const _) as *const c_char;
    }
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if !set_chars_option(
                wp,
                (*wp).w_onebuf_opt.wo_lcs,
                kListchars,
                true_0 != 0,
                ::core::ptr::null_mut::<c_char>(),
                0 as size_t,
            )
            .is_null()
            {
                return (e_conflicts_with_value_of_listchars.ptr() as *const _) as *const c_char;
            }
            if !set_chars_option(
                wp,
                (*wp).w_onebuf_opt.wo_fcs,
                kFillchars,
                true_0 != 0,
                ::core::ptr::null_mut::<c_char>(),
                0 as size_t,
            )
            .is_null()
            {
                return (e_conflicts_with_value_of_fillchars.ptr() as *const _) as *const c_char;
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return ::core::ptr::null::<c_char>();
}
pub const true_0: c_int = 1 as c_int;
pub const false_0: c_int = 0 as c_int;
pub const INT_MAX: c_int = __INT_MAX__;
pub const __INT_MAX__: c_int = 2147483647 as c_int;
unsafe extern "C" fn c2rust_run_static_initializers() {
    fcs_tab.set([
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).stl,
            name: String_0 {
                data: b"stl\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 4]>().wrapping_sub(1 as size_t),
            },
            def: b" \0".as_ptr() as *const c_char,
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).stlnc,
            name: String_0 {
                data: b"stlnc\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 6]>().wrapping_sub(1 as size_t),
            },
            def: b" \0".as_ptr() as *const c_char,
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).wbr,
            name: String_0 {
                data: b"wbr\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 4]>().wrapping_sub(1 as size_t),
            },
            def: b" \0".as_ptr() as *const c_char,
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).horiz,
            name: String_0 {
                data: b"horiz\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 6]>().wrapping_sub(1 as size_t),
            },
            def: b"\xE2\x94\x80\0".as_ptr() as *const c_char,
            fallback: b"-\0".as_ptr() as *const c_char,
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).horizup,
            name: String_0 {
                data: b"horizup\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 8]>().wrapping_sub(1 as size_t),
            },
            def: b"\xE2\x94\xB4\0".as_ptr() as *const c_char,
            fallback: b"-\0".as_ptr() as *const c_char,
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).horizdown,
            name: String_0 {
                data: b"horizdown\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 10]>().wrapping_sub(1 as size_t),
            },
            def: b"\xE2\x94\xAC\0".as_ptr() as *const c_char,
            fallback: b"-\0".as_ptr() as *const c_char,
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).vert,
            name: String_0 {
                data: b"vert\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 5]>().wrapping_sub(1 as size_t),
            },
            def: b"\xE2\x94\x82\0".as_ptr() as *const c_char,
            fallback: b"|\0".as_ptr() as *const c_char,
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).vertleft,
            name: String_0 {
                data: b"vertleft\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 9]>().wrapping_sub(1 as size_t),
            },
            def: b"\xE2\x94\xA4\0".as_ptr() as *const c_char,
            fallback: b"|\0".as_ptr() as *const c_char,
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).vertright,
            name: String_0 {
                data: b"vertright\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 10]>().wrapping_sub(1 as size_t),
            },
            def: b"\xE2\x94\x9C\0".as_ptr() as *const c_char,
            fallback: b"|\0".as_ptr() as *const c_char,
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).verthoriz,
            name: String_0 {
                data: b"verthoriz\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 10]>().wrapping_sub(1 as size_t),
            },
            def: b"\xE2\x94\xBC\0".as_ptr() as *const c_char,
            fallback: b"+\0".as_ptr() as *const c_char,
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).fold,
            name: String_0 {
                data: b"fold\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 5]>().wrapping_sub(1 as size_t),
            },
            def: b"\xC2\xB7\0".as_ptr() as *const c_char,
            fallback: b"-\0".as_ptr() as *const c_char,
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).foldopen,
            name: String_0 {
                data: b"foldopen\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 9]>().wrapping_sub(1 as size_t),
            },
            def: b"-\0".as_ptr() as *const c_char,
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).foldclosed,
            name: String_0 {
                data: b"foldclose\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 10]>().wrapping_sub(1 as size_t),
            },
            def: b"+\0".as_ptr() as *const c_char,
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).foldsep,
            name: String_0 {
                data: b"foldsep\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 8]>().wrapping_sub(1 as size_t),
            },
            def: b"\xE2\x94\x82\0".as_ptr() as *const c_char,
            fallback: b"|\0".as_ptr() as *const c_char,
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).foldinner,
            name: String_0 {
                data: b"foldinner\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 10]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<c_char>(),
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).diff,
            name: String_0 {
                data: b"diff\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 5]>().wrapping_sub(1 as size_t),
            },
            def: b"-\0".as_ptr() as *const c_char,
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).msgsep,
            name: String_0 {
                data: b"msgsep\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 7]>().wrapping_sub(1 as size_t),
            },
            def: b" \0".as_ptr() as *const c_char,
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).eob,
            name: String_0 {
                data: b"eob\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 4]>().wrapping_sub(1 as size_t),
            },
            def: b"~\0".as_ptr() as *const c_char,
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).lastline,
            name: String_0 {
                data: b"lastline\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 9]>().wrapping_sub(1 as size_t),
            },
            def: b"@\0".as_ptr() as *const c_char,
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).trunc,
            name: String_0 {
                data: b"trunc\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 6]>().wrapping_sub(1 as size_t),
            },
            def: b">\0".as_ptr() as *const c_char,
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*fcs_chars.ptr()).truncrl,
            name: String_0 {
                data: b"truncrl\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 8]>().wrapping_sub(1 as size_t),
            },
            def: b"<\0".as_ptr() as *const c_char,
            fallback: ::core::ptr::null::<c_char>(),
        },
    ]);
    lcs_tab.set([
        chars_tab {
            cp: &raw mut (*lcs_chars.ptr()).eol,
            name: String_0 {
                data: b"eol\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 4]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<c_char>(),
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*lcs_chars.ptr()).ext,
            name: String_0 {
                data: b"extends\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 8]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<c_char>(),
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*lcs_chars.ptr()).nbsp,
            name: String_0 {
                data: b"nbsp\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 5]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<c_char>(),
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*lcs_chars.ptr()).prec,
            name: String_0 {
                data: b"precedes\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 9]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<c_char>(),
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*lcs_chars.ptr()).space,
            name: String_0 {
                data: b"space\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 6]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<c_char>(),
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*lcs_chars.ptr()).tab2,
            name: String_0 {
                data: b"tab\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 4]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<c_char>(),
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*lcs_chars.ptr()).leadtab2,
            name: String_0 {
                data: b"leadtab\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 8]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<c_char>(),
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*lcs_chars.ptr()).lead,
            name: String_0 {
                data: b"lead\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 5]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<c_char>(),
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*lcs_chars.ptr()).trail,
            name: String_0 {
                data: b"trail\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 6]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<c_char>(),
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: &raw mut (*lcs_chars.ptr()).conceal,
            name: String_0 {
                data: b"conceal\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 8]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<c_char>(),
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: ::core::ptr::null_mut::<schar_T>(),
            name: String_0 {
                data: b"multispace\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 11]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<c_char>(),
            fallback: ::core::ptr::null::<c_char>(),
        },
        chars_tab {
            cp: ::core::ptr::null_mut::<schar_T>(),
            name: String_0 {
                data: b"leadmultispace\0".as_ptr() as *const c_char as *mut c_char,
                size: ::core::mem::size_of::<[c_char; 15]>().wrapping_sub(1 as size_t),
            },
            def: ::core::ptr::null::<c_char>(),
            fallback: ::core::ptr::null::<c_char>(),
        },
    ]);
}
#[used]
#[cfg_attr(target_os = "linux", unsafe(link_section = ".init_array"))]
#[cfg_attr(target_os = "windows", unsafe(link_section = ".CRT$XIB"))]
#[cfg_attr(target_os = "macos", unsafe(link_section = "__DATA,__mod_init_func"))]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [c2rust_run_static_initializers];
