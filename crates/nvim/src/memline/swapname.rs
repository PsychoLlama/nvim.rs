//! Deciding what to call the swap file, and what to do when
//! that name is taken.
//!
//! `findswapname` walks `'directory'` for a name nothing else is using. A name
//! that *is* using it means either a crash to recover from or another Nvim with
//! the file open, which is what `attention_message` and the `SwapExists`
//! autocommand (`do_swapexists`) exist to sort out.
//!
//! `recover_names` walks the same option in the other direction: which swap
//! files already exist for a given file, for `:recover`, `swapfilelist()` and
//! the ATTENTION message.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::buffer::BufFlags;
use crate::cstr;
use crate::ex_docmd::cmdmod_has;
use crate::guard::{Lock, Suppress};
use crate::memory::xstrlcpy;
use crate::message_fmt::c_str;
use crate::path::ExpandFlags;
use crate::semsg;
use ::libc::{EINVAL, ENOENT};
use core::ffi::{CStr, c_char, c_int, c_uint};

use super::*;
use crate::types::{CmdModFlags, Failed, MAXPATHL, NUL, ShmFlag, Vv};

/// Rename the swap file after the buffer's file name changed.
///
/// The name is what identifies the swap file to the next `:recover`, so it
/// has to follow the file. Failing that, the swap file is at least reopened
/// under its old name — losing it entirely is worse than a stale name.
pub unsafe fn ml_setname(buf: *mut buf_T) {
    let mfp = unsafe { (*buf).b_ml.ml_mfp };
    if unsafe { (*mfp).mf_fd } < 0 {
        // There is no swap file yet: with `'updatecount'` zero and
        // `'noswapfile'` there never was one. Help files get one now.
        if p_uc.get() != 0 && !cmdmod_has(CmdModFlags::NOSWAPFILE) {
            unsafe { ml_open_file(buf) };
        }
        return;
    }

    // Try every directory in 'directory'.
    let mut success = false;
    let mut dirp = p_dir.get();
    let mut found_existing_dir = false;
    while unsafe { *dirp } as c_int != NUL {
        let fname = unsafe {
            findswapname(
                buf,
                &raw mut dirp,
                mf_fname(mfp),
                &raw mut found_existing_dir,
            )
        };
        if dirp.is_null() {
            break; // out of memory
        }
        if fname.is_null() {
            continue; // no name found for this directory
        }

        // Already called that: nothing to do.
        if unsafe { path_fnamecmp(fname, mf_fname(mfp)) } == 0 {
            unsafe { xfree(fname.cast()) };
            success = true;
            break;
        }
        // The swap file has to be closed before it can be renamed.
        if unsafe { (*mfp).mf_fd } >= 0 {
            unsafe { close((*mfp).mf_fd) };
            unsafe { (*mfp).mf_fd = -1 };
        }
        if unsafe { vim_rename(mf_fname(mfp), fname) } == 0 {
            success = true;
            unsafe { mf_free_fnames(mfp) };
            unsafe { mf_set_fnames(mfp, fname) };
            unsafe { ml_upd_block0(buf, UB_SAME_DIR) };
            break;
        }
        unsafe { xfree(fname.cast()) }; // this name did not work, try another
    }

    if unsafe { (*mfp).mf_fd } == -1 {
        unsafe { (*mfp).mf_fd = os_open(mf_fname(mfp), O_RDWR, 0) };
        if unsafe { (*mfp).mf_fd } < 0 {
            // Could not reopen the swap file. Nothing can be done.
            complain(c"E301: Oops, lost the swap file!!!");
            return;
        }
        unsafe { os_set_cloexec((*mfp).mf_fd) };
    }
    if !success {
        complain(c"E302: Could not rename swap file");
    }
}

/// Append `name`'s full path to `dir`, with the path separators turned into
/// percent signs, so that a `'directory'` entry ending in `//` can hold the
/// swap files of files from everywhere without their names colliding.
///
/// An unnamed buffer is handled as `""`, i.e. `<currentdir>/""`. The last
/// character of `dir` must be an extra path separator; it is removed.
pub unsafe fn make_percent_swname(
    dir: *mut c_char,
    dir_end: *mut c_char,
    name: *const c_char,
) -> *mut c_char {
    let f = unsafe { fix_fname(if name.is_null() { c"".as_ptr() } else { name }) };
    if f.is_null() {
        return core::ptr::null_mut();
    }

    let s = unsafe { xstrdup(f) };
    let mut d = s;
    while unsafe { *d } as c_int != NUL {
        if vim_ispathsep(unsafe { *d } as c_int) {
            unsafe { *d = b'%' as c_char };
        }
        d = unsafe { d.offset(utfc_ptr2len(d) as isize) };
    }

    unsafe { *dir_end.offset(-1) = NUL as c_char }; // remove one trailing slash
    let joined = unsafe { concat_fnames(dir, s, true) };
    unsafe { xfree(s.cast()) };
    unsafe { xfree(f.cast()) };
    joined
}

