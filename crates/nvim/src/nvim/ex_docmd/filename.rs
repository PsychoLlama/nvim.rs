//! What a command's file argument stands for: `%`, `#`, the `<…>` family,
//! wildcards, and the backtick form.
//!
//! Expansion rewrites the command line in place — every replacement is a
//! fresh allocation the whole `exarg_T` is repointed into, which is what
//! `repl_cmdline` does and why it is the only place allowed to free the
//! old line.
#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::src::nvim::arglist::arg_all;
use crate::src::nvim::buffer::buflist_findnr;
use crate::src::nvim::charset::{backslash_halve, getdigits_int, skipwhite};
use crate::src::nvim::cmdexpand::{ExpandInit, ExpandOne};
use crate::src::nvim::eval::fs::modify_fname;
use crate::src::nvim::eval::skip_expr;
use crate::src::nvim::eval::typval::tv_list_find_str;
use crate::src::nvim::eval::vars::get_vim_var_list;
use crate::src::nvim::ex_docmd::cmdline::sourcing_entry;
use crate::src::nvim::ex_docmd::scan::skip_grep_pat;
use crate::src::nvim::ex_docmd::{
    ECMD_LAST, ESTACK_SCRIPT, ESTACK_SFILE, ESTACK_STACK, EX_NOSPC, EXPAND_FILES, FAIL, FIND_EVAL,
    FIND_IDENT, FIND_STRING, FNAME_HYP, FNAME_MESS, MAXPATHL, NUL, OK, VALID_HEAD, VALID_PATH,
    WILD_ADD_SLASH, WILD_EXPAND_FREE, WILD_ICASE, WILD_LIST_NOTFOUND, WILD_NOERROR, dollar_command,
    e_no_autocommand_buffer_number_to_substitute_for_abuf,
    e_no_autocommand_file_name_to_substitute_for_afile,
    e_no_autocommand_match_name_to_substitute_for_amatch, e_no_call_stack_to_substitute_for_stack,
    e_no_line_number_to_use_for_sflnum, e_no_line_number_to_use_for_slnum,
    e_no_script_file_name_to_substitute_for_script, e_no_source_file_name_to_substitute_for_sfile,
};
use crate::src::nvim::file_search::file_name_at_cursor;
use crate::src::nvim::main::{
    NameBuff, autocmd_bufnr, autocmd_fname, autocmd_fname_full, autocmd_match, curbuf,
    current_sctx, e_usingsid, escape_chars, p_gp, p_mp, p_wic,
};
use crate::src::nvim::memory::{xfree, xmalloc, xmemdupz, xstrdup, xstrlcpy};
use crate::src::nvim::message::{emsg, msg_make};
use crate::src::nvim::normal::find_ident_under_cursor;
use crate::src::nvim::os::env::{expand_env_esc, expand_env_save};
use crate::src::nvim::os::libc::{gettext, strcmp, strpbrk};
use crate::src::nvim::os::libc::{memmove, snprintf, strcat, strcpy, strlen, strncmp, strrchr};
use crate::src::nvim::path::{FullName_save, path_has_wildcard, path_tail, path_try_shorten_fname};
use crate::src::nvim::quickfix::grep_internal;
use crate::src::nvim::runtime::estack_sfile;
use crate::src::nvim::strings::{strrep, vim_strchr, vim_strsave_escaped};
use crate::src::nvim::types::{
    CMD_bang, CMD_grep, CMD_grepadd, CMD_lgrep, CMD_lgrepadd, CMD_lmake, CMD_make, CMD_terminal,
    VV_OLDFILES, exarg_T, expand_T, linenr_T, size_t, ssize_t, uint8_t, uint32_t,
};

