//! Commands that change the buffer text or the cursor, including
//! `:normal`, which re-enters the normal-mode state machine.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::memline::MlFlags;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::smsg;
use crate::types::CmdIdx;
use core::ffi::{c_char, c_int};
use core::ptr;

use crate::ascii::ascii_isdigit;
use crate::charset::getdigits_int;
use crate::cursor::{check_cursor, check_cursor_col};

use crate::drawscreen::{UPD_VALID, redraw_later, setcursor_mayforce};
use crate::edit::{BeginlineOpts, beginline};
use crate::event::r#loop::process_events_until;
use crate::ex_cmds::{do_move, ex_copy, ex_substitute, ex_substitute_preview, global_exe};

use crate::ex_docmd::address::get_address;
use crate::ex_docmd::cmdline::do_cmdline;
use crate::ex_docmd::scan::{find_nextcmd, get_flags};
use crate::ex_docmd::{
    DoCmdOpts, EXFLAG_LIST, EXFLAG_NR, OPTION_MAGIC_OFF, OPTION_MAGIC_ON, REMAP_NONE, kMTLineWise,
};
use crate::ex_getln::getexline;
use crate::fold::{fold_create, fold_manual_allowed, has_folding, op_fold_range};
use crate::getchar::{beep_flush, stuff_empty, typeahead, vpeekc};

use crate::ex_docmd::state::{ex_no_reprint, exec_from_reg};
use crate::getchar::state::got_int;
use crate::keycodes::Ctrl_O;
use crate::lua::executor::ex_lua;
use crate::mark::{checkpcmark, setmark, setpcmark};
use crate::message::{e_argreq, e_empty_buffer, e_invrange, e_undobang_cannot_redo_or_move_branch};
use crate::search::state::magic_overruled;
use crate::startup::main_loop;
use crate::state::mode::{did_syncbind, virtual_op};

use crate::memline::{goto_byte, ml_clearmarked, ml_setmarked};

use crate::r#move::{
    cursor_correct, cursor_valid, scrolldown, scrollup, update_curswant, update_topline,
    validate_cursor,
};
use crate::normal::{end_visual_mode, get_vtopline, visual_active};
use crate::ops::{do_join, op_delete, op_shift};

use crate::option::{cpo_has, get_scrolloff_value};

use crate::os::input::os_breakcheck;
use crate::plines::plines_m_win_fill;
use crate::pos::MAXLNUM;
use crate::register::{do_execreg, do_put, op_yank};
use crate::search::{BACKWARD, FORWARD};
use crate::types::{
    ColNr, CpoFlag, ExArg, Failed, Handle, LineNr, NUL, OpArg, OpType, OptMagic, PUT_CURSLINE,
    PUT_FIXINDENT, PUT_LINE, Pos, int64_t, size_t, ssize_t,
};
use crate::ui::{ui_busy_start, ui_busy_stop, ui_flush};

use crate::undo::store::header_chain;
use crate::undo::{u_clearline, u_redo, u_undo};

use crate::winlayer::{Buf, Ea, Win, windows};

/// `:print`, `:number` and `:list`.
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_print(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    if Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY) {
        emsg(gettext(e_empty_buffer.as_ptr()));
    } else {
        let idx = args.cmdidx;
        let numbered = idx == CmdIdx::number || idx == CmdIdx::pound || args.flags & EXFLAG_NR != 0;
        let listed = idx == CmdIdx::list || args.flags & EXFLAG_LIST != 0;
        let mut line = args.line1;
        while line <= args.line2 && !got_int.get() {
            print_line(line, numbered, listed, line == args.line1);
            line += 1;
            os_breakcheck();
        }
        setpcmark();
        Win::current().w_cursor.lnum = args.line2;
        beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    }
    // Ex mode has just printed the line itself; it must not print it
    // again.
    ex_no_reprint.set(true);
}

/// `:goto` — the range is a byte offset, not a line number.
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_goto(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    goto_byte(args.line2 as c_int);
}

