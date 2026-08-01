use crate::src::nvim::api::private::helpers::{api_set_error, arena_dict, cstr_as_string};
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::autocmd::{EVENT_COLORSCHEME, EVENT_COLORSCHEMEPRE, apply_autocmds};
use crate::src::nvim::charset::{skiptowhite, skipwhite, vim_isprintc, vim_strsize};
use crate::src::nvim::cursor_shape::cursor_mode_uses_syn_id;
use crate::src::nvim::decoration_provider::decor_provider_invalidate_hl;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, UPD_SOME_VALID, redraw_all_later};
use crate::src::nvim::eval::last_set_msg;
use crate::src::nvim::eval::vars::{do_unlet, get_var_value};
use crate::src::nvim::ex_docmd::ends_excmd;
use crate::src::nvim::garray::{ga_append_via_ptr, ga_grow, ga_set_growsize};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight::{
    HL_ALTFONT, HL_BLINK, HL_BOLD, HL_CONCEALED, HL_DEFAULT, HL_DIM, HL_INVERSE, HL_ITALIC,
    HL_NOCOMBINE, HL_OVERLINE, HL_STANDOUT, HL_STRIKETHROUGH, HL_UNDERCURL, HL_UNDERDASHED,
    HL_UNDERDOTTED, HL_UNDERDOUBLE, HL_UNDERLINE, HL_UNDERLINE_MASK, HLATTRS_INIT, hl_get_syn_attr,
    hl_get_ui_attr, hlattrs2dict, ns_get_hl, syn_attr2entry,
};
use crate::src::nvim::lua::executor::nlua_set_sctx;
use crate::src::nvim::main::{
    Columns, clear_cmdline, cterm_normal_bg_color, cterm_normal_fg_color, curbuf, current_sctx,
    curwin, e_highlight_group_name_invalid_char, e_highlight_group_name_too_long, e_invarg2,
    got_int, highlight_attr, highlight_attr_last, highlight_stlnc, highlight_user, hlf_names,
    include_default, include_link, include_none, msg_col, msg_grid, msg_silent,
    need_highlight_changed, normal_bg, normal_fg, normal_sp, p_bg, p_verbose, starting, t_colors,
    updating_screen,
};
use crate::src::nvim::map::{map_put_ref_cstr_t_int, mh_get_cstr_t};
use crate::src::nvim::memory::{
    ARENA_EMPTY, arena_memdupz, xfree, xmalloc, xmemrchr, xstrdup, xstrlcat,
};
use crate::src::nvim::message::{
    emsg, message_filtered, msg_advance, msg_clr_eos, msg_ext_set_kind, msg_outtrans, msg_putchar,
    msg_puts_hl, msg_source, semsg,
};
use crate::src::nvim::option::{option_was_set, reset_option_was_set, set_option_value_give_err};
use crate::src::nvim::options::kOptBackground;
use crate::src::nvim::os::libc::{
    __assert_fail, __ctype_b_loc, abort, atoi, gettext, memcmp, memcpy, memmove, memset, snprintf,
    strcasecmp, strchr, strcmp, strlen, strncasecmp, strncmp, strtol,
};
use crate::src::nvim::os::time::os_delay;
use crate::src::nvim::runtime::{exestack, source_runtime_vim_lua};
use crate::src::nvim::strings::{vim_memcpy_up, vim_strup};
use crate::src::nvim::types::api::{kErrorTypeNone, kErrorTypeValidation};
use crate::src::nvim::types::ui::{kUILinegrid, kUIMessages};
use crate::src::nvim::types::{
    Arena, Boolean, Dict, Error, HlAttrs, Integer, KeyDict_get_highlight, KeyDict_highlight,
    KeyValuePair, Map_cstr_t_int, MapHash, NS, Object, OptInt, OptVal, OptValData, OptValType,
    RgbValue, Set_cstr_t, TriState, color_name_table_T, cstr_t, estack_T, expand_T, garray_T,
    int16_t, int32_t, kObjectTypeBoolean, kObjectTypeDict, kObjectTypeNil, kObjectTypeString,
    key_value_pair, object, object_data as C2Rust_Unnamed_0, sctx_T, size_t, uint8_t, uint32_t,
    uint64_t,
};
use crate::src::nvim::ui::{
    ui_call_hl_group_set, ui_default_colors_set, ui_flush, ui_has, ui_mode_info_set, ui_refresh,
    ui_rgb_attached,
};
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISxdigit: C2Rust_Unnamed = 4096;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_15 = 76;
pub const HLF_MSG: C2Rust_Unnamed_15 = 63;
pub const HLF_INACTIVE: C2Rust_Unnamed_15 = 60;
pub const HLF_W: C2Rust_Unnamed_15 = 26;
pub const HLF_SNC: C2Rust_Unnamed_15 = 20;
pub const HLF_S: C2Rust_Unnamed_15 = 19;
pub const HLF_D: C2Rust_Unnamed_15 = 5;
pub const HLF_NONE: C2Rust_Unnamed_15 = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const HLATTRS_DICT_SIZE: C2Rust_Unnamed_16 = 24;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_int;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_17 = 13;
pub const EXPAND_NOTHING: C2Rust_Unnamed_17 = 0;
pub const kOptValTypeString: OptValType = 2;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const MAX_HL_ID: C2Rust_Unnamed_20 = 20000;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HlGroup {
    pub sg_name: *mut ::core::ffi::c_char,
    pub sg_name_u: *mut ::core::ffi::c_char,
    pub sg_cleared: bool,
    pub sg_attr: ::core::ffi::c_int,
    pub sg_link: ::core::ffi::c_int,
    pub sg_deflink: ::core::ffi::c_int,
    pub sg_set: ::core::ffi::c_int,
    pub sg_deflink_sctx: sctx_T,
    pub sg_script_ctx: sctx_T,
    pub sg_cterm: ::core::ffi::c_int,
    pub sg_cterm_fg: ::core::ffi::c_int,
    pub sg_cterm_bg: ::core::ffi::c_int,
    pub sg_cterm_bold: bool,
    pub sg_gui: ::core::ffi::c_int,
    pub sg_rgb_fg: RgbValue,
    pub sg_rgb_bg: RgbValue,
    pub sg_rgb_sp: RgbValue,
    pub sg_rgb_fg_idx: ::core::ffi::c_int,
    pub sg_rgb_bg_idx: ::core::ffi::c_int,
    pub sg_rgb_sp_idx: ::core::ffi::c_int,
    pub sg_blend: ::core::ffi::c_int,
    pub sg_parent: ::core::ffi::c_int,
}
pub const kColorIdxNone: C2Rust_Unnamed_24 = -1;
pub const kColorIdxBg: C2Rust_Unnamed_24 = -4;
pub const kColorIdxFg: C2Rust_Unnamed_24 = -3;
pub const SG_LINK: C2Rust_Unnamed_23 = 8;
pub const kColorIdxHex: C2Rust_Unnamed_24 = -2;
pub const SG_GUI: C2Rust_Unnamed_23 = 4;
pub const SG_CTERM: C2Rust_Unnamed_23 = 2;
pub const DIP_OPT: C2Rust_Unnamed_22 = 16;
pub const DIP_START: C2Rust_Unnamed_22 = 8;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_21 {
    pub dest: *mut ::core::ffi::c_int,
    pub val: RgbValue,
    pub name: Object,
}
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_int;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 45] = unsafe {
    ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
        *b"_Bool hlgroup2dict(Dict *, NS, int, Arena *)\0",
    )
};
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Dict = Dict {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<KeyValuePair>(),
};
pub const ARRAY_DICT_INIT: Dict = KV_INITIAL_VALUE;
pub const KEYSET_OPTIDX_highlight__bg: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__fg: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__sp: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__update: ::core::ffi::c_int = 13 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_highlight__id: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_highlight__link: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_highlight__name: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_highlight__create: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
static value_init_int: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_cstr_t = Set_cstr_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<cstr_t>(),
};
pub const MAP_INIT: Map_cstr_t_int = Map_cstr_t_int {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<::core::ffi::c_int>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_get_cstr_t_int(
    mut map: *mut Map_cstr_t_int,
    mut key: cstr_t,
) -> ::core::ffi::c_int {
    let mut k: uint32_t = mh_get_cstr_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_int.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
#[inline]
unsafe extern "C" fn map_put_cstr_t_int(
    mut map: *mut Map_cstr_t_int,
    mut key: cstr_t,
    mut value: ::core::ffi::c_int,
) {
    let mut val: *mut ::core::ffi::c_int = map_put_ref_cstr_t_int(
        map,
        key,
        ::core::ptr::null_mut::<*mut cstr_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL_0,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const MAX_SYN_NAME: ::core::ffi::c_int = 200 as ::core::ffi::c_int;
static highlight_ga: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);
pub static highlight_arena: GlobalCell<Arena> = GlobalCell::new(ARENA_EMPTY);
pub static highlight_unames: GlobalCell<Map_cstr_t_int> = GlobalCell::new(MAP_INIT);
static hl_name_table: GlobalCell<[*mut ::core::ffi::c_char; 18]> = GlobalCell::new([
    c"bold".as_ptr().cast_mut(),
    c"standout".as_ptr().cast_mut(),
    c"underline".as_ptr().cast_mut(),
    c"undercurl".as_ptr().cast_mut(),
    c"underdouble".as_ptr().cast_mut(),
    c"underdotted".as_ptr().cast_mut(),
    c"underdashed".as_ptr().cast_mut(),
    c"italic".as_ptr().cast_mut(),
    c"reverse".as_ptr().cast_mut(),
    c"inverse".as_ptr().cast_mut(),
    c"strikethrough".as_ptr().cast_mut(),
    c"altfont".as_ptr().cast_mut(),
    c"dim".as_ptr().cast_mut(),
    c"blink".as_ptr().cast_mut(),
    c"conceal".as_ptr().cast_mut(),
    c"overline".as_ptr().cast_mut(),
    c"nocombine".as_ptr().cast_mut(),
    c"NONE".as_ptr().cast_mut(),
]);
static hl_attr_table: GlobalCell<[::core::ffi::c_int; 18]> = GlobalCell::new([
    HL_BOLD,
    HL_STANDOUT,
    HL_UNDERLINE,
    HL_UNDERCURL,
    HL_UNDERDOUBLE,
    HL_UNDERDOTTED,
    HL_UNDERDASHED,
    HL_ITALIC,
    HL_INVERSE,
    HL_INVERSE,
    HL_STRIKETHROUGH,
    HL_ALTFONT,
    HL_DIM,
    HL_BLINK,
    HL_CONCEALED,
    HL_OVERLINE,
    HL_NOCOMBINE,
    0 as ::core::ffi::c_int,
]);
static e_highlight_group_name_not_found_str: GlobalCell<[::core::ffi::c_char; 36]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
            *b"E411: Highlight group not found: %s\0",
        )
    });
static e_group_has_settings_highlight_link_ignored: GlobalCell<[::core::ffi::c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
            *b"E414: Group has settings, highlight link ignored\0",
        )
    });
static e_unexpected_equal_sign_str: GlobalCell<[::core::ffi::c_char; 32]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
            *b"E415: Unexpected equal sign: %s\0",
        )
    });
static e_missing_equal_sign_str_2: GlobalCell<[::core::ffi::c_char; 29]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
            *b"E416: Missing equal sign: %s\0",
        )
    });
