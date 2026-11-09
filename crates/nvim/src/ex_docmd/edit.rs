//! Commands that change the buffer text or the cursor, including
//! `:normal`, which re-enters the normal-mode state machine.
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::getchar::Remap;
use crate::getchar::ins_typebuf;
use crate::memline::MlFlags;
use crate::message::emsg;
use crate::message_fmt::msg_bytes;
use crate::narrow::number_as_lnum;
use crate::ops::run_ex_operator;
use crate::os::cshim::gettext;
use crate::os::time::sleep_serving_events;
use crate::register::put_at_cursor;
use crate::semsg;
use crate::smsg;
use crate::types::CmdIdx;
use core::ffi::c_int;

use crate::charset::getdigits_int_at;
use crate::cursor::{check_cursor, check_cursor_col};

use crate::drawscreen::{UPD_VALID, redraw_later, setcursor_mayforce};
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_cmds::{do_move, ex_copy, ex_substitute, ex_substitute_preview, global_exe};

use crate::ex_docmd::address::get_address;
use crate::ex_docmd::cmdline::do_cmdline;
use crate::ex_docmd::scan::get_flags;
use crate::ex_docmd::source::LineSource;
use crate::ex_docmd::{
    DoCmdOpts, EXFLAG_LIST, EXFLAG_NR, MT_LINE_WISE, OPTION_MAGIC_OFF, OPTION_MAGIC_ON,
};
use crate::fold::{fold_create, fold_manual_allowed, op_fold_range};
use crate::getchar::{beep_flush, stuff_empty, typeahead, vpeekc};

use crate::ex_docmd::state::{ex_no_reprint, exec_from_reg};
use crate::getchar::state::GOT_INT;
use crate::lua::executor::ex_lua;
use crate::mark::{checkpcmark, setmark, setpcmark};
use crate::message::{E_ARGREQ, E_EMPTY_BUFFER, E_INVRANGE, E_UNDOBANG_CANNOT_REDO_OR_MOVE_BRANCH};
use crate::search::state::magic_overruled;
use crate::state::mode::{DID_SYNCBIND, VIRTUAL_OP};

use crate::memline::{goto_byte, ml_clearmarked, ml_setmarked};

use crate::r#move::{
    cursor_correct, cursor_valid, scrolldown, scrollup, update_curswant, update_topline,
    validate_cursor,
};
use crate::normal::{end_visual_mode, get_vtopline, visual_active};
use crate::ops::do_join;

use crate::option::{cpo_has, get_scrolloff_value};

use crate::normal::defs::OpArg;
use crate::ops::OpType;
use crate::option::defs::CpoFlag;
use crate::os::input::os_breakcheck;
use crate::os::libc::int64_t;
use crate::plines::plines_m_win_fill;
use crate::pos::MAXLNUM;
use crate::regexp::OptMagic;
use crate::register::defs::{PUT_CURSLINE, PUT_FIXINDENT, PUT_LINE, PutFlags};
use crate::register::do_execreg;
use crate::search::{BACKWARD, FORWARD};
use crate::types::{ColNr, ExArg, Handle, LineNr, Pos};
use crate::ui::{ui_busy_start, ui_busy_stop, ui_flush};

use crate::undo::{u_clearline, u_redo, u_undo};

use crate::winlayer::{Buf, Win, windows};

/// `:print`, `:number` and `:list`.
pub(crate) fn ex_print(excmd: &mut ExArg) {
    if Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY) {
        emsg(gettext(E_EMPTY_BUFFER));
    } else {
        let idx = excmd.cmdidx;
        let numbered =
            idx == CmdIdx::number || idx == CmdIdx::pound || excmd.flags & EXFLAG_NR != 0;
        let listed = idx == CmdIdx::list || excmd.flags & EXFLAG_LIST != 0;
        let mut line = excmd.line1;
        while line <= excmd.line2 && !GOT_INT.get() {
            print_line(line, numbered, listed, line == excmd.line1);
            line += 1;
            os_breakcheck();
        }
        setpcmark();
        Win::current().w_cursor.lnum = excmd.line2;
        beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    }
    // Ex mode has just printed the line itself; it must not print it
    // again.
    ex_no_reprint.set(true);
}

/// `:goto` — the range is a byte offset, not a line number.
pub(crate) fn ex_goto(excmd: &mut ExArg) {
    goto_byte(excmd.number2);
}

