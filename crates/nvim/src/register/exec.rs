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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::winlayer::Win;
use core::ffi::{c_char, c_int, c_void};

use super::*;
use crate::normal::visual_active;
use crate::types::{Failed, NUL};

/// Put the allocated string `p` in register `regname` as a single charwise
/// line, appending for an uppercase name.
///
/// Takes ownership of `p` on every path.
///
/// # Safety
/// `p` must be an allocated, NUL-terminated string.
unsafe fn stuff_yank(regname: c_int, p: *mut c_char) -> Result<(), Failed> {
    // SAFETY: `valid_yank_reg` only looks the name up.
    if regname != 0 && !unsafe { valid_yank_reg(regname, true) } {
        // SAFETY: `p` is the allocated string this function took over.
        unsafe { xfree(p as *mut c_void) };
        return Err(Failed);
    }
    if regname == '_' as c_int {
        // SAFETY: as above -- the black hole just drops it.
        unsafe { xfree(p as *mut c_void) };
        return Ok(());
    }

    // SAFETY: `p` is NUL-terminated, and a valid register name answers a
    // live register.
    let (plen, reg) = unsafe { (strlen(p), get_yank_register(regname, YREG_YANK)) };
    // SAFETY: `reg` is that live register, so its `y_array` is there to test.
    if is_append_register(regname) && !unsafe { (*reg).y_array.is_null() } {
        // Append to the register's last line rather than replacing it.
        // SAFETY: a non-null `y_array` holds `y_size` lines, so `pp` names
        // the last of them; `tmp` is sized for that line's bytes, `p`'s
        // `plen`, and a NUL, and both originals are freed once copied.
        let last = unsafe { (*reg).y_size }.wrapping_sub(1);
        let pp = unsafe { (*reg).y_array.add(last) };
        let tmplen = unsafe { *pp }.len().wrapping_add(plen);
        let tmp = unsafe { xmalloc(tmplen.wrapping_add(1)) } as *mut c_char;
        unsafe { memcpy(tmp as *mut c_void, (*pp).data().cast(), (*pp).len()) };
        unsafe { memcpy(tmp.add((*pp).len()).cast(), p as *const c_void, plen) };
        unsafe { *tmp.add(tmplen) = NUL as c_char };
        unsafe { xfree(p as *mut c_void) };
        unsafe { xfree((*pp).data() as *mut c_void) };
        unsafe { *pp = String_0::from_raw_parts(tmp, tmplen) };
    } else {
        // SAFETY: `reg` is a live register. It is emptied and then given a
        // one-element array holding `p`, whose ownership passes to it.
        unsafe { free_register(reg) };
        unsafe { (*reg).additional_data = ::core::ptr::null_mut() };
        unsafe { (*reg).y_array = xmalloc(::core::mem::size_of::<String_0>()) as *mut String_0 };
        unsafe { *(*reg).y_array = String_0::from_raw_parts(p, plen) };
        unsafe { (*reg).y_size = 1 };
        unsafe { (*reg).y_type = kMTCharWise };
    }
    // SAFETY: `reg` is a live register.
    unsafe { (*reg).timestamp = os_time() };
    Ok(())
}

/// Build the `v:event` dictionary RecordingLeave sees, and fire the event.
///
/// `contents` is what was recorded, with its `K_SPECIAL` escaping already
/// undone; null when the recording produced nothing.
///
/// # Safety
/// `contents` must be null or NUL-terminated. Runs arbitrary autocommands.
unsafe fn fire_recording_leave(regname: c_int, contents: *mut c_char) {
    let mut save_v_event = save_v_event_T {
        sve_did_save: false,
        sve_hashtab: hashtab_T {
            ht_mask: 0,
            ht_used: 0,
            ht_filled: 0,
            ht_changed: 0,
            ht_locked: 0,
            ht_array: ::core::ptr::null_mut(),
            ht_smallarray: [hashitem_T {
                hi_hash: 0,
                hi_key: ::core::ptr::null_mut(),
            }; 16],
        },
    };
    // SAFETY: `save_v_event` is a writable local, which is saved into here
    // and read back by `restore_v_event` at the end.
    let dict = unsafe { get_v_event(&raw mut save_v_event) };
    if !contents.is_null() {
        // SAFETY: `dict` is that event dictionary, the key is a literal of
        // the length given, and the caller promises a NUL-terminated value.
        let _ = unsafe { tv_dict_add_str(dict, c"regcontents".as_ptr(), 11, contents) };
    }
    let mut buf: [c_char; 67] = [0; 67];
    buf[0] = regname as c_char;
    buf[1] = NUL as c_char;
    // SAFETY: as above; `buf` is NUL-terminated by the line before.
    let _ = unsafe { tv_dict_add_str(dict, c"regname".as_ptr(), 7, buf.as_mut_ptr()) };
    // SAFETY: `dict` is the event dictionary.
    unsafe { tv_dict_set_keys_readonly(dict) };
    let no_fname: *mut c_char = ::core::ptr::null_mut();
    // SAFETY: the event carries no file name; running the autocommands is
    // what this function is for, and the caller allows it.
    unsafe {
        apply_autocmds(
            EVENT_RECORDINGLEAVE,
            no_fname,
            no_fname,
            false,
            curbuf.get(),
        )
    };
    // SAFETY: `dict` and `save_v_event` are the pair `get_v_event` made.
    unsafe { restore_v_event(dict, &raw mut save_v_event) };
}