/// `:make` and `:grep` are 'makeprg'/'grepprg' with `$*` replaced by the
/// argument — spliced in here, before `%` and `#` are expanded, so that
/// the program string can use them too.
///
/// Answers where the argument now starts, which is the whole new line for
/// a program that had no `$*`.
pub unsafe fn replace_makeprg(
    eap: *mut exarg_T,
    mut arg: *mut c_char,
    cmdlinep: *mut *mut c_char,
) -> *mut c_char {
    unsafe {
        let idx = (*eap).cmdidx as c_int;
        let is_grep = idx == CMD_grep as c_int
            || idx == CMD_lgrep as c_int
            || idx == CMD_grepadd as c_int
            || idx == CMD_lgrepadd as c_int;
        let is_make = idx == CMD_make as c_int || idx == CMD_lmake as c_int;
        // `grep_internal` means 'grepprg' is `internal`, which is not a
        // program at all.
        if !(is_make || is_grep) || grep_internal((*eap).cmdidx) {
            return arg;
        }

        let buf = &*curbuf.get();
        let program: *const c_char = if is_grep {
            if *buf.b_p_gp as c_int == NUL {
                p_gp.get()
            } else {
                buf.b_p_gp
            }
        } else if *buf.b_p_mp as c_int == NUL {
            p_mp.get()
        } else {
            buf.b_p_mp
        };

        arg = skipwhite(arg);
        let mut new_cmdline = strrep(program, c"$*".as_ptr(), arg);
        if new_cmdline.is_null() {
            // No `$*`: the argument goes on the end.
            new_cmdline = xmalloc(strlen(program) + strlen(arg) + 2) as *mut c_char;
            strcpy(new_cmdline, program as *mut c_char);
            strcat(new_cmdline, c" ".as_ptr());
            strcat(new_cmdline, arg);
        }

        msg_make(arg);
        xfree(*cmdlinep as *mut c_void);
        *cmdlinep = new_cmdline;
        new_cmdline
    }
}

