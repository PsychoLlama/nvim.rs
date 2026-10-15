//! The precomputed 'runtimepath' search path, and the thread-safe copy of it.
//!
//! Walking 'runtimepath' *and* 'packpath' for every `:runtime` is O(entries)
//! filesystem calls, so the resolved, ordered, deduplicated list of directories
//! is built once ([`runtime_search_path_build`]) and reused until either option
//! changes.  `after/` directories sort last and `pack/*/start` entries are
//! spliced in where the pack was found, which is the ordering the whole plugin
//! ecosystem depends on and which nothing else in the tree recomputes.
//!
//! There are two copies.  The main one is invalidated by
//! [`did_set_runtimepackpath`] and rebuilt lazily by
//! [`runtime_search_path_validate`]; the second is a snapshot behind a mutex
//! for the *thread* that serves `nvim_get_runtime_file` off the main loop,
//! refreshed by [`update_runtime_search_path_thread`].
//!
//! # Sharing
//!
//! The cache is an [`Rc`]: a walk takes a clone ([`runtime_search_path_get_cached`])
//! and holds it for as long as it reads. A rebuild while somebody walks only
//! replaces the cell's handle, so the walker keeps the old path and frees it
//! when it lets go -- which is what lets `do_in_cached_path` source files that
//! themselves change 'runtimepath'. Upstream spells the same thing as a
//! single owner slot holding the address of the first borrower's local.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cstr;
use crate::memory::XString;
use crate::message_fmt::msg_cstr;
use crate::option::split_option_part;
use crate::option::vars::P_PP;
use crate::option::vars::P_RTP;
use crate::path::{ExpandFlags, add_pathsep};
use crate::semsg;
use crate::smsg;

use crate::types::{FAIL, OK, OptError};
use core::ffi::{CStr, c_int};
use std::rc::Rc;
use std::sync::PoisonError;

/// Borrow the cached search path, validating it first.
///
/// The clone *is* the borrow: a rebuild while it is held leaves this copy to
/// the caller, which frees it on drop.
pub(crate) fn runtime_search_path_get_cached() -> Rc<Vec<SearchPathItem>> {
    runtime_search_path_validate();
    RUNTIME_SEARCH_PATH
        .with(|path| path.clone())
        .unwrap_or_default()
}

/// The cached search path as it stands, without validating it -- what
/// `nvim__runtime_inspect()` reports.
pub(crate) fn runtime_search_path_as_is() -> Rc<Vec<SearchPathItem>> {
    RUNTIME_SEARCH_PATH
        .with(|path| path.clone())
        .unwrap_or_default()
}

/// Find the file `name` in the cached search path and hand `visit` each
/// match.  `name` may contain wildcards; `None` hands over every directory.
///
/// `RuntimeOpts::ALL` visits every match, `RuntimeOpts::DIR` looks for
/// directories, `RuntimeOpts::ERR` turns "nothing found" into an error
/// message.  Answers OK when something was found -- which, as upstream, a
/// walk over the directories alone never says.
pub(crate) fn do_in_cached_path(
    name: Option<&CStr>,
    flags: RuntimeOpts,
    visit: &mut Visit,
) -> c_int {
    if p_verbose() > 10
        && let Some(name) = name
    {
        verbose_enter();
        let name = msg_cstr(name);
        smsg!(0, "Searching for \"{name}\" in runtime path");
        verbose_leave();
    }

    let path = runtime_search_path_get_cached();
    let do_all = flags.has(RuntimeOpts::ALL);
    let ew_flags = wildcard_flags(flags) | ExpandFlags::NOBREAK;
    let mut did_one = false;
    // One buffer for the whole walk, as upstream's stack buffer.
    let mut buf = Vec::with_capacity(PATH_ROOM);

    for item in path.iter() {
        if skips_entry(flags, item.after) {
            continue;
        }
        let Some(name) = name else {
            visit(&[item.path.as_cstr()], do_all);
            continue;
        };
        if item.path.len() + name.count_bytes() + 2 >= PATH_ROOM {
            continue;
        }
        buf.clear();
        buf.extend_from_slice(&item.path);
        // The length test above proves the separator fits.
        add_pathsep(&mut buf);
        expand_name_patterns(
            &mut buf,
            name.to_bytes(),
            ew_flags,
            do_all,
            &mut did_one,
            visit,
        );
    }

    if !did_one && let Some(name) = name {
        let name = msg_cstr(name);
        if flags.has(RuntimeOpts::ERR) {
            semsg!(
                "E919: Directory not found in '{}': \"{name}\"",
                "runtime path"
            );
        } else if p_verbose() > 1 {
            verbose_enter();
            smsg!(0, "not found in runtime path: \"{name}\"");
            verbose_leave();
        }
    }

    if did_one { OK } else { FAIL }
}

