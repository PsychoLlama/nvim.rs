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

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::path::ExpandFlags;

use crate::types::{FAIL, Failed, MAXPATHL, OK, OptionSetFlags};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

/// Which half of a package's work [`add_pack_plugins`] should do.
///
/// Upstream keeps three `static int`s and hands their *addresses* to
/// `do_in_path` as cookies, purely because three distinct addresses were the
/// cheapest three distinct tokens available.  Three small non-zero integers
/// are the same tokens without the globals; the value never leaves this file.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
enum PackWork {
    /// `:packadd!` — add the directory to 'runtimepath' and stop there.
    AddDir = 1,
    /// Startup — the directories are already in 'runtimepath'; source them.
    Load = 2,
    /// `:packadd` — both.
    Both = 3,
}

impl PackWork {
    /// This token as a `do_in_path` cookie.
    fn cookie(self) -> *mut c_void {
        ptr::without_provenance_mut(self as usize)
    }

    /// The token a cookie carries, if it is one of ours.
    fn from_cookie(cookie: *mut c_void) -> Option<Self> {
        match cookie.addr() {
            1 => Some(Self::AddDir),
            2 => Some(Self::Load),
            3 => Some(Self::Both),
            _ => None,
        }
    }

    /// Whether the package directory wants adding to 'runtimepath'.
    ///
    /// Upstream tests `cookie != &APP_LOAD`, so anything that is not `Load` —
    /// a cookie from outside this file included — means yes.
    fn adds_dir(work: Option<Self>) -> bool {
        work != Some(Self::Load)
    }

    /// Whether the package's plugins want sourcing.  As [`Self::adds_dir`].
    fn sources(work: Option<Self>) -> bool {
        work != Some(Self::AddDir)
    }
}

/// The 'runtimepath' prefix a package directory sits under, resolved.
///
/// `fname` is `{rtp}/pack/{name}/{start,opt}/{name}`, so the answer is
/// `{rtp}/` — the four path separators back — run through `fix_fname`, which
/// is the form the entries of 'runtimepath' are compared against.  Answers
/// null when the path cannot be resolved.
///
/// # Safety
/// `fname` must be a writable NUL-terminated path.
unsafe fn package_root(fname: *mut c_char) -> *mut c_char {
    // SAFETY: `fname` is a NUL-terminated path this walk stays inside.
    let mut p1 = unsafe { get_past_head(fname) };
    let (mut p2, mut p3, mut p4) = (p1, p1, p1);
    let mut p = p1;
    while unsafe { *p } != 0 {
        if vim_ispathsep_nocolon(unsafe { *p } as c_int) {
            p4 = p3;
            p3 = p2;
            p2 = p1;
            p1 = p;
        }
        p = unsafe { p.add(utfc_ptr2len(p) as usize) };
    }
    // Cut *after* the separator, so `fix_fname` is asked about a directory
    // and expands its symlink.
    let cut = unsafe { p4.add(1) };
    let c = unsafe { *cut };
    unsafe { *cut = 0 };
    let ffname = unsafe { fix_fname(fname) };
    unsafe { *cut = c };
    ffname
}

/// Where in 'runtimepath' a package and its `after/` directory belong.
///
/// Both are pointers *into* 'runtimepath' — the option is rebuilt around
/// them, so they are only valid until it is written.
struct InsertPoints {
    /// The entry to insert before; the option's terminator when the package
    /// belongs at the end.
    insp: *const c_char,
    /// The first `after/` entry, or null when 'runtimepath' has none.
    after_insp: *const c_char,
}

