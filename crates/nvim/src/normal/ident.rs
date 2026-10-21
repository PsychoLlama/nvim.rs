//! The identifier under the cursor, and the commands that look it up:
//! tags, `:help`, 'keywordprg', a declaration, a file name.

#![forbid(unsafe_code)]

use super::identfind::find_ident_under_cursor;
use super::state::with_scratch_cmd;
use crate::ascii::ascii_iswhite;
use crate::cstr;
use crate::ex_cmds::EcmdFlags;
use crate::ex_cmds::newlnum;
use crate::option::vars::P_KP;
use crate::strings::has_char;
use crate::winlayer::{Buf, Win};

use crate::buffer::buf_hide;
use crate::charset::is_word_at;
use crate::cmdhist::{add_to_history, init_history};
use crate::cursor::check_cursor_lnum;
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_cmds::do_ecmd;
use crate::ex_cmds2::autowrite;
use crate::ex_docmd::do_cmdline_cmd;
use crate::ex_getln::fnameescape;
use crate::file_search::grab_file_name;
use crate::keycodes::Ctrl_RSB;
use crate::mapping::add_map;
use crate::mark::setpcmark;
use crate::mbyte::{cluster_len, head_off};
use crate::memline::Lines;
use crate::message::e_noident;
use crate::message::emsg;
use crate::normal::{
    DT_POP, FIND_IDENT, FIND_STRING, HIST_SEARCH, POUND, VSE_NONE, check_clear_op_quit,
    check_text_or_curbuf_locked, clear_op, get_visual_text, normal_search, visual_active,
};
use crate::option::magic_isset;
use crate::optionstr::{OptString, OptStringRef, local_or_global};
use crate::os::cshim::gettext;
use crate::search::state::no_smartcase;
use crate::state::MODE_TERMINAL;
use crate::state::mode::restart_edit;
use crate::strings::shellescape;
use crate::tag::do_tag;
use crate::tag::state::g_tag_at_cursor;
use crate::types::{CmdArg, ColNr, LineNr, NUL, int64_t};
use crate::undo::curbuf_is_changed;
use crate::window::check_can_set_curbuf_disabled;
use core::ffi::{CStr, c_int};

/// Run one of the identifier commands from outside the command loop, with a
/// command argument built for it. `CTRL-W ]` and friends use this.
pub(crate) fn do_nv_ident(c1: c_int, c2: c_int) {
    with_scratch_cmd(|ca| {
        ca.cmdchar = c1;
        ca.nchar = c2;
        nv_ident(ca);
    });
}

/// Add the command `K` should run to `out`. Answers the length of the
/// identifier still to be appended, or 0 when there is nothing to look up.
///
/// `text` is the identifier and the rest of its line, and `start` the offset
/// in it the identifier's `n` bytes begin at -- advanced past the leading
/// dashes a shell command would take for options.
#[allow(clippy::too_many_arguments)]
fn build_keywordprg_cmd(
    count0: c_int,
    kp: &[u8],
    kp_help: bool,
    kp_ex: bool,
    text: &[u8],
    start: &mut usize,
    mut n: usize,
    out: &mut Vec<u8>,
) -> usize {
    if kp_help {
        out.clear();
        out.extend_from_slice(b"help! ");
        return n;
    }
    if kp_ex {
        // An Ex 'keywordprg' takes the word as its argument, after an
        // optional count.
        out.extend_from_slice(kp);
        out.push(b' ');
        if count0 != 0 {
            out.extend_from_slice(format!("{count0} ").as_bytes());
        }
        return n;
    }

    // A shell 'keywordprg' runs in a terminal in a new tab. Leading
    // dashes would look like options to it.
    while cstr::byte_at(text, *start) == b'-' && n > 0 {
        *start += 1;
        n -= 1;
    }
    if n == 0 {
        emsg(gettext(e_noident));
        return 0;
    }
    // `man` and `man -s` take the count as a section number, which goes
    // in front of the word rather than becoming a line range.
    let isman = kp == b"man";
    let isman_s = kp == b"man -s";
    if count0 != 0 && !(isman || isman_s) {
        out.extend_from_slice(format!(".,.+{}", int64_t::from(count0) - 1).as_bytes());
    }
    let _ = do_cmdline_cmd(c"tabnew");
    out.extend_from_slice(b"terminal ");
    if count0 == 0 && isman_s {
        // `man -s` with no section is just `man`.
        out.extend_from_slice(b"man ");
    } else {
        out.extend_from_slice(kp);
        out.push(b' ');
    }
    if count0 != 0 && (isman || isman_s) {
        out.extend_from_slice(format!("{count0} ").as_bytes());
    }
    n
}

/// The characters that have to be backslash-escaped for the command being
/// built, given what it is.
fn ident_escapes(cmdchar: c_int, tag_cmd: bool) -> &'static CStr {
    if cmdchar == '*' as c_int {
        return if magic_isset() {
            c"/.*~[^$\\"
        } else {
            c"/^$\\"
        };
    }
    if cmdchar == '#' as c_int {
        return if magic_isset() {
            c"/?.*~[^$\\"
        } else {
            c"/?^$\\"
        };
    }
    if !tag_cmd {
        return c"\\|\"\n*?[";
    }
    // A help tag may contain any of these, so nothing is escaped.
    if Buf::current().b_p_ft.bytes() == b"help" {
        c""
    } else {
        c"\\|\"\n["
    }
}

