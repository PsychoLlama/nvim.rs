//! The per-command context switch.
//!
//! [`set_context_by_cmdname`] is C's `set_context_by_cmdname`: one arm per
//! `CMD_*` whose argument has its own completion.  [`set_one_cmd_context`]
//! walks a single command's arguments to find the one the cursor is in, and
//! [`set_cmd_context`] / [`expand_cmdline`] are the two entry points over
//! both.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::ascii::CTRL_V;
use crate::ascii::{TAB, ascii_isspace};
use crate::autocmd::set_context_in_autocmd;
use crate::charset::skip;
use crate::cmdexpand::defs::ExpandContext;
use crate::cstr;
use crate::eval::set_context_for_expression;
use crate::ex_docmd::{excmd_get_argt, is_user_cmd, skip_cmd_arg_at, skip_range};
use crate::getchar::beep_flush;
use crate::highlight_group::set_context_in_highlight_cmd;
use crate::mapping::set_context_in_map_cmd;
use crate::menu::set_context_in_menu_cmd;
use crate::option::defs::OptionSetFlags;
use crate::option::vars::{WOP_FLAGS, p_wic};
use crate::option::{get_findfunc, set_context_in_set_cmd};
use crate::options::OPT_WOP_FLAG_TAGFILE;
use crate::profile::set_context_in_profile_cmd;
use crate::runtime::set_context_in_runtime_cmd;
use crate::sign::set_context_in_sign_cmd;
use crate::syntax::{set_context_in_echohl_cmd, set_context_in_syntax_cmd};
use crate::types::{CmdAddr, CmdIdx, ExArg, ExArgt};
use crate::usercmd::{set_context_in_user_cmd, set_context_in_user_cmdarg};
use crate::winlayer::Cc;
use core::ffi::{CStr, c_int, c_uint};

