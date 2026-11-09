//! Commands that change the text without entering insert mode, and the
//! ones whose whole job is to enter it.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::keycodes::SpecialKey;
use crate::memline::MlFlags;
use crate::narrow::len_as_int;
use crate::types::Key;
use crate::winlayer::{Buf, Win};

use crate::ascii::NUL;
use crate::buffer::{buf_get_changedtick, buf_is_prompt, current_buf};
use crate::change::OPENLINE_DO_COM;
use crate::change::{changed_lines, del_chars, deleted_lines, ins_char, ins_char_bytes, open_line};
use crate::cursor::{
    check_cursor, coladvance, coladvance_force, gchar_cursor, get_cursor_pos_len, getviscol,
    inc_cursor, with_cursor_rest,
};
use crate::diff::nv_diffgetput;
use crate::drawscreen::win_cursorline_standout;
use crate::edit::{
    BeginlineOpts, beginline, edit, get_literal, ins_copychar, prompt_curpos_editable,
    set_last_insert,
};
use crate::fold::fold_update_after_insert;
use crate::getchar::state::GOT_INT;
use crate::getchar::{
    append_to_redobuff, append_to_redobuff_char, stuff_empty, stuff_readbuf, stuff_readbuf_char,
    stuff_readbuf_number,
};
use crate::guard::Suppress;
use crate::mbyte::{char_count, mb_adjust_cursor};
use crate::memline::ML_DEL_MESSAGE;
use crate::memline::{inc, ml_delete_flags};
use crate::message::E_MODIFIABLE;
use crate::message::emsg;
use crate::r#move::WinValid;
use crate::normal::defs::{CmdArg, Outcome};
use crate::normal::{
    TAB, V_ISUAL_MODE_ORIG, VisualMode, check_clear_op, check_clear_op_quit, clear_op,
    clear_op_beep, nv_object, nv_operator, prep_redo, prep_redo_cmd, set_visual_active,
    set_visual_mode, v_swap_corners, v_visop, visual_active, visual_mode,
};
use crate::ops::OpType;
use crate::ops::{
    REPLACE_CR_NCHAR, REPLACE_NL_NCHAR, do_join, do_pending_operator, op_addsub, swapchar,
};
use crate::option::defs::FoFlag;
use crate::option::get_ve_flags;
use crate::option::vars::{CB_FLAGS, P_SEL, p_sta, p_to, p_ww};
use crate::options::{OPT_CB_FLAG_UNNAMED, OPT_CB_FLAG_UNNAMEDPLUS, OPT_VE_FLAG_ALL};
use crate::os::cshim::gettext;
use crate::register::defs::{
    PUT_BLOCK_INNER, PUT_CURSEND, PUT_FIXINDENT, PUT_LINE, PUT_LINE_FORWARD, PUT_LINE_SPLIT,
    YankReg,
};
use crate::register::{copy_register, do_put};
use crate::search::{BACKWARD, FORWARD};
use crate::state::mode::{RESTART_EDIT, STATE};
use crate::state::{MODE_INSERT, MODE_REPLACE, virtual_active};
use crate::textformat::{auto_format, has_format_option};
use crate::types::{ColNr, LineNr};
use crate::undo::{u_clearline, u_save, u_save_cursor, u_savesub};
use core::ffi::{CStr, c_int, c_uint};

/// Refuse a change in a prompt buffer that is not on its own editable line.
fn prompt_refuses(cmd_arg: &mut CmdArg) -> bool {
    if buf_is_prompt(current_buf()) && !prompt_curpos_editable() {
        clear_op_beep(cmd_arg.op());
        return true;
    }
    false
}