/// Resolve a symbolic link in the *last* component of a file name, writing
/// the result into `buf[MAXPATHL]`.
///
/// `resolve()` in Vimscript does this for every part of the path; this does
/// not. Returns `Ok` when `buf` holds a resolved name, `Err` when the caller
/// should keep the name it already has.
pub unsafe fn resolve_symlink(fname: *const c_char, buf: *mut c_char) -> Result<(), Failed> {
    if fname.is_null() {
        return Err(Failed);
    }

    // The result so far lives in `tmp`, starting with the original name.
    let mut tmp: [c_char; MAXPATHL as usize] = [0; MAXPATHL as usize];
    unsafe { xstrlcpy(tmp.as_mut_ptr(), fname, MAXPATHL as size_t) };

    let mut depth = 0;
    loop {
        // Bound the depth, to catch a loop of links pointing at each
        // other.
        depth += 1;
        if depth == 100 {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let fname = unsafe { c_str(fname) };
            semsg!("E773: Symlink loop for \"{fname}\"");
            return Err(Failed);
        }

        let ret = unsafe { readlink(tmp.as_mut_ptr(), buf, MAXPATHL as size_t - 1) } as c_int;
        if ret <= 0 {
            if unsafe { *__errno_location() } != EINVAL && unsafe { *__errno_location() } != ENOENT
            {
                // Some other error reading the link: keep the original.
                return Err(Failed);
            }
            // Not a symlink, or it does not exist, so `tmp` is as
            // resolved as it gets. At the first level that means nothing
            // was resolved at all, and the caller keeps its own name.
            if depth == 1 {
                return Err(Failed);
            }
            break;
        }
        unsafe { *buf.offset(ret as isize) = NUL as c_char };

        // A relative link is relative to the directory of the name it
        // was found in, so it replaces only the tail of `tmp`.
        if unsafe { path_is_absolute(buf) } {
            unsafe { xstrlcpy(tmp.as_mut_ptr(), buf, tmp.len()) };
        } else {
            let tail = unsafe { path_tail(tmp.as_ptr()) };
            if unsafe { cstr::bytes_at(tail) }.len() + unsafe { cstr::bytes_at(buf) }.len()
                >= MAXPATHL as size_t
            {
                return Err(Failed);
            }
            let room = tmp.len() - unsafe { tail.offset_from(tmp.as_ptr()) } as usize;
            unsafe { xstrlcpy(tail, buf, room) };
        }
    }

    // Resolve the full name too, so that opening the same relative
    // symlink from two working directories still picks one swap file.
    unsafe { vim_full_name(tmp.as_ptr(), buf, MAXPATHL as size_t, true) }
}

/// The swap file name for `fname` under the `'directory'` entry `dir_name`,
/// allocated, or null.
pub unsafe fn makeswapname(
    fname: *mut c_char,
    _ffname: *mut c_char,
    _buf: *mut buf_T,
    dir_name: *mut c_char,
) -> *mut c_char {
    // Expand a symlink, so that the swap file goes with the actual file
    // rather than with the link.
    let mut fname_buf: [c_char; MAXPATHL as usize] = [0; MAXPATHL as usize];
    let fname_res = if unsafe { resolve_symlink(fname, fname_buf.as_mut_ptr()) }.is_ok() {
        fname_buf.as_mut_ptr()
    } else {
        fname
    };

    let len = unsafe { cstr::bytes_at(dir_name) }.len();
    let end = unsafe { dir_name.add(len) };
    if unsafe { after_pathsep(dir_name, end) } != 0
        && len > 1
        && unsafe { *end.offset(-1) } as c_int == unsafe { *end.offset(-2) } as c_int
    {
        // Ends with "//": the swap file's name encodes the full path.
        let mut r = core::ptr::null_mut();
        let s = unsafe { make_percent_swname(dir_name, end, fname_res) };
        if !s.is_null() {
            r = unsafe { modname(s, c".swp".as_ptr(), false) };
            unsafe { xfree(s.cast()) };
        }
        return r;
    }

    // A swap file in the file's own directory gets a leading '.'.
    let r = unsafe {
        modname(
            fname_res,
            c".swp".as_ptr(),
            *dir_name as c_int == '.' as c_int && *dir_name.offset(1) as c_int == NUL,
        )
    };
    if r.is_null() {
        return core::ptr::null_mut(); // out of memory
    }
    let s = unsafe { get_file_in_dir(r, dir_name) };
    unsafe { xfree(r.cast()) };
    s
}

