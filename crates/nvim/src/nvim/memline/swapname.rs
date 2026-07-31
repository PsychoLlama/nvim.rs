//! Deciding what to call the swap file, and what to do when
//! that name is taken.
//!
//! `findswapname` walks `'directory'` for a name nothing else is using. A name
//! that *is* using it means either a crash to recover from or another Nvim with
//! the file open, which is what `attention_message` and the `SwapExists`
//! autocommand (`do_swapexists`) exist to sort out.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

#[allow(unused_imports)]
use super::*;

/// Rename the swap file after the buffer's file name changed.
///
/// The name is what identifies the swap file to the next `:recover`, so it
/// has to follow the file. Failing that, the swap file is at least reopened
/// under its old name — losing it entirely is worse than a stale name.
pub unsafe fn ml_setname(buf: *mut buf_T) {
    unsafe {
        let mfp = (*buf).b_ml.ml_mfp;
        if (*mfp).mf_fd < 0 {
            // There is no swap file yet: with `'updatecount'` zero and
            // `'noswapfile'` there never was one. Help files get one now.
            if p_uc.get() != 0 && (*cmdmod.ptr()).cmod_flags & CMOD_NOSWAPFILE as c_int == 0 {
                ml_open_file(buf);
            }
            return;
        }

        // Try every directory in 'directory'.
        let mut success = false;
        let mut dirp = p_dir.get();
        let mut found_existing_dir = false;
        while *dirp as c_int != NUL {
            let fname = findswapname(
                buf,
                &raw mut dirp,
                (*mfp).mf_fname,
                &raw mut found_existing_dir,
            );
            if dirp.is_null() {
                break; // out of memory
            }
            if fname.is_null() {
                continue; // no name found for this directory
            }

            // Already called that: nothing to do.
            if path_fnamecmp(fname, (*mfp).mf_fname) == 0 {
                xfree(fname.cast());
                success = true;
                break;
            }
            // The swap file has to be closed before it can be renamed.
            if (*mfp).mf_fd >= 0 {
                close((*mfp).mf_fd);
                (*mfp).mf_fd = -1;
            }
            if vim_rename((*mfp).mf_fname, fname) == 0 {
                success = true;
                mf_free_fnames(mfp);
                mf_set_fnames(mfp, fname);
                ml_upd_block0(buf, UB_SAME_DIR);
                break;
            }
            xfree(fname.cast()); // this name did not work, try another
        }

        if (*mfp).mf_fd == -1 {
            (*mfp).mf_fd = os_open((*mfp).mf_fname, O_RDWR, 0);
            if (*mfp).mf_fd < 0 {
                // Could not reopen the swap file. Nothing can be done.
                emsg(gettext(c"E301: Oops, lost the swap file!!!".as_ptr()));
                return;
            }
            os_set_cloexec((*mfp).mf_fd);
        }
        if !success {
            emsg(gettext(c"E302: Could not rename swap file".as_ptr()));
        }
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
    unsafe {
        let f = fix_fname(if name.is_null() { c"".as_ptr() } else { name });
        if f.is_null() {
            return core::ptr::null_mut();
        }

        let s = xstrdup(f);
        let mut d = s;
        while *d as c_int != NUL {
            if vim_ispathsep(*d as c_int) {
                *d = b'%' as c_char;
            }
            d = d.offset(utfc_ptr2len(d) as isize);
        }

        *dir_end.offset(-1) = NUL as c_char; // remove one trailing slash
        let joined = concat_fnames(dir, s, true);
        xfree(s.cast());
        xfree(f.cast());
        joined
    }
}

/// Resolve a symbolic link in the *last* component of a file name, writing
/// the result into `buf[MAXPATHL]`.
///
/// `resolve()` in Vimscript does this for every part of the path; this does
/// not. Returns `OK` when `buf` holds a resolved name, `FAIL` when the caller
/// should keep the name it already has.
pub unsafe fn resolve_symlink(fname: *const c_char, buf: *mut c_char) -> c_int {
    unsafe {
        if fname.is_null() {
            return FAIL;
        }

        // The result so far lives in `tmp`, starting with the original name.
        let mut tmp: [c_char; MAXPATHL as usize] = [0; MAXPATHL as usize];
        xstrlcpy(tmp.as_mut_ptr(), fname, MAXPATHL as size_t);

        let mut depth = 0;
        loop {
            // Bound the depth, to catch a loop of links pointing at each
            // other.
            depth += 1;
            if depth == 100 {
                semsg(gettext(c"E773: Symlink loop for \"%s\"".as_ptr()), fname);
                return FAIL;
            }

            let ret = readlink(tmp.as_mut_ptr(), buf, MAXPATHL as size_t - 1) as c_int;
            if ret <= 0 {
                if *__errno_location() != EINVAL && *__errno_location() != ENOENT {
                    // Some other error reading the link: keep the original.
                    return FAIL;
                }
                // Not a symlink, or it does not exist, so `tmp` is as
                // resolved as it gets. At the first level that means nothing
                // was resolved at all, and the caller keeps its own name.
                if depth == 1 {
                    return FAIL;
                }
                break;
            }
            *buf.offset(ret as isize) = NUL as c_char;

            // A relative link is relative to the directory of the name it
            // was found in, so it replaces only the tail of `tmp`.
            if path_is_absolute(buf) {
                strcpy(tmp.as_mut_ptr(), buf);
            } else {
                let tail = path_tail(tmp.as_ptr());
                if strlen(tail) + strlen(buf) >= MAXPATHL as size_t {
                    return FAIL;
                }
                strcpy(tail, buf);
            }
        }

        // Resolve the full name too, so that opening the same relative
        // symlink from two working directories still picks one swap file.
        vim_FullName(tmp.as_ptr(), buf, MAXPATHL as size_t, true)
    }
}

/// The swap file name for `fname` under the `'directory'` entry `dir_name`,
/// allocated, or null.
pub unsafe fn makeswapname(
    fname: *mut c_char,
    _ffname: *mut c_char,
    _buf: *mut buf_T,
    dir_name: *mut c_char,
) -> *mut c_char {
    unsafe {
        // Expand a symlink, so that the swap file goes with the actual file
        // rather than with the link.
        let mut fname_buf: [c_char; MAXPATHL as usize] = [0; MAXPATHL as usize];
        let fname_res = if resolve_symlink(fname, fname_buf.as_mut_ptr()) == OK {
            fname_buf.as_mut_ptr()
        } else {
            fname
        };

        let len = strlen(dir_name) as usize;
        let end = dir_name.add(len);
        if after_pathsep(dir_name, end) != 0
            && len > 1
            && *end.offset(-1) as c_int == *end.offset(-2) as c_int
        {
            // Ends with "//": the swap file's name encodes the full path.
            let mut r = core::ptr::null_mut();
            let s = make_percent_swname(dir_name, end, fname_res);
            if !s.is_null() {
                r = modname(s, c".swp".as_ptr(), false);
                xfree(s.cast());
            }
            return r;
        }

        // A swap file in the file's own directory gets a leading '.'.
        let r = modname(
            fname_res,
            c".swp".as_ptr(),
            *dir_name as c_int == '.' as c_int && *dir_name.offset(1) as c_int == NUL,
        );
        if r.is_null() {
            return core::ptr::null_mut(); // out of memory
        }
        let s = get_file_in_dir(r, dir_name);
        xfree(r.cast());
        s
    }
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
    unsafe {
        let tail = path_tail(fname);
        if *dname as c_int == '.' as c_int && *dname.offset(1) as c_int == NUL {
            xstrdup(fname)
        } else if *dname as c_int == '.' as c_int && vim_ispathsep(*dname.offset(1) as c_int) {
            if tail == fname {
                // No path in front of the file name.
                concat_fnames(dname.offset(2), tail, true)
            } else {
                let save_char = *tail;
                *tail = NUL as c_char;
                let t = concat_fnames(fname, dname.offset(2), true);
                *tail = save_char;
                let retval = concat_fnames(t, tail, true);
                xfree(t.cast());
                retval
            }
        } else {
            concat_fnames(dname, tail, true)
        }
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
    unsafe {
        assert!(!(*buf).b_fname.is_null());

        emsg(gettext(c"E325: ATTENTION".as_ptr()));
        kv_puts(msg, c"Found a swap file by the name \"");
        kv_do_printf(msg, c"%s\"\n".as_ptr(), fhname);
        let swap_mtime = swapfile_info(fname, msg);
        kv_puts(msg, c"While opening file \"");
        kv_do_printf(msg, c"%s\"\n".as_ptr(), (*buf).b_fname);

        let mut file_info: FileInfo = core::mem::zeroed();
        if !os_fileinfo((*buf).b_fname, &raw mut file_info) {
            kv_puts(msg, c"      CANNOT BE FOUND");
        } else {
            kv_puts(msg, c"             dated: ");
            let x = file_info.stat.st_mtim.tv_sec as time_t;
            let mut ctime_buf: [c_char; 50] = [0; 50];
            kv_do_printf(msg, c"%s".as_ptr(), os_ctime_r(x, &mut ctime_buf, true));
            if swap_mtime != 0 && x > swap_mtime {
                kv_puts(msg, c"      NEWER than swap file!\n");
            }
        }

        // Some of these are long, to leave room for translation.
        kv_puts(
            msg,
            c"\n(1) Another program may be editing the same file.  If this is the case,\n    be careful not to end up with two different instances of the same\n    file when making changes.  Quit, or continue with caution.\n",
        );
        kv_puts(msg, c"(2) An edit session for this file crashed.\n");
        kv_puts(
            msg,
            c"    If this is the case, use \":recover\" or \"nvim -r ",
        );
        kv_do_printf(msg, c"%s".as_ptr(), (*buf).b_fname);
        kv_puts(
            msg,
            c"\"\n    to recover the changes (see \":help recovery\").\n",
        );
        kv_puts(msg, c"    If you did this already, delete the swap file \"");
        kv_do_printf(msg, c"%s".as_ptr(), fname);
        kv_puts(msg, c"\"\n    to avoid this message.\n");
    }
}

/// Fire the `SwapExists` autocommands and read the choice they left in
/// `v:swapchoice`.
unsafe fn do_swapexists(buf: *mut buf_T, fname: *mut c_char) -> sea_choice_T {
    unsafe {
        set_vim_var_string(VV_SWAPNAME, fname, -1);
        set_vim_var_string(VV_SWAPCHOICE, core::ptr::null(), -1);

        // `<afile>` is the file being edited. Changing directory is not
        // allowed from here.
        *allbuf_lock.ptr() += 1;
        apply_autocmds(
            EVENT_SWAPEXISTS,
            (*buf).b_fname,
            core::ptr::null_mut(),
            false,
            core::ptr::null_mut(),
        );
        *allbuf_lock.ptr() -= 1;

        set_vim_var_string(VV_SWAPNAME, core::ptr::null(), -1);

        match *get_vim_var_str(VV_SWAPCHOICE) as u8 {
            b'o' => SEA_CHOICE_READONLY,
            b'e' => SEA_CHOICE_EDIT,
            b'r' => SEA_CHOICE_RECOVER,
            b'd' => SEA_CHOICE_DELETE,
            b'q' => SEA_CHOICE_QUIT,
            b'a' => SEA_CHOICE_ABORT,
            _ => SEA_CHOICE_NONE,
        }
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
    unsafe {
        // Only worth a word if the swap file belongs to *this* file, the
        // buffer was not already recovered, and 'shortmess' allows it.
        if swapfile_is_for_other_file(buf, fname)
            || (*curbuf.get()).b_flags & BF_RECOVERED != 0
            || !vim_strchr(p_shm.get(), SHM_ATTENTION as c_int).is_null()
        {
            return false;
        }

        let mut choice = SEA_CHOICE_NONE;

        // Deleting it is safe when the file exists and the swap file records
        // no changes and looks intact.
        if os_path_exists((*buf).b_fname) && swapfile_unchanged(fname) {
            choice = SEA_CHOICE_DELETE;
            if p_verbose.get() > 0 {
                verb_msg(gettext(
                    c"Found a swap file that is not useful, deleting it".as_ptr(),
                ));
            }
        }

        // A SwapExists autocommand gets the next word, if the caller is in a
        // position to honour what it says. It may still decline (0) and leave
        // the question to the user.
        if choice == SEA_CHOICE_NONE
            && swap_exists_action.get() != SEA_NONE
            && has_autocmd(EVENT_SWAPEXISTS, buf_fname, buf)
        {
            choice = do_swapexists(buf, fname);
        }
        if choice == SEA_CHOICE_NONE && swap_exists_action.get() == SEA_READONLY {
            choice = SEA_CHOICE_READONLY;
        }

        // Set by attention_message -> swapfile_info, below.
        proc_running.set(0);
        if choice == SEA_CHOICE_NONE {
            choice = ask_about_swapfile(buf, fname);
        }

        match choice {
            SEA_CHOICE_READONLY => (*buf).b_p_ro = true_0,
            SEA_CHOICE_RECOVER => swap_exists_action.set(SEA_RECOVER),
            SEA_CHOICE_DELETE => {
                os_remove(fname);
            }
            SEA_CHOICE_QUIT => swap_exists_action.set(SEA_QUIT),
            SEA_CHOICE_ABORT => {
                swap_exists_action.set(SEA_QUIT);
                got_int.set(true);
            }
            SEA_CHOICE_NONE => {
                msg_puts(c"\n".as_ptr());
                if msg_silent.get() == 0 {
                    need_wait_return.set(true); // call wait_return() later
                }
            }
            // SEA_CHOICE_EDIT: use the file as it is.
            _ => {}
        }

        // If the swap file was deleted, this name can be used after all.
        choice != SEA_CHOICE_NONE && !os_path_exists(fname)
    }
}

/// Show the ATTENTION message, as a dialog if the caller can act on an
/// answer and as a warning otherwise.
unsafe fn ask_about_swapfile(buf: *mut buf_T, fname: *mut c_char) -> sea_choice_T {
    unsafe {
        let mut choice = SEA_CHOICE_NONE;
        *no_wait_return.ptr() += 1;

        // kv_resize(msg, IOSIZE): a screenful before the first realloc.
        let mut msg: StringBuilder = KV_INITIAL_VALUE;
        msg.capacity = 1024 + 1;
        msg.items = xrealloc(msg.items.cast(), msg.capacity).cast();

        let fhname = home_replace_save(core::ptr::null_mut(), fname);
        attention_message(buf, fname, fhname, &raw mut msg);

        // A 'q' typed at the more-prompt must not interrupt loading the
        // file, and a "simalt ~x" in the vimrc must not answer the prompt
        // below.
        got_int.set(false);
        flush_buffers(FLUSH_TYPEAHEAD);

        if swap_exists_action.get() != SEA_NONE {
            kv_puts(&raw mut msg, c"Swap file \"");
            kv_do_printf(&raw mut msg, c"%s".as_ptr(), fhname);
            kv_puts(&raw mut msg, c"\" already exists!");
            // "Delete it" is not offered while the owning process is alive.
            let run_but =
                gettext(c"&Open Read-Only\n&Edit anyway\n&Recover\n&Quit\n&Abort".as_ptr());
            let but = gettext(
                c"&Open Read-Only\n&Edit anyway\n&Recover\n&Delete it\n&Quit\n&Abort".as_ptr(),
            );
            let running = proc_running.get() != 0;
            choice = do_dialog(
                VIM_WARNING as c_int,
                gettext(c"VIM - ATTENTION".as_ptr()),
                msg.items,
                if running { run_but } else { but },
                1,
                core::ptr::null(),
                false_0,
            ) as sea_choice_T;
            // Compensate for the missing "Delete it" button.
            choice = choice.wrapping_add((running && choice >= 4) as sea_choice_T);
            // Pretend the screen did not scroll; it needs a redraw anyway.
            msg_reset_scroll();
        } else {
            let mut need_clear = false;
            msg_ext_set_kind(c"wmsg".as_ptr());
            msg_multiline(
                String_0 {
                    data: msg.items,
                    size: msg.size,
                },
                0,
                false,
                false,
                &raw mut need_clear,
            );
        }

        *no_wait_return.ptr() -= 1;
        xfree(msg.items.cast()); // kv_destroy(msg)
        xfree(fhname.cast());
        choice
    }
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
    old_fname: *mut c_char,
    found_existing_dir: *mut bool,
) -> *mut c_char {
    unsafe {
        let buf_fname = (*buf).b_fname;

        // Isolate one directory name out of *dirp.
        let dir_len = strlen(*dirp) + 1;
        let dir_name = xmalloc(dir_len) as *mut c_char;
        copy_option_part(dirp, dir_name, dir_len, c",".as_ptr().cast_mut());

        let mut fname = makeswapname(buf_fname, (*buf).b_ffname, buf, dir_name);
        loop {
            if fname.is_null() {
                break; // out of memory
            }
            let n = strlen(fname) as usize;
            if n == 0 {
                // Safety check.
                xfree(fname.cast());
                fname = core::ptr::null_mut();
                break;
            }

            // Is the name taken? A swap file that is a symbolic link is most
            // likely a symlink attack, so the link itself counts as taken.
            let mut file_info: FileInfo = core::mem::zeroed();
            if !os_fileinfo_link(fname, &raw mut file_info) {
                break;
            }
            // A name this buffer already owns is free for it to keep.
            if !old_fname.is_null() && path_fnamecmp(fname, old_fname) == 0 {
                break;
            }

            // The name is taken. On the first try — the plain ".swp" — that
            // means a real swap file, and it is worth telling the user about
            // unless we are recovering, have no file name, are in a help file
            // or in a dummy buffer.
            if *fname.offset(n as isize - 2) as u8 == b'w'
                && *fname.offset(n as isize - 1) as u8 == b'p'
                && !recoverymode.get()
                && !buf_fname.is_null()
                && !(*buf).b_help
                && (*buf).b_flags & BF_DUMMY == 0
                && resolve_swapfile_clash(buf, fname, buf_fname)
            {
                break;
            }

            // Permute the extension to find a name nothing is using: first
            // count the last character down (".swo", ".swn", …), and when
            // that runs out the one before it (".svz", ".suz", …). Both can
            // happen with many Nvims editing one file, including "No Name"
            // buffers.
            if *fname.offset(n as isize - 1) as u8 == b'a' {
                if *fname.offset(n as isize - 2) as u8 == b'a' {
                    // ".saa": tried enough, give up.
                    emsg(gettext(c"E326: Too many swap files found".as_ptr()));
                    xfree(fname.cast());
                    fname = core::ptr::null_mut();
                    break;
                }
                *fname.offset(n as isize - 2) -= 1;
                *fname.offset(n as isize - 1) = b'z' as c_char + 1;
            }
            *fname.offset(n as isize - 1) -= 1;
        }

        if os_isdir(dir_name) {
            *found_existing_dir = true;
        } else if !*found_existing_dir && **dirp as c_int == NUL {
            // The last entry in 'directory' is created on demand.
            let mut failed_dir = core::ptr::null_mut();
            let ret = os_mkdir_recurse(dir_name, 0o755, &raw mut failed_dir, core::ptr::null_mut());
            if ret != 0 {
                semsg(
                    gettext(
                        c"E303: Unable to create directory \"%s\" for swap file, recovery impossible: %s"
                            .as_ptr(),
                    ),
                    failed_dir,
                    uv_strerror(ret),
                );
                xfree(failed_dir.cast());
            }
        }

        xfree(dir_name.cast());
        fname
    }
}
