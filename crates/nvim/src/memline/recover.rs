//! Surviving a crash: keeping the swap file current, and
//! getting a buffer back out of one.
//!
//! `ml_sync_all` and `ml_preserve` are the writing half — the timer and
//! `:preserve` making sure the swap file is worth recovering from.
//!
//! `ml_recover` is the recovery itself. Everything it reads comes off disk
//! from a file that was, by definition, not written cleanly, so every count
//! and every block number in it is suspect: the walk validates each one, and
//! where it cannot, it appends a `???` marker line and keeps going. Getting
//! *most* of the buffer back is the whole point.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::allocator::Owned;
use crate::autocmd::fire_autocmds_for;
use crate::buffer::{BufFlags, alloc_unregistered_buffer};
use crate::cstr;
use crate::env::{expand_env_within, home_to_tilde};
use crate::fileio::{Loaded, ReadFlags};
use crate::guard::Suppress;
use crate::memfile::{BlockId, MemFile};
use crate::message_fmt::msg_cstr;
use crate::narrow::len_as_int;
use crate::os::fs::file_info;
use crate::smsg;
use crate::types::ApiString;
use core::ffi::{CStr, c_int, c_long, c_uint};
use std::ffi::CString;

use super::*;
use crate::memory::ThinCString;
use crate::option::defs::OptionSetFlags;
use crate::path::MAXPATHL;
use crate::types::Failed;
use crate::winlayer::{Buf, Win};

/// Read the original file into `curbuf`, starting after line `from`.
///
/// Both callers name `curbuf`'s own full path and pass no `:read` command,
/// which is the whole of `readfile`'s precondition; they differ only in the
/// range and in whether the buffer is new.
fn read_original(from: LineNr, skip: i32, lines: i32, flags: c_int) -> Result<Loaded, Failed> {
    // A copy: the read's autocommands may rename the buffer.
    let name = Buf::current().name.full().map(CStr::to_owned);
    let flags = ReadFlags::from_bits(flags);
    readfile_named(name.as_deref(), None, from, skip, lines, None, flags, false)
}

/// `name` as a message argument.
fn shown(name: &CStr) -> impl core::fmt::Display + '_ {
    msg_cstr(name)
}

/// What a swap file's block zero says about the buffer it belongs to, kept
/// once the block has gone back to the memfile.
#[derive(Debug)]
struct Recorded {
    /// The edited file's name, as block zero spells it.
    fname: Vec<u8>,
    /// `'fileformat'` + 1, or 0 for none.
    fileformat: c_int,
    /// `'fileencoding'`, when it was recorded.
    fileencoding: Option<Vec<u8>>,
    /// The edited file's modification time, low 32 bits.
    mtime: c_long,
    /// The PID of the Nvim that wrote it.
    pid: c_long,
    /// The view of block zero `swapfile_proc_running` reads.
    block: Vec<u8>,
}

