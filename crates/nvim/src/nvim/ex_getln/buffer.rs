//! The `ccline.cmdbuff` allocation, and pasting into it.
//!
//! [`realloc_cmdbuff`] is the one every caller has to be careful of: it moves
//! the buffer, so nothing may hold a pointer into it across a call.
//! [`cmdline_paste`] and [`ccheck_abbr`] are the two writers that go through
//! the register and abbreviation machinery, and the `*_fnameescape` helpers
//! escape a file name on its way in.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::keycodes::{
    Ctrl_A, Ctrl_BSL, Ctrl_C, Ctrl_F, Ctrl_L, Ctrl_N, Ctrl_P, Ctrl_V, Ctrl_W,
};

/// The command line's own bytes: `cmdbuff[..cmdlen]`.
///
/// Two things this writes down once instead of at every reader.  `cmdbuff`
/// is NULL between [`dealloc_cmdbuff`] and the next [`alloc_cmdbuff`] and
/// `slice::from_raw_parts` may not be handed a null pointer even with a
/// length of zero, so the empty line is a separate arm.  And the slice must
/// not outlive the next [`realloc_cmdbuff`] or [`put_on_cmdline`] — both
/// move the allocation — which is why every caller takes it inside the
/// expression that reads it rather than binding it across a call.
pub(crate) unsafe fn cmdline_bytes<'a>(cc: *const CmdlineInfo) -> &'a [::core::ffi::c_char] {
    unsafe {
        if (*cc).cmdbuff.is_null() {
            &[]
        } else {
            ::core::slice::from_raw_parts((*cc).cmdbuff, (*cc).cmdlen.max(0) as usize)
        }
    }
}

/// Get an Ex command line for the `:` command.
///
/// `c` is normally `:`, and NUL for `:append`; `indent` is the indent for
/// inside conditionals.  Registered as a `LineGetter` in several tables, so
/// this one keeps its C ABI.
pub unsafe extern "C" fn getexline(
    c: ::core::ffi::c_int,
    _cookie: *mut ::core::ffi::c_void,
    indent: ::core::ffi::c_int,
    do_concat: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        // When executing a register, remove the ':' in front of each line.
        if exec_from_reg.get() && vpeekc() == ':' as ::core::ffi::c_int {
            vgetc();
        }
        getcmdline(c, 1, indent, do_concat)
    }
}

pub unsafe fn cmdline_overstrike() -> bool {
    unsafe { (*ccline.ptr()).overstrike != 0 }
}

/// Whether the cursor is at the end of the command line.
pub unsafe fn cmdline_at_end() -> bool {
    unsafe {
        let cc = ccline.ptr();
        (*cc).cmdpos >= (*cc).cmdlen
    }
}

/// Deallocate the command-line buffer, updating its size and length.
pub(crate) unsafe fn dealloc_cmdbuff() {
    unsafe {
        let cc = ccline.ptr();
        xfree((*cc).cmdbuff as *mut ::core::ffi::c_void);
        (*cc).cmdbuff = ::core::ptr::null_mut();
        (*cc).cmdbufflen = 0;
        (*cc).cmdlen = 0;
    }
}

/// Allocate a new command-line buffer into `ccline.cmdbuff`/`cmdbufflen`.
pub(crate) unsafe fn alloc_cmdbuff(mut len: ::core::ffi::c_int) {
    unsafe {
        // Give some extra space to avoid having to allocate all the time.
        if len < 80 {
            len = 100;
        } else {
            len += 20;
        }
        let cc = ccline.ptr();
        (*cc).cmdbuff = xmalloc(len as size_t) as *mut ::core::ffi::c_char;
        (*cc).cmdbufflen = len;
    }
}

/// Re-allocate the command line to `len` plus something extra.
///
/// This *moves* the buffer.  `xp_pattern` is the one pointer into it
/// upstream knows about and re-derives here; anything else holding a
/// pointer or an offset into `cmdbuff` across a call is a bug — the
/// completion code deliberately keeps indices rather than pointers for
/// that reason.
pub unsafe fn realloc_cmdbuff(len: ::core::ffi::c_int) {
    unsafe {
        let cc = ccline.ptr();
        if len < (*cc).cmdbufflen {
            return; // no need to resize
        }
        let old = (*cc).cmdbuff;
        alloc_cmdbuff(len); // will get some more
        // There isn't always a NUL after the command, but it may need to be
        // there, so copy up to the NUL and add one.
        memmove(
            (*cc).cmdbuff as *mut ::core::ffi::c_void,
            old as *const ::core::ffi::c_void,
            (*cc).cmdlen as size_t,
        );
        *(*cc).cmdbuff.offset((*cc).cmdlen as isize) = NUL as ::core::ffi::c_char;

        let xpc = (*cc).xpc;
        if !xpc.is_null()
            && !(*xpc).xp_pattern.is_null()
            && (*xpc).xp_context != EXPAND_NOTHING
            && (*xpc).xp_context != EXPAND_UNSUCCESSFUL
        {
            // If xp_pattern pointed inside the old cmdbuff it has to be
            // adjusted to point into the newly allocated memory.
            let i = (*xpc).xp_pattern.offset_from(old) as ::core::ffi::c_int;
            if i >= 0 && i <= (*cc).cmdlen {
                (*xpc).xp_pattern = (*cc).cmdbuff.offset(i as isize);
            }
        }
        xfree(old as *mut ::core::ffi::c_void);
    }
}

