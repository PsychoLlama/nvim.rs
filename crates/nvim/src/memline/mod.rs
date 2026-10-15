//! The memline: a buffer's lines, stored in a B-tree of blocks that a
//! [`memfile`](super::memfile) keeps paged to a swap file.
//!
//! Pointer blocks branch by line count and data blocks hold the text; block
//! zero is the header that makes a swap file recognisable to another Nvim.
//! This module owns opening and closing one; the children own the rest —
//! [`tree`] the walk, [`edit`] the insert and delete, [`lines`] the API the
//! editor calls, [`offsets`] byte offsets, [`block0`] the header,
//! [`swapname`] where the swap file goes and [`recover`] reading one back;
//! [`blocks`] is the three block layouts, as views over a memfile page.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use crate::types::AutoEvent;

use crate::autocmd::has_autocmd;
use crate::buffer::state::swap_exists_action;
use crate::buffer::{BufFlags, buf_inc_changedtick, buf_spname, open_buffer, setfname};
use crate::change::{changed_internal, unchanged};
use crate::cursor::{check_cursor, coladvance};
use crate::drawscreen::state::cmdline_row;
use crate::drawscreen::{UPD_NOT_VALID, redraw_curbuf_later};
use crate::eval::vars::{set_vim_var_string, with_vim_var_str};
use crate::ex_docmd::cmdmod_has;
use crate::fileio::state::{did_check_timestamps, need_check_timestamps};
use crate::fileio::{
    buf_store_file_info, modname, readfile_named, rename_file, temp_name, vim_deltempdir,
};
use crate::getchar::flush_buffers;
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::guard::{Allow, Suppress};
use crate::input::prompt_for_input;
use crate::mark::setpcmark;
use crate::mbyte::{char_at, cluster_len, head_off, mb_adjust_cursor, utf_lengths};
use crate::memfile::{MFS_ALL, MFS_FLUSH, MFS_STOP, MFS_ZERO, MemFile, MfDirty, mf_close_file};
use crate::memory::XString;
use crate::message::state::{
    msg_ext_skip_flush, msg_row, msg_silent, need_wait_return, no_lines_msg,
};
use crate::message::{
    do_dialog, emsg, iemsg, msg, msg_display, msg_end, msg_ext_set_kind, msg_home_replace,
    msg_multiline, msg_outnum, msg_putchar, msg_reset_scroll, msg_start, msg_str, msg_str_hl,
    set_keep_msg, verb_msg,
};
use crate::message_fmt::msg_cstr_opt;
use crate::option::vars::{P_DIR, P_SHM, p_uc, p_verbose};
use crate::option::{get_fileformat, set_fileformat, set_option_value_give_err};
use crate::options::kOptFileencoding;
use crate::os::cshim::gettext;
use crate::os::env::os_get_pid;
use crate::os::fs::{os_remove, path_exists};
use crate::os::input::{line_breakcheck, os_char_avail};
use crate::os::proc::os_proc_running;
use crate::os::time::{ctime_text, os_time};
use crate::path::{
    after_pathsep, concat_fnames, expand_wildcards, fix_fname, path_fnamecmp, path_full_compare,
    path_is_absolute, path_tail, same_directory, shorten_dir, tail_index, vim_full_name,
    vim_ispathsep,
};
use crate::pos::MAXLNUM;
use crate::semsg;
use crate::spell::spell_delete_wordlist;
use crate::startup::{getout, recoverymode};
use crate::types::ui::kUIMessages;
use crate::types::{
    BlockNr, Buffer, CmdModFlags, ColNr, Dict, Failed, FileComparison, FileOffset, FlushBuffers,
    InfoPtr, LineNr, List, OptVal, Pos, Timestamp, VarNumber, int64_t, size_t, time_t, uint8_t,
    uint64_t, uv_uid_t,
};
use crate::ui::{ui_flush, ui_has};
use crate::undo::buf_is_changed;
use crate::version::min_vim_version_name;
use crate::winlayer::{Buf, buffers};
use core::ffi::c_int;

