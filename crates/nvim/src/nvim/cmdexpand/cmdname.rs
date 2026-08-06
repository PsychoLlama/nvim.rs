//! The per-command context switch.
//!
//! [`set_context_by_cmdname`] is C's `set_context_by_cmdname`: one arm per
//! `CMD_*` whose argument has its own completion.  [`set_one_cmd_context`]
//! walks a single command's arguments to find the one the cursor is in, and
//! [`set_cmd_context`] / [`expand_cmdline`] are the two entry points over
//! both.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::keycodes::Ctrl_V;
use crate::src::nvim::types::{
    CMD_SIZE, CMD_USER, CMD_USER_BUF, CMD_abbreviate, CMD_aboveleft, CMD_amenu, CMD_and,
    CMD_anoremenu, CMD_append, CMD_argdelete, CMD_argdo, CMD_augroup, CMD_aunmenu, CMD_autocmd,
    CMD_bdelete, CMD_belowright, CMD_botright, CMD_breakadd, CMD_breakdel, CMD_browse, CMD_bufdo,
    CMD_buffer, CMD_bunload, CMD_bwipeout, CMD_cabbrev, CMD_caddexpr, CMD_call, CMD_cd, CMD_cdo,
    CMD_cexpr, CMD_cfdo, CMD_cgetexpr, CMD_chdir, CMD_checkhealth, CMD_checktime, CMD_cmap,
    CMD_cmapclear, CMD_cmenu, CMD_cnoreabbrev, CMD_cnoremap, CMD_cnoremenu, CMD_colorscheme,
    CMD_command, CMD_compiler, CMD_confirm, CMD_const, CMD_cunabbrev, CMD_cunmap, CMD_cunmenu,
    CMD_debug, CMD_delcommand, CMD_delfunction, CMD_diffget, CMD_diffput, CMD_djump, CMD_dlist,
    CMD_doautoall, CMD_doautocmd, CMD_dsearch, CMD_dsplit, CMD_echo, CMD_echoerr, CMD_echohl,
    CMD_echomsg, CMD_echon, CMD_elseif, CMD_emenu, CMD_equal, CMD_execute, CMD_filetype,
    CMD_filter, CMD_find, CMD_folddoclosed, CMD_folddoopen, CMD_for, CMD_function, CMD_global,
    CMD_help, CMD_hide, CMD_highlight, CMD_history, CMD_horizontal, CMD_iabbrev, CMD_if, CMD_ijump,
    CMD_ilist, CMD_imap, CMD_imapclear, CMD_imenu, CMD_inoreabbrev, CMD_inoremap, CMD_inoremenu,
    CMD_isearch, CMD_isplit, CMD_iunabbrev, CMD_iunmap, CMD_iunmenu, CMD_keepalt, CMD_keepjumps,
    CMD_keepmarks, CMD_keeppatterns, CMD_laddexpr, CMD_language, CMD_lcd, CMD_lchdir, CMD_ldo,
    CMD_leftabove, CMD_let, CMD_lexpr, CMD_lfdo, CMD_lgetexpr, CMD_lmap, CMD_lmapclear,
    CMD_lnoremap, CMD_lockmarks, CMD_lshift, CMD_lsp, CMD_ltag, CMD_lua, CMD_lunmap, CMD_map,
    CMD_mapclear, CMD_match, CMD_menu, CMD_messages, CMD_nmap, CMD_nmapclear, CMD_nmenu,
    CMD_nnoremap, CMD_nnoremenu, CMD_noautocmd, CMD_noreabbrev, CMD_noremap, CMD_noremenu,
    CMD_noswapfile, CMD_nunmap, CMD_nunmenu, CMD_omap, CMD_omapclear, CMD_omenu, CMD_onoremap,
    CMD_onoremenu, CMD_ounmap, CMD_ounmenu, CMD_ownsyntax, CMD_packadd, CMD_pbuffer, CMD_popup,
    CMD_profdel, CMD_profile, CMD_psearch, CMD_ptag, CMD_ptjump, CMD_ptselect, CMD_read, CMD_redir,
    CMD_restart, CMD_retab, CMD_return, CMD_rightbelow, CMD_rshift, CMD_runtime, CMD_sandbox,
    CMD_sbuffer, CMD_scriptnames, CMD_set, CMD_setfiletype, CMD_setglobal, CMD_setlocal, CMD_sfind,
    CMD_sign, CMD_silent, CMD_smap, CMD_smapclear, CMD_snoremap, CMD_stag, CMD_stjump,
    CMD_stselect, CMD_substitute, CMD_sunmap, CMD_syntax, CMD_syntime, CMD_tab, CMD_tabdo,
    CMD_tabfind, CMD_tag, CMD_tcd, CMD_tchdir, CMD_tjump, CMD_tlmenu, CMD_tlnoremenu, CMD_tlunmenu,
    CMD_tmenu, CMD_topleft, CMD_tselect, CMD_tunmenu, CMD_unabbreviate, CMD_unlet, CMD_unmap,
    CMD_unmenu, CMD_unsilent, CMD_update, CMD_verbose, CMD_vertical, CMD_vglobal, CMD_vmap,
    CMD_vmapclear, CMD_vmenu, CMD_vnoremap, CMD_vnoremenu, CMD_vunmap, CMD_vunmenu, CMD_while,
    CMD_windo, CMD_write, CMD_xmap, CMD_xmapclear, CMD_xnoremap, CMD_xunmap,
};
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;