/// Find `ffname` in 'runtimepath', ignoring `/` versus `\`, and stop at the
/// first `after/` directory.
///
/// Answers `None` when an entry could not be resolved, which upstream treats
/// as a failure of the whole operation.
///
/// # Safety
/// `ffname` must be NUL-terminated and `fname_len` its length.
unsafe fn find_insert_points(ffname: *const c_char, fname_len: size_t) -> Option<InsertPoints> {
    let mut buf = [0 as c_char; MAXPATHL as usize];
    let mut insp: *const c_char = ptr::null();
    let mut after_insp: *const c_char = ptr::null();
    let mut entry: *const c_char = p_rtp.get();

    // SAFETY: `entry` walks 'runtimepath' and `buf` has `MAXPATHL` bytes.
    while unsafe { *entry } != 0 {
        let cur_entry = entry;
        // SAFETY: as above.
        unsafe {
            copy_option_part(
                &raw mut entry as *mut *mut c_char,
                buf.as_mut_ptr(),
                MAXPATHL as size_t,
                c",".as_ptr().cast_mut(),
            )
        };
        // SAFETY: `buf` is NUL-terminated; the `p[5]` reads are inside it
        // because `strstr` found five bytes of "after" there.
        let is_after = unsafe {
            let p = strstr(buf.as_mut_ptr(), c"after".as_ptr());
            !p.is_null()
                && p > buf.as_mut_ptr()
                && vim_ispathsep(*p.sub(1) as c_int)
                && (vim_ispathsep(*p.add(5) as c_int)
                    || *p.add(5) == 0
                    || *p.add(5) == b',' as c_char)
        };
        if is_after {
            if insp.is_null() {
                // "ffname" was not found before the first `after/` directory,
                // so it goes in front of this entry.
                insp = cur_entry;
            }
            after_insp = cur_entry;
            break;
        }
        if !insp.is_null() {
            continue;
        }
        // SAFETY: `buf` has room for the separator (`copy_option_part` stopped
        // short of `MAXPATHL`), and `fix_fname` answers an owned string.
        unsafe { add_pathsep(buf.as_mut_ptr()) };
        let rtp_ffname = unsafe { fix_fname(buf.as_mut_ptr()) };
        if rtp_ffname.is_null() {
            return None;
        }
        if unsafe { path_fnamencmp(rtp_ffname, ffname, fname_len) } == 0 {
            // Insert after this entry, and its comma.
            insp = entry;
        }
        unsafe { xfree(rtp_ffname.cast()) };
    }

    if insp.is_null() {
        // Neither "fname" nor an `after/` directory: append at the end.
        // SAFETY: 'runtimepath' is NUL-terminated.
        insp = unsafe { p_rtp.get().add(cstr::bytes_at(p_rtp.get()).len()) };
    }
    Some(InsertPoints { insp, after_insp })
}

/// The new 'runtimepath', and where the two new entries landed in it.
struct Spliced {
    rtp: *mut c_char,
    /// Byte offset of the package directory.
    first_pos: size_t,
    /// Byte offset of the comma before its `after/` directory; zero when
    /// there is no `after/`.
    after_pos: size_t,
}

