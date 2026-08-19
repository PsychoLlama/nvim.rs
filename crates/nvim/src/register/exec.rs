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

use core::ffi::{c_char, c_int, c_void};

use super::*;
use crate::types::{FAIL, OK};

/// Put the allocated string `p` in register `regname` as a single charwise
/// line, appending for an uppercase name.
///
/// Takes ownership of `p` on every path.
///
/// # Safety
/// `p` must be an allocated, NUL-terminated string.
unsafe fn stuff_yank(regname: c_int, p: *mut c_char) -> c_int {
    unsafe {
        if regname != 0 && !valid_yank_reg(regname, true) {
            xfree(p as *mut c_void);
            return FAIL;
        }
        if regname == '_' as c_int {
            xfree(p as *mut c_void); // black hole: discard
            return OK;
        }

        let plen = strlen(p);
        let reg = get_yank_register(regname, YREG_YANK);
        if is_append_register(regname) && !(*reg).y_array.is_null() {
            // Append to the register's last line rather than replacing it.
            let pp = (*reg).y_array.add((*reg).y_size.wrapping_sub(1));
            let tmplen = (*pp).size.wrapping_add(plen);
            let tmp = xmalloc(tmplen.wrapping_add(1)) as *mut c_char;
            memcpy(tmp as *mut c_void, (*pp).data as *const c_void, (*pp).size);
            memcpy(tmp.add((*pp).size) as *mut c_void, p as *const c_void, plen);
            *tmp.add(tmplen) = NUL as c_char;
            xfree(p as *mut c_void);
            xfree((*pp).data as *mut c_void);
            *pp = String_0 {
                data: tmp,
                size: tmplen,
            };
        } else {
            free_register(reg);
            (*reg).additional_data = ::core::ptr::null_mut();
            (*reg).y_array = xmalloc(::core::mem::size_of::<String_0>()) as *mut String_0;
            *(*reg).y_array = String_0 {
                data: p,
                size: plen,
            };
            (*reg).y_size = 1;
            (*reg).y_type = kMTCharWise;
        }
        (*reg).timestamp = os_time();
        OK
    }
}

/// Build the `v:event` dictionary RecordingLeave sees, and fire the event.
///
/// `contents` is what was recorded, with its `K_SPECIAL` escaping already
/// undone; null when the recording produced nothing.
///
/// # Safety
/// `contents` must be null or NUL-terminated. Runs arbitrary autocommands.
unsafe fn fire_recording_leave(regname: c_int, contents: *mut c_char) {
    unsafe {
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
        let dict = get_v_event(&raw mut save_v_event);
        if !contents.is_null() {
            tv_dict_add_str(dict, c"regcontents".as_ptr(), 11, contents);
        }
        let mut buf: [c_char; 67] = [0; 67];
        buf[0] = regname as c_char;
        buf[1] = NUL as c_char;
        tv_dict_add_str(dict, c"regname".as_ptr(), 7, buf.as_mut_ptr());
        tv_dict_set_keys_readonly(dict);
        apply_autocmds(
            EVENT_RECORDINGLEAVE,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            false,
            curbuf.get(),
        );
        restore_v_event(dict, &raw mut save_v_event);
    }
}

/// `q`: start recording into register `c`, or stop and store what was
/// recorded.
///
/// Answers `FAIL` for an invalid register name, or when the recording
/// produced nothing.
///
/// # Safety
/// Runs arbitrary autocommands (RecordingEnter/RecordingLeave).
pub unsafe fn do_record(c: c_int) -> c_int {
    unsafe {
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
                return FAIL;
            }
            reg_recording.set(c);
            showmode();
            regname.set(c);
            apply_autocmds(
                EVENT_RECORDINGENTER,
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                false,
                curbuf.get(),
            );
            return OK;
        }

        // Stop recording.
        let p = get_recorded();
        if !p.is_null() {
            vim_unescape_ks(p);
        }
        fire_recording_leave(regname.get(), p);
        reg_recorded.set(reg_recording.get());
        reg_recording.set(0);
        if p_ch.get() == 0 || ui_has(kUIMessages) {
            showmode();
        } else {
            // Clear the "recording @a" message.
            msg(c"".as_ptr(), 0);
        }
        if p.is_null() {
            return FAIL;
        }
        // Recording into a register must not move `""`.
        let old_y_previous = y_previous.get();
        let retval = stuff_yank(regname.get(), p);
        y_previous.set(old_y_previous);
        retval
    }
}