/// `:syncbind` — line up every 'scrollbind' window at the same relative
/// position.
///
/// # Safety
///
/// `_args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_syncbind(_args: *mut ExArg) {
    let old_linenr = Win::current().w_cursor.lnum;
    setpcmark();

    // The topline to use is the smallest that every bound window can
    // reach: one of them may be shorter than the rest.
    let mut vtopline: LineNr = 1;
    if Win::current().w_onebuf_opt.wo_scb != 0 {
        vtopline = get_vtopline(Win::current()) as LineNr;
        for wp in windows() {
            if wp.w_onebuf_opt.wo_scb != 0 && !wp.w_buffer.is_null() {
                let limit = unsafe { plines_m_win_fill(wp, 1, (*wp.w_buffer).b_ml.ml_line_count) }
                    as LineNr
                    - get_scrolloff_value(Win::current()) as LineNr;
                vtopline = vtopline.min(limit);
            }
        }
        vtopline = vtopline.max(1);
    }

    for mut wp in windows() {
        if wp.w_onebuf_opt.wo_scb != 0 {
            let y = vtopline as c_int - get_vtopline(wp);
            if y > 0 {
                scrollup(wp, y as LineNr, true);
            } else {
                scrolldown(wp, -(y as LineNr), true);
            }
            wp.w_scbind_pos = vtopline as c_int;
            redraw_later(wp, UPD_VALID);
            cursor_correct(wp);
            wp.w_redr_status = true;
        }
    }

    if Win::current().w_onebuf_opt.wo_scb != 0 {
        did_syncbind.set(true);
        checkpcmark();
        // The cursor moved with the scroll; CTRL-O puts it back.
        if old_linenr != Win::current().w_cursor.lnum {
            let ctrl_o: [c_char; 2] = [Ctrl_O as c_char, 0];
            let _ = ins_typebuf(
                ctrl_o.as_ptr() as *mut c_char,
                REMAP_NONE as c_int,
                0,
                true,
                false,
            );
        }
    }
}

/// `:=` — the line number, unless something follows it, in which case it
/// is `:lua`'s alias.
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_equal(args: *mut ExArg) {
    let mut args = unsafe { Ea::new(args) };
    if byte(args.arg) != NUL && byte(args.arg) != '|' as c_int {
        unsafe { ex_lua(args.raw()) };
    } else {
        args.nextcmd = unsafe { find_nextcmd(args.arg) };
        smsg!(0, "{}", args.line2 as int64_t);
    }
}

/// `:sleep` — the count is in seconds unless it is followed by `m`.
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_sleep(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    if cursor_valid(Win::current()) != 0 {
        setcursor_mayforce(Win::current(), true);
    }
    let mut len = args.line2 as int64_t;
    match byte(args.arg) {
        c if c == 'm' as c_int => {}
        c if c == NUL => len *= 1000,
        _ => {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = unsafe { c_str(args.arg) };
            semsg!("E475: Invalid argument: {arg}");
            return;
        }
    }
    do_sleep(len, args.forceit != 0);
}

/// Wait `msec` milliseconds, still serving events, and stop early on an
/// interrupt.
pub fn do_sleep(msec: int64_t, hide_cursor: bool) {
    if hide_cursor {
        ui_busy_start();
    }
    unsafe { ui_flush() };
    unsafe {
        process_events_until(main_loop.ptr(), (*main_loop.ptr()).events, msec, || {
            got_int.get()
        })
    };
    if got_int.get() {
        // Take the interrupt out of the typeahead.
        vpeekc();
    }
    if hide_cursor {
        ui_busy_stop();
    }
}

/// `:delete`, `:yank`, `:<` and `:>` — the four normal-mode operators that
/// have an Ex spelling.
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_operators(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    let mut oa: OpArg = unsafe { core::mem::zeroed() };
    clear_oparg(&raw mut oa);
    oa.regname = args.regname;
    oa.start.lnum = args.line1;
    oa.end.lnum = args.line2;
    oa.line_count = args.line2 - args.line1 + 1;
    oa.motion_type = kMTLineWise;
    // An Ex range is whole lines, so 'virtualedit' must not apply.
    virtual_op.set(Some(false));

    // `:yank` does not move the cursor, so it does not set the previous
    // context mark either.
    if args.cmdidx != CmdIdx::yank {
        setpcmark();
        Win::current().w_cursor.lnum = args.line1;
        beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    }
    if visual_active() {
        end_visual_mode();
    }

    match args.cmdidx {
        CmdIdx::delete => {
            oa.op_type = OpType::Delete;
            // `:delete` reports its own refusals; nothing more to do.
            let _ = unsafe { op_delete(&raw mut oa) };
        }
        CmdIdx::yank => {
            oa.op_type = OpType::Yank;
            unsafe { op_yank(&raw mut oa, true) };
        }
        _ => {
            // In a 'rightleft' window the two shift commands swap.
            oa.op_type = if (args.cmdidx == CmdIdx::rshift) as c_int
                ^ Win::current().w_onebuf_opt.wo_rl
                != 0
            {
                OpType::Rshift
            } else {
                OpType::Lshift
            };
            unsafe { op_shift(&raw mut oa, false, args.amount) };
        }
    }
    virtual_op.set(None);
    unsafe { ex_may_print(args.raw()) };
}