/// Set the completion context in `xp` for command `cmd` with index `cmdidx`.
///
/// The argument to the command is `arg` and the argument flags are `argt`.
/// For user-defined commands and for environment variables, `context` carries
/// the completion type.
///
/// Returns a pointer to the next command, or NULL if there is no next command.
pub(crate) unsafe fn set_context_by_cmdname(
    cmd: *const c_char,
    cmdidx: cmdidx_T,
    xp: *mut expand_T,
    mut arg: *const c_char,
    argt: uint32_t,
    context: c_int,
    forceit: bool,
) -> *const c_char {
    unsafe {
        match cmdidx {
            CMD_find | CMD_sfind | CMD_tabfind => {
                if (*xp).xp_context == EXPAND_FILES {
                    (*xp).xp_context = if *get_findfunc() as c_int != NUL {
                        EXPAND_FINDFUNC
                    } else {
                        EXPAND_FILES_IN_PATH
                    };
                }
            }
            CMD_cd | CMD_chdir | CMD_lcd | CMD_lchdir | CMD_tcd | CMD_tchdir => {
                if (*xp).xp_context == EXPAND_FILES {
                    (*xp).xp_context = EXPAND_DIRS_IN_CDPATH;
                }
            }
            CMD_help => {
                (*xp).xp_context = EXPAND_HELP;
                (*xp).xp_pattern = arg as *mut c_char;
            }

            // Command modifiers: return the argument.  Also for commands with
            // an argument that is a command.
            CMD_aboveleft | CMD_argdo | CMD_belowright | CMD_botright | CMD_browse | CMD_bufdo
            | CMD_cdo | CMD_cfdo | CMD_confirm | CMD_debug | CMD_folddoclosed | CMD_folddoopen
            | CMD_hide | CMD_horizontal | CMD_keepalt | CMD_keepjumps | CMD_keepmarks
            | CMD_keeppatterns | CMD_ldo | CMD_leftabove | CMD_lfdo | CMD_lockmarks
            | CMD_noautocmd | CMD_noswapfile | CMD_restart | CMD_rightbelow | CMD_sandbox
            | CMD_silent | CMD_tab | CMD_tabdo | CMD_topleft | CMD_unsilent | CMD_verbose
            | CMD_vertical | CMD_windo => return arg,

            CMD_filter => return set_context_in_filter_cmd(xp, arg),

            CMD_match => return set_context_in_match_cmd(xp, arg),

            // All completion for the +cmdline_compl feature goes here.
            CMD_command => return set_context_in_user_cmd(xp, arg),

            CMD_delcommand => {
                (*xp).xp_context = EXPAND_USER_COMMANDS;
                (*xp).xp_pattern = arg as *mut c_char;
            }

            CMD_global | CMD_vglobal => {
                let nextcmd = find_cmd_after_global_cmd(arg);
                if nextcmd.is_null() && may_expand_pattern.get() {
                    set_context_with_pattern(xp);
                }
                return nextcmd;
            }

            CMD_and | CMD_substitute => {
                let nextcmd = find_cmd_after_substitute_cmd(arg);
                if nextcmd.is_null() && may_expand_pattern.get() {
                    set_context_with_pattern(xp);
                }
                return nextcmd;
            }

            CMD_isearch | CMD_dsearch | CMD_ilist | CMD_dlist | CMD_ijump | CMD_psearch
            | CMD_djump | CMD_isplit | CMD_dsplit => {
                return find_cmd_after_isearch_cmd(xp, arg);
            }

            CMD_autocmd => return set_context_in_autocmd(xp, arg as *mut c_char, false),

            CMD_doautocmd | CMD_doautoall => {
                return set_context_in_autocmd(xp, arg as *mut c_char, true);
            }

            CMD_set => set_context_in_set_cmd(xp, arg as *mut c_char, 0),
            CMD_setglobal => set_context_in_set_cmd(xp, arg as *mut c_char, OPT_GLOBAL),
            CMD_setlocal => set_context_in_set_cmd(xp, arg as *mut c_char, OPT_LOCAL),

            CMD_tag | CMD_stag | CMD_ptag | CMD_ltag | CMD_tselect | CMD_stselect
            | CMD_ptselect | CMD_tjump | CMD_stjump | CMD_ptjump => {
                (*xp).xp_context = if wop_flags.get() & kOptWopFlagTagfile as c_uint != 0 {
                    EXPAND_TAGS_LISTFILES
                } else {
                    EXPAND_TAGS
                };
                (*xp).xp_pattern = arg as *mut c_char;
            }

            CMD_augroup => {
                (*xp).xp_context = EXPAND_AUGROUP;
                (*xp).xp_pattern = arg as *mut c_char;
            }

            CMD_syntax => set_context_in_syntax_cmd(xp, arg),

            CMD_const | CMD_let | CMD_if | CMD_elseif | CMD_while | CMD_for | CMD_echo
            | CMD_echon | CMD_execute | CMD_echomsg | CMD_echoerr | CMD_call | CMD_return
            | CMD_cexpr | CMD_caddexpr | CMD_cgetexpr | CMD_lexpr | CMD_laddexpr | CMD_lgetexpr => {
                set_context_for_expression(xp, arg as *mut c_char, cmdidx);
            }

            CMD_unlet => return set_context_in_unlet_cmd(xp, arg),

            CMD_function | CMD_delfunction => {
                (*xp).xp_context = EXPAND_USER_FUNC;
                (*xp).xp_pattern = arg as *mut c_char;
            }

            CMD_echohl => set_context_in_echohl_cmd(xp, arg),
            CMD_highlight => set_context_in_highlight_cmd(xp, arg),
            CMD_sign => set_context_in_sign_cmd(xp, arg as *mut c_char),

            CMD_bdelete | CMD_bwipeout | CMD_bunload => {
                // Only the argument the cursor is in is completed.  Upstream
                // falls through into the buffer-name arm below.
                loop {
                    (*xp).xp_pattern = strchr(arg, ' ' as c_int);
                    if (*xp).xp_pattern.is_null() {
                        break;
                    }
                    arg = (*xp).xp_pattern.add(1);
                }
                (*xp).xp_context = EXPAND_BUFFERS;
                (*xp).xp_pattern = arg as *mut c_char;
            }
            CMD_buffer | CMD_sbuffer | CMD_pbuffer | CMD_checktime => {
                (*xp).xp_context = EXPAND_BUFFERS;
                (*xp).xp_pattern = arg as *mut c_char;
            }

            CMD_diffget | CMD_diffput => {
                // If current buffer is in diff mode, complete buffer names
                // which are in diff mode, and different than current buffer.
                (*xp).xp_context = EXPAND_DIFF_BUFFERS;
                (*xp).xp_pattern = arg as *mut c_char;
            }

            CMD_USER | CMD_USER_BUF => {
                return set_context_in_user_cmdarg(cmd, arg, argt, context, xp, forceit);
            }

            CMD_map | CMD_noremap | CMD_nmap | CMD_nnoremap | CMD_vmap | CMD_vnoremap
            | CMD_omap | CMD_onoremap | CMD_imap | CMD_inoremap | CMD_cmap | CMD_cnoremap
            | CMD_lmap | CMD_lnoremap | CMD_smap | CMD_snoremap | CMD_xmap | CMD_xnoremap => {
                return set_context_in_map_cmd(
                    xp,
                    cmd as *mut c_char,
                    arg as *mut c_char,
                    forceit,
                    false,
                    false,
                    cmdidx,
                );
            }
            CMD_unmap | CMD_nunmap | CMD_vunmap | CMD_ounmap | CMD_iunmap | CMD_cunmap
            | CMD_lunmap | CMD_sunmap | CMD_xunmap => {
                return set_context_in_map_cmd(
                    xp,
                    cmd as *mut c_char,
                    arg as *mut c_char,
                    forceit,
                    false,
                    true,
                    cmdidx,
                );
            }
            CMD_mapclear | CMD_nmapclear | CMD_vmapclear | CMD_omapclear | CMD_imapclear
            | CMD_cmapclear | CMD_lmapclear | CMD_smapclear | CMD_xmapclear => {
                (*xp).xp_context = EXPAND_MAPCLEAR;
                (*xp).xp_pattern = arg as *mut c_char;
            }

            CMD_abbreviate | CMD_noreabbrev | CMD_cabbrev | CMD_cnoreabbrev | CMD_iabbrev
            | CMD_inoreabbrev => {
                return set_context_in_map_cmd(
                    xp,
                    cmd as *mut c_char,
                    arg as *mut c_char,
                    forceit,
                    true,
                    false,
                    cmdidx,
                );
            }
            CMD_unabbreviate | CMD_cunabbrev | CMD_iunabbrev => {
                return set_context_in_map_cmd(
                    xp,
                    cmd as *mut c_char,
                    arg as *mut c_char,
                    forceit,
                    true,
                    true,
                    cmdidx,
                );
            }

            CMD_menu | CMD_noremenu | CMD_unmenu | CMD_amenu | CMD_anoremenu | CMD_aunmenu
            | CMD_nmenu | CMD_nnoremenu | CMD_nunmenu | CMD_vmenu | CMD_vnoremenu | CMD_vunmenu
            | CMD_omenu | CMD_onoremenu | CMD_ounmenu | CMD_imenu | CMD_inoremenu | CMD_iunmenu
            | CMD_cmenu | CMD_cnoremenu | CMD_cunmenu | CMD_tlmenu | CMD_tlnoremenu
            | CMD_tlunmenu | CMD_tmenu | CMD_tunmenu | CMD_popup | CMD_emenu => {
                return set_context_in_menu_cmd(xp, cmd, arg as *mut c_char, forceit);
            }

            CMD_colorscheme => {
                (*xp).xp_context = EXPAND_COLORS;
                (*xp).xp_pattern = arg as *mut c_char;
            }
            CMD_compiler => {
                (*xp).xp_context = EXPAND_COMPILER;
                (*xp).xp_pattern = arg as *mut c_char;
            }
            CMD_ownsyntax => {
                (*xp).xp_context = EXPAND_OWNSYNTAX;
                (*xp).xp_pattern = arg as *mut c_char;
            }
            CMD_setfiletype => {
                (*xp).xp_context = EXPAND_FILETYPE;
                (*xp).xp_pattern = arg as *mut c_char;
            }
            CMD_packadd => {
                (*xp).xp_context = EXPAND_PACKADD;
                (*xp).xp_pattern = arg as *mut c_char;
            }

            CMD_runtime => set_context_in_runtime_cmd(xp, arg),

            CMD_language => return set_context_in_lang_cmd(xp, arg),

            CMD_profile => set_context_in_profile_cmd(xp, arg),

            CMD_checkhealth => (*xp).xp_context = EXPAND_CHECKHEALTH,
            CMD_lsp => (*xp).xp_context = EXPAND_LSP,

            CMD_retab => {
                (*xp).xp_context = EXPAND_RETAB;
                (*xp).xp_pattern = arg as *mut c_char;
            }
            CMD_messages => {
                (*xp).xp_context = EXPAND_MESSAGES;
                (*xp).xp_pattern = arg as *mut c_char;
            }
            CMD_history => {
                (*xp).xp_context = EXPAND_HISTORY;
                (*xp).xp_pattern = arg as *mut c_char;
            }
            CMD_syntime => {
                (*xp).xp_context = EXPAND_SYNTIME;
                (*xp).xp_pattern = arg as *mut c_char;
            }

            CMD_argdelete => {
                loop {
                    (*xp).xp_pattern = vim_strchr(arg, ' ' as c_int);
                    if (*xp).xp_pattern.is_null() {
                        break;
                    }
                    arg = (*xp).xp_pattern.add(1);
                }
                (*xp).xp_context = EXPAND_ARGLIST;
                (*xp).xp_pattern = arg as *mut c_char;
            }

            CMD_breakadd | CMD_profdel | CMD_breakdel => {
                return set_context_in_breakadd_cmd(xp, arg, cmdidx);
            }

            CMD_scriptnames => return set_context_in_scriptnames_cmd(xp, arg),

            CMD_filetype => return set_context_in_filetype_cmd(xp, arg),

            CMD_lua | CMD_equal => (*xp).xp_context = EXPAND_LUA,

            _ => {}
        }
        ptr::null()
    }
}

