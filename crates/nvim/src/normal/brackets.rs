//! The `[` and `]` commands: the block, comment, define and method
//! searches, the mark and fold jumps, and the paste variants.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ptr;

use crate::cursor::{dec_cursor, gchar_cursor, inc_cursor};
use crate::diff::diff_move_to;
use crate::edit::{BeginlineOpts, beginline};
use crate::fold::fold_move_to;
use crate::keycodes::{K_LEFTMOUSE, K_RIGHTRELEASE};
use crate::main::{curbuf, curwin};
use crate::mark::{getnextmark, pos_to_mark, setpcmark};
use crate::memory::{xfree, xmemdupz};
use crate::mouse::do_mouse;
use crate::normal::{
    _ISlower, _ISupper, ACTION_GOTO, ACTION_SHOW, ACTION_SHOW_ALL, FIND_ANY, FIND_DEFINE,
    FIND_IDENT, FM_BACKWARD, FM_FORWARD, SMT_BAD, SMT_RARE, clearop, clearopbeep,
    find_ident_under_cursor, kDirectionNotSet, kMTCharWise, kMarkBeginLine, kMarkContext,
    may_fold_open, nv_gotofile, nv_mark_move_to, nv_put_opt,
};
use crate::options::{kOptFdoFlagBlock, kOptFdoFlagSearch};
use crate::os::cshim::__ctype_b_loc;
use crate::pos::MAXLNUM;
use crate::search::{BACKWARD, FORWARD, find_pattern_in_path, findmatchlimit};
use crate::spell::{SMT_ALL, spell_move_to};
use crate::strings::vim_strchr;
use crate::textobject::findpar;
use crate::types::{MarkMove, OP_NOP, PUT_FIXINDENT, cmdarg_T, linenr_T, pos_T, smt_T};
use core::ffi::{CStr, c_char, c_int, c_uint, c_ushort, c_void};

/// Which way a `[` or `]` command searches.
unsafe fn direction(cap: *mut cmdarg_T) -> c_int {
    // SAFETY: `cap` is the caller's live command argument.
    if unsafe { (*cap).cmdchar } == ']' as c_int {
        FORWARD as c_int
    } else {
        BACKWARD as c_int
    }
}

/// The same choice spelled in `findmatchlimit`'s own flags, which are not the
/// `Direction` constants.
unsafe fn match_direction(cap: *mut cmdarg_T) -> c_int {
    // SAFETY: `cap` is the caller's live command argument.
    if unsafe { (*cap).cmdchar } == '[' as c_int {
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
unsafe fn nv_bracket_block(cap: *mut cmdarg_T, old_pos: *const pos_T) {
    // SAFETY: `cap` is the caller's live command argument and `old_pos` is the
    // cursor position its caller saved.
    unsafe {
        let mut new_pos = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut prev_pos = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut pos: Option<pos_T> = None;

        // `[*` and `]*` are spelled `[/` and `]/` to findmatchlimit.
        if (*cap).nchar == '*' as c_int {
            (*cap).nchar = '/' as c_int;
        }
        let method = (*cap).nchar == 'm' as c_int || (*cap).nchar == 'M' as c_int;
        let findc = if method {
            if (*cap).cmdchar == '[' as c_int {
                '{' as c_int
            } else {
                '}' as c_int
            }
        } else {
            (*cap).nchar
        };
        let mut n = if method { 9999 } else { (*cap).count1 };

        while n > 0 {
            pos = findmatchlimit((*cap).oap, findc, match_direction(cap), 0);
            let Some(found) = pos else {
                if new_pos.lnum == 0 {
                    // Nothing found at all. A method search says so by leaving
                    // `pos` empty for the second pass to notice.
                    if !method {
                        clearopbeep((*cap).oap);
                    }
                } else {
                    // Ran out of enclosing blocks: the last one found is it.
                    pos = Some(new_pos);
                }
                break;
            };
            prev_pos = new_pos;
            (*curwin.get()).w_cursor = found;
            new_pos = found;
            n -= 1;
        }
        (*curwin.get()).w_cursor = *old_pos;

        if method {
            // `[m` and `]M` want the brace itself; `[M` and `]m` want the one
            // before it. `norm` is true for the first pair.
            let norm = (findc == '{' as c_int) == ((*cap).nchar == 'm' as c_int);
            n = (*cap).count1;
            if prev_pos.lnum != 0 {
                pos = Some(prev_pos);
                (*curwin.get()).w_cursor = prev_pos;
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
                            clearopbeep((*cap).oap);
                        }
                        n = 0;
                        break;
                    }
                    let c = gchar_cursor();
                    if c != '{' as c_int && c != '}' as c_int {
                        continue;
                    }
                    if (c == findc && norm) || (n == 1 && !norm) {
                        new_pos = (*curwin.get()).w_cursor;
                        pos = Some(new_pos);
                        n = 0;
                    } else if new_pos.lnum == 0 {
                        new_pos = (*curwin.get()).w_cursor;
                        pos = Some(new_pos);
                    } else {
                        // A brace of the other kind: step over the block it
                        // opens or closes.
                        pos = findmatchlimit((*cap).oap, findc, match_direction(cap), 0);
                        match pos {
                            None => n = 0,
                            Some(found) => (*curwin.get()).w_cursor = found,
                        }
                    }
                    break;
                }
                n -= 1;
            }
            (*curwin.get()).w_cursor = *old_pos;
            // A position was found on the way out but lost on the way back in.
            if pos.is_none() && new_pos.lnum != 0 {
                clearopbeep((*cap).oap);
            }
        }

        if let Some(pos) = pos {
            setpcmark();
            (*curwin.get()).w_cursor = pos;
            (*curwin.get()).w_set_curswant = true;
            may_fold_open(cap, kOptFdoFlagBlock as c_uint);
        }
    }
}

