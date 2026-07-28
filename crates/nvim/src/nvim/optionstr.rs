use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::autocmd::check_ei;
use crate::src::nvim::charset::{buf_init_chartab, check_isopt};
use crate::src::nvim::diff::{diffanchors_changed, diffopt_changed};
use crate::src::nvim::digraph::keymap_init;
use crate::src::nvim::drawscreen::{redraw_buf_later, redraw_later, status_redraw_buf};
use crate::src::nvim::eval::userfunc::get_scriptlocal_funcname;
use crate::src::nvim::fold::{
    foldUpdateAll, foldmethodIsDiff, foldmethodIsExpr, foldmethodIsIndent, foldmethodIsMarker,
    newFoldLevel,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::indent::tabstop_set;
use crate::src::nvim::indent_c::parse_cino;
use crate::src::nvim::insexpand::set_cpt_callbacks;
use crate::src::nvim::main::{
    bkc_flags, cia_flags, cot_flags, didset_vim, didset_vimruntime, e_invarg, e_modifiable,
    e_unsupportedoption, p_bex, p_bkc, p_bs, p_cia, p_cot, p_enc, p_fenc, p_hlg, p_isk, p_pm, p_tc,
    secure, spo_flags, tc_flags,
};
use crate::src::nvim::mark::free_fmark;
use crate::src::nvim::mbyte::{enc_canonize, utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memline::ml_setflags;
use crate::src::nvim::memory::{strequal, xfree};
use crate::src::nvim::option::{
    copy_option_part, get_fileformat, get_option_varp_scope_from, redraw_titles,
    set_iminsert_global, set_imsearch_global, set_option_direct, skip_to_option_part,
};
use crate::src::nvim::options::{
    kOptBkcFlagAuto, kOptBkcFlagNo, kOptBkcFlagYes, kOptComments, opt_bh_values, opt_bkc_values,
    opt_bt_values, opt_cot_values, opt_spo_values, opt_tc_values,
};
use crate::src::nvim::os::env::vim_unsetenv_ext;
use crate::src::nvim::os::libc::{memset, strcmp, strstr};
use crate::src::nvim::os::time::os_time;
use crate::src::nvim::spell::{
    compile_cap_prog, did_set_spell_option, spell_reload, valid_spellfile, valid_spelllang,
};
use crate::src::nvim::spellfile::spell_check_msm;
use crate::src::nvim::spellsuggest::spell_check_sps;
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, CharsOption, ErrorType, FloatRelative, OptInt, OptVal,
    OptValData, OptValType, String_0, Terminal, WinSplit, WinStyle, buf_T, colnr_T, fcs_chars_T,
    fmark_T, fmarkv_T, lcs_chars_T, linenr_T, optset_T, pos_T, regmatch_T, schar_T, size_t,
    uint8_t, win_T,
};
use crate::src::nvim::window::global_stl_height;
use core::ffi::{CStr, c_char, c_int, c_uchar, c_uint, c_void};

// The carve of a 4,000-line transpiled module; see the child docs.
mod check;
mod frame;
pub use self::check::*;
mod flags;
pub use self::flags::*;
mod expand;
pub use self::expand::*;
mod chars;
pub use self::chars::*;
mod display;
pub use self::display::*;
mod buffer;
pub use self::buffer::*;
mod complete;
pub use self::complete::*;
mod statusline;
pub use self::statusline::*;
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
/// The letters 'formatoptions' accepts.
pub const FO_ALL: &CStr = c"tcro/q2vlb1mMBn,aw]jp";
/// The letters 'cpoptions' accepts.
pub const CPO_VI: &CStr = c"aAbBcCdDeEfFiIJKlLmMnoOpPqrRsStuvWxXyZ$!%+>;~_";
/// The letters 'whichwrap' accepts.
pub const WW_ALL: &CStr = c"bshl<>[]~";
/// The letters 'mouse' accepts.
pub const MOUSE_ALL: &CStr = c"anvichr";
pub const MOUSESCROLL_VERT_DFLT: c_int = 3 as c_int;
pub const MOUSESCROLL_HOR_DFLT: c_int = 6 as c_int;
/// The letters 'concealcursor' accepts.
pub const COCU_ALL: &CStr = c"nvic";
/// The flag letters a 'comments' part may carry before its own separator.
pub const COM_ALL: &CStr = c"nbsmexflrO";
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
/// The letters 'shortmess' accepts. The trailing "nfxi" were removed
/// as flags and are now silently ignored.
pub const SHM_ALL: &CStr = c"rmlwaWtToOsAIcCqFSnfxi";
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
pub const true_0: c_int = 1 as c_int;
pub const false_0: c_int = 0 as c_int;
pub const INT_MAX: c_int = __INT_MAX__;
pub const __INT_MAX__: c_int = 2147483647 as c_int;
