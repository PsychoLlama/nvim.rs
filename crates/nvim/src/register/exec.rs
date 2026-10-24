//! Recording a register, and replaying one.
//!
//! [`do_record`] is `q`: it turns recording on, and on the second `q` moves
//! what `get_recorded` collected into the register (appending for an
//! uppercase name).
//!
//! Replay is the other half, and the thing to know about it is that
//! **[`do_execreg`] does not run anything**: it stuffs the register's text
//! into the typeahead buffer so that the normal-mode loop reads it as if
//! typed. That is why the register is pushed *backwards*, why an interrupted
//! `@` leaves the rest of it queued, and why [`put_reedit_in_typebuf`] has to
//! re-enter Insert mode by queueing an `i`. [`insert_reg`] is CTRL-R in Insert
//! mode and [`cmdline_paste_reg`] CTRL-R on the command line; both go through
//! the read buffer rather than the typeahead one.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr::{self, until_nul};
use crate::mbyte::char_count;
use crate::types::RegLines;
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::c_int;

use super::*;
use crate::normal::visual_active;
use crate::types::{Failed, NUL};

/// Put `p` in register `regname` as a single charwise line, appending for an
/// uppercase name.
fn stuff_yank(regname: c_int, p: XString) -> Result<(), Failed> {
    if regname != 0 && !valid_yank_reg(regname, true) {
        return Err(Failed);
    }
    if regname == '_' as c_int {
        return Ok(()); // the black hole just drops it
    }

    get_yank_register(regname, YREG_YANK).write(|reg| {
        if is_append_register(regname) && reg.lines.is_set() {
            // Append to the register's last line rather than replacing it.
            // An unset register has no lines, and a set one at least one --
            // except one the provider set to none, which the append then
            // starts.
            let lines = reg.lines.lines_mut();
            match lines.last_mut() {
                Some(last) => last.push_bytes(&p),
                None => lines.push(p),
            }
        } else {
            reg.clear();
            reg.lines = RegLines::new(vec![p]);
            reg.kind = kMTCharWise;
        }
        reg.timestamp = os_time();
    });
    Ok(())
}

/// Build the `v:event` dictionary RecordingLeave sees, and fire the event.
///
/// `contents` is what was recorded, with its `K_SPECIAL` escaping already
/// undone. Runs arbitrary autocommands.
fn fire_recording_leave(regname: c_int, contents: &XString) {
    let mut save_v_event = SaveVEvent {
        sve_did_save: false,
        sve_hashtab: HashTab::new(),
    };
    let dict = get_v_event(&mut save_v_event);
    let _ = dict
        .edit()
        .add_str(b"regcontents", Some(contents.as_cstr()));
    let name = [regname as u8, NUL as u8];
    let _ = dict.edit().add_str(b"regname", Some(cstr::in_bytes(&name)));
    dict.edit().set_keys_readonly();
    apply_autocmds(
        AutoEvent::RecordingLeave,
        None,
        None,
        false,
        Buf::current_or_none(),
    );
    restore_v_event(dict, &mut save_v_event);
}

/// `q`: start recording into register `c`, or stop and store what was
/// recorded.
///
/// Answers `Err` for an invalid register name, or when the recording
/// produced nothing.
pub fn do_record(c: c_int) -> Result<(), Failed> {
    /// Which register the recording in progress goes into; kept across
    /// the two calls because `reg_recording` is cleared before the store.
    static regname: GlobalCell<c_int> = GlobalCell::new(0);

    if reg_recording.get() == 0 {
        // Start recording. A letter or a digit, and `"` -- note that
        // this is *not* `is_literal_register`, which also takes `*`/`+`.
        let alnum = (b'A' as c_int..=b'Z' as c_int).contains(&c)
            || (b'a' as c_int..=b'z' as c_int).contains(&c)
            || ascii_isdigit(c);
        if c < 0 || (!alnum && c != '"' as c_int) {
            return Err(Failed);
        }
        reg_recording.set(c);
        showmode();
        regname.set(c);
        apply_autocmds(
            AutoEvent::RecordingEnter,
            None,
            None,
            false,
            Buf::current_or_none(),
        );
        return Ok(());
    }

    // Stop recording.
    let mut recorded = get_recorded();
    let unescaped = unescape_ks_in(&mut recorded);
    recorded.truncate(unescaped);
    let p = XString::from_vec(recorded);
    // The autocommands this fires are the ones the caller allows.
    fire_recording_leave(regname.get(), &p);
    reg_recorded.set(reg_recording.get());
    reg_recording.set(0);
    if p_ch() == 0 || ui_has(kUIMessages) {
        showmode();
    } else {
        // Clear the "recording @a" message.
        msg(c"", 0);
    }
    // Recording into a register must not move `""`.
    let old_y_previous = y_previous.get();
    let retval = stuff_yank(regname.get(), p);
    y_previous.set(old_y_previous);
    retval
}