/// Try to recover `curbuf` from its swap file.
///
/// `checkext`: whether the buffer's own name may itself be a swap file name,
/// as it is for `nvim -r file.swp`.
pub fn ml_recover(checkext: bool) {
    recoverymode.set(true);
    let called_from_main = !Buf::current().b_ml.is_open();

    // Who owns what `buf` names. The recovery buffer is not in the
    // registry, so this frame is its owner; `buf` is only the handle the
    // memline code below works through. Its memline owns the memfile on
    // the old swap file, and closing that is this frame's job too.
    let mut owned_buf: Option<Owned<Buffer>> = None;
    let mut recovery: Option<Buf> = None;
    // Nothing was recovered yet, so a failure now leaves the buffer with
    // no memline at all.
    let mut serious_error = true;

    let outcome: Result<(), MemlineError> = 'theend: {
        // A copy: the autocommands below may rename the buffer.
        let fname: CString = if Buf::current().name.is_unnamed() {
            CString::default()
        } else {
            Buf::current().name.shown().unwrap_or(c"").to_owned()
        };
        // A name ending in ".s[a-w][a-z]" is taken to be the swap file
        // itself; otherwise its swap files are searched for.
        let directly = checkext && looks_like_swapfile(fname.as_bytes());
        let fname_used: CString = if directly {
            fname.clone()
        } else {
            match choose_swapfile(&fname) {
                Ok(chosen) => chosen.as_cstr().to_owned(),
                Err(err) => break 'theend Err(err),
            }
        };
        // When called from main() the storage structure still needs
        // initialising.
        if called_from_main && ml_open(Buf::current()).is_err() {
            getout(1);
        }

        // A buffer structure for the swap file being recovered. Only the
        // memline in it is really used, and it is never registered or
        // put on the buffer list -- see `alloc_unregistered_buffer`.
        let mut buf = Buf::owned_by(owned_buf.insert(alloc_unregistered_buffer()));
        recovery = Some(buf);
        buf.b_ml.stack_clear(); // nothing in the stack
        buf.b_ml.clear_cache(); // no cached line
        buf.b_ml.ml_locked = None; // no locked block
        buf.b_ml.ml_flags = MlFlags::NONE;

        // Open the memfile on the old swap file.
        let Some(memfile) = MemFile::open(fname_used.clone(), O_RDONLY) else {
            break 'theend Err(MemlineError::CannotOpen(ThinCString::from_cstr(
                &fname_used,
            )));
        };
        buf.b_ml.ml_mfp = Some(memfile);

        let recorded = match read_block_zero(buf, directly) {
            Ok(recorded) => recorded,
            Err(err) => break 'theend Err(err),
        };

        // Given the swap file's name directly, the buffer takes its name
        // from what the swap file says it belongs to.
        if directly {
            let name = cstr::owned(&recorded.fname);
            let path = expand_env_within(&name, MAXPATHL as usize);
            if let Err(err) = Buf::current().set_name(Some(&cstr::owned(&path)), None, true) {
                break 'theend Err(err.into());
            }
        }

        msg_ext_set_kind(c"wmsg");
        msg_ext_skip_flush.set(true);
        let swap_name = buf.b_ml.memfile().fname().unwrap_or(c"").to_owned();
        let path = home_to_tilde(None, &swap_name, MAXPATHL as usize, true);
        smsg!(0, "Using swap file \"{}\"", shown(path.as_cstr()));
        let path = match buf_spname(&Buf::current()) {
            Some(special) => XString::from_cstr(special),
            None => {
                let full = Buf::current().name.full().unwrap_or(c"").to_owned();
                home_to_tilde(None, &full, MAXPATHL as usize, true)
            }
        };
        msg_putchar('\n' as c_int);
        smsg!(0, "Original file \"{}\"", shown(path.as_cstr()));
        msg_putchar('\n' as c_int);
        msg_ext_skip_flush.set(false);

        // Compare the dates of the swap file and the original.
        // Block zero keeps the low 32 bits; so does this.
        let mtime = low32(recorded.mtime).cast_signed();
        let original = Buf::current().name.full().and_then(file_info);
        if let Some(original) = original {
            let swap = file_info(&swap_name);
            let newer =
                swap.is_some_and(|swap| original.stat.st_mtim.tv_sec > swap.stat.st_mtim.tv_sec);
            if newer || original.stat.st_mtim.tv_sec != mtime.into() {
                complain(c"E308: Warning: Original file may have been changed");
            }
        }
        ui_flush();

        // Recovery is going ahead, so the buffer's current contents go.
        while !Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY) {
            let _ = ml_delete(LineNr::ONE);
        }

        // Read the original file, to pick up 'fileformat', 'fileencoding'
        // and friends. Errors are ignored, and the text itself is not
        // used — except as the "unchanged?" comparison below.
        let mut orig_file_status = Err(Failed);
        if Buf::current().name.full().is_some() {
            orig_file_status =
                read_original(LineNr::ZERO, 0, MAXLNUM.get(), READ_NEW.cast_signed());
        }

        // What the swap file recorded wins over what the file suggests.
        if recorded.fileformat != 0 {
            set_fileformat(recorded.fileformat - 1, OptionSetFlags::LOCAL);
        }
        if let Some(fenc) = &recorded.fileencoding {
            set_option_value_give_err(
                kOptFileencoding,
                OptVal::string(ApiString::from_bytes(fenc)),
                OptionSetFlags::LOCAL,
            );
        }
        unchanged(Buf::current(), true, true);

        serious_error = false;
        let (lnum, error) = match recover_lines(buf) {
            Ok(found) => found,
            Err(err) => break 'theend Err(err),
        };

        // Compare the recovered contents with the original file's.
        // Lines 1 to lnum are what was recovered, lines lnum + 1 to
        // ml_line_count are the file's, and line ml_line_count + 1 is the
        // empty buffer's dummy line.
        if orig_file_status != Ok(Loaded::Read)
            || Buf::current().b_ml.ml_line_count != LineNr::new(lnum.get() * 2 + 1)
        {
            // Recovering an empty file gives two lines of which the first
            // is empty; that is not a modification.
            if !(Buf::current().b_ml.ml_line_count == LineNr::new(2)
                && Buf::current().lines().line(LineNr::ONE).is_empty())
            {
                changed_internal(Buf::current());
                buf_inc_changedtick(Buf::current());
            }
        } else {
            for idx in 1..=lnum.get() {
                // One of the two lines has to be copied: fetching the
                // other flushes the cache the first came out of.
                let recovered = Buf::current().lines().line_copy(LineNr::new(idx));
                let same = *recovered == *Buf::current().lines().line(lnum + idx);
                if !same {
                    changed_internal(Buf::current());
                    buf_inc_changedtick(Buf::current());
                    break;
                }
            }
        }

        // Drop the original file's lines and the empty buffer's dummy
        // line; they are now past the end of what was recovered.
        while Buf::current().b_ml.ml_line_count > lnum
            && !Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY)
        {
            let _ = ml_delete(Buf::current().b_ml.ml_line_count);
        }
        Buf::current().b_flags |= BufFlags::RECOVERED;
        check_cursor(Win::current());

        msg_ext_skip_flush.set(!got_int.get());
        recoverymode.set(false);
        report_recovery(error, &recorded, &fname_used);
        redraw_curbuf_later(UPD_NOT_VALID);
        Ok(())
    };
    // Said before anything is put back: the cleanup below fires
    // autocommands.
    if let Err(err) = outcome {
        err.report();
    }

    msg_ext_skip_flush.set(false);
    recoverymode.set(false);
    if let Some(mut buf) = recovery {
        // Whatever block the walk still held goes back, and the memfile
        // closes; the swap file stays.
        if let Some(locked) = buf.b_ml.unlock() {
            buf.b_ml.put(locked.id, false);
        }
        if let Some(memfile) = buf.b_ml.ml_mfp.take() {
            memfile.close(false);
        }
    }
    // The free: `Buffer`'s destructor runs, taking the block stack with
    // it, and the memory goes back.
    drop(owned_buf);
    if serious_error && called_from_main {
        ml_close(Buf::current(), 1);
    } else {
        // `curbuf` and its name are read afresh for each event: the
        // first one's autocommands may have changed either.
        for event in [AutoEvent::BufReadPost, AutoEvent::BufWinEnter] {
            let current = Buf::current();
            let name = current.name.shown().map(XString::from_cstr);
            fire_autocmds_for(event, None, name, false, Some(current), None);
        }
    }
}