/// Expand every `%`, `#`, `` `cmd` `` and `<…>` in a command's file
/// argument, then expand wildcards if the command takes exactly one name.
pub unsafe fn expand_filename(
    eap: *mut exarg_T,
    cmdlinep: *mut *mut c_char,
    errormsgp: *mut *const c_char,
) -> c_int {
    unsafe {
        let ea = &mut *eap;
        // A `:vimgrep` pattern is not a file name, so the scan starts after
        // it.
        let mut p = skip_grep_pat(eap);
        let mut has_wildcards = path_has_wildcard(p);

        while *p as c_int != NUL {
            if *p as c_int == '`' as c_int && *p.add(1) as c_int == '=' as c_int {
                // `` `=expr` `` is evaluated much later, by the shell
                // expansion; step over it without touching it.
                p = p.add(2);
                skip_expr(&raw mut p, ptr::null_mut());
                if *p as c_int == '`' as c_int {
                    p = p.add(1);
                }
                continue;
            }
            if vim_strchr(c"%#<".as_ptr(), *p as uint8_t as c_int).is_null() {
                p = p.add(1);
                continue;
            }

            let mut srclen: size_t = 0;
            let mut escaped: c_int = 0;
            let mut repl = eval_vars(
                p,
                ea.arg,
                &raw mut srclen,
                &raw mut ea.do_ecmd_lnum,
                errormsgp,
                &raw mut escaped,
                true,
            );
            if !(*errormsgp).is_null() {
                return FAIL;
            }
            if repl.is_null() {
                p = p.add(srclen as usize);
                continue;
            }

            if !vim_strchr(repl, '$' as c_int).is_null()
                || !vim_strchr(repl, '~' as c_int).is_null()
            {
                let old = repl;
                repl = expand_env_save(repl);
                xfree(old as *mut c_void);
            }

            // A name that will be handed to a *file* argument gets the
            // shell-special characters escaped — but not for the commands
            // that hand the whole argument to a shell themselves, and not
            // for a name that came back already escaped (`##`).
            let idx = ea.cmdidx as c_int;
            if ea.usefilter == 0
                && escaped == 0
                && idx != CMD_bang as c_int
                && idx != CMD_grep as c_int
                && idx != CMD_grepadd as c_int
                && idx != CMD_lgrep as c_int
                && idx != CMD_lgrepadd as c_int
                && idx != CMD_lmake as c_int
                && idx != CMD_make as c_int
                && idx != CMD_terminal as c_int
                && ea.argt & EX_NOSPC as uint32_t == 0
            {
                let mut l = repl;
                while *l != 0 {
                    if !vim_strchr(escape_chars.get(), *l as uint8_t as c_int).is_null() {
                        let escaped_repl = vim_strsave_escaped(repl, escape_chars.get());
                        xfree(repl as *mut c_void);
                        repl = escaped_repl;
                        break;
                    }
                    l = l.add(1);
                }
            }
            // A `!` in the replacement would be read as "the previous
            // command" by the shell-command line parser.
            if (ea.usefilter != 0 || idx == CMD_bang as c_int || idx == CMD_terminal as c_int)
                && !strpbrk(repl, c"!".as_ptr()).is_null()
            {
                let escaped_repl = vim_strsave_escaped(repl, c"!".as_ptr());
                xfree(repl as *mut c_void);
                repl = escaped_repl;
            }

            p = repl_cmdline(eap, p, srclen, repl, cmdlinep);
            xfree(repl as *mut c_void);
        }

        // `EX_NOSPC` means the argument is one file name, so wildcards in
        // it can be expanded to exactly one match.
        if ea.argt & EX_NOSPC as uint32_t == 0 || ea.usefilter != 0 {
            return OK;
        }

        if has_wildcards {
            // Environment variables first: they may hold the wildcards, or
            // may be all that looked like one.
            if !vim_strchr(ea.arg, '$' as c_int).is_null()
                || !vim_strchr(ea.arg, '~' as c_int).is_null()
            {
                expand_env_esc(
                    ea.arg,
                    NameBuff.ptr() as *mut c_char,
                    MAXPATHL,
                    true,
                    true,
                    ptr::null_mut(),
                );
                has_wildcards = path_has_wildcard(NameBuff.ptr() as *mut c_char);
                repl_cmdline(
                    eap,
                    ea.arg,
                    strlen(ea.arg),
                    NameBuff.ptr() as *mut c_char,
                    cmdlinep,
                );
            }
        }
        if !has_wildcards {
            backslash_halve(ea.arg);
            return OK;
        }

        let mut xpc: expand_T = core::mem::zeroed();
        ExpandInit(&raw mut xpc);
        xpc.xp_context = EXPAND_FILES as c_int;
        let mut options =
            WILD_LIST_NOTFOUND as c_int | WILD_NOERROR as c_int | WILD_ADD_SLASH as c_int;
        if p_wic.get() != 0 {
            options += WILD_ICASE as c_int;
        }
        let expanded = ExpandOne(
            &raw mut xpc,
            ea.arg,
            ptr::null_mut(),
            options,
            WILD_EXPAND_FREE as c_int,
        );
        if expanded.is_null() {
            return FAIL;
        }
        repl_cmdline(eap, ea.arg, strlen(ea.arg), expanded, cmdlinep);
        xfree(expanded as *mut c_void);
        OK
    }
}