/// Place `fname` in the directory `dname` names, for a swap or backup file.
///
/// - `dname` is `"."`: beside the file, i.e. `fname` itself.
/// - `dname` starts with `"./"`: relative to the file's own directory, with
///   the rest of `dname` spliced in before the tail.
/// - otherwise: in `dname`, under `fname`'s tail.
///
/// The result is allocated, and may be null.
pub unsafe fn get_file_in_dir(fname: *mut c_char, dname: *mut c_char) -> *mut c_char {
    let tail = unsafe { path_tail(fname) };
    if unsafe { *dname } as c_int == '.' as c_int && unsafe { *dname.offset(1) } as c_int == NUL {
        unsafe { xstrdup(fname) }
    } else if unsafe { *dname } as c_int == '.' as c_int
        && vim_ispathsep(unsafe { *dname.offset(1) } as c_int)
    {
        if tail == fname {
            // No path in front of the file name.
            unsafe { concat_fnames(dname.offset(2), tail, true) }
        } else {
            let save_char = unsafe { *tail };
            unsafe { *tail = NUL as c_char };
            let t = unsafe { concat_fnames(fname, dname.offset(2), true) };
            unsafe { *tail = save_char };
            let retval = unsafe { concat_fnames(t, tail, true) };
            unsafe { xfree(t.cast()) };
            retval
        }
    } else {
        unsafe { concat_fnames(dname, tail, true) }
    }
}

/// Build the ATTENTION message: what is known about the swap file that is in
/// the way, and what the user can do about it.
///
/// `fhname` is `fname` with the home directory replaced by `~`.
unsafe fn attention_message(
    buf: *mut buf_T,
    fname: *mut c_char,
    fhname: *mut c_char,
    msg: *mut StringBuilder,
) {
    debug_assert!(!unsafe { (*buf).b_fname.is_null() });

    complain(c"E325: ATTENTION");
    unsafe { kv_puts(msg, c"Found a swap file by the name \"") };
    unsafe { kv_do_printf(msg, c"%s\"\n".as_ptr(), fhname) };
    let swap_mtime = unsafe { swapfile_info(fname, msg) };
    unsafe { kv_puts(msg, c"While opening file \"") };
    unsafe { kv_do_printf(msg, c"%s\"\n".as_ptr(), (*buf).b_fname) };

    let mut file_info: FileInfo = unsafe { core::mem::zeroed() };
    if !unsafe { os_fileinfo((*buf).b_fname, &raw mut file_info) } {
        unsafe { kv_puts(msg, c"      CANNOT BE FOUND") };
    } else {
        unsafe { kv_puts(msg, c"             dated: ") };
        let x = file_info.stat.st_mtim.tv_sec as time_t;
        let mut ctime_buf: [c_char; 50] = [0; 50];
        unsafe { kv_do_printf(msg, c"%s".as_ptr(), os_ctime_r(x, &mut ctime_buf, true)) };
        if swap_mtime != 0 && x > swap_mtime {
            unsafe { kv_puts(msg, c"      NEWER than swap file!\n") };
        }
    }

    // Some of these are long, to leave room for translation.
    unsafe {
        kv_puts(
        msg,
        c"\n(1) Another program may be editing the same file.  If this is the case,\n    be careful not to end up with two different instances of the same\n    file when making changes.  Quit, or continue with caution.\n",
    )
    };
    unsafe { kv_puts(msg, c"(2) An edit session for this file crashed.\n") };
    unsafe {
        kv_puts(
            msg,
            c"    If this is the case, use \":recover\" or \"nvim -r ",
        )
    };
    unsafe { kv_do_printf(msg, c"%s".as_ptr(), (*buf).b_fname) };
    unsafe {
        kv_puts(
            msg,
            c"\"\n    to recover the changes (see \":help recovery\").\n",
        )
    };
    unsafe { kv_puts(msg, c"    If you did this already, delete the swap file \"") };
    unsafe { kv_do_printf(msg, c"%s".as_ptr(), fname) };
    unsafe { kv_puts(msg, c"\"\n    to avoid this message.\n") };
}