/// Read and check block zero of the swap file `buf`'s memfile is open on,
/// fixing the memfile's page size from it, and answer what it records, or
/// why the file cannot be recovered from.
///
/// `directly`: the user named the swap file itself, so its recorded file
/// name is wanted.
fn read_block_zero(mut recovery: Buf, directly: bool) -> Result<Recorded, MemlineError> {
    let swap_name = recovery.b_ml.memfile().fname().unwrap_or(c"").to_owned();

    // The page size `MemFile::open` picked need not be the one the swap
    // file was written with; the real one is in block zero. Reading
    // block zero needs *a* page size, so use the smallest one a swap
    // file can have, and correct it below.
    recovery.b_ml.memfile().mf_page_size = MIN_SWAP_PAGE_SIZE;

    msg_ext_set_kind(c"emsg");
    let Some(id) = recovery.b_ml.memfile().get(0, 1) else {
        return Err(MemlineError::NoBlockZero(ThinCString::from_cstr(
            &swap_name,
        )));
    };
    let recorded = check_block_zero(recovery, id, &swap_name, directly);
    // Release block zero.
    recovery.b_ml.put(id, false);
    recorded
}

/// [`read_block_zero`] once the block is in memory.
fn check_block_zero(
    mut recovery: Buf,
    id: BlockId,
    swap_name: &CStr,
    directly: bool,
) -> Result<Recorded, MemlineError> {
    let swap = || ThinCString::from_cstr(swap_name);
    let b0 = ZeroBlock(recovery.b_ml.page(id));
    if b0.version().starts_with(b"VIM 3.0") {
        return Err(MemlineError::Vim3(swap()));
    }
    if !ml_check_b0_id(&b0) {
        return Err(MemlineError::NotSwapFile(swap()));
    }
    if b0_magic_wrong(&b0) {
        // Only the host name is wanted, and it is terminated inside its own
        // field so that printing it cannot run off the end of a corrupted
        // one.
        let host = ThinCString::from_bytes(cstr::until_nul(b0.hname()));
        return Err(MemlineError::OtherComputer(Box::new([swap(), host])));
    }

    // The guessed page size was wrong, so the highest block number
    // in the file has to be worked out again.
    let recorded_page_size = low32(b0.page_size());
    let memfile = recovery.b_ml.memfile();
    if memfile.mf_page_size != recorded_page_size {
        let previous_page_size = memfile.mf_page_size;
        memfile.set_page_size(recorded_page_size);
        if memfile.mf_page_size < previous_page_size {
            return Err(MemlineError::PageTooSmall(swap()));
        }
        let size = memfile.file_size();
        // Zero means no file, or an empty one.
        memfile.mf_blocknr_max = if size <= 0 {
            0
        } else {
            size / FileOffset::from(memfile.mf_page_size)
        };
        memfile.mf_infile_count = memfile.mf_blocknr_max;

        // Block zero's own page was read at the guessed size.
        memfile.resize_page(id, memfile.mf_page_size as usize);
    }

    let b0 = ZeroBlock(recovery.b_ml.page(id));
    // Take 'fileformat' and 'fileencoding' from block zero. The
    // encoding sits at the very end of the name field, behind a NUL,
    // so it is found by scanning back from there.
    let fileencoding = (b0.flags() & B0_HAS_FENC != 0).then(|| {
        let field = &b0.fname()[..B0_FNAME_SIZE_NOCRYPT as usize];
        let from = field.iter().rposition(|&b| b == 0).map_or(0, |at| at + 1);
        field[from..].to_vec()
    });
    Ok(Recorded {
        fname: if directly {
            cstr::until_nul(b0.fname()).to_vec()
        } else {
            Vec::new()
        },
        fileformat: b0.flags() & B0_FF_MASK,
        fileencoding,
        mtime: b0.mtime(),
        pid: b0.pid(),
        block: recovery.b_ml.page(id)[..B0_SIZE].to_vec(),
    })
}

