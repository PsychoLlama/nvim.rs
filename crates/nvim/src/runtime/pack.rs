//! Packages: `:packadd`, `:packloadall`, and the `pack/*/start` trees loaded at
//! startup.
//!
//! A package is a directory under 'packpath' that gets *added to
//! 'runtimepath'* and then sourced.  [`add_pack_dir_to_rtp`] is the hard half
//! -- it has to insert the package at the right point in the option string,
//! after the last non-`after` entry that is a prefix of it, and insert its own
//! `after/` directory symmetrically at the other end, so that a package's
//! `after/` still runs after everything it should.  [`load_pack_plugin`]
//! sources what the package contains, and the `add_*_pack_plugins` family
//! walks the `start` (loaded at startup) and `opt` (loaded on demand) trees.
//!
//! Adding an *opt* package also splices the two new directories straight into
//! the cached search path rather than paying for a rebuild -- see
//! [`splice_cached_path`], which is why `pos_in_rtp` has to be a byte offset
//! into 'runtimepath' and not merely an ordinal.

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
use crate::option::split_option_part;
use crate::option::vars::{P_PP, P_RTP};
use crate::os::fs::dir_exists;
use crate::path::{ExpandFlags, add_pathsep};

use crate::types::{FAIL, Failed, OK, OptionSetFlags};
use core::ffi::{CStr, c_int};
use std::rc::Rc;

/// Which half of a package's work [`add_pack_plugins`] should do.
///
/// Upstream keeps three `static int`s and hands their *addresses* to
/// `do_in_path` as cookies, purely because three distinct addresses were the
/// cheapest three distinct tokens available. The closures carry the value
/// itself.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PackWork {
    /// `:packadd!` — add the directory to 'runtimepath' and stop there.
    AddDir,
    /// Startup — the directories are already in 'runtimepath'; source them.
    Load,
    /// `:packadd` — both.
    Both,
}

impl PackWork {
    /// Whether the package directory wants adding to 'runtimepath'.
    fn adds_dir(self) -> bool {
        self != Self::Load
    }

    /// Whether the package's plugins want sourcing.
    fn sources(self) -> bool {
        self != Self::AddDir
    }
}

/// The 'runtimepath' prefix a package directory sits under, resolved.
///
/// `fname` is `{rtp}/pack/{name}/{start,opt}/{name}`, so the answer is
/// `{rtp}/` — the four path separators back — run through `fix_fname`, which
/// is the form the entries of 'runtimepath' are compared against.
fn package_root(fname: &CStr) -> XString {
    let name = fname.to_bytes();
    let mut p1 = past_head_index(name);
    let (mut p2, mut p3, mut p4) = (p1, p1, p1);
    let mut p = p1;
    while p < name.len() {
        if vim_ispathsep_nocolon(c_int::from(name[p])) {
            p4 = p3;
            p3 = p2;
            p2 = p1;
            p1 = p;
        }
        p += cluster_len(&name[p..]);
    }
    // Cut *after* the separator, so `fix_fname` is asked about a directory
    // and expands its symlink.
    let cut = (p4 + 1).min(name.len());
    fix_fname(&cstr::owned(&name[..cut]))
}

/// Where in 'runtimepath' a package and its `after/` directory belong, as
/// byte offsets into the copy of the option they were found in.
struct InsertPoints {
    /// The entry to insert before; the option's length when the package
    /// belongs at the end.
    insp: usize,
    /// The first `after/` entry, or `None` when 'runtimepath' has none.
    after_insp: Option<usize>,
}

/// Whether a 'runtimepath' entry is an `after/` directory, the way this
/// search asks it: the first `after` in it is a whole path component.
fn names_an_after_dir(entry: &[u8]) -> bool {
    let Some(p) = entry.windows(5).position(|w| w == b"after") else {
        return false;
    };
    let next = entry.get(p + 5).copied().unwrap_or(0);
    p > 0
        && vim_ispathsep(c_int::from(entry[p - 1]))
        && (vim_ispathsep(c_int::from(next)) || next == 0 || next == b',')
}

/// Find `ffname` in 'runtimepath', ignoring `/` versus `\`, and stop at the
/// first `after/` directory.
///
/// `rtp` is the caller's copy of 'runtimepath': the walk and the splice that
/// follows share one copy rather than each reading the option again.
fn find_insert_points(rtp: &[u8], ffname: &[u8]) -> InsertPoints {
    let mut insp = None;
    let mut after_insp = None;
    let mut rest = rtp;
    while !rest.is_empty() {
        let cur_entry = rtp.len() - rest.len();
        let (mut entry, after) = split_option_part(rest, PATH_ROOM, b",");
        rest = after;
        if names_an_after_dir(&entry) {
            // "ffname" was not found before the first `after/` directory,
            // so it goes in front of this entry.
            insp.get_or_insert(cur_entry);
            after_insp = Some(cur_entry);
            break;
        }
        if insp.is_some() {
            continue;
        }
        add_pathsep(&mut entry);
        let rtp_ffname = cstr::with_terminated(&entry, fix_fname);
        if path_fnamencmp(&rtp_ffname, ffname, ffname.len()) == 0 {
            // Insert after this entry, and its comma.
            insp = Some(rtp.len() - rest.len());
        }
    }
    InsertPoints {
        // Neither "fname" nor an `after/` directory: append at the end.
        insp: insp.unwrap_or(rtp.len()),
        after_insp,
    }
}