/// Build the new 'runtimepath' with `fname` spliced in at `insp` and
/// `afterdir` at `after_insp`.
///
/// 'runtimepath' is `{keep}{keep_after}{rest}`, and the answer is
/// `{keep},{fname}{keep_after},{afterdir}{rest}` — or, when there was no
/// `after/` entry to sit in front of, `{keep},{fname}{rest},{afterdir}`.
///
/// Answers `None` when the allocation failed.
///
/// # Safety
/// `points` must still describe the current 'runtimepath'; `fname` and
/// `afterdir` must be NUL-terminated, and `addlen`/`afterlen` their lengths
/// plus one for the comma (`afterlen` is zero when there is no `after/`).
unsafe fn splice_rtp(
    fname: *mut c_char,
    afterdir: *mut c_char,
    addlen: size_t,
    afterlen: size_t,
    points: &InsertPoints,
) -> Option<Spliced> {
    // SAFETY: 'runtimepath' is NUL-terminated.
    let oldlen = unsafe { cstr::bytes_at(p_rtp.get()) }.len();
    let capacity = oldlen + addlen + afterlen + 1; // +1 for the NUL
    // SAFETY: `try_malloc` answers null rather than aborting.
    let new_rtp = unsafe { try_malloc(capacity) }.cast::<c_char>();
    if new_rtp.is_null() {
        return None;
    }

    // SAFETY: every write below stays inside `capacity`, which was sized from
    // the same three lengths. `insp` points into 'runtimepath'.
    let mut keep = unsafe { points.insp.offset_from(p_rtp.get()) } as size_t;
    let mut first_pos = keep;
    unsafe { new_rtp.cast::<u8>().copy_from(p_rtp.get().cast(), keep) };
    let mut len = keep;
    if unsafe { *points.insp } == 0 {
        // Appending at the end: the comma goes before.
        unsafe { *new_rtp.add(len) = b',' as c_char };
        len += 1;
        first_pos += 1;
    }
    unsafe {
        new_rtp
            .add(len)
            .cast::<u8>()
            .copy_from(fname.cast(), addlen - 1)
    };
    len += addlen - 1;
    if unsafe { *points.insp } != 0 {
        unsafe { *new_rtp.add(len) = b',' as c_char };
        len += 1;
    }

    let mut after_pos = 0;
    if afterlen > 0 && !points.after_insp.is_null() {
        let keep_after = unsafe { points.after_insp.offset_from(p_rtp.get()) } as size_t;
        unsafe {
            new_rtp
                .add(len)
                .cast::<u8>()
                .copy_from(p_rtp.get().add(keep).cast(), keep_after - keep)
        };
        len += keep_after - keep;
        unsafe {
            new_rtp
                .add(len)
                .cast::<u8>()
                .copy_from(afterdir.cast(), afterlen - 1)
        };
        len += afterlen - 1;
        unsafe { *new_rtp.add(len) = b',' as c_char };
        len += 1;
        keep = keep_after;
        after_pos = keep_after;
    }

    if unsafe { *p_rtp.get().add(keep) } != 0 {
        unsafe {
            new_rtp
                .add(len)
                .cast::<u8>()
                .copy_from(p_rtp.get().add(keep).cast(), oldlen - keep + 1)
        };
    } else {
        unsafe { *new_rtp.add(len) = 0 };
    }

    if afterlen > 0 && points.after_insp.is_null() {
        // No `after/` entry to sit in front of: append it at the end.
        after_pos = unsafe { xstrlcat(new_rtp, c",".as_ptr(), capacity) };
        unsafe { xstrlcat(new_rtp, afterdir, capacity) };
    }

    Some(Spliced {
        rtp: new_rtp,
        first_pos,
        after_pos,
    })
}

/// Where an entry belonging at byte offset `pos` goes in `path`.
///
/// `pos_in_rtp` is monotonic across the path, so this is a partition point;
/// upstream found the same index by walking down from the top, copying each
/// entry two slots up as it went.
fn insert_at(path: &[SearchPathItem], pos: size_t) -> usize {
    path.partition_point(|item| item.pos_in_rtp < pos)
}

/// Splice a newly added package into the cached search path.
///
/// Rebuilding the whole path for a `:packadd optpack` is needlessly slow when
/// the answer differs from the old one by two entries in known places.  This
/// is only done for `opt` packages: a `pack/*/start/*` bundle is added with
/// wildcards still in it, which wants a real expansion.
///
/// # Safety
/// `fname` and `afterdir` must be NUL-terminated; `first_pos`/`after_pos` and
/// `addlen`/`afterlen` must be what [`splice_rtp`] answered for them.
unsafe fn splice_cached_path(
    fname: *mut c_char,
    afterdir: *mut c_char,
    addlen: size_t,
    afterlen: size_t,
    first_pos: size_t,
    after_pos: size_t,
) {
    runtime_search_path_valid.set(true);
    runtime_search_path_valid_thread.set(false);
    runtime_search_path.with_mut(|cached| {
        // SAFETY: the cache, taken back for the two insertions and handed
        // straight back. Nothing here reenters the cell, and the borrow slot
        // in `runtime_search_path_get_cached` is what says no reader holds
        // the buffer this may reallocate.
        let mut path = unsafe { cached.into_vec() };

        // Both indices are found against the path as it stands: the
        // `after/` entry belongs above the other, and the entries between
        // them gain only the first entry's length. With no `after/` entry
        // there is no split, and `after_pos` is not even set.
        let after_at = if afterlen > 0 {
            insert_at(&path, after_pos)
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
            path.insert(
                after_at,
                SearchPathItem {
                    // SAFETY: the caller's NUL-terminated directory.
                    path: unsafe { xstrdup(afterdir) },
                    after: true,
                    pack_inserted: true,
                    has_lua: None,
                    pos_in_rtp: after_pos + addlen,
                },
            );
        }
        path.insert(
            first_at,
            SearchPathItem {
                // SAFETY: as above.
                path: unsafe { xstrdup(fname) },
                after: false,
                pack_inserted: true,
                has_lua: None,
                pos_in_rtp: first_pos,
            },
        );
        *cached = RuntimeSearchPath::from_vec(path);
    });
}

