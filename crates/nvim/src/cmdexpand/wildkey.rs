//! The wildmenu's own key handling.
//!
//! While the wildmenu is up, some keys mean "move in the menu" rather than
//! what they usually mean.  [`wildmenu_translate_key`] does the remapping and
//! [`wildmenu_process_key`] applies it, with a different rule for menu names
//! than for file names.

#![forbid(unsafe_code)]

use super::*;
use crate::cmdexpand::state::{save_p_ls, save_p_wmh, wild_menu_showing};
use crate::drawscreen::state::cmdline_row;
use crate::drawscreen::{redraw_statuslines, update_screen, win_redraw_last_status};
use crate::ex_docmd::set_no_hlsearch;
use crate::ex_getln::{put_on_cmdline_bytes, redrawcmd};
use crate::getchar::state::KeyTyped;
use crate::guard::Allow;
use crate::keycodes::{Ctrl_N, Ctrl_P, Key};
use crate::mbyte::head_off;
use crate::option::vars::{P_LS, P_WMH, p_wc, p_wmnu};
use crate::path::{PATHSEP, vim_ispathsep};
use crate::types::{ExpandContext, OptInt};
use crate::window::last_status;
use crate::winlayer::{Cc, current_topframe};
use core::ffi::c_int;

/// One directory level up, as it is spelled on the command line.
///
/// [`UPSEG_TAIL`] is what gets inserted; the four-byte form with the leading
/// separator is what an existing "../" step is recognised by.
const UPSEG: &[u8] = b"/../";
/// [`UPSEG`] without its leading separator.
const UPSEG_TAIL: &[u8] = b"../";
const _: () = assert!(
    PATHSEP == b'/' as c_int,
    "UPSEG hard-codes the path separator"
);

/// The byte at `i` of the command line, or NUL outside it.
fn line_byte(cclp: Cc, i: c_int) -> u8 {
    usize::try_from(i)
        .ok()
        .and_then(|i| cclp.text_bytes().get(i).copied())
        .unwrap_or(0)
}

/// Translate a key pressed while the wildmenu is up.
///
/// The horizontal arrows step through the matches, and `<CR>` after a menu
/// name ending in `.` opens the submenu rather than executing.
pub(crate) fn wildmenu_translate_key(
    cclp: Cc,
    key: c_int,
    expand: &Expand,
    did_wild_list: bool,
) -> c_int {
    let mut c = key;
    if cmdline_pum_active() || did_wild_list || wild_menu_showing.get() != 0 {
        if c == Key::Left.code() {
            c = Ctrl_P;
        } else if c == Key::Right.code() {
            c = Ctrl_N;
        }
    }

    // Hitting CR after "emenu Name.": complete the submenu.
    if expand.context == ExpandContext::Menunames
        && cclp.cmdpos > 1
        && line_byte(cclp, cclp.cmdpos - 1) == b'.'
        && line_byte(cclp, cclp.cmdpos - 2) != b'\\'
        && (c == c_int::from(b'\n') || c == c_int::from(b'\r') || c == Key::Kenter.code())
    {
        c = Key::Down.code();
    }
    c
}

/// Delete characters on the command line, from `from` to the current position.
fn cmdline_del(mut cclp: Cc, from: c_int) {
    debug_assert!(cclp.cmdpos <= cclp.len());
    let cursor = cclp.cmdpos;
    cclp.replace_range(from, cursor, &[], 0);
    cclp.cmdpos = from;
}

/// Ask for a fresh completion of what is now on the command line.
///
/// `KeyTyped` is set in case the key that got us here came from a mapping:
/// the wildchar has to look typed or the completion will not run.
fn recomplete() -> c_int {
    KeyTyped.set(true);
    p_wc() as c_int
}

/// A key pressed while the wildmenu for menu names (`ExpandContext::Menunames`) is up.
fn wildmenu_process_key_menunames(cclp: Cc, key: c_int, expand: &mut Expand) -> c_int {
    let at = |k: c_int| line_byte(cclp, k);
    if key == Key::Down.code() && cclp.cmdpos > 0 && at(cclp.cmdpos - 1) == b'.' {
        // Hitting <Down> after "emenu Name.": complete the submenu.
        return recomplete();
    }
    if key != Key::Up.code() {
        return key;
    }

    // Hitting <Up>: remove one submenu name in front of the cursor.  The
    // walk stops at the *second* unescaped '.', or at the first unescaped
    // space, which is where the menu name itself starts.
    let mut found = false;
    let mut i = 0;
    let mut j = as_count(expand.pattern);
    loop {
        j -= 1;
        if j <= 0 {
            break;
        }
        let unescaped = at(j - 1) != b'\\';
        if at(j) == b' ' && unescaped {
            i = j + 1; // start of the menu name
            break;
        }
        if at(j) == b'.' && unescaped {
            if found {
                i = j + 1; // start of a submenu name
                break;
            }
            found = true;
        }
    }
    if i > 0 {
        cmdline_del(cclp, i);
    }
    expand.context = ExpandContext::Nothing;
    recomplete()
}

