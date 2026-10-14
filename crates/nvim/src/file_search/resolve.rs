//! Looking a name up along `'path'` or `'cdpath'`.
//!
//! [`PathWalk`] is the loop over the option's entries: it expands
//! environment variables in the name, decides whether the name is absolute
//! enough to skip the option entirely, and otherwise drives a
//! [`FindContext`] once per entry, remembering where it got to so that a
//! repeat call answers the next match. `'suffixesadd'` is tried at every
//! candidate.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::memory::XString;
use crate::message_fmt::{emsg_text, msg_cstr};
use crate::option::vars::P_CDPATH;
use crate::option::{option_parts, split_option_part};
use crate::optionstr::OptString;
use crate::os::env::expand_env_esc_of;
use crate::os::fs::path_exists;
use crate::path::buffer_path;
use crate::tr_c;
use crate::types::MAXPATHL;
use crate::winlayer::Buf;
use core::ffi::c_int;

/// The `nth` match (from 1) for `name` along `'path'`, relative to the
/// current buffer's file; reported when there is none and `options` asks
/// for it. Each match after the first resumes the walk the first started.
pub(crate) fn find_nth_file_in_path(
    name: &[u8],
    options: FileNameOpts,
    nth: LineNr,
) -> Option<XString> {
    let mut walk = PathWalk::new();
    // A copy: the search can run user code, which may rename the buffer.
    let rel_fname = Buf::current().name.full().map(XString::from_cstr);
    let rel_fname = rel_fname.as_ref().map(XString::as_cstr);
    let mut found = find_file_in_path(&mut walk, name, options, rel_fname);
    let mut n = nth;
    while found.is_some() && {
        n -= 1;
        n > 0
    } {
        found = walk.next(options);
    }
    found
}

/// Start `walk` on the file `name` in `'path'`, answering the first match.
/// Also finds directory names. [`PathWalk::next`] answers the others.
///
/// If nothing is found, `FileNameOpts::MESS` issues
/// `Can't find file "<file>" in path`.
///
/// `rel_fname` is the file the search is relative to.
pub(crate) fn find_file_in_path(
    walk: &mut PathWalk,
    name: &[u8],
    options: FileNameOpts,
    rel_fname: Option<&CStr>,
) -> Option<XString> {
    let suffixes = Buf::current().b_p_sua.get();
    walk.start(
        name,
        options,
        buffer_path(),
        FINDFILE_BOTH,
        rel_fname,
        suffixes.as_cstr(),
    )
}

/// Find the directory name `name` in `'cdpath'`.
///
/// options:
/// - `FileNameOpts::MESS`   give error message when not found
/// - `FileNameOpts::UNESC`  unescape backslashes
///
/// `rel_fname` is the file the search is relative to.
pub(crate) fn find_directory_in_path(
    name: &[u8],
    options: FileNameOpts,
    rel_fname: Option<&CStr>,
) -> Option<XString> {
    PathWalk::new().start(name, options, P_CDPATH.get(), FINDFILE_DIR, rel_fname, c"")
}

/// `name` with environment variables expanded.
///
/// With `FileNameOpts::UNESC` every `"\ "` in the result becomes a plain
/// space, so that a name escaped for the command line reaches the file
/// system whole.
fn prepare_name(name: &[u8], options: FileNameOpts) -> XString {
    let expanded = cstr::with_terminated(name, |name| expand_env_esc_of(name, false, true, None));
    if !options.has(FileNameOpts::UNESC) {
        return expanded;
    }
    // Change all "\ " to " ".
    let mut name = expanded.to_vec();
    let mut out = 0;
    let mut at = 0;
    while at < name.len() {
        if name[at] == b'\\' && name.get(at + 1) == Some(&b' ') {
            at += 1;
        }
        name[out] = name[at];
        out += 1;
        at += 1;
    }
    name.truncate(out);
    XString::from_vec(name)
}

