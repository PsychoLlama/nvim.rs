#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use core::ffi::{CStr, c_int, c_uint};

use crate::api::private::helpers::cstr_to_string;
use crate::decoration_provider::decor_provider_invalidate_hl;
use crate::drawscreen::state::clear_cmdline;
use crate::highlight::state::{
    highlight_attr, highlight_attr_last, highlight_stlnc, highlight_user, need_highlight_changed,
};
use crate::highlight::{HlAttrFlags, hl_get_ui_attr, syn_attr2entry};
use crate::message::msg_grid_ref;
use crate::types::Integer;
use crate::ui::ui_call_hl_group_set;
mod hlf;

pub(crate) use self::hlf::*;

// The carve of the transpiled module; see each child's docs.
mod colornames;
pub(crate) use self::colornames::*;
mod cterm;
pub(crate) use self::cterm::*;
mod defaults;
pub(crate) use self::defaults::*;
mod table;
pub(crate) use self::table::*;
mod list;
pub(crate) use self::list::*;
mod query;
pub(crate) use self::query::*;
mod command;
pub(crate) use self::command::*;
mod apply;
pub(crate) use self::apply::*;

/// The highest highlight id, and the longest group name.
pub(crate) const MAX_HL_ID: c_uint = 20000;
pub(crate) const MAX_SYN_NAME: c_int = 200;

/// `HlGroup::set`: which parts of a group have been given explicitly, so
/// that `init` definitions know what not to overwrite.
pub(crate) const SG_CTERM: c_uint = 2;
pub(crate) const SG_GUI: c_uint = 4;
pub(crate) const SG_LINK: c_uint = 8;

/// `HlGroup::rgb_*_idx`: where an RGB colour came from. A non-negative value
/// is an index into `COLOR_NAMES` instead.
pub(crate) const kColorIdxNone: c_int = -1;
pub(crate) const kColorIdxHex: c_int = -2;
pub(crate) const kColorIdxFg: c_int = -3;
pub(crate) const kColorIdxBg: c_int = -4;

pub(crate) const e_group_has_settings_highlight_link_ignored: &CStr =
    c"E414: Group has settings, highlight link ignored";

/// The names `term=`/`cterm=`/`gui=` accept, and the attribute bit each one
/// means. `reverse` and `inverse` are the same bit; the `NONE` sentinel ends
/// the list, and is what makes an unrecognised name an error.
pub(crate) static ATTR_NAMES: [(&CStr, HlAttrFlags); 18] = [
    (c"bold", HlAttrFlags::BOLD),
    (c"standout", HlAttrFlags::STANDOUT),
    (c"underline", HlAttrFlags::UNDERLINE),
    (c"undercurl", HlAttrFlags::UNDERCURL),
    (c"underdouble", HlAttrFlags::UNDERDOUBLE),
    (c"underdotted", HlAttrFlags::UNDERDOTTED),
    (c"underdashed", HlAttrFlags::UNDERDASHED),
    (c"italic", HlAttrFlags::ITALIC),
    (c"reverse", HlAttrFlags::INVERSE),
    (c"inverse", HlAttrFlags::INVERSE),
    (c"strikethrough", HlAttrFlags::STRIKETHROUGH),
    (c"altfont", HlAttrFlags::ALTFONT),
    (c"dim", HlAttrFlags::DIM),
    (c"blink", HlAttrFlags::BLINK),
    (c"conceal", HlAttrFlags::CONCEALED),
    (c"overline", HlAttrFlags::OVERLINE),
    (c"nocombine", HlAttrFlags::NOCOMBINE),
    (c"NONE", HlAttrFlags::NONE),
];
/// Applies the difference between `User{i+1}` and `StatusLine` to
/// `StatusLineNC`, in scratch entry `hlcnt + i`, and answers the attribute
/// id that combination resolves to.
///
/// `id` is the `User` group, `id_s` `StatusLine`, `id_alt` `StatusLineNC`.
/// The scratch entry has to be a real group because `syn_id2attr` is asked
/// for it — see [`highlight_changed`].
fn combine_stl_hlt(
    id: c_int,
    id_s: c_int,
    id_alt: c_int,
    hlcnt: c_int,
    i: c_int,
    hlf: c_int,
) -> c_int {
    // SAFETY (whole body): the editor's own tables, on the main thread.
    let scratch = hlcnt + i + 1;
    let mut combined = if id_alt == 0 {
        // No `StatusLineNC` of its own: start from the resolved attribute.
        // Upstream puts the resolved attribute *id* into the two
        // attribute-*bit* fields here (`sg_cterm`/`sg_gui`), which is not
        // a set of attribute bits at all. The arm is unreachable --
        // `id_alt` is `HLF_SNC`'s final id, and a builtin group always
        // has one -- so this is carried as written rather than fixed.
        let attr = HlAttrFlags::from_bits(highlight_attr.with(|attrs| attrs[hlf as usize]));
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
    // "the difference between User{i} and StatusLine, applied to
    // StatusLineNC" -- an exclusive-or over the whole word, which is not
    // a set operation, so it goes through the bits.
    combined.cterm =
        HlAttrFlags::from_bits(combined.cterm.bits() ^ user.cterm.bits() ^ stl.cterm.bits());
    if user.cterm_fg != stl.cterm_fg {
        combined.cterm_fg = user.cterm_fg;
    }
    if user.cterm_bg != stl.cterm_bg {
        combined.cterm_bg = user.cterm_bg;
    }
    combined.gui = HlAttrFlags::from_bits(combined.gui.bits() ^ user.gui.bits() ^ stl.gui.bits());
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

/// Resolves every builtin group into `highlight_attr[]`, and sets up
/// `User1`..`User9`.
///
/// Called when nvim starts and on the first redraw after any `:highlight`.
pub(crate) fn highlight_changed() {
    // `HLF_MSG`'s blend flag lives on the message grid; acquired once.
    let mut msg_grid = msg_grid_ref();
    // SAFETY (whole body): the editor's own tables and UI, on the main
    // thread; `hlf_names` is static and NUL-terminated.
    need_highlight_changed.set(false);

    // Sentinel: used when no highlight is active.
    highlight_attr.with_mut(|attrs| attrs[HLF_NONE as usize] = 0);

    let mut id_s = -1;
    let mut id_snc = 0;
    for hlf in 1..HLF_COUNT {
        let name = hlf_names[hlf as usize];
        // SAFETY: a builtin group name, NUL-terminated.
        let id = syn_check_group(unsafe { CStr::from_ptr(name) }.to_bytes());
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
            msg_grid.blending = syn_attr2entry(attr).hl_blend > -1;
        }
        ui_call_hl_group_set(unsafe { cstr_to_string(name) }, Integer::from(attr));
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
        let userhl = cstr::owned(format!("User{}", i + 1).as_bytes());
        let id = syn_name2id(&userhl);
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
