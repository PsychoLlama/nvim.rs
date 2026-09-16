//! The `[` and `]` commands: the block, comment, define and method
//! searches, the mark and fold jumps, and the paste variants.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::keycodes::Key;
use crate::strings::has_char;
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ptr;

use crate::cursor::{dec_cursor, gchar_cursor, inc_cursor};
use crate::diff::diff_move_to;
use crate::edit::{BeginlineOpts, beginline};
use crate::fold::fold_move_to;
use crate::mark::{getnextmark, pos_to_mark, setpcmark};
use crate::memory::{xfree, xmemdupz};
use crate::mouse::do_mouse;
use crate::normal::{
    _ISlower, _ISupper, ACTION_GOTO, ACTION_SHOW, ACTION_SHOW_ALL, FIND_ANY, FIND_DEFINE,
    FIND_IDENT, FM_BACKWARD, FM_FORWARD, SMT_BAD, SMT_RARE, clear_op, clear_op_beep,
    find_ident_under_cursor, kDirectionNotSet, kMTCharWise, kMarkBeginLine, kMarkContext,
    may_fold_open, nv_gotofile, nv_mark_move_to, nv_put_opt,
};
use crate::options::{kOptFdoFlagBlock, kOptFdoFlagSearch};
use crate::os::cshim::__ctype_b_loc;
use crate::pos::MAXLNUM;
use crate::search::{BACKWARD, FORWARD, find_pattern_in_path, findmatchlimit};
use crate::spell::{SMT_ALL, spell_move_to};
use crate::textobject::findpar;
use crate::types::{CmdArg, FileMark, MarkMove, OpType, PUT_FIXINDENT, Pos, SpellMoveType};
use core::ffi::{CStr, c_char, c_int, c_uint, c_ushort, c_void};

/// Which way a `[` or `]` command searches.
fn direction(cmd_arg: &mut CmdArg) -> c_int {
    if cmd_arg.cmdchar == ']' as c_int {
        FORWARD as c_int
    } else {
        BACKWARD as c_int
    }
}

/// The same choice spelled in `findmatchlimit`'s own flags, which are not the
/// `Direction` constants.
fn match_direction(cmd_arg: &mut CmdArg) -> c_int {
    if cmd_arg.cmdchar == '[' as c_int {
        FM_BACKWARD as c_int
    } else {
        FM_FORWARD as c_int
    }
}