/// `q`: start recording into register `c`, or stop and store what was
/// recorded.
///
/// Answers `Err` for an invalid register name, or when the recording
/// produced nothing.
///
/// # Safety
/// Runs arbitrary autocommands (RecordingEnter/RecordingLeave).
pub unsafe fn do_record(c: c_int) -> Result<(), Failed> {
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
        // SAFETY: main thread, with the message area set up.
        unsafe { showmode() };
        regname.set(c);
        let no_fname: *mut c_char = ::core::ptr::null_mut();
        // SAFETY: the event carries no file name, and this function's own
        // caller allows the autocommands it fires.
        unsafe {
            apply_autocmds(
                EVENT_RECORDINGENTER,
                no_fname,
                no_fname,
                false,
                curbuf.get(),
            )
        };
        return Ok(());
    }

    // Stop recording.
    // SAFETY: main thread; the recording buffer is this thread's own.
    let p = unsafe { get_recorded() };
    if !p.is_null() {
        // SAFETY: `get_recorded` answered an allocated NUL-terminated
        // string, unescaped in place -- which only ever shortens it.
        unsafe { vim_unescape_ks(p) };
    }
    // SAFETY: `p` is null or that NUL-terminated string; the autocommands
    // this fires are the ones the caller allows.
    unsafe { fire_recording_leave(regname.get(), p) };
    reg_recorded.set(reg_recording.get());
    reg_recording.set(0);
    if p_ch.get() == 0 || ui_has(kUIMessages) {
        // SAFETY: main thread, as above.
        unsafe { showmode() };
    } else {
        // Clear the "recording @a" message.
        msg(c"", 0);
    }
    if p.is_null() {
        return Err(Failed);
    }
    // Recording into a register must not move `""`.
    let old_y_previous = y_previous.get();
    // SAFETY: `p` is the allocated string `get_recorded` answered, which
    // `stuff_yank` takes ownership of on every path.
    let retval = unsafe { stuff_yank(regname.get(), p) };
    y_previous.set(old_y_previous);
    retval
}

/// Queue `s` in the typeahead buffer so that it is read back as if typed.
///
/// `esc` escapes `K_SPECIAL` and turns mapping off; `colon` wraps the text in
/// `:` and `<CR>` so that it runs as an Ex command line.
///
/// # Safety
/// `s` must be NUL-terminated.
unsafe fn put_in_typebuf(
    s: *mut c_char,
    esc: bool,
    colon: bool,
    silent: c_int,
) -> Result<(), Failed> {
    let mut retval = Ok(());
    // SAFETY: main thread, writing the typeahead buffer.
    unsafe { put_reedit_in_typebuf(silent) };

    // Pushed backwards: the `<CR>` first, then the text, then the `:`.
    if colon {
        let nl = c"\n".as_ptr().cast_mut();
        // SAFETY: a NUL-terminated literal, copied into the typeahead.
        retval = unsafe { ins_typebuf(nl, REMAP_NONE, 0, true, silent != 0) };
    }
    if retval.is_ok() {
        // SAFETY: `s` is NUL-terminated, so the escaped copy is too.
        let p = if esc {
            unsafe { vim_strsave_escape_ks(s) }
        } else {
            s
        };
        if p.is_null() {
            retval = Err(Failed);
        } else {
            let remap = if esc { REMAP_NONE } else { REMAP_YES };
            // SAFETY: `p` is NUL-terminated either way, and is copied.
            retval = unsafe { ins_typebuf(p, remap, 0, true, silent != 0) };
        }
        if esc {
            // SAFETY: the escaped copy is ours, and has been queued.
            unsafe { xfree(p as *mut c_void) };
        }
    }
    if colon && retval.is_ok() {
        let colon_key = c":".as_ptr().cast_mut();
        // SAFETY: a NUL-terminated literal, copied into the typeahead.
        retval = unsafe { ins_typebuf(colon_key, REMAP_NONE, 0, true, silent != 0) };
    }
    retval
}