/// `:syncbind` — line up every 'scrollbind' window at the same relative
/// position.
pub(crate) fn ex_syncbind(_excmd: &mut ExArg) {
    let old_linenr = Win::current().w_cursor.lnum;
    setpcmark();

    // The topline to use is the smallest that every bound window can
    // reach: one of them may be shorter than the rest.
    let mut vtopline: LineNr = LineNr::ONE;
    if Win::current().w_onebuf_opt.wo_scb != 0 {
        vtopline = LineNr::new(get_vtopline(Win::current()));
        for wp in windows() {
            if wp.w_onebuf_opt.wo_scb != 0 && !wp.w_buffer.is_null() {
                let limit = LineNr::new(plines_m_win_fill(
                    wp,
                    LineNr::ONE,
                    wp.w_buffer.b_ml.ml_line_count,
                )) - number_as_lnum(get_scrolloff_value(Win::current()));
                vtopline = vtopline.min(LineNr::new(limit));
            }
        }
        vtopline = vtopline.max(LineNr::ONE);
    }

    for mut wp in windows() {
        if wp.w_onebuf_opt.wo_scb != 0 {
            let y = vtopline.get() - get_vtopline(wp);
            if y > 0 {
                scrollup(wp, y, true);
            } else {
                scrolldown(wp, -y, true);
            }
            wp.w_scbind_pos = vtopline.get();
            redraw_later(wp, UPD_VALID);
            cursor_correct(wp);
            wp.w_redr_status = true;
        }
    }

    if Win::current().w_onebuf_opt.wo_scb != 0 {
        DID_SYNCBIND.set(true);
        checkpcmark();
        // The cursor moved with the scroll; CTRL-O puts it back.
        if old_linenr != Win::current().w_cursor.lnum {
            let _ = ins_typebuf(b"\x0f", Remap::No, 0, true, false);
        }
    }
}

/// `:=` — the line number, unless something follows it, in which case it
/// is `:lua`'s alias.
pub(crate) fn ex_equal(excmd: &mut ExArg) {
    if excmd.line.byte_at(excmd.line.arg) != 0 && excmd.line.byte_at(excmd.line.arg) != b'|' {
        ex_lua(excmd);
    } else {
        excmd.line.next = excmd.line.find_next(excmd.line.arg);
        smsg!(0, "{}", i64::from(excmd.line2));
    }
}

/// `:sleep` — the count is in seconds unless it is followed by `m`.
pub(crate) fn ex_sleep(excmd: &mut ExArg) {
    if cursor_valid(Win::current()) != 0 {
        setcursor_mayforce(Win::current(), true);
    }
    let mut len = i64::from(excmd.number2);
    match excmd.line.byte_at(excmd.line.arg) {
        b'm' => {}
        0 => len *= 1000,
        _ => {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = msg_bytes(excmd.line.arg());
            semsg!("E475: Invalid argument: {arg}");
            return;
        }
    }
    do_sleep(len, excmd.forceit);
}

/// Wait `msec` milliseconds, still serving events, and stop early on an
/// interrupt.
pub fn do_sleep(msec: int64_t, hide_cursor: bool) {
    if hide_cursor {
        ui_busy_start();
    }
    ui_flush();
    sleep_serving_events(msec);
    if GOT_INT.get() {
        // Take the interrupt out of the typeahead.
        vpeekc().code();
    }
    if hide_cursor {
        ui_busy_stop();
    }
}

/// `:delete`, `:yank`, `:<` and `:>` — the four normal-mode operators that
/// have an Ex spelling.
pub(crate) fn ex_operators(excmd: &mut ExArg) {
    let line = |lnum| Pos {
        lnum,
        ..Pos::default()
    };
    let mut oa = OpArg {
        regname: excmd.regname,
        start: line(excmd.line1),
        end: line(excmd.line2),
        line_count: (excmd.line2 - excmd.line1 + 1),
        motion_type: MT_LINE_WISE,
        ..OpArg::default()
    };
    // An Ex range is whole lines, so 'virtualedit' must not apply.
    VIRTUAL_OP.set(Some(false));

    // `:yank` does not move the cursor, so it does not set the previous
    // context mark either.
    if excmd.cmdidx != CmdIdx::yank {
        setpcmark();
        Win::current().w_cursor.lnum = excmd.line1;
        beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    }
    if visual_active() {
        end_visual_mode();
    }

    match excmd.cmdidx {
        CmdIdx::delete => {
            oa.op_type = OpType::Delete;
        }
        CmdIdx::yank => {
            oa.op_type = OpType::Yank;
        }
        _ => {
            // In a 'rightleft' window the two shift commands swap.
            oa.op_type = if c_int::from(excmd.cmdidx == CmdIdx::rshift)
                ^ Win::current().w_onebuf_opt.wo_rl
                != 0
            {
                OpType::Rshift
            } else {
                OpType::Lshift
            };
        }
    }
    run_ex_operator(&mut oa, excmd.amount);
    VIRTUAL_OP.set(None);
    ex_may_print(excmd);
}

