//! Which files hold the tags.
//!
//! [`TagFiles`] walks the `'tags'` option one name at a time, expanding
//! wildcards, following `./` relative to the current file and, in a help
//! buffer, visiting every `doc/tags` in `'runtimepath'` instead.
//! [`expand_tag_fname`] is the other half of `'tagrelative'`: it turns a
//! file name a tags file mentions into one the editor can open.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cmdexpand::{WildMode, WildOpts};
use crate::cstr;
use crate::file_search::{FIND_LEVEL, Name};
use crate::option::split_option_part;
use crate::option::vars::{P_HF, P_TAGS};
use crate::optionstr::{OptString, local_or_global};
use crate::path::tail_index;
use crate::runtime::{RuntimeOpts, do_in_runtimepath_visit};
use crate::types::{ExpandContext, MAXPATHL};
use crate::winlayer::Buf;
use core::ffi::CStr;

/// The tags files that apply to the current buffer, in the order they are
/// searched.
///
/// A help buffer takes its tags from every `doc/tags` and `doc/tags-??` in
/// `'runtimepath'` — a completely different list — so which half of this
/// runs is decided once, when the walk starts.
///
/// (Upstream re-reads `curbuf->b_help` on every call but only initialises
/// on the first, so a buffer that stopped being a help buffer part-way
/// through a walk would dereference a NULL `'tags'` copy. Deciding once
/// is the same behaviour without the crash.)
pub(crate) struct TagFiles {
    /// The help tags files found in `'runtimepath'`, and how many of them
    /// have been answered. `None` outside a help buffer.
    help: Option<HelpTags>,
    /// A private copy of `'tags'`: autocommands may change the option
    /// without telling us. `None` in a help buffer.
    tags: Option<Name>,
    /// How far into that copy the walk has got.
    at: usize,
    /// The file search running over the current `'tags'` entry.
    search: Search,
}

/// The help tags files, once `'runtimepath'` has been walked for them.
struct HelpTags {
    found: Vec<Name>,
    /// How many have been answered. One past the end means the
    /// `'helpfile'` fallback has been answered too.
    at: usize,
}

/// A [`FindContext`], dropped when the walk ends.
///
/// The same context is handed back to [`FindContext::start`] for each new
/// `'tags'` entry, which is what keeps one entry from answering a file an
/// earlier entry already did.
#[derive(Default)]
struct Search {
    ctx: Option<FindContext>,
    /// Whether `ctx` is set up for an entry and can be stepped.
    open: bool,
}

impl TagFiles {
    /// Start a walk over the tags files for the current buffer.
    pub(crate) fn new() -> Self {
        // SAFETY: `curbuf` is live, and the buffer-local and global
        // `'tags'` are NUL-terminated option strings.
        let help = Buf::current().b_help;
        let tags = (!help).then(|| local_or_global(&Buf::current().b_p_tags, P_TAGS).get());
        TagFiles {
            help: help.then(HelpTags::collect),
            tags: tags.map(|tags| Name::from_bytes(&tags)),
            at: 0,
            search: Search::default(),
        }
    }

    /// The next tags file to read, or `None` when there are no more.
    ///
    /// The name is not checked for existence in a help buffer; elsewhere
    /// it is one [`FindContext::next_match`] just found.
    pub(crate) fn next(&mut self) -> Option<Name> {
        match &mut self.help {
            Some(help) => help.next(),
            None => self.next_listed(),
        }
    }

    /// The next name from `'tags'`.
    fn next_listed(&mut self) -> Option<Name> {
        loop {
            if !self.search.open {
                self.open_next_entry()?;
                continue;
            }
            match self.search.ctx.as_mut().and_then(FindContext::next_match) {
                Some(found) => return Some(Name::from_bytes(&found)),
                None => self.search.open = false,
            }
        }
    }

    /// Set the search up for the next `'tags'` entry, answering `None`
    /// when every entry has been used.
    fn open_next_entry(&mut self) -> Option<()> {
        let tags = self.tags.as_mut()?;
        if tags.at(self.at) == 0 {
            // Every part of 'tags' has been used. Free the context now
            // rather than at the end of the walk, as upstream does.
            self.search = Search::default();
            return None;
        }

        let entry = Entry::split(tags, &mut self.at);
        let buffer = Buf::current();
        self.search.ctx = FindContext::start(
            self.search.ctx.take(),
            entry.dir.bytes(),
            entry.file.bytes(),
            entry.stop.as_ref().map(Name::bytes),
            FIND_LEVEL,
            FINDFILE_FILE.cast_signed(),
            true,
            buffer.name.full(),
        );
        self.search.open = self.search.ctx.is_some();
        Some(())
    }
}

