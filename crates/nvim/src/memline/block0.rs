//! Block zero: the swap file's first page.
//!
//! Everything else in a swap file is addressed by block number; block zero is
//! found at offset zero and says what the rest means — the page size, the magic
//! numbers that prove the endianness and type sizes match, and the name,
//! timestamp and inode of the file being edited. `swapfile_info` and its
//! friends read one back to describe it, which is all the `swapinfo()` builtin
//! and the ATTENTION message do.
//!
//! `long_to_char`/`char_to_long` are the little-endian pair every number in
//! block zero goes through.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_double, c_int, c_long};

use super::*;
use crate::types::{FAIL, MAXPATHL, NUL, OK};

/// Why a swap file yielded no block zero.
pub(crate) enum NoBlock {
    CannotOpen,
    CannotRead,
}

impl ZeroBlock {
    /// Upstream keeps two more fields in the last two bytes of the name:
    /// `#define b0_dirty b0_fname[B0_FNAME_SIZE_ORG - 1]` and
    /// `#define b0_flags b0_fname[B0_FNAME_SIZE_ORG - 2]`. That is why the
    /// name itself is only ever written up to `B0_FNAME_SIZE_NOCRYPT`.
    const DIRTY: usize = B0_FNAME_SIZE_ORG as usize - 1;
    const FLAGS: usize = B0_FNAME_SIZE_ORG as usize - 2;

    /// [`B0_SAME_DIR`], [`B0_HAS_FENC`] and the `'fileformat'` in
    /// [`B0_FF_MASK`].
    pub(crate) fn flags(&self) -> c_int {
        self.b0_fname[Self::FLAGS] as c_int
    }

    pub(crate) fn set_flags(&mut self, flags: c_int) {
        self.b0_fname[Self::FLAGS] = flags as c_char;
    }

    pub(crate) fn set_flag(&mut self, flag: c_int, on: bool) {
        let flags = self.flags();
        self.set_flags(if on { flags | flag } else { flags & !flag });
    }

    /// Whether the buffer had unsaved changes when this was last written.
    /// It is what makes `:recover` worth offering.
    pub(crate) fn dirty(&self) -> bool {
        self.b0_fname[Self::DIRTY] != 0
    }

    pub(crate) fn set_dirty(&mut self, dirty: bool) {
        self.b0_fname[Self::DIRTY] = if dirty { B0_DIRTY } else { 0 } as c_char;
    }

    /// The PID recorded by the Nvim that last wrote this block.
    pub(crate) fn pid(&self) -> c_long {
        b0_read_number(&self.b0_pid)
    }

    /// Read the block zero of the swap file `path`. A short read leaves
    /// nothing usable, which is why that is reported rather than returning a
    /// half-filled block; the two failures are told apart because
    /// `swapinfo()` and the ATTENTION message report them differently.
    pub(crate) unsafe fn read(path: *const c_char) -> Result<Self, NoBlock> {
        // SAFETY: `path` is a NUL-terminated path, and this is a
        // plain-old-data struct, so zero is a valid value for it and so is
        // whatever `read_eintr` puts in its `size_of::<Self>()` bytes.
        unsafe {
            let fd = os_open(path, O_RDONLY, 0);
            if fd < 0 {
                return Err(NoBlock::CannotOpen);
            }
            let mut b0: Self = core::mem::zeroed();
            let got = read_eintr(fd, (&raw mut b0).cast(), size_of::<Self>());
            close(fd);
            if got as usize == size_of::<Self>() {
                Ok(b0)
            } else {
                Err(NoBlock::CannotRead)
            }
        }
    }
}

/// Record the file's timestamp in the swap file, after it has been written.
pub unsafe fn ml_timestamp(buf: *mut buf_T) {
    unsafe { ml_upd_block0(buf, UB_FNAME) }
}

/// Whether the two bytes that identify a swap file are at the head of this
/// block. They are the first thing read back, and a mismatch means the file
/// is not a swap file at all.
pub(crate) fn ml_check_b0_id(b0: &ZeroBlock) -> bool {
    b0.b0_id[0] as c_int == BLOCK0_ID0 as c_int && b0.b0_id[1] as c_int == BLOCK0_ID1 as c_int
}