/// Save `ccline`, because obtaining the `=` register may execute
/// `normal :cmd` and overwrite it.
pub(crate) unsafe fn save_cmdline(ccp: *mut CmdlineInfo) {
    unsafe {
        let cc = ccline.ptr();
        *ccp = *cc;
        memset(
            cc as *mut ::core::ffi::c_void,
            0,
            ::core::mem::size_of::<CmdlineInfo>(),
        );
        (*cc).prev_ccline = ccp;
        (*cc).cmdbuff = ::core::ptr::null_mut(); // signal that ccline is not in use
    }
}

/// Restore `ccline` after it has been saved with [`save_cmdline`].
pub(crate) unsafe fn restore_cmdline(ccp: *mut CmdlineInfo) {
    unsafe {
        ccline.set(*ccp);
    }
}

/// Paste a yank register into the command line, for CTRL-R.
///
/// `insert_reg()` can't be used here, because special characters from the
/// register contents would be interpreted as commands.  `literally` inserts
/// the text as-is rather than as typed; `remcr` removes a trailing CR.
/// Answers false for failure.
pub(crate) unsafe fn cmdline_paste(
    regname: ::core::ffi::c_int,
    literally: bool,
    remcr: bool,
) -> bool {
    unsafe {
        // Check for a valid regname; also accept the special characters
        // CTRL-R takes on the command line.
        if regname != Ctrl_F
            && regname != Ctrl_P
            && regname != Ctrl_W
            && regname != Ctrl_A
            && regname != Ctrl_L
            && !valid_yank_reg(regname, false)
        {
            return false;
        }

        // A register containing CTRL-R can cause an endless loop. Allow
        // using CTRL-C to break out of it.
        line_breakcheck();
        if got_int.get() {
            return false;
        }

        // "textlock" avoids nasty things like going to another buffer while
        // evaluating an expression.
        let mut arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
        let mut allocated: bool = false;
        (*textlock.ptr()) += 1;
        let got_special = get_spec_reg(regname, &raw mut arg, &raw mut allocated, true);
        (*textlock.ptr()) -= 1;

        if !got_special {
            return cmdline_paste_reg(regname, literally, remcr);
        }

        // Got the value of a special register in "arg".
        if arg.is_null() {
            return false;
        }
        let mut p = arg;
        // With 'incsearch' set and CTRL-R CTRL-W used: skip the duplicate
        // part of the word.
        if p_is.get() != 0 && regname == Ctrl_W {
            let cc = ccline.ptr();
            // Locate the start of the last word in the cmd buffer.
            let end = (*cc).cmdbuff.offset((*cc).cmdpos as isize);
            let mut w = end;
            while w > (*cc).cmdbuff {
                let len = utf_head_off((*cc).cmdbuff, w.offset(-1)) + 1;
                if !vim_iswordc(utf_ptr2char(w.offset(-(len as isize)))) {
                    break;
                }
                w = w.offset(-(len as isize));
            }
            let len = end.offset_from(w) as ::core::ffi::c_int;
            let same = if p_ic.get() != 0 {
                strncasecmp(w, arg, len as size_t) == 0
            } else {
                strncmp(w, arg, len as size_t) == 0
            };
            if same {
                p = p.offset(len as isize);
            }
        }

        cmdline_paste_str(p, literally);
        if allocated {
            xfree(arg as *mut ::core::ffi::c_void);
        }
        true
    }
}

