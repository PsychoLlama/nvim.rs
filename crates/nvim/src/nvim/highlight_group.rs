#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_int, c_uint};

use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::decoration_provider::decor_provider_invalidate_hl;
use crate::src::nvim::highlight::{
    HL_ALTFONT, HL_BLINK, HL_BOLD, HL_CONCEALED, HL_DIM, HL_INVERSE, HL_ITALIC, HL_NOCOMBINE,
    HL_OVERLINE, HL_STANDOUT, HL_STRIKETHROUGH, HL_UNDERCURL, HL_UNDERDASHED, HL_UNDERDOTTED,
    HL_UNDERDOUBLE, HL_UNDERLINE, HL_UNDERLINE_MASK, hl_get_ui_attr, syn_attr2entry,
};
use crate::src::nvim::main::{
    clear_cmdline, highlight_attr, highlight_attr_last, highlight_stlnc, highlight_user, msg_grid,
    need_highlight_changed,
};
use crate::src::nvim::types::{Integer, OptValType, TriState, size_t};
use crate::src::nvim::ui::ui_call_hl_group_set;
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

/// `TriState`: an answer that can also be "leave it alone".
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;

/// The most entries `hlattrs2dict` can write, which is what its callers have
/// to allocate.
pub const HLATTRS_DICT_SIZE: size_t = 24;

/// `xp_context` values: what `:highlight`'s completion is expanding.
pub const EXPAND_HIGHLIGHT: c_int = 13;
pub const EXPAND_NOTHING: c_int = 0;

/// `OptValType`, for the one option this family sets (`'background'`).
pub const kOptValTypeString: OptValType = 2;

/// `source_runtime_vim_lua` flags: where a colour scheme may be found.
pub const DIP_START: c_uint = 8;
pub const DIP_OPT: c_uint = 16;

/// The highest highlight id, and the longest group name.
pub const MAX_HL_ID: c_uint = 20000;
pub const MAX_SYN_NAME: c_int = 200;

/// `HlGroup::set`: which parts of a group have been given explicitly, so
/// that `init` definitions know what not to overwrite.
pub const SG_CTERM: c_uint = 2;
pub const SG_GUI: c_uint = 4;
pub const SG_LINK: c_uint = 8;

/// `HlGroup::rgb_*_idx`: where an RGB colour came from. A non-negative value
/// is an index into `COLOR_NAMES` instead.
pub const kColorIdxNone: c_int = -1;
pub const kColorIdxHex: c_int = -2;
pub const kColorIdxFg: c_int = -3;
pub const kColorIdxBg: c_int = -4;

/// Keyset bit positions, from `api/keysets_defs.h`.
pub const KEYSET_OPTIDX_highlight__bg: c_int = 1;
pub const KEYSET_OPTIDX_highlight__fg: c_int = 2;
pub const KEYSET_OPTIDX_highlight__sp: c_int = 3;
pub const KEYSET_OPTIDX_highlight__update: c_int = 13;
pub const KEYSET_OPTIDX_get_highlight__id: c_int = 1;
pub const KEYSET_OPTIDX_get_highlight__link: c_int = 2;
pub const KEYSET_OPTIDX_get_highlight__name: c_int = 3;
pub const KEYSET_OPTIDX_get_highlight__create: c_int = 4;

/// `OK`, the answer `:colorscheme` reads from `load_colors`.
pub const OK: c_int = 1;

/// The `:highlight` parse errors.
pub(crate) const e_highlight_group_name_not_found_str: &CStr =
    c"E411: Highlight group not found: %s";
pub(crate) const e_group_has_settings_highlight_link_ignored: &CStr =
    c"E414: Group has settings, highlight link ignored";
pub(crate) const e_unexpected_equal_sign_str: &CStr = c"E415: Unexpected equal sign: %s";
pub(crate) const e_missing_equal_sign_str_2: &CStr = c"E416: Missing equal sign: %s";
pub(crate) const e_missing_argument_str: &CStr = c"E417: Missing argument: %s";

/// The names `term=`/`cterm=`/`gui=` accept, and the `HL_*` bit each one
/// means. `reverse` and `inverse` are the same bit; the `NONE` sentinel ends
/// the list, and is what makes an unrecognised name an error.
pub(crate) static ATTR_NAMES: [(&CStr, c_int); 18] = [
    (c"bold", HL_BOLD),
    (c"standout", HL_STANDOUT),
    (c"underline", HL_UNDERLINE),
    (c"undercurl", HL_UNDERCURL),
    (c"underdouble", HL_UNDERDOUBLE),
    (c"underdotted", HL_UNDERDOTTED),
    (c"underdashed", HL_UNDERDASHED),
    (c"italic", HL_ITALIC),
    (c"reverse", HL_INVERSE),
    (c"inverse", HL_INVERSE),
    (c"strikethrough", HL_STRIKETHROUGH),
    (c"altfont", HL_ALTFONT),
    (c"dim", HL_DIM),
    (c"blink", HL_BLINK),
    (c"conceal", HL_CONCEALED),
    (c"overline", HL_OVERLINE),
    (c"nocombine", HL_NOCOMBINE),
    (c"NONE", 0),
];
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