/// Replace `srclen` bytes at `src` with `repl`, in a freshly allocated copy
/// of the whole command line.
///
/// Everything in the `exarg_T` that points into the old line is repointed:
/// `cmd`, `arg`, `nextcmd`, the API's argument vector and `do_ecmd_cmd`.
/// Answers where the text after the replacement now lives, which is where
/// the caller's scan resumes.
pub(crate) unsafe fn repl_cmdline(
    eap: *mut exarg_T,
    src: *mut c_char,
    srclen: size_t,
    repl: *mut c_char,
    cmdlinep: *mut *mut c_char,
) -> *mut c_char {
    unsafe {
        let ea = &mut *eap;
        let len = strlen(repl);
        // The tail after the replacement, the replacement itself, a
        // terminator, and — because `nextcmd` is stored past the end — the
        // next command and its own terminator.
        let mut size = src.offset_from(*cmdlinep) as size_t + strlen(src.add(srclen)) + len + 3;
        if !ea.nextcmd.is_null() {
            size += strlen(ea.nextcmd);
        }
        let new_cmdline = xmalloc(size) as *mut c_char;

        let offset = src.offset_from(*cmdlinep) as size_t;
        memmove(
            new_cmdline as *mut c_void,
            *cmdlinep as *const c_void,
            offset,
        );
        memmove(
            new_cmdline.add(offset as usize) as *mut c_void,
            repl as *const c_void,
            len,
        );
        let tail = offset + len;
        strcpy(new_cmdline.add(tail as usize), src.add(srclen));
        let resume = new_cmdline.add(tail as usize);

        if !ea.nextcmd.is_null() {
            let after = strlen(new_cmdline) + 1;
            strcpy(new_cmdline.add(after as usize), ea.nextcmd);
            ea.nextcmd = new_cmdline.add(after as usize);
        }
        ea.cmd = new_cmdline.offset(ea.cmd.offset_from(*cmdlinep));
        ea.arg = new_cmdline.offset(ea.arg.offset_from(*cmdlinep));
        // An argument after the replacement moved by the length difference;
        // one before it did not move at all.
        for j in 0..ea.argc {
            let old = *ea.args.add(j);
            let old_off = old.offset_from(*cmdlinep);
            *ea.args.add(j) = if offset >= old_off as size_t {
                new_cmdline.offset(old_off)
            } else {
                new_cmdline.offset(old_off + len.wrapping_sub(srclen) as isize)
            };
        }
        // The `+cmd` argument, unless it is the shared `$` constant, which
        // is not in the command line at all.
        if !ea.do_ecmd_cmd.is_null() && ea.do_ecmd_cmd != dollar_command.ptr() as *mut c_char {
            ea.do_ecmd_cmd = new_cmdline.offset(ea.do_ecmd_cmd.offset_from(*cmdlinep));
        }

        xfree(*cmdlinep as *mut c_void);
        *cmdlinep = new_cmdline;
        resume
    }
}

/// The `%`, `#` and `<…>` items, in the order `find_cmdline_var` answers
/// them. The index *is* the answer, so the order is load-bearing.
const SPECIALS: [&CStr; 15] = [
    c"%",        // the current file
    c"#",        // the alternate file, or `#99`
    c"<cword>",  // the word under the cursor
    c"<cWORD>",  // the WORD under the cursor
    c"<cexpr>",  // the expression under the cursor
    c"<cfile>",  // the path name under the cursor
    c"<sfile>",  // the `:source`d file's name
    c"<slnum>",  // its line number
    c"<stack>",  // the call stack
    c"<script>", // the script file's name
    c"<afile>",  // the autocommand's file name
    c"<abuf>",   // its buffer number
    c"<amatch>", // what its pattern matched
    c"<sflnum>", // the script file's line number
    c"<SID>",    // the script ID, as `<SNR>123_`
];

const SPEC_PERC: ssize_t = 0;
const SPEC_HASH: ssize_t = 1;
const SPEC_CWORD: ssize_t = 2;
const SPEC_CCWORD: ssize_t = 3;
const SPEC_CEXPR: ssize_t = 4;
const SPEC_CFILE: ssize_t = 5;
const SPEC_SFILE: ssize_t = 6;
const SPEC_SLNUM: ssize_t = 7;
const SPEC_STACK: ssize_t = 8;
const SPEC_SCRIPT: ssize_t = 9;
const SPEC_AFILE: ssize_t = 10;
const SPEC_ABUF: ssize_t = 11;
const SPEC_AMATCH: ssize_t = 12;
const SPEC_SFLNUM: ssize_t = 13;
const SPEC_SID: ssize_t = 14;

/// Does `src` start with one of the special items? Answers its index and
/// sets `*usedlen` to its length, or answers −1.
pub unsafe fn find_cmdline_var(src: *const c_char, usedlen: *mut size_t) -> ssize_t {
    unsafe {
        for (i, spec) in SPECIALS.iter().enumerate() {
            let len = spec.to_bytes().len() as size_t;
            if strncmp(src, spec.as_ptr(), len) == 0 {
                *usedlen = len;
                return i as ssize_t;
            }
        }
        -1
    }
}

