//! The per-command context switch.
//!
//! [`set_context_by_cmdname`] is C's `set_context_by_cmdname`: one arm per
//! `CMD_*` whose argument has its own completion.  [`set_one_cmd_context`]
//! walks a single command's arguments to find the one the cursor is in, and
//! [`set_cmd_context`] / [`expand_cmdline`] are the two entry points over
//! both.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cmdexpand::WildOpts;
use crate::cstr;
use crate::keycodes::Ctrl_V;
use crate::types::{
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
    CMD_windo, CMD_write, CMD_xmap, CMD_xmapclear, CMD_xnoremap, CMD_xunmap, ExArgt, ExpandContext,
    NUL, OptionSetFlags,
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
    argt: ExArgt,
    context: ExpandContext,
    forceit: bool,
) -> *const c_char {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    match cmdidx {
        CMD_find | CMD_sfind | CMD_tabfind => {
            if xp.xp_context == ExpandContext::Files {
                xp.xp_context = if unsafe { *get_findfunc() } as c_int != NUL {
                    ExpandContext::Findfunc
                } else {
                    ExpandContext::FilesInPath
                };
            }
        }
        CMD_cd | CMD_chdir | CMD_lcd | CMD_lchdir | CMD_tcd | CMD_tchdir => {
            if xp.xp_context == ExpandContext::Files {
                xp.xp_context = ExpandContext::DirsInCdpath;
            }
        }
        CMD_help => {
            xp.xp_context = ExpandContext::Help;
            xp.xp_pattern = arg as *mut c_char;
        }

        // Command modifiers: return the argument.  Also for commands with
        // an argument that is a command.
        CMD_aboveleft | CMD_argdo | CMD_belowright | CMD_botright | CMD_browse | CMD_bufdo
        | CMD_cdo | CMD_cfdo | CMD_confirm | CMD_debug | CMD_folddoclosed | CMD_folddoopen
        | CMD_hide | CMD_horizontal | CMD_keepalt | CMD_keepjumps | CMD_keepmarks
        | CMD_keeppatterns | CMD_ldo | CMD_leftabove | CMD_lfdo | CMD_lockmarks | CMD_noautocmd
        | CMD_noswapfile | CMD_restart | CMD_rightbelow | CMD_sandbox | CMD_silent | CMD_tab
        | CMD_tabdo | CMD_topleft | CMD_unsilent | CMD_verbose | CMD_vertical | CMD_windo => {
            return arg;
        }

        CMD_filter => return unsafe { set_context_in_filter_cmd(xp.raw(), arg) },

        CMD_match => return unsafe { set_context_in_match_cmd(xp.raw(), arg) },

        // All completion for the +cmdline_compl feature goes here.
        CMD_command => return unsafe { set_context_in_user_cmd(xp.raw(), arg) },

        CMD_delcommand => {
            xp.xp_context = ExpandContext::UserCommands;
            xp.xp_pattern = arg as *mut c_char;
        }

        CMD_global | CMD_vglobal => {
            let nextcmd = unsafe { find_cmd_after_global_cmd(arg) };
            if nextcmd.is_null() && may_expand_pattern.get() {
                unsafe { set_context_with_pattern(xp.raw()) };
            }
            return nextcmd;
        }

        CMD_and | CMD_substitute => {
            let nextcmd = unsafe { find_cmd_after_substitute_cmd(arg) };
            if nextcmd.is_null() && may_expand_pattern.get() {
                unsafe { set_context_with_pattern(xp.raw()) };
            }
            return nextcmd;
        }

        CMD_isearch | CMD_dsearch | CMD_ilist | CMD_dlist | CMD_ijump | CMD_psearch | CMD_djump
        | CMD_isplit | CMD_dsplit => {
            return unsafe { find_cmd_after_isearch_cmd(xp.raw(), arg) };
        }

        CMD_autocmd => {
            return unsafe { set_context_in_autocmd(xp.raw(), arg as *mut c_char, false) };
        }

        CMD_doautocmd | CMD_doautoall => {
            return unsafe { set_context_in_autocmd(xp.raw(), arg as *mut c_char, true) };
        }

        CMD_set => unsafe {
            set_context_in_set_cmd(xp.raw(), arg as *mut c_char, OptionSetFlags::NONE)
        },
        CMD_setglobal => unsafe {
            set_context_in_set_cmd(xp.raw(), arg as *mut c_char, OptionSetFlags::GLOBAL)
        },
        CMD_setlocal => unsafe {
            set_context_in_set_cmd(xp.raw(), arg as *mut c_char, OptionSetFlags::LOCAL)
        },

        CMD_tag | CMD_stag | CMD_ptag | CMD_ltag | CMD_tselect | CMD_stselect | CMD_ptselect
        | CMD_tjump | CMD_stjump | CMD_ptjump => {
            xp.xp_context = if wop_flags.get() & kOptWopFlagTagfile as c_uint != 0 {
                ExpandContext::TagsListFiles
            } else {
                ExpandContext::Tags
            };
            xp.xp_pattern = arg as *mut c_char;
        }

        CMD_augroup => {
            xp.xp_context = ExpandContext::Augroup;
            xp.xp_pattern = arg as *mut c_char;
        }

        CMD_syntax => unsafe { set_context_in_syntax_cmd(xp.raw(), arg) },

        CMD_const | CMD_let | CMD_if | CMD_elseif | CMD_while | CMD_for | CMD_echo | CMD_echon
        | CMD_execute | CMD_echomsg | CMD_echoerr | CMD_call | CMD_return | CMD_cexpr
        | CMD_caddexpr | CMD_cgetexpr | CMD_lexpr | CMD_laddexpr | CMD_lgetexpr => {
            unsafe { set_context_for_expression(xp.raw(), arg as *mut c_char, cmdidx) };
        }

        CMD_unlet => return unsafe { set_context_in_unlet_cmd(xp.raw(), arg) },

        CMD_function | CMD_delfunction => {
            xp.xp_context = ExpandContext::UserFunc;
            xp.xp_pattern = arg as *mut c_char;
        }

        CMD_echohl => unsafe { set_context_in_echohl_cmd(xp.raw(), arg) },
        CMD_highlight => unsafe { set_context_in_highlight_cmd(xp.raw(), arg) },
        CMD_sign => unsafe { set_context_in_sign_cmd(xp.raw(), arg as *mut c_char) },

        CMD_bdelete | CMD_bwipeout | CMD_bunload => {
            // Only the argument the cursor is in is completed.  Upstream
            // falls through into the buffer-name arm below.
            loop {
                xp.xp_pattern = unsafe { strchr(arg, ' ' as c_int) };
                if xp.xp_pattern.is_null() {
                    break;
                }
                arg = unsafe { xp.xp_pattern.add(1) };
            }
            xp.xp_context = ExpandContext::Buffers;
            xp.xp_pattern = arg as *mut c_char;
        }
        CMD_buffer | CMD_sbuffer | CMD_pbuffer | CMD_checktime => {
            xp.xp_context = ExpandContext::Buffers;
            xp.xp_pattern = arg as *mut c_char;
        }

        CMD_diffget | CMD_diffput => {
            // If current buffer is in diff mode, complete buffer names
            // which are in diff mode, and different than current buffer.
            xp.xp_context = ExpandContext::DiffBuffers;
            xp.xp_pattern = arg as *mut c_char;
        }

        CMD_USER | CMD_USER_BUF => {
            return unsafe {
                set_context_in_user_cmdarg(cmd, arg, argt, context, xp.raw(), forceit)
            };
        }

        CMD_map | CMD_noremap | CMD_nmap | CMD_nnoremap | CMD_vmap | CMD_vnoremap | CMD_omap
        | CMD_onoremap | CMD_imap | CMD_inoremap | CMD_cmap | CMD_cnoremap | CMD_lmap
        | CMD_lnoremap | CMD_smap | CMD_snoremap | CMD_xmap | CMD_xnoremap => {
            return unsafe {
                set_context_in_map_cmd(
                    xp.raw(),
                    cmd as *mut c_char,
                    arg as *mut c_char,
                    forceit,
                    false,
                    false,
                    cmdidx,
                )
            };
        }
        CMD_unmap | CMD_nunmap | CMD_vunmap | CMD_ounmap | CMD_iunmap | CMD_cunmap | CMD_lunmap
        | CMD_sunmap | CMD_xunmap => {
            return unsafe {
                set_context_in_map_cmd(
                    xp.raw(),
                    cmd as *mut c_char,
                    arg as *mut c_char,
                    forceit,
                    false,
                    true,
                    cmdidx,
                )
            };
        }
        CMD_mapclear | CMD_nmapclear | CMD_vmapclear | CMD_omapclear | CMD_imapclear
        | CMD_cmapclear | CMD_lmapclear | CMD_smapclear | CMD_xmapclear => {
            xp.xp_context = ExpandContext::Mapclear;
            xp.xp_pattern = arg as *mut c_char;
        }

        CMD_abbreviate | CMD_noreabbrev | CMD_cabbrev | CMD_cnoreabbrev | CMD_iabbrev
        | CMD_inoreabbrev => {
            return unsafe {
                set_context_in_map_cmd(
                    xp.raw(),
                    cmd as *mut c_char,
                    arg as *mut c_char,
                    forceit,
                    true,
                    false,
                    cmdidx,
                )
            };
        }
        CMD_unabbreviate | CMD_cunabbrev | CMD_iunabbrev => {
            return unsafe {
                set_context_in_map_cmd(
                    xp.raw(),
                    cmd as *mut c_char,
                    arg as *mut c_char,
                    forceit,
                    true,
                    true,
                    cmdidx,
                )
            };
        }

        CMD_menu | CMD_noremenu | CMD_unmenu | CMD_amenu | CMD_anoremenu | CMD_aunmenu
        | CMD_nmenu | CMD_nnoremenu | CMD_nunmenu | CMD_vmenu | CMD_vnoremenu | CMD_vunmenu
        | CMD_omenu | CMD_onoremenu | CMD_ounmenu | CMD_imenu | CMD_inoremenu | CMD_iunmenu
        | CMD_cmenu | CMD_cnoremenu | CMD_cunmenu | CMD_tlmenu | CMD_tlnoremenu | CMD_tlunmenu
        | CMD_tmenu | CMD_tunmenu | CMD_popup | CMD_emenu => {
            return unsafe { set_context_in_menu_cmd(xp.raw(), cmd, arg as *mut c_char, forceit) };
        }

        CMD_colorscheme => {
            xp.xp_context = ExpandContext::Colors;
            xp.xp_pattern = arg as *mut c_char;
        }
        CMD_compiler => {
            xp.xp_context = ExpandContext::Compiler;
            xp.xp_pattern = arg as *mut c_char;
        }
        CMD_ownsyntax => {
            xp.xp_context = ExpandContext::Ownsyntax;
            xp.xp_pattern = arg as *mut c_char;
        }
        CMD_setfiletype => {
            xp.xp_context = ExpandContext::Filetype;
            xp.xp_pattern = arg as *mut c_char;
        }
        CMD_packadd => {
            xp.xp_context = ExpandContext::Packadd;
            xp.xp_pattern = arg as *mut c_char;
        }

        CMD_runtime => unsafe { set_context_in_runtime_cmd(xp.raw(), arg) },

        CMD_language => return unsafe { set_context_in_lang_cmd(xp.raw(), arg) },

        CMD_profile => unsafe { set_context_in_profile_cmd(xp.raw(), arg) },

        CMD_checkhealth => xp.xp_context = ExpandContext::Checkhealth,
        CMD_lsp => xp.xp_context = ExpandContext::Lsp,

        CMD_retab => {
            xp.xp_context = ExpandContext::Retab;
            xp.xp_pattern = arg as *mut c_char;
        }
        CMD_messages => {
            xp.xp_context = ExpandContext::Messages;
            xp.xp_pattern = arg as *mut c_char;
        }
        CMD_history => {
            xp.xp_context = ExpandContext::History;
            xp.xp_pattern = arg as *mut c_char;
        }
        CMD_syntime => {
            xp.xp_context = ExpandContext::Syntime;
            xp.xp_pattern = arg as *mut c_char;
        }

        CMD_argdelete => {
            loop {
                xp.xp_pattern = unsafe { vim_strchr(arg, ' ' as c_int) };
                if xp.xp_pattern.is_null() {
                    break;
                }
                arg = unsafe { xp.xp_pattern.add(1) };
            }
            xp.xp_context = ExpandContext::Arglist;
            xp.xp_pattern = arg as *mut c_char;
        }

        CMD_breakadd | CMD_profdel | CMD_breakdel => {
            return unsafe { set_context_in_breakadd_cmd(xp.raw(), arg, cmdidx) };
        }

        CMD_scriptnames => return unsafe { set_context_in_scriptnames_cmd(xp.raw(), arg) },

        CMD_filetype => return unsafe { set_context_in_filetype_cmd(xp.raw(), arg) },

        CMD_lua | CMD_equal => xp.xp_context = ExpandContext::Lua,

        _ => {}
    }
    ptr::null()
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
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    let mut ea = exarg_T {
        cmdidx: CMD_append,
        addr_type: CmdAddr::Lines,
        ..unsafe { core::mem::zeroed() }
    };
    let mut context = ExpandContext::Nothing;
    let mut forceit = false;
    let mut usefilter = false; // Filter instead of file name.

    unsafe { expand_init(xp.raw()) };
    xp.xp_pattern = buff as *mut c_char;
    xp.xp_line = buff as *mut c_char;
    xp.xp_context = ExpandContext::Commands; // Default until we get past command
    ea.argt = ExArgt::NONE;

    // 1. skip comment lines and leading space, colons or bars
    let mut cmd: *const c_char = buff;
    while !unsafe { vim_strchr(c" \t:|".as_ptr(), *cmd as u8 as c_int) }.is_null() {
        cmd = unsafe { cmd.add(1) };
    }
    xp.xp_pattern = cmd as *mut c_char;

    if unsafe { *cmd } as c_int == NUL {
        return ptr::null();
    }
    if unsafe { *cmd } as c_int == '"' as c_int {
        // Ignore comment lines.
        xp.xp_context = ExpandContext::Nothing;
        return ptr::null();
    }

    // 3. skip over a range specifier of the form: addr [,addr] [;addr] ..
    // `field_ptr`, not `&raw mut xp.xp_context`: an address taken off a
    // `Deref` dies at the next field write, and `skip_range` writes through
    // this one while the walk below keeps reading `xp`.
    let context_field = xp.field_ptr(core::mem::offset_of!(expand_T, xp_context));
    // SAFETY: `cmd` is inside the command line and `context` is the live
    // context's own field.
    cmd = unsafe { skip_range(cmd, context_field) };
    xp.xp_pattern = cmd as *mut c_char;
    if unsafe { *cmd } as c_int == NUL {
        return ptr::null();
    }
    if unsafe { *cmd } as c_int == '"' as c_int {
        xp.xp_context = ExpandContext::Nothing;
        return ptr::null();
    }

    if unsafe { *cmd } as c_int == '|' as c_int || unsafe { *cmd } as c_int == '\n' as c_int {
        return unsafe { cmd.add(1) }; // There's another command
    }

    // Get the command index.
    let mut p = unsafe { set_cmd_index(cmd, &raw mut ea, xp.raw(), &raw mut context) };
    if p.is_null() {
        return ptr::null();
    }

    xp.xp_context = ExpandContext::Nothing; // Default now that we're past command

    if unsafe { *p } as c_int == '!' as c_int {
        // Forced commands.
        forceit = true;
        p = unsafe { p.add(1) };
    }

    // 6. parse arguments
    if ea.cmdidx >= 0 {
        ea.argt = unsafe { excmd_get_argt(ea.cmdidx) };
    }

    let mut arg = unsafe { skipwhite(p) };

    // Does command allow "++argopt" argument?
    if ea.argt.has(ExArgt::ARGOPT) {
        while unsafe { *arg } as c_int != NUL && unsafe { cstr::starts_with(arg, b"++") } {
            p = unsafe { arg.add(2) };
            while unsafe { *p } != 0 && !ascii_isspace(unsafe { *p } as c_int) {
                p = unsafe { p.add(utfc_ptr2len(p) as usize) };
            }

            // Still touching the command after "++"?
            if unsafe { *p } as c_int == NUL && ea.argt.has(ExArgt::ARGOPT) {
                return unsafe { set_context_in_argopt(xp.raw(), arg.add(2)) };
            }

            arg = unsafe { skipwhite(p) };
        }
    }

    if ea.cmdidx == CMD_write || ea.cmdidx == CMD_update {
        if unsafe { *arg } as c_int == '>' as c_int {
            // Append.
            arg = unsafe { arg.add(1) };
            if unsafe { *arg } as c_int == '>' as c_int {
                arg = unsafe { arg.add(1) };
            }
            arg = unsafe { skipwhite(arg) };
        } else if unsafe { *arg } as c_int == '!' as c_int && ea.cmdidx == CMD_write {
            // :w !filter
            arg = unsafe { arg.add(1) };
            usefilter = true;
        }
    }

    if ea.cmdidx == CMD_read {
        usefilter = forceit; // :r! filter if forced
        if unsafe { *arg } as c_int == '!' as c_int {
            // :r !filter
            arg = unsafe { arg.add(1) };
            usefilter = true;
        }
    }

    if ea.cmdidx == CMD_lshift || ea.cmdidx == CMD_rshift {
        // Allow any number of '>' or '<'.
        while unsafe { *arg } as c_int == unsafe { *cmd } as c_int {
            arg = unsafe { arg.add(1) };
        }
        arg = unsafe { skipwhite(arg) };
    }

    // Does command allow "+command"?
    if ea.argt.has(ExArgt::CMDARG) && !usefilter && unsafe { *arg } as c_int == '+' as c_int {
        // Check if we're in the +command.
        p = unsafe { arg.add(1) };
        arg = unsafe { skip_cmd_arg(arg as *mut c_char, false) };

        // Still touching the command after '+'?
        if unsafe { *arg } as c_int == NUL {
            return p;
        }

        // Skip space(s) after +command to get to the real argument.
        arg = unsafe { skipwhite(arg) };
    }

    // Check for '|' to separate commands and '"' to start comments.
    // Don't do this for ":read !cmd" and ":write !cmd".
    if ea.argt.has(ExArgt::TRLBAR) && !usefilter {
        p = arg;
        // ":redir @" is not the start of a comment.
        if ea.cmdidx == CMD_redir
            && unsafe { *p } as c_int == '@' as c_int
            && unsafe { *p.add(1) } as c_int == '"' as c_int
        {
            p = unsafe { p.add(2) };
        }
        while unsafe { *p } != 0 {
            if unsafe { *p } as c_int == Ctrl_V {
                if unsafe { *p.add(1) } as c_int != NUL {
                    p = unsafe { p.add(1) };
                }
            } else if ((unsafe { *p } as c_int == '"' as c_int && !ea.argt.has(ExArgt::NOTRLCOM))
                || unsafe { *p } as c_int == '|' as c_int
                || unsafe { *p } as c_int == '\n' as c_int)
                && unsafe { *p.sub(1) } as c_int != '\\' as c_int
            {
                if unsafe { *p } as c_int == '|' as c_int || unsafe { *p } as c_int == '\n' as c_int
                {
                    return unsafe { p.add(1) };
                }
                return ptr::null(); // It's a comment
            }
            p = unsafe { p.add(utfc_ptr2len(p) as usize) };
        }
    }

    if !ea.argt.has(ExArgt::EXTRA)
        && unsafe { *arg } as c_int != NUL
        && unsafe { strchr(c"|\"".as_ptr(), *arg as c_int) }.is_null()
    {
        // No arguments allowed but there is something.
        return ptr::null();
    }

    // Find start of last argument (argument just before cursor).
    p = buff;
    xp.xp_pattern = p as *mut c_char;
    let len = unsafe { strlen(buff) };
    while unsafe { *p } != 0 && p < unsafe { buff.add(len) } {
        if unsafe { *p } as c_int == ' ' as c_int || unsafe { *p } as c_int == TAB {
            // Argument starts after a space.
            p = unsafe { p.add(1) };
            xp.xp_pattern = p as *mut c_char;
        } else {
            if unsafe { *p } as c_int == '\\' as c_int && unsafe { *p.add(1) } as c_int != NUL {
                p = unsafe { p.add(1) }; // Skip over escaped character.
            }
            p = unsafe { p.add(utfc_ptr2len(p) as usize) };
        }
    }

    if ea.argt.has(ExArgt::XFILE) {
        unsafe {
            set_context_for_wildcard_arg(&raw mut ea, arg, usefilter, xp.raw(), &raw mut context)
        };
    }

    // Switch on command name.
    unsafe { set_context_by_cmdname(cmd, ea.cmdidx, xp.raw(), arg, ea.argt, context, forceit) }
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
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    let mut ccline = Cc::current();
    let mut old_char = NUL as c_char;

    // Avoid a UMR warning from Purify, only save the character if it has
    // been written before.
    if col < len {
        old_char = unsafe { *str.offset(col as isize) };
    }
    unsafe { *str.offset(col as isize) = NUL as c_char };

    if use_ccline && ccline.cmdfirstc == '=' as c_int {
        // Pass CMD_SIZE because there is no real command.
        unsafe { set_context_for_expression(xp.raw(), str, CMD_SIZE) };
    } else if use_ccline && ccline.input_fn != 0 {
        xp.xp_context = ccline.xp_context;
        xp.xp_pattern = ccline.text();
        xp.xp_arg = ccline.xp_arg;
        if xp.xp_context == ExpandContext::ShellCmdLine {
            let mut context = xp.xp_context;
            unsafe {
                set_context_for_wildcard_arg(
                    ptr::null_mut(),
                    xp.xp_pattern,
                    false,
                    xp.raw(),
                    &raw mut context,
                )
            };
        }
    } else {
        let mut nextcomm: *const c_char = str;
        while !nextcomm.is_null() {
            nextcomm = unsafe { set_one_cmd_context(xp.raw(), nextcomm) };
        }
    }

    // Store the string here so that call_user_expand_func() can get to
    // them easily.
    xp.xp_line = str;
    xp.xp_col = col;

    unsafe { *str.offset(col as isize) = old_char };
}