/// `:put`.
pub(crate) fn ex_put(excmd: &mut ExArg) {
    put_lines(excmd, PUT_LINE | PUT_CURSLINE);
}

/// `:iput` — the same, re-indenting what is put.
pub(crate) fn ex_iput(excmd: &mut ExArg) {
    put_lines(excmd, PUT_LINE | PUT_CURSLINE | PUT_FIXINDENT);
}

/// `:0put` puts *above* line 1, which is spelled as a forced put at line 1.
fn put_lines(excmd: &mut ExArg, flags: PutFlags) {
    if excmd.line2 == LineNr::ZERO {
        excmd.line2 = LineNr::ONE;
        excmd.forceit = true;
    }
    Win::current().w_cursor.lnum = excmd.line2;
    check_cursor_col(Win::current());
    let dir = if excmd.forceit { BACKWARD } else { FORWARD };
    put_at_cursor(excmd.regname, dir, 1, flags);
}

/// `:copy` and `:move` — both take a destination address after the
/// command, which is why they parse one more address here.
pub(crate) fn ex_copymove(excmd: &mut ExArg) {
    // The scan walks a cursor of its own; see `parse_cmd_address`.
    let (cmdidx, addr_type) = (excmd.cmdidx, excmd.addr_type);
    let mut at = excmd.line.arg;
    let read = get_address(
        Some(cmdidx),
        excmd.line.buffer_mut(),
        &mut at,
        addr_type,
        false,
        false,
        false,
        1,
    );
    let n = match read {
        Ok(n) => n,
        Err(refused) => {
            refused.report();
            excmd.line.next = None;
            return;
        }
    };
    excmd.line.arg = at;
    get_flags(excmd);

    // `MAXLNUM` is what `get_address` answers for "no address at all".
    if n == MAXLNUM || n < LineNr::ZERO || n > Buf::current().b_ml.ml_line_count {
        emsg(gettext(E_INVRANGE));
        return;
    }

    if excmd.cmdidx == CmdIdx::r#move {
        if do_move(excmd.line1, excmd.line2, n).is_err() {
            return;
        }
    } else {
        ex_copy(excmd.line1, excmd.line2, n);
    }
    u_clearline(Buf::current());
    beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    ex_may_print(excmd);
}

/// Print the current line, if the command carried an `l`, `p` or `#` flag.
pub fn ex_may_print(excmd: &mut ExArg) {
    if excmd.flags != 0 {
        print_line(
            Win::current().w_cursor.lnum,
            excmd.flags & EXFLAG_NR != 0,
            excmd.flags & EXFLAG_LIST != 0,
            true,
        );
        ex_no_reprint.set(true);
    }
}

/// `:smagic` and `:snomagic` — `:substitute` with 'magic' forced either
/// way for the duration.
pub(crate) fn ex_submagic(excmd: &mut ExArg) {
    let saved = force_magic(excmd);
    ex_substitute(excmd);
    magic_overruled.set(saved);
}

/// The 'inccommand' preview of the same.
pub(crate) fn ex_submagic_preview(
    excmd: &mut ExArg,
    cmdpreview_ns: c_int,
    cmdpreview_bufnr: Handle,
) -> c_int {
    let saved = force_magic(excmd);
    let retv = ex_substitute_preview(excmd, cmdpreview_ns, cmdpreview_bufnr);
    magic_overruled.set(saved);
    retv
}

/// Override 'magic' for this command, answering what it was.
fn force_magic(excmd: &mut ExArg) -> OptMagic {
    let saved = magic_overruled.get();
    magic_overruled.set(if excmd.cmdidx == CmdIdx::smagic {
        OPTION_MAGIC_ON
    } else {
        OPTION_MAGIC_OFF
    } as OptMagic);
    saved
}