/// Whether every string in the block is terminated inside its own field.
/// A swap file is read from disk, so nothing may be assumed about it: an
/// unterminated name would be read past by everything downstream.
pub(crate) fn ml_check_b0_strings(b0: &ZeroBlock) -> bool {
    b0.b0_version.contains(&0)
        && b0.b0_uname.contains(&0)
        && b0.b0_hname.contains(&0)
        && b0.b0_fname[..B0_FNAME_SIZE_CRYPT as usize].contains(&0)
}

/// Bring block zero up to date with the buffer: either the file name and
/// timestamp ([`UB_FNAME`]), or the "swap file is beside the file" flag
/// ([`UB_SAME_DIR`]).
pub(crate) unsafe fn ml_upd_block0(buf: *mut buf_T, what: upd_block0_T) {
    unsafe {
        let mfp = (*buf).b_ml.ml_mfp;
        if mfp.is_null() {
            return;
        }
        let hp = mf_get(mfp, 0, 1);
        if hp.is_null() {
            return;
        }

        let b0p = (*hp).bh_data as *mut ZeroBlock;
        if !ml_check_b0_id(&*b0p) {
            iemsg(gettext(
                c"E304: ml_upd_block0(): Didn't get block 0??".as_ptr(),
            ));
        } else if what == UB_FNAME {
            set_b0_fname(b0p, buf);
        } else {
            set_b0_dir_flag(b0p, buf);
        }
        mf_put(mfp, hp, true, false);
    }
}

/// Write the file's name, timestamp and inode into block zero, and set
/// `buf->b_mtime` from the same `stat`.
///
/// Must not use the caller's name buffer: some of them still hold it.
pub(crate) unsafe fn set_b0_fname(b0p: *mut ZeroBlock, buf: *mut buf_T) {
    unsafe {
        if (*buf).b_ffname.is_null() {
            (*b0p).b0_fname[0] = NUL as c_char;
        } else {
            // A file under the current user's home directory is recorded as
            // "~user/...", so that the same file opened from another machine
            // over a network is still recognised as the same file.
            // `home_replace` writes "~/", and the user name is spliced in
            // after the tilde.
            let name = &mut (*b0p).b0_fname;
            home_replace(
                core::ptr::null::<buf_T>(),
                (*buf).b_ffname,
                name.as_mut_ptr(),
                B0_FNAME_SIZE_CRYPT as size_t,
                true,
            );
            if name[0] as c_int == '~' as c_int {
                let mut uname: [c_char; B0_UNAME_SIZE as usize] = [0; B0_UNAME_SIZE as usize];
                let named = os_get_username(uname.as_mut_ptr(), B0_UNAME_SIZE as size_t);
                let ulen = strlen(uname.as_ptr()) as usize;
                // `flen` counts the bytes after the tilde *including* the
                // terminator, which is exactly what has to shift right.
                let flen = strlen(name.as_ptr()) as usize;
                if named == FAIL || ulen + flen > B0_FNAME_SIZE_CRYPT as usize - 1 {
                    // No user name, or no room for one: keep the path as it
                    // was given.
                    xstrlcpy(
                        name.as_mut_ptr(),
                        (*buf).b_ffname,
                        B0_FNAME_SIZE_CRYPT as size_t,
                    );
                } else {
                    name.copy_within(1..1 + flen, 1 + ulen);
                    name[1..1 + ulen].copy_from_slice(&uname[..ulen]);
                }
            }

            let mut file_info: FileInfo = core::mem::zeroed();
            if os_fileinfo((*buf).b_ffname, &raw mut file_info) {
                b0_store_number(file_info.stat.st_mtim.tv_sec, &mut (*b0p).b0_mtime);
                b0_store_number(
                    os_fileinfo_inode(&raw mut file_info) as c_long,
                    &mut (*b0p).b0_ino,
                );
                buf_store_file_info(buf, &raw mut file_info);
                (*buf).b_mtime_read = (*buf).b_mtime;
                (*buf).b_mtime_read_ns = (*buf).b_mtime_ns;
            } else {
                b0_store_number(0, &mut (*b0p).b0_mtime);
                b0_store_number(0, &mut (*b0p).b0_ino);
                (*buf).b_mtime = 0;
                (*buf).b_mtime_ns = 0;
                (*buf).b_mtime_read = 0;
                (*buf).b_mtime_read_ns = 0;
                (*buf).b_orig_size = 0;
                (*buf).b_orig_mode = 0;
            }
        }

        // Upstream passes `curbuf` here, not `buf`. Preserved: the two are
        // the same for every reachable caller.
        add_b0_fenc(b0p, curbuf.get());
    }
}