/// `CTRL-A` and `CTRL-X`: add to or subtract from the number under the cursor.
pub(crate) fn nv_addsub(cmd_arg: &mut CmdArg) {
    if prompt_refuses(cmd_arg) {
        return;
    }
    if !visual_active() && cmd_arg.op().op_type == OpType::Nop {
        // Not an operator: run it here and then put the operator back.
        prep_redo_cmd(cmd_arg);
        cmd_arg.op().op_type = if cmd_arg.cmdchar == Key::CTRL_A {
            OpType::NrAdd
        } else {
            OpType::NrSub
        };
        let (count, g_cmd) = (LineNr::new(cmd_arg.count1), cmd_arg.arg != 0);
        cmd_arg.op().on_copy(|op| op_addsub(op, count.get(), g_cmd));
        cmd_arg.op().op_type = OpType::Nop;
    } else if visual_active() {
        nv_operator(cmd_arg);
    } else {
        clear_op(cmd_arg.op());
    }
}

/// `r`: replace `count1` characters with the one that follows.
pub(crate) fn nv_replace(cmd_arg: &mut CmdArg) {
    if check_clear_op(cmd_arg.op()) || prompt_refuses(cmd_arg) {
        return;
    }
    // `r CTRL-V` reads the next key literally. Only a byte-sized answer
    // still counts as literal: a larger one came from a digraph or a
    // `<C-u>` escape and behaves as an ordinary character.
    let mut literal = false;
    if cmd_arg.nchar == Key::CTRL_V || cmd_arg.nchar == Key::CTRL_Q {
        cmd_arg.nchar = get_literal(false);
        literal = (0..=0x7f).contains(&cmd_arg.nchar.code());
    }
    if cmd_arg.nchar.is_special() {
        clear_op_beep(cmd_arg.op());
        return;
    }
    if visual_active() {
        // The selection form is an operator; the interrupt a long
        // selection may have raised is not this command's business.
        if GOT_INT.get() {
            GOT_INT.set(false);
        }
        if literal {
            // A literal line break has to survive being carried through
            // `cmd_arg` as a character.
            if cmd_arg.nchar == Key::CAR {
                cmd_arg.nchar = REPLACE_CR_NCHAR;
            } else if cmd_arg.nchar == Key::NL {
                cmd_arg.nchar = REPLACE_NL_NCHAR;
            }
        }
        nv_operator(cmd_arg);
        return;
    }
    if virtual_active(Win::current()) {
        if u_save_cursor().is_err() {
            return;
        }
        if gchar_cursor() == NUL {
            // Past the end of the line: make room for the whole count and
            // then step back to where the replacing starts.
            coladvance_force(getviscol() + cmd_arg.count1);
            Win::current().w_cursor.col -= cmd_arg.count1;
        } else if gchar_cursor() == TAB {
            // Land on the tab's first cell, not the cell of it the cursor
            // happens to be showing on.
            coladvance_force(getviscol());
        }
    }
    // There have to be `count1` characters left on the line, counted both
    // ways: the byte length rules out a short line cheaply.
    // A negative length or count reads as huge, as upstream's unsigned
    // comparison does.
    let want = usize::try_from(cmd_arg.count1).unwrap_or(usize::MAX);
    if usize::try_from(get_cursor_pos_len()).unwrap_or(usize::MAX) < want
        || with_cursor_rest(char_count) < want
    {
        clear_op_beep(cmd_arg.op());
        return;
    }
    // A tab that 'expandtab' or 'smarttab' would turn into spaces is
    // easier to get right by replaying the whole thing as `R<Tab><Esc>`.
    if !literal && cmd_arg.nchar == Key::TAB && (Buf::current().b_p_et != 0 || p_sta()) {
        stuff_readbuf_number(cmd_arg.count1);
        stuff_readbuf_char(Key::ascii(b'R'));
        stuff_readbuf_char(Key::ascii(b'\t'));
        stuff_readbuf_char(Key::ESC);
        return;
    }
    if u_save_cursor().is_err() {
        return;
    }
    if !literal && (cmd_arg.nchar == Key::CAR || cmd_arg.nchar == Key::NL) {
        // Replacing with a line break splits the line, which is an insert.
        let _ = del_chars(cmd_arg.count1, 0);
        stuff_readbuf_char(Key::ascii(b'\r'));
        stuff_readbuf_char(Key::ESC);
        invoke_edit(cmd_arg, true, Key::ascii(b'r'), false);
        fold_update_after_insert();
        return;
    }

    prep_redo(
        cmd_arg.op().regname,
        cmd_arg.count1,
        Key::NUL,
        Key::ascii(b'r'),
        Key::NUL,
        if literal { Key::CTRL_V } else { Key::NUL },
        Key::NUL,
    );
    Buf::current().b_op_start = Win::current().w_cursor;
    let old_state = STATE.get();
    if cmd_arg.nchar_len > 0 {
        append_to_redobuff(cstr::until_nul(&cmd_arg.nchar_composing));
    } else {
        append_to_redobuff_char(cmd_arg.nchar);
    }
    for _ in 0..cmd_arg.count1 {
        // `ins_char` looks at 'State' to decide it is overwriting rather
        // than inserting.
        STATE.set(MODE_REPLACE);
        if cmd_arg.nchar == Key::CTRL_E || cmd_arg.nchar == Key::CTRL_Y {
            // `r CTRL-E` and `r CTRL-Y` copy from the line below or above.
            let from =
                Win::current().w_cursor.lnum + if cmd_arg.nchar == Key::CTRL_Y { -1 } else { 1 };
            let c = ins_copychar(from);
            if !c.is_nul() {
                ins_char(c);
            } else {
                // Nothing there to copy: leave the character alone and
                // step over it.
                Win::current().w_cursor.col += 1;
            }
        } else if cmd_arg.nchar_len != 0 {
            let bytes = cstr::until_nul(&cmd_arg.nchar_composing);
            ins_char_bytes(&bytes[..cmd_arg.nchar_len]);
        } else {
            ins_char(cmd_arg.nchar);
        }
        STATE.set(old_state);
    }
    Win::current().w_cursor.col -= 1;
    mb_adjust_cursor();
    Buf::current().b_op_end = Win::current().w_cursor;
    Win::current().w_set_curswant = true;
    set_last_insert(cmd_arg.nchar);
    fold_update_after_insert();
}