/// Set the completion context in `expand` for command `cmd` with index `cmdidx`.
///
/// The argument to the command starts at `arg` and the argument flags are
/// `argt`. For user-defined commands and for environment variables, `context`
/// carries the completion type.
///
/// The commands fall into families first -- a command argument, one context
/// for the whole argument, a mapping, an expression, a menu -- and the
/// `match` is what is left: the commands whose argument has a parser of its
/// own.
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
    if takes_command(cmdidx) {
        return Some(arg);
    }
    if let Some(context) = whole_argument_context(cmdidx) {
        expand.context = context;
        expand.pattern = arg;
        return None;
    }
    let cmd_name = cstr::suffix(text, cmd);
    if let Some(map) = MapCommand::of(cmdidx) {
        return set_context_in_map_cmd(
            expand, cmd_name, arg, forceit, map.abbrev, map.unmap, cmdidx,
        );
    }
    if takes_expression(cmdidx) {
        set_context_for_expression(expand, arg, cmdidx);
        return None;
    }
    if is_menu_command(cmdidx) {
        return set_context_in_menu_cmd(expand, cmd_name, arg, forceit);
    }
    let t = text.to_bytes();
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

        CmdIdx::filter => return set_context_in_filter_cmd(expand, text, arg),

        CmdIdx::r#match => return set_context_in_match_cmd(expand, text, arg),

        // All completion for the +cmdline_compl feature goes here.
        CmdIdx::command => return set_context_in_user_cmd(expand, arg),

        CmdIdx::global | CmdIdx::vglobal => {
            let nextcmd = find_cmd_after_global_cmd(text, arg);
            if nextcmd.is_none() && MAY_EXPAND_PATTERN.get() {
                set_context_with_pattern(expand);
            }
            return nextcmd;
        }

        CmdIdx::and | CmdIdx::substitute => {
            let nextcmd = find_cmd_after_substitute_cmd(text, arg);
            if nextcmd.is_none() && MAY_EXPAND_PATTERN.get() {
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
            expand.context = if WOP_FLAGS.get() & OPT_WOP_FLAG_TAGFILE as c_uint != 0 {
                ExpandContext::TagsListFiles
            } else {
                ExpandContext::Tags
            };
            expand.pattern = arg;
        }

        CmdIdx::syntax => set_context_in_syntax_cmd(expand, arg),

        CmdIdx::unlet => return set_context_in_unlet_cmd(expand, text, arg),

        CmdIdx::echohl => set_context_in_echohl_cmd(expand, arg),
        CmdIdx::highlight => set_context_in_highlight_cmd(expand, arg),
        CmdIdx::sign => set_context_in_sign_cmd(expand, arg),

        CmdIdx::bdelete | CmdIdx::bwipeout | CmdIdx::bunload => {
            // Only the argument the cursor is in is completed.  Upstream
            // falls through into the buffer-name arm.
            expand.context = ExpandContext::Buffers;
            expand.pattern = last_word(arg);
        }

        CmdIdx::USER | CmdIdx::USER_BUF => {
            return set_context_in_user_cmdarg(expand, cmd_name, arg, argt, context, forceit);
        }

        CmdIdx::runtime => set_context_in_runtime_cmd(expand, arg),

        CmdIdx::language => return set_context_in_lang_cmd(expand, text, arg),

        CmdIdx::profile => set_context_in_profile_cmd(expand, arg),

        CmdIdx::checkhealth => expand.context = ExpandContext::Checkhealth,
        CmdIdx::lsp => expand.context = ExpandContext::Lsp,

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

/// Whether `cmdidx` is a command modifier or a command whose argument is
/// itself a command (`:vertical`, `:argdo`): completion goes on in the
/// argument as a command line of its own.
fn takes_command(cmdidx: CmdIdx) -> bool {
    matches!(
        cmdidx,
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
            | CmdIdx::windo
    )
}

/// The one context `cmdidx`'s whole argument completes in, for the commands
/// whose argument is a single name of one kind (`:help`, `:colorscheme`).
fn whole_argument_context(cmdidx: CmdIdx) -> Option<ExpandContext> {
    Some(match cmdidx {
        CmdIdx::help => ExpandContext::Help,
        CmdIdx::delcommand => ExpandContext::UserCommands,
        CmdIdx::augroup => ExpandContext::Augroup,
        CmdIdx::function | CmdIdx::delfunction => ExpandContext::UserFunc,
        CmdIdx::buffer | CmdIdx::sbuffer | CmdIdx::pbuffer | CmdIdx::checktime => {
            ExpandContext::Buffers
        }
        // If the current buffer is in diff mode, the buffer names that are
        // in diff mode and are not the current buffer.
        CmdIdx::diffget | CmdIdx::diffput => ExpandContext::DiffBuffers,
        CmdIdx::mapclear
        | CmdIdx::nmapclear
        | CmdIdx::vmapclear
        | CmdIdx::omapclear
        | CmdIdx::imapclear
        | CmdIdx::cmapclear
        | CmdIdx::lmapclear
        | CmdIdx::smapclear
        | CmdIdx::xmapclear => ExpandContext::Mapclear,
        CmdIdx::colorscheme => ExpandContext::Colors,
        CmdIdx::compiler => ExpandContext::Compiler,
        CmdIdx::ownsyntax => ExpandContext::Ownsyntax,
        CmdIdx::setfiletype => ExpandContext::Filetype,
        CmdIdx::packadd => ExpandContext::Packadd,
        CmdIdx::retab => ExpandContext::Retab,
        CmdIdx::messages => ExpandContext::Messages,
        CmdIdx::history => ExpandContext::History,
        CmdIdx::syntime => ExpandContext::Syntime,
        _ => return None,
    })
}

/// A `:map`/`:abbreviate` family command, as `set_context_in_map_cmd` reads
/// it.
#[derive(Debug, PartialEq, Eq)]
struct MapCommand {
    /// An abbreviation command rather than a mapping one.
    abbrev: bool,
    /// A command that removes (`:unmap`, `:unabbreviate`).
    unmap: bool,
}

impl MapCommand {
    /// The mapping command `cmdidx` is, if it is one; the `:mapclear`s are
    /// not, their argument is [`whole_argument_context`]'s.
    fn of(cmdidx: CmdIdx) -> Option<MapCommand> {
        let (abbrev, unmap) = match cmdidx {
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
            | CmdIdx::xnoremap => (false, false),
            CmdIdx::unmap
            | CmdIdx::nunmap
            | CmdIdx::vunmap
            | CmdIdx::ounmap
            | CmdIdx::iunmap
            | CmdIdx::cunmap
            | CmdIdx::lunmap
            | CmdIdx::sunmap
            | CmdIdx::xunmap => (false, true),
            CmdIdx::abbreviate
            | CmdIdx::noreabbrev
            | CmdIdx::cabbrev
            | CmdIdx::cnoreabbrev
            | CmdIdx::iabbrev
            | CmdIdx::inoreabbrev => (true, false),
            CmdIdx::unabbreviate | CmdIdx::cunabbrev | CmdIdx::iunabbrev => (true, true),
            _ => return None,
        };
        Some(MapCommand { abbrev, unmap })
    }
}

/// Whether `cmdidx`'s argument is a Vim expression (`:let`, `:echo`,
/// `:cexpr`).
fn takes_expression(cmdidx: CmdIdx) -> bool {
    matches!(
        cmdidx,
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
            | CmdIdx::lgetexpr
    )
}

/// Whether `cmdidx` defines, removes or runs a menu entry.
fn is_menu_command(cmdidx: CmdIdx) -> bool {
    matches!(
        cmdidx,
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
            | CmdIdx::emenu
    )
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
    expand.has_pattern = true;
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

    let mut arg = skip::white_from(t, p);

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

            arg = skip::white_from(t, p);
        }
    }

    if ea.cmdidx == CmdIdx::write || ea.cmdidx == CmdIdx::update {
        if byte(t, arg) == b'>' {
            // Append.
            arg += 1;
            if byte(t, arg) == b'>' {
                arg += 1;
            }
            arg = skip::white_from(t, arg);
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
        arg = skip::white_from(t, arg);
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
        arg = skip::white_from(t, arg);
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
            if byte(t, p) == CTRL_V {
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
        if byte(t, p) == b' ' || byte(t, p) == TAB {
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

    if use_ccline && ccline.cmdfirstc == b'=' {
        // Pass CmdIdx::SIZE because there is no real command.
        set_context_for_expression(expand, 0, CmdIdx::SIZE);
        expand.has_pattern = true;
    } else if use_ccline && ccline.input_fn {
        expand.context = ccline.xp_context;
        expand.pattern = 0;
        expand.has_pattern = true;
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
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The families were one `match`, where a command named twice is an
    /// unreachable pattern; as separate tests nothing else says so.
    #[test]
    fn a_command_is_in_at_most_one_family() {
        let builtins = usize::try_from(CmdIdx::SIZE.code()).expect("a count");
        for row in 0..builtins {
            let cmdidx = CmdIdx::at_row(row);
            let families = [
                takes_command(cmdidx),
                whole_argument_context(cmdidx).is_some(),
                MapCommand::of(cmdidx).is_some(),
                takes_expression(cmdidx),
                is_menu_command(cmdidx),
            ];
            let count = families.iter().filter(|&&f| f).count();
            assert!(count <= 1, "{cmdidx:?} is in {count} families");
        }
    }

    #[test]
    fn mapping_commands_say_what_they_do() {
        let map = |abbrev, unmap| Some(MapCommand { abbrev, unmap });
        assert_eq!(MapCommand::of(CmdIdx::nnoremap), map(false, false));
        assert_eq!(MapCommand::of(CmdIdx::xunmap), map(false, true));
        assert_eq!(MapCommand::of(CmdIdx::inoreabbrev), map(true, false));
        assert_eq!(MapCommand::of(CmdIdx::cunabbrev), map(true, true));
        assert_eq!(MapCommand::of(CmdIdx::nmapclear), None);
        assert_eq!(
            whole_argument_context(CmdIdx::nmapclear),
            Some(ExpandContext::Mapclear)
        );
    }
}