/// Put a string on the command line.
///
/// With `literally` set the text is inserted as-is; otherwise it is stuffed
/// back as if typed — which does not leave the command line, but does mean
/// every character that would end it has to be quoted with CTRL-V.
pub unsafe fn cmdline_paste_str(mut s: *const ::core::ffi::c_char, literally: bool) {
    unsafe {
        if literally {
            put_on_cmdline(s, -1, true);
            return;
        }
        while *s as ::core::ffi::c_int != NUL {
            let cv = *s as uint8_t as ::core::ffi::c_int;
            if cv == Ctrl_V && *s.offset(1) as ::core::ffi::c_int != 0 {
                s = s.offset(1);
            }
            let c = mb_cptr2char_adv(&raw mut s);
            if cv == Ctrl_V
                || c == ESC
                || c == Ctrl_C
                || c == CAR
                || c == NL
                || c == Ctrl_L
                || (c == Ctrl_BSL && *s as ::core::ffi::c_int == Ctrl_N)
            {
                stuffcharReadbuff(Ctrl_V);
            }
            stuffcharReadbuff(c);
        }
    }
}

/// Check whether typing `c` completes an abbreviation on the command line.
pub(crate) unsafe fn ccheck_abbr(c: ::core::ffi::c_int) -> bool {
    unsafe {
        if p_paste.get() != 0 || no_abbr.get() {
            // no abbreviations, or in paste mode
            return false;
        }
        let cc = ccline.ptr();
        let line = cmdline_bytes(cc);

        // Do not consider '<,'> to be part of the mapping; skip leading
        // whitespace first. This actually accepts any mark.
        let mut spos = line
            .iter()
            .position(|&ch| !ascii_iswhite(ch as ::core::ffi::c_int))
            .unwrap_or(line.len());
        if line.len() - spos > 5
            && line[spos] == '\'' as ::core::ffi::c_char
            && line[spos + 2] == ',' as ::core::ffi::c_char
            && line[spos + 3] == '\'' as ::core::ffi::c_char
        {
            spos += 5;
        } else {
            // Check the abbreviation from the start of the command line.
            spos = 0;
        }

        check_abbr(c, (*cc).cmdbuff, (*cc).cmdpos, spos as ::core::ffi::c_int)
    }
}

/// Escape the special characters in `fname`, depending on `what`:
/// `VSE_NONE` for a file-name argument after a Vim command, `VSE_SHELL` for
/// a shell command, `VSE_BUFFER` for `:buffer`.  Answers allocated memory.
pub unsafe fn vim_strsave_fnameescape(
    fname: *const ::core::ffi::c_char,
    what: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let esc = if what == VSE_SHELL {
            SHELL_ESC_CHARS.as_ptr()
        } else if what == VSE_BUFFER {
            BUFFER_ESC_CHARS.as_ptr()
        } else {
            PATH_ESC_CHARS.as_ptr()
        };
        let mut p = vim_strsave_escaped(fname, esc);
        if what == VSE_SHELL && csh_like_shell() {
            // csh and similar shells need two backslashes before '!': one
            // is taken by Vim, one by the shell.
            let s = vim_strsave_escaped(p, c"!".as_ptr());
            xfree(p as *mut ::core::ffi::c_void);
            p = s;
        }
        // '>' and '+' are special at the start of some commands, e.g.
        // ":edit" and ":write". "cd -" has a special meaning.
        let first = *p as ::core::ffi::c_int;
        if first == '>' as ::core::ffi::c_int
            || first == '+' as ::core::ffi::c_int
            || (first == '-' as ::core::ffi::c_int && *p.offset(1) as ::core::ffi::c_int == NUL)
        {
            escape_fname(&raw mut p);
        }
        p
    }
}

/// Put a backslash before the file name in `pp`, which is allocated memory.
pub unsafe fn escape_fname(pp: *mut *mut ::core::ffi::c_char) {
    unsafe {
        let p = xmalloc(strlen(*pp).wrapping_add(2)) as *mut ::core::ffi::c_char;
        *p = '\\' as ::core::ffi::c_char;
        strcpy(p.offset(1), *pp);
        xfree(*pp as *mut ::core::ffi::c_void);
        *pp = p;
    }
}

/// For each name in `files[..num_files]`: if `orig_pat` starts with `~/`,
/// put the home directory back as `~`.
pub unsafe fn tilde_replace(
    orig_pat: *mut ::core::ffi::c_char,
    num_files: ::core::ffi::c_int,
    files: *mut *mut ::core::ffi::c_char,
) {
    unsafe {
        if *orig_pat as ::core::ffi::c_int != '~' as ::core::ffi::c_int
            || !vim_ispathsep(*orig_pat.offset(1) as ::core::ffi::c_int)
        {
            return;
        }
        for file in ::core::slice::from_raw_parts_mut(files, num_files.max(0) as usize) {
            let p = home_replace_save(::core::ptr::null_mut::<buf_T>(), *file);
            xfree(*file as *mut ::core::ffi::c_void);
            *file = p;
        }
    }
}