/// `R` and `gR`: replace mode, virtual with the argument set.
pub(crate) fn nv_replace_mode(cmd_arg: &mut CmdArg) {
    if visual_active() {
        // A selection is replaced linewise, which is `c` over whole lines.
        cmd_arg.cmdchar = Key::ascii(b'c');
        cmd_arg.nchar = Key::NUL;
        V_ISUAL_MODE_ORIG.set(visual_mode());
        set_visual_mode(VisualMode::LINE);
        nv_operator(cmd_arg);
        return;
    }
    if check_clear_op_quit(cmd_arg.op()) {
        return;
    }
    if Buf::current().b_p_ma == 0 {
        emsg(gettext(E_MODIFIABLE));
        return;
    }
    if virtual_active(Win::current()) {
        coladvance(Win::current(), getviscol());
    }
    let kind = if cmd_arg.arg != 0 {
        Key::ascii(b'V')
    } else {
        Key::ascii(b'R')
    };
    invoke_edit(cmd_arg, false, kind, false);
}

/// `gr`: replace one character virtually -- the following text does not move.
pub(crate) fn nv_vreplace(cmd_arg: &mut CmdArg) {
    if visual_active() {
        cmd_arg.cmdchar = Key::ascii(b'r');
        cmd_arg.nchar = cmd_arg.extra_char;
        nv_replace(cmd_arg);
        return;
    }
    if check_clear_op_quit(cmd_arg.op()) {
        return;
    }
    if Buf::current().b_p_ma == 0 {
        emsg(gettext(E_MODIFIABLE));
        return;
    }
    if cmd_arg.extra_char == Key::CTRL_V || cmd_arg.extra_char == Key::CTRL_Q {
        cmd_arg.extra_char = get_literal(false);
    }
    // Replay the character through virtual replace mode. A control
    // character needs its own CTRL-V to survive the replay.
    if cmd_arg.extra_char.code() < 0x20 {
        stuff_readbuf_char(Key::ctrl(b'V'));
    }
    stuff_readbuf_char(cmd_arg.extra_char);
    stuff_readbuf_char(Key::ESC);
    if virtual_active(Win::current()) {
        coladvance(Win::current(), getviscol());
    }
    invoke_edit(cmd_arg, true, Key::ascii(b'v'), false);
}