/// Whether this name is itself a swap file name: it ends in `.s`, a letter
/// from `a` to `w`, and any letter — the extensions `findswapname` permutes
/// through.
fn looks_like_swapfile(fname: &[u8]) -> bool {
    let len = fname.len();
    len >= 4
        && fname[len - 4..len - 2].eq_ignore_ascii_case(b".s")
        && (b'a'..=b'w').contains(&fname[len - 2].to_ascii_lowercase())
        && fname[len - 1].is_ascii_alphabetic()
}

/// Pick which of `fname`'s swap files to recover from: the only one there is,
/// or the one the user names out of a listing.
///
/// Returns the name, or why there is none.
fn choose_swapfile(fname: &CStr) -> Result<XString, MemlineError> {
    let (count, _) = recover_names(Some(fname), false, None, 0);
    if count == 0 {
        return Err(MemlineError::NoSwapFile(ThinCString::from_cstr(fname)));
    }
    let nr = if count == 1 {
        1
    } else {
        let _ = recover_names(Some(fname), true, None, 0);
        if !ui_has(kUIMessages) {
            msg_putchar('\n' as c_int);
        }
        let question = gettext(c"Enter number of swap file to use (0 to quit): ");
        let nr = prompt_for_input(Some(question), 0, false, None);
        if nr < 1 || nr > count {
            return Err(MemlineError::NoneChosen);
        }
        nr
    };
    recover_names(Some(fname), false, None, nr)
        .1
        .ok_or(MemlineError::NoneChosen)
}