/// The new 'runtimepath', and where the two new entries landed in it.
struct Spliced {
    rtp: Vec<u8>,
    /// Byte offset of the package directory.
    first_pos: usize,
    /// Where the `after/` directory's half of the splice sits, as upstream
    /// counts it; zero when there is no `after/`.
    after_pos: usize,
}

/// Build the new 'runtimepath' with `fname` spliced in at `insp` and, when
/// the package has one, `afterdir` at `after_insp`.
///
/// 'runtimepath' is `{keep}{keep_after}{rest}`, and the answer is
/// `{keep},{fname}{keep_after},{afterdir}{rest}` — or, when there was no
/// `after/` entry to sit in front of, `{keep},{fname}{rest},{afterdir}`.
fn splice_rtp(rtp: &[u8], fname: &[u8], afterdir: Option<&[u8]>, points: &InsertPoints) -> Spliced {
    let mut keep = points.insp;
    let mut first_pos = keep;
    let mut new_rtp =
        Vec::with_capacity(rtp.len() + fname.len() + 2 + afterdir.map_or(0, |a| a.len() + 1));
    new_rtp.extend_from_slice(&rtp[..keep]);
    let at_end = keep == rtp.len();
    if at_end {
        // Appending at the end: the comma goes before.
        new_rtp.push(b',');
        first_pos += 1;
    }
    new_rtp.extend_from_slice(fname);
    if !at_end {
        new_rtp.push(b',');
    }

    let mut after_pos = 0;
    if let (Some(afterdir), Some(keep_after)) = (afterdir, points.after_insp) {
        new_rtp.extend_from_slice(&rtp[keep..keep_after]);
        new_rtp.extend_from_slice(afterdir);
        new_rtp.push(b',');
        keep = keep_after;
        after_pos = keep_after;
    }
    new_rtp.extend_from_slice(&rtp[keep..]);

    if let (Some(afterdir), None) = (afterdir, points.after_insp) {
        // No `after/` entry to sit in front of: append it at the end.
        // Upstream's `xstrlcat` answers the length it made, the comma
        // counted.
        after_pos = new_rtp.len() + 1;
        new_rtp.push(b',');
        new_rtp.extend_from_slice(afterdir);
    }

    Spliced {
        rtp: new_rtp,
        first_pos,
        after_pos,
    }
}

/// Where an entry belonging at byte offset `pos` goes in `path`.
///
/// `pos_in_rtp` is monotonic across the path, so this is a partition point;
/// upstream found the same index by walking down from the top, copying each
/// entry two slots up as it went.
fn insert_at(path: &[SearchPathItem], pos: usize) -> usize {
    path.partition_point(|item| item.pos_in_rtp < pos)
}

/// Splice a newly added package into the cached search path.
///
/// Rebuilding the whole path for a `:packadd optpack` is needlessly slow when
/// the answer differs from the old one by two entries in known places.  This
/// is only done for `opt` packages -- a `pack/*/start/*` bundle is added with
/// wildcards still in it, which wants a real expansion -- and only when no
/// walk holds the cache, as upstream's owner slot says.
///
/// `addlen`/`afterlen` are the two new entries' lengths plus one for their
/// comma (`afterlen` zero when there is no `after/`), and `first_pos`/
/// `after_pos` what [`splice_rtp`] answered for them.
fn splice_cached_path(
    fname: &[u8],
    afterdir: &[u8],
    addlen: usize,
    afterlen: usize,
    first_pos: usize,
    after_pos: usize,
) {
    let spliced = RUNTIME_SEARCH_PATH.with_mut(|cached| {
        let Some(path) = cached.as_mut().and_then(Rc::get_mut) else {
            return false;
        };

        // Both indices are found against the path as it stands: the
        // `after/` entry belongs above the other, and the entries between
        // them gain only the first entry's length. With no `after/` entry
        // there is no split, and `after_pos` is not even set.
        let after_at = if afterlen > 0 {
            insert_at(path, after_pos)
        } else {
            path.len()
        };
        let first_at = insert_at(&path[..after_at], first_pos);
        for item in &mut path[after_at..] {
            item.pos_in_rtp += addlen + afterlen;
        }
        for item in &mut path[first_at..after_at] {
            item.pos_in_rtp += addlen;
        }

        // The higher index first, so the lower one still means what it did.
        if afterlen > 0 {
            let mut item =
                SearchPathItem::new(XString::from_bytes(afterdir), true, after_pos + addlen);
            item.pack_inserted = true;
            path.insert(after_at, item);
        }
        let mut item = SearchPathItem::new(XString::from_bytes(fname), false, first_pos);
        item.pack_inserted = true;
        path.insert(first_at, item);
        true
    });
    if spliced {
        SEARCH_PATH_VALID.set(true);
        THREAD_SEARCH_PATH_VALID.set(false);
    }
}

