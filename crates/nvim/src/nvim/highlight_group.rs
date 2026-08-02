#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_int};

use crate::src::nvim::api::private::helpers::{api_set_error, arena_dict, cstr_as_string};
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::charset::{skiptowhite, skipwhite, vim_strsize};
use crate::src::nvim::cursor_shape::cursor_mode_uses_syn_id;
use crate::src::nvim::decoration_provider::decor_provider_invalidate_hl;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, UPD_SOME_VALID, redraw_all_later};
use crate::src::nvim::eval::last_set_msg;
use crate::src::nvim::eval::vars::do_unlet;
use crate::src::nvim::ex_docmd::ends_excmd;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight::{
    HL_ALTFONT, HL_BLINK, HL_BOLD, HL_CONCEALED, HL_DEFAULT, HL_DIM, HL_INVERSE, HL_ITALIC,
    HL_NOCOMBINE, HL_OVERLINE, HL_STANDOUT, HL_STRIKETHROUGH, HL_UNDERCURL, HL_UNDERDASHED,
    HL_UNDERDOTTED, HL_UNDERDOUBLE, HL_UNDERLINE, HL_UNDERLINE_MASK, hl_get_syn_attr,
    hl_get_ui_attr, hlattrs2dict, ns_get_hl, syn_attr2entry,
};
use crate::src::nvim::lua::executor::nlua_set_sctx;
use crate::src::nvim::main::{
    Columns, clear_cmdline, cterm_normal_bg_color, cterm_normal_fg_color, current_sctx, e_invarg2,
    got_int, highlight_attr, highlight_attr_last, highlight_stlnc, highlight_user, include_default,
    include_link, include_none, msg_col, msg_grid, msg_silent, need_highlight_changed, normal_bg,
    normal_fg, normal_sp, p_bg, p_verbose, starting, t_colors, updating_screen,
};
use crate::src::nvim::memory::xstrlcat;
use crate::src::nvim::message::{
    emsg, message_filtered, msg_advance, msg_clr_eos, msg_ext_set_kind, msg_outtrans, msg_putchar,
    msg_puts_hl, semsg,
};
use crate::src::nvim::option::{option_was_set, reset_option_was_set, set_option_value_give_err};
use crate::src::nvim::options::kOptBackground;
use crate::src::nvim::os::libc::{
    __assert_fail, atoi, gettext, memcmp, memcpy, snprintf, strcasecmp, strchr, strcmp, strlen,
    strncasecmp, strncmp, strtol,
};
use crate::src::nvim::os::time::os_delay;
use crate::src::nvim::runtime::exestack;
use crate::src::nvim::strings::vim_memcpy_up;
use crate::src::nvim::types::api::{kErrorTypeNone, kErrorTypeValidation};
use crate::src::nvim::types::ui::{kUILinegrid, kUIMessages};
use crate::src::nvim::types::{
    Arena, Boolean, Dict, Error, HlAttrs, Integer, KeyDict_get_highlight, KeyDict_highlight,
    KeyValuePair, NS, Object, OptInt, OptVal, OptValData, OptValType, RgbValue, TriState, estack_T,
    expand_T, int32_t, kObjectTypeBoolean, kObjectTypeDict, kObjectTypeNil, kObjectTypeString,
    key_value_pair, object, object_data as C2Rust_Unnamed_0, size_t, uint8_t, uint64_t,
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
/// Applies the difference between `User{i+1}` and `StatusLine` to
/// `StatusLineNC`, in scratch entry `hlcnt + i`, and answers the attribute
/// id that combination resolves to.
///
/// `id` is the `User` group, `id_s` `StatusLine`, `id_alt` `StatusLineNC`.
/// The scratch entry has to be a real group because `syn_id2attr` is asked
/// for it — see [`highlight_changed`].
///
/// # Safety
/// Reaches the attribute table; main thread only.
unsafe fn combine_stl_hlt(
    id: c_int,
    id_s: c_int,
    id_alt: c_int,
    hlcnt: c_int,
    i: c_int,
    hlf: c_int,
) -> c_int {
    // SAFETY (whole body): the editor's own tables, on the main thread.
    unsafe {
        let scratch = hlcnt + i + 1;
        let mut combined = if id_alt == 0 {
            // No `StatusLineNC` of its own: start from the resolved attribute.
            let attr = highlight_attr.with(|attrs| attrs[hlf as usize]);
            HlGroup {
                cterm: attr,
                gui: attr,
                ..HlGroup::ZEROED
            }
        } else {
            group(id_alt)
        };
        let user = group(id);
        let stl = group(id_s);

        combined.link = 0;
        combined.cterm ^= user.cterm ^ stl.cterm;
        if user.cterm_fg != stl.cterm_fg {
            combined.cterm_fg = user.cterm_fg;
        }
        if user.cterm_bg != stl.cterm_bg {
            combined.cterm_bg = user.cterm_bg;
        }
        combined.gui ^= user.gui ^ stl.gui;
        if user.rgb_fg != stl.rgb_fg {
            combined.rgb_fg = user.rgb_fg;
        }
        if user.rgb_bg != stl.rgb_bg {
            combined.rgb_bg = user.rgb_bg;
        }
        if user.rgb_sp != stl.rgb_sp {
            combined.rgb_sp = user.rgb_sp;
        }

        with_group(scratch, |entry| *entry = combined);
        set_hl_attr(scratch);
        syn_id2attr(scratch)
    }
}

/// Resolves every builtin group into `highlight_attr[]`, and sets up
/// `User1`..`User9`.
///
/// Called when nvim starts and on the first redraw after any `:highlight`.
///
/// # Safety
/// Adds groups, resolves attributes and emits UI events; main thread only.
pub unsafe fn highlight_changed() {
    // SAFETY (whole body): the editor's own tables and UI, on the main
    // thread; `hlf_names` is static and NUL-terminated.
    unsafe {
        need_highlight_changed.set(false);

        // Sentinel: used when no highlight is active.
        highlight_attr.with_mut(|attrs| attrs[HLF_NONE as usize] = 0);

        let mut id_s = -1;
        let mut id_snc = 0;
        for hlf in 1..HLF_COUNT {
            let name = hlf_names.with(|names| names[hlf as usize]);
            let id = syn_check_group(name, CStr::from_ptr(name).count_bytes() as size_t);
            assert!(id != 0, "builtin highlight group {hlf} could not be added");

            let mut ns_id = -1;
            let mut final_id = id;
            syn_ns_get_final_id(&mut ns_id, &mut final_id);
            if hlf == HLF_SNC {
                id_snc = final_id;
            } else if hlf == HLF_S {
                id_s = final_id;
            }

            let attr = hl_get_ui_attr(ns_id, hlf, final_id, hlf == HLF_INACTIVE);
            highlight_attr.with_mut(|attrs| attrs[hlf as usize] = attr);
            if attr == highlight_attr_last.with(|last| last[hlf as usize]) {
                continue;
            }
            if hlf == HLF_MSG {
                clear_cmdline.set(true);
                (*msg_grid.ptr()).blending = syn_attr2entry(attr).hl_blend > -1;
            }
            ui_call_hl_group_set(cstr_as_string(name), Integer::from(attr));
            highlight_attr_last.with_mut(|last| last[hlf as usize] = attr);
        }

        // Ten scratch entries, live at once in case the attribute table
        // overflows while they are being built: nine for User1-User9 combined
        // with StatusLineNC, one for the StatusLine default.
        let hlcnt = highlight_num_groups();
        open_scratch(hlcnt, 10);
        if id_s == -1 {
            // Make id_s always valid, using the last (all-zero) scratch entry.
            id_s = hlcnt + 10;
        }
        for i in 0..9 {
            let userhl = format!("User{}\0", i + 1);
            let id = syn_name2id(userhl.as_ptr().cast());
            let (user, stlnc) = if id == 0 {
                (0, 0)
            } else {
                (
                    syn_id2attr(id),
                    combine_stl_hlt(id, id_s, id_snc, hlcnt, i, HLF_SNC),
                )
            };
            highlight_user.with_mut(|table| table[i as usize] = user);
            highlight_stlnc.with_mut(|table| table[i as usize] = stlnc);
        }
        close_scratch(hlcnt);

        decor_provider_invalidate_hl();
    }
}

pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