/// The directories already in the search path, by their bytes: the dedup
/// `runtime_search_path_build` walks with. Membership only.
type PathSet = IdSet<Box<[u8]>>;

/// Add `entry` to the search path unless it is already there.
fn push_path(
    search_path: &mut Vec<SearchPathItem>,
    rtp_used: &mut PathSet,
    entry: &[u8],
    after: bool,
    pos_in_rtp: usize,
) {
    if rtp_used.insert(entry.into()) {
        search_path.push(SearchPathItem::new(
            XString::from_bytes(entry),
            after,
            pos_in_rtp,
        ));
    }
}

/// Expand one 'runtimepath' entry — which may hold wildcards — and add every
/// directory it names to the search path.
fn expand_rtp_entry(
    search_path: &mut Vec<SearchPathItem>,
    rtp_used: &mut PathSet,
    entry: &[u8],
    after: bool,
    pos_in_rtp: usize,
) {
    if rtp_used.contains(entry) {
        return;
    }
    if entry.is_empty() {
        push_path(search_path, rtp_used, entry, after, pos_in_rtp);
    }
    let found = cstr::with_terminated(entry, |pattern| {
        gen_expand_wildcards(&[pattern], ExpandFlags::DIR | ExpandFlags::NOBREAK)
    });
    for name in found.iter().flatten() {
        // Reusing the position is fine: it only has to be monotonic, not
        // strictly increasing.
        push_path(search_path, rtp_used, name, after, pos_in_rtp);
    }
}

/// The two shapes a 'packpath' entry's `start` packages can take.
const START_PATTERNS: [&[u8]; 2] = [b"/pack/*/start/*", b"/start/*"];

/// Add one 'packpath' entry's `start` packages to the search path, and queue
/// their `after/` directories for the end of the build.
fn expand_pack_entry(
    search_path: &mut Vec<SearchPathItem>,
    rtp_used: &mut PathSet,
    after_path: &mut Vec<Vec<u8>>,
    pack_entry: &[u8],
    pos_in_rtp: usize,
) {
    for start_pat in START_PATTERNS {
        if pack_entry.len() + start_pat.len() + 1 > PATH_ROOM {
            continue;
        }
        let pattern = [pack_entry, start_pat].concat();
        expand_rtp_entry(search_path, rtp_used, &pattern, false, pos_in_rtp);

        // The `after/` directories go in one block at the end of the
        // build, so they sort behind every non-`after` entry.
        after_path.push([&pattern[..], b"/after"].concat());
    }
}

/// Whether `dir` names an `after/` directory.
///
/// Only a component spelled exactly `after` counts.  Vim 8 treats `foo/bar_after`
/// and `Xafter` as `after` directories in some code paths but not all; this is
/// the strict reading.
pub(crate) fn path_is_after(dir: &[u8]) -> bool {
    let len = dir.len();
    len >= 5 && (len < 6 || vim_ispathsep(c_int::from(dir[len - 6]))) && dir.ends_with(b"after")
}

