//! Which files hold the tags.
//!
//! [`TagFiles`] walks the `'tags'` option one name at a time, expanding
//! wildcards, following `./` relative to the current file and, in a help
//! buffer, visiting every `doc/tags` in `'runtimepath'` instead.
//! [`expand_tag_fname`] is the other half of `'tagrelative'`: it turns a
//! file name a tags file mentions into one the editor can open.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cmdexpand::{WildMode, WildOpts};
use crate::file_search::Name;
use crate::path::tail_index;
use crate::runtime::DIP_ALL;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

/// The `'tags'` and `'helpfile'` names are copied into a buffer this big;
/// anything longer is truncated, as upstream does.
const MAXPATHL: usize = super::MAXPATHL as usize;

/// How deep [`vim_findfile`] may descend under a `'tags'` entry.
const FIND_LEVEL: c_int = 100;

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

/// A [`vim_findfile`] context, freed when the walk ends.
///
/// The same context is handed back to [`vim_findfile_init`] for each new
/// `'tags'` entry, which is what keeps one entry from answering a file an
/// earlier entry already did.
#[derive(Default)]
struct Search {
    ctx: *mut c_void,
    /// Whether `ctx` is set up for an entry and can be stepped.
    open: bool,
}

impl Drop for Search {
    fn drop(&mut self) {
        // SAFETY: `ctx` came from `vim_findfile_init`, or is NULL.
        unsafe { vim_findfile_cleanup(self.ctx) };
    }
}

impl TagFiles {
    /// Start a walk over the tags files for the current buffer.
    pub(crate) fn new() -> Self {
        // SAFETY: `curbuf` is live, and the buffer-local and global
        // `'tags'` are NUL-terminated option strings.
        unsafe {
            let help = (*curbuf.get()).b_help;
            let local = (*curbuf.get()).b_p_tags;
            TagFiles {
                help: help.then(HelpTags::collect),
                tags: (!help)
                    .then(|| Name::from_ptr(if *local != 0 { local } else { p_tags.get() })),
                at: 0,
                search: Search::default(),
            }
        }
    }

    /// The next tags file to read, or `None` when there are no more.
    ///
    /// The name is not checked for existence in a help buffer; elsewhere
    /// it is one [`vim_findfile`] just found.
    pub(crate) fn next(&mut self) -> Option<Name> {
        match &mut self.help {
            Some(help) => help.next(),
            None => self.next_listed(),
        }
    }

    /// The next name from `'tags'`.
    fn next_listed(&mut self) -> Option<Name> {
        // SAFETY: the context came from `vim_findfile_init`, and what
        // `vim_findfile` answers is an allocated NUL-terminated name that
        // becomes ours.
        unsafe {
            loop {
                if !self.search.open {
                    self.open_next_entry()?;
                    continue;
                }
                let found = vim_findfile(self.search.ctx);
                if found.is_null() {
                    self.search.open = false;
                    continue;
                }
                let name = Name::from_ptr(found);
                xfree(found.cast());
                return Some(name);
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

        let mut entry = Entry::split(tags, &mut self.at);
        let file_len = entry.file.len();
        let stop = entry
            .stop
            .as_mut()
            .map_or(ptr::null_mut(), Name::as_mut_ptr);
        // SAFETY: `entry` owns three NUL-terminated names that outlive the
        // call, and the context is either NULL or one this walk made.
        // `vim_findfile_init` takes ownership of it either way.
        self.search.ctx = unsafe {
            vim_findfile_init(
                entry.dir.as_mut_ptr(),
                entry.file.as_mut_ptr(),
                file_len,
                stop,
                FIND_LEVEL,
                // Keep the visited list: an entry must not answer a file
                // an earlier entry already did.
                false,
                FINDFILE_FILE as c_int,
                self.search.ctx,
                true,
                (*curbuf.get()).b_ffname,
            )
        };
        self.search.open = !self.search.ctx.is_null();
        Some(())
    }
}

/// One `'tags'` entry, split into what [`vim_findfile_init`] wants.
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
        // The entry is copied into a fixed buffer first, because
        // `copy_option_part` truncates there and `vim_findfile_stopdir`
        // rewrites in place.
        let mut buf = vec![0 as c_char; MAXPATHL];
        // SAFETY: `at` indexes into `tags`, which is NUL-terminated, and
        // `buf` is `MAXPATHL` writable bytes that both calls terminate.
        unsafe {
            let mut read = tags.as_ptr().add(*at).cast_mut();
            copy_option_part(
                &raw mut read,
                buf.as_mut_ptr(),
                MAXPATHL - 1,
                c" ,".as_ptr().cast_mut(),
            );
            *at = read.offset_from(tags.as_ptr()) as usize;
            let stop = vim_findfile_stopdir(buf.as_mut_ptr());
            let stop = (!stop.is_null()).then(|| Name::from_ptr(stop));

            // What is left in the buffer is the directory to search and,
            // as its last component, the name to look for.
            let entry = CStr::from_ptr(buf.as_ptr()).to_bytes();
            let tail = tail_index(entry);
            Entry {
                dir: Name::from_bytes(&entry[..tail]),
                file: Name::from_bytes(&entry[tail..]),
                stop,
            }
        }
    }
}