/// `~` when 'tildeop' is off: swap the case of `count1` characters.
pub(crate) fn n_swapchar(cmd_arg: &mut CmdArg) {
    if check_clear_op_quit(cmd_arg.op()) {
        return;
    }
    // An empty line has nothing to swap unless 'whichwrap' lets `~` move
    // to the next one.
    let wraps = p_ww(|value| value.to_bytes().contains(&b'~'));
    if Buf::current()
        .lines()
        .line(Win::current().w_cursor.lnum)
        .is_empty()
        && !wraps
    {
        clear_op_beep(cmd_arg.op());
        return;
    }
    prep_redo_cmd(cmd_arg);
    if u_save_cursor().is_err() {
        return;
    }
    let startpos = Win::current().w_cursor;
    let mut did_change = false;
    let mut n = cmd_arg.count1;
    while n > 0 {
        did_change |= swapchar(cmd_arg.op().op_type, Win::current().w_cursor);
        inc_cursor();
        if gchar_cursor() == NUL {
            if !(wraps && Win::current().w_cursor.lnum < Buf::current().b_ml.ml_line_count) {
                break;
            }
            Win::current().w_cursor.lnum += 1;
            Win::current().w_cursor.col = ColNr::ZERO;
            // Each further line needs its own undo entry.
            if n > 1 {
                if u_savesub(Win::current().w_cursor.lnum).is_err() {
                    break;
                }
                u_clearline(Buf::current());
            }
        }
        n -= 1;
    }
    check_cursor(Win::current());
    Win::current().w_set_curswant = true;
    if did_change {
        let (from, col) = (startpos.lnum, startpos.col);
        let to = Win::current().w_cursor.lnum + 1;
        changed_lines(Buf::current(), from, col, to, 0, true);
        Buf::current().b_op_start = startpos;
        Buf::current().b_op_end = Win::current().w_cursor;
        if Buf::current().b_op_end.col > ColNr::ZERO {
            Buf::current().b_op_end.col -= 1;
        }
    }
}

/// `s` and `S`: substitute a character or a line.
pub(crate) fn nv_subst(cmd_arg: &mut CmdArg) {
    if prompt_refuses(cmd_arg) {
        return;
    }
    if visual_active() {
        if cmd_arg.cmdchar.is('S') {
            // `S` on a selection is linewise however the selection was
            // made; the original kind is remembered for the redo.
            V_ISUAL_MODE_ORIG.set(visual_mode());
            set_visual_mode(VisualMode::LINE);
        }
        cmd_arg.cmdchar = Key::ascii(b'c');
        nv_operator(cmd_arg);
    } else {
        nv_optrans(cmd_arg);
    }
}

/// `x`, `X`, `D`, `C`, `Y`: the one-key spellings of an operator and a motion.
pub(crate) fn nv_abbrev(cmd_arg: &mut CmdArg) {
    if cmd_arg.cmdchar == SpecialKey::Del || cmd_arg.cmdchar == SpecialKey::Kdel {
        cmd_arg.cmdchar = Key::ascii(b'x');
    }
    if visual_active() {
        v_visop(cmd_arg);
    } else {
        nv_optrans(cmd_arg);
    }
}