/// Expand one special item at `src` into a freshly allocated string.
///
/// `*usedlen` comes back as how much of `src` was consumed, which includes
/// any `:p:h`-style modifiers. A null answer with `*errormsg` unset means
/// "nothing here": the caller steps over `*usedlen` bytes and carries on.
///
/// `empty_is_error` is what tells an expansion that produced nothing from
/// one that is not allowed to.
pub unsafe fn eval_vars(
    src: *mut c_char,
    srcstart: *const c_char,
    usedlen: *mut size_t,
    lnump: *mut linenr_T,
    errormsg: *mut *const c_char,
    escaped: *mut c_int,
    empty_is_error: bool,
) -> *mut c_char {
    unsafe {
        let mut result: *mut c_char = c"".as_ptr() as *mut c_char;
        let mut resultbuf: *mut c_char = ptr::null_mut();
        let mut resultlen: size_t;
        let mut valid = VALID_HEAD as c_int | VALID_PATH as c_int;
        let mut tilde_file = false;
        let mut skip_mod = false;
        let mut strbuf: [c_char; 30] = [0; 30];

        *errormsg = ptr::null();
        if !escaped.is_null() {
            *escaped = 0;
        }

        let spec_idx = find_cmdline_var(src, usedlen);
        if spec_idx < 0 {
            *usedlen = 1;
            return ptr::null_mut();
        }

        // A backslash before it means "take it literally": remove the
        // backslash and answer nothing.
        if src > srcstart as *mut c_char && *src.offset(-1) as c_int == '\\' as c_int {
            *usedlen = 0;
            memmove(
                src.offset(-1) as *mut c_void,
                src as *const c_void,
                strlen(src) + 1,
            );
            return ptr::null_mut();
        }

        if spec_idx == SPEC_CWORD || spec_idx == SPEC_CCWORD || spec_idx == SPEC_CEXPR {
            let what = match spec_idx {
                SPEC_CWORD => FIND_IDENT as c_int | FIND_STRING as c_int,
                SPEC_CEXPR => FIND_IDENT as c_int | FIND_STRING as c_int | FIND_EVAL as c_int,
                _ => FIND_STRING as c_int,
            };
            resultlen = find_ident_under_cursor(&raw mut result, what, ptr::null_mut());
            if resultlen == 0 {
                // An empty message: the caller reports nothing, but stops.
                *errormsg = c"".as_ptr();
                return ptr::null_mut();
            }
        } else {
            match spec_idx {
                SPEC_PERC => {
                    if (*curbuf.get()).b_fname.is_null() {
                        result = c"".as_ptr() as *mut c_char;
                        valid = 0;
                    } else {
                        result = (*curbuf.get()).b_fname;
                        tilde_file = strcmp(result, c"~".as_ptr()) == 0;
                    }
                }
                SPEC_HASH => {
                    if *src.add(1) as c_int == '#' as c_int {
                        // `##` is the whole argument list, already escaped.
                        result = arg_all();
                        resultbuf = result;
                        *usedlen = 2;
                        if !escaped.is_null() {
                            *escaped = 1;
                        }
                        skip_mod = true;
                    } else {
                        let mut s = src.add(1);
                        if *s as c_int == '<' as c_int {
                            s = s.add(1);
                        }
                        let i = getdigits_int(&raw mut s, false, 0);
                        // `#-` is not a negative buffer number; give the `-`
                        // back.
                        if s == src.add(2) && *src.add(1) as c_int == '-' as c_int {
                            s = s.offset(-1);
                        }
                        *usedlen = s.offset_from(src) as size_t;

                        if *src.add(1) as c_int == '<' as c_int && i != 0 {
                            // `#<3` is the third entry of `v:oldfiles`.
                            if *usedlen < 2 {
                                *usedlen = 1;
                                return ptr::null_mut();
                            }
                            result = tv_list_find_str(get_vim_var_list(VV_OLDFILES), i - 1)
                                as *mut c_char;
                            if result.is_null() {
                                *errormsg = c"".as_ptr();
                                return ptr::null_mut();
                            }
                        } else {
                            if i == 0 && *src.add(1) as c_int == '<' as c_int && *usedlen > 1 {
                                *usedlen = 1;
                            }
                            let buf = buflist_findnr(i);
                            if buf.is_null() {
                                *errormsg = gettext(
                                    c"E194: No alternate file name to substitute for '#'".as_ptr(),
                                );
                                return ptr::null_mut();
                            }
                            if !lnump.is_null() {
                                *lnump = ECMD_LAST as linenr_T;
                            }
                            if (*buf).b_fname.is_null() {
                                result = c"".as_ptr() as *mut c_char;
                                valid = 0;
                            } else {
                                result = (*buf).b_fname;
                                tilde_file = strcmp(result, c"~".as_ptr()) == 0;
                            }
                        }
                    }
                }
                SPEC_CFILE => {
                    result = file_name_at_cursor(
                        FNAME_MESS as c_int | FNAME_HYP as c_int,
                        1,
                        ptr::null_mut(),
                    );
                    if result.is_null() {
                        *errormsg = c"".as_ptr();
                        return ptr::null_mut();
                    }
                    resultbuf = result;
                }
                SPEC_AFILE => {
                    // The autocommand's file name is shortened on first use
                    // and the shortened form is kept.
                    if !(*autocmd_fname.ptr()).is_null() && !autocmd_fname_full.get() {
                        autocmd_fname_full.set(true);
                        result = FullName_save(autocmd_fname.get(), false);
                        xstrlcpy(autocmd_fname.get(), result, MAXPATHL as size_t);
                        xfree(result as *mut c_void);
                    }
                    result = autocmd_fname.get();
                    if result.is_null() {
                        *errormsg =
                            gettext(e_no_autocommand_file_name_to_substitute_for_afile.as_ptr());
                        return ptr::null_mut();
                    }
                    result = path_try_shorten_fname(result);
                }
                SPEC_ABUF => {
                    if autocmd_bufnr.get() <= 0 {
                        *errormsg =
                            gettext(e_no_autocommand_buffer_number_to_substitute_for_abuf.as_ptr());
                        return ptr::null_mut();
                    }
                    snprintf(
                        &raw mut strbuf as *mut c_char,
                        size_of::<[c_char; 30]>(),
                        c"%d".as_ptr(),
                        autocmd_bufnr.get(),
                    );
                    result = &raw mut strbuf as *mut c_char;
                }
                SPEC_AMATCH => {
                    result = autocmd_match.get();
                    if result.is_null() {
                        *errormsg =
                            gettext(e_no_autocommand_match_name_to_substitute_for_amatch.as_ptr());
                        return ptr::null_mut();
                    }
                }
                SPEC_SFILE | SPEC_STACK | SPEC_SCRIPT => {
                    let (which, msg) = match spec_idx {
                        SPEC_SFILE => (
                            ESTACK_SFILE,
                            e_no_source_file_name_to_substitute_for_sfile.as_ptr(),
                        ),
                        SPEC_STACK => (
                            ESTACK_STACK,
                            e_no_call_stack_to_substitute_for_stack.as_ptr(),
                        ),
                        _ => (
                            ESTACK_SCRIPT,
                            e_no_script_file_name_to_substitute_for_script.as_ptr(),
                        ),
                    };
                    result = estack_sfile(which);
                    if result.is_null() {
                        *errormsg = gettext(msg as *const c_char);
                        return ptr::null_mut();
                    }
                    resultbuf = result;
                }
                SPEC_SLNUM => {
                    let entry = &*sourcing_entry();
                    if entry.es_name.is_null() || entry.es_lnum == 0 {
                        *errormsg = gettext(e_no_line_number_to_use_for_slnum.as_ptr());
                        return ptr::null_mut();
                    }
                    snprintf(
                        &raw mut strbuf as *mut c_char,
                        size_of::<[c_char; 30]>(),
                        c"%d".as_ptr(),
                        entry.es_lnum,
                    );
                    result = &raw mut strbuf as *mut c_char;
                }
                SPEC_SFLNUM => {
                    // The line the *script* is on, which is the script's own
                    // offset plus the line inside it.
                    let lnum = (*current_sctx.ptr()).sc_lnum + (*sourcing_entry()).es_lnum;
                    if lnum == 0 {
                        *errormsg = gettext(e_no_line_number_to_use_for_sflnum.as_ptr());
                        return ptr::null_mut();
                    }
                    snprintf(
                        &raw mut strbuf as *mut c_char,
                        size_of::<[c_char; 30]>(),
                        c"%d".as_ptr(),
                        lnum,
                    );
                    result = &raw mut strbuf as *mut c_char;
                }
                SPEC_SID => {
                    if (*current_sctx.ptr()).sc_sid <= 0 {
                        *errormsg = gettext(&raw const e_usingsid as *const c_char);
                        return ptr::null_mut();
                    }
                    snprintf(
                        &raw mut strbuf as *mut c_char,
                        size_of::<[c_char; 30]>(),
                        c"<SNR>%d_".as_ptr(),
                        (*current_sctx.ptr()).sc_sid,
                    );
                    result = &raw mut strbuf as *mut c_char;
                }
                _ => {
                    *errormsg = c"".as_ptr();
                }
            }

            resultlen = strlen(result);
            if *src.add(*usedlen) as c_int == '<' as c_int {
                // A trailing `<` drops the extension.
                *usedlen += 1;
                let dot = strrchr(result, '.' as c_int);
                if !dot.is_null() && dot >= path_tail(result) {
                    resultlen = dot.offset_from(result) as size_t;
                }
            } else if !skip_mod {
                valid |= modify_fname(
                    src,
                    tilde_file,
                    usedlen,
                    &raw mut result,
                    &raw mut resultbuf,
                    &raw mut resultlen,
                );
                if result.is_null() {
                    *errormsg = c"".as_ptr();
                    return ptr::null_mut();
                }
            }
        }

        if resultlen == 0 || valid != VALID_HEAD as c_int + VALID_PATH as c_int {
            if empty_is_error {
                *errormsg = if valid != VALID_HEAD as c_int + VALID_PATH as c_int {
                    gettext(
                        c"E499: Empty file name for '%' or '#', only works with \":p:h\"".as_ptr(),
                    )
                } else {
                    gettext(c"E500: Evaluates to an empty string".as_ptr())
                };
            }
            result = ptr::null_mut();
        } else {
            result = xmemdupz(result as *const c_void, resultlen) as *mut c_char;
        }
        xfree(resultbuf as *mut c_void);
        result
    }
}