/// Walk the swap file's block tree and append every line it can find to
/// `curbuf`, after line 0.
///
/// `buffer` is the scratch buffer whose memline is open on the swap file and
/// whose `ml_stack` records the descent. Nothing in the file is trusted: a
/// count or a block number that cannot be right costs a `???` line and the
/// walk carries on. Returns the number of lines appended and the number of
/// problems found, or why block 1 itself is unusable, which leaves nothing
/// to recover.
///
/// The block the walk is on is the recovery memline's locked block, so that
/// the caller releases it whichever way the walk ends.
fn recover_lines(mut buffer: Buf) -> Result<(LineNr, c_int), MemlineError> {
    let mut bnum: BlockNr = 1; // start with block 1
    let mut page_count: c_uint = 1; // which is one page
    let mut lnum: LineNr = LineNr::ZERO; // append after line 0 in curbuf
    let mut line_count: LineNr = LineNr::ZERO;
    let mut idx: usize = 0; // start with the first index in block 1
    let mut error = 0;
    buffer.b_ml.stack_clear();
    let swap_name = buffer.b_ml.memfile().fname().unwrap_or(c"").to_owned();

    // Without a file to fall back on, a data block whose number went
    // negative (never written to the swap file) is simply lost.
    let mut cannot_open = Buf::current().name.full().is_none();

    let append = |lnum: &mut LineNr, text: &[u8]| {
        let _ = ml_append_terminated(*lnum, text, true);
        *lnum += 1;
    };
    let append_note = |lnum: &mut LineNr, text: &'static CStr| {
        append(lnum, gettext(text).to_bytes_with_nul());
    };

    'walk: while !got_int.get() {
        'step: {
            if let Some(held) = buffer.b_ml.unlock() {
                buffer.b_ml.put(held.id, false); // release the previous block
            }
            let block = buffer.b_ml.memfile().get(bnum, page_count);
            if let Some(id) = block {
                buffer.b_ml.lock(id, LineNr::ZERO, LineNr::ZERO);
            }
            if block.is_none() {
                if bnum == 1 {
                    return Err(MemlineError::NoBlockOne(ThinCString::from_cstr(&swap_name)));
                }
                error += 1;
                append_note(&mut lnum, c"???MANY LINES MISSING");
            } else if let Some(id) = block
                && PointerBlock(buffer.b_ml.page(id)).is_pointer()
            {
                // The counts in the header have to fit the page size this
                // build uses, or the entries cannot be walked at all.
                let page_size = buffer.b_ml.memfile().mf_page_size;
                let count_max = PointerBlock::<&[u8]>::count_max_for(page_size);
                let mut pointers = PointerBlock(buffer.b_ml.page_mut(id));
                let mut ptr_block_error = false;
                if pointers.count_max() != count_max {
                    ptr_block_error = true;
                    pointers.set_count_max(count_max);
                }
                if pointers.count() > pointers.count_max() {
                    ptr_block_error = true;
                    pointers.set_count(pointers.count_max());
                }
                if ptr_block_error {
                    complain(c"E1364: Warning: Pointer block corrupted");
                }

                let pointers = PointerBlock(buffer.b_ml.page(id));
                let count = usize::from(pointers.count());
                // The first time down this block, its entries should
                // account for exactly the line count promised above it.
                if idx == 0 && line_count != LineNr::ZERO {
                    line_count -= pointers.line_total();
                    if line_count != LineNr::ZERO {
                        error += 1;
                        append_note(&mut lnum, c"???LINE COUNT WRONG");
                    }
                }

                if count == 0 {
                    append_note(&mut lnum, c"???EMPTY BLOCK");
                    error += 1;
                } else if idx < count {
                    let pe = PointerBlock(buffer.b_ml.page(id)).entry(idx);
                    if pe.pe_bnum < 0 {
                        // A data block whose number is still negative was
                        // never written out, so its lines are only in the
                        // original file. Reading them back from there is
                        // slow, but it works.
                        if !cannot_open {
                            line_count = LineNr::new(pe.pe_line_count);
                            // `pe_line_count` and `pe_old_lnum` come off
                            // disk; readfile() must not be handed either
                            // of them unchecked.
                            if line_count <= LineNr::ZERO
                                || pe.pe_old_lnum < LineNr::ONE
                                || read_original(
                                    lnum,
                                    (pe.pe_old_lnum - 1).get(),
                                    line_count.get(),
                                    0,
                                )
                                .is_err()
                            {
                                cannot_open = true;
                            } else {
                                lnum += line_count.get();
                            }
                        }
                        if cannot_open {
                            error += 1;
                            append_note(&mut lnum, c"???LINES MISSING");
                        }
                        idx += 1; // same block again, for the next index
                        break 'step;
                    }

                    // One block deeper in the tree.
                    let top = ml_add_stack(buffer);
                    let frame = InfoPtr {
                        ip_bnum: bnum,
                        ip_low: LineNr::ZERO,
                        ip_high: LineNr::ZERO,
                        ip_index: len_as_int(idx),
                    };
                    buffer.b_ml.stack_set(top, frame);

                    bnum = pe.pe_bnum;
                    line_count = LineNr::new(pe.pe_line_count);
                    page_count = pe.pe_page_count.cast_unsigned();
                    // `pe_page_count` sizes the allocation `get` makes, so a
                    // bogus value (0x40000000, say) would ask for gigabytes.
                    // It must be at least one page, and the block must lie
                    // inside the file.
                    let blocknr_max = buffer.b_ml.memfile().mf_blocknr_max;
                    if page_count < 1 || bnum + BlockNr::from(page_count) > blocknr_max + 1 {
                        error += 1;
                        append_note(&mut lnum, c"???ILLEGAL BLOCK NUMBER");
                        // Skip this entry and pop back up, to recover
                        // whatever else there is.
                        let ip = buffer.b_ml.stack_at(top);
                        idx = ip.ip_index.cast_unsigned() as usize + 1;
                        bnum = ip.ip_bnum;
                        page_count = 1;
                        buffer.b_ml.stack_pop();
                        break 'step;
                    }
                    idx = 0;
                    break 'step;
                }
            } else if let Some(id) = block
                && !DataBlock(buffer.b_ml.page(id)).is_data()
            {
                if bnum == 1 {
                    return Err(MemlineError::BlockOneId(ThinCString::from_cstr(&swap_name)));
                }
                error += 1;
                append_note(&mut lnum, c"???BLOCK MISSING");
            } else if let Some(id) = block {
                // A data block: append every line in it.
                let mut has_error = false;
                let page_size = buffer.b_ml.memfile().mf_page_size;
                let mut data = DataBlock(buffer.b_ml.page_mut(id));

                // The block's length has to match the page count the
                // pointer block promised; if it does not, trust the
                // pointer block.
                let end = page_count.wrapping_mul(page_size);
                let txt_end_wrong = end != data.txt_end();
                if txt_end_wrong {
                    data.set_txt_end(end);
                }

                // Make sure the block ends in a NUL, so that copying the
                // last line out of it cannot run past the end.
                let last = data.txt_end() as usize - 1;
                data.write(last, &[0]);

                let block_lines = data.line_count();
                let (txt_start, txt_end) = (data.txt_start() as usize, data.txt_end() as usize);
                if txt_end_wrong {
                    append_note(
                        &mut lnum,
                        c"??? from here until ???END lines may be messed up",
                    );
                    error += 1;
                    has_error = true;
                }

                // Likewise for the line count.
                if c_long::from(line_count) != block_lines {
                    append_note(
                        &mut lnum,
                        c"??? from here until ???END lines may have been inserted/deleted",
                    );
                    error += 1;
                    has_error = true;
                }

                let mut did_questions = false;
                for i in 0..usize::try_from(block_lines).unwrap_or(0) {
                    if DataBlock::<&[u8]>::index_at(i) >= txt_start {
                        // The line count must be wrong: the index array
                        // has run into the text.
                        error += 1;
                        append_note(&mut lnum, c"??? lines may be missing");
                        break;
                    }
                    let data = DataBlock(buffer.b_ml.page(id));
                    let start = data.line_start(i) as usize;
                    if start <= HEADER_SIZE || start >= txt_end {
                        error += 1;
                        // One "???" for a run of them is enough.
                        if did_questions {
                            continue;
                        }
                        did_questions = true;
                        append(&mut lnum, b"???\0");
                    } else {
                        did_questions = false;
                        // The data block's own NUL-terminated line, ended
                        // at the latest by the NUL written above.
                        let text = data.text(start, txt_end);
                        let len = text.iter().position(|&b| b == 0).unwrap_or(text.len() - 1);
                        let line = text[..=len].to_vec();
                        append(&mut lnum, &line);
                    }
                }
                if has_error {
                    append_note(&mut lnum, c"???END");
                }
            }

            // One block back up the tree, and on to the next index.
            let Some(ip) = buffer.b_ml.stack_pop() else {
                break 'walk; // finished
            };
            bnum = ip.ip_bnum;
            idx = ip.ip_index.cast_unsigned() as usize + 1;
            page_count = 1;
        }
        line_breakcheck();
    }

    Ok((lnum, error))
}