/// Record whether the file and its swap file are in the same directory.
///
/// Fail safe: anything short of proof leaves the flag clear.
pub(crate) unsafe fn set_b0_dir_flag(b0p: *mut ZeroBlock, buf: *mut buf_T) {
    unsafe {
        let same = same_directory(mf_fname((*buf).b_ml.ml_mfp).cast_mut(), (*buf).b_ffname);
        (*b0p).set_flag(B0_SAME_DIR, same);
    }
}

/// Append the buffer's `'fileencoding'` to block zero, if it fits.
///
/// It goes at the *end* of the name field with a NUL in front of it, so a
/// reader that does not know about [`B0_HAS_FENC`] still sees a terminated
/// name and never reaches the encoding.
pub(crate) unsafe fn add_b0_fenc(b0p: *mut ZeroBlock, buf: *mut buf_T) {
    unsafe {
        let size = B0_FNAME_SIZE_NOCRYPT as usize;
        let fenc = (*buf).b_p_fenc;
        let n = strlen(fenc) as usize;
        let name = &mut (*b0p).b0_fname;
        let fits = strlen(name.as_ptr()) as usize + n < size;
        if fits {
            let at = size - n;
            core::ptr::copy_nonoverlapping(fenc, name[at..].as_mut_ptr(), n);
            name[at - 1] = NUL as c_char;
        }
        (*b0p).set_flag(B0_HAS_FENC, fits);
    }
}

/// The PID of the Nvim that owns this swap file, if that process is still
/// running; zero if it is not, or cannot be.
///
/// "Cannot be" is the reboot case: a swap file older than the system's uptime
/// was written by a process from before the boot, so whatever PID it names
/// belongs to somebody else now.
pub(crate) fn swapfile_proc_running(b0: &ZeroBlock, swap_fname: *const c_char) -> c_int {
    // SAFETY: `swap_fname` is a NUL-terminated path; the two out-parameters
    // are locals.
    unsafe {
        let mut st: FileInfo = core::mem::zeroed();
        let mut uptime: c_double = 0.;
        if os_fileinfo(swap_fname, &raw mut st)
            && uv_uptime(&raw mut uptime) == 0
            // Unsigned, and upstream lets it wrap: on a machine whose clock
            // predates its uptime the subtraction is meant to come out huge.
            && (st.stat.st_mtim.tv_sec as Timestamp) < os_time().wrapping_sub(uptime as Timestamp)
        {
            return 0;
        }
        let pid = b0.pid() as c_int;
        if os_proc_running(pid) { pid } else { 0 }
    }
}

/// Describe a swap file for the `swapinfo()` builtin: the block-zero fields
/// if they can be read and make sense, an `error` key saying why not if they
/// cannot.
pub unsafe fn swapfile_dict(fname: *const c_char, d: *mut dict_T) {
    unsafe {
        let error = |text: &CStr| dict_add_str(d, c"error", text.as_ptr(), -1);
        match ZeroBlock::read(fname) {
            Err(NoBlock::CannotOpen) => error(c"Cannot open file"),
            Err(NoBlock::CannotRead) => error(c"Cannot read file"),
            Ok(b0) if !ml_check_b0_id(&b0) => error(c"Not a swap file"),
            Ok(b0) if b0_magic_wrong(&b0) => error(c"Magic number mismatch"),
            Ok(b0) => {
                // The strings are reported at their full field width rather
                // than up to the NUL: this is the raw block, and a caller
                // inspecting a damaged swap file wants what is really there.
                dict_add_str(d, c"version", b0.b0_version.as_ptr(), 10);
                dict_add_str(d, c"user", b0.b0_uname.as_ptr(), B0_UNAME_SIZE as c_int);
                dict_add_str(d, c"host", b0.b0_hname.as_ptr(), B0_HNAME_SIZE as c_int);
                dict_add_str(
                    d,
                    c"fname",
                    b0.b0_fname.as_ptr(),
                    B0_FNAME_SIZE_ORG as c_int,
                );
                dict_add_nr(d, c"pid", swapfile_proc_running(&b0, fname) as varnumber_T);
                dict_add_nr(d, c"mtime", b0_read_number(&b0.b0_mtime) as varnumber_T);
                dict_add_nr(d, c"dirty", b0.dirty() as varnumber_T);
                dict_add_nr(d, c"inode", b0_read_number(&b0.b0_ino) as varnumber_T);
            }
        }
    }
}