static e_missing_argument_str: GlobalCell<[::core::ffi::c_char; 27]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E417: Missing argument: %s\0")
});
static highlight_init_both: GlobalCell<[*const ::core::ffi::c_char; 175]> = GlobalCell::new([
    c"Cursor            guifg=bg      guibg=fg".as_ptr(),
    c"CursorLineNr      gui=bold      cterm=bold".as_ptr(),
    c"PmenuMatch        gui=bold      cterm=bold".as_ptr(),
    c"PmenuMatchSel     gui=bold      cterm=bold".as_ptr(),
    c"PmenuSel          gui=reverse   cterm=reverse,underline blend=0".as_ptr(),
    c"RedrawDebugNormal gui=reverse   cterm=reverse".as_ptr(),
    c"TabLineSel        gui=bold      cterm=NONE".as_ptr(),
    c"TermCursor        gui=reverse   cterm=reverse".as_ptr(),
    c"Underlined        gui=underline cterm=underline".as_ptr(),
    c"lCursor           guifg=bg      guibg=fg".as_ptr(),
    c"default link CursorIM         Cursor".as_ptr(),
    c"default link CursorLineFold   FoldColumn".as_ptr(),
    c"default link CursorLineSign   SignColumn".as_ptr(),
    c"default link DiffTextAdd      DiffText".as_ptr(),
    c"default link EndOfBuffer      NonText".as_ptr(),
    c"default link FloatBorder      NormalFloat".as_ptr(),
    c"default link FloatFooter      FloatTitle".as_ptr(),
    c"default link FloatTitle       Title".as_ptr(),
    c"default link FoldColumn       SignColumn".as_ptr(),
    c"default link IncSearch        CurSearch".as_ptr(),
    c"default link LineNrAbove      LineNr".as_ptr(),
    c"default link LineNrBelow      LineNr".as_ptr(),
    c"default link MsgSeparator     StatusLine".as_ptr(),
    c"default link MsgArea          NONE".as_ptr(),
    c"default link NormalNC         NONE".as_ptr(),
    c"default link PmenuExtra       Pmenu".as_ptr(),
    c"default link PmenuExtraSel    PmenuSel".as_ptr(),
    c"default link PmenuKind        Pmenu".as_ptr(),
    c"default link PmenuKindSel     PmenuSel".as_ptr(),
    c"default link PmenuSbar        Pmenu".as_ptr(),
    c"default link PmenuBorder        Pmenu".as_ptr(),
    c"default link PmenuShadow        FloatShadow".as_ptr(),
    c"default link PmenuShadowThrough FloatShadowThrough".as_ptr(),
    c"default link PreInsert        Added".as_ptr(),
    c"default link ComplMatchIns    NONE".as_ptr(),
    c"default link ComplHint        NonText".as_ptr(),
    c"default link ComplHintMore    MoreMsg".as_ptr(),
    c"default link Substitute       Search".as_ptr(),
    c"default link StatusLineTerm   StatusLine".as_ptr(),
    c"default link StatusLineTermNC StatusLineNC".as_ptr(),
    c"default link StderrMsg        ErrorMsg".as_ptr(),
    c"default link StdoutMsg        NONE".as_ptr(),
    c"default link TabLine          StatusLineNC".as_ptr(),
    c"default link TabLineFill      TabLine".as_ptr(),
    c"default link VertSplit        WinSeparator".as_ptr(),
    c"default link VisualNOS        Visual".as_ptr(),
    c"default link Whitespace       NonText".as_ptr(),
    c"default link WildMenu         PmenuSel".as_ptr(),
    c"default link WinSeparator     Normal".as_ptr(),
    c"default link Character      Constant".as_ptr(),
    c"default link Number         Constant".as_ptr(),
    c"default link Boolean        Constant".as_ptr(),
    c"default link Float          Number".as_ptr(),
    c"default link Conditional    Statement".as_ptr(),
    c"default link Repeat         Statement".as_ptr(),
    c"default link Label          Statement".as_ptr(),
    c"default link Keyword        Statement".as_ptr(),
    c"default link Exception      Statement".as_ptr(),
    c"default link Include        PreProc".as_ptr(),
    c"default link Define         PreProc".as_ptr(),
    c"default link Macro          PreProc".as_ptr(),
    c"default link PreCondit      PreProc".as_ptr(),
    c"default link StorageClass   Type".as_ptr(),
    c"default link Structure      Type".as_ptr(),
    c"default link Typedef        Type".as_ptr(),
    c"default link Tag            Special".as_ptr(),
    c"default link SpecialChar    Special".as_ptr(),
    c"default link SpecialComment Special".as_ptr(),
    c"default link Debug          Special".as_ptr(),
    c"default link SpecialKey     Special".as_ptr(),
    c"default link Ignore         Normal".as_ptr(),
    c"default link LspCodeLens                 NonText".as_ptr(),
    c"default link LspCodeLensSeparator        LspCodeLens".as_ptr(),
    c"default link LspInlayHint                NonText".as_ptr(),
    c"default link LspReferenceRead            LspReferenceText".as_ptr(),
    c"default link LspReferenceText            Visual".as_ptr(),
    c"default link LspReferenceWrite           LspReferenceText".as_ptr(),
    c"default link LspReferenceTarget          LspReferenceText".as_ptr(),
    c"default link LspSignatureActiveParameter Visual".as_ptr(),
    c"default link SnippetTabstop              Visual".as_ptr(),
    c"default link SnippetTabstopActive        SnippetTabstop".as_ptr(),
    c"default link DiagnosticFloatingError    DiagnosticError".as_ptr(),
    c"default link DiagnosticFloatingWarn     DiagnosticWarn".as_ptr(),
    c"default link DiagnosticFloatingInfo     DiagnosticInfo".as_ptr(),
    c"default link DiagnosticFloatingHint     DiagnosticHint".as_ptr(),
    c"default link DiagnosticFloatingOk       DiagnosticOk".as_ptr(),
    c"default link DiagnosticVirtualTextError DiagnosticError".as_ptr(),
    c"default link DiagnosticVirtualTextWarn  DiagnosticWarn".as_ptr(),
    c"default link DiagnosticVirtualTextInfo  DiagnosticInfo".as_ptr(),
    c"default link DiagnosticVirtualTextHint  DiagnosticHint".as_ptr(),
    c"default link DiagnosticVirtualTextOk    DiagnosticOk".as_ptr(),
    c"default link DiagnosticVirtualLinesError DiagnosticError".as_ptr(),
    c"default link DiagnosticVirtualLinesWarn  DiagnosticWarn".as_ptr(),
    c"default link DiagnosticVirtualLinesInfo  DiagnosticInfo".as_ptr(),
    c"default link DiagnosticVirtualLinesHint  DiagnosticHint".as_ptr(),
    c"default link DiagnosticVirtualLinesOk    DiagnosticOk".as_ptr(),
    c"default link DiagnosticSignError        DiagnosticError".as_ptr(),
    c"default link DiagnosticSignWarn         DiagnosticWarn".as_ptr(),
    c"default link DiagnosticSignInfo         DiagnosticInfo".as_ptr(),
    c"default link DiagnosticSignHint         DiagnosticHint".as_ptr(),
    c"default link DiagnosticSignOk           DiagnosticOk".as_ptr(),
    c"default link DiagnosticUnnecessary      Comment".as_ptr(),
    c"default link @variable.builtin           Special".as_ptr(),
    c"default link @variable.parameter.builtin Special".as_ptr(),
    c"default link @constant         Constant".as_ptr(),
    c"default link @constant.builtin Special".as_ptr(),
    c"default link @module         Structure".as_ptr(),
    c"default link @module.builtin Special".as_ptr(),
    c"default link @label          Label".as_ptr(),
    c"default link @string             String".as_ptr(),
    c"default link @string.regexp      @string.special".as_ptr(),
    c"default link @string.escape      @string.special".as_ptr(),
    c"default link @string.special     SpecialChar".as_ptr(),
    c"default link @string.special.url Underlined".as_ptr(),
    c"default link @character         Character".as_ptr(),
    c"default link @character.special SpecialChar".as_ptr(),
    c"default link @boolean      Boolean".as_ptr(),
    c"default link @number       Number".as_ptr(),
    c"default link @number.float Float".as_ptr(),
    c"default link @type         Type".as_ptr(),
    c"default link @type.builtin Special".as_ptr(),
    c"default link @attribute         Macro".as_ptr(),
    c"default link @attribute.builtin Special".as_ptr(),
    c"default link @property          Identifier".as_ptr(),
    c"default link @function         Function".as_ptr(),
    c"default link @function.builtin Special".as_ptr(),
    c"default link @constructor Special".as_ptr(),
    c"default link @operator    Operator".as_ptr(),
    c"default link @keyword Keyword".as_ptr(),
    c"default link @punctuation         Delimiter".as_ptr(),
    c"default link @punctuation.special Special".as_ptr(),
    c"default link @comment Comment".as_ptr(),
    c"default link @comment.error   DiagnosticError".as_ptr(),
    c"default link @comment.warning DiagnosticWarn".as_ptr(),
    c"default link @comment.note    DiagnosticInfo".as_ptr(),
    c"default link @comment.todo    Todo".as_ptr(),
    c"@markup.strong        gui=bold          cterm=bold".as_ptr(),
    c"@markup.italic        gui=italic        cterm=italic".as_ptr(),
    c"@markup.strikethrough gui=strikethrough cterm=strikethrough".as_ptr(),
    c"@markup.underline     gui=underline     cterm=underline".as_ptr(),
    c"default link @markup         Special".as_ptr(),
    c"default link @markup.heading Title".as_ptr(),
    c"default link @markup.link    Underlined".as_ptr(),
    c"default link @diff.plus  Added".as_ptr(),
    c"default link @diff.minus Removed".as_ptr(),
    c"default link @diff.delta Changed".as_ptr(),
    c"default link @tag         Tag".as_ptr(),
    c"default link @tag.builtin Special".as_ptr(),
    c"default @markup.heading.1.delimiter.vimdoc guibg=bg guifg=bg guisp=fg gui=underdouble,nocombine ctermbg=NONE ctermfg=NONE cterm=underdouble,nocombine".as_ptr(),
    c"default @markup.heading.2.delimiter.vimdoc guibg=bg guifg=bg guisp=fg gui=underline,nocombine ctermbg=NONE ctermfg=NONE cterm=underline,nocombine".as_ptr(),
    c"default link @lsp.type.class         @type".as_ptr(),
    c"default link @lsp.type.comment       @comment".as_ptr(),
    c"default link @lsp.type.decorator     @attribute".as_ptr(),
    c"default link @lsp.type.enum          @type".as_ptr(),
    c"default link @lsp.type.enumMember    @constant".as_ptr(),
    c"default link @lsp.type.event         @type".as_ptr(),
    c"default link @lsp.type.function      @function".as_ptr(),
    c"default link @lsp.type.interface     @type".as_ptr(),
    c"default link @lsp.type.keyword       @keyword".as_ptr(),
    c"default link @lsp.type.macro         @constant.macro".as_ptr(),
    c"default link @lsp.type.method        @function.method".as_ptr(),
    c"default link @lsp.type.modifier      @type.qualifier".as_ptr(),
    c"default link @lsp.type.namespace     @module".as_ptr(),
    c"default link @lsp.type.number        @number".as_ptr(),
    c"default link @lsp.type.operator      @operator".as_ptr(),
    c"default link @lsp.type.parameter     @variable.parameter".as_ptr(),
    c"default link @lsp.type.property      @property".as_ptr(),
    c"default link @lsp.type.regexp        @string.regexp".as_ptr(),
    c"default link @lsp.type.string        @string".as_ptr(),
    c"default link @lsp.type.struct        @type".as_ptr(),
    c"default link @lsp.type.type          @type".as_ptr(),
    c"default link @lsp.type.typeParameter @type.definition".as_ptr(),
    c"default link @lsp.type.variable      @variable".as_ptr(),
    c"default link @lsp.mod.deprecated DiagnosticDeprecated".as_ptr(),
    ::core::ptr::null(),
]);
static highlight_init_light: GlobalCell<[*const ::core::ffi::c_char; 71]> = GlobalCell::new([
    c"Normal guifg=NvimDarkGrey2 guibg=NvimLightGrey2 ctermfg=NONE ctermbg=NONE".as_ptr(),
    c"Added                guifg=NvimDarkGreen                                  ctermfg=2".as_ptr(),
    c"Changed              guifg=NvimDarkCyan                                   ctermfg=6".as_ptr(),
    c"ColorColumn                               guibg=NvimLightGrey4            cterm=reverse".as_ptr(),
    c"Conceal              guifg=NvimLightGrey4".as_ptr(),
    c"CurSearch            guifg=NvimLightGrey1 guibg=NvimDarkYellow            ctermfg=15 ctermbg=3".as_ptr(),
    c"CursorColumn                              guibg=NvimLightGrey3".as_ptr(),
    c"CursorLine                                guibg=NvimLightGrey3".as_ptr(),
    c"DiffAdd              guifg=NvimDarkGrey1  guibg=NvimLightGreen            ctermfg=15 ctermbg=2".as_ptr(),
    c"DiffChange           guifg=NvimDarkGrey1  guibg=NvimLightGrey4".as_ptr(),
    c"DiffDelete           guifg=NvimDarkRed                          gui=bold  ctermfg=1 cterm=bold".as_ptr(),
    c"DiffText             guifg=NvimDarkGrey1  guibg=NvimLightCyan             ctermfg=15 ctermbg=6".as_ptr(),
    c"Directory            guifg=NvimDarkCyan                                   ctermfg=6".as_ptr(),
    c"ErrorMsg             guifg=NvimDarkRed                                    ctermfg=1".as_ptr(),
    c"FloatShadow                               guibg=NvimLightGrey4            ctermbg=0 blend=80".as_ptr(),
    c"FloatShadowThrough                        guibg=NvimLightGrey4            ctermbg=0 blend=100".as_ptr(),
    c"Folded               guifg=NvimDarkGrey4  guibg=NvimLightGrey1".as_ptr(),
    c"LineNr               guifg=NvimLightGrey4".as_ptr(),
    c"MatchParen                                guibg=NvimLightGrey4  gui=bold  cterm=bold,underline".as_ptr(),
    c"ModeMsg              guifg=NvimDarkGreen                                  ctermfg=2".as_ptr(),
    c"MoreMsg              guifg=NvimDarkCyan                                   ctermfg=6".as_ptr(),
    c"NonText              guifg=NvimLightGrey4".as_ptr(),
    c"NormalFloat                               guibg=NvimLightGrey1".as_ptr(),
    c"OkMsg                guifg=NvimDarkGreen                                  ctermfg=2".as_ptr(),
    c"Pmenu                                     guibg=NvimLightGrey3            cterm=reverse".as_ptr(),
    c"PmenuThumb                                guibg=NvimLightGrey4".as_ptr(),
    c"Question             guifg=NvimDarkCyan                                   ctermfg=6".as_ptr(),
    c"QuickFixLine         guifg=NvimDarkCyan                                   ctermfg=6".as_ptr(),
    c"RedrawDebugClear                          guibg=NvimLightYellow           ctermfg=15 ctermbg=3".as_ptr(),
    c"RedrawDebugComposed                       guibg=NvimLightGreen            ctermfg=15 ctermbg=2".as_ptr(),
    c"RedrawDebugRecompose                      guibg=NvimLightRed              ctermfg=15 ctermbg=1".as_ptr(),
    c"Removed              guifg=NvimDarkRed                                    ctermfg=1".as_ptr(),
    c"Search               guifg=NvimDarkGrey1  guibg=NvimLightYellow           ctermfg=15 ctermbg=3".as_ptr(),
    c"SignColumn           guifg=NvimLightGrey4".as_ptr(),
    c"SpellBad             guisp=NvimDarkRed    gui=undercurl                   cterm=undercurl".as_ptr(),
    c"SpellCap             guisp=NvimDarkYellow gui=undercurl                   cterm=undercurl".as_ptr(),
    c"SpellLocal           guisp=NvimDarkGreen  gui=undercurl                   cterm=undercurl".as_ptr(),
    c"SpellRare            guisp=NvimDarkCyan   gui=undercurl                   cterm=undercurl".as_ptr(),
    c"StatusLine           guifg=NvimDarkGrey2  guibg=NvimLightGrey4            cterm=reverse".as_ptr(),
    c"StatusLineNC         guifg=NvimDarkGrey3  guibg=NvimLightGrey3            cterm=bold,underline".as_ptr(),
    c"Title                guifg=NvimDarkGrey2                        gui=bold  cterm=bold".as_ptr(),
    c"Visual                                    guibg=NvimLightGrey4            ctermfg=15 ctermbg=0".as_ptr(),
    c"WarningMsg           guifg=NvimDarkYellow                                 ctermfg=3".as_ptr(),
    c"WinBar               guifg=NvimDarkGrey4  guibg=NvimLightGrey1  gui=bold  cterm=bold".as_ptr(),
    c"WinBarNC             guifg=NvimDarkGrey4  guibg=NvimLightGrey1            cterm=bold".as_ptr(),
    c"Constant   guifg=NvimDarkGrey2".as_ptr(),
    c"Operator   guifg=NvimDarkGrey2".as_ptr(),
    c"PreProc    guifg=NvimDarkGrey2".as_ptr(),
    c"Type       guifg=NvimDarkGrey2".as_ptr(),
    c"Delimiter  guifg=NvimDarkGrey2".as_ptr(),
    c"Comment    guifg=NvimDarkGrey4".as_ptr(),
    c"String     guifg=NvimDarkGreen                    ctermfg=2".as_ptr(),
    c"Identifier guifg=NvimDarkBlue                     ctermfg=4".as_ptr(),
    c"Function   guifg=NvimDarkCyan                     ctermfg=6".as_ptr(),
    c"Statement  guifg=NvimDarkGrey2 gui=bold           cterm=bold".as_ptr(),
    c"Special    guifg=NvimDarkCyan                     ctermfg=6".as_ptr(),
    c"Error      guifg=NvimDarkGrey1 guibg=NvimLightRed ctermfg=15 ctermbg=1".as_ptr(),
    c"Todo       guifg=NvimDarkGrey2 gui=bold           cterm=bold".as_ptr(),
    c"DiagnosticError          guifg=NvimDarkRed                      ctermfg=1".as_ptr(),
    c"DiagnosticWarn           guifg=NvimDarkYellow                   ctermfg=3".as_ptr(),
    c"DiagnosticInfo           guifg=NvimDarkCyan                     ctermfg=6".as_ptr(),
    c"DiagnosticHint           guifg=NvimDarkBlue                     ctermfg=4".as_ptr(),
    c"DiagnosticOk             guifg=NvimDarkGreen                    ctermfg=2".as_ptr(),
    c"DiagnosticUnderlineError guisp=NvimDarkRed    gui=underline     cterm=underline".as_ptr(),
    c"DiagnosticUnderlineWarn  guisp=NvimDarkYellow gui=underline     cterm=underline".as_ptr(),
    c"DiagnosticUnderlineInfo  guisp=NvimDarkCyan   gui=underline     cterm=underline".as_ptr(),
    c"DiagnosticUnderlineHint  guisp=NvimDarkBlue   gui=underline     cterm=underline".as_ptr(),
    c"DiagnosticUnderlineOk    guisp=NvimDarkGreen  gui=underline     cterm=underline".as_ptr(),
    c"DiagnosticDeprecated     guisp=NvimDarkRed    gui=strikethrough cterm=strikethrough".as_ptr(),
    c"@variable guifg=NvimDarkGrey2".as_ptr(),
    ::core::ptr::null(),
]);
static highlight_init_dark: GlobalCell<[*const ::core::ffi::c_char; 71]> = GlobalCell::new([
    c"Normal guifg=NvimLightGrey2 guibg=NvimDarkGrey2 ctermfg=NONE ctermbg=NONE".as_ptr(),
    c"Added                guifg=NvimLightGreen                                ctermfg=10".as_ptr(),
    c"Changed              guifg=NvimLightCyan                                 ctermfg=14".as_ptr(),
    c"ColorColumn                                guibg=NvimDarkGrey4           cterm=reverse".as_ptr(),
    c"Conceal              guifg=NvimDarkGrey4".as_ptr(),
    c"CurSearch            guifg=NvimDarkGrey1   guibg=NvimLightYellow         ctermfg=0 ctermbg=11".as_ptr(),
    c"CursorColumn                               guibg=NvimDarkGrey3".as_ptr(),
    c"CursorLine                                 guibg=NvimDarkGrey3".as_ptr(),
    c"DiffAdd              guifg=NvimLightGrey1  guibg=NvimDarkGreen           ctermfg=0 ctermbg=10".as_ptr(),
    c"DiffChange           guifg=NvimLightGrey1  guibg=NvimDarkGrey4".as_ptr(),
    c"DiffDelete           guifg=NvimLightRed                         gui=bold ctermfg=9 cterm=bold".as_ptr(),
    c"DiffText             guifg=NvimLightGrey1  guibg=NvimDarkCyan            ctermfg=0 ctermbg=14".as_ptr(),
    c"Directory            guifg=NvimLightCyan                                 ctermfg=14".as_ptr(),
    c"ErrorMsg             guifg=NvimLightRed                                  ctermfg=9".as_ptr(),
    c"FloatShadow                                guibg=NvimDarkGrey4           ctermbg=0 blend=80".as_ptr(),
    c"FloatShadowThrough                         guibg=NvimDarkGrey4           ctermbg=0 blend=100".as_ptr(),
    c"Folded               guifg=NvimLightGrey4  guibg=NvimDarkGrey1".as_ptr(),
    c"LineNr               guifg=NvimDarkGrey4".as_ptr(),
    c"MatchParen                                 guibg=NvimDarkGrey4  gui=bold cterm=bold,underline".as_ptr(),
    c"ModeMsg              guifg=NvimLightGreen                                ctermfg=10".as_ptr(),
    c"MoreMsg              guifg=NvimLightCyan                                 ctermfg=14".as_ptr(),
    c"NonText              guifg=NvimDarkGrey4".as_ptr(),
    c"NormalFloat                                guibg=NvimDarkGrey1".as_ptr(),
    c"OkMsg                guifg=NvimLightGreen                                ctermfg=10".as_ptr(),
    c"Pmenu                                      guibg=NvimDarkGrey3           cterm=reverse".as_ptr(),
    c"PmenuThumb                                 guibg=NvimDarkGrey4".as_ptr(),
    c"Question             guifg=NvimLightCyan                                 ctermfg=14".as_ptr(),
    c"QuickFixLine         guifg=NvimLightCyan                                 ctermfg=14".as_ptr(),
    c"RedrawDebugClear                           guibg=NvimDarkYellow          ctermfg=0 ctermbg=11".as_ptr(),
    c"RedrawDebugComposed                        guibg=NvimDarkGreen           ctermfg=0 ctermbg=10".as_ptr(),
    c"RedrawDebugRecompose                       guibg=NvimDarkRed             ctermfg=0 ctermbg=9".as_ptr(),
    c"Removed              guifg=NvimLightRed                                  ctermfg=9".as_ptr(),
    c"Search               guifg=NvimLightGrey1  guibg=NvimDarkYellow          ctermfg=0 ctermbg=11".as_ptr(),
    c"SignColumn           guifg=NvimDarkGrey4".as_ptr(),
    c"SpellBad             guisp=NvimLightRed    gui=undercurl                 cterm=undercurl".as_ptr(),
    c"SpellCap             guisp=NvimLightYellow gui=undercurl                 cterm=undercurl".as_ptr(),
    c"SpellLocal           guisp=NvimLightGreen  gui=undercurl                 cterm=undercurl".as_ptr(),
    c"SpellRare            guisp=NvimLightCyan   gui=undercurl                 cterm=undercurl".as_ptr(),
    c"StatusLine           guifg=NvimLightGrey2  guibg=NvimDarkGrey4           cterm=reverse".as_ptr(),
    c"StatusLineNC         guifg=NvimLightGrey3  guibg=NvimDarkGrey3           cterm=bold,underline".as_ptr(),
    c"Title                guifg=NvimLightGrey2                       gui=bold cterm=bold".as_ptr(),
    c"Visual                                     guibg=NvimDarkGrey4           ctermfg=0 ctermbg=15".as_ptr(),
    c"WarningMsg           guifg=NvimLightYellow                               ctermfg=11".as_ptr(),
    c"WinBar               guifg=NvimLightGrey4  guibg=NvimDarkGrey1  gui=bold cterm=bold".as_ptr(),
    c"WinBarNC             guifg=NvimLightGrey4  guibg=NvimDarkGrey1           cterm=bold".as_ptr(),
    c"Constant   guifg=NvimLightGrey2".as_ptr(),
    c"Operator   guifg=NvimLightGrey2".as_ptr(),
    c"PreProc    guifg=NvimLightGrey2".as_ptr(),
    c"Type       guifg=NvimLightGrey2".as_ptr(),
    c"Delimiter  guifg=NvimLightGrey2".as_ptr(),
    c"Comment    guifg=NvimLightGrey4".as_ptr(),
    c"String     guifg=NvimLightGreen                   ctermfg=10".as_ptr(),
    c"Identifier guifg=NvimLightBlue                    ctermfg=12".as_ptr(),
    c"Function   guifg=NvimLightCyan                    ctermfg=14".as_ptr(),
    c"Statement  guifg=NvimLightGrey2 gui=bold          cterm=bold".as_ptr(),
    c"Special    guifg=NvimLightCyan                    ctermfg=14".as_ptr(),
    c"Error      guifg=NvimLightGrey1 guibg=NvimDarkRed ctermfg=0 ctermbg=9".as_ptr(),
    c"Todo       guifg=NvimLightGrey2 gui=bold          cterm=bold".as_ptr(),
    c"DiagnosticError          guifg=NvimLightRed                      ctermfg=9".as_ptr(),
    c"DiagnosticWarn           guifg=NvimLightYellow                   ctermfg=11".as_ptr(),
    c"DiagnosticInfo           guifg=NvimLightCyan                     ctermfg=14".as_ptr(),
    c"DiagnosticHint           guifg=NvimLightBlue                     ctermfg=12".as_ptr(),
    c"DiagnosticOk             guifg=NvimLightGreen                    ctermfg=10".as_ptr(),
    c"DiagnosticUnderlineError guisp=NvimLightRed    gui=underline     cterm=underline".as_ptr(),
    c"DiagnosticUnderlineWarn  guisp=NvimLightYellow gui=underline     cterm=underline".as_ptr(),
    c"DiagnosticUnderlineInfo  guisp=NvimLightCyan   gui=underline     cterm=underline".as_ptr(),
    c"DiagnosticUnderlineHint  guisp=NvimLightBlue   gui=underline     cterm=underline".as_ptr(),
    c"DiagnosticUnderlineOk    guisp=NvimLightGreen  gui=underline     cterm=underline".as_ptr(),
    c"DiagnosticDeprecated     guisp=NvimLightRed    gui=strikethrough cterm=strikethrough".as_ptr(),
    c"@variable guifg=NvimLightGrey2".as_ptr(),
    ::core::ptr::null(),
]);
#[unsafe(no_mangle)]
pub static highlight_init_cmdline: GlobalCell<[*const ::core::ffi::c_char; 141]> =
    GlobalCell::new([
        c"NvimInternalError ctermfg=Red ctermbg=Red guifg=Red guibg=Red".as_ptr(),
        c"default link NvimAssignment Operator".as_ptr(),
        c"default link NvimPlainAssignment NvimAssignment".as_ptr(),
        c"default link NvimAugmentedAssignment NvimAssignment".as_ptr(),
        c"default link NvimAssignmentWithAddition NvimAugmentedAssignment".as_ptr(),
        c"default link NvimAssignmentWithSubtraction NvimAugmentedAssignment".as_ptr(),
        c"default link NvimAssignmentWithConcatenation NvimAugmentedAssignment".as_ptr(),
        c"default link NvimOperator Operator".as_ptr(),
        c"default link NvimUnaryOperator NvimOperator".as_ptr(),
        c"default link NvimUnaryPlus NvimUnaryOperator".as_ptr(),
        c"default link NvimUnaryMinus NvimUnaryOperator".as_ptr(),
        c"default link NvimNot NvimUnaryOperator".as_ptr(),
        c"default link NvimBinaryOperator NvimOperator".as_ptr(),
        c"default link NvimComparison NvimBinaryOperator".as_ptr(),
        c"default link NvimComparisonModifier NvimComparison".as_ptr(),
        c"default link NvimBinaryPlus NvimBinaryOperator".as_ptr(),
        c"default link NvimBinaryMinus NvimBinaryOperator".as_ptr(),
        c"default link NvimConcat NvimBinaryOperator".as_ptr(),
        c"default link NvimConcatOrSubscript NvimConcat".as_ptr(),
        c"default link NvimOr NvimBinaryOperator".as_ptr(),
        c"default link NvimAnd NvimBinaryOperator".as_ptr(),
        c"default link NvimMultiplication NvimBinaryOperator".as_ptr(),
        c"default link NvimDivision NvimBinaryOperator".as_ptr(),
        c"default link NvimMod NvimBinaryOperator".as_ptr(),
        c"default link NvimTernary NvimOperator".as_ptr(),
        c"default link NvimTernaryColon NvimTernary".as_ptr(),
        c"default link NvimParenthesis Delimiter".as_ptr(),
        c"default link NvimLambda NvimParenthesis".as_ptr(),
        c"default link NvimNestingParenthesis NvimParenthesis".as_ptr(),
        c"default link NvimCallingParenthesis NvimParenthesis".as_ptr(),
        c"default link NvimSubscript NvimParenthesis".as_ptr(),
        c"default link NvimSubscriptBracket NvimSubscript".as_ptr(),
        c"default link NvimSubscriptColon NvimSubscript".as_ptr(),
        c"default link NvimCurly NvimSubscript".as_ptr(),
        c"default link NvimContainer NvimParenthesis".as_ptr(),
        c"default link NvimDict NvimContainer".as_ptr(),
        c"default link NvimList NvimContainer".as_ptr(),
        c"default link NvimIdentifier Identifier".as_ptr(),
        c"default link NvimIdentifierScope NvimIdentifier".as_ptr(),
        c"default link NvimIdentifierScopeDelimiter NvimIdentifier".as_ptr(),
        c"default link NvimIdentifierName NvimIdentifier".as_ptr(),
        c"default link NvimIdentifierKey NvimIdentifier".as_ptr(),
        c"default link NvimColon Delimiter".as_ptr(),
        c"default link NvimComma Delimiter".as_ptr(),
        c"default link NvimArrow Delimiter".as_ptr(),
        c"default link NvimRegister SpecialChar".as_ptr(),
        c"default link NvimNumber Number".as_ptr(),
        c"default link NvimFloat NvimNumber".as_ptr(),
        c"default link NvimNumberPrefix Type".as_ptr(),
        c"default link NvimOptionSigil Type".as_ptr(),
        c"default link NvimOptionName NvimIdentifier".as_ptr(),
        c"default link NvimOptionScope NvimIdentifierScope".as_ptr(),
        c"default link NvimOptionScopeDelimiter NvimIdentifierScopeDelimiter".as_ptr(),
        c"default link NvimEnvironmentSigil NvimOptionSigil".as_ptr(),
        c"default link NvimEnvironmentName NvimIdentifier".as_ptr(),
        c"default link NvimString String".as_ptr(),
        c"default link NvimStringBody NvimString".as_ptr(),
        c"default link NvimStringQuote NvimString".as_ptr(),
        c"default link NvimStringSpecial SpecialChar".as_ptr(),
        c"default link NvimSingleQuote NvimStringQuote".as_ptr(),
        c"default link NvimSingleQuotedBody NvimStringBody".as_ptr(),
        c"default link NvimSingleQuotedQuote NvimStringSpecial".as_ptr(),
        c"default link NvimDoubleQuote NvimStringQuote".as_ptr(),
        c"default link NvimDoubleQuotedBody NvimStringBody".as_ptr(),
        c"default link NvimDoubleQuotedEscape NvimStringSpecial".as_ptr(),
        c"default link NvimFigureBrace NvimInternalError".as_ptr(),
        c"default link NvimSingleQuotedUnknownEscape NvimInternalError".as_ptr(),
        c"default link NvimSpacing Normal".as_ptr(),
        c"default link NvimInvalidSingleQuotedUnknownEscape NvimInternalError".as_ptr(),
        c"default link NvimInvalid Error".as_ptr(),
        c"default link NvimInvalidAssignment NvimInvalid".as_ptr(),
        c"default link NvimInvalidPlainAssignment NvimInvalidAssignment".as_ptr(),
        c"default link NvimInvalidAugmentedAssignment NvimInvalidAssignment".as_ptr(),
        c"default link NvimInvalidAssignmentWithAddition NvimInvalidAugmentedAssignment".as_ptr(),
        c"default link NvimInvalidAssignmentWithSubtraction NvimInvalidAugmentedAssignment"
            .as_ptr(),
        c"default link NvimInvalidAssignmentWithConcatenation NvimInvalidAugmentedAssignment"
            .as_ptr(),
        c"default link NvimInvalidOperator NvimInvalid".as_ptr(),
        c"default link NvimInvalidUnaryOperator NvimInvalidOperator".as_ptr(),
        c"default link NvimInvalidUnaryPlus NvimInvalidUnaryOperator".as_ptr(),
        c"default link NvimInvalidUnaryMinus NvimInvalidUnaryOperator".as_ptr(),
        c"default link NvimInvalidNot NvimInvalidUnaryOperator".as_ptr(),
        c"default link NvimInvalidBinaryOperator NvimInvalidOperator".as_ptr(),
        c"default link NvimInvalidComparison NvimInvalidBinaryOperator".as_ptr(),
        c"default link NvimInvalidComparisonModifier NvimInvalidComparison".as_ptr(),
        c"default link NvimInvalidBinaryPlus NvimInvalidBinaryOperator".as_ptr(),
        c"default link NvimInvalidBinaryMinus NvimInvalidBinaryOperator".as_ptr(),
        c"default link NvimInvalidConcat NvimInvalidBinaryOperator".as_ptr(),
        c"default link NvimInvalidConcatOrSubscript NvimInvalidConcat".as_ptr(),
        c"default link NvimInvalidOr NvimInvalidBinaryOperator".as_ptr(),
        c"default link NvimInvalidAnd NvimInvalidBinaryOperator".as_ptr(),
        c"default link NvimInvalidMultiplication NvimInvalidBinaryOperator".as_ptr(),
        c"default link NvimInvalidDivision NvimInvalidBinaryOperator".as_ptr(),
        c"default link NvimInvalidMod NvimInvalidBinaryOperator".as_ptr(),
        c"default link NvimInvalidTernary NvimInvalidOperator".as_ptr(),
        c"default link NvimInvalidTernaryColon NvimInvalidTernary".as_ptr(),
        c"default link NvimInvalidDelimiter NvimInvalid".as_ptr(),
        c"default link NvimInvalidParenthesis NvimInvalidDelimiter".as_ptr(),
        c"default link NvimInvalidLambda NvimInvalidParenthesis".as_ptr(),
        c"default link NvimInvalidNestingParenthesis NvimInvalidParenthesis".as_ptr(),
        c"default link NvimInvalidCallingParenthesis NvimInvalidParenthesis".as_ptr(),
        c"default link NvimInvalidSubscript NvimInvalidParenthesis".as_ptr(),
        c"default link NvimInvalidSubscriptBracket NvimInvalidSubscript".as_ptr(),
        c"default link NvimInvalidSubscriptColon NvimInvalidSubscript".as_ptr(),
        c"default link NvimInvalidCurly NvimInvalidSubscript".as_ptr(),
        c"default link NvimInvalidContainer NvimInvalidParenthesis".as_ptr(),
        c"default link NvimInvalidDict NvimInvalidContainer".as_ptr(),
        c"default link NvimInvalidList NvimInvalidContainer".as_ptr(),
        c"default link NvimInvalidValue NvimInvalid".as_ptr(),
        c"default link NvimInvalidIdentifier NvimInvalidValue".as_ptr(),
        c"default link NvimInvalidIdentifierScope NvimInvalidIdentifier".as_ptr(),
        c"default link NvimInvalidIdentifierScopeDelimiter NvimInvalidIdentifier".as_ptr(),
        c"default link NvimInvalidIdentifierName NvimInvalidIdentifier".as_ptr(),
        c"default link NvimInvalidIdentifierKey NvimInvalidIdentifier".as_ptr(),
        c"default link NvimInvalidColon NvimInvalidDelimiter".as_ptr(),
        c"default link NvimInvalidComma NvimInvalidDelimiter".as_ptr(),
        c"default link NvimInvalidArrow NvimInvalidDelimiter".as_ptr(),
        c"default link NvimInvalidRegister NvimInvalidValue".as_ptr(),
        c"default link NvimInvalidNumber NvimInvalidValue".as_ptr(),
        c"default link NvimInvalidFloat NvimInvalidNumber".as_ptr(),
        c"default link NvimInvalidNumberPrefix NvimInvalidNumber".as_ptr(),
        c"default link NvimInvalidOptionSigil NvimInvalidIdentifier".as_ptr(),
        c"default link NvimInvalidOptionName NvimInvalidIdentifier".as_ptr(),
        c"default link NvimInvalidOptionScope NvimInvalidIdentifierScope".as_ptr(),
        c"default link NvimInvalidOptionScopeDelimiter NvimInvalidIdentifierScopeDelimiter"
            .as_ptr(),
        c"default link NvimInvalidEnvironmentSigil NvimInvalidOptionSigil".as_ptr(),
        c"default link NvimInvalidEnvironmentName NvimInvalidIdentifier".as_ptr(),
        c"default link NvimInvalidString NvimInvalidValue".as_ptr(),
        c"default link NvimInvalidStringBody NvimStringBody".as_ptr(),
        c"default link NvimInvalidStringQuote NvimInvalidString".as_ptr(),
        c"default link NvimInvalidStringSpecial NvimStringSpecial".as_ptr(),
        c"default link NvimInvalidSingleQuote NvimInvalidStringQuote".as_ptr(),
        c"default link NvimInvalidSingleQuotedBody NvimInvalidStringBody".as_ptr(),
        c"default link NvimInvalidSingleQuotedQuote NvimInvalidStringSpecial".as_ptr(),
        c"default link NvimInvalidDoubleQuote NvimInvalidStringQuote".as_ptr(),
        c"default link NvimInvalidDoubleQuotedBody NvimInvalidStringBody".as_ptr(),
        c"default link NvimInvalidDoubleQuotedEscape NvimInvalidStringSpecial".as_ptr(),
        c"default link NvimInvalidDoubleQuotedUnknownEscape NvimInvalidValue".as_ptr(),
        c"default link NvimInvalidFigureBrace NvimInvalidDelimiter".as_ptr(),
        c"default link NvimInvalidSpacing ErrorMsg".as_ptr(),
        c"default link NvimDoubleQuotedUnknownEscape NvimInvalidValue".as_ptr(),
        ::core::ptr::null(),
    ]);
