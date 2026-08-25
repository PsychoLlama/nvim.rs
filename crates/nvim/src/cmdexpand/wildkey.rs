//! The wildmenu's own key handling.
//!
//! While the wildmenu is up, some keys mean "move in the menu" rather than
//! what they usually mean.  [`wildmenu_translate_key`] does the remapping and
//! [`wildmenu_process_key`] applies it, with a different rule for menu names
//! than for file names.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::guard::Allow;
use crate::keycodes::{Ctrl_N, Ctrl_P};
use crate::types::ExpandContext;
use core::ffi::{c_char, c_int};

/// One directory level up, as it is spelled on the command line.
///
/// [`UPSEG_TAIL`] is what gets inserted; the four-byte form with the leading
/// separator is what an existing "../" step is recognised by.
const UPSEG: &CStr = c"/../";
/// [`UPSEG`] without its leading separator.
const UPSEG_TAIL: &CStr = c"../";
const _: () = assert!(
    PATHSEP == b'/' as c_int,
    "UPSEG hard-codes the path separator"
);

/// Translate a key pressed while the wildmenu is up.
///
/// The horizontal arrows step through the matches, and `<CR>` after a menu
/// name ending in `.` opens the submenu rather than executing.
pub(crate) unsafe fn wildmenu_translate_key(
    cclp: Cc,
    key: c_int,
    xp: *mut expand_T,
    did_wild_list: bool,
) -> c_int {
    unsafe {
        let mut c = key;
        if cmdline_pum_active() || did_wild_list || wild_menu_showing.get() != 0 {
            if c == K_LEFT {
                c = Ctrl_P;
            } else if c == K_RIGHT {
                c = Ctrl_N;
            }
        }

        // Hitting CR after "emenu Name.": complete the submenu.
        if (*xp).xp_context == ExpandContext::Menunames
            && cclp.cmdpos > 1
            && *cclp.at(cclp.cmdpos - 1) == b'.' as c_char
            && *cclp.at(cclp.cmdpos - 2) != b'\\' as c_char
            && (c == '\n' as c_int || c == '\r' as c_int || c == K_KENTER)
        {
            c = K_DOWN;
        }
        c
    }
}

/// Delete characters on the command line, from `from` to the current position.
unsafe fn cmdline_del(mut cclp: Cc, from: c_int) {
    unsafe {
        debug_assert!(cclp.cmdpos <= cclp.len());
        // +1 for the NUL.
        core::ptr::copy(
            cclp.text().offset(cclp.cmdpos as isize),
            cclp.at(from),
            (cclp.len() - cclp.cmdpos + 1) as size_t,
        );
        cclp.set_len(cclp.len() - (cclp.cmdpos - from));
        cclp.cmdpos = from;
    }
}

/// Ask for a fresh completion of what is now on the command line.
///
/// `KeyTyped` is set in case the key that got us here came from a mapping:
/// the wildchar has to look typed or the completion will not run.
fn recomplete() -> c_int {
    KeyTyped.set(true);
    p_wc.get() as c_int
}

