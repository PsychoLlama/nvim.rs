//! The info window and the selection that feeds it.
//!
//! [`pum_set_selected`] scrolls the menu to the new selection and, when
//! `'completeopt'` asks for it, fills a preview window (a split, or a
//! float under `popup`) with the item's `info` text.
//!
//! The float and the split are two different windows with two different
//! lifetimes: the float is found or created by `winfloat`'s preview helpers
//! and only hidden when it is not wanted, while the split is opened by
//! `prepare_tagpreview` and edited into a scratch buffer. Both end up in
//! [`pum_preview_set_text`].

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::guard::{Allow, Suppress};
use crate::message::emsg_ptr;
use crate::option::boolean_optval;
use crate::pos::MAXCOL;
use crate::types::OptionSetFlags;
use crate::winlayer::Win;

/// How tall a preview split starts out.
const PUM_PREVIEW_HEIGHT: c_int = 3;

/// `STATIC_CSTR_AS_OPTVAL`: an option value borrowed from a string literal.
fn static_optval(value: &'static ::core::ffi::CStr) -> OptVal {
    OptVal {
        type_0: kOptValTypeString,
        data: OptValData {
            string: String_0::from_raw_parts(value.as_ptr().cast_mut(), value.count_bytes()),
        },
    }
}

/// The selected item's `info` text, if it has one.
///
/// Answers `None` for "nothing selected" as well; upstream reads
/// `pum_array[pum_selected]` after testing only the lower bound, which is out
/// of range whenever a caller passes an index past the end.
///
/// # Safety
/// The item array must be the live one.
unsafe fn pum_selected_info() -> Option<*mut c_char> {
    // SAFETY: the array outlives the menu.
    let selected = pum_selected.get();
    let items = unsafe { pum_items() };
    if selected < 0 || selected as usize >= items.len() {
        return None;
    }
    let info = items[selected as usize].pum_info;
    (!info.is_null()).then_some(info)
}

/// Fill `win`'s buffer with `info`, one buffer line per `\n`-separated line.
///
/// Answers the number of lines written and the widest of them in cells,
/// which is what the caller sizes the window with. A trailing empty line is
/// dropped; empty lines anywhere else are kept.
///
/// # Safety
/// `win` must be live and `info` NUL-terminated. `info` is written through
/// and restored, so it must be writable — the callers own it.
unsafe fn pum_preview_set_text(win: *mut win_T, info: *mut c_char) -> (linenr_T, c_int) {
    // SAFETY: the buffer is `win`'s own and `nvim_buf_set_lines` copies out of
    // `replacement` before it is freed.
    let buf = unsafe { (*win).w_buffer };
    unsafe { (*buf).b_p_ma = 1 };

    let mut lines: Vec<Object> = Vec::new();
    let mut max_width = 0;
    let mut curr = info;
    while !curr.is_null() {
        let next = unsafe { strchr(curr, '\n' as c_int) };
        if !next.is_null() {
            // Terminate the line for `cstr_to_string` and the width
            // measurement, then put the newline back.
            unsafe { *next = 0 };
        }
        // An empty line is only dropped when it is the last one.
        if unsafe { *curr } == 0 && next.is_null() {
            break;
        }

        // 'wrap' off while measuring: 'showbreak'/'linebreak' would
        // inflate the answer in a narrow window.
        let save_wrap = unsafe { (*win).w_onebuf_opt.wo_wrap };
        unsafe { (*win).w_onebuf_opt.wo_wrap = 0 };
        max_width =
            max_width.max(unsafe { win_linetabsize(Win::new(win), 0, curr, MAXCOL as c_int) });
        unsafe { (*win).w_onebuf_opt.wo_wrap = save_wrap };

        lines.push(Object::String(unsafe { cstr_to_string(curr) }));

        if !next.is_null() {
            unsafe { *next = b'\n' as c_char };
        }
        curr = if next.is_null() {
            ::core::ptr::null_mut()
        } else {
            unsafe { next.offset(1) }
        };
    }

    // Hand the lines over as an api `Array`, which `api_free_array` frees
    // with `xfree` — so the buffer has to come from `xmalloc`, not `Vec`.
    let mut replacement = ARRAY_DICT_INIT;
    if !lines.is_empty() {
        replacement.items =
            unsafe { xmalloc(size_of::<Object>().wrapping_mul(lines.len())) }.cast::<Object>();
        unsafe { ::core::ptr::copy_nonoverlapping(lines.as_ptr(), replacement.items, lines.len()) };
        replacement.size = lines.len() as size_t;
        replacement.capacity = replacement.size;
    }
    let lnum = lines.len() as linenr_T;

    let mut arena = ARENA_EMPTY;
    // Setting the lines is the editor's own doing, not a plugin's.
    let unlocked = Allow::text_changes();
    let set = unsafe {
        nvim_buf_set_lines(
            0,
            (*buf).handle as Buffer,
            0,
            -1,
            false,
            replacement,
            &raw mut arena,
        )
    };
    drop(unlocked);
    if let Err(mut err) = set {
        unsafe { emsg_ptr(err.message_or_empty().as_ptr()) };
        err.clear();
    }
    unsafe { arena_mem_free(arena_finish(&raw mut arena)) };
    unsafe { api_free_array(replacement) };
    unsafe { (*buf).b_p_ma = 0 };

    (lnum, max_width)
}