pub unsafe extern "C" fn highlight_num_groups() -> ::core::ffi::c_int {
    return (*highlight_ga.ptr()).ga_len;
}
pub unsafe extern "C" fn highlight_group_name(
    mut id: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    return (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(id as isize)).sg_name;
}
pub unsafe extern "C" fn highlight_link_id(mut id: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(id as isize)).sg_link;
}
pub unsafe extern "C" fn syn_init_cmdline_highlight(mut reset: bool, mut init: bool) {
    let mut i: size_t = 0 as size_t;
    while !(*highlight_init_cmdline.ptr())[i as usize].is_null() {
        do_highlight((*highlight_init_cmdline.ptr())[i as usize], reset, init);
        i = i.wrapping_add(1);
    }
}
pub unsafe extern "C" fn init_highlight(mut both: bool, mut reset: bool) {
    static had_both: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut p: *mut ::core::ffi::c_char =
        get_var_value(b"g:colors_name\0".as_ptr() as *const ::core::ffi::c_char);
    if !p.is_null() {
        let mut copy_p: *mut ::core::ffi::c_char = xstrdup(p);
        let mut okay: bool = load_colors(copy_p) != 0;
        xfree(copy_p as *mut ::core::ffi::c_void);
        if okay {
            return;
        }
    }
    if both {
        had_both.set(true_0 != 0);
        let pp: *const *const ::core::ffi::c_char =
            highlight_init_both.ptr() as *mut *const ::core::ffi::c_char;
        let mut i: size_t = 0 as size_t;
        while !(*pp.offset(i as isize)).is_null() {
            do_highlight(*pp.offset(i as isize), reset, true_0 != 0);
            i = i.wrapping_add(1);
        }
    } else if !had_both.get() {
        return;
    }
    let pp_0: *const *const ::core::ffi::c_char =
        if *p_bg.get() as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
            highlight_init_light.ptr() as *mut *const ::core::ffi::c_char
        } else {
            highlight_init_dark.ptr() as *mut *const ::core::ffi::c_char
        };
    let mut i_0: size_t = 0 as size_t;
    while !(*pp_0.offset(i_0 as isize)).is_null() {
        do_highlight(*pp_0.offset(i_0 as isize), reset, true_0 != 0);
        i_0 = i_0.wrapping_add(1);
    }
    syn_init_cmdline_highlight(false_0 != 0, false_0 != 0);
}
pub unsafe extern "C" fn load_colors(mut name: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if recursive.get() {
        return OK;
    }
    recursive.set(true_0 != 0);
    let mut buflen: size_t = strlen(name).wrapping_add(12 as size_t);
    let mut buf: *mut ::core::ffi::c_char = xmalloc(buflen) as *mut ::core::ffi::c_char;
    apply_autocmds(
        EVENT_COLORSCHEMEPRE,
        name,
        (*curbuf.get()).b_fname,
        false_0 != 0,
        curbuf.get(),
    );
    snprintf(
        buf,
        buflen,
        b"colors/%s.*\0".as_ptr() as *const ::core::ffi::c_char,
        name,
    );
    let mut retval: ::core::ffi::c_int = source_runtime_vim_lua(
        buf,
        DIP_START as ::core::ffi::c_int + DIP_OPT as ::core::ffi::c_int,
    );
    xfree(buf as *mut ::core::ffi::c_void);
    if retval == OK {
        apply_autocmds(
            EVENT_COLORSCHEME,
            name,
            (*curbuf.get()).b_fname,
            false_0 != 0,
            curbuf.get(),
        );
    }
    recursive.set(false_0 != 0);
    return retval;
}
static color_names: GlobalCell<[*mut ::core::ffi::c_char; 28]> = GlobalCell::new([
    c"Black".as_ptr().cast_mut(),
    c"DarkBlue".as_ptr().cast_mut(),
    c"DarkGreen".as_ptr().cast_mut(),
    c"DarkCyan".as_ptr().cast_mut(),
    c"DarkRed".as_ptr().cast_mut(),
    c"DarkMagenta".as_ptr().cast_mut(),
    c"Brown".as_ptr().cast_mut(),
    c"DarkYellow".as_ptr().cast_mut(),
    c"Gray".as_ptr().cast_mut(),
    c"Grey".as_ptr().cast_mut(),
    c"LightGray".as_ptr().cast_mut(),
    c"LightGrey".as_ptr().cast_mut(),
    c"DarkGray".as_ptr().cast_mut(),
    c"DarkGrey".as_ptr().cast_mut(),
    c"Blue".as_ptr().cast_mut(),
    c"LightBlue".as_ptr().cast_mut(),
    c"Green".as_ptr().cast_mut(),
    c"LightGreen".as_ptr().cast_mut(),
    c"Cyan".as_ptr().cast_mut(),
    c"LightCyan".as_ptr().cast_mut(),
    c"Red".as_ptr().cast_mut(),
    c"LightRed".as_ptr().cast_mut(),
    c"Magenta".as_ptr().cast_mut(),
    c"LightMagenta".as_ptr().cast_mut(),
    c"Yellow".as_ptr().cast_mut(),
    c"LightYellow".as_ptr().cast_mut(),
    c"White".as_ptr().cast_mut(),
    c"NONE".as_ptr().cast_mut(),
]);
static color_numbers_16: GlobalCell<[::core::ffi::c_int; 28]> = GlobalCell::new([
    0 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    9 as ::core::ffi::c_int,
    9 as ::core::ffi::c_int,
    10 as ::core::ffi::c_int,
    10 as ::core::ffi::c_int,
    11 as ::core::ffi::c_int,
    11 as ::core::ffi::c_int,
    12 as ::core::ffi::c_int,
    12 as ::core::ffi::c_int,
    13 as ::core::ffi::c_int,
    13 as ::core::ffi::c_int,
    14 as ::core::ffi::c_int,
    14 as ::core::ffi::c_int,
    15 as ::core::ffi::c_int,
    -1 as ::core::ffi::c_int,
]);
static color_numbers_88: GlobalCell<[::core::ffi::c_int; 28]> = GlobalCell::new([
    0 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    32 as ::core::ffi::c_int,
    72 as ::core::ffi::c_int,
    84 as ::core::ffi::c_int,
    84 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    82 as ::core::ffi::c_int,
    82 as ::core::ffi::c_int,
    12 as ::core::ffi::c_int,
    43 as ::core::ffi::c_int,
    10 as ::core::ffi::c_int,
    61 as ::core::ffi::c_int,
    14 as ::core::ffi::c_int,
    63 as ::core::ffi::c_int,
    9 as ::core::ffi::c_int,
    74 as ::core::ffi::c_int,
    13 as ::core::ffi::c_int,
    75 as ::core::ffi::c_int,
    11 as ::core::ffi::c_int,
    78 as ::core::ffi::c_int,
    15 as ::core::ffi::c_int,
    -1 as ::core::ffi::c_int,
]);
static color_numbers_256: GlobalCell<[::core::ffi::c_int; 28]> = GlobalCell::new([
    0 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    130 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    248 as ::core::ffi::c_int,
    248 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    242 as ::core::ffi::c_int,
    242 as ::core::ffi::c_int,
    12 as ::core::ffi::c_int,
    81 as ::core::ffi::c_int,
    10 as ::core::ffi::c_int,
    121 as ::core::ffi::c_int,
    14 as ::core::ffi::c_int,
    159 as ::core::ffi::c_int,
    9 as ::core::ffi::c_int,
    224 as ::core::ffi::c_int,
    13 as ::core::ffi::c_int,
    225 as ::core::ffi::c_int,
    11 as ::core::ffi::c_int,
    229 as ::core::ffi::c_int,
    15 as ::core::ffi::c_int,
    -1 as ::core::ffi::c_int,
]);
static color_numbers_8: GlobalCell<[::core::ffi::c_int; 28]> = GlobalCell::new([
    0 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    0 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    0 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
    -1 as ::core::ffi::c_int,
]);
unsafe extern "C" fn lookup_color(
    idx: ::core::ffi::c_int,
    foreground: bool,
    boldp: *mut TriState,
) -> ::core::ffi::c_int {
    let mut color: ::core::ffi::c_int = (*color_numbers_16.ptr())[idx as usize];
    if color < 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    if t_colors.get() == 8 as ::core::ffi::c_int {
        color = (*color_numbers_8.ptr())[idx as usize];
        if foreground {
            if color & 8 as ::core::ffi::c_int != 0 {
                *boldp = kTrue;
            } else {
                *boldp = kFalse;
            }
        }
        color &= 7 as ::core::ffi::c_int;
    } else if t_colors.get() == 16 as ::core::ffi::c_int {
        color = (*color_numbers_8.ptr())[idx as usize];
    } else if t_colors.get() == 88 as ::core::ffi::c_int {
        color = (*color_numbers_88.ptr())[idx as usize];
    } else if t_colors.get() >= 256 as ::core::ffi::c_int {
        color = (*color_numbers_256.ptr())[idx as usize];
    }
    return color;
}
pub unsafe extern "C" fn set_hl_group(
    mut id: ::core::ffi::c_int,
    mut attrs: HlAttrs,
    mut dict: *mut KeyDict_highlight,
    mut link_id: ::core::ffi::c_int,
) {
    let mut idx: ::core::ffi::c_int = id - 1 as ::core::ffi::c_int;
    let mut is_default: bool = attrs.rgb_ae_attr & HL_DEFAULT as int32_t != 0;
    if is_default as ::core::ffi::c_int != 0
        && hl_has_settings(idx, true_0 != 0) as ::core::ffi::c_int != 0
        && !(*dict).force
    {
        return;
    }
    let mut g: *mut HlGroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize);
    (*g).sg_cleared = false_0 != 0;
    let mut old_link: ::core::ffi::c_int = (*g).sg_link;
    if link_id > 0 as ::core::ffi::c_int {
        (*g).sg_link = link_id;
        (*g).sg_script_ctx = current_sctx.get();
        (*g).sg_script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum;
        nlua_set_sctx(&raw mut (*g).sg_script_ctx);
        (*g).sg_set |= SG_LINK as ::core::ffi::c_int;
        if is_default {
            (*g).sg_deflink = link_id;
            (*g).sg_deflink_sctx = current_sctx.get();
            (*g).sg_deflink_sctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum;
            nlua_set_sctx(&raw mut (*g).sg_deflink_sctx);
        }
    } else {
        (*g).sg_link = 0 as ::core::ffi::c_int;
    }
    let mut update: bool = (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__update
        != 0 as ::core::ffi::c_ulonglong
        && (*dict).update as ::core::ffi::c_int != 0;
    (*g).sg_gui = (attrs.rgb_ae_attr & !(HL_DEFAULT as int32_t)) as ::core::ffi::c_int;
    (*g).sg_rgb_fg = attrs.rgb_fg_color;
    (*g).sg_rgb_bg = attrs.rgb_bg_color;
    (*g).sg_rgb_sp = attrs.rgb_sp_color;
    let mut cattrs: [C2Rust_Unnamed_21; 4] = [
        C2Rust_Unnamed_21 {
            dest: &raw mut (*g).sg_rgb_fg_idx,
            val: (*g).sg_rgb_fg,
            name: if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__fg
                != 0 as ::core::ffi::c_ulonglong
            {
                (*dict).fg
            } else {
                (*dict).foreground
            },
        },
        C2Rust_Unnamed_21 {
            dest: &raw mut (*g).sg_rgb_bg_idx,
            val: (*g).sg_rgb_bg,
            name: if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__bg
                != 0 as ::core::ffi::c_ulonglong
            {
                (*dict).bg
            } else {
                (*dict).background
            },
        },
        C2Rust_Unnamed_21 {
            dest: &raw mut (*g).sg_rgb_sp_idx,
            val: (*g).sg_rgb_sp,
            name: if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__sp
                != 0 as ::core::ffi::c_ulonglong
            {
                (*dict).sp
            } else {
                (*dict).special
            },
        },
        C2Rust_Unnamed_21 {
            dest: ::core::ptr::null_mut::<::core::ffi::c_int>(),
            val: -1 as RgbValue,
            name: object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_0 { boolean: false },
            },
        },
    ];
    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !cattrs[j as usize].dest.is_null() {
        if cattrs[j as usize].name.type_0 as ::core::ffi::c_uint
            != kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if cattrs[j as usize].val < 0 as RgbValue {
                *cattrs[j as usize].dest = kColorIdxNone as ::core::ffi::c_int;
            } else if cattrs[j as usize].name.type_0 as ::core::ffi::c_uint
                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                && cattrs[j as usize].name.data.string.size != 0
            {
                name_to_color(
                    cattrs[j as usize].name.data.string.data,
                    cattrs[j as usize].dest,
                );
            } else {
                *cattrs[j as usize].dest = kColorIdxHex as ::core::ffi::c_int;
            }
        } else if !update {
            *cattrs[j as usize].dest = kColorIdxNone as ::core::ffi::c_int;
        } else if old_link > 0 as ::core::ffi::c_int && cattrs[j as usize].val >= 0 as RgbValue {
            let mut linked: *mut HlGroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                .offset((old_link - 1 as ::core::ffi::c_int) as isize);
            let mut linked_idx: ::core::ffi::c_int = if j == 0 as ::core::ffi::c_int {
                (*linked).sg_rgb_fg_idx
            } else if j == 1 as ::core::ffi::c_int {
                (*linked).sg_rgb_bg_idx
            } else {
                (*linked).sg_rgb_sp_idx
            };
            *cattrs[j as usize].dest = if linked_idx != kColorIdxNone as ::core::ffi::c_int {
                linked_idx
            } else {
                kColorIdxHex as ::core::ffi::c_int
            };
        }
        j += 1;
    }
    (*g).sg_cterm = (attrs.cterm_ae_attr & !(HL_DEFAULT as int32_t)) as ::core::ffi::c_int;
    (*g).sg_cterm_bg = attrs.cterm_bg_color as ::core::ffi::c_int;
    (*g).sg_cterm_fg = attrs.cterm_fg_color as ::core::ffi::c_int;
    (*g).sg_cterm_bold = (*g).sg_cterm & HL_BOLD != 0;
    if attrs.hl_blend != -1 as int32_t {
        (*g).sg_blend = attrs.hl_blend as ::core::ffi::c_int;
    } else if !update {
        (*g).sg_blend = -1 as ::core::ffi::c_int;
    }
    (*g).sg_script_ctx = current_sctx.get();
    (*g).sg_script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_lnum;
    nlua_set_sctx(&raw mut (*g).sg_script_ctx);
    (*g).sg_attr = hl_get_syn_attr(0 as ::core::ffi::c_int, id, attrs);
    if strcmp(
        (*g).sg_name_u,
        b"NORMAL\0".as_ptr() as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        cterm_normal_fg_color.set((*g).sg_cterm_fg);
        cterm_normal_bg_color.set((*g).sg_cterm_bg);
        let mut did_changed: bool = false_0 != 0;
        if normal_bg.get() != (*g).sg_rgb_bg
            || normal_fg.get() != (*g).sg_rgb_fg
            || normal_sp.get() != (*g).sg_rgb_sp
        {
            did_changed = true_0 != 0;
        }
        normal_fg.set((*g).sg_rgb_fg);
        normal_bg.set((*g).sg_rgb_bg);
        normal_sp.set((*g).sg_rgb_sp);
        if did_changed {
            highlight_attr_set_all();
        }
        ui_default_colors_set();
    } else if cursor_mode_uses_syn_id(id) {
        ui_mode_info_set();
    }
    if !updating_screen.get() {
        redraw_all_later(UPD_NOT_VALID);
    }
    need_highlight_changed.set(true_0 != 0);
}
unsafe extern "C" fn set_gui_color(
    mut idx: ::core::ffi::c_int,
    mut init: bool,
    mut arg: *const ::core::ffi::c_char,
    mut color: *mut RgbValue,
    mut color_idx: *mut ::core::ffi::c_int,
) -> bool {
    if init as ::core::ffi::c_int != 0
        && (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_set
            & SG_GUI as ::core::ffi::c_int
            != 0
    {
        return false_0 != 0;
    }
    if !init {
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_set |=
            SG_GUI as ::core::ffi::c_int;
    }
    let mut old_color: RgbValue = *color;
    let mut old_idx: ::core::ffi::c_int = *color_idx;
    if strcmp(arg, b"NONE\0".as_ptr() as *const ::core::ffi::c_char) != 0 as ::core::ffi::c_int {
        *color = name_to_color(arg, color_idx);
    } else {
        *color = -1 as ::core::ffi::c_int as RgbValue;
        *color_idx = kColorIdxNone as ::core::ffi::c_int;
    }
    return *color != old_color || *color_idx != old_idx;
}
pub unsafe extern "C" fn do_highlight(
    mut line: *const ::core::ffi::c_char,
    forceit: bool,
    init: bool,
) {
    if !init && ends_excmd(*line as uint8_t as ::core::ffi::c_int) != 0 {
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while i <= (*highlight_ga.ptr()).ga_len && !got_int.get() {
            highlight_list_one(i);
            i += 1;
        }
        return;
    }
    let mut dodefault: bool = false_0 != 0;
    let mut name_end: *const ::core::ffi::c_char = skiptowhite(line);
    let mut linep: *const ::core::ffi::c_char = skipwhite(name_end);
    if strncmp(
        line,
        b"default\0".as_ptr() as *const ::core::ffi::c_char,
        name_end.offset_from(line) as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        dodefault = true_0 != 0;
        line = linep;
        name_end = skiptowhite(line);
        linep = skipwhite(name_end);
    }
    let mut doclear: bool = false_0 != 0;
    let mut dolink: bool = false_0 != 0;
    if strncmp(
        line,
        b"clear\0".as_ptr() as *const ::core::ffi::c_char,
        name_end.offset_from(line) as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        doclear = true_0 != 0;
    } else if strncmp(
        line,
        b"link\0".as_ptr() as *const ::core::ffi::c_char,
        name_end.offset_from(line) as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        dolink = true_0 != 0;
    }
    if !doclear && !dolink && ends_excmd(*linep as uint8_t as ::core::ffi::c_int) != 0 {
        let mut id: ::core::ffi::c_int =
            syn_name2id_len(line, name_end.offset_from(line) as size_t);
        if id == 0 as ::core::ffi::c_int {
            semsg(
                gettext(
                    (e_highlight_group_name_not_found_str.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                line,
            );
        } else {
            msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
            highlight_list_one(id);
        }
        return;
    }
    if dolink {
        let mut from_start: *const ::core::ffi::c_char = linep;
        let mut to_id: ::core::ffi::c_int = 0;
        let mut hlgroup: *mut HlGroup = ::core::ptr::null_mut::<HlGroup>();
        let mut from_end: *const ::core::ffi::c_char = skiptowhite(from_start);
        let mut to_start: *const ::core::ffi::c_char = skipwhite(from_end);
        let mut to_end: *const ::core::ffi::c_char = skiptowhite(to_start);
        if ends_excmd(*from_start as uint8_t as ::core::ffi::c_int) != 0
            || ends_excmd(*to_start as uint8_t as ::core::ffi::c_int) != 0
        {
            semsg(
                gettext(
                    b"E412: Not enough arguments: \":highlight link %s\"\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                from_start,
            );
            return;
        }
        if ends_excmd(*skipwhite(to_end) as ::core::ffi::c_int) == 0 {
            semsg(
                gettext(
                    b"E413: Too many arguments: \":highlight link %s\"\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                from_start,
            );
            return;
        }
        let mut from_id: ::core::ffi::c_int =
            syn_check_group(from_start, from_end.offset_from(from_start) as size_t);
        if strncmp(
            to_start,
            b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            to_id = 0 as ::core::ffi::c_int;
        } else {
            to_id = syn_check_group(to_start, to_end.offset_from(to_start) as size_t);
        }
        if from_id > 0 as ::core::ffi::c_int {
            hlgroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                .offset((from_id - 1 as ::core::ffi::c_int) as isize);
            if dodefault as ::core::ffi::c_int != 0
                && (forceit as ::core::ffi::c_int != 0
                    || (*hlgroup).sg_deflink == 0 as ::core::ffi::c_int)
            {
                (*hlgroup).sg_deflink = to_id;
                (*hlgroup).sg_deflink_sctx = current_sctx.get();
                (*hlgroup).sg_deflink_sctx.sc_lnum += (*((*exestack.ptr()).ga_data
                    as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum;
                nlua_set_sctx(&raw mut (*hlgroup).sg_deflink_sctx);
            }
        }
        if from_id > 0 as ::core::ffi::c_int
            && (!init || (*hlgroup).sg_set == 0 as ::core::ffi::c_int)
        {
            if to_id > 0 as ::core::ffi::c_int
                && !forceit
                && !init
                && hl_has_settings(from_id - 1 as ::core::ffi::c_int, dodefault)
                    as ::core::ffi::c_int
                    != 0
            {
                if (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name
                .is_null()
                    && !dodefault
                {
                    emsg(gettext(
                        (e_group_has_settings_highlight_link_ignored.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ));
                }
            } else if (*hlgroup).sg_link != to_id
                || (*hlgroup).sg_script_ctx.sc_sid != (*current_sctx.ptr()).sc_sid
                || (*hlgroup).sg_cleared as ::core::ffi::c_int != 0
            {
                if !init {
                    (*hlgroup).sg_set |= SG_LINK as ::core::ffi::c_int;
                }
                (*hlgroup).sg_link = to_id;
                (*hlgroup).sg_script_ctx = current_sctx.get();
                (*hlgroup).sg_script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum;
                nlua_set_sctx(&raw mut (*hlgroup).sg_script_ctx);
                (*hlgroup).sg_cleared = false_0 != 0;
                redraw_all_later(UPD_SOME_VALID);
                need_highlight_changed.set(true_0 != 0);
            }
        }
        return;
    }
    if doclear {
        line = linep;
        if ends_excmd(*line as uint8_t as ::core::ffi::c_int) != 0 {
            do_unlet(
                b"g:colors_name\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 14]>().wrapping_sub(1 as size_t),
                true_0 != 0,
            );
            restore_cterm_colors();
            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while j < (*highlight_ga.ptr()).ga_len {
                highlight_clear(j);
                j += 1;
            }
            init_highlight(true_0 != 0, true_0 != 0);
            highlight_changed();
            redraw_all_later(UPD_NOT_VALID);
            return;
        }
        name_end = skiptowhite(line);
        linep = skipwhite(name_end);
    }
    let mut id_0: ::core::ffi::c_int = syn_check_group(line, name_end.offset_from(line) as size_t);
    if id_0 == 0 as ::core::ffi::c_int {
        return;
    }
    let mut idx: ::core::ffi::c_int = id_0 - 1 as ::core::ffi::c_int;
    if dodefault as ::core::ffi::c_int != 0
        && hl_has_settings(idx, true_0 != 0) as ::core::ffi::c_int != 0
    {
        return;
    }
    let mut item_before: HlGroup =
        *((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize);
    let mut is_normal_group: bool = strcmp(
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_name_u,
        b"NORMAL\0".as_ptr() as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int;
    if doclear as ::core::ffi::c_int != 0
        || forceit as ::core::ffi::c_int != 0 && init as ::core::ffi::c_int != 0
    {
        highlight_clear(idx);
        if !doclear {
            (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_set =
                0 as ::core::ffi::c_int;
        }
    }
    let mut did_change: bool = false_0 != 0;
    let mut error: bool = false_0 != 0;
    let mut key: [::core::ffi::c_char; 64] = [0; 64];
    let mut arg: [::core::ffi::c_char; 512] = [0; 512];
    if !doclear {
        let mut arg_start: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        while ends_excmd(*linep as uint8_t as ::core::ffi::c_int) == 0 {
            let mut key_start: *const ::core::ffi::c_char = linep;
            if *linep as ::core::ffi::c_int == '=' as ::core::ffi::c_int {
                semsg(
                    gettext(
                        (e_unexpected_equal_sign_str.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ),
                    key_start,
                );
                error = true_0 != 0;
                break;
            } else {
                while *linep as ::core::ffi::c_int != 0
                    && !ascii_iswhite(*linep as ::core::ffi::c_int)
                    && *linep as ::core::ffi::c_int != '=' as ::core::ffi::c_int
                {
                    linep = linep.offset(1);
                }
                let mut key_len: size_t = linep.offset_from(key_start) as size_t;
                if key_len
                    > ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(1 as usize)
                {
                    emsg(gettext(
                        b"E423: Illegal argument\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                    error = true_0 != 0;
                    break;
                } else {
                    vim_memcpy_up(&raw mut key as *mut ::core::ffi::c_char, key_start, key_len);
                    key[key_len as usize] = NUL as ::core::ffi::c_char;
                    linep = skipwhite(linep);
                    if strcmp(
                        &raw mut key as *mut ::core::ffi::c_char,
                        b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        if !init
                            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                .offset(idx as isize))
                            .sg_set
                                == 0 as ::core::ffi::c_int
                        {
                            if !init {
                                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                    .offset(idx as isize))
                                .sg_set |=
                                    SG_CTERM as ::core::ffi::c_int + SG_GUI as ::core::ffi::c_int;
                            }
                            highlight_clear(idx);
                        }
                    } else if *linep as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
                        semsg(
                            gettext(
                                (e_missing_equal_sign_str_2.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ),
                            key_start,
                        );
                        error = true_0 != 0;
                        break;
                    } else {
                        linep = linep.offset(1);
                        linep = skipwhite(linep);
                        if *linep as ::core::ffi::c_int == '\'' as ::core::ffi::c_int {
                            linep = linep.offset(1);
                            arg_start = linep;
                            linep = strchr(linep, '\'' as ::core::ffi::c_int);
                            if linep.is_null() {
                                semsg(
                                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                                    key_start,
                                );
                                error = true_0 != 0;
                                break;
                            }
                        } else {
                            arg_start = linep;
                            linep = skiptowhite(linep);
                        }
                        if linep == arg_start {
                            semsg(
                                gettext(
                                    (e_missing_argument_str.ptr() as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                key_start,
                            );
                            error = true_0 != 0;
                            break;
                        } else {
                            let mut arg_len: size_t = linep.offset_from(arg_start) as size_t;
                            if arg_len
                                > ::core::mem::size_of::<[::core::ffi::c_char; 512]>()
                                    .wrapping_sub(1 as usize)
                            {
                                emsg(gettext(b"E423: Illegal argument\0".as_ptr()
                                    as *const ::core::ffi::c_char));
                                error = true_0 != 0;
                                break;
                            } else {
                                memcpy(
                                    &raw mut arg as *mut ::core::ffi::c_char
                                        as *mut ::core::ffi::c_void,
                                    arg_start as *const ::core::ffi::c_void,
                                    arg_len,
                                );
                                arg[arg_len as usize] = NUL as ::core::ffi::c_char;
                                if *linep as ::core::ffi::c_int == '\'' as ::core::ffi::c_int {
                                    linep = linep.offset(1);
                                }
                                if strcmp(
                                    &raw mut key as *mut ::core::ffi::c_char,
                                    b"TERM\0".as_ptr() as *const ::core::ffi::c_char,
                                ) == 0 as ::core::ffi::c_int
                                    || strcmp(
                                        &raw mut key as *mut ::core::ffi::c_char,
                                        b"CTERM\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) == 0 as ::core::ffi::c_int
                                    || strcmp(
                                        &raw mut key as *mut ::core::ffi::c_char,
                                        b"GUI\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) == 0 as ::core::ffi::c_int
                                {
                                    let mut attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    let mut i_0: ::core::ffi::c_int = 0;
                                    while arg[off as usize] as ::core::ffi::c_int != NUL {
                                        i_0 = ::core::mem::size_of::<[::core::ffi::c_int; 18]>()
                                            .wrapping_div(
                                                ::core::mem::size_of::<::core::ffi::c_int>(),
                                            )
                                            .wrapping_div(
                                                (::core::mem::size_of::<[::core::ffi::c_int; 18]>()
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        ::core::ffi::c_int,
                                                    >(
                                                    ))
                                                    == 0)
                                                    as ::core::ffi::c_int
                                                    as usize,
                                            )
                                            as ::core::ffi::c_int;
                                        loop {
                                            i_0 -= 1;
                                            if i_0 < 0 as ::core::ffi::c_int {
                                                break;
                                            }
                                            let mut len: ::core::ffi::c_int = strlen(
                                                (*hl_name_table.ptr())[i_0 as usize]
                                                    as *const ::core::ffi::c_char,
                                            )
                                                as ::core::ffi::c_int;
                                            if strncasecmp(
                                                (&raw mut arg as *mut ::core::ffi::c_char)
                                                    .offset(off as isize),
                                                (*hl_name_table.ptr())[i_0 as usize],
                                                len as size_t,
                                            ) != 0 as ::core::ffi::c_int
                                            {
                                                continue;
                                            }
                                            if (*hl_attr_table.ptr())[i_0 as usize]
                                                & HL_UNDERLINE_MASK
                                                != 0
                                            {
                                                attr &= !(HL_UNDERLINE_MASK);
                                            }
                                            attr |= (*hl_attr_table.ptr())[i_0 as usize];
                                            off += len;
                                            break;
                                        }
                                        if i_0 < 0 as ::core::ffi::c_int {
                                            semsg(
                                                gettext(b"E418: Illegal value: %s\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                &raw mut arg as *mut ::core::ffi::c_char,
                                            );
                                            error = true_0 != 0;
                                            break;
                                        } else if arg[off as usize] as ::core::ffi::c_int
                                            == ',' as ::core::ffi::c_int
                                        {
                                            off += 1;
                                        }
                                    }
                                    if error {
                                        break;
                                    }
                                    if *(&raw mut key as *mut ::core::ffi::c_char)
                                        as ::core::ffi::c_int
                                        == 'C' as ::core::ffi::c_int
                                    {
                                        if !init
                                            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_set
                                                & SG_CTERM as ::core::ffi::c_int
                                                == 0
                                        {
                                            if !init {
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_set |= SG_CTERM as ::core::ffi::c_int;
                                            }
                                            (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_cterm = attr;
                                            (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_cterm_bold = false_0 != 0;
                                        }
                                    } else if *(&raw mut key as *mut ::core::ffi::c_char)
                                        as ::core::ffi::c_int
                                        == 'G' as ::core::ffi::c_int
                                    {
                                        if !init
                                            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_set
                                                & SG_GUI as ::core::ffi::c_int
                                                == 0
                                        {
                                            if !init {
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_set |= SG_GUI as ::core::ffi::c_int;
                                            }
                                            (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_gui = attr;
                                        }
                                    }
                                } else if strcmp(
                                    &raw mut key as *mut ::core::ffi::c_char,
                                    b"FONT\0".as_ptr() as *const ::core::ffi::c_char,
                                ) != 0 as ::core::ffi::c_int
                                {
                                    if strcmp(
                                        &raw mut key as *mut ::core::ffi::c_char,
                                        b"CTERMFG\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) == 0 as ::core::ffi::c_int
                                        || strcmp(
                                            &raw mut key as *mut ::core::ffi::c_char,
                                            b"CTERMBG\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                    {
                                        if !init
                                            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_set
                                                & SG_CTERM as ::core::ffi::c_int
                                                == 0
                                        {
                                            if !init {
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_set |= SG_CTERM as ::core::ffi::c_int;
                                            }
                                            if key[5 as ::core::ffi::c_int as usize]
                                                as ::core::ffi::c_int
                                                == 'F' as ::core::ffi::c_int
                                                && (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_cterm_bold
                                                    as ::core::ffi::c_int
                                                    != 0
                                            {
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_cterm &= !(HL_BOLD);
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_cterm_bold = false_0 != 0;
                                            }
                                            let mut color: ::core::ffi::c_int = 0;
                                            if ascii_isdigit(
                                                *(&raw mut arg as *mut ::core::ffi::c_char)
                                                    as ::core::ffi::c_int,
                                            ) {
                                                color =
                                                    atoi(&raw mut arg as *mut ::core::ffi::c_char);
                                            } else if strcasecmp(
                                                &raw mut arg as *mut ::core::ffi::c_char,
                                                b"fg\0".as_ptr() as *const ::core::ffi::c_char
                                                    as *mut ::core::ffi::c_char,
                                            ) == 0 as ::core::ffi::c_int
                                            {
                                                if cterm_normal_fg_color.get() != 0 {
                                                    color = cterm_normal_fg_color.get()
                                                        - 1 as ::core::ffi::c_int;
                                                } else {
                                                    emsg(gettext(
                                                        b"E419: FG color unknown\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ));
                                                    error = true_0 != 0;
                                                    break;
                                                }
                                            } else if strcasecmp(
                                                &raw mut arg as *mut ::core::ffi::c_char,
                                                b"bg\0".as_ptr() as *const ::core::ffi::c_char
                                                    as *mut ::core::ffi::c_char,
                                            ) == 0 as ::core::ffi::c_int
                                            {
                                                if cterm_normal_bg_color.get()
                                                    > 0 as ::core::ffi::c_int
                                                {
                                                    color = cterm_normal_bg_color.get()
                                                        - 1 as ::core::ffi::c_int;
                                                } else {
                                                    emsg(gettext(
                                                        b"E420: BG color unknown\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ));
                                                    error = true_0 != 0;
                                                    break;
                                                }
                                            } else {
                                                let mut off_0: ::core::ffi::c_int =
                                                    if (*(&raw mut arg as *mut ::core::ffi::c_char)
                                                        as ::core::ffi::c_int)
                                                        < 'a' as ::core::ffi::c_int
                                                        || *(&raw mut arg
                                                            as *mut ::core::ffi::c_char)
                                                            as ::core::ffi::c_int
                                                            > 'z' as ::core::ffi::c_int
                                                    {
                                                        *(&raw mut arg as *mut ::core::ffi::c_char)
                                                            as ::core::ffi::c_int
                                                    } else {
                                                        *(&raw mut arg as *mut ::core::ffi::c_char)
                                                            as ::core::ffi::c_int
                                                            - ('a' as ::core::ffi::c_int
                                                                - 'A' as ::core::ffi::c_int)
                                                    };
                                                let mut i_1: ::core::ffi::c_int = 0;
                                                i_1 = ::core::mem::size_of::<
                                                    [*mut ::core::ffi::c_char; 28],
                                                >(
                                                )
                                                .wrapping_div(::core::mem::size_of::<
                                                    *mut ::core::ffi::c_char,
                                                >(
                                                ))
                                                .wrapping_div(
                                                    (::core::mem::size_of::<
                                                        [*mut ::core::ffi::c_char; 28],
                                                    >(
                                                    )
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        *mut ::core::ffi::c_char,
                                                    >(
                                                    )) == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                )
                                                    as ::core::ffi::c_int;
                                                loop {
                                                    i_1 -= 1;
                                                    if i_1 < 0 as ::core::ffi::c_int {
                                                        break;
                                                    }
                                                    if off_0
                                                        == *(*color_names.ptr())[i_1 as usize]
                                                            .offset(
                                                                0 as ::core::ffi::c_int as isize,
                                                            )
                                                            as ::core::ffi::c_int
                                                        && strcasecmp(
                                                            (&raw mut arg
                                                                as *mut ::core::ffi::c_char)
                                                                .offset(
                                                                    1 as ::core::ffi::c_int
                                                                        as isize,
                                                                ),
                                                            (*color_names.ptr())[i_1 as usize]
                                                                .offset(
                                                                    1 as ::core::ffi::c_int
                                                                        as isize,
                                                                ),
                                                        ) == 0 as ::core::ffi::c_int
                                                    {
                                                        break;
                                                    }
                                                }
                                                if i_1 < 0 as ::core::ffi::c_int {
                                                    semsg(
                                                        gettext(
                                                            b"E421: Color name or number not recognized: %s\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                        ),
                                                        key_start,
                                                    );
                                                    error = true_0 != 0;
                                                    break;
                                                } else {
                                                    let mut bold: TriState = kNone;
                                                    color = lookup_color(
                                                        i_1,
                                                        key[5 as ::core::ffi::c_int as usize]
                                                            as ::core::ffi::c_int
                                                            == 'F' as ::core::ffi::c_int,
                                                        &raw mut bold,
                                                    );
                                                    if bold as ::core::ffi::c_int
                                                        == kTrue as ::core::ffi::c_int
                                                    {
                                                        (*((*highlight_ga.ptr()).ga_data
                                                            as *mut HlGroup)
                                                            .offset(idx as isize))
                                                        .sg_cterm |= HL_BOLD;
                                                        (*((*highlight_ga.ptr()).ga_data
                                                            as *mut HlGroup)
                                                            .offset(idx as isize))
                                                        .sg_cterm_bold = true_0 != 0;
                                                    } else if bold as ::core::ffi::c_int
                                                        == kFalse as ::core::ffi::c_int
                                                    {
                                                        (*((*highlight_ga.ptr()).ga_data
                                                            as *mut HlGroup)
                                                            .offset(idx as isize))
                                                        .sg_cterm &= !(HL_BOLD);
                                                    }
                                                }
                                            }
                                            if key[5 as ::core::ffi::c_int as usize]
                                                as ::core::ffi::c_int
                                                == 'F' as ::core::ffi::c_int
                                            {
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_cterm_fg = color + 1 as ::core::ffi::c_int;
                                                if is_normal_group {
                                                    cterm_normal_fg_color
                                                        .set(color + 1 as ::core::ffi::c_int);
                                                }
                                            } else {
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_cterm_bg = color + 1 as ::core::ffi::c_int;
                                                if is_normal_group {
                                                    cterm_normal_bg_color
                                                        .set(color + 1 as ::core::ffi::c_int);
                                                    if !ui_rgb_attached() {
                                                        if color >= 0 as ::core::ffi::c_int {
                                                            let mut dark: ::core::ffi::c_int =
                                                                -1 as ::core::ffi::c_int;
                                                            if t_colors.get()
                                                                < 16 as ::core::ffi::c_int
                                                            {
                                                                dark = (color
                                                                    == 0 as ::core::ffi::c_int
                                                                    || color
                                                                        == 4 as ::core::ffi::c_int)
                                                                    as ::core::ffi::c_int;
                                                            } else if color
                                                                < 16 as ::core::ffi::c_int
                                                            {
                                                                dark = (color
                                                                    < 7 as ::core::ffi::c_int
                                                                    || color
                                                                        == 8 as ::core::ffi::c_int)
                                                                    as ::core::ffi::c_int;
                                                            }
                                                            if dark != -1 as ::core::ffi::c_int
                                                                && dark
                                                                    != (*p_bg.get()
                                                                        as ::core::ffi::c_int
                                                                        == 'd'
                                                                            as ::core::ffi::c_int)
                                                                        as ::core::ffi::c_int
                                                                && !option_was_set(kOptBackground)
                                                            {
                                                                set_option_value_give_err(
                                                                    kOptBackground,
                                                                    OptVal {
                                                                        type_0: kOptValTypeString,
                                                                        data: OptValData {
                                                                            string: cstr_as_string(
                                                                                if dark != 0 {
                                                                                    b"dark\0".as_ptr() as *const ::core::ffi::c_char
                                                                                } else {
                                                                                    b"light\0".as_ptr() as *const ::core::ffi::c_char
                                                                                },
                                                                            ),
                                                                        },
                                                                    },
                                                                    0 as ::core::ffi::c_int,
                                                                );
                                                                reset_option_was_set(
                                                                    kOptBackground,
                                                                );
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else if strcmp(
                                        &raw mut key as *mut ::core::ffi::c_char,
                                        b"GUIFG\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) == 0 as ::core::ffi::c_int
                                    {
                                        did_change = set_gui_color(
                                            idx,
                                            init,
                                            &raw mut arg as *mut ::core::ffi::c_char,
                                            &raw mut (*((*highlight_ga.ptr()).ga_data
                                                as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_rgb_fg,
                                            &raw mut (*((*highlight_ga.ptr()).ga_data
                                                as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_rgb_fg_idx,
                                        );
                                        if is_normal_group {
                                            normal_fg.set(
                                                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_rgb_fg,
                                            );
                                        }
                                    } else if strcmp(
                                        &raw mut key as *mut ::core::ffi::c_char,
                                        b"GUIBG\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) == 0 as ::core::ffi::c_int
                                    {
                                        did_change = set_gui_color(
                                            idx,
                                            init,
                                            &raw mut arg as *mut ::core::ffi::c_char,
                                            &raw mut (*((*highlight_ga.ptr()).ga_data
                                                as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_rgb_bg,
                                            &raw mut (*((*highlight_ga.ptr()).ga_data
                                                as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_rgb_bg_idx,
                                        );
                                        if is_normal_group {
                                            normal_bg.set(
                                                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_rgb_bg,
                                            );
                                        }
                                    } else if strcmp(
                                        &raw mut key as *mut ::core::ffi::c_char,
                                        b"GUISP\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) == 0 as ::core::ffi::c_int
                                    {
                                        did_change = set_gui_color(
                                            idx,
                                            init,
                                            &raw mut arg as *mut ::core::ffi::c_char,
                                            &raw mut (*((*highlight_ga.ptr()).ga_data
                                                as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_rgb_sp,
                                            &raw mut (*((*highlight_ga.ptr()).ga_data
                                                as *mut HlGroup)
                                                .offset(idx as isize))
                                            .sg_rgb_sp_idx,
                                        );
                                        if is_normal_group {
                                            normal_sp.set(
                                                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_rgb_sp,
                                            );
                                        }
                                    } else if !(strcmp(
                                        &raw mut key as *mut ::core::ffi::c_char,
                                        b"START\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) == 0 as ::core::ffi::c_int
                                        || strcmp(
                                            &raw mut key as *mut ::core::ffi::c_char,
                                            b"STOP\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int)
                                    {
                                        if strcmp(
                                            &raw mut key as *mut ::core::ffi::c_char,
                                            b"BLEND\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                        {
                                            if strcmp(
                                                &raw mut arg as *mut ::core::ffi::c_char,
                                                b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
                                            ) != 0 as ::core::ffi::c_int
                                            {
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_blend = strtol(
                                                    &raw mut arg as *mut ::core::ffi::c_char,
                                                    ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(
                                                    ),
                                                    10 as ::core::ffi::c_int,
                                                )
                                                    as ::core::ffi::c_int;
                                            } else {
                                                (*((*highlight_ga.ptr()).ga_data
                                                    as *mut HlGroup)
                                                    .offset(idx as isize))
                                                .sg_blend = -1 as ::core::ffi::c_int;
                                            }
                                        } else {
                                            semsg(
                                                gettext(b"E423: Illegal argument: %s\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                key_start,
                                            );
                                            error = true_0 != 0;
                                            break;
                                        }
                                    }
                                }
                                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                    .offset(idx as isize))
                                .sg_cleared = false_0 != 0;
                                if !init
                                    || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                        .offset(idx as isize))
                                    .sg_set
                                        & SG_LINK as ::core::ffi::c_int
                                        == 0
                                {
                                    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                                        .offset(idx as isize))
                                    .sg_link = 0 as ::core::ffi::c_int;
                                }
                                linep = skipwhite(linep);
                            }
                        }
                    }
                }
            }
        }
    }
    let mut did_highlight_changed: bool = false_0 != 0;
    if !error && is_normal_group as ::core::ffi::c_int != 0 {
        highlight_attr_set_all();
        if !ui_has(kUILinegrid) && starting.get() == 0 as ::core::ffi::c_int {
            ui_refresh();
        } else {
            ui_default_colors_set();
        }
        did_highlight_changed = true_0 != 0;
        redraw_all_later(UPD_NOT_VALID);
    } else {
        set_hl_attr(idx);
    }
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_script_ctx =
        current_sctx.get();
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
        .sg_script_ctx
        .sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_lnum;
    nlua_set_sctx(
        &raw mut (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
            .sg_script_ctx,
    );
    if (did_change as ::core::ffi::c_int != 0
        || memcmp(
            ((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)
                as *const ::core::ffi::c_void,
            &raw mut item_before as *const ::core::ffi::c_void,
            ::core::mem::size_of::<HlGroup>(),
        ) != 0 as ::core::ffi::c_int)
        && !did_highlight_changed
    {
        if !updating_screen.get() {
            redraw_all_later(UPD_NOT_VALID);
        }
        need_highlight_changed.set(true_0 != 0);
    }
}
pub unsafe extern "C" fn restore_cterm_colors() {
    normal_fg.set(-1 as ::core::ffi::c_int as RgbValue);
    normal_bg.set(-1 as ::core::ffi::c_int as RgbValue);
    normal_sp.set(-1 as ::core::ffi::c_int as RgbValue);
    cterm_normal_fg_color.set(0 as ::core::ffi::c_int);
    cterm_normal_bg_color.set(0 as ::core::ffi::c_int);
}
unsafe extern "C" fn hl_has_settings(mut idx: ::core::ffi::c_int, mut check_link: bool) -> bool {
    return (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cleared
        as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && ((*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_attr
            != 0 as ::core::ffi::c_int
            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
                .sg_cterm_fg
                != 0 as ::core::ffi::c_int
            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
                .sg_cterm_bg
                != 0 as ::core::ffi::c_int
            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
                .sg_rgb_fg_idx
                != kColorIdxNone as ::core::ffi::c_int
            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
                .sg_rgb_bg_idx
                != kColorIdxNone as ::core::ffi::c_int
            || (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize))
                .sg_rgb_sp_idx
                != kColorIdxNone as ::core::ffi::c_int
            || check_link as ::core::ffi::c_int != 0
                && (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_set
                    & SG_LINK as ::core::ffi::c_int
                    != 0);
}
unsafe extern "C" fn highlight_clear(mut idx: ::core::ffi::c_int) {
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cleared =
        true_0 != 0;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_attr =
        0 as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cterm =
        0 as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cterm_bold =
        false_0 != 0;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cterm_fg =
        0 as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cterm_bg =
        0 as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_gui =
        0 as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_fg =
        -1 as ::core::ffi::c_int as RgbValue;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_bg =
        -1 as ::core::ffi::c_int as RgbValue;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_sp =
        -1 as ::core::ffi::c_int as RgbValue;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_fg_idx =
        kColorIdxNone as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_bg_idx =
        kColorIdxNone as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_rgb_sp_idx =
        kColorIdxNone as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_blend =
        -1 as ::core::ffi::c_int;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_link =
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_deflink;
    (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_script_ctx =
        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_deflink_sctx;
}
pub const LIST_ATTR: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LIST_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const LIST_INT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
unsafe extern "C" fn highlight_list_one(id: ::core::ffi::c_int) {
    let mut sgp: *const HlGroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
        .offset((id - 1 as ::core::ffi::c_int) as isize);
    let mut didh: bool = false_0 != 0;
    if message_filtered((*sgp).sg_name) {
        return;
    }
    if (*sgp).sg_parent != 0 && (*sgp).sg_cleared as ::core::ffi::c_int != 0 {
        return;
    }
    didh = highlight_list_arg(
        id,
        didh,
        LIST_ATTR,
        (*sgp).sg_cterm,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"cterm\0".as_ptr() as *const ::core::ffi::c_char,
    );
    didh = highlight_list_arg(
        id,
        didh,
        LIST_INT,
        (*sgp).sg_cterm_fg,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"ctermfg\0".as_ptr() as *const ::core::ffi::c_char,
    );
    didh = highlight_list_arg(
        id,
        didh,
        LIST_INT,
        (*sgp).sg_cterm_bg,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"ctermbg\0".as_ptr() as *const ::core::ffi::c_char,
    );
    didh = highlight_list_arg(
        id,
        didh,
        LIST_ATTR,
        (*sgp).sg_gui,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"gui\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut hexbuf: [::core::ffi::c_char; 8] = [0; 8];
    didh = highlight_list_arg(
        id,
        didh,
        LIST_STRING,
        0 as ::core::ffi::c_int,
        coloridx_to_name(
            (*sgp).sg_rgb_fg_idx,
            (*sgp).sg_rgb_fg as ::core::ffi::c_int,
            &raw mut hexbuf as *mut ::core::ffi::c_char,
        ),
        b"guifg\0".as_ptr() as *const ::core::ffi::c_char,
    );
    didh = highlight_list_arg(
        id,
        didh,
        LIST_STRING,
        0 as ::core::ffi::c_int,
        coloridx_to_name(
            (*sgp).sg_rgb_bg_idx,
            (*sgp).sg_rgb_bg as ::core::ffi::c_int,
            &raw mut hexbuf as *mut ::core::ffi::c_char,
        ),
        b"guibg\0".as_ptr() as *const ::core::ffi::c_char,
    );
    didh = highlight_list_arg(
        id,
        didh,
        LIST_STRING,
        0 as ::core::ffi::c_int,
        coloridx_to_name(
            (*sgp).sg_rgb_sp_idx,
            (*sgp).sg_rgb_sp as ::core::ffi::c_int,
            &raw mut hexbuf as *mut ::core::ffi::c_char,
        ),
        b"guisp\0".as_ptr() as *const ::core::ffi::c_char,
    );
    didh = highlight_list_arg(
        id,
        didh,
        LIST_INT,
        (*sgp).sg_blend + 1 as ::core::ffi::c_int,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"blend\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if (*sgp).sg_link != 0 && !got_int.get() {
        syn_list_header(didh, 0 as ::core::ffi::c_int, id, true_0 != 0);
        didh = true_0 != 0;
        msg_puts_hl(
            b"links to\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_D as ::core::ffi::c_int,
            false_0 != 0,
        );
        msg_putchar(' ' as ::core::ffi::c_int);
        msg_outtrans(
            (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(
                ((*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_link
                    - 1 as ::core::ffi::c_int) as isize,
            ))
            .sg_name,
            0 as ::core::ffi::c_int,
            false_0 != 0,
        );
    }
    if !didh {
        highlight_list_arg(
            id,
            didh,
            LIST_STRING,
            0 as ::core::ffi::c_int,
            b"cleared\0".as_ptr() as *const ::core::ffi::c_char,
            b"\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if p_verbose.get() > 0 as OptInt {
        last_set_msg((*sgp).sg_script_ctx);
    }
}
unsafe extern "C" fn hlgroup2dict(
    mut hl: *mut Dict,
    mut ns_id: NS,
    mut hl_id: ::core::ffi::c_int,
    mut arena: *mut Arena,
) -> bool {
    let mut sgp: *mut HlGroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
        .offset((hl_id - 1 as ::core::ffi::c_int) as isize);
    let mut ns: NS = ns_id;
    let mut link: ::core::ffi::c_int = if ns_id == 0 as ::core::ffi::c_int {
        (*sgp).sg_link
    } else {
        ns_get_hl(&mut ns, hl_id, true_0 != 0, (*sgp).sg_set != 0)
    };
    if link == -1 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    if ns_id == 0 as ::core::ffi::c_int
        && (*sgp).sg_cleared as ::core::ffi::c_int != 0
        && (*sgp).sg_set == 0 as ::core::ffi::c_int
    {
        return false_0 != 0;
    }
    ns = ns_id;
    let mut attr: HlAttrs = syn_attr2entry(if ns_id == 0 as ::core::ffi::c_int {
        (*sgp).sg_attr
    } else {
        ns_get_hl(&mut ns, hl_id, false_0 != 0, (*sgp).sg_set != 0)
    });
    *hl = arena_dict(
        arena,
        (HLATTRS_DICT_SIZE as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t,
    );
    if attr.rgb_ae_attr & HL_DEFAULT as int32_t != 0 {
        let c2rust_fresh1 = (*hl).size;
        (*hl).size = (*hl).size.wrapping_add(1);
        *(*hl).items.offset(c2rust_fresh1 as isize) = key_value_pair {
            key: cstr_as_string(b"default\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_0 { boolean: true },
            },
        };
    }
    if link > 0 as ::core::ffi::c_int {
        '_c2rust_label: {
            if 1 as ::core::ffi::c_int <= link && link <= (*highlight_ga.ptr()).ga_len {
            } else {
                __assert_fail(
                    b"1 <= link && link <= highlight_ga.ga_len\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/highlight_group.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1661 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            }
        };
        let c2rust_fresh2 = (*hl).size;
        (*hl).size = (*hl).size.wrapping_add(1);
        *(*hl).items.offset(c2rust_fresh2 as isize) = key_value_pair {
            key: cstr_as_string(b"link\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_0 {
                    string: cstr_as_string(
                        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                            .offset((link - 1 as ::core::ffi::c_int) as isize))
                        .sg_name,
                    ),
                },
            },
        };
    }
    let mut hl_cterm: Dict = arena_dict(arena, HLATTRS_DICT_SIZE as ::core::ffi::c_int as size_t);
    hlattrs2dict(&mut *hl, None, attr, true_0 != 0, true_0 != 0);
    hlattrs2dict(
        &mut *hl,
        Some(&mut hl_cterm),
        attr,
        false_0 != 0,
        true_0 != 0,
    );
    if hl_cterm.size != 0 {
        let c2rust_fresh3 = (*hl).size;
        (*hl).size = (*hl).size.wrapping_add(1);
        *(*hl).items.offset(c2rust_fresh3 as isize) = key_value_pair {
            key: cstr_as_string(b"cterm\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed_0 { dict: hl_cterm },
            },
        };
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn ns_get_hl_defs(
    mut ns_id: NS,
    mut opts: *mut KeyDict_get_highlight,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut rv: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut link: Boolean = if (*opts).is_set__get_highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_highlight__link
        != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).link as ::core::ffi::c_int
    } else {
        true_0
    } != 0;
    let mut id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if (*opts).is_set__get_highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_highlight__name
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut create: Boolean = if (*opts).is_set__get_highlight_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_highlight__create
            != 0 as ::core::ffi::c_ulonglong
        {
            (*opts).create as ::core::ffi::c_int
        } else {
            true_0
        } != 0;
        id = if create as ::core::ffi::c_int != 0 {
            syn_check_group((*opts).name.data, (*opts).name.size)
        } else {
            syn_name2id_len((*opts).name.data, (*opts).name.size)
        };
        if id == 0 as ::core::ffi::c_int && !create {
            let mut attrs: Dict = ARRAY_DICT_INIT;
            return attrs;
        }
    } else if (*opts).is_set__get_highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_highlight__id
        != 0 as ::core::ffi::c_ulonglong
    {
        id = (*opts).id as ::core::ffi::c_int;
    }
    if id != -1 as ::core::ffi::c_int {
        if !(1 as ::core::ffi::c_int <= id && id <= (*highlight_ga.ptr()).ga_len) {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"Highlight id out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            let mut attrs_0: Dict = ARRAY_DICT_INIT;
            hlgroup2dict(
                &raw mut attrs_0,
                ns_id,
                if link as ::core::ffi::c_int != 0 {
                    id
                } else {
                    syn_get_final_id(id)
                },
                arena,
            );
            return attrs_0;
        }
    } else if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
        rv = arena_dict(arena, (*highlight_ga.ptr()).ga_len as size_t);
        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while i <= (*highlight_ga.ptr()).ga_len {
            let mut attrs_1: Dict = ARRAY_DICT_INIT;
            if hlgroup2dict(&raw mut attrs_1, ns_id, i, arena) {
                let c2rust_fresh0 = rv.size;
                rv.size = rv.size.wrapping_add(1);
                *rv.items.offset(c2rust_fresh0 as isize) = key_value_pair {
                    key: cstr_as_string(
                        (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(
                            ((if link as ::core::ffi::c_int != 0 {
                                i
                            } else {
                                syn_get_final_id(i)
                            }) - 1 as ::core::ffi::c_int) as isize,
                        ))
                        .sg_name,
                    ),
                    value: object {
                        type_0: kObjectTypeDict,
                        data: C2Rust_Unnamed_0 { dict: attrs_1 },
                    },
                };
            }
            i += 1;
        }
        return rv;
    }
    return ARRAY_DICT_INIT;
}
unsafe extern "C" fn highlight_list_arg(
    id: ::core::ffi::c_int,
    mut didh: bool,
    type_0: ::core::ffi::c_int,
    mut iarg: ::core::ffi::c_int,
    mut sarg: *const ::core::ffi::c_char,
    name: *const ::core::ffi::c_char,
) -> bool {
    if got_int.get() {
        return false_0 != 0;
    }
    if if type_0 == LIST_STRING {
        sarg.is_null() as ::core::ffi::c_int
    } else {
        (iarg == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
    } != 0
    {
        return didh;
    }
    let mut buf: [::core::ffi::c_char; 100] = [0; 100];
    let mut ts: *const ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
    if type_0 == LIST_INT {
        snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 100]>(),
            b"%d\0".as_ptr() as *const ::core::ffi::c_char,
            iarg - 1 as ::core::ffi::c_int,
        );
    } else if type_0 == LIST_STRING {
        ts = sarg;
    } else {
        buf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (*hl_attr_table.ptr())[i as usize] != 0 as ::core::ffi::c_int {
            if (*hl_attr_table.ptr())[i as usize] & HL_UNDERLINE_MASK != 0
                && iarg & HL_UNDERLINE_MASK == (*hl_attr_table.ptr())[i as usize]
                || (*hl_attr_table.ptr())[i as usize] & HL_UNDERLINE_MASK == 0
                    && iarg & (*hl_attr_table.ptr())[i as usize] != 0
            {
                if buf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != NUL {
                    xstrlcat(
                        &raw mut buf as *mut ::core::ffi::c_char,
                        b",\0".as_ptr() as *const ::core::ffi::c_char,
                        100 as size_t,
                    );
                }
                xstrlcat(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    (*hl_name_table.ptr())[i as usize] as *const ::core::ffi::c_char,
                    100 as size_t,
                );
                if (*hl_attr_table.ptr())[i as usize] & HL_UNDERLINE_MASK == 0 {
                    iarg &= !(*hl_attr_table.ptr())[i as usize];
                }
            }
            i += 1;
        }
    }
    syn_list_header(
        didh,
        vim_strsize(ts) + strlen(name) as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
        id,
        false_0 != 0,
    );
    didh = true_0 != 0;
    if !got_int.get() {
        if *name as ::core::ffi::c_int != NUL {
            msg_puts_hl(name, HLF_D as ::core::ffi::c_int, false_0 != 0);
            msg_puts_hl(
                b"=\0".as_ptr() as *const ::core::ffi::c_char,
                HLF_D as ::core::ffi::c_int,
                false_0 != 0,
            );
        }
        msg_outtrans(ts, 0 as ::core::ffi::c_int, false_0 != 0);
    }
    return didh;
}
pub unsafe extern "C" fn highlight_has_attr(
    id: ::core::ffi::c_int,
    flag: ::core::ffi::c_int,
    modec: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    if id <= 0 as ::core::ffi::c_int || id > (*highlight_ga.ptr()).ga_len {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    let mut attr: ::core::ffi::c_int = 0;
    if modec == 'g' as ::core::ffi::c_int {
        attr = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
            .offset((id - 1 as ::core::ffi::c_int) as isize))
        .sg_gui;
    } else {
        attr = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
            .offset((id - 1 as ::core::ffi::c_int) as isize))
        .sg_cterm;
    }
    if flag & HL_UNDERLINE_MASK != 0 {
        let mut ul: ::core::ffi::c_int = attr & HL_UNDERLINE_MASK;
        return if ul == flag {
            b"1\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            ::core::ptr::null::<::core::ffi::c_char>()
        };
    } else {
        return if attr & flag != 0 {
            b"1\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            ::core::ptr::null::<::core::ffi::c_char>()
        };
    };
}
pub unsafe extern "C" fn highlight_color(
    id: ::core::ffi::c_int,
    what: *const ::core::ffi::c_char,
    modec: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    static name: GlobalCell<[::core::ffi::c_char; 20]> = GlobalCell::new([0; 20]);
    let mut fg: bool = false_0 != 0;
    let mut sp: bool = false_0 != 0;
    let mut font: bool = false_0 != 0;
    if id <= 0 as ::core::ffi::c_int || id > (*highlight_ga.ptr()).ga_len {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if (if (*what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        < 'A' as ::core::ffi::c_int
        || *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            > 'Z' as ::core::ffi::c_int
    {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    } else {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
    }) == 'f' as ::core::ffi::c_int
        && (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 'g' as ::core::ffi::c_int
    {
        fg = true_0 != 0;
    } else if (if (*what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        < 'A' as ::core::ffi::c_int
        || *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            > 'Z' as ::core::ffi::c_int
    {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    } else {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
    }) == 'f' as ::core::ffi::c_int
        && (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 'o' as ::core::ffi::c_int
        && (if (*what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 'n' as ::core::ffi::c_int
        && (if (*what.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *what.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *what.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *what.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 't' as ::core::ffi::c_int
    {
        font = true_0 != 0;
    } else if (if (*what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        < 'A' as ::core::ffi::c_int
        || *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            > 'Z' as ::core::ffi::c_int
    {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    } else {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
    }) == 's' as ::core::ffi::c_int
        && (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 'p' as ::core::ffi::c_int
    {
        sp = true_0 != 0;
    } else if !((if (*what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        < 'A' as ::core::ffi::c_int
        || *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            > 'Z' as ::core::ffi::c_int
    {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    } else {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
    }) == 'b' as ::core::ffi::c_int
        && (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 'g' as ::core::ffi::c_int)
    {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    let mut n: ::core::ffi::c_int = 0;
    if modec == 'g' as ::core::ffi::c_int {
        if *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '#' as ::core::ffi::c_int
            && ui_rgb_attached() as ::core::ffi::c_int != 0
        {
            if fg {
                n = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_fg as ::core::ffi::c_int;
            } else if sp {
                n = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_sp as ::core::ffi::c_int;
            } else {
                n = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_bg as ::core::ffi::c_int;
            }
            if n < 0 as ::core::ffi::c_int || n > 0xffffff as ::core::ffi::c_int {
                return ::core::ptr::null::<::core::ffi::c_char>();
            }
            snprintf(
                name.ptr() as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
                b"#%06x\0".as_ptr() as *const ::core::ffi::c_char,
                n,
            );
            return name.ptr() as *mut ::core::ffi::c_char;
        }
        if fg {
            return coloridx_to_name(
                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_fg_idx,
                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_fg as ::core::ffi::c_int,
                name.ptr() as *mut ::core::ffi::c_char,
            );
        } else if sp {
            return coloridx_to_name(
                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_sp_idx,
                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_sp as ::core::ffi::c_int,
                name.ptr() as *mut ::core::ffi::c_char,
            );
        } else {
            return coloridx_to_name(
                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_bg_idx,
                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_rgb_bg as ::core::ffi::c_int,
                name.ptr() as *mut ::core::ffi::c_char,
            );
        }
    }
    if font as ::core::ffi::c_int != 0 || sp as ::core::ffi::c_int != 0 {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if modec == 'c' as ::core::ffi::c_int {
        if fg {
            n = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                .offset((id - 1 as ::core::ffi::c_int) as isize))
            .sg_cterm_fg
                - 1 as ::core::ffi::c_int;
        } else {
            n = (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                .offset((id - 1 as ::core::ffi::c_int) as isize))
            .sg_cterm_bg
                - 1 as ::core::ffi::c_int;
        }
        if n < 0 as ::core::ffi::c_int {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        snprintf(
            name.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
            b"%d\0".as_ptr() as *const ::core::ffi::c_char,
            n,
        );
        return name.ptr() as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn syn_list_header(
    did_header: bool,
    outlen: ::core::ffi::c_int,
    id: ::core::ffi::c_int,
    mut force_newline: bool,
) -> bool {
    let mut endcol: ::core::ffi::c_int = 19 as ::core::ffi::c_int;
    let mut newline: bool = true_0 != 0;
    let mut name_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut adjust: bool = true_0 != 0;
    if !did_header {
        if !ui_has(kUIMessages) || msg_col.get() > 0 as ::core::ffi::c_int {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        if got_int.get() {
            return true_0 != 0;
        }
        name_col = msg_outtrans(
            (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                .offset((id - 1 as ::core::ffi::c_int) as isize))
            .sg_name,
            0 as ::core::ffi::c_int,
            false_0 != 0,
        );
        msg_col.set(name_col);
        endcol = 15 as ::core::ffi::c_int;
    } else if (ui_has(kUIMessages) as ::core::ffi::c_int != 0 || msg_silent.get() != 0)
        && !force_newline
    {
        msg_putchar(' ' as ::core::ffi::c_int);
        adjust = false_0 != 0;
    } else if msg_col.get() + outlen + 1 as ::core::ffi::c_int >= Columns.get()
        || force_newline as ::core::ffi::c_int != 0
    {
        msg_putchar('\n' as ::core::ffi::c_int);
        if got_int.get() {
            return true_0 != 0;
        }
    } else if msg_col.get() >= endcol {
        newline = false_0 != 0;
    }
    if adjust {
        if msg_col.get() >= endcol {
            endcol = msg_col.get() + 1 as ::core::ffi::c_int;
        }
        msg_advance(endcol);
    }
    if !did_header {
        if endcol == Columns.get() - 1 as ::core::ffi::c_int && endcol <= name_col {
            msg_putchar(' ' as ::core::ffi::c_int);
        }
        msg_puts_hl(
            b"xxx\0".as_ptr() as *const ::core::ffi::c_char,
            id,
            false_0 != 0,
        );
        msg_putchar(' ' as ::core::ffi::c_int);
    }
    return newline;
}
unsafe extern "C" fn set_hl_attr(mut idx: ::core::ffi::c_int) {
    let mut at_en: HlAttrs = HLATTRS_INIT;
    let mut sgp: *mut HlGroup =
        ((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize);
    at_en.cterm_ae_attr = (*sgp).sg_cterm as int32_t;
    at_en.cterm_fg_color = (*sgp).sg_cterm_fg as int16_t;
    at_en.cterm_bg_color = (*sgp).sg_cterm_bg as int16_t;
    at_en.rgb_ae_attr = (*sgp).sg_gui as int32_t;
    at_en.rgb_fg_color = if (*sgp).sg_rgb_fg_idx != kColorIdxNone as ::core::ffi::c_int {
        (*sgp).sg_rgb_fg
    } else {
        -1 as RgbValue
    };
    at_en.rgb_bg_color = if (*sgp).sg_rgb_bg_idx != kColorIdxNone as ::core::ffi::c_int {
        (*sgp).sg_rgb_bg
    } else {
        -1 as RgbValue
    };
    at_en.rgb_sp_color = if (*sgp).sg_rgb_sp_idx != kColorIdxNone as ::core::ffi::c_int {
        (*sgp).sg_rgb_sp
    } else {
        -1 as RgbValue
    };
    at_en.hl_blend = (*sgp).sg_blend as int32_t;
    (*sgp).sg_attr = hl_get_syn_attr(
        0 as ::core::ffi::c_int,
        idx + 1 as ::core::ffi::c_int,
        at_en,
    );
    if cursor_mode_uses_syn_id(idx + 1 as ::core::ffi::c_int) {
        ui_mode_info_set();
    }
}
pub unsafe extern "C" fn syn_name2id(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '@' as ::core::ffi::c_int
    {
        return syn_check_group(name, strlen(name));
    }
    return syn_name2id_len(name, strlen(name));
}
pub unsafe extern "C" fn syn_name2id_len(
    mut name: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut name_u: [::core::ffi::c_char; 201] = [0; 201];
    if len == 0 as size_t || len > MAX_SYN_NAME as size_t {
        return 0 as ::core::ffi::c_int;
    }
    vim_memcpy_up(&raw mut name_u as *mut ::core::ffi::c_char, name, len);
    name_u[len as usize] = NUL as ::core::ffi::c_char;
    return map_get_cstr_t_int(
        highlight_unames.ptr(),
        &raw mut name_u as *mut ::core::ffi::c_char as cstr_t,
    );
}
pub unsafe extern "C" fn syn_name2attr(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut id: ::core::ffi::c_int = syn_name2id(name);
    if id != 0 as ::core::ffi::c_int {
        return syn_id2attr(id);
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn highlight_exists(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return (syn_name2id(name) > 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
pub unsafe extern "C" fn syn_id2name(mut id: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    if id <= 0 as ::core::ffi::c_int || id > (*highlight_ga.ptr()).ga_len {
        return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    return (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
        .offset((id - 1 as ::core::ffi::c_int) as isize))
    .sg_name;
}
pub unsafe extern "C" fn syn_check_group(
    mut name: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    if len > MAX_SYN_NAME as size_t {
        emsg(gettext(
            &raw const e_highlight_group_name_too_long as *const ::core::ffi::c_char,
        ));
        return 0 as ::core::ffi::c_int;
    }
    let mut id: ::core::ffi::c_int = syn_name2id_len(name, len);
    if id == 0 as ::core::ffi::c_int {
        return syn_add_group(name, len);
    }
    return id;
}
unsafe extern "C" fn syn_add_group(
    mut name: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut i: size_t = 0 as size_t;
    while i < len {
        let mut c: ::core::ffi::c_int = *name.offset(i as isize) as uint8_t as ::core::ffi::c_int;
        if !vim_isprintc(c) {
            emsg(gettext(
                b"E669: Unprintable character in group name\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return 0 as ::core::ffi::c_int;
        } else if !(c as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && c as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || c as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && c as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(c) as ::core::ffi::c_int != 0)
            && c != '_' as ::core::ffi::c_int
            && c != '.' as ::core::ffi::c_int
            && c != '@' as ::core::ffi::c_int
            && c != '-' as ::core::ffi::c_int
        {
            msg_source(HLF_W as ::core::ffi::c_int);
            emsg(gettext(
                &raw const e_highlight_group_name_invalid_char as *const ::core::ffi::c_char,
            ));
            return 0 as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    let mut scoped_parent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if len > 1 as size_t
        && *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '@' as ::core::ffi::c_int
    {
        let mut delim: *mut ::core::ffi::c_char =
            xmemrchr(name as *const ::core::ffi::c_void, '.' as uint8_t, len)
                as *mut ::core::ffi::c_char;
        if !delim.is_null() {
            scoped_parent = syn_check_group(name, delim.offset_from(name) as size_t);
        }
    }
    if (*highlight_ga.ptr()).ga_data.is_null() {
        (*highlight_ga.ptr()).ga_itemsize = ::core::mem::size_of::<HlGroup>() as ::core::ffi::c_int;
        ga_set_growsize(highlight_ga.ptr(), 10 as ::core::ffi::c_int);
        ga_grow(highlight_ga.ptr(), 300 as ::core::ffi::c_int);
    }
    if (*highlight_ga.ptr()).ga_len >= MAX_HL_ID as ::core::ffi::c_int {
        emsg(gettext(
            b"E849: Too many highlight and syntax groups\0".as_ptr() as *const ::core::ffi::c_char,
        ));
        return 0 as ::core::ffi::c_int;
    }
    let mut hlgp: *mut HlGroup =
        ga_append_via_ptr(highlight_ga.ptr(), ::core::mem::size_of::<HlGroup>()) as *mut HlGroup;
    memset(
        hlgp as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<HlGroup>(),
    );
    (*hlgp).sg_name = arena_memdupz(highlight_arena.ptr(), name, len);
    (*hlgp).sg_rgb_bg = -1 as ::core::ffi::c_int as RgbValue;
    (*hlgp).sg_rgb_fg = -1 as ::core::ffi::c_int as RgbValue;
    (*hlgp).sg_rgb_sp = -1 as ::core::ffi::c_int as RgbValue;
    (*hlgp).sg_rgb_bg_idx = kColorIdxNone as ::core::ffi::c_int;
    (*hlgp).sg_rgb_fg_idx = kColorIdxNone as ::core::ffi::c_int;
    (*hlgp).sg_rgb_sp_idx = kColorIdxNone as ::core::ffi::c_int;
    (*hlgp).sg_blend = -1 as ::core::ffi::c_int;
    (*hlgp).sg_name_u = arena_memdupz(highlight_arena.ptr(), name, len);
    (*hlgp).sg_parent = scoped_parent;
    (*hlgp).sg_cleared = true_0 != 0;
    vim_strup((*hlgp).sg_name_u);
    let mut id: ::core::ffi::c_int = (*highlight_ga.ptr()).ga_len;
    map_put_cstr_t_int(highlight_unames.ptr(), (*hlgp).sg_name_u as cstr_t, id);
    return id;
}
pub unsafe extern "C" fn syn_id2attr(mut hl_id: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut optional: bool = false_0 != 0;
    return syn_ns_id2attr(-1 as ::core::ffi::c_int, hl_id, &raw mut optional);
}
pub unsafe extern "C" fn syn_ns_id2attr(
    mut ns_id: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
    mut optional: *mut bool,
) -> ::core::ffi::c_int {
    if syn_ns_get_final_id(&raw mut ns_id, &raw mut hl_id) {
        *optional = false_0 != 0;
    }
    let mut sgp: *mut HlGroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
        .offset((hl_id - 1 as ::core::ffi::c_int) as isize);
    let mut attr: ::core::ffi::c_int =
        ns_get_hl(&mut ns_id, hl_id, false_0 != 0, (*sgp).sg_set != 0);
    if attr >= 0 as ::core::ffi::c_int
        || *optional as ::core::ffi::c_int != 0 && ns_id > 0 as ::core::ffi::c_int
    {
        return attr;
    }
    return (*sgp).sg_attr;
}
pub unsafe extern "C" fn syn_get_final_id(mut hl_id: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut ns_id: ::core::ffi::c_int = (*curwin.get()).w_ns_hl_active;
    syn_ns_get_final_id(&raw mut ns_id, &raw mut hl_id);
    return hl_id;
}
pub unsafe extern "C" fn syn_ns_get_final_id(
    mut ns_id: *mut ::core::ffi::c_int,
    mut hl_idp: *mut ::core::ffi::c_int,
) -> bool {
    let mut hl_id: ::core::ffi::c_int = *hl_idp;
    let mut used: bool = false_0 != 0;
    if hl_id > (*highlight_ga.ptr()).ga_len || hl_id < 1 as ::core::ffi::c_int {
        *hl_idp = 0 as ::core::ffi::c_int;
        return false_0 != 0;
    }
    let mut count: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
    loop {
        count -= 1;
        if count < 0 as ::core::ffi::c_int {
            break;
        }
        let mut sgp: *mut HlGroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
            .offset((hl_id - 1 as ::core::ffi::c_int) as isize);
        let mut check: ::core::ffi::c_int =
            ns_get_hl(&mut *ns_id, hl_id, true_0 != 0, (*sgp).sg_set != 0);
        if check == 0 as ::core::ffi::c_int {
            *hl_idp = hl_id;
            return true_0 != 0;
        } else if check > 0 as ::core::ffi::c_int {
            used = true_0 != 0;
            hl_id = check;
        } else if (*sgp).sg_link > 0 as ::core::ffi::c_int
            && (*sgp).sg_link <= (*highlight_ga.ptr()).ga_len
        {
            hl_id = (*sgp).sg_link;
        } else {
            if !((*sgp).sg_cleared as ::core::ffi::c_int != 0
                && (*sgp).sg_parent > 0 as ::core::ffi::c_int)
            {
                break;
            }
            hl_id = (*sgp).sg_parent;
        }
    }
    *hl_idp = hl_id;
    return used;
}
pub unsafe extern "C" fn highlight_attr_set_all() {
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while idx < (*highlight_ga.ptr()).ga_len {
        let mut sgp: *mut HlGroup =
            ((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize);
        if (*sgp).sg_rgb_bg_idx == kColorIdxFg as ::core::ffi::c_int {
            (*sgp).sg_rgb_bg = normal_fg.get();
        } else if (*sgp).sg_rgb_bg_idx == kColorIdxBg as ::core::ffi::c_int {
            (*sgp).sg_rgb_bg = normal_bg.get();
        }
        if (*sgp).sg_rgb_fg_idx == kColorIdxFg as ::core::ffi::c_int {
            (*sgp).sg_rgb_fg = normal_fg.get();
        } else if (*sgp).sg_rgb_fg_idx == kColorIdxBg as ::core::ffi::c_int {
            (*sgp).sg_rgb_fg = normal_bg.get();
        }
        if (*sgp).sg_rgb_sp_idx == kColorIdxFg as ::core::ffi::c_int {
            (*sgp).sg_rgb_sp = normal_fg.get();
        } else if (*sgp).sg_rgb_sp_idx == kColorIdxBg as ::core::ffi::c_int {
            (*sgp).sg_rgb_sp = normal_bg.get();
        }
        set_hl_attr(idx);
        idx += 1;
    }
}
unsafe extern "C" fn combine_stl_hlt(
    mut id: ::core::ffi::c_int,
    mut id_S: ::core::ffi::c_int,
    mut id_alt: ::core::ffi::c_int,
    mut hlcnt: ::core::ffi::c_int,
    mut i: ::core::ffi::c_int,
    mut hlf: ::core::ffi::c_int,
    mut table: *mut ::core::ffi::c_int,
) {
    let hlt: *mut HlGroup = (*highlight_ga.ptr()).ga_data as *mut HlGroup;
    if id_alt == 0 as ::core::ffi::c_int {
        memset(
            hlt.offset((hlcnt + i) as isize) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<HlGroup>(),
        );
        (*hlt.offset((hlcnt + i) as isize)).sg_cterm = (*highlight_attr.ptr())[hlf as usize];
        (*hlt.offset((hlcnt + i) as isize)).sg_gui = (*highlight_attr.ptr())[hlf as usize];
    } else {
        memmove(
            hlt.offset((hlcnt + i) as isize) as *mut ::core::ffi::c_void,
            hlt.offset((id_alt - 1 as ::core::ffi::c_int) as isize) as *const ::core::ffi::c_void,
            ::core::mem::size_of::<HlGroup>(),
        );
    }
    (*hlt.offset((hlcnt + i) as isize)).sg_link = 0 as ::core::ffi::c_int;
    (*hlt.offset((hlcnt + i) as isize)).sg_cterm ^=
        (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_cterm
            ^ (*hlt.offset((id_S - 1 as ::core::ffi::c_int) as isize)).sg_cterm;
    if (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_cterm_fg
        != (*hlt.offset((id_S - 1 as ::core::ffi::c_int) as isize)).sg_cterm_fg
    {
        (*hlt.offset((hlcnt + i) as isize)).sg_cterm_fg =
            (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_cterm_fg;
    }
    if (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_cterm_bg
        != (*hlt.offset((id_S - 1 as ::core::ffi::c_int) as isize)).sg_cterm_bg
    {
        (*hlt.offset((hlcnt + i) as isize)).sg_cterm_bg =
            (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_cterm_bg;
    }
    (*hlt.offset((hlcnt + i) as isize)).sg_gui ^=
        (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_gui
            ^ (*hlt.offset((id_S - 1 as ::core::ffi::c_int) as isize)).sg_gui;
    if (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_rgb_fg
        != (*hlt.offset((id_S - 1 as ::core::ffi::c_int) as isize)).sg_rgb_fg
    {
        (*hlt.offset((hlcnt + i) as isize)).sg_rgb_fg =
            (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_rgb_fg;
    }
    if (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_rgb_bg
        != (*hlt.offset((id_S - 1 as ::core::ffi::c_int) as isize)).sg_rgb_bg
    {
        (*hlt.offset((hlcnt + i) as isize)).sg_rgb_bg =
            (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_rgb_bg;
    }
    if (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_rgb_sp
        != (*hlt.offset((id_S - 1 as ::core::ffi::c_int) as isize)).sg_rgb_sp
    {
        (*hlt.offset((hlcnt + i) as isize)).sg_rgb_sp =
            (*hlt.offset((id - 1 as ::core::ffi::c_int) as isize)).sg_rgb_sp;
    }
    (*highlight_ga.ptr()).ga_len = hlcnt + i + 1 as ::core::ffi::c_int;
    set_hl_attr(hlcnt + i);
    *table.offset(i as isize) = syn_id2attr(hlcnt + i + 1 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn highlight_changed() {
    let mut userhl: [::core::ffi::c_char; 30] = [0; 30];
    let mut id_S: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut id_SNC: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    need_highlight_changed.set(false_0 != 0);
    (*highlight_attr.ptr())[HLF_NONE as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int;
    let mut hlf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while hlf < HLF_COUNT as ::core::ffi::c_int {
        let mut id: ::core::ffi::c_int = syn_check_group(
            *(hlf_names.ptr() as *mut *const ::core::ffi::c_char).offset(hlf as isize),
            strlen(*(hlf_names.ptr() as *mut *const ::core::ffi::c_char).offset(hlf as isize)),
        );
        if id == 0 as ::core::ffi::c_int {
            abort();
        }
        let mut ns_id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut final_id: ::core::ffi::c_int = id;
        syn_ns_get_final_id(&raw mut ns_id, &raw mut final_id);
        if hlf == HLF_SNC as ::core::ffi::c_int {
            id_SNC = final_id;
        } else if hlf == HLF_S as ::core::ffi::c_int {
            id_S = final_id;
        }
        (*highlight_attr.ptr())[hlf as usize] = hl_get_ui_attr(
            ns_id,
            hlf,
            final_id,
            hlf == HLF_INACTIVE as ::core::ffi::c_int,
        );
        if (*highlight_attr.ptr())[hlf as usize] != (*highlight_attr_last.ptr())[hlf as usize] {
            if hlf == HLF_MSG as ::core::ffi::c_int {
                clear_cmdline.set(true_0 != 0);
                let mut attrs: HlAttrs = syn_attr2entry((*highlight_attr.ptr())[hlf as usize]);
                (*msg_grid.ptr()).blending = attrs.hl_blend > -1 as int32_t;
            }
            ui_call_hl_group_set(
                cstr_as_string(
                    *(hlf_names.ptr() as *mut *const ::core::ffi::c_char).offset(hlf as isize),
                ),
                (*highlight_attr.ptr())[hlf as usize] as Integer,
            );
            (*highlight_attr_last.ptr())[hlf as usize] = (*highlight_attr.ptr())[hlf as usize];
        }
        hlf += 1;
    }
    ga_grow(highlight_ga.ptr(), 10 as ::core::ffi::c_int);
    let mut hlcnt: ::core::ffi::c_int = (*highlight_ga.ptr()).ga_len;
    if id_S == -1 as ::core::ffi::c_int {
        memset(
            ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                .offset((hlcnt + 9 as ::core::ffi::c_int) as isize)
                as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<HlGroup>(),
        );
        id_S = hlcnt + 10 as ::core::ffi::c_int;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 9 as ::core::ffi::c_int {
        snprintf(
            &raw mut userhl as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
            b"User%d\0".as_ptr() as *const ::core::ffi::c_char,
            i + 1 as ::core::ffi::c_int,
        );
        let mut id_0: ::core::ffi::c_int = syn_name2id(&raw mut userhl as *mut ::core::ffi::c_char);
        if id_0 == 0 as ::core::ffi::c_int {
            (*highlight_user.ptr())[i as usize] = 0 as ::core::ffi::c_int;
            (*highlight_stlnc.ptr())[i as usize] = 0 as ::core::ffi::c_int;
        } else {
            (*highlight_user.ptr())[i as usize] = syn_id2attr(id_0);
            combine_stl_hlt(
                id_0,
                id_S,
                id_SNC,
                hlcnt,
                i,
                HLF_SNC as ::core::ffi::c_int,
                highlight_stlnc.ptr() as *mut ::core::ffi::c_int,
            );
        }
        i += 1;
    }
    (*highlight_ga.ptr()).ga_len = hlcnt;
    decor_provider_invalidate_hl();
}
pub unsafe extern "C" fn set_context_in_highlight_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) {
    (*xp).xp_context = EXPAND_HIGHLIGHT as ::core::ffi::c_int;
    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
    include_link.set(2 as ::core::ffi::c_int);
    include_default.set(1 as ::core::ffi::c_int);
    if *arg as ::core::ffi::c_int == NUL {
        return;
    }
    let mut p: *const ::core::ffi::c_char = skiptowhite(arg);
    if *p as ::core::ffi::c_int == NUL {
        return;
    }
    include_default.set(0 as ::core::ffi::c_int);
    if strncmp(
        b"default\0".as_ptr() as *const ::core::ffi::c_char,
        arg,
        p.offset_from(arg) as ::core::ffi::c_uint as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        arg = skipwhite(p);
        (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
        p = skiptowhite(arg);
    }
    if *p as ::core::ffi::c_int == NUL {
        return;
    }
    include_link.set(0 as ::core::ffi::c_int);
    if *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'i' as ::core::ffi::c_int
        && *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'N' as ::core::ffi::c_int
    {
        highlight_list();
    }
    if strncmp(
        b"link\0".as_ptr() as *const ::core::ffi::c_char,
        arg,
        p.offset_from(arg) as ::core::ffi::c_uint as size_t,
    ) == 0 as ::core::ffi::c_int
        || strncmp(
            b"clear\0".as_ptr() as *const ::core::ffi::c_char,
            arg,
            p.offset_from(arg) as ::core::ffi::c_uint as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        (*xp).xp_pattern = skipwhite(p);
        p = skiptowhite((*xp).xp_pattern);
        if *p as ::core::ffi::c_int != NUL {
            (*xp).xp_pattern = skipwhite(p);
            p = skiptowhite((*xp).xp_pattern);
        }
    }
    if *p as ::core::ffi::c_int != NUL {
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    }
}
unsafe extern "C" fn highlight_list() {
    let mut i: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        highlight_list_two(i, HLF_D as ::core::ffi::c_int);
    }
    let mut i_0: ::core::ffi::c_int = 40 as ::core::ffi::c_int;
    loop {
        i_0 -= 1;
        if i_0 < 0 as ::core::ffi::c_int {
            break;
        }
        highlight_list_two(99 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn highlight_list_two(mut cnt: ::core::ffi::c_int, mut id: ::core::ffi::c_int) {
    msg_puts_hl(
        (b"N \x08I \x08!  \x08\0".as_ptr() as *const ::core::ffi::c_char)
            .offset((cnt / 11 as ::core::ffi::c_int) as isize),
        id,
        false_0 != 0,
    );
    msg_clr_eos();
    ui_flush();
    os_delay(
        if cnt == 99 as ::core::ffi::c_int {
            40 as uint64_t
        } else {
            (cnt as uint64_t).wrapping_mul(50 as uint64_t)
        },
        false_0 != 0,
    );
}
pub unsafe extern "C" fn get_highlight_name(
    xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    return get_highlight_name_ext(xp, idx, true_0 != 0) as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn get_highlight_name_ext(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
    mut skip_cleared: bool,
) -> *const ::core::ffi::c_char {
    if idx < 0 as ::core::ffi::c_int {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if skip_cleared as ::core::ffi::c_int != 0
        && idx < (*highlight_ga.ptr()).ga_len
        && (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cleared
            as ::core::ffi::c_int
            != 0
    {
        return b"\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if idx == (*highlight_ga.ptr()).ga_len && include_none.get() != 0 as ::core::ffi::c_int {
        return b"none\0".as_ptr() as *const ::core::ffi::c_char;
    } else if idx == (*highlight_ga.ptr()).ga_len + include_none.get()
        && include_default.get() != 0 as ::core::ffi::c_int
    {
        return b"default\0".as_ptr() as *const ::core::ffi::c_char;
    } else if idx == (*highlight_ga.ptr()).ga_len + include_none.get() + include_default.get()
        && include_link.get() != 0 as ::core::ffi::c_int
    {
        return b"link\0".as_ptr() as *const ::core::ffi::c_char;
    } else if idx
        == (*highlight_ga.ptr()).ga_len
            + include_none.get()
            + include_default.get()
            + 1 as ::core::ffi::c_int
        && include_link.get() != 0 as ::core::ffi::c_int
    {
        return b"clear\0".as_ptr() as *const ::core::ffi::c_char;
    } else if idx >= (*highlight_ga.ptr()).ga_len {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_name;
}
/// One X11 colour-table entry. `color_name_table_T` holds a `*mut c_char`
/// because the C table did; nothing writes through it.
const fn color_entry(name: &'static ::core::ffi::CStr, color: RgbValue) -> color_name_table_T {
    color_name_table_T {
        name: name.as_ptr().cast_mut(),
        color,
    }
}
pub static color_name_table: GlobalCell<[color_name_table_T; 708]> = GlobalCell::new([
    color_entry(c"AliceBlue", 0xf0f8ff),
    color_entry(c"AntiqueWhite", 0xfaebd7),
    color_entry(c"AntiqueWhite1", 0xffefdb),
    color_entry(c"AntiqueWhite2", 0xeedfcc),
    color_entry(c"AntiqueWhite3", 0xcdc0b0),
    color_entry(c"AntiqueWhite4", 0x8b8378),
    color_entry(c"Aqua", 0x00ffff),
    color_entry(c"Aquamarine", 0x7fffd4),
    color_entry(c"Aquamarine1", 0x7fffd4),
    color_entry(c"Aquamarine2", 0x76eec6),
    color_entry(c"Aquamarine3", 0x66cdaa),
    color_entry(c"Aquamarine4", 0x458b74),
    color_entry(c"Azure", 0xf0ffff),
    color_entry(c"Azure1", 0xf0ffff),
    color_entry(c"Azure2", 0xe0eeee),
    color_entry(c"Azure3", 0xc1cdcd),
    color_entry(c"Azure4", 0x838b8b),
    color_entry(c"Beige", 0xf5f5dc),
    color_entry(c"Bisque", 0xffe4c4),
    color_entry(c"Bisque1", 0xffe4c4),
    color_entry(c"Bisque2", 0xeed5b7),
    color_entry(c"Bisque3", 0xcdb79e),
    color_entry(c"Bisque4", 0x8b7d6b),
    color_entry(c"Black", 0x000000),
    color_entry(c"BlanchedAlmond", 0xffebcd),
    color_entry(c"Blue", 0x0000ff),
    color_entry(c"Blue1", 0x0000ff),
    color_entry(c"Blue2", 0x0000ee),
    color_entry(c"Blue3", 0x0000cd),
    color_entry(c"Blue4", 0x00008b),
    color_entry(c"BlueViolet", 0x8a2be2),
    color_entry(c"Brown", 0xa52a2a),
    color_entry(c"Brown1", 0xff4040),
    color_entry(c"Brown2", 0xee3b3b),
    color_entry(c"Brown3", 0xcd3333),
    color_entry(c"Brown4", 0x8b2323),
    color_entry(c"BurlyWood", 0xdeb887),
    color_entry(c"Burlywood1", 0xffd39b),
    color_entry(c"Burlywood2", 0xeec591),
    color_entry(c"Burlywood3", 0xcdaa7d),
    color_entry(c"Burlywood4", 0x8b7355),
    color_entry(c"CadetBlue", 0x5f9ea0),
    color_entry(c"CadetBlue1", 0x98f5ff),
    color_entry(c"CadetBlue2", 0x8ee5ee),
    color_entry(c"CadetBlue3", 0x7ac5cd),
    color_entry(c"CadetBlue4", 0x53868b),
    color_entry(c"ChartReuse", 0x7fff00),
    color_entry(c"Chartreuse1", 0x7fff00),
    color_entry(c"Chartreuse2", 0x76ee00),
    color_entry(c"Chartreuse3", 0x66cd00),
    color_entry(c"Chartreuse4", 0x458b00),
    color_entry(c"Chocolate", 0xd2691e),
    color_entry(c"Chocolate1", 0xff7f24),
    color_entry(c"Chocolate2", 0xee7621),
    color_entry(c"Chocolate3", 0xcd661d),
    color_entry(c"Chocolate4", 0x8b4513),
    color_entry(c"Coral", 0xff7f50),
    color_entry(c"Coral1", 0xff7256),
    color_entry(c"Coral2", 0xee6a50),
    color_entry(c"Coral3", 0xcd5b45),
    color_entry(c"Coral4", 0x8b3e2f),
    color_entry(c"CornFlowerBlue", 0x6495ed),
    color_entry(c"Cornsilk", 0xfff8dc),
    color_entry(c"Cornsilk1", 0xfff8dc),
    color_entry(c"Cornsilk2", 0xeee8cd),
    color_entry(c"Cornsilk3", 0xcdc8b1),
    color_entry(c"Cornsilk4", 0x8b8878),
    color_entry(c"Crimson", 0xdc143c),
    color_entry(c"Cyan", 0x00ffff),
    color_entry(c"Cyan1", 0x00ffff),
    color_entry(c"Cyan2", 0x00eeee),
    color_entry(c"Cyan3", 0x00cdcd),
    color_entry(c"Cyan4", 0x008b8b),
    color_entry(c"DarkBlue", 0x00008b),
    color_entry(c"DarkCyan", 0x008b8b),
    color_entry(c"DarkGoldenrod", 0xb8860b),
    color_entry(c"DarkGoldenrod1", 0xffb90f),
    color_entry(c"DarkGoldenrod2", 0xeead0e),
    color_entry(c"DarkGoldenrod3", 0xcd950c),
    color_entry(c"DarkGoldenrod4", 0x8b6508),
    color_entry(c"DarkGray", 0xa9a9a9),
    color_entry(c"DarkGreen", 0x006400),
    color_entry(c"DarkGrey", 0xa9a9a9),
    color_entry(c"DarkKhaki", 0xbdb76b),
    color_entry(c"DarkMagenta", 0x8b008b),
    color_entry(c"DarkOliveGreen", 0x556b2f),
    color_entry(c"DarkOliveGreen1", 0xcaff70),
    color_entry(c"DarkOliveGreen2", 0xbcee68),
    color_entry(c"DarkOliveGreen3", 0xa2cd5a),
    color_entry(c"DarkOliveGreen4", 0x6e8b3d),
    color_entry(c"DarkOrange", 0xff8c00),
    color_entry(c"DarkOrange1", 0xff7f00),
    color_entry(c"DarkOrange2", 0xee7600),
    color_entry(c"DarkOrange3", 0xcd6600),
    color_entry(c"DarkOrange4", 0x8b4500),
    color_entry(c"DarkOrchid", 0x9932cc),
    color_entry(c"DarkOrchid1", 0xbf3eff),
    color_entry(c"DarkOrchid2", 0xb23aee),
    color_entry(c"DarkOrchid3", 0x9a32cd),
    color_entry(c"DarkOrchid4", 0x68228b),
    color_entry(c"DarkRed", 0x8b0000),
    color_entry(c"DarkSalmon", 0xe9967a),
    color_entry(c"DarkSeaGreen", 0x8fbc8f),
    color_entry(c"DarkSeaGreen1", 0xc1ffc1),
    color_entry(c"DarkSeaGreen2", 0xb4eeb4),
    color_entry(c"DarkSeaGreen3", 0x9bcd9b),
    color_entry(c"DarkSeaGreen4", 0x698b69),
    color_entry(c"DarkSlateBlue", 0x483d8b),
    color_entry(c"DarkSlateGray", 0x2f4f4f),
    color_entry(c"DarkSlateGray1", 0x97ffff),
    color_entry(c"DarkSlateGray2", 0x8deeee),
    color_entry(c"DarkSlateGray3", 0x79cdcd),
    color_entry(c"DarkSlateGray4", 0x528b8b),
    color_entry(c"DarkSlateGrey", 0x2f4f4f),
    color_entry(c"DarkTurquoise", 0x00ced1),
    color_entry(c"DarkViolet", 0x9400d3),
    color_entry(c"DarkYellow", 0xbbbb00),
    color_entry(c"DeepPink", 0xff1493),
    color_entry(c"DeepPink1", 0xff1493),
    color_entry(c"DeepPink2", 0xee1289),
    color_entry(c"DeepPink3", 0xcd1076),
    color_entry(c"DeepPink4", 0x8b0a50),
    color_entry(c"DeepSkyBlue", 0x00bfff),
    color_entry(c"DeepSkyBlue1", 0x00bfff),
    color_entry(c"DeepSkyBlue2", 0x00b2ee),
    color_entry(c"DeepSkyBlue3", 0x009acd),
    color_entry(c"DeepSkyBlue4", 0x00688b),
    color_entry(c"DimGray", 0x696969),
    color_entry(c"DimGrey", 0x696969),
    color_entry(c"DodgerBlue", 0x1e90ff),
    color_entry(c"DodgerBlue1", 0x1e90ff),
    color_entry(c"DodgerBlue2", 0x1c86ee),
    color_entry(c"DodgerBlue3", 0x1874cd),
    color_entry(c"DodgerBlue4", 0x104e8b),
    color_entry(c"Firebrick", 0xb22222),
    color_entry(c"Firebrick1", 0xff3030),
    color_entry(c"Firebrick2", 0xee2c2c),
    color_entry(c"Firebrick3", 0xcd2626),
    color_entry(c"Firebrick4", 0x8b1a1a),
    color_entry(c"FloralWhite", 0xfffaf0),
    color_entry(c"ForestGreen", 0x228b22),
    color_entry(c"Fuchsia", 0xff00ff),
    color_entry(c"Gainsboro", 0xdcdcdc),
    color_entry(c"GhostWhite", 0xf8f8ff),
    color_entry(c"Gold", 0xffd700),
    color_entry(c"Gold1", 0xffd700),
    color_entry(c"Gold2", 0xeec900),
    color_entry(c"Gold3", 0xcdad00),
    color_entry(c"Gold4", 0x8b7500),
    color_entry(c"Goldenrod", 0xdaa520),
    color_entry(c"Goldenrod1", 0xffc125),
    color_entry(c"Goldenrod2", 0xeeb422),
    color_entry(c"Goldenrod3", 0xcd9b1d),
    color_entry(c"Goldenrod4", 0x8b6914),
    color_entry(c"Gray", 0x808080),
    color_entry(c"Gray0", 0x000000),
    color_entry(c"Gray1", 0x030303),
    color_entry(c"Gray10", 0x1a1a1a),
    color_entry(c"Gray100", 0xffffff),
    color_entry(c"Gray11", 0x1c1c1c),
    color_entry(c"Gray12", 0x1f1f1f),
    color_entry(c"Gray13", 0x212121),
    color_entry(c"Gray14", 0x242424),
    color_entry(c"Gray15", 0x262626),
    color_entry(c"Gray16", 0x292929),
    color_entry(c"Gray17", 0x2b2b2b),
    color_entry(c"Gray18", 0x2e2e2e),
    color_entry(c"Gray19", 0x303030),
    color_entry(c"Gray2", 0x050505),
    color_entry(c"Gray20", 0x333333),
    color_entry(c"Gray21", 0x363636),
    color_entry(c"Gray22", 0x383838),
    color_entry(c"Gray23", 0x3b3b3b),
    color_entry(c"Gray24", 0x3d3d3d),
    color_entry(c"Gray25", 0x404040),
    color_entry(c"Gray26", 0x424242),
    color_entry(c"Gray27", 0x454545),
    color_entry(c"Gray28", 0x474747),
    color_entry(c"Gray29", 0x4a4a4a),
    color_entry(c"Gray3", 0x080808),
    color_entry(c"Gray30", 0x4d4d4d),
    color_entry(c"Gray31", 0x4f4f4f),
    color_entry(c"Gray32", 0x525252),
    color_entry(c"Gray33", 0x545454),
    color_entry(c"Gray34", 0x575757),
    color_entry(c"Gray35", 0x595959),
    color_entry(c"Gray36", 0x5c5c5c),
    color_entry(c"Gray37", 0x5e5e5e),
    color_entry(c"Gray38", 0x616161),
    color_entry(c"Gray39", 0x636363),
    color_entry(c"Gray4", 0x0a0a0a),
    color_entry(c"Gray40", 0x666666),
    color_entry(c"Gray41", 0x696969),
    color_entry(c"Gray42", 0x6b6b6b),
    color_entry(c"Gray43", 0x6e6e6e),
    color_entry(c"Gray44", 0x707070),
    color_entry(c"Gray45", 0x737373),
    color_entry(c"Gray46", 0x757575),
    color_entry(c"Gray47", 0x787878),
    color_entry(c"Gray48", 0x7a7a7a),
    color_entry(c"Gray49", 0x7d7d7d),
    color_entry(c"Gray5", 0x0d0d0d),
    color_entry(c"Gray50", 0x7f7f7f),
    color_entry(c"Gray51", 0x828282),
    color_entry(c"Gray52", 0x858585),
    color_entry(c"Gray53", 0x878787),
    color_entry(c"Gray54", 0x8a8a8a),
    color_entry(c"Gray55", 0x8c8c8c),
    color_entry(c"Gray56", 0x8f8f8f),
    color_entry(c"Gray57", 0x919191),
    color_entry(c"Gray58", 0x949494),
    color_entry(c"Gray59", 0x969696),
    color_entry(c"Gray6", 0x0f0f0f),
    color_entry(c"Gray60", 0x999999),
    color_entry(c"Gray61", 0x9c9c9c),
    color_entry(c"Gray62", 0x9e9e9e),
    color_entry(c"Gray63", 0xa1a1a1),
    color_entry(c"Gray64", 0xa3a3a3),
    color_entry(c"Gray65", 0xa6a6a6),
    color_entry(c"Gray66", 0xa8a8a8),
    color_entry(c"Gray67", 0xababab),
    color_entry(c"Gray68", 0xadadad),
    color_entry(c"Gray69", 0xb0b0b0),
    color_entry(c"Gray7", 0x121212),
    color_entry(c"Gray70", 0xb3b3b3),
    color_entry(c"Gray71", 0xb5b5b5),
    color_entry(c"Gray72", 0xb8b8b8),
    color_entry(c"Gray73", 0xbababa),
    color_entry(c"Gray74", 0xbdbdbd),
    color_entry(c"Gray75", 0xbfbfbf),
    color_entry(c"Gray76", 0xc2c2c2),
    color_entry(c"Gray77", 0xc4c4c4),
    color_entry(c"Gray78", 0xc7c7c7),
    color_entry(c"Gray79", 0xc9c9c9),
    color_entry(c"Gray8", 0x141414),
    color_entry(c"Gray80", 0xcccccc),
    color_entry(c"Gray81", 0xcfcfcf),
    color_entry(c"Gray82", 0xd1d1d1),
    color_entry(c"Gray83", 0xd4d4d4),
    color_entry(c"Gray84", 0xd6d6d6),
    color_entry(c"Gray85", 0xd9d9d9),
    color_entry(c"Gray86", 0xdbdbdb),
    color_entry(c"Gray87", 0xdedede),
    color_entry(c"Gray88", 0xe0e0e0),
    color_entry(c"Gray89", 0xe3e3e3),
    color_entry(c"Gray9", 0x171717),
    color_entry(c"Gray90", 0xe5e5e5),
    color_entry(c"Gray91", 0xe8e8e8),
    color_entry(c"Gray92", 0xebebeb),
    color_entry(c"Gray93", 0xededed),
    color_entry(c"Gray94", 0xf0f0f0),
    color_entry(c"Gray95", 0xf2f2f2),
    color_entry(c"Gray96", 0xf5f5f5),
    color_entry(c"Gray97", 0xf7f7f7),
    color_entry(c"Gray98", 0xfafafa),
    color_entry(c"Gray99", 0xfcfcfc),
    color_entry(c"Green", 0x008000),
    color_entry(c"Green1", 0x00ff00),
    color_entry(c"Green2", 0x00ee00),
    color_entry(c"Green3", 0x00cd00),
    color_entry(c"Green4", 0x008b00),
    color_entry(c"GreenYellow", 0xadff2f),
    color_entry(c"Grey", 0x808080),
    color_entry(c"Grey0", 0x000000),
    color_entry(c"Grey1", 0x030303),
    color_entry(c"Grey10", 0x1a1a1a),
    color_entry(c"Grey100", 0xffffff),
    color_entry(c"Grey11", 0x1c1c1c),
    color_entry(c"Grey12", 0x1f1f1f),
    color_entry(c"Grey13", 0x212121),
    color_entry(c"Grey14", 0x242424),
    color_entry(c"Grey15", 0x262626),
    color_entry(c"Grey16", 0x292929),
    color_entry(c"Grey17", 0x2b2b2b),
    color_entry(c"Grey18", 0x2e2e2e),
    color_entry(c"Grey19", 0x303030),
    color_entry(c"Grey2", 0x050505),
    color_entry(c"Grey20", 0x333333),
    color_entry(c"Grey21", 0x363636),
    color_entry(c"Grey22", 0x383838),
    color_entry(c"Grey23", 0x3b3b3b),
    color_entry(c"Grey24", 0x3d3d3d),
    color_entry(c"Grey25", 0x404040),
    color_entry(c"Grey26", 0x424242),
    color_entry(c"Grey27", 0x454545),
    color_entry(c"Grey28", 0x474747),
    color_entry(c"Grey29", 0x4a4a4a),
    color_entry(c"Grey3", 0x080808),
    color_entry(c"Grey30", 0x4d4d4d),
    color_entry(c"Grey31", 0x4f4f4f),
    color_entry(c"Grey32", 0x525252),
    color_entry(c"Grey33", 0x545454),
    color_entry(c"Grey34", 0x575757),
    color_entry(c"Grey35", 0x595959),
    color_entry(c"Grey36", 0x5c5c5c),
    color_entry(c"Grey37", 0x5e5e5e),
    color_entry(c"Grey38", 0x616161),
    color_entry(c"Grey39", 0x636363),
    color_entry(c"Grey4", 0x0a0a0a),
    color_entry(c"Grey40", 0x666666),
    color_entry(c"Grey41", 0x696969),
    color_entry(c"Grey42", 0x6b6b6b),
    color_entry(c"Grey43", 0x6e6e6e),
    color_entry(c"Grey44", 0x707070),
    color_entry(c"Grey45", 0x737373),
    color_entry(c"Grey46", 0x757575),
    color_entry(c"Grey47", 0x787878),
    color_entry(c"Grey48", 0x7a7a7a),
    color_entry(c"Grey49", 0x7d7d7d),
    color_entry(c"Grey5", 0x0d0d0d),
    color_entry(c"Grey50", 0x7f7f7f),
    color_entry(c"Grey51", 0x828282),
    color_entry(c"Grey52", 0x858585),
    color_entry(c"Grey53", 0x878787),
    color_entry(c"Grey54", 0x8a8a8a),
    color_entry(c"Grey55", 0x8c8c8c),
    color_entry(c"Grey56", 0x8f8f8f),
    color_entry(c"Grey57", 0x919191),
    color_entry(c"Grey58", 0x949494),
    color_entry(c"Grey59", 0x969696),
    color_entry(c"Grey6", 0x0f0f0f),
    color_entry(c"Grey60", 0x999999),
    color_entry(c"Grey61", 0x9c9c9c),
    color_entry(c"Grey62", 0x9e9e9e),
    color_entry(c"Grey63", 0xa1a1a1),
    color_entry(c"Grey64", 0xa3a3a3),
    color_entry(c"Grey65", 0xa6a6a6),
    color_entry(c"Grey66", 0xa8a8a8),
    color_entry(c"Grey67", 0xababab),
    color_entry(c"Grey68", 0xadadad),
    color_entry(c"Grey69", 0xb0b0b0),
    color_entry(c"Grey7", 0x121212),
    color_entry(c"Grey70", 0xb3b3b3),
    color_entry(c"Grey71", 0xb5b5b5),
    color_entry(c"Grey72", 0xb8b8b8),
    color_entry(c"Grey73", 0xbababa),
    color_entry(c"Grey74", 0xbdbdbd),
    color_entry(c"Grey75", 0xbfbfbf),
    color_entry(c"Grey76", 0xc2c2c2),
    color_entry(c"Grey77", 0xc4c4c4),
    color_entry(c"Grey78", 0xc7c7c7),
    color_entry(c"Grey79", 0xc9c9c9),
    color_entry(c"Grey8", 0x141414),
    color_entry(c"Grey80", 0xcccccc),
    color_entry(c"Grey81", 0xcfcfcf),
    color_entry(c"Grey82", 0xd1d1d1),
    color_entry(c"Grey83", 0xd4d4d4),
    color_entry(c"Grey84", 0xd6d6d6),
    color_entry(c"Grey85", 0xd9d9d9),
    color_entry(c"Grey86", 0xdbdbdb),
    color_entry(c"Grey87", 0xdedede),
    color_entry(c"Grey88", 0xe0e0e0),
    color_entry(c"Grey89", 0xe3e3e3),
    color_entry(c"Grey9", 0x171717),
    color_entry(c"Grey90", 0xe5e5e5),
    color_entry(c"Grey91", 0xe8e8e8),
    color_entry(c"Grey92", 0xebebeb),
    color_entry(c"Grey93", 0xededed),
    color_entry(c"Grey94", 0xf0f0f0),
    color_entry(c"Grey95", 0xf2f2f2),
    color_entry(c"Grey96", 0xf5f5f5),
    color_entry(c"Grey97", 0xf7f7f7),
    color_entry(c"Grey98", 0xfafafa),
    color_entry(c"Grey99", 0xfcfcfc),
    color_entry(c"Honeydew", 0xf0fff0),
    color_entry(c"Honeydew1", 0xf0fff0),
    color_entry(c"Honeydew2", 0xe0eee0),
    color_entry(c"Honeydew3", 0xc1cdc1),
    color_entry(c"Honeydew4", 0x838b83),
    color_entry(c"HotPink", 0xff69b4),
    color_entry(c"HotPink1", 0xff6eb4),
    color_entry(c"HotPink2", 0xee6aa7),
    color_entry(c"HotPink3", 0xcd6090),
    color_entry(c"HotPink4", 0x8b3a62),
    color_entry(c"IndianRed", 0xcd5c5c),
    color_entry(c"IndianRed1", 0xff6a6a),
    color_entry(c"IndianRed2", 0xee6363),
    color_entry(c"IndianRed3", 0xcd5555),
    color_entry(c"IndianRed4", 0x8b3a3a),
    color_entry(c"Indigo", 0x4b0082),
    color_entry(c"Ivory", 0xfffff0),
    color_entry(c"Ivory1", 0xfffff0),
    color_entry(c"Ivory2", 0xeeeee0),
    color_entry(c"Ivory3", 0xcdcdc1),
    color_entry(c"Ivory4", 0x8b8b83),
    color_entry(c"Khaki", 0xf0e68c),
    color_entry(c"Khaki1", 0xfff68f),
    color_entry(c"Khaki2", 0xeee685),
    color_entry(c"Khaki3", 0xcdc673),
    color_entry(c"Khaki4", 0x8b864e),
    color_entry(c"Lavender", 0xe6e6fa),
    color_entry(c"LavenderBlush", 0xfff0f5),
    color_entry(c"LavenderBlush1", 0xfff0f5),
    color_entry(c"LavenderBlush2", 0xeee0e5),
    color_entry(c"LavenderBlush3", 0xcdc1c5),
    color_entry(c"LavenderBlush4", 0x8b8386),
    color_entry(c"LawnGreen", 0x7cfc00),
    color_entry(c"LemonChiffon", 0xfffacd),
    color_entry(c"LemonChiffon1", 0xfffacd),
    color_entry(c"LemonChiffon2", 0xeee9bf),
    color_entry(c"LemonChiffon3", 0xcdc9a5),
    color_entry(c"LemonChiffon4", 0x8b8970),
    color_entry(c"LightBlue", 0xadd8e6),
    color_entry(c"LightBlue1", 0xbfefff),
    color_entry(c"LightBlue2", 0xb2dfee),
    color_entry(c"LightBlue3", 0x9ac0cd),
    color_entry(c"LightBlue4", 0x68838b),
    color_entry(c"LightCoral", 0xf08080),
    color_entry(c"LightCyan", 0xe0ffff),
    color_entry(c"LightCyan1", 0xe0ffff),
    color_entry(c"LightCyan2", 0xd1eeee),
    color_entry(c"LightCyan3", 0xb4cdcd),
    color_entry(c"LightCyan4", 0x7a8b8b),
    color_entry(c"LightGoldenrod", 0xeedd82),
    color_entry(c"LightGoldenrod1", 0xffec8b),
    color_entry(c"LightGoldenrod2", 0xeedc82),
    color_entry(c"LightGoldenrod3", 0xcdbe70),
    color_entry(c"LightGoldenrod4", 0x8b814c),
    color_entry(c"LightGoldenrodYellow", 0xfafad2),
    color_entry(c"LightGray", 0xd3d3d3),
    color_entry(c"LightGreen", 0x90ee90),
    color_entry(c"LightGrey", 0xd3d3d3),
    color_entry(c"LightMagenta", 0xffbbff),
    color_entry(c"LightPink", 0xffb6c1),
    color_entry(c"LightPink1", 0xffaeb9),
    color_entry(c"LightPink2", 0xeea2ad),
    color_entry(c"LightPink3", 0xcd8c95),
    color_entry(c"LightPink4", 0x8b5f65),
    color_entry(c"LightRed", 0xffbbbb),
    color_entry(c"LightSalmon", 0xffa07a),
    color_entry(c"LightSalmon1", 0xffa07a),
    color_entry(c"LightSalmon2", 0xee9572),
    color_entry(c"LightSalmon3", 0xcd8162),
    color_entry(c"LightSalmon4", 0x8b5742),
    color_entry(c"LightSeaGreen", 0x20b2aa),
    color_entry(c"LightSkyBlue", 0x87cefa),
    color_entry(c"LightSkyBlue1", 0xb0e2ff),
    color_entry(c"LightSkyBlue2", 0xa4d3ee),
    color_entry(c"LightSkyBlue3", 0x8db6cd),
    color_entry(c"LightSkyBlue4", 0x607b8b),
    color_entry(c"LightSlateBlue", 0x8470ff),
    color_entry(c"LightSlateGray", 0x778899),
    color_entry(c"LightSlateGrey", 0x778899),
    color_entry(c"LightSteelBlue", 0xb0c4de),
    color_entry(c"LightSteelBlue1", 0xcae1ff),
    color_entry(c"LightSteelBlue2", 0xbcd2ee),
    color_entry(c"LightSteelBlue3", 0xa2b5cd),
    color_entry(c"LightSteelBlue4", 0x6e7b8b),
    color_entry(c"LightYellow", 0xffffe0),
    color_entry(c"LightYellow1", 0xffffe0),
    color_entry(c"LightYellow2", 0xeeeed1),
    color_entry(c"LightYellow3", 0xcdcdb4),
    color_entry(c"LightYellow4", 0x8b8b7a),
    color_entry(c"Lime", 0x00ff00),
    color_entry(c"LimeGreen", 0x32cd32),
    color_entry(c"Linen", 0xfaf0e6),
    color_entry(c"Magenta", 0xff00ff),
    color_entry(c"Magenta1", 0xff00ff),
    color_entry(c"Magenta2", 0xee00ee),
    color_entry(c"Magenta3", 0xcd00cd),
    color_entry(c"Magenta4", 0x8b008b),
    color_entry(c"Maroon", 0x800000),
    color_entry(c"Maroon1", 0xff34b3),
    color_entry(c"Maroon2", 0xee30a7),
    color_entry(c"Maroon3", 0xcd2990),
    color_entry(c"Maroon4", 0x8b1c62),
    color_entry(c"MediumAquamarine", 0x66cdaa),
    color_entry(c"MediumBlue", 0x0000cd),
    color_entry(c"MediumOrchid", 0xba55d3),
    color_entry(c"MediumOrchid1", 0xe066ff),
    color_entry(c"MediumOrchid2", 0xd15fee),
    color_entry(c"MediumOrchid3", 0xb452cd),
    color_entry(c"MediumOrchid4", 0x7a378b),
    color_entry(c"MediumPurple", 0x9370db),
    color_entry(c"MediumPurple1", 0xab82ff),
    color_entry(c"MediumPurple2", 0x9f79ee),
    color_entry(c"MediumPurple3", 0x8968cd),
    color_entry(c"MediumPurple4", 0x5d478b),
    color_entry(c"MediumSeaGreen", 0x3cb371),
    color_entry(c"MediumSlateBlue", 0x7b68ee),
    color_entry(c"MediumSpringGreen", 0x00fa9a),
    color_entry(c"MediumTurquoise", 0x48d1cc),
    color_entry(c"MediumVioletRed", 0xc71585),
    color_entry(c"MidnightBlue", 0x191970),
    color_entry(c"MintCream", 0xf5fffa),
    color_entry(c"MistyRose", 0xffe4e1),
    color_entry(c"MistyRose1", 0xffe4e1),
    color_entry(c"MistyRose2", 0xeed5d2),
    color_entry(c"MistyRose3", 0xcdb7b5),
    color_entry(c"MistyRose4", 0x8b7d7b),
    color_entry(c"Moccasin", 0xffe4b5),
    color_entry(c"NavajoWhite", 0xffdead),
    color_entry(c"NavajoWhite1", 0xffdead),
    color_entry(c"NavajoWhite2", 0xeecfa1),
    color_entry(c"NavajoWhite3", 0xcdb38b),
    color_entry(c"NavajoWhite4", 0x8b795e),
    color_entry(c"Navy", 0x000080),
    color_entry(c"NavyBlue", 0x000080),
    color_entry(c"NvimDarkBlue", 0x004c73),
    color_entry(c"NvimDarkCyan", 0x007373),
    color_entry(c"NvimDarkGray1", 0x07080d),
    color_entry(c"NvimDarkGray2", 0x14161b),
    color_entry(c"NvimDarkGray3", 0x2c2e33),
    color_entry(c"NvimDarkGray4", 0x4f5258),
    color_entry(c"NvimDarkGreen", 0x005523),
    color_entry(c"NvimDarkGrey1", 0x07080d),
    color_entry(c"NvimDarkGrey2", 0x14161b),
    color_entry(c"NvimDarkGrey3", 0x2c2e33),
    color_entry(c"NvimDarkGrey4", 0x4f5258),
    color_entry(c"NvimDarkMagenta", 0x470045),
    color_entry(c"NvimDarkRed", 0x590008),
    color_entry(c"NvimDarkYellow", 0x6b5300),
    color_entry(c"NvimLightBlue", 0xa6dbff),
    color_entry(c"NvimLightCyan", 0x8cf8f7),
    color_entry(c"NvimLightGray1", 0xeef1f8),
    color_entry(c"NvimLightGray2", 0xe0e2ea),
    color_entry(c"NvimLightGray3", 0xc4c6cd),
    color_entry(c"NvimLightGray4", 0x9b9ea4),
    color_entry(c"NvimLightGreen", 0xb3f6c0),
    color_entry(c"NvimLightGrey1", 0xeef1f8),
    color_entry(c"NvimLightGrey2", 0xe0e2ea),
    color_entry(c"NvimLightGrey3", 0xc4c6cd),
    color_entry(c"NvimLightGrey4", 0x9b9ea4),
    color_entry(c"NvimLightMagenta", 0xffcaff),
    color_entry(c"NvimLightRed", 0xffc0b9),
    color_entry(c"NvimLightYellow", 0xfce094),
    color_entry(c"OldLace", 0xfdf5e6),
    color_entry(c"Olive", 0x808000),
    color_entry(c"OliveDrab", 0x6b8e23),
    color_entry(c"OliveDrab1", 0xc0ff3e),
    color_entry(c"OliveDrab2", 0xb3ee3a),
    color_entry(c"OliveDrab3", 0x9acd32),
    color_entry(c"OliveDrab4", 0x698b22),
    color_entry(c"Orange", 0xffa500),
    color_entry(c"Orange1", 0xffa500),
    color_entry(c"Orange2", 0xee9a00),
    color_entry(c"Orange3", 0xcd8500),
    color_entry(c"Orange4", 0x8b5a00),
    color_entry(c"OrangeRed", 0xff4500),
    color_entry(c"OrangeRed1", 0xff4500),
    color_entry(c"OrangeRed2", 0xee4000),
    color_entry(c"OrangeRed3", 0xcd3700),
    color_entry(c"OrangeRed4", 0x8b2500),
    color_entry(c"Orchid", 0xda70d6),
    color_entry(c"Orchid1", 0xff83fa),
    color_entry(c"Orchid2", 0xee7ae9),
    color_entry(c"Orchid3", 0xcd69c9),
    color_entry(c"Orchid4", 0x8b4789),
    color_entry(c"PaleGoldenrod", 0xeee8aa),
    color_entry(c"PaleGreen", 0x98fb98),
    color_entry(c"PaleGreen1", 0x9aff9a),
    color_entry(c"PaleGreen2", 0x90ee90),
    color_entry(c"PaleGreen3", 0x7ccd7c),
    color_entry(c"PaleGreen4", 0x548b54),
    color_entry(c"PaleTurquoise", 0xafeeee),
    color_entry(c"PaleTurquoise1", 0xbbffff),
    color_entry(c"PaleTurquoise2", 0xaeeeee),
    color_entry(c"PaleTurquoise3", 0x96cdcd),
    color_entry(c"PaleTurquoise4", 0x668b8b),
    color_entry(c"PaleVioletRed", 0xdb7093),
    color_entry(c"PaleVioletRed1", 0xff82ab),
    color_entry(c"PaleVioletRed2", 0xee799f),
    color_entry(c"PaleVioletRed3", 0xcd6889),
    color_entry(c"PaleVioletRed4", 0x8b475d),
    color_entry(c"PapayaWhip", 0xffefd5),
    color_entry(c"PeachPuff", 0xffdab9),
    color_entry(c"PeachPuff1", 0xffdab9),
    color_entry(c"PeachPuff2", 0xeecbad),
    color_entry(c"PeachPuff3", 0xcdaf95),
    color_entry(c"PeachPuff4", 0x8b7765),
    color_entry(c"Peru", 0xcd853f),
    color_entry(c"Pink", 0xffc0cb),
    color_entry(c"Pink1", 0xffb5c5),
    color_entry(c"Pink2", 0xeea9b8),
    color_entry(c"Pink3", 0xcd919e),
    color_entry(c"Pink4", 0x8b636c),
    color_entry(c"Plum", 0xdda0dd),
    color_entry(c"Plum1", 0xffbbff),
    color_entry(c"Plum2", 0xeeaeee),
    color_entry(c"Plum3", 0xcd96cd),
    color_entry(c"Plum4", 0x8b668b),
    color_entry(c"PowderBlue", 0xb0e0e6),
    color_entry(c"Purple", 0x800080),
    color_entry(c"Purple1", 0x9b30ff),
    color_entry(c"Purple2", 0x912cee),
    color_entry(c"Purple3", 0x7d26cd),
    color_entry(c"Purple4", 0x551a8b),
    color_entry(c"RebeccaPurple", 0x663399),
    color_entry(c"Red", 0xff0000),
    color_entry(c"Red1", 0xff0000),
    color_entry(c"Red2", 0xee0000),
    color_entry(c"Red3", 0xcd0000),
    color_entry(c"Red4", 0x8b0000),
    color_entry(c"RosyBrown", 0xbc8f8f),
    color_entry(c"RosyBrown1", 0xffc1c1),
    color_entry(c"RosyBrown2", 0xeeb4b4),
    color_entry(c"RosyBrown3", 0xcd9b9b),
    color_entry(c"RosyBrown4", 0x8b6969),
    color_entry(c"RoyalBlue", 0x4169e1),
    color_entry(c"RoyalBlue1", 0x4876ff),
    color_entry(c"RoyalBlue2", 0x436eee),
    color_entry(c"RoyalBlue3", 0x3a5fcd),
    color_entry(c"RoyalBlue4", 0x27408b),
    color_entry(c"SaddleBrown", 0x8b4513),
    color_entry(c"Salmon", 0xfa8072),
    color_entry(c"Salmon1", 0xff8c69),
    color_entry(c"Salmon2", 0xee8262),
    color_entry(c"Salmon3", 0xcd7054),
    color_entry(c"Salmon4", 0x8b4c39),
    color_entry(c"SandyBrown", 0xf4a460),
    color_entry(c"SeaGreen", 0x2e8b57),
    color_entry(c"SeaGreen1", 0x54ff9f),
    color_entry(c"SeaGreen2", 0x4eee94),
    color_entry(c"SeaGreen3", 0x43cd80),
    color_entry(c"SeaGreen4", 0x2e8b57),
    color_entry(c"SeaShell", 0xfff5ee),
    color_entry(c"Seashell1", 0xfff5ee),
    color_entry(c"Seashell2", 0xeee5de),
    color_entry(c"Seashell3", 0xcdc5bf),
    color_entry(c"Seashell4", 0x8b8682),
    color_entry(c"Sienna", 0xa0522d),
    color_entry(c"Sienna1", 0xff8247),
    color_entry(c"Sienna2", 0xee7942),
    color_entry(c"Sienna3", 0xcd6839),
    color_entry(c"Sienna4", 0x8b4726),
    color_entry(c"Silver", 0xc0c0c0),
    color_entry(c"SkyBlue", 0x87ceeb),
    color_entry(c"SkyBlue1", 0x87ceff),
    color_entry(c"SkyBlue2", 0x7ec0ee),
    color_entry(c"SkyBlue3", 0x6ca6cd),
    color_entry(c"SkyBlue4", 0x4a708b),
    color_entry(c"SlateBlue", 0x6a5acd),
    color_entry(c"SlateBlue1", 0x836fff),
    color_entry(c"SlateBlue2", 0x7a67ee),
    color_entry(c"SlateBlue3", 0x6959cd),
    color_entry(c"SlateBlue4", 0x473c8b),
    color_entry(c"SlateGray", 0x708090),
    color_entry(c"SlateGray1", 0xc6e2ff),
    color_entry(c"SlateGray2", 0xb9d3ee),
    color_entry(c"SlateGray3", 0x9fb6cd),
    color_entry(c"SlateGray4", 0x6c7b8b),
    color_entry(c"SlateGrey", 0x708090),
    color_entry(c"Snow", 0xfffafa),
    color_entry(c"Snow1", 0xfffafa),
    color_entry(c"Snow2", 0xeee9e9),
    color_entry(c"Snow3", 0xcdc9c9),
    color_entry(c"Snow4", 0x8b8989),
    color_entry(c"SpringGreen", 0x00ff7f),
    color_entry(c"SpringGreen1", 0x00ff7f),
    color_entry(c"SpringGreen2", 0x00ee76),
    color_entry(c"SpringGreen3", 0x00cd66),
    color_entry(c"SpringGreen4", 0x008b45),
    color_entry(c"SteelBlue", 0x4682b4),
    color_entry(c"SteelBlue1", 0x63b8ff),
    color_entry(c"SteelBlue2", 0x5cacee),
    color_entry(c"SteelBlue3", 0x4f94cd),
    color_entry(c"SteelBlue4", 0x36648b),
    color_entry(c"Tan", 0xd2b48c),
    color_entry(c"Tan1", 0xffa54f),
    color_entry(c"Tan2", 0xee9a49),
    color_entry(c"Tan3", 0xcd853f),
    color_entry(c"Tan4", 0x8b5a2b),
    color_entry(c"Teal", 0x008080),
    color_entry(c"Thistle", 0xd8bfd8),
    color_entry(c"Thistle1", 0xffe1ff),
    color_entry(c"Thistle2", 0xeed2ee),
    color_entry(c"Thistle3", 0xcdb5cd),
    color_entry(c"Thistle4", 0x8b7b8b),
    color_entry(c"Tomato", 0xff6347),
    color_entry(c"Tomato1", 0xff6347),
    color_entry(c"Tomato2", 0xee5c42),
    color_entry(c"Tomato3", 0xcd4f39),
    color_entry(c"Tomato4", 0x8b3626),
    color_entry(c"Turquoise", 0x40e0d0),
    color_entry(c"Turquoise1", 0x00f5ff),
    color_entry(c"Turquoise2", 0x00e5ee),
    color_entry(c"Turquoise3", 0x00c5cd),
    color_entry(c"Turquoise4", 0x00868b),
    color_entry(c"Violet", 0xee82ee),
    color_entry(c"VioletRed", 0xd02090),
    color_entry(c"VioletRed1", 0xff3e96),
    color_entry(c"VioletRed2", 0xee3a8c),
    color_entry(c"VioletRed3", 0xcd3278),
    color_entry(c"VioletRed4", 0x8b2252),
    color_entry(c"WebGray", 0x808080),
    color_entry(c"WebGreen", 0x008000),
    color_entry(c"WebGrey", 0x808080),
    color_entry(c"WebMaroon", 0x800000),
    color_entry(c"WebPurple", 0x800080),
    color_entry(c"Wheat", 0xf5deb3),
    color_entry(c"Wheat1", 0xffe7ba),
    color_entry(c"Wheat2", 0xeed8ae),
    color_entry(c"Wheat3", 0xcdba96),
    color_entry(c"Wheat4", 0x8b7e66),
    color_entry(c"White", 0xffffff),
    color_entry(c"WhiteSmoke", 0xf5f5f5),
    color_entry(c"X11Gray", 0xbebebe),
    color_entry(c"X11Green", 0x00ff00),
    color_entry(c"X11Grey", 0xbebebe),
    color_entry(c"X11Maroon", 0xb03060),
    color_entry(c"X11Purple", 0xa020f0),
    color_entry(c"Yellow", 0xffff00),
    color_entry(c"Yellow1", 0xffff00),
    color_entry(c"Yellow2", 0xeeee00),
    color_entry(c"Yellow3", 0xcdcd00),
    color_entry(c"Yellow4", 0x8b8b00),
    color_entry(c"YellowGreen", 0x9acd32),
    color_name_table_T {
        name: ::core::ptr::null_mut(),
        color: 0,
    },
]);
pub unsafe extern "C" fn name_to_color(
    mut name: *const ::core::ffi::c_char,
    mut idx: *mut ::core::ffi::c_int,
) -> RgbValue {
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '#' as ::core::ffi::c_int
        && *(*__ctype_b_loc()).offset(*name.offset(1 as ::core::ffi::c_int as isize) as uint8_t
            as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        && *(*__ctype_b_loc()).offset(*name.offset(2 as ::core::ffi::c_int as isize) as uint8_t
            as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        && *(*__ctype_b_loc()).offset(*name.offset(3 as ::core::ffi::c_int as isize) as uint8_t
            as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        && *(*__ctype_b_loc()).offset(*name.offset(4 as ::core::ffi::c_int as isize) as uint8_t
            as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        && *(*__ctype_b_loc()).offset(*name.offset(5 as ::core::ffi::c_int as isize) as uint8_t
            as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        && *(*__ctype_b_loc()).offset(*name.offset(6 as ::core::ffi::c_int as isize) as uint8_t
            as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        && *name.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        *idx = kColorIdxHex as ::core::ffi::c_int;
        return strtol(
            name.offset(1 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            16 as ::core::ffi::c_int,
        ) as RgbValue;
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"bg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0
        || strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"background\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0
    {
        *idx = kColorIdxBg as ::core::ffi::c_int;
        return normal_bg.get();
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"fg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0
        || strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"foreground\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0
    {
        *idx = kColorIdxFg as ::core::ffi::c_int;
        return normal_fg.get();
    }
    let mut lo: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut hi: ::core::ffi::c_int = ::core::mem::size_of::<[color_name_table_T; 708]>()
        .wrapping_div(::core::mem::size_of::<color_name_table_T>())
        .wrapping_div(
            (::core::mem::size_of::<[color_name_table_T; 708]>()
                .wrapping_rem(::core::mem::size_of::<color_name_table_T>())
                == 0) as ::core::ffi::c_int as usize,
        )
        .wrapping_sub(1 as usize) as ::core::ffi::c_int;
    while lo < hi {
        let mut m: ::core::ffi::c_int = (lo + hi) / 2 as ::core::ffi::c_int;
        let mut cmp: ::core::ffi::c_int = strcasecmp(
            name as *mut ::core::ffi::c_char,
            (*color_name_table.ptr())[m as usize].name,
        );
        if cmp < 0 as ::core::ffi::c_int {
            hi = m;
        } else if cmp > 0 as ::core::ffi::c_int {
            lo = m + 1 as ::core::ffi::c_int;
        } else {
            *idx = m;
            return (*color_name_table.ptr())[m as usize].color;
        }
    }
    *idx = kColorIdxNone as ::core::ffi::c_int;
    return -1 as RgbValue;
}
pub unsafe extern "C" fn coloridx_to_name(
    mut idx: ::core::ffi::c_int,
    mut val: ::core::ffi::c_int,
    mut hexbuf: *mut ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    if idx >= 0 as ::core::ffi::c_int {
        return (*color_name_table.ptr())[idx as usize].name;
    }
    match idx {
        -1 => return ::core::ptr::null::<::core::ffi::c_char>(),
        -3 => return b"fg\0".as_ptr() as *const ::core::ffi::c_char,
        -4 => return b"bg\0".as_ptr() as *const ::core::ffi::c_char,
        -2 => {
            snprintf(
                hexbuf as *mut ::core::ffi::c_char,
                (7 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t,
                b"#%06x\0".as_ptr() as *const ::core::ffi::c_char,
                val,
            );
            return hexbuf as *const ::core::ffi::c_char;
        }
        _ => {
            abort();
        }
    };
}
pub unsafe extern "C" fn name_to_ctermcolor(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut off: ::core::ffi::c_int = if (*name as ::core::ffi::c_int) < 'a' as ::core::ffi::c_int
        || *name as ::core::ffi::c_int > 'z' as ::core::ffi::c_int
    {
        *name as ::core::ffi::c_int
    } else {
        *name as ::core::ffi::c_int - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
    };
    i = ::core::mem::size_of::<[*mut ::core::ffi::c_char; 28]>()
        .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
        .wrapping_div(
            (::core::mem::size_of::<[*mut ::core::ffi::c_char; 28]>()
                .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                == 0) as ::core::ffi::c_int as usize,
        ) as ::core::ffi::c_int;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        if off
            == *(*color_names.ptr())[i as usize].offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
            && strcasecmp(
                name.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char,
                (*color_names.ptr())[i as usize].offset(1 as ::core::ffi::c_int as isize),
            ) == 0 as ::core::ffi::c_int
        {
            break;
        }
    }
    if i < 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    let mut bold: TriState = kNone;
    return lookup_color(i, false_0 != 0, &raw mut bold);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