/// Queue whatever will re-enter Insert mode, so that a register replayed from
/// Insert mode returns there afterwards.
///
/// `restart_edit` holds the mode as a key: `i`/`a`/`R`, or `V` for Virtual
/// Replace, which is two keys.
///
/// # Safety
/// Writes the typeahead buffer; main thread only.
unsafe fn put_reedit_in_typebuf(silent: c_int) {
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
    let keys = buf.as_mut_ptr() as *mut c_char;
    // SAFETY: `buf` is a local the branches above left NUL-terminated, and
    // `ins_typebuf` copies it into the typeahead buffer.
    let queued = unsafe { ins_typebuf(keys, REMAP_NONE, 0, true, silent != 0) };
    if queued.is_ok() {
        restart_edit.set(NUL);
    }
}

/// Join the line at `*idx` with the `\`-continuation lines in front of it.
///
/// The register is replayed backwards, so this walks *up* from `*idx` to the
/// first line that is not a continuation, concatenates the run, and leaves
/// `*idx` at that first line. A `"\ ` line is a comment inside a continuation
/// and contributes nothing.
///
/// # Safety
/// `lines` must hold at least `*idx + 1` strings, and `*idx` must be > 0.
unsafe fn execreg_line_continuation(lines: *mut String_0, idx: *mut size_t) -> *mut c_char {
    // SAFETY: the caller promises a readable, writable `idx`.
    let mut cmd_start = unsafe { *idx };
    debug_assert!(cmd_start > 0);
    let cmd_end = cmd_start;

    // Find the first line of the run.
    let mut ga = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut(),
    };
    // SAFETY: `ga` is a writable local, given the item size and growth step
    // every `ga_*` call below then works from.
    unsafe { ga_init(&raw mut ga, ::core::mem::size_of::<c_char>() as c_int, 400) };
    loop {
        cmd_start = cmd_start.wrapping_sub(1);
        if cmd_start == 0 {
            break;
        }
        // SAFETY: `cmd_start` only ever falls from the caller's `*idx`, so it
        // is one of the `*idx + 1` lines; each is NUL-terminated, which is
        // what `skipwhite` and the two tests below need.
        let p = unsafe { skipwhite((*lines.add(cmd_start)).data()) };
        if c_int::from(unsafe { *p }) != '\\' as c_int && !unsafe { is_continuation_comment(p) } {
            break;
        }
    }

    // Then concatenate it, dropping each continuation's leading `\`.
    // SAFETY: `cmd_start..=cmd_end` are lines of `lines`, as above, and each
    // is NUL-terminated; `p` walks inside one of them, so the length handed
    // to `ga_concat_len` is the rest of that line.
    let mut tmp = unsafe { lines.add(cmd_start) };
    unsafe { ga_concat_len(&raw mut ga, (*tmp).data(), (*tmp).len()) };
    for j in cmd_start + 1..=cmd_end {
        tmp = unsafe { lines.add(j) };
        let mut p = unsafe { skipwhite((*tmp).data()) };
        if c_int::from(unsafe { *p }) == '\\' as c_int {
            if ga.ga_len > 400 {
                unsafe { ga_set_growsize(&raw mut ga, ga.ga_len.min(8000)) };
            }
            p = unsafe { p.add(1) };
            let rest = unsafe { (*tmp).data().add((*tmp).len()).offset_from(p) } as size_t;
            unsafe { ga_concat_len(&raw mut ga, p, rest) };
        }
    }
    // SAFETY: `ga` holds `ga_len` bytes, which are copied out before it is
    // released.
    let str = unsafe {
        ga_append(&raw mut ga, NUL as u8);
        let str = xmemdupz(ga.ga_data, ga.ga_len as size_t) as *mut c_char;
        ga_clear(&raw mut ga);
        str
    };
    // SAFETY: the caller promises a writable `idx`.
    unsafe { *idx = cmd_start };
    str
}

