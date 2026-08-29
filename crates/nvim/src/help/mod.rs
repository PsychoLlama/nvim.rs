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
//! The first two work in `NameBuff` and `IObuff`, the editor's two shared
//! scratch buffers. That is upstream's choice, kept: the search pattern goes
//! into `IObuff` and `find_tags` reads it straight back out. Nothing reached
//! from here wants either buffer meanwhile, which is why they are taken as
//! raw pointers rather than borrowed.
//!
//! Original: `src/nvim/help.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

mod tags;

use crate::api::private::helpers::{api_clear_error, api_free_object, cstr_as_string};
use crate::ascii::{ascii_isalpha, ascii_iswhite};
use crate::buffer::{buf_is_help, cur_win, find_buf, set_buflisted, wipe_buffer};
use crate::charset::buf_init_chartab;
use crate::ex_cmds::do_ecmd;
use crate::ex_docmd::{cmdmod_has, do_cmdline_cmd};
use crate::highlight_group::HLF_E;
use crate::lua::executor::nlua_exec;
use crate::main::{
    Columns, KeyTyped, cmdmod, curbuf, curwin, e_noident, p_hf, p_hh, p_hlg, p_sb, restart_edit,
};
use crate::memory::{xfree, xstrdup, xstrlcpy};
use crate::message::{emsg, emsg_multiline};
use crate::option::set_option_direct;
use crate::options::{kOptBuftype, kOptFoldmethod, kOptIskeyword};
use crate::optionstr::check_buf_options;
use crate::os::cshim::{gettext, strncmp};
use crate::os::fs::os_fopen;
use crate::path::free_wild;
use crate::pos::MAXCOL;
use crate::tag::{do_tag, find_tags};
use crate::types::builders::static_cstring;
use crate::types::{
    Array, ArrayBuf, CmdModFlags, Error, FAIL, IOSIZE, LuaRetMode, NUL, OK, Object, OptInt, OptVal,
    OptValData, OptionSetFlags, exarg_T, file_comparison, kErrorTypeNone, kObjectTypeString,
    linenr_T, size_t,
};
use crate::window::{WSP_BOT, WSP_HELP, WSP_TOP, win_close, win_enter, win_setheight, win_split};
use crate::winlayer::windows;
use crate::{semsg_c, smsg_c};
use ::libc::{fclose, qsort, strcasecmp, strcmp, strlen};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

pub(crate) use tags::ex_helptags;

/// Constants the transpiler copied in from the headers this module includes.
mod flag {
    use super::{LuaRetMode, c_int, file_comparison};
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

    /// `do_ecmd` flags and its "last line" line number.
    pub(super) const ECMD_HIDE: c_uint = 1;
    pub(super) const ECMD_SET_HELP: c_uint = 2;
    pub(super) const ECMD_LASTL: c_int = 0;

    /// `path_full_compare`'s "same file" answer.
    pub(super) const kEqualFiles: file_comparison = 1;
    /// `nlua_exec` return modes.
    pub(super) const kRetObject: LuaRetMode = 0;
    pub(super) const kRetNilBool: LuaRetMode = 1;
    /// `OptVal`'s string tag.
    pub(super) const kOptValTypeString: c_int = 2;
}

use flag::{
    DT_HELP, ECMD_HIDE, ECMD_LASTL, ECMD_SET_HELP, TAG_HELP, TAG_KEEP_LANG, TAG_MANY, TAG_NAMES,
    TAG_NO_TAGFUNC, TAG_REGEXP, TAG_VERBOSE, kOptValTypeString, kRetNilBool, kRetObject,
};

/// An error slot with nothing in it: C's `ERROR_INIT`.
const NO_ERROR: Error = Error {
    type_0: kErrorTypeNone,
    msg: ptr::null_mut(),
};

/// Whether the `:keepalt` modifier is off, so that the alternate file may
/// be changed.
fn keepalt_is_off() -> bool {
    !cmdmod_has(CmdModFlags::KEEPALT)
}