/// Replay a one-key command as the operator and motion it stands for.
pub(crate) fn nv_optrans(cmd_arg: &mut CmdArg) {
    /// What each abbreviating key means. Upstream indexes two parallel arrays
    /// with a `strchr` offset, which reaches one past the end for a key that
    /// is not in the set -- unreachable, because only these keys route here,
    /// but not something a lookup has to make possible.
    const TRANSLATIONS: [(Key, &CStr); 8] = [
        (Key::ascii(b'x'), c"dl"),
        (Key::ascii(b'X'), c"dh"),
        (Key::ascii(b'D'), c"d$"),
        (Key::ascii(b'C'), c"c$"),
        (Key::ascii(b's'), c"cl"),
        (Key::ascii(b'S'), c"cc"),
        (Key::ascii(b'Y'), c"yy"),
        (Key::ascii(b'&'), c":s\r"),
    ];
    if !check_clear_op_quit(cmd_arg.op()) {
        if cmd_arg.count0 != 0 {
            stuff_readbuf_number(cmd_arg.count0);
        }
        for (key, keys) in TRANSLATIONS {
            if key == cmd_arg.cmdchar {
                stuff_readbuf(keys.to_bytes());
                break;
            }
        }
    }
    // The count went into the replayed keys, so it must not also apply to
    // whatever they turn out to be.
    cmd_arg.opcount = 0;
}

/// `o` and `O`: open a line below or above and start inserting on it.
pub(crate) fn n_opencmd(cmd_arg: &mut CmdArg) {
    if check_clear_op_quit(cmd_arg.op()) {
        return;
    }
    let mut win = Win::current();
    let opening_above = cmd_arg.cmdchar.is('O');
    // Open outside a closed fold rather than inside it: `O` stretches to the
    // fold's first line, `o` to its last.
    let lnum = win.w_cursor.lnum;
    let mut edge = lnum;
    if opening_above {
        win.has_folding(lnum, Some(&mut edge), None);
    } else {
        win.has_folding(lnum, None, Some(&mut edge));
    }
    win.w_cursor.lnum = edge;
    Buf::current().b_last_changedtick_i = buf_get_changedtick(Buf::current());
    let undo_first = win.w_cursor.lnum - i32::from(opening_above);
    let undo_last = win.w_cursor.lnum + i32::from(!opening_above);
    let dir = if opening_above {
        BACKWARD as c_int
    } else {
        FORWARD as c_int
    };
    let flags = if has_format_option(FoFlag::OPEN_COMS) {
        OPENLINE_DO_COM
    } else {
        0
    };
    let opened = u_save(LineNr::new(undo_first.get()), undo_last).is_ok()
        && open_line(dir, flags, 0, None).is_ok();
    if opened {
        if win_cursorline_standout(win) {
            // The cursor line moved, so its highlight has to be redrawn.
            win.w_valid.clear(WinValid::CROW);
        }
        invoke_edit(cmd_arg, false, cmd_arg.cmdchar, true);
    }
}

/// `~`: swap case, or the `g~` operator when 'tildeop' is on.
pub(crate) fn nv_tilde(cmd_arg: &mut CmdArg) {
    if !p_to() && !visual_active() && cmd_arg.op().op_type != OpType::Tilde {
        if prompt_refuses(cmd_arg) {
            return;
        }
        n_swapchar(cmd_arg);
    } else {
        nv_operator(cmd_arg);
    }
}

/// Put the cursor where `A` starts inserting: past the last character, or
/// past the last *cell* when 'virtualedit' is "all".
pub(crate) fn set_cursor_for_append_to_line() {
    Win::current().w_set_curswant = true;
    if get_ve_flags(Win::current()) == OPT_VE_FLAG_ALL as c_uint {
        // Insert mode is what makes `coladvance` allow the position one
        // past the end.
        let save_state = STATE.get();
        STATE.set(MODE_INSERT);
        coladvance(Win::current(), ColNr::MAX);
        STATE.set(save_state);
    } else {
        Win::current().w_cursor.col += len_as_int(with_cursor_rest(<[u8]>::len));
    }
}