// The carve of the transpiled module; see each child's docs.
mod blocks;
pub use self::blocks::*;
mod block0;
pub use self::block0::*;
mod swapname;
pub use self::swapname::*;
mod recover;
pub use self::recover::*;
mod tree;
pub(crate) use self::tree::*;
mod edit;
pub(crate) use self::edit::*;
mod offsets;
pub use self::offsets::*;
mod lines;
pub use self::lines::*;

pub(crate) static inhibit_delete_count: GlobalCell<c_int> = GlobalCell::new(0 as c_int);

pub const VIM_WARNING: ::core::ffi::c_uint = 2;
pub const READ_NEW: ::core::ffi::c_uint = 1;
pub const FLUSH_TYPEAHEAD: FlushBuffers = 1;
/// The smallest page a swap file can have, which is what block zero is first
/// read at when its real page size is not known yet.
pub const MIN_SWAP_PAGE_SIZE: ::core::ffi::c_uint = 1048;
pub const SEA_CHOICE_NONE: SwapExistsChoice = 0;
pub type SwapExistsChoice = ::core::ffi::c_uint;
pub const SEA_CHOICE_ABORT: SwapExistsChoice = 6;
pub const SEA_CHOICE_QUIT: SwapExistsChoice = 5;
pub const SEA_CHOICE_DELETE: SwapExistsChoice = 4;
pub const SEA_CHOICE_RECOVER: SwapExistsChoice = 3;
pub const SEA_CHOICE_EDIT: SwapExistsChoice = 2;
pub const SEA_CHOICE_READONLY: SwapExistsChoice = 1;
pub type UpdBlock0 = ::core::ffi::c_uint;
pub const UB_SAME_DIR: UpdBlock0 = 1;
pub const UB_FNAME: UpdBlock0 = 0;
/// `action` for [`ml_find_line`]: only release the locked block.
pub const ML_FLUSH: ::core::ffi::c_int = 2;
/// `action` for [`ml_find_line`]: a line is about to be deleted here.
pub const ML_DELETE: ::core::ffi::c_int = 17;
/// `action` for [`ml_find_line`]: a line is about to be inserted here.
pub const ML_INSERT: ::core::ffi::c_int = 18;
/// `action` for [`ml_find_line`]: just look the line up.
pub const ML_FIND: ::core::ffi::c_int = 19;
/// Lines below which two neighbouring chunk-index entries are merged, and
/// the size a split aims for.
pub const MLCS_MINL: ::core::ffi::c_int = 400;
/// Lines above which a chunk-index entry is split in two.
pub const MLCS_MAXL: ::core::ffi::c_int = 800;
/// `flags` for `ml_append_int`: carry the `DB_MARKED` bit onto the new line.
pub const ML_APPEND_MARK: ::core::ffi::c_int = 2;
/// `flags` for `ml_append_int`: this is a fresh file being read in, so the
/// block may be numbered negatively and need not keep its position.
pub const ML_APPEND_NEW: ::core::ffi::c_int = 1;
/// `flags` for `ml_delete_int`: say "--No lines in buffer--" if the buffer
/// ends up empty.
pub const ML_DEL_MESSAGE: ::core::ffi::c_int = 1;
pub const kEqualFiles: FileComparison = 1;
pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ML_CHNK_ADDLINE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ML_CHNK_DELLINE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ML_CHNK_UPDLINE: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
crate::flag_set! {
    /// What a buffer's memline is holding right now -- upstream's `ML_*`,
    /// the bits [`crate::types::MemLine::ml_flags`] carries.
    ///
    /// c2rust emitted these as a bare `int` and re-emitted the `#define`s
    /// once per translation unit: seventeen copies of `ML_EMPTY` alone, in
    /// seventeen modules. There is one definition now.
    pub struct MlFlags;

    /// The buffer holds a single empty line and nothing has been written
    /// into it -- what a brand new buffer is, and what `:enew` returns one
    /// to. Cleared by the first line that is appended or replaced.
    const EMPTY = 0x1;
}
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SEA_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEA_QUIT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SEA_RECOVER: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SEA_READONLY: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const STACK_INCR: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
/// One translated static message on the report, in `hl_id`.
///
/// These four exist because `recover.rs` and `swapname.rs` print forty-odd
/// fixed reports, and spelling each one `msg_str_hl(gettext(c"..."), hl, true)`
/// put most of them over the line width -- they went vertical, one line per
/// argument, for nothing. The translation is a `&'static CStr` in and a
/// `&'static CStr` out; no pointer is made anywhere along the way.
fn note(text: &'static ::core::ffi::CStr, hl_id: ::core::ffi::c_int) {
    msg_str_hl(gettext(text), hl_id, true);
}