/// Fire the `SwapExists` autocommands and read the choice they left in
/// `v:swapchoice`.
unsafe fn do_swapexists(buf: *mut buf_T, fname: *mut c_char) -> sea_choice_T {
    unsafe { set_vim_var_string(Vv::Swapname, fname, -1) };
    unsafe { set_vim_var_string(Vv::Swapchoice, core::ptr::null(), -1) };

    // `<afile>` is the file being edited. Changing directory is not
    // allowed from here.
    let locked = Lock::all_buffers();
    let name = unsafe { (*buf).b_fname };
    let (no_io, no_buf) = (core::ptr::null_mut(), core::ptr::null_mut());
    unsafe { apply_autocmds(AutoEvent::SwapExists, name, no_io, false, no_buf) };
    drop(locked);

    unsafe { set_vim_var_string(Vv::Swapname, core::ptr::null(), -1) };

    match unsafe { *get_vim_var_str(Vv::Swapchoice) } as u8 {
        b'o' => SEA_CHOICE_READONLY,
        b'e' => SEA_CHOICE_EDIT,
        b'r' => SEA_CHOICE_RECOVER,
        b'd' => SEA_CHOICE_DELETE,
        b'q' => SEA_CHOICE_QUIT,
        b'a' => SEA_CHOICE_ABORT,
        _ => SEA_CHOICE_NONE,
    }
}

/// A swap file already has the name we wanted. Decide what that means and
/// act on it: delete a useless one, let a `SwapExists` autocommand choose, or
/// put the ATTENTION message in front of the user.
///
/// Returns true when the swap file is gone afterwards, so its name is free.
unsafe fn resolve_swapfile_clash(
    buf: *mut buf_T,
    fname: *mut c_char,
    buf_fname: *mut c_char,
) -> bool {
    // Only worth a word if the swap file belongs to *this* file, the
    // buffer was not already recovered, and 'shortmess' allows it.
    if unsafe { swapfile_is_for_other_file(buf, fname) }
        || cur_buf().b_flags.has(BufFlags::RECOVERED)
        || ShmFlag::ATTENTION.is_in(unsafe { CStr::from_ptr(p_shm.get()) })
    {
        return false;
    }

    let mut choice = SEA_CHOICE_NONE;

    // Deleting it is safe when the file exists and the swap file records
    // no changes and looks intact.
    if unsafe { os_path_exists((*buf).b_fname) } && unsafe { swapfile_unchanged(fname) } {
        choice = SEA_CHOICE_DELETE;
        if p_verbose.get() > 0 {
            unsafe { verb_msg(tr(c"Found a swap file that is not useful, deleting it")) };
        }
    }

    // A SwapExists autocommand gets the next word, if the caller is in a
    // position to honour what it says. It may still decline (0) and leave
    // the question to the user.
    if choice == SEA_CHOICE_NONE
        && swap_exists_action.get() != SEA_NONE
        && unsafe { has_autocmd(AutoEvent::SwapExists, buf_fname, Buf::from_raw(buf)) }
    {
        choice = unsafe { do_swapexists(buf, fname) };
    }
    if choice == SEA_CHOICE_NONE && swap_exists_action.get() == SEA_READONLY {
        choice = SEA_CHOICE_READONLY;
    }

    // Set by attention_message -> swapfile_info, below.
    proc_running.set(0);
    if choice == SEA_CHOICE_NONE {
        choice = unsafe { ask_about_swapfile(buf, fname) };
    }

    match choice {
        SEA_CHOICE_READONLY => unsafe { (*buf).b_p_ro = 1 },
        SEA_CHOICE_RECOVER => swap_exists_action.set(SEA_RECOVER),
        SEA_CHOICE_DELETE => {
            unsafe { os_remove(fname) };
        }
        SEA_CHOICE_QUIT => swap_exists_action.set(SEA_QUIT),
        SEA_CHOICE_ABORT => {
            swap_exists_action.set(SEA_QUIT);
            got_int.set(true);
        }
        SEA_CHOICE_NONE => {
            unsafe { msg_puts(c"\n".as_ptr()) };
            if msg_silent.get() == 0 {
                need_wait_return.set(true); // call wait_return() later
            }
        }
        // SEA_CHOICE_EDIT: use the file as it is.
        _ => {}
    }

    // If the swap file was deleted, this name can be used after all.
    choice != SEA_CHOICE_NONE && !unsafe { os_path_exists(fname) }
}

