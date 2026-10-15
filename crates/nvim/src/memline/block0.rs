//! Block zero: the swap file's first page.
//!
//! Everything else in a swap file is addressed by block number; block zero is
//! found at offset zero and says what the rest means — the page size, the magic
//! numbers that prove the endianness and type sizes match, and the name,
//! timestamp and inode of the file being edited. `swapfile_info` and its
//! friends read one back to describe it, which is all the `swapinfo()` builtin
//! and the ATTENTION message do.
//!
//! The fields are read and written through [`ZeroBlock`], a view over the
//! page (or over the bytes read from a file's start); numbers go through
//! [`b0_store_number`]/[`b0_read_number`], the little-endian pair every number
//! in block zero takes.

#![forbid(unsafe_code)]

use crate::cstr;
use crate::memfile::BlockId;
use crate::path::tail_index;
use core::ffi::{CStr, c_int, c_long};
use std::io::Read;

use super::*;
use crate::optionstr::OptStringRef;
use crate::os::cshim::gettext;
use crate::os::env::{expand_env_into, home_replace_in, os_hostname_into};
use crate::os::fs::file_info;
use crate::os::time::{ctime_text, uptime};
use crate::os::users::{uname_within, username_named_within};
use crate::types::{MAXPATHL, NUL};

/// Why a swap file yielded no block zero.
pub(crate) enum NoBlock {
    CannotOpen,
    CannotRead,
}

/// Read the block zero of the swap file `path`. A short read leaves nothing
/// usable, which is why that is reported rather than returning a half-filled
/// block; the two failures are told apart because `swapinfo()` and the
/// ATTENTION message report them differently.
pub(crate) fn read_block_zero(path: &CStr) -> Result<ZeroBlock<Vec<u8>>, NoBlock> {
    let path = std::path::Path::new(
        <std::ffi::OsStr as std::os::unix::ffi::OsStrExt>::from_bytes(path.to_bytes()),
    );
    let mut file = std::fs::File::open(path).map_err(|_| NoBlock::CannotOpen)?;
    let mut bytes = vec![0; B0_SIZE];
    file.read_exact(&mut bytes)
        .map_err(|_| NoBlock::CannotRead)?;
    Ok(ZeroBlock(bytes))
}

/// Record the file's timestamp in the swap file, after it has been written.
pub fn ml_timestamp(buffer: Buf) {
    ml_upd_block0(buffer, UB_FNAME)
}

/// Whether the two bytes that identify a swap file are at the head of this
/// block. They are the first thing read back, and a mismatch means the file
/// is not a swap file at all.
pub(crate) fn ml_check_b0_id<P: AsRef<[u8]>>(b0: &ZeroBlock<P>) -> bool {
    b0.id() == &[BLOCK0_ID0, BLOCK0_ID1]
}

/// Whether every string in the block is terminated inside its own field.
/// A swap file is read from disk, so nothing may be assumed about it: an
/// unterminated name would be read past by everything downstream.
pub(crate) fn ml_check_b0_strings<P: AsRef<[u8]>>(b0: &ZeroBlock<P>) -> bool {
    b0.version().contains(&0)
        && b0.uname().contains(&0)
        && b0.hname().contains(&0)
        && b0.fname()[..B0_FNAME_SIZE_CRYPT as usize].contains(&0)
}

/// Bring block zero up to date with the buffer: either the file name and
/// timestamp ([`UB_FNAME`]), or the "swap file is beside the file" flag
/// ([`UB_SAME_DIR`]).
pub(crate) fn ml_upd_block0(mut buffer: Buf, what: UpdBlock0) {
    let Some(mfp) = buffer.b_ml.ml_mfp.as_deref_mut() else {
        return;
    };
    let Some(id) = mfp.get(0, 1) else {
        return;
    };

    if !ml_check_b0_id(&ZeroBlock(buffer.b_ml.page(id))) {
        iemsg(gettext(c"E304: ml_upd_block0(): Didn't get block 0??"));
    } else if what == UB_FNAME {
        set_b0_fname(buffer, id);
    } else {
        set_b0_dir_flag(buffer, id);
    }
    buffer.b_ml.put(id, true);
}