/// Look an identifier under the cursor up in the included files.
///
/// The case of the second character picks the action: an upper-case one lists
/// every match, a lower-case one lists the first, and a control character
/// jumps to it. `d`-family keys (`d`, `D`, CTRL-D) search for a `#define`
/// rather than for any occurrence, which is what the low-nibble comparison
/// tests -- CTRL-D, `d` and `D` all end in the same four bits.
unsafe fn nv_bracket_ident(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let mut found: *mut c_char = ptr::null_mut();
        let len = find_ident_under_cursor(&raw mut found, FIND_IDENT as c_int, ptr::null_mut());
        if len == 0 {
            clearop((*cap).oap);
            return;
        }
        let nchar = (*cap).nchar;
        let ctype = *(*__ctype_b_loc()).offset(nchar as isize) as c_int;
        let is_upper = ctype & _ISupper as c_ushort as c_int != 0;
        let is_lower = ctype & _ISlower as c_ushort as c_int != 0;
        // `find_pattern_in_path` keeps the name, so hand it a copy.
        let name = xmemdupz(found as *const c_void, len) as *mut c_char;
        find_pattern_in_path(
            name,
            kDirectionNotSet,
            len,
            true,
            // Without a count, a lower-case key searches case-insensitively.
            if (*cap).count0 == 0 { !is_upper } else { false },
            if nchar & 0xf == 'd' as c_int & 0xf {
                FIND_DEFINE as c_int
            } else {
                FIND_ANY as c_int
            },
            (*cap).count1,
            if is_upper {
                ACTION_SHOW_ALL as c_int
            } else if is_lower {
                ACTION_SHOW as c_int
            } else {
                ACTION_GOTO as c_int
            },
            // `]` starts below the cursor line, `[` at the top of the file.
            if (*cap).cmdchar == ']' as c_int {
                (*curwin.get()).w_cursor.lnum + 1
            } else {
                1
            },
            MAXLNUM as linenr_T,
            false,
            false,
        );
        xfree(name as *mut c_void);
        (*curwin.get()).w_set_curswant = true;
    }
}