/// `:put`.
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_put(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    put_lines(args, PUT_LINE as c_int | PUT_CURSLINE as c_int);
}

/// `:iput` — the same, re-indenting what is put.
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_iput(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    put_lines(
        args,
        PUT_LINE as c_int | PUT_CURSLINE as c_int | PUT_FIXINDENT as c_int,
    );
}

/// `:0put` puts *above* line 1, which is spelled as a forced put at line 1.
fn put_lines(mut args: Ea, flags: c_int) {
    if args.line2 == 0 {
        args.line2 = 1;
        args.forceit = 1;
    }
    Win::current().w_cursor.lnum = args.line2;
    check_cursor_col(Win::current());
    unsafe {
        do_put(
            args.regname,
            ptr::null_mut(),
            if args.forceit != 0 {
                BACKWARD as c_int
            } else {
                FORWARD as c_int
            },
            1,
            flags,
        )
    };
}

/// `:copy` and `:move` — both take a destination address after the
/// command, which is why they parse one more address here.
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_copymove(args: *mut ExArg) {
    let mut args = unsafe { Ea::new(args) };
    let mut errormsg = None;
    let n = unsafe {
        get_address(
            args.raw(),
            args.arg_ptr(),
            args.addr_type,
            false,
            false,
            0,
            1,
            &mut errormsg,
        )
    };
    if args.arg.is_null() {
        if let Some(msg) = &errormsg {
            emsg(msg.as_ptr());
        }
        args.nextcmd = ptr::null_mut();
        return;
    }
    get_flags(args);

    // `MAXLNUM` is what `get_address` answers for "no address at all".
    if n == MAXLNUM || n < 0 || n > Buf::current().b_ml.ml_line_count {
        emsg(gettext(e_invrange.as_ptr()));
        return;
    }

    if args.cmdidx == CmdIdx::r#move {
        if do_move(args.line1, args.line2, n).is_err() {
            return;
        }
    } else {
        ex_copy(args.line1, args.line2, n);
    }
    u_clearline(Buf::current());
    beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    unsafe { ex_may_print(args.raw()) };
}

/// Print the current line, if the command carried an `l`, `p` or `#` flag.
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub unsafe fn ex_may_print(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    if args.flags != 0 {
        print_line(
            Win::current().w_cursor.lnum,
            args.flags & EXFLAG_NR != 0,
            args.flags & EXFLAG_LIST != 0,
            true,
        );
        ex_no_reprint.set(true);
    }
}

/// `:smagic` and `:snomagic` — `:substitute` with 'magic' forced either
/// way for the duration.
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_submagic(args: *mut ExArg) {
    let saved = force_magic(unsafe { Ea::new(args) });
    unsafe { ex_substitute(args) };
    magic_overruled.set(saved);
}

/// The 'inccommand' preview of the same.
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_submagic_preview(
    args: *mut ExArg,
    cmdpreview_ns: c_int,
    cmdpreview_bufnr: Handle,
) -> c_int {
    let saved = force_magic(unsafe { Ea::new(args) });
    let retv = unsafe { ex_substitute_preview(args, cmdpreview_ns, cmdpreview_bufnr) };
    magic_overruled.set(saved);
    retv
}

/// Override 'magic' for this command, answering what it was.
fn force_magic(args: Ea) -> OptMagic {
    let saved = magic_overruled.get();
    magic_overruled.set(if args.cmdidx == CmdIdx::smagic {
        OPTION_MAGIC_ON
    } else {
        OPTION_MAGIC_OFF
    } as OptMagic);
    saved
}

/// `:join`.
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_join(args: *mut ExArg) {
    let mut args = unsafe { Ea::new(args) };
    Win::current().w_cursor.lnum = args.line1;
    if args.line1 == args.line2 {
        // One line: join it with the next, unless a two-address range
        // said exactly one line, or there is no next line.
        if args.addr_count >= 2 {
            return;
        }
        if args.line2 == Buf::current().b_ml.ml_line_count {
            beep_flush();
            return;
        }
        args.line2 += 1;
    }
    let _ = do_join(
        (args.line2 as ssize_t - args.line1 as ssize_t + 1) as size_t,
        args.forceit == 0,
        true,
        true,
        true,
    );
    beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    unsafe { ex_may_print(args.raw()) };
}