/// Add the package directory `fname` to 'runtimepath'.
///
/// `is_pack` says this is a `pack/*/start/*` bundle rather than a plain
/// directory, which changes how its `after/` is tested for and keeps the
/// cached path from being spliced.
///
/// # Safety
/// `fname` must be a writable NUL-terminated path.
unsafe fn add_pack_dir_to_rtp(fname: *mut c_char, is_pack: bool) -> Result<(), Failed> {
    // SAFETY: `fname` is the caller's path.
    let ffname = unsafe { package_root(fname) };
    if ffname.is_null() {
        return Err(Failed);
    }
    // SAFETY: `ffname` is an owned NUL-terminated string, freed below.
    let fname_len = unsafe { cstr::bytes_at(ffname) }.len();
    let points = unsafe { find_insert_points(ffname, fname_len) };

    let mut afterdir = ptr::null_mut();
    let mut retval = Err(Failed);
    if let Some(points) = points {
        // SAFETY: `fname` is NUL-terminated; `afterdir` is owned and freed
        // below.
        afterdir = unsafe { concat_fnames(fname, c"after".as_ptr(), true) };
        // Does `{fname}/after` exist — and, for a bundle, hold anything?
        let has_after = if is_pack {
            unsafe { pack_has_entries(afterdir) }
        } else {
            unsafe { os_isdir(afterdir) }
        };
        let afterlen = if has_after {
            unsafe { cstr::bytes_at(afterdir).len() + 1 }
        } else {
            0
        };
        let addlen = unsafe { cstr::bytes_at(fname) }.len() + 1; // +1 for the comma

        if let Some(spliced) = unsafe { splice_rtp(fname, afterdir, addlen, afterlen, &points) } {
            let was_valid = runtime_search_path_valid.get();
            set_option_value_give_err(
                kOptRuntimepath,
                OptVal::String(unsafe { cstr_as_string(spliced.rtp) }),
                OptionSetFlags::NONE,
            );
            debug_assert!(
                !runtime_search_path_valid.get(),
                "!runtime_search_path_valid"
            );
            if was_valid && !is_pack && runtime_search_path_ref.get().is_null() {
                unsafe {
                    splice_cached_path(
                        fname,
                        afterdir,
                        addlen,
                        afterlen,
                        spliced.first_pos,
                        spliced.after_pos,
                    )
                };
            }
            unsafe { xfree(spliced.rtp.cast()) };
            retval = Ok(());
        }
    }

    // SAFETY: both are this frame's own allocations.
    unsafe { xfree(ffname.cast()) };
    unsafe { xfree(afterdir.cast()) };
    retval
}

/// `"%s/plugin/**/*"` — every plugin file a package holds.
const PLUGIN_PATTERN: &CStr = c"%s/plugin/**/*";

/// `"%s/ftdetect/*"` — an opt package's filetype detection.
const FTDETECT_PATTERN: &CStr = c"%s/ftdetect/*";