/// [`note`], appended to the message being built.
fn say(text: &'static ::core::ffi::CStr) {
    msg_str(gettext(text));
}

/// [`note`], as an error.
fn complain(text: &'static ::core::ffi::CStr) {
    emsg(gettext(text));
}

/// [`note`], as a message of its own.
fn tell(text: &'static ::core::ffi::CStr, hl_id: ::core::ffi::c_int) {
    msg(gettext(text), hl_id);
}

/// The lowest line number that may still carry a [`DB_MARKED`] bit, so
/// `ml_firstmarked` need not start its search at line one.
static lowest_marked: GlobalCell<LineNr> = GlobalCell::new(0);

/// Published by `swapfile_info` for the ATTENTION dialog: whether the process
/// that owns the swap file it just described is still alive.
static proc_running: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);

/// Open a new memline for `buffer`: an in-memory memfile with block zero, a root
/// pointer block and one data block holding a single empty line.
///
/// No swap file is created here; [`ml_open_file`] does that later.
pub fn ml_open(mut buffer: Buf) -> Result<(), Failed> {
    // No stack, no cached block, no cached line, no chunk table yet.
    buffer.b_ml.stack_clear();
    buffer.b_ml.ml_locked = None;
    buffer.b_ml.clear_cache();
    buffer.b_ml.ml_chunks.free();

    if cmdmod_has(CmdModFlags::NOSWAPFILE) {
        buffer.b_p_swf = 0;
    }
    // A swap file may still be opened later, when 'updatecount' is set.
    buffer.b_may_swap = buffer.terminal.is_null() && p_uc() != 0 && buffer.b_p_swf != 0;

    buffer.b_ml.ml_mfp = Some(MemFile::in_memory());
    buffer.b_ml.ml_flags = MlFlags::EMPTY;
    buffer.b_ml.ml_line_count = 1;
    if ml_open_blocks(buffer).is_ok() {
        return Ok(());
    }
    if let Some(memfile) = buffer.b_ml.ml_mfp.take() {
        memfile.close(true); // also frees the swap file's name
    }
    Err(Failed)
}