/// Queue `s` in the typeahead buffer so that it is read back as if typed.
///
/// `esc` escapes `K_SPECIAL` and turns mapping off; `colon` wraps the text in
/// `:` and `<CR>` so that it runs as an Ex command line.
///
/// # Safety
/// `s` must be NUL-terminated.
unsafe fn put_in_typebuf(s: *mut c_char, esc: bool, colon: bool, silent: c_int) -> c_int {
    unsafe {
        let mut retval = OK;
        put_reedit_in_typebuf(silent);

        // Pushed backwards: the `<CR>` first, then the text, then the `:`.
        if colon {
            retval = ins_typebuf(c"\n".as_ptr().cast_mut(), REMAP_NONE, 0, true, silent != 0);
        }
        if retval == OK {
            let p = if esc { vim_strsave_escape_ks(s) } else { s };
            if p.is_null() {
                retval = FAIL;
            } else {
                retval = ins_typebuf(
                    p,
                    if esc { REMAP_NONE } else { REMAP_YES },
                    0,
                    true,
                    silent != 0,
                );
            }
            if esc {
                xfree(p as *mut c_void);
            }
        }
        if colon && retval == OK {
            retval = ins_typebuf(c":".as_ptr().cast_mut(), REMAP_NONE, 0, true, silent != 0);
        }
        retval
    }
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
    unsafe {
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
        if ins_typebuf(
            buf.as_mut_ptr() as *mut c_char,
            REMAP_NONE,
            0,
            true,
            silent != 0,
        ) == OK
        {
            restart_edit.set(NUL);
        }
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
    unsafe {
        let mut cmd_start = *idx;
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
        ga_init(&raw mut ga, ::core::mem::size_of::<c_char>() as c_int, 400);
        loop {
            cmd_start = cmd_start.wrapping_sub(1);
            if cmd_start == 0 {
                break;
            }
            let p = skipwhite((*lines.add(cmd_start)).data);
            if c_int::from(*p) != '\\' as c_int && !is_continuation_comment(p) {
                break;
            }
        }

        // Then concatenate it, dropping each continuation's leading `\`.
        let mut tmp = lines.add(cmd_start);
        ga_concat_len(&raw mut ga, (*tmp).data, (*tmp).size);
        for j in cmd_start + 1..=cmd_end {
            tmp = lines.add(j);
            let mut p = skipwhite((*tmp).data);
            if c_int::from(*p) == '\\' as c_int {
                if ga.ga_len > 400 {
                    ga_set_growsize(&raw mut ga, ga.ga_len.min(8000));
                }
                p = p.add(1);
                ga_concat_len(
                    &raw mut ga,
                    p,
                    (*tmp).data.add((*tmp).size).offset_from(p) as size_t,
                );
            }
        }
        ga_append(&raw mut ga, NUL as u8);
        let str = xmemdupz(ga.ga_data, ga.ga_len as size_t) as *mut c_char;
        ga_clear(&raw mut ga);
        *idx = cmd_start;
        str
    }
}

/// Whether `p` is a `"\ ` line -- a comment inside a `\`-continuation.
///
/// # Safety
/// `p` must be NUL-terminated.
unsafe fn is_continuation_comment(p: *const c_char) -> bool {
    unsafe {
        c_int::from(*p) == '"' as c_int
            && c_int::from(*p.add(1)) == '\\' as c_int
            && c_int::from(*p.add(2)) == ' ' as c_int
    }
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
pub unsafe fn do_execreg(regname: c_int, colon: c_int, addcr: c_int, silent: c_int) -> c_int {
    unsafe {
        let mut regname = regname;
        if regname == '@' as c_int {
            // `@@` repeats the last `@`.
            if execreg_lastc.get() == NUL {
                emsg(gettext(c"E748: No previously used register".as_ptr()));
                return FAIL;
            }
            regname = execreg_lastc.get();
        }
        if regname == '%' as c_int || regname == '#' as c_int || !valid_yank_reg(regname, false) {
            emsg_invreg(regname);
            return FAIL;
        }
        execreg_lastc.set(regname);

        if regname == '_' as c_int {
            return OK; // black hole: nothing to do
        }

        if regname == ':' as c_int {
            // The last command line, re-run. Control characters have to be
            // escaped with CTRL-V or the typeahead buffer would act on them.
            if last_cmdline.get().is_null() {
                emsg(gettext(&raw const e_nolastcmd as *const c_char));
                return FAIL;
            }
            xfree(new_last_cmdline.get() as *mut c_void);
            new_last_cmdline.set(::core::ptr::null_mut());
            let p = vim_strsave_escaped_ext(
                last_cmdline.get(),
                c"\x01\x02\x03\x04\x05\x06\x07\x08\t\n\x0B\x0C\r\x0E\x0F\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F".as_ptr(),
                Ctrl_V as c_char,
                false,
            );
            // A Visual-mode `@:` re-applies to the *current* selection, so
            // drop the `'<,'>` the command line was recorded with.
            let retval = if VIsual_active.get() && strncmp(p, c"'<,'>".as_ptr(), 5) == 0 {
                put_in_typebuf(p.add(5), true, true, silent)
            } else {
                put_in_typebuf(p, true, true, silent)
            };
            xfree(p as *mut c_void);
            return retval;
        }

        if regname == '=' as c_int {
            let p = get_expr_line();
            if p.is_null() {
                return FAIL;
            }
            let retval = put_in_typebuf(p, true, colon != 0, silent);
            xfree(p as *mut c_void);
            return retval;
        }

        if regname == '.' as c_int {
            let p = get_last_insert_save();
            if p.is_null() {
                emsg(gettext(&raw const e_noinstext as *const c_char));
                return FAIL;
            }
            let retval = put_in_typebuf(p, false, colon != 0, silent);
            xfree(p as *mut c_void);
            return retval;
        }

        let reg = get_yank_register(regname, YREG_PASTE);
        if (*reg).y_array.is_null() {
            return FAIL;
        }
        let remap = if colon != 0 { REMAP_NONE } else { REMAP_YES };
        put_reedit_in_typebuf(silent);

        // The typeahead buffer is a stack, so the register goes in last line
        // first.
        let mut retval = OK;
        let mut i = (*reg).y_size;
        while i > 0 {
            i -= 1;
            if (*reg).y_type == kMTLineWise || i < (*reg).y_size.wrapping_sub(1) || addcr != 0 {
                if ins_typebuf(c"\n".as_ptr().cast_mut(), remap, 0, true, silent != 0) == FAIL {
                    return FAIL;
                }
            }

            let mut str = (*(*reg).y_array.add(i)).data;
            let mut free_str = false;
            if colon != 0 && i > 0 {
                // A `\`-continued Ex command has to be joined back up before
                // it is queued; `i` is moved to the first line of the run.
                let p = skipwhite(str);
                if c_int::from(*p) == '\\' as c_int || is_continuation_comment(p) {
                    str = execreg_line_continuation((*reg).y_array, &raw mut i);
                    free_str = true;
                }
            }
            let escaped = vim_strsave_escape_ks(str);
            if free_str {
                xfree(str as *mut c_void);
            }
            retval = ins_typebuf(escaped, remap, 0, true, silent != 0);
            xfree(escaped as *mut c_void);
            if retval == FAIL {
                return FAIL;
            }
            if colon != 0
                && ins_typebuf(c":".as_ptr().cast_mut(), remap, 0, true, silent != 0) == FAIL
            {
                return FAIL;
            }
        }
        reg_executing.set(if regname == 0 { '"' as c_int } else { regname });
        pending_end_reg_executing.set(false);
        retval
    }
}

/// CTRL-R in Insert mode: queue register `regname` in the read buffer.
///
/// `literally_arg` inserts the text as-is rather than as if typed; a register
/// whose contents are always literal ([`is_literal_register`]) forces it.
/// `reg` may be a register already fetched by the caller.
///
/// # Safety
/// `reg` must be null or a live register. May run arbitrary Vimscript.
pub unsafe fn insert_reg(regname: c_int, reg: *mut yankreg_T, literally_arg: bool) -> c_int {
    unsafe {
        let literally = literally_arg || is_literal_register(regname);

        // A register may be a long list of lines; let CTRL-C out.
        os_breakcheck();
        if got_int.get() {
            return FAIL;
        }
        if regname != NUL && !valid_yank_reg(regname, false) {
            return FAIL;
        }

        if regname == '.' as c_int {
            // The last insert is re-inserted rather than stuffed, so that it
            // can be repeated.
            return stuff_inserted(NUL, 1, true_0);
        }

        let mut arg: *mut c_char = ::core::ptr::null_mut();
        let mut allocated = false;
        if get_spec_reg(regname, &raw mut arg, &raw mut allocated, true) {
            if arg.is_null() {
                return FAIL;
            }
            stuffescaped(arg, literally);
            if allocated {
                xfree(arg as *mut c_void);
            }
            return OK;
        }

        let reg = if reg.is_null() {
            get_yank_register(regname, YREG_PASTE)
        } else {
            reg
        };
        if (*reg).y_array.is_null() {
            return FAIL;
        }
        for i in 0..(*reg).y_size {
            if regname == '-' as c_int && (*reg).y_type == kMTCharWise {
                // The small-delete register goes in through `do_put`, so that
                // Replace mode's stack and the redo buffer stay right.
                let mut dir = BACKWARD;
                if State.get() & REPLACE_FLAG != 0 {
                    if u_save_cursor() == FAIL {
                        return FAIL;
                    }
                    del_chars(mb_charlen((*(*reg).y_array).data), true_0);
                    let curpos = (*curwin.get()).w_cursor;
                    if oneright() == FAIL {
                        dir = FORWARD;
                    }
                    (*curwin.get()).w_cursor = curpos;
                }
                AppendCharToRedobuff(Ctrl_R);
                AppendCharToRedobuff(regname);
                do_put(
                    regname,
                    ::core::ptr::null_mut(),
                    dir,
                    1,
                    PUT_CURSEND as c_int,
                );
            } else {
                stuffescaped((*(*reg).y_array.add(i)).data, literally);
                if (*reg).y_type == kMTLineWise || i < (*reg).y_size.wrapping_sub(1) {
                    stuffcharReadbuff('\n' as c_int);
                }
            }
        }
        OK
    }
}

/// CTRL-R on the command line: insert register `regname` there.
///
/// # Safety
/// May run arbitrary Vimscript.
pub unsafe fn cmdline_paste_reg(regname: c_int, literally_arg: bool, remcr: bool) -> bool {
    unsafe {
        let literally = literally_arg || is_literal_register(regname);
        let reg = get_yank_register(regname, YREG_PASTE);
        if (*reg).y_array.is_null() {
            return FAIL != 0;
        }
        for i in 0..(*reg).y_size {
            cmdline_paste_str((*(*reg).y_array.add(i)).data, literally);
            // Add a <CR> between lines, unless the caller is going to.
            if i < (*reg).y_size.wrapping_sub(1) && !remcr {
                cmdline_paste_str(c"\r".as_ptr(), literally);
            }
            os_breakcheck();
            if got_int.get() {
                return FAIL != 0;
            }
        }
        OK != 0
    }
}
