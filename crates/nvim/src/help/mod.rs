//! The help system: `:help`, the help window, and `:helptags`.
//!
//! Three things live here.
//!
//! - **Looking a subject up.** [`find_help_tags`] hands the subject to
//!   `vim._core.help.escape_subject()`, which turns what the user typed into
//!   a tag search pattern, then runs it through the ordinary tag machinery
//!   with `TAG_HELP`. `find_tags` appends a [`help_heuristic`] score after
//!   each match's NUL, which is what [`help_compare`] sorts on, so the first
//!   match is the best one.
//! - **The help window.** [`ex_help`] re-uses an existing one, or splits a
//!   new one and `do_ecmd`s into it; [`prepare_help_buffer`] then forces the
//!   options a help buffer needs, because an autocommand may have changed
//!   them.
//! - **Generating tags files**, which is [`tags`]'s: `:helptags` walks a
//!   `doc` directory and writes the tags file a lookup then searches.
//!
//! A subject, its matches and the tag jumped to are owned values: the
//! search pattern is built by Lua, cut to upstream's `IObuff` room, and
//! handed to `find_tags`, whose matches come back as `(tag, score)` pairs.
//!
//! Original: `src/nvim/help.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
// The constants here keep upstream's spelling.
#![allow(non_upper_case_globals)]

mod tags;

use crate::ascii::{ascii_isalpha, ascii_iswhite};
use crate::buffer::{buf_is_help, find_buf, set_buflisted, wipe_buffer};
use crate::charset::buf_init_chartab;
use crate::ex_cmds::EcmdFlags;
use crate::ex_cmds::do_ecmd;
use crate::ex_cmds::newlnum;
use crate::ex_docmd::state::cmdmod;
use crate::ex_docmd::{cmdmod_has, do_cmdline_cmd};
use crate::getchar::state::KeyTyped;
use crate::highlight_group::HLF_E;
use crate::lua::executor::nlua_exec_cstr;
use crate::memory::XString;
use crate::message::e_noident;
use crate::message::{emsg, emsg_multiline};
use crate::message_fmt::{msg_bytes, msg_cstr};
use crate::option::set_option_direct;
use crate::option::vars::{p_hf, p_hh, p_hlg, p_sb};
use crate::options::{kOptBuftype, kOptFoldmethod, kOptIskeyword};
use crate::optionstr::OptStringRef;
use crate::optionstr::check_buf_options;
use crate::os::cshim::gettext;
use crate::pos::MAXCOL;
use crate::semsg;
use crate::smsg;
use crate::state::mode::restart_edit;
use crate::tag::{HelpTagMatch, do_tag, find_help_tag_matches};
use crate::types::String_0;
use crate::types::{
    Array, ArrayBuf, CmdModFlags, ExArg, Failed, FileComparison, IOSIZE, LuaRetMode, Object,
    OptInt, OptVal, OptionSetFlags,
};
use crate::ui::state::Columns;
use crate::window::{WSP_BOT, WSP_HELP, WSP_TOP, win_close, win_enter, win_setheight, win_split};
use crate::winlayer::Buf;
use crate::winlayer::Win;
use crate::winlayer::windows;
use core::cmp::Ordering;
use core::ffi::{CStr, c_int};
use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;

pub(crate) use tags::ex_helptags;

/// Constants the transpiler copied in from the headers this module includes.
mod flag {
    use super::{FileComparison, LuaRetMode};
    use core::ffi::c_uint;

    /// `find_tags` flags.
    pub(super) const TAG_HELP: c_uint = 1;
    pub(super) const TAG_NAMES: c_uint = 2;
    pub(super) const TAG_REGEXP: c_uint = 4;
    pub(super) const TAG_VERBOSE: c_uint = 32;
    pub(super) const TAG_KEEP_LANG: c_uint = 128;
    pub(super) const TAG_NO_TAGFUNC: c_uint = 256;
    /// The longest match list `:help` will show.
    pub(super) const TAG_MANY: c_uint = 300;
    /// `do_tag`'s "this is a help tag" kind.
    pub(super) const DT_HELP: c_uint = 8;

    /// `path_full_compare`'s "same file" answer.
    pub(super) const kEqualFiles: FileComparison = 1;
    /// `nlua_exec` return modes.
    pub(super) const kRetObject: LuaRetMode = 0;
    pub(super) const kRetNilBool: LuaRetMode = 1;
}