/// Place the floating info window beside the menu.
///
/// It goes to the right when the text fits there, otherwise to the left,
/// otherwise on whichever side has more room. Answers false — with the
/// window hidden — when neither side has enough space to be worth it.
///
/// # Safety
/// `wp` must be a live float and the menu's placement settled.
unsafe fn pum_adjust_info_position(wp: *mut win_T, width: c_int) -> bool {
    // SAFETY: `wp` is live and `win_config_float` takes the config by value.
    let border_width = unsafe { pum_border_width() };
    let col = pum_col.get() + pum_width.get() + 1 + border_width.max(pum_scrollbar.get());
    // TODO(glepnir): support config align border by using completepopup
    // align menu
    let right_extra = Columns.get() - col;
    let left_extra = pum_col.get() - 2;

    // TODO(glepnir): Replace the hardcoded value (10) with values from the
    // 'completepopup' width/height options.
    let max_extra = right_extra.max(left_extra);
    if max_extra < 10 {
        unsafe { (*wp).w_config.hide = true };
        return false;
    }

    if right_extra > width {
        unsafe { (*wp).w_config.width = width };
        unsafe { (*wp).w_config.col = f64::from(col - 1) };
    } else if left_extra > width {
        unsafe { (*wp).w_config.width = width };
        unsafe { (*wp).w_config.col = f64::from(pum_col.get() - width - 1) };
    } else {
        // Neither side fits the text; take the bigger one.
        unsafe { (*wp).w_config.width = max_extra };
        unsafe {
            (*wp).w_config.col = f64::from(if right_extra > left_extra {
                col - 1
            } else {
                pum_col.get() - max_extra - 1
            })
        };
    }

    unsafe { (*wp).w_config.anchor = 0 }; // NW: align its top with the menu's top
    let count = unsafe { (*(*wp).w_buffer).b_ml.ml_line_count };
    unsafe { (*wp).w_view_width = (*wp).w_config.width };
    unsafe {
        (*wp).w_config.height = plines_m_win(Win::new(wp), (*wp).w_topline, count, Rows.get())
    };
    unsafe { (*wp).w_config.row = f64::from(pum_row.get()) };
    unsafe { (*wp).w_config.hide = false };
    win_config_float(unsafe { Win::new(wp) }, unsafe { (*wp).w_config.clone() });
    true
}