/// Source the scripts in a package's `plugin` directory.
///
/// For `opt` packages the `ftdetect` scripts are sourced too; `start`
/// packages already have those picked up by `filetype.lua`.
///
/// # Safety
/// `fname` must be a NUL-terminated path.
unsafe fn load_pack_plugin(opt: bool, fname: *mut c_char) -> Result<(), Failed> {
    // SAFETY: `fname` is the caller's path; `ffname` and `pat` are owned and
    // freed below.
    let ffname = unsafe { fix_fname(fname) };
    let len = unsafe { cstr::bytes_at(ffname) }.len() + PLUGIN_PATTERN.count_bytes() + 1;
    let mut pat = unsafe { xmallocz(len) }.cast::<c_char>();
    let visitor = Visitor {
        callback: Some(source_callback_vim_lua as DoInRuntimepathCBFn),
        cookie: ptr::null_mut(),
    };

    unsafe { vim_snprintf(pat, len, PLUGIN_PATTERN.as_ptr(), ffname) };
    let _ =
        unsafe { gen_expand_wildcards_and_cb(1, &raw mut pat, ExpandFlags::FILE, true, visitor) };

    // When runtime/filetype.lua has not been loaded yet, these scripts
    // are found when it is.
    let cmd = unsafe { xstrdup(c"g:did_load_filetypes".as_ptr()) };
    if opt && unsafe { eval_to_number(cmd, false) } > 0 {
        let _ = unsafe { do_cmdline_cmd(c"augroup filetypedetect".as_ptr()) };
        unsafe { vim_snprintf(pat, len, FTDETECT_PATTERN.as_ptr(), ffname) };
        let patp = &raw mut pat;
        let _ = unsafe { gen_expand_wildcards_and_cb(1, patp, ExpandFlags::FILE, true, visitor) };
        let _ = unsafe { do_cmdline_cmd(c"augroup END".as_ptr()) };
    }
    unsafe { xfree(cmd.cast()) };
    unsafe { xfree(pat.cast()) };
    unsafe { xfree(ffname.cast()) };
    Ok(())
}

/// Whether `fname` is already an entry of 'runtimepath'.
///
/// # Safety
/// `fname` must be NUL-terminated.
unsafe fn rtp_has_entry(fname: *mut c_char) -> bool {
    let mut buf = [0 as c_char; MAXPATHL as usize];
    let mut p: *const c_char = p_rtp.get();
    // SAFETY: `p` walks 'runtimepath'; `buf` has `MAXPATHL` bytes.
    while unsafe { *p } != 0 {
        // SAFETY: as above.
        unsafe {
            copy_option_part(
                &raw mut p as *mut *mut c_char,
                buf.as_mut_ptr(),
                MAXPATHL as size_t,
                c",".as_ptr().cast_mut(),
            )
        };
        if unsafe { path_fnamecmp(buf.as_mut_ptr(), fname) } == 0 {
            return true;
        }
    }
    false
}

/// Add each package in `fnames` to 'runtimepath' and/or source it, as the
/// cookie asks.
///
/// # Safety
/// `fnames` must hold `num_fnames` NUL-terminated paths.
unsafe fn add_pack_plugins(
    opt: bool,
    num_fnames: c_int,
    fnames: *mut *mut c_char,
    all: bool,
    cookie: *mut c_void,
) {
    // SAFETY: the callback contract.
    let fnames = unsafe { matches(num_fnames, fnames) };
    let work = PackWork::from_cookie(cookie);
    let mut did_one = false;

    if PackWork::adds_dir(work) {
        for &fname in fnames {
            // SAFETY: one of the caller's paths.
            if unsafe { !rtp_has_entry(fname) && add_pack_dir_to_rtp(fname, false).is_err() } {
                return;
            }
            did_one = true;
            if !all {
                break;
            }
        }
    }

    if !all && did_one {
        return;
    }

    if PackWork::sources(work) {
        for &fname in fnames {
            // SAFETY: as above.
            let _ = unsafe { load_pack_plugin(opt, fname) };
            if !all {
                break;
            }
        }
    }
}

/// [`add_pack_plugins`] for a `start` package.
///
/// # Safety
/// As [`add_pack_plugins`].
unsafe fn add_start_pack_plugins(
    num_fnames: c_int,
    fnames: *mut *mut c_char,
    all: bool,
    cookie: *mut c_void,
) -> bool {
    // SAFETY: the callback contract.
    unsafe { add_pack_plugins(false, num_fnames, fnames, all, cookie) };
    num_fnames > 0
}