/// `['`, `` [` ``, `]'` and `` ]` ``: jump to the next or previous lower-case
/// mark in this buffer.
unsafe fn nv_bracket_mark(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        // The walk starts from a mark standing for the cursor itself.
        let mut fm = pos_to_mark(curbuf.get(), ptr::null_mut(), (*curwin.get()).w_cursor);
        debug_assert!(!fm.is_null());
        let linewise = (*cap).nchar == '\'' as c_int;
        let mut prev_fm = ptr::null_mut();
        let mut n = (*cap).count1;
        while n > 0 {
            prev_fm = fm;
            fm = getnextmark(&raw mut (*fm).mark, direction(cap), linewise as c_int);
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
        nv_mark_move_to(cap, flags, fm);
    }
}

/// `[s`, `[r`, `[S`, `]s`, `]r` and `]S`: jump to a misspelled word.
unsafe fn nv_bracket_spell(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        setpcmark();
        let what = match u8::try_from((*cap).nchar) {
            Ok(b's') => SMT_ALL as smt_T,
            Ok(b'r') => SMT_RARE as smt_T,
            _ => SMT_BAD as smt_T,
        };
        for _ in 0..(*cap).count1 {
            if spell_move_to(curwin.get(), direction(cap), what, false, ptr::null_mut()) == 0 {
                clearopbeep((*cap).oap);
                break;
            }
            (*curwin.get()).w_set_curswant = true;
        }
        may_fold_open(cap, kOptFdoFlagSearch as c_uint);
    }
}

/// `[` and `]`, whose second character says what kind of jump this is.
pub(crate) unsafe fn nv_brackets(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        (*(*cap).oap).motion_type = kMTCharWise;
        (*(*cap).oap).inclusive = false;
        let old_pos = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor.coladd = 0;

        let nchar = (*cap).nchar;
        let opening = (*cap).cmdchar == '[' as c_int;
        // The bracket forms that name a block: `[{ [( [* [/ [# [m [M` and the
        // closing halves of each.
        let block_chars: &CStr = if opening { c"{(*/#mM" } else { c"})*/#mM" };

        if nchar == 'f' as c_int {
            nv_gotofile(cap);
        } else if !vim_strchr(c"iI\tdD\x04".as_ptr(), nchar).is_null() {
            nv_bracket_ident(cap);
        } else if !vim_strchr(block_chars.as_ptr(), nchar).is_null() {
            nv_bracket_block(cap, &raw const old_pos);
        } else if nchar == '[' as c_int || nchar == ']' as c_int {
            // `[[` and `]]` look for a section start, `[]` and `][` for its
            // end.
            let flag = if nchar == (*cap).cmdchar {
                '{' as c_int
            } else {
                '}' as c_int
            };
            (*curwin.get()).w_set_curswant = true;
            if !findpar(
                &raw mut (*(*cap).oap).inclusive,
                (*cap).arg,
                (*cap).count1,
                flag,
                (*(*cap).oap).op_type != OP_NOP
                    && (*cap).arg == FORWARD as c_int
                    && flag == '{' as c_int,
            ) {
                clearopbeep((*cap).oap);
            } else {
                if (*(*cap).oap).op_type == OP_NOP {
                    beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
                }
                may_fold_open(cap, kOptFdoFlagBlock as c_uint);
            }
        } else if nchar == 'p' as c_int || nchar == 'P' as c_int {
            // The put that reindents to the current line.
            nv_put_opt(cap, true);
        } else if nchar == '\'' as c_int || nchar == '`' as c_int {
            nv_bracket_mark(cap);
        } else if (K_RIGHTRELEASE..=K_LEFTMOUSE).contains(&nchar) {
            // A mouse click after `[` or `]` pastes at the click, reindenting.
            do_mouse(
                (*cap).oap,
                nchar,
                direction(cap),
                (*cap).count1,
                PUT_FIXINDENT as c_int != 0,
            );
        } else if nchar == 'z' as c_int {
            if fold_move_to(false, direction(cap), (*cap).count1) == 0 {
                clearopbeep((*cap).oap);
            }
        } else if nchar == 'c' as c_int {
            if diff_move_to(direction(cap), (*cap).count1) == 0 {
                clearopbeep((*cap).oap);
            }
        } else if nchar == 'r' as c_int || nchar == 's' as c_int || nchar == 'S' as c_int {
            nv_bracket_spell(cap);
        } else {
            clearopbeep((*cap).oap);
        }
    }
}