/// Say how it went, and what the user should do next.
fn report_recovery(error: c_int, recorded: &Recorded, fname_used: &CStr) {
    if got_int.get() {
        complain(c"E311: Recovery Interrupted");
        return;
    }
    if error != 0 {
        let no_prompt = Suppress::wait_return();
        msg_ext_set_kind(c"emsg");
        msg(c">>>>>>>>>>>>>\n", 0);
        emsg(gettext(
            c"E312: Errors detected while recovering; look for lines starting with ???",
        ));
        drop(no_prompt);
        msg_putchar('\n' as c_int);
        tell(c"See \":help E312\" for more information.", 0);
        msg(c"\n>>>>>>>>>>>>>", 0);
        return;
    }

    msg_ext_set_kind(c"wmsg");
    if Buf::current().b_changed != 0 {
        tell(
            c"Recovery completed. You should check if everything is OK.",
            0,
        );
        msg_str(gettext(
            c"\n(You might want to write out this file under another name\n",
        ));
        msg_str(gettext(
            c"and run diff with the original file to check for changes)",
        ));
    } else {
        tell(
            c"Recovery completed. Buffer contents equals file contents.",
            0,
        );
    }
    say(c"\nYou may want to delete the .swp file now.");
    if swapfile_proc_running(&ZeroBlock(&recorded.block[..]), fname_used) != 0 {
        // There may be a live Nvim on the same file; the user may want to
        // go and kill it.
        say(c"\nNote: process STILL RUNNING: ");
        // `%d`: the 32-bit truncation upstream printed.
        msg_outnum(low32(recorded.pid).cast_signed());
    }
    if !ui_has(kUIMessages) {
        msg_str(c"\n\n");
    }
    cmdline_row.set(msg_row.get());
}

