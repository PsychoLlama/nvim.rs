//! The screen changed size, and the autocommands that report it.
//!
//! [`win_new_screensize`] is the entry point when `'lines'` or `'columns'`
//! moved: it redistributes the new room over the frame tree and recomputes
//! every window's position.  The rest is the `WinScrolled`/`WinResized`
//! machinery -- [`snapshot_windows_scroll_size`] records every window's view
//! and size, [`scan_windows`] compares the current state against that
//! snapshot, and [`may_trigger_win_scrolled_resized`] fires the events with the
//! `v:event` dict [`win_info_dict`] builds.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::mem::size_of;
use core::ptr;

use super::arith::NextCurwin;
use super::*;
use crate::autocmd::{
    EVENT_WINRESIZED, EVENT_WINSCROLLED, apply_autocmds, event_ignored, has_event,
};
use crate::buffer::BufRef;
use crate::eval::typval::{
    tv_dict_add_dict, tv_dict_add_list, tv_dict_add_tv, tv_dict_alloc, tv_dict_extend,
    tv_dict_set_keys_readonly, tv_dict_unref, tv_list_alloc, tv_list_append_owned_tv,
};
use crate::eval::{get_v_event, restore_v_event};
use crate::ex_getln::compute_cmdrow;
use crate::garray::{ga_grow, ga_init};
use crate::global_cell::GlobalCell;
use crate::main::{Columns, Rows, curbuf, p_ch, p_window, skip_win_fix_scroll};
use crate::option::option_was_set;
use crate::options::kOptWindow;
use crate::strings::vim_snprintf;
use crate::types::{
    FAIL, OK, OptInt, VAR_NUMBER, VAR_UNLOCKED, buf_T, dict_T, garray_T, linenr_T, list_T,
    ptrdiff_t, save_v_event_T, size_t, typval_T, typval_vval_union, varnumber_T,
};
use crate::winfloat::win_reconfig_floats;
use crate::winlayer::{Win, windows};

/// The rows the frame tree has to itself: the screen minus the command line,
/// the tab line and a global status line.
fn frame_rows() -> c_int {
    (Rows.get() as OptInt - p_ch.get() - tabline_rows() as OptInt - global_stl_rows() as OptInt)
        as c_int
}

pub fn win_new_screensize() {
    static old_Rows: GlobalCell<c_int> = GlobalCell::new(0);
    static old_Columns: GlobalCell<c_int> = GlobalCell::new(0);
    if old_Rows.get() != Rows.get() {
        // If 'window' uses the whole screen, keep it using the whole screen.
        if p_window.get() == (old_Rows.get() - 1) as OptInt
            || (old_Rows.get() == 0 && !option_was_set(kOptWindow))
        {
            p_window.set((Rows.get() - 1) as OptInt);
        }
        old_Rows.set(Rows.get());
        new_screen_rows();
    }
    if old_Columns.get() != Columns.get() {
        old_Columns.set(Columns.get());
        new_screen_cols();
    }
}

pub fn win_new_screen_rows() {
    new_screen_rows();
}

/// Give the windows the new number of screen rows.
pub(crate) fn new_screen_rows() {
    if windows().next().is_none() {
        return; // not initialized yet
    }
    let top = current_topframe();
    let h = frame_rows().max(minheight(top, NextCurwin::Unset));
    // First try setting the heights of windows with 'winfixheight'; if that
    // does not result in the right height, forget about that option.
    new_height(top, h, false, true, false);
    if !arith::frame_check_height(top, h) {
        new_height(top, h, false, false, false);
    }
    comp_positions();
    win_reconfig_floats();
    // SAFETY: recomputes the row the command line starts on.
    unsafe { compute_cmdrow() };
    cur_tab().tp_ch_used = p_ch.get();
    if !skip_win_fix_scroll.get() {
        fix_scroll(true);
    }
}

pub fn win_new_screen_cols() {
    new_screen_cols();
}

/// Give the windows the new number of screen columns.
pub(crate) fn new_screen_cols() {
    if windows().next().is_none() {
        return; // not initialized yet
    }
    let top = current_topframe();
    // First try setting the widths of windows with 'winfixwidth'; if that does
    // not result in the right width, forget about that option.
    new_width(top, Columns.get(), false, true);
    if !arith::frame_check_width(top, Columns.get()) {
        new_width(top, Columns.get(), false, false);
    }
    comp_positions();
    win_reconfig_floats();
}

// ---------------------------------------------------------------------------
// WinScrolled and WinResized

pub fn snapshot_windows_scroll_size() {
    for mut wp in windows() {
        snapshot_window(&mut wp);
    }
}