/// The name block zero records for a file whose full name is `full`: with a
/// home directory folded to `~user/`, so that the same file opened from
/// another machine over a network is still recognised as the same file, and
/// cut to the field.
fn b0_name(full: &CStr) -> Vec<u8> {
    let room = B0_FNAME_SIZE_CRYPT as usize;
    // `home_replace` writes "~/", and the user name is spliced in after
    // the tilde.
    let name = home_replace_in(None, full, room, true).to_vec();
    if name.first() != Some(&b'~') {
        return name;
    }
    let named = username_named_within(B0_UNAME_SIZE as usize);
    match named {
        // `name.len() + 1` is the bytes after the tilde *including* the
        // terminator, which is exactly what has to shift right.
        Ok(uname) if uname.len() + name.len() < room => {
            let mut spliced = Vec::with_capacity(name.len() + uname.len());
            spliced.push(b'~');
            spliced.extend_from_slice(&uname);
            spliced.extend_from_slice(&name[1..]);
            spliced
        }
        // No user name, or no room for one: keep the path as it was given.
        _ => full.to_bytes()[..full.count_bytes().min(room - 1)].to_vec(),
    }
}

/// Write the file's name, timestamp and inode into block zero (block `id` of
/// `buffer`'s memfile), and set `buffer.b_mtime` from the same `stat`.
pub(crate) fn set_b0_fname(mut buffer: Buf, id: BlockId) {
    // A copy: the name is read before the block is written, and the block
    // is the buffer's own.
    let full = buffer.name.full().map(CStr::to_owned);
    if let Some(full) = full {
        let name = b0_name(&full);
        let info = file_info(&full);
        let (mtime, ino) = match &info {
            Some(info) => (info.stat.st_mtim.tv_sec, info.stat.st_ino as c_long),
            None => (0, 0),
        };
        let mut b0 = ZeroBlock(buffer.b_ml.page_mut(id));
        // The name, terminated; what lies past the terminator in the field
        // is left as it was, as the C string copy left it.
        let field = b0.fname_mut();
        field[..name.len()].copy_from_slice(&name);
        field[name.len()] = 0;
        b0.set_mtime(mtime);
        b0.set_ino(ino);
        match info {
            Some(info) => {
                buf_store_file_info(buffer, &info);
                buffer.b_mtime_read = buffer.b_mtime;
                buffer.b_mtime_read_ns = buffer.b_mtime_ns;
            }
            None => {
                buffer.b_mtime = 0;
                buffer.b_mtime_ns = 0;
                buffer.b_mtime_read = 0;
                buffer.b_mtime_read_ns = 0;
                buffer.b_orig_size = 0;
                buffer.b_orig_mode = 0;
            }
        }
    } else {
        ZeroBlock(buffer.b_ml.page_mut(id)).fname_mut()[0] = 0;
    }

    // Upstream passes `curbuf` here, not `buffer`. Preserved: the two are
    // the same for every reachable caller.
    let fenc = Buf::current().b_p_fenc.bytes().to_vec();
    add_b0_fenc(&mut ZeroBlock(buffer.b_ml.page_mut(id)), &fenc);
}

/// Record whether the file and its swap file are in the same directory.
///
/// Fail safe: anything short of proof leaves the flag clear.
pub(crate) fn set_b0_dir_flag(mut buffer: Buf, id: BlockId) {
    let swap_name = buffer
        .b_ml
        .ml_mfp
        .as_ref()
        .and_then(|mfp| mfp.fname())
        .map(CStr::to_owned);
    let same = match (swap_name, buffer.name.full()) {
        (Some(swap_name), Some(full)) => same_directory(&swap_name, full),
        _ => false,
    };
    ZeroBlock(buffer.b_ml.page_mut(id)).set_flag(B0_SAME_DIR, same);
}