use flag::{
    DT_HELP, TAG_HELP, TAG_KEEP_LANG, TAG_MANY, TAG_NAMES, TAG_NO_TAGFUNC, TAG_REGEXP, TAG_VERBOSE,
    kRetNilBool, kRetObject,
};

/// Whether the `:keepalt` modifier is off, so that the alternate file may
/// be changed.
fn keepalt_is_off() -> bool {
    !cmdmod_has(CmdModFlags::KEEPALT)
}

/// A borrowed string option value; `set_option_direct` copies what it keeps.
const fn cstr_optval(value: &'static CStr) -> OptVal {
    OptVal::static_string(value)
}

// -- `:help` ---------------------------------------------------------------

/// `:help`, and `:help!` — which resolves the best tag under the cursor
/// instead of taking a subject.
///
pub(crate) fn ex_help(excmd: &mut ExArg) {
    open_help(Some(excmd));
}

/// `:help` with the subject the command gave, or `F1`'s "no subject at all".
///
/// The command's `arg` is truncated at the first `\n`, `\r`, or `|` that
/// starts a following command.
pub(crate) fn open_help(excmd: Option<&mut ExArg>) {
    let old_key_typed = KeyTyped.get();

    let forceit = excmd.as_deref().is_some_and(|command| command.forceit);
    // The subject, owned: a `@xx` is split off it below.
    let mut subject = match excmd {
        None => Vec::new(),
        Some(command) => {
            split_off_next_cmd(command);
            if command.skip {
                return;
            }
            trimmed_subject(command.line.arg())
        }
    };
    let lang = split_help_lang(&mut subject);

    let helpbang = forceit && subject.is_empty();
    if subject.is_empty() && !helpbang {
        subject = b"help.txt".to_vec();
    }

    // `:help!` asks Lua for the best tag at the cursor.
    if helpbang {
        let Some(tag) = resolve_tag_at_cursor() else {
            emsg(gettext(e_noident));
            return;
        };
        subject = tag;
    }

    let found = find_help_tags(&subject, forceit);
    // The first match in the requested language is the best match.
    let best = found.as_ref().ok().and_then(|matches| match lang {
        None => matches.first(),
        Some(lang) => matches.iter().find(|tag| {
            let len = tag.len();
            len > 3 && tag[len - 3] == b'@' && tag[len - 2..].eq_ignore_ascii_case(&lang)
        }),
    });
    let Some(tag) = best.map(|tag| XString::from_bytes(tag)) else {
        let arg = msg_bytes(&subject);
        match lang {
            None => semsg!("E149: No help for {arg}"),
            Some(lang) => {
                let lang = msg_bytes(&lang);
                semsg!("E661: No '{lang}' help for {arg}")
            }
        };
        return;
    };
    drop(found);

    if let Some(opened) = enter_help_window() {
        restart_edit.set(0);
        // Restore KeyTyped: setting 'filetype=help' may have reset it,
        // and `do_tag` needs it to open folds under the cursor.
        KeyTyped.set(old_key_typed);

        do_tag(tag.as_cstr(), DT_HELP as c_int, 1, 0, true);

        // Delete the empty buffer if we are not using it. Careful:
        // autocommands may have jumped to another window, so check that
        // the buffer is not in one.
        if opened.empty_fnum != 0
            && Buf::current().handle != opened.empty_fnum
            && let Some(buf) = find_buf(opened.empty_fnum).filter(|b| b.b_nwindows == 0)
        {
            wipe_buffer(buf, true);
        }
        // Keep the previous alternate file.
        if opened.alt_fnum != 0
            && Win::current().w_alt_fnum == opened.empty_fnum
            && keepalt_is_off()
        {
            Win::current().w_alt_fnum = opened.alt_fnum;
        }
    }
}

/// A `:help` command ends at the first LF, or at a `|` followed by some
/// text. Terminate the argument there and point `nextcmd` at the rest.
fn split_off_next_cmd(excmd: &mut ExArg) {
    let mut at = excmd.line.arg;
    while excmd.line.byte_at(at) != 0 {
        let (c, next) = (excmd.line.byte_at(at), excmd.line.byte_at(at + 1));
        if c == b'\n' || c == b'\r' || (c == b'|' && next != 0 && next != b'|') {
            excmd.line.terminate_at(at);
            excmd.line.next = Some(at + 1);
            return;
        }
        at += 1;
    }
}