/// `:@` — run the contents of a register as Ex commands.
///
/// The register's text goes into the typeahead, and command lines are read
/// out of it until it is empty. `prev_len` is what tells "empty" from
/// "there was already typeahead before this".
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_at(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    let prev_len = typeahead().len();
    Win::current().w_cursor.lnum = args.line2;
    check_cursor_col(Win::current());

    let mut c = ubyte(args.arg) as c_int;
    if c == NUL {
        c = '@' as c_int;
    }
    // 'cpoptions' `e` makes `:@` run the register's last line
    // immediately rather than leaving it on the command line.
    if do_execreg(c, 1, cpo_has(CpoFlag::EXECBUF) as c_int, 1).is_err() {
        beep_flush();
        return;
    }
    let save_efr = exec_from_reg.get();
    exec_from_reg.set(true);
    while !stuff_empty() || typeahead().len() > prev_len {
        let _ = unsafe {
            do_cmdline(
                ptr::null_mut(),
                Some(getexline),
                ptr::null_mut(),
                DoCmdOpts::NOWAIT | DoCmdOpts::VERBOSE,
            )
        };
    }
    exec_from_reg.set(save_efr);
}

/// `:undo`, and `:undo N` which goes to a numbered state.
///
/// `:undo! N` is different again: it *forgets* the states between here and
/// N rather than moving to it, so it can only go backwards along the
/// current branch.
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_undo(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    if args.addr_count != 1 {
        if args.forceit != 0 {
            u_undo_and_forget(1, true);
        } else {
            u_undo(1);
        }
        return;
    }
    let step = args.line2;
    if args.forceit == 0 {
        undo_time(step as c_int, false, false, true);
        return;
    }

    if step >= Buf::current().b_u_seq_cur as LineNr {
        emsg(gettext(e_undobang_cannot_redo_or_move_branch.as_ptr()));
        return;
    }
    // Count how many states back `step` is along this branch.
    let start = if Buf::current().b_u_curhead.is_none() {
        Buf::current().b_u_newhead
    } else {
        Buf::current().b_u_curhead
    };
    let mut count = 0;
    let mut uhp = ::core::ptr::null_mut();
    for header in header_chain(Buf::current(), start, |uh| uh.uh_next) {
        if header.uh_seq as LineNr <= step {
            uhp = header.raw();
            break;
        }
        count += 1;
    }
    // Running past it, or off the end, means `step` is on another
    // branch. Sequence 0 is the state before any change and is always
    // reachable.
    if step != 0 && (uhp.is_null() || (unsafe { (*uhp).uh_seq } as LineNr) < step) {
        emsg(gettext(e_undobang_cannot_redo_or_move_branch.as_ptr()));
        return;
    }
    u_undo_and_forget(count, true);
}

/// `:redo`.
///
/// # Safety
///
/// `_args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_redo(_args: *mut ExArg) {
    u_redo(1);
}

