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

use crate::allocator::Owned;
use crate::autocmd::fire_autocmds_for;
use crate::buffer::{BufFlags, alloc_unregistered_buffer};
use crate::cstr;
use crate::fileio::{Loaded, ReadFlags};
use crate::guard::Suppress;
use crate::memfile::{BlockId, MemFile};
use crate::message_fmt::msg_cstr;
use crate::os::env::{expand_env_into, home_replace_in};
use crate::os::fs::file_info;
use crate::semsg;
use crate::smsg;
use crate::types::String_0;
use core::ffi::{CStr, c_int, c_long, c_uint};
use std::ffi::CString;

use super::*;
use crate::highlight_group::HLF_E;
use crate::types::{FAIL, Failed, MAXPATHL, OK, OptionSetFlags};
use crate::winlayer::{Buf, Win};

/// Read the original file into `curbuf`, starting after line `from`.
///
/// Both callers name `curbuf`'s own full path and pass no `:read` command,
/// which is the whole of `readfile`'s precondition; they differ only in the
/// range and in whether the buffer is new.
fn read_original(
    from: LineNr,
    skip: LineNr,
    lines: LineNr,
    flags: c_int,
) -> Result<Loaded, Failed> {
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

    'theend: {
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
            let Some(chosen) = choose_swapfile(&fname) else {
                break 'theend;
            };
            chosen.as_cstr().to_owned()
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
        let Some(mfp) = MemFile::open(fname_used.clone(), O_RDONLY) else {
            semsg!("E306: Cannot open {}", shown(&fname_used));
            break 'theend;
        };
        buf.b_ml.ml_mfp = Some(mfp);

        let Some(recorded) = read_block_zero(buf, directly) else {
            break 'theend;
        };

        // Given the swap file's name directly, the buffer takes its name
        // from what the swap file says it belongs to.
        if directly {
            let name = cstr::owned(&recorded.fname);
            let path = expand_env_into(&name, MAXPATHL as usize);
            if setfname(Buf::current(), Some(&cstr::owned(&path)), None, true).is_err() {
                break 'theend;
            }
        }

        msg_ext_set_kind(c"wmsg");
        msg_ext_skip_flush.set(true);
        let swap_name = buf.b_ml.mfp().fname().unwrap_or(c"").to_owned();
        let path = home_replace_in(None, &swap_name, MAXPATHL as usize, true);
        smsg!(0, "Using swap file \"{}\"", shown(path.as_cstr()));
        let path = match buf_spname(&Buf::current()) {
            Some(special) => XString::from_cstr(special),
            None => {
                let full = Buf::current().name.full().unwrap_or(c"").to_owned();
                home_replace_in(None, &full, MAXPATHL as usize, true)
            }
        };
        msg_putchar('\n' as c_int);
        smsg!(0, "Original file \"{}\"", shown(path.as_cstr()));
        msg_putchar('\n' as c_int);
        msg_ext_skip_flush.set(false);

        // Compare the dates of the swap file and the original.
        let mtime = recorded.mtime as c_int;
        let original = Buf::current().name.full().and_then(file_info);
        if let Some(original) = original {
            let swap = file_info(&swap_name);
            let newer =
                swap.is_some_and(|swap| original.stat.st_mtim.tv_sec > swap.stat.st_mtim.tv_sec);
            if newer || original.stat.st_mtim.tv_sec != mtime as _ {
                complain(c"E308: Warning: Original file may have been changed");
            }
        }
        ui_flush();

        // Recovery is going ahead, so the buffer's current contents go.
        while !Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY) {
            let _ = ml_delete(1);
        }

        // Read the original file, to pick up 'fileformat', 'fileencoding'
        // and friends. Errors are ignored, and the text itself is not
        // used — except as the "unchanged?" comparison below.
        let mut orig_file_status = Err(Failed);
        if Buf::current().name.full().is_some() {
            orig_file_status = read_original(0, 0, MAXLNUM, READ_NEW as c_int);
        }

        // What the swap file recorded wins over what the file suggests.
        if recorded.fileformat != 0 {
            set_fileformat(recorded.fileformat - 1, OptionSetFlags::LOCAL);
        }
        if let Some(fenc) = &recorded.fileencoding {
            set_option_value_give_err(
                kOptFileencoding,
                OptVal::string(String_0::from_bytes(fenc)),
                OptionSetFlags::LOCAL,
            );
        }
        unchanged(Buf::current(), true, true);

        serious_error = false;
        let Ok((lnum, error)) = recover_lines(buf) else {
            break 'theend;
        };

        // Compare the recovered contents with the original file's.
        // Lines 1 to lnum are what was recovered, lines lnum + 1 to
        // ml_line_count are the file's, and line ml_line_count + 1 is the
        // empty buffer's dummy line.
        if orig_file_status != Ok(Loaded::Read) || Buf::current().b_ml.ml_line_count != lnum * 2 + 1
        {
            // Recovering an empty file gives two lines of which the first
            // is empty; that is not a modification.
            if !(Buf::current().b_ml.ml_line_count == 2
                && Buf::current().lines().line(1).is_empty())
            {
                changed_internal(Buf::current());
                buf_inc_changedtick(Buf::current());
            }
        } else {
            for idx in 1..=lnum {
                // One of the two lines has to be copied: fetching the
                // other flushes the cache the first came out of.
                let recovered = Buf::current().lines().line_copy(idx);
                let same = *recovered == *Buf::current().lines().line(idx + lnum);
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
    }

    msg_ext_skip_flush.set(false);
    recoverymode.set(false);
    if let Some(mut buf) = recovery {
        // Whatever block the walk still held goes back, and the memfile
        // closes; the swap file stays.
        if let Some(locked) = buf.b_ml.unlock() {
            buf.b_ml.put(locked.id, false);
        }
        if let Some(mfp) = buf.b_ml.ml_mfp.take() {
            mfp.close(false);
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
            let name = current.name.shown().map(CStr::to_owned);
            fire_autocmds_for(event, None, name.as_deref(), false, Some(current), None);
        }
    }
}