/// A key pressed while the wildmenu for menu names (`ExpandContext::Menunames`) is up.
unsafe fn wildmenu_process_key_menunames(cclp: Cc, key: c_int, xp: *mut expand_T) -> c_int {
    unsafe {
        let buf = cclp.text();
        if key == K_DOWN
            && cclp.cmdpos > 0
            && *buf.offset((cclp.cmdpos - 1) as isize) == b'.' as c_char
        {
            // Hitting <Down> after "emenu Name.": complete the submenu.
            return recomplete();
        }
        if key != K_UP {
            return key;
        }

        // Hitting <Up>: remove one submenu name in front of the cursor.  The
        // walk stops at the *second* unescaped '.', or at the first unescaped
        // space, which is where the menu name itself starts.
        let mut found = false;
        let mut i = 0;
        let mut j = (*xp).xp_pattern.offset_from(buf) as c_int;
        loop {
            j -= 1;
            if j <= 0 {
                break;
            }
            let unescaped = *buf.offset((j - 1) as isize) != b'\\' as c_char;
            if *buf.offset(j as isize) == b' ' as c_char && unescaped {
                i = j + 1; // start of the menu name
                break;
            }
            if *buf.offset(j as isize) == b'.' as c_char && unescaped {
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
        (*xp).xp_context = ExpandContext::Nothing;
        recomplete()
    }
}

/// A key pressed while the wildmenu for file, directory or shell-command
/// names is up.
///
/// `<Down>` descends into the directory under the cursor and `<Up>` leaves it,
/// both by editing the path on the command line and asking for a fresh
/// completion of the result.
unsafe fn wildmenu_process_key_filenames(cclp: Cc, key: c_int, xp: *mut expand_T) -> c_int {
    unsafe {
        let buf = cclp.text();
        let at = |k: c_int| *buf.offset(k as isize);
        // Where the pattern being completed starts.
        let start = (*xp).xp_pattern.offset_from(buf) as c_int;

        if key == K_DOWN
            && cclp.cmdpos > 0
            && at(cclp.cmdpos - 1) == PATHSEP as c_char
            && (cclp.cmdpos < 3
                || at(cclp.cmdpos - 2) != b'.' as c_char
                || at(cclp.cmdpos - 3) != b'.' as c_char)
        {
            // Go down a directory.
            return recomplete();
        }

        if key == K_DOWN && strncmp((*xp).xp_pattern, UPSEG_TAIL.as_ptr(), 3) == 0 {
            // In a direct ancestor: strip off one "../" to go down.  Walk
            // back to the separator that ends the "..".
            let mut found = false;
            let mut j = cclp.cmdpos;
            loop {
                j -= 1;
                if j <= start {
                    break;
                }
                j -= utf_head_off(buf, buf.offset(j as isize));
                if vim_ispathsep(at(j) as c_int) {
                    found = true;
                    break;
                }
            }
            if found
                && at(j - 1) == b'.' as c_char
                && at(j - 2) == b'.' as c_char
                && (vim_ispathsep(at(j - 3) as c_int) || j == start + 2)
            {
                cmdline_del(cclp, j - 2);
                return recomplete();
            }
            return key;
        }

        if key != K_UP {
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
            j -= utf_head_off(buf, buf.offset(j as isize));
            if vim_ispathsep(at(j) as c_int) {
                if found {
                    i = j + 1;
                    break;
                }
                found = true;
            }
        }

        if !found {
            j = i;
        } else if strncmp(buf.offset(j as isize), UPSEG.as_ptr(), 4) == 0 {
            j += 4; // already "/../": step over it
        } else if strncmp(buf.offset(j as isize), UPSEG_TAIL.as_ptr(), 3) == 0 && j == i {
            j += 3; // the pattern itself starts "../"
        } else {
            j = 0;
        }

        if j > 0 {
            // TODO(tarruda): this is only for DOS/Unix systems - need to put
            // in machine-specific stuff here and in UPSEG.
            cmdline_del(cclp, j);
            put_on_cmdline(UPSEG_TAIL.as_ptr().cast_mut(), 3, false);
        } else if cclp.cmdpos > i {
            cmdline_del(cclp, i);
        }

        // Now complete in the new directory.
        recomplete()
    }
}

/// Handle a key pressed while the wildmenu is displayed.
pub(crate) unsafe fn wildmenu_process_key(cclp: Cc, key: c_int, xp: *mut expand_T) -> c_int {
    unsafe {
        // Special translations for 'wildmenu'.
        match (*xp).xp_context {
            ExpandContext::Menunames => wildmenu_process_key_menunames(cclp, key, xp),
            ExpandContext::Files | ExpandContext::Directories | ExpandContext::ShellCmd => {
                wildmenu_process_key_filenames(cclp, key, xp)
            }
            _ => key,
        }
    }
}

/// Take the wildmenu down again once the walk through the matches is over.
///
/// Which of the three ways it went up decides how it comes down: it either
/// scrolled the command line, borrowed the status line by forcing
/// `'laststatus'`, or drew over the last window's existing status line.
pub(crate) unsafe fn wildmenu_cleanup(cclp: Cc) {
    unsafe {
        if p_wmnu.get() == 0 || wild_menu_showing.get() == 0 {
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
            p_ls.set(save_p_ls.get() as OptInt);
            p_wmh.set(save_p_wmh.get() as OptInt);
            last_status(false);
            update_screen(); // redraw the screen NOW
            redrawcmd();
            save_p_ls.set(-1);
        } else {
            win_redraw_last_status(topframe.get());
            // Must be cleared before redraw_statuslines (#8385), which is why
            // this arm clears it itself rather than after the `if`.
            wild_menu_showing.set(0);
            redraw_statuslines();
        }
        wild_menu_showing.set(0);

        KeyTyped.set(skt);
        drop(redraw);
    }
}