/// Queue `s` in the typeahead buffer so that it is read back as if typed.
///
/// `esc` escapes `K_SPECIAL` and turns mapping off; `colon` wraps the text in
/// `:` and `<CR>` so that it runs as an Ex command line. `s` is read as a C
/// string, to its first NUL.
fn put_in_typebuf(s: &[u8], esc: bool, colon: bool, silent: c_int) -> Result<(), Failed> {
    let mut retval = Ok(());
    put_reedit_in_typebuf(silent);

    // Pushed backwards: the `<CR>` first, then the text, then the `:`.
    if colon {
        retval = ins_typebuf(b"\n", REMAP_NONE, 0, true, silent != 0);
    }
    if retval.is_ok() {
        let text = until_nul(s);
        let escaped;
        let p: &[u8] = if esc {
            escaped = escape_ks_bytes(text);
            &escaped
        } else {
            text
        };
        let remap = if esc { REMAP_NONE } else { REMAP_YES };
        retval = ins_typebuf(p, remap, 0, true, silent != 0);
    }
    if colon && retval.is_ok() {
        retval = ins_typebuf(b":", REMAP_NONE, 0, true, silent != 0);
    }
    retval
}

/// Queue whatever will re-enter Insert mode, so that a register replayed from
/// Insert mode returns there afterwards.
///
/// `restart_edit` holds the mode as a key: `i`/`a`/`R`, or `V` for Virtual
/// Replace, which is two keys.
fn put_reedit_in_typebuf(silent: c_int) {
    if restart_edit.get() == NUL {
        return;
    }
    let mut buf: [u8; 3] = [0; 3];
    if restart_edit.get() == 'V' as c_int {
        buf[0] = b'g';
        buf[1] = b'R';
        buf[2] = NUL as u8;
    } else {
        // `I` means "insert at the first non-blank", which as a key is a
        // plain `i` -- the column has already been set.
        buf[0] = if restart_edit.get() == 'I' as c_int {
            b'i'
        } else {
            restart_edit.get() as u8
        };
        buf[1] = NUL as u8;
    }
    // `ins_typebuf` reads `buf` up to the NUL the branches above left.
    let queued = ins_typebuf(&buf, REMAP_NONE, 0, true, silent != 0);
    if queued.is_ok() {
        restart_edit.set(NUL);
    }
}

/// Join the line at `*idx` with the `\`-continuation lines in front of it.
///
/// The register is replayed backwards, so this walks *up* from `*idx` to the
/// first line that is not a continuation, concatenates the run, and leaves
/// `*idx` at that first line. A `"\ ` line is a comment inside a continuation
/// and contributes nothing. `*idx` must be above 0 and a line of `lines`.
fn execreg_line_continuation(lines: &[XString], idx: &mut usize) -> Vec<u8> {
    let mut cmd_start = *idx;
    debug_assert!(cmd_start > 0);
    let cmd_end = cmd_start;

    // Find the first line of the run.
    loop {
        cmd_start -= 1;
        if cmd_start == 0 {
            break;
        }
        let p = skip_white(&lines[cmd_start]);
        if cstr::byte_at(p, 0) != b'\\' && !is_continuation_comment(p) {
            break;
        }
    }

    // Then concatenate it, dropping each continuation's leading `\`.
    let mut joined = Vec::<u8>::new();
    joined.extend_from_slice(&lines[cmd_start]);
    for line in &lines[cmd_start + 1..=cmd_end] {
        let p = skip_white(line);
        if cstr::byte_at(p, 0) == b'\\' {
            joined.extend_from_slice(&p[1..]);
        }
    }
    *idx = cmd_start;
    joined
}