/// Remember one window's view and size, so the next check can tell whether
/// either moved.
fn snapshot_window(wp: &mut Win) {
    wp.w_last_topline = wp.w_topline;
    wp.w_last_topfill = wp.w_topfill;
    wp.w_last_leftcol = wp.w_leftcol;
    wp.w_last_skipcol = wp.w_skipcol;
    wp.w_last_width = wp.w_width;
    wp.w_last_height = wp.w_height;
}

pub unsafe fn may_make_initial_scroll_size_snapshot() {
    if !did_initial_scroll_size_snapshot.get() {
        did_initial_scroll_size_snapshot.set(true);
        snapshot_windows_scroll_size();
    }
}

/// A dictionary with the six numbers a `WinScrolled`/`WinResized` `v:event`
/// entry carries, or null when one of them could not be added.
fn win_info_dict(deltas: [c_int; 6]) -> *mut dict_T {
    let d = new_dict();
    let keys = [
        c"width".to_bytes(),
        c"height".to_bytes(),
        c"topline".to_bytes(),
        c"topfill".to_bytes(),
        c"leftcol".to_bytes(),
        c"skipcol".to_bytes(),
    ];
    for (key, value) in keys.iter().zip(deltas) {
        let mut tv = typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_number: value as varnumber_T,
            },
        };
        let (name, len) = (key.as_ptr().cast::<c_char>(), key.len() as size_t);
        // SAFETY: a live dictionary, a static key of the given length, and a
        // value the dictionary takes over.
        if unsafe { tv_dict_add_tv(d, name, len, &raw mut tv) } == FAIL {
            unref_dict(d);
            return ptr::null_mut::<dict_T>();
        }
    }
    d
}

/// A fresh dictionary with one reference held, which is how upstream's
/// `v:event` entries start.
fn new_dict() -> *mut dict_T {
    // SAFETY: `tv_dict_alloc` answers a fresh live dictionary.
    unsafe {
        let d = tv_dict_alloc();
        (*d).dv_refcount = 1;
        d
    }
}

/// Give up one reference to a dictionary.
fn unref_dict(d: *mut dict_T) {
    // SAFETY: a live dictionary this file holds a reference to.
    unsafe { tv_dict_unref(d) };
}

/// Give up the caller's reference once the dictionary has an owner.
fn hand_over(d: *mut dict_T) {
    // SAFETY: as [`unref_dict`]; the new owner holds the other reference.
    unsafe { (*d).dv_refcount -= 1 };
}

/// What [`scan_windows`] is collecting on this pass: the counts and first
/// windows `may_trigger_win_scrolled_resized` needs, the window list
/// `WinResized` reports, or the per-window dictionary `WinScrolled` does.
enum Scan<'a> {
    /// The first pass: how many windows changed size, and the first window of
    /// each kind.
    Counts {
        size_count: &'a mut c_int,
        first_scroll: &'a mut Option<Win>,
        first_size: &'a mut Option<Win>,
    },
    /// A list of the handles of every window that changed size.
    Winlist(*mut list_T),
    /// A dictionary of per-window deltas, plus an `all` entry.
    Deltas(*mut dict_T),
}