/// Is `name` `"."`, `".."`, or something below one of them? Such a name is
/// meant relative to the current directory and never looked for in `'path'`.
fn rel_to_curdir(name: &[u8]) -> bool {
    let at = |i: usize| cstr::byte_at(name, i);
    let ends_component = |i: usize| at(i) == 0 || vim_ispathsep(c_int::from(at(i)));
    at(0) == b'.' && (ends_component(1) || (at(1) == b'.' && ends_component(2)))
}

/// Does `name` need no `'path'` at all? An absolute name means itself, and
/// "..", "../path", "." and "./path" mean the current directory.
fn needs_no_path(name: &XString) -> bool {
    vim_is_abs_name(name.as_cstr()) || rel_to_curdir(name)
}

/// Try `stem`, then `stem` with each part of `'suffixesadd'` appended, and
/// answer the first that exists and is of the wanted kind.
fn try_suffixes(stem: &[u8], find_what: c_int, suffixes: &[u8]) -> Option<XString> {
    let mut candidate = Name::from_bytes(stem);
    let mut suffixes = option_parts(suffixes);
    loop {
        if path_exists(candidate.as_cstr())
            && (find_what == FINDFILE_BOTH
                || (find_what == FINDFILE_DIR) == dir_exists(candidate.as_cstr()))
        {
            return Some(XString::from_bytes(candidate.bytes()));
        }
        let suffix = suffixes.next()?;
        // Cut where upstream's `copy_option_part` into the rest of a
        // `MAXPATHL` buffer cuts it.
        let room = (MAXPATHL as usize - 1).saturating_sub(stem.len());
        candidate.truncate(stem.len());
        candidate.extend(&suffix[..suffix.len().min(room)]);
    }
}

/// Look for a name that needs no `'path'`: absolute, or relative to the
/// current directory.
///
/// `FileNameOpts::REL` asks for the directory of `rel_fname` to be tried
/// first; the current directory is the second and last try.
fn find_without_path(
    name: &XString,
    options: FileNameOpts,
    find_what: c_int,
    rel_fname: Option<&CStr>,
    suffixes: &[u8],
) -> Option<XString> {
    if path_with_url(name.as_cstr()) != 0 {
        return Some(name.clone());
    }

    // Run 1 is relative to "rel_fname"'s directory, run 2 is relative to
    // the current directory. Only run 2 happens when the first does not
    // apply.
    if let Some(rel_fname) = rel_fname
        && rel_to_curdir(name)
        && options.has(FileNameOpts::REL)
        && rel_fname.count_bytes() + name.len() < MAXPATHL as usize
    {
        let rel_fname = rel_fname.to_bytes();
        let stem = [&rel_fname[..tail_index(rel_fname)], &name[..]].concat();
        if let Some(found) = try_suffixes(&stem, find_what, suffixes) {
            return Some(found);
        }
    }
    try_suffixes(name, find_what, suffixes)
}

/// Say that `name` is not there, in the wording the caller earned: a first
/// call has not found it at all, a repeat call has run out.
fn report_missing(first: bool, find_what: c_int, name: &CStr) {
    let message = match (first, find_what == FINDFILE_DIR) {
        (true, true) => e_cant_find_directory_str_in_cdpath,
        (true, false) => e_cant_find_file_str_in_path,
        (false, true) => e_no_more_directory_str_found_in_cdpath,
        (false, false) => e_no_more_file_str_found_in_path,
    };
    let name = msg_cstr(name);
    emsg_text(tr_c!(message, name));
}

/// The walk over the entries of `'path'` or `'cdpath'`: the search running
/// over one entry, and where in the option the next entry starts.
#[derive(Default)]
struct OptionWalk {
    /// A copy of the option, and how far into it the walk has got: the walk
    /// outlives the call that started it, and the option's own value may
    /// not.
    entries: Option<(XString, usize)>,
    /// The search over the current entry. It is handed on from entry to
    /// entry, which keeps one entry from answering a file an earlier one
    /// already did.
    search: Option<FindContext>,
    /// Whether `search` is set up for an entry and may be resumed.
    searching: bool,
}

impl OptionWalk {
    /// Start over on `option`, forgetting everything visited.
    fn restart(&mut self, option: XString) {
        if let Some(search) = &mut self.search {
            search.forget_visited();
        }
        self.entries = Some((option, 0));
        self.searching = false;
    }