/// A borrowed string option value; `set_option_direct` copies what it keeps.
const fn cstr_optval(value: &'static CStr) -> OptVal {
    OptVal {
        type_0: kOptValTypeString,
        data: OptValData {
            string: static_cstring(value),
        },
    }
}

// -- `:help` ---------------------------------------------------------------

/// `:help`, and `:help!` — which resolves the best tag under the cursor
/// instead of taking a subject.
///
/// # Safety
/// `eap` is null or the current Ex command; its `arg` is a writable,
/// NUL-terminated command line, which this truncates at the first `\n`,
/// `\r`, or `|` that starts a following command.
pub(crate) unsafe fn ex_help(eap: *mut exarg_T) {
    let old_key_typed = KeyTyped.get();

    // SAFETY: caller contract; the command line is writable.
    let mut arg = unsafe {
        if eap.is_null() {
            c"".as_ptr().cast_mut()
        } else {
            split_off_next_cmd(eap);
            if (*eap).skip != 0 {
                return;
            }
            trim_trailing_blanks((*eap).arg)
        }
    };

    // SAFETY: `arg` is NUL-terminated, and writable whenever `eap` is set --
    // which is the only case where `check_help_lang` can find a `@xx` to
    // strip.
    let lang = unsafe { check_help_lang(arg) };

    // SAFETY: caller contract.
    let helpbang = unsafe { !eap.is_null() && (*eap).forceit != 0 && *arg == NUL as c_char };
    if unsafe { *arg == NUL as c_char } && !helpbang {
        arg = c"help.txt".as_ptr().cast_mut();
    }

    // `:help!` asks Lua for the best tag at the cursor.
    let mut allocated_arg: *mut c_char = ptr::null_mut();
    if helpbang {
        // SAFETY: a static chunk, no arguments, and our own error slot.
        allocated_arg = unsafe { resolve_tag_at_cursor() };
        if allocated_arg.is_null() {
            emsg(gettext(e_noident));
            return;
        }
        arg = allocated_arg;
    }

    let mut num_matches: c_int = 0;
    let mut matches: *mut *mut c_char = ptr::null_mut();
    // SAFETY: `arg` is NUL-terminated; the two out-parameters are ours.
    let forceit = !eap.is_null() && unsafe { (*eap).forceit } != 0;
    let (out_n, out_m) = (&raw mut num_matches, &raw mut matches);
    let n = unsafe { find_help_tags(arg, out_n, out_m, forceit) };

    // The first match in the requested language is the best match.
    let mut i: c_int = 0;
    if n != FAIL && !lang.is_null() {
        while i < num_matches {
            // SAFETY: `matches` holds `num_matches` NUL-terminated strings.
            let m = unsafe { *matches.offset(i as isize) };
            let len = unsafe { strlen(m) } as c_int;
            if len > 3
                && unsafe { *m.offset((len - 3) as isize) } == b'@' as c_char
                && unsafe { strcasecmp(m.offset((len - 2) as isize), lang) } == 0
            {
                break;
            }
            i += 1;
        }
    }
    if i >= num_matches || n == FAIL {
        // SAFETY: `lang` and `arg` are NUL-terminated; both carry bytes, so
        // they go through vim's own printf rather than `format_args!`.
        if lang.is_null() {
            unsafe { semsg_c!(gettext(c"E149: No help for %s"), arg) };
        } else {
            unsafe { semsg_c!(gettext(c"E661: No '%s' help for %s"), lang, arg) };
        }
        if n != FAIL {
            unsafe { free_wild(num_matches, matches) };
        }
        unsafe { xfree(allocated_arg.cast::<c_void>()) };
        return;
    }

    // SAFETY: `i` is below `num_matches`.
    let tag = unsafe { xstrdup(*matches.offset(i as isize)) };
    // SAFETY: `matches` is `find_tags`'s allocation.
    unsafe { free_wild(num_matches, matches) };

    // SAFETY: the window list is live on the main thread; `tag` is owned.
    if let Some(opened) = unsafe { enter_help_window() } {
        restart_edit.set(0);
        // Restore KeyTyped: setting 'filetype=help' may have reset it,
        // and `do_tag` needs it to open folds under the cursor.
        KeyTyped.set(old_key_typed);

        unsafe { do_tag(tag, DT_HELP as c_int, 1, 0, true) };

        // Delete the empty buffer if we are not using it. Careful:
        // autocommands may have jumped to another window, so check that
        // the buffer is not in one.
        if opened.empty_fnum != 0
            && unsafe { (*curbuf.get()).handle } != opened.empty_fnum
            && let Some(buf) = find_buf(opened.empty_fnum).filter(|b| b.b_nwindows == 0)
        {
            wipe_buffer(buf, true);
        }
        // Keep the previous alternate file.
        if opened.alt_fnum != 0
            && unsafe { (*curwin.get()).w_alt_fnum } == opened.empty_fnum
            && keepalt_is_off()
        {
            unsafe { (*curwin.get()).w_alt_fnum = opened.alt_fnum };
        }
    }
    unsafe { xfree(tag.cast::<c_void>()) };
    unsafe { xfree(allocated_arg.cast::<c_void>()) };
}