/// `tv_dict_add_*` take the key and its length separately; upstream spells
/// that pair `S_LEN(key)`. A negative `len` means "up to the NUL".
unsafe fn dict_add_str(d: *mut dict_T, key: &CStr, val: *const c_char, len: c_int) {
    unsafe { tv_dict_add_str_len(d, key.as_ptr(), key.count_bytes(), val, len) };
}

unsafe fn dict_add_nr(d: *mut dict_T, key: &CStr, nr: varnumber_T) {
    unsafe { tv_dict_add_nr(d, key.as_ptr(), key.count_bytes(), nr) };
}

/// Describe a swap file in the ATTENTION message and in `:recover`'s listing.
///
/// Returns the swap file's own timestamp, or 0 if it could not be stat'ed.
pub(crate) unsafe fn swapfile_info(fname: *const c_char, msg: *mut StringBuilder) -> time_t {
    unsafe {
        debug_assert!(!fname.is_null());
        let mut x: time_t = 0;

        // The swap file's date, and on unix the name of whoever owns it.
        let mut file_info: FileInfo = core::mem::zeroed();
        if os_fileinfo(fname, &raw mut file_info) {
            let mut uname: [c_char; B0_UNAME_SIZE as usize] = [0; B0_UNAME_SIZE as usize];
            if os_get_uname(
                file_info.stat.st_uid as uv_uid_t,
                uname.as_mut_ptr(),
                B0_UNAME_SIZE as size_t,
            ) == OK
            {
                kv_do_printf(
                    msg,
                    c"%s%s".as_ptr(),
                    gettext(c"          owned by: ".as_ptr()),
                    uname.as_ptr(),
                );
                kv_puts(msg, c"   dated: ");
            } else {
                kv_puts(msg, c"             dated: ");
            }
            x = file_info.stat.st_mtim.tv_sec as time_t;
            // Hopefully enough for every language.
            let mut ctime_buf: [c_char; 100] = [0; 100];
            kv_do_printf(msg, c"%s".as_ptr(), os_ctime_r(x, &mut ctime_buf, true));
        }

        // What the swap file says about the file it belongs to.
        match ZeroBlock::read(fname) {
            Err(NoBlock::CannotOpen) => kv_puts(msg, c"         [cannot be opened]"),
            Err(NoBlock::CannotRead) => kv_puts(msg, c"         [cannot be read]"),
            Ok(b0) if strncmp(b0.b0_version.as_ptr(), c"VIM 3.0".as_ptr(), 7) == 0 => {
                kv_puts(msg, c"         [from Vim version 3.0]");
            }
            Ok(b0) if !ml_check_b0_id(&b0) => {
                kv_puts(msg, c"         [does not look like a Nvim swap file]");
            }
            Ok(b0) if !ml_check_b0_strings(&b0) => {
                kv_puts(msg, c"         [garbled strings (not nul terminated)]");
            }
            Ok(b0) => {
                kv_puts(msg, c"         file name: ");
                if b0.b0_fname[0] as c_int == NUL {
                    kv_puts(msg, c"[No Name]");
                } else {
                    kv_do_printf(msg, c"%s".as_ptr(), b0.b0_fname.as_ptr());
                }

                kv_puts(msg, c"\n          modified: ");
                kv_puts(msg, if b0.dirty() { c"YES" } else { c"no" });

                if b0.b0_uname[0] as c_int != NUL {
                    kv_puts(msg, c"\n         user name: ");
                    kv_do_printf(msg, c"%s".as_ptr(), b0.b0_uname.as_ptr());
                }

                if b0.b0_hname[0] as c_int != NUL {
                    // Only the second of the two gets a line of its own.
                    if b0.b0_uname[0] as c_int != NUL {
                        kv_puts(msg, c"   host name: ");
                    } else {
                        kv_puts(msg, c"\n         host name: ");
                    }
                    kv_do_printf(msg, c"%s".as_ptr(), b0.b0_hname.as_ptr());
                }

                if b0.pid() != 0 {
                    kv_puts(msg, c"\n        process ID: ");
                    kv_do_printf(msg, c"%d".as_ptr(), b0.pid() as c_int);
                    // Read back by `findswapname` to decide whether this
                    // is a crash to recover from or a live second editor.
                    proc_running.set(swapfile_proc_running(&b0, fname));
                    if proc_running.get() != 0 {
                        kv_puts(msg, c" (STILL RUNNING)");
                    }
                }

                if b0_magic_wrong(&b0) {
                    kv_puts(msg, c"\n         [not usable on this computer]");
                }
            }
        }
        kv_puts(msg, c"\n");
        x
    }
}

