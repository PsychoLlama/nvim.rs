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
use crate::ex_docmd::is_user_cmd;
use crate::keycodes::Ctrl_V;
use crate::types::CmdIdx;
use crate::types::{ExArgt, ExpandContext, NUL, OptionSetFlags};
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
    cmdidx: CmdIdx,
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
        CmdIdx::find | CmdIdx::sfind | CmdIdx::tabfind => {
            if xp.xp_context == ExpandContext::Files {
                xp.xp_context = if unsafe { *get_findfunc() } as c_int != NUL {
                    ExpandContext::Findfunc
                } else {
                    ExpandContext::FilesInPath
                };
            }
        }
        CmdIdx::cd
        | CmdIdx::chdir
        | CmdIdx::lcd
        | CmdIdx::lchdir
        | CmdIdx::tcd
        | CmdIdx::tchdir => {
            if xp.xp_context == ExpandContext::Files {
                xp.xp_context = ExpandContext::DirsInCdpath;
            }
        }
        CmdIdx::help => {
            xp.xp_context = ExpandContext::Help;
            xp.xp_pattern = arg as *mut c_char;
        }

        // Command modifiers: return the argument.  Also for commands with
        // an argument that is a command.
        CmdIdx::aboveleft
        | CmdIdx::argdo
        | CmdIdx::belowright
        | CmdIdx::botright
        | CmdIdx::browse
        | CmdIdx::bufdo
        | CmdIdx::cdo
        | CmdIdx::cfdo
        | CmdIdx::confirm
        | CmdIdx::debug
        | CmdIdx::folddoclosed
        | CmdIdx::folddoopen
        | CmdIdx::hide
        | CmdIdx::horizontal
        | CmdIdx::keepalt
        | CmdIdx::keepjumps
        | CmdIdx::keepmarks
        | CmdIdx::keeppatterns
        | CmdIdx::ldo
        | CmdIdx::leftabove
        | CmdIdx::lfdo
        | CmdIdx::lockmarks
        | CmdIdx::noautocmd
        | CmdIdx::noswapfile
        | CmdIdx::restart
        | CmdIdx::rightbelow
        | CmdIdx::sandbox
        | CmdIdx::silent
        | CmdIdx::tab
        | CmdIdx::tabdo
        | CmdIdx::topleft
        | CmdIdx::unsilent
        | CmdIdx::verbose
        | CmdIdx::vertical
        | CmdIdx::windo => {
            return arg;
        }

        CmdIdx::filter => return unsafe { set_context_in_filter_cmd(xp.raw(), arg) },

        CmdIdx::r#match => return unsafe { set_context_in_match_cmd(xp.raw(), arg) },

        // All completion for the +cmdline_compl feature goes here.
        CmdIdx::command => return unsafe { set_context_in_user_cmd(xp.raw(), arg) },

        CmdIdx::delcommand => {
            xp.xp_context = ExpandContext::UserCommands;
            xp.xp_pattern = arg as *mut c_char;
        }

        CmdIdx::global | CmdIdx::vglobal => {
            let nextcmd = unsafe { find_cmd_after_global_cmd(arg) };
            if nextcmd.is_null() && may_expand_pattern.get() {
                unsafe { set_context_with_pattern(xp.raw()) };
            }
            return nextcmd;
        }

        CmdIdx::and | CmdIdx::substitute => {
            let nextcmd = unsafe { find_cmd_after_substitute_cmd(arg) };
            if nextcmd.is_null() && may_expand_pattern.get() {
                unsafe { set_context_with_pattern(xp.raw()) };
            }
            return nextcmd;
        }

        CmdIdx::isearch
        | CmdIdx::dsearch
        | CmdIdx::ilist
        | CmdIdx::dlist
        | CmdIdx::ijump
        | CmdIdx::psearch
        | CmdIdx::djump
        | CmdIdx::isplit
        | CmdIdx::dsplit => {
            return unsafe { find_cmd_after_isearch_cmd(xp.raw(), arg) };
        }

        CmdIdx::autocmd => {
            return unsafe { set_context_in_autocmd(xp.raw(), arg as *mut c_char, false) };
        }

        CmdIdx::doautocmd | CmdIdx::doautoall => {
            return unsafe { set_context_in_autocmd(xp.raw(), arg as *mut c_char, true) };
        }

        CmdIdx::set => unsafe {
            set_context_in_set_cmd(xp.raw(), arg as *mut c_char, OptionSetFlags::NONE)
        },
        CmdIdx::setglobal => unsafe {
            set_context_in_set_cmd(xp.raw(), arg as *mut c_char, OptionSetFlags::GLOBAL)
        },
        CmdIdx::setlocal => unsafe {
            set_context_in_set_cmd(xp.raw(), arg as *mut c_char, OptionSetFlags::LOCAL)
        },

        CmdIdx::tag
        | CmdIdx::stag
        | CmdIdx::ptag
        | CmdIdx::ltag
        | CmdIdx::tselect
        | CmdIdx::stselect
        | CmdIdx::ptselect
        | CmdIdx::tjump
        | CmdIdx::stjump
        | CmdIdx::ptjump => {
            xp.xp_context = if wop_flags.get() & kOptWopFlagTagfile as c_uint != 0 {
                ExpandContext::TagsListFiles
            } else {
                ExpandContext::Tags
            };
            xp.xp_pattern = arg as *mut c_char;
        }

        CmdIdx::augroup => {
            xp.xp_context = ExpandContext::Augroup;
            xp.xp_pattern = arg as *mut c_char;
        }

        CmdIdx::syntax => unsafe { set_context_in_syntax_cmd(&mut xp, arg) },

        CmdIdx::r#const
        | CmdIdx::r#let
        | CmdIdx::r#if
        | CmdIdx::elseif
        | CmdIdx::r#while
        | CmdIdx::r#for
        | CmdIdx::echo
        | CmdIdx::echon
        | CmdIdx::execute
        | CmdIdx::echomsg
        | CmdIdx::echoerr
        | CmdIdx::call
        | CmdIdx::r#return
        | CmdIdx::cexpr
        | CmdIdx::caddexpr
        | CmdIdx::cgetexpr
        | CmdIdx::lexpr
        | CmdIdx::laddexpr
        | CmdIdx::lgetexpr => {
            unsafe { set_context_for_expression(xp.raw(), arg as *mut c_char, cmdidx) };
        }

        CmdIdx::unlet => return unsafe { set_context_in_unlet_cmd(xp.raw(), arg) },

        CmdIdx::function | CmdIdx::delfunction => {
            xp.xp_context = ExpandContext::UserFunc;
            xp.xp_pattern = arg as *mut c_char;
        }

        CmdIdx::echohl => set_context_in_echohl_cmd(&mut xp, arg),
        CmdIdx::highlight => unsafe { set_context_in_highlight_cmd(xp.raw(), arg) },
        CmdIdx::sign => unsafe { set_context_in_sign_cmd(xp.raw(), arg as *mut c_char) },

        CmdIdx::bdelete | CmdIdx::bwipeout | CmdIdx::bunload => {
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
        CmdIdx::buffer | CmdIdx::sbuffer | CmdIdx::pbuffer | CmdIdx::checktime => {
            xp.xp_context = ExpandContext::Buffers;
            xp.xp_pattern = arg as *mut c_char;
        }

        CmdIdx::diffget | CmdIdx::diffput => {
            // If current buffer is in diff mode, complete buffer names
            // which are in diff mode, and different than current buffer.
            xp.xp_context = ExpandContext::DiffBuffers;
            xp.xp_pattern = arg as *mut c_char;
        }

        CmdIdx::USER | CmdIdx::USER_BUF => {
            return unsafe {
                set_context_in_user_cmdarg(cmd, arg, argt, context, xp.raw(), forceit)
            };
        }

        CmdIdx::map
        | CmdIdx::noremap
        | CmdIdx::nmap
        | CmdIdx::nnoremap
        | CmdIdx::vmap
        | CmdIdx::vnoremap
        | CmdIdx::omap
        | CmdIdx::onoremap
        | CmdIdx::imap
        | CmdIdx::inoremap
        | CmdIdx::cmap
        | CmdIdx::cnoremap
        | CmdIdx::lmap
        | CmdIdx::lnoremap
        | CmdIdx::smap
        | CmdIdx::snoremap
        | CmdIdx::xmap
        | CmdIdx::xnoremap => {
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
        CmdIdx::unmap
        | CmdIdx::nunmap
        | CmdIdx::vunmap
        | CmdIdx::ounmap
        | CmdIdx::iunmap
        | CmdIdx::cunmap
        | CmdIdx::lunmap
        | CmdIdx::sunmap
        | CmdIdx::xunmap => {
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
        CmdIdx::mapclear
        | CmdIdx::nmapclear
        | CmdIdx::vmapclear
        | CmdIdx::omapclear
        | CmdIdx::imapclear
        | CmdIdx::cmapclear
        | CmdIdx::lmapclear
        | CmdIdx::smapclear
        | CmdIdx::xmapclear => {
            xp.xp_context = ExpandContext::Mapclear;
            xp.xp_pattern = arg as *mut c_char;
        }

        CmdIdx::abbreviate
        | CmdIdx::noreabbrev
        | CmdIdx::cabbrev
        | CmdIdx::cnoreabbrev
        | CmdIdx::iabbrev
        | CmdIdx::inoreabbrev => {
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
        CmdIdx::unabbreviate | CmdIdx::cunabbrev | CmdIdx::iunabbrev => {
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

        CmdIdx::menu
        | CmdIdx::noremenu
        | CmdIdx::unmenu
        | CmdIdx::amenu
        | CmdIdx::anoremenu
        | CmdIdx::aunmenu
        | CmdIdx::nmenu
        | CmdIdx::nnoremenu
        | CmdIdx::nunmenu
        | CmdIdx::vmenu
        | CmdIdx::vnoremenu
        | CmdIdx::vunmenu
        | CmdIdx::omenu
        | CmdIdx::onoremenu
        | CmdIdx::ounmenu
        | CmdIdx::imenu
        | CmdIdx::inoremenu
        | CmdIdx::iunmenu
        | CmdIdx::cmenu
        | CmdIdx::cnoremenu
        | CmdIdx::cunmenu
        | CmdIdx::tlmenu
        | CmdIdx::tlnoremenu
        | CmdIdx::tlunmenu
        | CmdIdx::tmenu
        | CmdIdx::tunmenu
        | CmdIdx::popup
        | CmdIdx::emenu => {
            return unsafe { set_context_in_menu_cmd(xp.raw(), cmd, arg as *mut c_char, forceit) };
        }

        CmdIdx::colorscheme => {
            xp.xp_context = ExpandContext::Colors;
            xp.xp_pattern = arg as *mut c_char;
        }
        CmdIdx::compiler => {
            xp.xp_context = ExpandContext::Compiler;
            xp.xp_pattern = arg as *mut c_char;
        }
        CmdIdx::ownsyntax => {
            xp.xp_context = ExpandContext::Ownsyntax;
            xp.xp_pattern = arg as *mut c_char;
        }
        CmdIdx::setfiletype => {
            xp.xp_context = ExpandContext::Filetype;
            xp.xp_pattern = arg as *mut c_char;
        }
        CmdIdx::packadd => {
            xp.xp_context = ExpandContext::Packadd;
            xp.xp_pattern = arg as *mut c_char;
        }

        CmdIdx::runtime => unsafe { set_context_in_runtime_cmd(xp.raw(), arg) },

        CmdIdx::language => return unsafe { set_context_in_lang_cmd(xp.raw(), arg) },

        CmdIdx::profile => unsafe { set_context_in_profile_cmd(xp.raw(), arg) },

        CmdIdx::checkhealth => xp.xp_context = ExpandContext::Checkhealth,
        CmdIdx::lsp => xp.xp_context = ExpandContext::Lsp,

        CmdIdx::retab => {
            xp.xp_context = ExpandContext::Retab;
            xp.xp_pattern = arg as *mut c_char;
        }
        CmdIdx::messages => {
            xp.xp_context = ExpandContext::Messages;
            xp.xp_pattern = arg as *mut c_char;
        }
        CmdIdx::history => {
            xp.xp_context = ExpandContext::History;
            xp.xp_pattern = arg as *mut c_char;
        }
        CmdIdx::syntime => {
            xp.xp_context = ExpandContext::Syntime;
            xp.xp_pattern = arg as *mut c_char;
        }

        CmdIdx::argdelete => {
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

        CmdIdx::breakadd | CmdIdx::profdel | CmdIdx::breakdel => {
            return unsafe { set_context_in_breakadd_cmd(xp.raw(), arg, cmdidx) };
        }

        CmdIdx::scriptnames => return unsafe { set_context_in_scriptnames_cmd(xp.raw(), arg) },

        CmdIdx::filetype => return unsafe { set_context_in_filetype_cmd(xp.raw(), arg) },

        CmdIdx::lua | CmdIdx::equal => xp.xp_context = ExpandContext::Lua,

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
        cmdidx: CmdIdx::append,
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
    if !is_user_cmd(ea.cmdidx) {
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

    if ea.cmdidx == CmdIdx::write || ea.cmdidx == CmdIdx::update {
        if unsafe { *arg } as c_int == '>' as c_int {
            // Append.
            arg = unsafe { arg.add(1) };
            if unsafe { *arg } as c_int == '>' as c_int {
                arg = unsafe { arg.add(1) };
            }
            arg = unsafe { skipwhite(arg) };
        } else if unsafe { *arg } as c_int == '!' as c_int && ea.cmdidx == CmdIdx::write {
            // :w !filter
            arg = unsafe { arg.add(1) };
            usefilter = true;
        }
    }

    if ea.cmdidx == CmdIdx::read {
        usefilter = forceit; // :r! filter if forced
        if unsafe { *arg } as c_int == '!' as c_int {
            // :r !filter
            arg = unsafe { arg.add(1) };
            usefilter = true;
        }
    }

    if ea.cmdidx == CmdIdx::lshift || ea.cmdidx == CmdIdx::rshift {
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
        if ea.cmdidx == CmdIdx::redir
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
    let len = unsafe { cstr::bytes_at(buff) }.len();
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
        // Pass CmdIdx::SIZE because there is no real command.
        unsafe { set_context_for_expression(xp.raw(), str, CmdIdx::SIZE) };
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