/// Show the ATTENTION message, as a dialog if the caller can act on an
/// answer and as a warning otherwise.
unsafe fn ask_about_swapfile(buf: *mut buf_T, fname: *mut c_char) -> sea_choice_T {
    let mut choice = SEA_CHOICE_NONE;
    let no_prompt = Suppress::wait_return();

    // kv_resize(msg, IOSIZE): a screenful before the first realloc.
    let mut msg: StringBuilder = KV_INITIAL_VALUE;
    msg.capacity = 1024 + 1;
    msg.items = unsafe { xrealloc(msg.items.cast(), msg.capacity) }.cast();

    let fhname = unsafe { home_replace_save(core::ptr::null_mut(), fname) };
    unsafe { attention_message(buf, fname, fhname, &raw mut msg) };

    // A 'q' typed at the more-prompt must not interrupt loading the
    // file, and a "simalt ~x" in the vimrc must not answer the prompt
    // below.
    got_int.set(false);
    unsafe { flush_buffers(FLUSH_TYPEAHEAD) };

    if swap_exists_action.get() != SEA_NONE {
        unsafe { kv_puts(&raw mut msg, c"Swap file \"") };
        unsafe { kv_do_printf(&raw mut msg, c"%s".as_ptr(), fhname) };
        unsafe { kv_puts(&raw mut msg, c"\" already exists!") };
        // "Delete it" is not offered while the owning process is alive.
        let run_but = tr(c"&Open Read-Only\n&Edit anyway\n&Recover\n&Quit\n&Abort");
        let but = tr(c"&Open Read-Only\n&Edit anyway\n&Recover\n&Delete it\n&Quit\n&Abort");
        let running = proc_running.get() != 0;
        let buttons = if running { run_but } else { but };
        let (title, warn) = (tr(c"VIM - ATTENTION"), VIM_WARNING as c_int);
        let none = core::ptr::null();
        choice = unsafe { do_dialog(warn, title, msg.items, buttons, 1, none, 0) } as sea_choice_T;
        // Compensate for the missing "Delete it" button.
        choice = choice.wrapping_add((running && choice >= 4) as sea_choice_T);
        // Pretend the screen did not scroll; it needs a redraw anyway.
        unsafe { msg_reset_scroll() };
    } else {
        let mut need_clear = false;
        unsafe { msg_ext_set_kind(c"wmsg".as_ptr()) };
        let text = String_0::from_raw_parts(msg.items, msg.size);
        let clear = &raw mut need_clear;
        unsafe { msg_multiline(text, 0, false, false, clear) };
    }

    drop(no_prompt);
    unsafe { xfree(msg.items.cast()) }; // kv_destroy(msg)
    unsafe { xfree(fhname.cast()) };
    choice
}