/// Fill in the three blocks a fresh memline starts with.
fn ml_open_blocks(mut buffer: Buf) -> Result<(), Failed> {
    // Block zero: the header that says what the rest of the file means.
    let memfile = buffer.b_ml.memfile();
    let id = memfile.new_block(false, 1);
    if memfile.block_nr(id) != 0 {
        iemsg(gettext(c"E298: Didn't get block nr 0?"));
        memfile.put(id, false, false);
        return Err(Failed);
    }
    let page_size = ::core::ffi::c_long::from(memfile.mf_page_size);
    let mut b0 = ZeroBlock(memfile.page_mut(id));
    b0.set_identity();
    // "VIM " and a five-byte version name, terminated, in the ten bytes.
    let version = b0.version_mut();
    let name = min_vim_version_name().to_bytes();
    let name = &name[..name.len().min(5)];
    version[..4].copy_from_slice(b"VIM ");
    version[4..4 + name.len()].copy_from_slice(name);
    version[4 + name.len()] = 0;
    b0.set_page_size(page_size);

    if !buffer.b_spell {
        let changed = buffer.b_changed != 0;
        let fileformat = get_fileformat(buffer) + 1;
        let mut b0 = ZeroBlock(buffer.b_ml.page_mut(id));
        b0.set_dirty(changed);
        b0.set_flags(fileformat);
        set_b0_fname(buffer, id);
        let user = crate::os::users::username_within(B0_UNAME_SIZE as usize);
        let mut host = [0u8; B0_HNAME_SIZE as usize];
        crate::os::env::os_hostname_into(&mut host);
        let mut b0 = ZeroBlock(buffer.b_ml.page_mut(id));
        let uname = b0.uname_mut();
        uname[..user.len()].copy_from_slice(&user);
        uname[user.len()] = 0;
        uname[B0_UNAME_SIZE as usize - 1] = 0;
        let hname = b0.hname_mut();
        let host = cstr::until_nul(&host);
        hname[..host.len()].copy_from_slice(host);
        hname[host.len()] = 0;
        hname[B0_HNAME_SIZE as usize - 1] = 0;
        b0.set_pid(os_get_pid());
    }

    // Always sync block zero, so that findswapname can read the file name
    // out of the swap file. Not for a help or spell buffer. This only
    // does anything once there is a swap file; otherwise it happens when
    // one is created.
    buffer.b_ml.put(id, true);
    if !buffer.b_help && !buffer.b_spell {
        // Best effort; there may not be a swap file yet.
        let _ = buffer.b_ml.memfile().sync(0);
    }

    // Block one: the root pointer block, pointing at the one data block.
    let memfile = buffer.b_ml.memfile();
    let id = ml_new_ptr(memfile);
    if memfile.block_nr(id) != 1 {
        iemsg(gettext(c"E298: Didn't get block nr 1?"));
        memfile.put(id, false, false);
        return Err(Failed);
    }
    let mut pointers = PointerBlock(memfile.page_mut(id));
    pointers.set_count(1);
    // Line count after the insertion below.
    pointers.set_child(0, 2, 1, 1, 1);
    memfile.put(id, true, false);

    // Block two: the first data block, holding one empty line.
    let id = ml_new_data(memfile, false, 1);
    if memfile.block_nr(id) != 2 {
        iemsg(gettext(c"E298: Didn't get block nr 2?"));
        memfile.put(id, false, false);
        return Err(Failed);
    }
    let mut data = DataBlock(memfile.page_mut(id));
    let start = data.txt_start() - 1; // at the end of the block
    data.set_txt_start(start);
    data.set_index(0, start);
    data.set_free(data.free() - 1 - INDEX_SIZE as ::core::ffi::c_uint);
    data.set_line_count(1);
    data.write(start as usize, &[0]);
    Ok(())
}

/// Open a swap file for every buffer that could use one.
pub fn ml_open_files() {
    for buf in buffers() {
        if buf.b_p_ro == 0 || buf.b_changed != 0 {
            ml_open_file(buf);
        }
    }
}