/// Build the ordered search path from 'runtimepath' and 'packpath'.
///
/// The order is what the whole plugin ecosystem depends on:
///
/// 1. every non-`after` 'runtimepath' entry, in order, with the `start`
///    packages of any 'packpath' entry that the entry also names spliced in
///    right behind it;
/// 2. the `start` packages of every 'packpath' entry 'runtimepath' did *not*
///    name;
/// 3. every `after/` directory of those packages;
/// 4. the `after/` tail of 'runtimepath'.
///
/// `pos_in_rtp` is the byte offset of the entry in 'runtimepath' that put a
/// directory here; entries that were not spelled in 'runtimepath' share the
/// offset of the comma before its `after/` tail, which keeps the sequence
/// monotonic. [`add_pack_dir_to_rtp`] splices new entries by comparing it.
fn runtime_search_path_build() -> Vec<SearchPathItem> {
    let mut pack_entries: Vec<&[u8]> = Vec::new();
    let mut pack_used: IdMap<Box<[u8]>, c_int> = id_map();
    let mut rtp_used: PathSet = id_set();
    let mut search_path: Vec<SearchPathItem> = Vec::new();
    let mut after_path: Vec<Vec<u8>> = Vec::new();

    // 'packpath' first, only to record which entries exist: they are matched
    // against 'runtimepath' below, and whatever is left over is appended.
    // As upstream, an entry is recorded as the option's own bytes, as many
    // as the separated part has -- the same text unless it escaped a comma.
    let pp = P_PP.get();
    let mut rest: &[u8] = &pp;
    while !rest.is_empty() {
        let (part, after) = split_option_part(rest, PATH_ROOM, b",");
        let the_entry = &rest[..part.len()];
        rest = after;
        pack_used.insert(the_entry.into(), 0);
        pack_entries.push(the_entry);
    }

    // 'runtimepath' up to its first `after/` entry.
    let rtp = P_RTP.get();
    let rtp: &[u8] = &rtp;
    let mut rest = rtp;
    while !rest.is_empty() {
        let pos_in_rtp = rtp.len() - rest.len();
        let (entry, after) = split_option_part(rest, PATH_ROOM, b",");
        if path_is_after(&entry) {
            // Leave it for the tail loop below, entry and all.
            break;
        }
        rest = after;
        // Fact: 'runtimepath' entries can contain wildcards.
        expand_rtp_entry(&mut search_path, &mut rtp_used, &entry, false, pos_in_rtp);
        if let Some(seen) = pack_used.get_mut(&entry[..]) {
            // Mark this 'packpath' entry as already covered.
            *seen += 1;
            expand_pack_entry(
                &mut search_path,
                &mut rtp_used,
                &mut after_path,
                &entry,
                pos_in_rtp,
            );
        }
    }

    // What follows was not spelled in 'runtimepath'.  Keeping `pos_in_rtp`
    // monotonic means giving it the comma between the two halves.
    let mut sentinel_pos_in_rtp = rtp.len() - rest.len();
    sentinel_pos_in_rtp -= usize::from(sentinel_pos_in_rtp > 0);

    for &item in &pack_entries {
        if pack_used.get(item).copied().unwrap_or(0) == 0 {
            expand_pack_entry(
                &mut search_path,
                &mut rtp_used,
                &mut after_path,
                item,
                sentinel_pos_in_rtp,
            );
        }
    }

    // The packages' `after/` directories.
    for dir in &after_path {
        expand_rtp_entry(
            &mut search_path,
            &mut rtp_used,
            dir,
            true,
            sentinel_pos_in_rtp,
        );
    }

    // The `after/` tail of 'runtimepath'.
    while !rest.is_empty() {
        let pos_in_rtp = rtp.len() - rest.len();
        let (entry, after) = split_option_part(rest, PATH_ROOM, b",");
        rest = after;
        let is_after = path_is_after(&entry);
        expand_rtp_entry(
            &mut search_path,
            &mut rtp_used,
            &entry,
            is_after,
            pos_in_rtp,
        );
    }

    search_path
}

/// `'runtimepath'`/`'packpath'` changed: the cache no longer describes them.
///
/// Never inlined: the option table holds it by address, which makes a
/// call-free leaf shippable cross-crate and would export the cell it sets.
#[inline(never)]
pub(crate) fn did_set_runtimepackpath(_args: &mut OptSet) -> Result<(), OptError> {
    SEARCH_PATH_VALID.set(false);
    Ok(())
}

/// Rebuild the cached search path if it has been invalidated.
pub(crate) fn runtime_search_path_validate() {
    // The path cannot be rebuilt in an async context. A plugin will invoke
    // itself asynchronously from sync code in the same plugin, so the lua or
    // autoload module it is looking for is almost certainly in the cached path
    // already: a stale cache beats an error here.
    if !nlua_is_deferred_safe() || SEARCH_PATH_VALID.get() {
        return;
    }
    // Nobody walking the old path: it goes with the rebuild, and the UI is
    // flushed first so that its callbacks cannot recurse into a half-freed
    // cache.
    if RUNTIME_SEARCH_PATH.with(|path| path.as_ref().is_none_or(|path| Rc::strong_count(path) == 1))
    {
        msg_ext_ui_flush();
    }
    let built = Rc::new(runtime_search_path_build());
    let old = RUNTIME_SEARCH_PATH.with_mut(|path| path.replace(built));
    drop(old);
    SEARCH_PATH_VALID.set(true);
    update_runtime_search_path_thread(true);
}

/// Refresh the snapshot the worker threads read.
///
/// Without `force` this is a no-op unless the main cache is valid and the
/// snapshot is not — that is, it is the cheap "catch up if you are behind"
/// call the `:packadd` family makes.
pub(crate) fn update_runtime_search_path_thread(force: bool) {
    if !force && !(SEARCH_PATH_VALID.get() && !THREAD_SEARCH_PATH_VALID.get()) {
        return;
    }
    let copy = Vec::clone(&runtime_search_path_as_is());
    let old = {
        let mut snapshot = THREAD_SEARCH_PATH
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        core::mem::replace(&mut *snapshot, copy)
    };
    // Freed outside the lock.
    drop(old);
    THREAD_SEARCH_PATH_VALID.set(true);
}
