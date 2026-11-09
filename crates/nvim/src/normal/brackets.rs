//! The `[` and `]` commands: the block, comment, define and method
//! searches, the mark and fold jumps, and the paste variants.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::keycodes::SpecialKey;
use crate::mark::{MARK_BEGIN_LINE, MARK_CONTEXT};
use crate::normal::defs::{CmdArg, MotionType};
use crate::search::{ACTION_GOTO, ACTION_SHOW, ACTION_SHOW_ALL, FIND_ANY, FIND_DEFINE};
use crate::strings::has_char;
use crate::types::{ColNr, Key, LineNr, Pos};
use crate::winlayer::{Buf, Win};

use crate::cursor::{dec_cursor, gchar_cursor, inc_cursor};
use crate::diff::diff_move_to;
use crate::edit::{BeginlineOpts, beginline};
use crate::fold::fold_move_to;
use crate::mark::defs::MarkMove;
use crate::mark::{getnextmark, pos_to_mark, setpcmark};
use crate::mouse::do_mouse;
use crate::normal::{
    DIRECTION_NOT_SET, FIND_IDENT, FM_BACKWARD, FM_FORWARD, SMT_BAD, SMT_RARE, clear_op,
    clear_op_beep, ident_under_cursor, may_fold_open, nv_gotofile, nv_mark_move_to, nv_put_opt,
};
use crate::ops::OpType;
use crate::options::{OPT_FDO_FLAG_BLOCK, OPT_FDO_FLAG_SEARCH};
use crate::pos::MAXLNUM;
use crate::register::defs::PUT_FIXINDENT;
use crate::search::{BACKWARD, FORWARD, find_pattern_in_path, findmatchlimit};
use crate::spell::{SMT_ALL, SpellMoveType};
use crate::textobject::findpar;
use core::ffi::{CStr, c_int, c_uint};

/// Which way a `[` or `]` command searches.
fn direction(cmd_arg: &mut CmdArg) -> c_int {
    if cmd_arg.cmdchar.is(']') {
        FORWARD as c_int
    } else {
        BACKWARD as c_int
    }
}