/// Expand every `<sfile>` in `arg`, in a fresh copy.
///
/// Answers null after reporting, when an expansion failed.
pub unsafe fn expand_sfile(arg: *mut c_char) -> *mut c_char {
    unsafe {
        let mut result = xstrdup(arg);
        let mut p = result;
        while *p != 0 {
            if strncmp(p, c"<sfile>".as_ptr(), 7) != 0 {
                p = p.add(1);
                continue;
            }
            let mut srclen: size_t = 0;
            let mut errormsg: *const c_char = ptr::null();
            let repl = eval_vars(
                p,
                result,
                &raw mut srclen,
                ptr::null_mut(),
                &raw mut errormsg,
                ptr::null_mut(),
                true,
            );
            if !errormsg.is_null() {
                if *errormsg != 0 {
                    emsg(errormsg);
                }
                xfree(result as *mut c_void);
                return ptr::null_mut();
            }
            if repl.is_null() {
                p = p.add(srclen as usize);
                continue;
            }
            let size = strlen(result) - srclen + strlen(repl) + 1;
            let newres = xmalloc(size) as *mut c_char;
            memmove(
                newres as *mut c_void,
                result as *const c_void,
                p.offset_from(result) as size_t,
            );
            strcpy(newres.offset(p.offset_from(result)), repl);
            let used = strlen(newres);
            strcat(newres, p.add(srclen as usize));
            xfree(repl as *mut c_void);
            xfree(result as *mut c_void);
            result = newres;
            p = newres.add(used as usize);
        }
        result
    }
}
