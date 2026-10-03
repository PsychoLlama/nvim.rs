//! Showing the matches: the command-line popup menu and the wildmenu.
//!
//! The two renderings of the same match list.  [`cmdline_pum_create`] turns
//! it into `pum_display` items; [`redraw_wildmenu`] draws the one-line
//! statusline form instead.  [`cmdline_compl_use_pum`] is the choice between
//! them, and the `cmdline_compl_*` accessors are what the popup menu's own
//! drawing reads.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::charset::{ptr2cells_at, transchar, transchar_byte};
use crate::cmdexpand::state::{save_p_ls, save_p_wmh, wild_menu_showing};
use crate::cstr;
use crate::drawscreen::state::cmdline_row;
use crate::drawscreen::win_redraw_last_status;
use crate::ex_getln::cmd_screencol;
use crate::grid::{
    default_gridview, grid_line_fill, grid_line_flush, grid_line_puts_bytes, grid_line_start,
};
use crate::highlight::win_hl_attr;
use crate::highlight_group::HLF_WM;
use crate::mbyte::cluster_len;
use crate::menu::is_separator;
use crate::message::state::msg_scrolled;
use crate::message::{msg_grid_view, msg_scroll_up};
use crate::option::csh_like_shell;
use crate::option::vars::{P_LS, P_WMH, p_ls, p_wmh, wop_flags};
use crate::options::kOptWopFlagPum;
use crate::popupmenu::{pum_display_items, pum_undisplay, pum_visible};
use crate::statusline::{fillchar_status_of, hl_attr};
use crate::types::ui::{kUICmdline, kUIPopupmenu, kUIWildmenu};
use crate::types::{ExpandContext, OptInt};
use crate::ui::state::{Columns, Rows};
use crate::ui::ui_has;
use crate::window::{global_stl_height, last_status};
use crate::winlayer::graph::cmdline_win;
use crate::winlayer::{Cc, Win, current_topframe, last_window};
use core::ffi::{CStr, c_int, c_uint};

/// The command line's completion popup menu: the rows, and what the menu's
/// drawing asks about the completion while it is up.
///
/// The rows' text points into the matches, so the menu keeps whichever list
/// it was built from alive: the completion's own (which the completion only
/// drops after removing the menu) or, for a listing nothing kept, its own
/// copy.
pub(crate) struct CmdlinePum {
    rows: Vec<PumItem>,
    /// The matches the rows point into, when the completion did not keep
    /// them itself.
    _kept: Vec<XString>,
    /// The text the matches replaced, which the menu highlights in each
    /// item: upstream reads it as the command line's `xp_orig`.
    orig: Option<XString>,
    /// The context completed, for whether its matches were fuzzy.
    context: ExpandContext,
}

/// Create the completion popup menu with items from `matches`, or from the
/// completion's own matches when `matches` is `None`.
pub(crate) fn cmdline_pum_create(
    ccline: Cc,
    expand: &Expand,
    matches: Option<Vec<XString>>,
    showtail: bool,
    noselect: bool,
) {
    let list = matches.as_deref().unwrap_or(expand.matches());
    // Add all the completion matches.
    let rows = list
        .iter()
        .map(|m| PumItem {
            // C's SHOW_MATCH(i).
            pum_text: m.as_ptr().cast_mut().wrapping_add(if showtail {
                showmatches_gettail(m, false)
            } else {
                0
            }),
            pum_info: core::ptr::null_mut(),
            pum_extra: core::ptr::null_mut(),
            pum_kind: core::ptr::null_mut(),
            pum_cpt_source_idx: 0,
            pum_user_abbr_hlattr: -1,
            pum_user_kind_hlattr: -1,
        })
        .collect();
    compl_match_array.set(Some(CmdlinePum {
        rows,
        _kept: matches.unwrap_or_default(),
        orig: expand.orig.clone(),
        context: expand.context,
    }));

    // Compute the popup menu starting column.
    let line = ccline.text_bytes();
    let from = expand.pattern.min(line.len());
    let endpos = if showtail {
        from + showmatches_gettail(&line[from..], noselect)
    } else {
        from
    };
    let col = as_count(endpos);
    compl_startcol.set(if ui_has(kUICmdline) && cmdline_win.get().is_none() {
        col
    } else {
        cmd_screencol(col)
    });
}

pub fn cmdline_pum_display(changed_array: bool) {
    // A copy: placing the menu can run autocommands that remove it.
    let mut rows = compl_match_array.with(|pum| pum.as_ref().map(|pum| pum.rows.clone()));
    let rows = rows.get_or_insert_default();
    pum_display_items(
        rows,
        compl_selected.get(),
        changed_array,
        compl_startcol.get(),
    );
}

/// True if the cmdline completion popup menu is being displayed.
pub fn cmdline_pum_active() -> bool {
    pum_visible() && compl_match_array.with(Option::is_some)
}