/// `:join`.
pub(crate) fn ex_join(excmd: &mut ExArg) {
    Win::current().w_cursor.lnum = excmd.line1;
    if excmd.line1 == excmd.line2 {
        // One line: join it with the next, unless a two-address range
        // said exactly one line, or there is no next line.
        if excmd.addr_count >= 2 {
            return;
        }
        if excmd.line2 == Buf::current().b_ml.ml_line_count {
            beep_flush();
            return;
        }
        excmd.line2 += 1;
    }
    let _ = do_join(
        (excmd.line2.get() as isize - excmd.line1.get() as isize + 1).cast_unsigned(),
        !excmd.forceit,
        true,
        true,
        true,
    );
    beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    ex_may_print(excmd);
}

/// `:@` — run the contents of a register as Ex commands.
///
/// The register's text goes into the typeahead, and command lines are read
/// out of it until it is empty. `prev_len` is what tells "empty" from
/// "there was already typeahead before this".
pub(crate) fn ex_at(excmd: &mut ExArg) {
    let prev_len = typeahead().len();
    Win::current().w_cursor.lnum = excmd.line2;
    check_cursor_col(Win::current());

    let mut c = excmd.line.byte_at(excmd.line.arg);
    if c == 0 {
        c = b'@';
    }
    // 'cpoptions' `e` makes `:@` run the register's last line
    // immediately rather than leaving it on the command line.
    if do_execreg(c, true, cpo_has(CpoFlag::EXECBUF), true).is_err() {
        beep_flush();
        return;
    }
    let save_efr = exec_from_reg.get();
    exec_from_reg.set(true);
    while !stuff_empty() || typeahead().len() > prev_len {
        let flags = DoCmdOpts::NOWAIT | DoCmdOpts::VERBOSE;
        let _ = do_cmdline(None, Some(&mut LineSource::Typed), flags);
    }
    exec_from_reg.set(save_efr);
}

/// `:undo`, and `:undo N` which goes to a numbered state.
///
/// `:undo! N` is different again: it *forgets* the states between here and
/// N rather than moving to it, so it can only go backwards along the
/// current branch.
pub(crate) fn ex_undo(excmd: &mut ExArg) {
    if excmd.addr_count != 1 {
        if excmd.forceit {
            u_undo_and_forget(1, true);
        } else {
            u_undo(1);
        }
        return;
    }
    let step = excmd.number2;
    if !excmd.forceit {
        undo_time(step, false, false, true);
        return;
    }

    if step >= Buf::current().b_u_seq_cur {
        emsg(gettext(E_UNDOBANG_CANNOT_REDO_OR_MOVE_BRANCH));
        return;
    }
    // Count how many states back `step` is along this branch.
    let start = if Buf::current().b_u_curhead.is_none() {
        Buf::current().b_u_newhead
    } else {
        Buf::current().b_u_curhead
    };
    let mut count = 0;
    let mut reached = None;
    for link in Buf::current().header_chain(start, |uh| uh.next) {
        if link.seq() <= step {
            reached = Some(link.seq());
            break;
        }
        count += 1;
    }
    // Running past it, or off the end, means `step` is on another
    // branch. Sequence 0 is the state before any change and is always
    // reachable.
    if step != 0 && reached.is_none_or(|seq| seq < step) {
        emsg(gettext(E_UNDOBANG_CANNOT_REDO_OR_MOVE_BRANCH));
        return;
    }
    u_undo_and_forget(count, true);
}

/// `:redo`.
pub(crate) fn ex_redo(_excmd: &mut ExArg) {
    u_redo(1);
}

