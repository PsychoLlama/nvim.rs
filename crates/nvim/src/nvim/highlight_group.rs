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
mod hlf;

pub use self::hlf::*;

// The carve of the transpiled module; see each child's docs.
mod colornames;
pub use self::colornames::*;
mod cterm;
pub use self::cterm::*;
mod defaults;
pub use self::defaults::*;
mod table;
pub use self::table::*;
mod list;
pub use self::list::*;
mod query;
pub use self::query::*;
mod command;
pub use self::command::*;
mod apply;
pub use self::apply::*;
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
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const MAX_SYN_NAME: ::core::ffi::c_int = 200 as ::core::ffi::c_int;
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
    (*highlight_attr.ptr())[HLF_NONE as usize] = 0 as ::core::ffi::c_int;
    let mut hlf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while hlf < HLF_COUNT {
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
        if hlf == HLF_SNC {
            id_SNC = final_id;
        } else if hlf == HLF_S {
            id_S = final_id;
        }
        (*highlight_attr.ptr())[hlf as usize] =
            hl_get_ui_attr(ns_id, hlf, final_id, hlf == HLF_INACTIVE);
        if (*highlight_attr.ptr())[hlf as usize] != (*highlight_attr_last.ptr())[hlf as usize] {
            if hlf == HLF_MSG {
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
                HLF_SNC,
                highlight_stlnc.ptr() as *mut ::core::ffi::c_int,
            );
        }
        i += 1;
    }
    (*highlight_ga.ptr()).ga_len = hlcnt;
    decor_provider_invalidate_hl();
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