/// `a`, `A`, `i` and `I`: enter insert mode.
pub(crate) fn nv_edit(cmd_arg: &mut CmdArg) {
    if cmd_arg.cmdchar == SpecialKey::Ins || cmd_arg.cmdchar == SpecialKey::Kins {
        cmd_arg.cmdchar = Key::ascii(b'i');
    }
    // With a selection up, `A` and `I` insert at every line's end or
    // start; `a` and `i` name a text object instead.
    if visual_active() && (cmd_arg.cmdchar.is('A') || cmd_arg.cmdchar.is('I')) {
        v_visop(cmd_arg);
        return;
    }
    if (cmd_arg.cmdchar.is('a') || cmd_arg.cmdchar.is('i'))
        && (cmd_arg.op().op_type != OpType::Nop || visual_active())
    {
        nv_object(cmd_arg);
        return;
    }
    // A terminal buffer is not 'modifiable' and is still editable.
    if Buf::current().b_p_ma == 0 && Buf::current().terminal.is_null() {
        emsg(gettext(E_MODIFIABLE));
        clear_op(cmd_arg.op());
        return;
    }
    if check_clear_op_quit(cmd_arg.op()) {
        return;
    }
    match cmd_arg.cmdchar.as_byte() {
        Some(b'A') => set_cursor_for_append_to_line(),
        Some(b'I') => beginline(BeginlineOpts::WHITE),
        Some(b'a') => {
            // `a` steps one right first. Under 'virtualedit' a position
            // inside a tab or past the end of the line moves by a cell.
            if virtual_active(Win::current())
                && (Win::current().w_cursor.coladd > 0
                    || gchar_cursor() == NUL
                    || gchar_cursor() == TAB)
            {
                Win::current().w_cursor.coladd += 1;
            } else if gchar_cursor() != NUL {
                inc_cursor();
            }
        }
        _ => {}
    }
    // Insert mode has no virtual column of its own, so anything but `A`
    // has to land on a real one first.
    if Win::current().w_cursor.coladd != 0 && !cmd_arg.cmdchar.is('A') {
        let save_state = STATE.get();
        STATE.set(MODE_INSERT);
        coladvance(Win::current(), getviscol());
        STATE.set(save_state);
    }
    invoke_edit(cmd_arg, false, cmd_arg.cmdchar, false);
}

/// Enter insert mode and report back whether the command loop should treat
/// this command as still running.
///
/// 'restart_edit' is put back afterwards only if insert mode did not set one
/// itself: whatever it asked for wins over what was pending before.
pub(crate) fn invoke_edit(cmd_arg: &mut CmdArg, repl: bool, cmd: Key, startln: bool) {
    // A replay or leftover typeahead is allowed to resume a pending
    // insert; a fresh command is not.
    let restart_edit_save = if repl || !stuff_empty() {
        RESTART_EDIT.get()
    } else {
        Key::NUL
    };
    RESTART_EDIT.set(Key::NUL);
    // `o` and `O` already recorded the tick before opening the line.
    if !cmd_arg.cmdchar.is('O') && !cmd_arg.cmdchar.is('o') {
        Buf::current().b_last_changedtick_i = buf_get_changedtick(Buf::current());
    }
    if edit(cmd, startln, cmd_arg.count1) == Ok(true) {
        cmd_arg.outcome |= Outcome::COMMAND_BUSY;
    }
    if RESTART_EDIT.get().is_nul() {
        RESTART_EDIT.set(restart_edit_save);
    }
}