/// Walk one command's arguments and set the context for the one the cursor is
/// in.
///
/// This is all pretty much copied from `do_one_cmd()`, with all the extra
/// stuff we don't need/want deleted.  Maybe this could be done better if we
/// didn't repeat all this stuff.  The only problem is that they may not stay
/// perfectly compatible with each other, but then the command line syntax
/// probably won't change that much -- webb.
///
/// `buff` is the command string.
pub(crate) unsafe fn set_one_cmd_context(xp: *mut expand_T, buff: *const c_char) -> *const c_char {
    unsafe {
        let mut ea = exarg_T {
            cmdidx: CMD_append,
            addr_type: ADDR_LINES,
            ..core::mem::zeroed()
        };
        let mut context: c_int = EXPAND_NOTHING;
        let mut forceit = false;
        let mut usefilter = false; // Filter instead of file name.

        ExpandInit(xp);
        (*xp).xp_pattern = buff as *mut c_char;
        (*xp).xp_line = buff as *mut c_char;
        (*xp).xp_context = EXPAND_COMMANDS; // Default until we get past command
        ea.argt = 0;

        // 1. skip comment lines and leading space, colons or bars
        let mut cmd: *const c_char = buff;
        while !vim_strchr(c" \t:|".as_ptr(), *cmd as u8 as c_int).is_null() {
            cmd = cmd.add(1);
        }
        (*xp).xp_pattern = cmd as *mut c_char;

        if *cmd as c_int == NUL {
            return ptr::null();
        }
        if *cmd as c_int == '"' as c_int {
            // Ignore comment lines.
            (*xp).xp_context = EXPAND_NOTHING;
            return ptr::null();
        }

        // 3. skip over a range specifier of the form: addr [,addr] [;addr] ..
        cmd = skip_range(cmd, &raw mut (*xp).xp_context);
        (*xp).xp_pattern = cmd as *mut c_char;
        if *cmd as c_int == NUL {
            return ptr::null();
        }
        if *cmd as c_int == '"' as c_int {
            (*xp).xp_context = EXPAND_NOTHING;
            return ptr::null();
        }

        if *cmd as c_int == '|' as c_int || *cmd as c_int == '\n' as c_int {
            return cmd.add(1); // There's another command
        }

        // Get the command index.
        let mut p = set_cmd_index(cmd, &raw mut ea, xp, &raw mut context);
        if p.is_null() {
            return ptr::null();
        }

        (*xp).xp_context = EXPAND_NOTHING; // Default now that we're past command

        if *p as c_int == '!' as c_int {
            // Forced commands.
            forceit = true;
            p = p.add(1);
        }

        // 6. parse arguments
        if ea.cmdidx >= 0 {
            ea.argt = excmd_get_argt(ea.cmdidx);
        }

        let mut arg = skipwhite(p);

        // Does command allow "++argopt" argument?
        if ea.argt & EX_ARGOPT != 0 {
            while *arg as c_int != NUL && strncmp(arg, c"++".as_ptr(), 2) == 0 {
                p = arg.add(2);
                while *p != 0 && !ascii_isspace(*p as c_int) {
                    p = p.add(utfc_ptr2len(p) as usize);
                }

                // Still touching the command after "++"?
                if *p as c_int == NUL && ea.argt & EX_ARGOPT != 0 {
                    return set_context_in_argopt(xp, arg.add(2));
                }

                arg = skipwhite(p);
            }
        }

        if ea.cmdidx == CMD_write || ea.cmdidx == CMD_update {
            if *arg as c_int == '>' as c_int {
                // Append.
                arg = arg.add(1);
                if *arg as c_int == '>' as c_int {
                    arg = arg.add(1);
                }
                arg = skipwhite(arg);
            } else if *arg as c_int == '!' as c_int && ea.cmdidx == CMD_write {
                // :w !filter
                arg = arg.add(1);
                usefilter = true;
            }
        }

        if ea.cmdidx == CMD_read {
            usefilter = forceit; // :r! filter if forced
            if *arg as c_int == '!' as c_int {
                // :r !filter
                arg = arg.add(1);
                usefilter = true;
            }
        }

        if ea.cmdidx == CMD_lshift || ea.cmdidx == CMD_rshift {
            // Allow any number of '>' or '<'.
            while *arg as c_int == *cmd as c_int {
                arg = arg.add(1);
            }
            arg = skipwhite(arg);
        }

        // Does command allow "+command"?
        if ea.argt & EX_CMDARG != 0 && !usefilter && *arg as c_int == '+' as c_int {
            // Check if we're in the +command.
            p = arg.add(1);
            arg = skip_cmd_arg(arg as *mut c_char, false);

            // Still touching the command after '+'?
            if *arg as c_int == NUL {
                return p;
            }

            // Skip space(s) after +command to get to the real argument.
            arg = skipwhite(arg);
        }

        // Check for '|' to separate commands and '"' to start comments.
        // Don't do this for ":read !cmd" and ":write !cmd".
        if ea.argt & EX_TRLBAR != 0 && !usefilter {
            p = arg;
            // ":redir @" is not the start of a comment.
            if ea.cmdidx == CMD_redir
                && *p as c_int == '@' as c_int
                && *p.add(1) as c_int == '"' as c_int
            {
                p = p.add(2);
            }
            while *p != 0 {
                if *p as c_int == Ctrl_V {
                    if *p.add(1) as c_int != NUL {
                        p = p.add(1);
                    }
                } else if (*p as c_int == '"' as c_int && ea.argt & EX_NOTRLCOM == 0)
                    || *p as c_int == '|' as c_int
                    || *p as c_int == '\n' as c_int
                {
                    if *p.sub(1) as c_int != '\\' as c_int {
                        if *p as c_int == '|' as c_int || *p as c_int == '\n' as c_int {
                            return p.add(1);
                        }
                        return ptr::null(); // It's a comment
                    }
                }
                p = p.add(utfc_ptr2len(p) as usize);
            }
        }

        if ea.argt & EX_EXTRA == 0
            && *arg as c_int != NUL
            && strchr(c"|\"".as_ptr(), *arg as c_int).is_null()
        {
            // No arguments allowed but there is something.
            return ptr::null();
        }

        // Find start of last argument (argument just before cursor).
        p = buff;
        (*xp).xp_pattern = p as *mut c_char;
        let len = strlen(buff);
        while *p != 0 && p < buff.add(len) {
            if *p as c_int == ' ' as c_int || *p as c_int == TAB {
                // Argument starts after a space.
                p = p.add(1);
                (*xp).xp_pattern = p as *mut c_char;
            } else {
                if *p as c_int == '\\' as c_int && *p.add(1) as c_int != NUL {
                    p = p.add(1); // Skip over escaped character.
                }
                p = p.add(utfc_ptr2len(p) as usize);
            }
        }

        if ea.argt & EX_XFILE != 0 {
            set_context_for_wildcard_arg(&raw mut ea, arg, usefilter, xp, &raw mut context);
        }

        // Switch on command name.
        set_context_by_cmdname(cmd, ea.cmdidx, xp, arg, ea.argt, context, forceit)
    }
}