/// Set the info text of the current item, for `nvim__complete_set`.
///
/// Answers the info window, or null when the menu is down, the item is not
/// the selected one, or there is no room for the window.
///
/// # Safety
/// `info` must be a writable NUL-terminated string owned by the caller.
pub unsafe fn pum_set_info(selected: c_int, info: *mut c_char) -> *mut win_T {
    // SAFETY: the preview helpers answer live windows or null.
    if !pum_is_visible.get() || !unsafe { compl_match_curr_select(selected) } {
        return ::core::ptr::null_mut();
    }
    unsafe { block_autocmds() };
    RedrawingDisabled.set(RedrawingDisabled.get() + 1);
    no_u_sync.set(no_u_sync.get() + 1);

    let mut wp = if let Some(wp) = win_float_find_preview() {
        wp
    } else if let Some(mut fresh) = win_float_create_preview(false, true) {
        fresh.w_topline = 1;
        fresh.w_onebuf_opt.wo_wfb = 1;
        fresh
    } else {
        // NOTE: leaves autocmds blocked and the two counters raised, as
        // upstream does.
        return ::core::ptr::null_mut();
    };

    let (_lnum, max_info_width) = unsafe { pum_preview_set_text(wp.raw(), info) };
    no_u_sync.set(no_u_sync.get() - 1);
    RedrawingDisabled.set(RedrawingDisabled.get() - 1);
    unsafe { redraw_later(wp.raw(), UPD_NOT_VALID) };

    // `unblock_autocmds` has to run whichever way the placement went, so
    // the answer is settled before it rather than after.
    let placed = unsafe { pum_adjust_info_position(wp.raw(), max_info_width) }.then(|| wp.raw());
    unsafe { unblock_autocmds() };
    placed.unwrap_or(::core::ptr::null_mut())
}

/// Scroll the menu so that `pum_selected` is visible, with context around it.
///
/// A jump of more than a few items is treated as a PageUp/PageDown and
/// scrolls a whole page; a small step keeps three lines of context.
fn pum_scroll_to_selected() {
    let (selected, height, size) = (pum_selected.get(), pum_height.get(), pum_size.get());
    let scroll_offset = selected - height;
    let mut first = pum_first.get();

    if first > selected - 4 {
        // Scroll up towards the selection; a jump means a PageUp.
        if first > selected - 2 {
            first = (first - (height - 2)).max(0).min(selected);
        } else {
            first = selected;
        }
    } else if first < scroll_offset + 5 {
        // Scroll down towards the selection; a jump means a PageDown.
        if first < scroll_offset + 3 {
            first = (first + height - 2).max(scroll_offset + 1);
        } else {
            first = scroll_offset + 1;
        }
    }

    // A few lines of context around the selection, when the menu is tall
    // enough for any to fit.
    let context = (height / 2).min(3);
    if height > 2 {
        if first > selected - context {
            first = (selected - context).max(0);
        } else if first < selected + context - height + 1 {
            first = selected + context - height + 1;
        }
    }

    pum_first.set(first.min(size - height));
}

/// Whether `'completeopt'` wants an info window for this call.
///
/// Skipped when the screen is too short, when the placement is being redone
/// (`repeat` above 1), and — for the split, not the float — inside the
/// command-line window.
fn wants_info_window(cot_flags: c_uint, repeat: c_int) -> bool {
    Rows.get() > 10
        && repeat <= 1
        && cot_flags & (kOptCotFlagPreview | kOptCotFlagPopup) != 0
        && !(cot_flags & kOptCotFlagPreview != 0 && cmdwin_type.get() != 0)
}