/// Look for windows whose size or view has moved since the last snapshot, and
/// record them the way `what` asks for.
fn scan_windows(what: &mut Scan) {
    let mut tot = [0 as c_int; 6];
    for mut wp in windows() {
        if wp.w_floating && wp.w_last_topline == 0 as linenr_T {
            // A just-created float has no previous size to compare with.
            snapshot_window(&mut wp);
            continue;
        }
        // SAFETY: the window's own 'eventignorewin' string.
        let eiw = wp.w_onebuf_opt.wo_eiw;
        let ignore_scroll = unsafe { event_ignored(EVENT_WINSCROLLED, eiw) };
        let ignore_resize = unsafe { event_ignored(EVENT_WINRESIZED, eiw) };
        let size_changed =
            !ignore_resize && (wp.w_last_width != wp.w_width || wp.w_last_height != wp.w_height);
        if size_changed {
            match what {
                Scan::Winlist(list) => {
                    let tv = typval_T {
                        v_type: VAR_NUMBER,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union {
                            v_number: wp.handle as varnumber_T,
                        },
                    };
                    // SAFETY: a live list, which takes ownership of `tv`.
                    unsafe { tv_list_append_owned_tv(*list, tv) };
                }
                Scan::Counts {
                    size_count,
                    first_scroll,
                    first_size,
                } => {
                    **size_count += 1;
                    first_size.get_or_insert(wp);
                    if !ignore_scroll {
                        first_scroll.get_or_insert(wp);
                    }
                }
                Scan::Deltas(_) => {}
            }
        }
        let scroll_changed = !ignore_scroll
            && (wp.w_last_topline != wp.w_topline
                || wp.w_last_topfill != wp.w_topfill
                || wp.w_last_leftcol != wp.w_leftcol
                || wp.w_last_skipcol != wp.w_skipcol);
        if scroll_changed && let Scan::Counts { first_scroll, .. } = what {
            first_scroll.get_or_insert(wp);
        }
        let Scan::Deltas(v_event) = what else {
            continue;
        };
        if !size_changed && !scroll_changed {
            continue;
        }
        let deltas = [
            wp.w_width - wp.w_last_width,
            wp.w_height - wp.w_last_height,
            wp.w_topline as c_int - wp.w_last_topline as c_int,
            wp.w_topfill - wp.w_last_topfill,
            wp.w_leftcol as c_int - wp.w_last_leftcol as c_int,
            wp.w_skipcol as c_int - wp.w_last_skipcol as c_int,
        ];
        let d = win_info_dict(deltas);
        if d.is_null() {
            break;
        }
        let mut winid = [0 as c_char; NUMBUFLEN as usize];
        let name = (&raw mut winid).cast::<c_char>();
        // SAFETY: `winid` is 65 bytes, which holds any window handle.
        let key_len =
            unsafe { vim_snprintf(name, size_of::<[c_char; 65]>(), c"%d".as_ptr(), wp.handle) };
        // SAFETY: a live dictionary, and a live dictionary to add to it.
        if unsafe { tv_dict_add_dict(*v_event, name, key_len as size_t, d) } == FAIL {
            unref_dict(d);
            break;
        }
        hand_over(d);
        for (total, delta) in tot.iter_mut().zip(deltas) {
            *total += delta.abs();
        }
    }
    let Scan::Deltas(v_event) = what else {
        return;
    };
    let alldict = win_info_dict(tot);
    if alldict.is_null() {
        return;
    }
    let (key, len) = (c"all".as_ptr(), 3 as size_t);
    // SAFETY: two live dictionaries.
    if unsafe { tv_dict_add_dict(*v_event, key, len, alldict) } == FAIL {
        unref_dict(alldict);
    } else {
        hand_over(alldict);
    }
}

/// The window whose id and buffer an event is reported against.
struct Subject {
    winid: [c_char; NUMBUFLEN as usize],
    bufref: BufRef,
}

impl Subject {
    fn of(win: Win) -> Self {
        let mut subject = Subject {
            winid: [0; NUMBUFLEN as usize],
            bufref: BufRef::of_opt(win.buffer_or_none()),
        };
        let name = (&raw mut subject.winid).cast::<c_char>();
        // SAFETY: a 65-byte buffer, which holds any window handle.
        unsafe { vim_snprintf(name, size_of::<[c_char; 65]>(), c"%d".as_ptr(), win.handle) };
        subject
    }

    fn name(&mut self) -> *mut c_char {
        (&raw mut self.winid).cast::<c_char>()
    }

    /// The buffer to fire the event for: the window's own if it is still
    /// there, the current one otherwise.
    fn buffer(&mut self) -> *mut buf_T {
        if self.bufref.valid() {
            self.bufref.raw()
        } else {
            curbuf.get()
        }
    }
}

pub unsafe fn may_trigger_win_scrolled_resized() {
    static recursive: GlobalCell<bool> = GlobalCell::new(false);
    // SAFETY: reads the autocommand tables.
    let (do_resize, do_scroll) = (has_event(EVENT_WINRESIZED), has_event(EVENT_WINSCROLLED));
    if recursive.get() || !(do_scroll || do_resize) || !did_initial_scroll_size_snapshot.get() {
        return;
    }

    let mut size_count = 0;
    let mut first_scroll = None;
    let mut first_size = None;
    scan_windows(&mut Scan::Counts {
        size_count: &mut size_count,
        first_scroll: &mut first_scroll,
        first_size: &mut first_size,
    });
    let trigger_resize = do_resize && size_count > 0;
    let trigger_scroll = do_scroll && first_scroll.is_some();
    if !trigger_resize && !trigger_scroll {
        return;
    }

    let mut windows_list = ptr::null_mut::<list_T>();
    if trigger_resize {
        // SAFETY: a fresh list of the right size.
        windows_list = unsafe { tv_list_alloc(size_count as ptrdiff_t) };
        scan_windows(&mut Scan::Winlist(windows_list));
    }
    let mut scroll_dict = ptr::null_mut::<dict_T>();
    if trigger_scroll {
        scroll_dict = new_dict();
        scan_windows(&mut Scan::Deltas(scroll_dict));
    }

    // Both events use the same snapshot, so take the new one before either
    // fires.
    snapshot_windows_scroll_size();
    recursive.set(true);

    let mut resize = first_size.map(Subject::of);
    let mut scroll = first_scroll.map(Subject::of);
    if let Some(resize) = resize.as_mut().filter(|_| trigger_resize) {
        fire_resized(resize, windows_list);
    }
    if let Some(scroll) = scroll.as_mut().filter(|_| trigger_scroll) {
        fire_scrolled(scroll, scroll_dict);
    }
    recursive.set(false);
}