/// `J`: join lines.
pub(crate) fn nv_join(cmd_arg: &mut CmdArg) {
    if visual_active() {
        nv_operator(cmd_arg);
        return;
    }
    if check_clear_op(cmd_arg.op()) {
        return;
    }
    // Joining fewer than two lines means nothing; `J` and `1J` both join
    // this line with the next.
    cmd_arg.count0 = cmd_arg.count0.max(2);
    if Win::current().w_cursor.lnum + cmd_arg.count0 - 1 > Buf::current().b_ml.ml_line_count {
        // A count that runs off the end joins what is left -- unless there
        // was no count, in which case there is nothing below to join to.
        if cmd_arg.count0 <= 2 {
            clear_op_beep(cmd_arg.op());
            return;
        }
        cmd_arg.count0 =
            (Buf::current().b_ml.ml_line_count - Win::current().w_cursor.lnum + 1) as c_int;
    }
    prep_redo(
        cmd_arg.op().regname,
        cmd_arg.count0,
        Key::NUL,
        cmd_arg.cmdchar,
        Key::NUL,
        Key::NUL,
        cmd_arg.nchar,
    );
    // `gJ` arrives with `nchar` set and does not insert or remove spaces.
    let _ = do_join(
        usize::try_from(cmd_arg.count0).unwrap_or(0),
        cmd_arg.nchar.is_nul(),
        true,
        true,
        true,
    );
}

/// `p` and `P`.
pub(crate) fn nv_put(cmd_arg: &mut CmdArg) {
    nv_put_opt(cmd_arg, false);
}