/// Remove the cmdline completion popup menu (if present) and free the list of
/// items.
pub fn cmdline_pum_remove(defer_redraw: bool) {
    pum_undisplay(!defer_redraw);
    compl_match_array.set(None);
}

pub(crate) fn cmdline_pum_cleanup(cclp: Cc) {
    cmdline_pum_remove(false);
    wildmenu_cleanup(cclp);
}

/// The text the cmdline completion's matches replaced, for the popup menu to
/// highlight in each item; `None` when there is no cmdline menu.
pub fn cmdline_compl_pattern() -> Option<XString> {
    compl_match_array.with(|pum| pum.as_ref().and_then(|pum| pum.orig.clone()))
}

/// True if the cmdline completion popup menu's matches are fuzzy ones.
pub fn cmdline_compl_is_fuzzy() -> bool {
    compl_match_array.with(|pum| {
        pum.as_ref().is_some_and(|pum| {
            let mut context = Expand::new();
            context.context = pum.context;
            cmdline_fuzzy_completion_supported(&context)
        })
    })
}

/// Whether the popup menu should be used for the cmdline completion wildmenu.
///
/// `need_wildmenu` says whether the current `'wildmode'` part wants one.
pub(crate) fn cmdline_compl_use_pum(need_wildmenu: bool) -> bool {
    (need_wildmenu
        && wop_flags.get() & kOptWopFlagPum as c_uint != 0
        && !(ui_has(kUICmdline) && cmdline_win.get().is_none()))
        || ui_has(kUIWildmenu)
        || (ui_has(kUICmdline) && ui_has(kUIPopupmenu))
}

/// The number of bytes that should be skipped in the wildmenu at the start
/// of `s`.
///
/// These are backslashes used for escaping.  Backslashes *are* shown in help
/// tags and in search pattern completion matches.
pub(crate) fn skip_wildmenu_char(expand: &Expand, s: &[u8]) -> usize {
    let at = |i: usize| s.get(i).copied().unwrap_or(0);
    let ctx = expand.context;
    let escaped = at(0) == b'\\' && at(1) != 0;
    if (escaped && ctx != ExpandContext::Help && ctx != ExpandContext::PatternInBuf)
        || ((ctx == ExpandContext::Menus || ctx == ExpandContext::Menunames)
            && (at(0) == b'\t' || escaped))
    {
        // TODO(bfredl): Why in the actual fuck are we special casing the
        // shell variety deep in the redraw logic?  Shell special
        // snowflakiness should already be eliminated multiple layers
        // before reaching the screen infrastructure.
        if expand.shell && csh_like_shell() && at(1) == b'\\' && at(2) == b'!' {
            return 2;
        }
        return 1;
    }
    0
}

/// The length of an item as it will be shown in the status line.
pub(crate) fn wildmenu_match_len(expand: &Expand, s: &CStr) -> c_int {
    let ctx = expand.context;
    let emenu = ctx == ExpandContext::Menus || ctx == ExpandContext::Menunames;

    // Check for menu separators - replace with '|'.
    if emenu && is_separator(s) {
        return 1;
    }

    let s = s.to_bytes();
    let mut len = 0;
    let mut p = 0;
    while p < s.len() {
        p += skip_wildmenu_char(expand, &s[p..]);
        if p >= s.len() {
            break;
        }
        len += ptr2cells_at(&s[p..]);
        p += cluster_len(&s[p..]);
    }

    len
}