/// `line` past its leading white space, as `skipwhite` reads a C string.
fn skip_white(line: &[u8]) -> &[u8] {
    let white = line
        .iter()
        .take_while(|&&b| ascii_iswhite(c_int::from(b)))
        .count();
    &line[white..]
}

/// Whether `p` is a `"\ ` line -- a comment inside a `\`-continuation.
fn is_continuation_comment(p: &[u8]) -> bool {
    p.starts_with(b"\"\\ ")
}

/// Queue the *contents* of register `regname` in the typeahead buffer, so
/// that the main loop reads it as if typed.
///
/// `colon` wraps every line in `:`/`<CR>`, which is what `:@a` wants; `addcr`
/// adds a final `<CR>` even to a charwise register; `silent` keeps the
/// queued text out of `'showcmd'`.
pub fn do_execreg(regname: c_int, colon: c_int, addcr: c_int, silent: c_int) -> Result<(), Failed> {
    let mut regname = regname;
    if regname == '@' as c_int {
        // `@@` repeats the last `@`.
        if execreg_lastc.get() == NUL {
            emsg(gettext(c"E748: No previously used register"));
            return Err(Failed);
        }
        regname = execreg_lastc.get();
    }
    if regname == '%' as c_int || regname == '#' as c_int || !valid_yank_reg(regname, false) {
        emsg_invreg(regname);
        return Err(Failed);
    }
    execreg_lastc.set(regname);

    if regname == '_' as c_int {
        return Ok(()); // black hole: nothing to do
    }

    if regname == ':' as c_int {
        // The last command line, re-run. Control characters have to be
        // escaped with CTRL-V or the typeahead buffer would act on them.
        let Some(last) = last_cmdline.with(Clone::clone) else {
            emsg(gettext(e_nolastcmd));
            return Err(Failed);
        };
        new_last_cmdline.set(None);
        let p = escaped_with(
            &last,
            b"\x01\x02\x03\x04\x05\x06\x07\x08\t\n\x0B\x0C\r\x0E\x0F\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F",
            Ctrl_V as u8,
            false,
        );
        // A Visual-mode `@:` re-applies to the *current* selection, so
        // drop the `'<,'>` the command line was recorded with.
        let text = match p.strip_prefix(b"'<,'>") {
            Some(rest) if visual_active() => rest,
            _ => &p[..],
        };
        return put_in_typebuf(text, true, true, silent);
    }

    if regname == '=' as c_int {
        // Evaluating the expression register runs Vimscript, which this
        // function's own caller allows.
        let Some(p) = get_expr_line() else {
            return Err(Failed);
        };
        return put_in_typebuf(&p, true, colon != 0, silent);
    }

    if regname == '.' as c_int {
        let Some(p) = get_last_insert_save() else {
            emsg(gettext(e_noinstext));
            return Err(Failed);
        };
        return put_in_typebuf(&p, false, colon != 0, silent);
    }

    // A share of the register's lines: queueing them runs nothing, but the
    // borrow of the store stays as short as the rule says.
    let reg = get_yank_register(regname, YREG_PASTE).snapshot();
    if !reg.lines.is_set() {
        return Err(Failed);
    }
    let lines = reg.lines.as_slice();
    let remap = if colon != 0 { REMAP_NONE } else { REMAP_YES };
    put_reedit_in_typebuf(silent);

    // The typeahead buffer is a stack, so the register goes in last line
    // first.
    let mut retval = Ok(());
    let mut i = lines.len();
    while i > 0 {
        i -= 1;
        // A linewise register, and every line of a charwise one but the
        // last, is followed by a newline.
        let nl_failed = (reg.kind == kMTLineWise || i < lines.len() - 1 || addcr != 0)
            && ins_typebuf(b"\n", remap, 0, true, silent != 0).is_err();
        if nl_failed {
            return Err(Failed);
        }

        let line = &lines[i];
        let joined;
        let mut text: &[u8] = line;
        if colon != 0 && i > 0 {
            // A `\`-continued Ex command has to be joined back up before
            // it is queued; `i` is moved to the first line of the run.
            let p = skip_white(until_nul(line));
            if cstr::byte_at(p, 0) == b'\\' || is_continuation_comment(p) {
                joined = execreg_line_continuation(lines, &mut i);
                text = &joined;
            }
        }
        // Escaping reads the text as a C string, to its first NUL.
        let escaped = escape_ks_bytes(text);
        retval = ins_typebuf(&escaped, remap, 0, true, silent != 0);
        retval?;
        if colon != 0 && ins_typebuf(b":", remap, 0, true, silent != 0).is_err() {
            return Err(Failed);
        }
    }
    reg_executing.set(if regname == 0 { '"' as c_int } else { regname });
    pending_end_reg_executing.set(false);
    retval
}