/// Read and check block zero of the swap file `buf`'s memfile is open on,
/// fixing the memfile's page size from it, and answer what it records.
/// `None`, with the reason said, when the file cannot be recovered from.
///
/// `directly`: the user named the swap file itself, so its recorded file
/// name is wanted.
fn read_block_zero(mut recovery: Buf, directly: bool) -> Option<Recorded> {
    let swap_name = recovery.b_ml.mfp().fname().unwrap_or(c"").to_owned();

    // The page size `MemFile::open` picked need not be the one the swap
    // file was written with; the real one is in block zero. Reading
    // block zero needs *a* page size, so use the smallest one a swap
    // file can have, and correct it below.
    recovery.b_ml.mfp().mf_page_size = MIN_SWAP_PAGE_SIZE;

    let hl_id = HLF_E;
    msg_ext_set_kind(c"emsg");
    let Some(id) = recovery.b_ml.mfp().get(0, 1) else {
        msg_start();
        note(c"Unable to read block 0 from ", hl_id);
        msg_display(&swap_name, hl_id, true);
        note(
            c"\nMaybe no changes were made or Nvim did not update the swap file.",
            hl_id,
        );
        msg_end();
        return None;
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
) -> Option<Recorded> {
    let hl_id = HLF_E;
    let b0 = ZeroBlock(recovery.b_ml.page(id));
    if b0.version().starts_with(b"VIM 3.0") {
        msg_start();
        msg_display(swap_name, 0, true);
        note(c" cannot be used with this version of Nvim.\n", 0);
        note(c"Use Vim version 3.0.\n", 0);
        msg_end();
        return None;
    }
    if !ml_check_b0_id(&b0) {
        semsg!(
            "E307: {} does not look like a Nvim swap file",
            shown(swap_name)
        );
        return None;
    }
    if b0_magic_wrong(&b0) {
        // Only the host name is wanted, and it is terminated inside its own
        // field so that printing it cannot run off the end of a corrupted
        // one.
        let host = cstr::owned(cstr::until_nul(b0.hname()));
        msg_start();
        msg_display(swap_name, hl_id, true);
        note(c" cannot be used on this computer.\n", hl_id);
        note(c"The file was created on ", hl_id);
        msg_str_hl(&host, hl_id, true);
        note(c",\nor the file has been damaged.", hl_id);
        msg_end();
        return None;
    }

    // The guessed page size was wrong, so the highest block number
    // in the file has to be worked out again.
    let recorded_page_size = b0.page_size() as c_uint;
    let mfp = recovery.b_ml.mfp();
    if mfp.mf_page_size != recorded_page_size {
        let previous_page_size = mfp.mf_page_size;
        mfp.set_page_size(recorded_page_size);
        if mfp.mf_page_size < previous_page_size {
            msg_start();
            msg_display(swap_name, hl_id, true);
            note(
                c" has been damaged (page size is smaller than minimum value).\n",
                hl_id,
            );
            msg_end();
            return None;
        }
        let size = mfp.file_size();
        // Zero means no file, or an empty one.
        mfp.mf_blocknr_max = if size <= 0 {
            0
        } else {
            size / FileOffset::from(mfp.mf_page_size)
        };
        mfp.mf_infile_count = mfp.mf_blocknr_max;

        // Block zero's own page was read at the guessed size.
        mfp.resize_page(id, mfp.mf_page_size as usize);
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
    Some(Recorded {
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
/// Returns the name, or `None` to give up.
fn choose_swapfile(fname: &CStr) -> Option<XString> {
    let (count, _) = recover_names(Some(fname), false, None, 0);
    if count == 0 {
        semsg!("E305: No swap file found for {}", shown(fname));
        return None;
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
            return None;
        }
        nr
    };
    recover_names(Some(fname), false, None, nr).1
}

/// Walk the swap file's block tree and append every line it can find to
/// `curbuf`, after line 0.
///
/// `buffer` is the scratch buffer whose memline is open on the swap file and
/// whose `ml_stack` records the descent. Nothing in the file is trusted: a
/// count or a block number that cannot be right costs a `???` line and the
/// walk carries on. Returns the number of lines appended and the number of
/// problems found, or `Err` when block 1 itself is unusable, which leaves
/// nothing to recover.
///
/// The block the walk is on is the recovery memline's locked block, so that
/// the caller releases it whichever way the walk ends.
fn recover_lines(mut buffer: Buf) -> Result<(LineNr, c_int), ()> {
    let mut bnum: BlockNr = 1; // start with block 1
    let mut page_count: c_uint = 1; // which is one page
    let mut lnum: LineNr = 0; // append after line 0 in curbuf
    let mut line_count: LineNr = 0;
    let mut idx: usize = 0; // start with the first index in block 1
    let mut error = 0;
    buffer.b_ml.stack_clear();
    let swap_name = buffer.b_ml.mfp().fname().unwrap_or(c"").to_owned();

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
            let block = buffer.b_ml.mfp().get(bnum, page_count);
            if let Some(id) = block {
                buffer.b_ml.lock(id, 0, 0);
            }
            if block.is_none() {
                if bnum == 1 {
                    semsg!("E309: Unable to read block 1 from {}", shown(&swap_name));
                    return Err(());
                }
                error += 1;
                append_note(&mut lnum, c"???MANY LINES MISSING");
            } else if let Some(id) = block
                && PointerBlock(buffer.b_ml.page(id)).is_pointer()
            {
                // The counts in the header have to fit the page size this
                // build uses, or the entries cannot be walked at all.
                let page_size = buffer.b_ml.mfp().mf_page_size;
                let count_max = PointerBlock::<&[u8]>::count_max_for(page_size);
                let mut pp = PointerBlock(buffer.b_ml.page_mut(id));
                let mut ptr_block_error = false;
                if pp.count_max() != count_max {
                    ptr_block_error = true;
                    pp.set_count_max(count_max);
                }
                if pp.count() > pp.count_max() {
                    ptr_block_error = true;
                    pp.set_count(pp.count_max());
                }
                if ptr_block_error {
                    complain(c"E1364: Warning: Pointer block corrupted");
                }

                let pp = PointerBlock(buffer.b_ml.page(id));
                let count = usize::from(pp.count());
                // The first time down this block, its entries should
                // account for exactly the line count promised above it.
                if idx == 0 && line_count != 0 {
                    line_count -= pp.line_total();
                    if line_count != 0 {
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
                            line_count = pe.pe_line_count;
                            // `pe_line_count` and `pe_old_lnum` come off
                            // disk; readfile() must not be handed either
                            // of them unchecked.
                            if line_count <= 0
                                || pe.pe_old_lnum < 1
                                || read_original(lnum, pe.pe_old_lnum - 1, line_count, 0).is_err()
                            {
                                cannot_open = true;
                            } else {
                                lnum += line_count;
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
                        ip_low: 0,
                        ip_high: 0,
                        ip_index: idx as c_int,
                    };
                    buffer.b_ml.stack_set(top, frame);

                    bnum = pe.pe_bnum;
                    line_count = pe.pe_line_count;
                    page_count = pe.pe_page_count as c_uint;
                    // `pe_page_count` sizes the allocation `get` makes, so a
                    // bogus value (0x40000000, say) would ask for gigabytes.
                    // It must be at least one page, and the block must lie
                    // inside the file.
                    let blocknr_max = buffer.b_ml.mfp().mf_blocknr_max;
                    if page_count < 1 || bnum + BlockNr::from(page_count) > blocknr_max + 1 {
                        error += 1;
                        append_note(&mut lnum, c"???ILLEGAL BLOCK NUMBER");
                        // Skip this entry and pop back up, to recover
                        // whatever else there is.
                        let ip = buffer.b_ml.stack_at(top);
                        idx = ip.ip_index as usize + 1;
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
                    semsg!(
                        "E310: Block 1 ID wrong ({} not a .swp file?)",
                        shown(&swap_name)
                    );
                    return Err(());
                }
                error += 1;
                append_note(&mut lnum, c"???BLOCK MISSING");
            } else if let Some(id) = block {
                // A data block: append every line in it.
                let mut has_error = false;
                let page_size = buffer.b_ml.mfp().mf_page_size;
                let mut dp = DataBlock(buffer.b_ml.page_mut(id));

                // The block's length has to match the page count the
                // pointer block promised; if it does not, trust the
                // pointer block.
                let end = page_count.wrapping_mul(page_size);
                let txt_end_wrong = end != dp.txt_end();
                if txt_end_wrong {
                    dp.set_txt_end(end);
                }

                // Make sure the block ends in a NUL, so that copying the
                // last line out of it cannot run past the end.
                let last = dp.txt_end() as usize - 1;
                dp.write(last, &[0]);

                let block_lines = dp.line_count();
                let (txt_start, txt_end) = (dp.txt_start() as usize, dp.txt_end() as usize);
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
                    let dp = DataBlock(buffer.b_ml.page(id));
                    let start = dp.line_start(i) as usize;
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
                        let text = dp.text(start, txt_end);
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
            idx = ip.ip_index as usize + 1;
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
        msg_outnum(recorded.pid as c_int);
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
            .is_none_or(|mfp| mfp.fname().is_none())
        {
            continue;
        }
        ml_flush_line(buf); // flush the buffered line
        let _ = ml_find_line(buf, 0, ML_FLUSH); // flush the locked block

        if buf_is_changed(buf)
            && check_file != 0
            && buf.b_ml.mfp().need_trans()
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

        if buf.b_ml.mfp().mf_dirty == MfDirty::Yes {
            // Best effort: `ml_sync_all` writes what it can and leaves
            // the rest dirty.
            let stop = if check_char != 0 { MFS_STOP } else { 0 };
            let flush = if do_fsync && buf_is_changed(buf) {
                MFS_FLUSH
            } else {
                0
            };
            let _ = buf.b_ml.mfp().sync(stop | flush);
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
pub fn ml_preserve(mut buffer: Buf, message: bool, do_fsync: bool) {
    if buffer
        .b_ml
        .ml_mfp
        .as_ref()
        .is_none_or(|mfp| mfp.fname().is_none())
    {
        if message {
            complain(c"E313: Cannot preserve, there is no swap file");
        }
        return;
    }

    // Only an interrupt from here on counts, not one from before.
    let got_int_save = got_int.get();
    got_int.set(false);

    ml_flush_line(buffer); // flush the buffered line
    let _ = ml_find_line(buffer, 0, ML_FLUSH); // flush the locked block
    let sync_flags = MFS_ALL | if do_fsync { MFS_FLUSH } else { 0 };
    // `ml_preserve` still answers OK/FAIL to its own callers, so the
    // memfile's result is converted here and again below.
    let mut status = buffer.b_ml.mfp().sync(sync_flags).map_or(FAIL, |()| OK);
    buffer.b_ml.stack_clear(); // the stack is invalid after MFS_ALL

    // Some data blocks may have gone from a negative to a positive block
    // number, which means the pointer blocks referring to them need
    // updating. Which pointer blocks those are is not recorded, so every
    // data block is visited until no translations are left (or the end of
    // the file is reached, which can only happen when a write failed —
    // a full file system, say). `ml_find_line` does the work, translating
    // the negative numbers as it fetches each block's first line.
    'theend: {
        if buffer.b_ml.mfp().need_trans() && !got_int.get() {
            let mut lnum: LineNr = 1;
            while buffer.b_ml.mfp().need_trans() && lnum <= buffer.b_ml.ml_line_count {
                if ml_find_line(buffer, lnum, ML_FIND).is_none() {
                    status = FAIL;
                    break 'theend;
                }
                lnum = buffer.b_ml.locked_high() + 1;
            }
            let _ = ml_find_line(buffer, 0, ML_FLUSH); // flush the locked block
            // Sync the pointer blocks that were just updated.
            if buffer.b_ml.mfp().sync(sync_flags).is_err() {
                status = FAIL;
            }
            buffer.b_ml.stack_clear(); // the stack is invalid now
        }
    }
    got_int.set(got_int.get() | got_int_save);

    if message {
        if status == OK {
            tell(c"File preserved", 0);
        } else {
            complain(c"E314: Preserve failed");
        }
    }
}