/// Append `fenc`, a `'fileencoding'`, to block zero, if it fits.
///
/// It goes at the *end* of the name field with a NUL in front of it, so a
/// reader that does not know about [`B0_HAS_FENC`] still sees a terminated
/// name and never reaches the encoding.
pub(crate) fn add_b0_fenc(b0: &mut ZeroBlock<&mut [u8]>, fenc: &[u8]) {
    let size = B0_FNAME_SIZE_NOCRYPT as usize;
    let n = fenc.len();
    let name = b0.fname_mut();
    let fits = cstr::until_nul(&name[..]).len() + n < size;
    if fits {
        let at = size - n;
        name[at..size].copy_from_slice(fenc);
        name[at - 1] = NUL as u8;
    }
    b0.set_flag(B0_HAS_FENC, fits);
}

/// The PID of the Nvim that owns this swap file, if that process is still
/// running; zero if it is not, or cannot be.
///
/// "Cannot be" is the reboot case: a swap file older than the system's uptime
/// was written by a process from before the boot, so whatever PID it names
/// belongs to somebody else now.
pub(crate) fn swapfile_proc_running<P: AsRef<[u8]>>(b0: &ZeroBlock<P>, swap_fname: &CStr) -> c_int {
    if let (Some(st), Some(uptime)) = (file_info(swap_fname), uptime())
        // Unsigned, and upstream lets it wrap: on a machine whose clock
        // predates its uptime the subtraction is meant to come out huge.
        && (st.stat.st_mtim.tv_sec as Timestamp) < os_time().wrapping_sub(uptime as Timestamp)
    {
        return 0;
    }
    let pid = b0.pid() as c_int;
    if os_proc_running(pid) { pid } else { 0 }
}

/// Describe a swap file for the `swapinfo()` builtin: the block-zero fields
/// if they can be read and make sense, an `error` key saying why not if they
/// cannot.
pub fn swapfile_dict(fname: &CStr, d: &mut Dict) {
    let mut error = |text: &CStr| {
        let _ = d.add_str(b"error", Some(text));
    };
    match read_block_zero(fname) {
        Err(NoBlock::CannotOpen) => error(c"Cannot open file"),
        Err(NoBlock::CannotRead) => error(c"Cannot read file"),
        Ok(b0) if !ml_check_b0_id(&b0) => error(c"Not a swap file"),
        Ok(b0) if b0_magic_wrong(&b0) => error(c"Magic number mismatch"),
        Ok(b0) => {
            // The strings are reported at their full field width rather
            // than up to the NUL: this is the raw block, and a caller
            // inspecting a damaged swap file wants what is really there.
            let _ = d.add_str_len(b"version", Some(&b0.version()[..]));
            let _ = d.add_str_len(b"user", Some(&b0.uname()[..]));
            let _ = d.add_str_len(b"host", Some(&b0.hname()[..]));
            let _ = d.add_str_len(b"fname", Some(&b0.fname()[..]));
            let pid = swapfile_proc_running(&b0, fname);
            let _ = d.add_number(b"pid", pid as VarNumber);
            let _ = d.add_number(b"mtime", b0.mtime() as VarNumber);
            let _ = d.add_number(b"dirty", b0.dirty() as VarNumber);
            let _ = d.add_number(b"inode", b0.ino() as VarNumber);
        }
    }
}