/// Open (or reuse) the preview window and put the item's `info` in it.
///
/// Answers whether a window was resized, which is what makes `pum_display`
/// redo the whole placement.
///
/// # Safety
/// Must be called with a selected item that has info text. Autocommands run
/// from here, so nothing may be held across it.
unsafe fn pum_show_info(
    info: *mut c_char,
    repeat: c_int,
    use_float: bool,
    prev_selected: c_int,
) -> bool {
    // SAFETY: every window pointer below is re-checked with `win_valid`
    // after anything that can run autocommands.
    let mut resized = false;
    let curwin_save = curwin.get();
    let curtab_save = curtab.get();

    if use_float {
        unsafe { block_autocmds() };
    }

    // A preview split is 3 lines by default, less if 'previewheight' is.
    g_do_tagpreview.set(PUM_PREVIEW_HEIGHT);
    if p_pvh.get() > 0 && p_pvh.get() < OptInt::from(g_do_tagpreview.get()) {
        g_do_tagpreview.set(p_pvh.get() as c_int);
    }
    let redraw_off = Suppress::redraw();
    // An autocommand that syncs undo here does weird things to the tree.
    let no_sync = Suppress::undo_sync();

    if !use_float {
        resized = unsafe { prepare_tagpreview(false) };
    } else {
        if let Some(wp) = win_float_find_preview() {
            unsafe { win_enter(wp.raw(), false) };
        } else if win_float_create_preview(true, true).is_some() {
            resized = true;
        }
    }

    drop(no_sync);
    drop(redraw_off);
    g_do_tagpreview.set(0);

    if unsafe { (*curwin.get()).w_onebuf_opt.wo_pvw } != 0
        || unsafe { (*curwin.get()).w_float_is_info }
    {
        let mut res = Ok(());
        if !resized
            && unsafe { (*curbuf.get()).b_nwindows } == 1
            && unsafe { (*curbuf.get()).b_fname }.is_null()
            && buf_is_nofile(current_buf())
            && unsafe { *(*curbuf.get()).b_p_bh } == b'w' as c_char
        {
            // Already a "wipeout" buffer: just empty it.
            buf_clear();
        } else {
            let no_sync = Suppress::undo_sync();
            res = unsafe {
                do_ecmd(
                    0,
                    ::core::ptr::null_mut(),
                    ::core::ptr::null_mut(),
                    ::core::ptr::null_mut(),
                    ECMD_ONE as linenr_T,
                    0,
                    ::core::ptr::null_mut(),
                )
            };
            drop(no_sync);

            if res.is_ok() {
                // A new, empty, throwaway buffer.
                for (option, value) in [
                    (kOptSwapfile, boolean_optval(Some(false))),
                    (kOptBuflisted, boolean_optval(Some(false))),
                    (kOptBuftype, static_optval(c"nofile")),
                    (kOptBufhidden, static_optval(c"wipe")),
                    (kOptDiff, boolean_optval(Some(false))),
                ] {
                    set_option_value_give_err(option, value, OptionSetFlags::LOCAL);
                }
            }
        }

        if res.is_ok() {
            resized = unsafe {
                pum_fill_info(info, repeat, use_float, prev_selected, resized, curwin_save)
            };
            resized = unsafe { pum_restore_window(curwin_save, curtab_save, resized) };
        }
    }

    if use_float {
        unsafe { unblock_autocmds() };
    }
    resized
}

/// Put the info text in the window `pum_show_info` just entered and size it.
///
/// # Safety
/// `curwin` must be the preview window.
unsafe fn pum_fill_info(
    info: *mut c_char,
    repeat: c_int,
    use_float: bool,
    prev_selected: c_int,
    mut resized: bool,
    curwin_save: *mut win_T,
) -> bool {
    // SAFETY: `curwin`/`curbuf` are the preview window and its buffer;
    // `curwin_save` is the window completion started in and is re-validated.
    let (lnum, max_info_width) = unsafe { pum_preview_set_text(curwin.get(), info) };

    // Grow a preview split to fit the text, up to 'previewheight'.
    if repeat == 0 && !use_float {
        let lnum = lnum.min(p_pvh.get() as linenr_T);
        if linenr_T::from(unsafe { (*curwin.get()).w_height }) < lnum {
            win_setheight(lnum as c_int);
            resized = true;
        }
    }

    unsafe { (*curbuf.get()).b_changed = 0 };
    unsafe { (*curbuf.get()).b_p_ma = 0 };
    if pum_selected.get() != prev_selected {
        unsafe { (*curwin.get()).w_topline = 1 };
    } else if unsafe { (*curwin.get()).w_topline } > unsafe { (*curbuf.get()).b_ml.ml_line_count } {
        unsafe { (*curwin.get()).w_topline = (*curbuf.get()).b_ml.ml_line_count };
    }
    unsafe { (*curwin.get()).w_cursor.lnum = 1 };
    unsafe { (*curwin.get()).w_cursor.col = 0 };

    if use_float
        && !unsafe { pum_adjust_info_position(curwin.get(), max_info_width) }
        && win_valid(curwin_save)
    {
        unsafe { win_enter(curwin_save, false) };
    }
    resized
}