/// The same choice spelled in `findmatchlimit`'s own flags, which are not the
/// `Direction` constants.
fn match_direction(cmd_arg: &mut CmdArg) -> c_int {
    if cmd_arg.cmdchar.is('[') {
        FM_BACKWARD.cast_signed()
    } else {
        FM_FORWARD.cast_signed()
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
/// `old_pos` is the cursor position the caller saved.
fn nv_bracket_block(cmd_arg: &mut CmdArg, old_pos: Pos) {
    let mut new_pos = Pos {
        lnum: LineNr::ZERO,
        col: ColNr::ZERO,
        coladd: 0,
    };
    let mut prev_pos = Pos {
        lnum: LineNr::ZERO,
        col: ColNr::ZERO,
        coladd: 0,
    };
    let mut pos: Option<Pos> = None;

    // `[*` and `]*` are spelled `[/` and `]/` to findmatchlimit.
    if cmd_arg.nchar.is('*') {
        cmd_arg.nchar = Key::ascii(b'/');
    }
    let method = cmd_arg.nchar.is('m') || cmd_arg.nchar.is('M');
    let findc = if method {
        if cmd_arg.cmdchar.is('[') {
            c_int::from(b'{')
        } else {
            c_int::from(b'}')
        }
    } else {
        cmd_arg.nchar.code()
    };
    let mut n = if method { 9999 } else { cmd_arg.count1 };

    while n > 0 {
        pos = findmatchlimit(Some(&mut cmd_arg.op()), findc, match_direction(cmd_arg), 0);
        let Some(found) = pos else {
            if new_pos.lnum == LineNr::ZERO {
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
    Win::current().w_cursor = old_pos;

    if method {
        // `[m` and `]M` want the brace itself; `[M` and `]m` want the one
        // before it. `norm` is true for the first pair.
        let norm = (findc == c_int::from(b'{')) == (cmd_arg.nchar.is('m'));
        n = cmd_arg.count1;
        if prev_pos.lnum != LineNr::ZERO {
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
                let stepped = if findc == c_int::from(b'{') {
                    dec_cursor()
                } else {
                    inc_cursor()
                };
                if stepped.is_stuck() {
                    // Hit the end of the buffer with nothing found.
                    if pos.is_none() {
                        clear_op_beep(cmd_arg.op());
                    }
                    n = 0;
                    break;
                }
                let c = gchar_cursor();
                if c != c_int::from(b'{') && c != c_int::from(b'}') {
                    continue;
                }
                if (c == findc && norm) || (n == 1 && !norm) {
                    new_pos = Win::current().w_cursor;
                    pos = Some(new_pos);
                    n = 0;
                } else if new_pos.lnum == LineNr::ZERO {
                    new_pos = Win::current().w_cursor;
                    pos = Some(new_pos);
                } else {
                    // A brace of the other kind: step over the block it
                    // opens or closes.
                    pos =
                        findmatchlimit(Some(&mut cmd_arg.op()), findc, match_direction(cmd_arg), 0);
                    match pos {
                        None => n = 0,
                        Some(found) => Win::current().w_cursor = found,
                    }
                }
                break;
            }
            n -= 1;
        }
        Win::current().w_cursor = old_pos;
        // A position was found on the way out but lost on the way back in.
        if pos.is_none() && new_pos.lnum != LineNr::ZERO {
            clear_op_beep(cmd_arg.op());
        }
    }

    if let Some(pos) = pos {
        setpcmark();
        Win::current().w_cursor = pos;
        Win::current().w_set_curswant = true;
        may_fold_open(cmd_arg, OPT_FDO_FLAG_BLOCK as c_uint);
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
    // `find_pattern_in_path` keeps the name, so it is handed a copy.
    let name = ident_under_cursor(FIND_IDENT);
    if name.is_empty() {
        clear_op(cmd_arg.op());
        return;
    }
    let nchar = cmd_arg.nchar;
    // Only `i`, `I`, `d`, `D` and their control characters reach here.
    let key = nchar.as_byte().unwrap_or(0);
    let (is_upper, is_lower) = (key.is_ascii_uppercase(), key.is_ascii_lowercase());
    // Without a count, a lower-case key searches case-insensitively.
    let fold_case = if cmd_arg.count0 == 0 {
        !is_upper
    } else {
        false
    };
    let what = if nchar.code() & 0xf == c_int::from(b'd') & 0xf {
        FIND_DEFINE
    } else {
        FIND_ANY
    };
    let action = if is_upper {
        ACTION_SHOW_ALL
    } else if is_lower {
        ACTION_SHOW
    } else {
        ACTION_GOTO
    };
    // `]` starts below the cursor line, `[` at the top of the file.
    let from = if cmd_arg.cmdchar.is(']') {
        Win::current().w_cursor.lnum + 1
    } else {
        LineNr::ONE
    };
    let (dir, n) = (DIRECTION_NOT_SET, cmd_arg.count1);
    let last = MAXLNUM;
    find_pattern_in_path(
        &name, dir, true, fold_case, what, n, action, from, last, false, false,
    );
    Win::current().w_set_curswant = true;
}

/// `['`, `` [` ``, `]'` and `` ]` ``: jump to the next or previous lower-case
/// mark in this buffer.
fn nv_bracket_mark(cmd_arg: &mut CmdArg) {
    // The walk starts from a mark standing for the cursor itself; every
    // later one is a copy of a buffer mark. Running out of marks stops at the
    // last one found rather than failing.
    let mut fm = pos_to_mark(Buf::current(), Win::current().w_cursor);
    let linewise = cmd_arg.nchar == Key::ascii(b'\'');
    for _ in 0..cmd_arg.count1 {
        match getnextmark(fm.mark, direction(cmd_arg), linewise) {
            Some(next) => fm = next,
            None => break,
        }
    }
    let mut flags = MARK_CONTEXT as MarkMove;
    if linewise {
        flags |= MARK_BEGIN_LINE as MarkMove;
    }
    nv_mark_move_to(cmd_arg, flags, Some(&fm));
}

/// `[s`, `[r`, `[S`, `]s`, `]r` and `]S`: jump to a misspelled word.
fn nv_bracket_spell(cmd_arg: &mut CmdArg) {
    setpcmark();
    let what = match cmd_arg.nchar.as_byte() {
        Some(b's') => SMT_ALL as SpellMoveType,
        Some(b'r') => SMT_RARE as SpellMoveType,
        _ => SMT_BAD as SpellMoveType,
    };
    for _ in 0..cmd_arg.count1 {
        if Win::current().spell_move_to(direction(cmd_arg), what, false, None) == 0 {
            clear_op_beep(cmd_arg.op());
            break;
        }
        Win::current().w_set_curswant = true;
    }
    may_fold_open(cmd_arg, OPT_FDO_FLAG_SEARCH as c_uint);
}

/// `[` and `]`, whose second character says what kind of jump this is.
pub(crate) fn nv_brackets(cmd_arg: &mut CmdArg) {
    cmd_arg.op().motion_type = MotionType::CharWise;
    cmd_arg.op().inclusive = false;
    let old_pos = Win::current().w_cursor;
    Win::current().w_cursor.coladd = 0;

    let nchar = cmd_arg.nchar;
    let opening = cmd_arg.cmdchar.is('[');
    // The bracket forms that name a block: `[{ [( [* [/ [# [m [M` and the
    // closing halves of each.
    let block_chars: &CStr = if opening { c"{(*/#mM" } else { c"})*/#mM" };

    if nchar.is('f') {
        nv_gotofile(cmd_arg);
    } else if has_char(c"iI\tdD\x04", nchar.code()) {
        nv_bracket_ident(cmd_arg);
    } else if has_char(block_chars, nchar.code()) {
        nv_bracket_block(cmd_arg, old_pos);
    } else if nchar.is('[') || nchar.is(']') {
        // `[[` and `]]` look for a section start, `[]` and `][` for its
        // end.
        let flag = if nchar == cmd_arg.cmdchar {
            c_int::from(b'{')
        } else {
            c_int::from(b'}')
        };
        Win::current().w_set_curswant = true;
        let both_ways = cmd_arg.op().op_type != OpType::Nop
            && cmd_arg.arg == FORWARD as c_int
            && flag == c_int::from(b'{');
        let (dir, n) = (cmd_arg.arg, cmd_arg.count1);
        if let Some(inclusive) = findpar(dir, n, flag, both_ways) {
            if inclusive {
                cmd_arg.op().inclusive = true;
            }
            if cmd_arg.op().op_type == OpType::Nop {
                beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
            }
            may_fold_open(cmd_arg, OPT_FDO_FLAG_BLOCK as c_uint);
        } else {
            clear_op_beep(cmd_arg.op());
        }
    } else if nchar.is('p') || nchar.is('P') {
        // The put that reindents to the current line.
        nv_put_opt(cmd_arg, true);
    } else if nchar.is('\'') || nchar.is('`') {
        nv_bracket_mark(cmd_arg);
    } else if (SpecialKey::Rightrelease.code()..=SpecialKey::Leftmouse.code())
        .contains(&nchar.code())
    {
        // A mouse click after `[` or `]` pastes at the click, reindenting.
        let (dir, n) = (direction(cmd_arg), cmd_arg.count1);
        do_mouse(
            Some(cmd_arg.op()),
            nchar,
            dir,
            n,
            PUT_FIXINDENT.cast_signed() != 0,
        );
    } else if nchar.is('z') {
        if !fold_move_to(false, direction(cmd_arg), cmd_arg.count1) {
            clear_op_beep(cmd_arg.op());
        }
    } else if nchar.is('c') {
        if diff_move_to(direction(cmd_arg), cmd_arg.count1).is_err() {
            clear_op_beep(cmd_arg.op());
        }
    } else if nchar.is('r') || nchar.is('s') || nchar.is('S') {
        nv_bracket_spell(cmd_arg);
    } else {
        clear_op_beep(cmd_arg.op());
    }
}