/// Describe a swap file in the ATTENTION message and in `:recover`'s listing.
///
/// Returns the swap file's own timestamp, or 0 if it could not be stat'ed.
pub(crate) fn swapfile_info(fname: &CStr, msg: &mut Vec<u8>) -> time_t {
    let mut x: time_t = 0;

    // The swap file's date, and on unix the name of whoever owns it.
    if let Some(info) = file_info(fname) {
        if let Ok(owner) = uname_within(info.stat.st_uid as uv_uid_t, B0_UNAME_SIZE as usize) {
            push_tr(msg, c"          owned by: ");
            msg.extend_from_slice(&owner);
            push_tr(msg, c"   dated: ");
        } else {
            push_tr(msg, c"             dated: ");
        }
        x = info.stat.st_mtim.tv_sec as time_t;
        // Hopefully enough for every language.
        msg.extend_from_slice(&ctime_text(x, 100, true));
    }

    // What the swap file says about the file it belongs to.
    match read_block_zero(fname) {
        Err(NoBlock::CannotOpen) => push_tr(msg, c"         [cannot be opened]"),
        Err(NoBlock::CannotRead) => push_tr(msg, c"         [cannot be read]"),
        Ok(b0) if b0.version().starts_with(b"VIM 3.0") => {
            push_tr(msg, c"         [from Vim version 3.0]");
        }
        Ok(b0) if !ml_check_b0_id(&b0) => {
            push_tr(msg, c"         [does not look like a Nvim swap file]");
        }
        Ok(b0) if !ml_check_b0_strings(&b0) => {
            push_tr(msg, c"         [garbled strings (not nul terminated)]");
        }
        Ok(b0) => {
            // Every string below is terminated inside its own field:
            // `ml_check_b0_strings` above is what proves it.
            push_tr(msg, c"         file name: ");
            if b0.fname()[0] == 0 {
                push_tr(msg, c"[No Name]");
            } else {
                msg.extend_from_slice(cstr::until_nul(b0.fname()));
            }

            push_tr(msg, c"\n          modified: ");
            push_tr(msg, if b0.dirty() { c"YES" } else { c"no" });

            if b0.uname()[0] != 0 {
                push_tr(msg, c"\n         user name: ");
                msg.extend_from_slice(cstr::until_nul(b0.uname()));
            }

            if b0.hname()[0] != 0 {
                // Only the second of the two gets a line of its own.
                if b0.uname()[0] != 0 {
                    push_tr(msg, c"   host name: ");
                } else {
                    push_tr(msg, c"\n         host name: ");
                }
                msg.extend_from_slice(cstr::until_nul(b0.hname()));
            }

            if b0.pid() != 0 {
                push_tr(msg, c"\n        process ID: ");
                // `%d`, so the same 32-bit truncation upstream printed.
                msg.extend_from_slice((b0.pid() as c_int).to_string().as_bytes());
                // Read back by `findswapname` to decide whether this
                // is a crash to recover from or a live second editor.
                proc_running.set(swapfile_proc_running(&b0, fname));
                if proc_running.get() != 0 {
                    push_tr(msg, c" (STILL RUNNING)");
                }
            }

            if b0_magic_wrong(&b0) {
                push_tr(msg, c"\n         [not usable on this computer]");
            }
        }
    }
    msg.push(b'\n');
    x
}

/// Append a translated message.
///
/// Upstream hands the *translated* text to `kv_printf` as the format
/// string, so a `%` a translator wrote is a directive there and reads an
/// argument that was never passed. Not reproduced: the text is appended as
/// it stands.
pub(crate) fn push_tr(msg: &mut Vec<u8>, text: &'static CStr) {
    msg.extend_from_slice(gettext(text).to_bytes());
}