/// Expand the command line `str` from context `xp`, which must have been set
/// by [`set_cmd_context`].
///
/// `xp->xp_pattern` points into `str`, to where the text that is to be
/// expanded starts.  `matchcount` and `matches` return the answer.
///
/// What an expansion attempt came to.
///
/// Upstream answers through the same `int` as an `xp_context` and reuses
/// three of its names, one of which (`EXPAND_OK`) is not a context at all.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Expanded {
    /// The matches are in the out-parameters.
    Ok,
    /// Nothing to expand — the caller may insert the key that triggered the
    /// expansion literally.
    Nothing,
    /// Something illegal stands before the cursor; the editor has beeped.
    Unsuccessful,
}

/// Expand the command line `str` from context `xp`, which must have been set
/// by [`set_cmd_context`].
///
/// `xp->xp_pattern` points into `str`, to where the text that is to be
/// expanded starts.  `matchcount` and `matches` return the answer.
pub unsafe fn expand_cmdline(
    xp: *mut expand_T,
    str: *const c_char,
    col: c_int,
    matchcount: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> Expanded {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    let mut options = WildOpts::ADD_SLASH | WildOpts::SILENT;

    if xp.xp_context == ExpandContext::Unsuccessful {
        beep_flush();
        return Expanded::Unsuccessful; // Something illegal on command line
    }
    if xp.xp_context == ExpandContext::Nothing {
        // Caller can use the character as a normal char instead.
        return Expanded::Nothing;
    }

    // Add star to file name, or convert to regexp if not expanding files.
    debug_assert!(unsafe { str.offset(col as isize).offset_from(xp.xp_pattern) } >= 0);
    unsafe { xp.xp_pattern_len = str.offset(col as isize).offset_from(xp.xp_pattern) as size_t };
    let file_str = if unsafe { cmdline_fuzzy_completion_supported(xp.raw()) } {
        // If fuzzy matching, don't modify the search string.
        unsafe { xstrdup(xp.xp_pattern) }
    } else {
        unsafe { addstar(xp.xp_pattern, xp.xp_pattern_len, xp.xp_context) }
    };

    if p_wic.get() != 0 {
        options |= WildOpts::ICASE;
    }

    // Find all files that match the description.
    if unsafe { expand_from_context(xp.raw(), file_str, matches, matchcount, options) }.is_err() {
        unsafe { *matchcount = 0 };
        unsafe { *matches = ptr::null_mut() };
    }
    unsafe { xfree(file_str as *mut c_void) };

    Expanded::Ok
}