/// Find a name for `buf`'s swap file under the next entry of `'directory'`.
///
/// Names are tried in turn until one is free: `".swp"`, then `".swo"`,
/// `".swn"` and so on down to `".saa"`. The last directory in the option is
/// created if it does not exist.
///
/// May trigger the `SwapExists` autocommand, so pointers may change.
///
/// `dirp` walks the `'directory'` list and is advanced past the entry used.
/// `old_fname`, when given, is a name this buffer already owns and may keep.
/// `found_existing_dir` starts out false and is only ever set — once any
/// directory in the list exists, no new one is created.
///
/// Returns the allocated name, or null.
pub(crate) unsafe fn findswapname(
    buf: *mut buf_T,
    dirp: *mut *mut c_char,
    old_fname: *const c_char,
    found_existing_dir: *mut bool,
) -> *mut c_char {
    let buf_fname = unsafe { (*buf).b_fname };

    // Isolate one directory name out of *dirp.
    let dir_len = unsafe { cstr::bytes_at(*dirp) }.len() + 1;
    let dir_name = unsafe { xmalloc(dir_len) } as *mut c_char;
    unsafe { copy_option_part(dirp, dir_name, dir_len, c",".as_ptr().cast_mut()) };

    let mut fname = unsafe { makeswapname(buf_fname, (*buf).b_ffname, buf, dir_name) };
    loop {
        if fname.is_null() {
            break; // out of memory
        }
        let n = unsafe { cstr::bytes_at(fname) }.len();
        if n == 0 {
            // Safety check.
            unsafe { xfree(fname.cast()) };
            fname = core::ptr::null_mut();
            break;
        }

        // Is the name taken? A swap file that is a symbolic link is most
        // likely a symlink attack, so the link itself counts as taken.
        let mut file_info: FileInfo = unsafe { core::mem::zeroed() };
        if !unsafe { os_fileinfo_link(fname, &raw mut file_info) } {
            break;
        }
        // A name this buffer already owns is free for it to keep.
        if !old_fname.is_null() && unsafe { path_fnamecmp(fname, old_fname) } == 0 {
            break;
        }

        // The name is taken. On the first try — the plain ".swp" — that
        // means a real swap file, and it is worth telling the user about
        // unless we are recovering, have no file name, are in a help file
        // or in a dummy buffer.
        if unsafe { *fname.offset(n as isize - 2) } as u8 == b'w'
            && unsafe { *fname.offset(n as isize - 1) } as u8 == b'p'
            && !recoverymode.get()
            && !buf_fname.is_null()
            && !unsafe { (*buf).b_help }
            && !unsafe { (*buf).b_flags.has(BufFlags::DUMMY) }
            && unsafe { resolve_swapfile_clash(buf, fname, buf_fname) }
        {
            break;
        }

        // Permute the extension to find a name nothing is using: first
        // count the last character down (".swo", ".swn", …), and when
        // that runs out the one before it (".svz", ".suz", …). Both can
        // happen with many Nvims editing one file, including "No Name"
        // buffers.
        if unsafe { *fname.offset(n as isize - 1) } as u8 == b'a' {
            if unsafe { *fname.offset(n as isize - 2) } as u8 == b'a' {
                // ".saa": tried enough, give up.
                complain(c"E326: Too many swap files found");
                unsafe { xfree(fname.cast()) };
                fname = core::ptr::null_mut();
                break;
            }
            unsafe { *fname.offset(n as isize - 2) -= 1 };
            unsafe { *fname.offset(n as isize - 1) = b'z' as c_char + 1 };
        }
        unsafe { *fname.offset(n as isize - 1) -= 1 };
    }

    if unsafe { os_isdir(dir_name) } {
        unsafe { *found_existing_dir = true };
    } else if !unsafe { *found_existing_dir } && unsafe { **dirp } as c_int == NUL {
        // The last entry in 'directory' is created on demand.
        let mut failed_dir = core::ptr::null_mut();
        let ret = unsafe {
            os_mkdir_recurse(dir_name, 0o755, &raw mut failed_dir, core::ptr::null_mut())
        };
        if ret != 0 {
            // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
            let (shown, why) = unsafe { (c_str(failed_dir), c_str(uv_strerror(ret))) };
            semsg!(
                "E303: Unable to create directory \"{shown}\" for swap file, recovery impossible: {why}"
            );
            unsafe { xfree(failed_dir.cast()) };
        }
    }

    unsafe { xfree(dir_name.cast()) };
    fname
}

/// Every file matching one of `pats`, into `files`.
///
/// The flags are the same at both call sites and are what makes this a
/// lookup rather than an expansion the user sees: keep everything, files
/// only, and say nothing about a pattern that matches none.
fn expanded(
    n: c_int,
    pats: *mut *mut c_char,
    count: *mut c_int,
    files: *mut *mut *mut c_char,
) -> Result<(), Failed> {
    let how = ExpandFlags::KEEPALL | ExpandFlags::FILE | ExpandFlags::SILENT;
    // SAFETY: the caller's array of `n` patterns and its two out-parameters.
    unsafe { expand_wildcards(n, pats, count, files, how) }
}