/// Whether this swap file can be deleted without losing anything: it is
/// intact, records no unsaved changes, and the process that wrote it died on
/// this same host.
pub(crate) fn swapfile_unchanged(fname: &CStr) -> bool {
    if !path_exists(fname) {
        return false;
    }
    let Ok(b0) = read_block_zero(fname) else {
        return false;
    };

    // The ID and magic number must be right, and there must be nothing
    // unsaved in it.
    let mut ret = ml_check_b0_id(&b0) && !b0_magic_wrong(&b0) && !b0.dirty();

    // The host name must be known and be this host, otherwise comparing
    // the PID against this machine's processes is meaningless.
    if b0.hname()[0] == 0 {
        ret = false;
    } else {
        let mut hostname = [0u8; B0_HNAME_SIZE as usize];
        os_hostname_into(&mut hostname);
        // The last byte is a terminator either way, in case of corruption.
        let stored = cstr::until_nul(&b0.hname()[..B0_HNAME_SIZE as usize - 1]);
        let here = cstr::until_nul(&hostname[..B0_HNAME_SIZE as usize - 1]);
        if !stored.eq_ignore_ascii_case(here) {
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

/// Whether the swap file `fname` was left behind for a file *other* than the
/// one `buffer` is editing — the common case when `'directory'` gathers every
/// swap file into one place.
///
/// Also publishes [`proc_running`], which the dialog below reads.
pub(crate) fn swapfile_is_for_other_file(buffer: Buf, fname: &CStr) -> bool {
    let Ok(b0) = read_block_zero(fname) else {
        return false;
    };
    proc_running.set(swapfile_proc_running(&b0, fname));

    // When the swap file sits in the same directory as the file, the
    // directory names need not agree — they can be reached through
    // different mount points — so only the tails are compared.
    let stored = cstr::until_nul(b0.fname());
    let stored_tail = &stored[tail_index(stored)..];
    let full = buffer.name.full();
    if b0.flags() & B0_SAME_DIR != 0
        && full.is_some_and(|full| {
            path_fnamecmp(path_tail(full).to_bytes(), stored_tail) == 0
                && same_directory(fname, full)
        })
    {
        return false;
    }
    // The name in the swap file may be "~user/path/file". Symlinks can
    // point at the same file under two names, so the inode has the last
    // word.
    let expanded = expand_env_into(&cstr::owned(stored), MAXPATHL as usize);
    files_differ(full, Some(&cstr::owned(&expanded)), b0.ino())
}

/// Whether the four magic numbers say this swap file was written by a
/// different build: they are one value per width, so a change of
/// endianness or of a type's size shows up as a mismatch.
pub(crate) fn b0_magic_wrong<P: AsRef<[u8]>>(b0: &ZeroBlock<P>) -> bool {
    b0.magic_long() != c_long::from(B0_MAGIC_LONG)
        || b0.magic_int() != B0_MAGIC_INT as c_int
        // Truncated to 16 bits on the way in, so compare it truncated.
        || b0.magic_short() != B0_MAGIC_SHORT as i16
        || b0.magic_char() != B0_MAGIC_CHAR
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
pub(crate) fn files_differ(
    fname_c: Option<&CStr>,
    fname_s: Option<&CStr>,
    ino_block0: c_long,
) -> bool {
    let inode = |name: Option<&CStr>| name.and_then(file_info).map(|info| info.stat.st_ino);
    let ino_c: uint64_t = inode(fname_c).unwrap_or(0);
    let ino_s: uint64_t = inode(fname_s).unwrap_or(ino_block0 as uint64_t);
    if ino_c != 0 && ino_s != 0 {
        return ino_c != ino_s;
    }

    // One of the inodes is unknown: force a full path for each and
    // compare those instead.
    // A missing name has no full path either.
    let full = |name: Option<&CStr>| match name {
        Some(name) => vim_full_name(name, MAXPATHL as usize, true),
        None => Err(Vec::new()),
    };
    let (full_c, full_s) = (full(fname_c), full(fname_s));
    let (ok_c, ok_s) = (full_c.is_ok(), full_s.is_ok());
    if let (Ok(full_c), Ok(full_s)) = (&full_c, &full_s) {
        return full_c != full_s;
    }

    // Neither inodes nor full paths. If neither file appears to exist,
    // the names as given are all there is to go on; otherwise guess that
    // they are different.
    if ino_s == 0 && ino_c == 0 && !ok_c && !ok_s {
        return fname_c != fname_s;
    }
    true
}

/// Update the flags block zero carries about the buffer — whether it has
/// unsaved changes, its `'fileformat'` and its `'fileencoding'` — and push
/// block zero alone to disk.
pub fn ml_setflags(mut buffer: Buf) {
    // Only if block zero is already in memory; there is nothing here
    // worth a read for.
    let Some(id) = buffer.b_ml.ml_mfp.as_ref().and_then(|mfp| mfp.find(0)) else {
        return;
    };

    let changed = buffer.b_changed != 0;
    let fileformat = c_int::from((get_fileformat(buffer) + 1) as uint8_t);
    let fenc = buffer.b_p_fenc.bytes().to_vec();
    let mut b0 = ZeroBlock(buffer.b_ml.page_mut(id));
    b0.set_dirty(changed);
    let flags = b0.flags() & !B0_FF_MASK | fileformat;
    b0.set_flags(flags);
    add_b0_fenc(&mut b0, &fenc);
    let mfp = buffer.b_ml.mfp();
    mfp.mark_dirty(id);
    // Best effort: a swap file that cannot take block zero is reported
    // where it is created, and there is nothing to do about it here.
    let _ = mfp.sync(MFS_ZERO);
}