/// A key pressed while the wildmenu for file, directory or shell-command
/// names is up.
///
/// `<Down>` descends into the directory under the cursor and `<Up>` leaves it,
/// both by editing the path on the command line and asking for a fresh
/// completion of the result.
fn wildmenu_process_key_filenames(cclp: Cc, key: c_int, expand: &Expand) -> c_int {
    let at = |k: c_int| line_byte(cclp, k);
    // Where the pattern being completed starts.
    let start = as_count(expand.pattern);
    // Step back from `j` to the start of the character before it.
    let back_to_head =
        |j: c_int| as_count(head_off(cclp.text_bytes(), usize::try_from(j).unwrap_or(0)));
    let starts_with_at = |j: c_int, what: &[u8]| {
        usize::try_from(j)
            .ok()
            .and_then(|j| cclp.text_bytes().get(j..))
            .is_some_and(|rest| rest.starts_with(what))
    };

    if key == Key::Down.code()
        && cclp.cmdpos > 0
        && at(cclp.cmdpos - 1) == PATHSEP as u8
        && (cclp.cmdpos < 3 || at(cclp.cmdpos - 2) != b'.' || at(cclp.cmdpos - 3) != b'.')
    {
        // Go down a directory.
        return recomplete();
    }

    // The command line as it stands, with the match inserted, is what the
    // pattern's offset is read against: upstream's `xp_pattern` followed
    // the insertion.
    if key == Key::Down.code() && starts_with_at(start, UPSEG_TAIL) {
        // In a direct ancestor: strip off one "../" to go down.  Walk
        // back to the separator that ends the "..".
        let mut found = false;
        let mut j = cclp.cmdpos;
        loop {
            j -= 1;
            if j <= start {
                break;
            }
            j -= back_to_head(j);
            if vim_ispathsep(c_int::from(at(j))) {
                found = true;
                break;
            }
        }
        if found
            && at(j - 1) == b'.'
            && at(j - 2) == b'.'
            && (vim_ispathsep(c_int::from(at(j - 3))) || j == start + 2)
        {
            cmdline_del(cclp, j - 2);
            return recomplete();
        }
        return key;
    }

    if key != Key::Up.code() {
        return key;
    }

    // Go up a directory: walk back to the *second* separator, so that
    // what is deleted is the whole trailing path component.
    let mut found = false;
    let mut i = start;
    let mut j = cclp.cmdpos - 1;
    loop {
        j -= 1;
        if j <= i {
            break;
        }
        j -= back_to_head(j);
        if vim_ispathsep(c_int::from(at(j))) {
            if found {
                i = j + 1;
                break;
            }
            found = true;
        }
    }

    if !found {
        j = i;
    } else if starts_with_at(j, UPSEG) {
        j += 4; // already "/../": step over it
    } else if starts_with_at(j, UPSEG_TAIL) && j == i {
        j += 3; // the pattern itself starts "../"
    } else {
        j = 0;
    }

    if j > 0 {
        // TODO(tarruda): this is only for DOS/Unix systems - need to put
        // in machine-specific stuff here and in UPSEG.
        cmdline_del(cclp, j);
        put_on_cmdline_bytes(UPSEG_TAIL, false);
    } else if cclp.cmdpos > i {
        cmdline_del(cclp, i);
    }

    // Now complete in the new directory.
    recomplete()
}

/// Handle a key pressed while the wildmenu is displayed.
pub(crate) fn wildmenu_process_key(cclp: Cc, key: c_int, expand: &mut Expand) -> c_int {
    // Special translations for 'wildmenu'.
    match expand.context {
        ExpandContext::Menunames => wildmenu_process_key_menunames(cclp, key, expand),
        ExpandContext::Files | ExpandContext::Directories | ExpandContext::ShellCmd => {
            wildmenu_process_key_filenames(cclp, key, expand)
        }
        _ => key,
    }
}

/// Take the wildmenu down again once the walk through the matches is over.
///
/// Which of the three ways it went up decides how it comes down: it either
/// scrolled the command line, borrowed the status line by forcing
/// `'laststatus'`, or drew over the last window's existing status line.
pub(crate) fn wildmenu_cleanup(cclp: Cc) {
    if !p_wmnu() || wild_menu_showing.get() == 0 {
        return;
    }

    let skt = KeyTyped.get();
    let redraw = (cclp.input_fn != 0).then(Allow::redraw);

    // Clear highlighting applied during wildmenu activity.
    set_no_hlsearch(true);

    if wild_menu_showing.get() == WM_SCROLLED {
        // Entered the command line, move it up.
        cmdline_row.set(cmdline_row.get() - 1);
        redrawcmd();
    } else if save_p_ls.get() != -1 {
        // Restore 'laststatus' and 'winminheight'.
        P_LS.set(save_p_ls.get() as OptInt);
        P_WMH.set(save_p_wmh.get() as OptInt);
        last_status(false);
        let _ = update_screen(); // redraw the screen NOW
        redrawcmd();
        save_p_ls.set(-1);
    } else {
        win_redraw_last_status(current_topframe());
        // Must be cleared before redraw_statuslines (#8385), which is why
        // this arm clears it itself rather than after the `if`.
        wild_menu_showing.set(0);
        redraw_statuslines();
    }
    wild_menu_showing.set(0);

    KeyTyped.set(skt);
    drop(redraw);
}