/// Append a translated message. Upstream hands it to `kv_printf` as the
/// format string, so a `%` in a translation is a directive there too;
/// preserved.
pub(crate) unsafe fn kv_puts(msg: *mut StringBuilder, text: &CStr) {
    unsafe { kv_do_printf(msg, gettext(text.as_ptr())) };
}

/// Whether this swap file can be deleted without losing anything: it is
/// intact, records no unsaved changes, and the process that wrote it died on
/// this same host.
pub(crate) unsafe fn swapfile_unchanged(fname: *const c_char) -> bool {
    unsafe {
        if !os_path_exists(fname) {
            return false;
        }
        let Ok(mut b0) = ZeroBlock::read(fname) else {
            return false;
        };

        // The ID and magic number must be right, and there must be nothing
        // unsaved in it.
        let mut ret = ml_check_b0_id(&b0) && !b0_magic_wrong(&b0) && !b0.dirty();

        // The host name must be known and be this host, otherwise comparing
        // the PID against this machine's processes is meaningless.
        if b0.b0_hname[0] as c_int == NUL {
            ret = false;
        } else {
            let mut hostname: [c_char; B0_HNAME_SIZE as usize] = [0; B0_HNAME_SIZE as usize];
            os_get_hostname(hostname.as_mut_ptr(), B0_HNAME_SIZE as size_t);
            hostname[B0_HNAME_SIZE as usize - 1] = NUL as c_char;
            // In case of corruption.
            b0.b0_hname[B0_HNAME_SIZE as usize - 1] = NUL as c_char;
            if strcasecmp(b0.b0_hname.as_ptr(), hostname.as_ptr()) != 0 {
                ret = false;
            }
        }

        // The process must be known, and must not be running.
        if b0.pid() == 0 || swapfile_proc_running(&b0, fname) != 0 {
            ret = false;
        }

        // The user is deliberately not checked: it has no bearing on whether
        // the swap file is still worth anything.

        ret
    }
}

/// Whether the swap file `fname` was left behind for a file *other* than the
/// one `buf` is editing — the common case when `'directory'` gathers every
/// swap file into one place.
///
/// Also publishes [`proc_running`], which the dialog below reads.
pub(crate) unsafe fn swapfile_is_for_other_file(buf: *mut buf_T, fname: *mut c_char) -> bool {
    // The expanded name out of block zero; upstream shares `NameBuff`, and
    // `set_b0_fname` above documents that its callers hold it.
    let mut expanded = [0 as c_char; MAXPATHL as usize];
    unsafe {
        let mut differ = false;
        if let Ok(mut b0) = ZeroBlock::read(fname) {
            proc_running.set(swapfile_proc_running(&b0, fname));

            // When the swap file sits in the same directory as the file, the
            // directory names need not agree — they can be reached through
            // different mount points — so only the tails are compared.
            if b0.flags() & B0_SAME_DIR == 0
                || path_fnamecmp(path_tail((*buf).b_ffname), path_tail(b0.b0_fname.as_ptr())) != 0
                || !same_directory(fname, (*buf).b_ffname)
            {
                // The name in the swap file may be "~user/path/file".
                // Symlinks can point at the same file under two names, so the
                // inode has the last word.
                expand_env(b0.b0_fname.as_mut_ptr(), expanded.as_mut_ptr(), MAXPATHL);
                let (name, ino) = (expanded.as_mut_ptr(), b0_read_number(&b0.b0_ino));
                differ = files_differ((*buf).b_ffname, name, ino);
            }
        }
        differ
    }
}

/// Whether the four magic numbers say this swap file was written by a
/// different build: they are one value per width, so a change of
/// endianness or of a type's size shows up as a mismatch.
pub(crate) fn b0_magic_wrong(b0: &ZeroBlock) -> bool {
    b0.b0_magic_long != B0_MAGIC_LONG as c_long
        || b0.b0_magic_int != B0_MAGIC_INT as c_int
        // Truncated to 16 bits on the way in, so compare it truncated.
        || b0.b0_magic_short != B0_MAGIC_SHORT as int16_t
        || b0.b0_magic_char as c_int != B0_MAGIC_CHAR as c_int
}