/// `arg` without its trailing whitespace, except where a backslash escapes
/// it.
fn trimmed_subject(arg: &[u8]) -> Vec<u8> {
    let mut end = arg.len();
    // The first byte is never dropped, as upstream's walk stops at it.
    while end > 1 && ascii_iswhite(c_int::from(arg[end - 1])) && arg[end - 2] != b'\\' {
        end -= 1;
    }
    arg[..end].to_vec()
}

/// The tag `vim._core.help.resolve_tag()` picks for the cursor position, as
/// owned bytes, or `None` when it declines to pick one.
fn resolve_tag_at_cursor() -> Option<Vec<u8>> {
    let chunk = c"return require'vim._core.help'.resolve_tag()";
    let res = nlua_exec_cstr(chunk, Array::EMPTY, kRetObject).ok()?;
    let tag = res.as_string().filter(|tag| !tag.is_empty())?;
    // Upstream copied the answer as a C string: up to a NUL.
    Some(
        tag.as_bytes()
            .split(|&b| b == 0)
            .next()
            .unwrap_or_default()
            .to_vec(),
    )
}

/// The buffer numbers [`ex_help`] must undo if the jump lands elsewhere.
struct HelpWindow {
    /// The empty buffer a freshly split help window holds, to be wiped if
    /// the tag jump does not land in it.
    empty_fnum: c_int,
    /// The buffer that was current before the split, to be kept as the
    /// alternate file.
    alt_fnum: c_int,
}

/// Make a help window current, splitting one off if there is none. `None`
/// means the split failed or the 'helpfile' could not be opened, in which
/// case a message has already been shown.
fn enter_help_window() -> Option<HelpWindow> {
    let mut opened = HelpWindow {
        empty_fnum: 0,
        alt_fnum: 0,
    };
    // Re-use an existing help window; always open a new one for
    // `:tab help`.
    let same_tab = cmdmod.with(|m| m.cmod_tab) == 0;
    if buf_is_help(Win::current().buffer_or_none()) && same_tab {
        return Some(opened);
    }
    let existing = same_tab
        .then(|| {
            windows().find(|wp| {
                buf_is_help(wp.buffer_or_none()) && !wp.w_config.hide && wp.w_config.focusable
            })
        })
        .flatten();
    if let Some(wp) = existing.filter(|wp| wp.buffer().b_nwindows > 0) {
        win_enter(wp, true);
        return Some(opened);
    }

    // No help window yet: check that 'helpfile' can be read at all.
    let readable = p_hf(|value| File::open(OsStr::from_bytes(value.to_bytes())).is_ok());
    if !readable {
        let helpfile = p_hf(CStr::to_owned);
        let arg0 = msg_cstr(&helpfile);
        smsg!(0, "Help file \"{arg0}\" not found");
        return None;
    }

    // Split off a help window; put it at the far top when no position
    // was given and the current window is vertically split and narrow.
    let mut split = WSP_HELP as c_int;
    if cmdmod.with(|m| m.cmod_split) == 0
        && Win::current().w_width != Columns.get()
        && Win::current().w_width < 80
    {
        split |= if p_sb() {
            WSP_BOT as c_int
        } else {
            WSP_TOP as c_int
        };
    }
    if win_split(0, split).is_err() {
        return None;
    }
    if (Win::current().w_height as OptInt) < p_hh() {
        win_setheight(p_hh() as c_int);
    }

    // Open the help file. `do_ecmd` sets `b_help` and `readfile` sets
    // 'readonly'. The buffer is still open, so don't store info.
    opened.alt_fnum = Buf::current().handle;
    let (lnum, flags) = (newlnum::LASTL, EcmdFlags::HIDE | EcmdFlags::SET_HELP);
    let _ = do_ecmd(0, None, None, None, lnum, flags, None);
    if keepalt_is_off() {
        Win::current().w_alt_fnum = opened.alt_fnum;
    }
    opened.empty_fnum = Buf::current().handle;
    Some(opened)
}

/// `:helpclose`: close the first help window in the current tab page.
pub(crate) fn ex_helpclose(excmd: &mut ExArg) {
    let Some(win) = windows().find(|wp| buf_is_help(wp.buffer_or_none())) else {
        return;
    };
    win_close(win, false, excmd.forceit);
}