/// Set the completion context in `xp` for command line `str`.
///
/// `len` is the length of the command line excluding the NUL, `col` the cursor
/// position, and `use_ccline` asks for the command line info to be consulted.
pub unsafe fn set_cmd_context(
    xp: *mut expand_T,
    str: *mut c_char,
    len: c_int,
    col: c_int,
    use_ccline: bool,
) {
    unsafe {
        let ccline: *mut CmdlineInfo = get_cmdline_info();
        let mut old_char = NUL as c_char;

        // Avoid a UMR warning from Purify, only save the character if it has
        // been written before.
        if col < len {
            old_char = *str.offset(col as isize);
        }
        *str.offset(col as isize) = NUL as c_char;

        if use_ccline && (*ccline).cmdfirstc == '=' as c_int {
            // Pass CMD_SIZE because there is no real command.
            set_context_for_expression(xp, str, CMD_SIZE);
        } else if use_ccline && (*ccline).input_fn != 0 {
            (*xp).xp_context = (*ccline).xp_context;
            (*xp).xp_pattern = (*ccline).cmdbuff;
            (*xp).xp_arg = (*ccline).xp_arg;
            if (*xp).xp_context == EXPAND_SHELLCMDLINE {
                let mut context = (*xp).xp_context;
                set_context_for_wildcard_arg(
                    ptr::null_mut(),
                    (*xp).xp_pattern,
                    false,
                    xp,
                    &raw mut context,
                );
            }
        } else {
            let mut nextcomm: *const c_char = str;
            while !nextcomm.is_null() {
                nextcomm = set_one_cmd_context(xp, nextcomm);
            }
        }

        // Store the string here so that call_user_expand_func() can get to
        // them easily.
        (*xp).xp_line = str;
        (*xp).xp_col = col;

        *str.offset(col as isize) = old_char;
    }
}