/// A `:help` command ends at the first LF, or at a `|` followed by some
/// text. Terminate the argument there and point `nextcmd` at the rest.
///
/// # Safety
/// `eap`'s `arg` is a writable NUL-terminated command line.
unsafe fn split_off_next_cmd(eap: *mut exarg_T) {
    // SAFETY: caller contract.
    let mut arg = unsafe { (*eap).arg };
    while unsafe { *arg } != NUL as c_char {
        if unsafe { *arg } == b'\n' as c_char
            || unsafe { *arg } == b'\r' as c_char
            || (unsafe { *arg } == b'|' as c_char
                && unsafe { *arg.offset(1) } != NUL as c_char
                && unsafe { *arg.offset(1) } != b'|' as c_char)
        {
            unsafe { *arg = NUL as c_char };
            arg = unsafe { arg.offset(1) };
            unsafe { (*eap).nextcmd = arg };
            return;
        }
        arg = unsafe { arg.offset(1) };
    }
}

/// Terminate `arg` before its trailing whitespace, except where a backslash
/// escapes it, and return it.
///
/// # Safety
/// `arg` is writable and NUL-terminated.
unsafe fn trim_trailing_blanks(arg: *mut c_char) -> *mut c_char {
    // SAFETY: caller contract. `p` starts one before the NUL, which is `arg`
    // itself for an empty string, so the loop never steps below `arg`.
    let mut p = unsafe { arg.add(strlen(arg)).offset(-1) };
    while p > arg
        && ascii_iswhite(unsafe { *p } as c_int)
        && unsafe { *p.offset(-1) } != b'\\' as c_char
    {
        unsafe { *p = NUL as c_char };
        p = unsafe { p.offset(-1) };
    }
    arg
}

