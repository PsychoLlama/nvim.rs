//! The per-command context switch.
//!
//! [`set_context_by_cmdname`] is C's `set_context_by_cmdname`: one arm per
//! `CMD_*` whose argument has its own completion.  [`set_one_cmd_context`]
//! walks a single command's arguments to find the one the cursor is in, and
//! [`set_cmd_context`] / [`expand_cmdline`] are the two entry points over
//! both.

#![forbid(unsafe_code)]

use super::*;
use crate::ascii::ascii_isspace;
use crate::autocmd::set_context_in_autocmd;
use crate::cstr;
use crate::eval::set_context_for_expression;
use crate::ex_docmd::{excmd_get_argt, is_user_cmd, skip_cmd_arg_at, skip_range};
use crate::getchar::beep_flush;
use crate::highlight_group::set_context_in_highlight_cmd;
use crate::keycodes::Ctrl_V;
use crate::mapping::set_context_in_map_cmd;
use crate::menu::set_context_in_menu_cmd;
use crate::option::get_findfunc;
use crate::option::set_context_in_set_cmd;
use crate::option::vars::{p_wic, wop_flags};
use crate::options::kOptWopFlagTagfile;
use crate::profile::set_context_in_profile_cmd;
use crate::runtime::set_context_in_runtime_cmd;
use crate::sign::set_context_in_sign_cmd;
use crate::syntax::{set_context_in_echohl_cmd, set_context_in_syntax_cmd};
use crate::types::{CmdAddr, CmdIdx, ExArg, ExArgt, ExpandContext, OptionSetFlags, TAB};
use crate::usercmd::{set_context_in_user_cmd, set_context_in_user_cmdarg};
use crate::winlayer::Cc;
use core::ffi::{CStr, c_int, c_uint};