/// Add the package directory `fname` to 'runtimepath'.
///
/// `is_pack` says this is a `pack/*/start/*` bundle rather than a plain
/// directory, which changes how its `after/` is tested for and keeps the
/// cached path from being spliced.
fn add_pack_dir_to_rtp(fname: &CStr, is_pack: bool) -> Result<(), Failed> {
    let ffname = package_root(fname);
    // One copy of 'runtimepath' for the walk and the splice that follows.
    let rtp = P_RTP.get();
    let points = find_insert_points(&rtp, &ffname);

    let afterdir = concat_fnames(fname.to_bytes(), b"after", true);
    // Does `{fname}/after` exist — and, for a bundle, hold anything?
    let has_after = if is_pack {
        pack_has_entries(afterdir.as_cstr())
    } else {
        dir_exists(afterdir.as_cstr())
    };
    let afterlen = if has_after { afterdir.len() + 1 } else { 0 };
    let addlen = fname.count_bytes() + 1; // +1 for the comma

    let spliced = splice_rtp(
        &rtp,
        fname.to_bytes(),
        has_after.then_some(&afterdir[..]),
        &points,
    );
    let was_valid = SEARCH_PATH_VALID.get();
    set_option_value_give_err(
        kOptRuntimepath,
        OptVal::string(String_0::from_bytes(&spliced.rtp)),
        OptionSetFlags::NONE,
    );
    debug_assert!(!SEARCH_PATH_VALID.get(), "!runtime_search_path_valid");
    if was_valid && !is_pack {
        splice_cached_path(
            fname.to_bytes(),
            &afterdir,
            addlen,
            afterlen,
            spliced.first_pos,
            spliced.after_pos,
        );
    }
    Ok(())
}

/// Every plugin file a package holds, under the package's directory.
const PLUGIN_GLOB: &[u8] = b"/plugin/**/*";

/// An opt package's filetype detection, under the same.
const FTDETECT_GLOB: &[u8] = b"/ftdetect/*";

/// Source the scripts in a package's `plugin` directory.
///
/// For `opt` packages the `ftdetect` scripts are sourced too; `start`
/// packages already have those picked up by `filetype.lua`.
fn load_pack_plugin(opt: bool, fname: &CStr) {
    let dir = fix_fname(fname);
    let glob = |suffix: &[u8]| cstr::owned(&[&dir[..], suffix].concat());
    let mut source = |found: &[&CStr], all: bool| source_found_vim_lua(found, all, None);

    let pat = glob(PLUGIN_GLOB);
    let _ = expand_and_visit(&pat, ExpandFlags::FILE, true, &mut source);

    // When runtime/filetype.lua has not been loaded yet, these scripts
    // are found when it is.
    if opt && eval_to_number(b"g:did_load_filetypes", false) > 0 {
        let _ = do_cmdline_cmd(c"augroup filetypedetect");
        let pat = glob(FTDETECT_GLOB);
        let _ = expand_and_visit(&pat, ExpandFlags::FILE, true, &mut source);
        let _ = do_cmdline_cmd(c"augroup END");
    }
}

/// Whether `fname` is already an entry of 'runtimepath'.
fn rtp_has_entry(fname: &CStr) -> bool {
    let rtp = P_RTP.get();
    let mut rest: &[u8] = &rtp;
    while !rest.is_empty() {
        let (entry, after) = split_option_part(rest, PATH_ROOM, b",");
        rest = after;
        if path_fnamecmp(&entry, fname.to_bytes()) == 0 {
            return true;
        }
    }
    false
}

/// Add each package in `fnames` to 'runtimepath' and/or source it, as `work`
/// asks. Answers whether there was anything to work on.
fn add_pack_plugins(opt: bool, fnames: &[&CStr], all: bool, work: PackWork) -> bool {
    let mut did_one = false;

    if work.adds_dir() {
        for &fname in fnames {
            if !rtp_has_entry(fname) && add_pack_dir_to_rtp(fname, false).is_err() {
                return !fnames.is_empty();
            }
            did_one = true;
            if !all {
                break;
            }
        }
    }

    if !all && did_one {
        return !fnames.is_empty();
    }

    if work.sources() {
        for &fname in fnames {
            load_pack_plugin(opt, fname);
            if !all {
                break;
            }
        }
    }
    !fnames.is_empty()
}