/// Append `n` bytes of `text` from `start` on to `out`, escaping whatever
/// the command cannot take literally.
///
/// A character's bytes go together: the escape is decided on its first. At
/// the end of the line (`text` runs out) the bytes read as NUL and the copy
/// runs on until `n` is spent -- upstream's `utfc_ptr2len` answering 0 there,
/// which is what stops a NUL ending the copy early.
fn append_escaped(out: &mut Vec<u8>, text: &[u8], start: usize, mut n: usize, escapes: &CStr) {
    let mut at = start;
    while n > 0 {
        n -= 1;
        let c = cstr::byte_at(text, at);
        if has_char(escapes, c_int::from(c)) {
            out.push(b'\\');
        }
        let trailing = cluster_len(text.get(at..).unwrap_or_default()).wrapping_sub(1);
        let mut i = 0;
        while i < trailing && n > 0 {
            out.push(cstr::byte_at(text, at));
            at += 1;
            i += 1;
            n -= 1;
        }
        out.push(cstr::byte_at(text, at));
        at += 1;
    }
}

/// `*`, `#`, `K`, `]`, `CTRL-]` and their `g` forms: look up the identifier
/// under the cursor.
pub(crate) fn nv_ident(cmd_arg: &mut CmdArg) {
    let (typed, nchar) = (cmd_arg.cmdchar, cmd_arg.nchar);
    // The `g` forms carry the real command in `nchar`.
    let g_cmd = typed == 'g' as c_int;
    let mut cmdchar = if g_cmd { nchar } else { typed };
    if cmdchar == POUND {
        cmdchar = '#' as c_int;
    }

    // The identifier and the rest of its line, copied: three of the commands
    // take a Visual selection instead of the word under the cursor, and the
    // others the word under the cursor, which also says what column of the
    // cursor's line it starts at.
    let mut text: Option<Vec<u8>> = None;
    let mut n: usize = 0;
    let mut word_col: ColNr = 0;
    let mut visual_sel = false;
    if cmdchar == ']' as c_int || cmdchar == Ctrl_RSB || cmdchar == 'K' as c_int {
        if visual_active() {
            let Some(selection) = get_visual_text(Some(cmd_arg)) else {
                return;
            };
            n = selection.len();
            text = Some(selection.to_end_of_line().to_bytes().to_vec());
            visual_sel = true;
        }
        if check_clear_op_quit(cmd_arg.op()) {
            return;
        }
    }
    let text = match text {
        Some(text) => text,
        None => {
            // `*` and `#` fall back to the string under the cursor when
            // there is no identifier there.
            let searchy = cmdchar == '*' as c_int || cmdchar == '#' as c_int;
            let find_type = FIND_IDENT as c_int | if searchy { FIND_STRING as c_int } else { 0 };
            let Some((col, len)) = find_ident_under_cursor(find_type) else {
                clear_op(cmd_arg.op());
                return;
            };
            (word_col, n) = (col, len);
            let lnum = Win::current().w_cursor.lnum;
            let from = usize::try_from(col).unwrap_or(0);
            Lines::current().line(lnum)[from..].to_vec()
        }
    };
    let mut start = 0;

    // 'keywordprg', which decides what `K` does. A copy, because it is
    // built into a command line further down.
    let keywordprg = local_or_global(&Buf::current().b_p_kp, P_KP).get();
    let kp: &[u8] = &keywordprg;
    let kp_helpbang = keywordprg == c":help!";
    let kp_help =
        kp_helpbang || keywordprg.is_empty() || keywordprg == c":he" || keywordprg == c":help";
    if kp_help && !kp_helpbang && text.iter().all(|&b| ascii_iswhite(c_int::from(b))) {
        emsg(gettext(e_noident));
        return;
    }
    let kp_ex = cstr::byte_at(kp, 0) == b':';

    // Room for the command, the word with every byte escaped, and more.
    let mut out: Vec<u8> = Vec::with_capacity(n.wrapping_mul(2).wrapping_add(30) + kp.len());

    // Whether the command being built is a tag lookup, which decides the
    // escaping below.
    let mut tag_cmd = false;
    match u8::try_from(cmdchar) {
        Ok(b'*' | b'#') => {
            // These become a search, so the cursor moves to the start of
            // the word first.
            setpcmark();
            Win::current().w_cursor.col = word_col;
            if !g_cmd && is_word_at(&text, Buf::current()) {
                // The plain forms anchor at a word boundary.
                out.extend_from_slice(b"\\<");
            }
            no_smartcase.set(true);
        }
        Ok(b'K') => {
            let count0 = cmd_arg.count0;
            n = build_keywordprg_cmd(count0, kp, kp_help, kp_ex, &text, &mut start, n, &mut out);
            if n == 0 {
                return;
            }
        }
        Ok(b']') => {
            tag_cmd = true;
            out.clear();
            out.extend_from_slice(b"tselect ");
        }
        // CTRL-] and everything else: a plain tag jump.
        _ => {
            tag_cmd = true;
            let count0 = cmd_arg.count0;
            let cmd: &[u8] = if Buf::current().b_help {
                b"help! "
            } else if g_cmd {
                b"tjump "
            } else if count0 == 0 {
                b"tag "
            } else {
                // A count picks which of the matching tags to jump to.
                out.extend_from_slice(format!(":{count0}tag ").as_bytes());
                b""
            };
            if !cmd.is_empty() {
                out.clear();
                out.extend_from_slice(cmd);
            }
        }
    }

    if cmdchar == 'K' as c_int && kp_helpbang && !visual_sel {
        // `:help!` with no selection opens the help index rather than
        // looking anything up.
        out.clear();
        out.extend_from_slice(b"help!");
    } else if cmdchar == 'K' as c_int && !kp_help {
        // A shell or Ex command takes the word quoted, not escaped
        // character by character.
        let word = cstr::until_nul(text.get(start..).unwrap_or_default());
        let word = &word[..n.min(word.len())];
        let quoted = if kp_ex {
            fnameescape(&cstr::owned(word), VSE_NONE as c_int)
        } else {
            shellescape(word, true, true)
        };
        out.extend_from_slice(&quoted);
    } else {
        append_escaped(&mut out, &text, start, n, ident_escapes(cmdchar, tag_cmd));
    }

    if cmdchar == '*' as c_int || cmdchar == '#' as c_int {
        if !g_cmd && word_end_is_word(word_col, n) {
            out.extend_from_slice(b"\\>");
        }
        // The search goes into the history as if it had been typed.
        init_history();
        add_to_history(HIST_SEARCH as c_int, &out, true, NUL as u8);
        let dir = if cmdchar == '*' as c_int { '/' } else { '?' } as c_int;
        normal_search(cmd_arg, dir, &out, 0, None);
    } else {
        // `taglist()` and friends need to know the tag came from under
        // the cursor rather than from a command line.
        g_tag_at_cursor.set(true);
        let _ = do_cmdline_cmd(&cstr::owned(&out));
        g_tag_at_cursor.set(false);
        if cmdchar == 'K' as c_int && !kp_ex && !kp_help {
            // The terminal 'keywordprg' opened above: let <Esc> close it.
            restart_edit.set('i' as c_int);
            add_map(b"<esc>", b"<Cmd>bdelete!<CR>", MODE_TERMINAL, true);
        }
    }
}