/// `:exusage`.
pub(crate) fn ex_exusage(_excmd: &mut ExArg) {
    let _ = do_cmdline_cmd(c"help ex-cmd-index");
}

/// `:viusage`.
pub(crate) fn ex_viusage(_excmd: &mut ExArg) {
    let _ = do_cmdline_cmd(c"help normal-index");
}

// -- Finding a subject -----------------------------------------------------

/// Look for a `@xx` language specifier at the end of `subject`. When there
/// is one, cut it off and answer the two language letters.
fn split_help_lang(subject: &mut Vec<u8>) -> Option<[u8; 2]> {
    let len = subject.len();
    if len >= 3
        && subject[len - 3] == b'@'
        && ascii_isalpha(c_int::from(subject[len - 2]))
        && ascii_isalpha(c_int::from(subject[len - 1]))
    {
        let lang = [subject[len - 2], subject[len - 1]];
        subject.truncate(len - 3);
        return Some(lang);
    }
    None
}

/// How well `tag` matches, smaller being better. In order of priority:
/// fewer alphanumeric characters, then fewer characters overall, then a
/// match nearer the start; a match starting with `+` is worse, since that is
/// a feature name rather than a command.
///
/// The tag is assumed to already match the requested subject, at `offset`.
pub(crate) fn help_heuristic(tag: &[u8], offset: c_int, wrong_case: bool) -> c_int {
    let num_letters = tag.iter().filter(|c| c.is_ascii_alphanumeric()).count() as c_int;

    // Weight the letter count far above the character count. A match
    // starting in the middle of a word goes in the last half; a match more
    // than two characters in goes after the ones at the start.
    let at = |i: c_int| {
        usize::try_from(i)
            .ok()
            .and_then(|i| tag.get(i))
            .is_some_and(u8::is_ascii_alphanumeric)
    };
    let mut offset = if offset > 0 && at(offset) && at(offset - 1) {
        offset + 10000
    } else if offset > 2 {
        offset * 200
    } else {
        offset
    };
    if wrong_case {
        offset += 5000;
    }
    // "+" alone is not a feature.
    if tag.len() > 1 && tag[0] == b'+' {
        offset += 100;
    }
    100 * num_letters + tag.len() as c_int + offset
}

/// The order of the match list: by the heuristic score `find_tags` stored
/// with each tag, then by the tag itself so that equal scores order the same
/// way every run.
fn help_compare(a: &HelpTagMatch, b: &HelpTagMatch) -> Ordering {
    a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0))
}

/// Every help tag matching `subject`, best match first, at most
/// `TAG_MANY` of them. With `keep_lang` set, prefer the language of the
/// current buffer.
fn find_help_tags(subject: &[u8], keep_lang: bool) -> Result<Vec<Vec<u8>>, Failed> {
    let mut args = ArrayBuf::<1>::new();
    args.push(Object::string(String_0::from_bytes(subject)));
    let chunk = c"return require'vim._core.help'.escape_subject(...)";
    let res = match nlua_exec_cstr(chunk, args.array(), kRetObject) {
        Ok(value) => value,
        Err(mut err) => {
            emsg_multiline(err.message_or_empty(), Some(c"lua_error"), HLF_E, true);
            err.clear();
            return Err(Failed);
        }
    };
    let escaped = res
        .as_string()
        .expect("`vim._core.help.escape_subject()` answers with a string");
    // The search pattern, cut to the size upstream's copy in the shared
    // `IObuff` had, and read as the C string it was there.
    let escaped = escaped.as_bytes();
    let escaped = &escaped[..escaped.len().min(IOSIZE as usize - 1)];
    let pattern = XString::from_bytes(escaped.split(|&b| b == 0).next().unwrap_or_default());
    drop(res);

    let mut flags = (TAG_HELP | TAG_REGEXP | TAG_NAMES | TAG_VERBOSE | TAG_NO_TAGFUNC) as c_int;
    if keep_lang {
        flags |= TAG_KEEP_LANG as c_int;
    }
    let mut matches = find_help_tag_matches(pattern.as_cstr(), flags, MAXCOL as c_int)?;
    // Sort on the heuristic number `find_tags` put after the tag.
    matches.sort_unstable_by(help_compare);
    // Drop everything past TAG_MANY to keep the listing short.
    matches.truncate(TAG_MANY as usize);
    Ok(matches.into_iter().map(|(tag, _)| tag).collect())
}