/// [`add_pack_plugins`] for an `opt` package.
///
/// # Safety
/// As [`add_pack_plugins`].
unsafe fn add_opt_pack_plugins(
    num_fnames: c_int,
    fnames: *mut *mut c_char,
    all: bool,
    cookie: *mut c_void,
) -> bool {
    // SAFETY: the callback contract.
    unsafe { add_pack_plugins(true, num_fnames, fnames, all, cookie) };
    num_fnames > 0
}

/// Add all packages in the `start` directories to 'runtimepath'.
pub unsafe fn add_pack_start_dirs() {
    // SAFETY: `add_pack_start_dir` ignores its cookie.
    unsafe {
        do_in_path(
            p_pp.get(),
            c"".as_ptr(),
            ptr::null_mut(),
            RuntimeOpts::ALL | RuntimeOpts::DIR,
            Some(add_pack_start_dir as DoInRuntimepathCBFn),
            ptr::null_mut(),
        )
    };
}

/// Whether the wildcard pattern `buf` matches any directory at all.
///
/// # Safety
/// `buf` must be a NUL-terminated pattern.
unsafe fn pack_has_entries(buf: *mut c_char) -> bool {
    let mut num_files: c_int = 0;
    let mut files: *mut *mut c_char = ptr::null_mut();
    let mut pat = [buf];
    // SAFETY: a one-element pattern array; the matches are ours to free.
    if unsafe {
        gen_expand_wildcards(
            1,
            pat.as_mut_ptr(),
            &raw mut num_files,
            &raw mut files,
            ExpandFlags::DIR,
        )
    }
    .is_ok()
    {
        unsafe { free_wild(num_files, files) };
    }
    num_files > 0
}

/// The two shapes a 'packpath' entry's `start` packages can take.
const START_PATTERNS: [&CStr; 2] = [c"/start/*", c"/pack/*/start/*"];

/// Add one 'packpath' directory's `start` packages to 'runtimepath'.
///
/// # Safety
/// `fnames` must hold `num_fnames` NUL-terminated directories.
unsafe fn add_pack_start_dir(
    num_fnames: c_int,
    fnames: *mut *mut c_char,
    all: bool,
    _cookie: *mut c_void,
) -> bool {
    let mut buf = [0 as c_char; MAXPATHL as usize];
    // SAFETY: the callback contract.
    let fnames = unsafe { matches(num_fnames, fnames) };
    for &fname in fnames {
        for start_pat in START_PATTERNS {
            // SAFETY: the length test keeps both halves inside `buf`, and
            // `xstrlcpy`/`xstrlcat` NUL-terminate within the size given.
            if unsafe { cstr::bytes_at(fname) }.len() + start_pat.count_bytes() + 1
                > MAXPATHL as size_t
            {
                continue;
            }
            unsafe { xstrlcpy(buf.as_mut_ptr(), fname, MAXPATHL as size_t) };
            unsafe { xstrlcat(buf.as_mut_ptr(), start_pat.as_ptr(), buf.len()) };
            if unsafe { pack_has_entries(buf.as_mut_ptr()) } {
                let _ = unsafe { add_pack_dir_to_rtp(buf.as_mut_ptr(), true) };
            }
        }
        if !all {
            break;
        }
    }
    num_fnames > 1
}

/// Load the plugins of every package in the `start` directories.
pub unsafe fn load_start_packages() {
    did_source_packages.set(true);
    // SAFETY: `add_start_pack_plugins` takes a `PackWork` cookie.
    for name in [c"pack/*/start/*", c"start/*"] {
        unsafe {
            do_in_path(
                p_pp.get(),
                c"".as_ptr(),
                name.as_ptr().cast_mut(),
                RuntimeOpts::ALL | RuntimeOpts::DIR,
                Some(add_start_pack_plugins as DoInRuntimepathCBFn),
                PackWork::Load.cookie(),
            )
        };
    }
    unsafe { update_runtime_search_path_thread(false) };
}

/// `:packloadall[!]`.
pub unsafe fn ex_packloadall(eap: *mut exarg_T) {
    // SAFETY: `eap` is the live command.
    if did_source_packages.get() && unsafe { (*eap).forceit } == 0 {
        return;
    }
    // One round to add every directory to 'runtimepath', then a second to
    // load the plugins, so a plugin may use another plugin's autoload
    // directory.
    // SAFETY: neither reads the command.
    unsafe { add_pack_start_dirs() };
    unsafe { load_start_packages() };
}