/// Sync every buffer's memline.
///
/// `check_file`: also check that the original file still exists and is
/// unchanged. `check_char`: stop as soon as a character is typed, having
/// synced at least one block.
pub fn ml_sync_all(check_file: c_int, check_char: c_int, do_fsync: bool) {
    for mut buf in buffers() {
        if buf
            .b_ml
            .ml_mfp
            .as_ref()
            .is_none_or(|memfile| memfile.fname().is_none())
        {
            continue;
        }
        ml_flush_line(buf); // flush the buffered line
        let _ = ml_find_line(buf, LineNr::ZERO, ML_FLUSH); // flush the locked block

        if buf_is_changed(buf)
            && check_file != 0
            && buf.b_ml.memfile().need_trans()
            && let Some(full) = buf.name.full()
        {
            // If the original file is gone or has changed, preserve
            // now, to get rid of all the negative numbered blocks.
            let changed = file_info(full).is_none_or(|info| {
                info.stat.st_mtim.tv_sec != buf.b_mtime_read
                    || info.stat.st_mtim.tv_nsec != buf.b_mtime_read_ns
                    || info.stat.st_size != buf.b_orig_size
            });
            if changed {
                ml_preserve(buf, false, do_fsync);
                did_check_timestamps.set(false);
                need_check_timestamps.set(true); // give the message later
            }
        }

        if buf.b_ml.memfile().mf_dirty == MfDirty::Yes {
            // Best effort: `ml_sync_all` writes what it can and leaves
            // the rest dirty.
            let stop = if check_char != 0 { MFS_STOP } else { 0 };
            let flush = if do_fsync && buf_is_changed(buf) {
                MFS_FLUSH
            } else {
                0
            };
            let _ = buf.b_ml.memfile().sync(stop | flush);
            if check_char != 0 && os_char_avail() {
                break; // a character is available now
            }
        }
    }
}