/// `:earlier` and `:later` — a count of changes, of seconds (`s`, `m`,
/// `h`, `d`) or of file writes (`f`).
pub(crate) fn ex_later(excmd: &mut ExArg) {
    let mut count = 0;
    let mut sec = false;
    let mut file = false;
    let mut at = excmd.line.arg;
    if excmd.line.byte_at(at) == 0 {
        count = 1;
    } else if excmd.line.byte_at(at).is_ascii_digit() {
        let (n, past) = getdigits_int_at(excmd.line.buffer_mut(), at, false, 0);
        count = n;
        at = past;
        match excmd.line.byte_at(at) {
            b's' => {
                at += 1;
                sec = true;
            }
            // The three multiplications are `int` arithmetic on a
            // number the user typed, and the C wraps: `:later
            // 100000000d` is nine orders of magnitude past `INT_MAX`.
            // `undo_time` clamps whatever comes out.
            b'm' => {
                at += 1;
                sec = true;
                count = count.wrapping_mul(60);
            }
            b'h' => {
                at += 1;
                sec = true;
                count = count.wrapping_mul(60 * 60);
            }
            b'd' => {
                at += 1;
                sec = true;
                count = count.wrapping_mul(24 * 60 * 60);
            }
            b'f' => {
                at += 1;
                file = true;
            }
            _ => {}
        }
    }
    if excmd.line.byte_at(at) != 0 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = msg_bytes(excmd.line.arg());
        semsg!("E475: Invalid argument: {arg}");
        return;
    }
    undo_time(
        if excmd.cmdidx == CmdIdx::earlier {
            count.wrapping_neg()
        } else {
            count
        },
        sec,
        file,
        false,
    );
}

/// `:mark` and `:k`.
pub(crate) fn ex_mark(excmd: &mut ExArg) {
    if excmd.line.byte_at(excmd.line.arg) == 0 {
        emsg(gettext(E_ARGREQ));
        return;
    }
    if excmd.line.byte_at(excmd.line.arg + 1) != 0 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = msg_bytes(excmd.line.arg());
        semsg!("E488: Trailing characters: {arg}");
        return;
    }
    // The mark is set at the first non-blank of the addressed line, so
    // the cursor goes there and comes back.
    let pos = Win::current().w_cursor;
    Win::current().w_cursor.lnum = excmd.line2;
    beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    if setmark(excmd.line.byte_at(excmd.line.arg)).is_err() {
        emsg(gettext(
            c"E191: Argument must be a letter or forward/backward quote",
        ));
    }
    Win::current().w_cursor = pos;
}

/// Put the cursor and the window back in agreement after a command that
/// moved either.
pub fn update_topline_cursor() {
    check_cursor(Win::current());
    update_topline(Win::current());
    if Win::current().w_onebuf_opt.wo_wrap == 0 {
        validate_cursor(Win::current());
    }
    update_curswant();
}

/// `:fold`.
pub(crate) fn ex_fold(excmd: &mut ExArg) {
    if fold_manual_allowed(true).is_ok() {
        fold_create(Win::current(), range_start(excmd), range_end(excmd));
    }
}

/// `:foldopen` and `:foldclose`.
pub(crate) fn ex_foldopen(excmd: &mut ExArg) {
    op_fold_range(
        range_start(excmd),
        range_end(excmd),
        c_int::from(excmd.cmdidx == CmdIdx::foldopen),
        c_int::from(excmd.forceit),
        false,
    );
}

/// The range's first line, as a position in column 1.
fn range_start(excmd: &mut ExArg) -> Pos {
    Pos {
        lnum: excmd.line1,
        col: ColNr::new(1),
        coladd: 0,
    }
}

/// The range's last line, likewise.
fn range_end(excmd: &mut ExArg) -> Pos {
    Pos {
        lnum: excmd.line2,
        col: ColNr::new(1),
        coladd: 0,
    }
}

/// `:folddoopen` and `:folddoclosed` — run a command on every line that is
/// (or is not) inside a closed fold.
pub(crate) fn ex_folddo(excmd: &mut ExArg) {
    let want_closed = c_int::from(excmd.cmdidx == CmdIdx::folddoclosed);
    let mut lnum = excmd.line1;
    while lnum <= excmd.line2 {
        if c_int::from(Win::current().has_folding(lnum, None, None)) == want_closed {
            ml_setmarked(lnum);
        }
        lnum += 1;
    }
    global_exe(excmd.line.arg());
    ml_clearmarked();
}

/// `print_line()` as checked code.
fn print_line(lnum: LineNr, use_number: bool, list: bool, first: bool) {
    crate::ex_cmds::print_line(lnum, use_number, list, first)
}

/// `u_undo_and_forget()` as checked code.
fn u_undo_and_forget(count: c_int, do_buf_event: bool) -> bool {
    crate::undo::u_undo_and_forget(count, do_buf_event)
}

/// `ui_cursor_shape()` as checked code.
pub(super) fn ui_cursor_shape() {
    crate::ui::ui_cursor_shape()
}

/// `undo_time()` as checked code.
fn undo_time(step: c_int, sec: bool, file: bool, absolute: bool) {
    crate::undo::undo_time(step, sec, file, absolute)
}