/// Open a swap file for `buffer`'s memfile, if it has none yet.
///
/// If no usable file name can be found the memfile keeps no name and
/// remains memory-only, with no recovery possible.
pub fn ml_open_file(mut buffer: Buf) {
    if buffer
        .b_ml
        .ml_mfp
        .as_ref()
        .is_none_or(|memfile| memfile.mf_fd >= 0)
        || buffer.b_p_swf == 0
        || cmdmod_has(CmdModFlags::NOSWAPFILE)
        || !buffer.terminal.is_null()
    {
        return; // nothing to do
    }

    // A spell buffer gets a temp file name.
    if buffer.b_spell {
        if let Some(fname) = temp_name() {
            // A spell buffer keeps working without a swap file.
            let _ = buffer.b_ml.memfile().open_file(fname.as_cstr().to_owned());
        }
        buffer.b_may_swap = false;
        return;
    }

    // Try every directory in 'directory'.
    // A copy: the walk outlives a projection's borrow.
    let dir = P_DIR.get();
    let mut found_existing_dir = false;
    let mut rest: &[u8] = &dir;
    while !rest.is_empty() {
        let (dir_name, after) = crate::option::split_option_part(rest, rest.len() + 1, b",");
        rest = after;
        // Between choosing the name and creating the file another Nvim
        // may have created it; then the create fails and the next
        // directory is tried.
        let found = findswapname(
            buffer,
            &dir_name,
            rest.is_empty(),
            None,
            &mut found_existing_dir,
        );
        let Some(fname) = found else {
            continue;
        };
        if buffer
            .b_ml
            .memfile()
            .open_file(fname.as_cstr().to_owned())
            .is_err()
        {
            continue;
        }
        buffer.b_ml.memfile().mf_dirty = MfDirty::YesNoSync; // don't sync yet in ml_sync_all
        ml_upd_block0(buffer, UB_SAME_DIR);

        // Flush block zero, so others can read it.
        if buffer.b_ml.memfile().sync(MFS_ZERO).is_ok() {
            // Mark every block that belongs in the swap file dirty, for
            // when 'swapfile' was reset (deleting the file) and set again.
            buffer.b_ml.memfile().set_dirty();
            break;
        }
        // Writing block zero failed: close it and try another directory.
        mf_close_file(buffer, false);
    }

    if P_DIR.first_byte() != 0 && buffer.b_ml.memfile().fname().is_none() {
        need_wait_return.set(true); // call wait_return() later
        let _no_prompt = Suppress::wait_return();
        let arg0 = msg_cstr_opt(buf_spname(&buffer).or(buffer.name.shown()));
        semsg!("E303: Unable to open swap file for \"{arg0}\", recovery impossible");
    }

    buffer.b_may_swap = false; // don't try to open a swap file again
}

/// Create the swap file now, if one is still wanted and this is a writable
/// file being opened or a read into an existing buffer.
pub fn check_need_swap(newfile: bool) {
    // The swap dialog may prompt, and the user has to see it; E325 may
    // reset this again.
    let _loud = Allow::messages();
    if Buf::current().b_may_swap && (Buf::current().b_p_ro == 0 || !newfile) {
        ml_open_file(Buf::current());
    }
}

/// Close `buffer`'s memline, deleting the swap file if `del_file`.
pub fn ml_close(mut buffer: Buf, del_file: ::core::ffi::c_int) {
    let Some(memfile) = buffer.b_ml.ml_mfp.take() else {
        return; // not open
    };
    // Nothing is held any more: the memfile goes, and every block in it.
    buffer.b_ml.ml_locked = None;
    memfile.close(del_file != 0); // closes the .swp file
    // The cached line, whatever it was: an owned one is dropped here.
    buffer.b_ml.clear_cache();
    buffer.b_ml.stack_free();
    buffer.b_ml.ml_chunks.free();

    // Clear the "recovered" flag, so the ATTENTION prompt comes back the
    // next time this buffer is loaded.
    buffer.b_flags.clear(BufFlags::RECOVERED);
}

/// Close every memline and memfile. Only used when exiting.
pub fn ml_close_all(del_file: bool) {
    for buf in buffers() {
        ml_close(buf, del_file as ::core::ffi::c_int);
    }
    spell_delete_wordlist(); // delete the internal wordlist
    vim_deltempdir(); // delete the temp directory that was created
}

/// Close the memfile of every unmodified buffer. Only for use just before
/// exiting.
pub fn ml_close_notmod() {
    for buf in buffers() {
        if !buf_is_changed(buf) {
            ml_close(buf, 1);
        }
    }
}

pub const EOL_DOS: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

/// [`makeswapname`](swapname::makeswapname) for names the caller holds: the
/// swap file name for `fname` under the `'directory'` entry `dir_name`, or
/// `None`.
pub(crate) fn swap_name_for(
    fname: &::core::ffi::CStr,
    _ffname: &::core::ffi::CStr,
    dir_name: &::core::ffi::CStr,
) -> Option<XString> {
    makeswapname(Some(fname), dir_name.to_bytes())
}