impl HelpTags {
    /// Find every `doc/tags` and `doc/tags-??` in `'runtimepath'`.
    fn collect() -> Self {
        let mut found: Vec<Name> = Vec::new();
        // SAFETY: the callback is handed `found` as its cookie and does
        // not outlive this call.
        unsafe {
            do_in_runtimepath(
                c"doc/tags doc/tags-??".as_ptr().cast_mut(),
                DIP_ALL as c_int,
                Some(found_tagfile_cb),
                (&raw mut found).cast(),
            );
        }
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
        // SAFETY: `'helpfile'` is a NUL-terminated option string.
        let hf = unsafe { CStr::from_ptr(p_hf.get()) }.to_bytes();
        if hf.is_empty() {
            return None;
        }
        // Upstream copies it into a `MAXPATHL` buffer with four bytes
        // held back for "tags"; the truncation is kept.
        let hf = &hf[..hf.len().min(MAXPATHL - c"tags".count_bytes() - 1)];
        let mut name = Name::from_bytes(&[&hf[..tail_index(hf)], b"tags".as_slice()].concat());
        simplify(&mut name);
        // Avoid answering a name 'runtimepath' already gave us.
        (!self.found.iter().any(|f| f.bytes() == name.bytes())).then_some(name)
    }
}

/// [`do_in_runtimepath`] callback: collect the `doc/tags` files it found.
///
/// # Safety
/// `cookie` must be the `Vec<Name>` [`HelpTags::collect`] passed in, and
/// `fnames` must hold `num_fnames` NUL-terminated names.
unsafe fn found_tagfile_cb(
    num_fnames: c_int,
    fnames: *mut *mut c_char,
    all: bool,
    cookie: *mut c_void,
) -> bool {
    unsafe {
        let found = &mut *cookie.cast::<Vec<Name>>();
        for i in 0..num_fnames as usize {
            let mut name = Name::from_ptr(*fnames.add(i));
            simplify(&mut name);
            found.push(name);
            if !all {
                break;
            }
        }
        num_fnames > 0
    }
}

/// [`simplify_filename`] over an owned name.
fn simplify(name: &mut Name) {
    // SAFETY: the name is writable and NUL-terminated, and
    // `simplify_filename` only ever shortens it.
    let len = unsafe { simplify_filename(name.as_mut_ptr()) };
    name.truncate(len);
}

/// The name of the file a tags file's entry points at, as the editor can
/// open it.
///
/// With `expand`, wildcards and environment variables in `fname` are
/// expanded first — but never a backtick, which would run a shell command.
/// With `'tagrelative'` set (and always in a help buffer), a relative name
/// is taken relative to the tags file's own directory rather than to the
/// current one.
///
/// The answer is allocated; the caller frees it.
///
/// # Safety
/// Both names must be NUL-terminated. (B11-8: the three callers are still
/// transpiled, which is why this answers a raw pointer.)
pub(crate) unsafe fn expand_tag_fname(
    fname: *mut c_char,
    tag_fname: *mut c_char,
    expand: bool,
) -> *mut c_char {
    unsafe {
        // Expand the file name (for environment variables) when needed.
        // Backticks are disallowed: they could run arbitrary shell
        // commands. This is not needed for tags file names themselves.
        let mut expanded = ptr::null_mut::<c_char>();
        if expand && path_has_wildcard(fname) && vim_strchr(fname, '`' as c_int).is_null() {
            let mut xpc: expand_T = core::mem::zeroed();
            ExpandInit(&raw mut xpc);
            xpc.xp_context = EXPAND_FILES as c_int;
            expanded = ExpandOne(
                &raw mut xpc,
                fname,
                ptr::null_mut(),
                WildOpts::LIST_NOTFOUND | WildOpts::SILENT,
                WildMode::ExpandFree,
            );
        }
        let fname = if expanded.is_null() { fname } else { expanded };

        // The tags file's own directory, empty when it has none.
        let dir = CStr::from_ptr(tag_fname).to_bytes();
        let dir = &dir[..tail_index(dir)];

        let retval = if (p_tr.get() != 0 || (*curbuf.get()).b_help)
            && !vim_isAbsName(fname)
            && !dir.is_empty()
        {
            let name = CStr::from_ptr(fname).to_bytes();
            let mut joined = Name::from_bytes(
                &[dir, &name[..name.len().min(MAXPATHL - dir.len() - 1)]].concat(),
            );
            // Translate names like "src/a/../b/file.c" into "src/b/file.c".
            simplify(&mut joined);
            xstrdup(joined.as_ptr())
        } else {
            xstrdup(fname)
        };

        xfree(expanded.cast());
        retval
    }
}