/// `[{`, `]}`, `[(`, `])`, `[*`, `]/`, `[#`, `[m`, `]M` and friends: jump to
/// an unmatched bracket, or to the start or end of a method.
///
/// The `m`/`M` forms run in two passes. The first walks *out* through as many
/// enclosing `{}` as it can (up to 9,999), which finds the outermost block the
/// cursor is inside; the second walks back in from there, counting the braces
/// the count asked for. `prev_pos` carries the second-outermost block between
/// the two, which is what makes `2[m` mean "the method one level out".
///
/// # Safety
///
/// `old_pos` must point at an initialized position.
unsafe fn nv_bracket_block(cmd_arg: &mut CmdArg, old_pos: *const Pos) {
    // SAFETY: `old_pos` is the cursor position the caller saved.
    let mut new_pos = Pos {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut prev_pos = Pos {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut pos: Option<Pos> = None;

    // `[*` and `]*` are spelled `[/` and `]/` to findmatchlimit.
    if cmd_arg.nchar == '*' as c_int {
        cmd_arg.nchar = '/' as c_int;
    }
    let method = cmd_arg.nchar == 'm' as c_int || cmd_arg.nchar == 'M' as c_int;
    let findc = if method {
        if cmd_arg.cmdchar == '[' as c_int {
            '{' as c_int
        } else {
            '}' as c_int
        }
    } else {
        cmd_arg.nchar
    };
    let mut n = if method { 9999 } else { cmd_arg.count1 };

    while n > 0 {
        pos = unsafe { findmatchlimit(cmd_arg.oap, findc, match_direction(cmd_arg), 0) };
        let Some(found) = pos else {
            if new_pos.lnum == 0 {
                // Nothing found at all. A method search says so by leaving
                // `pos` empty for the second pass to notice.
                if !method {
                    clear_op_beep(cmd_arg.op());
                }
            } else {
                // Ran out of enclosing blocks: the last one found is it.
                pos = Some(new_pos);
            }
            break;
        };
        prev_pos = new_pos;
        Win::current().w_cursor = found;
        new_pos = found;
        n -= 1;
    }
    Win::current().w_cursor = unsafe { *old_pos };

    if method {
        // `[m` and `]M` want the brace itself; `[M` and `]m` want the one
        // before it. `norm` is true for the first pair.
        let norm = (findc == '{' as c_int) == (cmd_arg.nchar == 'm' as c_int);
        n = cmd_arg.count1;
        if prev_pos.lnum != 0 {
            pos = Some(prev_pos);
            Win::current().w_cursor = prev_pos;
            if norm {
                n -= 1;
            }
        } else {
            pos = None;
        }
        while n > 0 {
            loop {
                let stepped = if findc == '{' as c_int {
                    dec_cursor()
                } else {
                    inc_cursor()
                };
                if stepped < 0 {
                    // Hit the end of the buffer with nothing found.
                    if pos.is_none() {
                        clear_op_beep(cmd_arg.op());
                    }
                    n = 0;
                    break;
                }
                let c = gchar_cursor();
                if c != '{' as c_int && c != '}' as c_int {
                    continue;
                }
                if (c == findc && norm) || (n == 1 && !norm) {
                    new_pos = Win::current().w_cursor;
                    pos = Some(new_pos);
                    n = 0;
                } else if new_pos.lnum == 0 {
                    new_pos = Win::current().w_cursor;
                    pos = Some(new_pos);
                } else {
                    // A brace of the other kind: step over the block it
                    // opens or closes.
                    pos =
                        unsafe { findmatchlimit(cmd_arg.oap, findc, match_direction(cmd_arg), 0) };
                    match pos {
                        None => n = 0,
                        Some(found) => Win::current().w_cursor = found,
                    }
                }
                break;
            }
            n -= 1;
        }
        Win::current().w_cursor = unsafe { *old_pos };
        // A position was found on the way out but lost on the way back in.
        if pos.is_none() && new_pos.lnum != 0 {
            clear_op_beep(cmd_arg.op());
        }
    }

    if let Some(pos) = pos {
        setpcmark();
        Win::current().w_cursor = pos;
        Win::current().w_set_curswant = true;
        may_fold_open(cmd_arg, kOptFdoFlagBlock as c_uint);
    }
}

/// Look an identifier under the cursor up in the included files.
///
/// The case of the second character picks the action: an upper-case one lists
/// every match, a lower-case one lists the first, and a control character
/// jumps to it. `d`-family keys (`d`, `D`, CTRL-D) search for a `#define`
/// rather than for any occurrence, which is what the low-nibble comparison
/// tests -- CTRL-D, `d` and `D` all end in the same four bits.
fn nv_bracket_ident(cmd_arg: &mut CmdArg) {
    let mut found: *mut c_char = ptr::null_mut();
    let len =
        unsafe { find_ident_under_cursor(&raw mut found, FIND_IDENT as c_int, ptr::null_mut()) };
    if len == 0 {
        clear_op(cmd_arg.op());
        return;
    }
    let nchar = cmd_arg.nchar;
    let ctype = unsafe { *(*__ctype_b_loc()).offset(nchar as isize) } as c_int;
    let is_upper = ctype & _ISupper as c_ushort as c_int != 0;
    let is_lower = ctype & _ISlower as c_ushort as c_int != 0;
    // `find_pattern_in_path` keeps the name, so hand it a copy.
    let name = unsafe { xmemdupz(found as *const c_void, len) } as *mut c_char;
    // Without a count, a lower-case key searches case-insensitively.
    let fold_case = if cmd_arg.count0 == 0 {
        !is_upper
    } else {
        false
    };
    let what = if nchar & 0xf == 'd' as c_int & 0xf {
        FIND_DEFINE as c_int
    } else {
        FIND_ANY as c_int
    };
    let action = if is_upper {
        ACTION_SHOW_ALL as c_int
    } else if is_lower {
        ACTION_SHOW as c_int
    } else {
        ACTION_GOTO as c_int
    };
    // `]` starts below the cursor line, `[` at the top of the file.
    let from = if cmd_arg.cmdchar == ']' as c_int {
        Win::current().w_cursor.lnum + 1
    } else {
        1
    };
    let (dir, n) = (kDirectionNotSet, cmd_arg.count1);
    let last = MAXLNUM;
    // SAFETY: `name` is the NUL-terminated identifier copied above.
    unsafe {
        find_pattern_in_path(
            name, dir, len, true, fold_case, what, n, action, from, last, false, false,
        )
    };
    unsafe { xfree(name as *mut c_void) };
    Win::current().w_set_curswant = true;
}

/// `['`, `` [` ``, `]'` and `` ]` ``: jump to the next or previous lower-case
/// mark in this buffer.
fn nv_bracket_mark(cmd_arg: &mut CmdArg) {
    // The walk starts from a mark standing for the cursor itself, in this
    // frame's own record — every later `fm` is a store's address instead.
    let mut here = FileMark::UNSET;
    let mut fm = unsafe { pos_to_mark(Buf::current(), &raw mut here, Win::current().w_cursor) };
    debug_assert!(!fm.is_null());
    let linewise = cmd_arg.nchar == '\'' as c_int;
    let mut prev_fm = ptr::null_mut();
    let mut n = cmd_arg.count1;
    while n > 0 {
        prev_fm = fm;
        fm = unsafe { getnextmark(&raw mut (*fm).mark, direction(cmd_arg), linewise as c_int) };
        if fm.is_null() {
            break;
        }
        n -= 1;
    }
    // Running out of marks stops at the last one rather than failing.
    if fm.is_null() {
        fm = prev_fm;
    }
    let mut flags = kMarkContext as MarkMove;
    if linewise {
        flags |= kMarkBeginLine as MarkMove;
    }
    unsafe { nv_mark_move_to(cmd_arg, flags, fm) };
}

/// `[s`, `[r`, `[S`, `]s`, `]r` and `]S`: jump to a misspelled word.
fn nv_bracket_spell(cmd_arg: &mut CmdArg) {
    setpcmark();
    let what = match u8::try_from(cmd_arg.nchar) {
        Ok(b's') => SMT_ALL as SpellMoveType,
        Ok(b'r') => SMT_RARE as SpellMoveType,
        _ => SMT_BAD as SpellMoveType,
    };
    for _ in 0..cmd_arg.count1 {
        if unsafe {
            spell_move_to(
                Win::current(),
                direction(cmd_arg),
                what,
                false,
                ptr::null_mut(),
            )
        } == 0
        {
            clear_op_beep(cmd_arg.op());
            break;
        }
        Win::current().w_set_curswant = true;
    }
    may_fold_open(cmd_arg, kOptFdoFlagSearch as c_uint);
}

/// `[` and `]`, whose second character says what kind of jump this is.
pub(crate) fn nv_brackets(cmd_arg: &mut CmdArg) {
    cmd_arg.op().motion_type = kMTCharWise;
    cmd_arg.op().inclusive = false;
    let old_pos = Win::current().w_cursor;
    Win::current().w_cursor.coladd = 0;

    let nchar = cmd_arg.nchar;
    let opening = cmd_arg.cmdchar == '[' as c_int;
    // The bracket forms that name a block: `[{ [( [* [/ [# [m [M` and the
    // closing halves of each.
    let block_chars: &CStr = if opening { c"{(*/#mM" } else { c"})*/#mM" };

    if nchar == 'f' as c_int {
        nv_gotofile(cmd_arg);
    } else if has_char(c"iI\tdD\x04", nchar) {
        nv_bracket_ident(cmd_arg);
    } else if has_char(block_chars, nchar) {
        unsafe { nv_bracket_block(cmd_arg, &raw const old_pos) };
    } else if nchar == '[' as c_int || nchar == ']' as c_int {
        // `[[` and `]]` look for a section start, `[]` and `][` for its
        // end.
        let flag = if nchar == cmd_arg.cmdchar {
            '{' as c_int
        } else {
            '}' as c_int
        };
        Win::current().w_set_curswant = true;
        let both_ways = cmd_arg.op().op_type != OpType::Nop
            && cmd_arg.arg == FORWARD as c_int
            && flag == '{' as c_int;
        let (incl, dir, n) = (&raw mut cmd_arg.op().inclusive, cmd_arg.arg, cmd_arg.count1);
        if !unsafe { findpar(incl, dir, n, flag, both_ways) } {
            clear_op_beep(cmd_arg.op());
        } else {
            if cmd_arg.op().op_type == OpType::Nop {
                beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
            }
            may_fold_open(cmd_arg, kOptFdoFlagBlock as c_uint);
        }
    } else if nchar == 'p' as c_int || nchar == 'P' as c_int {
        // The put that reindents to the current line.
        nv_put_opt(cmd_arg, true);
    } else if nchar == '\'' as c_int || nchar == '`' as c_int {
        nv_bracket_mark(cmd_arg);
    } else if (Key::Rightrelease.code()..=Key::Leftmouse.code()).contains(&nchar) {
        // A mouse click after `[` or `]` pastes at the click, reindenting.
        let (dir, n) = (direction(cmd_arg), cmd_arg.count1);
        do_mouse(
            Some(cmd_arg.op()),
            nchar,
            dir,
            n,
            PUT_FIXINDENT as c_int != 0,
        );
    } else if nchar == 'z' as c_int {
        if fold_move_to(false, direction(cmd_arg), cmd_arg.count1) == 0 {
            clear_op_beep(cmd_arg.op());
        }
    } else if nchar == 'c' as c_int {
        if diff_move_to(direction(cmd_arg), cmd_arg.count1).is_err() {
            clear_op_beep(cmd_arg.op());
        }
    } else if nchar == 'r' as c_int || nchar == 's' as c_int || nchar == 'S' as c_int {
        nv_bracket_spell(cmd_arg);
    } else {
        clear_op_beep(cmd_arg.op());
    }
}