/// The put commands. `fix_indent` is the `]p`/`[p` family, which reindents the
/// text to the current line.
pub(crate) fn nv_put_opt(cmd_arg: &mut CmdArg, fix_indent: bool) {
    let mut win = Win::current();
    let save_fen = win.w_onebuf_opt.wo_fen;
    if cmd_arg.op().op_type != OpType::Nop {
        // `dp` is not "delete, then put": it is the diff command.
        if cmd_arg.op().op_type == OpType::Delete && cmd_arg.cmdchar.is('p') {
            clear_op(cmd_arg.op());
            debug_assert!(cmd_arg.opcount >= 0);
            nv_diffgetput(true, usize::try_from(cmd_arg.opcount).unwrap_or(0));
        } else {
            clear_op_beep(cmd_arg.op());
        }
        return;
    }
    if buf_is_prompt(current_buf()) && !prompt_curpos_editable() {
        // On the prompt's own line, put in front of the prompt text
        // rather than refusing.
        if win.w_cursor.lnum == Buf::current().b_prompt_start.mark.lnum {
            win.w_cursor.col = Buf::current().b_prompt_start.mark.col;
            cmd_arg.cmdchar = Key::ascii(b'P');
        } else {
            clear_op_beep(cmd_arg.op());
            return;
        }
    }

    let mut flags = 0;
    let mut dir;
    if fix_indent {
        dir = if cmd_arg.cmdchar.is(']') && cmd_arg.nchar.is('p') {
            FORWARD
        } else {
            BACKWARD
        };
        flags |= PUT_FIXINDENT;
    } else {
        dir = if cmd_arg.cmdchar.is('P')
            || ((cmd_arg.cmdchar.is('g') || cmd_arg.cmdchar.is('z')) && cmd_arg.nchar.is('P'))
        {
            BACKWARD
        } else {
            FORWARD
        };
    }
    prep_redo_cmd(cmd_arg);
    // `gp` leaves the cursor after the new text; `zp` puts a blockwise
    // register without widening the lines it lands on.
    if cmd_arg.cmdchar.is('g') {
        flags |= PUT_CURSEND;
    } else if cmd_arg.cmdchar.is('z') {
        flags |= PUT_BLOCK_INNER;
    }

    let was_visual = visual_active();
    let mut savereg: Option<YankReg> = None;
    let mut emptied = false;
    if was_visual {
        let regname = cmd_arg.op().regname;
        let keep_registers = cmd_arg.cmdchar.is('P');
        // Putting over a selection deletes it first, and that delete would
        // otherwise overwrite the very register being put.
        let clipoverwrite = (regname == b'+' || regname == b'*')
            && CB_FLAGS.get() & (OPT_CB_FLAG_UNNAMED | OPT_CB_FLAG_UNNAMEDPLUS) != 0;
        if regname == 0
            || regname == b'"'
            || clipoverwrite
            || regname.is_ascii_digit()
            || regname == b'-'
        {
            savereg = Some(copy_register(regname));
        }
        // The delete must not close or open folds under the selection.
        win.w_onebuf_opt.wo_fen = 0;
        // The condition is upstream's; only the `.` register on a
        // charwise selection skips the delete.
        if !visual_active() || visual_mode().is_line() || regname != b'.' {
            cmd_arg.cmdchar = Key::ascii(b'd');
            cmd_arg.nchar = Key::NUL;
            cmd_arg.op().regname = if keep_registers { b'_' } else { 0 };
            let silenced = Suppress::messages();
            nv_operator(cmd_arg);
            do_pending_operator(cmd_arg, 0, false);
            // The delete may have left the buffer with one empty line
            // that the put should not keep.
            emptied = Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY);
            drop(silenced);
            cmd_arg.op().regname = regname;
        }
        if visual_mode().is_line() {
            flags |= PUT_LINE;
        } else if visual_mode().is_char() {
            flags |= PUT_LINE_SPLIT;
        }
        if visual_mode().is_block() && dir == FORWARD {
            flags |= PUT_LINE_FORWARD;
        }
        // Put where the selection was, which is where the delete left the
        // cursor -- forwards only when it left it before the start.
        dir = BACKWARD;
        if (!visual_mode().is_line() && win.w_cursor.col < Buf::current().b_op_start.col)
            || (visual_mode().is_line() && win.w_cursor.lnum < Buf::current().b_op_start.lnum)
        {
            dir = FORWARD;
        }
        set_visual_active(true);
    }

    do_put(
        cmd_arg.op().regname,
        savereg.as_ref(),
        dir,
        cmd_arg.count1,
        flags,
    );
    if was_visual {
        if save_fen != 0 {
            win.w_onebuf_opt.wo_fen = 1;
        }
        // Leave `gv` naming what was just put.
        Buf::current().b_visual.vi_start = Buf::current().b_op_start;
        Buf::current().b_visual.vi_end = Buf::current().b_op_end;
        if P_SEL.first_byte() == b'e' {
            // A copy: `inc` reads the buffer the field is in.
            let mut end = Buf::current().b_visual.vi_end;
            inc(&mut end);
            Buf::current().b_visual.vi_end = end;
        }
    }
    let last = Buf::current().b_ml.ml_line_count;
    if emptied && Buf::current().lines().line(last).is_empty() {
        let _ = ml_delete_flags(Buf::current().b_ml.ml_line_count, ML_DEL_MESSAGE);
        deleted_lines(Buf::current().b_ml.ml_line_count + 1, 1);
        if win.w_cursor.lnum > Buf::current().b_ml.ml_line_count {
            win.w_cursor.lnum = Buf::current().b_ml.ml_line_count;
            coladvance(win, ColNr::MAX);
        }
    }
    auto_format(false, true);
}

/// `o` and `O` -- or, with a pending delete, the diff command, and with a
/// selection, "swap to the other corner".
pub(crate) fn nv_open(cmd_arg: &mut CmdArg) {
    if cmd_arg.op().op_type == OpType::Delete && cmd_arg.cmdchar.is('o') {
        // `do` is `:diffget`, not "delete, then open".
        clear_op(cmd_arg.op());
        debug_assert!(cmd_arg.opcount >= 0);
        nv_diffgetput(false, usize::try_from(cmd_arg.opcount).unwrap_or(0));
    } else if visual_active() {
        v_swap_corners(cmd_arg.cmdchar);
    } else if buf_is_prompt(current_buf())
        && Win::current().w_cursor.lnum < Buf::current().b_prompt_start.mark.lnum
    {
        clear_op_beep(cmd_arg.op());
    } else {
        n_opencmd(cmd_arg);
    }
}