/// `:earlier` and `:later` — a count of changes, of seconds (`s`, `m`,
/// `h`, `d`) or of file writes (`f`).
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_later(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    let mut count = 0;
    let mut sec = false;
    let mut file = false;
    let mut p = args.arg;
    if byte(p) == NUL {
        count = 1;
    } else if ascii_isdigit(ubyte(p) as c_int) {
        count = unsafe { getdigits_int(&raw mut p, false, 0) };
        match ubyte(p) {
            b's' => {
                p = unsafe { p.add(1) };
                sec = true;
            }
            // The three multiplications are `int` arithmetic on a
            // number the user typed, and the C wraps: `:later
            // 100000000d` is nine orders of magnitude past `INT_MAX`.
            // `undo_time` clamps whatever comes out.
            b'm' => {
                p = unsafe { p.add(1) };
                sec = true;
                count = count.wrapping_mul(60);
            }
            b'h' => {
                p = unsafe { p.add(1) };
                sec = true;
                count = count.wrapping_mul(60 * 60);
            }
            b'd' => {
                p = unsafe { p.add(1) };
                sec = true;
                count = count.wrapping_mul(24 * 60 * 60);
            }
            b'f' => {
                p = unsafe { p.add(1) };
                file = true;
            }
            _ => {}
        }
    }
    if byte(p) != NUL {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str(args.arg) };
        semsg!("E475: Invalid argument: {arg}");
        return;
    }
    undo_time(
        if args.cmdidx == CmdIdx::earlier {
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
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_mark(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    if byte(args.arg) == NUL {
        emsg(gettext(e_argreq.as_ptr()));
        return;
    }
    if byte_at(args.arg, 1) != NUL {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str(args.arg) };
        semsg!("E488: Trailing characters: {arg}");
        return;
    }
    // The mark is set at the first non-blank of the addressed line, so
    // the cursor goes there and comes back.
    let pos = Win::current().w_cursor;
    Win::current().w_cursor.lnum = args.line2;
    beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    if unsafe { setmark(*args.arg as c_int) }.is_err() {
        emsg(gettext(
            c"E191: Argument must be a letter or forward/backward quote".as_ptr(),
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
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_fold(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    if fold_manual_allowed(true) != 0 {
        fold_create(Win::current(), range_start(args), range_end(args));
    }
}

/// `:foldopen` and `:foldclose`.
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_foldopen(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    op_fold_range(
        range_start(args),
        range_end(args),
        (args.cmdidx == CmdIdx::foldopen) as c_int,
        args.forceit,
        false,
    );
}

/// The range's first line, as a position in column 1.
fn range_start(args: Ea) -> Pos {
    Pos {
        lnum: args.line1,
        col: 1 as ColNr,
        coladd: 0 as ColNr,
    }
}

/// The range's last line, likewise.
fn range_end(args: Ea) -> Pos {
    Pos {
        lnum: args.line2,
        col: 1 as ColNr,
        coladd: 0 as ColNr,
    }
}

/// `:folddoopen` and `:folddoclosed` — run a command on every line that is
/// (or is not) inside a closed fold.
///
/// # Safety
///
/// `args` must point at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn ex_folddo(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    let want_closed = (args.cmdidx == CmdIdx::folddoclosed) as c_int;
    let mut lnum = args.line1;
    while lnum <= args.line2 {
        if has_folding(Win::current(), lnum, None, None) as c_int == want_closed {
            ml_setmarked(lnum);
        }
        lnum += 1;
    }
    unsafe { global_exe(args.arg) };
    ml_clearmarked();
}

/// `clear_oparg()` as checked code.
pub(super) fn clear_oparg(op: *mut OpArg) {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::ops::clear_oparg(op) }
}

/// `emsg()` as checked code.
pub(super) fn emsg(s: *const c_char) -> bool {
    // SAFETY: a NUL-terminated message.
    unsafe { crate::message::emsg_ptr(s) }
}

/// `gettext()` as checked code.
pub(super) fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    // SAFETY: a NUL-terminated message; `gettext` answers one too.
    unsafe { crate::os::cshim::gettext_ptr(__msgid).as_ptr().cast_mut() }
}

/// `ins_typebuf()` as checked code.
pub(super) fn ins_typebuf(
    str: *mut c_char,
    noremap: c_int,
    offset: c_int,
    nottyped: bool,
    silent: bool,
) -> Result<(), Failed> {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::getchar::ins_typebuf(str, noremap, offset, nottyped, silent) }
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
    // SAFETY: reads the editor's own state, which exists from startup to exit.
    unsafe { crate::ui::ui_cursor_shape() }
}

/// `undo_time()` as checked code.
fn undo_time(step: c_int, sec: bool, file: bool, absolute: bool) {
    crate::undo::undo_time(step, sec, file, absolute)
}

/// `utfc_ptr2len()` as checked code.
pub(super) fn utfc_ptr2len(p: *const c_char) -> c_int {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::mbyte::utfc_ptr2len(p) }
}

/// The byte `p` points at, as the C's `*p` reads it.
pub(super) fn byte(p: *const c_char) -> c_int {
    // SAFETY: a NUL-terminated string the command line owns.
    unsafe { *p as c_int }
}

/// The byte `p` points at, unsigned, as the C's `(uint8_t)*p` reads it.
fn ubyte(p: *const c_char) -> u8 {
    // SAFETY: a NUL-terminated string the command line owns.
    unsafe { *p as u8 }
}

/// The byte at `p[i]`, as the C's `*(p + i)` reads it.
fn byte_at(p: *const c_char, i: isize) -> c_int {
    // SAFETY: an offset within the NUL-terminated string `p` points into.
    unsafe { *p.offset(i) as c_int }
}