/// Add all packages in the `start` directories to 'runtimepath'.
pub fn add_pack_start_dirs() {
    do_in_path_visit(
        P_PP.get(),
        c"packpath",
        c"",
        None,
        RuntimeOpts::ALL | RuntimeOpts::DIR,
        &mut add_pack_start_dir,
    );
}

/// Whether the wildcard pattern `pattern` matches any directory at all.
fn pack_has_entries(pattern: &CStr) -> bool {
    gen_expand_wildcards(&[pattern], ExpandFlags::DIR).is_ok_and(|found| !found.is_empty())
}

/// The two shapes a 'packpath' entry's `start` packages can take.
const START_PATTERNS: [&[u8]; 2] = [b"/start/*", b"/pack/*/start/*"];

/// Add one 'packpath' directory's `start` packages to 'runtimepath'.
fn add_pack_start_dir(fnames: &[&CStr], all: bool) -> bool {
    for &fname in fnames {
        for start_pat in START_PATTERNS {
            if fname.count_bytes() + start_pat.len() + 1 > PATH_ROOM {
                continue;
            }
            let pattern = cstr::owned(&[fname.to_bytes(), start_pat].concat());
            if pack_has_entries(&pattern) {
                let _ = add_pack_dir_to_rtp(&pattern, true);
            }
        }
        if !all {
            break;
        }
    }
    // Upstream's answer, which no caller reads: more than one directory.
    fnames.len() > 1
}

/// Load the plugins of every package in the `start` directories.
pub fn load_start_packages() {
    did_source_packages.set(true);
    for name in [c"pack/*/start/*", c"start/*"] {
        do_in_path_visit(
            P_PP.get(),
            c"packpath",
            c"",
            Some(name),
            RuntimeOpts::ALL | RuntimeOpts::DIR,
            &mut |found, all| add_pack_plugins(false, found, all, PackWork::Load),
        );
    }
    update_runtime_search_path_thread(false);
}

/// `:packloadall[!]`.
pub fn ex_packloadall(excmd: &mut ExArg) {
    if did_source_packages.get() && !excmd.forceit {
        return;
    }
    // One round to add every directory to 'runtimepath', then a second to
    // load the plugins, so a plugin may use another plugin's autoload
    // directory.
    add_pack_start_dirs();
    load_start_packages();
}

/// Read all the plugin files at startup.
pub fn load_plugins() {
    if !p_lpl() {
        return;
    }
    let plugin_pattern = c"plugin/**/*";
    // A copy of 'runtimepath' taken *before* the `start` packages are
    // spliced in, which is what upstream's `rtp_copy` was for.
    let rtp_copy = P_RTP.get();
    if !did_source_packages.get() {
        add_pack_start_dirs();
    }

    // Not `source_runtime_vim_lua` yet, so `:packloadall` can be checked
    // for below.
    source_in_path_vim_lua(
        rtp_copy,
        plugin_pattern,
        RuntimeOpts::ALL | RuntimeOpts::NOAFTER,
    );
    time_msg_at(c"loading rtp plugins");

    // Only source "start" packages when a `:packloadall` has not already.
    if !did_source_packages.get() {
        load_start_packages();
    }
    time_msg_at(c"loading packages");

    let _ = source_runtime_vim_lua(plugin_pattern, RuntimeOpts::ALL | RuntimeOpts::AFTER);
    time_msg_at(c"loading after plugins");
}

/// `:packadd[!] {name}`.
pub fn ex_packadd(excmd: &mut ExArg) {
    // `pack/*/{start,opt}/{name}` — where `:packadd` looks.
    let name = excmd.line.arg().to_vec();
    let under = |which: &[u8]| cstr::owned(&[b"pack/*/", which, b"/", &name[..]].concat());
    let work = if excmd.forceit {
        PackWork::AddDir
    } else {
        PackWork::Both
    };

    // Only look under "start" when loading packages was not done yet.
    let mut res = OK;
    if !did_source_packages.get() {
        let pat = under(b"start");
        res = do_in_path_visit(
            P_PP.get(),
            c"packpath",
            c"",
            Some(&pat),
            RuntimeOpts::ALL | RuntimeOpts::DIR,
            &mut |found, all| add_pack_plugins(false, found, all, work),
        );
    }

    // Give a "not found" error when nothing was found in 'start' or 'opt'.
    let pat = under(b"opt");
    do_in_path_visit(
        P_PP.get(),
        c"packpath",
        c"",
        Some(&pat),
        RuntimeOpts::ALL | RuntimeOpts::DIR | RuntimeOpts::ERR.when(res == FAIL),
        &mut |found, all| add_pack_plugins(true, found, all, work),
    );

    update_runtime_search_path_thread(false);
}