/// Set the completion context in `expand` for command `cmd` with index `cmdidx`.
///
/// The argument to the command starts at `arg` and the argument flags are
/// `argt`. For user-defined commands and for environment variables, `context`
/// carries the completion type.
///
/// Returns where the next command starts, or `None` when there is none.
#[allow(clippy::too_many_arguments)]
pub(crate) fn set_context_by_cmdname(
    text: &CStr,
    cmd: usize,
    cmdidx: CmdIdx,
    expand: &mut Expand,
    arg: usize,
    argt: ExArgt,
    context: ExpandContext,
    forceit: bool,
) -> Option<usize> {
    let t = text.to_bytes();
    let cmd_name = cstr::suffix(text, cmd);
    // Many commands complete their whole argument in one context.
    let whole_argument = |expand: &mut Expand, context| {
        expand.context = context;
        expand.pattern = arg;
    };
    // A command that takes several names completes only the one the cursor
    // is in, the last.
    let last_word = |from: usize| match t[from..].iter().rposition(|&c| c == b' ') {
        Some(space) => from + space + 1,
        None => from,
    };
    match cmdidx {
        CmdIdx::find | CmdIdx::sfind | CmdIdx::tabfind => {
            if expand.context == ExpandContext::Files {
                expand.context = if !get_findfunc().is_empty() {
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
            if expand.context == ExpandContext::Files {
                expand.context = ExpandContext::DirsInCdpath;
            }
        }
        CmdIdx::help => whole_argument(expand, ExpandContext::Help),

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
            return Some(arg);
        }

        CmdIdx::filter => return set_context_in_filter_cmd(expand, text, arg),

        CmdIdx::r#match => return set_context_in_match_cmd(expand, text, arg),

        // All completion for the +cmdline_compl feature goes here.
        CmdIdx::command => return set_context_in_user_cmd(expand, arg),

        CmdIdx::delcommand => whole_argument(expand, ExpandContext::UserCommands),

        CmdIdx::global | CmdIdx::vglobal => {
            let nextcmd = find_cmd_after_global_cmd(text, arg);
            if nextcmd.is_none() && may_expand_pattern.get() {
                set_context_with_pattern(expand);
            }
            return nextcmd;
        }

        CmdIdx::and | CmdIdx::substitute => {
            let nextcmd = find_cmd_after_substitute_cmd(text, arg);
            if nextcmd.is_none() && may_expand_pattern.get() {
                set_context_with_pattern(expand);
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
            return find_cmd_after_isearch_cmd(expand, text, arg);
        }

        CmdIdx::autocmd => return set_context_in_autocmd(expand, arg, false),
        CmdIdx::doautocmd | CmdIdx::doautoall => {
            return set_context_in_autocmd(expand, arg, true);
        }

        CmdIdx::set => set_context_in_set_cmd(expand, arg, OptionSetFlags::NONE),
        CmdIdx::setglobal => set_context_in_set_cmd(expand, arg, OptionSetFlags::GLOBAL),
        CmdIdx::setlocal => set_context_in_set_cmd(expand, arg, OptionSetFlags::LOCAL),

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
            let context = if wop_flags.get() & kOptWopFlagTagfile as c_uint != 0 {
                ExpandContext::TagsListFiles
            } else {
                ExpandContext::Tags
            };
            whole_argument(expand, context);
        }

        CmdIdx::augroup => whole_argument(expand, ExpandContext::Augroup),

        CmdIdx::syntax => set_context_in_syntax_cmd(expand, arg),

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
        | CmdIdx::lgetexpr => set_context_for_expression(expand, arg, cmdidx),

        CmdIdx::unlet => return set_context_in_unlet_cmd(expand, text, arg),

        CmdIdx::function | CmdIdx::delfunction => whole_argument(expand, ExpandContext::UserFunc),

        CmdIdx::echohl => set_context_in_echohl_cmd(expand, arg),
        CmdIdx::highlight => set_context_in_highlight_cmd(expand, arg),
        CmdIdx::sign => set_context_in_sign_cmd(expand, arg),

        CmdIdx::bdelete | CmdIdx::bwipeout | CmdIdx::bunload => {
            // Only the argument the cursor is in is completed.  Upstream
            // falls through into the buffer-name arm below.
            expand.context = ExpandContext::Buffers;
            expand.pattern = last_word(arg);
        }
        CmdIdx::buffer | CmdIdx::sbuffer | CmdIdx::pbuffer | CmdIdx::checktime => {
            whole_argument(expand, ExpandContext::Buffers);
        }

        CmdIdx::diffget | CmdIdx::diffput => {
            // If current buffer is in diff mode, complete buffer names
            // which are in diff mode, and different than current buffer.
            whole_argument(expand, ExpandContext::DiffBuffers);
        }

        CmdIdx::USER | CmdIdx::USER_BUF => {
            return set_context_in_user_cmdarg(expand, cmd_name, arg, argt, context, forceit);
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
            return set_context_in_map_cmd(expand, cmd_name, arg, forceit, false, false, cmdidx);
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
            return set_context_in_map_cmd(expand, cmd_name, arg, forceit, false, true, cmdidx);
        }
        CmdIdx::mapclear
        | CmdIdx::nmapclear
        | CmdIdx::vmapclear
        | CmdIdx::omapclear
        | CmdIdx::imapclear
        | CmdIdx::cmapclear
        | CmdIdx::lmapclear
        | CmdIdx::smapclear
        | CmdIdx::xmapclear => whole_argument(expand, ExpandContext::Mapclear),

        CmdIdx::abbreviate
        | CmdIdx::noreabbrev
        | CmdIdx::cabbrev
        | CmdIdx::cnoreabbrev
        | CmdIdx::iabbrev
        | CmdIdx::inoreabbrev => {
            return set_context_in_map_cmd(expand, cmd_name, arg, forceit, true, false, cmdidx);
        }
        CmdIdx::unabbreviate | CmdIdx::cunabbrev | CmdIdx::iunabbrev => {
            return set_context_in_map_cmd(expand, cmd_name, arg, forceit, true, true, cmdidx);
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
            return set_context_in_menu_cmd(expand, cmd_name, arg, forceit);
        }

        CmdIdx::colorscheme => whole_argument(expand, ExpandContext::Colors),
        CmdIdx::compiler => whole_argument(expand, ExpandContext::Compiler),
        CmdIdx::ownsyntax => whole_argument(expand, ExpandContext::Ownsyntax),
        CmdIdx::setfiletype => whole_argument(expand, ExpandContext::Filetype),
        CmdIdx::packadd => whole_argument(expand, ExpandContext::Packadd),

        CmdIdx::runtime => set_context_in_runtime_cmd(expand, arg),

        CmdIdx::language => return set_context_in_lang_cmd(expand, text, arg),

        CmdIdx::profile => set_context_in_profile_cmd(expand, arg),

        CmdIdx::checkhealth => expand.context = ExpandContext::Checkhealth,
        CmdIdx::lsp => expand.context = ExpandContext::Lsp,

        CmdIdx::retab => whole_argument(expand, ExpandContext::Retab),
        CmdIdx::messages => whole_argument(expand, ExpandContext::Messages),
        CmdIdx::history => whole_argument(expand, ExpandContext::History),
        CmdIdx::syntime => whole_argument(expand, ExpandContext::Syntime),

        CmdIdx::argdelete => {
            expand.context = ExpandContext::Arglist;
            expand.pattern = last_word(arg);
        }

        CmdIdx::breakadd | CmdIdx::profdel | CmdIdx::breakdel => {
            return set_context_in_breakadd_cmd(expand, text, arg, cmdidx);
        }

        CmdIdx::scriptnames => return set_context_in_scriptnames_cmd(expand, text, arg),

        CmdIdx::filetype => return set_context_in_filetype_cmd(expand, text, arg),

        CmdIdx::lua | CmdIdx::equal => expand.context = ExpandContext::Lua,

        _ => {}
    }
    None
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
/// The command starts at `buff` in `text`. Returns where the next command
/// starts, when there is one before the cursor.
pub(crate) fn set_one_cmd_context(expand: &mut Expand, text: &CStr, buff: usize) -> Option<usize> {
    let t = text.to_bytes();
    let mut ea = ExArg {
        cmdidx: CmdIdx::append,
        addr_type: CmdAddr::Lines,
        ..ExArg::default()
    };
    let mut context = ExpandContext::Nothing;
    let mut forceit = false;
    let mut usefilter = false; // Filter instead of file name.

    expand.reset_keeping_line();
    expand.context = ExpandContext::Commands; // Default until we get past command
    ea.argt = ExArgt::NONE;

    // 1. skip comment lines and leading space, colons or bars
    let mut cmd = buff;
    while is_one_of(c" \t:|", t, cmd) {
        cmd += 1;
    }
    expand.pattern = cmd;

    if byte(t, cmd) == 0 {
        return None;
    }
    if byte(t, cmd) == b'"' {
        // Ignore comment lines.
        expand.context = ExpandContext::Nothing;
        return None;
    }

    // 3. skip over a range specifier of the form: addr [,addr] [;addr] ..
    cmd += skip_range(&t[cmd..], Some(&mut expand.context));
    expand.pattern = cmd;
    if byte(t, cmd) == 0 {
        return None;
    }
    if byte(t, cmd) == b'"' {
        expand.context = ExpandContext::Nothing;
        return None;
    }

    if byte(t, cmd) == b'|' || byte(t, cmd) == b'\n' {
        return Some(cmd + 1); // There's another command
    }

    // Get the command index.
    let mut p = set_cmd_index(text, cmd, &mut ea, expand, &mut context)?;

    expand.context = ExpandContext::Nothing; // Default now that we're past command

    if byte(t, p) == b'!' {
        // Forced commands.
        forceit = true;
        p += 1;
    }

    // 6. parse arguments
    if !is_user_cmd(ea.cmdidx) {
        ea.argt = excmd_get_argt(ea.cmdidx);
    }

    let mut arg = skipwhite(t, p);

    // Does command allow "++argopt" argument?
    if ea.argt.has(ExArgt::ARGOPT) {
        while byte(t, arg) != 0 && t[arg..].starts_with(b"++") {
            p = arg + 2;
            while byte(t, p) != 0 && !ascii_isspace(c_int::from(byte(t, p))) {
                p += char_len(t, p);
            }

            // Still touching the command after "++"?
            if byte(t, p) == 0 && ea.argt.has(ExArgt::ARGOPT) {
                return set_context_in_argopt(expand, text, arg + 2);
            }

            arg = skipwhite(t, p);
        }
    }

    if ea.cmdidx == CmdIdx::write || ea.cmdidx == CmdIdx::update {
        if byte(t, arg) == b'>' {
            // Append.
            arg += 1;
            if byte(t, arg) == b'>' {
                arg += 1;
            }
            arg = skipwhite(t, arg);
        } else if byte(t, arg) == b'!' && ea.cmdidx == CmdIdx::write {
            // :w !filter
            arg += 1;
            usefilter = true;
        }
    }

    if ea.cmdidx == CmdIdx::read {
        usefilter = forceit; // :r! filter if forced
        if byte(t, arg) == b'!' {
            // :r !filter
            arg += 1;
            usefilter = true;
        }
    }

    if ea.cmdidx == CmdIdx::lshift || ea.cmdidx == CmdIdx::rshift {
        // Allow any number of '>' or '<'.
        while byte(t, arg) == byte(t, cmd) {
            arg += 1;
        }
        arg = skipwhite(t, arg);
    }

    // Does command allow "+command"?
    if ea.argt.has(ExArgt::CMDARG) && !usefilter && byte(t, arg) == b'+' {
        // Check if we're in the +command.
        p = arg + 1;
        arg = skip_cmd_arg_at(t, arg);

        // Still touching the command after '+'?
        if byte(t, arg) == 0 {
            return Some(p);
        }

        // Skip space(s) after +command to get to the real argument.
        arg = skipwhite(t, arg);
    }

    // Check for '|' to separate commands and '"' to start comments.
    // Don't do this for ":read !cmd" and ":write !cmd".
    if ea.argt.has(ExArgt::TRLBAR) && !usefilter {
        p = arg;
        // ":redir @" is not the start of a comment.
        if ea.cmdidx == CmdIdx::redir && byte(t, p) == b'@' && byte(t, p + 1) == b'"' {
            p += 2;
        }
        while byte(t, p) != 0 {
            if c_int::from(byte(t, p)) == Ctrl_V {
                if byte(t, p + 1) != 0 {
                    p += 1;
                }
            } else if ((byte(t, p) == b'"' && !ea.argt.has(ExArgt::NOTRLCOM))
                || byte(t, p) == b'|'
                || byte(t, p) == b'\n')
                && byte(t, p - 1) != b'\\'
            {
                if byte(t, p) == b'|' || byte(t, p) == b'\n' {
                    return Some(p + 1);
                }
                return None; // It's a comment
            }
            p += char_len(t, p);
        }
    }

    if !ea.argt.has(ExArgt::EXTRA) && byte(t, arg) != 0 && !is_one_of(c"|\"", t, arg) {
        // No arguments allowed but there is something.
        return None;
    }

    // Find start of last argument (argument just before cursor).
    p = buff;
    expand.pattern = p;
    while byte(t, p) != 0 {
        if byte(t, p) == b' ' || c_int::from(byte(t, p)) == TAB {
            // Argument starts after a space.
            p += 1;
            expand.pattern = p;
        } else {
            if byte(t, p) == b'\\' && byte(t, p + 1) != 0 {
                p += 1; // Skip over escaped character.
            }
            p += char_len(t, p);
        }
    }

    if ea.argt.has(ExArgt::XFILE) {
        set_context_for_wildcard_arg(Some(ea.cmdidx), text, arg, usefilter, expand, &mut context);
    }

    // Switch on command name.
    set_context_by_cmdname(text, cmd, ea.cmdidx, expand, arg, ea.argt, context, forceit)
}

/// Set the completion context in `expand` for the command line `line`, with
/// the cursor at `col`; `use_ccline` asks for the command line being edited
/// to be consulted for what kind of line it is.
///
/// `expand` keeps a copy of `line` as [`Expand::line`]: while the context is
/// worked out the copy stops at the cursor, as upstream's NUL poked into the
/// caller's string did, and the rest is put back once it is.
pub fn set_cmd_context(expand: &mut Expand, line: &[u8], col: c_int, use_ccline: bool) {
    let ccline = Cc::current();
    let cut = usize::try_from(col).unwrap_or(0).min(line.len());
    expand.line = owned(&line[..cut]);
    // The text the walk reads; the context's own copy is what the parsers
    // of other modules read, and the two are the same bytes.
    let snapshot = expand.line.clone();
    let text = snapshot.as_cstr();

    if use_ccline && ccline.cmdfirstc == c_int::from(b'=') {
        // Pass CmdIdx::SIZE because there is no real command.
        set_context_for_expression(expand, 0, CmdIdx::SIZE);
    } else if use_ccline && ccline.input_fn != 0 {
        expand.context = ccline.xp_context;
        expand.pattern = 0;
        expand.arg = ccline.xp_arg.clone();
        if expand.context == ExpandContext::ShellCmdLine {
            let mut context = expand.context;
            set_context_for_wildcard_arg(None, text, 0, false, expand, &mut context);
        }
    } else {
        let mut nextcomm = Some(0);
        while let Some(at) = nextcomm {
            nextcomm = set_one_cmd_context(expand, text, at);
        }
    }

    // Store the whole line so that call_user_expand_func() can get to it
    // easily.
    expand.line.push_bytes(&line[cut..]);
    expand.col = col;
}

/// What an expansion attempt came to.
///
/// Upstream answers through the same `int` as an `xp_context` and reuses
/// three of its names, one of which (`EXPAND_OK`) is not a context at all.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Expanded {
    /// The matches came back.
    Ok,
    /// Nothing to expand — the caller may insert the key that triggered the
    /// expansion literally.
    Nothing,
    /// Something illegal stands before the cursor; the editor has beeped.
    Unsuccessful,
}

/// Expand the line in `expand` with the cursor at `col`, the context having
/// been set by [`set_cmd_context`]. Answers what the attempt came to and,
/// for `Expanded::Ok`, the matches.
pub fn expand_cmdline(expand: &mut Expand, col: c_int) -> (Expanded, Vec<XString>) {
    let mut options = WildOpts::ADD_SLASH | WildOpts::SILENT;

    if expand.context == ExpandContext::Unsuccessful {
        beep_flush();
        return (Expanded::Unsuccessful, Vec::new()); // Something illegal on command line
    }
    if expand.context == ExpandContext::Nothing {
        // Caller can use the character as a normal char instead.
        return (Expanded::Nothing, Vec::new());
    }

    // Add star to file name, or convert to regexp if not expanding files.
    let col = usize::try_from(col).unwrap_or(0);
    debug_assert!(col >= expand.pattern);
    expand.pattern_len = col.saturating_sub(expand.pattern);
    let file_str = if cmdline_fuzzy_completion_supported(expand) {
        // If fuzzy matching, don't modify the search string.
        owned(expand.pattern_text())
    } else {
        addstar(expand.pattern_span(), expand.context)
    };

    if p_wic() {
        options |= WildOpts::ICASE;
    }

    // Find all files that match the description.
    let found = expand_from_context(expand, file_str.as_cstr(), options).unwrap_or_default();
    (Expanded::Ok, found)
}