/// Whether `p` is a `"\ ` line -- a comment inside a `\`-continuation.
///
/// # Safety
/// `p` must be NUL-terminated.
unsafe fn is_continuation_comment(p: *const c_char) -> bool {
    // SAFETY: `p` is NUL-terminated, and each test in the chain is the proof
    // that the next byte is still inside the string, so it stays whole.
    c_int::from(unsafe { *p }) == '"' as c_int
        && c_int::from(unsafe { *p.add(1) }) == '\\' as c_int
        && c_int::from(unsafe { *p.add(2) }) == ' ' as c_int
}

/// Queue the *contents* of register `regname` in the typeahead buffer, so
/// that the main loop reads it as if typed.
///
/// `colon` wraps every line in `:`/`<CR>`, which is what `:@a` wants; `addcr`
/// adds a final `<CR>` even to a charwise register; `silent` keeps the
/// queued text out of `'showcmd'`.
///
/// # Safety
/// May run arbitrary Vimscript through the `"=` register.
pub unsafe fn do_execreg(
    regname: c_int,
    colon: c_int,
    addcr: c_int,
    silent: c_int,
) -> Result<(), Failed> {
    let mut regname = regname;
    if regname == '@' as c_int {
        // `@@` repeats the last `@`.
        if execreg_lastc.get() == NUL {
            emsg(gettext(c"E748: No previously used register"));
            return Err(Failed);
        }
        regname = execreg_lastc.get();
    }
    // SAFETY: `valid_yank_reg` only looks the name up.
    if regname == '%' as c_int
        || regname == '#' as c_int
        || !unsafe { valid_yank_reg(regname, false) }
    {
        // SAFETY: reports the name; nothing of the caller's is dereferenced.
        unsafe { emsg_invreg(regname) };
        return Err(Failed);
    }
    execreg_lastc.set(regname);

    if regname == '_' as c_int {
        return Ok(()); // black hole: nothing to do
    }

    if regname == ':' as c_int {
        // The last command line, re-run. Control characters have to be
        // escaped with CTRL-V or the typeahead buffer would act on them.
        if last_cmdline.get().is_null() {
            emsg(gettext(e_nolastcmd));
            return Err(Failed);
        }
        // SAFETY: `new_last_cmdline` owns whatever it holds, and is cleared
        // on the next line so the freed pointer is never read back.
        unsafe { xfree(new_last_cmdline.get() as *mut c_void) };
        new_last_cmdline.set(::core::ptr::null_mut());
        // SAFETY: `last_cmdline` is non-null (tested just above) and
        // NUL-terminated, as is the literal set of characters to escape;
        // the answer is a fresh allocation of ours.
        let p = unsafe {
            vim_strsave_escaped_ext(
                last_cmdline.get(),
                c"\x01\x02\x03\x04\x05\x06\x07\x08\t\n\x0B\x0C\r\x0E\x0F\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F".as_ptr(),
                Ctrl_V as c_char,
                false,
            )
        };
        // A Visual-mode `@:` re-applies to the *current* selection, so
        // drop the `'<,'>` the command line was recorded with.
        // SAFETY: `p` is that NUL-terminated copy, so `strncmp` may read up
        // to five of its bytes, and a match is the proof that `p.add(5)` is
        // still inside it -- keep the test and the skip together.
        let retval = if visual_active() && unsafe { cstr::starts_with(p, b"'<,'>") } {
            unsafe { put_in_typebuf(p.add(5), true, true, silent) }
        } else {
            unsafe { put_in_typebuf(p, true, true, silent) }
        };
        // SAFETY: `p` is ours, and `put_in_typebuf` copied what it needed.
        unsafe { xfree(p as *mut c_void) };
        return retval;
    }

    if regname == '=' as c_int {
        // SAFETY: evaluating the expression register runs Vimscript, which
        // this function's own caller allows; the answer is null or an
        // allocated NUL-terminated string.
        let p = unsafe { get_expr_line() };
        if p.is_null() {
            return Err(Failed);
        }
        // SAFETY: `p` is that non-null NUL-terminated string, and is ours to
        // free once it has been copied into the typeahead.
        let retval = unsafe { put_in_typebuf(p, true, colon != 0, silent) };
        unsafe { xfree(p as *mut c_void) };
        return retval;
    }

    if regname == '.' as c_int {
        // SAFETY: main thread; answers null or an allocated NUL-terminated
        // copy of the last inserted text.
        let p = unsafe { get_last_insert_save() };
        if p.is_null() {
            emsg(gettext(e_noinstext));
            return Err(Failed);
        }
        // SAFETY: as above -- `p` is NUL-terminated and ours to free.
        let retval = unsafe { put_in_typebuf(p, false, colon != 0, silent) };
        unsafe { xfree(p as *mut c_void) };
        return retval;
    }

    // SAFETY: a valid register name (checked above) answers a live register.
    let reg = unsafe { get_yank_register(regname, YREG_PASTE) };
    // SAFETY: `reg` is that live register, so its `y_array` is there to test.
    if unsafe { (*reg).y_array.is_null() } {
        return Err(Failed);
    }
    let remap = if colon != 0 { REMAP_NONE } else { REMAP_YES };
    // SAFETY: main thread, writing the typeahead buffer.
    unsafe { put_reedit_in_typebuf(silent) };

    // The typeahead buffer is a stack, so the register goes in last line
    // first.
    let mut retval = Ok(());
    // SAFETY: `reg` is live, so `y_size` is its line count.
    let mut i = unsafe { (*reg).y_size };
    while i > 0 {
        i -= 1;
        // A linewise register, and every line of a charwise one but the
        // last, is followed by a newline.
        // SAFETY: `reg` is live, so its type and size are there to read; the
        // tests decide whether `ins_typebuf` runs at all, so the chain stays
        // whole. The key is a NUL-terminated literal, which is copied.
        let nl_failed = unsafe {
            ((*reg).y_type == kMTLineWise || i < (*reg).y_size.wrapping_sub(1) || addcr != 0)
                && ins_typebuf(c"\n".as_ptr().cast_mut(), remap, 0, true, silent != 0).is_err()
        };
        if nl_failed {
            return Err(Failed);
        }

        // SAFETY: a non-null `y_array` holds `y_size` lines and `i` is below
        // that, so this is one of them -- NUL-terminated, and owned by `reg`.
        let mut str = unsafe { (*(*reg).y_array.add(i)).data() };
        let mut free_str = false;
        if colon != 0 && i > 0 {
            // A `\`-continued Ex command has to be joined back up before
            // it is queued; `i` is moved to the first line of the run.
            // SAFETY: `str` is that NUL-terminated line, so `skipwhite` stops
            // inside it and `*p` is one of its bytes; `i` is in range and
            // above zero, which is what `execreg_line_continuation` asks for.
            let p = unsafe { skipwhite(str) };
            if c_int::from(unsafe { *p }) == '\\' as c_int || unsafe { is_continuation_comment(p) }
            {
                str = unsafe { execreg_line_continuation((*reg).y_array, &raw mut i) };
                free_str = true;
            }
        }
        // SAFETY: `str` is NUL-terminated either way -- a register line, or
        // the joined copy, which is ours to free.
        let escaped = unsafe { vim_strsave_escape_ks(str) };
        if free_str {
            unsafe { xfree(str as *mut c_void) };
        }
        // SAFETY: `escaped` is a fresh NUL-terminated copy, which
        // `ins_typebuf` copies again, leaving it ours to free.
        retval = unsafe { ins_typebuf(escaped, remap, 0, true, silent != 0) };
        unsafe { xfree(escaped as *mut c_void) };
        if retval.is_err() {
            return Err(Failed);
        }
        // SAFETY: a NUL-terminated literal, copied into the typeahead.
        if colon != 0
            && unsafe { ins_typebuf(c":".as_ptr().cast_mut(), remap, 0, true, silent != 0) }
                .is_err()
        {
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
/// `reg` may be a register already fetched by the caller.
///
/// # Safety
/// `reg` must be null or a live register. May run arbitrary Vimscript.
pub unsafe fn insert_reg(
    regname: c_int,
    reg: *mut yankreg_T,
    literally_arg: bool,
) -> Result<(), Failed> {
    let literally = literally_arg || is_literal_register(regname);

    // A register may be a long list of lines; let CTRL-C out.
    os_breakcheck();
    if got_int.get() {
        return Err(Failed);
    }
    // SAFETY: `valid_yank_reg` only looks the name up.
    if regname != NUL && !unsafe { valid_yank_reg(regname, false) } {
        return Err(Failed);
    }

    if regname == '.' as c_int {
        // The last insert is re-inserted rather than stuffed, so that it
        // can be repeated.
        // SAFETY: main thread; re-runs the last insert, which is among the
        // Vimscript the caller allows.
        return unsafe { stuff_inserted(NUL, 1, 1) };
    }

    let mut arg: *mut c_char = ::core::ptr::null_mut();
    let mut allocated = false;
    // SAFETY: `arg` and `allocated` are writable locals this fills in for a
    // special register; reading one may run Vimscript, which the caller
    // allows.
    if unsafe { get_spec_reg(regname, &raw mut arg, &raw mut allocated, true) } {
        if arg.is_null() {
            return Err(Failed);
        }
        // SAFETY: `arg` is non-null and NUL-terminated, and is copied into
        // the read buffer; it is ours to free only when it was allocated.
        unsafe { stuffescaped(arg, literally) };
        if allocated {
            unsafe { xfree(arg as *mut c_void) };
        }
        return Ok(());
    }

    let reg = if reg.is_null() {
        // SAFETY: a valid register name (checked above) answers a live
        // register.
        unsafe { get_yank_register(regname, YREG_PASTE) }
    } else {
        reg
    };
    // SAFETY: `reg` is live -- the caller's, or the one just fetched.
    if unsafe { (*reg).y_array.is_null() } {
        return Err(Failed);
    }
    // SAFETY: `reg` is live, so `y_size` is its line count. The original
    // read it once here too, to bound the loop.
    let y_size = unsafe { (*reg).y_size };
    for i in 0..y_size {
        // SAFETY: `reg` is live, so its `y_type` is there to read.
        if regname == '-' as c_int && unsafe { (*reg).y_type } == kMTCharWise {
            // The small-delete register goes in through `do_put`, so that
            // Replace mode's stack and the redo buffer stay right.
            let mut dir = BACKWARD;
            if State.get() & REPLACE_FLAG != 0 {
                // SAFETY: main thread with a current buffer; saves the
                // cursor line for undo.
                if u_save_cursor().is_err() {
                    return Err(Failed);
                }
                // SAFETY: a non-null `y_array` starts with a NUL-terminated
                // line, whose character count is what is deleted.
                let _ = unsafe { del_chars(mb_charlen((*(*reg).y_array).data()), 1) };
                let curpos = cur_win().w_cursor;
                // SAFETY: main thread with a current window and buffer.
                if unsafe { oneright() }.is_err() {
                    dir = FORWARD;
                }
                cur_win().w_cursor = curpos;
            }
            append_to_redobuff_char(Ctrl_R);
            append_to_redobuff_char(regname);
            // SAFETY: a null expression pointer means `do_put` takes the
            // text from the register `regname` names; it may run Vimscript,
            // which the caller allows.
            unsafe {
                do_put(
                    regname,
                    ::core::ptr::null_mut(),
                    dir,
                    1,
                    PUT_CURSEND as c_int,
                )
            };
        } else {
            // SAFETY: a non-null `y_array` holds `y_size` NUL-terminated
            // lines and `i` is below that count, so this is one of them.
            unsafe { stuffescaped((*(*reg).y_array.add(i)).data(), literally) };
            // SAFETY: `reg` is live, so its type and size are there to read.
            if unsafe { (*reg).y_type == kMTLineWise || i < (*reg).y_size.wrapping_sub(1) } {
                stuff_readbuf_char('\n' as c_int);
            }
        }
    }
    Ok(())
}

/// CTRL-R on the command line: insert register `regname` there.
///
/// # Safety
/// May run arbitrary Vimscript.
pub unsafe fn cmdline_paste_reg(regname: c_int, literally_arg: bool, remcr: bool) -> bool {
    let literally = literally_arg || is_literal_register(regname);
    // SAFETY: main thread; every register name answers a live register.
    let reg = unsafe { get_yank_register(regname, YREG_PASTE) };
    // SAFETY: `reg` is that live register, so its `y_array` is there to test.
    if unsafe { (*reg).y_array.is_null() } {
        return false;
    }
    // SAFETY: `reg` is live, so `y_size` is its line count. The original
    // read it once here too, to bound the loop.
    let y_size = unsafe { (*reg).y_size };
    for i in 0..y_size {
        // SAFETY: a non-null `y_array` holds `y_size` NUL-terminated lines
        // and `i` is below that count, so this is one of them.
        unsafe { cmdline_paste_str((*(*reg).y_array.add(i)).data(), literally) };
        // Add a <CR> between lines, unless the caller is going to.
        // SAFETY: `reg` is live; `"\r"` is a NUL-terminated literal.
        if unsafe { i < (*reg).y_size.wrapping_sub(1) } && !remcr {
            unsafe { cmdline_paste_str(c"\r".as_ptr(), literally) };
        }
        os_breakcheck();
        if got_int.get() {
            return false;
        }
    }
    true
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