/// Tidy the match list for display: strip `@ab` where `ab` is the head of
/// 'helplang', and strip `@en` from any tag that exists in English only.
fn cleanup_help_tags(tags: &mut [Vec<u8>]) {
    // The preferred language as a `@ab` suffix, or none for English.
    let suffix = p_hlg(|hlg| {
        let hlg = hlg.to_bytes();
        // A non-empty 'helplang' is a comma-separated list of two-letter
        // codes, so it always has two bytes.
        (!hlg.is_empty() && hlg[..2.min(hlg.len())] != *b"en")
            .then(|| [b'@', hlg[0], hlg.get(1).copied().unwrap_or(0)])
    });

    for i in 0..tags.len() {
        let Some(len) = tags[i].len().checked_sub(3).filter(|&len| len > 0) else {
            continue;
        };
        if tags[i][len..] != *b"@en" {
            continue;
        }
        // Sorting on priority means the same item in another language
        // may be anywhere; search all of them for a match up to the
        // "@en".
        let tag = &tags[i];
        let elsewhere = tags
            .iter()
            .enumerate()
            .any(|(j, other)| j != i && other.len() == len + 3 && other[..=len] == tag[..=len]);
        if !elsewhere {
            // The item exists only with "@en": drop the suffix.
            tags[i].truncate(len);
        }
    }

    if let Some(suffix) = suffix {
        for tag in tags.iter_mut() {
            if tag.len() > 3 && tag.ends_with(&suffix) {
                tag.truncate(tag.len() - 3);
            }
        }
    }
}

/// The help tags `arg` matches, cleaned up the way the command line offers
/// them -- [`find_help_tags`] then [`cleanup_help_tags`], owned.
pub(crate) fn help_tag_matches(arg: &CStr) -> Result<Vec<XString>, Failed> {
    let mut tags = find_help_tags(arg.to_bytes(), false)?;
    cleanup_help_tags(&mut tags);
    Ok(tags.into_iter().map(XString::from_vec).collect())
}

// -- The help buffer -------------------------------------------------------

/// Force the options a help buffer needs. Called whenever one starts being
/// edited, because a user autocommand may have changed them since the last
/// time.
pub(crate) fn prepare_help_buffer() {
    Buf::current().b_help = true;
    set_option_direct(kOptBuftype, cstr_optval(c"help"), OptionSetFlags::LOCAL, 0);

    // Accept every ASCII character as a keyword character except ' ',
    // '*', '"' and '|', plus the latin1 word characters translated help
    // files use. Only set it when needed: `buf_init_chartab` is work.
    let isk = c"!-~,^*,^|,^\",192-255";
    if Buf::current().b_p_isk.bytes() != isk.to_bytes() {
        set_option_direct(kOptIskeyword, cstr_optval(isk), OptionSetFlags::LOCAL, 0);
        check_buf_options(Buf::current());
        buf_init_chartab(Buf::current(), false);
    }

    // Don't use the global foldmethod.
    set_option_direct(
        kOptFoldmethod,
        cstr_optval(c"manual"),
        OptionSetFlags::LOCAL,
        0,
    );

    Buf::current().b_p_ts = 8;
    Buf::current().b_p_ma = 0; // not modifiable
    Buf::current().b_p_bin = 0; // reset 'bin' before reading the file
    let mut win = Win::current();
    let wo = &mut win.w_onebuf_opt;
    wo.wo_list = 0;
    wo.wo_nu = 0;
    wo.wo_rnu = 0;
    wo.wo_scb = 0; // no scroll binding
    wo.wo_crb = 0; // no cursor binding
    wo.wo_arab = 0;
    wo.wo_rl = 0; // help windows are left-to-right
    wo.wo_fen = 0; // no folding
    wo.wo_diff = 0;
    wo.wo_spell = 0;

    set_buflisted(0);
}