/// Whether the file being edited and the file the swap file names are
/// different files.
///
/// Inodes first, since the *name* in block zero may be stale or may not even
/// be a valid path on this machine; the inode recorded in block zero is the
/// last resort for the swap file's side, and only its low 32 bits survived
/// the format. Where neither inode is known the full paths are compared
/// instead, and where even those cannot be resolved the answer is a guess:
/// "different", unless neither file exists at all, in which case the names
/// as given decide.
///
/// `ino_block0` is the inode block zero recorded.
pub(crate) unsafe fn files_differ(
    fname_c: *mut c_char,
    fname_s: *mut c_char,
    ino_block0: c_long,
) -> bool {
    unsafe {
        let mut file_info: FileInfo = core::mem::zeroed();
        let ino_c: uint64_t = if os_fileinfo(fname_c, &raw mut file_info) {
            os_fileinfo_inode(&raw mut file_info)
        } else {
            0
        };
        let ino_s: uint64_t = if os_fileinfo(fname_s, &raw mut file_info) {
            os_fileinfo_inode(&raw mut file_info)
        } else {
            ino_block0 as uint64_t
        };
        if ino_c != 0 && ino_s != 0 {
            return ino_c != ino_s;
        }

        // One of the inodes is unknown: force a full path for each and
        // compare those instead.
        let mut buf_c: [c_char; MAXPATHL as usize] = [0; MAXPATHL as usize];
        let mut buf_s: [c_char; MAXPATHL as usize] = [0; MAXPATHL as usize];
        let ok_c = vim_full_name(fname_c, buf_c.as_mut_ptr(), MAXPATHL as size_t, true) == OK;
        let ok_s = vim_full_name(fname_s, buf_s.as_mut_ptr(), MAXPATHL as size_t, true) == OK;
        if ok_c && ok_s {
            return strcmp(buf_c.as_ptr(), buf_s.as_ptr()) != 0;
        }

        // Neither inodes nor full paths. If neither file appears to exist,
        // the names as given are all there is to go on; otherwise guess that
        // they are different.
        if ino_s == 0 && ino_c == 0 && !ok_c && !ok_s {
            return strcmp(fname_c, fname_s) != 0;
        }
        true
    }
}

/// Store a number in block zero: four bytes, little-endian.
///
/// Only the low 32 bits survive — the format has no room for more, and a
/// timestamp or inode wider than that is simply truncated, as upstream's
/// `(unsigned)n >> 8` stepping did.
pub(crate) fn b0_store_number(n: c_long, dest: &mut [c_char; 4]) {
    *dest = (n as u32).to_le_bytes().map(|b| b as c_char);
}

/// Read back a number stored by [`b0_store_number`]. The result is the
/// unsigned 32-bit value widened, never negative.
pub(crate) fn b0_read_number(src: &[c_char; 4]) -> c_long {
    u32::from_le_bytes(src.map(|b| b as u8)) as c_long
}

/// Update the flags block zero carries about the buffer — whether it has
/// unsaved changes, its `'fileformat'` and its `'fileencoding'` — and push
/// block zero alone to disk.
pub unsafe fn ml_setflags(buf: *mut buf_T) {
    unsafe {
        let mfp = (*buf).b_ml.ml_mfp;
        if mfp.is_null() {
            return;
        }
        // Only if block zero is already in memory; there is nothing here
        // worth a read for.
        let hp = mf_find(mfp, 0);
        if hp.is_null() {
            return;
        }

        let b0p = (*hp).bh_data as *mut ZeroBlock;
        (*b0p).set_dirty((*buf).b_changed != 0);
        let fileformat = (get_fileformat(buf) + 1) as uint8_t as c_int;
        (*b0p).set_flags((*b0p).flags() & !B0_FF_MASK | fileformat);
        add_b0_fenc(b0p, buf);
        (*hp).bh_flags |= BH_DIRTY;
        // Best effort: a swap file that cannot take block zero is reported
        // where it is created, and there is nothing to do about it here.
        let _ = mf_sync(mfp, MFS_ZERO as c_int);
    }
}