/// Go back to the window the completion started in, redrawing on the way.
///
/// Does nothing when opening the preview did not leave that window, which is
/// the float case once the float already existed.
///
/// # Safety
/// `curwin_save`/`curtab_save` are re-checked before use.
unsafe fn pum_restore_window(
    curwin_save: *mut win_T,
    curtab_save: *mut tabpage_T,
    resized: bool,
) -> bool {
    // SAFETY: both pointers are validated before they are entered.
    let left_window = curwin.get() != curwin_save && win_valid(curwin_save);
    let left_tab = curtab.get() != curtab_save && valid_tabpage(curtab_save);
    if !left_window && !left_tab {
        return resized;
    }
    if left_tab {
        unsafe { goto_tabpage_tp(curtab_save, false, false) };
    }

    // On the first completion, with the preview window not resized, skip
    // its status line redraw.
    if ins_compl_active() && !resized {
        unsafe { (*curwin.get()).w_redr_status = false };
    }

    validate_cursor(unsafe { Win::current() });
    unsafe { redraw_later(curwin.get(), UPD_SOME_VALID) };

    // A resized preview window needs the buffer view updated, which only
    // happens in the window itself.
    if resized && win_valid(curwin_save) {
        let no_sync = Suppress::undo_sync();
        unsafe { win_enter(curwin_save, true) };
        drop(no_sync);
        update_topline(unsafe { Win::current() });
    }

    // Draw the screen before the menu goes back on top of it, with the
    // status lines enabled again.
    // TODO(bfredl): can simplify, get rid of the flag munging? or at
    // least eliminate the extra redraw before win_enter()?
    pum_is_visible.set(false);
    let _ = unsafe { update_screen() };
    pum_is_visible.set(true);

    if !resized && win_valid(curwin_save) {
        let _no_sync = Suppress::undo_sync();
        unsafe { win_enter(curwin_save, true) };
    }

    // Autocommands may have changed it again.
    pum_is_visible.set(false);
    let _ = unsafe { update_screen() };
    pum_is_visible.set(true);
    resized
}

/// Select item `n`, scrolling the menu and updating the info window.
///
/// `repeat` counts how often `pum_display` has already redone the placement:
/// 0 opens the preview window normally, 1 opens it without setting its size,
/// 2 does not open it at all.
///
/// Answers true when a window was resized, so the caller must recompute the
/// menu's placement.
///
/// # Safety
/// The item array must be live. Autocommands run from here.
pub(crate) unsafe fn pum_set_selected(n: c_int, repeat: c_int) -> bool {
    // SAFETY: the array outlives the menu; every window pointer is
    // re-validated after anything that can run autocommands.
    let prev_selected = pum_selected.replace(n);
    let cot_flags = unsafe { get_cot_flags() };
    let use_float = cot_flags & kOptCotFlagPopup != 0;
    let info = unsafe { pum_selected_info() };

    // Back to no selection, or to one with nothing to show: hide the
    // float rather than closing it, so the next item can reuse it.
    if use_float
        && info.is_none()
        && let Some(mut wp) = win_float_find_preview()
    {
        wp.w_config.hide = true;
        win_config_float(wp, wp.w_config.clone());
    }

    if pum_selected.get() < 0 || pum_selected.get() >= pum_size.get() {
        return false;
    }
    pum_scroll_to_selected();

    match info {
        Some(info) if wants_info_window(cot_flags, repeat) => unsafe {
            pum_show_info(info, repeat, use_float, prev_selected)
        },
        _ => false,
    }
}