/// Find the swap files in the current directory and in every directory of
/// the `'directory'` option.
///
/// Used to list them for `nvim -r`, to count them while recovering, to list
/// them while recovering, to fill `swapfilelist()`, and to name the n'th one.
///
/// `fname` is the file whose swap files are wanted, or null for all of them.
/// `do_list` lists the names; `ret_list`, when given, collects them; `nr`,
/// when non-zero, asks for the n'th name in `fname_out`.
///
/// Returns the number of swap files found.
pub unsafe fn recover_names(
    fname: *mut c_char,
    do_list: bool,
    ret_list: *mut list_T,
    nr: c_int,
    fname_out: *mut *mut c_char,
) -> c_int {
    // Expand a symlink, because the swap file was created against the
    // actual file rather than the link.
    let mut fname_buf: [c_char; MAXPATHL as usize] = [0; MAXPATHL as usize];
    let mut fname_res: *mut c_char = core::ptr::null_mut();
    if !fname.is_null() {
        fname_res = if unsafe { resolve_symlink(fname, fname_buf.as_mut_ptr()) }.is_ok() {
            fname_buf.as_mut_ptr()
        } else {
            fname
        };
    }

    msg_ext_skip_flush.set(true);
    if do_list {
        // Use msg() to start the scrolling properly.
        unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
        tell(c"Swap files found:", 0);
        unsafe { msg_putchar('\n' as c_int) };
    }

    let mut file_count = 0;
    let mut names: [*mut c_char; 6] = [core::ptr::null_mut(); 6];
    // One buffer for the directory name, big enough for the longest
    // entry in 'directory'.
    let mut dir_name = String_0::from_raw_parts(
        unsafe { xmalloc(cstr::bytes_at(p_dir.get()).len() + 1) } as *mut c_char,
        0,
    );
    let mut dirp = p_dir.get();
    while unsafe { *dirp } != 0 {
        // Isolate one directory name and advance `dirp` past it. The
        // buffer is known to be large enough, hence the 31000.
        dir_name.set_len(unsafe {
            copy_option_part(
                &raw mut dirp,
                dir_name.data(),
                31000,
                c",".as_ptr().cast_mut(),
            )
        });

        let num_names;
        let current_dir = unsafe { *dir_name.data() } as c_int == '.' as c_int
            && unsafe { *dir_name.data().offset(1) } as c_int == NUL;
        if fname.is_null() {
            // Every swap file, under whatever name. On unix a leading dot
            // is special, so the two forms are listed separately.
            let patterns = [c"*.sw?", c".*.sw?", c".sw?"];
            for (slot, pattern) in names.iter_mut().zip(patterns) {
                *slot = if current_dir {
                    unsafe {
                        xmemdupz(pattern.as_ptr().cast(), pattern.count_bytes()) as *mut c_char
                    }
                } else {
                    unsafe { concat_fnames(dir_name.data(), pattern.as_ptr(), true) }
                };
            }
            num_names = 3;
        } else if current_dir {
            num_names = unsafe { recov_file_names(&mut names, fname_res, true) };
        } else {
            let end = unsafe { dir_name.data().add(dir_name.len()) };
            let tail = if unsafe { after_pathsep(dir_name.data(), end) } != 0
                && dir_name.len() > 1
                && unsafe { *end.offset(-1) } as c_int == unsafe { *end.offset(-2) } as c_int
            {
                // Ends with "//": the swap file's name holds the full path.
                unsafe { make_percent_swname(dir_name.data(), end, fname_res) }
            } else {
                unsafe { concat_fnames(dir_name.data(), path_tail(fname_res), true) }
            };
            num_names = unsafe { recov_file_names(&mut names, tail, false) };
            unsafe { xfree(tail.cast()) };
        }

        let mut num_files = 0;
        let mut files: *mut *mut c_char = core::ptr::null_mut();
        if num_names != 0
            && expanded(
                num_names,
                names.as_mut_ptr(),
                &raw mut num_files,
                &raw mut files,
            )
            .is_err()
        {
            num_files = 0;
        }

        // Nothing found may mean the wildcard expansion itself failed
        // (no shell to run, say). Try the plain ".swp" name.
        if unsafe { *dirp } as c_int == NUL && file_count + num_files == 0 && !fname.is_null() {
            let mut swapname = unsafe { modname(fname_res, c".swp".as_ptr(), true) };
            if !swapname.is_null() {
                if unsafe { os_path_exists(swapname) } {
                    files = unsafe { xmalloc(size_of::<*mut c_char>()) } as *mut *mut c_char;
                    unsafe { *files = swapname };
                    swapname = core::ptr::null_mut();
                    num_files = 1;
                }
                unsafe { xfree(swapname.cast()) };
            }
        }

        // The current buffer's own swap file is not interesting — except
        // to swapfilelist(), which wants everything.
        let mine = if cur_buf().b_ml.ml_mfp.is_null() {
            core::ptr::null()
        } else {
            unsafe { mf_fname(cur_buf().b_ml.ml_mfp) }
        };
        if !mine.is_null() && ret_list.is_null() {
            let mut i = 0;
            while i < num_files {
                // Do not expand wildcards: on Windows that would try to
                // expand the "%tmp%" in "%tmp%file".
                if unsafe {
                    path_full_compare(mine.cast_mut(), *files.offset(i as isize), true, false)
                } as c_uint
                    & kEqualFiles as c_uint
                    != 0
                {
                    // Drop it and move the rest down. When the array
                    // empties it is freed here, since free_wild() below
                    // will not be reached.
                    unsafe { xfree((*files.offset(i as isize)).cast()) };
                    num_files -= 1;
                    if num_files == 0 {
                        unsafe { xfree(files.cast()) };
                    } else {
                        while i < num_files {
                            unsafe { *files.offset(i as isize) = *files.offset(i as isize + 1) };
                            i += 1;
                        }
                    }
                }
                i += 1;
            }
        }

        if nr > 0 {
            file_count += num_files;
            if nr <= file_count {
                unsafe {
                    *fname_out = xstrdup(*files.offset((nr - 1 + num_files - file_count) as isize))
                };
                dirp = c"".as_ptr().cast_mut(); // stop searching
            }
        } else if do_list {
            if current_dir {
                if fname.is_null() {
                    say(c"   In current directory:\n");
                } else {
                    say(c"   Using specified name:\n");
                }
            } else {
                say(c"   In directory ");
                unsafe { msg_home_replace(dir_name.data()) };
                unsafe { msg_puts(c":\n".as_ptr()) };
            }

            if num_files == 0 {
                say(c"      -- none --\n");
            } else {
                for i in 0..num_files {
                    file_count += 1;
                    unsafe { msg_outnum(file_count) };
                    unsafe { msg_puts(c".    ".as_ptr()) };
                    unsafe { msg_puts(path_tail(*files.offset(i as isize))) };
                    unsafe { msg_putchar('\n' as c_int) };

                    // kv_resize(msg, IOSIZE)
                    let mut msg_buf = KV_INITIAL_VALUE;
                    msg_buf.capacity = 1024 + 1;
                    msg_buf.items =
                        unsafe { xrealloc(msg_buf.items.cast(), msg_buf.capacity) }.cast();
                    unsafe { swapfile_info(*files.offset(i as isize), &raw mut msg_buf) };
                    let mut need_clear = false;
                    let text = String_0::from_raw_parts(msg_buf.items, msg_buf.size);
                    let clear = &raw mut need_clear;
                    unsafe { msg_multiline(text, 0, false, false, clear) };
                    unsafe { xfree(msg_buf.items.cast()) }; // kv_destroy(msg)
                }
            }
            unsafe { ui_flush() };
        } else if !ret_list.is_null() {
            for i in 0..num_files {
                let name =
                    unsafe { concat_fnames(dir_name.data(), *files.offset(i as isize), true) };
                unsafe { tv_list_append_allocated_string(ret_list, name) };
            }
        } else {
            file_count += num_files;
        }

        for name in names.iter().take(num_names as usize) {
            unsafe { xfree((*name).cast()) };
        }
        if num_files > 0 {
            unsafe { free_wild(num_files, files) };
        }
    }
    msg_ext_skip_flush.set(false);
    unsafe { xfree(dir_name.data().cast()) };
    file_count
}