/// Sync one buffer, including its negative numbered blocks, so that
/// afterwards everything is in the swap file.
///
/// This is `:preserve`, and what happens when the original file has been
/// changed or deleted. `message` reports whether it worked.
pub fn ml_preserve(buffer: Buf, message: bool, do_fsync: bool) {
    let preserved = preserve(buffer, do_fsync);
    if !message {
        return;
    }
    match preserved {
        Ok(()) => tell(c"File preserved", 0),
        Err(err) => err.report(),
    }
}

/// [`ml_preserve`]'s work, answering why it did not take.
fn preserve(mut buffer: Buf, do_fsync: bool) -> Result<(), MemlineError> {
    if buffer
        .b_ml
        .ml_mfp
        .as_ref()
        .is_none_or(|memfile| memfile.fname().is_none())
    {
        return Err(MemlineError::NothingToPreserve);
    }

    // Only an interrupt from here on counts, not one from before.
    let got_int_save = got_int.get();
    got_int.set(false);

    ml_flush_line(buffer); // flush the buffered line
    let _ = ml_find_line(buffer, LineNr::ZERO, ML_FLUSH); // flush the locked block
    let sync_flags = MFS_ALL | if do_fsync { MFS_FLUSH } else { 0 };
    let mut status = buffer
        .b_ml
        .memfile()
        .sync(sync_flags)
        .map_err(|_| MemlineError::PreserveFailed);
    buffer.b_ml.stack_clear(); // the stack is invalid after MFS_ALL

    // Some data blocks may have gone from a negative to a positive block
    // number, which means the pointer blocks referring to them need
    // updating. Which pointer blocks those are is not recorded, so every
    // data block is visited until no translations are left (or the end of
    // the file is reached, which can only happen when a write failed —
    // a full file system, say). `ml_find_line` does the work, translating
    // the negative numbers as it fetches each block's first line.
    'theend: {
        if buffer.b_ml.memfile().need_trans() && !got_int.get() {
            let mut lnum: LineNr = LineNr::ONE;
            while buffer.b_ml.memfile().need_trans() && lnum <= buffer.b_ml.ml_line_count {
                if ml_find_line(buffer, lnum, ML_FIND).is_none() {
                    status = Err(MemlineError::PreserveFailed);
                    break 'theend;
                }
                lnum = buffer.b_ml.locked_high() + 1;
            }
            let _ = ml_find_line(buffer, LineNr::ZERO, ML_FLUSH); // flush the locked block
            // Sync the pointer blocks that were just updated.
            if buffer.b_ml.memfile().sync(sync_flags).is_err() {
                status = Err(MemlineError::PreserveFailed);
            }
            buffer.b_ml.stack_clear(); // the stack is invalid now
        }
    }
    got_int.set(got_int.get() | got_int_save);
    status
}