/// CTRL-R in Insert mode: queue register `regname` in the read buffer.
///
/// `literally_arg` inserts the text as-is rather than as if typed; a register
/// whose contents are always literal ([`is_literal_register`]) forces it.
/// `reg` may be a register already fetched by the caller. May run arbitrary
/// Vimscript.
pub fn insert_reg(
    regname: c_int,
    reg: Option<&YankReg>,
    literally_arg: bool,
) -> Result<(), Failed> {
    let literally = literally_arg || is_literal_register(regname);

    // A register may be a long list of lines; let CTRL-C out.
    os_breakcheck();
    if got_int.get() {
        return Err(Failed);
    }
    if regname != NUL && !valid_yank_reg(regname, false) {
        return Err(Failed);
    }

    if regname == '.' as c_int {
        // The last insert is re-inserted rather than stuffed, so that it
        // can be repeated.
        return stuff_inserted(NUL, 1, 1);
    }

    // Reading a special register may run Vimscript, which the caller allows.
    if let Some(arg) = get_spec_reg(regname, true) {
        let Some(arg) = arg else {
            return Err(Failed);
        };
        stuffescaped(&arg, literally);
        return Ok(());
    }

    // The register as it is now: each `"-` line below goes in through a
    // put, which may run code that writes it.
    let reg = match reg {
        Some(reg) => reg.clone(),
        None => get_yank_register(regname, YREG_PASTE).snapshot(),
    };
    if !reg.lines.is_set() {
        return Err(Failed);
    }
    let lines = reg.lines.as_slice();
    for (i, line) in lines.iter().enumerate() {
        if regname == '-' as c_int && reg.kind == kMTCharWise {
            // The small-delete register goes in through `do_put`, so that
            // Replace mode's stack and the redo buffer stay right.
            let mut dir = BACKWARD;
            if State.get() & REPLACE_FLAG != 0 {
                // Saves the cursor line for undo.
                u_save_cursor()?;
                // As many characters as the register's first line holds, as
                // a C string reads it.
                let count = char_count(until_nul(&lines[0]));
                let _ = del_chars(c_int::try_from(count).unwrap_or(c_int::MAX), 1);
                let curpos = Win::current().w_cursor;
                if oneright().is_err() {
                    dir = FORWARD;
                }
                Win::current().w_cursor = curpos;
            }
            append_to_redobuff_char(Ctrl_R);
            append_to_redobuff_char(regname);
            // `do_put` takes the text from the register `regname` names; it
            // may run Vimscript, which the caller allows.
            do_put(regname, None, dir, 1, PUT_CURSEND as c_int);
        } else {
            stuffescaped(line, literally);
            if reg.kind == kMTLineWise || i < lines.len() - 1 {
                stuff_readbuf_char('\n' as c_int);
            }
        }
    }
    Ok(())
}

/// CTRL-R on the command line: insert register `regname` there.
pub fn cmdline_paste_reg(regname: c_int, literally_arg: bool, remcr: bool) -> bool {
    let literally = literally_arg || is_literal_register(regname);
    let reg = get_yank_register(regname, YREG_PASTE).snapshot();
    if !reg.lines.is_set() {
        return false;
    }
    let lines = reg.lines.as_slice();
    for (i, line) in lines.iter().enumerate() {
        cmdline_paste_str(line.as_cstr(), literally);
        // Add a <CR> between lines, unless the caller is going to.
        if i < lines.len() - 1 && !remcr {
            cmdline_paste_str(c"\r", literally);
        }
        os_breakcheck();
        if got_int.get() {
            return false;
        }
    }
    true
}