/// The tag `vim._core.help.resolve_tag()` picks for the cursor position, as
/// an owned string, or null when it declines to pick one.
///
/// # Safety
/// Runs Lua: main thread only.
unsafe fn resolve_tag_at_cursor() -> *mut c_char {
    let mut err = NO_ERROR;
    // SAFETY: a static chunk, an empty argument array, and our error slot.
    let chunk = static_cstring(c"return require'vim._core.help'.resolve_tag()");
    let (name, arena, slot) = (ptr::null(), ptr::null_mut(), &raw mut err);
    let res = unsafe { nlua_exec(chunk, name, Array::EMPTY, kRetObject, arena, slot) };
    // SAFETY: `res` is the chunk's answer and `err` our slot; both are
    // consumed here.
    let tag = if err.type_0 == kErrorTypeNone
        && res.type_0 == kObjectTypeString
        && !unsafe { res.data.string }.is_empty()
    {
        unsafe { xstrdup(res.data.string.data()) }
    } else {
        ptr::null_mut()
    };
    unsafe { api_free_object(res) };
    unsafe { api_clear_error(&raw mut err) };
    tag
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
///
/// # Safety
/// Main thread; the window list is live. Runs autocommands.
unsafe fn enter_help_window() -> Option<HelpWindow> {
    let mut opened = HelpWindow {
        empty_fnum: 0,
        alt_fnum: 0,
    };
    // Re-use an existing help window; always open a new one for
    // `:tab help`.
    let same_tab = cmdmod.with(|m| m.cmod_tab) == 0;
    if buf_is_help(cur_win().buffer_or_none()) && same_tab {
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
        // SAFETY: a live window. Runs autocommands.
        unsafe { win_enter(wp.raw(), true) };
        return Some(opened);
    }

    // SAFETY: the window list is a live intrusive list on the main thread.
    // No help window yet: check that 'helpfile' can be read at all.
    let helpfd = unsafe { os_fopen(p_hf.get(), c"rb".as_ptr()) };
    if helpfd.is_null() {
        unsafe { smsg_c!(0, c"Help file \"%s\" not found".as_ptr(), p_hf.get()) };
        return None;
    }
    unsafe { fclose(helpfd) };

    // Split off a help window; put it at the far top when no position
    // was given and the current window is vertically split and narrow.
    let mut split = WSP_HELP as c_int;
    if cmdmod.with(|m| m.cmod_split) == 0
        && unsafe { (*curwin.get()).w_width } != Columns.get()
        && unsafe { (*curwin.get()).w_width } < 80
    {
        split |= if p_sb.get() != 0 {
            WSP_BOT as c_int
        } else {
            WSP_TOP as c_int
        };
    }
    if win_split(0, split) == FAIL {
        return None;
    }
    if (unsafe { (*curwin.get()).w_height } as OptInt) < p_hh.get() {
        win_setheight(p_hh.get() as c_int);
    }

    // Open the help file. `do_ecmd` sets `b_help` and `readfile` sets
    // 'readonly'. The buffer is still open, so don't store info.
    opened.alt_fnum = unsafe { (*curbuf.get()).handle };
    let (fnum, fname, sfname) = (0, ptr::null_mut(), ptr::null_mut());
    let (eap_0, win) = (ptr::null_mut(), ptr::null_mut());
    let (lnum, flags) = (ECMD_LASTL as linenr_T, (ECMD_HIDE + ECMD_SET_HELP) as c_int);
    // SAFETY: the editor's own current window and buffer.
    unsafe { do_ecmd(fnum, fname, sfname, eap_0, lnum, flags, win) };
    if keepalt_is_off() {
        unsafe { (*curwin.get()).w_alt_fnum = opened.alt_fnum };
    }
    opened.empty_fnum = unsafe { (*curbuf.get()).handle };
    Some(opened)
}

/// `:helpclose`: close the first help window in the current tab page.
///
/// # Safety
/// `eap` is the current Ex command.
pub(crate) unsafe fn ex_helpclose(eap: *mut exarg_T) {
    let Some(win) = windows().find(|wp| buf_is_help(wp.buffer_or_none())) else {
        return;
    };
    // SAFETY: caller contract; a live window.
    unsafe { win_close(win.raw(), false, (*eap).forceit != 0) };
}

/// `:exusage`.
///
/// # Safety
/// `eap` is unused, but the signature is the Ex-command one.
pub(crate) unsafe fn ex_exusage(_eap: *mut exarg_T) {
    // SAFETY: a static command line.
    unsafe { do_cmdline_cmd(c"help ex-cmd-index".as_ptr()) };
}

/// `:viusage`.
///
/// # Safety
/// As [`ex_exusage`].
pub(crate) unsafe fn ex_viusage(_eap: *mut exarg_T) {
    // SAFETY: a static command line.
    unsafe { do_cmdline_cmd(c"help normal-index".as_ptr()) };
}

// -- Finding a subject -----------------------------------------------------

/// Look for a `@xx` language specifier at the end of `arg`. When there is
/// one, cut it off — the `@` becomes the terminator — and answer the two
/// language letters; otherwise answer null.
///
/// # Safety
/// `arg` is NUL-terminated, and writable if it can end in `@xx`.
pub(crate) unsafe fn check_help_lang(arg: *mut c_char) -> *mut c_char {
    // SAFETY: caller contract.
    let len = unsafe { strlen(arg) } as isize;
    if len >= 3
        && unsafe { *arg.offset(len - 3) } == b'@' as c_char
        && ascii_isalpha(unsafe { *arg.offset(len - 2) } as c_int)
        && ascii_isalpha(unsafe { *arg.offset(len - 1) } as c_int)
    {
        unsafe { *arg.offset(len - 3) = NUL as c_char };
        return unsafe { arg.offset(len - 2) };
    }
    ptr::null_mut()
}

/// How well `matched_string` matches, smaller being better. In order of
/// priority: fewer alphanumeric characters, then fewer characters overall,
/// then a match nearer the start; a match starting with `+` is worse, since
/// that is a feature name rather than a command.
///
/// The string is assumed to already match the requested subject.
///
/// # Safety
/// `matched_string` is NUL-terminated with at least `offset + 1` bytes.
pub(crate) unsafe fn help_heuristic(
    matched_string: *mut c_char,
    offset: c_int,
    wrong_case: bool,
) -> c_int {
    // SAFETY: caller contract.
    let tag = unsafe { CStr::from_ptr(matched_string) }.to_bytes();
    let num_letters = tag.iter().filter(|c| c.is_ascii_alphanumeric()).count() as c_int;

    // Weight the letter count far above the character count. A match
    // starting in the middle of a word goes in the last half; a match more
    // than two characters in goes after the ones at the start.
    let at = |i: c_int| tag.get(i as usize).is_some_and(u8::is_ascii_alphanumeric);
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

/// `qsort` comparator over the match list: by the heuristic number
/// `find_tags` stored after each tag's NUL, then by the tag itself so that
/// equal scores order the same way every run.
///
/// # Safety
/// Both arguments point at a `char *` whose target is NUL-terminated and
/// followed by a second NUL-terminated string.
unsafe extern "C" fn help_compare(s1: *const c_void, s2: *const c_void) -> c_int {
    // SAFETY: caller contract.
    let t1 = unsafe { *s1.cast::<*mut c_char>() };
    let t2 = unsafe { *s2.cast::<*mut c_char>() };
    let cmp = unsafe { strcmp(t1.add(strlen(t1)).offset(1), t2.add(strlen(t2)).offset(1)) };
    if cmp != 0 {
        return cmp;
    }
    unsafe { strcmp(t1, t2) }
}

/// Find every help tag matching `arg`, best match first, and hand the list
/// back through `matches`/`num_matches`. With `keep_lang` set, prefer the
/// language of the current buffer.
///
/// # Safety
/// `arg` is NUL-terminated; `num_matches` and `matches` are writable. Runs
/// Lua, so main thread only.
pub(crate) unsafe fn find_help_tags(
    arg: *const c_char,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
    keep_lang: bool,
) -> c_int {
    let mut err = NO_ERROR;
    // The search pattern lives here between the two calls below. Upstream
    // uses the shared `IObuff`, and `find_tags` runs a tag function.
    let mut pattern = [0 as c_char; IOSIZE as usize];
    let iobuff = pattern.as_mut_ptr();
    let mut args = ArrayBuf::<1>::new();
    // SAFETY: `arg` is NUL-terminated and outlives the call, which only
    // reads it.
    args.push(Object::string(unsafe { cstr_as_string(arg) }));
    // SAFETY: a static chunk, an argument array borrowing `args`, and our
    // own error slot.
    let chunk = static_cstring(c"return require'vim._core.help'.escape_subject(...)");
    let (name, arena, slot) = (ptr::null(), ptr::null_mut(), &raw mut err);
    let res = unsafe { nlua_exec(chunk, name, args.array(), kRetObject, arena, slot) };

    // SAFETY: `err` is our slot and `res` the chunk's answer.
    if err.type_0 != kErrorTypeNone {
        unsafe { emsg_multiline(err.msg, c"lua_error".as_ptr(), HLF_E, true) };
        unsafe { api_clear_error(&raw mut err) };
        return FAIL;
    }
    unsafe { api_clear_error(&raw mut err) };
    debug_assert!(
        res.type_0 == kObjectTypeString,
        "res.type == kObjectTypeString"
    );
    unsafe { xstrlcpy(iobuff, res.data.string.data(), IOSIZE as usize) };
    unsafe { api_free_object(res) };

    let mut flags = (TAG_HELP | TAG_REGEXP | TAG_NAMES | TAG_VERBOSE | TAG_NO_TAGFUNC) as c_int;
    if keep_lang {
        flags |= TAG_KEEP_LANG as c_int;
    }
    // SAFETY: the out-parameters are the caller's, and `pattern` holds the
    // NUL-terminated text written just above.
    unsafe { *matches = ptr::null_mut() };
    unsafe { *num_matches = 0 };
    let (mincount, buf) = (MAXCOL as c_int, ptr::null_mut());
    let found = unsafe { find_tags(iobuff, num_matches, matches, flags, mincount, buf) };
    if found == OK && unsafe { *num_matches } > 0 {
        // Sort on the heuristic number `find_tags` put after the tag.
        let base = unsafe { (*matches).cast::<c_void>() };
        let (count, width) = (unsafe { *num_matches } as size_t, size_of::<*mut c_char>());
        unsafe { qsort(base, count, width, Some(help_compare)) };
        // Drop everything past TAG_MANY to keep the listing short.
        while unsafe { *num_matches } > TAG_MANY as c_int {
            unsafe { *num_matches -= 1 };
            unsafe { xfree((*(*matches).offset(*num_matches as isize)).cast::<c_void>()) };
        }
    }
    OK
}

/// Tidy the match list for display: strip `@ab` where `ab` is the head of
/// 'helplang', and strip `@en` from any tag that exists in English only.
///
/// # Safety
/// `file` holds `num_file` writable NUL-terminated strings.
pub(crate) unsafe fn cleanup_help_tags(num_file: c_int, file: *mut *mut c_char) {
    // The preferred language as a `@ab` suffix, or empty for English.
    let mut suffix = [NUL as c_char; 4];
    // SAFETY: 'helplang' is a NUL-terminated option string; a non-empty one
    // always has at least two bytes, since it is a comma-separated list of
    // two-letter codes.
    let hlg = p_hlg.get();
    if unsafe { *hlg } != NUL as c_char
        && (unsafe { *hlg } != b'e' as c_char || unsafe { *hlg.offset(1) } != b'n' as c_char)
    {
        suffix[0] = b'@' as c_char;
        suffix[1] = unsafe { *hlg };
        suffix[2] = unsafe { *hlg.offset(1) };
    }

    // SAFETY: caller contract; every truncation writes inside a string.
    for i in 0..num_file {
        let tag = unsafe { *file.offset(i as isize) };
        let len = unsafe { strlen(tag) } as c_int - 3;
        if len <= 0 || unsafe { strcmp(tag.offset(len as isize), c"@en".as_ptr()) } != 0 {
            continue;
        }
        // Sorting on priority means the same item in another language
        // may be anywhere; search all of them for a match up to the
        // "@en".
        let mut j = 0;
        while j < num_file {
            let other = unsafe { *file.offset(j as isize) };
            if j != i
                && unsafe { strlen(other) } as c_int == len + 3
                && unsafe { strncmp(tag, other, len as size_t + 1) } == 0
            {
                break;
            }
            j += 1;
        }
        if j == num_file {
            // The item exists only with "@en": drop the suffix.
            unsafe { *tag.offset(len as isize) = NUL as c_char };
        }
    }

    if suffix[0] != NUL as c_char {
        for i in 0..num_file {
            let tag = unsafe { *file.offset(i as isize) };
            let len = unsafe { strlen(tag) } as c_int - 3;
            if len > 0 && unsafe { strcmp(tag.offset(len as isize), suffix.as_ptr()) } == 0 {
                unsafe { *tag.offset(len as isize) = NUL as c_char };
            }
        }
    }
}

// -- The help buffer -------------------------------------------------------

/// Force the options a help buffer needs. Called whenever one starts being
/// edited, because a user autocommand may have changed them since the last
/// time.
///
/// # Safety
/// Main thread; `curbuf` and `curwin` are live.
pub(crate) unsafe fn prepare_help_buffer() {
    // SAFETY: `curbuf`/`curwin` are the editor's current buffer and window.
    unsafe { (*curbuf.get()).b_help = true };
    set_option_direct(kOptBuftype, cstr_optval(c"help"), OptionSetFlags::LOCAL, 0);

    // Accept every ASCII character as a keyword character except ' ',
    // '*', '"' and '|', plus the latin1 word characters translated help
    // files use. Only set it when needed: `buf_init_chartab` is work.
    let isk = c"!-~,^*,^|,^\",192-255";
    if unsafe { strcmp((*curbuf.get()).b_p_isk, isk.as_ptr()) } != 0 {
        set_option_direct(kOptIskeyword, cstr_optval(isk), OptionSetFlags::LOCAL, 0);
        unsafe { check_buf_options(curbuf.get()) };
        unsafe { buf_init_chartab(curbuf.get(), false) };
    }

    // Don't use the global foldmethod.
    set_option_direct(
        kOptFoldmethod,
        cstr_optval(c"manual"),
        OptionSetFlags::LOCAL,
        0,
    );

    unsafe { (*curbuf.get()).b_p_ts = 8 };
    unsafe { (*curbuf.get()).b_p_ma = 0 }; // not modifiable
    unsafe { (*curbuf.get()).b_p_bin = 0 }; // reset 'bin' before reading the file
    let wo = unsafe { &raw mut (*curwin.get()).w_onebuf_opt };
    unsafe { (*wo).wo_list = 0 };
    unsafe { (*wo).wo_nu = 0 };
    unsafe { (*wo).wo_rnu = 0 };
    unsafe { (*wo).wo_scb = 0 }; // no scroll binding
    unsafe { (*wo).wo_crb = 0 }; // no cursor binding
    unsafe { (*wo).wo_arab = 0 };
    unsafe { (*wo).wo_rl = 0 }; // help windows are left-to-right
    unsafe { (*wo).wo_fen = 0 }; // no folding
    unsafe { (*wo).wo_diff = 0 };
    unsafe { (*wo).wo_spell = 0 };

    unsafe { set_buflisted(0) };
}

/// Populate `*local-additions*` in `help.txt`.
///
/// # Safety
/// Runs Lua: main thread only.
pub(crate) unsafe fn get_local_additions() {
    let mut err = NO_ERROR;
    // SAFETY: a static chunk, no arguments, and our own error slot.
    let chunk = static_cstring(c"return require'vim._core.help'.local_additions()");
    let (name, arena, slot) = (ptr::null(), ptr::null_mut(), &raw mut err);
    let res = unsafe { nlua_exec(chunk, name, Array::EMPTY, kRetNilBool, arena, slot) };
    if err.type_0 != kErrorTypeNone {
        unsafe { emsg_multiline(err.msg, c"lua_error".as_ptr(), HLF_E, true) };
    }
    unsafe { api_free_object(res) };
    unsafe { api_clear_error(&raw mut err) };
}