/// Whether the character before byte `word_col + n` of the cursor's line --
/// the last of the `n`-byte word at `word_col` -- is a word character, which
/// is what decides whether `*` and `#` anchor the end of their pattern.
fn word_end_is_word(word_col: ColNr, n: usize) -> bool {
    let lnum = Win::current().w_cursor.lnum;
    let mut lines = Lines::current();
    let line = lines.line(lnum);
    let end = usize::try_from(word_col).unwrap_or(0) + n;
    let prev = if end == 0 {
        0
    } else {
        end - 1 - head_off(line, (end - 1).min(line.len()))
    };
    is_word_at(line.get(prev..).unwrap_or_default(), Buf::current())
}

/// `CTRL-T`: back up the tag stack.
pub(crate) fn nv_tagpop(cmd_arg: &mut CmdArg) {
    if check_clear_op_quit(cmd_arg.op()) {
        return;
    }
    do_tag(c"", DT_POP as c_int, cmd_arg.count1, 0, true);
}

/// `gf`, `gF` and `[f`: edit the file named under the cursor.
pub(crate) fn nv_gotofile(cmd_arg: &mut CmdArg) {
    // SAFETY: `oap` is the live operator this command is pending on.
    if check_text_or_curbuf_locked(Some(cmd_arg.op())) || !check_can_set_curbuf_disabled() {
        return;
    }
    // `gF` also takes a line number off the end of the name.
    let mut lnum: LineNr = -1;
    let Some(name) = grab_file_name(cmd_arg.count1, Some(&mut lnum)) else {
        clear_op(cmd_arg.op());
        return;
    };
    // Leaving the only window on a changed buffer that cannot be hidden
    // means writing it first.
    let must_write =
        curbuf_is_changed() && Buf::current().b_nwindows <= 1 && !buf_hide(Buf::current());
    if must_write {
        let _ = autowrite(Buf::current(), false);
    }
    setpcmark();
    let hidden = buf_hide(Buf::current());
    let hide = EcmdFlags::HIDE.when(hidden);
    let last = newlnum::LAST as LineNr;
    let win = Some(Win::current().id());
    let opened = do_ecmd(0, Some(name.as_cstr()), None, None, last, hide, win);
    if opened.is_ok() && cmd_arg.nchar == 'F' as c_int && lnum >= 0 {
        Win::current().w_cursor.lnum = lnum;
        check_cursor_lnum(Win::current());
        beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    }
}