/// Fire `WinResized` with `v:event.windows` set to the resized windows.
fn fire_resized(resize: &mut Subject, windows_list: *mut list_T) {
    let mut save = save_v_event_T::default();
    // SAFETY: `get_v_event` hands back the dictionary it saved into `save`.
    let v_event = unsafe { get_v_event(&raw mut save) };
    let (key, len) = (c"windows".as_ptr(), 7 as size_t);
    // SAFETY: a live dictionary, a static key, and a live list it takes over.
    if unsafe { tv_dict_add_list(v_event, key, len, windows_list) } == OK {
        let (name, buf) = (resize.name(), resize.buffer());
        // SAFETY: a live dictionary, a NUL-terminated name and a live buffer.
        unsafe { tv_dict_set_keys_readonly(v_event) };
        // SAFETY: as above; this fires user autocommands.
        unsafe { apply_autocmds(EVENT_WINRESIZED, name, name, false, buf) };
    }
    // SAFETY: the dictionary `get_v_event` saved into `save`.
    unsafe { restore_v_event(v_event, &raw mut save) };
}

/// Fire `WinScrolled` with `v:event` holding the per-window deltas.
fn fire_scrolled(scroll: &mut Subject, scroll_dict: *mut dict_T) {
    let mut save = save_v_event_T::default();
    // SAFETY: as [`fire_resized`]; `scroll_dict` is live and is unreferenced
    // once its contents have been copied in.
    let v_event = unsafe { get_v_event(&raw mut save) };
    // SAFETY: two live dictionaries and a static key.
    unsafe { tv_dict_extend(v_event, scroll_dict, c"move".as_ptr()) };
    // SAFETY: a live dictionary.
    unsafe { tv_dict_set_keys_readonly(v_event) };
    unref_dict(scroll_dict);
    let (name, buf) = (scroll.name(), scroll.buffer());
    // SAFETY: a NUL-terminated name and a live buffer; fires autocommands.
    unsafe { apply_autocmds(EVENT_WINSCROLLED, name, name, false, buf) };
    // SAFETY: the dictionary `get_v_event` saved into `save`.
    unsafe { restore_v_event(v_event, &raw mut save) };
}

// ---------------------------------------------------------------------------
// Saving and restoring every window's size

pub unsafe fn win_size_save(gap: *mut garray_T) {
    let room = windows().count() as c_int * 2 + 1;
    // SAFETY: the caller's promise -- a growable array to initialise.
    unsafe { ga_init(gap, size_of::<c_int>() as c_int, 1) };
    // SAFETY: as above.
    unsafe { ga_grow(gap, room) };
    // The total number of rows first, so a restore can tell the screen did not
    // change size in between.
    let mut sizes = Sizes(gap);
    sizes.push(frame_rows() + global_stl_rows() - last_stl_rows(false));
    for wp in windows() {
        sizes.push(wp.w_width + wp.w_vsep_width);
        sizes.push(wp.w_height);
    }
}

pub fn win_size_restore(gap: *mut garray_T) {
    let sizes = Sizes(gap);
    if windows().count() as c_int * 2 + 1 != sizes.len()
        || sizes.at(0) as OptInt
            != (frame_rows() + global_stl_rows() - last_stl_rows(false)) as OptInt
    {
        return;
    }
    // Do this twice to handle some window layouts properly.
    for _ in 0..2 {
        let mut i = 1;
        for wp in windows() {
            let (width, height) = (sizes.at(i), sizes.at(i + 1));
            i += 2;
            if !wp.w_floating {
                set_frame_width(wp.frame(), width);
                setheight_win(height, wp);
            }
        }
    }
    comp_positions();
}

/// `gap`, read as the array of `int`s `win_size_save` fills it with.
struct Sizes(*mut garray_T);

impl Sizes {
    fn len(&self) -> c_int {
        // SAFETY: a live growable array.
        unsafe { (*self.0).ga_len }
    }

    fn at(&self, i: c_int) -> c_int {
        // SAFETY: a live array of `int`, and `i` is inside its length.
        unsafe { *(*self.0).ga_data.cast::<c_int>().offset(i as isize) }
    }

    /// Append one size, which `ga_grow` has already made room for.
    fn push(&mut self, value: c_int) {
        let len = self.len();
        // SAFETY: as above; the caller grew the array to fit every push.
        unsafe { *(*self.0).ga_data.cast::<c_int>().offset(len as isize) = value };
        // SAFETY: as above.
        unsafe { (*self.0).ga_len = len + 1 };
    }
}