/// Expand the command line `str` from context `xp`, which must have been set
/// by [`set_cmd_context`].
///
/// `xp->xp_pattern` points into `str`, to where the text that is to be
/// expanded starts.  `matchcount` and `matches` return the answer.
///
/// Returns `EXPAND_UNSUCCESSFUL` when there is something illegal before the
/// cursor, `EXPAND_NOTHING` when there is nothing to expand — the caller may
/// then insert the key that triggered expansion literally — and `EXPAND_OK`
/// otherwise.
pub unsafe fn expand_cmdline(
    xp: *mut expand_T,
    str: *const c_char,
    col: c_int,
    matchcount: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    unsafe {
        let mut options = WILD_ADD_SLASH | WILD_SILENT;

        if (*xp).xp_context == EXPAND_UNSUCCESSFUL {
            beep_flush();
            return EXPAND_UNSUCCESSFUL; // Something illegal on command line
        }
        if (*xp).xp_context == EXPAND_NOTHING {
            // Caller can use the character as a normal char instead.
            return EXPAND_NOTHING;
        }

        // Add star to file name, or convert to regexp if not expanding files.
        debug_assert!(str.offset(col as isize).offset_from((*xp).xp_pattern) >= 0);
        (*xp).xp_pattern_len = str.offset(col as isize).offset_from((*xp).xp_pattern) as size_t;
        let file_str = if cmdline_fuzzy_completion_supported(xp) {
            // If fuzzy matching, don't modify the search string.
            xstrdup((*xp).xp_pattern)
        } else {
            addstar((*xp).xp_pattern, (*xp).xp_pattern_len, (*xp).xp_context)
        };

        if p_wic.get() != 0 {
            options += WILD_ICASE;
        }

        // Find all files that match the description.
        if ExpandFromContext(xp, file_str, matches, matchcount, options) == FAIL {
            *matchcount = 0;
            *matches = ptr::null_mut();
        }
        xfree(file_str as *mut c_void);

        EXPAND_OK
    }
}