/// Read all the plugin files at startup.
pub unsafe fn load_plugins() {
    if p_lpl.get() == 0 {
        return;
    }
    let plugin_pattern = c"plugin/**/*".as_ptr().cast_mut();
    // SAFETY: the whole body is startup sequencing over NUL-terminated
    // patterns; `rtp_copy` is owned only when it was copied.
    let mut rtp_copy = p_rtp.get();
    if !did_source_packages.get() {
        rtp_copy = unsafe { xstrdup(p_rtp.get()) };
        unsafe { add_pack_start_dirs() };
    }

    // Not `source_runtime_vim_lua` yet, so `:packloadall` can be checked
    // for below. NB: after this call "rtp_copy" may have been freed, if it
    // was not copied.
    unsafe {
        source_in_path_vim_lua(
            rtp_copy,
            plugin_pattern,
            RuntimeOpts::ALL | RuntimeOpts::NOAFTER,
        )
    };
    unsafe { time_msg_now(c"loading rtp plugins") };

    // Only source "start" packages when a `:packloadall` has not already.
    if !did_source_packages.get() {
        unsafe { xfree(rtp_copy.cast()) };
        unsafe { load_start_packages() };
    }
    unsafe { time_msg_now(c"loading packages") };

    let _ =
        unsafe { source_runtime_vim_lua(plugin_pattern, RuntimeOpts::ALL | RuntimeOpts::AFTER) };
    unsafe { time_msg_now(c"loading after plugins") };
}

/// `TIME_MSG()`: note a startup milestone, when `--startuptime` asked for one.
///
/// # Safety
/// `msg` outlives the call, which every literal does.
unsafe fn time_msg_now(msg: &CStr) {
    if time_fd.get().is_null() {
        return;
    }
    // SAFETY: a literal, and the null proftime means "now".
    unsafe { time_msg(msg.as_ptr(), ptr::null()) };
}

/// `pack/*/{start,opt}/{name}` — where `:packadd` looks.
const PACKADD_PATTERN: &CStr = c"pack/*/%s/%s";

/// `:packadd[!] {name}`.
pub unsafe fn ex_packadd(eap: *mut exarg_T) {
    // SAFETY: `eap` is the live command; `pat` is owned and freed below.
    let arg = unsafe { (*eap).arg };
    let len = PACKADD_PATTERN.count_bytes() + 1 + unsafe { cstr::bytes_at(arg) }.len() + 5;
    let pat = unsafe { xmallocz(len) }.cast::<c_char>();
    let cookie = if unsafe { (*eap).forceit } != 0 {
        PackWork::AddDir
    } else {
        PackWork::Both
    }
    .cookie();

    // Only look under "start" when loading packages was not done yet.
    let mut res = OK;
    if !did_source_packages.get() {
        unsafe { vim_snprintf(pat, len, PACKADD_PATTERN.as_ptr(), c"start".as_ptr(), arg) };
        res = unsafe {
            do_in_path(
                p_pp.get(),
                c"".as_ptr(),
                pat,
                RuntimeOpts::ALL | RuntimeOpts::DIR,
                Some(add_start_pack_plugins as DoInRuntimepathCBFn),
                cookie,
            )
        };
    }

    // Give a "not found" error when nothing was found in 'start' or 'opt'.
    unsafe { vim_snprintf(pat, len, PACKADD_PATTERN.as_ptr(), c"opt".as_ptr(), arg) };
    unsafe {
        do_in_path(
            p_pp.get(),
            c"".as_ptr(),
            pat,
            RuntimeOpts::ALL | RuntimeOpts::DIR | RuntimeOpts::ERR.when(res == FAIL),
            Some(add_opt_pack_plugins as DoInRuntimepathCBFn),
            cookie,
        )
    };

    unsafe { update_runtime_search_path_thread(false) };
    unsafe { xfree(pat.cast()) };
}