/// Fill `names` with the wildcard patterns that match `path`'s swap files,
/// returning how many were written.
///
/// `prepend_dot` also asks for the hidden form, for a swap file kept in the
/// same directory as the file itself.
unsafe fn recov_file_names(
    names: &mut [*mut c_char; 6],
    path: *mut c_char,
    prepend_dot: bool,
) -> c_int {
    let mut num_names = 0usize;
    if prepend_dot {
        names[num_names] = unsafe { modname(path, c".sw?".as_ptr(), true) };
        if names[num_names].is_null() {
            return num_names as c_int;
        }
        num_names += 1;
    }

    // The plain form: the name with ".sw?" appended.
    names[num_names] = unsafe { concat_fnames(path, c".sw?".as_ptr(), false) };
    if num_names == 0 {
        num_names += 1;
    } else {
        // Both forms may have come out the same; keep only one.
        let mut p = names[num_names - 1];
        let extra = unsafe { cstr::bytes_at(names[num_names - 1]) }.len() as isize
            - unsafe { cstr::bytes_at(names[num_names]) }.len() as isize;
        if extra > 0 {
            p = unsafe { p.offset(extra) }; // the name was expanded to a full path
        }
        if !unsafe { cstr::eq(p, names[num_names]) } {
            num_names += 1;
        } else {
            unsafe { xfree(names[num_names].cast()) };
        }
    }
    num_names as c_int
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