    /// The next match for `name`, one entry of the option at a time.
    fn next(&mut self, name: &[u8], find_what: c_int, rel_fname: Option<&CStr>) -> Option<XString> {
        loop {
            if self.searching {
                if let Some(found) = self.search.as_mut().and_then(FindContext::next_match) {
                    return Some(found);
                }
                self.searching = false;
                continue;
            }

            let entry = self.entries.as_mut().and_then(|(value, at)| {
                let rest = value.get(*at..).filter(|rest| !rest.is_empty())?;
                let (entry, after) = split_option_part(rest, MAXPATHL as usize, b" ,");
                *at = value.len() - after.len();
                Some(entry)
            });
            let Some(entry) = entry else {
                // We searched all entries of the option, now we can free
                // the search context.
                self.search = None;
                return None;
            };
            // Split the entry at an unescaped ';' into the directory and
            // the stop directories after it.
            let (dir, stopdirs) = split_stop_directories(&entry);
            self.search = FindContext::start(
                self.search.take(),
                &dir,
                name,
                stopdirs,
                FIND_LEVEL,
                find_what,
                false,
                rel_fname,
            );
            self.searching = self.search.is_some();
        }
    }
}

/// One name's walk along a search option, resumed once per match: what
/// `findfile()`, `finddir()`, `gf` and `:find` drive. Dropping it ends the
/// walk.
///
/// Everything a later match is looked for with is the walk's own copy,
/// taken when it starts, so no option value or buffer name is held across
/// the calls in between -- which can run user code. For the same reason
/// every string handed to it must be the caller's own, never a borrow of an
/// option or a buffer: the search itself can reach user code.
#[derive(Default)]
pub(crate) struct PathWalk {
    /// The name looked for, environment variables expanded; `None` until a
    /// start that had a name.
    name: Option<XString>,
    /// `FINDFILE_FILE`, `_DIR` or `_BOTH`.
    find_what: c_int,
    /// The file the search is relative to; never a URL.
    rel_fname: Option<XString>,
    option: OptionWalk,
}

impl PathWalk {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Start looking for `name` along `path_option` (`'path'` or
    /// `'cdpath'`), answering the first match.
    ///
    /// `find_what` is `FINDFILE_FILE`, `_DIR` or `_BOTH`; `rel_fname` the
    /// file the search is relative to, and `suffixes` `'suffixesadd'`. A
    /// walk that was already started starts over.
    pub(crate) fn start(
        &mut self,
        name: &[u8],
        options: FileNameOpts,
        path_option: XString,
        find_what: c_int,
        rel_fname: Option<&CStr>,
        suffixes: &CStr,
    ) -> Option<XString> {
        // Do not attempt to search "relative" to a URL. #6009
        self.rel_fname = rel_fname
            .filter(|rel| path_with_url(rel) == 0)
            .map(XString::from_cstr);
        self.find_what = find_what;
        if name.is_empty() {
            return None;
        }
        let name = self.name.insert(prepare_name(name, options));
        let rel_fname = self.rel_fname.as_ref().map(XString::as_cstr);

        let found = if needs_no_path(name) {
            find_without_path(name, options, find_what, rel_fname, suffixes.to_bytes())
        } else {
            self.option.restart(path_option);
            self.option.next(name, find_what, rel_fname)
        };
        if found.is_none() && options.has(FileNameOpts::MESS) {
            report_missing(true, find_what, name.as_cstr());
        }
        found
    }

    /// The next match of the name the walk started on, or `None` when there
    /// are no more -- reported when `options` asks for it.
    pub(crate) fn next(&mut self, options: FileNameOpts) -> Option<XString> {
        let name = self.name.as_ref()?;
        // A name that needs no path answered its one match on the first
        // call.
        let found = if needs_no_path(name) {
            None
        } else {
            let rel_fname = self.rel_fname.as_ref().map(XString::as_cstr);
            self.option.next(name, self.find_what, rel_fname)
        };
        if found.is_none() && options.has(FileNameOpts::MESS) {
            report_missing(false, self.find_what, name.as_cstr());
        }
        found
    }
}