/// Populate `*local-additions*` in `help.txt`.
pub(crate) fn get_local_additions() {
    let chunk = c"return require'vim._core.help'.local_additions()";
    if let Err(mut err) = nlua_exec_cstr(chunk, Array::EMPTY, kRetNilBool) {
        emsg_multiline(err.message_or_empty(), Some(c"lua_error"), HLF_E, true);
        err.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tag names chosen for every scoring rule: alphanumerics against
    /// punctuation, `+feature` and a bare `+`, a match mid-word, multibyte
    /// letters (not alphanumeric to the ASCII test), and the empty name.
    const NAMES: &[&[u8]] = &[
        b"",
        b"+",
        b"+a",
        b"+demofeature",
        b"a",
        b"ab",
        b":demo",
        b":democmd",
        b"'dm'",
        b"'demoopt'",
        b"CTRL-X",
        b"i_CTRL-X_demo",
        b"i_<Esc>",
        b"v_demo",
        b"demo()",
        b"demo.txt",
        b"demo-intro@en",
        b"g:demo_var",
        b"E9999",
        b"Demo-Upper",
        b"xdemo",
        b"quote\"",
        b"|",
        b"*",
        b"multi-\xc3\xa9",
        b"\xe6\x97\xa5\xe6\x9c\xac\xe8\xaa\x9e",
        b"a b\tc",
        b"0123456789",
    ];

    /// FNV-1a, as `tests/unit/support/corpus.rs`'s `Digest`.
    fn fold(digest: &mut u64, value: i64) {
        for byte in value.to_le_bytes() {
            *digest ^= u64::from(byte);
            *digest = digest.wrapping_mul(0x0100_0000_01b3);
        }
    }

    fn score(name: &[u8], offset: c_int, wrong_case: bool) -> c_int {
        help_heuristic(name, offset, wrong_case)
    }

    #[test]
    fn heuristic_scores_are_the_recorded_ones() {
        let mut digest = 0xcbf2_9ce4_8422_2325_u64;
        for name in NAMES {
            let len = c_int::try_from(name.len()).expect("short");
            for offset in 0..=len + 2 {
                for wrong_case in [false, true] {
                    fold(&mut digest, i64::from(score(name, offset, wrong_case)));
                }
            }
        }
        assert_eq!(digest, HEURISTIC, "help_heuristic scores moved");
        // A few by hand, so a moved digest has something to read.
        assert_eq!(score(b"+demofeature", 0, false), 100 * 11 + 12 + 100);
        assert_eq!(score(b"demo-intro", 5, false), 100 * 9 + 10 + 5 * 200);
        assert_eq!(score(b"xdemo", 1, true), 100 * 5 + 5 + 1 + 10000 + 5000);
    }

    /// The sort `find_help_tags` gives its matches: score first, then name.
    #[test]
    fn matches_sort_by_score_then_name() {
        let rows: &[(&[u8], &[u8])] = &[
            (b"b@en", b"000300"),
            (b"a@en", b"000300"),
            (b"c@cn", b"000100"),
            (b"a@cn", b"001000"),
            (b"\xc3\xa9@en", b"000300"),
            (b"z@en", b"000099"),
            (b"a@en", b"000300"),
        ];
        let mut owned: Vec<HelpTagMatch> = rows
            .iter()
            .map(|(tag, score)| (tag.to_vec(), score.to_vec()))
            .collect();
        owned.sort_unstable_by(help_compare);
        let sorted: Vec<&[u8]> = owned.iter().map(|(tag, _)| tag.as_slice()).collect();
        let want: &[&[u8]] = &[
            b"z@en",
            b"c@cn",
            b"a@en",
            b"a@en",
            b"b@en",
            b"\xc3\xa9@en",
            b"a@cn",
        ];
        assert_eq!(sorted, want);
    }

    /// `@xx` at the end of a subject is cut off and answered.
    #[test]
    fn a_language_suffix_is_split_off() {
        for (subject, rest, lang) in [
            (&b"demo@cn"[..], &b"demo"[..], Some(*b"cn")),
            (b"@en", b"", Some(*b"en")),
            (b"demo@c1", b"demo@c1", None),
            (b"demo@", b"demo@", None),
            (b"x@EN", b"x", Some(*b"EN")),
            (b"ab", b"ab", None),
            (b"demo@@cn", b"demo@", Some(*b"cn")),
        ] {
            let mut owned = subject.to_vec();
            let found = split_help_lang(&mut owned);
            assert_eq!((owned.as_slice(), found), (rest, lang), "{subject:?}");
        }
    }

    /// Recorded on the pointer form.
    const HEURISTIC: u64 = 18_163_768_497_455_475_670;
}