/// One `'tags'` entry, split into what [`FindContext::start`] wants.
struct Entry {
    /// Where to start looking, wildcards and all.
    dir: Name,
    /// The name of the file to look for.
    file: Name,
    /// Where an upward search stops, from the entry's `;` on.
    stop: Option<Name>,
}

impl Entry {
    /// Take the next entry out of `tags`, advancing `at` past it.
    fn split(tags: &Name, at: &mut usize) -> Self {
        // Cut where upstream's `copy_option_part` into a `MAXPATHL` buffer
        // cuts it.
        let rest = &tags.bytes()[*at..];
        let (entry, after) = split_option_part(rest, MAXPATHL as usize - 1, b" ,");
        *at = tags.len() - after.len();
        let (entry, stop) = split_stop_directories(&entry);

        // What is left is the directory to search and, as its last
        // component, the name to look for.
        let tail = tail_index(&entry);
        Entry {
            dir: Name::from_bytes(&entry[..tail]),
            file: Name::from_bytes(&entry[tail..]),
            stop: stop.map(Name::from_bytes),
        }
    }
}

impl HelpTags {
    /// Find every `doc/tags` and `doc/tags-??` in `'runtimepath'`.
    fn collect() -> Self {
        let mut found: Vec<Name> = Vec::new();
        let _ = do_in_runtimepath_visit(
            c"doc/tags doc/tags-??",
            RuntimeOpts::ALL,
            &mut |fnames: &[&CStr], all: bool| {
                for name in fnames {
                    let mut name = Name::from_bytes(name.to_bytes());
                    name.simplify();
                    found.push(name);
                    if !all {
                        break;
                    }
                }
                !fnames.is_empty()
            },
        );
        HelpTags { found, at: 0 }
    }

    fn next(&mut self) -> Option<Name> {
        if let Some(name) = self.found.get(self.at) {
            self.at += 1;
            return Some(name.clone());
        }
        // Nothing more in 'runtimepath': use 'helpfile' if it exists and
        // has not been used yet, with "help.txt" replaced by "tags".
        if self.at > self.found.len() {
            return None;
        }
        self.at += 1;
        // A copy: the name is built out of it below.
        let helpfile = P_HF.get();
        let hf = &*helpfile;
        if hf.is_empty() {
            return None;
        }
        // Upstream copies it into a `MAXPATHL` buffer with four bytes
        // held back for "tags"; the truncation is kept.
        let hf = &hf[..hf.len().min(MAXPATHL as usize - c"tags".count_bytes() - 1)];
        let mut name = Name::from_bytes(&[&hf[..tail_index(hf)], b"tags".as_slice()].concat());
        name.simplify();
        // Avoid answering a name 'runtimepath' already gave us.
        (!self.found.iter().any(|f| f.bytes() == name.bytes())).then_some(name)
    }
}

/// The name of the file a tags file's entry points at, as the editor can
/// open it.
///
/// With `expand`, wildcards and environment variables in `fname` are
/// expanded first — but never a backtick, which would run a shell command.
/// With `'tagrelative'` set (and always in a help buffer), a relative name
/// is taken relative to the tags file's own directory rather than to the
/// current one.
pub(crate) fn expand_tag_fname(fname: &[u8], tag_fname: &[u8], expand: bool) -> XString {
    // Expand the file name (for environment variables) when needed.
    // Backticks are disallowed: they could run arbitrary shell
    // commands. This is not needed for tags file names themselves.
    let expanded = if expand && has_wildcard(fname) && !fname.contains(&b'`') {
        let mut xpc = Expand::new();
        xpc.context = ExpandContext::Files;
        let options = WildOpts::LIST_NOTFOUND | WildOpts::SILENT;
        let pattern = cstr::owned(fname);
        expand_one(
            &mut xpc,
            Some(&pattern),
            None,
            options,
            WildMode::ExpandFree,
        )
    } else {
        None
    };
    let fname = expanded.as_deref().unwrap_or(fname);

    // The tags file's own directory, empty when it has none.
    let dir = &tag_fname[..tail_index(tag_fname)];

    if (p_tr() || Buf::current().b_help)
        && !cstr::with_terminated(fname, vim_is_abs_name)
        && !dir.is_empty()
    {
        let mut joined = Name::from_bytes(
            &[
                dir,
                &fname[..fname.len().min(MAXPATHL as usize - dir.len() - 1)],
            ]
            .concat(),
        );
        // Translate names like "src/a/../b/file.c" into "src/b/file.c".
        joined.simplify();
        XString::from_bytes(joined.bytes())
    } else {
        XString::from_bytes(fname)
    }
}