/// Show wildchar matches in the status line.
///
/// At least the `match_idx` item is shown.  We start at item `first_match` in
/// the list and show all matches that fit; if inversion is possible we use it,
/// else `=` characters are used.
pub(crate) fn redraw_wildmenu(
    expand: &Expand,
    matches: &[XString],
    match_idx: c_int,
    showtail: bool,
) {
    // Where the listing starts, remembered across redraws so that paging
    // through the matches does not jump.
    static first_match: GlobalCell<c_int> = GlobalCell::new(0);

    let num_matches = as_count(matches.len());
    // C's SHOW_MATCH().
    let show_match = |i: c_int| -> &CStr {
        let m = &matches[usize::try_from(i).unwrap_or(0)];
        let tail = if showtail {
            showmatches_gettail(m, false)
        } else {
            0
        };
        cstr::suffix(m.as_cstr(), tail)
    };

    let mut highlight = true;
    let mut selection: Option<(usize, c_int)> = None;
    let mut selend = 0;
    let mut add_left = false;
    let mut i;

    let mut match_idx = match_idx;
    if match_idx == -1 {
        // Don't show match but original text.
        match_idx = 0;
        highlight = false;
    }
    // Length in screen cells; count 1 for the ending ">".
    let mut clen = wildmenu_match_len(expand, show_match(match_idx)) + 3;
    if match_idx == 0 {
        first_match.set(0);
    } else if match_idx < first_match.get() {
        // Jumping left, as far as we can go.
        first_match.set(match_idx);
        add_left = true;
    } else {
        // Check if match fits on the screen.
        i = first_match.get();
        while i < match_idx {
            clen += wildmenu_match_len(expand, show_match(i)) + 2;
            i += 1;
        }
        if first_match.get() > 0 {
            clen += 2;
        }
        // Jumping right, put match at the left.
        if clen > Columns.get() {
            first_match.set(match_idx);
            // If showing the last match, we can add some on the left.
            clen = 2;
            i = match_idx;
            while i < num_matches {
                clen += wildmenu_match_len(expand, show_match(i)) + 2;
                if clen >= Columns.get() {
                    break;
                }
                i += 1;
            }
            if i == num_matches {
                add_left = true;
            }
        }
    }
    if add_left {
        while first_match.get() > 0 {
            clen += wildmenu_match_len(expand, show_match(first_match.get() - 1)) + 2;
            if clen >= Columns.get() {
                break;
            }
            first_match.set(first_match.get() - 1);
        }
    }

    let (group, fillchar) = fillchar_status_of(Win::current());
    let attr = win_hl_attr(Win::current(), group as c_int);

    let mut buf = Vec::new();
    if first_match.get() > 0 {
        buf.extend_from_slice(b"< ");
    }
    clen = as_count(buf.len());

    i = first_match.get();
    while clen + wildmenu_match_len(expand, show_match(i)) + 2 < Columns.get() {
        if i == match_idx {
            selection = Some((buf.len(), clen));
        }

        let s = show_match(i);
        // Check for menu separators - replace with '|'.
        let ctx = expand.context;
        let emenu = ctx == ExpandContext::Menus || ctx == ExpandContext::Menunames;
        if emenu && is_separator(s) {
            let bar = transchar(c_int::from(b'|'));
            let bar = cstr::in_chars(&bar).to_bytes();
            buf.extend_from_slice(bar);
            clen += as_count(bar.len());
        } else {
            let s = s.to_bytes();
            let mut p = 0;
            while p < s.len() {
                p += skip_wildmenu_char(expand, &s[p..]);
                if p >= s.len() {
                    break;
                }
                clen += ptr2cells_at(&s[p..]);
                let l = cluster_len(&s[p..]);
                if l > 1 {
                    buf.extend_from_slice(&s[p..p + l]);
                    p += l;
                } else {
                    let shown = transchar_byte(c_int::from(s[p]));
                    buf.extend_from_slice(cstr::in_chars(&shown).to_bytes());
                    p += 1;
                }
            }
        }
        if i == match_idx {
            selend = buf.len();
        }

        buf.extend_from_slice(b"  ");
        clen += 2;
        i += 1;
        if i == num_matches {
            break;
        }
    }

    if i != num_matches {
        buf.push(b'>');
        clen += 1;
    }

    let mut row = cmdline_row.get() - 1;
    if row >= 0 {
        if wild_menu_showing.get() == 0 {
            if msg_scrolled.get() > 0 {
                // Put the wildmenu just above the command line.  If there
                // is no room, scroll the screen one line up.
                if cmdline_row.get() == Rows.get() - 1 {
                    msg_scroll_up(false, false);
                    msg_scrolled.set(msg_scrolled.get() + 1);
                } else {
                    cmdline_row.set(cmdline_row.get() + 1);
                    row += 1;
                }
                wild_menu_showing.set(WM_SCROLLED);
            } else {
                // Create status line if needed by setting 'laststatus' to
                // 2.  Set 'winminheight' to zero to avoid that the window
                // is resized.
                if needs_status_line() {
                    save_p_ls.set(p_ls() as c_int);
                    save_p_wmh.set(p_wmh() as c_int);
                    P_LS.set(2 as OptInt);
                    P_WMH.set(0 as OptInt);
                    last_status(false);
                }
                wild_menu_showing.set(WM_SHOWN);
            }
        }

        // Tricky: the wildmenu can be drawn either over a status line, or
        // at empty scrolled space in the message output.
        grid_line_start(
            if wild_menu_showing.get() == WM_SCROLLED {
                msg_grid_view()
            } else {
                default_gridview()
            },
            row,
        );

        grid_line_puts_bytes(0, &buf, attr);
        if let Some((selstart, selstart_col)) = selection
            && highlight
        {
            grid_line_puts_bytes(
                selstart_col,
                &buf[selstart..selend],
                hl_attr(HLF_WM as c_int),
            );
        }

        grid_line_fill(clen, Columns.get(), fillchar, attr);

        grid_line_flush();
    }

    win_redraw_last_status(current_topframe());
}

/// Whether the wildmenu has to turn 'laststatus' on to get a line to draw
/// in: upstream's `lastwin->w_status_height == 0 && global_stl_height() == 0`.
fn needs_status_line() -> bool {
    last_window().is_some_and(|wp| wp.w_status_height == 0) && global_stl_height() == 0
}
